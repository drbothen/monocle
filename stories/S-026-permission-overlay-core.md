---
document_type: story
level: L4
story_id: S-026
epic_id: EPIC-06
version: "1.0"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 13
wave: 6
tdd_mode: strict
priority: P0
depends_on: [S-024, S-022, S-023]
blocks: [S-027, S-029]
target_module: monocle-tui
subsystems: [SS-06]
behavioral_contracts: [BC-2.06.008, BC-2.06.009, BC-2.06.011, BC-2.06.012, BC-2.06.013, BC-2.06.014, BC-2.06.016, BC-2.06.023]
verification_properties: []
estimated_days: 5
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.008.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.009.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.011.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.012.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.013.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.014.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.016.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.023.md, version: "1.0.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.06.008..009 (overlay push/pop), BC-2.06.011..014 (permission decision dispatch), BC-2.06.016 (disconnect clear), BC-2.06.023 (UUID removal)"
---

# S-026: Permission Overlay Core — Push/Pop, Decision Dispatch, Disconnect Clear, UUID Removal

## Narrative

As a daemon operator, I want the TUI to display permission prompts in a FIFO overlay
stack, allow me to approve or reject each prompt via keyboard shortcuts, and automatically
clear the overlay when the daemon disconnects, so that I can manage tool permissions
without losing any pending prompts and without manual cleanup after disconnects.

## Acceptance Criteria

### AC-001 (traces to BC-2.06.008 postcondition PC-1 — PermissionPromptQueued push)
When `ServerToClient::PermissionPromptQueued { prompt_id, session_id, tool_name, tool_payload }`
is received, the TUI constructs a `PromptModal` (with `received_at: Instant::now()`) and
calls `app.overlay_stack.push_back(modal)`. The TUI then transitions to
`Overlay { stack: app.overlay_stack.clone(), prior: current_focus }` via `transition()`
if not already in Overlay mode. If already in Overlay mode, the stack grows in place
(the IPC push is NOT routed through `transition()`).

### AC-002 (traces to BC-2.06.008 postcondition PC-2 — FIFO order)
`PromptModal` items are served in FIFO order. The oldest (first received) prompt is
`overlay_stack.front()` and is displayed as the active prompt. New prompts are appended
via `push_back`. This ordering is preserved across reconnects (the daemon's
`overlay_stack` in `FullState` preserves insertion order).

### AC-003 (traces to BC-2.06.009 postcondition PC-1 — y/Enter accepts front prompt)
In `Overlay` mode, pressing `y` or `Enter` sends
`ClientToServer::PermissionDecision { prompt_id: front.prompt_id, decision: PermissionDecision::Accept }`
over the IPC connection. The TUI does NOT pop the modal immediately; it waits for
`ServerToClient::PermissionPromptResolved { prompt_id }` before removing it from the stack.

### AC-004 (traces to BC-2.06.009 postcondition PC-2 — A sends AcceptAlways)
In `Overlay` mode, pressing `A` (uppercase) sends
`ClientToServer::PermissionDecision { prompt_id: front.prompt_id, decision: PermissionDecision::AcceptAlways }`.
Same round-trip semantics as AC-003 — wait for `PermissionPromptResolved` before pop.

### AC-005 (traces to BC-2.06.009 postcondition PC-3 — n/r rejects front prompt)
In `Overlay` mode, pressing `n` or `r` sends
`ClientToServer::PermissionDecision { prompt_id: front.prompt_id, decision: PermissionDecision::Reject }`.
Same round-trip semantics — wait for `PermissionPromptResolved` before pop.

### AC-006 (traces to BC-2.06.011 postcondition PC-1 — PermissionPromptResolved pop)
When `ServerToClient::PermissionPromptResolved { prompt_id }` is received, the TUI
removes the `PromptModal` whose `prompt_id` matches (using `retain()` to find it
regardless of stack position, not assuming it is at front). After removal, if the
stack is now empty, `transition()` collapses to `Dashboard { focused: prior }` via the
empty-stack collapse invariant.

