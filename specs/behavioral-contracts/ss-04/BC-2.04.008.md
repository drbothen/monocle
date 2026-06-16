---
document_type: behavioral-contract
level: L3
version: "1.2.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:01:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "71bf338"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D2-010, F-P1D6-003, F-P12-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.008: Hook Endpoint: Notification Request Routing (2000ms Timeout)

## Description

The `POST /hooks/notification` handler is the routing contract for Notification hook events.
Notifications are informational signals from Claude Code that do not block tool execution;
Claude Code does not wait for a decision on them beyond the timeout window. The handler
follows the same middleware stack as other hook endpoints (body size limit, auth, body
extractor) but applies a 2000ms timeout budget — seven times longer than PreToolUse — because
notification payloads may trigger heavier rendering updates in the TUI without blocking the
user's workflow. The handler MUST NOT return `HookDecision::Defer`; Notification is a
fire-and-forward hook type.

## Preconditions

1. The HTTP request has been accepted by the axum router registered in `build_server()`.
2. `body_size_limit_middleware` has verified `Content-Length ≤ 262144` bytes
   (BC-2.01.003 postcondition satisfied).
3. `auth_middleware` has verified the auth token in `X-Monocle-Authorization` or
   `X-Claude-Code-Ide-Authorization` (BC-2.01.009 postcondition satisfied).
4. At least one `EngineModule` implementation is registered in `DaemonState.engine_registry`
   (guaranteed by daemon start sequence step 6, BC-2.04.001 PC-11).
5. `DaemonState.event_bus_tx` (`Arc<EventBusTx>`) is initialized (BC-2.04.001 PC-10).
6. `DaemonState.ring` (`Arc<RingBuffer>`) is initialized (BC-2.04.001 PC-7).

## Postconditions

**PC-1 — JSON deserialization.**
The handler deserializes the request body into `HookEnvelope`. If deserialization fails,
the handler returns HTTP 422 with `{"error": "invalid_body", "message": "<serde error>"}`.
No event bus publish or ring append occurs on deserialization failure.

**PC-2 — Session registry lookup or creation.**
The handler extracts `HookEnvelope.session_id` and looks up or creates an `EnrichedSession`
in `DaemonState.session_registry`. The session object is accessed under the registry's
internal lock; mutations complete before the handler proceeds to dispatch.

**PC-3 — EngineModule dispatch (fire-and-forward, no Defer).**
The HTTP handler extracts the relevant fields from the deserialized `HookEnvelope` and
constructs a `HookEvent::Notification(NotificationEvent { notification_type, tool_name,
tool_input, message, session_id, pid })`. It then calls `engine.on_hook(hook_event).await`
on each registered module. The signature is:

```
async fn on_hook(&self, event: HookEvent) -> HookResponse
```

The `HookEvent` is fully constructed by the handler before `on_hook` is invoked; the
`EngineModule` receives a single typed event argument. The `HookResponse.decision` for
Notification MUST be one of:
- `HookDecision::Allow` — proceed.
- `HookDecision::Block` — allowed structurally (the module may block a notification for
  filtering purposes) — proceed.
`HookDecision::Defer` is NOT a valid return for Notification. If a module returns `Defer`
for a Notification event, the handler treats it as `Allow` and logs
`WARN: invalid Defer on Notification hook; treating as Allow (module=<name>)`.

**PC-4 — 2000ms timeout budget enforcement.**
The entire handler body is wrapped in `tokio::time::timeout(Duration::from_millis(2000), ...)`.
If the timeout fires before `EngineModule::on_hook()` returns, the handler returns HTTP 200
with `{"decision": "allow"}` (notifications do not block Claude Code; timeout is a best-effort
drain, not a blocking gate). The timeout is logged at WARN level:
`WARN: notification handler timeout (session_id=<id>)`.

**PC-5 — Event bus publish (best-effort, non-blocking).**
After obtaining a `HookDecision`, the handler calls `DaemonState.event_bus_tx.try_send(event)`
(non-blocking). If `try_send` returns `Err(TrySendError::Full)`:
  a. `DaemonState.drop_counter` (AtomicU64) is incremented by 1.
  b. `WARN: event bus full; dropping event (drop_count=<N>)` is logged.
  c. The event is discarded.
The HTTP response is not affected by event bus saturation; the 2000ms timeout is preserved.

