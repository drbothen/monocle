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
//!   F-S039-009 first-entry buffering → test_BC_2_09_001_first_entry_session_not_preinserted
//!   Parser creation via on_initial_state → test_BC_2_09_001_on_initial_state_creates_parsers_no_clobber
//!   Parser creation via SessionListUpdate → test_BC_2_09_001_session_list_update_creates_and_gcs_parsers
//!   GC via SessionStateChanged::Terminated → test_BC_2_09_001_session_terminated_gc
//!   Async attach rollback on send err → test_BC_2_09_001_enter_embedded_rollback_on_send_failure
//!   Render arm invokes correct parser → test_BC_2_09_001_render_frame_embedded_terminal_uses_focused_parser

// BC-2.09.001: test names use SCREAMING_SNAKE_CASE to embed the BC ID for traceability.
// This violates the Rust non_snake_case lint but is required by the TDD naming contract.
#![allow(non_snake_case)]

use monocle_config::MonocleConfig;
use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::tui::state::{
    clamp_scrollback_rows, default_scrollback_rows, AppMode, FocusSnapshot,
};
use monocle_ipc::types::{ClientToServer, ServerToClient, SessionState};
use monocle_tui::app::{
    enter_embedded_terminal, exit_embedded_terminal, handle_server_message, on_initial_state,
    on_pty_output, on_scrollback_dump_complete, render_frame, App, IPC_READER_CHANNEL_CAPACITY,
};
use monocle_tui::pty_output_channel;
use monocle_tui::ui::sessions_panel::SessionsPanelState;
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

/// Build a minimal `EnrichedSession` for use in integration test fixtures.
fn make_session(id: &str) -> EnrichedSession {
    EnrichedSession::new(
        id.to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Active,
        None,
        None,
        None,
        0,
        None,
    )
}

/// Extract the trimmed plain-text content of the vt100 screen as a single string.
///
/// Uses `screen.contents()` (plain text without ANSI escapes) and trims trailing
/// whitespace/newlines so assertions are not sensitive to terminal padding.
fn screen_text(parser: &vt100::Parser) -> String {
    parser.screen().contents().trim_end().to_string()
}

