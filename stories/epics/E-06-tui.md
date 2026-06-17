---
document_type: epic
epic_id: EPIC-06
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
subsystems: [SS-06]
capabilities: [CAP-006]
behavioral_contracts: [BC-2.06.001, BC-2.06.002, BC-2.06.003, BC-2.06.004, BC-2.06.005, BC-2.06.006, BC-2.06.007, BC-2.06.008, BC-2.06.009, BC-2.06.010, BC-2.06.011, BC-2.06.012, BC-2.06.013, BC-2.06.014, BC-2.06.015, BC-2.06.016, BC-2.06.017, BC-2.06.018, BC-2.06.019, BC-2.06.020, BC-2.06.021, BC-2.06.022, BC-2.06.023, BC-2.06.024, BC-2.06.025]
verification_properties: []
---

# EPIC-06: TUI

## Purpose

Implement the monocle ratatui TUI binary: the `Ctrl-\` popup, sessions panel with Nucleo
fuzzy filter, permission overlay (VecDeque stack, diff preview, decision keybindings,
SOQ-3 clear), event ribbon, status bar (drop counter, breadcrumb, keybinding hints),
killer scenario (≤6 keystrokes dual permission resolve), and the multi-project sessions
panel with full lifecycle actions. This epic delivers the complete SS-06 TUI subsystem
across Waves 4-8.

## Success Criteria

- All 25 BC-2.06.NNN behavioral contracts pass their verification properties
- `AppMode` state machine: compile-time mutual exclusion; `transition()` validates all state combinations
- `Ctrl-\` popup appears and dismisses without state loss (FocusSnapshot restored)
- Sessions panel renders from IPC state, supports `/` fuzzy filter (Nucleo), `Enter` → Fullscreen
- Permission overlay: VecDeque stack with `[↑↓]` rotation, diff preview via `similar 3`, Accept-Once/Accept-Always/Reject/Esc keybindings
- Overlay cleared on daemon disconnect (SOQ-3) and on `PermissionPromptResolved` receipt
- Event ribbon: rolling log of last N hook events
- Status bar: drop counter, breadcrumb, keybinding hint line
- Killer scenario: ≤6 keystrokes for dual permission resolve (BC-2.06.022)
- Multi-project sessions panel: grouped by project, lifecycle actions (Kill/Detach/Rename), 5-state indicator, Enter-on-Detached → AttachSession → EmbeddedTerminal
- BC-2.06.017 (render latency ≤100ms): gap — deferred to Phase 3 integration test infrastructure (GAP-P2-005)
- `cargo clippy --workspace --all-targets -- -D warnings` → 0 warnings

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-024 | TUI Core Types: AppMode, Action, FocusSnapshot, transition(), 5-Level Dispatch | 8 | Wave 4 | S-011, S-014 |
| S-025 | TUI Binary Skeleton, Ctrl-\ Popup Integration, and Sessions Panel | 8 | Wave 6 | S-024, S-022, S-030 |
| S-026 | Permission Overlay: VecDeque Stack, Decision Keybindings, Esc Hide, SOQ-3 | 13 | Wave 6 | S-024, S-022, S-023 |
| S-027 | Permission Overlay Rendering, Diff Preview (similar 3), Status Bar | 8 | Wave 7 | S-026, S-025 |
| S-028 | Sessions Panel Nucleo Filter + Event Ribbon Rolling Log | 5 | Wave 7 | S-025, S-021 |
| S-029 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | 5 | Wave 7 | S-026, S-027, S-022, S-018 |
| S-048 | Sessions Panel — Multi-Project Grouping, Lifecycle Actions, and State-Aware Blocking | 8 | Wave 8 | S-022, S-025, S-028, S-033, S-047 |

**Total: 55 points (13 pts Wave 4 + 34 pts Waves 6-7 + 8 pts Wave 8 v1A delta)**

## Architecture Scope

- Implementing modules: `monocle-tui` (TUI binary, event loop, panels, overlays)
- Architecture source: `architecture/SS-tui.md` v1.8.2
- Architecture dependency: `architecture/SS-ipc.md` v1.24.0 (IPC message types consumed by TUI)
- Architecture dependency: `architecture/SS-config.md` v1.3.0 (config read in TUI panels)
