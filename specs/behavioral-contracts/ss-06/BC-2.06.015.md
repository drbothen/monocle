---
document_type: behavioral-contract
level: L3
version: "1.0.7"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-01T12:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "64a61b4"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010, S-027-ADV-CONTRADICTION-FIX, S-027-ADV-BINDING-LAYER-FIX]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.015: Permission Overlay: `[t]` Trace-to-Source Stub

## Description

In `AppMode::Overlay`, pressing `[t]` (bound to `Action::PermissionTraceToSource` via
the per-context `AppModeTag::Overlay` slot in `build_builtin_binding_layers()`) sets
`App.status_message` to the placeholder text
`"[t] Trace to source — Phase 2 feature (Static plane)"`, which the status bar renders
on the next draw tick using the shared `Color::Yellow` convention.

No navigation occurs. No file is opened. No IPC message is sent to the daemon. The
`AppMode` remains `Overlay { prior }` unchanged and `App.overlay_stack` is not modified.
The keybinding is hardcoded in `build_builtin_binding_layers()` (non-user-overridable)
and appears in the keybinding hint line so it is discoverable in Phase 1. This stub
reserves the `[t]` binding for Phase 2 (the Static plane), preventing future keybinding
conflicts when the full trace-to-source behavior is implemented.

**Mechanism rationale (S-027 contradiction resolution):** `App.status_message: Option<String>`
is the canonical mechanism for transient status bar notifications in this architecture (see
`app.rs` — it is set by `on_transport_event` for reconnect messages and by
`reconnect_from_offline` for offline messages). Reusing it for the `[t]` placeholder is
architecturally consistent. The press is gated — the message only appears after `[t]` is
pressed, not on every render — but no `trace_pressed: bool` flag is needed because
`status_message` is already the right single-field transient mechanism (it is cleared by
subsequent IPC events or by the user making a decision that transitions mode).

## Preconditions

1. `AppMode` is `Overlay { prior }` and `App.overlay_stack.len() >= 1`.
2. `build_builtin_binding_layers()` has registered `t` → `Action::PermissionTraceToSource`
   in the `BindingLayers::per_context` map keyed by `(key_t, AppModeTag::Overlay)`.
   This is the correct scope for an Overlay-only, non-user-overridable binding: the
   `BindingLayers::builtin` map is mode-agnostic (`HashMap<KeyEvent, Action>` with no
   mode discriminant), so inserting `t` there would make it fire in Dashboard and all
   other modes, directly violating EC-099. The per-context entry is hardcoded inside
   `build_builtin_binding_layers()` — it is not loaded from any user customisation file —
   so it satisfies the non-overridable intent of INV-1 despite being placed at the
   `PerContext` layer in `BindingSource` terms.
3. The `Action::PermissionTraceToSource` variant is defined in `monocle-core/src/tui/state.rs`
   (the `Action` enum) with the comment `// Phase 1: stub sets status_message placeholder`.
4. `App.status_message` is `Option<String>` — the existing transient notification field
   (already present in `App` at app.rs line 137; set by `on_transport_event` and
   `reconnect_from_offline`).

## Postconditions

1. **Placeholder set in status_message:** When `[t]` is pressed in `AppMode::Overlay`,
   the handler sets:
   ```
   app.status_message = Some("[t] Trace to source — Phase 2 feature (Static plane)".to_string());
   ```
   The status bar renders this message on the next draw tick using the shared
   `status_message` rendering convention (`Color::Yellow` — the same styling used by
   transport notifications such as `"[disconnected] reconnecting..."` and offline status
   messages; see `ui/status_bar.rs`). No bespoke/special styling is required beyond this
   shared convention; distinguishing `[t]` stub messages by source at the render layer
   would require a separate message-source discriminant and is out of scope for Phase 1.
   The message is press-gated: it appears only after `[t]` is pressed, not on every render
   of the overlay.
2. **No AppMode transition:** `transition(Overlay { prior }, PermissionTraceToSource)`
   returns `Overlay { prior }` unchanged. `App.overlay_stack` is not modified. This is
   the identity transition for this action.
