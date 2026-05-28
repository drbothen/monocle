---
document_type: adversarial-pass
story: S-022
pass: 1
producer: vsdd-factory:adversary
timestamp: 2026-05-27T22:45:00Z
classification: BLOCKER_PRESENT
findings_count:
  blocker: 5
  high: 3
  medium: 4
  nitpick: 1
---

# S-022 Adversarial Pass 1

## Summary

Reviewed S-022 worktree diff (3 commits since develop: 1b91643 stubs, eab7a14 failing tests, fb436a3 Round 1 impl), story spec, BC-2.05.002 v1.0.3 + BC-2.05.005 v1.6.0. Multiple BLOCKER-level production-grade violations centered on a single root cause: the implementation is "in-test-harness-only." Zero production wiring exists — `run_accept_loop` is never called from `daemon_start_sequence`, `DaemonState.pending_decisions` is never initialized in production, the PreToolUse Defer path is unchanged from S-018 with a fresh "S-022 TODO" comment, and `UdsTransport::accept_loop` is now dead code. Tests pass green because the test harness manually initializes what production never will.

## Findings

### F-S022-ADV1-BLK-001 — PreToolUse Defer path is NOT wired; S-022 TODO remains
**Severity:** BLOCKER. **AC:** AC-007..AC-011. **Location:** crates/monocle-runtime/src/hooks/pre_tool_use.rs:179-207. **Routing:** implementer.
Defer branch still contains placeholder oneshot + "S-022 TODO" comment. Tests pass via direct subscriber injection — production path unwired.

### F-S022-ADV1-BLK-002 — DaemonState.pending_decisions never initialized in production
**Severity:** BLOCKER. **AC:** AC-007, AC-009, AC-010, AC-011, AC-015. **Location:** state.rs:312 (None), lifecycle.rs:592-596. **Routing:** implementer.
Every PermissionDecision arriving on UDS would be silently dropped.

### F-S022-ADV1-BLK-003 — run_accept_loop never spawned by daemon
**Severity:** BLOCKER. **AC:** AC-001, AC-002, AC-005. **Location:** lifecycle.rs:593-597. **Routing:** implementer.
"S-022 TODO" comment marker. No TUI can connect to a real daemon.

### F-S022-ADV1-BLK-004 — UdsTransport::accept_loop is dead code AND missing InitialState
**Severity:** BLOCKER. **AC:** AC-002, BC-2.05.002 invariant 1. **Location:** uds.rs:240-265. **Routing:** implementer (architect signoff on delete-vs-relocate).
If ever called, would hang clients indefinitely (never sends InitialState).

### F-S022-ADV1-BLK-005 — ring_tail empty Vec with self-defer ("deferred to S-023")
**Severity:** BLOCKER. **AC:** AC-002. **Location:** state.rs:351-381. **Routing:** implementer (path a: implement conversion) OR product-owner (path b: amend BC).
CLAUDE.md Principle 3 violation. "Deferred to S-023" is wrong anchor (S-023 is reconnect, not ring storage).

### F-S022-ADV1-HIGH-001 — snapshot_initial_state returns empty sessions with "wired in S-023" defer
**Severity:** HIGH. **AC:** AC-002. **Location:** state.rs:372-377. **Routing:** implementer.
Same anti-pattern as BLK-005.

### F-S022-ADV1-HIGH-002 — Slow client during PermissionPromptResolved broadcast not removed
**Severity:** HIGH. **AC:** AC-001, BC-2.05.005 PC-3. **Location:** ipc_server.rs:219-225. **Routing:** implementer.
Missing drain-and-retain pattern present in fan_out_message; dead/slow senders accumulate.

### F-S022-ADV1-HIGH-003 — Frontmatter input-hash is [pending]
**Severity:** HIGH. **AC:** governance. **Location:** S-022 story frontmatter:26. **Routing:** state-manager.

### F-S022-ADV1-MED-001 — accept_loop doc reference stale
**Severity:** MEDIUM. **Location:** uds.rs:53. **Routing:** implementer.

### F-S022-ADV1-MED-002 — UdsClientTransport server-perspective; blocks S-025/S-026
**Severity:** MEDIUM. **Routing:** implementer (path a: TuiTransport wrapper).

### F-S022-ADV1-MED-003 — Single-commit implementation violates TDD micro-commit discipline
**Severity:** MEDIUM. **Routing:** [process-gap].

### F-S022-ADV1-MED-004 — AC-004 misleading prompt_id arg in register_prompt
**Severity:** MEDIUM. **Routing:** implementer (refactor API).

### F-S022-ADV1-NITPICK-001 — register_prompt doc inconsistency
**Severity:** NITPICK. **Routing:** implementer (doc-only).

## Process-Gap Findings

- [process-gap] F-S022-ADV1-MED-003 single-giant-commit recurring pattern
- [process-gap] "S-022 TODO" production-code anti-pattern; recommend CI rule to reject S-NNN TODO matches in commits for the current story

## Conclusion

Would not merge. Production deployment would fail to spawn accept loop, fail to initialize pending_decisions, fail to broadcast PermissionPromptQueued, return empty sessions and empty ring_tail. Convergence state: passes_clean_consecutive=0, converged=false. Recommend implementer Round 2 fix all BLOCKERs + HIGHs in-scope.
