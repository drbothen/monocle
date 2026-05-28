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
/// Populates `app.sessions`, `app.drop_counter`, and `app.overlay_stack` from
/// the daemon's initial state push. Each entry in `overlay_stack` is inserted
/// via `apply_permission_prompt_queued` to enforce prompt_id idempotency.
/// If the resulting overlay_stack is non-empty, transitions to `AppMode::Overlay`.
pub fn on_initial_state(
    app: &mut App,
    sessions: Vec<EnrichedSession>,
    _ring_tail: Vec<HookEventRecord>,
    overlay_stack: Vec<PermissionPromptPayload>,
    drop_counter: u64,
) {
    app.sessions = sessions;
    app.drop_counter = drop_counter;

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
    use crossterm::event::{self, Event, KeyCode, KeyEvent};
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
            // Wait for any keypress before exiting with code 1 (AC-002).
            loop {
                if event::poll(Duration::from_millis(200))? {
                    if let Event::Key(_) = event::read()? {
                        break;
                    }
                }
            }
            std::process::exit(1);
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
            tracing::warn!(
                "expected InitialState as first message, got unexpected variant; ignoring"
            );
            drop(other);
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to receive InitialState; continuing with empty state");
        }
    }

    // Set up the ratatui terminal.
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Main event loop.
    let tick_rate = Duration::from_millis(16); // ~60fps

    loop {
        // Render the current frame.
        let _ = terminal.draw(|_frame| {
            // TODO(S-025): full layout rendering is wired in S-026/S-027.
            // For S-025, we have the app state ready; render is exercised via
            // unit tests. The main.rs draw loop will be completed in S-027
            // which adds the full layout render pass.
        });

        // Poll for input events.
        if event::poll(tick_rate)? {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Esc,
                ..
            }) = event::read()?
            {
                // AC-001: clean exit from Dashboard mode.
                if matches!(app.mode, AppMode::Dashboard { .. }) {
                    break;
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
                handle_server_message(&mut app, msg);
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
fn handle_server_message(app: &mut App, msg: ServerToClient) {
    match msg {
        ServerToClient::InitialState {
            sessions,
            ring_tail,
            overlay_stack,
            drop_counter,
        } => {
            on_initial_state(app, sessions, ring_tail, overlay_stack, drop_counter);
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
}
