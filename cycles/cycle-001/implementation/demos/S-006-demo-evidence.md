---
story_id: S-006
title: "Lock File Atomic Lifecycle (Create + Pid Check + Cleanup)"
points: 8
wave: 2
recorded_by: vsdd-factory:demo-recorder
recorded_at: 2026-05-25
product_type: library (no binary; Rust integration tests)
toolchain: cargo test (VHS not applicable — no CLI surface in this story)
---

# S-006 Demo Evidence — Lock File Atomic Lifecycle

## Summary

S-006 is a library-level story delivering `DaemonLock::acquire()`, `DaemonLock::release()`,
`monocle_runtime::auth::generate_session_token()`, and `resolve_runtime_dir()` in
`monocle-runtime`. There is no binary or TUI surface in this story, so VHS recording is not
applicable. Evidence is captured via cargo integration test runs.

All 14 acceptance criteria are covered. 30 tests pass across 2 integration test files. Clippy
reports zero warnings with `-D warnings`. No source code was modified during evidence capture.

---

## Test Run Results

### lock_file_lifecycle (23 tests)

```
cargo test -p monocle-runtime --test lock_file_lifecycle -- --nocapture 2>&1
```

```
running 23 tests
test test_BC_2_01_005_env_override_empty_string_falls_through ... ok
test test_BC_2_01_005_runtimedirunresolvable_when_no_home ... ok
test test_BC_2_01_005_env_override_monocle_runtime_dir ... ok
test test_BC_2_01_005_acquire_rejects_port_zero ... ok
test test_BC_2_01_005_runtime_dir_created_with_0o700 ... ok
test test_BC_2_01_005_release_removes_lock_file ... ok
test test_BC_2_01_005_live_pid_conflict_returns_error ... ok
test test_BC_2_01_005_lock_file_mode_is_0o600 ... ok
test test_BC_2_01_008_generate_session_token_is_random ... ok
test test_BC_2_01_005_clean_start_creates_lock_file ... ok
test test_BC_2_01_008_generate_session_token_format ... ok
test test_BC_2_01_005_lock_file_path_is_in_runtime_dir ... ok
test test_BC_2_01_008_auth_token_is_64_hex ... ok
test test_BC_2_01_005_json_has_7_fields_correct_types ... ok
test test_BC_2_01_005_json_start_time_is_iso8601 ... ok
test test_BC_2_01_005_json_field_contract_version_is_first ... ok
test test_BC_2_01_005_json_pid_field_is_current_process ... ok
test test_BC_2_01_005_json_app_field_is_monocle ... ok
test test_BC_2_01_005_release_removes_sock_file ... ok
test test_BC_2_01_005_runtime_dir_created_recursively ... ok
test test_BC_2_01_005_stale_pid_new_lock_acquired ... ok
test test_BC_2_01_005_stale_pid_cleaned_up ... ok
test test_BC_2_01_008_auth_token_matches_regex ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### lock_file_contract (7 tests)

```
cargo test -p monocle-runtime --test lock_file_contract -- --nocapture 2>&1
```

```
running 7 tests
test test_BC_2_01_010_contract_version_equals_1_and_is_first_key ... ok
test test_BC_2_01_010_invariant_contract_version_first_via_raw_scan ... ok
test test_BC_2_01_010_app_field_equals_monocle ... ok
test test_BC_2_01_010_string_contract_version_handled_gracefully ... ok
test test_BC_2_01_010_contract_version_key_absent_entirely_treated_as_stale ... ok
test test_BC_2_01_010_missing_contract_version_treated_as_stale ... ok
test test_BC_2_01_010_unknown_contract_version_treated_as_stale ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Total: 30 / 30 tests pass. 0 failures. 0 skipped.**

---

## Clippy Status

```
cargo clippy -p monocle-runtime --all-targets -- -D warnings
```

Result: `Finished` with zero warnings. No clippy findings.

---

## AC Coverage Map

