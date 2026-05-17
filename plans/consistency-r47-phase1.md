---
document_type: consistency-report
level: ops
version: "1.0"
producer: vsdd-factory:consistency-validator
timestamp: 2026-05-17T23:45:00Z
phase: phase-1-spec-crystallization
cycle: cycle-001
round: R47
counter: 0/3
traces_to: "STATE.md v5.66; adversary R108 (parallel)"
---

# Consistency Report R47 — Phase 1 Spec Package

> **Companion to adversary R108.** Post-Round-6 / post-R107 closure audit.
> Counter position: 0/3 (requires CLEAN result to advance to 1/3).

---

## §Summary

| Dimension | Status | Notes |
|-----------|--------|-------|
| Spec ID references (BC, VP, NFR, DI, ADR, E-XXX, EC) | PASS | All sampled IDs resolve |
| Counts (22 BCs, 22 VPs, 5 ADRs, 7 DIs, 3 CAPs, 7 EC in BC-2.01.009, 8 test vectors BC-2.01.009) | PASS | All counts verified correct |
| Naming consistency | PASS | BC-S.SS.NNN form enforced across BCs, PRD, BC-INDEX |
| Traceability chains (L1→L2→L3→L4) | PASS | All 22 BCs trace to BCs, VPs trace to BCs, BCs trace to CAPs |
| Frontmatter canonical fields | PASS | All checked artifacts carry required fields |
| ADR-0005 cascade integrity | FAIL | One WARN message schema divergence across 2 supplements |
| EC-013 / /shutdown cross-reference consistency | PASS | Probe 9.5 covers the EC-013 behavior; /shutdown endpoint documented in interface-definitions.md |
| E-AUTH-003 cross-reference consistency | FAIL | E-AUTH-003 message text in error-taxonomy.md ≠ BC-2.01.009 INV-6 / ADR-0005 canonical |
| Stale version pins (Round 6D informational bumps) | FAIL | Three arch docs got INFORMATIONAL-only bumps that propagated to version numbers not reflected in VP-INDEX, BC-INDEX, PRD traces_to |
| Unresolved commit-pending placeholders | FAIL | VP-009 §References carries 4 unresolved "commit pending" placeholders |
| PRD traces_to placeholder | FAIL | "pending BA Dispatch 6" stale — Dispatch 6 complete since 2026-05-17T14:00:00Z |

**Verdict: GAPS**
**GAP count: 1 HIGH, 2 MEDIUM, 2 LOW**
**PASS dimensions: 7 of 11 checked**

---

## §Findings

### GAP-R47-1 | HIGH | WARN log message schema divergence (E-AUTH-003 / ADR-0005 cascade)

**Severity:** HIGH
**Routing:** vsdd-factory:product-owner (error-taxonomy.md) + vsdd-factory:formal-verifier (interface-definitions.md verification text)

**Description:**

The canonical WARN deprecation log message emitted on every alias-path authentication attempt is specified as TWO DIFFERENT literal strings across artifacts:

**String A (BC-2.01.009 INV-6, ADR-0005 §Decision Priority 2, VP-009 §Pre-conditions, VP-009 §Property Statement):**
```
WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization
```

**String B (error-taxonomy.md E-AUTH-003 Message Format, interface-definitions.md §Compatibility Alias Header §WARN log, interface-definitions.md §Auth Response Examples):**
```
X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization
```
(with `WARN:` prefix added in the Error Catalog `WARN log` column context)

**Evidence:**

- BC-2.01.009 INV-6 (line 61): `The log message is: \`WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization\``
- ADR-0005 §Decision Priority 2 (line 96): `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization`
- VP-009 §Pre-conditions (lines 146-148): same string A form
- error-taxonomy.md E-AUTH-003 row (line 40): `WARN: X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization`
- interface-definitions.md line 236: `"X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization"`
- interface-definitions.md lines 265, 272: `WARN log emitted: "X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization"`

**Impact:** The implementer will find two incompatible literal strings for a single normative log line. An integration test writer asserting the literal log string (required by VP-009 §Pre-conditions for WARN-log presence/absence assertions) cannot satisfy both. Production code using one string will fail VP-009 probe-matrix WARN-log assertions built against the other.

