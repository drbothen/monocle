---
document_type: epic
epic_id: EPIC-03
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
subsystems: [SS-03]
capabilities: [CAP-003]
behavioral_contracts: [BC-2.03.001, BC-2.03.002, BC-2.03.003, BC-2.03.004]
verification_properties: [VP-019, VP-020, VP-021, VP-022]
---

# EPIC-03: Engine Module Abstraction

## Purpose

Implement the `EngineModule` open trait in `monocle-core` and the `ClaudeCodeModule`
built-in adapter for Phase 1. This epic delivers the complete SS-03 Engine Module
subsystem: the harness-plane abstraction that allows Claude Code (Phase 1),
CodeMachine (Phase 4), and WASM plugins (Phase 3+) to integrate with the monocle
daemon without coupling to implementation details.

## Success Criteria

- All 4 BC-2.03.NNN behavioral contracts pass their verification properties
- `EngineModule` trait: 5 methods, `#[async_trait]`, no `Sealed` bound, `Send + Sync + 'static`
- `ClaudeCodeModule::detect()` rejects non-claude processes via strict basename match
- `HomeUnresolvable` error triggers daemon exit with correct diagnostic message
- `hook_paths()` returns exactly 5 entries matching the 5 `HookType` variants

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-014 | EngineModule Trait Definition | 5 | Wave 2 | S-010 |
| S-015 | ClaudeCodeModule Implementation | 8 | Wave 3 | S-014 |

## Architecture Scope

- Implementing module: `monocle-core` (EngineModule trait, ClaudeCodeModule adapter)
- Architecture source: `architecture/SS-engine-module.md` v1.1.20
- Architecture dependency: `architecture/SS-core-types-and-abi.md` v1.2.13 (HookEvent, HookResponse types)
