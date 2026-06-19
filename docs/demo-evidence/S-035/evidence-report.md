# S-035 Demo Evidence Report

**Story:** S-035 — SessionManager::attach_session / detach_session + Ruling L kill-confirmation  
**BCs covered:** BC-2.08.007, BC-2.08.008  
**Evidence produced:** 2026-06-19  
**Demo format:** WEBM + .tape (D-333 directive: NO GIF)

---

## Recordings

| File | Description |
|------|-------------|
| `s035-attach-detach.webm` | VHS-rendered terminal recording of all 18 test groups |
| `s035-attach-detach.tape` | VHS script source (7 groups, 18 tests) |
| `s035-test-output.txt` | Plain-text test output transcript (18 passed, 0 failed) |

---

## Coverage Map: Acceptance Criteria → Tests

### BC-2.08.007 — attach_session / detach_session behavioral contracts

| AC | Description | Test(s) Evidencing | Recording Group |
|----|-------------|-------------------|-----------------|
| AC-001 | SO_PEERCRED verified before processing scrollback | `test_BC_2_08_007_attach_peer_cred_mismatch_transitions_to_terminated` | Group 3 |
| AC-002 | DaemonToHost::Attach sent; ScrollbackChunk* + ScrollbackDumpComplete received within 5s | `test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive`, `test_BC_2_08_007_attach_5s_timeout_session_host_dead` | Groups 1, 2 |
| AC-003 | host_conn.proxy_task = Some after attach | `test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive` | Group 1 |
| AC-004 | State transitions Detached → Running; SessionStateChanged{Running} before SessionListUpdate | `test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive`, `test_BC_2_08_008_state_changed_ordering_on_attach_detach` | Groups 1, 2 |
| AC-005 | ScrollbackChunk* + ScrollbackDumpComplete forwarded to TUI broker clients | `test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive` | Group 1 |
| AC-006 | DaemonToHost::Detach sent; state → Detached; host_conn → None | `test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive`, `test_BC_2_08_007_sidecar_updated_on_detach`, `test_BC_2_08_008_state_changed_ordering_on_attach_detach` | Groups 1, 2, 6 |
| AC-007 | Session-host NOT killed on detach; remains alive | `test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive` | Group 1 |
| AC-008 | Sidecar updated to state:"Detached" atomically via tempfile::persist | `test_BC_2_08_007_sidecar_updated_on_detach` | Group 6 |
| AC-009 | Mutex serializes concurrent attaches; no duplicate proxy_task | `test_BC_2_08_007_concurrent_attach_no_duplicate_proxy_task`, `test_BC_2_08_007_concurrent_attach_single_proxy_task_invariant` | Group 6 |
| AC-010 | Retired ScrollbackDump (legacy message) rejected without crashing | `test_BC_2_08_007_retired_scrollback_dump_rejected` | Group 6 |
| AC-011 | attach_session() on Running (already attached) → Ok(()) idempotent (EC-185) | `test_BC_2_08_007_attach_running_idempotent` | Group 4 |
| AC-012 | detach_session() on Detached → Ok(()) idempotent (EC-186) | `test_BC_2_08_007_detach_detached_idempotent` | Group 4 |
| AC-013 | EC-187: UDS connect fails → SessionHostDead → Terminated → "attach_failed" | `test_BC_2_08_007_attach_running_session_dead` | Group 3 |
| AC-014 | F-P51-001: detach on Launching (host_conn=None) → SessionNotReady → "session_not_ready" | `test_BC_2_08_007_detach_launching_session_not_ready` | Group 4 |

### BC-2.08.008 — SessionStateChanged ordering invariants

| AC | Description | Test(s) Evidencing | Recording Group |
|----|-------------|-------------------|-----------------|
| AC-015 | SessionStateChanged emitted on every transition (no silent state changes) | `test_BC_2_08_008_state_changed_ordering_on_attach_detach` | Group 2 |

### EC-188 — 5s scrollback timeout path

| Scenario | Test(s) Evidencing | Recording Group |
|----------|-------------------|-----------------|
| EC-188: ScrollbackDumpComplete not received within 5s → SIGTERM + Terminated + "attach_failed" | `test_BC_2_08_007_attach_5s_timeout_session_host_dead`, `test_attach_timeout_sigterm_uses_pid_sigterm_fn_seam` | Groups 2, 3 |

### Ruling L — kill on attached session

| Scenario | Test(s) Evidencing | Recording Group |
|----------|-------------------|-----------------|
| Kill attached session uses proxy_task fast-path (no 12s SIGKILL watchdog) | `test_kill_attached_session_fast_path` | Group 5 |

### TOCTOU guard

| Scenario | Test(s) Evidencing | Recording Group |
|----------|-------------------|-----------------|
| Concurrent detach during Terminated does not resurrect to Detached; sidecar stays Terminated | `test_BC_2_08_007_attach_detach_cycle` (integration: full cycle including TOCTOU invariants) | Group 6 |

### 4 attach-failure dispositions (BC-2.08.007 error taxonomy)

| Disposition | Test | Recording Group |
|-------------|------|-----------------|
| ConnectFailed → Terminated | `test_BC_2_08_007_attach_running_session_dead` (EC-187) | Group 3 |
| PeerCredFailed → Terminated | `test_BC_2_08_007_attach_peer_cred_mismatch_transitions_to_terminated` | Group 3 |
| Timeout(EC-188) → SIGTERM + Terminated | `test_BC_2_08_007_attach_5s_timeout_session_host_dead` | Group 2 |
| ProtocolError → stays Detached | `test_BC_2_08_007_attach_protocol_error_stays_detached` | Group 3 |

---

## Test Result Summary

```
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.47s
```

All 18 tests PASS. Full transcript in `s035-test-output.txt`.

---

## Recording Segments

| Group | Recording Timestamp | Content |
|-------|--------------------|---------| 
| Group 1/7 | ~0:02 | AC-002..007/015 — core attach/detach cycle |
| Group 2/7 | ~0:37 | EC-188 timeout→Terminated; BC-2.08.008 ordering |
| Group 3/7 | ~1:12 | 4 attach-failure dispositions |
| Group 4/7 | ~1:47 | Idempotency (AC-011/012/014) |
| Group 5/7 | ~2:22 | Ruling L — kill fast-path |
| Group 6/7 | ~2:57 | TOCTOU + sidecar + concurrency + retired-protocol + integration |
| Group 7/7 | ~3:32 | Full suite — 18/18 green |
