---
document_type: consistency-pass
level: ops
phase: phase-2
round: r04
producer: consistency-validator
status: CONDITIONAL-PASS
gaps_total: 2
gaps_by_severity:
  critical: 0
  high: 0
  medium: 0
  low: 2
input-hash: "[live-state]"
inputs:
  - stories/STORY-INDEX.md (v1.2)
  - stories/dependency-graph.md (v1.3)
  - stories/wave-schedule.md (v1.2)
  - stories/sprint-state.yaml (v1.1)
  - stories/holdout-scenarios.md (v1.1)
  - stories/S-001-cargo-workspace-ci-setup.md (v1.3)
  - stories/S-002-healthz-endpoint.md (v1.0)
  - stories/S-003-status-endpoint.md (v1.2)
  - stories/S-004-body-size-limit.md
  - stories/S-005-graceful-shutdown.md (v1.2)
  - stories/S-006-lock-file-lifecycle.md
  - stories/S-007-crash-recovery-checkpoint.md
  - stories/S-008-jsonl-ring-format-version.md (v1.2)
  - stories/S-009-auth-token-header-validation.md (v1.3)
  - stories/S-010-monocle-core-abi-version.md (v1.1)
  - stories/S-011-non-exhaustive-enum-policy.md (v1.1)
  - stories/S-012-factory-adapter-trait.md (v1.3)
  - stories/S-013-hook-envelope-proto-wire-format.md
  - stories/S-014-engine-module-trait.md
  - stories/S-015-claude-code-module-impl.md (v1.3)
  - stories/S-DTU-001-claude-code-hook-clone.md
  - stories/S-PHASE-3-PREP-spec-kit-mcp-integration.md
  - stories/epics/E-01-daemon-lifecycle.md
  - stories/epics/E-02-core-types-and-abi.md
  - stories/epics/E-03-engine-module.md
  - stories/epics/E-DTU-hook-protocol-clone.md
  - stories/epics/E-PREP-phase3-prep.md
traces_to: "Phase 2 story corpus post-r03-remediation at commit 37c234e"
timestamp: 2026-05-19T10:00:00Z
---

# Consistency Pass: Phase 2 Story Corpus — Round 04

> **Scope:** Full re-validation of all 20 check categories (checks 1–20) + Decision 3 + Decision 4
> propagation, r03 gap closure verification, and new sibling-sweep / inverse-coverage checks
> against the r03-remediated story corpus at commit `37c234e`. Read-only audit. No artifacts modified.

## Executive Summary

| Status | CONDITIONAL-PASS |
|--------|------|
| Checks run | All 20 check categories + Decision 3 + Decision 4 propagation + sibling-sweep + inverse-coverage |
| r03 gaps closed | 2 of 2 (100%) |
| r03 gaps still open | 0 |
| New gaps (r04) | 2 |
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 2 |
| Gate recommendation | CONDITIONAL PASS — both r04 gaps are LOW-severity BC Clause Coverage Matrix invariant-row omissions with no behavioral or implementer-confusion impact. Story corpus is ready for Phase 3 TDD dispatch. r04 gaps are non-blocking. |

---

## r03 Gap Closure Verification

Independent re-derivation of each r03 gap at commit `37c234e`.

| Gap ID | Severity | Description | Status | Evidence |
|--------|----------|-------------|--------|----------|
| GAP-PHASE2-R03-1 | LOW | `sprint-state.yaml:21` `traces_to_full` references STORY-INDEX v1.1; current version is v1.2 | CLOSED | `sprint-state.yaml:21` — `traces_to_full: ".factory/stories/STORY-INDEX.md v1.2"`. Version pin updated. |
| GAP-PHASE2-R03-2 | LOW | `holdout-scenarios.md:18` `traces_to` references STORY-INDEX v1.1; current version is v1.2 | CLOSED | `holdout-scenarios.md:18` — `traces_to: ".factory/stories/STORY-INDEX.md v1.2"`. Version pin updated. |

**r03 closure rate: 2/2 (100%).**

---

## Decision 4 Propagation Verification

Scope: verify `S-011` correctly incorporates all 9 canonical enums including the 3 permissions enums from `SS-permissions-phase1.md`, verify `permissions.rs` module is declared, and verify no `monocle-auth` positive references exist.

