---
document_type: behavioral-contract
level: L3
version: "1.5.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T00:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/SS-ipc.md, architecture/ARCH-INDEX.md]
input-hash: "637ae20"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D2-010, F-P1D3-007, O-P1D3-004]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.023: TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved

## Description

When the TUI receives a `ServerToClient::PermissionPromptResolved { prompt_id }` message
from the daemon, it removes the matching `PromptModal` from its `VecDeque<PromptModal>`
overlay stack. If the removal makes the stack empty and `AppMode` is `Overlay`, the TUI
collapses to `Dashboard` per the BC-2.06.001 empty-stack invariant. If no matching
`prompt_id` is found in the stack (e.g., the prompt was already cleared by a SOQ-3
disconnect or a previous resolution), the message is a no-op. This BC handles the daemon's
authoritative notification that a permission prompt has been resolved by any means (user
decision, timeout fail-open, or another TUI client).

## Preconditions

1. The TUI is connected to the daemon via IPC (Unix domain socket).
2. The TUI receives a `ServerToClient::PermissionPromptResolved { prompt_id }` message
   from the daemon over the established IPC connection.
3. The TUI's `VecDeque<PromptModal>` overlay stack may or may not contain a `PromptModal`
   with a `prompt_id` field matching the received `prompt_id`. Both cases are valid
   preconditions and are handled by this BC.

## Postconditions

**PC-1 — Matching prompt found in VecDeque: prompt removed.**
If the `VecDeque<PromptModal>` contains a `PromptModal` entry whose `prompt_id` field
matches the received `prompt_id`, that `PromptModal` is removed from the `VecDeque`.
The removal preserves the relative order of all remaining entries in the `VecDeque`.

**PC-2 — Stack empties after removal: AppMode collapses to Dashboard.**
If, after the removal in PC-1, `App.overlay_stack` becomes empty AND `AppMode` is currently
`Overlay { prior }`, the `AppMode` MUST transition to
`Dashboard { focused: prior.sessions_or_default() }` per the BC-2.06.001 empty-stack
collapse invariant. This transition is identical to the collapse triggered by the last
user decision keystroke — the mechanism is reused, not duplicated.

**PC-3 — No matching prompt_id in VecDeque: no-op.**
If the `VecDeque` does not contain any `PromptModal` with the received `prompt_id`,
the message is silently discarded. No state change occurs. No error is logged at WARN
or ERROR level; this is expected behavior when:
  a. The prompt was already removed via the user's own decision (BC-2.06.011/012/013).
  b. The prompt was cleared by SOQ-3 (daemon disconnect per BC-2.06.016) before this
     `PermissionPromptResolved` arrived (e.g., after reconnect).
  c. The prompt timed out and was auto-resolved by the daemon before the TUI saw it.

**PC-4 — VecDeque non-empty after removal: overlay remains open.**
If the `VecDeque` is non-empty after the removal in PC-1, `AppMode` remains `Overlay`.
The overlay re-renders with the remaining front-of-queue prompt as the active entry.
The breadcrumb and prompt count badge update to reflect the new stack size.

## Invariants

1. **Retain-all removal.** `overlay_stack.retain(|m| m.prompt_id != prompt_id)` removes
   ALL entries matching `prompt_id` in a single pass — not just the first. In the normal
   case (daemon idempotent insert per BC-2.05.002 Invariant 4) there is at most one match,
   so retain-all and remove-first produce identical results. The retain-all form is required
   for correctness under reconnect races: a `PermissionPromptQueued` duplicate that slips
   through before the dedup helper fires could leave two entries with the same `prompt_id`;
   `retain()` removes both atomically. Using a find-and-remove-first strategy MUST NOT be
   substituted — it would leave a stale duplicate in the VecDeque, blocking the
   empty-stack collapse and leaving the overlay stuck open.
2. **Prompt removal uses UUID-based search-and-remove; empty-stack collapse reuses the BC-2.06.001 logic.**
   The prompt removal itself is performed by the TUI event handler directly: it searches the
   `VecDeque` by `prompt_id`, removes the matching entry. This is NOT an Action dispatch to
   `transition()` — the `transition()` function only handles Action-based transitions. After
   the removal, IF the `VecDeque` is empty AND `AppMode` is `Overlay`, the TUI applies the
   same empty-stack-to-Dashboard collapse defined in BC-2.06.001. The collapse logic is
   reused (not duplicated), but it is triggered after the UUID-based removal step, not as
   part of the `transition()` call chain.
