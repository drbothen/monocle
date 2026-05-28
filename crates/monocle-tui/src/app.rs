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
use monocle_ipc::types::{HookEventRecord, PermissionPromptPayload, ServerToClient};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;
use tokio::io::AsyncReadExt;

// ---------------------------------------------------------------------------
// Stub types
// ---------------------------------------------------------------------------

// MERGE-COORDINATION (S-023 → S-025):
// `TransportEvent` is defined here as a local stub because S-023 introduces
// the canonical type in `monocle-ipc::events::TransportEvent`. When S-023 merges
// to develop, S-025's merge-time conflict resolution MUST:
//   1. Delete this local enum.
//   2. Replace `use crate::app::TransportEvent` with `use monocle_ipc::events::TransportEvent`.
//   3. Verify variant shape matches BC-2.05.007 (single `Disconnected` variant currently).
// Surfaced by F-S025-ADV2-MED-003.

/// Signal that the IPC transport has changed connection state.
///
/// Local stub — see MERGE-COORDINATION block above. Aligns with the canonical
/// `monocle-ipc::events::TransportEvent` type being defined in S-023.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TransportEvent {
    /// The UDS connection was lost (daemon exited or socket closed).
    Disconnected,
}

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
    /// Loaded monocle configuration (MonocleConfig::load on startup).
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
}