| Sub-check | Verification | Result |
|-----------|-------------|--------|
| S-011 lists 9 canonical enums | `S-011:49–55` — lists `HookType`, `HookEvent`, `HookDecision`, `DeferUntil`, `BlockingSeverity`, `SessionStatus` (6 core) + `DenyReason`, `AllowPattern`, `DenyPattern` (3 permissions) = 9 total. | PASS |
| `permissions.rs` module declared in AC-001b | `S-011:57–60` — AC-001b declares `monocle-core/src/permissions.rs` module with the 3 enums per `SS-permissions-phase1.md §Permission Types lines 162–203`. | PASS |
| SS-permissions-phase1.md pinned in inputs | `S-011:30` — `{path: .factory/specs/architecture/SS-permissions-phase1.md, version: "1.5.2"}` in `inputs:` array. | PASS |
| `monocle-auth` absent as positive declaration | Corpus-wide grep across all 22 story corpus files: zero positive declarations found. All hits are in negation context (`MUST NOT appear`, `NOT a separate crate`, `forbidden`, `Decision 3`). | PASS |
| `monocle_runtime::auth::generate_session_token()` canonical | Positive function references in S-006, S-009 all use the qualified path with `monocle_runtime::auth::` prefix or note it explicitly. | PASS |
| dep-graph BC-2.02.003 PC-4 row present | `dependency-graph.md:280` — `BC-2.02.003 | 4 | postcondition (canonical minimum 9 enums including DenyReason, AllowPattern, DenyPattern) | AC-001, AC-001b | S-011`. Row added per F-PHASE2-R03-06. | PASS |

**Decision 4 propagation: FULLY CONSISTENT.**

---

## Checks Passed — Full Re-verification (Checks 1-20)

All checks re-verified at commit `37c234e`. Active re-verification performed on all checks
touched by the r03 remediation burst.

| Check | Description | Result |
|-------|-------------|--------|
| 1 | Version pin freshness: all spec versions current | PASS — no spec version changes in corpus; all input pins match canonical artifact versions (BC-INDEX v1.11, VP-INDEX v1.16, PRD v1.26.15, ARCH-INDEX v1.0.10) |
| 2 | BC ID validity: all 22 BC-S.SS.NNN in stories exist in BC-INDEX v1.11 | PASS |
| 3 | VP ID validity: all 22 VP-NNN in stories exist in VP-INDEX v1.16 | PASS |
| 4 | Error code validity: all 15 E-NNN exist in error-taxonomy v1.5 | PASS |
| 5 | NFR validity: all 12 P0 NFRs exist in nfr-catalog v1.7 | PASS |
| 6 | Frontmatter BC coverage coherence | PASS — S-015 has `[BC-2.03.001, BC-2.03.002, BC-2.03.003, BC-2.03.004]`; S-003 has `[BC-2.01.002, BC-2.02.001]`; S-009 has `[BC-2.01.008, BC-2.01.009]`; all frontmatter BC arrays cross-checked against body ACs |
| 7 | Story count: STORY-INDEX 17, dependency-graph 17, sprint-state 17 | PASS — all three agree; sprint-state `total_stories: 17`, `not_started: 16`, `blocked: 1` |
| 8 | Story ID uniqueness; filename slugs | PASS |
| 9 | STORY-INDEX Blocks column integrity | PASS — S-005 Blocks="—", S-006 Blocks="S-007, S-008", S-008 Blocks="S-009"; consistent with dep-graph Blocks Edges table |
| 10 | STORY-INDEX wave column vs dep-graph vs story frontmatter | PASS — S-009 wave: 3 in all three; S-012 wave: 3 in all three; S-015 wave: 3 in all three |
| 11 | Wave point totals: Wave 2=41, Wave 3=34 | PASS — STORY-INDEX Wave Summary, wave-schedule header, sprint-state `wave_2_points: 41`, `wave_3_points: 34` all agree |
| 12 | sprint-state.yaml: 17 stories, 16 not_started, 1 blocked | PASS — `total_stories: 17`, `not_started: 16`, `blocked: 1` (S-PHASE-3-PREP blocked on spec-kit-mcp rc.19+) |
| 13 | Holdout non-leakage: 12 scenarios, no implementer-visible leakage | PASS — holdout-scenarios.md version 1.1, `visibility: holdout-evaluator-only` in frontmatter |
| 14 | Epic membership: all 5 epics, all 17 stories | PASS — EPIC-01 (9 product stories), EPIC-02 (4), EPIC-03 (2), EPIC-DTU (1), EPIC-PREP (1) = 17 |
| 15 | BC/VP/NFR/error coverage rollups | PASS — 22/22 BCs, 22/22 VPs, 12/12 NFRs, 15/15 error codes (STORY-INDEX Coverage Tables) |
| 16 | Production-grade language: no TBD/placeholder in corpus | PASS — todo!() stubs explicitly declared as intentional Phase 1 stubs with binding signatures (AC-008/AC-009 in S-015); no unresolved TBDs |
| 17 | S-PHASE-3-PREP integrity | PASS |
| 18 | Wave-restructure consistency | PASS — Wave 3 paragraph: "All 5 stories are listed (S-009 may start only after S-008 completes within the wave due to Decision 1 S-008→S-009 dependency; S-007, S-012, and S-015 are fully independent...)" |
| 19 | Auth token mechanism consistency | PASS — `generate_session_token()` throughout; no `generate_auth_token()` in positive context; `monocle_runtime::auth` qualified path used in all callsites |
| 20 | Frontmatter retrofit completeness (all plan docs) | PASS — all plan docs have `level:`, `version:`, `inputs:`, `input-hash:`, `traces_to:`; GAP-PHASE2-R03-1 and GAP-PHASE2-R03-2 are CLOSED at this commit |

