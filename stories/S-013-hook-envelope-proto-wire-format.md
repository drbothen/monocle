---
document_type: story
story_id: S-013
epic_id: EPIC-02
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-010]
blocks: []
target_module: monocle-proto
subsystems: [SS-02]
behavioral_contracts: [BC-2.02.006, BC-2.02.007, BC-2.02.008]
verification_properties: [VP-016, VP-017, VP-018]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.11"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.006.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.007.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.008.md, version: "1.0.3"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-016-hook-envelope-proto-field-numbers.md, version: "1.0.12"}
  - {path: .factory/specs/verification-properties/vp-017-hook-envelope-schema-version-field.md, version: "1.0.13"}
  - {path: .factory/specs/verification-properties/vp-018-phase4-schema-version-validation.md, version: "1.0.12"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.10"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/architecture/SS-forward-compatibility.md, version: "1.2.19"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.02.006 (HookEnvelope Proto Field Number), BC-2.02.007 (HookEnvelope Rust Struct schema_version), BC-2.02.008 (Phase 4 schema_version Validation); verifies VP-016, VP-017, VP-018; addresses E-PROTO-001."
---

# S-013: HookEnvelope Proto Wire Format (FC-05)

## Narrative

As a Phase 4 federation layer consumer, I want the `HookEnvelope` protobuf message to
have `schema_version` as field number 1 and the corresponding Rust struct to expose
`pub schema_version: u32` with value `1`, so that Phase 4 can version-dispatch on the
first decoded field without parsing the full protobuf message.

## Acceptance Criteria

### AC-001 (traces to BC-2.02.006 postcondition 1 — proto field number 1 = schema_version)
In the `HookEnvelope` protobuf message definition (`monocle-proto/proto/hook_envelope.proto`),
`schema_version` is field number 1. This field number is IMMUTABLE per FC-05 — any change
is a BREAKING change requiring an ADR.

### AC-002 (traces to BC-2.02.007 postcondition 1 — Rust struct schema_version field)
The generated/hand-written `HookEnvelope` Rust struct in `monocle-proto` exposes
`pub schema_version: u32`. In Phase 1, `schema_version` value is always `1`.

### AC-003 (traces to BC-2.02.007 postcondition 2 — schema_version value 1 in Phase 1)
Integration test: create a `HookEnvelope { schema_version: 1, ... }` and assert the field
value equals `1`. No Phase 1 code path sets `schema_version` to any other value.

### AC-004 (traces to BC-2.02.008 postcondition 1 — Phase 4 dispatch contract documented)
`monocle-proto/src/lib.rs` includes a rustdoc comment on `HookEnvelope` documenting the
Phase 4 schema_version dispatch contract: unknown `schema_version` values (>1) must trigger
`WARN: HookEnvelope schema_version <N> not recognized; skipping` (E-PROTO-001).

### AC-005 (traces to BC-2.02.008 postcondition 2 — Phase 1 structural recap)
Phase 1 does not use protobuf encoding for any runtime wire path (hook POSTs use JSON).
`monocle-proto` declares `prost 0.14` as a dependency but NO Phase 1 code path invokes
protobuf serialization. The proto schema is declared now to lock the field number contract
before Phase 4 activation.

### AC-006 (traces to BC-2.02.006 invariant 1 — field number immutability)
`monocle-proto/tests/proto_field_numbers.rs` integration test parses the `.proto` file
and asserts that `schema_version` is field number 1. This test fails if the proto is
regenerated with a different field number.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~700 |
| BC-2.02.006.md | ~500 |
| BC-2.02.007.md | ~500 |
| BC-2.02.008.md | ~500 |
| VP-016 + VP-017 + VP-018 files | ~1,200 |
| SS-core-types-and-abi.md (proto section, ~60 lines) | ~900 |
| Test file | ~500 |
| **Total estimate** | **~4,800** |

## Tasks

- [ ] Create `monocle-proto/proto/hook_envelope.proto` with `HookEnvelope` message:
  - Field 1: `uint32 schema_version = 1;`
  - Additional Phase 1 fields: `string hook_type = 2`, `bytes payload = 3`
- [ ] Configure `monocle-proto/build.rs` with `prost-build` to generate Rust from proto
  - OR write the `HookEnvelope` Rust struct by hand to avoid build-time prost-build dependency
  - Decision: hand-write struct in Phase 1 (prost-build adds complexity; Phase 4 activates full prost)
- [ ] Create `monocle-proto/src/lib.rs` with hand-written `HookEnvelope` struct:
  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  pub struct HookEnvelope {
      pub schema_version: u32,  // Field 1; value 1 in Phase 1
      pub hook_type: String,    // Field 2
      pub payload: Vec<u8>,     // Field 3
  }
  ```
- [ ] Add rustdoc on `HookEnvelope` documenting Phase 4 dispatch contract (E-PROTO-001)
- [ ] Create `monocle-proto/tests/proto_field_numbers.rs` (VP-016 probe):
  - Parse `hook_envelope.proto` source; assert field number 1 = schema_version
- [ ] Integration test `monocle-proto/tests/schema_version.rs` (VP-017 probe):
  - `HookEnvelope { schema_version: 1, ... }.schema_version == 1`

## Previous Story Intelligence

S-010 (Wave 2): `monocle-core` crate foundation established. `monocle-proto` crate stub exists from S-001.
`prost 0.14` is pinned in workspace (EXACT pin). In Phase 1, prost is declared but NOT used at runtime.
The hand-written approach for Phase 1 avoids build.rs complexity while preserving the field-number contract.

## Architecture Compliance Rules

From `architecture/SS-core-types-and-abi.md` v1.2.13 §HookEnvelope Proto:
- Field number 1 = `schema_version` is IMMUTABLE — changing it is a BREAKING change requiring ADR
- Phase 1: prost is declared as a crate dependency but NO proto encoding/decoding occurs at runtime
- Phase 4: prost activates for cross-host federation wire format decoding of untrusted input

**Forbidden Dependencies:**
- `monocle-proto` MUST NOT depend on `monocle-runtime` or `monocle-tui`
- Phase 1 runtime MUST NOT call any prost encoding/decoding functions

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| prost | 0.14 | Declared as dep; no Phase 1 runtime usage (audit baseline lock) |
| serde | 1 | Serialize/Deserialize derive on HookEnvelope |

## File Structure Requirements

Files to create:
- `monocle-proto/proto/hook_envelope.proto` — proto message definition
- `monocle-proto/src/lib.rs` — hand-written `HookEnvelope` struct with rustdoc
- `monocle-proto/tests/proto_field_numbers.rs` — VP-016 proto field audit
- `monocle-proto/tests/schema_version.rs` — VP-017/VP-018 struct tests
