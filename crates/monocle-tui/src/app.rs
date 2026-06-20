//! Application state and IPC message handlers for the monocle TUI.
//!
//! `App` is the central state struct. All IPC handlers receive `&mut App` and
//! update it in place. The render loop reads `App` fields directly.
//!
//! # Idempotency (BC-2.05.002 Invariant 4)
//!
//! `apply_permission_prompt_queued` is the ONLY insertion path for
//! `PromptModal` into `overlay_stack`. It enforces idempotent-on-`prompt_id`
//! semantics: if the prompt_id is already present, the duplicate is silently
//! discarded. This applies to both `InitialState.overlay_stack` population and
//! streaming `PermissionPromptQueued` handling (S-026 reuses this helper).

use anyhow::{Context, Result};
use directories::ProjectDirs;
use monocle_config::{detect_ccr, load_config, write_config, MonocleConfig};
use monocle_core::engine::EnrichedSession;
use monocle_core::pty_constants::{
    DUMP_WINDOW_TIMEOUT, MAX_PENDING_PTY_BYTES, MAX_PENDING_PTY_MESSAGES,
};
use monocle_core::tui::state::{
    clamp_scrollback_rows, default_scrollback_rows, AppMode, FocusSnapshot, PromptModal,
    ToolPayload, PTY_DEFAULT_COLS, PTY_DEFAULT_ROWS,
};
use monocle_ipc::error::IpcError;
use monocle_ipc::framing::read_framed;
use monocle_ipc::reconnect::{BackoffState, RECONNECT_WINDOW_SECS};
use monocle_ipc::types::{
    ClientToServer, HookEventRecord, HookType, PermissionPromptPayload, ServerToClient,
};
// S-039: vt100 parser for PTY output pipeline (BC-2.09.001 / SS-embedded-pty.md §Parser ownership).
// vt100::Parser is an effectful-shell type (stateful mutation via .process()); it lives in the
// effectful monocle-tui crate, NOT in pure monocle-core.
// PermissionDecisionKind: re-exported from lib.rs for integration tests and for S-026
// decision handler implementations. The re-export here ensures the type is visible at
// the app module level when todo!() stubs are replaced with real code.
pub use monocle_ipc::types::PermissionDecisionKind;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// TransportEvent re-export (S-023 canonical type)
// ---------------------------------------------------------------------------

// `TransportEvent` is defined in `monocle-ipc::events` (S-023, BC-2.05.007).
// Re-exported here so that integration tests (which import from `monocle_tui::app`)
// continue to work without import changes. The local stub that existed before
// S-023 merged has been removed per MERGE-COORDINATION (F-S025-ADV2-MED-003).
pub use monocle_ipc::events::TransportEvent;

// ---------------------------------------------------------------------------
// App constants
// ---------------------------------------------------------------------------

/// Capacity of the in-process event ring buffer (matching the daemon's RAM ring).
///
/// Mirrors `monocle_runtime::ring::RAM_RING_CAPACITY` (4096) per BC-2.04.012 PC-1.
/// The TUI-side ring must not exceed the daemon-side ring size — there is no value
/// in holding more events than the daemon can produce. Overflow eviction is FIFO;
/// evicted entries are NOT counted in `App::drop_counter` (that counter tracks IPC
/// channel packet drops, not ring evictions).
///
/// Do NOT import `monocle_runtime::ring::RAM_RING_CAPACITY` directly — that would
/// create a monocle-tui → monocle-runtime dependency not in the current dep graph.
pub const EVENT_RING_CAPACITY: usize = 4096;

/// Canonical error message shown when the TUI fails to connect to the daemon.
///
/// This is the single source of truth for AC-002 (BC-2.06.004 PC-1). Both the
/// production `run()` path and the AC-002 integration test reference this const —
/// eliminating any possibility of vacuous-mirror drift between test and production.
///
/// Any change to this string must be accompanied by a BC-2.06.004 version bump.
pub const DAEMON_NOT_RUNNING_ERROR: &str =
    "Daemon not running. Start it with: monocle daemon start";

/// Status bar text shown when the IPC transport disconnects and the TUI enters
/// the reconnect wait state (BC-2.06.004 + BC-2.05.007 on_transport_event path).
///
/// Single source of truth — both the production `on_transport_event()` path and
/// any integration test referencing this state must use this const.
pub const DAEMON_DISCONNECT_STATUS: &str = "[disconnected] reconnecting...";

/// Status bar text shown when the daemon is unreachable after reconnect attempts
/// exhaust (offline-mode fallback — BC-2.05.006 PC-5 / `reconnect_to_daemon` timeout path).
///
/// Duplicated inline at both protocol-violation and reader-disconnect call sites;
/// this const is the single authoritative source for both.
pub const DAEMON_OFFLINE_STATUS: &str = "[daemon: offline]";

/// Base status-bar product name label (legacy — superseded by `render_status_bar`).
///
/// This constant was used by the S-025 one-row `Paragraph` renderer in `render_frame`
/// (which rendered `"monocle"` or `"[dropped: N] monocle"`). S-027 replaced that path
/// with `render_status_bar`, which no longer uses this label.
///
/// Retained as a public re-export because `startup_connect.rs` tests assert the
/// ABSENCE of the old format (regression guard), which requires the constant to
/// compile. Do NOT use this in new rendering code.
pub const MONOCLE_STATUS_LABEL: &str = "monocle";

/// Format the legacy S-025 drop-counter label `"[dropped: N] monocle"`.
///
/// S-027 superseded this with `render_status_bar` / `drop_counter_span` which renders
/// the canonical BC-2.06.019 PC-2 text `"drops: N"` (no brackets, no product name).
///
/// Retained as a public function because `startup_connect.rs` tests assert the
/// ABSENCE of the old format (negative regression guard) — they call this function
/// to produce the expected-absent string. Do NOT use in new rendering code.
pub fn format_drop_counter(n: u64) -> String {
    format!("[dropped: {n}] {MONOCLE_STATUS_LABEL}")
}

// ---------------------------------------------------------------------------
// AppEvent — internal application events (S-039 adversarial Pass-4, F-PASS4-MED-001)
// ---------------------------------------------------------------------------

/// Internal application events delivered from spawned tasks into the app event channel.
///
/// The app event channel (`app_event_tx` / receiver in the run loop) lets background
/// tasks (e.g., dump-window timeout sleeps) deliver events to the main event loop
/// without going through the IPC reader channel.
///
/// MUST NOT carry large payloads — these events are lightweight signals only.
/// The run loop drains the channel each tick alongside `ipc_rx`.
#[non_exhaustive]
pub enum AppEvent {
    /// The dump-window timeout elapsed for a session without receiving
    /// `ScrollbackDumpComplete`. The event loop routes this to
    /// [`on_dump_window_timeout`].
    ///
    /// F-PASS4-MED-001 / BC-2.09.001 Invariant 5 timeout path.
    DumpWindowTimeout {
        /// Session ID whose dump window elapsed.
        session_id: String,
    },
}

// ---------------------------------------------------------------------------
// ProfilePickerState (S-031, BC-2.07.004/005)
// ---------------------------------------------------------------------------

/// Transient state for the profile picker modal (AC-001, BC-2.07.004 PC-1).
///
/// Stored as `App::profile_picker: Option<ProfilePickerState>`.
/// MUST NOT be modeled as an `AppMode` variant — it is an orthogonal overlay that
/// can appear over any `AppMode` (AC-008, BC-2.07.004 INV-1 / BC-2.07.005 INV-4).
///
/// Populated when `Action::ProfilePicker` fires (Ctrl-P):
/// - `profiles` is a sorted snapshot of `config.harness_profiles[*].id` at open time.
/// - `selected_index` tracks the highlighted row (0-based, wraps on j/k).
///
/// Dismissed (set to `None`) on:
/// - `Esc` — closes without change.
/// - `Enter` — selects, triggers persistence + CCR re-detect, then closes.
#[derive(Clone, Debug)]
pub struct ProfilePickerState {
    /// Index of the currently highlighted row (0-based, wraps on navigation).
    pub selected_index: usize,
    /// Snapshot of profile IDs at picker-open time, sorted alphabetically.
    /// Immutable for the lifetime of one picker session (AC-002 / BC-2.07.005 EC-112).
    pub profiles: Vec<String>,
    /// The directory the picker was opened for — used by the widget to mark the
    /// per-directory active profile with `"* "` (BC-2.07.004 PC-2 / BC-2.07.005 PC-2).
    /// Set to the verbatim CWD at open time; never canonicalized (BC-2.07.004 INV-1).
    pub current_dir: String,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

/// Central TUI application state (S-025, BC-2.06.004, BC-2.06.005, BC-2.06.007).
///
/// Every field is `pub` so that downstream stories (S-026 permission overlay,
/// S-027 status bar, S-028 sessions filter) can read and extend without
/// re-declaring visibility.
#[non_exhaustive]
pub struct App {
    /// Current TUI state machine mode.
    pub mode: AppMode,
    /// Loaded monocle configuration (loaded via load_config() at startup).
    pub config: MonocleConfig,
    /// Live session list sourced from `ServerToClient::SessionListUpdate`.
    pub sessions: Vec<EnrichedSession>,
    /// Cumulative IPC event drop counter from `ServerToClient::DropCounterUpdate`.
    pub drop_counter: u64,
    /// Local copy of the permission prompt overlay stack.
    ///
    /// Populated via `apply_permission_prompt_queued` — NEVER push directly.
    /// The VecDeque (not Option<PromptModal>) is the canonical pattern per
    /// SS-conventions-anti-patterns.md §forbidden-patterns.
    pub overlay_stack: VecDeque<PromptModal>,
    /// Optional status bar notification message (e.g., `"[disconnected] reconnecting..."`).
    ///
    /// Set by transport event handlers; cleared when normal operation resumes.
    /// `None` means no notification; `Some(msg)` is rendered in the status bar.
    pub status_message: Option<String>,
    /// Recent hook events from the daemon RAM ring, seeded from `InitialState::ring_tail`
    /// and extended by subsequent push messages (S-027).
    ///
    /// Bounded to [`EVENT_RING_CAPACITY`] entries — same as the daemon's RAM ring
    /// (BC-2.04.012 PC-1, `ring.rs::RAM_RING_CAPACITY = 4096`). Oldest entries are
    /// evicted on overflow; evicted entries are NOT counted in `App::drop_counter`
    /// (that counter tracks IPC channel packet drops, not ring evictions).
    pub event_ring: VecDeque<HookEventRecord>,

    // -----------------------------------------------------------------------
    // S-028 fields: sessions filter (BC-2.06.006) + event ribbon (BC-2.06.018)
    // -----------------------------------------------------------------------
    /// Shared nucleo fuzzy matcher for the sessions filter panel (BC-2.06.006 INV-1).
    ///
    /// Instantiated once at startup in `App::new()` and reused across all filter
    /// keystrokes. MUST NOT be recreated per keystroke — recreating resets internal
    /// caches and degrades performance at the P0 60fps target (BC-2.06.006 AC-005,
    /// INV-1). The matcher is held here (not in `AppMode::Filtering`) to satisfy the
    /// "shared Matcher" architecture constraint in S-028 §Architecture Compliance Rules.
    pub matcher: nucleo::Matcher,

    /// All hook events received from the daemon, across all sessions (BC-2.06.018).
    ///
    /// Populated from two sources (BC-2.05.002 + BC-2.05.004):
    /// 1. `InitialState::ring_tail` (on connect) — backfills historical events via
    ///    `on_initial_state_event_ribbon()`.
    /// 2. `ServerToClient::HookEventReceived` messages (streaming) — appended via
    ///    `on_hook_event_received()`.
    ///
    /// Contains events for ALL sessions. The Event Ribbon panel filters client-side
    /// by the selected `session_id` at render time (BC-2.05.004 INV-3: no IPC-layer
    /// filtering; BC-2.06.018 INV-1: no new IPC request on session-change).
    ///
    /// The `VecDeque` is bounded to `panel_height` entries (determined at render time)
    /// per BC-2.06.018 PC-3. Oldest entries are popped from the back when full.
    pub event_ribbon_events: VecDeque<crate::ui::event_ribbon::HookEventRow>,

    /// Event ribbon scroll and pin state (BC-2.06.018 PC-5/PC-8/INV-1/AC-009).
    ///
    /// Stored in `App` (not in `render_frame` locals) so that:
    /// 1. `on_hook_event_received` can auto-scroll to row 0 when `!pinned_top` (AC-008).
    /// 2. `dispatch_key_event` calls `reset_on_session_change` in the `ScrollDown`/`ScrollUp`
    ///    arms when the selected session changes (BC-2.06.018 INV-1 / AC-009).
    pub event_ribbon_state: crate::ui::event_ribbon::EventRibbonState,

    /// Last rendered event ribbon panel height (rows), captured in `render_frame`.
    ///
    /// Used as the dynamic cap for `push_event_row` in `on_hook_event_received`
    /// (BC-2.06.018 PC-3: VecDeque bounded to `panel_height`). Initialises to
    /// `EVENT_RING_CAPACITY` so the first push before any render sees a safe cap.
    /// Updated each frame in `render_frame` when the EventRibbon widget is rendered.
    pub event_ribbon_panel_height: usize,

    // -----------------------------------------------------------------------
    // S-031 fields: profile picker (BC-2.07.004/005) + CCR path (BC-2.07.005)
    // -----------------------------------------------------------------------
    /// Profile picker transient overlay state (S-031, BC-2.07.004/005).
    ///
    /// `None` means the picker is closed. `Some(state)` means the picker is open
    /// and rendering a centered modal over the current view. This field is orthogonal
    /// to `App::mode` — the picker can appear over any `AppMode` (BC-2.07.005 INV-4).
    ///
    /// Set to `Some(ProfilePickerState { .. })` by the `Action::ProfilePicker` handler.
    /// Set to `None` by the `Esc` handler (no change) or `Enter` handler (after profile switch).
    ///
    /// MUST NOT use `AppMode::Overlay` for the profile picker — that variant is reserved
    /// for permission prompts (AC-008 / BC-2.07.004 INV-1 / BC-2.07.005 INV-4).
    pub profile_picker: Option<ProfilePickerState>,

    /// Resolved CCR binary path (S-031, BC-2.07.005 PC-3 / AC-007).
    ///
    /// Populated at startup via `detect_ccr(&config)` and updated after every
    /// successful profile switch. `None` means CCR is not detected (either not
    /// configured or not on PATH). Used by the status bar footer to render
    /// `"CCR: <path>"` or `"CCR: none"`.
    ///
    /// Distinct from `MonocleConfig::ccr_path` (an explicit override string) —
    /// this is the RESOLVED executable path returned by `monocle_config::detect_ccr`.
    pub ccr_path: Option<std::path::PathBuf>,

    /// Sender half of the IPC outbound channel for dispatching `ClientToServer`
    /// messages to the daemon (S-026, BC-2.06.011/012/013).
    ///
    /// `None` on construction; set by the run loop after the IPC connection is
    /// established and before the main event loop starts. Decision key handlers
    /// (`PermissionAcceptOnce`, `PermissionAcceptAlways`, `PermissionReject`) send
    /// via this channel using `try_send` (bounded, non-blocking — BC-2.04.011).
    ///
    /// Using `Option` rather than a unit-struct sentinel avoids phantom-send bugs:
    /// a handler that attempts to send before the channel is wired will produce a
    /// tracing WARN rather than silently discarding the message.
    pub ipc_tx: Option<tokio::sync::mpsc::Sender<ClientToServer>>,

    /// Pending key prefix state for multi-keystroke sequences (BC-2.06.018 PC-5 / AC-007).
    ///
    /// Used to implement the vi-style `gg` (jump to newest) two-keystroke sequence:
    /// - First `g` keypress (in `Dashboard { focused: EventRibbon }`): sets `pending_key = Some('g')`.
    /// - Second `g` keypress while `pending_key == Some('g')`: fires `jump_newest` and clears.
    /// - Any other key while pending: `pending_key` is cleared and the key is processed normally.
    ///
    /// `None` means no pending prefix (the common case). Only meaningful in
    /// `Dashboard { focused: EventRibbon }` context; other modes do not set this field.
    pub pending_key: Option<char>,

    /// Session ID of the currently selected session in the Sessions panel.
    ///
    /// Used by `on_hook_event_received` to gate auto-scroll (BC-2.06.018 AC-008 / PC-8):
    /// auto-scroll to row 0 fires ONLY when the incoming event's session_id matches
    /// the currently selected session. An event for a non-selected session must NOT
    /// disturb the scroll position of the selected session's ribbon view.
    ///
    /// # Semantics
    ///
    /// `None` means no explicit selection has been recorded (e.g., before the first
    /// render populates the sessions list). When `None`, `on_hook_event_received` falls
    /// back to `app.sessions.first()` as the effective selected session so that the
    /// auto-scroll behavior is consistent with the Sessions panel default (first session
    /// highlighted on startup).
    ///
    /// Set by `dispatch_key_event` when `ScrollDown`/`ScrollUp` advance the Sessions
    /// cursor (Dashboard/Sessions focus: j/k/↓/↑ resolve to these actions via per-context
    /// binding — `SelectNext`/`SelectPrev` are unreachable in Dashboard mode per ADV Pass-6).
    /// Set by `render_frame` after the sessions list is (re-)populated to track the
    /// `SessionsPanelState::list_state.selected()` index.
    pub selected_session_id: Option<String>,

    // -----------------------------------------------------------------------
    // S-039: PTY output pipeline fields (BC-2.09.001)
    // -----------------------------------------------------------------------
    /// Per-session `vt100::Parser` instances (effectful shell — SS-embedded-pty.md §Parser ownership).
    ///
    /// Keyed by session UUID string. Populated when a session is added (via `SessionListUpdate`
    /// or `InitialState`). Removed on session GC (`SessionState::Terminated` + list removal).
    ///
    /// Classified as EFFECTFUL because `vt100::Parser::process()` is a stateful mutation that
    /// drives an internal terminal state machine. Only `monocle-tui` holds this map.
    pub pty_parsers: HashMap<String, vt100::Parser>,

    /// Per-session PTY scroll offsets (pure core — in-memory usize, no I/O).
    ///
    /// Keyed by session UUID string. Initialised to 0 for each session. Reset on resize
    /// (S-042 owns the `ResizePane` handler that writes the reset). Read at render time by
    /// `render_embedded_terminal`. S-043 consumes this value for scrollback navigation.
    pub pty_scroll_offsets: HashMap<String, usize>,

    /// Sessions for which a full scrollback dump has been received in the current TUI lifetime
    /// (pure core — in-memory HashSet, no I/O).
    ///
    /// A session_id is INSERTED here on receipt of `ServerToClient::ScrollbackDumpComplete`
    /// for that session. It is REMOVED when the user detaches from a session (via
    /// `exit_embedded_terminal`), so that the next `enter_embedded_terminal` call for the
    /// same session re-runs the full attach + dump protocol (AC-005 re-attach clause).
    ///
    /// Distinguished from `dump_in_progress`: this field marks "a complete dump was received
    /// for this continuous attach period"; `dump_in_progress` marks "a dump is currently in
    /// flight". Both fields exist and MUST NOT be conflated (SS-embedded-pty.md §Auto-attach mandate).
    pub pty_dump_received: HashSet<String>,

    /// In-flight dump signal per session (pure core — in-memory HashMap<String, bool>).
    ///
    /// Set to `true` BEFORE `ClientToServer::AttachSession` is sent (not on first chunk receipt).
    /// Live `PtyOutput` arriving while `dump_in_progress[session_id] == true` is BUFFERED in
    /// `pending_pty_bytes`, not fed to the parser.
    /// Set to `false` after `ScrollbackDumpComplete` replay is complete.
    ///
    /// Invariant: `dump_in_progress` is set BEFORE `AttachSession` is sent — any `PtyOutput`
    /// arriving after the send (before the first chunk) is buffered correctly.
    pub dump_in_progress: HashMap<String, bool>,

    /// Buffered PTY bytes received while a scrollback dump is in progress (pure core).
    ///
    /// Keyed by session UUID string. Each inner `Vec<u8>` is one `PtyOutput` message's bytes.
    /// The outer `Vec` preserves receipt order. After `ScrollbackDumpComplete`, all buffered
    /// bytes are replayed through the reset parser in receipt order, then the buffer is cleared.
    pub pending_pty_bytes: HashMap<String, Vec<Vec<u8>>>,

    /// Configured PTY scrollback row count (pure core — loaded from config at startup).
    ///
    /// Sourced from `~/.monocle/config.json:pty_scrollback_rows` (default 1000 if absent or
    /// invalid; maximum 10000; values above 10000 are clamped). Owned by S-039.
    /// S-043 reads this value via `app.scrollback_rows`; it does NOT re-load it.
    pub scrollback_rows: u16,

