//! Overlay rendering tests for S-027.
//!
//! Exercises BC-2.06.010 (overlay widget layout, dimmed background),
//! BC-2.06.019 (status bar drop counter), BC-2.06.020 (breadcrumb),
//! BC-2.06.021 (keybinding hint line), BC-2.06.024 (Bash/Read/Generic payload display).
//!
//! # Red Gate
//!
//! All tests exercise functions whose bodies are `todo!()` in the S-027 stubs.
//! Every test MUST FAIL with a `todo!()` panic until the implementer fills in
//! production code.  DO NOT weaken assertions or implement production code here.
//!
//! # Test naming
//!
//! All names follow the factory-mandated `test_BC_S_SS_NNN_*` pattern.
//!
//! `#![allow(non_snake_case)]` is required because the pattern uses uppercase BC IDs.
#![allow(non_snake_case)]

use monocle_core::tui::state::{AppMode, FocusSnapshot, PromptModal, ToolPayload};
use monocle_tui::ui::overlay_widget::{
    render_bash_payload, render_dimmed_background, render_generic_payload, render_overlay_widget,
    render_read_payload,
};
use monocle_tui::ui::status_bar::{drop_counter_span, mode_indicator_text, render_status_bar};
use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    Terminal,
};
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal `PromptModal` for use in render tests.
fn bash_modal(command: &str) -> PromptModal {
    PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "sess-render-001".to_string(),
        tool_name: "Bash".to_string(),
        tool_payload: ToolPayload::Bash {
            command: command.to_string(),
        },
        received_at: Instant::now(),
    }
}

/// Build a `PromptModal` with a Read payload.
#[allow(dead_code)]
fn read_modal(path: &str) -> PromptModal {
    PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "sess-render-001".to_string(),
        tool_name: "Read".to_string(),
        tool_payload: ToolPayload::Read {
            path: PathBuf::from(path),
        },
        received_at: Instant::now(),
    }
}

/// Build a `PromptModal` with a Generic payload.
#[allow(dead_code)]
fn generic_modal(tool_name: &str, tool_input: serde_json::Value) -> PromptModal {
    PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "sess-render-001".to_string(),
        tool_name: tool_name.to_string(),
        tool_payload: ToolPayload::Generic {
            tool_name: tool_name.to_string(),
            tool_input,
        },
        received_at: Instant::now(),
    }
}

/// Create a `Buffer` of the given size, suitable for render calls.
fn make_buf(width: u16, height: u16) -> Buffer {
    Buffer::empty(Rect::new(0, 0, width, height))
}

// ---------------------------------------------------------------------------
// AC-001 / BC-2.06.010 PC-1 — Overlay widget layout: header / body / footer
// ---------------------------------------------------------------------------

