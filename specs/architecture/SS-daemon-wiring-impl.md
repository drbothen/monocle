---
document_type: architecture-subsystem-doc
level: L4
section: "daemon-wiring-impl"
subsystem: SS-04
version: "1.3.0"
status: approved
producer: vsdd-factory:architect
phase: phase-3
timestamp: 2026-06-03T00:00:00Z
inputs:
  - {path: crates/monocle-runtime/src/main.rs}
  - {path: crates/monocle-runtime/src/lifecycle.rs}
  - {path: crates/monocle-runtime/src/server.rs}
  - {path: crates/monocle-runtime/src/state.rs}
  - {path: crates/monocle-runtime/src/lock.rs}
  - {path: crates/monocle-runtime/src/event_bus.rs}
  - {path: crates/monocle/src/auto_start.rs}
  - {path: crates/monocle-runtime/src/hooks/pre_tool_use.rs}
  - {path: .factory/specs/architecture/SS-daemon-wiring.md, version: "1.3.0"}
  - {path: crates/monocle-runtime/tests/daemon_start_sequence.rs}
  - {path: crates/monocle-runtime/tests/graceful_shutdown.rs}
input-hash: "[pending]"
traces_to: architecture/SS-daemon-wiring.md
project: monocle
---

# SS-04 Implementation Spec: Daemon Binary Wiring

## Purpose

`monocle-runtime/src/main.rs` is currently a stub (S-016 deliverable). It acquires a
`DaemonLock` with a hardcoded port `39_001` and enters an infinite `sleep(1s)` loop. The
daemon never serves HTTP, never writes `hooks-settings.json`, never runs the UDS accept
loop, and never initializes a tracing subscriber. The product does not function.

This document specifies the exact changes the implementer must make to wire the existing
library functions into a live, serving daemon. All library functions already exist and are
tested; only the composition in `main.rs` and one signature change in `lifecycle.rs` are
required.

---

## Decision: Listener Seam (Option A)

### The Problem

`daemon_start_sequence` binds a `TcpListener` at step 3 (local variable `listener`) to
record the OS-assigned port. That listener is used to read the port, then **dropped at
function return**. `run_server(state, listener)` never receives it. The HTTP server is
never served.

### Three Options Considered

**(a) Change `daemon_start_sequence` return type to `(Arc<DaemonState>, TcpListener)`**
so `main()` passes the listener to `run_server`.

**(b) Add a higher-level `run_daemon(runtime_dir)` that combines start_sequence +
run_server into one call**; `main()` calls one function.

**(c) Store the listener on `DaemonState`** (requires `tokio::net::TcpListener` as a
field, makes `DaemonState` non-Clone, and ties server binding to state construction).

### Decision: Option (a)

**Change `daemon_start_sequence` to return `(Arc<DaemonState>, TcpListener)`.**

Rationale:

- **Minimal change surface.** Only the return type and call-sites in tests change. No new
  public functions, no trait changes, no new crate-level abstractions.
- **SOQ-2 invariant preserved.** The listener is bound at step 3 inside the function, the
  port is read from it there, the lock file is written at step 8 with that port, and the
  listener is returned — all in the same scope. No timing gap.
- **INV-6 preserved.** The cleanup-on-failure logic (`daemon_lock.release()` on post-step-8
  failure) remains entirely inside the function, unchanged.
- **`run_server` unchanged.** `run_server(state: Arc<DaemonState>, listener: TcpListener)`
  already has the correct signature; it just needs to receive the listener.
- **Testability unchanged.** Integration tests that call `daemon_start_sequence` and inspect
  filesystem artifacts are unaffected — they don't call `run_server` and can ignore the
  returned listener (or bind it on a background task if the test needs the server).
- **Option (b) rejected** because it creates a new public entry point that is harder to test
  incrementally (must test the combined function, not the parts). Existing tests for
  `daemon_start_sequence` continue to work against the unchanged library behavior.
- **Option (c) rejected** because storing a bound socket on `DaemonState` permanently ties
  port lifecycle to state lifecycle, makes `DaemonState` non-serializable, and requires
  introducing `Option<TcpListener>` (which is `None` in every unit test that constructs
  `DaemonState::new()` — ADV-W5GATE-HIGH-001 class of defect).

---

## Changes Required

### 1. `crates/monocle-runtime/src/lifecycle.rs`

**Function:** `daemon_start_sequence`

**Current signature:**
```rust
pub async fn daemon_start_sequence(
    runtime_dir: &Path,
) -> Result<std::sync::Arc<crate::state::DaemonState>, DaemonStartError>
```

**New signature:**
```rust
pub async fn daemon_start_sequence(
    runtime_dir: &Path,
) -> Result<(std::sync::Arc<crate::state::DaemonState>, tokio::net::TcpListener), DaemonStartError>
```

**Change at step 3 (line ~441):** The `listener` local variable is already correct. No
behavior changes inside the function. Only the return statement changes:

```rust
// Current (line ~622):
Ok(daemon_state)

// New:
Ok((daemon_state, listener))
```

**No other changes inside `daemon_start_sequence`.** All 12 steps, all cleanup paths, all
tracing, SOQ-2 ordering, and INV-6 logic are unchanged.

### 2. `crates/monocle-runtime/src/main.rs`

**Full replacement.** The stub is removed entirely. The new `main.rs` must:

1. Call `setsid()` and block SIGHUP (already correct in stub — keep as-is).
2. Initialize a tracing subscriber to stderr **before** any fallible operation (resolves
   ADV-W4GATE-MED-002). Use `tracing_subscriber::fmt().with_writer(std::io::stderr).init()`.
   This must be the first statement after the process-session setup, before `resolve_runtime_dir`.
3. Resolve and ensure the runtime directory (already in stub — keep logic, remove the
   separate `DaemonLock::acquire` call).
4. Build a `tokio` multi-thread runtime via `tokio::runtime::Builder::new_multi_thread`
   (see §Tokio Runtime Choice below).
5. Inside the runtime: call `daemon_start_sequence(&runtime_dir).await`. On error: call
   `exit_with(DaemonExit::StartupFailure)`.
6. Receive `(state, listener)` from step 5.
7. Call `run_server(Arc::clone(&state), listener).await` to serve HTTP.
   The UDS accept loop is already spawned inside `daemon_start_sequence` (step 10 of the
   sequence) — no additional UDS wiring is needed in `main()`.
8. After `run_server` returns (graceful shutdown triggered): release the lock, remove
   `hooks-settings.json`, and call `exit_with` with the appropriate `DaemonExit` variant
   (see §Shutdown Sequence in main()).

#### Tokio Runtime Choice

Use a manually-built `tokio::runtime::Builder` (NOT `#[tokio::main]`). Rationale: `setsid()`
and `sigprocmask` must execute on the process main thread **before** tokio spawns its thread
pool. `#[tokio::main]` hands control to tokio before any user code runs; the signal setup
in the stub's `main()` body executes after tokio is already managing threads, which is safe
but can be confusing. Using `Builder::new_multi_thread().enable_all().build()` lets the
implementer guarantee that the synchronous setup (setsid, sigprocmask, tracing init, runtime
dir resolution) is complete before the async executor starts.

#### main() Pseudo-Structure

```rust
fn main() {
    // Step 1: Session detach (synchronous, must precede tokio).
    let _ = nix::unistd::setsid();
    // Block SIGHUP.
    let mut hup_mask = nix::sys::signal::SigSet::empty();
    hup_mask.add(nix::sys::signal::Signal::SIGHUP);
    let _ = nix::sys::signal::sigprocmask(SigmaskHow::SIG_BLOCK, Some(&hup_mask), None);

    // Step 2: Tracing subscriber — BEFORE any fallible operation.
    // Daemon errors are logged to stderr; the parent process / shell will redirect stderr
    // to a log file if needed. SS-conventions-anti-patterns.md §Logging: no println!.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Step 3: Resolve runtime directory (synchronous).
    let runtime_dir = match monocle_runtime::lifecycle::resolve_runtime_dir() {
        Ok(dir) => dir,
        Err(e) => {
            tracing::error!(error = %e, "daemon: failed to resolve runtime directory");
            monocle_runtime::lifecycle::exit_with(DaemonExit::StartupFailure);
        }
    };
    if let Err(e) = monocle_runtime::lifecycle::ensure_runtime_dir(&runtime_dir) {
        tracing::error!(error = %e, "daemon: failed to create runtime directory");
        monocle_runtime::lifecycle::exit_with(DaemonExit::StartupFailure);
    }

    // Step 4: Build tokio runtime.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "daemon: failed to build tokio runtime");
            monocle_runtime::lifecycle::exit_with(DaemonExit::StartupFailure);
        });

    // Step 5-8: Async block.
    rt.block_on(async move {
        // Step 5: Full 12-step start sequence.
        let (state, listener) =
            match monocle_runtime::lifecycle::daemon_start_sequence(&runtime_dir).await {
                Ok(pair) => pair,
                Err(e) => {
                    tracing::error!(error = %e, "daemon: start sequence failed");
                    monocle_runtime::lifecycle::exit_with(DaemonExit::StartupFailure);
                }
            };

        tracing::info!("daemon: start sequence complete; serving HTTP");

        // Step 6: Run HTTP server.
        // run_server blocks until a shutdown signal fires (SIGTERM, SIGINT, or
        // POST /shutdown). The UDS accept loop was already spawned inside
        // daemon_start_sequence (step 10 of the sequence).
        let serve_result = monocle_runtime::server::run_server(
            std::sync::Arc::clone(&state),
            listener,
        )
        .await;

        if let Err(e) = serve_result {
            tracing::error!(error = %e, "daemon: HTTP server returned error");
        }

        // Step 7: Graceful shutdown — release lock + remove hooks-settings.json.
        // BC-2.01.004 PC-7: DaemonLock::release() BEFORE exit_with.
        // BC-2.04.010 PC-5: remove_hooks_settings BEFORE exit_with.
        //
        // UDS transport cleanup: UdsTransport::cleanup() removes monocle.sock.
        // This is stored on DaemonState.uds_transport (Some variant).
        if let Some(transport) = state.uds_transport.as_ref() {
            if let Err(e) = transport.cleanup() {
                tracing::warn!(error = %e, "daemon: UDS socket cleanup failed; continuing");
            }
        }

        let lock_taken = state.daemon_lock.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(lock) = lock_taken {
            if let Err(e) = lock.release() {
                tracing::warn!(error = %e, "daemon: lock release failed; continuing");
            }
        }

        let _ = monocle_runtime::lifecycle::remove_hooks_settings(&runtime_dir);

        // Step 8: Determine exit variant.
        // force_exit flag is set by post_shutdown handler when a second POST /shutdown
        // arrives during drain (EC-050; DaemonExit::AdminForceStop = exit code 2).
        use std::sync::atomic::Ordering;
        let exit_reason = if state.force_exit.load(Ordering::SeqCst) {
            monocle_runtime::lifecycle::DaemonExit::AdminForceStop
        } else {
            monocle_runtime::lifecycle::DaemonExit::Graceful
        };

        tracing::info!(?exit_reason, "daemon: clean shutdown complete");
        monocle_runtime::lifecycle::exit_with(exit_reason);
    });
}
```

### 3. Port Consistency — Remove Hardcoded 39001

**Current stub bug (main.rs line 83):**
```rust
let _lock = match monocle_runtime::lock::DaemonLock::acquire(&runtime_dir, 39_001) {
```

This hardcoded port `39_001` must be **removed entirely**. The correct flow is:
- `daemon_start_sequence` binds `TcpListener::bind("127.0.0.1:0")` at step 3 → receives
  an OS-assigned ephemeral port → writes that port into `monocle.lock` and
  `hooks-settings.json`.
- `DaemonLock::acquire` with a hardcoded port is the S-006 legacy interface; it is NOT
  called in the new main().
- The CLI poller, TUI, and Claude Code's curl commands all read the port from `monocle.lock`
  (the `port` field) — they never use a hardcoded constant.
- The new `main()` never calls `DaemonLock::acquire` directly; that call is encapsulated
  inside `daemon_start_sequence` (specifically `write_lock_file` at step 8).

**Verification:** After the fix, `grep -r "39_001\|39001" crates/` must return zero hits
in non-test source files.

### 4. DaemonState Completeness (ADV-W5GATE-HIGH-001, ADV-W3GATE-MED-002)

**These are already fixed in `daemon_start_sequence`.** The reading of the actual code
confirms:

- `ring: Some(ring)` — line ~566 of lifecycle.rs. The `Arc<RingBuffer>` is constructed
  at step 4 and stored as `Some`. No change needed.
- `event_bus_tx: Some(Arc::new(event_tx))` — line ~579. Constructed at step 5.
- `drop_counter: Some(drop_counter)` — line ~580.
- `session_registry: Some(...)` — line ~581.
- `pending_decisions: Some(...)` — line ~582.
- `ipc_subscribers: Some(...)` — line ~583.
- `uds_transport: Some(uds_transport)` — line ~584.

The `daemon_start_sequence` function already fully wires `DaemonState`. ADV-W5GATE-HIGH-001
and ADV-W3GATE-MED-002 are resolved once `main()` calls `daemon_start_sequence` instead of
the stub `DaemonLock::acquire`. **No changes to `daemon_start_sequence` internals are
required beyond the return-type change.**

### 5. UDS Cleanup in main()

The `DaemonState.uds_transport` field holds a `monocle_ipc::uds::UdsTransport`. On graceful
shutdown, `transport.cleanup()` removes `monocle.sock`. The implementer must call this in
the shutdown sequence in `main()` (step 7 above). This resolves the case where `DaemonLock::release`
removes `monocle.lock` but the socket file is left on disk.

Note: `UdsTransport` is stored as `Option<UdsTransport>` (not `Arc`) in `DaemonState`, but
`DaemonState` is `Arc`-wrapped. The implementer must call `.as_ref()` on the `Option` — do
not take ownership via `.take()` unless the `UdsTransport::cleanup` API requires it. Confirm
the `UdsTransport::cleanup` method signature before writing the shutdown path.

---

## Tests to Update

### Existing Tests in `daemon_start_sequence.rs`

