---
document_type: adversary-pass
pass_id: R105
attempt: 1
policy: D-047-strict
counter_before: "0/3"
counter_after: "0/3"
verdict: FAIL
timestamp: 2026-05-17T19:00:00Z
producer: vsdd-factory:adversary
artifact_pins:
  - artifact: prd.md
    version: "1.26.1"
  - artifact: BC-INDEX.md
    version: "1.1"
  - artifact: VP-INDEX.md
    version: "1.1"
  - artifact: ARCH-INDEX.md
    version: "1.0.1"
  - artifact: L2-INDEX.md
    version: "1.0.2"
  - artifact: SS-deps-pin-manifest.md
    version: "1.1.17"
disciplines_in_force: 33
findings_count:
  critical: 1
  high: 4
  medium: 6
  low: 3
observations_count: 2
---

# Adversary Pass R105 — Phase 1 Spec Crystallization

**Policy:** D-047 strict (pass 1 attempt 1 against restructured artifacts)
**Verdict:** FAIL — 14 findings (1 CRIT + 4 HIGH + 6 MED + 3 LOW + 2 process-gap observations)
**Counter:** 0/3 (HOLDS; first attempt against restructured artifacts post D-122 restructure)

## Summary

D-047 strict pass 1 attempt 1 against the restructured artifact set (post D-122 template-compliance
remediation chain, audit R3 CLEAN milestone D-126). The D-122 restructure was structurally sound
— it correctly sharded the monolithic PRD (4480 → 282 lines) and VP monolith (15,612 → 0 lines)
into per-file indexed artifacts — but was substantively incomplete: it did not back-propagate
updated content into supplements, L2 invariants, or cross-reference IDs across the architecture
documents.

**CRITICAL anchor (F-R105-1): HookEventRecord schema diverges across 3 canonical artifacts.**
BC-2.01.007.md specifies 7 fields; interface-definitions.md specifies 6 fields with different field
names; CAP-001-daemon-lifecycle.md specifies 5 fields with opaque payload. This 3-way divergence
BLOCKS Phase 3 TDD — implementers cannot determine the canonical schema.

**Structural observation (O-R105-1):** The validate-template-compliance gate (D-123) correctly
catches structural non-compliance. A second gate — sibling-propagation gate — is MISSING. The
perimeter-checking gate verifies structure; no gate verifies that content was back-propagated to
all consumer artifacts (supplements, L2, architecture docs, manifest). This missing gate is why
all 14 R105 findings survived into the restructured artifact set.

**Content fidelity verdict:** BC and VP extraction during D-122 was CLEAN — substantive content
from the monolithic forms was faithfully preserved. The failures are propagation gaps: consumers
(supplements, L2 invariants, architecture docs, manifest) were not updated to match the new
canonical structure.

---

## Findings

### F-R105-1 — CRITICAL

**Severity:** CRITICAL
**Class:** Schema divergence — 3-way canonical conflict
**Routing:** product-owner + business-analyst (coordinated dispatch)
**Task:** T-128a

**Description:** HookEventRecord schema diverges across 3 canonical artifact locations.

**Evidence:**
- BC-2.01.007.md line 50: 7 fields
  ```
  format_version: u32
  session_id: String
  timestamp_micros: i64
  pid: u32
  hook_type: String
  tool_name: Option<String>
  tool_input: Option<serde_json::Value>
  ```
- interface-definitions.md line 225: 6 fields, uses `received_at` instead of `timestamp_micros`,
  no `pid` field
- CAP-001-daemon-lifecycle.md lines 78-84: 5 fields with `payload_json` opaque blob instead of
  structured `tool_name`/`tool_input` fields

**Impact:** Phase 3 TDD blocked. Implementer cannot determine which schema to code to. Integration
tests will fail against whichever variant the implementation chooses. Three separate agents citing
three different field sets produces an unresolvable conflict at implementation time.

