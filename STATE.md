---
document_type: pipeline-state
level: ops
project: monocle
version: "5.7"
status: active
producer: state-manager
timestamp: 2026-05-15T06:00:00Z
phase: phase-1-spec-crystallization
current_step: phase-1-f-r70-fix-burst-complete-r71-pending
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "**PRE-PHASE-1 GATE PASS** declared 2026-05-14 per D-054. 26 adversary rounds + fix bursts in cycle-001. 22 BCs implementable; 0 content defects. 18+ META defense layers. Permanent residual catalog: F-R55-adv-1, F-R55-adv-3, F-R61-adv-1, F-R61-2 (frozen). Phase 1+ reverts to D-047 strict 3-clean-pass."
awaiting: "Adversary R71 + consistency-validator round 10 fresh-context re-review of PRD v1.6 + VP v1.6 + arch v1.0.12. D-047 strict pass 1 attempt 6."
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: pending
dtu_services: [hook-endpoints-x5]
current_cycle: cycle-001
---

<!--
DURABILITY-CHECKPOINT: fresh-context-resume-ready
Cycle: cycle-001 (ACTIVE — Phase 1 Spec Crystallization)
Phase: phase-1-spec-crystallization
Step: phase-1-f-r70-fix-burst-complete-r71-pending
-->

# Pipeline State: Monocle — ZERO-CONTEXT RESUME GUIDE

## READ THIS FIRST (fresh-context session)

Context was cleared. This file is your only prior context. Do:

1. Read this file completely before doing anything.
2. Read `/Users/jmagady/Dev/monocle/CLAUDE.md` — canonical principle + Correct Agent Routing companion principle bind every action.
3. Verify git state: `git -C /Users/jmagady/Dev/monocle/.factory log --oneline -5`
4. **Pre-Phase-1 Final Gate: PASS** declared 2026-05-14 per D-054. Architecture specs implementable; 22 BCs locatable; 0 content defects. 4-entry permanent META residual catalog frozen (see §Pre-Phase-1 Gate PASS below).
5. **F-R62 fix-burst COMPLETE (2026-05-14):** PRD v1.1 (f855835) + VPs v1.1 (8454ff2) + arch v1.0.8 (2db408f). D-047 strict 3-clean-pass cycle restarts; adversary R63 pass 1 ran.
6. **F-R63 fix-burst COMPLETE (2026-05-14):** PRD v1.2 (5a49b0b) + VPs v1.2 (4e220e3) + arch v1.0.9 (8bf3759). D-047 strict 3-clean-pass cycle restarts at pass 1; adversary R64 + consistency round 3 ran.
7. **R3-001 closure chain COMPLETE (2026-05-14):** Arch v1.0.10 (dc3af71; Pattern B oscillation-prevention) → PRD v1.3 (d8e66c3; 31 sites propagated) → VP v1.3 (2b24735; 32+42=74 sites propagated). D-047 strict pass 1 attempt 2 cycle restarted; R65 + cons R4 dispatched.
8. **F-R65 closure chain COMPLETE (2026-05-14):** R65 (77fccb7) FAIL — 3 content defects in arch BC-AUTH-002 (survived since F-R62-8: "Three" count + Bearer disposition). Cons R4 (3d33937) GAPS — 1 LOW R4-001 (5 missed PRD v1.2→v1.3 propagations in VP v1.3). Architect v1.0.11 (af2101d) closed F-R65-1/2/3. PRD v1.4 (e704b50) propagated arch v1.0.11 pin at 31 sites. VP v1.4 (56b57ac) propagated arch v1.0.11 + PRD v1.4 pin + R4-001 closure at 5 sites. D-047 strict pass 1 attempt 3 cycle restarted; R66 + cons R5 dispatched.
9. **F-R67 fix-burst COMPLETE (2026-05-15):** R66 (0fcab9f) CLEAN (counter 1/3) + cons R5 (f2edb33) CLEAN. Cons R6 (1f777ae) CLEAN. R67 (3d15abf) FAIL — 2 HIGH findings (F-R67-1: VP-TYPES-001 §Mechanism vs §Post-conditions intra-block contradiction; F-R67-2: PRD EC-045 off-by-one 262,144→262,145) + 1 process-gap observation Obs-1. Counter reset to 0/3. PRD v1.5 (d321935) closed F-R67-2. VPs v1.5 (6831e23) closed F-R67-1 + preemptive intra-block sweep of all 22 VPs (21 clean; only VP-TYPES-001 contradicted). D-047 strict pass 1 attempt 4 cycle restarted; R68 + cons R7 dispatched.
10. **R68+cons R7 burst COMPLETE (2026-05-15):** R68 (180e964) CLEAN (retry; first attempt API 529) — counter HELD at 0/3 (new attempt chain). Cons R7 (5f7c4e0) GAPS — 1 LOW R7-001 (VP-DAEMON-001 line 249 missed PRD v1.4→v1.5 pin propagation). VP v1.5.1 (f07d66c) closed R7-001 via single-line citation fix (patch-bump; zero semantic change; D-060). D-047 strict pass 1 attempt 5 restarted; R69 + cons R8 dispatched.
11. **R69+cons R8+R70+cons R9 burst COMPLETE (2026-05-15):** R69 (587dd0d) CLEAN (counter 1/3) + cons R8 (d75d15a) CLEAN. R70 (4b4aea1) FAIL — 3 substantive (F-R70-1: macOS runtime_dir None + F-R70-3: POSIX exit-code 130 SIGTERM mis-encoding + F-R70-2: sub-second timestamp precision) + 2 obs. Cons R9 (d8a61f2) CLEAN. Counter reset to 0/3. Arch v1.0.12 (727c826) closed F-R70-1+F-R70-3 (D-061). PRD v1.6 (76570ac) closed F-R70-2+BC-DAEMON-004/005 content propagation+E-DAEMON-004+EC-057/058/059. VP v1.6 (7ba155a) closed VP-DAEMON-004/005/006 content propagation+Obs-R70-2. D-047 strict pass 1 attempt 6 restarted; R71 + cons R10 pending.
12. Phase 1+ reverts to D-047 strict 3-clean-pass convergence (option b/c relaxations were pre-Phase-1 ONLY).

