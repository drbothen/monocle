---
document_type: story
level: L4
story_id: S-037
epic_id: EPIC-08
version: "1.0.3"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 3
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-033, S-034]
blocks: []
target_module: monocle-runtime
subsystems: [SS-08]
behavioral_contracts: [BC-2.08.005]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.005.md, version: "1.0.6"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.11.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "ab7950d"
traces_to: "Implements BC-2.08.005 (GC task: Terminated sessions removed from registry after 10s grace period; sidecar + socket deleted; SessionListUpdate published)"
# BC status: BC-2.08.005 v1.0.8 — non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-037: SessionManager GC Task — Terminated Sessions Removed After 10s Grace Period

## Narrative

As the monocle daemon, I want terminated sessions to remain in the registry for a 10-second
grace period after transition to `Terminated` — so the TUI can display a "session exited"
indicator — and then a per-session GC tokio task to remove the `SessionEntry`, delete the
`session-state.json` sidecar, and publish a `SessionListUpdate` — so that the TUI session
list stays accurate and `runtime_dir` does not accumulate stale sidecar files indefinitely.

## Acceptance Criteria

### AC-001 (traces to BC-2.08.005 postcondition 1 — SessionEntry removed at 10s ± 1s)

At 10 seconds (±1 second jitter allowance) after a `SessionEntry.state` first transitions to
`Terminated`, the `SessionEntry` is removed from `SessionManager.sessions`.

### AC-002 (traces to BC-2.08.005 postcondition 2 — session-state.json deleted)

`session-state.json` at `<runtime_dir>/session-<session_id>.json` is deleted when the GC
task fires. Deletion uses `std::fs::remove_file` (tolerates ENOENT; logs trace if absent).
GC MUST NOT fail on a missing sidecar.

### AC-003 (traces to BC-2.08.005 postcondition 3 — per-session UDS socket file deleted)

The per-session UDS socket file at `<runtime_dir>/session-<session_id>.sock` is deleted
if it still exists (best-effort; session-host should have removed it on `Goodbye`, but GC
cleans up any remaining socket file). Missing socket file is not an error.

### AC-004 (traces to BC-2.08.005 postcondition 4 — SessionListUpdate published within one broker tick)

`ServerToClient::SessionListUpdate` IPC message is published to the broker within one broker
tick of registry removal. Connected TUI clients update their session list.

### AC-005 (traces to BC-2.08.005 postcondition 5 — TUI vt100::Parser cleanup)

If the session's `vt100::Parser` is held in the TUI's `App::pty_parsers` map, the TUI removes
it on receipt of the `SessionListUpdate` (or `SessionStateChanged::Terminated`, whichever
arrives first). This AC is validated at the TUI side in SS-09 stories; daemon responsibility
is solely to emit `SessionListUpdate` after GC.

### AC-006 (traces to BC-2.08.005 invariant 1 — GC timer starts at FIRST Terminated transition; re-transition does NOT reset)

The 10-second grace period begins when `SessionEntry.state` FIRST transitions to `Terminated`.
A second `StateChanged::Terminated` message (re-transition — should not occur in normal
operation) does NOT reset the timer. The GC task is not cancellable after `Terminated` is entered.

### AC-007 (traces to BC-2.08.005 invariant 2 — orphaned sidecars GC'd immediately during re-discovery; no grace period)

Orphaned sidecars found during `rediscover_sessions()` with a dead PID are GC'd IMMEDIATELY
(no 10-second grace period). The 10-second grace period applies ONLY to sessions that were
alive in the registry and then transitioned to `Terminated` during the daemon's current
lifecycle. (This AC validates the boundary between S-037 and S-036; the immediate-GC path
in re-discovery is implemented in S-036.)

### AC-008 (traces to BC-2.08.005 invariant 3 — std::fs::remove_file; tolerates ENOENT)

`session-state.json` deletion MUST use `std::fs::remove_file` (NOT `std::fs::write` or
`tempfile::persist` — these are for writing, not deletion). ENOENT is not an error; log at
`tracing::trace!` level only.

### AC-009 (traces to BC-2.08.005 invariant 4 — rename on Terminated returns InvalidSessionName{reason: "session terminated"})

`rename_session()` on a Terminated-in-grace session MUST return
`Err(SessionError::InvalidSessionName { reason: "session terminated" })` → wire code
`"rename_failed"`. The GC task is not cancellable; the rename error is the observable
safety mechanism that prevents display_name mutation on a dead session entry.
Test: spawn → kill → wait for Terminated (before 10s) → call `rename_session()` → assert `Err(InvalidSessionName { reason: "session terminated" })`.

