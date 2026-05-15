---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.15 80bfe86 + VP v1.19 022ce3c + arch v1.0.17 a798d51 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 19 (R86); post-F-R85 serial fix-burst snapshot"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T18:00:00Z
pass_number: 1
attempt: 19
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 CRITICAL + 1 HIGH + 1 MEDIUM + 2 LOW observations
---

# Adversarial Review R86 — Phase 1 (D-047 Strict, Pass 1 Attempt 19 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

- 1 CRITICAL: C-R86-1 — VP-DAEMON-005 §Post-cond 4 → VP-DAEMON-004 §Mech 5: unidirectional in-burst-added citation. The F-R85 fix-burst (VP v1.19) introduced a new citation at VP-DAEMON-005 §Post-cond 4 (lines 712-723) referencing VP-DAEMON-004 §Mech 5. This citation was described in §Trace v1.19 as a "reciprocation" (part of the SE-15d 5-reciprocation backfill). However, VP-DAEMON-004 §Mech 5 (lines 473-477) contains NO reciprocal reference back to VP-DAEMON-005. The SE-15d reciprocation audit table in §Trace v1.19 covered only the 9 pre-existing cross-property/cross-check citations; it did NOT audit the new citations introduced in the same burst. This is a recursive Extension 16 failure: the audit scope verified pre-existing citations but did not cover in-burst-added citations, leaving the just-added VP-DAEMON-005 → VP-DAEMON-004 citation unverified for bidirectionality at burst-end.

- 1 HIGH: I-R86-1 — VP-PROTO-002 §Harness location line 1867 cites stale PRD v1.10 (8feecad). Current PRD version is v1.15 (80bfe86). This citation has survived 5+ PRD version bumps (v1.11 → v1.12 → v1.13 → v1.14 → v1.15) without correction. The F-R85 sweep methodology (wrap-continuation regex + multi-line grep) consistently missed this site, suggesting the citation appears in a structural context (§Harness location sub-section header or annotation comment) rather than a standard test-name annotation form.

- 1 MEDIUM: I-R86-2 — PRD §Trace v1.15 backfill summary count contradiction. The §Trace narrative for PRD v1.15 (commit 80bfe86) states the backfill sweep covered 12 NFR rows with "4 new VP probe cites added." However, the §Trace table lists 5 rows receiving new VP probe citations (NFR-004, NFR-005, NFR-007, NFR-010, NFR-012). This is a 4-vs-5 row count discrepancy internal to the §Trace v1.15 narrative — the prose summary count does not match the detailed table. This is a fabrication-pattern finding at the §Trace narrative axis (Extension 9 class: §Trace prose vs §Trace table internal consistency).

**Observations (process-relevant):**

- O-R86-1 — VP v1.18 → v1.19 timestamp regression (informational). VP v1.18 was timestamped `2026-05-16T08:00:00Z` (future-dated relative to session date 2026-05-15). VP v1.19 corrected to `2026-05-15T20:00:00Z`. This correction is documented in §Trace v1.19 items (f)/(g) with rationale (session-date-drift correction). However, the §Trace does not explicitly state that v1.18's timestamp was future-dated. The timestamp regression (`2026-05-15T20:00:00Z` < `2026-05-16T08:00:00Z`) breaks monotonic ordering: tooling that sorts by frontmatter timestamp would order v1.19 BEFORE v1.18. Per SE-16b (to be codified), timestamp corrections require explicit monotonicity-correction documentation. This is a documentation polish per OBS-R25-001 from cons R25 — non-blocking, but should be applied in the F-R86 fix-burst.

- O-R86-2 — Extension 16 in-burst-added citation audit scope gap (META pattern). The F-R85 fix-burst audit table (§Trace v1.19 SE-15d backfill section) covered 9 pre-existing cross-property/cross-check citations. At burst-end, when the FV agent added the VP-DAEMON-005 → VP-DAEMON-004 citation as one of the 5 new reciprocations, no re-audit pass verified the newly added citation for bidirectionality. Extension 16's mandatory backfill sweep rule (§3: "explicit backfill section must list each pre-existing site and its disposition") was applied to pre-existing sites; however, Extension 16 does NOT currently require a re-audit pass at burst-end that covers in-burst-added citations. This is a scope gap in Extension 16's rules. SE-16a (to be codified) closes this: Extension 16's mandatory backfill sweep MUST include a burst-end re-audit step covering ALL citations added in the same burst, not just pre-existing ones. The C-R86-1 finding is a direct consequence of this scope gap.

---