**Fix:** Standardize on String A (`WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization`) — it is the form present in three independent authoritative artifacts (BC-2.01.009 INV-6, ADR-0005, VP-009). Update:
1. `prd-supplements/error-taxonomy.md` E-AUTH-003 Message Format column.
2. `prd-supplements/interface-definitions.md` §Compatibility Alias Header §WARN log + all `WARN log emitted:` rows.

---

### GAP-R47-2 | MEDIUM | VP-009 §References stale BC version pin + unresolved commit-pending placeholders

**Severity:** MEDIUM
**Routing:** vsdd-factory:formal-verifier

**Description:**

VP-009 §References (post-v1.0.5 state) carries multiple stale references:

1. **BC-2.01.009 version pin:** `Source contract: behavioral-contracts/ss-01/BC-2.01.009.md v1.0.3` — BC-2.01.009 is currently at v1.0.4 (F-R107-10 added EC-013 in Round 6A PO dispatch at 2026-05-17T23:30:00Z).

2. **Unresolved `commit pending` placeholders (4 instances):**
   - `BC-2.01.009.md v1.0.3 (commit pending — F-R106-7 fabricated-FC-ID removal + ADR-0005 dual-accept propagation)`
   - `BC-INDEX.md v1.5 (commit pending — PO 6A R107 Round 6A finalization; ...)`
   - `SS-daemon-lifecycle.md v1.0.30 §Start Sequence (commit pending — architect 5E F-FC-I005 removal + dual-accept consolidation)`
   - `prd.md v1.26.5 §BC-2.01.009 (Dispatch 4 commit 1030c65; refreshed to v1.26.5 in F-R107-3 / GAP-R46-1 closure, parallel PO 6B dispatch — commit pending; ...)`

   All four `commit pending` annotations were written during concurrent-dispatch coordination in Rounds 5D and 6C. Those commits have since landed (confirmed by actual file versions on disk), but VP-009 §References was not updated with the concrete SHAs.

**Evidence:**
- VP-009 §References line 87: `BC-2.01.009 v1.0.3`
- VP-009 §References lines 430, 432, 434, 435: `commit pending`
- BC-2.01.009 frontmatter: `version: "1.0.4"` (2026-05-17T23:30:00Z)
- BC-INDEX.md frontmatter: `version: "1.5"` (2026-05-17T23:30:00Z) — commit exists on disk

**Fix:** Update VP-009 §References:
- Refresh BC-2.01.009 pin to v1.0.4 with note about EC-013 addition.
- Resolve all four `commit pending` annotations to concrete commit SHAs (obtainable via `git log --oneline --follow -- .factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md` etc.).

---

### GAP-R47-3 | MEDIUM | PRD `traces_to` stale placeholder for L2-INDEX

**Severity:** MEDIUM
**Routing:** vsdd-factory:product-owner

**Description:**

PRD v1.26.5 frontmatter `traces_to` field ends with:
```
domain-spec/L2-INDEX.md (pending BA Dispatch 6)
```

Dispatch 6 was completed at 2026-05-17T14:00:00Z (§Trace v1.0, L2-INDEX v1.0). L2-INDEX is now at v1.0.7 (2026-05-17T23:00:00Z). The "(pending BA Dispatch 6)" placeholder was never resolved after the dispatch completed.

**Evidence:**
- PRD v1.26.5 frontmatter line 11: `...domain-spec/L2-INDEX.md (pending BA Dispatch 6)"`
- L2-INDEX.md frontmatter: `version: "1.0.7"` `timestamp: 2026-05-17T23:00:00Z`
- L2-INDEX §Trace v1.0: `2026-05-17T14:00:00Z` — Dispatch 6 completed and artifact exists on disk

**Fix:** Update PRD `traces_to` to replace `domain-spec/L2-INDEX.md (pending BA Dispatch 6)` with `domain-spec/L2-INDEX.md v1.0.7` (or the current version at time of fix, with appropriate SE-16d monotonicity check).

---

### GAP-R47-4 | LOW | Stale architecture-source version pins after Round 6D INFORMATIONAL-only bumps

**Severity:** LOW
**Routing:** vsdd-factory:formal-verifier (VP-INDEX), vsdd-factory:product-owner (PRD traces_to)

**Description:**

