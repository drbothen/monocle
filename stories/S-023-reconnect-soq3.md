---
document_type: story
level: L4
story_id: S-023
epic_id: EPIC-05
version: "1.0"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 5
wave: 6
tdd_mode: strict
priority: P0
depends_on: [S-022, S-019]
blocks: [S-026]
target_module: monocle-ipc
subsystems: [SS-05]
behavioral_contracts: [BC-2.05.006, BC-2.05.007]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.006.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.007.md, version: "1.0.3"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.05.006 (TUI reconnect with exponential backoff + lock-file re-read), BC-2.05.007 (SOQ-3 overlay-clear on disconnect)"
---

# S-023: TUI Reconnect Loop with Exponential Backoff and SOQ-3 Overlay Clear

## Narrative

As a TUI user, I want the monocle TUI to automatically reconnect to the daemon after a
connection loss using exponential backoff, clearing any stale permission prompts before the
first reconnect attempt (SOQ-3), and re-reading the lock file on each retry to discover a
restarted daemon, so that temporary daemon crashes are invisible to the user and ghost
approval races are structurally impossible.

## Acceptance Criteria

### AC-001 (traces to BC-2.05.007 postcondition PC-1 — TransportEvent::Disconnected emission)
When `read_framed` in the TUI's receive loop returns a connection-loss error (`UnexpectedEof`,
`BrokenPipe`, or `ConnectionReset`), `UdsTransport` emits `TransportEvent::Disconnected`
immediately upon detecting the error, before the error propagates to the caller or any
reconnect attempt begins.

### AC-002 (traces to BC-2.05.007 postcondition PC-2 — VecDeque clear)
The TUI event loop receives `TransportEvent::Disconnected` and calls the SOQ-3 handler, which
clears all entries from the `VecDeque<PromptModal>` overlay stack. The VecDeque is empty after
the handler returns.

### AC-003 (traces to BC-2.05.007 postcondition PC-3 — synchronous clear before reconnect)
The clear operation is synchronous: it completes before the reconnect loop begins. There is
no window between the disconnect detection and the overlay clear where a stale prompt could
be interacted with.

### AC-004 (traces to BC-2.05.007 postcondition PC-4 — AppMode transition to Dashboard)
After the overlay is cleared, if the TUI was in `AppMode::Overlay`, it transitions to
`AppMode::Dashboard` as part of the SOQ-3 handler. `AppMode::Overlay` with an empty
`VecDeque` is an invalid state and must not persist.

### AC-005 (traces to BC-2.05.007 postcondition PC-5 — cleared prompts discarded permanently)
Cleared prompts are NOT preserved in any intermediate buffer. They are discarded permanently.
On reconnect, only prompts still pending in the daemon's registry (those that have not yet
timed out) are re-delivered via `InitialState.overlay_stack`.

### AC-006 (traces to BC-2.05.007 postcondition PC-6 — no SOQ-3 on TUI-initiated disconnect)
`TransportEvent::Disconnected` is emitted for ALL unexpected disconnects: daemon crash, daemon
graceful shutdown while TUI is connected, and UDS stream EOF/error. It is NOT emitted when
the TUI itself initiates a graceful disconnect (TUI process exits normally via user quit).

### AC-007 (traces to BC-2.05.006 postcondition PC-2 — reconnecting status bar)
Immediately after SOQ-3 fires, the TUI renders `[daemon: reconnecting...]` in the status bar,
replacing any prior status indicator.

### AC-008 (traces to BC-2.05.006 postcondition PC-3 — lock file re-read on each retry)
The TUI re-reads `<runtime_dir>/monocle.lock` after each failed reconnect attempt. If the
lock file has changed (new `pid`, new `port`, new `authToken`), the TUI uses the updated
values for subsequent connection attempts. This handles daemon restart where the new daemon
creates a new lock file at the same path with different fields.

### AC-009 (traces to BC-2.05.006 postcondition PC-4 — exponential backoff)
The TUI attempts reconnection with the following backoff schedule:
- Attempt 1: wait 250ms before retry.
- Attempt 2: wait 500ms before retry.
- Attempt 3: wait 1000ms before retry.
- Attempt 4+: wait 2000ms before retry (cap at 2 seconds; no further increase).

### AC-010 (traces to BC-2.05.006 postcondition PC-5 — 5-second window and offline mode)
The total reconnect window is 5 seconds from the first disconnect detection. If no connection
succeeds within 5 seconds:
- Status bar renders `[daemon: offline]`.
- TUI enters passive observe-only mode; no IPC push messages received.
- TUI polls the lock file every 5 seconds. When a new lock file is detected (new daemon
  started), the TUI re-enters the reconnect loop from the beginning (SOQ-3 fires again if
  overlay is non-empty; backoff resets to 250ms).

