---
document_type: behavioral-contract
level: L3
version: "1.0.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-31T12:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "ee4d690"
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

# Behavioral Contract BC-2.06.021: Status Bar: Keybinding Hint Line

## Description

The status bar's bottom row (the lower of the two-row status bar) renders a
context-sensitive one-line summary of the available keybinding actions for the current
`AppMode`. The hint line changes on every `AppMode` transition and is derived from the
`Builtin` binding table for that mode. It is always visible — even in Fullscreen and
Overlay modes. The purpose is discoverability: users should never need to consult external
documentation to know what keys are available in the current mode.

## Preconditions

1. The TUI is in any valid `AppMode`.
2. The status bar is allocated the bottom 2 rows of the terminal. The hint line occupies
   the last (lower) row.
3. The `Builtin` binding table in the `Dispatcher` defines the canonical key hints for
   each mode.

## Postconditions

1. **Hint line per AppMode:**

   | AppMode | Hint Line Text |
   |---------|---------------|
   | `Dashboard` (any focus) | `Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit` |
   | `Overlay` | `y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace` |
   | `Filtering` | `(type to filter)  Esc: cancel` |
   | `Fullscreen` | `Esc: back  /: filter  q: quit` |

2. **Context-sensitivity:** The hint line is derived from the current `AppMode` only.
   It does NOT reflect custom user bindings (UserCustomCommand or PerContext overrides).
   The hint line shows `Builtin` bindings, which are always available as fallback keys.
   This is intentional: the hint line is a discovery aid for first-time users, not a
   complete binding documentation.

3. **Always rendered:** The hint line renders on every `draw()` tick. There is no AppMode
   in which the hint line is hidden or replaced.

4. **No truncation at 80 columns:** All hint strings from Postcondition 1 must fit on a
   single line at 80 columns. The longest is the Overlay hint:
   `y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace`
   (72 display columns, counting ↑↓ as 2 columns each). This fits within 80 columns.