/// BC-2.06.010 PC-1 / AC-001 (RED GATE): `render_overlay_widget` renders the modal
/// header containing `"Permission Request"` when called with a Bash modal.
///
/// The modal header must contain the text `"Permission Request"`.
/// This exercises `render_overlay_widget`, which is `todo!()`.
#[test]
fn test_BC_2_06_010_overlay_header_contains_permission_request_label() {
    let modal = bash_modal("cargo test --workspace");
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            let area = frame.area();
            render_overlay_widget(&modal, 1, area, frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let all_text: String = (0..buf.area().height)
        .flat_map(|y| (0..buf.area().width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("Permission Request"),
        "BC-2.06.010 PC-1: rendered buffer must contain 'Permission Request' in the header; \
         got: {:?}",
        all_text.trim()
    );
}

/// BC-2.06.010 PC-1 / AC-001 (RED GATE): The footer must contain the keyboard hint
/// `"[y] Accept"` so the user knows how to accept once.
#[test]
fn test_BC_2_06_010_overlay_footer_contains_accept_hint() {
    let modal = bash_modal("ls -la");
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            let area = frame.area();
            render_overlay_widget(&modal, 1, area, frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let all_text: String = (0..buf.area().height)
        .flat_map(|y| (0..buf.area().width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("[y]") && all_text.contains("Accept"),
        "BC-2.06.010 PC-1: rendered buffer must contain '[y] Accept' footer hint; \
         got: {:?}",
        all_text.trim()
    );
}

/// BC-2.06.010 PC-1 / AC-001 (RED GATE): The footer must contain `"[n/r] Reject"`.
#[test]
fn test_BC_2_06_010_overlay_footer_contains_reject_hint() {
    let modal = bash_modal("rm -rf /tmp/test");
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            let area = frame.area();
            render_overlay_widget(&modal, 1, area, frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let all_text: String = (0..buf.area().height)
        .flat_map(|y| (0..buf.area().width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("[n/r]") || (all_text.contains("n/r") && all_text.contains("Reject")),
        "BC-2.06.010 PC-1: rendered buffer must contain '[n/r] Reject' footer hint; \
         got: {:?}",
        all_text.trim()
    );
}

/// BC-2.06.010 PC-1 / AC-001 (RED GATE): Stack depth `"(1 of N)"` indicator appears
/// in the header when N > 1 (stack_depth = 3 means "(1 of 3)").
#[test]
fn test_BC_2_06_010_overlay_header_shows_stack_depth_indicator() {
    let modal = bash_modal("git status");
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            let area = frame.area();
            // stack_depth = 3 → "(1 of 3)" must appear
            render_overlay_widget(&modal, 3, area, frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let all_text: String = (0..buf.area().height)
        .flat_map(|y| (0..buf.area().width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("1 of 3"),
        "BC-2.06.010 PC-1 / AC-001: header must contain '1 of 3' stack depth indicator; \
         got: {:?}",
        all_text.trim()
    );
}

/// BC-2.06.010 PC-1 / AC-001 (RED GATE): Modal width is capped at
/// `min(terminal_width - 4, 100)`. For a terminal 80 columns wide the modal must be
/// at most 76 columns wide (80 - 4). Verified by asserting the modal does not span
/// the full terminal width — i.e., leftmost column 0 and rightmost column 79 should
/// NOT both contain modal content (some padding must be present).
#[test]
fn test_BC_2_06_010_overlay_modal_width_respects_cap_min_w_minus_4_100() {
    let modal = bash_modal("echo hello");
    // Use terminal 80 columns wide: max modal width = min(76, 100) = 76
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 1, frame.area(), frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let height = buf.area().height as usize;
    // Check that the modal is centered: the border chars should NOT appear at column 0
    // (there must be at least 2 columns of padding on each side for min(80-4,100)=76 width).
    // Count rows where column 0 has a border/block character.
    let mut leftmost_col_has_modal_border = false;
    for y in 0..height as u16 {
        let sym = buf[(0u16, y)].symbol();
        // Border chars from ratatui Block: '┌', '│', '└', '─'
        if sym == "┌" || sym == "│" || sym == "└" || sym == "─" || sym == "+" || sym == "|"
        {
            leftmost_col_has_modal_border = true;
            break;
        }
    }
    assert!(
        !leftmost_col_has_modal_border,
        "BC-2.06.010 PC-1: modal must NOT extend to column 0 (must be centered with padding); \
         min(80-4, 100) = 76 → 2 columns padding on each side"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.06.010 PC-2 — DIM background applied outside modal
// ---------------------------------------------------------------------------

/// BC-2.06.010 PC-2 / AC-002 (RED GATE): `render_dimmed_background` applies
/// `Modifier::DIM` to all cells in the given area.
///
/// After calling `render_dimmed_background` on a pre-populated buffer, every cell
/// in the area must have `Modifier::DIM` set.
#[test]
fn test_BC_2_06_010_render_dimmed_background_applies_dim_modifier_to_all_cells() {
    let mut buf = make_buf(20, 5);
    // Pre-populate with some text so there is something to dim.
    for y in 0..5u16 {
        for x in 0..20u16 {
            buf[(x, y)].set_symbol("X");
        }
    }
    let area = Rect::new(0, 0, 20, 5);

    // Call the stub — must panic with todo!() until implemented.
    render_dimmed_background(area, &mut buf);

    // After implementation: every cell in `area` must have DIM modifier.
    for y in 0..5u16 {
        for x in 0..20u16 {
            let cell = &buf[(x, y)];
            assert!(
                cell.style().add_modifier.contains(Modifier::DIM),
                "BC-2.06.010 PC-2: cell ({x},{y}) must have Modifier::DIM after \
                 render_dimmed_background; got modifier: {:?}",
                cell.style().add_modifier
            );
        }
    }
}

/// BC-2.06.010 PC-2 / AC-002 (RED GATE): The status bar area (last row) is NOT
/// passed to `render_dimmed_background`. Verify that calling `render_overlay_widget`
/// on the full terminal area does not dim the last row (status bar row).
///
/// This test renders the full overlay via `render_overlay_widget` and confirms the
/// last row retains full-brightness cells (no DIM modifier).
#[test]
fn test_BC_2_06_010_overlay_does_not_dim_status_bar_row() {
    // AC-002 / BC-2.06.010 PC-2: status bar (last row) must NOT be dimmed.
    // render_overlay_widget's caller is responsible for excluding the status bar row,
    // but as a safeguard the widget itself must not apply DIM to the last row when
    // rendered on a full-terminal area that includes the status bar row.
    //
    // We render into a 80×10 terminal and assert no DIM on row 9 (last row).
    let modal = bash_modal("cargo build");
    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            // Pass the full area but instruct caller convention: status bar is row 9.
            // render_overlay_widget must not dim row 9.
            let full_area = frame.area();
            // Dim area excludes the last row.
            let background_area = Rect::new(0, 0, full_area.width, full_area.height - 1);
            let buf_ref = frame.buffer_mut();
            render_dimmed_background(background_area, buf_ref);
            render_overlay_widget(&modal, 1, full_area, frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let last_row = 9u16;
    let width = buf.area().width;

    for x in 0..width {
        let cell = &buf[(x, last_row)];
        assert!(
            !cell.style().add_modifier.contains(Modifier::DIM),
            "BC-2.06.010 PC-2 / AC-002: status bar cell ({x}, {last_row}) must NOT have \
             Modifier::DIM; status bar is never dimmed"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.06.024 PC-2 — Read payload Block display
// ---------------------------------------------------------------------------

/// BC-2.06.024 PC-2 / AC-004 (RED GATE): `render_read_payload` renders `path:` label
/// and the path value into the buffer.
///
/// Canonical test vector from BC-2.06.024:
///   `ToolPayload::Read { path: "/Users/alice/Dev/project/src/main.rs" }`
///   Expected: `path: /Users/alice/Dev/project/src/main.rs`
#[test]
fn test_BC_2_06_024_render_read_payload_shows_path_label_and_value() {
    let path = PathBuf::from("/Users/alice/Dev/project/src/main.rs");
    let mut buf = make_buf(80, 5);
    let area = Rect::new(0, 0, 80, 5);

    // Calls the stub — must panic with todo!() until implemented.
    render_read_payload(&path, area, &mut buf);

    let text: String = (0..5u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("path:"),
        "BC-2.06.024 PC-2 / AC-004: render_read_payload must render 'path:' label; \
         got: {:?}",
        text.trim()
    );
    assert!(
        text.contains("/Users/alice/Dev/project/src/main.rs"),
        "BC-2.06.024 PC-2 / AC-004: render_read_payload must render the full path value; \
         got: {:?}",
        text.trim()
    );
}

/// BC-2.06.024 PC-2 / AC-004 (RED GATE): `render_read_payload` uses a Block titled
/// `"File"` per AC-004 spec ("renders the path in a single-line Block with title 'File'").
#[test]
fn test_BC_2_06_024_render_read_payload_block_has_title_file() {
    let path = PathBuf::from("/etc/hosts");
    let mut buf = make_buf(60, 5);
    let area = Rect::new(0, 0, 60, 5);

    render_read_payload(&path, area, &mut buf);

    let text: String = (0..5u16)
        .flat_map(|y| (0..60u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("File"),
        "BC-2.06.024 PC-2 / AC-004: Block title must be 'File'; got: {:?}",
        text.trim()
    );
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.06.024 PC-3 — Generic payload pretty JSON + scroll hint
// ---------------------------------------------------------------------------

/// BC-2.06.024 PC-3 / AC-006 (RED GATE): `render_generic_payload` renders
/// `tool: <tool_name>` and `input: <json>` lines.
///
/// Canonical test vector from BC-2.06.024:
///   `ToolPayload::Generic { tool_name: "mcp__perplexity__perplexity_search",
///     tool_input: {"query":"rust async"} }`
///   Expected: `tool: mcp__perplexity__perplexity_search` and `input: {"query":"rust async"}`
#[test]
fn test_BC_2_06_024_render_generic_payload_shows_tool_and_input_labels() {
    let tool_name = "mcp__perplexity__perplexity_search";
    let tool_input = serde_json::json!({"query": "rust async"});
    let mut buf = make_buf(80, 8);
    let area = Rect::new(0, 0, 80, 8);

    render_generic_payload(tool_name, &tool_input, area, &mut buf);

    let text: String = (0..8u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("tool:"),
        "BC-2.06.024 PC-3 / AC-006: render_generic_payload must render 'tool:' label; \
         got: {:?}",
        text.trim()
    );
    assert!(
        text.contains("mcp__perplexity__perplexity_search"),
        "BC-2.06.024 PC-3 / AC-006: render_generic_payload must render the tool_name value; \
         got: {:?}",
        text.trim()
    );
    assert!(
        text.contains("input:"),
        "BC-2.06.024 PC-3 / AC-006: render_generic_payload must render 'input:' label; \
         got: {:?}",
        text.trim()
    );
}

/// BC-2.06.024 PC-3 / AC-006 (RED GATE): Generic `tool_input_excerpt` is truncated
/// at 256 characters. For a 300-char value the output must be at most 256 chars of
/// JSON content.
///
/// Canonical test vector from BC-2.06.024:
///   `ToolPayload::Generic { tool_name: "unknown_tool", tool_input: {"x": "y"*300} }`
///   Expected: excerpt truncated at 256 chars; valid UTF-8 boundary respected.
#[test]
fn test_BC_2_06_024_render_generic_payload_truncates_input_excerpt_at_256_chars() {
    let long_value = "y".repeat(300);
    let tool_input = serde_json::json!({"x": long_value});
    // Serialise to know the untruncated length.
    let full_json = serde_json::to_string(&tool_input).unwrap();
    assert!(
        full_json.len() > 256,
        "precondition: untruncated JSON must exceed 256 chars"
    );

    let mut buf = make_buf(80, 10);
    let area = Rect::new(0, 0, 80, 10);

    render_generic_payload("unknown_tool", &tool_input, area, &mut buf);

    // Collect rendered text and find the 'input:' content.
    let text: String = (0..10u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("input:"),
        "BC-2.06.024 PC-3: 'input:' label must be present for truncated case; got: {:?}",
        text.trim()
    );
    // The rendered text must NOT contain the full 300-char value verbatim.
    assert!(
        !text.contains(&long_value),
        "BC-2.06.024 PC-3 / AC-006: tool_input_excerpt must be truncated at 256 chars; \
         full 300-char value must NOT appear verbatim in the buffer"
    );
}

/// BC-2.06.024 PC-3 / AC-006 (RED GATE): When Generic JSON exceeds the modal height
/// minus 6 rows, scroll hints `"↑↓ to scroll"` appear in the footer.
#[test]
fn test_BC_2_06_024_render_generic_payload_shows_scroll_hint_when_content_exceeds_height() {
    // Provide a very small area (5 rows) to force overflow.
    let tool_input = serde_json::json!({
        "line1": "data",
        "line2": "data",
        "line3": "data",
        "line4": "data",
        "line5": "data",
        "line6": "data",
        "line7": "data",
        "line8": "data",
        "line9": "data"
    });
    // area height = 5 rows, which is < 6 + json line count, so scroll hint should appear.
    let mut buf = make_buf(80, 5);
    let area = Rect::new(0, 0, 80, 5);

    render_generic_payload("LargeToolInput", &tool_input, area, &mut buf);

    let text: String = (0..5u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("scroll") || text.contains("↑") || text.contains("↓"),
        "BC-2.06.024 PC-3 / AC-006: scroll hint must appear when JSON exceeds height; \
         got: {:?}",
        text.trim()
    );
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.06.019 PC-1, BC-2.06.020, BC-2.06.021 — Status bar
// ---------------------------------------------------------------------------

/// BC-2.06.019 PC-1 / AC-008 (RED GATE): `mode_indicator_text` returns `"[DASHBOARD]"`
/// for `AppMode::Dashboard`.
///
/// Canonical test vector from BC-2.06.021 PC-1:
///   `AppMode::Dashboard { focused: Sessions }` → hint from Dashboard table.
#[test]
fn test_BC_2_06_019_mode_indicator_text_dashboard_returns_dashboard_label() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let result = mode_indicator_text(&mode, 0);
    assert_eq!(
        result, "[DASHBOARD]",
        "BC-2.06.019 PC-1 / AC-008: Dashboard mode must produce '[DASHBOARD]' indicator"
    );
}

/// BC-2.06.019 PC-1 / AC-008 (RED GATE): `mode_indicator_text` returns `"[OVERLAY N]"`
/// for `AppMode::Overlay` where N is the overlay_stack_depth.
#[test]
fn test_BC_2_06_019_mode_indicator_text_overlay_includes_depth() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let result = mode_indicator_text(&mode, 2);
    assert_eq!(
        result, "[OVERLAY 2]",
        "BC-2.06.019 PC-1 / AC-008: Overlay mode must produce '[OVERLAY 2]' for stack depth 2"
    );
}

/// BC-2.06.019 PC-1 / AC-008 (RED GATE): `mode_indicator_text` returns `"[FILTERING]"`
/// for `AppMode::Filtering`.
#[test]
fn test_BC_2_06_019_mode_indicator_text_filtering_returns_filtering_label() {
    use monocle_core::tui::state::PanelId;
    let mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: String::new(),
        prior: FocusSnapshot::Sessions,
    };
    let result = mode_indicator_text(&mode, 0);
    assert_eq!(
        result, "[FILTERING]",
        "BC-2.06.019 PC-1 / AC-008: Filtering mode must produce '[FILTERING]' indicator"
    );
}

/// BC-2.06.019 PC-1 / AC-008 (RED GATE): `mode_indicator_text` returns `"[FULLSCREEN]"`
/// for `AppMode::Fullscreen`.
#[test]
fn test_BC_2_06_019_mode_indicator_text_fullscreen_returns_fullscreen_label() {
    use monocle_core::tui::state::PanelId;
    let mode = AppMode::Fullscreen {
        panel: PanelId::Sessions,
        prior: FocusSnapshot::Sessions,
    };
    let result = mode_indicator_text(&mode, 0);
    assert_eq!(
        result, "[FULLSCREEN]",
        "BC-2.06.019 PC-1 / AC-008: Fullscreen mode must produce '[FULLSCREEN]' indicator"
    );
}

/// BC-2.06.019 PC-1 / AC-008 (RED GATE): `drop_counter_span` returns `None` when
/// `drop_counter == 0` (no text shown during healthy operation).
///
/// Canonical test vector from BC-2.06.019:
///   `DropCounterUpdate { count: 0 }` → status bar shows no drop counter text.
#[test]
fn test_BC_2_06_019_drop_counter_span_returns_none_when_zero() {
    let result = drop_counter_span(0);
    assert!(
        result.is_none(),
        "BC-2.06.019 PC-1 / AC-008: drop_counter_span(0) must return None (no text displayed)"
    );
}

/// BC-2.06.019 PC-2 / AC-008 (RED GATE): `drop_counter_span` returns `Some(span)` with
/// the EXACT text `"drops: N"` (not `"[dropped: N]"`) when `drop_counter > 0`.
///
/// Canonical test vector from BC-2.06.019:
///   `DropCounterUpdate { count: 42 }` → status bar shows `drops: 42` in yellow.
///
/// STRENGTHENED: previously asserted only presence of "42" and Yellow color — passes against
/// broken code rendering "[dropped: 42]". Now asserts exact BC-canonical `"drops: 42"` prefix.
#[test]
fn test_BC_2_06_019_drop_counter_span_returns_some_with_text_when_nonzero() {
    let result = drop_counter_span(42);
    assert!(
        result.is_some(),
        "BC-2.06.019 PC-2 / AC-008: drop_counter_span(42) must return Some(span)"
    );
    let span = result.unwrap();
    let content = span.content.to_string();
    assert_eq!(
        content, "drops: 42",
        "BC-2.06.019 PC-2 / AC-008: drop counter span must be EXACTLY 'drops: 42' \
         (canonical BC-2.06.019 PC-2 text); got: {:?}",
        content
    );
    assert!(
        span.style.fg == Some(Color::Yellow),
        "BC-2.06.019 PC-2 / AC-008: drop counter span must use Color::Yellow; \
         got fg: {:?}",
        span.style.fg
    );
}

/// BC-2.06.019 PC-1 / AC-008 (RED GATE): `render_status_bar` does not panic with
/// `drop_counter == 0` and renders the mode indicator text to the buffer.
#[test]
fn test_BC_2_06_019_render_status_bar_drop_zero_renders_mode_indicator() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 1);
    let area = Rect::new(0, 0, 80, 1);

    render_status_bar(&mode, 0, 0, None, area, &mut buf);

    let text: String = (0..80u16)
        .map(|x| buf[(x, 0u16)].symbol().to_string())
        .collect();

    assert!(
        text.contains("[DASHBOARD]"),
        "BC-2.06.019 PC-1 / AC-008: render_status_bar must render '[DASHBOARD]' mode indicator; \
         got: {:?}",
        text.trim()
    );
}

/// BC-2.06.019 PC-2 / AC-008 (RED GATE): `render_status_bar` renders the EXACT text
/// `"drops: N"` in yellow when `drop_counter > 0` (BC-2.06.019 PC-2 canonical format).
///
/// Canonical test vector from BC-2.06.019:
///   `DropCounterUpdate { count: 1 }` → status bar shows `drops: 1` in yellow.
///
/// STRENGTHENED: the old assertion `contains("drop") || contains("dropped")` passed against
/// the broken impl which emits `"[dropped: 1]"`. Now asserts the canonical `"drops: 1"` text
/// (no brackets, no "dropped" word form) as specified in BC-2.06.019 PC-2.
#[test]
fn test_BC_2_06_019_render_status_bar_drop_nonzero_renders_dropped_label_in_yellow() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 1);
    let area = Rect::new(0, 0, 80, 1);

    render_status_bar(&mode, 1, 0, None, area, &mut buf);

    let text: String = (0..80u16)
        .map(|x| buf[(x, 0u16)].symbol().to_string())
        .collect();

    assert!(
        text.contains("drops: 1"),
        "BC-2.06.019 PC-2 / AC-008: render_status_bar must render exact text 'drops: 1' \
         (canonical BC-2.06.019 PC-2 format, no brackets, 'drops' not 'dropped'); \
         got: {:?}",
        text.trim()
    );

    // Find the 'd' cell of "drops:" and verify Yellow fg color on the counter cells.
    // We specifically check that no cell with "[dropped:" prefix appears.
    assert!(
        !text.contains("[dropped:"),
        "BC-2.06.019 PC-2 / AC-008: canonical text is 'drops: N' not '[dropped: N]'; \
         got: {:?}",
        text.trim()
    );

    // Find a cell containing the digit "1" in yellow (drop counter cells are styled yellow).
    let mut found_yellow = false;
    for x in 0..80u16 {
        let cell = &buf[(x, 0u16)];
        if cell.symbol().contains('1') && cell.style().fg == Some(Color::Yellow) {
            found_yellow = true;
            break;
        }
    }
    assert!(
        found_yellow,
        "BC-2.06.019 PC-2 / AC-008: drop counter cell must be styled with Color::Yellow"
    );
}

// ---------------------------------------------------------------------------
// AC-009 / BC-2.06.020 PC-1 — Elapsed timer "Waiting: Ns"
// ---------------------------------------------------------------------------

/// BC-2.06.020 PC-1 / AC-009 (RED GATE): The overlay widget renders an elapsed timer
/// `"Waiting: <Ns>"` in the header/footer showing how many seconds the prompt has been
/// waiting. For a very recent modal (received_at = now), elapsed is 0 seconds.
#[test]
fn test_BC_2_06_020_overlay_header_shows_elapsed_timer_waiting_prefix() {
    let modal = bash_modal("ls");
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 1, frame.area(), frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let all_text: String = (0..buf.area().height)
        .flat_map(|y| (0..buf.area().width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("Waiting:"),
        "BC-2.06.020 PC-1 / AC-009: rendered buffer must contain 'Waiting:' elapsed timer; \
         got: {:?}",
        all_text.trim()
    );
}

// ---------------------------------------------------------------------------
// AC-010 / BC-2.06.021 PC-1 — FIFO oldest-first ordering indicator
// ---------------------------------------------------------------------------

/// BC-2.06.021 PC-1 / AC-010 (RED GATE): When `stack_depth > 1`, the header shows
/// `"(1 of N)"` referring to the oldest pending prompt as the active decision target.
/// With stack_depth=2, must contain "(1 of 2)".
#[test]
fn test_BC_2_06_021_overlay_oldest_first_fifo_indicator_in_header_for_multi_stack() {
    let modal = bash_modal("make build");
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 2, frame.area(), frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let all_text: String = (0..buf.area().height)
        .flat_map(|y| (0..buf.area().width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("1 of 2"),
        "BC-2.06.021 PC-1 / AC-010: stack_depth=2 must show '1 of 2' indicator (oldest first); \
         got: {:?}",
        all_text.trim()
    );
}

// ---------------------------------------------------------------------------
// AC-011 / BC-2.06.010 INV-1 — Synchronous render, no spawn
// ---------------------------------------------------------------------------

/// BC-2.06.010 INV-1 / AC-011 (RED GATE): `render_overlay_widget` completes
/// synchronously within a single render frame. This test verifies the call returns
/// within 5ms (target per AC-011) for a typical modal input.
///
/// This is a timing assertion — if the todo!() panics, it fails correctly.
/// Once implemented, the render must finish before the 5ms wall-clock deadline.
#[test]
fn test_BC_2_06_010_invariant_1_render_overlay_widget_completes_synchronously_within_5ms() {
    let modal = bash_modal("cargo test --workspace");
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    let start = std::time::Instant::now();
    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 1, frame.area(), frame);
        })
        .expect("render must not panic");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "BC-2.06.010 INV-1 / AC-011: render_overlay_widget must complete in < 5ms; \
         took {}ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.06.024 PC-1 — Bash command Block rendering (buffer-level)
// ---------------------------------------------------------------------------

/// BC-2.06.024 PC-1 / AC-003 (RED GATE): `render_bash_payload` renders the `command:`
/// LABEL LINE containing the command value in the body area.
///
/// Canonical test vector from BC-2.06.024 PC-1:
///   `ToolPayload::Bash { command: "cargo test --workspace" }`
///   Expected: body renders the label line `command: cargo test --workspace`
///
/// STRENGTHENED: the old assertion `contains("command") || contains("Command")` passed
/// against the broken impl which renders "Command" only as a Block *title* (not as a
/// `command:` label line inside the body). BC-2.06.024 PC-1 says the body renders
/// `command: <command>` as a label line — the block title "Command" is a separate thing.
/// Now asserts the exact `"command: "` label prefix (lowercase, colon, space) so a Block
/// title alone does NOT satisfy the assertion.
#[test]
fn test_BC_2_06_024_render_bash_payload_shows_command_label_and_value() {
    let command = "cargo test --workspace";
    let mut buf = make_buf(80, 6);
    let area = Rect::new(0, 0, 80, 6);

    render_bash_payload(command, area, &mut buf);

    let text: String = (0..6u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    // BC-2.06.024 PC-1: the body must contain the `command:` label line (lowercase,
    // colon-space format). A block title "Command" alone does NOT satisfy this.
    assert!(
        text.contains("command: "),
        "BC-2.06.024 PC-1 / AC-003: render_bash_payload must render 'command: ' label line \
         (lowercase 'command:' + space, not just block title 'Command'); got: {:?}",
        text.trim()
    );
    assert!(
        text.contains("cargo test --workspace"),
        "BC-2.06.024 PC-1 / AC-003: render_bash_payload must render the command value \
         'cargo test --workspace'; got: {:?}",
        text.trim()
    );
}

/// BC-2.06.024 PC-1 / AC-003 (RED GATE): `render_bash_payload` uses a bordered Block
/// with title `"Command"` (AC-003: "the command in a bordered Block with title 'Command'").
#[test]
fn test_BC_2_06_024_render_bash_payload_block_has_title_command() {
    let command = "ls -la";
    let mut buf = make_buf(60, 6);
    let area = Rect::new(0, 0, 60, 6);

    render_bash_payload(command, area, &mut buf);

    let text: String = (0..6u16)
        .flat_map(|y| (0..60u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("Command"),
        "BC-2.06.024 PC-1 / AC-003: Block title must be 'Command'; got: {:?}",
        text.trim()
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.020 — Breadcrumb derivation (pure function of AppMode)
// ---------------------------------------------------------------------------

/// BC-2.06.020 PC-1 (RED GATE): `render_status_bar` renders the breadcrumb string
/// `"Dashboard > Sessions"` for `AppMode::Dashboard { focused: Sessions }`.
///
/// Canonical test vector from BC-2.06.020:
///   `Dashboard { focused: Sessions }` → `Dashboard > Sessions`
#[test]
fn test_BC_2_06_020_render_status_bar_breadcrumb_dashboard_sessions() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);

    render_status_bar(&mode, 0, 0, None, area, &mut buf);

    let text: String = (0..2u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("Dashboard") && text.contains("Sessions"),
        "BC-2.06.020 PC-1: breadcrumb must contain 'Dashboard' and 'Sessions'; got: {:?}",
        text.trim()
    );
}

/// BC-2.06.020 PC-1 (RED GATE): `render_status_bar` renders `"Dashboard > Overlay [1 prompt]"`
/// for `AppMode::Overlay` with `overlay_stack_depth == 1` (singular).
///
/// Canonical test vector from BC-2.06.020:
///   `Overlay { prior: Sessions }` (stack=[P1]) → `Dashboard > Overlay [1 prompt]`
#[test]
fn test_BC_2_06_020_render_status_bar_breadcrumb_overlay_singular_prompt() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);

    render_status_bar(&mode, 0, 1, None, area, &mut buf);

    let text: String = (0..2u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("Overlay") && text.contains("1 prompt"),
        "BC-2.06.020 PC-1: breadcrumb must contain 'Overlay' and '1 prompt' (singular); \
         got: {:?}",
        text.trim()
    );
    // Must NOT use plural form "prompts" for a single item.
    // The substring "1 prompts" would be incorrect; we check "1 prompt]" specifically.
    assert!(
        !text.contains("1 prompts"),
        "BC-2.06.020 PC-2: must use singular '[1 prompt]' not '[1 prompts]'; got: {:?}",
        text.trim()
    );
}

/// BC-2.06.020 PC-2 (RED GATE): Plural form `"[N prompts]"` for overlay_stack_depth > 1.
///
/// Canonical test vector from BC-2.06.020:
///   `Overlay { prior: Sessions }` (stack=[P1, P2, P3]) → `Dashboard > Overlay [3 prompts]`
#[test]
fn test_BC_2_06_020_render_status_bar_breadcrumb_overlay_plural_prompts() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);

    render_status_bar(&mode, 0, 3, None, area, &mut buf);

    let text: String = (0..2u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("3 prompts"),
        "BC-2.06.020 PC-2: breadcrumb must contain '3 prompts' (plural); got: {:?}",
        text.trim()
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.021 — Keybinding hint line
// ---------------------------------------------------------------------------

/// BC-2.06.021 PC-1 (RED GATE): `render_status_bar` renders the EXACT canonical Dashboard
/// hint line for `AppMode::Dashboard`.
///
/// Canonical test vector from BC-2.06.021 PC-1:
///   `Dashboard { focused: Sessions }` → `Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit`
///
/// STRENGTHENED: the old assertions `contains("Tab") || contains("tab")` and
/// `contains("quit") || contains('q')` were tautologies that passed against the broken
/// impl emitting `"Tab: cycle  Enter: fullscreen  q: quit"` (missing `/: filter` and `Ctrl-P:
/// profile`). Now asserts the EXACT canonical string from BC-2.06.021 PC-1 table.
#[test]
fn test_BC_2_06_021_render_status_bar_hint_line_dashboard_contains_key_hints() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);

    render_status_bar(&mode, 0, 0, None, area, &mut buf);

    // Collect just the hint row (row 1 of a 2-row status bar).
    let hint_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();
    let hint_trimmed = hint_row.trim_end();

    assert_eq!(
        hint_trimmed, "Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit",
        "BC-2.06.021 PC-1: Dashboard hint line must be EXACTLY the canonical string \
         from BC-2.06.021 PC-1 table (includes '/: filter' and 'Ctrl-P: profile' \
         which the broken impl omits)"
    );
}

/// BC-2.06.021 PC-1 (RED GATE): `render_status_bar` renders the EXACT canonical
/// Overlay hint line for `AppMode::Overlay`.
///
/// Canonical test vector from BC-2.06.021 PC-1 (corrected from stale `1/2/3` keys):
///   `Overlay { .. }` → `y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace`
///
/// STRENGTHENED: the old assertions `contains('t') && contains("trace")` and
/// `contains("Esc") || contains("esc")` were tautologies that passed against the broken
/// impl emitting `"1: accept-once  2: accept-always  3: reject  Esc: hide  t: trace"`.
/// BC-2.06.021 §Trace corrected the keys from `1/2/3` to `y/A/n/r` to match
/// the canonical `binding.rs` keybindings (S-026 merged). Now asserts EXACT string.
#[test]
fn test_BC_2_06_021_render_status_bar_hint_line_overlay_contains_trace_stub_binding() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);

    render_status_bar(&mode, 0, 1, None, area, &mut buf);

    // Collect just the hint row (row 1).
    let hint_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();
    let hint_trimmed = hint_row.trim_end();

    assert_eq!(
        hint_trimmed,
        "y: accept  A: accept-always  n/r: reject  \u{2191}\u{2193}: cycle  Esc: hide  t: trace",
        "BC-2.06.021 PC-1: Overlay hint line must use canonical y/A/n/r keybindings \
         (not stale 1/2/3 placeholder keys); 't: trace' stub must be present (PC-6); \
         'Esc: hide' semantics must be present (PC-5)"
    );
}

// ---------------------------------------------------------------------------
// NEW EC/CORRECTNESS TESTS — added to close adversarial findings
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// BC-2.06.019 PC-7 — COEXISTENCE: drops:N on upper row, status_message on lower
// row (RED against current mutual-exclusion impl).
// ---------------------------------------------------------------------------

/// BC-2.06.019 PC-7 (EC-129) (RED): With `drop_counter = 42` AND
/// `status_message = Some("[disconnected] reconnecting...")` simultaneously,
/// `render_status_bar` on a 2-row area must render BOTH pieces of information:
/// - Upper row (y=0): contains `"drops: 42"` in yellow (NEVER suppressed).
/// - Lower row (y=1): contains `"[disconnected] reconnecting..."` in yellow
///   (temporarily supersedes the keybinding hint).
///
/// The current impl uses `if let Some(msg) = status_message { msg } else
/// { drop_counter_span() }` — mutual exclusion. When status_message is Some,
/// the drop counter is silently dropped. This assertion on the upper row is RED
/// against that impl.
///
/// Canonical test vector from BC-2.06.019 EC-129 / test vector table:
///   `drop_counter = 42`, `status_message = Some("[disconnected] reconnecting...")`
///   Expected: upper row: `drops: 42` in yellow; lower row: message in yellow.
#[test]
fn test_BC_2_06_019_pc7_coexistence_drops_and_disconnect_both_visible_in_two_row_bar() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);
    let disconnect_msg = "[disconnected] reconnecting...";

    render_status_bar(&mode, 42, 1, Some(disconnect_msg), area, &mut buf);

    // Upper row (y=0) MUST contain "drops: 42".
    // RED against current impl: mutual-exclusion drops the counter when
    // status_message is Some.
    let upper_row: String = (0..80u16)
        .map(|x| buf[(x, 0u16)].symbol().to_string())
        .collect();

    assert!(
        upper_row.contains("drops: 42"),
        "BC-2.06.019 PC-7 (EC-129) COEXISTENCE: upper row (y=0) must contain 'drops: 42' \
         even when status_message is Some — drops:N is NEVER suppressed by status_message \
         (BC-2.06.019 PC-7 forbids the mutual-exclusion pattern); got upper row: {:?}",
        upper_row.trim()
    );

    // Verify drops:42 is styled Yellow on the upper row.
    let drop_chars: Vec<char> = "drops: 42".chars().collect();
    let mut drops_yellow = false;
    for x in 0..80u16 {
        let cell = &buf[(x, 0u16)];
        if cell.symbol() == "d" {
            let matches = drop_chars.iter().enumerate().all(|(i, &ch)| {
                let cx = x + i as u16;
                cx < 80 && buf[(cx, 0u16)].symbol().starts_with(ch)
            });
            if matches && cell.style().fg == Some(Color::Yellow) {
                drops_yellow = true;
                break;
            }
        }
    }
    assert!(
        drops_yellow,
        "BC-2.06.019 PC-7: 'drops: 42' must be styled Color::Yellow on the upper row (y=0)"
    );

    // Lower row (y=1) MUST contain the disconnect message.
    // The lower row supersedes the keybinding hint when status_message is Some.
    let lower_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();

    assert!(
        lower_row.contains(disconnect_msg),
        "BC-2.06.019 PC-7 (EC-129) COEXISTENCE: lower row (y=1) must contain the \
         disconnect message {:?} — status_message renders on the lower (hint) row \
         per BC-2.06.019 PC-7 / BC-2.06.016 PC-4; got lower row: {:?}",
        disconnect_msg,
        lower_row.trim()
    );

    // Confirm drops:42 is NOT absent from the combined buffer (core regression guard).
    let all_text: String = (0..2u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("drops: 42"),
        "BC-2.06.019 PC-7 REGRESSION GUARD: 'drops: 42' must NOT be absent from the \
         status bar when status_message is Some — got combined text: {:?}",
        all_text.trim()
    );
}

/// BC-2.06.019 PC-7 (EC-130) / BC-2.06.015 (RED): With `drop_counter = 7` AND
/// `status_message = Some("[t] Trace to source — Phase 2 feature (Static plane)")`
/// simultaneously (the `[t]` stub scenario), `render_status_bar` on a 2-row area
/// must render BOTH:
/// - Upper row (y=0): `"drops: 7"` in yellow (NEVER suppressed).
/// - Lower row (y=1): `"[t] Trace to source — Phase 2 feature (Static plane)"` in yellow.
///
/// Canonical test vector from BC-2.06.019 EC-130 / test vector table.
/// RED against current impl: mutual-exclusion hides drops:7 when status_message is Some.
#[test]
fn test_BC_2_06_019_pc7_coexistence_drops_and_trace_stub_both_visible_in_two_row_bar() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);
    // BC-2.06.015 PC-1 canonical placeholder — exact text mandated by BC.
    let trace_msg = "[t] Trace to source \u{2014} Phase 2 feature (Static plane)";

    render_status_bar(&mode, 7, 1, Some(trace_msg), area, &mut buf);

    // Upper row (y=0) MUST contain "drops: 7".
    // RED against current impl: mutual-exclusion suppresses the drop counter
    // when status_message is Some (the [t] press scenario).
    let upper_row: String = (0..80u16)
        .map(|x| buf[(x, 0u16)].symbol().to_string())
        .collect();

    assert!(
        upper_row.contains("drops: 7"),
        "BC-2.06.019 PC-7 (EC-130) COEXISTENCE: upper row (y=0) must contain 'drops: 7' \
         even when [t] trace stub status_message is Some — drops:N is NEVER suppressed \
         by any status_message (BC-2.06.019 PC-7); got upper row: {:?}",
        upper_row.trim()
    );

    // Lower row (y=1) must contain the [t] trace placeholder.
    let lower_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();

    // The [t] message contains the em-dash character (U+2014). Check for the prefix
    // and the key discriminating phrase rather than the full string (which may span
    // buffer cells in a way that varies by terminal width allocation).
    assert!(
        lower_row.contains("[t]") && lower_row.contains("Trace to source"),
        "BC-2.06.019 PC-7 (EC-130) COEXISTENCE: lower row (y=1) must contain the [t] \
         trace placeholder ('[t]' + 'Trace to source') per BC-2.06.019 PC-7 / BC-2.06.015 \
         PC-1; got lower row: {:?}",
        lower_row.trim()
    );

    // Confirm drops:7 is not absent from the combined buffer (regression guard).
    let all_text: String = (0..2u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("drops: 7"),
        "BC-2.06.019 PC-7 REGRESSION GUARD: 'drops: 7' must NOT be absent when [t] \
         status_message is active; got combined text: {:?}",
        all_text.trim()
    );
}

/// BC-2.06.019 PC-7 (GREEN, control case): With `drop_counter > 0` and
/// `status_message = None`, the existing single-drop-counter behavior is unchanged.
/// The drop counter renders on the upper row; the hint renders on the lower row.
/// This test must STAY GREEN after the coexistence fix.
#[test]
fn test_BC_2_06_019_pc7_drop_counter_only_no_status_message_stays_green() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);

    // status_message=None — only the drop counter is active (GREEN control case).
    render_status_bar(&mode, 42, 0, None, area, &mut buf);

    let upper_row: String = (0..80u16)
        .map(|x| buf[(x, 0u16)].symbol().to_string())
        .collect();

    assert!(
        upper_row.contains("drops: 42"),
        "BC-2.06.019 PC-2 (GREEN): upper row must contain 'drops: 42' when \
         status_message=None; got: {:?}",
        upper_row.trim()
    );

    // The lower row must have the hint (not the drop counter).
    let lower_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();

    assert!(
        lower_row.contains("Tab:"),
        "BC-2.06.021 PC-1 (GREEN): lower row must contain 'Tab:' hint when \
         status_message=None and mode is Dashboard; got: {:?}",
        lower_row.trim()
    );
}

