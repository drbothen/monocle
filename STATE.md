---
document_type: pipeline-state
level: ops
project: monocle
version: "3.0"
status: active
producer: state-manager
timestamp: 2026-05-13T23:00:00Z
phase: pre-phase-1-final-gate-round-25-complete
current_step: round-26-validation-pending
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "round 25 fix burst commits 436d4d3 (engine v1.1.6) + f287592 (deps v1.1.7) + 3b90235 (conventions v1.4) + 11185a1 (brief v1.4.12); resolves F-R24-adv-1/2/3/5; round-24 consistency findings F-R24-cons-1/2/3/4 also resolved"
awaiting: "round 26 validation; on convergence, Phase 1 gate to human with 2 gate questions: vision-vs-architecture authority + architect-brief-routing precedent"
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: pending
dtu_services: [hook-endpoints-x5]
current_cycle: cycle-001
---

<!--
  STATE.md SIZE BUDGET: Keep this file under 200 lines.
  Historical content belongs in cycle files, NOT here.
  Run /vsdd-factory:compact-state if this file grows past 200 lines.
-->

# Pipeline State: Monocle — ZERO-CONTEXT RESUME GUIDE

## READ THIS FIRST (fresh-context session)

Context was cleared by the human. This file is the only prior context. Do:
1. Read this file completely before doing anything.
2. Read `/Users/jmagady/Dev/monocle/CLAUDE.md` — the canonical principle binds everything.
3. Follow the Immediate Next Action in the Session Resume Checkpoint section.
4. Verify: `git -C /Users/jmagady/Dev/monocle/.factory log --oneline -5`

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | monocle — single-binary Rust TUI for AI coding harness sessions |
| **Mode** | greenfield-with-reference-ingest (8 repos in semport/) |
| **Language** | Rust; MSRV Phase 1: 1.86 |
| **Current Phase** | pre-phase-1-final-gate |
| **Current Step** | round-26-validation-pending |
| **Brief** | `.factory/specs/product-brief.md` v1.4.12 (commit 11185a1) |
| **Vision** | `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 (approved) |
| **Last Updated** | 2026-05-13T23:00:00Z |

## Phase Progress

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| -1: Reference Ingest (8 repos) | DONE | 2026-05-11 | 57+ artifacts; semport/ |
| 0.5-0.9: Brief v1.0->v1.4.10 + arch stubs | DONE | 2026-05-12 | |
| 0.99a-j: Rounds 1-19 convergence | DONE | 2026-05-13 | see cycles/cycle-001/burst-log.md |
| 0.99k: Round 20 validation | DONE | 2026-05-13 | consistency CLEAN; adversary 0 CRIT + 2 MED + 1 LOW |
| Pre-Phase-1 Final Gate | PENDING round-26 validation | — | round-25 fix burst complete (commits 436d4d3 + f287592 + 3b90235 + 11185a1); 17 artifacts; 16 BCs pre-staged; temp-env ^0.3 + async_with_vars; env-var list corrected; Test Conventions v1.4; brief v1.4.12 ratified |
| 1: Spec Crystallization | READY — awaiting convergence + human approval | — | |
| 2-7 | not-started | — | |

## Current Phase Steps

<!-- Last 5 steps only. Older steps archived to cycles/cycle-001/burst-log.md. -->

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| Round 21 fix: F-R20-1 typed metadata/enrich error + F-R20-2 parse_frontmatter_field guard parity + F-R20-3 rustdoc url crate removal | architect | DONE | commits 83d5fc5 + 3495812 |
| Round 21 close-out: STATE.md + burst-log + session-checkpoints; D-030 logged | state-manager | DONE | commit (round-21) |
| Round 23 fix: F-R22-1/2 vision-verbatim vs vision-spirit-aligned provenance precision + F-R22-3 BC-ENGINE-002-ERR HomeUnresolvable test spec | architect | DONE | commits 563b573 + afe72a2 + 4f15092 |
| Round 24 validation: consistency + adversary (3 MEDIUM + 2 LOW surfaced); reports persisted | validator+adversary | DONE | commit 2fb7d82 (consistency-audit-round-24.md) |
| Round 25 fix burst: F-R24-adv-1 async test split + F-R24-adv-3 env-var coverage + F-R24-adv-5 test convention + F-R24-adv-2 brief ratification + F-R24-cons-3 daemon-lifecycle citation | architect+product-owner | DONE | commits 436d4d3 + f287592 + 3b90235 + 11185a1 |

## Decisions Log

<!-- D-001..D-015 archived to cycles/cycle-001/burst-log.md. -->

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-025 | FC lock-in: 6 FC items as Phase 1 contracts; SS-core-types-and-abi.md (700 lines); 10 BCs pre-staged | 2026-05-12 | state-manager |
| D-026 | Round 13 fix: 13 FC adversary defects; SS-engine-module.md NEW; ADR-0004 NEW; BC 10->13 | 2026-05-12 | state-manager |
| D-027 | Round 15 fix: vision authority restored; sealing removed; BC-ENGINE-003; BC 13->15 | 2026-05-13 | state-manager |
| D-028 | Round 17 fix: 8 round-16 findings; directories crate; constructors; ProcessSnapshot ppid+exe_path | 2026-05-13 | state-manager |
| D-029 | Round 19 fix: F-R18-1 CRITICAL BaseDirs::home_dir/.claude; F-R18-2 rustdoc+InvalidHookUrl; F-R18-3 frontmatter parser | 2026-05-13 | state-manager |
| D-030 | Round 21 fix: F-R20-1 typed EngineMetadataError (metadata + enrich both return Result); F-R20-2 parse_frontmatter_field guard parity; F-R20-3 url crate ref removed from rustdoc | 2026-05-13 | state-manager |
| D-031 | Round 23 fix: vision deemed non-authoritative for Phase 1 trait signatures per CLAUDE.md §Architectural Authority (later/more-specific wins); BC-ENGINE-002-ERR added specifying HomeUnresolvable test path with temp-env isolation; temp-env ^0.2 added as dev-dep to SS-deps (RAII cleanup superior to serial_test for panic safety). Vision document NOT edited. Pre-staging table in SS-engine-module still shows 3 engine BCs (stale); consistency gap noted for round-24 audit. | 2026-05-13 | state-manager |
| D-032 | Round 25 fix: temp-env ^0.2 → ^0.3 (async_with_vars for enrich() test half; with_vars retained for metadata() half); env-var unset list corrected (HOME/USERPROFILE/HOMEDRIVE/HOMEPATH; XDG_* removed); Test Conventions subsection added to SS-conventions-anti-patterns v1.4; product-owner authored v1.4.12 ratification of architect's v1.4.11 routing-precedent edit (option B: leave in place + ratify, less disruptive); routing-precedent question flagged for Phase 1 gate (see Phase 1 Gate Questions). | 2026-05-13 | state-manager |

User decisions (Q-series): Q-A1 vision v1.1.2 re-approved; Q-B R-001 at less than 10%; Q-license MIT/Apache-2.0 dual; Q-permission-enum Option A; Q-DTU-Phase-1 dtu-claude-code-hooks-v1 is Phase 1; Q-15-1 sealing removed; Q-16-5 FactoryAdapter divergence intentional; Q-16-6 FactoryState Option types; Q-Round-20 fix round-20 findings. All binding.

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | no | TUI product requires UX spec |

## Blocking Issues

_None — round 26 validation pending._

## Session Resume Checkpoint

**ROUND-25-CLOSE-OUT** | Cycle: cycle-001 | Phase: pre-phase-1-final-gate-round-25-complete

### Immediate Next Action

Round 26 validation chain. Orchestrator dispatches consistency-validator + adversary in parallel. Consistency-validator scope: full check of architecture docs + brief + vision + STATE.md for: BC count reconciliation (16); BC-ENGINE-002-ERR enumeration in all BC lists; version pointer consistency (SS-engine-module v1.1.6, SS-deps v1.1.7, SS-conventions v1.4, brief v1.4.12); temp-env v0.3 pin reference consistency. Adversary scope: SS-engine-module.md v1.1.6 (BC-ENGINE-002-ERR test spec) + SS-deps-pin-manifest.md v1.1.7 (temp-env pin + features) + SS-conventions-anti-patterns.md v1.4 (Test Conventions section) + product-brief.md v1.4.12 (v1.4.12 changelog entry + 3 daemon-lifecycle refresh); fresh context; production-grade lens. Specifically verify: (a) test spec is now genuinely compilable (sync + async halves correctly structured); (b) env-var list is complete and platform-correct; (c) v1.4 Test Conventions section is enforceable (CI rule executable); (d) v1.4.12 routing-precedent question is captured clearly for human ratification at Phase 1 gate; (e) no new silent-failure patterns introduced; (f) no further routing violations.

### Critical Artifacts (read for Phase 1)

1. `/Users/jmagady/Dev/monocle/CLAUDE.md` — canonical principle + agent routing
2. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2
3. `.factory/specs/product-brief.md` v1.4.12
4. `.factory/specs/architecture/SS-core-types-and-abi.md` v1.2.3
5. `.factory/specs/architecture/SS-engine-module.md` v1.1.6
6. `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.4
7. `.factory/specs/architecture/SS-permissions-phase1.md` v1.1
8. `.factory/specs/architecture/SS-deps-pin-manifest.md` v1.1.7
9. `.factory/specs/architecture/SS-conventions-anti-patterns.md` v1.4
10. `.factory/specs/architecture/SS-forward-compatibility.md` v1.2.1

