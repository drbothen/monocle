---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.19 2e24e09 + VP v1.25 ba8eea4 + arch v1.0.19 8a68cc9 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 25 (R92); post-F-R91 serial fix-burst snapshot; CONTENT-CENTRIC LENS — arch↔PRD↔VP anchor consistency + F-R91 content verification"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T20:37:58Z
pass_number: 1
attempt: 25
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 3 HIGH + 2 LOW observations
lens_class: CONTENT-CENTRIC (anchor consistency + F-R91 content verification)
---

# Adversarial Review R92 — Phase 1 (D-047 Strict, Pass 1 Attempt 25 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

**Lens class: CONTENT-CENTRIC (arch↔PRD↔VP anchor consistency + F-R91 content verification).** This pass applies the SE-14b anchor consistency discipline to verify the F-R91 serial fix-burst produced correct anchor citations throughout, and cross-checks the F-R91 BC lift content (BC-DAEMON-001/002 Postcondition 1 semver regex, BC-DAEMON-002 Postcondition 1 pid≥1, BC-DAEMON-006 Invariant 1 4-field constraints, EC-061 empty-string current_cycle) against corresponding VP probe text.

**Cons R31 verdict:** CLEAN (commit 611a97c) — 0 findings. The clean consistency round does NOT advance the D-047 counter because R92 adversary FAIL overrides per D-047 strict policy.

**KEY META INSIGHT (I-R92-5):** SE-14b was codified as VERIFICATION discipline only ("every VP §Post-condition that cites BC anchors MUST resolve") but the F-R91 burst needed it as AUTHORING discipline ("every BC lift in a serial chain MUST cause downstream VP probes to ADD new BC-anchor citations"). The v1.25 burst applied SE-14b as a verification audit (validating existing citations) but did NOT add new citations for the newly-lifted BC content. 3 of 5 expected new BC anchors were missed: VP-DAEMON-001 §Post-7 semver, VP-DAEMON-002 §Post-7 numeric pid, VP-DAEMON-002 §Post-8 string-format. This asymmetry reveals SE-14b was codified as one-directional (VP→BC) but needs to be bidirectional (BC lift → VP citation update).

---

## Findings

### I-R92-1 — HIGH: VP-DAEMON-005 §Post-9 line 883 mis-pairs v1.18 with commit 2e24e09 (v1.19's commit)

**Severity:** HIGH
**Category:** Version-Commit Pair Integrity — Stale Version Reference

**Evidence:**

VP-DAEMON-005 §Post-condition 9 (line 883 area) contains a `Traces to:` or pin annotation that pairs PRD version `v1.18` with commit `2e24e09`. The commit `2e24e09` is PRD v1.19 (the F-R91 PO burst commit). PRD v1.18 is commit `3a18306`. This is a mis-matched version-commit pair — a recurrence of the partial-propagation pattern that SE-14b's verification sub-rule was intended to close.

**Root cause:** The F-R91 FV burst (VP v1.25 ba8eea4) propagated the PRD pin from v1.18 → v1.19 at the majority of sites but introduced a stale version label at VP-DAEMON-005 §Post-9, citing the new SHA (2e24e09) under the old version string (v1.18). This class of error is the reverse of the typical stale-SHA pattern: version string stale, SHA correct.

**Fix:** VP-DAEMON-005 §Post-9 citation must be updated from `PRD v1.18 (2e24e09)` to `PRD v1.19 (2e24e09)`. FV-only fix — no PRD change required.

---

### I-R92-3 — HIGH [pattern]: 4 VP §Harness location lines still say "(unit)" despite F-R88-5 §Mechanism relabel to integration-test

**Severity:** HIGH
**Category:** Partial-Fix Regression — §Harness Location Label vs §Mechanism Distribution

**Evidence:**

