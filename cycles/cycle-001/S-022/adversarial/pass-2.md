---
document_type: adversarial-pass
story: S-022
pass: 2
producer: vsdd-factory:adversary
timestamp: 2026-05-27T23:30:00Z
classification: BLOCKER_PRESENT
findings_count:
  blocker: 0
  high: 3
  medium: 4
  nitpick: 0
prior_pass_resolution:
  resolved: 10
  partial: 1
  not_fixed: 1
  phantom: 1
  over_corrected: 0
---

# S-022 Adversarial Pass 2

## Part A — Pass 1 Resolution Verification

| Pass 1 Finding | Verdict | Notes |
|---|---|---|
| F-S022-ADV1-BLK-001 (PreToolUse Defer not wired) | RESOLVED | pre_tool_use.rs:223-329 full wiring |
| F-S022-ADV1-BLK-002 (pending_decisions uninitialized) | RESOLVED | lifecycle.rs:608 |
| F-S022-ADV1-BLK-003 (run_accept_loop not spawned) | PARTIAL | spawned but JoinHandle dropped — see HIGH-001 |
| F-S022-ADV1-BLK-004 (dead UdsTransport::accept_loop) | RESOLVED | deleted; bind() returns (Self, UnixListener) |
| F-S022-ADV1-BLK-005 (ring_tail empty + defer comment) | RESOLVED with new defect | hook_event_record_to_hook_event silently fabricates empty strings — see HIGH-002 |
| F-S022-ADV1-HIGH-001 (sessions empty) | RESOLVED | snapshot_enriched_sessions in hooks/mod.rs:158-177 |
| F-S022-ADV1-HIGH-002 (slow-client cleanup) | RESOLVED | broadcast_to_subscribers drain-and-retain |
| F-S022-ADV1-HIGH-003 (input-hash [pending]) | RESOLVED | story frontmatter: input-hash 931a19a |
| F-S022-ADV1-MED-001 (stale accept_loop doc) | RESOLVED | doc redirects to monocle-runtime::ipc_server |
| F-S022-ADV1-MED-002 (TuiTransport direction) | PHANTOM | wrapper is named UdsClientTransport; behavior is correct but name mismatch |
| F-S022-ADV1-MED-003 (single-giant-commit) | NOT-FIXED | Round 2 also single squashed commit b50252e |
| F-S022-ADV1-MED-004 (misleading prompt_id arg) | RESOLVED | API now overwrites payload.prompt_id |
| F-S022-ADV1-NITPICK-001 (register_prompt doc) | RESOLVED | |

Counts: RESOLVED 10 / PARTIAL 1 / NOT-FIXED 1 / PHANTOM 1 / OVER-CORRECTED 0.

## Part B — NEW Pass 2 Findings

### F-S022-ADV2-HIGH-001 — Accept-loop JoinHandle dropped; no shutdown_rx; socket-removal race
**Severity:** HIGH. **AC:** AC-001 (BC-2.05.002 PC-1 lifecycle). **Location:** lifecycle.rs:615-618.
tokio::spawn JoinHandle dropped. Graceful shutdown cannot abort listener. In-flight accept races socket removal. **Fix:** Store JoinHandle on DaemonState and .abort() on shutdown, OR thread shutdown_rx into run_accept_loop with tokio::select!. **Routing:** implementer.

### F-S022-ADV2-HIGH-002 — hook_event_record_to_hook_event silently fabricates empty strings; BC-2.05.002 PC-2 fidelity violation
**Severity:** HIGH. **AC:** AC-002. **Location:** state.rs:440-487.
Reconstruction silently replaces cwd, transcript_path, prompt, stop_reason, notification_type, message with "". No WARN emitted. **Fix (architect-route):** (a) extend HookEventRecord with missing fields; OR (b) change InitialState.ring_tail to Vec<HookEventRecord>, push reconstruction to TUI side (requires BC-2.05.002 update). **Routing:** architect.

### F-S022-ADV2-HIGH-003 — tui_attached flag never set on TUI connect/disconnect; BC-2.01.002 PC-1 violation
**Severity:** HIGH. **AC:** observable contract. **Location:** state.rs:150 (flag exists), ipc_server.rs:78-153 (per-client task never writes flag).
/status permanently reports tui_attached: false despite live S-022 wiring. **Fix:** In spawn_client_task: after register_subscriber, store true. After remove on disconnect, check subscriber list emptiness and conditionally clear. Use AtomicUsize count for race safety. **Routing:** implementer.

### F-S022-ADV2-MED-001 — register_prompt API silently overwrites caller-supplied prompt_id
**Severity:** MEDIUM. **Location:** permissions.rs:101-118. **Routing:** implementer.
Leaky API: accepts PermissionPromptPayload whose prompt_id field is ignored. Fix: take PromptPayloadInputs (without prompt_id), return (Uuid, full payload).

### F-S022-ADV2-MED-002 — Dead UdsTransport::broadcast_session_list_update and broadcast_hook_event_received
**Severity:** MEDIUM. **Location:** uds.rs:166-260. **Routing:** implementer.
Methods exist with 256 KiB guard but are unreachable from production (lifecycle uses tokio::net::UnixListener directly, not UdsTransport). Future SessionListUpdate broadcast wiring would bypass the guard. **Fix:** Delete OR route all broadcasts through these methods.

### F-S022-ADV2-MED-003 — monocle-core new() constructors landed without architect adjudication
**Severity:** MEDIUM. **Location:** monocle-core/src/hook_events.rs:85-220. **Routing:** architect.
Adding pub fn new(...) to #[non_exhaustive] structs partially undermines additivity guarantee. ADV-W5GATE-MED-003 was the explicit OPEN follow-up; S-022 implementer landed without architect routing. **Fix:** Architect ratifies; either (a) document breaking-change discipline OR (b) replace new() with Builder pattern. **[process-gap]:** routing-table breach.

### F-S022-ADV2-MED-004 — Double-serialize in send_initial_state
**Severity:** MEDIUM. **Location:** ipc_server.rs:170-190. **Routing:** implementer.
serde_json::to_vec called twice per send. Fix: compute size from serialized vec, write the same vec with 4-byte LE prefix.

## Process-Gap Findings

- [process-gap] F-S022-ADV1-MED-003 single-giant-commit recurred in Round 2 (b50252e)
- [process-gap] F-S022-ADV2-MED-003 routing-table breach: monocle-core ABI change made by implementer without architect routing

## Novelty Assessment
**Novelty: HIGH.** Three genuine NEW HIGH findings + 4 MEDIUM findings, none overlapping with Pass 1. Substantive new pass.

## Conclusion
Convergence: passes_clean_consecutive=0, converged=false. Dispatch implementer for HIGH-001 + HIGH-003 + MEDs; architect for HIGH-002 + MED-003. Re-run Pass 3.