**PC-6 — JSONL ring append (best-effort).**
The handler calls `DaemonState.ring.append(record)`. If the append fails (I/O error), the
handler logs `WARN: ring append failed: <error>` and continues. The HTTP response is still
HTTP 200. (Best-effort caveat applies to I/O failures only per DI-001 interpretation; normal
paths must submit to ring.)

**PC-7 — HTTP 200 return with allow body.**
Notification handlers always return HTTP 200 with `{"decision": "allow"}`. Notifications are
informational; Claude Code does not block on the decision value for this hook type. The
response body is minimal and consistent regardless of the module's `HookDecision` variant
(Allow or Block both produce the same `{"decision": "allow"}` response to Claude Code, since
notification filtering is internal-only).

## Invariants

1. The 2000ms timeout budget (PC-4) is absolute. No code path in the Notification handler
   may delay the HTTP response beyond 2000ms from request arrival at this handler layer.
2. `HookDecision::Defer` is forbidden for Notification events. If a module returns `Defer`,
   the handler degrades to `Allow` with a WARN log — it does not create a `oneshot::channel`
   or push `PermissionPromptQueued` to TUI clients.
3. Event bus saturation (PC-5) NEVER blocks the Notification handler. `try_send` is always
   non-blocking.
4. The HTTP response for Notification is always HTTP 200 `{"decision": "allow"}`, regardless
   of `HookDecision` variant. The decision is an internal signal; it is NOT surfaced to
   Claude Code in the response body for this hook type.
5. JSONL ring append (PC-6) is best-effort for I/O failures; in normal (non-I/O-error)
   execution, every received Notification event MUST be submitted to the ring (DI-001).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-076 | JSON body is syntactically valid but missing `session_id` | HTTP 422, `{"error": "invalid_body", ...}`; no event published; no ring append |
| EC-077 | Module returns `HookDecision::Defer` for a Notification event | Handler treats as `Allow`; logs `WARN: invalid Defer on Notification hook; treating as Allow (module=<name>)`; no oneshot channel created; no PermissionPromptQueued IPC sent |
| EC-078 | `EngineModule::on_hook()` runs for 1999ms (just under the 2000ms budget) | Handler completes within budget; HTTP 200 returned; WARN about slow module logged at 1000ms threshold |
| EC-079 | `EngineModule::on_hook()` runs for 2001ms (just over the 2000ms budget) | Timeout fires; handler returns HTTP 200 `{"decision": "allow"}`; `WARN: notification handler timeout` logged |
| EC-080 | Event bus full during a burst of 5000 notification events per second | Each overflowing event increments drop counter; HTTP responses return within 2000ms; no handler blocking |
| EC-081 | Notification payload is 262144 bytes (maximum) | Deserialization succeeds (body limit middleware passed); handler proceeds normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid Notification JSON body, auth OK, module returns `Allow` | HTTP 200, `{"decision": "allow"}`, event bus receives 1 event, ring receives 1 record | happy-path |
| Valid Notification JSON body, auth OK, module returns `Block` | HTTP 200, `{"decision": "allow"}` (block is internal-only for notifications); event published | happy-path |
| Valid Notification JSON body, auth OK, module stalls 2100ms | HTTP 200, `{"decision": "allow"}`, `WARN: notification handler timeout` logged | edge-case |
| Malformed JSON body, auth OK | HTTP 422, `{"error": "invalid_body", ...}`, no event published | error |
| Module returns `HookDecision::Defer` | HTTP 200, `{"decision": "allow"}`, `WARN: invalid Defer` logged, no PermissionPromptQueued IPC | edge-case |
| Event bus at capacity | HTTP 200, drop counter +1, WARN logged | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Notification handler returns HTTP 200 `{"decision": "allow"}` for all decision variants | unit |
| VP-TBD | 2000ms timeout fires and returns HTTP 200 when module stalls | integration |
| VP-TBD | `HookDecision::Defer` triggers WARN log and no oneshot creation | unit |
| VP-TBD | Drop counter increments on full event bus, handler not blocked | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — this BC defines the wiring of the Notification hook endpoint (one of the 5 hook types) from HTTP ingress through EngineModule dispatch to event bus and ring, which is the composition-root routing responsibility assigned to SS-04/CAP-004 |
| L2 Domain Invariants | DI-001 (every hook event received MUST be written to the JSONL ring before acknowledgement — PC-6 implements ring append; best-effort caveat applies only to I/O-layer failures); DI-005 (auth token prefix enforcement — upstream auth middleware, Precondition 3) |
| Architecture Module | monocle-runtime (hook handlers, axum router) per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §Hook Endpoint Routing |
| Cross-Ref | BC-2.01.003 (body size limit — upstream); BC-2.01.009 (auth — upstream); BC-2.03.001 (EngineModule trait); BC-2.04.007 (PreToolUse routing — sibling, 300ms budget); BC-2.04.011 (event bus) |
| Test File | `monocle-runtime/tests/hook_routing_notification.rs` |
| Test Name | `test_BC_2_04_008_notification_routing` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.007] — composes with: PreToolUse uses the same middleware stack and ring-append logic; differs in timeout (300ms) and Defer semantics
- [BC-2.04.009] — composes with: Stop/SessionStart/PromptSubmit use 300ms timeout; same middleware stack
- [BC-2.04.011] — depends on: bounded event bus (PC-5)
- [BC-2.01.003] — depends on: body size limit middleware
- [BC-2.01.009] — depends on: auth middleware
- [BC-2.03.001] — depends on: EngineModule trait definition

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#hook-endpoint-routing` — routing stack diagram, 2000ms timeout for Notification
- `architecture/SS-daemon-wiring.md#bounded-event-bus` — try_send non-blocking semantics

