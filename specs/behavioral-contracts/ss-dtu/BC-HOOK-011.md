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

# BC-HOOK-011: Hooks-Settings.json Is Never Cleaned Up by WriteHooksSettingsFile

## Description

The `WriteHooksSettingsFile` function is write-only: it creates or overwrites the
file but never registers a cleanup, never calls `os.Remove`, and never schedules
deferred deletion. The file persists on disk across monocle runs until the OS
cleans the temp directory. This is a deliberate design: the content is deterministic
and stable, so the file is treated as a build artifact, not ephemeral state.

## Preconditions

1. `WriteHooksSettingsFile` has been called at least once.

## Postconditions

1. No cleanup of `<runtimeDir>/hooks-settings.json` occurs within `WriteHooksSettingsFile`.
2. No `defer os.Remove(path)` is registered.
3. When the monocle process exits (clean or crash), the file persists on disk.
4. When monocle restarts, the file is overwritten with fresh content (same content in practice).

## Invariants

1. The file is stable between restarts (content is deterministic, not session-specific).
2. The file is effectively a build cache: expensive to generate only if the hook commands change, which they don't during normal operation.
3. A test harness that relies on file absence after a session MUST explicitly delete the file — it will NOT be cleaned up automatically.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Monocle crashes mid-write | File may be truncated/empty; on restart, WriteHooksSettingsFile overwrites it with valid content |
| EC-002 | OS temp dir is cleaned by the OS (e.g., tmpwatch) | File is deleted; next monocle start recreates it |
| EC-003 | Test harness checks for file absence after session stop | Test must explicitly `fs::remove_file` — file persists by design |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Stop monocle session | File still exists at `<runtimeDir>/hooks-settings.json` | lint |
| Restart monocle | File is overwritten (not deleted and recreated) | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone does not delete hooks-settings.json on session stop | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the persistence behavior of hooks-settings.json is a lifecycle decision; the file's non-cleanup on stop is an intentional lifecycle property |
| L2 Domain Invariants | None directly (no-cleanup is an implementation detail) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-011 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:49-75 (entire `WriteHooksSettingsFile` — no cleanup call) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-011 (gene-source: deep-hooks-r1 §4 BC-HOOK-011) |
| Test name | test_BC_HOOK_011_settings_file_persists_after_session_stop |

## Related BCs

- [BC-HOOK-009] — depends on: BC-HOOK-009 covers the file path and mode; this BC covers the lifecycle persistence
- [BC-HOOK-039] — composes with: BC-HOOK-039 covers the atomicity risk during overwrites

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:49-75 (full `WriteHooksSettingsFile` — no `os.Remove` call present anywhere).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
