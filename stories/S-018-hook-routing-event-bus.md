---
document_type: story
level: L4
story_id: S-018
epic_id: EPIC-04
version: "1.0"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 8
wave: 5
tdd_mode: strict
priority: P0
depends_on: [S-017, S-002, S-003, S-004, S-009, S-014]
blocks: [S-021, S-022, S-029]
target_module: monocle-runtime
subsystems: [SS-04]
behavioral_contracts: [BC-2.04.007, BC-2.04.008, BC-2.04.009, BC-2.04.011]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.007.md, version: "1.4.0"}
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.008.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.009.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.011.md, version: "1.3.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.04.007 (PreToolUse routing), BC-2.04.008 (Notification routing), BC-2.04.009 (Stop/SessionStart/PromptSubmit routing), BC-2.04.011 (bounded event bus)"
---

# S-018: Hook Endpoint Routing + Bounded Event Bus with Drop Counter

## Narrative

As the monocle daemon, I want all five hook endpoint handlers to route incoming hook POST
requests through `EngineModule::on_hook()`, publish events to a bounded channel, and append
JSONL records — all within their respective timeout budgets — so that Claude Code receives
timely responses and the TUI receives real-time event notifications.

## Acceptance Criteria

### AC-001 (traces to BC-2.04.011 postcondition PC-1 — channel capacity 4096)
`tokio::sync::mpsc::channel::<HookEvent>(4096)` is called exactly once per daemon start
at step 5. The capacity `4096` is a compile-time constant, NOT configurable at runtime.
`DaemonState.event_bus_tx` holds `Arc<EventBusTx>`. The receiver is consumed by the fan-out task.

### AC-002 (traces to BC-2.04.011 postcondition PC-2 — drop counter initialized to 0)
`DaemonState.drop_counter` is initialized as `Arc<AtomicU64>::new(0)` (or equivalent `AtomicU64`
accessed via `Arc<DaemonState>`). The counter NEVER resets to 0 during a daemon run.

### AC-003 (traces to BC-2.04.011 postcondition PC-3 — try_send non-blocking)
All hook handlers (PreToolUse, Notification, Stop, SessionStart, PromptSubmit) use
`DaemonState.event_bus_tx.try_send(event)` (non-blocking). If `try_send` returns
`Err(TrySendError::Full)`:
a) `DaemonState.drop_counter.fetch_add(1, Ordering::Relaxed)` is called.
b) `WARN: event bus full; dropping event (drop_count=<N>)` is logged.
c) The event is silently discarded.
Blocking `.send(event).await` is FORBIDDEN in all hook handlers.

### AC-004 (traces to BC-2.04.011 postcondition PC-4/PC-5 — fan-out task)
A dedicated tokio task is spawned during daemon startup. The task:
1. Awaits `EventBusRx.recv()` — exits if channel closed.
2. Attempts `HookEventReceived` IPC send to each TUI client in `DaemonState.tui_clients`.
3. If a client write fails, removes the client from `tui_clients`.
4. Per-client write timeout is 50ms — slow clients are removed without stalling `recv()`.

### AC-005 (traces to BC-2.04.011 postcondition PC-6 — drop counter in state pushes)
Every daemon-to-TUI state push MUST include the current `DaemonState.drop_counter` value
(read at `Ordering::Relaxed`).

### AC-006 (traces to BC-2.04.011 postcondition PC-7 — graceful shutdown via channel close)
On graceful shutdown, `EventBusTx` is dropped before the tokio runtime shuts down. This
causes `EventBusRx.recv()` in the fan-out task to return `None`, terminating the loop.
The fan-out task MUST NOT be forcibly aborted.

### AC-007 (traces to BC-2.04.011 postcondition PC-8 — DropCounterUpdate debounce 100ms)
`ServerToClient::DropCounterUpdate` is sent at most once per 100ms regardless of how many
drop events occur in the window. The value reflects the cumulative counter at debounce-fire
time (not a delta). Implementation: tokio interval or sleep loop at 100ms cadence.

### AC-008 (traces to BC-2.04.007 postcondition PC-1 — PreToolUse JSON deserialization)
The `POST /hooks/pre-tool-use` handler deserializes the request body into `HookEnvelope`.
If deserialization fails, returns HTTP 422 with `{"error": "invalid_body", "message": "<serde error>"}`.
No event bus publish or ring append on deserialization failure.

### AC-009 (traces to BC-2.04.007 postcondition PC-3 — PreToolUse EngineModule dispatch)
The handler constructs `HookEvent::PreToolUse(PreToolUseEvent { tool_name, tool_input, session_id, pid })`
and calls `engine.on_hook(hook_event).await`. The 1-parameter form `on_hook(hook_event)` is
used — NOT a 3-parameter form.

### AC-010 (traces to BC-2.04.007 postcondition PC-3 — HookDecision::Defer handling)
When the module returns `HookDecision::Defer`, the handler:
a) Creates a per-invocation `oneshot::channel` keyed by session ID + nonce.
b) Pushes `PermissionPromptQueued` IPC message to all connected TUI clients before awaiting.
c) Awaits the oneshot receiver subject to the 300ms timeout budget.

