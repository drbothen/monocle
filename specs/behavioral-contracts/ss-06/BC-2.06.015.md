---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T14:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.015: Permission Overlay: `[t]` Trace-to-Source Stub

## Description

In `AppMode::Overlay`, pressing `[t]` (bound to `Action::PermissionTraceToSource` in the
`Builtin` binding table for `AppMode::Overlay`) renders a placeholder message in the
overlay footer area:

```
[t] Trace to source — Phase 2 feature (Static plane)
```

No navigation occurs. No file is opened. No IPC message is sent to the daemon. The
`AppMode` remains `Overlay { stack, prior }` unchanged. The keybinding is registered in
the `Builtin` table and appears in the keybinding hint line so it is discoverable in
Phase 1. This stub reserves the `[t]` binding for Phase 2 (the Static plane), preventing
future keybinding conflicts when the full trace-to-source behavior is implemented.

## Preconditions

1. `AppMode` is `Overlay { stack, prior }` with `stack.len() >= 1`.
2. The `Builtin` binding table for `AppMode::Overlay` maps key `t` to
   `Action::PermissionTraceToSource`.
3. The `Action::PermissionTraceToSource` variant is defined in `monocle-core/src/action.rs`
   with the comment `// Phase 1: stub renders placeholder message`.
4. The overlay layout has a footer area (at least 1 row) where the placeholder message can
   be rendered.

## Postconditions

1. **Placeholder rendered:** When `[t]` is pressed in `AppMode::Overlay`, the overlay
   footer area renders the text:
   `[t] Trace to source — Phase 2 feature (Static plane)`
   The text is rendered in the default terminal color (no special styling required in
   Phase 1).
2. **No AppMode transition:** `transition(Overlay { stack, prior }, PermissionTraceToSource)`
   returns `Overlay { stack, prior }` unchanged. This is the identity transition for this
   action.
3. **No IPC message sent:** The TUI does NOT send any IPC message to the daemon when `[t]`
   is pressed. The placeholder message is a local render-state change only.
4. **No navigation:** No file viewer, no editor, no subprocess is launched. The TUI
   remains in the ratatui event loop showing the permission overlay.
5. **Keybinding discoverable:** The keybinding hint line in the status bar (when
   `AppMode::Overlay` is active) includes `t: trace` or `t: trace (Phase 2)` to surface
   the binding to the user. The exact hint text is implementation-defined but must include
   `t` and some indication of the Phase 2 scope.
