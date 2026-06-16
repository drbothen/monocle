---
document_type: story
level: L4
story_id: S-044
epic_id: EPIC-09
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 13
wave: 9
tdd_mode: strict
priority: P0
depends_on: [S-033, S-035, S-040, S-041]
blocks: []
target_module: monocle-tui
subsystems: [SS-09]
behavioral_contracts: [BC-2.09.008, BC-2.09.009]
verification_properties: []
estimated_days: 6
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.008.md, version: "1.3.1"}
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.009.md, version: "1.1.2"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.6.0"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.23.2"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.1"}
input-hash: "[pending]"
traces_to: "Implements BC-2.09.008 (EmbeddedTerminal/SessionCreation AppMode enter/exit transitions; wizard auto-transition; SpawnAck; launching_session_id) and BC-2.09.009 (permission badge + bell while in EmbeddedTerminal or SessionCreation)"
# BC status: BC-2.09.008 v1.3.2, BC-2.09.009 v1.1.3 — non-empty; status draft pending Phase-2 adversarial convergence gate
# Clustering rationale: BC-2.09.008 (AppMode transitions) and BC-2.09.009 (permission badge+bell) are clustered
# because the badge-and-bell behavior fires INSIDE EmbeddedTerminal and SessionCreation modes, which BC-2.09.008 defines.
# Implementing one without the other leaves an incomplete AppMode entry/exit contract. BC-2.09.009's
# overlay_stack integration also extends the exit path defined by BC-2.09.008. The stories share
# the same monocle-tui files (app.rs event handler, status bar renderer) with no additional seam.
# Also captures the SS-09 obligation from Burst A GAP-4: BC-2.08.008 PC-9 specifies that TUI-side
# EmbeddedTerminal exits on SessionStateChanged::Terminated — that TUI-side behavior is the SS-09 scope
# of the Burst A gap and is specified in BC-2.09.008 Postconditions (exiting EmbeddedTerminal) PC-3.
---

# S-044: EmbeddedTerminal + SessionCreation AppMode Transitions, SessionCreation Wizard, SpawnAck, and Permission Badge+Bell

## Narrative

As the monocle TUI user, I want to enter embedded terminal mode on a running session (or
launch a new session via the SessionCreation wizard), have the TUI auto-transition to
`EmbeddedTerminal` when the session reaches `Running`, and — while in embedded terminal or
wizard mode — immediately see a status bar badge and hear an audible bell for any incoming
permission prompt, so that the monocle killer feature (permission management) is never silently
suppressed regardless of what I am doing in the TUI.

## Acceptance Criteria

### AC-001 (traces to BC-2.09.008 postcondition entering EmbeddedTerminal — PC-1: AppMode transitions to EmbeddedTerminal)

When a user presses Enter on a `Running` session in the sessions panel:
- `AppMode` transitions to `EmbeddedTerminal { session_id: <running session id>, prior: <current FocusSnapshot> }`.
- `enter_embedded_terminal(session_id)` executes (scoped mouse capture, keyboard context switch, auto-attach-on-first-entry per S-039/S-040/S-041).

### AC-002 (traces to BC-2.09.008 postcondition entering EmbeddedTerminal — PC-3: keyboard context; scroll actions active)

In `AppMode::EmbeddedTerminal`, `Action::PtyScrollUp`, `Action::PtyScrollDown`, and the
Esc intercept are active. All other keystrokes are forwarded to PTY.

### AC-003 (traces to BC-2.09.008 postcondition entering EmbeddedTerminal — PC-5: status bar indicator)

The status bar shows `[EmbeddedTerminal: <session_display_name>]` while in `EmbeddedTerminal` mode.

### AC-004 (traces to BC-2.09.008 postcondition entering SessionCreation — PC-1/PC-2: wizard starts at ProfilePicker)

When the user triggers "new session" action (e.g., `n` key while in sessions panel):
- `AppMode` transitions to `SessionCreation { step: ProfilePicker, prior: <current FocusSnapshot>, launching_session_id: None }`.
- The ProfilePicker step renders using existing profile-picker logic (BC-2.07.004/005).

### AC-005 (traces to BC-2.09.008 postcondition entering SessionCreation — PC-3/PC-4: ProjectPicker and WorktreeConfirm steps)