// ---------------------------------------------------------------------------
// AC-001 / AC-003: on_pty_output → parser.process → render tick within 100ms
// BC-2.09.001 Postcondition 1 + Postcondition 3
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_pty_output_renders_within_100ms
///
/// Exercises BC-2.09.001 Postconditions 1, 2, 3:
///   - on_pty_output called within one mpsc cycle (PC-1)
///   - parser.process(&bytes) called, updating the screen model (PC-2)
///   - after process(), the screen reflects the written bytes (PC-3)
///
/// Test vector (from BC canonical table):
///   Input:  PtyOutput { session_id: "s1", bytes: b"Hello\r\n" }
///   Output: screen text contains "Hello"
///
/// F-S039-008 fix: assert the screen CONTAINS "Hello" (real content check), not
/// just `row0.is_some()` (vacuously true on any fresh vt100 screen). Remove the
/// paused-clock timing assertion — tokio::time::pause makes elapsed ~0 regardless
/// of work done, making the timing assertion meaningless. Replace with a content
/// assertion that can only pass when parser.process() was actually called.
#[tokio::test]
async fn test_BC_2_09_001_pty_output_renders_within_100ms() {
    // Arrange
    let session_id = "s1-bc2-09-001-render-100ms";
    let mut app = make_app_with_session(session_id);

    // Act: feed "Hello\r\n" (canonical test vector from BC table) to on_pty_output.
    // This MUST call parser.process(&bytes) on the parser for session_id.
    on_pty_output(&mut app, session_id.to_string(), b"Hello\r\n".to_vec());

    // Assert: the parser's screen now contains "Hello".
    // This is the only assertion that can distinguish "process() was called" from
    // "process() was not called" — row0.is_some() is always true on a fresh vt100
    // screen and cannot detect whether bytes were processed (F-S039-008 fix).
    let parser = app
        .pty_parsers
        .get(session_id)
        .expect("BC-2.09.001 PC-2: parser must exist for session after on_pty_output");

    let text = screen_text(parser);
    assert!(
        text.contains("Hello"),
        "BC-2.09.001 PC-3: parser screen must contain 'Hello' after processing b\"Hello\\r\\n\" — \
         got: {:?}",
        text
    );

    // Assert: the parser byte total reflects the processed input.
    // A well-formed vt100 parser fed 7 bytes ("Hello\r\n") must have advanced its
    // internal cursor; the screen content check above is the observable proxy.
    // (No timing assertion — paused clock makes elapsed assertions vacuous.)
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
///   Expected: s1's parser is updated with "NonFocused"; s2's parser is unchanged.
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
    let s2_text_before = {
        let p = app.pty_parsers.get(s2).unwrap();
        screen_text(p)
    };

    // Act: send PtyOutput to s1 (non-focused)
    on_pty_output(&mut app, s1.to_string(), b"NonFocused\r\n".to_vec());

    // Assert 1 (BC-2.09.001 PC-5 / Invariant 2): s1's parser was updated with the bytes.
    // on_pty_output MUST call process() even for non-focused sessions.
    {
        let p1 = app
            .pty_parsers
            .get(s1)
            .expect("s1 parser must exist after on_pty_output");
        let s1_text = screen_text(p1);
        assert!(
            s1_text.contains("NonFocused"),
            "BC-2.09.001 PC-5: s1 parser screen must contain 'NonFocused' after on_pty_output — \
             got: {:?}",
            s1_text
        );
    }

    // Assert 2 (BC-2.09.001 Invariant 2 / AC-006): s2's parser is unchanged.
    // PtyOutput for s1 must NOT mutate s2's parser.
    let s2_text_after = {
        let p = app.pty_parsers.get(s2).unwrap();
        screen_text(p)
    };
    assert_eq!(
        s2_text_before, s2_text_after,
        "BC-2.09.001 Invariant 2: s2 parser must not be mutated by PtyOutput for s1"
    );
    // s2 parser should still be blank (no "NonFocused" text)
    assert!(
        !s2_text_after.contains("NonFocused"),
        "BC-2.09.001 Invariant 2: s2 parser must not contain s1's bytes"
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

    // Assert E (BC-2.09.001 Invariant 5): buffered bytes were NOT yet fed to the parser.
    // The parser screen must NOT contain "buffered-first" before ScrollbackDumpComplete.
    {
        let parser = app.pty_parsers.get(session_id).unwrap();
        let text_before_dump = screen_text(parser);
        assert!(
            !text_before_dump.contains("buffered-first"),
            "BC-2.09.001 Invariant 5: parser must NOT contain buffered bytes before \
             ScrollbackDumpComplete — got: {:?}",
            text_before_dump
        );
    }

    // Act 3: ScrollbackDumpComplete — triggers parser reset + replay.
    on_scrollback_dump_complete(&mut app, session_id.to_string(), 24, 80);

    // Assert F (BC-2.09.001 Invariant 5 step e): dump_in_progress = false after complete.
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        Some(false),
        "BC-2.09.001 Invariant 5: dump_in_progress must be false after ScrollbackDumpComplete"
    );

    // Assert G (BC-2.09.001 Invariant 5 step f): session inserted into pty_dump_received.
    assert!(
        app.pty_dump_received.contains(session_id),
        "BC-2.09.001 Invariant 5: session_id must be in pty_dump_received after ScrollbackDumpComplete"
    );

    // Assert H (BC-2.09.001 Invariant 5 step d): pending_pty_bytes cleared.
    let pending_after = app
        .pending_pty_bytes
        .get(session_id)
        .map(|v| v.len())
        .unwrap_or(0);
    assert_eq!(
        pending_after, 0,
        "BC-2.09.001 Invariant 5: pending_pty_bytes must be empty after replay"
    );

    // Assert I (BC-2.09.001 Invariant 5 step c): buffered bytes were replayed through parser.
    // After replay of "buffered-first\r\n" and "buffered-second\r\n", the screen must
    // contain the replayed content.
    let parser = app
        .pty_parsers
        .get(session_id)
        .expect("BC-2.09.001 Invariant 5: parser must exist after ScrollbackDumpComplete");
    let text_after_replay = screen_text(parser);
    assert!(
        text_after_replay.contains("buffered-first"),
        "BC-2.09.001 Invariant 5: parser screen must contain 'buffered-first' after replay — \
         got: {:?}",
        text_after_replay
    );
    assert!(
        text_after_replay.contains("buffered-second"),
        "BC-2.09.001 Invariant 5: parser screen must contain 'buffered-second' after replay — \
         got: {:?}",
        text_after_replay
    );
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
///      `IPC_READER_CHANNEL_CAPACITY` (64).
///   2. `IPC_READER_CHANNEL_CAPACITY == 64` — the named constant is the canonical
///      value from BC-2.09.001 Invariant 3.
#[test]
fn test_BC_2_09_001_invariant_bounded_channel_send_await_not_try_send() {
    // Assert the named constant holds the contractual capacity value.
    // If the capacity is ever changed, this assertion catches the regression.
    assert_eq!(
        IPC_READER_CHANNEL_CAPACITY, 64,
        "BC-2.09.001 Invariant 3: IPC_READER_CHANNEL_CAPACITY must be 64"
    );

    // Call the S-039 production channel constructor.
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
#[test]
fn test_BC_2_09_001_invariant_scrollback_rows_default_and_clamp() {
    // Test 1: default scrollback_rows comes from the production default helper.
    assert_eq!(
        default_scrollback_rows(),
        1000,
        "BC-2.09.001 Invariant 4: default_scrollback_rows() must return 1000 \
         (contractual default when config is absent or invalid)"
    );

    // Test 2: vt100::Parser initialized with scrollback_rows.
    let parser = vt100::Parser::new(24, 80, 1000);
    let screen = parser.screen();
    let (rows, cols) = screen.size();
    assert_eq!(rows, 24, "BC-2.09.001 Invariant 4: parser rows must be 24");
    assert_eq!(cols, 80, "BC-2.09.001 Invariant 4: parser cols must be 80");

    // Test 3: clamping boundary — value above 10000 is clamped to 10000.
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
///
/// This test verifies that processing 100 sequential PtyOutput messages does not
/// corrupt the parser state and that the last written line is visible on screen.
#[tokio::test]
async fn test_BC_2_09_001_high_frequency_frame_merge() {
    // Arrange
    let session_id = "s1-high-freq";
    let mut app = make_app_with_session(session_id);

    // Act: send 100 PtyOutput messages in rapid succession (simulates >100 msg/s burst).
    // The last line written is "line99\r\n" — it must appear on the screen.
    for i in 0u8..100 {
        let bytes = format!("line{}\r\n", i).into_bytes();
        on_pty_output(&mut app, session_id.to_string(), bytes);
    }

    // Assert 1 (EC-202): All 100 messages processed without panic.

    // Assert 2 (EC-202): Parser is in a valid state after burst processing.
    // The last line sent was "line99" — it must appear on the screen, proving
    // that all 100 calls actually processed bytes (not a no-op count).
    let parser = app
        .pty_parsers
        .get(session_id)
        .expect("EC-202: parser must exist after burst processing");
    let text = screen_text(parser);
    assert!(
        text.contains("line99"),
        "BC-2.09.001 EC-202: parser screen must contain 'line99' after 100-burst processing — \
         got: {:?}",
        text
    );
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
// F-S039-008 fix applied: assert screen reflects bytes IN ORDER; reversed replay FAILS
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_scrollback_replay_order
///
/// Exercises BC-2.09.001 Invariant 5 step c:
///   Buffered bytes in pending_pty_bytes[session_id] are replayed through the
///   reset parser in RECEIPT ORDER after ScrollbackDumpComplete.
///
/// F-S039-008 fix: write distinguishable cursor-positioned byte sequences that land
/// on different rows. After replay, assert the parser screen shows FIRST on row 0
/// and SECOND visible on the screen. A reversed replay would show SECOND on the
/// earlier position and FIRST later, breaking the assertion.
///
/// The specific observable invariant: after replaying "ALPHA\r\n" then "BETA\r\n"
/// in receipt order, the screen text must contain "ALPHA" appearing BEFORE "BETA"
/// (ALPHA occupies an earlier position in `screen.contents()` because it was
/// processed first — i.e., it is above BETA in the scrollable content).
#[tokio::test]
async fn test_BC_2_09_001_scrollback_replay_order() {
    // Arrange
    let session_id = "s1-replay-order";
    let mut app = make_app_with_session(session_id);

    // Manually set dump_in_progress to simulate an in-progress dump.
    app.dump_in_progress.insert(session_id.to_string(), true);

    // Buffer two messages (simulating arrival while dump is in progress).
    // Receipt order: "ALPHA\r\n" (first), then "BETA\r\n" (second).
    // Using all-caps distinguishable tokens to make ordering unambiguous.
    on_pty_output(&mut app, session_id.to_string(), b"ALPHA\r\n".to_vec());
    on_pty_output(&mut app, session_id.to_string(), b"BETA\r\n".to_vec());

    // Verify both are buffered in order before replay.
    {
        let pending = app
            .pending_pty_bytes
            .get(session_id)
            .expect("must be buffered");
        assert_eq!(pending.len(), 2, "must have exactly 2 buffered messages");
        assert_eq!(pending[0], b"ALPHA\r\n", "first message must be ALPHA");
        assert_eq!(pending[1], b"BETA\r\n", "second message must be BETA");
    }

    // Act: ScrollbackDumpComplete — resets parser and replays buffered bytes in order.
    on_scrollback_dump_complete(&mut app, session_id.to_string(), 24, 80);

    // Assert 1: buffer is cleared after replay.
    let pending_after = app
        .pending_pty_bytes
        .get(session_id)
        .map(|v| v.len())
        .unwrap_or(0);
    assert_eq!(
        pending_after, 0,
        "BC-2.09.001 Invariant 5: pending_pty_bytes must be empty after replay"
    );

    // Assert 2: parser screen contains both tokens.
    let parser = app
        .pty_parsers
        .get(session_id)
        .expect("parser must exist after ScrollbackDumpComplete");
    let text = screen_text(parser);
    assert!(
        text.contains("ALPHA"),
        "BC-2.09.001 Invariant 5 replay: screen must contain 'ALPHA' — got: {:?}",
        text
    );
    assert!(
        text.contains("BETA"),
        "BC-2.09.001 Invariant 5 replay: screen must contain 'BETA' — got: {:?}",
        text
    );

    // Assert 3 (ORDER): ALPHA must appear BEFORE BETA in the screen text.
    // In receipt-order replay, "ALPHA\r\n" is processed first (occupies an earlier
    // screen position / earlier byte offset in `screen.contents()`), so its character
    // offset must be less than BETA's. A reversed replay would invert this — this
    // assertion enforces the FIFO receipt-order invariant.
    let alpha_pos = text.find("ALPHA").expect("ALPHA must be in screen text");
    let beta_pos = text.find("BETA").expect("BETA must be in screen text");
    assert!(
        alpha_pos < beta_pos,
        "BC-2.09.001 Invariant 5 replay ORDER: 'ALPHA' (pos {}) must appear before 'BETA' (pos {}) \
         in screen contents — reversed replay would break this (receipt-order violation)",
        alpha_pos,
        beta_pos
    );
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
/// This test drives the GC path via gc_pty_session and verifies all entries
/// are removed.
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
fn gc_session(app: &mut App, session_id: &str) {
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
/// render_embedded_terminal, verifying that "Hello PTY" appears in the buffer.
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
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            render_embedded_terminal(frame, area, &parser);
        })
        .expect("terminal.draw must succeed");

    // Assert: the PseudoTerminal widget wrote "Hello PTY" to the buffer.
    // We check cell (0, 0) for the 'H' character — the first character of the output.
    let buffer = terminal.backend().buffer().clone();
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

// ---------------------------------------------------------------------------
// F-S039-009: First-entry production sequence — session NOT pre-inserted
// BC-2.09.001 Invariant 5 / AC-005
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_first_entry_session_not_preinserted
///
/// Exercises F-S039-009: the REAL first-entry production sequence where the session
/// is NOT pre-inserted into pty_parsers before entering EmbeddedTerminal.
///
/// Production flow:
///   1. Session is absent from pty_parsers (race: PtyOutput arrives before SessionListUpdate).
///      OR: user presses Enter on EmbeddedTerminal before on_initial_state has run.
///   2. enter_embedded_terminal sets dump_in_progress = true, sends AttachSession.
///   3. Live PtyOutput arrives during dump_in_progress — buffered into pending_pty_bytes
///      (NOT fed to a non-existent parser — EC-200 drop guard is bypassed because
///       dump_in_progress check runs FIRST in on_pty_output).
///   4. ScrollbackDumpComplete creates the parser via reset and replays buffered bytes.
///   5. Post-dump parser screen contains the buffered content in order.
///
/// This test confirms that the session NOT being in pty_parsers before enter does NOT
/// cause data loss: bytes buffered during the dump window are always replayed after the
/// parser is created by ScrollbackDumpComplete.
#[tokio::test]
async fn test_BC_2_09_001_first_entry_session_not_preinserted() {
    // Arrange: App with NO pre-inserted parser for the session.
    let session_id = "s1-not-preinserted";
    let mut app = make_app();
    // Deliberately do NOT insert into pty_parsers — this is the F-S039-009 scenario.
    assert!(
        !app.pty_parsers.contains_key(session_id),
        "precondition: session must NOT be in pty_parsers for F-S039-009 scenario"
    );

    // Wire IPC channel so enter_embedded_terminal can send AttachSession.
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(cmd_tx);

    // Act 1: enter_embedded_terminal — session NOT in pty_dump_received.
    // Should set dump_in_progress = true and send AttachSession.
    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // Assert A: dump_in_progress is true.
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        Some(true),
        "F-S039-009: dump_in_progress must be true after enter_embedded_terminal \
         (session not pre-inserted)"
    );

    // Assert B: AttachSession was sent.
    let msg = cmd_rx.try_recv().expect(
        "F-S039-009: AttachSession must be sent even when session not pre-inserted in pty_parsers",
    );
    assert!(
        matches!(msg, ClientToServer::AttachSession { session_id: ref sid } if sid == session_id),
        "F-S039-009: IPC message must be ClientToServer::AttachSession"
    );

    // Act 2: PtyOutput arrives while dump_in_progress — must be BUFFERED.
    // With dump_in_progress=true, on_pty_output buffers into pending_pty_bytes BEFORE
    // checking pty_parsers (BC-2.09.001 Invariant 5 / on_pty_output implementation).
    let buffered_data = b"live-during-dump\r\n".to_vec();
    on_pty_output(&mut app, session_id.to_string(), buffered_data.clone());

    // Assert C: bytes were buffered (not silently dropped via EC-200 path).
    {
        let pending = app
            .pending_pty_bytes
            .get(session_id)
            .expect("F-S039-009: pending_pty_bytes must have entry when dump_in_progress=true");
        assert_eq!(pending.len(), 1, "F-S039-009: one buffered chunk expected");
        assert_eq!(
            pending[0], buffered_data,
            "F-S039-009: buffered chunk must match sent bytes"
        );
    }

    // Assert D: parser does NOT exist yet (session was not pre-inserted).
    // ScrollbackDumpComplete is the event that CREATES the parser.
    assert!(
        !app.pty_parsers.contains_key(session_id),
        "F-S039-009: pty_parsers must NOT contain session before ScrollbackDumpComplete \
         (parser created by on_scrollback_dump_complete, not by enter_embedded_terminal)"
    );

    // Act 3: ScrollbackDumpComplete — creates parser via reset, replays buffered bytes.
    on_scrollback_dump_complete(&mut app, session_id.to_string(), 24, 80);

    // Assert E: parser now exists (created by ScrollbackDumpComplete).
    assert!(
        app.pty_parsers.contains_key(session_id),
        "F-S039-009: pty_parsers must contain session after ScrollbackDumpComplete"
    );

    // Assert F: dump_in_progress = false.
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        Some(false),
        "F-S039-009: dump_in_progress must be false after ScrollbackDumpComplete"
    );

    // Assert G: pty_dump_received contains the session.
    assert!(
        app.pty_dump_received.contains(session_id),
        "F-S039-009: pty_dump_received must contain session after ScrollbackDumpComplete"
    );

    // Assert H: pending_pty_bytes cleared.
    let pending_len = app
        .pending_pty_bytes
        .get(session_id)
        .map(|v| v.len())
        .unwrap_or(0);
    assert_eq!(
        pending_len, 0,
        "F-S039-009: pending_pty_bytes must be empty after replay"
    );

    // Assert I: parser screen contains the buffered content (replayed in order).
    // This is the key assertion — it proves the buffered bytes were NOT lost.
    let parser = app.pty_parsers.get(session_id).unwrap();
    let text = screen_text(parser);
    assert!(
        text.contains("live-during-dump"),
        "F-S039-009: parser screen must contain 'live-during-dump' after replay — \
         buffered bytes must NOT be lost when session was not pre-inserted. Got: {:?}",
        text
    );
}

// ---------------------------------------------------------------------------
// Integration: Parser creation via on_initial_state (no-clobber invariant)
// BC-2.09.001 Invariant 5 / AC-001 — F-S039-001
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_on_initial_state_creates_parsers_no_clobber
///
/// Exercises the production dispatch path for parser creation:
///   - `on_initial_state` creates a parser for each session in the initial roster.
///   - `pty_scroll_offsets[id] == 0` for each created parser.
///   - No-clobber: a session already in pty_parsers is NOT replaced on roster refresh.
///
/// This drives the PRODUCTION `on_initial_state` code path (not the test-local helper).
/// The no-clobber invariant ensures reconnect does not destroy parser state for
/// sessions that survived the reconnect (BC-2.09.001 F-S039-001 no-clobber invariant).
#[test]
fn test_BC_2_09_001_on_initial_state_creates_parsers_no_clobber() {
    let mut app = make_app();

    let s1 = "session-init-001";
    let s2 = "session-init-002";

    // Act 1: on_initial_state with two sessions.
    on_initial_state(
        &mut app,
        vec![make_session(s1), make_session(s2)],
        vec![],
        vec![],
        0,
    );

    // Assert A: parsers created for both sessions.
    assert!(
        app.pty_parsers.contains_key(s1),
        "BC-2.09.001 F-S039-001: on_initial_state must create parser for s1"
    );
    assert!(
        app.pty_parsers.contains_key(s2),
        "BC-2.09.001 F-S039-001: on_initial_state must create parser for s2"
    );

    // Assert B: scroll offsets initialized to 0.
    assert_eq!(
        app.pty_scroll_offsets.get(s1).copied(),
        Some(0),
        "BC-2.09.001 F-S039-001: pty_scroll_offsets[s1] must be 0 after on_initial_state"
    );
    assert_eq!(
        app.pty_scroll_offsets.get(s2).copied(),
        Some(0),
        "BC-2.09.001 F-S039-001: pty_scroll_offsets[s2] must be 0 after on_initial_state"
    );

    // Act 2 (no-clobber): feed content into s1's parser, then call on_initial_state again.
    // The reconnect path calls on_initial_state with the same sessions still present.
    {
        let parser = app.pty_parsers.get_mut(s1).unwrap();
        parser.process(b"existing-content\r\n");
    }

    // Verify content is present before the second on_initial_state call.
    {
        let text = screen_text(app.pty_parsers.get(s1).unwrap());
        assert!(
            text.contains("existing-content"),
            "precondition: s1 parser must have content before second on_initial_state"
        );
    }

    // Call on_initial_state again (simulating reconnect with same session roster).
    on_initial_state(
        &mut app,
        vec![make_session(s1), make_session(s2)],
        vec![],
        vec![],
        0,
    );

    // Assert C (no-clobber): s1's parser content must be preserved.
    // on_initial_state uses `if !pty_parsers.contains_key(id)` — existing parsers
    // are NOT replaced. This is the no-clobber invariant (BC-2.09.001 F-S039-001).
    let text_after = screen_text(app.pty_parsers.get(s1).unwrap());
    assert!(
        text_after.contains("existing-content"),
        "BC-2.09.001 F-S039-001 no-clobber: s1 parser must RETAIN content after second \
         on_initial_state (reconnect must not clobber existing parsers). Got: {:?}",
        text_after
    );
}

// ---------------------------------------------------------------------------
// Integration: SessionListUpdate creates parsers for new sessions; GCs removed ones
// BC-2.09.001 Invariant 5 / AC-008 — F-S039-001 / F-S039-003
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_session_list_update_creates_and_gcs_parsers
///
/// Exercises the production `handle_server_message → SessionListUpdate` dispatch path:
///   - New sessions in the updated roster get a parser + scroll_offset=0.
///   - Sessions absent from the updated roster are GC'd via gc_pty_session.
///   - No-clobber: existing parsers with content are not replaced on roster refresh.
///
/// This test drives `handle_server_message` directly (production dispatch path).
#[test]
fn test_BC_2_09_001_session_list_update_creates_and_gcs_parsers() {
    let mut app = make_app();

    let existing = "session-existing-001";
    let new_session = "session-new-002";

    // Seed with one session via on_initial_state.
    on_initial_state(&mut app, vec![make_session(existing)], vec![], vec![], 0);
    assert!(
        app.pty_parsers.contains_key(existing),
        "precondition: existing session parser must be created by on_initial_state"
    );

    // Give the existing session some content to verify no-clobber.
    {
        let parser = app.pty_parsers.get_mut(existing).unwrap();
        parser.process(b"retain-me\r\n");
    }

    // Act: SessionListUpdate that adds new_session and retains existing.
    handle_server_message(
        &mut app,
        ServerToClient::SessionListUpdate {
            sessions: vec![make_session(existing), make_session(new_session)],
        },
    )
    .expect("handle_server_message must succeed for SessionListUpdate");

    // Assert A: new session now has a parser.
    assert!(
        app.pty_parsers.contains_key(new_session),
        "BC-2.09.001 F-S039-001: SessionListUpdate must create parser for new_session"
    );

    // Assert B: new session scroll offset is 0.
    assert_eq!(
        app.pty_scroll_offsets.get(new_session).copied(),
        Some(0),
        "BC-2.09.001 F-S039-001: pty_scroll_offsets[new_session] must be 0 after SessionListUpdate"
    );

    // Assert C (no-clobber): existing session parser content is preserved.
    let text = screen_text(app.pty_parsers.get(existing).unwrap());
    assert!(
        text.contains("retain-me"),
        "BC-2.09.001 F-S039-001 no-clobber: existing session parser must NOT be replaced \
         on SessionListUpdate. Got: {:?}",
        text
    );

    // Act 2: SessionListUpdate that removes existing (only new_session remains).
    handle_server_message(
        &mut app,
        ServerToClient::SessionListUpdate {
            sessions: vec![make_session(new_session)],
        },
    )
    .expect("handle_server_message must succeed for SessionListUpdate with removal");

    // Assert D: existing session was GC'd from all maps.
    assert!(
        !app.pty_parsers.contains_key(existing),
        "BC-2.09.001 F-S039-003 GC: pty_parsers must remove 'existing' when absent from roster"
    );
    assert!(
        !app.pty_scroll_offsets.contains_key(existing),
        "BC-2.09.001 F-S039-003 GC: pty_scroll_offsets must remove 'existing' when absent from roster"
    );

    // Assert E: new_session still present (not accidentally GC'd).
    assert!(
        app.pty_parsers.contains_key(new_session),
        "BC-2.09.001 F-S039-003 GC: new_session must NOT be GC'd (still in roster)"
    );
}

// ---------------------------------------------------------------------------
// Integration: SessionStateChanged::Terminated triggers GC
// BC-2.09.001 AC-008 / F-S039-003
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_session_terminated_gc
///
/// Exercises the production `handle_server_message → SessionStateChanged::Terminated`
/// dispatch path:
///   - Receiving Terminated for a session removes it from all PTY pipeline maps.
///   - pty_parsers, pty_scroll_offsets, pty_dump_received, dump_in_progress,
///     pending_pty_bytes all cleared for the terminated session.
///
/// This drives the PRODUCTION dispatch path (not a direct gc_pty_session call).
#[test]
fn test_BC_2_09_001_session_terminated_gc() {
    let session_id = "session-terminated-001";
    let mut app = make_app_with_session(session_id);

    // Pre-populate all GC-relevant maps.
    app.pty_dump_received.insert(session_id.to_string());
    app.dump_in_progress.insert(session_id.to_string(), false);
    app.pending_pty_bytes
        .insert(session_id.to_string(), vec![b"pending".to_vec()]);

    // Give the parser some content so we can verify it's truly removed.
    {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        parser.process(b"should-be-gc'd\r\n");
    }

    // Preconditions
    assert!(app.pty_parsers.contains_key(session_id));
    assert!(app.pty_scroll_offsets.contains_key(session_id));
    assert!(app.pty_dump_received.contains(session_id));
    assert!(app.dump_in_progress.contains_key(session_id));
    assert!(app.pending_pty_bytes.contains_key(session_id));

    // Act: send SessionStateChanged::Terminated through production dispatch.
    handle_server_message(
        &mut app,
        ServerToClient::SessionStateChanged {
            session_id: session_id.to_string(),
            new_state: SessionState::Terminated,
        },
    )
    .expect("handle_server_message must succeed for SessionStateChanged::Terminated");

    // Assert: all per-session PTY pipeline state removed.
    assert!(
        !app.pty_parsers.contains_key(session_id),
        "BC-2.09.001 F-S039-003 GC: pty_parsers must be removed on Terminated"
    );
    assert!(
        !app.pty_scroll_offsets.contains_key(session_id),
        "BC-2.09.001 F-S039-003 GC: pty_scroll_offsets must be removed on Terminated"
    );
    assert!(
        !app.pty_dump_received.contains(session_id),
        "BC-2.09.001 F-S039-003 GC: pty_dump_received must be removed on Terminated"
    );
    assert!(
        !app.dump_in_progress.contains_key(session_id),
        "BC-2.09.001 F-S039-003 GC: dump_in_progress must be removed on Terminated"
    );
    assert!(
        !app.pending_pty_bytes.contains_key(session_id),
        "BC-2.09.001 F-S039-003 GC: pending_pty_bytes must be removed on Terminated"
    );
}

// ---------------------------------------------------------------------------
// F-S039-004: Async attach rollback on send failure
// BC-2.09.001 AC-005 / enter_embedded_terminal error path
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_enter_embedded_rollback_on_send_failure
///
/// Exercises F-S039-004 (BC-2.09.001 Invariant 5 / AC-005 error path):
///   When the outbound IPC channel is closed (send returns Err), enter_embedded_terminal
///   MUST perform FULL ROLLBACK:
///   - dump_in_progress is NOT set (or is rolled back to absent/false)
///   - AppMode is NOT transitioned to EmbeddedTerminal
///
/// This tests the negative path: a broken IPC channel must not leave the app in a
/// half-entered EmbeddedTerminal mode where bytes buffer forever without a dump completing.
#[tokio::test]
async fn test_BC_2_09_001_enter_embedded_rollback_on_send_failure() {
    let session_id = "s1-rollback";
    let mut app = make_app_with_session(session_id);

    // Wire a channel and immediately DROP the receiver to simulate a closed channel.
    // When enter_embedded_terminal tries to send AttachSession, tx.send().await will
    // return Err (receiver dropped).
    let (cmd_tx, cmd_rx) = mpsc::channel::<ClientToServer>(1);
    drop(cmd_rx); // Close the channel receiver — all sends will fail.
    app.ipc_tx = Some(cmd_tx);

    // Record whether we were in Dashboard mode before the enter attempt.
    let was_dashboard_before = matches!(app.mode, AppMode::Dashboard { .. });
    assert!(
        was_dashboard_before,
        "precondition: mode must be Dashboard before enter attempt"
    );

    // Act: enter_embedded_terminal with a broken channel.
    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // Assert A (F-S039-004 rollback): dump_in_progress must NOT be true after send failure.
    // The correct behavior is: dump_in_progress.remove(&id) (rollback) on Err.
    let dip = app
        .dump_in_progress
        .get(session_id)
        .copied()
        .unwrap_or(false);
    assert!(
        !dip,
        "F-S039-004: dump_in_progress must be rolled back (false/absent) when AttachSession \
         send fails (channel closed). Got: {:?}",
        app.dump_in_progress.get(session_id)
    );

    // Assert B (F-S039-004 rollback): AppMode must NOT have transitioned to EmbeddedTerminal.
    assert!(
        !matches!(&app.mode, AppMode::EmbeddedTerminal { .. }),
        "F-S039-004: AppMode must NOT be EmbeddedTerminal after AttachSession send failure."
    );
}

// ---------------------------------------------------------------------------
// Integration: render_frame EmbeddedTerminal arm uses focused parser
// BC-2.09.001 AC-003 / Postcondition 3 — F-S039-002
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_render_frame_embedded_terminal_uses_focused_parser
///
/// Exercises the production `render_frame` dispatch for AppMode::EmbeddedTerminal:
///   - When mode is EmbeddedTerminal { session_id }, render_frame calls
///     render_embedded_terminal with app.pty_parsers[session_id].
///   - The focused parser's content appears in the rendered terminal buffer.
///   - Content from a different (non-focused) session does NOT appear.
///
/// This test drives the PRODUCTION render_frame path using TestBackend.
#[test]
fn test_BC_2_09_001_render_frame_embedded_terminal_uses_focused_parser() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let focused_id = "session-focused-render";
    let other_id = "session-other-render";

    let mut app = make_app_with_session(focused_id);
    // Add a second parser for another session (must NOT appear in the render).
    let other_parser = vt100::Parser::new(24, 80, 1000);
    app.pty_parsers.insert(other_id.to_string(), other_parser);
    app.pty_scroll_offsets.insert(other_id.to_string(), 0);

    // Feed distinguishable content to each parser.
    {
        let parser = app.pty_parsers.get_mut(focused_id).unwrap();
        parser.process(b"FOCUSED_CONTENT\r\n");
    }
    {
        let parser = app.pty_parsers.get_mut(other_id).unwrap();
        parser.process(b"OTHER_CONTENT\r\n");
    }

    // Set mode to EmbeddedTerminal for the focused session.
    app.mode = AppMode::EmbeddedTerminal {
        session_id: focused_id.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Build a headless ratatui terminal and run render_frame.
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");
    let mut sessions_state = SessionsPanelState::default();

    terminal
        .draw(|frame| {
            render_frame(&mut app, &mut sessions_state, frame);
        })
        .expect("render_frame must succeed in EmbeddedTerminal mode");

    // Assert: the rendered buffer contains "FOCUSED_CONTENT".
    // The PseudoTerminal widget renders the focused parser's screen into the terminal area.
    let buffer = terminal.backend().buffer().clone();
    let rendered: String = (0..80)
        .map(|col| buffer[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        rendered.contains("FOCUSED_CONTENT") || {
            // Check all rows for the content (content may not land on row 0 if layout
            // places the terminal area below a header row).
            let full_rendered: String = (0..24)
                .flat_map(|row| (0..80).map(move |col| (row, col)))
                .map(|(row, col)| buffer[(col, row)].symbol().to_string())
                .collect();
            full_rendered.contains("FOCUSED_CONTENT")
        },
        "BC-2.09.001 F-S039-002: render_frame in EmbeddedTerminal mode must render the \
         focused parser's content ('FOCUSED_CONTENT') into the terminal buffer. \
         Row 0: {:?}",
        rendered
    );

    // Assert: "OTHER_CONTENT" must NOT appear in the buffer (wrong parser).
    let full_buffer: String = (0..24)
        .flat_map(|row| (0..80).map(move |col| (row, col)))
        .map(|(row, col)| buffer[(col, row)].symbol().to_string())
        .collect();
    assert!(
        !full_buffer.contains("OTHER_CONTENT"),
        "BC-2.09.001 F-S039-002: render_frame must NOT render the non-focused parser's \
         content ('OTHER_CONTENT') — only the focused session_id parser is rendered"
    );
}

// ---------------------------------------------------------------------------
// F-S039-P2-001: ipc_tx == None rollback
// BC-2.09.001 Invariant 5 / enter_embedded_terminal offline guard
//
// Regression guard: the pre-fix code set dump_in_progress = true and then fell
// through to mode transition even when ipc_tx was None (daemon offline). This left
// the app permanently stuck in EmbeddedTerminal with dump_in_progress = true and
// no AttachSession ever in flight, so the dump could never complete and bytes
// buffered forever.
//
// Post-fix code (F-S039-P2-001): the `let Some(ref tx) = app.ipc_tx else { rollback;
// return; }` guard treats None identically to a send Err — dump_in_progress is
// removed and mode is NOT transitioned.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_enter_embedded_rollback_when_ipc_offline
///
/// Exercises F-S039-P2-001 (BC-2.09.001 Invariant 5):
///   When app.ipc_tx is None (daemon offline / channel not yet wired),
///   enter_embedded_terminal MUST perform FULL ROLLBACK:
///   - dump_in_progress is NOT true (removed / never set to true without rollback).
///   - AppMode is NOT transitioned to EmbeddedTerminal.
///
/// This test would FAIL against the pre-fix code that set dump_in_progress = true
/// (and attempted to transition mode) before checking whether ipc_tx was Some.
#[tokio::test]
async fn test_BC_2_09_001_enter_embedded_rollback_when_ipc_offline() {
    // Arrange: App with NO ipc_tx (daemon offline state).
    let session_id = "s1-p2-001-offline";
    let mut app = make_app_with_session(session_id);

    // Ensure ipc_tx is None — daemon offline, channel not wired.
    app.ipc_tx = None;

    // Precondition: session is NOT in pty_dump_received (would trigger the
    // auto-attach protocol path in enter_embedded_terminal).
    assert!(
        !app.pty_dump_received.contains(session_id),
        "precondition (F-S039-P2-001): session must not be in pty_dump_received"
    );

    // Precondition: mode is Dashboard before the enter attempt.
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "precondition (F-S039-P2-001): mode must be Dashboard before enter attempt"
    );

    // Act: enter_embedded_terminal with ipc_tx == None.
    // Pre-fix: would set dump_in_progress = true and potentially transition mode.
    // Post-fix: must detect None, rollback dump_in_progress, return early without mode change.
    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // Assert A (F-S039-P2-001): dump_in_progress must NOT be true.
    // Acceptable values: key absent, or key present with value false.
    // NOT acceptable: Some(true) — that would mean the flag was set and not rolled back.
    let dip = app
        .dump_in_progress
        .get(session_id)
        .copied()
        .unwrap_or(false);
    assert!(
        !dip,
        "F-S039-P2-001 (BC-2.09.001 Invariant 5): dump_in_progress must NOT be true when \
         ipc_tx is None — None path must rollback identically to send Err. \
         Got dump_in_progress[{}] = {:?}",
        session_id,
        app.dump_in_progress.get(session_id)
    );

    // Assert B (F-S039-P2-001): AppMode must NOT have transitioned to EmbeddedTerminal.
    // If mode is EmbeddedTerminal, the pre-fix code path executed and the session is
    // permanently stuck (dump never completes, bytes buffer forever).
    assert!(
        !matches!(&app.mode, AppMode::EmbeddedTerminal { .. }),
        "F-S039-P2-001 (BC-2.09.001 Invariant 5): AppMode must NOT be EmbeddedTerminal \
         when ipc_tx is None — mode transition must be skipped on offline rollback"
    );
}

// ---------------------------------------------------------------------------
// F-S039-P2-002: idempotency guard — spurious ScrollbackDumpComplete must not
// destroy a live parser
// BC-2.09.001 Invariant 5 / on_scrollback_dump_complete idempotency
//
// Regression guard: the pre-fix code unconditionally reset the parser on any
// ScrollbackDumpComplete message, regardless of whether dump_in_progress was
// true. A spurious, duplicate, cross-client, or post-detach completion would
// wipe a live populated parser — causing visible screen corruption.
//
// Post-fix code (F-S039-P2-002): the guard `if app.dump_in_progress.get(&session_id)
// != Some(&true) { return; }` drops all completions outside the dump window before
// any parser reset or replay occurs.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_scrollback_dump_complete_idempotency_guard
///
/// Exercises F-S039-P2-002 (BC-2.09.001 Invariant 5):
///   A ScrollbackDumpComplete that arrives when dump_in_progress is NOT true
///   (spurious / duplicate / post-detach) MUST be a no-op:
///   - The parser's existing content is NOT wiped.
///   - pty_dump_received is NOT updated for this session.
///
/// Setup: create a session whose parser already holds live content
/// ("LIVE-CONTENT"), with dump_in_progress absent / false (NOT in a dump window).
/// Drive on_scrollback_dump_complete via handle_server_message (production dispatch).
///
/// This test would FAIL against the pre-fix code that unconditionally called
/// `app.pty_parsers.insert(session_id, vt100::Parser::new(...))` on every
/// ScrollbackDumpComplete, destroying the live parser's content.
#[test]
fn test_BC_2_09_001_scrollback_dump_complete_idempotency_guard() {
    // Arrange: session with a live parser holding known content.
    let session_id = "s1-p2-002-idempotent";
    let mut app = make_app_with_session(session_id);

    // Feed live content to the parser so we can detect if it survives.
    {
        let parser = app
            .pty_parsers
            .get_mut(session_id)
            .expect("precondition: parser must exist");
        parser.process(b"LIVE-CONTENT\r\n");
    }

    // Verify live content is present before the spurious completion arrives.
    {
        let text = screen_text(app.pty_parsers.get(session_id).unwrap());
        assert!(
            text.contains("LIVE-CONTENT"),
            "precondition (F-S039-P2-002): parser must contain 'LIVE-CONTENT' before the \
             spurious ScrollbackDumpComplete — got: {:?}",
            text
        );
    }

    // Precondition: dump_in_progress is NOT true for this session (outside dump window).
    // Explicitly ensure it's false/absent — the spurious message should not be processed.
    // Use insert(false) to match the "exists but false" case (not just "absent").
    app.dump_in_progress.insert(session_id.to_string(), false);

    // Precondition: session is NOT in pty_dump_received before the spurious completion.
    assert!(
        !app.pty_dump_received.contains(session_id),
        "precondition (F-S039-P2-002): session must not be in pty_dump_received before \
         spurious ScrollbackDumpComplete"
    );

    // Act: drive a ScrollbackDumpComplete through the PRODUCTION dispatch path
    // (handle_server_message → on_scrollback_dump_complete).
    // This simulates a spurious, duplicate, or post-detach completion arriving
    // when no dump is in flight.
    handle_server_message(
        &mut app,
        ServerToClient::ScrollbackDumpComplete {
            session_id: session_id.to_string(),
            total_chunks: 0,
            cursor_row: 0,
            cursor_col: 0,
            pty_rows: 24,
            pty_cols: 80,
        },
    )
    .expect("handle_server_message must not panic on spurious ScrollbackDumpComplete");

    // Assert A (F-S039-P2-002): parser content must NOT have been wiped.
    // The pre-fix unconditional reset would replace the parser with a blank one,
    // so "LIVE-CONTENT" would be absent. The post-fix guard prevents the reset.
    let text_after = screen_text(
        app.pty_parsers
            .get(session_id)
            .expect("F-S039-P2-002: parser must still exist after spurious completion"),
    );
    assert!(
        text_after.contains("LIVE-CONTENT"),
        "F-S039-P2-002 (BC-2.09.001 Invariant 5): parser content must survive a spurious \
         ScrollbackDumpComplete outside the dump window — 'LIVE-CONTENT' must still be \
         present. Got: {:?}",
        text_after
    );

    // Assert B (F-S039-P2-002): pty_dump_received must NOT have been updated.
    // The no-op guard must return before the `pty_dump_received.insert` step.
    assert!(
        !app.pty_dump_received.contains(session_id),
        "F-S039-P2-002 (BC-2.09.001 Invariant 5): pty_dump_received must NOT be updated \
         by a spurious ScrollbackDumpComplete outside the dump window"
    );
}

// ---------------------------------------------------------------------------
// F-S039-P2-003: Terminated exits EmbeddedTerminal BEFORE GC
// BC-2.09.001 Invariant 5 / SessionStateChanged::Terminated ordering
//
// Regression guard: the pre-fix code called gc_pty_session without first
// checking whether the TUI was currently in EmbeddedTerminal mode for the
// terminating session. This left app.mode == EmbeddedTerminal { session_id }
// while all per-session parser state was removed — the next render would
// attempt to look up pty_parsers[session_id] and find nothing, causing a
// missing-parser trap (panic in unwrap or blank screen with no recovery path).
//
// Post-fix code (F-S039-P2-003): when SessionStateChanged::Terminated arrives
// and app.mode is EmbeddedTerminal for that session, exit_embedded_terminal is
// called FIRST (restores Dashboard mode and removes from pty_dump_received),
// THEN gc_pty_session removes all 5 per-session maps.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_terminated_session_exits_embedded_mode_before_gc
///
/// Exercises F-S039-P2-003 (BC-2.09.001 Invariant 5 / Ruling C):
///   When SessionStateChanged::Terminated arrives while app.mode is
///   EmbeddedTerminal for the terminating session:
///   1. app.mode is NOT EmbeddedTerminal after the message (mode exited to Dashboard).
///   2. pty_parsers does NOT contain the session_id (parser GC'd).
///
/// Both assertions must hold after a SINGLE handle_server_message call, and the
/// function must not panic. The ordering — mode exit BEFORE parser GC — is what
/// prevents a render trap in the same tick.
///
/// This test would FAIL against the pre-fix code that GC'd without exiting mode,
/// leaving app.mode == EmbeddedTerminal with pty_parsers[session_id] removed —
/// any subsequent render_frame call would find no parser for the embedded session.
#[test]
fn test_BC_2_09_001_terminated_session_exits_embedded_mode_before_gc() {
    // Arrange: app in EmbeddedTerminal for "s1" with a live parser present.
    let session_id = "s1-p2-003-terminated";
    let mut app = make_app_with_session(session_id);

    // Give the parser some content to confirm it gets GC'd (not just empty).
    {
        let parser = app.pty_parsers.get_mut(session_id).unwrap();
        parser.process(b"live-before-terminate\r\n");
    }

    // Pre-populate all PTY pipeline state for this session.
    app.pty_dump_received.insert(session_id.to_string());
    app.dump_in_progress.insert(session_id.to_string(), false);

    // Put app into EmbeddedTerminal mode for the session that will be terminated.
    app.mode = AppMode::EmbeddedTerminal {
        session_id: session_id.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Preconditions
    assert!(
        matches!(&app.mode, AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == session_id),
        "precondition (F-S039-P2-003): mode must be EmbeddedTerminal for the session"
    );
    assert!(
        app.pty_parsers.contains_key(session_id),
        "precondition (F-S039-P2-003): parser must exist before Terminated"
    );

    // Act: drive SessionStateChanged::Terminated through the PRODUCTION dispatch path.
    // Pre-fix: would GC parsers without exiting EmbeddedTerminal mode first — leaving
    // app.mode == EmbeddedTerminal with no parser, trapping the next render.
    // Post-fix: must exit EmbeddedTerminal (restoring Dashboard) THEN GC everything.
    handle_server_message(
        &mut app,
        ServerToClient::SessionStateChanged {
            session_id: session_id.to_string(),
            new_state: SessionState::Terminated,
        },
    )
    .expect("handle_server_message must not panic on SessionStateChanged::Terminated");

    // Assert A (F-S039-P2-003): app.mode must NOT be EmbeddedTerminal for the terminated session.
    // Acceptable: Dashboard (the exit_embedded_terminal path restores prior focus).
    // NOT acceptable: EmbeddedTerminal with session_id — that's the pre-fix trap.
    assert!(
        !matches!(&app.mode, AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == session_id),
        "F-S039-P2-003 (BC-2.09.001 Invariant 5): app.mode must NOT remain EmbeddedTerminal \
         for the terminated session — mode must exit BEFORE GC"
    );

    // Assert B (F-S039-P2-003): pty_parsers must NOT contain the session (GC'd).
    // This verifies GC still ran (the fix must not have accidentally skipped the GC step).
    assert!(
        !app.pty_parsers.contains_key(session_id),
        "F-S039-P2-003 (BC-2.09.001 Invariant 5): pty_parsers must NOT contain the terminated \
         session after handle_server_message — GC must have run"
    );
}
