---
document_type: epic
epic_id: EPIC-09
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
subsystems: [SS-09]
capabilities: [CAP-009]
behavioral_contracts: [BC-2.09.001, BC-2.09.002, BC-2.09.003, BC-2.09.004, BC-2.09.005, BC-2.09.006, BC-2.09.007, BC-2.09.008, BC-2.09.009]
verification_properties: []
---

# EPIC-09: Embedded PTY

## Purpose

Implement the monocle embedded terminal: PTY output pipeline (vt100::Parser, PseudoTerminal
render, PtyOutput IPC handler, auto-attach on first EmbeddedTerminal entry), full-fidelity
keyboard forwarding (all v1A key classes, Kitty keyboard protocol CSI u sequences, bracketed
paste), mouse forwarding (SGR 1006 encoding, scoped entry/exit), PTY resize detection with
50ms debounce, scrollback navigation (1000-row default, configurable, per-session offsets),
and EmbeddedTerminal + SessionCreation AppMode transitions including the SessionCreation
wizard, SpawnAck auto-advance, and permission badge+bell. This epic delivers the complete
SS-09 Embedded PTY subsystem as the terminal-emulation core of the v1A control-center scope.

## Success Criteria

- All 9 BC-2.09.NNN behavioral contracts pass their verification properties
- PTY output renders within 100ms of byte receipt at TUI (BC-2.09.001)
- All v1A key classes forwarded correctly: printable chars, Ctrl+[A-Z], arrows, function keys, Alt+char, Shift+Tab, Kitty enhanced combos, bracketed paste
- Mouse events forwarded in SGR 1006 encoding; scoped capture enabled on EmbeddedTerminal ENTRY and disabled on EXIT
- PTY resized within 2 render ticks of pane area change; 50ms debounce (BC-2.09.006)
- Scrollback: 1000 rows default; PtyScrollUp/Down navigate per-session offsets
- EmbeddedTerminal AppMode enter/exit transitions compile-time validated; SessionCreation wizard auto-advances to EmbeddedTerminal on SpawnAck
- Permission badge rendered in status bar + audible bell within one render tick (BC-2.09.009)
- `cargo clippy --workspace --all-targets -- -D warnings` → 0 warnings

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-039 | PTY Output Pipeline — vt100::Parser, PseudoTerminal Render, PtyOutput IPC Handler, Auto-Attach on First Entry | 8 | Wave 9 | S-021, S-025, S-035 |
| S-040 | Full-Fidelity Keyboard Forwarding — key_event_to_pty_bytes, Kitty Protocol CSI u, and Bracketed Paste | 8 | Wave 9 | S-039 |
| S-041 | Mouse Forwarding — mouse_event_to_pty_bytes, SGR 1006 Scoped Entry/Exit, Out-of-Pane Clip | 5 | Wave 9 | S-040 |
| S-042 | PTY Resize Detection, 50ms Debounce, and ResizePane IPC | 5 | Wave 9 | S-039 |
| S-043 | Scrollback Navigation — PtyScrollUp/Down, Per-Session Offsets, Configurable Capacity | 3 | Wave 9 | S-039, S-042 |
| S-044 | EmbeddedTerminal + SessionCreation AppMode Transitions, SessionCreation Wizard, SpawnAck, and Permission Badge+Bell | 13 | Wave 9 | S-033, S-035, S-040, S-041 |

**Total: 42 points (all Wave 9 v1A)**

## Architecture Scope

- Implementing modules: `monocle-core` (pure PTY key/mouse conversion functions), `monocle-tui` (EmbeddedTerminal panel, event dispatch, AppMode transitions), `monocle-runtime` (PTY output pipeline, PtyOutput IPC handler)
- Architecture source: `architecture/SS-embedded-pty.md` v1.7.0
- Architecture dependency: `architecture/SS-ipc.md` v1.24.0 (KeyInput, ResizePane, PtyOutput wire types)
- Architecture dependency: `architecture/SS-tui.md` v1.8.2 (AppMode state machine, EmbeddedTerminal panel)
- Architecture dependency: `architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md` v1.2.1 (portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4)