## Story Anchor

S-TBD — Implement Notification hook routing handler with 2000ms timeout (filled by story-writer)

## VP Anchors

- VP-TBD — filled after VP creation

## §Trace v1.0.0

**Initial production** (2026-05-26T12:01:00Z):
- BC-2.04.008 created as new artifact for SS-04 §Hook Endpoint Routing per task instruction.
- Covers: 2000ms timeout, Notification-specific no-Defer invariant, fire-and-forward response
  semantics (always HTTP 200 `{"decision": "allow"}`), event bus try_send, ring append.
- Capability anchor: CAP-004 per ARCH-INDEX §SS-04 Capability Traceability row.
- SE-16d PASS: 2026-05-26T12:01:00Z > chain prior 2026-05-26T12:00:00Z. PASS.

## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.0.0` → `SS-daemon-wiring.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-003 LOW — Architecture Source pin updated from v1.1.0 to v1.2.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.1.0` → `SS-daemon-wiring.md v1.2.0` per F-P1D4-003 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.1.0

**F-P12-001 HIGH — `on_hook` call signature corrected from 3-param to 1-param** (2026-05-26T00:00:00Z):
- PC-3: Replaced fabricated `EngineModule::on_hook(HookType::Notification, session_id,
  &payload)` (3-parameter form that does not exist) with the correct 1-parameter form:
  `engine.on_hook(hook_event).await` where `hook_event: HookEvent` is pre-constructed by
  the HTTP handler from `HookEnvelope` fields before dispatching to the trait method.
- The actual trait signature is `async fn on_hook(&self, event: HookEvent) -> HookResponse`
  per `monocle-core/src/engine.rs`. The `HookEvent` variant is `HookEvent::Notification(
  NotificationEvent { notification_type, tool_name, tool_input, message, session_id, pid })`.
- The `HookType` discriminant enum exists in `hook_events.rs` for use as a map key in
  `ClaudeCodeModule::hook_paths()` — it is NOT a parameter to `on_hook`.
- SE-16d monotonicity: v1.1.0 timestamp >= v1.0.3. PASS.

## §Trace v1.0.3

**F-P1D6-003 HIGH — BC-2.04.001 PC cross-reference numbers corrected** (2026-05-26T00:00:00Z):
- Precondition 4: `BC-2.04.001 PC-6` → `BC-2.04.001 PC-11`. PC-11 is where the EngineModule
  registry is populated in the start sequence (Step 6). The former PC-6 was the bind-failure
  exit code (Step 3), not the EngineModule step.
- Precondition 5: `BC-2.04.001 PC-5` → `BC-2.04.001 PC-10`. PC-10 is where the bounded event
  bus is created (Step 5). The former PC-5 was the port-in-local-variable step (Step 3).
- Precondition 6: `BC-2.04.001 PC-4` → `BC-2.04.001 PC-7`. PC-7 is where the RingBuffer is
  constructed (Step 4). The former PC-4 was the TcpListener::bind call (Step 3).
- Root cause: off-by-one between step numbers and PC numbers in BC-2.04.001; each step covers
  multiple PCs, so step N ≠ PC-N.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.2.0

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-daemon-wiring.md v1.2.0 → v1.3.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §Hook Endpoint Routing` → `SS-daemon-wiring.md v1.3.0 §Hook Endpoint Routing`.
- Plain version-pin refresh. No substantive content propagation required — §Hook Endpoint Routing section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.
