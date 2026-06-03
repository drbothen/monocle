---
document_type: gene-source-disposition
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
gene_source: lazygit
disposition_pass: v2 (D-236 control-center pivot)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md §5
---

# Gene-Source Disposition v2: lazygit (Control-Center Lens)

## Vision Lens Applied

monocle v1 (re-baselined): full TUI control center. lazygit is monocle's TUI philosophy gene —
the interaction model, key dispatch, and panel philosophy. The pivot adds new capabilities
but does not fundamentally change the interaction model.

## Original Disposition Summary

lazygit was the TUI philosophy reference. The original vision adopted:
- 5-level binding precedence (SearchPrompt > UserCustomCommand > PerContext > Global > Builtin)
- Telescope `?` help overlay
- `/` filter convention with Nucleo fuzzy matching
- Context-aware Action enum dispatch
- Popup stacking (VecDeque<PromptModal> fixing lazygit's single-popup drop)
- Layered config (defaults + global + per-repo hot-reload)
- Docked-bottom log viewer with expand
- Modal cascade with FocusSnapshot restoration

All of these are implemented in Phase 1 (S-027 wave 7, S-028 filter, S-029 permission overlay).

## Disposition by Capability Area

### 1. 5-Level Binding Precedence (originally ADOPT, built)

**New verdict: ADOPT (already built, confirmed).** The control-center adds many new actions
but the precedence model is unchanged. New actions register at the appropriate precedence level:
- Session launch/kill/attach → PerContext (Sessions panel context)
- PTY keyboard forwarding → special case in EmbeddedTerminal AppMode (all keys below
  the "exit terminal" key go to PTY; the escape key exits EmbeddedTerminal mode)
- Tune apply/reset → PerContext (Tune panel context)

The 5-level model handles all of this without modification.

### 2. Popup Stacking / VecDeque<PromptModal> (originally ADOPT, built)

**New verdict: ADOPT (already built, confirmed).** S-029 delivered the killer scenario
(≤6 keystrokes per concurrent permission prompt pair) with VecDeque stack. This is the
operational core that the control-center preserves and builds around.

No change from the pivot. The permission overlay remains central and correct.

### 3. Telescope Help Overlay (originally ADOPT)

**New verdict: ADOPT (confirmed).** The `?` help overlay now has more actions to list
(session launch, kill, attach, PTY navigation), but the overlay format and behavior is
unchanged. New actions are added to the builtin action registry, which the help overlay
displays automatically.

### 4. `/` Filter Convention (originally ADOPT, built)

**New verdict: ADOPT (already built, confirmed).** The Nucleo-based filter for the sessions
panel is built (S-028). The control-center extends the filter to the multi-project session
list (filter by project name, session name, harness type). No architectural change.

### 5. Context-Aware Action Enum Dispatch (originally ADOPT, built)

**New verdict: ADOPT + EXTEND.** New actions added to the `Action` enum for the
control-center:

```rust
pub enum Action {
    // Existing (confirmed unchanged) ...
    SessionSelect, SessionKill, SessionAttach,
    PermissionAcceptOnce, PermissionAcceptAlways, PermissionReject,
    // NEW: session lifecycle
    SessionLaunch,           // open session creation wizard
    SessionDetach,           // detach TUI from session PTY (session continues in daemon)
    SessionRename,           // rename in-place
    // NEW: PTY interaction
    EnterEmbeddedTerminal,   // enter EmbeddedTerminal AppMode for focused session
    ExitEmbeddedTerminal,    // return to Dashboard from EmbeddedTerminal
    ForwardKeyToPty(KeyEvent), // internal action; only valid in EmbeddedTerminal mode
    PtyScrollUp,             // scroll the PTY scrollback buffer
    PtyScrollDown,
    // NEW: Tune plane
    TuneEditBinding,         // open binding editor overlay for selected binding
    TuneApplyProfile,        // apply a profile to the focused session
    TuneResetBinding,        // reset binding to builtin default
}
```

All new actions follow the existing pattern: enum variants are `Clone + PartialEq + Eq + Hash
+ Serialize + Deserialize` for inspect ability and binding-serialization.

### 6. Layered Config + Hot-Reload (originally ADOPT)

**New verdict: ADOPT (confirmed).** The config hot-reload (PollWatcher + 100ms debounce,
from zellij gene via lazygit inspiration) is unchanged. The control-center adds per-session
config overlay (active profile, model routing) but the config loading architecture is the same.

### 7. Extras/Log-Viewer Docked-Bottom (originally ADOPT)

**New verdict: ADOPT (confirmed).** The event ribbon (bottom-docked hook event log, S-028)
is the monocle implementation of this pattern. The control-center adds PTY bytes to the
daemon's output stream but the event ribbon continues to show hook events (not PTY bytes —
PTY bytes go to the embedded terminal pane, not the event ribbon).

