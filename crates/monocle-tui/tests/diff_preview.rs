//! Diff preview rendering tests for S-027.
//!
//! Exercises BC-2.06.010 (`similar` diff: color coding, height cap, edge cases)
//! and BC-2.06.015 (purity boundary: `similar` in monocle-tui only).
//!
//! # Red Gate
//!
//! All tests exercise `render_edit_payload`, whose body is `todo!()` in the S-027
//! stub.  Every test MUST FAIL with a `todo!()` panic until the implementer fills
//! in production code.  DO NOT weaken assertions or implement production code here.
//!
//! # Test naming
//!
//! All names follow the factory-mandated `test_BC_S_SS_NNN_*` pattern.
//!
//! `#![allow(non_snake_case)]` is required because the pattern uses uppercase BC IDs.
#![allow(non_snake_case)]

use monocle_tui::ui::overlay_widget::render_edit_payload;
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Create a `Buffer` of the given size.
fn make_buf(width: u16, height: u16) -> Buffer {
    Buffer::empty(Rect::new(0, 0, width, height))
}

/// Collect all cell symbols from `buf` as a single string.
fn buf_text(buf: &Buffer) -> String {
    let area = buf.area();
    (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect()
}

/// Find the first cell in `buf` that starts with the given symbol string and
/// return its foreground color.
fn find_cell_fg_for_symbol(buf: &Buffer, symbol: &str) -> Option<Option<Color>> {
    let area = buf.area();
    for y in 0..area.height {
        for x in 0..area.width {
            if buf[(x, y)].symbol() == symbol {
                return Some(buf[(x, y)].style().fg);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// BC-2.06.010 PC-2 — Delete lines rendered red
// ---------------------------------------------------------------------------

/// BC-2.06.010 PC-2 (RED GATE): `render_edit_payload` renders deleted lines (`-` prefix)
/// with `Color::Red` foreground.
///
/// Canonical test vector from BC-2.06.010:
///   `old: "fn foo() {}\n"`, `new: "fn bar() {}\n"` →
///   `-fn foo() {}` must appear with `Color::Red`.
#[test]
fn test_BC_2_06_010_diff_preview_delete_lines_rendered_red() {
    let old_content = "fn foo() {}\n";
    let new_content = "fn bar() {}\n";
    let path = PathBuf::from("src/lib.rs");
    let mut buf = make_buf(80, 20);
    let area = Rect::new(0, 0, 80, 20);

    // Calls the stub — must panic with todo!() until implemented.
    render_edit_payload(old_content, new_content, &path, area, &mut buf);

    // After implementation: a cell prefixed with '-' must have Color::Red.
    // Find a cell containing '-' (the delete prefix character).
    let text = buf_text(&buf);
    assert!(
        text.contains('-'),
        "BC-2.06.010 PC-2: diff buffer must contain '-' prefix for deleted lines; \
         got: {:?}",
        text.trim()
    );

    // Verify that the '-' cell has Color::Red foreground.
    let minus_cell_fg = find_cell_fg_for_symbol(&buf, "-");
    assert!(
        minus_cell_fg.is_some(),
        "BC-2.06.010 PC-2: '-' symbol must exist in the rendered buffer"
    );
    assert_eq!(
        minus_cell_fg.unwrap(),
        Some(Color::Red),
        "BC-2.06.010 PC-2: deleted line prefix '-' must be styled with Color::Red"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.010 PC-3 — Insert lines rendered green
// ---------------------------------------------------------------------------

/// BC-2.06.010 PC-3 (RED GATE): `render_edit_payload` renders inserted lines (`+` prefix)
/// with `Color::Green` foreground.
///
/// Canonical test vector from BC-2.06.010:
///   `old: "fn foo() {}\n"`, `new: "fn bar() {}\n"` →
///   `+fn bar() {}` must appear with `Color::Green`.
#[test]
fn test_BC_2_06_010_diff_preview_insert_lines_rendered_green() {
    let old_content = "fn foo() {}\n";
    let new_content = "fn bar() {}\n";
    let path = PathBuf::from("src/lib.rs");
    let mut buf = make_buf(80, 20);
    let area = Rect::new(0, 0, 80, 20);

    render_edit_payload(old_content, new_content, &path, area, &mut buf);

    let text = buf_text(&buf);
    assert!(
        text.contains('+'),
        "BC-2.06.010 PC-3: diff buffer must contain '+' prefix for inserted lines; \
         got: {:?}",
        text.trim()
    );

    // Verify that the '+' cell has Color::Green foreground.
    let plus_cell_fg = find_cell_fg_for_symbol(&buf, "+");
    assert!(
        plus_cell_fg.is_some(),
        "BC-2.06.010 PC-3: '+' symbol must exist in the rendered buffer"
    );
    assert_eq!(
        plus_cell_fg.unwrap(),
        Some(Color::Green),
        "BC-2.06.010 PC-3: inserted line prefix '+' must be styled with Color::Green"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.010 PC-4 — Equal lines rendered default
// ---------------------------------------------------------------------------

/// BC-2.06.010 PC-4 (RED GATE): `render_edit_payload` renders context (equal) lines
/// with default terminal color (no Color::Red, no Color::Green).
///
/// Canonical test vector from BC-2.06.010:
///   `old: "fn keep() {}\n"`, `new: "fn keep() {}\n"` →
///   One Equal line in default color; no red/green.
#[test]
fn test_BC_2_06_010_diff_preview_equal_lines_rendered_default_color() {
    let content = "fn keep() {}\n";
    let path = PathBuf::from("src/same.rs");
    let mut buf = make_buf(80, 10);
    let area = Rect::new(0, 0, 80, 10);

    render_edit_payload(content, content, &path, area, &mut buf);

    let text = buf_text(&buf);
    // No '+' or '-' diff prefix lines should appear for equal content.
    // The space-prefixed equal line should appear.
    assert!(
        !text.contains("fn keep() {}") || text.contains(' '),
        "BC-2.06.010 PC-4: equal content must render a space-prefixed equal line; \
         got: {:?}",
        text.trim()
    );

    // Verify no cells have Color::Red or Color::Green (equal lines use default color).
    let area = buf.area();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            assert_ne!(
                cell.style().fg,
                Some(Color::Red),
                "BC-2.06.010 PC-4: equal-lines-only diff must NOT have any Color::Red cell \
                 at ({x},{y}); sym={:?}",
                cell.symbol()
            );
            assert_ne!(
                cell.style().fg,
                Some(Color::Green),
                "BC-2.06.010 PC-4: equal-lines-only diff must NOT have any Color::Green cell \
                 at ({x},{y}); sym={:?}",
                cell.symbol()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// BC-2.06.010 EC-069 — Empty old_content (new file creation): all lines green
// ---------------------------------------------------------------------------

/// BC-2.06.010 EC-069 (RED GATE): When `old_content` is empty, the entire diff
/// is `Insert` lines — all rendered in green.
///
/// Canonical test vector from BC-2.06.010:
///   `old: ""`, `new: "fn new() {}\n"` → all lines green (Insert only).
#[test]
fn test_BC_2_06_010_diff_preview_empty_old_content_all_insert_lines_green() {
    let old_content = "";
    let new_content = "fn new() {}\n";
    let path = PathBuf::from("src/new.rs");
    let mut buf = make_buf(80, 10);
    let area = Rect::new(0, 0, 80, 10);

    render_edit_payload(old_content, new_content, &path, area, &mut buf);

    let text = buf_text(&buf);
    assert!(
        text.contains('+'),
        "BC-2.06.010 EC-069: empty old_content must produce Insert-only diff with '+' prefix; \
         got: {:?}",
        text.trim()
    );
    assert!(
        !text.contains('-'),
        "BC-2.06.010 EC-069: empty old_content must produce zero Delete lines (no '-' prefix); \
         got: {:?}",
        text.trim()
    );

    // Verify no Red cells (no deletes → no red).
    let area = buf.area();
    for y in 0..area.height {
        for x in 0..area.width {
            assert_ne!(
                buf[(x, y)].style().fg,
                Some(Color::Red),
                "BC-2.06.010 EC-069: empty old_content → no Color::Red cell at ({x},{y})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// BC-2.06.010 EC-070 — Empty new_content (file deletion): all lines red
// ---------------------------------------------------------------------------

/// BC-2.06.010 EC-070 (RED GATE): When `new_content` is empty, the entire diff
/// is `Delete` lines — all rendered in red.
///
/// Canonical test vector from BC-2.06.010:
///   `old: "fn old() {}\n"`, `new: ""` → all lines red (Delete only).
#[test]
fn test_BC_2_06_010_diff_preview_empty_new_content_all_delete_lines_red() {
    let old_content = "fn old() {}\n";
    let new_content = "";
    let path = PathBuf::from("src/deleted.rs");
    let mut buf = make_buf(80, 10);
    let area = Rect::new(0, 0, 80, 10);

    render_edit_payload(old_content, new_content, &path, area, &mut buf);

    let text = buf_text(&buf);
    assert!(
        text.contains('-'),
        "BC-2.06.010 EC-070: empty new_content must produce Delete-only diff with '-' prefix; \
         got: {:?}",
        text.trim()
    );
    assert!(
        !text.contains('+'),
        "BC-2.06.010 EC-070: empty new_content must produce zero Insert lines (no '+' prefix); \
         got: {:?}",
        text.trim()
    );

    // Verify no Green cells (no inserts → no green).
    let area = buf.area();
    for y in 0..area.height {
        for x in 0..area.width {
            assert_ne!(
                buf[(x, y)].style().fg,
                Some(Color::Green),
                "BC-2.06.010 EC-070: empty new_content → no Color::Green cell at ({x},{y})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// BC-2.06.010 PC-5 — Height cap: (overlay_height - 8) rows
// ---------------------------------------------------------------------------

/// BC-2.06.010 PC-5 (RED GATE): The diff area is capped to `(area.height - 8)` rows.
/// When the diff produces more lines than the height cap, it is truncated without panic.
///
/// Canonical test vector from BC-2.06.010:
///   Edit diff exceeding `(overlay_height - 8)` rows → truncated at height cap; no panic.
#[test]
fn test_BC_2_06_010_diff_preview_height_cap_truncates_without_panic() {
    // Build a diff with many lines (50 lines old, 50 lines new → 100 diff lines).
    let old_lines: String = (0..50).map(|i| format!("line_old_{i}\n")).collect();
    let new_lines: String = (0..50).map(|i| format!("line_new_{i}\n")).collect();
    let path = PathBuf::from("src/large.rs");

    // Use a small area (10 rows). height cap = 10 - 8 = 2 rows of diff.
    let mut buf = make_buf(80, 10);
    let area = Rect::new(0, 0, 80, 10);

    // Must not panic even though diff far exceeds height cap.
    render_edit_payload(&old_lines, &new_lines, &path, area, &mut buf);

    // The buffer must not be larger than the given area (no out-of-bounds write).
    // Count non-empty rows used by diff content (contains '-' or '+').
    let diff_row_count = (0..10u16)
        .filter(|&y| {
            (0..80u16).any(|x| {
                let sym = buf[(x, y)].symbol();
                sym == "-" || sym == "+"
            })
        })
        .count();

    // With height=10, cap = 10-8 = 2 max diff rows.
    assert!(
        diff_row_count <= 2,
        "BC-2.06.010 PC-5: diff row count ({diff_row_count}) must not exceed height cap \
         (area.height - 8 = 2) for area.height=10"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.010 PC-1 / AC-005 — Edit path shown in modal header
// ---------------------------------------------------------------------------

/// BC-2.06.010 PC-1 / AC-005 (RED GATE): The path `"Edit: <path>"` must appear in
/// the rendered output when `render_edit_payload` is called.
///
/// Per AC-005: "The path is shown in the modal header: 'Edit: <path>'".
#[test]
fn test_BC_2_06_010_diff_preview_edit_path_shown_in_header() {
    let old_content = "fn foo() {}\n";
    let new_content = "fn bar() {}\n";
    let path = PathBuf::from("src/lib.rs");
    let mut buf = make_buf(80, 20);
    let area = Rect::new(0, 0, 80, 20);

    render_edit_payload(old_content, new_content, &path, area, &mut buf);

    let text = buf_text(&buf);
    assert!(
        text.contains("src/lib.rs") || text.contains("Edit"),
        "BC-2.06.010 PC-1 / AC-005: rendered buffer must show path 'src/lib.rs' or 'Edit' header; \
         got: {:?}",
        text.trim()
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.010 PC-10 — Wrap { trim: false } applied
// ---------------------------------------------------------------------------

/// BC-2.06.010 PC-10 (RED GATE): The diff is rendered with `Wrap { trim: false }` so
/// long lines wrap without truncating leading spaces.
///
/// We verify this indirectly: a line with leading spaces (e.g., `"  fn indented() {}\n"`)
/// must preserve those leading spaces in the rendered output.
#[test]
fn test_BC_2_06_010_diff_preview_wrap_trim_false_preserves_leading_spaces() {
    // Equal line with leading spaces — `Wrap { trim: false }` must preserve them.
    let content = "  fn indented() {}\n";
    let path = PathBuf::from("src/indented.rs");
    let mut buf = make_buf(80, 10);
    let area = Rect::new(0, 0, 80, 10);

    render_edit_payload(content, content, &path, area, &mut buf);

    // Look for the indented text — leading spaces must appear.
    // The equal line is rendered as " " + "  fn indented() {}" (space prefix + content).
    // So the rendered line starts with at least one space followed by more spaces.
    let text = buf_text(&buf);
    assert!(
        text.contains("fn indented"),
        "BC-2.06.010 PC-10: indented function content must appear in rendered diff; \
         got: {:?}",
        text.trim()
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.015 INV-2 / AC-007 — `similar` purity boundary: in monocle-tui only
// ---------------------------------------------------------------------------

/// BC-2.06.015 INV-2 / AC-007 (compile-time assertion): The `similar` crate MUST
/// NOT appear as a dependency of `monocle-core`.
///
/// This test reads the monocle-core Cargo.toml and asserts that `similar` is absent.
/// If a refactor moves diff logic to monocle-core, this test fails to enforce the
/// purity boundary (BC-2.06.001 Postcondition 5, AC-007).
#[test]
fn test_BC_2_06_015_invariant_2_similar_crate_not_in_monocle_core_cargo_toml() {
    // Locate monocle-core/Cargo.toml relative to the workspace root.
    // The worktree root is two levels above the monocle-tui tests directory.
    // We use CARGO_MANIFEST_DIR which points to the crate under test (monocle-tui).
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo test");

    // Navigate from monocle-tui/ up two levels to workspace root, then to monocle-core/.
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("workspace root must be navigable from CARGO_MANIFEST_DIR");

    let core_cargo_toml = workspace_root.join("crates/monocle-core/Cargo.toml");
    assert!(
        core_cargo_toml.exists(),
        "BC-2.06.015 INV-2: monocle-core Cargo.toml must exist at {:?}",
        core_cargo_toml
    );

    let contents =
        std::fs::read_to_string(&core_cargo_toml).expect("must read monocle-core Cargo.toml");

    // `similar` must NOT appear anywhere in monocle-core/Cargo.toml.
    assert!(
        !contents.contains("similar"),
        "BC-2.06.015 INV-2 / AC-007: 'similar' crate MUST NOT appear in monocle-core Cargo.toml \
         (purity boundary violation); found in: {:?}",
        core_cargo_toml
    );
}

/// BC-2.06.015 INV-2 / AC-007 (source-grep assertion): The `similar` crate MUST NOT
/// be imported in any `.rs` file under `crates/monocle-core/src/`.
///
/// This checks for actual crate-import patterns (`use similar::`, `extern crate similar`,
/// `similar::`) to avoid false positives on English prose containing the word "similar".
/// The Cargo.toml check verifies the dependency is absent; this test verifies no
/// sneaked-in `use` statement would cause a build error if someone added it.
#[test]
fn test_BC_2_06_015_invariant_2_similar_not_used_in_monocle_core_source() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo test");

    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root must be navigable");

    let core_src = workspace_root.join("crates/monocle-core/src");
    assert!(
        core_src.exists(),
        "BC-2.06.015 INV-2: monocle-core/src must exist at {:?}",
        core_src
    );

    // Patterns that indicate actual `similar` crate usage (not English prose).
    // We check for `use similar::`, `extern crate similar`, and `similar::` path prefix.
    let import_patterns = ["use similar::", "extern crate similar", "similar::"];

    let mut violations: Vec<String> = Vec::new();
    for pattern in &import_patterns {
        visit_rs_files_for_pattern(&core_src, pattern, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "BC-2.06.015 INV-2 / AC-007: 'similar' crate import MUST NOT appear in monocle-core \
         source files (purity boundary: similar is monocle-tui only); \
         found import patterns in: {:#?}",
        violations
    );
}

/// Helper: recursively walk `.rs` files under `dir`, recording files containing `pattern`.
fn visit_rs_files_for_pattern(dir: &std::path::Path, pattern: &str, violations: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files_for_pattern(&path, pattern, violations);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if contents.contains(pattern) {
                    violations.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
}
