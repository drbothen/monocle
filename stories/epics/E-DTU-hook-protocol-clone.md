---
document_type: epic
epic_id: EPIC-DTU
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
subsystems: []
capabilities: [CAP-001]
behavioral_contracts: []
verification_properties: []
tdd_mode: facade
---

# EPIC-DTU: Digital Twin Universe — Claude Code Hook Protocol Clone

## Purpose

Build the `dtu-claude-code-hooks-v1` behavioral clone of the Claude Code 5-endpoint
hook protocol. This clone is required for Phase 4 holdout evaluation (NFR-011: DTU
fidelity ≥0.95) and for integration testing without a live Claude Code instance.

The clone replicates the auth behavior that Claude Code hook scripts exhibit in practice:
sending `X-Claude-Code-Ide-Authorization: <raw-64-hex>` (no prefix) as documented in
BC-HOOK-016 and ADR-0005. This exercises the daemon's compatibility alias code path.

## Success Criteria

- Clone achieves ≥0.95 fidelity against fixture corpus at `tests/fixtures/dtu/claude-code-hook-2x/`
- All 5 endpoints from dtu-assessment.md endpoint matrix are implemented
- Clone sends `X-Claude-Code-Ide-Authorization` (alias path) matching real Claude Code behavior
- Docker packaging works for CI portability
- NFR-011 validated by `dtu-validator` agent

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-DTU-001 | Claude Code Hook Protocol DTU Clone (L3 Behavioral) | 3 | Wave 1 | — |

## Architecture Scope

- Clone implementation: `dtu-clones/claude-code-hooks-v1/` (Docker container)
- Reference: `specs/dtu-assessment.md` v1.7.5 — endpoint matrix (monocle-canonical column)
- Reference: `architecture/SS-core-types-and-abi.md` v1.2.13 §Non-Exhaustive Inner Structs
