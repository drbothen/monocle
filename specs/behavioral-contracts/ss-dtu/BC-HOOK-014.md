---
document_type: behavioral-contract
level: L3
version: "1.0.1"
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

# BC-HOOK-014: Lock File Path Is Hardcoded to MONOCLE_RUNTIME_DIR (Not Env-Var Overridable in JS)

## Description

The hook inline JS discovers lock files from the monocle runtime directory
(`~/.monocle/run/` or the `MONOCLE_RUNTIME_DIR`-derived path). Critically, the
Go/Rust side of monocle reads the `MONOCLE_RUNTIME_DIR` environment variable to
override the lock file location, but the inline JS hook commands do NOT. In test
isolation, this creates an asymmetry: the Rust daemon writes its lock file to the
env-overridden dir, but if Claude Code runs the hook script in a subprocess, that
subprocess scans the non-overridden path.

This is a known porting consideration inherited from the gene source
(P1 finding in hooks-r1: `LAZYCLAUDE_IDE_DIR` env var honored by Go side but NOT
by hook JS). For monocle, this means integration tests that use `MONOCLE_RUNTIME_DIR`
to redirect the daemon's lock file CANNOT exercise the actual hook JS → daemon flow
end-to-end unless the JS also reads the env var.

## Preconditions

1. Hook JS is executing as part of a Claude Code subprocess.
2. `MONOCLE_RUNTIME_DIR` may or may not be set in the environment.

## Postconditions

1. The hook JS reads `os.homedir()` to construct the lock file directory path.
2. The lock file directory is `<home>/.monocle/run/` — the env var `MONOCLE_RUNTIME_DIR` is NOT consulted by the JS.
3. If the daemon writes its lock file to a different directory (via `MONOCLE_RUNTIME_DIR`), the hook JS will NOT find it.

## Invariants

1. The hook JS has no `process.env` reads (beyond `os.homedir()` which uses `$HOME`/$`USERPROFILE`).
2. The env-var asymmetry is a known limitation: in production, the daemon always writes to `~/.monocle/run/` so the asymmetry doesn't matter. In isolated tests, the lock file must be at the default path or the hook path must be patched.
3. The DTU clone SHOULD add `MONOCLE_RUNTIME_DIR` support to the inline JS to close this asymmetry. This is a monocle improvement over the gene source.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Production deployment; `MONOCLE_RUNTIME_DIR` not set | Hook scans `~/.monocle/run/`; daemon writes to `~/.monocle/run/`; match — works |
| EC-002 | CI test with `MONOCLE_RUNTIME_DIR=/tmp/monocle-test`; daemon writes lock to override dir | Hook JS scans `~/.monocle/run/`; daemon lock is in `/tmp/monocle-test/`; hook cannot find daemon — integration test failure |
| EC-003 | Monocle improvement: inline JS reads `process.env.MONOCLE_RUNTIME_DIR` | Asymmetry closed; CI test works correctly |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Production path (no env override) | Hook finds daemon at `~/.monocle/run/<port>.lock` | happy-path |
| CI with MONOCLE_RUNTIME_DIR; JS patched to read env | Hook finds daemon at `$MONOCLE_RUNTIME_DIR/<port>.lock` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone inline JS uses MONOCLE_RUNTIME_DIR if set, falling back to ~/.monocle/run/ | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the lock file path used by hook scripts is fundamental to hook event ingestion; the env-var asymmetry affects test isolation of the daemon lifecycle |
| L2 Domain Invariants | DI-002 (lock file precondition — the hook JS can only satisfy DI-002 if the lock file path is consistent between the daemon (writer) and the hook JS (reader)) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-014 (P1 finding: LAZYCLAUDE_IDE_DIR asymmetry) |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:13-14 (`const home=require('os').homedir(); const lockDir=path.join(home,'.claude','ide')`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-014 (gene-source: deep-hooks-r1 §5 BC-HOOK-014) |
| Test name | test_BC_HOOK_014_lock_path_monocle_runtime_dir_env_var |

## Related BCs

- [BC-HOOK-013] — depends on: BC-HOOK-013 covers the lock file scan algorithm; this BC covers the scan base directory
- [BC-HOOK-029] — composes with: BC-HOOK-029 covers the hook JS's minimal env-var footprint

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:13-14 (hardcoded `path.join(home,'.claude','ide')`; no env-var read).
- Monocle improvement note: inline JS should read `process.env.MONOCLE_RUNTIME_DIR` to close the asymmetry identified in hooks-r1 P1 finding.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
