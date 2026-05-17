---
document_type: consistency-report
round: R45
verdict: GAPS
phase: phase-1-spec-crystallization
cycle: cycle-001
producer: vsdd-factory:consistency-validator
timestamp: 2026-05-17T21:30:00Z
companion: adversary-R106 (parallel, independent)
artifact_pins:
  prd: "v1.26.3"
  bc_index: "v1.3"
  vp_index: "v1.2"
  arch_index: "v1.0.4"
  l2_index: "v1.0.6"
  product_brief: "v1.4.24"
  ss_daemon_lifecycle: "v1.0.29"
  ss_core_types_abi: "v1.2.11"
  ss_engine_module: "v1.1.18"
  ss_deps_pin_manifest: "v1.1.17"
  dtu_assessment: "v1.7.3"
  adr_0005: "v1.0.1"
  bc_2_01_009: "v1.0.2"
  vp_009: "v1.0.3"
---

# Consistency Validation Report: R45 — Post-R105 + R44 Closure Chain

> Companion to adversary pass R106. Fresh context. Independent perspective.
> Read-only validation. No artifact modifications.

## Context

R45 validates the restructured Phase 1 spec package post-D-128 Option A full closure
(14 R105 + 5 R44 + 3 ADR-0005-cascade follow-ups + 1 VP-019 fix). The package now has:
22 sharded BCs (ss-01/ss-02/ss-03), 22 sharded VPs, sharded domain-spec (3 CAPs), sharded
architecture (7 SS docs + ARCH-INDEX), PRD v1.26.3, ADR-0005 (auth header dual-accept).