    // -----------------------------------------------------------------------
    // S-039 adversarial Pass-4: pending buffer caps + dump-window timeout handles
    // F-PASS4-MED-001 — bound pending_pty_bytes (cap + timeout)
    // -----------------------------------------------------------------------
    /// Per-session drop counter for `pending_pty_bytes` overflow evictions (pure core).
    ///
    /// Incremented each time `on_pty_output` evicts the OLDEST entry from
    /// `pending_pty_bytes[session_id]` due to the byte-cap
    /// (`MAX_PENDING_PTY_BYTES`) or message-cap (`MAX_PENDING_PTY_MESSAGES`)
    /// being exceeded.
    ///
    /// Rendered in the status bar as `"[dump: N drops]"` when
    /// `dump_in_progress[focused] == Some(true)` AND `N > 0`.
    ///
    /// F-PASS4-MED-001 / BC-2.09.001 Invariant 5 cap.
    pub pending_pty_drop_count: HashMap<String, u64>,

    /// Per-session abort handles for the dump-window timeout tasks (effectful shell).
    ///
    /// On each successful `AttachSession` send (Ok path only), `enter_embedded_terminal`
    /// spawns `tokio::time::sleep(DUMP_WINDOW_TIMEOUT)` and stores the `AbortHandle`
    /// here keyed by `session_id`.
    ///
    /// The handle is cancelled (`.abort()`) in `on_scrollback_dump_complete` when the
    /// dump completes normally (preventing a spurious `DumpWindowTimeout` event after
    /// the dump arrives). On timeout the task delivers
    /// `AppEvent::DumpWindowTimeout { session_id }` into the app event channel and
    /// the handle is removed from this map.
    ///
    /// On disconnect (`on_transport_event` Disconnected), ALL handles are aborted and
    /// the map is drained (F-PASS4-MED-002).
    ///
    /// Classified as EFFECTFUL: `tokio::task::AbortHandle` requires a running tokio
    /// runtime. Lives in monocle-tui only (never in pure monocle-core).
    pub dump_timeout_handles: HashMap<String, tokio::task::AbortHandle>,

