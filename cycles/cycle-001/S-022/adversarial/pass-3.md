---
document_type: adversarial-pass
story: S-022
pass: 3
producer: vsdd-factory:adversary
timestamp: 2026-05-28T00:00:00Z
classification: BLOCKER_PRESENT
findings_count:
  blocker: 0
  high: 2
  medium: 3
  nitpick: 3
prior_pass_resolution:
  resolved: 18
  partial: 0
  not_fixed: 0
  phantom: 1
  over_corrected: 0
  resolved_with_new_defect: 1
---

# S-022 Adversarial Pass 3

## Summary

Fresh-context review of the 11-commit branch state. Production-grade lens (CLAUDE.md) applied. All 20 prior findings verified against current code. No new BLOCKERs in the strict-classification sense; under CLAUDE.md production-grade lens the two HIGHs are severity-promoted to BLOCKER because BC-2.05.001 EC-002 is silently bypassed in production.

## Part A — Pass 1 + Pass 2 Resolution Verification

### Pass 1 (13 findings) — all RESOLVED except F-S022-ADV1-MED-002 (PHANTOM, confirmed)

### Pass 2 (7 findings) — all RESOLVED; F-S022-ADV2-MED-002 RESOLVED_WITH_NEW_DEFECT (sibling-scope recurrence — see Pass 3 HIGH-002)

Counts: RESOLVED 18 / RESOLVED_WITH_NEW_DEFECT 1 / PHANTOM 1 / PARTIAL 0 / NOT-FIXED 0 / OVER-CORRECTED 0.

## Part B — NEW Pass 3 Findings

### F-S022-ADV3-HIGH-001 — BC-2.05.001 EC-002 path-length guard bypassed in production
**Severity:** HIGH (production-grade BLOCKER). **AC:** BC-2.05.001 EC-002. **Location:** lifecycle.rs:510-571 vs uds.rs:102-142. **Routing:** implementer.
Production daemon_start_sequence step 10 inlines the UDS bind logic and omits the UDS_PATH_LIMIT_BYTES check that UdsTransport::bind enforces. Deep XDG runtime paths or custom MONOCLE_RUNTIME_DIR > ~92 bytes silently fail with cryptic ENAMETOOLONG instead of the production-grade IpcError::PathTooLong with explicit ERROR log. The test test_BC_2_05_001_uds_path_too_long_rejected_with_pathtoolong_error verifies UdsTransport::bind directly — but production doesn't call it. Green-but-disconnected coverage.
**Required fix:** route production bind through UdsTransport::bind OR replicate path-length check inline before UnixListener::bind + add integration test exercising via daemon_start_sequence.

### F-S022-ADV3-HIGH-002 — UdsTransport public surface dead in production (sibling-scope partial-fix recurrence)
**Severity:** HIGH. **Location:** uds.rs:69-175. **Routing:** implementer (architect signoff on deletion scope).
Pass 2 MED-002 deleted broadcast_session_list_update + broadcast_hook_event_received. Sibling members still dead: UdsTransport::add_subscriber, subscriber_count, subscribers field, cleanup (only test-called), bind (only test-called; production duplicates). UdsTransport is essentially a test fixture in disguise. Future S-025/S-026 implementers could wire state.uds_transport.add_subscriber and create a divergent subscriber list with no fan-out path to production broadcast_to_subscribers.
**Required fix:** route production through UdsTransport::bind, store transport on DaemonState, use UdsTransport::cleanup in shutdown path. OR delete UdsTransport entirely (keep UdsClientTransport per-client handle) and move bind to a free function.
**[process-gap]:** CLAUDE.md S-7.01 partial-fix discipline — Pass 2 MED-002 fix didn't propagate to sibling dead members in same struct.

### F-S022-ADV3-MED-001 — Stale "Red Gate" / "Hits todo!()" comments across passing tests
**Severity:** MEDIUM. **Location:** permission_prompt.rs (15 instances), connection_handshake.rs (17 instances), transport_uds.rs (6 instances). **Routing:** implementer.
Test docstrings still claim Red Gate behavior; common::spawn_test_daemon is fully implemented. Doc-string sweep needed.

### F-S022-ADV3-MED-002 — daemon_start_sequence docstring claims accept-loop JoinHandle returned
**Severity:** MEDIUM. **Location:** lifecycle.rs:344-351. **Routing:** implementer.
"and the spawned UDS accept loop join handle" — but DaemonState has no such field. Functionally OK (shutdown_rx terminates cleanly) but misleading for S-029 SOQ-2 implementer.
**Required fix:** change to "the spawned UDS accept loop terminates via state.shutdown_rx watch channel."

### F-S022-ADV3-MED-003 — broadcast_to_subscribers slow-client drain-and-retain has zero integration test coverage
**Severity:** MEDIUM. **Location:** fan_out.rs:14-16 + ipc_server.rs:234-252. **Routing:** implementer.
Pass 2 MED-002 deleted UdsTransport::fan_out_message tests but never wrote equivalent coverage in monocle-runtime/tests/. The drain-and-retain branch is dead-coverage; only the 256 KiB path is exercised by ac_004.
**Required fix:** write integration test in monocle-runtime/tests/ipc_broadcast.rs that saturates one client's mpsc channel, triggers broadcast, asserts slow client removed + WARN log line.

### F-S022-ADV3-NITPICK-001 — connection_handshake.rs:208 references HookEvent records
**Severity:** NITPICK. **Routing:** implementer.
Docstring should reference PermissionPromptPayload (inflating overlay_stack, not ring_tail).

### F-S022-ADV3-NITPICK-002 — types.rs:127 references abandoned reconstruction path
**Severity:** NITPICK. **Routing:** implementer.
Add "(previous attempt; superseded by ADR-0006)" or rephrase.

### F-S022-ADV3-NITPICK-003 — Arc<Mutex<Option<Uuid>>> overweight in pre_tool_use
**Severity:** NITPICK. **Routing:** implementer.
Cell<Option<Uuid>> would suffice; Mutex chosen for Send-across-await safety. Worth a comment.

## Process-Gap Findings
- [process-gap] F-S022-ADV3-HIGH-002 + F-S022-ADV3-MED-003: sibling-scope partial-fix regression (CLAUDE.md S-7.01)
- [process-gap] F-S022-ADV3-MED-001: TDD Red Gate docstring sweep should be standard implementer-phase mechanical task

## Novelty Assessment
**Novelty: MEDIUM.** Decay trajectory monotonically improving by severity (13 -> 7 -> 8). Two HIGHs are tightly coupled. Estimated 1-2 hours for Round 5 to close all.

## Conclusion
passes_clean_consecutive=0, converged=false. Recommend implementer Round 5 to close HIGH-001 + HIGH-002 + MEDs + NITPICKs. Pass 4 likely converges to NITPICK_ONLY.