### AC-011 (traces to BC-2.04.007 postcondition PC-4 — PreToolUse 300ms timeout)
The entire PreToolUse handler body is wrapped in `tokio::time::timeout(Duration::from_millis(300), ...)`.
If timeout fires, returns HTTP 200 with `{"decision": "allow", "reason": "timeout"}` (fail-open
per BC-HOOK-001). The 300ms budget is ABSOLUTE — no code path delays the response beyond 300ms.

### AC-012 (traces to BC-2.04.007 postcondition PC-7 — HookDecision::Block unit variant)
`HookDecision::Block` is a UNIT variant. The `"reason"` field in the HTTP JSON response body
is populated from `HookResponse.diagnostic` (e.g., `diagnostic: Some("reason text")`), not
from an enum variant field.

### AC-013 (traces to BC-2.04.008 postcondition PC-3 — Notification: no Defer)
The `POST /hooks/notification` handler does NOT support `HookDecision::Defer`. If a module
returns `Defer` for a Notification, the handler treats it as `Allow` and logs:
`WARN: invalid Defer on Notification hook; treating as Allow (module=<name>)`.
No `oneshot::channel` is created; no `PermissionPromptQueued` IPC is sent.

### AC-014 (traces to BC-2.04.008 postcondition PC-4 — Notification 2000ms timeout)
The Notification handler uses `tokio::time::timeout(Duration::from_millis(2000), ...)`.
On timeout, returns HTTP 200 with `{"decision": "allow"}` and logs:
`WARN: notification handler timeout (session_id=<id>)`.

### AC-015 (traces to BC-2.04.008 postcondition PC-7 — Notification always allow response)
Notification always returns HTTP 200 `{"decision": "allow"}` regardless of `HookDecision` variant.
Block is treated as an internal-only signal; it is NOT surfaced to Claude Code in the response.

### AC-016 (traces to BC-2.04.009 postcondition PC-2 — Stop: session state = Stopped)
The `POST /hooks/stop` handler marks the session as `SessionState::Stopped` in the session
registry after `EngineModule::on_hook()` returns.

### AC-017 (traces to BC-2.04.009 postcondition PC-3 — no Defer for Stop/SessionStart/PromptSubmit)
Stop, SessionStart, and PromptSubmit handlers do NOT support `HookDecision::Defer`. If a module
returns `Defer`, the handler treats it as `Allow` and logs:
`WARN: invalid Defer on <hook_type> hook; treating as Allow (module=<name>)`.

### AC-018 (traces to BC-2.04.009 postcondition PC-7 — PromptSubmit Block response)
For `POST /hooks/prompt-submit`, `HookDecision::Block` produces HTTP 200
`{"decision": "block", "reason": "<reason>"}` where `reason` comes from `HookResponse.diagnostic`.
Timeout produces `{"decision": "allow"}` (fail-open).

### AC-019 (traces to BC-2.04.007 postcondition PC-5/PC-6 — ring append and bus publish)
After obtaining a `HookDecision`, all handlers:
a) Call `DaemonState.event_bus_tx.try_send(event)` (non-blocking).
b) Call `DaemonState.ring.append(record)`. On I/O failure, log WARN and continue.
The HTTP response is NOT affected by bus saturation or ring append failure.

### AC-020 (traces to BC-2.04.011 invariant 5 — Ordering::Relaxed for drop counter)
The drop counter uses `Ordering::Relaxed` for both reads (state pushes, PC-6) and writes
(increment on drop, PC-3). Relaxed ordering is sufficient as the counter is for monitoring only.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,900 |
| BC-2.04.007.md | ~1,400 |
| BC-2.04.008.md | ~900 |
| BC-2.04.009.md | ~1,000 |
| BC-2.04.011.md | ~1,100 |
| SS-daemon-wiring.md §Hook Endpoint Routing + §Bounded Event Bus | ~5,000 |
| S-017 (DaemonState API) | ~400 |
| Test files | ~1,200 |
| **Total estimate** | **~12,900** |

## Tasks

- [ ] Implement `POST /hooks/pre-tool-use` handler in `monocle-runtime/src/hooks/pre_tool_use.rs`:
  - JSON deserialization of `HookEnvelope`
  - Session registry lookup/create
  - `HookEvent::PreToolUse` construction and `engine.on_hook(hook_event).await` dispatch
  - `HookDecision::Defer` path: `oneshot::channel` + `PermissionPromptQueued` IPC push + await
  - `tokio::time::timeout(300ms, ...)` wrapping entire handler body
  - Fail-open on timeout: `{"decision": "allow", "reason": "timeout"}`
  - `HookDecision::Block` → populate reason from `HookResponse.diagnostic`
