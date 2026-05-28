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
use monocle_ipc::framing::read_framed;
use monocle_ipc::types::{HookEventRecord, PermissionPromptPayload, ServerToClient};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Stub types
// ---------------------------------------------------------------------------

/// Signal that the IPC transport has changed connection state.
///
/// TODO(S-025): align with monocle-ipc::TransportEvent once S-023 (daemon
/// reconnect) is merged and `TransportEvent` is defined in `monocle-ipc`.
/// Until then this local stub provides the minimal surface for tests and
/// the event loop match arm.
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
        app.mode = AppMode::Overlay {
            stack: app.overlay_stack.clone(),
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

    // Set up the ratatui terminal.
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Render state for the Sessions panel (selection tracking).
    let mut sessions_state = crate::ui::sessions_panel::SessionsPanelState::default();

    // Build the builtin binding layers once for the session (AC-006, BLOCKER-002).
    // Future: merge user-custom and per-context layers from config.
    let binding_layers = build_builtin_binding_layers();

    // Main event loop.
    let tick_rate = Duration::from_millis(16); // ~60fps

    loop {
        // Render the current frame (AC-001, AC-005, BLOCKER-004).
        terminal.draw(|frame| {
            use crate::ui::layout::build_dashboard_layout;
            use crate::ui::sessions_panel::SessionsPanel;
            use ratatui::{
                style::{Color, Style},
                text::{Line, Span},
                widgets::{Paragraph, StatefulWidget, Widget},
            };

            let layout = build_dashboard_layout(frame.area());

            // Render the Sessions panel (left 60%).
            let panel = SessionsPanel::new(&app);
            panel.render(layout.sessions_area, frame.buffer_mut(), &mut sessions_state);

            // Render the status bar (bottom 2 rows): drop counter + breadcrumb.
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
            Widget::render(
                Paragraph::new(status_line),
                layout.status_bar_area,
                frame.buffer_mut(),
            );
        })?;

        // Poll for input events (BLOCKER-002: full binding dispatch via resolve_binding).
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
                        // Move selection down in the current panel list.
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
                    Some((monocle_core::tui::state::Action::SelectPrev, _)) => {
                        // Move selection up in the current panel list.
                        if app.sessions.len() > 0 {
                            let prev = sessions_state
                                .list_state
                                .selected()
                                .map(|i| i.saturating_sub(1))
                                .unwrap_or(0);
                            sessions_state.list_state.select(Some(prev));
                        }
                    }
                    Some((action, _)) => {
                        // All other actions: drive the AppMode state machine.
                        use monocle_core::tui::state::transition;
                        // Check for clean exit: q or Esc from Dashboard.
                        let is_quit = matches!(
                            (&app.mode, &action),
                            (AppMode::Dashboard { .. }, monocle_core::tui::state::Action::Esc)
                        );
                        app.mode = transition(app.mode.clone(), action);
                        if is_quit {
                            break;
                        }
                    }
                }
            }
        }

        // Poll for IPC messages (non-blocking: use try_recv equivalent).
        // We use a 1ms poll to avoid blocking the render loop.
        match tokio::time::timeout(
            Duration::from_millis(1),
            read_framed::<_, ServerToClient>(&mut transport),
        )
        .await
        {
            Ok(Ok(msg)) => {
                if let Err(e) = handle_server_message(&mut app, msg) {
                    // Protocol violation (e.g., duplicate InitialState) — close connection.
                    tracing::error!(error = %e, "fatal protocol error; closing IPC connection");
                    on_transport_event(&mut app, TransportEvent::Disconnected);
                    break;
                }
            }
            Ok(Err(e)) => {
                // Connection lost — treat as Disconnected.
                tracing::warn!(error = %e, "IPC read error; treating as disconnect");
                on_transport_event(&mut app, TransportEvent::Disconnected);
            }
            Err(_timeout) => {
                // Normal: no message within 1ms.
            }
        }
    }

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
        }
        ServerToClient::PermissionPromptResolved { prompt_id } => {
            app.overlay_stack.retain(|m| m.prompt_id != prompt_id);
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
/// - Esc / q (Dashboard) → exit (resolved as `Action::Esc`)
/// - Tab → `Action::MoveFocus` (cycle Sessions ↔ EventRibbon)
/// - Enter → `Action::EnterFullscreen { Sessions }` (expand current panel)
/// - j / ↓ → `Action::SelectNext` (move selection down)
/// - k / ↑ → `Action::SelectPrev` (move selection up)
///
/// Future waves add user-custom and per-context layers; for now only builtin
/// and global layers are populated.
pub fn build_builtin_binding_layers() -> monocle_core::tui::binding::BindingLayers {
    use monocle_core::tui::binding::{BindingLayers, KeyCode, KeyEvent, KeyModifiers};
    use monocle_core::tui::state::{Action, PanelId};

    let no_mod = KeyModifiers::default();

    let mut layers = BindingLayers::empty();

    // Global bindings (active in all modes).
    // Tab → MoveFocus
    layers.global.insert(
        KeyEvent { code: KeyCode::Tab, modifiers: no_mod.clone() },
        Action::MoveFocus,
    );

    // Builtin bindings (lowest precedence; hard-coded fallbacks).
    // Esc → Esc (handled by transition; Dashboard+Esc is the quit path)
    layers.builtin.insert(
        KeyEvent { code: KeyCode::Esc, modifiers: no_mod.clone() },
        Action::Esc,
    );
    // q (no modifiers) → Esc (same as physical Esc for Dashboard quit)
    layers.builtin.insert(
        KeyEvent { code: KeyCode::Char('q'), modifiers: no_mod.clone() },
        Action::Esc,
    );
    // Enter → EnterFullscreen { Sessions }
    layers.builtin.insert(
        KeyEvent { code: KeyCode::Enter, modifiers: no_mod.clone() },
        Action::EnterFullscreen { panel: PanelId::Sessions },
    );
    // j → SelectNext
    layers.builtin.insert(
        KeyEvent { code: KeyCode::Char('j'), modifiers: no_mod.clone() },
        Action::SelectNext,
    );
    // ↓ → SelectNext
    layers.builtin.insert(
        KeyEvent { code: KeyCode::Down, modifiers: no_mod.clone() },
        Action::SelectNext,
    );
    // k → SelectPrev
    layers.builtin.insert(
        KeyEvent { code: KeyCode::Char('k'), modifiers: no_mod.clone() },
        Action::SelectPrev,
    );
    // ↑ → SelectPrev
    layers.builtin.insert(
        KeyEvent { code: KeyCode::Up, modifiers: no_mod.clone() },
        Action::SelectPrev,
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
        // Any other crossterm key code maps to Noop via no binding match.
        // We emit Char('\0') as a safe unmappable sentinel that won't match
        // any registered binding.
        _ => KeyCode::Char('\0'),
    };

    let modifiers = KeyModifiers {
        shift: ct.modifiers.contains(CtMod::SHIFT),
        ctrl: ct.modifiers.contains(CtMod::CONTROL),
        alt: ct.modifiers.contains(CtMod::ALT),
    };

    KeyEvent { code, modifiers }
}
