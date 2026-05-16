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
source_bc: BC-2.02.007
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

# VP-017: Rust `HookEnvelope` Struct Exposes `pub schema_version: u32` with Value `1`

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-PROTO-001b (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

The prost-build-generated Rust type `monocle_proto::v1::HookEnvelope`
exposes a public field `schema_version: u32`. For all Phase 1-origin
messages (those constructed inside Phase 1 monocle code), the value of
`schema_version` is `1`.

## Source Contract

- **BC:** BC-2.02.007 — Hook Envelope `schema_version` Rust Surface.
- **Postcondition/Invariant:** BC-2.02.007 Postconditions asserting
  generated Rust struct field type `pub schema_version: u32` and Phase 1
  canonical value `1`; round-trip serialize/deserialize preserves field
  value; cross-property with VP-016 (proto wire surface) and VP-018
  (Phase 1 structural recap + Phase 4 runtime dispatch).
- **Traces to (historical):** BC-PROTO-001b (SS-core-types-and-abi.md
  §Prost Wire Schemas; PRD v1.25 §BC-PROTO-001b Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | prost | Bounded — single struct field | Field-type `u32`, value `1`, encode/decode round-trip preservation |

## Mechanism

Integration test (harness located at
`monocle-proto/tests/schema_version.rs` — files in `<crate>/tests/` are
cargo integration tests; PRD v1.25 §7 RTM Test Type column labels this
BC `Unit` referring to conceptual scope, but the harness layout is
cargo-integration per file location). The harness constructs a
`HookEnvelope` with `schema_version: 1` and any oneof event variant,
asserts the field value equals `1`, then performs a `prost::Message`
encode → decode round-trip and asserts the post-decode `schema_version`
still equals `1`.

## Pre-conditions

- `monocle-proto` builds cleanly.
- The `pub use monocle::v1` re-export is present so callers can access
  `monocle_proto::v1::HookEnvelope`.
- `prost 0.14` is the project pin (per SS-deps-pin-manifest.md v1.1.15)
  for the prost-build-generated `HookEnvelope` struct. The struct exposes
  `#[derive(prost::Message)]` to enable wire-format `encode_to_vec()` /
  `decode()` round-trips asserted in §Post-conditions 1 and 2.
- Bare `serde 1` is NOT a required dependency for this VP. The
  prost-build-generated `HookEnvelope` struct uses `prost::Message`
  derives (which provide protobuf wire-format encode/decode) and does
  NOT derive `serde::Serialize` / `serde::Deserialize`. The
  `monocle-proto` crate declares `prost` and `bytes` only (per
  SS-deps-pin-manifest.md v1.1.15 workspace dep graph: `proto → prost`,
  `proto → bytes`) and has no workspace edge to `serde`. The F-R76-1
  closure (architect v1.1.10 → v1.1.11) added bare `serde 1` to the
  manifest for `monocle-runtime` and `monocle-core` consumers;
  `monocle-proto` is NOT among them. This VP's integration test
  (`monocle-proto/tests/schema_version.rs`) constructs the struct via
  `HookEnvelope { schema_version: 1, ... }` and asserts via
  `prost::Message::encode_to_vec` + `prost::Message::decode`, never via
  `serde_json::to_string` or `serde_yaml_ng::to_string`. (F-R76-1
  §Trace audit-row reconciliation — prior §Trace v1.8/v1.9/v1.10 audits
  claimed this VP cited `serde 1` verbatim; that claim was a fabrication.
  The correct disposition, documented here, is that serde is not on the
  `monocle-proto` dependency path.)

## Post-conditions

1. An integration test constructs a `HookEnvelope` with
   `schema_version: 1` and any `oneof event` variant (e.g.,
   `SessionStartEvent { cwd: "/", transcript_path: "" }`) and asserts
   `envelope.schema_version == 1`.
2. Round-trip serialize/deserialize preserves `schema_version`:
   `HookEnvelope::decode(envelope.encode_to_vec().as_slice()).unwrap()
   .schema_version == 1`.
3. The Rust struct field declaration order is NOT asserted (per
   BC-2.02.007 normative carve-out — the proto-tag-number is the wire
   contract, not the Rust field declaration order).

## Counter-examples

1. The `.proto` file changes `uint32 schema_version = 1;` to
   `int32 schema_version = 1;` — would change the Rust type to `i32`;
   the `pub schema_version: u32` type check fails.
2. A constructor helper in `monocle-proto` defaults `schema_version` to
   `0` — fails post-condition 1.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 17.a | Construct `HookEnvelope { schema_version: 1, event: SessionStartEvent { ... } }` | `envelope.schema_version == 1` |
| 17.b | Round-trip: `HookEnvelope::decode(envelope.encode_to_vec())` | Decoded `schema_version == 1` |
| 17.c | Type pin: `let _: u32 = envelope.schema_version;` | Compiles |
| 17.d | Mutate `.proto` to `int32 schema_version = 1;` | Probe 17.c fails (type becomes `i32`) |
| 17.e | Mutate constructor helper default to `0` | Probe 17.a fails |

## Harness Location

- `monocle-proto/tests/schema_version.rs` (integration test)
- Test name: `test_BC_PROTO_001b_schema_version_rust_field` (per PRD
  v1.25 §BC-PROTO-001b, Verification subsection — to be migrated to
  `test_BC_2_02_007_schema_version_rust_field` post BC renumber
  propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-PROTO-001b at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.007.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §Prost Wire Schemas.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.007 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15
  (workspace dep graph: `proto → prost`, `proto → bytes`).
- Cross-VP: VP-016 (proto wire surface; same `schema_version` contract from
  proto side); VP-018 (Phase 1 structural recap + Phase 4 runtime dispatch).
