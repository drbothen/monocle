---
document_type: adversary-pass
producer: adversary
version: "1.0"
timestamp: 2026-05-18T14:45:00Z
phase: phase-1-spec-crystallization
round: R117
verdict: FAIL
findings_count: 4
findings_breakdown: "0 CRIT + 2 HIGH + 1 MED + 1 LOW + 0 process-gap obs"
counter_state: "0/3 → 0/3 HOLDS (FAIL, no advance)"
---

# Adversary Pass R117 — Phase 1 Spec Crystallization

**Verdict: FAIL**
**Findings: 0 CRIT + 2 HIGH + 1 MED + 1 LOW + 0 process-gap obs = 4 items**
**Counter: 0/3 HOLDS (was 0/3 entering this pass; FAIL holds)**

---

## Summary

Fresh-context Phase 1 review found 4 substantive defects, all in the same sibling-propagation class that R116 just closed (F-R116-1, F-R116-2). The R15A/R15B sweep correctly closed VP-INDEX H1↔row drift (R15A) and the brief BC-INDEX pin back-cascade (R15B) but did NOT extend the sibling-propagation sweep to:

1. PRD `traces_to` brief pin (same back-cascade class as R15B — different artifact)
2. ARCH-INDEX ADR Registry row title (same H1↔INDEX drift class as R15A — different index)
3. BC pin-symmetry (extension of F-R110-8 VP discipline to BCs)
4. L2-INDEX internal naming consistency

The R116 closure chain demonstrated the SE-22 sibling-sweep obligation on a single index. R117 surfaces that the obligation extends to ALL sibling indexes (BC-INDEX, ARCH-INDEX, L2-INDEX) and to ALL back-cascade dependents (not just the brief). This is the 2nd explicit SE-22-class occurrence (1st was O-R116-1).

---

## Critical Findings

(none)

---

## Important (HIGH) Findings

### F-R117-1 HIGH — PRD `traces_to` brief pin stale at v1.4.27 (current v1.4.28)

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/prd.md` line 11
**Class:** same-class to F-R116-2 (back-cascade gap on sibling spec bump)
**Confidence:** HIGH
**Routing:** `vsdd-factory:product-owner`

**Evidence:**
- PRD line 11: `traces_to: "product-brief.md v1.4.27; ...; behavioral-contracts/BC-INDEX.md v1.9; ...; domain-spec/L2-INDEX.md v1.0.8"`
- Brief frontmatter `/Users/jmagady/Dev/monocle/.factory/specs/product-brief.md` line 4: `version: "1.4.28"` (bumped in R15B commit 08d1ef4)
- PRD §Trace v1.26.9 (line 624-626) updated L2-INDEX and BC-INDEX pins but did NOT update the brief pin in the same edit window.

**Why FAIL:** F-R116-2 just closed the identical class for the brief→BC-INDEX direction (R15B). R117 fresh-context surfaces that the brief v1.4.27→v1.4.28 bump also requires back-cascade to PRD's `traces_to` field. This is the SAME-CLASS sibling-propagation gap that the R116 SE-22 candidate observation (O-R116-1) is meant to flag.

**Fix:** Update PRD line 11 `traces_to` field: `product-brief.md v1.4.27` → `product-brief.md v1.4.28`. Bump PRD version v1.26.9 → v1.26.10. Add §Trace v1.26.10 documenting F-R117-1 closure. SE-16d monotonicity: new timestamp > 2026-05-18T14:00:00Z.

---

### F-R117-2 HIGH — ARCH-INDEX ADR-0002 row title omits "for Phase 1" present in ADR-0002 H1

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/architecture/ARCH-INDEX.md` line 88
**Class:** same-class to F-R116-1 (H1↔INDEX-row title drift)
**Confidence:** HIGH
**Routing:** `vsdd-factory:architect`

**Evidence:**
- ARCH-INDEX line 88: `| ADR-0002 | Accept nucleo 0.5 Dormancy Risk with Explicit Re-eval Trigger | accepted | adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md |`
- ADR-0002 H1 at `/Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md` line 21: `# ADR-0002: Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`
- INDEX drops " for Phase 1" — policy-relevant qualifier per PG-5 anchor-parenthetical-non-contradiction discipline codified in BC-INDEX §Conventions and SS-conventions-anti-patterns.md.

**Why FAIL:** R15A swept VP-INDEX row title sync per the F-R116-1 sibling-sweep obligation. SE-17e sibling-propagation requires extending the sweep to ALL sibling indexes in the same architectural layer. ARCH-INDEX ADR Registry rows are the same contract-bearing surface class as VP-INDEX rows. The "for Phase 1" qualifier is normatively load-bearing — the ADR's scope is Phase 1-only nucleo acceptance, not all-phases acceptance.

**Fix:** Update ARCH-INDEX line 88 ADR-0002 row title to canonical H1 verbatim. Bump ARCH-INDEX version v1.0.9 → v1.0.10. Add §Trace v1.0.10 with SE-17f BEFORE/AFTER evidence. Sweep ALL other ARCH-INDEX rows (Document Map, ADR Registry, Subsystem Registry) for the same drift class as part of the same dispatch per Production-Grade Rule 4 (consolidated coupled fixes).

---

## Medium Findings

