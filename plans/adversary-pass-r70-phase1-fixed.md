---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.5 d321935 + VP v1.5.1 f07d66c + arch v1.0.11 af2101d; D-047 strict pass 2 of 3"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T08:00:00Z
pass_number: 2
policy: D-047-strict
---

# Adversarial Review Pass R70 — Phase 1 (D-047 Strict, Pass 2 of 3 — FINDINGS)

## Summary

**Verdict:** FINDINGS (3 substantive: 1 HIGH, 2 MEDIUM; 2 LOW observations)
**Counter:** RESET to 0/3

Fresh-context pass R70 pursued a new defect-class lens (cross-platform invariants + signal-handling convention correctness + VP-vs-BC over-tightening) and found 1 HIGH cross-platform implementation defect, 2 MEDIUM contract drifts, and 2 LOW observations. The HIGH defect (F-R70-1) survived 9 prior fresh-context passes (R62-R69) because prior axes focused on count/identifier consistency, intra-block coherence, and pin propagation — not platform-target invariants under the `directories` crate semantics.

## 22-BC ↔ 22-VP Audit

All 22 BCs map 1:1 to 22 VPs with matching identifiers, test names, test paths, and version pins (verified across PRD v1.5 + VP v1.5.1 + arch v1.0.11). The 3 findings concern semantic correctness within BC contracts, not identifier coherence.

## Findings

### F-R70-1 [HIGH] — macOS `runtime_dir()` returns `None`; daemon cannot start on a primary-target platform

**Files:**
- `.factory/specs/architecture/SS-daemon-lifecycle.md` line 30 + line 165
- `.factory/specs/prd.md` line 1179 (NFR-008 platform targets) + BC-DAEMON-005 precondition 2 (line 312)

**Defect:** `directories::ProjectDirs::runtime_dir()` returns `None` on macOS and Windows (only Linux has `$XDG_RUNTIME_DIR`). Arch §Start Sequence step 1 mandates this resolution with no fallback. macOS is declared primary target per NFR-008. Implementer has no defined behavior when `runtime_dir()` returns `None` — must invent a fallback (cache_dir? data_local_dir? /tmp/<uid>?), and different implementers will pick differently, breaking BC-DAEMON-005 + BC-LOCK-001 + §Lock File Discovery Policy.

**Asymmetry:** Spec explicitly handles `BaseDirs::new() == None` for engine module (BC-ENGINE-002-ERR fail-fast with `HomeUnresolvable` error variant). Parallel daemon-lifecycle platform-resolution failure is silently ignored — CLAUDE.md SOUL #4 violation (no silent fallback).

**Severity:** HIGH. Primary-target platform deployment blocker.

**Routing:** architect (fallback strategy is architectural — options: new BC-DAEMON-007 for runtime-dir-unresolvable fail-fast OR explicit platform-specific fallback chain in BC-DAEMON-005 OR `MONOCLE_RUNTIME_DIR` env override).

### F-R70-2 [MEDIUM] — VP-DAEMON-006 over-tightens BC-DAEMON-006 timestamp format