3. **No-op is not an error.** A `PermissionPromptResolved` for an unknown `prompt_id`
   is treated as informational — a routine occurrence in multi-client configurations
   and after reconnection. It MUST NOT trigger UI error notifications.
4. **Multiple TUI clients receive the same broadcast.** Per BC-2.05.005, the daemon
   broadcasts `PermissionPromptResolved` to ALL connected TUI clients. Each client
   removes the prompt independently; PC-3 handles the case where one client already
   resolved the prompt and the second client receives a stale notification.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Multiple TUI clients connected; Client A resolves prompt P1 via user keystroke; daemon sends `PermissionPromptResolved { prompt_id: P1 }` to Client B | Client B removes P1 from its VecDeque (PC-1). If B's stack empties, B collapses to Dashboard (PC-2). P1's Claude Code session is unblocked; Client B's overlay is consistent with daemon state |
| EC-002 | Prompt P1 was cleared by SOQ-3 disconnect (BC-2.06.016); TUI reconnects; daemon sends `PermissionPromptResolved { prompt_id: P1 }` for the old prompt | TUI has no P1 in VecDeque (cleared at disconnect). PC-3 applies: no-op. No error logged. VecDeque state is correct |
| EC-003 | `PermissionPromptResolved` arrives while `AppMode` is `Dashboard` (not `Overlay`) | The VecDeque is empty (Dashboard implies no overlay). PC-3 applies: no matching `prompt_id`. No-op. AppMode remains Dashboard |
| EC-004 | `PermissionPromptResolved` arrives for P2 (second in stack) while P1 is at the front | P2 is removed from the VecDeque interior. P1 remains at the front. Overlay re-renders with P1 still active; badge reflects reduced stack count |
| EC-005 | User resolves P1 at the exact moment the daemon sends `PermissionPromptResolved { P1 }` (race) | User decision path already removed P1 from the VecDeque. `PermissionPromptResolved` arrives and finds no match (PC-3): no-op. No double-removal. No panic. This is the standard multi-source resolution race and is handled idempotently |
| EC-006 | `PermissionPromptResolved` causes the last prompt to be removed from a 1-element VecDeque | PC-1: P1 removed; VecDeque now empty. PC-2: AppMode collapses from `Overlay { prior }` to `Dashboard`. Overlay is gone from screen. Identical behavior to user pressing `y` (accept-once) on the last prompt |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| VecDeque = [P1, P2]; receive `PermissionPromptResolved { prompt_id: P1 }` | VecDeque = [P2]; AppMode remains Overlay; overlay re-renders with P2 at front; badge shows 1 prompt | happy-path |
| VecDeque = [P1]; receive `PermissionPromptResolved { prompt_id: P1 }` | VecDeque = []; AppMode collapses to Dashboard; overlay gone | happy-path (collapse) |
| VecDeque = []; receive `PermissionPromptResolved { prompt_id: P1 }` | No-op; VecDeque still empty; AppMode unchanged | no-op (PC-3) |
| VecDeque = [P1, P2]; receive `PermissionPromptResolved { prompt_id: P2 }` (interior removal) | VecDeque = [P1]; AppMode remains Overlay; P1 at front | edge-case (interior) |
| VecDeque = [P1]; receive `PermissionPromptResolved { prompt_id: P2 }` (unknown ID) | No-op; VecDeque unchanged; AppMode unchanged; no error notification | no-op (PC-3) |
| AppMode = Dashboard; receive `PermissionPromptResolved { prompt_id: P1 }` | No-op; AppMode remains Dashboard | no-op (PC-3) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Matching prompt removed from VecDeque on `PermissionPromptResolved` | unit (inject IPC message; assert VecDeque length decremented; assert prompt_id absent) |
| VP-TBD | AppMode collapses to Dashboard when VecDeque empties after removal | integration (1-element stack; assert AppMode transition to Dashboard) |
| VP-TBD | Unknown prompt_id produces no-op (no state change, no error) | unit (assert VecDeque unchanged; assert no error notification rendered) |
| VP-TBD | Interior VecDeque removal preserves front-of-queue ordering | unit (2-element stack; remove second; assert first still at front) |
| VP-TBD | Multi-client broadcast race: second client processes as no-op without panic | integration (two mock TUI clients; first resolves; second receives PermissionPromptResolved) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability §SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §SS-06 — this BC specifies "permission overlay stack" behavior (VecDeque stack management) and the "AppMode state machine" empty-stack collapse, both of which are named explicitly as CAP-006 responsibilities; `PermissionPromptResolved` handling is the daemon-initiated complement to the user-initiated resolution path already covered by BC-2.06.011/012/013 |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — this BC does not write any files; it only updates in-memory TUI state); DI-001 (every hook event received MUST be written to the JSONL ring — the `PermissionPromptResolved` IPC message is not a hook event; it is a daemon-to-TUI state synchronization message; DI-001 is not applicable here) |
| Architecture Module | monocle-tui (VecDeque state update, AppMode transition) per ARCH-INDEX Subsystem Registry SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Permission Overlay §Overlay Stack Lifecycle; SS-ipc.md v1.11.0 §ServerToClient::PermissionPromptResolved (lines 288-289 debounce; message type definition) |
| Cross-Ref | BC-2.05.005 (daemon broadcasts PermissionPromptResolved — the IPC message this BC handles at the TUI layer); BC-2.06.001 (AppMode state machine empty-stack collapse — PC-2 reuses this invariant); BC-2.06.008 (overlay push on PermissionPromptQueued — this BC is the inverse: removal on resolution); BC-2.06.016 (disconnect clear — SOQ-3 is the other removal mechanism; EC-002 of this BC handles the post-reconnect no-op case) |
| Test File | `monocle-tui/tests/permission_overlay_resolved.rs` |
| Test Name | `test_BC_2_06_023_tui_removes_resolved_prompt_on_permission_prompt_resolved` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.005] — depends on: daemon broadcasts `PermissionPromptResolved` via IPC (the source of the message this BC handles)
- [BC-2.06.001] — depends on: AppMode empty-stack collapse at PC-2 reuses BC-2.06.001 transition() invariant
- [BC-2.06.008] — inverse of: BC-2.06.008 pushes prompts onto the VecDeque; this BC removes them via daemon-initiated resolution
- [BC-2.06.016] — related: SOQ-3 disconnect clears the entire VecDeque; EC-002 of this BC handles the no-op case after reconnection when `PermissionPromptResolved` arrives for a cleared prompt

