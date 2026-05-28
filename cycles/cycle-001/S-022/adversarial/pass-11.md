---
document_type: adversarial-pass
story: S-022
pass: 11
producer: vsdd-factory:adversary
timestamp: 2026-05-28T08:30:00Z
classification: NITPICK_ONLY
findings_count:
  blocker: 0
  high: 0
  medium: 0
  nitpick: 1
prior_pass_resolution:
  resolved: 0
  partial: 0
  not_fixed: 0
  phantom: 0
  over_corrected: 0
---

# S-022 Adversarial Pass 11

## Summary

Fresh-context Pass-8-depth re-audit of 23-commit branch. Walked every BC postcondition + edge case against test coverage. Traced 5 production functions. One LOW nitpick: timeout-vs-decision race in pre_tool_use.rs causes potential duplicate PermissionPromptResolved broadcast (BC-compliant — TUI idempotent — but defensive fix available).

## BC/EC Coverage Audit

**BC-2.05.002:** PC-1..PC-6 ✓, Inv 1-3 ✓, EC-001..EC-006 ✓
**BC-2.05.005:** PC-1..PC-4 ✓, Inv 1-3 ✓, EC-001..EC-006 ✓
**No untested BC postconditions or edge cases.**

## Pass 7 OBS-1 (select! biasedness) Reassessment

LOW NITPICK confirmed. Pseudo-random branch selection is safer default for per-client task (no must-prefer branch).

## NEW Finding

### F-S022-ADV11-LOW-001 — Timeout-vs-decision race causes duplicate PermissionPromptResolved
**Severity:** LOW (NITPICK). **Routing:** implementer (one-line fix).
**Location:** crates/monocle-runtime/src/hooks/pre_tool_use.rs:110-125

Timeout arm calls `remove_timed_out_prompt(prompt_id)` (line 113) and IGNORES the Option<PermissionPromptPayload> return value, then unconditionally broadcasts PermissionPromptResolved (line 119). If a TUI client's PermissionDecision arrives during the narrow window between tokio::time::timeout returning Err and the timeout arm running remove_timed_out_prompt, the per-client IPC task's handle_permission_decision will have already removed the entry via resolve_prompt and broadcast Resolved once. Timeout arm then broadcasts Resolved a SECOND time.

**BC compliance:** Strictly compliant. BC-2.05.005 Inv 3 doesn't forbid duplicate Resolved. BC-2.06.023 PC-3 makes TUI idempotent on Resolved (unknown prompt_id → no-op).

**Production-grade concern:** Duplicate broadcast is observable on wire, doubles broadcast traffic for racy resolutions. One-line fix:
```rust
if state.pending_decisions.as_ref()
    .and_then(|r| r.remove_timed_out_prompt(prompt_id))
    .is_some()
{ /* broadcast Resolved */ }
```

**Why LOW not HIGH/MEDIUM:** TUI already idempotent; race window microseconds; spec doesn't forbid behavior.

## Spec Drift Check

BC-2.05.002 v1.0.5 ✓; BC-2.05.005 v1.6.0 ✓; SS-ipc v1.8.0 ✓ (story still pins SS-ipc v1.4.0 in 3 inline references — content-equivalent, cosmetic stale pin).

## Sister Story Propagation

S-025/S-026 BC-2.05.002 anchors verified clean. No new gaps.

## Process-Gap Findings

None.

## Conclusion

passes_clean_consecutive=2. last_classification=NITPICK_ONLY. converged=false. Earliest convergence: Pass 12 (one more NITPICK_ONLY needed).

Recommendation: Optional one-line fix to pre_tool_use.rs timeout arm. Either fix-in-scope (production-grade) or accept-as-documented (BC-compliant) acceptable.
