---
document_type: behavioral-contract
level: L3
version: "1.5.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "d8d8cec"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D2-001, F-P1D2-009, F-P1D2-010, F-P1D11-001, F-P12-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.007: Hook Endpoint: PreToolUse Request Routing

## Description

The `POST /hooks/pre-tool-use` handler is the critical-path routing contract for PreToolUse
hook events. It receives an HTTP request that has already passed through
`body_size_limit_middleware` (256 KiB cap, BC-2.01.003) and `auth_middleware` (dual-accept
token validation, BC-2.01.009), deserializes the JSON body into a `HookEnvelope`, dispatches
to `EngineModule::on_hook()`, publishes to the event bus, appends a JSONL record, and returns
a `HookResponse` JSON body within a 300ms timeout budget. If the module returns
`HookDecision::Defer`, the handler holds the HTTP connection open and waits for a user
decision from the TUI via a per-invocation `oneshot::channel`, still subject to the 300ms
budget.

## Preconditions

1. The HTTP request has been accepted by the axum router registered in `build_server()`.
2. `body_size_limit_middleware` has already verified `Content-Length ≤ 262144` bytes;
   the request body fits in memory (BC-2.01.003 postcondition satisfied).
3. `auth_middleware` has already verified the `X-Monocle-Authorization` or
   `X-Claude-Code-Ide-Authorization` header carries a valid monocle auth token
   (BC-2.01.009 postcondition satisfied); the request is authenticated.
4. At least one `EngineModule` implementation is registered in `DaemonState.engine_registry`
   (invariant: the daemon start sequence always registers `ClaudeCodeModule` at step 6 per
   BC-2.04.001 PC-11).
5. `DaemonState.event_bus_tx` (`Arc<EventBusTx>`) is initialized and points to a live
   `mpsc::channel(4096)` sender (invariant: step 5 of the daemon start sequence per
   BC-2.04.001 PC-10).
6. `DaemonState.ring` (`Arc<RingBuffer>`) is initialized (invariant: step 4 of the daemon
   start sequence per BC-2.04.001 PC-7).

## Postconditions

**PC-1 — JSON deserialization.**
The handler calls `axum::extract::Json::<HookEnvelope>::from_request()` (or equivalent
axum extractor). If deserialization fails (malformed JSON, missing required fields per
`monocle-proto` `HookEnvelope` schema), the handler returns HTTP 422 with body:
`{"error": "invalid_body", "message": "<serde error message>"}`.

**PC-2 — Session registry lookup or creation.**
The handler extracts `HookEnvelope.session_id` and looks up the session in
`DaemonState.session_registry`. If no session exists for this ID, a new
`EnrichedSession` is created and inserted. The session object is mutable for the
duration of this handler invocation only; all mutations are protected by the registry's
internal lock.

**PC-3 — EngineModule dispatch.**
The HTTP handler extracts the relevant fields from the deserialized `HookEnvelope` and
constructs a `HookEvent::PreToolUse(PreToolUseEvent { tool_name, tool_input, session_id,
pid })`. It then calls `engine.on_hook(hook_event).await` on each registered `EngineModule`
in the registry (Phase 1: exactly one module, `ClaudeCodeModule`). The signature is:

```
async fn on_hook(&self, event: HookEvent) -> HookResponse
```

The `HookEvent` is fully constructed by the handler before `on_hook` is invoked; the
`EngineModule` receives a single typed event argument. The returned `HookResponse.decision`
is:
- `HookDecision::Allow` — proceed to PC-6 immediately.
- `HookDecision::Block` — proceed to PC-6 with a block response. `HookDecision::Block` is a
  unit variant (no fields). The block reason is carried in `HookResponse.diagnostic`
  (e.g., `HookResponse { decision: HookDecision::Block, diagnostic: Some("reason text") }`).
- `HookDecision::Defer` — create a per-invocation `oneshot::channel`; push a
  `PermissionPromptQueued` IPC message to all connected TUI clients; await the oneshot
  receiver or timeout (PC-4).

**PC-4 — 300ms timeout budget enforcement.**
The entire handler body is wrapped in `tokio::time::timeout(Duration::from_millis(300), ...)`.
If the timeout fires before `EngineModule::on_hook()` returns (or before the deferred user
decision arrives via the oneshot channel), the handler produces a fail-open `HookResponse`
(`{"decision": "allow", "reason": "timeout"}`) and returns HTTP 200. The decision is NOT held
beyond the 300ms budget under any circumstances.