Every test that calls `daemon_start_sequence(&runtime_dir).await` must be updated from:

```rust
daemon_start_sequence(&runtime_dir)
    .await
    .expect("daemon_start_sequence must succeed");
```

to:

```rust
let (_state, _listener) = daemon_start_sequence(&runtime_dir)
    .await
    .expect("daemon_start_sequence must succeed");
```

(or use a `let _ =` discard if the test only cares about filesystem artifacts).

Count: approximately 20 call-sites in `daemon_start_sequence.rs`. Search for
`daemon_start_sequence` in the file and update all of them. The `_listener` can be dropped
immediately in tests that do not need to serve HTTP — dropping the `TcpListener` releases
the port back to the OS, which is correct for isolation tests.

Tests that need to exercise `run_server` (e.g., a new E2E test in §E2E Verification
Contract below) must retain the listener and pass it to `run_server`.

### No Changes to `graceful_shutdown.rs`

The graceful shutdown tests construct `DaemonState` directly via `DaemonState::new()` and
use `build_server` + `tower::ServiceExt::oneshot`. They do not call `daemon_start_sequence`.
No changes needed.

---

## E2E Verification Contract

This section specifies what the **implementer must write** as a live-binary integration test
in a new file `crates/monocle-runtime/tests/daemon_e2e_serve.rs`. This test closes the
verification gap that allowed the stub to pass Phase 4.

### Test Setup

```rust
// Spawn the real monocle-runtime binary via `std::process::Command`.
// Use MONOCLE_RUNTIME_DIR=<tempdir> to isolate from the user's real runtime dir.
// The binary is located via CARGO_BIN_EXE_monocle-runtime (set by cargo for integration tests)
// or by building it first and resolving the target path.
```

### Required Assertions

The test **must** assert all of the following in order:

**AC-E2E-001 — Lock file appears with real OS-assigned port**

After spawning the daemon binary with `MONOCLE_RUNTIME_DIR=<tmpdir>`:
- Poll for `<tmpdir>/monocle.lock` to appear (max 5 seconds, 50ms poll interval).
- Parse the lock file JSON. Assert `port` field is present, is a `u16`, and is NOT 0 and
  NOT 39001.
- Assert `contract_version == "monocle-lock-v1"`.
- Assert `pid` matches the spawned process's PID.

**AC-E2E-002 — hooks-settings.json references the real port**

- After lock file appears: assert `<tmpdir>/hooks-settings.json` exists.
- Parse as JSON. Assert at least one active hook command URL contains
  `http://127.0.0.1:<port>/hooks/` where `<port>` matches the port from the lock file.
- Assert `<port>` is NOT 39001.

**AC-E2E-003 — GET /healthz returns 200**

- Issue `GET http://127.0.0.1:<port>/healthz`.
- Assert HTTP 200 and body `{"status":"ok", ...}`.

**AC-E2E-004 — POST /hooks/pre-tool-use is accepted with auth**

- Read the `token` field from the lock file. Strip the `monocle-v1:` prefix to get the
  64-hex auth token.
- Issue `POST http://127.0.0.1:<port>/hooks/pre-tool-use` with:
  - Header: `X-Monocle-Authorization: monocle-v1:<64-hex-token>`
  - Header: `Content-Type: application/json`
  - Body: a minimal valid `HookEnvelope` JSON for a `PreToolUse` event
- Assert HTTP 200 response (daemon accepted and processed the hook).

**AC-E2E-005 — UDS socket exists**

- Assert `<tmpdir>/monocle.sock` exists and is a socket (use `std::fs::metadata` +
  `std::os::unix::fs::FileTypeExt::is_socket`).

**AC-E2E-006 — SIGTERM causes graceful exit 0 + cleanup**

- Send SIGTERM to the daemon process.
- Wait up to 5 seconds for the process to exit.
- Assert exit code 0 (`DaemonExit::Graceful`).
- Assert `<tmpdir>/hooks-settings.json` does NOT exist (removed on graceful shutdown per
  BC-2.04.010 PC-5).
- Assert `<tmpdir>/monocle.lock` does NOT exist (released on graceful shutdown per
  BC-2.01.004 PC-7).

### Test File Location

`crates/monocle-runtime/tests/daemon_e2e_serve.rs`

The test requires the binary to be built. Use `#[cfg(feature = "...")]` or rely on
`cargo test --test daemon_e2e_serve` (integration tests are compiled alongside the crate's
binaries by cargo). Cargo provides `CARGO_BIN_EXE_monocle-runtime` as an environment
variable when running integration tests if the binary target is named `monocle-runtime`.
Confirm the binary name in `Cargo.toml` matches.

### Required Test Helper

```rust
fn wait_for_file(path: &std::path::Path, max_wait: std::time::Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < max_wait {
        if path.exists() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    false
}
```

---

## Invariants Preserved

| Invariant | How it is preserved |
|-----------|---------------------|
| SOQ-2 (port bind → lock write → hooks-settings write) | Unchanged inside `daemon_start_sequence`. Only the return type changes. |
| INV-6 (lock cleanup on post-step-8 failure) | Unchanged inside `daemon_start_sequence`. |
| `exit_with` is the sole `process::exit` call-site | `main()` calls `exit_with` for all exit paths. No direct `process::exit`. |
| SIGHUP blocked before tokio | `sigprocmask` executes synchronously in `fn main()` before `rt.block_on`. |
| Tracing subscriber initialized before any fallible path | `tracing_subscriber::fmt().init()` is the second statement in `main()`. |
| UDS accept loop runs concurrently with HTTP server | Spawned inside `daemon_start_sequence` as a `tokio::spawn` task. Still running when `run_server` is awaited. |
| Graceful drain: 10s window enforced at main() call-site | `run_server` uses `axum::serve(...).with_graceful_shutdown(...)` which waits for in-flight requests. The 10-second drain timeout (BC-2.01.004 INV-1) is enforced by `tokio::time::timeout` wrapping the `run_server` call in `main()`. On timeout, remaining connections are force-closed; cleanup tail runs normally (C2 fix — SS-daemon-wiring-impl.md §Fix Addendum C2). |

---

## Files the Implementer Must Change

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/lifecycle.rs` | Change `daemon_start_sequence` return type; update return statement. |
| `crates/monocle-runtime/src/main.rs` | Full replacement per §main() Pseudo-Structure above. |
| `crates/monocle-runtime/tests/daemon_start_sequence.rs` | Update ~20 call-sites to destructure `(state, listener)` from return value. |
| `crates/monocle-runtime/tests/daemon_e2e_serve.rs` | **NEW FILE** — live binary E2E test per §E2E Verification Contract. |

**Files the implementer must NOT change:**
- `crates/monocle-runtime/src/server.rs` — `run_server` signature is already correct.
- `crates/monocle-runtime/src/state.rs` — `DaemonState` is fully wired by `daemon_start_sequence`.
- `crates/monocle-runtime/src/lock.rs` — `DaemonLock` is unchanged.
- `crates/monocle-runtime/tests/graceful_shutdown.rs` — no call to `daemon_start_sequence`.
- Any other test files — they do not call `daemon_start_sequence`.

---

## CI Parity Checklist (pre-push, per CLAUDE.md)

Before opening a PR for this work:

1. `cargo clippy --workspace --all-targets -- -D warnings` (in worktree) — must be clean.
2. `python3 scripts/check_version_pins.py` (from repo root) — must pass.
3. `python3 scripts/check_structural_claims.py` (from repo root) — must pass.
4. `cargo test --workspace` — all 90 test suites must pass plus the new E2E test.
5. Confirm `grep -rn "39_001\|39001" crates/monocle-runtime/src/` returns zero hits.
6. Confirm `grep -rn "loop {" crates/monocle-runtime/src/main.rs` returns zero hits
   (the infinite sleep loop is gone).
7. Confirm `grep -rn "DaemonLock::acquire" crates/monocle-runtime/src/main.rs` returns
   zero hits (the S-006 legacy acquire call is gone from main).

---

## Open Follow-Ups (Non-Blocking for This Story)

These items are noted from the durable task register and are intentionally out of scope
for this story. They must NOT be fixed here unless explicitly re-scoped.

- **ADV-W5GATE-HIGH-002** — Duplicate S-009 handler dead code: separate tracking item.
- **ADV-W5GATE-MED-001** — S-017 UDS socket spurious WARN: not a wiring defect.
- **ADV-W4GATE-MED-002** — No tracing subscriber in the daemon binary: RESOLVED by this
  story (step 2 in main() pseudo-structure above).
- **FLAKY-TIMING-5MS** — `test_BC_2_06_010` 5ms boundary flake: widen at wave-7 gate.
- **F-S025-ADV37-DEFER-001** — STORY-INDEX stale BC→AC ranges: story-writer scope.

---

## Version Note

This document is version `1.0.0`. It is a new artifact and does not update any existing
versioned spec. `SS-daemon-wiring.md` v1.3.0 remains the authoritative behavioral
specification; this document is the implementer execution plan that closes the gap between
what `SS-daemon-wiring.md §Step 12` specifies and what `main.rs` currently does.

If the implementer discovers discrepancies between this spec and the actual code during
implementation, the implementer must surface them to the architect (routing table: architect
owns SS-04 wiring) rather than silently deviating from the design.

---

## Fix Addendum: Adversarial Findings C1 / C2 / I1 / I2

**Addendum version: 1.1.0 (2026-06-03). Supersedes the false invariant table claim "Graceful drain: 10s window enforced by run_server" and expands the implementer change-list.**

This addendum records the architect's decisions on three HIGH/MEDIUM adversarial findings
identified after the initial daemon-serve wiring was implemented on `feat/daemon-wire-serve`.
All three findings must be fixed in this story before the PR is opened. I2 is a
strengthened test assertion routed to the implementer.

---

### C1 — Dead Event Bus (HIGH)

#### Problem

`daemon_start_sequence` (lifecycle.rs line ~475) does:

```rust
let (event_tx, _event_rx) = tokio::sync::mpsc::channel::<EventBusHookEvent>(EVENT_BUS_CAPACITY);
```

The receiver `_event_rx` is immediately dropped. `event_bus_fan_out_task` and
`drop_counter_debounce_task` (event_bus.rs) are defined and fully correct but are never
spawned in production — they were only spawned in early tests. Every hook invocation calls
`try_publish_event`, which calls `tx.try_send(event)`. With no receiver alive, the channel
is permanently in the `Closed` state. Every `try_send` returns `Err(TrySendError::Closed)`,
which increments the drop counter and logs a WARN. In a production session with active hooks
this produces a WARN storm — one per hook event — that will fill log storage and mislead
operators.

#### Decision: Spawn the tasks in `daemon_start_sequence`; channel drains without TUI delivery

**The minimum correct fix is to retain `event_rx` and spawn `event_bus_fan_out_task` in
`daemon_start_sequence` so the channel has a live consumer.**

Key determination about what the fan-out task actually does in Phase 1:

Reading `event_bus.rs` lines 74–92: the fan-out task body is already implemented. It
receives events from the channel, calls `tracing::trace!` with `"fan-out: event received
(no TUI clients in Phase 1)"`, and loops. The comment says "Phase 1: no TUI clients to fan
out to." There are no `tui_clients` writes. The task simply drains the channel and exits
cleanly when the channel closes (all senders dropped). **In Phase 1 this is a drain-only
task — no IPC delivery occurs.**

This is sufficient to fix the defect. With a live receiver:
- `tx.try_send(event)` succeeds (channel has capacity)
- No `TrySendError::Closed`, no WARN, no drop counter increment on normal operation
- Full ribbon streaming to UDS subscribers remains S-032's scope per the existing Phase-1
  comment in the fan-out task body

The `drop_counter_debounce_task` must also be spawned. Its body (event_bus.rs lines
109–141) checks the counter every 100ms and calls `tracing::debug!` when it changes.
In Phase 1 it debounces to a debug log rather than a TUI IPC message, but it must run so
that the counter infrastructure is alive and correct for Phase 2 (S-032).

Both tasks must observe the shutdown signal so they exit cleanly without blocking the
tokio runtime shutdown.

#### Exact Change Required in `lifecycle.rs`

At step 5 (line ~475), change from:

```rust
// Step 5: Create bounded mpsc channel for event bus (4096 slots), drop counter AtomicU64.
let (event_tx, _event_rx) = tokio::sync::mpsc::channel::<crate::types::EventBusHookEvent>(
    crate::types::EVENT_BUS_CAPACITY,
);
let drop_counter: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
```

to:

```rust
// Step 5: Create bounded mpsc channel for event bus (4096 slots), drop counter AtomicU64.
// Retain event_rx so the channel has a live consumer; fan-out and debounce tasks are
// spawned below at step 5b. Dropping event_rx here would make the channel permanently
// Closed, causing every try_send to fail with TrySendError::Closed and produce a WARN per
// hook event (C1 fix — SS-daemon-wiring-impl.md §Fix Addendum C1).
let (event_tx, event_rx) = tokio::sync::mpsc::channel::<crate::types::EventBusHookEvent>(
    crate::types::EVENT_BUS_CAPACITY,
);
let drop_counter: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
```

Then, **after** `DaemonState` is fully constructed (it needs `drop_counter` and `ring` to
be set), add a new step 5b immediately before step 6 (EngineModuleRegistry):

```rust
// Step 5b: Spawn event-bus fan-out and drop-counter-debounce tasks.
// Both tasks are Phase-1 drain/log tasks only (no TUI IPC delivery in Phase 1).
// Full TUI ribbon streaming is S-032 scope (BC-2.05.004 PC-2 obligation).
// Both tasks hold a clone of Arc<DaemonState> and observe daemon_state.shutdown_rx so
// they exit cleanly within one scheduler tick of shutdown (C1 fix).
{
    let fan_out_state = Arc::clone(&daemon_state);
    tokio::spawn(crate::event_bus::event_bus_fan_out_task(event_rx, fan_out_state));

    let debounce_state = Arc::clone(&daemon_state);
    tokio::spawn(crate::event_bus::drop_counter_debounce_task(debounce_state));
}
```

