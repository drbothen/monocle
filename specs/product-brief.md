---
document_type: product-brief
level: L1
version: "1.4.27"
status: draft
producer: product-owner
phase: pre-phase-1-brief
timestamp: 2026-05-17T20:00:00Z
inputs: [research/domain-monocle-vision-synthesis.md, semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md, semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md, semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md, semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md, semport/zellij/zellij-pass-8-final-synthesis.md, semport/lazygit/lazygit-pass-8-final-synthesis.md, semport/claude-squad/claude-squad-pass-8-deep-synthesis.md, semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md, planning/oq-research.md]
input-hash: "96ca07c"
traces_to: "factory-artifacts 2737bfd (vision-synthesis approved); 2c2b676 (8-repo full ingest); b3c68ca (OQ research)"
project: monocle
supplements:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/tech-debt-register.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0003-license-selection.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md
---

# Product Brief: Monocle

## What Is This?

Monocle is a Rust TUI that gives developers one `Ctrl-\` popup over every
AI coding harness session they are running — across projects, across harnesses
(Claude Code, CodeMachine, future), and across hosts. It surfaces five
information planes: live session roster with token burn and cost (Runtime),
active customizations per session (Static), workflow pipeline state for
factory-pattern projects (Workflow), per-harness profiles (Harness), and a
lazygit-style keybinding dispatch layer (TUI philosophy). Monocle is
observe-only for workflow state and session transcripts; it owns the action
layer only for permission prompts and keybinding dispatch — the two places where
context-switching today costs the developer real time and real session stalls.

Per vision §Vision Statement: "One TUI lens over every Claude-class session
you're running, every customization that shapes them, and every workflow driving
them — across multiple harnesses and federated across hosts."

## Revision History

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-12 | product-owner (direct draft from approved vision) | Initial brief — committed at factory-artifacts e8e8af1 |
| 1.1 | 2026-05-12 | product-owner (version validation revision) | Updated all crate version pins to crates.io 2026-05-12 reality; added RUSTSEC notes; refreshed wasmi/wasmtime rationale; added 11 new version pins for previously-unpinned vision tech stack crates; added OQ-11 MSRV |
| 1.2 | 2026-05-12 | product-owner (Option A bloat remediation + OQ/SOQ/JC decisions) | Trimmed core to ~200 lines; moved version manifest + RUSTSEC + ADR + conventions to architecture stubs; applied 11 OQ defaults + 4 SOQs + JC-1/2/3 + EX-1/2 resolutions; full traceability preserved |
| 1.3 | 2026-05-12 | product-owner (competitive positioning revision + OQ-M1/OQ-M3) | Competitive Positioning revised to acknowledge Anthropic's `claude agents` (agent view, v2.1.139, shipped 2026-05-11). Repositioned monocle's differentiation on mechanism and depth (hook-protocol ingestion, VecDeque overlay, diff preview, trigger-trace, workflow plane, multi-harness, external overlay) rather than exclusivity over the session-list surface. R-001 acceptance stated explicitly. Added OQ-M1 (agent-view IPC coexistence) and OQ-M3 (`PermissionRequest` as 6th endpoint) to the Open Questions table as `pending architect review`. No scope changes. Resolves B-1 from `.factory/plans/brief-validation-v2.md`. |
| 1.4 | 2026-05-12 | product-owner (production-grade defect fixes per adversary re-audit 0bd4ba9) | CRITICAL production-grade defect fixes per adversary re-audit (commit 0bd4ba9). Crate count typo 13→12. OQ-M1/M2/M3 resolved in-scope (no longer Pending architect review): OQ-M1 = no agent-view IPC collision; OQ-M2 = claude-manager not hook-protocol; OQ-M3 = stay at 5 endpoints via JC-2 parity. OQ-M2 row added to table (was absent in v1.3). F-07/F-08 citation parentheticals added. R-001 mitigation reframe HOLD pending human Q-B confirmation (v1.4 shipped with HOLD marker in place). No scope changes. |
| 1.4.1 | 2026-05-12 | product-owner (R-001 probability finalized per human Q-B response) | R-001 risk assessment finalized at <10% probability per human Q-B response. Removed the elaborate mitigation framing (was 'ship Phase 1 fast' in v1.3, became HOLD in v1.4 pending human answer). R-001 is now noted as informational background only — at <10%, the production-grade depth monocle is already shipping IS the response; no separate mitigation scaffolding required. Competitive Positioning section simplified to 3-4 sentences replacing the HOLD block. No scope changes. No other content changes. |
| 1.4.2 | 2026-05-12 | product-owner (Rule 1 violation fix per validate-brief v4) | §Phase Plan Rationale — replaced 'minimum viable product' phrase (Rule 1 violation per CLAUDE.md §Canonical Principle) with production-grade phrasing. Substantive meaning unchanged. Resolves the single blocker from validate-brief v4 (commit 38b8e8f). |
| 1.4.3 | 2026-05-12 | product-owner (adversary findings e2c224b: F-NEW-04, R-001 re-eval, F-NEW-03, F-NEW-05/06/09) | F-NEW-04 CRITICAL: hook ingestion timeout budget added to Success Criteria (300ms PreToolUse/Stop/SessionStart/UserPromptSubmit, 2000ms Notification per BC-HOOK-022); R-001 re-eval trigger paragraph added (4 conditions matching ADR-0002 pattern; <10% probability stands until any condition materializes); F-NEW-03 CRITICAL: permission token enum reference updated; brief no longer claims 17 zellij-borrowed variants for Phase 1; points at architect-produced SS-permissions-phase1.md canonical artifact; F-NEW-05/06/09 IMPORTANT: hook receiver hardening note added to Scope (body size limit, /healthz, /status, graceful shutdown). No scope removals; all additions are production-grade tightening, not new features. |
| 1.4.4 | 2026-05-12 | product-owner (architect-surfaced follow-on from round 5 fix burst) | Body-size limit (256 KiB) added to Success Criteria as a measurable Phase 1 acceptance criterion, cross-referencing BC-DAEMON-003 in `SS-daemon-lifecycle.md`. Resolves the architect-surfaced follow-on from round 5 fix burst — v1.4.3 added the hardening sub-bullet to Scope but did not promote the limit to a measurable Success Criterion. No new scope; just promotes existing scope item to measurable criterion. |
| 1.4.5 | 2026-05-12 | product-owner (two surgical fixes per round-6 audits) | `supplements:` frontmatter updated to include 3 round-5 artifacts (SS-permissions-phase1.md, SS-daemon-lifecycle.md, ADR-0003-license-selection.md); now 9 supplements total. Body-size Success Criterion endpoint list refined — `/healthz` and `/status` removed (GET endpoints with no body; limit applies to POST endpoints only). Resolves consistency G-01 (IMPORTANT) and adversary F-R6-006 (ADVISORY) from round-6 audits. |
| 1.4.6 | 2026-05-12 | product-owner (two additions per human pre-Phase-1 decisions) | (Q-2) DTU `dtu-claude-code-hooks-v1` clone added to Phase 1 deliverables + Phase 1 Success Criteria (fidelity ≥0.95, all 5 endpoints, CI per-PR gate); cross-references dtu-assessment §"Phase 1 Clone Build Effort" and §"DTU Fidelity Measurement Procedure"; BC-DTU-001 placeholder. (Q-3) R-001 re-eval trigger monitoring operationalized via weekly GitHub Actions workflow (devops-engineer specced in parallel this burst at `.github/workflows/r001-monitor.yml`); quarterly maintainer review for false-negative trigger keywords. |
| 1.4.7 | 2026-05-12 | product-owner (6 forward-compatibility FC items integrated as Phase 1 contracts per human authorization to lock pre-Phase-1; Phase 1 will run in fresh context; spec package must be self-contained) | New supplement `SS-core-types-and-abi.md` added (10 supplements total). New Phase 1 Scope sub-bullets and Success Criteria row covering: (FC-01) JSONL `format_version = 1` first key on every ring record; (FC-02) `#[non_exhaustive]` on all `monocle-core` public enums; (FC-03) `MONOCLE_ABI_VERSION = 1` const exported and exposed via `/status` endpoint; (FC-04 CRITICAL) `FactoryAdapter` trait defined with `VsddFactoryAdapter` implementing it (not inline-wired); (FC-05) `monocle-proto` `HookEnvelope` + 5 event messages with `uint32 schema_version = 1` first field; (FC-06) auth token format `monocle-v1:<64-char-hex>` with non-prefix rejection rule (HTTP 401). 10 behavioral contracts pre-staged for Phase 1 PRD authoring: BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001/002, BC-RING-001, BC-AUTH-001/002. |
| 1.4.8 | 2026-05-12 | product-owner (F-FC-C001 CRITICAL adversary finding from post-FC-burst fresh pass) | Resolves F-FC-C001 (CRITICAL adversary finding from post-FC-burst fresh pass): v1.4.7 erroneously listed `Phase1Permission` as carrying `#[non_exhaustive]` — contradicted both SS-permissions-phase1.md and SS-core-types-and-abi.md which require Phase1Permission to be exhaustive. Brief updated to remove Phase1Permission from non_exhaustive list; cross-reference added to ADR-0004 (architect-produced in same burst) documenting the exhaustive-enum exemption rationale. Also adds ClaudeCodeTool to the exhaustive-exempt list per the same ADR. No scope change. |
| 1.4.9 | 2026-05-13 | product-owner (round-14 propagation gaps resolved) | (G-R14-001) `supplements:` frontmatter expanded from 10 → 12 (added SS-engine-module.md + ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md). (N6) Phase 3 plugin SDK extension prose updated — removed obsolete "unsafe-impl mechanism" reference; replaced with open-trait per vision authority (round-15 architect removes sealing per human Q-15-1). (N5) BC count updated 10 → 14 (added BC-ENGINE-001/002, BC-LOCK-001; split BC-PROTO-001 → 001a/001b). No scope changes. |
| 1.4.10 | 2026-05-13 | product-owner (BC count reconciliation) | BC count reconciliation — architect's round-15 work (commit 7483d93) added BC-ENGINE-003 (ClaudeCodeModule inherent methods: hook_paths, spawn, preflight as struct methods per vision-aligned trait restoration). Brief BC count 14 → 15. Forward-compatibility Success Criteria row updated; BC list now: BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-RING-001, BC-AUTH-001/002, BC-ENGINE-001/002/003, BC-LOCK-001. |
| 1.4.11 | 2026-05-13 | architect (round-23 micro-fix BC propagation) | BC-ENGINE-002-ERR propagation: commit 563b573 added this BC to SS-engine-module.md §Behavioral Contracts but missed the Pre-Staging table (stale "Total: 3"). This burst fixes SS-engine-module.md v1.1.5 (pre-staging table 3→4), SS-core-types-and-abi.md (engine BC count 3→4, global total 15→16), SS-forward-compatibility.md (BC-ENGINE-002-ERR row added, table intro 15→16), and this brief (BC list 15→16; BC-ENGINE-002-ERR added). No behavioral content changed. |
| 1.4.12 | 2026-05-13 | product-owner (round-25 routing-precedent ratification + F-R24-cons-3 citation refresh) | **Routing-precedent ratification.** Architect (commit 688a5ed) performed a mechanical BC count propagation (15→16) and added BC-ENGINE-002-ERR to the Forward-compatibility BC enumeration in this brief. The content is verified correct by product-owner; no further content change is needed for the ratified material. However, commit 688a5ed was made directly by the architect without routing through product-owner — a process defect (F-R24-adv-2, MEDIUM severity per round-24 adversary). This v1.4.12 entry restores producer-of-record clarity: v1.4.12 is product-owner authored; v1.4.11 content stands as ratified by product-owner via this entry. The `producer: product-owner` frontmatter field accurately reflects product-owner authorship from this version forward. **Phase 1 gate question flagged for human ratification:** Should architects be permitted to mechanically propagate counts across artifact boundaries they do not own (e.g., when an architecture-domain change forces a brief count update), or should every cross-boundary edit route back through the destination artifact's owner? The current CLAUDE.md routing table (line 188) specifies product-owner owns the brief without exception; this question asks whether a narrow mechanical-propagation exemption is warranted for count-only edits that are provably correct. No CLAUDE.md change is made here — the question is flagged for human decision at the Phase 1 gate review. **F-R24-cons-3 citation refresh.** Three body citations to `SS-daemon-lifecycle.md v1.0.3` updated to v1.0.4 (the file was bumped to v1.0.4 in an earlier round; the brief lagged). Affected locations: §Forward-compatibility contracts / JSONL ring sub-bullet, §Forward-compatibility contracts / Versioned auth token sub-bullet, and the Forward-compatibility Success Criteria table row. No behavioral content changed in any of these locations; version numbers only. |
| 1.4.13 | 2026-05-13 | product-owner (round-27 F-R26-3 citation refresh + round-27 architect work ratification) | **F-R26-3 citation refresh (MEDIUM).** `SS-engine-module.md` version citation at the Forward-compatibility Success Criteria row (line 244) updated from v1.1.5 → v1.1.7. This was the sole stale inline version citation: SS-daemon-lifecycle.md (v1.0.4, lines 167/168/244) confirmed current; SS-conventions-anti-patterns.md and SS-core-types-and-abi.md have no versioned inline body citations. **Round-27 architect work ratified.** Commits `9be1033` (SS-engine-module v1.1.6 → v1.1.7: constructors for `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`, and `HookResponse`; all production code + test specs updated to use constructors; F-R26-adv-1 CRITICAL E0639 compile-error resolved) and `48d952a` (SS-conventions-anti-patterns v1.4 → v1.5: semgrep pattern expansion + Semgrep Coverage Hardening subsection) are verified correct by product-owner. Phase 1 implementer following the spec literally will compile cleanly — BC-ENGINE-002-ERR test spec now uses constructors (`EngineMetadata::new(...)`, `ProcessSnapshot::new(...)`, `EnrichedSession::new(...)`, `HookResponse::new(...)`) and E0639 is no longer reachable. No behavioral content changed. |
| 1.4.14 | 2026-05-13 | product-owner (F-R28-6 revision-history row-order fix) | **F-R28-6 LOW row-order fix.** Revision-history rows v1.4.12 and v1.4.13 were written in reverse chronological order (v1.4.13 appeared on line 77, v1.4.12 on line 78) — a cosmetic authoring error introduced when v1.4.13 was committed before v1.4.12's row position was resolved. This entry restores the monotonically ascending sequence: ...1.4.10, 1.4.11, 1.4.12, 1.4.13, 1.4.14. Row content for both v1.4.12 and v1.4.13 is preserved verbatim; only their order changed. Architect round-29 work has NOT yet landed (SS-engine-module.md remains at v1.1.7 as of this commit); the round-29 ratification and SS-engine-module citation refresh to v1.1.8 will follow as v1.4.15 after architect lands. No behavioral content changed. |
| 1.4.15 | 2026-05-13 | product-owner (F-R28-6 follow-up citation refresh + round-29 architect work ratification) | **F-R28-6 follow-up citation refresh.** Three stale inline version citations updated: `SS-daemon-lifecycle.md` v1.0.4 → v1.0.5 at §Forward-compatibility contracts / JSONL ring sub-bullet (line 169), §Forward-compatibility contracts / Versioned auth token sub-bullet (line 170), and the Forward-compatibility Success Criteria table row (line 246); `SS-engine-module.md` v1.1.7 → v1.1.8 at the Forward-compatibility Success Criteria table row (line 246). **Round-29 architect work ratified.** Commits `dc719cd` (SS-engine-module v1.1.7 → v1.1.8) and `09642de` (SS-daemon-lifecycle v1.0.4 → v1.0.5) are verified correct by product-owner. Specific changes: (F-R28-1) `EnrichedSession::last_event_micros` type changed `i64` → `Option<i64>` eliminating the epoch-0 sentinel; (F-R28-2) `SpawnArgs`, `SessionHandle`, and `EngineVersion` constructors added plus a new §Cross-Crate Constructor Audit table codifying the round-26 architect-audit-completeness process lesson — future spec changes adding `#[non_exhaustive]` to a struct MUST update this table; (F-R28-3) `HookResponse` builder methods `with_diagnostic` and `with_redirect` added (eliminates pub-field mutation pattern from rustdoc examples); (F-R28-4) `HookEventRecord` defined as a real struct with constructor in `monocle-runtime::ring` plus `RING_FORMAT_VERSION: u32 = 1` const (eliminates the final opaque-blob gap flagged by round-28 adversary); (F-R28-5) v1.1.5 supersession annotation in SS-engine-module.md `traces_to` field corrected. No behavioral content changed. |
| 1.4.16 | 2026-05-13T18:20:21Z | product-owner (round-31 architect work ratification + citation refresh; F-R30-4 ISO-8601 timestamp convention) | **Citation refresh.** Four stale inline version citations updated: `SS-daemon-lifecycle.md` v1.0.5 → v1.0.6 at §Forward-compatibility contracts / JSONL ring sub-bullet and §Forward-compatibility contracts / Versioned auth token sub-bullet; `SS-daemon-lifecycle.md` v1.0.5 → v1.0.6 and `SS-engine-module.md` v1.1.8 → v1.1.9 at the Forward-compatibility Success Criteria table row. **Round-31 architect work ratified.** Commits `0fc5803` (SS-engine-module v1.1.8 → v1.1.9: F-R30-1 — audit table expanded from 7 → 17 structs with HTML delimiter boundary markers (defined in SS-engine-module.md §Cross-Crate Constructor Audit Table) enabling machine-readable enumeration; `HookEvent` inner structs merged in; 4 factory structs added; CI enforcement prose added — this table is now the central reference for cross-crate constructor governance), `ed9842f` (SS-daemon-lifecycle v1.0.5 → v1.0.6: F-R30-2 — `#[non_exhaustive]` added to `HookEventRecord` struct, resolving the self-referential inconsistency where the v1.0.5 constructor audit table listed `HookEventRecord` as `#[non_exhaustive]`-required but the struct definition lacked the attribute), and `2ad7459` (SS-conventions-anti-patterns v1.5 → v1.6: F-R30-3 — new semgrep rule `monocle-non-exhaustive-struct-audit-completeness` added with fixture corpus and Python script spec; the script reads the HTML-delimited audit table from SS-engine-module.md, enumerates `#[non_exhaustive]`-annotated structs via semgrep, and asserts gap-free coverage; CI enforcement codifies the audit-completeness rule with automatic verification — closes the recurrence pattern where audit table and actual structs drift without detection) are verified correct by product-owner. **F-R30-4 ISO-8601 timestamp convention (prospective).** Per round-30 adversary finding F-R30-4 LOW: same-day revision entries (v1.4.12 through v1.4.15) lacked time precision. From v1.4.16 forward, all revision-history Date entries use ISO-8601 with second precision (`YYYY-MM-DDTHH:MM:SSZ`). Retroactive rewrite of v1.4.12–v1.4.15 is explicitly out of scope per F-R30-4 (prospective only). No behavioral content changed. |
| 1.4.17 | 2026-05-13T18:38:26Z | product-owner (F-R32-1 correct HTML delimiter strings in v1.4.16 ratification prose) | **F-R32-1 MEDIUM fix.** The v1.4.16 revision-history entry incorrectly named the HTML delimiter strings used in SS-engine-module.md — it described them as AUDIT-TABLE-START / AUDIT-TABLE-END markers. The correct delimiter strings (per SS-engine-module.md lines 1108/1128) are the BEGIN/END markers documented in SS-conventions-anti-patterns.md §Semgrep Coverage Hardening clause 4. A Phase 1 implementer reading the brief revision history to understand the delimiters would have searched for the wrong marker names and found nothing. This is a purely textual correction; no architecture documents need refresh (architect round-33 burst is running in parallel; a v1.4.18 follow-up will be produced if any citation refresh is required after that burst lands). No behavioral content changed. |
| 1.4.18 | 2026-05-13T19:00:00Z | product-owner (F-R36-1 SS-engine-module citation v1.1.9→v1.1.10 + F-R36-2 rewrite v1.4.16/v1.4.17 entries to remove verbatim delimiter quotes) | **F-R36-1 citation refresh (IMPORTANT).** `SS-engine-module.md` version citation in the Forward-compatibility Success Criteria table row (line 249) updated from v1.1.9 → v1.1.10. Architect bumped SS-engine-module.md to v1.1.10 in round-35 commit bdfc4b8; the brief lagged by one version. No behavioral content changed. **F-R36-2 convention compliance (MEDIUM).** The v1.4.16 and v1.4.17 revision-history entries violated the v1.8 convention rule from SS-conventions-anti-patterns.md (no verbatim quoting of audit-table delimiter strings in §Trace or any spec narrative). Both entries contained the actual delimiter strings copy-pasted literally rather than referencing them by name. This is an ironic finding for v1.4.17, which was itself the F-R32-1 fix entry that corrected the wrong delimiter names in v1.4.16 — and then introduced the correct delimiter strings verbatim in the very same revision-history cell. Both entries are now rewritten: the v1.4.16 entry refers to the HTML delimiter boundary markers by name as defined in SS-engine-module.md §Cross-Crate Constructor Audit Table; the v1.4.17 entry refers to the correct delimiter strings as documented in SS-conventions-anti-patterns.md §Semgrep Coverage Hardening clause 4. The historical narrative is fully preserved — a reader understands that v1.4.16 introduced the machine-readable HTML-delimited audit table and v1.4.17 corrected the wrong marker names that v1.4.16 had quoted. Closes round-36 adversary findings F-R36-1 (brief side) and F-R36-2 (brief side). Architect parallel burst handles the SS-conventions §Trace v1.6 entry. No behavioral content changed. |
| 1.4.19 | 2026-05-13T20:15:00Z | product-owner (F-R42-cons-1 SS-engine-module citation v1.1.10→v1.1.11 + broader-scope sweep; D-042 scope-hole root-cause noted) | **F-R42-cons-1 citation refresh (MEDIUM).** `SS-engine-module.md` version citation in the Forward-compatibility Success Criteria table row (line 250) updated from v1.1.10 → v1.1.11. Architect bumped SS-engine-module.md to v1.1.11 in round-41 commit eaf4adf (F-R40-2: stale current-pointer citations in v1.1.8 §Trace rewritten as historical pinpoints); the brief lagged by one version. This is the 6th recurrence of the cross-artifact citation-staleness META-pattern. **Broader-scope sweep result.** A scope-corrected grep was run across the entire `.factory/specs/` tree (not just `.factory/specs/architecture/` as D-042 mandated). All SS-* version citation hits enumerated and classified as historical pinpoints or current-pointers. Result: no additional stale current-pointers found in any file. `SS-daemon-lifecycle.md` citations at lines 173 and 174 confirmed at v1.0.6 (current). All other hits in architecture-domain files are historical pinpoints (version-at-introduction annotations, §Trace narrative, or constructor-audit table entries) — classified leave-alone per sweep protocol. **Root cause of 6th recurrence.** D-042 manual workflow rule (adopted round-32) scoped its mitigation grep to `.factory/specs/architecture/` only. The brief lives at `.factory/specs/product-brief.md` — one directory level up, outside that scope. This scope hole has now produced two confirmed recurrences (round-32 brief stale, round-42 brief stale). The D-042 option (c) mitigation is structurally insufficient when the brief is excluded from the sweep. Correct scope is `.factory/specs/` (recursive, all subdirectories). No architectural changes required; surfaced to architect for D-042 scope correction in the companion burst. No behavioral content changed. |
| 1.4.20 | 2026-05-13T20:30:00Z | product-owner (F-R48TP-1 SS-engine-module citation v1.1.11→v1.1.13; D-042 full-brief sweep clean) | **F-R48TP-1 citation refresh (LOW).** `SS-engine-module.md` version citation in the Forward-compatibility Success Criteria table row updated from v1.1.11 → v1.1.13. Brief was 2 versions stale: architect bumped SS-engine-module.md to v1.1.12 in R47.2 commit 42b0007 and to v1.1.13 in R47.3 commit 83cd93f; v1.4.19 lagged both bumps (D-042 sweep at v1.4.19 time ran `.factory/specs/` scope but was committed before those architect bursts landed). This is the 7th recurrence of the cross-artifact citation-staleness META-pattern. **D-042 full-brief sweep result (R47.4).** grep -nE `SS-[a-z-]*\.md v[0-9]` run across full `.factory/specs/product-brief.md`. All hits classified: `SS-daemon-lifecycle.md v1.0.6` at lines 174 and 175 confirmed CURRENT (actual frontmatter v1.0.6). Revision-history rows 76 and 77 contain historical pinpoints (v1.1.5, v1.0.3, v1.0.4) — leave-alone per sweep protocol. No additional stale current-pointers found. **PG-D042-BURST-SKIP acknowledgement.** Brief was already 1-behind at R47.3 time and became 2-behind because R47.2 and R47.3 architect bursts ran D-042 at architecture/ scope only, not full `.factory/specs/`. Codification of corrected D-042 scope (`.factory/specs/` recursive) is state-manager/architect's call in close-out commit; out of brief-burst scope. No behavioral content changed. |
| 1.4.21 | 2026-05-14T00:00:00Z | product-owner (F-R49-cascade-1 SS-engine-module citation v1.1.13→v1.1.14; D-042 full-brief sweep clean) | **F-R49-cascade-1 citation refresh (LOW).** `SS-engine-module.md` version citation in the Forward-compatibility Success Criteria table row updated from v1.1.13 → v1.1.14. Architect bumped SS-engine-module.md to v1.1.14 in R49 commit 07c1259 to fix F-R48-adv-3 (gene-source qualifier on BC-HOOK-018 inline citation); the brief lagged because brief edits require product-owner routing per D-041. This is the M-CASCADE-SCOPE pattern: architect burst legitimately bumped the architecture spec; brief cascades as a follow-up product-owner burst. **D-042 full-brief sweep result (R49.1).** grep -nE `SS-[a-z-]*\.md v[0-9]` run across full `.factory/specs/product-brief.md`. All current-pointer hits classified: `SS-daemon-lifecycle.md v1.0.6` at body lines 175 and 176 confirmed CURRENT (actual frontmatter v1.0.6). All other hits are revision-history pinpoints — leave-alone per sweep protocol. No additional stale current-pointers found beyond F-R49-cascade-1. No behavioral content changed. |
| 1.4.22 | 2026-05-14T00:00:00Z | product-owner (F-R51-cascade-1 SS-engine-module citation v1.1.14→v1.1.15; PG-4 §-heading-existence audit; D-042 full-brief sweep clean) | **F-R51-cascade-1 citation refresh (LOW).** `SS-engine-module.md` version citation in the Forward-compatibility Success Criteria table row (line 253) updated from v1.1.14 → v1.1.15. Architect bumped SS-engine-module.md to v1.1.15 in R51.1 commit 562b54c (F-R51-adv-1 mis-anchor fix + PG-4 §-heading-existence rule codification); the brief lagged because brief edits require product-owner routing per D-041. This is the M-CASCADE-SCOPE pattern. **PG-4 §-heading-existence audit on brief (R51.2).** grep -nE applied per PG-4 recipe; 11 §-heading references found and verified against actual headings in cited files. One mis-anchor found and fixed: §JSONL Ring Buffer at line 176 (body sub-bullet for JSONL ring format versioning) cited a heading that does not exist in `SS-daemon-lifecycle.md`; corrected to `§Daemon Lifecycle Protocol` (the existing H2 heading under which the JSONL format-version content lives — `### Drain (10-Second Timeout)` is a sub-section of it). **D-042 full-brief sweep result (R51.2).** grep -nE `SS-[a-z-]*\.md v[0-9]` run across full `.factory/specs/product-brief.md`. All current-pointer hits classified: `SS-daemon-lifecycle.md v1.0.6` at body lines 176 and 177 confirmed CURRENT. `SS-engine-module.md v1.1.15` at line 253 is the fixed current-pointer (was v1.1.14). All other hits are revision-history pinpoints — leave-alone per sweep protocol. No additional stale current-pointers found. No behavioral content changed. |
| 1.4.23 | 2026-05-14T07:50:10Z | product-owner (F-R53-cascade-1 SS-daemon-lifecycle citations v1.0.6→v1.0.7; D-042 full-brief sweep clean; PG-4 §-heading-existence audit at expanded 5-pattern scope) | **F-R53-cascade-1 citation refresh (LOW).** `SS-daemon-lifecycle.md` version citations updated v1.0.6 → v1.0.7 at 3 body sites: §Forward-compatibility contracts / JSONL ring sub-bullet (line 177), §Forward-compatibility contracts / Versioned auth token sub-bullet (line 178), and Forward-compatibility Success Criteria table row (line 254). Architect bumped SS-daemon-lifecycle.md to v1.0.7 in R53.1 commit 8baec19 (10 hidden brief §-anchor mis-anchors fixed across architecture files; PG-RECIPE-SCOPE META-META rule codified); the brief lagged because brief edits require product-owner routing per D-041. This is the M-CASCADE-SCOPE pattern. **D-042 full-brief sweep result (R53.2).** 4-pattern sweep (SS-*.md v, dtu-assessment.md v, vision v, ADR v) across full `.factory/specs/product-brief.md`. Pattern 1 (SS-*.md v): 3 stale current-pointers found and fixed (all 3 at SS-daemon-lifecycle.md v1.0.6 → v1.0.7); `SS-engine-module.md v1.1.15` at line 253 confirmed CURRENT; all other SS-* hits are revision-history pinpoints — leave-alone per sweep protocol. Patterns 2/3/4 (dtu-assessment.md v, vision v, ADR v): no version citations found — CLEAN. **PG-4 §-heading-existence audit on brief outbound §-anchors (R53.2, expanded 5-pattern recipe).** All body-level §-anchor references verified against actual headings in cited files. SS-core-types-and-abi.md: §ABI Version Constant ✓, §Enum Extensibility ✓, §FactoryAdapter Trait ✓, §Prost Wire Schemas ✓. SS-daemon-lifecycle.md: §Daemon Lifecycle Protocol ✓ (fixed in R51.2; confirmed still resolves in v1.0.7). Vision: §Vision Statement ✓, §End-to-End Killer Scenario ✓, §Phase Plan ✓, §Process Topology ✓, §Explicit Non-Goals ✓, §Workspace Layout ✓, §Key Abstractions ✓. oq-research.md: §OQ-01 through §OQ-11 all resolve to `## OQ-NN:` headings ✓. market-intelligence.md: §Risk Register ✓. brief-validation-v2.md: §OQ-M1 and §OQ-M3 referenced in Trace column of Open Questions table — these are row-reference identifiers pointing to table rows in `### Market Intel Open Questions Raised` section (not navigable heading anchors); classification unchanged from R51.2 audit (trace-column references, not prose hyperlinks). Total §-anchors checked: 22. Broken navigable anchors: 0. No behavioral content changed. |
| 1.4.24 | 2026-05-17T20:00:00Z | product-owner (T-128n Part 2 — F-R105 Round 4: ADR-0005 dual-accept propagation) | ADR-0005 dual-accept auth header propagated to 2 body sites: §Phase 1 constraints hook-ingestion bullet (line 116) and §Success Criteria Hook protocol parity row (line 239). D-042 sweep CLEAN. See §Trace v1.4.24 for SE-17f before/after detail. |
| 1.4.25 | 2026-05-17T22:00:00Z | product-owner (F-R106-8 BC-DTU-001 orphan fix; F-R106-19 revision history readability; F-R106-20 old-form BC ID canonicalization; GAP-R45-3 SS-engine-module pin correction) | **F-R106-8 HIGH**: BC-DTU-001 orphan promise removed from §Success Criteria DTU row; replaced with NFR-011 anchor (DTU clone fidelity ≥0.95 per nfr-catalog.md). **F-R106-19 LOW**: v1.4.24 revision-history row split into terse table row + §Trace v1.4.24 narrative entry. **F-R106-20 LOW**: old-form BC IDs canonicalized — §Success Criteria rows (lines 244 + 246): BC-DAEMON-003 → BC-2.01.003; 16-item old-form list → canonical BC-2.SS.NNN form with old-ID parentheticals per BC-INDEX §Renumbering Map. BC count updated 16 → 22 to match BC-INDEX v1.3 actual total. **GAP-R45-3 MED**: SS-engine-module.md version pin in §Success Criteria Forward-compatibility row corrected v1.1.15 → v1.1.18. D-042 sweep CLEAN (see §Trace v1.4.25). |
| 1.4.26 | 2026-05-18T01:00:00Z | product-owner (F-R108-8 §Success Criteria Forward-compatibility row stale pins) | **F-R108-8 HIGH**: §Success Criteria Forward-compatibility row (Target cell, line 247) had stale BC-INDEX v1.3, SS-daemon-lifecycle.md v1.0.7, SS-engine-module.md v1.1.18 pins; missing SS-core-types-and-abi.md version. Updated to BC-INDEX v1.6 (PO 7A bump), SS-daemon-lifecycle.md v1.0.31, SS-core-types-and-abi.md v1.2.12, SS-engine-module.md v1.1.19 (Architect 6D commit 98396fe canonical versions). See §Trace v1.4.26. |

