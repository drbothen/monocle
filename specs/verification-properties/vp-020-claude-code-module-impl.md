---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.03.002
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

# VP-020: `ClaudeCodeModule::detect` Strict Basename Match; Cmdline Ignored

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-ENGINE-002 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

`ClaudeCodeModule::detect(&snapshot)` returns `true` iff `snapshot.exe_path`
is `Some(p)` AND `p.file_name() == Some("claude") || p.file_name() ==
Some("claude.js")`. The method NEVER consults `snapshot.cmdline` for
identification.

## Source Contract

- **BC:** BC-2.03.002 — `ClaudeCodeModule::detect` Strict Basename Match.
- **Postcondition/Invariant:** BC-2.03.002 §Postcondition asserting
  strict basename match on `exe_path.file_name()` for exactly the set
  `{"claude", "claude.js"}`; `cmdline` argument ignored regardless of
  content; absent `exe_path` (`None`) always yields `false`.
- **Traces to (historical):** BC-ENGINE-002 (SS-engine-module.md
  §Behavioral Contracts; PRD v1.25 §BC-ENGINE-002 Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test | Bounded — 6 canonical probes | Truth table over `{exe_path, cmdline}` Cartesian product with diagnostic basenames covering prefix, contains, separator, and `None` regressions |

## Mechanism

Integration test (harness located at
`monocle-runtime/tests/engine_module_claude_detect.rs` — files in
`<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test Type
column labels this BC `Unit` referring to conceptual scope, but the
harness layout is cargo-integration per file location). The harness
constructs `ProcessSnapshot` instances per the 6-probe truth table and
asserts `module.detect(&snapshot)` matches the expected boolean for each.

## Pre-conditions

- `ProcessSnapshot::new(pid, exe_path, cmdline, start_time_secs)`
  constructor is available (per F-R26-adv-1 fix, v1.1.7).
- `ClaudeCodeModule::new("http://127.0.0.1:7891".into())` constructs a
  module.

## Post-conditions (per probe)

| Probe | exe_path | cmdline | Expected `detect()` |
|-------|----------|---------|---------------------|
| (a) | `Some("/usr/local/bin/claude")` | `vec![]` | `true` |
| (b) | `Some("/usr/local/bin/claude-squad")` | `vec![]` | `false` |
| (c) | `None` | `vec!["claude".to_string()]` | `false` (exe_path=None regardless of cmdline) |
| (d) | `Some("/opt/anthropic/claude.js")` | `vec![]` | `true` |
| (e) | `Some("/usr/local/bin/claudio")` | `vec!["claude", "--debug"]` | `false` |
| (f) | `Some("/home/x/bin/claude-code-router")` | `vec![]` | `false` |

## Counter-examples

1. `detect` uses `cmdline[0].contains("claude")` — probe (c) returns
   `true`; the integration test must assert `false`.
2. `detect` uses `exe_path.starts_with("/usr/local/bin/claude")` (prefix
   match, not basename) — probe (b) returns `true`; the integration test
   asserts `false`.
3. `detect` uses `exe_path.contains("claude")` — probes (b), (e), (f)
   all return `true`; the integration test asserts `false` for each.

## Probe Matrix

| Probe | Snapshot | Expected | Anti-regression target |
|-------|----------|----------|-------------------------|
| 20.a | `claude` basename | `true` | Baseline |
| 20.b | `claude-squad` basename | `false` | Prefix-match regression (e.g., `starts_with`) |
| 20.c | `exe_path: None`, cmdline contains "claude" | `false` | Cmdline-fallback regression |
| 20.d | `claude.js` basename | `true` | Baseline (js variant) |
| 20.e | `claudio` basename + cmdline contains "claude" | `false` | Substring-match + cmdline-fallback regression |
| 20.f | `claude-code-router` basename | `false` | Substring-match regression |

## Harness Location

- `monocle-runtime/tests/engine_module_claude_detect.rs` (integration test)
- Test name: `test_BC_ENGINE_002_claude_code_module_strict_basename_detect`
  (per PRD v1.25 §BC-ENGINE-002, Verification subsection — to be
  migrated to `test_BC_2_03_002_claude_code_module_strict_basename_detect`
  post BC renumber propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-ENGINE-002 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-03/BC-2.03.002.md`.
- Architecture: `architecture/SS-engine-module.md` §Behavioral Contracts.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.03.002 (Dispatch 4 commit 1030c65).
- Cross-VP: VP-019 (`EngineModule` trait surface that `detect` belongs to);
  VP-021 (env-isolation sibling — shared `monocle-runtime/tests/`
  isolation pattern per monolithic line 853).
