---
document_type: behavioral-contract
level: L3
version: "1.1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "d80f11d"
traces_to: prd.md
origin: greenfield
subsystem: SS-09
capability: CAP-009
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.09.008: EmbeddedTerminal AppMode Enter/Exit Transitions; SessionCreation Wizard Auto-Transitions to EmbeddedTerminal

## Description

`AppMode::EmbeddedTerminal { session_id, prior }` is entered via an explicit action on a
`Running` session. `AppMode::SessionCreation` is a multi-step wizard for launching new
sessions; it auto-transitions to `AppMode::EmbeddedTerminal` when the spawned session
reaches `SessionState::Running`. Exiting `EmbeddedTerminal` via Esc restores the `prior`
AppMode. A `Ctrl-D` or session termination also exits `EmbeddedTerminal` automatically.

## Preconditions (entering EmbeddedTerminal)

1. A session exists in the registry with `SessionState::Running`.
2. The user presses Enter (or equivalent action) on the session in the sessions panel, OR
   `SessionCreation::Launching` receives `SessionStateChanged { new_state: Running }`.

## Preconditions (SessionCreation wizard)

1. `AppMode::SessionCreation` is not already active.
2. The user triggers "new session" action (e.g., `n` in sessions panel).

## Postconditions (entering EmbeddedTerminal)

1. `AppMode` transitions to `EmbeddedTerminal { session_id: <running session id>, prior: <current FocusSnapshot> }`.
2. If the session-host is `Detached`, `attach_session()` is triggered automatically to
   begin receiving PTY output before rendering.
3. Keyboard enhancement context switches: `Action::PtyScrollUp`, `Action::PtyScrollDown`,
   and the Esc intercept are active. All other keystrokes are forwarded to PTY.
4. SGR mouse mode is written to the terminal (`ESC [ ? 1006 h`).
5. A status bar indicator shows `[EmbeddedTerminal: <session_display_name>]`.

## Postconditions (entering SessionCreation)

1. `AppMode` transitions to `SessionCreation { step: ProfilePicker, prior: <current FocusSnapshot> }`.
2. Step 1 (ProfilePicker): existing profile-picker UI (BC-2.07.004/005 logic) renders.
3. Step 2 (ProjectPicker): nucleo-filtered list of recent project roots; free-text entry.
4. Step 3 (WorktreeConfirm): editable display name + resolved worktree path.
5. Step 4 (Launching): TUI sends `ClientToServer::SpawnSession` to daemon. Status bar shows
   `[Launching session...]`. When `ServerToClient::SessionStateChanged { new_state: Running }` is
   received, AppMode auto-transitions to `EmbeddedTerminal { session_id, prior: Dashboard }`.
6. If spawn fails (daemon returns error): wizard returns to `ProfilePicker` with an error banner.

## Postconditions (exiting EmbeddedTerminal)

1. Esc in `EmbeddedTerminal`: `Action::ExitEmbeddedTerminal` fires. AppMode transitions
   to `prior` AppMode (typically `Dashboard`). SGR mouse mode disabled (`ESC [ ? 1006 l`).
2. `Ctrl-D` in `EmbeddedTerminal`: forwarded to PTY as `\x04`; when session transitions to
   `Terminated`, AppMode auto-exits to `prior`.
3. `SessionStateChanged { new_state: Terminated }` received while in `EmbeddedTerminal`:
   AppMode auto-transitions to `prior`; status bar shows `[Session terminated]`.
4. On any exit: `pty_scroll_offsets[session_id]` for this session is reset to 0.

## Invariants

1. `EmbeddedTerminal` entry requires `SessionState::Running` as a direct-entry precondition.
   The behavior for other session states is defined per-state as follows (per SS-embedded-pty.md
   lines ~101-102 and BC-2.09.001 PC-6; `SessionState::Killed` is RETIRED and does NOT exist):
   - **`Terminated`:** No-op with status bar message `"Session not running (state: Terminated)"`.
     The terminal and its process no longer exist; no attach is possible.
   - **`Terminating`:** No-op with status bar message `"Session not running (state: Terminating)"`.
     The kill is in-flight; attaching is not safe. User must wait for `Terminated`.
   - **`Launching`:** No-op with status bar message `"Session launching — please wait"`. The
     session-host UDS socket is not yet connectable; entry is deferred until `Running` is received.
   - **`Detached`:** NOT a no-op. A `Detached` session is alive and attachable. Per PC-2,
     `attach_session()` is triggered automatically when a `Detached` session is selected.
     `AppMode::EmbeddedTerminal` is entered after `SessionStateChanged { new_state: Running }`
     is received confirming the re-attach. See BC-2.08.007 for `attach_session()` behavior.
   `SessionState::Killed` does NOT exist in the reachable state set (removed per
   SS-session-manager.md §Session lifecycle state machine I4 audit — superseded by
   `Terminating`). Any reference to `Killed` in this context is a retroactive error.
2. `SessionCreation` is mutually exclusive with `Overlay` — permission overlays cannot
   appear while the wizard is active (they are held in the daemon overlay_stack and visible
   only via the status bar badge per BC-2.09.009).
3. `AppMode::EmbeddedTerminal` and `AppMode::SessionCreation` BOTH suppress standard
   keybinding dispatch. All keystrokes are routed to the PTY (in EmbeddedTerminal) or to
   wizard navigation (in SessionCreation).