Round 6D (F-R107-8 architect dispatch, 2026-05-17T23:00:00Z) bumped three architecture subsystem documents with INFORMATIONAL-only changes (§Trace historical-pin clarification, zero normative content delta):
- `SS-daemon-lifecycle.md`: v1.0.30 → v1.0.31
- `SS-core-types-and-abi.md`: v1.2.11 → v1.2.12
- `SS-engine-module.md`: v1.1.18 → v1.1.19

The following artifacts were not updated to reflect these new version numbers:
- VP-INDEX.md §SS-01 header: `Architecture source: SS-daemon-lifecycle.md v1.0.30` (actual: v1.0.31)
- VP-INDEX.md §SS-02 header: `Architecture source: SS-core-types-and-abi.md v1.2.11` (actual: v1.2.12)
- VP-INDEX.md §SS-03 header: `Architecture source: SS-engine-module.md v1.1.18` (actual: v1.1.19)
- All 10 ss-01 BC files §Traceability Architecture Source: `SS-daemon-lifecycle.md v1.0.30` (actual: v1.0.31)
- PRD v1.26.5 `traces_to`: `SS-core-types-and-abi.md v1.2.11; SS-engine-module.md v1.1.18` (actuals: v1.2.12, v1.1.19)

**Mitigating factor:** All three v→v+1 bumps were INFORMATIONAL-only (§Trace historical-pin clarification, identical substantive §Trace wording per §Trace v1.0.31/v1.2.12/v1.1.19 entries). No normative content changed. The downstream consumers receive correct normative information even with stale version pins. Severity is LOW, not MAJOR, because no correctness regression results.

**Evidence:**
- SS-daemon-lifecycle.md frontmatter: `version: "1.0.31"` `timestamp: 2026-05-17T23:00:00Z`
- SS-daemon-lifecycle.md §Trace v1.0.31: "INFORMATIONAL: §Trace v1.0.28 BC-INDEX cite... expanded to historical-pin form"
- SS-core-types-and-abi.md frontmatter: `version: "1.2.12"` — §Trace v1.2.12: "INFORMATIONAL"
- SS-engine-module.md frontmatter: `version: "1.1.19"` — §Trace v1.1.19: "INFORMATIONAL"
- VP-INDEX §SS-01 header: pin at v1.0.30
- VP-INDEX §SS-02 header: pin at v1.2.11
- VP-INDEX §SS-03 header: pin at v1.1.18

**Fix:** Update VP-INDEX §SS-01/SS-02/SS-03 architecture-source pins, 10 ss-01 BC §Traceability Architecture Source rows for SS-daemon-lifecycle, and PRD `traces_to` SS-core-types-and-abi + SS-engine-module pins to reflect the Round 6D post-informational-bump versions.

---

### GAP-R47-5 | LOW | VP-009 has no explicit EC-013 cross-reference annotation

**Severity:** LOW
**Routing:** vsdd-factory:formal-verifier (informational; no correctness impact)

**Description:**

EC-013 was added to BC-2.01.009 in Round 6A (F-R107-10) at timestamp 2026-05-17T23:30:00Z. EC-013 covers the `Authorization: Bearer <token>` case (neither recognized auth header present → dual-absence → `missing_auth_token`).

VP-009 v1.0.5 was finalized in Round 6C (2026-05-17T23:00:00Z) before EC-013 was added to the BC. VP-009 probe 9.5 already covers the identical behavior (`Authorization: Bearer fake-token` (Bearer header only; neither recognized header present) → 401 `missing_auth_token`, WARN log absent). Counter-example 1 in VP-009 also describes this specific anti-pattern.

However, VP-009 §Source Contract / §References do not cite EC-013 by ID, making it non-obvious that probe 9.5 is the test vehicle for EC-013. The BC-2.01.009 §Trace v1.0.4 explicitly says EC-013 aligns with "VP-009 probe 9.5 and test-vectors row 5" — but this annotation exists only in the BC, not in the VP.

**Evidence:**
- VP-009 probe 9.5: `Authorization: Bearer fake-token` → 401 `missing_auth_token` (present)
- BC-2.01.009 §Trace v1.0.4 line 149: "EC-013 aligns with probe 9.5 and test-vectors row 5" (in BC §Trace)
- VP-009: `grep -n "EC-013" vp-009-auth-header-validation.md` → 0 matches
- BC-2.01.009 canonical test vector row 5: `Authorization: Bearer fake-token` → `missing_auth_token`

