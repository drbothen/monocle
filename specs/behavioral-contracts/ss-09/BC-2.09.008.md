---
document_type: behavioral-contract
level: L3
version: "1.3.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-16T00:00:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md, architecture/SS-ipc.md]
input-hash: "4367e18"
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
4. The scoped mouse-capture entry sequence executes in order: (a) `crossterm::execute!(stdout(), EnableMouseCapture)?` (enables button-event tracking mode 1002), then (b) `print!("\x1b[?1006h")` (enables SGR 1006 extended encoding). Both steps are required; SGR 1006 alone is insufficient without the preceding `EnableMouseCapture` call. Non-authoritative restatement — authoritative contract is **BC-2.09.002 Invariant-5** (entry/exit sequence, ordering, and rationale are defined there).
5. A status bar indicator shows `[EmbeddedTerminal: <session_display_name>]`.

## Postconditions (entering SessionCreation)

1. `AppMode` transitions to `SessionCreation { step: ProfilePicker, prior: <current FocusSnapshot> }`.
2. Step 1 (ProfilePicker): existing profile-picker UI (BC-2.07.004/005 logic) renders.
3. Step 2 (ProjectPicker): nucleo-filtered list of recent project roots; free-text entry.
4. Step 3 (WorktreeConfirm): editable display name + resolved worktree path.
5. Step 4 (Launching): TUI sends `ClientToServer::SpawnSession` to daemon. Status bar shows
   `[Launching session...]`. On receipt of `ServerToClient::SpawnAck { session_id }`, the wizard
   stores it: `AppMode::SessionCreation { launching_session_id: Some(session_id.clone()), .. }`.
   This is a point-to-point message to the requesting client only — it is NOT broadcast.
   `SpawnAck` is guaranteed to arrive before any broker-published
   `ServerToClient::SessionStateChanged { new_state: Launching }` by TWO complementary
   properties: (1) **Causal step ordering** — in the daemon IPC handler, `SpawnAck` is sent at
   step 2 (before `spawn_session()` is called at step 4, which is before the broker emits
   `SessionStateChanged { Launching }` at step 5); and (2) **Per-client FIFO** — the requesting
   client's per-client `mpsc` channel delivers messages in send order, guaranteeing that
   `SpawnAck` (step 2) arrives at the TUI before any broker-published
   `SessionStateChanged { Launching }` (step 5).
   Canonical source: SS-ipc.md §ServerToClient::SpawnAck §Delivery ordering steps 1-5.
6. Step 5 (auto-advance): When `ServerToClient::SessionStateChanged { session_id: <matching launching_session_id>, new_state: Running }` is received, `AppMode` auto-transitions to `EmbeddedTerminal { session_id, prior: Dashboard }`. The match is against `launching_session_id` (deterministic — populated from `SpawnAck`), NOT a broadcast heuristic. `SessionStateChanged` events whose `session_id` does not match `launching_session_id` are ignored by the wizard.
7. If spawn fails (daemon returns `ServerToClient::Error`): wizard clears `launching_session_id` to `None` and returns to `ProfilePicker` with an error banner.

## Postconditions (exiting EmbeddedTerminal)

1. Esc in `EmbeddedTerminal`: `Action::ExitEmbeddedTerminal` fires. AppMode transitions
   to `prior` AppMode (typically `Dashboard`). The scoped mouse-capture exit sequence executes in order: (a) `print!("\x1b[?1006l")` (disables SGR 1006 encoding), then (b) `crossterm::execute!(stdout(), DisableMouseCapture)?` (disables button-event tracking mode 1002). Ordering is critical — SGR `l` BEFORE `DisableMouseCapture`. Non-authoritative restatement — authoritative contract is **BC-2.09.002 Invariant-5** (exit sequence, ordering, and rationale are defined there).
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
| EC-303 | `SessionStateChanged { session_id: X }` received while in `SessionCreation::Launching` but `X` does not match `launching_session_id` | Event is silently ignored by the wizard — no AppMode transition. `launching_session_id` is set from `SpawnAck` (point-to-point to requesting client) before any `SessionStateChanged { Launching }` broadcast arrives (causal step ordering + per-client FIFO guarantee in SS-ipc.md §SpawnAck §Delivery ordering). A non-matching `session_id` therefore belongs to a concurrent spawn from another TUI client or a stale broadcast from a prior spawn — it MUST NOT trigger auto-advance to EmbeddedTerminal. The wizard only auto-advances on `SessionStateChanged { new_state: Running, session_id: <matching launching_session_id> }` (PC-6). |

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
| Architecture Source | SS-embedded-pty.md v1.7.0 §TUI AppMode Extensions (EmbeddedTerminal, SessionCreation with `launching_session_id: Option<String>`, SessionCreationStep — F-P41-IMP-001); §Session Creation Wizard (SpawnAck receipt + launching_session_id storage + auto-advance match logic); §State machine invariants; SS-ipc.md v1.24.0 §ServerToClient::SpawnAck (point-to-point delivery to requesting client; wizard storage and filtering obligation) |
| Test Name | test_BC_2_09_008_embedded_terminal_transitions |