Step 2 (ProjectPicker): nucleo-filtered list of recent project roots + free-text entry. Navigation
(Enter to select, Esc to cancel) advances or cancels the wizard.
Step 3 (WorktreeConfirm): displays resolved worktree path + editable display name. Validates the
path (exists + git work-tree check) before allowing Confirm. Validation failure shows inline error
and stays on Step 3.

### AC-006 (traces to BC-2.09.008 postcondition entering SessionCreation — PC-5/PC-6: Step 4 Launching; SpawnAck receipt; launching_session_id populated)

On Step 4 (Launching):
1. The TUI sends `ClientToServer::SpawnSession { opts }` to the daemon.
2. Status bar shows `[Launching session...]`.
3. On receipt of `ServerToClient::SpawnAck { session_id }`, the wizard stores it:
   `AppMode::SessionCreation { launching_session_id: Some(session_id.clone()), .. }`.
   `SpawnAck` is point-to-point (requesting client only; NOT broadcast). It arrives before any
   broker-published `SessionStateChanged { Launching }` by TWO complementary properties:
   (1) **Causal step ordering**: `SpawnAck` is sent by the daemon IPC handler at step 2, before
   `spawn_session()` at step 4, before the broker emits `SessionStateChanged { Launching }` at step 5.
   (2) **Per-client FIFO**: the requesting client's `mpsc` channel delivers messages in send order.
4. When `SessionStateChanged { session_id: <matching launching_session_id>, new_state: Running }` is
   received, `AppMode` auto-transitions to `EmbeddedTerminal { session_id, prior: Dashboard }`.
   The match is against `launching_session_id` (deterministic — from `SpawnAck`), NOT a broadcast heuristic.
   `SessionStateChanged` events not matching `launching_session_id` are ignored by the wizard.

### AC-007 (traces to BC-2.09.008 postcondition entering SessionCreation — PC-7: spawn-fail clears launching_session_id and returns to ProfilePicker)

If `ServerToClient::Error` is received after `SpawnSession`:
- `AppMode::SessionCreation.launching_session_id` is cleared to `None`.
- The wizard returns to `ProfilePicker` step with an error banner.
- No session is created.

### AC-008 (traces to BC-2.09.008 postcondition exiting EmbeddedTerminal — PC-1: Esc exits to prior; scoped mouse exit)

Esc in `AppMode::EmbeddedTerminal` fires `Action::ExitEmbeddedTerminal`:
- `exit_embedded_terminal()` executes (SGR 1006l + `DisableMouseCapture` in that order per BC-2.09.002 Invariant-5 / S-041).
- `AppMode` transitions to `prior` AppMode (typically `Dashboard`).
- `pty_scroll_offsets[session_id]` is reset to 0 (exit resets scroll position per BC-2.09.008 PC-4).
- If `overlay_stack` is non-empty, `AppMode` immediately transitions to `AppMode::Overlay { prior: Dashboard }` to display the front of the stack.

### AC-009 (traces to BC-2.09.008 postcondition exiting EmbeddedTerminal — PC-2: Ctrl-D EOT forwarded; auto-exit on Terminated)

`Ctrl-D` in `EmbeddedTerminal` is forwarded as `\x04` (EOT) per S-040. When the resulting
`SessionStateChanged { new_state: Terminated }` arrives for `session_id`, `AppMode` auto-transitions
to `prior` AppMode. No manual Esc required.

### AC-010 (traces to BC-2.09.008 postcondition exiting EmbeddedTerminal — PC-3: SessionStateChanged::Terminated auto-exits; GAP-4 TUI-side obligation)

When `ServerToClient::SessionStateChanged { session_id, new_state: Terminated }` is received while
`AppMode::EmbeddedTerminal { session_id: matching, .. }` is active:
- `exit_embedded_terminal()` executes.
- `AppMode` transitions to `prior`.
- Status bar shows `[Session terminated]`.

This is the TUI-side obligation of Burst A GAP-4 (BC-2.08.008 PC-9): the TUI MUST exit
`EmbeddedTerminal` when the session it is displaying terminates. The daemon-side of this
obligation (broadcasting `SessionStateChanged::Terminated`) is implemented in S-033/S-034.

### AC-011 (traces to BC-2.09.008 invariant 1 — non-Running states are no-ops with status bar message)