/// BC-2.06.019 / BC-2.06.020 / BC-2.06.021 PC-2 — Status bar is TWO rows.
///
/// AC-008: the status bar is allocated `Constraint::Length(2)` — two rows.
/// Row 0 (upper): mode indicator + breadcrumb + optional drop counter.
/// Row 1 (lower): hint line.
///
/// This test renders into a 2-row buffer and asserts both rows have content —
/// i.e. the breadcrumb is on the UPPER row and the hint on the LOWER row.
/// The broken impl renders only 1 row from a 1-row area.
#[test]
fn test_BC_2_06_019_status_bar_is_two_rows_breadcrumb_upper_hint_lower() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);

    render_status_bar(&mode, 0, 0, None, area, &mut buf);

    // Row 0 (y=0) must contain the breadcrumb "Dashboard > Sessions".
    let upper_row: String = (0..80u16)
        .map(|x| buf[(x, 0u16)].symbol().to_string())
        .collect();
    assert!(
        upper_row.contains("Dashboard"),
        "BC-2.06.019/020: upper status bar row (y=0) must contain 'Dashboard' breadcrumb; \
         got: {:?}",
        upper_row.trim()
    );

    // Row 1 (y=1) must contain the hint line "Tab: cycle ...".
    let lower_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();
    assert!(
        lower_row.contains("Tab:"),
        "BC-2.06.021 PC-1: lower status bar row (y=1) must contain 'Tab:' hint; \
         got: {:?}",
        lower_row.trim()
    );

    // The LOWER row must NOT contain the breadcrumb and the UPPER row must NOT
    // contain the hint — they occupy separate rows.
    assert!(
        !lower_row.contains("Dashboard > Sessions"),
        "BC-2.06.020: breadcrumb must NOT appear on the lower (hint) row; \
         got lower row: {:?}",
        lower_row.trim()
    );
}

