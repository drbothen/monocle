---
title: S-025 Adversarial Pass 27
pass_number: 27
counter_before: 0/3
counter_after: 0/3 (HOLD — 1 MED structural-spec drift instance #2; TRIPWIRE FIRED; closed via 3-track strategy)
verdict: MED
head_sha_reviewed: 2d1188f (worktree) + b75219b (factory-artifacts pre-Pass-27-burst)
created: 2026-05-29
---

## Summary

Pass 27 dispatched at post-Pass-26 HEAD. Pass 26 2-agent closure (architect e8d1088 + implementer 2d1188f) verified clean. **However, Pass 27 found a 9th META-pattern instance + 2nd structural-spec drift at story-body type-name layer.**

D-205 m.6 OBS-001 TRIPWIRE FIRED at 2/3 threshold. Per orchestrator's explicit armament: "If Pass 27 finds another structural-spec drift instance → architect strategic dispatch."

**3-Track Strategy executed** (proven template from Pass 25 + 26):
- **Architect strategic** (cb68158): Option B chosen — ratified ADR-0008 for structural-claim discipline. ADR-0007 left focused on literal-pin discipline; ADR-0008 owns POL-12 for structural claims (type names, table shapes, postcondition counts, enum variants).
- **Story-writer tactical** (30fb391): S-025 v1.9 → v1.10; lines 144 + 228 `Vec<SessionState>` → `Vec<EnrichedSession>`. Sweep-wider clean. Cross-story propagation to S-028:63,147 deferred per BC-5.39.002 PC2.
- **State-manager** (this burst): D-206 captures both + 9-instance META-pattern bound by 2 ADRs.

## Verifications Performed

- [x] Pass 26 closures verified at 2d1188f + b75219b
- [x] sessions_panel.rs:7-17 module doc table = 7 columns with Session ID first
- [x] ADR-0007 v1.0.1 §Historical Anchor Classification = at-least-one-of (matches SS-conventions v1.32.1)
- [x] All Pass 11-25 fixes preserved at 2d1188f
- [x] CI on 2d1188f: all 9 SUCCESS
- [ ] **Story body type-name consistency** — FAIL (F-S025-ADV27-MED-001)

## Findings

### F-S025-ADV27-MED-001 — Story body type-name drift Vec<SessionState> vs canonical Vec<EnrichedSession> (META 9th + structural-spec #2)

**Severity:** MED. **Confidence:** HIGH. **Status:** CLOSED via story-writer 30fb391 + architect cb68158 strategic ratification.

**Evidence:**
- S-025:144 (Tasks list): `sessions: Vec<SessionState>`
- S-025:228 (Downstream Consumer Contract): `pub sessions: Vec<SessionState>,`
- Canonical: SS-tui.md:845 + app.rs:130 + BC-2.06.005 all use `Vec<EnrichedSession>`
- `SessionState` exists as runtime-internal enum (monocle-runtime::hooks) — NOT the App.sessions type

**Class identity:** 9th META-pattern instance + structural-spec drift instance #2 at story-body layer (Pass 26 instance #1 was code-comment column-table).

**Tactical closure:** Story-writer 30fb391 fixed lines 144 + 228; S-025 v1.9 → v1.10; STORY-INDEX v5.14 → v5.15. Sweep-wider clean (line 164 disclaimer correctly preserved). Cross-story S-028:63,147 deferred to wave-gate.

**Strategic closure:** Architect cb68158 ratified ADR-0008 (Option B):
- ADR-0008 v1.0.0 "Structural-Claim Discipline" — POL-12 with canonical source registry, structural-claim historical-anchor classification, CI enforcement spec
- ADR-0007 v1.0.1 → v1.0.2 (forward-reference paragraph only — no decision change)
- SS-conventions v1.32.1 → v1.32.2 (new §Structural-Claim Discipline section)
- ARCH-INDEX v1.0.17 → v1.0.18 (ADR-0008 registered)

**Architect rationale (per cb68158 commit):** Option B over Option A because POL-11 regex (literal-pin) and POL-12 table-parser (structural-claim) are incompatible algorithms — bundling produces unmaintainable single hook. ADR proliferation concern unfounded at project scale. 2 ADRs for 2 distinct detection mechanisms is correct factoring.

## Tripwire Status — D-205 m.6 OBS-001: CLOSED

Tripwire armed at D-205 with 2/3-threshold (orchestrator interpreted as "if Pass 27/28/29 finds 2nd instance → architect dispatch"). FIRED at Pass 27 with instance #2. Architect adjudication: Option B (ADR-0008 ratification). Tripwire CLOSED — species architecturally bound.

## META-Pattern Status — Bound by 2 ADRs

| Pass | Layer | Class | Coverage |
|------|-------|-------|----------|
| 9 | test-assertion vacuous-mirror | distinct | N/A |
| 16 | struct-metadata audit-table | distinct | N/A |
| 18 | impl-code worktree pointers | literal-pin | ADR-0007 POL-11 |
| 22 | spec-filename broken anchor | distinct | NO different defect type |
| 23 | BC-body→arch-doc pins | literal-pin | ADR-0007 POL-11 |
| 24 | sibling-artifact-directory (story inputs[] + VP) | literal-pin | ADR-0007 POL-11 |
| 25 | code-citation BC-version pins | literal-pin | ADR-0007 POL-11 |
| 26 | module-doc structural-spec TABLE | structural-claim #1 | **ADR-0008 POL-12** |
| 27 | story-body type-name | structural-claim #2 | **ADR-0008 POL-12** |

9 instances confirmed. 2 sub-species (literal-pin + structural-claim) each bound by their own ADR. Strategic interventions validated.

## Counter Decision

HOLDS at 0/3. Pass 28 dispatches at post-fix HEAD (2d1188f worktree + 30fb391 + cb68158 factory-artifacts).

## Angles Attacked

| Angle | Result |
|-------|--------|
| OO (demo readiness end-to-end) | PASS |
| PP (story body internal consistency) | **FAIL — F-S025-ADV27-MED-001** |
| RR (cross-cutting test discipline) | PASS |
| SS (ADR-0007 implementation pre-flight) | PASS |
| TT (Pass 1-26 re-verification) | PASS |

## Defense of the Search

Pass 27 attacked 5 angles + comprehensive prior-pass re-verification. Pass 26 closures verified clean. Story-body angle PP surfaced novel type-name drift that 26 prior passes missed because ADR-0007 focused attention on literal-pin layers and story-body internal type-consistency was never a prior attack axis.

The 2nd structural-spec drift instance at successive higher layer (code-comment → story-body) matches the literal-pin META-pattern's progressive escalation (impl-code → BC-body → sibling-artifact → code-citation). Orchestrator's tripwire armament was correctly calibrated.

## Recommended Next Action

1. Architect strategic CLOSED via cb68158 (ADR-0008 Option B)
2. Story-writer tactical CLOSED via 30fb391 (S-025 v1.10)
3. **Pass 28** at post-fix HEAD targets 0/3 → 1/3 (first advance after 9 META-pattern stalls)
4. Cross-story S-028:63,147 deferred per BC-5.39.002 PC2 (Task #9 wave-gate)
5. POL-12 devops implementation tracked per ADR-0008 §Implementation Plan (Task #9)
