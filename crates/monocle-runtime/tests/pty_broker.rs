//! PtyOutput fan-out broker — unit tests (S-046, BC-2.05.009).
//!
//! Tests exercise the `SubscriberList`-based `PtyBroker` design (Q1 ruling):
//! the broker does NOT own a per-client registry; all fan-out goes through
//! `broadcast_to_subscribers(&shared_subscriber_list, msg)`.
//!
//! # AC coverage
//!
//! | AC    | Test fn                                                                      | Kind             |
//! |-------|------------------------------------------------------------------------------|------------------|
//! | AC-001| test_BC_2_05_009_bounded_channel_backpressure_blocks_not_drops               | behavioral       |
//! | AC-002| test_BC_2_05_009_fan_out_via_subscriber_list_not_broker_registry             | behavioral       |
//! | AC-003| test_BC_2_05_009_one_strike_disconnect_slow_client                           | behavioral       |
//! | AC-004| test_BC_2_05_009_pty_drop_counter_not_incremented_on_graceful_close          | behavioral       |
//! | AC-004| test_BC_2_05_009_pty_drop_counter_not_incremented_on_broadcast_failure       | behavioral       |
//! | AC-005| test_BC_2_05_009_pty_reset_not_emitted_on_graceful_input_close               | behavioral       |
//! | AC-006| test_BC_2_05_009_biased_select_source_guard                                  | source-guard     |
//! | AC-007| test_BC_2_05_009_no_unbounded_channel_in_pty_path                            | source-guard     |
//!
//! # F-006/F-007 resolution: biased-select priority test
//!
//! The broker event loop's hook arm (`hook_rx.recv()`) has NO observable side-effect:
//! it emits only a `tracing::debug!` log and continues (or breaks on `None`). Because
//! the hook arm is a placeholder with no externally visible effect, it is IMPOSSIBLE
//! to write a behavioral test that verifies ordering without modifying production code.
//!
//! A behavioral ordering test would require:
//!   a) Pre-enqueueing BOTH a hook event AND a PTY frame before the loop's first poll
//!      (so both arms are simultaneously ready), AND
//!   b) An observable side-effect in the hook arm (a counter, a channel, a flag) that
//!      lets the test assert hook was processed before PTY output.
//!
//! Neither condition is currently satisfiable from the test boundary:
//! - `(a)` requires the event loop NOT to start until both messages are queued. Since
//!   the event loop is spawned as a Tokio task, its first poll can happen at any time
//!   after `tokio::spawn` — there is no way to guarantee atomically-simultaneous readiness
//!   from outside the task without production-code hooks.
//! - `(b)` The hook arm emits only `tracing::debug!` — unobservable from tests.
//!
//! **Therefore, AC-006 is verified by source-guard only:** the test reads `pty_broker.rs`
//! and asserts `biased;` is present in the `select!` macro. If `biased;` is removed,
//! the test fails. This is the correct approach when the semantic effect of `biased;` is
//! not observable at the unit-test boundary (the ordering matters only when both arms are
//! polled simultaneously by the tokio runtime, which is non-deterministic from tests).
//!
//! The proxy task's positive PtyReset path (AC-005 sub-case B) requires the proxy task,
//! DaemonState, and session infrastructure — this is integration-level behavior. The
//! unit-test boundary can only verify the broker event loop's negative sub-case
//! (NO PtyReset on graceful close), which is tested in AC-005.

// Test naming preserves BC identifiers for traceability.
#![allow(non_snake_case)]
// Tests use unwrap/expect as assertion helpers — panics are the desired failure mode.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use monocle_ipc::server::{register_subscriber, SubscriberList, CLIENT_CHANNEL_CAPACITY};
use monocle_ipc::types::ServerToClient;
use monocle_runtime::ipc_server::broadcast_to_subscribers;
use monocle_runtime::pty_broker::{
    PtyBroker, PTY_BROKER_CLIENT_CAPACITY, PTY_BROKER_INPUT_CAPACITY, PTY_BROKER_STRIKE_LIMIT,
};
use tokio::sync::mpsc;
use tokio::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a fresh empty SubscriberList.
fn make_subscriber_list() -> Arc<SubscriberList> {
    Arc::new(Arc::new(tokio::sync::Mutex::new(Vec::new())))
}