## Related BCs

- [BC-2.09.009] — composes with: permission badge + bell fires while in EmbeddedTerminal or SessionCreation
- [BC-2.09.001] — depends on: PTY output renders once EmbeddedTerminal is entered
- [BC-2.08.001] — depends on: SessionCreation wizard triggers spawn_session()

## Architecture Anchors

- `architecture/SS-embedded-pty.md#tui-appmode-extensions` — EmbeddedTerminal/SessionCreation variant definitions; `launching_session_id: Option<String>` field (F-P41-IMP-001)
- `architecture/SS-embedded-pty.md#session-creation-wizard` — wizard step sequence; SpawnAck receipt → launching_session_id storage; auto-advance match against launching_session_id
- `architecture/SS-ipc.md#servertoClientspawnack` — SpawnAck variant; point-to-point delivery; wizard storage obligation

## Story Anchor

S-044 — Implement EmbeddedTerminal/SessionCreation AppMode transitions in monocle-tui and monocle-core

## VP Anchors

VP-TBD — AppMode transition tests (filled after VP creation)

## §Trace v1.3.3

**Phase-2 Pass-1 fix burst — EC-303 added: SpawnAck/launching_session_id deterministic filter for non-matching SessionStateChanged events** (2026-06-16):

- **Gap closed:** SS-ipc.md §ServerToClient::SpawnAck doc-comment (line 489) and
  §ClientToServer::SpawnSession comment (line 574) both reference "EC-303" as the
  normative edge case defining the deterministic `session_id` filter applied by the wizard
  in `SessionCreation::Launching` when it receives `SessionStateChanged` events whose
  `session_id` does NOT match `launching_session_id`. Prior to this patch, BC-2.09.008
  had no edge case EC-303 (or equivalent). The forward-reference in SS-ipc.md pointed
  to a non-existent EC — a genuine broken cross-reference.

- **EC-303 (new):** `SessionStateChanged { session_id: X }` received in
  `SessionCreation::Launching` where `X` does not match `launching_session_id`. Expected
  behavior: wizard silently ignores the event. Only a `SessionStateChanged { new_state:
  Running, session_id: <matching launching_session_id> }` triggers auto-advance (PC-6).
  The EC cites the causal step ordering + per-client FIFO guarantee from
  SS-ipc.md §SpawnAck §Delivery ordering as the basis for why non-matching events
  indicate a different TUI client's spawn or a stale broadcast.

- **EC number selection:** EC-303 is the next sequential number after EC-302 (the highest EC
  in BC-2.06.025, the sibling BC authored immediately before BC-2.09.008 in the same burst).
  EC-303 does NOT collide with BC-2.09.008's existing EC namespace (EC-250..EC-254). Using
  303 matches the identifier already cited in SS-ipc.md — no renaming needed.

- **PATCH bump: v1.3.2 → v1.3.3** (adds one new edge case; no postcondition or invariant
  content changed; no existing behavioral obligations modified).

SE-16d monotonicity: v1.3.3 timestamp 2026-06-16T00:00:00Z > v1.3.2 timestamp 2026-06-14T19:00:00Z. PASS.

## §Trace v1.3.2

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-044** (2026-06-15):
- Story Anchor filled from Phase-2 Burst B story decomposition. No behavioral content changed.

## §Trace v1.3.1

**F-P46-IMP-001 — §Architecture Anchors: version parentheticals stripped; version-less navigational convention adopted** (2026-06-14):