    /// Sender half of the internal app event channel (effectful shell).
    ///
    /// Set by the run loop after construction (before the main event loop). Background
    /// tasks (e.g., dump-window timeout sleeps) clone this sender to deliver
    /// `AppEvent` variants to the main event loop.
    ///
    /// `None` on construction; set to `Some(tx)` by the run loop at startup.
    /// `enter_embedded_terminal` guards on `Some(ref tx)` before spawning the
    /// timeout task — if `None` (e.g., unit tests that do not run the full event loop),
    /// no timeout is spawned and no abort handle is stored.
    pub app_event_tx: Option<tokio::sync::mpsc::Sender<AppEvent>>,
}

impl App {
    /// Construct a default `App` from the provided config.
    ///
    /// Starts in `Dashboard { focused: Sessions }` with empty collections.
    /// The `nucleo::Matcher` is initialized once here (BC-2.06.006 AC-005 / INV-1:
    /// shared Matcher — NOT recreated per keystroke).
    /// `ccr_path` is initialized at construction via `detect_ccr(&config)` so the
    /// status bar footer shows the CCR path from the very first render (AC-007 / AC-010 /
    /// BC-2.07.005 PC-3). Callers do NOT need to set `app.ccr_path` after construction.
    pub fn new(config: MonocleConfig) -> Self {
        let ccr_path = detect_ccr(&config);
        // AC-008 / BC-2.09.001 Invariant 4: derive scrollback_rows from config.
        // `None` (field absent from JSON) → apply 1000-row default.
        // Non-None values are clamped to [1, 10000] by clamp_scrollback_rows.
        // Both helpers live in monocle-core (pure arithmetic, no I/O).
        let scrollback_rows = match config.pty_scrollback_rows {
            None => default_scrollback_rows(),
            Some(raw) => clamp_scrollback_rows(raw),
        };
        Self {
            mode: AppMode::Dashboard {
                focused: FocusSnapshot::Sessions,
            },
            ccr_path,
            config,
            sessions: Vec::new(),
            drop_counter: 0,
            overlay_stack: VecDeque::new(),
            status_message: None,
            event_ring: VecDeque::with_capacity(EVENT_RING_CAPACITY),
            // S-028: nucleo Matcher initialized once at startup (BC-2.06.006 INV-1).
            // nucleo::Config::DEFAULT is the standard configuration for case-insensitive
            // fuzzy matching (SS-deps-pin-manifest.md §nucleo 0.5 / ADR-0002).
            matcher: nucleo::Matcher::new(nucleo::Config::DEFAULT),
            // S-028: event ribbon event log for all sessions (BC-2.06.018).
            // Initially empty; populated by on_initial_state (ring_tail) and
            // on_hook_event_received (streaming). No pre-allocated capacity because
            // the effective cap is dynamic (panel_height, determined at render time).
            event_ribbon_events: VecDeque::new(),
            // S-028: ribbon scroll/pin state stored in App so dispatch and IPC handlers
            // can mutate it (auto-scroll, session-change reset — BC-2.06.018 PC-5/PC-8/INV-1).
            event_ribbon_state: crate::ui::event_ribbon::EventRibbonState::default(),
            // S-028: initial cap is EVENT_RING_CAPACITY so the first push before any render is safe.
            // Updated each frame by render_frame to the actual event_ribbon_area height.
            event_ribbon_panel_height: EVENT_RING_CAPACITY,
            // S-031: profile picker closed at startup (BC-2.07.004 INV-1).
            profile_picker: None,
            ipc_tx: None,
            // BC-2.06.018 PC-5 / AC-007: no pending key prefix initially.
            // Set to Some('g') on first 'g' press in Dashboard { EventRibbon } focus;
            // cleared on second 'g' (fires gg jump) or any other key.
            pending_key: None,
            // BC-2.06.018 AC-008 / PC-8: no selected session initially.
            // on_hook_event_received falls back to sessions.first() when None.
            // Set by dispatch_key_event (SelectNext/SelectPrev) and render_frame.
            selected_session_id: None,

            // S-039: PTY output pipeline fields (BC-2.09.001).
            // Parsers are created per-session in on_initial_state / on_session_list_update.
            pty_parsers: HashMap::new(),
            pty_scroll_offsets: HashMap::new(),
            pty_dump_received: HashSet::new(),
            dump_in_progress: HashMap::new(),
            pending_pty_bytes: HashMap::new(),
            // AC-008 / BC-2.09.001 Invariant 4: computed above from config.pty_scrollback_rows.
            // None → 1000 (default_scrollback_rows); Some(raw) → clamp_scrollback_rows(raw).
            scrollback_rows,

            // S-039 adversarial Pass-4: buffer-cap drop counters + dump-window timeout handles.
            // F-PASS4-MED-001: both maps start empty; populated on first on_pty_output overflow
            // (drop_count) and on each successful AttachSession send (timeout_handles).
            pending_pty_drop_count: HashMap::new(),
            dump_timeout_handles: HashMap::new(),
            // app_event_tx: None at construction; set by the run loop before the event loop.
            // Unit tests that do not run the full event loop leave this as None, which
            // prevents timeout tasks from being spawned (no-op guard in enter_embedded_terminal).
            app_event_tx: None,
        }
    }
}

// ---------------------------------------------------------------------------
// S-039: PTY output pipeline methods (BC-2.09.001)
// ---------------------------------------------------------------------------

/// Handle a `ServerToClient::PtyOutput` IPC message.
///
/// Called from `handle_server_message` on receipt of `PtyOutput`.
/// If a scrollback dump is in progress for this session, the bytes are buffered in
/// `pending_pty_bytes` rather than fed to the parser (BC-2.09.001 Invariant 5).
/// Otherwise, `pty_parsers[session_id].process(&bytes)` is called.
///
/// # Edge case EC-200 (BC-2.09.001)
///
/// If `session_id` is absent from `pty_parsers` (race during session creation), bytes
/// are silently dropped for this tick. No panic. TRACE log only (not WARN).
///
pub fn on_pty_output(app: &mut App, session_id: String, bytes: Vec<u8>) {
    // BC-2.09.001 Invariant 5: if a scrollback dump is in progress, buffer bytes.
    if app
        .dump_in_progress
        .get(&session_id)
        .copied()
        .unwrap_or(false)
    {
        app.pending_pty_bytes
            .entry(session_id.clone())
            .or_default()
            .push(bytes);

        // F-PASS4-MED-001: enforce byte-cap and message-cap on the pending buffer.
        // Drop OLDEST (front) entries first until both caps are satisfied.
        // Increment pending_pty_drop_count for each evicted entry.
        let buffer = app.pending_pty_bytes.entry(session_id.clone()).or_default();
        loop {
            // Check message-count cap.
            let over_msgs = buffer.len() > MAX_PENDING_PTY_MESSAGES;
            // Check byte-volume cap (sum of all entry lengths).
            let total_bytes: usize = buffer.iter().map(|v| v.len()).sum();
            let over_bytes = total_bytes > MAX_PENDING_PTY_BYTES;

            if !over_msgs && !over_bytes {
                break;
            }
            // Drop the OLDEST entry (index 0 = first inserted = oldest).
            buffer.remove(0);
            *app.pending_pty_drop_count
                .entry(session_id.clone())
                .or_default() += 1;
        }

        return;
    }

    // BC-2.09.001 EC-200: unknown session_id → silent drop (TRACE only, no panic).
    let Some(parser) = app.pty_parsers.get_mut(&session_id) else {
        tracing::trace!(
            session_id = %session_id,
            "on_pty_output: session not in pty_parsers — bytes silently dropped (EC-200)"
        );
        return;
    };

    // BC-2.09.001 PC-2: feed bytes to the parser.
    parser.process(&bytes);
}

/// Transition to `AppMode::EmbeddedTerminal` for `session_id`.
///
/// If the session is NOT present in `pty_dump_received` (i.e., no complete dump has been
/// received in this attach period), runs the auto-attach-on-first-entry protocol:
/// 1. Sets `dump_in_progress[session_id] = true` BEFORE awaiting `AttachSession` send.
/// 2. Sends `ClientToServer::AttachSession { session_id }` via `app.ipc_tx.send().await`.
///    On `Err(_)` (channel closed): FULL ROLLBACK — `dump_in_progress.remove(&id)`, mode
///    is NOT transitioned, error surfaced via `tracing::error!`, returns early.
/// 3. On `Ok(())`: transitions `app.mode` to `AppMode::EmbeddedTerminal { session_id, prior }`.
///
/// If the session IS already in `pty_dump_received`, transitions directly to
/// `AppMode::EmbeddedTerminal` (parser is already populated; O(1) — BC-2.09.001 AC-004).
///
/// # Architect ruling (F-S039-004)
///
/// This function uses `.send().await` (NOT `try_send`) per the architect's ruling.
/// This provides backpressure and prevents silent drops of `AttachSession` commands.
/// The call site (Action::EnterEmbeddedTerminal dispatch arm) MUST `.await` this function.
pub async fn enter_embedded_terminal(app: &mut App, session_id: String) {
    // BC-2.09.001 PC-6 / SS-embedded-pty.md §Auto-attach mandate (I11-001 PRONG A):
    // If the session has NOT received a ScrollbackDumpComplete in this TUI lifetime,
    // run the full auto-attach + buffering + dump protocol.
    if !app.pty_dump_received.contains(&session_id) {
        // F-S039-004 step 1: set dump_in_progress = true BEFORE the await.
        // Live PtyOutput may arrive between the await and the first ScrollbackChunk —
        // buffering MUST start immediately, not on first chunk receipt.
        app.dump_in_progress.insert(session_id.clone(), true);

        // F-S039-004 step 2: send AttachSession via .send().await (NOT try_send).
        // Provides backpressure; prevents silent drops of AttachSession commands.
        //
        // F-S039-P2-001: ipc_tx == None (daemon offline / reconnecting) MUST be treated
        // identically to a send Err — FULL ROLLBACK in both cases. The `let Some` guard
        // exits early with rollback when the channel is not yet wired or has gone offline,
        // preventing dump_in_progress from being left true with no AttachSession in flight.
        let Some(ref tx) = app.ipc_tx else {
            // None path: IPC channel offline — identical rollback to the Err branch.
            app.dump_in_progress.remove(&session_id);
            tracing::error!(
                session_id = %session_id,
                "enter_embedded_terminal: AttachSession not sent — IPC channel offline; \
                 dump_in_progress rolled back, mode NOT transitioned (F-S039-P2-001)"
            );
            return;
        };

        if let Err(e) = tx
            .send(ClientToServer::AttachSession {
                session_id: session_id.clone(),
            })
            .await
        {
            // F-S039-004 step 3 (Err path): FULL ROLLBACK.
            // dump_in_progress is removed; AppMode is NOT transitioned.
            app.dump_in_progress.remove(&session_id);
            tracing::error!(
                session_id = %session_id,
                error = %e,
                "enter_embedded_terminal: AttachSession send failed (channel closed) — \
                 dump_in_progress rolled back, mode NOT transitioned (F-S039-004)"
            );
            return;
        }
        // F-S039-004 step 3 (Ok path): AttachSession sent — proceed to mode transition.
        //
        // F-PASS4-MED-001: spawn a dump-window timeout task.
        // If app_event_tx is wired (run-loop path), spawn the sleep and store the
        // AbortHandle. If None (unit-test path without run loop), skip silently.
        if let Some(ref event_tx) = app.app_event_tx {
            let sid = session_id.clone();
            let tx_clone = event_tx.clone();
            let timeout_task = tokio::spawn(async move {
                tokio::time::sleep(DUMP_WINDOW_TIMEOUT).await;
                // Deliver timeout event into the app event channel.
                // If the channel is full or closed, log and discard (idempotency
                // guard in on_dump_window_timeout handles duplicate/late events).
                if let Err(e) = tx_clone
                    .send(AppEvent::DumpWindowTimeout {
                        session_id: sid.clone(),
                    })
                    .await
                {
                    tracing::warn!(
                        session_id = %sid,
                        error = %e,
                        "dump-window timeout: app_event_tx send failed — event lost \
                         (dump state may need manual cleanup)"
                    );
                }
            });
            let abort_handle = timeout_task.abort_handle();
            // F-S039-P5-004: abort any in-flight timeout for this session before
            // inserting the new handle. Dropping an AbortHandle does NOT abort the
            // task — explicit abort() is required to prevent orphaned sleep tasks that
            // would fire a spurious DumpWindowTimeout against a newer attach window.
            if let Some(old_handle) = app.dump_timeout_handles.remove(&session_id) {
                old_handle.abort();
            }
            app.dump_timeout_handles
                .insert(session_id.clone(), abort_handle);
        }
    }
    // BC-2.09.001 AC-004 O(1) path: session already dumped → transition directly.
    // No AttachSession, no dump_in_progress change.

    // Determine prior focus for restoration on exit.
    let prior = match &app.mode {
        AppMode::Dashboard { focused } => focused.clone(),
        _ => FocusSnapshot::Sessions,
    };

    // Transition to EmbeddedTerminal mode.
    app.mode = AppMode::EmbeddedTerminal { session_id, prior };
}

/// Transition out of `AppMode::EmbeddedTerminal`, restoring the prior `Dashboard` focus.
///
/// Removes `session_id` from `pty_dump_received` so that the NEXT call to
/// `enter_embedded_terminal` for the same session re-runs the full attach + dump protocol
/// (AC-005 re-attach clause / BC-2.09.001 AC-005).
pub fn exit_embedded_terminal(app: &mut App, session_id: &str) {
    // BC-2.09.001 AC-005 re-attach clause: remove session_id from pty_dump_received.
    // This ensures the NEXT enter_embedded_terminal call for the same session re-runs
    // the full attach + dump protocol (fresh dump from daemon side per S-047 AC-006).
    app.pty_dump_received.remove(session_id);

    // Restore prior AppMode (Dashboard with prior focus).
    let prior = match &app.mode {
        AppMode::EmbeddedTerminal { prior, .. } => prior.clone(),
        _ => FocusSnapshot::Sessions,
    };
    app.mode = AppMode::Dashboard { focused: prior };
}

/// Handle `ServerToClient::ScrollbackDumpComplete` for a session.
///
/// Implements exactly 5 steps per architect ruling (F-S039-005/006):
/// 1. `pty_parsers[id] = vt100::Parser::new(pty_rows, pty_cols, scrollback_rows)` using
///    `pty_rows`/`pty_cols` FROM the `ScrollbackDumpComplete` message.
/// 2. Replay `pending_pty_bytes[id]` in receipt order via `parser.process(&chunk)`.
/// 3. `pending_pty_bytes[id].clear()`.
/// 4. `dump_in_progress.insert(id, false)`.
/// 5. `pty_dump_received.insert(id)`.
///
/// S-039 does NOT own styled-cell reconstruction or cursor restore (S-047 scope).
/// The `ScrollbackChunk` dispatch arm remains a no-op for S-039 (S-047 owns accumulation).
pub fn on_scrollback_dump_complete(
    app: &mut App,
    session_id: String,
    pty_rows: u16,
    pty_cols: u16,
) {
    // F-S039-P2-002 idempotency guard (BC-2.09.001 Inv-5):
    // Only process ScrollbackDumpComplete when a dump is actually in progress for this session.
    // Spurious, duplicate, cross-client, or post-detach completions MUST be dropped before any
    // parser reset/replay so a live populated parser is never destroyed by a stale message.
    if app.dump_in_progress.get(&session_id) != Some(&true) {
        tracing::trace!(
            session_id = %session_id,
            "ScrollbackDumpComplete outside dump window — no-op (F-S039-P2-002)"
        );
        return;
    }

    // F-PASS4-MED-001: abort the dump-window timeout task if one is in flight.
    // This prevents a spurious DumpWindowTimeout event from firing after the dump
    // completes normally. Must run BEFORE parser reset (after idempotency guard).
    if let Some(handle) = app.dump_timeout_handles.remove(&session_id) {
        handle.abort();
    }

    // Step 1: Reset parser with actual PTY dimensions from ScrollbackDumpComplete message.
    // Using pty_rows/pty_cols from the message ensures the parser matches the daemon's PTY.
    let scrollback_rows = app.scrollback_rows as usize;
    app.pty_parsers.insert(
        session_id.clone(),
        vt100::Parser::new(pty_rows, pty_cols, scrollback_rows),
    );

    // S-047: styled-cell reconstruction from ScrollbackChunk rows; cursor restore from
    // cursor_row/cursor_col. (Not implemented in S-039 — S-047 owns this extension point.)

    // Step 2: Replay pending_pty_bytes in receipt order through the reset parser.
    if let Some(buffered) = app.pending_pty_bytes.remove(&session_id) {
        if let Some(parser) = app.pty_parsers.get_mut(&session_id) {
            for chunk in buffered {
                parser.process(&chunk);
            }
        }
        // Step 3: pending_pty_bytes[id] cleared by remove() above.
    }

    // Step 4: remove session_id from dump_in_progress (dump window closed).
    // F-S039-REV-003: use remove() not insert(false). The idempotency guard at the top
    // of this function checks `dump_in_progress.get(&session_id) == Some(&true)`, so
    // absent (None) and present-false are treated identically — both are no-ops.
    // Removing the entry eliminates stale Some(false) entries that would otherwise
    // accumulate indefinitely, keeping the map compact (BC-2.09.001 Inv-5 step d).
    app.dump_in_progress.remove(&session_id);

    // Step 5: Insert into pty_dump_received (session has a complete dump for this attach).
    app.pty_dump_received.insert(session_id);
}

/// Handle `AppEvent::DumpWindowTimeout` — the dump-window timeout elapsed without
/// receiving `ScrollbackDumpComplete`.
///
/// # Idempotency guard
///
/// Only acts if `dump_in_progress.get(&session_id) == Some(&true)`.  A spurious or
/// late timeout event (e.g., arriving after a normal `ScrollbackDumpComplete` already
/// cleared the flag) is silently discarded.
///
/// # On timeout
///
/// 1. Remove `dump_in_progress[session_id]` (dump window closed — failure path).
/// 2. Clear `pending_pty_bytes[session_id]` (buffered bytes are stale; dump never completed).
/// 3. Clear `pending_pty_drop_count[session_id]` (drop counter for this session reset).
/// 4. Reset `pty_parsers[session_id]` to default PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS
///    (with `app.scrollback_rows`) — stale screen better than a corrupt one.
/// 5. Do NOT insert into `pty_dump_received` — next `enter_embedded_terminal` call
///    for the same session will re-run the full attach + dump protocol.
/// 6. Remove the (already-elapsed) timeout handle from `dump_timeout_handles`.
/// 7. Emit `tracing::warn!` and set `app.status_message` to a timeout notification.
///
/// F-PASS4-MED-001 / BC-2.09.001 Invariant 5 timeout path.
pub fn on_dump_window_timeout(app: &mut App, session_id: String) {
    // Idempotency guard: only act when a dump is actually in progress.
    if app.dump_in_progress.get(&session_id) != Some(&true) {
        tracing::trace!(
            session_id = %session_id,
            "on_dump_window_timeout: no active dump for session — discarding (idempotency guard)"
        );
        return;
    }

    tracing::warn!(
        session_id = %session_id,
        "scrollback dump timed out after {}s — clearing in-flight dump state (F-PASS4-MED-001)",
        DUMP_WINDOW_TIMEOUT.as_secs()
    );

    // Step 1: Remove dump_in_progress (failure path — dump never completed).
    app.dump_in_progress.remove(&session_id);

    // Step 2: Clear buffered bytes (stale; dump never completed).
    app.pending_pty_bytes.remove(&session_id);

    // Step 3: Clear drop counter for this session.
    app.pending_pty_drop_count.remove(&session_id);

    // Step 4: Reset the parser to defaults (stale screen better than blank/corrupt).
    // PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS — dims will be refreshed on next attach.
    let scrollback_rows = app.scrollback_rows as usize;
    app.pty_parsers.insert(
        session_id.clone(),
        vt100::Parser::new(PTY_DEFAULT_ROWS, PTY_DEFAULT_COLS, scrollback_rows),
    );

    // Step 5: Do NOT insert into pty_dump_received — next enter_embedded_terminal
    // must re-run the full attach + dump protocol.

    // Step 6: Remove the (already-elapsed) timeout handle (no-op abort on elapsed task).
    app.dump_timeout_handles.remove(&session_id);

    // Step 7: Set status bar warning for the focused user.
    let msg = format!("[warn] scrollback dump timed out for {session_id}");
    app.status_message = Some(msg);
}

/// Remove all PTY pipeline state for a session on GC (Terminated + list removal).
///
/// Called when a session transitions to `SessionState::Terminated` and is removed from
/// the session list. Removes all per-session state from:
/// - `app.pty_parsers`
/// - `app.pty_scroll_offsets`
/// - `app.pty_dump_received`
/// - `app.dump_in_progress`
/// - `app.pending_pty_bytes`
///
/// This is the canonical GC cleanup path for S-039 (BC-2.09.001 AC-008 / Invariant 4).
/// S-042 owns the `pty_scroll_offsets[session_id] = 0` reset on resize (ResizePane handler);
/// S-039 owns this full-removal on termination.
///
/// # Callers
///
/// Do NOT call this directly for session removal events. Use
/// [`gc_session_with_mode_exit`] instead — it checks whether the TUI is in
/// `EmbeddedTerminal` for the session being GC'd and exits that mode first.
/// Calling `gc_pty_session` directly while in `EmbeddedTerminal` for the target
/// session will leave the TUI stuck in `EmbeddedTerminal` with a destroyed parser
/// (permanent "Connecting to PTY..." render-trap).
pub fn gc_pty_session(app: &mut App, session_id: &str) {
    // BC-2.09.001 AC-008 / Invariant 4: remove all per-session PTY state on GC.
    app.pty_parsers.remove(session_id);
    app.pty_scroll_offsets.remove(session_id);
    app.pty_dump_received.remove(session_id);
    app.dump_in_progress.remove(session_id);
    app.pending_pty_bytes.remove(session_id);
    // F-PASS4-MED-001: also clean up Pass-4 fields on GC.
    // Abort any in-flight dump timeout task so it doesn't fire after the session is gone.
    if let Some(handle) = app.dump_timeout_handles.remove(session_id) {
        handle.abort();
    }
    // Remove drop counter (stale after session GC).
    app.pending_pty_drop_count.remove(session_id);
}

/// Exit `EmbeddedTerminal` mode (if currently focused on `session_id`) then GC.
///
/// This is the canonical removal path for ALL session-disappears events — both
/// `SessionStateChanged::Terminated` and `SessionListUpdate` roster-diff removal.
/// It must be called instead of [`gc_pty_session`] directly whenever a session
/// may be GC'd while the TUI is viewing it in `EmbeddedTerminal` mode.
///
/// # Behaviour
///
/// 1. Checks `app.mode` via a `matches!` guard. If the current mode is
///    `AppMode::EmbeddedTerminal { session_id: sid, .. }` AND `sid == session_id`,
///    calls [`exit_embedded_terminal`] to restore Dashboard mode and clear
///    `pty_dump_received` for the session.
/// 2. Calls [`gc_pty_session`] unconditionally to remove all 5 per-session maps.
///
/// # Safety contract (F-S039-REV-001)
///
/// - The `matches!` guard ensures that GC of a DIFFERENT session while the TUI is
///   viewing ANOTHER session does NOT exit `EmbeddedTerminal` mode for the
///   still-live session.
/// - `exit_embedded_terminal` MUST NOT send `ClientToServer::DetachSession` to a
///   dead/removed session. It does not — `exit_embedded_terminal` is a pure mode
///   transition that only mutates `app.mode` and `app.pty_dump_received`.
/// - This function NEVER panics.
pub fn gc_session_with_mode_exit(app: &mut App, session_id: &str) {
    // F-S039-REV-001: check whether we are currently in EmbeddedTerminal for THIS session.
    // If so, exit that mode first so Dashboard mode is restored before the parser is destroyed.
    // The matches! guard also ensures we do NOT exit mode for a different session's terminal.
    let is_focused_on_this_session = matches!(
        &app.mode,
        AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == session_id
    );
    if is_focused_on_this_session {
        tracing::debug!(
            session_id = %session_id,
            "gc_session_with_mode_exit: exiting EmbeddedTerminal before GC (F-S039-REV-001)"
        );
        exit_embedded_terminal(app, session_id);
    }
    gc_pty_session(app, session_id);
}

/// Type alias for the inbound IPC message channel used by the PTY output pipeline.
///
/// Aliases the `(Sender, Receiver)` pair type for `Result<ServerToClient, IpcError>` to
/// avoid the `type_complexity` clippy lint on `pty_output_channel()` return type.
type PtyOutputChannelPair = (
    tokio::sync::mpsc::Sender<Result<ServerToClient, IpcError>>,
    tokio::sync::mpsc::Receiver<Result<ServerToClient, IpcError>>,
);

/// Create the bounded inbound channel for PTY output pipeline messages (S-039 / AC-007).
///
/// Returns a `(Sender, Receiver)` pair bounded at `IPC_READER_CHANNEL_CAPACITY` (64).
/// The sender end is forwarded from `spawn_ipc_reader` into the event loop;
/// the receiver end is retained by the event loop and drained each tick via `try_recv`.
///
/// The reader task MUST use `tx.send(msg).await` (blocking backpressure), NOT
/// `tx.try_send(msg)` — silent message drops violate at-least-once delivery for
/// `PtyOutput` frames (BC-2.09.001 Invariant 3 / AC-007).
///
pub fn pty_output_channel() -> PtyOutputChannelPair {
    // BC-2.09.001 Invariant 3: bounded channel with backpressure (.send().await, not try_send).
    tokio::sync::mpsc::channel::<Result<ServerToClient, IpcError>>(IPC_READER_CHANNEL_CAPACITY)
}

// ---------------------------------------------------------------------------
// PermissionPromptPayload → PromptModal conversion
// ---------------------------------------------------------------------------

/// Convert a [`PermissionPromptPayload`] from the IPC layer into a [`PromptModal`]
/// for the TUI overlay stack.
///
/// The conversion lives in `monocle-tui` (the effectful boundary) rather than
/// `monocle-core` (the pure layer) to avoid adding a monocle-ipc dependency to
/// monocle-core. This is the `payload_to_modal()` function referenced in
/// BC-2.06.004.
///
/// # Mapping rules (BC-2.06.024 / AC-016)
///
/// - `tool_name == "Bash"` AND `tool_input["command"]` present → `ToolPayload::Bash { command }`
/// - `tool_name == "Bash"` AND `tool_input["command"]` absent → `ToolPayload::Generic`
/// - `tool_name == "Edit" | "Write"` AND (`old_content.is_some() || new_content.is_some()`)
///   AND `tool_input["path"]` present → `ToolPayload::Edit { old_content, new_content, path }`
/// - `tool_name == "Edit" | "Write"` AND BOTH content fields `None`
///   OR `tool_input["path"]` absent → `ToolPayload::Generic`
///   (In Phase 1 the daemon always sends `old_content: None, new_content: None` for all
///   deferred prompts, so ALL Phase-1 Edit/Write prompts produce `ToolPayload::Generic`.)
/// - `tool_name == "Read"` AND `tool_input["path"]` present → `ToolPayload::Read { path }`
/// - `tool_name == "Read"` AND `tool_input["path"]` absent → `ToolPayload::Generic`
/// - Anything else → `ToolPayload::Generic { tool_name, tool_input }`
pub fn payload_to_modal(payload: PermissionPromptPayload) -> PromptModal {
    let tool_payload = match payload.tool_name.as_str() {
        "Bash" => {
            // AC-016 (BC-2.06.024): fall back to Generic when "command" key is absent.
            match payload.tool_input.get("command").and_then(|v| v.as_str()) {
                Some(cmd) => ToolPayload::Bash {
                    command: cmd.to_string(),
                },
                None => ToolPayload::Generic {
                    tool_name: payload.tool_name.clone(),
                    tool_input: payload.tool_input.clone(),
                },
            }
        }
        "Edit" | "Write" => {
            // AC-016 v1.10 (BC-2.06.024): Edit/Write → ToolPayload::Edit ONLY when
            // at least one content field is Some AND path is present.
            //
            // Both-None → Generic (the Phase-1 normal path: daemon sends None/None for
            // all deferred prompts).  An absent path → Generic (no meaningful diff to
            // show regardless of content).  This avoids rendering a blank diff pane
            // (BC-2.06.010) when no content is available, surfacing the raw tool_input
            // JSON instead which at minimum contains the path field.
            let has_content = payload.old_content.is_some() || payload.new_content.is_some();
            let path_opt = payload
                .tool_input
                .get("path")
                .and_then(|v| v.as_str())
                .map(std::path::PathBuf::from);

            if has_content {
                if let Some(path) = path_opt {
                    ToolPayload::Edit {
                        old_content: payload.old_content.unwrap_or_default(),
                        new_content: payload.new_content.unwrap_or_default(),
                        path,
                    }
                } else {
                    // Content present but no path — fall back to Generic.
                    ToolPayload::Generic {
                        tool_name: payload.tool_name.clone(),
                        tool_input: payload.tool_input.clone(),
                    }
                }
            } else {
                // Both-None (Phase-1 normal path) → Generic.
                ToolPayload::Generic {
                    tool_name: payload.tool_name.clone(),
                    tool_input: payload.tool_input.clone(),
                }
            }
        }
        "Read" => {
            // AC-016 (BC-2.06.024): fall back to Generic when "path" key is absent.
            match payload.tool_input.get("path").and_then(|v| v.as_str()) {
                Some(p) => ToolPayload::Read {
                    path: std::path::PathBuf::from(p),
                },
                None => ToolPayload::Generic {
                    tool_name: payload.tool_name.clone(),
                    tool_input: payload.tool_input.clone(),
                },
            }
        }
        _ => ToolPayload::Generic {
            tool_name: payload.tool_name.clone(),
            tool_input: payload.tool_input.clone(),
        },
    };

    PromptModal {
        prompt_id: payload.prompt_id,
        session_id: payload.session_id,
        tool_name: payload.tool_name,
        tool_payload,
        received_at: Instant::now(),
    }
}

// ---------------------------------------------------------------------------
// Idempotent overlay insert (BC-2.05.002 Invariant 4)
// ---------------------------------------------------------------------------

/// Insert a permission prompt into the overlay stack, enforcing prompt_id
/// idempotency (BC-2.05.002 Invariant 4).
///
/// If `payload.prompt_id` is already present in `overlay`, the duplicate is
/// silently discarded and the function returns immediately. This covers the
/// at-least-once delivery race where a `PermissionPromptQueued` streaming
/// message arrives after (or before) the same prompt appears in
/// `InitialState.overlay_stack`.
///
/// # Usage
///
/// This is the ONLY function that should push to `App::overlay_stack`.
/// Direct `push_back` on the VecDeque bypasses the idempotency guard and
/// violates BC-2.05.002 Invariant 4.
///
/// # Arguments
///
/// * `overlay` — mutable reference to the overlay stack.
/// * `payload` — the permission prompt payload from the IPC message.
pub fn apply_permission_prompt_queued(
    overlay: &mut VecDeque<PromptModal>,
    payload: PermissionPromptPayload,
) {
    if overlay.iter().any(|m| m.prompt_id == payload.prompt_id) {
        tracing::trace!(
            prompt_id = %payload.prompt_id,
            "duplicate prompt_id, silently discarding"
        );
        return;
    }
    overlay.push_back(payload_to_modal(payload));
}

// ---------------------------------------------------------------------------
// IPC message handlers
// ---------------------------------------------------------------------------

/// Handle `ServerToClient::InitialState` on first connection (AC-008, BC-2.06.004 PC-2).
///
/// Populates `app.sessions`, `app.drop_counter`, `app.overlay_stack`, and
/// `app.event_ring` from the daemon's initial state push.
///
/// `ring_tail` is drained into `app.event_ring` (bounded to `EVENT_RING_CAPACITY`;
/// oldest entries are evicted FIFO on overflow — eviction does NOT increment
/// `app.drop_counter` per architect decision F-S025-ADV1-HIGH-002).
///
/// Each entry in `overlay_stack` is inserted via `apply_permission_prompt_queued`
/// to enforce prompt_id idempotency. If the resulting overlay_stack is non-empty,
/// transitions to `AppMode::Overlay`.
pub fn on_initial_state(
    app: &mut App,
    sessions: Vec<EnrichedSession>,
    ring_tail: Vec<HookEventRecord>,
    overlay_stack: Vec<PermissionPromptPayload>,
    drop_counter: u64,
) {
    app.drop_counter = drop_counter;

    // F-S039-REV-002: GC stale sessions BEFORE assigning the new roster.
    // On reconnect, the incoming InitialState roster may not contain sessions that
    // existed in the old app.sessions (e.g., sessions that terminated while the TUI
    // was disconnected). Without this GC step, parsers/maps for those stale sessions
    // accumulate across reconnects — unbounded growth.
    //
    // Algorithm: compute sessions present in OLD app.sessions but absent from the
    // incoming roster, then call gc_session_with_mode_exit for each.
    // gc_session_with_mode_exit handles the EmbeddedTerminal exit-before-GC contract
    // (F-S039-REV-001) so this path is correct even if the TUI was viewing a stale session.
    {
        let new_ids: HashSet<&str> = sessions.iter().map(|s| s.session_id.as_str()).collect();
        let stale: Vec<String> = app
            .sessions
            .iter()
            .filter(|s| !new_ids.contains(s.session_id.as_str()))
            .map(|s| s.session_id.clone())
            .collect();
        for id in stale {
            gc_session_with_mode_exit(app, &id);
        }
    }

    app.sessions = sessions;

    // F-S039-001 (BC-2.09.001 Inv-5): create parsers for all sessions in the initial roster.
    // Per-session PTY dims are not available at InitialState time; use canonical defaults
    // (PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS — monocle-core pure-core; SS-embedded-pty §Parser
    // ownership; F-S039-P2-004). The parser will be RESET with actual pty_rows/pty_cols when
    // ScrollbackDumpComplete arrives (on_scrollback_dump_complete).
    // Only create a parser for sessions that don't already have one (no-clobber on reconnect).
    for session in &app.sessions {
        let id = &session.session_id;
        if !app.pty_parsers.contains_key(id) {
            app.pty_parsers.insert(
                id.clone(),
                vt100::Parser::new(
                    PTY_DEFAULT_ROWS,
                    PTY_DEFAULT_COLS,
                    app.scrollback_rows as usize,
                ),
            );
            app.pty_scroll_offsets.insert(id.clone(), 0);
        }
    }

    // Seed the event ring from the daemon's ring snapshot.
    // Bounded to EVENT_RING_CAPACITY; ring_tail from daemon is already bounded
    // to RAM_RING_CAPACITY (4096) so overflow is not expected, but enforced defensively.
    app.event_ring.clear();
    // S-028 (BC-2.05.002 PC-2): also pre-populate event_ribbon_events from ring_tail.
    // Clear first to avoid duplicate entries on reconnect.
    app.event_ribbon_events.clear();
    for record in ring_tail {
        if app.event_ring.len() == EVENT_RING_CAPACITY {
            app.event_ring.pop_front(); // FIFO eviction; does NOT increment drop_counter
        }
        // Build a HookEventRow from the record and prepend to the ribbon rolling window.
        let row = crate::ui::event_ribbon::hook_event_row_from_record(&record);
        crate::ui::event_ribbon::push_event_row(
            &mut app.event_ribbon_events,
            row,
            EVENT_RING_CAPACITY,
        );
        app.event_ring.push_back(record);
    }

    for payload in overlay_stack {
        apply_permission_prompt_queued(&mut app.overlay_stack, payload);
    }

    if !app.overlay_stack.is_empty() {
        // F-S025-ADV2-HIGH-003: AppMode::Overlay no longer stores the stack.
        // App::overlay_stack IS the stack. Mode variant signals "in overlay mode".
        app.mode = AppMode::Overlay {
            prior: FocusSnapshot::Sessions,
        };
    }
}

/// Handle `ServerToClient::DropCounterUpdate { drop_counter }` (AC-007, BC-2.06.005 PC-3).
///
/// Updates `app.drop_counter`. The render loop reads this field to show
/// `"[dropped: N]"` in yellow in the Sessions panel status bar.
pub fn on_drop_counter_update(app: &mut App, drop_counter: u64) {
    app.drop_counter = drop_counter;
}

/// Handle `ServerToClient::PermissionPromptQueued` on the streaming IPC path
/// (BC-2.06.008 / BC-2.05.002 Invariant 4).
///
/// Performs the idempotent insert via [`apply_permission_prompt_queued`] and
/// then transitions `app.mode` to `AppMode::Overlay` if the stack is now
/// non-empty and the app is not already in Overlay mode.  Prior-focus capture
/// preserves the focused panel from Dashboard, Filtering, and Fullscreen modes
/// so that the overlay can restore it on dismiss.
///
/// Extracted as a `pub` free function so that tests drive the production code
/// path directly (F-S026-ADV2-HIGH-001 streaming-IPC handler testability).
/// `handle_server_message` delegates to this function.
pub fn on_permission_prompt_queued(app: &mut App, payload: PermissionPromptPayload) {
    // BC-2.06.018 PC-4: set pending=true on the most recent PreToolUse ribbon row
    // matching the prompt's session_id BEFORE inserting into overlay_stack.
    // This links the permission prompt to its corresponding ribbon event display.
    let session_id = &payload.session_id;
    for row in app.event_ribbon_events.iter_mut() {
        if row.session_id == *session_id
            && matches!(row.hook_type, monocle_ipc::types::HookType::PreToolUse)
            && !row.pending
        {
            row.pending = true;
            break; // Set only the most recent matching row (front = newest).
        }
    }

    apply_permission_prompt_queued(&mut app.overlay_stack, payload);
    // F-S025-ADV2-HIGH-003: mode update is App-level; transition() does not
    // mutate overlay_stack. Enter Overlay mode if not already in it.
    if !app.overlay_stack.is_empty() && !matches!(app.mode, AppMode::Overlay { .. }) {
        let prior = match &app.mode {
            AppMode::Dashboard { focused } => focused.clone(),
            AppMode::Filtering { prior, .. } => prior.clone(),
            AppMode::Fullscreen { prior, .. } => prior.clone(),
            AppMode::Overlay { .. } => FocusSnapshot::Sessions, // unreachable
            // S-039: permission prompts arriving while in EmbeddedTerminal or
            // SessionCreation save Sessions as the restore target.
            AppMode::EmbeddedTerminal { .. } | AppMode::SessionCreation { .. } => {
                FocusSnapshot::Sessions
            }
        };
        app.mode = AppMode::Overlay { prior };
    }
}

/// Handle `ServerToClient::PermissionPromptResolved` on the streaming IPC path
/// (BC-2.06.023 / BC-2.05.002 Invariant 4).
///
/// Removes every entry whose `prompt_id` matches `prompt_id` from the overlay
/// stack (retain-all semantics).  If the stack is now empty and the app is in
/// `AppMode::Overlay`, collapses back to `AppMode::Dashboard { focused: prior }`.
///
/// If `prompt_id` is not present in the stack the call is a silent no-op
/// (BC-2.06.023 PC-3: no WARN/ERROR on unknown prompt; TRACE at most).
///
/// Extracted as a `pub` free function so that tests drive the production code
/// path directly (F-S026-ADV2-HIGH-001 streaming-IPC handler testability).
/// `handle_server_message` delegates to this function.
pub fn on_permission_prompt_resolved(app: &mut App, prompt_id: Uuid) {
    let before_len = app.overlay_stack.len();
    app.overlay_stack.retain(|m| m.prompt_id != prompt_id);
    if app.overlay_stack.len() == before_len {
        // BC-2.06.023 PC-3: unknown prompt_id — silent discard, TRACE only.
        tracing::trace!(
            %prompt_id,
            "PermissionPromptResolved for unknown prompt_id; silently discarding"
        );
    }
    // F-S025-ADV2-HIGH-003: if stack is now empty, collapse to Dashboard.
    if app.overlay_stack.is_empty() {
        if let AppMode::Overlay { prior } = app.mode.clone() {
            app.mode = AppMode::Dashboard { focused: prior };
        }
    }
}

/// Handle a `TransportEvent` on the IPC channel (AC-003, BC-2.06.004 PC-2).
///
/// On `TransportEvent::Disconnected`: clears the overlay stack, transitions
/// `app.mode` to `Dashboard { focused: Sessions }`, and sets a status bar
/// notification `"[disconnected] reconnecting..."`.
pub fn on_transport_event(app: &mut App, event: TransportEvent) {
    match event {
        TransportEvent::Disconnected => {
            app.overlay_stack.clear();

            // F-PASS4-MED-002: clear ALL in-flight dump state on disconnect.
            // Dump state from the previous connection is invalid for the new connection —
            // ScrollbackDumpComplete will never arrive for in-flight dumps from a dead session.
            // Steps 1-6 per architect ruling (F-PASS4-MED-002):

            // 1. Clear dump_in_progress.
            app.dump_in_progress.clear();
            // 2. Clear pending_pty_bytes (buffered bytes from dead connection are stale).
            app.pending_pty_bytes.clear();
            // 3. Clear pending_pty_drop_count (stale drop counts from prior attach windows).
            app.pending_pty_drop_count.clear();
            // 4. Clear pty_dump_received (reconnect invalidates all prior dump receipts;
            //    next enter_embedded_terminal must re-run the full attach + dump protocol).
            app.pty_dump_received.clear();
            // 5. Abort ALL dump timeout handles and drain the map.
            for (_, handle) in app.dump_timeout_handles.drain() {
                handle.abort();
            }
            // 6. If currently in EmbeddedTerminal, exit via exit_embedded_terminal (restores
            //    prior mode + removes session from pty_dump_received — AC-005 / F-PASS4-MED-002
            //    step 6). For all other modes, reset to Dashboard directly (existing behaviour).
            if let AppMode::EmbeddedTerminal { session_id, .. } = &app.mode.clone() {
                let sid = session_id.clone();
                exit_embedded_terminal(app, &sid);
                // exit_embedded_terminal already sets mode to Dashboard { focused: prior }.
            } else {
                app.mode = AppMode::Dashboard {
                    focused: FocusSnapshot::Sessions,
                };
            }
            // pty_parsers is NOT cleared — stale screen is better than blank (no-clobber).

            // 7. Set reconnecting status bar message (overrides the exit_embedded_terminal
            //    status which may have been set to None by re-attach cleanup).
            app.status_message = Some(DAEMON_DISCONNECT_STATUS.to_string());
            tracing::warn!("IPC transport disconnected; entering reconnect state (in-flight dump state cleared — F-PASS4-MED-002)");
        }
        // TransportEvent is #[non_exhaustive] (monocle-ipc::events) — future variants
        // (e.g., Reconnecting, Reconnected) will be added as S-025+ extends the protocol.
        // The default arm ensures forward compatibility without requiring a BC revision.
        _ => {
            tracing::debug!(event = ?event, "on_transport_event: unhandled variant (future extension)");
        }
    }
}

// ---------------------------------------------------------------------------
// S-028: Event ribbon IPC handlers (BC-2.05.002 + BC-2.05.004)
// ---------------------------------------------------------------------------

/// Handle `ServerToClient::HookEventReceived` — append new event to the ribbon log.
///
/// Called from `handle_server_message` for every `HookEventReceived` IPC message
/// (BC-2.05.004). Converts the message fields into a `HookEventRow` and appends it
/// to `app.event_ribbon_events` (the rolling log for all sessions).
///
/// # Rolling window (BC-2.06.018 PC-3)
///
/// Events are prepended to the front of `app.event_ribbon_events` (newest at front,
/// oldest at back — BC-2.06.018 PC-2 newest-first ordering). When the `VecDeque` is
/// at capacity (`panel_height` bound, determined at render time), the oldest event
/// (back) is popped before prepending. The `panel_height` cap is enforced here using
/// the compile-time fallback `EVENT_RING_CAPACITY` until the render-time cap is
/// applied at `push_event_row` call sites.
///
/// # Client-side filtering (BC-2.05.004 INV-3)
///
/// ALL events (from all sessions) are appended unconditionally. Session filtering
/// for display is deferred to the Event Ribbon render path (EventRibbon::render
/// filters by `selected_session_id`). No IPC-layer filtering is performed.
///
/// Extracted as a `pub` free function for test-driven dispatch (same testability
/// pattern as `on_permission_prompt_queued` — F-S026-ADV2-HIGH-001).
pub fn on_hook_event_received(
    app: &mut App,
    hook_type: HookType,
    session_id: String,
    _payload_excerpt: String,
    latency_ms: u64,
    timestamp_micros: i64,
) {
    // BC-2.05.004: append new event to the ribbon log for all sessions (no IPC filtering).
    // BC-2.06.018 PC-2: newest at front (prepend).
    // BC-2.06.018 PC-3: use event_ribbon_panel_height as dynamic cap (updated by render_frame
    // each cycle; initialises to EVENT_RING_CAPACITY so the first push before any render is safe).
    // BC-2.05.004 PC-2 / SS-ipc: use the daemon's timestamp_micros (not TUI receive time).
    //
    // Capture session_id as a String before it is moved into hook_event_row_from_received.
    // This copy is used by the AC-008 auto-scroll gate below to compare against the
    // currently selected session. One heap copy per received event is acceptable at the
    // expected event rate (bound by IPC throughput, not TUI frame rate).
    let session_id_owned = session_id.clone();
    let row = crate::ui::event_ribbon::hook_event_row_from_received(
        hook_type,
        session_id,
        latency_ms,
        timestamp_micros,
    );
    let cap = app.event_ribbon_panel_height;
    crate::ui::event_ribbon::push_event_row(&mut app.event_ribbon_events, row, cap);

    // BC-2.06.018 PC-8 / AC-008: auto-scroll to row 0 (newest) ONLY when:
    //   1. !pinned_top (user has not pinned the scroll position), AND
    //   2. The incoming event's session_id matches the currently selected session.
    //
    // An event for a non-selected session must NOT disturb the selected session's
    // scroll position (BC-2.06.018 AC-008).
    //
    // Effective selected session resolution:
    //   - Use app.selected_session_id if explicitly set (by dispatch_key_event or render_frame).
    //   - Fall back to app.sessions.first() when selected_session_id is None (startup default:
    //     the Sessions panel highlights the first row before any cursor movement).
    // Effective selected session resolution:
    //   - Use app.selected_session_id if explicitly set (by dispatch_key_event / render_frame).
    //   - Fall back to app.sessions.first() when selected_session_id is None (startup default:
    //     Sessions panel highlights the first row before any cursor movement).
    //   - When sessions is empty and selected_session_id is None, effective_selected is None;
    //     in that case auto-scroll fires unconditionally (no session to discriminate against).
    let effective_selected: Option<&str> = app
        .selected_session_id
        .as_deref()
        .or_else(|| app.sessions.first().map(|s| s.session_id.as_str()));

    // Gate: scroll only when !pinned AND (no discriminating session OR incoming matches selected).
    let session_matches = effective_selected
        .map(|sel| sel == session_id_owned.as_str())
        .unwrap_or(true); // no sessions → no discrimination → any event may scroll

    if !app.event_ribbon_state.pinned_top && session_matches {
        app.event_ribbon_state.list_state.select(Some(0));
    }
}

// ---------------------------------------------------------------------------
// Runtime dir resolution
// ---------------------------------------------------------------------------

/// Resolve the monocle daemon runtime directory.
///
/// Resolution order (mirrors `monocle-runtime::lifecycle::resolve_runtime_dir`):
/// 1. `MONOCLE_RUNTIME_DIR` environment variable (if non-empty).
/// 2. Platform XDG data dir via `directories::ProjectDirs`.
///
/// Returns `Err` if both sources are unavailable (no HOME env, no XDG dir).
pub fn resolve_runtime_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("MONOCLE_RUNTIME_DIR") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    let proj = ProjectDirs::from("dev", "monocle", "monocle")
        .context("could not resolve runtime dir: no valid home directory found")?;
    Ok(proj.data_local_dir().to_path_buf())
}