**PC-5 — Event bus publish (best-effort, non-blocking).**
After obtaining a `HookDecision`, the handler calls `DaemonState.event_bus_tx.try_send(event)`
(non-blocking). If `try_send` returns `Err(TrySendError::Full)`, the handler:
  a. Increments `DaemonState.drop_counter` (AtomicU64) by 1.
  b. Logs `WARN: event bus full; dropping event (drop_count=<N>)`.
  c. Discards the event.
The hook response is NOT affected by event bus saturation; PC-4 timeout is preserved
regardless of bus state.

**PC-6 — JSONL ring append (best-effort).**
The handler calls `DaemonState.ring.append(record)`. If the append fails (e.g., I/O error
during async flush), the handler logs `WARN: ring append failed: <error>` and continues.
The HTTP response is still HTTP 200; the hook event is NOT retried. (Per DI-001, best-effort
ring writes are acceptable when the write mechanism itself fails; the invariant governs
non-failure normal paths.)

**PC-7 — HookResponse formation and HTTP 200 return.**
The handler serializes a `HookResponse` JSON body and returns HTTP 200. The response body
format is governed by the DTU contracts (BC-HOOK-xxx series). For PreToolUse:
- `HookDecision::Allow` → `{"decision": "allow"}` (or compatible allow-variant per DTU spec).
- `HookDecision::Block` → `{"decision": "block", "reason": "<reason>"}`. Note: `HookDecision::Block`
  is a unit variant at the Rust level. The `"reason"` field in the JSON response body is populated
  from `HookResponse.diagnostic` (i.e., the serializer reads `diagnostic: Some("reason text")` and
  emits the `"reason"` key). The reason is NOT a field on the enum variant itself.
- Timeout → `{"decision": "allow", "reason": "timeout"}` (fail-open, matching BC-HOOK-001
  gene-source semantics). Phase 1 treats all PreToolUse timeouts uniformly as fail-open per
  BC-HOOK-001. Phase 2+ may introduce per-tool-type timeout policies.

## Invariants

1. The 300ms timeout budget (PC-4) is absolute. No code path in the PreToolUse handler may
   block the HTTP response beyond 300ms from request arrival at this handler layer.
2. Event bus saturation (PC-5) NEVER blocks the hook handler or delays the HTTP response.
   `try_send` is always non-blocking; dropping with counter increment is the defined behavior.
3. Middleware execution order is immutable: `body_size_limit_middleware` → `auth_middleware` →
   `DefaultBodyLimit::max(262144)` → this handler. This contract begins after auth passes.
4. The JSONL ring append (PC-6) is best-effort for I/O failures; however, DI-001 requires that
   a successful in-memory hook event MUST be submitted to the ring. The ring's internal
   queuing makes the submission synchronous from the handler's perspective; only the async
   disk flush may fail silently.
5. A `HookDecision::Defer` MUST result in a `PermissionPromptQueued` IPC push to all currently
   connected TUI clients before this handler awaits the oneshot receiver. No defer may be
   silent (no notification to TUI).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-070 | JSON body is syntactically valid but fails HookEnvelope schema validation (e.g., missing `session_id` field) | Handler returns HTTP 422 with `{"error": "invalid_body", "message": "<serde error>"}` |
