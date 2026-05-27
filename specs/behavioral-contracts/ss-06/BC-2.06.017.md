---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T18:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: []
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
(`decision_required: true`) until either (a) the TUI user sends a `DecisionResponse` via
IPC, or (b) the hook type's timeout ceiling is reached. The TUI's obligation is to render
the overlay within 100ms of receiving `PermissionPromptQueued` (the TUI render budget),
leaving the remaining budget for user keypress and the IPC decision response path. On
timeout, the daemon applies fail-open (Allow) semantics for `PreToolUse`, `Stop`,
`SessionStart`, and `UserPromptSubmit` hooks, and the `Notification` hook never defers
(no decision required). The TUI has no timeout logic itself: it presents the overlay,
waits for user input, and sends the decision as fast as the user acts.

## Preconditions

1. The daemon received a hook POST with `decision_required: true` and is holding the HTTP
   response open (the axum handler has not yet returned a response to Claude Code).
2. The TUI has received `IpcServerMessage::PermissionPromptQueued` and is rendering
   `AppMode::Overlay`.
3. The hook type's timeout ceiling is defined:

   | Hook type | Timeout ceiling | Fail-open/fail-closed |
   |-----------|----------------|----------------------|
   | PreToolUse | 300ms | Fail-open (Allow) — per BC-HOOK-001 |
   | Stop | 300ms | Fail-open (Allow) |
   | SessionStart | 300ms | Fail-open (Allow) |
   | UserPromptSubmit | 300ms | Fail-open (Allow) |
   | Notification | 2000ms | N/A — Notification never defers; always acknowledged immediately |

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
   (`[1]`, `[2]`, `[3]`) in `AppMode::Overlay`, the TUI sends `IpcClientMessage::DecisionResponse`
   to the daemon immediately via the non-blocking IPC send channel. The send is fire-and-forget
   from the TUI's perspective; the daemon correlates by `prompt_id`.

3. **Daemon timeout enforcement: not in TUI scope.** The daemon is responsible for
   starting a `tokio::time::timeout` when it begins holding the HTTP response open. The TUI
   has no timer. The daemon closes the held response (with the timeout-default decision) if
   the clock expires before a `DecisionResponse` arrives. The TUI is not notified of timeout
   expiry; it simply observes that the associated `PromptModal` is no longer queued on the
   next daemon push.

4. **On PreToolUse timeout: daemon returns fail-open (Allow).** The daemon sends
   `{"decision": "allow"}` (or equivalent permissive response) to the stalled Claude Code
   HTTP connection. This matches the gene-source canonical behavior in BC-HOOK-001.

5. **On Stop/SessionStart/UserPromptSubmit timeout: daemon returns fail-open (Allow).**
   Same fail-open behavior as PreToolUse. These hook types share the 300ms ceiling.

6. **Notification hooks are never deferred.** A `Notification` hook POST is acknowledged
   immediately by the daemon (HTTP 200) without waiting for a TUI decision. `Notification`
   events are informational only; they appear in the Event Ribbon (BC-2.06.018) but do NOT
   generate a `PermissionPromptQueued` IPC message and do NOT push a `PromptModal`.

7. **Modal does not linger after timeout.** When the daemon times out and auto-resolves a
   `PromptModal`, the daemon removes it from `DaemonState::queued_prompts` and pushes an
   `IpcServerMessage::PromptAutoResolved { prompt_id }` (or equivalent state update) to the
   TUI. The TUI removes the corresponding `PromptModal` from the local `VecDeque` and
   transitions to `Dashboard` if the stack becomes empty.

## Invariants

1. The TUI never starts a timeout timer for a `PromptModal`. Timeout enforcement is
   exclusively the daemon's responsibility. The TUI is a presenter, not a time arbiter.
2. The TUI render budget (≤100ms from IPC receive to screen paint) is a non-functional
   upper bound, not a hard real-time guarantee. Under high CPU load or terminal resize
   events, frames may be delayed. The Success Criterion applies to steady-state operation
   on localhost with a 60fps draw loop (16ms tick).
3. Fail-open is the default for all hook types with decision semantics (`PreToolUse`,
   `Stop`, `SessionStart`, `UserPromptSubmit`). Fail-closed (Deny) is never applied on
   timeout without explicit configuration (Phase 2 feature).
4. The 300ms / 2000ms ceilings are sourced from BC-HOOK-022 (gene-source canonical
   timeouts). monocle does not define or override these values; it operates within them.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-105 | User presses decision key (`[1]`) at 299ms into a 300ms budget | Decision sent to daemon at ~299ms; daemon receives it just before timeout; daemon resolves with user's decision (not fail-open); Both TUI and Claude Code proceed correctly |
