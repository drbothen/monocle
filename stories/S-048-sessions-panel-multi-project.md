---
document_type: story
level: L4
story_id: S-048
epic_id: EPIC-06
version: "1.3"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
points: 8
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-022, S-025, S-028, S-033, S-047]
blocks: []
target_module: monocle-tui
subsystems: [SS-06]
behavioral_contracts: [BC-2.06.025]
verification_properties: []
estimated_days: 5
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.025.md, version: "1.5.4"}
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.8.2"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.24.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.6.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.06.025 — multi-session grouped sessions panel (SessionSnapshot wire type, project_root grouping, lifecycle action keys n/k/d/r/D, state-aware blocking, [M]/[E]/[?]/[!] badges)"
# BC status: BC-2.06.025 non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-048: Sessions Panel — Multi-Project Grouping, Lifecycle Actions, and State-Aware Blocking

## Narrative

As a monocle TUI user, I want the sessions panel to show all running sessions grouped by
`project_root` (alphabetically), with `[M]`/`[E]`/`[?]` provenance badges and `[!]` degraded
indicators, and to let me perform lifecycle actions with keyboard shortcuts (`n`=spawn, `k`/`d`=kill,
`r`=rename, `D`=detach) that are automatically blocked for sessions in incompatible
states — so that I can manage multiple projects from a single TUI view without needing to track
session state manually.

## Acceptance Criteria

### AC-001 (traces to BC-2.06.025 postcondition 1 — SessionSnapshot wire type, not EnrichedSession)

The sessions panel binds to `Vec<SessionSnapshot>` from `ServerToClient::SessionListUpdate`.
It NEVER accesses `EnrichedSession` or any daemon-internal session type directly. The TUI
receives only the fields available on the canonical `SessionSnapshot` struct
(defined in `crates/monocle-ipc/src/lib.rs`, authority: SS-ipc.md §Supporting Types):
`session_id: String`, `display_name: String`, `state: SessionState`, `harness_id: String`,
`project_root: String`, `cwd: String`, `spawned_by_monocle: Option<bool>`,
`started_at_micros: i64`, `pty_rows: u16`, `pty_cols: u16`,
`degraded: bool`, `degraded_reason: Option<String>`.
Note: the field is `degraded` (NOT `is_degraded`); there is no `worktree_root` field
(the canonical struct has `cwd` for the effective working directory); there is no `pid` field.
These canonical names MUST be used in TUI code; do NOT invent `is_degraded` or `worktree_root`.

### AC-002 (traces to BC-2.06.025 postcondition 1 — grouped by project_root, sorted alphabetically)

Sessions are visually grouped under project_root headers in the panel. Within each group,
sessions are sorted by `display_name` (ascending). Groups themselves are sorted by
`project_root` path (ascending, byte-order). An empty project_root is treated as
`"(no project)"` and sorts last.

### AC-003 (traces to BC-2.06.025 postcondition 4 — [M]/[E]/[?] provenance badges)

Each session row displays a provenance badge:
- `[M]` when `spawned_by_monocle == Some(true)` (monocle-spawned).
- `[E]` when `spawned_by_monocle == Some(false)` (externally-detected).
- `[?]` when `spawned_by_monocle == None` (origin unknown).
- `[!]` when `degraded == true` (appended after the provenance badge, e.g., `[M][!]`).
  Field name is `degraded` (canonical per SS-ipc.md §SessionSnapshot); NOT `is_degraded`.
The badge is part of the rendered row text, not a separate column.

### AC-004 (traces to BC-2.06.025 postcondition 3 — lifecycle action key bindings)

The sessions panel handles the following keys when a session row is focused:
- `n` — spawn a new session (opens spawn dialog or sends `SpawnSession` with defaults).
- `k` or `d` — kill/terminate the focused session (both send `KillSession { session_id }`; `d` is the kill alias per BC-2.06.025 PC-3).
- `r` — rename the focused session (opens inline rename input; sends `RenameSession` on confirm).
- `D` (shift-d) — detach the focused session (sends `DetachSession { session_id }`).
Each key is shown in the status bar hint line when the sessions panel is active.

### AC-005 (traces to BC-2.06.025 invariant 5 — Launching state: kill=ALLOWED, detach=BLOCKED, rename=ALLOWED)

