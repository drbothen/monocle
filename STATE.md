---
document_type: pipeline-state
level: ops
project: monocle
version: "3.0"
status: active
producer: state-manager
timestamp: 2026-05-14T03:00:00Z
phase: pre-phase-1-final-gate-round-45-complete
current_step: round-46-validation-pending
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "round 45 fix burst commits e281286 (adv-r44 persist) + e7ef2b5 (conventions v1.13 Option b + narrative count drift closure); resolves F-R44-adv-1/2/3/4; 0/3 clean adversary passes after 12 rounds"
awaiting: "round 46 validation — if CLEAN, 1-of-3 required clean adversary passes (12 prior adversary rounds yielded zero clean passes); orchestrator will surface convergence-definition question to human after R46 result regardless of outcome per O-R44-1"
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
| **Current Phase** | pre-phase-1-final-gate-round-45-complete |
| **Current Step** | round-46-validation-pending |
| **Brief** | `.factory/specs/product-brief.md` v1.4.19 (commit c938364) |
| **Vision** | `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 (approved) |
| **Last Updated** | 2026-05-14T03:00:00Z |

## Phase Progress

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| -1: Reference Ingest (8 repos) | DONE | 2026-05-11 | 57+ artifacts; semport/ |
| 0.5-0.9: Brief v1.0->v1.4.10 + arch stubs | DONE | 2026-05-12 | |
| 0.99a-j: Rounds 1-19 convergence | DONE | 2026-05-13 | see cycles/cycle-001/burst-log.md |
| 0.99k: Round 20 validation | DONE | 2026-05-13 | consistency CLEAN; adversary 0 CRIT + 2 MED + 1 LOW |
| Pre-Phase-1 Final Gate | PENDING — round 46 validation; 0/3 clean adversary passes after 12 rounds | — | D-040/D-041/D-042 human ratifications valid but conditional; Q-3 still pending human CLAUDE.md refresh; gate retracted per D-043 (protocol violation); convergence-definition question surfaces post-R46 per O-R44-1 |
| 1: Spec Crystallization | not-started | — | |
| 2-7 | not-started | — | |

## Current Phase Steps

<!-- Last 5 steps only. Older steps archived to cycles/cycle-001/burst-log.md. -->

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| Round 41 fix burst: F-R40-1 (SS-conventions v1.11 Option a — removed --include CLI; rule paths.include authoritative) + F-R40-2 (SS-engine-module v1.1.11 — historical pinpoints) + proactive full 7-doc citation sweep (16 instances, only 2 stale found in F-R40-2 scope) | architect | DONE | commits 0bf426a + 6fc5ef4 + eaf4adf |
| Round 42 validation: consistency F-R42-cons-1 (brief stale citation 6th-recurrence) + adversary F-R42-adv-1 (POL-11 partial-arm-coverage in 3 sibling semgrep rules); adv-report persisted | validator+adversary+state-manager | DONE | commit c3440cf (adversary-pass-round-42.md) |
| Round 43 fix burst: F-R42-adv-1 (3 sibling semgrep rules dual-shape) + F-R42-cons-1 (brief citation + broader sweep) + D-042 scope correction (proactive root-cause closure) | architect+product-owner | DONE | commits c3440cf + 9cfd779 + c938364 + 9f3da82 |
| Round 44 validation: adversary F-R44-adv-1 HIGH (paths.include vs fixture corpus) + F-R44-adv-2/3 MEDIUM (narrative count drifts) + F-R44-adv-4 LOW; adv-report persisted | adversary+state-manager | DONE | commit e281286 (adversary-pass-round-44.md) |
| Round 45 fix burst: F-R44-adv-1 HIGH (Option b — fixture path to paths.include + Python script struct-name exclusion) + F-R44-adv-2/3 narrative count drift (5 rules / 3 steps) + F-R44-adv-4 auto-resolved | architect | DONE | commits e281286 + e7ef2b5 |

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
| D-039 | Round 39 fix: F-R38-1 Option B — narrowly drawn exception added to SS-conventions v1.10 clause 4 (regex constant strings within code-specification blocks are permitted as they ARE the specification; narrative prose elsewhere must still refer to delimiters by name); F-R38-2 4th-recurrence cross-artifact version-citation staleness fix (SS-forward-compatibility v1.2.1→v1.2.2; FC-01/FC-06 lock-in cells updated from v1.0.3→v1.0.6 for SS-daemon-lifecycle.md); META-pattern workflow mitigation rule documented by architect: run `grep -rn 'SS-[name].md v' .factory/specs/architecture/` before any version bump to enumerate all citation sites in one pass (addresses O-R36-1 process-gap at author-discipline level; CI codification remains O-R36-1 open decision). | 2026-05-13T22:00:00Z | state-manager |
| D-040 | D-031 RATIFIED by human at Phase 1 gate. Architecture-vs-vision authority: architecture wins on Phase 1 surfaces. Vision (`domain-monocle-vision-synthesis.md` v1.1.2) remains human-approved for intent; SS-*.md architecture docs are canonical for Phase 1 trait signatures. No spec edits required — spec text already reflects this framing. (Policy decision valid; applies once 3-clean-pass convergence threshold met and input-hash drift check passes.) | 2026-05-13T23:00:00Z | human (Josh Magady) |
| D-041 | D-032 RESOLVED by human at Phase 1 gate: STRICT ROUTING. No narrow exemption for mechanical count-propagation. Commit 688a5ed (architect edited product-brief.md) was a routing violation; product-owner v1.4.12 ratification was correct remediation. Going forward: architects discovering a cross-boundary citation refresh need MUST surface to orchestrator; orchestrator dispatches product-owner (or appropriate owner). CLAUDE.md routing table binding without exemption. Human may optionally codify "no narrow exemptions" language in CLAUDE.md §Correct Agent Routing at convenience; AI does not edit CLAUDE.md. (Policy decision valid; applies once 3-clean-pass convergence threshold met and input-hash drift check passes.) | 2026-05-13T23:00:00Z | human (Josh Magady) |
| D-042 | O-R36-1 RESOLVED by human at Phase 1 gate: OPTION (c) ACCEPTED. Manual workflow mitigation only — no CI codification, no tech-debt-register entry. Architect's grep-before-version-bump rule (documented in SS-forward-compatibility.md v1.2.2 §Trace) is the canonical mitigation. Author discipline relied upon; future cross-artifact version-citation staleness instances caught reactively by consistency-validator. (Policy decision valid; applies once 3-clean-pass convergence threshold met and input-hash drift check passes.) | 2026-05-13T23:00:00Z | human (Josh Magady) |
| D-043 | Orchestrator protocol violation detected by human: presented Phase 1 gate before achieving 3-clean-adversary-pass convergence threshold (required per orchestrator AGENTS.md Phase 1d). Adversary trajectory across R22-R39 had ZERO clean passes (a clean pass = 0 CRIT + 0 HIGH + 0 MED; LOW with human acceptance OK). Additionally, `/vsdd-factory:check-input-drift` was never executed (mandatory pre-gate requirement). Gate framing rolled back 2026-05-13T23:30:00Z; resuming convergence iteration. D-040/D-041/D-042 preserved as valid human policy ratifications, conditional on convergence. | 2026-05-13T23:30:00Z | orchestrator (recorded by state-manager) |
| D-044 | Round 41 fix: F-R40-1 Option (a) CLI flag removal (rule paths.include is authoritative scope governor; removing --include CLI eliminates root cause rather than patching glob semantics); F-R40-2 historical-pinpoint rewrite (§Trace narrates what was current at v1.1.8 fix time — refreshing to current would falsify historical narrative; annotated with explicit current-state note); D-042 manual citation-sweep rule retroactively applied across 7 architecture docs (16 instances examined, 2 stale found in F-R40-2 scope only, 0 other surprises). | 2026-05-13T23:55:00Z | state-manager |
| D-045 | Round 43 fix: F-R32-2 dual-shape discipline propagated to 3 sibling semgrep rules (S-7.01 propagation completeness; shell_injection 1→2 arms, naked_fs_write 1→2, raw_env_mutation 2→4; normative MUST language + CI sanity check added). F-R42-cons-1 brief citation refresh (SS-engine-module v1.1.10→v1.1.11) + broader-scope sweep across .factory/specs/ (23 hits classified, 1 stale found). D-042 scope hole proactively closed: grep pattern .factory/specs/architecture/ → .factory/specs/ recursive; secondary anchor-tolerant pattern added to SS-forward-compatibility v1.2.3; 6 documented recurrences cited. | 2026-05-14T01:30:00Z | state-manager |
| D-046 | Round 45 fix: F-R44-adv-1 Option (b) chosen (Option a rejected — reintroduces F-R40-1 CLI override pattern; Option c rejected — doubles maintenance via parallel rule); semgrep-fixtures/**/*.rs added to audit-completeness rule paths.include + Python script defines FIXTURE_STRUCT_NAMES = {"AuditFixtureMinimal", "AuditFixtureDerived"} for name-based exclusion in production-scan and Step 3 audit-table gap check. F-R44-adv-2/3 narrative count drift: 4 stale references fixed (lines 68-69 "five rules"; line 280 "three steps"; line 449 "three steps"). Proactive grep confirmed no other stale narrative-count references exist. F-R44-adv-4 auto-resolved. SS-conventions-anti-patterns v1.12 → v1.13. | 2026-05-14T03:00:00Z | state-manager |

User decisions (Q-series): Q-A1 vision v1.1.2 re-approved; Q-B R-001 at less than 10%; Q-license MIT/Apache-2.0 dual; Q-permission-enum Option A; Q-DTU-Phase-1 dtu-claude-code-hooks-v1 is Phase 1; Q-15-1 sealing removed; Q-16-5 FactoryAdapter divergence intentional; Q-16-6 FactoryState Option types; Q-Round-20 fix round-20 findings. All binding.

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | no | TUI product requires UX spec |

## Blocking Issues

_None — round 46 validation pending; 0/3 clean adversary passes after 12 rounds_

## Session Resume Checkpoint

**ROUND-45-COMPLETE** | Cycle: cycle-001 | Phase: pre-phase-1-final-gate-round-45-complete

### Immediate Next Action

**Round 46 validation chain.** Dispatch consistency-validator + adversary in parallel against post-round-45 state (SS-conventions v1.13 + SS-forward-compatibility v1.2.3 + brief v1.4.19 + SS-engine-module v1.1.11 + others current). Adversary scope: fresh context; production-grade lens; SS-conventions v1.13 (fixture path in paths.include + Python script FIXTURE_STRUCT_NAMES exclusion + narrative count corrections). Verify: (a) F-R44-adv-1 Option b GENUINELY resolves the fixture/production boundary without introducing new META-gaps; (b) F-R44-adv-2/3 narrative count fixes propagated completely; (c) any new META-pattern; (d) CLEAN PASS (0+0+0). If CLEAN: 1-of-3 required consecutive clean passes. If FINDINGS: route to specialist; re-validate.

**CRITICAL post-R46 orchestrator action (O-R44-1):** Surface the convergence-definition question to human REGARDLESS of R46 outcome. 12 adversary rounds (R22-R44) yielded zero clean passes. Novelty decreasing but not at zero. Per O-R42-2 + O-R44-1: human may need to ratify alternative convergence definition (e.g., "no HIGH+ findings for 3 consecutive rounds with severity decay" vs "strict 0+0+0"). After R46: orchestrator presents data to human with options.

**Q-3 still pending:** human will manually refresh CLAUDE.md operational pointers at convenience. AI does not edit CLAUDE.md. Phase 1 dispatch still requires Q-3 refresh + explicit human `/vsdd-factory:run-phase 1` invocation after convergence.

### Critical Artifacts (read for Phase 1)

1. `/Users/jmagady/Dev/monocle/CLAUDE.md` — canonical principle + agent routing
2. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2
3. `.factory/specs/product-brief.md` v1.4.19
4. `.factory/specs/architecture/SS-core-types-and-abi.md` v1.2.3
5. `.factory/specs/architecture/SS-engine-module.md` v1.1.11
6. `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.6
7. `.factory/specs/architecture/SS-permissions-phase1.md` v1.1
8. `.factory/specs/architecture/SS-deps-pin-manifest.md` v1.1.7
9. `.factory/specs/architecture/SS-conventions-anti-patterns.md` v1.13
10. `.factory/specs/architecture/SS-forward-compatibility.md` v1.2.3

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

