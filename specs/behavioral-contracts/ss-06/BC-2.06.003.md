---
document_type: behavioral-contract
level: L3
version: "1.1.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "c1e8267"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010, ADJ-ADV2-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.003: Action Dispatch: 5-Level Binding Precedence

## Description

The keybinding dispatcher in `monocle-tui::keybinding::Dispatcher` holds five
`HashMap<KeyEvent, Action>` tables — one per `BindingSource` level — and resolves a raw
crossterm `KeyEvent` to a `Binding` by walking the levels in strict priority order:
`SearchPrompt > UserCustomCommand > PerContext > Global > Builtin`. The dispatcher stops
at the first level that contains a matching entry. If no level matches, it returns `None`
and the keypress is discarded. The dispatch is deterministic: given the same `(KeyEvent,
AppMode)`, the `Dispatcher::resolve()` call always produces the same `Option<Binding>`.

## Preconditions

1. `BindingSource` is defined in `monocle-core/src/binding.rs` with five variants:
   `SearchPrompt`, `UserCustomCommand`, `PerContext`, `Global`, `Builtin`.
2. `Binding` is defined in `monocle-core/src/binding.rs` as `{ action: Action, source: BindingSource }`.
3. `Dispatcher` in `monocle-tui::keybinding` holds five `HashMap<KeyEvent, Action>` fields:
   `search_prompt`, `user_custom`, `per_context`, `global`, `builtin`.
4. `Dispatcher::resolve(key: KeyEvent, mode: &AppMode) -> Option<Binding>` iterates the
   five tables in fixed order and returns the first match.
5. The `per_context` table is rebuilt by `Dispatcher::update_context(mode: &AppMode)` on
   every `AppMode` change. Rebuilding replaces the previous `per_context` table; it does
   not merge or accumulate.
6. The `builtin` table is populated at `Dispatcher` construction time from compiled-in
   defaults and is never mutated at runtime.
7. The `user_custom` table is populated from `monocle-config` binding overrides at startup
   and on `Action::ConfigReload`.

## Postconditions

1. **First-match-wins, strict priority:** For any `KeyEvent` that appears in multiple
   binding levels, the resolved `BindingSource` is always the highest-priority level that
   contains the key. A key bound in `Builtin` that is also bound in `UserCustomCommand`
   resolves to `UserCustomCommand`.
2. **SearchPrompt captures printable keys:** When `AppMode` is `Filtering`, the
   `search_prompt` table contains bindings for all printable characters (mapped to
   `Action::FilterType(char)`) and for `Escape` (mapped to `Action::Escape`). This ensures
   that printable keystrokes during filter mode are captured before any lower-level table
   can intercept them.
3. **SearchPrompt captures overlay decision keys:** In `Overlay` mode, the `search_prompt`
   table (highest priority) contains the permission decision bindings:
   - `KeyCode::Char('y')` → `Action::PermissionAcceptOnce`
   - `KeyCode::Enter` → `Action::PermissionAcceptOnce`
   - `KeyCode::Char('A')` → `Action::PermissionAcceptAlways`
   - `KeyCode::Char('n')` → `Action::PermissionReject`
   - `KeyCode::Char('r')` → `Action::PermissionReject`
   - `KeyCode::Char('t')` → `Action::PermissionTraceToSource`
   These bindings reside in the `SearchPrompt` layer (not `PerContext`) because overlay
   decision keys must take highest priority, overriding any `PerContext`, `Global`, or
   `Builtin` bindings that might otherwise intercept `y`, `Enter`, `A`, `n`, or `r` during
   overlay mode. They are NOT present in `search_prompt` when `AppMode` is `Dashboard` or
   `Fullscreen`, preventing accidental permission decisions outside overlay mode.
4. **No-match returns `None`:** If a `KeyEvent` is not present in any of the five tables
   at the time of resolution, `resolve()` returns `None`. The TUI draw loop discards the
   keypress with no error and no state change.
5. **Determinism:** Given the same `Dispatcher` state and the same `(KeyEvent, AppMode)`,
   `resolve()` always returns the same `Option<Binding>`. No randomness, no I/O, no shared
   mutable state is read during resolution.
6. **`update_context` replaces `per_context` atomically:** After `update_context(mode)` is
   called, `per_context` contains exactly the bindings for the new `mode`. No stale entries
   from the previous `AppMode` remain.

## Invariants

1. The five-table walk order is immutable and cannot be configured. Users may only influence
   resolution by populating the `UserCustomCommand` or `PerContext` tables — they cannot
   reorder the precedence levels.
2. The `Builtin` table always contains at minimum the following bindings (regardless of
   `AppMode`): `Tab` → `CyclePanel`, `Enter` → `Enter`, `Escape` → `Escape`,
   `Char('/')` → `FilterStart`, `Char('q')` → `Quit`, `Ctrl-P` → `ProfilePicker`.