Attempting to enter `EmbeddedTerminal` for a non-Running session produces the following per-state behavior:
- `Terminated`: no-op; status bar `"Session not running (state: Terminated)"`.
- `Terminating`: no-op; status bar `"Session not running (state: Terminating)"`.
- `Launching`: no-op; status bar `"Session launching — please wait"`.
- `Detached`: NOT a no-op. `attach_session()` is triggered automatically. `EmbeddedTerminal` is entered after `SessionStateChanged { new_state: Running }` confirming re-attach.

`SessionState::Killed` does NOT exist in the reachable state set and MUST NOT be referenced.

### AC-012 (traces to BC-2.09.008 edge case EC-250 — Launching session; no-op with message)

Entering `EmbeddedTerminal` for a `Launching` session is a no-op. Status bar shows
`"Session launching — please wait"`.

### AC-013 (traces to BC-2.09.008 edge case EC-251 — session terminates while in EmbeddedTerminal)

Same as AC-010. AppMode auto-transitions to `prior`; status bar `"Session terminated"`.

### AC-014 (traces to BC-2.09.008 edge case EC-252 — SessionCreation spawn fails)

Wizard returns to `ProfilePicker` with error banner; `launching_session_id` cleared. No session created.
(Same as AC-007.)

### AC-015 (traces to BC-2.09.008 edge case EC-253 — SessionCreation cancelled via Esc)

Esc at any wizard step cancels the wizard. `AppMode` transitions to `prior`. No session created.

### AC-016 (traces to BC-2.09.009 postcondition 1–4 — permission badge rendered within one render tick)

When `ServerToClient::PermissionPromptQueued { ... }` is received while `AppMode::EmbeddedTerminal`
OR `AppMode::SessionCreation` is active:
1. The payload is added to `App::overlay_stack` (per BC-2.06.008 — existing behavior; always active).
2. Within one render tick: the status bar renders `[N pending permission(s)]` (where N = `overlay_stack.len()`).
3. `\x07` (BEL character) is written to stdout — once per `PermissionPromptQueued` event, including the second and subsequent prompts. Every new prompt rings the bell.
4. The badge is visible in the status bar even while the PTY widget occupies the main pane.

### AC-017 (traces to BC-2.09.009 invariant 1 — no silent queueing; badge is mandatory minimum)

Incoming `PermissionPromptQueued` messages MUST NOT be held invisibly while in embedded terminal
or wizard mode. The status bar badge + bell is the production-grade non-negotiable minimum. Any
implementation that defers badge rendering or bell emission until after mode exit violates this
invariant.

### AC-018 (traces to BC-2.09.009 invariant 2 — bell fired per-prompt; not capped after first)

The BEL character (`\x07`) is written to stdout exactly once per `PermissionPromptQueued` event.
This applies to the second, third, and subsequent prompts in rapid succession — every prompt
rings the bell independently. Rationale: each new blocking permission is an independent attention
demand. This rule is consistent with HS-EXP-013 step 7.

### AC-019 (traces to BC-2.09.009 edge case EC-260 — two prompts in rapid succession)

Two `PermissionPromptQueued` events arrive rapidly:
- Badge shows `[2 pending permissions]` (or `[2 pending permission(s)]`).
- Bell (`\x07`) emitted twice (once per event).

### AC-020 (traces to BC-2.09.009 edge case EC-261 — prompt during SessionCreation::Launching)

`PermissionPromptQueued` received while in `SessionCreation::Launching` step:
- Badge appears in status bar.
- Bell emitted.
- Wizard continues uninterrupted.
- User can Esc to cancel wizard and reach overlay.

### AC-021 (traces to BC-2.09.009 edge case EC-264 — Esc in EmbeddedTerminal with no pending overlays)

When `overlay_stack` is empty and user presses Esc in `EmbeddedTerminal`:
- AppMode → `prior` (Dashboard).
- No overlay displayed.
- Normal Dashboard render.

## Tasks

