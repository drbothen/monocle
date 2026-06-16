---
document_type: behavioral-contract
level: L3
version: "1.0.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T05:10:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "605896c"
traces_to: prd.md
origin: greenfield
subsystem: SS-02
capability: CAP-002
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

# Behavioral Contract BC-2.02.001: ABI Version in /status Endpoint (FC-03)

## Description

The monocle daemon exposes the compiled `MONOCLE_ABI_VERSION` constant via the
authenticated `/status` endpoint so that plugin SDKs, federation peers, and TUI
clients can verify ABI compatibility before activating. For Phase 1 binaries the
value is always the integer `1`. Changing this value requires an ADR.

## Preconditions

1. The monocle daemon is running and has been authenticated successfully.
2. A `GET /status` request is issued with a valid `X-Monocle-Authorization: monocle-v1:<token>` header.

## Postconditions

1. The JSON response body includes an `abi_version` field with integer value `1`.
2. The value equals `monocle_core::MONOCLE_ABI_VERSION` as compiled into the running binary. For Phase 1 binaries, this is always `1`.
3. The full `/status` response shape (per SS-daemon-lifecycle.md §Health and Status Endpoints) includes: `pid`, `uptime_sec`, `version`, `abi_version`, `lock_file`, `hook_endpoints`, `ring_buffer_fill_pct`, `channel_saturation_pct`, `last_hook_ts`, `tui_attached`.

## Invariants

1. `MONOCLE_ABI_VERSION` is a compile-time constant. It cannot differ between a running daemon and the constant exported by `monocle-core`. If they differ, the binary was built with a different `monocle-core` than the one the plugin SDK or federation layer expects.
2. Changing `MONOCLE_ABI_VERSION` from `1` requires an ADR.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-013 | Phase 3 plugin SDK encounter with a Phase 1 daemon. The SDK reads `abi_version` from `/status` and must refuse to activate plugins compiled against a different ABI version. | Phase 1 scope: ensure the field is present and correct; plugin refusal is Phase 3 behavior |
| EC-014 | Federation handshake (Phase 4) where two daemons running different ABI versions attempt to federate. The initiating daemon reads `abi_version` from the peer's `/status`. | Phase 1 scope: serve the field; compatibility resolution (HTTP 409) is Phase 4 scope |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `GET /status` (authenticated) | JSON body includes `"abi_version": 1` | happy-path |
| `GET /status` (unauthenticated) | HTTP 401 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-011 | `/status` response includes `"abi_version": 1` (integer, not string) when daemon is running a Phase 1 binary | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability §SS-02 |
| Capability Anchor Justification | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability — this BC governs the ABI version field in the /status endpoint, which is the primary forward-compatibility signal for the ABI contract |
| L2 Domain Invariants | DI-004 (all public wire types must carry a version discriminant as their first field — the abi_version field in the /status JSON response is the version discriminant for the daemon's ABI contract, enabling plugin SDKs and federation peers to detect ABI version before processing without parsing the full response) |
| Architecture Module | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) per ARCH-INDEX Subsystem Registry SS-02 |
| Architecture Source | SS-core-types-and-abi.md v1.2.13 §ABI Version Constant |
| FC | FC-03 |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — ABI version constant) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-ABI-001 |
| Test name | test_BC_ABI_001_status_endpoint_returns_abi_version_1 |

## Related BCs (Recommended)

- [BC-2.02.002] — composes with: the crate-root ABI constant that this endpoint reflects
- [BC-2.01.002] — depends on: the full /status endpoint shape that contains `abi_version`

## Architecture Anchors (Recommended)

- `architecture/SS-core-types-and-abi.md#abi-version-constant` — ABI version constant definition and export spec

## Story Anchor (Recommended)

S-TBD — Implement ABI version constant and /status endpoint abi_version field (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-011-abi-version-status-endpoint.md` — VP-011 ABI version in /status integration test

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-002 per ARCH-INDEX is authoritative source`
  - After: `DI-004 ...`
  - DI-004 mapping: The abi_version field in /status is the version discriminant for the daemon's public ABI — downstream consumers (plugin SDKs, federation peers) read this field before activating, enabling format evolution detection at the wire level.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T12:00:00Z (v1.0).

## §Trace v1.0.2

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.2.8 → v1.2.13** (2026-05-18T05:10:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.2.8 → v1.2.13); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-core-types-and-abi.md v1.2.8 §ABI Version Constant`
  - SE-17f AFTER: `SS-core-types-and-abi.md v1.2.13 §ABI Version Constant`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:10:00Z > prior 2026-05-17T18:00:00Z (v1.0.1). ARITHMETICALLY TRUE: 2026-05-18T05:10:00Z > 2026-05-17T18:00:00Z PASS.
