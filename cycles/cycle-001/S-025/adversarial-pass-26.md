---
title: S-025 Adversarial Pass 26
pass_number: 26
counter_before: 0/3
counter_after: 0/3 (HOLD — 1 HIGH ADR-0007 self-inconsistency + 1 MED META-pattern 8th instance + 1 LOW POL-11 omission; all 3 closed in 2-agent fix round)
verdict: MED
head_sha_reviewed: 0813d4f (worktree) + 89ee097 (factory-artifacts)
created: 2026-05-29
---

## Summary

Pass 26 is the empirical test of ADR-0007's effectiveness. **Result: ADR-0007 strategic intervention works correctly for its target species (literal version-pin staleness in instances 18, 23, 24, 25)** — but Pass 26 found:

1. **Same-burst internal inconsistency** in D-204 architect output (HIGH-001 + LOW-001) — ADR-0007 §Historical Anchor Classification vs SS-conventions §Historical Anchor Classification disagree on logic (ALL-of vs at-least-ONE-of) + POL-11 dispatch missing .yaml extension. Independent of META-pattern.

2. **META-pattern 8th instance at NEW LAYER NOT covered by ADR-0007** (MED-001) — sessions_panel.rs:7-16 module-level doc-comment column table is stale 6-column relative to canonical 7-column BC-2.06.005 PC-2 v1.0.6. ADR-0007 POL-11 enforces version-pin literal freshness; structural-spec drift (column count) is a different mechanism.

All 3 findings closed in 2-agent fix round. Counter HOLDS at 0/3.

## Verifications Performed

- [x] Pass 25 closures verified at 0813d4f + 89ee097
- [x] ADR-0007 ratification verified at 5e79f6a
- [x] SS-conventions §Citation Discipline section verified at 89ee097 (v1.32.0)
- [x] ARCH-INDEX v1.0.17 includes ADR-0006 + ADR-0007 rows
- [x] All Pass 11-25 fixes preserved at 0813d4f
- [x] CI on 0813d4f: all 9 SUCCESS
- [ ] **ADR-0007 internal-consistency audit** — FAIL (HIGH-001 + LOW-001)
- [ ] **Worktree code beyond version-pin literals** — FAIL (MED-001 META-pattern 8th instance)

## Findings

### F-S025-ADV26-HIGH-001 — ADR-0007 ↔ SS-conventions Historical-Anchor Rule Mismatch (CLOSED via architect e8d1088 Option B)

**Severity:** HIGH. **Confidence:** HIGH. **Routing:** architect (same agent as D-204 burst owner; in-burst correction per Principle 4). **Status:** CLOSED via e8d1088.

**Evidence:**
- ADR-0007 §Historical Anchor Classification lines 157-166: "ALL of the following: contextual marker AND version-monotonicity"
- SS-conventions §Historical Anchor Classification lines 1617-1630: "at least ONE of: §Trace, version-pin-historical, time qualifier"
- ADR-0007 §Implementation Plan POL-11 dispatch line 347: matched SS-conventions (3rd formulation)

**Closure:** Architect adopted Option B (at-least-one-of contextual marker). Rationale: (1) implementation complexity (monotonicity requires git-log lookups), (2) contextual-marker is already discriminating, (3) ADR-0007 §Decision body examples implied Option B from the start. ADR-0007 §Historical Anchor Classification rewritten; POL-11 dispatch aligned. ADR-0007 v1.0.0→v1.0.1 with §Trace v1.0.1; SS-conventions v1.32.0→v1.32.1 (body unchanged — already correct).

### F-S025-ADV26-MED-001 — META-pattern 8th instance: module-level doc-comment column table stale 6→7 (CLOSED via implementer 2d1188f)

**Severity:** MED. **Confidence:** HIGH. **Routing:** implementer (mechanical column-table refresh). **Status:** CLOSED via 2d1188f.

**Evidence:** sessions_panel.rs:7-16 module doc table listed 6 columns (Icon, Project, Status, Tokens, Cost, Uptime); canonical BC-2.06.005 PC-2 v1.0.6 mandates 7 columns starting with Session ID. Function doc (line 411), code (line 432), tests (sessions_panel.rs:669) all CORRECTLY use 7 columns; module doc is the lone outlier.

**Class identity:** META-pattern 8th instance at NEW LAYER not covered by ADR-0007. Same species root (authoring-time documentation drifts as canonical evolves) but different mechanism (structural-table shape vs literal pin).

**Closure:** Implementer added Session ID as first row to module doc table. Sweep-wider clean across ui/mod.rs + lib.rs + app.rs (no other stale structural-spec tables).

### F-S025-ADV26-LOW-001 — POL-11 dispatch extension list missing .yaml (CLOSED via architect e8d1088)

**Severity:** LOW. **Confidence:** HIGH. **Routing:** architect (same burst owner). **Status:** CLOSED via e8d1088.