- **Change:** Removed ` (vX.Y.Z)` parentheticals from all 3 entries in §Architecture Anchors (SS-embedded-pty.md ×2, SS-ipc.md). No normative content changed.
- **Rationale:** Version pins in navigational anchors duplicate the authoritative §Architecture Source Traceability-table row (POL-11-enforced) and are invisible to POL-11's ID↔version adjacency regex when in the `` `path#anchor` (vX.Y.Z) `` form. Eliminating the duplication removes the drift class entirely. Authoritative version citations remain in the §Architecture Source table unchanged: SS-embedded-pty.md v1.6.0 <!-- version-pin-historical: canonical at §Trace v1.3.1 authoring time; superseded by v1.7.0 at Phase-2 Pass-2 fix burst -->, SS-ipc.md v1.22.0 <!-- version-pin-historical: canonical at §Trace v1.3.1 authoring time -->.
- **Bump disposition:** Errata-no-bump (navigational-anchor-only change, precedent D-275). BC version stays at v1.3.1.

**CV-SS-005-SIBLING — PC Step 4 (Launching): expand FIFO-only ordering claim to state BOTH causal step ordering AND per-client FIFO (mirroring BC-2.08.008 v1.2.1 fix)** (2026-06-14):

- **Finding:** PC Step 4 (Launching) contained the same incomplete ordering claim that was fixed
  in BC-2.08.008 PC-5 during this burst (CV-SS-005). The claim cited only "per-client FIFO ordering"
  without stating WHY that is sufficient — specifically, without naming the causal step ordering
  that makes FIFO alone meaningful: `SpawnAck` is sent at step 2 in the daemon IPC handler,
  BEFORE `spawn_session()` is called at step 4, and the broker does not emit
  `SessionStateChanged { Launching }` until step 5. The FIFO channel then guarantees arrival
  order, but FIFO alone cannot guarantee the invariant without the causal step ordering
  establishing the enqueue order in the first place.

- **Fix:** PC Step 4 Launching ordering claim expanded to state both properties explicitly:
  (1) **Causal step ordering** — IPC handler sequence steps 2, 4, 5; and
  (2) **Per-client FIFO** — requesting client's `mpsc` channel delivers in send order.
  Canonical source citation added: SS-ipc.md §ServerToClient::SpawnAck §Delivery ordering steps 1-5.
  Wording mirrors BC-2.08.008 PC-5 v1.2.1 fix verbatim for class consistency.

- **Whole-class check:** All FIFO/ordering/SpawnAck-arrival claims in BC-2.09.008 scanned.
  Two instances of "FIFO ordering" language found:
  1. PC Step 4 (Launching) live postcondition — FIXED in this patch.
  2. §Trace v1.3.0 historical narrative (lines ~176-178) — this is a historical record of
     the v1.3.0 understanding at the time of that patch; it is preserved verbatim as immutable
     historical trace. The §Trace v1.3.1 entry (this entry) supersedes it for the live claim.
  Zero remaining FIFO-only-without-causal survivors in the live postconditions of BC-2.09.008.

- **No wire/contract change.** No field names, variant names, step labels, or wire behaviors
  changed. This is a precision/completeness fix to the ordering justification only.

- **Semver decision:** PATCH bump (1.3.0 → 1.3.1). The fix adds justification precision to an
  existing ordering claim; the claim itself (SpawnAck arrives before SessionStateChanged{Launching})
  was already correct. No behavioral content changed. Mirrors the PATCH decision in BC-2.08.008
  v1.2.1 for the same class of fix.

- SE-16d monotonicity: v1.3.1 timestamp 2026-06-14T19:00:00Z > v1.3.0 timestamp 2026-06-14T12:00:00Z. PASS.

## §Trace v1.3.0

**F-P41-IMP-001 — Wizard Step 4 (Launching): SpawnAck receipt + launching_session_id storage; Step 5 (auto-advance): deterministic match against launching_session_id; arch-source pins to SS-embedded-pty v1.6.0 <!-- version-pin-historical: canonical at §Trace v1.3.0 authoring time; superseded by v1.7.0 at Phase-2 Pass-2 fix burst --> + SS-ipc v1.22.0 <!-- version-pin-historical -->** (2026-06-14):

