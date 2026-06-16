---
document_type: behavioral-contract
level: L3
version: "1.0.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:03:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "0fcd5bd"
traces_to: prd.md
origin: greenfield
subsystem: SS-01
capability: CAP-001
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.01.004: Graceful Shutdown (10-Second Drain)

## Description

When the monocle daemon receives a shutdown signal (SIGTERM, SIGINT, or authenticated
`POST /shutdown`), it transitions immediately to `ShuttingDown` AppMode, rejects new hook
POSTs with HTTP 503, and drains in-flight requests for up to 10 seconds before exiting.
The exit code distinguishes clean drain (0), SIGINT hard-kill (130), SIGTERM hard-kill (143),
admin forced-stop (2), and startup failure (1) per POSIX 128+N conventions.

## Preconditions

1. The monocle daemon is running and may have in-flight hook POST requests.
2. A shutdown signal arrives: SIGTERM, SIGINT, or an authenticated `POST /shutdown`.

## Postconditions

1. AppMode transitions to `ShuttingDown` immediately.
2. All new hook POST requests to `/hooks/*` receive HTTP 503 with header `Retry-After: 10` and body `{"error":"daemon_shutting_down"}`.
3. `/healthz` returns HTTP 503 with body `{"status":"shutting_down"}` during drain.
4. `/status` continues to serve (read-only) during drain for monitoring purposes.
5. The daemon waits up to 10 seconds for in-flight hook POSTs to complete (`tokio::time::timeout(Duration::from_secs(10), drain_inflight())`).
6. If `--persistent-events` flag is set, the JSONL ring buffer is flushed to `<runtime_dir>/monocle-events.jsonl` during drain.
7. After drain or on second signal or second admin `/shutdown`: lock file removed, UDS socket closed, daemon exits.
8. The exit code written to the OS process table on daemon termination MUST match the trigger (POSIX 128+N convention for signal-induced exits):
   - `0`: graceful drain succeeded; all in-flight requests completed within the 10-second window; ring buffer flushed if applicable.
   - `130`: hard-killed by SIGINT (signal 2) during drain — POSIX convention 128+2. Typical cause: user pressed Ctrl-C a second time while draining.
   - `143`: hard-killed by SIGTERM (signal 15) during drain — POSIX convention 128+15. Typical cause: systemd/k8s sent a second SIGTERM after the graceful-shutdown window.
   - `2`: hard-killed by a second authenticated `POST /shutdown` during drain (admin forced-stop). This is a monocle-specific programmatic code, chosen outside the POSIX 128+N space (which starts at 129) and distinct from startup-failure exit 1.
   - `1`: daemon failed to start (startup failure — e.g., `DaemonStartError::RuntimeDirUnresolvable`, port bind failure, existing live lock file).

## Invariants

