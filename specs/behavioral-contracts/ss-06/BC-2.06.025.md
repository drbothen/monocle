---
document_type: behavioral-contract
level: L3
version: "1.5.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-14T01:00:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-tui.md, architecture/SS-session-manager.md]
input-hash: "bb2f26d"
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
   with session data from the daemon as `Vec<SessionSnapshot>` (SS-ipc.md v1.24.0 — the wire
   boundary type is `SessionSnapshot`, not `EnrichedSession`; `EnrichedSession` is internal
   to `EngineModule::detect()` and never crosses the UDS wire).

## Postconditions

1. The sessions panel renders a grouped list from `Vec<SessionSnapshot>` received via
   `InitialState.sessions` or `SessionListUpdate.sessions` (SS-ipc.md v1.24.0 `SessionSnapshot`
   wire type — NOT `EnrichedSession`; rendering reads `SessionSnapshot` fields directly):
   - Sessions are sorted by `SessionSnapshot.project_root` (alphabetical by project path).
   - Each unique `project_root` has a header row showing the project's basename
     (e.g., `monocle` for `/home/user/Dev/monocle`).
   - Under each header, session rows show: harness icon (`[M]` for monocle-launched
     `spawned_by_monocle: Some(true)`, `[E]` for externally-detected `Some(false)`,
     `[?]` for pre-v1A forward-compat sessions where `spawned_by_monocle: None` — the
     `None` case occurs for sessions discovered from sidecars written before this field
     existed; `[?]` indicates "origin unknown, treat as external"), `display_name`,
     `SessionState` indicator (`Running`, `Launching`, `Detached`, `Terminating`, `Terminated`).
   - Sessions with `SessionState::Terminating` render a `[Terminating]` indicator (e.g.,
     a spinner or dimmed name). Lifecycle actions (`k`/`d`, `D`, `r`) are DISABLED for
     `Terminating` sessions — the kill is already in progress. (`d` is the kill alias
     per PC-3; both `k` and `d` trigger the same `KillSession` IPC and are equally disabled.)
   - Sessions with `SessionSnapshot.degraded == true` render a `[!]` degraded badge (amber
     or warning color) alongside the session row. The `SessionSnapshot.degraded_reason`
     (e.g., `"Missing env: HOME, PATH"`) is displayed in the status bar when the degraded
     session is focused. A degraded session is otherwise functional — lifecycle actions
     remain enabled. (I3-009 fix: degraded-env surfaced to TUI via SessionSnapshot.)
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

   **Launching-state keybinding rules (F-P51-001 / action-specific, NOT a blanket disable):**
   Pressing a lifecycle key while the focused session has `SessionState::Launching` is governed
   by these rules. Launching is a transient state (post-spawn monitor connects within ~ms); the
   rules are action-specific because the feasibility of each operation differs:

   - **Kill (`k`/`d`) on Launching — ALLOWED:** `KillSession` IPC is sent normally; the daemon
     handles the Launching kill path (uses `host_conn.writer` if post-spawn monitor has connected,
     or PID-based SIGTERM/SIGKILL fallback if not yet connected). Transition:
     `Launching → Terminating`. Sessions panel updates on receipt of `SessionStateChanged{Terminating}`.
     (Rationale: kill is always valid; the daemon has defined kill semantics for every session state.
     See BC-2.08.003 Precondition 1 and Invariant 3: kill on Launching is explicitly allowed.)

   - **Detach (`D`) on Launching — BLOCKED at TUI (no-op + status hint):** Pressing `D` on a
     `Launching` session is a no-op. No `ClientToServer::DetachSession` message is dispatched.
     The status bar shows "Session launching — please wait". A Launching session has no live
     session-host connection to detach from (the post-spawn monitor is in the process of
     connecting); dispatching `DetachSession` would cause the daemon to return
     `SessionError::SessionNotReady` (BC-2.05.010 §DetachSession PC-4 / BC-2.08.007 Precondition
     1). The TUI guard here ensures `session_not_ready` is a defensive/untrusted-client-only path,
     not a normal user-facing error. (Cross-reference: BC-2.05.010 §DetachSession PC-4;
     BC-2.08.007 §Preconditions (detach) defensive note.)

   - **Rename (`r`) on Launching — ALLOWED:** `RenameSession` IPC is sent normally. Rename is
     a metadata operation (`display_name` update); it does not require an active session-host
     connection (`host_conn`) and the daemon's `rename_session()` path succeeds on any non-
     Terminated session. The session transitions through Launching → Running normally; the rename
     takes effect immediately in the session registry and is reflected in the next
     `SessionListUpdate` broadcast.
