---
document_type: behavioral-contract
level: L3
version: "1.6.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T18:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "89b4d96"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-002, F-P1D2-010, F-P1D3-006, F-P1D7-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.017: Permission Response Within Hook Timeout Budget

## Description

The daemon holds the Claude Code HTTP response open for a hook event requiring a decision
(`decision_required: true`) until either (a) the TUI user sends a `ClientToServer::PermissionDecision` via
IPC, or (b) the hook type's timeout ceiling is reached. The TUI's obligation is to render
the overlay within 100ms of receiving `PermissionPromptQueued` (the TUI render budget),
leaving the remaining budget for user keypress and the IPC decision response path. Only
`PreToolUse` has decision-relevant timeout semantics (fail-open / Allow on timeout). `Stop`,
`SessionStart`, and `UserPromptSubmit` are fire-and-forget hooks with no decision semantics:
on timeout, the daemon closes the HTTP connection without sending a decision payload.
`Notification` is also fire-and-forget (no decision required). The TUI has no timeout logic
itself: it presents the overlay, waits for user input, and sends the decision as fast as the
user acts.

## Preconditions

1. The daemon received a hook POST with `decision_required: true` and is holding the HTTP
   response open (the axum handler has not yet returned a response to Claude Code).
2. The TUI has received `ServerToClient::PermissionPromptQueued` and is rendering
   `AppMode::Overlay`.
3. The hook type's timeout ceiling is defined:

   | Hook type | Timeout ceiling | Timeout semantics |
   |-----------|----------------|-------------------|
   | PreToolUse | 300ms | Fail-open (Allow) — decision-relevant; per BC-HOOK-001 |
   | Stop | 300ms | Fire-and-forget; timeout closes HTTP connection (no decision semantics) |
   | SessionStart | 300ms | Fire-and-forget; timeout closes HTTP connection (no decision semantics) |
   | UserPromptSubmit | 300ms | Fire-and-forget; timeout closes HTTP connection (no decision semantics) |
   | Notification | 2000ms | Fire-and-forget; never defers; always acknowledged immediately |

4. The total budget from hook POST receipt to decision HTTP response is the timeout ceiling
   for that hook type (e.g., 300ms for PreToolUse).

## Postconditions

1. **TUI render budget: ≤100ms.** From when the TUI receives `PermissionPromptQueued`
   to when the overlay is painted on screen: ≤100ms. This is the Success Criterion from
   `product-brief.md §Success Criteria`. The budget breakdown:
   - `hook POST receipt → IPC push` (daemon-to-TUI message enqueue): target ≤50ms
   - `IPC push → TUI render` (message draining + ratatui draw): target ≤50ms
   - Total TUI hop: ≤100ms

2. **Decision delivery: no artificial delay.** When the user presses a decision key
   (`[y]/[Enter]`, `[A]`, `[n]/[r]`) in `AppMode::Overlay`, the TUI sends `ClientToServer::PermissionDecision`
   to the daemon immediately via the non-blocking IPC send channel. The send is fire-and-forget
   from the TUI's perspective; the daemon correlates by `prompt_id`.

3. **Daemon timeout enforcement: not in TUI scope.** The daemon is responsible for
   starting a `tokio::time::timeout` when it begins holding the HTTP response open. The TUI
   has no timer. The daemon closes the held response (with the timeout-default decision) if
   the clock expires before a `ClientToServer::PermissionDecision` arrives. The TUI is not notified of timeout
   expiry; it simply observes that the associated `PromptModal` is no longer queued on the
   next daemon push.

4. **On PreToolUse timeout: daemon returns fail-open (Allow).** The daemon sends
   `{"decision": "allow"}` (or equivalent permissive response) to the stalled Claude Code
   HTTP connection. This matches the gene-source canonical behavior in BC-HOOK-001.

5. **On Stop/SessionStart/UserPromptSubmit timeout: daemon closes HTTP connection.**
   These hooks are fire-and-forget: they have no decision semantics. On timeout, the daemon
   closes the HTTP connection. No decision payload (allow/deny) is sent. Claude Code treats
   the closed connection as a completed hook invocation and continues. These hooks share the
   300ms ceiling but the timeout action differs from PreToolUse's fail-open response.

6. **Notification hooks are never deferred.** A `Notification` hook POST is acknowledged
   immediately by the daemon (HTTP 200) without waiting for a TUI decision. `Notification`
   events are informational only; they appear in the Event Ribbon (BC-2.06.018) but do NOT
   generate a `PermissionPromptQueued` IPC message and do NOT push a `PromptModal`.

