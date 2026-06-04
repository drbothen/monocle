---
document_type: behavioral-contract
level: L3
version: "1.1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-01T14:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "bfb91e5"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010, S-027-ADV-DROPS-COEXISTENCE]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.020: Status Bar: Breadcrumb

## Description

The status bar's second row (top of the two-row status bar) renders a breadcrumb string
derived from the current `AppMode`. The breadcrumb provides orientation: it shows the
user's current location in the navigation hierarchy at a glance, without requiring them to
count open panels or remember how they navigated. The string is computed deterministically
from `AppMode` and rendered on every `draw()` tick. The status bar is always visible —
even in Fullscreen and Overlay modes.

## Preconditions

1. The TUI is in any valid `AppMode`.
2. The status bar is allocated the bottom 2 rows of the terminal via the ratatui layout
   constraint (`Constraint::Length(2)` per SS-tui.md §Rendering Architecture §Draw Loop).
3. The breadcrumb occupies the first (upper) of the two status bar rows, alongside the
   drop counter (BC-2.06.019) on the same row.

## Postconditions

1. **Breadcrumb derivation table:** The breadcrumb string is derived from `AppMode` as
   follows:

   | AppMode | Breadcrumb String |
   |---------|------------------|
   | `Dashboard { focused: Sessions }` | `Dashboard > Sessions` |
   | `Dashboard { focused: EventRibbon }` | `Dashboard > Events` |
   | `Overlay { prior }` where `App.overlay_stack.len() == 1` | `Dashboard > Overlay [1 prompt]` |
   | `Overlay { prior }` where `App.overlay_stack.len() == N (N > 1)` | `Dashboard > Overlay [N prompts]` |
   | `Fullscreen { panel: Sessions, .. }` | `Dashboard > Sessions > Fullscreen` |
   | `Fullscreen { panel: EventRibbon, .. }` | `Dashboard > Events > Fullscreen` |
   | `Filtering { panel: Sessions, .. }` | `Dashboard > Sessions > Filter` |
   | `Filtering { panel: EventRibbon, .. }` | `Dashboard > Events > Filter` |

2. **Grammatical plurality:** When `App.overlay_stack.len() == 1`, render `[1 prompt]` (singular).
   When `App.overlay_stack.len() > 1`, render `[N prompts]` (plural). This distinction is load-bearing:
   it communicates to the user exactly how many decisions are queued.

3. **Always rendered:** The breadcrumb renders on every `draw()` tick regardless of
   `AppMode`. There is no mode in which the breadcrumb is hidden or replaced with a
   different widget.

4. **No truncation on minimum terminal width:** At 80 columns, all breadcrumb strings
   from the derivation table fit without truncation. The longest string is
   `Dashboard > Sessions > Fullscreen` (34 characters) or
   `Dashboard > Overlay [99 prompts]` (32 characters for 99 queued prompts).
   Both fit within 80 columns alongside the drop counter text.

5. **Left-aligned:** The breadcrumb is left-aligned on the upper (breadcrumb) row of the
   two-row status bar. The drop counter (`drops: N`) is right-aligned on the same upper
   row when non-zero (BC-2.06.019 PC-7). Transient `status_message` notifications
   (disconnect indicator, `[t]` stub, etc.) render on the lower (hint) row — they do
   NOT displace the breadcrumb or the drop counter from the upper row.

## Invariants

1. The breadcrumb is a pure function of `AppMode`. Given the same `AppMode`, the same
   breadcrumb string is always produced. There is no state beyond `AppMode` that affects
   the breadcrumb.
2. Phase 2 will add `Customizations` and `Workflow` panel focus states. The breadcrumb
   derivation table will be extended at that point. Phase 1 only includes `Sessions` and
   `EventRibbon` focus variants.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-125 | `Overlay` mode with `App.overlay_stack.len() == 0` (unreachable per BC-2.06.001 empty-stack invariant) | If reached in testing, renders `Dashboard > Overlay [0 prompts]` — no panic; the impossible state is displayed faithfully |
| EC-126 | Terminal resize reduces width to <34 columns | Breadcrumb is truncated at the right edge; no line wrap; no panic; truncation is the lesser evil compared to wrap breaking column layout |
| EC-127 | Very large overlay stack: 99 prompts | Renders `Dashboard > Overlay [99 prompts]`; fits in 80 columns |
| EC-128 | AppMode transitions mid-frame (race between draw and IPC message handling) | The draw loop drains IPC messages before drawing; breadcrumb reflects the `AppMode` after IPC drain; no partial-state render |

## Canonical Test Vectors

