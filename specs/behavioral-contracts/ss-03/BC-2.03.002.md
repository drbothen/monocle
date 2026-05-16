---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T12:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
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
| L2 Domain Invariants | N/A — no domain-spec/invariants.md exists; CAP-003 per ARCH-INDEX is authoritative source |
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

- `verification-properties/vp-020-claude-code-module-detect.md` — VP-020 strict-basename detect integration test
