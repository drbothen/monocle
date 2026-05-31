//! UI submodules for monocle-tui rendering.
//!
//! - `sessions_panel`: `SessionsPanel` widget (BC-2.06.005, BC-2.06.007).
//! - `layout`: Layout builder for Dashboard and Fullscreen modes.
//! - `overlay`: Permission overlay rendering (S-027 fills the implementation;
//!   module declared here per S-026 §File Structure Requirements).

pub mod layout;
pub mod overlay;
pub mod sessions_panel;