## Task Queue (active)

| # | Task | Status | Routing |
|---|------|--------|---------|
| T-1 | Phase 1: PRD synthesis from 22 pre-staged BCs | COMPLETE | product-owner (commits c69518d → f855835) |
| T-2 | Phase 1: Verification properties authoring (22 VPs) | COMPLETE | formal-verifier (commits b7a5715 → 8454ff2) |
| T-3 | Phase 1d: Adversary R62 pass 1 (D-047 strict) | COMPLETE — 10 FINDINGS → F-R62 fix-burst applied | adversary (commit 5713ccc) |
| T-4 | Phase 1 consistency audit pass 1 | COMPLETE — 3 GAPS → F-R62 fix-burst applied | consistency-validator (commit 0e322da) |
| T-7 | Adversary R63 (D-047 strict pass 1) on PRD v1.1 + VP v1.1 + arch v1.0.8 | COMPLETE — 2 FINDINGS → F-R63 fix-burst applied | adversary (commit 11a98c4) |
| T-8 | Consistency-validator round 2 on PRD v1.1 + VP v1.1 + arch v1.0.8 | COMPLETE — 3 GAPS → F-R63 fix-burst applied | consistency-validator (commit 200eb68) |
| T-9 | Adversary R64 (D-047 strict pass 1) on PRD v1.2 + VP v1.2 + arch v1.0.9 | COMPLETE — CLEAN (adversary) / GAPS 1 MED R3-001 (consistency R3) → R3-001 closure chain applied | adversary (81322c7) + consistency-validator (ba62a15) |
| T-10 | Consistency round 3 on PRD v1.2 + VP v1.2 + arch v1.0.9 | COMPLETE — 1 MED finding R3-001 → architect closure via D-057 Pattern B | consistency-validator (ba62a15) |
| T-11 | Adversary R65 (D-047 strict pass 1 attempt 2) on PRD v1.3 + VP v1.3 + arch v1.0.10 | COMPLETE — FAIL (3 content defects arch BC-AUTH-002; F-R65-1/2/3 → fix-burst applied) | adversary (commit 77fccb7) |
| T-12 | Consistency round 4 on PRD v1.3 + VP v1.3 + arch v1.0.10 | COMPLETE — GAPS (1 LOW R4-001 → closure in VP v1.4) | consistency-validator (commit 3d33937) |
| T-13 | Adversary R66 (D-047 strict pass 1 attempt 3) on PRD v1.4 + VP v1.4 + arch v1.0.11 | COMPLETE CLEAN — counter advanced 0→1/3 | adversary (commit 0fcab9f) |
| T-14 | Consistency round 5+6 on PRD v1.4 + VP v1.4 + arch v1.0.11 | COMPLETE CLEAN — cons R5 (f2edb33) CLEAN; cons R6 (1f777ae) CLEAN | consistency-validator |
| T-15 | Adversary R67 (D-047 strict pass 2) on PRD v1.4 + VP v1.4 + arch v1.0.11 | COMPLETE — FAIL (2 HIGH: F-R67-1 VP-TYPES-001 intra-block contradiction; F-R67-2 EC-045 off-by-one) → F-R67 fix-burst applied; counter RESET to 0/3 | adversary (commit 3d15abf) |
| T-16 | Adversary R68 (D-047 strict pass 1 attempt 4) on PRD v1.5 + VP v1.5 + arch v1.0.11 | COMPLETE CLEAN (retry; API 529 on attempt 1; commit 180e964) — counter HELD 0/3 | adversary |
| T-17 | Consistency round 7 on PRD v1.5 + VP v1.5 + arch v1.0.11 | COMPLETE GAPS — 1 LOW R7-001 (VP-DAEMON-001 line 249 missed PRD v1.4→v1.5 pin; commit 5f7c4e0); VP v1.5.1 (f07d66c) closed | consistency-validator |
| T-18 | Adversary R69 (D-047 strict pass 1 attempt 5) on PRD v1.5 + VP v1.5.1 + arch v1.0.11 | COMPLETE CLEAN — counter advanced 0→1/3 (commit 587dd0d) | adversary |
| T-19 | Consistency round 8 on PRD v1.5 + VP v1.5.1 + arch v1.0.11 | COMPLETE CLEAN (commit d75d15a) | consistency-validator |
| T-19b | Adversary R70 (D-047 strict pass 2 attempt 5) on PRD v1.5 + VP v1.5.1 + arch v1.0.11 | COMPLETE FAIL — 3 substantive + 2 obs; counter RESET 0/3 (commit 4b4aea1) → F-R70 fix-burst applied | adversary |
| T-19c | Consistency round 9 on PRD v1.5 + VP v1.5.1 + arch v1.0.11 | COMPLETE CLEAN (commit d8a61f2) | consistency-validator |
| T-20 | Adversary R71 (D-047 strict pass 1 attempt 6) on PRD v1.6 + VP v1.6 + arch v1.0.12 | pending dispatch | adversary |
| T-21 | Consistency round 10 on PRD v1.6 + VP v1.6 + arch v1.0.12 | pending dispatch | consistency-validator |
| T-22 | Adversary pass 2 (D-047 strict) | blocked: T-20 must produce CLEAN | adversary |
| T-23 | Adversary pass 3 (D-047 strict — convergence) | blocked: T-22 must produce CLEAN | adversary |
| T-24 | Input-hash drift check pre-human-gate | blocked: T-20..T-23 + T-21 | devops-engineer |
| T-25 | Human Phase 1 approval gate | blocked: T-24 | human |
| T-26 | Phase 2 entry (Story Decomposition) | blocked: T-25 | story-writer |

