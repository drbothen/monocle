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
source_bc: BC-2.02.002
module: monocle-core
proof_method: compile-time-check
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

# VP-012: `monocle_core::MONOCLE_ABI_VERSION` Pub Const Equals `1`

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-ABI-002 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

1. `monocle_core::MONOCLE_ABI_VERSION` is publicly accessible at the crate
   root (no `pub use` from a private module that fails to re-export).
2. Its type is `u32`.
3. Its value is `1`.
4. The constant is usable in const contexts — i.e.,
   `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1);` compiles.

## Source Contract

- **BC:** BC-2.02.002 — ABI Version Constant (Crate Root Surface).
- **Postcondition/Invariant:** BC-2.02.002 invariants 1-4 asserting public
  accessibility at crate root, `u32` type, value `1`, and const-context
  usability (the compile-time `const _:` assert form).
- **Traces to (historical):** BC-ABI-002 (SS-core-types-and-abi.md
  §ABI Version Constant; PRD v1.25 §BC-ABI-002 Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Compile-time check (primary) | rustc / cargo check --tests | N/A — static | `const _: () = assert!(...)` build-gate probe; type-pinning let-binding |
| Runtime assertion (auxiliary) | cargo test | Bounded — single value | `assert_eq!(MONOCLE_ABI_VERSION, 1u32)` integer + type equality |

## Mechanism

Compile-time check (compile-time `const _: () = assert!(...)` assertion in
`monocle-core/tests/abi_stability.rs`; this is a build-gate probe rather
than a runtime test — `cargo check --tests` is the verification driver;
PRD v1.25 §7 RTM Test Type column labels this BC `Lint/compile`). A
complementary runtime `assert_eq!` provides defense-in-depth for the
value assertion if the const-context assert is ever removed during
refactoring.

## Pre-conditions

- `monocle-core` is the project pinned crate.
- `cargo check --tests` is the verification driver.

## Post-conditions

1. The `tests/abi_stability.rs` file contains `const _: () =
   assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "ABI version drift");`
   and compiles cleanly.
2. A runtime assertion `assert_eq!(monocle_core::MONOCLE_ABI_VERSION, 1u32);`
   passes.
3. The type assertion `let _: u32 = monocle_core::MONOCLE_ABI_VERSION;`
   compiles (catches accidental promotion to `u64` or demotion to `u8`).

## Counter-examples

1. `MONOCLE_ABI_VERSION` re-typed as `u64` — fails the type-pinning let-binding.
2. `MONOCLE_ABI_VERSION` defined as `pub static` instead of `pub const` —
   fails the const-context assertion (statics cannot be used in `const _:`
   blocks).
3. `MONOCLE_ABI_VERSION` moved into a private module without `pub use` —
   fails to compile because `monocle_core::MONOCLE_ABI_VERSION` is
   unresolved.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 12.a | `const _: () = assert!(MONOCLE_ABI_VERSION == 1, "...");` in test file | compiles cleanly |
| 12.b | `assert_eq!(MONOCLE_ABI_VERSION, 1u32);` at runtime | passes |
| 12.c | `let _: u32 = MONOCLE_ABI_VERSION;` type-pinning | compiles cleanly |
| 12.d | Mutation: type changed to `u64` | compile error on probe 12.c |
| 12.e | Mutation: declared `pub static` | compile error on probe 12.a |
| 12.f | Mutation: moved to private module without `pub use` | compile error on `monocle_core::MONOCLE_ABI_VERSION` path |

## Harness Location

- `monocle-core/tests/abi_stability.rs` (compile-time-check)
- Test name: `test_BC_ABI_002_abi_version_const_exported` (per PRD v1.25
  §BC-ABI-002, Verification subsection — to be migrated to
  `test_BC_2_02_002_abi_version_const_exported` post BC renumber
  propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-ABI-002 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.002.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §ABI Version Constant.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.002 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-VP: VP-011 (runtime `/status` endpoint surface).
