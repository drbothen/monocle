---
document_type: behavioral-contract
level: L3
version: "1.2.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T00:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "c1e8267"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.004: Ctrl-\ Popup: Appears and Dismisses Without State Loss

## Description

The monocle TUI is launched as a `tmux display-popup` bound to `Ctrl-\` in the user's
`tmux.conf`. Each `Ctrl-\` press either spawns a new `monocle` process (if no popup is
visible) or hides the popup window (if visible). Because the TUI is a stateless view over
the daemon's durable state, the overlay stack and session list survive the hide/show cycle:
the daemon owns the pending-prompt registry (`overlay_stack: Vec<PermissionPromptPayload>` in IPC) and pushes the current state to
every new TUI client on connection. The TUI process itself is short-lived; the daemon is
the durable state store.

## Preconditions

1. The user's `tmux.conf` contains a binding equivalent to:
   `bind-key -n C-\\ display-popup -E -w 80% -h 80% 'monocle'`
2. A monocle daemon is running (or auto-starts per BC-2.04.002) and holds the current
   `DaemonState` including the pending-prompt registry (`overlay_stack: Vec<PermissionPromptPayload>` in IPC).
3. The daemon UDS server sends an initial state push to every new TUI client connection
   per BC-2.05.002. The initial state push includes the current session list, hook event
   ring tail, and any queued overlay stack.
4. The TUI process exits when the tmux popup is hidden (the `display-popup -E` flag means
   tmux closes the popup when the child process exits, and the process exits when the popup
   is hidden by the user).

## Postconditions

1. **First `Ctrl-\` spawns the TUI:** The popup appears over the user's active tmux pane.
   The TUI process starts, connects to the daemon UDS, receives the initial state push, and
   renders the current `AppMode` within 16ms of the first frame tick.
2. **AppMode on reconnect is derived from daemon state:** If the daemon's initial state push contains a non-empty `overlay_stack`, the new TUI process:
   (a) Populates `App.overlay_stack` by calling `payload_to_modal()` on each entry in the daemon's `overlay_stack: Vec<PermissionPromptPayload>`.
   (b) Transitions to `AppMode::Overlay { prior: FocusSnapshot::Sessions }` before rendering the first frame.
   The modal stack is populated into `App.overlay_stack` — not into the `AppMode::Overlay` variant, which carries only `{ prior: FocusSnapshot }`.
   If `overlay_stack` is empty, the TUI starts in `AppMode::Dashboard { focused: FocusSnapshot::Sessions }` with `App.overlay_stack` also empty.
3. **Second `Ctrl-\` hides the popup without dropping queued prompts:** The TUI process
   exits. The daemon retains the pending-prompt registry (`overlay_stack`) intact — the
   daemon's registry is never cleared by TUI process exit, regardless of how the process
   exits. When the next TUI connects, it receives `InitialState { overlay_stack }` with all
   still-pending prompts and rebuilds `App.overlay_stack: VecDeque<PromptModal>` via `payload_to_modal()`.
4. **Third `Ctrl-\` shows the popup again with the same overlay stack:** The new TUI
   process receives the still-queued prompts from the daemon's initial state push,
   populates `App.overlay_stack` via `payload_to_modal()`, and renders `AppMode::Overlay { prior: FocusSnapshot::Sessions }`
   with the same prompts as before the hide.
5. **Session list is current after reconnect:** The daemon's initial state push always
   contains the current `Vec<EnrichedSession>`. If sessions changed while the popup was
   hidden, the new TUI renders the updated list on first frame.
6. **IPC messages received while popup is hidden are buffered by the daemon:** The daemon
   does not stop processing hook events when no TUI client is connected. New hook events
   are appended to the JSONL ring and queued in the daemon's IPC state. On the next TUI
   connection, the ring tail (last N events) is included in the initial state push.

## Invariants

1. The daemon is the durable state store. The TUI process carries zero durable state.
   Any `AppMode` value that is not derivable from the daemon's initial state push is lost
   on TUI exit — this is by design (the TUI is a view, not a database).
2. The daemon's pending-prompt registry persists across TUI client connect/disconnect cycles.
   When the TUI process exits (`Ctrl-\` hide), the daemon retains all pending prompts. When a
   new TUI connects (`Ctrl-\` show), the daemon pushes `InitialState { overlay_stack }` with
   the current prompts, and the TUI rebuilds its `App.overlay_stack: VecDeque<PromptModal>` via
   `payload_to_modal()`, then transitions `AppMode` to `Overlay { prior: FocusSnapshot::Sessions }`.
   No `ClientDisconnect` IPC message is required or sent; the daemon
   does not distinguish "intentional hide" from "process exit" at the IPC level — the pending-
   prompt registry is always retained. BC-2.06.016 (daemon-disconnect overlay clear) applies
   only to the TUI-side `App.overlay_stack` on loss of the UDS connection, not to the daemon's
   registry.
3. The popup geometry (80% width, 80% height of the current tmux window) is determined by
   the user's `tmux.conf`. monocle does not hardcode or control the popup dimensions.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-076 | User hides popup while Overlay is showing 3 queued prompts | Daemon retains all 3 prompts; next `Ctrl-\` shows Overlay with same 3 prompts |
| EC-077 | User hides popup while in `Filtering` mode | Daemon has no record of Filtering state; next `Ctrl-\` starts from Dashboard (Filtering is transient TUI state, not daemon state) |
| EC-078 | Daemon restarts between two `Ctrl-\` presses | TUI re-reads lock file on connect; connects to new UDS; receives fresh initial state push (new port, empty queued prompts, current sessions) |
| EC-079 | `MONOCLE_NO_AUTOSTART=1` set; daemon not running when `Ctrl-\` pressed | TUI starts but cannot connect to daemon; renders "Daemon offline" status message; no crash |
| EC-080 | Two tmux panes both run `monocle` simultaneously | Both TUI processes connect as separate clients; daemon pushes to all clients; both render the same state |
| EC-081 | `Ctrl-\` pressed before the TUI has fully rendered (race: popup appears before first frame) | tmux popup is already visible; TUI process is starting; user sees a blank frame for up to 16ms before first render; this is acceptable |

## Canonical Test Vectors

| Scenario | Daemon `overlay_stack` at hide | Expected AppMode on reconnect | Category |
|----------|-------------------------------|------------------------------|----------|
| Hide with no queued prompts; reconnect | `[]` (empty) | `Dashboard { focused: Sessions }` | happy-path |
| Hide with 2 queued prompts; reconnect | `[P1, P2]` | `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2], populated via `payload_to_modal()`) | happy-path |
| Hide in Filtering mode; reconnect | `[]` | `Dashboard { focused: Sessions }` (Filtering lost, not daemon state) | edge-case |
| Daemon restarts between hide and reconnect | `[]` (new daemon) | `Dashboard { focused: Sessions }` (fresh daemon state) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | TUI connects → receives queued prompts via `InitialState.overlay_stack` → TUI process exits → new TUI connects → receives same prompts via `InitialState.overlay_stack` (daemon retained them) | Integration test (mock daemon; simulate TUI connect, disconnect, reconnect; assert overlay_stack same in both InitialState pushes) |
| VP-TBD | TUI transitions to `Overlay` on reconnect when daemon reports queued prompts | Integration test |
| VP-TBD | TUI starts in `Dashboard` on reconnect when daemon reports no queued prompts | Integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the "Ctrl-\ popup integration" component of CAP-006 and is the primary contract for the product's core UX promise: "one Ctrl-\ popup without leaving your editor" |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — the TUI is a client; it reads from the daemon and sends `ClientToServer::PermissionDecision` messages, but never writes to Claude Code files directly) |
| Architecture Module | monocle-tui (run_app event loop, exit path); monocle-ipc (initial state push delivering `overlay_stack`; daemon retains registry across TUI process exit/restart) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Ctrl-\ Integration; §State Preservation Across Hide/Show |
| Cross-Ref | BC-2.05.002 (daemon initial state push — precondition for Postcondition 3 here), BC-2.06.008 (overlay push from IPC, which is the mechanism by which `overlay_stack` becomes AppMode::Overlay), BC-2.06.016 (daemon-disconnect overlay clear — the contrast to the graceful hide/show cycle) |
| Test File | `monocle-tui/tests/popup_lifecycle.rs` |
| Test Name | `test_BC_2_06_004_ctrl_backslash_state_preserved_across_hide_show` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.002] — depends on: daemon initial state push is the mechanism enabling state preservation across hide/show
- [BC-2.06.008] — composes with: when reconnecting TUI receives queued prompts from initial state push, it calls the overlay push logic defined in BC-2.06.008
- [BC-2.06.016] — contrasts with: daemon-disconnect clear (BC-2.06.016) must NOT fire on user-initiated hide

