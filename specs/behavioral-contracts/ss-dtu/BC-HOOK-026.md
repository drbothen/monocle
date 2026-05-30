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

# BC-HOOK-026: No Producer-Side State — Hook Discovery Is Stateless Per Invocation

## Description

Each hook invocation is a fresh `node -e "..."` process. The `srvPort` and `srvToken`
variables are declared as `let srvPort=null,srvToken=null;` at the top of each
invocation — there is no cross-invocation caching, no file-based cache, no shared
memory. The only persistent state is the filesystem lock files.

## Preconditions

1. Multiple hook invocations occur over time.

## Postconditions

1. Each hook invocation starts with `srvPort = null` and `srvToken = null`.
2. Each invocation independently re-scans the lock file directory.
3. No invocation leaves any persistent state that affects the next invocation (other than the lock files themselves, which are managed by the daemon).

## Invariants

1. The hook process is a fresh node subprocess launched by Claude Code for each hook event. There is no long-lived hook agent/process.
2. Statelessness is what enables restart resilience (BC-HOOK-025): when the daemon restarts, the next invocation naturally finds the new lock file.
3. The hook process inherits the Claude Code subprocess environment but reads no monocle-specific env vars (beyond `$HOME`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 1000 concurrent PreToolUse invocations | Each is an independent process; no shared state contention |
| EC-002 | Invocation A finds port 7860; invocation B starts 1ms later, daemon restarts, B finds port 7861 | A and B independently resolved their own ports; no conflict |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Two sequential invocations with same lock file | Both POST to same port (independent re-discovery yields same result) | happy-path |
| Restart between two invocations | Each invocation independently resolves its own port | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone invocations are stateless (no cross-invocation caching) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — stateless per-invocation discovery is the mechanism that keeps hook connectivity working across daemon lifecycle events (restart, crash) without requiring the hooks-settings.json to be re-issued |
| L2 Domain Invariants | DI-002 (lock file precondition — stateless discovery means every hook invocation independently verifies DI-002 by scanning for a valid live lock file) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-026 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:26-27 (`let srvPort=null,srvToken=null;` — re-initialized every invocation) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-026 (gene-source: deep-hooks-r1 §7 BC-HOOK-026) |
| Test name | test_BC_HOOK_026_stateless_per_invocation_discovery |

## Related BCs

- [BC-HOOK-013] — composes with: BC-HOOK-013 covers the per-invocation scan; this BC covers the statelessness property of that scan
- [BC-HOOK-025] — composes with: BC-HOOK-025 covers the restart resilience enabled by this statelessness

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:26-27 (`let srvPort=null,srvToken=null;` — per-invocation initialization).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
