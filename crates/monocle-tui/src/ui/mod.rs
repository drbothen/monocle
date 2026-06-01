//! UI submodules for monocle-tui rendering.
//!
//! - `sessions_panel`: `SessionsPanel` widget (BC-2.06.005, BC-2.06.007).
//! - `layout`: Layout builder for Dashboard and Fullscreen modes.
//! - `overlay`: Permission overlay rendering (S-027 fills the implementation;
//!   module declared here per S-026 §File Structure Requirements).
//! - `overlay_widget`: Overlay modal widget functions (S-027, BC-2.06.010/015/024).
//! - `status_bar`: Always-visible status bar widget (S-027, BC-2.06.019/020/021).
//! - `event_ribbon`: Event Ribbon rolling log panel (S-028, BC-2.06.018).

pub mod event_ribbon;
pub mod layout;
pub mod overlay;
pub mod overlay_widget;
pub mod sessions_panel;
pub mod status_bar;
