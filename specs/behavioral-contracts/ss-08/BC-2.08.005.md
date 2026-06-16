---
document_type: behavioral-contract
level: L3
version: "1.0.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md]
input-hash: "a407816"
traces_to: prd.md
origin: greenfield
subsystem: SS-08
capability: CAP-008
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.08.005: Session GC — Terminated Sessions Removed from Registry After 10s Grace Period

## Description

`Terminated` sessions remain in the registry for a 10-second grace period to allow the TUI
to display a "session exited" indicator to the user. After the grace period, a tokio GC task
removes the `SessionEntry`, deletes `session-state.json`, and publishes a
`SessionListUpdate`. Orphaned sidecars discovered during re-discovery at startup are GC'd
immediately (no grace period for sidecars without a live process).

## Preconditions

1. A `SessionEntry` with `state: Terminated` exists in the registry.
2. The GC timer has been running for exactly 10 seconds since the transition to `Terminated`.

## Postconditions

1. At 10 seconds (+/- 1 second jitter) after transition to `Terminated`, the `SessionEntry`
   is removed from `SessionManager.sessions`.
2. `session-state.json` at `<runtime_dir>/session-<session_id>.json` is deleted.
3. The per-session UDS socket file at `<runtime_dir>/session-<session_id>.sock` is deleted
   if it still exists (best-effort; `session-host` should have removed it on `Goodbye`, but
   GC cleans up any remaining socket file).
4. A `ServerToClient::SessionListUpdate` IPC message is published to the broker within one
   broker tick of registry removal. Connected TUI clients update their session list.
5. If the session's `vt100::Parser` is held in the TUI's `App::pty_parsers` map, the TUI
   removes it on receipt of the `SessionListUpdate` (or `SessionStateChanged::Terminated`).

## Invariants

1. The 10-second grace period begins when `SessionEntry.state` first transitions to
   `Terminated`. Re-transition to `Terminated` (e.g., second `StateChanged::Terminated`
   message) does NOT reset the timer.
2. Orphaned sidecars (found during `rediscover_sessions()` with dead PID) are GC'd
   IMMEDIATELY during re-discovery — no 10-second grace period. The grace period applies
   only to sessions that were in the registry and transitioned to `Terminated` during the
   daemon's current lifecycle.
3. `session-state.json` deletion MUST use `std::fs::remove_file` (tolerates ENOENT if the
   session-host already deleted it). GC MUST NOT fail on a missing sidecar.
4. GC is managed by a single tokio task per session that fires via `tokio::time::sleep`.
   Cancellation of the GC task is not possible after `Terminated` is entered. Reviving a
   Terminated session via `rename_session()` is not allowed — attempting `rename_session()` on
   a Terminated-in-grace session returns `Err(SessionError::InvalidSessionName { reason:
   "session terminated" })`, which maps to wire code `"rename_failed"` per
   `session_error_to_code()` (SS-session-manager.md v2.6.1 §Terminated-in-grace defensive
   action×state matrix, F-P52-001). The GC task is not cancellable; the rename error is the
   observable safety mechanism that prevents display_name mutation on a corpse. The TUI-side
   guard (BC-2.06.025 Invariant 6) ensures `"rename_failed"` with this reason is a
   defensive/untrusted-client-only path — unreachable from the official TUI.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-173 | Session transitions to Terminated; daemon exits before 10s GC fires | Sidecar still exists in runtime_dir; on next daemon startup, re-discovery finds dead PID (PID is dead), GC's sidecar immediately |
| EC-174 | GC task fires but sidecar is already deleted (session-host cleaned it up) | GC calls `remove_file` → ENOENT; logs trace-level message; continues; no error |
| EC-175 | Two sessions both terminate within 1 second of each other | Two independent GC tasks, each with their own 10s timer; both fire independently; no interference |
| EC-176 | TUI client disconnects before GC fires | GC fires at 10s; `SessionListUpdate` published to broker; no connected clients receive it (no error); session is removed from registry |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Session transitions to Terminated; wait 10s | `SessionEntry` removed; sidecar deleted; `SessionListUpdate` published | happy-path |
| Session terminated; daemon restarts before 10s | Sidecar found at restart; dead PID; GC'd immediately during re-discovery | edge-case |
| Sidecar pre-deleted by session-host | GC fires; `remove_file` → ENOENT; no error; session removed from registry | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `SessionEntry` removed from registry ≥10s and ≤11s after Terminated transition | unit (tokio::time::pause) |
| VP-TBD | `session-state.json` deleted after GC fires | unit |
| VP-TBD | `SessionListUpdate` published after GC | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — GC is explicitly named in CAP-008; this BC defines the 10-second grace period, sidecar cleanup, and SessionListUpdate publication that constitute the GC policy |
| Architecture Module | monocle-runtime (SessionManager GC tokio task) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v2.6.1 §Session GC policy; SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state matrix (F-P52-001); SS-session-manager.md v2.6.1 §session_error_to_code() (InvalidSessionName → "rename_failed"); SS-ipc.md v1.24.0 §ServerToClient::Error taxonomy; SS-daemon-wiring-v2-delta.md v1.11.4 |
| Test Name | test_BC_2_08_005_terminated_session_gc_after_10s |

## Related BCs

- [BC-2.08.003] — depends on: session kill transitions to Terminating → Terminated, which triggers GC
- [BC-2.08.004] — composes with: orphaned sidecars are GC'd immediately during re-discovery (instant GC path)

## Architecture Anchors

- `architecture/SS-session-manager.md#session-gc-policy` — 10s grace period, sidecar deletion, socket cleanup

