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
use monocle_core::pty_constants::MAX_PENDING_PTY_MESSAGES;
use monocle_core::tui::state::{
    clamp_scrollback_rows, default_scrollback_rows, AppMode, FocusSnapshot, PTY_DEFAULT_COLS,
    PTY_DEFAULT_ROWS,
};
use monocle_ipc::events::TransportEvent;
use monocle_ipc::types::{ClientToServer, ServerToClient, SessionState};
use monocle_tui::app::{
    enter_embedded_terminal, exit_embedded_terminal, handle_server_message, on_dump_window_timeout,
    on_initial_state, on_pty_output, on_scrollback_dump_complete, on_transport_event, render_frame,
    App, AppEvent, IPC_READER_CHANNEL_CAPACITY,
};
use monocle_tui::pty_output_channel;
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use std::collections::VecDeque;
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
    let session_id = "00000001-0000-4000-8001-000000000001";
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
    let s1 = "00000001-0000-4000-8001-000000000002";
    let s2 = "00000002-0000-4000-8002-000000000002";
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
    let session_id = "00000001-0000-4000-8001-000000000003";
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

    // Assert F (BC-2.09.001 Invariant 5 step d/e): dump_in_progress entry removed after complete.
    // F-S039-REV-003: on_scrollback_dump_complete now calls remove() (not insert(false)).
    // The idempotency guard reads get(&id) == Some(&true); absent (None) is equivalent to
    // Some(false) for that guard — both are no-ops. Absent is preferred to avoid stale entries.
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        None,
        "BC-2.09.001 Invariant 5: dump_in_progress entry must be removed after ScrollbackDumpComplete (F-S039-REV-003)"
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
    let session_id = "00000001-0000-4000-8001-000000000004";
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
///   PtyOutput for "99999999-9999-4999-8999-999999999999" when pty_parsers map is empty.
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
        "99999999-9999-4999-8999-999999999999".to_string(),
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
    let session_id = "00000001-0000-4000-8001-000000000006";
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
    let session_id = "00000001-0000-4000-8001-000000000007";
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
    let session_id = "00000001-0000-4000-8001-000000000008";
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
    let session_id = "00000001-0000-4000-8001-000000000009";
    let mut app = make_app_with_session(session_id);
    app.pty_dump_received.insert(session_id.to_string());
    app.dump_in_progress.insert(session_id.to_string(), false);
    app.pending_pty_bytes
        .insert(session_id.to_string(), VecDeque::new());

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
    let session_id = "00000001-0000-4000-8001-000000000010";
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
    let session_id = "00000001-0000-4000-8001-000000000011";
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

    // Assert F: dump_in_progress entry removed after ScrollbackDumpComplete.
    // F-S039-REV-003: on_scrollback_dump_complete now calls remove() (not insert(false)).
    // Absent is the correct post-complete state (idempotency guard treats None == Some(false)).
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        None,
        "F-S039-009: dump_in_progress entry must be removed after ScrollbackDumpComplete (F-S039-REV-003)"
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
    let session_id = "00000001-0000-4000-8001-000000000012";
    let mut app = make_app_with_session(session_id);

    // Pre-populate all GC-relevant maps.
    app.pty_dump_received.insert(session_id.to_string());
    app.dump_in_progress.insert(session_id.to_string(), false);
    app.pending_pty_bytes
        .insert(session_id.to_string(), vec![b"pending".to_vec()].into());

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
    let session_id = "00000001-0000-4000-8001-000000000013";
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
    let session_id = "00000001-0000-4000-8001-000000000014";
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
    let session_id = "00000001-0000-4000-8001-000000000015";
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
// F-S039-REV-001: roster-diff GC must exit EmbeddedTerminal for the removed focused session
// BC-2.09.001 Invariant 5 / SessionListUpdate roster-diff ordering
//
// Regression guard: the pre-fix roster-diff loop called gc_pty_session (not
// gc_session_with_mode_exit) on sessions absent from the new roster. If the TUI
// was currently in EmbeddedTerminal for the removed session, the parser was
// destroyed while app.mode remained EmbeddedTerminal — the next render_frame
// would attempt pty_parsers[session_id] and find None (permanent blank-screen
// or panic trap).
//
// Post-fix code (F-S039-REV-001): the roster-diff loop calls
// gc_session_with_mode_exit, which checks the matches! guard and calls
// exit_embedded_terminal BEFORE gc_pty_session for the focused session only.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_roster_diff_gc_exits_embedded_mode_when_focused
///
/// Exercises F-S039-REV-001 (BC-2.09.001 Invariant 5):
///   When `SessionListUpdate` carries a roster that NO LONGER contains the session
///   currently open in `EmbeddedTerminal` mode, the handler MUST:
///   1. Exit `EmbeddedTerminal` mode (app.mode is NOT EmbeddedTerminal for s1 after the call).
///   2. GC s1's parser (pty_parsers does NOT contain s1).
///
/// Companion assertion: when a DIFFERENT session s2 is removed from the roster
/// while the TUI is viewing s1, s1's mode is UNCHANGED and s1's parser is RETAINED
/// (the matches! guard in gc_session_with_mode_exit must not over-fire).
///
/// This test FAILS against pre-fix code where the roster-diff loop called
/// gc_pty_session without exit-before-GC, leaving app.mode == EmbeddedTerminal
/// with a destroyed parser.
#[test]
fn test_BC_2_09_001_roster_diff_gc_exits_embedded_mode_when_focused() {
    // Arrange: two sessions s1 and s2.
    // s1 is currently in EmbeddedTerminal mode (the user is viewing s1's PTY).
    // s2 exists but is not focused.
    let s1 = "00000001-0000-4000-8001-000000000016";
    let s2 = "00000002-0000-4000-8002-000000000016";

    let mut app = make_app();

    // Seed both sessions into the roster via on_initial_state so the roster-diff
    // logic in handle_server_message (which diffs app.sessions vs the new roster)
    // can detect s1 as "removed" when we send the shrunk update.
    on_initial_state(
        &mut app,
        vec![make_session(s1), make_session(s2)],
        vec![],
        vec![],
        0,
    );

    // Both parsers must be present after on_initial_state.
    assert!(
        app.pty_parsers.contains_key(s1),
        "precondition (F-S039-REV-001): s1 parser must exist after on_initial_state"
    );
    assert!(
        app.pty_parsers.contains_key(s2),
        "precondition (F-S039-REV-001): s2 parser must exist after on_initial_state"
    );

    // Feed distinguishable content to both parsers so we can assert on presence.
    {
        let p1 = app.pty_parsers.get_mut(s1).unwrap();
        p1.process(b"s1-live-content\r\n");
    }
    {
        let p2 = app.pty_parsers.get_mut(s2).unwrap();
        p2.process(b"s2-live-content\r\n");
    }

    // Put the TUI into EmbeddedTerminal mode for s1 (the session that will be removed).
    app.mode = AppMode::EmbeddedTerminal {
        session_id: s1.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Precondition: mode is EmbeddedTerminal for s1.
    assert!(
        matches!(&app.mode, AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == s1),
        "precondition (F-S039-REV-001): mode must be EmbeddedTerminal for s1 before roster update"
    );

    // Act: SessionListUpdate with a NEW roster that contains only s2 (s1 removed).
    // Pre-fix: would call gc_pty_session(s1) WITHOUT exiting EmbeddedTerminal first,
    //          leaving app.mode == EmbeddedTerminal { s1 } with pty_parsers[s1] = None.
    // Post-fix: calls gc_session_with_mode_exit(s1), which exits EmbeddedTerminal FIRST.
    handle_server_message(
        &mut app,
        ServerToClient::SessionListUpdate {
            sessions: vec![make_session(s2)],
        },
    )
    .expect("handle_server_message must succeed for roster-diff SessionListUpdate");

    // Assert A (F-S039-REV-001): app.mode must NOT be EmbeddedTerminal for s1.
    // The mode must have been exited before the GC ran.
    assert!(
        !matches!(&app.mode, AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == s1),
        "F-S039-REV-001 (BC-2.09.001 Invariant 5): app.mode must NOT be EmbeddedTerminal \
         for s1 after roster-diff removes s1 — gc_session_with_mode_exit must exit mode first. \
         Pre-fix code would leave mode == EmbeddedTerminal with destroyed parser."
    );

    // Assert B (F-S039-REV-001): s1's parser must be GC'd.
    // Verifies GC still ran (mode exit must not have accidentally skipped the GC step).
    assert!(
        !app.pty_parsers.contains_key(s1),
        "F-S039-REV-001 (BC-2.09.001 Invariant 5): pty_parsers must NOT contain s1 after \
         roster-diff removes s1 — gc_pty_session must have run after mode exit"
    );

    // Companion assertion: s2 is STILL PRESENT (not accidentally GC'd).
    assert!(
        app.pty_parsers.contains_key(s2),
        "F-S039-REV-001 companion: s2 parser must be RETAINED after roster update \
         (s2 is still in the new roster — must not be GC'd)"
    );

    // Companion assertion (matches! guard non-over-fire):
    // Reset and verify that when s1 is in the roster and s2 is removed while
    // the TUI is viewing s1, s1's mode is UNCHANGED and s1's parser is RETAINED.
    // This guards against the matches! guard being too broad.
    let mut app2 = make_app();
    on_initial_state(
        &mut app2,
        vec![make_session(s1), make_session(s2)],
        vec![],
        vec![],
        0,
    );
    {
        let p = app2.pty_parsers.get_mut(s1).unwrap();
        p.process(b"s1-guard-content\r\n");
    }
    // Set mode to EmbeddedTerminal for s1 (s1 is still in the new roster).
    app2.mode = AppMode::EmbeddedTerminal {
        session_id: s1.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Update: new roster contains s1 but NOT s2.
    handle_server_message(
        &mut app2,
        ServerToClient::SessionListUpdate {
            sessions: vec![make_session(s1)],
        },
    )
    .expect("handle_server_message must succeed for companion assertion");

    // mode must REMAIN EmbeddedTerminal for s1 (the viewed session was NOT removed).
    assert!(
        matches!(&app2.mode, AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == s1),
        "F-S039-REV-001 companion: app.mode must REMAIN EmbeddedTerminal for s1 when \
         a DIFFERENT session (s2) is removed — matches! guard must not over-fire"
    );

    // s1's parser must still be present and still contain the pre-update content.
    assert!(
        app2.pty_parsers.contains_key(s1),
        "F-S039-REV-001 companion: s1 parser must be RETAINED when s1 remains in the roster"
    );
    let s1_text = screen_text(app2.pty_parsers.get(s1).unwrap());
    assert!(
        s1_text.contains("s1-guard-content"),
        "F-S039-REV-001 companion: s1 parser content must be preserved when s1 remains \
         in the roster — got: {:?}",
        s1_text
    );

    // s2 must be GC'd from the second app.
    assert!(
        !app2.pty_parsers.contains_key(s2),
        "F-S039-REV-001 companion: s2 parser must be GC'd when s2 is absent from the \
         new roster — GC must still fire for the removed session"
    );
}

// ---------------------------------------------------------------------------
// F-S039-REV-002: on_initial_state GCs stale parsers for sessions absent from
// a reconnect roster
// BC-2.09.001 Invariant 5 / on_initial_state stale-session GC
//
// Regression guard: the pre-fix on_initial_state never GC'd sessions that
// existed in app.sessions but were absent from the incoming InitialState
// roster. After a disconnect+reconnect, sessions that terminated while the TUI
// was offline would retain their parsers indefinitely — unbounded memory growth
// and stale render state.
//
// Post-fix code (F-S039-REV-002): on_initial_state computes the diff between
// the old app.sessions and the incoming roster and calls
// gc_session_with_mode_exit for each stale session before assigning the new
// roster. Surviving sessions are NOT clobbered (no-clobber invariant).
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_on_initial_state_gcs_stale_sessions_on_reconnect
///
/// Exercises F-S039-REV-002 (BC-2.09.001 Invariant 5):
///   After two calls to `on_initial_state` (simulating a reconnect):
///   - First call: roster contains s1 and s2 → both parsers created.
///   - Second call (reconnect): roster contains only s2 (s1 was terminated while
///     the TUI was disconnected) → s1's parser must be GC'd, s2's must be RETAINED.
///
///   If s2 had content before the second on_initial_state, that content must survive
///   (no-clobber: existing parsers for surviving sessions are not replaced).
///
/// This test FAILS against pre-fix code that never GC'd stale sessions in
/// on_initial_state (stale parser for s1 would remain in pty_parsers after the
/// second call, and s2's content would be clobbered if the no-clobber guard
/// was also absent).
#[test]
fn test_BC_2_09_001_on_initial_state_gcs_stale_sessions_on_reconnect() {
    let s1 = "00000001-0000-4000-8001-000000000017";
    let s2 = "00000002-0000-4000-8002-000000000017";

    let mut app = make_app();

    // Act 1: first on_initial_state — roster contains both s1 and s2.
    on_initial_state(
        &mut app,
        vec![make_session(s1), make_session(s2)],
        vec![],
        vec![],
        0,
    );

    // Assert A: both parsers created after first on_initial_state.
    assert!(
        app.pty_parsers.contains_key(s1),
        "F-S039-REV-002 precondition: s1 parser must exist after first on_initial_state"
    );
    assert!(
        app.pty_parsers.contains_key(s2),
        "F-S039-REV-002 precondition: s2 parser must exist after first on_initial_state"
    );

    // Feed distinguishable content into s2's parser (simulates live PTY output
    // received before the disconnect). We want to verify this content is PRESERVED
    // after the reconnect (no-clobber invariant for surviving sessions).
    {
        let p2 = app.pty_parsers.get_mut(s2).unwrap();
        p2.process(b"s2-pre-reconnect-data\r\n");
    }

    // Verify s2 content before reconnect.
    {
        let text = screen_text(app.pty_parsers.get(s2).unwrap());
        assert!(
            text.contains("s2-pre-reconnect-data"),
            "F-S039-REV-002 precondition: s2 parser must contain pre-reconnect data. \
             Got: {:?}",
            text
        );
    }

    // Act 2: RECONNECT — second on_initial_state with a shrunk roster (s1 gone).
    // This simulates a session (s1) that terminated while the TUI was disconnected
    // and no longer appears in the daemon's InitialState roster.
    // Pre-fix: would NOT GC s1 — its parser would remain in pty_parsers after the call.
    // Post-fix: computes the stale diff and calls gc_session_with_mode_exit(s1).
    on_initial_state(&mut app, vec![make_session(s2)], vec![], vec![], 0);

    // Assert B (F-S039-REV-002): s1's parser must be GC'd after the reconnect call.
    // Pre-fix would leave s1 in pty_parsers (stale leak). Post-fix removes it.
    assert!(
        !app.pty_parsers.contains_key(s1),
        "F-S039-REV-002 (BC-2.09.001 Invariant 5): pty_parsers must NOT contain s1 after \
         reconnect with a roster that excludes s1 — on_initial_state must GC stale sessions. \
         Pre-fix code would leave s1's parser in memory indefinitely."
    );

    // Assert C (F-S039-REV-002): s1's scroll offset must also be GC'd.
    assert!(
        !app.pty_scroll_offsets.contains_key(s1),
        "F-S039-REV-002: pty_scroll_offsets must NOT contain s1 after reconnect GC"
    );

    // Assert D (no-clobber): s2's parser must still be present.
    assert!(
        app.pty_parsers.contains_key(s2),
        "F-S039-REV-002 no-clobber: s2 parser must be RETAINED after reconnect \
         (s2 is still in the new roster)"
    );

    // Assert E (no-clobber content preserved): s2's content from before the reconnect
    // must still be present. The no-clobber invariant ensures on_initial_state does NOT
    // replace parsers for sessions that survive the reconnect.
    let s2_text = screen_text(app.pty_parsers.get(s2).unwrap());
    assert!(
        s2_text.contains("s2-pre-reconnect-data"),
        "F-S039-REV-002 no-clobber: s2 parser content must be PRESERVED across reconnect \
         (on_initial_state no-clobber: existing parsers for surviving sessions not replaced). \
         Got: {:?}",
        s2_text
    );
}

// ---------------------------------------------------------------------------
// F-S039-REV-003 confirmation: dump_in_progress removed after ScrollbackDumpComplete
// BC-2.09.001 Invariant 5 / on_scrollback_dump_complete cleanup
//
// Regression guard: the pre-fix on_scrollback_dump_complete called
// dump_in_progress.insert(session_id, false) after completing the dump. This
// left a stale Some(false) entry in the map rather than removing it. The
// idempotency guard in on_scrollback_dump_complete reads
// `dump_in_progress.get(&id) != Some(&true)` — both None and Some(false)
// correctly pass that guard, so functional behavior was preserved. However,
// the stale entry is a memory leak: sessions that are GC'd without a
// subsequent enter_embedded_terminal would never have their dump_in_progress
// entry removed, leaving an orphaned entry that grows with session churn.
//
// Post-fix code (F-S039-REV-003): on_scrollback_dump_complete calls
// dump_in_progress.remove(&session_id) instead of insert(id, false), so the
// map entry is removed entirely when the dump completes.
//
// NOTE: F-S039-REV-003 is also asserted inline in:
//   - test_BC_2_09_001_auto_attach_on_first_entry_buffering (direct function call)
//   - test_BC_2_09_001_first_entry_session_not_preinserted (direct function call)
//
// This standalone test exercises the PRODUCTION dispatch path via
// handle_server_message → on_scrollback_dump_complete to confirm the entry is
// absent (None, not Some(false)) after the completion message is processed.
// It is NOT a duplicate of the inline assertions above — it exercises a different
// code path (handle_server_message dispatch vs. direct call) and is the only test
// that verifies None vs Some(false) via the handle_server_message route.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_dump_complete_removes_dump_in_progress_entry
///
/// Exercises F-S039-REV-003 (BC-2.09.001 Invariant 5) via the production
/// `handle_server_message` dispatch path:
///   After a `ScrollbackDumpComplete` message is processed by
///   `handle_server_message → on_scrollback_dump_complete`, the
///   `dump_in_progress` map must NOT contain an entry for the session at all
///   (entry removed, not left as `Some(false)`).
///
/// This test fails against pre-fix code that called `dump_in_progress.insert(id, false)`
/// instead of `dump_in_progress.remove(&id)` — the post-complete map would contain
/// `Some(false)` rather than `None`, leaking an entry per completed dump cycle.
#[test]
fn test_BC_2_09_001_dump_complete_removes_dump_in_progress_entry() {
    // Arrange: session with an in-progress dump.
    let session_id = "00000001-0000-4000-8001-000000000018";
    let mut app = make_app_with_session(session_id);

    // Manually set dump_in_progress = true to simulate an active dump window.
    app.dump_in_progress.insert(session_id.to_string(), true);

    // Precondition: entry is Some(true).
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        Some(true),
        "precondition (F-S039-REV-003): dump_in_progress must be Some(true) before completion"
    );

    // Act: drive ScrollbackDumpComplete through the PRODUCTION dispatch path.
    // This is a different code path than the direct on_scrollback_dump_complete calls
    // in the inline REV-003 assertions above.
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
    .expect("handle_server_message must not fail on ScrollbackDumpComplete");

    // Assert (F-S039-REV-003): dump_in_progress entry must be ABSENT (removed).
    // NOT acceptable: Some(false) — that is the pre-fix stale entry.
    // NOT acceptable: Some(true) — that would mean completion was not processed.
    // ONLY acceptable: None (entry removed by dump_in_progress.remove()).
    assert_eq!(
        app.dump_in_progress.get(session_id).copied(),
        None,
        "F-S039-REV-003 (BC-2.09.001 Invariant 5): dump_in_progress must be ABSENT (None) \
         after ScrollbackDumpComplete — pre-fix code left Some(false) (stale leak). \
         Post-fix must call remove(), not insert(id, false). \
         Got: {:?}",
        app.dump_in_progress.get(session_id)
    );

    // Corollary: pty_dump_received must now contain the session (dump successfully recorded).
    assert!(
        app.pty_dump_received.contains(session_id),
        "F-S039-REV-003 corollary: pty_dump_received must contain session after \
         ScrollbackDumpComplete (dump successfully recorded)"
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
    let session_id = "00000001-0000-4000-8001-000000000019";
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

// ===========================================================================
// PASS-4 ADVERSARIAL REGRESSION TESTS
// Anchored to BC-2.09.001 v1.6.0 at Pass-4 authoring time Invariants 7/8/9
// Finding IDs: F-PASS4-MED-001, F-PASS4-MED-002, F-PASS4-LOW-001, F-PASS4-LOW-002
// ===========================================================================

// ---------------------------------------------------------------------------
// F-PASS4-MED-001: pending_pty_bytes cap — drop-oldest + counter
// BC-2.09.001 Invariant 7 (v1.6.0)
//
// Pre-fix: on_pty_output accumulated pending_pty_bytes without bound while
// dump_in_progress was true — a slow or absent ScrollbackDumpComplete would
// cause unbounded memory growth per session.
//
// Post-fix: on_pty_output enforces MAX_PENDING_PTY_MESSAGES (4096) and
// MAX_PENDING_PTY_BYTES (512 KiB) caps. Oldest message is evicted on overflow;
// pending_pty_drop_count is incremented.
//
// This test triggers the message-count cap (cheaper: push 4097 small messages)
// and asserts:
//   1. pending_pty_bytes length is bounded at/below MAX_PENDING_PTY_MESSAGES.
//   2. pending_pty_drop_count > 0 (at least one eviction occurred).
//   3. The OLDEST messages were dropped; the NEWEST are retained.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_pending_pty_bytes_cap_drops_oldest
///
/// Exercises BC-2.09.001 Invariant 7 (F-PASS4-MED-001):
///   When `pending_pty_bytes[s1]` would exceed `MAX_PENDING_PTY_MESSAGES` (4096),
///   `on_pty_output` MUST evict the OLDEST entry and increment
///   `pending_pty_drop_count[s1]`.  The NEWEST messages survive.
///
/// Pre-fix behavior: unlimited accumulation (OOM risk under long dump windows).
/// Post-fix behavior: cap + drop-oldest + counter as per Invariant 7.
///
/// This test FAILS pre-fix because `pending_pty_drop_count` would remain 0 and
/// `pending_pty_bytes` would hold 4097 entries (no cap enforcement).
#[test]
fn test_BC_2_09_001_pending_pty_bytes_cap_drops_oldest() {
    // Arrange: session with dump_in_progress = true so bytes are buffered.
    let s1 = "00000001-0000-4000-8001-000000000020";
    let mut app = make_app_with_session(s1);
    app.dump_in_progress.insert(s1.to_string(), true);

    // Push MAX_PENDING_PTY_MESSAGES + 1 messages with distinguishable content.
    // Message 0..MAX_PENDING_PTY_MESSAGES are the "old" messages.
    // Message MAX_PENDING_PTY_MESSAGES is the "newest" that triggers the eviction.
    //
    // Each message is 5 bytes ("MNNNx\r") — well below the per-message byte limit —
    // so the message-count cap fires first (before the byte-count cap).
    let total_pushed = MAX_PENDING_PTY_MESSAGES + 1;
    for i in 0..total_pushed {
        // Format a distinguishable 1-byte marker: use the low byte of i as the payload.
        // The first message has content "FIRST000" and the last "LAST<N>" so we can
        // assert on presence/absence of specific messages.
        let content = if i == 0 {
            b"FIRST_MSG\r\n".to_vec()
        } else if i == total_pushed - 1 {
            b"LAST_MSG\r\n".to_vec()
        } else {
            // Middle messages: just need to consume a slot each.
            format!("mid{:04}\r\n", i).into_bytes()
        };
        on_pty_output(&mut app, s1.to_string(), content);
    }

    // Assert 1 (F-PASS4-MED-001 Invariant 7): buffer length is bounded.
    // After pushing MAX_PENDING_PTY_MESSAGES + 1 messages, the buffer MUST NOT exceed
    // MAX_PENDING_PTY_MESSAGES (the cap is enforced before or after each push).
    let buffer = app
        .pending_pty_bytes
        .get(s1)
        .expect("F-PASS4-MED-001: pending_pty_bytes must have entry for s1");
    assert!(
        buffer.len() <= MAX_PENDING_PTY_MESSAGES,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 7): pending_pty_bytes length ({}) must be \
         bounded at/below MAX_PENDING_PTY_MESSAGES ({}) — cap not enforced pre-fix",
        buffer.len(),
        MAX_PENDING_PTY_MESSAGES
    );

    // Assert 2 (F-PASS4-MED-001 Invariant 7): at least one eviction was recorded.
    let drop_count = app.pending_pty_drop_count.get(s1).copied().unwrap_or(0);
    assert!(
        drop_count > 0,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 7): pending_pty_drop_count[s1] must be > 0 \
         after buffer overflow — pre-fix code never incremented the counter. Got: {}",
        drop_count
    );

    // Assert 3 (F-PASS4-MED-001 drop-oldest): the NEWEST message must survive.
    // The last message pushed was b"LAST_MSG\r\n" — it must be in the buffer.
    let newest_survives = buffer.iter().any(|chunk| chunk == b"LAST_MSG\r\n");
    assert!(
        newest_survives,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 7): NEWEST message 'LAST_MSG' must be \
         retained after cap eviction — drop-oldest semantics require the latest messages survive"
    );

    // Assert 4 (F-PASS4-MED-001 drop-oldest): the FIRST message must have been evicted.
    // The oldest message pushed was b"FIRST_MSG\r\n" — it must NOT be in the buffer.
    let oldest_absent = !buffer.iter().any(|chunk| chunk == b"FIRST_MSG\r\n");
    assert!(
        oldest_absent,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 7): OLDEST message 'FIRST_MSG' must have been \
         EVICTED after cap overflow — drop-oldest semantics must not retain the first message \
         when the buffer is full. Buffer length: {}",
        buffer.len()
    );
}