impl App {
    /// Construct a default `App` from the provided config.
    ///
    /// Starts in `Dashboard { focused: Sessions }` with empty collections.
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
/// # Mapping rules
///
/// - `tool_name == "Bash"` → `ToolPayload::Bash { command: tool_input["command"] }`
/// - `tool_name == "Edit"` or `"Write"` → `ToolPayload::Edit { old_content, new_content, path }`
/// - `tool_name == "Read"` → `ToolPayload::Read { path }`
/// - Anything else → `ToolPayload::Generic { tool_name, tool_input }`
pub fn payload_to_modal(payload: PermissionPromptPayload) -> PromptModal {
    let tool_payload = match payload.tool_name.as_str() {
        "Bash" => ToolPayload::Bash {
            command: payload
                .tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        },
        "Edit" | "Write" => ToolPayload::Edit {
            old_content: payload.old_content.unwrap_or_default(),
            new_content: payload.new_content.unwrap_or_default(),
            path: payload
                .tool_input
                .get("path")
                .and_then(|v| v.as_str())
                .map(std::path::PathBuf::from)
                .unwrap_or_default(),
        },
        "Read" => ToolPayload::Read {
            path: payload
                .tool_input
                .get("path")
                .and_then(|v| v.as_str())
                .map(std::path::PathBuf::from)
                .unwrap_or_default(),
        },
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
    for record in ring_tail {
        if app.event_ring.len() == EVENT_RING_CAPACITY {
            app.event_ring.pop_front(); // FIFO eviction; does NOT increment drop_counter
        }
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
            app.status_message = Some("[disconnected] reconnecting...".to_string());
            tracing::warn!("IPC transport disconnected; entering reconnect state");
        }
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
// Main async run loop
// ---------------------------------------------------------------------------

/// Run the TUI event loop.
///
/// Called by `main()` after terminal setup. Connects to the monocle daemon
/// UDS, loads config, and drives the render+event loop until exit.
///
/// # Exit paths
///
/// - `q` or `Esc` from `Dashboard` mode → clean exit (status 0).
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
                let error_msg = "Daemon not running. Start it with: monocle daemon start";
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

    // Transfer transport ownership to the reader task (Option B — F-S025-ADV2-BLOCKER-001).
    //
    // The reader task loops forever, calling read_framed to completion and forwarding
    // completed frames (or disconnect errors) into the bounded mpsc channel.
    // The event loop drains ipc_rx with try_recv() — never calling read_framed directly.
    //
    // Channel capacity N=64: at 1000 events/sec (SS-conventions channel convention) and
    // 16ms render cadence, the loop drains ~16 events/tick — well within budget. N=64
    // provides 4× headroom against burst (64 × ~1KB ≈ 64KB max enqueued).
    //
    // Drop policy: BLOCK (tx.send().await). Silent drop on full would violate at-least-once
    // delivery for PermissionPromptQueued (BC-2.05.002 Invariant 4).
    //
    // Sender ownership (F-S025-ADV3-MED-003): `ipc_tx` is passed by move (not clone) to
    // `spawn_ipc_reader`. The reader task holds the ONLY sender; when it exits (disconnect or
    // error), the channel closes and `ipc_rx.try_recv()` returns `TryRecvError::Disconnected`.
    // This makes the disconnect arm in the drain loop reachable on natural reader exit.
    //
    // Reconnect channel (F-S025-ADV3-MED-003): on reconnect (S-023 merge), the channel is
    // re-created with a fresh `(ipc_tx2, ipc_rx2)` pair and `ipc_rx` is shadowed. This is
    // simpler than retaining a long-lived sender clone — there is no performance cost to
    // channel re-creation (it allocates a small fixed-size ring).
    let (ipc_tx, mut ipc_rx) = tokio::sync::mpsc::channel::<Result<ServerToClient, IpcError>>(64);
    let reader_handle = spawn_ipc_reader(transport, ipc_tx); // ipc_tx MOVED here — no clone retained

    // Set up the ratatui terminal.
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Render state for the Sessions panel (selection tracking).
    let mut sessions_state = crate::ui::sessions_panel::SessionsPanelState::default();

    // Build the builtin binding layers once for the session (AC-006, BLOCKER-002).
    // Future: merge user-custom and per-context layers from config.
    let binding_layers = build_builtin_binding_layers();

    // Main event loop (~60fps render cadence, keyboard polling, IPC drain).
    let tick_rate = Duration::from_millis(16); // ~60fps; also the keyboard poll ceiling

    loop {
        // 1. Render the current frame (AC-001, AC-005, BLOCKER-004, BC-2.06.007 PC-7).
        terminal.draw(|frame| {
            use crate::ui::layout::{build_dashboard_layout, build_fullscreen_layout};
            use crate::ui::sessions_panel::SessionsPanel;
            use monocle_core::tui::state::PanelId;
            use ratatui::{
                style::{Color, Style},
                text::{Line, Span},
                widgets::{Paragraph, StatefulWidget, Widget},
            };

            // Build the status line (shared between Dashboard and Fullscreen).
            let status_line = if app.drop_counter > 0 {
                Line::from(vec![Span::styled(
                    format!("[dropped: {}] monocle", app.drop_counter),
                    Style::default().fg(Color::Yellow),
                )])
            } else {
                Line::from(Span::styled(
                    "monocle",
                    Style::default().fg(Color::DarkGray),
                ))
            };

            // Branch on app.mode for layout and panel rendering (BC-2.06.007 PC-7).
            // Fullscreen mode: panel occupies full main area; Dashboard: 60/40 split.
            match &app.mode {
                AppMode::Fullscreen { panel, .. } => {
                    let layout = build_fullscreen_layout(frame.area());
                    match panel {
                        PanelId::Sessions => {
                            let p = SessionsPanel::new(&app);
                            p.render(layout.panel_area, frame.buffer_mut(), &mut sessions_state);
                        }
                        _ => {
                            // Future panels (EventRibbon fullscreen — S-027, others).
                            // PanelId is #[non_exhaustive]; render a placeholder until
                            // each panel's fullscreen renderer is implemented.
                            Widget::render(
                                Paragraph::new(Line::from(Span::styled(
                                    "Panel (S-027+)",
                                    Style::default().fg(Color::DarkGray),
                                ))),
                                layout.panel_area,
                                frame.buffer_mut(),
                            );
                        }
                    }
                    Widget::render(
                        Paragraph::new(status_line),
                        layout.status_bar_area,
                        frame.buffer_mut(),
                    );
                }
                _ => {
                    // Dashboard, Overlay, Filtering: all use dashboard 60/40 split.
                    // Overlay is rendered on top by S-026. Sessions panel always visible.
                    let layout = build_dashboard_layout(frame.area());

                    // Render the Sessions panel (left 60%).
                    let panel = SessionsPanel::new(&app);
                    panel.render(
                        layout.sessions_area,
                        frame.buffer_mut(),
                        &mut sessions_state,
                    );

                    // Render the status bar (bottom 2 rows): drop counter + breadcrumb.
                    Widget::render(
                        Paragraph::new(status_line),
                        layout.status_bar_area,
                        frame.buffer_mut(),
                    );
                }
            }
        })?;

        // 2. Poll keyboard (non-blocking, bounded by tick_rate — BLOCKER-002: full binding
        //    dispatch via resolve_binding). The 16ms ceiling is unchanged from the original
        //    implementation; the 1ms was only in the removed timeout wrapper.
        if event::poll(tick_rate)? {
            if let Event::Key(ct_key) = event::read()? {
                // Convert crossterm KeyEvent → monocle-core KeyEvent (pure-core type).
                let core_key = crossterm_key_to_core(&ct_key);

                // Resolve the binding through the 5-level precedence chain.
                let resolved = monocle_core::tui::binding::resolve_binding(
                    &core_key,
                    &app.mode,
                    &binding_layers,
                );

                match resolved {
                    Some((monocle_core::tui::state::Action::Noop, _)) | None => {
                        // No binding or explicit no-op — do nothing.
                    }
                    Some((monocle_core::tui::state::Action::SelectNext, _)) => {
                        // AC-006: SelectNext is confined to Dashboard { focused: Sessions }.
                        // In Overlay or Fullscreen mode, the keypress is dropped — no
                        // cursor mutation behind the overlay or in other modes.
                        if matches!(
                            app.mode,
                            AppMode::Dashboard {
                                focused: FocusSnapshot::Sessions
                            }
                        ) {
                            let len = app.sessions.len();
                            if len > 0 {
                                let next = sessions_state
                                    .list_state
                                    .selected()
                                    .map(|i| (i + 1).min(len - 1))
                                    .unwrap_or(0);
                                sessions_state.list_state.select(Some(next));
                            }
                        }
                    }
                    Some((monocle_core::tui::state::Action::SelectPrev, _)) => {
                        // AC-006: SelectPrev is confined to Dashboard { focused: Sessions }.
                        // In Overlay or Fullscreen mode, the keypress is dropped — no
                        // cursor mutation behind the overlay or in other modes.
                        if matches!(
                            app.mode,
                            AppMode::Dashboard {
                                focused: FocusSnapshot::Sessions
                            }
                        ) {
                            if !app.sessions.is_empty() {
                                let prev = sessions_state
                                    .list_state
                                    .selected()
                                    .map(|i| i.saturating_sub(1))
                                    .unwrap_or(0);
                                sessions_state.list_state.select(Some(prev));
                            }
                        }
                    }
                    Some((action, _)) => {
                        // All other actions: drive the AppMode state machine.
                        use monocle_core::tui::state::{transition, Action};
                        // Check for clean exit: Action::Quit (bound to `q` in Dashboard
                        // via per-context layer — F-S025-ADV2-HIGH-002 / MED-004 fix).
                        // `q` in Filtering mode is intercepted by SearchPrompt as FilterType,
                        // so it never reaches this arm in non-Dashboard modes.
                        let is_quit = matches!(&action, Action::Quit);

                        // F-S025-ADV2-HIGH-003: Overlay stack mutations are App-level.
                        // PopOverlay: pop from overlay_stack first; transition() always
                        // returns Dashboard; re-enter Overlay if stack still non-empty.
                        // OverlayCycleNext: rotate overlay_stack; mode stays Overlay.
                        // PushOverlay from key binding is unusual (normally IPC-driven)
                        // but still handled correctly: push to overlay_stack, then transition.
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
                            _ => {
                                app.mode = transition(app.mode.clone(), action);
                            }
                        }

                        if is_quit {
                            break;
                        }
                    }
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
                        on_transport_event(&mut app, TransportEvent::Disconnected);
                        reader_handle.abort();

                        // TODO(S-023-merge): reconnect call site.
                        //
                        // Reconnect logic (exponential backoff 250ms→2s, 5s total window)
                        // belongs to S-023's `monocle_ipc::reconnect` module. S-023 is
                        // in parallel development and has NOT merged to develop yet.
                        //
                        // When S-023 merges:
                        //   1. Replace this block with:
                        //      match monocle_ipc::reconnect::reconnect_with_backoff(&sock_path).await {
                        //          Ok((new_transport, _)) => {
                        //              // Re-create the channel (F-S025-ADV3-MED-003):
                        //              // ipc_tx was moved to the old reader task; re-creation
                        //              // is necessary and has negligible allocation cost.
                        //              let (ipc_tx2, ipc_rx2) = tokio::sync::mpsc::channel(64);
                        //              ipc_rx = ipc_rx2;
                        //              reader_handle = spawn_ipc_reader(new_transport, ipc_tx2);
                        //              app.status_message = None; // reconnected
                        //          }
                        //          Err(IpcError::ReconnectTimeout) => {
                        //              app.status_message = Some("[daemon: offline]".to_string());
                        //          }
                        //          Err(e) => { tracing::error!(error = %e, "reconnect failed"); }
                        //      }
                        //   2. Delete this TODO block.
                        //   3. Verify variant shapes against BC-2.05.007.
                        //
                        // This is the ONE acceptable TODO marker in S-025 — it tracks a
                        // merge-coordination dependency (S-023 not yet on develop), NOT a
                        // deferred defect. The reconnect scaffolding exists; S-023 provides
                        // the API. See F-S025-ADV2-BLOCKER-001 architect decision doc §Cross-Story.
                        tracing::warn!(
                            "reconnect not yet available (pending S-023 merge); \
                                        entering offline mode"
                        );
                        app.status_message = Some("[daemon: offline]".to_string());
                        break;
                    }
                }
                Ok(Err(e)) => {
                    // Reader task forwarded a disconnect or transport error.
                    tracing::warn!(error = %e, "IPC reader task disconnect; entering reconnect state");
                    on_transport_event(&mut app, TransportEvent::Disconnected);
                    reader_handle.abort();

                    // TODO(S-023-merge): same reconnect call site as above (channel re-creation).
                    // See the detailed comment in the protocol-violation arm above.
                    tracing::warn!(
                        "reconnect not yet available (pending S-023 merge); \
                                    entering offline mode"
                    );
                    app.status_message = Some("[daemon: offline]".to_string());
                    break;
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

    // Clean exit: abort the reader task before returning so the tokio runtime doesn't
    // leak the background task between test runs or on graceful shutdown.
    reader_handle.abort();

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
        ServerToClient::PermissionPromptResolved { prompt_id } => {
            app.overlay_stack.retain(|m| m.prompt_id != prompt_id);
            // F-S025-ADV2-HIGH-003: if stack is now empty, collapse to Dashboard.
            if app.overlay_stack.is_empty() {
                if let AppMode::Overlay { prior } = app.mode.clone() {
                    app.mode = AppMode::Dashboard { focused: prior };
                }
            }
        }
        ServerToClient::HookEventReceived { .. } => {
            // Hook events update the event ribbon — handled in S-027.
            tracing::trace!("HookEventReceived: event ribbon update deferred to S-027");
        }
    }
    Ok(())
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
/// - Esc (builtin) → `Action::Esc` — context-sensitive: Dashboard=identity, Overlay=no-op,
///   Filtering=cancel (wired in transition()). Not used as a quit path.
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