**Fix scope:** BC-2.01.007.md is the canonical BC definition — its 7-field schema is authoritative.
Reconcile interface-definitions.md and CAP-001-daemon-lifecycle.md to match the BC-2.01.007
7-field canonical schema. Coordinated PO + BA dispatch required (PO owns BC files; BA owns
supplement and L2/CAP files).

---

### F-R105-2 — HIGH

**Severity:** HIGH
**Class:** Stale VP citations + phantom VP IDs in NFR catalog
**Routing:** product-owner + formal-verifier (coordinated)
**Task:** T-128c

**Description:** PRD NFR catalog cites 11 stale VP IDs (renamed during D-122 renumbering) plus 4
PHANTOM VP IDs that do not exist in VP-INDEX.

**Evidence:**
- 11 stale renamed VP IDs throughout NFR catalog (old `VP-DOMAIN-NNN` form persists in NFR rows
  where `VP-NNN` canonical now applies)
- 4 PHANTOM VPs cited in NFR rows: VP-TUI-001, VP-BUILD-001, VP-BUILD-002, VP-DTU-001
  — none of these IDs appear in VP-INDEX.md v1.1

**Impact:** NFR traceability chain broken. VPs cited in NFR rows do not resolve. Holdout evaluation
and formal hardening cannot verify NFR coverage.

**Fix scope:** PO sweeps NFR catalog; FV must either (a) create the 4 phantom VPs or (b) remove
the phantom references from the NFR rows with documented deferral reasoning. Phantom VPs suggest
Phase 4/6 work items that were pre-cited but never authored.

---

### F-R105-3 — HIGH

**Severity:** HIGH
**Class:** L2 domain invariants orphaned from BC traceability
**Routing:** product-owner
**Task:** T-128b

**Description:** All 22 BC files state "L2 Domain Invariants | N/A — no domain-spec/invariants.md
exists" in their Traceability section. However, L2-INDEX.md (authored in D-122 D6 dispatch) defines
DI-001 through DI-007 — 7 domain invariants with full definitions. All 7 DIs are orphaned; no BC
file anchors to any of them.

**Evidence:**
- L2-INDEX.md v1.0.2 §Domain Invariants: DI-001 through DI-007 defined
- 22 BC files Traceability table: L2 Domain Invariants row says "N/A — no domain-spec/invariants.md
  exists" (stale text from before D6 dispatch)

**Impact:** DI-001..DI-007 provide the authoritative domain invariants from which the behavioral
contracts derive. Orphaned DIs means the BC traceability chain to the domain model is broken.
Phase 4 holdout evaluation cannot verify domain-invariant coverage.

**Fix scope:** PO sweeps all 22 BC files. For each BC, identify which DI(s) it implements and
update the Traceability L2 Domain Invariants cell to cite the applicable DI-NNN IDs. This is a
substantial sweep — PO dispatch with explicit per-BC DI mapping guidance.

---

### F-R105-4 — HIGH

**Severity:** HIGH
**Class:** Manifest version bump without §Trace documentation
**Routing:** architect
**Task:** T-128d

**Description:** SS-deps-pin-manifest.md frontmatter shows `version: "1.1.17"` but the §Trace
section contains ZERO entries documenting what changed in v1.1.17. The most recent §Trace entry
documents v1.1.16. An undocumented edit bumped the version without authoring the §Trace entry.

**Evidence:**
- `SS-deps-pin-manifest.md` frontmatter: `version: "1.1.17"`
- `SS-deps-pin-manifest.md` §Trace section: last documented entry is v1.1.16; no v1.1.17 entry

**Impact:** Manifest §Trace is the audit trail for all dependency pin changes. A missing §Trace
entry for v1.1.17 means there is no record of what changed — reviewers and future agents cannot
determine the semantic change or verify the bump was intentional.

**Fix scope:** Architect either (a) documents what actually changed in v1.1.17 by inspecting git
diff, or (b) if the bump was accidental/empty, rolls back to v1.1.16 and removes the spurious
version bump. SE-17f evidence requirement applies.

