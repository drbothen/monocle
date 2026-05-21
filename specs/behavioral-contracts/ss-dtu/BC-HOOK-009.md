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

# BC-HOOK-009: Hooks-Settings.json Written at runtimeDir/hooks-settings.json with Mode 0o600

## Description

The hooks-settings.json file is always written at the fixed path
`<runtimeDir>/hooks-settings.json` (a literal filename, not session-derived).
The file is created with mode `0o600` (owner-read/write only). The parent directory
`runtimeDir` is created with `os.MkdirAll` at mode `0o755` if it does not already
exist.

## Preconditions

1. `runtimeDir` is resolvable (either from environment variable or default OS temp dir).

## Postconditions

1. File path is exactly `<runtimeDir>/hooks-settings.json` — no session ID suffix, no timestamp.
2. File mode is `0o600` (rw-------).
3. Parent directory `runtimeDir` is created at mode `0o755` if absent (via `os.MkdirAll`).
4. On repeated calls with the same `runtimeDir`, the same file is overwritten (last-writer wins).
5. The file is NOT cleaned up by `WriteHooksSettingsFile` — it persists until the OS cleans temp dirs.

## Invariants

1. The filename `hooks-settings.json` is a fixed constant — it MUST NOT be varied based on port, session ID, or process ID.
2. Mode `0o600` is a security requirement: only the owner (monocle daemon user) should read the hook commands and embedded lock-discovery logic.
3. `runtimeDir` gets `0o755` (executable by others — required so Claude Code subprocess can traverse into it). The settings file itself gets `0o600`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `runtimeDir` does not exist | `os.MkdirAll(runtimeDir, 0o755)` creates it; file created successfully |
| EC-002 | `runtimeDir` exists; file already exists from previous run | File is overwritten (same content; deterministic) |
| EC-003 | `runtimeDir` is on a filesystem that doesn't support Unix permissions (e.g., FAT32 on Linux) | `WriteFile` may succeed but mode is not enforced by OS; documented as a platform limitation |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Call WriteHooksSettingsFile with a temp dir | Path ends with `/hooks-settings.json`; mode is `0o600` | happy-path |
| Call WriteHooksSettingsFile twice with same dir | Second call overwrites first; single file present | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone produces hooks-settings.json at correct path with 0o600 mode | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the hooks-settings.json file path and permissions are part of the daemon lifecycle; the daemon writes this file at startup as part of hook injection setup |
| L2 Domain Invariants | DI-002 (lock file precondition — hooks-settings.json is written alongside the lock file as part of daemon startup; both must be present before Claude Code can discover the daemon) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-009 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:66-74 (`os.MkdirAll(runtimeDir, 0o755); os.WriteFile(path, buf.Bytes(), 0o600)`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-009 (gene-source: deep-hooks-r1 §4 BC-HOOK-009) |
| Test name | test_BC_HOOK_009_settings_file_path_and_mode |

## Related BCs

- [BC-HOOK-010] — composes with: BC-HOOK-010 covers the per-runtimeDir (not per-session) scoping
- [BC-HOOK-011] — composes with: BC-HOOK-011 covers the no-cleanup persistence behavior

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:66-74 (`os.MkdirAll` + `os.WriteFile` with `0o600`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
