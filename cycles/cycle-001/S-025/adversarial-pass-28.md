---
title: S-025 Adversarial Pass 28
pass_number: 28
counter_before: 0/3
counter_after: 0/3 (HOLD — 2 MED findings; closed via 3-track + CRITICAL elevation to devops POL implementation)
verdict: MED (2 findings: MED-001 + MED-002)
head_sha_reviewed: 2d1188f (worktree) + 12170b4/344366d (factory-artifacts pre-Pass-28-burst)
created: 2026-05-29
agents_dispatched:
  - architect: 12170b4 (factory-artifacts)
  - story_writer: 344366d (factory-artifacts)
  - devops: f0926fe (feature/S-025-tui-skeleton-sessions) + 5ea8ef3 (factory-artifacts)
  - state_manager: D-207 (this burst)
---

## Summary

Pass 28 dispatched at post-Pass-27 HEAD (2d1188f worktree + cb68158+30fb391 factory-artifacts). Pass 27 3-track closure verified. Pass 28 found 2 MED findings — one at the ADR-0008 internal-consistency layer and one at the story-body structural-claim layer (§Downstream Consumer Contract struct shape).

This was the BIGGEST single-burst round in cycle-001. The orchestrator elevated devops from advisory to CRITICAL to implement POL-11 + POL-12 CI enforcement in-scope rather than deferring to Task #9 post-merge batch. The elevation was vindicated: POL-11 self-test at authoring time caught 13 residual stale pins that 28 prior adversarial passes had missed.

**3-Track + devops POL elevation strategy executed:**
- **Architect strategic** (12170b4): ADR-0008 v1.0.1 — line-range correction §Canonical Source Registry + self-application policy + SS-conventions v1.32.3
- **Story-writer tactical** (344366d): S-025 v1.11 — §Downstream Consumer Contract historical-anchor annotation (Option B) + STORY-INDEX v5.16
- **Devops CRITICAL elevation** (f0926fe + 5ea8ef3): POL-11 + POL-12 CI implementation LIVE; 13 residual stale pins inline-fixed; version-pin-registry.yaml seeded
- **State-manager** (D-207, this burst): full closure + zero-context resume checkpoint rewrite

## Verifications Performed

- [x] Pass 27 closures verified at 2d1188f + cb68158 + 30fb391
- [x] ADR-0008 v1.0.0 §Canonical Source Registry table reviewed
- [x] SS-conventions v1.32.2 §Structural-Claim Discipline verified
- [x] S-025 v1.10 §Downstream Consumer Contract code block reviewed
- [x] CI on 2d1188f: all 9 SUCCESS (confirmed pre-Pass-28)
- [ ] **ADR-0008 §Canonical Source Registry line-range accuracy** — FAIL (F-S025-ADV28-MED-002)
- [ ] **S-025 §Downstream Consumer Contract struct-shape structural claim** — FAIL (F-S025-ADV28-MED-001)

## Findings

### F-S025-ADV28-MED-001 — Story §Downstream Consumer Contract struct shape (META 10th + structural-claim #3)

**Severity:** MED. **Confidence:** HIGH. **Status:** CLOSED via story-writer 344366d (Option B historical-anchor annotation).

**Evidence:**
- S-025:225-231 §Downstream Consumer Contract code block listed 5 App fields:
  `sessions`, `selected_idx`, `last_refresh`, `is_loading`, `error_banner`
- Canonical SS-tui.md §App struct: 9 fields (as of v1.8.2)
- Production app.rs: 7 fields (as of 2d1188f)
- 3-way divergence: story (5 fields) vs canonical spec (9 fields) vs production (7 fields)

**Class identity:** 10th META-pattern instance + structural-claim instance #3 (story-body struct shape). Same root species as Pass 26 (module-doc column table) and Pass 27 (story-body type-name).

