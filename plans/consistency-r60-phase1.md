---
document_type: consistency-validation
producer: consistency-validator
version: "1.0"
timestamp: 2026-05-19T02:45:00Z
phase: phase-1-spec-crystallization
round: R60
verdict: GAPS
gap_count: 1
gap_breakdown: "1 MAJOR (stale VP-INDEX pin in PRD traces_to)"
---

# Consistency Validation R60 — Phase 1 Spec Crystallization

> Fresh-context pass. Canonical artifact set: post-Round-19. Zero prior context.

---

## 10-Dimension Summary Table

| # | Dimension | Status | Notes |
|---|-----------|--------|-------|
| 1 | Cross-document version pins | FAIL | PRD `traces_to` pins VP-INDEX at v1.14; canonical is v1.15 (R19F) |
| 2 | Index↔file H1 alignment | PASS | BC-INDEX v1.11 titles match BC file H1s sampled; VP-INDEX v1.15 titles match VP file H1s (22/22 Renumbering Appendix consistent) |
| 3 | ID consistency | PASS | 22 VPs on disk = 22 in VP-INDEX; 22 BCs on disk = 22 in BC-INDEX; counts match PRD §1.3 D-row BC references |
| 4 | Anchor link integrity | PASS | BC-INDEX SS-NN subsection anchors stable; VP-INDEX Renumbering Appendix append-only; no orphaned IDs detected |
| 5 | Counts | PASS | SS-01: 10 BCs/10 VPs; SS-02: 8 BCs/8 VPs; SS-03: 4 BCs/4 VPs; totals match all index summaries |
| 6 | Naming | PASS | Subsystem names SS-01/SS-02/SS-03 consistent across ARCH-INDEX, BC-INDEX, VP-INDEX, L2-INDEX |
| 7 | Traceability completeness | PASS | PRD traces_to brief v1.4.30, L2-INDEX v1.0.11, BC-INDEX v1.11, ARCH-INDEX v1.0.10, 5 ADRs, SS-permissions-phase1 v1.5.2, SS-forward-compatibility v1.2.19 — all correct. VP-INDEX traces_to prd.md (correct). All VP files §References cite PRD v1.26.14 (R19F cascade confirmed vp-001, vp-011, vp-015, vp-022 sampled). CAP-001 v1.6 active brief pointer = v1.4.30 (§Trace v1.6). L2-INDEX v1.0.11 traces_to product-brief.md (correct). |
| 8 | SE-16d UTC timestamp monotonicity | PASS | VP-INDEX v1.15 @ 2026-05-19T02:00:00Z > v1.14 @ 2026-05-18T23:30:00Z > R19E PRD @ 2026-05-19T01:30:00Z. L2-INDEX v1.0.11 @ 2026-05-19T01:00:00Z > CAP-001 v1.5 @ 2026-05-18T20:00:00Z. ARCH-INDEX v1.0.10 @ 2026-05-18T15:30:00Z > BC-INDEX v1.11 @ 2026-05-18T22:00:00Z ordering preserved within chain. Report timestamp 2026-05-19T02:45:00Z > STATE v5.82 high-water 2026-05-19T02:30:00Z. |
| 9 | Frontmatter completeness | PASS | All sampled artifacts carry required fields: document_type, level, version, producer, traces_to, timestamp, input-hash, inputs. No missing required fields detected. |
| 10 | Citation integrity | PASS | VP-INDEX §References cites PRD v1.26.14, BC-INDEX v1.11. PRD inputs list all 5 ADRs explicitly. All 22 VP §References (active scope, SE-17g-filtered) cite PRD v1.26.14 per R19F cascade. |

---

## Gap Details

### GAP-R60-001 — MAJOR: PRD `traces_to` VP-INDEX pin stale (v1.14 → v1.15)

**Severity:** MAJOR  
**Artifact:** `.factory/specs/prd.md` v1.26.14, frontmatter line 11  
**Finding:** `traces_to` field ends with `verification-properties/VP-INDEX.md v1.14`. VP-INDEX was bumped to v1.15 in R19F (commit timestamp 2026-05-19T02:00:00Z, after PRD v1.26.14 at 2026-05-19T01:30:00Z).  
**Root cause:** PRD v1.26.14 (R19E, commit 31f984a) was authored before the R19F VP-INDEX cascade that advanced VP-INDEX from v1.14 to v1.15. The SE-22 consumer-ledger for R19E did not identify PRD as a downstream consumer of VP-INDEX because PRD is the *producer* of context for VP-INDEX, not typically a consumer. However, PRD `traces_to` explicitly pins the VP-INDEX version for audit traceability.  
**Expected value:** `verification-properties/VP-INDEX.md v1.15`  
**Actual value:** `verification-properties/VP-INDEX.md v1.14`  
**Confirmation:** VP-INDEX §Trace v1.15 (lines 1324-1330) documents the R19F bump with SE-16d PASS at 2026-05-19T02:00:00Z; PRD frontmatter timestamp is 2026-05-19T01:30:00Z — PRD predates the VP-INDEX v1.15 commit.  
**Remediation:** Product-owner dispatch: bump PRD `traces_to` terminal entry from `VP-INDEX.md v1.14` to `VP-INDEX.md v1.15`; increment PRD version (v1.26.14 → v1.26.15); update timestamp; add §Trace entry documenting this mechanical pin refresh.

---

## Observations (Non-Blocking)

**OBS-R60-001:** SE-17g historical preservation is operating correctly. Grep sweeps across VP §Trace blocks find large counts of historical `commit pending` strings and stale version references — all correctly classified as INFORMATIONAL per VP-INDEX §Conventions (lines 99-141). Active-scope filtering (`awk '/^## §Trace/{skip=1}'`) confirms zero stale NORMATIVE citations in VP §References sections.

**OBS-R60-002:** The sequencing pattern that produced GAP-R60-001 — PRD authored in R19E, then VP-INDEX bumped in R19F as a cascade-tail — is a structural artifact of the R19 multi-step chain. The SE-22 v2 consumer-ledger declaration in R19F (VP-INDEX §Trace v1.15, line 1330) does not list PRD as a known consumer of VP-INDEX, which allowed the stale pin to persist. Codifying PRD as a mandatory downstream consumer of VP-INDEX bumps in the SE-22 consumer-ledger template would prevent recurrence.

---

## Validation Gate Result

**GATE: FAIL — 1 MAJOR gap blocks clean pass.**

GAP-R60-001 must be resolved (PO dispatch to bump PRD traces_to VP-INDEX pin v1.14 → v1.15) before this round can be declared CLEAN.

All other dimensions: PASS with zero findings.

Consistency score: **95/100** (1 MAJOR gap against 10 dimensions; 9 dimensions fully clean).