| AppMode | Expected Breadcrumb | Category |
|---------|-------------------|----------|
| `Dashboard { focused: Sessions }` | `Dashboard > Sessions` | happy-path |
| `Dashboard { focused: EventRibbon }` | `Dashboard > Events` | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) | `Dashboard > Overlay [1 prompt]` | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2, P3]) | `Dashboard > Overlay [3 prompts]` | happy-path |
| `Fullscreen { panel: Sessions, prior: Sessions }` | `Dashboard > Sessions > Fullscreen` | happy-path |
| `Filtering { panel: Sessions, query: "foo", prior: Sessions }` | `Dashboard > Sessions > Filter` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Breadcrumb is a pure function of `AppMode` (same mode → same string) | unit test (snapshot test for all table entries) |
| VP-TBD | Overlay breadcrumb uses singular `[1 prompt]` and plural `[N prompts]` correctly | unit test |
| VP-TBD | Breadcrumb renders in all AppModes without panic | unit test (enumerate all variants) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the status bar breadcrumb which is the primary orientation aid for the "AppMode state machine" component of CAP-006 |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — satisfied: breadcrumb is read-only display derived from in-memory AppMode) |
| Architecture Module | monocle-tui (status bar renderer `draw_status_bar()`, breadcrumb derivation from `AppMode`); monocle-core (`AppMode` enum) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Panel Architecture §Status Bar (breadcrumb subsection with all 4 AppMode derivation examples) |
| Cross-Ref | BC-2.06.001 (AppMode state machine — breadcrumb is derived from it); BC-2.06.019 (drop counter — shares the same status bar row); BC-2.06.021 (keybinding hint — occupies the second status bar row) |
| Test File | `monocle-tui/tests/status_bar.rs` |
| Test Name | `test_BC_2_06_020_breadcrumb_derivation` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: breadcrumb is a pure function of `AppMode` defined in BC-2.06.001
- [BC-2.06.019] — composes with: drop counter shares the breadcrumb row in the status bar
- [BC-2.06.021] — composes with: keybinding hint occupies the second status bar row (below breadcrumb)

## Architecture Anchors

- `architecture/SS-tui.md#status-bar` — breadcrumb subsection (derivation table and layout)

## Story Anchor

S-TBD — Implement status bar breadcrumb: pure derivation from AppMode, singular/plural prompt count (filled by story-writer)

## VP Anchors

- VP-TBD — Snapshot test: breadcrumb string for every AppMode variant

## §Trace v1.0.0

**Initial production** (2026-05-26T18:00:00Z):
- BC-2.06.020 created as part of SS-06 TUI behavioral contract burst (BCs 016–022).
- Reads: SS-tui.md v1.1.0 §Panel Architecture §Status Bar (breadcrumb derivation table);
  prd-expansion-scope.md §3.3 BC-2.06.020 description (F-51).
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: read-only display derivation.
- Postcondition 2 specifies singular/plural grammar for the prompt count — prevents a
  common "0 prompts / 1 prompts" localization error.
- Invariant 2 explicitly documents Phase 2 scope for Customizations/Workflow focus states.
- EC-125 handles the unreachable empty-stack Overlay state gracefully rather than
  panicking — defense-in-depth for test isolation.


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

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. Breadcrumb derivation table: `Overlay { stack, prior }` → `Overlay { prior }` with `App.overlay_stack.len()` as the prompt count source. Postcondition 2, EC-125, and test vectors updated.
- SE-16d monotonicity: v1.0.4 timestamp 2026-05-28T00:00:00Z > v1.0.3. PASS.

## §Trace v1.0.5

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): already propagated in §Trace v1.0.4 above (`Overlay { stack, prior }` → `Overlay { prior }` with `App.overlay_stack.len()`).
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers breadcrumb rendering; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC (breadcrumb shows current AppMode, not daemon-disconnect text).
- SE-16d monotonicity: v1.0.5 timestamp 2026-05-29T00:00:00Z > v1.0.4. PASS.

## §Trace v1.1.0

**S-027-ADV-DROPS-COEXISTENCE LOW — PC-5 layout language aligned with coexistence rule** (2026-06-01T14:00:00Z):

BC-2.06.019 v1.1.0 established the canonical coexistence layout: `drops: N` is permanent
on the upper row; `status_message` renders on the lower row. PC-5 of this BC was updated
to use consistent "upper row" / "lower row" terminology and to note explicitly that
transient notifications do NOT displace breadcrumb or drop counter from the upper row.
No other changes. This is a terminology alignment, not a behavior change.
- SE-16d monotonicity: v1.1.0 timestamp 2026-06-01T14:00:00Z > v1.0.5. PASS.