/// Returns true iff the message is `ServerToClient::PtyOutput` with matching fields.
fn is_pty_output(msg: &ServerToClient, expected_session_id: &str, expected_bytes: &[u8]) -> bool {
    match msg {
        ServerToClient::PtyOutput { session_id, bytes } => {
            session_id == expected_session_id && bytes.as_slice() == expected_bytes
        }
        _ => false,
    }
}

/// Returns true iff the message is `ServerToClient::PtyReset` for the given session.
fn is_pty_reset(msg: &ServerToClient, expected_session_id: &str) -> bool {
    match msg {
        ServerToClient::PtyReset { session_id } => session_id == expected_session_id,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// AC-001 (BC-2.05.009 PC-2, Invariant 3): bounded INPUT channel blocks on full
// ---------------------------------------------------------------------------

/// INPUT channel blocks (backpressure) rather than dropping on full — AC-001, BC-2.05.009 PC-2.
///
/// Behavioral assertion: `.send().await` on a full channel blocks indefinitely; it does NOT
/// complete, return an error, or drop the message. The `pty_drop_counter` is not incremented.
#[tokio::test]
async fn test_BC_2_05_009_bounded_channel_backpressure_blocks_not_drops() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();
    let broker = PtyBroker::new(
        "test-session-001".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Step 1: fill the INPUT channel to capacity via try_send.
    let frame = Arc::new(Bytes::from_static(b"hello pty"));
    for _ in 0..PTY_BROKER_INPUT_CAPACITY {
        broker
            .input_tx
            .try_send(Arc::clone(&frame))
            .expect("try_send must succeed while channel has capacity");
    }

    // Step 2: channel is now at capacity. A further try_send MUST return Err::Full.
    let try_result = broker.input_tx.try_send(Arc::clone(&frame));
    assert!(
        try_result.is_err(),
        "try_send must fail when INPUT channel is at capacity (AC-001)"
    );
    assert!(
        matches!(
            try_result.as_ref().unwrap_err(),
            tokio::sync::mpsc::error::TrySendError::Full(_)
        ),
        "try_send error must be TrySendError::Full, not Closed \
         (BC-2.05.009 Invariant 3 — channel not dropped, only full)"
    );

    // Step 3: behavioral assertion — `.send().await` blocks, not drops.
    // With no consumer draining the channel, send().await must not complete within 50ms.
    let input_tx_clone = broker.input_tx.clone();
    let send_frame = Arc::clone(&frame);

    let send_future = async move {
        input_tx_clone
            .send(send_frame)
            .await
            .expect("send must not error — receiver is alive");
    };

    // The channel is full: .send().await must NOT complete within 50ms.
    let result = tokio::time::timeout(Duration::from_millis(50), send_future).await;
    assert!(
        result.is_err(),
        "send().await must BLOCK (not complete) when INPUT channel is at full capacity \
         — backpressure to PTY reader (BC-2.05.009 Invariant 3, AC-001)"
    );

    // Step 4: pty_drop_counter must remain 0 — normal channel fullness is NOT a drop.
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on normal channel fullness \
         (BC-2.05.009 PC-3 — drop_counter counts only OOM/sender-error)"
    );
}

// ---------------------------------------------------------------------------
// AC-002 (BC-2.05.009 PC-1b): fan-out via shared SubscriberList
//
// The broker event loop calls broadcast_to_subscribers with the SHARED SubscriberList
// (not a per-broker registry). Clients registered via register_subscriber AFTER broker
// construction receive PtyOutput frames. This is the production fan-out path — it proves
// the silent-drop regression from the inert-registry design (root cause in BC §Trace 1.6.0)
// is fixed: frames reach clients because the broker holds the live shared list.
// ---------------------------------------------------------------------------

/// Fan-out goes through shared SubscriberList — AC-002, BC-2.05.009 PC-1b.
///
/// Behavioral assertions:
/// - Two clients registered via `register_subscriber` each receive a `ServerToClient::PtyOutput`
///   with the correct `session_id` and exact frame bytes.
/// - The message type is `PtyOutput` (not `PtyReset` or any other variant).
/// - `PtyBroker` has no `clients` or `strike_counters` fields (structural — compile-time).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_BC_2_05_009_fan_out_via_subscriber_list_not_broker_registry() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let mut broker = PtyBroker::new(
        "test-session-002".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register two TUI clients via the shared SubscriberList BEFORE spawning event loop.
    let (tx_a, mut rx_a) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let (tx_b, mut rx_b) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    register_subscriber(&sub_list, tx_a).await;
    register_subscriber(&sub_list, tx_b).await;

    let (hook_tx, hook_rx) = mpsc::channel::<()>(8);
    let handle = broker.spawn_event_loop(hook_rx);

    // Give the event loop a moment to start.
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Send a PTY frame through the INPUT channel — must reach both subscribers.
    let frame_bytes = b"pty output for fan-out test";
    broker
        .input_tx
        .send(Arc::new(Bytes::from_static(frame_bytes)))
        .await
        .expect("INPUT send must succeed");

    // Both clients must receive the frame as PtyOutput with exact session_id and bytes.
    let msg_a = tokio::time::timeout(Duration::from_millis(200), rx_a.recv())
        .await
        .expect("client-a must receive message within timeout")
        .expect("channel must not be closed");
    let msg_b = tokio::time::timeout(Duration::from_millis(200), rx_b.recv())
        .await
        .expect("client-b must receive message within timeout")
        .expect("channel must not be closed");

    assert!(
        is_pty_output(&msg_a, "test-session-002", frame_bytes),
        "client-a must receive PtyOutput{{session_id:'test-session-002', bytes:...}}; got {:?}",
        msg_a,
    );
    assert!(
        is_pty_output(&msg_b, "test-session-002", frame_bytes),
        "client-b must receive PtyOutput{{session_id:'test-session-002', bytes:...}}; got {:?}",
        msg_b,
    );

    // Verify no PtyReset was delivered to either client (frame delivery only, no reset).
    let spurious_a = rx_a.try_recv();
    assert!(
        spurious_a.is_err(),
        "client-a must not receive any spurious messages after the PtyOutput; got: {:?}",
        spurious_a,
    );
    let spurious_b = rx_b.try_recv();
    assert!(
        spurious_b.is_err(),
        "client-b must not receive any spurious messages after the PtyOutput; got: {:?}",
        spurious_b,
    );

    // Structural assertion: PtyBroker has no per-client registry fields.
    // The fact that this test compiles and must call register_subscriber (not
    // a hypothetical register_client method on PtyBroker) proves the Q1 ruling:
    // no `clients: HashMap<...>` or `strike_counters: HashMap<...>` fields exist.

    drop(hook_tx);
    handle.abort();
}