5. **`Esc: hide` semantics in Overlay hint:** The Overlay hint shows `Esc: hide` — not
   `Esc: dismiss`. This communicates that `Esc` hides the overlay (via `Ctrl-\` tmux
   dismiss) without making a decision. Combined with BC-2.06.014's behavior (Esc is a
   no-op on the stack), this text guides the user to the correct mental model: prompts
   are not rejected by hiding.

6. **`t: trace` in Overlay hint:** The `[t]` keybinding appears in the hint line for
   `Overlay` mode even though it is a Phase 1 stub (BC-2.06.015). The hint is present
   so the keybinding is discoverable. The action renders a placeholder message, not
   actual trace navigation.

## Invariants

1. The hint line is a pure function of `AppMode`. Given the same mode, the same hint
   string is always rendered.
2. The hint line reflects ONLY `Builtin` bindings. If a user has remapped `q` to quit
   via `UserCustomCommand`, the hint still shows `q: quit` — the builtin default. This
   prevents the hint from showing incorrect keys for other users' configurations.
3. At 80 columns, no hint line from Postcondition 1 exceeds 79 display columns (leaving 1
   column margin). The current longest is the Overlay hint at 72 display columns. New hint
   entries must be checked against this constraint before adding.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-129 | Terminal is 79 columns wide | Hint line text (≤79 chars) renders on a single line; no truncation at 79 columns for any hint in Postcondition 1 |
| EC-130 | Terminal is 60 columns wide | Hint line is truncated at the right edge; no line wrap; no panic; the most important keys (first in the string) are preserved |
| EC-131 | AppMode transitions from Overlay to Dashboard after decision | Hint line changes from Overlay hint to Dashboard hint on the next draw tick |
| EC-132 | `Filtering` mode is active; user types a character | Hint continues to show `(type to filter)  Esc: cancel`; the character input does not change the hint |
| EC-133 | Phase 2 AppMode variant (e.g., `Customizations`) is encountered (future) | Returns a "not implemented" hint or empty string; no panic; Phase 1 implementation can safely default to `""` for unknown variants |

## Canonical Test Vectors

| AppMode | Expected Hint Line | Category |
|---------|-------------------|----------|
| `Dashboard { focused: Sessions }` | `Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit` | happy-path |
| `Dashboard { focused: EventRibbon }` | `Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit` | happy-path |
| `Overlay { prior: Sessions }` (with `App.overlay_stack = [P1]`) | `y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace` | happy-path |
| `Fullscreen { panel: Sessions, prior: Sessions }` | `Esc: back  /: filter  q: quit` | happy-path |
| `Filtering { panel: Sessions, query: "", prior: Sessions }` | `(type to filter)  Esc: cancel` | happy-path |
| AppMode transition: Overlay → Dashboard (decision made) | Hint changes from `y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace` to Dashboard hint on next draw | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Hint line is a pure function of `AppMode` (snapshot test) | unit test |
| VP-TBD | All hint strings fit in 80 columns | unit test (assert len <= 79 for each table entry) |
| VP-TBD | Hint changes immediately on AppMode transition (next draw) | unit test |
| VP-TBD | Hint renders in all AppModes without panic | unit test (enumerate all Phase 1 variants) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the keybinding hint line which is the discoverable surface for the "keybinding dispatch" component of CAP-006 |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — satisfied: hint line is read-only display derived from in-memory AppMode) |
| Architecture Module | monocle-tui (status bar renderer `draw_status_bar()`, hint line derivation from `AppMode`); monocle-core (`AppMode`, `Dispatcher::builtin` binding table) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Panel Architecture §Status Bar (keybinding hint line subsection with all 4 AppMode hint examples) |
| Cross-Ref | BC-2.06.001 (AppMode state machine — hint is derived from it); BC-2.06.003 (5-level binding precedence — hint shows Builtin level only); BC-2.06.020 (breadcrumb — shares the status bar, occupies the upper row); BC-2.06.014 (Esc in Overlay — hint says `Esc: hide` to match the no-op behavior); BC-2.06.015 (`[t]` trace stub — hint shows `t: trace` as a discoverable stub) |
| Test File | `monocle-tui/tests/status_bar.rs` |
| Test Name | `test_BC_2_06_021_keybinding_hint_line` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: hint is a pure function of `AppMode` defined in BC-2.06.001
- [BC-2.06.003] — composes with: hint shows Builtin binding level; dispatcher walks all 5 levels at runtime
- [BC-2.06.020] — composes with: breadcrumb occupies upper status bar row; hint occupies lower row
- [BC-2.06.014] — semantic coupling: `Esc: hide` in the hint matches the no-op Esc behavior specified in BC-2.06.014

## Architecture Anchors

- `architecture/SS-tui.md#status-bar` — keybinding hint line subsection (all 4 AppMode examples)

## Story Anchor

S-TBD — Implement status bar keybinding hint line: context-sensitive, pure AppMode derivation, 80-column fit (filled by story-writer)

## VP Anchors

- VP-TBD — Snapshot test: hint string for every Phase 1 AppMode variant; all within 79 characters

## §Trace v1.0.0

**Initial production** (2026-05-26T18:00:00Z):
- BC-2.06.021 created as part of SS-06 TUI behavioral contract burst (BCs 016–022).
- Reads: SS-tui.md v1.1.0 §Panel Architecture §Status Bar (hint line subsection with 4
  AppMode examples); prd-expansion-scope.md §3.3 BC-2.06.021 description (F-52).
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: read-only display derivation.
- Postcondition 2 explicitly limits hint to Builtin bindings to prevent per-user
  configuration leaking into what is meant to be a universal discovery aid.
- Postcondition 5 documents the `Esc: hide` semantic coupling with BC-2.06.014 — the
  hint must match the behavior or it actively misleads the user.
- Postcondition 6 documents `t: trace` appearing in the hint even as a stub — prevents
  a story implementer from omitting the hint entry for unimplemented Phase 2 features.
- Invariant 3 adds a forward-compatibility constraint: new hint entries must fit in 80
  columns before being added, preventing silent overflow in future phases.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-S025-ADV4-HIGH-001 — Test vector row 3 body sweep completion** (2026-05-28T13:00:00Z):
- Finding: The Architect Pass 2 HIGH-003 sweep (commit `6d4fbb3`, §Trace v1.24 of BC-INDEX)
  updated 16 BCs for the `AppMode::Overlay { stack }` shape removal but did NOT include
  BC-2.06.021. No §Trace entry was added at that time. Canonical test vector row 3
  (AppMode column) still contained the stale `Overlay { stack: [P1], prior: Sessions }` shape.
- Fix — Canonical test vector row 3 (AppMode column):
  `Overlay { stack: [P1], prior: Sessions }` →
  `Overlay { prior: Sessions }` (with `App.overlay_stack = [P1]`).
  The parenthetical makes the stack content visible in the test vector without embedding
  it in the `AppMode` shape, consistent with the canonical two-step semantics.
- No other body content in BC-2.06.021 contained stale `Overlay { stack }` shape —
  all other references use `Overlay` without stack fields (hint line content is
  mode-agnostic w.r.t. stack shape).
- SE-16d monotonicity: v1.0.4 timestamp 2026-05-28T13:00:00Z > v1.0.3 timestamp 2026-05-26T00:00:00Z. PASS.

## §Trace v1.0.6

**ADV-S027-BC021-001 HIGH — Overlay hint keys corrected from stale `1/2/3` to canonical `y/A/n/r`** (2026-05-31T12:00:00Z):
- Finding: BC-2.06.021 PC-1 Overlay hint row contained `1: accept-once  2: accept-always  3: reject`
  which contradicts PC-3 (stating the Builtin binding table is canonical) and directly contradicts
  the merged Builtin table in `monocle-core/src/tui/binding.rs` lines ~250-254 (shipped in S-026 /
  PR #30). The canonical Builtin bindings in `binding.rs` are:
    `'y'` or `Enter` → `PermissionAcceptOnce`
    `'A'`            → `PermissionAcceptAlways`
    `'n'` or `'r'`   → `PermissionReject`
- Root cause: PC-1 was written before the keybinding table was canonicalized in S-026. The
  `1/2/3` keys were a design placeholder that was superseded by `y/A/n/r` (aligned with
  standard accept/reject UX conventions). The `binding.rs` implementation is the authoritative
  source of truth per CLAUDE.md "merged, tested code wins".
- Changes:
  - PC-1 table Overlay row: `1: accept-once  2: accept-always  3: reject  ↑↓: cycle  Esc: hide  t: trace`
    → `y: accept  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide  t: trace`
  - PC-4: Updated explicit hint string example and column count (75 display cols → 72 display
    cols; counting ↑↓ as 2 display columns each per Unicode east-asian-width rules).
  - INV-3: Updated column-count description from "79 characters" to "79 display columns" (more
    precise) and replaced the implicit claim with the explicit current maximum (72).
  - Test vectors: Overlay happy-path row and Overlay→Dashboard transition row updated to
    reflect the corrected hint string.
- No changes to PC-2 (context-sensitivity), PC-3 (Builtin-only derivation), PC-5 (`Esc: hide`
  semantics), PC-6 (`t: trace` stub), EC table, VP table, or Traceability section.
- Consistency note: BC-2.06.022 (§Trace v1.0.5) already contained the correct hint text
  (`y: accept-once  A: accept-always  n/r: reject`); BC-2.06.021 was the only remaining BC
  with the stale `1/2/3` keys.
- SE-16d monotonicity: v1.0.6 timestamp 2026-05-31T12:00:00Z > v1.0.5 timestamp 2026-05-29T00:00:00Z. PASS.

## §Trace v1.0.5

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): already propagated in §Trace v1.0.4 above (test vector row 3 Overlay shape updated; hint line content is mode-agnostic w.r.t. stack shape).
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers keybinding hint line; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC (hint line shows mode-specific keybindings, not daemon connection status).
- SE-16d monotonicity: v1.0.5 timestamp 2026-05-29T00:00:00Z > v1.0.4. PASS.
