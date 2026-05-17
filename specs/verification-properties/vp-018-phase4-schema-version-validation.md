---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.02.008
module: monocle-proto
proof_method: integration-test+fuzz
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

# VP-018: `schema_version` Forward-Compat Contract (Phase 1 Structural Recap; Phase 4 Runtime Dispatch)

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-PROTO-002 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

**Phase 1 (structural):**

1. The compiled proto schema has a field named `schema_version` at proto
   field number `1` of `HookEnvelope` with type `uint32`. This is the
   structural precondition for any future runtime dispatcher.
2. The generated Rust struct `monocle_proto::v1::HookEnvelope` exposes
   `pub schema_version: u32` and the value `1` is the Phase 1 canonical
   value.

These two properties are already covered by VP-016 (wire-format) and
VP-017 (Rust surface). VP-018's Phase 1 verification is therefore a
structural recap that asserts these two properties IN COMBINATION — both
must hold for any future dispatcher to function.

**Phase 4 (runtime dispatch — deferred):**

1. When Phase 4's `monocle-ipc` crate exists with its dispatcher (to be
   designed in Phase 4 architecture), a `HookEnvelope` message with
   `schema_version = 0` or any unrecognized value other than `1` MUST be
   processed by:
   - Emitting a `tracing::warn!` event with the structured field
     `schema_version = <unknown_value>` and a descriptive message.
   - Returning success (skip) without panic and without propagating an
     error to the caller. The exact dispatcher API (function signature,
     error type, return type) is a Phase 4 design decision.
2. The forward-compatibility contract is: a Phase 1 daemon talking to a
   future Phase 4 peer that sends an unknown `schema_version` MUST NOT
   crash; conversely, a Phase 4 daemon receiving a Phase 1
   `schema_version = 1` message MUST process it normally.

## Source Contract

- **BC:** BC-2.02.008 — Hook Envelope Forward-Compat Dispatch Contract.
- **Postcondition/Invariant:** BC-2.02.008 Phase 1 structural recap of
  VP-016 + VP-017 plus Phase 4 runtime warn-and-skip dispatch contract.
  The Phase 4 mechanical property does NOT mandate a Phase 1 code surface
  in `monocle-proto`. The `monocle-ipc::dispatch` crate, the
  `dispatch_envelope` function signature, and the `DispatchError` type
  (or equivalent) are Phase 4 deliverables and will be specified by the
  Phase 4 architecture artifact.
- **Traces to (historical):** BC-PROTO-002 (SS-core-types-and-abi.md v1.2.8
  §Prost Wire Schemas; PRD v1.25 §BC-PROTO-002 Verification subsection).

### Reframing Rationale (F-R62-7 — PG-5 Historical)

v1.0 of this catalog required `monocle-proto` to export a Phase 1 stub
`pub fn dispatch_envelope(env: &HookEnvelope) -> Result<(), DispatchError>`
with a Phase 1 runtime semantics. That requirement fabricated a Phase 1
code surface — neither `SS-core-types-and-abi.md` nor any other
architecture artifact specifies a Phase 1 dispatcher; PRD v1.25
§BC-PROTO-002 explicitly classifies the runtime test as Phase 4. The
v1.1 reframing splits this VP into a Phase 1 structural contract
(verifiable now without fabricating new code surface) and a Phase 4
runtime-dispatch contract (verifiable when the Phase 4 IPC dispatcher
exists).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test — Phase 1 structural (primary) | cargo test | Bounded — cross-property recap | Cross-property assertion of VP-016 + VP-017 in combination |
| Integration test — Phase 4 runtime (deferred) | cargo test | N/A — Phase 4 deliverable | Warn-and-skip behavior on unknown `schema_version` |
| Fuzz (Phase 4 deferred) | cargo-fuzz | Unbounded `u32` value space | Arbitrary `u32` `schema_version` values; assert no-panic + warn-and-skip |

## Mechanism

- **Phase 1:** integration-test (structural — cross-property recap of
  VP-016 + VP-017; the structural recap is discharged by the VP-016 +
  VP-017 cargo-integration harnesses at
  `monocle-proto/tests/wire_field_order.rs` and
  `monocle-proto/tests/schema_version.rs` per PRD v1.25 §7 RTM Test Type
  column `Integration` for BC-PROTO-002). The Phase 1 cross-property
  recap test (per BC-2.02.008 §Phase 1 verification) MAY live in
  `monocle-proto/tests/forward_compat_recap.rs` to make the cross-property
  dependency greppable; the structural assertions delegate to VP-016 +
  VP-017 harness invocations.
- **Phase 4 (deferred):** integration-test (runtime warn-and-skip
  behavior; will live in the Phase 4 `monocle-ipc/tests/` crate) + fuzz
  (auxiliary — arbitrary `u32` value space for `schema_version`).

## Pre-conditions

**Phase 1:**

- `monocle-proto` builds cleanly.
- `prost-build` emits a Rust struct for `HookEnvelope` with
  `pub schema_version: u32` (verified by VP-017).
- The compiled proto descriptor has `schema_version` at field number
  `1` (verified by VP-016).

**Phase 4 (deferred):**

