---
document_type: story
story_id: S-014
epic_id: EPIC-03
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-010]
blocks: [S-015]
target_module: monocle-core
subsystems: [SS-03]
behavioral_contracts: [BC-2.03.001]
verification_properties: [VP-019]
estimated_days: 2
---

# S-014: EngineModule Trait Definition

## Narrative

As a harness adapter implementer (Phase 1: ClaudeCodeModule; Phase 3: WASM plugins;
Phase 4: CodeMachineModule), I want a stable open `EngineModule` trait in `monocle-core`
with exactly 5 `#[async_trait]` methods and `Send + Sync + 'static` supertraits, so that
I can implement it without coupling to daemon internals or risking forward-incompatibility.

## Acceptance Criteria

### AC-001 (traces to BC-2.03.001 postcondition 1 — 5 methods exact with correct signatures)
`EngineModule` trait in `monocle-core::engine` has exactly these 5 methods:
- `fn id(&self) -> &'static str`
- `fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError>`
- `fn detect(&self, proc: &ProcessSnapshot) -> bool`
- `async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>`
- `async fn on_hook(&self, event: HookEvent) -> HookResponse`

### AC-002 (traces to BC-2.03.001 postcondition 2 — no Sealed bound)
`EngineModule` has NO `private::Sealed` supertrait. Supertrait bounds: `Send + Sync + 'static`
only. AST audit via VP-019 verifies.

### AC-003 (traces to BC-2.03.001 postcondition 3 — supporting types co-located)
`monocle-core::engine` declares: `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`,
`SessionStatus`, `HookResponse`, `HookDecision`, `DeferUntil`, `EngineMetadataError`.

### AC-004 (traces to BC-2.03.001 postcondition 4 — last_event_micros is Option<i64>)
`EnrichedSession::last_event_micros` is `Option<i64>`. `None` means no hook events
received yet. `Some(t)` is microseconds since Unix epoch. The value `0i64` is NOT a
sentinel — it represents Unix epoch 1970-01-01.

### AC-005 (traces to BC-2.03.001 postcondition 5 — no silent fallback in metadata/enrich)
`metadata()` and `enrich()` MUST return `Err(EngineMetadataError::HomeUnresolvable)` when
the platform home directory is unresolvable. They MUST NOT substitute a default path.

### AC-006 (traces to BC-2.03.001 invariant 1 — trait is OPEN)
The trait has no sealed bound. AST audit confirms. Downstream crates may implement `EngineModule`.

### AC-007 (traces to BC-2.03.001 invariant 3 — async_trait macro required)
`#[async_trait::async_trait]` macro is applied to the trait declaration. Rustdoc on the trait
explains why: native `async fn` in traits does not provide ergonomic dyn-compatibility on MSRV 1.86.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~800 |
| BC-2.03.001.md | ~700 |
| VP-019 file | ~500 |
| SS-engine-module.md (trait section, ~80 lines) | ~1,200 |
| async-trait crate documentation | ~300 |
| Test file | ~600 |
| **Total estimate** | **~4,100** |

## Tasks

- [ ] Create `monocle-core/src/engine.rs` with `EngineModule` trait
  - Apply `#[async_trait::async_trait]` to trait definition
  - 5 methods with exact signatures from postcondition 1
  - Supertrait: `Send + Sync + 'static`
  - Rustdoc explaining async_trait requirement
- [ ] Define supporting types in `monocle-core/src/engine.rs`:
  - `EngineMetadata` (fields: `id`, `display_name`, `config_paths: Vec<PathBuf>`, `hook_schema_version: u32`)
  - `ProcessSnapshot` (fields: `pid: u32`, `exe_path: Option<PathBuf>`, `cmdline: Option<String>`)
  - `EnrichedSession` (fields: `engine_id: String`, `session_id: String`, `last_event_micros: Option<i64>`, ...)
  - `SessionStatus` (enum, `#[non_exhaustive]`: `Running`, `Idle`, `Exited`)
  - `HookResponse` (fields: `decision: HookDecision`, `deferred_until: Option<DeferUntil>`)
  - `HookDecision` (enum, `#[non_exhaustive]`: `Allow`, `Block`, `Defer`)
  - `DeferUntil` (enum, `#[non_exhaustive]`: `UserApproval`, `Timeout(Duration)`)
  - `EngineMetadataError` (enum: `HomeUnresolvable`)
- [ ] Create `monocle-core/tests/engine_module_surface.rs` (VP-019 AST audit):
  - syn 2 parse `engine.rs`: assert 5 methods, no `Sealed`, correct return-type token streams
- [ ] Add `async-trait = "0.1"` to `monocle-core/Cargo.toml`

## Previous Story Intelligence

S-010 (Wave 2): `monocle-core` crate structure established. `async-trait 0.1` pinned in workspace.
S-011 (Wave 2): `#[non_exhaustive]` policy enforced — apply to `SessionStatus`, `HookDecision`, `DeferUntil`.
`HookEvent` type is referenced from `monocle-core/src/hook_events.rs` — create this module.

## Architecture Compliance Rules

From `architecture/SS-engine-module.md` v1.1.20 §EngineModule Trait Signature:
- `#[async_trait]` is REQUIRED — native `async fn` in traits is NOT dyn-compatible on MSRV 1.86
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

## File Structure Requirements

Files to create:
- `monocle-core/src/engine.rs` — `EngineModule` trait + all supporting types
- `monocle-core/src/hook_events.rs` — `HookEvent` type (referenced by trait)
- `monocle-core/tests/engine_module_surface.rs` — VP-019 AST audit

Files to modify:
- `monocle-core/src/lib.rs` — add `pub mod engine; pub mod hook_events;`
- `monocle-core/Cargo.toml` — add `async-trait = "0.1"`, `thiserror = "2"`
