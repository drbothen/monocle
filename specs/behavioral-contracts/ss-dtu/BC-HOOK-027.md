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

# BC-HOOK-027: Monocle Never Writes ~/.monocle/settings.json — Hook Injection Is Via --settings Flag

## Description

monocle NEVER writes the user's `~/.monocle/settings.json` (or `~/.claude/settings.json`
in Claude Code's namespace). Hook injection is done via the `--settings <path>` CLI flag
passed to the `claude` binary when launching a session. The runtime hooks-settings.json
at `<runtimeDir>/hooks-settings.json` is the ephemeral injection vehicle; the user's
global settings file is never touched.

## Preconditions

1. A monocle session is being launched by calling the `claude` binary.

## Postconditions

1. The `claude` binary is invoked with `--settings <runtimeDir>/hooks-settings.json`.
2. `~/.monocle/settings.json` is NOT written by monocle's hook injection code.
3. `~/.claude/settings.json` is NOT written by monocle's hook injection code.
4. The user's global Claude Code settings are preserved untouched.

## Invariants

1. The `--settings` override is the sole hook injection mechanism.
2. Writing to user global settings would affect ALL Claude Code sessions, not just monocle-managed ones — this must never happen.
3. If `WriteHooksSettingsFile` errors, the session launches WITHOUT hooks (fail-soft: session starts but monocle receives no hook events).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `WriteHooksSettingsFile` fails (disk full, permission denied) | Session still launches via `claude` binary; `--settings` flag is OMITTED; no hooks received |
| EC-002 | User has existing `~/.claude/settings.json` with hooks configured | monocle's `--settings` flag overrides for this session only; global file unchanged |
| EC-003 | Parallel monocle sessions on same machine | Each session uses its own `--settings <runtimeDir>/hooks-settings.json`; global file never touched |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Session launch command | Contains `--settings <runtimeDir>/hooks-settings.json` | lint |
| Filesystem scan after session launch | `~/.claude/settings.json` unchanged | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone verifies monocle uses --settings injection, not global settings write | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the --settings injection mechanism is how hook event ingestion is bootstrapped for each managed session; the non-modification of global settings is a safety invariant of the lifecycle |
| L2 Domain Invariants | None directly (non-modification of global settings is a safety/isolation property) |
| Architecture Module | monocle-runtime (daemon binary, session launcher) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-027 |
| Gene Source | any-context-lazyclaude/internal/session/manager.go:706-709 (`sb.WriteString(" --settings "); sb.WriteString(shell.Quote(settingsFile))`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-027 (gene-source: deep-hooks-r1 §8 BC-HOOK-027) |
| Test name | test_BC_HOOK_027_settings_injection_via_flag_not_global_write |

## Related BCs

- [BC-HOOK-009] — depends on: BC-HOOK-009 covers the hooks-settings.json file creation; this BC covers how it is injected
- [BC-HOOK-028] — composes with: BC-HOOK-028 confirms there is no env-var alternative to --settings

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: manager.go:706-709 (`--settings` flag injection; no `~/.claude/settings.json` write confirmed by grep).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