When the focused session is in `Launching` state:
- `k` or `d` (kill): ALLOWED — both send `KillSession { session_id: String }`. TUI transitions display to `Terminating`.
  (Daemon enforces the actual state; TUI is optimistic.) `d` is the kill alias — it dispatches `KillSession`, not `DetachSession`.
- `r` (rename): ALLOWED — sends `RenameSession` as normal (metadata operation; does not require active host_conn).
- `D` (detach): BLOCKED — EC-298 (BC-2.06.025) prohibits `D`/DetachSession on Launching; key press is a no-op; status bar shows
  `"Session launching — please wait"` for 3 seconds. A Launching session has no live host_conn to detach from.

### AC-006 (traces to BC-2.06.025 invariant 4 — Terminating=blanket block for all lifecycle actions)

When the focused session is in `Terminating` state:
- ALL lifecycle actions (`k`, `d`, `r`, `D`) are BLOCKED.
- Key press for any blocked action shows `"Session is terminating — action unavailable"` for
  3 seconds in the status bar.
- `n` (spawn new session) is NOT blocked (it is not scoped to the focused session).

### AC-007 (traces to BC-2.06.025 invariant 6 — Terminated-in-grace=blanket block, F-P52-001)

When the focused session is in `Terminated` state (within the grace retention window):
- ALL lifecycle actions (`k`/`d`, `r`, `D`) are BLOCKED.
- Status bar: `"Session has terminated"` for 3 seconds.
- The session row REMAINS VISIBLE during the grace retention window; it is NOT automatically
  removed from the list. Removal happens when the grace window expires or the daemon
  removes it from the next `SessionListUpdate`. There is no explicit user-triggered removal action.

### AC-008 (traces to BC-2.06.025 postcondition 3 Launching-state keybinding rules + invariant 5 — k on Launching sends KillSession, transitions to Terminating, EC-293)

When `k` is pressed on a `Launching` session (EC-293 from BC-2.06.025):
- `ClientToServer::KillSession { session_id }` is sent.
- The TUI optimistically renders the session as `Terminating` immediately (before the next
  `SessionListUpdate` arrives). If the daemon returns `Error { code: "session_not_found" }`,
  the TUI resets to the last known `SessionListUpdate` state.

### AC-009 (traces to BC-2.06.025 invariant 4 — Terminating blanket block, EC-296: k on Terminating is no-op)

When `k` is pressed on a `Terminating` session (EC-296):
- No IPC message is sent.
- Status bar shows `"Session is terminating — action unavailable"` for 3 seconds.
- The session row state is unchanged.

### AC-010 (traces to BC-2.06.025 invariant 5 — Launching-state Detach BLOCKED, EC-298: D on Launching is no-op)

When `D` (detach) is pressed on a `Launching` session (EC-298 from BC-2.06.025):
- No IPC message is sent (`ClientToServer::DetachSession` is NOT dispatched).
- Status bar shows `"Session launching — please wait"` for 3 seconds.
- `D` (detach) is BLOCKED on `Launching` per BC-2.06.025 EC-298 and Invariant 5. A Launching
  session has no established host_conn to detach from; dispatching `DetachSession` would cause the
  daemon to return `session_not_ready`. Kill (`k`/`d`) and rename (`r`) remain ALLOWED during Launching.

### AC-011 (traces to BC-2.06.025 invariant 6 — Terminated-in-grace blanket block F-P52-001, EC-300/EC-301/EC-302: all actions blocked on Terminated)

When any of `k`, `d`, `r`, `D` is pressed on a `Terminated` session (EC-300, EC-301, EC-302):
- No IPC message is sent.
- Status bar: `"Session has terminated"` for 3 seconds.
- The session row is not visually changed.

### AC-012 (traces to BC-2.06.025 postcondition 1 + precondition 2 — panel receives SessionSnapshot wire type, not EnrichedSession)

A compile-time assertion or type annotation ensures the sessions panel component accepts
`&[SessionSnapshot]` (from `monocle_ipc::SessionSnapshot` via `crates/monocle-ipc/src/lib.rs`),
never `&[monocle_runtime::session::EnrichedSession]`. This constraint is enforced by the function
signature of the panel's `render()` method.

### AC-013 (traces to BC-2.06.025 postcondition 1 — state indicator renders ALL 5 canonical states)

