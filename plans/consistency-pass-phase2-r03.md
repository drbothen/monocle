---
document_type: consistency-pass
level: ops
phase: phase-2
round: r03
producer: consistency-validator
status: GAPS
gaps_total: 2
gaps_by_severity:
  critical: 0
  high: 0
  medium: 0
  low: 2
input-hash: "[live-state]"
inputs:
  - stories/STORY-INDEX.md (post-be3703f, v1.2)
  - stories/dependency-graph.md (post-be3703f, v1.2)
  - stories/wave-schedule.md (post-be3703f, v1.2)
  - stories/sprint-state.yaml (post-be3703f, v1.1)
  - stories/holdout-scenarios.md (post-be3703f, v1.1)
  - stories/S-001-cargo-workspace-ci-setup.md (v1.2)
  - stories/S-002-healthz-endpoint.md
  - stories/S-003-status-endpoint.md
  - stories/S-004-body-size-limit.md
  - stories/S-005-graceful-shutdown.md (v1.1)
  - stories/S-006-lock-file-lifecycle.md (v1.2)
  - stories/S-007-crash-recovery-checkpoint.md
  - stories/S-008-jsonl-ring-format-version.md
  - stories/S-009-auth-token-header-validation.md (v1.2)
  - stories/S-010-monocle-core-abi-version.md
  - stories/S-011-non-exhaustive-enum-policy.md
  - stories/S-012-factory-adapter-trait.md
  - stories/S-013-hook-envelope-proto-wire-format.md
  - stories/S-014-engine-module-trait.md
  - stories/S-015-claude-code-module-impl.md (v1.2)
  - stories/S-DTU-001-claude-code-hook-clone.md
  - stories/S-PHASE-3-PREP-spec-kit-mcp-integration.md
  - stories/epics/E-01-daemon-lifecycle.md
  - stories/epics/E-02-core-types-and-abi.md
  - stories/epics/E-03-engine-module.md
  - stories/epics/E-DTU-hook-protocol-clone.md
  - stories/epics/E-PREP-phase3-prep.md
traces_to: "Phase 2 story corpus post-r02-remediation at commit be3703f"
timestamp: 2026-05-19T09:00:00Z
---

# Consistency Pass: Phase 2 Story Corpus — Round 03

> **Scope:** Re-validation of all 20 checks from r01/r02 against the r02-remediated story
> corpus at commit `be3703f`. Plus verification of Orchestrator Decision 3 propagation.
> Read-only audit. No artifacts modified.

## Executive Summary

| Status | GAPS |
|--------|------|
| Checks run | All 20 check categories (checks 1-20) + Decision 3 propagation sweep |
| r02 gaps closed | 4 of 4 (100%) |
| r02 gaps still open | 0 |
| New gaps (r03) | 2 |
| Total new gaps (r03) | 2 |
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 2 |
| Gate recommendation | CONDITIONAL PASS — both r03 gaps are LOW-severity frontmatter version-pin drift with no behavioral or implementer-confusion impact. Gate-blocking: neither gap is blocking. Story corpus is ready for Phase 3 TDD dispatch after trivial 2-line fixup (or waived by human). |

---

## r02 Gap Closure Verification

Independent re-derivation of each r02 gap. Evidence column cites current artifact state at `be3703f`.

| Gap ID | Severity | Description | Status | Evidence |
|--------|----------|-------------|--------|----------|
| GAP-PHASE2-R02-1 | HIGH | STORY-INDEX Blocks column: S-005 still shows S-007; S-006 still shows S-009 | CLOSED | `STORY-INDEX.md:47` — `\| S-005 \| Graceful Shutdown \| EPIC-01 \| 5 \| 2 \| draft \| — \|`; `STORY-INDEX.md:48` — `\| S-006 \| Lock File Atomic Lifecycle \| EPIC-01 \| 8 \| 2 \| draft \| S-007, S-008 \|`. Both corrections applied. |
| GAP-PHASE2-R02-2 | MEDIUM | wave-schedule.md Wave 3 says "all 4 stories" (should be 5) | CLOSED | `wave-schedule.md:124` — "All 5 stories are listed (S-009 may start only after S-008 completes within the wave due to Decision 1 S-008→S-009 dependency; S-007, S-012, and S-015 are fully independent and can run concurrently with each other and with S-008)." Count corrected and within-wave dep noted. |
| GAP-PHASE2-R02-3 | MEDIUM | S-009 File Structure uses `generate_auth_token()` vs `generate_session_token()` | CLOSED | `S-009:182` — "Do NOT add a `generate_auth_token()` function — the canonical name is `generate_session_token()`". The former conflation is gone; generate_auth_token() appears only in prohibitive context. |
| GAP-PHASE2-R02-4 | LOW | holdout-scenarios.md missing `level:` and `version:` frontmatter fields | CLOSED | `holdout-scenarios.md:3` — `level: ops`; `holdout-scenarios.md:4` — `version: "1.1"`. Both fields now present. |

**r02 closure rate: 4/4 (100%).**

