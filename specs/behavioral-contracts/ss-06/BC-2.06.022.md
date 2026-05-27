---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T18:00:00Z
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

# Behavioral Contract BC-2.06.022: Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve

## Description

The monocle product's primary user promise: a developer with two Claude Code sessions
stalled on permission prompts can unblock both sessions with 4 keystrokes (`Ctrl-\`, `2`,
`1`, `Ctrl-\`) without leaving their editor, switching tmux windows, or typing in any
terminal. This is the end-to-end integration contract that validates that all upstream
components (daemon hook ingestion, IPC transport, TUI overlay stack, keybinding dispatch,
decision send path) compose correctly to satisfy the product's core value proposition.
The Success Criterion is ≤6 keystrokes; the canonical happy path achieves exactly 4.

## Preconditions

1. The user is working in a text editor (e.g., `nvim`) inside a tmux session. The editor
   is focused. No other terminal windows are open.
2. Two Claude Code sessions are running as background processes. Each is stalled, awaiting
   a `PreToolUse` decision from the hook server.
3. The daemon has received 2 `PermissionPromptQueued` events via the `PreToolUse` hook
   endpoint. The daemon's `DaemonState::queued_prompts` holds both `PromptModal` entries
   (P1 and P2, in insertion order).
4. The monocle TUI process is NOT running. The last `Ctrl-\` popup was previously
   dismissed (the user's tmux session has the `Ctrl-\` → `display-popup -E monocle`
   binding configured per SS-tui.md §Ctrl-\ Integration).
5. No tmux window switch or pane focus change has occurred since the editor was last
   focused.

## Postconditions

The complete 4-keystroke flow is specified step by step. Each step has testable
preconditions, postconditions, and daemon actions.

### Step 1: `Ctrl-\` (Keystroke 1) — TUI appears with overlay

| Aspect | Specification |
|--------|---------------|
| Action | User presses `Ctrl-\` in tmux |
| tmux behavior | `display-popup -E monocle` spawns a new `monocle` process |
| TUI startup | `monocle` starts, reads lock file, connects to UDS at `<runtime_dir>/monocle.sock` |
| Daemon initial state push | Daemon sends `IpcServerMessage::InitialState { queued_prompts: [P1, P2], sessions: [...], recent_events: [...] }` |
| TUI AppMode | Transitions to `AppMode::Overlay { stack: [P1, P2], prior: FocusSnapshot::Sessions }` because `queued_prompts` is non-empty |
| Screen render | Overlay renders P1 (front of stack) with its `ToolPayload`; peek shows P2 header; status bar breadcrumb shows `Dashboard > Overlay [2 prompts]`; hint shows `1: accept-once  2: accept-always  3: reject  ↑↓: cycle  Esc: hide  t: trace` |
| Latency | TUI overlay is on screen within 100ms of receiving `InitialState` (Success Criterion per BC-2.06.017) |
| Editor focus | Editor is NOT disturbed. tmux popup floats over the editor; the editor process is not sent any signal |

### Step 2: `2` (Keystroke 2) — Accept-always P1; P2 becomes front

| Aspect | Specification |
|--------|---------------|
| Action | User presses `2` in the TUI overlay |
| Keybinding resolution | `PerContext` table for `AppMode::Overlay` maps `2` → `Action::PermissionAcceptAlways` |
| Decision send | TUI sends `IpcClientMessage::DecisionResponse { prompt_id: P1.prompt_id, decision: PermissionDecision::Always }` to daemon via `ipc_tx` (non-blocking) |
| Daemon action | Daemon receives `DecisionResponse` for P1; sends `{"decision":"always"}` HTTP response to P1's stalled Claude Code session; P1's Claude Code process unblocks and resumes |
| Pattern recording | Daemon records the `PermissionDecision::Always` pattern for future auto-accept (per BC-2.06.012) |
| TUI AppMode transition | `transition(Overlay { stack: [P1, P2], prior: Sessions }, PermissionAcceptAlways)` → `Overlay { stack: [P2], prior: Sessions }` (P1 popped from front, P2 now front) |
| Screen render | Overlay re-renders with P2 as the active prompt; badge shows `[1 prompt]`; breadcrumb shows `Dashboard > Overlay [1 prompt]` |
| Claude Code session P1 | Unblocked. Resumes execution. Developer does NOT need to be aware of this. |

### Step 3: `1` (Keystroke 3) — Accept-once P2; overlay closes

| Aspect | Specification |
|--------|---------------|
| Action | User presses `1` in the TUI overlay |
| Keybinding resolution | `PerContext` table for `AppMode::Overlay` maps `1` → `Action::PermissionAcceptOnce` |
| Decision send | TUI sends `IpcClientMessage::DecisionResponse { prompt_id: P2.prompt_id, decision: PermissionDecision::Once }` to daemon |
| Daemon action | Daemon receives `DecisionResponse` for P2; sends `{"decision":"accept"}` HTTP response to P2's stalled Claude Code session; P2's Claude Code process unblocks |
| TUI AppMode transition | `transition(Overlay { stack: [P2], prior: Sessions }, PermissionAcceptOnce)` → `Dashboard { focused: Sessions }` (P2 popped; stack empty; collapses to Dashboard per BC-2.06.001 invariant) |
| Screen render | Dashboard renders. Sessions panel shows. Status bar shows `Dashboard > Sessions`. Overlay is gone. |
| Claude Code session P2 | Unblocked. Resumes execution. |

### Step 4: `Ctrl-\` (Keystroke 4) — TUI dismissed; editor focused

| Aspect | Specification |
|--------|---------------|
| Action | User presses `Ctrl-\` again |
| tmux behavior | `display-popup` closes the popup window. The `monocle` TUI process exits (or is SIGTERM'd by tmux). |
| Editor focus | Editor regains focus immediately. The user is back in `nvim` exactly where they left off. |
| Post-state | Both Claude Code sessions are running. The developer never left the editor. No tmux window switch. No pane switch. |
| Daemon state | `DaemonState::queued_prompts` is empty. Daemon is healthy and continues receiving hooks from both sessions. |

### Summary Postcondition

After completing all 4 steps:
1. Both Claude Code sessions (P1 and P2) are unblocked and running.
2. The developer's editor focus is restored exactly as it was before Step 1.
3. Total keystrokes: 4 (`Ctrl-\`, `2`, `1`, `Ctrl-\`). This satisfies the ≤6 keystroke
   Success Criterion.
4. No tmux window switch occurred. No terminal pane was manually selected.
5. The entire flow completes within the 300ms PreToolUse timeout budget (Steps 2 and 3
   each send a decision in under 1 second of user think time; daemon receives them within
   the budget if the user is responsive).

## Invariants

1. **Editor focus is preserved.** At no point in the 4-step flow does the user's editor
   lose focus in a way that requires manual re-focus. The `display-popup` mechanism
   ensures this: tmux floats a popup over the current pane without switching panes.
2. **No window switch.** The user's tmux session has 0 pane switches and 0 window
   switches during the flow. This is the "without leaving the editor" guarantee.
3. **Keystrokes are counted as user-initiated input.** `Ctrl-\` counts as 1 keystroke
   even though it is a tmux chord. The total count must not exceed 6 (Success Criterion).
   In the canonical flow: `Ctrl-\` + `2` + `1` + `Ctrl-\` = 4 keystrokes.
4. **Both Claude Code sessions are unblocked.** After the flow, both P1 and P2 have
   received their HTTP decision responses. Neither session is still waiting.
5. **The AppMode empty-stack collapse is the mechanism for the automatic return to
   Dashboard.** Step 3 pops the last `PromptModal`, the `transition()` function collapses
   `Overlay { stack: [], prior }` → `Dashboard { focused: Sessions }` atomically. The
   user does NOT need to press `Esc` to leave the overlay — the overlay disappears when
   all decisions are made.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-134 | A third Claude Code session queues a new prompt (P3) between keystrokes 2 and 3 | P3 is pushed to the back of the `VecDeque` after P2. After keystroke 3 resolves P2, `stack = [P3]` — overlay remains open showing P3. The flow requires a 5th keystroke to resolve P3. Total: 5 keystrokes ≤ 6. Still within Success Criterion. |
| EC-135 | P1's PreToolUse times out (300ms) while user is reading P1 before pressing `2` | Daemon sends fail-open for P1; pushes `PromptAutoResolved { P1 }` to TUI; TUI removes P1 from stack; overlay shows P2 only. User still needs to decide P2. P1's Claude Code session resumed (via fail-open), which is acceptable per BC-2.06.017. |
| EC-136 | User presses `2` for P1 at exactly 299ms (near-timeout) | Race between user decision and daemon timeout. If daemon sends fail-open before decision arrives: duplicate resolution — daemon MUST handle idempotently (log warning, ignore duplicate). If user decision arrives first: normal flow. Either outcome is acceptable. |
| EC-137 | 6 Claude Code sessions are all stalled simultaneously | `DaemonState::queued_prompts` holds 6 entries. Overlay stack shows 6. User needs up to 6 keystrokes for the overlay decisions (1 per session) + 2 for open/close = 8 total. This exceeds the ≤6 success criterion for >4 concurrent stalled sessions. The ≤6 criterion is defined for the 2-session (dual) case. |
| EC-138 | User dismisses the popup (`Ctrl-\`) before resolving all prompts (between steps 2 and 3) | P2 remains in daemon's `queued_prompts`. On next `Ctrl-\`, new TUI process opens overlay with P2 at front. Total keystrokes across two openings: `Ctrl-\`, `2`, `Ctrl-\`, `Ctrl-\`, `1`, `Ctrl-\` = 6. Still ≤6 Success Criterion if this is counted. |
| EC-139 | `monocle` daemon is not running when user presses `Ctrl-\` | Daemon auto-starts (per BC-2.04.001 MONOCLE_NO_AUTOSTART not set). TUI connects after auto-start. Overlay shows queued prompts from the new daemon session. First `Ctrl-\` is the startup keystroke. Adds ~1s latency for daemon start; not a blocker for the killer scenario correctness. |

## Canonical Test Vectors

This BC is an end-to-end scenario; individual unit tests for each step are defined in
the component BCs. The E2E test vector is:

| Test ID | Setup | Keystroke Sequence | Expected Final State |
|---------|-------|-------------------|---------------------|
| KS-001 | Daemon has P1 + P2 queued; TUI not running | `[connect]`, `2`, `1`, `[disconnect]` | P1 resolved Always; P2 resolved Once; both Claude Code sessions unblocked; AppMode Dashboard; 4 user keystrokes |
| KS-002 | Same setup; P3 arrives during flow | `[connect]`, `2`, `1`, `1`, `[disconnect]` | All 3 resolved; 5 keystrokes ≤ 6 |
| KS-003 | Daemon has 1 queued prompt (single session stalled) | `[connect]`, `1`, `[disconnect]` | P1 resolved Once; 3 keystrokes ≤ 6 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E2E: 2 stalled sessions unblocked in 4 keystrokes (`[connect]`, `2`, `1`, `[disconnect]`) | E2E integration test with mock daemon and mock Claude Code HTTP endpoint |
| VP-TBD | Editor focus not lost during flow (no tmux pane switch) | E2E test with tmux automation (verify active pane ID unchanged across all steps) |
| VP-TBD | Overlay auto-collapses to Dashboard when last PromptModal is resolved | unit test (steps 1–3 without tmux) |
| VP-TBD | Both `DecisionResponse` messages delivered to daemon before HTTP response timeout | integration test (assert Claude Code mock HTTP responses received within 300ms) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC is the E2E validation contract for CAP-006 in its entirety: it exercises AppMode state machine, keybinding dispatch, permission overlay stack, and Ctrl-\ popup integration in a single end-to-end scenario that is the product's core value proposition |
| L2 Domain Invariants | DI-001 (every hook event received by the daemon MUST be written to the JSONL ring — both P1 and P2 `PreToolUse` events were written to the ring when the daemon received them; this BC verifies the decision path after ring write); DI-007 (monocle MUST NOT write to any file owned by a harness — satisfied: decisions are sent via IPC HTTP response; no harness files are modified by the TUI) |
| Architecture Module | monocle-tui (overlay rendering, keybinding dispatch, IPC decision send); monocle-core (AppMode transition function, VecDeque pop-and-collapse); monocle-ipc (DecisionResponse delivery); monocle-runtime (daemon HTTP response hold and release) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.0.0 §Killer Scenario Flow (complete step-by-step table including AppMode transitions, daemon actions, and Claude Code unblock verification); §Permission Overlay §Overlay Stack Lifecycle steps 3 and 2 (decision send and rotate) |
| Cross-Ref | BC-2.06.008 (overlay push on PermissionPromptQueued — Step 1 precondition); BC-2.06.012 (Accept-Always — Step 2); BC-2.06.011 (Accept-Once — Step 3); BC-2.06.001 (AppMode empty-stack collapse — Step 3 automatic Dashboard return); BC-2.06.017 (hook timeout budget — overall timing constraint); BC-2.05.002 (initial state push — Step 1 mechanism for receiving queued prompts) |
| Test File | `monocle-tui/tests/killer_scenario.rs` |
| Test Name | `test_BC_2_06_022_killer_scenario_dual_permission_resolve` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.008] — depends on: overlay push (Step 1 — overlay appears because daemon has queued prompts)
- [BC-2.06.012] — composes with: Accept-Always decision (Step 2)
- [BC-2.06.011] — composes with: Accept-Once decision (Step 3)
- [BC-2.06.001] — depends on: empty-stack collapse in `transition()` (Step 3 automatic Dashboard return)
- [BC-2.06.017] — depends on: hook timeout budget (300ms PreToolUse ceiling governs the timing window)
- [BC-2.05.002] — depends on: initial state push delivers queued prompts on TUI connect (Step 1)
- [BC-2.05.005] — depends on: `PermissionPromptQueued` IPC message is how P1/P2 were queued
- [BC-2.06.004] — composes with: `Ctrl-\` popup lifecycle (Steps 1 and 4)

## Architecture Anchors

- `architecture/SS-tui.md#killer-scenario-flow` — canonical 4-step table (AppMode before/after, daemon action, user action)
- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle (push, decide, auto-collapse)
- `architecture/SS-tui.md#ctrl-integration` — state preservation and display-popup lifecycle

## Story Anchor

S-TBD — Implement and verify killer scenario: E2E integration test proving ≤4 keystrokes unblocks 2 concurrent Claude Code sessions (filled by story-writer)

## VP Anchors

- VP-TBD — E2E integration test: `[connect]` + `2` + `1` + `[disconnect]`; both mock HTTP responses delivered; total keystrokes = 4

## §Trace v1.0.0

**Initial production** (2026-05-26T18:00:00Z):
- BC-2.06.022 created as part of SS-06 TUI behavioral contract burst (BCs 016–022).
- Reads: SS-tui.md v1.0.0 §Killer Scenario Flow (complete step table); §Permission Overlay
  §Overlay Stack Lifecycle; §Ctrl-\ Integration; prd-expansion-scope.md §3.3 BC-2.06.022
  description (F-29, F-34, F-36, F-41) and §4 Success Criteria Gap Closure (session
  management killer scenario row).
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-001 cited: both PreToolUse events were written to the JSONL ring at ingestion; this
  BC verifies the decision path downstream of that invariant.
- DI-007 cited: no harness file writes; decisions go via IPC only.
- Each step in Postconditions is specified at the level of a test-executable precondition
  + action + postcondition + daemon action — no step is hand-wavy.
- EC-134 proves the ≤6 criterion holds for 3 concurrent sessions (5 keystrokes).
- EC-137 is explicit that the ≤6 criterion is defined for the 2-session (dual) case, not
  for arbitrarily large concurrent stall counts — this prevents ambiguity in acceptance.
- Invariant 5 documents the `AppMode` empty-stack collapse as the mechanism for the
  automatic Dashboard return — ensuring the implementer uses `transition()` correctly
  rather than adding a manual mode-reset after the last decision.