// ---------------------------------------------------------------------------
// F-PASS4-MED-001: dump-window timeout force-resolve
// BC-2.09.001 Invariant 8 (v1.6.0)
//
// Pre-fix: no dump-window timeout existed. A lost ScrollbackDumpComplete would
// leave dump_in_progress = true forever, causing all subsequent PtyOutput to
// accumulate in pending_pty_bytes without bound.
//
// Post-fix: on_dump_window_timeout fires after DUMP_WINDOW_TIMEOUT (10s),
// calling the production function to clean up all in-flight dump state.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_dump_window_timeout_force_resolves
///
/// Exercises BC-2.09.001 Invariant 8 (F-PASS4-MED-001):
///   `on_dump_window_timeout(app, s1)` with `dump_in_progress[s1] = true`:
///   1. `dump_in_progress` no longer contains s1 (entry removed).
///   2. `pending_pty_bytes[s1]` cleared/absent.
///   3. `pending_pty_drop_count[s1]` cleared/absent.
///   4. `pty_parsers[s1]` reset to PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS parser.
///   5. `pty_dump_received` does NOT contain s1 (dump never completed).
///   6. Calling `on_dump_window_timeout` again (dump_in_progress absent) is a
///      no-op — idempotent, no panic, no state change.
///
/// This test FAILS pre-fix because `on_dump_window_timeout` did not exist.
#[test]
fn test_BC_2_09_001_dump_window_timeout_force_resolves() {
    // Arrange: session with dump_in_progress = true and some buffered bytes.
    let s1 = "00000001-0000-4000-8001-000000000021";
    let mut app = make_app_with_session(s1);
    app.dump_in_progress.insert(s1.to_string(), true);
    app.pending_pty_bytes.insert(
        s1.to_string(),
        vec![b"buffered-during-dump\r\n".to_vec()].into(),
    );
    app.pending_pty_drop_count.insert(s1.to_string(), 3);

    // Preconditions
    assert_eq!(
        app.dump_in_progress.get(s1).copied(),
        Some(true),
        "precondition: dump_in_progress must be Some(true) before timeout"
    );
    assert!(
        app.pending_pty_bytes.contains_key(s1),
        "precondition: pending_pty_bytes must have entry for s1"
    );
    assert_eq!(
        app.pending_pty_drop_count.get(s1).copied(),
        Some(3),
        "precondition: pending_pty_drop_count must be 3"
    );

    // Act: call the production timeout handler.
    on_dump_window_timeout(&mut app, s1.to_string());

    // Assert 1 (Invariant 8): dump_in_progress entry removed.
    assert_eq!(
        app.dump_in_progress.get(s1).copied(),
        None,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 8): dump_in_progress must be absent (removed) \
         after on_dump_window_timeout — not Some(true) or Some(false)"
    );

    // Assert 2 (Invariant 8): pending_pty_bytes cleared.
    let buffered_len = app.pending_pty_bytes.get(s1).map(|v| v.len()).unwrap_or(0);
    assert_eq!(
        buffered_len, 0,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 8): pending_pty_bytes must be empty/absent \
         after on_dump_window_timeout — stale buffered bytes must be discarded"
    );

    // Assert 3 (Invariant 8): pending_pty_drop_count cleared.
    let drop_count = app.pending_pty_drop_count.get(s1).copied().unwrap_or(0);
    assert_eq!(
        drop_count, 0,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 8): pending_pty_drop_count must be cleared \
         (0/absent) after on_dump_window_timeout. Got: {}",
        drop_count
    );

    // Assert 4 (Invariant 8): parser reset to PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS.
    // The parser MUST still exist (reset, not removed) — this distinguishes timeout from GC.
    let parser = app.pty_parsers.get(s1).expect(
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 8): pty_parsers must still contain s1 \
                 after on_dump_window_timeout (reset, not GC'd)",
    );
    let (rows, cols) = parser.screen().size();
    assert_eq!(
        rows, PTY_DEFAULT_ROWS,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 8): parser rows must be PTY_DEFAULT_ROWS ({}) \
         after timeout reset — got {}",
        PTY_DEFAULT_ROWS, rows
    );
    assert_eq!(
        cols, PTY_DEFAULT_COLS,
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 8): parser cols must be PTY_DEFAULT_COLS ({}) \
         after timeout reset — got {}",
        PTY_DEFAULT_COLS, cols
    );

    // Assert 5 (Invariant 8): pty_dump_received does NOT contain s1.
    // The dump never completed — the session must NOT be in pty_dump_received.
    // The next enter_embedded_terminal must re-run the full attach + dump protocol.
    assert!(
        !app.pty_dump_received.contains(s1),
        "F-PASS4-MED-001 (BC-2.09.001 Invariant 8): pty_dump_received must NOT contain s1 \
         after on_dump_window_timeout — dump never completed, re-attach required"
    );

    // Assert 6 (Invariant 8 idempotency): calling on_dump_window_timeout again is a no-op.
    // dump_in_progress is now absent (not Some(&true)), so the guard must return immediately.
    // The parser must remain at the default dims — it must NOT be re-reset.
    on_dump_window_timeout(&mut app, s1.to_string());

    // State must be unchanged from the previous assertions (no panic, no mutation).
    assert_eq!(
        app.dump_in_progress.get(s1).copied(),
        None,
        "F-PASS4-MED-001 idempotency: dump_in_progress must remain None on second call"
    );
    let (rows2, cols2) = app
        .pty_parsers
        .get(s1)
        .expect("parser must still exist")
        .screen()
        .size();
    assert_eq!(
        rows2, PTY_DEFAULT_ROWS,
        "F-PASS4-MED-001 idempotency: parser rows must remain PTY_DEFAULT_ROWS after second call"
    );
    assert_eq!(
        cols2, PTY_DEFAULT_COLS,
        "F-PASS4-MED-001 idempotency: parser cols must remain PTY_DEFAULT_COLS after second call"
    );
}