**Important ordering constraint:** `daemon_state` must be fully constructed before step 5b
so the task closures receive a complete `Arc<DaemonState>`. Check lifecycle.rs to confirm
`daemon_state` is constructed at step ~line 565–595 (after ring, session_registry,
pending_decisions, ipc_subscribers are all wired). If it is, move the spawn block to after
that construction point. Do NOT spawn before `daemon_state` is finalized.

#### Shutdown behavior

`event_bus_fan_out_task` exits when `rx.recv()` returns `None`, which happens when all
`EventBusTx` senders are dropped. `DaemonState.event_bus_tx` is `Option<Arc<EventBusTx>>`.
The Arc is dropped when `DaemonState` is dropped (after `run_server` returns and main's
cleanup tail completes). This is correct: the fan-out task will exit as soon as the last
sender is released during the main shutdown tail — no explicit task abort required.

`drop_counter_debounce_task` runs a 100ms sleep loop. It does not hold the channel. It
needs explicit cancellation or shutdown signalling. The production-grade approach: pass the
`shutdown_rx` watch receiver into `drop_counter_debounce_task` and add a `tokio::select!`
branch that exits the loop when `shutdown_rx` fires.

**Required signature change for `drop_counter_debounce_task`:**

```rust
// Current (event_bus.rs line 109):
pub async fn drop_counter_debounce_task(state: Arc<DaemonState>) { ... }

// New:
pub async fn drop_counter_debounce_task(
    state: Arc<DaemonState>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) { ... }
```

Loop body change (add select! around the interval tick):

```rust
loop {
    tokio::select! {
        _ = interval.tick() => {
            // Existing debounce logic (check counter, log if changed).
            let current = match state.drop_counter.as_ref() { ... };
            if current != last_sent {
                last_sent = current;
                tracing::debug!(drop_count = current, "drop counter update ...");
            }
        }
        _ = shutdown_rx.changed() => {
            tracing::debug!("drop_counter_debounce_task: shutdown signal received; exiting");
            return;
        }
    }
}
```

Call-site in step 5b:

```rust
let debounce_state = Arc::clone(&daemon_state);
let debounce_shutdown_rx = daemon_state.shutdown_rx.clone();
tokio::spawn(crate::event_bus::drop_counter_debounce_task(
    debounce_state,
    debounce_shutdown_rx,
));
```

#### Deferral anchor (S-032)

The fan-out task body (event_bus.rs lines 81–91) already contains the S-032 marker:
"Future (S-021/S-022): iterate tui_clients, write with 50ms timeout per client, remove
failed/slow clients from the list." This deferral is legitimate: full ribbon streaming
requires S-021/S-022 IPC wiring, which BC-2.05.004 PC-2 obligation anchors to S-032.

**No new tech-debt-register entry is required.** The deferral target is the real story
`S-032` (Wave 8, EPIC-05, BC-2.05.004 PC-2 obligation, in STORY-INDEX v5.33 as
"deferred daemon fan-out"). The comment in event_bus.rs constitutes a code-level anchor
per CLAUDE.md Principle 3.

#### Files affected by C1

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/lifecycle.rs` | Retain `event_rx`; add step 5b spawn block after `DaemonState` construction. |
| `crates/monocle-runtime/src/event_bus.rs` | Add `shutdown_rx` parameter to `drop_counter_debounce_task`; add `select!` shutdown branch. |

---

### C2 — 10s Drain Timeout Unimplemented (HIGH)

#### Problem

`run_server` (server.rs lines 195–198) calls:

```rust
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal)
    .await
```

`with_graceful_shutdown` stops accepting new connections once the signal fires, but it
waits indefinitely for in-flight requests to drain. There is no timeout. The doc-comment
in server.rs line 124 states "the 10-second drain window from BC-2.01.004 INV-1 is
enforced at the `main` call-site via `tokio::time::timeout`." The doc-comment in main.rs
line 156–158 repeats this claim: "10s drain window per BC-2.01.004 INV-1, enforced inside
run_server via tokio::signal." Both comments are false. BC-2.01.004 INV-1 requires the
10-second bound. Without it a slow or adversarial client can hold an in-flight connection
and prevent the daemon from ever exiting.

#### Decision: Enforce timeout in `main()` wrapping `run_server`; correct all false comments

The doc-comment in server.rs says "enforced at the `main` call-site" — which is exactly
where it belongs. `run_server` should not embed the timeout because it is a library
function that tests call directly (via `build_server` + oneshot). Embedding a hard 10s
timeout inside `run_server` would make graceful-shutdown tests time out unnecessarily.

**Enforcement point: wrap `run_server` in `main()` with `tokio::time::timeout`.**

The exact change in `main.rs` step 6 (currently lines 159–163):

```rust
// Current:
let serve_result = monocle_runtime::server::run_server(
    std::sync::Arc::clone(&state),
    listener,
)
.await;

// New:
const GRACEFUL_DRAIN_TIMEOUT_SECS: u64 = 10;
let serve_result = tokio::time::timeout(
    std::time::Duration::from_secs(GRACEFUL_DRAIN_TIMEOUT_SECS),
    monocle_runtime::server::run_server(
        std::sync::Arc::clone(&state),
        listener,
    ),
)
.await;

// Flatten the Result<Result<(), io::Error>, Elapsed>:
let serve_result = match serve_result {
    Ok(inner) => inner, // timeout did not fire; propagate io::Error if any
    Err(_elapsed) => {
        // BC-2.01.004 INV-1: 10s drain window exhausted. Force-close remaining connections.
        // axum drops the serve future here; any in-flight connections are aborted.
        // Proceed to cleanup and exit to prevent daemon from hanging indefinitely.
        tracing::warn!(
            timeout_secs = GRACEFUL_DRAIN_TIMEOUT_SECS,
            "graceful drain timeout exhausted; forcing close of remaining connections \
             (BC-2.01.004 INV-1)"
        );
        Ok(()) // treat as clean exit — cleanup tail runs normally
    }
};
```

After this, the existing `if let Err(e) = serve_result { ... }` block at line 165–169
handles the `io::Error` case unchanged.

#### Interaction with `force_exit` / second-SIGTERM path

The second-SIGTERM and `AdminForceStop` paths (server.rs SIGINT arm, main.rs step 8) are
not affected. Those paths fire a second shutdown signal, which is already handled. The
drain timeout is an outer bound that fires only if connections are still alive after the
initial graceful-shutdown signal. Both paths can coexist: the `force_exit` flag is still
read at step 8 to determine the exit code regardless of whether the timeout fired.

#### False doc-comment corrections

**server.rs line 123–124** — currently reads:
```
/// (the 10-second drain window from BC-2.01.004 INV-1
/// is enforced at the `main` call-site via `tokio::time::timeout`).
```

This is now correct after the main.rs fix. No change needed to server.rs prose — the
description matches the new reality. Verify after implementing.

**main.rs lines 156–158** — currently reads:
```
// Graceful shutdown: run_server uses axum::serve(...).with_graceful_shutdown(...)
// which waits for in-flight requests to complete (10s drain window per BC-2.01.004
// INV-1, enforced inside run_server via tokio::signal).
```

The claim "enforced inside run_server via tokio::signal" is false. Replace with:

```rust
// Graceful shutdown: run_server uses axum::serve(...).with_graceful_shutdown(...)
// which waits for in-flight requests to complete. The 10-second outer drain timeout
// (BC-2.01.004 INV-1) is enforced by the tokio::time::timeout wrapper here in main().
// If the timeout fires, remaining connections are force-closed and cleanup proceeds.
```

#### Drain-timeout subprocess test (AC-E2E-007)

The existing `daemon_e2e_serve.rs` has AC-E2E-006 (SIGTERM → exit 0 + cleanup within 5s).
A companion test must verify the 10s bound specifically for the slow-request case.

**AC-E2E-007 — Drain timeout fires within 10s under held connection:**

Test setup:
1. Spawn the real monocle-runtime binary. Wait for lock file (same as AC-E2E-001).
2. Issue an in-flight POST to a hook endpoint that the daemon will NOT complete quickly.
   Use the `MONOCLE_HOOK_DELAY_MS` env var (already supported via `state.hook_delay_ms`)
   set to a value beyond 10s (e.g., `MONOCLE_HOOK_DELAY_MS=15000`) to simulate a slow
   request. Issue the request in a background thread with a long client-side read timeout.
3. Send SIGTERM to the daemon while the slow request is in-flight.
4. Assert the daemon exits within approximately 11 seconds (10s drain + 1s tolerance).
5. Assert exit code 0 (`DaemonExit::Graceful` — timeout is not an error; the daemon
   proceeded to cleanup and exited cleanly).
6. Assert `monocle.lock` and `hooks-settings.json` are absent (cleanup ran).

**Important:** `MONOCLE_HOOK_DELAY_MS` is currently a per-handler delay injected via
`state.hook_delay_ms`. This field is set via a `DaemonState` test constructor, not an
env var. For the E2E subprocess test to use it, either:
- (a) Add env var `MONOCLE_HOOK_DELAY_MS` reading in `daemon_start_sequence` and write it
  to `DaemonState.hook_delay_ms`, OR
- (b) Use a different mechanism to create a slow in-flight request (e.g., issue a large
  body that the daemon processes slowly, or use the mock engine path).

**Decision: option (a)** — add `MONOCLE_HOOK_DELAY_MS` env var support in
`daemon_start_sequence` so the E2E test can inject a delay without code changes. This is
a test-only field and may be gated by `#[cfg(test)]` or by convention (env var absent in
production deployments).

If option (a) is too invasive for the implementer, option (b) is acceptable: write a
`/healthz` response delay by injecting a slowpath into the healthz handler for E2E testing
only. Confirm with the implementer which path is less risky.

The test file location is `crates/monocle-runtime/tests/daemon_e2e_serve.rs` (same file
as AC-E2E-001 through AC-E2E-006).