### AC-010 (traces to BC-2.08.005 edge case EC-173 — daemon exits before GC fires)

If the daemon exits before the 10-second GC fires:
- The sidecar still exists in `runtime_dir`.
- On the NEXT daemon startup, `rediscover_sessions()` finds the sidecar.
- The PID is dead (harness child exited).
- Re-discovery GC's the sidecar immediately (no grace period for orphaned sidecars).

### AC-011 (traces to BC-2.08.005 edge case EC-174 — GC fires after session-host already deleted sidecar)

If the GC task fires but the sidecar has already been deleted (session-host cleaned it up on
`Goodbye`): `remove_file` returns ENOENT; logs at trace level; no error; GC continues;
registry entry is removed; `SessionListUpdate` published.

### AC-012 (traces to BC-2.08.005 edge case EC-175 — two sessions terminate within 1 second)

Two independent GC tasks, each with their own 10-second timer, fire independently.
No interference between concurrent GC tasks.

## Tasks

- [ ] Implement GC task as a `tokio::spawn` started when a `SessionEntry.state` first transitions to `Terminated` (in `kill_session()`, in the watchdog task, in the post-spawn monitor on startup failure, and in re-discovery for alive-then-dead paths):
  - `tokio::time::sleep(Duration::from_secs(10)).await`.
  - Under `SessionManager` mutex: remove `SessionEntry` from `sessions`; publish `SessionListUpdate` to broker.
  - After mutex release: `std::fs::remove_file(sidecar_path)` (ENOENT ok); `std::fs::remove_file(socket_path)` (ENOENT ok); log trace.
- [ ] Implement `rename_session()` state guard: if `SessionEntry.state == Terminated`, return `Err(SessionError::InvalidSessionName { reason: "session terminated".to_string() })` immediately.
- [ ] Implement `rename_session()` for non-Terminated sessions (metadata update: `display_name` field; update sidecar; publish `SessionListUpdate` only — NOT `SessionStateChanged`; per BC-2.08.008 PC-4a rename rule).
- [ ] NOTE — IPC arm NOT in this story's scope: `ClientToServer::RenameSession` IPC handler arm is owned by S-047 (BC-2.05.010 / "Rename Ownership Disambiguation" section below). S-037 owns ONLY `rename_session()` on `SessionManager`. The test `test_BC_2_08_005_rename_on_running_succeeds` exercises the method directly (bypassing IPC dispatch); wire-level dispatch is tested in S-047.
- [ ] Write unit test `test_BC_2_08_005_terminated_session_gc_after_10s`: `tokio::time::pause()`; spawn session; inject Terminated; advance virtual time 10s; assert `SessionEntry` removed from registry; sidecar deleted; `SessionListUpdate` published. Wall clock not used.
- [ ] Write unit test `test_BC_2_08_005_gc_sidecar_enoent_no_error`: mock sidecar pre-deleted; GC fires; `remove_file` ENOENT; no error; session removed from registry.
- [ ] Write unit test `test_BC_2_08_005_rename_on_terminated_fails`: session in Terminated state; `rename_session()` → `Err(InvalidSessionName { reason: "session terminated" })`; wire code `"rename_failed"`.
- [ ] Write unit test `test_BC_2_08_005_two_sessions_terminate_independently`: two sessions; both transition to Terminated 1s apart; both GC'd after 10s each; no interference.
- [ ] Write unit test `test_BC_2_08_005_rename_on_running_succeeds`: `rename_session()` on Running session; `display_name` updated; sidecar updated; `SessionListUpdate` published; NO `SessionStateChanged`.

## Previous Story Intelligence

- **S-033** (spawn): `SessionManager`, `SessionEntry`, `SessionState`, and `Broker<Event>` publish pattern established. GC task is a `tokio::spawn` that mirrors the post-spawn monitor pattern (a background task operating on a specific session_id).
- **S-034** (kill): The `Terminating → Terminated` transition path (both from session-host confirmation and from the 12s watchdog) is where the GC timer must be started. This story adds the 10s GC task at those transition points.
- The GC task needs the session_id and the paths to sidecar/socket. These are available from `SessionEntry` fields. After the 10s sleep, the task must re-lock the mutex and verify the session is still in Terminated state (guard against race with a new spawn that reuses the same session_id — astronomically unlikely but defensive).
- `rename_session()` is a NEW public API method not yet implemented by prior stories. Define it in this story.

