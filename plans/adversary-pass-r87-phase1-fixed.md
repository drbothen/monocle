---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.16 cd6541f + VP v1.20 f94c499 + arch v1.0.17 a798d51 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 20 (R87); post-F-R86 serial fix-burst snapshot"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T21:00:00Z
pass_number: 1
attempt: 20
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 HIGH + 1 MEDIUM + 2 LOW observations
---

# Adversarial Review R87 — Phase 1 (D-047 Strict, Pass 1 Attempt 20 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

- 1 HIGH: I-R87-1 — VP §Trace v1.20 Extension 16 audit table dropped the v1.19 pre-existing row for VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002. The bidirectional pair EXISTS in the VP body (both VPs cite each other in their §Mechanism blocks), but the Extension 16 mandatory audit table in §Trace v1.20 only enumerates 10 citation entries and does not include a row for this pair. This is a third-order META failure: the Extension 16 audit MECHANISM (whose purpose is to enumerate all cross-property/cross-check citations and verify their bidirectionality) has itself an incomplete enumeration. The audit table omitted a v1.19 pre-existing row that was present in prior bursts' audit tables, meaning the audit table regressed (dropped a row) rather than accumulated (added rows or held). SE-16c (canonical Extension 16 audit grep target) is the codification candidate to close this.

- 1 MEDIUM: I-R87-2 — VP frontmatter Fix 6 count narrative ambiguous. The §Trace v1.20 fix-count narrative for Fix 6 reads `(61 + 5 + 6)` in a position where the total is stated as 66 sites swept. However, the arithmetic `61 + 5 + 6 = 72`, not 66. A reader parsing the inline arithmetic would derive 72; the narrative claims 66. The actual count is 66 (61 pre-existing + 5 new SE-16a additions = 66 total unique citations), and the `+ 6` in the expression may refer to a distinct audit sub-category rather than an addend to the sum. This internal arithmetic ambiguity in the §Trace count narrative is a fabrication-pattern finding at the §Trace narrative axis (Extension 9 class: §Trace prose arithmetic vs §Trace detailed table internal consistency).

**Observations (process-relevant):**

- O-R87-1 — VP §Trace v1.19 "Pre-burst line" column (historical-record observation, informational). The §Trace v1.19 table includes a "Pre-burst line" column whose values record line numbers in the v1.18 artifact snapshot. These values are HISTORICAL RECORD only — they reflect a snapshot state that no longer corresponds to the current artifact. This is architecturally appropriate (§Trace preserves historical evidence), not a defect. However, fresh-context agents reading §Trace v1.19 may misinterpret "Pre-burst line" values as CURRENT line numbers and attempt to re-verify them against the v1.20 artifact, causing false-positive findings. This observation is LOW severity: the column header clearly identifies it as "Pre-burst" scope, satisfying the historical-record-only intent. No fix required; noted here for SE-16c codification context.

- O-R87-2 — SE-16c codification candidate (canonical Extension 16 audit grep target). The I-R87-1 HIGH finding reveals that Extension 16 audit table enumeration is performed manually each burst, creating vulnerability to row-dropping between bursts. The canonical fix is to codify a grep target that mechanically generates the Extension 16 audit table rows from the VP body, rather than relying on burst-author re-enumeration. Proposed canonical grep: `grep -nE "[Cc]ross-property|[Cc]ross-check" .factory/specs/verification-properties.md | grep -v "§Trace"`. This observation is routed to the state-manager for SE-16c codification in cycle lessons. This is the 4th orchestration/discipline-protocol axis codification (Extension 15 = cross-layer dispatch; Extension 16 = mandatory backfill; SE-16a = in-burst-added; SE-16b = timestamp monotonicity; SE-16c = canonical grep target).

---

## I-R87-1 — HIGH: VP §Trace v1.20 Extension 16 audit table dropped v1.19 pre-existing row (VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002)

### Description

The VP v1.20 §Trace Extension 16 audit table enumerates 10 bidirectional citation rows. The F-R86 fix-burst added one new citation (C-R86-1: VP-DAEMON-004 §Mech 5 reciprocation). The prior v1.19 audit table (in the F-R85 fix-burst SE-15d backfill section) included a row for VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002.

The v1.20 audit table contains 10 rows. If this pair from v1.19 is counted, the expected total would be at least 11 rows (10 preserved + any net-new from F-R86). The v1.20 audit table appears to have re-enumerated the pairs from scratch (as a manual audit) rather than carrying forward the v1.19 table and appending new rows. During this re-enumeration, the VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002 pair was dropped.

### Evidence

The bidirectional pair EXISTS in the VP v1.20 body:
- VP-DAEMON-004 §Post-condition 7 (runtime-dir permission security context) contains a cross-property citation to VP-AUTH-002.
- VP-AUTH-002 §Mechanism contains a cross-property citation back to VP-DAEMON-004.

Both citations are present in the body text. The audit table in §Trace v1.20, which is supposed to enumerate and verify ALL cross-property/cross-check citations, does not include a row for this pair.