#### Files affected by C2

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/main.rs` | Add `tokio::time::timeout` wrapper around `run_server`. Fix false doc-comment at step 6. |
| `crates/monocle-runtime/tests/daemon_e2e_serve.rs` | Add AC-E2E-007 drain-timeout test. |

The false invariant in this document's own §Invariants Preserved table must also be
corrected. The row currently reads:

> | Graceful drain: 10s window enforced by run_server | `run_server` uses `axum::serve(...).with_graceful_shutdown(...)` which waits for in-flight requests. The 10-second drain window is enforced by the SIGTERM handler in `run_server` itself via `tokio::signal::unix`. |

Replace with:

> | Graceful drain: 10s window enforced at main() call-site | `run_server` uses `axum::serve(...).with_graceful_shutdown(...)` which waits for in-flight requests. The 10-second drain timeout (BC-2.01.004 INV-1) is enforced by `tokio::time::timeout` wrapping the `run_server` call in `main()`. On timeout, remaining connections are force-closed; cleanup tail runs normally. |

---

### I1 — In-Process Fallback Dead Port (MEDIUM)

#### Problem

`auto_start.rs:launch_daemon_in_process` (line ~438–448):

```rust
async fn launch_daemon_in_process(runtime_dir: &Path) -> DaemonHandle {
    let dir = runtime_dir.to_path_buf();
    tokio::spawn(async move {
        if let Err(e) = monocle_runtime::lifecycle::daemon_start_sequence(&dir).await {
            tracing::warn!(error = %e, "in-process daemon_start_sequence failed");
        }
    });
    DaemonHandle::Task(())
}
```

`daemon_start_sequence` now returns `(Arc<DaemonState>, TcpListener)`. The in-process
path destructures neither: the `Ok(pair)` arm implicitly drops the tuple, which drops the
`TcpListener`. Dropping the listener releases the OS port. The lock file records the
(now-dead) port. Any polling client reads a port that immediately refuses connections.
The in-process fallback claims `Connected` to a dead port.

#### Decision: Spawn `run_server` inside the in-process task (option a)

**Option (a): make the in-process fallback also run the server.**

Option (b) — "remove in-process fallback, treat missing binary as offline mode" — would
be a behavioral regression. BC-2.04.002 PC-4 specifies the in-process fallback as the
canonical recovery path when the subprocess binary is absent (test environments, dev
builds without the binary installed). Removing it would break all tests that exercise
auto-start without the subprocess binary.

Option (a) is the production-grade path. The in-process task must:
1. Retain `(state, listener)` from `daemon_start_sequence`.
2. Call `run_server(state, listener).await` so HTTP is served.
3. Run the same shutdown tail that `main()` runs (UDS cleanup, lock release, hooks removal).

**Exact change to `launch_daemon_in_process`:**

```rust
async fn launch_daemon_in_process(runtime_dir: &Path) -> DaemonHandle {
    let dir = runtime_dir.to_path_buf();
    tokio::spawn(async move {
        let (state, listener) =
            match monocle_runtime::lifecycle::daemon_start_sequence(&dir).await {
                Ok(pair) => pair,
                Err(e) => {
                    tracing::warn!(error = %e, "in-process daemon_start_sequence failed");
                    return;
                }
            };

        tracing::info!("in-process daemon: start sequence complete; serving HTTP");

        // Run HTTP server — blocks until shutdown signal fires.
        // The UDS accept loop is already spawned inside daemon_start_sequence.
        if let Err(e) =
            monocle_runtime::server::run_server(std::sync::Arc::clone(&state), listener).await
        {
            tracing::warn!(error = %e, "in-process daemon: HTTP server returned error");
        }

        // Shutdown tail — mirror main()'s cleanup (step 7).
        // Note: in-process daemon does not call exit_with; it returns from the task and lets
        // the tokio runtime drop DaemonState. The lock release and hooks removal are still
        // required so that no stale files are left on disk when the TUI process exits.
        if let Some(transport) = state.uds_transport.as_ref() {
            transport.cleanup();
        }
        let lock_opt = state
            .daemon_lock
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(lock) = lock_opt {
            if let Err(e) = lock.release() {
                tracing::warn!(error = %e, "in-process daemon: lock release failed; continuing");
            }
        }
        if let Err(e) = monocle_runtime::lifecycle::remove_hooks_settings(&dir) {
            tracing::warn!(error = %e, "in-process daemon: hooks-settings removal failed; continuing");
        }

        tracing::info!("in-process daemon: shutdown complete");
    });
    DaemonHandle::Task(())
}
```

**The 10s drain timeout:** the in-process daemon does NOT wrap `run_server` with
`tokio::time::timeout`. Rationale: the in-process daemon runs inside the TUI's tokio
runtime. If the TUI process exits, the runtime drops and the tasks are aborted. Hanging
indefinitely inside the TUI's async runtime is not possible in the same way it is for a
standalone subprocess. If the production-grade behavior is required (in-process daemon also
drains within 10s), add the same `tokio::time::timeout` wrapper — but this is not
required by BC-2.01.004 INV-1, which references the daemon binary's behavior. The
implementer MAY add it for consistency; it is not a blocking requirement for this story.

#### Files affected by I1

| File | Change |
|------|--------|
| `crates/monocle/src/auto_start.rs` | Rewrite `launch_daemon_in_process` to retain `(state, listener)` and call `run_server`, with shutdown tail. |

---

### I2 — Strengthen AC-E2E-004 Assertion (Note to Implementer — Route Only)

#### Background

AC-E2E-004 currently asserts that `POST /hooks/pre-tool-use` returns HTTP 200. After the
C1 fix, the event bus has a live receiver so events are actually dequeued. The ring
append path (pre_tool_use.rs lines 363–389) is **independent** of the event bus: it calls
`ring.append(record)` in a sequential statement after `try_publish_event`. The ring write
does not depend on the channel being open or having a receiver — it operates directly on
`state.ring`.

**This means ring ingestion is observable even with C1's Phase-1 deferral** (fan-out task
is drain-only; no UDS delivery). The ring path always fires after a successful hook POST
regardless of event bus state.

#### Strengthened assertion for AC-E2E-004

The implementer should replace the HTTP 200 assertion with:

1. Assert HTTP 200 (existing).
2. Assert that `<runtime_dir>/monocle-events.jsonl` exists after the POST.
3. Read the file and assert at least one JSON line is present whose `hook_type` field is
   `"PreToolUse"`. Optionally assert the `session_id` matches the one sent in the request
   body.

This verifies end-to-end ingestion through the real production path:
hook POST → handler → `ring.append()` → JSONL file on disk.

No UDS subscriber assertion is needed for C1 correctness verification because full UDS
delivery is deferred to S-032.

**Routing:** this is an implementer-scope change to `daemon_e2e_serve.rs`. No spec changes
are required beyond this note.

---

### Invariants Preserved by C1/C2/I1

| Invariant | How it is preserved |
|-----------|---------------------|
| Event bus channel never Closed during normal operation | fan-out task holds the receiver; `try_send` succeeds (Full or Ok) while the receiver is alive. |
| Fan-out task exits cleanly on shutdown | `rx.recv()` returns `None` when all senders drop after DaemonState is released. No abort needed. |
| drop_counter_debounce_task exits cleanly on shutdown | `shutdown_rx.changed()` branch exits the loop within one 100ms interval of shutdown signal. |
| BC-2.01.004 INV-1: 10s drain bound | `tokio::time::timeout(10s, run_server(...))` in `main()`. On timeout: force-close, cleanup, exit. |
| Graceful drain: cleanup tail always runs | Timeout arm returns `Ok(())`, which falls through to the shutdown tail in main(). No path bypasses cleanup. |
| In-process daemon serves HTTP | `launch_daemon_in_process` retains listener and calls `run_server`; port in lock file is live. |
| Lock and hooks-settings removed on in-process shutdown | Shutdown tail in `launch_daemon_in_process` mirrors main()'s step 7. |

---

### Updated Implementer Change-List (v1.1.0)

The original §Files the Implementer Must Change table is extended:

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/lifecycle.rs` | (original) Return type change to `(DaemonState, TcpListener)`. ALSO: retain `event_rx`; add step 5b spawn block after `DaemonState` construction (C1). |
| `crates/monocle-runtime/src/event_bus.rs` | Add `shutdown_rx` parameter to `drop_counter_debounce_task`; add `select!` shutdown branch (C1). |
| `crates/monocle-runtime/src/main.rs` | (original) Full replacement. ALSO: add `tokio::time::timeout` around `run_server`; fix step-6 doc-comment (C2). |
| `crates/monocle/src/auto_start.rs` | Rewrite `launch_daemon_in_process` to call `run_server` with shutdown tail (I1). |
| `crates/monocle-runtime/tests/daemon_start_sequence.rs` | (original) Update ~20 call-sites for new return type. |
| `crates/monocle-runtime/tests/daemon_e2e_serve.rs` | (original) NEW FILE — AC-E2E-001 through AC-E2E-006. ALSO: add AC-E2E-007 drain-timeout test (C2); strengthen AC-E2E-004 with ring file assertion (I2). |

**Files the implementer must NOT change (unchanged from original §):**
- `crates/monocle-runtime/src/server.rs`
- `crates/monocle-runtime/src/state.rs`
- `crates/monocle-runtime/src/lock.rs`
- `crates/monocle-runtime/tests/graceful_shutdown.rs`

---

### §Trace

| Version | Date | Change |
|---------|------|--------|
| 1.0.0 | 2026-06-03 | Initial document — listener seam decision, main.rs wiring, test contract. |
| 1.1.0 | 2026-06-03 | Fix addendum: C1 dead event bus (spawn fan-out + debounce tasks), C2 10s drain timeout (tokio::time::timeout in main), I1 in-process fallback dead port (launch_daemon_in_process calls run_server), I2 strengthened AC-E2E-004 ring assertion. Corrected false invariant table claim for drain timeout. |
| 1.2.0 | 2026-06-03 | Fix addendum Round 2: CRITICAL-1 ring flush event-loss on shutdown (drain-before-exit JoinHandle + write-channel close), HIGH-1 MONOCLE_HOOK_DELAY_MS unbounded production affordance (cargo feature flag e2e-test-affordances), HIGH-2 dead SigtermDuringDrain/SigintDuringDrain variants (required v1 contract; anchored deferral S-DAEMON-WIRE-FIX-001 with explicit code-level contract gap marker). AC-E2E-008 drain-durability test specified. |
| 1.3.0 | 2026-06-03 | Fix addendum Round 3: H1 ring-drain close race (explicit shutdown Notify signal to flush_task_loop; eliminates Arc-refcount-dependent channel close), M3 fan-out task shutdown parity (shutdown_rx select branch in event_bus_fan_out_task), H2-doc second e2e-test-affordances affordance MONOCLE_RING_FLUSH_DELAY_MS retroactively authorized, F-DW-HIGH-001 CI E2E job invocation reaffirmed for devops routing. |

---

## Fix Addendum Round 2 (CRITICAL-1 / HIGH-1 / HIGH-2)

**Addendum version: 1.2.0 (2026-06-03). Supplements the v1.1.0 C1/C2/I1/I2 addendum.**

This addendum records the architect's decisions on three further adversarial findings
identified on `feat/daemon-wire-serve`. All decisions must be executed by the implementer
in the same story delivery as the v1.1.0 fixes, except HIGH-2 which has an anchored
deferral (see §HIGH-2 below). No finding may be left undecided or informally deferred.

---

### CRITICAL-1 — Ring Flush Event-Loss on Shutdown

#### Problem (verified in code)

`lifecycle.rs:478`: `ring.spawn_flush_task()` returns a `tokio::task::JoinHandle<()>` that
is immediately discarded (the call is a statement, not a `let` binding). The flush task runs
asynchronously; `ring.append()` enqueues records into the bounded write-queue (`write_tx`).
On shutdown, `main()` executes the cleanup tail (UDS, lock release, hooks removal) and then
calls `exit_with(reason)` which calls `std::process::exit(code)`. `process::exit` terminates
all threads immediately without running destructors. The `Arc<DaemonState>` is never dropped.
The `Arc<RingBuffer>` inside it is never dropped. The `write_tx` sender is never dropped.
Therefore, the flush task's `rx.recv()` never returns `None`. Records enqueued by hooks that
completed HTTP 200 (caller confirmed durability) but not yet written to disk by the flush
task are silently lost. BC-2.01.007 / BC-2.04.012 durability contract is violated.

The same path exists in `auto_start.rs:launch_daemon_in_process`: the tokio task drops its
local `Arc<DaemonState>` when the task exits, but only AFTER `run_server` returns — at that
point the `Arc` refcount drops from 2 (main task + DaemonState owner) to 1 (outer caller
hold), and the ring is still not explicitly drained before the task exits.

#### Mechanism for signal (verified in ring.rs)

`RingBuffer.write_tx` is `Arc<tokio::sync::mpsc::Sender<HookEventRecord>>` (ring.rs:145).
`flush_task_loop` receives a `tokio::sync::mpsc::Receiver<HookEventRecord>` (ring.rs:603).
The flush loop runs `while let Some(record) = rx.recv().await { ... }` — it exits naturally
when the Receiver sees all Senders are closed (dropped). The only way to signal the flush
task to drain-and-stop is to drop ALL clones of the Arc-wrapped sender, which closes the
channel. Concretely: take `DaemonState.ring` out of `Some` → set to `None` → drop the
`Arc<RingBuffer>` → this drops the `Arc<Sender>` inside the ring (IF this is the last Arc
to the ring) → channel closes → flush task's `rx.recv()` returns `None` → flush loop exits.

There is no explicit `shutdown` or `close` method on `RingBuffer`. The close mechanism is
channel closure via sender drop. This is the correct idiomatic Tokio pattern.

#### Decision: Store JoinHandle on DaemonState; drain in shutdown tail before exit_with

**The fix has four parts:**

**Part 1: Store the flush-task JoinHandle on DaemonState.**

Add a field to `DaemonState`:

```rust
/// JoinHandle for the ring buffer flush task (BC-2.04.012 PC-4).
///
/// Stored so the shutdown tail can await it after closing the write channel.
/// `None` until `daemon_start_sequence` spawns the task at step 4b.
/// Taken (set to None) during the shutdown drain sequence.
pub ring_flush_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
```

In `daemon_start_sequence`, change line 478 from:

```rust
ring.spawn_flush_task();
```

to:

```rust
let flush_handle = ring.spawn_flush_task();
```

Store the handle in `DaemonState` construction (before the `Arc::new` wrapping):

```rust
ring_flush_handle: Mutex::new(Some(flush_handle)),
```

Initialize `DaemonState::new()` (unit-test constructor) with:

```rust
ring_flush_handle: Mutex::new(None),
```

**Part 2: Signal the flush task to drain by dropping the ring Arc before exit_with.**

In `main()` shutdown tail, BEFORE calling `exit_with`, add a drain step. Insert AFTER hooks
removal (step (c)) and BEFORE the exit-code determination (step 8):

```rust
// (d) Ring flush drain — close the write channel and await the flush task.
//
// BC-2.01.007 / BC-2.04.012: records enqueued by hooks that returned HTTP 200
// must not be lost on shutdown. We close the write channel by dropping the ring
// Arc from DaemonState, which closes write_tx when no other Arc<RingBuffer> holders
// exist (only DaemonState holds one). The flush task's rx.recv() then returns None,
// draining all pending records and exiting. We await the JoinHandle with a 2-second
// bounded timeout so a stuck disk cannot delay exit indefinitely.
//
// Ordering: this step runs AFTER hooks-settings.json is removed (step c) so no new
// hook events arrive from Claude Code after we begin draining. The write channel
// close is thus a clean quiesce point: all in-flight hooks have already returned
// (run_server has completed its drain), and no new hooks will be enqueued.

const RING_FLUSH_DRAIN_TIMEOUT_SECS: u64 = 2;

// Step (d-1): Drop the ring Arc, closing write_tx.
// The Mutex<Option<_>> take ensures no other path holds a reference after this point.
// After this take, DaemonState.ring is None; any concurrent code that calls
// ring.append() on a None ring logs WARN and returns (AC-005 best-effort policy).
{
    let _ = state.ring.as_ref().map(|_| {
        // We cannot take from Option<Arc<RingBuffer>> via .take() because DaemonState
        // is Arc-wrapped and we have a shared reference. Instead, we drop the ring via
        // replacing the field. Since DaemonState fields are not pub(mutable), the
        // correct approach is:
    });
    // Correct mechanism: the ring field is pub on DaemonState; we can write through
    // the Arc by taking the field value via the mutex-wrapped container.
    // DaemonState.ring is Option<Arc<RingBuffer>> without a Mutex wrapper.
    // To drop the Arc, we need mutable access or use unsafe. The production-grade
    // approach: add a method to DaemonState:
    //   pub fn take_ring(&self) -> Option<Arc<RingBuffer>> { ... }
    // This cannot be done without interior mutability (the ring field is not Mutex-wrapped).
    //
    // ARCHITECT DECISION: wrap DaemonState.ring in a Mutex<Option<Arc<RingBuffer>>> so it
    // can be taken (closed) from the shared Arc<DaemonState>. See §CRITICAL-1 Field Change.
}
```

**Part 2 requires a field type change — see below.**

**Part 3: Field change — wrap DaemonState.ring in Mutex.**

`DaemonState.ring` is currently `Option<Arc<RingBuffer>>` (state.rs:141). To allow the
shutdown tail (which only has `Arc<DaemonState>`) to take the ring and close the write
channel, this field must be wrapped in `Mutex<Option<Arc<RingBuffer>>>`:

```rust
// Old:
pub ring: Option<Arc<RingBuffer>>,

// New:
pub ring: Mutex<Option<Arc<RingBuffer>>>,
```

All existing callers that read `state.ring` must be updated to call
`state.ring.lock().unwrap_or_else(|e| e.into_inner())`. In hook handlers, this is a
shared-read (locks are short-lived): the pattern is:

```rust
// Old:
if let Some(ring) = state.ring.as_ref() { ring.append(...); }

// New:
if let Some(ring) = state.ring.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
    ring.append(...);
}
```

The `DaemonState::new()` constructor initializes with:

```rust
ring: Mutex::new(None),
```

The `daemon_start_sequence` wires with:

```rust
ring: Mutex::new(Some(ring)),
```

