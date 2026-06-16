---
document_type: behavioral-contract
level: L3
version: "1.0.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:01:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "17f5b4f"
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

# Behavioral Contract BC-2.01.002: Status Endpoint (Authenticated Daemon State)

## Description

The monocle daemon exposes a `GET /status` endpoint that returns comprehensive daemon
observability data including PID, uptime, ABI version, ring buffer fill levels, channel
saturation, and per-hook-type last-event timestamps. The endpoint is on the authenticated
router and requires a valid auth header per the dual-accept protocol (ADR-0005 v1.0.2):
either `X-Monocle-Authorization: monocle-v1:<token>` (canonical) or
`X-Claude-Code-Ide-Authorization: <64-hex>` (compatibility alias); full validation
semantics are defined in BC-2.01.009. The endpoint continues serving during graceful
shutdown drain to allow drain monitoring.

## Preconditions

1. The monocle daemon is running.
2. A `GET /status` request arrives with a valid auth header per dual-accept protocol (ADR-0005 v1.0.2): either `X-Monocle-Authorization: monocle-v1:<token>` (canonical) or `X-Claude-Code-Ide-Authorization: <64-hex>` (compatibility alias). Full dual-accept validation semantics are defined in BC-2.01.009.

## Postconditions

1. HTTP 200 with a JSON body containing all of the following fields:
   - `pid`: positive integer PID (≥ 1) of the daemon process per POSIX (PID 0 is reserved for the scheduler)
   - `uptime_sec`: integer seconds since daemon start
   - `version`: daemon binary semver string matching regex `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$` (SemVer 2.0; no leading `v` prefix permitted)
   - `abi_version`: integer `1` (`monocle_core::MONOCLE_ABI_VERSION` as compiled)
   - `lock_file`: absolute path string to `<runtime_dir>/monocle.lock`
   - `hook_endpoints`: JSON array of 5 hook path strings (`["/hooks/pre-tool-use", "/hooks/notification", "/hooks/stop", "/hooks/session-start", "/hooks/prompt-submit"]`)
   - `ring_buffer_fill_pct`: float 0.0–100.0 representing ring buffer fill percentage
   - `channel_saturation_pct`: float 0.0–100.0 representing bounded channel fill percentage
   - `last_hook_ts`: JSON object with per-hook-type ISO 8601 timestamps or `null` for hook types that have not fired since daemon start
   - `tui_attached`: boolean — `true` if a TUI client is currently connected via UDS
2. If the auth token is invalid: HTTP 401 per BC-2.01.009.
3. `/status` continues to serve during graceful shutdown drain (read-only; useful for drain monitoring).

## Invariants

1. `/status` requires authentication because it exposes internal buffer fill levels and channel saturation metrics that could reveal load patterns to a local adversary.
2. The `abi_version` field in the response MUST equal `monocle_core::MONOCLE_ABI_VERSION`. This enables Phase 3 plugin SDK and Phase 4 federation to gate on ABI compatibility.
3. `/status` is subject to the 256 KiB body size limit (BC-2.01.003) in its request path (even though GET responses are unbounded — the limit protects request ingestion, not response generation).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-042 | Phase 4 federation reads `abi_version` from a peer's `/status` and refuses to activate if the version is incompatible | Phase 1 daemon only needs to serve the field with the correct value; compatibility negotiation is Phase 4 scope |
| EC-043 | Initial daemon state — no ring or channel events yet | `ring_buffer_fill_pct` is `0.0`; `channel_saturation_pct` is `0.0` |
| EC-044 | `last_hook_ts` for a hook type that has not fired since daemon start | Value is JSON `null` (not an empty string); uses ISO 8601 format `YYYY-MM-DDTHH:MM:SS.sssZ` UTC for hook types that have fired |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `GET /status` with valid `X-Monocle-Authorization: monocle-v1:<token>` (canonical) | HTTP 200; body contains all 10 fields; `abi_version == 1` | happy-path |
| `GET /status` with valid `X-Claude-Code-Ide-Authorization: <64-hex>` (alias); no canonical header | HTTP 200; body contains all 10 fields; `abi_version == 1`; WARN deprecation log emitted | happy-path (alias) |
| `GET /status` (no auth header) | HTTP 401 `{"error":"missing_auth_token"}` | error |
| `GET /status` with invalid token | HTTP 401 `{"error":"invalid_auth_token"}` | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | Authenticated `/status` returns HTTP 200 with all 10 required fields including `abi_version == 1` | integration |
| VP-002 | Unauthenticated `/status` returns HTTP 401 | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the daemon state observation endpoint that exposes lifecycle and ring buffer health required for managing hook ingestion |
| L2 Domain Invariants | DI-002 (lock file must be present with valid port and auth token before hook endpoints accept connections — /status requires valid auth token from lock file); DI-005 (daemon must not accept a token that does not begin with the canonical monocle-v1: prefix — /status auth requirement enforces this per Postcondition 2 and BC-2.01.009) |
| Architecture Module | monocle-runtime (daemon binary, HTTP server) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.33 §Health and Status Endpoints §GET /status |
| Brief Section | §Scope (hook receiver hardening sub-bullet — `/status` daemon-state query endpoint) |
| Test File | `monocle-runtime/tests/status_endpoint_auth.rs` |
| Test Name | `test_BC_DAEMON_002_status_endpoint_requires_auth_and_returns_abi_version` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-DAEMON-002 |