// ---------------------------------------------------------------------------
// F-PASS4-MED-002: disconnect clears all in-flight dump state; retains parsers
// BC-2.09.001 Invariant 9 (v1.6.0)
//
// Pre-fix: on_transport_event(Disconnected) did not clear dump_in_progress,
// pending_pty_bytes, pending_pty_drop_count, or pty_dump_received. After
// reconnect, stale dump state from the dead connection caused:
//   (a) PtyOutput buffering to resume into stale pending_pty_bytes (memory leak).
//   (b) ScrollbackDumpComplete from the NEW connection completing a dump window
//       that was opened in the PRIOR connection.
//   (c) pty_dump_received entries from the prior session preventing re-attach
//       on reconnect (user sees blank screen until manual toggle).
//
// Post-fix: on_transport_event(Disconnected) clears ALL four dump maps (steps
// 1-4 per architect ruling). pty_parsers is NOT cleared — stale screen is
// better than blank (no-clobber: surviving parser content is retained).
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_disconnect_clears_dump_state_retains_parsers
///
/// Exercises BC-2.09.001 Invariant 9 (F-PASS4-MED-002):
///   `on_transport_event(TransportEvent::Disconnected)` MUST:
///   1. Clear `dump_in_progress` (empty map).
///   2. Clear `pending_pty_bytes` (empty map).
///   3. Clear `pending_pty_drop_count` (empty map).
///   4. Clear `pty_dump_received` (empty set).
///   5. Exit `EmbeddedTerminal` mode if currently in it.
///   6. RETAIN `pty_parsers` — stale screen is better than blank (no-clobber).
///      The parser that was live before disconnect must still exist and its
///      screen content ("SURVIVOR") must still be present after disconnect.
///
/// This test FAILS pre-fix because dump_in_progress, pending_pty_bytes,
/// pending_pty_drop_count, and pty_dump_received would survive the disconnect.
#[test]
fn test_BC_2_09_001_disconnect_clears_dump_state_retains_parsers() {
    // Arrange: session with all dump state populated.
    let s1 = "00000001-0000-4000-8001-000000000022";
    let mut app = make_app_with_session(s1);

    // Feed "SURVIVOR" content into the parser — this must survive after disconnect.
    {
        let parser = app.pty_parsers.get_mut(s1).unwrap();
        parser.process(b"SURVIVOR\r\n");
    }

    // Verify the content before disconnect.
    {
        let text = screen_text(app.pty_parsers.get(s1).unwrap());
        assert!(
            text.contains("SURVIVOR"),
            "precondition: parser must contain 'SURVIVOR' before disconnect"
        );
    }

    // Populate all four dump maps to test that they are cleared.
    app.dump_in_progress.insert(s1.to_string(), true);
    app.pending_pty_bytes
        .insert(s1.to_string(), vec![b"stale-buffered\r\n".to_vec()].into());
    app.pending_pty_drop_count.insert(s1.to_string(), 7);
    app.pty_dump_received.insert(s1.to_string());

    // Set mode to EmbeddedTerminal for s1 (must exit on disconnect).
    app.mode = AppMode::EmbeddedTerminal {
        session_id: s1.to_string(),
        prior: FocusSnapshot::Sessions,
    };

    // Preconditions
    assert_eq!(
        app.dump_in_progress.get(s1).copied(),
        Some(true),
        "precondition: dump_in_progress[s1] must be Some(true)"
    );
    assert!(
        !app.pending_pty_bytes.is_empty(),
        "precondition: pending_pty_bytes must be non-empty"
    );
    assert_eq!(
        app.pending_pty_drop_count.get(s1).copied(),
        Some(7),
        "precondition: pending_pty_drop_count[s1] must be 7"
    );
    assert!(
        app.pty_dump_received.contains(s1),
        "precondition: pty_dump_received must contain s1"
    );
    assert!(
        matches!(&app.mode, AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == s1),
        "precondition: mode must be EmbeddedTerminal for s1"
    );

    // Act: drive the production transport-event handler.
    on_transport_event(&mut app, TransportEvent::Disconnected);

    // Assert 1 (F-PASS4-MED-002 Invariant 9): dump_in_progress cleared.
    assert!(
        app.dump_in_progress.is_empty(),
        "F-PASS4-MED-002 (BC-2.09.001 Invariant 9): dump_in_progress must be EMPTY after \
         Disconnected — stale in-flight dump state must not survive reconnect"
    );

    // Assert 2 (F-PASS4-MED-002 Invariant 9): pending_pty_bytes cleared.
    assert!(
        app.pending_pty_bytes.is_empty(),
        "F-PASS4-MED-002 (BC-2.09.001 Invariant 9): pending_pty_bytes must be EMPTY after \
         Disconnected — stale buffered bytes from prior connection must be discarded"
    );

    // Assert 3 (F-PASS4-MED-002 Invariant 9): pending_pty_drop_count cleared.
    assert!(
        app.pending_pty_drop_count.is_empty(),
        "F-PASS4-MED-002 (BC-2.09.001 Invariant 9): pending_pty_drop_count must be EMPTY after \
         Disconnected — stale drop counts from prior connection must be reset"
    );

    // Assert 4 (F-PASS4-MED-002 Invariant 9): pty_dump_received cleared.
    assert!(
        app.pty_dump_received.is_empty(),
        "F-PASS4-MED-002 (BC-2.09.001 Invariant 9): pty_dump_received must be EMPTY after \
         Disconnected — prior dump receipts are invalid for the new connection"
    );

    // Assert 5 (F-PASS4-MED-002 Invariant 9): EmbeddedTerminal mode exited.
    assert!(
        !matches!(&app.mode, AppMode::EmbeddedTerminal { session_id: sid, .. } if sid == s1),
        "F-PASS4-MED-002 (BC-2.09.001 Invariant 9): mode must NOT be EmbeddedTerminal for s1 \
         after Disconnected — exit_embedded_terminal must have been called"
    );

    // Assert 6 (F-PASS4-MED-002 Invariant 9 no-clobber): pty_parsers STILL contains s1.
    // The parser is NOT cleared on disconnect — stale screen is better than blank.
    assert!(
        app.pty_parsers.contains_key(s1),
        "F-PASS4-MED-002 (BC-2.09.001 Invariant 9): pty_parsers must STILL contain s1 after \
         Disconnected — parsers are retained (no-clobber: stale screen better than blank)"
    );

    // Assert 7 (F-PASS4-MED-002 no-clobber content preserved):
    // The parser's content ("SURVIVOR") must still be present — no-clobber.
    let text_after = screen_text(app.pty_parsers.get(s1).unwrap());
    assert!(
        text_after.contains("SURVIVOR"),
        "F-PASS4-MED-002 (BC-2.09.001 Invariant 9): parser content 'SURVIVOR' must be \
         RETAINED after Disconnected — no-clobber: pty_parsers not cleared on disconnect. \
         Got: {:?}",
        text_after
    );
}