## Story Anchor

S-037 — Implement SessionManager GC task

## VP Anchors

VP-TBD — GC timing tests using tokio::time::pause (filled after VP creation)

## §Trace v1.0.4

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-037** (2026-06-15):
- Story Anchor filled from Phase-2 Burst A story decomposition. No behavioral content changed.

## §Trace v1.0.3

**P57-sweep — Architecture Source anchor misattribution correction: §session_error_to_code() moved from SS-ipc.md to SS-session-manager.md** (2026-06-14):
- **Architecture Source (line 109 — live pin only):** The `§session_error_to_code()` anchor was incorrectly attributed to `SS-ipc.md v1.24.0`. Per `SS-daemon-wiring-v2-delta.md:138`, `session_error_to_code()` is defined in `SS-session-manager.md` (monocle-runtime), NOT `SS-ipc.md`. SS-ipc.md has no `§session_error_to_code()` section.
- **Correction:** `SS-ipc.md v1.24.0 §session_error_to_code()` → `SS-session-manager.md v2.6.1 §session_error_to_code()`. SS-ipc.md v1.24.0 citation retained separately for `§ServerToClient::Error taxonomy` (which IS defined in SS-ipc.md). The functional semantic is unchanged — the error-to-code mapping still produces `"rename_failed"` from `InvalidSessionName { reason: "session terminated" }` — only the source document attribution is corrected.
- **Finding origin:** Phase-1d→human-gate fresh-context consistency audit (OBS-P57-001, upgraded to IMPORTANT severity by consistency lens). Confirmed by `SS-daemon-wiring-v2-delta.md:138`.
- No behavioral content changed.
- Patch bump: v1.0.2 → v1.0.3.

SE-16d monotonicity: v1.0.3 timestamp 2026-06-14 > v1.0.2 timestamp 2026-06-14. PASS.

## §Trace v1.0.2

**F-P52-001 — Invariant 4: explicit rename-on-Terminated error-return backing** (2026-06-14):

- **Gap closed (F-P52-001):** Invariant 4 previously stated GC task cancellation "must not be
  possible after Terminated is entered" and cited "rename_session() revives a Terminated session
  — which is not allowed" as the motivating case. However, it did not specify the observable
  safety mechanism: what actually happens at the daemon layer if `rename_session()` IS called on
  a Terminated-in-grace session. The architect closed this at SS-session-manager.md v2.6.1
  §Terminated-in-grace defensive action×state matrix (F-P52-001):
  - `rename_session()` on Terminated-in-grace → `Err(SessionError::InvalidSessionName { reason:
    "session terminated" })` → wire code `"rename_failed"` via exhaustive `session_error_to_code()`.
  - `detach_session()` on Terminated-in-grace → idempotent `Ok(())` (no-op; no host to detach).
  - `kill_session()` on Terminated-in-grace → idempotent `Ok(())` (already in BC-2.08.003 Inv 2).
  - `resize_session()` on Terminated-in-grace → WARN-drop (no-op).

- **Invariant 4 updated:** Added explicit `Err(SessionError::InvalidSessionName { reason:
  "session terminated" })` → `"rename_failed"` specification. Added citation of SS-session-manager.md
  v2.6.0 §Terminated-in-grace defensive action×state matrix (F-P52-001). Added TUI-side guard
  cross-reference to BC-2.06.025 Invariant 6 (the "rename_failed" reason code is a
  defensive/untrusted-client-only path — unreachable from the official TUI).

- **Architecture Source updated:** SS-session-manager.md pin v2.5.1 → v2.6.0; SS-ipc.md v1.24.0
  (session_error_to_code taxonomy); SS-daemon-wiring-v2-delta.md v1.11.4 added.

- **No new error variants or wire codes introduced.** `InvalidSessionName` already exists in the
  9-variant `SessionError` enum; `"rename_failed"` already exists in the 12-code wire taxonomy.
  The `{ reason: "session terminated" }` field is a new reason string within the existing
  `InvalidSessionName` variant — not a schema extension.

- **SessionError 9-variant / 12-code counts NOT regressed.** No new variants; no new wire codes.

- **Patch bump: v1.0.1 → v1.0.2** (trace-level errata adding explicit error-return specification
  to Invariant 4; behavioral obligation was already implicit in "which is not allowed"; this makes
  the observable mechanism explicit and testable).

SE-16d monotonicity: v1.0.2 timestamp 2026-06-14 > v1.0.1 timestamp 2026-06-13. PASS.

## §Trace v1.0.1

**S22-002 — Related-BCs: retired Killed state corrected to Terminating→Terminated kill path** (2026-06-13):
- S22-002 (Phase-1d Pass 22 SUGGESTION): Related-BCs entry for [BC-2.08.003] read
  "session kill transitions to Killed → Terminated". `SessionState::Killed` was REMOVED per
  SS-session-manager.md v1.3.0 §Session lifecycle state machine I4 audit (superseded by
  `Terminating`). The canonical kill path per BC-2.08.003 Invariant 1 is
  `Running | Detached | Launching → Terminating → Terminated`.
  Updated prose: "Killed → Terminated" → "Terminating → Terminated".
- Version bump: 1.0.0 → 1.0.1 (patch: prose correction only; no behavioral content change).

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.005 authored for SS-08 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): The 10s grace period allows the TUI to display a "session exited"
  indicator. The orphaned-sidecar immediate GC path (re-discovery) is specified here alongside
  the normal GC path to make the two-path GC model explicit and testable.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).

## §Trace v1.0.6

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.0.6 timestamp >= v1.0.5. PASS.