## Architecture Anchors

- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle: push (BC-2.06.008), decide (BC-2.06.011/012/013), daemon-resolve (this BC), disconnect-clear (BC-2.06.016)
- `architecture/SS-ipc.md#servertoclient-permissionpromptresolved` — message type definition and broadcast semantics

## Story Anchor

S-TBD — Implement TUI handler for `PermissionPromptResolved` IPC message (filled by story-writer)

## VP Anchors

VP-TBD — VecDeque removal and AppMode collapse integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- BC-2.06.023 created as new artifact for SS-06 per F-P1D-006 finding from Phase 1d Pass 1
  adversarial review. The adversary identified a gap: no BC specified TUI-side handling of
  `ServerToClient::PermissionPromptResolved`.
- This BC closes the gap between BC-2.05.005 (daemon broadcasts the message) and the TUI's
  required response (remove matching PromptModal from VecDeque, collapse AppMode if empty).
- Covers: matching removal (PC-1), empty-stack collapse (PC-2), no-op for unknown IDs
  (PC-3), non-empty stack after removal (PC-4), 4 invariants, 6 edge cases, 6 test vectors,
  5 verification properties.
- EC-001 documents the multi-client broadcast case where Client B receives a resolution
  triggered by Client A — the primary motivation for this BC's existence.
- EC-002 documents the post-reconnect no-op case (SOQ-3 interaction).
- EC-005 documents the user-decision/daemon-broadcast race — handled idempotently via PC-3.
- Capability anchor: CAP-006 per ARCH-INDEX §SS-06 Capability Traceability ("permission
  overlay" and "AppMode state machine" named explicitly in CAP-006 statement).
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact).


## §Trace v1.1.0

**F-P1D3-007 HIGH — Invariant 2 corrected: removal mechanism is UUID-search-and-remove, not transition()** (2026-05-26T14:00:00Z):
- Invariant 2 rewrote the incorrect claim that `transition()` handles the removal. The correct
  mechanism is: TUI event handler performs UUID-based search-and-remove on the VecDeque directly
  (not via Action dispatch); the empty-stack-to-Dashboard collapse thereafter reuses BC-2.06.001
  logic, but the removal itself bypasses `transition()`. Added explicit distinction between the
  removal step (UUID search-and-remove) and the post-removal collapse step (reused BC-2.06.001
  empty-stack logic).