3. **No IPC message sent:** The TUI does NOT send any IPC message to the daemon when `[t]`
   is pressed. The placeholder is a local `App.status_message` mutation only.
4. **No navigation:** No file viewer, no editor, no subprocess is launched. The TUI
   remains in the ratatui event loop showing the permission overlay.
5. **Keybinding discoverable via hint line (BC-2.06.021):** The keybinding hint line in
   the status bar when `AppMode::Overlay` is active includes `t: trace` (per BC-2.06.021
   PC-1 Overlay row). This is the primary discoverability path for the stub. The
   `status_message` content provides additional confirmation on press.
6. **status_message lifecycle:** The placeholder message remains in `App.status_message`
   until it is overwritten by a subsequent event (e.g., IPC reconnect sets
   `"[disconnected] reconnecting..."`, or a permission decision transitions mode). It is
   NOT cleared automatically on the next render tick. This matches the existing
   `status_message` lifecycle used by transport event handlers.
7. **`Action::PermissionTraceToSource` defined in `monocle-core` regardless of Phase:**
   The variant exists in the `Action` enum in Phase 1 so that `[t]` can be bound without
   dead-code warnings or missing-match-arm compiler errors in `monocle-tui`. The
   `monocle-tui` match arm for this action sets `app.status_message`.

## Invariants

1. `Action::PermissionTraceToSource` is a **non-user-overridable, hardcoded binding**
   registered in `build_builtin_binding_layers()` under the `PerContext` map scoped to
   `AppModeTag::Overlay`. It intentionally does NOT use the `BindingLayers::builtin`
   (`HashMap<KeyEvent, Action>`) map: that map has no mode discriminant, so registering
   `t` there would cause `t` to fire in Dashboard, Filtering, and all other modes —
   directly violating EC-099 ("Dashboard `t` must be identity/unbound"). The per-context
   placement achieves Overlay-only scope while being equally non-overridable (the entry
   is hardcoded in the binary's `build_builtin_binding_layers()`, not loaded from any
   user customisation file). The resolved `BindingSource` is `BindingSource::PerContext`,
   not `BindingSource::Builtin`; INV-1's "non-overridable" intent is satisfied by the
   registration site, not by the `BindingSource` enum variant.
2. The `[t]` stub is present in Phase 1 SOLELY to reserve the keybinding. The Phase 2
   implementation (trace-to-source in the Static plane) will replace the stub behavior
   without needing to add a new binding — the binding already exists.
3. The placeholder text includes "Phase 2 feature" to communicate clearly to users who
   press `[t]` in Phase 1 that this is an upcoming feature, not a broken one.
4. No persistent per-press boolean state is stored. There is no `trace_pressed: bool`
   field in `App`. The placeholder message is delivered via `App.status_message`, which is
   the existing transient notification field — NOT a dedicated trace flag. This field is
   shared with other notification sources (reconnect status, offline status); the last
   write wins. A `[t]` press simply participates in the normal `status_message` lifecycle
   without requiring any new App fields.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-097 | `[t]` pressed when no `PromptModal` has a file-backed tool (e.g., tool is `Bash { command }`) | Same `status_message` set; stub does not inspect `tool_payload`; no change from the normal stub behavior |
