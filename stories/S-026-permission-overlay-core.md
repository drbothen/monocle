---
document_type: story
level: L4
story_id: S-026
epic_id: EPIC-06
version: "1.10"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-28T00:00:00Z
phase: 2
points: 13
wave: 6
tdd_mode: strict
priority: P0
depends_on: [S-024, S-022, S-023]
blocks: [S-027, S-029]
target_module: monocle-tui
subsystems: [SS-06]
behavioral_contracts: [BC-2.06.008, BC-2.06.009, BC-2.06.011, BC-2.06.012, BC-2.06.013, BC-2.06.014, BC-2.06.016, BC-2.06.023, BC-2.06.024, BC-2.05.002]
verification_properties: []
estimated_days: 5
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.008.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.009.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.011.md, version: "1.2.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.012.md, version: "1.2.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.013.md, version: "1.2.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.014.md, version: "1.0.7"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.016.md, version: "1.0.8"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.023.md, version: "1.5.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.024.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md, version: "1.0.5"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "b8e5828"
traces_to: "Implements BC-2.06.008 (overlay push/FIFO), BC-2.06.009 (stack rotation), BC-2.06.011 (Accept-Once keybinding), BC-2.06.012 (Accept-Always keybinding), BC-2.06.013 (Reject keybinding), BC-2.06.014 (Esc hide), BC-2.06.016 (disconnect clear/reconnect), BC-2.06.023 (UUID removal), BC-2.06.024 (PermissionPromptPayload→PromptModal conversion), BC-2.05.002 Invariant 4 (idempotent PermissionPromptQueued handler)"
---

# S-026: Permission Overlay Core — Push/Pop, Decision Dispatch, Disconnect Clear, UUID Removal

## Narrative

As a daemon operator, I want the TUI to display permission prompts in a FIFO overlay
stack, allow me to approve or reject each prompt via keyboard shortcuts, and automatically
clear the overlay when the daemon disconnects, so that I can manage tool permissions
without losing any pending prompts and without manual cleanup after disconnects.

## Acceptance Criteria

### AC-001 (traces to BC-2.06.008 PC-1 — PermissionPromptQueued push to VecDeque; also traces to BC-2.05.002 Invariant 4 — idempotent insert)
When `ServerToClient::PermissionPromptQueued { payload: PermissionPromptPayload }`
is received, the TUI MUST use the `apply_permission_prompt_queued(overlay, payload)` helper
(defined in S-025; if S-026 implements first, define it here). This helper checks whether
`payload.prompt_id` is already present in `app.overlay_stack` before inserting:
- If `prompt_id` is already present: silently discard (TRACE log only — no INFO, no WARN).
  Do NOT push to the VecDeque. Do NOT transition mode.
- If `prompt_id` is not present: construct `PromptModal` via `payload_to_modal()` (see AC-016)
  and call `app.overlay_stack.push_back(modal)`.

After a successful (non-duplicate) push, the TUI transitions to
`Overlay { prior: current_focus }` via `transition()`
if not already in Overlay mode. (The modal stack is carried in `App.overlay_stack`, not
in the `Overlay` variant — per BC-2.06.004 v1.2.1 PC-2.) If already in Overlay mode,
the stack grows in place (the IPC push is NOT routed through `transition()`).

Precondition (BC-2.05.002 Invariant 4): The IPC layer provides at-least-once delivery for
`PermissionPromptQueued` across the connection snapshot window. A `prompt_id` already
present in `app.overlay_stack` (from `InitialState.overlay_stack`) MUST be silently
discarded on the second delivery. This invariant is symmetric with the no-op behavior
required for `PermissionPromptResolved` (AC-007: unknown `prompt_id` → WARN + no-op).

### AC-002 (traces to BC-2.06.008 PC-2 — FIFO ordering preserved)
`PromptModal` items are served in FIFO order. The oldest (first received) prompt is
`overlay_stack.front()` and is displayed as the active prompt. New prompts are appended
via `push_back`. This ordering is preserved across reconnects (the daemon's
`overlay_stack` in `InitialState` preserves insertion order).