### 8. DisabledReason Gate Pattern (from lazygit pass-8)

**New verdict: MODEL.** lazygit's `DisabledReason` (five-shape gate for graying out actions
in context) is the right pattern for monocle's `check_action` gate. New disabled reasons for
control-center actions:
- `SessionLaunch` disabled when: no harness profiles configured.
- `SessionKill` disabled when: no session selected or session is already Terminated.
- `EnterEmbeddedTerminal` disabled when: selected session is not Running.
- `ForwardKeyToPty` disabled when: not in EmbeddedTerminal mode.

Rust implementation: each `Action` variant maps to a `fn is_available(&self, mode: &AppMode, state: &AppState) -> Option<DisabledReason>` function in the action dispatcher.

### 9. Panel Layout (3-column with docked bottom rows) (originally ADOPT)

**New verdict: ADOPT + EXTEND.** The original 5-panel layout:
```
┌─ Sessions ─┬─ Preview ─┬─ Workflow ─┐
├─ Customizations ────────────────────┤
├─ Events (event ribbon) ─────────────┤
└─ Status bar ────────────────────────┘
```

The control-center extends this with an embedded terminal pane that can occupy the Preview
slot (or fullscreen over all panels):
- Normal mode: Preview shows session detail (token count, cost, uptime, hooks). Unchanged.
- EmbeddedTerminal mode: Preview pane is replaced by the tui-term PTY widget for the
  focused session. Sessions panel remains visible for switching. Event ribbon remains visible
  below.
- Fullscreen terminal: `F` key (Fullscreen AppMode with panel=Terminal) expands the PTY pane
  to the full width, hiding Sessions/Workflow/Events panels temporarily.

The layout reuse is clean: the existing `FocusSnapshot::Preview` slot becomes the terminal
pane. No new panel slot is needed for the basic case.

## Summary Table

| Capability | Original Verdict | New Verdict | Change? |
|-----------|-----------------|-------------|---------|
| 5-level binding precedence | ADOPT (built) | ADOPT (confirmed; new actions fit existing precedence) | Confirmed |
| Popup stacking VecDeque | ADOPT (built) | ADOPT (confirmed) | Confirmed |
| Telescope help overlay | ADOPT | ADOPT (confirmed; new actions auto-listed) | Confirmed |
| `/` filter convention | ADOPT (built) | ADOPT (confirmed; multi-project extension) | Extended |
| Context-aware Action enum | ADOPT (built) | ADOPT + EXTEND (session lifecycle + PTY + Tune actions) | Extended |
| Layered config + hot-reload | ADOPT | ADOPT (confirmed) | Confirmed |
| Event ribbon (docked bottom) | ADOPT (built) | ADOPT (confirmed; PTY bytes go to terminal pane, not ribbon) | Clarified |
| DisabledReason gate | MODEL | MODEL (extended for new actions) | Extended |
| Panel layout (3-column) | ADOPT | ADOPT + EXTEND (Preview slot becomes terminal pane) | Extended |

## Net Assessment

lazygit remains the TUI philosophy gene with no major reversals. The control-center pivot
adds new actions and a new AppMode variant (`EmbeddedTerminal`) but the underlying interaction
model (5-level precedence, enum dispatch, popup stacking, panel layout) is unchanged.

The most significant addition this disposition introduces:
- `EmbeddedTerminal` AppMode — the new TUI state where all keystrokes are forwarded to
  the PTY master, with a single escape key to return to Dashboard.
- Layout extension: Preview pane slot hosts the tui-term PTY widget in EmbeddedTerminal mode.
  This reuses the existing panel layout without adding a new panel slot.

All lazygit-sourced genes that were already ADOPT remain ADOPT. No reversals needed.
