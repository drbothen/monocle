---
document_type: adversary-pass
pass_id: R106
attempt: 2
policy: D-047-strict
counter_before: "0/3"
counter_after: "0/3"
verdict: FAIL
timestamp: 2026-05-17T22:00:00Z
producer: vsdd-factory:adversary
traces_to:
  - .factory/specs/prd.md v1.26.3 (b2b378b) → v1.26.4 (df5605a)
  - .factory/specs/behavioral-contracts/BC-INDEX.md v1.3 → v1.4 (bb088a2)
  - .factory/specs/verification-properties/VP-INDEX.md v1.2 → v1.3 (7b8d6e8)
  - .factory/specs/architecture/ARCH-INDEX.md v1.0.4 → v1.0.5 (03a4c57)
  - .factory/specs/domain-spec/L2-INDEX.md v1.0.6 (unchanged)
  - .factory/specs/product-brief.md v1.4.24 → v1.4.25 (56c11fe)
artifact_pins_before:
  - artifact: prd.md
    version: "1.26.3"
  - artifact: BC-INDEX.md
    version: "1.3"
  - artifact: VP-INDEX.md
    version: "1.2"
  - artifact: ARCH-INDEX.md
    version: "1.0.4"
  - artifact: L2-INDEX.md
    version: "1.0.6"
  - artifact: SS-deps-pin-manifest.md
    version: "1.1.17"
  - artifact: product-brief.md
    version: "1.4.24"
artifact_pins_after:
  - artifact: prd.md
    version: "1.26.4"
  - artifact: BC-INDEX.md
    version: "1.4"
  - artifact: VP-INDEX.md
    version: "1.3"
  - artifact: ARCH-INDEX.md
    version: "1.0.5"
  - artifact: SS-daemon-lifecycle.md
    version: "1.0.30"
  - artifact: SS-engine-module.md
    version: "1.1.18"
  - artifact: product-brief.md
    version: "1.4.25"
disciplines_in_force: 33
findings_count:
  critical: 3
  high: 7
  medium: 6
  low: 4
observations_count: 2
closure_chain: "Round 5 — 6 commits (bb088a2, df5605a, 56c11fe+fd790b8, 7b8d6e8, 03a4c57, SM-5F)"
---

# Adversary Pass R106 — Phase 1 Spec Crystallization

**Policy:** D-047 strict (pass 1 attempt 2 against restructured artifacts)
**Verdict:** FAIL — 20 findings (3 CRIT + 7 HIGH + 6 MED + 4 LOW + 2 process-gap observations)
**Counter:** 0/3 (HOLDS; counter held per D-047 strict)
**Consistency companion:** R45 returned GAPS (5 findings: 1 HIGH + 2 MED + 2 LOW + 4 OBS)
**Closure status:** ALL 25 FINDINGS CLOSED (R106 × 20 + R45 × 5) in Round 5 (6 commits)

## Summary

