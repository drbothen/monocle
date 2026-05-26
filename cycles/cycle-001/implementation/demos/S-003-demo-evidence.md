# S-003 Demo Evidence — Status Endpoint (Authenticated Daemon State)

**Story:** S-003 — Status Endpoint (Authenticated Daemon State)
**Branch:** `story/S-003-status-endpoint`
**Date:** 2026-05-25
**Product type:** CLI/library (Rust workspace, no binary daemon in Phase 1)
**Recording method:** Integration test output (no running daemon — library-level story)

---

## Test Execution Summary

| Test suite | Tests | Result |
|---|---|---|
| `status_endpoint_auth` (32 tests) | 32 | PASS |
| `status_abi_version` (12 tests) | 12 | PASS |
| **S-003 subtotal** | **44** | **PASS** |
| Workspace total (all crates) | 199 | PASS |

All 199 workspace tests pass. Zero failures. Zero warnings from `cargo clippy --workspace --all-targets -- -D warnings`.

---

## AC Coverage Map

### AC-001: Valid auth → 200 with 10 fields

**Tests covering this AC:**

| Test name | Assertion |
|---|---|
| `test_BC_2_01_002_valid_canonical_auth_returns_200_with_all_fields` | HTTP 200, exactly 10 fields present |
| `test_BC_2_01_002_field_types_are_correct` | All 10 fields have correct JSON types simultaneously |
| `test_BC_2_01_002_pid_is_positive_u32` | `pid` is integer >= 1 (POSIX: PID 0 reserved) |
| `test_BC_2_01_002_version_matches_semver_regex` | `version` matches SemVer 2.0, no leading `v`, equals CARGO_PKG_VERSION |
| `test_BC_2_01_002_float_fields_in_valid_range` | `ring_buffer_fill_pct` and `channel_saturation_pct` in [0.0, 100.0] |
| `test_BC_2_01_002_uptime_sec_is_u64` | `uptime_sec` is JSON integer (non-negative) |
| `test_BC_2_01_002_tui_attached_is_bool_defaults_false` | `tui_attached` is JSON boolean, defaults `false` |
| `test_BC_2_01_002_lock_file_is_string` | `lock_file` is JSON string |

**Result: PASS** — HTTP 200 returned with all 10 required fields (`pid`, `uptime_sec`, `version`, `abi_version`, `lock_file`, `hook_endpoints`, `ring_buffer_fill_pct`, `channel_saturation_pct`, `last_hook_ts`, `tui_attached`), each with correct JSON type.

---

### AC-002: Alias auth → 200 + WARN log

**Tests covering this AC:**

| Test name | Assertion |
|---|---|
| `test_BC_2_01_002_alias_auth_returns_200_and_emits_warn` | `X-Claude-Code-Ide-Authorization` with valid 64-hex token → HTTP 200, 10-field body |
| `test_BC_2_01_002_alias_auth_warn_message_content_matches_inv6` | WARN event captured with exact INV-6 normative string |

**WARN message verified (INV-6 normative string):**
```
WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization
```

**Result: PASS** — Alias path returns HTTP 200 AND emits the exact INV-6 WARN string. Implemented via globally-registered `GlobalWarnCapture` subscriber to avoid tracing callsite-caching race with parallel test runner.

---

### AC-003: Missing auth → 401 E-AUTH-001

**Tests covering this AC:**

| Test name | Assertion |
|---|---|
| `test_BC_2_01_002_no_auth_returns_401_missing_token` | No auth headers → HTTP 401, `{"error":"missing_auth_token"}` (exactly 1 key) |
| `test_BC_2_01_002_drain_unauthenticated_still_returns_401` | No auth during ShuttingDown → still HTTP 401 E-AUTH-001 (drain does not relax auth) |
| `test_BC_2_01_002_bearer_auth_header_returns_401_missing` | `Authorization: Bearer <token>` (wrong header name) → treated as missing → E-AUTH-001 |

**Result: PASS** — Missing auth header returns HTTP 401 with `{"error":"missing_auth_token"}` in all contexts including drain mode.

---

### AC-004: Invalid token → 401 E-AUTH-002

**Tests covering this AC:**