The sessions panel state indicator column renders a visual label for ALL 5 canonical `SessionState`
variants without panicking or falling through to an unhandled match arm:
- `Launching` → indicator label `"Launching"` (or equivalent styled text).
- `Running` → indicator label `"Running"`.
- `Detached` → indicator label `"Detached"`.
- `Terminating` → indicator label `"Terminating"`.
- `Terminated` → indicator label `"Terminated"`.

The match over `SessionState` in the render path MUST be exhaustive against all 5 variants.
`Created` and `Killed` are retired variants and MUST NOT appear in the match.
Test: `test_BC_2_06_025_state_indicator_renders_all_5_states` — constructs a `SessionSnapshot`
for each of the 5 states and asserts the rendered row contains the expected indicator text.

### AC-014 (traces to BC-2.06.025 postcondition 2 — Enter on Detached session sends AttachSession; transitions to EmbeddedTerminal)

When the focused session is in `Detached` state and the user presses `Enter`:
- `ClientToServer::AttachSession { session_id }` is sent immediately (using `snapshot.session_id.clone()` — no newtype wrapping).
- The TUI transitions to `AppMode::EmbeddedTerminal` upon receiving
  `ServerToClient::SessionStateChanged { session_id, new_state: SessionState::Running }` for the
  same session (matching the optimistic/await pattern established in S-044 AC-011).
  Alternatively, the TUI may optimistically transition to `AppMode::EmbeddedTerminal` immediately
  after sending `AttachSession` if the optimistic pattern is used consistently in the codebase —
  implementer resolves against the pattern chosen by S-044.
- `Enter` on a `Detached` session MUST NOT be treated as a no-op. It is the re-attach trigger.
- `Enter` on a `Running` session transitions to `AppMode::EmbeddedTerminal` directly (no IPC
  `AttachSession` needed — the session is already attached). This is the existing S-025 AC-007/AC-008
  behavior; this AC does NOT change it.
Test: `test_BC_2_06_025_enter_on_detached_sends_attach_session` — creates a `SessionSnapshot` with
`state: Detached`, simulates `Enter`, asserts `ClientToServer::AttachSession { session_id }` sent
and that the TUI mode transitions to `EmbeddedTerminal` (or that the transition is triggered on
`SessionStateChanged{Running}`).

## Tasks

### Data Model
- [ ] Ensure `SessionSnapshot` is defined in `crates/monocle-ipc/src/lib.rs` with the canonical
      fields per SS-ipc.md §Supporting Types (authority: SS-ipc.md, not BC-2.06.025
      which may use prose-level field names):
      `session_id: String`, `display_name: String`, `state: SessionState`, `harness_id: String`,
      `project_root: String`, `cwd: String`, `spawned_by_monocle: Option<bool>`,
      `started_at_micros: i64`, `pty_rows: u16`, `pty_cols: u16`,
      `degraded: bool` (NOT `is_degraded`), `degraded_reason: Option<String>`.
      There is NO `worktree_root` field (use `cwd` for effective working dir) and NO `pid` field.
      All wire IDs (`session_id`) are `String` (UUID-as-String) per SS-ipc.md §Wire IDs.
      Verify against the struct definition — do NOT add extra fields not in the canonical struct.
- [ ] Ensure `SessionState` enum is defined in `crates/monocle-ipc/src/lib.rs` with the canonical 5 variants:
      `Launching`, `Running`, `Detached`, `Terminating`, `Terminated`.
      (`Created` and `Killed` are retired variants — do NOT use them.)
      S-033 owns the definition of `SessionState` in `crates/monocle-ipc/src/lib.rs` (wire-type location, per F-P16-IMP-001).
      Verify against S-033; add if not present; do not duplicate. `monocle-ipc` MUST NOT depend on `monocle-runtime`.

### TUI Panel (monocle-tui)
- [ ] Create `monocle-tui/src/panels/sessions_panel.rs`:
      - `SessionsPanel` struct with `sessions: Vec<SessionSnapshot>`, `focused_idx: usize`. <!-- structural-claim-historical: SS-tui.md §App struct carries Vec<EnrichedSession> (daemon-internal type from Phase-1 baseline); this story migrates SessionsPanel to Vec<SessionSnapshot> (wire type from monocle-ipc) as the v1A multi-session redesign intent -->
        `SessionSnapshot` is imported from `monocle_ipc` — NEVER from `monocle_runtime`.
      - `group_by_project_root()` method: returns `Vec<(String, Vec<&SessionSnapshot>)>` sorted
        alphabetically per AC-002. Note: `SessionSnapshot.project_root` is `String` (wire type),
        NOT `PathBuf`; group key is `String`.
      - `render(&self, frame: &mut Frame, area: Rect)` using ratatui `0.30` widgets.
      - Project root headers rendered as `Block` borders or styled list items.
      - Badge rendering per AC-003 (`[M]`, `[E]`, `[?]`, `[!]`).