Counter state entering R45: 0/3 (D-047 strict; pass-1 attempt 2 of T-127').

---

## 10-Dimension Summary

| # | Dimension | Result | Notes |
|---|-----------|--------|-------|
| 1 | ID References (BC, VP, NFR, DI, ADR) resolve to canonical sources | PASS | All 22 BC IDs, 22 VP IDs, 12 NFR IDs, 7 DI IDs, 5 ADR IDs resolve. Old-form IDs in BC-INDEX and VP-INDEX Renumbering Appendices correctly preserved as append-only history. |
| 2 | Anchor links (§Section cites) | PASS | Core §-anchor citations validated in key artifacts. No broken navigable anchors found. |
| 3 | Inline counts (BCs, VPs, DIs, ADRs, NFRs) | PASS | 22 BCs in BC-INDEX (10+8+4) = 22 actual files. 22 VPs in VP-INDEX (10+8+4) = 22 actual files. 7 DIs in L2-INDEX = all cited. 5 ADRs in ARCH-INDEX = 5 actual files. 4 prd-supplements = 4 actual files. All arithmetic checks pass. |
| 4 | Naming consistency (headers, tokens, field names) | PASS | `X-Monocle-Authorization` is consistently canonical throughout all artifacts. `X-Claude-Code-Ide-Authorization` is consistently labelled as compatibility alias only (never as canonical). `authToken` (camelCase) is consistent in interface-definitions.md live schema (GAP-R44-3 closed). |
| 5 | Traceability chains (FR/NFR → VP → BC → arch SS → L2 DI/CAP) | PASS with OBS | Full chain present and complete. All 7 DIs covered by at least one BC. All 12 NFRs covered (NFR-007/008 correctly deferred to Phase 6; NFR-011 to Phase 4). All 22 VPs have source_bc field pointing to active BCs. VP probe matrix coverage gap is a separate GAP (see GAP-R45-1). |
| 6 | Frontmatter version pin propagation (current-pointer cites) | GAP | PRD traces_to is stale (cites old SS doc versions + BC-INDEX v1.1). Brief line 246 stale current-pointer. VP-INDEX §References cites BC-INDEX v1.2 not v1.3. CLAUDE.md line 225 stale manifest label. See GAP-R45-2, GAP-R45-3, GAP-R45-4, GAP-R45-5. |
| 7 | UTC ISO-8601 timestamp monotonicity | PASS | SE-16d chain verified monotonic: ARCH-INDEX 19:00 < PRD 19:30 < BC-INDEX 20:00 = L2-INDEX 20:00 < VP-INDEX 20:30. VP-019 21:00 is latest. All UTC `Z` form. |
| 8 | ADR-0005 cascade integrity | PARTIAL GAP | BC-2.01.009 ✓, SS-daemon-lifecycle ✓, dtu-assessment ✓, CAP-001 ✓, product-brief ✓. VP-009 probe matrix NOT updated for alias-path probes. See GAP-R45-1. |
| 9 | Sharding integrity (index↔file consistency) | PASS | Every BC file has traces_to: prd.md. Every VP file has traces_to: prd.md and source_bc: BC-2.SS.NNN. BC-INDEX references all 22 BC files. VP-INDEX references all 22 VP files. L2-INDEX lists all 3 CAP files. ARCH-INDEX lists all 7 SS docs. All directories have INDEX files. |
| 10 | BC title H1 ↔ BC-INDEX title consistency | PASS | All 22 BC file H1 headings match BC-INDEX title column exactly. VP file H1 titles are richer than VP-INDEX title column (VP-INDEX has abbreviated summaries) — consistent across all 22 VPs and matches established pattern (OBS-R45-1). |

**Dimension pass count: 8 PASS / 2 PARTIAL-GAP (6 and 8)**

---

## Findings

### GAP-R45-1 — VP-009 Probe Matrix Missing ADR-0005 Alias-Path Probes (HIGH)

**Severity:** HIGH  
**Artifact:** `verification-properties/vp-009-auth-header-validation.md` v1.0.3  
**Routing:** vsdd-factory:formal-verifier

**Description:**

BC-2.01.009 was updated in Round 4 (T-128n, 2026-05-17T20:00:00Z) to reflect ADR-0005
dual-accept semantics. BC-2.01.009's Verification Properties table now claims VP-009 covers:

> "All alias-path failure modes return HTTP 401 `{"error":"invalid_auth_token"}` with WARN log emitted"
> "Alias-path success returns HTTP 200 with WARN log emitted"
> "Canonical priority: when both headers present, `X-Monocle-Authorization` wins; no WARN log emitted"

However, VP-009 (v1.0.3, timestamp 2026-05-17T20:30:00Z) has a 7-probe matrix (probes
9.1–9.7) that covers ONLY the canonical `X-Monocle-Authorization` path. There are **zero
alias-path probes** (`X-Claude-Code-Ide-Authorization`) and **zero both-headers-present
probes** in VP-009.

**Evidence:**

- `behavioral-contracts/ss-01/BC-2.01.009.md`, lines 92–96 (VP-009 coverage claims):
  ```
  | VP-009 | All alias-path failure modes return HTTP 401 {"error":"invalid_auth_token"} with WARN log emitted | integration |
  | VP-009 | Alias-path success returns HTTP 200 with WARN log emitted | integration |
  | VP-009 | Canonical priority: when both headers present, X-Monocle-Authorization wins; no WARN log emitted | integration |
  ```
- `verification-properties/vp-009-auth-header-validation.md`, lines 133–140 (complete probe matrix):
  ```
  | 9.1 | (no X-Monocle-Authorization header) | 401 | {"error":"missing_auth_token"} |
  | 9.2 | X-Monocle-Authorization: deadbeef...64chars (bare token) | 401 | {"error":"invalid_auth_token"} |
  | 9.3 | X-Monocle-Authorization: monocle-v2:... (wrong version prefix) | 401 | {"error":"invalid_auth_token"} |
  | 9.4 | X-Monocle-Authorization: monocle-v1: (prefix only) | 401 | {"error":"invalid_auth_token"} |
  | 9.5 | Authorization: Bearer fake-token (wrong header name) | 401 | {"error":"missing_auth_token"} |
  | 9.6 | X-Monocle-Authorization: monocle-v1:<wrong-64-hex> | 401 | {"error":"invalid_auth_token"} |
  | 9.7 | X-Monocle-Authorization: monocle-v1:<correct-64-hex> | 200 | (route body) |
  ```
  No probe 9.8 (alias wrong secret), 9.9 (alias correct secret), 9.10 (both headers present)
  appear anywhere in the VP-009 file — confirmed by `grep -n "alias\|X-Claude-Code-Ide\|9.8\|9.9\|9.10\|EC-010\|EC-011\|EC-012" vp-009-auth-header-validation.md` → 0 hits in body (only §Trace history lines mentioning "VP alias" and "PRD alias" in reference update context).

**Impact:**

ADR-0005 adds 3 new behavioral paths to BC-2.01.009 (alias-path failure, alias-path success,
both-headers-present canonical-wins). BC-2.01.009 explicitly asserts VP-009 covers these paths.
VP-009 does not. The gap means:
1. The verification coverage claimed in BC-2.01.009 is false — the BC overstates its VP coverage.
2. Implementers following VP-009 will write tests for only 7 of the ~10 behavioral cases BC-2.01.009 specifies.
3. The alias path (primary interop with real Claude Code) has no formal VP proof obligation.

**Also note:** VP-009 probe 9.1 tests "no X-Monocle-Authorization header" returning `missing_auth_token`.
Under dual-accept (BC-2.01.009 Postcondition 1), "missing" now means BOTH headers absent. Probe 9.1
should distinguish: absent X-Monocle-Authorization + X-Claude-Code-Ide-Authorization ALSO absent. The
current probe description is now ambiguous under the dual-accept model (an implementer could argue
probe 9.1 only tests one of the two absent-header scenarios).

**Recommended Fix:**

FV to update VP-009 to add alias-path probes:
- Probe 9.8: `X-Claude-Code-Ide-Authorization: <wrong-64-hex>` (alias path, wrong secret) → HTTP 401 + WARN log
- Probe 9.9: `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (alias path, correct secret) → HTTP 200 + WARN log
- Probe 9.10: Both `X-Monocle-Authorization` (valid) + `X-Claude-Code-Ide-Authorization` (valid) present → HTTP 200, canonical wins, NO WARN log
- Refine probe 9.1 description: "no `X-Monocle-Authorization` AND no `X-Claude-Code-Ide-Authorization`"
- Sync VP-009 title in VP-INDEX to reflect dual-accept scope: e.g., "Auth Header Dual-Accept Two-Body Taxonomy"

---

### GAP-R45-2 — PRD traces_to Frontmatter Cites Stale SS Doc Versions and BC-INDEX v1.1 (MED)

**Severity:** MED  
**Artifact:** `prd.md` v1.26.3, line 11  
**Routing:** vsdd-factory:product-owner

**Description:**

The PRD `traces_to` frontmatter field (line 11) is a current-pointer field citing the versions
of inputs at PRD authoring time. It has not been updated to reflect subsequent SS doc and index
bumps that occurred during the T-128 restructuring chain (Dispatches 1–8, 2026-05-17).

**Evidence:**

`prd.md` line 11:
```yaml
traces_to: "product-brief.md v1.4.23; vision-synthesis v1.1.2; SS-daemon-lifecycle.md v1.0.25;
SS-core-types-and-abi.md v1.2.8; SS-engine-module.md v1.1.15; SS-deps-pin-manifest.md v1.1.17;
architecture/ARCH-INDEX.md; behavioral-contracts/BC-INDEX.md v1.1;
22 BCs sharded under behavioral-contracts/ss-NN/ (Dispatch 2 commit d02bf2a + Dispatch 3 commit f259ade);
domain-spec/L2-INDEX.md (pending BA Dispatch 6)"
```

Stale citations in this field:

| Field | Cited | Actual Current | Delta |
|-------|-------|---------------|-------|
| SS-daemon-lifecycle.md | v1.0.25 | v1.0.29 | +4 versions |
| SS-core-types-and-abi.md | v1.2.8 | v1.2.11 | +3 versions |
| SS-engine-module.md | v1.1.15 | v1.1.18 | +3 versions |
| BC-INDEX | v1.1 | v1.3 | +2 versions |
| domain-spec/L2-INDEX.md | "(pending BA Dispatch 6)" | v1.0.6 (COMPLETE) | resolved |
| ADR-0005 | absent from inputs list | accepted 2026-05-17 | new input |

Additionally, the `inputs:` frontmatter field (line 9) lists ADR-0001 through ADR-0004 but does
NOT include `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md`
even though ADR-0005 materially affects PRD content (BC-2.01.009 postconditions, §7 RTM row
for BC-2.01.009, BC-2.01.009 test file assertion claims).

**Note on §7 RTM version pins:** The SS doc version pins in PRD §7 RTM rows (e.g., "SS-daemon-lifecycle.md
v1.0.25 §GET /healthz") are intentional historical pinpoints (version-at-requirement-introduction),
not current-pointer citations. These are explicitly governed by the D-042 "leave-alone per sweep
protocol" for historical pinpoints. They are PASS under this dimension. Only the `traces_to` and
`inputs` frontmatter fields are current-pointer fields that should track current versions.

**Recommended Fix:**

PO to update `prd.md` frontmatter:
1. `traces_to`: refresh all stale version citations; remove "(pending BA Dispatch 6)" note
2. `inputs`: add `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md`

---

### GAP-R45-3 — product-brief.md Line 246 Stale Current-Pointer: SS-engine-module.md v1.1.15 (MED)

**Severity:** MED  
**Artifact:** `product-brief.md` v1.4.24, line 246  
**Routing:** vsdd-factory:product-owner

**Description:**

Brief line 246 (Forward-compatibility contracts row in Phase 1 Success Criteria table) cites
`SS-engine-module.md v1.1.15` as a current-pointer. This citation was last updated in brief
v1.4.22 (commit context). The D-042 sweep in brief v1.4.24 incorrectly classified this as
"confirmed CURRENT."

**Evidence:**

`product-brief.md` line 246 (body):
```
... Per `SS-core-types-and-abi.md`, `SS-daemon-lifecycle.md` v1.0.7, and `SS-engine-module.md` v1.1.15.
```

`product-brief.md` line 79 (§Trace v1.4.24 D-042 sweep result):
```
SS-engine-module.md v1.1.15 at line 253 confirmed CURRENT per v1.4.22 trace.
```

Actual current SS-engine-module.md version:
```
version: "1.1.18"  # frontmatter, 2026-05-17T17:00:00Z
```

Timeline:
- SS-engine-module.md v1.1.16 authored: 2026-05-17T11:00:00Z (Dispatch 1 — structural fix)
- SS-engine-module.md v1.1.18 authored: 2026-05-17T17:00:00Z (Dispatch 5b/T-128h — BC ID canonicalization)
- brief v1.4.24 D-042 sweep ran: 2026-05-17T20:00:00Z (after both bumps were committed)

The D-042 sweep missed the v1.1.15→v1.1.16→v1.1.18 progression because the SS grep scope
check compared against the v1.4.22 entry ("v1.1.15 confirmed CURRENT") rather than running
a fresh `grep` against the actual file's current frontmatter version.

This is the 8th recurrence of the cross-artifact citation-staleness META-pattern (prior
occurrences: brief v1.4.13, v1.4.15, v1.4.16, v1.4.19, v1.4.20, v1.4.21, v1.4.22).

**Recommended Fix:**

PO to update brief line 246: `SS-engine-module.md v1.1.15` → `SS-engine-module.md v1.1.18`.
Run D-042 sweep against actual file frontmatter version, not prior-entry version.

---

### GAP-R45-4 — VP-INDEX §References Cites BC-INDEX v1.2 (Current: v1.3) (LOW)

**Severity:** LOW  
**Artifact:** `verification-properties/VP-INDEX.md` v1.2, lines 141 and 174  
**Routing:** vsdd-factory:formal-verifier

**Description:**

VP-INDEX §References section cites BC-INDEX v1.2. The VP-INDEX §Trace v1.2 was explicitly
created to refresh this citation from v1.1→v1.2 (F-R105-13). However, BC-INDEX was subsequently
bumped to v1.3 in T-128n Round 4 (BC-2.01.009 ADR-0005 dual-accept update, 2026-05-17T20:00:00Z)
— exactly 30 minutes BEFORE VP-INDEX v1.2 was authored (2026-05-17T20:30:00Z).

**Evidence:**

`verification-properties/VP-INDEX.md` lines 141 and 174:
```
- BC index: `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure; supersedes v1.1 Dispatch 3 commit f259ade).
```
```
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7).
```

`behavioral-contracts/BC-INDEX.md` frontmatter:
```yaml
version: "1.3"
timestamp: 2026-05-17T20:00:00Z
```

VP-INDEX v1.2 frontmatter:
```yaml
timestamp: 2026-05-17T20:30:00Z
```

BC-INDEX v1.3 was EARLIER (20:00) than VP-INDEX v1.2 (20:30). The VP-INDEX §Trace v1.2
text was authored as part of the same dispatch context as BC-INDEX v1.3 (both in Round 4),
but the V-INDEX §References body was not updated to reflect the final BC-INDEX v1.3 after
the BC-2.01.009 update was merged.

**Impact:** LOW — no implementer confusion (VP-INDEX §References is informational; the
normative VP-to-BC links are in the VP-INDEX table rows via `Source BC` column, which
correctly cite BC-2.01.009 not a BC-INDEX version).

**Recommended Fix:**

FV to update VP-INDEX §References:
- Line 141: `v1.2 (commit 61133a7 ...)` → `v1.3 (commit <BC-INDEX-v1.3-commit> — BC-2.01.009 ADR-0005 dual-accept update, Round 4)`
- Line 174: same update

---

### GAP-R45-5 — CLAUDE.md Line 225 Stale Manifest Label "current v1.1.15" (LOW)

**Severity:** LOW  
**Artifact:** `CLAUDE.md` (project root), line 225  
**Routing:** vsdd-factory:state-manager

**Description:**

CLAUDE.md line 225 contains a routing example that includes the text "current v1.1.15" for
`SS-deps-pin-manifest.md`. This was previously F-R99-7 LOW (corrected from v1.1.1 to v1.1.15
in commit 8d78984). The manifest has since been bumped to v1.1.17.

**Evidence:**

`CLAUDE.md` line 225:
```
The `SS-deps-pin-manifest.md` stub was correctly extracted by product-owner but its production
version (v1.1.1 at architect's stub-completion; current v1.1.15) was completed by architect.
```

`architecture/SS-deps-pin-manifest.md` frontmatter:
```yaml
version: "1.1.17"
```

The F-R99-7 fix (commit 8d78984, 2026-05-15) corrected "v1.1.1" to the then-current "v1.1.15".
Subsequent manifest bumps to v1.1.17 (2026-05-17, T-128g pin refresh + minor corrections)
were not propagated to this informational example.

**Impact:** LOW — the routing example is illustrative prose, not a normative specification.
However, CLAUDE.md's §Architectural Authority section #1 says SS-deps-pin-manifest.md is the
canonical version-pin source, and the inline "current" label creates confusion about which
version is canonical.

**Recommended Fix:**

State-manager to update CLAUDE.md line 225:
`current v1.1.15` → `current v1.1.17`

---

## Observations

### OBS-R45-1 — VP File H1 Titles Richer Than VP-INDEX Title Column (Informational)

**Severity:** OBS (informational, non-blocking)  
**Artifact:** All 22 VP files vs `VP-INDEX.md`

VP-INDEX title column uses abbreviated summary titles (e.g., "Auth Header Two-Body Taxonomy",
"Healthz Endpoint — Unauthenticated Liveness 200/503"). VP file H1 headings are richer (e.g.,
"Auth Header Validation — Two-Body Taxonomy (`missing_auth_token` vs `invalid_auth_token`)",
"Healthz Endpoint — Unauthenticated Liveness 200/503 with Uptime + Version"). This is a
consistent pattern across all 22 VPs — not a random drift. The abbreviated form in VP-INDEX
serves as a navigation summary; the full form in VP files is the normative title.

This is NOT a blocking finding under criterion 75 (which targets BC file H1 vs BC-INDEX title
mismatch causing implementer confusion). The VP-INDEX abbreviation pattern does not mislead;
the file provides the complete title. However, if VP-INDEX titles are intended as source-of-truth
(per the VP-INDEX preamble "VP INDEX is source of truth"), FV should align file H1 titles with
index titles at next VP sweep.

---

### OBS-R45-2 — VP-INDEX §References "Current as of" Timestamp Stale (Informational)

**Severity:** OBS (informational, non-blocking)  
**Artifact:** `verification-properties/VP-INDEX.md` v1.2, line 133

`VP-INDEX.md` line 133:
```
- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
```

VP-INDEX is now at v1.2, timestamp 2026-05-17T20:30:00Z. The "Current as of" informational
timestamp was not updated when the §References section was refreshed in §Trace v1.2.
Non-blocking: the timestamp is in the informational §References section, not normative
frontmatter. Next VP-INDEX touch should update this line.

---

### OBS-R45-3 — SS-engine-module.md Version History Skip v1.1.16 → v1.1.18 (Informational)

**Severity:** OBS (informational, non-blocking)  
**Artifact:** `architecture/SS-engine-module.md`

SS-engine-module.md §Trace sections show v1.1.16 (2026-05-17T11:00:00Z, Dispatch 1 structural fix)
directly followed by v1.1.18 (2026-05-17T17:00:00Z, F-R105-8 BC ID canonicalization). No v1.1.17
§Trace entry is present. Similar to the documented L2-INDEX v1.0.1 skip (which GAP-R44-5 resolved
by retroactive backfill), this version skip should be documented for audit-trail completeness.

---

### OBS-R45-4 — ADR-0005 inputs Field Contains Malformed Path Prefix (Informational)

**Severity:** OBS (borderline LOW, flagged for architect review)  
**Artifact:** `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md` v1.0.1, line 15

`ADR-0005` frontmatter `inputs:` field (line 15):
```yaml
inputs: [product-brief.md, architecture/SS-daemon-lifecycle.md, specs/behavioral-contracts/ss-01/BC-2.01.009.md, semport/...]
```

The path `specs/behavioral-contracts/ss-01/BC-2.01.009.md` has an extraneous `specs/` prefix.
All other artifacts under `.factory/specs/` use paths relative to `.factory/specs/` (e.g.,
`behavioral-contracts/ss-01/BC-2.01.009.md`). The `specs/` prefix would cause the
`compute-input-hash` tool to resolve an incorrect path (`.factory/specs/specs/...`).

Comparison: ADR-0001 inputs use `research/domain-monocle-vision-synthesis.md`; ADR-0004 uses
`SS-core-types-and-abi.md`. The canonical convention is path-relative-to-`.factory/specs/`.

This is informational (the input-hash field is already computed and stored; no runtime failure).
However, if `compute-input-hash --scan` is run at Phase 3 start, this path will generate a
lookup error. Architect should correct the path before Phase 3.

---

## Cross-Check: R44 Closure Status

| R44 GAP | Closed? | Evidence |
|---------|---------|---------|
| GAP-R44-1 (nfr-catalog stale VP IDs) | CLOSED | nfr-catalog now uses VP-001/003/005/006/008 (new canonical IDs); phantom IDs removed |
| GAP-R44-2 (CAP-001 wrong auth header) | CLOSED | CAP-001 v1.3 (L2-INDEX §Trace v1.0.6) correctly documents dual-accept with X-Monocle-Authorization canonical + X-Claude-Code-Ide-Authorization alias |
| GAP-R44-3 (interface-definitions auth_token vs authToken) | CLOSED | interface-definitions.md live schema uses `authToken` (camelCase); `auth_token` appears only in §Trace before-evidence block |
| GAP-R44-4 (PRD §5 "6 subsystem abbreviations" vs 7 actual) | CLOSED (per PRD §Trace v1.26.3 F-R105-12) | Not independently re-verified in R45 (accepted based on §Trace evidence) |
| GAP-R44-5 (L2-INDEX version skip documentation) | CLOSED | L2-INDEX §Trace v1.0.5 contains retroactive backfill for v1.0.1 and v1.0.2 |
| OBS-R44-1 (stale VP IDs in BC-2.01.005/006 body) | CLOSED | BC-INDEX §Trace v1.2 confirms 2 stale VP IDs corrected in BC-2.01.005 and BC-2.01.006 |

---

## ADR-0005 Cascade Integrity Check (Dimension 8)

| Artifact | Required Update | Status | Evidence |
|----------|----------------|--------|---------|
| BC-2.01.009 | Postconditions 1-4 extended for dual-accept; 3 new ECs; 2 new test vectors | DONE | v1.0.2, timestamp 20:00, 4 postconditions present |
| SS-daemon-lifecycle.md | Router-level dual-accept middleware spec | DONE | v1.0.29, lines 147-172, 355-415 confirmed dual-accept spec + Rust stub |
| dtu-assessment.md | ADR-0005 rationale block in endpoint matrix | DONE | v1.7.3, lines 101-110, ADR-0005 rationale block present |
| CAP-001-daemon-lifecycle.md | §P2 step 1 dual-accept alias note | DONE | v1.3, lines 134-140 dual-accept note present |
| product-brief.md | Lines 116 + 239 dual-accept propagation | DONE | v1.4.24, line 117 + line 240 updated |
| VP-009 probe matrix | Alias-path probes (9.8/9.9/9.10) | **NOT DONE** | v1.0.3 — probe matrix only has 9.1-9.7 (canonical path only) |
| PRD inputs field | ADR-0005 listed as input | **NOT DONE** | v1.26.3 inputs field lists ADR-0001..0004 only |

---

## Dimension Pass/Fail Summary

| Dimension | Pass? |
|-----------|-------|
| 1. ID reference integrity | PASS |
| 2. Anchor link integrity | PASS |
| 3. Count claims | PASS |
| 4. Naming consistency | PASS |
| 5. Traceability chains | PASS |
| 6. Frontmatter version pin propagation | FAIL (GAP-R45-2, GAP-R45-3, GAP-R45-4, GAP-R45-5) |
| 7. Timestamp monotonicity | PASS |
| 8. ADR-0005 cascade integrity | FAIL (GAP-R45-1) |
| 9. Sharding integrity | PASS |
| 10. BC title consistency | PASS |

**Dimensions PASS: 8 of 10**

---

## Overall Verdict

**GAPS — counter holds at 0/3**

| Severity | Count | GAP IDs |
|----------|-------|---------|
| HIGH | 1 | GAP-R45-1 (VP-009 missing alias-path probes — BC overclaims VP coverage) |
| MED | 2 | GAP-R45-2 (PRD traces_to stale + missing ADR-0005 input), GAP-R45-3 (brief stale SS-engine-module pin) |
| LOW | 2 | GAP-R45-4 (VP-INDEX BC-INDEX v1.2 cite), GAP-R45-5 (CLAUDE.md manifest label) |
| OBS | 4 | OBS-R45-1 through OBS-R45-4 |

**Restructure Consistency Verdict:** The 22-BC, 22-VP, 3-CAP, 7-SS-doc sharded structure is
internally consistent. Index↔file mappings are complete. ID renumbering appendices are intact.
DI coverage is complete across all 7 invariants. NFR coverage is complete (deferred items
correctly anchored to Phase 3/4/6). The blocking gaps are in post-restructuring propagation:
specifically VP-009 not updated for ADR-0005 dual-accept semantics (HIGH), and version pin
staleness in PRD/brief/VP-INDEX frontmatter (MED/LOW).

**Gate result: FAIL — counter holds at 0/3 per D-047 strict.**

---

## Remediation Routing Summary

| GAP | Blocking? | Routing |
|-----|----------|---------|
| GAP-R45-1 (VP-009 alias probes) | YES — HIGH | vsdd-factory:formal-verifier |
| GAP-R45-2 (PRD traces_to + inputs) | YES — MED | vsdd-factory:product-owner |
| GAP-R45-3 (brief SS-engine-module pin) | YES — MED | vsdd-factory:product-owner |
| GAP-R45-4 (VP-INDEX BC-INDEX v1.2) | YES — LOW | vsdd-factory:formal-verifier |
| GAP-R45-5 (CLAUDE.md manifest label) | NO — LOW informational | vsdd-factory:state-manager |
| OBS-R45-1 through OBS-R45-4 | NO | Informational only |