---

### F-R105-5 — HIGH

**Severity:** HIGH
**Class:** ARCH-INDEX §Trace cites wrong input-hash
**Routing:** architect
**Task:** T-128e

**Description:** ARCH-INDEX.md §Trace v1.0.1 (lines 103-105) claims `input-hash updated to
561ef4d` but the actual frontmatter line 10 shows `ee1f76a`. The §Trace documents a hash that
does not match the artifact's own frontmatter.

**Evidence:**
- ARCH-INDEX.md §Trace v1.0.1 body: `input-hash updated to 561ef4d`
- ARCH-INDEX.md frontmatter line 10: `input-hash: ee1f76a`

**Impact:** Input-hash drift detection is broken for ARCH-INDEX. The drift detection system relies
on §Trace documenting the correct hash that was set; a wrong §Trace hash means the audit trail
is inaccurate.

**Fix scope:** Architect reconciles: either (a) the §Trace cite was wrong at authoring time and
should be updated to `ee1f76a`, or (b) the frontmatter was subsequently changed and the §Trace
correctly documents an older state (in which case the frontmatter is the defect). SE-17c-d
evidence requirement applies to the reconciliation §Trace entry.

---

### F-R105-6 — MEDIUM

**Severity:** MEDIUM
**Class:** Wrong authorization header name in CAP-001
**Routing:** business-analyst
**Task:** T-128f

**Description:** CAP-001-daemon-lifecycle.md lines 128-129 states hook scripts should send
`X-Claude-Code-Ide-Authorization` header. The canonical authorization header per
SS-daemon-lifecycle.md §330+ is `X-Monocle-Authorization`.

**Evidence:**
- CAP-001-daemon-lifecycle.md lines 128-129: `X-Claude-Code-Ide-Authorization`
- SS-daemon-lifecycle.md §330+: `X-Monocle-Authorization` (canonical product name)

**Impact:** If CAP-001 is used as implementation reference, authentication will fail because the
hook client will send the wrong header name. The monocle daemon will reject all hook payloads.

**Fix scope:** BA fixes CAP-001-daemon-lifecycle.md at the cited lines. BA should also sweep
CAP-002 and CAP-003 for the same conflation (same defect class may appear in sibling CAP files).

---

### F-R105-7 — MEDIUM

**Severity:** MEDIUM
**Class:** Stale manifest version pin across 17+ artifacts
**Routing:** product-owner + formal-verifier (parallel after T-128d stabilizes manifest)
**Task:** T-128g

**Description:** PRD v1.26.1 and 17 VP files cite the manifest as v1.1.15 when the canonical
current version is v1.1.17 (per frontmatter). The pin propagation stalled during D-122 restructure
— the restructure dispatches did not update the manifest version citation in the sharded artifacts.

**Evidence:**
- PRD v1.26.1: cites manifest v1.1.15
- 17 VP files (sample confirmed): cite manifest v1.1.15
- SS-deps-pin-manifest.md frontmatter: `version: "1.1.17"`

**Impact:** Stale manifest pin citations mean the PRD and VPs do not correctly describe which
dependency set they were authored against. This introduces traceability ambiguity — were the VPs
written against v1.1.15 semantics or v1.1.17 semantics?

**Fix scope:** BLOCKED on T-128d (architect must first document or roll back v1.1.17 §Trace). Once
the canonical manifest version is stable, PO sweeps PRD and FV sweeps 22 VP files for manifest
version citation. SE-15e serial cascade applies (PO before FV).

---

### F-R105-8 — MEDIUM

**Severity:** MEDIUM
**Class:** Stale BC IDs in architecture documents (165+ occurrences)
**Routing:** architect
**Task:** T-128h

**Description:** Architecture SS-* files retain 165+ stale BC identifiers in the old
`BC-DOMAIN-NNN` format (e.g., `BC-DAEMON-001`, `BC-ENGINE-002`, `BC-RING-001`) instead of
the canonical `BC-2.SS.NNN` format established by D-122 restructure.

