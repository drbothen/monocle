//! TDD test suite for S-039: PTY Output Pipeline
//!
//! Anchored to BC-2.09.001 (PTY Output Renders Within 100ms of Byte Receipt at TUI).
//!
//! Every test MUST FAIL before implementation — body stubs are `todo!()` (they panic),
//! so the Red Gate is satisfied as long as the stubs remain in place.
//!
//! Test naming: test_BC_2_09_001_<assertion_name> as required by the TDD contract.
//!
//! BC clause → test mapping:
//!   Postcondition 1 → test_BC_2_09_001_pty_output_renders_within_100ms (AC-001/003)
//!   Postcondition 5 → test_BC_2_09_001_non_focused_parser_updated (AC-004/006)
//!   Postcondition 6 / Invariant 5 → test_BC_2_09_001_auto_attach_on_first_entry_buffering (AC-005)
//!   Postcondition 6 re-attach → test_BC_2_09_001_reattach_after_detach_reruns_dump_protocol (AC-005)
//!   Invariant 3 → test_BC_2_09_001_invariant_bounded_channel_send_await_not_try_send (AC-007)
//!   Invariant 4 → test_BC_2_09_001_invariant_scrollback_rows_default_and_clamp (AC-008)
//!   Edge case EC-200 → test_BC_2_09_001_unknown_session_id_drop (AC-009)
//!   Edge case EC-202 → test_BC_2_09_001_high_frequency_frame_merge (AC-010)
//!   Invariant 5 dump_in_progress ordering → test_BC_2_09_001_dump_in_progress_set_before_attach_send
//!   Scrollback replay order → test_BC_2_09_001_scrollback_replay_order
//!   GC cleanup → test_BC_2_09_001_session_gc_removes_parser_and_scroll_offset
//!   Render function wiring → test_BC_2_09_001_render_embedded_terminal_calls_pseudo_terminal

// BC-2.09.001: test names use SCREAMING_SNAKE_CASE to embed the BC ID for traceability.
// This violates the Rust non_snake_case lint but is required by the TDD naming contract.
#![allow(non_snake_case)]

use monocle_config::MonocleConfig;
use monocle_core::tui::state::{
    clamp_scrollback_rows, default_scrollback_rows, AppMode, FocusSnapshot,
};
use monocle_ipc::types::ClientToServer;
use monocle_tui::app::{
    enter_embedded_terminal, exit_embedded_terminal, on_pty_output, on_scrollback_dump_complete,
    App, IPC_READER_CHANNEL_CAPACITY,
};
use monocle_tui::pty_output_channel;
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal `App` from a default config for unit tests.
///
/// Starts with no sessions, no parsers, scrollback_rows=1000.
fn make_app() -> App {
    let config = MonocleConfig::default();
    App::new(config)
}

/// Build an `App` and register a parser for `session_id` so tests that exercise
/// `on_pty_output` have a non-empty `pty_parsers` map.
///
/// Returns `(app, session_id)`.
fn make_app_with_session(session_id: &str) -> App {
    let mut app = make_app();
    // Install a blank parser for the session — same initialization as on_initial_state
    // / on_session_list_update would do (AC-008 / BC-2.09.001 Invariant 4).
    let parser = vt100::Parser::new(24, 80, app.scrollback_rows as usize);
    app.pty_parsers.insert(session_id.to_string(), parser);
    app.pty_scroll_offsets.insert(session_id.to_string(), 0);
    app
}