// ---------------------------------------------------------------------------
// Dedicated IPC reader task (Option B — F-S025-ADV2-BLOCKER-001)
// ---------------------------------------------------------------------------

/// Spawn a dedicated reader task that calls `read_framed` in a loop and forwards
/// `Result<ServerToClient, IpcError>` into a bounded `mpsc::channel(64)`.
///
/// # Cancellation safety
///
/// `read_framed` is NOT cancellation-safe: the two sequential `read_exact` calls
/// inside it will silently corrupt the byte stream if the future is dropped between
/// the first and second call (e.g., inside a `tokio::time::timeout` wrapper).
/// This dedicated task holds `read_framed` to completion on every call — the event
/// loop never cancels it. The event loop uses `ipc_rx.try_recv()` (non-blocking,
/// infallible) to drain available messages each tick instead.
///
/// # Channel semantics (BC-2.05.002 Invariant 4 — at-least-once delivery)
///
/// The sender uses `tx.send(msg).await` (blocking backpressure), NOT `try_send`.
/// Dropping messages silently when the channel is full would violate the at-least-once
/// delivery guarantee for `PermissionPromptQueued`. Backpressure is the correct policy:
/// if the event loop is consistently slower than the daemon, that is a render
/// performance problem to diagnose, not a message-loss policy to encode.
///
/// # Lifecycle
///
/// The task exits when:
/// 1. `read_framed` returns any `IpcError` (disconnect forwarded to channel, then break).
/// 2. The channel receiver is dropped (TUI exiting — task exits cleanly without error).
///
/// The caller retains the `JoinHandle` to call `.abort()` on clean exit or reconnect.
///
/// # Reconnect
///
/// On reconnect, the caller calls `reader_handle.abort()` to ensure the old task is
/// cleaned up, then re-creates the channel with a fresh `(ipc_tx2, ipc_rx2)` pair
/// (F-S025-ADV3-MED-003). Because `ipc_tx` is passed by MOVE (not clone), the channel
/// closes naturally when the reader exits — `ipc_rx.try_recv()` returns
/// `TryRecvError::Disconnected` instead of `TryRecvError::Empty` forever.
/// Channel re-creation on reconnect has negligible allocation cost.
pub fn spawn_ipc_reader<R>(
    mut reader: R,
    tx: tokio::sync::mpsc::Sender<Result<ServerToClient, IpcError>>,
) -> tokio::task::JoinHandle<()>
where
    R: AsyncReadExt + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            match read_framed::<_, ServerToClient>(&mut reader).await {
                Ok(msg) => {
                    if tx.send(Ok(msg)).await.is_err() {
                        // Receiver dropped (TUI exiting): exit cleanly without error.
                        return;
                    }
                }
                Err(IpcError::Disconnected) => {
                    // Forwarding the disconnect signal lets the event loop fire
                    // on_transport_event(Disconnected) and enter the reconnect path.
                    let _ = tx.send(Err(IpcError::Disconnected)).await;
                    return;
                }
                Err(e) => {
                    // All other errors (MessageTooLarge, IoError, SerializeError):
                    // forward and exit. The event loop treats any Err as a disconnect.
                    let _ = tx.send(Err(e)).await;
                    return;
                }
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Outbound IPC writer task (F-S026-ADV5-CRIT-001 — BC-2.06.011/012/013)
// ---------------------------------------------------------------------------

/// Capacity of the inbound `ServerToClient` IPC reader channel.
///
/// The reader task uses `tx.send(msg).await` (blocking backpressure, NOT `try_send`)
/// so messages are never silently dropped when the event loop is slow.
/// N=64 provides burst absorption for high-frequency `PtyOutput` messages while
/// keeping bounded-channel semantics (SS-conventions-anti-patterns.md §forbidden-patterns:
/// no unbounded channels). Canonical value per BC-2.09.001 Invariant 3 / AC-007.
///
/// S-039 extracts this from the inline literal in `setup_ipc_streams_with_rx`
/// to make the capacity assertable by name in tests (rather than by magic number).
pub const IPC_READER_CHANNEL_CAPACITY: usize = 64;

/// Capacity of the outbound `ClientToServer` command channel.
///
/// Lower than the inbound reader channel (`IPC_READER_CHANNEL_CAPACITY`) because
/// `ClientToServer` messages are rare user-driven keypresses, not high-frequency
/// daemon events.  N=32 provides headroom for burst scenarios while keeping
/// bounded-channel semantics (SS-conventions-anti-patterns.md §forbidden-patterns:
/// no unbounded channels).
pub const IPC_CMD_CHANNEL_CAPACITY: usize = 32;

/// Spawn a dedicated outbound writer task that drains `cmd_rx` and writes each
/// `ClientToServer` message to `writer` using 4-byte LE length-prefix framing.
///
/// # Channel semantics (BC-2.04.011 — bounded, non-blocking send)
///
/// The caller uses `try_send` (bounded, non-blocking) to enqueue commands.
/// This task uses `recv().await` on the receiving end — blocking on the task,
/// not on the event loop.  `try_send` from the event loop returns
/// `TrySendError::Full` when the channel is at capacity; the caller logs WARN
/// (surfaced in the status bar via `App::drop_counter`) and discards the message.
///
/// # Lifecycle
///
/// The task exits when:
/// 1. `write_framed` returns any `IpcError` (daemon gone — logs WARN, task exits).
/// 2. The channel sender side is dropped (TUI exiting or reconnect — task exits cleanly).
///
/// The caller retains the `JoinHandle` to call `.abort()` on clean exit or reconnect.
///
/// # Reconnect
///
/// On reconnect, the caller aborts the old writer task and calls `setup_ipc_streams`
/// with the fresh stream, which creates a new cmd channel + writer task and assigns
/// `app.ipc_tx = Some(new_cmd_tx)`.
pub fn spawn_ipc_writer<W>(
    mut writer: W,
    mut cmd_rx: tokio::sync::mpsc::Receiver<ClientToServer>,
) -> tokio::task::JoinHandle<()>
where
    W: tokio::io::AsyncWriteExt + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        while let Some(msg) = cmd_rx.recv().await {
            if let Err(e) = monocle_ipc::framing::write_framed(&mut writer, &msg).await {
                tracing::warn!(
                    error = %e,
                    "spawn_ipc_writer: write_framed failed — daemon gone; writer task exiting \
                     (BC-2.06.011/012/013)"
                );
                return;
            }
        }
        // cmd_rx exhausted (sender dropped → TUI exiting or reconnect): exit cleanly.
        tracing::debug!("spawn_ipc_writer: cmd_rx closed — writer task exiting cleanly");
    })
}

// ---------------------------------------------------------------------------
// IPC stream setup helper (F-S026-ADV5-CRIT-001 — wires app.ipc_tx)
// ---------------------------------------------------------------------------

/// Wire the IPC connection for `app` using a fresh `UnixStream`.
///
/// Splits the stream into read and write halves, spawns the inbound reader task
/// (`spawn_ipc_reader`) and the outbound writer task (`spawn_ipc_writer`), creates
/// bounded channels for both directions, and assigns `app.ipc_tx = Some(cmd_tx)`.
///
/// # Why this function exists
///
/// Before this function, `run()` moved the whole `UnixStream` into `spawn_ipc_reader`
/// (read-only task) and never created an outbound writer task.  `App.ipc_tx` was
/// permanently `None`, causing every `send_permission_decision` call to WARN and
/// silently discard the message.  This function is the production fix for
/// F-S026-ADV5-CRIT-001.
///
/// # Returns
///
/// `(reader_handle, writer_handle)` — both `JoinHandle<()>`. The caller MUST:
/// 1. Retain both handles.
/// 2. Call `reader_handle.abort()` AND `writer_handle.abort()` on clean exit or
///    reconnect to prevent task leaks.
///
/// # Reconnect
///
/// On reconnect, call this function again with the fresh stream halves.  It will:
/// - Drop the old `cmd_tx` (the old writer task exits when its `cmd_rx` is closed).
/// - Create a new `cmd_tx` and assign it to `app.ipc_tx`.
///
/// The caller is responsible for aborting the old handles before or after calling
/// this function.  The old writer task will exit on its own when `app.ipc_tx` is
/// replaced (the old `cmd_tx` is dropped → channel closes → writer task exits), but
/// an explicit `.abort()` is cleaner and ensures immediate cleanup.
///
/// # Generics
///
/// `R` must implement `AsyncReadExt + Unpin + Send + 'static` (read half).
/// `W` must implement `tokio::io::AsyncWriteExt + Unpin + Send + 'static` (write half).
///
/// This generic form allows tests to pass `tokio::io::DuplexStream` halves
/// without requiring a real `UnixStream`.  Production code (in `run()`) passes
/// `OwnedReadHalf` / `OwnedWriteHalf` from `UnixStream::into_split()`.
pub fn setup_ipc_streams<R, W>(
    app: &mut App,
    read_half: R,
    write_half: W,
) -> (tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>)
where
    R: AsyncReadExt + Unpin + Send + 'static,
    W: tokio::io::AsyncWriteExt + Unpin + Send + 'static,
{
    let (rh, wh, _inbound_rx) = setup_ipc_streams_with_rx(app, read_half, write_half);
    (rh, wh)
}

/// Internal variant used by `run()`: returns the inbound receiver so the event loop
/// can drain it via `ipc_rx.try_recv()`.
///
/// This function performs the authoritative wiring — `setup_ipc_streams` is a thin
/// wrapper that discards `inbound_rx` for test convenience (when the test only needs
/// to verify that `app.ipc_tx` is wired and messages flow through the write half).
///
/// Returns `(reader_handle, writer_handle, inbound_rx)`.
///
/// # Integration test seam
///
/// Exposed as `pub` (with `#[doc(hidden)]`) so that external integration tests in
/// `crates/monocle-tui/tests/` can retain the real `inbound_rx` and drive the full
/// inbound dispatch path without duplicating channel setup. Not part of the stable
/// public API — callers should pin to the `monocle-tui` crate version.
#[doc(hidden)]
pub fn setup_ipc_streams_with_rx<R, W>(
    app: &mut App,
    read_half: R,
    write_half: W,
) -> (
    tokio::task::JoinHandle<()>,
    tokio::task::JoinHandle<()>,
    tokio::sync::mpsc::Receiver<Result<ServerToClient, IpcError>>,
)
where
    R: AsyncReadExt + Unpin + Send + 'static,
    W: tokio::io::AsyncWriteExt + Unpin + Send + 'static,
{
    // Inbound reader channel: ServerToClient messages from the daemon.
    // F-PASS4-LOW-001: route through pty_output_channel() so the capacity test
    // exercises the same constructor as the real event loop (not a parallel inline literal).
    // N=IPC_READER_CHANNEL_CAPACITY=64 — see spawn_ipc_reader doc comment for rationale.
    let (inbound_tx, inbound_rx) = pty_output_channel();
    let reader_handle = spawn_ipc_reader(read_half, inbound_tx);

    // Outbound command channel: ClientToServer messages to the daemon (BC-2.06.011/012/013).
    // N=IPC_CMD_CHANNEL_CAPACITY=32 — lower than inbound because commands are rare,
    // user-driven keypresses, not high-frequency daemon events.
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<ClientToServer>(IPC_CMD_CHANNEL_CAPACITY);
    let writer_handle = spawn_ipc_writer(write_half, cmd_rx);

    // Wire app.ipc_tx to the new cmd channel sender.
    // This is the production fix for F-S026-ADV5-CRIT-001: ipc_tx is now Some after wiring.
    app.ipc_tx = Some(cmd_tx);

    (reader_handle, writer_handle, inbound_rx)
}

// ---------------------------------------------------------------------------
// Reconnect helper (S-023 integration — BC-2.05.006)
// ---------------------------------------------------------------------------

