---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-tui.md, architecture/SS-session-manager.md]
input-hash: "02d89c9"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
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

# Behavioral Contract BC-2.06.025: Multi-Session / Multi-Project Sessions Panel — Grouped by Project, Fast Switching, TUI Lifecycle Actions

## Description

The sessions panel renders all sessions (both monocle-launched and externally-detected)
grouped by `project_root`. Each group has a project header row followed by session rows.
The user can navigate sessions with arrow keys and switch focus with Enter. Session lifecycle
actions (create/kill/rename) are available via keybindings in the sessions panel. Fast session
switching (O(1) cost) switches the focused session's PTY parser for rendering in the preview
pane.

## Preconditions

1. `AppMode::Dashboard` (or `AppMode::Sessions` if sessions panel is fullscreen) is active.
2. The TUI has received `ServerToClient::InitialState` or `ServerToClient::SessionListUpdate`
   with session data from the daemon.

## Postconditions

1. The sessions panel renders a grouped list:
   - Sessions are sorted by `project_root` (alphabetical by project path).
   - Each unique `project_root` has a header row showing the project's basename
     (e.g., `monocle` for `/home/user/Dev/monocle`).
   - Under each header, session rows show: harness icon (`[M]` for monocle-launched,
     `[E]` for externally-detected), `display_name`, `SessionState` indicator
     (`Running`, `Launching`, `Detached`, `Terminated`).
2. Fast session switching:
   - Arrow keys navigate sessions within the list (including across project groups).
   - Enter on a `Running` session: AppMode transitions to `EmbeddedTerminal { session_id }`.
   - Enter on a `Detached` session: attach is triggered automatically; on `Running`,
     transitions to `EmbeddedTerminal`.
   - Enter on a `Launching` session: status bar shows "Session launching — please wait".
   - Switching focused session (arrow key navigation) changes which session's preview is
     shown in the preview pane (using the pre-maintained `pty_parsers` map — O(1) switch).
3. Session lifecycle keybindings in sessions panel:
   - `n`: open `AppMode::SessionCreation` wizard (new session).
   - `k` or `d`: kill/terminate the focused session (`KillSession` IPC sent).
   - `r`: rename focused session (inline edit or modal).
   - `D`: detach focused session (`DetachSession` IPC sent).
4. Monocle-launched sessions show a `[M]` badge. Externally-detected sessions show `[E]`.
   This reflects the `spawned_by_monocle: Some(true)` / `Some(false)` field on
   `EnrichedSession` per SS-engine-module-v2-delta.md.

## Invariants

1. Project grouping is presentational only — the underlying `sessions` list is flat; grouping
   is computed at render time from `project_root` values.
2. O(1) session switching: the focused session change (arrow key) only updates the
   `focused_session_id` field in the TUI's `App` state. The render cycle picks up
   `pty_parsers[focused_session_id].screen()` for the preview pane. No IPC round-trip.
3. The sessions panel list updates in real-time when `ServerToClient::SessionListUpdate`
   is received (adds/removes/updates rows without full panel re-render).
4. Sessions with `SessionState::Terminated` are shown in the list with a `[X]` indicator
   until GC removes them (BC-2.08.005 10-second grace period).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-290 | No sessions in list | Sessions panel shows "No sessions. Press 'n' to create one." |
| EC-291 | All sessions from same project_root | Single project group with all sessions as children |
| EC-292 | `project_root` is an empty string (edge case in session registry) | Grouped under `"<unknown>"` project header |
| EC-293 | `k` key on a `Launching` session | KillSession IPC sent; session transitions to Killed/Terminated; session-host handles Kill command |
| EC-294 | Rename with empty string | `RenameSession` with empty `new_name` → `ServerToClient::Error`; inline editor shows error indicator |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| 3 sessions; 2 same project, 1 different | 2 project headers; 3 session rows | happy-path |
| Arrow key navigation; Enter on Running session | AppMode → EmbeddedTerminal | happy-path |
| Press `n` | AppMode → SessionCreation (wizard) | happy-path |
| Press `k` on running session | KillSession IPC sent; session → Killed; list updates | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Project group headers rendered correctly for sessions with distinct project_roots | unit |
| VP-TBD | O(1) session switch: arrow key changes focused session; no IPC sent | unit |
| VP-TBD | `n` → SessionCreation; `k` → KillSession IPC; `D` → DetachSession IPC | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability traceability §SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability traceability — this BC extends the sessions panel capability in CAP-006 with multi-session, multi-project grouping, and lifecycle actions |
| Architecture Module | monocle-tui (sessions panel renderer, session list grouping logic, lifecycle keybindings) per ARCH-INDEX Subsystem Registry SS-06 |
| Architecture Source | SS-session-manager.md v1.0.1 §SessionManager §Public API (session_list()); SS-embedded-pty.md v1.0.2 §Fast switching; SS-engine-module-v2-delta.md v1.0.1 §ProcessSnapshot.spawned_by_monocle field |
| Cross-Ref | BC-2.05.010 (KillSession/DetachSession/RenameSession IPC variants triggered from sessions panel); BC-2.09.008 (SessionCreation wizard and EmbeddedTerminal enter) |
| Test Name | test_BC_2_06_025_multi_session_grouped_by_project |

## Related BCs

- [BC-2.06.005] — extends: existing sessions panel render from IPC state; this BC extends with grouping + lifecycle actions
- [BC-2.09.008] — composes with: Enter → EmbeddedTerminal or SessionCreation wizard entry
- [BC-2.05.010] — depends on: lifecycle IPC variants (Kill/Detach/Rename) sent from sessions panel

## Architecture Anchors

- `architecture/SS-session-manager.md#public-api` — session_list() returns SessionSnapshot vec

## Story Anchor

S-TBD — Implement multi-session grouped sessions panel with lifecycle actions (filled by story-writer)

## VP Anchors

VP-TBD — Sessions panel multi-session render tests (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.06.025 authored for SS-06 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): keybinding choices (n/k/r/D) selected per convention from
  lazygit-style single-char actions. These are TUI conventions that do not require human
  input — they are standard for this kind of panel. Conflict check against existing
  keybinding table (BC-2.06.003): `n`, `k`, `r`, `D` are not currently bound in Dashboard
  mode; allocation is safe.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