- [ ] Implement sessions-panel `Enter` key handler: check `SessionEntry.state`; for `Running` → `enter_embedded_terminal()`; for `Detached` → trigger `attach_session()` + wait; for others → status bar message (no-op).
- [ ] Implement `Action::NewSession` handler (e.g., `n` key in sessions panel): transition to `AppMode::SessionCreation { step: ProfilePicker, prior: current, launching_session_id: None }`.
- [ ] Implement SessionCreation wizard multi-step UI in `crates/monocle-tui/src/ui/session_creation.rs`:
  - `ProfilePicker` step: renders existing profile-picker component (BC-2.07.004/005).
  - `ProjectPicker` step: nucleo-filtered recent project roots + free-text entry.
  - `WorktreeConfirm` step: path display + edit; validate path (exists + git work-tree); inline error on failure.
  - `Launching` step: "Launching session..." status bar; sends `ClientToServer::SpawnSession { opts }`.
- [ ] Wire `ServerToClient::SpawnAck { session_id }` IPC arm: store into `AppMode::SessionCreation.launching_session_id = Some(session_id)`.
- [ ] Wire `ServerToClient::SessionStateChanged { session_id, new_state: Running }` in wizard context: match against `launching_session_id`; if match → `enter_embedded_terminal(session_id)`.
- [ ] Wire `ServerToClient::SessionStateChanged { session_id, new_state: Terminated }` in `EmbeddedTerminal` context: call `exit_embedded_terminal()`; transition to `prior`; status bar `"Session terminated"`. (GAP-4 TUI-side obligation.)
- [ ] Implement `ServerToClient::Error` handler in wizard context: clear `launching_session_id = None`; return to `ProfilePicker` with error banner.
- [ ] Implement Esc handler in `SessionCreation` (all steps): cancel wizard; transition to `prior`.
- [ ] Add `PermissionPromptQueued` IPC arm: always add to `overlay_stack` (BC-2.06.008 path — if not already wired); ADDITIONALLY, if `AppMode` is `EmbeddedTerminal` or `SessionCreation`, write `\x07` to stdout and trigger immediate re-render.
- [ ] Implement status bar badge: if `overlay_stack.len() > 0` in `EmbeddedTerminal` or `SessionCreation` modes, render `[N pending permission(s)]` in the status bar.
- [ ] Implement Esc-from-EmbeddedTerminal-with-overlay: after `exit_embedded_terminal()`, if `overlay_stack.is_not_empty()` → transition to `AppMode::Overlay`.
- [ ] Write unit test `test_BC_2_09_008_enter_running_session_transitions_to_embedded_terminal`.
- [ ] Write unit test `test_BC_2_09_008_enter_launching_session_noop_status_message`.
- [ ] Write unit test `test_BC_2_09_008_esc_exits_to_prior`.
- [ ] Write unit test `test_BC_2_09_008_session_terminated_auto_exits_embedded_terminal`: (GAP-4 TUI obligation) assert `AppMode::EmbeddedTerminal` → `prior` on `SessionStateChanged::Terminated`.
- [ ] Write unit test `test_BC_2_09_008_session_creation_wizard_full_flow`: ProfilePicker → ProjectPicker → WorktreeConfirm → Launching → SpawnAck received → Running received → EmbeddedTerminal.
- [ ] Write unit test `test_BC_2_09_008_wizard_spawn_ack_before_state_changed`: verify `launching_session_id` is `Some()` before first `SessionStateChanged{Launching}` arrives.
- [ ] Write unit test `test_BC_2_09_008_wizard_spawn_fail_returns_to_profile_picker`.
- [ ] Write unit test `test_BC_2_09_008_wizard_esc_cancels`.
- [ ] Write unit test `test_BC_2_09_009_permission_badge_rendered_in_embedded_terminal`: `PermissionPromptQueued` received; assert badge rendered; bell `\x07` written.
- [ ] Write unit test `test_BC_2_09_009_bell_per_prompt_not_once`: two `PermissionPromptQueued` events; assert two `\x07` writes.
- [ ] Write unit test `test_BC_2_09_009_esc_from_embedded_with_overlay_transitions_to_overlay`.
- [ ] Write unit test `test_BC_2_09_009_esc_from_embedded_without_overlay_goes_to_prior`.

## Previous Story Intelligence

