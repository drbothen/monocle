---
document_type: adversarial-pass
story: S-022
pass: 6
producer: vsdd-factory:adversary
timestamp: 2026-05-28T03:30:00Z
classification: HIGH_PRESENT
findings_count:
  blocker: 0
  high: 1
  medium: 1
  nitpick: 0
prior_pass_resolution:
  resolved: 0
  partial: 0
  not_fixed: 0
  phantom: 0
  over_corrected: 0
---

# S-022 Adversarial Pass 6

## Summary

Fresh-context re-derivation of BC-2.05.002 + BC-2.05.005 compliance from scratch on the unchanged 18-commit branch. Pass 5's "clean" verdict was PREMATURE. Identified two substantive issues: (1) slow-client soft-disconnect leaves per-client task running because the per-client tx clone keeps rx alive (BC-2.05.004 EC-005 violation); (2) race window between subscriber registration and snapshot causes duplicate delivery of PermissionPromptQueued (BC-2.05.002 EC-005 vs AC-006 inherent contradiction).

## Part A — Pass 5 Resolution Verification

Pass 5 reported zero findings — nothing to verify.

## Part B — NEW Pass 6 Findings

### F-S022-ADV6-HIGH-001 — broadcast_to_subscribers removes from list but does NOT close per-client connection
**Severity:** HIGH (CLAUDE.md Principle 1 violation).
**Location:** crates/monocle-runtime/src/ipc_server.rs:234-252 (broadcast helper) + ipc_server.rs:140-180 (per-client task continues after removal).
**Routing:** implementer.

**Trace:**
1. Slow client A registered (tx_a in subscribers, per-client task holds local tx).
2. Channel saturates (CLIENT_CHANNEL_CAPACITY=64).
3. broadcast_to_subscribers calls try_send → Err(TrySendError::Full).
4. Code at line 241-244: logs WARN, drops sender from broadcast iteration. The Full(_) owned tx_a clone is dropped, but per-client task in spawn_client_task still holds its own tx (line 112) so rx is NOT closed.
5. Per-client task continues to run. Outbound channel no longer in subscribers; no future broadcasts reach it. But inbound read loop continues accepting ClientToServer::PermissionDecision messages.

**Spec violation:** BC-2.05.004 EC-005 mandates "removes slow client from fan-out subscriber list; closes the per-client connection; logs WARN: removed slow TUI client (send buffer full)". Implementation removes from list and logs WARN, but does NOT close connection.

**Impact:** Slow client silently desyncs. Can still send permission decisions; will never see updated state. TUI rendering drifts from daemon truth without visible error.

**Scope adjudication:** BC-2.05.004 not in S-022 anchored bcs. HOWEVER, S-022 OWNS broadcast_to_subscribers — shipping the helper with semantic gap violates CLAUDE.md Principle 1.

**Required fix:** Signal connection closure to per-client task on TrySendError::Full. Options: (a) store JoinHandle pairs on subscribers list, abort on slow-disconnect; (b) use parallel tokio::sync::Notify per client; (c) restructure to drop the per-client task's tx clone.

**Test gap:** ipc_broadcast.rs:36-182 asserts the WRONG postcondition (list removal not connection closure).

### F-S022-ADV6-MED-001 — Duplicate PermissionPromptQueued window between register_subscriber and InitialState snapshot
**Severity:** MEDIUM. **Routing:** architect (BC contradiction) OR implementer (snapshot-epoch dedup).
**Location:** ipc_server.rs:114-122 (register subscriber → atomic increment → take snapshot → send InitialState).

**Trace:**
1. register_subscriber adds new client's tx (line 116).
2. Before snapshot_initial_state (line 207), another task triggers PermissionPromptQueued broadcast. New prompt enters BOTH the new client's mpsc channel AND the pending_decisions registry.
3. snapshot_initial_state runs; overlay_stack includes new prompt via pending_decisions.snapshot_payloads().
4. send_initial_state writes InitialState { overlay_stack: [..., new_prompt] }.
5. Per-client send loop drains rx, writes queued PermissionPromptQueued { payload }.

TUI receives prompt TWICE.

**Spec contradiction:** BC-2.05.002 AC-006 requires registering subscribers BEFORE snapshot (no gap). BC-2.05.002 EC-005 forbids duplicate events. Both cannot be true with current registration-before-snapshot ordering under concurrent state mutations.

**Required fix:** Either (a) BC-2.05.002 needs explicit dedup semantics (architect/PO), OR (b) spawn_client_task tracks snapshot epoch (sequence number for pending_decisions registry) and discards streaming PermissionPromptQueued messages that arrived before snapshot epoch.

## Process-Gap Findings

None. Both findings content-defect routed.

## Conclusion

Convergence: passes_clean_consecutive=0 (reset from 1). last_classification=HIGH_PRESENT. converged=false. Earliest convergence now Pass 9.

Pass 5's clean verdict was premature: the slow-client soft-disconnect bug is grep-visible in ipc_server.rs:234-252 and the BC EC-005 violation is an obvious mismatch — but Pass 5 read only the broadcast_to_subscribers body without tracing per-client task lifecycle interaction. Fresh context surfaced it.

Recommend: dispatch implementer for HIGH-001 + architect for MED-001 BC contradiction. Do not run Pass 7 until both fixed.
