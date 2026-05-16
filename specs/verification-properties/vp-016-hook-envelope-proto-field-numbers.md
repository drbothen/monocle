---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
traces_to: prd.md
source_bc: BC-2.02.006
module: monocle-proto
proof_method: integration-test
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-016: Proto Field Number 1 in `HookEnvelope` is `schema_version`

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-PROTO-001a (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

In `monocle-proto/proto/monocle/v1/hook_envelope.proto`, the `HookEnvelope`
message's field assigned to proto-tag-number `1` has the field name
`schema_version` and type `uint32`. The wire-level invariant is verified
by encoding a `HookEnvelope` and decoding the first field tag.

## Source Contract

- **BC:** BC-2.02.006 — Hook Envelope Proto Field Numbers (Wire Schema).
- **Postcondition/Invariant:** BC-2.02.006 Postcondition asserting
  `schema_version = 1` proto-tag wire-format assignment in `HookEnvelope`
  message; cross-property with VP-017 (Rust surface) and VP-018
  (forward-compat dispatch structural recap).
- **Traces to (historical):** BC-PROTO-001a (SS-core-types-and-abi.md
  §Prost Wire Schemas; PRD v1.25 §BC-PROTO-001a Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | prost + prost-reflect | Bounded — single message field | Wire-tag decode of field 1; FileDescriptorSet name lookup |

## Mechanism

Integration test (harness located at
`monocle-proto/tests/wire_field_order.rs` — files in `<crate>/tests/`
are cargo integration tests; PRD v1.25 §7 RTM Test Type column labels
this BC `Unit` referring to conceptual scope, but the harness layout is
cargo-integration per file location). The harness encodes a
`HookEnvelope` via `prost::Message::encode_to_vec`, parses the first
wire-tag to confirm field number 1 with wire type `Varint`, then
inspects the FileDescriptorSet (from `prost-build`'s
`OUT_DIR/file_descriptor_set.bin` or compile-time include) to confirm
field 1 name == "schema_version".

## Pre-conditions

- `monocle-proto` build script (`build.rs`) compiles the `.proto` files
  via `prost-build`.
- `prost-reflect` or direct `prost::encoding` is available in
  `[dev-dependencies]`.

## Post-conditions

1. Encoding a `HookEnvelope { schema_version: 1, ... }` via
   `prost::Message::encode_to_vec(&envelope)` produces a byte stream
   whose first wire-tag decodes to field number 1 with wire type
   `Varint` (proto3 `uint32` = varint).
2. A `prost-build`-generated descriptor inspection (via
   `prost_reflect::DescriptorPool::decode(...)` over the
   FileDescriptorSet emitted by `build.rs`) confirms field number 1 is
   named `schema_version`.

## Counter-examples

1. The `.proto` file is edited so `schema_version = 5;` — must fail the
   wire-tag decode (the first tag would decode to field 5 instead of 1).
2. A new field `string trace_id = 1;` is inserted, displacing
   `schema_version` to a new number — must fail the field-name lookup.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 16.a | `HookEnvelope { schema_version: 1, ... }` encoded; parse first wire-tag | field_number == 1; wire_type == Varint |
| 16.b | Inspect FileDescriptorSet via `prost_reflect::DescriptorPool::decode` | field 1 name == "schema_version" |
| 16.c | Mutate `.proto` to renumber `schema_version = 5;` | Probe 16.a fails (first tag decodes to 5) |
| 16.d | Mutate `.proto` to insert `string trace_id = 1;` displacing `schema_version` | Probe 16.b fails (field 1 name != "schema_version") |

## Harness Location

- `monocle-proto/tests/wire_field_order.rs` (integration test)
- Test name: `test_BC_PROTO_001a_schema_version_field_number_1` (per
  PRD v1.25 §BC-PROTO-001a, Verification subsection — to be migrated to
  `test_BC_2_02_006_schema_version_field_number_1` post BC renumber
  propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-PROTO-001a at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.006.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §Prost Wire Schemas.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.006 (Dispatch 4 commit 1030c65).
- Cross-VP: VP-017 (Rust surface; same wire contract from Rust side); VP-018
  (Phase 1 structural recap + Phase 4 runtime dispatch).