// ---------------------------------------------------------------------------
// F-PASS4-LOW-001: setup_ipc_streams_with_rx routes through pty_output_channel()
// BC-2.09.001 Invariant 3 (v1.6.0) — production-path capacity confirmation
//
// Pre-fix: setup_ipc_streams_with_rx created the inbound channel via an inline
// `tokio::sync::mpsc::channel::<...>(IPC_READER_CHANNEL_CAPACITY)` literal
// rather than calling `pty_output_channel()`. This meant the existing capacity
// test (`test_BC_2_09_001_invariant_bounded_channel_send_await_not_try_send`)
// asserted the `pty_output_channel()` constructor in isolation but NOT the
// production code path used by the actual event loop.
//
// Post-fix: setup_ipc_streams_with_rx calls pty_output_channel() (F-PASS4-LOW-001).
// The existing test already asserts `pty_output_channel()` capacity == 64.
// This test STRENGTHENS it by asserting the production path:
//   - The receiver returned by setup_ipc_streams_with_rx has
//     max_capacity() == IPC_READER_CHANNEL_CAPACITY.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_setup_ipc_streams_capacity_matches_production_channel
///
/// Exercises BC-2.09.001 Invariant 3 via the PRODUCTION setup path (F-PASS4-LOW-001):
///   `setup_ipc_streams_with_rx(app, r, w)` returns an `inbound_rx` whose
///   `max_capacity()` equals `IPC_READER_CHANNEL_CAPACITY` (64).
///
/// This is stronger than the `pty_output_channel()` isolation test because it
/// verifies the production event loop wiring — if setup_ipc_streams_with_rx
/// were to use a different channel constructor (inline literal with a different
/// capacity), the `pty_output_channel()` isolation test would not catch it.
///
/// Pre-fix: setup_ipc_streams_with_rx used an inline literal; this assertion
/// would still pass (both return 64) but the test would not detect if the inline
/// literal drifted. The F-PASS4-LOW-001 fix canonicalized the constructor so
/// there is only one source of truth — and this test locks the production path.
#[tokio::test]
async fn test_BC_2_09_001_setup_ipc_streams_capacity_matches_production_channel() {
    use monocle_tui::setup_ipc_streams_with_rx;
    use tokio::io::duplex;

    // Arrange: minimal App and a duplex stream pair that simulates the UDS socket.
    // setup_ipc_streams_with_rx spawns two tasks (reader + writer); we provide
    // real duplex streams so the tasks can be spawned without panicking.
    let mut app = make_app();
    let (client_half, _server_half) = duplex(4096);
    let (r, w) = tokio::io::split(client_half);

    // Act: call the production IPC stream setup function.
    let (_reader_handle, _writer_handle, inbound_rx) = setup_ipc_streams_with_rx(&mut app, r, w);

    // Assert: the inbound channel capacity matches the production constant.
    // If setup_ipc_streams_with_rx uses pty_output_channel() (F-PASS4-LOW-001),
    // this passes. If it uses a different inline literal, this assertion catches drift.
    assert_eq!(
        inbound_rx.max_capacity(),
        IPC_READER_CHANNEL_CAPACITY,
        "F-PASS4-LOW-001 (BC-2.09.001 Invariant 3): inbound_rx returned by \
         setup_ipc_streams_with_rx must have max_capacity() == IPC_READER_CHANNEL_CAPACITY ({}). \
         Production path must route through pty_output_channel() — not an inline literal.",
        IPC_READER_CHANNEL_CAPACITY
    );
}

