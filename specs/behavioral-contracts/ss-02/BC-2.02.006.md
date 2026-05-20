---
document_type: behavioral-contract
level: L3
version: "1.0.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T05:15:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "f354d83"
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

# Behavioral Contract BC-2.02.006: HookEnvelope Proto Field Number Contract (FC-05, wire-format)

## Description

`schema_version` is assigned proto field number 1 in the `HookEnvelope` message. This is
a wire-format contract — the binary protobuf encoding uses tag `0x08` for field number 1.
The five Phase 1 event variants occupy field numbers 10–14 in a `oneof event`. Phase 4
federation additions must use field numbers 100–999; Phase 5+ must use 1000+. Any change
to Phase 1 field numbers 1–99 is a BREAKING change requiring a schema version bump and ADR.

## Preconditions

1. The `.proto` file `monocle-proto/proto/monocle/v1/hook_envelope.proto` exists and compiles via `protoc` or `prost-build`.

## Postconditions

1. `schema_version` is declared at proto FIELD NUMBER 1 in `HookEnvelope` (`uint32 schema_version = 1;`).
2. This is a WIRE-FORMAT contract: field number 1 in proto3 binary encoding uses tag `0x08` (field 1, varint type). Any proto consumer in any language sees field number 1 as `schema_version`.
3. The proto package is `monocle.v1`. The file path is `monocle-proto/proto/monocle/v1/hook_envelope.proto`.
4. The oneof `event` in `HookEnvelope` uses field numbers 10–14 for the five event variants: `session_start (SessionStartEvent) = 10`, `prompt_submit (UserPromptSubmitEvent) = 11`, `pre_tool_use (PreToolUseEvent) = 12`, `notification (NotificationEvent) = 13`, `stop (StopEvent) = 14`. Phase 1 reserved range is 1–99. Field names are snake_case.
5. Phase 4 federation additions MUST use field numbers 100–999. Phase 5+ MUST use 1000+.

## Invariants

1. Any change to a Phase 1 field (field numbers 1–99) in `HookEnvelope` or any event message is a BREAKING change: bump `schema_version` AND produce an ADR.
2. Removing a Phase 1 field is forbidden. Deprecated fields are marked `[deprecated = true]` and the field number is retained as reserved.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-024 | Phase 1 receiver encounters a `HookEnvelope` with fields in the 100–999 range (Phase 4 additions) | Proto3 forward compatibility: unknown fields are preserved; Phase 1 receiver does not crash or reject the message |
| EC-025 | `schema_version` field value is `0` in a received message | Phase 4 federation test case: receiver MUST log a warning and skip without panic; value `0` is not a valid Phase 1 schema version |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| `protoc --decode monocle.v1.HookEnvelope` on an encoded Phase 1 message | Field number 1 is `schema_version` | happy-path |
| Round-trip `HookEnvelope { schema_version: 1, ... }` via prost encode/decode | `envelope.schema_version == 1` after decode | happy-path |
| Receive envelope with unknown field numbers 100–999 | Message decoded without error; known fields intact | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-016 | Round-trip encode/decode of `HookEnvelope` via prost asserts field number 1 is `schema_version` via prost-build's generated descriptor | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability §SS-02 |
| Capability Anchor Justification | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability — this BC governs the wire format stability component of CAP-002 by locking HookEnvelope proto field numbers |
| L2 Domain Invariants | DI-004 (all public wire types must carry a version discriminant as their first field — schema_version at proto field number 1 in HookEnvelope is the wire-format version discriminant; assigning field number 1 ensures the discriminant is the first varint in the serialized binary, enabling readers to detect format evolution without parsing the full envelope) |
| Architecture Module | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) per ARCH-INDEX Subsystem Registry SS-02 |
| Architecture Source | SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas |
| FC | FC-05 |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — prost wire format) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-PROTO-001a |
| Test name | test_BC_PROTO_001a_schema_version_field_number_1 |

## Related BCs (Recommended)

- [BC-2.02.007] — composes with: the Rust struct surface companion to this wire-format contract
- [BC-2.02.008] — depends on: Phase 4 schema_version validation relies on this field number being stable

## Architecture Anchors (Recommended)

- `architecture/SS-core-types-and-abi.md#prost-wire-schemas` — HookEnvelope field number registry, phase-range assignments, deprecation policy

## Story Anchor (Recommended)

S-TBD — Implement monocle-proto crate with HookEnvelope proto definition (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-016-hook-envelope-proto-field-numbers.md` — VP-016 prost encode/decode field number test

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-002 per ARCH-INDEX is authoritative source`
  - After: `DI-004 ...`
  - DI-004 mapping: Proto field number 1 = schema_version is exactly "version discriminant as first field" in the binary wire encoding. Field number 1 encodes as tag 0x08, the lowest-numbered tag, appearing first in the proto3 canonical serialization. This is the wire-format implementation of DI-004.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.2.8 → v1.2.13** (2026-05-18T05:15:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.2.8 → v1.2.13); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas`
  - SE-17f AFTER: `SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:15:00Z > prior 2026-05-17T18:00:00Z (v1.0.2). ARITHMETICALLY TRUE: 2026-05-18T05:15:00Z > 2026-05-17T18:00:00Z PASS.
