---
document_type: behavioral-contract
level: L3
version: "1.0.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T05:11:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "b647859"
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

# Behavioral Contract BC-2.02.002: ABI Version Constant at Crate Root (FC-03)

## Description

The `monocle-core` crate exports `pub const MONOCLE_ABI_VERSION: u32 = 1;` at
`monocle_core::MONOCLE_ABI_VERSION`. This compile-time constant is the canonical ABI
version signal for downstream crates, the plugin SDK (Phase 3), and the federation IPC
layer (Phase 4). Its type is `u32` for parity with the proto `uint32` field. Any change
to the constant value requires an ADR.

## Preconditions

1. The `monocle-core` crate compiles successfully.

## Postconditions

1. `monocle-core` exports `pub const MONOCLE_ABI_VERSION: u32 = 1;` accessible at the crate root as `monocle_core::MONOCLE_ABI_VERSION`.
2. The declaration is in `monocle-core/src/abi.rs` and re-exported from `monocle-core/src/lib.rs` via `pub use abi::MONOCLE_ABI_VERSION;`.
3. Downstream crates can write compile-time assertions against the constant:
   ```rust
   const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1,
       "ABI version mismatch — check monocle-core version");
   ```

## Invariants

1. The constant is `u32`, not `u8`, `u16`, or `usize` — chosen for proto field parity (proto `uint32`).
2. The constant is `pub` (not `pub(crate)`) — must be accessible from `monocle-plugin-sdk` (Phase 3) and `monocle-ipc` (Phase 4).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-015 | Compile-time assertion in `monocle-plugin-sdk`. If `MONOCLE_ABI_VERSION` is changed without updating the plugin SDK's compile-time assertion | SDK fails to compile with a clear error message — this is the intended behavior |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `monocle_core::MONOCLE_ABI_VERSION` | Compile-time value `1u32` | happy-path |
| `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "...");` | Compiles without error | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-012 | `MONOCLE_ABI_VERSION` is `1u32`, `pub`, at the crate root of `monocle-core`, declared in `abi.rs` and re-exported from `lib.rs` | lint/compile |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability §SS-02 |
| Capability Anchor Justification | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability — this BC defines the compile-time ABI constant that is the cornerstone of the forward-compatible ABI contract |
| L2 Domain Invariants | DI-004 (all public wire types must carry a version discriminant as their first field — MONOCLE_ABI_VERSION is the compile-time source of the version discriminant constant that all ABI-carrying public wire types reference; it is the single source of truth whose value propagates to every public surface that carries DI-004's version discriminant requirement) |
| Architecture Module | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) per ARCH-INDEX Subsystem Registry SS-02 |
| Architecture Source | SS-core-types-and-abi.md v1.2.13 §ABI Version Constant |
| FC | FC-03 |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — ABI version constant) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-ABI-002 |
| Test name | test_BC_ABI_002_abi_version_const_exported |

## Related BCs (Recommended)

- [BC-2.02.001] — composes with: the /status endpoint that exposes this constant at runtime
- [BC-2.02.006] — composes with: the proto field that carries this value on the wire

## Architecture Anchors (Recommended)

- `architecture/SS-core-types-and-abi.md#abi-version-constant` — ABI version constant definition, type rationale, and re-export spec

## Story Anchor (Recommended)

S-TBD — Implement ABI version constant in monocle-core (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-012-abi-version-crate-root.md` — VP-012 ABI version crate root lint/compile test

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-002 per ARCH-INDEX is authoritative source`
  - After: `DI-004 ...`
  - DI-004 mapping: MONOCLE_ABI_VERSION is the canonical numeric constant that all public wire types use as their version discriminant value. This BC establishes the single source of truth for that value.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.2.8 → v1.2.13** (2026-05-18T05:11:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.2.8 → v1.2.13); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-core-types-and-abi.md v1.2.8 §ABI Version Constant`
  - SE-17f AFTER: `SS-core-types-and-abi.md v1.2.13 §ABI Version Constant`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:11:00Z > prior 2026-05-17T18:00:00Z (v1.0.2). ARITHMETICALLY TRUE: 2026-05-18T05:11:00Z > 2026-05-17T18:00:00Z PASS.