**Evidence:** ADR-0007 line 347 POL-11 dispatch enumerated `.md/.rs/.toml/.yml` but §CI gate scope (lines 143-146) included `.yaml` and registry itself is `.yaml`.

**Closure:** Architect added `.yaml` to extension list. Also caught additional internal inconsistency at line 86 (§Scope of Affected Artifact Types) — also missing `.yaml`, fixed in same commit.

## ADR-0007 Conformance Audit

**Internal-consistency:** FAILS at HIGH (HIGH-001 + LOW-001) at Pass 26 inspection; **CLOSED via architect e8d1088 same-burst correction**.

**Coverage analysis vs META-pattern species:**
- ADR-0007 covers: literal `vN.M.P` citations in artifact bodies. PASSES coverage on instances 1-7 (specifically 18, 23, 24, 25 are the literal-pin instances).
- ADR-0007 does NOT cover: structural-spec drift in module-level doc-comments (column counts, table shapes, postcondition narrative). FAILS coverage on Pass 26 MED-001.

**Implementation status (per Pass 26 brief):**
- m.1-m.5: NOT YET IMPLEMENTED. Not a Pass 26 blocker.

## Strategic Scope-Extension Question (Pass 26 OBS)

ADR-0007 POL-11 catches literal version-pin drift; does it need extension to catch structural-spec drift in code-comment doc-tables?

**Orchestrator adjudication: DEFER per S-7.02 codification rule (3 instances trigger):**
1. Pass 26 MED-001 is the FIRST structural-spec drift instance (not yet a META-pattern by itself — META requires recurrence)
2. ADR-0007 POL-11 implementation hasn't landed yet (Task #9 m.1 pending) — extending scope before measuring effectiveness is premature
3. Bundling strategic question with tactical fix would expand S-025 scope past per-story convergence

**Anchored to Task #9 m.6 with TRIPWIRE**: if Pass 27/28/29 finds another structural-spec drift instance, immediately dispatch architect for ADR-0007 scope extension OR ADR-0008.

## Angles Attacked

| Angle | Result |
|-------|--------|
| II (ADR-0007 conformance audit) | FAIL — HIGH-001 + LOW-001 |
| KK (S-025 demo readiness re-audit per ADR-0007) | PASS — S-025 line 108 citation is legacy active pointer subject to opportunistic migration per ADR-0007 |
| LL (Cargo workspace post-Path-B regression) | PASS — workspace coherent |
| MM (ADR-0006 audit-table cross-crate coverage) | PASS — App + SessionsPanel correctly classified |
| NN (worktree test failure quality) | PASS — known-flaky list unchanged |
| XX (Pass 1-25 re-verification) | PASS |
| Worktree code beyond version-pin literals (sweep-wider) | FAIL — MED-001 META 8th instance |

## Counter Decision

HOLDS at 0/3. Pass 26 found 1 HIGH + 1 MED + 1 LOW. All 3 CLOSED in 2-agent fix round (architect e8d1088 + implementer 2d1188f). Counter does NOT advance until Pass 27 dispatches at post-fix HEAD and verifies clean.

## Defense of the Search

Pass 26 attacked 6 angles (II-XX) + wider worktree sweep. Three angles failed: II (ADR-0007 self-consistency) + worktree-sweep (META 8th instance) + LOW-001 sweep within ADR-0007. Pass 25 closures all verified clean. The architect's ADR-0007 strategic intervention is structurally sound for its target species but has same-burst internal-consistency defects that fresh-context catches (HIGH-001 + LOW-001) and a scope boundary (MED-001 outside coverage).

## META-Pattern Status

8 instances confirmed.

| Pass | Layer | ADR-0007 coverage |
|------|-------|-------------------|
| 9 | test-assertion (vacuous-mirror) | N/A different species |
| 16 | struct-metadata (audit-table) | N/A different species |
| 18 | impl-code worktree pointers | YES POL-11 catches |
| 22 | spec-filename broken anchor | NO different defect type |
| 23 | BC-body→arch-doc pins | YES POL-11 catches |
| 24 | sibling-artifact-directory (story inputs[] + VP) | YES POL-11 catches |
| 25 | code-citation BC-version pins | YES POL-11 catches |
| **26** | **module-doc structural-spec TABLE** | **NO different mechanism** |

ADR-0007 is a CORRECT strategic intervention for its target species (literal version-pin staleness at instances 18, 23, 24, 25). The 8th instance (column-table staleness) demonstrates the broader species root has more facets. Strategic scope extension DEFERRED per S-7.02 codification rule (need 3 instances of structural-spec drift before ADR scope extension).

## Recommended Next Actions

1. Architect e8d1088: HIGH-001 + LOW-001 closed (Option B adjudication)
2. Implementer 2d1188f: MED-001 closed (column table 6→7)
3. **Pass 27** at post-fix HEAD (2d1188f worktree + e8d1088 factory-artifacts) — targets 0/3 → 1/3
4. ADR-0007 scope-extension question DEFERRED to Task #9 m.6 with tripwire (2 more structural-spec drift instances triggers architect dispatch)