**Evidence breakdown:**
- SS-daemon-lifecycle.md: 95 occurrences of stale BC ID format
- SS-engine-module.md: 31 occurrences
- SS-core-types-and-abi.md: 39 occurrences
- Total: 165+ stale BC ID references across 3 architecture documents

**Impact:** Architecture documents are the implementation reference for Phase 3 TDD. Stale BC IDs
in the architecture mean implementers cannot resolve BC citations to the actual sharded BC files
under `behavioral-contracts/ss-NN/`. The cross-reference chain from architecture → BC → VP is
broken.

**Fix scope:** Architect sweeps 3 SS-* files and replaces all stale `BC-DOMAIN-NNN` forms with
canonical `BC-2.SS.NNN` per BC-INDEX.md. This is a substantial mechanical sweep requiring
attention to correct subsystem prefix (ss-01, ss-02, ss-03 per BC-INDEX subsystem table).

---

### F-R105-9 — MEDIUM

**Severity:** MEDIUM
**Class:** BC body prose cites retired VP/BC IDs inline
**Routing:** product-owner
**Task:** T-128i

**Description:** BC body prose (in §Postconditions, §Invariants, §Edge Cases sections) cites
retired pre-D-122 VP and BC identifiers. These inline citations are different from the
Traceability table — they appear in normative body text and cannot be resolved to current artifacts.

**Evidence (specific):**
- BC-2.01.005 line 63: `VP-DAEMON-005` (retired; current canonical: `VP-005`)
- BC-2.01.005 line 70: `BC-ENGINE-002-ERR` (retired; current canonical: `BC-2.03.003`)
- BC-2.01.006 line 59: `VP-DAEMON-006` (retired; current canonical: `VP-006`)
- BC-2.03.003 lines 86+100: stale VP/BC ID references
- BC-2.01.003 line 96: stale BC-RING-001 inline cite
- BC-2.01.007 line 99: `BC-RING-001 EC-002` (stale; should cite canonical BC-2.01.001 form)

**Impact:** Normative body prose with unresolvable ID citations creates ambiguity for implementers
and holdout evaluators. A reader following a prose citation cannot navigate to the referenced
artifact.

**Fix scope:** PO sweeps all 22 BC files for inline VP/BC ID citations that retain the old
`VP-DOMAIN-NNN` or `BC-DOMAIN-NNN` format. Update to canonical `VP-NNN` and `BC-2.SS.NNN` per
BC-INDEX and VP-INDEX. Note: Old ID column in Traceability table may legitimately preserve pre-D122
IDs as historical record (PG-5 pattern); body prose should not.

---

### F-R105-10 — MEDIUM

**Severity:** MEDIUM
**Class:** VP-014 title mismatch with BC-INDEX canonical
**Routing:** formal-verifier
**Task:** T-128j

**Description:** vp-014 line 57 cites the BC title as "FactoryAdapter Trait Surface (Open, No
Sealed Bound)" but BC-INDEX.md canonical title is "FactoryAdapter Trait Definition (FC-04
CRITICAL)". Title mismatch breaks the VP → BC traceability chain.

**Evidence:**
- vp-014 line 57: `"FactoryAdapter Trait Surface (Open, No Sealed Bound)"`
- BC-INDEX.md: `"FactoryAdapter Trait Definition (FC-04 CRITICAL)"` (canonical)

**Impact:** VP-to-BC title cross-reference is normative — reviewers use it to locate the cited BC.
Mismatched title causes navigation confusion and may indicate the VP was authored against a draft
BC that was subsequently retitled.

**Fix scope:** FV updates vp-014 line 57 to cite the canonical BC-INDEX title.

---

### F-R105-11 — MEDIUM

**Severity:** MEDIUM
**Class:** VP-007 stale sister-VP cross-reference
**Routing:** formal-verifier
**Task:** T-128j

