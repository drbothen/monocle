# S-002 Demo Evidence: Healthz Endpoint

**Story:** S-002 — Healthz Endpoint (BC-2.01.001 Unauthenticated Liveness Probe)
**Generated:** 2026-05-25T21:20:10Z
**Product Type:** Rust library (no runnable daemon binary in Wave 2; all ACs verified via in-process integration tests using `tower::ServiceExt::oneshot`)
**Branch:** main (S-002 worktree at `/Users/jmagady/Dev/monocle/.worktrees/S-002`)

---

## Build Status

```
$ cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
```

Result: CLEAN — zero errors, zero warnings.

---

## Lint Status

```
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```

Result: CLEAN — zero warnings, zero lint violations across all targets including test code.

---

## Test Suite: healthz_endpoint Integration Tests

```
$ cargo test -p monocle-runtime --test healthz_endpoint -- --nocapture

running 20 tests
test test_BC_2_01_001_invariant_healthz_does_not_import_constant_time_eq ... ok
test test_BC_2_01_001_invariant_healthz_does_not_import_monocle_tui ... ok
test test_BC_2_01_001_invariant_default_body_limit_on_auth_router_only ... ok
test test_BC_2_01_001_invariant_semver_regex_shape ... ok
test test_BC_2_01_001_large_body_returns_200_not_413 ... ok
test test_BC_2_01_001_hook_receiver_abnormal_exit_returns_503 ... ok
test test_BC_2_01_001_garbage_auth_header_is_ignored_returns_200 ... ok
test test_BC_2_01_001_shutting_down_body_has_exactly_one_key ... ok
test test_BC_2_01_001_hook_receiver_healthy_returns_200 ... ok
test test_BC_2_01_001_response_body_has_exactly_three_keys ... ok
test test_BC_2_01_001_uptime_sec_is_integer_gte_zero ... ok
test test_BC_2_01_001_valid_auth_header_is_ignored_returns_200 ... ok
test test_BC_2_01_001_version_equals_cargo_pkg_version ... ok
test test_BC_2_01_001_normal_mode_returns_200_alive ... ok
test test_BC_2_01_001_response_within_100ms ... ok
test test_BC_2_01_001_no_auth_header_returns_200_not_401 ... ok
test test_BC_2_01_001_poisoned_lock_returns_503 ... ok
test test_BC_2_01_001_shutting_down_mode_returns_503 ... ok
test test_BC_2_01_001_shutting_down_with_failed_hook_receiver_returns_503 ... ok
test test_BC_2_01_001_version_matches_semver_regex ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Note: The `thread '<unnamed>' panicked` line during `test_BC_2_01_001_poisoned_lock_returns_503` is expected and intentional — that test deliberately poisons an RwLock by spawning a thread that panics while holding the write guard, then asserts the handler degrades gracefully to HTTP 503 instead of propagating the panic. The test itself passes (`ok`).

---

## AC-to-Test Mapping

### AC-001: Normal mode → HTTP 200 with `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}`

| Test Function | What It Verifies | Result |
|---|---|---|
| `test_BC_2_01_001_normal_mode_returns_200_alive` | AppMode::Running → HTTP 200, body.status == "alive" | PASS |
| `test_BC_2_01_001_response_body_has_exactly_three_keys` | Body has exactly 3 keys: status, uptime_sec, version | PASS |
| `test_BC_2_01_001_uptime_sec_is_integer_gte_zero` | uptime_sec is a JSON integer (not string), value >= 0 | PASS |
| `test_BC_2_01_001_version_matches_semver_regex` | version matches SemVer 2.0 regex, no leading `v` | PASS |
| `test_BC_2_01_001_version_equals_cargo_pkg_version` | version == CARGO_PKG_VERSION at compile time | PASS |
| `test_BC_2_01_001_hook_receiver_healthy_returns_200` | Running + hook_receiver_status=Some(Ok) → HTTP 200 (3-key body) | PASS |

AC-001 verdict: **PASS** (6/6 tests green)

---

### AC-002: ShuttingDown mode OR hook-receiver failed → HTTP 503 with `{"status":"shutting_down"}`

| Test Function | What It Verifies | Result |
|---|---|---|
| `test_BC_2_01_001_shutting_down_mode_returns_503` | AppMode::ShuttingDown → HTTP 503, body.status == "shutting_down" | PASS |
| `test_BC_2_01_001_shutting_down_body_has_exactly_one_key` | ShuttingDown body has exactly 1 key per BC literal | PASS |
| `test_BC_2_01_001_hook_receiver_abnormal_exit_returns_503` | Running + hook_receiver_status=Some(Err) → HTTP 503 | PASS |
| `test_BC_2_01_001_shutting_down_with_failed_hook_receiver_returns_503` | ShuttingDown + hook_receiver=Some(Err) → HTTP 503 (compound path) | PASS |
| `test_BC_2_01_001_poisoned_lock_returns_503` | Poisoned RwLock degrades gracefully to HTTP 503 (safe sentinel) | PASS |

AC-002 verdict: **PASS** (5/5 tests green; covers ShuttingDown, hook-receiver-failed, compound, and poisoned-lock paths)

---

### AC-003: No auth header → HTTP 200 (not 401)

| Test Function | What It Verifies | Result |
|---|---|---|
| `test_BC_2_01_001_no_auth_header_returns_200_not_401` | No X-Monocle-Authorization → HTTP 200, explicitly asserts != 401 | PASS |
| `test_BC_2_01_001_valid_auth_header_is_ignored_returns_200` | Valid-format auth header present → HTTP 200 (header silently ignored) | PASS |
| `test_BC_2_01_001_garbage_auth_header_is_ignored_returns_200` | Garbage auth header present → HTTP 200 (no auth middleware on unauth router) | PASS |

AC-003 verdict: **PASS** (3/3 tests green; no-header, valid-header, and garbage-header paths all return 200)

---

### AC-004: No body limit on healthz (large body → 200, not 413)

| Test Function | What It Verifies | Result |
|---|---|---|
| `test_BC_2_01_001_large_body_returns_200_not_413` | 1 MiB body (4x the 256 KiB authenticated limit) → HTTP 200, not 413 | PASS |

AC-004 verdict: **PASS** (1/1 test green; DefaultBodyLimit not applied to unauthenticated router)

---

### AC-005: /healthz NOT on authenticated router

| Test Function | What It Verifies | Result |
|---|---|---|
| `test_BC_2_01_001_invariant_default_body_limit_on_auth_router_only` | Source grep: DefaultBodyLimit absent from router.rs and healthz.rs non-comment lines | PASS |
| `test_BC_2_01_001_invariant_healthz_does_not_import_constant_time_eq` | Source grep: constant_time_eq absent from healthz.rs non-comment lines (auth path only) | PASS |
| `test_BC_2_01_001_invariant_healthz_does_not_import_monocle_tui` | Source grep: monocle-tui absent from healthz.rs (not a Phase 1 crate) | PASS |

AC-005 verdict: **PASS** (3/3 structural invariant tests green; healthz confirmed outside auth boundary at source level)

---

### AC-006: Response within 100ms

| Test Function | What It Verifies | Result |
|---|---|---|
| `test_BC_2_01_001_response_within_100ms` | In-process tower::oneshot latency < 100ms (EC-040 TUI hung-daemon detection boundary) | PASS |

AC-006 verdict: **PASS** (1/1 test green; in-process handler confirms <100ms without TCP overhead)

---

### Auxiliary / VP-001 Proof Method Tests

| Test Function | What It Verifies | Result |
|---|---|---|
| `test_BC_2_01_001_invariant_semver_regex_shape` | SemVer regex matches 12 valid forms, rejects 8 invalid forms including `v`-prefixed strings | PASS |

---

## Full Workspace Regression Check

```
$ cargo test --workspace --locked

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  [healthz_endpoint]
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  [workspace_structure monocle-runtime]
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   [monocle-test-harness unit]
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  [integration_auth]
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  [integration_bc_hooks]
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   [integration_binary]
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  [integration_endpoints]
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  [integration_fidelity]
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  [integration_filters]
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  [integration_payload]
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   [workspace_structure xtask]
```

Total workspace: **154 tests, 154 passed, 0 failed, 0 regressions introduced by S-002.**

---

## Evidence Summary

| AC | Description | Tests | Verdict |
|---|---|---|---|
| AC-001 | Normal mode → HTTP 200 alive JSON | 6 | PASS |
| AC-002 | ShuttingDown OR hook-receiver failed → HTTP 503 | 5 | PASS |
| AC-003 | No auth header → HTTP 200 (not 401) | 3 | PASS |
| AC-004 | Large body → 200 (no body limit on healthz) | 1 | PASS |
| AC-005 | /healthz not on authenticated router | 3 | PASS |
| AC-006 | Response within 100ms | 1 | PASS |
| Auxiliary | SemVer regex property corpus | 1 | PASS |
| **Total** | | **20** | **20/20 PASS** |

**Overall: S-002 VERIFIED. All 6 acceptance criteria satisfied. Zero regressions. Build and lint clean.**

---

## Note on VHS Recording

S-002 is a pure Rust library story: `monocle-runtime` exposes handlers and a router that are exercised in-process via `tower::ServiceExt::oneshot`. There is no runnable daemon binary that accepts TCP connections in Wave 2. VHS terminal recordings are not applicable — they require a binary invoked at the command line. The authoritative demo evidence for a library story is the integration test output above, which is deterministic, reproducible, and directly traces every line of test logic to the BC clauses it exercises.

Per the Demo Recorder operating procedure, VHS recordings are used for CLI products. A daemon binary (S-004 or later) is the appropriate target for VHS-based demos when it ships.