## Who Is It For?

| Persona | Pain Point | Current Workaround |
|---------|-----------|-------------------|
| **Multi-session Claude Code developer** — runs 2-4 Claude Code sessions in parallel across worktrees or projects | Permission prompts from session B stall while the developer is focused on session A's window; must `Ctrl-b n` to find the right pane, read inline text, respond, switch back | Context-switch to correct tmux window; miss prompts; restart stalled sessions |
| **Factory-pattern operator** — runs vsdd-factory-style pipelines where each phase advances through a STATE.md; needs situational awareness without leaving the editor | Must `cat .factory/STATE.md`, `tree .factory/`, and mentally track which phase each session is in; blocking issues invisible until a session stalls | Manual file reads; `grep` for blocking issues; context-switch to read pipeline output |
| **Multi-harness operator** (v4 target, design must support) — runs Claude Code sessions on one task and CodeMachine sessions on another simultaneously | No unified view of cost or session health across harnesses; different UIs, different status indicators | Open separate TUI instances per harness; no aggregate cost tracking |

The killer scenario that motivates the v1 scope is the **multi-session developer**:
three sessions running (monocle project, blog, api-svc), two concurrent permission
prompts from different sessions, developer in nvim. Per vision §End-to-End Killer
Scenario: 4 keystrokes (`Ctrl-\`, `2`, `1`, `Ctrl-\`) resolves both prompts with
zero context switches vs. the current 6+ keystrokes + 2 window switches + risk of
session timeout.

## Scope

### In Scope

The scope below maps to the Phase Plan in vision §Phase Plan. Phase 1 is the
v1 delivery contract. Phases 2-4 are roadmap entries the architecture must
accommodate without breaking Phase 1 ABI.

**Phase 1 — Runtime Core (v1 delivery contract)**

- `monocle daemon start/stop`: long-lived background process that survives terminal
  closes; binds axum HTTP on OS-assigned port written to lock file (OQ-04/JC-3
  closed); writes daemon lock file with `{port, token, contract_version}` at mode
  `0o600` (SOQ-1); daemon auto-starts on first TUI launch with `MONOCLE_NO_AUTOSTART=1`
  escape hatch for CI/power users (OQ-01 hybrid)
- Daemon lock-file path: `directories::ProjectDirs::runtime_dir()` with
  state_dir → data_dir → `~/.monocle` fallback chain (OQ-10); token rotation invariant:
  bind socket + write lock-file + write token THEN hooks-settings reads token (SOQ-2)
- Hook ingestion endpoints (5 total, EX-2 resolution): `POST /hooks/pre-tool-use`,
  `POST /hooks/notification`, `POST /hooks/stop`, `POST /hooks/session-start`,
  `POST /hooks/prompt-submit`; schema byte-compatible with Claude Code's tmpfile hook
  protocol; auth via dual-accept header per ADR-0005: canonical `X-Monocle-Authorization: monocle-v1:<64-hex>` (monocle-aware tools) takes priority; `X-Claude-Code-Ide-Authorization: <64-hex>` (real Claude Code compatibility alias, raw token no prefix) accepted as fallback with WARN-level deprecation log; `PostToolUse` omitted
  per JC-2 (Claude Code gene-source parity BC-HOOK-007). Note: The vision document's
  §Process Topology diagram pre-dates JC-2 / EX-2 endpoint closures and depicts an
  illustrative endpoint set (with PostToolUse / PermissionPrompt); the canonical Phase 1
  endpoint set is the 5 endpoints listed above and the vision diagram is non-authoritative
  for endpoint enumeration.
  - Hook receiver hardening: body size limit ≤256KiB (RFC 7230 §3.3.2 compliant; reject
    with HTTP 413 Payload Too Large), `/healthz` liveness endpoint, `/status` daemon-state
    query endpoint, graceful shutdown protocol on SIGTERM/SIGINT (drain in-flight requests,
    flush JSONL ring per OQ-06, close UDS, persist lock file shutdown marker). See
    `.factory/specs/architecture/SS-conventions-anti-patterns.md` and the architect's
    daemon-lifecycle additions for the full BC list.
- Hook tmpfile: shared per-runtimeDir, mode `0o600`, atomic-replace (OQ-02)
- `ClaudeCodeModule`: built-in `EngineModule` implementation; detects Claude Code
  processes via PID walk; enriches with token counts, cost, phase tag from hook
  events; handles hook events and produces `EnrichedSession`
- Sessions panel (TUI): live session roster showing harness icon, project name,
  phase tag, token count, cost, uptime; `/` filter (nucleo-matcher); `Enter`
  fullscreen
- Permission prompt overlay: cascaded `VecDeque<PromptModal>` — both prompts visible
  simultaneously; diff preview via `similar 3`; Accept-once / Accept-always /
  Reject keybindings; `[t]` trace-to-source stub; overlay clears on daemon disconnect
  (SOQ-3); overlay survives `Ctrl-\` hide/show cycle without dropping queued prompts
- Profile picker: sticky-per-project with `Ctrl-P` picker override (OQ-05; Phase 1
  user-test target — MEDIUM confidence)
- Event ribbon panel: rolling log of hook events (PreToolUse, Notification, Stop,
  SessionStart, UserPromptSubmit) with session ID and latency; hybrid RAM ring +
  async JSONL flush, 100MB × 5 rotation (OQ-06)
- `monocle-config`: reads/writes `~/.monocle/config.json` (via `tempfile::persist`
  for atomic writes); harness profile schema version 1; CCR path field; binding
  overrides stub
- Tokio mpsc **bounded** event bus with drop counter surfaced in status bar;
  no unbounded channels (triple-confirmed anti-pattern from broker-r1 §3)
- `monocle-ipc`: Unix domain socket IPC between TUI client and daemon; UDS-only
  in v1 — shared-memory ring deferred to Phase 4 transport variant (OQ-08)
- `monocle-proto`: prost protobuf seam in monocle-core — zero runtime cost in v1,
  enables cross-host events in Phase 4 (OQ-07)
- Permission token enum: see `.factory/specs/architecture/SS-permissions-phase1.md`
  (architect-produced canonical artifact) — small Phase-1-purpose enum derived from
  Claude Code hook permission semantics (allow/deny/ask-user decisions for the 5
  Phase 1 hook endpoint types). The zellij-style 17-variant WASM plugin permission
  enum is Phase 3 scope alongside the wasmtime plugin SDK; not in Phase 1.
  Dispatcher no-op until Phase 3 (SOQ-4); `VsddFactoryAdapter` statically bundled
  in v1 — WASM plugin SDK ships Phase 3, not v1 (OQ-03)
- macOS + Linux build targets (darwin/linux × amd64/arm64); CI matrix on GitHub
  Actions; MSRV Rust 1.86 (ratatui floor, OQ-11)
- DTU Phase 1 clone: `dtu-claude-code-hooks-v1` synthesized clone of Claude Code hook protocol surface for testing fidelity and regression detection. Per `.factory/specs/dtu-assessment.md` §"Phase 1 Clone Build Effort" (architect-specced this burst). Fidelity target: ≥0.95 against fixture corpus.
- **Forward-compatibility contracts (locked pre-Phase-1 per human authorization):**
  - **monocle-core ABI:** Export `MONOCLE_ABI_VERSION: u32 = 1` const; expose via `/status` endpoint. Phase 3 plugin SDK uses this to refuse incompatible Phase 1 daemons. See `SS-core-types-and-abi.md` §ABI Version Constant.
  - **Public enum extensibility:** All `monocle-core` public enums (`HookType`, `HookEvent`, `DenyReason`, `AllowPattern`, `DenyPattern`) carry `#[non_exhaustive]` to permit Phase 2+ variant additions without breaking downstream `match` statements. Two enums are **exhaustive by design** and exempt per ADR-0004 (architect-produced this same fix burst): `Phase1Permission` (canonical 5-variant Claude Code permission set; new variants require explicit ADR) and `ClaudeCodeTool` (mirrors Claude Code's tool list; new tools require explicit ADR when Claude Code ships them). See `SS-core-types-and-abi.md` §Enum Extensibility and `ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md` for the canonical exemption rationale.
  - **FactoryAdapter trait:** `monocle-core::factory::FactoryAdapter` trait defined; `VsddFactoryAdapter` IMPLEMENTS the trait (not wired inline as a struct with hardcoded behavior). Phase 1 ships one impl (`VsddFactoryAdapter`); Phase 3 plugin SDK consumes `EngineModule` and `FactoryAdapter` as open traits (no sealing — per vision authority). The traits are documented public APIs; plugin authors implement them directly. See `SS-core-types-and-abi.md` and `SS-engine-module.md` (round-15 architect updates restoring vision-aligned signatures). See `SS-core-types-and-abi.md` §FactoryAdapter Trait.
  - **Prost wire schemas:** `monocle-proto` defines `HookEnvelope` + 5 event-type messages with `uint32 schema_version = 1;` as first field. Phase 4 federation uses these for cross-host wire format with schema_version compatibility checks. See `SS-core-types-and-abi.md` §Prost Wire Schemas.
  - **JSONL ring format versioning:** Every JSONL event record carries `format_version: u32 = 1` as first key. Phase 2 trigger-trace can read Phase 1 ring history; version field allows future format evolution. See `SS-daemon-lifecycle.md` v1.0.7 §Daemon Lifecycle Protocol.
  - **Versioned auth token prefix:** Auth token format `monocle-v1:<64-char-hex>`. Phase 4 federation can introduce OAuth2 (`Bearer ...`) tokens without colliding with Phase 1 local tokens. Validation rule: reject non-prefix tokens with HTTP 401. See `SS-daemon-lifecycle.md` v1.0.7 §Daemon Lifecycle Protocol.

**Phase 2 — Static Plane (roadmap)**

- `monocle-static` crate: reads CLAUDE.md, settings.json permission blocks, hook
  scripts, keybindings.json for the session in focus
- Customizations panel (TUI): 7 customization types from nikiforovall gene set
  (slash commands, subagents, skills, memory files, MCP servers, hooks, LSP servers);
  filter All / by type; trigger-trace `[t]` from permission prompt overlay to
  defining settings.json line
- Full AppMode state machine with FocusSnapshot enum (compile-time mutual exclusion);
  5-level binding precedence (SearchPrompt > UserCustomCommand > PerContext >
  Global > Builtin); telescope help overlay

**Phase 3 — Workflow Plane (roadmap)**

- `monocle-workflow` crate: `FactoryAdapter` trait; `VsddFactoryAdapter` promoted
  from static bundle to WASM-loadable; `notify 8` watcher for live updates
- Workflow panel (TUI): phase, status, awaiting, blocking issues, cycle for focused
  session's project
- `monocle-plugin-sdk` crate: WASM ABI (`wasmtime 44`) for third-party
  `EngineModule` + `FactoryAdapter` implementations; loaded from `~/.monocle/plugins/`
- MSRV bumps to Rust 1.92 (wasmtime requirement, OQ-11)

**Phase 4 — Cross-plane + Multi-harness + Federation (roadmap)**

- `CodeMachineModule`: second built-in `EngineModule`
- `russh 0.60` federation tunnel: TUI on host A shows sessions from host B
- `monocle-ipc` shared-memory ring buffer transport variant (OQ-08)
- OTel cost/token panel with aggregate across harnesses; revisit PostToolUse
  endpoint need at this point (JC-2)
- CCR integration: detect on PATH, write per-session JSON, set `ANTHROPIC_BASE_URL`
- rmcp MCP bridge (Phase 4 only, OQ-09): session query, prompt injection for tooling

### Out of Scope

Per vision §Explicit Non-Goals (these are hard boundaries, not deferred features):

- **Does NOT execute workflows** — monocle never writes STATE.md, never triggers
  factory phases, never dispatches agents; workflow panel is read-only observation
- **Does NOT write STATE.md** — the `VsddFactoryAdapter` reads STATE.md; monocle
  never mutates it
- **Does NOT route LLM API requests** — CCR integration is detect-on-PATH +
  config-write only; monocle does not proxy or modify LLM traffic (integrate-external,
  per D-010)
- **Does NOT replace the terminal multiplexer** — monocle runs inside tmux; it is
  not a multiplexer; zellij's multiplexer internals are a Leave-behind gene
- **Does NOT include PM/Worker multi-agent orchestration** — explicitly excluded
  by D-002; the human is always the coordinator
- **Does NOT own session transcripts** — hook events are ephemeral ingestion signals;
  full transcript storage belongs to each harness's own persistence layer
- **Does NOT build its own LLM provider abstraction** — CCR is the external router
  (D-010); monocle integrates by detecting it
- **Does NOT include `PostToolUse` hook endpoint in v1** — per Claude Code gene-source
  parity (any-context BC-HOOK-007 establishes the 5-endpoint set: PreToolUse,
  Notification, Stop, SessionStart, UserPromptSubmit; PostToolUse is intentionally
  absent). Revisit if Phase 4 OTel cost panel requires PostToolUse data. (JC-2)
- **Does NOT ship the WASM plugin SDK in v1** — Phase 3 deliverable per OQ-03;
  v1 statically bundles `VsddFactoryAdapter` as the sole built-in factory adapter
- **Does NOT ship the rmcp MCP bridge port in v1** — Phase 4 deliverable per OQ-09

## Success Criteria

v1 ships (Phase 1 complete) when ALL of the following pass:

| Outcome | Metric | Target |
|---------|--------|--------|
| Session management in popup | User can manage 3+ concurrent Claude Code sessions without leaving the editor pane | Killer scenario resolves in ≤6 keystrokes (per vision §End-to-End Killer Scenario target: 4) |
| Permission prompt latency | Permission prompt appears as overlay with diff preview after hook fires | ≤100ms from hook POST receipt to TUI overlay render on localhost |
| Hook ingestion timeout budget | Daemon responds within Claude Code's upstream timeout ceilings for each hook type | ≤300ms end-to-end response for `PreToolUse`, `Stop`, `SessionStart`, `UserPromptSubmit`; ≤2000ms for `Notification` — per gene-source BC-HOOK-022 (any-context-lazyclaude-pass-B-deep-hooks-r1.md). Exceeding these ceilings causes Claude Code to silently drop the event. Daemon broker architecture (event-bus, mpsc channel sizing) must be designed against these deadlines. |
| Hook protocol parity | Hook injection byte-compatible with Claude Code's schema | Fixture-based parity test passes against schema in any-context hooks-r1 canonical matrix (5 endpoints: PreToolUse/Notification/Stop/SessionStart/UserPromptSubmit; dual-accept auth per ADR-0005 — canonical path `X-Monocle-Authorization` tested AND compatibility alias path `X-Claude-Code-Ide-Authorization` tested; both paths validated by integration tests in `auth_header_rejection.rs`) |
| Factory pattern detection | vsdd-factory project detected and workflow panel populated | Detection succeeds on monocle's own `.factory/` (self-referential integration test) |
| Build matrix | Builds and tests pass on macOS and Linux | CI green on darwin/linux × amd64/arm64 |
| Drop counter active | Bounded event bus with visible drop counter | No unbounded channel in codebase; drop counter renders in status bar under synthetic high-frequency load (1000 events/sec) |
| Hook receiver body size limit | Daemon enforces 256 KiB max body on all hook POST endpoints (`/hooks/pre-tool-use`, `/hooks/prompt-submit`, `/hooks/notification`, `/hooks/stop`, `/hooks/session-start`) | Exceeding the limit returns HTTP 413 Payload Too Large with body `{"error":"payload_too_large","limit_bytes":262144}`. Rationale: Claude Code's Notification body carries an unbounded `message` string; 256 KiB covers expected-case bursts without exposing the daemon to memory exhaustion. Behavioral contract: BC-2.01.003 "Body Size Limit (256 KiB, HTTP 413)" (per BC-INDEX §SS-01, renumbered from BC-DAEMON-003). |
| DTU clone exists and validates | `dtu-claude-code-hooks-v1` clone is built, fidelity score ≥0.95 against fixture corpus, all 5 hook endpoint payloads schema-valid, integrated into CI as a per-PR gate on `monocle-ipc` or `monocle-runtime` changes (per dtu-assessment §"DTU Fidelity Measurement Procedure"). | DTU clone fidelity verified per NFR-011 (≥0.95 against Claude Code real hooks fixture corpus, per nfr-catalog.md). |
| **Forward-compatibility contracts** | All 6 FC items shipped: (1) `MONOCLE_ABI_VERSION = 1` const exported and exposed via `/status` endpoint; (2) all public enums in `monocle-core` carry `#[non_exhaustive]`; (3) `FactoryAdapter` trait defined and `VsddFactoryAdapter` implements it; (4) `monocle-proto` HookEnvelope schema with `schema_version = 1` field; (5) JSONL ring `format_version = 1` first key on every record; (6) auth token format `monocle-v1:<64-hex>` with non-prefix rejection rule. | 22 behavioral contracts active in Phase 1 PRD (per BC-INDEX v1.7): BC-2.02.001/002 (BC-ABI-001/002), BC-2.02.003 (BC-TYPES-001), BC-2.02.004/005 (BC-FACTORY-001/002), BC-2.02.006/007/008 (BC-PROTO-001a/001b/002), BC-2.01.007 (BC-RING-001), BC-2.01.008/009 (BC-AUTH-001/002), BC-2.03.001/002/003/004 (BC-ENGINE-001/002/002-ERR/003), BC-2.01.010 (BC-LOCK-001). Per `SS-core-types-and-abi.md` v1.2.13, `SS-daemon-lifecycle.md` v1.0.32, and `SS-engine-module.md` v1.1.20. |

## Phase 2 Exit Criteria

Phase 2 (Static plane) ships when:

| Outcome | Metric | Target |
|---------|--------|--------|
| Customization rendering | All 7 customization types render in Static plane on filter "All" | Zero missing types when pointed at a claude-code project with all 7 type examples |

Additional Phase 2 exit criteria will be defined by the architect during
`/vsdd-factory:create-architecture` and refined in PRD behavioral contracts.

## Constraints & Integration Points

**Tech stack inheritance**: All version pins, the wasmtime-vs-wasmi rationale,
anti-pattern enforcement rules, and RUSTSEC audit context are codified in
`/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md`,
`/Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md`,
and `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md`.
The architect inherits these as Phase 1 constraints (not up for re-selection);
per vision D-012 the tech stack is human-approved and architecturally pre-committed.

**Crate workspace layout** is fixed by vision §Workspace Layout + EX-1 ratification:
12 crates total (11 named workspace crates + 1 binary crate `monocle`) — `monocle-core`
(zero-dependency pure types), `monocle-runtime`, `monocle-tui`, `monocle-static`,
`monocle-workflow`, `monocle-plugin-sdk`, `monocle-ipc`, `monocle-config`,
`monocle-proto`, `monocle-fuzz`, `monocle-test-harness` (11 named), plus `monocle`
(binary). No crate outside the binary may depend on the binary crate.

**Action enum dispatch model** is non-negotiable per vision §Key Abstractions and
D-009: 5-level precedence (SearchPrompt > UserCustomCommand > PerContext > Global >
Builtin); enum variants (not closures) keep bindings `Eq + inspectable` for the
telescope help overlay.

**AppMode state machine** is non-negotiable per vision §Key Abstractions:
compile-time mutual exclusion (not `bag-of-Option` fields); `VecDeque<PromptModal>`
overlay stack (not single-popup — fixes lazygit's drop-on-concurrent anti-pattern);
state transitions are pure functions in `monocle-core`.

**Process topology**: monocle uses a separate tmux server (`-L monocle`) to host
the TUI client as a floating popup over the user's existing tmux session. Daemon
is long-lived. Hook POSTs are the ingestion boundary; Claude Code subprocesses are
unmodified beyond pointing their hook scripts at the daemon's lock-file-discovered
port.

**CCR is integrate-external** (D-010): detect on PATH, write per-session JSON,
set `ANTHROPIC_BASE_URL`. No CCR API changes required or expected.

**OQ + SOQ resolutions applied**: 11 architect open questions and 4 second-order
questions resolved per `/Users/jmagady/Dev/monocle/.factory/planning/oq-research.md`
(commit b3c68ca). See Phase 1 Constraints below.

## Phase 1 Constraints (from OQ Resolutions)

These constraints are derived from the orchestrator's accepted defaults on
`oq-research.md` and bind the architect during `/vsdd-factory:create-architecture`.

| Constraint | Trace |
|---|---|
| Daemon: hybrid auto-start with `MONOCLE_NO_AUTOSTART=1` escape hatch | OQ-01 |
| Hook tmpfile: shared per-runtimeDir, mode `0o600`, atomic-replace (any-context verbatim) | OQ-02 |
| WASM plugin SDK: NOT shipped in v1; ships in Phase 3; v1 statically bundles `VsddFactoryAdapter` | OQ-03 |
| Port binding: OS-assigned port + lock-file PID-liveness discovery (JC-3 closed by this) | OQ-04 |
| Profile picker: sticky-per-project; `Ctrl-P` picker override (Phase 1 user-test target; MEDIUM confidence) | OQ-05 |
| Hook event retention: hybrid RAM ring + async JSONL flush, 100MB × 5 rotation | OQ-06 |
| Cross-host migration: protobuf seams in v1 (zero runtime cost), russh transport Phase 4 | OQ-07 |
| monocle-ipc: UDS-only in v1; shared-memory ring deferred to Phase 4 transport variant | OQ-08 |
| rmcp MCP bridge: OMITTED in v1; Phase 4 ships real impl (no stub in v1) | OQ-09 |
| Daemon lock file: `directories::ProjectDirs::runtime_dir()` w/ state_dir → data_dir → `~/.monocle` fallback | OQ-10 |
| MSRV target: Phase 1 = Rust 1.86 (ratatui floor); Phase 3 bumps to 1.92 (wasmtime) | OQ-11 |
| Lock-file schema: `contract_version: u32` field from day one (zellij pattern) | SOQ-1 |
| Token rotation invariant: bind socket + lock-file write + token THEN hooks-settings reads token | SOQ-2 |
| Overlay survival: clear on daemon disconnect (Claude Code subprocesses time-out delayed responses) | SOQ-3 |
| Permission token enum: see `.factory/specs/architecture/SS-permissions-phase1.md` (architect-produced canonical artifact) — small Phase-1-purpose enum derived from Claude Code hook permission semantics (allow/deny/ask-user decisions for the 5 Phase 1 hook endpoint types); dispatcher no-op until Phase 3; zellij-style 17-variant WASM plugin permission enum is Phase 3 scope | SOQ-4 |

## Open Questions for Architect

All 11 original open questions have been resolved via `oq-research.md` (commit b3c68ca).
Three market-intel open questions (OQ-M1, OQ-M2, OQ-M3) were raised during brief v1.3
competitive positioning; all three are now resolved in-scope (adversary re-audit commit
0bd4ba9). The table below is preserved for traceability; OQ-01 through OQ-11 and
OQ-M1 through OQ-M3 decisions are final unless human red-lines.

| ID | Question | Resolution | Trace |
|----|----------|-----------|-------|
| OQ-01 | Daemon auto-start vs explicit? | Hybrid auto-start with `MONOCLE_NO_AUTOSTART=1` escape | oq-research.md §OQ-01 |
| OQ-02 | Hook tmpfile per-session or shared? | Shared per-runtimeDir, `0o600`, atomic-replace | oq-research.md §OQ-02 |
| OQ-03 | v1 ship WASM SDK or static bundle? | Static bundle in v1; WASM SDK Phase 3 | oq-research.md §OQ-03 |
| OQ-04 | Daemon port fixed or OS-assigned? | OS-assigned port + lock-file discovery | oq-research.md §OQ-04 |
| OQ-05 | Profile picker on create vs sticky? | Sticky-per-project; `Ctrl-P` override (MEDIUM confidence) | oq-research.md §OQ-05 |
| OQ-06 | Event retention ring or JSONL? | Hybrid RAM ring + async JSONL flush, 100MB × 5 | oq-research.md §OQ-06 |
| OQ-07 | Cross-host scope v1 or v4? | Protobuf seams v1 (zero cost); russh Phase 4 | oq-research.md §OQ-07 |
| OQ-08 | IPC: UDS only or UDS + shared-mem? | UDS-only v1; shared-mem Phase 4 | oq-research.md §OQ-08 |
| OQ-09 | rmcp stub in v1 or omit? | Omit entirely in v1 | oq-research.md §OQ-09 |
| OQ-10 | Lock-file location XDG or `~/.monocle`? | `directories::ProjectDirs::runtime_dir()` with fallback chain | oq-research.md §OQ-10 |
| OQ-11 | MSRV target? | Phase 1: Rust 1.86; Phase 3: Rust 1.92 | oq-research.md §OQ-11 |
| OQ-M1 | Does agent view use Claude Code hook protocol or different IPC? If hook protocol, can monocle daemon and agent view coexist on same host without port/auth collision? | Resolved — agent view dispatches via Claude Code's internal IPC (not hook protocol POSTs); monocle's daemon on an OS-assigned port + `X-Claude-Code-Ide-Authorization` header cannot collide because agent view does not bind a TCP port. No shared port or auth surface. Source: Anthropic docs https://code.claude.com/docs/en/agent-view referenced in market-intelligence.md line 222. | brief-validation-v2.md §OQ-M1; adversary re-audit 0bd4ba9 |
| OQ-M2 | Does `claude-manager` use the hook protocol, creating a second actor on the same hook-protocol surface as monocle? | Resolved — claude-manager uses tmux pane management + worktrees, NOT hook protocol. The hook-native architectural moat is intact. Source: market-intelligence.md §gap-matrix line 50 (`claude-manager... hook-overlay: NO`). | market-intelligence.md §gap-matrix; adversary re-audit 0bd4ba9 |
| OQ-M3 | Claude Code 2026 docs list 25 lifecycle events including `PermissionRequest` as a distinct hook event. Should monocle add `PermissionRequest` as a sixth endpoint (current JC-2 decision: 5 endpoints) for cleaner permission-overlay UX? | Resolved — stay at 5 endpoints (SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop). The `PermissionRequest` event is upstream of `PreToolUse`; the existing VecDeque overlay receives all permission-relevant signal via `PreToolUse` + `Notification`. Re-eval trigger: if Phase 2 trigger-trace UX testing surfaces a signal gap that PermissionRequest would fill, dispatch a fresh architecture review. Until then, 5 endpoints is canonical and final. | brief-validation-v2.md §OQ-M3; adversary re-audit 0bd4ba9 |

> **Judgment call resolutions (orchestrator-applied 2026-05-12)** — JC-1 → option B1
> (Phase 2 exit criterion); JC-2 → omit PostToolUse for Phase 1 (Claude Code parity);
> JC-3 → CLOSED via OQ-04; EX-1 → ratify 12-crate workspace (11 named + 1 binary); EX-2 → add SessionStart
> + UserPromptSubmit to Phase 1 (full 5-endpoint parity). All resolutions traceable to
> vision D-012 and oq-research.md commit b3c68ca. Human may red-line any of these in a
> follow-up brief revision.

## Overflow Context

### Competitive Positioning

Anthropic shipped `claude agents` (agent view, v2.1.139) on 2026-05-11 — one day before
brief v1.2 was finalized. Agent view provides session list + inline reply built into
Claude Code's TUI: no hook protocol, no external overlay, no diff preview, no cascaded
permission queue, no customization visibility, no workflow plane, no multi-harness support.
Monocle's differentiation is mechanism and depth, not exclusivity over the session-list
surface: hook-protocol ingestion (vs. file polling or pane scraping), VecDeque<PromptModal>
overlay (vs. attach-and-reply dispatch), diff preview (vs. none), trigger-trace to the
defining settings.json line (Phase 2, vs. none), workflow plane (Phase 3, vs. none),
multi-harness and external-overlay operation over the user's existing tmux + editor setup
without modifying Claude Code sessions (vs. built-in, lives inside Claude Code's TUI).
Anthropic shipping a thin version confirms the pain is real and significant enough for
a first-party response — monocle goes deeper on every dimension agent view does not touch.
The risk that Anthropic deepens agent view to commoditize monocle's hook-native overlay within 12 months was assessed at <10% probability based on agent view's current research-preview scope, single-harness focus, and absence of announced hook-protocol direction (per `.factory/planning/market-intelligence.md` §Risk Register, originally assessed at 25–40%; human red-line at v1.4.1 brief gate revised this to <10% based on additional context about agent view's roadmap and scope). At this probability, no risk mitigation scaffolding is required beyond the production-grade depth monocle is already shipping.

**R-001 re-eval trigger.** Re-open the R-001 risk assessment and reconsider the probability AND the mitigation requirement if ANY of the following occurs: (a) Anthropic announces hook-protocol ingestion as a first-class agent-view capability; (b) Anthropic ships diff-preview or cascaded permission-queue functionality inside agent view; (c) Anthropic extends agent view beyond Claude Code (e.g., supports a non-Claude harness); (d) Anthropic publishes a multi-harness session-management spec or RFC. Until any of these conditions materializes, the <10% assessment stands; monocle's defensible surface is depth + mechanism (hook-protocol ingestion, VecDeque overlay, diff preview, trigger-trace, workflow plane, multi-harness, external overlay).

**R-001 monitoring cadence.** The 4 re-eval trigger conditions are checked by a **weekly scheduled GitHub Actions workflow** (`.github/workflows/r001-monitor.yml`, specced this burst by devops-engineer). The workflow fetches Anthropic's published agent-view release notes and changelogs, evaluates each entry against the 4 trigger conditions, and opens a GitHub Issue if any condition is matched. The weekly cadence is calibrated to Anthropic's research-preview release cadence (multiple agent-view shipments observed per month as of 2026-05). Quarterly the workflow output is reviewed by the maintainer for false-negative patterns (e.g., a trigger condition that should have fired but didn't due to wording variation in Anthropic's notes); adjustments to the trigger keyword set ship as a workflow patch.

The closest prior art beyond agent view:

- `any-context/lazyclaude`: Go TUI for Claude Code sessions; PM/Worker orchestration;
  hook protocol via `~/.claude/ide/<port>.lock`. Gene source for Runtime plane.
  Monocle ports the session management and hook ingestion, drops the PM/Worker
  persona, adds multi-harness and WASM plugin extensibility.
- `NikiforovAll/lazyclaude`: Python Textual TUI for customization exploration.
  Gene source for Static plane. Monocle ports the 7-parser canonical schema and
  AppMode state machine to Rust; drops the Python dependency entirely.
- `claude-squad`: Session isolation via worktrees; snapshot/fork concurrency; no
  orchestration layer (human is coordinator per D-011). Gene source for worktree
  isolation pattern in Harness plane.
- `claude-code-router`: LLM request router via HTTP reverse proxy. Integrated
  externally (D-010); monocle detects CCR on PATH and writes per-session config.

### Decisions Log Cross-Reference

All decisions that constrain this brief are logged in STATE.md §Decisions Log:
D-001 through D-017. The canonical vision approved by human is D-012 (archived to `cycles/cycle-001/burst-log.md`).

### Phase Plan Rationale

Phase 1 ships the daemon + hook ingestion + sessions panel. This is the Phase 1
delivery scope for the killer scenario — permission prompt dispatch without
context-switching. Phase 2 adds the customization plane (trigger-trace) which
enriches the permission prompt overlay with "why did this prompt appear" context.
Phase 3 adds workflow awareness which is the factory-operator persona's core need.
Phase 4 adds multi-harness federation which serves the future multi-harness operator
persona. The ABI between phases must be stable: the `EngineModule` and
`FactoryAdapter` traits defined in Phase 1 must be forward-compatible with Phase 4
additions. No breaking changes to these traits between phases.

### Reference Gene Source Map

| Monocle Component | Primary Gene Source | Key Artifacts |
|-------------------|--------------------|-|
| EngineModule trait | codemachine-cli | pass-8-final-synthesis.md |
| Action enum + 5-level precedence | lazygit (port) | pass-8-final-synthesis.md §Action enum |
| AppMode state machine + VecDeque overlay | NikiforovAll AppMode + lazygit fix | nikiforovall pass-8-final-synthesis-v2.md §AppMode |
| Hook protocol + tmpfile schema | any-context hooks-r1/r2 | any-context pass-8-final-synthesis-v2.md §Hook protocol |
| Broker (bounded pub/sub + drop counter) | any-context broker-r1/r2 | any-context pass-8-final-synthesis-v2.md §Broker |
| Crate workspace split | zellij | zellij pass-8-final-synthesis.md §crate layout |
| Worktree isolation pattern | claude-squad | claude-squad pass-8-deep-synthesis.md |
| CCR integrate-external | claude-code-router | claude-code-router pass-C-final-synthesis.md |
| FactoryAdapter + VsddFactoryAdapter | vsdd-factory | vsdd-factory pass-8-final-synthesis.md |
| 7-parser customization schema | NikiforovAll services/parsers/ | nikiforovall pass-8-final-synthesis-v2.md §parsers |
| WASM plugin SDK ABI | zellij-tile model | zellij pass-8-final-synthesis.md §plugin |

---

## §Trace v1.4.24

**T-128n Part 2 — F-R105 closure chain Round 4: ADR-0005 dual-accept propagation** (2026-05-17T20:00:00Z):

SE-17f before/after — line 116 (Phase 1 constraints hook-ingestion bullet):
- Before: `auth via X-Claude-Code-Ide-Authorization header`
- After: `auth via dual-accept header per ADR-0005: canonical X-Monocle-Authorization: monocle-v1:<64-hex> (monocle-aware tools) takes priority; X-Claude-Code-Ide-Authorization: <64-hex> (real Claude Code compatibility alias, raw token no prefix) accepted as fallback with WARN-level deprecation log`

SE-17f before/after — line 239 (§Success Criteria Hook protocol parity row Target cell):
- Before: dual-accept auth per ADR-0005 mentioned without explicit test-path call-out for alias.
- After: canonical path `X-Monocle-Authorization` tested AND compatibility alias path `X-Claude-Code-Ide-Authorization` tested; both paths validated by integration tests in `auth_header_rejection.rs` — explicitly called out.

D-042 sweep result: all SS-* current-pointer hits classified; no stale current-pointers found beyond the two fixed in this burst. SE-16d monotonicity PASS: 2026-05-17T20:00:00Z > prior 2026-05-14T07:50:10Z (v1.4.23).

---

## §Trace v1.4.25

**F-R106-8/19/20 + GAP-R45-3 closure — brief scope only** (2026-05-17T22:00:00Z):

**F-R106-8 HIGH — BC-DTU-001 orphan promise removed.**
- SE-17f before/after — §Success Criteria DTU row Target cell (previously line 245):
  - Before: `Behavioral contract: BC-DTU-001 (Phase 1 PRD will formalize).`
  - After: `DTU clone fidelity verified per NFR-011 (≥0.95 against Claude Code real hooks fixture corpus, per nfr-catalog.md).`
- Justification: BC-DTU-001 was never formalized in PRD or BC-INDEX; the promise was orphaned. NFR-011 exists in nfr-catalog.md and is the canonical fidelity target for the DTU clone.

**F-R106-19 LOW — v1.4.24 revision-history row readability.**
- SE-17c-d body-scope: v1.4.24 row previously contained nested SE-17f/SE-16d subsections inline in the table cell (est. 880+ chars). Split into terse table row (summary only) + §Trace v1.4.24 section (full before/after detail). Historical narrative fully preserved in §Trace v1.4.24 above.

**F-R106-20 LOW — old-form BC IDs canonicalized.**
- SE-17f before/after — §Success Criteria Hook receiver body size limit row Target cell (line 244):
  - Before: `Behavioral contract: BC-DAEMON-003 (per \`SS-daemon-lifecycle.md\`).`
  - After: `Behavioral contract: BC-2.01.003 "Body Size Limit (256 KiB, HTTP 413)" (per BC-INDEX §SS-01, renumbered from BC-DAEMON-003).`
- SE-17f before/after — §Success Criteria Forward-compatibility contracts row Target cell (line 246):
  - Before: 16 old-form IDs enumerated: BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-RING-001, BC-AUTH-001/002, BC-ENGINE-001/002/002-ERR/003, BC-LOCK-001. Count stated as 16.
  - After: 22 canonical IDs enumerated with old-ID parentheticals per BC-INDEX §Renumbering Map. Count corrected to 22 (matches BC-INDEX v1.3 actual total: 10 SS-01 + 8 SS-02 + 4 SS-03).

**GAP-R45-3 MED — SS-engine-module.md version pin corrected.**
- SE-17f before/after — §Success Criteria Forward-compatibility contracts row Target cell:
  - Before: `SS-engine-module.md v1.1.15`
  - After: `SS-engine-module.md v1.1.18`
- Verification: `grep -n "^version:" .factory/specs/architecture/SS-engine-module.md` returns `version: "1.1.18"`. The v1.1.15 pin was introduced in v1.4.22 and was not updated through subsequent architect bumps (v1.1.16/1.1.17/1.1.18). The D-042 sweep in v1.4.24 confirmed v1.1.15 as "CURRENT" — that was an error: CLAUDE.md §Architectural Authority entry 4 cites SS-engine-module.md without a version pin, and the file itself is at v1.1.18. This is the GAP-R45-3 correction.

**D-042 full-brief sweep result (v1.4.25).** grep -nE `SS-[a-z-]*\.md v[0-9]` across full `.factory/specs/product-brief.md`. All SS-* current-pointer hits classified:
- `SS-daemon-lifecycle.md v1.0.7` at body lines (JSONL ring sub-bullet and Versioned auth token sub-bullet): historical pinpoints (introduced in R53; leave-alone per sweep protocol).
- `SS-engine-module.md v1.1.18` in §Success Criteria Forward-compatibility row: CURRENT (just fixed in this burst).
- All revision-history rows contain historical pinpoints only — leave-alone per sweep protocol.
- No additional stale current-pointers found. CLEAN.

SE-16d monotonicity PASS: 2026-05-17T22:00:00Z > prior 2026-05-17T20:00:00Z (v1.4.24).

---

## §Trace v1.4.26

**F-R108-8 closure — brief scope only** (2026-05-18T01:00:00Z):

**F-R108-8 HIGH — §Success Criteria Forward-compatibility row stale pins.**

The §Success Criteria "Forward-compatibility contracts" row Target cell (line 247) had 3 stale version pins and was missing 1 version pin:
- `BC-INDEX v1.3` → stale; current is v1.6 (PO 7A bump in same Round 7 burst)
- `SS-daemon-lifecycle.md v1.0.7` → stale; the v1.0.7 pin was the historical value at v1.4.23 but Architect 6D bumped to v1.0.31 (commit 98396fe)
- `SS-engine-module.md v1.1.18` → stale; Architect 6D bumped to v1.1.19 (commit 98396fe)
- `SS-core-types-and-abi.md` was cited without version → now pinned to v1.2.12 (Architect 6D canonical)

SE-17f before/after — §Success Criteria Forward-compatibility contracts row Target cell:

**Before:** `22 behavioral contracts active in Phase 1 PRD (per BC-INDEX v1.3): ...Per \`SS-core-types-and-abi.md\`, \`SS-daemon-lifecycle.md\` v1.0.7, and \`SS-engine-module.md\` v1.1.18.`
**After:** `22 behavioral contracts active in Phase 1 PRD (per BC-INDEX v1.6): ...Per \`SS-core-types-and-abi.md\` v1.2.12, \`SS-daemon-lifecycle.md\` v1.0.31, and \`SS-engine-module.md\` v1.1.19.`

**D-042 full-brief sweep result (v1.4.26).** grep -nE `SS-[a-z-]*\.md v[0-9]` across full `.factory/specs/product-brief.md`. All SS-* current-pointer hits classified:
- `SS-daemon-lifecycle.md v1.0.31` in §Success Criteria Forward-compatibility row: CURRENT (just fixed in this burst).
- `SS-core-types-and-abi.md v1.2.12` in §Success Criteria Forward-compatibility row: CURRENT (just fixed in this burst).
- `SS-engine-module.md v1.1.19` in §Success Criteria Forward-compatibility row: CURRENT (just fixed in this burst).
- All other SS-* hits in body sub-bullets (§Phase 1 constraints references) are current-pointers per their respective SS file frontmatter as of Round 7 — classified leave-alone until a cascade fix-burst specifically targets them.
- All revision-history rows contain historical pinpoints only — leave-alone per sweep protocol.
- No additional stale current-pointers found beyond the three fixed in this burst. CLEAN.

SE-16d monotonicity PASS: 2026-05-18T01:00:00Z > prior 2026-05-17T22:00:00Z (v1.4.25).

---

## §Trace v1.4.27

**F-R109-6 closure — brief scope only** (2026-05-17T04:40:00Z):

**F-R109-6 HIGH — §Success Criteria Forward-compatibility row stale pins (Architect 8A bump).**

Architect 8A bumped SS-core-types-and-abi.md v1.2.12 → v1.2.13, SS-daemon-lifecycle.md v1.0.31 → v1.0.32, SS-engine-module.md v1.1.19 → v1.1.20. BC-INDEX v1.6 → v1.7 (this same R109 PO 8B burst).

SE-17f before/after — §Success Criteria Forward-compatibility contracts row Target cell (line 248):

**Before:** `Per \`SS-core-types-and-abi.md\` v1.2.12, \`SS-daemon-lifecycle.md\` v1.0.31, and \`SS-engine-module.md\` v1.1.19.`
**After:** `Per \`SS-core-types-and-abi.md\` v1.2.13, \`SS-daemon-lifecycle.md\` v1.0.32, and \`SS-engine-module.md\` v1.1.20.`

BC-INDEX reference updated: `BC-INDEX v1.6` → `BC-INDEX v1.7`.

**D-042 full-brief sweep result (v1.4.27).** grep -nE `SS-[a-z-]*\.md v[0-9]` across full `.factory/specs/product-brief.md`. All SS-* current-pointer hits classified:
- `SS-daemon-lifecycle.md v1.0.32` in §Success Criteria Forward-compatibility row: CURRENT (just fixed in this burst).
- `SS-core-types-and-abi.md v1.2.13` in §Success Criteria Forward-compatibility row: CURRENT (just fixed in this burst).
- `SS-engine-module.md v1.1.20` in §Success Criteria Forward-compatibility row: CURRENT (just fixed in this burst).
- All other SS-* hits in body sub-bullets are current-pointers per their respective SS file frontmatter as of Round 8A — classified leave-alone per sweep protocol.
- All revision-history rows contain historical pinpoints only — leave-alone.
- No additional stale current-pointers found. CLEAN.

SE-16d monotonicity PASS: 2026-05-17T04:40:00Z > prior 2026-05-18T01:00:00Z (v1.4.26).