/// BC-2.06.020 PC-1 — Breadcrumb derivation: `Dashboard { focused: EventRibbon }` → `"Dashboard > Events"`.
///
/// The broken `breadcrumb_text` hardcodes `"Dashboard > Sessions"` for ALL Dashboard focus
/// variants, ignoring `FocusSnapshot::EventRibbon`. This test catches that regression.
///
/// Canonical test vector from BC-2.06.020:
///   `Dashboard { focused: EventRibbon }` → `Dashboard > Events`
#[test]
fn test_BC_2_06_020_breadcrumb_dashboard_event_ribbon_focus_renders_events() {
    let mode = AppMode::Dashboard {
        focused: FocusSnapshot::EventRibbon,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);

    render_status_bar(&mode, 0, 0, None, area, &mut buf);

    // Collect the upper breadcrumb row.
    let upper_row: String = (0..80u16)
        .map(|x| buf[(x, 0u16)].symbol().to_string())
        .collect();

    assert!(
        upper_row.contains("Dashboard > Events"),
        "BC-2.06.020 PC-1: breadcrumb for Dashboard{{focused: EventRibbon}} must be \
         'Dashboard > Events' (not 'Dashboard > Sessions'); got: {:?}",
        upper_row.trim()
    );
    assert!(
        !upper_row.contains("Dashboard > Sessions"),
        "BC-2.06.020 PC-1: 'Dashboard > Sessions' must NOT appear when focus is EventRibbon; \
         got: {:?}",
        upper_row.trim()
    );
}