**Description:** vp-007 line 87 cross-references `VP-TYPES-001` in the sister-VP citation. The
canonical current ID for this VP is `VP-013` per VP-INDEX.md v1.1.

**Evidence:**
- vp-007 line 87: `VP-TYPES-001` (retired pre-D-122 ID)
- VP-INDEX.md v1.1: canonical ID is `VP-013`

**Impact:** Unresolvable sister-VP cross-reference breaks the bidirectional VP coherence chain
(SE-15d requirement). A reviewer following this citation cannot locate VP-013.

**Fix scope:** FV updates vp-007 line 87 to `VP-013`. FV should sweep all 22 VP files for other
retained `VP-DOMAIN-NNN` inline citations in body prose (same defect class as F-R105-9 on the
VP side).

---

### F-R105-12 — LOW

**Severity:** LOW
**Class:** PRD §7 NFR-012 stale VP citation
**Routing:** product-owner
**Task:** T-128k

**Description:** PRD §7 line 283 NFR-012 row cites `VP-DAEMON-005` (retired pre-D-122 ID) in
the VP column. The canonical current ID is `VP-005` per VP-INDEX.md.

**Evidence:**
- PRD §7 line 283 NFR-012 row VP column: `VP-DAEMON-005`
- VP-INDEX.md: canonical ID `VP-005`

**Fix scope:** PO updates PRD §7 NFR-012 row VP citation to `VP-005`.

---

### F-R105-13 — LOW

**Severity:** LOW
**Class:** VP §References sections cite stale PRD version and non-existent §BC section
**Routing:** formal-verifier
**Task:** T-128k

**Description:** vp-001 line 226 (and 21 other VP §References sections similarly) cites "PRD
v1.26 §BC-2.01.001". Two defects: (a) PRD is now v1.26.1, not v1.26; (b) `§BC-2.01.001` section
no longer exists in the restructured PRD (PRD §3 was deleted in D4 restructure; BCs now live in
sharded BC files, not in PRD §3).

**Evidence:**
- vp-001 line 226: `"PRD v1.26 §BC-2.01.001"`
- PRD v1.26.1: §3 does not exist; BCs are at `.factory/specs/behavioral-contracts/ss-01/BC-2.01.001.md`

**Impact:** VP §References sections guide reviewers to authoritative sources. Stale PRD version +
non-existent §BC section means the reference is unreachable.

**Fix scope:** FV sweeps all 22 VP §References sections. Update PRD version to v1.26.1. Update
§BC-2.SS.NNN references to point at the canonical file path `behavioral-contracts/ss-NN/BC-2.SS.NNN.md`
rather than a PRD §3 subsection that no longer exists.

---

### F-R105-14 — LOW

**Severity:** LOW
**Class:** L2-INDEX §Trace cites non-existent brief anchor
**Routing:** business-analyst
**Task:** T-128k

**Description:** L2-INDEX.md line 150 §Trace cites `brief §Tier 1` as a source reference. The
anchor `§Tier 1` does not exist in product-brief.md. The brief's structure does not include a
`§Tier 1` section.

**Evidence:**
- L2-INDEX.md line 150 §Trace: `brief §Tier 1`
- product-brief.md: no `§Tier 1` section exists (brief uses different section structure)

**Fix scope:** BA updates L2-INDEX §Trace to cite the actual brief section that corresponds to
Tier 1 priority scope (e.g., `brief §Phase 1 Scope` or similar canonical section that exists in
the brief).

---

## Observations (Process-Gap Candidates)

### O-R105-1 — Sibling-Propagation Gate Missing

**Class:** Process-gap
**Status:** OPEN — codification candidate
**Task:** T-128m DEFERRED per Goodhart's law D-114

**Description:** The `validate-template-compliance` perimeter-checking gate (D-123) correctly
verifies that artifacts conform to VSDD template structure. However, it does NOT verify that
content was propagated to sibling and consumer artifacts when the canonical artifact was updated.