| Test name | Assertion |
|---|---|
| `test_BC_2_01_002_wrong_token_returns_401_invalid_token` | Valid-format token with wrong hex value → HTTP 401, `{"error":"invalid_auth_token"}` |
| `test_BC_2_01_002_empty_canonical_header_value_returns_401_invalid` | Empty value for canonical header → E-AUTH-002 (not E-AUTH-001: header IS present) |
| `test_BC_2_01_002_canonical_prefix_only_no_hex_returns_401_invalid` | `monocle-v1:` with no hex suffix → E-AUTH-002 |
| `test_BC_2_01_002_canonical_header_without_prefix_returns_401_invalid` | Correct token value but no `monocle-v1:` prefix → E-AUTH-002 |
| `test_BC_2_01_002_wrong_version_prefix_returns_401_invalid` | `monocle-v2:<token>` → E-AUTH-002 |
| `test_BC_2_01_002_alias_wrong_secret_returns_401_invalid` | Alias header with wrong 64-hex → E-AUTH-002 |
| `test_BC_2_01_002_alias_empty_value_returns_401_invalid` | Alias header with empty value → E-AUTH-002 |
| `test_BC_2_01_002_invariant_no_third_error_body` | Exactly 2 error bodies exist (E-AUTH-001 and E-AUTH-002); no third variant |
| `test_BC_2_01_002_invariant_missing_and_invalid_are_distinct_bodies` | Missing-auth and invalid-token bodies are structurally distinct (INV-2) |

**Result: PASS** — All invalid-token paths return HTTP 401 with `{"error":"invalid_auth_token"}`. Missing vs. invalid distinction preserved per BC-2.01.009 INV-2.

---

### AC-005: abi_version == 1

**Tests covering this AC:**

| Test name | Assertion |
|---|---|
| `test_BC_2_02_001_abi_version_field_equals_1` | `abi_version` field equals integer `1` (VP-011 Probe 11.a) |
| `test_BC_2_02_001_abi_version_matches_compile_time_const` | Runtime value equals `monocle_core::MONOCLE_ABI_VERSION` const |
| `test_BC_2_02_001_compile_time_drift_guard` | `src/main.rs` contains `MONOCLE_ABI_VERSION == 1` compile-time assert (structural grep) |
| `test_BC_2_02_001_status_handler_references_monocle_abi_version_const` | `src/handlers/status.rs` references the const (not a hardcoded literal) |
| `test_BC_2_02_001_status_handler_does_not_hardcode_abi_version_literal` | `abi_version: 1` (raw literal without const) does NOT appear in status handler |

**Result: PASS** — `abi_version` is `1`, bound to `monocle_core::MONOCLE_ABI_VERSION` const at compile time. Drift guard in `main.rs` prevents silent divergence.

---

### AC-006: hook_endpoints = 5 canonical paths

**Tests covering this AC:**

| Test name | Assertion |
|---|---|
| `test_BC_2_01_002_hook_endpoints_array_exactly_5_paths` | Array of exactly 5 strings in canonical spec order |

**Canonical order verified:**
1. `/hooks/pre-tool-use`
2. `/hooks/notification`
3. `/hooks/stop`
4. `/hooks/session-start`
5. `/hooks/prompt-submit`

**Result: PASS** — `hook_endpoints` is a JSON array of exactly 5 strings in the canonical order specified by BC-2.01.002 PC-1 + BC-2.01.008 PC-4.

---

### AC-007: last_hook_ts ISO 8601 format

**Tests covering this AC:**

| Test name | Assertion |
|---|---|
| `test_BC_2_01_002_last_hook_ts_has_exactly_5_fields` | `last_hook_ts` is JSON object with exactly 5 keys |
| `test_BC_2_01_002_last_hook_ts_field_names_are_canonical` | Keys are `pre_tool_use`, `notification`, `stop`, `session_start`, `prompt_submit` |
| `test_BC_2_01_002_last_hook_ts_unfired_hooks_are_null` | Unfired hooks serialize as JSON `null` (not string "null", not integer 0) |
| `test_BC_2_01_002_ec_044_unfired_hook_ts_is_json_null_not_string` | `Option<String>::None` serializes as JSON null (schema-level assertion) |
| `test_BC_2_01_002_ec_044_fired_hook_ts_uses_iso8601_ms_precision` | Fired hooks use `YYYY-MM-DDTHH:MM:SS.sssZ` (millisecond precision, UTC mandatory) |
| `test_BC_2_01_002_initial_state_ring_and_channel_are_zero` | Initial state: `ring_buffer_fill_pct=0.0`, `channel_saturation_pct=0.0` |

**Format validated:** `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` (ISO 8601 UTC with mandatory millisecond precision per `chrono::format("%Y-%m-%dT%H:%M:%S%.3fZ")`).

**Result: PASS** — Unfired hooks: JSON `null`. Fired hooks: ISO 8601 ms-precision UTC string. 5-field structure with canonical underscore-named keys.

---

### AC-008: /status serves during drain

**Tests covering this AC:**

| Test name | Assertion |
|---|---|
| `test_BC_2_01_002_status_serves_during_drain` | `AppMode::ShuttingDown` + valid auth → HTTP 200 with 10-field body (NOT 503) |
| `test_BC_2_01_002_drain_unauthenticated_still_returns_401` | `AppMode::ShuttingDown` + no auth → HTTP 401 (auth not relaxed during drain) |