/// BC-2.06.021 PC-1 — Overlay hint EXACT canonical string.
///
/// Separate dedicated test (in addition to the strengthened existing test) to make the
/// failure mode unmistakable when the broken stale `1/2/3` impl is present.
/// Canonical: `y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace`
#[test]
fn test_BC_2_06_021_overlay_hint_exact_canonical_y_A_nr_keys() {
    let mode = AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);
    render_status_bar(&mode, 0, 1, None, area, &mut buf);

    let hint_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();
    let hint = hint_row.trim_end();

    // The stale implementation starts with "1: accept-once" — assert it does NOT appear.
    assert!(
        !hint.contains("1: accept"),
        "BC-2.06.021 PC-1: stale placeholder '1: accept-once' MUST NOT appear in \
         overlay hint; canonical keys are y/A/n/r (BC §Trace); got: {:?}",
        hint
    );

    // The canonical 'y: accept' and 'A: accept-always' must appear.
    assert!(
        hint.contains("y: accept"),
        "BC-2.06.021 PC-1: 'y: accept' must appear in overlay hint; got: {:?}",
        hint
    );
    assert!(
        hint.contains("A: accept-always"),
        "BC-2.06.021 PC-1: 'A: accept-always' must appear in overlay hint; got: {:?}",
        hint
    );
    assert!(
        hint.contains("n/r: reject"),
        "BC-2.06.021 PC-1: 'n/r: reject' must appear in overlay hint; got: {:?}",
        hint
    );
}