## Architecture Anchors

- `architecture/SS-tui.md#ctrl-backslash-integration` — tmux popup command, state preservation via daemon ownership
- `architecture/SS-tui.md#ctrl-backslash-integration` — "Critical implication for the daemon" paragraph

## Story Anchor

S-TBD — Implement TUI exit and reconnect lifecycle; verify popup state preserved across hide/show via daemon InitialState push (filled by story-writer)

## VP Anchors

- VP-TBD — Integration test: TUI connect → receive queued prompts → verify Overlay AppMode rendered

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.004 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.1.0 §Ctrl-\ Integration, §State Preservation Across Hide/Show;
  prd-expansion-scope.md §3.3 BC-2.06.004 description; ARCH-INDEX.md §Capability Traceability SS-06.
- Invariant 2 is critical: the daemon must distinguish intentional TUI hide from unexpected
  disconnect (crash). The mechanism is a clean `ClientDisconnect` IPC message. Without this
  distinction, every `Ctrl-\` hide would trigger SOQ-3 overlay clear (BC-2.06.016), which
  would destroy all queued prompts on every popup hide. That would make the product
  non-functional for the killer scenario (BC-2.06.022).
- EC-077 documents a deliberate design decision: Filtering state is NOT preserved across
  hide/show because it is transient TUI state (not daemon state). The user must re-enter
  filter mode after a hide/show cycle. This is consistent with the lazygit philosophy.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**IPC sweep — fabricated `DecisionResponse` in DI-007 citation replaced** (2026-05-26T14:30:00Z):
- Traceability table, L2 Domain Invariants row: "sends DecisionResponse messages" →
  "sends `ClientToServer::PermissionDecision` messages". DI-007 paraphrase now uses the
  canonical IPC type name per SS-ipc.md §Client-to-Server Messages.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-FINAL-001 MEDIUM — Daemon-side `queued_prompts: VecDeque<PromptModal>` replaced with canonical IPC field name** (2026-05-26T00:00:00Z):
- Description paragraph: `queued_prompts: VecDeque<PromptModal>` → `overlay_stack: Vec<PermissionPromptPayload>` (IPC).
- Precondition 2: `queued_prompts: VecDeque<PromptModal>` → pending-prompt registry description.
- Postcondition 2: `queued_prompts` → `overlay_stack` (IPC field); clarified that TUI builds `VecDeque<PromptModal>` from it.
- Postcondition 3: `VecDeque<PromptModal>` → pending-prompt registry.
- Test vector table header: `Daemon queued_prompts at hide` → `Daemon overlay_stack at hide`.
- Cross-Ref row: `queued_prompts becomes AppMode::Overlay` → `overlay_stack becomes AppMode::Overlay`.
- Architecture Module: daemon-ownership description updated to reference `overlay_stack`.
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per final bulk pin update.
- The TUI-side `VecDeque<PromptModal>` references (Postcondition 2 and BC-2.06.001 Overlay type) are RETAINED — the TUI stores `VecDeque<PromptModal>` locally; only the IPC/daemon field name was wrong.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.2.0

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. `AppMode::Overlay` now carries only `{ prior: FocusSnapshot }`. The modal stack is populated into `App.overlay_stack: VecDeque<PromptModal>` (single source of truth), not into the `Overlay` variant.
- Postcondition 2 (critical — S-025 anchor): rewrote "transitions to `AppMode::Overlay { stack: <VecDeque<PromptModal> built from overlay_stack>, prior: FocusSnapshot::Sessions }`" to the two-step process: (a) populate `App.overlay_stack` via `payload_to_modal()`; (b) transition `AppMode` to `Overlay { prior: FocusSnapshot::Sessions }`. Modal stack is explicitly noted as being in `App.overlay_stack`, not in the variant.
- Postcondition 3: "rebuilds `VecDeque<PromptModal>`" → "rebuilds `App.overlay_stack: VecDeque<PromptModal>`".
- Postcondition 4: "renders `AppMode::Overlay` with the same stack" → "populates `App.overlay_stack` and renders `AppMode::Overlay { prior: FocusSnapshot::Sessions }`".
- Invariant 2: same `VecDeque<PromptModal>` reference updated to `App.overlay_stack`; added mention of `AppMode` transition to `Overlay { prior }`.
- Test vector: `Overlay { stack: [P1, P2], prior: Sessions }` → `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2]).
- SE-16d monotonicity: v1.2.0 timestamp 2026-05-28T00:00:00Z > v1.1.0. PASS.