### F-R117-3 MEDIUM — BC-2.01.010 Architecture Source row mixes pinned and unpinned SS references

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md` line 89
**Class:** Cross-SS Architecture-Source Pin Symmetry (F-R110-8 discipline extension to BCs)
**Confidence:** HIGH
**Routing:** `vsdd-factory:product-owner`

**Evidence:**
- BC-2.01.010 line 89: `| Architecture Source | SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging |`
- First reference pins `v1.0.32`; second reference (SS-core-types-and-abi.md) has NO version pin.
- F-R110-8 established the Cross-SS Architecture-Source Pin Symmetry discipline (VP-INDEX §Conventions). All 22 VP files now carry pinned SS references. BC-2.01.010 violates the same symmetry within a single cell.

**Why FAIL:** Production-Grade Default + sibling-propagation: discipline established for VP files should propagate to BC files carrying the same Architecture Source contract surface. Mixed pin/unpin within one cell creates a future staleness audit blind-spot (the unpinned SS-core-types reference cannot be re-validated against a target version).

**Fix:** Update line 89 to pin both SS references: `SS-daemon-lifecycle.md v1.0.32 §...; SS-core-types-and-abi.md v1.2.13 §Phase 1 PRD BC Pre-Staging`. Bump BC-2.01.010 patch version. Add §Trace with SE-17f evidence. Sweep all 22 BC files for the same class — extract pin-symmetry as a BC-INDEX convention codification candidate.

---

## Low Findings

### F-R117-4 LOW — L2-INDEX Document Map row labels diverge from Capabilities Registry canonical capability names

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/domain-spec/L2-INDEX.md` lines 44, 45
**Class:** internal naming inconsistency (single-file)
**Confidence:** HIGH
**Routing:** `vsdd-factory:business-analyst`

**Evidence:**
- L2-INDEX line 44 (Document Map): `| Forward-Compat Wire Formats | CAP-002-forward-compat-wire-formats.md | ...`
- L2-INDEX line 62 (Capabilities Registry): `| CAP-002 | Forward-Compatible Wire Formats | P0 | SS-02 | ...`
- CAP-002 H1: `# CAP-002: Forward-Compatible Wire Formats` (matches Capabilities Registry, diverges from Document Map)
- L2-INDEX line 45 (Document Map): `| Multi-Harness Adapter | CAP-003-multi-harness-adapter.md | ...`
- L2-INDEX line 63 (Capabilities Registry): `| CAP-003 | Multi-Harness Adapter Surface | ...`
- CAP-003 H1: `# CAP-003: Multi-Harness Adapter Surface`

**Why FAIL (LOW):** Document Map is a navigation aid (not contract surface), but internal inconsistency within a single index file is confusing to readers and any downstream tooling that greps for canonical capability names. The pattern is the same H1↔INDEX class as F-R116-1 just closed.

**Fix:** Align L2-INDEX line 44 → `Forward-Compatible Wire Formats`; line 45 → `Multi-Harness Adapter Surface`. Bump L2-INDEX v1.0.8 → v1.0.9 with §Trace v1.0.9.

---

## Observations

(none — no new process-gap candidates this pass; the SE-22 sibling-sweep candidate O-R116-1 already exists and is HELD per D-114; the 4 findings above are evidence that the prior SE-22 instance has not yet been broadly enough applied — content gap, not new process gap. Per D-114 Goodhart's-law deferral, the SE-22 candidate has now had 2 explicit occurrences. 3+ needed for codification.)

---

## Novelty Assessment

**Novelty: HIGH.** F-R117-1 and F-R117-2 are genuinely new instances of the F-R116-1/F-R116-2 sibling-propagation class — distinct artifacts (PRD, ARCH-INDEX) and distinct field surfaces (`traces_to`, ADR Registry). F-R117-3 extends the F-R110-8 pin-symmetry discipline from VPs to BCs (cross-layer propagation). F-R117-4 is a fresh-context discovery of L2-INDEX internal inconsistency that prior passes' focus on VP-INDEX/BC-INDEX/ARCH-INDEX axes left unexamined.

This is exactly the "Fresh-Context Compounding Value" the agent prompt describes — patterns invisible to passes anchored to their own assumptions.

---

## Counter Decision Rationale

Counter is at 0/3 (R116 FAIL just reset). With 4 findings (2 HIGH + 1 MED + 1 LOW), R117 = FAIL. Counter HOLDS at 0/3. Next round (R118) will be the test after these 4 findings + the parallel GAP-R56-002 (L2-INDEX §Trace line 149 brief pin) are closed in an R16 fix-burst chain.

---

## Status

**AWAITING ROUND 16 CLOSURE CHAIN — orchestrator dispatching:**

- R16A (PO): F-R117-1 PRD `traces_to` brief pin v1.4.27 → v1.4.28. Patch bump PRD v1.26.9 → v1.26.10.
- R16B (Architect): F-R117-2 ARCH-INDEX line 88 ADR-0002 row + sweep all ARCH-INDEX rows. Bump ARCH-INDEX v1.0.9 → v1.0.10.
- R16C (PO): F-R117-3 BC-2.01.010 pin-symmetry + sweep all 22 BCs.
- R16D (BA): F-R117-4 + GAP-R56-002 combined L2-INDEX fix. Bump L2-INDEX v1.0.8 → v1.0.9.
- R16E (SM): STATE v5.76 + input-hash + closure commit.
- Then R118 + cons R57 dispatch.