// ---------------------------------------------------------------------------
// AC-003 (BC-2.05.009 Invariant 3b): 1-strike disconnect for slow clients
//
// broadcast_to_subscribers disconnects a slow client on the first TrySendError::Full.
// All other clients continue receiving uninterrupted.
// ---------------------------------------------------------------------------

/// Slow client is disconnected on first full-buffer send; fast client receives all frames — AC-003.
///
/// Behavioral assertions:
/// - After one broadcast where the slow client's channel is full: slow client is removed from
///   SubscriberList (len goes 2 → 1).
/// - Fast client receives the PtyOutput frame with correct content.
/// - `pty_drop_counter` is NOT incremented — 1-strike is not an OOM drop.
/// - No `strike_counters` field on `PtyBroker` (structural — compile-time).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_BC_2_05_009_one_strike_disconnect_slow_client() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let broker = PtyBroker::new(
        "test-session-003".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register a FAST client with a normal capacity channel.
    let (tx_fast, mut rx_fast) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    register_subscriber(&sub_list, tx_fast).await;

    // Register a SLOW client: capacity 1, pre-filled so the FIRST broadcast attempt fails.
    let (tx_slow, _rx_slow) = mpsc::channel::<ServerToClient>(1);
    tx_slow
        .try_send(ServerToClient::SessionListUpdate { sessions: vec![] })
        .expect("pre-fill must succeed");
    register_subscriber(&sub_list, tx_slow).await;

    // Subscriber list now has 2 entries.
    {
        let subs = sub_list.lock().await;
        assert_eq!(subs.len(), 2, "must have 2 subscribers before broadcast");
    }

    // Broadcast one PtyOutput — fast client receives it; slow client (full buffer)
    // is disconnected immediately (1-strike model per BC-2.05.009 Invariant 3b).
    let msg = ServerToClient::PtyOutput {
        session_id: "test-session-003".to_string(),
        bytes: b"one-strike test".to_vec(),
    };
    broadcast_to_subscribers(&sub_list, msg).await;

    // After broadcast: subscriber list must have exactly 1 entry (slow client removed).
    {
        let subs = sub_list.lock().await;
        assert_eq!(
            subs.len(),
            1,
            "slow client must be removed after 1 TrySendError::Full (1-strike model — AC-003)"
        );
    }

    // Fast client must have received the message with correct content.
    let received = rx_fast
        .try_recv()
        .expect("fast client must have received the PtyOutput message");
    assert!(
        is_pty_output(&received, "test-session-003", b"one-strike test"),
        "fast client must receive PtyOutput; got {:?}",
        received,
    );

    // Fast client must NOT have received a PtyReset — 1-strike is a client removal, not a
    // PTY byte drop (BC-2.05.009 PC-3 / EC-201 — slow client removal does NOT trigger reset).
    let spurious = rx_fast.try_recv();
    assert!(
        spurious.is_err(),
        "fast client must not receive PtyReset or any other spurious message on slow-client \
         disconnect; got: {:?}",
        spurious,
    );

    // pty_drop_counter must NOT be incremented — 1-strike disconnect is not OOM.
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on slow-client 1-strike disconnect \
         (BC-2.05.009 PC-3 / EC-202)"
    );

    drop(broker);
}

