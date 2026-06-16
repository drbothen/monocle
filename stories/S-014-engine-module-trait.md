---
document_type: story
level: L4
story_id: S-014
epic_id: EPIC-03
version: "1.8"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-010]
blocks: [S-015, S-018, S-021, S-024, S-033]
target_module: monocle-core
subsystems: [SS-03]
behavioral_contracts: [BC-2.02.003, BC-2.03.001]
verification_properties: [VP-019]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.001.md, version: "1.0.6"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.003.md, version: "1.0.2"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-019-engine-module-trait.md, version: "1.0.13"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-engine-module.md, version: "1.1.20"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.004.md, version: "1.0.4"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.18"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.03.001 (EngineModule Trait Definition); verifies VP-019; covers EC-029 (AC-005: HomeUnresolvable fail-fast), EC-030 (AC-001: method signatures exact), EC-031 (AC-003b: wildcard arm for non_exhaustive HookEvent); addresses DI-006."
---

# S-014: EngineModule Trait Definition

## Narrative

As a harness adapter implementer (Phase 1: ClaudeCodeModule; Phase 3: WASM plugins;
Phase 4: CodeMachineModule), I want a stable open `EngineModule` trait in `monocle-core`
with exactly 5 `#[async_trait]` methods and `Send + Sync + 'static` supertraits, so that
I can implement it without coupling to daemon internals or risking forward-incompatibility.

## Acceptance Criteria

### AC-001 (traces to BC-2.03.001 postcondition 1 — 5 methods exact with correct signatures; anchors EC-030)
`EngineModule` trait in `monocle-core::engine` has exactly these 5 methods:
- `fn id(&self) -> &'static str`
- `fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError>`
- `fn detect(&self, proc: &ProcessSnapshot) -> bool`
- `async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>`
- `async fn on_hook(&self, event: HookEvent) -> HookResponse`

### AC-002 (traces to BC-2.03.001 postcondition 2 — no Sealed bound)
`EngineModule` has NO `private::Sealed` supertrait. Supertrait bounds: `Send + Sync + 'static`
only. AST audit via VP-019 assertion 19.b verifies — specifically: syn 2 token-stream substring-absence
check confirms the identifier `Sealed` does not appear in the supertrait bound position of the
`EngineModule` trait declaration in `monocle-core/src/engine.rs`.

### AC-003 (traces to BC-2.03.001 postcondition 3 — supporting types co-located)
`monocle-core::engine` declares: `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`,
`SessionStatus`, `HookResponse`, `HookDecision`, `EngineMetadataError`.
`HookEvent` is defined in `monocle-core/src/hook_events.rs` (BC-2.03.001 invariant 2;
SS-core-types-and-abi.md §Non-Exhaustive Inner Structs — non-exhaustive inner structs policy)
— NOT re-declared in engine.rs. Note: `DeferUntil` is NOT part of the engine module surface;
it has no canonical home in SS-engine-module.md v1.1.20 (F-D-03 resolution).