**Files:**
- `.factory/specs/prd.md` BC-DAEMON-006 invariant 1 (line 378): "shutdown_utc":"<ISO8601>" — generic, no precision constraint
- `.factory/specs/verification-properties.md` VP-DAEMON-006 line 611 + line 672 + line 687: regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` (mandatory `.\d{3}Z`)

**Defect:** VP enforces stricter format than BC. Valid ISO 8601 like `2026-05-15T07:30:00Z` (seconds-only) is BC-compliant but VP-non-compliant. Implementer following BC produces seconds-only timestamps; test against VP fails — VP defect not BC violation.

**Resolution:** (a) Tighten BC-DAEMON-006 invariant 1 to match EC-044 precedent (millisecond format for `last_hook_ts`). (b) Loosen VP regex. Recommend (a) for cross-field consistency.

**Routing:** product-owner (BC tightening preferred) OR formal-verifier (VP loosening); architect adjudicates if disagreement.

### F-R70-3 [MEDIUM] — Exit code 130 semantically inappropriate for SIGTERM scenario

**Files:**
- `.factory/specs/architecture/SS-daemon-lifecycle.md` lines 530-533
- `.factory/specs/prd.md` BC-DAEMON-004 postcondition 8 (line 270) + canonical test vector (line 292)

**Defect:** POSIX convention: signal-terminated process exit code = 128 + N. SIGINT (2) → 130. SIGTERM (15) → 143. Spec uses 130 for "second SIGTERM during drain" — semantically encodes Ctrl-C origin, not SIGTERM. External monitoring (systemd `Restart=on-failure`, k8s probes, CI status parsers) will misinterpret.

If daemon explicitly `exit(130)` programmatically after second SIGTERM, misleading. If OS default-terminates, actual exit code is 143 — making spec untestable.

**Resolution:** (a) Change to 143 (POSIX SIGTERM); (b) Use non-convention code with explicit non-POSIX documentation; (c) Distinguish SIGINT-during-drain → 130 vs SIGTERM-during-drain → 143 vs admin /shutdown second-call → programmatic third code.

**Severity:** MEDIUM. Real correctness gap for monitoring/CI integrations. Not blocking implementation but blocks production deployment correctness.

**Routing:** architect (protocol decision authority per §BC Summary footer authority split).

## Observations

### Obs-R70-1 [LOW] — EC-031 fail-open default lacks security rationale

**File:** `.factory/specs/prd.md` line 999

**Concern:** "fail-open default for unrecognized event types (HookDecision::Allow per BC-ENGINE-002 Phase 1)" — fail-open on permission decisions is generally a security antipattern. May be intentional for forward-compat (unknown variant presumed non-permission-relevant) but rationale absent. Future audit would need justification.

**Routing:** product-owner (add rationale OR change to Defer).

### Obs-R70-2 [LOW] — VP-DAEMON-004 over-budget exit-code loosens BC binary spec

**Files:**
- BC-DAEMON-004 postcondition 8 (line 270): binary 0 or 130
- VP-DAEMON-004 post-condition 6 (lines 466-471): tolerates either 0 or 130

**Concern:** VP relaxes BC's binary specification for over-budget (15s handler) scenario. BC offers no relaxation. Both interpretations defensible but drift is real.

**Routing:** product-owner (BC clarification) or formal-verifier (VP tightening).

## Frozen META Catalog Status (D-054)

| ID | Re-litigated? |
|----|---------------|
| F-R55-adv-1 | NO |
| F-R55-adv-3 | NO |
| F-R61-adv-1 | NO |
| F-R61-2 | NO |

All 4 preserved.

## Novelty Assessment

**Novelty: HIGH.** F-R70 findings target a defect class not exercised by R62-R69:
- F-R70-1: cross-platform invariant under-specification (the `directories` crate's macOS behavior). Prior passes focused on identifier consistency, test-name canonicalization, pin propagation, intra-document numeric coherence — none checked dependency-crate platform semantics against platform-target NFRs.
- F-R70-2: new BC-vs-VP contract drift sub-class (BC under-specifies, VP over-tightens).
- F-R70-3: POSIX-convention semantic correctness gap.

These reinforce the AgenticAKM finding that fresh-context passes find genuinely new defects through pass 9+.

## Pass 2 Verdict and Pass 1 attempt 6 Readiness

**Verdict:** FINDINGS — D-047 counter RESET to 0/3.

**Required closure chain:**
1. **architect**: F-R70-1 (runtime-dir resolution) + F-R70-3 (exit-code convention) — both architecture decisions
2. **product-owner**: F-R70-2 (BC-DAEMON-006 timestamp tightening) + Obs-R70-1 (EC-031 security rationale)
3. **formal-verifier**: VP propagation + Obs-R70-2 closure
4. **state-manager**: STATE.md update + L-F-R63 Extension 3 codification (platform-invariant + POSIX-convention sweep discipline)

After fix-burst, dispatch R71 + cons R10 as pass 1 attempt 6.

## Cycle health observation (for human gate consideration)

Convergence trajectory across 10 attempts: 13→5→1→4→0→2→1→0→0→3 findings. NOT monotone. Each fresh-context pass examines new defect lenses and finds new substantive defects. The system is not converging under strict D-047 because adversaries can always identify new review angles.

Surface to human at T-5: consider whether D-047 strict (0×3 consecutive) is achievable in finite time, OR whether a checkpoint-with-residuals model is more appropriate. The findings are GENUINELY substantive (F-R70-1 is a real macOS deployment blocker), so the strict policy is producing value — but the policy may have an asymptotic convergence problem.
