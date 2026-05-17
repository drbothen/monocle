---
document_type: behavioral-contract
level: L3
version: "1.0.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T18:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "0fad9fc"
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

# Behavioral Contract BC-2.02.008: Phase 4 schema_version Validation Requirement (FC-05)

## Description

Phase 4 federation nodes must check `schema_version` before deserializing event payloads
from `HookEnvelope` messages. An unrecognized `schema_version` value triggers a log-and-skip
response rather than a crash. Phase 1 defines the canonical `HookEnvelope` schema that Phase
4 will activate; no schema changes between Phase 1 definition and Phase 4 activation are
permitted without bumping `schema_version` and producing an ADR.

## Preconditions

1. Phase 4 federation is active (out of scope for Phase 1 testing; this BC defines the contract Phase 1 schema must support).
2. A Phase 4 federation node receives a `HookEnvelope` message.

## Postconditions

1. Phase 4 federation nodes check `schema_version` before deserializing event payloads.
2. A node receiving a message with an unrecognized `schema_version` MUST log a warning and skip the message rather than crash (proto3 unknown-field semantics apply).
3. The Phase 1 `HookEnvelope` schema is the canonical wire representation for cross-host federation in Phase 4. No schema changes are permitted between Phase 1 definition and Phase 4 activation without bumping `schema_version` and producing an ADR.

## Invariants

1. Proto3 forward compatibility guarantee: a Phase 4 receiver that understands Phase 5+ fields can still decode a Phase 1 message (unknown fields are preserved in proto3).
2. The `schema_version` field exists specifically so Phase 4 can distinguish Phase 1 messages from future-format messages without heuristics.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-027 | Phase 4 receiver encounters `schema_version: 0` | Must skip with WARN log; value 0 is not a defined version; must not panic |
| EC-028 | Phase 4 receiver encounters `schema_version: 2` (hypothetical Phase 5 format) | Must skip with WARN log (unrecognized version); must not attempt to decode as Phase 1 format |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Message with `schema_version: 0` | Skip with WARN log; no panic | edge-case |
| Message with `schema_version: 1` | Decode successfully | happy-path |
| Message with `schema_version: 2` | Skip with WARN log; no panic | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-018 | Phase 1 gate: `schema_version` field is present at field number 1 in the compiled schema (BC-2.02.006/BC-2.02.007); Phase 4 test (out of Phase 1 scope): `schema_version: 0` message skips without panic | integration (Phase 1 gate); integration (Phase 4 gate) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability §SS-02 |
| Capability Anchor Justification | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability — this BC ensures the wire format stability requirement extends into Phase 4 federation by mandating schema_version validation at message ingestion |
| L2 Domain Invariants | DI-004 (all public wire types must carry a version discriminant as their first field — this BC specifies how the schema_version discriminant defined by BC-2.02.006/BC-2.02.007 is consumed at runtime: readers check it BEFORE deserializing remaining fields, which is exactly "detect format evolution without parsing the full record"; unrecognized values log-and-skip rather than crash) |
| Architecture Module | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) per ARCH-INDEX Subsystem Registry SS-02 |
| Architecture Source | SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas |
| FC | FC-05 |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — prost wire format) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-PROTO-002 |
| Test name | test_BC_PROTO_002_schema_version_validation_skip_unknown (Phase 4 test) |

## Related BCs (Recommended)

- [BC-2.02.006] — depends on: the field number contract that makes schema_version detectable
- [BC-2.02.007] — depends on: the Rust struct field that Phase 4 code reads to check the version

## Architecture Anchors (Recommended)

- `architecture/SS-core-types-and-abi.md#prost-wire-schemas` — schema_version validation requirement, Phase 4 field number reservation, deprecation policy

## Story Anchor (Recommended)

S-TBD — Phase 4 federation schema_version validation (filled by story-writer; Phase 4 scope)

## VP Anchors (Recommended)

- `verification-properties/vp-018-phase4-schema-version-validation.md` — VP-018 Phase 4 schema_version skip-without-panic test

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-002 per ARCH-INDEX is authoritative source`
  - After: `DI-004 ...`
  - DI-004 mapping: This BC is the runtime consumption half of DI-004 — it specifies how Phase 4 readers use the schema_version discriminant to gate deserialization. Postcondition 1 ("check schema_version before deserializing") is the behavioral expression of DI-004's "detect format evolution without parsing the full record."
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).
