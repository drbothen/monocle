---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.14 4997354 + VP v1.18 6915b5d + arch v1.0.17 a798d51 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 18 (R85); post-F-R84 serial fix-burst snapshot"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T16:01:00Z
pass_number: 1
attempt: 18
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 CRITICAL + 3 HIGH + 2 LOW observations
---

# Adversarial Review R85 — Phase 1 (D-047 Strict, Pass 1 Attempt 18 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

- 1 CRITICAL: F-R85-CRIT-1 — VP-DAEMON-004 §Cross-Property Citations: line 441 cites VP-DAEMON-002 "Post-condition 6" — but VP-DAEMON-002 has only 5 Post-conditions (Post-conditions 1–5). Post-condition 6 does not exist in VP-DAEMON-002. This is an anchor mis-cite: the citation resolves to a non-existent postcondition anchor, creating a traceability gap that a fresh-context implementer following the VP dependency chain cannot resolve.

- 1 HIGH: F-R85-IMP-1 — 5 line-wrapped `PRD v1.12` test-name citation sites escaped the F-R84 sweep. VP v1.18 still cites `PRD v1.12` (db7f50e) in 5 locations where wrap-continuation lines were not caught by the sweep. The consistency auditor (cons R24, GAP-R24-001) independently caught 1 of these 5 sites (line 251). The remaining 4 sites also escaped. Root cause: the F-R84 sweep used a grep pattern that matched only single-line `PRD v1.12` citations — wrap-continuation lines where `v1.12` begins a new line after a hyphen or open-paren were not covered.

- 1 HIGH: F-R85-IMP-2 — SE-15c recursive failure. SE-15c (convention back-propagation) was codified in the F-R84 burst: when a new citation convention is established on one row of a homogeneous table, sweep ALL sibling rows. However, SE-15c was applied to exactly one trigger site (NFR-012 VP probe citation form) and NOT recursively swept to all sibling rows. Specifically: NFR-004, NFR-005, and NFR-010 in the PRD §4 NFR table each lack VP probe citations in their Validation Method cells — the same convention deficit that SE-15c's application to NFR-012 should have back-propagated to all siblings. This is a codification-without-backfill regression: the discipline was codified but the codification burst did not perform the mandatory backward sweep of pre-existing sites.

- 1 HIGH: F-R85-IMP-3 — SE-15d recursive failure. SE-15d (cross-property VP reciprocity) was codified in the F-R84 burst: when VP-A cites VP-B in §Mechanism cross-property, VP-B MUST cite VP-A back. SE-15d was applied to exactly one trigger pair (VP-AUTH-001 ↔ VP-DAEMON-005) and NOT recursively swept to all cross-property citation pairs in VP v1.18. Review of VP v1.18 §Mechanism blocks reveals at least 4 additional unidirectional cross-property citations lacking the required reciprocation. These are unresolved traceability gaps that SE-15d, by its own terms, required to be swept at codification time.

**Observations (process-relevant):**

- Obs-R85-1 — META pattern: codification-without-backfill. SE-15c and SE-15d were each applied to the trigger finding's site but the codification protocol included no mandatory step requiring a recursive sweep of ALL pre-existing sites in the same artifact set where the new rule applies. This is a systematic protocol gap: every new Sub-Extension (SE-Nx) codified in a fix-burst must trigger a mandatory backfill sweep of all pre-existing sites in the same artifact class. The absence of this step in the F-R84 codification protocol produced two immediate regressions (F-R85-IMP-2, F-R85-IMP-3). Codification without backfill creates a new finding class: "just-codified rule, not-yet-applied to own history."

- Obs-R85-2 — SE-15d citation-form ambiguity. VP-DAEMON-002 §Post-condition 5 uses `cross-check VP-DAEMON-003` (rather than `cross-property with VP-DAEMON-003`). The form `cross-check` is a citation that creates an implicit dependency. If SE-15d applies only to citations using the canonical `cross-property` form, then `cross-check` citations escape the reciprocity requirement — creating an ambiguity that adversarial passes can exploit. SE-15d should be explicitly extended to cover BOTH `cross-property` AND `cross-check` citation forms.

---

## F-R85-CRIT-1 — CRITICAL: VP-DAEMON-004 cites non-existent VP-DAEMON-002 Post-condition 6

