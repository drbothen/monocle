---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.21 0f124a9 + VP v1.28 a6a0976 + arch v1.0.21 42504b4 + manifest v1.1.13 42504b4; D-047 strict pass 1 attempt 28 (R95); post-F-R94 serial fix-burst snapshot; META LENS — §Trace audit-row integrity recursive + arch↔PRD round-trip"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T22:52:55Z
pass_number: 1
attempt: 28
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 4 MEDIUM + 4 LOW observations
lens_class: META (§Trace audit-row integrity)
---

# Adversary Pass R95 — Phase 1 Spec Review

**Policy:** D-047 strict (0 findings of any severity for 3 consecutive passes required)
**Attempt:** 28 (pass 1 of current cycle)
**Lens:** META — §Trace audit-row integrity recursive + arch↔PRD round-trip
**Input artifacts:** PRD v1.21 (0f124a9) + VP v1.28 (a6a0976) + arch v1.0.21 (42504b4) + manifest v1.1.13 (42504b4)
**Counter before:** 0/3
**Counter after:** 0/3 (FINDINGS — counter stays)

---

## Verdict: FINDINGS (4 MEDIUM + 4 LOW)

Counter stays at 0/3. Consistency review R34 was CLEAN.

Three of four MEDIUM findings (C-R95-1, C-R95-2, C-R95-4) share the same defect class: VP v1.28 §Trace contained stale audit metadata — line-number citations, awk boundary expressions, and frontmatter count claims were authored mid-burst against partial-edit state and not re-validated against final post-burst file state.

---

## MEDIUM Findings

### C-R95-1 — VP §Trace SE-16c++ supplementary grep audit-table: 7 of 10 L-number citations off-by-1 to off-by-17 from actual file state

**Severity:** MEDIUM
**Artifact:** verification-properties.md v1.28 (a6a0976)
**Location:** §Trace SE-16c++ supplementary grep audit-table rows
**Finding:** The audit-table in §Trace v1.28 cites line numbers L-N for 10 audit rows. Verification against the current file state reveals 7 of 10 L-number citations are incorrect: drifts range from off-by-1 (single-line insertion earlier in the file) to off-by-17 (multiple-line block additions). The line numbers were captured when the §Trace section was authored during the burst, before all subsequent edits to the verification-properties.md body were finalized. The §Trace was not re-validated after the file's final state was settled.

**Pattern class:** Stale audit metadata — mid-burst §Trace authoring without final-state revalidation.
**SE-17c candidate:** This finding, combined with C-R95-2 and C-R95-4, establishes the SE-17c codification: final-state line-number revalidation discipline.

---

### C-R95-2 — VP §Trace SE-16c++ awk boundary `$1 < 3086` hardcoded; actual §Trace heading at line 3108 (22-line drift)

**Severity:** MEDIUM
**Artifact:** verification-properties.md v1.28 (a6a0976)
**Location:** §Trace SE-16c++ supplementary grep audit — awk filter clause
**Finding:** The §Trace SE-16c++ supplementary grep command includes `awk '$1 < 3086'` to exclude §Trace-internal citations. The actual `## §Trace` heading in v1.28 is at line 3108, not 3086. The 22-line delta is attributable to body additions made after the awk boundary was set during §Trace authoring. The boundary was not re-derived after all other edits finalized.

**Correct derivation:** `grep -n "^## §Trace" .factory/specs/verification-properties.md | head -1 | cut -d: -f1` — this command must be run at burst-finalization time, not during §Trace authoring.

**Pattern class:** Stale awk boundary — hardcoded from mid-burst state rather than derived from final file state.
**SE-17c candidate:** Same defect class as C-R95-1 and C-R95-4. Awk boundary derivation must occur at burst-finalization time.

---

### C-R95-3 — VP line 910 stale `per C-R91-1` attribution; current pin is F-R94 (v1.21)

**Severity:** MEDIUM
**Artifact:** verification-properties.md v1.28 (a6a0976)
**Location:** Line 910, §Post-condition citation block
**Finding:** VP line 910 contains `per C-R91-1` attribution. C-R91-1 was the fabricated-anchor defect from R91; the attribution style implies this citation was introduced as a C-R91-1 remediation artifact and traces to PRD v1.19 content. The current canonical PRD pin is v1.21 (0f124a9) introduced by F-R94. The citation survives 3 fix-bursts (F-R91 → F-R92 → F-R93 → F-R94) without being updated to the current-version attribution form.

**Impact:** Stale attribution leaves auditors uncertain whether the cited content is current or historical. Under Extension 15 SERIAL cascade, every PO PRD bump triggers FV VP propagation; the attribution form at line 910 should reflect the most recent normative pin, not a historical closure reference.
**Routing:** FV-only fix — VP v1.29 update line 910 attribution to current pin form.

---

### C-R95-4 — VP §Trace frontmatter "8 sites" count claim mismatches audit-table 10 rows