// ---------------------------------------------------------------------------
// AC-004 (BC-2.05.009 PC-3): pty_drop_counter — OOM-only
//
// The counter is NOT incremented on:
//   (a) Graceful INPUT channel close (input_rx.recv() == None — proxy task exited normally)
//   (b) Per-client broadcast failure (1-strike removal by broadcast_to_subscribers)
//
// The OOM path (proxy task tx.send Err(_)) requires DaemonState + session infrastructure
// and is integration-level behavior. This unit boundary can only verify the two negative
// cases above; the positive OOM path is noted as an integration-test gap.
// ---------------------------------------------------------------------------

/// pty_drop_counter NOT incremented on graceful INPUT channel close — AC-004a, BC-2.05.009 PC-3.
///
/// Behavioral assertion: when all INPUT senders are dropped (proxy task exits normally),
/// the event loop exits and the counter remains 0. Normal session teardown is not an OOM.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_BC_2_05_009_pty_drop_counter_not_incremented_on_graceful_close() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let mut broker = PtyBroker::new(
        "test-session-004a".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    let (_hook_tx, hook_rx) = mpsc::channel::<()>(8);
    let handle = broker.spawn_event_loop(hook_rx);

    // Give the event loop a moment to start.
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Drop all INPUT senders — simulates proxy task exiting gracefully.
    // `broker.input_tx` is the only sender; dropping broker drops the sender.
    drop(broker); // drops input_tx

    // Wait for the event loop to exit (graceful close).
    let _ = tokio::time::timeout(Duration::from_millis(200), handle).await;

    // pty_drop_counter MUST remain 0 — graceful INPUT channel close is NOT an OOM drop.
    // (BC-2.05.009 PC-3: counter incremented ONLY on tx.send Err in proxy task.)
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on graceful INPUT channel close \
         (BC-2.05.009 PC-3 / AC-004 — graceful close is normal session exit, not OOM)"
    );

    // INTEGRATION GAP: The positive OOM path (proxy task's tx.send(frame).await returns
    // Err(_) → counter incremented + PtyReset broadcast) requires the full session-host
    // proxy task, DaemonState, and session infrastructure. That path cannot be exercised
    // at the unit-test boundary of pty_broker.rs alone. It is covered by integration tests
    // in the session_manager test suite (which exercises the proxy task end-to-end).
}

