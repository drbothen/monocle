---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:02:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.009: Hook Endpoint: Stop/SessionStart/PromptSubmit Routing (300ms Timeout)

## Description

Three hook endpoints — `POST /hooks/stop`, `POST /hooks/session-start`, and
`POST /hooks/prompt-submit` — share an identical routing contract with a 300ms timeout
budget. Like PreToolUse (BC-2.04.007), these are time-critical hooks where Claude Code
blocks execution until it receives a response; unlike PreToolUse, they do not support
`HookDecision::Defer` (no permission overlay is queued for Stop, SessionStart, or
PromptSubmit). Each endpoint follows the same middleware pipeline (body size limit, auth,
body extractor), dispatches to `EngineModule::on_hook()`, publishes to the event bus, appends
a JSONL record, and returns an appropriate `HookResponse` within the 300ms budget.

## Preconditions

1. The HTTP request has arrived at one of the three axum routes:
   `POST /hooks/stop`, `POST /hooks/session-start`, or `POST /hooks/prompt-submit`.
2. `body_size_limit_middleware` has verified `Content-Length ≤ 262144` bytes
   (BC-2.01.003 postcondition satisfied).
3. `auth_middleware` has verified the auth token (BC-2.01.009 postcondition satisfied).
4. At least one `EngineModule` is registered in `DaemonState.engine_registry`
   (guaranteed by BC-2.04.001 PC-6).
5. `DaemonState.event_bus_tx` (`Arc<EventBusTx>`) is initialized (BC-2.04.001 PC-5).
6. `DaemonState.ring` (`Arc<RingBuffer>`) is initialized (BC-2.04.001 PC-4).

## Postconditions

**PC-1 — JSON deserialization.**
Each handler deserializes its request body into the appropriate `HookEnvelope` variant for
the hook type (Stop, SessionStart, or UserPromptSubmit per monocle-proto schema). If
deserialization fails, the handler returns HTTP 422 with
`{"error": "invalid_body", "message": "<serde error>"}`. No event bus publish or ring
append occurs on deserialization failure.

**PC-2 — Session registry update.**
The handler extracts `HookEnvelope.session_id` and looks up or creates an `EnrichedSession`.
For `Stop` events specifically, the handler additionally marks the session as
`SessionState::Stopped` in the registry after `EngineModule::on_hook()` returns, so that
subsequent hooks from this session (if any arrive before cleanup) are handled correctly.

**PC-3 — EngineModule dispatch (no Defer).**
The handler calls `EngineModule::on_hook(hook_type, session_id, &payload)` where `hook_type`
is `HookType::Stop`, `HookType::SessionStart`, or `HookType::UserPromptSubmit` respectively.
`HookDecision::Defer` is NOT valid for these hook types. If a module returns `Defer`, the
handler treats it as `Allow` and logs:
`WARN: invalid Defer on <hook_type> hook; treating as Allow (module=<name>)`.

**PC-4 — 300ms timeout budget enforcement.**
The entire handler body is wrapped in `tokio::time::timeout(Duration::from_millis(300), ...)`.
If the timeout fires, the handler returns HTTP 200 with `{"decision": "allow"}` (fail-open
for non-permission hooks). Timeout is logged at WARN level:
`WARN: <hook_type> handler timeout (session_id=<id>)`.

**PC-5 — Event bus publish (best-effort, non-blocking).**
The handler calls `DaemonState.event_bus_tx.try_send(event)` after obtaining a
`HookDecision`. On `Err(TrySendError::Full)`:
  a. `DaemonState.drop_counter` (AtomicU64) incremented by 1.
  b. `WARN: event bus full; dropping event (drop_count=<N>)` logged.
  c. Event discarded. HTTP response not affected.

**PC-6 — JSONL ring append (best-effort).**
The handler calls `DaemonState.ring.append(record)`. I/O failures are logged at WARN and
do not affect the HTTP response. In non-failure paths, every received event is submitted
to the ring (DI-001).

**PC-7 — HTTP 200 response.**
All three handlers return HTTP 200 with a JSON response body. Response body semantics:
- **Stop:** `{"decision": "allow"}` always. Stop is informational; monocle cannot block
  session termination.
- **SessionStart:** `{"decision": "allow"}` always. Session start is informational.
- **PromptSubmit (UserPromptSubmit):** `{"decision": "allow"}` for Allow; `{"decision":
  "block", "reason": "<reason>"}` for Block. Block on PromptSubmit causes Claude Code to
  reject the user's prompt with the given reason. Timeout produces `{"decision": "allow"}`
  (fail-open for PromptSubmit; not blocking in a security context).

## Invariants

1. The 300ms timeout budget (PC-4) is absolute for all three hook types. No code path may
   delay the HTTP response beyond 300ms from request arrival at the handler layer.
2. `HookDecision::Defer` is forbidden for Stop, SessionStart, and PromptSubmit. No
   `oneshot::channel` is created; no `PermissionPromptQueued` IPC is sent.
3. Event bus saturation (PC-5) NEVER blocks any of the three handlers.
4. The Stop handler MUST update the session's `SessionState` to `Stopped` after dispatch
   (PC-2 addendum). This state transition enables the TUI to render sessions as terminated
   in the sessions panel.
