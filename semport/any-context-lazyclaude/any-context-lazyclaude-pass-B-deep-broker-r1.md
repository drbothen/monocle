# Pass B Deepening — internal/core/event/broker.go (Round 1)

**Subject:** `internal/core/event/broker.go` + `broker_test.go` of any-context/lazyclaude
**Reference:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/event/broker.go` (122 LOC) and `broker_test.go` (385 LOC)
**Why this round exists:** The B.5 v2 audit flagged this file as a Drift-Category-B round-driven blind spot — load-bearing pub/sub primitive that received only Pass-3 test-derived BC extraction (BC-BROKER-001..012) and no Phase B verification. The single-mutex correction surfaced in `server-r2` was never propagated. Monocle has committed (Pass-8 §Section 5) to adopt the broker pattern for the runtime plane's hook→UI fan-out, so the contract details must be locked down before crystallization.

---

## 1. Implementation anatomy (source-verified)

### 1.1 Struct shape

`broker.go:9-22`:

| Field | Type | Purpose |
|-------|------|---------|
| `mu` | `sync.Mutex` | **single** mutex; guards `subs`, `nextID`, `closed` |
| `subs` | `map[uint64]*Subscription[T]` | active subscriber routing table, keyed by per-broker monotonic ID |
| `nextID` | `uint64` | monotonically increasing; never reused even after `Cancel`/`Close` |
| `closed` | `bool` | one-way flag: once `true`, the broker rejects new subscribers (returns closed channels) and no-ops on `Publish` |

`Subscription[T]` (`broker.go:17-22`):

| Field | Type | Purpose |
|-------|------|---------|
| `id` | `uint64` | back-pointer to the `subs` map key |
| `ch` | `chan T` | the buffered (or unbuffered) delivery channel |
| `broker` | `*Broker[T]` | back-pointer for `Cancel` to acquire the broker mutex |
| `once` | `sync.Once` | guards `Cancel` against double-close panics |

**Verified:** generic over `T any`. Both lazyclaude call sites instantiate `Broker[model.Event]`.

### 1.2 Locking discipline — verified single sync.Mutex

| Method | Mutex op | Lines |
|--------|----------|-------|
| `Subscribe(bufSize)` | `mu.Lock` / `defer mu.Unlock` | `broker.go:37-38` |
| `Publish(event)` | `mu.Lock` / `defer mu.Unlock` | `broker.go:61-62` |
| `HasSubscribers()` | `mu.Lock` / `defer mu.Unlock` | `broker.go:79-80` |
| `Close()` | `mu.Lock` / `defer mu.Unlock` | `broker.go:87-88` |
| `Subscription.Cancel()` | `b.mu.Lock` / `defer b.mu.Unlock` (under `once.Do`) | `broker.go:110-113` |
| `Subscription.Ch()` | **no lock** — returns the channel pointer (`broker.go:103-105`) |

**There is exactly one mutex. There is no RWMutex.** Even read-only `HasSubscribers` holds the same write-lock that `Publish` does. This is meaningful for contention: every published event serializes against every subscribe/cancel/has-subscribers query.

### 1.3 The race window that "single-mutex broker" implies (correction confirmed)

The Pass-8 comment that called broker locking "separate locks for Has and Publish" is **mechanism-incorrect** but **behavior-correct**. There is one mutex, taken sequentially across the two calls. Between `HasSubscribers()` returning `true` and the subsequent `Publish` taking the lock, another goroutine **can** run `Cancel()` and remove the last subscriber. The event still gets serialized through `Publish` (which takes the lock, sees `len(b.subs) == 0`, ranges over an empty map, and returns). **No panic, no leak — the event is silently dropped at the dispatch step.** This matches `BC-MCPSRV-008` (`/notify` skips `notify.Enqueue` when `HasSubscribers()` is true → if subscribers vanish between the check and the publish, the event is lost from both paths). Confirmed as a known benign race in `server-r2`.

---

## 2. Lifecycle — Subscribe → publish → Cancel

### 2.1 Subscribe (`broker.go:36-55`)

```go
func (b *Broker[T]) Subscribe(bufSize int) *Subscription[T] {
    b.mu.Lock(); defer b.mu.Unlock()
    ch := make(chan T, bufSize)
    s := &Subscription[T]{id: b.nextID, ch: ch, broker: b}
    b.nextID++
    if b.closed { close(ch); return s }
    b.subs[s.id] = s
    return s
}
```

Observations:
- `bufSize` is passed straight to `make(chan T, bufSize)` — **no validation, no clamping**. `bufSize < 0` would panic (`make(chan T, -1)`), but Go's `make` validates this so the panic happens inside `make`, not in broker code.
- `bufSize == 0` produces an **unbuffered channel**. Combined with the `select { default }` in `Publish`, this means **every** event is dropped unless the receiver happens to be parked in a receive at the same instant. Tested by `TestBroker_ZeroBufferSize` (`broker_test.go:320-333`) which only asserts non-panic, not delivery.
- `nextID` increments before the `closed` check. Even subscriptions to a closed broker burn an ID. Not load-bearing (IDs are 64-bit), but worth noting.
- The returned subscription on a closed broker has `id` set but is **not** in `subs` — `Cancel()` on it sees `_, exists := b.subs[s.id]` is `false` and is a no-op. The channel is already closed. Idempotency verified.

### 2.2 Publish (`broker.go:60-75`)

```go
func (b *Broker[T]) Publish(event T) {
    b.mu.Lock(); defer b.mu.Unlock()
    if b.closed { return }
    for _, s := range b.subs {
        select {
        case s.ch <- event:
        default:
            // Subscriber buffer full; drop to preserve non-blocking guarantee.
        }
    }
}
```

Observations:
- **The entire fan-out runs under the broker mutex.** Each `s.ch <- event` is attempted while holding `mu`. The send is non-blocking due to the `default`, so the mutex is held for ~O(N) channel ops + map iteration time — bounded and brief, but it does mean **subscribe/cancel/has-subscribers cannot interleave with a publish**. For a 5-subscriber fan-out with capacity-8 buffers, each iteration is a few-nanosecond channel push; total mutex hold time is bounded by `len(b.subs) × channel-send-cost`, no blocking.
- Map iteration order is randomized (Go runtime). **Subscribers receive events in undefined order relative to each other for any one Publish call**, but **the order of events within a single subscriber's channel is the order of Publish calls** (because Publish holds the mutex throughout, so two concurrent publishes serialize). This is the broker's strongest ordering guarantee. The test `TestBroker_SubscribeAndPublish` verifies per-subscriber order; no test verifies cross-subscriber order (and none should — it's undefined).

### 2.3 Cancel (`broker.go:109-122`)

```go
func (s *Subscription[T]) Cancel() {
    s.once.Do(func() {
        b := s.broker
        b.mu.Lock(); defer b.mu.Unlock()
        if _, exists := b.subs[s.id]; exists {
            delete(b.subs, s.id)
            close(s.ch)
        }
    })
}
```

Observations:
- `sync.Once` makes `Cancel()` idempotent at the subscription level. Calling `Cancel()` from N goroutines on the same sub: only one acquires the broker mutex; the rest are no-ops.
- The `if _, exists := b.subs[s.id]; exists` guard handles the **broker-already-closed** case: `Close()` clears the map (`broker.go:99`), so subsequent `Cancel()` on a subscription whose channel was closed by `Close()` sees `exists == false` and skips the double-close. **Verified safe across Close+Cancel sequencing.**
- **The channel close happens under the broker mutex.** A concurrent `Publish` cannot be in mid-`s.ch <- event` because the mutex serializes them.

### 2.4 Close (`broker.go:86-100`)

```go
func (b *Broker[T]) Close() {
    b.mu.Lock(); defer b.mu.Unlock()
    if b.closed { return }
    b.closed = true
    for _, s := range b.subs { close(s.ch) }
    b.subs = make(map[uint64]*Subscription[T])
}
```

Observations:
- `closed = true` is set *before* closing the channels but *after* taking the lock — any blocked publisher waiting on `mu` will see `closed == true` and return without sending. Correct ordering.
- The map is replaced with an empty map (`broker.go:99`), not just `delete`d in a loop, to make `Cancel`'s `exists` check uniform.
- The comment on line 98 ("Clear the map so Cancel calls on existing subscriptions are idempotent") is **correct** and the test `TestBroker_CloseIdempotent` covers double-Close.

### 2.5 HasSubscribers (`broker.go:78-82`)

```go
func (b *Broker[T]) HasSubscribers() bool {
    b.mu.Lock(); defer b.mu.Unlock()
    return len(b.subs) > 0
}
```

After `Close()`, `b.subs` is replaced with an empty map, so `HasSubscribers()` returns `false` — verified by `TestBroker_HasSubscribers_AfterClose` (`broker_test.go:360-368`). The race documented in §1.3 applies: the return value is a snapshot, valid only at the lock-release instant.

---

## 3. BC-BROKER-003 verification — HOW does the drop happen?

**Question from the audit:** Silent drop? Logged? Telemetry?

**Source (`broker.go:68-74`):**

```go
for _, s := range b.subs {
    select {
    case s.ch <- event:
    default:
        // Subscriber buffer full; drop to preserve non-blocking guarantee.
    }
}
```

**Verdict — completely silent.**

| Channel | Observable? | Evidence |
|---------|-------------|----------|
| Log line | **No** | The `default` arm is empty. No `b.log`, `slog`, `log.Printf` — broker has no logger field at all. |
| Metric / counter | **No** | No `expvar`, `prometheus`, or atomic counter on the struct. There is no `droppedCount` field anywhere. |
| Per-subscriber tracking | **No** | `Subscription[T]` has no drop counter; the `default` arm doesn't reference `s` for anything. |
| Returned error | **No** | `Publish` has no return value. |
| Caller-visible side effect | **No** | The publisher cannot tell whether any subscriber dropped the event. |

**Confirmed by the test:** `TestBroker_NonBlockingPublish` (`broker_test.go:93-118`) publishes 100 events into a buffer-1 subscriber that never reads, and asserts only that `Publish` does not block — **not** that any specific count was dropped, **not** that drops were logged. The test is silent on observability because the implementation provides none.

### 3.1 Architectural implication for monocle

This is the single highest-leverage finding of this deepening round.

When monocle ports the broker pattern, **silent drop is a hazardous default** for an observability surface (hook → UI fan-out). If the GUI's broker-receive goroutine ever stalls (say, locked behind a slow `gocui.Update`), the broker will drop events silently and the porter will get bug reports of the form "the activity icon stops updating after a while and there's no error log."

**Recommendation: monocle should add per-subscriber drop counters at port time.** Either:

1. **Atomic counter on `Subscription[T]`:** `s.droppedCount.Add(1)` in the `default` arm. Expose via `Subscription.DroppedCount() uint64`. Zero-cost when delivery succeeds.
2. **Optional logger / callback on the broker:** `NewBroker(WithDropCallback(func(sub, event){...}))`. Zero-cost when not set.

Option 1 is cheaper and sufficient for the monocle threat model. Option 2 is more flexible but invites callback-in-hot-path pitfalls.

The lazyclaude maintainers chose silent drop deliberately (the test name is `TestBroker_NonBlockingPublish` not `TestBroker_ReportsDroppedEvents`). It is defensible for their specific use case where lost activity events self-correct on the next event. **Monocle should not inherit this property silently** — the porter must consciously decide whether to keep it.

---

## 4. Buffer-size policy across call sites

There is no shared per-broker buffer size: every `Subscribe` call passes its own. Inventory of all production call sites:

| Call site | Buffer | Subscriber goroutine pattern | Risk if drops happen |
|-----------|--------|------------------------------|----------------------|
| `internal/gui/notify_loop.go:44` (`SetBroker`) | **8** | Single GUI ticker goroutine, drains in `app.go:284` `for { select { case ev := <-brokerCh: ... }` inside same loop that handles `output` events + 100ms ticker. | Lost activity-state transitions → stale sidebar icon. Self-heals on next hook event. |
| `internal/daemon/server_sse.go:44` (`handleSSE`) | **64** | One goroutine per SSE-connected GUI client; drains tightly in `for { select { case evt := <-sub.Ch(): writeSSEEvent(...) } }`. | Lost SSE-fanout events for remote GUIs. Larger buffer because remote network jitter / HTTP flush latency can stall the drain. |
| `internal/server/server_*_test.go` (multiple) | **4** | Test helpers. Not relevant to production. |
| `internal/server/server_test.go` | **8** | Test helpers (`srv.NotifyBroker().Subscribe(8)`). |

**Buffer asymmetry verified:** `8` for the local GUI (lowest-latency drain, gocui event loop), `64` for the remote SSE fan-out (longest drain latency, network in the path).

### 4.1 Recommended GUI buffer size for monocle (concrete)

**Recommendation: 16, with a tactical bump to 32 if the runtime-plane drain ever blocks on egui's frame thread.**

**Justification:**

1. **lazyclaude uses 8** because its drain goroutine is **always running** and the only thing that can stall it is `a.gui.Update(func)` enqueueing a closure on gocui's queue. gocui's queue is internally large; the drain rarely backs up by more than 1–2 events in practice. 8 has 6 events of safety margin.
2. **monocle's GUI is egui, not gocui.** The fan-out target is egui's repaint scheduler via `egui::Context::request_repaint`, which is non-blocking. But the in-between layer is whatever Rust thread holds the application state — likely a separate worker thread that consumes broker events and updates the `Arc<RwLock<AppState>>`. If that thread takes a `RwLock` write under contention with the egui paint thread, it can stall for one frame (~16ms at 60fps).
3. **One hook event burst is typically a tool invocation sequence: `PreToolUse → Notification (if permission needed) → Stop` per tool call.** Claude can fire 5–10 tool calls in rapid succession during a single iteration. With permission popups, that's up to 30 events in a burst.
4. **Calculation:** `1 frame stall (16ms) × peak hook rate (estimate 1 event/2ms during a tool burst) = 8 events queued during a worst-case stall`. Add 2x safety = **16**. If profiling shows multi-frame stalls (which would indicate a different bug), bump to 32.
5. **Why not larger?** Each `model.Event` is small (single-digit pointers + a couple of strings) but the memory cost of buffering 64+ is still ~5 KB at typical event sizes. More importantly, **a larger buffer hides the underlying problem** — if the GUI drain is consistently slow enough to fill 32 events, the problem is the GUI drain, not the broker, and a bigger buffer just postpones the diagnostic.
6. **Asymmetry with the remote-fanout case:** monocle's remote-plane equivalent (if it exists) should follow lazyclaude and use 64 for the same reason — network latency in the drain path.

**For BC-RUNTIME-003 (monocle's equivalent of BC-BROKER-003):** make the buffer size a configurable runtime parameter with a documented default of 16. The contract should be: "Buffered to `N=16` events; on overflow, the event is dropped and a drop counter increments."

---

## 5. Concurrency invariants (formalized)

| Invariant | Holds because | Verified by |
|-----------|---------------|-------------|
| Per-subscriber FIFO order: a single subscriber sees events in the order they were `Publish`ed | Both `Publish` and `Cancel/Close` acquire the same mutex, so the channel send for event N completes before the mutex is released; the next `Publish` blocks on the mutex. | `TestBroker_SubscribeAndPublish` (broker_test.go:44-60) |
| Cross-subscriber order undefined within one Publish | `for _, s := range b.subs` is Go map iteration, randomized per iteration | (no test — by design) |
| No send-on-closed-channel panic | `Cancel` deletes from `subs` *and* closes the channel under the mutex; subsequent `Publish` holding the mutex sees an empty entry in `subs` and skips. `Close` flips `closed` first, blocking future publishes. | `TestBroker_CancelledSubDoesNotReceive` (broker_test.go:288-313), `TestBroker_PublishAfterClose` (broker_test.go:233-242) |
| Cancel + Close + Cancel never double-closes | `sync.Once` on `Subscription`; `Close` replaces the map; `Cancel` checks `exists` before closing | `TestBroker_CancelIdempotent`, `TestBroker_CloseIdempotent` |
| No goroutine leaks | `goleak.VerifyTestMain(m)` covers all tests; no goroutine is spawned by `Subscribe` or `Publish` themselves — the broker is purely synchronous on the publisher's goroutine. | `broker_test.go:14-16` |
| HasSubscribers + Publish is not atomic | Sequential mutex acquisitions; documented benign race | server-r2 documented it; no test (correctly so) |

**Backpressure to publishers: none.** A publisher cannot tell that a subscriber is slow. There is no `BackpressureCh`, no `WaitForDrain`, no `MaxBacklog` mechanism. This is consistent with the explicit non-blocking publish contract — backpressure would necessarily block, defeating the contract.

---

## 6. Restart-resilience: WithBroker / ownsBroker verified

**Mechanism verified at the source:**

| Location | Line | Role |
|----------|------|------|
| `internal/server/server.go:49-50` | `notifyBroker *event.Broker[model.Event]; ownsBroker bool` | Struct fields |
| `internal/server/server.go:69-78` | `WithBroker(b)` | Functional option; sets `s.notifyBroker = b; s.ownsBroker = false` |
| `internal/server/server.go:101-105` | Fallback in `New()` | If no `WithBroker` opt, creates an owned broker and sets `ownsBroker = true` |
| `internal/server/server.go:178-180` | `Stop()` | `if s.ownsBroker { s.notifyBroker.Close() }` — only the owner closes |

**Single-broker injection lifecycle (TUI launch path):**

```
cmd/lazyclaude/root.go:104  notifyBroker := event.NewBroker[model.Event]()
cmd/lazyclaude/root.go:105  lc.Register("notify-broker", func() { notifyBroker.Close() })
cmd/lazyclaude/root.go:107  inProcessSrv := tryStartInProcessServer(..., notifyBroker)
cmd/lazyclaude/root.go:356  app.SetNotifyBroker(notifyBroker)          ← GUI subscribes
cmd/lazyclaude/root.go:430  srv := server.New(cfg, ..., server.WithBroker(broker))   ← server publishes
```

**Verification of "broker outlives server restart":**

- Test `TestServer_WithBroker_StopDoesNotCloseBroker` (`server_broker_test.go:279-322`) explicitly:
  1. Creates an external `event.NewBroker[model.Event]()`
  2. Injects it via `server.WithBroker(externalBroker)`
  3. Subscribes to the external broker (`externalBroker.Subscribe(4)`)
  4. Stops the server
  5. Publishes to the external broker
  6. Asserts the subscription **still receives the event**

**Confirmed:** the broker survives server `Stop()` when injected. GUI subscriptions remain valid.

**What `lc.Register("notify-broker", ...)` does:** the `lc` is the GUI lifecycle manager (`internal/gui/lifecycle`). It registers cleanup callbacks invoked when the TUI exits. `notifyBroker.Close()` runs **only** on TUI shutdown, not on server restart. So:

| Event | Broker state |
|-------|--------------|
| TUI start → `New(... WithBroker(broker))` | Broker created, server starts publishing |
| Server `Stop()` (mid-session, e.g. port change) | Broker survives (`ownsBroker=false`) |
| Server `New(... WithBroker(broker))` again | Same broker reused, same subscribers still attached |
| TUI exit → `lc.Cleanup()` | Broker `Close()` invoked, all subs receive channel-close |

**What happens if the server is restarted with a *new* broker (deliberate or accidental):** The old subscribers (GUI `notify_loop.brokerSub`) keep listening to the old broker, which never publishes again, and the new server publishes to the new broker, which has no subscribers. **Silent decoupling — no error, no log.** There is no test for "WithBroker called with a different broker on a New call." This is a hazard worth calling out in the spec — the contract should be: **the broker passed to `WithBroker` is global to the TUI lifetime and must not be replaced.**

### 6.1 P1 finding: no protection against accidental broker replacement

| Severity | Description |
|----------|-------------|
| **P1** | If two distinct `event.NewBroker` instances are created in TUI startup (e.g. refactor accident, conditional path), and one is given to the GUI via `SetNotifyBroker` while the other is given to the server via `WithBroker`, the system enters a "broker decoupled" state: server publishes events that no one receives; GUI subscribes to a broker that nobody publishes to. **No log line, no panic, no test.** The GUI's `outputPending` fallback path will still partially recover (file-based polling at 100ms cadence), so the symptom is "popups delayed / activity icon lags by ~100ms" not "broken." Hard to diagnose. |

**Monocle mitigation:** make the broker a singleton (`OnceLock<Arc<Broker<Event>>>` in Rust idiom) injected from one place, and have the server constructor accept `Arc<Broker<...>>` non-optionally — no functional-options fallback that creates its own.

---

## 7. Test coverage assessment (going beyond the audit's "citations exact")

The audit verified that all 6 cited line ranges in BC-BROKER-001..012 are exact. This deepening looks for **edge cases the tests do not cover**:

| Edge case | Test coverage? | Risk |
|-----------|---------------|------|
| `bufSize == 0` (unbuffered) with **slow reader** | Asserts non-panic only (`broker_test.go:320-333`). Does not verify delivery. | Low — by design, drops are expected. |
| `bufSize < 0` (negative) | No test. Would panic in `make(chan T, -1)`. | Low — Go runtime handles it; would crash the caller. Worth a defensive check on port. |
| Subscribe → Publish → Cancel → Subscribe interleaved across goroutines under `-race` | `TestBroker_ConcurrentPublish` covers concurrent **publishers** but not concurrent **subscribers** with churn. | Low — single mutex serializes everything; race detector would have caught it. |
| Many subscribers (~1000) | `BenchmarkBroker_Publish` uses 1 subscriber. No stress test for high fan-out. | Low — map iteration is O(N); single mutex hold time grows linearly. Not pathological until N > 10k. |
| Type instantiations beyond `string`, `int`, `model.Event` | Only those three are exercised. | Zero — Go generics are monomorphized; behavior is type-independent. |
| Cancel called from inside the receive loop (i.e. `for ev := range sub.Ch() { sub.Cancel() }`) | Not directly tested. Per `Close`/`Cancel` semantics, this should work: `Cancel()` closes `sub.ch`, the range loop terminates on the next iteration. | Low — verified by inspection. |
| Publish called from inside a subscriber's receive goroutine (re-entrant publish) | Not tested. **Would deadlock:** the receive goroutine holds nothing, calls `Publish`, which calls `b.mu.Lock`. If the original `Publish` is still on the stack holding the lock, this would deadlock. **But:** Go channels are not re-entrant under a single goroutine, so the original `Publish` cannot be on the stack of the same goroutine that's draining the channel — the drain runs in a different goroutine. So in practice this is safe, but the deadlock would occur if a subscriber's drain goroutine itself published while another goroutine was holding the publish mutex for an extended period (it doesn't, because publish never blocks). | Very low. |
| Subscribe → goroutine exits without Cancel | No test. The subscription leaks: `b.subs` still has the entry, and the channel is never closed. Future publishes will keep trying to send. If buffer fills, future events drop silently — but no error. **This is a real leak path.** | **MEDIUM — worth a contract.** See §7.1. |

### 7.1 New BC-BROKER-013 to add: subscriber lifecycle invariant

**BC-BROKER-013 (new from this deepening):** A subscriber that exits its receive goroutine without calling `Cancel()` leaks one map entry in `b.subs` and one unbuffered/buffered channel. Future `Publish` calls will continue attempting to send to the orphaned channel; if the channel is buffered and fills, all events to that orphan are silently dropped; if the channel is unbuffered, every event drops. **There is no mechanism to detect leaked subscribers.** `HasSubscribers()` returns true as long as the orphan is in the map.

**Evidence:** no test exists for this case; implementation analysis at `broker.go:68-74` (Publish iterates `subs` unconditionally) and `broker.go:78-82` (HasSubscribers counts every map entry, alive or orphaned).

**Confidence:** HIGH (by inspection)
**Importance:** load-bearing — monocle must add an explicit "every Subscribe is paired with a deferred Cancel" convention, or use scope-guards (RAII Drop in Rust) to make this impossible by construction.

---

## 8. Errors and panics catalogued

| Operation | Can panic? | When | Mitigation in code |
|-----------|-----------|------|--------------------|
| `NewBroker[T]()` | No | — | — |
| `Subscribe(bufSize)` | **Yes** if `bufSize < 0` | `make(chan T, -1)` panics | None — caller responsibility |
| `Subscribe` after `Close` | No | Returns sub with already-closed channel | `broker.go:48-51` |
| `Publish` after `Close` | No | Early return on `closed` flag | `broker.go:64-66` |
| `Publish` to full subscriber | No | `select default` arm | `broker.go:71-73` |
| `Cancel` twice | No | `sync.Once` | `broker.go:110` |
| `Cancel` after `Close` | No | `exists` check | `broker.go:117` |
| `Close` twice | No | Early return on `closed` flag | `broker.go:90-92` |
| Send on closed channel | Possible only if Cancel doesn't hold the mutex during close | Cancel does hold the mutex → safe | `broker.go:112-119` |

No `recover()`, no error returns, no contextual cancellation. The broker is a deliberately minimal primitive — it does not engage with `context.Context` at any point.

---

## 9. Errata / corrections to prior passes

### 9.1 Pass-3 BC-BROKER-003 wording clarification

Current Pass-3 text:

> **Postconditions:** Publishing 100 events completes within 2 seconds (i.e., does not block on the slow subscriber). Some events are silently dropped for that subscriber.

This is correct. But the **"silently"** word should be reinforced — no log, no counter, no callback. The implementation has zero observability for drops. Recommend reading "silently dropped" as "completely silent — no observable signal whatsoever." See §3 of this round.

### 9.2 server-r2 §"single-mutex correction" — propagated here

The Pass-8 architectural description that the broker has "an internal mutex" (singular, vague) is now formally verified: **exactly one `sync.Mutex`, no RWMutex variant, no shard, no atomic.** This deepening propagates the correction.

### 9.3 No prior contract for orphaned subscribers (BC-BROKER-013)

The Pass-3 contracts assume every Subscribe is paired with a Cancel. No contract spells out what happens if it isn't. See §7.1.

---

## 10. Coverage for monocle crystallization

| Question | Answer (this round) |
|----------|---------------------|
| Locking discipline | Single `sync.Mutex`; all methods serialize through it. |
| Buffer-size policy | Per-subscriber, caller-chosen, no clamping. lazyclaude uses 8 (GUI) and 64 (SSE). |
| BC-BROKER-003 drop mechanism | Silent drop in `select default`. No log, no metric, no callback. **Monocle should add a per-sub drop counter.** |
| Subscription identification | Monotonic `uint64` ID assigned at Subscribe time; never reused. |
| Leak on goroutine exit | **YES** — subscriber that exits without Cancel leaks one map entry + one channel indefinitely. See BC-BROKER-013 (new). |
| Concurrency ordering | Per-subscriber FIFO guaranteed; cross-subscriber order undefined. Verified by single-mutex. |
| HasSubscribers semantics | Snapshot under mutex; sequential race with Publish/Cancel by design (benign — events drop silently if subscribers vanish between check and publish). |
| Error paths & panics | Only one panic vector: `Subscribe(-1)`. All other states are no-ops by design. |
| Restart resilience (WithBroker / ownsBroker) | **Verified by test**: external broker survives server `Stop()`; GUI subs stay attached. Risk: accidental broker replacement is silently undetectable (P1 finding). |
| Backpressure | None by design. Non-blocking publish is the explicit contract. |
| Recommended monocle GUI buffer | **16** (justified in §4.1). 64 for any remote-fanout equivalent. |

---

## Delta Summary

- **New items added:**
  - 1 new contract: **BC-BROKER-013** — orphaned-subscriber leak invariant
  - 1 new finding: **P1 — accidental broker replacement is silently undetectable** (no test, no log, partial fallback masks the bug)
  - 1 architectural recommendation: **monocle GUI buffer = 16**, with per-subscriber drop counter to make BC-BROKER-003 observable
- **Existing items refined:**
  - BC-BROKER-003: confirmed completely silent drop; no log, no metric, no callback. Mechanism cleared.
  - Pass-8 §architecture (broker "internal mutex"): refined to single `sync.Mutex`, all methods. The server-r2 correction is now formally propagated.
  - WithBroker/ownsBroker pattern: lifecycle traced end-to-end (root.go:104 → app:356 → server.New:430), and the cross-restart invariant confirmed by `TestServer_WithBroker_StopDoesNotCloseBroker`.
- **Remaining gaps:**
  - No test for `Subscribe(-1)` defensive behavior (low — caller-responsibility).
  - No test for high fan-out (1000+ subscribers); not relevant for lazyclaude's use case (single-digit subscriber counts).
  - No test for the "accidental broker replacement" hazard described in §6.1. This is appropriately a spec-level (not test-level) concern.

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: This round produced (a) a new behavioral contract (BC-BROKER-013) for orphaned subscribers that no prior pass identified, (b) a P1 finding on broker-replacement detectability that affects how monocle structures broker injection, (c) a concrete and justified buffer-size recommendation for monocle (16) that the prior passes left unanswered, and (d) a definitive mechanism description for BC-BROKER-003's silent drop (no log, no metric, no callback — a property that the audit specifically flagged as needing verification). Removing this round's findings would change how monocle specs the broker primitive — specifically, monocle would inherit silent drops by default and have no policy on subscriber leak detection.

## Convergence Declaration

**Another round needed.** The substantive findings above (BC-BROKER-013, P1 broker-replacement, monocle buffer recommendation, drop-observability) all warrant a second round to (1) verify BC-BROKER-013 against any production call sites that might inadvertently leak subscribers, (2) confirm whether any monocle-relevant edge case was missed in §7 (specifically: concurrent Subscribe+Close, and the high-fan-out stress contour), and (3) cross-check the WithBroker hazard against the actual `tryStartInProcessServer` code path to confirm it cannot accidentally pass a different broker.

The audit recommended 1 round; the Iron Law (run until honest NITPICK, cap 5) overrides the audit's count. One more round at minimum.

## State Checkpoint

```yaml
pass: B
subject: internal/core/event/broker.go
round: 1
status: complete
files_scanned:
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/event/broker.go (122 LOC, fully read)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/event/broker_test.go (385 LOC, fully read)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/server.go (WithBroker/ownsBroker section, lines 40-235)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/server_broker_test.go (lines 227-322 — WithBroker tests)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/gui/notify_loop.go (full)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/gui/app.go (broker-drain section, lines 255-355, 418-432)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/gui/app_broker_test.go (full)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/daemon/server_sse.go (full)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/cmd/lazyclaude/root.go (broker section, lines 95-225, 430)
prior_passes_referenced:
  - any-context-lazyclaude-pass-3-behavioral-contracts.md (BC-BROKER-001..012 verified)
  - any-context-lazyclaude-pass-B-deep-server-r2.md (single-mutex correction confirmed)
  - any-context-lazyclaude-pass-B-deep-gui-r1.md (notify_loop subscriber pattern)
  - any-context-lazyclaude-pass-B5-coverage-audit-v2.md (audit recommended this round)
timestamp: 2026-05-11T00:00:00Z
novelty: SUBSTANTIVE
next_round_targets:
  - Verify BC-BROKER-013 against every production Subscribe call site (audit goroutine-exit pairing)
  - Check tryStartInProcessServer for any path that could pass a second broker (P1 hazard)
  - Investigate whether concurrent Subscribe+Close has any window for a race not covered by single-mutex
  - Verify Publish-fan-out cost under high subscriber count (>100) is acceptable
```