4. Monocle-launched sessions show a `[M]` badge. Externally-detected sessions show `[E]`.
   Sessions with `spawned_by_monocle: None` (pre-v1A forward-compat or legacy sidecars) show `[?]`.
   This tri-state reflects the `SessionSnapshot.spawned_by_monocle: Option<bool>` field
   (SS-ipc.md v1.24.0 `SessionSnapshot` type): `Some(true)` → `[M]`, `Some(false)` → `[E]`,
   `None` → `[?]`. Sessions with `SessionSnapshot.degraded == true` additionally show a `[!]`
   badge (PC-1 degraded badge rule).

## Invariants

1. Project grouping is presentational only — the underlying `sessions` list is flat; grouping
   is computed at render time from `project_root` values.
2. O(1) session switching: the focused session change (arrow key) only updates the
   `focused_session_id` field in the TUI's `App` state. The render cycle picks up
   `pty_parsers[focused_session_id].screen()` for the preview pane. No IPC round-trip.
3. The sessions panel list updates in real-time when `ServerToClient::SessionListUpdate`
   is received (adds/removes/updates rows without full panel re-render).
   (For lifecycle keybinding rules during specific session states — in particular that `D`
   (DetachSession) is BLOCKED and MUST NOT be dispatched while a session is in
   `SessionState::Launching` — see Invariant 5. EC-298 documents this edge case.)
4. Sessions with `SessionState::Terminated` are shown in the list with a `[X]` indicator
   until GC removes them (BC-2.08.005 10-second grace period).
   Sessions with `SessionState::Terminating` are shown with a `[Terminating]` indicator (a
   spinner or dimmed style) and MUST NOT allow lifecycle actions (`k`/`d`, `D`, `r` are no-ops
   or explicitly blocked with a status bar hint). The `[Terminating]` state persists until
   the daemon broadcasts `SessionListUpdate` with `state: Terminated` (on session-host
   confirmation or 12s watchdog). See BC-2.08.003 Invariant 4 for the Terminating state
   definition and transition rules.

5. **Launching-state lifecycle action invariant (F-P51-001):** For sessions with
   `SessionState::Launching`, lifecycle actions are governed individually — this is NOT a blanket
   disable like the Terminating rule in Invariant 4:
   - **Kill (`k`/`d`)**: ALLOWED. `KillSession` IPC MUST be dispatched (daemon handles Launching
     kill path per BC-2.08.003 Invariant 3). The TUI MUST NOT suppress kill on Launching sessions.
   - **Detach (`D`)**: BLOCKED. The TUI MUST NOT dispatch `ClientToServer::DetachSession` for a
     Launching session. `D` is a no-op; status bar shows "Session launching — please wait". This
     ensures `SessionError::SessionNotReady` / `ServerToClient::Error { code: "session_not_ready" }`
     is a defensive/untrusted-client path only — never reachable from the official TUI.
     (Backing guarantee for BC-2.05.010 §DetachSession PC-4 and BC-2.08.007 §Preconditions (detach)
     defensive note.)
   - **Rename (`r`)**: ALLOWED. `RenameSession` IPC MUST be dispatched (rename is metadata-only;
     does not require `host_conn`; succeeds on any non-Terminated session).
   This invariant is the normative target of all cites of "BC-2.06.025 guards" in BC-2.05.010
   §DetachSession PC-4 and BC-2.08.007 §Preconditions (detach) defensive note.