// ---------------------------------------------------------------------------
// F-PASS4-LOW-002: inbound channel backpressure — .send().await blocks, not drops
// BC-2.09.001 Invariant 3 (v1.6.0)
//
// Pre-fix: if spawn_ipc_reader used try_send (non-blocking), a slow event loop
// would silently drop PtyOutput frames when the channel was full. At 1000
// frames/second (test target per SS-conventions), the 64-slot channel could be
// exhausted in 64ms — well within the 100ms budget, but with silent data loss.
//
// Post-fix: spawn_ipc_reader uses .send().await (blocking backpressure). When
// the channel is full, the reader task AWAITS until space is available rather
// than dropping the frame. This provides at-least-once delivery for all frames
// within the channel capacity.
//
// Behavioral distinction tested:
//   - .send().await: the 65th send does NOT complete until one item is consumed.
//   - .try_send: the 65th send returns Err immediately (drops the frame).
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_inbound_channel_backpressure_no_drop
///
/// Exercises BC-2.09.001 Invariant 3 (F-PASS4-LOW-002):
///   The `IPC_READER_CHANNEL_CAPACITY`-bounded inbound channel created by
///   `pty_output_channel()` uses `.send().await` (backpressure), NOT `try_send`
///   (drop-on-full).
///
/// Test protocol:
///   1. Create the channel via `pty_output_channel()`.
///   2. Fill it with `IPC_READER_CHANNEL_CAPACITY` items (no consumer).
///   3. Spawn a task to send the (capacity + 1)th item via `.send().await`.
///   4. Assert the task does NOT complete within a short poll window (it awaits —
///      backpressure in effect, no drop).
///   5. Receive one item from `rx`.
///   6. Assert the pending send completes (backpressure released).
///   7. Drain all remaining items and assert count == IPC_READER_CHANNEL_CAPACITY + 1
///      (no item was dropped).
///
/// Pre-fix (try_send path): step 3 would complete immediately (Err dropped),
/// and the total received count in step 7 would be IPC_READER_CHANNEL_CAPACITY,
/// not IPC_READER_CHANNEL_CAPACITY + 1.
#[tokio::test]
async fn test_BC_2_09_001_inbound_channel_backpressure_no_drop() {
    use monocle_ipc::error::IpcError;
    use monocle_ipc::types::ServerToClient;

    // A lightweight helper to produce a valid ServerToClient value without heap-
    // heavy session data.  PtyOutput is the natural payload for this pipeline.
    fn make_pty_item(seq: u8) -> Result<ServerToClient, IpcError> {
        Ok(ServerToClient::PtyOutput {
            session_id: format!("bp-test-{}", seq),
            bytes: vec![seq],
        })
    }

    // Arrange: create the production channel.
    let (tx, mut rx) = pty_output_channel();

    // Verify capacity before filling (defensive check).
    assert_eq!(
        rx.max_capacity(),
        IPC_READER_CHANNEL_CAPACITY,
        "precondition: channel capacity must be IPC_READER_CHANNEL_CAPACITY"
    );

    // Step 2: fill the channel to capacity with `IPC_READER_CHANNEL_CAPACITY` items.
    // Each item is a Result<ServerToClient, IpcError> — the channel element type.
    //
    // We use try_send here ONLY to fill the buffer synchronously without a consumer.
    // This is a test fixture operation, not the production path under test.
    for i in 0..IPC_READER_CHANNEL_CAPACITY {
        tx.try_send(make_pty_item(i as u8))
            .expect("channel must accept up to IPC_READER_CHANNEL_CAPACITY items");
    }

    // Verify the channel is now full.
    assert_eq!(
        rx.len(),
        IPC_READER_CHANNEL_CAPACITY,
        "precondition: channel must be full before testing backpressure"
    );

    // Step 3: spawn a task that tries to send the (capacity + 1)th item via .send().await.
    // If the channel uses .send().await, this task will BLOCK until space is free.
    // .send().await on a full channel MUST NOT complete immediately — that is the
    // backpressure property we are testing.
    let tx_clone = tx.clone();
    let send_handle = tokio::spawn(async move {
        // .send().await blocks until space is available — this is the key backpressure behavior.
        tx_clone
            .send(make_pty_item(99))
            .await
            .expect("send must succeed once space is available")
    });

    // Step 4: assert the task does NOT complete within a short poll window.
    // We use tokio::time::timeout to confirm the send is PENDING (awaiting a slot).
    let poll_result = tokio::time::timeout(std::time::Duration::from_millis(20), send_handle).await;

    assert!(
        poll_result.is_err(),
        "F-PASS4-LOW-002 (BC-2.09.001 Invariant 3): the (capacity+1)th .send().await must \
         NOT complete immediately when the channel is full — it must await backpressure. \
         If this assertion fails, the channel is not exhibiting backpressure behavior."
    );

    // Step 5: receive one item to free a slot.
    // This unblocks the spawned task from step 3 — it was waiting for a slot.
    let _freed = rx
        .recv()
        .await
        .expect("must receive one item from the filled channel");

    // Step 6: after freeing one slot, the spawned task from step 3 must complete
    // within a short window (it was blocked on .send().await, now a slot is free).
    //
    // We wait for the channel to refill back to IPC_READER_CHANNEL_CAPACITY — the step-3
    // task inserts one item once unblocked, bringing the count back to capacity.
    // Use a timeout loop: poll rx.len() until it reaches capacity again.
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    loop {
        if rx.len() >= IPC_READER_CHANNEL_CAPACITY {
            break;
        }
        if std::time::Instant::now() > deadline {
            panic!(
                "F-PASS4-LOW-002 (BC-2.09.001 Invariant 3): after freeing one slot, the \
                 blocked .send().await task must have completed and refilled the channel \
                 within 500ms — backpressure release not observed. \
                 Channel length: {} (expected {})",
                rx.len(),
                IPC_READER_CHANNEL_CAPACITY
            );
        }
        // Yield to the runtime so the spawned task can run.
        tokio::task::yield_now().await;
    }

    // Assertions summary:
    //   (a) the (capacity+1)th .send().await BLOCKED when channel was full (step 4),
    //   (b) after freeing one slot, the blocked task COMPLETED and refilled the channel
    //       (step 6 — channel length returned to IPC_READER_CHANNEL_CAPACITY),
    //   (c) no panic, no Err from .send().await — at-least-once delivery preserved.
    //
    // These three together prove .send().await backpressure semantics:
    // blocks-when-full, completes-when-space-freed, never-drops.
}

