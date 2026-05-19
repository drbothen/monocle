---
document_type: story
story_id: S-012
epic_id: EPIC-02
version: "1.3"
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
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.11"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.004.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.005.md, version: "1.0.2"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-014-factory-adapter-trait.md, version: "1.0.13"}
  - {path: .factory/specs/verification-properties/vp-015-vsdd-factory-adapter.md, version: "1.0.12"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.10"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.02.004 (FactoryAdapter Trait Definition), BC-2.02.005 (VsddFactoryAdapter Implementation); verifies VP-014, VP-015; covers EC-018, EC-019, EC-020; addresses E-FACT-001, E-FACT-002."
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

### AC-004 (traces to BC-2.02.004 postcondition 4 — FactoryState canonical 7 fields; NO raw_frontmatter)
`FactoryState` declares exactly these 7 fields as defined in SS-core-types-and-abi.md §FactoryState
(lines 364–400):
1. `phase: String` — pipeline phase identifier
2. `status: String` — workflow status
3. `awaiting: Option<String>` — what the orchestrator is waiting on (populated from `awaiting:` frontmatter key)
4. `blocking_issues: Vec<BlockingIssue>` — structured blocking issues
5. `convergence: Option<ConvergenceMetrics>` — convergence round count
6. `cycle: Option<String>` — current cycle identifier
7. `custom_fields: std::collections::HashMap<String, serde_yaml_ng::Value>` — forward-compatibility escape hatch

`raw_frontmatter` is FORBIDDEN — it is not in the canonical 7-field list and violates
SS-core-types-and-abi.md §FactoryState (it is also a BC INV-2 red-line).
`custom_fields` uses `serde_yaml_ng::Value` (NOT `serde_json::Value`).
`awaiting` is `Option<String>` — `None` means legitimate absence, NOT unknown.
Consumers display `"—"` for `None` fields.

### AC-005 (traces to BC-2.02.005 postcondition 1 — VsddFactoryAdapter detect)
`VsddFactoryAdapter::detect(workspace_root)` returns `Some(FactoryDetection { ... })` if
and only if `workspace_root/.factory/STATE.md` exists and its first YAML frontmatter line
contains `document_type: pipeline-state`. Returns `None` otherwise.

### AC-006 (traces to BC-2.02.005 postcondition 2 — self-referential detection)
When the monocle binary is run in a VSDD factory project (e.g., the monocle repo itself),
`VsddFactoryAdapter::detect(monocle_repo_root)` returns `Some(...)`. Integration test
verifies detection on the monocle project's own `.factory/STATE.md`.

### AC-007 (traces to BC-2.02.005 invariant 3 — subscribe() Phase 1 stub)
`VsddFactoryAdapter::subscribe()` returns `Ok(Box::pin(futures::stream::empty()))`.
No file watcher is instantiated in Phase 1.
(BC-2.02.005 invariant 3 is the canonical locus: "subscribe() returns
Ok(Box::pin(futures::stream::empty())) in Phase 1".)

### AC-008 (traces to BC-2.02.005 postcondition 4 — error handling)
`VsddFactoryAdapter::read_state()` returns:
- `Err(FactoryReadError::NotFound)` if STATE.md does not exist → logs E-FACT-001
- `Err(FactoryReadError::ParseError)` if STATE.md cannot be parsed → logs E-FACT-002
- `Ok(FactoryState)` on success

### AC-009 (traces to BC-2.02.005 invariant 3 — subscribe() Phase 1 invariant; test verification)
Integration test verifies that `subscribe()` returns `Ok(Box::pin(empty()))` and that
polling the returned stream returns `None` immediately (empty stream).
(BC-2.02.005 invariant 3 is the canonical locus for the Phase 1 subscribe() stub constraint.)

### AC-010 (traces to BC-2.02.005 invariant 2 — display_name() returns "VSDD Factory")
`VsddFactoryAdapter::display_name()` returns the exact string `"VSDD Factory"` — the string
used in TUI display and in the self-referential detection test vector (BC-2.02.005 invariant 2
verbatim: `display_name() returns "VSDD Factory"`). Unit test asserts:
`assert_eq!(adapter.display_name(), "VSDD Factory");`

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
- [ ] Define `FactoryState` with exactly 7 canonical fields per SS-core-types-and-abi.md §FactoryState:
  `phase`, `status`, `awaiting`, `blocking_issues`, `convergence`, `cycle`, `custom_fields`
  - `awaiting: Option<String>` — MANDATORY; populated from `awaiting:` frontmatter key
  - `custom_fields: HashMap<String, serde_yaml_ng::Value>` — NOT serde_json::Value
  - `convergence: Option<ConvergenceMetrics>`; `cycle: Option<String>`
  - `raw_frontmatter` MUST NOT be added — it is a red-line forbidden field per SS-core-types-and-abi.md
- [ ] Define `FactoryReadError`, `FactorySubscribeError`, `StateChangeStream` type alias
- [ ] Create `monocle-core/src/factory/vsdd.rs` with `VsddFactoryAdapter` implementation
  - `detect()`: check for `.factory/STATE.md` + frontmatter `document_type: pipeline-state`
  - `read_state()`: parse YAML frontmatter via `serde_yaml_ng`; return typed `FactoryState`
  - `subscribe()`: return `Ok(Box::pin(futures::stream::empty()))`
- [ ] Create `monocle-core/tests/factory_adapter_surface.rs` (VP-014 AST audit):
  - syn 2 parse `factory.rs`: assert exactly 7 methods, no `Sealed`, `Send + Sync + 'static`
- [ ] Implement `VsddFactoryAdapter::display_name()` returning the exact string literal `"VSDD Factory"` (BC-2.02.005 INV-2)
- [ ] Create `monocle-core/tests/factory_self_referential.rs` (VP-015 integration):
  - `VsddFactoryAdapter::detect(monocle_repo_root)` → Some
  - `read_state()` → `Ok(FactoryState)` on real `.factory/STATE.md`
  - `subscribe()` → stream is empty
  - `display_name()` → `"VSDD Factory"` (exact string, AC-010)

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