**Tactical closure (Option B):** Story-writer 344366d annotated the §Downstream Consumer Contract code block at lines 225-231 with `<!-- structural-claim-historical -->` per ADR-0008 §Historical-Anchor protocol. Tasks list line 144 clarified with parenthetical "(S-025 introduces these App fields; existing/future per SS-tui.md §App struct + app.rs)". STORY-INDEX v5.15 → v5.16.

**System-level 3-way divergence (DEFERRED):** The divergence between story 5 fields, SS-tui.md 9 fields, and production app.rs 7 fields requires architectural alignment. DEFERRED to phase-5 per BC-5.39.002 PC2 (cross-story architectural alignment) — logged as F-S025-ADV28-OBS-002 in durable_task_register.

### F-S025-ADV28-MED-002 — ADR-0008 §Canonical Source Registry off-by-2 line range (self-application defect)

**Severity:** MED. **Confidence:** HIGH. **Status:** CLOSED via architect 12170b4.

**Evidence:**
- ADR-0008 v1.0.0 §Canonical Source Registry listed App struct at SS-tui.md lines 831-864
- Actual location in SS-tui.md: lines 833-864 (off by 2 at line-range start)
- SS-conventions v1.32.2 §Structural-Claim Discipline propagated the same off-by-2 error (mirrored citation)

**Class identity:** Self-application defect — ADR-0008 §Canonical Source Registry is explicitly subject to POL-12 (per ADR-0008 §Self-Application Policy added in this fix). The ADR's own structural claim about SS-tui.md line location was stale/incorrect at authoring time. This is the 3rd consecutive ADR same-burst internal-consistency defect: ADR-0006 (Pass 16), ADR-0007 (Pass 26), ADR-0008 (Pass 28). Pattern codified as F-S025-ADV28-OBS-001 (architect protocol enhancement).

**Tactical closure:** Architect 12170b4 — ADR-0008 v1.0.0 → v1.0.1: line-range corrected to 833-864 in §Canonical Source Registry; §Self-Application Policy made explicit (ADR-0008 §Canonical Source Registry is subject to POL-12). SS-conventions v1.32.2 → v1.32.3: same off-by-2 propagation corrected.

## Devops CRITICAL Elevation — POL-11 + POL-12 Implementation

### Decision context

The orchestrator elevated the Task #9 m.1 + m.7 devops work from "post-S-025-merge" priority to CRITICAL in-scope. Rationale: Pass 28 represents the first pass with both ADRs architecturally ratified; dispatching Pass 29 against an unenforceable policy corpus (POL-11 + POL-12 defined but not mechanically enforced) would produce findings that could only be resolved by later CI enforcement anyway.

### Files added to S-025 branch (f0926fe)