5. For Stop and SessionStart, the HTTP response body is always `{"decision": "allow"}`
   regardless of `HookDecision` variant. These are lifecycle notifications; monocle cannot
   veto them.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-082 | `Stop` event arrives for an unknown `session_id` (session never registered) | A new `EnrichedSession` is created in `Stopped` state; event published and ring append attempted; HTTP 200 returned |
| EC-083 | `Stop` event arrives immediately after `SessionStart` for the same session_id | Session transitions from `SessionState::Starting` (or `Active`) to `Stopped`; both events published to event bus; TUI receives both IPC messages |
| EC-084 | `PromptSubmit` with `HookDecision::Block { reason: "disallowed content" }` | HTTP 200 `{"decision": "block", "reason": "disallowed content"}`; event published to bus; Claude Code rejects the prompt |
| EC-085 | Module returns `HookDecision::Defer` for `SessionStart` | Treated as `Allow`; WARN logged; no oneshot channel; HTTP 200 `{"decision": "allow"}` |
| EC-086 | `PromptSubmit` handler times out at 300ms | HTTP 200 `{"decision": "allow"}` (fail-open); WARN logged; no block imposed on the user's prompt |
| EC-087 | Three concurrent Stop events for three different sessions | Each handled in its own tokio task; session registry locks per-session; no cross-session contamination; all three return HTTP 200 within 300ms |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid Stop JSON, auth OK, module returns `Allow` | HTTP 200, `{"decision": "allow"}`, session state = Stopped, event published, ring record appended | happy-path |
| Valid SessionStart JSON, auth OK, module returns `Allow` | HTTP 200, `{"decision": "allow"}`, new EnrichedSession created, event published | happy-path |
| Valid PromptSubmit JSON, auth OK, module returns `Block { reason: "X" }` | HTTP 200, `{"decision": "block", "reason": "X"}`, event published | happy-path |
| Malformed JSON body for any of the three hook types | HTTP 422, `{"error": "invalid_body", ...}`, no event, no ring record | error |
| PromptSubmit module stalls 310ms (timeout) | HTTP 200, `{"decision": "allow"}` (fail-open), WARN logged | edge-case |
| Stop event for unknown session_id | HTTP 200, new EnrichedSession created in Stopped state, event published | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Stop handler marks session as Stopped in registry | unit |
| VP-TBD | PromptSubmit returns Block response when module blocks | unit |
| VP-TBD | All three handlers return HTTP 200 within 300ms under normal load | integration |
| VP-TBD | Timeout on PromptSubmit produces fail-open allow response | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — this BC defines the wiring of the Stop, SessionStart, and PromptSubmit hook endpoints (three of the five hook types) through the composition root's routing layer to EngineModule, event bus, and ring, which are core composition-root responsibilities of CAP-004 |
| L2 Domain Invariants | DI-001 (every hook event received MUST be written to the JSONL ring — PC-6 implements ring append; best-effort only for I/O failures); DI-005 (auth token prefix enforcement — upstream auth middleware, Precondition 3) |
| Architecture Module | monocle-runtime (hook handlers, axum router) per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.0.0 §Hook Endpoint Routing |
| Cross-Ref | BC-2.01.003 (body size limit — upstream); BC-2.01.009 (auth — upstream); BC-2.03.001 (EngineModule trait); BC-2.04.007 (PreToolUse routing — sibling, same 300ms budget); BC-2.04.008 (Notification routing — sibling, 2000ms budget); BC-2.04.011 (event bus) |
| Test File | `monocle-runtime/tests/hook_routing_stop_session_prompt.rs` |
| Test Name | `test_BC_2_04_009_stop_session_prompt_routing` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.007] — composes with: PreToolUse routing shares the 300ms budget; differs in Defer support and fail-closed vs fail-open timeout behavior
- [BC-2.04.008] — composes with: Notification uses 2000ms budget; same middleware stack and event bus pattern
- [BC-2.04.011] — depends on: bounded event bus (PC-5)
- [BC-2.01.003] — depends on: body size limit middleware
- [BC-2.01.009] — depends on: auth middleware
- [BC-2.03.001] — depends on: EngineModule trait definition

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#hook-endpoint-routing` — routing stack, 300ms budget for Stop/SessionStart/PromptSubmit
- `architecture/SS-daemon-wiring.md#bounded-event-bus` — try_send non-blocking semantics

## Story Anchor

S-TBD — Implement Stop/SessionStart/PromptSubmit hook routing handlers with 300ms timeout (filled by story-writer)

## VP Anchors

- VP-TBD — filled after VP creation

## §Trace v1.0.0

**Initial production** (2026-05-26T12:02:00Z):
- BC-2.04.009 created as new artifact for SS-04 §Hook Endpoint Routing per task instruction.
- Covers: Stop, SessionStart, PromptSubmit hook types; 300ms timeout; no-Defer invariant;
  Stop-specific session state transition to Stopped; PromptSubmit Block response surfacing;
  fail-open timeout semantics; event bus and ring append.
- Capability anchor: CAP-004 per ARCH-INDEX §SS-04 Capability Traceability row.
- SE-16d PASS: 2026-05-26T12:02:00Z > chain prior 2026-05-26T12:01:00Z. PASS.
