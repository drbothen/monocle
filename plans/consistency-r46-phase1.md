---
document_type: consistency-report
level: ops
version: "1.0"
producer: vsdd-factory:consistency-validator
timestamp: 2026-05-17T23:30:00Z
phase: phase-1-spec-crystallization
cycle: cycle-001
round: R46
counter: "0/3 (attempt 3 of pass 1)"
traces_to: prd.md
---

# Consistency Report R46 — monocle Phase 1 Spec Package

**Round:** R46 (T-127'' re-audit cycle, pass 1 attempt 3)
**Scope:** Post-R106-closure + post-R45-closure + pre-R107 VP-INDEX fix; 25 findings closed
**Audited artifact set:** PRD v1.26.4, BC-INDEX v1.4, VP-INDEX v1.4, ARCH-INDEX v1.0.5, L2-INDEX v1.0.6, 22 BCs, 22 VPs, 4 PRD supplements, 5 ADRs, dtu-assessment v1.7.3, product-brief v1.4.25, SS-deps-pin-manifest v1.1.17

---

## §Summary

**Verdict: GAPS**
**Total GAPs:** 5
**Severity breakdown:** HIGH: 2 | MED: 2 | LOW: 1
**Blocking (HIGH):** 2

### 10-Dimension Pass Summary

| Dim | Description | Result | Notes |
|-----|-------------|--------|-------|
| D1 | Spec ID references (BC, VP, NFR, DI, ADR, E-XXX) resolve to canonical sources | PASS | All BC/VP/NFR/DI/ADR/E-XXX IDs checked against indexes; no phantom IDs found |
| D2 | Anchor links resolve to actual section names | PASS | No dead anchor links detected in cross-reference prose |
| D3 | Counts cited inline match actual | PASS | 22 BCs, 22 VPs, 7 DIs, 5 ADRs, 15 error codes, 8 test vectors for BC-2.01.009 all correct |
| D4 | Naming consistency (X-Monocle-Authorization canonical; alias only) | PASS-WITH-OBS | All normative text correct; 1 non-normative code-sketch comment is imprecise (Obs-R46-1) |
| D5 | Traceability chains complete: FR/NFR → VP → BC → arch SS → L2 DI/CAP | PASS | All 22 BCs map to VPs; all VPs map to BCs; all BCs map to subsystems and DIs |
| D6 | Frontmatter version pin propagation — cited versions match current | GAP | GAP-R46-1 (HIGH), GAP-R46-2 (HIGH), GAP-R46-3 (MED), GAP-R46-4 (LOW) |
| D7 | UTC ISO-8601 timestamp monotonicity within §Trace sections | PASS | BC-INDEX v1.1→v1.2→v1.3→v1.4 monotonic; VP-INDEX v1.1→v1.2→v1.3→v1.4 monotonic |
| D8 | ADR-0005 cascade integrity (dual-accept propagation) | GAP | GAP-R46-5 (MED): BC-2.01.004 INV-3 cites canonical header only, not dual-accept |
| D9 | /shutdown endpoint specification cross-references | PASS | interface-definitions v1.3 correctly cites BC-2.01.004 + BC-2.01.008 + BC-2.01.009; VP-004 + VP-009 cross-reference |
| D10 | E-AUTH-003 cross-referencing from BC-2.01.009 + VP-009 + nfr-catalog | PASS-WITH-OBS | E-AUTH-003 exists in error-taxonomy citing BC-2.01.009 INV-6; BC-2.01.009 and VP-009 reference the same behavior via INV-6 not the error code; nfr-catalog does not reference E-AUTH-003 (Obs-R46-2; not required by template) |

**Pass count (PASS/PASS-WITH-OBS):** 8/10
**GAP dimensions:** D6 (version pins) and D8 (ADR-0005 cascade)

---

## §Findings

### GAP-R46-1 | HIGH | 22 VP §References sections cite stale PRD v1.26.3

