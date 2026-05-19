---
document_type: story
story_id: S-PHASE-3-PREP
epic_id: EPIC-PREP
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 3
wave: 0
tdd_mode: facade
priority: P0
depends_on: []
blocks: []
target_module: .factory/specs
subsystems: []
behavioral_contracts: []
verification_properties: []
external_dependency: vsdd-factory-spec-kit-mcp-rc19plus
# BC status: pending PO authorship — this is a pre-implementation mechanical sweep story.
# BCs are authored when spec-kit-mcp ships and scope is concrete. Cannot be ready until
# spec-kit-mcp rc.19+ is available and BCs can be grounded in actual tool APIs.
# Wave 0 = pre-Phase-3 gate. Does NOT block any Phase 2 story.
# Source: tech-debt-register.md TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE §Future Attachment.
inputs:
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.10"}
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.11"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
input-hash: "[live-state]"
traces_to: "Pre-Phase-3 prep: spec-kit-mcp rc.19+ mechanical sweep. Anchored to TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE §Future Attachment."
---

# S-PHASE-3-PREP: spec-kit-mcp Integration — Phase 3 Pre-Implementation Mechanical Sweep

## Narrative

As the Phase 3 pre-implementation gate, I want to run the `spec-kit-mcp` tool suite against
the monocle `.factory/` artifact set, so that the PRD ↔ VP-INDEX reverse-cascade asymptote
(TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE) is resolved by schema-enforced invariants before
Phase 3 TDD implementation begins — eliminating this class of finding permanently and allowing
all 39 prose disciplines to be migrated to spec-kit-managed invariants.

## Background

This story fulfills the §Future Attachment obligation in `tech-debt-register.md`
TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE. The residual was accepted (human-directed per
CLAUDE.md §Canonical Principle Rule 3) with a concrete future dependency:
vsdd-factory upstream `spec-kit-mcp` library shipping at rc.19+.

**Contingency:** This story is BLOCKED until vsdd-factory upstream spec-kit-mcp rc.19+ ships.
When it ships, the human must explicitly approve dispatch of this story. It does NOT block
any Phase 2 story or Wave 1/2/3 implementation work.

## Acceptance Criteria

### AC-001 (resolves TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE — spec_kit_verify_invariants)
`spec_kit_verify_invariants(scope="all")` returns zero violations when run against the
monocle `.factory/` artifact set after Phase 3 entry.

### AC-002 (resolves residual pin staleness — spec_kit_bump_artifact cascade)
`spec_kit_bump_artifact()` cascade-tail closes any remaining PRD ↔ VP-INDEX pin staleness
that persisted from the Phase 1 asymptote. Zero NORMATIVE stale pins remain after this sweep.

### AC-003 (prose rule migration — POL-29 / SE-22 to schema invariants)
POL-29 and SE-22 v1/v2 prose rules are migrated to schema-enforced invariants in the
spec-kit schema file. The adversarial review loop no longer needs to catch version-pin
staleness manually — the spec-kit pre-commit hook blocks it at write time.

### AC-004 (human approval of spec-kit-mcp rc.19+ availability)
Before dispatch of this story, the human confirms: "vsdd-factory upstream spec-kit-mcp rc.19+
is available on the release channel." This confirmation is the dispatch gate.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~700 |
| tech-debt-register.md (TD-VSDD entry) | ~800 |
| spec-kit-mcp tool documentation (when available) | ~TBD |
| monocle .factory/ artifact inventory | ~500 |
| **Total estimate** | **~2,000 + spec-kit docs** |

## Tasks

- [ ] GATE: Confirm vsdd-factory spec-kit-mcp rc.19+ is shipped and available
- [ ] Install spec-kit-mcp in monocle factory environment
- [ ] Run `spec_kit_verify_invariants(scope="all")` against monocle `.factory/`
- [ ] Run `spec_kit_bump_artifact()` for any remaining stale pins
- [ ] Migrate POL-29 prose → spec-kit invariant schema entry
- [ ] Migrate SE-22 v1/v2 prose → spec-kit invariant schema entries
- [ ] Verify pre-commit hook blocks version-pin staleness at write time
- [ ] Run adversarial review pass to confirm no new version-pin findings

## Previous Story Intelligence

N/A — Wave 0 story; precedes all Phase 3 implementation.
Dependency: vsdd-factory upstream (external dependency not under monocle's control).

## Architecture Compliance Rules

From `tech-debt-register.md` TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE §Required for Resolution:
- Target library: `vsdd-spec-kit-core` (typed graph + invariants)
- Target MCP server: `spec-kit-mcp` (agent-callable mutation tools)
- Target invariant: INV-005 (transitive closure with fixed-point iteration)
- Dispatcher pre-commit hook: `vsdd-spec-kit-validator.wasm`

**Forbidden Dependencies:**
- This story MUST NOT modify any `.factory/specs/behavioral-contracts/` BC files
- This story MUST NOT add new BCs or modify existing BC IDs

## Library & Framework Requirements

| Tool | Version | Usage |
|------|---------|-------|
| spec-kit-mcp | rc.19+ (external, vsdd-factory upstream) | Invariant verification and cascade-tail bump |

## File Structure Requirements

Files to modify:
- `.factory/specs/*.md` — pin corrections as needed by cascade-tail sweep
- `.factory/policies.yaml` — migrate POL-29 to spec-kit invariant schema
- `.factory/` — install spec-kit pre-commit hook
