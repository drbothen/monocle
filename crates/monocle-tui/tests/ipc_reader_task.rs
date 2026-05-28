//! Integration test for the dedicated IPC reader task (Option B — F-S025-ADV2-BLOCKER-001).
//!
//! This test proves that frame corruption does NOT occur when the daemon emits messages
//! with inter-message gaps that would previously have triggered the cancellation-unsafe
//! `tokio::time::timeout(Duration::from_millis(1), read_framed(...))` wrapper.
//!
//! # Why this test validates the fix
//!
//! Under the old pattern (1ms outer timeout), any gap between messages exceeding 1ms
//! would cause `tokio::time::timeout` to drop the `read_framed` future mid-frame.
//! `read_framed` calls `read_exact(&mut len_buf)` followed by `read_exact(&mut payload)`
//! in sequence — neither is cancellation-safe. Bytes consumed from the kernel buffer on
//! the first `read_exact` are lost when the future is dropped, causing frame desync.
//!
//! Under Option B, the reader task holds the `read_framed` future to completion regardless
//! of event loop cadence. The >5ms gap between messages guarantees the old timeout would
//! have fired; under the new pattern all N messages arrive intact.
//!
//! Traces to: BC-2.05.002 Postcondition 6, F-S025-ADV2-BLOCKER-001.

use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_ipc::framing::write_framed;
use monocle_ipc::types::ServerToClient;
use monocle_tui::app::spawn_ipc_reader;
use std::time::Duration;
use tokio::io::DuplexStream;
use tokio::sync::mpsc::error::TryRecvError;

/// Convenience error type for test assertions: converts any error into a test failure.
type TestResult = anyhow::Result<()>;

/// Test that N=10 messages with >5ms inter-message gaps are received without frame corruption.
///
/// Asserts:
/// - Exactly N=10 messages received.
/// - Each decoded as `ServerToClient::SessionListUpdate`.
/// - Session list contents match emission order (message i has session with session_id `"si"`).
#[tokio::test]
async fn test_bc_2_05_002_pc_6_no_frame_corruption_across_inter_message_gap() -> TestResult {
    const N: usize = 10;
    const INTER_MESSAGE_GAP_MS: u64 = 8; // >5ms: guarantees old 1ms timeout would have fired

    // Create an in-process duplex pipe — no UDS socket needed for unit-level isolation.
    // `tokio::io::duplex` returns two `DuplexStream` halves, each implementing
    // `AsyncRead + AsyncWrite`. We treat `server_side` as the writer (daemon) and
    // `client_side` as the reader (TUI), matching the real-world data direction.
    let (client_side, server_side): (DuplexStream, DuplexStream) = tokio::io::duplex(65536);

    // Spawn the mock daemon writer: emits N SessionListUpdate messages with inter-message gaps.
    // Returns Err(IpcError) if any write fails so the test can observe it.
    let writer_handle = tokio::spawn(async move {
        let mut writer = server_side;
        for i in 1..=N {
            let session_id = format!("s{i}");
            let msg = ServerToClient::SessionListUpdate {
                sessions: vec![EnrichedSession::new(
                    session_id,
                    "claude-code".to_string(),
                    None, // transcript_path
                    None, // config_path
                    SessionStatus::Active,
                    None, // last_event_micros
                    None, // project_name
                    None, // started_at
                    0,    // token_count
                    None, // cost_usd
                )],
            };
            write_framed(&mut writer, &msg).await?;
            // Sleep >1ms to ensure the old 1ms timeout wrapper would have dropped the future.
            tokio::time::sleep(Duration::from_millis(INTER_MESSAGE_GAP_MS)).await;
        }
        // `writer` drops here: EOF on the reader side → IpcError::Disconnected forwarded by reader task.
        Ok::<(), monocle_ipc::error::IpcError>(())
    });

    // Spawn the dedicated reader task (Option B — the fix under test).
    let (ipc_tx, mut ipc_rx) =
        tokio::sync::mpsc::channel::<Result<ServerToClient, monocle_ipc::error::IpcError>>(64);
    let reader_handle = spawn_ipc_reader(client_side, ipc_tx);

    // Collect all N messages with a generous deadline.
    let timeout_ms = (N as u64) * (INTER_MESSAGE_GAP_MS + 50);
    let mut received: Vec<ServerToClient> = Vec::with_capacity(N);

    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if tokio::time::Instant::now() >= deadline {
            panic!(
                "timeout: received {}/{} messages before deadline — possible frame corruption \
                 or regression to the old cancellation-unsafe pattern",
                received.len(),
                N
            );
        }

        match ipc_rx.try_recv() {
            Ok(Ok(msg)) => {
                received.push(msg);
                if received.len() == N {
                    break;
                }
            }
            Ok(Err(e)) => {
                // Forwarded disconnect from EOF after all N messages — expected.
                // If we received all N already, this is normal. Otherwise it's a failure.
                if received.len() < N {
                    panic!(
                        "IPC disconnect before all messages received ({}/{}): {e}",
                        received.len(),
                        N
                    );
                }
                break;
            }
            Err(TryRecvError::Empty) => {
                // No message available yet — yield and retry.
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            Err(TryRecvError::Disconnected) => {
                if received.len() < N {
                    panic!(
                        "reader task exited before all messages received ({}/{})",
                        received.len(),
                        N
                    );
                }
                break;
            }
        }
    }

    // Assert: exactly N messages received (no drops, no extras).
    assert_eq!(
        received.len(),
        N,
        "expected {N} messages, got {}",
        received.len()
    );

    // Assert: each message is SessionListUpdate with the correct session_id in order.
    for (i, msg) in received.iter().enumerate() {
        let expected_id = format!("s{}", i + 1);
        match msg {
            ServerToClient::SessionListUpdate { sessions } => {
                assert_eq!(
                    sessions.len(),
                    1,
                    "message {}: expected 1 session, got {}",
                    i + 1,
                    sessions.len()
                );
                assert_eq!(
                    sessions[0].session_id,
                    expected_id,
                    "message {}: expected session_id={expected_id:?}, got {:?}",
                    i + 1,
                    sessions[0].session_id
                );
            }
            other => {
                panic!(
                    "message {}: expected SessionListUpdate, got {other:?}",
                    i + 1
                );
            }
        }
    }

    // Clean up — abort reader task and await the writer.
    reader_handle.abort();
    let write_result = writer_handle.await.unwrap_or_else(|e| {
        Err(monocle_ipc::error::IpcError::IoError(
            std::io::Error::other(e),
        ))
    });
    assert!(
        write_result.is_ok(),
        "mock writer task failed: {write_result:?}"
    );

    Ok(())
}