| EC-071 | `EngineModule::on_hook()` panics | The tokio task catches the panic via `tokio::task::catch_unwind` or axum's built-in unwind handler; handler returns HTTP 500; daemon continues serving other requests |
| EC-072 | `HookDecision::Defer` returned but no TUI clients are connected | `PermissionPromptQueued` IPC push is a no-op (zero connected clients); oneshot receiver awaits timeout; handler returns fail-open allow after 300ms |
| EC-073 | TUI user responds to Defer BEFORE the 300ms deadline | Oneshot sender fires; handler receives the decision and returns before timeout fires |
| EC-074 | Two concurrent PreToolUse requests arrive for the same `session_id` | Each request creates its own `oneshot::channel` keyed by session ID + invocation nonce; session registry lock prevents concurrent mutation; both requests are handled independently with their own 300ms budgets |
| EC-075 | `DaemonState.ring` append blocks longer than the remaining timeout budget | The `tokio::time::timeout` wrapping the handler body fires; ring append is cancelled; handler returns fail-open allow; WARN logged |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid PreToolUse JSON body, auth OK, module returns `Allow` | HTTP 200, `{"decision": "allow"}`, event bus receives 1 event, ring receives 1 record | happy-path |
| Valid PreToolUse JSON body, auth OK, module returns `Block` (with `diagnostic: Some("policy")`) | HTTP 200, `{"decision": "block", "reason": "policy"}`, event bus receives 1 event | happy-path |
| Valid PreToolUse JSON body, auth OK, module returns `Defer`, TUI accepts within 300ms | HTTP 200, `{"decision": "allow"}`, `PermissionPromptQueued` IPC sent, event bus event published | happy-path |
| Valid PreToolUse JSON body, auth OK, module returns `Defer`, 300ms elapses | HTTP 200, `{"decision": "allow", "reason": "timeout"}`, event published (pre-timeout path) | edge-case |
| Malformed JSON body (auth OK) | HTTP 422, `{"error": "invalid_body", ...}`, no event bus publish, no ring append | error |
| Event bus full (channel at 4096 capacity), valid request | HTTP 200 with decision; drop counter incremented by 1; WARN logged; request not blocked | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | PreToolUse handler returns HTTP 200 with `{"decision": "allow"}` for valid allow-path request | integration |
| VP-TBD | 300ms timeout fires and returns fail-open allow when module hangs | integration |
| VP-TBD | Event bus drop counter increments when channel is full | integration |
| VP-TBD | Invalid JSON returns HTTP 422 | unit |
| VP-TBD | `HookDecision::Defer` sends `PermissionPromptQueued` IPC to connected TUI clients | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — this BC defines the routing logic that connects incoming hook HTTP events to the EngineModule dispatch, event bus, and ring buffer, which is the core wiring responsibility of the binary composition root defined in CAP-004 |
| L2 Domain Invariants | DI-001 (every hook event received MUST be written to the JSONL ring before acknowledgement — PC-6 implements ring append before HTTP response; PC-6 best-effort caveat applies only to I/O-layer failures, not to normal paths); DI-005 (daemon MUST NOT accept token without canonical prefix — enforced by auth_middleware upstream of this handler, Precondition 3) |
| Architecture Module | monocle-runtime (hook handlers, axum router) per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §Hook Endpoint Routing |
| Cross-Ref | BC-2.01.003 (body size limit middleware — upstream of this handler); BC-2.01.009 (auth middleware — upstream of this handler); BC-2.03.001 (EngineModule trait — dispatched at PC-3); BC-2.04.011 (bounded event bus — used at PC-5) |
| Test File | `monocle-runtime/tests/hook_routing_pre_tool_use.rs` |
| Test Name | `test_BC_2_04_007_pre_tool_use_routing` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.008] — composes with: Notification routing shares the same middleware stack and ring-append logic; differs in timeout (2000ms) and response semantics
- [BC-2.04.009] — composes with: Stop/SessionStart/PromptSubmit routing shares the middleware stack; differs in hook type and payload shape
- [BC-2.04.011] — depends on: event bus (PC-5) is specified by BC-2.04.011
- [BC-2.01.003] — depends on: body size limit middleware runs before this handler
- [BC-2.01.009] — depends on: auth middleware runs before this handler
- [BC-2.03.001] — depends on: EngineModule trait definition (PC-3 dispatch target)

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#hook-endpoint-routing` — full routing stack diagram, timeout budgets, Defer hold logic
- `architecture/SS-daemon-wiring.md#bounded-event-bus` — try_send semantics and drop counter
- `architecture/SS-engine-module.md#hook-decision` — HookDecision variants

## Story Anchor

S-TBD — Implement PreToolUse hook routing handler with 300ms timeout and EngineModule dispatch (filled by story-writer)

## VP Anchors

- VP-TBD — filled after VP creation

## §Trace v1.1.0

**F-P1D2-001 CRITICAL — PreToolUse timeout changed from fail-closed to fail-open** (2026-05-26T00:00:00Z):
- PC-4: "fail-closed HookResponse" → "fail-open HookResponse (`{"decision": "allow", "reason": "timeout"}`)" per F-P1D2-001. Architectural decision: PreToolUse daemon-side timeout is FAIL-OPEN for Phase 1 to match BC-HOOK-001 gene-source semantics; the daemon must never permanently block Claude Code.
- PC-7: Timeout response changed from `{"decision": "block", "reason": "timeout"} (fail-closed for writes)` to `{"decision": "allow", "reason": "timeout"} (fail-open, matching BC-HOOK-001 gene-source semantics)`. Note added: Phase 1 treats all PreToolUse timeouts uniformly as fail-open per BC-HOOK-001. Phase 2+ may introduce per-tool-type timeout policies.
- EC-072: "returns fail-closed block after 300ms" → "returns fail-open allow after 300ms".
- EC-075: "handler returns fail-closed block" → "handler returns fail-open allow".
- Canonical test vector: timeout case response changed from block to allow.
- VP table: "returns fail-closed block" → "returns fail-open allow".
- Also resolves F-P1D2-011: the read/write tool ambiguity is removed by treating all PreToolUse timeouts uniformly.