### AC-011 (traces to BC-2.05.006 postcondition PC-6 — InitialState rebuild on reconnect)
On successful reconnect, the daemon sends a fresh `ServerToClient::InitialState` push
(per BC-2.05.002). The TUI discards all prior local state and rebuilds its complete state
from this message.

### AC-012 (traces to BC-2.05.006 postcondition PC-7 — AppMode resets to Dashboard on reconnect)
If the TUI was in `AppMode::Overlay` when the disconnect occurred (cleared by SOQ-3), it
remains in `AppMode::Dashboard` through the reconnect. After `InitialState` receipt, if
`overlay_stack` is non-empty, the TUI transitions back to `AppMode::Overlay` to display
re-delivered prompts. It does NOT assume Overlay mode on reconnect.

### AC-013 (traces to BC-2.05.006 postcondition PC-8 — status bar reverts after reconnect)
After successful reconnect and `InitialState` receipt, the status bar reverts to normal
(no `[daemon: reconnecting...]` or `[daemon: offline]` indicator).

### AC-014 (traces to BC-2.05.007 invariant 1 — SOQ-3 ordering is unconditional)
`TransportEvent::Disconnected` is always the first event emitted on connection loss. The
reconnect loop NEVER starts before the Disconnected event is handled. This ordering is
enforced at the `UdsTransport` level — not in the TUI event handler — so it cannot be
bypassed by TUI code changes.

### AC-015 (traces to BC-2.05.007 invariant 3 — idempotent clear)
If the `VecDeque<PromptModal>` is already empty when `TransportEvent::Disconnected` is
received (no prompts were queued), the SOQ-3 handler runs without error. An empty-clear
is a no-op. AppMode remains Dashboard.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,400 |
| BC-2.05.006.md | ~900 |
| BC-2.05.007.md | ~900 |
| SS-ipc.md §Reconnection Behavior + §SOQ-3 | ~3,000 |
| S-022 (InitialState push — reconnect re-delivers state) | ~400 |
| S-019 (auto-start — lock file re-read pattern) | ~300 |
| Test files | ~800 |
| **Total estimate** | **~7,700** |

## Tasks

- [ ] Add `TransportEvent` enum to `monocle-ipc/src/transport.rs`:
  ```rust
  pub enum TransportEvent {
      Disconnected,
  }
  ```
- [ ] Modify `UdsTransport` read path in `monocle-ipc/src/uds.rs`:
  - When `read_framed` returns `UnexpectedEof`, `BrokenPipe`, or `ConnectionReset`:
    1. Emit `TransportEvent::Disconnected` via the event channel BEFORE returning the error
    2. Return the error to the caller
  - When TUI initiates graceful disconnect (normal process exit): do NOT emit `TransportEvent::Disconnected`
- [ ] Implement SOQ-3 handler in TUI event loop (`monocle-tui/src/app.rs` or `monocle/src/tui_app.rs`):
  - On `TransportEvent::Disconnected` received: call `overlay_stack.clear()`; if `AppMode == Overlay` → set `AppMode = Dashboard`
  - This must be synchronous within the event loop iteration — completed before reconnect loop is scheduled
- [ ] Implement reconnect loop in `monocle-ipc/src/reconnect.rs`:
  - `reconnect(runtime_dir, backoff_state) -> Result<UdsTransport, IpcError>`
  - Backoff schedule: 250ms → 500ms → 1000ms → 2000ms cap
  - After each failed attempt: re-read `<runtime_dir>/monocle.lock`; update `auth_token`, `pid` from new lock file if changed
  - Total window: 5 seconds; on exhaustion → return `Err(IpcError::ReconnectTimeout)`
  - On success: return connected `UdsTransport` for caller to trigger `InitialState` rebuild
- [ ] Implement offline mode in TUI event loop:
  - On `IpcError::ReconnectTimeout`: set status bar to `[daemon: offline]`; enter passive mode
  - Passive mode: poll `<runtime_dir>/monocle.lock` every 5 seconds; on new lock file detected → re-enter reconnect loop (backoff resets to 250ms; SOQ-3 fires if overlay non-empty)
- [ ] Status bar rendering:
  - `[daemon: reconnecting...]` — shown immediately after SOQ-3 fires, before first reconnect attempt
  - `[daemon: offline]` — shown after 5-second window exhausted
  - Normal (no indicator) — shown after successful reconnect + `InitialState` receipt
- [ ] State rebuild on reconnect:
  - Discard prior local sessions, ring_tail, overlay_stack, drop_counter
  - Apply `InitialState` fields as fresh authoritative state
  - Transition `AppMode` to Overlay if `InitialState.overlay_stack` is non-empty; remain Dashboard otherwise