3. A `UserCustomCommand` binding that conflicts with a `Builtin` binding always wins (by
   the first-match-wins rule). This is the intended user customization model.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-070 | `Char('y')` pressed in `Dashboard` mode | `search_prompt` table for Dashboard has no `Char('y')` overlay entry; `per_context`, `user_custom`, `global`, `builtin` are checked; if none match, returns `None` |
| EC-071 | `Char('y')` or `Enter` pressed in `Overlay` mode | `search_prompt` contains `Char('y')` and `Enter` → `PermissionAcceptOnce`; resolved at highest priority before reaching any lower-level table; returns `Binding { action: PermissionAcceptOnce, source: SearchPrompt }` |
| EC-072 | User binds `Tab` to a custom action in `user_custom` table | `Tab` → custom action from `user_custom` wins over `Tab` → `CyclePanel` from `builtin` |
| EC-073 | `update_context()` called twice in rapid succession (e.g., two AppMode transitions in one tick) | Second call replaces the `per_context` table set by the first; no stale entries; final state is correct for the last `AppMode` passed |
| EC-074 | `Dispatcher` constructed with empty `user_custom` and `per_context` tables | `resolve()` falls through to `global` and `builtin`; builtin defaults are always present |
| EC-075 | Modifier key combination (e.g., `Ctrl-P`) pressed in `Filtering` mode | `search_prompt` is checked first; `Ctrl-P` is not a printable character and is not in `search_prompt`; falls through to `global` or `builtin` where `Ctrl-P` → `ProfilePicker` is registered |

## Canonical Test Vectors

| Key | AppMode | Expected Action | Expected Source | Category |
|-----|---------|----------------|-----------------|----------|
| `Tab` | `Dashboard { focused: Sessions }` | `CyclePanel` | `Builtin` | happy-path |
| `Char('/')` | `Dashboard { focused: Sessions }` | `FilterStart` | `Builtin` | happy-path |
| `Char('a')` | `Filtering { .. }` | `FilterType('a')` | `SearchPrompt` | happy-path |
| `Char('y')` | `Overlay { .. }` | `PermissionAcceptOnce` | `SearchPrompt` | happy-path |
| `Enter` | `Overlay { .. }` | `PermissionAcceptOnce` | `SearchPrompt` | happy-path |
| `Char('A')` | `Overlay { .. }` | `PermissionAcceptAlways` | `SearchPrompt` | happy-path |
| `Char('n')` | `Overlay { .. }` | `PermissionReject` | `SearchPrompt` | happy-path |
| `Char('y')` | `Dashboard { .. }` | `None` (no match) | N/A | edge-case |
| `Char('q')` | `Dashboard { .. }` | `Quit` | `Builtin` | happy-path |
| `Ctrl-P` | `Dashboard { .. }` | `ProfilePicker` | `Builtin` | happy-path |
| Unknown key (e.g., `F12`) | Any | `None` | N/A | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `Char('y')` and `Enter` resolve to `PermissionAcceptOnce` (source: `SearchPrompt`) in Overlay and `None` in Dashboard | Unit test (table-driven) |
| VP-TBD | `update_context()` removes all prior per_context bindings before installing new ones | Unit test |
| VP-TBD | For any key in `user_custom`, `resolve()` returns `UserCustomCommand` source regardless of `builtin` contents | Unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the "keybinding dispatch" component of CAP-006: the 5-level precedence system is the mechanism by which user customizations interact safely with builtin bindings and AppMode-specific context bindings |
| L2 Domain Invariants | DI-006 (EngineModule statelessness — orthogonally supported: `Dispatcher::resolve()` is stateless beyond the pre-populated tables; no global mutable state is read during resolution) |
| Architecture Module | monocle-core (BindingSource, Binding enums); monocle-tui (Dispatcher, per_context rebuild) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Action Enum and 5-Level Binding Precedence (Dispatcher Logic sketch, BindingSource enum) |
| Cross-Ref | BC-2.06.001 (Action enum definition), BC-2.06.004 (Ctrl-\ binding is external to dispatcher — in tmux, not monocle), BC-2.06.011 (accept-once `[y]`/`[Enter]` bindings are in SearchPrompt layer per this dispatch contract), BC-2.06.012 (accept-always `[A]` binding — SearchPrompt layer), BC-2.06.013 (reject `[n]`/`[r]` bindings — SearchPrompt layer) |
| Test File | `monocle-tui/tests/keybinding_dispatcher.rs` |
| Test Name | `test_BC_2_06_003_five_level_binding_precedence` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: `Action` enum is defined in monocle-core (same crate as BindingSource/Binding)
- [BC-2.06.011] — composes with: `[y]`/`[Enter]` accept-once bindings are registered in SearchPrompt (highest priority) for Overlay mode per this dispatch contract
- [BC-2.06.012] — composes with: `[A]` accept-always binding is registered in SearchPrompt for Overlay mode
- [BC-2.06.013] — composes with: `[n]`/`[r]` reject bindings are registered in SearchPrompt for Overlay mode

## Architecture Anchors