| AC | Description | Test(s) | Result |
|----|-------------|---------|--------|
| AC-001 | Clean start: lock file created atomically at `<runtime_dir>/monocle.lock` with mode `0o600` via `tempfile::persist` | `test_BC_2_01_005_clean_start_creates_lock_file`, `test_BC_2_01_005_lock_file_mode_is_0o600`, `test_BC_2_01_005_lock_file_path_is_in_runtime_dir` | PASS |
| AC-002 | JSON schema: 7 fields in correct order (`contract_version` first), correct types, `app="monocle"`, ISO-8601 timestamp | `test_BC_2_01_005_json_has_7_fields_correct_types`, `test_BC_2_01_005_json_field_contract_version_is_first`, `test_BC_2_01_005_json_pid_field_is_current_process`, `test_BC_2_01_005_json_app_field_is_monocle`, `test_BC_2_01_005_json_start_time_is_iso8601` | PASS |
| AC-003 | Live PID conflict: `Err(LockFileConflict { pid })` when existing lock holds a live process | `test_BC_2_01_005_live_pid_conflict_returns_error` | PASS |
| AC-004 | Stale lock cleanup: dead PID (`ESRCH`) causes stale removal and new lock acquisition | `test_BC_2_01_005_stale_pid_cleaned_up`, `test_BC_2_01_005_stale_pid_new_lock_acquired` | PASS |
| AC-005 | Graceful shutdown: `release()` removes `monocle.lock` and `monocle.sock` | `test_BC_2_01_005_release_removes_lock_file`, `test_BC_2_01_005_release_removes_sock_file` | PASS |
| AC-006 | Runtime dir created with mode `0o700` when absent; recursive creation supported | `test_BC_2_01_005_runtime_dir_created_with_0o700`, `test_BC_2_01_005_runtime_dir_created_recursively` | PASS |
| AC-007 | `MONOCLE_RUNTIME_DIR` env override used when set and non-empty; empty string treated as unset | `test_BC_2_01_005_env_override_monocle_runtime_dir`, `test_BC_2_01_005_env_override_empty_string_falls_through` | PASS |
| AC-008 | macOS platform fallback: `data_local_dir()` used when `runtime_dir()` returns `None` | Covered implicitly by `test_BC_2_01_005_env_override_empty_string_falls_through` on macOS (platform where this worktree runs); `resolve_runtime_dir()` succeeds via fallback | PASS |
| AC-009 | `RuntimeDirUnresolvable` when HOME and all XDG paths are absent | `test_BC_2_01_005_runtimedirunresolvable_when_no_home` | PASS |
| AC-010 | `contract_version` is first key with value `1`; unknown version treated as stale | `test_BC_2_01_010_contract_version_equals_1_and_is_first_key`, `test_BC_2_01_010_invariant_contract_version_first_via_raw_scan`, `test_BC_2_01_010_unknown_contract_version_treated_as_stale`, `test_BC_2_01_010_app_field_equals_monocle`, `test_BC_2_01_005_json_field_contract_version_is_first` | PASS |
| AC-011 | Missing `contract_version` key treated as stale (log E-LOCK-002, restart) | `test_BC_2_01_010_missing_contract_version_treated_as_stale` | PASS |
| AC-012 | `contract_version` as string `"1"` instead of integer: handled gracefully, no crash | `test_BC_2_01_010_string_contract_version_handled_gracefully` | PASS |
| AC-013 | `contract_version` key entirely absent: treated as stale, new lock with `contract_version=1` written | `test_BC_2_01_010_contract_version_key_absent_entirely_treated_as_stale`, `test_BC_2_01_010_missing_contract_version_treated_as_stale` | PASS |
| AC-014 | Auth token: 64-char lowercase hex from `OsRng`, matches `/^[0-9a-f]{64}$/`, unique per call | `test_BC_2_01_008_auth_token_is_64_hex`, `test_BC_2_01_008_auth_token_matches_regex`, `test_BC_2_01_008_generate_session_token_format`, `test_BC_2_01_008_generate_session_token_is_random` | PASS |

**Coverage: 14 / 14 ACs covered. All PASS.**

---

## Security Highlights

### OsRng cryptographic token (BC-2.01.008 PC-1)

`monocle_runtime::auth::generate_session_token()` generates 32 bytes from `rand::rngs::OsRng`
(EXACT pin `=0.8.6` per SS-deps-pin-manifest.md). The output is lowercase hex-encoded to a
64-character string matching `/^[0-9a-f]{64}$/`. The randomness invariant is validated by
`test_BC_2_01_008_generate_session_token_is_random` which asserts two successive calls produce
distinct values (collision probability 2^-256).

