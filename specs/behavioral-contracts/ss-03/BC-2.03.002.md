---
document_type: behavioral-contract
level: L3
version: "1.0.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T18:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "2184d8f"
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

# Behavioral Contract BC-2.03.002: ClaudeCodeModule Implementation (Strict-Basename Detect)

## Description

`ClaudeCodeModule` is the Phase 1 concrete implementation of `EngineModule` for Claude Code.
Detection uses a STRICT basename match on the RESOLVED exe path: only `"claude"` or
`"claude.js"` are accepted as `file_name()` values. This prevents false positives from
`claude-squad`, `claudio`, or any other binary whose name contains "claude" as a prefix or
substring. `cmdline` is retained for `enrich()` but is never the primary detection signal.

## Preconditions

1. `monocle-runtime` crate compiles.
2. `ClaudeCodeModule` is defined in `monocle-runtime::engine::claude_code`.
3. A `ProcessSnapshot` is provided to `detect()`.

## Postconditions

1. `ClaudeCodeModule` implements `EngineModule`.
2. A public `ClaudeCodeModule::new(hook_base_url: String) -> Self` constructor is provided. Construction is infallible — URL validation is deferred to `preflight()`.
3. `id()` returns the string `"claude-code"` — stable, never changes.
4. `detect()` returns `true` for any process whose `exe_path.file_name()` equals `"claude"` or `"claude.js"`. STRICT basename match on the RESOLVED exe path. NOT a suffix match on `cmdline[0]`.
5. `detect()` returns `false` when `exe_path` is `None`, regardless of `cmdline` contents.

## Invariants

1. The strict-basename rule prevents false positives from: `claude-squad`, `claudio`, `claude-code-router`, or any other binary whose name contains "claude" as a prefix or substring.
2. `cmdline` is retained for `enrich()` (reading `CLAUDE_SESSION_ID`). It MUST NOT be used as the primary detection signal — `exe_path` is the canonical signal.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-032 | Process `cmdline[0]` is `"claude"` but `exe_path` resolves to `/usr/local/bin/claude-squad` | `detect()` returns `false` — `cmdline[0]` is not consulted |
| EC-033 | `exe_path` is `/usr/local/bin/claude` (no extension) | `file_name()` is `"claude"`; `detect()` returns `true` |
| EC-034 | `exe_path` is `/usr/local/bin/claude.js` (Node.js wrapper) | `file_name()` is `"claude.js"`; `detect()` returns `true` — matches the second allowed name |
| EC-035 | `exe_path` is `Some(PathBuf::from("/usr/local/bin/claude-squad"))` | `file_name()` is `"claude-squad"`; neither `"claude"` nor `"claude.js"`; `detect()` returns `false` |

## Canonical Test Vectors

| Scenario | ProcessSnapshot | Expected Output | Category |
|----------|----------------|----------------|----------|
| Real claude binary | `exe_path: Some("/usr/local/bin/claude")` | `detect() == true` | happy-path |
| claude.js wrapper | `exe_path: Some("/usr/local/bin/claude.js")` | `detect() == true` | happy-path |
| claude-squad (false positive risk) | `exe_path: Some("/usr/local/bin/claude-squad")` | `detect() == false` | edge-case |
| exe_path None | `exe_path: None, cmdline: vec!["claude"]` | `detect() == false` | edge-case |
| claudio (false positive risk) | `exe_path: Some("/usr/local/bin/claudio")` | `detect() == false` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-020 | Integration test covers all 5 detect() test vectors in the canonical table; strict-basename rule prevents all false positives | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC defines the ClaudeCodeModule, which is explicitly the Claude Code Phase 1 adapter named in CAP-003 |
| L2 Domain Invariants | DI-006 (every EngineModule implementation must be stateless with respect to process detection — detect() must not perform I/O and must not mutate shared state — ClaudeCodeModule::detect() performs a pure in-memory comparison of exe_path.file_name() against two string literals; Invariant 1 explicitly states the strict-basename rule uses no I/O and Invariant 2 prohibits cmdline as a signal to further prevent any shared-state dependency) |
| Architecture Module | monocle-core (EngineModule trait, ClaudeCodeModule adapter) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module.md v1.1.15 §Phase 1 Implementation: ClaudeCodeModule |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-ENGINE-002 |
| Test name | test_BC_ENGINE_002_claude_code_module_strict_basename_detect |

## Related BCs (Recommended)

- [BC-2.03.001] — depends on: ClaudeCodeModule implements the EngineModule trait defined here
- [BC-2.03.003] — composes with: HomeUnresolvable error behavior for metadata() and enrich() on this module
- [BC-2.03.004] — composes with: inherent methods hook_paths, spawn, preflight on this struct

## Architecture Anchors (Recommended)

- `architecture/SS-engine-module.md#phase-1-implementation-claudecodemodule` — strict-basename detect rule, cmdline non-primary-signal invariant, infallible constructor

## Story Anchor (Recommended)

S-TBD — Implement ClaudeCodeModule with strict-basename detect (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-020-claude-code-module-impl.md` — VP-020 ClaudeCodeModule implementation integration test

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-003 per ARCH-INDEX is authoritative source`
  - After: `DI-006 ...`
  - DI-006 mapping: ClaudeCodeModule::detect() is a pure function over a ProcessSnapshot struct (in-memory data). No filesystem access, no network I/O, no state mutation. The strict-basename comparison is a string equality check — the simplest possible stateless predicate.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).
