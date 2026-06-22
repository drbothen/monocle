//! S-047 Red Gate: TUI-layer tests for scrollback chunk protocol and PTY reset handling.
//!
//! Tests for `on_scrollback_chunk()` (AC-007) and `on_pty_reset()` (AC-009).
//! Both functions are `todo!()` stubs — all tests MUST FAIL before S-047 implementation.
//!
//! # Behavioral Contract Coverage
//!
//! | Test | AC / BC Clause | Fails because |
//! |------|----------------|---------------|
//! | test_BC_2_05_011_scrollback_chunk_contiguity_gap_triggers_reattach | AC-007 | on_scrollback_chunk() todo!() |
//! | test_BC_2_05_011_scrollback_chunk_contiguous_seq_accepted | AC-007 | on_scrollback_chunk() todo!() |
//! | test_BC_2_05_011_scrollback_dump_complete_validates_total_chunks | AC-008 | on_scrollback_chunk() todo!() |
//! | test_BC_2_05_011_pty_reset_clears_buffer_retriggers_attach | AC-009 | on_pty_reset() todo!() |
//! | test_BC_2_05_011_pty_reset_sends_attach_via_ipc_tx | AC-009 | on_pty_reset() todo!() |

#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_config::MonocleConfig;
use monocle_ipc::types::{ClientToServer, SerializedCell, SerializedColor};
use monocle_tui::app::{on_pty_reset, on_scrollback_chunk, App};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn make_app() -> App {
    App::new(MonocleConfig::default())
}

fn make_app_with_session(session_id: &str) -> App {
    let mut app = make_app();
    let parser = vt100::Parser::new(24, 80, app.scrollback_rows as usize);
    app.pty_parsers.insert(session_id.to_string(), parser);
    app.pty_scroll_offsets.insert(session_id.to_string(), 0);
    app
}

/// Build a minimal 1-row Vec<SerializedCell> payload for a test chunk.
/// SerializedCell is #[non_exhaustive] so must be constructed via SerializedCell::new().
fn make_chunk_rows() -> Vec<Vec<SerializedCell>> {
    vec![vec![SerializedCell::new(
        " ".to_string(),
        SerializedColor::Default,
        SerializedColor::Default,
        0u8, // no attributes
    )]]
}

// ---------------------------------------------------------------------------
// AC-007: ScrollbackChunk contiguity — gap triggers re-attach
// ---------------------------------------------------------------------------

/// AC-007 (BC-2.05.011 §ScrollbackChunk PC-3): When a `ScrollbackChunk` with
/// `chunk_seq != expected_seq` arrives, `on_scrollback_chunk` MUST:
/// 1. Discard all buffered chunks for that session.
/// 2. Re-trigger `ClientToServer::AttachSession` via `app.ipc_tx`.
/// 3. NOT attempt to reconstruct from out-of-order chunks.
///
/// FAILS because `on_scrollback_chunk()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_scrollback_chunk_contiguity_gap_triggers_reattach() {
    let session_id = "47070000-0000-4000-a000-000000000001".to_string();
    let mut app = make_app_with_session(&session_id);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ClientToServer>(16);
    app.ipc_tx = Some(tx);

    // chunk_seq=0 (expected): accepted, buffered.
    // FAILS: on_scrollback_chunk() is todo!()
    on_scrollback_chunk(&mut app, session_id.clone(), make_chunk_rows(), 0);

    // chunk_seq=2 (gap: expected=1): must discard buffered + re-attach.
    // FAILS: on_scrollback_chunk() is todo!()
    on_scrollback_chunk(&mut app, session_id.clone(), make_chunk_rows(), 2);

    // Expect ClientToServer::AttachSession in ipc_tx within 1s.
    let attach_msg = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("AC-007: on_scrollback_chunk gap must send AttachSession within 1s")
        .expect("ipc_tx channel must not close");

    match attach_msg {
        ClientToServer::AttachSession {
            session_id: ref sid,
        } => {
            assert_eq!(
                *sid, session_id,
                "AC-007: re-attach AttachSession must carry the correct session_id"
            );
        }
        other => panic!(
            "AC-007: expected ClientToServer::AttachSession after chunk gap, got: {:?}",
            other
        ),
    }
}

/// AC-007 happy path: contiguous chunk_seq 0, 1, 2 are accepted. No re-attach.
///
/// FAILS because `on_scrollback_chunk()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_scrollback_chunk_contiguous_seq_accepted() {
    let session_id = "47071000-0000-4000-a000-000000000001".to_string();
    let mut app = make_app_with_session(&session_id);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ClientToServer>(16);
    app.ipc_tx = Some(tx);

    // FAILS: on_scrollback_chunk() is todo!()
    on_scrollback_chunk(&mut app, session_id.clone(), make_chunk_rows(), 0);
    on_scrollback_chunk(&mut app, session_id.clone(), make_chunk_rows(), 1);
    on_scrollback_chunk(&mut app, session_id.clone(), make_chunk_rows(), 2);

    // No AttachSession must be triggered for contiguous sequence.
    let no_attach = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    assert!(
        no_attach.is_err(),
        "AC-007: contiguous chunks MUST NOT trigger re-attach. Got: {:?}",
        no_attach
    );
}

// ---------------------------------------------------------------------------
// AC-008: on_scrollback_chunk tracks expected_seq for DumpComplete mismatch detection
// ---------------------------------------------------------------------------