- **PC Step 4 (Launching) — SpawnAck receipt (normative addition):** When
  `ServerToClient::SpawnAck { session_id }` is received, the wizard stores it:
  `AppMode::SessionCreation { launching_session_id: Some(session_id.clone()), .. }`.
  `SpawnAck` is point-to-point (requesting client only; NOT broadcast). It arrives
  before any broker-published `SessionStateChanged { Launching }` per per-client
  FIFO ordering. Status bar shows `[Launching session...]`. Old Step 5 (auto-advance
  on `SessionStateChanged`) is renumbered to Step 6; old Step 6 (spawn-fail) is
  renumbered to Step 7.

- **PC Step 5 (auto-advance) — deterministic match (normative rewrite):** Auto-advance
  now matches `SessionStateChanged.session_id` against `launching_session_id` (populated
  from `SpawnAck`), NOT a broadcast-race heuristic. Events not matching `launching_session_id`
  are ignored. This replaces the implicit "any Running event" model that was unimplementable
  for concurrent spawns from multiple TUI clients.

- **Spawn-fail clearing (normative addition):** On receipt of `ServerToClient::Error`
  after `SpawnSession`, wizard clears `launching_session_id` to `None` before returning
  to ProfilePicker. (SpawnAck was already sent by the daemon for the failed spawn; the
  id is now stale.)

- **Arch-source pin:** SS-embedded-pty.md v1.6.0 <!-- version-pin-historical: at §Trace v1.3.0 authoring time, pin was confirmed as v1.6.0 → v1.6.0 (no-op update); superseded by v1.7.0 at Phase-2 Pass-2 fix burst --> → v1.6.0 (new `launching_session_id`
  field in `AppMode::SessionCreation`; wizard SpawnAck wiring — F-P41-IMP-001);
  SS-ipc.md v1.22.0 added (SpawnAck variant). Architecture Anchors updated to match.

- No change to: EmbeddedTerminal enter/exit postconditions (PC-1..PC-5), Invariants
  1-4, EC-250..EC-254, Canonical Test Vectors, or Verification Properties.

- SE-16d monotonicity: v1.3.0 timestamp 2026-06-14 > v1.2.0 timestamp 2026-06-14. PASS.

## §Trace v1.2.0

**S38-001 — PC-4/PC-1: complete scoped mouse-capture sequences; add BC-2.09.002 Invariant-5 cross-references** (2026-06-14 / Pass-38 adversarial finding):

- S38-001 (Pass-38 IMPORTANT): PC-4 (entering EmbeddedTerminal, postcondition 4) named only `ESC [ ? 1006 h` (the SGR write), omitting the paired `EnableMouseCapture` step that must precede it. PC-1 (exiting EmbeddedTerminal, postcondition 1) named only `ESC [ ? 1006 l` (the SGR disable), omitting the paired `DisableMouseCapture` step that must follow it. Both restatements were partial — they implied the SGR writes alone are sufficient, which contradicts the authoritative scoped model in BC-2.09.002 Invariant-5 and SS-embedded-pty.md §EmbeddedTerminal ENTRY/EXIT (lines 278-288 / 349-360).
- PC-4 rewritten: full two-step entry sequence (EnableMouseCapture → SGR 1006 h), with explicit cross-reference to BC-2.09.002 Invariant-5 as the authoritative owning contract. Non-authoritative restatement label added.
- PC-1 rewritten: full two-step exit sequence (SGR 1006 l → DisableMouseCapture), with explicit cross-reference to BC-2.09.002 Invariant-5 as the authoritative owning contract. Critical ordering preserved (SGR `l` BEFORE DisableMouseCapture). Non-authoritative restatement label added.
- Whole-class sweep: all SS-09 BCs (001-009) and all SS-05 BCs (001-011) scanned for same partial-restatement pattern. No other live instances found. BC-2.09.002 and BC-2.09.003 are authoritative and were NOT modified (already correct).
- Version bump: 1.1.1 → 1.2.0 (minor: new normative postcondition content — full sequences specified where only SGR codes appeared previously).

## §Trace v1.1.1

**Arch-source pin v1.5.1→v1.5.2** (2026-06-13 / D-277):
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- No behavioral content changed. Patch bump only.

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