## Phase Progress

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| -1: Reference Ingest (8 repos) | DONE | 2026-05-11 | 57+ artifacts; semport/ |
| 0.5-0.9: Brief v1.0→v1.4.23 + arch stubs | DONE | 2026-05-14 | |
| 0.99a-j: Rounds 1-19 convergence | DONE | 2026-05-13 | see cycles/cycle-001/burst-log.md |
| 0.99k-m: Rounds 20-26 (R20-R61) | DONE | 2026-05-14 | see cycles/cycle-001/burst-log.md |
| Pre-Phase-1 Final Gate | **DONE** | 2026-05-14 | **GATE PASS per D-054**. 26 adv rounds. 18+ defense layers. 22 BCs; 0 content defects. 4-entry frozen META catalog. |
| 1: Spec Crystallization | **IN PROGRESS** | — | PRD v1.0+VP v1.0 → R62 FAIL(10f) → F-R62(v1.1) → R63 adv FAIL(2f)+cons R2 GAPS(3f) → F-R63(v1.2) → R64 CLEAN(adv)/GAPS 1 MED R3-001(cons R3) → arch v1.0.10(R3-001, Pattern B, D-057) → PRD v1.3+VP v1.3 → R65 FAIL(3f arch BC-AUTH-002)+cons R4 GAPS(1 LOW R4-001) → F-R65(arch v1.0.11+PRD v1.4+VP v1.4) → R66 CLEAN(counter 1/3)+cons R5+R6 CLEAN → R67 FAIL(2 HIGH: intra-block contradiction+EC-045 off-by-one; counter RESET 0/3) → F-R67(PRD v1.5+VP v1.5; intra-block sweep 21/22 VPs clean) → R68 CLEAN(retry; API 529 on attempt 1; counter HELD 0/3)+cons R7 GAPS(1 LOW R7-001) → VP v1.5.1(R7-001 closure; D-060) → R69 CLEAN(counter 1/3)+cons R8 CLEAN → R70 FAIL(3 substantive: macOS runtime_dir+POSIX exit codes+timestamp precision; counter RESET 0/3)+cons R9 CLEAN → F-R70(arch v1.0.12+PRD v1.6+VP v1.6; D-061) → R71+cons R10 pending. Convergence trajectory (attempts 1-10): 13→5→1→4→0→2→1→0→0→3. Pattern: NOT monotone; each fresh-context pass rotates review lens. D-047 strict producing genuine content corrections (R70 cross-platform invariants were a real macOS deployment blocker). Cycle-health observation surfaced for T-25 human gate. |
| 2-7 | not-started | — | |