- [ ] Implement `SessionsPanel::handle_key(key: KeyEvent, sender: &mpsc::Sender<ClientToServer>)`
      with the state-aware dispatch table per AC-004..AC-011, AC-014. Includes:
      - `Enter` on `Detached` state → send `ClientToServer::AttachSession { session_id }` + transition to `AppMode::EmbeddedTerminal` (AC-014).
      - `Enter` on `Running` state → transition to `AppMode::EmbeddedTerminal` directly (existing S-025 behavior; no IPC sent for already-attached session).
      All IPC messages carry `session_id: String` (from `SessionSnapshot.session_id`) — never a
      newtype `SessionId`; wire fields are plain `String` per SS-ipc.md §Wire IDs.
- [ ] Add status bar message queue integration: 3-second and 5-second timed messages per AC-005/006/007.
- [ ] Wire `SessionsPanel` into the main TUI layout (replace or extend the existing session list
      from S-025/S-028 — confirm which component it extends).

### Tests
- [ ] Write unit tests in `monocle-tui/tests/sessions_panel.rs`:
      - `test_BC_2_06_025_grouped_by_project_root_alphabetical` (AC-002)
      - `test_BC_2_06_025_badges_monocle_external_unknown_degraded` (AC-003)
      - `test_BC_2_06_025_launching_kill_allowed_detach_blocked` (AC-005)
      - `test_BC_2_06_025_terminating_all_actions_blocked` (AC-006)
      - `test_BC_2_06_025_terminated_all_actions_blocked` (AC-007)
      - `test_BC_2_06_025_ec293_k_on_launching_sends_kill_optimistic_render` (AC-008)
      - `test_BC_2_06_025_ec296_k_on_terminating_no_ipc_sent` (AC-009)
      - `test_BC_2_06_025_ec298_d_on_launching_no_op_blocked` (AC-010)
      - `test_BC_2_06_025_ec300_302_all_actions_blocked_terminated` (AC-011)
      - `test_BC_2_06_025_sessions_snapshot_type_not_enriched_session` (AC-012 — type assertion)
      - `test_BC_2_06_025_state_indicator_renders_all_5_states` (AC-013 — exhaustive match over Launching/Running/Detached/Terminating/Terminated)
      - `test_BC_2_06_025_enter_on_detached_sends_attach_session` (AC-014 — Enter on Detached sends AttachSession + transitions to EmbeddedTerminal)

## Previous Story Intelligence

S-025 established the TUI skeleton and layout system. S-028 added the nucleo filter, event
ribbon, and initial session list display (using `EnrichedSession` at that phase). S-022 established
the IPC receive path that delivers `SessionListUpdate`. S-033 established `SpawnSession` in the
daemon.

This story REPLACES or EXTENDS the session display component from S-028 with the full
multi-project sessions panel using `SessionSnapshot` wire type. Before implementing, confirm
whether S-028's session list component is a separate widget or inline in the main layout —
if separate, replace it; if inline, extract it to `sessions_panel.rs` first.

Key lesson: S-028 used `EnrichedSession` — this story explicitly migrates to `SessionSnapshot`
wire type (AC-001, AC-012). The compile-time type enforcement (AC-012) prevents regression.

## Architecture Compliance Rules

From `architecture/SS-tui.md v1.8.2`:
- ratatui `0.30` — all widget construction uses the builder pattern from this exact version.
  No `Table::new()` with positional column widths from ratatui 0.29 API.
- `VecDeque<PromptModal>` for permission overlays (NOT `Option<PromptModal>`) — sessions panel
  must NOT touch the overlay queue; it dispatches lifecycle IPC only.
- Status bar message queue: timerbound messages use `Instant` + duration, checked on each
  frame render. Do NOT use `tokio::time::sleep` for status bar expiry in the render path.

