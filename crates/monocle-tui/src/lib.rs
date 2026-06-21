//! `monocle-tui` — terminal user interface binary crate for monocle.
//!
//! This lib target exists so that integration tests in `tests/` can import
//! `monocle_tui::app::App` and `monocle_tui::apply_permission_prompt_queued`
//! without duplicating the `[[bin]]` build unit.
//!
//! # Architecture boundary (SS-tui.md)
//!
//! `monocle-tui` is the effectful boundary: ratatui, crossterm, tokio, and all
//! terminal I/O live here. `monocle-core` (pure) is a dependency of this crate —
//! not the reverse. The crate MUST NOT be depended upon by `monocle-core`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod app;
pub mod ui;

/// Crossterm-to-PtyKey type conversions — the ONLY place in the workspace where
/// crossterm and ratatui types touch the `monocle-core` purity boundary (S-040).
///
/// Contains `crossterm_key_to_pty()` for converting `crossterm::event::KeyEvent`
/// to `monocle_core::keyboard::PtyKeyEvent`. S-041 extends this file with
/// `crossterm_mouse_to_pty()` and `ratatui_rect_to_pty()`.
pub mod keyboard_conv;

/// Keyboard enhancement setup/teardown and `AppMode::EmbeddedTerminal` key/paste dispatch (S-040).
///
/// Contains `setup_keyboard_enhancement()`, `teardown_keyboard_enhancement()`,
/// `dispatch_embedded_terminal_key()`, and `dispatch_embedded_terminal_paste()`.
pub mod event_loop;

// Re-exports for integration tests and downstream consumers.
pub use app::apply_permission_prompt_queued;
pub use app::format_drop_counter;
pub use app::payload_to_modal;
pub use app::resolve_runtime_dir;
pub use app::spawn_ipc_reader;
pub use app::App;
pub use app::DAEMON_DISCONNECT_STATUS;
pub use app::DAEMON_NOT_RUNNING_ERROR;
pub use app::DAEMON_OFFLINE_STATUS;
pub use app::EVENT_RING_CAPACITY;
pub use app::MONOCLE_STATUS_LABEL;
// S-026: PermissionDecisionKind re-exported for overlay decision integration tests.
pub use app::PermissionDecisionKind;
// S-031: profile picker state and handlers re-exported for integration tests.
pub use app::close_profile_picker;
pub use app::commit_profile_selection;
pub use app::commit_profile_selection_with_path;
pub use app::open_profile_picker;
pub use app::open_profile_picker_with_dir;
pub use app::picker_select_next;
pub use app::picker_select_prev;
pub use app::ProfilePickerState;
// Re-export pub consts from ui/sessions_panel.rs for external test crates
// (L-W6-S025-003 discipline: pub const extraction must be accessible at crate root)
pub use ui::sessions_panel::SESSIONS_EMPTY_LINE_1;
pub use ui::sessions_panel::SESSIONS_EMPTY_LINE_2;
pub use ui::sessions_panel::TOKEN_COUNT_OVERFLOW_CAP;
pub use ui::sessions_panel::UPTIME_OVERFLOW_CAP;

// S-029: inbound IPC dispatch seam — exposed for killer-scenario E2E test (H-1 closure).
// These are NOT part of the stable public API; #[doc(hidden)] signals "test seam only".
// External integration tests import these via `monocle_tui::handle_server_message` /
// `monocle_tui::setup_ipc_streams_with_rx` (or via `monocle_tui::app::{...}`).
#[doc(hidden)]
pub use app::handle_server_message;
#[doc(hidden)]
pub use app::setup_ipc_streams_with_rx;

// S-039: PTY output pipeline channel primitives — exposed for AC-007 / Invariant-3 tests.
// Tests assert `pty_output_channel()` returns a Receiver with `max_capacity()` == 64,
// going RED against the todo!() stub until S-039 implements it.
#[doc(hidden)]
pub use app::pty_output_channel;
#[doc(hidden)]
pub use app::IPC_READER_CHANNEL_CAPACITY;

// S-040: crossterm event routing seam — exposed for wiring integration tests.
// handle_crossterm_event encapsulates the per-event dispatch (Key → embedded dispatch
// or binding chain; Paste → bracketed paste) so tests can drive it without a real
// terminal event loop.
#[doc(hidden)]
pub use app::handle_crossterm_event;
// S-040: builtin binding layers constructor — exposed so integration tests can build
// a real BindingLayers without re-implementing the full layer stack.
#[doc(hidden)]
pub use app::build_builtin_binding_layers;

// S-042: resize detection + debounce functions — exposed for BC-2.09.006 TDD tests.
// These are test seams; not part of the stable public API.
#[doc(hidden)]
pub use app::check_resize_debounce;
#[doc(hidden)]
pub use app::clear_resize_debounce_state;
#[doc(hidden)]
pub use app::exit_embedded_terminal;
#[doc(hidden)]
pub use app::on_resize_detected;