**Severity:** MEDIUM
**Artifact:** verification-properties.md v1.28 (a6a0976)
**Location:** §Trace frontmatter narrative count claim vs. audit-table body
**Finding:** The §Trace frontmatter narrative states "8 sites surfaced" (or equivalent count claim N=8). The audit-table body contains 10 rows. The discrepancy is a count inconsistency: the frontmatter count was authored when 8 sites had been identified; the audit-table was expanded to 10 rows as additional sites were discovered, but the frontmatter count was not updated to match.

**Pattern class:** Frontmatter count vs. body row count mismatch. Same class as I-R87-2 (Fix 6 count narrative `61+5+6` ambiguous). SE-17c rule (c) closes this: every frontmatter narrative count claim MUST equal the audit-table row count in the body, verified at burst-finalization time.
**SE-17c candidate:** Count consistency sub-rule.

---

## LOW Observations (informational — do not advance counter)

### I-R95-1 — PRD v1.20/v1.21 dual-version pattern in §Trace historical-snapshot rows

**Severity:** LOW (informational)
**Artifact:** prd.md v1.21 (0f124a9)
**Location:** §Trace historical-snapshot rows
**Observation:** §Trace contains rows that cite both v1.20 and v1.21 version labels in adjacent historical entries. The dual-version pattern is technically correct (v1.20 → v1.21 serial chain is the F-R94 closure) but creates visual ambiguity for fresh-context readers who may interpret the dual-version rows as a consistency gap rather than an intentional changelog sequence. No defect — the content is accurate — but the framing could be clarified with an inline note distinguishing the two-step PRD update chain.
**Routing:** Optional polish; no fix required for D-047 counter advancement.

### O-R95-1 — VP §Purpose cross-artifact claim references pre-C-R95-3 state

**Severity:** LOW (informational)
**Observation:** §Purpose cross-artifact consistency claim in VP v1.28 is accurate at v1.21 PRD pin level but will need a 16th-attempt update when VP v1.29 is produced (SE-17c application will be §Purpose META 16th-attempt). Not a defect in current v1.28 state; noting for FV dispatch.
**Routing:** FV VP v1.29 §Purpose META 16th-attempt application.

### O-R95-2 — SE-17c codification is a third META audit-discipline sub-extension

**Severity:** LOW (process-gap observation)
**Observation:** SE-17c (final-state line-number revalidation) extends the Extension 17 sweep-evidence discipline. SE-17a covered multi-line grep patterns; SE-17b covered self-verification before §Trace assertion; SE-17c covers final-state revalidation at burst-finalization time. The discipline class matures via observed defect patterns: C-R95-1 (L-numbers), C-R95-2 (awk boundary), C-R95-4 (count claim) are three distinct forms of the same root cause (mid-burst authoring without final-state revalidation). SE-17c should be codified with a 5-step application order.
**Routing:** State-manager codification in lessons.md + STATE.md Critical Hook Lessons.

### O-R95-3 — arch↔PRD round-trip secondary lens: all 11 sampled BCs resolve; no findings

**Severity:** LOW (informational — secondary lens CLEAN)
**Observation:** The secondary lens for this pass (arch↔PRD round-trip: verify BC text in PRD matches arch SS-daemon-lifecycle.md behavioral contracts) was applied to all 11 sampled BCs (BC-RING-001, BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-AUTH-001/002). All 11 resolve without substantive content discrepancy. Manifest↔BC pin coherence lens was also clean. Strong empirical signal: the substantive content layer is converged; no content defects were found on the secondary lens. Remaining defects are exclusively META audit-discipline.
**Routing:** No fix required. Record in STATE.md §Surfaced for Human Gate Decision as empirical convergence evidence.

---

## Consistency Review R34: CLEAN

Consistency review R34 (companion to R95) returned CLEAN — 0 blocking findings. Counter evaluation: R95 FAIL (4 MEDIUM findings) overrides cons R34 CLEAN; counter stays at 0/3 per D-047 strict.

---

## Disposition

| ID | Class | Severity | Disposition |
|----|-------|----------|-------------|
| C-R95-1 | VP §Trace audit-table stale L-numbers | MEDIUM | FV-only fix-burst — VP v1.29 |
| C-R95-2 | VP §Trace awk boundary 22-line drift | MEDIUM | FV-only fix-burst — VP v1.29 |
| C-R95-3 | VP line 910 stale C-R91-1 attribution | MEDIUM | FV-only fix-burst — VP v1.29 |
| C-R95-4 | VP §Trace frontmatter count vs audit-table row mismatch | MEDIUM | FV-only fix-burst — VP v1.29 |
| I-R95-1 | PRD dual-version pattern informational | LOW | No fix required |
| O-R95-1 | VP §Purpose pre-C-R95-3 state | LOW | FV VP v1.29 §Purpose META 16th-attempt |
| O-R95-2 | SE-17c codification candidate | LOW | State-manager codification |
| O-R95-3 | arch↔PRD secondary lens CLEAN (11 BCs) | LOW | No fix required |

**Next burst:** FV-only fix-burst — VP v1.29 (5 fixes: C-R95-1 through C-R95-4 + O-R95-1 §Purpose META 16th-attempt). Apply SE-17c at burst-finalization. No PRD or arch changes required.
