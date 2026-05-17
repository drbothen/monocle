---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T11:30:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "03a845a"
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
| L2 Domain Invariants | N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source |
| Architecture Module | monocle-runtime (daemon binary, HTTP server) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.25 §Health and Status Endpoints §GET /healthz |
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

- `verification-properties/vp-001-healthz-liveness.md` — VP-001 liveness probe integration test