**Result: PASS** — `/status` is drain-exempt per BC-2.01.002 PC-3. Valid authenticated requests continue to return HTTP 200 during graceful shutdown. Auth enforcement is not relaxed.

---

## Security Fix — Empty Token Bypass (F-S003-ADV2-001)

**Vulnerability patched:** Empty `auth_token` bypass via `constant_time_eq("", "") == true`.

**Attack vectors closed:**
- Canonical path: `X-Monocle-Authorization: monocle-v1:` → `strip_prefix` yields `""` → without `is_empty()` guard, `constant_time_eq("".as_bytes(), "".as_bytes())` returns `true`
- Alias path: `X-Claude-Code-Ide-Authorization: ` (empty value) → same empty-equals-empty bypass

**Fix:** `is_empty()` guard on `DaemonState.auth_token` placed BEFORE any header extraction. If the stored token is empty (pre-S-004 startup state), all requests are rejected with HTTP 401 `{"error":"invalid_auth_token"}`.

**Test verifying the fix:**

| Test name | Sub-tests | Assertion |
|---|---|---|
| `test_BC_2_01_009_empty_auth_token_rejects_all_requests` | 3 | Uninitialized token (`DaemonState::new()`) → all bypass vectors return HTTP 401 |

**Result: PASS** — All three bypass vectors correctly rejected.

---

## Additional Architecture Compliance Tests

| Test name | Assertion |
|---|---|
| `test_BC_2_01_002_invariant_auth_uses_constant_time_eq` | `src/auth.rs` uses `constant_time_eq` in non-comment executable code (NFR-010) |
| `test_BC_2_01_002_invariant_status_handler_does_not_compare_tokens_with_eq` | `handlers/status.rs` does NOT use `constant_time_eq` (auth is upstream in middleware) |
| `test_BC_2_01_002_invariant_status_handler_does_not_import_monocle_tui` | `handlers/status.rs` does NOT import `monocle_tui` (not a Phase 1 crate) |
| `test_BC_2_01_002_invariant_default_body_limit_on_auth_router` | `server.rs` applies `DefaultBodyLimit` in non-comment executable code |
| `test_BC_2_01_002_invariant_body_limit_value_is_256kib` | Body limit is `262144` bytes (256 KiB) per SS-daemon-lifecycle §Body Size Limit |
| `test_BC_2_01_002_both_headers_canonical_wins` | Both headers present → canonical wins, no WARN emitted (EC-011 / PC-4) |
| `test_BC_2_01_002_canonical_header_present_alias_ignored` | Canonical header present → alias ignored regardless of alias value |
| `test_BC_2_01_002_poisoned_last_hook_ts_lock_returns_200_with_null_fields` | Poisoned `RwLock` on `last_hook_ts` → graceful degradation: HTTP 200, all-null fallback |

**Result: PASS** — All architecture compliance invariants verified.

---

## Clippy Status

```
cargo clippy --workspace --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```

Zero warnings. Zero errors.

---

## Branch Commits (S-003 implementation chain)

| SHA | Message |
|---|---|
| `3926588` | `test(S-003): F-S003-ADV1-002 + F-S003-ADV2-003 — WARN content capture + poison lock tests` |
| `3156117` | `fix(S-003): F-S003-ADV2-001 empty auth_token bypass closed` |
| `28e6243` | `feat(S-003): implement status endpoint, auth middleware, and server builder` |
| `ed72aa0` | `feat(S-3.01): add S-003 module stubs — status endpoint, auth middleware, server` |

---

## Coverage Summary

| AC | Description | Tests | Status |
|---|---|---|---|
| AC-001 | Valid auth → 200 + 10 fields | 8 | PASS |
| AC-002 | Alias auth → 200 + WARN log | 2 | PASS |
| AC-003 | Missing auth → 401 E-AUTH-001 | 3 | PASS |
| AC-004 | Invalid token → 401 E-AUTH-002 | 9 | PASS |
| AC-005 | abi_version == 1 | 5 | PASS |
| AC-006 | hook_endpoints = 5 canonical paths | 1 | PASS |
| AC-007 | last_hook_ts ISO 8601 format | 6 | PASS |
| AC-008 | /status serves during drain | 2 | PASS |
| Security fix F-S003-ADV2-001 | Empty token bypass prevention | 1 (3 sub-tests) | PASS |
| Architecture compliance | Structural source-grep invariants | 7 | PASS |
| **Total** | | **44** | **PASS** |

All 8 acceptance criteria verified. Workspace: 199 tests passing, 0 failing.
