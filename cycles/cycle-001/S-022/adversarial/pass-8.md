---
document_type: adversarial-pass
story: S-022
pass: 8
producer: vsdd-factory:adversary
timestamp: 2026-05-28T05:30:00Z
classification: HIGH_PRESENT
findings_count:
  blocker: 0
  high: 2
  medium: 0
  nitpick: 0
prior_pass_resolution:
  resolved: 0
  partial: 1
  not_fixed: 0
  phantom: 0
  over_corrected: 0
---

# S-022 Adversarial Pass 8

## Summary

Fresh-context Pass-6-depth audit of 20-commit branch. Two HIGH findings: (1) Pass 6 MED-001 deferral verified at story-NAME level in Pass 7 but not at story-CONTENT level — S-025 and S-026 don't encode the architect's idempotent-insert directive (CLAUDE.md Principle 3 violation, deferral functionally orphaned); (2) ac_001 test has zero assertions, body is 5-step assertion plan written as COMMENTS — AC-001 per-client EOF cleanup unverified. Pass 7's NITPICK_ONLY was premature.

## Part A — Pass 7 + prior resolution

Pass 7 reported 1 LOW OBS (select! biasedness) — non-blocking, unchanged.

Re-verifying Pass 6 deferral with deeper analysis:
| Finding | Verdict | Evidence |
|---|---|---|
| F-S022-ADV6-MED-001 (deferral to S-025/S-026) | PARTIAL | Architect directive in architect-decisions-pass-6.md:147-194 names S-025/S-026, but neither story contains AC, task, or test for the directive. Re-surfaced as F-S022-ADV8-HIGH-001. |

## Part B — NEW Pass 8 Findings

### F-S022-ADV8-HIGH-001 — Orphaned dedup-on-insert directive (CLAUDE.md Principle 3 violation)
**Severity:** HIGH. **Routing:** story-writer (primary) + product-owner co-sign.
**Locations:**
- Directive: architect-decisions-pass-6.md:147-194
- Missing in S-025-tui-skeleton-sessions.md:90-94 (AC-008), :122-138 (tasks)
- Missing in S-026-permission-overlay-core.md:48-55 (AC-001), :178-209 (tasks)

**Trace:** Pass 6 architect Option D mandates that whichever of S-025/S-026 first implements VecDeque<PromptModal> MUST add idempotent-on-prompt_id check before push_back AND add test_snapshot_window_prompt_dedup integration test. S-025 declares the VecDeque field + AC-008 initialization — NO dedup. S-026 AC-001 says "calls app.overlay_stack.push_back(modal)" — NO precondition. Neither story references BC-2.05.002 or Invariant 4. Neither has BC-2.05.002 in frontmatter.

**Impact:** When S-026 ships per current AC-001, implementer will push_back without dedup. BC-2.05.002 Invariant 4 (v1.0.5) violated at runtime. TUI overlay renders prompt twice.

**Required remediation:**
- S-026 AC-001: add precondition "if payload.prompt_id already in app.overlay_stack, silently discard (TRACE log)"; cite BC-2.05.002 Invariant 4
- S-026 tasks: apply_permission_prompt_queued helper + test_snapshot_window_prompt_dedup
- S-025 AC-008: same idempotent insert when populating from InitialState.overlay_stack
- Both stories behavioral_contracts frontmatter: add BC-2.05.002

### F-S022-ADV8-HIGH-002 — ac_001 has zero assertions; AC-001 untested
**Severity:** HIGH. **Routing:** test-writer.
**Location:** crates/monocle-ipc/tests/connection_handshake.rs:38-52

**Trace:** ac_001_per_client_tokio_task_spawned body = tempdir + spawn_test_daemon + 5 COMMENT lines describing assertions never executed. No connect, no length check, no EOF, no post-EOF check. Test passes vacuously.

AC-001 traces to BC-2.05.002 PC-1 (per-client task spawn + EOF subscriber removal). Production code at ipc_server.rs:197-198 does call remove_subscriber on loop break, but no integration test asserts this end-to-end.

**Impact:** Regression that removes remove_subscriber line in ipc_server.rs:197 would silently pass all tests; subscriber list would leak on every TUI EOF.

**Required remediation:** Populate ac_001 body per the prose. Pre-fix mutation test: delete remove_subscriber line and confirm new ac_001 fails red. This is required, not suggested.

## Process-Gap Findings

None. Both content defects.

## Novelty Assessment

**Novelty: MEDIUM.** HIGH-001 audits downstream artifact of prior-pass deferral (prior passes didn't do this). HIGH-002 sample test body for assertion presence (Pass 7 didn't sample). Both emerge from Pass-6-depth analysis.

## Conclusion

passes_clean_consecutive reset to 0 (from 1). last_classification=HIGH_PRESENT. converged=false. Earliest convergence: Pass 11.

Dispatch story-writer for HIGH-001 + test-writer for HIGH-002. Do not run Pass 9 until both fixed AND mutation test verified.

Pass 5 and Pass 7 "clean" verdicts demonstrate: a deferral named at story level must be verified anchored AT the story content level. Adding this audit step to future adversary protocol is recommended.
