---
document_type: story
level: L4
story_id: S-010
epic_id: EPIC-02
version: "1.3"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001, S-003]
blocks: [S-011, S-012, S-013, S-014]
target_module: monocle-core
subsystems: [SS-02]
behavioral_contracts: [BC-2.02.001, BC-2.02.002]
verification_properties: [VP-011, VP-012]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.001.md, version: "1.0.2"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.002.md, version: "1.0.3"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-011-abi-version-status-endpoint.md, version: "1.0.13"}
  - {path: .factory/specs/verification-properties/vp-012-abi-version-crate-root.md, version: "1.0.13"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/architecture/SS-forward-compatibility.md, version: "1.2.19"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.02.001 (ABI Version in /status), BC-2.02.002 (ABI Version Constant at Crate Root); verifies VP-011, VP-012."
---

# S-010: Populate monocle-core ABI Version Constant (FC-03)

## Narrative

As an implementer of downstream monocle crates, I want `monocle-core` to export
`MONOCLE_ABI_VERSION: u32 = 1` at the crate root and expose it via `/status`, so that
Phase 3 plugin SDK and Phase 4 federation layer can compile-time-assert compatibility
without coupling to internal implementation details.

## Acceptance Criteria

### AC-001 (traces to BC-2.02.002 postcondition 1 — ABI constant at crate root)
`monocle-core` exports `pub const MONOCLE_ABI_VERSION: u32 = 1` at the crate root
(`monocle_core::MONOCLE_ABI_VERSION`). Callers can compile-time-assert:
`const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "ABI version mismatch");`
(§ABI Version Constant SS-core-types-and-abi.md v1.2.13 lines 40-101)

### AC-002 (traces to BC-2.02.002 postcondition 2 — crate root re-export)
`MONOCLE_ABI_VERSION` is re-exported via `pub use abi::MONOCLE_ABI_VERSION` in
`monocle-core/src/lib.rs` from `monocle-core/src/abi.rs`. Callers need not qualify
with `monocle_core::abi::MONOCLE_ABI_VERSION`.
(§ABI Version Constant SS-core-types-and-abi.md v1.2.13 lines 40-101)

### AC-003 (traces to BC-2.02.001 postcondition 1 — ABI version in /status)
The `/status` endpoint response includes `"abi_version": 1`. This field reads from
`monocle_core::MONOCLE_ABI_VERSION` at runtime — NOT hardcoded in `monocle-runtime`.
Integration test: `GET /status | jq .abi_version == 1`.
Sub-clause: The `/status` integration test harness is reused from S-003 §AC-005; this
story imports `monocle_core::MONOCLE_ABI_VERSION` in
`monocle-runtime/src/handlers/status.rs` (created by S-003) to replace any hardcoded
literal. The dependency direction is: `monocle-runtime` → `monocle-core`.

### AC-004 (traces to BC-2.02.002 postcondition 3 — compile-time stability test)
The file `monocle-core/tests/abi_stability.rs` contains a top-level
`const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "ABI version mismatch");`
(file-scope, not inside `#[test] fn`). This is a compile-fail assertion — `cargo build
--tests` fails if the constant changes without updating the assertion.

### AC-005 (traces to BC-2.02.001 postcondition 1 + postcondition 2 — const value is 1 in Phase 1; equals compiled value)
`MONOCLE_ABI_VERSION` equals `1` in Phase 1. Any change to this value requires an ADR
(is a breaking change per SS-core-types-and-abi.md §ABI Version Constant v1.2.13 lines 40-101;
FC-03 per SS-forward-compatibility.md §FC-03).

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~700 |
| BC-2.02.001.md | ~500 |
| BC-2.02.002.md | ~500 |
| VP-011 + VP-012 files | ~800 |
| SS-core-types-and-abi.md (ABI section, ~50 lines) | ~800 |
| Test file | ~400 |
| **Total estimate** | **~3,700** |

## Tasks

- [ ] Create `monocle-core/src/abi.rs` with `pub const MONOCLE_ABI_VERSION: u32 = 1;` and
  rustdoc copy-equivalent to SS-core-types-and-abi.md v1.2.13 lines 47-53
- [ ] Add `pub use abi::MONOCLE_ABI_VERSION;` in `monocle-core/src/lib.rs`
- [ ] Create `monocle-core/tests/abi_stability.rs` with `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "ABI version mismatch");`
- [ ] Update `monocle-runtime/src/handlers/status.rs` to read `abi_version` from `monocle_core::MONOCLE_ABI_VERSION` (NOT hardcoded)
- [ ] Add `monocle-core` as dependency of `monocle-runtime` in `monocle-runtime/Cargo.toml`
- [ ] Integration test: verify `GET /status` body `.abi_version == 1`

## Previous Story Intelligence

S-001 (Wave 1): `monocle-core/src/lib.rs` stub exists.
S-003 (Wave 2): `/status` handler exists; update it to import `monocle_core::MONOCLE_ABI_VERSION`.
The dependency direction is: `monocle-runtime` → `monocle-core` (correct; never reverse).

## Architecture Compliance Rules

From `architecture/SS-core-types-and-abi.md` v1.2.13 §ABI Version Constant:
- `MONOCLE_ABI_VERSION` lives in `monocle-core/src/abi.rs`, re-exported from `lib.rs`
- Any modification to this constant requires an ADR
- `monocle-runtime` reads it; does not redefine it

**Forbidden Dependencies:**
- `monocle-core` MUST NOT depend on `monocle-runtime` (cycle)
- `monocle-core` MUST NOT depend on `monocle-tui` (cycle)
- `MONOCLE_ABI_VERSION` MUST NOT be hardcoded in `monocle-runtime` — must import from `monocle-core`

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| (no external deps for abi.rs) | — | Const declaration only |
| monocle-core (intra-workspace) | path = "../monocle-core" | monocle-runtime consumes the ABI constant |

## File Structure Requirements

Files to create:
- `monocle-core/src/abi.rs` — ABI version constant with rustdoc
- `monocle-core/tests/abi_stability.rs` — compile-time assertion test

Files to modify:
- `monocle-core/src/lib.rs` — `pub mod abi; pub use abi::MONOCLE_ABI_VERSION;`
- `monocle-runtime/src/handlers/status.rs` — import `monocle_core::MONOCLE_ABI_VERSION`
- `monocle-runtime/Cargo.toml` — add `monocle-core = { path = "../monocle-core" }`

## §Trace v1.2

**Phase 3.B Batch 2 — spec-reviewer remediation** (2026-05-20):
- F-D-01 [LOW]: Title rephrased to "Populate monocle-core ABI Version Constant (FC-03)"
  (S-001 creates the crate skeleton; S-010 only populates abi.rs).
- F-A-01 [LOW]: Library & Framework Requirements — intra-workspace dep row added for
  `monocle-core (path = "../monocle-core")` consumed by monocle-runtime.
- F-B-01 [LOW]: §-anchor appended to AC-001, AC-002, AC-005 pointing to
  SS-core-types-and-abi.md v1.2.13 lines 40-101.
- F-B-02 [LOW]: FC-03 anchored to SS-forward-compatibility.md §FC-03 in AC-005.
- F-C-01 + F-E-01 [MED]: AC-003 sub-clause added — /status integration test harness
  reused from S-003 §AC-005; this story imports MONOCLE_ABI_VERSION in status.rs created
  by S-003. S-003 added to depends_on.
- F-C-02 [MED]: AC-004 compile-fail mechanism reworded — file-scope `const _: ()` assertion
  specified (not inside `#[test] fn`); `cargo build --tests` fails if constant changes.
- F-D-03 [LOW]: Tasks updated — rustdoc cross-referenced to SS-core-types-and-abi.md v1.2.13 lines 47-53.
## §Trace 1.3 — POL-11 cascade remediation (2026-05-30)

**Bump:** 1.2 → 1.3.
**Scope:** AC-005 body: `SS-forward-compatibility.md v1.2.19 §FC-03` → `SS-forward-compatibility.md §FC-03` (Option 2 version-free; cascade from SS-forward-compatibility v1.2.19 → v1.2.20 bump in same remediation burst; version-free permanently prevents re-staling).
**SE-16d PASS:** 2026-05-30 >= prior date (cascade patch).
