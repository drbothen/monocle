---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T12:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
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

# Behavioral Contract BC-2.02.007: HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface)

## Description

The prost-build-generated `HookEnvelope` Rust struct exposes `pub schema_version: u32`.
For all Phase 1-origin messages this field has value `1`. The generated struct field order
is an implementation detail of prost-build and is NOT contractual. Only the proto field
number (BC-2.02.006) and the Rust field accessibility are contractual. `prost 0.14` is
pinned exactly in `monocle-proto` to prevent silent behavior changes from altering the
generated Rust API between builds.

## Preconditions

1. `monocle-proto` crate compiles (prost-build generates Rust types from the `.proto` file).

## Postconditions

1. The prost-build-generated `HookEnvelope` Rust struct exposes `pub schema_version: u32`.
2. For all Phase 1-origin messages, this field has value `1`.
3. The generated struct field order is an implementation detail of prost-build and is NOT a behavioral contract. Only the proto field number (BC-2.02.006) and the Rust field accessibility (`pub schema_version: u32`) are contractual.

## Invariants

1. `monocle-proto` declares `prost 0.14` with an EXACT version pin (per SS-deps-pin-manifest.md). The exact pin prevents silent prost-build behavior changes from altering the generated Rust API between builds.
2. The `build.rs` in `monocle-proto` generates Rust types but activates no wire path in Phase 1 — the types are compiled into the binary and available for Phase 4 without workspace changes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-026 | prost-build version change (if exact pin is relaxed in a future PR) | The `pub schema_version: u32` field accessibility must be re-verified after any prost-build version change; exact pin prevents this in normal operation |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Construct `HookEnvelope { schema_version: 1, ... }` in Rust | Compiles; `envelope.schema_version == 1` | happy-path |
| Access `HookEnvelope::schema_version` from `monocle-runtime` | Field is accessible (`pub`) without re-import | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-017 | `HookEnvelope` generated struct has `pub schema_version: u32` accessible from dependent crates; constructing with `schema_version: 1` compiles and asserts correctly | compile/integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability §SS-02 |
| Capability Anchor Justification | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability — this BC governs the Rust API surface of the wire format type, ensuring downstream Rust crates can access the schema_version field as part of the stable ABI contract |
| L2 Domain Invariants | N/A — no domain-spec/invariants.md exists; CAP-002 per ARCH-INDEX is authoritative source |
| Architecture Module | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) per ARCH-INDEX Subsystem Registry SS-02 |
| Architecture Source | SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas |
| FC | FC-05 (Rust surface; wire-format covered by BC-2.02.006) |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — prost wire format) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-PROTO-001b |
| Test name | test_BC_PROTO_001b_schema_version_rust_field |

## Related BCs (Recommended)

- [BC-2.02.006] — composes with: the wire-format companion that locks the proto field number
- [BC-2.02.008] — depends on: Phase 4 validation reads this Rust field to check schema version

## Architecture Anchors (Recommended)

- `architecture/SS-core-types-and-abi.md#prost-wire-schemas` — prost exact pin policy, build.rs spec, Rust surface vs wire-format distinction

## Story Anchor (Recommended)

S-TBD — Implement monocle-proto crate with prost-build type generation (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-017-hook-envelope-rust-surface.md` — VP-017 Rust struct field accessibility compile/integration test
