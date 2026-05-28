# monocle-tui Coverage Deferrals

This file documents test coverage gaps that require out-of-process or
infrastructure-level test harness support beyond what S-025 delivered.
Each entry names the acceptance criterion, explains why inline coverage is
not feasible, and anchors the deferral to a future story.

---

## AC-002 — Error panel content reachable from production startup path

**AC:** When the UDS connection fails, `run()` renders an error panel containing
`"Daemon not running. Start it with: monocle daemon start"` and waits for a
keypress.

**Existing coverage:** `test_bc_2_06_004_pc1_ac002_error_message_text_is_canonical`
(in `startup_connect.rs`) asserts that the string constant at `app.rs:446` matches
the canonical value. It does NOT exercise the `run()` code path — the assertion is
on the string literal, not on an actual render.

**Why not covered inline:** Reaching `app.rs:446` from a test requires:
1. A mock or in-process UDS socket that refuses the `connect()` call, AND
2. Either spawning `run()` in a background task with a synthetic keyboard input
   (to satisfy the `event::poll` wait loop), or refactoring the connection error
   branch into a testable helper.

Both approaches require an async runtime fixture (tokio::test) that injects a
UnixStream substitute. The test surface for `run()` is out-of-scope for S-025's
unit/integration test layer — it belongs to end-to-end or component test infra.

**Deferred to:** HS-EXP-009 (holdout evaluation scenario for exit/error paths).
When HS-EXP-009 is scheduled, the implementer should add a
`run_with_transport(transport: impl AsyncRead + AsyncWrite)` overload and test
the error branch via a `tokio_test::io::Builder` mock transport.

---

## AC-009 — Panic hook restores terminal raw mode

**AC:** If the TUI panics after `terminal::setup()`, the panic hook installed by
`main()` must restore the terminal to cooked mode before unwinding, preventing
a raw-mode leak in the user's shell.

**Existing coverage:** The panic hook installation is exercised by
`test_bc_2_06_007_pc1_ac001_app_constructs_for_startup` (verifies the `App`
struct constructs without panic). The hook itself is installed in `main()` at
program startup — not in `run()` or any library function.

**Why not covered inline:** Testing a panic hook requires triggering a real panic
and asserting that `crossterm::terminal::disable_raw_mode()` was called before
the process exited or the thread unwound. Options:
1. **Sub-process test:** spawn the binary with `std::process::Command`, inject a
   panic trigger (e.g., via an environment variable), and assert the exit code /
   terminal state from the parent process. This requires build-artifact access in
   the test, not available in unit/integration test context.
2. **Thread-level panic with catch_unwind:** `std::panic::catch_unwind` can capture
   the panic, but raw-mode state is a process-global side effect — asserting it
   from within the same process requires global state mutation tracking, which
   conflicts with parallel test execution.

**Deferred to:** HS-EXP-009 (exit/error path holdout scenarios). The holdout
evaluator should exercise the binary directly via `std::process::Command` and
assert on terminal state after an injected panic.
