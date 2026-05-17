---
document_type: behavioral-contract
level: L3
version: "1.0.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T05:09:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "a9aeb88"
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

# Behavioral Contract BC-2.01.010: Lock File Contract Version Field

## Description

The monocle lock file is a JSON object whose first key is always `contract_version: 1`,
enabling any Phase 1 or future reader to validate the format before deserializing remaining
fields. The `app` field is `"monocle"` to support future hook-discovery tooling. This
forward-compatibility convention parallels the `format_version` first-key pattern in the
JSONL ring (BC-2.01.007), ensuring consistent version-gate semantics across all monocle
wire formats. Readers encountering an unrecognized `contract_version` must log a warning
and skip the file gracefully (no crash).

## Preconditions

1. The monocle daemon has completed step 6 of its start sequence (lock file written via `tempfile::persist`).

## Postconditions

1. The lock file JSON is a valid JSON object containing at minimum these fields in the stated order: `contract_version` (first), `pid`, `port`, `authToken`, `startTimeUtc`, `app`, `version`.
2. `contract_version` is always the FIRST key in the JSON object. Value is `1` for all Phase 1 daemons.
3. `app` field is `"monocle"` — allows future hook-discovery tooling to filter by app name without scanning all lock files.
4. Any lock-file reader MUST check `contract_version == 1` before consuming other fields. An unrecognized `contract_version` triggers a graceful skip with a log warning — no panic, no crash.

## Invariants

1. `contract_version` field order parallels the `format_version` convention in the JSONL ring (BC-2.01.007). Both formats put the version sentinel first so readers can validate before deserializing remaining fields.
2. The lock file is always written atomically via `tempfile::persist` — no partial lock file with only some fields is observable by concurrent readers.
3. Lock file mode: `0o600` (owner-only read/write). Neither group nor other permissions are set.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-010 | Stale lock file with `contract_version` from a future daemon (hypothetical Phase 4 format) | Phase 1 TUI reader encountering an unrecognized `contract_version` MUST log `WARN: lock file contract_version <N> not recognized; skipping` and proceed as if no lock file exists (trigger the "no daemon running" flow, which auto-starts the daemon) |
| EC-011 | Lock file with `contract_version` key present but value not an integer (e.g., `"contract_version": "1"` as string instead of integer) | Phase 1 reader must handle this gracefully (coerce-to-integer or log and skip) |
| EC-012 | Lock file with `contract_version` key missing entirely (pre-Phase-1 format) | Same treatment as EC-010: log `WARN: lock file contract_version missing; skipping` and proceed as if no lock file exists |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Read lock file after daemon start | `contract_version` is integer `1`, present as first key | happy-path |
| Read `app` field from lock file | `"monocle"` — exact string, no variants | happy-path |
| Read `authToken` field from lock file | 64-char hex string matching `/^[0-9a-f]{64}$/` | happy-path |
| Reader encounters lock file with `contract_version: 99` | WARN log, skip, proceed as if no daemon running | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-010 | Lock file has `contract_version == 1` as first key (verified via `serde_json::Value::Object` iteration which preserves insertion order for `serde_json::Map<String, Value>`) | integration |
| VP-010 | Lock file `app` field equals `"monocle"` | integration |
| VP-010 | Reader encountering unrecognized `contract_version` logs WARN and skips without crashing | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the lock file forward-compatibility contract that enables future daemon versions and tooling to safely interoperate with the hook ingestion subsystem |
| L2 Domain Invariants | DI-002 (the lock file must contain a valid port and auth token before any hook endpoint accepts connections — this BC defines the JSON schema of that lock file, including the contract_version first-key convention that allows readers to validate the format before consuming port and authToken); DI-004 (all public wire types must carry a version discriminant as their first field — contract_version as the first key in the lock file JSON directly implements DI-004 for the lock file wire format, paralleling the format_version convention in BC-2.01.007) |
| Architecture Module | monocle-runtime (daemon binary, lock file) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging |
| Test File | `monocle-runtime/tests/lock_file_contract.rs` |
| Test Name | `test_BC_LOCK_001_contract_version_first_key` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-LOCK-001 |

## Related BCs (Recommended)

- [BC-2.01.005] — composes with: lock file is created by BC-2.01.005 start sequence; this BC specifies the JSON schema of that file
- [BC-2.01.007] — related to: `contract_version` first-key convention parallels `format_version` first-key convention in BC-2.01.007 JSONL ring
- [BC-2.01.008] — composes with: `authToken` field format (64-char hex) is specified in BC-2.01.008

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#daemon-lifecycle-protocol` — lock file JSON schema, field order, `contract_version` convention
- `architecture/SS-core-types-and-abi.md` — Phase 1 PRD BC Pre-Staging reference for this contract

## Story Anchor (Recommended)

S-TBD — Implement lock file JSON schema with contract_version first-key (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-010-lock-file-contract-version.md` — VP-010 lock file contract version integration tests

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-002 ... ; DI-004 ...`
  - DI-002 mapping: This BC specifies the lock file schema that enables consumers to validate the format (contract_version) before extracting the port and authToken. It directly supports DI-002 by defining what "valid" lock file content means. DI-004 mapping: contract_version as the first key is the explicit first-field version discriminant implementation for the lock file wire type.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T11:30:00Z (v1.0).

## §Trace v1.0.2

**F-R107-2 CRITICAL — Architecture Source pin refresh v1.0.25 → v1.0.30** (2026-05-17T23:30:00Z):
- F-R107-2: Sibling-layer cascade miss from Round 5D (VPs swept but BCs not). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.25 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.30 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging`
  - Canonical version per architect 5E commit 03a4c57 post-R106 closure. (Note: SS-core-types-and-abi.md pin is not updated here — it is a different architecture document not in scope of F-R107-2 which is specifically about SS-daemon-lifecycle.md version pins.)
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T18:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.0.30 → v1.0.32; F-R109-14 MED — §Trace reordered ascending** (2026-05-18T05:09:00Z):
- F-R109-4: Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32 (Round 8A). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.30 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging`
- F-R109-14: §Trace blocks were descending (v1.0.2, v1.0.1). Reordered to ascending (v1.0.1, v1.0.2, v1.0.3). Content of each section preserved verbatim; only insertion order corrected.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:09:00Z > prior 2026-05-17T23:30:00Z (v1.0.2). ARITHMETICALLY TRUE: 2026-05-18T05:09:00Z > 2026-05-17T23:30:00Z PASS.