### Description

VP-DAEMON-004 §Cross-Property Citations (line 441) contains the citation:

```
cross-property with VP-DAEMON-002 Post-condition 6
```

VP-DAEMON-002 defines exactly 5 Post-conditions: Post-conditions 1 through 5. Post-condition 6 does not exist. This is an anchor mis-cite that cannot be resolved by a fresh-context agent following the traceability chain. Any implementer attempting to verify the VP-DAEMON-004 ↔ VP-DAEMON-002 dependency will encounter a dead reference.

### Impact

CRITICAL severity because: (1) this is a normative traceability anchor — not prose ambiguity; (2) it silently breaks the VP-DAEMON-004 → VP-DAEMON-002 dependency chain; (3) a fresh-context verifier cannot distinguish "Post-condition 6 was removed and the citation was not updated" from "Post-condition 6 was intended but was never written."

### Required fix

VP-DAEMON-004 §Cross-Property Citations line 441: change `Post-condition 6` to the correct Post-condition number (either Post-condition 5 if that is the intended referent, or document which postcondition governs the dependency being cited). Formal-verifier must identify which VP-DAEMON-002 Post-condition the dependency was intended to reference and update accordingly.

---

## F-R85-IMP-1 — HIGH: 5 line-wrapped `PRD v1.12` citations in VP v1.18 escaped F-R84 sweep

### Description

Consistency auditor cons R24 (GAP-R24-001) independently caught one site: VP v1.18 line 251 contains a line-wrapped citation where `PRD v1.12` appears as a wrap-continuation. The F-R84 sweep used a single-line grep pattern that did not catch wrap-continuation forms. Four additional sites with the same wrap-continuation pattern remain in VP v1.18.

### Root cause

The grep pattern used in the F-R84 sweep was:

```
grep -nE "PRD v1\.12" verification-properties.md
```

This pattern does not match cases where the version string is split across a line boundary by markdown wrap. The correct pattern must also cover wrap-continuation forms such as:

```
(PRD
  v1.12
```

or

```
— PRD
v1.12
```

### Impact

HIGH severity: VP v1.18 is the canonical Phase 1 VP document. Five citation sites still reference PRD v1.12 (db7f50e) — the pre-F-R79 version — rather than PRD v1.14 (4997354). A fresh-context verifier following these citations will load the wrong PRD version and potentially conclude different acceptance criteria apply.

### Required fix

Formal-verifier must run a multi-line-aware grep (or manual review of the 5 known sites + a fresh grep confirming 0 remaining) to close all `PRD v1.12` citations to `PRD v1.14`. SE-15b evidence transcripts required.

---

## F-R85-IMP-2 — HIGH: SE-15c recursive failure — NFR-004/005/010 VP probe citations missing

### Description

SE-15c (L-F-R63 Extension 15 Sub-Extension c, codified in F-R84 burst) requires: when a new citation convention is established on one row of a homogeneous table, sweep ALL sibling rows for the same convention. The F-R84 PO burst applied SE-15c to NFR-012 (added VP probe 5.e citation to its Validation Method cell) and swept NFR-009 as the single "sibling" check (per Obs-R84-1 in the F-R84 report). However, NFR-004, NFR-005, and NFR-010 were not swept. All three have existing VP coverage (probes exist in the VP catalog) but their Validation Method cells in PRD §4 NFR table lack the VP probe citation form that NFR-012 established.

### Impact

HIGH severity: SE-15c was codified in the same fix-burst that produced this gap. The just-codified discipline was applied to exactly 2 rows (NFR-012 + NFR-009) but not to the remaining 10 rows of the PRD §4 NFR table. This is the canonical "codification-without-backfill" anti-pattern that Obs-R85-1 names.

### Required fix

Product-owner must sweep all 12 rows of PRD §4 NFR table for VP probe citation gaps (applying SE-15c to all sibling rows). NFR-004, NFR-005, and NFR-010 have confirmed gaps. The sweep must emit SE-15b grep transcripts.

---

## F-R85-IMP-3 — HIGH: SE-15d recursive failure — 4+ unidirectional cross-property citations

### Description

