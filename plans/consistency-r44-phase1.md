---
document_type: consistency-pass
pass_id: R44
attempt: 1
counter: "0/3 (NEW D-047 cycle against restructured artifacts)"
verdict: GAPS
artifact_pins:
  - artifact: "PRD"
    version: "v1.26.1"
    commit: "1a09095"
    lines: 282
  - artifact: "prd-supplements (4)"
    version: "v1.0"
    commit: "1a09095"
    note: "interface-definitions, error-taxonomy, test-vectors, nfr-catalog"
  - artifact: "BC files (22)"
    version: "SS-01 v1.0.1 (1a09095); SS-02/SS-03 v1.0.1 (1a09095)"
    count: "10+8+4 = 22"
  - artifact: "BC-INDEX.md"
    version: "v1.1"
    commit: "f259ade"
  - artifact: "VP files (22)"
    version: "v1.0.1"
    commit: "4090d0b"
  - artifact: "VP-INDEX.md"
    version: "v1.1"
    commit: "e3824ec"
  - artifact: "ARCH-INDEX.md"
    version: "v1.0.1"
    commit: "0af206a"
  - artifact: "arch SS-* (7)"
    version: "per ARCH-INDEX Document Map"
  - artifact: "L2-INDEX.md"
    version: "v1.0.2"
    commit: "2a852d1"
  - artifact: "CAP-001/002/003 (3 L2 shards)"
    version: "v1.0"
    commit: "2a852d1"
  - artifact: "SS-deps-pin-manifest.md"
    version: "v1.1.17"
    commit: "75501ba"
  - artifact: "STATE.md"
    version: "v5.62"
    commit: "544bdfb"
dimensions_applied: 10
timestamp: 2026-05-16T00:00:00Z
---

# Consistency Pass R44 — Phase 1 Restructured Artifact Set

> **D-047 strict pass 1 attempt 1** against restructured template-compliant
> artifact set. New counter reset: 0/3. Paired with adversary R105.

## §Summary

| Dimension | Status | Notes |
|-----------|--------|-------|
| 1. Cross-artifact pin coherence | PASS | 28 dep pins coherent; no pin conflicts |
| 2. PRD §2 ↔ BC-INDEX ↔ BC files | PASS | 22 BCs consistent across all three layers |
| 3. VP source_bc ↔ BC files | PASS | All 22 VP frontmatter source_bc fields use new canonical IDs |
| 4. BC ## VP Anchors ↔ VP files | PASS | BC VP Anchor sections correctly reference new VP file paths |
| 5. §Trace / §References lineage | PARTIAL GAP | L2-INDEX frontmatter v1.0.2 but §Trace body only documents v1.0; see GAP-R44-5 |
| 6. manifest ↔ arch ↔ VP triple pin | PASS | Coherent; OBS-R41-1 unchanged (see §Open Observations) |
| 7. PRD §10 Glossary completeness | PASS | All normative terms have glossary entries |
| 8. EC anchoring (61 ECs in BC files) | PASS | ECs distributed correctly across 22 BC files |
| 9. NFR-to-VP coverage | GAP | VP Probe Citations in nfr-catalog.md uses stale old VP IDs; 4 cited VPs are phantom (not in VP-INDEX); see GAP-R44-1 |
| 10. CLAUDE.md cites + cross-doc consistency | PARTIAL GAP | Content-fidelity defects from D-122 extraction; see GAP-R44-2, GAP-R44-3, GAP-R44-4 |

**Verdict: GAPS — MED+ findings present; counter holds at 0/3.**

---

## §Findings

### GAP-R44-1 — NFR Catalog VP Probe Citations Use Stale Pre-Renumbering IDs (MED)

**Dimension:** 9 (NFR-to-VP coverage)
**Severity:** MED
**Artifact:** `/Users/jmagady/Dev/monocle/.factory/specs/prd-supplements/nfr-catalog.md` §VP Probe Citations (lines 72-83)
**Also affected:** `/Users/jmagady/Dev/monocle/.factory/specs/prd.md` §7 RTM NFR-012 row (line 283)

**Description:**

The nfr-catalog.md §VP Probe Citations table and the NFR Registry Validation Method column both use pre-renumbering VP IDs (VP-DAEMON-NNN, VP-AUTH-NNN) that are now historical. Additionally, four VP IDs cited in the table reference VPs that do not exist in VP-INDEX.md at all.

