//! `monocle-tui` — terminal user interface binary crate for monocle.
//!
//! This lib target exists so that integration tests in `tests/` can import
//! `monocle_tui::app::App` and `monocle_tui::apply_permission_prompt_queued`
//! without duplicating the `[[bin]]` build unit.
//!
//! # Architecture boundary (SS-tui-core.md)
//!
//! `monocle-tui` is the effectful boundary: ratatui, crossterm, tokio, and all
//! terminal I/O live here. `monocle-core` (pure) is a dependency of this crate —
//! not the reverse. The crate MUST NOT be depended upon by `monocle-core`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod app;
pub mod ui;

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
// Re-export pub consts from ui/sessions_panel.rs for external test crates
// (L-W6-S025-003 discipline: pub const extraction must be accessible at crate root)
pub use ui::sessions_panel::SESSIONS_EMPTY_LINE_1;
pub use ui::sessions_panel::SESSIONS_EMPTY_LINE_2;
pub use ui::sessions_panel::TOKEN_COUNT_OVERFLOW_CAP;
pub use ui::sessions_panel::UPTIME_OVERFLOW_CAP;
