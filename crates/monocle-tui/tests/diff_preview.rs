//! Diff preview rendering tests for S-027.
//!
//! This module is the test file placeholder declared in S-027 §File Structure
//! Requirements. The test-writer agent fills this file with failing tests for:
//!
//! - BC-2.06.010 (diff via `similar`): color coding (red/green/default), height cap
//! - BC-2.06.015: `similar` purity boundary — crate in monocle-tui ONLY
//! - BC-2.06.024 PC-2: `ToolPayload::Edit` diff body rendering
//!
//! All test names follow the canonical pattern `test_BC_S_SS_NNN_*`.
//!
//! Canonical test vectors from BC-2.06.010:
//!
//! | Input | Expected Output |
//! |-------|----------------|
//! | Edit { old: "fn foo() {}\n", new: "fn bar() {}\n" } | `-fn foo()` red, `+fn bar()` green |
//! | Edit { old: "", new: "fn new() {}\n" } | All lines green (Insert only) |
//! | Edit { old: "fn keep() {}\n", new: "fn keep() {}\n" } | One Equal line, default color |
//! | Bash { command: "rm -rf /tmp/test" } | No diff rendered |
//! | Edit diff exceeding (overlay_height - 8) rows | Truncated at height cap; no panic |
//!
//! This file intentionally contains no test logic — it is the Red Gate scaffold.
//! The test-writer agent writes the failing tests; the implementer then makes them pass.