D-047 strict pass 1 attempt 2 against the restructured artifact set (post D-128 Option A full
closure, T-127' re-audit cycle). The R105 closure chain (D-128) fixed 14+5 findings but introduced
a new failure class: ADR-0005 dual-accept cascade was incompletely propagated — 5 artifacts adopted
the canonical-priority postcondition (BC-2.01.009) without rippling the same constraint to
BC-2.01.008, BC-2.01.005, BC-2.01.002, BC-2.01.003, BC-2.01.007, and the prd-supplements.

**CRITICAL dominant finding (F-R106-1 + F-R106-3):** ADR-0005 cascade incomplete. BC-2.01.009
adopted dual-accept with canonical priority (X-Monocle-Authorization wins when both present) but
sister BCs (BC-2.01.008, BC-2.01.005, BC-2.01.002, BC-2.01.003) did not receive matching
postcondition updates. The prd-supplements (interface-definitions.md v1.2) still defined the old
single-accept semantics. VP-009 had no coverage for the dual-accept path. Together these 5 cascade
gaps blocked Phase 3 TDD.

**Fabrication finding (F-R106-7 HIGH):** SS-daemon-lifecycle.md v1.0.29 §Trace referenced
F-FC-I005 from the template-compliance audit chain as if it were a live finding in v1.0.29; the
F-FC-I005 finding was CLOSED in the R2 residual fix chain and should not appear as normative
context in the new §Trace entry. The fabrication was minor (stale cross-reference) but class-MED
by the 33-discipline framework.

**Orphan Promise finding (F-R106-8 HIGH):** product-brief.md v1.4.24 still cited
`BC-DTU-001` and two other retired monolithic BC IDs as normative requirements. The D-122
restructure renumbered all BCs to BC-2.SS.NNN format; the brief had not been updated to track
the new IDs. This is an orphan-promise class finding: the brief promises behavior anchored to IDs
that no longer exist in the specification index.

**META-class analysis:** 11 of 20 findings are META-class:
- Fabrication: 1 (F-R106-7)
- Orphan Promise: 1 (F-R106-8)
- Cross-BC Contradiction: 1 (F-R106-2)
- Cascade-Incomplete (ADR-0005): 5 (F-R106-1, F-R106-3, F-R106-5, F-R106-15, F-R106-16)
- Stale Citation Pattern: 3 (F-R106-4, F-R106-9, F-R106-10)

---

## Findings

| ID | Severity | Class | Routing | Task | Status | Closure Commit | Description |
|----|----------|-------|---------|------|--------|---------------|-------------|
| F-R106-1 | CRITICAL | Cascade-Incomplete (ADR-0005) | formal-verifier | T-128r | CLOSED | 7b8d6e8 | VP-009 missing dual-accept probes — no coverage for ADR-0005 alternate-header accept path |
| F-R106-2 | CRITICAL | Cross-BC Contradiction | product-owner | T-128s | CLOSED | bb088a2 | BC-2.01.009 PC-4 canonical-priority postcondition conflicted with BC-2.01.008 PC-3 which lacked matching priority rule — cross-BC contradiction when both headers present |
| F-R106-3 | CRITICAL | Cascade-Incomplete (ADR-0005) | product-owner | T-128t | CLOSED | df5605a | interface-definitions.md v1.2 still specified single-accept semantics for hook authorization; dual-accept from ADR-0005 not reflected |
| F-R106-4 | HIGH | Stale Citation Pattern | product-owner | T-128t | CLOSED | df5605a | nfr-catalog.md v1.1 cited SS-engine-module.md at v1.1.15 pin; canonical post-Round-4 is v1.1.18 |
| F-R106-5 | HIGH | Cascade-Incomplete (ADR-0005) | product-owner | T-128t | CLOSED | df5605a | test-vectors supplement lacked test vectors for dual-accept path (alias header correct-secret → 200 + WARN; alias header wrong-secret → 401) |
| F-R106-6 | HIGH | Stale Citation Pattern | product-owner | T-128t | CLOSED | df5605a | error-taxonomy.md missing E-AUTH-003 for alias-header-valid-but-canonical-absent (edge case introduced by ADR-0005 dual-accept) |
| F-R106-7 | HIGH | Fabrication | architect + product-owner | T-128u | CLOSED | 03a4c57 (arch) + bb088a2 (PO) | SS-daemon-lifecycle.md v1.0.29 §Trace cited F-FC-I005 as live-finding context; F-FC-I005 was CLOSED in R2 residual chain; stale normative cross-reference (fabrication class) |
| F-R106-8 | HIGH | Orphan Promise | product-owner | T-128v | CLOSED | 56c11fe | product-brief.md v1.4.24 cited BC-DTU-001 and two retired monolithic BC IDs as normative requirements; post D-122 all BCs are BC-2.SS.NNN; orphan promise (IDs no longer exist in index) |
| F-R106-9 | HIGH | Stale Citation Pattern | formal-verifier | T-128r | CLOSED | 7b8d6e8 | 10 VP files cited SS-daemon-lifecycle.md at stale v1.0.28 or v1.0.29 pre-R5 pin; canonical post-5D is v1.0.30 |
| F-R106-10 | HIGH | Stale Citation Pattern | formal-verifier | T-128r | CLOSED | 7b8d6e8 | VP-INDEX.md v1.2 cited stale SS-daemon-lifecycle pin in 3 summary rows; VP-INDEX v1.3 required |
| F-R106-11 | MED | Cascade-Incomplete (ADR-0005) partial | product-owner | T-128s | CLOSED | bb088a2 | BC-2.01.005 missing dual-accept postcondition matching BC-2.01.009 PC-1/PC-2 |
| F-R106-12 | MED | Cascade-Incomplete (ADR-0005) partial | product-owner | T-128s | CLOSED | bb088a2 | BC-2.01.002 missing alias-path postcondition; no BC text for X-Claude-Code-Ide-Authorization forwarding behavior |
| F-R106-13 | MED | Cascade-Incomplete (ADR-0005) partial | product-owner | T-128s | CLOSED | bb088a2 | BC-2.01.003 canonical-priority rule absent; BC bodies diverge on priority semantics |
| F-R106-14 | MED | Fabrication (stale path) | architect | T-128u | CLOSED | 03a4c57 | ADR-0005 v1.0.1 §inputs listed wrong path for SS-daemon-lifecycle.md — path used pre-restructure filename; post D-122 path is canonical shard path |
| F-R106-15 | MED | Cascade-Incomplete (ADR-0005) | product-owner | T-128t | CLOSED | df5605a | PRD v1.26.3 lacked mass pin refresh for supplements post-Round-4 changes; 17+ stale supplement version cites in prd.md body |
| F-R106-16 | MED | Cascade-Incomplete (ADR-0005) | product-owner | T-128t | CLOSED | df5605a | PRD v1.26.3 body did not cite ADR-0005 in §Decisions traceability column; normative decision without PRD trace |
| F-R106-17 | LOW | Stale Citation Pattern | product-owner | T-128s | CLOSED | bb088a2 | BC-2.01.007 §Trace listed ascending order incorrectly for §Trace version history; reorder to chronological ascending per template |
| F-R106-18 | LOW | Stale Citation Pattern | formal-verifier | T-128r | CLOSED | 7b8d6e8 | VP-INDEX.md v1.2 had 2 VP rows with stale §Source-Contract column values; VP-INDEX v1.3 corrected |
| F-R106-19 | LOW | Orphan Promise (minor) | product-owner | T-128v | CLOSED | 56c11fe | product-brief.md revision history section referenced "v1.4.23 → v1.4.24" in non-chronological order (split revision history entry) |
| F-R106-20 | LOW | Orphan Promise (minor) | product-owner | T-128v | CLOSED | 56c11fe | product-brief.md §SS-engine-module pin cited v1.1.15 (stub-completion); canonical is v1.1.18 post-Round-4 |

---

## Observations (Process-Gap)

### O-R106-1 — ADR-cascade-completion checklist absent (process-gap)

**Class:** Process-gap observation
**Codification candidate:** SE-18-alt or new ADR-cascade gate
**Status:** NOT CODIFIED per D-114 Goodhart's law — first occurrence; 3+ recurrences required

When a new ADR is ratified that modifies a behavioral contract (here ADR-0005 modifying BC-2.01.009),
the existing discipline set (SE-15e cross-layer serial propagation, Extension 14 sibling-site
propagation) does NOT automatically enumerate ALL BCs that share the same behavioral domain. The
ADR-0005 cascade touched BC-2.01.009 explicitly but the 5 sister BCs (BC-2.01.008, BC-2.01.005,
BC-2.01.002, BC-2.01.003, BC-2.01.007) were not enumerated in the original Round 4 closure scope.

**Codification deferred:** This is the first occurrence. Goodhart's law per D-114 requires 3+
empirical recurrences before codification. Recorded as observation for future codification review.

### O-R106-2 — Pin-refresh sweeps scoped too narrowly (process-gap)

**Class:** Process-gap observation
**Codification candidate:** SE-17e extension or new pin-scope gate
**Status:** NOT CODIFIED per D-114 Goodhart's law — first occurrence; 3+ recurrences required

The Round 4 (T-128 chain) pin-refresh sweeps were scoped to architecture documents and primary
spec files. The prd-supplements (interface-definitions, nfr-catalog, test-vectors, error-taxonomy)
were not included in the pin-sweep scope. This allowed stale version cites to persist in supplements
even after the primary artifact set was refreshed.

**Codification deferred:** First occurrence. Per D-114, not codified. Recorded as observation.

---

## Counter Decision

**FAIL — counter held 0/3.**

R106 produced 20 findings (3 CRIT + 7 HIGH + 6 MED + 4 LOW). Per D-047 strict policy, any
finding of any severity resets/holds the counter. Counter remains at 0/3.

Consistency R45 companion: GAPS (5 findings: 1 HIGH + 2 MED + 2 LOW + 4 OBS). Per D-047 strict,
a GAPS verdict from the consistency-validator also holds the counter.

Combined: counter held 0/3. Advance requires CLEAN R107 + CLEAN R46.

---

## Restructure Validity Verdict

**Structure VALID.** The D-122 template-compliance restructure was structurally sound:
- PRD shard structure correct (282-line index + supplements)
- 22 BC files correctly sharded and indexed
- 22 VP files correctly sharded and indexed
- L2 domain spec with 3 CAP shards correct

The R106 failures were exclusively propagation gaps (ADR-0005 cascade incomplete, supplement
pin staleness, brief orphan-promise IDs) — NOT structural non-compliance. The validate-template-
compliance gate (D-123 prerequisite) correctly passed before this adversary round, confirming
structural validity.

---

## META-class Finding Count

| META Class | Finding IDs | Count |
|-----------|-------------|-------|
| Fabrication | F-R106-7 (arch §Trace stale F-FC-I005 cite), F-R106-14 (ADR-0005 stale path) | 2 |
| Orphan Promise | F-R106-8 (brief BC-DTU-001 + 2 retired IDs), F-R106-19, F-R106-20 | 3 |
| Cross-BC Contradiction | F-R106-2 (BC-2.01.009 PC-4 vs BC-2.01.008 PC-3 mismatch) | 1 |
| Cascade-Incomplete (ADR-0005) | F-R106-1, F-R106-3, F-R106-5, F-R106-15, F-R106-16 | 5 |
| Stale Citation Pattern | F-R106-4, F-R106-6, F-R106-9, F-R106-10 | 4 |
| **META subtotal** | | **15** |
| Non-META (partial cascade BC bodies) | F-R106-11, F-R106-12, F-R106-13, F-R106-17, F-R106-18 | 5 |
| **Grand total** | | **20** |

Note: 15 of 20 findings (75%) are META-class (structural pattern propagation gaps). This is
consistent with the Phase 1 Phase-1d convergence history where substantive content has been
CLEAN since R88 in monolithic form and the restructured form preserved that fidelity. The R106
failure is propagation-gap not content-gap.

---

## Cross-Artifact Integrity Verdict

**FAIL — as of R106 pre-Round-5.** Status: NOW CLOSED post-Round-5.

The 3-way cross-artifact integrity failures were:
1. ADR-0005 semantic chain: BC-2.01.009 → sister BCs → supplements → VP-009 → brief
2. SS-daemon-lifecycle pin chain: v1.0.29 referenced in 10 VP files and VP-INDEX after v1.0.30 landed
3. Brief orphan-promise chain: product-brief.md cited 3 retired monolithic BC IDs

All 3 integrity chains CLOSED by Round 5 commits.

---

## Recommended Routing (per finding)

| Agent | Findings Closed |
|-------|----------------|
| product-owner | F-R106-2, F-R106-3, F-R106-4, F-R106-5, F-R106-6, F-R106-8, F-R106-11, F-R106-12, F-R106-13, F-R106-15, F-R106-16, F-R106-17, F-R106-19, F-R106-20 |
| formal-verifier | F-R106-1, F-R106-9, F-R106-10, F-R106-18 |
| architect | F-R106-7 (arch portion), F-R106-14 |

---

## Consistency R45 Companion Findings

| ID | Severity | Class | Status | Closure Commit | Description |
|----|----------|-------|--------|---------------|-------------|
| GAP-R45-1 | HIGH | VP coverage gap | CLOSED | 7b8d6e8 | VP-009 had no probes for dual-accept alternate-header path (overlaps F-R106-1; FV closed both) |
| GAP-R45-2 | MED | Supplement stale pin | CLOSED | df5605a | interface-definitions.md auth-header semantics diverged from ADR-0005 (overlaps F-R106-3; PO closed both) |
| GAP-R45-3 | MED | Brief version mismatch | CLOSED | 56c11fe | product-brief.md cited stale SS-engine-module pin (overlaps F-R106-20; PO closed both) |
| GAP-R45-4 | LOW | VP-INDEX count | CLOSED | 7b8d6e8 | VP-INDEX v1.2 summary row count inconsistency (independent from R106 findings; FV closed) |
| GAP-R45-5 | LOW | CLAUDE.md routing example pin | CLOSED | SM-5F (main) | CLAUDE.md §Routing examples line 225 cited "current v1.1.15"; canonical post-Round-5 is v1.1.17 |

---

## §Trace (SE-16d cross-chain monotonicity)

**SE-16d audit (Round 5 chain):**

Prior chain high-water: STATE v5.64 at 2026-05-17T21:00:00Z.

Round 5 commit timestamps (CST → UTC):
- 03a4c57 (arch 5E): 2026-05-17 12:21:36 CST = 2026-05-17T17:21:36Z
- bb088a2 (PO 5A): 2026-05-17 12:22:42 CST = 2026-05-17T17:22:42Z
- 56c11fe (PO 5C factory-artifacts): 2026-05-17 12:20:49 CST = 2026-05-17T17:20:49Z
- fd790b8 (PO 5C main): 2026-05-17 12:21:09 CST = 2026-05-17T17:21:09Z
- df5605a (PO 5B): 2026-05-17 12:24:57 CST = 2026-05-17T17:24:57Z
- 7b8d6e8 (FV 5D): 2026-05-17 12:33:59 CST = 2026-05-17T17:33:59Z

All Round 5 commits occurred at UTC 17:20–17:33Z. STATE v5.64 was at 2026-05-17T21:00:00Z.
STATE v5.65 uses 2026-05-17T22:30:00Z.

**SE-16d monotonicity chain:** Round 5 commits (17:20–17:33Z) < STATE v5.64 (21:00Z) < STATE v5.65 (22:30Z).
**SE-16d verdict: PASS.** Monotonicity holds across the complete Round 5 → SM-5F closure chain.

---

## Closure Status

**Round 5 chain COMPLETE (6 commits):**
- bb088a2 (PO 5A): BC ADR-0005 cascade — BC-2.01.008/.009/.005/.002/.003/.007 versions + BC-INDEX v1.4 (§Trace ascending reorder). Closed: F-R106-2 CRIT, F-R106-7 HIGH (PO portion), F-R106-11/12/13 MED, F-R106-17 LOW.
- df5605a (PO 5B): Supplements + PRD pin refresh — test-vectors v1.1, interface-definitions v1.3 (dual-accept + /shutdown spec), nfr-catalog v1.2, error-taxonomy v1.1 (E-AUTH-003 added), PRD v1.26.4 (mass pin refresh + ADR-0005 added). Closed: F-R106-3 CRIT, F-R106-4/5/6 HIGH, F-R106-15/16 MED, GAP-R45-2 MED.
- 56c11fe + fd790b8 (PO 5C): Brief v1.4.25 (BC-DTU-001 → NFR-011, old IDs canonicalized, revision history split, SS-engine pin v1.1.15→v1.1.18) + CLAUDE.md brief version bump. Closed: F-R106-8 HIGH, F-R106-19/20 LOW, GAP-R45-3 MED.
- 7b8d6e8 (FV 5D): VP-009 v1.0.4 dual-accept expansion (15 probes — target ≥12 exceeded; 12 counter-examples; 3-dim fuzz harness; ADR-0005 cite) + 10 VPs SS-daemon-lifecycle pin v1.0.25→v1.0.30 + VP-INDEX v1.3 (4 combined fixes). Closed: F-R106-1 CRIT, F-R106-9/10 HIGH, F-R106-18 LOW, GAP-R45-1 HIGH, GAP-R45-4 LOW.
- 03a4c57 (Architect 5E): ADR-0005 v1.0.2 (path fix) + SS-daemon-lifecycle v1.0.30 (F-FC-I005 fabrication removal) + ARCH-INDEX v1.0.5 (trace-only bump). Closed: F-R106-14 MED, F-R106-7 HIGH (architect portion).
- SM 5F (this commit): STATE.md v5.65 + R106 report + GAP-R45-5 CLAUDE.md fix (main). Closed: GAP-R45-5 LOW.

**T-127'' re-audit cycle (R107 + cons R46) is the next required action.**
Counter: 0/3. A CLEAN R107 + CLEAN R46 advances counter to 1/3.

Codification candidates O-R106-1 + O-R106-2: both held at OBSERVATION status per D-114 Goodhart's law (first occurrence each; 3+ recurrences required). SE-18 candidate also held. No new disciplines codified in Round 5.