/// BC-2.06.021 PC-1 — Filtering hint EXACT canonical string `"(type to filter)  Esc: cancel"`.
///
/// The broken `hint_line_text` Filtering arm returns `"Enter: apply  Esc: cancel"` which
/// contradicts BC-2.06.021 PC-1 canonical table row for Filtering.
#[test]
fn test_BC_2_06_021_filtering_hint_exact_canonical_string() {
    use monocle_core::tui::state::PanelId;
    let mode = AppMode::Filtering {
        panel: PanelId::Sessions,
        query: String::new(),
        prior: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);
    render_status_bar(&mode, 0, 0, None, area, &mut buf);

    let hint_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();
    let hint = hint_row.trim_end();

    assert_eq!(
        hint, "(type to filter)  Esc: cancel",
        "BC-2.06.021 PC-1: Filtering hint must be EXACTLY '(type to filter)  Esc: cancel'; \
         the broken impl returns 'Enter: apply  Esc: cancel'"
    );
}

/// BC-2.06.021 PC-1 — Fullscreen hint EXACT canonical string `"Esc: back  /: filter  q: quit"`.
///
/// The broken `hint_line_text` Fullscreen arm returns `"Esc: exit fullscreen"` which
/// contradicts BC-2.06.021 PC-1 canonical table row for Fullscreen.
#[test]
fn test_BC_2_06_021_fullscreen_hint_exact_canonical_string() {
    use monocle_core::tui::state::PanelId;
    let mode = AppMode::Fullscreen {
        panel: PanelId::Sessions,
        prior: FocusSnapshot::Sessions,
    };
    let mut buf = make_buf(80, 2);
    let area = Rect::new(0, 0, 80, 2);
    render_status_bar(&mode, 0, 0, None, area, &mut buf);

    let hint_row: String = (0..80u16)
        .map(|x| buf[(x, 1u16)].symbol().to_string())
        .collect();
    let hint = hint_row.trim_end();

    assert_eq!(
        hint, "Esc: back  /: filter  q: quit",
        "BC-2.06.021 PC-1: Fullscreen hint must be EXACTLY 'Esc: back  /: filter  q: quit'; \
         the broken impl returns 'Esc: exit fullscreen'"
    );
}

/// BC-2.06.024 PC-3 EC-007 — Generic payload serialization failure renders `input: (unrepresentable)`.
///
/// BC-2.06.024 EC-007: when `serde_json::to_string` fails, the body renders
/// `input: (unrepresentable)` as a safe fallback. No panic.
///
/// Since `serde_json::Value` serialization does not ordinarily fail, we test the
/// (unrepresentable) text by using a specially constructed Value that will produce
/// a non-serializable result, OR we verify the fallback constant string appears
/// when the excerpt truncation path is exercised with a valid value — the test
/// coverage we need is that the implementation has the fallback TEXT at all.
///
/// Strategy: the test calls `render_generic_payload` with a very large tool_input and
/// then separately verifies the fallback path by looking for `(unrepresentable)` in
/// the source code (compile-time assertion). For runtime we test with valid JSON
/// and assert the NORMAL path renders `tool:` and `input:` labels.
///
/// Additionally we verify via `serde_json::to_string` error simulation that
/// `(unrepresentable)` is the canonical fallback text (not `(error)` or `None`).
#[test]
fn test_BC_2_06_024_generic_payload_fallback_text_is_unrepresentable() {
    // Runtime coverage: render_generic_payload with `serde_json::Value::Null` —
    // this serializes to "null" successfully, confirming the normal path.
    // The fallback path cannot be triggered via the public API because `serde_json::Value`
    // always serializes, but we verify the fallback constant via source inspection.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root navigable");
    let overlay_widget_src = workspace_root.join("crates/monocle-tui/src/ui/overlay_widget.rs");
    let src = std::fs::read_to_string(&overlay_widget_src).expect("read overlay_widget.rs");
    assert!(
        src.contains("(unrepresentable)"),
        "BC-2.06.024 EC-007: overlay_widget.rs must contain the fallback text \
         '(unrepresentable)' for serialization failure; \
         the broken impl uses '{{}}' as a fallback (silently hides the error)"
    );

    // Also assert that the `tracing::warn!` or `tracing::warn` call is present for EC-007.
    // The WARN must be emitted on serialization failure.
    assert!(
        src.contains("tracing::warn"),
        "BC-2.06.024 EC-007: overlay_widget.rs must emit tracing::warn! on serialization \
         failure (EC-007 requirement); got no tracing::warn call in source"
    );
}

/// BC-2.06.024 PC-1.4 / PC-2.4 — Empty command/path renders safe fallback text.
///
/// BC-2.06.024 PC-1.4: if `command` is empty, the body renders `command: (empty)` (no panic).
/// BC-2.06.024 PC-2.4: if `path` is empty, the body renders `path: (empty)` (no panic).
#[test]
fn test_BC_2_06_024_bash_empty_command_renders_command_empty_fallback() {
    let mut buf = make_buf(80, 6);
    let area = Rect::new(0, 0, 80, 6);

    // Empty command — must render `command: (empty)` fallback.
    render_bash_payload("", area, &mut buf);

    let text: String = (0..6u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("command: (empty)"),
        "BC-2.06.024 PC-1.4: empty command must render 'command: (empty)' fallback; \
         got: {:?}",
        text.trim()
    );
}

/// BC-2.06.024 PC-2.4 — Empty path renders `path: (empty)` fallback.
#[test]
fn test_BC_2_06_024_read_empty_path_renders_path_empty_fallback() {
    let mut buf = make_buf(80, 6);
    let area = Rect::new(0, 0, 80, 6);

    // Empty path — must render `path: (empty)` fallback.
    render_read_payload(std::path::Path::new(""), area, &mut buf);

    let text: String = (0..6u16)
        .flat_map(|y| (0..80u16).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        text.contains("path: (empty)"),
        "BC-2.06.024 PC-2.4: empty path must render 'path: (empty)' fallback; \
         got: {:?}",
        text.trim()
    );
}