6. **Terminated-in-grace lifecycle action invariant (F-P52-001):** Sessions with
   `SessionState::Terminated` remain in the sessions panel with a `[X]` indicator for up to
   10 seconds (BC-2.08.005 GC grace period). During this window, lifecycle actions MUST NOT
   be dispatched — the session is a corpse in the GC grace period:
   - **Kill (`k`/`d`) on Terminated**: BLOCKED at TUI. No `ClientToServer::KillSession` IPC
     dispatched. The kill is already complete (BC-2.08.003 Invariant 2 — kill on Terminated
     is idempotent at the daemon, but the TUI MUST NOT dispatch). Status bar shows
     "Session has terminated".
   - **Detach (`D`) on Terminated**: BLOCKED at TUI. No `ClientToServer::DetachSession` IPC
     dispatched. A Terminated session has no live session-host connection to detach from.
     Status bar shows "Session has terminated". (Daemon-side: detach_session() on Terminated
     returns idempotent Ok(()) per SS-session-manager.md v2.6.1 §Terminated-in-grace defensive
     action×state matrix — TUI guard makes this path unreachable from official TUI.)
   - **Rename (`r`) on Terminated**: BLOCKED at TUI. No `ClientToServer::RenameSession` IPC
     dispatched. If dispatched (e.g., from an untrusted client), the daemon returns
     `Err(SessionError::InvalidSessionName { reason: "session terminated" })` → wire code
     `"rename_failed"` per SS-session-manager.md v2.6.1 §Terminated-in-grace defensive
     action×state matrix (F-P52-001) and BC-2.08.005 Invariant 4. Status bar shows
     "Session has terminated".
   This is a blanket block (mirrors the Terminating guard in Invariant 4). All three actions
   (`k`/`d`, `D`, `r`) are no-ops + status bar hint for Terminated-in-grace sessions.
   Cross-references: BC-2.08.005 Invariant 4 (revive-via-rename not allowed; GC task not
   cancellable); SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state
   matrix (F-P52-001).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-290 | No sessions in list | Sessions panel shows "No sessions. Press 'n' to create one." |
