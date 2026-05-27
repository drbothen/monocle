//! TUI plane — core types for the monocle terminal user interface.
//!
//! Provides `AppMode`, `FocusSnapshot`, `PanelId`, `Action`, and `PromptModal`
//! (state machine types) and key-binding resolution types (`binding` submodule).
//! No I/O or terminal dependencies — this module is pure-core; all ratatui/crossterm
//! integration lives in the `monocle` binary crate.

pub mod binding;
pub mod state;