### Key Tech Stack

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14, serde_yaml_ng 0.10, wasmtime 44, directories 6, notify 8, russh 0.60, rmcp 1.6, reqwest 0.13, nucleo 0.5, serde_json =1.0.149 (EXACT), rand =0.8.6 (EXACT), async-trait ^0.1, futures ^0.3, constant_time_eq ^0.3. 29 named workspace pins; 9 EXACT-pinned. EngineModule open (non-sealed) per vision.

### Critical Hook Lessons

- `validate-input-hash`: update frontmatter to `[live-state]` BEFORE editing a cycle file that has a computed hash
- `validate-template-compliance`: STATE.md must keep all required H2 sections from state-template.md
- `block-ai-attribution`: rejects "Co-Authored-By: Claude" + robot emoji in commits
- Use `git commit -F /tmp/<file>` for messages over 2KB
- Round-17 lesson: fix-axis (replace crate X) + behavioral-invariant axis (preserve Y) both required
- FC items LOCKED: read SS-core-types-and-abi.md; do NOT re-derive

## Task Queue Snapshot

Persisted from harness TaskList tool (not auto-saved). Active queue at last context-clear preparation:

### Pending (dependency-ordered)

| ID | Status | Subject | BlockedBy |
|---|---|---|---|
| #35 | done | Round 21 fix burst — F-R20-1/2/3 architect | — |
| #36 | done | Round 21 state-manager close-out | #35 |
| #37 | done | Round 22 validation chain | #36 |
| #38a | done | Round 23 fix burst — F-R22-1/2/3 architect | #37 |
| #38b | done | Round 23 state-manager close-out | #38a |
| #39 | done | Round 24 validation chain | #38b |
| #39b | done | Round 25 fix burst — F-R24-adv-1/2/3/5 architect + product-owner | #39 |
| #39c | done | Round 25 state-manager close-out | #39b |
| #40 | pending | Round 26 validation chain | #39c |
| #41 | pending | Iterate fix-validate cycle to convergence | #40 |
| #12 | in_progress | Re-present Phase 1 gate to human | #41 |

