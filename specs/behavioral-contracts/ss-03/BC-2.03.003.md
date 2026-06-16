---
document_type: behavioral-contract
level: L3
version: "1.0.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:12:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "220cb6e"
traces_to: prd.md
origin: greenfield
subsystem: SS-03
capability: CAP-003
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.03.003: HomeUnresolvable Error Contract

## Description

When `directories::BaseDirs::new()` returns `None` (platform home directory is
unresolvable), both `ClaudeCodeModule::metadata()` and `ClaudeCodeModule::enrich()` must
return `Err(EngineMetadataError::HomeUnresolvable)`. Neither method may substitute a
relative path or hardcoded fallback. Daemon initialization must propagate this error and
surface a diagnostic. Env-isolation in tests uses `temp-env ^0.3` (with
`features = ["async_closure"]`), not `std::env::remove_var` (unsafe in multi-threaded
harnesses).

## Preconditions

1. `ClaudeCodeModule` is instantiated via `ClaudeCodeModule::new("http://127.0.0.1:7891".into())`.
2. Platform home directory resolution fails: `directories::BaseDirs::new()` returns `None`. This is induced in tests by unsetting `HOME`, `USERPROFILE`, `HOMEDRIVE`, and `HOMEPATH` using `temp-env ^0.3` with `features = ["async_closure"]`.

## Postconditions

1. `ClaudeCodeModule::metadata()` returns `Err(EngineMetadataError::HomeUnresolvable)`.
2. `ClaudeCodeModule::enrich()` returns `Err(EngineMetadataError::HomeUnresolvable)`.
3. Neither method substitutes a relative path (e.g., `.claude`) or a hardcoded fallback path when `BaseDirs::new()` returns `None`.
4. The daemon initialization code that calls `metadata()` at startup MUST propagate this error and surface a diagnostic message rather than silently continuing with a wrong path.

## Invariants