## C-R86-1 — CRITICAL: VP-DAEMON-005 → VP-DAEMON-004 §Mech 5 unidirectional in-burst-added citation

### Description

VP v1.19 (F-R85 fix-burst) introduced a new citation at VP-DAEMON-005 §Post-cond 4 (lines 712-723). The §Trace v1.19 SE-15d backfill section describes this as one of 5 SE-15d reciprocations applied in the burst. The citation text reads (approximately):

```
cross-property with VP-DAEMON-004 §Mech 5 (runtime_dir resolution chain; confirms daemon startup precondition)
```

VP-DAEMON-004 §Mech 5 (lines 473-477) covers the runtime_dir resolution chain mechanism. Examination of VP-DAEMON-004 §Mech 5 confirms: NO reciprocal citation to VP-DAEMON-005 exists there. VP-DAEMON-004 §Mech 5 does not cite VP-DAEMON-005 §Post-cond 4, does not reference VP-DAEMON-005 by ID, and contains no `cross-property` or `cross-check` annotation pointing back to VP-DAEMON-005.

### Root cause

The SE-15d backfill audit table in §Trace v1.19 covered the 9 pre-existing cross-property/cross-check citation pairs identified BEFORE the burst. The 5 new citations added AS PART OF the F-R85 burst's SE-15d remediation were NOT re-audited at burst-end for bidirectionality. The FV agent added "VP-DAEMON-005 → VP-DAEMON-004 §Mech 5" to VP-DAEMON-005 but did not correspondingly add "VP-DAEMON-004 §Mech 5 → VP-DAEMON-005" to VP-DAEMON-004. The §Trace narrative describes the citation as a "reciprocation" — implying VP-DAEMON-004 already cited VP-DAEMON-005 — but this is incorrect. The new citation is one-directional.

### Why Extension 16 did not catch this

Extension 16's rule #3 requires: "Every §Trace narrative for a burst that introduces a new Extension/SE codification must include a section titled `### Backfill sweep: Extension N / SE-Nx pre-existing sites` listing each pre-existing site and its disposition." The key phrase is "pre-existing sites." VP v1.19's §Trace SE-15d backfill section correctly listed the 9 pre-existing citation pairs. But the 5 NEW pairs added by the same burst are not "pre-existing" — they are in-burst-added. Extension 16 currently has no rule requiring that in-burst-added citations be verified for bidirectionality at burst-end.

### Impact

CRITICAL severity because: (1) the finding is described as a "reciprocation" in §Trace v1.19 — the §Trace audit itself is incorrect (fabrication-pattern at §Trace axis); (2) the VP-DAEMON-005 ↔ VP-DAEMON-004 traceability chain is broken at §Mech 5; (3) SE-16a is required to prevent structural recurrence.

### Required fix

**Route: formal-verifier → VP v1.20**

1. Add reciprocal citation to VP-DAEMON-004 §Mech 5: `cross-property with VP-DAEMON-005 §Post-cond 4 (runtime_dir mode invariant; post-condition constraint on this mechanism's output)`.
2. Verify all 4 other in-burst-added citations from F-R85 are bidirectional (SE-16a application).
3. §Trace v1.20 must include: `### Burst-end re-audit: in-burst-added SE-15d citations` with grep evidence for each of the 5 new pairs.

---

## I-R86-1 — HIGH: VP-PROTO-002 §Harness location line 1867 stale PRD v1.10 citation

### Description

VP-PROTO-002 contains a citation at line 1867 (in the §Harness location subsection) that references `PRD v1.10 (8feecad)`. The current PRD version is v1.15 (80bfe86). This citation has remained stale across 5 consecutive PRD version bumps:

- PRD v1.10 → v1.11 (F-R77 chain): citation NOT updated
- PRD v1.11 → v1.12 (F-R79 chain): citation NOT updated
- PRD v1.12 → v1.13 (F-R83 chain / F-R84 serial burst): citation NOT updated
- PRD v1.13 → v1.14 (F-R84 PO burst): citation NOT updated
- PRD v1.14 → v1.15 (F-R85 PO burst): citation NOT updated

### Root cause

The sweep methodology employed in F-R84 and F-R85 bursts used grep patterns optimized for:
- Single-line `PRD v1.12` citations in test-name annotation form
- Multi-line wrap-continuation patterns with `v1.X` on a continuation line after a hyphen/paren

VP-PROTO-002 §Harness location subsection uses a citation form embedded in a structural annotation (e.g., `### Harness location (per PRD v1.10 §4.2)` or equivalent heading/label form) rather than the standard test-name annotation pattern. This structural form escaped all prior sweep regexes.