| EC-098 | `[t]` pressed multiple times in sequence | Each press sets `app.status_message` to the same placeholder string; no state accumulation; no error; the status bar continues to render the placeholder |
| EC-099 | `[t]` pressed in `AppMode::Dashboard` | No binding match (not in Global or Dashboard PerContext tables); identity transition; keypress discarded silently; `status_message` is not changed |
| EC-100 | Phase 2 story implements real trace-to-source behavior | Phase 2 replaces the stub `status_message` arm with real navigation; `Action::PermissionTraceToSource` variant is already defined; no breaking change to the Action enum |
| EC-101 | Terminal is too narrow to render the full placeholder text in the status bar | ratatui truncates the `status_message` text at the right edge; no panic; no error; the most important prefix is preserved |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]), `PermissionTraceToSource` | `Overlay { prior: Sessions }` (App.overlay_stack = [P1], unchanged); `app.status_message = Some("[t] Trace to source — Phase 2 feature (Static plane)")` | happy-path |
| `Dashboard { focused: Sessions }`, `PermissionTraceToSource` (unbound) | `Dashboard { focused: Sessions }` (identity; `t` is not bound in Dashboard); `app.status_message` unchanged | edge-case |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]), `PermissionTraceToSource` × 3 | `Overlay { prior: Sessions }` (App.overlay_stack = [P1], still unchanged); `app.status_message = Some("[t] Trace to source — Phase 2 feature (Static plane)")` (last write wins; idempotent) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `[t]` in `AppMode::Overlay` does not modify `overlay_stack` or send IPC | unit test |
| VP-TBD | `[t]` in `AppMode::Overlay` sets `app.status_message` to the exact placeholder string | unit test |
| VP-TBD | `[t]` in `AppMode::Dashboard` is unbound: `status_message` unchanged | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the `[t]` trace-to-source stub within the "permission overlay stack" component of CAP-006; the stub reserves the keybinding for Phase 2 Static plane integration while ensuring the binding is discoverable in Phase 1 via the hint line |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: the stub sends no IPC message and writes no files; it is a pure `App.status_message` mutation) |
| Architecture Module | monocle-core (Action::PermissionTraceToSource variant — reserved); monocle-tui (key handler stub arm for PermissionTraceToSource sets `app.status_message`) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Permission Overlay §Trace-to-Source Stub; §Action Enum (PermissionTraceToSource variant with `// Phase 1: stub` comment); §Status Bar §Keybinding hint line (Overlay mode includes `t: trace`) |
| Cross-Ref | BC-2.06.001 (pure transition function — PermissionTraceToSource identity arm), BC-2.06.003 (5-level binding precedence — PerContext/Overlay level for `[t]`; non-overridable by virtue of hardcoded registration in `build_builtin_binding_layers()`), BC-2.06.021 (`t: trace` in Overlay hint line) |
| Test File | `monocle-tui/tests/overlay_stub.rs` |
| Test Name | `monocle-tui/tests/overlay_stub.rs`: `test_BC_2_06_015_handler_sets_status_message_exact_canonical_text` (PC-1), `test_BC_2_06_015_handler_sets_status_message_mode_stays_overlay` (PC-2), `test_BC_2_06_015_handler_no_ipc_sent_on_trace_key` (PC-3), `test_BC_2_06_015_ec099_t_in_dashboard_status_message_unchanged` (EC-099), `test_BC_2_06_015_production_binding_t_in_overlay_resolves_trace_to_source` (INV-1), `test_BC_2_06_015_ec098_repeated_t_press_idempotent_status_message` (EC-098); `monocle-core/tests/tui_binding.rs`: `test_BC_2_06_015_permission_trace_to_source_variant_compiles_and_not_resolved_from_empty_layers` (PC-7 compile gate); `monocle-core/tests/tui_state_machine.rs`: `test_BC_2_06_015_transition_overlay_permission_trace_to_source_is_identity` (PC-2 state machine) |
| Stories | S-027 |

## Related BCs

- [BC-2.06.001] — depends on: `PermissionTraceToSource` is the identity arm in the `transition()` pure function; the variant must be covered in any exhaustive match
- [BC-2.06.003] — depends on: `[t]` is registered at the `PerContext` level (scoped to `AppModeTag::Overlay`) of the 5-level binding precedence system; the binding is non-user-overridable because it is hardcoded in `build_builtin_binding_layers()`, not because it uses the `Builtin` layer
- [BC-2.06.021] — composes with: `t: trace` appears in the Overlay hint line as the primary discoverability surface for this stub

## Architecture Anchors

- `architecture/SS-tui.md#trace-to-source-stub` — Phase 1 stub behavior (sets `App.status_message`)
- `architecture/SS-tui.md#action-enum` — `PermissionTraceToSource` variant definition with Phase 1 comment
- `crates/monocle-tui/src/app.rs` — `App.status_message: Option<String>` (transient notification field reused for stub)

## Story Anchor

S-027 — Permission Overlay RENDERING + Diff Preview + Status Bar (absorbs `[t]` trace-to-source stub)

## VP Anchors

- VP-TBD — Unit tests for stub `status_message` set and no-op transition behavior