### AC-003 (traces to BC-2.06.011 PC-1 — y/Enter Accept-Once keybinding send)
In `Overlay` mode, pressing `y` or `Enter` sends
`ClientToServer::PermissionDecision { prompt_id: front.prompt_id, decision: PermissionDecision::Accept }`
over the IPC connection. The `PromptModal` remains in the `VecDeque` until
`ServerToClient::PermissionPromptResolved { prompt_id }` arrives — the TUI does NOT pop
immediately after the IPC send.

### AC-004 (traces to BC-2.06.012 PC-1 — A Accept-Always keybinding send)
In `Overlay` mode, pressing `A` (uppercase) sends
`ClientToServer::PermissionDecision { prompt_id: front.prompt_id, decision: PermissionDecision::AcceptAlways }`.
The `PromptModal` remains in the `VecDeque` until `PermissionPromptResolved` arrives.

### AC-005 (traces to BC-2.06.013 PC-1 — n/r Reject keybinding send)
In `Overlay` mode, pressing `n` or `r` sends
`ClientToServer::PermissionDecision { prompt_id: front.prompt_id, decision: PermissionDecision::Reject }`.
The `PromptModal` remains in the `VecDeque` until `PermissionPromptResolved` arrives.

### AC-006 (traces to BC-2.06.023 PC-1 — PermissionPromptResolved UUID removal via retain)
When `ServerToClient::PermissionPromptResolved { prompt_id }` is received, the TUI
uses `overlay_stack.retain(|m| m.prompt_id != prompt_id)` to remove the resolved modal
regardless of its stack position (NOT assuming it is at front). This is NOT routed
through `transition()` — it is a direct mutation of the VecDeque followed by a
stack-collapse check. If the stack is now empty after `retain()`, call
`transition(current_mode, Action::PopOverlay)` or equivalent to collapse to
`Dashboard { focused: prior }`. `retain()` removes ALL entries matching the `prompt_id`
(in case of duplicates from reconnect races), not just the first.

### AC-007 (traces to BC-2.06.023 PC-3 — unknown prompt_id is no-op)
If `PermissionPromptResolved { prompt_id }` refers to a `prompt_id` not present in the
local `overlay_stack`, the TUI logs a WARN and takes no further action. This handles
the case where the daemon resolved a prompt that the TUI was not aware of (e.g.,
auto-resolved before connect).

### AC-008 (traces to BC-2.06.014 PC-1 — Esc in Overlay is no-op identity)
Pressing `Esc` while in `Overlay` mode is a no-op identity: `transition(Overlay{..}, Action::Esc)`
returns the same `Overlay` state unchanged. Esc NEVER rejects a prompt, NEVER pops the
stack, NEVER exits overlay mode, and NEVER sends any IPC message.

### AC-009 (traces to BC-2.06.003 — SearchPrompt layer registers overlay decision bindings)
The `SearchPrompt` binding layer registers the overlay decision keys (`y`, `Enter`, `A`,
`n`, `r`) with highest priority when mode is `Overlay`. These bindings override any
Global or PerContext layers for these keys while a prompt is displayed, per the
5-level binding precedence: SearchPrompt > UserCustomCommand > PerContext > Global > Builtin.

### AC-010 (traces to BC-2.06.003 — overlay binding isolation blocks session navigation)
While in `Overlay` mode, session list navigation keys (`j`, `k`, `Tab`, `Enter` on
sessions) are consumed by the overlay binding layer and do NOT pass through to scroll
the sessions list behind the overlay. Only overlay-specific keys (`y`, `Enter`, `A`,
`n`, `r`, `Esc`) produce actions.

### AC-011 (traces to BC-2.06.016 PC-1 — disconnect clears overlay stack)
When `TransportEvent::Disconnected` is received, the TUI MUST clear `app.overlay_stack`
(set to `VecDeque::new()`) and transition to `Dashboard { focused: default_focus }`.
Any pending `PermissionDecision` sends that were in-flight are abandoned (no retry).
This satisfies SOQ-3.

