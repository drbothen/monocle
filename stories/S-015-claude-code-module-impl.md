---
document_type: story
level: L4
story_id: S-015
epic_id: EPIC-03
version: "1.8"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 8
wave: 3
tdd_mode: strict
priority: P0
depends_on: [S-014]
blocks: []
target_module: monocle-runtime
subsystems: [SS-03]
behavioral_contracts: [BC-2.03.001, BC-2.03.002, BC-2.03.003, BC-2.03.004]
verification_properties: [VP-020, VP-021, VP-022]
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.001.md, version: "1.0.6"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.002.md, version: "1.0.4"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.003.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.004.md, version: "1.0.4"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-020-claude-code-module-impl.md, version: "1.0.12"}
  - {path: .factory/specs/verification-properties/vp-021-home-unresolvable-error.md, version: "1.0.13"}
  - {path: .factory/specs/verification-properties/vp-022-claude-code-module-inherent-methods.md, version: "1.0.12"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-engine-module.md, version: "1.1.20"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.03.001#postconditions (EngineModule trait — DI-006 postcondition 6), BC-2.03.002#postconditions (ClaudeCodeModule strict-basename detect), BC-2.03.003#invariants-line-55 (HomeUnresolvable error contract), BC-2.03.004#postconditions (ClaudeCodeModule inherent methods); verifies VP-020, VP-021, VP-022; covers EC-029, EC-030, EC-031, EC-032, EC-033, EC-034, EC-035, EC-038, EC-039; addresses NFR per SS-engine-module.md."
---

# S-015: ClaudeCodeModule Implementation

## Narrative

As the monocle daemon, I want `ClaudeCodeModule` to implement `EngineModule` with strict
basename detection (rejecting claude-squad, claudio, and other look-alike processes), fail-fast
`HomeUnresolvable` on missing home directory, and exactly 5 hook paths matching the hook
endpoint matrix, so that only genuine Claude Code processes are registered and session
management is accurate.

## Acceptance Criteria

### AC-001 (traces to BC-2.03.002 postcondition 4 + EC-034 — strict basename detect, both allowed names)
`ClaudeCodeModule::detect(proc)` returns `true` if and only if `proc.exe_path.file_name()`
equals exactly `"claude"` OR `"claude.js"` (case-sensitive STRICT basename match on the
RESOLVED exe path). Returns `false` for `"claude-squad"`, `"claudio"`, `"claude-code"`,
`"claude-code-runner"`, `"Claude"`, or any other string that is not precisely one of
these two allowed basenames. (BC-2.03.002 PC-4: `"claude"` or `"claude.js"`; EC-034:
`claude.js` Node.js wrapper returns `true`.)

### AC-002 (traces to BC-2.03.002 postcondition 5 + invariant 2 — exe_path None → false; cmdline not consulted)
When `proc.exe_path` is `None` (process exited before path resolved), `detect()` returns
`false` regardless of `cmdline` contents (BC-2.03.002 PC-5). `detect()` MUST NOT inspect
`proc.cmdline` as the primary signal — only `exe_path.file_name()` is canonical
(BC-2.03.002 invariant 2). EC-032: process with `cmdline[0] = "claude"` but
`exe_path = /usr/local/bin/claude-squad` → `detect()` returns `false`.

### AC-003 (traces to BC-2.03.002 postcondition 2 — infallible constructor)
`ClaudeCodeModule::new(hook_base_url: String) -> Self` constructor is provided. Construction
is infallible — URL validation is deferred to `preflight()` (BC-2.03.002 PC-2).

### AC-004 (traces to BC-2.03.002 postcondition 3 — id() returns "claude-code")
`ClaudeCodeModule::id()` returns the string `"claude-code"` — stable identifier, never
changes (BC-2.03.002 PC-3).

### AC-005 (traces to BC-2.03.003 postcondition 1 — HomeUnresolvable error on metadata/enrich)
When all four home-env vars are unset (`HOME`, `USERPROFILE`, `HOMEDRIVE`, `HOMEPATH`),
`ClaudeCodeModule::metadata()` and `ClaudeCodeModule::enrich()` return
`Err(EngineMetadataError::HomeUnresolvable)`. No default path is substituted
(BC-2.03.001 PC-5; BC-2.03.003 PC-1). (per BC-2.03.003 Invariant 3 — `XDG_*` is NOT
consulted by `BaseDirs::home_dir()`; on Windows, `HOMEDRIVE` is the active resolver source
and clearing it is essential to exercise the `HomeUnresolvable` path)

### AC-006 (traces to BC-2.03.003 postcondition 2 — daemon logs E-ENG-001 on HomeUnresolvable)
When `metadata()` or `enrich()` returns `HomeUnresolvable`, the daemon logs:
`ERROR: platform home directory unresolvable (BaseDirs::new() returned None)` (E-ENG-001).
Integration test: unset HOME + USERPROFILE + HOMEDRIVE + HOMEPATH; verify E-ENG-001 in log
output (BC-2.03.003 PC-2).

### AC-007 (traces to BC-2.03.004 postcondition 1 — hook_paths returns HashMap<HookType, String> with exactly 5 entries)
`ClaudeCodeModule::hook_paths()` is an inherent method (NOT a trait method) returning
`HashMap<HookType, String>` with exactly 5 entries (BC-2.03.004 PC-1; SS-engine-module.md
§Struct-level inherent operations). The 5 entries are:
- `HookType::SessionStart` → `"/hooks/session-start"`
- `HookType::UserPromptSubmit` → `"/hooks/prompt-submit"`
- `HookType::PreToolUse` → `"/hooks/pre-tool-use"`
- `HookType::Notification` → `"/hooks/notification"`
- `HookType::Stop` → `"/hooks/stop"`
`PostToolUse` is NOT included (JC-2 gene-source parity; BC-2.03.004 invariant 1).
VP-022 verifies `hook_paths().len() == 5` (vp-022-claude-code-module-inherent-methods.md; see VP-022 assertion strength anchor for exact line).

### AC-008 (traces to BC-2.03.004 postcondition 2 + EC-038 — spawn is todo!() stub)
`ClaudeCodeModule::spawn(args: SpawnArgs) -> Result<SessionHandle, SpawnError>` is an
inherent async method. Phase 1 implementation is `todo!()` — the signature is binding;
the implementation is intentionally stubbed (BC-2.03.004 PC-2; EC-038: `spawn()` called
in Phase 1 returns `todo!()` panic — intentional).

### AC-009 (traces to BC-2.03.004 postcondition 3 + EC-039 — preflight is todo!() stub)
`ClaudeCodeModule::preflight() -> Result<EngineVersion, PreflightError>` is an inherent
async method. Phase 1 implementation is `todo!()` — the signature is binding
(BC-2.03.004 PC-3; EC-039: `preflight()` called in Phase 1 returns `todo!()` panic;
Phase 1 story replaces the stub with `which claude` + `claude --version` checks in Phase 3).

### AC-010 (traces to BC-2.03.001 postcondition 6 + EC-031 — detect() is I/O-free; on_hook() fail-open)
`ClaudeCodeModule::detect()` performs no I/O — DI-006 enforcement. BC-2.03.001 postcondition 6
(verbatim): "`detect()` MUST NOT perform any I/O, environment lookups, file reads, or shared
state mutation. `detect()` is a pure function of its arguments. It MUST be safe to call from
any thread, repeatedly, without side effects." `on_hook()` returns
`HookResponse::new(HookDecision::Allow)` for unrecognized `HookEvent` variants (wildcard arm;
fail-open per EC-031; BC-2.03.001 EC-031).

Note: BC-2.03.001 invariant 2 covers the `HookEvent` type location in `hook_events.rs`
(a separate concern). DI-006 enforcement for detect() I/O-free is specifically postcondition 6
(added in BC-2.03.001 v1.0.4 at S-015 authoring time), which is the authoritative clause per BC-2.03.001 §Traceability
DI-006 mapping.

**Per-variant dispatch scope note:** Phase 1 `on_hook()` returns `HookResponse::new(HookDecision::Allow)`
for ALL `HookEvent` variants including the wildcard arm. Per-variant dispatch is out of scope
for S-015 and lives in the permission-overlay story (post-Phase-1 scope). This is consistent
with BC-2.03.001 EC-031 (Phase 1 always fail-open) and SS-engine-module.md line 650 (Phase 1
unconditionally returns `Allow`).

### AC-011 (traces to BC-2.03.001 postcondition 6 + EC-031 — per-variant regression guard)
Each of the 5 known `HookEvent` variants (`SessionStart`, `UserPromptSubmit`, `PreToolUse`,
`Notification`, `Stop`) passed to `on_hook()` returns `HookResponse` with
`decision == HookDecision::Allow`. This AC is a defensive regression guard ensuring no variant
accidentally returns `Deny` or `Defer` during Phase 1. (Per-variant semantics are Phase 2+ scope.)

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,400 |
| BC-2.03.001.md (1.0.6) | ~700 |
| BC-2.03.002.md (1.0.4) | ~700 |
| BC-2.03.003.md (1.0.3) | ~600 |
| BC-2.03.004.md (1.0.4) | ~700 |
| VP-020 + VP-021 + VP-022 files | ~1,200 |
| SS-engine-module.md v1.1.20 (ClaudeCodeModule + supporting types sections) | ~2,000 | <!-- version-pin-historical: at S-015 authoring time -->
| temp-env test isolation | ~300 |
| Test files | ~1,200 |
| **Total estimate** | **~8,800** |

Well within 20% of 200k context window. No split required.

## Tasks

- [ ] Create `monocle-runtime/src/engine/claude_code.rs` with `ClaudeCodeModule` struct
  - `pub struct ClaudeCodeModule { hook_base_url: String }` (construction infallible per AC-003)
  - `pub fn new(hook_base_url: String) -> Self`
- [ ] Implement `EngineModule` for `ClaudeCodeModule` with `#[async_trait]`
  - `id()` returns `"claude-code"` (AC-004)
  - `detect()`: STRICT basename match — canonical pattern (SS-engine-module.md v1.1.20 lines 604-608 at S-015 authoring time):
    ```rust
    proc.exe_path.as_ref()
        .and_then(|p| p.file_name())
        .map(|name| name == "claude" || name == "claude.js")
        .unwrap_or(false)
    ```
    Note: `detect()` returns `bool`, NOT `Option<bool>` — the `?` operator is illegal here. Use `and_then`/`map`/`unwrap_or` chain. NO I/O (AC-001, AC-010)
  - `metadata()`: uses `directories::BaseDirs::new()` → `None` → `Err(HomeUnresolvable)` (AC-005)
  - `enrich()`: same `HomeUnresolvable` failure path as `metadata()` (AC-005)
  - `on_hook()`: wildcard arm for unrecognized `HookEvent` → `HookResponse::new(HookDecision::Allow)` (AC-010, EC-031)
- [ ] Implement inherent methods:
  - `hook_paths() -> HashMap<HookType, String>` — 5 entries per AC-007 (NOT `Vec<PathBuf>`)
  - `spawn(args: SpawnArgs) -> Result<SessionHandle, SpawnError>` — `todo!()` stub per AC-008
  - `preflight() -> Result<EngineVersion, PreflightError>` — `todo!()` stub per AC-009
- [ ] Integration tests `monocle-runtime/tests/engine_module_claude.rs` (VP-020):
  - `detect()` true for `exe_path = /usr/local/bin/claude` (basename = "claude") [EC-033]
  - `detect()` true for `exe_path = /usr/local/bin/claude.js` (Node.js wrapper) [EC-034]
  - `detect()` false for `exe_path = /usr/local/bin/claude-squad` [EC-035]
  - `detect()` false for `exe_path = /usr/local/bin/claudio`
  - `detect()` false for `exe_path = None, cmdline: ["claude"]` [EC-032]
  - `detect()` false for `exe_path = /usr/local/bin/Claude` (case-sensitive)
- [ ] Integration tests `monocle-runtime/tests/engine_module_home_unresolvable.rs` (VP-021):
  - Use `temp-env 0.3` `async_with_vars` to unset HOME, USERPROFILE, HOMEDRIVE, HOMEPATH
  - Call `metadata()` → `Err(HomeUnresolvable)`
  - Call `enrich(proc)` → `Err(HomeUnresolvable)`
  - Verify E-ENG-001 log message emitted exactly
- [ ] Integration test for VP-022:
  - `hook_paths().len() == 5`
  - Each `HookType` maps to exact path string per AC-007

## Previous Story Intelligence

S-014 (Wave 2): `EngineModule` trait defined. `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`,
`EngineMetadataError::HomeUnresolvable` all defined.
`temp-env 0.3` is pinned as dev-dependency in `monocle-runtime/Cargo.toml`.

### Inherited supporting-type surface from S-014 (monocle_core::engine)

**Important:** `DeferUntil` was DROPPED in S-014 Batch-3 fix (commit 08e03475) — S-015 MUST NOT list
or import `DeferUntil`. `SessionStatus` has 5 variants (not 4). `HookResponse` has 3 fields (not 4;
no `deferred_until` field).

From `monocle_core::engine` (S-014 deliverable, SS-engine-module v1.1.20 at S-015 authoring time):

- `EngineModule` trait (lines 540-549 in SS-engine-module v1.1.20 at S-015 authoring time)
- `EngineMetadata` struct (lines 138-149); constructor `EngineMetadata::new()`
- `ProcessSnapshot` struct (lines 195-224); constructor `ProcessSnapshot::new()`
- `EnrichedSession` struct (lines 301-330); constructor `EnrichedSession::new()`
- `HookEvent` enum (canonical 5 variants per BC-2.03.001 invariant 2)
- `HookDecision` enum (`Allow` / `Deny` / `Defer` — but Phase 1 only uses `Allow`)
- `HookResponse` struct (3 fields: `decision`, `redirect_url: Option<String>`, `diagnostic: Option<String>`); constructor `HookResponse::new(decision)`, builders `with_diagnostic` / `with_redirect`
- `SessionStatus` enum (5 variants: `Active`, `Idle`, `WaitingOnPermission`, `Stopping`, `Stopped`)
- `EngineMetadataError` enum (variant: `HomeUnresolvable`)

**TDD pre-gate stub:** S-015 implementation MUST NOT proceed until S-014 supporting types match
SS-engine-module v1.1.20 lines 380-531 verbatim at S-015 authoring time. Compile-check before S-015 RED phase begins:
```rust
let _ = HookResponse::new(HookDecision::Allow);
let _ = SessionStatus::Active;
```
Both must compile against `monocle_core::engine` re-exports before dispatch.

## Architecture Compliance Rules

From `architecture/SS-engine-module.md` v1.1.20 §Phase 1 Implementation (at S-015 authoring time): ClaudeCodeModule (canonical reference impl lines 545-657):
- Strict basename match: ONLY `"claude"` OR `"claude.js"` — NOT `"claude-squad"`, NOT `"claudio"`, NOT partial match
- `hook_paths()` returns `HashMap<HookType, String>` (NOT `Vec<PathBuf>`) per SS-engine-module.md §Struct-level inherent operations
- `spawn(args: SpawnArgs) -> Result<SessionHandle, SpawnError>` — binding signature; Phase 1 body is `todo!()`
- `preflight() -> Result<EngineVersion, PreflightError>` — binding signature; Phase 1 body is `todo!()`
- `detect()` is I/O-free — DI-006 (BC-2.03.001 postcondition 6)
- `HomeUnresolvable` fail-fast — no default path substitution (BC-2.03.001 PC-5; BC-2.03.003 PC-1)
- `on_hook()` wildcard arm for unknown HookEvent variants → fail-open: `HookResponse::new(HookDecision::Allow)` (BC-2.03.001 EC-031)

**Forbidden Dependencies:**
- `detect()` MUST NOT perform any I/O (no `std::fs` calls, no `std::process` calls)
- `ClaudeCodeModule` MUST NOT write to any Claude Code files (DI-007)
- `hook_paths()` MUST NOT return `Vec<PathBuf>` — return type is `HashMap<HookType, String>`

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| directories | 6 | `BaseDirs::new()` for home directory resolution |
| async-trait | 0.1 | `#[async_trait]` on EngineModule impl |
| temp-env | 0.3 | Test: unset home env vars (dev-dependency) |
| tracing | 0.1 | E-ENG-001 log on HomeUnresolvable |

## File Structure Requirements

Files to create:
- `monocle-runtime/src/engine/claude_code.rs` — `ClaudeCodeModule` implementation
- `monocle-runtime/src/engine/mod.rs` — module declaration + re-exports
- `monocle-runtime/tests/engine_module_claude.rs` — detect() integration tests
- `monocle-runtime/tests/engine_module_home_unresolvable.rs` — VP-021 tests

Files to modify:
- `monocle-runtime/src/lib.rs` — engine module is now `pub mod engine` (sub-module)
- `monocle-runtime/Cargo.toml` — confirm `temp-env` dev-dependency present

> **Implementation Note — `which` crate (Phase 3 scope):** `preflight()` is `todo!()` in Phase 1 per EC-039 (BC-2.03.004 PC-3). The `which::which()` crate for $PATH lookup of `claude`/`claude.js` binaries is NOT a Phase 1 dependency. When the Phase 3 preflight story is created, the architect MUST add `which` (or a functionally equivalent crate) to `SS-deps-pin-manifest.md` with an explicit version pin before dispatch. Do not add `which` to `monocle-runtime/Cargo.toml` in Phase 1.

## Trace

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-19 | vsdd-factory:story-writer | Initial story decomposition (Phase 2) |
| 1.1–1.5 | 2026-05-19 | vsdd-factory:story-writer | Iterative adversarial convergence (r01..r13) |
| 1.6 | 2026-05-20 | vsdd-factory:story-writer | Phase 3.B Batch 5: XDG_HOME (not a real env var) replaced with HOMEDRIVE per BC-2.03.003 Invariant 3 in AC-005, AC-006, Tasks; detect() pseudocode ? operator (won't compile) replaced with canonical and_then/map/unwrap_or pattern; inherited supporting-type surface enumerated (DeferUntil dropped, SessionStatus 5 variants, HookResponse 3 fields); AC-010 per-variant dispatch clarification; AC-011 added (defensive per-variant regression guard); traces_to fragment anchors added; SS-engine-module canonical impl anchor added to Architecture Compliance Rules; VP-022 line reference added to AC-007 |
| 1.7 | 2026-05-29 | vsdd-factory:story-writer | Path B Wave 6 MSRV propagation: BC-2.03.001 input pin v1.0.5 → v1.0.6 (PO commit 5006528); Token Budget table BC version cell updated; no MSRV 1.86 body references found in S-015 (sweep confirmed clean). Per bc_array_changes_propagate_to_body_and_acs policy (F-S025-ADV17-LOW-001 closure). |

## §Trace

**v1.8** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at story authoring time. No normative content changed.
