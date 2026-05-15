---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.6 76570ac + VP v1.6 7ba155a + arch v1.0.12 727c826; F-R70 closure chain applied; D-047 strict pass 1 of 3 (attempt 6); L-F-R63 Extensions 1+2+3 codified"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T07:00:00Z
pass_number: 1
policy: D-047-strict
---

# Adversarial Review Pass R71 — Phase 1 (D-047 Strict, Pass 1 attempt 6 — FINDINGS)

## Summary

**Verdict:** FINDINGS — 2 HIGH + 2 MEDIUM + 1 LOW substantive + 1 process-gap observation.

| Severity | Count |
|----------|-------|
| HIGH | 2 |
| MEDIUM | 2 |
| LOW | 1 |
| Observations | 1 (process-gap) |
| **Total substantive** | **5** |

**Counter:** RESET to 0/3.

## 22-BC x 22-VP Audit

22/22 mapping verified. F-R70 closure properly anchored (BC-DAEMON-004/005/006, EC-057/058/059, E-DAEMON-004). Count claims (22 BCs, 14 error codes, 59 edge cases, 23 test names) all consistent.

## Findings

### F-R71-1 [HIGH] — VP cites `directories 5`; canonical pin is `directories 6`

**Files:** VP-DAEMON-005 §Pre-conditions (line 642); VP §Trace v1.6 (line 2197).

**Evidence:** SS-deps-pin-manifest.md line 48 = `directories 6`; CLAUDE.md Key Tech Stack = `directories 6`. VP's "directories 5 (or pinned equivalent)" contradicts canonical.

**Impact:** Implementer would pin wrong major version (5 vs 6 SemVer-incompatible). L-F-R63 Extension 3 dependency-crate sweep discipline violation.

**Routing:** formal-verifier (VP fix at 2 sites).

### F-R71-2 [HIGH] — Arch v1.0.12 test name drift from PRD-canonical

**Files:** arch v1.0.12 lines 635 + 770 (`test_BC_DAEMON_004_exit_codes`); PRD v1.6 line 306 + VP v1.6 line 556 (`test_BC_DAEMON_004_exit_codes_posix_distinct`).

**Impact:** Implementer following arch will write differently-named test, creating integration mismatch with PRD §7 RTM. Recurrent of F-R63-adv-2 class.

**Authority:** Arch's own §BC Summary footer (lines 707-709) states "PRD is source-of-truth for canonical test names". Arch must adopt PRD canonical.

**Routing:** architect (arch v1.0.12 → v1.0.13: 2-site test name correction).

### F-R71-3 [MEDIUM] — NFR-008 cited 5+ times as "macOS primary target"; NFR-008 actually says macOS + Linux coequal

**Files:**
- arch v1.0.12 lines 35, 197-198, 731-732, 744 ("macOS is the primary target (NFR-008)")
- PRD v1.6 line 328 BC-DAEMON-005 precondition 2 rationale

**Evidence:** PRD line 1210 NFR-008 = "`macOS + Linux (darwin/linux x amd64/arm64)`" — coequal, no "primary" designation. "Primary" framing originates in brief §Scope as SHARED across both platforms.

**Impact:** Semantic mis-anchor. Disposition rationale (macOS users shouldn't need env config) is factually correct, but cited authority does not support claim.

**Routing:** architect (4 arch sites) + product-owner (1 PRD site).

### F-R71-4a [MEDIUM] — VP cites `tower 0.5` as project pin per manifest; tower not in SS-deps-pin-manifest.md

**File:** VP-DAEMON-004 §Pre-conditions line 473.

**Evidence:** SS-deps-pin-manifest.md has no tower entry. Tower is transitive via axum 0.8 but not an explicit workspace pin.

**Routing:** architect (manifest decision — add tower or remove VP citation suffix) + formal-verifier (VP rephrase per architect's choice).

### F-R71-4b [MEDIUM] — VP-DAEMON-005 "nix 0.30 OR libc 0.2" — Principle 6 violation (pending architect review)

**File:** VP-DAEMON-005 §Pre-conditions lines 645-647.

**Evidence:** VP says "`nix 0.30` ... is the project pin OR `libc 0.2` is used directly — the test asserts the chosen mechanism in the source." CLAUDE.md Principle 6 forbids "pending architect review" for answerable mechanical questions.

**Recommended resolution:** Architect picks nix 0.30 (typed wrapper, safer Signal::None API) or libc 0.2 (minimal-dep). Architect-binding decision required.

**Routing:** architect (decide + manifest update) + formal-verifier (VP single binding pin).

### F-R71-5 [LOW] — Recovery JSON `pid` field type placeholder drift across artifacts

**Files:** arch line 583 (`<N>`); PRD line 406 (`<N>`); VP line 776 (`<int>`).

**Impact:** Cosmetic. VP-only outlier breaks established `<N>` convention.

**Routing:** formal-verifier (VP `<int>` → `<N>`).

## Observation

### Obs-R71-1 [process-gap] — L-F-R63 Extension 3 not enforced in F-R70 dispatch prompts

The deps-pin-manifest sweep discipline was CODIFIED post-R70 in cycle-001/lessons.md but the F-R70 closure burst dispatch prompts did NOT include the discipline. Result: F-R71-1 (directories 5 in VP v1.6) and F-R71-4 (fabricated tower + nix-or-libc in VP v1.6) are direct misses by Extension 3.

**Recommendation:** Orchestrator dispatch prompts for formal-verifier (or any agent doing pin propagation) MUST include mandatory deps-pin-manifest sweep checklist. Codification needs an ENFORCEMENT mechanism, not just documentation.

**Routing:** state-manager (post-fix) to add enforcement language to dispatch templates.

## Frozen META Catalog Status (D-054)

| ID | Re-litigated? |
|----|---------------|
| F-R55-adv-1 | NO |
| F-R55-adv-3 | NO |
| F-R61-adv-1 | NO |
| F-R61-2 | NO |

All 4 preserved.

## Novelty Assessment

- F-R71-1: HIGH novelty (first Extension 3 deps-pin sweep finding).
- F-R71-2: MEDIUM-HIGH (recurrent class, new instance).
- F-R71-3: MEDIUM (new semantic mis-anchor class — NFR citations).
- F-R71-4: MEDIUM (Principle 6 violation + fabricated citation).
- F-R71-5: LOW (cosmetic).
- Obs-R71-1: HIGH process-gap (codification-without-enforcement).

## Convergence Trajectory (11 attempts)

`13→5→1→4→0→2→1→0→0→3→5`. NOT monotone. Lens-rotation continues to find new defect classes. Cycle-health concern for T-5 human gate remains valid.

## Pass 1 Attempt 7 Readiness

BLOCKED until F-R71 closure chain applied:

1. **architect:** arch v1.0.13 with F-R71-2 + F-R71-3 + F-R71-4 dispositions (test name correction + NFR mis-anchor rephrase + nix/libc binding + tower decision)
2. **product-owner:** PRD v1.7 with PRD-side propagation (test name pin, NFR phrasing fix, arch pin propagation)
3. **formal-verifier:** VP v1.7 with F-R71-1 (directories) + F-R71-4 propagation + F-R71-5 placeholder + arch + PRD pin propagation
4. **state-manager:** STATE.md update + Obs-R71-1 enforcement codification
5. R72 + cons R11 dispatch
