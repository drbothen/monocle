//! Permission overlay rendering for monocle-tui (S-027 implementation).
//!
//! This module is declared by S-026 (Permission Overlay Core) as a compilation
//! placeholder. The rendering implementation is the responsibility of S-027
//! (Overlay Rendering + Diff Preview).
//!
//! # Architecture
//!
//! The overlay renders `App::overlay_stack.front()` as the active prompt modal
//! centred over the dashboard layout. S-027 fills in:
//! - The `render_overlay` function that draws the `PromptModal` as a ratatui widget.
//! - The diff preview pane for `ToolPayload::Edit` prompts.
//! - The badge counter showing `App::overlay_stack.len()` remaining prompts.
//!
//! This module intentionally contains no implementation logic — adding any
//! rendering stubs here would create false-green tests for S-027 (Red Gate
//! violation per BC-5.38.001).