---

## Orchestrator Decision 3 Propagation Sweep

Scope: verify `monocle-auth` crate does not appear anywhere as a positive declaration in the 22
corpus files; verify `monocle_runtime::auth::generate_session_token()` qualified path is used
consistently; verify S-001 declares exactly 3 Phase 1 workspace members.

| Sub-check | Verification | Result |
|-----------|-------------|--------|
| `monocle-auth` as workspace member (S-001) | `S-001:69` — "monocle-runtime, monocle-proto. monocle-auth is NOT a separate workspace crate". AC-005 lists 3 crates: monocle-core, monocle-runtime, monocle-proto. | PASS |
| `monocle-auth` as crate dependency anywhere | Corpus-wide grep: 8 hits, all in negation context ("NOT a separate crate", "MUST NOT appear", "NOT in a separate monocle-auth crate"). Zero positive declarations. | PASS |
| Qualified path `monocle_runtime::auth::generate_session_token()` | All positive function references use the canonical qualified path: S-006:109,115,141; S-009:52,117,144,183. No bare `generate_session_token()` without module prefix found in ACs or Tasks. | PASS |
| `generate_auth_token()` eliminated | `S-009:182` — only appears as a prohibition ("Do NOT add a generate_auth_token() function"). Zero positive usages corpus-wide. | PASS |
| S-001 Phase 1 workspace member count = 3 | `S-001 AC-005` — "exactly these 3 crates as Phase 1 members: monocle-core, monocle-runtime, monocle-proto". | PASS |

**Decision 3 propagation: FULLY CONSISTENT across all 22 corpus files.**

---

## Checks Passed — Full Re-verification (Checks 1-20)

All checks that passed in r02 still pass after r02 remediation. Active re-verification
performed on all checks touched by the r02 remediation burst.

| Check | Description | Result |
|-------|-------------|--------|
| 1 | Version pin freshness: all spec versions current | PASS — no version pin changes in corpus |
| 2 | BC ID validity: all 22 BC-S.SS.NNN in stories exist in BC-INDEX v1.11 | PASS |
| 3 | VP ID validity: all 22 VP-NNN in stories exist in VP-INDEX v1.16 | PASS |
| 4 | Error code validity: all 15 E-NNN exist in error-taxonomy v1.5 | PASS |
| 5 | NFR validity: all 12 P0 NFRs exist in nfr-catalog v1.7 | PASS |
| 6 | Frontmatter BC coverage coherence — S-015 has BC-2.03.001 | PASS — S-015:18 `behavioral_contracts: [BC-2.03.001, BC-2.03.002, BC-2.03.003, BC-2.03.004]` |
| 6 | Frontmatter BC coverage coherence — S-003 has BC-2.02.001 | PASS — S-003:18 `behavioral_contracts: [BC-2.01.002, BC-2.02.001]` |
| 7 | Story count: STORY-INDEX 17, dependency-graph 17, sprint-state 17 | PASS |
| 8 | Story ID uniqueness; filename slugs | PASS |
| 9 | STORY-INDEX Blocks column integrity | PASS — S-005 Blocks="—", S-006 Blocks="S-007, S-008"; dep-graph Blocks Edges table consistent |
| 10 | STORY-INDEX wave column vs dep-graph vs story frontmatter | PASS — S-009 wave: 3 in all three locations |
| 11 | Wave point totals: Wave 2=41, Wave 3=34 | PASS — STORY-INDEX Wave Summary, wave-schedule header, sprint-state.yaml wave_2_points/wave_3_points all agree |
| 12 | sprint-state.yaml: 17 stories, 16 not_started, 1 blocked | PASS |
| 13 | Holdout non-leakage: 12 scenarios, no implementer-visible leakage | PASS |
| 14 | Epic membership: all 5 epics, all 17 stories | PASS |
| 15 | BC/VP/NFR/error coverage rollups | PASS — 22/22/12/15 (STORY-INDEX Coverage Tables) |
| 16 | Production-grade language: no TBD/placeholder in corpus | PASS |
| 17 | S-PHASE-3-PREP integrity | PASS |
| 18 | Wave-restructure consistency | PASS — Wave 3 paragraph corrected to "all 5 stories"; S-008→S-009 within-wave dep documented |
| 19 | Auth token mechanism consistency | PASS — generate_session_token() throughout; no generate_auth_token() in positive context |
| 20 | Frontmatter retrofit completeness (all plan docs) | PARTIAL PASS — level/version/inputs/input-hash/traces_to all present in all files; see GAP-PHASE2-R03-1 and GAP-PHASE2-R03-2 for stale version numbers |

---

## New Gaps Found (r03)

### GAP-PHASE2-R03-1 — LOW
**Check:** #20 (Frontmatter retrofit completeness — stale traces_to version pin)
**Title:** `sprint-state.yaml` `traces_to_full` references STORY-INDEX.md v1.1; current version is v1.2