**Part 4: Shutdown tail drain — updated step (d) with correct mechanism.**

After adding the Mutex wrapper, the shutdown tail in `main()` AFTER hooks removal:

```rust
// (d) Ring flush drain (BC-2.01.007 / BC-2.04.012 durability contract).
// Drop the ring Arc to close the write channel, then await flush task with 2s timeout.
{
    // Step (d-1): Take the ring Arc out of DaemonState (sets ring to None, drops Arc).
    let ring_arc = state.ring.lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    drop(ring_arc); // Explicitly drop to close write_tx (if this was the last Arc).

    // Step (d-2): Take the flush JoinHandle.
    let flush_handle = state.ring_flush_handle.lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();

    // Step (d-3): Await with bounded timeout.
    if let Some(handle) = flush_handle {
        match tokio::time::timeout(
            std::time::Duration::from_secs(RING_FLUSH_DRAIN_TIMEOUT_SECS),
            handle,
        )
        .await
        {
            Ok(_) => {
                tracing::info!(
                    "ring flush task drained and exited cleanly (BC-2.04.012 durability)"
                );
            }
            Err(_elapsed) => {
                tracing::warn!(
                    timeout_secs = RING_FLUSH_DRAIN_TIMEOUT_SECS,
                    "ring flush drain timeout — some pending records may be lost \
                     (disk I/O too slow); proceeding with exit (BC-2.04.012 best-effort)"
                );
            }
        }
    }
}
```

**Same drain steps apply to `auto_start.rs:launch_daemon_in_process`** shutdown tail, using
the same field access pattern. The in-process path does not call `exit_with` (returns from
the task instead), so runtime drop would eventually close the ring — but "eventually" is
non-deterministic under tokio's task scheduling. Apply the explicit drain for determinism.

**Ordering relative to lock release and hooks-settings removal:**

The drain MUST occur AFTER hooks-settings.json removal (step c) so no new hook events arrive
during the drain window. This is the clean quiesce sequence:
1. (a) UDS socket cleanup
2. (b) Lock file release
3. (c) Remove hooks-settings.json — Claude Code stops posting hooks immediately
4. **(d) Ring flush drain — close write channel, await flush task with 2s timeout**
5. Determine exit code
6. `exit_with(reason)`

#### New E2E Test: AC-E2E-008 — Ring Durability Under Immediate SIGTERM

Add to `crates/monocle-runtime/tests/daemon_e2e_serve.rs`:

```
AC-E2E-008 — Post hook then SIGTERM immediately; record persists to disk:
1. Spawn daemon binary (MONOCLE_RUNTIME_DIR=<tmpdir>). Wait for lock file.
2. POST a valid PreToolUse hook. Do NOT wait for the HTTP response before sending SIGTERM.
   Use a background thread: spawn the request, then immediately from the main test thread
   send SIGTERM to the daemon process (no sleep between POST and SIGTERM).
3. Wait for the daemon process to exit (max 15 seconds: 10s drain + 2s ring flush + 3s buffer).
4. Assert exit code 0 (DaemonExit::Graceful).
5. Assert <tmpdir>/monocle-events.jsonl exists.
6. Read the file and assert at least one JSONL line with hook_type == "PreToolUse".
   This verifies that the hook record enqueued before SIGTERM was written to disk by the
   flush task before process::exit fired (BC-2.01.007 / BC-2.04.012 durability contract).
```

**Note on feasibility:** The test is racy by design — the hook POST and SIGTERM are issued
in close succession. The daemon must handle: (1) complete the HTTP response to the hook
within 300ms, (2) enqueue the ring record, (3) receive SIGTERM, (4) remove hooks-settings,
(5) drain the ring flush task (up to 2s), (6) exit 0. The 15-second wait bound gives
sufficient margin. The JSONL assertion closes the verification gap that the I2 fix opened:
I2 proved the ring path fires; AC-E2E-008 proves it survives shutdown.

#### BC Traceability

- BC-2.01.007: "hook events are persisted to disk before the HTTP response is sent" —
  the ring drain ensures records enqueued during the handler lifetime reach disk.
- BC-2.04.012 PC-4: async flush path — durability requires flush task completion before exit.

#### Files Affected by CRITICAL-1

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/state.rs` | Change `ring` field to `Mutex<Option<Arc<RingBuffer>>>`. Add `ring_flush_handle: Mutex<Option<tokio::task::JoinHandle<()>>>` field. Update `DaemonState::new()` constructor. |
| `crates/monocle-runtime/src/lifecycle.rs` | Store `flush_handle` from `ring.spawn_flush_task()`. Wire `ring_flush_handle: Mutex::new(Some(flush_handle))` and `ring: Mutex::new(Some(ring))` in DaemonState construction. |
| `crates/monocle-runtime/src/main.rs` | Add ring drain step (d) in shutdown tail between hooks removal and exit_with. |
| `crates/monocle/src/auto_start.rs` | Add same ring drain step (d) in `launch_daemon_in_process` shutdown tail. |
| `crates/monocle-runtime/src/hooks/pre_tool_use.rs` | Update `state.ring.as_ref()` → lock pattern. |
| `crates/monocle-runtime/src/hooks/notification.rs` | Same ring lock update. |
| `crates/monocle-runtime/src/hooks/stop_session_prompt.rs` | Same ring lock update. |
| `crates/monocle-runtime/tests/daemon_e2e_serve.rs` | Add AC-E2E-008 drain-durability test. |

**Note for implementer:** check `grep -rn "state.ring\|\.ring\." crates/monocle-runtime/src/`
for all ring access sites before implementing, to ensure all callers of the `ring` field are
updated to the Mutex lock pattern. Do not miss any access site.

---

### HIGH-1 — MONOCLE_HOOK_DELAY_MS Unbounded Production Affordance

#### Problem (verified in code)

`lifecycle.rs:617-619` reads `MONOCLE_HOOK_DELAY_MS` from the environment and writes it to
`DaemonState.hook_outer_delay_ms` unconditionally in the production `daemon_start_sequence`
path (no `#[cfg(test)]` gate, no cargo feature gate). `pre_tool_use.rs:75-77` sleeps for
this duration at the outer handler level, BEFORE the 300ms inner timeout budget. There is no
upper bound on the value: `MONOCLE_HOOK_DELAY_MS=99999999` would sleep forever, creating an
unbounded local denial-of-service condition and violating BC-2.04.007 PC-4 / INV-1
(300ms absolute timeout). This is test infrastructure compiled and active in every production
binary. Any operator who accidentally sets this env var (e.g., from a test environment's
exported env) gets a broken daemon with no diagnostic. CLAUDE.md explicitly forbids test
logic compiled into the production hook path.

#### Decision: Option A — Cargo Feature Flag `e2e-test-affordances`

Three options were evaluated:

**Option A:** Gate the env var read and sleep behind a cargo feature flag
(`e2e-test-affordances`) so production binaries compiled without the flag never contain the
delay code path.

**Option B:** Keep the env var but clamp to a small maximum (e.g., 5 seconds), emit a WARN
at startup when set, and document as a bounded diagnostic affordance.

**Option C:** Remove the delay affordance from production entirely; hold AC-E2E-007's
in-flight request open via a slow HTTP client in the test (e.g., send body in small chunks
with inter-chunk delays using a raw TCP stream or a custom reqwest client with throttled
body transmission).

**Decision: Option A.** Rationale:

- **Zero production binary pollution is required** per CLAUDE.md ("test logic compiled into
  the production hook path is forbidden"). Option B leaves the delay path compiled in; any
  environment that exports the variable gets a degraded daemon. Option B is incompatible with
  the production-grade principle.
- **Option C is feasible** but requires the test to synthesize a genuinely slow HTTP request
  at the transport layer (e.g., connect via raw `std::net::TcpStream`, write the HTTP headers,
  then deliberately pause before writing the body — axum reads body lazily, so the handler
  stays in-flight while the client holds the connection open without sending the body). This
  works correctly but is substantially more complex test code, harder to maintain, and requires
  deep knowledge of axum's connection handling. Option A is cleaner.
- **Option A compiler-enforces the boundary**: production `cargo build` never compiles the
  delay path. The feature flag is explicit and searchable. CI can enforce `--no-default-features`
  for the production artifact check.

#### Exact Changes for Option A

**Step 1: Define the feature in `monocle-runtime/Cargo.toml`:**

```toml
[features]
# E2E test affordances — test-only delay injection for drain-timeout tests.
# MUST NOT be enabled in production builds. Only enabled by the E2E test binary target
# via the test target's `required-features` or the `cargo test` invocation in CI.
e2e-test-affordances = []
```

**Step 2: Gate the env var read in `lifecycle.rs`.**

Current (lifecycle.rs:611-619):

```rust
hook_delay_ms: None, // Unit-test override only; not set via env var.
// MONOCLE_HOOK_DELAY_MS: test-only env var for E2E drain-timeout testing (C2 fix —
// SS-daemon-wiring-impl.md §Fix Addendum C2). When set, the hook outer handler (before
// the 300ms inner timeout budget) sleeps for this many milliseconds, creating a
// genuinely in-flight HTTP request that holds axum's graceful-shutdown drain open.
// Not set in production deployments. Absent env var → None → no delay.
hook_outer_delay_ms: std::env::var("MONOCLE_HOOK_DELAY_MS")
    .ok()
    .and_then(|v| v.parse::<u64>().ok()),
```

New:

```rust
hook_delay_ms: None, // Unit-test override only; not set via env var.
#[cfg(feature = "e2e-test-affordances")]
hook_outer_delay_ms: std::env::var("MONOCLE_HOOK_DELAY_MS")
    .ok()
    .and_then(|v| v.parse::<u64>().ok()),
#[cfg(not(feature = "e2e-test-affordances"))]
hook_outer_delay_ms: None, // e2e-test-affordances feature not enabled; delay disabled.
```

**Step 3: Gate the sleep in `pre_tool_use.rs`.**

Current (pre_tool_use.rs:75-77):

```rust
if let Some(delay_ms) = state.hook_outer_delay_ms {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}
```

New:

```rust
#[cfg(feature = "e2e-test-affordances")]
if let Some(delay_ms) = state.hook_outer_delay_ms {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}
```

**Step 4: Update the E2E test to build the daemon binary with the feature.**

In `daemon_e2e_serve.rs`, the test that uses `MONOCLE_HOOK_DELAY_MS` (AC-E2E-007) must
ensure the daemon binary is built with `e2e-test-affordances`. The test helper that spawns
the binary must be annotated and the binary must be built with:

```
cargo test --test daemon_e2e_serve --features e2e-test-affordances
```

OR the test file uses a build script / `#[cfg(feature = "e2e-test-affordances")]` guard:

```rust
#[test]
#[cfg(feature = "e2e-test-affordances")]
fn test_ac_e2e_007_drain_timeout() {
    // Test body — uses MONOCLE_HOOK_DELAY_MS=15000.
    // Only compiled when e2e-test-affordances feature is active.
    // Run with: cargo test --test daemon_e2e_serve --features e2e-test-affordances
}
```

The CI drain-test job must explicitly pass `--features e2e-test-affordances` when running
AC-E2E-007. The standard `cargo test --workspace` run (without the feature) skips AC-E2E-007
(compile-time exclusion). This is correct: CI has a dedicated E2E job for drain tests.

**Step 5: Update doc-comments.**

In `state.rs`, the `hook_outer_delay_ms` field doc comment must be updated to note the
feature gate:

```rust
/// Artificial delay override for E2E tests (outer handler, before 300ms timeout budget).
///
/// `None` — no delay (production default; always None when `e2e-test-affordances` feature
///   is not enabled).
/// `Some(ms)` — sleep `ms` milliseconds at the outer handler level, BEFORE the 300ms inner
///   timeout budget starts. Used by AC-E2E-007 to create a genuinely in-flight HTTP request
///   that holds axum's graceful-shutdown drain open. Only compiled when the
///   `e2e-test-affordances` cargo feature is active. Never present in production binaries.
pub hook_outer_delay_ms: Option<u64>,
```

#### AC-E2E-007 Mechanism After Fix

AC-E2E-007 continues to use `MONOCLE_HOOK_DELAY_MS=15000`. The daemon binary spawned by
the test must be compiled with `--features e2e-test-affordances`. The test runner invokes:

```
cargo test --test daemon_e2e_serve --features e2e-test-affordances -- test_ac_e2e_007
```

The binary path (`CARGO_BIN_EXE_monocle-runtime`) resolves to a feature-enabled binary when
cargo runs integration tests with `--features`. No changes to AC-E2E-007's test logic are
required beyond the `#[cfg(feature = "e2e-test-affordances")]` annotation on the test
function itself.

#### Files Affected by HIGH-1

| File | Change |
|------|--------|
| `crates/monocle-runtime/Cargo.toml` | Add `[features]` section with `e2e-test-affordances = []`. |
| `crates/monocle-runtime/src/lifecycle.rs` | Gate env var read with `#[cfg(feature = "e2e-test-affordances")]` / `#[cfg(not(...))]`. |
| `crates/monocle-runtime/src/hooks/pre_tool_use.rs` | Gate sleep with `#[cfg(feature = "e2e-test-affordances")]`. |
| `crates/monocle-runtime/src/state.rs` | Update `hook_outer_delay_ms` field doc comment to note feature gate. |
| `crates/monocle-runtime/tests/daemon_e2e_serve.rs` | Annotate AC-E2E-007 with `#[cfg(feature = "e2e-test-affordances")]`. |

---

### HIGH-2 — Dead SigtermDuringDrain/SigintDuringDrain Exit Variants

#### Problem (verified in code)

`DaemonExit::SigtermDuringDrain` (exit 143, POSIX 128+15) and
`DaemonExit::SigintDuringDrain` (exit 130, POSIX 128+2) are defined in `lifecycle.rs:191-200`
and documented as the BC-2.01.004 INV-4 monitoring contract ("External monitoring systems
MUST use exit code 143 to detect SIGTERM hard-kill during drain"). However, `main()` step 8
(lines 234-238) only yields `Graceful(0)` or `AdminForceStop(2)`. No second-signal handler
exists. A second SIGTERM during the axum drain window hits the OS default (process killed,
exit code undefined by daemon — OS kills it directly). The monitoring contract is live in the
code as a doc-comment claim with no implementation behind it.

#### Adjudication: Required for v1 — Deferred to S-DAEMON-WIRE-FIX-001

**Assessment of v1 requirement:**

Under the production-grade principle, a live monitoring contract claim with no implementation
is a defect, not a deferral. The variants are defined, documented, and the `to_exit_code`
table is complete — the external interface (exit 143/130) is published. Leaving it
unimplemented means any monitoring system that consults the documentation and sets up alerting
on exit 143 will never receive that signal; they'll instead see SIGKILL from the OS (which
produces no consistent exit code in k8s / systemd restart tracking).

**However:** implementing second-signal detection requires a non-trivial change to the signal
handling architecture. `run_server` currently installs a one-shot SIGTERM/SIGINT handler that
fires the `shutdown_tx` watch channel. After that fires, axum begins draining. A second
SIGTERM during the drain requires a SECOND independent signal handler that can communicate
with `main()` after `run_server` has already returned its first signal. The Tokio signal API
(`tokio::signal::unix::signal`) allows creating a persistent signal stream (multiple `.recv()`
calls), but `run_server` takes ownership of the signal futures. Structuring a second-signal
watcher correctly requires restructuring the shutdown signal architecture in `run_server` and
`main()`.

This is **in scope** for v1 correctness but is **not trivial enough to safely add within the
current daemon-wire-serve story** without risk of introducing a signal-handler bug. The
correct action per the production-grade principle: anchor it to a real, immediately-scheduled
follow-up story, and mark the unimplemented contract gap explicitly in the code so it is not
a silent false claim.

**Deferral anchor: S-DAEMON-WIRE-FIX-001**

The implementer must create story `S-DAEMON-WIRE-FIX-001` in the STORY-INDEX with:
- Title: "Second-signal detection during drain (SigtermDuringDrain/SigintDuringDrain exit codes)"
- Epic: EPIC-04 (daemon lifecycle correctness)
- Wave: next available (Wave 8 or Phase 4 convergence-fix batch)
- Scope: implement second-signal handler in `run_server` + `main()` so that a second SIGTERM
  during the axum drain window produces `DaemonExit::SigtermDuringDrain` (exit 143) and a
  second Ctrl-C produces `DaemonExit::SigintDuringDrain` (exit 130) per BC-2.01.004 INV-4.
- BC reference: BC-2.01.004 INV-4 (monitoring contract)
- Points: 3

**Required code-level contract gap marker (implementer must add this):**

In `lifecycle.rs`, add a comment immediately above `SigtermDuringDrain` and
`SigintDuringDrain` in `DaemonExit`:

```rust
/// A second SIGTERM (signal 15) was received while a drain was in progress.
/// POSIX convention 128+15. Exit code `143`.
///
/// # IMPLEMENTATION STATUS
///
/// CONTRACT GAP (S-DAEMON-WIRE-FIX-001): this variant is defined and documented as the
/// BC-2.01.004 INV-4 monitoring contract but is not yet produced by any code path.
/// `main()` currently yields only `Graceful(0)` or `AdminForceStop(2)`. A second SIGTERM
/// during the drain window hits the OS default (process killed without this exit code).
/// Second-signal detection is anchored to story S-DAEMON-WIRE-FIX-001.
SigtermDuringDrain,
```

Add the same marker above `SigintDuringDrain`.

**Why a code-level marker satisfies the production-grade principle here:**

- The deferral target is a real story ID (S-DAEMON-WIRE-FIX-001), not "Phase X" or "later."
- The contract gap is explicitly documented in the source code at the contract definition site.
- No monitoring system that reads the source code will be misled — the gap is clearly marked.
- The story ID prevents the item from being lost across sprint cycles.
- CLAUDE.md Principle 3 requires a future story anchor AND an explicit human direction for
  tech-debt-register entries. This marker is a CODE-level anchor, not a tech-debt-register
  entry — it requires no human approval because the implementer is anchoring a finding they
  discovered, per production-grade Principle 4 (fix-or-anchor, not silently defer).

#### Second-Signal Detection Seam (for S-DAEMON-WIRE-FIX-001 implementer reference)

The correct implementation when S-DAEMON-WIRE-FIX-001 is executed:

1. In `run_server`: change the `signal` stream creation from a one-shot future to a
   persistent `tokio::signal::unix::Signal` that remains alive. After the first recv,
   instead of returning from the signal future, loop to detect a second recv. OR: separate
   the first and second signal futures entirely — the first fires the shutdown_tx watch
   channel; the second sets a `second_sigterm: Arc<AtomicBool>` flag on `DaemonState`.

2. In `main()`: after `tokio::time::timeout(10s, run_server(...))` returns, check
   `state.second_sigterm.load(SeqCst)` (and `second_sigint`). If set, the exit variant is
   `SigtermDuringDrain` or `SigintDuringDrain` respectively, taking precedence over
   `Graceful` and `AdminForceStop`.

3. The `tokio::select!` structure in `run_server` would become a two-phase loop:
   Phase 1 (before first shutdown signal): select on {watch_rx, SIGTERM, SIGINT}.
   Phase 2 (during drain): install a second SIGTERM listener that writes to
   `DaemonState.second_sigterm`.

This seam information is stored here so the S-DAEMON-WIRE-FIX-001 implementer has the
architectural context without needing to re-investigate from scratch.

#### Files Affected by HIGH-2

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/lifecycle.rs` | Add CONTRACT GAP markers above `SigtermDuringDrain` and `SigintDuringDrain` variants. |
| STORY-INDEX | Add S-DAEMON-WIRE-FIX-001 entry (story-writer scope). |

**Note:** STORY-INDEX is story-writer scope per the routing table. The implementer surfaces
the story creation requirement; the story-writer agent (or human) creates the STORY-INDEX
entry before the wave-8-gate. The contract gap marker in the code is the implementer's
deliverable for this story.

---

### Updated Invariants Table (Round 2)

| Invariant | How it is preserved |
|-----------|---------------------|
| BC-2.01.007 / BC-2.04.012: ring records persisted before process exit | Shutdown tail drops ring Arc (closes write_tx), awaits flush JoinHandle with 2s timeout before exit_with. |
| Flush task drain is bounded | 2-second tokio::time::timeout on flush JoinHandle await; WARN logged on timeout; exit proceeds regardless. |
| Flush drain quiesce ordering | Ring drain step (d) runs AFTER hooks-settings.json removal (step c); no new hook events arrive during drain. |
| MONOCLE_HOOK_DELAY_MS never present in production binary | Env var read and sleep gated by `#[cfg(feature = "e2e-test-affordances")]`; not compiled without the feature. |
| BC-2.04.007 INV-1: 300ms absolute timeout not violated by outer delay | Outer delay code is compiler-excluded in production builds; `hook_outer_delay_ms` is always None. |
| BC-2.01.004 INV-4 monitoring contract gap explicitly flagged | CONTRACT GAP markers added to SigtermDuringDrain and SigintDuringDrain; anchored to S-DAEMON-WIRE-FIX-001. |
| Second-signal detection gap is anchored | S-DAEMON-WIRE-FIX-001 created in STORY-INDEX with BC-2.01.004 INV-4 reference. |

---

### Updated Implementer Change-List (v1.2.0)

The v1.1.0 implementer change-list is extended with Round 2 additions:

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/state.rs` | **[CRITICAL-1]** Change `ring` to `Mutex<Option<Arc<RingBuffer>>>`. Add `ring_flush_handle: Mutex<Option<tokio::task::JoinHandle<()>>>`. Update `DaemonState::new()` and `Default` impl. |
| `crates/monocle-runtime/src/lifecycle.rs` | **[CRITICAL-1]** Store `flush_handle` from `ring.spawn_flush_task()`; wire into DaemonState. **[HIGH-2]** Add CONTRACT GAP markers to SigtermDuringDrain and SigintDuringDrain. |
| `crates/monocle-runtime/src/main.rs` | **[CRITICAL-1]** Add ring drain step (d) in shutdown tail (after hooks removal, before exit_with). |
| `crates/monocle/src/auto_start.rs` | **[CRITICAL-1]** Add ring drain step (d) in `launch_daemon_in_process` shutdown tail. |
| `crates/monocle-runtime/src/hooks/pre_tool_use.rs` | **[CRITICAL-1]** Update ring access to Mutex lock pattern. **[HIGH-1]** Gate outer delay sleep with `#[cfg(feature = "e2e-test-affordances")]`. |
| `crates/monocle-runtime/src/hooks/notification.rs` | **[CRITICAL-1]** Update ring access to Mutex lock pattern. |
| `crates/monocle-runtime/src/hooks/stop_session_prompt.rs` | **[CRITICAL-1]** Update ring access to Mutex lock pattern. |
| `crates/monocle-runtime/Cargo.toml` | **[HIGH-1]** Add `[features]` with `e2e-test-affordances = []`. |
| `crates/monocle-runtime/src/lifecycle.rs` | **[HIGH-1]** Gate `MONOCLE_HOOK_DELAY_MS` env var read with `#[cfg(feature)]` / `#[cfg(not(feature))]`. |
| `crates/monocle-runtime/tests/daemon_e2e_serve.rs` | **[CRITICAL-1]** Add AC-E2E-008 ring-durability test. **[HIGH-1]** Annotate AC-E2E-007 with `#[cfg(feature = "e2e-test-affordances")]`. |
| STORY-INDEX | **[HIGH-2]** Add S-DAEMON-WIRE-FIX-001 (story-writer routes; implementer surfaces the story creation requirement). |

**Files unchanged (confirm before submitting PR):**
- `crates/monocle-runtime/src/server.rs` — no changes required in this story.
- `crates/monocle-runtime/src/ring.rs` — `spawn_flush_task` signature and behavior unchanged; only the JoinHandle return value must now be retained by the caller.
- `crates/monocle-runtime/tests/graceful_shutdown.rs` — unaffected (does not use ring drain path).

---

### Registry Atomicity Note (for state-manager)

This addendum bumps `SS-daemon-wiring-impl` from version `1.1.0` to `1.2.0`. The
`version-pin-registry.yaml` entry for `SS-daemon-wiring-impl` must be updated in the same
factory-artifacts commit as this spec file (CLAUDE.md REGISTRY ATOMICITY rule, CI POL-11).

Required update in `.factory/specs/version-pin-registry.yaml`:

```yaml
SS-daemon-wiring-impl:
  path: specs/architecture/SS-daemon-wiring-impl.md
  current_version: "1.3.0"
  last_bump_commit: "[DAEMON-WIRE adversarial fix addendum Round 3 H1/M3/H2-doc/F-DW-HIGH-001]"
  last_bump_date: "2026-06-03"
```

---

## Fix Addendum Round 3 (H1 / M3 / H2-doc / F-DW-HIGH-001)

**Addendum version: 1.3.0 (2026-06-03). Supplements the v1.2.0 Round 2 addendum.**

This addendum records architect decisions on three adversarial findings plus one CI routing
finding identified on `feat/daemon-wire-serve` after the Round 2 fixes were designed.
H1 and M3 are SPEC changes that extend implementer obligations. H2-doc retroactively
authorizes an implementer affordance added during F-1. F-DW-HIGH-001 is a devops routing
note (no spec design required; reaffirms an existing CI obligation).

---

### H1 — Ring-Drain Close Race (Deterministic Channel Close)

#### Problem

The Round 2 CRITICAL-1 fix stores the ring in `Mutex<Option<Arc<RingBuffer>>>` and the
shutdown tail does:

```rust
let ring_arc = state.ring.lock().unwrap_or_else(|e| e.into_inner()).take();
drop(ring_arc); // close write_tx when last Arc holder
```

The spec comment at this site claims "DaemonState holds the only Arc" — this is the
mechanism by which `drop(ring_arc)` closes the `write_tx` channel and causes the flush
task's `rx.recv()` to return `None`.

That assumption is fragile on the FORCE-CLOSE path. When `tokio::time::timeout(10s)`
elapses and the serve future is dropped, in-flight axum handler tasks are NOT aborted
(tokio `spawn` tasks run independently of the future that spawned them). Those tasks hold
`Arc<DaemonState>`. Inside the handler, `ring.append()` calls `self.write_tx.try_send()`
where `write_tx` is `Arc<Sender<HookEventRecord>>` stored inside `RingBuffer`. The
`Arc<RingBuffer>` is inside `DaemonState.ring`. The Mutex lock pattern ensures handlers
access `ring` sequentially, not via a clone. However, `write_tx` is `Arc<Sender>` and
`RingBuffer.write_tx` is a field of type `Arc<Sender>`. If any caller does
`Arc::clone(&ring.write_tx)` (or if `append()` itself produces a clone — it calls
`self.write_tx.try_send()` which only borrows, not clones), there is no second strong
Arc to the Sender.

The actual race is subtler: if a handler task is executing inside `ring.lock()` when the
shutdown tail calls `ring.lock().take()`, the tail blocks (correct — mutex serializes).
The tail proceeds only after the handler drops the guard. This is safe. But:

- On the FORCE-CLOSE path, the serve future is dropped at 10s. Handler tasks can still
  be running (their `JoinHandle`s were never awaited). The shutdown tail runs
  concurrently with those tasks.
- The tail's `drop(ring_arc)` reduces the `Arc<RingBuffer>` refcount by one (the
  DaemonState holder). If a handler task holds `Arc<DaemonState>` (always true — it's
  cloned into the handler by axum), the `DaemonState.ring` Mutex now contains `None`
  after the take. The `Arc<RingBuffer>` refcount is 1 (the `ring_arc` local in the
  shutdown tail). `drop(ring_arc)` brings it to 0 — `write_tx` drops — channel closes.
  The flush task exits after draining its queue.
- The tail then immediately awaits `ring_flush_handle` with a 2s timeout.

This sequence IS deterministic IF no other code holds a separate `Arc<RingBuffer>` clone.
The current handler code `if let Some(ring) = state.ring.lock()..as_ref()` borrows through
the guard — no clone. So the race does not exist in the current code. However:

