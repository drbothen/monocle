# S-033 Demo Evidence Report

**Story:** S-033 — `SessionManager::spawn_session`, SessionHostSpawner, SessionEntry, Sidecar, SpawnAck, and SessionStateChanged{Launching}
**Story Version:** v1.9
**Story Points:** 8 (Wave 8, EPIC-08)
**BCs:** BC-2.03.008, BC-2.08.001, BC-2.08.008
**Evidence Date:** 2026-06-17
**Worktree:** `.worktrees/S-033`

---

## Demo Modality

S-033 is a daemon/IPC/session-spawning **backend story** with no TUI surface. Demos are VHS terminal recordings driven by:
- Unit tests in `crates/monocle-runtime/src/session_manager/mod.rs` (47 tests, BC-named)
- Integration tests in `crates/monocle-runtime/tests/s033_blocker_red_gate.rs` (14 tests, production-wiring)

All recordings show `test result: ok` for every test batch. No test was skipped or expected-to-fail.

---

## Recordings

### AC-001-005-012-spawn-session-happy-path (.gif / .webm / .tape)

**Evidences:** AC-001, AC-002, AC-003, AC-004, AC-005, AC-012

| Recording segment | Tests run | AC covered |
|-------------------|-----------|------------|
| Unit tests (spawn happy path) | `test_BC_2_08_001_spawn_session_entry_created_within_2s`, `test_BC_2_08_001_ipc_arm_spawn_ack_precedes_state_changed_launching`, `test_BC_2_08_001_sidecar_written_with_schema_v3`, `test_BC_2_08_001_spawn_session_returns_ok_without_waiting_for_running`, `test_BC_2_08_008_session_state_changed_before_session_list_update_on_spawn`, `test_BC_2_08_008_spawn_ack_before_state_changed_launching`, `test_BC_2_08_001_spawn_session_entry_fields_correct_on_launch` | AC-001, AC-002, AC-003, AC-004, AC-005, AC-012 |
| MED-001 real binary | `test_BC_2_08_001_MED001_real_session_host_reaches_running` | AC-004 (async Running transition via real monocle-session-host binary) |
| B-002 production broker | `test_BC_2_08_001_B002_production_broker_receives_state_changed` | AC-005, AC-012 (daemon_start_sequence() wired; broadcast reaches subscriber registered on real ipc_subscribers) |

**What the recording shows:**
- `spawn_session()` called; `SpawnAck` sent to client BEFORE `spawn_session()` (AC-001, AC-012)
- `SessionEntry{state: Launching, host_conn: None}` appears in registry immediately (AC-002)
- `session-state.json` written atomically via `tempfile::persist` with `schema_version: 3` (AC-003)
- `spawn_session()` returns `Ok(session_id)` before session-host reaches Running (AC-004)
- `SessionStateChanged{Launching}` broadcast BEFORE `SessionListUpdate` (AC-005)
- Real `monocle-session-host` binary launches, logs `sent StateChanged{Running}`, post-spawn monitor transitions to Running (MED-001)

---

### AC-008-009-error-paths (.gif / .webm / .tape)

**Evidences:** AC-008, AC-009, AC-009b, AC-009c, AC-009d

| Recording segment | Tests run | AC covered |
|-------------------|-----------|------------|
| BinaryNotFound propagation | `test_BC_2_08_001_binary_not_found_propagation`, `test_BC_2_08_001_binary_not_found_ipc_code_is_binary_not_found`, `test_BC_2_08_001_session_error_to_code_binary_not_found_maps_correctly` | AC-008 |
| Sidecar fail + orphan-kill | `test_BC_2_08_001_sidecar_write_fail_orphan_kill` | AC-009 |
| SpawnFailed (OS error) | `test_BC_2_08_001_spawn_failed_os_error_returns_spawn_failed_code`, `test_BC_2_08_001_spawn_failed_ipc_code_is_spawn_failed` | AC-009b |
| UnsupportedOperation -> spawn_unsupported | `test_BC_2_03_008_default_spawn_recipe_unsupported_operation`, `test_BC_2_03_008_EC_112_unsupported_operation_maps_to_spawn_unsupported`, `test_BC_2_03_008_spawn_session_unsupported_engine_returns_engine_error` | AC-009c, AC-009d |

**What the recording shows:**
- `spawn_recipe()` returns `EngineError::BinaryNotFound` → no OS process spawned; no registry entry; IPC code `"binary_not_found"` (AC-008, EC-150)
- Spawner succeeds but sidecar write fails → SIGTERM → 2s → SIGKILL to orphan; `Err(SidecarWriteFailed)`; no registry entry (AC-009, EC-151)
- `spawner.spawn()` returns OS error → `Err(SpawnFailed)`; no sidecar; no registry entry; IPC code `"spawn_failed"` (AC-009b)
- Default `spawn_recipe()` impl returns `Err(UnsupportedOperation("spawn_recipe"))` (AC-009c)
- `session_error_to_code(IpcOp::Spawn, EngineError::UnsupportedOperation)` returns `"spawn_unsupported"` NOT `"invalid_request"` (AC-009d, EC-112)

---

### AC-006-007-010-011-invariants (.gif / .webm / .tape)

**Evidences:** AC-006, AC-007, AC-010, AC-011; also B-001, B-003, B-004, B-005, MED-002, MED-003, MED-004