- `scripts/check_version_pins.py` — POL-11 implementation (580 lines, stdlib-only Python). Sweeps all 10 artifact directories (worktree code + factory specs + stories + VPs + BCs); extracts cited SS-doc version pins; compares against version-pin-registry.yaml canonical; reports stale + historical-anchor-exempt hits.
- `scripts/check_structural_claims.py` — POL-12 Phase 1 (560 lines, stdlib-only Python). Parses struct-shape code blocks and table headers; compares against canonical source registry in version-pin-registry.yaml; reports unlabeled structural claims.
- `scripts/structural-claim-deferrals.yaml` — authorized deferral registry. S-028 line 147 deferred per ADR-0008 §Deferral Protocol (cross-story propagation, anchored to Task #9 m.8).
- `scripts/tests/run_pol_tests.py` + 4 fixture files (pol11/12 × stale/historical) — 4/4 fixture tests PASS.
- `lefthook.yml` — pre-commit hook wiring for both POL-11 and POL-12 checks.

### Files modified on S-025 branch (f0926fe)

- `.github/workflows/ci.yml` — new `pol-lint` job; `build-and-test` job requires `pol-lint` to pass.
- `CLAUDE.md` (4 sites) — S-022 changelog entries converted to time-qualified historical anchors per ADR-0008 §Historical-Anchor protocol.
- `clippy.toml` — SS-conventions v1.31.0 → version-free semantic anchor.
- `deny.toml` — SS-conventions v1.31.0 → version-free semantic anchor.
- `crates/monocle-ipc/src/framing.rs` — SS-ipc v1.4.0 → version-free semantic anchor.
- `crates/monocle-ipc/src/types.rs` (2 sites) — BC-2.05.002 v1.0.4 → version-free semantic anchor.
- `crates/monocle-runtime/src/engine/claude_code.rs` (2 sites) — SS-engine-module v1.1.20 → version-free semantic anchor.
- `crates/monocle-runtime/src/state.rs` — BC-2.05.002 v1.0.4 → version-free semantic anchor.

### Files added to factory-artifacts (5ea8ef3)

- `.factory/specs/version-pin-registry.yaml` — 91 entries. Contains canonical versions for all SS-docs, ADRs, BCs, VPs referenced across the worktree. This is the authoritative registry that POL-11 + POL-12 scripts consult. Task #9 m.2 DONE inline.
- `.factory/stories/S-028-sessions-filter-event-ribbon.md` — structural-claim-historical annotation at line 147 per ADR-0008 authorized deferral protocol (Task #9 m.8 partial — cross-story propagation).

### Self-test results at f0926fe (KEY EMPIRICAL EVIDENCE)

- POL-11: **13 RESIDUAL STALE PINS** caught at authoring time that 28 prior adversarial passes had missed. All fixed inline during f0926fe authoring.
- POL-12: 0 stale structural claims + 1 authorized deferral (S-028).
- 4/4 fixture tests PASS.
- CI runtime: sub-second for both checks.

**Significance:** POL-11 is empirically more effective than 28 fresh-context adversary passes at catching the literal-pin sub-species. The 13 inline fixes confirmed the architectural intervention (ADR-0007) was correct: enforcement at authoring time catches what human review misses at review time. The elevation to CRITICAL was vindicated by empirical evidence.

## Counter & Trajectory

- Pass 28 counter: HOLDS at 0/3 (2 MED findings found and closed; counter rule — any MED+ holds or resets counter)
- Trajectory append: `→M(28)+POL-live`
- 1/3 → 2/3 transition failure count: **10** (Passes 9, 16, 18, 22, 23, 24, 25, 26, 27, 28)
- Pass 29 is the FIRST adversarial pass with POL-11 + POL-12 mechanically enforced in CI

## Artifact Versions After This Burst (D-207)

- ADR-0008 v1.0.0 → **v1.0.1** (12170b4: line-range fix 831→833 + self-application policy)
- SS-conventions v1.32.2 → **v1.32.3** (12170b4: off-by-2 propagation corrected)
- S-025 v1.10 → **v1.11** (344366d: §Downstream Consumer Contract historical-anchor annotation)
- STORY-INDEX v5.15 → **v5.16** (344366d)
- version-pin-registry.yaml **NEW** (91 entries, 5ea8ef3)
- ADR-0007: still v1.0.2 (no change)
- ARCH-INDEX: still v1.0.18 (no change)

## Pattern-of-Patterns Observation

3 consecutive ADR same-burst internal-consistency defects:
- Pass 16 → ADR-0006 audit-table inconsistency (one-off assumption at authoring)
- Pass 26 → ADR-0007 §Historical Anchor Classification ALL-of vs at-least-ONE-of mismatch
- Pass 28 → ADR-0008 §Canonical Source Registry off-by-2 line range

This is a 3-instance pattern within architect ADR authoring. Per S-7.02 codification rule at 3 instances: architect protocol enhancement recommended. Logged as F-S025-ADV28-OBS-001. Proposed enhancement: pre-commit self-consistency step — before committing any ADR, re-read each cited canonical line range to verify accuracy. Anchored to Task #9 m.9 (new entry).

## CI Status at Pass 28 Closure

PR #28 HEAD: `f0926fe`. CI runs pending at time of state-manager burst (likely workflow trigger delay after devops push). **A fresh orchestrator resuming MUST verify CI green on f0926fe before dispatching Pass 29.** Required: all 9 prior jobs SUCCESS + new `pol-lint` job SUCCESS.