The D-122 restructure correctly extracted BC content into sharded BC files and VP content into
sharded VP files (structure PASS). But it did not back-propagate:
- Updated BC IDs to architecture SS-* documents (F-R105-8)
- Updated BC titles to VP body prose (F-R105-10, F-R105-11)
- Updated VP IDs to NFR catalog (F-R105-2)
- L2 domain invariants to BC Traceability cells (F-R105-3)
- Schema consensus across supplement and L2 files (F-R105-1)

A sibling-propagation gate (SE-17h-alt or new gate) would check that when canonical artifacts
are updated, consumer artifacts are updated in the same burst. The SE-17g/SE-17a perimeter checks
structure not propagation; the validate-template-compliance gate checks structure not semantic
propagation.

**Disposition:** DEFERRED per Goodhart's law D-114 (codifying a gate before empirically testing
whether the closure chain T-128 resolves the propagation gap pattern). If R106 + cons R45 still
produce propagation-class findings after Option A closure, SE-17h-alt should be codified.

---

### O-R105-2 — SE-17f Should Extend to All §Trace Bumps

**Class:** Process-gap
**Status:** OPEN — codification candidate
**Task:** T-128m DEFERRED

**Description:** SE-17f (§Trace Evidence-Block Self-Revalidation Gate; 31st discipline) currently
applies to §Trace entries that contain literal grep transcripts or L-number citations. However,
F-R105-4 and F-R105-5 demonstrate that §Trace BUMPS (version entries) can themselves be
incorrect or absent. SE-17f should be extended to cover ALL §Trace bumps: every time a version
is bumped in frontmatter, a corresponding §Trace entry must exist and must accurately describe
what changed.

**Disposition:** DEFERRED per Goodhart's law D-114. Empirically test whether F-R105-4/5 closure
in T-128d/e is sufficient, or whether the class recurs in R106.

---

## Counter Decision

**Counter holds at 0/3.**

D-047 strict policy: any finding of any severity requires counter reset. R105 returned 14 findings
(1 CRIT + 4 HIGH + 6 MED + 3 LOW). Counter advance requires CLEAN R106 + cons R45 (pass 1
attempt 2) after Option A closure chain T-128 completes.

---

## Restructure Validity Verdict

**Structurally SOUND, substantively INCOMPLETE.**

The D-122 restructure (7-dispatch chain, commits 75501ba through 51e77cb) correctly:
- Sharded 22 BCs from PRD §3 inline → `behavioral-contracts/ss-NN/` per-file
- Sharded 22 VPs from monolith → `verification-properties/vp-NNN-*.md` per-file
- Reduced PRD from 4480 → 282 lines (index model)
- Created BC-INDEX, VP-INDEX, ARCH-INDEX, L2-INDEX
- Authored L2 domain spec with 7 DI invariants + 3 CAP shards

The restructure did NOT:
- Update architecture SS-* document BC ID references (165+ stale)
- Update BC body prose VP/BC cross-references (F-R105-9)
- Update NFR catalog VP citations (F-R105-2)
- Propagate BC schema consensus to supplements (F-R105-1)
- Propagate BC Traceability L2 anchors (F-R105-3)
- Document manifest v1.1.17 bump (F-R105-4)
- Reconcile ARCH-INDEX §Trace hash (F-R105-5)

---

## Content Fidelity Audit

**BC extraction from PRD monolith:** CLEAN — substantive BC content was faithfully preserved.
All 22 BC files contain the correct postconditions, invariants, edge cases, and error codes from
the monolithic PRD v1.25.

**VP extraction from VP monolith:** CLEAN — substantive VP content was faithfully preserved.
All 22 VP files contain the correct probes, counter-examples, and mechanism descriptions from
VP v1.35.

**Supplement authoring (interface-definitions, error-taxonomy, nfr-catalog, test-vectors):** FAIL
at F-R105-1 (HookEventRecord schema divergence) and F-R105-2 (stale VP citations in NFR catalog).

