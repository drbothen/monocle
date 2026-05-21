---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-20T21:00:00Z
phase: 1a
inputs:
  - {path: .factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md, version: "r1"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
input-hash: "[live-state]"
traces_to: prd.md
origin: gene-transfusion
subsystem: SS-01
capability: CAP-001
dtu_service: claude-code-hook-protocol
gene_source: any-context-lazyclaude/internal/core/config/hooks.go
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

# BC-HOOK-028: No Env-Var Alternative for Hook Injection — Only --settings Flag

## Description

There is no `CLAUDE_HOOKS`, `CLAUDE_SETTINGS_PATH`, or similar environment variable
that can be used to configure hook injection. The sole mechanism is the `--settings <path>`
CLI flag. This has been verified by exhaustive grep of the session launcher code
(manager.go:842-873): the only env vars injected into Claude Code subprocess are
`CLAUDE_CODE_AUTO_CONNECT_IDE`, `LAZYCLAUDE_SESSION_ID`, and passthrough of
`CLAUDE_CODE_OAUTH_TOKEN`, `ANTHROPIC_API_KEY`, `CLAUDE_CODE_API_KEY`.

## Preconditions

1. monocle is configuring hook injection for a new Claude Code session.

## Postconditions

1. The `--settings <runtimeDir>/hooks-settings.json` flag is the ONLY way to inject hooks.
2. No `CLAUDE_*HOOKS*` or `CLAUDE_SETTINGS*` env var is set by monocle.
3. The Claude Code subprocess environment contains no monocle-controlled hook configuration env vars.

## Invariants

1. `--settings` overrides Claude Code's default settings for this session only.
2. A future Claude Code version that adds a `CLAUDE_SETTINGS_PATH` env var would be a protocol change requiring monocle updates.
3. The env-var injection list is fixed and documented in manager.go; any additions require explicit code changes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | User sets `CLAUDE_SETTINGS_PATH` in their environment | This env var is passed through as-is (if Claude Code reads it); monocle does not control it; monocle's `--settings` flag takes precedence per Claude Code's flag-over-env convention |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Inspect Claude Code subprocess env vars set by monocle | No CLAUDE_HOOKS or CLAUDE_SETTINGS_PATH present | lint |
| Claude Code launch command | Contains `--settings` flag | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone confirms no hook-related env vars are injected; only --settings flag is used | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the exclusivity of --settings as the hook injection mechanism is a lifecycle design constraint; no env-var alternative exists |
| L2 Domain Invariants | None directly (mechanism exclusivity is an implementation invariant) |
| Architecture Module | monocle-runtime (daemon binary, session launcher) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-028 |
| Gene Source | any-context-lazyclaude/internal/session/manager.go:842-873 (`claudeEnv` function — exhaustive env var list; no CLAUDE_HOOKS or CLAUDE_SETTINGS*) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-028 (gene-source: deep-hooks-r1 §8 BC-HOOK-028) |
| Test name | test_BC_HOOK_028_no_env_var_hook_injection_alternative |

## Related BCs

- [BC-HOOK-027] — composes with: BC-HOOK-027 covers the --settings injection mechanism; this BC confirms there is no env-var alternative

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: manager.go:842-873 (`claudeEnv` — exhaustive env var injection; no `CLAUDE_HOOKS` or `CLAUDE_SETTINGS*` present).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