## Architecture Compliance Rules

- GC timer starts when `SessionEntry.state` FIRST transitions to `Terminated`. Do NOT reset on duplicate Terminated messages.
- `std::fs::remove_file` for sidecar and socket deletion. ENOENT is not an error.
- GC task MUST publish `SessionListUpdate` under the `SessionManager` mutex BEFORE releasing. This ensures TUI clients see an atomic list without the GC'd session.
- `rename_session()` MUST NOT emit `SessionStateChanged` — it is a metadata operation, not a state transition (BC-2.08.008 PC-4a). Only `SessionListUpdate` is emitted.
- `rename_session()` on Terminated-in-grace: `Err(InvalidSessionName { reason: "session terminated" })` → wire code `"rename_failed"`. No new SessionError variant; uses existing `InvalidSessionName` with a `reason` field (F-P52-001 constraint honored).
- The GC task's `SessionListUpdate` publish is the ONLY IPC message emitted by GC. No `SessionStateChanged` is emitted by the GC task itself (`SessionStateChanged{Terminated}` was already emitted at the point of Terminated transition in S-034).
- Forbidden dependency: `monocle-runtime` MUST NOT depend on `monocle-tui`.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tokio` | `=1.52` (exact) | `tokio::spawn`; `tokio::time::sleep`; `tokio::time::pause()` in tests | SS-deps-pin-manifest.md |
| `thiserror` | `"2"` | `SessionError::InvalidSessionName` (from S-033) | SS-deps-pin-manifest.md |
| `tempfile` | `"3"` | Atomic sidecar writes for `rename_session()` | SS-deps-pin-manifest.md |
| `serde_json` | `=1.0.149` (exact) | Sidecar update for `rename_session()` | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/session_manager/mod.rs` | Add GC task launch at Terminated transition points (kill_session confirmation path, watchdog timeout path, post-spawn monitor startup-failure path); add `rename_session()` implementation |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~2,500 |
| BC-2.08.005 | ~2,000 |
| SS-session-manager.md (GC policy; Terminated-in-grace action matrix; rename rule) | ~4,000 |
| Existing session_manager code from S-033/S-034 | ~5,000 |
| Test files | ~3,000 |
| **Total estimate** | **~16,500** |

Estimate is comfortably within the 30% context window bound. No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.08.005 | Session GC — Terminated Sessions Removed from Registry After 10s Grace Period | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| GC task (per-session tokio::spawn) | `monocle-runtime/src/session_manager/mod.rs` | Effectful (tokio::time::sleep; mutex; filesystem; broker publish) |
| `rename_session()` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (sidecar write; broker publish) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-173 | Daemon exits before 10s GC fires | Sidecar persists; next daemon startup re-discovery GC's immediately (no grace period for orphaned sidecars) |
| EC-174 | GC fires; sidecar already deleted by session-host | `remove_file` ENOENT; trace log; no error; session removed from registry |
| EC-175 | Two sessions terminate within 1s of each other | Two independent GC tasks; both fire at their own 10s mark; no interference |
| EC-176 | TUI client disconnects before GC fires | GC fires at 10s; `SessionListUpdate` published; no connected clients receive it; no error |

## Rename Ownership Disambiguation

S-037 owns the **`rename_session()` method** on `SessionManager`
(implementing BC-2.08.005/BC-2.08.008 PC-4a — metadata update, no state transition, `SessionListUpdate`
only). This is a new method introduced in S-037. **Do not confuse** with S-047's
`ClientToServer::RenameSession { session_id, new_name }` IPC arm (BC-2.05.010) — S-047 owns the IPC
wire variant and calls `session_manager.rename_session()`. The method lives in SS-08; the IPC arm
lives in SS-05. These are complementary, not duplicates.

## Subsystem Anchor Justifications

**SS-08 owns this story's scope** because `SessionManager` GC is defined in SS-session-manager.md §Session GC policy and the Terminated-in-grace action matrix is part of SS-session-manager.md §Terminated-in-grace defensive action×state matrix.

**Dependency Anchors:**
- STORY-037 depends on S-033 because `SessionManager`, `SessionEntry`, `SessionState::Terminated`, and the broker publish pattern must exist.
- STORY-037 depends on S-034 because the `Terminating → Terminated` transition path (where the GC timer starts) is established by kill_session() and its watchdog.