### Impact

This is a third-order META failure:
1. First order: a spec content gap (missing or incorrect citation in the body).
2. Second order: an audit mechanism gap (audit table does not enumerate a known citation pair).
3. Third order: the audit mechanism discipline (Extension 16) that was codified to PREVENT second-order gaps has itself a gap — its enumeration process is manual and therefore vulnerable to row-dropping.

The fix requires both: (a) adding the missing row to the §Trace v1.20 audit table (FV VP v1.21), AND (b) codifying SE-16c to make the enumeration mechanically reproducible (state-manager, cycle lessons).

### Fix routing

- **FV (formal-verifier):** VP v1.21 — add missing VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002 row to Extension 16 audit table in §Trace v1.20 section.
- **State-manager:** SE-16c codification in cycle lessons.

---

## I-R87-2 — MEDIUM: VP frontmatter Fix 6 count narrative `(61 + 5 + 6)` ambiguous

### Description

VP v1.20 §Trace Fix 6 count narrative contains the expression `(61 + 5 + 6)` in a context where the stated total is 66 sites. The arithmetic `61 + 5 + 6 = 72 ≠ 66`. This internal arithmetic ambiguity is a fabrication-pattern finding at the §Trace prose internal-consistency axis (Extension 9 class).

### Evidence and disambiguation

The intended decomposition is likely:
- 61: pre-existing cross-property/cross-check citation sites in VP v1.19 before F-R86 burst
- 5: in-burst-added SE-16a citations introduced in F-R86 (the C-R86-1 reciprocation + 4 others from the SE-16a burst-end re-audit)
- Total: 61 + 5 = 66

The `+ 6` term may refer to a distinct sub-category (e.g., 6 §Mechanism cross-check entries that were audited in a separate step), making the expression `(61 + 5) + 6-audit-step` rather than `61 + 5 + 6 = sum`. However, the notation `(61 + 5 + 6)` without sub-categorization labels produces arithmetic ambiguity.

### Impact

A fresh-context adversary or auditor reading the §Trace count summary would derive 72 from the inline arithmetic, then search for 72 citation entries in the audit table and find only 10 (or the expanded count if all rows are enumerated). This creates a false-positive discrepancy that wastes verification effort. More critically, if the actual count IS 66 but the arithmetic parses as 72, the §Trace is making a false claim about the audit scope.

### Fix routing

- **FV (formal-verifier):** VP v1.21 — rewrite the §Trace Fix 6 count narrative expression to unambiguously state the decomposition. Preferred form: `(61 pre-existing + 5 in-burst-added SE-16a = 66 total)` with the `+ 6` sub-category, if applicable, labeled explicitly and excluded from the primary sum.

---

## O-R87-1 — LOW: VP §Trace v1.19 "Pre-burst line" column (historical-record, informational)

### Description

The §Trace v1.19 table columns include "Pre-burst line" whose values are line numbers from the VP v1.18 snapshot. These values are historical record — they do not reflect current artifact state.

### Disposition

No fix required. The column header clearly marks the scope as "Pre-burst," satisfying the historical-record-only intent per Extension 9 (§Trace preserves historical evidence). Fresh-context agents should not attempt to re-verify "Pre-burst line" values against the current artifact. This observation is noted here for SE-16c context: the canonical grep target approach (SE-16c) would eliminate the need for "Pre-burst line" columns in future audit tables, since the grep derives current state rather than recording historical state.

---

## O-R87-2 — SE-16c codification candidate (process-gap observation)

### Description

I-R87-1 reveals that the Extension 16 audit table is produced by manual re-enumeration each burst. The burst author must scan the VP body for all cross-property/cross-check citation pairs and reconstruct the table from memory or visual inspection. This manual process is vulnerable to:

1. Row-dropping between bursts (the I-R87-1 defect: VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002 dropped from v1.20 table)
2. Row-fabrication (audit-row with incorrect pair members)
3. Incomplete enumeration (pairs that exist in the body but were not observed)

The canonical fix is SE-16c: codify a grep target that mechanically produces all citation pair candidates, eliminating the re-enumeration risk.

### Proposed SE-16c canonical grep target

```bash
grep -nE "[Cc]ross-property|[Cc]ross-check" .factory/specs/verification-properties.md | grep -v "§Trace"
```

The `grep -v "§Trace"` exclusion filters out §Trace narrative entries, which contain historical-snapshot citations that reference past burst states rather than current body state.

For each grep hit, the audit table MUST include a row with:
- Site line number
- From-VP citation
- To-VP target
- Reciprocation status (NEW this burst / HOLDING from v1.X / overlap with row N)

This converts the Extension 16 audit from "burst author re-enumerates manually" (vulnerable) to "burst author runs canonical grep and enumerates every match" (mechanically complete).

### Routing

State-manager: codify SE-16c in `.factory/cycles/cycle-001/lessons.md` as the 4th orchestration/discipline-protocol sub-extension axis. This is the SE-16c process-gap closure.

