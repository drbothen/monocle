---
document_type: story
story_id: S-010
epic_id: EPIC-02
version: "1.1"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001]
blocks: [S-011, S-012, S-013, S-014]
target_module: monocle-core
subsystems: [SS-02]
behavioral_contracts: [BC-2.02.001, BC-2.02.002]
verification_properties: [VP-011, VP-012]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.11"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.001.md, version: "1.0.2"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.002.md, version: "1.0.3"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-011-abi-version-status-endpoint.md, version: "1.0.13"}
  - {path: .factory/specs/verification-properties/vp-012-abi-version-crate-root.md, version: "1.0.13"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.10"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/architecture/SS-forward-compatibility.md, version: "1.2.19"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.02.001 (ABI Version in /status), BC-2.02.002 (ABI Version Constant at Crate Root); verifies VP-011, VP-012."
---

# S-010: monocle-core Crate Foundation + ABI Version Constant (FC-03)

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

### AC-002 (traces to BC-2.02.002 postcondition 2 — crate root re-export)
`MONOCLE_ABI_VERSION` is re-exported via `pub use abi::MONOCLE_ABI_VERSION` in
`monocle-core/src/lib.rs` from `monocle-core/src/abi.rs`. Callers need not qualify
with `monocle_core::abi::MONOCLE_ABI_VERSION`.

### AC-003 (traces to BC-2.02.001 postcondition 1 — ABI version in /status)
The `/status` endpoint response includes `"abi_version": 1`. This field reads from
`monocle_core::MONOCLE_ABI_VERSION` at runtime — NOT hardcoded in `monocle-runtime`.
Integration test: `GET /status | jq .abi_version == 1`.

### AC-004 (traces to BC-2.02.002 postcondition 3 — compile-time stability test)
`monocle-core/tests/abi_stability.rs` contains a lint test that asserts `MONOCLE_ABI_VERSION == 1`
via `const` assertion. This test fails at compile time if the constant changes without
updating the assertion.

### AC-005 (traces to BC-2.02.001 postcondition 1 + postcondition 2 — const value is 1 in Phase 1; equals compiled value)
`MONOCLE_ABI_VERSION` equals `1` in Phase 1. Any change to this value requires an ADR
(is a breaking change per SS-core-types-and-abi.md §ABI Version Constant).

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

- [ ] Create `monocle-core/src/abi.rs` with `pub const MONOCLE_ABI_VERSION: u32 = 1;` and rustdoc
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

## File Structure Requirements

Files to create:
- `monocle-core/src/abi.rs` — ABI version constant with rustdoc
- `monocle-core/tests/abi_stability.rs` — compile-time assertion test

Files to modify:
- `monocle-core/src/lib.rs` — `pub mod abi; pub use abi::MONOCLE_ABI_VERSION;`
- `monocle-runtime/src/handlers/status.rs` — import `monocle_core::MONOCLE_ABI_VERSION`
- `monocle-runtime/Cargo.toml` — add `monocle-core = { path = "../monocle-core" }`
