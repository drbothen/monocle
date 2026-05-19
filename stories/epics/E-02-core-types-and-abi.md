---
document_type: epic
epic_id: EPIC-02
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
subsystems: [SS-02]
capabilities: [CAP-002]
behavioral_contracts: [BC-2.02.001, BC-2.02.002, BC-2.02.003, BC-2.02.004, BC-2.02.005, BC-2.02.006, BC-2.02.007, BC-2.02.008]
verification_properties: [VP-011, VP-012, VP-013, VP-014, VP-015, VP-016, VP-017, VP-018]
---

# EPIC-02: Core Types and ABI Stability

## Purpose

Implement `monocle-core` crate with all Phase 1 forward-compatibility surfaces:
ABI version constant, non-exhaustive enum policy, FactoryAdapter open trait with
VsddFactoryAdapter implementation, and HookEnvelope protobuf wire schema. This epic
delivers the complete SS-02 Core Types and ABI subsystem and ensures Phase 2/3/4
evolution never requires breaking changes to Phase 1 consumers.

## Success Criteria

- All 8 BC-2.02.NNN behavioral contracts pass their verification properties
- `MONOCLE_ABI_VERSION = 1` exported as pub const at crate root
- `FactoryAdapter` trait compiles with no `Sealed` supertrait; 7 methods exact
- `HookEnvelope` proto field 1 is `schema_version: u32`
- VsddFactoryAdapter self-referential detection works against monocle's own `.factory/`

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-010 | monocle-core Crate + ABI Version Constant | 5 | Wave 2 | S-001 |
| S-011 | Non-Exhaustive Enum Policy | 3 | Wave 2 | S-010 |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | 8 | Wave 3 | S-010, S-011 |
| S-013 | HookEnvelope Proto Wire Format | 5 | Wave 2 | S-010 |

## Architecture Scope

- Implementing module: `monocle-core` (FactoryAdapter trait, wire format types, protocol versioning)
- Architecture source: `architecture/SS-core-types-and-abi.md` v1.2.13
- Architecture dependency: `architecture/SS-forward-compatibility.md` v1.2.19 (FC contracts)