**Description:** All 22 individual VP files (`vp-001-healthz-endpoint.md` through `vp-022-claude-code-module-inherent-methods.md`) have `prd.md v1.26.3` in their `## References` section bodies. The VP-INDEX §Trace v1.4 (the pre-R107 fix burst) correctly refreshed the VP-INDEX's own §References PRD cite to v1.26.4, but the individual VP files were not touched in that burst. Current PRD is v1.26.4 (PO 5B commit df5605a — F-R106-4 PRD §7 mass pin refresh).

**Scope:** All 22 VP files.

**Evidence (representative sample):**

`vp-001-healthz-endpoint.md` line 226:
```
- PRD: `.factory/specs/prd.md` v1.26.3 §BC-2.01.001 (Dispatch 4 commit 1030c65;
  refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
```

`vp-009-auth-header-validation.md` line 434:
```
- PRD: `.factory/specs/prd.md` v1.26.3 §BC-2.01.009 (Dispatch 4 commit 1030c65;
  refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
```

Confirmed by sweep: `grep -rn "prd\.md.*v1\.26\.3" .factory/specs/verification-properties/vp-*.md` returns 22 matches, one per file, each in the `## References` body section.

**Current correct version:** PRD v1.26.4 (frontmatter confirmed: `grep -n "^version:" .factory/specs/prd.md` → `4:version: "1.26.4"`).

**Root cause:** The pre-R107 fix burst (VP-INDEX §Trace v1.4) updated VP-INDEX §References but not the 22 individual VP file §References sections. The VP-INDEX is the canonical entry point for Phase 1 VPs; the per-VP §References PRD cite was separately established in the F-R105-13 sweep (§Trace v1.0.3 in each VP) and was not part of the scope of the VP-INDEX pre-R107 fix.

**Recommended fix:** FV agent — sweep all 22 VP `## References` sections and refresh PRD cite from `v1.26.3 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b)` to `v1.26.4 (Dispatch 4 commit 1030c65; refreshed to v1.26.4 in F-R106-4 closure, PO 5B commit df5605a — PRD §7 mass pin refresh; supersedes v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b)`. SE-17c-d body-scope grep required post-edit. SE-16d monotonicity required.

**Routing:** `vsdd-factory:formal-verifier` (VP file ownership).

---

### GAP-R46-2 | HIGH | 9 BC Traceability §Architecture Source rows cite stale SS-daemon-lifecycle version (v1.0.25 or v1.0.29)

**Description:** The SS-daemon-lifecycle.md current version is v1.0.30 (confirmed: `grep "^version:" .factory/specs/architecture/SS-daemon-lifecycle.md` → `6:version: "1.0.30"`). Eight SS-01 BCs cite v1.0.25 in their Traceability §Architecture Source row; BC-2.01.009 cites v1.0.29. Neither is current.

**Stale citations (all outside §Trace blocks):**

| BC file | Line | Cited version | Current |
|---------|------|--------------|---------|
| `ss-01/BC-2.01.001.md` | 84 | v1.0.25 | v1.0.30 |
| `ss-01/BC-2.01.002.md` | 95 | v1.0.25 | v1.0.30 |
| `ss-01/BC-2.01.003.md` | 86 | v1.0.25 | v1.0.30 |
| `ss-01/BC-2.01.004.md` | 102 | v1.0.25 | v1.0.30 |
| `ss-01/BC-2.01.005.md` | 114 | v1.0.25 | v1.0.30 |
| `ss-01/BC-2.01.006.md` | 103 | v1.0.25 | v1.0.30 |
| `ss-01/BC-2.01.007.md` | 89 | v1.0.25 | v1.0.30 |
| `ss-01/BC-2.01.008.md` | 87 | v1.0.25 | v1.0.30 |
| `ss-01/BC-2.01.009.md` | 106 | v1.0.29 | v1.0.30 |
| `ss-01/BC-2.01.010.md` | 89 | v1.0.25 (also cites `SS-core-types-and-abi.md` with no version) | v1.0.30 |