| EC-106 | User is mid-read of the diff preview when 300ms elapses | Daemon times out; sends fail-open to Claude Code; pushes `PromptAutoResolved` to TUI; TUI removes `PromptModal` from stack; TUI transitions to Dashboard if empty; user sees prompt disappear from overlay |
| EC-107 | Two prompts in overlay; first prompt times out while user is cycling to view the second | Daemon resolves P1 fail-open; TUI receives `PromptAutoResolved { prompt_id: P1 }`; TUI removes P1 from `VecDeque`; P2 is now at the front; overlay re-renders showing P2 only |
| EC-108 | `Notification` hook arrives while overlay is open (PreToolUse pending) | Daemon acknowledges Notification immediately (no `PermissionPromptQueued`); Notification appears in Event Ribbon (BC-2.06.018); overlay is NOT affected; PreToolUse prompt remains in stack |
| EC-109 | IPC send channel is at capacity when user sends decision | `ipc_tx.try_send()` returns `Err(Full)`; TUI logs `tracing::warn!("IPC send channel full; decision dropped")` and increments drop counter; daemon does NOT receive the decision; daemon times out and applies fail-open; drop counter increments in status bar |
| EC-110 | TUI render takes >100ms due to large diff in PromptModal (e.g., 500-line Edit diff) | Render exceeds budget; Success Criterion violated; the `similar::TextDiff` computation is synchronous in the draw loop — large diffs must be truncated or paginated. Implementation must cap rendered diff lines to `(overlay_height - 8)` per SS-tui.md §Diff Preview to bound render time |
| EC-111 | User opens TUI after daemon has already timed out a prompt | New TUI process receives initial state push; daemon's `queued_prompts` no longer contains the timed-out prompt; TUI renders with the remaining queue (possibly empty) |

## Canonical Test Vectors

| Scenario | Input | Expected Output | Category |
|----------|-------|----------------|----------|
| Decision within budget | `PermissionPromptQueued` at T=0; user presses `[1]` at T=50ms | `DecisionResponse { decision: accept }` sent at T=50ms; Claude Code unblocks | happy-path |
| PreToolUse timeout | `PermissionPromptQueued` at T=0; no user input; daemon timeout at T=300ms | Daemon sends fail-open; `PromptAutoResolved` pushed to TUI; overlay cleared; Dashboard shown | edge-case |
| Notification — no defer | `HookEventReceived { hook_type: Notification }` | No `PermissionPromptQueued`; event appears in ribbon; no overlay | happy-path |
| Render within 100ms | Inject `PermissionPromptQueued`; measure time to first `draw()` containing overlay | ≤100ms elapsed | performance |
| IPC send channel full | Inject full `ipc_tx` channel; user presses `[1]` | Warn logged; drop counter incremented; no panic; daemon applies fail-open on timeout | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | TUI renders overlay within 100ms of `PermissionPromptQueued` receipt (steady-state localhost) | performance test (measure draw latency with synthetic IPC injection) |
| VP-TBD | TUI sends `DecisionResponse` on user keypress without artificial delay | unit test (assert send channel has message immediately after action dispatch) |
| VP-TBD | `Notification` hooks do not produce `PermissionPromptQueued` messages | integration test |
| VP-TBD | On `PromptAutoResolved`, TUI removes the corresponding `PromptModal` from the stack | unit test |
| VP-TBD | TUI has no `tokio::time::timeout` or timer for prompt decisions | static analysis / code review |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the timing contract for the "permission overlay stack" decision path, directly operationalizing the hook ingestion timeout budget Success Criterion |
| L2 Domain Invariants | DI-001 (every hook event received MUST be written to the JSONL ring before acknowledgement — this BC's timing constraints are compatible: the daemon writes to the ring before opening the hold, not after the decision returns); DI-007 (monocle MUST NOT write to any file owned by a harness — satisfied: decision is sent via IPC only) |
| Architecture Module | monocle-tui (overlay render timing, IPC send on decision); monocle-runtime (daemon hold + timeout enforcement) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.0.0 §Hook Timeout Budget (BC-2.06.017); §Permission Overlay §Overlay Stack Lifecycle steps 2–3 (decision send path); §Rendering Architecture §Draw Loop (16ms tick, ~60fps) |
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
- Postcondition 7 specifies `PromptAutoResolved` as the mechanism by which the TUI learns
  of a timeout expiry — the TUI is notified, not left with a stale modal.
