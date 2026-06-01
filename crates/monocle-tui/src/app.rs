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
use monocle_config::{load_config, MonocleConfig};
use monocle_core::engine::EnrichedSession;
use monocle_core::tui::state::{AppMode, FocusSnapshot, PromptModal, ToolPayload};
use monocle_ipc::error::IpcError;
use monocle_ipc::framing::read_framed;
use monocle_ipc::reconnect::{BackoffState, RECONNECT_WINDOW_SECS};
use monocle_ipc::types::{
    ClientToServer, HookEventRecord, HookType, PermissionPromptPayload, ServerToClient,
};
// PermissionDecisionKind: re-exported from lib.rs for integration tests and for S-026
// decision handler implementations. The re-export here ensures the type is visible at
// the app module level when todo!() stubs are replaced with real code.
pub use monocle_ipc::types::PermissionDecisionKind;
use std::collections::VecDeque;
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
    /// 2. `dispatch_key_event` can call `reset_on_session_change` in `SelectNext`/`SelectPrev`
    ///    arms when the selected session changes (BC-2.06.018 INV-1 / AC-009).
    pub event_ribbon_state: crate::ui::event_ribbon::EventRibbonState,

    /// Last rendered event ribbon panel height (rows), captured in `render_frame`.
    ///
    /// Used as the dynamic cap for `push_event_row` in `on_hook_event_received`
    /// (BC-2.06.018 PC-3: VecDeque bounded to `panel_height`). Initialises to
    /// `EVENT_RING_CAPACITY` so the first push before any render sees a safe cap.
    /// Updated each frame in `render_frame` when the EventRibbon widget is rendered.
    pub event_ribbon_panel_height: usize,

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
}

impl App {
    /// Construct a default `App` from the provided config.
    ///
    /// Starts in `Dashboard { focused: Sessions }` with empty collections.
    /// The `nucleo::Matcher` is initialized once here (BC-2.06.006 AC-005 / INV-1:
    /// shared Matcher — NOT recreated per keystroke).
    pub fn new(config: MonocleConfig) -> Self {
        Self {
            mode: AppMode::Dashboard {
                focused: FocusSnapshot::Sessions,
            },
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
            ipc_tx: None,
            // BC-2.06.018 PC-5 / AC-007: no pending key prefix initially.
            // Set to Some('g') on first 'g' press in Dashboard { EventRibbon } focus;
            // cleared on second 'g' (fires gg jump) or any other key.
            pending_key: None,
        }
    }
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
/// # Mapping rules (BC-2.06.024 / AC-016 v1.10)
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
    app.sessions = sessions;
    app.drop_counter = drop_counter;

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
            app.mode = AppMode::Dashboard {
                focused: FocusSnapshot::Sessions,
            };
            app.status_message = Some(DAEMON_DISCONNECT_STATUS.to_string());
            tracing::warn!("IPC transport disconnected; entering reconnect state");
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
    // BC-2.05.004 PC-2 / SS-ipc v1.10.0: use the daemon's timestamp_micros (not TUI receive time).
    let row = crate::ui::event_ribbon::hook_event_row_from_received(
        hook_type,
        session_id,
        latency_ms,
        timestamp_micros,
    );
    let cap = app.event_ribbon_panel_height;
    crate::ui::event_ribbon::push_event_row(&mut app.event_ribbon_events, row, cap);