**Stale ID mapping (resolvable via VP-INDEX Renumbering Appendix):**

| NFR | Cited (stale) | Correct | Status |
|-----|--------------|---------|--------|
| NFR-001 | VP-DAEMON-001 | VP-001 | VP exists |
| NFR-002 | VP-DAEMON-002 | VP-002 | VP exists |
| NFR-004 | VP-AUTH-001 | VP-008 | VP exists |
| NFR-005 | VP-DAEMON-003 | VP-003 | VP exists |
| NFR-006 | VP-DAEMON-006 | VP-006 | VP exists |
| NFR-009 | VP-DAEMON-005 | VP-005 | VP exists |
| NFR-010 | VP-AUTH-001 | VP-008 | VP exists |
| NFR-012 | VP-DAEMON-005 | VP-005 | VP exists |

**Phantom VPs (no entry in VP-INDEX — not Phase 1 VPs):**

| NFR | Cited | Status |
|-----|-------|--------|
| NFR-003 | VP-TUI-001 | NOT IN VP-INDEX — no TUI VP in Phase 1 |
| NFR-007 | VP-BUILD-001 | NOT IN VP-INDEX — no Build VP in Phase 1 |
| NFR-008 | VP-BUILD-002 | NOT IN VP-INDEX — no Build VP in Phase 1 |
| NFR-011 | VP-DTU-001 | NOT IN VP-INDEX — no DTU VP in Phase 1 |

**Impact:** NFR-003, NFR-007, NFR-008, NFR-011 have no valid VP probe citation. The Phase 1 VP registry has no TUI, Build, or DTU VPs. These NFRs are verified by non-VP mechanisms (CI matrix, integration tests with latency assertions, DTU fidelity measurement), but the VP Probe Citations table claims VP IDs that do not exist.

**Additional:** PRD §7 RTM NFR-012 row Test Type column contains `VP-DAEMON-005 Post-condition 9 / probe 5.e` — a stale old VP ID; should be `VP-005 Post-condition 9 / probe 5.e`.

**Inline NFR Registry Validation Method column** also contains stale old VP IDs in normative text (lines 27-28, 32-33, 35 of nfr-catalog.md) for NFR-004, NFR-005, NFR-009, NFR-010, NFR-012.

**Remediation:**
- Product-owner: Update nfr-catalog.md §VP Probe Citations table — replace all stale VP-NNN-OLD IDs with VP-NNN IDs per VP-INDEX Renumbering Appendix.
- Product-owner: Update nfr-catalog.md NFR Registry Validation Method column inline VP references (lines 27-28, 32-33, 35) to use VP-NNN canonical form.
- Product-owner: For NFR-003, NFR-007, NFR-008, NFR-011: replace phantom VP citations with the actual verification mechanism (CI matrix check, integration test latency probe, DTU fidelity measurement). Document these as non-VP-backed validation or add a footnote explaining they are verified without a Phase 1 VP.
- Product-owner: Update PRD §7 RTM NFR-012 row Test Type column from `VP-DAEMON-005 ...` to `VP-005 ...`.

---

### GAP-R44-2 — CAP-001 Hook Ingestion Auth Header Name Is Wrong (MED)

**Dimension:** 10 (CLAUDE.md cites + cross-doc consistency)
**Severity:** MED
**Artifact:** `/Users/jmagady/Dev/monocle/.factory/specs/domain-spec/CAP-001-daemon-lifecycle.md` line 129

**Description:**

CAP-001 §Domain Processes §P2: Hook Event Ingestion, step 1 reads:

> "A harness subprocess fires an HTTP POST to `POST /hooks/<type>` with the `X-Claude-Code-Ide-Authorization` header set to the token read from the lock file."

The correct monocle auth header is `X-Monocle-Authorization` (used in PRD, all 22 BC files, all 22 VP files, interface-definitions.md, SS-daemon-lifecycle.md, and every other normative document). `X-Claude-Code-Ide-Authorization` is Claude Code's internal IDE header, mentioned in the architecture as an optional per-handler check inside hook handlers — it is NOT the header hook scripts use to authenticate to monocle's daemon.

