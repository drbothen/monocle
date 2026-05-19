---
document_type: story
story_id: S-012
epic_id: EPIC-02
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 8
wave: 3
tdd_mode: strict
priority: P0
depends_on: [S-010, S-011]
blocks: []
target_module: monocle-core
subsystems: [SS-02]
behavioral_contracts: [BC-2.02.004, BC-2.02.005]
verification_properties: [VP-014, VP-015]
estimated_days: 3
---

# S-012: FactoryAdapter Trait + VsddFactoryAdapter Implementation (FC-04)

## Narrative

As a monocle plugin developer, I want a stable open `FactoryAdapter` trait in `monocle-core`
with exactly 7 methods and no sealed bound, so that `VsddFactoryAdapter` ships in Phase 1
and WASM plugin adapters can implement the trait in Phase 3 without breaking changes.

## Acceptance Criteria

### AC-001 (traces to BC-2.02.004 postcondition 1 — 7 methods exact)
`FactoryAdapter` trait in `monocle-core::factory` defines exactly these 7 methods:
`detect`, `matches`, `state_file_path`, `read_state`, `subscribe`, `display_name`, `abi_version`.
`detect` has `where Self: Sized` bound. `abi_version` has a default impl returning
`crate::MONOCLE_ABI_VERSION`.

### AC-002 (traces to BC-2.02.004 postcondition 2 — no Sealed bound)
`FactoryAdapter` has NO `private::Sealed` supertrait. Supertrait bounds are `Send + Sync + 'static`
ONLY. AST audit via VP-014 verifies zero `Sealed` references in the trait definition.

### AC-003 (traces to BC-2.02.004 postcondition 3 — supporting types co-located)
`monocle-core::factory` module declares: `FactoryDetection` (3 fields), `FactoryState` (7 fields),
`BlockingIssue`, `BlockingSeverity`, `ConvergenceMetrics`, `FactoryReadError`,
`FactorySubscribeError`, `StateChangeStream` type alias.

### AC-004 (traces to BC-2.02.004 postcondition 4 — FactoryState field types)
`FactoryState` uses `serde_yaml_ng::Value` for `custom_fields` (NOT `serde_json::Value`).
`convergence` is `Option<ConvergenceMetrics>`. `cycle` is `Option<String>`.
`None` means legitimate absence — NOT unknown. Consumers display `"—"` for `None`.

### AC-005 (traces to BC-2.02.005 postcondition 1 — VsddFactoryAdapter detect)
`VsddFactoryAdapter::detect(workspace_root)` returns `Some(FactoryDetection { ... })` if
and only if `workspace_root/.factory/STATE.md` exists and its first YAML frontmatter line
contains `document_type: pipeline-state`. Returns `None` otherwise.

### AC-006 (traces to BC-2.02.005 postcondition 2 — self-referential detection)
When the monocle binary is run in a VSDD factory project (e.g., the monocle repo itself),
`VsddFactoryAdapter::detect(monocle_repo_root)` returns `Some(...)`. Integration test
verifies detection on the monocle project's own `.factory/STATE.md`.

### AC-007 (traces to BC-2.02.005 postcondition 3 — subscribe() Phase 1 stub)
`VsddFactoryAdapter::subscribe()` returns `Ok(Box::pin(futures::stream::empty()))`.
No file watcher is instantiated in Phase 1.

### AC-008 (traces to BC-2.02.005 postcondition 4 — error handling)
`VsddFactoryAdapter::read_state()` returns:
- `Err(FactoryReadError::NotFound)` if STATE.md does not exist → logs E-FACT-001
- `Err(FactoryReadError::ParseError)` if STATE.md cannot be parsed → logs E-FACT-002
- `Ok(FactoryState)` on success