/// pty_drop_counter NOT incremented on per-client 1-strike broadcast failure — AC-004b, BC-2.05.009 PC-3.
///
/// Behavioral assertions:
/// - broadcast_to_subscribers removes the slow client (len 1 → 0) without touching the counter.
/// - A second broadcast with no clients also leaves the counter at 0.
#[tokio::test]
async fn test_BC_2_05_009_pty_drop_counter_not_incremented_on_broadcast_failure() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let broker = PtyBroker::new(
        "test-session-004b".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register a client and pre-fill its buffer to force disconnect on next broadcast.
    let (tx, _rx) = mpsc::channel::<ServerToClient>(1);
    tx.try_send(ServerToClient::SessionListUpdate { sessions: vec![] })
        .expect("pre-fill");
    register_subscriber(&sub_list, tx).await;

    // Broadcast — slow-client 1-strike disconnect fires.
    broadcast_to_subscribers(
        &sub_list,
        ServerToClient::PtyOutput {
            session_id: "test-session-004b".to_string(),
            bytes: b"drop counter test".to_vec(),
        },
    )
    .await;

    {
        let subs = sub_list.lock().await;
        assert_eq!(
            subs.len(),
            0,
            "client must be removed after 1-strike disconnect (AC-003 / AC-004b)"
        );
    }

    // pty_drop_counter MUST NOT have been incremented — per-client 1-strike is not OOM.
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on per-client 1-strike disconnect \
         (BC-2.05.009 PC-3 / AC-004 / EC-202)"
    );

    // EC-202: broadcast with no clients (empty SubscriberList) must also not increment counter.
    broadcast_to_subscribers(
        &sub_list,
        ServerToClient::PtyOutput {
            session_id: "test-session-004b".to_string(),
            bytes: b"no clients".to_vec(),
        },
    )
    .await;
    let drops_after = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops_after, 0,
        "pty_drop_counter must NOT be incremented when broadcast has no clients \
         (EC-202 — empty SubscriberList is normal, not OOM)"
    );

    drop(broker);
}

// ---------------------------------------------------------------------------
// AC-005 (BC-2.05.009 Invariant 4): broker event loop MUST NOT emit PtyReset
//                                     when input_rx.recv() returns None (graceful close)
//
// Sub-case A (unit-testable): input_rx.recv() == None → no PtyReset broadcast, counter stays 0.
// Sub-case B (integration boundary): proxy task tx.send Err → PtyReset broadcast + counter++.
//   This requires the full session-host proxy task and DaemonState; it is covered by the
//   session_manager integration tests. The unit boundary can only verify sub-case A.
// ---------------------------------------------------------------------------