`rand 0.9` is explicitly rejected: `OsRng` moved behind a feature flag in 0.9, which is an
ergonomic regression. The `=0.8.6` EXACT pin closes that regression permanently.

### Lock file mode 0o600 (BC-2.01.005 PC-3 / INV-3)

The lock file is written via `tempfile::NamedTempFile::persist()`. Mode `0o600` is set so only
the daemon owner can read the `authToken` field. Test assertion uses
`metadata.permissions().mode() & 0o777 == 0o600` (NOT `metadata.mode() == 0o600`, which would
include file-type bits and produce incorrect results).

Forbidden pattern `std::fs::write` is enforced at project level by semgrep rule
`ANTI-001`. Test helpers in both integration test files use `NamedTempFile + persist` for all
fixture writes, exercising the same atomic path as production code.

### Runtime directory mode 0o700 (BC-2.01.005 PC-8 / NFR-012)

`ensure_runtime_dir()` uses `DirBuilder::new().mode(0o700).recursive(true).create()` with
`std::os::unix::fs::DirBuilderExt` in scope. `std::fs::create_dir_all` is forbidden (honors
umask, typically produces 0o755 which leaks the directory to other OS users). Test assertion
uses `metadata.permissions().mode() & 0o777 == 0o700`.

### Atomic tempfile::persist (BC-2.01.005 PC-3)

Atomicity is enforced at the write level: the lock file path receives data via POSIX `rename(2)`
semantics inside `tempfile::persist`. No partial lock file is observable to concurrent readers.
The test helper `write_test_fixture_to` uses the same pattern for fixture data, ensuring test
setup paths are also atomic (required by the project-wide semgrep ANTI-001 ban on `std::fs::write`).

---

## Error Path Coverage

| Error path | Test | Verdict |
|------------|------|---------|
| Live PID conflict (E-LOCK-001) | `test_BC_2_01_005_live_pid_conflict_returns_error` | PASS |
| Stale lock with dead PID (E-LOCK-002) | `test_BC_2_01_005_stale_pid_cleaned_up` | PASS |
| Unknown `contract_version` (E-LOCK-003) | `test_BC_2_01_010_unknown_contract_version_treated_as_stale` | PASS |
| Missing `contract_version` key | `test_BC_2_01_010_missing_contract_version_treated_as_stale`, `test_BC_2_01_010_contract_version_key_absent_entirely_treated_as_stale` | PASS |
| String-typed `contract_version` | `test_BC_2_01_010_string_contract_version_handled_gracefully` | PASS |
| `RuntimeDirUnresolvable` (no HOME) | `test_BC_2_01_005_runtimedirunresolvable_when_no_home` | PASS |
| Port 0 rejection (F-S006-ADV1-004) | `test_BC_2_01_005_acquire_rejects_port_zero` | PASS |
| Empty `MONOCLE_RUNTIME_DIR` fallthrough (EC-060) | `test_BC_2_01_005_env_override_empty_string_falls_through` | PASS |

---

## Behavioral Contract Traceability

| BC | Tests |
|----|-------|
| BC-2.01.005 (Lock File Atomic Lifecycle) | 19 tests in `lock_file_lifecycle.rs` |
| BC-2.01.008 (Auth Token Generation) | 4 tests in `lock_file_lifecycle.rs` |
| BC-2.01.010 (Contract Version Field) | 7 tests in `lock_file_contract.rs` |

VP-005 (lock file lifecycle) and VP-010 (contract version field) are both covered per their
respective test headers.

---

## Note on AC-008 (macOS platform fallback)

AC-008 requires that on macOS, `resolve_runtime_dir()` falls back to `data_local_dir()` when
`runtime_dir()` returns `None`. This platform is macOS (Darwin 25.5.0), which is the exact
platform where the fallback applies. The `test_BC_2_01_005_env_override_empty_string_falls_through`
test exercises this path: with `MONOCLE_RUNTIME_DIR=""` and `HOME` set, it falls through to
`ProjectDirs` and returns a valid data-local path. The production `resolve_runtime_dir()` code
handles the `None` case from `runtime_dir()` by falling back to `data_local_dir()`, satisfying
AC-008. No isolated AC-008 unit test exists in the integration suite because the fallback is
exercised by the environment in which all tests run.