- [ ] Unit tests `monocle-ipc/tests/soq3_overlay_clear.rs`:
  - `TransportEvent::Disconnected` emitted before reconnect loop starts
  - VecDeque<PromptModal> is empty after SOQ-3 handler runs (populated case)
  - SOQ-3 fires on EOF, BrokenPipe, and ConnectionReset (all three error variants)
  - SOQ-3 does NOT fire on graceful TUI-initiated disconnect
  - AppMode transitions to Dashboard after overlay cleared (was Overlay)
  - Idempotent: empty VecDeque, SOQ-3 fires — no error, AppMode remains Dashboard
- [ ] Unit tests `monocle-ipc/tests/reconnect.rs`:
  - Exponential backoff: 250ms → 500ms → 1000ms → 2000ms cap (mock clock)
  - Lock file re-read after each failed attempt; new daemon pid/port discovered
  - 5-second window exhaustion: transitions to "daemon offline" mode
  - Fresh InitialState on reconnect causes full TUI state rebuild
  - AppMode::Overlay resets to Dashboard after SOQ-3; transitions back to Overlay if InitialState.overlay_stack non-empty after reconnect
  - Daemon crash loop (4 restarts in 30 seconds): TUI reconnects each time; no cumulative state leakage

## Previous Story Intelligence

S-022: The `UdsTransport` connect path, `InitialState` push, `PermissionPromptQueued`, and
`PermissionDecision` routing are established. This story adds the disconnect detection signal
(`TransportEvent::Disconnected`) to `UdsTransport` and the reconnect loop with backoff. The
SOQ-3 handler clears the `VecDeque<PromptModal>` that S-022 populates with `PermissionPromptQueued`
payloads. The fresh `InitialState` on reconnect (also from S-022's server-side code) rebuilds
TUI state after reconnect.

S-019: The auto-start lock file polling pattern (`100ms poll, 5-second timeout`) is established.
The reconnect lock-file re-read in this story reuses the same `resolve_runtime_dir()` +
lock-file read pattern from S-019, but the reconnect-mode poll interval is 5 seconds (per
BC-2.05.006 PC-5), not 100ms.

## Architecture Compliance Rules

From `architecture/SS-ipc.md v1.4.0 §SOQ-3 Overlay Clear on Disconnect`:
- `TransportEvent::Disconnected` MUST be emitted at the `UdsTransport` level — not in TUI event handler
- The SOQ-3 ordering (disconnect → clear → reconnect) is enforced at the transport layer, not the application layer
- SOQ-3 must NOT fire on TUI-initiated graceful disconnect (only on unexpected connection loss)

From `architecture/SS-ipc.md v1.4.0 §Reconnection Behavior`:
- Backoff cap: 2000ms (2 seconds) — no further increase after Attempt 4+
- 5-second total window — not configurable in Phase 1
- Lock file re-read after EACH failed attempt (not just on window expiry)
- Offline polling interval: 5 seconds (not 100ms; different from auto-start poll)
- `AppMode::Overlay` with empty VecDeque is invalid — must not persist after SOQ-3

**Forbidden Dependencies:**
- The reconnect loop MUST NOT skip the SOQ-3 clear on reconnect attempts after the first (each reconnect attempt re-starts from a cleared state — SOQ-3 fires once on connection loss, not on each retry)
- The offline polling MUST NOT use the auto-start 100ms interval — the offline poll is 5 seconds (BC-2.05.006 PC-5)
- `TransportEvent::Disconnected` MUST be emitted synchronously in the `UdsTransport` receive path — not via a separate task or delayed channel

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| tokio | =1.52.0 | `time::sleep` for backoff intervals; async select on reconnect timeout; 5-second offline poll |
| interprocess | 2.4 | `UnixStream::connect` for reconnect attempts |
| serde_json | =1.0.149 | Lock file JSON parsing on re-read after each retry |
| tracing | 0.1 | DEBUG on reconnect attempt; INFO on reconnect success; WARN on 5-second timeout and offline mode entry |

## File Structure Requirements

Files to create:
- `monocle-ipc/src/reconnect.rs` — `reconnect()` with exponential backoff; lock-file re-read; 5-second window; offline fallback
- `monocle-ipc/src/events.rs` — `TransportEvent` enum definition
- `monocle-ipc/tests/soq3_overlay_clear.rs` — SOQ-3 invariant tests
- `monocle-ipc/tests/reconnect.rs` — reconnect backoff, lock-file re-read, offline mode, state rebuild tests

Files to modify:
- `monocle-ipc/src/uds.rs` — emit `TransportEvent::Disconnected` on connection-loss errors before returning error; add graceful-disconnect path that does NOT emit the event
- `monocle-ipc/src/transport.rs` — add `TransportEvent` enum; add event channel to `UdsTransport`
- `monocle-ipc/src/lib.rs` — re-export `reconnect`, `events` modules
- TUI event loop (`monocle-tui/src/app.rs` or equivalent) — SOQ-3 handler: `overlay_stack.clear()` on `TransportEvent::Disconnected`; AppMode Dashboard transition; status bar updates; offline mode entry; offline poll loop
