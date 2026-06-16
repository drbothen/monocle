---
document_type: behavioral-contract
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "220cb6e"
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

# Behavioral Contract BC-2.01.001: Healthz Endpoint (Unauthenticated Liveness Probe)

## Description

The monocle daemon exposes a `GET /healthz` endpoint on `127.0.0.1:<port>` that returns
the daemon's liveness state without requiring authentication. This endpoint allows the TUI
client and external health monitors to probe whether the daemon is alive or shutting down.
It is explicitly placed on the unauthenticated router so that auth-token rotation during
crash recovery does not block liveness checks.

## Preconditions

1. The monocle daemon is running and bound on `127.0.0.1:<port>`.
2. A `GET /healthz` request arrives (no auth header required).

## Postconditions

1. When AppMode is normal (not `ShuttingDown`) and the hook-receiver task is alive: HTTP 200 with body `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}` where `uptime_sec` is integer seconds since daemon start and `version` is the monocle binary semver string matching regex `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$` (SemVer 2.0; no leading `v` prefix permitted).
2. When AppMode is `ShuttingDown` OR the hook-receiver task has exited abnormally: HTTP 503 with body `{"status":"shutting_down"}`.
3. `/healthz` is unauthenticated — no `X-Monocle-Authorization` header is required or checked.
4. `/healthz` has no request body and no `DefaultBodyLimit` applies (the limit is applied to the authenticated router only).

## Invariants

1. The endpoint must succeed even if the auth token has rotated during crash recovery. Unauthenticated access is warranted because `uptime_sec` and `version` are not secret, and a local adversary with `127.0.0.1` access already has OS-level process enumeration capability.
2. `/healthz` is registered on the unauthenticated router and MUST NOT be co-located on the authenticated router (which would inadvertently apply the auth middleware).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-040 | TUI client behavior when `/healthz` is unreachable AND the lock file exists with a live pid (`kill(pid, 0)` succeeds) | TUI concludes daemon is hung (accepting TCP, not responding) and initiates recovery flow with a 10-second countdown before auto-restarting |
| EC-041 | TUI client behavior when `/healthz` is unreachable AND the lock file exists with a dead pid | TUI treats the lock file as stale and initiates normal auto-start |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `GET /healthz` (no auth header) | HTTP 200 `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}` | happy-path |
| `GET /healthz` during graceful shutdown | HTTP 503 `{"status":"shutting_down"}` | edge-case |
| `GET /healthz` with no `X-Monocle-Authorization` header | HTTP 200 (not HTTP 401) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-001 | `/healthz` returns HTTP 200 with `"status":"alive"` and numeric `uptime_sec` when daemon is running normally | integration |
| VP-001 | `/healthz` returns HTTP 503 with `"status":"shutting_down"` when AppMode is `ShuttingDown` | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the liveness probe that is a prerequisite for hook ingestion and daemon lifecycle management |
| L2 Domain Invariants | DI-002 (lock file must be present before hook endpoints accept connections — healthz is on the unauthenticated router explicitly to remain reachable even when the lock file and auth token are being rotated, making it the observable complement of the DI-002 lock-file lifecycle) |
| Architecture Module | monocle-runtime (daemon binary, HTTP server) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.33 §Health and Status Endpoints §GET /healthz |
| Brief Section | §Scope (hook receiver hardening sub-bullet — `/healthz` liveness endpoint) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-DAEMON-001 |

## Related BCs (Recommended)

- [BC-2.01.002] — composes with: `/status` endpoint is the authenticated counterpart to the unauthenticated `/healthz`
- [BC-2.01.004] — depends on: graceful shutdown changes AppMode to `ShuttingDown`, triggering the HTTP 503 response defined here

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#health-and-status-endpoints` — HTTP server routing, unauthenticated router, `/healthz` endpoint spec

## Story Anchor (Recommended)

S-TBD — Implement daemon HTTP server with healthz endpoint (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-001-healthz-endpoint.md` — VP-001 healthz endpoint integration test

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-002 (lock file must be present before hook endpoints accept connections — healthz is on the unauthenticated router explicitly to remain reachable even when the lock file and auth token are being rotated, making it the observable complement of the DI-002 lock-file lifecycle)`
  - Mapping rationale: DI-002 requires the lock file present before hook endpoints accept connections. The healthz endpoint is structurally placed on the UNAUTHENTICATED router specifically so it remains reachable during lock-file lifecycle events (creation, crash recovery, token rotation). This BC is the observable complement of DI-002 enforcement.
- F-R105-9 (SE-17c-d body-scope grep): NO stale BC IDs found (`grep BC-DAEMON\|BC-RING\|BC-AUTH` → 0 matches in body prose). NO stale VP IDs found (`grep VP-DAEMON\|VP-AUTH` → 0 matches). F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R107-2 CRITICAL — Architecture Source pin refresh v1.0.25 → v1.0.30** (2026-05-17T23:30:00Z):
- F-R107-2: Sibling-layer cascade miss from Round 5D (VPs swept but BCs not). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.25 §Health and Status Endpoints §GET /healthz`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.30 §Health and Status Endpoints §GET /healthz`
  - Canonical version per architect 5E commit 03a4c57 post-R106 closure.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T18:00:00Z (v1.0.2).

## §Trace v1.0.4

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.0.30 → v1.0.32; F-R109-14 MED — §Trace reordered ascending** (2026-05-18T05:00:00Z):
- F-R109-4: Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32 (Round 8A). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.30 §Health and Status Endpoints §GET /healthz`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.32 §Health and Status Endpoints §GET /healthz`
- F-R109-14: §Trace blocks were descending (v1.0.3, v1.0.2). Reordered to ascending (v1.0.2, v1.0.3, v1.0.4). Content of each section preserved verbatim; only insertion order corrected.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:00:00Z > prior 2026-05-17T23:30:00Z (v1.0.3). ARITHMETICALLY TRUE: 2026-05-18T05:00:00Z > 2026-05-17T23:30:00Z PASS.

## §Trace v1.0.5

**GAP-PHASE2-R06-1 closure — Architecture Source pin SS-daemon-lifecycle v1.0.32 → v1.0.33** (2026-05-19T12:00:00Z):
- GAP-PHASE2-R06-1: architect commit `2d43127` bumped SS-daemon-lifecycle.md v1.0.32 → v1.0.33 (Ring Buffer Rotation Policy added). BC ledger Architecture Source cell was not cascaded in that commit.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.32 §Health and Status Endpoints §GET /healthz`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.33 §Health and Status Endpoints §GET /healthz`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:00:00Z > prior 2026-05-18T05:00:00Z (v1.0.4). ARITHMETICALLY TRUE: PASS.