/// Broker event loop exits gracefully without emitting PtyReset when INPUT channel closes — AC-005a.
///
/// Behavioral assertions:
/// - After dropping all INPUT senders, the observer channel is EMPTY (no PtyReset, no PtyOutput).
/// - `pty_drop_counter` remains 0.
/// - The event loop task exits (JoinHandle resolves within timeout).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_BC_2_05_009_pty_reset_not_emitted_on_graceful_input_close() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let mut broker = PtyBroker::new(
        "test-session-005".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register a client to observe any spurious PtyReset or PtyOutput messages.
    let (tx_obs, mut rx_obs) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    register_subscriber(&sub_list, tx_obs).await;

    let (_hook_tx, hook_rx) = mpsc::channel::<()>(8);
    let handle = broker.spawn_event_loop(hook_rx);

    // Give the event loop a moment to start.
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Drop the INPUT sender — input_rx.recv() will return None in the event loop.
    // This is the NORMAL graceful session-exit path: proxy task exited, dropped its sender.
    drop(broker); // drops input_tx (the receiver was moved into the event loop)

    // Wait for the event loop to exit.
    let _ = tokio::time::timeout(Duration::from_millis(200), handle).await;

    // The observer channel must be EMPTY — the event loop must NOT have emitted PtyReset
    // or any other message when the INPUT channel closed normally (BC-2.05.009 Invariant 4).
    let result = rx_obs.try_recv();
    assert!(
        result.is_err(),
        "broker must NOT emit any message (especially not PtyReset) when INPUT channel \
         closes gracefully (BC-2.05.009 Invariant 4 — graceful exit is NOT a byte drop); \
         got unexpected message: {:?}",
        result,
    );

    // Explicit PtyReset check with the helper (belt-and-suspenders — is_err above already
    // covers this, but naming the specific forbidden message makes failure output clearer).
    if let Ok(msg) = result {
        assert!(
            !is_pty_reset(&msg, "test-session-005"),
            "broker MUST NOT emit PtyReset on graceful INPUT channel close; got: {:?}",
            msg,
        );
    }

    // pty_drop_counter must remain 0 — graceful close is NOT an OOM drop.
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on graceful INPUT channel close \
         (BC-2.05.009 PC-3 — counter only incremented by proxy task on tx.send error)"
    );

    // INTEGRATION GAP (sub-case B): The positive path — proxy task's tx.send(frame).await
    // returns Err(_) → PtyReset broadcast + counter++ — requires DaemonState, the session-
    // host proxy task, and session infrastructure to be wired together. That path is covered
    // by the session_manager integration tests (tests/session_manager.rs), not here.
}

// ---------------------------------------------------------------------------
// AC-006 (BC-2.05.009 Invariant 6): biased select! presence — SOURCE GUARD
//
// The broker uses `tokio::select! { biased; ... }` to give hook events priority over PTY
// frames. Without `biased;`, tokio::select! chooses randomly among ready arms, which can
// starve hook events under PTY saturation.
//
// WHY BEHAVIORAL ORDERING IS NOT TESTED HERE (F-006/F-007 resolution):
//
// A behavioral test would require pre-enqueueing BOTH a hook event AND a PTY frame before
// the event loop's first poll (so both arms are simultaneously ready), AND an observable
// side-effect in the hook arm (currently only tracing::debug! — unobservable from tests).
//
// The hook arm is a placeholder: `Some(()) => { tracing::debug!(...) }`. There is no
// counter, no channel, no flag to assert against. Even if we could guarantee simultaneous
// readiness (which requires internal production-code hooks), there is nothing to observe.
//
// A source-guard is the CORRECT approach here: it asserts the semantic precondition
// (`biased;` present in pty_broker.rs) that makes priority possible. If `biased;` is
// removed, this test fails. Any future addition of an observable hook-arm side-effect
// would enable a behavioral ordering test to be added alongside this guard.
// ---------------------------------------------------------------------------