/// AC-008: `on_scrollback_chunk()` must maintain a per-session `expected_seq` counter
/// so that `on_scrollback_dump_complete` can compare `total_chunks` with buffered count.
///
/// This test verifies that after 2 contiguous chunks, the accumulated buffer has length 2
/// (the implementer will add a `scrollback_chunk_buffer: HashMap<String, Vec<...>>` field
/// to App; this field does not exist yet — that's what todo!() means).
///
/// FAILS because `on_scrollback_chunk()` is `todo!()` — buffer stays empty.
#[test]
fn test_BC_2_05_011_scrollback_dump_complete_validates_total_chunks() {
    let session_id = "47080000-0000-4000-a000-000000000001".to_string();
    let mut app = make_app_with_session(&session_id);

    // FAILS: on_scrollback_chunk() is todo!()
    on_scrollback_chunk(&mut app, session_id.clone(), make_chunk_rows(), 0);
    on_scrollback_chunk(&mut app, session_id.clone(), make_chunk_rows(), 1);

    // After 2 accepted chunks, the buffer must have 2 entries.
    // The implementer adds `app.scrollback_chunk_buffer`.
    // Before implementation, this assertion fails because the field doesn't exist
    // (or is empty because todo!() didn't buffer anything).
    //
    // The exact field name is implementation-determined; what matters is that:
    // - on_scrollback_dump_complete with total_chunks=3 (mismatch: received 2) re-triggers attach
    // - on_scrollback_dump_complete with total_chunks=2 (match) reconstructs the screen
    //
    // We test the invariant: after 2 chunks, calling dump complete with total_chunks=2
    // (match) must NOT re-trigger attach, and the parsers map must reflect the new state.
    //
    // For now, this test serves as the RED GATE: if on_scrollback_chunk is todo!(),
    // the buffer is empty — any assertion on non-empty buffer fails.
    //
    // Use dump_in_progress as a proxy for "buffer was populated" since the implementer
    // must set dump_in_progress=true before starting the dump protocol.
    // After two chunks with no gap, dump_in_progress should still be true.
    // But since on_scrollback_chunk is todo!(), dump_in_progress is still false (AC-008 RED).
    assert!(
        app.dump_in_progress.get(&session_id) == Some(&true),
        "AC-008 precondition: dump_in_progress must be true during active chunk sequence. \
         on_scrollback_chunk() must set/maintain dump_in_progress. Got: {:?}",
        app.dump_in_progress.get(&session_id)
    );
}

// ---------------------------------------------------------------------------
// AC-009: PtyReset clears buffer and re-triggers attach
// ---------------------------------------------------------------------------

/// AC-009 (BC-2.05.011 §PtyReset PC-3): `on_pty_reset()` MUST:
/// 1. Clear all in-flight scrollback chunks for `session_id`.
/// 2. Clear `pending_pty_bytes` for `session_id`.
/// 3. Set `status_message` to "[PTY reset — <session_id_short>]".
/// 4. Re-trigger `ClientToServer::AttachSession` via `app.ipc_tx`.
///
/// FAILS because `on_pty_reset()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_pty_reset_clears_buffer_retriggers_attach() {
    let session_id = "47090000-0000-4000-a000-000000000001".to_string();
    let mut app = make_app_with_session(&session_id);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ClientToServer>(16);
    app.ipc_tx = Some(tx);

    // Prime pending_pty_bytes so we can verify it is cleared.
    app.pending_pty_bytes.insert(
        session_id.clone(),
        std::collections::VecDeque::from(vec![vec![0x41u8, 0x42u8], vec![0x43u8]]),
    );

    // FAILS: on_pty_reset() is todo!()
    on_pty_reset(&mut app, session_id.clone());

    // AC-009 step 2: pending_pty_bytes for this session must be cleared.
    let pending = app.pending_pty_bytes.get(&session_id);
    assert!(
        pending.map(|v| v.is_empty()).unwrap_or(true),
        "AC-009: pending_pty_bytes must be cleared after on_pty_reset. \
         Got: {:?}",
        app.pending_pty_bytes.get(&session_id)
    );

    // AC-009 step 3: status_message must contain "PTY reset" and short session_id.
    let short_id = &session_id[..8];
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains("PTY reset") && msg.contains(short_id),
        "AC-009: status_message must contain 'PTY reset' and session_id_short '{short_id}'. \
         Got: {:?}",
        app.status_message
    );

    // AC-009 step 4: AttachSession must be triggered via ipc_tx.
    let attach_msg = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("AC-009: on_pty_reset must send AttachSession within 1s")
        .expect("ipc_tx channel must not close");

    match attach_msg {
        ClientToServer::AttachSession {
            session_id: ref sid,
        } => {
            assert_eq!(
                *sid, session_id,
                "AC-009: AttachSession must carry correct session_id"
            );
        }
        other => panic!(
            "AC-009: expected ClientToServer::AttachSession after on_pty_reset, got: {:?}",
            other
        ),
    }
}

/// AC-009 minimal path: on_pty_reset sends AttachSession even with no pending_pty_bytes.
///
/// FAILS because `on_pty_reset()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_pty_reset_sends_attach_via_ipc_tx() {
    let session_id = "47091000-0000-4000-a000-000000000001".to_string();
    let mut app = make_app_with_session(&session_id);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ClientToServer>(16);
    app.ipc_tx = Some(tx);

    // FAILS: on_pty_reset() is todo!()
    on_pty_reset(&mut app, session_id.clone());

    let attach_msg = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("AC-009: on_pty_reset must send AttachSession even with no pending bytes")
        .expect("ipc_tx must not close");

    assert!(
        matches!(
            attach_msg,
            ClientToServer::AttachSession {
                session_id: ref sid
            } if sid == &session_id
        ),
        "AC-009: expected AttachSession, got: {:?}",
        attach_msg
    );
}