1. The spec comment "DaemonState holds the only Arc" is still factually wrong: there is
   no enforcement preventing a future code change from cloning the Arc. It is an implicit,
   unenforced invariant.
2. The flush task's exit depends entirely on Arc-refcount-reaching-zero → `write_tx`
   dropping → channel closing. If a future caller adds `let ring_clone = Arc::clone(&ring)`,
   the shutdown silently hangs at the 2s timeout instead of draining cleanly. This is a
   latent reliability defect.
3. The spec's false comment (main.rs:~230-232, `"DaemonState holds the only Arc"`) must
   be corrected regardless.

#### Decision: Option A — Explicit Shutdown Signal to flush_task_loop

**Add an explicit `tokio::sync::Notify` (or oneshot) shutdown signal to the flush task
so it exits deterministically on shutdown, independent of whether Arc refcount reaches zero.**

Three options were considered:

**Option A:** Pass a dedicated `Arc<tokio::sync::Notify>` into `spawn_flush_task` /
`flush_task_loop`. The shutdown tail calls `notify.notify_one()`. The flush loop uses
`tokio::select!` to race `rx.recv()` against `notify.notified()`. On the notify branch:
drain all remaining queued records via `rx.try_recv()` loop, then break.

**Option B:** Reuse `DaemonState.shutdown_rx` (the existing `watch::Receiver<bool>`)
as the signal. Clone it into `spawn_flush_task`. The flush loop selects on it.

**Option C:** Retain the Arc-drop mechanism (CRITICAL-1 as written) and only correct
the false comment.

**Decision: Option A.** Rationale:

- **Zero dependency on Arc refcount.** The flush task exits via an explicit signal, not
  via "no one else holds the ring Arc." Future callers that add `Arc::clone` do not
  silently break shutdown.
- **Option B is viable** but couples the flush task to `DaemonState` (it already
  receives `write_rx: Receiver`, not `DaemonState`). `flush_task_loop` is a pure-ish
  function that receives only the channel and config — threading a watch receiver into
  it is lower friction than a DaemonState dependency. However, `shutdown_rx` is
  already on `DaemonState`; a dedicated `Notify` makes the shutdown contract explicit
  and avoids the ambiguity of "what does watch value `true` mean to the flush task."
  Option A is cleaner.
- **Option C is insufficient** per the production-grade principle: a false comment that
  encodes a fragile invariant must be fixed, not just annotated. The invariant must be
  made structurally true, not documented as a hope.
- **Drain-before-exit is preserved:** the flush loop on the signal branch drains all
  currently-queued records via `rx.try_recv()` before breaking. This ensures records
  enqueued before the signal (by the last in-flight hooks) are written. AC-E2E-008
  (ring durability under immediate SIGTERM) continues to be satisfied.

#### Exact Changes for Option A

##### Step 1: Add `RingShutdown` type to `ring.rs`

```rust
/// Shutdown signal for the flush task (H1 fix — SS-daemon-wiring-impl.md §Round 3 H1).
///
/// The shutdown tail calls `RingShutdown::signal()` to trigger deterministic drain-and-exit
/// of the flush task, independent of `Arc<RingBuffer>` refcount.
///
/// Cloned into `spawn_flush_task` via `Arc::clone`. The flush task holds the `Arc` and
/// polls `notified()` inside the select! loop. The tail holds the other `Arc` and calls
/// `notify_one()`.
pub type RingShutdownNotify = std::sync::Arc<tokio::sync::Notify>;
```

No new struct is needed — `Arc<Notify>` is sufficient.

##### Step 2: Change `spawn_flush_task` signature

```rust
// Old:
pub fn spawn_flush_task(&self) -> tokio::task::JoinHandle<()>

// New:
pub fn spawn_flush_task(
    &self,
    ring_shutdown: RingShutdownNotify,
) -> tokio::task::JoinHandle<()>
```

Inside `spawn_flush_task`, clone `ring_shutdown` and pass it to `flush_task_loop`:

```rust
pub fn spawn_flush_task(&self, ring_shutdown: RingShutdownNotify) -> tokio::task::JoinHandle<()> {
    let rx = self
        .write_rx
        .lock()
        .expect("write_rx mutex poisoned")
        .take()
        .expect("spawn_flush_task() called twice");

    let path = self.path.clone();
    let config = self.config.clone();
    let byte_count = Arc::clone(&self.byte_count);
    let disk_error = Arc::clone(&self.disk_error);

    tokio::spawn(async move {
        flush_task_loop(rx, path, config, byte_count, disk_error, ring_shutdown).await;
    })
}
```

##### Step 3: Restructure `flush_task_loop` with `select!`

```rust
async fn flush_task_loop(
    mut rx: tokio::sync::mpsc::Receiver<HookEventRecord>,
    path: PathBuf,
    config: RotationConfig,
    byte_count: Arc<Mutex<u64>>,
    disk_error: Arc<Mutex<bool>>,
    ring_shutdown: RingShutdownNotify,
) {
    loop {
        tokio::select! {
            biased;  // Check shutdown signal first so it is not starved by a burst.

            // Shutdown signal: drain all remaining queued records, then exit.
            _ = ring_shutdown.notified() => {
                tracing::debug!("flush_task_loop: shutdown signal received; draining queue");
                // Drain all currently-queued records before breaking.
                // try_recv() is non-blocking; returns Err(Empty) when the queue is empty.
                loop {
                    match rx.try_recv() {
                        Ok(record) => {
                            // Write this record using the same logic as the normal path.
                            // Factor into a helper to avoid duplication.
                            flush_one_record(&record, &path, &config, &byte_count, &disk_error);
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                    }
                }
                tracing::info!("flush_task_loop: drain complete; exiting");
                return;
            }

            // Normal path: receive and write the next record.
            record = rx.recv() => {
                match record {
                    None => {
                        // All senders dropped — channel closed. Exit cleanly.
                        tracing::info!("flush_task_loop: channel closed; exiting");
                        return;
                    }
                    Some(record) => {
                        flush_one_record(&record, &path, &config, &byte_count, &disk_error);
                    }
                }
            }
        }
    }
}
```

The `flush_one_record` helper extracts the serialise-write-rotate logic from the current
loop body into a private function. This avoids duplicating the full write logic in the
drain branch. No behavioral change to the normal path.

**`biased` select justification:** Use `biased;` so that the `ring_shutdown.notified()`
branch is checked FIRST on each loop iteration. Without `biased`, tokio randomly selects
between the two ready branches. In a burst scenario where both the signal fires and a
record is ready simultaneously, `biased` ensures the shutdown path is taken deterministically
— we drain via `try_recv()` rather than processing records one-at-a-time through `recv()`.
This prevents an adversarial scenario where a rapid stream of `append()` calls keeps the
flush loop on the `recv()` branch indefinitely after the signal fires.

##### Step 4: Add `ring_flush_shutdown` field to `DaemonState`

```rust
/// Shutdown notify for the ring flush task (BC-2.04.012 / H1 fix).
///
/// The shutdown tail calls `ring_flush_shutdown.notify_one()` to signal the flush task
/// to drain and exit deterministically, independent of Arc<RingBuffer> refcount.
/// `None` if no flush task has been spawned (unit-test constructor path).
pub ring_flush_shutdown: Option<RingShutdownNotify>,
```

Initialize in `DaemonState::new()`:

```rust
ring_flush_shutdown: None,
```

Initialize in `daemon_start_sequence` DaemonState construction:

```rust
ring_flush_shutdown: Some(Arc::clone(&ring_shutdown_notify)),
```

where `ring_shutdown_notify` is created at step 4b (before `DaemonState` construction):

```rust
// Step 4b: Create ring shutdown notify for flush task.
let ring_shutdown_notify: crate::ring::RingShutdownNotify = Arc::new(tokio::sync::Notify::new());
let flush_handle = ring.spawn_flush_task(Arc::clone(&ring_shutdown_notify));
```

##### Step 5: Update shutdown tail in `main()` and `launch_daemon_in_process`

In the ring drain step (d), replace the `drop(ring_arc)` mechanism with the notify:

```rust
// (d) Ring flush drain — signal flush task to drain and exit, then await JoinHandle.
{
    // Step (d-1): Signal the flush task to drain remaining records and exit.
    // This is deterministic: the task's select! shutdown branch drains try_recv()
    // until empty, writes all pending records, then returns (H1 fix — eliminates
    // Arc-refcount-dependent channel close from CRITICAL-1 design).
    if let Some(notify) = state.ring_flush_shutdown.as_ref() {
        notify.notify_one();
    }

    // Step (d-2): Take the ring Arc from DaemonState (optional — belt-and-suspenders).
    // The notify signal is the primary close mechanism. Dropping the ring_arc here
    // ensures write_tx is closed even if the notify path were somehow missed.
    // Both mechanisms are safe to apply simultaneously.
    let ring_arc = state.ring.lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    drop(ring_arc);

    // Step (d-3): Await flush JoinHandle with 2-second bounded timeout.
    let flush_handle = state.ring_flush_handle.lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    if let Some(handle) = flush_handle {
        match tokio::time::timeout(
            std::time::Duration::from_secs(RING_FLUSH_DRAIN_TIMEOUT_SECS),
            handle,
        ).await {
            Ok(_) => tracing::info!(
                "ring flush task drained and exited cleanly (BC-2.04.012 durability)"
            ),
            Err(_elapsed) => tracing::warn!(
                timeout_secs = RING_FLUSH_DRAIN_TIMEOUT_SECS,
                "ring flush drain timeout — some pending records may be lost (BC-2.04.012 best-effort)"
            ),
        }
    }
}
```

The same drain step applies to `launch_daemon_in_process` shutdown tail.

##### Step 6: Correct the false comment

The comment at the ring-drain site (formerly main.rs ~line 230-232, revised in Round 2)
must state the new mechanism:

```rust
// The flush task exits via an explicit Notify signal (H1 fix) rather than relying on
// Arc<RingBuffer> refcount reaching zero. The notify_one() call triggers the select!
// drain branch in flush_task_loop, which drains all pending records via try_recv()
// and returns. The ring_arc drop below is belt-and-suspenders.
```

Remove any occurrence of "DaemonState holds the only Arc" — that claim is no longer part
of the close mechanism.

#### AC-E2E-008 Impact

AC-E2E-008 (ring durability under immediate SIGTERM) is not changed. The test verifies
that the JSONL file contains a PreToolUse record after SIGTERM. With the Notify path,
the flush task drains via `try_recv()` before exiting — all enqueued records are written.
The test continues to pass unchanged.

#### Files Affected by H1

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/ring.rs` | Add `RingShutdownNotify` type alias. Change `spawn_flush_task` signature to accept `ring_shutdown: RingShutdownNotify`. Restructure `flush_task_loop` to use `select! { _ = ring_shutdown.notified() => drain+exit, record = rx.recv() => write }`. Extract `flush_one_record` private helper to share write logic between normal and drain branches. |
| `crates/monocle-runtime/src/state.rs` | Add `ring_flush_shutdown: Option<RingShutdownNotify>` field. Initialize to `None` in `DaemonState::new()`. |
| `crates/monocle-runtime/src/lifecycle.rs` | Create `ring_shutdown_notify` at step 4b. Call `ring.spawn_flush_task(Arc::clone(&ring_shutdown_notify))`. Wire `ring_flush_shutdown: Some(Arc::clone(&ring_shutdown_notify))` into DaemonState construction. |
| `crates/monocle-runtime/src/main.rs` | Update ring drain step (d-1) to call `notify.notify_one()` before `drop(ring_arc)`. Update/remove the false "only Arc" comment. |
| `crates/monocle/src/auto_start.rs` | Same ring drain step (d-1) update in `launch_daemon_in_process` shutdown tail. |

**Files NOT changed by H1:**
- `daemon_e2e_serve.rs` — AC-E2E-008 is unchanged.
- `graceful_shutdown.rs` — does not call `spawn_flush_task`.
- Ring tests that call `spawn_flush_task` directly must pass a `Arc::new(Notify::new())`
  as the new parameter. Search for `spawn_flush_task` in test files and update call-sites.

---

### M3 — Fan-Out Task Shutdown Parity

#### Problem

`event_bus_fan_out_task` (event_bus.rs:69–93) exits ONLY when all `EventBusTx` senders
drop and `rx.recv()` returns `None`. The channel close mechanism is:

- Subprocess path: `std::process::exit` kills all threads without running destructors.
  `DaemonState.event_bus_tx` (the `Arc<EventBusTx>` holder) is never explicitly dropped.
  OS reclaims everything. This is benign — the process is gone.
- In-process path (`auto_start.rs:launch_daemon_in_process`): the tokio task runs the
  full lifecycle including the shutdown tail. After `run_server` returns and the shutdown
  tail completes, the task's `Arc<DaemonState>` drops. But if OTHER parts of the runtime
  still hold clones of `Arc<DaemonState>` (e.g., spawned tasks that haven't exited yet),
  the `event_bus_tx` is not dropped, the channel stays open, and the fan-out task keeps
  looping. In the in-process path, the TUI main task continues to run — it holds no
  `Arc<DaemonState>` itself, but the accept-loop task and the debounce task do.

The `drop_counter_debounce_task` already has a `shutdown_rx` select branch (added in
Round 2 C1). The fan-out task does not. This is an asymmetry: both tasks are long-running
background tasks that should exit promptly on shutdown. The debounce task exits within
one 100ms interval; the fan-out task can linger until all DaemonState clones drop — which
is non-deterministic under tokio's task scheduling.

For the subprocess path this is benign (OS kills all tasks). For the in-process path it
is a potential resource leak: the fan-out task continues consuming the event bus channel
after the daemon has logically shut down, until all `Arc<DaemonState>` holders drop.

#### Decision: Add `shutdown_rx` select branch to `event_bus_fan_out_task`

Mirror the `drop_counter_debounce_task` pattern exactly.

**Required signature change for `event_bus_fan_out_task`:**

```rust
// Current (event_bus.rs line 69):
pub async fn event_bus_fan_out_task(mut rx: EventBusRx, state: Arc<DaemonState>) { ... }