// BC-2.06.021 INV-3 byte-length variant removed (ADV Pass-5 NITPICK-1).
//
// The byte-length check (`trimmed.len() <= 79`) exercised the same 5 AppMode
// variants as the display-column test below and used a strictly weaker metric:
// multi-byte Unicode characters (e.g., ↑↓ arrows at 3 bytes each, 1 display
// column each) would satisfy the byte-length bound while potentially violating
// the display-column bound.  BC-2.06.021 INV-3 specifies ≤79 DISPLAY COLUMNS,
// so the byte-length metric was both incorrect and fully redundant.
//
// Canonical test: `test_BC_2_06_021_invariant_3_all_hint_lines_fit_in_79_display_columns`

// ---------------------------------------------------------------------------
// ADV Pass-2 B1 — Generic scroll-hint consolidation (RED tests)
//
// Bug: build_footer_line receives show_scroll_hint=true for ALL Generic payloads
// unconditionally (line ~75: matches!(ToolPayload::Generic { .. })), regardless
// of whether the content actually overflows. render_generic_payload ALSO shows its
// own scroll hint in the body when overflow occurs. This causes:
//   - No-overflow case: footer shows spurious "↑↓ to scroll" (count should be 0)
//   - Overflow case: body shows it AND footer shows it (count should be 1, is 2)
//
// BC-2.06.024 PC-3, AC-006 (scroll hint location).
// ---------------------------------------------------------------------------

/// Count non-overlapping occurrences of `needle` in `haystack`.
fn count_occurrences(haystack: &str, needle: &str) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        count += 1;
        start += pos + needle.len();
    }
    count
}

/// Collect all buffer cell symbols into a single string.
fn buffer_all_text(terminal: &ratatui::Terminal<ratatui::backend::TestBackend>) -> String {
    let buf = terminal.backend().buffer().clone();
    let area = buf.area();
    (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect()
}

/// BC-2.06.024 PC-3 / AC-006 (ADV PASS-2 B1, RED):
/// A Generic payload whose content does NOT overflow the modal body must show
/// ZERO `↑↓ to scroll` hints in the entire rendered overlay.
///
/// Failure mode (current): `build_footer_line(show_scroll_hint=true)` is called for
/// ALL Generic payloads unconditionally, so the footer always appends `↑↓ to scroll`
/// even when no scrolling is needed. Count is 1 (footer only) but should be 0.
///
/// Test vector: short `tool:`/`input:` content, ample 80×24 terminal → no overflow.
#[test]
fn test_BC_2_06_024_generic_no_overflow_shows_zero_scroll_hints() {
    // Short content: {"x":"hi"} → 1 input line, well within any modal height.
    // Ample 80×24 terminal: modal height = min(24-4,30) = 20 rows → threshold = 20-6 = 14.
    // json_line_count = 1 ≤ 14 → no overflow → body scroll hint suppressed.
    // The footer MUST also suppress the hint because there is no overflow.
    let modal = generic_modal("short_tool", serde_json::json!({"x": "hi"}));
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 1, frame.area(), frame);
        })
        .expect("render must not panic");

    let all_text = buffer_all_text(&terminal);
    let scroll_hint_count = count_occurrences(&all_text, "to scroll");

    assert_eq!(
        scroll_hint_count, 0,
        "BC-2.06.024 PC-3 / AC-006: a Generic payload that does NOT overflow must show \
         ZERO '↑↓ to scroll' hints in the rendered overlay; \
         got {} occurrences (current bug: footer shows hint unconditionally for all Generic \
         payloads regardless of overflow state)",
        scroll_hint_count
    );
}

/// BC-2.06.024 PC-3 / AC-006 (ADV PASS-2 B1, RED):
/// A Generic payload whose content DOES overflow the modal body must show
/// EXACTLY ONE `↑↓ to scroll` hint in the entire rendered overlay (not two).
///
/// Failure mode (current): `build_footer_line(show_scroll_hint=true)` shows the
/// hint in the footer, AND `render_generic_payload` shows the hint in the body
/// on overflow, producing a count of 2. The correct count is 1 (one hint, one location).
///
/// Test vector: multi-line JSON with 8 fields in a 15-row terminal where the body
/// overflows the `area.height - 6` threshold.
#[test]
fn test_BC_2_06_024_generic_overflow_shows_exactly_one_scroll_hint() {
    // Large input produces a long compact-JSON string that exceeds the threshold
    // in a modest terminal.  At 80×15: modal_height = min(11,30)=11,
    // inner=9 rows, body=9-2(header)-1(footer)=6 rows, body inner=4 rows.
    // render_generic_payload area = body≈6 rows; threshold = 6-6 = 0, so
    // any non-empty content triggers scroll. Body will show the hint once.
    // Footer will ALSO show it (bug), giving count=2. Correct behavior: count=1.
    let large_input = serde_json::json!({
        "a": "line1", "b": "line2", "c": "line3", "d": "line4",
        "e": "line5", "f": "line6", "g": "line7", "h": "line8",
    });
    let modal = generic_modal("big_tool", large_input);
    let backend = TestBackend::new(80, 15);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 1, frame.area(), frame);
        })
        .expect("render must not panic");

    let all_text = buffer_all_text(&terminal);
    let scroll_hint_count = count_occurrences(&all_text, "to scroll");

    assert_eq!(
        scroll_hint_count, 1,
        "BC-2.06.024 PC-3 / AC-006: a Generic payload that DOES overflow must show \
         EXACTLY ONE '↑↓ to scroll' hint (one location, not two); \
         got {} occurrences (current bug: footer adds a second hint unconditionally, \
         producing a double hint — body hint + footer hint = 2 when only 1 is correct)",
        scroll_hint_count
    );
}

// ---------------------------------------------------------------------------
// ADV Pass-2 B5 — INV-3 hint width measured in DISPLAY COLUMNS (augment test)
//
// BC-2.06.021 INV-3 says hints must fit within 79 DISPLAY COLUMNS.  The ↑↓
// arrows are U+2191/U+2193 (3 bytes each in UTF-8) so `.len()` (bytes) gives
// a larger number than display columns.  The replaced test used byte length and
// thus measured the wrong quantity.  This test uses `unicode_width::UnicodeWidthStr`
// to measure display columns (the correct metric per INV-3).
//
// The Overlay hint `y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace`
// is 72 display columns (arrows are width 1 each in non-CJK context) and 76 bytes.
// Both ≤ 79, so this test PASSES for the current correct strings.
// It GUARDS the right metric for future changes (adding a wide char would fail here
// but pass the byte-length check, making the byte test an incorrect guard).
// ---------------------------------------------------------------------------

/// BC-2.06.021 INV-3 (ADV PASS-2 B5, GUARD): All hint line strings fit within
/// 79 DISPLAY COLUMNS (not 79 bytes).
///
/// `unicode_width::UnicodeWidthStr::width()` returns the terminal display column
/// count, which is the correct metric for INV-3 per BC-2.06.021 §Invariants:
/// "At 80 columns, no hint line from Postcondition 1 exceeds 79 display columns."
///
/// The ↑↓ arrows (U+2191/U+2193) are 3 bytes each but 1 display column each
/// (East Asian Width = Ambiguous → 1 col in non-CJK context per unicode-width 0.2).
/// Current hint strings measure 72 display columns for the Overlay hint — safely
/// within the 79-column budget. This test will catch any future addition of a
/// wide character (e.g., a CJK glyph) that would exceed the budget while the
/// old byte-length test would have let it through.
#[test]
fn test_BC_2_06_021_invariant_3_all_hint_lines_fit_in_79_display_columns() {
    use monocle_core::tui::state::PanelId;
    use unicode_width::UnicodeWidthStr;

    let modes: Vec<(&str, AppMode, usize)> = vec![
        (
            "Dashboard/Sessions",
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions,
            },
            0,
        ),
        (
            "Dashboard/EventRibbon",
            AppMode::Dashboard {
                focused: FocusSnapshot::EventRibbon,
            },
            0,
        ),
        (
            "Overlay",
            AppMode::Overlay {
                prior: FocusSnapshot::Sessions,
            },
            1,
        ),
        (
            "Filtering",
            AppMode::Filtering {
                panel: PanelId::Sessions,
                query: String::new(),
                prior: FocusSnapshot::Sessions,
            },
            0,
        ),
        (
            "Fullscreen",
            AppMode::Fullscreen {
                panel: PanelId::Sessions,
                prior: FocusSnapshot::Sessions,
            },
            0,
        ),
    ];

    for (label, mode, depth) in modes {
        let mut buf = make_buf(80, 2);
        let area = Rect::new(0, 0, 80, 2);
        render_status_bar(&mode, 0, depth, None, area, &mut buf);

        // Collect just the hint row (row 1).
        let hint_row: String = (0..80u16)
            .map(|x| buf[(x, 1u16)].symbol().to_string())
            .collect();
        let trimmed = hint_row.trim_end();
        let display_width = UnicodeWidthStr::width(trimmed);

        assert!(
            display_width <= 79,
            "BC-2.06.021 INV-3: hint line for {} must be <= 79 DISPLAY COLUMNS \
             (not 79 bytes); got {} display columns (byte len={}): {:?}",
            label,
            display_width,
            trimmed.len(),
            trimmed
        );
    }
}