- `monocle-ipc` crate exists with its dispatcher (Phase 4 deliverable).
- `tracing 0.1` is the project pin.

## Post-conditions

**Phase 1:**

1. The cross-property recap test instantiates a `HookEnvelope {
   schema_version: 1, event: <any oneof variant> }` and asserts
   `envelope.schema_version == 1` (cross-link to VP-017).
2. The same test inspects the FileDescriptorSet emitted by `build.rs`
   and asserts field number 1 is named `schema_version` (cross-link to
   VP-016). The test fails CLOSED if either underlying property is
   regressed — i.e., if VP-016 or VP-017 would fail, this structural
   recap also fails.
3. The test file is empty of any Phase 1 dispatcher invocation. It does
   NOT import a `dispatch_envelope` function (none is mandated).

**Phase 4 (deferred):**

1. The Phase 4 dispatcher accepts any `u32` value for `schema_version`
   without panic.
2. For unknown values (≠ 1), the dispatcher emits `tracing::warn!` with
   structured field `schema_version = <unknown_value>` and returns
   success (skip).
3. For known value `1`, the dispatcher processes the envelope normally.

## Counter-examples

**Phase 1:**

1. `schema_version` field renumbered to `2` (proto-tag change) — fails
   the field-number assertion (cross-property regression detected here
   even if VP-016's primary harness was disabled).
2. `schema_version` removed from the Rust struct (e.g., made private) —
   fails the Rust-surface assertion (cross-property regression).

**Phase 4 (deferred):**

1. Phase 4 dispatcher panics on unknown version.
2. Phase 4 dispatcher propagates an error to the caller instead of
   logging + skipping.
3. Phase 4 dispatcher silently accepts unknown versions without emitting
   a `tracing::warn!` event (the "silent acceptance" regression).

## Fuzz Harness (Phase 4 deferred)

When Phase 4 lands, a `cargo fuzz add fuzz_envelope_dispatch` target will
exercise arbitrary `u32` `schema_version` values and assert the no-panic
+ warn-and-skip behavior. This harness is NOT a Phase 1 deliverable.

Mutation-test rationale (per monolithic §Coverage Matrix): unknown
schema-version dispatch must never panic across `u32::MAX` value space;
Phase 4 harness only.

## Probe Matrix

| Probe | Phase | Setup | Expected outcome |
|-------|-------|-------|------------------|
| 18.a | 1 | Construct `HookEnvelope { schema_version: 1, event: <any oneof> }` | `envelope.schema_version == 1` (cross-VP-017) |
| 18.b | 1 | FileDescriptorSet inspection: field 1 name | "schema_version" (cross-VP-016) |
| 18.c | 1 | Test file does NOT import `dispatch_envelope` | Source-grep absence assertion |
| 18.d | 4 (deferred) | Dispatcher called with `schema_version: 0` | `tracing::warn!` emitted; no panic; returns success |
| 18.e | 4 (deferred) | Dispatcher called with `schema_version: u32::MAX` | `tracing::warn!` emitted; no panic; returns success |
| 18.f | 4 (deferred) | Dispatcher called with `schema_version: 1` | Envelope processed normally; no warn |
| 18.g | 4 (deferred fuzz) | Arbitrary `u32` value space | No panic; no propagated error for unknown values |

## Open Gap Reference

§G-3 catalogues the Phase 4 federation auth as out-of-Phase-1 scope; the
same out-of-scope boundary applies to the Phase 4 runtime dispatch
behavior of this VP. §G-3 is the future-attachment anchor for both
items. This catalog will be extended in a Phase 4 v2.0 revision with a
`VP-IPC-DISPATCH-001` (or similar) entry to author the runtime
mechanical property against the Phase 4 dispatcher.

## Harness Location

- Phase 1: No dedicated Phase 1 harness — the structural recap is
  discharged by VP-016's `monocle-proto/tests/wire_field_order.rs` and
  VP-017's `monocle-proto/tests/schema_version.rs`. An optional
  cross-property recap test MAY live in
  `monocle-proto/tests/forward_compat_recap.rs` for greppability. Per
  PRD v1.16 §Section 7 RTM, BC-PROTO-002 has no Phase 1 test file path.
- Phase 4 (deferred): `monocle-ipc/tests/envelope_dispatch.rs` (test
  file will be authored against `monocle-ipc/tests/...` when that crate
  exists).
- Test name: No Phase 1 test name — BC-PROTO-002 is Phase 4-deferred
  per PRD v1.25 §BC-PROTO-002 (Phase 4 test name
  `test_BC_PROTO_002_schema_version_validation_skip_unknown` documented
  in PRD v1.25 §BC-PROTO-002 Verification subsection for Phase 4
  implementation only; to be migrated to
  `test_BC_2_02_008_schema_version_validation_skip_unknown` post BC
  renumber propagation into Phase 4 source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-PROTO-002 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.008.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §Prost Wire Schemas.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.008 (Dispatch 4 commit 1030c65).
- Phase 4 future-attachment anchor: §G-3 (Phase 4 federation auth +
  runtime dispatch boundary).
- Cross-VP: VP-016 (wire surface — proto-tag-1 = `schema_version`); VP-017
  (Rust surface — `pub schema_version: u32`).
