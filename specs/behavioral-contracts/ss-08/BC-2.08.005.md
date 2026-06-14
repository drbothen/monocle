---
document_type: behavioral-contract
level: L3
version: "1.0.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md]
input-hash: "81ce91a"
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
   Cancellation of the GC task (e.g., if `rename_session()` revives a Terminated session —
   which is not allowed) must not be possible after `Terminated` is entered.

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
| Architecture Source | SS-session-manager.md v2.2.0 §Session GC policy |
| Test Name | test_BC_2_08_005_terminated_session_gc_after_10s |

## Related BCs

- [BC-2.08.003] — depends on: session kill transitions to Terminating → Terminated, which triggers GC
- [BC-2.08.004] — composes with: orphaned sidecars are GC'd immediately during re-discovery (instant GC path)

## Architecture Anchors

- `architecture/SS-session-manager.md#session-gc-policy` — 10s grace period, sidecar deletion, socket cleanup

## Story Anchor

S-TBD — Implement SessionManager GC task (filled by story-writer)

## VP Anchors

VP-TBD — GC timing tests using tokio::time::pause (filled after VP creation)

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