### AC-012 (traces to BC-2.06.016 PC-2 — overlay restored from InitialState on reconnect)
On reconnect, `ServerToClient::InitialState { overlay_stack, .. }` re-populates the
TUI's local stack. If the daemon still has pending prompts after the TUI reconnected,
the overlay is re-entered. See S-023 for reconnect mechanics.

### AC-013 (traces to BC-2.06.009 PC-1 — stack rotation when len > 1)
When `AppMode::Overlay` is active and `stack.len() > 1`, pressing `Up` or `Down`
dispatches `Action::OverlayCycleNext`. The transition function calls
`stack.pop_front()` and `stack.push_back(popped)`, moving the front prompt to the
back and bringing the next oldest prompt to front. The overlay re-renders with the
new `stack.front()` prompt's content.

### AC-014 (traces to BC-2.06.009 EC-065 — single-item rotation is a no-op)
When `stack.len() == 1`, pressing `Up` or `Down` dispatches `Action::OverlayCycleNext`.
The rotation moves the single item back to front (effectively a no-op). No visual
change occurs. No error is raised. The stack remains a single-item VecDeque.

### AC-015 (traces to BC-2.06.023 PC-4 — empty stack collapses to Dashboard)
After `retain()` removes the last `PromptModal` from the `VecDeque`, the TUI detects
`overlay_stack.is_empty()` and calls `transition(current_mode, Action::PopOverlay)` or
equivalent to transition to `Dashboard { focused: prior }`. The overlay is never left
in a state where `AppMode::Overlay` is active but the stack is empty.

### AC-016 (traces to BC-2.06.008 PC-1 and BC-2.06.024 — payload_to_modal() conversion)
When constructing a `PromptModal` from `PermissionPromptPayload` (received via
`ServerToClient::PermissionPromptQueued`), the `payload_to_modal()` function dispatches
on `tool_name` with the following exhaustive rules:

**Edit and Write file-modification tools (share identical conversion logic):**
- `tool_name == "Edit" | "Write"` AND (`old_content.is_some() || new_content.is_some()`) →
  `ToolPayload::Edit { old_content: old_content.unwrap_or_default(), new_content: new_content.unwrap_or_default(), path: tool_input["path"].as_str() }`.
  Additionally, if `tool_input["path"]` is absent or empty, fall back to
  `ToolPayload::Generic { tool_name, tool_input }`.
- `tool_name == "Edit" | "Write"` AND BOTH `old_content` AND `new_content` are `None` →
  `ToolPayload::Generic { tool_name, tool_input }` (NOT Edit with empty strings).
  Rationale: an Edit/Write with no content produces an empty diff pane (BC-2.06.010),
  which is less informative than displaying the raw `tool_input` JSON. The Generic fallback
  renders the path and tool inputs the user already knows, giving meaningful context.
  **In Phase 1, the daemon always sends `old_content: None, new_content: None` for ALL
  deferred permission prompts** (rich diff content is S-027 scope). Therefore ALL Phase-1
  Edit and Write prompts produce `ToolPayload::Generic` from this function. This is the
  correct production-grade behavior — the overlay shows the tool name and path JSON rather
  than a blank diff pane.

**Bash tool:**
- `tool_name == "Bash"` → `ToolPayload::Bash { command: tool_input["command"].as_str() }`
  (falls back to `ToolPayload::Generic` if `"command"` key is absent or empty)

**Read tool:**
- `tool_name == "Read"` → `ToolPayload::Read { path: tool_input["path"].as_str() }`
  (falls back to `ToolPayload::Generic` if `"path"` key is absent or empty)

**All other tool names:**
- → `ToolPayload::Generic { tool_name, tool_input }`