## Related BCs (Recommended)

- [BC-2.01.001] — composes with: `/healthz` is the unauthenticated counterpart; `/status` is the authenticated full-state view
- [BC-2.01.003] — depends on: 256 KiB body limit applies to `/status` request path per BC-2.01.002 Invariant 3
- [BC-2.01.009] — depends on: auth header validation governs the HTTP 401 responses for this endpoint

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#health-and-status-endpoints` — HTTP server routing, authenticated router, `/status` endpoint spec and full response schema

## Story Anchor (Recommended)

S-TBD — Implement daemon /status endpoint with full observability fields (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-002-status-endpoint.md` — VP-002 status endpoint integration tests

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-002 ... ; DI-005 ...`
  - DI-002 mapping: /status requires authentication via a valid auth token read from the lock file — lock file must be present per DI-002. DI-005 mapping: /status enforces the monocle-v1: prefix requirement (Postcondition 2 delegates to BC-2.01.009 which enforces DI-005).
- F-R105-9 (SE-17c-d body-scope grep): Stale test name `test_BC_DAEMON_002_status_endpoint_requires_auth_and_returns_abi_version` in Traceability table — this is an intentional historical test name in the Old ID row, NOT a stale cross-reference. Body prose Related BCs use canonical `BC-2.01.NNN` form. 0 stale BC IDs in non-historical body prose. 0 stale VP IDs in body prose. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T11:30:00Z (v1.0).

## §Trace v1.0.2

**F-R106-12 MED — Stale (BC-AUTH-002) parenthetical removal in Postcondition 2** (2026-05-17T22:30:00Z):
- F-R106-12: Postcondition 2 contained `BC-2.01.009 (BC-AUTH-002)`. The `(BC-AUTH-002)` parenthetical is redundant renumbering noise — BC-INDEX §Renumbering Map preserves the old-ID→new-ID mapping; inline parentheticals in body prose are not needed and accumulate as noise in future adversarial sweeps.
- **SE-17f Postcondition 2 before/after:**
  - Before: `If the auth token is invalid: HTTP 401 per BC-2.01.009 (BC-AUTH-002).`
  - After: `If the auth token is invalid: HTTP 401 per BC-2.01.009.`
  - Rationale: canonical form `BC-2.01.009` is sufficient; old ID is preserved only in the Old ID (historical) row of the referenced BC file and in BC-INDEX §Renumbering Map, not in cross-reference prose.
- SE-17c-d body-scope grep: `(BC-AUTH-002)` in Postcondition 2 was the only stale old-form parenthetical in non-historical body prose. Historical row `BC-DAEMON-002` in Traceability remains (correct; that is BC-2.01.002's own old ID). 0 stale VP IDs. 0 other stale BC IDs.
- SE-16d monotonicity PASS: 2026-05-17T22:30:00Z > prior 2026-05-17T18:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R107-2 CRITICAL — Architecture Source pin refresh v1.0.25 → v1.0.30** (2026-05-17T23:30:00Z):
- F-R107-2: Sibling-layer cascade miss from Round 5D (VPs swept but BCs not). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.25 §Health and Status Endpoints §GET /status`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.30 §Health and Status Endpoints §GET /status`
  - Canonical version per architect 5E commit 03a4c57 post-R106 closure.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T22:30:00Z (v1.0.2).

## §Trace v1.0.4

**F-R108-17 MEDIUM — Dual-accept alignment: Description, Precondition 2, test vector** (2026-05-18T01:00:00Z):
- F-R108-17: BC-2.01.002 Description (line 34), Precondition 2 (line 40), and canonical test vector all implied single-header `X-Monocle-Authorization` only. This contradicts ADR-0005 v1.0.2 dual-accept protocol (both `X-Monocle-Authorization` canonical and `X-Claude-Code-Ide-Authorization` alias are valid) which is correctly specified in BC-2.01.009 INV-3 and BC-2.01.004 INV-3.
- **SE-17f Description BEFORE:** `...requires a valid X-Monocle-Authorization: monocle-v1:<token> header.`
- **SE-17f Description AFTER:** `...requires a valid auth header per the dual-accept protocol (ADR-0005 v1.0.2): either X-Monocle-Authorization: monocle-v1:<token> (canonical) or X-Claude-Code-Ide-Authorization: <64-hex> (compatibility alias); full validation semantics are defined in BC-2.01.009.`
- **SE-17f Precondition 2 BEFORE:** `A GET /status request arrives with a valid X-Monocle-Authorization: monocle-v1:<token> header.`
- **SE-17f Precondition 2 AFTER:** `A GET /status request arrives with a valid auth header per dual-accept protocol (ADR-0005 v1.0.2): either X-Monocle-Authorization: monocle-v1:<token> (canonical) or X-Claude-Code-Ide-Authorization: <64-hex> (compatibility alias). Full dual-accept validation semantics are defined in BC-2.01.009.`
- **SE-17f test vector BEFORE (happy-path row):** `GET /status with valid X-Monocle-Authorization header`
- **SE-17f test vector AFTER:** split into two rows — canonical happy-path and alias happy-path (alias emits WARN deprecation log per BC-2.01.009 PC-3/INV-6).
- SE-17c-d body-scope grep: Description, Precondition 2, and test vector table are the only normative changes in this version. 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. Postconditions 1-3 and Invariants 1-3 are unchanged — they correctly delegate auth-failure semantics to BC-2.01.009.
- SE-16d monotonicity PASS: 2026-05-18T01:00:00Z > prior 2026-05-17T23:30:00Z (v1.0.3).

## §Trace v1.0.5

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.0.30 → v1.0.32; F-R109-14 MED — §Trace reordered ascending** (2026-05-18T05:01:00Z):
- F-R109-4: Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32 (Round 8A). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.30 §Health and Status Endpoints §GET /status`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.32 §Health and Status Endpoints §GET /status`
- F-R109-14: §Trace blocks were descending (v1.0.4, v1.0.3, v1.0.2, v1.0.1). Reordered to ascending (v1.0.1 → v1.0.4 → v1.0.5). Content of each section preserved verbatim; only insertion order corrected.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:01:00Z > prior 2026-05-18T01:00:00Z (v1.0.4). ARITHMETICALLY TRUE: 2026-05-18T05:01:00Z > 2026-05-18T01:00:00Z PASS.

## §Trace v1.0.6

**GAP-PHASE2-R06-1 closure — Architecture Source pin SS-daemon-lifecycle v1.0.32 → v1.0.33** (2026-05-19T12:01:00Z):
- GAP-PHASE2-R06-1: architect commit `2d43127` bumped SS-daemon-lifecycle.md v1.0.32 → v1.0.33 (Ring Buffer Rotation Policy added). BC ledger Architecture Source cell was not cascaded in that commit.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.32 §Health and Status Endpoints §GET /status`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.33 §Health and Status Endpoints §GET /status`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:01:00Z > prior 2026-05-18T05:01:00Z (v1.0.5). ARITHMETICALLY TRUE: PASS.
