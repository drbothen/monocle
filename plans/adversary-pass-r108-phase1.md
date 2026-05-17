---
document_type: adversary-pass
producer: adversary
version: "1.0"
level: ops
timestamp: 2026-05-18T00:00:00Z
traces_to: >
  D-047 strict pass 1 attempt 4 against restructured artifacts (T-127''').
  Cycle: cycle-001. Counter held at 0/3 (4th consecutive FAIL).
  Cons R47 companion report at .factory/plans/consistency-r47-phase1.md.
  Artifact set: PRD v1.26.5 (d92e4a7) + 22 BC files + 22 VP files +
  SS-daemon-lifecycle v1.0.31 + SS-engine-module v1.1.19 +
  SS-core-types-and-abi v1.2.12 + SS-forward-compatibility v1.2.16 +
  SS-deps-pin-manifest v1.1.17 + ARCH-INDEX v1.0.6 + BC-INDEX v1.5 +
  VP-INDEX v1.5 + L2-INDEX v1.0.7 + interface-definitions v1.4 +
  nfr-catalog v1.3 + error-taxonomy v1.2 + test-vectors v1.2 +
  ADR-0001..ADR-0005 (v1.0.2) + CAP-001/002/003 + dtu-assessment v1.7.3.
---

# Adversary Pass R108 — Phase 1 Spec Review

**Verdict: FAIL**

**Counter: 0/3 HELD** — R108 returned findings. Counter does not advance.

**Finding count: 22 substantive findings + 3 process-gap observations**
- CRIT: 4
- HIGH: 8
- MED: 6
- LOW: 4
- Process-gap observations: 3

**Status: AWAITING HUMAN ADJUDICATION ON CONVERGENCE STRATEGY**
(4th consecutive FAIL; escalated to human per orchestrator Production-Grade Default boundary)

---

## Summary

R108 is the 4th consecutive adversary FAIL against the restructured artifact set
(D-122 restructure). Finding count INCREASED from R107 (13) to R108 (22), reversing
the prior severity decay trend. This finding-count increase, combined with the
4-round consecutive-FAIL pattern (R105→14f, R106→25f, R107→18f, R108→22f), has
triggered orchestrator escalation to human for convergence strategy adjudication.

**Dominant defect classes this round:**
1. **VERSION FABRICATION (Round 6D regression):** 4 architecture documents + ARCH-INDEX
   claim "current canonical BC-INDEX is v1.4" when actual is v1.5 — a stale Round 6D
   snapshot reference that Round 6A (which produced v1.5) had not been reflected in.
2. **ADR-0005 cascade incomplete:** Round 6D fixed SS-forward-compatibility but did NOT
   complete the cascade into the /hooks/* endpoint interface files and /status endpoint,
   which remain single-header-only (no alias path).
3. **NFR fabricated §Out-of-Scope citations:** NFR-003/006/007/008/011 cite §Out-of-Scope
   sections in the product brief that do not exist at those locations — fabricated anchors.
4. **BC-INDEX §Trace ordering broken:** v1.5 row was inserted BEFORE the v1.4 row in the
   §Trace table, inverting chronological order (v1.5 appears at line N-2, v1.4 at line N).

**META-class summary:** 5 findings qualify as META-class (recurring fabrication patterns
where prior disciplines should have caught the instance). See §META-class Finding Count.

**Cons R47 companion:** 5 findings (1 HIGH + 2 MED + 2 LOW). HIGH finding is a WARN log
message schema divergence — two incompatible literal string forms used across the spec
corpus for the same warning condition. See §Cons R47 Companion Findings.

No closure actions taken. This report is a durable record for human adjudication.

---

## Findings

| ID | Severity | Class | Routing | Description | Evidence |
|----|----------|-------|---------|-------------|----------|
| F-R108-1 | CRIT | VERSION-FABRICATION | architect | 4 arch docs + ARCH-INDEX claim BC-INDEX "current canonical" is v1.4; actual is v1.5 (Round 6A d92e4a7). Round 6D snapshot view. | ARCH-INDEX.md §Document Map BC-INDEX row; SS-daemon-lifecycle §Trace; SS-engine-module §Trace; SS-core-types-and-abi §Trace |
| F-R108-2 | CRIT | CASCADE-INCOMPLETE | architect | ADR-0005 cascade incomplete — interface-definitions /hooks/* endpoint files + /status endpoint still specify single `X-Monocle-Authorization` header only; dual-header alias path (ADR-0005 v1.0.2) not reflected. Round 6D fixed SS-forward-compatibility only. | interface-definitions.md v1.4 §/hooks/* §Authentication; §/status §Authentication |
| F-R108-3 | CRIT | FABRICATED-ANCHOR | product-owner | NFR-003/006/007/008/011 cite §Out-of-Scope sections in product-brief.md at paths that do not exist. The anchor text resolves to nothing in brief v1.4.25. | nfr-catalog.md NFR-003 §Source, NFR-006 §Source, NFR-007 §Source, NFR-008 §Source, NFR-011 §Source |
| F-R108-4 | CRIT | TRACE-ORDER-BROKEN | product-owner | BC-INDEX.md §Trace table: v1.5 row appears BEFORE v1.4 row, inverting chronological ascending order required by SE-17g + §Trace ordering convention. | BC-INDEX.md §Trace, lines approx 90–100 |
| F-R108-5 | HIGH | STALE-PIN | architect | SS-deps-pin-manifest §Trace v1.1.17 cites ARCH-INDEX v1.0.5 as the authority for the §Trace authoring discipline (Round 4 reference); current ARCH-INDEX is v1.0.6 (98396fe Round 6D). | SS-deps-pin-manifest.md §Trace |
| F-R108-6 | HIGH | STALE-PIN | formal-verifier | VP-INDEX §Trace v1.5 cites BD14774 as the Round 6C commit SHA for VP-INDEX; correct (bd14774). However, it also claims all 22 VP files were swept in Round 6C — 3 VP files (VP-001, VP-002, VP-016) were NOT updated in Round 6C per git diff; their §References PRD cite remains at v1.26.3, not v1.26.5. | VP-001.md §References; VP-002.md §References; VP-016.md §References |
| F-R108-7 | HIGH | SCHEMA-DIVERGENCE | product-owner | interface-definitions.md v1.4 §HookEventRecord table lists `received_at` as field 7 (type: `DateTime<Utc>`). BC-2.01.007 lists `received_at` as field 7 with type `chrono::DateTime<Utc>`. error-taxonomy.md v1.2 §Processing references `received_at` with type `String (ISO 8601)`. Three incompatible type representations for the same field. | BC-2.01.007.md §Schema; interface-definitions.md §HookEventRecord; error-taxonomy.md §Processing |
| F-R108-8 | HIGH | STALE-PIN | architect | ARCH-INDEX §Document Map: SS-core-types-and-abi row shows version v1.2.11 (pre-Round 6D); actual version after Round 6D is v1.2.12 (98396fe). | ARCH-INDEX.md §Document Map SS-core-types-and-abi row |
| F-R108-9 | HIGH | STALE-PIN | formal-verifier | VP-009 v1.0.5 (bd14774) §Pre-conditions cites ADR-0005 v1.0.2 at line ~45; BC-2.01.009 §References also cites ADR-0005 v1.0.2 — these are correct. BUT VP-009 §Probe 2 (dual-accept behavior) references BC-2.01.009 Postcondition 2 which was renumbered in Round 6A; the postcondition index cited (PC-2) does not match the Round 6A canonical numbering (PC-1 is now dual-absent; PC-2 is alias-routed). Postcondition label mismatch. | VP-009.md §Probe 2 line ~80; BC-2.01.009.md §Postconditions |
| F-R108-10 | HIGH | MISSING-PROPAGATION | business-analyst | CAP-001 v1.3 §Phase 2 step 2 references "BC-2.01.009 dual-accept postconditions" without citing the ADR-0005 v1.0.2 alias reasoning. Round 6 added the ADR-0005 alias path but CAP-001 §Phase 2 body prose was not updated to reflect that step 2 now has TWO acceptable authorization outcomes, not one. | CAP-001.md §Phase 2, step 2 |
| F-R108-11 | HIGH | FABRICATED-EVIDENCE | architect | SS-forward-compatibility §Trace v1.2.16 (98396fe Round 6D) SE-17f NORMATIVE transcript block at [N-1] claims `$ grep -n "BC-2\\.SS\\." ... (17 hits)` but actual count after the Round 6D edit is 21 hits (4 additional BC IDs in the cascade section were updated but not enumerated). Count claim 17 vs actual 21. | SS-forward-compatibility.md §Trace [N-1] transcript block |
| F-R108-12 | HIGH | MISSING-PROPAGATION | product-owner | PRD v1.26.5 §3 NFR table — NFR-011 Validation Method column still reads "P4 holdout evaluation" verbatim from pre-restructure text; per D-122 restructure and nfr-catalog.md v1.3, NFR-011 validation method was updated to cross-reference VP-011 file. PRD §3 NFR table not synced with nfr-catalog v1.3 update. | prd.md §3 NFR table NFR-011 row; nfr-catalog.md NFR-011 |
| F-R108-13 | MED | STALE-PIN | product-owner | PRD v1.26.5 §6 lists supplement versions as of Round 6A; test-vectors.md is listed as v1.2 (correct) but the §6 line also says "nfr-catalog.md v1.2" — actual nfr-catalog after Round 6A is v1.3. §6 supplement inventory stale. | prd.md §6 Supplements table nfr-catalog row |
| F-R108-14 | MED | STALE-PIN | formal-verifier | VP-012.md §References lists BC-2.03.012 — this is the old monolithic BC ID. After D-122 renumbering, the canonical ID is BC-3.02.012. VP-012 was not fully updated in the D-122 chain or Round 4/5/6 sweeps. | VP-012.md §References first row |
| F-R108-15 | MED | SCHEMA-DIVERGENCE | architect | SS-core-types-and-abi v1.2.12 §HookEventRecord definition lists `payload_json: serde_json::Value` as field 5. BC-2.01.007 lists `payload_json: Option<serde_json::Value>` (Option-wrapped). Optionality mismatch between arch and BC for the same field. | SS-core-types-and-abi.md §HookEventRecord; BC-2.01.007.md §Schema |
| F-R108-16 | MED | MISSING-PROPAGATION | product-owner | error-taxonomy.md v1.2 §EC-013 was added in Round 6A (d92e4a7) but its §Source field cites only "BC-2.01.007" — per D-122 conventions, EC entries should also cite the SS-* doc anchor where the error is handled. No SS-* anchor cited for EC-013. | error-taxonomy.md §EC-013 row |
| F-R108-17 | MED | STALE-PIN | business-analyst | L2-INDEX v1.0.7 (fcf2b2d) §Trace cites brief v1.4.23 in the body narrative ("brief v1.4.23 pin corrected") but the CORRECTION target was v1.4.25 (the current canonical). The §Trace narrative reads as if v1.4.23 is the corrected-to version, which is wrong — v1.4.23 was the stale value that was replaced by v1.4.25. Narrative inversion. | L2-INDEX.md §Trace Round 6E entry |
| F-R108-18 | MED | TRACE-INTEGRITY | architect | SS-engine-module v1.1.19 §Trace (Round 6D entry) SE-17g INFORMATIONAL label applied to the NORMATIVE `$ grep` transcript block. The transcript is a literal `$ grep -nE` command with output — per SE-17g, this is NORMATIVE class and SE-17f re-run is required. It was labeled INFORMATIONAL, bypassing SE-17f re-verification. | SS-engine-module.md §Trace [N-1] block label |
| F-R108-19 | LOW | STALE-PIN | product-owner | BC-2.01.003.md §Traceability — DI-003 cell updated in Round 4 (D-128b) but the §Trace v1.0.4 entry (Round 6A d92e4a7) does not include a SE-17f NORMATIVE revalidation for the DI anchor. The §Trace entry closes a different fix (ADR pin sweep) without re-verifying the DI cell. | BC-2.01.003.md §Traceability; §Trace v1.0.4 |
| F-R108-20 | LOW | STALE-PIN | formal-verifier | VP-INDEX v1.5 §Trace total row count claim "22 VP files indexed" — this is correct. But the §Coverage column for VP-007 says "SS-02 (BC-2.02.007)" — the correct subsystem for VP-007 is SS-01 (BC-2.01.007 is in SS-01). VP-INDEX §Coverage column has VP-007 in wrong subsystem. | VP-INDEX.md §Coverage table VP-007 row |
| F-R108-21 | LOW | MISSING-PROPAGATION | product-owner | PRD v1.26.5 §7 RTM: BC-2.01.009 row Test File column still lists the pre-Round-6A test file name `test_bc_2_01_009_hook_dispatch.rs` — Round 6A updated the BC body postconditions (dual-accept) but did not update the RTM test file name to the canonical `test_bc_2_01_009_dual_accept.rs` from BC-2.01.009 §Verification. | prd.md §7 RTM BC-2.01.009 row Test File column |
| F-R108-22 | LOW | STALE-PIN | architect | ADR-0005 v1.0.2 §Status line reads "Accepted — 2026-05-17". However, ARCH-INDEX §Document Map ADR-0005 row Date column shows "2026-05-16" (pre-Round-4 date). Date discrepancy between ADR body and ARCH-INDEX row. | ADR-0005.md §Status; ARCH-INDEX.md §Document Map ADR-0005 row |

---

## Cons R47 Companion Findings

| ID | Severity | Class | Routing | Description |
|----|----------|-------|---------|-------------|
| GAP-R47-1 | HIGH | SCHEMA-DIVERGENCE | architect + product-owner | WARN log message schema divergence: SS-daemon-lifecycle v1.0.31 §Warn section uses literal string `"hook dispatch timeout; dropping event"` while interface-definitions v1.4 §Error Handling table uses `"hook_dispatch_timeout: event dropped"`. Two incompatible WARN message formats for the same condition. Implementation will produce inconsistent log output depending on which document the implementer follows. |
| GAP-R47-2 | MED | STALE-PIN | formal-verifier | VP-INDEX v1.5 §Active Disciplines row lists "SE-17g (33rd)" — this references the STATE.md discipline count register, not the VP-INDEX's own content. VP-INDEX is a spec artifact, not a state artifact; this citation is out of place and creates a cross-artifact coupling to STATE.md internal numbering. |
| GAP-R47-3 | MED | MISSING-PROPAGATION | product-owner | BC-2.01.008.md §Postconditions — PC-1 updated to dual-accept in Round 5 (bb088a2). The §Verification §Harness section still references a single-path test (`test_bc_2_01_008_accept.rs`), not a dual-path test (`test_bc_2_01_008_dual_accept.rs`). Test file name not updated to reflect dual-accept postcondition. |
| GAP-R47-4 | LOW | STALE-PIN | business-analyst | CAP-001 v1.3 §Trace Round 5 entry cites L2-INDEX v1.0.5 (b9e83bd). Actual L2-INDEX version after Round 6E is v1.0.7 (fcf2b2d). The §Trace entry is historical and records the Round 5 state correctly, but the §References section at the bottom of CAP-001 v1.3 was not updated to reflect the L2-INDEX version bump through Round 6E. |
| GAP-R47-5 | LOW | STALE-PIN | product-owner | test-vectors.md v1.2 §Coverage table lists PRD §3 BC count as "22 behavioral contracts" — this is correct. However, it also cites "BC-INDEX v1.4" as the source — BC-INDEX is v1.5 after Round 6A (d92e4a7). test-vectors.md §Coverage BC-INDEX cite stale. |

---

## Counter Decision

**FAIL — counter holds 0/3.**

R108 returned 22 substantive findings (4 CRIT + 8 HIGH + 6 MED + 4 LOW). Counter cannot
advance on a FAIL round. No finding of any severity was waived.

---

## META-class Finding Count

5 findings qualify as META-class (a prior codified discipline should have caught the
instance but did not):

| Finding | META Class | Discipline That Should Have Caught |
|---------|-----------|-----------------------------------|
| F-R108-1 | VERSION-FABRICATION | SE-17g NORMATIVE sweep — §Trace entries citing v1.4 are NORMATIVE (current-version claims) |
| F-R108-3 | FABRICATED-ANCHOR | Extension 3 + SE-17a — brief §Out-of-Scope cite should have been grep-verified |
| F-R108-4 | TRACE-ORDER-BROKEN | SE-17c final-state revalidation — §Trace ascending order is a structural requirement |
| F-R108-11 | FABRICATED-EVIDENCE | SE-17f NORMATIVE self-revalidation — transcript `(17 hits)` should have been re-run post-edit |
| F-R108-18 | TRACE-INTEGRITY | SE-17g NORMATIVE/INFORMATIONAL taxonomy — literal `$ grep` transcript is definitionally NORMATIVE |

---

## Divergence Pattern Documentation

4-round trajectory (restructured artifact set, D-047 strict pass 1 attempts 1–4):

| Round | Adversary | Cons | Total Findings | Notes |
|-------|-----------|------|---------------|-------|
| Round 4 (R105) | 14f | +5f (R44) | 19 | CRIT: HookEventRecord schema 3-way divergence |
| Round 5 (R106) | 20f | +5f (R45) | 25 | 3 CRIT; ADR-0005 cascade dominant |
| Round 6 (R107) | 13f | +5f (R46) | 18 | 2 CRIT; fabrication introductions in fix bursts |
| Round 7 (R108) | 22f | +5f (R47) | 27 | 4 CRIT; INCREASED from R107; pattern confirmed |

**Pattern:** Each closure burst introduces new defects while closing prior ones. Finding
counts have NOT converged to zero across 4 rounds. The divergence trajectory (19 → 25 →
18 → 27) shows no monotonic decay. This is the 4th consecutive FAIL under D-047 strict.

---

## Critical Findings Detail

### F-R108-1 (CRIT): BC-INDEX Version Fabrication — Round 6D Stale Snapshot

4 architecture documents plus ARCH-INDEX.md contain the phrase "current canonical BC-INDEX
is v1.4" or equivalent (citing BC-INDEX v1.4 as the live version). Round 6A (commit
d92e4a7) updated BC-INDEX to v1.5. Round 6D (commit 98396fe) updated the architecture
documents but retained the v1.4 reference in at least 4 locations as a snapshot artifact
from the Round 6D author context.

This is classified as the "Round 6D fabrication" pattern — the same class as F-R107-1
(CRIT; ADR-0005 pin inconsistency). Every closure burst that touches §Trace entries is
producing new fabrications in the "current canonical version" citation zone.

**Files affected:** ARCH-INDEX.md §Document Map BC-INDEX row + §Trace body; SS-daemon-lifecycle.md §Trace Round 6D entry; SS-engine-module.md §Trace Round 6D entry; SS-core-types-and-abi.md §Trace Round 6D entry.

### F-R108-2 (CRIT): ADR-0005 Cascade Incomplete — /hooks/* and /status Still Single-Header

ADR-0005 v1.0.2 mandates that all hook endpoints MUST accept EITHER `X-Monocle-Authorization`
(canonical) OR `X-Claude-Code-Ide-Authorization` (alias, per D-128/T-128m). Round 6D
updated SS-forward-compatibility.md (17 BC IDs canonicalized). Round 5 updated BC files and
prd-supplements partially. However, interface-definitions.md v1.4 §Authentication sections
for the /hooks/* endpoint group and /status endpoint were NOT updated to show the dual-header
acceptance. They remain single-header-only (`X-Monocle-Authorization` only), directly
contradicting ADR-0005 v1.0.2.

This is the 2nd cascade-incomplete CRIT in consecutive rounds (R107-1 was ADR-0005 v1.0.2
pin inconsistency; R108-2 is ADR-0005 cascade endpoint coverage gap).

**Files affected:** interface-definitions.md v1.4 §/hooks/* §Authentication (lines ~30–45);
§/status §Authentication (lines ~70–80).

### F-R108-3 (CRIT): NFR §Out-of-Scope Fabricated Anchors

NFR-003 (Latency), NFR-006 (Throughput), NFR-007 (Startup Time), NFR-008 (Platform), and
NFR-011 (Plugin Sandbox) each cite a §Out-of-Scope subsection in product-brief.md as the
normative source for their scope boundary. These subsections do not exist at the cited paths
in brief v1.4.25. The product brief has an §Explicit Non-Goals section but no per-NFR
§Out-of-Scope subsection structure.

This fabrication pattern dates to the nfr-catalog.md v1.0 initial authoring (pre-D-122);
it survived through 5 NFR catalog updates (v1.0 through v1.3) without any sweep catching
it because Extension 3 and SE-17a do not specifically cover brief-anchor existence.

This finding is a 2nd occurrence of the "semantic anchor existence check absent" class
(previously O-R107-2 process-gap observation — now promoted to CRIT finding because 5 NFRs
are affected and the anchors are normative references, not informational).

**Files affected:** nfr-catalog.md NFR-003/006/007/008/011 §Source anchor cells.

### F-R108-4 (CRIT): BC-INDEX §Trace Ordering Broken — v1.5 Before v1.4

BC-INDEX.md §Trace table has the v1.5 entry (d92e4a7, Round 6A) appearing at a HIGHER
line number than the v1.4 entry — meaning v1.5 is listed BEFORE v1.4 in reading order.
The §Trace convention requires ascending chronological order (oldest entry first). This
inversion makes the §Trace history misleading and violates SE-17c structural correctness.

The v1.5 row was inserted by the Round 6A PO burst (d92e4a7) without verifying that it
was appended AFTER the v1.4 row. It appears the insertion occurred at the wrong position
in the §Trace table.

**File affected:** BC-INDEX.md §Trace table, approximately lines 88–105.

---

## Process-Gap Observations

| ID | Class | Description | Occurrence Count |
|----|-------|-------------|-----------------|
| O-R108-1 | CODIFICATION-ELIGIBLE | "commit pending" placeholder convention: fix-burst planning documents use "commit pending" as a placeholder for SHAs not yet known. This placeholder has appeared in 3 consecutive STATE.md burst records (Round 4, Round 5, Round 6) without a resolution mechanism — the placeholder persists into the final SM commit rather than being replaced with the actual SHA. This is the 3rd occurrence; D-114 threshold (3+) is MET. Codification eligible: define a sweep step that greps for "commit pending" before final SM commit and replaces with actual SHA from `git log --oneline -1`. | 3rd (CODIFICATION THRESHOLD MET per D-114) |
| O-R108-2 | NEW-CLASS | Finding-ID re-purposing across closure narratives: in Round 6's closure burst records, F-R107-6 (originally the "BC-INDEX self-referential SHA cite" finding) is referenced in the Round 6A PO dispatch narrative as "F-R107-6 = historical-trace-pin" — but F-R107-6 per the R107 report was the BC-INDEX SHA finding, not the trace-pin policy. The finding ID was re-used with a different semantic payload in the closure narrative. This creates audit-trail ambiguity and is a NEW defect class. Observation only (1st occurrence; not codification-eligible per D-114). | 1st |
| O-R108-3 | CODIFICATION-ELIGIBLE | "current canonical" fabrication in §Trace entries: Round 6D §Trace entries used "current canonical X is vN.M" phrasing to assert the live version of sibling artifacts. This assertion form has appeared in CRIT findings in Round 6 (F-R107-1) and now Round 7 (F-R108-1) — 2nd occurrence of the FINDING-class (both were CRIT). The process-gap root cause (agents making live-version claims in §Trace without SE-17f re-verification) is a 2nd occurrence as a PROCESS observation. Codification eligible: define a rule that §Trace entries MUST NOT assert "current canonical vN.M" for sibling artifacts without a NORMATIVE grep confirming the version at commit time. | 2nd (approaching D-114 threshold) |

---

## Status

**AWAITING HUMAN ADJUDICATION ON CONVERGENCE STRATEGY.**

R108 returned FAIL with 22 findings (INCREASED from R107's 13). The 4-consecutive-round
divergence pattern (R105→14f, R106→25f, R107→18f, R108→22f) shows no convergence.
Counter held at 0/3 for 4 rounds.

**Orchestrator has escalated to human per Production-Grade Default boundary:** convergence
strategy selection (continue, accept residuals, or structural process change) is a genuine
human decision involving risk acceptance and scope-vs-deadline tradeoffs.

**Codification candidates at threshold (per D-114 — 3+ occurrences):**
- O-R108-1: "commit pending" resolution (3rd occurrence — THRESHOLD MET)
- O-R107-1: input-hash path-existence hook (2nd occurrence as process gap; F-R108 promoted to CRIT for anchor existence)
- O-R108-3: "current canonical" fabrication in §Trace (2nd occurrence as process-gap; was F-R107-1 CRIT + F-R108-1 CRIT)
- SE-18: commit-burst hygiene (2nd occurrence from Rounds 3 + 6 co-mingling)

No closure, no codification, no dispatch initiated. Preserve as audit record.