6. **Placeholder persists until next render with different state:** The placeholder message
   renders as long as `AppMode::Overlay` is active and the overlay is being drawn. It does
   not auto-dismiss. The user can dismiss the overlay by making a decision (`[1]`, `[2]`,
   `[3]`) or hiding the popup (`Ctrl-\`).
7. **`Action::PermissionTraceToSource` defined in `monocle-core` regardless of Phase:**
   The variant exists in `action.rs` in Phase 1 so that `[t]` can be bound without
   dead-code warnings or missing-match-arm compiler errors in `monocle-tui`. The
   `monocle-tui` match arm for this action renders the placeholder.

## Invariants

1. `Action::PermissionTraceToSource` is a `Builtin` binding (not `PerContext`, not
   `UserCustomCommand`). It cannot be overridden by the user in Phase 1.
2. The `[t]` stub is present in Phase 1 SOLELY to reserve the keybinding. The Phase 2
   implementation (trace-to-source in the Static plane) will replace the stub behavior
   without needing to add a new binding — the binding already exists.
3. The placeholder text includes "Phase 2 feature" to communicate clearly to users who
   press `[t]` in Phase 1 that this is an upcoming feature, not a broken one.
4. No state is stored for the `[t]` press. There is no `trace_pressed: bool` field in
   `App`. The placeholder rendering is driven entirely by whether `AppMode::Overlay` is
   active and whether the render pass needs to draw the footer.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-097 | `[t]` pressed when no `PromptModal` has a file-backed tool (e.g., tool is `Bash { command }`) | Same placeholder rendered; stub does not inspect `tool_payload`; no change from the normal stub behavior |
| EC-098 | `[t]` pressed multiple times in sequence | Each press triggers a re-render with the same placeholder; no state accumulation; no error |
| EC-099 | `[t]` pressed in `AppMode::Dashboard` | No binding match (not in Global or Dashboard PerContext tables); identity transition; keypress discarded silently |
| EC-100 | Phase 2 story implements real trace-to-source behavior | Phase 2 replaces the stub render arm; `Action::PermissionTraceToSource` variant is already defined; no breaking change to `action.rs` |
| EC-101 | Terminal is too narrow to render the full placeholder text | ratatui `Wrap` behavior truncates or wraps the text; no panic; no error |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Overlay { stack: [P1], prior: Sessions }`, `PermissionTraceToSource` | `Overlay { stack: [P1], prior: Sessions }` (unchanged) + overlay footer renders `[t] Trace to source — Phase 2 feature (Static plane)` | happy-path |
| `Dashboard { focused: Sessions }`, `PermissionTraceToSource` | `Dashboard { focused: Sessions }` (identity; not bound in Dashboard) | edge-case |
| `Overlay { stack: [P1], prior: Sessions }`, `PermissionTraceToSource` × 3 | `Overlay { stack: [P1], prior: Sessions }` (still unchanged after multiple presses) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `[t]` in `AppMode::Overlay` does not modify `stack` or send IPC | unit test |
| VP-TBD | Overlay footer renders the Phase 2 placeholder text when `PermissionTraceToSource` is the last action | unit test (render output inspection) |
| VP-TBD | `[t]` in `AppMode::Dashboard` has no effect | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the `[t]` trace-to-source stub within the "permission overlay stack" component of CAP-006; the stub reserves the keybinding for Phase 2 Static plane integration while ensuring the binding is discoverable in Phase 1 via the hint line |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: the stub sends no IPC message and writes no files; it is a pure render-state change) |
| Architecture Module | monocle-core (Action::PermissionTraceToSource variant — reserved); monocle-tui (overlay renderer stub arm for PermissionTraceToSource) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.0.0 §Permission Overlay §Trace-to-Source Stub; §Action Enum (PermissionTraceToSource variant with `// Phase 1: stub` comment); §Status Bar §Keybinding hint line (Overlay mode includes `t: trace`) |
| Cross-Ref | BC-2.06.001 (pure transition function — PermissionTraceToSource identity arm), BC-2.06.003 (5-level binding precedence — Builtin level for `[t]`) |
| Test File | `monocle-tui/tests/overlay_stub.rs` |
| Test Name | `test_BC_2_06_015_trace_to_source_stub_renders_placeholder` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: `PermissionTraceToSource` is the identity arm in the `transition()` pure function; the variant must be covered in any exhaustive match
- [BC-2.06.003] — depends on: `[t]` is registered at the `Builtin` level of the 5-level binding precedence system

## Architecture Anchors

- `architecture/SS-tui.md#trace-to-source-stub` — Phase 1 stub behavior with exact placeholder text
- `architecture/SS-tui.md#action-enum` — `PermissionTraceToSource` variant definition with Phase 1 comment

## Story Anchor

S-TBD — Implement `[t]` trace-to-source stub: register Builtin binding, render placeholder footer (filled by story-writer)

## VP Anchors

- VP-TBD — Unit tests for stub render and no-op transition behavior

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.015 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.0.0 §Permission Overlay §Trace-to-Source Stub, §Action Enum
  (PermissionTraceToSource variant); prd-expansion-scope.md §3.3 BC-2.06.015 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: stub renders placeholder text only; no file writes or IPC sends.
- Priority P1 (vs P0 for BCs 009–014) per SS-tui.md BC table and prd-expansion-scope.md §3.3.
- Invariant 1 records that `[t]` is a `Builtin` binding (non-overridable in Phase 1).
- Postcondition 7 documents why the variant must be defined in `monocle-core` even though
  it does nothing in Phase 1: to prevent missing-match-arm compiler errors.