**Evidence (BC-2.01.008, line 87):**
```
| Architecture Source | SS-daemon-lifecycle.md v1.0.25 §Daemon Lifecycle Protocol §Start Sequence; ADR-0005 (dual-accept auth header decision) |
```

**Evidence (BC-2.01.009, line 106):**
```
| Architecture Source | SS-daemon-lifecycle.md v1.0.29 §Daemon Lifecycle Protocol §Start Sequence; ADR-0005 (dual-accept auth header decision) |
```

**Root cause:** The F-R106 Round 5 dispatch that bumped SS-daemon-lifecycle v1.0.29 → v1.0.30 (Architect 5E) did not cascade a pin refresh to the 10 SS-01 BC Traceability §Architecture Source rows. The VP files were swept (VP-INDEX §Trace v1.3 Change 1 confirms SS-01 architecture pin refreshed from v1.0.25 to v1.0.30 in VP-INDEX), but the per-BC rows were not updated in that dispatch.

**Note on SS-02 and SS-03 BCs:** SS-02 BCs (BC-2.02.001 through BC-2.02.008) cite `SS-core-types-and-abi.md v1.2.8` while current is v1.2.11. SS-03 BCs (BC-2.03.001 through BC-2.03.004) cite `SS-engine-module.md v1.1.15` while current is v1.1.18. These are the same class of finding — stale architecture source pins in BC Traceability rows. Counting them together with the SS-01 findings: **all 22 BCs carry stale architecture source version pins** for their respective subsystem architecture files.

**Recommended fix:** PO agent — sweep all 22 BC Traceability §Architecture Source rows and update to current versions (SS-daemon-lifecycle v1.0.30, SS-core-types-and-abi v1.2.11, SS-engine-module v1.1.18). SE-17c-d body-scope grep + SE-16d monotonicity required per burst.

**Routing:** `vsdd-factory:product-owner` (BC file ownership).

---

### GAP-R46-3 | MED | All 4 PRD supplements have incorrect ADR-0005 filename in `inputs:` frontmatter field

**Description:** All four PRD supplements reference the ADR-0005 file in their `inputs:` frontmatter field using a truncated filename that does not exist on disk.

**Actual filename on disk:**
```
architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md
```

**Frontmatter `inputs:` field in all 4 supplements:**
```
architecture/adr/ADR-0005-dual-accept-auth-header.md
```

**Evidence:**

`prd-supplements/interface-definitions.md` frontmatter line 9:
```yaml
inputs: [prd.md, architecture/adr/ADR-0005-dual-accept-auth-header.md]
```

`prd-supplements/error-taxonomy.md` frontmatter line 9:
```yaml
inputs: [prd.md, architecture/adr/ADR-0005-dual-accept-auth-header.md]
```

`prd-supplements/nfr-catalog.md` frontmatter line 9:
```yaml
inputs: [prd.md, architecture/adr/ADR-0005-dual-accept-auth-header.md]
```

`prd-supplements/test-vectors.md` frontmatter line 9:
```yaml
inputs: [prd.md, behavioral-contracts/, architecture/adr/ADR-0005-dual-accept-auth-header.md]
```

**Impact:** The `compute-input-hash` tool and any agent consuming these `inputs:` fields would attempt to resolve a non-existent file path, causing input-hash computation errors or silent hash divergence. The actual ADR-0005 file was normalized in Architect 5E dispatch (ARCH-INDEX §Trace v1.0.5 confirms path normalization occurred for ADR-0005.md `inputs:` field correction on the ADR file itself, but the supplement frontmatter was not swept).

**Recommended fix:** PO agent — correct the `inputs:` ADR-0005 path in all 4 supplement frontmatter fields from `ADR-0005-dual-accept-auth-header.md` to `ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md`. SE-16d monotonicity required.

**Routing:** `vsdd-factory:product-owner` (supplement file ownership).