| Recording segment | Tests run | AC covered |
|-------------------|-----------|------------|
| UUID collision retry | `test_BC_2_08_001_EC_152_ipc_handler_regenerates_on_first_collision_succeeds`, `test_BC_2_08_001_EC_152_ipc_handler_second_collision_sends_error`, `test_BC_2_08_001_spawn_returns_err_collision_when_id_already_exists`, `test_BC_2_08_001_ipc_handler_two_attempt_retry_on_collision` | AC-006, EC-152 |
| Atomic sidecar write | `test_BC_2_08_001_invariant_sidecar_write_is_atomic`, `test_BC_2_08_001_invariant_partial_write_detector_catches_truncation` | AC-007 |
| StateChanged broadcast | `test_BC_2_08_008_session_state_changed_broadcast_to_all_clients`, `test_BC_2_08_008_session_state_changed_new_state_is_launching_on_spawn`, `test_BC_2_08_008_no_clients_connected_broadcast_discarded_no_error`, `test_BC_2_08_008_rename_session_does_not_emit_session_state_changed` | AC-010, AC-011 |
| Full red gate suite | All 14 tests in `s033_blocker_red_gate.rs` (B-001, B-002a, B-002b, B-003, B-003b, B-004, B-005, HIGH-001, HIGH-002, HIGH-003, MED-001, MED-002, MED-003, MED-004) | All ACs (production wiring) |

**What the recording shows:**
- `spawn_session()` returns `Err(SessionIdCollision)` on collision; IPC handler retries once with fresh UUID and second `SpawnAck`; second consecutive collision surfaces `"session_id_collision"` to client (AC-006, EC-152)
- Concurrent reader of sidecar path never sees partial write (AC-007, `tempfile::persist` atomic rename)
- `SessionStateChanged` emitted for every state transition; rename does NOT emit `SessionStateChanged`; no clients → discard with no error (AC-010, AC-011)
- `daemon_start_sequence()` wires `session_manager = Some(...)` (B-001 production wiring)
- SO_PEERCRED UID mismatch → `SessionStateChanged{Terminated}` broadcast + sidecar GC'd (B-003, EC-163)
- Daemon-owned fields (`project_root`, `harness_id`, `profile_id`, `started_at`) survive session-host step-8 sidecar clobber (B-005)
- `host_conn = Some(writer)` stored after Running transition (HIGH-001)
- Missing session-host binary maps to `SessionError::SpawnFailed` NOT `BinaryNotFound` (HIGH-002)
- Sidecar re-persisted with `state: Running` on Running transition (HIGH-003)
- Collision retry uses injectable UUID seam (MED-002)
- Running pair `(SessionStateChanged{Running}, SessionListUpdate)` emitted under single mutex acquisition — no interleaving (MED-003, Ruling G)
- `degraded_env: Some(_)` in `StateChanged{Running}` sets `SessionEntry.degraded = true` (MED-004)

---

## Coverage Summary

| AC | Description | Recording | Status |
|----|-------------|-----------|--------|
| AC-001 | SpawnAck before spawn_session(); spawn_recipe() first; within 2s | AC-001-005-012-spawn-session-happy-path | EVIDENCED |
| AC-002 | SessionEntry{Launching} in registry | AC-001-005-012-spawn-session-happy-path | EVIDENCED |
| AC-003 | Sidecar atomic write, schema_version 3 | AC-001-005-012-spawn-session-happy-path | EVIDENCED |
| AC-004 | spawn_session() Ok without waiting for Running | AC-001-005-012-spawn-session-happy-path | EVIDENCED |
| AC-005 | SessionStateChanged{Launching} before SessionListUpdate | AC-001-005-012-spawn-session-happy-path | EVIDENCED |
| AC-006 | UUID collision -> Err(SessionIdCollision); IPC handler retries once | AC-006-007-010-011-invariants | EVIDENCED |
| AC-007 | tempfile::persist atomic; no partial writes observable | AC-006-007-010-011-invariants | EVIDENCED |
| AC-008 | BinaryNotFound propagation; no process; IPC code "binary_not_found" | AC-008-009-error-paths | EVIDENCED |
| AC-009 | Sidecar fail -> orphan-kill; Err(SidecarWriteFailed); no registry entry | AC-008-009-error-paths | EVIDENCED |
| AC-009b | SpawnFailed (OS error from spawner); IPC code "spawn_failed" | AC-008-009-error-paths | EVIDENCED |
| AC-009c | Default spawn_recipe() returns UnsupportedOperation | AC-008-009-error-paths | EVIDENCED |
| AC-009d | UnsupportedOperation maps to "spawn_unsupported" (not "invalid_request") | AC-008-009-error-paths | EVIDENCED |
| AC-010 | SessionStateChanged on every transition; no silent transitions | AC-006-007-010-011-invariants | EVIDENCED |
| AC-011 | Broadcast to all TUI clients; no clients -> discard no error | AC-006-007-010-011-invariants | EVIDENCED |
| AC-012 | SpawnAck before SessionStateChanged{Launching} (causal ordering) | AC-001-005-012-spawn-session-happy-path | EVIDENCED |

**All 12 acceptance criteria evidenced. No AC gaps.**

---

## Reproducibility

```bash
# From the worktree root:
cd /Users/jmagady/Dev/monocle/.worktrees/S-033

# Re-record all tapes (requires vhs on PATH):
cd docs/demo-evidence/S-033
vhs AC-001-005-012-spawn-session-happy-path.tape
vhs AC-008-009-error-paths.tape
vhs AC-006-007-010-011-invariants.tape

# Run just the tests (no recording):
cargo test -p monocle-runtime -- test_BC_2 --test-threads=1
cargo test -p monocle-runtime --test s033_blocker_red_gate -- --test-threads=1
```

**Test counts confirmed before recording:**
- Unit tests (session_manager): 47 tests, 0 failures
- Integration tests (s033_blocker_red_gate): 14 tests, 0 failures
- Total S-033-specific tests: 61