1. `temp-env ^0.3` (with `features = ["async_closure"]`) is the env-isolation strategy — NOT `std::env::remove_var` (unsafe in multi-threaded test harnesses) and NOT `#[serial]` (mitigates race but doesn't guarantee cleanup on panic).
2. `metadata()` (synchronous) and `enrich()` (async) require different `temp-env` wrappers: `temp_env::with_vars` for sync and `temp_env::async_with_vars` for async. They MUST NOT be co-located in the same `with_vars` call.
3. The four env vars to clear: `HOME`, `USERPROFILE`, `HOMEDRIVE`, `HOMEPATH`. NOT `XDG_*` variables (not consulted by `BaseDirs::home_dir()`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-036 | Windows CI runner has a registered user SID — `BaseDirs::new()` may succeed via `SHGetKnownFolderPath` even with all four env vars cleared | Test is best-effort for the `None` path on Windows CI; contract is fully deterministic on Linux/macOS |
| EC-037 | `enrich()` called with a `ProcessSnapshot` that has `working_dir: None` when home IS resolvable | Separate from the `HomeUnresolvable` error path; if `BaseDirs::new()` succeeds, `enrich()` returns `Ok(EnrichedSession)` with `transcript_path: None` |

## Canonical Test Vectors

| Scenario | Input | Expected Output | Category |
|----------|-------|----------------|----------|
| metadata() with HOME unset | `temp_env::with_vars([("HOME", None::<&str>), ("USERPROFILE", None), ("HOMEDRIVE", None), ("HOMEPATH", None)], ...)` | `Err(EngineMetadataError::HomeUnresolvable)` | happy-path |
| enrich() with HOME unset | `temp_env::async_with_vars([("HOME", None::<&str>), ...], async { module.enrich(&snapshot).await })` | `Err(EngineMetadataError::HomeUnresolvable)` | happy-path |
| metadata() with HOME set | Normal environment | `Ok(EngineMetadata { config_paths: [~/.claude, ~/.claude.json], ... })` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-021 | Sync half of test uses `temp_env::with_vars`; async half uses `temp_env::async_with_vars` in a separate `#[tokio::test]`; both return `Err(HomeUnresolvable)` when all 4 home env vars are cleared | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC governs the no-silent-fallback error behavior of the ClaudeCodeModule adapter component of CAP-003 |
| L2 Domain Invariants | DI-006 (every EngineModule implementation must be stateless — metadata() and enrich() are non-detect methods, but DI-006's no-I/O and no-state-mutation constraints on the EngineModule interface require that error paths also be stateless: HomeUnresolvable fails fast without side effects, no retry state, no mutable shared variables); DI-007 (monocle must not write to any file owned by a harness or factory workflow system — HomeUnresolvable prevents incorrect path substitution that could cause metadata() or enrich() to write to an unintended location; by failing fast with a diagnostic, no file write is attempted with a potentially wrong path) |
| Architecture Module | monocle-core (EngineModule trait, EnrichedSession, HookEvent types); monocle-runtime (ClaudeCodeModule implementation — monocle-runtime/src/engine/claude_code.rs) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module.md v1.1.27 §Behavioral Contracts BC-ENGINE-002-ERR |
| CLAUDE.md SOUL | SOUL #4 (no silent fallback for unresolvable platform home directory) |
| Dev Dependency | `temp-env = { version = "^0.3", features = ["async_closure"] }` in `monocle-runtime` `[dev-dependencies]` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-ENGINE-002-ERR |
| Test name | test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich |

## Related BCs (Recommended)

- [BC-2.03.001] — depends on: HomeUnresolvable error is defined as the EngineMetadataError variant specified in the EngineModule trait
- [BC-2.03.002] — composes with: this error contract applies to the ClaudeCodeModule implementation defined in BC-2.03.002

## Architecture Anchors (Recommended)

- `architecture/SS-engine-module.md#behavioral-contracts` — BC-ENGINE-002-ERR, temp-env isolation strategy, four env vars to clear

## Story Anchor (Recommended)

S-TBD — Implement HomeUnresolvable error path with temp-env test isolation (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-021-home-unresolvable-error.md` — VP-021 HomeUnresolvable integration test (sync + async halves)

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-003 per ARCH-INDEX is authoritative source`
  - After: `DI-006 ... ; DI-007 ...`
  - DI-006 mapping: metadata() and enrich() fail-fast error returns are themselves stateless — no retry state, no mutable side effects. The Err(HomeUnresolvable) return is a pure computation on the absence of env vars. DI-007 mapping: the fail-fast prevents a potential write to a wrong path — no file write ever occurs on this error path, directly upholding DI-007.
- F-R105-9 (SE-17c-d body-scope grep): Architecture Source row references `§Behavioral Contracts BC-ENGINE-002-ERR` — this is a section heading reference within SS-engine-module.md, not a stale BC cross-reference. 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T12:00:00Z (v1.0).

## §Trace v1.0.2

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.1.15 → v1.1.20** (2026-05-18T05:20:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.1.15 → v1.1.20); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-engine-module.md v1.1.15 §Behavioral Contracts BC-ENGINE-002-ERR`
  - SE-17f AFTER: `SS-engine-module.md v1.1.20 §Behavioral Contracts BC-ENGINE-002-ERR`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:20:00Z > prior 2026-05-17T18:00:00Z (v1.0.1). ARITHMETICALLY TRUE: 2026-05-18T05:20:00Z > 2026-05-17T18:00:00Z PASS.

## §Trace v1.0.3

**GAP-PHASE2-R06-3 closure — Architecture Module pin updated per ARCH-INDEX v1.0.11 cascade (trait/impl split clarification)** (2026-05-19T12:12:00Z):
- GAP-PHASE2-R06-3: ARCH-INDEX v1.0.10 → v1.0.11 corrected SS-03 split (EngineModule trait in monocle-core, ClaudeCodeModule implementation in monocle-runtime). BC Architecture Module cell still read pre-correction text `monocle-core (EngineModule trait, ClaudeCodeModule adapter)`.
  - SE-17f BEFORE: `monocle-core (EngineModule trait, ClaudeCodeModule adapter) per ARCH-INDEX Subsystem Registry SS-03`
  - SE-17f AFTER: `monocle-core (EngineModule trait, EnrichedSession, HookEvent types); monocle-runtime (ClaudeCodeModule implementation — monocle-runtime/src/engine/claude_code.rs) per ARCH-INDEX Subsystem Registry SS-03`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:12:00Z > prior 2026-05-18T05:20:00Z (v1.0.2). ARITHMETICALLY TRUE: PASS.

## §Trace v1.0.4

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-engine-module.md v1.1.20 → v1.1.26 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-engine-module.md v1.1.20 §Behavioral Contracts BC-ENGINE-002-ERR` → `SS-engine-module.md v1.1.26 §Behavioral Contracts BC-ENGINE-002-ERR`.
- Plain version-pin refresh. No substantive content propagation required — §Behavioral Contracts section structure unchanged between v1.1.20 and v1.1.26.
- SE-16d monotonicity PASS: 2026-05-29T00:00:00Z > prior 2026-05-19T12:12:00Z (v1.0.3). ARITHMETICALLY TRUE: PASS.