**Evidence of conflict:**
- `interface-definitions.md` §Authentication Header Format: `X-Monocle-Authorization: monocle-v1:<64-hex-lowercase>`
- BC-2.01.008 postcondition 2: `X-Monocle-Authorization: monocle-v1:<64-char-hex>`
- BC-2.01.009: validates `X-Monocle-Authorization` header
- SS-daemon-lifecycle.md line 146: "share the same `X-Monocle-Authorization` middleware layer"
- SS-daemon-lifecycle.md line 147: the `X-Claude-Code-Ide-Authorization` is checked per-handler inside hook handlers (it is Claude Code's own header for IDE integration, checked optionally)

**Impact:** A developer reading only CAP-001 would write hook scripts using `X-Claude-Code-Ide-Authorization` and get HTTP 401 on every request. This is a content-fidelity defect introduced in D-122 L2 shard extraction.

**Remediation:**
- Business-analyst: Replace `X-Claude-Code-Ide-Authorization` with `X-Monocle-Authorization: monocle-v1:<token>` in CAP-001 §P2 step 1. Optionally add a note that Claude Code's IDE uses a separate `X-Claude-Code-Ide-Authorization` header for its own IPC, which monocle checks per-handler inside hook handlers.

---

### GAP-R44-3 — interface-definitions.md Lock File Uses `auth_token` (snake_case) vs `authToken` (camelCase) Everywhere Else (MED)

**Dimension:** 10 (cross-doc consistency)
**Severity:** MED
**Artifact:** `/Users/jmagady/Dev/monocle/.factory/specs/prd-supplements/interface-definitions.md` §Lock File Schema (lines 196-210)

**Description:**

The interface-definitions.md lock file JSON schema uses `auth_token` (snake_case):

```json
{
  "contract_version": 1,
  "pid": <integer>,
  "port": <integer>,
  "auth_token": "<64-hex-lowercase>",
  ...
}
```

All other normative documents use `authToken` (camelCase):

| Artifact | Field name |
|----------|-----------|
| SS-daemon-lifecycle.md (lines 391, 396, 403, 406-407) | `authToken` |
| BC-2.01.010 postcondition 1 (line 45) | `authToken` |
| BC-2.01.010 test vector (line 70) | `authToken` |
| BC-2.01.005 postcondition 4 (line 55) | `authToken` |
| VP-008 (auth token VP) body | `authToken` |
| CAP-001 DaemonLockFile entity | `token` (neither — and also a minor inconsistency; uses short name `token` without `auth_` prefix) |

**Impact:** An implementer reading interface-definitions.md will write `serde` with `auth_token` as the field name, but all BC tests expect `authToken`. The lock file contract will mismatch test assertions. This is a content-fidelity defect introduced when interface-definitions.md was extracted during D-122.

**Remediation:**
- Product-owner: Update interface-definitions.md §Lock File Schema to use `authToken` (matching SS-daemon-lifecycle.md and BCs). Also add the `startTimeUtc`, `app`, `version` fields that BC-2.01.010 postcondition 1 requires but interface-definitions.md omits.
- Business-analyst: Update CAP-001 DaemonLockFile entity `token` field to `authToken` for alignment.

---

### GAP-R44-4 — PRD §5 Error Count Says "6 Subsystem Abbreviations" But 7 Exist (LOW-MED)

**Dimension:** 10 (cross-doc consistency / supplement internal consistency)
**Severity:** LOW-MED
**Artifact:** `/Users/jmagady/Dev/monocle/.factory/specs/prd.md` line 159

**Description:**

PRD §5 reads:

> "Phase 1 defines 14 error codes across **6** subsystem abbreviations (`DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`)."

The parenthetical list contains 7 abbreviations: DAEMON, AUTH, LOCK, RING, FACT, ENG, PROTO. The count "6" is wrong; it should be "7". The error-taxonomy.md confirms exactly 7 distinct subsystem abbreviations appear in the error catalog.

**Impact:** Minor count mismatch. An implementer validating "6 subsystem abbreviations" against the catalog will see a discrepancy.

**Remediation:**
- Product-owner: Update PRD §5 line 159 to read "14 error codes across **7** subsystem abbreviations".

---

### GAP-R44-5 — L2-INDEX Frontmatter Version 1.0.2 But §Trace Only Documents v1.0 (LOW)

**Dimension:** 5 (§Trace lineage)
**Severity:** LOW
**Artifact:** `/Users/jmagady/Dev/monocle/.factory/specs/domain-spec/L2-INDEX.md`

**Description:**

The L2-INDEX.md frontmatter declares `version: "1.0.2"`, but the §Trace section contains only a single `§Trace v1.0` entry documenting the creation at Dispatch 6. There are no `§Trace v1.0.1` or `§Trace v1.0.2` entries. Versions 1.0.1 and 1.0.2 were applied to the frontmatter (likely as part of audit R2 residual fix passes that swept all artifacts) without corresponding §Trace body documentation of what changed.

**Impact:** Audit trail gap. An implementer cannot determine what changed between v1.0 and v1.0.2.

**Remediation:**
- Business-analyst or state-manager: Add `§Trace v1.0.1` and `§Trace v1.0.2` entries documenting the changes applied in each bump. If the bumps were mechanical (input-hash normalization only), document that explicitly.

---

## §PASS Dimensions

The following dimensions were checked and found consistent:

**Dimension 1 — Cross-artifact pin coherence:** All 28 dependency pins in SS-deps-pin-manifest.md v1.1.17 are coherent across architecture documents and VP files. No version conflicts detected. Phase 2/3/4 pins correctly annotated as non-Phase-1.

**Dimension 2 — PRD §2 ↔ BC-INDEX ↔ BC files:** All 22 BC IDs are consistent across PRD §2.1/2.2/2.3 tables, BC-INDEX.md registry (three subsystem sections), and individual BC files. Titles match exactly. Priorities match exactly. File paths in BC-INDEX resolve to existing files.

**Dimension 3 — VP source_bc ↔ BC files:** All 22 VP frontmatter `source_bc:` fields use new canonical BC-S.SS.NNN IDs. Every source_bc value resolves to an existing BC file. No fabricated anchors.

**Dimension 4 — BC ## VP Anchors ↔ VP files:** All sampled BC files' `## VP Anchors (Recommended)` sections correctly reference new VP file paths (`verification-properties/vp-NNN-name.md`). RES-02 fix is confirmed held.

**Dimension 5 (partial):** SE-16d timestamp chain is monotonic: BC-INDEX 12:00 < VP-INDEX 13:30 < L2-INDEX 14:00 < ARCH-INDEX 16:30 = L2-INDEX 16:30 < PRD 17:30 < VP files 18:00. All in UTC Z form. Chain passes SE-16d. GAP-R44-5 is a §Trace body documentation gap, not a timestamp violation.

**Dimension 6 — manifest ↔ arch ↔ VP triple pin:** 28 dep pins coherent. OBS-R41-1 unchanged (see §Open Observations).

**Dimension 7 — PRD §10 Glossary completeness:** All normative terms in BC files, VP files, and architecture are covered by PRD §10 Glossary entries. Glossary entries reference current canonical BC IDs.

**Dimension 8 — EC anchoring:** 22 BC files contain EC-NNN entries distributed by domain. Sampling confirms EC IDs are correctly scoped (EC-040/041 in BC-2.01.001, EC-004/005/006 in BC-2.01.008, EC-018/019/020 in BC-2.02.004, EC-032-035 in BC-2.03.002). No orphan ECs detected.

**Dimension 9 (partial pass):** The 12 NFRs in nfr-catalog.md map to existing Phase 1 VPs where VPs exist. NFR-001 → VP-001, NFR-002 → VP-002, NFR-004 → VP-008, NFR-005 → VP-003, NFR-006 → VP-006, NFR-009 → VP-005, NFR-010 → VP-008, NFR-012 → VP-005. The GAP is in the VP Probe Citations table using old IDs and citing 4 phantom VPs (see GAP-R44-1).

**Dimension 10 (partial pass):** CLAUDE.md cross-references to brief v1.4.23, vision v1.1.2, and arch artifacts are current. BC-INDEX renumbering appendix preserves all 22 old IDs. VP-INDEX renumbering appendix preserves all 22 old PG-5 IDs. Stale old BC IDs in VP Harness Location sections are correctly qualified with "per PRD v1.25 §BC-OLD-ID ... to be migrated" (historical cross-references, not normative identifiers). Stale old VP IDs in BC-2.01.005 and BC-2.01.006 body text are normative in-text references (not qualified as historical) — but they appear alongside new canonical IDs and the old IDs resolve via the renumbering appendix; assessed as LOW within this dimension given the qualification in §Trace. Content-fidelity defects from D-122 extraction are captured as GAP-R44-2, GAP-R44-3, GAP-R44-4.

---

## §Open Observations

### OBS-R41-1 — reqwest 0.13 Has No Consumer Edge in Phase 1 Workspace (UNCHANGED)

**Status:** UNCHANGED from R41/R42/R43 passes.
**Severity:** LOW informational — non-blocking.

`reqwest 0.13` is listed in the Phase 1 Pin Manifest (SS-deps-pin-manifest.md v1.1.17 line 62) under "Phase 1 Pin Manifest" with role "HTTP client". However, the Workspace Dependency Graph (§Workspace Dependency Graph) contains no crate with a `reqwest` dependency edge. No Phase 1 crate (monocle-runtime, monocle-core, monocle-tui, etc.) has an HTTP client role in Phase 1 — the daemon uses axum (server), not reqwest (client).

**Disposition:** Deferred to Phase 1 architecture creation, where the architect will resolve whether reqwest is a Phase 2+ dependency incorrectly placed in the Phase 1 manifest table, or whether it serves a use case not yet assigned. OBS-R41-1 continues non-blocking per STATE.md Blocking Issues table.

### OBS-R44-1 — BC Body Text Contains Stale VP IDs (Normative References, Not Historical) (LOW)

**New observation this pass.**

BC-2.01.005 (line 63) and BC-2.01.006 (line 59) contain stale old VP IDs in normative postcondition/postcondition body text:

- BC-2.01.005 line 63: `Cross-reference: VP-DAEMON-005 Post-condition 9 and probe 5.e`
- BC-2.01.006 line 59: `VP-DAEMON-006 enforces this with regex ...`

Unlike the VP file references (which are explicitly qualified as "per PRD v1.25 §BC-OLD-ID ... to be migrated"), these BC body references are normative cross-references that lack the "historical" qualification. They should reference VP-005 and VP-006 respectively.

**Severity:** LOW — both old IDs resolve unambiguously via VP-INDEX Renumbering Appendix, and no implementer will be misled as to which VP is intended. But the references are formally stale.

**Recommended fix (product-owner):** Update BC-2.01.005 line 63 to `VP-005 Post-condition 9 and probe 5.e` and BC-2.01.006 line 59 to `VP-006`.

---

## §Restructure Consistency Verdict

D-122 template-compliance remediation + D-124 audit R2 residuals + D-126 audit R3 produced a structurally compliant artifact set with:
- All 22 BC files correctly sharded into ss-01/ss-02/ss-03 directories
- All 22 VP files correctly sharded as individual files
- BC-INDEX, VP-INDEX, ARCH-INDEX, L2-INDEX all present with correct frontmatter
- 4 prd-supplements all present with correct traces_to
- BC-INDEX renumbering appendix complete (22 old IDs mapped)
- VP-INDEX renumbering appendix complete (22 old PG-5 IDs mapped)
- All 22 VP frontmatter source_bc fields updated to new canonical IDs
- All normative PRD §2 and §7 BC references updated to new IDs

**Content-fidelity defects introduced during D-122 extraction:**
- GAP-R44-2: CAP-001 wrong auth header name (MED — implementer-misleading)
- GAP-R44-3: interface-definitions.md lock file field `auth_token` vs `authToken` everywhere else (MED — test assertion mismatch)
- GAP-R44-4: PRD §5 "6 subsystem abbreviations" vs 7 actual (LOW-MED — count wrong)

**Pre-existing renumbering propagation gap:**
- GAP-R44-1: nfr-catalog.md VP Probe Citations using stale old VP IDs + 4 phantom VPs (MED)

**Trace bookkeeping gap:**
- GAP-R44-5: L2-INDEX version 1.0.2 but §Trace documents only v1.0 (LOW)

**Conclusion:** D-122 + D-124 + D-126 restructuring is structurally sound and ID-consistent. Content-fidelity defects (GAP-R44-2, GAP-R44-3) are resolvable by product-owner/business-analyst without structural changes. GAP-R44-1 (NFR VP probe citations) was a pre-existing renumbering propagation gap — the nfr-catalog supplement was not updated when VP IDs were renumbered in Dispatch 5a/5b.

**Gate result: FAIL — counter holds at 0/3.** MED+ findings (GAP-R44-1, GAP-R44-2, GAP-R44-3) block advancement per D-047 strict gate policy.