1. The 10-second drain window is a hard timeout. A second SIGTERM during drain triggers immediate hard shutdown without waiting for in-flight requests.
2. Signal handling uses `tokio::signal::unix::signal(SignalKind::terminate())` for SIGTERM and `tokio::signal::ctrl_c()` for SIGINT. Both are awaited in a `tokio::select!` loop alongside the oneshot shutdown receiver. The signal type that triggered hard shutdown is recorded for exit-code selection.
3. The `POST /shutdown` endpoint requires authentication via the dual-accept protocol per ADR-0005 v1.0.2: either `X-Monocle-Authorization: monocle-v1:<64-hex>` (canonical) or `X-Claude-Code-Ide-Authorization: <64-hex>` (compatibility alias) — unauthenticated shutdown requests (neither header present) receive HTTP 401 `{"error":"missing_auth_token"}` per BC-2.01.009 PC-1; value-present failures receive HTTP 401 `{"error":"invalid_auth_token"}` per BC-2.01.009 PC-2/PC-3.
4. External monitoring systems (systemd `Restart=on-failure`, k8s `terminationGracePeriodSeconds`, CI status parsers) MUST use exit code 143 (not 130) to detect SIGTERM hard-kill during drain. Exit 130 encodes SIGINT (Ctrl-C second press), not SIGTERM.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-048 | Hook POST arrives mid-drain after the drain timeout has expired but before the connection is force-closed | Daemon rejects with HTTP 503 `{"error":"daemon_shutting_down"}` |
| EC-049 | Ring buffer flush fails during drain (e.g., filesystem full) | Daemon logs `WARN: ring buffer flush failed: <io-error>` (E-RING-001) and proceeds with shutdown; partial flush is acceptable — Phase 2 readers skip incomplete trailing lines |
| EC-050 | `POST /shutdown` with valid auth during a drain already in progress | Daemon acknowledges with HTTP 200; second shutdown call triggers immediate hard close with exit code 2 (admin forced-stop) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SIGTERM or `POST /shutdown` (authenticated) | AppMode → ShuttingDown; new hooks get HTTP 503 + `Retry-After: 10` | happy-path |
| POST /hooks/* during drain | HTTP 503 `{"error":"daemon_shutting_down"}`, `Retry-After: 10` | edge-case |
| All in-flight requests complete within 10s | Exit code 0 | happy-path |
| Second SIGINT (Ctrl-C) during drain | Exit code 130 (POSIX 128+2) | edge-case |
| Second SIGTERM during drain | Exit code 143 (POSIX 128+15) | edge-case |
| Second `POST /shutdown` during drain | Exit code 2 (monocle-specific admin forced-stop) | edge-case |
| `DaemonStartError::RuntimeDirUnresolvable` or port bind failure | Exit code 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | SIGTERM triggers AppMode → ShuttingDown; subsequent hook POSTs return HTTP 503 with `Retry-After: 10` | integration |
| VP-004 | Second SIGTERM during drain triggers exit code 143 | integration |
| VP-004 | Second SIGINT during drain triggers exit code 130 | integration |
| VP-004 | Second `POST /shutdown` during drain triggers exit code 2 | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the graceful shutdown protocol which is core daemon lifecycle management for the hook ingestion subsystem |
| L2 Domain Invariants | DI-001 (every hook event must be written to the JSONL ring before any acknowledgement is returned — the 10-second drain window ensures in-flight hook POSTs complete their ring writes before the daemon acknowledges shutdown; Postcondition 6 explicitly flushes the ring buffer during drain before exit) |
| Architecture Module | monocle-runtime (daemon binary, HTTP server) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.33 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain |
| Brief Section | §Scope (hook receiver hardening sub-bullet — graceful shutdown protocol on SIGTERM/SIGINT) |
| Test File | `monocle-runtime/tests/graceful_shutdown.rs`; `monocle-runtime/tests/daemon_lifecycle.rs` |
| Test Name | `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests`; `test_BC_DAEMON_004_exit_codes_posix_distinct` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-DAEMON-004 |

## Related BCs (Recommended)

- [BC-2.01.001] — composes with: `/healthz` returns HTTP 503 during shutdown drain as defined in BC-2.01.001 Postcondition 2
- [BC-2.01.002] — composes with: `/status` continues serving during drain per BC-2.01.004 Postcondition 4
- [BC-2.01.005] — depends on: lock file is removed during clean shutdown per BC-2.01.005 Postconditions (clean shutdown)
- [BC-2.01.006] — composes with: crash recovery checkpoint is written during drain before lock file removal

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#daemon-lifecycle-protocol` — shutdown signal handling, drain sequence, exit code table
- `architecture/SS-daemon-lifecycle.md#shutdown-signal-handling` — SIGTERM/SIGINT/POST /shutdown dispatch

## Story Anchor (Recommended)

S-TBD — Implement graceful shutdown drain with POSIX exit codes (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-004-graceful-shutdown.md` — VP-004 shutdown drain integration tests

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-001 ...`
  - DI-001 mapping: The 10-second drain window and Postcondition 6 (ring buffer flush) directly enforce DI-001's requirement that every hook event reaches the JSONL ring before acknowledgement. The drain waits for in-flight writes; the flush persists them before exit.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T11:30:00Z (v1.0).

## §Trace v1.0.2

**F-R107-2 CRITICAL + GAP-R46-5 MEDIUM** (2026-05-17T23:30:00Z):

**F-R107-2 — Architecture Source pin refresh v1.0.25 → v1.0.30:**
- SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.25 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain`
- SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.30 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain`
- Canonical version per architect 5E commit 03a4c57 post-R106 closure.

**GAP-R46-5 — INV-3 dual-accept fix for /shutdown (ADR-0005 alignment):**
- Defect: INV-3 specified `/shutdown` requires only `X-Monocle-Authorization`. This contradicts ADR-0005 dual-accept protocol which applies to ALL authenticated endpoints including `/shutdown`, and is inconsistent with BC-2.01.009 PC-1/PC-2/PC-3 which govern the auth middleware applied to `/shutdown` on the authenticated router.
- SE-17f INV-3 BEFORE: `The POST /shutdown endpoint requires X-Monocle-Authorization authentication — unauthenticated shutdown requests receive HTTP 401.`
- SE-17f INV-3 AFTER: `The POST /shutdown endpoint requires authentication via the dual-accept protocol per ADR-0005 v1.0.2: either X-Monocle-Authorization: monocle-v1:<64-hex> (canonical) or X-Claude-Code-Ide-Authorization: <64-hex> (compatibility alias) — unauthenticated shutdown requests (neither header present) receive HTTP 401 {"error":"missing_auth_token"} per BC-2.01.009 PC-1; value-present failures receive HTTP 401 {"error":"invalid_auth_token"} per BC-2.01.009 PC-2/PC-3.`
- Rationale: ADR-0005 §Decision applies the dual-accept middleware to the entire authenticated router (hook endpoints + `/status` + `/shutdown`). The single-header-only INV-3 was a propagation gap — the BC-2.01.009 update (T-128n Round 4) that added dual-accept semantics was not mirrored into BC-2.01.004's description of `/shutdown`'s auth requirement.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. INV-3 and Architecture Source are the only normative changes in this version.
- SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T18:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.0.30 → v1.0.32; F-R109-14 MED — §Trace reordered ascending** (2026-05-18T05:03:00Z):
- F-R109-4: Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32 (Round 8A). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.30 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain`
- F-R109-14: §Trace blocks were descending (v1.0.2, v1.0.1). Reordered to ascending (v1.0.1, v1.0.2, v1.0.3). Content of each section preserved verbatim; only insertion order corrected.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:03:00Z > prior 2026-05-17T23:30:00Z (v1.0.2). ARITHMETICALLY TRUE: 2026-05-18T05:03:00Z > 2026-05-17T23:30:00Z PASS.

## §Trace v1.0.4

**GAP-PHASE2-R06-1 closure — Architecture Source pin SS-daemon-lifecycle v1.0.32 → v1.0.33** (2026-05-19T12:03:00Z):
- GAP-PHASE2-R06-1: architect commit `2d43127` bumped SS-daemon-lifecycle.md v1.0.32 → v1.0.33 (Ring Buffer Rotation Policy added). BC ledger Architecture Source cell was not cascaded in that commit.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.33 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:03:00Z > prior 2026-05-18T05:03:00Z (v1.0.3). ARITHMETICALLY TRUE: PASS.
