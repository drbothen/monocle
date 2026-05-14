---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.3 d8e66c3 + VP v1.3 2b24735 + arch v1.0.10 dc3af71; R3-001 closure chain applied; D-047 strict pass 1 of 3 (attempt 2)"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-14T23:55:00Z
pass_number: 1
policy: D-047-strict
---

# Adversarial Review Pass R65 — Phase 1 (D-047 Strict, Pass 1 attempt 2 of 3)

## Summary

**Verdict:** FINDINGS (3 — D-047 strict FAIL)

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 2 |
| MEDIUM | 0 |
| LOW | 0 |
| **TOTAL** | **3** |

All three findings concern a single defect cluster in `SS-daemon-lifecycle.md` v1.0.10's BC-AUTH-002 block. The defects predate v1.0.10 — likely introduced in the F-R62-8 fix-burst that collapsed the 3-body taxonomy down to 2 bodies but did not back-propagate the prose lead-in count or fully reconcile the Bearer-header case. These defects survived R62, R63, R64 fresh-context adversary passes and are caught now via deeper semantic content review.

## 22-BC ↔ 22-VP Mapping Audit

All 22 BCs ↔ 22 VPs map 1:1 with matching IDs, test names, and test file paths. Cross-version-pin consistency holds (PRD v1.3 cites arch v1.0.10; VP v1.3 cites arch v1.0.10 + PRD v1.3). Path coherence: 22/22 verbatim. Name coherence: 22/22 verbatim.

## Findings

### F-R65-1 [HIGH] — Architecture BC-AUTH-002 "Three auth failure modes" prose contradicts the 2-mode enumeration

**File:** `.factory/specs/architecture/SS-daemon-lifecycle.md`
**Locations:** Line 307 + Line 593

**Evidence:**
- Line 307: "- **BC-AUTH-002:** Three auth failure modes are specified:" → renders 2-row table at lines 309-316 (Missing header, Invalid token).
- Line 593: "| BC-AUTH-002 | Three auth failure modes: (1) absent header → HTTP 401 ...; (2) header present but fails for any reason ..." — enumerates (1) and (2) only.
- PRD v1.3 BC-AUTH-002 Invariant 1: "The two-body taxonomy ... There is no third body."
- VP v1.3 §VP-AUTH-002 heading: "Two-Body Taxonomy".
- D-055 architect adjudication: 2 bodies remain (`invalid_auth_token_format` RETIRED).

**Routing:** architect (arch fix: "Three" → "Two" at lines 307 + 593).

### F-R65-2 [CRITICAL] — Architecture BC-AUTH-002 internal contradiction: Bearer-header case has two conflicting expected error bodies

**File:** `.factory/specs/architecture/SS-daemon-lifecycle.md`
**Locations:** Lines 320-321 (Bearer → `invalid_auth_token`) AND Line 333 (Bearer → `missing_auth_token`)

**Evidence:**
- Lines 320-321: "Phase 4 OAuth2 federation tokens use `Authorization: Bearer` ... they receive HTTP 401 `{\"error\":\"invalid_auth_token\"}` (header present but `Authorization: Bearer ...` does not begin with `monocle-v1:`)."
- Line 333: "`Authorization: Bearer fake` (wrong header name) → HTTP 401 `{\"error\":\"missing_auth_token\"}`"

Both occurrences live within BC-AUTH-002's normative block and describe the IDENTICAL scenario (inbound request with `Authorization: Bearer <anything>` and no `X-Monocle-Authorization`) with contradictory expected error bodies.

**Cross-artifact agreement check:**
- PRD v1.3 BC-AUTH-002 postcondition 3: Bearer → `missing_auth_token`
- PRD v1.3 Canonical Test Vector row 5: Bearer → `missing_auth_token`
- VP v1.3 VP-AUTH-002 probe 5: Bearer → `missing_auth_token`

PRD/VP and arch line 333 agree on `missing_auth_token`. Arch lines 320-321 is the outlier.

**Production-grade reasoning:** From the server's perspective, the inbound request has NO `X-Monocle-Authorization` header — so the structurally correct disposition is `missing_auth_token`. The presence of an `Authorization: Bearer` header is irrelevant (different header). Arch lines 320-321 treats Bearer-presence as equivalent to `X-Monocle-Authorization`-presence, which is a logic error.

**Routing:** architect (arch fix: lines 320-321 → `missing_auth_token`).

### F-R65-3 [HIGH] — Arch-vs-PRD/VP cross-artifact contradiction on Bearer test vector

**Files:**
- Arch SS-daemon-lifecycle.md lines 320-321 (Bearer → `invalid_auth_token`)
- PRD v1.3 line 520 + line 544 (Bearer → `missing_auth_token`)
- VP v1.3 line 876 (Bearer → `missing_auth_token`)

**Why a separate finding from F-R65-2:** F-R65-2 is the arch internal contradiction. F-R65-3 is the cross-artifact contradiction (arch's claim vs PRD/VP's claim). Per CLAUDE.md Architectural Authority, when artifacts disagree, the LATER/MORE-SPECIFIC wins — but arch internally disagrees with itself (line 333 agrees with PRD/VP). Resolution: F-R65-2 fix closes F-R65-3 automatically.

**Routing:** architect (closes with F-R65-2 fix).

## Frozen META Residual Catalog Status (D-054)

| ID | Re-litigated? |
|----|---------------|
| F-R55-adv-1 | NO |
| F-R55-adv-3 | NO |
| F-R61-adv-1 | NO |
| F-R61-2 | NO |

All 4 frozen entries respected. None of the R65 findings are META-class; all 3 are content defects.

## Novelty Assessment

**Novelty: HIGH.** These are NEW content findings, not META-class. They predate v1.0.10 (likely introduced in F-R62-8 fix-burst) but survived R62, R63, R64 fresh-context passes — caught by R65's deeper semantic content review of BC-AUTH-002. The L-F-R63-PARTIAL-FIX lesson is directly relevant: F-R62-8 should have updated all sibling sites (lead-in count, Bearer disposition) when collapsing 3-body to 2-body taxonomy.

## Pass 1 attempt 2 Verdict and Pass 1 attempt 3 Readiness

**Verdict:** FINDINGS — D-047 strict pass 1 attempt 2 FAILS. Counter: 0/3.

**Required closure chain:**
1. architect: arch v1.0.10 → v1.0.11 (F-R65-1 count fix; F-R65-2 Bearer disposition fix)
2. product-owner: PRD v1.4 (arch pin propagation v1.0.10 → v1.0.11; NO content changes — PRD already has correct 2-body taxonomy and Bearer disposition)
3. formal-verifier: VP v1.4 (arch pin propagation + PRD pin propagation; NO content changes — VP already has correct 2-body taxonomy)
4. state-manager: STATE.md update
5. R66 + cons R5 dispatch as pass 1 attempt 3