From `architecture/SS-ipc.md` (§Wire IDs):
- `SessionSnapshot` is the IPC wire type for sessions in `ServerToClient::SessionListUpdate`.
  The TUI receives `Vec<SessionSnapshot>` and must not call into daemon internals.
- All `session_id` fields in wire messages are `String` (UUID-as-String). `SessionId`/`ClientId`
  newtypes are daemon-internal ONLY and MUST NOT appear in IPC message structs or TUI code.
  Building an IPC message from a focused `SessionSnapshot`: use `snapshot.session_id.clone()`
  directly — no `.into()` or newtype wrapping.

From `architecture/SS-conventions-anti-patterns.md v1.32.6`:
- No `println!` or `eprintln!` in TUI render paths.
- `Arc<RwLock<>>` for shared theme/config state (if sessions panel needs config access).

**Forbidden dependencies**: `monocle-tui` MUST NOT import from `monocle-runtime`. All types
accessed via `monocle_ipc` public API (from `crates/monocle-ipc/src/lib.rs`). If monocle-tui gains a dependency on
`monocle-runtime`, the build MUST fail.

## Library and Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| `ratatui` | `^0.30` | Sessions panel widget rendering |
| `tokio` | `=1.52.0` (EXACT) | `mpsc::Sender<ClientToServer>` for key dispatch |
| `serde` | `^1` (features = ["derive"]) | `SessionSnapshot` deserialization |
| `tracing` | `^0.1` | Debug/warn logging in key handler |

