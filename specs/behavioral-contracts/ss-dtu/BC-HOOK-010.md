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

# BC-HOOK-010: Hooks-Settings.json Is Per-runtimeDir, Not Per-Session

## Description

A single hooks-settings.json file is shared by all monocle sessions launched in
the same runtime directory. The filename has no session-ID suffix, no port suffix,
and no timestamp. Repeated calls to `WriteHooksSettingsFile` with the same
`runtimeDir` overwrite the same file. All concurrent sessions in the same runtimeDir
use identical hook configurations — the hook commands are parameterless constants.

## Preconditions

1. Multiple monocle sessions are launched from the same runtime directory.

## Postconditions

1. All sessions use `<runtimeDir>/hooks-settings.json` — the same file.
2. The file content is deterministic: same hook commands every time (modulo JSON key ordering, which is alphabetical per Go's encoder and struct order per serde_json).
3. No per-session hooks-settings.json files are created.

## Invariants

1. The hooks-settings.json content is a pure function of the hook command strings — it contains no session-specific data, no port, no token.
2. Per-session naming is NOT required because the hook commands do runtime lock-file discovery (BC-HOOK-013); they don't need a static port baked in.
3. This design means a test environment that isolates sessions by `runtimeDir` MUST use different `runtimeDir` values if it needs different hook configurations.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two sessions launch concurrently, both call WriteHooksSettingsFile | Race: both write identical content; last writer wins; result is the same content either way (deterministic) |
| EC-002 | First session writes file; second session reads it before first session finishes | Torn read possible (BC-HOOK-039); content is deterministic so worst case is parse error → Claude Code launches without hooks (fail-soft) |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Launch session A and session B from same runtimeDir | One file at `<runtimeDir>/hooks-settings.json`; not two files | happy-path |
| Inspect file content after two writes | File content is byte-identical to single write | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone produces a single hooks-settings.json per runtimeDir regardless of session count | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the per-runtimeDir (not per-session) scoping of hooks-settings.json is a lifecycle design choice that simplifies the daemon's session management |
| L2 Domain Invariants | None directly (per-runtimeDir scoping is a lifecycle simplification, not a correctness invariant at the domain level) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-010 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:70 (hardcoded `"/hooks-settings.json"` filename; no session suffix) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-010 (gene-source: deep-hooks-r1 §4 BC-HOOK-010) |
| Test name | test_BC_HOOK_010_settings_file_per_runtimedir_not_per_session |

## Related BCs

- [BC-HOOK-009] — depends on: BC-HOOK-009 covers the file path and mode; this BC covers the per-runtimeDir vs per-session scope
- [BC-HOOK-039] — composes with: BC-HOOK-039 covers the non-atomic write race condition for concurrent sessions

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:70 (hardcoded `"/hooks-settings.json"` — no session suffix); config.go:32 (`RuntimeDir: os.TempDir()`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
