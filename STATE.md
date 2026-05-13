---
document_type: pipeline-state
level: ops
project: monocle
version: "3.0"
status: active
producer: state-manager
timestamp: 2026-05-13T21:15:00Z
phase: pre-phase-1-final-gate-round-37-complete
current_step: round-38-validation-pending
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "round 37 fix burst commits 17373a3 (adv-r36 persist) + ee3f8ab (conventions v1.9) + ddc18b1 (brief v1.4.18); resolves F-R36-1 IMPORTANT + F-R36-2 MEDIUM"
awaiting: "round 38 validation — final convergence projected; if CLEAN, Phase 1 gate to human with 3 questions"
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
| **Current Step** | round-38-validation-pending |
| **Brief** | `.factory/specs/product-brief.md` v1.4.18 (commit ddc18b1) |
| **Vision** | `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 (approved) |
| **Last Updated** | 2026-05-13T21:15:00Z |

## Phase Progress

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| -1: Reference Ingest (8 repos) | DONE | 2026-05-11 | 57+ artifacts; semport/ |
| 0.5-0.9: Brief v1.0->v1.4.10 + arch stubs | DONE | 2026-05-12 | |
| 0.99a-j: Rounds 1-19 convergence | DONE | 2026-05-13 | see cycles/cycle-001/burst-log.md |
| 0.99k: Round 20 validation | DONE | 2026-05-13 | consistency CLEAN; adversary 0 CRIT + 2 MED + 1 LOW |
| Pre-Phase-1 Final Gate | PENDING round-38 validation | — | round-37 fix burst complete (commits 17373a3 + ee3f8ab + ddc18b1); F-R36-1 brief citation v1.1.9→v1.1.10 resolved; F-R36-2 §Trace de-quote propagation completeness (SS-conventions v1.8→v1.9 + brief v1.4.17→v1.4.18); O-R36-1 process-gap surfaced for human direction |
| 1: Spec Crystallization | READY — awaiting convergence + human approval | — | |
| 2-7 | not-started | — | |

## Current Phase Steps

<!-- Last 5 steps only. Older steps archived to cycles/cycle-001/burst-log.md. -->

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| Round 29 fix burst: F-R28-1 EnrichedSession Option<i64> + F-R28-2 3 more constructors + Cross-Crate Audit table + F-R28-3 HookResponse builder + F-R28-4 HookEventRecord struct + F-R28-5 v1.1.5 trace + F-R28-6 brief row order + ratification | architect+product-owner | DONE | commits 0b3f89d + dc719cd + 09642de + 03f08ad + 1427f4d |
| Round 30 validation: consistency + adversary (1 HIGH + 2 MED + 1 LOW — NEEDS_ONE_MORE); adv-report persisted | validator+adversary | DONE | commit bdbb97f (adversary-pass-round-30.md) |
| Round 31 fix burst: F-R30-1 audit table 7→17 + HTML delimiters + F-R30-2 HookEventRecord #[non_exhaustive] + F-R30-3 semgrep rule + Python script CI enforcement + F-R30-4 ISO-8601 convention + ratification | architect+product-owner | DONE | commits bdbb97f + ed9842f + 0fc5803 + 2ad7459 + 442190f |
| Round 33 fix burst: F-R32-1 brief delimiter strings + F-R32-2 POL-11 META-GAP pattern-either + dual fixtures + F-R32-4 Python script edge cases + F-R32-3 STATE.md Q-3 version refresh | architect+product-owner+state-manager | DONE | commits 31ff515 + 2f05ab6 + e2e7d5a |
| Round 35 fix burst: F-R34-1 CRITICAL META-pattern defense-in-depth (line-anchored regex + §Trace prose de-quote + convention rule) + F-R34-2 standard #[$ATTR(...)] semgrep wildcard + F-R34-3 12-path workspace scope | architect | DONE | commits 5f35b1b + bdfc4b8 + f584c59 |
| Round 37 mechanical fix burst: F-R36-1 brief citation refresh + F-R36-2 §Trace v1.6 + brief v1.4.16/v1.4.17 de-quote (S-7.01 propagation completeness) | architect+product-owner | DONE | commits 17373a3 + ee3f8ab + ddc18b1 |

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
| D-033 | Round 27 fix: E0639 cross-crate struct-literal CRITICAL resolved via constructors on 4 structs (EngineMetadata, ProcessSnapshot with two variants, EnrichedSession, HookResponse); semgrep pattern-either expanded to cover use-import idiom; §Semgrep Coverage Hardening fixture-corpus + CI assertion specified per POL-11; brief v1.4.13 ratifies architect's round-27 work. | 2026-05-13 | state-manager |
| D-034 | Round 29 fix: EnrichedSession last_event_micros i64→Option<i64> (epoch sentinel eliminated); SpawnArgs/SessionHandle/EngineVersion constructors added (E0639 prevented in monocle-runtime/tests/); HookResponse builder pattern replaces pub-field mutation; HookEventRecord defined as real struct in monocle-runtime::ring with RING_FORMAT_VERSION const; v1.1.5 supersession annotation corrected; brief v1.4.15 ratifies + adds Cross-Crate Constructor Audit table codification rule. | 2026-05-13 | state-manager |
| D-035 | Round 31 fix: Cross-Crate Constructor Audit table expanded 7→17 structs with HTML delimiters for CI machine-parsing; HookEventRecord gets #[non_exhaustive]; new semgrep rule monocle-non-exhaustive-struct-audit-completeness + Python script spec gap-checks the audit table against semgrep-enumerated structs (audit-mechanism CI enforcement codified); ISO-8601 timestamp convention adopted prospectively for brief revision history. | 2026-05-13T18:30:00Z | state-manager |
| D-036 | Round 33 fix: semgrep rule pattern-either hardened (Shape A `AuditFixtureMinimal` + Shape B `AuditFixtureDerived` with #[derive(...)] interposed) closes POL-11 META-GAP that would have shipped audit-rule as functionally inert; Python script edge cases specified for all 5 malformed-input scenarios (header/separator skip, missing file, malformed delimiter pairs, duplicate delimiters, empty table); brief delimiter strings copy-pasted verbatim from source (corrected from paraphrase); F-R32-3 Q-3 staleness refreshed (v1.4.13→v1.4.17 current; SS-engine-module v1.1.5→v1.1.9). | 2026-05-13T19:15:00Z | state-manager |
| D-037 | Round 35 fix: F-R34-1 CRITICAL META-pattern — defense-in-depth: (1) line-anchored regex `^<!-- BEGIN: ... -->$`; (2) §Trace prose de-quoted (delimiters by name); (3) v1.8 convention rule prohibits verbatim quoting. Architect also fixed "Future audit maintenance" body-prose verbatim-quote (in-scope, not in adversary scope). F-R34-2: `#[$ATTR(...)]` standard semgrep form replaces `#[...]` (multi-arg derives handled by ellipsis). F-R34-3: paths.include 4→12 covering all 11 crates + binary. | 2026-05-13T20:30:00Z | state-manager |
| D-038 | Round 37 fix: F-R36-1 brief Success Criteria SS-engine-module citation v1.1.9→v1.1.10; F-R36-2 propagation completeness — v1.8 no-verbatim-quoting convention rule fully propagated to v1.6 §Trace entry (SS-conventions v1.8→v1.9) and brief v1.4.16/v1.4.17 revision-history entries (brief v1.4.17→v1.4.18); grep verified zero verbatim delimiter quotes in either file outside the canonical regex constant definitions. S-7.01 Partial-Fix Regression Discipline applied: convention rules introduced in one burst retro-applied to existing siblings in the same layer in the same burst. | 2026-05-13T21:15:00Z | state-manager |