7. **Modal does not linger after timeout.** When the daemon times out and auto-resolves a
   `PromptModal`, the daemon removes it from the pending-prompt registry (`overlay_stack`) and pushes a
   `ServerToClient::PermissionPromptResolved { prompt_id }` to the TUI. The TUI removes
   the corresponding `PromptModal` from the local `VecDeque<PromptModal>` and transitions to `Dashboard`
   if the stack becomes empty.

## Invariants

1. The TUI never starts a timeout timer for a `PromptModal`. Timeout enforcement is
   exclusively the daemon's responsibility. The TUI is a presenter, not a time arbiter.
2. The TUI render budget (≤100ms from IPC receive to screen paint) is a non-functional
   upper bound, not a hard real-time guarantee. Under high CPU load or terminal resize
   events, frames may be delayed. The Success Criterion applies to steady-state operation
   on localhost with a 60fps draw loop (16ms tick).
3. Fail-open (Allow) is the timeout default ONLY for `PreToolUse` (the sole hook type with
   decision semantics in Phase 1). `Stop`, `SessionStart`, and `UserPromptSubmit` are
   fire-and-forget: on timeout the daemon closes the HTTP connection without a decision
   payload. Fail-closed (Deny) is never applied on timeout without explicit configuration
   (Phase 2 feature).
4. The 300ms / 2000ms ceilings are sourced from BC-HOOK-022 (gene-source canonical
   timeouts). monocle does not define or override these values; it operates within them.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-105 | User presses decision key (`[y]`) at 299ms into a 300ms budget | Decision sent to daemon at ~299ms; daemon receives it just before timeout; daemon resolves with user's decision (not fail-open); Both TUI and Claude Code proceed correctly |
| EC-106 | User is mid-read of the diff preview when 300ms elapses | Daemon times out; sends fail-open to Claude Code; pushes `PermissionPromptResolved { prompt_id }` to TUI; TUI removes `PromptModal` from stack; TUI transitions to Dashboard if empty; user sees prompt disappear from overlay |
| EC-107 | Two prompts in overlay; first prompt times out while user is cycling to view the second | Daemon resolves P1 fail-open; TUI receives `PermissionPromptResolved { prompt_id: P1 }`; TUI removes P1 from `VecDeque`; P2 is now at the front; overlay re-renders showing P2 only |
| EC-108 | `Notification` hook arrives while overlay is open (PreToolUse pending) | Daemon acknowledges Notification immediately (no `PermissionPromptQueued`); Notification appears in Event Ribbon (BC-2.06.018); overlay is NOT affected; PreToolUse prompt remains in stack |
| EC-109 | IPC send channel is at capacity when user sends decision | `ipc_tx.try_send()` returns `Err(Full)`; TUI logs `tracing::warn!("IPC send channel full; decision dropped")` and increments drop counter; daemon does NOT receive the decision; daemon times out and applies fail-open; drop counter increments in status bar |
| EC-110 | TUI render takes >100ms due to large diff in PromptModal (e.g., 500-line Edit diff) | Render exceeds budget; Success Criterion violated; the `similar::TextDiff` computation is synchronous in the draw loop — large diffs must be truncated or paginated. Implementation must cap rendered diff lines to `(overlay_height - 8)` per SS-tui.md §Diff Preview to bound render time |
| EC-111 | User opens TUI after daemon has already timed out a prompt | New TUI process receives initial state push; daemon's `overlay_stack` no longer contains the timed-out prompt; TUI renders with the remaining queue (possibly empty) |

## Canonical Test Vectors

