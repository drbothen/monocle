---
document_type: pipeline-state
level: ops
project: monocle
version: "3.0"
status: active
producer: state-manager
timestamp: 2026-05-13T15:00:00Z
phase: pre-phase-1-final-gate
current_step: durability-close-out-round-21-pending
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "round-20 validation: consistency CLEAN + adversary 0+2+1"
awaiting: "round 21 fix burst — F-R20-1/2/3; then round 22 validation; then Phase 1 gate"
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
| **Current Step** | round-21-fix-burst-pending |
| **Brief** | `.factory/specs/product-brief.md` v1.4.10 (commit 08b4a9c) |
| **Vision** | `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 (approved) |
| **Last Updated** | 2026-05-13T14:30:00Z |

## Phase Progress

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| -1: Reference Ingest (8 repos) | DONE | 2026-05-11 | 57+ artifacts; semport/ |
| 0.5-0.9: Brief v1.0->v1.4.10 + arch stubs | DONE | 2026-05-12 | |
| 0.99a-j: Rounds 1-19 convergence | DONE | 2026-05-13 | see cycles/cycle-001/burst-log.md |
| 0.99k: Round 20 validation | DONE | 2026-05-13 | consistency CLEAN; adversary 0 CRIT + 2 MED + 1 LOW |
| Pre-Phase-1 Final Gate | PENDING round-21 fix + round-22 validation | — | 17 artifacts; 15 BCs pre-staged |
| 1: Spec Crystallization | READY — awaiting convergence + human approval | — | |
| 2-7 | not-started | — | |

## Current Phase Steps

<!-- Last 5 steps only. Older steps archived to cycles/cycle-001/burst-log.md. -->

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| Round 19 fix: SS-engine-module v1.1.2 — F-R18-1 BaseDirs + rustdoc + BC-ENGINE-002 wording | architect | DONE | commit 4e386d9 |
| Round 19 fix: SS-core-types-and-abi v1.2.2 — F-R18-2 rustdoc + InvalidHookUrl + F-R18-3 frontmatter parser | architect | DONE | commit 33b5a0a |
| Round 19 close-out: STATE.md + burst-log + session-checkpoints; D-029 logged | state-manager | DONE | commit 1b26c54 |
| Round 20 validation: consistency CLEAN + adversary 0+2+1; round-20 report persisted | validator+adversary | DONE | commit 636d8d4 + this commit |
| Durability close-out: STATE.md zero-context rewrite; all untracked committed | state-manager | DONE | this commit |

## Decisions Log

<!-- D-001..D-015 archived to cycles/cycle-001/burst-log.md. -->

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-025 | FC lock-in: 6 FC items as Phase 1 contracts; SS-core-types-and-abi.md (700 lines); 10 BCs pre-staged | 2026-05-12 | state-manager |
| D-026 | Round 13 fix: 13 FC adversary defects; SS-engine-module.md NEW; ADR-0004 NEW; BC 10->13 | 2026-05-12 | state-manager |
| D-027 | Round 15 fix: vision authority restored; sealing removed; BC-ENGINE-003; BC 13->15 | 2026-05-13 | state-manager |
| D-028 | Round 17 fix: 8 round-16 findings; directories crate; constructors; ProcessSnapshot ppid+exe_path | 2026-05-13 | state-manager |
| D-029 | Round 19 fix: F-R18-1 CRITICAL BaseDirs::home_dir/.claude; F-R18-2 rustdoc+InvalidHookUrl; F-R18-3 frontmatter parser | 2026-05-13 | state-manager |

User decisions (Q-series): Q-A1 vision v1.1.2 re-approved; Q-B R-001 at less than 10%; Q-license MIT/Apache-2.0 dual; Q-permission-enum Option A; Q-DTU-Phase-1 dtu-claude-code-hooks-v1 is Phase 1; Q-15-1 sealing removed; Q-16-5 FactoryAdapter divergence intentional; Q-16-6 FactoryState Option types; Q-Round-20 fix round-20 findings. All binding.

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | no | TUI product requires UX spec |

## Blocking Issues

| ID | Issue | Severity | Owner |
|----|-------|----------|-------|
| F-R20-1 | Silent fallback in metadata() returns relative .claude path when HOME unresolvable | MEDIUM | architect |
| F-R20-2 | parse_frontmatter_field lacks guards present in sibling (block scalars, empty, flow lists) | MEDIUM | architect |
| F-R20-3 | Rustdoc references unpinned url crate absent from SS-deps | LOW | architect |

Full report: `.factory/plans/adversary-pass-round-20.md`

## Session Resume Checkpoint

**DURABILITY-CHECKPOINT: zero-context-resume-ready** | Cycle: cycle-001 | Phase: pre-phase-1-final-gate-round-20-complete

### Immediate Next Action

**Round 21 fix burst AUTHORIZED but NOT dispatched.** Dispatch vsdd-factory:architect with:

> Round 21 architect fix burst. Read CLAUDE.md CANONICAL PRINCIPLE first. Three findings at .factory/plans/adversary-pass-round-20.md:
>
> Fix 1 — F-R20-1 MEDIUM (SS-engine-module.md lines 309+345): Replace silent unwrap_or_else(PathBuf::from(".claude")) fallback. Add EngineMetadataError::HomeUnresolvable variant; change metadata() to return Result<EngineMetadata, EngineMetadataError>; fail fast on None from BaseDirs::new(). Update BC-ENGINE-001.
>
> Fix 2 — F-R20-2 MEDIUM (SS-core-types-and-abi.md lines 712-739): parse_frontmatter_field needs same guards as sibling parse_frontmatter_extra_fields: return None for block scalars, empty values, flow lists. Update rustdoc.
>
> Fix 3 — F-R20-3 LOW (SS-engine-module.md line 413): Remove Url::parse recommendation from ClaudeCodeModule::new rustdoc — url crate not in SS-deps-pin-manifest.
>
> Bump SS-engine-module v1.1.2->v1.1.3; SS-core-types-and-abi v1.2.2->v1.2.3. Use git commit -F /tmp/<file>. Never add Co-Authored-By or robot emoji.

After architect lands: state-manager logs D-030, updates current_step to round-21-complete. Then dispatch round-22 validation (consistency + adversary parallel). If both clean: present Phase 1 entry gate to human for /vsdd-factory:run-phase 1 approval.

### Critical Artifacts (read for Phase 1)

1. `/Users/jmagady/Dev/monocle/CLAUDE.md` — canonical principle + agent routing
2. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2
3. `.factory/specs/product-brief.md` v1.4.10
4. `.factory/specs/architecture/SS-core-types-and-abi.md` v1.2.2
5. `.factory/specs/architecture/SS-engine-module.md` v1.1.2 (pending v1.1.3 from round-21)
6. `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.4
7. `.factory/specs/architecture/SS-permissions-phase1.md` v1.1
8. `.factory/specs/architecture/SS-deps-pin-manifest.md` v1.1.5
9. `.factory/specs/architecture/SS-conventions-anti-patterns.md` v1.2.2
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
| #35 | pending | Round 21 fix burst — F-R20-1/2/3 architect | — |
| #36 | pending | Round 21 state-manager close-out | #35 |
| #37 | pending | Round 22 validation chain | #36 |
| #38 | pending | Iterate fix-validate cycle to convergence | #37 |
| #12 | in_progress | Re-present Phase 1 gate to human | #38 |

