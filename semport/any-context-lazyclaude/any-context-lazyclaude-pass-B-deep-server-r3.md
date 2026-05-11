# Pass B Deepening — `internal/server/` Round 3 (FINAL)

**Subsystem:** `internal/server/`
**Round:** 3 (narrow scope — JSON-RPC handler + final polish)
**Prior outputs read:**
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-server-r1.md`
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-server-r2.md`

This round is **deliberately short** — I committed at end of r2 to one more pass to formalize the JSON-RPC handler (handler.go = 188 LOC) and check for residual surprises. Spoiler: nothing architectural; a handful of small contracts.

---

## 1. JSON-RPC Handler — Residual Contracts

### BC-MCPSRV-069: `IsNotification()` predicate accepts BOTH `id == nil` AND literal `id: null`
**Source:** jsonrpc.go:33-35: `return r.ID == nil || string(r.ID) == "null"`. The `string(r.ID)` of an explicit `null` literal in JSON gives the four-char string `"null"`.
**Tests:** `TestParseRequest_Notification` (jsonrpc_test.go:24-32), `TestParseRequest_NullID` (jsonrpc_test.go:34-41).
**Confidence:** HIGH.
**Why this matters:** JSON-RPC 2.0 says notifications omit the `id` field; the spec is silent on `id: null` semantics. The handler treats both equivalently. A strict spec-conformant client would never send `id: null`, but tolerating it is defense-in-depth.

### BC-MCPSRV-070: `ParseRequest` rejects any JSON-RPC version string other than literal `"2.0"`
**Source:** jsonrpc.go:62-64: `if req.JSONRPC != "2.0" { return nil, fmt.Errorf("unsupported jsonrpc version: %q", req.JSONRPC) }`.
**Tests:** `TestParseRequest_WrongVersion` (jsonrpc_test.go:59-64), `TestParseRequest_MissingVersion` (jsonrpc_test.go:66-70).
**Confidence:** HIGH.
**Caveat:** Missing JSONRPC field defaults to `""`, also rejected. So a request lacking the `"jsonrpc"` key is rejected — strict.

### BC-MCPSRV-071: `Handler.HandleMessage` returns `nil` for unknown notification methods; returns `-32601` error response for unknown request methods
**Source:** handler.go:51-57.
- `if req.IsNotification() { return nil }` — silent ignore.
- Else: `NewErrorResponse(req.ID, -32601, "method not found: <method>")`.
**Tests:** `TestHandler_UnknownMethod_Request`, `TestHandler_UnknownMethod_Notification` (handler_test.go:227-255).
**Confidence:** HIGH.

### BC-MCPSRV-072: `handleIDEConnected` silently no-ops on three failure modes (invalid params JSON, pid≤0, unresolvable window)
**Source:** handler.go:89-115.
- Each failure is logged via `h.log.Printf`. None propagates to caller; no JSON-RPC response (this is a notification — see handler.go:46 dispatch).
- `resolveWindow` error AND empty window → log and return (lines 100-108). Note: a `resolveWindow` error WITH a non-empty window (cache hit) does not exist by construction (cache hits always return `(w, nil)`).
**Tests:** `TestHandler_IDEConnected_InvalidPID`, `TestHandler_IDEConnected_InvalidParams` (handler_test.go:90-116).
**Confidence:** HIGH.

### BC-MCPSRV-073: `resolveWindow` returns `("", err)` on pidwalk failure; `("", nil)` on pidwalk success but no match
**Source:** handler.go:117-132.
- Cache hit: `(w, nil)`.
- Cache miss + pidwalk error: `("", err)` — propagated.
- Cache miss + pidwalk success + window found: `(w.ID, nil)`.
- Cache miss + pidwalk success + no window: `("", nil)` — **NOT an error**.

The third case (no match without error) means "PID exists but is not under a tmux pane". Caller (`handleIDEConnected`) treats this same as the error case (both result in window=="", which triggers the no-window-found log).
**Tests:** Cached path tested in `TestHandler_IDEConnected_CachedWindow` (handler_test.go:118-136); error and not-found branches exercised indirectly.
**Confidence:** HIGH.

### BC-MCPSRV-074: `validateFilePath` distinguishes "empty path" from "non-absolute path" with distinct error messages
**Source:** handler.go:139-147.
- `path == ""` → `fmt.Errorf("empty file path")`.
- `!filepath.IsAbs(path)` → `fmt.Errorf("path must be absolute: %s", path)`.
- Both propagate as `-32602` via `NewErrorResponse(req.ID, -32602, err.Error())`.

Pass 3 BC-MCPSRV-013 grouped these as "non-absolute returns -32602". The distinction matters because a strict client that empty-strings the path gets a different error message.
**Tests:** `TestHandler_OpenDiff_InvalidParams` (handler_test.go:179-195) exercises bad JSON only — neither distinct path-validation error is asserted.
**Confidence:** HIGH (source); MEDIUM (test).
**Gap note:** Test coverage gap, not a behavioral issue.

### BC-MCPSRV-075: `handleOpenDiff` decouples response success from `notify.Enqueue` success
**Source:** handler.go:170-187.
- If `h.runtimeDir != ""`, enqueue is attempted; failure is logged but doesn't change the response.
- The success response (`-32603`-free) is returned regardless of enqueue outcome.
- If `h.runtimeDir == ""`, enqueue is skipped entirely (no failure).
**Tests:** `TestHandler_OpenDiff_WritesNotification` (handler_test.go:197-225) asserts enqueue happens when runtimeDir is set; no test for the runtimeDir=="" skip path.
**Confidence:** HIGH (source); MEDIUM (test on skip path).

