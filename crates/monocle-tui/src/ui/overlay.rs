//! Permission overlay rendering for monocle-tui (S-027 implementation).
//!
//! This module serves as the namespace anchor for the overlay subsystem. The
//! actual rendering logic lives in `overlay_widget.rs` (S-027):
//!
//! # Architecture
//!
//! The overlay renders `App::overlay_stack.front()` as the active prompt modal
//! centred over the dashboard layout. Implemented by S-027:
//! - `render_overlay_widget` in `overlay_widget.rs` — draws the `PromptModal`
//!   as a centered ratatui widget with header, body, and footer.
//! - `render_dimmed_background` — dims all non-status-bar cells behind the modal.
//! - Diff preview pane for `ToolPayload::Edit` prompts (via `render_edit_payload`).
//! - Stack depth indicator showing `App::overlay_stack.len()` remaining prompts.
