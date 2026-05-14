---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.4 e704b50 + VP v1.4 56b57ac + arch v1.0.11 af2101d; D-047 strict pass 2 of 3"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T14:30:00Z
pass_number: 2
policy: D-047-strict
---

# Adversarial Review Pass R67 — Phase 1 (D-047 Strict, Pass 2 of 3 — FINDINGS)

## Summary

**Verdict:** FINDINGS (2 HIGH, 1 process-gap observation). D-047 strict counter RESETS to 0/3.

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 0 |
| LOW | 0 |
| Observations | 1 |

## 22-BC ↔ 22-VP Audit

Coverage matrix verified end-to-end. 22/22 mapping holds (IDs, names, paths, source-of-truth versions all reconcile).

## Findings

### F-R67-1 [HIGH] — VP-TYPES-001 §Mechanism prose mis-states primary verification mechanism

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md` line 1080.

**Defect:** §Mechanism prose says "unit-test (primary, via a `cargo clippy` lint configuration); mutation-test (auxiliary)" — but VP's own §Post-conditions (lines 1091-1099) and PRD BC-TYPES-001 invariant 1 (PRD line 707) both authoritatively state the primary mechanism is a `syn 2` AST audit at `monocle-core/tests/enum_audit.rs`. Clippy is supplement only.

**Internal contradiction:** §Mechanism (clippy primary) vs §Post-condition 1 (syn 2 primary) in the same VP block.
**External contradiction:** §Mechanism (clippy primary) vs PRD invariant 1 (syn 2 primary, "NOT clippy"; clippy "supplement only").

**Impact:** An implementer reading §Mechanism first would build a clippy-only check and miss the syn 2 AST audit harness.

**Root cause:** Pre-F-R62-6 phrasing was retained in §Mechanism prose; F-R62-6 upgraded PRD/VP rigor to AST audit but propagation sweep updated post-condition 1 without updating §Mechanism prose. L-F-R63-PARTIAL-FIX pattern recurring at intra-block granularity.

**Recommended fix:** Replace line 1080 with: "**Mechanism:** unit-test (primary, via a `syn 2` AST audit at `monocle-core/tests/enum_audit.rs` per PRD v1.4 §BC-TYPES-001 invariant 1); mutation-test (auxiliary); clippy `non_exhaustive_omitted_patterns` lint configuration (supplementary)."

**Routing:** formal-verifier.

### F-R67-2 [HIGH] — PRD §3 EC-045 prose wrong boundary byte count for 256 KiB body limit

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/prd.md` line 228.

**Defect:** EC-045 prose says "Request body is exactly 262,144 bytes: HTTP 413 (limit is strictly exclusive — `> limit` triggers the rejection; axum's `DefaultBodyLimit::max(N)` rejects bodies strictly exceeding N bytes)."

The leading verdict ("exactly 262,144 bytes: HTTP 413") contradicts its own rationale ("`> limit` triggers the rejection"). For N=262,144, "strictly exceeding" = > 262,144 = ≥ 262,145. Body of exactly 262,144 should return HTTP 200.

Wrong-boundary value contradicts 11 sibling sites:
1. PRD §9 EC-045 catalog row (line 1342): "262,145 bytes → HTTP 413" — correct
2. PRD BC-DAEMON-003 PC2 (line 213): "exceeding 262,144 bytes" — correct
3. PRD BC-DAEMON-003 canonical test vector (line 238): "262,145 bytes | HTTP 413" — correct
4. PRD BC-DAEMON-003 §Verification (line 243): "262,145-byte POST" — correct
5. PRD NFR-005 (line 1176): "262,145-byte body, assert 413" — correct
6. PRD §1.3 D-7 (line 1220): "262,145-byte body returns HTTP 413" — correct
7. VP-DAEMON-003 mechanical property 1 (line 339-342): "262,145 bytes returns HTTP 413" — correct
8. VP-DAEMON-003 mechanical property 3 (line 343-348): "262,144 bytes also succeed" — correct
9. VP-DAEMON-003 post-condition 2 (line 371-372): "262,144-byte body → HTTP 200" — correct
10. Arch SS-daemon-lifecycle.md §Body Size Limit (line 110): "exceeding" semantic — correct
11. VP-DAEMON-003 fuzz harness range (line 397-402): "≤ 262,144 = 200; > 262,144 = 413" — correct

EC-045 prose is the SOLE outlier. Isolated typo where "262,144" replaced "262,145" for the rejection-boundary illustration. Self-contradicted by the rationale clause immediately following.

**Impact:** An implementer reading EC-045 prose first would write `assert_eq!(send_body(262144), 413)`. That assertion fails in practice (axum returns 200 for body = N) and conflicts with VP-DAEMON-003 PC2.

**Recommended fix:** Change "262,144" to "262,145" in PRD line 228.

**Routing:** product-owner.

## Observations

### Obs-1 [process-gap] — L-F-R63-PARTIAL-FIX needs intra-artifact same-ID consistency check extension

Both F-R67-1 and F-R67-2 are same-block / same-ID semantic propagation defects that survived F-R62-6 / F-R63 / F-R65 sweep protocols. The codified Semantic propagation sweep lesson calls for prose-lead-in count sweeps but NOT for same-block contradiction sweeps (e.g., "§Mechanism vs §Post-conditions internal consistency"; "EC-NNN catalog row vs EC-NNN prose verdict for same EC ID").

**Recommended codification (post fix-burst):** Extend L-F-R65 semantic propagation sweep to include intra-artifact same-ID consistency check — for every catalog/index entry that cross-references body prose (EC-NNN, BC-NNN, VP-NNN, NFR-NNN), verify catalog row and body prose verdict agree on numeric facts and outcome claims.

**Routing:** state-manager (post fix-burst) to amend `cycles/cycle-001/lessons.md` and add new META rule to `SS-conventions-anti-patterns.md`. Not blocking convergence; surfaced for codification.

## Frozen META Catalog Status (D-054)

| ID | Re-litigated? |
|----|---------------|
| F-R55-adv-1 | NO |
| F-R55-adv-3 | NO |
| F-R61-adv-1 | NO |
| F-R61-2 | NO |

All 4 preserved.

## Novelty Assessment

**Novelty: HIGH.** Both findings genuinely NEW — not surfaced in R62-R66. F-R67-1 (§Mechanism mis-attribution) survived F-R62-6 closure because that fix updated post-condition 1 but not the sibling §Mechanism prose in the same block. F-R67-2 (EC-045 off-by-one) survived all prior boundary-condition reviews because they focused on auth taxonomy, test names, pin propagation. Fresh-context value demonstrated: re-derived understanding caught 2 substantive defects.

## Pass 2 Verdict and Pass 3 Readiness

**Pass 2 verdict: FINDINGS.** D-047 counter RESET to 0/3.

**Pass 3 readiness:** BLOCKED until F-R67 fix-burst applied (formal-verifier closes F-R67-1; product-owner closes F-R67-2). After fix-burst, 3-clean-pass cycle restarts at pass 1 attempt 4.
