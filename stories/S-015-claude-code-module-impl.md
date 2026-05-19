---
document_type: story
story_id: S-015
epic_id: EPIC-03
version: "1.0"
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
target_module: monocle-core
subsystems: [SS-03]
behavioral_contracts: [BC-2.03.002, BC-2.03.003, BC-2.03.004]
verification_properties: [VP-020, VP-021, VP-022]
estimated_days: 3
---

# S-015: ClaudeCodeModule Implementation

## Narrative

As the monocle daemon, I want `ClaudeCodeModule` to implement `EngineModule` with strict
basename detection (rejecting claude-squad, claudio, and other look-alike processes), fail-fast
`HomeUnresolvable` on missing home directory, and exactly 5 hook paths matching the hook
endpoint matrix, so that only genuine Claude Code processes are registered and session
management is accurate.

## Acceptance Criteria

### AC-001 (traces to BC-2.03.002 postcondition 1 — strict basename detect)
`ClaudeCodeModule::detect(proc)` returns `true` if and only if `proc.exe_path` basename
equals exactly `"claude"` (case-sensitive, no extension matching). Returns `false` for
`"claude-squad"`, `"claudio"`, `"claude-code"`, `"Claude"`, and any other string that is
not the exact basename `"claude"`.

### AC-002 (traces to BC-2.03.002 postcondition 2 — cmdline ignored)
`detect()` does NOT inspect `proc.cmdline` — only `proc.exe_path.file_name()` is checked.
Integration test: process with `exe_path = "/usr/local/bin/claude"` and
`cmdline = Some("claude-squad --session abc")` → `detect()` returns `true`.

### AC-003 (traces to BC-2.03.002 postcondition 3 — exe_path None → false)
When `proc.exe_path` is `None` (process exited before path resolved), `detect()` returns `false`.

### AC-004 (traces to BC-2.03.003 postcondition 1 — HomeUnresolvable error on metadata/enrich)
When all four home-env vars are unset (`HOME`, `USERPROFILE`, `HOMEPATH`, `XDG_HOME`),
`ClaudeCodeModule::metadata()` and `ClaudeCodeModule::enrich()` return
`Err(EngineMetadataError::HomeUnresolvable)`.

### AC-005 (traces to BC-2.03.003 postcondition 2 — daemon logs E-ENG-001 on HomeUnresolvable)
When `metadata()` or `enrich()` returns `HomeUnresolvable`, the daemon logs:
`ERROR: platform home directory unresolvable (BaseDirs::new() returned None)` (E-ENG-001).
Integration test: unset HOME + USERPROFILE + HOMEPATH + XDG_HOME; call daemon start;
verify E-ENG-001 in log output.

### AC-006 (traces to BC-2.03.004 postcondition 1 — hook_paths returns exactly 5 entries)
`ClaudeCodeModule::hook_paths()` returns a `Vec<PathBuf>` of exactly 5 entries, one per
`HookType` variant:
- `HookType::PreToolUse` → `/hooks/pre-tool-use`
- `HookType::Notification` → `/hooks/notification`
- `HookType::Stop` → `/hooks/stop`
- `HookType::SessionStart` → `/hooks/session-start`
- `HookType::UserPromptSubmit` → `/hooks/prompt-submit`

### AC-007 (traces to BC-2.03.004 postcondition 2 — spawn)
`ClaudeCodeModule::spawn(session_id, config)` produces a subprocess handle that monocle
can use to start a new Claude Code session. Phase 1 scope: method signature matches trait;
full spawn implementation is Phase 3.

### AC-008 (traces to BC-2.03.004 postcondition 3 — preflight)
`ClaudeCodeModule::preflight()` checks that `claude` binary is on PATH. Returns
`Ok(())` if found; `Err(EnginePreflightError::BinaryNotFound)` if absent.

### AC-009 (traces to BC-2.03.001 invariant — detect() is I/O-free)
`ClaudeCodeModule::detect()` performs no I/O (no file reads, no process list queries,
no network calls). DI-006 enforcement. Integration test: mock `ProcessSnapshot` with
`exe_path = None`; verify `detect()` returns instantly without blocking.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,000 |
| BC-2.03.002.md | ~600 |
| BC-2.03.003.md | ~600 |
| BC-2.03.004.md | ~600 |
| VP-020 + VP-021 + VP-022 files | ~1,200 |
| SS-engine-module.md (ClaudeCodeModule section, ~100 lines) | ~1,500 |
| temp-env test isolation | ~300 |
| Test files | ~1,000 |
| **Total estimate** | **~6,800** |