| EC-291 | All sessions from same project_root | Single project group with all sessions as children |
| EC-292 | `project_root` is an empty string (edge case in session registry) | Grouped under `"<unknown>"` project header |
| EC-293 | `k` key on a `Launching` session | KillSession IPC sent; daemon transitions `Launching → Terminating`; `[Terminating]` renders; eventually `→ Terminated` |
| EC-296 | `k` key on a `Terminating` session | No-op (idempotent); kill already in progress; status bar shows "Session is already terminating…"; no duplicate KillSession IPC sent |
| EC-294 | Rename with empty string | `RenameSession` with empty `new_name` → `ServerToClient::Error`; inline editor shows error indicator |
| EC-295 | `spawned_by_monocle: None` (pre-v1A forward-compat session — sidecar has no `spawned_by_monocle` field) | Session row renders `[?]` badge; treated as "external" for lifecycle purposes (Kill/Detach/Rename all available); NOT treated as monocle-launched for hook injection purposes |
| EC-297 | `SessionSnapshot.degraded == true` received in InitialState or SessionListUpdate | Session row renders `[!]` badge (amber/warning style) alongside normal state indicator. When user navigates to the session, status bar shows the `degraded_reason` (e.g., "Missing env: HOME, PATH"). Lifecycle actions remain enabled — the session is running in a degraded environment but is not terminated. |
| EC-298 | `D` key on a `Launching` session | No-op — `ClientToServer::DetachSession` is NOT dispatched. Status bar shows "Session launching — please wait". Launching session has no established `host_conn` to detach from; dispatching `DetachSession` would yield `session_not_ready` from the daemon (BC-2.05.010 §DetachSession PC-4; BC-2.08.007 Precondition 1 defensive note). Per Invariant 5. |
| EC-299 | `r` key on a `Launching` session | Rename is ALLOWED — `RenameSession` IPC dispatched normally. Rename is metadata-only (display_name); does not require an active host_conn. Per Invariant 5. |
| EC-300 | `k` or `d` key on a `Terminated` (GC-grace) session | No-op — `ClientToServer::KillSession` is NOT dispatched. Kill is already complete. Status bar shows "Session has terminated". Per Invariant 6. |
| EC-301 | `D` key on a `Terminated` (GC-grace) session | No-op — `ClientToServer::DetachSession` is NOT dispatched. Terminated session has no live host connection. Status bar shows "Session has terminated". Per Invariant 6. |
| EC-302 | `r` key on a `Terminated` (GC-grace) session | No-op — `ClientToServer::RenameSession` is NOT dispatched. The daemon would return `Err(InvalidSessionName{"session terminated"})` → `"rename_failed"` if dispatched (BC-2.08.005 Invariant 4; SS-session-manager.md v2.6.1 §Terminated-in-grace matrix). Status bar shows "Session has terminated". Per Invariant 6. |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| 3 sessions; 2 same project, 1 different | 2 project headers; 3 session rows | happy-path |
| Arrow key navigation; Enter on Running session | AppMode → EmbeddedTerminal | happy-path |
| Press `n` | AppMode → SessionCreation (wizard) | happy-path |
| Press `k` on running session | KillSession IPC sent; session → Terminated (on confirmation); list updates | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Project group headers rendered correctly for sessions with distinct project_roots | unit |
| VP-TBD | O(1) session switch: arrow key changes focused session; no IPC sent | unit |
| VP-TBD | `n` → SessionCreation; `k` → KillSession IPC; `D` → DetachSession IPC | unit |
| VP-TBD | `Terminating` session renders `[Terminating]` indicator; lifecycle actions are no-ops | unit |
| VP-TBD | `k` on `Terminating` session → no KillSession IPC sent; status bar hint shown | unit |
| VP-TBD | `spawned_by_monocle: None` session renders `[?]` badge (not blank, not `[M]`, not `[E]`) | unit |
| VP-TBD | `SessionSnapshot.degraded == true` → `[!]` badge rendered; `degraded_reason` in status bar on focus | unit |
| VP-TBD | `D` on Launching session → no DetachSession IPC sent; status bar shows "Session launching — please wait" | unit |
| VP-TBD | `k`/`d` on Launching session → KillSession IPC sent; session → Terminating (kill ALLOWED during Launching) | unit |
| VP-TBD | `r` on Launching session → RenameSession IPC sent; rename ALLOWED during Launching | unit |
| VP-TBD | `k`/`d` on Terminated (GC-grace) session → no KillSession IPC dispatched; status bar shows "Session has terminated" | unit |
| VP-TBD | `D` on Terminated (GC-grace) session → no DetachSession IPC dispatched; status bar shows "Session has terminated" | unit |
| VP-TBD | `r` on Terminated (GC-grace) session → no RenameSession IPC dispatched; status bar shows "Session has terminated" | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability traceability §SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability traceability — this BC extends the sessions panel capability in CAP-006 with multi-session, multi-project grouping, and lifecycle actions |
| Architecture Module | monocle-tui (sessions panel renderer, session list grouping logic, lifecycle keybindings) per ARCH-INDEX Subsystem Registry SS-06 |
| Architecture Source | SS-ipc.md v1.24.0 §SessionSnapshot (wire boundary type; `degraded` and `degraded_reason` fields; `spawned_by_monocle: Option<bool>` field); SS-session-manager.md v2.6.1 §SessionManager §Public API (session_list() returns Vec<SessionSnapshot>); SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state matrix (F-P52-001); SS-embedded-pty.md v1.7.0 §Fast switching; SS-daemon-wiring-v2-delta.md v1.11.4 |
| Cross-Ref | BC-2.05.010 §DetachSession PC-4 (session_not_ready is defensive/untrusted-client-only; official TUI never sends DetachSession during Launching — this BC's Invariant 5 is the normative target); BC-2.08.003 Invariant 2 (kill on Terminated is idempotent at daemon; TUI guard in Invariant 6 prevents dispatch); BC-2.08.003 Invariant 3 (kill on Launching is explicitly allowed; kill path uses host_conn.writer or PID fallback); BC-2.08.005 Invariant 4 (rename on Terminated → Err(InvalidSessionName{"session terminated"}) → "rename_failed"; GC task not cancellable — Invariant 6 of this BC is the TUI-side guard); BC-2.08.007 §Preconditions (detach) defensive note (TUI guard enforced here prevents session_not_ready on official TUI path); SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state matrix (F-P52-001) (daemon-side dispositions: rename → Err/rename_failed; detach → idempotent Ok(()); kill → idempotent Ok(); resize → WARN-drop); BC-2.09.008 (SessionCreation wizard and EmbeddedTerminal enter) |
| Test Name | test_BC_2_06_025_multi_session_grouped_by_project |

## Related BCs

- [BC-2.06.005] — extends: existing sessions panel render from IPC state; this BC extends with grouping + lifecycle actions
- [BC-2.09.008] — composes with: Enter → EmbeddedTerminal or SessionCreation wizard entry
- [BC-2.05.010] — depends on: lifecycle IPC variants (Kill/Detach/Rename) sent from sessions panel

## Architecture Anchors

- `architecture/SS-session-manager.md#public-api` — session_list() returns SessionSnapshot vec

## Story Anchor

S-048 — Implement multi-session grouped sessions panel with lifecycle actions

## VP Anchors

VP-TBD — Sessions panel multi-session render tests (filled after VP creation)

## §Trace v1.5.1

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-048; Invariant 3 cross-reference note added for D/Launching rule** (2026-06-15):
- Story Anchor filled from Phase-2 Burst C story decomposition. No behavioral content changed.
- Invariant 3 (real-time list update): appended a parenthetical cross-reference note directing readers to Invariant 5 for lifecycle keybinding rules during Launching state, specifically that `D` (DetachSession) is BLOCKED. This closes the reference gap exposed by S-048 AC-010 ("per BC-2.06.025 Invariant 3 — detach blocked"), making Invariant 3 and EC-298 self-consistent for readers navigating from story-level references. The normative rule itself (BLOCKED `D` on Launching) has been in Invariant 5 since v1.4.0 and is unchanged. This is a clarifying cross-reference, not a new behavioral obligation.

SE-16d monotonicity: v1.5.1 timestamp 2026-06-15 > v1.5.0 timestamp 2026-06-14. PASS.

## §Trace v1.5.0

**F-P52-001 — Terminated-in-grace panel action guards: k/d, D, r all BLOCKED for [X] sessions** (2026-06-14):

- **Gap closed (F-P52-001):** BC-2.06.025 Invariant 4 (as written through v1.4.0) blocked lifecycle
  actions only for `Terminating` sessions. Sessions with `SessionState::Terminated` rendered a `[X]`
  indicator (Invariant 4 prose) but had no corresponding lifecycle-action guard — a correct
  implementer following v1.4.0 would dispatch `k`, `D`, or `r` on a Terminated session still visible
  in the 10s GC grace window. The architect closed the daemon-side defensive dispositions for these
  cases in SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state matrix
  (F-P52-001): rename → `Err(InvalidSessionName{"session terminated"})` → `"rename_failed"`;
  detach → idempotent `Ok(())`; kill → idempotent `Ok(())`; resize → WARN-drop.

- **Invariant 6 (new — Terminated-in-grace lifecycle action invariant):** Formalizes the blanket
  block for Terminated-in-grace sessions, mirroring the Terminating block in Invariant 4:
  - Kill (`k`/`d`): BLOCKED at TUI. No KillSession IPC dispatched.
  - Detach (`D`): BLOCKED at TUI. No DetachSession IPC dispatched.
  - Rename (`r`): BLOCKED at TUI. No RenameSession IPC dispatched.
  Status bar shows "Session has terminated" for all three blocked actions. This is a blanket
  block — NOT action-specific (unlike the Launching rules in Invariant 5 which are action-specific).

- **EC-300 (new):** `k`/`d` on Terminated (GC-grace) → no KillSession IPC; status bar hint.
- **EC-301 (new):** `D` on Terminated (GC-grace) → no DetachSession IPC; status bar hint.
- **EC-302 (new):** `r` on Terminated (GC-grace) → no RenameSession IPC; status bar hint;
  daemon-side error noted (Err(InvalidSessionName{"session terminated"}) → "rename_failed").

- **VP table:** Three new unit test VPs added covering k/d, D, r on Terminated-in-grace sessions.

- **Traceability Cross-Ref expanded:** Added BC-2.08.003 Invariant 2 (kill idempotency at daemon),
  BC-2.08.005 Invariant 4 (rename-on-Terminated error return; GC task not cancellable), and
  SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state matrix (F-P52-001).

- **Architecture Source updated:** SS-session-manager.md pin v2.5.1 → v2.6.0 (new
  §Terminated-in-grace defensive action×state matrix); SS-ipc.md pin v1.23.1 → v1.23.2;
  SS-daemon-wiring-v2-delta.md v1.11.2 → v1.11.3; SS-session-manager §I3-009 note retained.

- **Pass-51 Launching rules (Invariant 5) NOT regressed.** Kill ALLOWED, Detach BLOCKED, Rename
  ALLOWED for Launching sessions — fully preserved.

- **Minor bump: v1.4.0 → v1.5.0** (normative addition: new Terminated-in-grace action guard —
  Invariant 6 + EC-300/301/302 + VP entries + Cross-Ref expansion; behavioral obligation added for
  all three lifecycle actions during Terminated-in-grace state).

SE-16d monotonicity: v1.5.0 timestamp 2026-06-14T01:00:00Z > v1.4.0 timestamp 2026-06-14T00:00:00Z. PASS.

## §Trace v1.4.0

**F-P51-001 — Explicit Launching-state action rules: DetachSession BLOCKED, Kill ALLOWED, Rename ALLOWED** (2026-06-14):

- **Gap closed (F-P51-001):** BC-2.05.010 §DetachSession PC-4 and BC-2.08.007 §Preconditions
  (detach) defensive note both cite "BC-2.06.025 guards" as the TUI-side guarantee that the
  official TUI never dispatches `ClientToServer::DetachSession` during `Launching`. Prior to
  this version, BC-2.06.025 had no explicit Launching-state action rule for `D` (DetachSession).
  The existing PC-3 listed `D` → DetachSession as an unconditional keybinding, and the existing
  Invariant 4 only blocked actions for Terminating sessions. As written, an implementer following
  the spec literally WOULD dispatch DetachSession during Launching, making `session_not_ready`
  a normal user-facing path — contradicting the "defensive/untrusted-client-only" framing in
  BC-2.05.010 and BC-2.08.007.

- **PC-3 extended — Launching-state keybinding rules subsection (normative):** Added
  "Launching-state keybinding rules" block with three explicit action dispositions:
  - Kill (`k`/`d`): ALLOWED — `KillSession` IPC dispatched normally (daemon handles
    Launching kill per BC-2.08.003 Invariant 3 / PC-1 three-case logic).
  - Detach (`D`): BLOCKED at TUI — `DetachSession` NOT dispatched; status bar shows
    "Session launching — please wait". No-op. Backing guarantee for the "defensive/
    untrusted-client-only" framing in BC-2.05.010 §DetachSession PC-4 and BC-2.08.007
    §Preconditions (detach) defensive note.
  - Rename (`r`): ALLOWED — `RenameSession` IPC dispatched normally (rename is metadata
    only; does not require `host_conn`; succeeds on any non-Terminated session).

- **Invariant 5 (new — Launching-state lifecycle action invariant):** Formalizes the three
  Launching-state dispositions as normative invariants. Explicitly states that this is NOT
  a blanket disable (unlike Invariant 4 for Terminating). States that Invariant 5 is the
  normative target of all "BC-2.06.025 guards" cites in BC-2.05.010 and BC-2.08.007.

- **EC-298 (new):** `D` on Launching → no DetachSession IPC; status bar hint. Per Invariant 5.

- **EC-299 (new):** `r` on Launching → RenameSession IPC dispatched; ALLOWED. Per Invariant 5.

- **VP table:** Added three new unit test VPs covering the Launching-state action rules
  (detach no-op, kill allowed, rename allowed).

- **Traceability Cross-Ref:** Expanded to include BC-2.05.010 §DetachSession PC-4,
  BC-2.08.003 Invariant 3, and BC-2.08.007 §Preconditions (detach) defensive note
  as bidirectional cross-references. These resolve the dangling "BC-2.06.025 guards" cites.

- **Cited symbols verified:** BC-2.08.003 Invariant 3 (line 107: kill on Launching allowed;
  Launching → Terminating). BC-2.08.003 PC-1 three-case structure (lines 53-55: Launching
  with/without host_conn kill sub-paths). BC-2.05.010 §DetachSession PC-4 (lines 139-148:
  session_not_ready defensive path). BC-2.08.007 §Preconditions (detach) defensive note
  (lines 47-55: official TUI never sends DetachSession during Launching). All resolve.

- **Minor bump: v1.3.1 → v1.4.0** (normative addition: new Launching-state action rules —
  PC-3 subsection + Invariant 5 + EC-298/299 + VP entries; behavioral obligation added for
  all three lifecycle actions during Launching state).

SE-16d monotonicity: v1.4.0 timestamp 2026-06-14 > v1.3.1 timestamp 2026-06-14. PASS.

## §Trace v1.3.1 (errata)

**S-P47-001 — Add `d` kill-alias to Terminating disabled-action lists** (2026-06-14):
- **Finding (S-P47-001):** PC-1 and Invariant 4 disabled-action lists enumerated `k`, `D`, `r`
  for Terminating sessions but omitted the lowercase `d` kill-alias. PC-3 defines kill as
  "`k` or `d`", making the omission an intra-document contradiction: a literal implementer
  could permit `d`-triggered kill on a Terminating session, contradicting EC-296.
- **PC-1 (clarifying errata):** `(`k`, `D`, `r`)` → `(`k`/`d`, `D`, `r`)` with a parenthetical
  noting `d` is the kill alias per PC-3.
- **Invariant 4 (clarifying errata):** Same substitution.
- **Bump disposition:** Errata-no-bump — this is clarification of clear existing intent;
  no normative obligation changes. (`d`≡`k` was already implied by PC-3.) Version stays v1.3.1.

**Arch-source pin v1.5.1→v1.5.2** (2026-06-13 / D-277):
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.3.0

**Adversarial Pass 3 fixes — I3-009 (degraded badge + SessionSnapshot wire type)** (2026-06-03):
- I3-009: PC-1 updated to explicitly source session data from `Vec<SessionSnapshot>` (SS-ipc.md
  v1.12.0 wire boundary type) not `EnrichedSession`. `EnrichedSession` is internal to
  `EngineModule::detect()` and never crosses the UDS wire. PC-1 adds the degraded badge rule:
  `SessionSnapshot.degraded == true` → `[!]` badge + `degraded_reason` in status bar on focus.
  PC-4 updated: `spawned_by_monocle` field is now from `SessionSnapshot` (not `EnrichedSession`
  per SS-engine-module-v2-delta.md — that reference was for the detection path, not the wire).
- Precondition 2 updated: wire type is `SessionSnapshot`, not `EnrichedSession`.
- EC-297 added: degraded session edge case.
- VP table: added degraded badge unit test.
- Architecture Source updated to SS-ipc.md v1.13.0 (SessionSnapshot fields) and
  SS-session-manager.md v1.5.0 (I3-009 degraded-env mechanism).

## §Trace v1.2.0

**Architect-delegated BC edits — Terminating state render + lifecycle action blocking** (2026-06-03):
- Architect delegated from SS-session-manager.md v1.3.0 §Terminating state (I2-004): sessions
  panel must render `[Terminating]` state and disable lifecycle actions for Terminating sessions.
- PC-1: `Terminating` added to SessionState indicator list with `[Terminating]` render spec
  and lifecycle-action-disabled rule.
- Invariant 4: `Terminating` state render and lifecycle action blocking specified with cross-
  reference to BC-2.08.003 Invariant 4.
- EC-293: updated — `k` on Launching sends Kill and transitions to Terminating, not directly
  to Terminated.
- EC-296 added: `k` on Terminating session is idempotent (no-op, status bar hint).
- VP table: added Terminating render and no-op verification properties.

## §Trace v1.1.0

**Adversarial pass-1 fix O5 — specify `spawned_by_monocle: None` render** (2026-06-03):
- PC-1 extended: added `[?]` badge for `spawned_by_monocle: None` (pre-v1A forward-compat
  sessions). The tri-state `Option<bool>` field now has a fully specified render for all three
  values: `Some(true)` → `[M]`, `Some(false)` → `[E]`, `None` → `[?]`.
- PC-4 extended: added `None` → `[?]` to the monocle/external badge specification.
- EC-295 added: edge case documents the `None` session behavior and lifecycle treatment.
- VP table: added unit test VP for `[?]` badge render.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.06.025 authored for SS-06 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): keybinding choices (n/k/r/D) selected per convention from
  lazygit-style single-char actions. These are TUI conventions that do not require human
  input — they are standard for this kind of panel. Conflict check against existing
  keybinding table (BC-2.06.003): `n`, `k`, `r`, `D` are not currently bound in Dashboard
  mode; allocation is safe.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).

## §Trace v1.5.3

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.5.3 timestamp >= v1.5.2. PASS.