F-R88-5 (closed in VP v1.22 e4c1a1e, propagated through v1.23–v1.25) rewrote VP §Mechanism Distribution from uniform "unit-test 22" to the accurate taxonomy: "18 integration-test + 3 ast-audit + 1 compile-time-check." The §Mechanism Distribution block reflects the correct taxonomy in VP v1.25.

However, 4 individual VP §Harness entries (per-VP blocks) still carry the legacy `(unit)` location annotation in their `§Harness:` or `Location:` lines, despite the F-R88-5 sweep claiming all 22 VPs were updated:

- VP-DAEMON-001 §Harness: `tests/daemon_lifecycle.rs (unit)`
- VP-DAEMON-003 §Harness: `tests/session_lifecycle.rs (unit)`
- VP-AUTH-001 §Harness: `tests/auth.rs (unit)`
- VP-ENGINE-002 §Harness: `tests/engine_module.rs (unit)`

These should all read `(integration-test)` per F-R88-5's §Mechanism relabel. The F-R88-5 "preemptive intra-block sweep of all 22 VPs" in VP v1.22 was incomplete — these 4 sites escaped the sweep.

**Pattern significance:** This is a recurrence of the partial-fix pattern at the §Harness location line level. The F-R88-5 sweep updated §Mechanism Distribution (the summary block) but missed 4 body-level §Harness annotation sites. This parallels the GAP-R17-001 pattern (PRD pin propagation updated most but not all annotation sites).

**Fix:** Update 4 §Harness location annotations from `(unit)` to `(integration-test)`. FV-only fix — no PRD or arch change required.

---

### I-R92-5 — HIGH [pattern]: 3 VP probes (VP-DAEMON-001 §Post-7 + VP-DAEMON-002 §Post-7 + §Post-8) missed new BC-anchor citations from F-R91 lifts — SE-14b applied as verification not authoring

**Severity:** HIGH
**Category:** SE-14b Authoring Gap — BC Lift Without Corresponding VP Citation Addition

**Evidence:**

The F-R91 PO burst (PRD v1.19 2e24e09) lifted 5 BC content items:
1. BC-DAEMON-001 Postcondition 1 — added semver regex `^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$`
2. BC-DAEMON-002 Postcondition 1 — added pid≥1 integer constraint
3. BC-DAEMON-002 Postcondition 1 — added semver regex (same as BC-DAEMON-001)
4. BC-DAEMON-006 Invariant 1 — added 4 explicit field constraints (app_mode, last_app_mode, shutdown_reason, version)
5. BC-FACTORY-002 EC-061 — added empty-string current_cycle edge case

The F-R91 FV burst (VP v1.25 ba8eea4) was expected per SE-14b to add NEW BC-anchor citations to the corresponding VP probes. Verification of VP v1.25 shows:

- VP-DAEMON-006 §Post-10 → BC-DAEMON-006 §Invariant 1: **PRESENT** (correct)
- VP-FACTORY-002 §Post-7 → EC-061: **PRESENT** (correct)
- VP-DAEMON-001 §Post-7 semver probe: **MISSING** new `per BC-DAEMON-001 §Postcondition 1` citation for the semver regex constraint
- VP-DAEMON-002 §Post-7 numeric pid probe: **MISSING** new `per BC-DAEMON-002 §Postcondition 1` citation for the pid≥1 constraint
- VP-DAEMON-002 §Post-8 string-format probe: **MISSING** new `per BC-DAEMON-002 §Postcondition 1` citation for the semver regex constraint

3 of 5 expected new BC anchors were missed. The FV burst (ba8eea4) applied SE-14b as a verification audit — running the anchor-existence grep to confirm existing citations resolve — but did NOT identify the sites where NEW citations were needed (i.e., where BC content was freshly lifted and VP probes already existed testing the same constraint but without the new BC anchor citation).