The `received_at` field is set to `Instant::now()` at conversion time (NOT the daemon's
creation timestamp). The `PermissionPromptPayload` is the IPC wire type; `PromptModal`
is the TUI-local type — these are NOT the same struct.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~3,200 |
| BC-2.06.008.md | ~900 |
| BC-2.06.009.md | ~900 |
| BC-2.06.011.md | ~800 |
| BC-2.06.012.md | ~700 |
| BC-2.06.013.md | ~700 |
| BC-2.06.014.md | ~700 |
| BC-2.06.016.md | ~800 |
| BC-2.06.023.md | ~700 |
| BC-2.06.024.md | ~700 |
| BC-2.05.002.md (Invariant 4) | ~300 |
| S-024 (transition, AppMode) | ~700 |
| S-022 (IPC types) | ~600 |
| S-023 (reconnect, SOQ-3) | ~500 |
| Test files | ~2,000 |
| **Total estimate** | **~14,700** |

## Tasks

- [ ] Implement `payload_to_modal()` conversion function per AC-016:
      - `"Edit" | "Write"` with at least one content field Some AND path present → `ToolPayload::Edit`
      - `"Edit" | "Write"` with BOTH content fields None → `ToolPayload::Generic` (Phase-1 normal path)
      - `"Bash"` with command present → `ToolPayload::Bash`; absent → `ToolPayload::Generic`
      - `"Read"` with path present → `ToolPayload::Read`; absent → `ToolPayload::Generic`
      - all other tool names → `ToolPayload::Generic`
      - sets `received_at: Instant::now()` (AC-016)
- [ ] Use `apply_permission_prompt_queued` helper (from S-025) for the streaming
      `PermissionPromptQueued` IPC handler — do NOT call `push_back` directly; all inserts
      go through the idempotent helper (BC-2.05.002 Invariant 4; AC-001)
- [ ] Implement `PermissionPromptQueued` IPC handler in `monocle-tui/src/app.rs`:
      call `apply_permission_prompt_queued(overlay, payload)` (idempotent), then enter/grow
      Overlay via `transition()` or direct if push was non-duplicate (AC-001)
- [ ] Integration test `test_snapshot_window_prompt_dedup` (in
      `monocle-ipc/tests/connection_handshake.rs` or `monocle-tui/tests/overlay_idempotency.rs`):
      (1) Daemon running with queued prompt_id=X in pending_decisions.
      (2) New TUI connects; concurrent prompt_id=Y arrives during snapshot window.
      (3) TUI receives InitialState.overlay_stack containing X and Y (snapshot included Y).
      (4) TUI also receives streaming PermissionPromptQueued { payload: Y } from mpsc.
      (5) Assert: TUI VecDeque<PromptModal> contains X and Y exactly once each (Y not doubled).
      Per architect-decisions-pass-6.md §Implementer Directive.
- [ ] Implement `PermissionPromptResolved` IPC handler: `retain()` on `overlay_stack`,
      trigger empty-stack collapse check, log WARN on unknown prompt_id (AC-006, AC-007, AC-015)
- [ ] Implement keyboard handler for `y`/`Enter` → `PermissionDecision::Accept` IPC send;
      modal stays in VecDeque until resolved (AC-003)
- [ ] Implement keyboard handler for `A` → `PermissionDecision::AcceptAlways` IPC send (AC-004)
- [ ] Implement keyboard handler for `n`/`r` → `PermissionDecision::Reject` IPC send (AC-005)
- [ ] Implement `Action::OverlayCycleNext` handler: `pop_front` + `push_back` for rotation;
      no-op when `stack.len() == 1` (AC-013, AC-014)
- [ ] Register `Up`/`Down` keys in `SearchPrompt` layer to dispatch `Action::OverlayCycleNext`
      when `AppMode::Overlay` is active
- [ ] Implement `TransportEvent::Disconnected` handler: clear `overlay_stack`, transition to `Dashboard` (AC-011)
- [ ] Implement `ServerToClient::InitialState` handler (extend S-025): re-populate `overlay_stack`
      from `initial_state.overlay_stack` on reconnect (AC-012)
- [ ] Register overlay decision bindings (`y`, `Enter`, `A`, `n`, `r`) in `SearchPrompt` layer
      of `BindingLayers`; verify Esc is identity no-op (AC-008, AC-009)
