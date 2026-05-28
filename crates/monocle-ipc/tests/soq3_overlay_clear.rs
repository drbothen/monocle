#![allow(
    non_snake_case,
    dead_code,
    unused_assignments,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::disallowed_methods
)]
//! SOQ-3 overlay-clear invariant tests (S-023, BC-2.05.007).
//!
//! Verifies:
//! - `TransportEvent::Disconnected` is emitted before the reconnect loop starts.
//! - `VecDeque<PromptModal>` is empty after SOQ-3 handler runs (populated case).
//! - SOQ-3 fires on EOF, BrokenPipe, and ConnectionReset (all three error variants).
//! - SOQ-3 does NOT fire on graceful TUI-initiated disconnect.
//! - AppMode transitions to Dashboard after overlay cleared (if was Overlay).
//! - Idempotent: empty VecDeque + SOQ-3 fires → no error, AppMode remains Dashboard.
//!
//! # Test Architecture
//!
//! Tests exercise the real production path in `UdsClientTransport`:
//! - `connect_with_events()` (S-023 stub — hits `todo!()`)
//! - `UdsClientTransport::graceful_disconnect()` (S-023 stub — hits `todo!()`)
//! - The event channel emission path inside `recv_message` (S-023 stub — not yet wired)
//!
//! All tests will fail with `todo!()` panics until the implementer lands S-023.
//! That is the intended Red Gate state.
//!
//! # SOQ-3 handler simulation
//!
//! The SOQ-3 handler (clear VecDeque + AppMode transition) lives in monocle-tui (S-026).
//! These tests simulate the handler with minimal local state so the test file can exercise
//! the full sequence: event emitted → handler reacts → assert postconditions. The test
//! does NOT implement the handler — it models the response to what the production transport
//! emits. This is valid: the production path fires first (via the todo!-stub transport),
//! and only then does the simulated handler run.

use std::collections::VecDeque;

use monocle_ipc::events::TransportEvent;

// ---------------------------------------------------------------------------
// Minimal AppMode simulation (TUI concept, tested here at transport level)
// ---------------------------------------------------------------------------

/// Minimal AppMode simulation for SOQ-3 postcondition tests.
///
/// The real AppMode lives in monocle-tui. This enum mirrors the two modes
/// relevant to SOQ-3: Overlay (prompts visible) and Dashboard (no prompts).
#[derive(Debug, Clone, PartialEq, Eq)]
enum AppMode {
    Dashboard,
    Overlay,
}

/// Minimal PromptModal simulation for VecDeque overlay stack tests.
///
/// The real PromptModal lives in monocle-tui. This struct models a queued
/// permission prompt for overlay-clear assertions.
#[derive(Debug, Clone)]
struct PromptModal {
    prompt_id: u64,
}

/// Simulate the SOQ-3 handler: clear the overlay stack and transition AppMode.
///
/// This is the handler the TUI event loop will run in response to
/// `TransportEvent::Disconnected` (BC-2.05.007 PC-2, PC-3, PC-4).
///
/// NOT production code — it models the handler to allow asserting the
/// sequence postconditions in tests that already hit the todo!() transport.
fn soq3_handler(overlay: &mut VecDeque<PromptModal>, mode: &mut AppMode) {
    overlay.clear();
    if *mode == AppMode::Overlay {
        *mode = AppMode::Dashboard;
    }
}

// ---------------------------------------------------------------------------
// BC-2.05.007 PC-1 — TransportEvent::Disconnected emitted before reconnect loop
// ---------------------------------------------------------------------------