**Root cause:** SE-14b codification (b15effb commit) defined the anchor verification sub-rule ("every VP §Post-condition that cites BC anchors MUST resolve") but did NOT define an authoring sub-rule ("when BC content is freshly lifted, identify VP probes that test the lifted constraint and ADD new BC-anchor citations"). This is the authoring gap: SE-14b only ensured existing citations were valid; it did not ensure new citations were created for newly-lifted BC elements.

**Fix:** VP v1.26 must add `per BC-DAEMON-001 §Postcondition 1` to VP-DAEMON-001 §Post-7, `per BC-DAEMON-002 §Postcondition 1` to VP-DAEMON-002 §Post-7, and `per BC-DAEMON-002 §Postcondition 1` to VP-DAEMON-002 §Post-8. SE-14b must be extended with the AUTHORING-mandatory sub-rule. FV-only fix.

---

## Observations (LOW — Process Gaps)

### O-R92-1 — LOW: VP v1.25 frontmatter boundary claim off by 16 lines (recurrence of O-R91-1)

**Severity:** LOW
**Category:** Frontmatter Precision — §Trace Boundary Claim

**Evidence:**

VP v1.25 frontmatter or §Trace states the file has N lines, but the actual line count is N+16 (or N-16). This is a recurrence of O-R91-1 (same class: VP frontmatter line-count claim does not match wc -l output). The off-by-16 delta suggests a structural block (likely a 16-line addition in the BC-lift content) was not reflected in the boundary claim update.

**Note for human gate:** This is a bookkeeping precision gap, not a semantic defect. The 16-line discrepancy indicates a VP structural addition that was not mirrored in the boundary metadata. Low severity; include in FV-only fix burst.

---

### O-R92-2 — LOW: PRD §7 RTM "Unit" vs VP "integration-test" divergence — pending intent verification

**Severity:** LOW
**Category:** Cross-Document Label Consistency — RTM Test Type Column vs VP §Mechanism Taxonomy

**Evidence:**

PRD §7 RTM Test Type column (for several BCs) uses "Unit" as the test-type label. VP §Mechanism Distribution (corrected in F-R88-5) uses "integration-test" for the same tests. This is a known consequence of F-R88-5's F-R88-5 §Mechanism relabel — the VP taxonomy was updated but the PRD §7 RTM Test Type column was not propagated.

**Intent question for human gate:** Should PRD §7 RTM Test Type column match VP file-layout taxonomy ("integration-test"), or should PRD §7 RTM retain the conceptual test-type label ("Unit" = unit-of-behavior test, even if implemented as Rust integration test)? These are two valid conventions that need a human decision:

- **(a)** Align to VP taxonomy: PRD §7 RTM Test Type → "Integration" for all `tests/` files. Requires PRD update.
- **(b)** Retain conceptual label: PRD §7 RTM Test Type stays "Unit" as conceptual scope; VP §Mechanism uses file-layout taxonomy. No PRD change; document the two-taxonomy convention.

This question cannot be answered by an FV agent without human input on the intended convention. Surfaced for human gate decision.

---

## Counter and Next Step

**Counter after R92:** 0/3 (unchanged). R92 FINDINGS (3 HIGH + 2 LOW) resets any counter advancement. Cons R31 CLEAN is overridden by R92 FAIL per D-047 strict policy.

**SE-14b extension required:** I-R92-5 reveals SE-14b must be extended with an AUTHORING-mandatory sub-rule. The authoring sub-rule closes the asymmetry between verification discipline (existing citations resolve) and authoring discipline (new citations added when BC lifts happen). Full extension language is recorded in `cycles/cycle-001/lessons.md §SE-14b` via the SE-14b extension appended in the v5.35 state update.

**Next:** FV-only fix-burst (VP v1.26) — fixes I-R92-1 (version-commit pair), I-R92-3 (4 §Harness (unit) labels), I-R92-5 (3 missing BC-anchor citations), O-R92-1 (boundary claim). O-R92-2 deferred to human gate decision.

**D-089 recorded.** 27 disciplines in force. SE-14b extended with AUTHORING-mandatory sub-rule.
