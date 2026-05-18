---
document_type: consistency-report
level: ops
version: "1.0.0"
producer: vsdd-factory:consistency-validator
timestamp: 2026-05-18T05:00:00Z
traces_to: prd.md
project: monocle
---

# Consistency Pass R61 — Phase 1 Cross-Document Review

**Timestamp:** 2026-05-18T05:00:00Z
**Pass:** R61 (fresh context, zero prior state)
**Artifact baseline:** post-Round-20

---

## Summary

| Dimension | Result | Notes |
|-----------|--------|-------|
| D1 Pins | FAIL | 1 stale pin: PRD traces_to VP-INDEX v1.15 (canonical: v1.16) |
| D2 Index H1 alignment | PASS | All 4 index H1 headings consistent with document_type |
| D3 ID integrity | PASS | 22 BCs, 22 VPs — counts match on disk and in all index tables |
| D4 Anchors | PASS | No broken anchor cross-references in active body sections |
| D5 Counts | PASS | VP-INDEX: SS-01=10, SS-02=8, SS-03=4, Total=22 — arithmetic correct |
| D6 Naming | PASS | Project name consistent (monocle/Monocle); BC/VP IDs canonical throughout |
| D7 Traceability | PASS | All 22 VPs cite PRD v1.26.15 in active §References; VP-INDEX §References cites PRD v1.26.15 |
| D8 SE-16d monotonicity | PASS | Chain: brief 00:30Z < PRD 03:00Z < VP-INDEX 03:30Z — strictly increasing |
| D9 Frontmatter | PASS | All VP versions in declared range v1.0.12–v1.0.16; all 22 VP timestamps 03:30Z |
| D10 Citation integrity | PASS | VP-INDEX §References cites PRD v1.26.15, BC-INDEX v1.11; no stale active citations |

**Overall verdict: FAIL — 1 gap**

---

## Findings

### GAP-R61-001 (HIGH) — PRD traces_to VP-INDEX pin stale: v1.15 → v1.16

**Pre-surfaced by:** SM round-20C (SE-23)

**Location:** `/Users/jmagady/Dev/monocle/.factory/specs/prd.md` frontmatter line 11

**Evidence:**
```
traces_to: "... verification-properties/VP-INDEX.md v1.15"
```

**Canonical state:** VP-INDEX.md frontmatter `version: "1.16"`, timestamp `2026-05-19T03:30:00Z` (R20B).

**Root cause:** Same reverse-cascade gap class as F-R121-1 / GAP-R60-001. R20A (PRD v1.26.15, commit 68863bd) bumped the VP-INDEX pin v1.14 → v1.15. R20B subsequently bumped VP-INDEX v1.15 → v1.16 as the cascade-tail consumer-ledger refresh. PRD forward-pin was not updated in R20B because VP-INDEX is a downstream consumer of the PRD, not an input. However, `traces_to:` on the PRD asserts VP-INDEX version as a normative reference — and the normative reference is stale.

**Affected site:** `prd.md` line 11 (frontmatter `traces_to:` field), sole normative occurrence.

**Not affected:** All 22 VP files `§References PRD:` lines correctly cite v1.26.15. VP-INDEX `§References PRD:` line correctly cites v1.26.15. BC-INDEX unaffected.

**Required fix:** PO dispatch — `prd.md` version bump v1.26.15 → v1.26.16; `traces_to:` VP-INDEX pin v1.15 → v1.16; `timestamp` refresh; `§Trace v1.26.16` added with SE-16d/SE-17a/SE-22 v2 declarations. Routing: `vsdd-factory:product-owner`.

---

## CLEAN surfaces (confirmed zero drift)

| Surface | Verified state |
|---------|---------------|
| VP-INDEX version | v1.16 |
| VP-INDEX frontmatter `traces_to` | `prd.md` (bare — correct, VP-INDEX traces up to PRD) |
| VP-INDEX §References `PRD:` | v1.26.15 — current |
| VP-INDEX §References `BC index:` | v1.11 — current |
| All 22 VP active §References `PRD:` | v1.26.15 — current (22/22) |
| VP file count on disk | 22 (matches index) |
| BC-INDEX active table rows | 22 (matches summary) |
| VP-INDEX active table rows | 22 (matches summary) |
| VP version range | v1.0.12–v1.0.16 (within declared range) |
| All 22 VP timestamps | 2026-05-19T03:30:00Z (co-incident with VP-INDEX — correct for batch dispatch) |
| SE-16d monotonicity (R20 chain) | PASS: STATE 02:50Z → PRD 03:00Z → VP-INDEX 03:30Z → STATE 04:00Z |
| Brief v1.4.30 version | current |
| ARCH-INDEX v1.0.10 version | current |
| BC-INDEX v1.11 version | current |
| L2-INDEX v1.0.11 version | current |

---

## Gate decision

**FAIL** — GAP-R61-001 is a normative pin staleness gap in `prd.md` `traces_to:`. Dispatch to `vsdd-factory:product-owner` for R21 PO burst (PRD v1.26.15 → v1.26.16).