**Fix:** Add an EC-013 annotation to VP-009 probe 9.5 description, e.g., append `(BC-2.01.009 EC-013 — Bearer-fallback dual-absence case)` to probe 9.5's Header column. This is the bidirectional traceability annotation — BC cites VP, VP should cite back to the EC.

---

## §PASS Dimensions

The following dimensions were checked and returned PASS:

1. **Spec ID resolution**: Sampled BC-S.SS.NNN (all 22 IDs), VP-NNN (all 22 IDs), ADR-NNN (5 IDs — ADR-0001 through ADR-0005), DI-NNN (DI-001 through DI-007), EC-NNN (EC-007 through EC-013 in BC-2.01.009), E-XXX-NNN (E-AUTH-001 through E-AUTH-003, E-DAEMON-001 through E-DAEMON-004, E-LOCK-001 through E-LOCK-003, E-ENG-001, E-FACT-001, E-FACT-002, E-RING-001, E-PROTO-001) — all 15 error codes resolve, all IDs are unique and non-reused.

2. **Count arithmetic**:
   - BC-INDEX: 22 total (10 SS-01 + 8 SS-02 + 4 SS-03) VERIFIED CORRECT
   - VP-INDEX: 22 total (10 SS-01 + 8 SS-02 + 4 SS-03) VERIFIED CORRECT
   - ADRs: 5 (ADR-0001 through ADR-0005) VERIFIED CORRECT
   - DIs: 7 (DI-001 through DI-007 in L2-INDEX + CAP section files) VERIFIED CORRECT
   - CAPs: 3 (CAP-001, CAP-002, CAP-003) — note: the validation prompt cited "6 CAPs" but actual domain spec has 3; the 3-CAP count matches the CANONICAL ARTIFACT PINS specification which reads "3 CAPs (CAP-001 v1.3)" VERIFIED CORRECT
   - BC-2.01.009 test vectors: 8 in BC canonical table, 8 in test-vectors.md BC Vector Index VERIFIED CORRECT
   - BC-2.01.009 edge cases post-EC-013: EC-007, EC-008, EC-009, EC-010, EC-011, EC-012, EC-013 = 7 ECs VERIFIED CORRECT
   - Error codes: 15 (E-AUTH-001..003, E-DAEMON-001..004, E-LOCK-001..003, E-ENG-001, E-FACT-001..002, E-RING-001, E-PROTO-001) VERIFIED CORRECT

3. **Naming consistency**: All BCs use BC-2.SS.NNN form in BC-INDEX, PRD §2, and individual BC file H1 headings. PRD §2 titles match BC-INDEX titles for all 22 BCs (sampled 12 pairs, all match). Old BC-ID forms (BC-DAEMON-NNN, BC-AUTH-NNN, etc.) appear only in historical/renumbering contexts.

4. **Traceability chains**: All 22 BCs have `traces_to: prd.md` in frontmatter. All 22 VPs have `traces_to: prd.md` and `source_bc: BC-2.SS.NNN` in frontmatter. PRD traces to product-brief.md + vision-synthesis. L2-INDEX traces to product-brief.md. BC-INDEX traces to prd.md. VP-INDEX traces to prd.md.

5. **ADR-0005 cascade integrity (11 surfaces)**: The 11 target surfaces identified in Round 4/5/6:
   - BC-2.01.008 ✓ (dual-accept in Postcondition 4)
   - BC-2.01.009 ✓ (dual-accept postconditions 1-4, invariants 5-7)
   - BC-2.01.004 INV-3 ✓ (updated to dual-accept per GAP-R46-5)
   - SS-daemon-lifecycle.md ✓ (router-level dual-accept protocol documented)
   - ADR-0005 v1.0.2 ✓ (canonical ADR document)
   - VP-009 ✓ (dual-accept probe matrix categories A/B/C)
   - CAP-001-daemon-lifecycle.md ✓ (alias note in §P2 Hook Event Ingestion)
   - interface-definitions.md ✓ (/shutdown endpoint + dual-accept semantics)
   - error-taxonomy.md ✓ (E-AUTH-001 dual-absence, E-AUTH-003 entry present)
   - test-vectors.md ✓ (8 vectors including alias-path rows 7-8)
   - dtu-assessment.md ✓ (not re-read this round; verified per Round 6 closure record)
   - **The WARN message text divergence is a sub-issue within this cascade (see GAP-R47-1), but the structural presence of ADR-0005 content across all 11 surfaces is confirmed.**