- [ ] Verify overlay keyboard bindings block session nav keys while in Overlay mode (AC-010)
- [ ] Unit tests `monocle-tui/tests/overlay_push_pop.rs` — FIFO push/pop, empty-stack collapse,
      FIFO ordering, payload_to_modal() conversion covering:
      - Edit with both content Some → ToolPayload::Edit
      - Edit with both content None (Phase-1 normal) → ToolPayload::Generic
      - Write with both content None (Phase-1 normal) → ToolPayload::Generic
      - Write with at least one content Some → ToolPayload::Edit
      - Bash with command → ToolPayload::Bash; Bash without command → ToolPayload::Generic
      - Read with path → ToolPayload::Read; Read without path → ToolPayload::Generic
      - Unknown tool → ToolPayload::Generic
- [ ] Unit tests `monocle-tui/tests/overlay_decision.rs` — Accept/AcceptAlways/Reject IPC send,
      wait-for-resolved semantics
- [ ] Unit tests `monocle-tui/tests/overlay_rotation.rs` — Up/Down rotation with len>1,
      single-item no-op rotation
- [ ] Unit tests `monocle-tui/tests/overlay_disconnect.rs` — clear on disconnect, restore on
      reconnect via InitialState
- [ ] Unit tests `monocle-tui/tests/overlay_uuid_removal.rs` — retain() semantics, duplicate
      removal, unknown prompt_id no-op, empty-stack collapse after last removal

## Previous Story Intelligence

S-024 (TUI core types): `transition()` enforces empty-stack collapse internally. After
`retain()`, if `overlay_stack` is empty, the TUI must trigger an equivalent collapse —
either by calling `transition(current, Action::PopOverlay)` (which will use the last
known prior) or by directly setting mode to `Dashboard { focused: prior }`. The cleanest
approach is to store `prior: FocusSnapshot` separately in `App` and collapse manually
after `retain()`.

S-022 (UDS IPC types): `ClientToServer::PermissionDecision { prompt_id: Uuid, decision: PermissionDecision }`.
`PermissionDecision` enum has `Accept`, `AcceptAlways`, `Reject` variants. Confirm exact
field names from S-022 before implementing — do not invent field names.

S-023 (reconnect + SOQ-3): SOQ-3 requires overlay cleared on daemon disconnect (AC-011).
S-023 implements the reconnect polling; this story implements the clear-on-disconnect.
These are complementary: S-026 clears, S-023 restores on reconnect.

## Architecture Compliance Rules

From `architecture/SS-tui.md`:
- IPC push path (`PermissionPromptQueued`) is NOT routed through `transition()` —
  push directly to `app.overlay_stack`, then update AppMode