/// Attempt to reconnect to the daemon UDS socket with exponential backoff.
///
/// Called by the event loop after `on_transport_event(Disconnected)` runs (SOQ-3 clear
/// completes before this function is invoked — BC-2.05.007 Invariant 1).
///
/// # Reconnect strategy (BC-2.05.006 PC-3, PC-4, PC-5)
///
/// Uses `monocle_ipc::reconnect::BackoffState` for the backoff schedule (250ms → 2000ms
/// cap). The raw `tokio::net::UnixStream::connect` is used directly so the caller's
/// `spawn_ipc_reader` (which reads `ServerToClient` frames) can own the socket. The
/// `monocle_ipc::reconnect::reconnect()` function is NOT used here because it returns
/// `UdsClientTransport`, which reads `ClientToServer` frames — the wrong direction for
/// the TUI's `spawn_ipc_reader` task.
///
/// # Returns
///
/// - `Ok(stream)` — a fresh `UnixStream` connected to the daemon. The caller MUST:
///   1. Call `read_framed::<_, ServerToClient>` to receive the `InitialState` push.
///   2. Spawn a new `spawn_ipc_reader` with a fresh channel.
/// - `Err(IpcError::ReconnectTimeout)` — the 5-second window was exhausted.
///   The caller MUST enter offline mode via `poll_for_new_daemon`.
async fn reconnect_to_daemon(
    sock_path: &std::path::Path,
    backoff: &mut BackoffState,
) -> Result<tokio::net::UnixStream, IpcError> {
    let window = std::time::Duration::from_secs(RECONNECT_WINDOW_SECS);
    let deadline = tokio::time::Instant::now() + window;
    let mut attempt = 0u32;

    loop {
        // Check window before sleeping.
        if tokio::time::Instant::now() >= deadline {
            tracing::warn!(
                sock_path = %sock_path.display(),
                "reconnect_to_daemon: 5-second window exhausted — entering offline mode \
                 (BC-2.05.006 PC-5)"
            );
            return Err(IpcError::ReconnectTimeout);
        }

        attempt = attempt.saturating_add(1);
        let delay = backoff.next_delay();
        let delay_ms = delay.as_millis();

        tracing::debug!(
            attempt,
            sock_path = %sock_path.display(),
            delay_ms,
            "reconnect_to_daemon: attempt {attempt} — waiting {delay_ms}ms before connecting"
        );

        tokio::time::sleep(delay).await;

        // Re-check after sleep.
        if tokio::time::Instant::now() >= deadline {
            tracing::warn!(
                sock_path = %sock_path.display(),
                "reconnect_to_daemon: window exhausted after backoff sleep — offline mode"
            );
            return Err(IpcError::ReconnectTimeout);
        }

        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let connect_result =
            tokio::time::timeout(remaining, tokio::net::UnixStream::connect(sock_path)).await;

        match connect_result {
            Ok(Ok(stream)) => {
                tracing::info!(
                    attempt,
                    sock_path = %sock_path.display(),
                    "reconnect_to_daemon: connected on attempt {attempt}"
                );
                return Ok(stream);
            }
            Ok(Err(e)) => {
                tracing::debug!(
                    attempt,
                    sock_path = %sock_path.display(),
                    error = %e,
                    "reconnect_to_daemon: attempt {attempt} failed — will retry"
                );
            }
            Err(_elapsed) => {
                tracing::warn!(
                    attempt,
                    sock_path = %sock_path.display(),
                    "reconnect_to_daemon: connect() timed out within remaining window — offline mode"
                );
                return Err(IpcError::ReconnectTimeout);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Offline reconnect orchestration (BC-2.05.006 PC-5 step 5 — F-WAVE6-GATE-CRIT-001)
// ---------------------------------------------------------------------------

/// Orchestrate the offline→poll→reconnect cycle after the 5-second window exhausted.
///
/// Called when [`reconnect_to_daemon`] returns [`IpcError::ReconnectTimeout`] in either
/// the fatal-protocol arm or the reader-disconnect arm of the event loop.  It implements
/// BC-2.05.006 PC-5 step 5:
///
/// > "TUI polls the lock file every 5 seconds. When a new lock file is detected
/// > (new daemon started), the TUI re-enters the reconnect loop from step 1."
///
/// # What the broken code did
///
/// Before this fix, both `ReconnectTimeout` arms set `app.status_message = offline`
/// and `break`'d out of the IPC drain loop without re-entering the reconnect loop.
/// The outer event loop continued, but `ipc_rx.try_recv()` returned `Disconnected`
/// (reader task still aborted) → `on_transport_event(Disconnected)` → `break` again.
/// The TUI was permanently stuck `[daemon: offline]` with `ipc_tx = None`.
///
/// # What this function does
///
/// 1. Polls `<runtime_dir>/monocle.lock` via [`monocle_ipc::reconnect::poll_for_new_daemon`]
///    until a new daemon is detected (5-second interval — BC-2.05.006 PC-5).
///    This call blocks until a new daemon lock file is detected, satisfying EC-002
///    (daemon never restarts → indefinite poll, no busy-spin, no crash).
/// 2. Sets `app.status_message = Some(DAEMON_DISCONNECT_STATUS)` — reconnecting indicator.
/// 3. Re-enters [`reconnect_to_daemon`] with a fresh [`BackoffState`] (backoff resets).
/// 4. On successful reconnect: reads the fresh `InitialState` push (BC-2.05.006 PC-6),
///    calls `on_initial_state` to rebuild TUI state, clears `app.status_message = None`
///    (BC-2.05.006 PC-8), and wires the new stream via `setup_ipc_streams_with_rx`.
/// 5. Returns the new `(reader_handle, writer_handle, ipc_rx)` for the caller to shadow
///    its existing handles.
///
/// # If reconnect_to_daemon times out again
///
/// The TUI re-enters offline mode: `poll_for_new_daemon` is called again.  This loop
/// continues indefinitely, satisfying EC-002 without busy-spin (each iteration sleeps
/// for 5 seconds via `poll_for_new_daemon` before any reconnect attempt).
///
/// # Cancellation
///
/// The returned future is NOT cancel-safe on the `poll_for_new_daemon` leg — dropping
/// it mid-poll will restart the poll discriminant (no state corruption, but the next
/// poll will reload the initial discriminant).  This is acceptable because the TUI only
/// calls this function during daemon-offline mode when no meaningful state transitions
/// can occur.
///
/// # Extracted seam for testability
///
/// This function is extracted from `run()` following the precedent of
/// `setup_ipc_streams_with_rx` and `reconnect_to_daemon`.  Integration tests drive
/// the offline→detect→reconnect transition without needing a live terminal.
pub async fn reconnect_from_offline(
    runtime_dir: &std::path::Path,
    sock_path: &std::path::Path,
    app: &mut App,
) -> anyhow::Result<(
    tokio::task::JoinHandle<()>,
    tokio::task::JoinHandle<()>,
    tokio::sync::mpsc::Receiver<Result<ServerToClient, IpcError>>,
)> {
    // BC-2.05.006 PC-5 step 5: loop until a successful reconnect.
    // Each iteration: poll for new daemon → attempt reconnect → on success return.
    // On another timeout: poll again (EC-002 — daemon never restarts = indefinite loop).
    loop {
        // 1. Poll until a new daemon is detected (blocks on 5s interval).
        //    On initial call with None discriminant, poll_for_new_daemon reads the lock
        //    file at call time as the baseline and returns as soon as it changes.
        //    If the lock file is already fresh (new daemon already present), it may
        //    return immediately on the next poll tick.
        monocle_ipc::reconnect::poll_for_new_daemon(runtime_dir).await;
        tracing::info!(
            "offline mode: new daemon detected — re-entering reconnect loop \
             (BC-2.05.006 PC-5 step 5)"
        );

        // 2. Show reconnecting indicator (BC-2.05.006 PC-2).
        app.status_message = Some(DAEMON_DISCONNECT_STATUS.to_string());

        // 3. Re-enter reconnect with a fresh BackoffState (backoff resets — BC-2.05.006 PC-5).
        let mut fresh_backoff = BackoffState::new();
        match reconnect_to_daemon(sock_path, &mut fresh_backoff).await {
            Ok(mut new_stream) => {
                // 4. Read fresh InitialState (BC-2.05.006 PC-6).
                match read_framed::<_, ServerToClient>(&mut new_stream).await {
                    Ok(ServerToClient::InitialState {
                        sessions,
                        ring_tail,
                        overlay_stack: overlay,
                        drop_counter,
                    }) => {
                        on_initial_state(app, sessions, ring_tail, overlay, drop_counter);
                        // BC-2.05.006 PC-8: clear status bar on success.
                        app.status_message = None;
                        tracing::info!(
                            "offline reconnect succeeded — TUI state rebuilt from fresh InitialState"
                        );
                    }
                    Ok(other) => {
                        tracing::error!(
                            unexpected_message = ?other,
                            "offline reconnect: first message not InitialState \
                             (BC-2.05.002 Inv 1) — dropping reconnected stream, re-entering offline mode"
                        );
                        app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                        // Re-loop: poll again (treat as if reconnect failed).
                        continue;
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "offline reconnect: failed to read InitialState — re-entering offline mode"
                        );
                        app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                        continue;
                    }
                }

                // 5. Wire the new stream (BC-2.05.006 PC-6 / F-WAVE6-GATE-CRIT-001 fix).
                //    split → spawn reader/writer tasks → set app.ipc_tx = Some(...).
                let (ns_read, ns_write) = new_stream.into_split();
                let (rh, wh, rx) = setup_ipc_streams_with_rx(app, ns_read, ns_write);
                return Ok((rh, wh, rx));
            }
            Err(IpcError::ReconnectTimeout) => {
                // 5-second window exhausted again — re-poll (EC-002 indefinite loop).
                tracing::warn!(
                    "offline reconnect: 5s window exhausted again — re-entering offline poll \
                     (BC-2.05.006 EC-002 path)"
                );
                app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                // Loop back to poll_for_new_daemon (no busy-spin — poll_for_new_daemon sleeps).
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "offline reconnect: unexpected error — re-entering offline poll"
                );
                app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                // Loop back (unexpected error treated like timeout for robustness).
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Profile picker handlers (S-031, BC-2.07.004/005)
// ---------------------------------------------------------------------------

/// Open the profile picker for a specific `current_dir`.
///
/// Core implementation — populates a `ProfilePickerState` from `config.harness_profiles`,
/// sorted alphabetically, with per-directory pre-selection via
/// `resolve_profile_for_dir(&config, current_dir)` (BC-2.07.005 PC-4 / MAJOR-1 fix).
///
/// Called by `open_profile_picker` (which resolves CWD automatically) and by tests
/// that need deterministic per-directory pre-selection.
///
/// # Idempotency (BC-2.07.005 EC-110)
///
/// If `app.profile_picker` is already `Some(...)`, this is a no-op — only one picker
/// instance is active at a time. The second Ctrl-P keypress does NOT replace the picker.
///
/// # AppMode contract (BC-2.07.005 PC-1 / BC-2.07.004 INV-1)
///
/// Does NOT change `app.mode`. The picker coexists over any AppMode.
///
/// # Default selection (BC-2.07.005 PC-4 / BC-2.07.004 INV-1)
///
/// Pre-selects the sticky profile for `current_dir` via `resolve_profile_for_dir`.
/// If no sticky entry exists for this dir, index 0 is used (first profile in sorted order).
/// This is a per-directory lookup — NOT a first-match over all `project_profiles.values()`
/// (that was the MAJOR-1 defect: HashMap value iteration is non-deterministic).
pub fn open_profile_picker_with_dir(app: &mut App, current_dir: &str) {
    use monocle_config::resolve_profile_for_dir;

    // BC-2.07.005 EC-110: idempotent — if already open, do nothing.
    if app.profile_picker.is_some() {
        return;
    }

    // Build sorted snapshot of profile IDs (AC-001 / BC-2.07.004 PC-1).
    let mut profiles: Vec<String> = app
        .config
        .harness_profiles
        .iter()
        .map(|p| p.id.clone())
        .collect();
    profiles.sort();

    // BC-2.07.005 PC-4 / MAJOR-1 fix: pre-select the sticky profile for THIS DIRECTORY.
    // Use resolve_profile_for_dir (pure, no I/O) keyed by current_dir — NOT a first-match
    // over all project_profiles.values() (which was non-deterministic).
    let selected_index = resolve_profile_for_dir(&app.config, current_dir)
        .and_then(|profile| profiles.iter().position(|id| id == &profile.id))
        .unwrap_or(0);

    app.profile_picker = Some(ProfilePickerState {
        selected_index,
        profiles,
        current_dir: current_dir.to_string(),
    });
}

/// Open the profile picker using the process's current working directory for pre-selection.
///
/// Resolves `std::env::current_dir()` (verbatim, no canonicalization — BC-2.07.004 INV-1 /
/// BC-2.07.005 INV-5) and delegates to [`open_profile_picker_with_dir`].
/// Called by `dispatch_key_event` when `Action::ProfilePicker` fires (Ctrl-P).
///
/// # Pre-selection strategy (BC-2.07.005 PC-4)
///
/// Pre-selection is determined exclusively by `resolve_profile_for_dir(&config, &cwd)`.
/// If the CWD has no sticky entry, pre-selection falls back to index 0 (first profile
/// in sorted order). There is no fallback over `project_profiles.values()` — such a
/// fallback would produce non-deterministic behaviour when multiple project entries exist.
///
/// # Idempotency (BC-2.07.005 EC-110)
/// If picker is already open, this is a no-op (handled by the delegate).
pub fn open_profile_picker(app: &mut App) {
    // BC-2.07.004 INV-1 / BC-2.07.005 INV-5: verbatim CWD, no canonicalization.
    let current_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    open_profile_picker_with_dir(app, &current_dir);
}

/// Close the profile picker without committing any selection.
///
/// Called when `Esc` is pressed while `app.profile_picker` is `Some(..)`.
/// Sets `app.profile_picker = None`. Does not call `write_config`. Does not
/// change `app.mode` (BC-2.07.005 PC-8).
pub fn close_profile_picker(app: &mut App) {
    app.profile_picker = None;
}

/// Commit the currently highlighted profile selection using the canonical config path.
///
/// Convenience wrapper around [`commit_profile_selection_with_path`] that resolves
/// `MonocleConfig::config_path()` automatically. Called by `dispatch_key_event` on
/// Enter while the picker is open — the production path always writes to the real
/// config file.
///
/// `current_dir` must be the verbatim, non-symlink-resolved CWD string
/// (BC-2.07.004 INV-1 / BC-2.07.005 INV-5 normalization contract).
///
/// If no picker is open, this is a no-op.
pub fn commit_profile_selection(app: &mut App, current_dir: &str) {
    // BC-2.07.005 PC-5 / MAJOR-1 guard: hoist empty-CWD check BEFORE config_path() so
    // BOTH the Ok and Err branches are protected by a single early return. Without this
    // guard the Err branch would proceed to insert project_profiles[""] = id — silent
    // config corruption (INV-5 normalization contract).
    if current_dir.is_empty() {
        app.status_message = Some("Config save failed: CWD resolution failed".to_string());
        app.profile_picker = None;
        return;
    }

    let path_result = MonocleConfig::config_path();
    match path_result {
        Ok(path) => commit_profile_selection_with_path(app, current_dir, &path),
        Err(e) => {
            // config_path() failed (no HOME dir etc.) — treat as a write failure.
            // In-memory update still happens: get selected_id first if picker is open.
            let selected_id = app
                .profile_picker
                .as_ref()
                .and_then(|s| s.profiles.get(s.selected_index))
                .cloned();
            if let Some(id) = selected_id {
                // EC-106 guard: only write if there IS a selected profile (non-empty list).
                if !id.is_empty() {
                    app.config
                        .project_profiles
                        .insert(current_dir.to_string(), id.clone());
                }
                tracing::warn!(
                    error = %e,
                    profile = %id,
                    "commit_profile_selection: config_path() failed — in-memory profile updated, \
                     persistence failed (BC-2.07.005 PC-5c)"
                );
                app.status_message = Some(format!("Config save failed: {e}"));
            }
            app.profile_picker = None;
            app.ccr_path = detect_ccr(&app.config);
        }
    }
}

/// Commit the currently highlighted profile selection, writing to `config_path`.
///
/// Core implementation — testable via an injected `config_path` that can be forced to
/// fail (write-failure seam, AC-006 / BC-2.07.005 PC-5c).
///
/// Implements BC-2.07.005 PC-5:
/// 1. EC-106 guard: if `harness_profiles` is empty or selected index is out of range,
///    close picker WITHOUT writing (no empty-string project_profiles entry).
/// 2. Write selected profile ID into `config.project_profiles[current_dir]` (in-memory).
/// 3. Call `write_config(&config, config_path)` (atomic write — AC-009 / BC-2.07.005 INV-2).
/// 4. On `Err`: render transient error notification in `app.status_message`; in-memory
///    profile is still updated (BC-2.07.005 PC-5c — decoupled from write success).
/// 5. Set `app.profile_picker = None`.
/// 6. Log `INFO: profile switched to <name>`.
/// 7. Call `detect_ccr(&config)` and update `app.ccr_path` (AC-007 / BC-2.07.005 PC-3).
///
/// `current_dir` must be the verbatim, non-symlink-resolved CWD string
/// (BC-2.07.004 INV-1 / BC-2.07.005 INV-5 normalization contract).
///
/// If no picker is open, this is a no-op.
pub fn commit_profile_selection_with_path(
    app: &mut App,
    current_dir: &str,
    config_path: &std::path::Path,
) {
    // BC-2.07.005 PC-5 / MAJOR-2 guard: empty current_dir means CWD resolution failed.
    // Writing project_profiles[""] would silently corrupt the config — never persisted.
    if current_dir.is_empty() {
        app.status_message = Some("Config save failed: CWD resolution failed".to_string());
        app.profile_picker = None;
        return;
    }

    // Guard: if picker is not open, nothing to commit.
    let selected_id = match app.profile_picker.as_ref() {
        Some(state) => {
            match state.profiles.get(state.selected_index) {
                Some(id) => id.clone(),
                // EC-106: selected_index out of range (empty profiles or invalid index).
                // Close picker without writing any project_profiles entry.
                None => {
                    app.profile_picker = None;
                    return;
                }
            }
        }
        None => return,
    };

    // EC-106: if selected_id is empty (should not happen with valid profiles, but guard
    // defensively), close without writing an empty-string entry.
    if selected_id.is_empty() {
        app.profile_picker = None;
        return;
    }

    // BC-2.07.005 PC-5a: write selected profile ID into project_profiles[current_dir] (in-memory).
    app.config
        .project_profiles
        .insert(current_dir.to_string(), selected_id.clone());

    // BC-2.07.005 PC-5b: atomic write via write_config (AC-009 / BC-2.07.005 INV-2).
    let write_result = write_config(&app.config, config_path);

    if let Err(ref e) = write_result {
        // BC-2.07.005 PC-5c: on write failure, set transient error notification.
        // In-memory profile IS already updated above — intentional per BC (decoupled).
        tracing::warn!(
            error = %e,
            profile = %selected_id,
            "commit_profile_selection_with_path: write_config failed — in-memory profile \
             updated, persistence failed (BC-2.07.005 PC-5c)"
        );
        app.status_message = Some(format!("Config save failed: {e}"));
    }

    // BC-2.07.005 PC-4 step 3 / AC-005: close picker regardless of write outcome.
    app.profile_picker = None;

    // BC-2.07.005 PC-4 step 4 / AC-005: log profile switch.
    tracing::info!(profile = %selected_id, "profile switched to {}", selected_id);

    // BC-2.07.005 PC-3 / AC-007: update ccr_path after switch (success or failure).
    app.ccr_path = detect_ccr(&app.config);
    match &app.ccr_path {
        Some(path) => {
            tracing::info!(ccr_path = %path.display(), "ccr_path resolved to {}", path.display());
        }
        None => {
            tracing::warn!(
                profile = %selected_id,
                "ccr_path not found for profile {}",
                selected_id
            );
        }
    }
}

/// Navigate the picker selection down one row (wraps to top).
///
/// Called when `j` / `↓` is pressed while `app.profile_picker` is `Some(..)`.
/// A no-op if `profile_picker` is `None` or `profiles` is empty.
pub fn picker_select_next(app: &mut App) {
    if let Some(ref mut state) = app.profile_picker {
        if state.profiles.is_empty() {
            return;
        }
        // Wrap: len-1 → 0.
        state.selected_index = (state.selected_index + 1) % state.profiles.len();
    }
}

/// Navigate the picker selection up one row (wraps to bottom).
///
/// Called when `k` / `↑` is pressed while `app.profile_picker` is `Some(..)`.
/// A no-op if `profile_picker` is `None` or `profiles` is empty.
pub fn picker_select_prev(app: &mut App) {
    if let Some(ref mut state) = app.profile_picker {
        if state.profiles.is_empty() {
            return;
        }
        // Wrap: 0 → len-1.
        if state.selected_index == 0 {
            state.selected_index = state.profiles.len() - 1;
        } else {
            state.selected_index -= 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Main async run loop
// ---------------------------------------------------------------------------

/// Run the TUI event loop.
///
/// Called by `main()` after terminal setup. Connects to the monocle daemon
/// UDS, loads config, and drives the render+event loop until exit.
///
/// # Exit paths
///
/// - `q` from `Dashboard` mode → clean exit (status 0); `Esc` is context-sensitive
///   and does NOT quit (per F-S025-ADV2-HIGH-002).
/// - IPC connection failure → renders error panel, exits with code 1 after
///   any key press (AC-002).
/// - `TransportEvent::Disconnected` → transitions to reconnect mode (AC-003).
///   Does NOT exit.
pub async fn run() -> Result<()> {
    use crossterm::event::{self, Event};
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;
    use std::time::Duration;

    // AC-004: load config with graceful fallback.
    let config = match MonocleConfig::config_path() {
        Err(e) => {
            tracing::error!(error = %e, "config_path() failed; using defaults");
            MonocleConfig::default()
        }
        Ok(path) => match load_config(&path) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!(error = %e, "config load failed; using defaults");
                MonocleConfig::default()
            }
        },
    };

    // AC-002: attempt UDS connection.
    let runtime_dir = resolve_runtime_dir()?;
    let sock_path = runtime_dir.join("monocle.sock");
    let mut transport = match tokio::net::UnixStream::connect(&sock_path).await {
        Ok(t) => t,
        Err(e) => {
            // Connection failed — render the error panel and wait for any keypress.
            tracing::error!(error = %e, "daemon connection failed");
            let backend = CrosstermBackend::new(io::stdout());
            let mut terminal = Terminal::new(backend)?;
            terminal.draw(|frame| {
                use ratatui::text::Text;
                use ratatui::widgets::{Block, Borders, Paragraph};
                let error_msg = DAEMON_NOT_RUNNING_ERROR;
                let p = Paragraph::new(Text::raw(error_msg))
                    .block(Block::default().borders(Borders::ALL).title("Error"));
                frame.render_widget(p, frame.area());
            })?;
            // Wait for any keypress before returning the error (AC-002).
            // Return Err instead of std::process::exit(1) so main() can call
            // restore_terminal() before exiting — prevents terminal raw-mode leak
            // (F-S025-ADV1-BLOCKER-001).
            loop {
                if event::poll(Duration::from_millis(200))? {
                    if let Event::Key(_) = event::read()? {
                        break;
                    }
                }
            }
            return Err(anyhow::anyhow!("daemon unavailable: {e}"));
        }
    };

    let mut app = App::new(config);

    // F-PASS4-MED-001: create the app event channel for internal task → event-loop delivery.
    // Bounded at 64 to match IPC_READER_CHANNEL_CAPACITY; dump-window timeout events are
    // rare (one per attach window) so this provides ample headroom.
    // The receiver is drained in the main event loop alongside ipc_rx.
    let (app_event_tx, mut app_event_rx) =
        tokio::sync::mpsc::channel::<AppEvent>(IPC_READER_CHANNEL_CAPACITY);
    app.app_event_tx = Some(app_event_tx);

    // AC-008: receive and process InitialState from daemon.
    // The daemon sends InitialState immediately after connection.
    let initial = read_framed::<_, ServerToClient>(&mut transport).await;
    match initial {
        Ok(ServerToClient::InitialState {
            sessions,
            ring_tail,
            overlay_stack,
            drop_counter,
        }) => {
            on_initial_state(&mut app, sessions, ring_tail, overlay_stack, drop_counter);
        }
        Ok(other) => {
            // BC-2.05.002 Invariant 1: the first message from the daemon MUST be
            // InitialState. Any other message variant signals a protocol violation —
            // silent continuation is forbidden (F-S025-ADV1-MED-001).
            tracing::error!(
                unexpected_message = ?other,
                "BC-2.05.002 Inv 1 violation: first message was not InitialState; \
                 closing connection"
            );
            return Err(anyhow::anyhow!(
                "protocol violation: first message not InitialState (BC-2.05.002 Invariant 1)"
            ));
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to receive InitialState; continuing with empty state");
        }
    }

    // Wire the IPC connection: split the stream, spawn reader + writer tasks, set app.ipc_tx.
    //
    // F-S026-ADV5-CRIT-001 fix: the original code moved `transport` whole into
    // `spawn_ipc_reader` (read-only task), leaving no writer half and no outbound channel.
    // `app.ipc_tx` was permanently `None`, causing every `send_permission_decision` call
    // to WARN and silently discard the message.
    //
    // `setup_ipc_streams_with_rx` fixes this by:
    // 1. Receiving (read_half, write_half) from `transport.into_split()`.
    // 2. Spawning `spawn_ipc_reader(read_half, inbound_tx)` — inbound frames.
    // 3. Creating a bounded cmd channel (N=32) + spawning `spawn_ipc_writer(write_half, cmd_rx)`.
    // 4. Assigning `app.ipc_tx = Some(cmd_tx)` — outbound decisions are now wired.
    // 5. Returning the inbound receiver (`inbound_rx`) for the event loop drain loop.
    //
    // Channel capacity:
    //   - Inbound (ServerToClient): N=64 — matches original; see spawn_ipc_reader doc.
    //   - Outbound (ClientToServer): N=32 — rare user keypresses; see IPC_CMD_CHANNEL_CAPACITY.
    //
    // Reconnect: on disconnect, abort both handles, split the new stream, call
    // setup_ipc_streams_with_rx; `ipc_rx` is shadowed and `app.ipc_tx` is replaced.
    let (transport_read, transport_write) = transport.into_split();
    let (mut reader_handle, mut writer_handle, mut ipc_rx) =
        setup_ipc_streams_with_rx(&mut app, transport_read, transport_write);

    // Set up the ratatui terminal.
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Render state for the Sessions panel (selection tracking).
    let mut sessions_state = crate::ui::sessions_panel::SessionsPanelState::default();

    // Build the builtin binding layers once for the session (AC-006, BLOCKER-002).
    // Future: merge user-custom and per-context layers from config.
    let binding_layers = build_builtin_binding_layers();

    // Main event loop (100ms tick rate — AC-009 / BC-2.06.020: timer updates for
    // the overlay "Waiting: Ns" elapsed timer require 100ms granularity).
    // 100ms is also the keyboard poll ceiling; key response latency is acceptable
    // for a permission overlay workflow where decisions are deliberate, not rapid.
    let tick_rate = Duration::from_millis(100);

    loop {
        // 1. Render the current frame (AC-001, AC-005, BLOCKER-004, BC-2.06.007 PC-7).
        terminal.draw(|frame| {
            render_frame(&mut app, &mut sessions_state, frame);
        })?;

        // 2. Poll keyboard (non-blocking, bounded by tick_rate — BLOCKER-002: full binding
        //    dispatch via resolve_binding). The 16ms ceiling is unchanged from the original
        //    implementation; the 1ms was only in the removed timeout wrapper.
        if event::poll(tick_rate)? {
            if let Event::Key(ct_key) = event::read()? {
                // Convert crossterm KeyEvent → monocle-core KeyEvent (pure-core type).
                let core_key = crossterm_key_to_core(&ct_key);

                // Dispatch through the full binding chain (F-S025-ADV4-MED-004:
                // extracted to `dispatch_key_event` for testability — tests call
                // the same function rather than duplicating the gate logic).
                if dispatch_key_event(&mut app, &core_key, &binding_layers, &mut sessions_state)
                    == KeyOutcome::Quit
                {
                    break;
                }
            }
        }

        // 3. Drain IPC channel — non-blocking try_recv; process all available messages
        //    this tick (Option B — F-S025-ADV2-BLOCKER-001 fix; replaces the removed
        //    `tokio::time::timeout(Duration::from_millis(1), read_framed(...))` wrapper).
        loop {
            use tokio::sync::mpsc::error::TryRecvError;

            match ipc_rx.try_recv() {
                Ok(Ok(msg)) => {
                    if let Err(e) = handle_server_message(&mut app, msg) {
                        // Fatal protocol violation (e.g., duplicate InitialState).
                        tracing::error!(error = %e, "fatal protocol error; closing IPC connection");
                        // SOQ-3: clear overlay stack before reconnect (BC-2.05.007 Invariant 1).
                        on_transport_event(&mut app, TransportEvent::Disconnected);
                        reader_handle.abort();
                        writer_handle.abort();
                        // F-S026-ADV6-MED-001: clear ipc_tx so the Some⇒wired invariant holds
                        // while the writer task is dead. setup_ipc_streams_with_rx re-assigns
                        // Some(new_cmd_tx) on a successful reconnect.
                        app.ipc_tx = None;

                        // Reconnect with exponential backoff (BC-2.05.006 PC-4).
                        let mut backoff = BackoffState::new();
                        match reconnect_to_daemon(&sock_path, &mut backoff).await {
                            Ok(mut new_stream) => {
                                // Re-read InitialState from the fresh connection
                                // (BC-2.05.006 PC-6: TUI rebuilds from new InitialState).
                                match read_framed::<_, ServerToClient>(&mut new_stream).await {
                                    Ok(ServerToClient::InitialState {
                                        sessions,
                                        ring_tail,
                                        overlay_stack: overlay,
                                        drop_counter,
                                    }) => {
                                        on_initial_state(
                                            &mut app,
                                            sessions,
                                            ring_tail,
                                            overlay,
                                            drop_counter,
                                        );
                                        // BC-2.05.006 PC-8: clear reconnect status bar on success.
                                        app.status_message = None;
                                        tracing::info!("reconnect succeeded — TUI state rebuilt");
                                    }
                                    Ok(other) => {
                                        tracing::error!(
                                            unexpected_message = ?other,
                                            "reconnect: first message not InitialState \
                                             (BC-2.05.002 Inv 1) — dropping reconnected stream"
                                        );
                                        app.status_message =
                                            Some(DAEMON_OFFLINE_STATUS.to_string());
                                        break;
                                    }
                                    Err(e2) => {
                                        tracing::warn!(
                                            error = %e2,
                                            "reconnect: failed to read InitialState from fresh \
                                             connection — entering offline mode"
                                        );
                                        app.status_message =
                                            Some(DAEMON_OFFLINE_STATUS.to_string());
                                        break;
                                    }
                                }
                                // Wire the new stream (F-S026-ADV5-CRIT-001): split into
                                // read/write halves, spawn both tasks, set app.ipc_tx.
                                let (ns_read, ns_write) = new_stream.into_split();
                                let (rh2, wh2, rx2) =
                                    setup_ipc_streams_with_rx(&mut app, ns_read, ns_write);
                                reader_handle = rh2;
                                writer_handle = wh2;
                                ipc_rx = rx2;
                            }
                            Err(IpcError::ReconnectTimeout) => {
                                tracing::warn!(
                                    "reconnect_to_daemon: 5s window exhausted — offline mode \
                                     (BC-2.05.006 PC-5)"
                                );
                                // Offline mode: poll for new daemon then RE-ENTER reconnect loop
                                // (BC-2.05.006 PC-5 step 5 — F-WAVE6-GATE-CRIT-001 fix).
                                // reconnect_from_offline blocks on poll_for_new_daemon (5s interval)
                                // then reconnects with fresh BackoffState. On success it rewires
                                // reader/writer/ipc_rx. On repeated timeout it loops indefinitely
                                // (EC-002: no busy-spin, no crash).
                                app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                                match reconnect_from_offline(&runtime_dir, &sock_path, &mut app)
                                    .await
                                {
                                    Ok((rh2, wh2, rx2)) => {
                                        reader_handle = rh2;
                                        writer_handle = wh2;
                                        ipc_rx = rx2;
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            error = %e,
                                            "reconnect_from_offline: fatal error — breaking event loop"
                                        );
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    error = %e,
                                    "reconnect_to_daemon: unexpected error — entering offline mode \
                                     (BC-2.05.006 PC-5 step 5 via reconnect_from_offline)"
                                );
                                app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                                match reconnect_from_offline(&runtime_dir, &sock_path, &mut app)
                                    .await
                                {
                                    Ok((rh2, wh2, rx2)) => {
                                        reader_handle = rh2;
                                        writer_handle = wh2;
                                        ipc_rx = rx2;
                                    }
                                    Err(e2) => {
                                        tracing::error!(
                                            error = %e2,
                                            "reconnect_from_offline: fatal error — breaking event loop"
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    // Reader task forwarded a disconnect or transport error.
                    tracing::warn!(error = %e, "IPC reader task disconnect; entering reconnect state");
                    // SOQ-3: clear overlay stack before reconnect (BC-2.05.007 Invariant 1).
                    on_transport_event(&mut app, TransportEvent::Disconnected);
                    reader_handle.abort();
                    writer_handle.abort();
                    // F-S026-ADV6-MED-001: clear ipc_tx so the Some⇒wired invariant holds
                    // while the writer task is dead. setup_ipc_streams_with_rx re-assigns
                    // Some(new_cmd_tx) on a successful reconnect.
                    app.ipc_tx = None;

                    // Reconnect with exponential backoff (BC-2.05.006 PC-4).
                    let mut backoff = BackoffState::new();
                    match reconnect_to_daemon(&sock_path, &mut backoff).await {
                        Ok(mut new_stream) => {
                            // Re-read InitialState from the fresh connection
                            // (BC-2.05.006 PC-6: TUI rebuilds from new InitialState).
                            match read_framed::<_, ServerToClient>(&mut new_stream).await {
                                Ok(ServerToClient::InitialState {
                                    sessions,
                                    ring_tail,
                                    overlay_stack: overlay,
                                    drop_counter,
                                }) => {
                                    on_initial_state(
                                        &mut app,
                                        sessions,
                                        ring_tail,
                                        overlay,
                                        drop_counter,
                                    );
                                    // BC-2.05.006 PC-8: clear reconnect status bar on success.
                                    app.status_message = None;
                                    tracing::info!("reconnect succeeded — TUI state rebuilt");
                                }
                                Ok(other) => {
                                    tracing::error!(
                                        unexpected_message = ?other,
                                        "reconnect: first message not InitialState \
                                         (BC-2.05.002 Inv 1) — dropping reconnected stream"
                                    );
                                    app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                                    break;
                                }
                                Err(e2) => {
                                    tracing::warn!(
                                        error = %e2,
                                        "reconnect: failed to read InitialState from fresh \
                                         connection — entering offline mode"
                                    );
                                    app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                                    break;
                                }
                            }
                            // Wire the new stream (F-S026-ADV5-CRIT-001): split into
                            // read/write halves, spawn both tasks, set app.ipc_tx.
                            let (ns_read, ns_write) = new_stream.into_split();
                            let (rh2, wh2, rx2) =
                                setup_ipc_streams_with_rx(&mut app, ns_read, ns_write);
                            reader_handle = rh2;
                            writer_handle = wh2;
                            ipc_rx = rx2;
                        }
                        Err(IpcError::ReconnectTimeout) => {
                            tracing::warn!(
                                "reconnect_to_daemon: 5s window exhausted — offline mode \
                                 (BC-2.05.006 PC-5)"
                            );
                            // Offline mode: poll for new daemon then RE-ENTER reconnect loop
                            // (BC-2.05.006 PC-5 step 5 — F-WAVE6-GATE-CRIT-001 fix).
                            app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                            match reconnect_from_offline(&runtime_dir, &sock_path, &mut app).await {
                                Ok((rh2, wh2, rx2)) => {
                                    reader_handle = rh2;
                                    writer_handle = wh2;
                                    ipc_rx = rx2;
                                }
                                Err(e) => {
                                    tracing::error!(
                                        error = %e,
                                        "reconnect_from_offline: fatal error — breaking event loop"
                                    );
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                error = %e,
                                "reconnect_to_daemon: unexpected error — entering offline mode \
                                 (BC-2.05.006 PC-5 step 5 via reconnect_from_offline)"
                            );
                            app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string());
                            match reconnect_from_offline(&runtime_dir, &sock_path, &mut app).await {
                                Ok((rh2, wh2, rx2)) => {
                                    reader_handle = rh2;
                                    writer_handle = wh2;
                                    ipc_rx = rx2;
                                }
                                Err(e2) => {
                                    tracing::error!(
                                        error = %e2,
                                        "reconnect_from_offline: fatal error — breaking event loop"
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    // No message available this tick — normal, proceed to next iteration.
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    // Reader task exited unexpectedly (should not happen except on TUI exit).
                    tracing::warn!("IPC reader task channel disconnected unexpectedly");
                    on_transport_event(&mut app, TransportEvent::Disconnected);
                    break;
                }
            }
        }

        // 4. Drain app event channel — non-blocking try_recv; process all internal events
        //    this tick (dump-window timeouts, etc. — F-PASS4-MED-001).
        loop {
            use tokio::sync::mpsc::error::TryRecvError;
            match app_event_rx.try_recv() {
                Ok(AppEvent::DumpWindowTimeout { session_id }) => {
                    on_dump_window_timeout(&mut app, session_id);
                }
                Err(TryRecvError::Empty) => {
                    // No app event this tick — normal.
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    // app_event_tx dropped — should only happen on TUI exit.
                    tracing::debug!("app_event_rx: channel disconnected; no more internal events");
                    break;
                }
            }
        }
    }

    // Clean exit: abort both IPC tasks before returning so the tokio runtime doesn't
    // leak background tasks between test runs or on graceful shutdown.
    reader_handle.abort();
    writer_handle.abort();

    Ok(())
}

/// Dispatch an incoming `ServerToClient` message to the appropriate handler.
///
/// Returns `Ok(())` on successful dispatch, or `Err` if the message represents
/// a fatal protocol violation (e.g., duplicate `InitialState`). The event loop
/// treats an `Err` return as a connection-close signal.
///
/// # Integration test seam
///
/// Exposed as `pub` (with `#[doc(hidden)]`) so that external integration tests in
/// `crates/monocle-tui/tests/` can call the real inbound dispatch router directly
/// after injecting messages through the `inbound_rx` channel obtained from
/// [`setup_ipc_streams_with_rx`]. Not part of the stable public API.
#[doc(hidden)]
pub fn handle_server_message(app: &mut App, msg: ServerToClient) -> Result<()> {
    match msg {
        ServerToClient::InitialState { .. } => {
            // BC-2.05.002 Invariant 1: a second InitialState on an already-initialized
            // connection signals daemon-side state machine corruption or a protocol
            // violation. Silent continuation would cause TUI state to diverge from
            // daemon reality. Log an error and close the connection.
            tracing::error!(
                "BC-2.05.002 Inv 1 violation: duplicate InitialState received; \
                 closing IPC connection to prevent state divergence"
            );
            return Err(anyhow::anyhow!(
                "protocol violation: duplicate InitialState (BC-2.05.002 Invariant 1)"
            ));
        }
        ServerToClient::SessionListUpdate { sessions } => {
            // F-S039-001 (BC-2.09.001 Inv-5): create parsers for newly-appeared sessions.
            // Iterate the NEW roster; for each session not yet in pty_parsers, create one.
            // Do NOT clobber parsers for sessions already present (no-clobber invariant).
            let scrollback_rows = app.scrollback_rows as usize;
            for session in &sessions {
                let id = &session.session_id;
                if !app.pty_parsers.contains_key(id) {
                    app.pty_parsers.insert(
                        id.clone(),
                        vt100::Parser::new(PTY_DEFAULT_ROWS, PTY_DEFAULT_COLS, scrollback_rows),
                    );
                    app.pty_scroll_offsets.insert(id.clone(), 0);
                }
            }

            // F-S039-003 (BC-2.09.001 Inv-5 / AC-008): GC sessions removed from the roster.
            // Build a set of IDs in the new roster, then GC any parser whose session_id is
            // absent. This catches sessions that were terminated and dropped from the list
            // without an explicit SessionStateChanged::Terminated signal.
            //
            // F-S039-REV-001: use gc_session_with_mode_exit (NOT gc_pty_session directly).
            // If the TUI is currently viewing a removed session in EmbeddedTerminal mode,
            // gc_session_with_mode_exit exits that mode first (restores Dashboard) before
            // destroying the parser. Calling gc_pty_session directly would leave the TUI
            // in EmbeddedTerminal with a destroyed parser → permanent "Connecting..." trap.
            let new_ids: HashSet<&str> = sessions.iter().map(|s| s.session_id.as_str()).collect();
            let removed: Vec<String> = app
                .sessions
                .iter()
                .filter(|s| !new_ids.contains(s.session_id.as_str()))
                .map(|s| s.session_id.clone())
                .collect();
            for id in removed {
                gc_session_with_mode_exit(app, &id);
            }

            app.sessions = sessions;
        }
        ServerToClient::DropCounterUpdate { drop_counter } => {
            on_drop_counter_update(app, drop_counter);
        }
        ServerToClient::PermissionPromptQueued { payload } => {
            on_permission_prompt_queued(app, payload);
        }
        ServerToClient::PermissionPromptResolved { prompt_id } => {
            on_permission_prompt_resolved(app, prompt_id);
        }
        ServerToClient::HookEventReceived {
            hook_type,
            session_id,
            payload_excerpt,
            latency_ms,
            timestamp_micros,
        } => {
            // S-028 (BC-2.05.004): delegate to on_hook_event_received for event ribbon
            // population. The handler appends to app.event_ribbon_events (all sessions;
            // client-side session filter applied at render time per BC-2.05.004 INV-3).
            // BC-2.05.004 PC-2 / SS-ipc: pass daemon's timestamp_micros through.
            on_hook_event_received(
                app,
                hook_type,
                session_id,
                payload_excerpt,
                latency_ms,
                timestamp_micros,
            );
        }
        // S-033 variants: handled by TUI stories S-033/S-036/S-037/S-047.
        // Stub arms prevent non-exhaustive match compilation failure.
        // The TUI implementation is authored by the respective story implementers.
        ServerToClient::SpawnAck { .. } => {
            // SpawnAck is received by the TUI after sending ClientToServer::SpawnSession.
            // The TUI uses the session_id to track the in-progress spawn.
            // Implementation: S-033 TUI story.
            tracing::debug!("SpawnAck received (S-033 stub — TUI handler not yet implemented)");
        }
        ServerToClient::SessionStateChanged {
            session_id,
            new_state,
        } => {
            // S-033 TUI story owns session panel state indicator updates.
            // F-S039-003 (BC-2.09.001 AC-008 / Inv-5): GC PTY state on Terminated.
            // SessionStateChanged::Terminated is the authoritative signal that a session's
            // process has exited. Invoke gc_session_with_mode_exit to exit EmbeddedTerminal
            // mode first (if focused on this session) then remove all per-session PTY
            // pipeline state (parser, scroll offset, dump flags, pending bytes).
            //
            // F-S039-REV-001: use gc_session_with_mode_exit (centralises exit-before-GC
            // logic; the matches! guard inside it ensures we only exit mode when the TUI
            // is actually viewing THIS session, not a different one).
            use monocle_ipc::types::SessionState;
            if matches!(new_state, SessionState::Terminated) {
                gc_session_with_mode_exit(app, &session_id);
                tracing::debug!(
                    session_id = %session_id,
                    "SessionStateChanged::Terminated — PTY pipeline state GC'd (F-S039-003 / F-S039-REV-001)"
                );
            } else {
                tracing::debug!(
                    session_id = %session_id,
                    new_state = ?new_state,
                    "SessionStateChanged received (S-033 stub — panel state indicator update not yet implemented)"
                );
            }
        }
        ServerToClient::Error { code, message } => {
            // Error is sent by the daemon when a lifecycle operation fails.
            // The TUI displays a banner or error indicator.
            // Implementation: S-033 TUI story.
            tracing::warn!("ServerToClient::Error {{ code: {code}, message: {message} }} received (S-033 stub)");
        }

        // -----------------------------------------------------------------------
        // S-039: PTY output pipeline (BC-2.09.001)
        // -----------------------------------------------------------------------
        ServerToClient::PtyOutput { session_id, bytes } => {
            // AC-001 (BC-2.09.001 PC-1): call on_pty_output within one mpsc cycle.
            on_pty_output(app, session_id, bytes);
            // F-S039-012 (AC-003 / BC-2.09.001 PC-3): request render tick after on_pty_output.
            // The run loop calls terminal.draw() at the top of each tick, so the next render
            // is guaranteed. This comment documents the AC-003 invariant explicitly rather than
            // relying on "draw is always called next."
            // request_render() is not a separate call here — the event-loop architecture
            // (draw at tick-start) satisfies AC-003 without an explicit trigger.
        }
        ServerToClient::ScrollbackDumpComplete {
            session_id,
            pty_rows,
            pty_cols,
            ..
        } => {
            // BC-2.09.001 Invariant 5: parser reset + buffered-bytes replay.
            // Owned by S-047 (variant defined there); consumed here per story S-039.
            on_scrollback_dump_complete(app, session_id, pty_rows, pty_cols);
        }
        ServerToClient::ScrollbackChunk { .. } => {
            // ScrollbackChunk is accumulated by the S-047 implementer before
            // ScrollbackDumpComplete arrives. S-039 stub: no-op (accumulation
            // logic belongs to the S-047 implementation of on_scrollback_dump_complete).
            tracing::trace!(
                "ScrollbackChunk received (S-047/S-039 stub — accumulation not yet implemented)"
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Key dispatch helper (extracted for testability — F-S025-ADV4-MED-004)
// ---------------------------------------------------------------------------

/// Outcome returned by [`dispatch_key_event`].
///
/// The run loop inspects this to decide whether to break the event loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyOutcome {
    /// The user triggered a quit action (e.g., `q` in Dashboard); the run loop
    /// should exit cleanly.
    Quit,
    /// Normal dispatch — continue the event loop.
    Continue,
}

// ---------------------------------------------------------------------------
// Permission decision send helper (BC-2.06.011/012/013)
// ---------------------------------------------------------------------------

/// Send a `ClientToServer::PermissionDecision` for the front overlay modal.
///
/// Uses `try_send` (bounded, non-blocking — BC-2.04.011). If the overlay stack
/// is empty or `ipc_tx` is not wired (pre-connection), a WARN is logged and no
/// message is sent. The overlay stack is NOT mutated — the modal stays visible
/// until `ServerToClient::PermissionPromptResolved` arrives (BC-2.06.023).
///
/// On channel-full (`TrySendError::Full`): logs WARN with drop signal. The
/// decision is discarded; the user can re-key. This matches the project's
/// bounded-channel policy (SS-conventions-anti-patterns.md §forbidden-patterns).
fn send_permission_decision(app: &mut App, decision: PermissionDecisionKind) {
    let Some(prompt_id) = app.overlay_stack.front().map(|m| m.prompt_id) else {
        tracing::warn!(
            "send_permission_decision: overlay_stack is empty; ignoring {:?} (BC-2.06.011/012/013)",
            decision
        );
        return;
    };
    let Some(tx) = app.ipc_tx.as_ref() else {
        tracing::warn!(
            "send_permission_decision: ipc_tx is None (not connected); ignoring {:?} for \
             prompt_id={} (BC-2.06.011/012/013)",
            decision,
            prompt_id
        );
        return;
    };
    let msg = ClientToServer::PermissionDecision {
        prompt_id,
        decision,
    };
    if let Err(e) = tx.try_send(msg) {
        tracing::warn!(
            "send_permission_decision: try_send failed for prompt_id={}: {} \
             (channel full or closed — BC-2.04.011 drop policy)",
            prompt_id,
            e
        );
    }
}

/// Dispatch a single key event through the full 5-level binding chain.
///
/// Extracted from `run()` so that integration tests can exercise the SAME code
/// path without spawning an async runtime or connecting to a real UDS socket
/// (F-S025-ADV4-MED-004: eliminates vacuous-mirror anti-pattern in AC-006 tests).
///
/// # Behaviour
///
/// 1. Resolves the key against `binding_layers` in the context of `app.mode`.
/// 2. For `SelectNext` / `SelectPrev`: no-op (Dashboard j/k/↑/↓ are shadowed by
///    per-context `ScrollDown`/`ScrollUp`; these actions only reach here from
///    non-Dashboard modes where no cursor mutation applies).
/// 3. For all other actions: drives the `AppMode` state machine via
///    `monocle_core::tui::state::transition()`.  Overlay-stack mutations
///    (`PopOverlay`, `OverlayCycleNext`) are applied at App-level here (see
///    F-S025-ADV2-HIGH-003).
/// 4. Returns [`KeyOutcome::Quit`] when an `Action::Quit` fires; the caller
///    is responsible for breaking the event loop.
pub fn dispatch_key_event(
    app: &mut App,
    core_key: &monocle_core::tui::binding::KeyEvent,
    binding_layers: &monocle_core::tui::binding::BindingLayers,
    sessions_state: &mut crate::ui::sessions_panel::SessionsPanelState,
) -> KeyOutcome {
    use monocle_core::tui::binding::{resolve_binding, KeyCode, KeyModifiers};
    use monocle_core::tui::state::{transition, Action};

    // ---------------------------------------------------------------------------
    // AC-004 / BC-2.07.005 PC-9: picker-open pre-check (S-031).
    //
    // When the picker is open ALL key events are consumed by the picker handler
    // BEFORE resolve_binding() runs. This enforces keyboard isolation — session
    // nav keys (Tab, j/k, Enter on sessions) do NOT fire while the picker is open.
    //
    // Routing while picker is open:
    //   ↓ / j → picker_select_next
    //   ↑ / k → picker_select_prev
    //   Enter  → commit_profile_selection (closes picker)
    //   Esc    → close_profile_picker (closes picker, no write)
    //   Ctrl-P → CONSUMED (idempotent: open_profile_picker is a no-op when open)
    //   all other keys → silently consumed (not forwarded to resolve_binding)
    // ---------------------------------------------------------------------------
    if app.profile_picker.is_some() {
        let no_mod = KeyModifiers::default();
        let is_down = (core_key.code == KeyCode::Down || core_key.code == KeyCode::Char('j'))
            && core_key.modifiers == no_mod;
        let is_up = (core_key.code == KeyCode::Up || core_key.code == KeyCode::Char('k'))
            && core_key.modifiers == no_mod;
        let is_enter = core_key.code == KeyCode::Enter && core_key.modifiers == no_mod;
        let is_esc = core_key.code == KeyCode::Esc && core_key.modifiers == no_mod;

        if is_down {
            picker_select_next(app);
        } else if is_up {
            picker_select_prev(app);
        } else if is_enter {
            // BC-2.07.004 INV-1 / BC-2.07.005 PC-5: use the open-time snapshot stored in
            // ProfilePickerState::current_dir so that pre-selection, the `*` active marker,
            // and the write key all use ONE source-of-truth directory.  Re-resolving
            // std::env::current_dir() here would break INV-1 if the process CWD changes
            // between open and Enter (e.g., in tests or shell integrations).
            let current_dir = app
                .profile_picker
                .as_ref()
                .map(|s| s.current_dir.clone())
                .unwrap_or_default();
            commit_profile_selection(app, &current_dir);
        } else if is_esc {
            close_profile_picker(app);
        }
        // All other keys (Tab, q, etc.) are silently consumed — isolation enforced.
        return KeyOutcome::Continue;
    }

    // S-028 (BC-2.06.006 INV-3): intercept Backspace in Filtering mode before
    // resolve_binding — backspace removes the last character from the query.
    // This is App-level mutation (query is a field in AppMode::Filtering, not in
    // the binding type system). No transition() call needed.
    if matches!(&app.mode, AppMode::Filtering { .. })
        && core_key.code == KeyCode::Backspace
        && !core_key.modifiers.ctrl
        && !core_key.modifiers.alt
    {
        if let AppMode::Filtering { ref mut query, .. } = app.mode {
            query.pop(); // BC-2.06.006 INV-3: removes last char; no-op on empty query
        }
        return KeyOutcome::Continue;
    }

    // BC-2.06.018 PC-5 / AC-007: intercept G / g / gg in Dashboard { EventRibbon } before
    // resolve_binding (the binding table has no entries for 'G' or 'g' in this context).
    // These are App-level mutations with no AppMode transition, matching the Backspace
    // interception pattern above.
    //
    // - 'G' (uppercase): jump to OLDEST event (last index in newest-first list), pinned_top=true.
    // - 'g' first press: set pending_key = Some('g') — no visible effect yet.
    // - 'g' second press (pending_key == Some('g')): jump to NEWEST event (row 0), pinned_top=false.
    // - Any other key while pending_key == Some('g'): clear pending_key and process normally.
    if matches!(
        &app.mode,
        AppMode::Dashboard {
            focused: FocusSnapshot::EventRibbon
        }
    ) && !core_key.modifiers.ctrl
        && !core_key.modifiers.alt
    {
        match core_key.code {
            KeyCode::Char('G') => {
                // G → jump to oldest (bottom of newest-first list).
                app.pending_key = None; // clear any pending prefix
                crate::ui::event_ribbon::jump_oldest(
                    &mut app.event_ribbon_state,
                    &app.event_ribbon_events,
                );
                return KeyOutcome::Continue;
            }
            KeyCode::Char('g') => {
                if app.pending_key == Some('g') {
                    // Second 'g': fire gg → jump to newest (row 0).
                    app.pending_key = None;
                    crate::ui::event_ribbon::jump_newest(&mut app.event_ribbon_state);
                } else {
                    // First 'g': enter pending-key state; no ribbon change yet.
                    app.pending_key = Some('g');
                }
                return KeyOutcome::Continue;
            }
            _ => {
                // Any other key while pending: clear pending_key and fall through
                // to normal resolve_binding dispatch.
                if app.pending_key.is_some() {
                    app.pending_key = None;
                }
            }
        }
    } else {
        // Not in Dashboard { EventRibbon } — clear any stale pending_key (mode switched).
        if app.pending_key.is_some() {
            app.pending_key = None;
        }
    }

    let resolved = resolve_binding(core_key, &app.mode, binding_layers);

    match resolved {
        Some((Action::Noop, _)) | None => KeyOutcome::Continue,

        Some((Action::SelectNext, _)) => {
            // ADV Pass-6 NITPICK-1: Dashboard j/↓ now resolves to Action::ScrollDown via
            // per-context binding (AppModeTag::Dashboard priority 3 > builtin priority 5),
            // so SelectNext is NEVER produced in Dashboard mode. In non-Dashboard modes
            // (Fullscreen, Overlay, Filtering) there is no cursor-move semantics — no-op.
            KeyOutcome::Continue
        }

        Some((Action::SelectPrev, _)) => {
            // ADV Pass-6 NITPICK-1: Dashboard k/↑ now resolves to Action::ScrollUp via
            // per-context binding (AppModeTag::Dashboard priority 3 > builtin priority 5),
            // so SelectPrev is NEVER produced in Dashboard mode. In non-Dashboard modes
            // (Fullscreen, Overlay, Filtering) there is no cursor-move semantics — no-op.
            KeyOutcome::Continue
        }

        Some((Action::FilterType(c), _)) => {
            // S-028 (BC-2.06.006 PC-2): append character to the Filtering mode query.
            // App-level mutation: transition() does not handle FilterType (query is not
            // in the AppMode type system — it is a field of AppMode::Filtering).
            if let AppMode::Filtering { ref mut query, .. } = app.mode {
                query.push(c);
            }
            KeyOutcome::Continue
        }

        Some((Action::ScrollDown, _)) => {
            // BC-2.06.018 AC-010 §2 (F-1 FIX): Action::ScrollDown is now the per-context
            // binding for j/↓ in ALL Dashboard focuses (AppModeTag::Dashboard cannot
            // discriminate by sub-focus). Dispatch checks the live focus:
            //   - Sessions focus (non-empty list) → sessions cursor down (same semantics as
            //     old SelectNext arm). Empty session list falls through to the ribbon arm.
            //   - EventRibbon focus (or any other) → scroll ribbon toward older events.
            match &app.mode {
                AppMode::Dashboard {
                    focused: FocusSnapshot::Sessions,
                } if !app.sessions.is_empty() => {
                    let len = app.sessions.len();
                    let prev_idx = sessions_state.list_state.selected();
                    let next = prev_idx.map(|i| (i + 1).min(len - 1)).unwrap_or(0);
                    sessions_state.list_state.select(Some(next));
                    // BC-2.06.018 AC-008: keep selected_session_id in sync with the
                    // Sessions cursor so on_hook_event_received can gate auto-scroll
                    // to the session the user is actually viewing.
                    app.selected_session_id = app.sessions.get(next).map(|s| s.session_id.clone());
                    // BC-2.06.018 INV-1 / AC-009: on session change, reset ribbon scroll.
                    if prev_idx != Some(next) {
                        crate::ui::event_ribbon::reset_on_session_change(
                            &mut app.event_ribbon_state,
                            app.selected_session_id.as_deref().unwrap_or(""),
                        );
                    }
                }
                _ => {
                    // EventRibbon focus, empty sessions, or non-Dashboard: scroll ribbon toward older events.
                    crate::ui::event_ribbon::scroll_ribbon_down(
                        &mut app.event_ribbon_state,
                        &app.event_ribbon_events,
                    );
                }
            }
            KeyOutcome::Continue
        }

        Some((Action::ScrollUp, _)) => {
            // BC-2.06.018 AC-010 §2 (F-1 FIX): Action::ScrollUp is now the per-context
            // binding for k/↑ in ALL Dashboard focuses. Dispatch checks the live focus:
            //   - Sessions focus → sessions cursor up (same semantics as old SelectPrev arm).
            //   - EventRibbon focus (or any other) → scroll ribbon toward newer events.
            match &app.mode {
                AppMode::Dashboard {
                    focused: FocusSnapshot::Sessions,
                } if !app.sessions.is_empty() => {
                    let prev_idx = sessions_state.list_state.selected();
                    let prev = prev_idx.map(|i| i.saturating_sub(1)).unwrap_or(0);
                    sessions_state.list_state.select(Some(prev));
                    // BC-2.06.018 AC-008: keep selected_session_id in sync with the
                    // Sessions cursor so on_hook_event_received can gate auto-scroll
                    // to the session the user is actually viewing.
                    app.selected_session_id = app.sessions.get(prev).map(|s| s.session_id.clone());
                    // BC-2.06.018 INV-1 / AC-009: on session change, reset ribbon scroll.
                    if prev_idx != Some(prev) {
                        crate::ui::event_ribbon::reset_on_session_change(
                            &mut app.event_ribbon_state,
                            app.selected_session_id.as_deref().unwrap_or(""),
                        );
                    }
                }
                _ => {
                    // EventRibbon focus, empty sessions, or non-Dashboard: scroll ribbon toward newer events.
                    crate::ui::event_ribbon::scroll_ribbon_up(
                        &mut app.event_ribbon_state,
                        &app.event_ribbon_events,
                    );
                }
            }
            KeyOutcome::Continue
        }

        Some((action, _)) => {
            // All other actions: drive the AppMode state machine.
            let is_quit = matches!(&action, Action::Quit);

            // F-S025-ADV2-HIGH-003: Overlay stack mutations are App-level.
            match &action {
                Action::PopOverlay => {
                    app.overlay_stack.pop_front();
                    // transition() collapses to Dashboard { prior }.
                    app.mode = transition(app.mode.clone(), action);
                    // Re-enter Overlay if stack still has items.
                    if !app.overlay_stack.is_empty() {
                        let prior = match &app.mode {
                            AppMode::Dashboard { focused } => focused.clone(),
                            _ => FocusSnapshot::Sessions,
                        };
                        app.mode = AppMode::Overlay { prior };
                    }
                }
                Action::OverlayCycleNext => {
                    // Rotate overlay_stack; transition() is identity.
                    if app.overlay_stack.len() > 1 {
                        if let Some(front) = app.overlay_stack.pop_front() {
                            app.overlay_stack.push_back(front);
                        }
                    }
                    app.mode = transition(app.mode.clone(), action);
                }
                // ---------------------------------------------------------------------------
                // S-026 — Permission decision key handlers (BC-2.06.011/012/013)
                //
                // All three arms:
                //   1. Look up `app.overlay_stack.front()` to get the target `prompt_id`.
                //   2. Enqueue `ClientToServer::PermissionDecision { prompt_id, decision }`
                //      on `app.ipc_tx` via `try_send` (bounded, non-blocking — BC-2.04.011).
                //   3. Do NOT pop or retain the overlay_stack — the modal stays visible
                //      until `ServerToClient::PermissionPromptResolved` arrives (BC-2.06.023).
                //   4. Mode does NOT change here — `transition()` is an identity for these.
                // ---------------------------------------------------------------------------
                Action::PermissionAcceptOnce => {
                    send_permission_decision(app, PermissionDecisionKind::Allow);
                }
                Action::PermissionAcceptAlways => {
                    send_permission_decision(app, PermissionDecisionKind::AcceptAlways);
                }
                Action::PermissionReject => {
                    send_permission_decision(app, PermissionDecisionKind::Deny);
                }
                // ---------------------------------------------------------------------------
                // BC-2.06.015 PC-1 — [t] trace-to-source stub
                //
                // Phase 1: sets App.status_message to the canonical placeholder text.
                // No AppMode transition (identity via transition()), no overlay_stack
                // mutation, no IPC send (PC-3). The per-context binding for AppModeTag::Overlay
                // ensures this arm is only reachable in Overlay mode (EC-099 holds).
                // ---------------------------------------------------------------------------
                Action::PermissionTraceToSource => {
                    app.status_message = Some(
                        "[t] Trace to source \u{2014} Phase 2 feature (Static plane)".to_string(),
                    );
                    // Identity transition: mode stays Overlay { prior }, overlay_stack unchanged.
                    app.mode = transition(app.mode.clone(), action);
                }
                // ---------------------------------------------------------------------------
                // S-031 — Profile Picker open (BC-2.07.005 PC-1 / AC-001 / AC-010).
                //
                // Action::ProfilePicker fires on Ctrl-P from ANY AppMode (Global layer binding).
                // Calls open_profile_picker (which uses std::env::current_dir() for CWD).
                // Does NOT call transition() — AppMode is UNCHANGED by picker open (BC-2.07.005 INV-1).
                // The picker pre-check above this block ensures this arm is NOT reached while the
                // picker is already open (idempotency handled by open_profile_picker_with_dir).
                // ---------------------------------------------------------------------------
                Action::ProfilePicker => {
                    open_profile_picker(app);
                    // AppMode is NOT transitioned — picker coexists over any AppMode.
                }
                _ => {
                    app.mode = transition(app.mode.clone(), action);
                }
            }

            if is_quit {
                KeyOutcome::Quit
            } else {
                KeyOutcome::Continue
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Frame render helper (extracted for testability — F-S025-ADV4 AC-007)
// ---------------------------------------------------------------------------

/// Render a single application frame into the given ratatui `Frame`.
///
/// Extracted from `run()` so that integration tests can assert on the buffer
/// contents using `TestBackend` without spinning up an async event loop
/// (F-S025-ADV4 coverage: AC-007 page-level status bar drop counter).
///
/// # Layout branches
///
/// - `AppMode::Fullscreen` — panel occupies the full main area; status bar below.
/// - All other modes (Dashboard, Overlay, Filtering) — 60/40 dashboard split;
///   Sessions panel left, status bar below.
///
/// # Drop counter (BC-2.06.019 PC-2 / BC-2.06.005 PC-3)
///
/// Drop-counter rendering is delegated to `render_status_bar`. When
/// `app.drop_counter > 0`, `render_status_bar` emits `"drops: N"` in yellow on
/// the UPPER (breadcrumb) row of the two-row status bar (BC-2.06.019 PC-2).
/// When the counter is zero, no drop text is emitted. The Sessions panel widget
/// does NOT duplicate the drop counter (F-S025-ADV2-MED-002).
pub fn render_frame(
    app: &mut App,
    sessions_state: &mut crate::ui::sessions_panel::SessionsPanelState,
    frame: &mut ratatui::Frame,
) {
    use crate::ui::event_ribbon::EventRibbon;
    use crate::ui::layout::{build_dashboard_layout, build_fullscreen_layout};
    use crate::ui::overlay_widget::{render_dimmed_background, render_overlay_widget};
    use crate::ui::profile_picker_widget::render_profile_picker;
    use crate::ui::sessions_panel::{render_sessions_filter, SessionsPanel};
    use monocle_core::tui::state::PanelId;
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span},
        widgets::{Paragraph, StatefulWidget, Widget},
    };

    use crate::ui::status_bar::render_status_bar;

    // Branch on app.mode for layout and panel rendering (BC-2.06.007 PC-7).
    // Clone mode to avoid borrow conflict when mutating app fields below.
    let mode_tag = match &app.mode {
        AppMode::Dashboard { .. } => 0u8,
        AppMode::Filtering { .. } => 1u8,
        AppMode::Overlay { .. } => 2u8,
        AppMode::Fullscreen { .. } => 3u8,
        AppMode::EmbeddedTerminal { .. } => 4u8,
        AppMode::SessionCreation { .. } => 5u8,
    };

    match mode_tag {
        3u8 => {
            // Fullscreen mode.
            let panel = match &app.mode {
                AppMode::Fullscreen { panel, .. } => panel.clone(),
                _ => unreachable!(),
            };
            let layout = build_fullscreen_layout(frame.area());
            match &panel {
                PanelId::Sessions => {
                    let p = SessionsPanel::new(app);
                    p.render(layout.panel_area, frame.buffer_mut(), sessions_state);
                }
                PanelId::EventRibbon => {
                    // S-028 (AC-010): EventRibbon fullscreen.
                    // Pre-compute and assign panel_height before borrowing app for widget.
                    let fs_panel_height = layout.panel_area.height as usize;
                    app.event_ribbon_panel_height = fs_panel_height;
                    let selected_sid: Option<String> = sessions_state
                        .list_state
                        .selected()
                        .and_then(|i| app.sessions.get(i))
                        .map(|s| s.session_id.clone());
                    // Render widget: split borrow — widget borrows app.event_ribbon_events
                    // immutably; state is separate from the immutable widget fields.
                    // Use a local state snapshot to avoid aliasing via &mut App.
                    let mut local_ribbon_state = {
                        use ratatui::widgets::ListState;
                        let mut s = ListState::default();
                        s.select(app.event_ribbon_state.list_state.selected());
                        crate::ui::event_ribbon::EventRibbonState {
                            list_state: s,
                            pinned_top: app.event_ribbon_state.pinned_top,
                        }
                    };
                    let sid_ref = selected_sid.as_deref();
                    let widget = EventRibbon::new(app, sid_ref);
                    StatefulWidget::render(
                        widget,
                        layout.panel_area,
                        frame.buffer_mut(),
                        &mut local_ribbon_state,
                    );
                    // Write back the updated state.
                    app.event_ribbon_state
                        .list_state
                        .select(local_ribbon_state.list_state.selected());
                    crate::ui::event_ribbon::trim_to_panel_height(
                        &mut app.event_ribbon_events,
                        fs_panel_height,
                    );
                }
                _ => {
                    // Future panels.
                    Widget::render(
                        Paragraph::new(Line::from(Span::styled(
                            "Panel (S-028+)",
                            Style::default().fg(Color::DarkGray),
                        ))),
                        layout.panel_area,
                        frame.buffer_mut(),
                    );
                }
            }
            // Status bar: always full-brightness, never dimmed (AC-008 / BC-2.06.019 PC-1).
            render_status_bar(
                &app.mode,
                app.drop_counter,
                app.overlay_stack.len(),
                app.status_message.as_deref(),
                app.ccr_path.as_deref(),
                layout.status_bar_area,
                frame.buffer_mut(),
            );

            // S-031 (AC-002 / BC-2.07.005 PC-2): profile picker modal overlay.
            // Rendered AFTER the status bar so it floats above all content.
            if let Some(picker_state) = &app.profile_picker {
                render_profile_picker(picker_state, &app.config, frame.area(), frame.buffer_mut());
            }
        }
        4u8 => {
            // AppMode::EmbeddedTerminal — render the vt100 parser state via tui_term widget.
            // F-S039-002 (BC-2.09.001 AC-003 / Postcondition 3).
            use crate::ui::embedded_terminal::render_embedded_terminal;
            use crate::ui::layout::build_dashboard_layout;

            let session_id = match &app.mode {
                AppMode::EmbeddedTerminal { session_id, .. } => session_id.clone(),
                _ => unreachable!(),
            };

            // Use the full frame area as the terminal pane (status bar reserved below).
            // build_dashboard_layout gives us a panel_area; we use sessions_area as the main pane.
            let layout = build_dashboard_layout(frame.area());
            let terminal_area = layout.sessions_area;

            if let Some(parser) = app.pty_parsers.get(&session_id) {
                render_embedded_terminal(frame, terminal_area, parser);
            } else {
                // Parser not yet created (race before InitialState wires it); render placeholder.
                use ratatui::widgets::Widget;
                use ratatui::{
                    style::{Color, Style},
                    text::{Line, Span},
                    widgets::Paragraph,
                };
                Widget::render(
                    Paragraph::new(Line::from(Span::styled(
                        "Connecting to PTY...",
                        Style::default().fg(Color::DarkGray),
                    ))),
                    terminal_area,
                    frame.buffer_mut(),
                );
            }

            // F-PASS4-MED-001: if a dump is in progress AND bytes were dropped due to
            // the cap, render "[dump: N drops]" in the status bar (overrides status_message
            // for this frame while the condition holds — transient, never persisted to
            // app.status_message).
            let dump_drop_status: Option<String> = {
                let in_progress = app
                    .dump_in_progress
                    .get(&session_id)
                    .copied()
                    .unwrap_or(false);
                let drops = app
                    .pending_pty_drop_count
                    .get(&session_id)
                    .copied()
                    .unwrap_or(0);
                if in_progress && drops > 0 {
                    Some(format!("[dump: {drops} drops]"))
                } else {
                    None
                }
            };
            let pty_status_msg = dump_drop_status
                .as_deref()
                .or(app.status_message.as_deref());

            // Always render the status bar in EmbeddedTerminal mode.
            render_status_bar(
                &app.mode,
                app.drop_counter,
                app.overlay_stack.len(),
                pty_status_msg,
                app.ccr_path.as_deref(),
                layout.status_bar_area,
                frame.buffer_mut(),
            );

            // Profile picker can appear over EmbeddedTerminal (BC-2.07.005 INV-4).
            if let Some(picker_state) = &app.profile_picker {
                render_profile_picker(picker_state, &app.config, frame.area(), frame.buffer_mut());
            }
        }
        _ => {
            // Dashboard, Overlay, Filtering: all use dashboard 60/40 split.
            let layout = build_dashboard_layout(frame.area());

            // S-028 (AC-010 / BC-2.06.006 PC-1): In Filtering mode, render the filter
            // input box + scored session list instead of the regular SessionsPanel.
            // In all other modes (Dashboard, Overlay), render SessionsPanel normally.
            // F-W7G3-MED-001: In Filtering mode, render_sessions_filter returns the
            // session_id of the highlighted row in the SCORED (filtered/reordered) list.
            // In non-filtering modes, derive selected_sid from sessions_state.list_state,
            // which indexes into app.sessions (insertion order == display order there).
            let filter_selected_sid: Option<Option<String>>;

            if let AppMode::Filtering {
                panel: PanelId::Sessions,
                ref query,
                ..
            } = app.mode.clone()
            {
                // BC-2.06.006 PC-1/PC-2/PC-8: filter input box + scored list.
                // Returns Some(session_id) of the highlighted row in the scored list,
                // or None when no row is highlighted or zero sessions match the query.
                let sid = render_sessions_filter(
                    app,
                    query.as_str(),
                    layout.sessions_area,
                    frame.buffer_mut(),
                    sessions_state,
                );
                filter_selected_sid = Some(sid);
            } else {
                // Dashboard / Overlay: regular sessions panel.
                let panel = SessionsPanel::new(app);
                panel.render(layout.sessions_area, frame.buffer_mut(), sessions_state);
                filter_selected_sid = None;
            }

            // S-028 (AC-010 / BC-2.06.018 PC-1): Render Event Ribbon in the right 40% area.
            // Derive the selected session_id from the Sessions panel state.
            //
            // F-W7G3-MED-001: When in Filtering mode, use the session_id returned directly
            // by render_sessions_filter (which knows the scored ordering). In other modes,
            // sessions_state.list_state.selected() safely indexes into app.sessions because
            // the list is in insertion order.
            let selected_sid: Option<String> = match filter_selected_sid {
                Some(sid) => sid,
                None => sessions_state
                    .list_state
                    .selected()
                    .and_then(|i| app.sessions.get(i))
                    .map(|s| s.session_id.clone()),
            };
            let ribbon_area = layout.event_ribbon_area;
            let panel_height = ribbon_area.height as usize;

            // BC-2.06.018 PC-3: update the cached panel_height for push-time cap before render.
            app.event_ribbon_panel_height = panel_height;

            // Split borrow: EventRibbon borrows app.event_ribbon_events immutably (through &App).
            // We need &mut app.event_ribbon_state for StatefulWidget::render. To avoid aliasing,
            // extract a local copy of the state, render into it, then write back.
            let mut local_ribbon_state = {
                use ratatui::widgets::ListState;
                let mut s = ListState::default();
                s.select(app.event_ribbon_state.list_state.selected());
                crate::ui::event_ribbon::EventRibbonState {
                    list_state: s,
                    pinned_top: app.event_ribbon_state.pinned_top,
                }
            };
            let sid_ref = selected_sid.as_deref();
            let widget = EventRibbon::new(app, sid_ref);
            StatefulWidget::render(
                widget,
                ribbon_area,
                frame.buffer_mut(),
                &mut local_ribbon_state,
            );
            // Write back updated list scroll state (ratatui may adjust selected() during render).
            app.event_ribbon_state
                .list_state
                .select(local_ribbon_state.list_state.selected());

            // BC-2.06.018 INV-3: trim on resize — remove events beyond new panel_height.
            crate::ui::event_ribbon::trim_to_panel_height(
                &mut app.event_ribbon_events,
                panel_height,
            );

            // S-027 (AC-002 / BC-2.06.010 PC-2): Apply DIM when overlay is active.
            if matches!(&app.mode, AppMode::Overlay { .. }) {
                let full_area = frame.area();
                let background_area = ratatui::layout::Rect {
                    x: full_area.x,
                    y: full_area.y,
                    width: full_area.width,
                    height: full_area
                        .height
                        .saturating_sub(layout.status_bar_area.height),
                };
                render_dimmed_background(background_area, frame.buffer_mut());

                if let Some(modal) = app.overlay_stack.front() {
                    let stack_depth = app.overlay_stack.len();
                    render_overlay_widget(modal, stack_depth, background_area, frame);
                }
            }

            // S-027 (AC-012 / BC-2.06.019/020/021): Render the always-visible two-row status bar.
            render_status_bar(
                &app.mode,
                app.drop_counter,
                app.overlay_stack.len(),
                app.status_message.as_deref(),
                app.ccr_path.as_deref(),
                layout.status_bar_area,
                frame.buffer_mut(),
            );

            // S-031 (AC-002 / BC-2.07.005 PC-2): profile picker modal overlay.
            // Rendered AFTER the status bar so it floats above all content (including Overlay).
            // The picker can appear over any AppMode — including Overlay mode with a
            // permission prompt (BC-2.07.004 INV-1 / BC-2.07.005 INV-4).
            if let Some(picker_state) = &app.profile_picker {
                render_profile_picker(picker_state, &app.config, frame.area(), frame.buffer_mut());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Key conversion helpers (BLOCKER-002: full binding dispatch)
// ---------------------------------------------------------------------------

/// Build the builtin `BindingLayers` for Phase 1.
///
/// Registers the minimum set of bindings required for AC-006:
/// - `q` (Dashboard only, per-context) → `Action::Quit` — exits only from Dashboard.
///   Typing `q` in Filtering mode inserts the character (SearchPrompt layer intercepts
///   it first). This fixes F-S025-ADV2-HIGH-002 / MED-004: `q` must not quit from
///   non-Dashboard modes where it is a valid input character.
/// - Esc (builtin) → `Action::Esc` — context-sensitive: Dashboard=identity, Overlay=no-op
///   (explicit arm in transition()), Fullscreen=identity (only `Action::ExitFullscreen` exits,
///   wired by the fullscreen-view story), Filtering=identity (`Action::CancelFilter` cancels
///   filtering — not Esc). Not used as a quit path.
/// - Tab → `Action::MoveFocus` (cycle Sessions ↔ EventRibbon)
/// - Enter → `Action::EnterFullscreen { Sessions }` (expand current panel)
/// - j / ↓ → `Action::SelectNext` (builtin, non-Dashboard modes only); overridden by
///   per-context `(j, Dashboard)` → `Action::ScrollDown` for all Dashboard focuses.
///   The `ScrollDown` dispatch arm handles Sessions-cursor vs ribbon-scroll (BC-2.06.018 AC-010 §2).
///   `SelectNext` is therefore unreachable in Dashboard mode (ADV Pass-6 NITPICK-1).
/// - k / ↑ → `Action::SelectPrev` (builtin, non-Dashboard modes only); similarly overridden by
///   `(k, Dashboard)` → `Action::ScrollUp`. `SelectPrev` unreachable in Dashboard mode.
///
/// Future waves add user-custom and per-context layers; for now only builtin,
/// global, and per-context layers are populated.
pub fn build_builtin_binding_layers() -> monocle_core::tui::binding::BindingLayers {
    use monocle_core::tui::binding::{AppModeTag, BindingLayers, KeyCode, KeyEvent, KeyModifiers};
    use monocle_core::tui::state::{Action, PanelId};

    let no_mod = KeyModifiers::default();

    let mut layers = BindingLayers::empty();

    // Global bindings (active in ALL modes — BC-2.07.005 INV-1).
    // Tab → MoveFocus
    layers.global.insert(
        KeyEvent {
            code: KeyCode::Tab,
            modifiers: no_mod,
        },
        Action::MoveFocus,
    );

    // Ctrl-P → Action::ProfilePicker (BC-2.07.005 INV-1: fires in ALL AppModes — no guard).
    // Registered in the Global layer so that resolve_binding returns it regardless of AppMode.
    layers.global.insert(
        KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers {
                ctrl: true,
                shift: false,
                alt: false,
            },
        },
        Action::ProfilePicker,
    );

    // Builtin bindings (lowest precedence; hard-coded fallbacks).
    // Esc → Esc (handled by transition; context-sensitive but NOT the quit path)
    layers.builtin.insert(
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: no_mod,
        },
        Action::Esc,
    );
    // Enter → EnterFullscreen { Sessions }
    layers.builtin.insert(
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: no_mod,
        },
        Action::EnterFullscreen {
            panel: PanelId::Sessions,
        },
    );
    // j → SelectNext
    layers.builtin.insert(
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: no_mod,
        },
        Action::SelectNext,
    );
    // ↓ → SelectNext
    layers.builtin.insert(
        KeyEvent {
            code: KeyCode::Down,
            modifiers: no_mod,
        },
        Action::SelectNext,
    );
    // k → SelectPrev
    layers.builtin.insert(
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: no_mod,
        },
        Action::SelectPrev,
    );
    // ↑ → SelectPrev
    layers.builtin.insert(
        KeyEvent {
            code: KeyCode::Up,
            modifiers: no_mod,
        },
        Action::SelectPrev,
    );

    // Per-context bindings (mode-scoped, higher precedence than global/builtin).
    //
    // F-S025-ADV2-HIGH-002: `q` → Action::Quit ONLY in Dashboard mode.
    // In Filtering mode, `q` is intercepted by the SearchPrompt layer as
    // Action::FilterType('q') before this layer is consulted. In Overlay mode,
    // `q` is not a permission decision key and would fall through to no binding —
    // correct behaviour (Overlay decisions are y/A/n/r only).
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: no_mod,
            },
            AppModeTag::Dashboard,
        ),
        Action::Quit,
    );

    // BC-2.06.015 INV-1: `t` → PermissionTraceToSource ONLY in Overlay mode.
    //
    // Registered as a per-context binding scoped to AppModeTag::Overlay so that
    // EC-099 holds naturally: `t` in Dashboard resolves to None (no binding).
    // INV-1 states this is a Builtin (non-user-overridable) binding; per-context
    // at AppModeTag::Overlay is the correct mechanism — it is hardcoded in this
    // function (not in any user customisation file), so it cannot be overridden.
    //
    // NOTE: The permission decision keys (y/Enter/A/n/r) are captured at the SearchPrompt
    // layer (Level 1) inside the Overlay arm of resolve_binding. `t` is NOT among those keys
    // (the Overlay SearchPrompt arm matches only y/A/n/r/Up/Down, returning None for `t`).
    // `t` therefore falls through to the PerContext layer (Level 3) where it is registered
    // here as Action::PermissionTraceToSource for AppModeTag::Overlay.
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Char('t'),
                modifiers: no_mod,
            },
            AppModeTag::Overlay,
        ),
        Action::PermissionTraceToSource,
    );

    // S-028 (BC-2.06.006 PC-1 / AC-001): `/` → StartFilter { Sessions } in Dashboard.
    // `f` is also an alias for filter entry (AC-001 'f' binding).
    // Both are registered as per-context bindings for Dashboard mode so they only fire
    // in Dashboard and not in other modes (e.g., typing '/' in Filtering appends to query
    // via the SearchPrompt FilterType capture, not this binding).
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Char('/'),
                modifiers: no_mod,
            },
            AppModeTag::Dashboard,
        ),
        Action::StartFilter {
            panel: PanelId::Sessions,
        },
    );
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: no_mod,
            },
            AppModeTag::Dashboard,
        ),
        Action::StartFilter {
            panel: PanelId::Sessions,
        },
    );

    // S-028 (BC-2.06.006 PC-2 exit / AC-003): Enter → CommitFilter in Filtering mode.
    // Esc → CancelFilter in Filtering mode.
    //
    // Registered as per-context bindings for AppModeTag::Filtering:
    // - In Filtering mode the SearchPrompt layer intercepts non-printable keys via the
    //   search_prompt table (priority 1), then falls through. Since Enter/Esc must ONLY
    //   map to CommitFilter/CancelFilter in Filtering (not in Dashboard where Enter →
    //   EnterFullscreen), per-context (priority 3) is the correct layer.
    // - The SearchPrompt layer for Filtering only captures printable chars as FilterType
    //   and then looks up the search_prompt table for non-printables. Since Enter/Esc are
    //   not in search_prompt, they fall through to per-context — where these bindings fire.
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: no_mod,
            },
            AppModeTag::Filtering,
        ),
        Action::CommitFilter,
    );
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: no_mod,
            },
            AppModeTag::Filtering,
        ),
        Action::CancelFilter,
    );

    // BC-2.06.018 AC-010 §2 (F-1 FIX): j/↓ → Action::ScrollDown and k/↑ → Action::ScrollUp
    // as per-context bindings for AppModeTag::Dashboard.
    //
    // These override the builtin SelectNext/SelectPrev in Dashboard mode. The dispatch
    // arms for ScrollDown/ScrollUp check the focus (Sessions vs EventRibbon) and route
    // accordingly — Sessions focus does cursor movement, EventRibbon focus scrolls the ribbon.
    //
    // Because AppModeTag cannot discriminate focus sub-state (Sessions vs EventRibbon),
    // the focus discrimination lives in dispatch_key_event's ScrollDown/ScrollUp arms,
    // not in the binding layer. resolve_binding returns ScrollDown/ScrollUp for ALL
    // Dashboard focus sub-states; dispatch handles the split.
    //
    // The SelectNext/SelectPrev builtin bindings remain for non-Dashboard contexts
    // (Fullscreen, etc.) where the session cursor semantics still apply.
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: no_mod,
            },
            AppModeTag::Dashboard,
        ),
        Action::ScrollDown,
    );
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Down,
                modifiers: no_mod,
            },
            AppModeTag::Dashboard,
        ),
        Action::ScrollDown,
    );
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: no_mod,
            },
            AppModeTag::Dashboard,
        ),
        Action::ScrollUp,
    );
    layers.per_context.insert(
        (
            KeyEvent {
                code: KeyCode::Up,
                modifiers: no_mod,
            },
            AppModeTag::Dashboard,
        ),
        Action::ScrollUp,
    );

    layers
}