User decisions (Q-series): Q-A1 vision v1.1.2 re-approved; Q-B R-001 at less than 10%; Q-license MIT/Apache-2.0 dual; Q-permission-enum Option A; Q-DTU-Phase-1 dtu-claude-code-hooks-v1 is Phase 1; Q-15-1 sealing removed; Q-16-5 FactoryAdapter divergence intentional; Q-16-6 FactoryState Option types; Q-Round-20 fix round-20 findings. All binding.

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | no | TUI product requires UX spec |

## Blocking Issues

_None — round 38 validation pending; final convergence projected._

## Session Resume Checkpoint

**ROUND-37-CLOSE-OUT** | Cycle: cycle-001 | Phase: pre-phase-1-final-gate-round-37-complete

### Immediate Next Action

Round 38 validation chain — FINAL convergence projected. Orchestrator dispatches consistency-validator + adversary in parallel.

Consistency scope: (a) F-R36-1 brief citation now v1.1.10 confirmed? (b) F-R36-2 zero verbatim delimiter quotes in §Trace prose anywhere across architecture docs + brief revision history? (c) v1.6/v1.4.16/v1.4.17 §Trace narrative preserved meaningfully (no semantic loss from de-quoting)? (d) audit table 17 structs + delimiters intact? (e) STATE.md Critical Artifacts list reflects v1.9 + v1.4.18.

