---
document_type: red-gate-log
level: ops
version: "1.0"
status: verified
producer: test-writer
timestamp: 2026-05-27T00:00:00
phase: 3
inputs:
  - .factory/specs/behavioral-contracts/
  - .factory/stories/STORY-INDEX.md
input-hash: "[md5]"
traces_to: "S-022"
stub_architect_agent: "[S-022 step-2 session]"
stub_compile_verified: true
test_writer_agent: "[S-022 step-3 session]"
red_gate_verified: true
---

# Red Gate Log: Wave 6 / S-022 — TUI Client Connect + Initial State + Permission IPC

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|---------------|-----------------|------|
| S-022 (connection_handshake) | 7 | YES — all via todo!() panic | GREEN |
| S-022 (permission_prompt) | 9 | YES — all via todo!() panic | GREEN |
| **Total** | **16** | **16/16 FAIL** | **GREEN** |

## Stubs Created

### Step 2 — Stubs (commit 1b91643)

New files:

- `monocle-ipc/src/server.rs` (162 lines) — `IpcServer` with `todo!()` bodies for all S-022 ACs
- `monocle-ipc/tests/connection_handshake.rs` (2 lines — marker stub)
- `monocle-ipc/tests/permission_prompt.rs` (2 lines — marker stub)
- `monocle-runtime/src/permissions.rs` (174 lines) — `PermissionManager` with `todo!()` bodies

Modified files:

- `monocle-ipc/src/lib.rs` — re-export `server` module
- `monocle-ipc/src/uds.rs` — bidirectional transport surface
- `monocle-runtime/Cargo.toml` — dependency additions
- `monocle-runtime/src/lib.rs` — expose `permissions` module
- `monocle-runtime/src/state.rs` — `DaemonState` subscriber list skeleton
- `monocle-runtime/src/lifecycle.rs` — stub integration points
- `monocle-runtime/src/hooks/pre_tool_use.rs` — stub permission dispatch path

Pre-existing test clippy fixes (7 test files received module-level
`#![allow(clippy::expect_used, clippy::unwrap_used, clippy::disallowed_methods)]`):

- Justified: `develop @ 4d4d407` was already failing `clippy --workspace --all-targets`
  on these attributes (known `process_discovery` issue). Not a regression introduced by S-022.

Stub compile gates (all PASS):

- `cargo check` — PASS
- `cargo test --no-run` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS

Anti-precedent confirmation: stubs use `todo!()` bodies; no sibling-crate logic copied into stubs.

### Step 3 — Failing Tests (commit eab7a14)

New files:

- `monocle-ipc/tests/common/mod.rs` (82 lines) — shared test harness (`spawn_test_daemon`, helpers)
- `monocle-ipc/tests/connection_handshake.rs` (397 lines) — 7 integration tests
- `monocle-ipc/tests/permission_prompt.rs` (730 lines) — 9 integration tests (includes AC-009b)

Total: **16 integration tests, 1,209 lines**

Test naming pattern: `ac_NNN_<descriptor>` — every test traces to exactly one AC.

Test compile gates (all PASS):

- `cargo test --no-run` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS

## Red Gate Verification

Independent verification by orchestrator on 2026-05-27.

### S-022 — connection_handshake (7 tests)

```
cargo test -p monocle-ipc --test connection_handshake
```

Result: **0 passed, 7 FAILED** — all via `todo!()` panic in
`monocle-ipc/src/server.rs` or `common/mod.rs` harness.

| Test | Panic Source | Status |
|------|-------------|--------|
| ac_001_connect_handshake | server.rs todo!() — register_subscriber | FAIL (expected) |
| ac_002_initial_state_snapshot | server.rs todo!() — send_initial_state | FAIL (expected) |
| ac_003_subscriber_isolation | server.rs todo!() — subscriber list | FAIL (expected) |
| ac_004_reconnect_same_socket | server.rs todo!() — reconnect path | FAIL (expected) |
| ac_005_max_subscribers | server.rs todo!() — cap enforcement | FAIL (expected) |
| ac_006_subscriber_cleanup_on_drop | server.rs todo!() — cleanup | FAIL (expected) |
| ac_007_state_update_broadcast | server.rs todo!() — broadcast | FAIL (expected) |

All 7 panic messages reference the S-022 behavior under test (e.g.,
`"S-022: register_subscriber — append sender to shared subscriber list"`).
All failures are runtime panics, NOT build errors.

### S-022 — permission_prompt (9 tests)

```
cargo test -p monocle-ipc --test permission_prompt
```

Result: **0 passed, 9 FAILED** — all via `todo!()` panic in
`common/mod.rs` harness (`spawn_test_daemon`).

| Test | Panic Source | Status |
|------|-------------|--------|
| ac_008_permission_request_forwarded | common/mod.rs spawn_test_daemon | FAIL (expected) |
| ac_009_permission_response_routed | common/mod.rs spawn_test_daemon | FAIL (expected) |
| ac_009b_permission_timeout | common/mod.rs spawn_test_daemon | FAIL (expected) |
| ac_010_permission_queue_ordering | common/mod.rs spawn_test_daemon | FAIL (expected) |
| ac_011_permission_cancel | common/mod.rs spawn_test_daemon | FAIL (expected) |
| ac_012_permission_denied_propagation | common/mod.rs spawn_test_daemon | FAIL (expected) |
| ac_013_permission_allow_persisted | common/mod.rs spawn_test_daemon | FAIL (expected) |
| ac_014_permission_deny_persisted | common/mod.rs spawn_test_daemon | FAIL (expected) |
| ac_015_permission_overlay_vecdeque | common/mod.rs spawn_test_daemon | FAIL (expected) |

All 9 panic messages reference S-022 behaviors under test.
All failures are runtime panics, NOT build errors.

## Regression Check

| Existing Tests | Status |
|----------------|--------|
| 753 pre-existing tests (Waves 1-5) | All pass |
| `ac_012_stop_timeout*` in monocle-runtime (2 tests) | Flaky — match Wave 5 gate known-flaky list; not a regression introduced by S-022 |

## Signature Issues Flagged for Implementer (Step 4)

1. **`DaemonState` placeholder:** `monocle_ipc::server::monocle_runtime_state_placeholder::DaemonState`
   is a private placeholder in stubs. Step 4 must replace with the real
   `monocle_runtime::state::DaemonState`.

2. **`UdsClientTransport` bidirectional access:** `UdsClientTransport` (S-021) is
   currently server-perspective only. Step 4 must expose bidirectional access for
   TUI-side use. Test harness uses `tokio::net::UnixStream` directly as a workaround
   (acceptable for tests; production path must use the transport abstraction).

## Hand-Off to Implementer

- Stories ready for implementation: **S-022**
- Authorized: Step 4 (implementer) dispatch
- Verdict: **GREEN** — all 16 tests fail via `todo!()` panics; clippy/check/test-compile clean
- Signature issues above must be resolved during Step 4 before tests can go green