/// Convert a `crossterm::event::KeyEvent` to a `monocle_core::tui::binding::KeyEvent`.
///
/// Translates crossterm-specific key codes and modifiers into the pure-core
/// key event type used by `resolve_binding`. Called in the event loop before
/// dispatching to `resolve_binding`.
pub fn crossterm_key_to_core(
    ct: &crossterm::event::KeyEvent,
) -> monocle_core::tui::binding::KeyEvent {
    use crossterm::event::{KeyCode as CtCode, KeyModifiers as CtMod};
    use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};

    let code = match ct.code {
        CtCode::Char(c) => KeyCode::Char(c),
        CtCode::Enter => KeyCode::Enter,
        CtCode::Esc => KeyCode::Esc,
        CtCode::Up => KeyCode::Up,
        CtCode::Down => KeyCode::Down,
        CtCode::Left => KeyCode::Left,
        CtCode::Right => KeyCode::Right,
        CtCode::Tab => KeyCode::Tab,
        CtCode::Backspace => KeyCode::Backspace,
        // Any other crossterm key code maps to Unknown — the canonical sentinel
        // for unmapped keys (F-S025-ADV2-LOW-001). The binding resolver never
        // registers Unknown, so these keys silently produce no action.
        _ => KeyCode::Unknown,
    };

    let modifiers = KeyModifiers {
        shift: ct.modifiers.contains(CtMod::SHIFT),
        ctrl: ct.modifiers.contains(CtMod::CONTROL),
        alt: ct.modifiers.contains(CtMod::ALT),
    };

    KeyEvent { code, modifiers }
}