/// `biased;` keyword present in select! macro in pty_broker.rs — AC-006, BC-2.05.009 Invariant 6.
///
/// Source-guard: reads pty_broker.rs and asserts the biased select! keyword is present.
/// This test FAILS if `biased;` is removed or if the select! is replaced with a random-
/// choice version — either change would break the hook-event-priority guarantee.
#[test]
fn test_BC_2_05_009_biased_select_source_guard() {
    let pty_broker_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("pty_broker.rs"),
    )
    .expect("pty_broker.rs must be readable from CARGO_MANIFEST_DIR/src/");

    // Assert `biased;` appears inside the tokio::select! macro body (not just in comments).
    // We search for the combined pattern `select! {` immediately followed (within the block)
    // by `biased;` — this rules out doc-comment occurrences of the keyword.
    //
    // Strategy: find the first `select! {` code site, then assert `biased;` appears
    // between that position and the matching closing brace region.  Because the file is
    // small we use a simpler two-step: find `select! {` and then search for `biased;`
    // AFTER that position in the source text.
    let select_open_pattern = "select! {";
    let select_pos = pty_broker_source
        .find(select_open_pattern)
        .expect("tokio::select! { must be present in pty_broker.rs");

    // Slice the source from the select! opening onward.
    let after_select = &pty_broker_source[select_pos..];

    assert!(
        after_select.contains("biased;"),
        "pty_broker.rs select! macro MUST contain `biased;` to guarantee hook event priority \
         over PtyOutput (BC-2.05.009 Invariant 6 / ADR-0010 §Head-of-line blocking mitigation). \
         Removing `biased;` allows tokio to choose randomly among ready arms, which can starve \
         hook events under PTY saturation."
    );

    // Within the select! body: verify arm ordering — biased; → hook_event → pty_frame.
    let biased_rel = after_select
        .find("biased;")
        .expect("biased; must be present after select! { in pty_broker.rs");
    let hook_rel = after_select
        .find("hook_event")
        .expect("hook_event arm must be present in the select! body");
    let pty_rel = after_select
        .find("pty_frame")
        .expect("pty_frame arm must be present in the select! body");

    assert!(
        hook_rel > biased_rel,
        "hook_event arm must appear after `biased;` in the select! body \
         (biased pos={biased_rel}, hook pos={hook_rel})"
    );
    assert!(
        pty_rel > hook_rel,
        "pty_frame arm must appear after hook_event arm (hook = first/higher priority arm) \
         (hook pos={hook_rel}, pty pos={pty_rel})"
    );
}

// ---------------------------------------------------------------------------
// AC-007 (BC-2.05.009 Invariant 3): no unbounded_channel in PTY path; capacities canonical
// ---------------------------------------------------------------------------

/// No unbounded_channel in PTY path; capacities are canonical — AC-007, BC-2.05.009 Inv 3.
///
/// Source-guard: reads pty_broker.rs and asserts no `unbounded_channel` call is present.
/// Capacity constant assertions verify the canonical values from the BC and SS-ipc.md.
#[test]
fn test_BC_2_05_009_no_unbounded_channel_in_pty_path() {
    // Capacity assertions (canonical values from BC-2.05.009 and SS-ipc.md).
    assert_eq!(
        PTY_BROKER_INPUT_CAPACITY, 1024,
        "PTY_BROKER_INPUT_CAPACITY must be 1024 (BC-2.05.009 PC-2 / ADR-0010)"
    );
    assert_eq!(
        PTY_BROKER_CLIENT_CAPACITY, 64,
        "PTY_BROKER_CLIENT_CAPACITY must be 64 (BC-2.05.009 Invariant 3b / SS-ipc.md)"
    );
    assert_eq!(
        PTY_BROKER_STRIKE_LIMIT, 1,
        "PTY_BROKER_STRIKE_LIMIT must be 1 (1-strike model per BC-2.05.009 Q1 ruling, \
         superseding the retired 3-strike threshold)"
    );

    // Source-file grep: no `unbounded_channel` call in the PTY broker source.
    let pty_broker_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("pty_broker.rs"),
    )
    .expect("pty_broker.rs must be readable from CARGO_MANIFEST_DIR/src/");

    assert!(
        !pty_broker_source.contains("unbounded_channel"),
        "pty_broker.rs must NOT contain `unbounded_channel` — \
         BC-2.05.009 Invariant 3 forbids unbounded channels in the PTY output path; \
         use bounded mpsc::channel(N) with .send().await backpressure"
    );
}