### Resumption protocol for fresh-context session

1. Re-create the queue above via 4 `TaskCreate` calls (subjects from this table) + 1 `TaskUpdate` to set `#12` back to `in_progress` (assuming you re-create it; otherwise add it fresh).
2. Set blocking dependencies via `TaskUpdate addBlockedBy` per the table.
3. Mark `#35` as `in_progress` when you dispatch the round 21 fix burst.
4. Reference Immediate Next Action Step B above for the literal architect prompt.

### Completed history (rounds 1-20)

All 28 prior tasks (TaskList #6-#34) tracked the round-by-round convergence work. Full chronology is in `cycles/cycle-001/burst-log.md` (Bursts 1-22) — TaskList completion order matches burst order. Major milestones:
- Tasks #6-#19: Round 1-4 textual defer-pattern decay + Round 5 production-grade principle articulation + R5 substantive fix burst (commits 0bd4ba9 through 6638de5)
- Tasks #20-#27: Round 5-10 convergence to PRODUCTION_READY (R10 commit e0caccf)
- Tasks #28-#29: FC items locked pre-Phase-1 + round 13 fix burst (vision drift surfaced)
- Tasks #30-#33: Vision authority restored (Q-15-1); rounds 15-17 (commits 9fa9ebe + 48852c8 + 1b26c54)
- Task #34: Round 20 validation (consistency CLEAN; adversary INLINE → 3 round-20 findings persisted at adversary-pass-round-20.md)

If a fresh-context session needs full task history for retrospective analysis, read burst-log.md sequentially.

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (all bursts) | `cycles/cycle-001/burst-log.md` |
| Prior session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Full decisions D-001..D-024 | `cycles/cycle-001/burst-log.md` |
| Adversary reports | `.factory/plans/adversary-pass-*.md` |
| Consistency audits | `.factory/plans/consistency-audit-*.md` |