### Impact

HIGH severity because: (1) VP-PROTO-002 is a high-traffic verification property for the core protocol surface; (2) a fresh-context implementer following the §Harness location citation to PRD v1.10 §4.2 would read stale content; (3) this demonstrates a systematic sweep-pattern blind spot: heading-embedded and label-embedded PRD citations are not covered by current grep patterns.

### Required fix

**Route: formal-verifier → VP v1.20**

1. Update VP-PROTO-002 §Harness location line 1867 citation from PRD v1.10 (8feecad) to PRD v1.15 (80bfe86).
2. Expand the wrap-continuation grep pattern to include heading-embedded and structural-label-embedded citation forms.
3. Apply expanded grep to all 22 VP files and document in §Trace v1.20 `### Harness-location citation sweep` with grep -nE transcripts.

---

## I-R86-2 — MEDIUM: PRD §Trace v1.15 backfill summary count contradiction (4 vs 5 rows)

### Description

PRD v1.15 (commit 80bfe86) §Trace narrative for the F-R85 PO burst states (approximately):

> "Backfill sweep: SE-15c pre-existing sites — 12 NFR rows audited; **4 new VP probe cites added** (NFR-004, NFR-005, NFR-010, and NFR-012 already compliant)"

However, the detailed §Trace table lists these rows receiving new VP probe citations:
- NFR-004: new VP probe cite added
- NFR-005: new VP probe cite added
- NFR-007: new VP probe cite added
- NFR-010: new VP probe cite added
- NFR-012: already had VP probe cite (compliant — no change)

This is 5 rows receiving new citations (NFR-004, NFR-005, NFR-007, NFR-010) versus the prose summary claiming 4. The count discrepancy is internal to §Trace v1.15 — the prose summary does not match the detailed table row-by-row result.

### Why this matters

Extension 9 (§Coverage Matrix footer + closure-chain narrative audit) covers three-way consistency between §Coverage Matrix footer, §Trace closure narrative, and §References item 1 lineage. This finding is in the §Trace internal consistency axis: the §Trace prose summary count must agree with the §Trace table. This is the same fabrication-pattern class (Extension 9 axis) — the difference is the contradiction is internal to §Trace rather than between §Trace and §Coverage Matrix.

### Required fix

**Route: product-owner → PRD v1.16**

1. Reconcile §Trace v1.15 prose summary count with §Trace table. Determine whether 4 or 5 rows received new VP probe cites and correct the prose.
2. If NFR-007 did receive a new cite, update prose to "5 new VP probe cites." If it did not, remove NFR-007 from the table.
3. Apply Extension 9 three-way consistency check to PRD v1.15 §Trace, §Coverage Matrix footer (if any), and §References item 1.

---

## Closure Verification — F-R85 findings (confirmatory)

All 4 substantive findings from R85 are verified CLOSED in VP v1.19 + PRD v1.15:

| Finding | Status | Evidence |
|---------|--------|----------|
| F-R85-CRIT-1 VP-DAEMON-004 line 441 anchor mis-cite (non-existent Post-condition 6) | CLOSED | VP v1.19: VP-DAEMON-002 Post-condition 6 was LIFTED (now exists); VP-DAEMON-004 line 441 citation is now valid |
| F-R85-IMP-1 6 wrap-continuation `PRD v1.12` citations | CLOSED | VP v1.19: all 6 sites updated (5 listed in R85 + 1 additional at line 568 found via Extension 16 backfill) |
| F-R85-IMP-2 NFR-004/005/010 missing SE-15c VP probe cites | CLOSED | PRD v1.15: NFR-004, NFR-005, NFR-010 (and NFR-007) §Validation Method cells updated with VP probe cites; 12-row backfill sweep documented |
| F-R85-IMP-3 SE-15d cross-property/cross-check unidirectional pairs | CLOSED | VP v1.19: 5 reciprocation sites applied; §Trace SE-15d backfill section documents 9 pre-existing pairs + 5 new pairs |

F-R80 META closure (Extension 13 machine-greppable evidence requirement) CONTINUES HOLDING. VP v1.19 §Trace includes inline grep transcripts for the SE-15d backfill sweep. Extension 16 (codification-protocol mandatory backfill sweep) is PROVEN EFFECTIVE — the F-R85 burst applied Extension 16 end-to-end including mandatory backfill sections for both SE-15c and SE-15d. The one gap (C-R86-1) is at the NEW in-burst-added citation axis — not a recurrence of the pre-existing codification-without-backfill axis that Extension 16 closed.

---