    // BC-2.06.018 PC-8 / AC-008: auto-scroll to row 0 (newest) when not pinned_top.
    // The ribbon tracks all sessions; auto-scroll unconditionally resets to front when
    // !pinned_top (any new event is potentially relevant to the selected session).
    if !app.event_ribbon_state.pinned_top {
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

/// Capacity of the outbound `ClientToServer` command channel.
///
/// Lower than the inbound reader channel (64) because `ClientToServer` messages are
/// rare user-driven keypresses, not high-frequency daemon events.  N=32 provides
/// headroom for burst scenarios while keeping bounded-channel semantics
/// (SS-conventions-anti-patterns.md §forbidden-patterns: no unbounded channels).
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
fn setup_ipc_streams_with_rx<R, W>(
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
    // N=64 — same as original; see spawn_ipc_reader doc comment for capacity rationale.
    let (inbound_tx, inbound_rx) =
        tokio::sync::mpsc::channel::<Result<ServerToClient, IpcError>>(64);
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
fn handle_server_message(app: &mut App, msg: ServerToClient) -> Result<()> {
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
            // BC-2.05.004 PC-2 / SS-ipc v1.10.0: pass daemon's timestamp_micros through.
            on_hook_event_received(
                app,
                hook_type,
                session_id,
                payload_excerpt,
                latency_ms,
                timestamp_micros,
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
/// 2. For `SelectNext` / `SelectPrev`: applies the AC-006 gate (only fires in
///    `Dashboard { focused: Sessions }`); mutates `sessions_state` on pass.
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
    use monocle_core::tui::binding::resolve_binding;
    use monocle_core::tui::binding::KeyCode;
    use monocle_core::tui::state::{transition, Action};

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
            // AC-006: SelectNext is confined to Dashboard mode.
            // - Dashboard { focused: Sessions } → cursor move in session list.
            // - Dashboard { focused: EventRibbon } → scroll ribbon down (toward older events,
            //   BC-2.06.018 PC-5 / AC-007). This dual behaviour is intentional: the binding
            //   layer cannot distinguish panel focus, so dispatch does the disambiguation here.
            match &app.mode {
                AppMode::Dashboard {
                    focused: FocusSnapshot::Sessions,
                } => {
                    let len = app.sessions.len();
                    if len > 0 {
                        let prev_idx = sessions_state.list_state.selected();
                        let next = prev_idx.map(|i| (i + 1).min(len - 1)).unwrap_or(0);
                        sessions_state.list_state.select(Some(next));
                        // BC-2.06.018 INV-1 / AC-009: on session change, reset ribbon scroll.
                        // Only reset when the cursor actually moved to a different session.
                        if prev_idx != Some(next) {
                            let new_sid = app.sessions.get(next).map(|s| s.session_id.clone());
                            crate::ui::event_ribbon::reset_on_session_change(
                                &mut app.event_ribbon_state,
                                new_sid.as_deref().unwrap_or(""),
                            );
                        }
                    }
                }
                AppMode::Dashboard {
                    focused: FocusSnapshot::EventRibbon,
                } => {
                    // BC-2.06.018 PC-5 / AC-007: scroll ribbon one row toward older events.
                    crate::ui::event_ribbon::scroll_ribbon_down(
                        &mut app.event_ribbon_state,
                        &app.event_ribbon_events,
                    );
                }
                _ => {} // Other modes: no-op (Overlay, Fullscreen, etc.).
            }
            KeyOutcome::Continue
        }

        Some((Action::SelectPrev, _)) => {
            // AC-006: SelectPrev is confined to Dashboard mode.
            // - Dashboard { focused: Sessions } → cursor move in session list (up).
            // - Dashboard { focused: EventRibbon } → scroll ribbon up (toward newer events,
            //   BC-2.06.018 PC-5 / AC-007).
            match &app.mode {
                AppMode::Dashboard {
                    focused: FocusSnapshot::Sessions,
                } if !app.sessions.is_empty() => {
                    let prev_idx = sessions_state.list_state.selected();
                    let prev = prev_idx.map(|i| i.saturating_sub(1)).unwrap_or(0);
                    sessions_state.list_state.select(Some(prev));
                    // BC-2.06.018 INV-1 / AC-009: on session change, reset ribbon scroll.
                    if prev_idx != Some(prev) {
                        let new_sid = app.sessions.get(prev).map(|s| s.session_id.clone());
                        crate::ui::event_ribbon::reset_on_session_change(
                            &mut app.event_ribbon_state,
                            new_sid.as_deref().unwrap_or(""),
                        );
                    }
                }
                AppMode::Dashboard {
                    focused: FocusSnapshot::EventRibbon,
                } => {
                    // BC-2.06.018 PC-5 / AC-007: scroll ribbon one row toward newer events.
                    crate::ui::event_ribbon::scroll_ribbon_up(
                        &mut app.event_ribbon_state,
                        &app.event_ribbon_events,
                    );
                }
                _ => {} // Other modes or empty sessions: no-op.
            }
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
/// # Drop counter (AC-007, BC-2.06.005 PC-3)
///
/// When `app.drop_counter > 0`, the page-level status bar renders
/// `"[dropped: N] monocle"` in yellow. When `app.drop_counter == 0`, it renders
/// `"monocle"` in dark-gray. This is the ONLY location where the drop counter
/// text is rendered — the Sessions panel widget itself does NOT duplicate it
/// (F-S025-ADV2-MED-002).
pub fn render_frame(
    app: &mut App,
    sessions_state: &mut crate::ui::sessions_panel::SessionsPanelState,
    frame: &mut ratatui::Frame,
) {
    use crate::ui::event_ribbon::EventRibbon;
    use crate::ui::layout::{build_dashboard_layout, build_fullscreen_layout};
    use crate::ui::overlay_widget::{render_dimmed_background, render_overlay_widget};
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
                layout.status_bar_area,
                frame.buffer_mut(),
            );
        }
        _ => {
            // Dashboard, Overlay, Filtering: all use dashboard 60/40 split.
            let layout = build_dashboard_layout(frame.area());

            // S-028 (AC-010 / BC-2.06.006 PC-1): In Filtering mode, render the filter
            // input box + scored session list instead of the regular SessionsPanel.
            // In all other modes (Dashboard, Overlay), render SessionsPanel normally.
            if let AppMode::Filtering {
                panel: PanelId::Sessions,
                ref query,
                ..
            } = app.mode.clone()
            {
                // BC-2.06.006 PC-1/PC-2/PC-8: filter input box + scored list.
                render_sessions_filter(
                    app,
                    query.as_str(),
                    layout.sessions_area,
                    frame.buffer_mut(),
                    sessions_state,
                );
            } else {
                // Dashboard / Overlay: regular sessions panel.
                let panel = SessionsPanel::new(app);
                panel.render(layout.sessions_area, frame.buffer_mut(), sessions_state);
            }

            // S-028 (AC-010 / BC-2.06.018 PC-1): Render Event Ribbon in the right 40% area.
            // Derive the selected session_id from the Sessions panel state.
            let selected_sid: Option<String> = sessions_state
                .list_state
                .selected()
                .and_then(|i| app.sessions.get(i))
                .map(|s| s.session_id.clone());
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
                layout.status_bar_area,
                frame.buffer_mut(),
            );
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
/// - j / ↓ → `Action::SelectNext` (move selection down)
/// - k / ↑ → `Action::SelectPrev` (move selection up)
///
/// Future waves add user-custom and per-context layers; for now only builtin,
/// global, and per-context layers are populated.
pub fn build_builtin_binding_layers() -> monocle_core::tui::binding::BindingLayers {
    use monocle_core::tui::binding::{AppModeTag, BindingLayers, KeyCode, KeyEvent, KeyModifiers};
    use monocle_core::tui::state::{Action, PanelId};

    let no_mod = KeyModifiers::default();

    let mut layers = BindingLayers::empty();

    // Global bindings (active in all modes).
    // Tab → MoveFocus
    layers.global.insert(
        KeyEvent {
            code: KeyCode::Tab,
            modifiers: no_mod,
        },
        Action::MoveFocus,
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