All prior task history archived to `cycles/cycle-001/burst-log.md`. Current active task: CONVERGENCE ITERATION — Round 44 validation chain pending (immediate next action). Re-initialize from Immediate Next Action above if resuming in fresh context.

## Phase 1 Gate Questions — CONVERGENCE IN PROGRESS

1. **Vision-vs-architecture authority (D-031):** **Tentatively ratified by human 2026-05-13 (D-040); confirmation will be re-affirmed at re-presented gate post-convergence.**

2. **Architect-brief-routing precedent (D-032):** **Tentatively resolved by human 2026-05-13 (D-041) — STRICT ROUTING; will be re-affirmed at re-presented gate post-convergence.**

3. **CLAUDE.md operational pointer refresh:** **Still PENDING HUMAN ACTION.** Human will manually refresh §Current Pipeline State (Brief v1.4.2→v1.4.19) and §Architectural Authority (v1.4.2→v1.4.19, v1.1.1→v1.1.2) at convenience. AI does not edit CLAUDE.md. Phase 1 dispatch awaits this refresh.

## Pending Human Direction

**O-R36-1:** **Tentatively resolved by human 2026-05-13 (D-042) — option (c) manual mitigation accepted; will be re-affirmed at re-presented gate post-convergence.**

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (all bursts) | `cycles/cycle-001/burst-log.md` |
| Prior session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Full decisions D-001..D-024 | `cycles/cycle-001/burst-log.md` |
| Adversary reports | `.factory/plans/adversary-pass-*.md` |
| Consistency audits | `.factory/plans/consistency-audit-*.md` |