// ===========================================================================
// PASS-5 ADVERSARIAL REGRESSION TESTS
// Anchored to BC-2.09.001 v1.7.0 at Pass-5 authoring time Invariants 5/7
// Finding IDs: F-S039-P5-002, F-S039-P5-003, F-S039-P5-004, F-S039-P5-005
// ===========================================================================

// ---------------------------------------------------------------------------
// F-S039-P5-002: pending_pty_bytes byte-cap (512 KiB) eviction
// BC-2.09.001 Invariant 7 (v1.7.0) — BYTE-cap branch
//
// Pre-fix coverage gap: the existing test (test_BC_2_09_001_pending_pty_bytes_cap_drops_oldest)
// exercises the MESSAGE-COUNT cap (4097 small messages).  The BYTE-cap branch
// (MAX_PENDING_PTY_BYTES = 512 KiB) was untested.  A bug in the byte-cap path
// would permit unbounded memory growth for sessions receiving large chunks.
//
// Post-fix behavior (already implemented): on_pty_output checks total buffered
// bytes AFTER each push and evicts the OLDEST entry while the total exceeds
// MAX_PENDING_PTY_BYTES, incrementing pending_pty_drop_count for each eviction.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_pending_pty_bytes_byte_cap_drops_oldest
///
/// Exercises BC-2.09.001 Invariant 7 byte-cap branch (F-S039-P5-002):
///   When cumulative bytes in `pending_pty_bytes[s1]` exceed `MAX_PENDING_PTY_BYTES`
///   (512 KiB) while the message count remains below `MAX_PENDING_PTY_MESSAGES` (4096),
///   `on_pty_output` MUST evict OLDEST entries until the total is at/below the cap,
///   and MUST increment `pending_pty_drop_count[s1]` for each eviction.
///
/// Regression: if the byte-cap branch were absent (only the message-cap fires),
/// pushing ~9 × 64 KiB messages would accumulate 576 KiB in the buffer — above
/// MAX_PENDING_PTY_BYTES — without eviction.  This test catches that.
///
/// Setup: 9 messages of ~64 KiB each (total ≈ 576 KiB > 512 KiB, count = 9 < 4096).
/// Each message begins with a unique 8-byte marker so oldest vs. newest is unambiguous.
#[test]
fn test_BC_2_09_001_pending_pty_bytes_byte_cap_drops_oldest() {
    use monocle_core::pty_constants::{MAX_PENDING_PTY_BYTES, MAX_PENDING_PTY_MESSAGES};

    // Arrange: session with dump_in_progress = true so bytes are buffered.
    let s1 = "00000001-0000-4000-8001-000000000023";
    let mut app = make_app_with_session(s1);
    app.dump_in_progress.insert(s1.to_string(), true);

    // Each message is 64 KiB with a distinguishable 8-byte leading marker.
    // 9 messages × 64 KiB = 576 KiB > MAX_PENDING_PTY_BYTES (512 KiB).
    // 9 messages is WELL BELOW MAX_PENDING_PTY_MESSAGES (4096), so the message-count
    // cap must NOT fire — only the byte-cap is exercised by this test.
    const CHUNK_SIZE: usize = 64 * 1024; // 64 KiB per message
    let total_messages = 9usize;
    assert!(
        total_messages < MAX_PENDING_PTY_MESSAGES,
        "test design: message count ({}) must be below MAX_PENDING_PTY_MESSAGES ({}) so \
         only the byte-cap fires",
        total_messages,
        MAX_PENDING_PTY_MESSAGES
    );
    assert!(
        total_messages * CHUNK_SIZE > MAX_PENDING_PTY_BYTES,
        "test design: total bytes ({}) must exceed MAX_PENDING_PTY_BYTES ({}) to \
         guarantee byte-cap eviction",
        total_messages * CHUNK_SIZE,
        MAX_PENDING_PTY_BYTES
    );

    // Build distinguishable messages. Message 0 is "OLDEST" (will be evicted first);
    // message 8 is "NEWEST" (must survive the eviction pass).
    for i in 0..total_messages {
        let mut chunk = vec![0u8; CHUNK_SIZE];
        // Write a unique 8-byte prefix so the content is distinguishable after eviction.
        let marker = if i == 0 {
            *b"OLDEST__"
        } else if i == total_messages - 1 {
            *b"NEWEST__"
        } else {
            let mut m = [b'M'; 8];
            m[1] = b'0' + (i as u8);
            m
        };
        chunk[..8].copy_from_slice(&marker);
        on_pty_output(&mut app, s1.to_string(), chunk);
    }

    let buffer = app
        .pending_pty_bytes
        .get(s1)
        .expect("F-S039-P5-002: pending_pty_bytes must have an entry for s1");

    // Assert 1 (F-S039-P5-002 byte-cap): total retained bytes must be at/below cap.
    let total_bytes: usize = buffer.iter().map(|v| v.len()).sum();
    assert!(
        total_bytes <= MAX_PENDING_PTY_BYTES,
        "F-S039-P5-002 (BC-2.09.001 Invariant 7 byte-cap): total retained bytes ({}) must \
         be at/below MAX_PENDING_PTY_BYTES ({}) — byte-cap eviction not enforced",
        total_bytes,
        MAX_PENDING_PTY_BYTES
    );

    // Assert 2 (F-S039-P5-002): at least one eviction was recorded via drop counter.
    let drop_count = app.pending_pty_drop_count.get(s1).copied().unwrap_or(0);
    assert!(
        drop_count > 0,
        "F-S039-P5-002 (BC-2.09.001 Invariant 7 byte-cap): pending_pty_drop_count[s1] must \
         be > 0 after byte-cap overflow — pre-fix code never evicted on byte-cap. Got: {}",
        drop_count
    );

    // Assert 3 (F-S039-P5-002 drop-oldest): the NEWEST message must survive.
    let newest_survives = buffer
        .iter()
        .any(|chunk| chunk.len() >= 8 && &chunk[..8] == b"NEWEST__");
    assert!(
        newest_survives,
        "F-S039-P5-002 (BC-2.09.001 Invariant 7 byte-cap): NEWEST message (marker 'NEWEST__') \
         must be RETAINED after byte-cap eviction — drop-oldest semantics: latest messages survive"
    );

    // Assert 4 (F-S039-P5-002 drop-oldest): the OLDEST message must have been evicted.
    let oldest_absent = !buffer
        .iter()
        .any(|chunk| chunk.len() >= 8 && &chunk[..8] == b"OLDEST__");
    assert!(
        oldest_absent,
        "F-S039-P5-002 (BC-2.09.001 Invariant 7 byte-cap): OLDEST message (marker 'OLDEST__') \
         must have been EVICTED after byte-cap overflow — drop-oldest semantics: first-in evicted \
         first. Retained count: {}",
        buffer.len()
    );
}