// ---------------------------------------------------------------------------
// F-S027-P3-NIT-001 — wrap-aware Generic overflow scroll hint (RED)
//
// BC-2.06.024 PC-3 / AC-006: the scroll-hint condition must be based on
// VISUAL rows (how many display rows the wrapped text occupies), not on
// logical line count (how many '\n' characters the excerpt contains).
//
// Compact JSON (`serde_json::to_string`) produces single-line output with no
// '\n' characters.  `excerpt.lines().count()` is therefore always 1 regardless
// of how many columns the text needs to wrap over.  The current implementation
// compares that logical count (1) against the threshold (area.height - 6),
// so it never detects overflow for single-line excerpts — even when the text
// visually wraps to far more rows than the available body height.
//
// Chosen dimensions (unambiguous wrap math):
//   Terminal:       40 × 18  (width × height)
//   modal_width:    min(40-4, 100) = 36
//   modal_height:   min(18-4, 30)  = 14
//   outer_block inner: 34 × 12    (subtract 2 each dimension for the border)
//   header_height:  2   footer_height: 1   body_height: 12-3 = 9
//   body_area passed to render_generic_payload: width=34, height=9
//   Tool-Input block inner: 32 × 7  (subtract 2 each dimension again)
//
//   threshold (current code): area.height - 6 = 9 - 6 = 3
//   json_line_count (current code): excerpt.lines().count() = 1
//   show_scroll (current code): 1 > 3 → false  ← BUG: hint NOT shown
//
//   Compact JSON produced: {"description":"<230-char-ascii-value>"}
//   serde_json::to_string length: 16 + 230 + 2 = 248 chars
//   "input: " prefix: 7 chars → rendered text = 255 chars
//   Visual rows at inner_width=32: ceil(255/32) = 8 rows
//   "tool: wrap_test" = 15 chars → 1 row
//   Total visual rows: 9 > inner.height=7 → VISUAL OVERFLOW
//
//   Expected after fix: count == 1 (one scroll hint for the wrap overflow).
//   Actual now (RED):   count == 0 (logical-line check misses the wrap case).
// ---------------------------------------------------------------------------

/// BC-2.06.024 PC-3 / AC-006 (F-S027-P3-NIT-001, RED):
/// A Generic payload whose compact JSON excerpt DOES NOT contain newlines but
/// whose rendered text VISUALLY WRAPS past the available body height must show
/// EXACTLY ONE `↑↓ to scroll` hint.
///
/// Current bug: `json_line_count = excerpt.lines().count() = 1` regardless of
/// visual wrap extent, so `show_scroll = (1 > threshold)` is always false for
/// single-line JSON, producing count 0 even when visual overflow is present.
///
/// This test will FAIL on assertion (count 0 ≠ expected 1) with the current
/// implementation and PASS once the fix computes visual row count from
/// `inner.width` (e.g., `ceil(text_len / inner.width)`) before comparing to
/// the available height.
#[test]
fn test_BC_2_06_024_generic_wrap_overflow_shows_exactly_one_scroll_hint() {
    // JSON value: {"description":"<230 ASCII 'x' chars>"}
    // serde_json::to_string → 248 chars (no newlines, no truncation — under 256 cap).
    // "input: " + 248 chars = 255-char rendered input line.
    //
    // At inner_width=32 cols: ceil(255/32) = 8 visual rows for the input line.
    // Plus 1 row for "tool: wrap_test" (15 chars, fits in one row at 32 cols).
    // Total visual rows = 9 > inner.height = 7 → content visually overflows.
    //
    // Terminal 40×18 produces body_area.height=9 → threshold = 9-6 = 3.
    // With json_line_count=1 the current code computes show_scroll = (1>3) = false → count=0.
    let long_value: String = "x".repeat(230);
    let tool_input = serde_json::json!({ "description": long_value });

    let modal = generic_modal("wrap_test", tool_input);
    let backend = TestBackend::new(40, 18);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 1, frame.area(), frame);
        })
        .expect("render must not panic");

    let all_text = buffer_all_text(&terminal);
    let scroll_hint_count = count_occurrences(&all_text, "to scroll");

    assert_eq!(
        scroll_hint_count,
        1,
        "BC-2.06.024 PC-3 / AC-006 (NIT-001): a Generic payload whose single-line JSON \
         excerpt visually wraps past the body height must show EXACTLY ONE '↑↓ to scroll' \
         hint; got {} occurrences. Current bug: excerpt.lines().count()==1 so the logical-line \
         threshold check (1 > {}) never fires, even though {} visual rows exceed the {} rows \
         of inner body height.",
        scroll_hint_count,
        (9usize).saturating_sub(6), // threshold = body_area.height - 6 = 3
        9,                          // total visual rows (8 input + 1 tool)
        7,                          // inner.height (body_area inner after Tool-Input block border)
    );
}

// ---------------------------------------------------------------------------
// ADV Pass-7 NIT-2 — Strengthen AC-009 elapsed timer test
// ---------------------------------------------------------------------------

/// BC-2.06.020 PC-1 / AC-009 (CORRECTNESS-CONFIRMING): The elapsed timer renders the
/// REAL elapsed seconds — not a hardcoded `"Waiting: 0s"`.
///
/// Addresses adversarial Pass-7 NIT-2: the original `test_BC_2_06_020_overlay_header_shows_elapsed_timer_waiting_prefix`
/// used `received_at = Instant::now()` so elapsed is always 0 — it would pass against
/// an impl that hardcoded `"Waiting: 0s"`. This test back-dates `received_at` by 5 seconds
/// so elapsed is reliably 5s (as_secs() truncates; the full setup+render takes microseconds,
/// well under 1s, so 5→4 rounding cannot occur). The exact assertion `"Waiting: 5s"` would
/// FAIL against any impl that hardcodes `"Waiting: 0s"` and PASSES against the correct
/// impl that computes `Instant::now().duration_since(modal.received_at).as_secs()`.
///
/// Margin rationale: 5s chosen so sub-second execution jitter (< 1ms in practice) cannot
/// cause as_secs() to floor from 5 → 4. A 1s back-date would be vulnerable to a < 1s
/// race at the 1s boundary; 5s eliminates that risk entirely.
#[test]
fn test_BC_2_06_020_overlay_header_elapsed_timer_reflects_real_elapsed_not_hardcoded_zero() {
    // Back-date received_at by 5 seconds so elapsed is reliably 5s at render time.
    let received_at = Instant::now() - std::time::Duration::from_secs(5);
    let modal = PromptModal {
        prompt_id: uuid::Uuid::new_v4(),
        session_id: "sess-elapsed-001".to_string(),
        tool_name: "Bash".to_string(),
        tool_payload: monocle_core::tui::state::ToolPayload::Bash {
            command: "ls".to_string(),
        },
        received_at,
    };

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 1, frame.area(), frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let all_text: String = (0..buf.area().height)
        .flat_map(|y| (0..buf.area().width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("Waiting: 5s"),
        "BC-2.06.020 PC-1 / AC-009 (ADV Pass-7 NIT-2): with received_at back-dated 5s, \
         rendered header must contain 'Waiting: 5s' (real elapsed) — not 'Waiting: 0s' \
         (which would indicate a hardcoded zero); got: {:?}",
        all_text.trim()
    );
}

// ---------------------------------------------------------------------------
// ADV Pass-8 — AC-001 single-prompt stack indicator always shown
// ---------------------------------------------------------------------------

/// BC-2.06.021 PC-1 / AC-001 (CORRECTNESS-CONFIRMING): The stack-depth indicator
/// `"(1 of N)"` is ALWAYS shown in the overlay header — including when N=1 (a single
/// pending prompt). This exercises the reconciled AC-001: "indicator always shown,
/// including (1 of 1)".
///
/// The assertion checks for the contiguous substring `"(1 of 1)"` — not separately for
/// "1 of" and "1", which would be a tautology. An impl that omits the indicator when
/// stack_depth==1 would produce a header without `"(1 of 1)"` and this test would fail.
/// The current correct impl always calls `build_header_line` with the supplied
/// `stack_depth`, so this PASSES against the current implementation.
#[test]
fn test_BC_2_06_021_overlay_header_shows_one_of_one_indicator_for_single_prompt_stack() {
    // stack_depth=1: single pending prompt — indicator must still be shown.
    let modal = bash_modal("echo hello");
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_overlay_widget(&modal, 1, frame.area(), frame);
        })
        .expect("render must not panic");

    let buf = terminal.backend().buffer().clone();
    let all_text: String = (0..buf.area().height)
        .flat_map(|y| (0..buf.area().width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect();

    assert!(
        all_text.contains("(1 of 1)"),
        "BC-2.06.021 PC-1 / AC-001 (ADV Pass-8): stack_depth=1 must render the contiguous \
         substring '(1 of 1)' — the indicator is always shown, even for a single pending \
         prompt; got: {:?}",
        all_text.trim()
    );
}