### Resumption protocol for fresh-context session

1. Re-create the queue above via `TaskCreate` calls (subjects from this table) + 1 `TaskUpdate` to set `#12` back to `in_progress` (assuming you re-create it; otherwise add it fresh).
2. Set blocking dependencies via `TaskUpdate addBlockedBy` per the table.
3. Mark `#39` as `in_progress` when you dispatch the round 24 validation chain.
4. Immediate Next Action above contains the full round 24 dispatch instructions.

### Completed history (rounds 1-20)

All prior tasks tracked the round-by-round convergence work. Full chronology is in `cycles/cycle-001/burst-log.md` (Bursts 1-23) — TaskList completion order matches burst order. Major milestones:
- Tasks #6-#19: Round 1-4 textual defer-pattern decay + Round 5 production-grade principle articulation + R5 substantive fix burst (commits 0bd4ba9 through 6638de5)
- Tasks #20-#27: Round 5-10 convergence to PRODUCTION_READY (R10 commit e0caccf)
- Tasks #28-#29: FC items locked pre-Phase-1 + round 13 fix burst (vision drift surfaced)
- Tasks #30-#33: Vision authority restored (Q-15-1); rounds 15-17 (commits 9fa9ebe + 48852c8 + 1b26c54)
- Task #34: Round 20 validation (consistency CLEAN; adversary INLINE → 3 round-20 findings persisted at adversary-pass-round-20.md)

If a fresh-context session needs full task history for retrospective analysis, read burst-log.md sequentially.

## Phase 1 Gate Questions for Human Review

These questions must be answered by the human before entering Phase 1. Both are flagged by the adversary as process-gap items requiring explicit ratification.

1. **Vision-vs-architecture authority (D-031):** Architect declared the vision "non-authoritative for Phase 1 trait signatures" per CLAUDE.md §Architectural Authority (later/more-specific wins). Content is correct per the principle; flagged because the vision was human-approved verbatim on 2026-05-11. Does the human ratify this framing explicitly?

2. **Architect-brief-routing precedent (D-032):** In commit 688a5ed, architect mechanically propagated a BC count update into product-brief.md (product-owner territory per CLAUDE.md routing table). Content was correct; routing was a violation. v1.4.12 was authored by product-owner as ratification. Does the human accept a narrow exemption for mechanical count-propagation across artifact boundaries, or should every cross-boundary edit route through the destination owner even when content is mechanical?

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (all bursts) | `cycles/cycle-001/burst-log.md` |
| Prior session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Full decisions D-001..D-024 | `cycles/cycle-001/burst-log.md` |
| Adversary reports | `.factory/plans/adversary-pass-*.md` |
| Consistency audits | `.factory/plans/consistency-audit-*.md` |