Adversary scope: SS-conventions v1.9 + brief v1.4.18 + remaining specs at current versions; fresh context; production-grade lens. Verify: (a) all F-R36 findings GENUINELY resolved; (b) S-7.01 propagation completeness — no other §Trace entries in any file violate the convention; (c) no new META-pattern recurrence; (d) trajectory genuinely converged.

If CLEAN: present Phase 1 gate to human with 3 standing questions (D-031, D-032, Q-3) + Pending Human Direction (O-R36-1). If NOT clean: route to correct specialist appropriately.

### Critical Artifacts (read for Phase 1)

1. `/Users/jmagady/Dev/monocle/CLAUDE.md` — canonical principle + agent routing
2. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2
3. `.factory/specs/product-brief.md` v1.4.18
4. `.factory/specs/architecture/SS-core-types-and-abi.md` v1.2.3
5. `.factory/specs/architecture/SS-engine-module.md` v1.1.10
6. `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.6
7. `.factory/specs/architecture/SS-permissions-phase1.md` v1.1
8. `.factory/specs/architecture/SS-deps-pin-manifest.md` v1.1.7
9. `.factory/specs/architecture/SS-conventions-anti-patterns.md` v1.9
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

All prior task history archived to `cycles/cycle-001/burst-log.md`. Current active task: Round 38 validation chain (immediate next action). Re-initialize TaskList from Immediate Next Action above if resuming in fresh context.

## Phase 1 Gate Questions for Human Review

These questions must be answered by the human before entering Phase 1. Both are flagged by the adversary as process-gap items requiring explicit ratification.

1. **Vision-vs-architecture authority (D-031):** Architect declared the vision "non-authoritative for Phase 1 trait signatures" per CLAUDE.md §Architectural Authority (later/more-specific wins). Content is correct per the principle; flagged because the vision was human-approved verbatim on 2026-05-11. Does the human ratify this framing explicitly?

2. **Architect-brief-routing precedent (D-032):** In commit 688a5ed, architect mechanically propagated a BC count update into product-brief.md (product-owner territory per CLAUDE.md routing table). Content was correct; routing was a violation. v1.4.12 was authored by product-owner as ratification. Does the human accept a narrow exemption for mechanical count-propagation across artifact boundaries, or should every cross-boundary edit route through the destination owner even when content is mechanical?

3. **CLAUDE.md operational pointer refresh (F-R26-1):** `/Users/jmagady/Dev/monocle/CLAUDE.md` §Current Pipeline State cites `Brief: v1.4.2` (stale; current v1.4.18) and §Architectural Authority cites `vision v1.1.1` (current v1.1.2). The principle text in CLAUDE.md is human-authored authority and AI agents do not edit it. ACTION: At Phase 1 gate review, refresh the operational pointers to current versions before approving Phase 1 entry. Routing: human.

## Pending Human Direction

**O-R36-1 [process-gap, CLAUDE.md Rule 3]** — Cross-artifact version-citation staleness has recurred 3 times (R26 CLAUDE.md, R32 STATE.md, R36 brief). No CI check exists. AI may not add to tech-debt-register without explicit human direction. Human decision required from the following options:

> (a) **Codify as Phase 1 self-improvement story:** architect adds `scripts/check_version_citations.py` that grep-validates `<artifact> vX.Y.Z` citations across spec bodies against actual frontmatter versions. Runs in CI on every push to factory-artifacts.
>
> (b) **Add to tech-debt-register** with future-story anchor (e.g., Wave 1 story slot).
>
> (c) **Accept current state** — citation refresh is part of the round close-out process manually, acknowledged as recurring overhead.

Per CLAUDE.md Rule 3, AI agents may not add to tech-debt-register without explicit human direction AND a concrete future dependency. Select (a), (b), or (c) at Phase 1 gate review.

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (all bursts) | `cycles/cycle-001/burst-log.md` |
| Prior session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Full decisions D-001..D-024 | `cycles/cycle-001/burst-log.md` |
| Adversary reports | `.factory/plans/adversary-pass-*.md` |
| Consistency audits | `.factory/plans/consistency-audit-*.md` |
