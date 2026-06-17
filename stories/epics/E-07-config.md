---
document_type: epic
epic_id: EPIC-07
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
subsystems: [SS-07]
capabilities: [CAP-007]
behavioral_contracts: [BC-2.07.001, BC-2.07.002, BC-2.07.003, BC-2.07.004, BC-2.07.005, BC-2.07.006]
verification_properties: []
---

# EPIC-07: Config

## Purpose

Implement the monocle configuration crate (`monocle-config`): atomic write via
`tempfile::persist`, config schema version 1 (harness profile fields including
`ccr_base_url`, `claude_binary`, `display_name`, `profile_name`), missing/corrupted
config default application, CCR detection via `ccr_path` config field, and the
profile picker (sticky-per-project selection with `Ctrl-P` override). This epic
delivers the complete SS-07 Config subsystem.

## Success Criteria

- All 6 BC-2.07.NNN behavioral contracts pass their verification properties
- Config file written atomically via `tempfile::persist` with 0o600 file permissions
- Config schema version 1 fields: `profile_name`, `claude_binary`, `ccr_base_url`, `display_name`
- Missing or corrupted config applies safe defaults without panicking
- CCR detected when `ccr_path` field is non-empty and the binary exists
- Profile picker: sticky-per-project selection persisted across TUI restarts
- `Ctrl-P` override shows the picker regardless of existing sticky selection
- `cargo clippy --workspace --all-targets -- -D warnings` → 0 warnings

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-030 | Config Crate: Atomic Write, Schema v1, Missing/Corrupted Default, CCR Detection | 5 | Wave 4 | S-001 |
| S-031 | Profile Picker: Sticky-Per-Project Selection + Ctrl-P Override | 5 | Wave 7 | S-030, S-024, S-025 |

**Total: 10 points**

## Architecture Scope

- Implementing module: `monocle-config` (config crate)
- Architecture source: `architecture/SS-config.md` v1.3.0
- Architecture dependency: `architecture/SS-tui.md` v1.8.2 (S-031 profile picker UI)