SE-15d (L-F-R63 Extension 15 Sub-Extension d, codified in F-R84 burst) requires: when VP-A cites VP-B in §Mechanism cross-property, VP-B MUST cite VP-A back. The F-R84 FV burst established VP-AUTH-001 ↔ VP-DAEMON-005 reciprocity and swept that pair. However, the SE-15d codification prompt did not require a sweep of ALL cross-property citation pairs in VP v1.18. Review identifies at least 4 additional unidirectional citations:

1. VP-DAEMON-002 §Mechanism cites VP-DAEMON-006 — VP-DAEMON-006 does not cite VP-DAEMON-002 back.
2. VP-DAEMON-005 §Mechanism cites VP-LOCK-001 (the F-R84 pair, but only VP-DAEMON-005→VP-LOCK-001 was added; VP-LOCK-001→VP-DAEMON-005 was not confirmed as fully reciprocal in VP v1.18).
3. VP-ENGINE-001 §Mechanism cites VP-DAEMON-001 — VP-DAEMON-001 does not cite VP-ENGINE-001 back.
4. VP-PROTO-001b §Mechanism cites VP-AUTH-001 — VP-AUTH-001 does not cite VP-PROTO-001b back.

### Impact

HIGH severity: SE-15d by its own terms required a sweep "at codification time." The codification did not include this sweep. Four or more cross-property citation pairs remain asymmetric, each creating a traceability gap identical to the VP-AUTH-001 ↔ VP-DAEMON-005 gap that SE-15d was codified to close.

### Required fix

Formal-verifier must enumerate all `cross-property` and `cross-check` citation forms in VP v1.18 §Mechanism blocks, verify bidirectional reciprocity for each pair, and add reciprocal citations where missing. SE-15b transcripts required. Obs-R85-2 adjudication must be applied: SE-15d covers BOTH `cross-property` AND `cross-check` forms.

---

## Observations

### Obs-R85-1 — META pattern: codification-without-backfill

**Pattern:** When a new discipline (Extension or Sub-Extension) is codified in a fix-burst, the codification is applied to the trigger site (the specific finding that motivated the codification). No step in the current codification protocol requires a backward sweep of ALL pre-existing sites in the same artifact class where the new rule would apply.

**Evidence:** SE-15c was codified using NFR-012 as the trigger site. SE-15c requires sibling-row back-propagation. The codification burst swept NFR-009 (explicitly named in Obs-R84-1) and no other rows. NFR-004, NFR-005, NFR-010 remained unchecked. SE-15d was codified using VP-AUTH-001 ↔ VP-DAEMON-005 as the trigger pair. SE-15d requires all cross-property pairs. The codification burst swept that one pair and no others. At minimum 4 additional pairs remained unchecked.

**Rule implication:** Every new SE-Nx codification must include a mandatory backfill sweep step — see L-F-R63 Extension 16 (to be codified by state-manager in cycle lessons after this R85 report).

### Obs-R85-2 — SE-15d citation-form ambiguity (`cross-check` vs `cross-property`)

**Pattern:** VP-DAEMON-002 §Post-condition 5 uses the citation form `cross-check VP-DAEMON-003`. SE-15d as currently codified references only the `cross-property` citation form. If `cross-check` is considered a distinct form not covered by SE-15d, then VP-DAEMON-002 §Post-condition 5 escapes the reciprocity requirement.

**Adjudication required:** State-manager must adjudicate whether SE-15d applies uniformly to both `cross-property` AND `cross-check` forms. Production-grade default per CLAUDE.md §CANONICAL PRINCIPLE: answer in scope. Recommended adjudication: SE-15d applies to BOTH forms. Both `cross-property` and `cross-check` citations create bidirectional dependencies that must be reciprocated. The distinction in phrasing is a stylistic variation, not a semantic distinction that should affect the reciprocity requirement.

---

## F-R84 Closure Verification — All 10 Items HOLDING

The following table records the independent verification of F-R84 closure items in PRD v1.14 + VP v1.18 + arch v1.0.17 + manifest v1.1.12.