**O-P1D3-004 — L2 Capability text updated to ARCH-INDEX CAP-006 verbatim** (2026-05-26T14:00:00Z):
- Traceability §L2 Capability: old text "ratatui TUI; AppMode state machine; keybinding dispatch;
  permission overlay; sessions panel; event ribbon; status bar" replaced with verbatim ARCH-INDEX
  row: "User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon;
  permission overlay stack; Ctrl-\ popup integration".
- Capability Anchor Justification updated to match verbatim title; text "permission overlay" →
  "permission overlay stack" to align with the CAP-006 canonical statement.
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-26T14:00:00Z > v1.0.1. PASS.

## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.2.0

**F-P1D4-004/005 LOW — Architecture Source pins updated from v1.1.0 to current versions** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- Architecture Source: `SS-ipc.md v1.1.0` → `SS-ipc.md v1.3.0` per F-P1D4-004 bulk update.
- SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.4.0

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack, prior }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. PC-2: `Overlay { stack, prior }` → `Overlay { prior }` with `App.overlay_stack` as the emptiness check. The VecDeque is `App.overlay_stack` (App-level single source of truth), not a field of the `AppMode::Overlay` variant. Note: the remaining `VecDeque<PromptModal>` references in this BC correctly refer to `App.overlay_stack` as a data structure — they are not the stale variant shape.
- EC-006: stale keystroke `1` → `y` (accept-once canonical keybinding per ADJ-ADV2-001).
- SE-16d monotonicity: v1.4.0 timestamp 2026-05-28T00:00:00Z > v1.3.0. PASS.

## §Trace v1.4.1

**ADV23-SCOPE-002 — Architecture Source dual pin updated: SS-tui.md v1.5.0 → v1.8.2 + SS-ipc.md v1.4.0 → v1.9.0** (2026-05-29T00:00:00Z):
- Architecture Source row updated with two pin bumps:
  - `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
  - `SS-ipc.md v1.4.0` → `SS-ipc.md v1.9.0` per the same ADV23-SCOPE-001 sweep (ss-05 refreshes in cc1ea7d).
- Classification: Category A plain version-pin refresh for both citations. No substantive content changes required:
  - SS-tui.md v1.8.0 (Overlay shape): already propagated in §Trace v1.4.0 above.
  - SS-tui.md v1.8.1 (Sessions Panel 6→7 columns): this BC covers IPC-driven prompt removal; no Sessions Panel column table in scope.
  - SS-tui.md v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
  - SS-ipc.md v1.9.0: the `PermissionPromptResolved` message type definition and debounce behavior are unchanged between v1.4.0 and v1.9.0; only other features added. §Section anchor unchanged.
- SE-16d monotonicity: v1.4.1 timestamp 2026-05-29T00:00:00Z > v1.4.0. PASS.

## §Trace v1.5.0

**F-S026-ADV1-LOW-001 — Invariant 1 corrected to retain-all semantics** (2026-05-31T00:00:00Z):
- Finding: BC-2.06.023 Invariant 1 stated "only the first matching entry is removed" and
  "MUST NOT scan and remove all matches." This directly contradicted:
  - PC-1 (removes the matching `PromptModal` — no first-only restriction in postconditions)
  - AC-006 of S-026 (`retain()` removes ALL entries matching `prompt_id`)
  - The implementation (`overlay_stack.retain(|m| m.prompt_id != prompt_id)`)
  - The rationale for retain-all: reconnect-race defense requires removing all duplicates atomically.
- Fix: Invariant 1 rewritten to specify `retain()` (remove-all) semantics. The invariant now
  explains WHY retain-all is required (reconnect race), names the forbidden alternative
  (find-and-remove-first), and states the failure mode if the wrong strategy is used (stale
  duplicate leaves overlay stuck open).
- No other sections changed. PC-1/PC-2/PC-3/PC-4, edge cases, and test vectors are consistent
  with retain-all and required no edits.
- SE-16d monotonicity: v1.5.0 timestamp 2026-05-31T00:00:00Z > v1.4.1 (2026-05-29). PASS.

## §Trace v1.3.0

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0`; `SS-ipc.md v1.3.0` → `SS-ipc.md v1.4.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.3.0 timestamp >= v1.2.0. PASS.