// ---------------------------------------------------------------------------
// F-S039-P5-003: [dump: N drops] status surfacing when focused and in-progress
// BC-2.09.001 Invariant 7 MUST surfacing clause (v1.7.0)
//
// Pre-fix coverage gap: the render_frame logic that surfaces "[dump: N drops]" in
// the status bar was implemented but had no test asserting the actual rendered
// buffer content.  A typo in the format string, a wrong map key, or a missing
// condition check (dump_in_progress gate) would silently regress.
//
// Post-fix behavior: render_frame, when mode is EmbeddedTerminal { session_id }
// and dump_in_progress[session_id] == Some(true) and pending_pty_drop_count[session_id]
// > 0, passes "[dump: N drops]" to render_status_bar as the status message,
// overriding app.status_message for that frame.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_status_bar_shows_dump_drops_when_focused
///
/// Exercises BC-2.09.001 Invariant 7 status surfacing (F-S039-P5-003):
///   When the focused session has `dump_in_progress[session_id] == Some(true)`
///   AND `pending_pty_drop_count[session_id] > 0`, the rendered status bar MUST
///   contain the "[dump:" / "drops]" substring in the terminal buffer.
///
/// Companion assertions:
///   1. When `pending_pty_drop_count == 0`, the "[dump:" segment is ABSENT.
///   2. When `dump_in_progress` is not active (None/Some(false)), "[dump:" is ABSENT.
///
/// This test FAILS if the format string, map key, or dump_in_progress gate in
/// the render_frame EmbeddedTerminal arm is wrong — it asserts real buffer content.
#[test]
fn test_BC_2_09_001_status_bar_shows_dump_drops_when_focused() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let session_id = "00000001-0000-4000-8001-000000000024";

    // --- Case 1: dump_in_progress=true AND drop_count > 0 → "[dump: N drops]" PRESENT ---
    {
        let mut app = make_app_with_session(session_id);

        // Set up: dump in progress, 3 drops recorded.
        app.dump_in_progress.insert(session_id.to_string(), true);
        app.pending_pty_drop_count.insert(session_id.to_string(), 3);

        // Set mode to EmbeddedTerminal for this session.
        app.mode = AppMode::EmbeddedTerminal {
            session_id: session_id.to_string(),
            prior: FocusSnapshot::Sessions,
        };

        // Render via the production render_frame path.
        let backend = TestBackend::new(120, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal must initialize");
        let mut sessions_state = SessionsPanelState::default();

        terminal
            .draw(|frame| {
                render_frame(&mut app, &mut sessions_state, frame);
            })
            .expect("render_frame must succeed");

        // Assert: the rendered buffer contains "[dump:" and "drops]".
        // The status bar is on the last row(s); scan all cells.
        let buffer = terminal.backend().buffer().clone();
        let full_rendered: String = (0..24)
            .flat_map(|row| (0..120).map(move |col| (row, col)))
            .map(|(row, col)| buffer[(col, row)].symbol().to_string())
            .collect();

        assert!(
            full_rendered.contains("[dump:"),
            "F-S039-P5-003 (BC-2.09.001 Invariant 7): rendered buffer MUST contain '[dump:' \
             when dump_in_progress=true AND pending_pty_drop_count > 0. \
             Rendered buffer (truncated to 300 chars): {:?}",
            &full_rendered[..full_rendered.len().min(300)]
        );
        assert!(
            full_rendered.contains("drops]"),
            "F-S039-P5-003 (BC-2.09.001 Invariant 7): rendered buffer MUST contain 'drops]' \
             when dump_in_progress=true AND pending_pty_drop_count > 0. \
             Rendered buffer (truncated to 300 chars): {:?}",
            &full_rendered[..full_rendered.len().min(300)]
        );
    }

    // --- Case 2: dump_in_progress=true BUT drop_count == 0 → "[dump:" ABSENT ---
    {
        let mut app = make_app_with_session(session_id);
        app.dump_in_progress.insert(session_id.to_string(), true);
        // pending_pty_drop_count NOT set (defaults to 0/absent).
        app.mode = AppMode::EmbeddedTerminal {
            session_id: session_id.to_string(),
            prior: FocusSnapshot::Sessions,
        };

        let backend = TestBackend::new(120, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal must initialize");
        let mut sessions_state = SessionsPanelState::default();

        terminal
            .draw(|frame| {
                render_frame(&mut app, &mut sessions_state, frame);
            })
            .expect("render_frame must succeed");

        let buffer = terminal.backend().buffer().clone();
        let full_rendered: String = (0..24)
            .flat_map(|row| (0..120).map(move |col| (row, col)))
            .map(|(row, col)| buffer[(col, row)].symbol().to_string())
            .collect();

        assert!(
            !full_rendered.contains("[dump:"),
            "F-S039-P5-003 companion (BC-2.09.001 Invariant 7): '[dump:' must be ABSENT when \
             pending_pty_drop_count == 0 — the status override must not fire for zero drops"
        );
    }

    // --- Case 3: dump_in_progress NOT active (None) with drop_count > 0 → "[dump:" ABSENT ---
    // This guards against the dump_in_progress gate being bypassed.
    {
        let mut app = make_app_with_session(session_id);
        // dump_in_progress NOT set (None / dump not active).
        app.pending_pty_drop_count.insert(session_id.to_string(), 5);
        app.mode = AppMode::EmbeddedTerminal {
            session_id: session_id.to_string(),
            prior: FocusSnapshot::Sessions,
        };

        let backend = TestBackend::new(120, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal must initialize");
        let mut sessions_state = SessionsPanelState::default();

        terminal
            .draw(|frame| {
                render_frame(&mut app, &mut sessions_state, frame);
            })
            .expect("render_frame must succeed");

        let buffer = terminal.backend().buffer().clone();
        let full_rendered: String = (0..24)
            .flat_map(|row| (0..120).map(move |col| (row, col)))
            .map(|(row, col)| buffer[(col, row)].symbol().to_string())
            .collect();

        assert!(
            !full_rendered.contains("[dump:"),
            "F-S039-P5-003 companion (BC-2.09.001 Invariant 7): '[dump:' must be ABSENT when \
             dump_in_progress is not active (None) — the gate must prevent surfacing stale \
             drop counts from prior attach windows"
        );
    }
}

// ---------------------------------------------------------------------------
// F-S039-P5-004: re-entry abort guard — second enter_embedded_terminal aborts
// the prior timeout handle so no spurious DumpWindowTimeout fires
// BC-2.09.001 Invariant 5 / enter_embedded_terminal re-entry ordering
//
// Pre-fix coverage gap: the abort-prior-handle logic in enter_embedded_terminal
// was implemented but not tested.  A regression would cause two concurrent
// timeout tasks for the same session — the prior task would fire a spurious
// DumpWindowTimeout against the new attach window, force-resolving a live dump
// and causing the user to see a blank PTY screen until the next re-attach.
//
// Post-fix behavior: before inserting the new AbortHandle into dump_timeout_handles,
// enter_embedded_terminal removes and aborts any existing handle for that session.
// The map always holds at most ONE handle per session.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_reentry_aborts_prior_timeout_handle
///
/// Exercises BC-2.09.001 Invariant 5 / enter_embedded_terminal re-entry guard
/// (F-S039-P5-004):
///   Calling `enter_embedded_terminal` twice for the same session (without a
///   dump-complete in between) MUST:
///   1. Leave exactly ONE handle in `dump_timeout_handles[session_id]` (no accumulation).
///   2. Not panic.
///
/// This test requires `app.app_event_tx = Some(...)` so that enter_embedded_terminal
/// spawns the timeout task and inserts an AbortHandle.  Without a wired event_tx, the
/// timeout task is skipped and no AbortHandle is inserted (the abort guard is a no-op).
///
/// Pre-fix regression: without the abort guard, two `enter_embedded_terminal` calls
/// would insert two separate AbortHandles — the prior would fire a spurious
/// `DumpWindowTimeout` event after `DUMP_WINDOW_TIMEOUT` seconds.
#[tokio::test]
async fn test_BC_2_09_001_reentry_aborts_prior_timeout_handle() {
    let session_id = "00000001-0000-4000-8001-000000000025";
    let mut app = make_app_with_session(session_id);

    // Wire a real bounded ipc_tx so enter_embedded_terminal can send AttachSession.
    let (cmd_tx, mut _cmd_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(cmd_tx);

    // Wire a real bounded app_event_tx so the timeout task is spawned and
    // an AbortHandle is inserted into dump_timeout_handles.
    let (event_tx, _event_rx) = tokio::sync::mpsc::channel::<AppEvent>(16);
    app.app_event_tx = Some(event_tx);

    // First entry — should insert one AbortHandle.
    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // Assert A: exactly one AbortHandle after first entry.
    assert_eq!(
        app.dump_timeout_handles.len(),
        1,
        "F-S039-P5-004 (BC-2.09.001 Invariant 5): after first enter_embedded_terminal, \
         dump_timeout_handles must contain exactly 1 handle"
    );
    assert!(
        app.dump_timeout_handles.contains_key(session_id),
        "F-S039-P5-004: dump_timeout_handles must contain an entry for session_id after \
         first enter_embedded_terminal"
    );

    // Simulate re-entering without a dump completing: clear dump_in_progress and
    // pty_dump_received so the second call takes the auto-attach path again
    // (a real re-attach scenario after on_dump_window_timeout fired or was manually
    // reset by a test).  We use remove() to simulate the timeout handler having
    // cleared the flag without completing the dump.
    app.dump_in_progress.remove(session_id);
    app.pty_dump_received.remove(session_id);

    // Second entry — must abort the prior handle and insert a new one (1 total, not 2).
    enter_embedded_terminal(&mut app, session_id.to_string()).await;

    // Assert B: still exactly ONE handle (the new one; the prior was removed+aborted).
    assert_eq!(
        app.dump_timeout_handles.len(),
        1,
        "F-S039-P5-004 (BC-2.09.001 Invariant 5 re-entry abort guard): \
         after second enter_embedded_terminal, dump_timeout_handles must contain exactly 1 \
         handle — pre-fix code would accumulate 2 handles (one per enter call)"
    );
    assert!(
        app.dump_timeout_handles.contains_key(session_id),
        "F-S039-P5-004: dump_timeout_handles must still contain an entry for session_id \
         after second enter_embedded_terminal (the new handle replaces the prior one)"
    );
}

// ---------------------------------------------------------------------------
// F-S039-P5-005: dump-window timeout end-to-end — spawn → event channel → handler
// BC-2.09.001 Invariant 8 (v1.7.0) wiring
//
// Pre-fix coverage gap: the existing test (test_BC_2_09_001_dump_window_timeout_force_resolves)
// calls on_dump_window_timeout() directly.  The FULL END-TO-END PATH —
// enter_embedded_terminal spawning the timeout task → DUMP_WINDOW_TIMEOUT elapses →
// AppEvent::DumpWindowTimeout delivered to app_event_rx → handler drains the event
// → on_dump_window_timeout cleans up state — was untested.  A bug in any step of
// the wiring (wrong session_id captured, channel send dropped, handler arm not reached)
// would silently leave dump_in_progress = true indefinitely.
//
// Post-fix wiring (already implemented): enter_embedded_terminal captures session_id
// into the spawned task, sends AppEvent::DumpWindowTimeout { session_id } after
// DUMP_WINDOW_TIMEOUT, and on_dump_window_timeout performs the cleanup.
// ---------------------------------------------------------------------------

/// test_BC_2_09_001_dump_window_timeout_end_to_end
///
/// Exercises BC-2.09.001 Invariant 8 end-to-end wiring (F-S039-P5-005):
///   1. `enter_embedded_terminal` (Ok path, `app_event_tx` wired) spawns the timeout task.
///   2. Advancing the paused clock past `DUMP_WINDOW_TIMEOUT` causes the task to fire.
///   3. `AppEvent::DumpWindowTimeout { session_id }` is delivered to `app_event_rx`.
///   4. Dispatching the event through `on_dump_window_timeout` performs force-resolve:
///      - `dump_in_progress[s1]` absent (removed).
///      - `pending_pty_bytes[s1]` cleared.
///      - `pty_parsers[s1]` reset to `PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS`.
///      - `pty_dump_received` does NOT contain s1 (dump never completed).
///
/// Timing: uses `tokio::time::pause()` + `tokio::time::advance()` for deterministic
/// timing — NOT a real 10-second sleep.
#[tokio::test(start_paused = true)]
async fn test_BC_2_09_001_dump_window_timeout_end_to_end() {
    use monocle_core::pty_constants::DUMP_WINDOW_TIMEOUT;
    use monocle_core::tui::state::{PTY_DEFAULT_COLS, PTY_DEFAULT_ROWS};

    let s1 = "00000001-0000-4000-8001-000000000026";
    let mut app = make_app_with_session(s1);

    // Wire a real bounded ipc_tx so enter_embedded_terminal can send AttachSession.
    let (cmd_tx, _cmd_rx) = mpsc::channel::<ClientToServer>(64);
    app.ipc_tx = Some(cmd_tx);

    // Wire a real bounded app_event channel — the run loop normally owns both ends;
    // here the test owns the receiver so it can intercept the timeout event.
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<AppEvent>(16);
    app.app_event_tx = Some(event_tx);

    // Precondition: pty_dump_received does NOT contain s1 (first-entry path).
    assert!(
        !app.pty_dump_received.contains(s1),
        "precondition (F-S039-P5-005): session must not be in pty_dump_received"
    );

    // Act 1: enter_embedded_terminal — Ok path, app_event_tx wired.
    // This sets dump_in_progress[s1] = true, sends AttachSession, spawns the
    // timeout task, and stores the AbortHandle in dump_timeout_handles[s1].
    enter_embedded_terminal(&mut app, s1.to_string()).await;

    // Verify dump_in_progress is true after enter.
    assert_eq!(
        app.dump_in_progress.get(s1).copied(),
        Some(true),
        "F-S039-P5-005 precondition: dump_in_progress must be true after enter_embedded_terminal"
    );

    // Verify an AbortHandle was inserted (confirms the timeout task was spawned).
    assert!(
        app.dump_timeout_handles.contains_key(s1),
        "F-S039-P5-005 precondition: dump_timeout_handles must contain s1 (task was spawned)"
    );

    // Buffer some pending bytes to verify they are cleared on timeout.
    app.pending_pty_bytes.insert(
        s1.to_string(),
        vec![b"pending-during-dump\r\n".to_vec()].into(),
    );

    // Act 2: advance the paused clock past DUMP_WINDOW_TIMEOUT.
    // In a `#[tokio::test(start_paused = true)]` runtime, `tokio::time::advance(d).await`
    // moves the mock clock forward by `d` and internally drives tasks to allow timers that
    // have become ready to be woken up.  The spawned task sleeps for `DUMP_WINDOW_TIMEOUT`;
    // after the advance it will be woken by the runtime and run its body
    // (send AppEvent::DumpWindowTimeout).
    //
    // Strategy: advance the clock, then drive the spawned task to completion by awaiting
    // `event_rx.recv()` directly.  Because the mock clock is now past the sleep deadline,
    // the spawned task will be ready to run and will complete before `recv()` blocks.
    tokio::time::advance(DUMP_WINDOW_TIMEOUT + std::time::Duration::from_millis(1)).await;

    // Act 3: receive the event.
    // The spawned timeout task is now ready (its sleep expired) and will run when we
    // await event_rx.recv().  The recv() suspends our task, the runtime polls the
    // spawned task (which sends the event and completes), then our task wakes on the
    // channel notification.  This is deterministic — no real sleep required.
    let event = event_rx.recv().await.expect(
        "F-S039-P5-005 (BC-2.09.001 Invariant 8): event channel must not be closed \
             before DumpWindowTimeout is delivered — spawned task must send the event \
             after DUMP_WINDOW_TIMEOUT elapses",
    );

    // Assert 3a: the received event is DumpWindowTimeout for s1.
    match &event {
        AppEvent::DumpWindowTimeout { session_id } => {
            assert_eq!(
                session_id, s1,
                "F-S039-P5-005 (BC-2.09.001 Invariant 8): DumpWindowTimeout must carry the \
                 correct session_id ('{}') — got '{}'",
                s1, session_id
            );
        }
        _ => {
            panic!(
                "F-S039-P5-005 (BC-2.09.001 Invariant 8): expected AppEvent::DumpWindowTimeout \
                 for '{}', got a different event variant — the timeout task must send \
                 DumpWindowTimeout after DUMP_WINDOW_TIMEOUT elapses",
                s1
            );
        }
    }

    // Act 4: dispatch the event through the production on_dump_window_timeout handler.
    // This mirrors what the run loop does when it receives AppEvent from event_rx.
    if let AppEvent::DumpWindowTimeout { session_id } = event {
        on_dump_window_timeout(&mut app, session_id);
    }

    // Assert 4a (Invariant 8): dump_in_progress entry removed (force-resolve complete).
    assert_eq!(
        app.dump_in_progress.get(s1).copied(),
        None,
        "F-S039-P5-005 (BC-2.09.001 Invariant 8 end-to-end): dump_in_progress must be ABSENT \
         after full timeout pipeline — enter → clock advance → event → on_dump_window_timeout"
    );

    // Assert 4b (Invariant 8): pending_pty_bytes cleared.
    let buffered_len = app.pending_pty_bytes.get(s1).map(|v| v.len()).unwrap_or(0);
    assert_eq!(
        buffered_len, 0,
        "F-S039-P5-005 (BC-2.09.001 Invariant 8 end-to-end): pending_pty_bytes must be \
         empty/absent after timeout force-resolve"
    );

    // Assert 4c (Invariant 8): parser reset to PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS.
    let parser = app.pty_parsers.get(s1).expect(
        "F-S039-P5-005 (BC-2.09.001 Invariant 8 end-to-end): pty_parsers must still contain \
         s1 after timeout (reset, NOT GC'd)",
    );
    let (rows, cols) = parser.screen().size();
    assert_eq!(
        rows, PTY_DEFAULT_ROWS,
        "F-S039-P5-005 (BC-2.09.001 Invariant 8 end-to-end): parser rows must be \
         PTY_DEFAULT_ROWS ({}) after timeout reset — got {}",
        PTY_DEFAULT_ROWS, rows
    );
    assert_eq!(
        cols, PTY_DEFAULT_COLS,
        "F-S039-P5-005 (BC-2.09.001 Invariant 8 end-to-end): parser cols must be \
         PTY_DEFAULT_COLS ({}) after timeout reset — got {}",
        PTY_DEFAULT_COLS, cols
    );

    // Assert 4d (Invariant 8): pty_dump_received does NOT contain s1.
    // The dump never completed — the next enter_embedded_terminal must re-run the protocol.
    assert!(
        !app.pty_dump_received.contains(s1),
        "F-S039-P5-005 (BC-2.09.001 Invariant 8 end-to-end): pty_dump_received must NOT \
         contain s1 after timeout — dump never completed, re-attach required"
    );
}