| Scenario | Input | Expected Output | Category |
|----------|-------|----------------|----------|
| Decision within budget | `ServerToClient::PermissionPromptQueued` at T=0; user presses `[y]` at T=50ms | `ClientToServer::PermissionDecision { decision: PermissionDecision::Accept }` sent at T=50ms; Claude Code unblocks | happy-path |
| PreToolUse timeout | `PermissionPromptQueued` at T=0; no user input; daemon timeout at T=300ms | Daemon sends fail-open; `PermissionPromptResolved { prompt_id }` pushed to TUI; overlay cleared; Dashboard shown | edge-case |
| Notification — no defer | `HookEventReceived { hook_type: Notification }` | No `PermissionPromptQueued`; event appears in ribbon; no overlay | happy-path |
| Render within 100ms | Inject `PermissionPromptQueued`; measure time to first `draw()` containing overlay | ≤100ms elapsed | performance |
| IPC send channel full | Inject full `ipc_tx` channel; user presses `[y]` | Warn logged; drop counter incremented; no panic; daemon applies fail-open on timeout | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | TUI renders overlay within 100ms of `PermissionPromptQueued` receipt (steady-state localhost) | performance test (measure draw latency with synthetic IPC injection) |
| VP-TBD | TUI sends `ClientToServer::PermissionDecision` on user keypress without artificial delay | unit test (assert send channel has message immediately after action dispatch) |
| VP-TBD | `Notification` hooks do not produce `PermissionPromptQueued` messages | integration test |
| VP-TBD | On `PermissionPromptResolved`, TUI removes the corresponding `PromptModal` from the stack | unit test |
| VP-TBD | TUI has no `tokio::time::timeout` or timer for prompt decisions | static analysis / code review |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the timing contract for the "permission overlay stack" decision path, directly operationalizing the hook ingestion timeout budget Success Criterion |
| L2 Domain Invariants | DI-001 (every hook event received MUST be written to the JSONL ring before acknowledgement — this BC's timing constraints are compatible: the daemon writes to the ring before opening the hold, not after the decision returns); DI-007 (monocle MUST NOT write to any file owned by a harness — satisfied: decision is sent via IPC only) |
| Architecture Module | monocle-tui (overlay render timing, IPC send on decision); monocle-runtime (daemon hold + timeout enforcement) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Hook Timeout Budget (BC-2.06.017); §Permission Overlay §Overlay Stack Lifecycle steps 2–3 (decision send path); §Rendering Architecture §Draw Loop (16ms tick, ~60fps) |
| Cross-Ref | BC-HOOK-022 (gene-source canonical timeout ceilings: 300ms/2000ms); BC-HOOK-001 (PreToolUse fail-open on timeout); BC-2.06.008 (overlay push on PermissionPromptQueued); BC-2.06.011 (Accept-Once decision send path); BC-2.06.012 (Accept-Always decision send path); BC-2.06.013 (Reject decision send path); BC-2.06.018 (Notification events appear in ribbon, not overlay) |
| Test File | `monocle-tui/tests/hook_timeout_budget.rs` |
| Test Name | `test_BC_2_06_017_permission_response_within_timeout_budget` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.008] — depends on: overlay push on `PermissionPromptQueued` is the precondition for this budget contract
- [BC-2.06.011] — composes with: Accept-Once is the user decision that terminates the hold window
- [BC-2.06.012] — composes with: Accept-Always is the user decision that terminates the hold window
- [BC-2.06.013] — composes with: Reject is the user decision that terminates the hold window
- [BC-2.06.018] — DISTINCTION: Notification hooks appear in the Event Ribbon, not the overlay; this BC specifies that Notification is never deferred
- [BC-HOOK-022] — depends on: gene-source canonical timeout ceilings that define the 300ms and 2000ms values

## Architecture Anchors

- `architecture/SS-tui.md#hook-timeout-budget` — latency budget table and TUI render ≤100ms target
- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle decision send path (steps 2–3)
- `architecture/SS-tui.md#rendering-architecture` — draw loop tick rate (~60fps = 16ms)

## Story Anchor

S-TBD — Verify hook timeout budget compliance: render latency test, decision delivery path, fail-open behavior on timeout (filled by story-writer)

## VP Anchors

- VP-TBD — Performance test: TUI renders overlay within 100ms of PermissionPromptQueued on localhost

## §Trace v1.2.0

**F-P1D3-006 HIGH — Timeout semantics corrected for non-decision hooks** (2026-05-26T14:00:00Z):
- Description updated: removed incorrect "fail-open (Allow)" claim for Stop, SessionStart,
  and UserPromptSubmit. These hooks are fire-and-forget; on timeout the daemon closes the
  HTTP connection, not send a decision payload.
- Precondition table: column "Fail-open/fail-closed" → "Timeout semantics"; Stop/SessionStart/
  UserPromptSubmit cells updated to "Fire-and-forget; timeout closes HTTP connection (no
  decision semantics)"; Notification cell updated to "Fire-and-forget; never defers; always
  acknowledged immediately."
- Postcondition 5: "fail-open (Allow)" → HTTP connection close, with clarification that no
  decision payload is sent and that Claude Code treats the closed connection as completion.
- Invariant 3: Corrected "fail-open is the default for all hook types with decision semantics"
  to name PreToolUse as the ONLY decision-semantic hook, and explicitly contrast with
  fire-and-forget behavior of Stop/SessionStart/UserPromptSubmit.
- SE-16d monotonicity: v1.2.0 timestamp 2026-05-26T14:00:00Z >= v1.1.0. PASS.

## §Trace v1.3.0

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.3.0 timestamp >= v1.2.0. PASS.

## §Trace v1.4.0