### AC-003b (traces to BC-2.03.001 postcondition 3 + BC-2.02.003 invariant — HookEvent #[non_exhaustive] + variants)
`HookEvent` enum in `monocle-core/src/hook_events.rs` carries `#[non_exhaustive]` attribute
(BC-2.02.003; BC-2.03.001 invariant 2 — per SS-core-types-and-abi.md §Non-Exhaustive Inner Structs:
"all non-exhaustive inner structs and enums in `monocle-core` carry `#[non_exhaustive]` to permit
Phase 4 additions without binary-incompatibility"). The Phase 1 canonical variants are:
`SessionStart`, `UserPromptSubmit`, `PreToolUse`, `Notification`, `Stop`
(matching the 5 hook endpoints; `PostToolUse` is omitted per JC-2 — JC-2 records the explicit
decision that `post-tool-use` is not part of the Phase 1 canonical 5-endpoint surface; see
`oq-research.md §JC-2` for the full decision record; implemented by BC-2.03.004 invariant 1).
Wildcard match arm (`_ => HookResponse::new(HookDecision::Allow)`) is required in all
`HookEvent` match sites (EC-031 — fail-open for future Phase 4 variants; anchors EC-031).

### AC-004 (traces to BC-2.03.001 postcondition 4 — last_event_micros is Option<i64>)
`EnrichedSession::last_event_micros` is `Option<i64>`. `None` means no hook events
received yet. `Some(t)` is microseconds since Unix epoch. The value `0i64` is NOT a
sentinel — it represents Unix epoch 1970-01-01.

### AC-005 (traces to BC-2.03.001 postcondition 5 — no silent fallback in metadata/enrich; anchors EC-029)
`metadata()` and `enrich()` MUST return `Err(EngineMetadataError::HomeUnresolvable)` when
the platform home directory is unresolvable. They MUST NOT substitute a default path.

### AC-006 (traces to BC-2.03.001 invariant 1 — trait is OPEN)
The trait has no sealed bound. AST audit confirms. Downstream crates may implement `EngineModule`.

### AC-007 (traces to BC-2.03.001 invariant 3 — async_trait macro required)
`#[async_trait::async_trait]` macro is applied to the trait declaration. Rustdoc on the trait
explains why: native `async fn` in traits does not provide ergonomic dyn-compatibility on MSRV 1.88.
Executable oracle: a `grep`-based test in `engine_module_surface.rs` asserts that the canonical
BC-2.03.001 invariant 3 rationale text (substring: `"native async fn in traits does not provide"` OR
`"async_trait"`) appears in the rustdoc comment on the `EngineModule` trait declaration in
`monocle-core/src/engine.rs`.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~800 |
| BC-2.03.001.md v1.0.7 | ~700 |
| BC-2.02.003.md v1.0.2 | ~350 |
| BC-2.03.004.md v1.0.5 (JC-2 PostToolUse parity) | ~300 |
| VP-019 file | ~500 |
| SS-engine-module.md (trait section, ~80 lines) | ~1,200 |
| SS-deps-pin-manifest.md v1.1.18 (syn 2.0 dev-dep) | ~200 | <!-- version-pin-historical: at S-014 authoring time -->
| async-trait crate documentation | ~300 |
| Test file | ~600 |
| **Total estimate** | **~4,950** |

## Tasks

- [ ] Create `monocle-core/src/engine.rs` with `EngineModule` trait
  - Apply `#[async_trait::async_trait]` to trait definition
  - 5 methods with exact signatures from postcondition 1
  - Supertrait: `Send + Sync + 'static`
  - Rustdoc explaining async_trait requirement
- [ ] Define supporting types in `monocle-core/src/engine.rs` (verbatim from SS-engine-module.md §Supporting Types):
  - `EngineMetadata` (canonical 4 fields from SS-engine-module.md lines 138–149):
    `display_name: &'static str`, `icon: char`, `config_paths: Vec<PathBuf>`, `hook_schema_version: u32`
    (NO `id` field — `id` is a separate trait method `fn id(&self) -> &'static str` at SS-engine-module.md line 95)
    Carries `#[non_exhaustive]`. Provides `pub fn new(display_name, icon, config_paths, hook_schema_version) -> Self`.
  - `ProcessSnapshot` (7 fields from SS-engine-module.md lines 195–224):
    `pid: u32`, `ppid: Option<u32>`, `exe_path: Option<PathBuf>`, `cmdline: Vec<String>`,
    `working_dir: Option<PathBuf>`, `env: HashMap<String, String>`, `start_time_secs: i64`
    Carries `#[non_exhaustive]`. Provides `new()` and `with_full_context()` constructors.
  - `EnrichedSession` (6 fields from SS-engine-module.md lines 301–330):
    `session_id: String`, `harness_type: String`, `transcript_path: Option<PathBuf>`,
    `config_path: Option<PathBuf>`, `status: SessionStatus`, `last_event_micros: Option<i64>`
    (NO `engine_id` field — use `harness_type` per SS-engine-module.md)
    Carries `#[non_exhaustive]`.
  - `SessionStatus` (enum, `#[non_exhaustive]`: `Active`, `Idle`, `WaitingOnPermission`, `Stopping`, `Stopped`)
    — VERBATIM from SS-engine-module.md v1.1.20 lines 388-400 (F-D-01 fix: was 3 variants, canonical is 5)
  - `HookResponse` (`#[non_exhaustive]` struct, fields: `decision: HookDecision`, `redirect_url: Option<String>`, `diagnostic: Option<String>`)
    — VERBATIM from SS-engine-module.md v1.1.20 lines 403-416 (F-D-02 fix: was deferred_until, canonical is redirect_url + diagnostic)
    Provides `pub fn new(decision: HookDecision) -> Self` constructor (returns Self with None for redirect_url and diagnostic).
    Provides builder methods: `pub fn with_diagnostic(self, impl Into<String>) -> Self` and `pub fn with_redirect(self, impl Into<String>) -> Self`
    — VERBATIM from SS-engine-module.md v1.1.20 lines 432-439. <!-- version-pin-historical: at S-014 authoring time -->
  - `HookDecision` (enum, `#[non_exhaustive]`: `Allow`, `Block`, `Defer`)
  - NOTE: `DeferUntil` is DROPPED entirely — no HookResponse field uses it in the canonical schema (F-D-03 fix)
  - `EngineMetadataError` (enum, `#[non_exhaustive]`: `HomeUnresolvable`)
- [ ] Create `monocle-core/src/hook_events.rs` with `HookEvent` enum:
  - `#[non_exhaustive]` on `HookEvent`
  - Phase 1 variants: `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `Notification`, `Stop`
  - `PostToolUse` is NOT included (JC-2 parity per BC-2.03.004 invariant 1)
- [ ] Create `monocle-core/tests/engine_module_surface.rs` (VP-019 AST audit):
  - syn 2 parse `engine.rs`: assert VP-019's 4 discrete assertions (VP-019 §Mechanism line 73):
    - 19.a: exactly 5 trait methods with canonical name set
    - 19.b: no `Sealed` supertrait (AC-002 syn assertion — substring-absence check for `Sealed` token)
    - 19.c: `Send + Sync + 'static` supertrait bounds present
    - 19.d: `#[async_trait]` attribute on trait
  - Map: AC-006 → 19.a+19.b+19.c; AC-002 → 19.b specifically (no-Sealed); each AC maps to one VP-019 assertion
- [ ] Add `async-trait = "0.1"` to `monocle-core/Cargo.toml`
- [ ] Add `syn = { version = "2.0", features = ["full"] }` to `monocle-core/Cargo.toml` `[dev-dependencies]`
  (per SS-deps-pin-manifest v1.1.18 §Phase 1 Pin Manifest <!-- version-pin-historical: at S-014 authoring time --> — syn 2.0 added as dev-dependency for AST audit tests)

## Previous Story Intelligence

S-010 (Wave 2): `monocle-core` crate structure established. `async-trait 0.1` pinned in workspace per
S-010's Cargo.toml workspace dep block. Confirm `async-trait 0.1` appears in S-010 workspace Cargo.toml
`[workspace.dependencies]` before dispatching S-014; if absent, add it as part of S-014 Cargo.toml task.
S-011 (Wave 2): `#[non_exhaustive]` policy enforced — apply to `SessionStatus`, `HookDecision`. Note:
`DeferUntil` was removed from the engine module surface in Batch 3 remediation (F-D-03); S-011 should NOT
apply `#[non_exhaustive]` to a `DeferUntil` type in `monocle-core`.
`HookEvent` type is referenced from `monocle-core/src/hook_events.rs` — create this module.
S-011 declares three permissions enums (`DenyReason`, `AllowPattern`, `DenyPattern`) as first-declared
in S-011, not S-010. S-010 only declares `Phase1Permission` and `ClaudeCodeTool`.

## Architecture Compliance Rules

From `architecture/SS-engine-module.md` v1.1.20 §EngineModule Trait Signature:
- `#[async_trait]` is REQUIRED — native `async fn` in traits is NOT dyn-compatible on MSRV 1.88
- Trait MUST be OPEN — NO sealed bound (Phase 3 WASM extensibility)
- `EnrichedSession::last_event_micros: Option<i64>` — `0i64` is NOT a sentinel
- `metadata()` and `enrich()` MUST NOT substitute default home path — fail fast with `HomeUnresolvable`

**Forbidden Dependencies:**
- `monocle-core::engine` MUST NOT import from `monocle-runtime`
- `detect()` MUST NOT perform I/O (DI-006)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| async-trait | 0.1 | `#[async_trait]` macro on trait declaration |
| serde | 1 | Serialize/Deserialize derive on engine types |
| thiserror | 2 | `EngineMetadataError` derive |
| syn | 2.0 (caret) | dev-dependency — AST audit test for `#[non_exhaustive]` policy and HookResponse field set; per SS-deps-pin-manifest v1.1.18 §Phase 1 Pin Manifest <!-- version-pin-historical: at S-014 authoring time --> |

## File Structure Requirements

Files to create:
- `monocle-core/src/engine.rs` — `EngineModule` trait + all supporting types
- `monocle-core/src/hook_events.rs` — `HookEvent` type (referenced by trait)
- `monocle-core/tests/engine_module_surface.rs` — VP-019 AST audit

Files to modify:
- `monocle-core/src/lib.rs` — add `pub mod engine; pub mod hook_events;`
- `monocle-core/Cargo.toml` — add `async-trait = "0.1"`, `thiserror = "2"` to `[dependencies]`; add `syn = { version = "2.0", features = ["full"] }` to `[dev-dependencies]`

## Downstream Consumer Surface for S-015

S-015 (ClaudeCodeModule) imports the following from `monocle-core` at `wave 3` dispatch:

From `monocle_core::engine`:
- `EngineModule` trait (implements via `ClaudeCodeModule`)
- `EngineMetadata`, `EngineMetadata::new` constructor
- `ProcessSnapshot`, `EnrichedSession`, `EnrichedSession::new` constructor
- `SessionStatus` (variants: `Active`, `Idle`, `WaitingOnPermission`, `Stopping`, `Stopped`)
- `HookResponse`, `HookResponse::new(HookDecision)` constructor
- `HookResponse::with_diagnostic(impl Into<String>)` builder method
- `HookResponse::with_redirect(impl Into<String>)` builder method
- `HookDecision` (variants: `Allow`, `Block`, `Defer`)
- `EngineMetadataError` (variant: `HomeUnresolvable`)

From `monocle_core::hook_events`:
- `HookEvent` (variants: `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `Notification`, `Stop`)

S-015 implementer must NOT use struct-literal construction for `HookResponse` (E0639 — struct is
`#[non_exhaustive]`; only `HookResponse::new(...)` + builder methods are legal from outside `monocle-core`).

## §Trace

**v1.6** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at story authoring time. No normative content changed.

**v1.5 — Path B Wave 6 MSRV propagation** (2026-05-29):
- BC-2.03.001 input pin bumped v1.0.5 → v1.0.6 (PO commit 5006528 — MSRV 1.86 → 1.88 in BC body).
- Token Budget table: `BC-2.03.001.md v1.0.5` → `v1.0.6`.
- AC-007 (line 98): `MSRV 1.86` → `MSRV 1.88` (present-tense project MSRV claim in rustdoc rationale; not language-history reference).
- Architecture Compliance Rules (line 180): `MSRV 1.86` → `MSRV 1.88` (same reason).
- Per `bc_array_changes_propagate_to_body_and_acs` policy (F-S025-ADV17-LOW-001 closure).

**v1.4 — Phase 3.B Batch 3: arch-touching story remediation** (2026-05-20):
- F-D-01: `SessionStatus` variants corrected 3→5 (`Active, Idle, WaitingOnPermission, Stopping, Stopped`)
  per SS-engine-module.md v1.1.20 lines 388-400.
- F-D-02: `HookResponse` field set corrected to canonical (`decision, redirect_url, diagnostic`);
  added `with_diagnostic` and `with_redirect` builder methods per SS-engine-module.md lines 432-439.
- F-D-03: `DeferUntil` ghost type dropped entirely from supporting types and AC-003.
- F-A-01: `syn 2.0` dev-dep added to Library table and Cargo.toml tasks per SS-deps-pin-manifest v1.1.18.
- F-B-01: BC-2.03.001 invariant 2 text inlined with exact anchor in AC-003b.
- F-B-02: JC-2 inline-defined in AC-003b; BC-2.03.004 v1.0.4 added to frontmatter inputs.
- F-B-03: EC-029/030/031 explicit anchors added in traces_to, AC-001, AC-005, AC-003b.
- F-C-01: AC-002 syn 2 substring-absence assertion for `Sealed` made explicit.
- F-C-02: AC-007 rustdoc oracle added (grep test asserting async_trait rationale text present).
- F-C-03: VP-019 4 discrete assertions mapped in test task; AC-006→19.a+19.b+19.c, AC-002→19.b.
- F-D-05: VP-019 test fixture inline assertion list added to engine_module_surface.rs task.
- F-E-01: Previous Story Intelligence updated re async-trait pin verification + DeferUntil note.
- F-E-02: Downstream Consumer Surface section added enumerating S-015 import set.
## §Trace v1.7 — POL-11 version-pin remediation (2026-05-30)

**Bump:** 1.6 → 1.7.
**Scope:** Token Budget Estimate table — BC version pins bumped to canonical current (Option 1 per ADR-0007):
- `BC-2.03.001.md v1.0.6` → `BC-2.03.001.md v1.0.7` (registry canonical).
- `BC-2.03.004.md v1.0.4` → `BC-2.03.004.md v1.0.5` (registry canonical).
**SE-16d PASS:** 2026-05-30 >= prior date (patch; no normative behavioral change).
