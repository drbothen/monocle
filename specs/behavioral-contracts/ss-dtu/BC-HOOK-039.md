---
document_type: behavioral-contract
level: L3
version: "1.0.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-30T00:00:00Z
phase: 1a
inputs:
  - {path: .factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r2.md, version: "r2"}
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

# BC-HOOK-039: WriteHooksSettingsFile Is Not Atomic; Torn Read Theoretically Possible

## Description

`os.WriteFile` for hooks-settings.json is NOT atomic on POSIX: it uses `open(O_WRONLY|O_CREAT|O_TRUNC)` + `write()` + `close()`. A concurrent Claude Code launch that reads the file between `O_TRUNC` (file is now empty) and `write()` (file content is written) would see an empty file. The practical risk is low because: (a) the content is deterministic and identical across writes, and (b) parallel session launches are rare.

The monocle improvement is to use `tempfile::persist` (atomic temp-then-rename) for
hooks-settings.json writes, consistent with monocle's global atomic-write policy
(SS-conventions-anti-patterns.md).

## Preconditions

1. Two monocle sessions are launched nearly simultaneously.
2. Both call `WriteHooksSettingsFile` with the same `runtimeDir`.

## Postconditions (gene-source behavior — non-atomic):

1. `os.WriteFile` performs: open (truncates) → write → close.
2. A reader between truncate and write sees an empty file.
3. Claude Code launched with an empty settings file: `--settings` flag points to empty file; Claude Code's settings parser presumably falls back to defaults; session starts without hooks.

## Postconditions (monocle improvement — atomic write via tempfile::persist):

1. `WriteHooksSettingsFile` writes to a temp file in `runtimeDir`, then renames it to `hooks-settings.json`.
2. The rename is atomic on POSIX; any concurrent reader sees either the old file or the new file, never an empty/torn file.
3. Claude Code's `--settings` flag always points to a valid JSON file.

## Invariants

1. The monocle improvement (`tempfile::persist`) is consistent with SS-conventions-anti-patterns.md's global atomic-write policy (no naked `std::fs::write` for config files).
2. The practical risk of the non-atomic write is low but non-zero in CI environments with rapid test setup.
3. The content is deterministic: torn reads produce a corrupt/empty file, not incorrect content.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Concurrent writes; reader sees empty file (torn read without atomic write) | Claude Code launches without hooks; monocle receives no events — degraded but not crashed |
| EC-002 | Concurrent writes with atomic tempfile::persist | Reader always sees a complete file; torn read impossible |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Single write (no concurrency) | hooks-settings.json contains valid JSON after write | happy-path |
| Atomic write via tempfile::persist | Reader sees complete JSON; never empty | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone WriteHooksSettingsFile uses tempfile::persist (atomic write) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — atomic hooks-settings.json writes are a daemon lifecycle correctness property; torn reads result in session launches without hook event ingestion |
| L2 Domain Invariants | DI-001 (tee invariant — a torn read that causes hooks-settings.json to be empty prevents hook injection, so no hook events reach the daemon; this violates the spirit of DI-001 for that session) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-039; SS-conventions-anti-patterns.md v1.32.4 (atomic write policy) |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:71 (`os.WriteFile` — non-atomic) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-039 (gene-source: deep-hooks-r2 §3 BC-HOOK-039; P3 finding) |
| Test name | test_BC_HOOK_039_atomic_write_tempfile_persist |

## Related BCs

- [BC-HOOK-009] — depends on: BC-HOOK-009 covers the file path and mode; this BC covers the atomicity of the write
- [BC-HOOK-010] — composes with: BC-HOOK-010 covers concurrent writes from multiple sessions; this BC covers the torn-read risk of those concurrent writes

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`
- `specs/architecture/SS-conventions-anti-patterns.md` — atomic write policy (`tempfile::persist`)

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**1.0.2** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers and time qualifiers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at authoring time. No normative content changed.

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:71 (`os.WriteFile` — non-atomic; P3 finding in r2 §3).
- Monocle improvement: use `tempfile::persist` per SS-conventions-anti-patterns.md atomic-write policy.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.

## §Trace v1.0.1

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-conventions-anti-patterns.md v1.29.5 → v1.31.1 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-conventions-anti-patterns.md v1.29.5 (atomic write policy)` → `SS-conventions-anti-patterns.md v1.31.1 (atomic write policy)`.
- Plain version-pin refresh. No substantive content propagation required — the atomic-write policy (tempfile::persist) is unchanged between v1.29.5 and v1.31.1.
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.3

**POL-11 remediation: SS-conventions-anti-patterns Architecture Source pin v1.32.3 → v1.32.4** (2026-05-30):
- Architecture Source row: `SS-conventions-anti-patterns.md v1.32.3 (atomic write policy)` → `SS-conventions-anti-patterns.md v1.32.4 (atomic write policy)` (Option 1 per ADR-0007 §Decision — active navigation pointer, not historical provenance).
- SS-conventions canonical version is v1.32.4 per `version-pin-registry.yaml`.
- Version bumped v1.0.2 → v1.0.3.
- SE-16d monotonicity: v1.0.3 timestamp 2026-05-30 >= v1.0.1 timestamp 2026-05-29. PASS.