## Tasks

- [ ] Create `monocle-core/src/engine/claude.rs` with `ClaudeCodeModule` struct
- [ ] Implement `EngineModule` for `ClaudeCodeModule` with `#[async_trait]`
  - `id()` returns `"claude-code"`
  - `detect()`: `proc.exe_path.as_ref()?.file_name()?.to_str()? == "claude"` — no I/O
  - `metadata()`: uses `directories::BaseDirs::new()` → `None` → `HomeUnresolvable`
  - `enrich()`: same failure path as `metadata()`
  - `on_hook()`: fail-open for unknown `HookEvent` variants (wildcard arm → `HookResponse::allow()`)
- [ ] Implement inherent methods:
  - `hook_paths()` → Vec<PathBuf> of 5 entries per AC-006
  - `spawn()` → Phase 1 stub; signature matches trait expectation
  - `preflight()` → checks `which::which("claude")`
- [ ] Integration tests `monocle-runtime/tests/engine_module_claude.rs`:
  - `detect()` true for `exe_path = /usr/local/bin/claude` (basename = "claude")
  - `detect()` false for `exe_path = /usr/local/bin/claude-squad`
  - `detect()` false for `exe_path = None`
  - `detect()` false for `exe_path = /usr/local/bin/Claude` (case-sensitive)
- [ ] Integration tests `monocle-runtime/tests/engine_module_home_unresolvable.rs` (VP-021):
  - Use `temp-env 0.3` `async_with_vars` to unset HOME, USERPROFILE, HOMEPATH, XDG_HOME
  - Call `metadata()` → `Err(HomeUnresolvable)`
  - Call `enrich(proc)` → `Err(HomeUnresolvable)`
  - Verify E-ENG-001 log message emitted
- [ ] Integration test for VP-022:
  - `hook_paths()` returns vec of exactly 5 paths matching spec

## Previous Story Intelligence

S-014 (Wave 2): `EngineModule` trait defined. `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`,
`EngineMetadataError::HomeUnresolvable` all defined.
`temp-env 0.3` is pinned as dev-dependency in `monocle-runtime/Cargo.toml`.

## Architecture Compliance Rules

From `architecture/SS-engine-module.md` v1.1.20 §ClaudeCodeModule:
- Strict basename match: ONLY `"claude"` — NOT `"claude-squad"`, NOT `"Claude"`, NOT partial match
- `detect()` is I/O-free — DI-006
- `HomeUnresolvable` fail-fast — no default path substitution (postcondition 5 BC-2.03.001)
- `on_hook()` wildcard arm for unknown HookEvent variants → fail-open (EC-031 in BC-2.03.001)

**Forbidden Dependencies:**
- `detect()` MUST NOT perform any I/O (no `std::fs` calls, no `std::process` calls)
- `ClaudeCodeModule` MUST NOT write to any Claude Code files (DI-007)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| directories | 6 | `BaseDirs::new()` for home directory resolution |
| which | (add) | `which::which("claude")` for preflight |
| async-trait | 0.1 | `#[async_trait]` on EngineModule impl |
| temp-env | 0.3 | Test: unset home env vars (dev-dependency) |
| tracing | 0.1 | E-ENG-001 log on HomeUnresolvable |

## File Structure Requirements

Files to create:
- `monocle-core/src/engine/claude.rs` — `ClaudeCodeModule` implementation
- `monocle-core/src/engine/mod.rs` — module declaration + re-exports
- `monocle-runtime/tests/engine_module_claude.rs` — detect() integration tests
- `monocle-runtime/tests/engine_module_home_unresolvable.rs` — VP-021 tests

Files to modify:
- `monocle-core/src/lib.rs` — engine module is now `pub mod engine` (sub-module)
- `monocle-core/Cargo.toml` — add `which = "^4"` (or latest stable)
- `monocle-runtime/Cargo.toml` — confirm `temp-env` dev-dependency present