## §Trace v1.0.7

**S-027-ADV-BINDING-LAYER-FIX MEDIUM — Binding-layer placement corrected; test names grounded; PC-1 color clarified** (2026-06-01T12:00:00Z):

Three defects surfaced by adversarial review of the implemented `[t]` stub (S-027 TDD red gate passed):

**I-1 (MEDIUM) — Binding-layer placement corrected (PC-2, INV-1, Description, Related BCs, Cross-Ref):**
- Previous wording stated the `t` → `PermissionTraceToSource` binding is in the
  `Builtin` binding table for `AppMode::Overlay`. This is architecturally unsatisfiable:
  `BindingLayers::builtin` is `HashMap<KeyEvent, Action>` with no mode discriminant, so
  inserting `t` there makes it fire in ALL modes — directly violating EC-099.
- The implementation places the binding in `BindingLayers::per_context` keyed by
  `(key_t, AppModeTag::Overlay)` inside `build_builtin_binding_layers()`. This is the
  ONLY mechanism that achieves both Overlay-only scope AND non-user-overridability
  (hardcoded in the binary function, not in any user customisation file).
- PC-2 reworded: describes the actual `per_context` insertion with `AppModeTag::Overlay`,
  explains why `builtin` cannot be used (mode-agnostic → would break EC-099), and
  confirms non-overridable intent is satisfied by the registration site.
- INV-1 reworded: replaces `"a Builtin binding (not PerContext, not UserCustomCommand)"`
  with an accurate statement: non-user-overridable binding registered at `PerContext`
  scope for `AppModeTag::Overlay`; explains the mode-agnostic `builtin` map constraint.
  The resolved `BindingSource` is `BindingSource::PerContext`; the non-overridable intent
  is satisfied by the hardcoded registration site.
- Description, Related BCs (BC-2.06.003 bullet), Cross-Ref updated to use correct layer.

**N-2 (ADVISORY) — Test names corrected to match overlay_stub.rs implementation:**
- Previous `§Test Name`: `test_BC_2_06_015_trace_to_source_stub_sets_status_message`
  (non-existent; leftover from v1.0.6 partial update).
- Replaced with complete list of 8 real test functions grounded in
  `crates/monocle-tui/tests/overlay_stub.rs` and `crates/monocle-core/tests/`:
  `test_BC_2_06_015_handler_sets_status_message_exact_canonical_text` (PC-1),
  `test_BC_2_06_015_handler_sets_status_message_mode_stays_overlay` (PC-2),
  `test_BC_2_06_015_handler_no_ipc_sent_on_trace_key` (PC-3),
  `test_BC_2_06_015_ec099_t_in_dashboard_status_message_unchanged` (EC-099),
  `test_BC_2_06_015_production_binding_t_in_overlay_resolves_trace_to_source` (INV-1),
  `test_BC_2_06_015_ec098_repeated_t_press_idempotent_status_message` (EC-098),
  `test_BC_2_06_015_permission_trace_to_source_variant_compiles_and_not_resolved_from_empty_layers`
  (tui_binding.rs PC-7 compile gate),
  `test_BC_2_06_015_transition_overlay_permission_trace_to_source_is_identity`
  (tui_state_machine.rs PC-2 state machine).

**N-3 (ADVISORY) — PC-1 color clarification:**
- Previous PC-1 stated "default terminal color (no special styling required in Phase 1)".
  The implementation renders `status_message` via the shared status bar field which always
  applies `Color::Yellow` (confirmed in `ui/status_bar.rs` — same styling as
  `"[disconnected] reconnecting..."` and offline messages).
- PC-1 updated: "shared `status_message` rendering convention (`Color::Yellow`); no
  bespoke/special styling beyond this shared convention." Distinguishing `[t]` stub
  messages by source at the render layer would require a separate message-source
  discriminant — out of scope for Phase 1.

**NOTE for story-writer:** AC-013 in S-027 says "default terminal color" — story-writer
should align AC-013 to the shared `Color::Yellow` status_message convention (N-3).

- SE-16d monotonicity: v1.0.7 timestamp 2026-06-01T12:00:00Z > v1.0.6 timestamp 2026-06-01T00:00:00Z. PASS.

