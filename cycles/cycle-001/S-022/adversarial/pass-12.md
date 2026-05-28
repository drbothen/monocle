---
document_type: adversarial-pass
story: S-022
pass: 12
producer: vsdd-factory:adversary
timestamp: 2026-05-28T09:30:00Z
classification: MEDIUM_PRESENT
findings_count:
  blocker: 0
  high: 0
  medium: 1
  nitpick: 0
prior_pass_resolution:
  resolved: 0
  partial: 1
  not_fixed: 0
  phantom: 0
  over_corrected: 0
---

# S-022 Adversarial Pass 12

## Summary

Pass 11's F-S022-ADV11-LOW-001 fix (commit 71f80d2) PARTIALLY resolved. Production gate at pre_tool_use.rs:115-133 correctly added and BC-compliant. Accompanying unit test is vacuous mirror-test that does not exercise production code path. Pattern matches Pass 8/9 vacuous-assertion findings.

## Pass 11 Fix Verification

- Gate at pre_tool_use.rs:119 correctly conditioned on `removed.is_some()` ✓
- Comment documents race window ✓
- BC-2.05.005 PC-4 preserved ✓
- No regression risk to non-racy timeout path ✓
- BUT: Accompanying test is vacuous mirror-test ✗

## F-S022-ADV12-MED-001 — Pass 11 fix is mirror-tested (vacuous coverage)

**Severity:** MEDIUM. **Routing:** test-writer. **Confidence:** HIGH.
**Location:** pre_tool_use.rs:408-446

The new test `test_F_S022_ADV11_LOW_001_timeout_broadcast_skipped_when_entry_already_removed`:
1. Calls `PendingDecisionRegistry::remove_timed_out_prompt` directly on empty registry (L417), bypassing production
2. Lines 433-438 contain literal `if removed.is_some() { broadcast_to_subscribers(...) }` block — textual copy of production guard at pre_tool_use.rs:119-133
3. Assertion at L441-445 checks test-local copy did not broadcast — asserts on its own mirror
4. Production function `post_hook_pre_tool_use` is never invoked
5. Mutation test: replacing production gate with `if true` does NOT cause test to fail

No production-invoking test exercises the `is_none()` race branch. `test_BC_2_04_007_defer_timeout_returns_allow` always hits `is_some()` branch.

**Why MEDIUM not LOW:** Pass 11 explicitly invested a fix commit; that investment is undermined by a test that cannot detect regression. Production-grade default (CLAUDE.md Rule 1) binds the test to be production-grade too.

**Required fix:** Rewrite as integration test that drives `post_hook_pre_tool_use` through Defer + concurrent `resolve_prompt` + timeout, asserting exactly ONE PermissionPromptResolved broadcast (not two). Mutation-validate: deleting production gate must cause test to fail.

## Spec / BC Drift Check

BC-2.05.002 v1.0.5 ✓; BC-2.05.005 v1.6.0 PC-4 ✓; SS-ipc v1.8.0 ✓. No new vacuous-assertion patterns elsewhere.

## Process-Gap Findings

None. Mirror-test pattern is already codified discipline (Pass 8/9); this is a recurrence within the same story.

## Conclusion

passes_clean_consecutive=0 (reset from 2). last_classification=MEDIUM_PRESENT. converged=false. Earliest convergence: Pass 15.

Route F-S022-ADV12-MED-001 to test-writer for mutation-killing coverage. Do not advance to demo-recorder.
