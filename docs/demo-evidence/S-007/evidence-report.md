# Demo Evidence Report — S-007 Crash Recovery Checkpoint

**Story:** S-007 — Crash Recovery Checkpoint (BC-2.01.006)
**Evidence Type:** Integration test run output (library-level story — no TUI/binary entrypoint)
**Date:** 2026-05-26

## Evidence Rationale

S-007 implements library-level functions (`write_recovery_checkpoint`, `read_recovery_checkpoint`) 
in `monocle-runtime`. The daemon-level wiring (UDS dispatch, startup detection, 60s window, 
banner text) is explicitly deferred to the `S-005-main-wiring` integration story per the story spec.

For a library crate with no runnable binary entry point, the functional evidence is the 
integration test suite passing. VHS/Playwright demo recording is not applicable at this 
library layer.

## AC Coverage

| AC | Description | Evidence | Status |
|----|-------------|----------|--------|
| AC-008 | Recovery file schema (4 fields + validation) | `test_schema_validation_*`, `test_serde_roundtrip` | PASS |
| AC-009 | Write during drain, before lock removal (atomic) | `test_write_creates_valid_checkpoint`, permissions test | PASS |
| AC-010 | EC-054 malformed file handling | `test_read_malformed_returns_malformed`, `test_read_absent_returns_absent` | PASS |
| AC-001 thru AC-007 | Daemon startup / UDS dispatch / 60s window | Deferred to S-005-main-wiring integration story | DEFERRED |

## Test Run Evidence

```
running 15 tests
test test_write_creates_valid_checkpoint ... ok
test test_write_creates_file_with_correct_permissions ... ok
test test_serde_roundtrip ... ok
test test_schema_validation_pid_zero_rejected ... ok
test test_schema_validation_empty_app_mode_rejected ... ok
test test_schema_validation_invalid_shutdown_reason ... ok
test test_schema_validation_invalid_timestamp_format ... ok
test test_read_valid_checkpoint ... ok
test test_read_malformed_returns_malformed ... ok
test test_read_absent_returns_absent ... ok
test test_overwrite_existing_checkpoint ... ok
test test_shutdown_utc_format_matches_vp006_regex ... ok
test test_chrono_timestamp_precision ... ok
test test_debug_derive_on_types ... ok
test test_negative_path_all_validations ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Workspace-wide: 340+ tests pass, 0 regressions, clippy clean.
