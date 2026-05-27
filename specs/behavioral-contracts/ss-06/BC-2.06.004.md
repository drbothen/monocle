---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
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

# Behavioral Contract BC-2.06.004: Ctrl-\ Popup: Appears and Dismisses Without State Loss

## Description

The monocle TUI is launched as a `tmux display-popup` bound to `Ctrl-\` in the user's
`tmux.conf`. Each `Ctrl-\` press either spawns a new `monocle` process (if no popup is
visible) or hides the popup window (if visible). Because the TUI is a stateless view over
the daemon's durable state, the overlay stack and session list survive the hide/show cycle:
the daemon owns `queued_prompts: VecDeque<PromptModal>` and pushes the current state to
every new TUI client on connection. The TUI process itself is short-lived; the daemon is
the durable state store.

## Preconditions

1. The user's `tmux.conf` contains a binding equivalent to:
   `bind-key -n C-\\ display-popup -E -w 80% -h 80% 'monocle'`
2. A monocle daemon is running (or auto-starts per BC-2.04.002) and holds the current
   `DaemonState` including `queued_prompts: VecDeque<PromptModal>`.
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
2. **AppMode on reconnect is derived from daemon state:** If the daemon reports any
   `queued_prompts` in its initial state push, the new TUI process transitions to
   `AppMode::Overlay { stack: queued_prompts, prior: FocusSnapshot::Sessions }` before
   rendering the first frame. If no queued prompts, it starts in
   `AppMode::Dashboard { focused: FocusSnapshot::Sessions }`.
3. **Second `Ctrl-\` hides the popup without dropping queued prompts:** The TUI process
   exits. The daemon retains the `VecDeque<PromptModal>` intact. The queued prompts are
   NOT cleared on TUI disconnect (SOQ-3 clearance only applies to unexpected disconnects —
   see BC-2.06.016; a user-initiated `Ctrl-\` hide is NOT an unexpected disconnect).
4. **Third `Ctrl-\` shows the popup again with the same overlay stack:** The new TUI
   process receives the still-queued prompts from the daemon's initial state push and
   renders `AppMode::Overlay` with the same stack as before the hide.
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
2. A user-initiated hide via `Ctrl-\` (tmux closes the popup, TUI process exits) MUST NOT
   trigger `BC-2.06.016` (daemon-disconnect overlay clear). The distinction is signaled by
   whether the TUI sends a clean disconnect message before exiting. The TUI MUST send a
   clean `ClientDisconnect` IPC message during graceful exit so the daemon can distinguish
   intentional hide from crash.
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

| Scenario | Daemon `queued_prompts` at hide | Expected AppMode on reconnect | Category |
|----------|--------------------------------|------------------------------|----------|
| Hide with no queued prompts; reconnect | `[]` (empty) | `Dashboard { focused: Sessions }` | happy-path |
| Hide with 2 queued prompts; reconnect | `[P1, P2]` | `Overlay { stack: [P1, P2], prior: Sessions }` | happy-path |
| Hide in Filtering mode; reconnect | `[]` | `Dashboard { focused: Sessions }` (Filtering lost, not daemon state) | edge-case |
| Daemon restarts between hide and reconnect | `[]` (new daemon) | `Dashboard { focused: Sessions }` (fresh daemon state) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | TUI sends `ClientDisconnect` IPC message on graceful exit | Integration test (mock IPC server records received messages) |
| VP-TBD | TUI transitions to `Overlay` on reconnect when daemon reports queued prompts | Integration test |
| VP-TBD | TUI starts in `Dashboard` on reconnect when daemon reports no queued prompts | Integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the "Ctrl-\ popup integration" component of CAP-006 and is the primary contract for the product's core UX promise: "one Ctrl-\ popup without leaving your editor" |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — the TUI is a client; it reads from the daemon and sends DecisionResponse messages, but never writes to Claude Code files directly) |
| Architecture Module | monocle-tui (run_app event loop, graceful exit path); monocle-ipc (initial state push, ClientDisconnect message) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.0.0 §Ctrl-\ Integration; §State Preservation Across Hide/Show |
| Cross-Ref | BC-2.05.002 (daemon initial state push — precondition for Postcondition 3 here), BC-2.06.008 (overlay push from IPC, which is the mechanism by which queued_prompts becomes AppMode::Overlay), BC-2.06.016 (daemon-disconnect overlay clear — the contrast to the graceful hide/show cycle) |
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

S-TBD — Implement graceful TUI exit with ClientDisconnect IPC message; verify popup state preserved across hide/show (filled by story-writer)

## VP Anchors

- VP-TBD — Integration test: TUI connect → receive queued prompts → verify Overlay AppMode rendered

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.004 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.0.0 §Ctrl-\ Integration, §State Preservation Across Hide/Show;
  prd-expansion-scope.md §3.3 BC-2.06.004 description; ARCH-INDEX.md §Capability Traceability SS-06.
- Invariant 2 is critical: the daemon must distinguish intentional TUI hide from unexpected
  disconnect (crash). The mechanism is a clean `ClientDisconnect` IPC message. Without this
  distinction, every `Ctrl-\` hide would trigger SOQ-3 overlay clear (BC-2.06.016), which
  would destroy all queued prompts on every popup hide. That would make the product
  non-functional for the killer scenario (BC-2.06.022).
- EC-077 documents a deliberate design decision: Filtering state is NOT preserved across
  hide/show because it is transient TUI state (not daemon state). The user must re-enter
  filter mode after a hide/show cycle. This is consistent with the lazygit philosophy.