---

## Sibling-Sweep Verification: Architecture Compliance BC Citations

Scope: verify all Architecture Compliance sections across stories cite BC clauses that exist
and match the AC content in the same story.

| Story | Architecture Compliance Citation | Verification | Result |
|-------|--------------------------------|-------------|--------|
| S-003 | `chrono::Utc::now().format(...)` + `monocle_core::MONOCLE_ABI_VERSION` + auth `constant_time_eq` | Citations match ACs: ISO 8601 → AC-007; ABI const → AC-005; constant-time → AC-002/AC-004 | PASS |
| S-005 | `BC-2.01.004 PC-8 + INV-1 + INV-4` in arch-compliance (line 135); exit-code taxonomy vs drain-timeout clarified | After r03 fix: drain-timeout → exit 0 (in-process abort, SIGTERM originator); exit 2 = second POST /shutdown during drain only. Matches AC-004 and BC-2.01.004 PC-8 verbatim. | PASS |
| S-008 | `format_version MUST be first key` + `Async flush — NEVER synchronous` + tee invariant | Citations match AC-001 (format_version first), AC-004 (DI-001 tee), AC-003 (hybrid-RAM flush). No BC clause numbering error. | PASS |
| S-009 | `OsRng MANDATORY` + `constant_time_eq MANDATORY on BOTH paths` + WARN log on alias | Citations match AC-001 (OsRng), AC-008 (constant-time both paths), AC-005 (WARN on alias). All exist in BC-2.01.008/BC-2.01.009. | PASS |
| S-011 | `#[non_exhaustive] MANDATORY on all public monocle-core enums except ADR-0004 exemptions` | Citations match AC-001/AC-001b (9 enums) and AC-002 (Phase1Permission, ClaudeCodeTool exempt). | PASS |
| S-012 | `7 methods exactly — any addition is a BREAKING change` + `NO Sealed supertrait` | Citations match AC-001 (7 methods) and AC-002 (no Sealed). BC-2.02.004 PC-1/PC-2 confirmed. | PASS |
| S-015 | `detect() is I/O-free — DI-006 (BC-2.03.001 postcondition 5)` | After r03 fix: cites postcondition 5 (not invariant 2). Matches AC-010. PASS. | PASS |

**Sibling-sweep: ALL PASS. No stale or incorrect BC clause citations in Architecture Compliance sections.**

---

## Inverse-Coverage Verification: dep-graph BC Clause Coverage Matrix

Scope: verify all rows in the dep-graph BC Clause Coverage Matrix cite AC headings that exist
in the referenced story.

Spot-check on r03-remediated rows:

| dep-graph Row | Claimed AC | Story | Verification |
|---------------|-----------|-------|-------------|
| `BC-2.01.002 | 1 sub-bullet «abi_version» | postcondition | AC-005 | S-003` | AC-005 exists in S-003 | `S-003:69` — `### AC-005 (traces to BC-2.02.001 postcondition 1 — ABI version field; BC-2.01.002 postcondition 1 sub-bullet «abi_version»)` | PASS |
| `BC-2.01.002 | 3 | postcondition (/status during drain) | AC-008 | S-003` | AC-008 exists in S-003 | `S-003:85` — `### AC-008 (traces to BC-2.01.002 postcondition 3 — /status serves during drain)` | PASS |
| `BC-2.01.007 | 3 | postcondition (RING_FORMAT_VERSION const) | AC-006 | S-008` | AC-006 exists in S-008 | `S-008:85` — `### AC-006 (traces to BC-2.01.007 postcondition 5 — #[non_exhaustive] + pub fn new() constructor)` | NOTE: dep-graph row says "postcondition 3" but AC-006 actually traces to "postcondition 5". See GAP-PHASE2-R04-1 below. |
| `BC-2.01.008 | 4 | postcondition (dual-accept on hook endpoints) | AC-010a | S-009` | AC-010a exists in S-009 | `S-009:94` — `### AC-010a (traces to BC-2.01.008 postcondition 4 — dual-accept auth applies to hook endpoints)` | PASS |
| `BC-2.02.001 | 2 | postcondition (equals compiled value) | AC-005 | S-010` | AC-005 exists in S-010 | `S-010:67` — `### AC-005 (traces to BC-2.02.001 postcondition 1 + postcondition 2 — const value is 1 in Phase 1; equals compiled value)` | PASS |
| `BC-2.02.005 | 2 | invariant (display_name returns "VSDD Factory") | AC-010 | S-012` | AC-010 exists in S-012 | `S-012:105` — `### AC-010 (traces to BC-2.02.005 invariant 2 — display_name() returns "VSDD Factory")` | PASS |
| `BC-2.03.001 | 5 | postcondition (DI-006 detect I/O-free) | AC-010 | S-015` | AC-010 exists in S-015 | `S-015:111` — `### AC-010 (traces to BC-2.03.001 postcondition 5 + EC-031 — detect() is I/O-free; on_hook() fail-open)` | PASS |

---

## New Gaps Found (r04)

### GAP-PHASE2-R04-1 — LOW
**Check:** Inverse-coverage (dep-graph BC Clause Coverage Matrix row description mismatch)
**Title:** `dependency-graph.md` BC-2.01.007 row for AC-006 labels clause as "postcondition 3" but AC-006 traces to BC-2.01.007 postcondition 5

**Evidence:**

- `dependency-graph.md` line 246: `| BC-2.01.007 | 3 | postcondition (RING_FORMAT_VERSION const) | AC-006 (HookEventRecord::new sets const) | S-008 |`
- `S-008:85`: `### AC-006 (traces to BC-2.01.007 postcondition 5 — #[non_exhaustive] + pub fn new() constructor)`
- `dependency-graph.md` line 248: `| BC-2.01.007 | 5 | postcondition (#[non_exhaustive] + new()) | AC-006 | S-008 |`

**Analysis:** The dep-graph matrix has TWO rows for AC-006: one at clause "3" (description: "RING_FORMAT_VERSION const") and one at clause "5" (description: "#[non_exhaustive] + new()"). The clause-3 row carries the description that belongs to PC-3 but maps to AC-006 which is a PC-5 trace. The clause-5 row correctly maps AC-006 → BC-2.01.007 PC-5.

BC-2.01.007 PC-3 text: "The `RING_FORMAT_VERSION` constant is used by `HookEventRecord::new()` to set the `format_version` field." The RING_FORMAT_VERSION const is indeed set inside `new()` which is the AC-006 subject. But the AC-006 heading traces to postcondition 5 (the full `#[non_exhaustive] + new()` clause), not postcondition 3 alone.

The clause-3 row should either cite a distinct AC that covers PC-3 independently, or the row description should note that PC-3 is sub-covered by AC-006 (same AC as PC-5 since `new()` sets the const). The current state creates a phantom "clause 3 → AC-006" mapping that conflicts with the AC-006 heading which traces to PC-5.