**F-P1D7-001 HIGH — Fabricated `IpcServerMessage` and `IpcClientMessage` type names replaced** (2026-05-26T00:00:00Z):
- `IpcServerMessage::PermissionPromptQueued` → `ServerToClient::PermissionPromptQueued`.
- `IpcClientMessage::DecisionResponse` → `ClientToServer::PermissionDecision`.
- Standalone `DecisionResponse` shorthand → `ClientToServer::PermissionDecision`.
- Description (PC-2 text), Postcondition 2, PC-7 text (timeout scenario), EC-109, test vector
  (Decision within budget row), VP table (send-channel row): all updated.
- SE-16d monotonicity: v1.4.0 timestamp >= v1.3.0. PASS.

## §Trace v1.6.0

**Stale keybinding references replaced throughout** (2026-05-27T00:00:00Z):
- Postcondition 2: "`[1]`, `[2]`, `[3]`" → "`[y]/[Enter]`, `[A]`, `[n]/[r]`" (canonical decision keys per BC-2.06.013 v1.1.0).
- EC-105: "`[1]`" → "`[y]`" (Accept-Once keybinding; canonical per BC-2.06.011).
- Test vector "Decision within budget": "`[1]` at T=50ms" → "`[y]` at T=50ms".
- Test vector "IPC send channel full": "`[1]`" → "`[y]`".
- SE-16d monotonicity: v1.6.0 timestamp >= v1.5.0. PASS.

## §Trace v1.6.1

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): this BC references `AppMode::Overlay` by mode name only; no `Overlay { stack }` variant shape in scope.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers hook timeout budget; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
- SE-16d monotonicity: v1.6.1 timestamp 2026-05-29T00:00:00Z > v1.6.0. PASS.

## §Trace v1.5.0

**F-FINAL-001 MEDIUM — Daemon-side `DaemonState::queued_prompts` replaced with canonical IPC field name** (2026-05-26T00:00:00Z):
- Postcondition 7: `DaemonState::queued_prompts` → pending-prompt registry (`overlay_stack`); `VecDeque` retained as TUI-side type.
- EC-111: `daemon's queued_prompts` → `daemon's overlay_stack`.
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per final bulk pin update.
- SE-16d monotonicity: v1.5.0 timestamp >= v1.4.0. PASS.

## §Trace v1.1.0

**F-P1D2-002 CRITICAL — Fabricated `PromptAutoResolved` message type replaced** (2026-05-26T00:00:00Z):
- All occurrences of `PromptAutoResolved` (and `IpcServerMessage::PromptAutoResolved`) replaced with `PermissionPromptResolved` (canonical type from SS-ipc.md v1.1.0).
- Affected locations: PC-7 (daemon push on timeout), EC-106, EC-107, canonical test vector (PreToolUse timeout row), VP table (PromptModal removal property), §Trace v1.0.0 note.
- `PermissionPromptResolved { prompt_id }` is the canonical `IpcServerMessage` variant per SS-ipc.md §ServerToClient message types. The fabricated `PromptAutoResolved` variant does not exist in the IPC spec.
- BC-2.06.023 correctly uses `PermissionPromptResolved` and is the consuming BC for this message type.

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh for files modified in this session).

SE-16d monotonicity: v1.1.0 timestamp >= v1.0.0. PASS.

## §Trace v1.0.0

**Initial production** (2026-05-26T18:00:00Z):
- BC-2.06.017 created as part of SS-06 TUI behavioral contract burst (BCs 016–022).
- Reads: SS-tui.md v1.0.0 §Hook Timeout Budget (BC-2.06.017 section); §Permission Overlay
  §Overlay Stack Lifecycle steps 2–3 (decision send); §Rendering Architecture §Draw Loop;
  prd-expansion-scope.md §3.3 BC-2.06.017 description and §4 Success Criteria Gap Closure
  (hook ingestion timeout budget row); §5.2 dependency table (BC-HOOK-022 and BC-HOOK-001
  dependencies).
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-001 cited: ring write happens before hold opens — no conflict with this timing contract.
- DI-007 cited: decision is sent via IPC, no file writes.
- EC-110 identifies the large-diff render risk and cites the SS-tui.md §Diff Preview cap
  (`overlay_height - 8`) as the production-grade mitigation.
- Invariant 1 explicitly separates timeout enforcement (daemon) from render/decision (TUI).
- Postcondition 6 explicitly specifies Notification never defers — preventing a common
  misimplementation where all hook types are treated as deferrable.
- Postcondition 7 specifies `PermissionPromptResolved` as the mechanism by which the TUI learns
  of a timeout expiry — the TUI is notified, not left with a stale modal.