---

### GAP-R46-4 | LOW | test-vectors.md §Trace body cites BC-2.01.009 v1.0.2; current version is v1.0.3

**Description:** The test-vectors.md §Trace section (the F-R106-3 closure entry, line 74 context note and line 176 canonical source citation) cites `BC-2.01.009 v1.0.2` as the version that added the 2 alias-path vectors. The current version of BC-2.01.009 is v1.0.3 (F-R106-7 fabricated FC-ID removal). The body normative section line 74 context note is:

```
> Both headers absent → `missing_auth_token`. Alias-path entries are EC-010 from BC-2.01.009 v1.0.2.
```

**Evidence:**

`prd-supplements/test-vectors.md` line 74:
```
> Both headers absent → `missing_auth_token`. Alias-path entries are EC-010 from BC-2.01.009 v1.0.2.
```

**Note:** This citation appears in the body normative table context note (above the critical-vector table), not in the §Trace block. Per SE-17g classification, a context note above a normative table is NORMATIVE content.

**Current BC-2.01.009 version:** v1.0.3 (confirmed: `grep "^version:" .factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md` → `version: "1.0.3"`). The EC-010 edge cases were present in v1.0.2 and remain unchanged in v1.0.3 (v1.0.3 only removed the fabricated F-FC-I005 parenthetical from the Forward Compat Contract row). The citation is therefore factually accurate as to the version that introduced EC-010 but is stale as a "current version" pin. Severity is LOW because the EC-010 content itself is unchanged — only the version number is stale.

**Recommended fix:** PO agent — update line 74 context note from `BC-2.01.009 v1.0.2` to `BC-2.01.009 v1.0.3`. SE-16d monotonicity required.

**Routing:** `vsdd-factory:product-owner` (supplement file ownership).

---

### GAP-R46-5 | MED | BC-2.01.004 Invariant 3 cites canonical header only; inconsistent with ADR-0005 dual-accept

**Description:** BC-2.01.004 Invariant 3 reads:

```
3. The `POST /shutdown` endpoint requires `X-Monocle-Authorization` authentication —
   unauthenticated shutdown requests receive HTTP 401.
```

This is inconsistent with ADR-0005 dual-accept, which applies at the auth-layer middleware level to ALL authenticated routes including `/shutdown`. The interface-definitions v1.3 correctly specifies `/shutdown` Auth as:

```
**Auth:** Canonical `X-Monocle-Authorization: monocle-v1:<64-hex-token>` **or** alias
`X-Claude-Code-Ide-Authorization: <64-hex>` (ADR-0005 dual-accept applies; WARN log
emitted on alias path)
```

The interface-definitions Edge Cases table also includes:
```
| `POST /shutdown` with alias header only | HTTP 200 + drain initiated + WARN deprecation log (ADR-0005 alias path) |
```

The SS-daemon-lifecycle.md correctly shows the auth_layer applied to the authenticated router that includes `/shutdown` (dual-accept middleware on all authenticated routes per ADR-0005). BC-2.01.009 (the definitive auth taxonomy BC) applies uniformly to all authenticated endpoints. BC-2.01.004 Invariant 3 narrows this to the canonical header only, creating a contradiction within the Phase 1 spec.

**Evidence:**

`ss-01/BC-2.01.004.md` line 62:
```
3. The `POST /shutdown` endpoint requires `X-Monocle-Authorization` authentication —
   unauthenticated shutdown requests receive HTTP 401.
```

`prd-supplements/interface-definitions.md` line 168:
```
**Auth:** Canonical `X-Monocle-Authorization: monocle-v1:<64-hex-token>` **or** alias
`X-Claude-Code-Ide-Authorization: <64-hex>` (ADR-0005 dual-accept applies; WARN log
emitted on alias path)
```