---

## Closure Verification

### F-R86 closure verification (items that R87 was checking)

| Finding | Closure Status |
|---------|----------------|
| C-R86-1 VP-DAEMON-005 → VP-DAEMON-004 §Mech 5 unidirectional | CLOSED — VP v1.20 added reciprocal citation to VP-DAEMON-004 §Mech 5 |
| I-R86-1 VP-PROTO-002 stale PRD v1.10 (3 sites) | CLOSED — VP v1.20 §Trace documents 3-site sweep (lines 1867, 2279, 2369) |
| I-R86-2 PRD §Trace v1.15 count 4 vs 5 | CLOSED — PRD v1.16 §Trace updated |
| O-R86-1 VP clock-skew documentation | CLOSED — VP v1.20 §Trace includes timestamp monotonicity correction |
| O-R86-2 SE-16a scope gap | CLOSED — SE-16a codified; VP v1.20 burst-end re-audit section PASS |
| SE-16a first application | CONFIRMED PASS — VP v1.20 §Trace burst-end re-audit section present |
| SE-16b monotonicity check | CONFIRMED PASS — v1.20 timestamp 20:30:00Z ≥ v1.19 20:00:00Z |
| §Purpose META 7th-attempt | CONFIRMED PRESENT — VP v1.20 §Purpose updated |

### NEW findings requiring FV fix-burst

| ID | Severity | Fix target |
|----|----------|------------|
| I-R87-1 | HIGH | FV VP v1.21 — missing Extension 16 audit table row (VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002) |
| I-R87-2 | MEDIUM | FV VP v1.21 — §Trace Fix 6 count narrative arithmetic ambiguity `(61 + 5 + 6)` |

### State-manager codification

| ID | Action |
|----|--------|
| O-R87-2 | SE-16c codification in cycle-001 lessons.md |

---

## Lens rotation

This pass examined:
1. **Extension 16 audit table structural completeness** — I-R87-1 discovered row-dropping from v1.19 → v1.20
2. **§Trace prose arithmetic consistency** — I-R87-2 discovered count narrative ambiguity
3. **Closure verification for F-R86** — all 6 F-R86 items confirmed CLOSED
4. **SE-16a/b first application confirmation** — both PASS
5. **META-class audit-mechanism coverage** — O-R87-2 routes to SE-16c codification

Fresh-context adversary lens for R88 should focus on:
- VP v1.21 §Trace audit table completeness (does the corrected table include the re-added VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002 row AND apply the SE-16c canonical grep target discipline?)
- VP v1.21 §Trace Fix 6 count narrative clarity (is `(61 + 5 + 6)` unambiguously rewritten?)
- SE-16c first application evidence (does VP v1.21 §Trace include a `### SE-16c canonical grep transcript` section showing the grep output used to generate the audit table?)
- PRD v1.16 — no changes required for R87 closures (no PRD change needed)

---

## Novelty assessment

R87 finds are META-4 class (fourth-order meta-recursive finding):
- META-1 (F-R80): fabrication at the Extension 3 audit-row level (spec content about audit behavior was fabricated)
- META-2 (F-R84): orchestration parallel-dispatch protocol gap (the ORCHESTRATOR behavior was incorrect)
- META-3 (F-R86/SE-16a): in-burst-added citation not audited by the burst-end re-audit step (the audit procedure itself had a scope gap)
- META-4 (R87/I-R87-1): the audit table that enumerates the audit procedure's targets dropped a row (the enumeration mechanism for the audit procedure itself is incomplete)

Each META level operates at a strictly higher abstraction level than the previous. At META-4, the finding is not about spec content, not about orchestration protocol, not about audit-step scope — it is about the ENUMERATION MECHANISM that drives the audit step. SE-16c (canonical grep target) converts this enumeration from manual to mechanical, closing the META-4 axis at the process level.

The substantive spec content has been stable since R82 (when R82 was CLEAN, counter advanced to 1/3). R83–R87 findings have all been at the META-2, META-3, or META-4 axes — orchestration protocol, audit scope gaps, and enumeration mechanism gaps — not content defects. This asymptotic pattern is consistent with the human decision at T-27 to consider option (b) Convergence-with-Documented-Residuals.

---

## Routing summary

| Finding | Severity | Route | Action |
|---------|----------|-------|--------|
| I-R87-1 | HIGH | formal-verifier | VP v1.21: add missing VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002 row to Extension 16 audit table |
| I-R87-2 | MEDIUM | formal-verifier | VP v1.21: rewrite §Trace Fix 6 count narrative to unambiguous form |
| O-R87-1 | LOW (informational) | no-action | Historical-record column; no fix required |
| O-R87-2 | LOW (process-gap) | state-manager | SE-16c codification in cycle-001 lessons.md |

**No PRD change required for R87 closures.** FV-only fix-burst (VP v1.21) next.

**Counter:** Stays at 0/3. D-079 records R87 dispositions.
