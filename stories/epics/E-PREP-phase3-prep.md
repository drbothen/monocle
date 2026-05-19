---
document_type: epic
epic_id: EPIC-PREP
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
subsystems: []
capabilities: []
behavioral_contracts: []
verification_properties: []
---

# EPIC-PREP: Phase 3 Pre-Implementation Preparation

## Purpose

Execute the Phase-3-prep mechanical sweep mandated by tech-debt-register.md
TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE. This epic consists of a single
Wave 0 (blocking) story that must complete before Phase 3 TDD implementation
can begin. It is contingent on vsdd-factory upstream spec-kit-mcp rc.19+
shipping.

## Success Criteria

- `spec_kit_verify_invariants(scope="all")` returns zero violations against monocle `.factory/`
- `spec_kit_bump_artifact()` cascade-tail closes any remaining PRD ↔ VP-INDEX residual
- POL-29 / SE-22 v1/v2 prose rules migrated to schema-enforced invariants in spec-kit schema
- Human approval received confirming spec-kit-mcp rc.19+ is available

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-PHASE-3-PREP | spec-kit-mcp Integration: Phase-3 Mechanical Sweep | 3 | Wave 0 | vsdd-factory rc.19+ (external) |

## Dependency Note

This epic blocks Phase 3 dispatch. It does NOT block any Phase 2 story.
Wave 0 = pre-Phase-3 gate, not Wave 1 of Phase 2 implementation.