- [ ] Implement `POST /hooks/notification` handler with 2000ms timeout and no-Defer invariant
- [ ] Implement `POST /hooks/stop` handler with session state transition to Stopped
- [ ] Implement `POST /hooks/session-start` handler (informational, always allow)
- [ ] Implement `POST /hooks/prompt-submit` handler with Block response from diagnostic
- [ ] All handlers: `try_send` to event bus + `ring.append()` (both best-effort)
- [ ] Wire all 5 handler routes into `build_server()` axum router
- [ ] Implement `event_bus_task` fan-out loop: `recv()` → per-client 50ms timeout write → client removal on failure
- [ ] Implement drop counter `AtomicU64` increment on `TrySendError::Full`
- [ ] Implement `DropCounterUpdate` debounce: tokio interval at 100ms cadence, send only if counter changed
- [ ] Unit tests `monocle-runtime/tests/hook_routing_pre_tool_use.rs`:
  - Allow path: HTTP 200 `{"decision": "allow"}`
  - Block path: HTTP 200 `{"decision": "block", "reason": "..."}`
  - Defer path: PermissionPromptQueued IPC sent, oneshot resolved → allow
  - Timeout path: fail-open allow response after 300ms
  - Invalid JSON → HTTP 422
  - Event bus full → drop counter +1, WARN logged, response not blocked
- [ ] Unit tests `monocle-runtime/tests/hook_routing_notification.rs`
- [ ] Unit tests `monocle-runtime/tests/hook_routing_stop_session_prompt.rs`
- [ ] Integration test `monocle-runtime/tests/event_bus.rs`:
  - Channel at 4096 capacity; 4097th event → drop counter = 1, WARN logged
  - Fan-out task exits cleanly on channel close (graceful shutdown)
  - TUI client disconnect detected; subsequent events not sent to that client

## Previous Story Intelligence

S-002: `GET /healthz` axum route is in place. The `build_server()` function exists.
S-003: Auth middleware (`auth_middleware`) is in the middleware stack.
S-004: `body_size_limit_middleware` is wired before auth.
S-009: `auth_middleware` validates the `X-Monocle-Authorization` / `X-Claude-Code-Ide-Authorization` header.
S-014: `EngineModule` trait with `async fn on_hook(&self, event: HookEvent) -> HookResponse` is defined.
S-017: `DaemonState` struct with `event_bus_tx`, `drop_counter`, `ring`, `engine_registry`, `session_registry` is available.

The middleware execution order from prior waves is IMMUTABLE:
`body_size_limit_middleware` → `auth_middleware` → `DefaultBodyLimit::max(262144)` → hook handler.

## Architecture Compliance Rules

From `architecture/SS-daemon-wiring.md v1.2.0 §Hook Endpoint Routing`:
- `on_hook(hook_event)` is the CORRECT call signature — 1 parameter
- `HookDecision::Block` is a UNIT variant — reason comes from `HookResponse.diagnostic`
- Timeout is FAIL-OPEN for PreToolUse: `{"decision": "allow", "reason": "timeout"}` per BC-HOOK-001
- `try_send` is MANDATORY; `.send().await` is FORBIDDEN in hook handlers

From `architecture/SS-daemon-wiring.md v1.2.0 §Bounded Event Bus`:
- Channel capacity = 4096; NOT runtime-configurable in Phase 1
- Fan-out per-client timeout = 50ms
- Drop counter uses `Ordering::Relaxed`

From `architecture/SS-conventions-anti-patterns.md v1.29.5`:
- Bounded `mpsc::channel(N)` with drop counter is the canonical pattern
- Unbounded `mpsc::unbounded_channel()` is FORBIDDEN

**Forbidden Dependencies:**
- Hook handlers MUST NOT call `.send(event).await` (blocking send)
- `unbounded_channel()` MUST NOT appear in `monocle-runtime/src/`
- `HookDecision::Block { reason }` (struct variant form) MUST NOT be used

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| axum | =0.8.9 | Route handlers, `Json` extractor, `StatusCode` |
| tokio | =1.52.0 | `time::timeout`, `sync::mpsc::channel`, `sync::oneshot::channel` |
| serde_json | =1.0.149 | JSON deserialization of `HookEnvelope` |
| tracing | 0.1 | Structured WARN/ERROR logging per handler |
| uuid | (caret pin, `features=["v4", "serde"]`) | Per-invocation `oneshot` nonce for Defer prompts |

## File Structure Requirements

Files to create:
- `monocle-runtime/src/hooks/pre_tool_use.rs` — PreToolUse handler
- `monocle-runtime/src/hooks/notification.rs` — Notification handler
- `monocle-runtime/src/hooks/stop_session_prompt.rs` — Stop/SessionStart/PromptSubmit handlers
- `monocle-runtime/src/event_bus.rs` — fan-out task, drop counter, DropCounterUpdate debounce
- `monocle-runtime/tests/hook_routing_pre_tool_use.rs`
- `monocle-runtime/tests/hook_routing_notification.rs`
- `monocle-runtime/tests/hook_routing_stop_session_prompt.rs`
- `monocle-runtime/tests/event_bus.rs`

Files to modify:
- `monocle-runtime/src/server.rs` — wire 5 hook routes into `build_server()`
- `monocle-runtime/src/lib.rs` — expose hooks and event_bus modules