| F-R84 Item | Severity | Closure Site | Status in R85 |
|-----------|----------|-------------|---------------|
| F-R84-1: ~93 stale arch-pin sites (parallel-dispatch) | CRITICAL | PRD v1.14 + VP v1.18 | HOLDING — arch v1.0.17 a798d51 cited correctly in frontmatter and §Trace of both documents |
| F-R84-2: RTM BC-ID column NFR-012 schema violation | HIGH | PRD v1.14 | HOLDING — column header now reads "Requirement ID"; NFR-012 row properly placed |
| F-R84-3: §Purpose stale SHA (4th recurrence) | HIGH | VP v1.18 | HOLDING — §Purpose cites current arch v1.0.17 + PRD v1.14 pins |
| F-R84-4: VP §Trace version citations stale | HIGH | VP v1.18 | HOLDING — §Trace entries cite arch v1.0.17 throughout |
| F-R84-5: per-VP §Mechanism block version citations stale | MEDIUM | VP v1.18 | HOLDING — per-VP §Mechanism blocks updated to arch v1.0.17 |
| F-R84-6: PRD frontmatter traces_to stale | MEDIUM | PRD v1.14 | HOLDING — frontmatter traces_to cites arch v1.0.17 a798d51 |
| F-R84-7: Extension 14 codification lacks Extension 13 grep evidence | LOW | VP v1.18 | HOLDING — SE-15b evidence transcripts present in §Trace |
| Obs-R84-1: NFR-009 sibling-row convention (SE-15c trigger) | OBS | PRD v1.14 | PARTIALLY HOLDING — NFR-012 convention applied; NFR-009 swept; REGRESSION at NFR-004/005/010 (F-R85-IMP-2) |
| Obs-R84-2: VP-AUTH-001 ↔ VP-DAEMON-005 reciprocity (SE-15d trigger) | OBS | VP v1.18 | PARTIALLY HOLDING — trigger pair reciprocated; REGRESSION at 4+ additional pairs (F-R85-IMP-3) |
| Obs-R84-3: Serial dispatch protocol recommendation | OBS | Extension 15 codification | HOLDING — Extension 15 serial protocol codified and operationalized in F-R84 burst |

**Summary:** 8 of 10 F-R84 items fully holding. 2 items (Obs-R84-1/2) partially holding — the trigger-site fixes hold but codification-without-backfill created new regressions (F-R85-IMP-2/3). The 2 partial holds are the direct source of F-R85-IMP-2 and F-R85-IMP-3.

---

## F-R81 Closure + GAP-R20 Stability Re-Verification

All 5 F-R81/GAP-R20 closures independently verified holding in VP v1.18:

| Item | Status |
|------|--------|
| F-R81-1: Extension 11 canonical codification body | HOLDING — BC-id prefix grep pattern present in canonical body |
| F-R81-2/GAP-R20-001: §Purpose stale SHA recurrence guard | HOLDING — §Purpose on propagation target list; VP v1.18 §Purpose uses current pins |
| F-R81-3: §Trace v1.15 line-number refs → section-heading anchors | HOLDING — §Trace uses section headings, not line numbers |
| GAP-R20-002: §G-6 residual BC-HOOK-022 normative framing | HOLDING — BC-HOOK-022 not present in §G-6; NFR-006 is sole Phase 1 BC |
| GAP-R20-003: Extension 13 evidence form `grep -nE` | HOLDING — §Trace uses `grep -nE` form throughout |

---

## Lens Rotation Result

**Lens rotated to: codification-protocol axis (NEW AXIS)**

Prior lenses applied through R84: spec-content (R62–R80), orchestration-protocol (R84). R85 rotates to a meta-meta axis: the protocol for codifying disciplines. The finding class is "codification-without-backfill" — where the act of codifying a new rule produces an immediate regression by failing to apply the rule to its own pre-existing context.

This is a third orchestration-protocol-axis discipline (after Extension 15 cross-layer parallel-dispatch). The discipline class is:
- spec-content rules (Extensions 1–13)
- spec-structural rules (Extension 14)
- orchestration-protocol rules (Extensions 15+)

Extension 16 (codification-protocol mandatory backfill sweep) is required to close this new axis structurally.

**F-R80 META closure (fabrication-pattern at Extension 3 axis) continues to hold across R85.** No new instances of fabricated grep evidence detected. Extension 13 machine-greppable evidence requirement is operationally confirmed through VP v1.18.