## §Trace v1.0.6

**S-027-ADV-CONTRADICTION-FIX HIGH — Internal contradiction resolved; Story Anchor assigned** (2026-06-01T00:00:00Z):

Contradiction: PC-1 stated the placeholder was rendered "when `[t]` is pressed" (press-gated),
but Invariant 4 stated "the placeholder rendering is driven entirely by whether `AppMode::Overlay`
is active" (always-on, not press-gated). These two statements cannot both be true. Invariant 4's
"always-on" framing also implied a footer area that perpetually shows the stub text — inconsistent
with the existing architecture where the overlay footer renders action-specific content, not a
static Phase 2 notice.

Resolution — OPTION X (press-gated transient via `App.status_message`):
- `App` already has `pub status_message: Option<String>` (app.rs line 137) used by
  `on_transport_event` and `reconnect_from_offline` for transient notifications. Using it for
  the `[t]` placeholder is consistent with the existing architecture pattern.
- `Action::PermissionTraceToSource` is absent from the codebase (`grep` confirms no definition
  in `monocle-core/src/tui/state.rs` Action enum or `binding.rs`). The stub requires both the
  Action variant and a Builtin `[t]` binding to be added by S-027.
- PC-1 rewritten: pressing `[t]` sets `app.status_message = Some("<placeholder>")`.
- Invariant 4 rewritten: "No persistent per-press boolean state. No `trace_pressed: bool` field.
  `App.status_message` is the delivery vehicle — NOT a dedicated trace flag."
- PC-2 (identity transition), PC-3 (no IPC), PC-7 (Action variant defined in monocle-core) are
  semantically unchanged; PC-6 updated to describe `status_message` lifecycle (last-write-wins,
  not auto-cleared on render).
- PC-5 clarified: discoverability is primarily via BC-2.06.021 hint line (`t: trace`);
  `status_message` provides confirmation on press.
- Test vectors updated: Expected Output column now specifies `app.status_message = Some(...)`.
- Test Name updated: `test_BC_2_06_015_trace_to_source_stub_renders_placeholder` →
  `test_BC_2_06_015_trace_to_source_stub_sets_status_message`.
- Story Anchor: `S-TBD` → `S-027` (human-authorized absorption of stub into S-027 scope).
- VP Anchors: `VP-TBD` description updated to match new `status_message` mechanism.
- BC description revised to include the mechanism rationale inline.
- SE-16d monotonicity: v1.0.6 timestamp 2026-06-01T00:00:00Z > v1.0.5 timestamp 2026-05-29T00:00:00Z. PASS.

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.015 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.1.0 §Permission Overlay §Trace-to-Source Stub, §Action Enum
  (PermissionTraceToSource variant); prd-expansion-scope.md §3.3 BC-2.06.015 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: stub renders placeholder text only; no file writes or IPC sends.
- Priority P1 (vs P0 for BCs 009–014) per SS-tui.md BC table and prd-expansion-scope.md §3.3.
- Invariant 1 records that `[t]` is a `Builtin` binding (non-overridable in Phase 1).
- Postcondition 7 documents why the variant must be defined in `monocle-core` even though
  it does nothing in Phase 1: to prevent missing-match-arm compiler errors.


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
- Resolves F-S025-ADV3-BLOCKER-002. `Overlay { stack, prior }` → `Overlay { prior }` in description, Precondition 1, Postcondition 2. `App.overlay_stack` noted as unmodified by `[t]` press. Decision keybindings in Postcondition 6 updated from `[1]/[2]/[3]` to `[y]/[A]/[n/r]` (per ADJ-ADV2-001).
- SE-16d monotonicity: v1.0.4 timestamp 2026-05-28T00:00:00Z > v1.0.3. PASS.

## §Trace v1.0.5

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): already propagated in §Trace v1.0.4 above (`Overlay { stack, prior }` → `Overlay { prior }`; `App.overlay_stack` noted as unmodified by `[t]` press).
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers trace-to-source stub; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC (stub renders placeholder footer).
- SE-16d monotonicity: v1.0.5 timestamp 2026-05-29T00:00:00Z > v1.0.4. PASS.