6. **EC-013 / /shutdown cross-reference**: VP-009 probe 9.5 covers `Authorization: Bearer fake-token` (=EC-013 behavior) correctly. interface-definitions.md §POST /shutdown is present with auth behavior documented (added in Round 5 per F-R106-6). BC-2.01.009 precondition 2 explicitly lists `/shutdown` as an authenticated endpoint.

7. **Frontmatter canonical fields**: All checked artifacts carry `document_type`, `level`, `version`, `producer`, `traces_to`, `timestamp` in correct form. No missing required fields found in BC-INDEX v1.5, VP-INDEX v1.5, ARCH-INDEX v1.0.6, L2-INDEX v1.0.7, PRD v1.26.5, ADR-0005 v1.0.2, BC-2.01.009 v1.0.4, VP-009 v1.0.5.

---

## §Open Observations

**OBS-R47-1 (INFORMATIONAL):** The validation prompt cited "6 CAPs" in the count check list. The current domain spec has 3 CAPs (CAP-001, CAP-002, CAP-003) which is correct per the canonical artifact pins specification `3 CAPs (CAP-001 v1.3)`. The "6 CAPs" in the prompt appears to be a prompt-authoring error, not a spec defect.

**OBS-R47-2 (INFORMATIONAL):** VP-009 v1.0.5 carries a stale `"Current as of 2026-05-17T22:30:00Z (R106 Round 5D)"` note in §References — this is historical provenance text that records when the expansion was done, not a live version claim. The note is informational and should not be confused with a stale cite.

**OBS-R47-3 (PROCESS):** The Round 6D F-R107-8 INFORMATIONAL-only bumps to three arch docs (SS-daemon-lifecycle, SS-core-types-and-abi, SS-engine-module) introduced a new recurring class of stale-pin: downstream artifacts were not refreshed because the bump was informational. A process discipline (SE-18 candidate) could be: when any arch SS doc bumps version — even for INFORMATIONAL-only §Trace changes — the dispatch must include VP-INDEX/BC-INDEX architecture-source pin sweep as part of the same commit. This would prevent GAP-R47-4 from recurring.

---

## §Restructure Consistency Verdict

All R107 Round 6 full-closure targets confirmed present:
- 10 BCs updated with architecture-source pin v1.0.30 (F-R107-2) ✓
- BC-2.01.004 INV-3 dual-accept update (GAP-R46-5) ✓
- BC-2.01.009 EC-013 added (F-R107-10) ✓
- ADR-0005 v1.0.2 pin added to BC-2.01.008 + BC-2.01.009 (F-R107-9) ✓
- BC-INDEX v1.5 §Trace monotonicity (v1.4 → v1.5) ✓
- VP-INDEX v1.5 cascade (PRD + BC-INDEX cite refresh) ✓
- L2-INDEX v1.0.7 brief cite refresh (v1.4.23 → v1.4.25) ✓
- All 22 VPs PRD cite refresh to v1.26.5 (GAP-R46-1) ✓

ADR-0005 cascade structural completeness: PASS (11 surfaces present)
WARN message text schema: FAIL (GAP-R47-1 — two incompatible literal strings)

---

## §Validation Gate Result

**GATE: FAIL**

**Blocking findings:**
- GAP-R47-1 HIGH: WARN log message schema divergence blocks gate. An implementer must choose between two incompatible literal strings for a normative log line. Produces a definite integration test failure in Phase 3 (VP-009 WARN-log assertion will fail against whichever string the implementer does not use).

**Non-blocking findings (counter does not advance until HIGH is resolved):**
- GAP-R47-2 MEDIUM: VP-009 BC version pin stale + unresolved commit-pending (no correctness impact; traceability gap)
- GAP-R47-3 MEDIUM: PRD traces_to stale placeholder (no correctness impact; audit trail gap)
- GAP-R47-4 LOW: INFORMATIONAL-only arch version pin stale across VP-INDEX/BC-INDEX/PRD (no normative content impact)
- GAP-R47-5 LOW: VP-009 missing EC-013 ID annotation at probe 9.5 (behavior covered; bidirectional ID annotation missing)

**Counter advance:** 0/3 → 0/3 (no advance; GAP-R47-1 HIGH blocks)

---

*Report produced by vsdd-factory:consistency-validator, round R47, 2026-05-17T23:45:00Z.*