// New:
pub async fn event_bus_fan_out_task(
    mut rx: EventBusRx,
    state: Arc<DaemonState>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) { ... }
```

**Loop body change (add select! around the recv):**

```rust
pub async fn event_bus_fan_out_task(
    mut rx: EventBusRx,
    state: Arc<DaemonState>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    None => {
                        // All senders dropped — channel closed. Exit cleanly (BC-2.04.011 PC-7).
                        tracing::info!("event bus channel closed; fan-out task exiting");
                        return;
                    }
                    Some(event) => {
                        let _ = &state;
                        tracing::trace!(
                            received_at = %event.received_at,
                            "fan-out: event received (no TUI clients in Phase 1)"
                        );
                        // Phase 1: no TUI clients. S-032 will add per-client fan-out here.
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                tracing::debug!("event_bus_fan_out_task: shutdown signal received; exiting");
                return;
            }
        }
    }
}
```

**Call-site in `lifecycle.rs` step 5b (from Round 2 C1 addendum):**

The step 5b spawn block must be updated to pass `shutdown_rx`:

```rust
// Old (Round 2 C1):
let fan_out_state = Arc::clone(&daemon_state);
tokio::spawn(crate::event_bus::event_bus_fan_out_task(event_rx, fan_out_state));

// New (Round 3 M3):
let fan_out_state = Arc::clone(&daemon_state);
let fan_out_shutdown_rx = daemon_state.shutdown_rx.clone();
tokio::spawn(crate::event_bus::event_bus_fan_out_task(
    event_rx,
    fan_out_state,
    fan_out_shutdown_rx,
));
```

#### In-Process Shutdown Sequence

After the M3 change, the in-process daemon shutdown sequence is:

1. `run_server` returns (shutdown_tx sends `true`).
2. Shutdown tail: UDS cleanup → lock release → hooks-settings removal → ring drain.
3. `shutdown_rx` is closed when `shutdown_tx` (inside `DaemonState`) drops. But
   `shutdown_tx` drops only when `DaemonState` drops — which happens when all
   `Arc<DaemonState>` holders release. The `fan_out_shutdown_rx` has already cloned
   `shutdown_rx` which is a cheap `watch::Receiver` clone. The `shutdown_rx.changed()`
   branch fires as soon as `shutdown_tx.send(true)` is called — which happens when the
   `POST /shutdown` handler fires, BEFORE `run_server` returns. Therefore, the fan-out
   task exits promptly within one tokio scheduler tick of the shutdown signal, not
   waiting for all `Arc<DaemonState>` holders to drop.

#### Files Affected by M3

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/event_bus.rs` | Add `shutdown_rx: tokio::sync::watch::Receiver<bool>` parameter to `event_bus_fan_out_task`. Add `select!` shutdown branch to the loop body. |
| `crates/monocle-runtime/src/lifecycle.rs` | Update step 5b fan-out spawn to pass `daemon_state.shutdown_rx.clone()` as the third argument. |

---

### H2-doc — MONOCLE_RING_FLUSH_DELAY_MS Second E2E Affordance (Retroactive Authorization)

#### Background

During F-1 implementation (while implementing the ring flush path), the implementer added
`MONOCLE_RING_FLUSH_DELAY_MS` as a second e2e-test-affordances-gated env var alongside
`MONOCLE_HOOK_DELAY_MS`. This affordance:

- Is gated under `#[cfg(feature = "e2e-test-affordances")]` (same gate as HIGH-1).
- When set, inserts an artificial delay inside `flush_task_loop` between processing
  records, creating a window where records are enqueued but not yet written to disk.
- Purpose: make AC-E2E-008 non-vacuous. Without this affordance, the hook record is
  written to disk almost instantaneously by the flush task (bounded channel, fast disk),
  making it impossible to exercise the "shutdown arrives while records are queued-but-unflushed"
  scenario that AC-E2E-008 is designed to verify.

This affordance was not in the Round 2 HIGH-1 spec (which only authorized
`MONOCLE_HOOK_DELAY_MS`). It was added by the implementer under engineering judgment
during F-1. The spec must retroactively authorize it.

#### Decision: Authorize as Second Affordance Under `e2e-test-affordances`

`MONOCLE_RING_FLUSH_DELAY_MS` is hereby authorized as the second e2e-test-affordances-gated
affordance. The authorization criteria are identical to HIGH-1:

- **Cargo feature gate required.** The env var read and the sleep inside `flush_task_loop`
  MUST be inside `#[cfg(feature = "e2e-test-affordances")]` blocks. Without the feature,
  the code path does not exist in the production binary.
- **Purpose is AC-E2E-008 non-vacuity.** `MONOCLE_RING_FLUSH_DELAY_MS` inserts a delay
  between flush task writes, creating a window where records are queued but not yet on disk.
  The AC-E2E-008 test uses this to verify that the shutdown drain path (the `try_recv()`
  loop in H1's drain branch) actually writes those pending records before exit — not just
  that records were written by the flush task before shutdown arrived.
- **Dual-config build requirement.** The implementation MUST compile cleanly under BOTH:
  - `cargo build -p monocle-runtime` (no features) — production binary, delay absent.
  - `cargo build -p monocle-runtime --features e2e-test-affordances` — test binary, delay present.
  Both must pass `cargo clippy --workspace --all-targets -- -D warnings` with no warnings.
- **Production binary pollution check.** After implementation:
  `grep -rn "MONOCLE_RING_FLUSH_DELAY_MS" crates/monocle-runtime/src/` must return only
  `#[cfg(feature = "e2e-test-affordances")]`-gated lines. Zero ungated occurrences.

#### Doc Entry Required in `ring.rs`

The affordance must be documented in `spawn_flush_task`'s doc comment (or a dedicated
doc comment above the gated block) so it is discoverable:

```rust
/// ## E2E test affordances (feature = "e2e-test-affordances" only)
///
/// When the `e2e-test-affordances` cargo feature is enabled, two env vars are active:
///
/// - `MONOCLE_HOOK_DELAY_MS` — outer handler delay before the 300ms inner timeout
///   budget (pre_tool_use.rs). Used by AC-E2E-007 (drain timeout test).
/// - `MONOCLE_RING_FLUSH_DELAY_MS` — delay between flush task writes. Used by
///   AC-E2E-008 to create a window where records are enqueued but not yet on disk,
///   verifying that the shutdown drain path writes all pending records before exit.
///
/// Both affordances are absent from production binaries compiled without the feature.
```

Place this block in `spawn_flush_task`'s existing doc comment section, under a new
`## E2E test affordances` heading.

#### State.rs Field Doc Update

If the implementer added a corresponding `ring_flush_delay_ms: Option<u64>` field to
`DaemonState` (analogous to `hook_outer_delay_ms`), its doc comment must include the
feature gate note following the HIGH-1 pattern for `hook_outer_delay_ms`.

#### Files Affected by H2-doc

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/ring.rs` | Add `## E2E test affordances` doc section to `spawn_flush_task`. Verify `MONOCLE_RING_FLUSH_DELAY_MS` env var read is `#[cfg(feature = "e2e-test-affordances")]`-gated. Verify `flush_task_loop` sleep is gated. |
| `crates/monocle-runtime/src/state.rs` | If `ring_flush_delay_ms` field exists: update doc comment to note feature gate. |

---

### F-DW-HIGH-001 — CI E2E Job Missing (Route to Devops)

#### Finding

AC-E2E-007 (`#[cfg(feature = "e2e-test-affordances")]`) and AC-E2E-008 are both gated
under the `e2e-test-affordances` feature. The standard CI run is:

```
cargo test --workspace --locked
```

This does NOT pass `--features e2e-test-affordances`, so AC-E2E-007 and AC-E2E-008 are
never compiled or run by CI. They appear to pass only because they are silently excluded
from compilation. The spec at §HIGH-1 line ~1477 mandates:

> "The CI drain-test job must explicitly pass `--features e2e-test-affordances` when
> running AC-E2E-007."

This obligation is live but unexecuted. AC-E2E-008 was added in Round 2 and has the same
gap. Both tests are false-green in CI.

#### Routing Decision

This is a devops-engineer responsibility per the routing table (CI/CD configuration).
**No spec design is required here.** The architect reaffirms the existing CI obligation
and provides the exact invocation.

#### Exact CI Invocation (Devops Must Add)

A dedicated CI job or step — separate from the standard `cargo test --workspace` run —
must execute:

```bash
cargo test -p monocle-runtime \
  --test daemon_e2e_serve \
  --features e2e-test-affordances \
  --locked
```

**Job configuration requirements:**
- `timeout-minutes: 5` (AC-E2E-007 has a ~11s assertion window; AC-E2E-008 has a ~15s
  window; the overall test binary compiles + runs all E2E tests; 5 minutes is sufficient).
- Must run only when `monocle-runtime` crate files change (path filter:
  `crates/monocle-runtime/**`), or on all PRs to `develop` (simpler, lower risk of
  false-skip).
- Must NOT be combined with the standard `cargo test --workspace` step — it is an
  additive E2E gate, not a replacement.
- Must run AFTER the standard `cargo build --workspace --locked` step (daemon binary
  must be compiled before the integration tests that spawn it).
- The job name in CI YAML: `e2e-affordances-tests` (or similar that makes the scope clear).

**Routing:** devops-engineer implements this CI change. No architect or implementer action
required beyond this routing note.

---

### Updated Invariants Table (Round 3)

| Invariant | How it is preserved |
|-----------|---------------------|
| Ring flush drain is deterministic on FORCE-CLOSE path | `ring_shutdown_notify.notify_one()` signals the flush task explicitly; task drains via `try_recv()` loop then exits. No reliance on Arc refcount. |
| No production binary pollution from test affordances | `MONOCLE_HOOK_DELAY_MS` and `MONOCLE_RING_FLUSH_DELAY_MS` both gated by `#[cfg(feature = "e2e-test-affordances")]`. |
| AC-E2E-008 non-vacuity | `MONOCLE_RING_FLUSH_DELAY_MS` (authorized in H2-doc) creates the queued-but-unflushed window the test must observe. |
| Event bus fan-out task exits promptly on shutdown | `shutdown_rx.changed()` select branch added (M3); exits within one tokio scheduler tick of `shutdown_tx.send(true)`. |
| Debounce task and fan-out task have symmetric shutdown behavior | Both tasks now exit via `shutdown_rx.changed()`. Both are covered by the same shutdown signal. |
| CI E2E job covers feature-gated tests | Devops-engineer must add `cargo test --features e2e-test-affordances` job (F-DW-HIGH-001 routing). |
| False "DaemonState holds the only Arc" comment removed | H1 fix removes the comment; the new mechanism does not depend on that invariant. |

---

### Updated Implementer Change-List (v1.3.0)

The v1.2.0 implementer change-list is extended with Round 3 additions:

| File | Change | Finding |
|------|--------|---------|
| `crates/monocle-runtime/src/ring.rs` | Add `RingShutdownNotify` type alias. Change `spawn_flush_task` to accept `ring_shutdown`. Restructure `flush_task_loop` with `select!` drain branch. Extract `flush_one_record` helper. Add E2E affordances doc section. Verify `MONOCLE_RING_FLUSH_DELAY_MS` is feature-gated. | H1, H2-doc |
| `crates/monocle-runtime/src/state.rs` | Add `ring_flush_shutdown: Option<RingShutdownNotify>` field. Initialize to `None` in `new()`. Update `ring_flush_delay_ms` doc comment if field exists. | H1, H2-doc |
| `crates/monocle-runtime/src/lifecycle.rs` | Create `ring_shutdown_notify` at step 4b. Pass to `spawn_flush_task`. Wire `ring_flush_shutdown` into DaemonState construction. Update step 5b fan-out spawn to pass `shutdown_rx`. | H1, M3 |
| `crates/monocle-runtime/src/main.rs` | Update ring drain step (d-1) to call `notify.notify_one()` before `drop(ring_arc)`. Remove/replace false "only Arc" comment. | H1 |
| `crates/monocle/src/auto_start.rs` | Same ring drain step (d-1) update in `launch_daemon_in_process`. | H1 |
| `crates/monocle-runtime/src/event_bus.rs` | Add `shutdown_rx` parameter to `event_bus_fan_out_task`. Add `select!` shutdown branch to loop body. | M3 |

**Devops routing (CI only — not implementer scope):**

| File | Change | Finding |
|------|--------|---------|
| CI YAML (`.github/workflows/` or equivalent) | Add `e2e-affordances-tests` job: `cargo test -p monocle-runtime --test daemon_e2e_serve --features e2e-test-affordances --locked`, `timeout-minutes: 5`. | F-DW-HIGH-001 |

**Files unchanged from v1.2.0:**
- `crates/monocle-runtime/tests/daemon_e2e_serve.rs` — AC-E2E-007 and AC-E2E-008 are unchanged by Round 3.
- `crates/monocle-runtime/src/server.rs` — no changes.
- `crates/monocle-runtime/src/lock.rs` — no changes.
- `crates/monocle-runtime/tests/graceful_shutdown.rs` — no changes.

**Ring test call-sites update (implementer must check):**

Any test that calls `ring.spawn_flush_task()` directly must be updated to pass
`Arc::new(tokio::sync::Notify::new())` as the new `ring_shutdown` parameter. Search:

```bash
grep -rn "spawn_flush_task" crates/monocle-runtime/
```

Update all call-sites to match the new signature.

---

### Registry Atomicity Note (for state-manager)

This addendum bumps `SS-daemon-wiring-impl` from version `1.2.0` to `1.3.0`. The
`version-pin-registry.yaml` entry must be updated in the same factory-artifacts commit
as this spec file (CLAUDE.md REGISTRY ATOMICITY rule, CI POL-11).

Required update in `.factory/specs/version-pin-registry.yaml` (already reflected above):

```yaml
SS-daemon-wiring-impl:
  path: specs/architecture/SS-daemon-wiring-impl.md
  current_version: "1.3.0"
  last_bump_commit: "[DAEMON-WIRE adversarial fix addendum Round 3 H1/M3/H2-doc/F-DW-HIGH-001]"
  last_bump_date: "2026-06-03"
```