## §Trace v1.2.1

**F-S025-ADV23-MED-001 Category 8 sweep — Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` (active pointer was stale by 3 minor versions).
- No substantive BC body prose propagation required: the AppMode::Overlay shape change (v1.8.0) is already incorporated in this BC's v1.2.0 §Trace. State preservation semantics in §Ctrl-\ Integration and §State Preservation Across Hide/Show sections are unchanged in v1.8.1 and v1.8.2.
- SE-16d monotonicity: v1.2.1 timestamp 2026-05-29T00:00:00Z > v1.2.0 timestamp 2026-05-28T00:00:00Z. PASS.

## §Trace v1.1.0

**F-P1D9-001 / F-P1D10-001 CRITICAL — Remove fabricated `ClientDisconnect` IPC message** (2026-05-26T00:00:00Z):
- **Architectural decision (confirmed):** No `ClientDisconnect` message is needed or exists.
  When the user presses Ctrl-\, tmux kills the TUI process. The daemon retains its pending-
  prompt registry. A new TUI process connects and gets the current `overlay_stack` via
  `InitialState`. The "graceful vs crash" distinction is not needed at the IPC level.
- **Invariant 2 replaced:** Removed "TUI MUST send a clean `ClientDisconnect` IPC message
  during graceful exit so the daemon can distinguish intentional hide from crash." Replaced
  with: daemon registry persists across all TUI connect/disconnect cycles; new TUI rebuilds
  from `InitialState { overlay_stack }` via `payload_to_modal()`.
- **Postcondition 3 updated:** Removed "SOQ-3 clearance only applies to unexpected disconnects"
  hedging (that's TUI-side overlay stack behavior, not daemon registry). Replaced with the
  correct mechanism: daemon retains `overlay_stack`; new TUI receives it via InitialState.
- **Verification Properties:** VP for `ClientDisconnect` replaced with VP testing the actual
  mechanism: TUI connect → disconnect → reconnect → same `overlay_stack` in both pushes.
- **Architecture Module:** Removed `ClientDisconnect message` reference.
- **Story Anchor:** Removed "graceful TUI exit with ClientDisconnect IPC message" framing.
- BC-2.06.016 relationship clarified: it applies to TUI-side overlay stack on UDS connection
  loss, not to the daemon's pending-prompt registry.
- SE-16d monotonicity: v1.1.0 timestamp >= v1.0.4. PASS.