---
document_type: behavioral-contract
level: L3
version: "1.1.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md, architecture/SS-tui.md]
input-hash: "4234f2c"
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

# Behavioral Contract BC-2.09.009: Permission Badge+Bell — Status Bar Badge + Audible Bell Within One Render Tick While in EmbeddedTerminal or SessionCreation

## Description

Permission prompts (PreToolUse hooks requiring a decision) are monocle's killer feature.
While `AppMode::EmbeddedTerminal` or `AppMode::SessionCreation` is active, incoming
`PermissionPromptQueued` IPC messages MUST immediately raise a visible badge in the status
bar AND emit an audible terminal bell within one render tick. Prompts MUST NOT be silently
queued without user awareness. Pressing Esc exits embedded terminal mode and the pending
overlay is presented in the `prior` AppMode. This is the v1A production-grade minimum;
full pre-emption (overlay replacing embedded terminal) is a v1B enhancement requiring human
sign-off.

## Preconditions

1. `AppMode::EmbeddedTerminal { session_id, prior }` OR `AppMode::SessionCreation` is active.
2. The daemon receives a `PreToolUse` hook POST requiring a decision (a new permission prompt).
3. The daemon broadcasts `ServerToClient::PermissionPromptQueued { ... }` to all TUI clients.
4. The TUI is connected and receives the `PermissionPromptQueued` IPC message.

## Postconditions

1. The `PermissionPromptQueued` payload is added to `App::overlay_stack` (existing behavior
   from BC-2.06.008) within one render tick of IPC receipt.
2. The status bar renders a visible badge indicating pending prompts, within one render tick
   of the `overlay_stack` being updated. The badge format is:
   `[N pending permission(s)]` (where N is `overlay_stack.len()`).
3. The terminal bell is emitted (by writing `\x07` — BEL character — to stdout) once per
   new `PermissionPromptQueued` event — every prompt rings the bell. NOT one bell per render
   tick; NOT capped at the first prompt. Each arriving `PermissionPromptQueued` event triggers
   exactly one bell regardless of how many prompts are already pending.
4. The badge MUST be visible in the status bar even while the PTY widget occupies the main
   pane area. The status bar is rendered below the main pane and is always visible.
5. When the user presses Esc in `EmbeddedTerminal`:
   a. AppMode transitions to `prior` (per BC-2.09.008 Esc exit postcondition).
   b. Because `overlay_stack` is non-empty, AppMode immediately transitions to
      `AppMode::Overlay { prior: Dashboard }` (per existing overlay stack semantics).
   c. The permission overlay renders the front of `overlay_stack` (the oldest pending prompt).
6. While in `SessionCreation` mode, the badge renders identically. Pending overlays are
   accessible after the wizard is dismissed (Esc on any step → prior AppMode → Overlay if stack non-empty).

## Invariants

1. **No silent queueing:** Prompts MUST NOT be held invisibly until the user exits embedded
   mode. The badge + bell is the mandatory minimum visibility guarantee. This invariant is
   production-grade non-negotiable (SS-embedded-pty.md §State machine invariants, SUG-3).