- UUID removal via `retain()` is NOT through `transition()` — direct mutation + collapse check
- `transition(Overlay{..}, Esc)` is identity — never pop, never reject
- `ClientToServer::PermissionDecision` — NOT `PermissionResponse` or any other name
- `PermissionDecision::Accept` | `AcceptAlways` | `Reject` — exact variant names
- `ServerToClient::PermissionPromptResolved { prompt_id }` — NOT `PermptResolved` or abbreviated
- `TransportEvent::Disconnected` — NOT `IpcServerMessage::DaemonDisconnect` (doesn't exist)
- `overlay_stack` is the IPC field name in `InitialState` — local TUI copy is `VecDeque<PromptModal>`
- Do NOT send any IPC message on Esc in Overlay — it is purely local state

**Forbidden Dependencies:**
- Do NOT define `PermissionDecision` in `monocle-tui` — import from `monocle-ipc`
- Do NOT define `PromptModal` in `monocle-tui` — import from `monocle-core`
- Do NOT route `retain()` path through `transition()` — direct mutation is correct per BC-2.06.023

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| monocle-core | workspace path | `PromptModal`, `AppMode`, `transition()`, `Action` |
| monocle-ipc | workspace path | `ServerToClient`, `ClientToServer`, `PermissionDecision`, `TransportEvent` |
| uuid | workspace pin | `Uuid` match in `retain()` |
| std::time::Instant | stdlib | `PromptModal.received_at` |
| tracing | 0.1 | WARN log on unknown prompt_id |

## File Structure Requirements

Files to modify (all in `monocle-tui/src/`):
- `app.rs` — add `PermissionPromptQueued` handler, `PermissionPromptResolved` handler,
  `TransportEvent::Disconnected` extension (clear overlay), `InitialState` extension (overlay restore)
- `ui/mod.rs` — prepare overlay rendering module placeholder (S-027 fills it in)

Files to create:
- `monocle-tui/tests/overlay_push_pop.rs` — push/pop/FIFO/collapse tests + payload_to_modal() conversion tests
- `monocle-tui/tests/overlay_decision.rs` — decision dispatch + resolved round-trip tests
- `monocle-tui/tests/overlay_rotation.rs` — Up/Down rotation tests (len>1 and single-item no-op)
- `monocle-tui/tests/overlay_disconnect.rs` — disconnect clear + reconnect restore via InitialState tests
- `monocle-tui/tests/overlay_uuid_removal.rs` — retain() semantics, duplicate removal, empty-stack collapse tests

## Downstream Consumer Contract

Public behavior produced by this story for downstream consumption:

The `App` struct (from S-025) gains these guaranteed behaviors:
- `overlay_stack` (VecDeque<PromptModal>) correctly tracks the daemon's overlay state
- `mode` is always `Overlay { .. }` when `overlay_stack` is non-empty; never `Overlay { .. }` with `App.overlay_stack` empty
- Decision keys dispatch correct `ClientToServer::PermissionDecision` IPC messages
- Disconnect clears overlay; reconnect restores it from `InitialState`

S-027 (overlay rendering + diff preview) builds its UI atop these guaranteed behaviors.
S-029 (killer scenario integration test) validates the full round-trip.

## §Trace v1.10

**F-S026-ADV1-MED-001 — AC-016 Write tool + None/None → Generic fallback** (2026-05-31T00:00:00Z):
- Finding: AC-016 omitted "Write" as a handled `tool_name` and specified the Edit guard as
  `AND old_content.is_some() AND new_content.is_some()` (both-must-be-Some), contradicting the
  architecture source of truth (SS-tui.md v1.8.2 §IPC Payload to PromptModal Conversion) which
  uses `is_some() || is_some()` (at-least-one). More critically, AC-016 did not specify the
  None/None → Generic fallback path — which is the ONLY path exercised in Phase 1 (daemon always
  sends `old_content: None, new_content: None` per pre_tool_use.rs:252-253).
- Adjudication decisions:
  1. **"Write" MUST be handled.** `Write` is a distinct Claude Code tool (`monocle-core/src/permissions.rs:181`,
     `monocle-ipc/src/types.rs:255` — "for Edit/Write tools", tests throughout). AC-016 AND BC-2.06.024
     must name it explicitly. It shares Edit's conversion logic: same is_some() guard, same path extraction,
     same None/None → Generic fallback.
  2. **None/None → Generic is the correct production-grade behavior.** SS-tui.md v1.8.2 §F-P1D4-008 is
     explicit: "An Edit with no content to diff renders as an empty diff pane; the Generic fallback renders
     the raw tool_input JSON, which is more informative." The implementation's `unwrap_or_default()` with no
     guard is wrong — it produces Edit{old:"",new:""} which shows an empty diff pane. The spec must match
     the architecture: the guard is `is_some() || is_some()`; if both are None, Generic is the result.
  3. **Phase-1 implication stated explicitly.** All Phase-1 Edit and Write prompts will produce
     ToolPayload::Generic (because the daemon sends None/None). This is the correct behavior and must be
     tested. S-027 (diff preview) adds content population, enabling the Edit variant in Phase 2+.
- AC-016 rewritten to: (a) add "Write" alongside "Edit", (b) specify the OR guard
  (`is_some() || is_some()`), (c) state the None/None → Generic fallback explicitly, (d) document
  the Phase-1 implication so the implementer cannot miss it.
- Tasks: `payload_to_modal()` task updated with explicit bullet rules per the new AC-016.
- Tests: overlay_push_pop.rs task updated with Phase-1 test cases (Edit/Write None/None → Generic).
- BC-2.06.024 inputs pin bumped: v1.0.1 → v1.1.0 (BC-2.06.024 updated with Write + Or guard).
- BC-2.06.023 inputs pin bumped: v1.4.0 → v1.5.0 (F-S026-ADV1-LOW-001 fix).
- SE-16d monotonicity: v1.10 timestamp 2026-05-31 > v1.9 (2026-05-29). PASS.

## §Trace v1.8

**F-S025-ADV22-MED-001 sibling propagation — SS-tui-core.md → SS-tui.md (line 257)** (2026-05-29):
- Architecture Compliance Rules header: `architecture/SS-tui-core.md` → `architecture/SS-tui.md`.
- Systematic EPIC-06 story-writing burst defect; canonical anchor is `SS-tui.md` per BC-2.06.005 §Architecture Source + audit-table.md row 41.
- SE-16d monotonicity: v1.8 timestamp 2026-05-29 >= v1.7 timestamp 2026-05-28. PASS.

## §Trace v1.7

**F-S025-ADV11-HIGH-001 PO Option B — BC-2.06.016 pin propagation** (2026-05-28):
- BC-2.06.016 inputs pin updated: v1.0.7 → v1.0.8 (PO Option B decision: disconnect text style in PC-1/PC-2).
- No body changes required: AC-011 and AC-012 behavioral semantics unchanged.
- SE-16d monotonicity: v1.7 timestamp 2026-05-28T00:00:00Z >= v1.6 timestamp 2026-05-28T16:00:00Z. PASS (same-day).

## §Trace v1.6

**F-S025-ADV5-HIGH-003 / Pass 5 cumulative pin propagation** (2026-05-28):
- BC-2.06.014 inputs pin updated: v1.0.6 → v1.0.7.
- BC-2.06.014 v1.0.7 corrected EC-096 edge-case Expected Behavior text (Overlay shape: `Overlay { stack: empty, prior }` →
  `Overlay { prior }` with adjacent `App.overlay_stack` note). Behavioral semantics of AC-008 (Esc is no-op identity)
  are unchanged — no body edits to AC-008 required.
- SE-16d monotonicity: v1.6 timestamp 2026-05-28T16:00:00Z >= v1.5 timestamp 2026-05-28T00:00:00Z. PASS.

## §Trace v1.5

**F-S025-ADV4-BLOCKER-001 + BLOCKER-002 propagation** (2026-05-28):
- BC-2.06.016 pin v1.0.6 → v1.0.7 (Overlay shape sweep — no body changes required).
- SE-16d monotonicity: v1.5 timestamp 2026-05-28 >= v1.4 timestamp 2026-05-28. PASS.

## §Trace v1.4

**F-S025-ADV3-BLOCKER-002 — SS-06 BC version pins propagated from PO sweep (commit 6d4fbb3)** (2026-05-28):
- BC-2.06.008 inputs pin updated: v1.0.0 → v1.1.0.
- BC-2.06.009 inputs pin updated: v1.0.0 → v1.1.0.
- BC-2.06.011 inputs pin updated: v1.1.0 → v1.2.0.
- BC-2.06.012 inputs pin updated: v1.1.0 → v1.2.0.
- BC-2.06.013 inputs pin updated: v1.1.0 → v1.2.0.
- BC-2.06.014 inputs pin updated: v1.0.0 → v1.0.6.
- BC-2.06.016 inputs pin updated: v1.0.0 → v1.0.6.
- BC-2.06.023 inputs pin updated: v1.0.0 → v1.4.0.
- BC-2.06.024 inputs pin updated: v1.0.0 → v1.0.1.
- AC-001 body updated: `Overlay { stack: app.overlay_stack.clone(), prior: current_focus }` →
  `Overlay { prior: current_focus }` with explicit note that the modal stack lives in
  `App.overlay_stack` not the `Overlay` variant (BC-2.06.004 v1.2.0 PC-2 propagation).
- Downstream Consumer Contract: `Overlay { stack: empty, .. }` → `Overlay { .. }` with
  `App.overlay_stack` empty (same shape correction).
- SE-16d monotonicity: v1.4 timestamp 2026-05-28 >= v1.3 timestamp 2026-05-28. PASS.

## §Trace v1.3

**F-S022-ADV8-HIGH-001 — BC-2.05.002 Invariant 4 dedup directive propagated** (2026-05-28):
- Finding: Pass 6 architect's Option D decision (dedup-on-insert for `PermissionPromptQueued`)
  was named at the story level in architect-decisions-pass-6.md §Implementer Directive but was
  never propagated into S-026 story content. CLAUDE.md Principle 3 violation — the deferral
  was functionally orphaned.
- Fix: BC-2.05.002 added to `behavioral_contracts` frontmatter and `inputs` list (v1.0.5).
- Fix: AC-001 updated with idempotent-insert precondition: streaming `PermissionPromptQueued`
  handler MUST use `apply_permission_prompt_queued` helper; if `prompt_id` already present,
  silently discard at TRACE level. Per BC-2.05.002 Invariant 4.
- Fix: Tasks section updated — `apply_permission_prompt_queued` usage task added for streaming
  handler; `test_snapshot_window_prompt_dedup` integration test task added per
  architect-decisions-pass-6.md §Implementer Directive step 3.
- Token Budget: BC-2.05.002.md row added (~300 tokens); total updated ~14,400 → ~14,700.
- `traces_to` frontmatter updated to include BC-2.05.002 Invariant 4.
- SE-16d monotonicity: v1.3 timestamp 2026-05-28 >= v1.2 timestamp 2026-05-27. PASS.

## §Trace v1.2

**Phase 2 Adversarial Review Pass 2 — AC re-anchoring, rotation ACs, conversion AC, InitialState fixes** (2026-05-27):
- FIX-1 (CRITICAL, F-P2ADV-P2-001/002/009/010): All ACs re-anchored to correct BC traces:
  - AC-001 → BC-2.06.008 PC-1 (overlay push); AC-002 → BC-2.06.008 PC-2 (FIFO ordering)
  - AC-003 → BC-2.06.011 PC-1 (Accept-Once); AC-004 → BC-2.06.012 PC-1 (Accept-Always)
  - AC-005 → BC-2.06.013 PC-1 (Reject); AC-006 → BC-2.06.023 PC-1 (UUID removal via retain)
  - AC-007 → BC-2.06.023 PC-3 (unknown prompt_id no-op); AC-008 → BC-2.06.014 PC-1 (Esc identity)
  - AC-009 → BC-2.06.003 (SearchPrompt layer bindings); AC-010 → BC-2.06.003 (binding isolation)
  - AC-011 → BC-2.06.016 PC-1 (disconnect clear); AC-012 → BC-2.06.016 PC-2 (reconnect restore)
  - AC-015 → BC-2.06.023 PC-4 (empty-stack collapse to Dashboard)
- FIX-1 (continued): Old AC-013 (duplicate of AC-006 PermissionPromptResolved removal) MERGED into AC-006.
  Old AC-014/AC-015 renumbered: AC-014 (retain semantics) merged into AC-006; AC-015 (received_at) folded into AC-016.
  Net AC count: 15 (was 15, but restructured with 2 new rotation ACs + 1 conversion AC replacing merged duplicates).
- FIX-2 (CRITICAL): AC-013/AC-014 added for BC-2.06.009 stack rotation behavior and EC-065 single-item no-op.
- FIX-3 (HIGH, F-P2ADV-P2-005): All 7 occurrences of `FullState` replaced with `InitialState` throughout body.
- FIX-4 (HIGH, F-P2ADV-P2-007): AC-016 added for `payload_to_modal()` conversion spec (BC-2.06.024 + BC-2.06.008 PC-1).
- Frontmatter: BC-2.06.024 added to behavioral_contracts and inputs; BC-2.06.011/012/013 input versions bumped 1.0.0→1.1.0; traces_to updated.
- Token Budget updated: BC-2.06.024.md row added; total ~13,000→~14,400.
- Tasks section updated: rotation handler, payload_to_modal(), overlay_rotation.rs test file added.
- Version bumped v1.1→v1.2.