**Impact:** Documentation-only. An implementer reading S-008 will implement `HookEventRecord::new()` with `RING_FORMAT_VERSION` set internally (the substance of both PC-3 and PC-5 intent). The confusion is between the dep-graph row label and the AC heading — not between the implementation spec and the BC requirements. BC-2.01.007 PC-3 behavior (constant used by new()) is fully subsumed in PC-5 (new() constructor with the const).

**Proposed routing:** `vsdd-factory:story-writer`
- `dependency-graph.md` line 246 — Change clause "3" to "3 (sub-covered by AC-006 via PC-5 — RING_FORMAT_VERSION const set inside new() constructor)" or remove duplicate and add a note that PC-3 is sub-covered by PC-5/AC-006.

---

### GAP-PHASE2-R04-2 — LOW
**Check:** #7 (BC Clause Coverage Matrix — invariant row completeness)
**Title:** `dependency-graph.md` BC Clause Coverage Matrix has no explicit rows for BC-2.01.009 invariants 1–6; only invariant 7 has a row

**Evidence:**

- `dependency-graph.md:261`: `| BC-2.01.009 | 7 | invariant | AC-008 | S-009 |` — the only BC-2.01.009 invariant row
- `BC-2.01.009.md` §Invariants: 7 invariants defined (INV-1 through INV-7)
- `S-009` has ACs that cover INV-1 (AC-004/AC-009 cover missing-header taxonomy), INV-5 (AC-007 covers canonical-wins), INV-6 (AC-005 covers WARN log per alias attempt), INV-7 (AC-008 covers constant-time, mapped explicitly)
- INV-2 (value-present failures return same body), INV-3 (missing vs invalid distinction purpose), INV-4 (AuthError enum implementation) have no explicit dep-graph rows and are addressed only implicitly via the AC-004/AC-005/AC-006 postcondition coverage

**Analysis:** BC-2.01.009 INV-1 through INV-6 are policy and taxonomy invariants (two-body error surface, same-body response, missing-vs-invalid distinction, internal AuthError enum, canonical priority, WARN log once per alias request). All are semantically covered by the existing postcondition ACs — there are no behavioral outcomes from INV-1..6 that are NOT already tested via AC-004 (PC-1 missing), AC-005 (PC-2/PC-3 alias), AC-006 (PC-3 canonical), AC-007 (PC-4 canonical-wins), AC-008 (INV-7 constant-time explicitly). However, the dep-graph matrix convention for this BC leaves INV-1..6 undocumented, unlike the treatment of other BCs where all invariants are mapped (e.g., BC-2.01.001 both invariants mapped, BC-2.01.003 its one invariant mapped, BC-2.01.005 all three invariants mapped).

The omission is a matrix completeness concern, not a coverage gap. The story corpus test coverage is complete. The dep-graph invariant matrix is stylistically inconsistent for this BC.

**Impact:** Documentation-only. An implementer working from S-009 will implement all required auth behavior from the clearly-mapped postcondition ACs. No implementer confusion results from the missing invariant rows. The holdout evaluator works from BC-2.01.009 directly, not from the dep-graph matrix.

**Proposed routing:** `vsdd-factory:story-writer` (if waived is not chosen)
- `dependency-graph.md` — add rows for BC-2.01.009 INV-1..INV-6 noting which existing ACs (AC-004 through AC-008) provide implicit coverage, or add a note that INV-1..6 are taxonomy/policy declarations subsumed by the PC coverage above.

---

## Version Bump Rule Verification

The r03 burst changed 10 story files. Version bump compliance per the codified rule
("stories with AC/depends_on/blocks/behavioral_contracts changes → minor bump (+0.1)"):

| Story | Change Type | Bump Applied | Rule Compliant? |
|-------|------------|-------------|----------------|
| S-001 | AC-006 + Tasks: temp-env dev-dep added to inputs + tasks | v1.2 → v1.3 | YES — inputs and tasks change (temp-env) |
| S-002 | Tasks only: VP-001 probe text "GET /status" → "GET /healthz" | v1.0 (unchanged) | YES — Tasks-only fix; no AC/depends_on/blocks/bc change per bump rule |
| S-003 | AC changes: AC-002..AC-008 re-anchored; AC-008 new | v1.1 → v1.2 | YES |
| S-005 | Architecture Compliance text fix only | v1.1 → v1.2 | MARGINAL — arch-compliance-section is normative (implementer reads it); bump is defensible even though the bump rule specifically names AC/depends_on/blocks/bc. Acceptable. |
| S-008 | AC-003/AC-005/AC-007 re-anchored | v1.1 → v1.2 | YES |
| S-009 | AC-010 split into AC-010a + AC-010b | v1.2 → v1.3 | YES |
| S-010 | AC-005 re-anchored | v1.0 → v1.1 | YES |
| S-011 | AC-001 + AC-001b added (Decision 4 permissions enums); inputs updated | v1.0 → v1.1 | YES |
| S-012 | AC-010 new (BC-2.02.005 INV-2 display_name) | v1.2 → v1.3 | YES |
| S-015 | Architecture Compliance + traces_to DI-006 locus fix | v1.2 → v1.3 | MARGINAL — same as S-005; defensible |