## Consistency Round R25 — CLEAN

Consistency round R25 ran concurrently with R86. Result: **CLEAN — 0 blocking findings.**

**OBS-R25-001 (LOW — documentation polish, non-blocking):** VP §Trace v1.19 items (f)/(g) document the v1.18 → v1.19 timestamp correction with rationale but do not explicitly state that v1.18's timestamp `2026-05-16T08:00:00Z` was future-dated relative to the session date `2026-05-15`. The SE-16b rule (to be codified) requires explicit statement of: predecessor timestamp, session date, correction rationale. OBS-R25-001 should be addressed in the F-R86 fix-burst as documentation polish to the VP §Trace v1.19 entry.

Cons R25 CLEAN result: counter stays at 0/3 (cons results do not advance the D-047 adversary counter; they gate on their own clean criterion). The cons-CLEAN result confirms no cross-document consistency gaps from the F-R85 serial fix-burst beyond the 3 findings already in this R86 adversary report.

---

## Lens Rotation Result

The following lenses were rotated during R86 review:

1. **In-burst-added citation lens (NEW):** Post-fix-burst, explicitly scan all citations ADDED in the burst for SE-15d reciprocity, separately from the pre-existing-citation scan. This lens produced C-R86-1.

2. **Multi-version stale-citation depth scan:** For each VP file, grep for PRD version strings older than v1.14 (3+ versions stale). This lens produced I-R86-1 at VP-PROTO-002 line 1867 (PRD v1.10, 5 versions stale).

3. **§Trace internal consistency lens (Extension 9 class):** Verify §Trace prose summary counts match §Trace table row counts for all numeric claims. This lens produced I-R86-2 (4 vs 5 row count discrepancy in PRD §Trace v1.15).

4. **Extension 16 scope completeness lens:** Audit whether Extension 16's rules cover all citation categories (pre-existing, in-burst-added, cross-artifact). Produced O-R86-2 (in-burst-added citation scope gap → SE-16a).

5. **Timestamp monotonicity lens:** Check frontmatter timestamp ordering across version sequence. Produced O-R86-1 (v1.18 future-dated correction → SE-16b documentation requirement).

---

## Novelty Assessment

| Finding | Novel axis? | Prior closest finding |
|---------|-------------|----------------------|
| C-R86-1 | YES — in-burst-added citation escaping Extension 16 | O-R86-2 names this as META-3 pattern |
| I-R86-1 | NO — stale PRD citation axis (prior: F-R85-IMP-1, F-R77-4) | New structural-form blind spot |
| I-R86-2 | PARTIALLY NEW — §Trace INTERNAL consistency (prior Extension 9 covers §Trace vs §Coverage Matrix; not §Trace vs §Trace self) | Extension 9 class, new sub-axis |
| O-R86-1 | YES — timestamp monotonicity regression → SE-16b | No prior codification |
| O-R86-2 | YES — Extension 16 in-burst-added scope gap → SE-16a | Structural consequence of C-R86-1 |

**META-pattern META-3 (new):** "Apply audit rule to pre-existing instances, not to in-burst-added instances." This mirrors META-1 (apply rule to trigger site, not pre-existing history — closed by Extension 16) and META-2 (parallel-dispatch coordination gap — closed by Extension 15) but operates at a finer granularity: within a single burst, the audit scope is split between pre-existing and in-burst-added citations, and only pre-existing citations were covered. SE-16a closes META-3.

---

## Routing Recommendations

| Finding | Route | Agent | Target |
|---------|-------|-------|--------|
| C-R86-1 VP-DAEMON-004 §Mech 5 missing reciprocal citation | FIX | formal-verifier | VP v1.20 |
| I-R86-1 VP-PROTO-002 line 1867 stale PRD v1.10 | FIX | formal-verifier | VP v1.20 |
| I-R86-2 PRD §Trace v1.15 count discrepancy (4 vs 5) | FIX | product-owner | PRD v1.16 |
| O-R86-1 VP §Trace v1.19 timestamp monotonicity correction documentation | POLISH | formal-verifier | VP v1.20 |
| O-R86-2 SE-16a codification (Extension 16 in-burst-added scope) | CODIFY | state-manager | lessons.md |

**Serial fix-burst protocol (Extension 15):** PO first (PRD v1.16 closes I-R86-2), then FV (VP v1.20 closes C-R86-1 + I-R86-1 + O-R86-1 with new PRD v1.16 pin).

**SE-16a + SE-16b codification:** State-manager records both sub-extensions in lessons.md before or concurrent with the fix-burst dispatch.