- **S-033** (session-manager spawn): `ServerToClient::SpawnAck { session_id }` IPC variant exists; `SpawnSession` IPC handling on daemon side is live. The wizard's Step 4 `ClientToServer::SpawnSession` send will reach a live daemon handler.
- **S-035** (attach/detach): `ClientToServer::AttachSession` and the `attach_session()` daemon-side behavior are live. The `Detached` session entry path (AC-011) can trigger actual attach.
- **S-040** (keyboard forwarding): `enter_embedded_terminal()` / `exit_embedded_terminal()` skeletons are in place; keyboard setup done. This story adds the `SessionStateChanged::Terminated` auto-exit wiring and the overlay-stack post-exit transition.
- **S-041** (mouse forwarding): Scoped `EnableMouseCapture`/`DisableMouseCapture` are already wired in `enter_embedded_terminal()` / `exit_embedded_terminal()`. Do NOT duplicate.
- **S-026** (permission overlay core): `App::overlay_stack: VecDeque<PermissionModal>` exists. `PermissionPromptQueued` → `overlay_stack.push_back()` may already be wired. Verify; if so, this story ONLY adds the badge + bell side effect when `AppMode` is `EmbeddedTerminal` or `SessionCreation`.
- **S-031** (profile picker): The `ProfilePicker` UI component exists. The wizard's Step 1 reuses it directly.
- **GAP-4 from Burst A**: The `SessionStateChanged::Terminated` TUI-side auto-exit from `EmbeddedTerminal` was identified as a gap in Burst A (BC-2.08.008 PC-9 TUI-side obligation). This story is the canonical resolution vehicle. The session-manager-side broadcast (daemon → TUI) is live from S-033/S-034; the TUI handler is implemented here.

## Architecture Compliance Rules

- `AppMode::SessionCreation.launching_session_id` MUST be `None` in all steps except `Launching` (populated by `SpawnAck`) and MUST be cleared to `None` on wizard exit (success, failure, or cancellation). Do NOT leave a stale session_id in the field after the wizard ends.
- `SpawnAck` match must be gated: only process `SpawnAck { session_id }` while `AppMode::SessionCreation { step: Launching, .. }` is active. Ignore `SpawnAck` in any other AppMode.
- `SessionStateChanged::Running` wizard auto-advance must match against `launching_session_id` — NOT against "any Running event". This is the EC-303 deterministic filter.
- Bell (`\x07`) is written to `stdout()` via `print!("\x07")` or equivalent; it is NOT a crossterm command. Write directly, flush if necessary.
- `overlay_stack` is a `VecDeque<PermissionModal>` (BC-2.06.026 / S-026 convention). The badge shows `len()` as "N pending permission(s)". The plural logic: N=1 → "1 pending permission"; N>1 → "N pending permissions".
- `SessionState::Killed` does NOT exist. Any `match` on `SessionState` that includes `Killed` is a compile error.
- Forbidden dependency: `monocle-tui` MUST NOT depend on `monocle-runtime`. Session state comes from IPC messages, not from internal `SessionManager` access.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tokio` | `=1.52` (exact) | Async IPC send (`SpawnSession`); timer | SS-deps-pin-manifest.md §Exact-pinned |
| `crossterm` | `"0.29"` (caret) | Mouse capture exit (via S-041 wiring) | SS-deps-pin-manifest.md |
| `ratatui` | `"0.30"` (caret) | Badge rendering in status bar; wizard UI layout | SS-deps-pin-manifest.md |
| `nucleo` | `"0.5"` (caret) | ProjectPicker fuzzy filtering | SS-deps-pin-manifest.md (ADR-0002) |
| `tracing` | `"0.1"` | TRACE/INFO logging for wizard steps and badge events | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to CREATE:

| File | Purpose |
|------|---------|
| `crates/monocle-tui/src/ui/session_creation.rs` | SessionCreation wizard: ProfilePicker/ProjectPicker/WorktreeConfirm/Launching step renderers; navigation logic |

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-tui/src/app.rs` | Add `PermissionPromptQueued` handler with badge+bell side effect; add `SessionStateChanged::Terminated` auto-exit from EmbeddedTerminal; add Esc-then-overlay transition; add `Action::NewSession` handler; wire wizard navigation handlers |
| `crates/monocle-tui/src/event_loop.rs` | Wire `ServerToClient::SpawnAck` arm; wire `SessionStateChanged::Running` wizard match; wire `ServerToClient::Error` wizard fail path |
| `crates/monocle-tui/src/ui/status_bar.rs` (or equivalent) | Add `[N pending permission(s)]` badge rendering when in EmbeddedTerminal or SessionCreation; add `[EmbeddedTerminal: <name>]` and `[Launching session...]` indicators |
| `crates/monocle-tui/src/ui/mod.rs` | `pub mod session_creation;` |
| `crates/monocle-ipc/src/lib.rs` | Ensure `ServerToClient::SpawnAck { session_id: String }` and `ClientToServer::SpawnSession { opts: SpawnOptions }` exist; add if absent |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~6,500 |
| BC-2.09.008 | ~5,500 |
| BC-2.09.009 | ~3,000 |
| SS-embedded-pty.md §TUI AppMode Extensions; §Session Creation Wizard; §State machine invariants; §Permission prompts | ~10,000 |
| SS-ipc.md §ServerToClient::SpawnAck (delivery ordering, steps 1–5) | ~2,500 |
| Existing app.rs (S-039/S-040/S-041 context) + event_loop.rs | ~8,000 |
| S-026 overlay_stack context | ~2,000 |
| Test files to write | ~7,000 |
| **Total estimate** | **~44,500** |