## Pre-Phase-1 Final Gate — PASS (2026-05-14 per D-054)

**Status:** GATE PASS per D-054 human ratification option (c).

**18+ defense layers:** Constructor pattern + 17-struct audit; D-042 4-pattern recursive + WITHIN-FILE + DTU-SCOPE; PG-1 §Schema-Fact; PG-2 noun-agnostic narrative-count; PG-3 §Cross-Section Directional + §Trace-prose + ALL-PROSE + TRACE-NEW-ENTRY; PG-4 §-heading-existence 5-pattern + scope clause; PG-RECIPE-SCOPE; PG-5 §Historical-Anchor + Option B frontmatter + sweep-evidence; §Trace-Heading-Convention + heading-agnostic recipe; F-R60-corpus-sweep 5-step protocol; DTU split-column matrix; BC-HOOK-007 Option A gene-source qualifier.

**Permanent residual META catalog (frozen per D-054 — do NOT grow during Phase 1+):**

| ID | Description | Disposition |
|----|-------------|-------------|
| F-R55-adv-1 | PG-4 em-dash separator codification gap (em-dash form `§Item P3-1 — Verdict` accepted as alternate) | Permanent residual |
| F-R55-adv-3 | PG-4 intra-document scope hole (rule "cross-document" only; intra-doc bold-paragraph-label citations accepted) | Permanent residual |
| F-R61-adv-1 | PG-3-CLASSIFICATION-EVIDENCE (META rule's own §Trace may use bare L-numbers in post-fix summary shorthand) | Permanent residual |
| F-R61-2 | §Trace-Heading-Convention scope clause doesn't document ADR/vision/brief equivalents | Permanent residual |

**22 BCs in PRD v1.1 (f855835):** BC-RING-001, BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-AUTH-001/002, BC-LOCK-001, BC-ENGINE-001/002/002-ERR/003 (daemon-lifecycle: 10; core-types: 8; engine-module: 4) — authoritative count per SS-daemon-lifecycle.md §Behavioral Contract Summary. BC count UNCHANGED at 22 after F-R70 (BC-DAEMON-004/005 updated in place; no new BCs added).

## Phase 1+ Convergence Policy (per D-054 + D-047)

Phase 1+ work (PRD, VPs, architecture revisions, story decomposition, implementation, holdout eval, formal hardening, convergence) reverts to **D-047 strict**:
- 0 findings of any severity for 3 consecutive audit passes.
- Phase 1 adversarial review = R22-R61 pre-Phase-1 review level rigor.

The permanent META residual catalog (4 entries) is FROZEN per D-054. These items do not need re-fixing during Phase 1+. NEW findings on Phase 1 artifacts are NOT covered — they go through D-047 strict. If novel META-class instances emerge during Phase 1+, surface to human (do NOT auto-extend the bounded catalog).

## Phase 1 Entry — Artifact Inventory

| Artifact | Path | Status |
|----------|------|--------|
| Domain spec / Vision | `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 | EXISTS |
| Product brief | `.factory/specs/product-brief.md` v1.4.23 | EXISTS |
| PRD with 22 BCs + 14 error codes + 59 edge cases | `.factory/specs/prd.md` v1.6 | EXISTS (commit 76570ac; was v1.5 at d321935) |
| Verification properties (22 VPs) | `.factory/specs/verification-properties.md` v1.6 | EXISTS (commit 7ba155a; was v1.5.1 at f07d66c) |
| Architecture (7 SS files) | `.factory/specs/architecture/SS-*.md` | EXISTS (SS-daemon-lifecycle v1.0.12 at 727c826; was v1.0.11 at af2101d) |
| DTU assessment | `.factory/specs/dtu-assessment.md` v1.7 | EXISTS |
| ADRs (4) | `.factory/specs/architecture/adr/ADR-0001..0004` | EXISTS |
| CI/CD setup | `.github/workflows/` | MISSING — devops-engineer scope at Phase 3 |
| Phase 1d adversarial spec review | R69 CLEAN(counter 1/3)+cons R8 CLEAN → R70 FAIL(3 substantive; counter RESET 0/3)+cons R9 CLEAN → F-R70(arch v1.0.12+PRD v1.6+VP v1.6) → R71+cons R10 pending | IN PROGRESS |
| Human Phase 1 gate approval | pending T-20..T-24 | PENDING (T-25) |

## Decisions Log

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-047 | Human ratified strict 3-clean-pass policy. 3 consecutive 0+0+0+0 audit cycles required for convergence. | 2026-05-14 | human (Josh Magady) |
| D-053 | Pre-Phase-1 ONLY relaxation: 0 CRIT/HIGH + 0 MED-content + bounded LOW META for 3 passes. Phase 1+ reverts to D-047. | 2026-05-14 | human (Josh Magady) |
| D-054 | **PRE-PHASE-1 GATE PASS option (c):** 26 adv rounds (R22-R61). 22 BCs implementable; 0 content defects. 4-entry frozen META catalog. 18+ defense layers. Phase 1+ reverts to D-047 strict. | 2026-05-14 | human (Josh Magady) |
| D-055 | Architect adjudicated F-R62-8: BC-AUTH-002 disposition (c) Mixed. `missing_auth_token` for absent header; `invalid_auth_token` collapsed for all value-present failures. `invalid_auth_token_format` RETIRED. SS-daemon-lifecycle.md v1.0.8 commit 2db408f. | 2026-05-14 | architect (delegated via orchestrator) |
| D-056 | Product-owner adjudicated 4 test-name divergences (F-R63-adv-1). PRD v1.2 commit 5a49b0b; VP v1.2 commit 4e220e3. | 2026-05-14 | product-owner (delegated via orchestrator) |
| D-057 | Architect adjudicated R3-001: adopted Pattern B (explicit current/historical split with version-stable file path pointer) to prevent oscillation. SS-daemon-lifecycle.md v1.0.10 commit dc3af71. | 2026-05-14 | architect (delegated via orchestrator) |
| D-058 | Architect closed F-R65-1/F-R65-2/F-R65-3: corrected 3 content defects in arch BC-AUTH-002. SS-daemon-lifecycle.md v1.0.11 commit af2101d. | 2026-05-14 | architect (delegated via orchestrator) |
| D-059 | F-R67 closure dispositions: F-R67-1 routed to formal-verifier (VP-TYPES-001 §Mechanism); F-R67-2 routed to product-owner (PRD EC-045 prose 262,144→262,145). Intra-block sweep: 21 of 22 VPs clean. | 2026-05-15 | orchestrator (delegated to product-owner + formal-verifier) |
| D-060 | R7-001 (LOW — VP-DAEMON-001 line 249 missed PRD v1.4→v1.5 citation propagation) closed via VP v1.5.1 single-line citation fix. Patch-bump chosen; zero semantic change. | 2026-05-15 | formal-verifier (delegated via orchestrator) |
| D-061 | Architect closed F-R70-1 + F-R70-3 with disposition (c) on both: (c) hybrid runtime-dir resolution chain (MONOCLE_RUNTIME_DIR env override → ProjectDirs::runtime_dir() → ProjectDirs::data_local_dir() macOS/Win fallback → fail-fast DaemonStartError::RuntimeDirUnresolvable); (c) POSIX-correct distinct exit codes (0 graceful / 130 SIGINT / 143 SIGTERM / 2 admin shutdown / 1 startup fail). BC count UNCHANGED at 22 (BC-DAEMON-004 + BC-DAEMON-005 updated in place). Rationale: production-grade diagnostic information; macOS deployment unblocked without forcing operator env config; systemd/k8s/CI integrations receive distinguishable POSIX-correct exit codes. SS-daemon-lifecycle.md v1.0.12 commit 727c826. | 2026-05-15 | architect (delegated via orchestrator) |

User decisions (Q-series): Q-A1 vision v1.1.2; Q-B R-001 <10%; Q-license MIT/Apache-2.0 dual; Q-permission-enum Option A; Q-DTU-Phase-1 dtu-claude-code-hooks-v1; Q-15-1 sealing removed; Q-16-5 FactoryAdapter divergence intentional; Q-16-6 FactoryState Option types. D-048..D-052 in `cycles/cycle-001/burst-log.md`. All binding.

## Blocking Issues

_None — F-R70 closure chain complete (arch v1.0.12 at 727c826, PRD v1.6 at 76570ac, VP v1.6 at 7ba155a). R71 adversary pass 1 attempt 6 + consistency round 10 pending (T-20, T-21)._

## Session Resume Checkpoint

**F-R70 closure chain COMPLETE:** R69 (587dd0d) CLEAN (counter 1/3) + cons R8 (d75d15a) CLEAN. R70 (4b4aea1) FAIL — 3 substantive findings (F-R70-1: macOS `ProjectDirs::runtime_dir()` returns None — deployment blocker; F-R70-2: sub-second timestamp precision missing; F-R70-3: POSIX exit code 130 mapped to SIGTERM — should map to SIGINT; 143 is SIGTERM) + 2 observations. Cons R9 (d8a61f2) CLEAN. Counter RESET to 0/3. Arch v1.0.12 (727c826) closed F-R70-1 + F-R70-3 per D-061 (hybrid runtime-dir fallback chain + POSIX-correct exit codes 0/1/2/130/143). PRD v1.6 (76570ac) closed F-R70-2 (timestamp precision) + propagated BC-DAEMON-004/005 content + added E-DAEMON-004 RuntimeDirUnresolvable + new EC-057/058/059. VP v1.6 (7ba155a) propagated VP-DAEMON-004/005/006 content + closed Obs-R70-2. Error codes: 14 (was 13; +E-DAEMON-004). Edge cases: 59 (was 56; +EC-057/058/059). BC count UNCHANGED at 22.

Next actions: dispatch T-20 (adversary R71, D-047 strict pass 1 attempt 6) + T-21 (consistency-validator round 10) concurrently on PRD v1.6 + VP v1.6 + arch v1.0.12. Both fresh-context. D-047: 0 findings of any severity required for 3 consecutive passes. Convergence trajectory: 13→5→1→4→0→2→1→0→0→3 (10 attempts). R70's cross-platform findings were genuine macOS deployment blockers — validate that cycle-health observation and options (a/b/c) are surfaced for human decision at T-25 gate.

## Surfaced for Human Gate Decision

**Obs-R68-D2 (non-blocking — surfaced for T-25 human gate review):**

PRD §1.3 / §6 Differentiator D-2 ("VecDeque overlay stack") cites BC-ENGINE-001 + BC-ENGINE-002 as supporting BCs. Neither BC formally specifies TUI VecDeque overlay rendering — they specify the EngineModule trait + ClaudeCodeModule impl that enables the overlay data flow, not the rendering itself.

Three options for human decision at T-25:

- **(a)** Accept current framing — TUI overlay rendering is Phase 2 scope; cited BCs are architecturally necessary preconditions (no PRD change required).
- **(b)** Relabel D-2 in PRD §1.3/§6 to explicitly disclaim Phase-1 BC verification of TUI rendering (product-owner scope; minor PRD edit).
- **(c)** Add a new Phase-1 BC for TUI overlay rendering — expands BC count from 22 → 23 BCs (architect scope; potentially restarts D-047 counter).

Routing for resolution: product-owner (option b PRD update) OR architect (option c new BC) OR no-action (option a). Human decides at T-25 gate.

---

**Cycle-health observation — D-047 strict convergence trajectory (surfaced for T-25 human gate):**

Convergence cycle has run 10 attempts (R62-R70 + cons R1-R9). Trajectory: 13→5→1→4→0→2→1→0→0→3 findings. Pattern: NOT monotone. Each fresh-context pass examines different defect lenses and finds genuinely new substantive defects (R62 BC scope; R63 cross-artifact names; R64 stale anchor; R65 BC-AUTH-002 semantics; R67 intra-block + off-by-one; R70 cross-platform invariants + POSIX exit codes + BC-VP precision drift).

R70's findings (F-R70-1 macOS runtime-dir + F-R70-3 POSIX exit codes) were GENUINELY substantive — F-R70-1 was a real macOS deployment blocker. The strict policy is producing value but may have an asymptotic-convergence shape: each pass can always identify new review angles.

**Three options for human at T-25:**
- **(a)** Continue strict D-047 (0×3 consecutive) until convergence — finite time uncertain; each pass produces real findings.
- **(b)** Declare CONVERGENCE-WITH-DOCUMENTED-RESIDUALS at a checkpoint: accept current state (PRD v1.6, VP v1.6, arch v1.0.12) as PASS gate with explicit acknowledgment that future passes WILL find more defects (this is the nature of fresh-context adversary), proceed to Phase 2, defer further refinement to maintenance sweeps.
- **(c)** Tighten the policy further: require zero findings across N>3 consecutive passes with mandatory cross-domain lens variety (cross-platform, POSIX, security, performance, accessibility). Most rigorous but most expensive.

Human decision recommended at T-25.

## Critical Hook Lessons

- `validate-input-hash`: update frontmatter to `[live-state]` BEFORE editing a cycle file with computed hash
- `block-ai-attribution`: rejects "Co-Authored-By: Claude" + robot emoji in commits
- Use `git commit -F /tmp/<file>` for messages over 2KB
- FC items LOCKED: read SS-core-types-and-abi.md; do NOT re-derive
- 4-entry permanent META catalog (D-054) is frozen — do NOT add to it during Phase 1+
- Phase 1+ adversarial review reverts to D-047 strict (0 findings x 3 consecutive passes)
- 18+ codified META rules in SS-conventions-anti-patterns.md v1.28 guard Phase 1 work
- **Orchestrator dispatch-prompt scope verification (codified per F-R62-1 META-finding):** When authoring dispatch prompts that enumerate BC/VP/story scope, the orchestrator MUST verify the enumeration against the architecture's source-of-truth section (e.g., §Behavioral Contract Summary) rather than copying from secondary references like STATE.md task queues. STATE.md scope claims can drift from architecture; the architect's §Behavioral Contract Summary is the authoritative count. Codified after F-R62-1 (16-BC dispatch under-enumerated the architect's prescribed 22-BC PRD scope).
- **Partial-fix regression discipline (codified per F-R63 META-findings):** When a fix-burst reconciles cross-artifact drift (e.g., test paths, names, taxonomies), the fix MUST propagate to ALL affected artifacts in scope — including the architecture if architecture cited the prior values. F-R62-4 reconciled PRD ↔ VP test paths but did NOT propagate to architecture (S-7.01 violation); F-R63 propagated arch v1.0.9 to close the loop. Codification: every fix-burst dispatch must include an explicit propagation checklist for ALL artifact dimensions touched (paths, names, taxonomies, versions, counts) AND a sweep step that verifies all sibling artifacts that might cite the changed values.
- **Semantic propagation sweep (codified per F-R65 finding cluster):** When a fix-burst changes a count, enumeration size, or taxonomy boundary, the propagation checklist MUST include SEMANTIC sweeps for: (a) prose lead-in count statements; (b) cross-paragraph consistency on changed elements; (c) related test vectors; (d) sibling sections that summarize the count. Metadata-only sweeps (paths, names, versions) are INSUFFICIENT. Codified after F-R65 "Three" count surviving 3 passes because F-R62-8 dispatch only enumerated metadata propagation.
- **Intra-block / intra-artifact same-ID consistency sweep (codified per F-R67 Obs-1):** When a fix-burst updates any element of a multi-section block (BC/VP §Mechanism, §Post-conditions, §Verification, §Edge Cases, etc.) OR a multi-instance ID (EC-NNN, BC-NNN, VP-NNN that appears in both catalog/index AND body prose), the propagation checklist MUST include intra-block AND intra-artifact same-ID consistency verification. Codified after F-R67-1 (VP-TYPES-001 §Mechanism vs §Post-conditions contradiction) and F-R67-2 (PRD §3 EC-045 vs §9 catalog row numeric mismatch).
- **Cross-platform + POSIX-convention sweep discipline (codified per F-R70-1 + F-R70-3):** Spec authors and review agents MUST verify (a) every dependency-crate platform-behavior claim against the crate's documented platform-specific behavior (`directories::runtime_dir()` returns None on macOS/Windows; OS-specific paths differ for cache/data/runtime dirs); (b) every signal-handling exit-code claim against POSIX 128+N convention; (c) every cross-platform-invariant claim against NFR platform targets. Codified after F-R70-1 (macOS runtime_dir() blocker) and F-R70-3 (exit code 130 misencoding SIGTERM as SIGINT origin).

## Key Tech Stack

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14,
serde_yaml_ng 0.10, wasmtime 44, directories 6, notify 8, russh 0.60, rmcp 1.6,
reqwest 0.13, nucleo 0.5, serde_json =1.0.149 (EXACT), rand =0.8.6 (EXACT).

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (R1-R61) + D-048..D-052 | `cycles/cycle-001/burst-log.md` |
| Lessons learned (all rounds) | `cycles/cycle-001/lessons.md` |
| Prior session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Adversary reports | `.factory/plans/adversary-pass-*.md` |
| Consistency audits | `.factory/plans/consistency-audit-*.md` |
