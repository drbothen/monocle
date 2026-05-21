---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-20T21:00:00Z
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

# BC-HOOK-035: Lock File Read Errors and JSON Parse Errors Are Silently Skipped

## Description

When iterating over lock files, read errors (permissions, file deleted between
`readdirSync` and `readFileSync`) and JSON parse errors are caught by an inner
`try{}catch{}` and silently skipped. The enumeration continues to the next file.
No error logging, no stderr output. The hook behaves as if the problematic lock
file does not exist.

## Preconditions

1. The lock file directory exists and contains lock files.
2. One or more lock files are unreadable or contain malformed JSON.

## Postconditions

1. `JSON.parse(fs.readFileSync(path.join(lockDir, f), 'utf8'))` throws for the problematic file.
2. Inner `catch{}` catches the exception.
3. The loop continues to the next file.
4. No error is logged or emitted.
5. The `best` variable is updated only by lock files that successfully parse.

## Invariants

1. Lock file read failures are a normal operational condition (file may have been removed between `readdirSync` and `readFileSync`).
2. Silent skipping is the correct behavior: the hook should not fail on stale or corrupt lock files.
3. The lock file's `authToken` is only read after successful parse, so a corrupt token is impossible — either the parse succeeds and the token is valid, or the parse fails and the file is skipped.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Lock file deleted after `readdirSync` but before `readFileSync` | `readFileSync` throws ENOENT; inner catch fires; file skipped |
| EC-002 | Lock file readable but content is `{"pid": null}` (malformed authToken) | Parse succeeds; `lk.pid = null`; `process.kill(null, 0)` throws; inner catch fires; file skipped |
| EC-003 | Lock file is 0 bytes | `JSON.parse('')` throws; inner catch fires; file skipped |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Mix of valid and corrupt lock files | Valid lock selected; corrupt lock skipped | edge-case |
| Lock file deleted mid-scan | ENOENT caught; enumeration continues | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone lock file scan silently skips read errors and parse errors | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — silent skipping of corrupt lock files is a defensive lifecycle property that prevents hook process failure from interfering with Claude Code operation |
| L2 Domain Invariants | DI-002 (lock file precondition — corrupt or unreadable lock files are treated as absent; DI-002 requires a VALID lock file; an invalid one is equivalent to no lock file for this BC's purposes) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-035 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:16-20 (inner `try{...const lk = JSON.parse(...)...}catch{}` wrapping the per-file processing) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-035 (gene-source: deep-hooks-r2 §2 BC-HOOK-035) |
| Test name | test_BC_HOOK_035_lock_file_read_errors_silently_skipped |

## Related BCs

- [BC-HOOK-013] — depends on: BC-HOOK-013 covers the lock file scan loop; this BC covers the error handling within that loop

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:16-20 (inner `try{...}catch{}` wrapping `readFileSync` + `JSON.parse` + `process.kill`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