### AC-007 (traces to BC-2.06.011 postcondition PC-2 — unknown prompt_id is no-op)
If `PermissionPromptResolved { prompt_id }` refers to a `prompt_id` not in the local
`overlay_stack`, the TUI logs a WARN and takes no further action. This handles the case
where the daemon resolved a prompt that the TUI was not aware of (e.g., auto-resolved
before connect).

### AC-008 (traces to BC-2.06.012 postcondition PC-1 — Esc in Overlay is no-op)
Pressing `Esc` while in `Overlay` mode is a no-op identity: `transition(Overlay{..}, Action::Esc)`
returns the same `Overlay` state unchanged. Esc NEVER rejects a prompt, NEVER pops the
stack, NEVER exits overlay mode.

### AC-009 (traces to BC-2.06.013 postcondition PC-1 — overlay keyboard bindings)
The `SearchPrompt` binding layer registers the overlay decision keys (`y`, `Enter`, `A`,
`n`, `r`) with highest priority when mode is `Overlay`. These bindings override any
Global or PerContext layers for these keys while a prompt is displayed.

### AC-010 (traces to BC-2.06.014 postcondition PC-1 — overlay blocks session navigation)
While in `Overlay` mode, session list navigation keys (`j`, `k`, `Tab`, `Enter` on
sessions) are consumed by the overlay. They do NOT scroll the sessions list behind the
overlay. Only overlay-specific keys (`y`, `Enter`, `A`, `n`, `r`, `Esc`) are active.

### AC-011 (traces to BC-2.06.016 postcondition PC-1 — disconnect clears overlay)
When `TransportEvent::Disconnected` is received, the TUI MUST clear `app.overlay_stack`
(set to `VecDeque::new()`) and transition to `Dashboard { focused: default_focus }`.
Any pending `PermissionDecision` sends that were in-flight are abandoned (no retry).
This satisfies SOQ-3.

### AC-012 (traces to BC-2.06.016 postcondition PC-2 — overlay restored on reconnect)
On reconnect, `ServerToClient::FullState { overlay_stack, .. }` re-populates the
TUI's local stack. If the daemon still has pending prompts after the TUI reconnected,
the overlay is re-entered. See S-023 for reconnect mechanics.

### AC-013 (traces to BC-2.06.023 postcondition PC-1 — UUID removal via retain)
When `ServerToClient::PermissionPromptResolved { prompt_id }` is received (same as
AC-006), the TUI uses `overlay_stack.retain(|m| m.prompt_id != prompt_id)` to remove
the resolved modal. This is NOT routed through `transition()` — it is a direct mutation
of the VecDeque followed by a stack-collapse check (if stack is now empty, call
`transition(current_mode, Action::PopOverlay)` or equivalent to trigger the
empty-stack collapse path).

### AC-014 (traces to BC-2.06.023 invariant INV-1 — retain semantics)
`retain()` removes ALL entries matching the `prompt_id` (in case of duplicates from
reconnect races), not just the first. After `retain()`, the stack contains no modals
with the given `prompt_id`.

### AC-015 (traces to BC-2.06.008 invariant INV-1 — PromptModal received_at)
`PromptModal.received_at` is set to `Instant::now()` at the moment the TUI processes
the `PermissionPromptQueued` message, NOT when the daemon created the prompt. This is
used for timeout display in S-027.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~3,000 |
| BC-2.06.008.md | ~900 |
| BC-2.06.009.md | ~900 |
| BC-2.06.011.md | ~800 |
| BC-2.06.012.md | ~700 |
| BC-2.06.013.md | ~700 |
| BC-2.06.014.md | ~700 |
| BC-2.06.016.md | ~800 |
| BC-2.06.023.md | ~700 |
| S-024 (transition, AppMode) | ~700 |
| S-022 (IPC types) | ~600 |
| S-023 (reconnect, SOQ-3) | ~500 |
| Test files | ~2,000 |
| **Total estimate** | **~13,000** |

## Tasks

