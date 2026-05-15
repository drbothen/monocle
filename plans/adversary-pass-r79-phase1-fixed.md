---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.11 1f90b64 + VP v1.13 5367f2c + arch v1.0.16 6bb93e2 + manifest v1.1.12 8005075; F-R78 closure chain applied; D-047 strict pass 1 of 3 (attempt 13); ALL 11 codified disciplines in force"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T22:30:00Z
pass_number: 1
attempt: 13
policy: D-047-strict
---

# Adversarial Review Pass R79 — Phase 1 (D-047 Strict, Pass 1 attempt 13 — FINDINGS)

## Summary

**Verdict:** FINDINGS — 2 HIGH + 1 MED. Counter remains 0/3.

F-R78 + GAP-R17-001 + Extension 3 (line 1906) closures all verified GREEN. New lens rotation: BC-vs-Brief/Vision JC-closure alignment + §G-N future-attachment integrity + lift_invariants_to_bcs.

## Findings

### F-R79-1 [HIGH] — PRD §7 RTM Test File column omits second BC-DAEMON-004 test file

**Files:**
- PRD line 1264: RTM lists only `monocle-runtime/tests/graceful_shutdown.rs`
- PRD §3 BC-DAEMON-004 §Verification lines 304-306: BOTH files listed (graceful_shutdown.rs + daemon_lifecycle.rs)
- VP §Coverage Matrix line 2033: BOTH files listed correctly
- PRD §Trace v1.6 line 1732: documents the v1.6 burst added daemon_lifecycle.rs for `test_BC_DAEMON_004_exit_codes_posix_distinct`

**Defect:** PRD §7 RTM Test File column did not propagate the F-R70 closure that added `daemon_lifecycle.rs`. Same axis as F-R78-1 (closure landed but didn't propagate to a cross-reference column).

**Impact:** Test-writer using §7 RTM as canonical authority misses daemon_lifecycle.rs → POSIX exit code taxonomy test absent → BC-DAEMON-004 invariant 4 silently de-scoped.

**Routing:** product-owner.

### F-R79-2 [HIGH] — VP §G-6 NFR-002 description fabricates JC-2-omitted hook surface

**Files:**
- VP lines 2177-2178: "NFR-002 — compute-bounded hook surface, e.g., `post-tool-use` after large file edits"
- PRD line 1204 NFR-002: scoped to `Notification` only
- PRD §1.5 line 75 (Out of Scope): "Does NOT ship `PostToolUse` hook endpoint in Phase 1 — per JC-2"
- Brief §Explicit Non-Goals lines 232-235: confirms PostToolUse Phase 1 non-goal
- BC-ENGINE-003 hook_paths: 5 endpoints, PostToolUse absent

**Defect:** `post-tool-use` mentioned as illustrative example in VP §G-6, but it's a JC-2-OMITTED endpoint. Gene-source identifier (from upstream Claude Code endpoint naming) leaked into VP normative content as fabricated Phase 1 surface.

**Impact:** Phase 3 spec-evolution author reading §G-6 would author VP-LATENCY-002 against `post-tool-use` — non-existent endpoint → fabricated test target OR re-deferral recapitulating original JC-2 closure.

**META class:** Same as F-R77-3 (gene-source identifier leaked into VP normative content).

**Routing:** formal-verifier.

### F-R79-3 [MED] — VP-DAEMON-005 0o700 invariant anchored only to PRD EC-052, not §Postcondition/§Invariant

**Files:**
- VP Post-condition 9 (lines 706-722) + probe 5.e (lines 726-728) + counter-example 10 (lines 766-782) + mutation-test rationale: all enforce 0o700 runtime-dir mode as defense-in-depth security invariant
- PRD §BC-DAEMON-005 Postconditions (lines 330-339): NO 0o700 directory mode
- PRD §BC-DAEMON-005 Invariants (lines 341-345): only lock-FILE 0o600, not runtime-DIR 0o700
- PRD §BC-DAEMON-005 EC-052 line 351: contract present but at EC-tier
- arch line 255: single mention buried in §Start Sequence step 1

**Defect:** F-R75-1 closure landed on VP side as primary post-condition with security defense-in-depth narrative, but PRD-BC side buries the contract in EC-052 alone. lift_invariants_to_bcs gap.

**Impact:** Implementer reading only BC §Postconditions/§Invariants/§Verification misses the 0o700 contract → ships umask-default 0o755 vulnerability that F-R75-1 was authored to prevent.

**Routing:** product-owner (promote EC-052 0o700 contract to new §BC-DAEMON-005 Postcondition; reference in §Verification).

## Frozen META Catalog Status

All 4 D-054 entries preserved. None re-litigated.

## Codification Recommendations

The fabrication-pattern + lens-rotation cycle continues to discover NEW axes:

- **L-F-R63 Extension 10**: PRD §3 §Verification → §7 RTM Test File column propagation discipline (catches F-R79-1 class)
- **L-F-R63 Extension 11**: BC-vs-Brief/Vision JC-closure alignment audit — gene-source identifiers must NOT leak into VP normative prose without explicit Phase-N scope marking (catches F-R79-2 class)
- **L-F-R63 Extension 12**: VP-to-BC §Postcondition anchor audit — every VP §Post-condition with counter-example + mutation rationale must anchor to BC §Postcondition or §Invariant, not just EC (catches F-R79-3 class)

## Convergence trajectory

19 attempts: 13→5→1→4→0→2→1→0→0→3→5→3→0→3→2→2→6→2→3. The fabrication-pattern + axis-rotation META class continues to find new substantive defects at new axes. No monotonic convergence trend.

## Pass 1 attempt 14 readiness

BLOCKED until F-R79 closure chain:
1. product-owner: PRD v1.11 → v1.12 (F-R79-1 RTM Test File column + F-R79-3 0o700 Postcondition lift)
2. formal-verifier: VP v1.13 → v1.14 (F-R79-2 §G-6 NFR-002 description rewrite + PRD pin propagation)
3. state-manager: STATE.md + L-F-R63 Extensions 10/11/12 codification
4. R80 + cons R19