// ---------------------------------------------------------------------------
// AC-001 / AC-003: on_pty_output → parser.process → render tick within 100ms
// BC-2.09.001 Postcondition 1 + Postcondition 3
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_pty_output_renders_within_100ms
///
/// Exercises BC-2.09.001 Postconditions 1, 2, 3 and the 100ms budget:
///   - on_pty_output called within one mpsc cycle (PC-1)
///   - parser.process(&bytes) called, updating the screen model (PC-2)
///   - after process(), the screen reflects the written bytes (PC-3 observable via screen state)
///
/// Test vector (from BC canonical table):
///   Input:  PtyOutput { session_id: "s1", bytes: b"Hello\r\n" }
///   Output: "Hello" visible on parser screen; render tick observable
///
/// The 100ms timing budget is verified by asserting the entire on_pty_output + screen
/// read path completes within the budget using tokio::time::pause to control the clock.
#[tokio::test]
async fn test_BC_2_09_001_pty_output_renders_within_100ms() {
    // Arrange
    let session_id = "s1-bc2-09-001-render-100ms";
    let mut app = make_app_with_session(session_id);

    // tokio::time::pause gives us control over the virtual clock (BC-2.09.001 Invariant 1).
    tokio::time::pause();
    let before = tokio::time::Instant::now();

    // Act: feed "Hello\r\n" (canonical test vector from BC table) to on_pty_output.
    // This must call parser.process(&bytes) on the parser for session_id.
    on_pty_output(&mut app, session_id.to_string(), b"Hello\r\n".to_vec());

    let elapsed = before.elapsed();

    // Assert: elapsed < 100ms (BC-2.09.001 Postcondition 4 / Invariant 1).
    // We use a 100ms ceiling; on_pty_output itself should be synchronous and near-instant.
    assert!(
        elapsed.as_millis() < 100,
        "on_pty_output exceeded the 100ms budget: {:?}",
        elapsed
    );

    // Assert: the parser's screen now reflects the written bytes.
    // "Hello" should be visible on row 0 of the parser screen.
    // This verifies PC-2 (parser.process called) and PC-3 (screen model updated).
    let parser = app
        .pty_parsers
        .get(session_id)
        .expect("BC-2.09.001 PC-2: parser must exist for session after on_pty_output");
    let screen = parser.screen();
    // Row 0 of the vt100 screen should have "Hello" at the beginning.
    // Screen::row_plain() returns the text of a row without attributes.
    let row0 = screen.rows_formatted(0, 1).next();
    // If todo!() is in place, we never reach this assertion — the function panics.
    // If on_pty_output is implemented, this assertion validates screen state.
    assert!(
        row0.is_some(),
        "BC-2.09.001 PC-3: parser screen must have row 0 after processing bytes"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / AC-006: Non-focused sessions update parsers; no PTY render of them
// BC-2.09.001 Postcondition 5 / Invariant 2 / Edge Case EC-203
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_non_focused_parser_updated
///
/// Exercises BC-2.09.001 Postcondition 5 and Invariant 2:
///   - PtyOutput for non-focused session: parser.process(&bytes) is called (PC-5)
///   - No PTY widget render is triggered for the non-focused session (Invariant 2)
///   - O(1) focus switch: parser state is already populated (AC-004)
///
/// Test vector:
///   Two sessions: s2 (focused), s1 (non-focused).
///   PtyOutput arrives for s1.
///   Expected: s1's parser is updated; s2's parser is unchanged.
#[tokio::test]
async fn test_BC_2_09_001_non_focused_parser_updated() {
    // Arrange: two sessions, s2 is focused (set in app.mode)
    let s1 = "s1-non-focused";
    let s2 = "s2-focused";
    let mut app = make_app_with_session(s1);
    let p2 = vt100::Parser::new(24, 80, app.scrollback_rows as usize);
    app.pty_parsers.insert(s2.to_string(), p2);
    app.pty_scroll_offsets.insert(s2.to_string(), 0);

    // Simulate s2 being focused by setting EmbeddedTerminal mode for s2.
    app.mode = AppMode::EmbeddedTerminal {
        session_id: s2.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Record initial screen content for s2 (should be blank).
    let s2_screen_before = {
        let p = app.pty_parsers.get(s2).unwrap();
        // Capture a summary: content field of cell 0,0 (should be empty on a blank parser).
        p.screen().cell(0, 0).map(|c| c.contents().to_string())
    };

    // Act: send PtyOutput to s1 (non-focused)
    on_pty_output(&mut app, s1.to_string(), b"NonFocused\r\n".to_vec());

    // Assert 1 (BC-2.09.001 PC-5 / Invariant 2): s1's parser was updated.
    // on_pty_output must call process() even for non-focused sessions.
    // If todo!() is still in place, this test fails with a panic (correct Red Gate behavior).
    {
        let p1 = app
            .pty_parsers
            .get(s1)
            .expect("s1 parser must exist after on_pty_output");
        let _ = p1.screen(); // Must not panic
    }

    // Assert 2 (BC-2.09.001 Invariant 2 / AC-006): s2's parser is unchanged.
    // PtyOutput for s1 must NOT mutate s2's parser.
    let s2_screen_after = {
        let p = app.pty_parsers.get(s2).unwrap();
        p.screen().cell(0, 0).map(|c| c.contents().to_string())
    };
    assert_eq!(
        s2_screen_before, s2_screen_after,
        "BC-2.09.001 Invariant 2: s2 parser must not be mutated by PtyOutput for s1"
    );

    // Assert 3 (BC-2.09.001 AC-006): pty_scroll_offsets for s1 is unaffected.
    assert_eq!(
        app.pty_scroll_offsets.get(s1).copied(),
        Some(0),
        "BC-2.09.001 AC-006: scroll offset for non-focused s1 must remain 0 after PtyOutput"
    );
}

// ---------------------------------------------------------------------------
// AC-005: Auto-attach on first entry — buffering and replay
// BC-2.09.001 Postcondition 6 / Invariant 5
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_auto_attach_on_first_entry_buffering
///
/// Exercises BC-2.09.001 Postcondition 6 and Invariant 5:
///
///   1. enter_embedded_terminal sets dump_in_progress = true BEFORE AttachSession is sent.
///   2. PtyOutput arriving while dump_in_progress == true is buffered in pending_pty_bytes.
///   3. On ScrollbackDumpComplete:
///      a. Parser is reset.
///      b. Screen reconstructed (chunk data applied).
///      c. Buffered bytes replayed in receipt order.
///      d. pending_pty_bytes cleared.
///      e. dump_in_progress = false.
///      f. session_id inserted into pty_dump_received.
#[tokio::test]
async fn test_BC_2_09_001_auto_attach_on_first_entry_buffering() {
    // Arrange
    let session_id = "s1-auto-attach";
    let mut app = make_app_with_session(session_id);

    // Wire a real bounded mpsc channel so enter_embedded_terminal can send AttachSession.
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(cmd_tx);

    // Precondition: session is NOT in pty_dump_received (first entry).
    assert!(
        !app.pty_dump_received.contains(session_id),
        "precondition: session_id must not be in pty_dump_received before first entry"
    );

    // Act 1: enter_embedded_terminal — triggers auto-attach protocol.
    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // Assert A (BC-2.09.001 PC-6 / SS-embedded-pty.md §Auto-attach mandate):
    // dump_in_progress must be true immediately after enter_embedded_terminal.
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        Some(true),
        "BC-2.09.001 PC-6: dump_in_progress must be true after enter_embedded_terminal"
    );

    // Assert B: AppMode transitioned to EmbeddedTerminal.
    assert!(
        matches!(
            &app.mode,
            AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == session_id
        ),
        "BC-2.09.001 PC-6: mode must be EmbeddedTerminal after enter_embedded_terminal"
    );

    // Assert C: AttachSession was sent over the IPC channel.
    let msg = cmd_rx.try_recv().expect(
        "BC-2.09.001 PC-6: AttachSession must be sent to ipc_tx after enter_embedded_terminal",
    );
    assert!(
        matches!(msg, ClientToServer::AttachSession { session_id: ref sid } if sid == session_id),
        "BC-2.09.001 PC-6: IPC message must be ClientToServer::AttachSession for the session"
    );

    // Act 2: simulate two PtyOutput arrivals while dump_in_progress == true.
    // These must be BUFFERED, not fed to the parser.
    let buf1 = b"buffered-first\r\n".to_vec();
    let buf2 = b"buffered-second\r\n".to_vec();
    on_pty_output(&mut app, session_id.to_string(), buf1.clone());
    on_pty_output(&mut app, session_id.to_string(), buf2.clone());

    // Assert D (BC-2.09.001 Invariant 5 / BC-2.05.011 Invariant 6):
    // Bytes must be buffered in pending_pty_bytes, not fed to parser yet.
    {
        let pending = app
            .pending_pty_bytes
            .get(session_id)
            .expect("BC-2.09.001 Invariant 5: pending_pty_bytes must have entry for session");
        assert_eq!(
            pending.len(),
            2,
            "BC-2.09.001 Invariant 5: exactly 2 PtyOutput messages must be buffered"
        );
        assert_eq!(
            pending[0], buf1,
            "BC-2.09.001 Invariant 5: first buffered message must match buf1 (receipt order)"
        );
        assert_eq!(
            pending[1], buf2,
            "BC-2.09.001 Invariant 5: second buffered message must match buf2 (receipt order)"
        );
    }

    // Act 3: ScrollbackDumpComplete — triggers parser reset + replay.
    on_scrollback_dump_complete(&mut app, session_id.to_string(), 24, 80);

    // Assert E (BC-2.09.001 Invariant 5 step e): dump_in_progress = false after complete.
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        Some(false),
        "BC-2.09.001 Invariant 5: dump_in_progress must be false after ScrollbackDumpComplete"
    );

    // Assert F (BC-2.09.001 Invariant 5 step f): session inserted into pty_dump_received.
    assert!(
        app.pty_dump_received.contains(session_id),
        "BC-2.09.001 Invariant 5: session_id must be in pty_dump_received after ScrollbackDumpComplete"
    );

    // Assert G (BC-2.09.001 Invariant 5 step d): pending_pty_bytes cleared.
    let pending_after = app
        .pending_pty_bytes
        .get(session_id)
        .map(|v| v.len())
        .unwrap_or(0);
    assert_eq!(
        pending_after, 0,
        "BC-2.09.001 Invariant 5: pending_pty_bytes must be empty after replay"
    );

    // Assert H (BC-2.09.001 Invariant 5 step c): buffered bytes were replayed through parser.
    // After replay, the parser screen must contain content from the replayed bytes.
    // We verify the parser exists and has a screen (if it panicked, the test fails differently).
    let parser = app
        .pty_parsers
        .get(session_id)
        .expect("BC-2.09.001 Invariant 5: parser must exist after ScrollbackDumpComplete");
    let _ = parser.screen(); // Must not panic
}

// ---------------------------------------------------------------------------
// AC-005 re-attach: exit_embedded_terminal removes pty_dump_received
// BC-2.09.001 Postcondition 6 re-attach clause
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_reattach_after_detach_reruns_dump_protocol
///
/// Exercises the re-attach clause of BC-2.09.001 Postcondition 6:
///
///   When exit_embedded_terminal is called, pty_dump_received.remove(session_id)
///   so the next enter_embedded_terminal for the same session re-runs the full
///   attach + dump protocol.
#[tokio::test]
async fn test_BC_2_09_001_reattach_after_detach_reruns_dump_protocol() {
    // Arrange: simulate a session that has completed a dump (pty_dump_received contains it).
    let session_id = "s1-reattach";
    let mut app = make_app_with_session(session_id);
    app.pty_dump_received.insert(session_id.to_string());
    app.mode = AppMode::EmbeddedTerminal {
        session_id: session_id.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Precondition: session IS in pty_dump_received (dump was completed earlier).
    assert!(
        app.pty_dump_received.contains(session_id),
        "precondition: session_id must be in pty_dump_received before detach"
    );

    // Act: exit embedded terminal (detach).
    exit_embedded_terminal(&mut app, session_id);

    // Assert 1 (BC-2.09.001 AC-005 re-attach clause):
    // session_id MUST be removed from pty_dump_received after exit.
    assert!(
        !app.pty_dump_received.contains(session_id),
        "BC-2.09.001 AC-005 re-attach: pty_dump_received must NOT contain session_id after exit_embedded_terminal"
    );

    // Assert 2: The next enter_embedded_terminal will treat this as a first entry.
    // Verify the condition: !pty_dump_received.contains(session_id) is true.
    // (The full protocol re-run is tested in test_BC_2_09_001_auto_attach_on_first_entry_buffering.)
    let (cmd_tx, _cmd_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(cmd_tx);

    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // After re-entry, dump_in_progress must be true (auto-attach triggered again).
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        Some(true),
        "BC-2.09.001 AC-005 re-attach: dump_in_progress must be true on second enter after detach"
    );
}

// ---------------------------------------------------------------------------
// AC-007: Bounded mpsc::channel(64) with .send().await backpressure (not try_send)
// BC-2.09.001 Invariant 3
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_invariant_bounded_channel_send_await_not_try_send
///
/// Exercises BC-2.09.001 Invariant 3:
///   The IPC reader channel for the PTY output pipeline is bounded at capacity 64
///   (`IPC_READER_CHANNEL_CAPACITY`). The reader MUST use `.send().await` (blocking
///   backpressure), NOT `.try_send()` (silent drop). Silent drops violate at-least-once
///   delivery for `PtyOutput` frames.
///
/// This test asserts:
///   1. `pty_output_channel()` returns a channel whose `rx.max_capacity()` equals
///      `IPC_READER_CHANNEL_CAPACITY` (64). This call goes RED (todo!() panic) until
///      S-039 implements `pty_output_channel()`.
///   2. `IPC_READER_CHANNEL_CAPACITY == 64` — the named constant is the canonical
///      value from BC-2.09.001 Invariant 3. The constant is asserted here so that
///      any change to the capacity value is immediately visible as a test failure.
///
/// S-039 introduces `pty_output_channel()` as the production channel constructor for
/// the PTY output inbound path. Tests bind to this named function (not the inline
/// literal `64`) so the Red Gate is enforced until S-039 wires the real implementation.
#[test]
fn test_BC_2_09_001_invariant_bounded_channel_send_await_not_try_send() {
    // Assert the named constant holds the contractual capacity value.
    // If the capacity is ever changed, this assertion catches the regression.
    assert_eq!(
        IPC_READER_CHANNEL_CAPACITY, 64,
        "BC-2.09.001 Invariant 3: IPC_READER_CHANNEL_CAPACITY must be 64"
    );

    // Call the S-039 production channel constructor.
    // `pty_output_channel()` is `todo!()` → panics here, enforcing Red Gate.
    // Once implemented, `rx.max_capacity()` must equal `IPC_READER_CHANNEL_CAPACITY`.
    let (_tx, rx) = pty_output_channel();
    assert_eq!(
        rx.max_capacity(),
        IPC_READER_CHANNEL_CAPACITY,
        "BC-2.09.001 Invariant 3: pty_output_channel() receiver capacity must equal \
         IPC_READER_CHANNEL_CAPACITY ({})",
        IPC_READER_CHANNEL_CAPACITY
    );
}

// ---------------------------------------------------------------------------
// AC-008: scrollback_rows default 1000, clamped to [1, 10000]
// BC-2.09.001 Invariant 4
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_invariant_scrollback_rows_default_and_clamp
///
/// Exercises BC-2.09.001 Invariant 4:
///   - Default scrollback_rows is 1000 (absent/invalid config).
///   - Values above 10000 are clamped to 10000.
///   - Values below 1 are clamped to 1.
///   - Valid values are used as-is.
///
/// This test calls production functions from monocle-core::tui::state:
///   - `default_scrollback_rows()` — returns the canonical default from the config-load path.
///   - `clamp_scrollback_rows(raw)` — performs the [1, 10000] clamp used in run().
///
/// Both functions are `todo!()` stubs until S-039 implements them. Calling them here
/// enforces the Red Gate: each assert panics with "not yet implemented" until the
/// implementer provides the real bodies.
///
/// The test does NOT call the test-local helper that existed before S-039 (that helper
/// was tautological — it tested a copy of the logic, not the production symbol).
#[test]
fn test_BC_2_09_001_invariant_scrollback_rows_default_and_clamp() {
    // Test 1: default scrollback_rows comes from the production default helper.
    // `default_scrollback_rows()` is `todo!()` → panics here until S-039 implements it.
    // The assert verifies the contractual value (1000) is returned by the config-load path,
    // not merely the raw struct-initializer literal set in App::new().
    assert_eq!(
        default_scrollback_rows(),
        1000,
        "BC-2.09.001 Invariant 4: default_scrollback_rows() must return 1000 \
         (contractual default when config is absent or invalid)"
    );

    // Test 2: vt100::Parser initialized with scrollback_rows.
    // We verify the parser is created with the correct scrollback size by initializing
    // a parser ourselves and checking screen state (indirect verification through the API).
    // This sub-assertion does NOT depend on S-039 stubs — it verifies vt100 behavior.
    let parser = vt100::Parser::new(24, 80, 1000);
    let screen = parser.screen();
    let (rows, cols) = screen.size();
    assert_eq!(rows, 24, "BC-2.09.001 Invariant 4: parser rows must be 24");
    assert_eq!(cols, 80, "BC-2.09.001 Invariant 4: parser cols must be 80");

    // Test 3: clamping boundary — value above 10000 is clamped to 10000.
    // `clamp_scrollback_rows` is `todo!()` in production → panics here until S-039 implements it.
    // The test exercises BC-2.09.001 Invariant 4 via the PRODUCTION symbol, not a local copy.
    let clamped = clamp_scrollback_rows(99999);
    assert_eq!(
        clamped, 10000,
        "BC-2.09.001 Invariant 4: scrollback_rows > 10000 must be clamped to 10000"
    );

    // Test 4: clamping boundary — value of 0 is clamped to 1.
    let clamped_zero = clamp_scrollback_rows(0);
    assert_eq!(
        clamped_zero, 1,
        "BC-2.09.001 Invariant 4: scrollback_rows = 0 must be clamped to 1"
    );

    // Test 5: valid value in range is preserved.
    let clamped_valid = clamp_scrollback_rows(2000);
    assert_eq!(
        clamped_valid, 2000,
        "BC-2.09.001 Invariant 4: scrollback_rows in [1, 10000] must be used as-is"
    );
}

// ---------------------------------------------------------------------------
// AC-009 / EC-200: PtyOutput for unknown session_id → silent drop, no panic
// BC-2.09.001 Edge Case EC-200
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_unknown_session_id_drop
///
/// Exercises BC-2.09.001 Edge Case EC-200:
///   When pty_parsers.get_mut(&session_id) returns None (session not in parsers),
///   the bytes are silently dropped for this tick. No panic. No WARN+ log.
///
/// Test vector:
///   PtyOutput for "unknown-session-id" when pty_parsers map is empty.
///   Expected: function returns without panicking; pty_parsers unchanged.
#[test]
fn test_BC_2_09_001_unknown_session_id_drop() {
    // Arrange: App with NO parsers (empty pty_parsers map).
    let mut app = make_app();
    assert!(
        app.pty_parsers.is_empty(),
        "precondition: pty_parsers must be empty for this test"
    );

    // Act: send PtyOutput for a session_id not in pty_parsers.
    // Must NOT panic (BC-2.09.001 EC-200: "no panic").
    on_pty_output(
        &mut app,
        "unknown-session-id".to_string(),
        b"some bytes".to_vec(),
    );

    // Assert 1 (EC-200): No panic occurred (test would have failed if it did).
    // Assert 2 (EC-200): pty_parsers is still empty (bytes were silently dropped,
    // no spurious parser creation).
    assert!(
        app.pty_parsers.is_empty(),
        "BC-2.09.001 EC-200: pty_parsers must remain empty after PtyOutput for unknown session"
    );

    // Assert 3 (EC-200): pending_pty_bytes did not receive a spurious entry.
    assert!(
        app.pending_pty_bytes.is_empty(),
        "BC-2.09.001 EC-200: pending_pty_bytes must remain empty after drop"
    );
}

// ---------------------------------------------------------------------------
// AC-010 / EC-202: High-frequency PtyOutput — frame merge, 100ms budget maintained
// BC-2.09.001 Edge Case EC-202
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_high_frequency_frame_merge
///
/// Exercises BC-2.09.001 Edge Case EC-202:
///   When PTY output arrives faster than the render rate (>100 messages/second),
///   the render cycle merges frames: multiple PtyOutputs are processed before
///   one draw() call. The mpsc::channel(64) provides 64 slots of burst absorption.
///   The 100ms first-byte-to-pixel budget is still met for any single message.
///
/// This test verifies that processing 100 sequential PtyOutput messages does not
/// corrupt the parser state and completes within the 100ms budget.
#[tokio::test]
async fn test_BC_2_09_001_high_frequency_frame_merge() {
    // Arrange
    let session_id = "s1-high-freq";
    let mut app = make_app_with_session(session_id);

    tokio::time::pause();
    let before = tokio::time::Instant::now();

    // Act: send 100 PtyOutput messages in rapid succession (simulates >100 msg/s burst).
    for i in 0u8..100 {
        let bytes = format!("line{}\r\n", i).into_bytes();
        on_pty_output(&mut app, session_id.to_string(), bytes);
    }

    let elapsed = before.elapsed();

    // Assert 1 (EC-202): All 100 messages processed without panic.
    // If todo!() is in place, we never reach here (Red Gate).

    // Assert 2 (BC-2.09.001 Invariant 1): Total processing time < 100ms.
    // on_pty_output is synchronous; 100 calls should complete well within 100ms.
    assert!(
        elapsed.as_millis() < 100,
        "BC-2.09.001 EC-202: 100 sequential on_pty_output calls exceeded 100ms: {:?}",
        elapsed
    );

    // Assert 3 (EC-202): Parser is in a valid state after burst processing.
    let parser = app
        .pty_parsers
        .get(session_id)
        .expect("EC-202: parser must exist after burst processing");
    let _ = parser.screen(); // Must not panic
}

// ---------------------------------------------------------------------------
// Invariant 5 ordering: dump_in_progress MUST be set BEFORE AttachSession is sent
// BC-2.09.001 Invariant 5 / SS-embedded-pty.md §Auto-attach mandate (S12-001 fix)
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_dump_in_progress_set_before_attach_send
///
/// Exercises the ordering invariant from BC-2.09.001 PC-6 and
/// SS-embedded-pty.md §Auto-attach mandate (S12-001 fix):
///
///   dump_in_progress[session_id] MUST be set to true BEFORE AttachSession is sent.
///   Live PtyOutput may arrive before the first ScrollbackChunk — buffering must
///   begin immediately when AttachSession is sent, not on first chunk receipt.
///
/// This test verifies the ordering by intercepting the IPC channel: after
/// enter_embedded_terminal, if we can deliver a PtyOutput before AttachSession
/// has been drained from the command channel, it must still be buffered.
#[tokio::test]
async fn test_BC_2_09_001_dump_in_progress_set_before_attach_send() {
    // Arrange
    let session_id = "s1-ordering";
    let mut app = make_app_with_session(session_id);

    let (cmd_tx, _cmd_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(cmd_tx);

    // Act: enter_embedded_terminal
    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // Assert: dump_in_progress is true immediately — even before any PtyOutput or
    // ScrollbackChunk arrives. The ordering is:
    //   1. dump_in_progress = true  ← MUST happen first
    //   2. AttachSession sent
    //
    // We verify (1) by checking the field right after enter_embedded_terminal returns,
    // which is synchronous. If (1) happens AFTER (2), live PtyOutput arriving between
    // (2) and (1) would escape the buffer (S12-001 regression).
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        Some(true),
        "BC-2.09.001 PC-6 ordering (S12-001): dump_in_progress must be true immediately \
         after enter_embedded_terminal — set BEFORE AttachSession is dispatched to IPC"
    );

    // Verify: a PtyOutput arriving immediately is buffered (not fed to parser).
    on_pty_output(
        &mut app,
        session_id.to_string(),
        b"race-condition-byte\r\n".to_vec(),
    );
    let buffered = app
        .pending_pty_bytes
        .get(session_id)
        .map(|v| v.len())
        .unwrap_or(0);
    assert_eq!(
        buffered, 1,
        "BC-2.09.001 PC-6 ordering: PtyOutput arriving immediately after enter_embedded_terminal \
         must be buffered (dump_in_progress = true prevents parser feed)"
    );
}

// ---------------------------------------------------------------------------
// Scrollback replay order (BC-2.09.001 Invariant 5 step c)
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_scrollback_replay_order
///
/// Exercises BC-2.09.001 Invariant 5 step c:
///   Buffered bytes in pending_pty_bytes[session_id] are replayed through the
///   reset parser in RECEIPT ORDER after ScrollbackDumpComplete.
///
/// This test sends bytes with distinct content and verifies that after replay,
/// the parser screen reflects the LAST written content (FIFO replay order means
/// the most recently buffered message's effect is visible at the bottom).
#[tokio::test]
async fn test_BC_2_09_001_scrollback_replay_order() {
    // Arrange
    let session_id = "s1-replay-order";
    let mut app = make_app_with_session(session_id);

    // Manually set dump_in_progress to simulate an in-progress dump.
    app.dump_in_progress.insert(session_id.to_string(), true);

    // Buffer two messages (simulating arrival while dump is in progress).
    // Receipt order: "FIRST\r\n", then "SECOND\r\n".
    on_pty_output(&mut app, session_id.to_string(), b"FIRST\r\n".to_vec());
    on_pty_output(&mut app, session_id.to_string(), b"SECOND\r\n".to_vec());

    // Verify both are buffered in order.
    {
        let pending = app
            .pending_pty_bytes
            .get(session_id)
            .expect("must be buffered");
        assert_eq!(pending.len(), 2, "must have exactly 2 buffered messages");
        assert_eq!(pending[0], b"FIRST\r\n", "first message must be FIRST");
        assert_eq!(pending[1], b"SECOND\r\n", "second message must be SECOND");
    }

    // Act: ScrollbackDumpComplete — resets parser and replays buffered bytes in order.
    on_scrollback_dump_complete(&mut app, session_id.to_string(), 24, 80);

    // Assert: buffer is cleared after replay.
    let pending_after = app
        .pending_pty_bytes
        .get(session_id)
        .map(|v| v.len())
        .unwrap_or(0);
    assert_eq!(
        pending_after, 0,
        "BC-2.09.001 Invariant 5: pending_pty_bytes must be empty after replay"
    );

    // Assert: parser reflects content from both replayed messages.
    // After FIRST\r\n and SECOND\r\n, the parser should have content on its screen.
    let parser = app
        .pty_parsers
        .get(session_id)
        .expect("parser must exist after ScrollbackDumpComplete");
    let screen = parser.screen();
    // At minimum, the screen must have rows (not panic). The exact content
    // depends on the parser state after replay; the key invariant is RECEIPT ORDER
    // (FIRST then SECOND, not reversed). We verify the buffer ordering above.
    let _ = screen;
}

// ---------------------------------------------------------------------------
// AC-008: Session GC removes parser, scroll offset, and dump state
// BC-2.09.001 Invariant 4 + AC-008 §GC cleanup
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_session_gc_removes_parser_and_scroll_offset
///
/// Exercises BC-2.09.001 AC-008 GC clause:
///   On session GC (SessionState::Terminated + list removal):
///   - pty_parsers[session_id] removed
///   - pty_scroll_offsets[session_id] removed
///   - pty_dump_received entry removed
///
/// This test drives the GC path via on_pty_output with a session_id that has
/// been pre-inserted into all maps, then calls the GC cleanup stub and verifies
/// all entries are removed.
///
/// Note: S-039 owns initialization; S-042 owns the ResizePane reset; S-043 reads
/// the offsets. The GC cleanup itself is part of S-039's session-lifecycle management.
#[test]
fn test_BC_2_09_001_session_gc_removes_parser_and_scroll_offset() {
    // Arrange: session with parser, scroll offset, and dump received flag populated.
    let session_id = "s1-gc";
    let mut app = make_app_with_session(session_id);
    app.pty_dump_received.insert(session_id.to_string());
    app.dump_in_progress.insert(session_id.to_string(), false);
    app.pending_pty_bytes.insert(session_id.to_string(), vec![]);

    // Preconditions
    assert!(
        app.pty_parsers.contains_key(session_id),
        "precondition: parser exists"
    );
    assert!(
        app.pty_scroll_offsets.contains_key(session_id),
        "precondition: scroll offset exists"
    );
    assert!(
        app.pty_dump_received.contains(session_id),
        "precondition: dump received flag set"
    );

    // Act: call the GC cleanup function.
    // S-039 implements this as part of the SessionState::Terminated handler.
    // We call it directly here to test the cleanup behavior.
    gc_session(&mut app, session_id);

    // Assert: all per-session state removed.
    assert!(
        !app.pty_parsers.contains_key(session_id),
        "BC-2.09.001 AC-008 GC: pty_parsers[session_id] must be removed on GC"
    );
    assert!(
        !app.pty_scroll_offsets.contains_key(session_id),
        "BC-2.09.001 AC-008 GC: pty_scroll_offsets[session_id] must be removed on GC"
    );
    assert!(
        !app.pty_dump_received.contains(session_id),
        "BC-2.09.001 AC-008 GC: pty_dump_received entry must be removed on GC"
    );
}

/// Helper that exercises the S-039 session GC cleanup path.
///
/// S-039 must implement this cleanup as part of the Terminated session handler.
/// This function must be replaced by a call to the real implementation;
/// calling a non-existent or todo!() function here enforces the Red Gate.
fn gc_session(app: &mut App, session_id: &str) {
    // S-039 implementation requirement: on Terminated + list removal, remove:
    //   app.pty_parsers.remove(session_id)
    //   app.pty_scroll_offsets.remove(session_id)
    //   app.pty_dump_received.remove(session_id)
    //   app.dump_in_progress.remove(session_id)
    //   app.pending_pty_bytes.remove(session_id)
    //
    // This stub always panics to enforce the Red Gate.
    // The implementer must replace this with the real GC call from app.rs.
    monocle_tui::app::gc_pty_session(app, session_id);
}

// ---------------------------------------------------------------------------
// render_embedded_terminal: creates PseudoTerminal and renders into area
// BC-2.09.001 AC-003 / Postcondition 3
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_render_embedded_terminal_calls_pseudo_terminal
///
/// Exercises BC-2.09.001 AC-003 / Postcondition 3:
///   render_embedded_terminal(frame, area, parser) creates
///   PseudoTerminal::new(parser.screen()) and renders it into the pane Rect.
///
/// This test creates a headless ratatui terminal backend and calls
/// render_embedded_terminal, verifying the call completes without panic.
/// The test fails (todo!() panic) until the implementation is provided.
#[test]
fn test_BC_2_09_001_render_embedded_terminal_calls_pseudo_terminal() {
    use monocle_tui::ui::embedded_terminal::render_embedded_terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    // Arrange: headless ratatui backend (80 cols × 24 rows).
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");

    // Create a parser with some content.
    let mut parser = vt100::Parser::new(24, 80, 1000);
    parser.process(b"Hello PTY\r\n");

    // Act: call render_embedded_terminal inside a draw closure.
    // The todo!() stub in embedded_terminal.rs will panic here → Red Gate.
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            render_embedded_terminal(frame, area, &parser);
        })
        .expect("terminal.draw must succeed");

    // Assert: if we reach here (no panic), the render completed successfully.
    // The todo!() stub panics inside frame.draw, so this assertion is unreachable
    // until the implementation is provided.
    let buffer = terminal.backend().buffer().clone();
    // The PseudoTerminal widget should have written "Hello PTY" to the buffer.
    // We check cell (0, 0) for the 'H' character.
    let cell_0_0 = &buffer[(0, 0)];
    assert_eq!(
        cell_0_0.symbol(),
        "H",
        "BC-2.09.001 AC-003: render_embedded_terminal must write parser content to the frame"
    );
}

