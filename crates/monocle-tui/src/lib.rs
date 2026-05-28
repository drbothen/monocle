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
pub use app::resolve_runtime_dir;
pub use app::App;
