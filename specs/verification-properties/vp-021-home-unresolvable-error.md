---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
traces_to: prd.md
source_bc: BC-2.03.003
module: monocle-runtime
proof_method: integration-test
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-021: `metadata`/`enrich` Return `HomeUnresolvable` with All Four Home-Env Vars Unset

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-ENGINE-002-ERR (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

When `HOME`, `USERPROFILE`, `HOMEDRIVE`, and `HOMEPATH` are all unset
(set to `None::<&str>` via `temp_env::with_vars` / `async_with_vars`),
`ClaudeCodeModule::metadata()` and `ClaudeCodeModule::enrich(&snapshot)`
both return `Err(EngineMetadataError::HomeUnresolvable)`. The
implementation MUST NOT substitute a relative-path default, a
current-directory fallback, or any non-`HomeUnresolvable` error path.

## Source Contract

- **BC:** BC-2.03.003 — `EngineModule::metadata`/`enrich` `HomeUnresolvable`
  Error Path.
- **Postcondition/Invariant:** BC-2.03.003 §Postconditions asserting
  `metadata()` (sync) and `enrich()` (async) both return
  `Err(EngineMetadataError::HomeUnresolvable)` when all four home-env
  vars are unset; no relative-path / current-directory fallback
  substitution permitted; sync/async test halves share environment
  isolation via `temp-env ^0.3` per SS-deps-pin-manifest.md v1.1.15 pin.
- **Traces to (historical):** BC-ENGINE-002-ERR (SS-engine-module.md
  §Behavioral Contracts; PRD v1.25 §BC-ENGINE-002-ERR Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test + `temp-env ^0.3` | Bounded — 2 surface (sync + async) × 4 env-vars cleared | Sync `metadata()` + async `enrich()` both return `Err(HomeUnresolvable)`; env-isolation via `temp_env::with_vars` / `async_with_vars` RAII cleanup |
| Semgrep (supplementary) | semgrep | N/A — static | `monocle-no-raw-env-mutation-in-tests` rule fails harness using `std::env::set_var` / `remove_var` directly |

## Mechanism

Integration test (harness located at
`monocle-runtime/tests/engine_module_home_unresolvable.rs` with
`temp-env ^0.3` env-isolation per SS-deps-pin-manifest.md v1.1.15 pin;
files in `<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM
Test Type column labels this BC `Unit (env-isolation)` referring to
conceptual scope, but the harness layout is cargo-integration per file
location). The harness has two halves:

- **Sync half:** `temp_env::with_vars([("HOME", None), ("USERPROFILE",
  None), ("HOMEDRIVE", None), ("HOMEPATH", None)], || { module.metadata()
  })` — asserts `is_err()` + `matches!(_, HomeUnresolvable)`.
- **Async half:** `temp_env::async_with_vars([...], async { module.enrich(&snapshot).await }).await`
  — same assertion pattern under the async runtime.

## Pre-conditions

- `temp-env = { version = "^0.3", features = ["async_closure"] }` in
  `[dev-dependencies]`.
- Test does NOT use `std::env::set_var` / `remove_var` directly; only
  `temp_env::with_vars` / `temp_env::async_with_vars` (RAII cleanup safe
  under panic and multi-threaded harness).

## Post-conditions

1. Sync half: inside `temp_env::with_vars([("HOME", None::<&str>),
   ("USERPROFILE", None::<&str>), ("HOMEDRIVE", None::<&str>),
   ("HOMEPATH", None::<&str>)], || { ... })`:
   - `module.metadata().is_err()` is `true`;
   - `matches!(module.metadata().unwrap_err(),
     EngineMetadataError::HomeUnresolvable)` is `true`.
2. Async half: inside `temp_env::async_with_vars([...], async { ... }).await`:
   - `module.enrich(&snapshot).await.is_err()` is `true`;
   - `matches!(module.enrich(&snapshot).await.unwrap_err(),
     EngineMetadataError::HomeUnresolvable)` is `true`.
3. Test passes on Linux and macOS CI runners deterministically. On
   Windows CI the test is best-effort (Windows may resolve `home_dir()`
   via `FOLDERID_Profile` regardless of env-var state); the Linux/macOS
   gates are the canonical assertion.

## Counter-examples

1. `metadata()` returns `Ok(EngineMetadata { home_dir: PathBuf::from("."), ... })`
   substituting `.` for unresolvable home — fails the `is_err`
   assertion.
2. `metadata()` returns `Err(EngineMetadataError::Io(...))` instead of
   `HomeUnresolvable` — fails the `matches!` assertion.
3. Test uses `std::env::remove_var` instead of `temp_env::with_vars` —
   the audit (a semgrep rule in SS-conventions-anti-patterns.md
   §Semgrep Rules `monocle-no-raw-env-mutation-in-tests`) fails the
   harness.
4. Test omits any of the four required env-vars (e.g., only clears
   `HOME`) — Windows may resolve via `USERPROFILE` even on Linux
   containers with `wine`-style env shimming; the test must clear all
   four.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 21.a | `with_vars(all four = None)`, call `metadata()` | `Err(HomeUnresolvable)` |
| 21.b | `async_with_vars(all four = None)`, call `enrich(&snapshot).await` | `Err(HomeUnresolvable)` |
| 21.c | Mutation: `metadata()` returns `Ok(home_dir=".")` | Probe 21.a fails (`is_err` returns `false`) |
| 21.d | Mutation: `metadata()` returns `Err(Io(...))` | Probe 21.a fails (`matches!` does not match) |
| 21.e | Anti-pattern: test uses `std::env::remove_var` | Semgrep rule `monocle-no-raw-env-mutation-in-tests` fires |
| 21.f | Test clears only `HOME` (3 vars left set) | Windows may resolve via `USERPROFILE`; Linux test is brittle |

## Test Design Rationale (PRD v1.2 §Trace v1.2 adjudication)

The test name identifies the two behavioral surfaces under contract —
`metadata()` and `enrich()` — both of which must return
`Err(EngineMetadataError::HomeUnresolvable)` when all four home-env
vars are unset. The internal use of `temp_env::with_vars` for
`metadata()` (sync) and `temp_env::async_with_vars` for `enrich()`
(async) is a test-implementation strategy, not the behavioral
discriminator; per the PRD v1.2 adjudication "test names should
describe what is verified, not how the test harness is structured."
The property and post-conditions above remain valid; only the naming
convention was clarified by the F-R63 adjudication.

## Harness Location

- `monocle-runtime/tests/engine_module_home_unresolvable.rs` (integration test)
- Test name: `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich`
  (per PRD v1.25 §BC-ENGINE-002-ERR, Verification subsection — to be
  migrated to `test_BC_2_03_003_home_unresolvable_metadata_and_enrich`
  post BC renumber propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-ENGINE-002-ERR at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-03/BC-2.03.003.md`.
- Architecture: `architecture/SS-engine-module.md` §Behavioral Contracts.
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15
  (`temp-env ^0.3` pin with `async_closure` feature).
- Conventions: `architecture/SS-conventions-anti-patterns.md` §Semgrep Rules
  (`monocle-no-raw-env-mutation-in-tests`).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.03.003 (Dispatch 4 commit 1030c65).
- Cross-VP: VP-019 (trait method shapes — `metadata()` and `enrich()` typed
  `Result<_, EngineMetadataError>` returns are the no-silent-fallback
  property); VP-020 (env-isolation sibling — shared `monocle-runtime/tests/`
  isolation pattern per monolithic line 853).