### BC-MCPSRV-076: `openDiff` success response includes `window` and `old_path` — NOT `new_contents` echo
**Source:** handler.go:183-186. Response shape: `{"window": "<id>", "old_path": "<path>"}`.
**Why:** Avoids echoing large content back over the WebSocket — the diff data already lives in the notification file.
**Tests:** `TestHandler_OpenDiff` (handler_test.go:138-160) asserts both fields.
**Confidence:** HIGH.

### BC-MCPSRV-077: `Handler` does NOT handle `/notify`-style HTTP — it's WebSocket-only
**Source:** All `Handler.HandleMessage` invocations come from `(*Server).serveConn` (server.go:319). Hook HTTP endpoints (`/notify`, `/stop`, etc.) bypass `Handler` entirely and use `Server` methods directly. This is a clean boundary: `Handler` is the JSON-RPC dispatch layer; `Server` is the HTTP layer.
**Why this matters:** A porter might assume `Handler` is the universal dispatch — it is not. The JSON-RPC method set is exactly `{initialize, notifications/initialized, ide_connected, openDiff}` (handler.go:41-50).
**Confidence:** HIGH.

---

## 2. Residual Findings Status

| Item | r1 status | r2 status | r3 verdict |
|---|---|---|---|
| Diff-choice goroutine + `context.Background()` | P1 finding | P1 confirmed | P1 (carry to disposition-pass) |
| `/msg/resume` untested | P1 finding | P1 confirmed | P1 (carry) |
| `/msg/create` vs daemon schema drift | known from Pass 8 | new contract BC-MCPSRV-049 | Closed |
| `/msg/send` response shape drift | not raised | P1 new | P1 (carry) |
| `/msg/resume` error-leak inconsistency | P2 finding | confirmed | P2 (carry) |
| Pass 8 §311 lock-file mode claim | correction proposed | re-confirmed (0600) | Closed (Pass 8 needs amendment) |
| Broker single-mutex correction | not raised | BC-MCPSRV-057 | Closed (source comment imprecise) |
| `LastPendingWindow` untested | gap raised | verified test-only | Closed |
| `CapturePaneANSI` error fallback untested | gap raised | verified test-only | Closed |
| `/notify` DROPPED path untested | gap raised | verified test-only | Closed |
| `/stop` / `/session-start` empty-window publish | gap raised | by-design (BC-MCPSRV-058) | Closed |

---

## Delta Summary

- **New contracts:** BC-MCPSRV-069..077 (9 small contracts on JSON-RPC handler).
- **Confirmed prior contracts:** BC-MCPSRV-011..013 (Pass 3) refined into 069..077.
- **No new P0 / P1 / P2 findings.**
- **No new architectural surprise.**
- **No new dependency.**
- **No source bug or security concern uncovered.**

The new contracts cover exactly the JSON-RPC handler boundary I committed at end of r2 to formalize. Each captures a small invariant; none changes the model.

---

## Novelty Assessment

**Novelty: NITPICK.**

Test for novelty: "would removing this round's findings change how to spec the system?"

- BC-MCPSRV-069 (id:null tolerance) — marginal; defense-in-depth detail.
- BC-MCPSRV-070 (strict version reject) — already implicit in Pass 3 BC-MCPSRV-011.
- BC-MCPSRV-071 (unknown method dispatch) — direct refinement of Pass 3 BC-MCPSRV-011.
- BC-MCPSRV-072 (silent no-op modes) — small ergonomic detail.
- BC-MCPSRV-073 (resolveWindow tri-modal return) — useful for porters but a small clarification.
- BC-MCPSRV-074 (validateFilePath distinct messages) — refinement of Pass 3 BC-MCPSRV-013.
- BC-MCPSRV-075 (decoupled enqueue) — small clarification.
- BC-MCPSRV-076 (response field set) — useful but small.
- BC-MCPSRV-077 (Handler is WS-only) — useful for porters but inferable from r1 architecture.

These are real and good-to-have, but they refine rather than reshape. A porter armed with r1+r2 would derive these from source without effort. **They are nitpicks** — the kind of formalization that's nice for completeness but does not change a spec.

The round was bounded as I committed to in r2: JSON-RPC handler scope only. Within that bound, this is the full extraction. No reason to run a round 4.

## Convergence Declaration

**Pass B `internal/server/` has converged. NITPICK reached.**

Across rounds 1, 2, and 3 the package's behavioral surface is now formalized at the contract level: 68 server-specific contracts (BC-MCPSRV-001..077 with 9 gaps in numbering where r3 formalized handler-specific items). All 20 Pass 3 contracts verified. 3 P1 findings to forward to disposition-pass (untested /msg/resume; diff-choice goroutine + context bug; /msg/send response-shape divergence). 1 P2 finding (resume/create error-message asymmetry). 1 Pass 8 §311 correction (lock file mode IS 0600). 1 source-comment correction (broker single-mutex).

No architectural surprise, no hidden subsystem, no dependency miss, no missed security concern beyond Pass 8 §310 (which is correctly located in `startServer` here at ensure.go:127-131 and already on the Pass 8 radar).

## State Checkpoint

```yaml
pass: B
subsystem: internal/server
round: 3
status: complete
contracts_added: 9
contracts_total_for_subsystem: 77 (BC-MCPSRV-001..077, all formalized)
p1_findings_total_carried: 3
p2_findings_total_carried: 1
pass8_corrections_total: 1 (lock mode) + 1 (broker mutex doc) = 2
timestamp: 2026-05-11T18:20:00Z
novelty: NITPICK
converged: true
next_round_needed: false
```