**Version bump compliance: PASS. All bumps are justified. S-002 correctly NOT bumped (Tasks-only change).**

---

## Decision 3 Propagation — Re-confirmed

All five Decision 3 sub-checks confirmed at `37c234e` state (unchanged since r03 verification):
- `monocle-auth` as workspace member: ABSENT (S-001 AC-005 confirms 3-crate workspace)
- `monocle-auth` as crate dependency: ABSENT across corpus (zero positive declarations)
- `monocle_runtime::auth::generate_session_token()` qualified path: PRESENT in S-006, S-009
- `generate_auth_token()`: ABSENT in positive context (S-009 mentions only as prohibition)
- S-001 Phase 1 workspace member count = 3 (monocle-core, monocle-runtime, monocle-proto)

---

## Coverage Integrity — Unchanged Since r03

- **BC coverage: 22/22 — CONFIRMED.**
- **VP coverage: 22/22 — CONFIRMED.**
- **Error code coverage: 15/15 — CONFIRMED.**
- **NFR coverage: 12/12 — CONFIRMED.** 4 deferred to Phase 3 per Gap Register (GAP-P2-001..004 with non-empty justification).
- **DAG acyclicity — CONFIRMED.** 17 nodes, ACYCLIC. No edge changes in r03.
- **Holdout scenarios — 12 scenarios, no leakage — CONFIRMED.** HS-W2-002 renamed to HS-W3-006 per F-PHASE2-R03-10 (S-009 Wave 3 move). Wave Coverage Summary updated. Holdout content unchanged.
- **BC Clause Coverage Matrix — CONFIRMED.** GAP-P2-005 (BC-2.01.004 PC-6, --persistent-events Phase 3 scope) remains the only L1 gap; non-empty justification, future-story attachment present.
- **Epic membership — CONFIRMED.** EPIC-01 table correct: S-009 Wave 3 with S-001/S-004/S-006/S-008 depends-on.

---

## Routing Summary

| Gap ID | Severity | Description | Proposed Routing | Estimated Effort |
|--------|----------|-------------|-----------------|-----------------|
| GAP-PHASE2-R04-1 | LOW | dep-graph BC-2.01.007 clause-3 row cites AC-006 which traces to PC-5; dual-row ambiguity | vsdd-factory:story-writer | Trivial — 1 row clarification |
| GAP-PHASE2-R04-2 | LOW | dep-graph BC-2.01.009 INV-1..6 absent from clause coverage matrix | vsdd-factory:story-writer | Minor — add 6 coverage rows with implicit-coverage notes |

---

## §Trace v1.0

Consistency pass r04 created 2026-05-19T10:00:00Z by `consistency-validator`.
Inputs: Phase 2 story corpus at commit `37c234e` (r03 remediation burst).
r03 closure rate: 2/2 (100%). Zero r01/r02/r03 gaps remain open.
2 new LOW-severity gaps found: (1) dep-graph BC-2.01.007 clause-3 row label ambiguity; (2) BC-2.01.009 INV-1..6 absent from clause coverage matrix.
No behavioral coverage gaps. No BC/VP/NFR/error code validity failures. No dependency graph errors.
Decision 3 (monocle-auth dropped; generate_session_token() in monocle-runtime): FULLY CONSISTENT.
Decision 4 (S-011 9 enums + permissions.rs + SS-permissions-phase1 input pin): FULLY CONSISTENT.
Sibling-sweep (Architecture Compliance BC citations): ALL PASS.
Inverse-coverage (dep-graph AC cross-check): PASS with 1 ambiguity noted (GAP-PHASE2-R04-1).