4. `prior: FocusSnapshot` is captured at the moment of entry, NOT at exit. It represents
   the AppMode immediately before `EmbeddedTerminal` was entered.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-250 | Enter `EmbeddedTerminal` for a `Launching` session | No-op; status bar `"Session launching — please wait"` |
| EC-251 | Session transitions to `Terminated` while `EmbeddedTerminal` active | AppMode auto-transitions to `prior`; status bar `"Session terminated"` |
| EC-252 | `SessionCreation` spawn fails (daemon error) | Wizard returns to `ProfilePicker` with error banner; no session created |
| EC-253 | `SessionCreation` cancelled via Esc (any step) | `AppMode` transitions to `prior`; no session created |
| EC-254 | Enter `EmbeddedTerminal` while daemon is disconnected | IPC attach fails; status bar `"Daemon disconnected"`; AppMode stays in `Dashboard` |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Running session focused in sessions panel; user presses Enter | AppMode → `EmbeddedTerminal { session_id }`; PTY widget renders | happy-path |
| SessionCreation wizard; all steps; spawn succeeds | AppMode → `EmbeddedTerminal` when Running; PTY output visible | happy-path |
| SessionCreation wizard; spawn fails | Wizard back to ProfilePicker with error banner | error |
| Esc in EmbeddedTerminal | AppMode → `prior` (Dashboard) | happy-path |
| Session terminates while in EmbeddedTerminal | AppMode → `prior`; status bar message | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `EmbeddedTerminal` entry requires `Running` state; other states are no-ops | unit |
| VP-TBD | Esc exits EmbeddedTerminal to prior AppMode | unit |
| VP-TBD | SessionCreation auto-transitions to EmbeddedTerminal on Running state | unit |
| VP-TBD | Session termination auto-exits EmbeddedTerminal | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — session creation wizard and EmbeddedTerminal AppMode are both explicitly named in CAP-009; this BC covers the enter/exit transitions and the wizard auto-transition to EmbeddedTerminal |
| Architecture Module | monocle-core (AppMode::EmbeddedTerminal, AppMode::SessionCreation variants, SessionCreationStep enum); monocle-tui (transition logic, wizard UI) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md v1.5.0 §TUI AppMode Extensions (EmbeddedTerminal, SessionCreation, SessionCreationStep); §Session Creation Wizard; §State machine invariants |
| Test Name | test_BC_2_09_008_embedded_terminal_transitions |

## Related BCs

- [BC-2.09.009] — composes with: permission badge + bell fires while in EmbeddedTerminal or SessionCreation
- [BC-2.09.001] — depends on: PTY output renders once EmbeddedTerminal is entered
- [BC-2.08.001] — depends on: SessionCreation wizard triggers spawn_session()

## Architecture Anchors

- `architecture/SS-embedded-pty.md#tui-appmode-extensions` — EmbeddedTerminal/SessionCreation variant definitions
- `architecture/SS-embedded-pty.md#session-creation-wizard` — wizard step sequence

## Story Anchor

S-TBD — Implement EmbeddedTerminal/SessionCreation AppMode transitions in monocle-tui and monocle-core (filled by story-writer)

## VP Anchors

VP-TBD — AppMode transition tests (filled after VP creation)

## §Trace v1.1.0

**I22-002 — Invariant 1: remove Killed (retired state), reconcile Detached (attachable, not no-op), add Terminating; PC-4: per-session scroll offset** (2026-06-13):
- I22-002 (Phase-1d Pass 22 IMPORTANT): Invariant 1 had three defects:
  (a) Listed `SessionState::Killed` as a no-op state. `Killed` was REMOVED from the reachable
      state set in SS-session-manager.md v1.3.0 (I4 audit) — it is superseded by `Terminating`.
      Listing it as a state that produces a no-op is incorrect; the variant does not compile.
  (b) Listed `Detached` as a no-op. This directly contradicted PC-2 (same file, line ~51)
      which states: "If the session-host is `Detached`, `attach_session()` is triggered
      automatically". Per SS-embedded-pty.md v1.5.0 lines ~101-102, ONLY `Terminated` is a
      no-op entry path; `Detached` is explicitly attachable (auto-attach). BC-2.08.007
      AttachSession PC-5 confirms Detached → Running via attach_session().
  (c) Made no mention of `Terminating` — a reachable state (SS-session-manager.md §Session
      lifecycle state machine) that IS a no-op for entry. Without an explicit rule, an
      implementer could attempt to enter EmbeddedTerminal for a Terminating session.
  Resolution: Invariant 1 now enumerates per-state semantics for all five reachable states
  (Launching, Running, Detached, Terminating, Terminated). `Killed` is explicitly named as
  RETIRED with cross-reference to SS-session-manager.md I4 audit.
- PC-4 (exit postconditions): `pty_scroll_offset` (singular, retired) → `pty_scroll_offsets[session_id]`
  per I7 fix (SS-embedded-pty.md §Parser ownership in TUI; BC-2.09.007 v1.1.0 Invariant 5).
- Version bump: 1.0.0 → 1.1.0 (minor: Invariant 1 semantically restructured with normative
  per-state rules; Detached handling is a behavioral correction not just a term fix).

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.008 authored for SS-09 as part of the v1A control-center pivot BC burst.
- Covers: EmbeddedTerminal enter/exit, SessionCreation 4-step wizard, auto-transition on Running.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