**L2 domain spec (L2-INDEX + CAP-001/002/003):** FAIL at F-R105-6 (wrong auth header in CAP-001)
and F-R105-14 (stale brief anchor in L2-INDEX §Trace).

**Architecture propagation (SS-* → BC IDs):** FAIL at F-R105-8 (165+ stale BC IDs).

---

## Cross-Artifact Integrity Assessment

| Pair | Status | Finding |
|------|--------|---------|
| BC ↔ VP bidirectional | CLEAN | No unresolvable BC-VP pairs in traceability tables |
| BC ↔ L2 DI | FAIL | F-R105-3: all 22 BCs claim L2 invariants don't exist; DI-001..007 orphaned |
| NFR ↔ VP-INDEX | FAIL | F-R105-2: 11 stale + 4 phantom VP IDs in NFR catalog |
| manifest ↔ consumers | FAIL | F-R105-7: 17+ artifacts cite stale v1.1.15 |
| arch ↔ BC-INDEX | FAIL | F-R105-8: 165+ stale BC IDs in 3 SS-* files |
| HookEventRecord ↔ canonical | FAIL | F-R105-1: 3-way schema divergence (BC / supplement / L2) |
| VP body ↔ VP-INDEX | PARTIAL-FAIL | F-R105-10/11: VP title mismatch + stale sister-VP ref |
| PRD §7 ↔ VP-INDEX | FAIL | F-R105-12: NFR-012 stale VP cite |
| VP §References ↔ PRD | FAIL | F-R105-13: stale PRD version + non-existent §BC section |
| L2-INDEX §Trace ↔ brief | FAIL | F-R105-14: §Tier 1 anchor doesn't exist |

---

## Recommended Routing

| Finding | Severity | Primary Routing | Secondary |
|---------|----------|-----------------|-----------|
| F-R105-1 | CRITICAL | product-owner | business-analyst (coordinated) |
| F-R105-2 | HIGH | product-owner | formal-verifier (phantom VP creation/removal) |
| F-R105-3 | HIGH | product-owner | — |
| F-R105-4 | HIGH | architect | — |
| F-R105-5 | HIGH | architect | — |
| F-R105-6 | MEDIUM | business-analyst | — |
| F-R105-7 | MEDIUM | product-owner | formal-verifier (after T-128d) |
| F-R105-8 | MEDIUM | architect | — |
| F-R105-9 | MEDIUM | product-owner | — |
| F-R105-10 | MEDIUM | formal-verifier | — |
| F-R105-11 | MEDIUM | formal-verifier | — |
| F-R105-12 | LOW | product-owner | — |
| F-R105-13 | LOW | formal-verifier | — |
| F-R105-14 | LOW | business-analyst | — |
| O-R105-1 | process-gap | state-manager | (DEFERRED) |
| O-R105-2 | process-gap | state-manager | (DEFERRED) |

---

## Final Note

**User selected Option A (full closure chain)** for the new session after context-clear. The closure
chain is pre-planned in STATE.md v5.63 Task Queue entries T-128a through T-128k (11 specialist
dispatches + T-128l SM closure). The re-audit cycle follows: validate-template-compliance R4 +
adversary R106 + cons R45 (D-047 strict pass 1 attempt 2).

Cons R44 findings (5 total; 3 overlap R105 + 2 new) are persisted at
`.factory/plans/consistency-r44-phase1.md`. The 2 new GAP-R44 findings:
- GAP-R44-3 MED: interface-definitions.md lock file uses `auth_token` snake_case + omits
  `startTimeUtc, app, version` fields; canonical is `authToken` camelCase per SS-daemon-lifecycle
  + BC-2.01.005/010. Routing: T-128j (PO scope).
- GAP-R44-4 LOW: PRD §5 line 159 says "6 subsystem abbreviations" but inline list shows 7
  (DAEMON, AUTH, LOCK, RING, FACT, ENG, PROTO). Routing: T-128k (PO scope).