**F-P1D2-009 MEDIUM — PC cross-reference numbers corrected** (2026-05-26T00:00:00Z):
- Precondition 4: "BC-2.04.001 PC-6" → "BC-2.04.001 PC-11" (ClaudeCodeModule registered at step 6 = PC-11 per BC-2.04.001 §Step 6 — EngineModule registry populated).
- Precondition 5: "BC-2.04.001 PC-5" → "BC-2.04.001 PC-10" (event bus created at step 5 = PC-10 per BC-2.04.001 §Step 5 — Bounded event bus created).
- Precondition 6: "BC-2.04.001 PC-4" → "BC-2.04.001 PC-7" (ring buffer created at step 4 = PC-7 per BC-2.04.001 §Step 4 — RingBuffer created).

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.0.0` → `SS-daemon-wiring.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh for files modified in this session).

SE-16d monotonicity: v1.1.0 timestamp >= v1.0.0. PASS.

## §Trace v1.2.0

**F-P1D4-003 LOW — Architecture Source pin updated from v1.1.0 to v1.2.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.1.0` → `SS-daemon-wiring.md v1.2.0` per F-P1D4-003 bulk update.
- SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.4.0

**F-P12-001 HIGH — `on_hook` call signature corrected from 3-param to 1-param** (2026-05-26T00:00:00Z):
- PC-3: Replaced fabricated `EngineModule::on_hook(HookType::PreToolUse, session_id,
  &payload)` (3-parameter form that does not exist) with the correct 1-parameter form:
  `engine.on_hook(hook_event).await` where `hook_event: HookEvent` is pre-constructed by
  the HTTP handler from `HookEnvelope` fields before dispatching to the trait method.
- The actual trait signature is `async fn on_hook(&self, event: HookEvent) -> HookResponse`
  per `monocle-core/src/engine.rs`. The `HookEvent` variant is `HookEvent::PreToolUse(
  PreToolUseEvent { tool_name, tool_input, session_id, pid })`.
- The `HookType` discriminant enum exists in `hook_events.rs` for use as a map key in
  `ClaudeCodeModule::hook_paths()` — it is NOT a parameter to `on_hook`. The handler uses
  the `HookEvent` enum (with inner payload structs), not `HookType`, for dispatch.
- SE-16d monotonicity: v1.4.0 timestamp >= v1.3.0. PASS.

## §Trace v1.3.0

**F-P1D11-001 MEDIUM — HookDecision::Block corrected from struct variant to unit variant** (2026-05-26T00:00:00Z):
- PC-3: `HookDecision::Block { reason }` → `HookDecision::Block` (unit variant). Added
  clarification that the block reason is carried in `HookResponse.diagnostic`, not in the
  enum variant itself.
- PC-7: `HookDecision::Block { reason }` → `HookDecision::Block`. Added clarification that
  the `"reason"` field in the HTTP JSON response body is populated from `HookResponse.diagnostic`;
  the HTTP response format `{"decision": "block", "reason": "<reason>"}` is correct — the reason
  comes from `diagnostic`, not from a (non-existent) enum variant field.
- PC-7 timeout: was already `{"decision": "allow", "reason": "timeout"}` (fail-open). No change
  needed — the fail-open correction was applied in F-P1D2-001.
- Canonical test vectors: `Block { reason: "policy" }` → `Block (with diagnostic: Some("policy"))`.
- Root cause: `HookDecision::Block` is defined as a unit variant in `monocle-proto`; the block
  reason is carried separately in `HookResponse.diagnostic`. Spec had incorrectly treated it as
  a struct variant with a `reason` field.
- SE-16d monotonicity: v1.3.0 timestamp >= v1.2.0. PASS.

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.04.007 created as new artifact for SS-04 §Hook Endpoint Routing per task instruction.
- Covers: JSON deserialization, session registry, EngineModule dispatch, Defer/oneshot hold,
  300ms timeout enforcement, event bus try_send, drop counter, ring append best-effort,
  HookResponse serialization, HTTP 200 return.
- Capability anchor: CAP-004 per ARCH-INDEX §SS-04 Capability Traceability row.
- SE-16d PASS: 2026-05-26T12:00:00Z is the chain origin for this artifact.

## §Trace v1.5.0

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-daemon-wiring.md v1.2.0 → v1.3.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §Hook Endpoint Routing` → `SS-daemon-wiring.md v1.3.0 §Hook Endpoint Routing`.
- Plain version-pin refresh. No substantive content propagation required — §Hook Endpoint Routing section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.5.0 timestamp >= v1.4.0. PASS.