Within the 30% context window bound for a Sonnet-class model (~200k = 60k max per story). No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.09.008 | EmbeddedTerminal AppMode Enter/Exit Transitions; SessionCreation Wizard Auto-Transitions to EmbeddedTerminal | v1.3.1 |
| BC-2.09.009 | Permission Badge+Bell — Status Bar Badge + Audible Bell Within One Render Tick While in EmbeddedTerminal or SessionCreation | v1.1.2 |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `AppMode::EmbeddedTerminal` transition logic | `monocle-tui/src/app.rs` | Effectful shell (AppMode state machine + IPC trigger) |
| `AppMode::SessionCreation` wizard state machine | `monocle-tui/src/app.rs` + `ui/session_creation.rs` | Effectful shell (UI render + IPC send) |
| `SessionCreation.launching_session_id` field | `monocle-core/src/app_mode.rs` | Pure core (in-memory Option<String>) |
| `PermissionPromptQueued` badge + bell handler | `monocle-tui/src/app.rs` | Effectful shell (stdout bell + render trigger) |
| Status bar badge renderer | `monocle-tui/src/ui/status_bar.rs` | Effectful shell (ratatui render) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-250 | Enter EmbeddedTerminal for Launching session | No-op; status bar message |
| EC-251 | Session terminates while EmbeddedTerminal active | Auto-exit to prior; status bar "Session terminated" |
| EC-252 | SessionCreation spawn fails | Wizard → ProfilePicker with error banner |
| EC-253 | SessionCreation cancelled via Esc | AppMode → prior; no session created |
| EC-254 | Enter EmbeddedTerminal while daemon disconnected | IPC attach fails; status bar "Daemon disconnected"; Dashboard |
| EC-260 | Two PermissionPromptQueued in rapid succession | Badge `[2 pending permissions]`; two bells |
| EC-261 | Prompt during SessionCreation::Launching | Badge shown; bell emitted; wizard continues |
| EC-264 | Esc from EmbeddedTerminal with no pending overlays | AppMode → prior (Dashboard); no overlay |

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because:
- `AppMode::EmbeddedTerminal` and `AppMode::SessionCreation` are defined in SS-embedded-pty.md §TUI AppMode Extensions — the core SS-09 AppMode specification.
- `SessionCreation` wizard (4 steps + SpawnAck + auto-advance) is defined in SS-embedded-pty.md §Session Creation Wizard.
- Permission badge + bell in EmbeddedTerminal is defined in SS-embedded-pty.md §State machine invariants (SUG-3 fix) and BC-2.09.009.

**Dependency Anchors:**
- S-044 depends on S-033 because `ServerToClient::SpawnAck` is authored by the daemon's IPC handler (implemented in S-033); the wizard's Step 4 consumes it.
- S-044 depends on S-035 because the `Detached` session entry path triggers `attach_session()` (implemented in S-035) as part of `EmbeddedTerminal` entry.
- S-044 depends on S-040 because S-040 establishes `enter_embedded_terminal()` / `exit_embedded_terminal()` and the Esc intercept in the event loop — both of which S-044 extends with wizard logic and overlay transitions.
- S-044 depends on S-041 because S-041 wires the scoped mouse capture into `enter_embedded_terminal()` / `exit_embedded_terminal()`. S-044 must not duplicate that wiring.
- S-044 does not block other SS-09 stories — it is the final story in the EPIC-09 dependency chain.