- `architecture/SS-tui.md#action-enum-and-5-level-binding-precedence` — Dispatcher struct sketch and resolve() algorithm
- `architecture/SS-tui.md#action-enum-and-5-level-binding-precedence` — BindingSource enum definition

## Story Anchor

S-TBD — Implement Dispatcher with 5-level HashMap tables; update_context() rebuilds per_context on AppMode change (filled by story-writer)

## VP Anchors

- VP-TBD — Table-driven unit tests for all PerContext bindings per AppMode variant

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.003 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.1.0 §Action Enum and 5-Level Binding Precedence (Dispatcher struct
  sketch, BindingSource enum, per_context rebuild note); prd-expansion-scope.md §3.3
  BC-2.06.003 description; ARCH-INDEX.md §Capability Traceability SS-06.
- EC-075 documents an important edge case: Ctrl-P is a modifier combination, not a printable
  character, so it is NOT captured by search_prompt during Filtering mode — it falls through
  to the Global or Builtin tables. This ensures ProfilePicker is always accessible.
- Postcondition 3 explicitly lists the Overlay search_prompt bindings because the
  `[y]`/`[Enter]`/`[A]`/`[n]`/`[r]`/`[t]` key assignments are the most safety-critical dispatch rules
  in the system (they trigger IPC `ClientToServer::PermissionDecision` messages to the daemon).


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**IPC sweep — fabricated `DecisionResponse` in rationale note replaced** (2026-05-26T14:30:00Z):
- Design Notes paragraph: "trigger IPC DecisionResponse messages" → "trigger IPC `ClientToServer::PermissionDecision` messages".
  This is an explanatory note about why the permission key assignments are
  safety-critical; the informal shorthand `DecisionResponse` was replaced with the canonical
  `ClientToServer::PermissionDecision` per SS-ipc.md §Client-to-Server Messages.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.1.0

**ADJ-ADV2-001 HIGH — Keybinding + layer propagation from BC-2.06.011/012/013 v1.1.0** (2026-05-27T09:00:00Z):

BC-2.06.011, BC-2.06.012, and BC-2.06.013 were updated to v1.1.0 in the same burst as BC-INDEX
v1.21 (ADJ-ADV2-001 adjudication). BC-2.06.003 was NOT updated in that burst — it still referenced
`1`/`2`/`3` keybindings in the `PerContext` layer. This trace records the propagation fix.

**Keybinding changes (Postcondition 3, EC-070, EC-071, Test Vectors, VP, Cross-Refs, Related BCs):**
- `KeyCode::Char('1')` → `Action::PermissionAcceptOnce` replaced with `KeyCode::Char('y')` + `KeyCode::Enter` → `Action::PermissionAcceptOnce`.
- `KeyCode::Char('2')` → `Action::PermissionAcceptAlways` replaced with `KeyCode::Char('A')` → `Action::PermissionAcceptAlways`.
- `KeyCode::Char('3')` → `Action::PermissionReject` replaced with `KeyCode::Char('n')` + `KeyCode::Char('r')` → `Action::PermissionReject`.
- Rationale: mnemonic set (`y`/`Enter`/`A`/`n`/`r`) is canonical per ADJ-ADV2-001 adjudication
  (lazygit-philosophy verb keybindings; S-026/S-027 are authoritative UX design artifacts).

**Layer change (Postcondition 3):**
- Permission decision bindings moved from `per_context` table to `search_prompt` table (highest
  priority). This matches BC-2.06.011 v1.1.0 Precondition 2, BC-2.06.012 v1.1.0 Precondition 2,
  BC-2.06.013 v1.1.0 Precondition 2, and S-026 AC-009 (all of which specify `SearchPrompt` layer).
- Rationale: overlay decision keys must override any `PerContext`, `Global`, or `Builtin` bindings
  that could intercept `y`, `Enter`, `A`, `n`, or `r` during overlay mode. The `SearchPrompt`
  layer is the correct home for modal-input capture (as it is for filter-mode printable-char capture).

**Sections updated:** frontmatter version + modified[], Postcondition 3, EC-070, EC-071, all three
overlay-key test vector rows (replaced with 5 rows covering y/Enter/A/n/y-in-Dashboard), VP first
row, Cross-Ref table cell, Related BCs bullet points for 011/012/013, §Trace v1.0.0 and §Trace v1.0.3
notes referencing `[1]/[2]/[3]`.

**Sections NOT changed:** Postconditions 1/2/4/5/6, all Invariants, EC-072/EC-073/EC-074/EC-075, VP rows 2+3,
Architecture Anchors, Story Anchor, VP Anchors, Traceability rows other than Cross-Ref.

SE-16d monotonicity: v1.1.0 timestamp 2026-05-27T09:00:00Z > v1.0.4 (2026-05-26T00:00:00Z). PASS.

## §Trace v1.1.1

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): this BC's §Overlay mode keybindings reference `AppMode::Overlay` only by mode name; no `Overlay { stack }` variant shape in scope.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers keybinding dispatch; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
- SE-16d monotonicity: v1.1.1 timestamp 2026-05-29T00:00:00Z > v1.1.0. PASS.