2. The bell (`\x07`) is written to stdout exactly ONCE per `PermissionPromptQueued` event —
   every new prompt rings the bell, including the second, third, etc. prompts in rapid
   succession. Rationale: every new blocking permission prompt is an independent user
   attention demand; silencing bells after the first would allow subsequent prompts to
   arrive invisibly if the user is not watching the screen. Visibility wins over bell
   suppression. This rule is consistent with HS-EXP-013 step 7 ("a second bell is
   emitted") and must be consistent with the holdout's must-pass satisfaction criteria.
3. `overlay_stack` is populated by the standard `PermissionPromptQueued` handler
   (BC-2.06.008 applies regardless of the current AppMode). This BC adds only the badge
   and bell as additional side effects when `AppMode` is `EmbeddedTerminal` or
   `SessionCreation`.
4. The v1A behavior is badge-only (badge + bell). Full pre-emption (overlay immediately
   REPLACES embedded terminal without requiring Esc) is v1B scope and requires human
   ratification per SS-embedded-pty.md §State machine invariants §BC requirement flag.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-260 | Two permission prompts arrive in rapid succession | Badge shows `[2 pending permissions]`; bell emitted for EACH prompt (two bells total) |
| EC-261 | Permission prompt arrives while in `SessionCreation::Launching` | Badge appears in status bar; bell emitted; wizard continues; user can Esc to cancel wizard and reach overlay |
| EC-262 | User resolves all pending prompts (overlay_stack empties) | Badge disappears from status bar; no bell |
| EC-263 | Permission prompt from a non-focused session while in EmbeddedTerminal | Badge shows; bell emitted; the prompt is for a different session — the badge does not indicate which session's prompt it is (v1A scope; session-specific badges are v1B) |
| EC-264 | `AppMode::EmbeddedTerminal`; user presses Esc; no pending overlays | AppMode → `prior` (Dashboard); no overlay displayed; normal Dashboard render |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| EmbeddedTerminal active; `PermissionPromptQueued` received | Badge `[1 pending permission]` in status bar; bell `\x07` to stdout | happy-path |
| Two rapid `PermissionPromptQueued` events | Badge `[2 pending permissions]`; bell emitted twice (once per prompt) | happy-path |
| Esc in EmbeddedTerminal with 1 pending prompt | AppMode → Dashboard → Overlay; prompt displayed | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Badge rendered in status bar on `PermissionPromptQueued` while in EmbeddedTerminal | unit |
| VP-TBD | Bell (`\x07`) emitted once per new `PermissionPromptQueued` event — every prompt rings, including second+ prompts; not per render tick | unit |
| VP-TBD | Esc from EmbeddedTerminal with pending prompt → overlay appears | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — this BC governs the behavior of the embedded terminal mode when permission prompts arrive; it is an invariant of the EmbeddedTerminal AppMode, which is defined in CAP-009; the "never silently queue" guarantee is core to the embedded terminal UX |
| L2 Domain Invariants | (none — DI-NNN catalog not produced; permission prompt visibility is captured as an SS-embedded-pty invariant) |
| Architecture Module | monocle-tui (status bar badge renderer, bell emit, overlay_stack integration); monocle-core (AppMode transition logic) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md §State machine invariants (permission prompts while in EmbeddedTerminal; SUG-3 fix); §BC requirement flag (v1B pre-emption deferred) |
| Cross-Ref | BC-2.06.008 (Permission Overlay: VecDeque Stack Push on PermissionPromptQueued — overlay_stack populated regardless of AppMode); BC-2.09.008 (Esc exit from EmbeddedTerminal restores prior; with pending overlay → Overlay AppMode) |
| Test Name | test_BC_2_09_009_permission_badge_bell_in_embedded_terminal |

## Related BCs

- [BC-2.06.008] — depends on: PermissionPromptQueued always adds to overlay_stack; this BC adds badge+bell
- [BC-2.09.008] — composes with: Esc exit from EmbeddedTerminal is the user action that leads to overlay display

## Architecture Anchors

- `architecture/SS-embedded-pty.md#state-machine-invariants` — SUG-3 permission prompt visibility rule

## Story Anchor

S-044 — Implement permission badge + bell in EmbeddedTerminal status bar

## VP Anchors

VP-TBD — Badge + bell unit tests (filled after VP creation)

## §Trace v1.1.6

**Arch-source pin: SS-embedded-pty.md v1.10.0 → v1.11.0** (2026-06-20):
- S-040 delivery flag-set correction bumped SS-embedded-pty to v1.11.0. Architecture Source
  row updated. No behavioral content changed.
- SE-16d monotonicity: v1.1.6 timestamp >= v1.1.5. PASS.

## §Trace v1.1.4

**Arch-source pin: SS-embedded-pty.md v1.7.0 → v1.10.0** (2026-06-20):
- S-039 adversarial convergence bumped SS-embedded-pty to v1.10.0. This BC's Architecture Source
  row is updated to reflect the current version. No behavioral content changed.
- SE-16d monotonicity: v1.1.4 timestamp >= v1.1.3. PASS.

## §Trace v1.1.3

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-044** (2026-06-15):
- Story Anchor filled from Phase-2 Burst B story decomposition. No behavioral content changed.

## §Trace v1.1.2

**Arch-source pin v1.5.1→v1.5.2** (2026-06-13 / D-277):
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.1.1

**S24-002 stale §Trace annotation fix** (2026-06-13):
- Finding S24-002 (Phase-1d Pass 24, SUGGESTION): §Trace v1.0.0 "Design decision" bullet stated
  "Bell emitted once per new prompt (not per second prompt in rapid succession)" — the once-only
  rule that v1.1.0 C2-001 superseded. The normative body (PC-3, Invariant 2, EC-260, test
  vectors) was already correct at v1.1.0. Only the historical §Trace v1.0.0 bullet retained
  the superseded framing, creating a misleading audit trail for readers scanning §Trace chronology.
- Fix: Appended superseded marker to the §Trace v1.0.0 design decision bullet:
  "(SUPERSEDED by v1.1.0 C2-001 — the canonical rule is per-prompt: every prompt rings,
  including the second in rapid succession)".
- No normative body changes. Patch version bump to 1.1.1.

## §Trace v1.1.0

**C2-001 adversarial pass-2 fix — per-prompt bell rule** (2026-06-03):
- C2-001 finding: BC-2.09.009 had a three-way internal contradiction: PC-3 said "bell once per
  new `PermissionPromptQueued` event" which implies every prompt rings; Invariant 2 + EC-260
  said "second prompt does NOT re-ring the bell" which means only first rings; HS-EXP-013 step
  7 asserted "a second bell is emitted" which matches every-prompt. Three artifacts, three
  different rules.
- Resolution: per-prompt bell rule adopted as the single canonical rule. Every
  `PermissionPromptQueued` arrival emits exactly one bell. Rationale documented in Invariant
  2: visibility wins over bell suppression; each new blocking permission prompt is an
  independent attention demand; silencing after the first creates invisible follow-on prompts.
- PC-3: rewritten to state "every prompt rings" clearly.
- Invariant 2: rewritten with rationale for per-prompt rule; cites HS-EXP-013 step 7 for
  consistency.
- EC-260: updated from "FIRST prompt only" to "EACH prompt (two bells total)".
- Canonical test vector: updated from "bell emitted once" to "bell emitted twice (once per
  prompt)".
- BC is now internally consistent (PC-3 ↔ Invariant 2 ↔ EC-260 ↔ test vector) AND
  consistent with HS-EXP-013 (holdout step 7 asserts two bells).

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.009 authored for SS-09 as part of the v1A control-center pivot BC burst.
- SUG-3 contract: architect explicitly flagged this for product-owner authorship in
  SS-embedded-pty.md §BC requirement flag. This BC encodes the badge-only v1A minimum
  (production-grade non-negotiable per SS-embedded-pty.md v1.0.2). Pre-emption is v1B;
  human sign-off is required before v1B BC authoring.
- Design decision (in-scope): Bell emitted once per new prompt (not per second prompt in rapid
  succession) per Invariant 2. This is production-grade UX; per-prompt bells for rapid-fire
  prompts would be disruptive. **(SUPERSEDED by v1.1.0 C2-001 — the canonical rule is
  per-prompt: every prompt rings, including the second in rapid succession)**
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