### AC-009 (traces to BC-2.02.004 invariant 3 — subscribe() Phase 1 invariant)
Integration test verifies that `subscribe()` returns `Ok(Box::pin(empty()))` and that
polling the returned stream returns `None` immediately (empty stream).

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,000 |
| BC-2.02.004.md | ~700 |
| BC-2.02.005.md | ~700 |
| VP-014 + VP-015 files | ~1,000 |
| SS-core-types-and-abi.md (FactoryAdapter section, ~100 lines) | ~1,500 |
| serde_yaml_ng + futures crate patterns | ~400 |
| Test files | ~900 |
| **Total estimate** | **~6,200** |

## Tasks

- [ ] Create `monocle-core/src/factory.rs` with `FactoryAdapter` trait (7 methods exact)
- [ ] Define `FactoryDetection` (3 fields: `display_name`, `state_file_path`, `framework_version`)
- [ ] Define `FactoryState` (7 fields: `phase`, `cycle`, `status`, `blocking_issues`, `convergence`, `custom_fields`, `raw_frontmatter`)
  - `custom_fields: serde_yaml_ng::Value` — NOT serde_json::Value
  - `convergence: Option<ConvergenceMetrics>`; `cycle: Option<String>`
- [ ] Define `FactoryReadError`, `FactorySubscribeError`, `StateChangeStream` type alias
- [ ] Create `monocle-core/src/factory/vsdd.rs` with `VsddFactoryAdapter` implementation
  - `detect()`: check for `.factory/STATE.md` + frontmatter `document_type: pipeline-state`
  - `read_state()`: parse YAML frontmatter via `serde_yaml_ng`; return typed `FactoryState`
  - `subscribe()`: return `Ok(Box::pin(futures::stream::empty()))`
- [ ] Create `monocle-core/tests/factory_adapter_surface.rs` (VP-014 AST audit):
  - syn 2 parse `factory.rs`: assert exactly 7 methods, no `Sealed`, `Send + Sync + 'static`
- [ ] Create `monocle-core/tests/factory_self_referential.rs` (VP-015 integration):
  - `VsddFactoryAdapter::detect(monocle_repo_root)` → Some
  - `read_state()` → `Ok(FactoryState)` on real `.factory/STATE.md`
  - `subscribe()` → stream is empty

## Previous Story Intelligence

S-010 (Wave 2): `monocle-core` crate structure established.
S-011 (Wave 2): `#[non_exhaustive]` on `BlockingSeverity` already applied.
`futures 0.3` and `serde_yaml_ng 0.10` pinned in workspace.

## Architecture Compliance Rules

From `architecture/SS-core-types-and-abi.md` v1.2.13 §FactoryAdapter Trait:
- 7 methods exactly — any addition is a BREAKING change requiring an ADR
- NO `Sealed` supertrait — trait is OPEN for WASM plugin extensions
- `FactoryState.custom_fields` MUST use `serde_yaml_ng::Value` — serde_json::Value is FORBIDDEN
- Phase 1 `subscribe()` MUST return `Ok(Box::pin(futures::stream::empty()))` — no file watcher

**Forbidden Dependencies:**
- `FactoryAdapter` MUST NOT have `private::Sealed` supertrait
- `FactoryState.custom_fields` MUST NOT use `serde_json::Value`
- `VsddFactoryAdapter` MUST NOT write to `.factory/STATE.md` (observe-only; DI-007)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| serde_yaml_ng | 0.10 | YAML frontmatter parsing (NOT serde_yaml 0.8) |
| futures | 0.3 | `futures::stream::empty()` for Phase 1 subscribe stub |
| serde | 1 | Deserialize for FactoryState |
| tracing | 0.1 | WARN on E-FACT-001/E-FACT-002 |
| syn | 2 | VP-014 AST audit test |

## File Structure Requirements

Files to create:
- `monocle-core/src/factory.rs` — trait + supporting types
- `monocle-core/src/factory/vsdd.rs` — VsddFactoryAdapter implementation
- `monocle-core/tests/factory_adapter_surface.rs` — VP-014 AST audit
- `monocle-core/tests/factory_self_referential.rs` — VP-015 integration test

Files to modify:
- `monocle-core/src/lib.rs` — add `pub mod factory;`
- `monocle-core/Cargo.toml` — add `serde_yaml_ng = "0.10"`, `futures = "0.3"`
