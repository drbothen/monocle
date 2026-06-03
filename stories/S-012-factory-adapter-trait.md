---
document_type: story
level: L4
story_id: S-012
epic_id: EPIC-02
version: "1.5"
status: done
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
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.004.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.005.md, version: "1.0.2"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-014-factory-adapter-trait.md, version: "1.0.13"}
  - {path: .factory/specs/verification-properties/vp-015-vsdd-factory-adapter.md, version: "1.0.12"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
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
`monocle-core::factory` module declares: `FactoryDetection` (3 fields: `display_name: String`,
`workspace_root: PathBuf`, `state_file: PathBuf` per SS-core-types-and-abi.md lines 337-344),
`FactoryState` (7 fields), `BlockingIssue`, `BlockingSeverity`, `ConvergenceMetrics`,
`FactoryReadError`, `FactorySubscribeError`, `StateChangeStream` type alias.

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

**EC-021 negative test vector (BC-2.02.005 INV-1 + EC-021):** STATE.md that contains
`document_type: pipeline-state` ONLY in the document body (NOT in the `---`-delimited YAML
frontmatter block) MUST return `None` from `detect()`. Detection requires the key to appear
in the frontmatter. A file where the frontmatter is absent or does not contain this key
but the body text does is NOT a valid VSDD factory workspace. The reference implementation at
SS-core-types-and-abi.md line 602 uses `content.contains("document_type: pipeline-state")`
which matches body text — this is a known divergence; this story's implementation MUST use
frontmatter-only detection (not body search) per BC-2.02.005 INV-1.

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

### AC-011 (traces to BC-2.02.005 postcondition 1 — VsddFactoryAdapter::new() public constructor)
`VsddFactoryAdapter::new(workspace_root: PathBuf) -> Self` is a public constructor. It derives
`state_file = workspace_root.join(".factory").join("STATE.md")` and performs NO validation at
construction time — validation is deferred to `detect()` and `read_state()`.
(BC-2.02.005 PC-1 verbatim: "No validation is performed at construction time.")
Cite: SS-core-types-and-abi.md line 590 (`pub fn new(workspace_root: PathBuf) -> Self`).

Unit test vectors:
- Construct with absolute path `/tmp/my-project` → `state_file = /tmp/my-project/.factory/STATE.md`
  (no error; no filesystem access at construction time)
- Construct with relative path `some/path` → `state_file = some/path/.factory/STATE.md`
  (no error; no filesystem access at construction time)
- Construct with empty `PathBuf::new()` → `state_file = .factory/STATE.md`
  (no error; empty path treated as current directory join)
- All three constructions succeed without `Result<>` — constructor is infallible; `detect()`
  is where the path is validated against the filesystem

### AC-012 (traces to BC-2.02.005 postcondition 3 — absent optional fields → None, NOT "unknown")
`VsddFactoryAdapter::read_state()` returns `cycle: None` when `current_cycle:` is absent from
STATE.md YAML frontmatter; `convergence: None` when the §Session Resume Checkpoint section is
absent. Consumers MUST NOT receive the string `"unknown"` as a placeholder for absent optional
fields (BC-2.02.005 PC-3 verbatim).

Test vectors:
- STATE.md YAML frontmatter without `current_cycle:` key → `Some(FactoryState { cycle: None, ... })`
- STATE.md YAML frontmatter with `current_cycle: cycle-001` → `Some(FactoryState { cycle: Some("cycle-001".into()), ... })`
- STATE.md without §Session Resume Checkpoint section → `Some(FactoryState { convergence: None, ... })`
- Negative: `cycle: Some("unknown".into())` MUST NOT occur — `parse_frontmatter_field` returns
  `None` for absent keys; `"unknown"` is never injected as a default by the adapter

### AC-013 (traces to BC-2.02.005 postcondition 4 — parse_frontmatter_field guards)
`parse_frontmatter_field` and `parse_frontmatter_extra_fields` apply these four guards in order:
1. Skip continuation lines (lines beginning with whitespace) — multi-line YAML values are not parsed
2. Return `None` for empty values (key present but value empty string) — EC-061
3. Return `None` for flow-style list values beginning with `[` — EC-023
4. Return `None` for block scalar markers beginning with `|` or `>`

Additionally: YAML quoted scalars are unquoted — surrounding single and double quotes are stripped
from values before returning `Some(value)` — EC-022.

Unit test vectors for each guard:
- `current_cycle:   cycle-001` (with leading-space continuation value) → guard 1: continuation
  line skipped; `cycle: None` (BC-2.02.005 PC-4 guard 1)
- `current_cycle: ` (empty value, trailing space) → guard 2: `None` (EC-061)
- `current_cycle: ""` (empty quoted value) → guard 2 after unquoting: `None` (EC-061)
- `blocking_issues: []` → guard 3: `None` (EC-023; `blocking_issues` populated by Phase 3 body parsing)
- `some_field: |` (block scalar marker) → guard 4: `None`
- `some_field: > folded` (folded block scalar) → guard 4: `None`
- `awaiting: "round 18 validation chain"` → unquoted to `Some("round 18 validation chain")` (EC-022)
- `awaiting: 'single-quoted'` → unquoted to `Some("single-quoted")` (EC-022 covers single quotes too)

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,400 |
| BC-2.02.004.md | ~700 |
| BC-2.02.005.md (PC-1, PC-3, PC-4 — 3 new ACs) | ~700 |
| VP-014 + VP-015 files | ~1,000 |
| SS-core-types-and-abi.md (FactoryAdapter section, ~100 lines) | ~1,500 |
| serde_yaml_ng + futures crate patterns | ~400 |
| Test files | ~1,200 |
| **Total estimate** | **~6,900** |

## Tasks

- [ ] Create `monocle-core/src/factory.rs` with `FactoryAdapter` trait (7 methods exact)
- [ ] Define `FactoryDetection` (3 fields: `display_name: String`, `workspace_root: PathBuf`, `state_file: PathBuf`)
  per SS-core-types-and-abi.md lines 337-344 — NOT `state_file_path` or `framework_version` (TDD red-gate-blocker if wrong)
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
  - File MUST contain: `#[test] fn test_BC_FACTORY_001_trait_defined_open_no_sealed_bound() { ... }`
    (BC-2.02.004 line 98 canonical test-function name — verbatim, no variation)
- [ ] Implement `VsddFactoryAdapter::display_name()` returning the exact string literal `"VSDD Factory"` (BC-2.02.005 INV-2)
- [ ] Create `monocle-core/tests/factory_self_referential.rs` (VP-015 integration):
  - `VsddFactoryAdapter::detect(monocle_repo_root)` → Some
  - `read_state()` → `Ok(FactoryState)` on real `.factory/STATE.md`
  - `subscribe()` → stream is empty
  - `display_name()` → `"VSDD Factory"` (exact string, AC-010)
  - File MUST contain: `#[test] fn test_BC_FACTORY_002_vsdd_adapter_self_referential_detection() { ... }`
    (BC-2.02.005 line 93 canonical test-function name — verbatim, no variation)

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

## Out of Scope — Consumer Contract

This story delivers the `FactoryAdapter` trait and `VsddFactoryAdapter` implementation.
The following downstream consumption details are documented here for implementer awareness
but are OUT OF SCOPE for this story:

- **BC-2.02.004 PC-4 + BC-2.02.005 PC-3**: `"—"` display semantics for `None` optional fields
  (`awaiting: None`, `cycle: None`, `convergence: None`) — these display constants live in the
  TUI layer, not in `monocle-core::factory`. The adapter returns typed `None`; the TUI converts
  to `"—"` for display.
- **BC-2.01.002 (`/status` endpoint)**: The `/status` HTTP endpoint is the downstream consumer
  of `VsddFactoryAdapter::read_state()` results. The endpoint calls `read_state()` and maps
  `FactoryState` fields to the JSON response body. Wiring the adapter into `/status` is
  covered by S-003, not this story.
- **`"pending"` display value**: BC-2.02.005 PC-3 also specifies `"pending"` for the
  `awaiting` field when populated — this is a TUI/status display concern, not an adapter concern.

## §Trace v1.5

**Phase 3.B Batch 4 spec-reviewer remediation** (2026-05-20):
- F-D-01 (IMPORTANT) closed: FactoryDetection field names corrected in Tasks and AC-003 — `state_file_path`/`framework_version` → canonical `workspace_root: PathBuf`/`state_file: PathBuf` per SS-core-types-and-abi.md lines 337-344.
- F-D-03 (IMPORTANT) closed: Test function name pins added to Tasks — `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound` (BC-2.02.004 line 98) and `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection` (BC-2.02.005 line 93), verbatim.
- F-C-02 (IMPORTANT) closed: EC-021 frontmatter-vs-body negative test vector added to AC-005 — body-only `document_type: pipeline-state` → `detect()` returns `None`; note on SS-core-types-and-abi.md L602 divergence documented.
- F-E-03 (RECOMMENDED) closed: Out-of-Scope Consumer Contract section added — BC-2.02.004 PC-4 + BC-2.02.005 PC-3 `"—"`/`"pending"` display semantics; BC-2.01.002 (`/status` endpoint) named as downstream consumer.
- F-D-04 (OPTIONAL) deferred: Fixture convention for AC-013 guard vectors (8 fixture files) — not in formal findings scope; may be added by test-writer per convention.
- version bumped 1.4 → 1.5.