- [ ] Implement `PermissionPromptQueued` IPC handler in `monocle-tui/src/app.rs`:
      construct `PromptModal`, `push_back`, enter/grow Overlay via `transition()` or direct
- [ ] Implement `PermissionPromptResolved` IPC handler: `retain()` on `overlay_stack`,
      trigger empty-stack collapse check, log WARN on unknown prompt_id
- [ ] Implement keyboard handler for `y`/`Enter` → `PermissionDecision::Accept` IPC send
- [ ] Implement keyboard handler for `A` → `PermissionDecision::AcceptAlways` IPC send
- [ ] Implement keyboard handler for `n`/`r` → `PermissionDecision::Reject` IPC send
- [ ] Implement `TransportEvent::Disconnected` handler: clear `overlay_stack`, transition to `Dashboard`
- [ ] Implement `ServerToClient::FullState` handler (extend S-025): re-populate `overlay_stack` from `full_state.overlay_stack`
- [ ] Register overlay decision bindings (`y`, `Enter`, `A`, `n`, `r`) in `SearchPrompt` layer of `BindingLayers`
- [ ] Verify Esc in Overlay is a no-op (identity) — no action dispatched, no stack change
- [ ] Verify overlay keyboard bindings block session nav keys while in Overlay mode
- [ ] Unit tests `monocle-tui/tests/overlay_push_pop.rs` — FIFO push/pop, empty-stack collapse, FIFO ordering
- [ ] Unit tests `monocle-tui/tests/overlay_decision.rs` — Accept/AcceptAlways/Reject IPC send, wait-for-resolved semantics
- [ ] Unit tests `monocle-tui/tests/overlay_disconnect.rs` — clear on disconnect, restore on reconnect via FullState
- [ ] Unit tests `monocle-tui/tests/overlay_uuid_removal.rs` — retain() semantics, duplicate removal, unknown prompt_id no-op

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

From `architecture/SS-tui-core.md`:
- IPC push path (`PermissionPromptQueued`) is NOT routed through `transition()` —
  push directly to `app.overlay_stack`, then update AppMode
- UUID removal via `retain()` is NOT through `transition()` — direct mutation + collapse check
- `transition(Overlay{..}, Esc)` is identity — never pop, never reject
- `ClientToServer::PermissionDecision` — NOT `PermissionResponse` or any other name
- `PermissionDecision::Accept` | `AcceptAlways` | `Reject` — exact variant names
- `ServerToClient::PermissionPromptResolved { prompt_id }` — NOT `PermptResolved` or abbreviated
- `TransportEvent::Disconnected` — NOT `IpcServerMessage::DaemonDisconnect` (doesn't exist)
- `overlay_stack` is the IPC field name in `FullState` — local TUI copy is `VecDeque<PromptModal>`
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
  `TransportEvent::Disconnected` extension (clear overlay), `FullState` extension (overlay restore)
- `ui/mod.rs` — prepare overlay rendering module placeholder (S-027 fills it in)

Files to create:
- `monocle-tui/tests/overlay_push_pop.rs` — push/pop/FIFO/collapse tests
- `monocle-tui/tests/overlay_decision.rs` — decision dispatch + resolved round-trip tests
- `monocle-tui/tests/overlay_disconnect.rs` — disconnect clear + reconnect restore tests
- `monocle-tui/tests/overlay_uuid_removal.rs` — retain() semantics tests

## Downstream Consumer Contract

Public behavior produced by this story for downstream consumption:

The `App` struct (from S-025) gains these guaranteed behaviors:
- `overlay_stack` (VecDeque<PromptModal>) correctly tracks the daemon's overlay state
- `mode` is always `Overlay { .. }` when `overlay_stack` is non-empty; never `Overlay { stack: empty, .. }`
- Decision keys dispatch correct `ClientToServer::PermissionDecision` IPC messages
- Disconnect clears overlay; reconnect restores it from `FullState`

S-027 (overlay rendering + diff preview) builds its UI atop these guaranteed behaviors.
S-029 (killer scenario integration test) validates the full round-trip.