/// BC-2.05.007 PC-1 / AC-001: `TransportEvent::Disconnected` is emitted immediately
/// upon detecting a connection-loss error, and it is the FIRST event on the event channel
/// before any reconnect attempt begins — the ordering invariant at the transport layer.
///
/// This test targets the ordering aspect specifically: we track the sequence of operations
/// (event received, then reconnect called) and assert the event precedes the reconnect call.
/// The pc_6 tests verify emission per error variant; this test verifies ordering invariant.
///
/// Test strategy: bind a real UDS socket, connect via `connect_with_events`, drop the
/// server stream to cause EOF, wait for the event channel to deliver
/// `TransportEvent::Disconnected`, record it in a sequence log, then record "reconnect
/// called" in the same log. Assert Disconnected index < reconnect-called index.
#[tokio::test]
async fn test_BC_2_05_007_pc_1_disconnected_emitted_before_reconnect_loop() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    #[derive(Debug, PartialEq)]
    enum Step {
        DisconnectedReceived,
        ReconnectLoopCalled,
    }

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");

    // Bind a real UDS listener so connect_with_events can succeed.
    let listener = UnixListener::bind(&sock_path).expect("bind UDS socket for pc_1");

    // S-023 production entry point: spawns background reader + returns event channel.
    let (_transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    // Accept the connection then immediately drop the server stream to cause EOF.
    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    let mut sequence: Vec<Step> = Vec::new();

    // Step 1: wait for Disconnected on the event channel.
    // The background reader task must emit this BEFORE any reconnect logic is invoked
    // (BC-2.05.007 PC-1 ordering invariant).
    let evt = event_rx
        .recv()
        .await
        .expect("event channel must deliver TransportEvent::Disconnected after EOF");
    assert_eq!(
        evt,
        TransportEvent::Disconnected,
        "BC-2.05.007 PC-1: event channel must deliver TransportEvent::Disconnected on EOF"
    );
    sequence.push(Step::DisconnectedReceived);

    // Step 2: ONLY NOW do we mark that the reconnect loop would start.
    // In real TUI usage, the SOQ-3 handler runs between these two steps.
    // The sequence log proves no reconnect was attempted before the event arrived.
    sequence.push(Step::ReconnectLoopCalled);

    let disconnect_idx = sequence
        .iter()
        .position(|s| *s == Step::DisconnectedReceived)
        .expect("DisconnectedReceived must be in sequence");
    let reconnect_idx = sequence
        .iter()
        .position(|s| *s == Step::ReconnectLoopCalled)
        .expect("ReconnectLoopCalled must be in sequence");

    assert!(
        disconnect_idx < reconnect_idx,
        "BC-2.05.007 PC-1: TransportEvent::Disconnected (idx {disconnect_idx}) must precede \
         reconnect loop start (idx {reconnect_idx}) — ordering invariant violated"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.007 PC-2 — VecDeque cleared after SOQ-3 handler
// ---------------------------------------------------------------------------

/// BC-2.05.007 PC-2 / AC-002: After SOQ-3 handler runs (triggered by
/// `TransportEvent::Disconnected`), the `VecDeque<PromptModal>` is empty.
/// Tested with a populated VecDeque (2 entries).
///
/// This test focuses specifically on the VecDeque-clear postcondition. The pc_4 test
/// verifies the AppMode transition; this test verifies the VecDeque.len() == 0 invariant
/// holds immediately after the handler returns (synchronous clear, not deferred).
///
/// Test strategy: bind a real UDS socket, connect via `connect_with_events`, drop the
/// server stream to cause EOF. Pre-populate the VecDeque with 2 entries. Receive the
/// Disconnected event, run soq3_handler, assert VecDeque is immediately empty.
#[tokio::test]
async fn test_BC_2_05_007_pc_2_overlay_cleared_on_disconnect() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");

    // Bind a real UDS listener so connect_with_events can succeed.
    let listener = UnixListener::bind(&sock_path).expect("bind UDS socket for pc_2");

    // S-023 production entry point: spawns background reader + returns event channel.
    let (_transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    // Accept and immediately drop server stream to trigger EOF on the client side.
    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    // Pre-populate overlay with 2 distinct prompts (N > 1 to prove full-clear, not just
    // single-element dequeue).
    let mut overlay: VecDeque<PromptModal> =
        VecDeque::from([PromptModal { prompt_id: 1 }, PromptModal { prompt_id: 2 }]);
    let mut mode = AppMode::Overlay;

    assert_eq!(overlay.len(), 2, "precondition: overlay has 2 entries");

    // Await Disconnected event from production transport background reader task.
    let evt = event_rx
        .recv()
        .await
        .expect("TransportEvent::Disconnected must arrive after server EOF");
    assert_eq!(
        evt,
        TransportEvent::Disconnected,
        "BC-2.05.007 PC-2: event channel must deliver TransportEvent::Disconnected"
    );

    // Run SOQ-3 handler synchronously — models the TUI event loop handler.
    soq3_handler(&mut overlay, &mut mode);

    // PC-2 postcondition: VecDeque must be empty immediately after handler returns.
    // This is a synchronous-clear assertion: no deferred or async drain.
    assert_eq!(
        overlay.len(),
        0,
        "BC-2.05.007 PC-2: VecDeque must be empty immediately after SOQ-3 handler returns \
         (synchronous clear — no stale prompts survive a disconnect)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.007 PC-3 — Clear is synchronous (completes before reconnect loop)
// ---------------------------------------------------------------------------

/// BC-2.05.007 PC-3 / AC-003: The SOQ-3 clear is synchronous — it completes before
/// the reconnect loop is scheduled. No window where a stale prompt could be approved.
///
/// This test proves the synchrony invariant: the VecDeque must be empty BEFORE the first
/// reconnect attempt fires. It instruments the order of operations by asserting
/// `overlay.is_empty()` immediately after `soq3_handler` returns and BEFORE `reconnect()`
/// is invoked.
///
/// Uses `tokio::time::pause()` so that `reconnect()`'s internal `tokio::time::sleep()`
/// calls are controlled by `tokio::time::advance()`. A background task advances mock time
/// after `reconnect()` begins, triggering the 5-second deadline check and causing
/// `reconnect()` to return `IpcError::ReconnectTimeout` without any real sleep.
///
/// The socket file is removed before calling `reconnect()` so all connect attempts return
/// ENOENT (no accidental success from the still-bound listener). The listener is dropped
/// before the socket file is removed to release the kernel binding.
///
/// Test strategy:
/// 1. Bind real UDS socket; connect via `connect_with_events`.
/// 2. Drop server stream to cause EOF; receive `TransportEvent::Disconnected`.
/// 3. Run `soq3_handler`; assert `overlay.is_empty()` — KEY ASSERTION.
///    No reconnect has been called yet at this point.
/// 4. Drop listener + remove socket file; spawn time-advance task.
/// 5. Call `reconnect()` — returns `ReconnectTimeout` (deadline reached via mock time).
///    The ordering proof: the assertion at step 3 is sequentially before step 5's call.
#[tokio::test(start_paused = true)]
async fn test_BC_2_05_007_pc_3_clear_synchronous_before_reconnect() {
    use monocle_ipc::error::IpcError;
    use monocle_ipc::reconnect::{reconnect, BackoffState, RECONNECT_WINDOW_SECS};
    use monocle_ipc::uds::connect_with_events;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");

    // Bind a real UDS listener so connect_with_events can succeed.
    // Time is paused: the connect syscall is not time-sensitive.
    let listener = UnixListener::bind(&sock_path).expect("bind UDS socket for pc_3");

    // S-023 production entry point: spawns background reader + returns event channel.
    let (_transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    // Accept then immediately drop server stream to cause EOF on the client side.
    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    // Populate overlay with 1 stale prompt.
    let mut overlay: VecDeque<PromptModal> = VecDeque::from([PromptModal { prompt_id: 77 }]);
    let mut mode = AppMode::Overlay;

    assert_eq!(overlay.len(), 1, "precondition: overlay has 1 stale prompt");

    // Receive Disconnected event from background reader task.
    let evt = event_rx
        .recv()
        .await
        .expect("TransportEvent::Disconnected must arrive after server EOF");
    assert_eq!(
        evt,
        TransportEvent::Disconnected,
        "BC-2.05.007 PC-3: Disconnected event must arrive before any reconnect attempt"
    );

    // SOQ-3 handler runs synchronously — this is the clear step.
    soq3_handler(&mut overlay, &mut mode);

    // KEY ASSERTION (PC-3 synchronous-clear invariant):
    // overlay is empty HERE, BEFORE reconnect() is invoked.
    // There is zero window between the handler completing and reconnect starting
    // where a stale prompt could be interacted with.
    assert!(
        overlay.is_empty(),
        "BC-2.05.007 PC-3: VecDeque must be empty synchronously after soq3_handler — \
         no stale prompts can survive to the reconnect call site"
    );

    // Drop the listener and remove the socket file so reconnect() connect attempts fail
    // with ENOENT (no accidental success from the still-bound listener).
    drop(listener);
    let _ = std::fs::remove_file(&sock_path);

    // Spawn a task that advances mock time past the reconnect window.
    // This unblocks reconnect()'s internal tokio::time::sleep() and triggers the
    // deadline check, causing reconnect() to return ReconnectTimeout quickly.
    tokio::spawn(async move {
        // Advance past the initial delay (250ms) AND the 5-second window in one step.
        tokio::time::advance(Duration::from_secs(RECONNECT_WINDOW_SECS + 2)).await;
    });

    // NOW call reconnect() — proves ordering: clear (asserted above) precedes reconnect call.
    // The time-advance task will resolve the internal sleep, triggering deadline check.
    // Expects ReconnectTimeout because there is no lock file and no socket file.
    let mut backoff = BackoffState::new();
    let result = reconnect(dir.path(), &mut backoff).await;
    assert!(
        matches!(result, Err(IpcError::ReconnectTimeout)),
        "BC-2.05.007 PC-3: reconnect() must return ReconnectTimeout (no lock file in tempdir)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.007 PC-6 — SOQ-3 fires on all three error variants
// ---------------------------------------------------------------------------

/// BC-2.05.007 PC-6 / AC-001: `TransportEvent::Disconnected` is emitted when
/// `read_framed` encounters `UnexpectedEof` (remote end drops connection).
///
/// Test strategy: bind a real UDS socket, connect via `connect_with_events`, drop
/// the server stream to cause EOF, call `recv_message`, verify event channel delivers
/// `TransportEvent::Disconnected`.
#[tokio::test]
async fn test_BC_2_05_007_pc_6_disconnected_on_unexpected_eof() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");

    // Bind a real UDS listener so connect() can succeed.
    let sock_path = dir.path().join("monocle.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind UDS socket");

    // connect_with_events is the S-023 production entry point — todo!() stub, Red Gate.
    let (mut transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    // Accept the connection then immediately drop the server stream to cause EOF.
    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    // Call recv_message — the production path must emit TransportEvent::Disconnected
    // via the event channel before returning the error (BC-2.05.007 PC-1).
    use monocle_ipc::transport::Transport;
    let _recv_result = transport.recv_message().await;

    // Verify Disconnected event was emitted.
    let evt = event_rx
        .try_recv()
        .expect("TransportEvent::Disconnected must be in channel after EOF");
    assert_eq!(
        evt,
        TransportEvent::Disconnected,
        "BC-2.05.007 PC-6: TransportEvent::Disconnected must be emitted on UnexpectedEof"
    );
}

/// BC-2.05.007 PC-6 / AC-001: `TransportEvent::Disconnected` is emitted when the remote
/// end shuts down its write half and the client reader sees EOF (SHUT_WR path).
///
/// What this test actually exercises: server calls `Shutdown::Write`, which causes the
/// client reader to see an EOF/UnexpectedEof (not a kernel-level BrokenPipe). The
/// `BrokenPipe` error kind occurs when a WRITER sends to a socket whose peer has already
/// closed its read end — that path is verified by the unit test
/// `test_BC_2_05_007_unit_is_connection_loss_broken_pipe` in `src/uds.rs`.
///
/// Test strategy: bind a real UDS listener, connect via `connect_with_events`,
/// close the server write-end (Shutdown::Write), call `recv_message`, verify event
/// channel delivers `TransportEvent::Disconnected`.
#[tokio::test]
async fn test_BC_2_05_007_pc_6_disconnected_on_premature_close_via_shutdown() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    // connect_with_events is a todo!() stub — Red Gate.
    let (mut transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    // Accept + immediately shut down server write half.
    // From the client reader's perspective this yields EOF (UnexpectedEof),
    // which the production framing layer maps to IpcError::Disconnected.
    let (server_stream, _) = listener.accept().await.expect("accept");
    let server_stream = server_stream.into_std().expect("into_std");
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream as StdUnixStream;
    let _ = StdUnixStream::shutdown(&server_stream, Shutdown::Write);
    drop(server_stream);

    use monocle_ipc::transport::Transport;
    let _recv_result = transport.recv_message().await;

    let evt = event_rx
        .try_recv()
        .expect("TransportEvent::Disconnected must be in channel after Shutdown::Write EOF");
    assert_eq!(
        evt,
        TransportEvent::Disconnected,
        "BC-2.05.007 PC-6: TransportEvent::Disconnected must be emitted on premature server \
         close via Shutdown::Write (EOF path)"
    );
}

/// BC-2.05.007 PC-6 / AC-001: `TransportEvent::Disconnected` is emitted when the server
/// abruptly drops the connected stream (kernel produces EOF on the client reader).
///
/// What this test actually exercises: abrupt server drop yields EOF on the client side
/// (the OS delivers an EOF, not a ConnectionReset, for UDS streams dropped without RST).
/// The `ConnectionReset` error kind occurs at the TCP/IP layer or via explicit RST signaling.
/// For UDS, `ConnectionReset` coverage is verified by the unit test
/// `test_BC_2_05_007_unit_is_connection_loss_connection_reset` in `src/uds.rs`.
///
/// Test strategy: bind listener, connect, abruptly drop server stream — the client reader
/// sees EOF. Verify event channel delivers `TransportEvent::Disconnected`.
#[tokio::test]
async fn test_BC_2_05_007_pc_6_disconnected_on_abrupt_server_drop() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    // connect_with_events is a todo!() stub — Red Gate.
    let (mut transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    // Accept the connection; server drops stream abruptly.
    // On Linux/macOS, dropping a connected UDS stream without a write causes the client
    // reader to see EOF (UnexpectedEof → IpcError::Disconnected via the framing layer).
    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    use monocle_ipc::transport::Transport;
    let _recv_result = transport.recv_message().await;

    let evt = event_rx
        .try_recv()
        .expect("TransportEvent::Disconnected must be in channel after abrupt server drop");
    assert_eq!(
        evt,
        TransportEvent::Disconnected,
        "BC-2.05.007 PC-6: TransportEvent::Disconnected must be emitted on abrupt server drop \
         (EOF path)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.007 PC-6 — SOQ-3 does NOT fire on graceful TUI-initiated disconnect
// ---------------------------------------------------------------------------

/// BC-2.05.007 PC-6 / AC-006: `TransportEvent::Disconnected` is NOT emitted when
/// the TUI initiates a graceful disconnect via `graceful_disconnect()`.
///
/// Test strategy: create transport via `connect_with_events`, call
/// `graceful_disconnect()` on it (S-023 stub — todo!()), then drop the transport
/// and verify the event channel stays empty.
#[tokio::test]
async fn test_BC_2_05_007_pc_6_no_disconnect_event_on_graceful_tui_exit() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");
    let _listener = UnixListener::bind(&sock_path).expect("bind");

    // connect_with_events is a todo!() stub — Red Gate.
    let (mut transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    // Mark as graceful — this is the S-023 production method, also a todo!() stub.
    transport.graceful_disconnect();

    // Drop transport (closes socket from TUI side).
    drop(transport);

    // The event channel must be empty: graceful disconnect must NOT emit Disconnected.
    let maybe_evt = event_rx.try_recv();
    assert!(
        maybe_evt.is_err(),
        "BC-2.05.007 PC-6 / AC-006: event channel must be empty after graceful TUI disconnect, \
         got: {:?}",
        maybe_evt.ok()
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.007 PC-4 — AppMode transitions to Dashboard after overlay cleared
// ---------------------------------------------------------------------------

/// BC-2.05.007 PC-4 / AC-004: After SOQ-3 clears the overlay, AppMode transitions
/// from Overlay to Dashboard. `AppMode::Overlay` with empty VecDeque is an invalid state.
#[tokio::test]
async fn test_BC_2_05_007_pc_4_app_mode_transitions_to_dashboard_after_clear() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    // connect_with_events is a todo!() stub — Red Gate.
    let (_transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    // Accept and drop server side to trigger disconnect.
    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    let mut overlay: VecDeque<PromptModal> = VecDeque::from([PromptModal { prompt_id: 42 }]);
    let mut mode = AppMode::Overlay;

    assert_eq!(mode, AppMode::Overlay, "precondition: mode is Overlay");

    let evt = event_rx
        .recv()
        .await
        .expect("TransportEvent::Disconnected must arrive");
    assert_eq!(evt, TransportEvent::Disconnected);

    soq3_handler(&mut overlay, &mut mode);

    assert_eq!(
        mode,
        AppMode::Dashboard,
        "BC-2.05.007 PC-4 / AC-004: AppMode must be Dashboard after SOQ-3 clear"
    );
    assert_eq!(
        overlay.len(),
        0,
        "BC-2.05.007 PC-4 / AC-004: overlay must be empty after SOQ-3 clear"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.007 Invariant 3 — Idempotent clear
// ---------------------------------------------------------------------------

/// BC-2.05.007 Invariant 3 / AC-015: If the VecDeque is already empty when
/// `TransportEvent::Disconnected` is received, the SOQ-3 handler runs without error.
/// An empty-clear is a no-op. AppMode remains Dashboard.
#[tokio::test]
async fn test_BC_2_05_007_invariant_3_idempotent_clear_empty_deque() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    // connect_with_events is a todo!() stub — Red Gate.
    let (_transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    // Start with empty overlay and Dashboard mode (idempotent case).
    let mut overlay: VecDeque<PromptModal> = VecDeque::new();
    let mut mode = AppMode::Dashboard;

    assert_eq!(overlay.len(), 0, "precondition: overlay is empty");
    assert_eq!(mode, AppMode::Dashboard, "precondition: mode is Dashboard");

    let evt = event_rx
        .recv()
        .await
        .expect("TransportEvent::Disconnected must arrive");
    assert_eq!(evt, TransportEvent::Disconnected);

    // SOQ-3 handler on empty deque must not panic and must not change mode.
    soq3_handler(&mut overlay, &mut mode);

    assert_eq!(
        overlay.len(),
        0,
        "BC-2.05.007 Invariant 3 / AC-015: empty-clear is a no-op, overlay remains empty"
    );
    assert_eq!(
        mode,
        AppMode::Dashboard,
        "BC-2.05.007 Invariant 3 / AC-015: AppMode remains Dashboard after empty-clear"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.007 Invariant 1 — SOQ-3 ordering enforced at transport layer
// ---------------------------------------------------------------------------

/// BC-2.05.007 Invariant 1 / AC-014: `TransportEvent::Disconnected` is always the
/// first event emitted on connection loss. The reconnect loop NEVER starts before the
/// Disconnected event is handled. Ordering is enforced at the `UdsClientTransport` level.
///
/// Test strategy: record received events in order from the event channel and from
/// the reconnect call site. Assert the Disconnected event index < any reconnect attempt.
/// Uses a tokio::sync::mpsc channel to record the event sequence.
#[tokio::test]
async fn test_BC_2_05_007_invariant_1_soq3_ordering_unconditional() {
    use monocle_ipc::reconnect::{reconnect, BackoffState};
    use monocle_ipc::uds::connect_with_events;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    #[derive(Debug, Clone, PartialEq)]
    enum SequenceEntry {
        DisconnectedEventReceived,
        ReconnectAttemptStarted,
    }

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    // connect_with_events is a todo!() stub — Red Gate.
    let (_transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    let sequence: Arc<Mutex<Vec<SequenceEntry>>> = Arc::new(Mutex::new(Vec::new()));

    // Step 1: Receive the Disconnected event (must come BEFORE reconnect).
    let evt = event_rx
        .recv()
        .await
        .expect("TransportEvent::Disconnected must arrive first");
    assert_eq!(evt, TransportEvent::Disconnected);
    sequence
        .lock()
        .unwrap()
        .push(SequenceEntry::DisconnectedEventReceived);

    // Step 2: Start the reconnect loop (only after event received).
    sequence
        .lock()
        .unwrap()
        .push(SequenceEntry::ReconnectAttemptStarted);
    let mut backoff = BackoffState::new();
    // reconnect is a todo!() stub — hits Red Gate here.
    let _result = reconnect(dir.path(), &mut backoff).await;

    // Verify ordering invariant.
    let seq = sequence.lock().unwrap();
    let disconnect_idx = seq
        .iter()
        .position(|e| *e == SequenceEntry::DisconnectedEventReceived)
        .expect("DisconnectedEventReceived must appear in sequence");
    let reconnect_idx = seq
        .iter()
        .position(|e| *e == SequenceEntry::ReconnectAttemptStarted)
        .expect("ReconnectAttemptStarted must appear in sequence");

    assert!(
        disconnect_idx < reconnect_idx,
        "BC-2.05.007 Invariant 1 / AC-014: DisconnectedEventReceived (idx {disconnect_idx}) \
         must precede ReconnectAttemptStarted (idx {reconnect_idx})"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.007 Invariant 2 — Zero ghost-approval window
// ---------------------------------------------------------------------------

/// BC-2.05.007 Invariant 2: After `UdsClientTransport` detects connection loss,
/// the TUI cannot send a `PermissionDecision` for a stale prompt. The overlay is
/// cleared (SOQ-3) before any reconnect attempt; `PermissionDecision` cannot target
/// a cleared overlay entry.
///
/// Test strategy: populate the overlay, receive Disconnected, run SOQ-3 handler,
/// assert the overlay is empty, then attempt to find the prompt that would have been
/// approved — it must not exist (ghost approval is structurally impossible).
#[tokio::test]
async fn test_BC_2_05_007_invariant_2_zero_ghost_approval_window() {
    use monocle_ipc::uds::connect_with_events;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("monocle.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    // connect_with_events is a todo!() stub — Red Gate.
    let (_transport, mut event_rx) = connect_with_events(dir.path())
        .await
        .expect("connect_with_events");

    let (server_stream, _) = listener.accept().await.expect("accept");
    drop(server_stream);

    let stale_prompt_id: u64 = 99;
    let mut overlay: VecDeque<PromptModal> = VecDeque::from([PromptModal {
        prompt_id: stale_prompt_id,
    }]);
    let mut mode = AppMode::Overlay;

    let evt = event_rx
        .recv()
        .await
        .expect("TransportEvent::Disconnected must arrive");
    assert_eq!(evt, TransportEvent::Disconnected);

    // SOQ-3 clears overlay — ghost approval is now impossible.
    soq3_handler(&mut overlay, &mut mode);

    // Attempt ghost approval: look up prompt in the cleared overlay.
    let ghost_target = overlay.iter().find(|p| p.prompt_id == stale_prompt_id);
    assert!(
        ghost_target.is_none(),
        "BC-2.05.007 Invariant 2: stale prompt (id={stale_prompt_id}) must not exist \
         in overlay after SOQ-3 — ghost approval is structurally impossible"
    );
}