**Root cause:** BC-2.01.004 was authored before ADR-0005 was established (or the Round 4 dual-accept propagation sweep did not sweep BC-2.01.004's Invariant 3). The F-R105 closure chain Round 4 propagated ADR-0005 semantics to BC-2.01.009, BC-INDEX, and arch SS-daemon-lifecycle but did not update BC-2.01.004 INV-3.

**Recommended fix:** PO agent — update BC-2.01.004 Invariant 3 to include dual-accept semantics:
```
3. The `POST /shutdown` endpoint requires authentication per ADR-0005 dual-accept:
   canonical `X-Monocle-Authorization: monocle-v1:<64-hex>` (preferred) or alias
   `X-Claude-Code-Ide-Authorization: <64-hex>` (WARN deprecation log emitted).
   Unauthenticated requests (both recognized headers absent) receive HTTP 401
   `{"error":"missing_auth_token"}`.
```

**Routing:** `vsdd-factory:product-owner` (BC file ownership).

---

## §PASS Dimensions (detail)

**D1 — Spec ID references:** All BC-2.SS.NNN, VP-NNN, NFR-NNN, DI-NNN, ADR-NNN, and E-XXX-NNN IDs verified against their respective indexes. No phantom IDs found in normative content. BC-INDEX v1.4 maps 22 active BCs; VP-INDEX v1.4 maps 22 active VPs; DI registry in CAP-001/002/003 shows DI-001 through DI-007 (7 invariants); 5 ADRs in ARCH-INDEX ADR Registry; 15 error codes in error-taxonomy.md. All verified.

**D2 — Anchor links:** No dead anchor links detected in cross-reference prose. Cross-property references in VP files (VP-009 ↔ VP-002, VP-009 ↔ VP-004, VP-009 ↔ VP-008) all point to extant sections.

**D3 — Counts:**
- 22 BCs: BC-INDEX v1.4 Summary table = 10+8+4 = 22 ✓; 22 files exist on disk ✓
- 22 VPs: VP-INDEX v1.4 Summary = 10+8+4 = 22 ✓; 22 files exist on disk ✓
- 7 DIs: DI-001 through DI-007 across CAP-001, CAP-002, CAP-003 ✓
- 5 ADRs: ARCH-INDEX ADR Registry rows 1-5 ✓; 5 files exist on disk ✓
- 15 error codes: error-taxonomy.md catalog has E-AUTH-001/002/003, E-DAEMON-001-004, E-LOCK-001-003, E-ENG-001, E-FACT-001/002, E-RING-001, E-PROTO-001 = 15 ✓; PRD §5 says 15 ✓
- 8 test vectors for BC-2.01.009: test-vectors.md BC Vector Index row says 8; actual BC Canonical Test Vectors table has 8 rows ✓

**D4 — Naming consistency:** All normative content uses `X-Monocle-Authorization` as canonical and `X-Claude-Code-Ide-Authorization` as alias only per ADR-0005 decisions. Observation: SS-daemon-lifecycle.md line 172 has code-sketch comment `// X-Monocle-Authorization enforced on all routes above` which is imprecise (the auth_layer implements dual-accept, not canonical-only enforcement). This is a non-normative code comment in an architecture doc sketch — see Obs-R46-1.

**D5 — Traceability chains:** Every BC traces to a CAP (via BC-INDEX §Capability Anchor Justification and ARCH-INDEX §Capability traceability). Every VP maps to a BC via `source_bc:` frontmatter and §Source Contract. Every BC maps to an SS subsystem. All 7 DIs have at least one BC citing them (BC-2.01.001-010 cover DI-001/002/003; BC-2.02.* cover DI-004; BC-2.01.009 covers DI-005; BC-2.03.* cover DI-006/007). L2-INDEX traces to product-brief.md. All 3 CAP files trace to L2-INDEX.

**D7 — Timestamp monotonicity:**
- BC-INDEX §Trace chain: v1.1 (11:30) → v1.2 (18:00) → v1.3 (20:00) → v1.4 (23:00) ✓ monotonic
- VP-INDEX §Trace chain: v1.2 (20:30) → v1.3 (22:30) → v1.4 (22:50) ✓ monotonic
- PRD v1.26.4 §Trace: confirms prior sequence monotonic per SE-16d PASS attestations ✓

**D8 partial — /shutdown dual-accept:** Only BC-2.01.004 has the gap (GAP-R46-5). All other SS-01 BCs do not have a standalone auth invariant that would conflict. BC-2.01.009 (the auth taxonomy BC) is correct and comprehensive.

**D9 — /shutdown endpoint cross-references:**
- interface-definitions v1.3 §POST /shutdown: cites Contract BC-2.01.004 + BC-2.01.008 + BC-2.01.009 ✓
- VP-004 (graceful shutdown) has VP-009 as a cross-property citation ✓
- VP-009 has VP-004 §Post-condition 7 as a cross-property reciprocation ✓
- PRD §7 RTM row for BC-2.01.004 cites SS-daemon-lifecycle.md v1.0.30 ✓
- ADR-0005 dual-accept applies to /shutdown per interface-definitions edge-case table ✓

**D10 — E-AUTH-003 cross-references:**
- error-taxonomy.md: E-AUTH-003 catalog row exists, cites `BC-2.01.009 INV-6` and `ADR-0005 dual-accept deprecation signaling` ✓
- error-taxonomy.md Error-to-Module Mapping: E-AUTH-003 row exists with correct implementation site and test file ✓
- BC-2.01.009 INV-6: specifies the WARN log behavior that E-AUTH-003 captures; does not cross-cite E-AUTH-003 by code (this is acceptable — the error taxonomy is a supplement cross-referenced from the BC's behavior, not required to be reverse-cited)
- VP-009: references BC-2.01.009 INV-6 behavior in Pre-conditions and Counter-examples (CE-6, CE-7) with WARN-log assertion; does not use E-AUTH-003 code (acceptable — VP probes reference BC invariant numbers, not error taxonomy codes)
- nfr-catalog.md: does not reference E-AUTH-003 (acceptable — nfr-catalog links NFRs to VPs, not error taxonomy codes)
- PRD §5: cites E-AUTH-003 in Severity Definitions summary and in §Trace v1.26.4 ✓

Verdict D10: PASS-WITH-OBS. The cross-referencing surface follows established conventions (BCs define behavior, error-taxonomy provides discoverability, VPs test the behavior). No structural gap.

---

## §Open Observations

### Obs-R46-1 | LOW | SS-daemon-lifecycle.md code-sketch comment imprecise on auth scope

`SS-daemon-lifecycle.md` line 172, within the router Rust code sketch:
```rust
    .layer(auth_layer); // X-Monocle-Authorization enforced on all routes above
```

The comment says `X-Monocle-Authorization enforced` but the `auth_layer` implements ADR-0005 dual-accept (both canonical and alias headers accepted). The comment is factually misleading — it should say `auth_layer (dual-accept per ADR-0005) enforced on all authenticated routes`. This is a non-normative code comment in an architecture sketch, not in a behavioral specification. Severity LOW; not a blocking gap. Recommend PO/architect sweep when touching SS-daemon-lifecycle.md next.

### Obs-R46-2 | INFORMATIONAL | E-AUTH-003 not cited by code in BC-2.01.009, VP-009, nfr-catalog

The error taxonomy entry E-AUTH-003 is defined in error-taxonomy.md and cited by PRD §5. BC-2.01.009, VP-009, and nfr-catalog reference the underlying behavior (INV-6, WARN log contract) without using the error code label. This is consistent with the established pattern: BCs define behavior via invariants; VPs test the behavior via probe assertions; the error taxonomy provides a machine-referenceable code for implementation-phase lookup. No structural gap exists — the forward reference chain (E-AUTH-003 cites BC-2.01.009 INV-6) is complete; the reverse reference (BC-2.01.009 cites E-AUTH-003) is not required by template. INFORMATIONAL; no fix required.

---

## §Restructure Consistency Verdict

**Status: GAPS (5 findings; 2 HIGH, 2 MED, 1 LOW)**

The spec package retains strong structural integrity post-Round 5 closure and post-pre-R107 VP-INDEX fix. All 22 BCs and 22 VPs are structurally complete; traceability chains are intact; DI coverage is complete; the ADR-0005 dual-accept cascade is >95% propagated. The five remaining gaps are:

1. **GAP-R46-1 (HIGH):** 22 VP §References PRD pin stale (v1.26.3 vs v1.26.4). Introduced when VP-INDEX §Trace v1.4 (pre-R107 fix) updated the VP-INDEX §References but not the 22 individual VP files. This is a systematic citation gap identical in class to prior history (R13-001, R7-001, etc.). FV-only fix; no content cascade.

2. **GAP-R46-2 (HIGH):** All 22 BC Traceability §Architecture Source rows cite stale architecture file versions (SS-daemon-lifecycle v1.0.25 or v1.0.29 vs current v1.0.30; SS-core-types-and-abi v1.2.8 vs v1.2.11; SS-engine-module v1.1.15 vs v1.1.18). Systematic pin-citation gap introduced when architecture bumps were not propagated to BC Traceability rows. PO sweep required; no content cascade.

3. **GAP-R46-3 (MED):** All 4 PRD supplements have incorrect ADR-0005 filename in `inputs:` frontmatter. The file `ADR-0005-dual-accept-auth-header.md` does not exist; actual file is `ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md`. PO fix required.

4. **GAP-R46-4 (LOW):** test-vectors.md body context note cites BC-2.01.009 v1.0.2 in a normative location; current BC version is v1.0.3. Low impact (the underlying EC-010 content is unchanged across v1.0.2 → v1.0.3); PO fix required.

5. **GAP-R46-5 (MED):** BC-2.01.004 Invariant 3 specifies only `X-Monocle-Authorization` for `/shutdown` auth, contradicting ADR-0005 dual-accept that applies at the auth-layer level to all authenticated endpoints including `/shutdown`. The interface-definitions v1.3 (F-R106 Round 5) correctly includes the dual-accept semantics; BC-2.01.004 was not swept in that round.

**Gate result: FAIL — blocking findings exist (GAP-R46-1 HIGH, GAP-R46-2 HIGH)**

Counter remains 0/3. New closure chain required before D-047 strict pass 1 can advance.

**Consistency score: ~93%** — substantive content layer fully converged; all gaps are version-pin propagation or single BC invariant omission class.

---

## §Routing Summary

| GAP | Routing Agent | Fix Type | Priority |
|-----|--------------|----------|----------|
| GAP-R46-1 | `vsdd-factory:formal-verifier` | 22-VP §References PRD pin sweep v1.26.3 → v1.26.4 | HIGH / immediate |
| GAP-R46-2 | `vsdd-factory:product-owner` | 22-BC Traceability §Architecture Source pin sweep | HIGH / immediate |
| GAP-R46-3 | `vsdd-factory:product-owner` | 4-supplement `inputs:` ADR-0005 filename correction | MED / same burst |
| GAP-R46-4 | `vsdd-factory:product-owner` | test-vectors.md line 74 BC version cite refresh | LOW / same burst |
| GAP-R46-5 | `vsdd-factory:product-owner` | BC-2.01.004 INV-3 dual-accept propagation | MED / same burst |

**Recommended burst structure:** PO burst (GAP-R46-2 + GAP-R46-3 + GAP-R46-4 + GAP-R46-5 together) → FV burst (GAP-R46-1) per Extension 15 SERIAL cascade discipline (architect-source bumps propagate to PO before FV).

Note: GAP-R46-2 involves the architecture source version pins in BC bodies — these pins reference the architecture docs as external inputs to the BCs. No architecture doc content changes are required; the architecture doc versions are stable. This is a pure citation-update sweep.