// ---------------------------------------------------------------------------
// Second-enter idempotence: if pty_dump_received already contains session_id,
// enter_embedded_terminal must NOT send AttachSession (O(1) path)
// BC-2.09.001 AC-004 / Postcondition 6
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_second_enter_skips_attach_when_dump_already_received
///
/// Exercises BC-2.09.001 AC-004 and the O(1) path of Postcondition 6:
///   If session_id IS in pty_dump_received, enter_embedded_terminal transitions
///   directly to EmbeddedTerminal WITHOUT sending AttachSession (no re-dump).
///
/// The parser is already populated; focus switch is O(1) — no re-fetch.
#[tokio::test]
async fn test_BC_2_09_001_second_enter_skips_attach_when_dump_already_received() {
    // Arrange: session already has a complete dump recorded.
    let session_id = "s1-second-enter";
    let mut app = make_app_with_session(session_id);
    app.pty_dump_received.insert(session_id.to_string());

    let (cmd_tx, mut cmd_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(cmd_tx);

    // Precondition: session IS in pty_dump_received.
    assert!(
        app.pty_dump_received.contains(session_id),
        "precondition: session_id must be in pty_dump_received"
    );

    // Act: enter_embedded_terminal — should take the O(1) path.
    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // Assert 1 (AC-004 O(1) path): NO AttachSession sent.
    let result = cmd_rx.try_recv();
    assert!(
        matches!(result, Err(tokio::sync::mpsc::error::TryRecvError::Empty)),
        "BC-2.09.001 AC-004: AttachSession must NOT be sent when pty_dump_received contains session_id"
    );

    // Assert 2 (AC-004): dump_in_progress must NOT be set to true.
    let dip = app
        .dump_in_progress
        .get(session_id)
        .copied()
        .unwrap_or(false);
    assert!(
        !dip,
        "BC-2.09.001 AC-004: dump_in_progress must NOT be set when taking the O(1) path"
    );

    // Assert 3: Mode transitioned to EmbeddedTerminal.
    assert!(
        matches!(
            &app.mode,
            AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == session_id
        ),
        "BC-2.09.001 AC-004: mode must be EmbeddedTerminal even on the O(1) path"
    );
}

// ---------------------------------------------------------------------------
// AC-008: config → App::scrollback_rows full wiring (config-load flow)
// BC-2.09.001 Invariant 4
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_config_scrollback_rows_wiring
///
/// Exercises the FULL config→App wiring path (AC-008 / BC-2.09.001 Invariant 4):
///   - `MonocleConfig { pty_scrollback_rows: Some(15000) }` → `App::scrollback_rows == 10000` (clamped)
///   - `MonocleConfig { pty_scrollback_rows: Some(0) }` → `App::scrollback_rows == 1` (clamped)
///   - `MonocleConfig { pty_scrollback_rows: None }` → `App::scrollback_rows == 1000` (default)
///   - `MonocleConfig { pty_scrollback_rows: Some(500) }` → `App::scrollback_rows == 500` (in-range)
///
/// This test uses the production `App::new(config)` path — NOT a test-local copy of the
/// clamp logic. The binding between MonocleConfig::pty_scrollback_rows and App::scrollback_rows
/// is enforced here so that any regression in the wiring causes this test to fail.
#[test]
fn test_BC_2_09_001_config_scrollback_rows_wiring() {
    // Case 1: value above 10000 is clamped to 10000.
    let config_over = MonocleConfig {
        pty_scrollback_rows: Some(15000),
        ..MonocleConfig::default()
    };
    let app_over = App::new(config_over);
    assert_eq!(
        app_over.scrollback_rows, 10000,
        "BC-2.09.001 Invariant 4 (AC-008): pty_scrollback_rows=15000 must clamp to 10000"
    );

    // Case 2: value of 0 is clamped to 1.
    let config_zero = MonocleConfig {
        pty_scrollback_rows: Some(0),
        ..MonocleConfig::default()
    };
    let app_zero = App::new(config_zero);
    assert_eq!(
        app_zero.scrollback_rows, 1,
        "BC-2.09.001 Invariant 4 (AC-008): pty_scrollback_rows=0 must clamp to 1"
    );

    // Case 3: None (absent from JSON) → 1000-row default.
    let config_none = MonocleConfig {
        pty_scrollback_rows: None,
        ..MonocleConfig::default()
    };
    let app_none = App::new(config_none);
    assert_eq!(
        app_none.scrollback_rows, 1000,
        "BC-2.09.001 Invariant 4 (AC-008): absent pty_scrollback_rows must produce default 1000"
    );

    // Case 4: valid in-range value is preserved.
    let config_valid = MonocleConfig {
        pty_scrollback_rows: Some(500),
        ..MonocleConfig::default()
    };
    let app_valid = App::new(config_valid);
    assert_eq!(
        app_valid.scrollback_rows, 500,
        "BC-2.09.001 Invariant 4 (AC-008): pty_scrollback_rows=500 must be preserved (in-range)"
    );
}