**Evidence:**
- `sprint-state.yaml:21` — `traces_to_full: ".factory/stories/STORY-INDEX.md v1.1"`
- `STORY-INDEX.md:4` — `version: "1.2"` (bumped during r01/r02 remediation)
- `sprint-state.yaml:3` — `version: "1.1"` (not bumped; no content changes to sprint-state in r02)

**Impact:** Documentation-only. The `traces_to_full` field is a prose reference, not a machine-read pointer. An implementer reading sprint-state.yaml would see the stale version number but this causes no implementer confusion since story IDs, wave assignments, and points are all correct in the sprint-state body. Sprint-state.yaml was not given a minor bump because no story-state changes occurred in r02 (stories remain `not_started`).

**Proposed routing:** `vsdd-factory:story-writer`
- `sprint-state.yaml:21` — change `traces_to_full: ".factory/stories/STORY-INDEX.md v1.1"` to `traces_to_full: ".factory/stories/STORY-INDEX.md v1.2"`
- Consider bumping `sprint-state.yaml` version to "1.2" for symmetry with the other plan docs.

---

### GAP-PHASE2-R03-2 — LOW
**Check:** #20 (Frontmatter retrofit completeness — stale traces_to version pin)
**Title:** `holdout-scenarios.md` `traces_to` references STORY-INDEX.md v1.1; current version is v1.2

**Evidence:**
- `holdout-scenarios.md:18` — `traces_to: ".factory/stories/STORY-INDEX.md v1.1"`
- `STORY-INDEX.md:4` — `version: "1.2"` (bumped during r01/r02 remediation)
- `holdout-scenarios.md:4` — `version: "1.1"` (not bumped; holdout content unchanged)

**Impact:** Documentation-only. Holdout-scenarios.md content (12 scenarios) was not changed in r02 remediation, which is correct (holdout content must not be exposed to implementers). The stale version pin in the `traces_to` field is the only inconsistency. Not implementer-facing (holdout-evaluator-only document).

**Proposed routing:** `vsdd-factory:story-writer`
- `holdout-scenarios.md:18` — change `traces_to: ".factory/stories/STORY-INDEX.md v1.1"` to `traces_to: ".factory/stories/STORY-INDEX.md v1.2"`

---

## Routing Summary

| Gap ID | Severity | Description | Proposed Routing | Estimated Effort |
|--------|----------|-------------|-----------------|-----------------|
| GAP-PHASE2-R03-1 | LOW | sprint-state.yaml traces_to_full references STORY-INDEX v1.1 (current v1.2) | vsdd-factory:story-writer | Trivial — 1 field edit |
| GAP-PHASE2-R03-2 | LOW | holdout-scenarios.md traces_to references STORY-INDEX v1.1 (current v1.2) | vsdd-factory:story-writer | Trivial — 1 field edit |

---

## Coverage Integrity — Unchanged Since r02

The following coverage claims were re-verified by checking that no r02 remediation added or
removed BC/VP/NFR/error code assignments:

- **BC coverage: 22/22 — CONFIRMED.** STORY-INDEX BC Coverage Table shows S-014 + S-015 for BC-2.03.001 (updated in r02 per F-PHASE2-R02-04). Dep-graph BC Clause Coverage Matrix row at line 302 correctly maps BC-2.03.001 postcondition 5 (DI-006) → AC-010 → S-015.
- **VP coverage: 22/22 — CONFIRMED.** VP Coverage Table unchanged.
- **Error code coverage: 15/15 — CONFIRMED.**
- **NFR coverage: 12/12 — CONFIRMED.** 4 deferred to Phase 3 per nfr-catalog.md remain justified with non-empty Gap Register entries.
- **DAG acyclicity — CONFIRMED.** Kahn trace: 17 nodes, ACYCLIC. No changes to dependency edges in r02.
- **Holdout scenarios — 12 scenarios, no leakage — CONFIRMED.** Holdout content not modified in r02.
- **BC Clause Coverage Matrix — CONFIRMED.** GAP-P2-005 (BC-2.01.004 PC-6, --persistent-events Phase 3 scope) is the only L1 gap; it has non-empty justification and a future-story attachment (Phase 3 story decomposition). No new L1 gaps.
- **Epic membership — CONFIRMED.** EPIC-01 table (E-01-daemon-lifecycle.md) shows correct wave assignments: S-007/S-008/S-009 all Wave 3, S-001..S-006 correctly in Wave 1/2. No stale block-dependency columns in epic files.

---

## §Trace v1.0

Consistency pass r03 created 2026-05-19T09:00:00Z by `consistency-validator`.
Inputs: Phase 2 story corpus at commit `be3703f` (r02 remediation burst).
r02 closure rate: 4/4 (100%). Zero r01/r02 gaps remain open.
2 new LOW-severity gaps found: stale STORY-INDEX version pins in sprint-state.yaml and holdout-scenarios.md.
No behavioral coverage gaps. No BC/VP/NFR/error code validity failures. No dependency graph errors.
Decision 3 propagation (monocle-auth dropped; generate_session_token() in monocle-runtime): FULLY CONSISTENT across all 22 corpus files.