No new dependencies added. All are existing workspace deps.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/monocle-tui/src/panels/sessions_panel.rs` | CREATE | Main sessions panel widget + key handler |
| `crates/monocle-tui/src/panels/mod.rs` | MODIFY (or CREATE) | `pub mod sessions_panel;` |
| `crates/monocle-tui/src/main_layout.rs` (or equivalent) | MODIFY | Wire SessionsPanel into TUI layout, replacing/extending S-028 session list |
| `crates/monocle-ipc/src/lib.rs` | MODIFY (if needed) | Add `SessionSnapshot` struct and `SessionState` enum if not present from S-033/S-034 |
| `crates/monocle-tui/tests/sessions_panel.rs` | CREATE | Unit tests for all ACs |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-400 | Zero sessions in `SessionListUpdate` | Panel renders empty state: "No active sessions" message; `n` key still works (spawn new) |
| EC-401 | All sessions have the same `project_root` | Single group header; all sessions listed under it; sorted by `display_name` |
| EC-402 | `project_root` is empty string | Renders as `"(no project)"` header, sorted last |
| EC-403 | Session `degraded=true` (NOT `is_degraded`) AND `spawned_by_monocle=None` | Badge renders as `[?][!]` |
| EC-404 | Rapid consecutive `SessionListUpdate` messages (>10/sec during spawn storm) | Panel re-renders on each update using ratatui's lazy diffing; no panic, no partial state |
| EC-405 | `focused_idx` points beyond the end of a group after a session is removed | Clamp `focused_idx` to `(num_sessions - 1).max(0)` on next render |
| EC-406 | `r` (rename) on a `Running` session | Opens inline rename input; Enter confirms and sends `RenameSession`; Esc cancels (no IPC sent) |
| EC-407 | Multiple sessions with identical `display_name` in same group | Both shown; sort is stable (by `session_id` as tiebreaker) |
| EC-408 | User presses `k` and daemon returns `Error { code: "session_not_found" }` | TUI resets display to last known `SessionListUpdate` state; shows error in status bar for 3 sec |

## Token Budget Estimate

| Category | Estimate |
|----------|----------|
| Story spec (this file) | ~6 500 tokens |
| BC files (1 BC: BC-2.06.025) | ~5 000 tokens |
| Architecture sections (SS-tui, SS-ipc, SS-session-manager) | ~3 500 tokens |
| Existing code context (S-028 session list component, monocle-ipc/src/lib.rs, main_layout.rs) | ~5 000 tokens |
| Test file to write | ~3 500 tokens |
| **Total estimated** | **~23 500 tokens** |

Within the 20–30% context window constraint. No splitting needed.

## Dependency Justification

- S-048 depends on S-022 because IPC session list delivery (`ServerToClient::SessionListUpdate`)
  was established in S-022's `InitialState` push path; the sessions panel subscribes to that data.
- S-048 depends on S-025 because the TUI layout system and ratatui widget infrastructure were
  established in S-025; the sessions panel is a new panel within that layout.
- S-048 depends on S-028 because S-028 created the initial session list display using the
  nucleo filter and event ribbon; S-048 extends that with multi-project grouping and lifecycle
  actions — the component base exists.
- S-048 depends on S-033 because `SpawnSession` dispatch (pressing `n` in the sessions panel)
  calls into the daemon's `spawn_session()` established in S-033.
- S-048 depends on S-047 because the lifecycle action keys dispatch IPC variants defined and routed
  in S-047: `k`/`d` (kill alias) → `KillSession`; `D` → `DetachSession`; `r` → `RenameSession`.
- S-048 blocks nothing (it is the final story in Wave 8 for SS-06 multi-project scope).

## Subsystem Anchor Justification

SS-06 owns this story's scope because the sessions panel is a TUI-plane component that manages
session lifecycle visibility and user interaction — the domain of SS-06 (monocle-tui, session
display and interaction plane) per ARCH-INDEX Subsystem Registry SS-06.

## Trace

| Pass | Date | Change |
|------|------|--------|
| v1.3 | 2026-06-16 | F-P19-IMP-001: Corrected fabricated postcondition numbers in AC-002..AC-004 and AC-008..AC-012 trace headers. BC-2.06.025 has exactly 4 postconditions (PC-1: grouped list+wire-type+badges; PC-2: fast switching+Enter; PC-3: lifecycle keybindings; PC-4: [M]/[E]/[?] badges). Fixed AC-002 PC-2→PC-1 (grouping is PC-1); AC-003 PC-3→PC-4 ([M]/[E]/[?] badges); AC-004 PC-4→PC-3 (lifecycle keybindings). Fixed fabricated PC-5/6/7/8/9: AC-008 → PC-3 Launching-state rules + Invariant 5 (kill ALLOWED); AC-009 → Invariant 4 (Terminating blanket block); AC-010 → Invariant 5 (Detach BLOCKED on Launching); AC-011 → Invariant 6 (Terminated-in-grace blanket block, F-P52-001); AC-012 → PC-1 + Precondition 2 (SessionSnapshot wire type). EC references (EC-293/296/298/300-302) preserved — they were correct. AC bodies unchanged. |
| v1.2 | 2026-06-16 | F-P18-CRIT-001: Corrected inverted key→action mapping throughout. (1) Narrative: `d`=detach/`D`=destroy replaced with `k`/`d`=kill/`D`=detach (no destroy action). (2) AC-004: `d`→DetachSession removed; `k` or `d` → KillSession (d is kill alias per BC-2.06.025 PC-3); `D` → DetachSession. (3) AC-005: corrected Launching-state rules — `k`/`d` (kill) ALLOWED, `r` (rename) ALLOWED, `D` (detach) BLOCKED per EC-298/Invariant 5; removed fabricated "Cannot destroy:" status message; status bar now shows "Session launching — please wait" for `D`; BC trace updated from invariant 3 to invariant 5. (4) AC-007: BC trace updated from invariant 5 to invariant 6 (Terminated-in-grace); removed fabricated "destroy" removal-action prose. (5) AC-010: EC-298 correctly applied to `D`=detach being blocked on Launching (not "destroy"); status bar message corrected to "Session launching — please wait". (6) Dependency justification: wire command mapping clarified to `k`/`d`→KillSession, `D`→DetachSession, `r`→RenameSession. |
| v1.1 | 2026-06-16 | F-P16-IMP-002: (1) Corrected `SessionState` Task enumeration from 4 variants to the canonical 5 (added `Detached`; noted `Created`/`Killed` are retired); noted S-033 owns the definition in monocle-ipc per F-P16-IMP-001. (2) Added AC-013 (BC-2.06.025 PC-1): state indicator renders all 5 canonical states exhaustively including `Detached`. (3) Added AC-014 (BC-2.06.025 PC-2): Enter on `Detached` session sends `ClientToServer::AttachSession`; TUI transitions to `AppMode::EmbeddedTerminal` on `SessionStateChanged{Running}`. (4) Updated `handle_key` task to reference AC-014. (5) Added tests for AC-013 and AC-014 to the test task list. |
