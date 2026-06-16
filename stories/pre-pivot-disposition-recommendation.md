---
document_type: recommendation
level: planning
version: "1.1"
status: ratified
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
ratified_by: "Joshua Magady"
ratification_date: "2026-06-16"
ratification_decision: "D-315"
phase: phase-2-burst-d
traces_to: ".factory/STATE.md D-305, D-315"
ratification_required: false
bulk_archive_blocked: false
---

# Pre-Pivot Story Disposition Recommendation

> **RATIFIED 2026-06-16 (D-315) by Joshua Magady.**
> **Decision: Keep all 3 active (S-032, S-DAEMON-WIRE-FIX-001, S-PHASE-3-PREP remain ACTIVE).**
> **32 pre-pivot done stories (S-001..S-031 + S-DTU-001) classified DONE-HISTORICAL.**
> **0 stories archived. 0 stories retired. Bulk-archive: NOT PERFORMED (correctly).**
> **No story file status fields changed. No STORY-INDEX changes. No sprint-state changes.**

---

> **D-305 CONSTRAINT (SUPERSEDED by D-315 ratification):** This document was created with a
> D-305 constraint requiring human ratification before ANY disposition is executed. Ratification
> was provided by Joshua Magady on 2026-06-16 (D-315). The ratification decision confirmed the
> recommendations exactly as written: keep all 3 active; 32 done-historical; 0 archive/retire.

---

## Decisions Required From Human

Review the table in §Summary Table below and confirm or override each recommended
disposition. The following items require explicit judgment:

1. **S-032 (KEEP-ACTIVE vs RETIRE):** S-032 discharges a live CONTRACT GAP in production
   code (`lifecycle.rs` timestamp_micros obligation from BC-2.05.004 PC-2/INV-4). The v1A
   scope expands IPC (SS-05 gains BC-2.05.009/010/011 in S-046/S-047), making the daemon
   event-bus fan-out (S-032) directly relevant to the broader session broadcast architecture.
   Recommendation is KEEP-ACTIVE. If you prefer to absorb its scope into the v1A stories
   being authored in Burst F, RETIRE with an explicit note that S-046 or a new story absorbs
   BC-2.05.004 PC-2.

2. **S-DAEMON-WIRE-FIX-001 (KEEP-ACTIVE vs RETIRE):** This discharges a live CONTRACT GAP
   in `lifecycle.rs` for second-signal exit codes (SigtermDuringDrain=143,
   SigintDuringDrain=130). The v1A session-host-owns-PTY model adds signal-forwarding
   complexity (session-host process + daemon both need clean signal handling). The Wave-8
   fix scope is still valid and may need to expand. Recommendation is KEEP-ACTIVE.

3. **S-PHASE-3-PREP (KEEP-ACTIVE vs RETIRE):** This story is blocked on upstream
   vsdd-factory spec-kit-mcp rc.19+. It has `behavioral_contracts: []` and is tooling-
   infrastructure-only. Under the v1A pivot, Phase-3 TDD delivery continues for new stories.
   The spec-kit-mcp integration remains relevant if/when the upstream ships. Recommendation
   is KEEP-ACTIVE with status `blocked`. If you no longer plan to adopt spec-kit-mcp, RETIRE.

---

## Actual Count: Pre-Pivot Stories

**The STATE.md estimate of 143 is incorrect.** The actual story file count was determined by
inspecting every file under `.factory/stories/`. The correct count is:

| Category | Count |
|----------|-------|
| Pre-pivot stories, status `done` (Waves 1-7) | 32 |
| Pre-pivot stories, status `draft` (Wave 0 + Wave 8) | 3 |
| **Total pre-pivot stories** | **35** |

The "143" figure in STATE.md was an estimate carried forward from an earlier session context
and was never verified against the actual file count. The task brief acknowledges this:
"do not assume exactly 143 — report the real number." The real number is **35**.

**Post-pivot new stories (Bursts A-C, not subject to this recommendation):** 16 stories
(S-033..S-048), all status `draft`, created after D-236.

---

## Summary Table

| Story ID | Title | Current Status | Wave | Epic | Recommended Disposition | Rationale |
|----------|-------|---------------|------|------|------------------------|-----------|
| **Group A: Done, pre-pivot (Waves 1-7)** | | | | | | |
| S-001 | Cargo Workspace Init + CI/DevOps Setup | done | 1 | EPIC-01 | mark-done-historical | Delivered. Workspace structure valid under v1A (same 9 crates). Historically valid. |
| S-DTU-001 | Claude Code Hook Protocol DTU Clone | done | 1 | EPIC-DTU | mark-done-historical | Delivered. DTU clone fidelity 1.0. Hook protocol unchanged by v1A pivot. Valid under v1A. |
| S-002 | Healthz Endpoint | done | 2 | EPIC-01 | mark-done-historical | Delivered. Daemon HTTP liveness probe unchanged by pivot. Valid under v1A. |
| S-003 | Status Endpoint | done | 2 | EPIC-01 | mark-done-historical | Delivered. Authenticated daemon state endpoint unchanged by pivot. Valid under v1A. |
| S-004 | Body Size Limit | done | 2 | EPIC-01 | mark-done-historical | Delivered. 256 KiB hook request body limit unchanged by pivot. Valid under v1A. |
| S-005 | Graceful Shutdown | done | 2 | EPIC-01 | mark-done-historical | Delivered. Drain + shutdown logic unchanged by pivot. Partial coverage note (S-DAEMON-WIRE-FIX-001 covers PC-8/INV-4). Valid under v1A. |
| S-006 | Lock File Atomic Lifecycle | done | 2 | EPIC-01 | mark-done-historical | Delivered. Lock file protocol + auth token generation unchanged. Valid under v1A. |
| S-010 | Populate monocle-core ABI Version Constant | done | 2 | EPIC-02 | mark-done-historical | Delivered. ABI version constant unchanged. Valid under v1A. |
| S-011 | Non-Exhaustive Enum Policy | done | 2 | EPIC-02 | mark-done-historical | Delivered. `#[non_exhaustive]` policy unchanged; all v1A wire structs follow ADR-0006. Valid under v1A. |
| S-013 | HookEnvelope Proto Wire Format | done | 2 | EPIC-02 | mark-done-historical | Delivered. Proto wire format unchanged by pivot. Valid under v1A. |
| S-014 | EngineModule Trait Definition | done | 2 | EPIC-03 | mark-done-historical | Delivered. EngineModule trait is THE seam for v1A launch/lifecycle extensions (S-045 adds spawn recipe on top). Core trait itself valid. |
| S-007 | Crash Recovery Checkpoint | done | 3 | EPIC-01 | mark-done-historical | Delivered. Daemon crash recovery logic unchanged by pivot. Valid under v1A. |
| S-008 | JSONL Ring Format Version | done | 3 | EPIC-01 | mark-done-historical | Delivered. Ring format_version field unchanged. Valid under v1A. |
| S-009 | Auth Token Wire Format + Header Validation | done | 3 | EPIC-01 | mark-done-historical | Delivered. Auth token protocol unchanged by pivot. Valid under v1A. |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | done | 3 | EPIC-02 | mark-done-historical | Delivered. FactoryAdapter trait unchanged; factory pipeline awareness valid under v1A. |
| S-015 | ClaudeCodeModule Implementation | done | 3 | EPIC-03 | mark-done-historical | Delivered. ClaudeCodeModule detect/id/hook_paths() unchanged. S-045 extends the spawn path ON TOP of this foundation. Valid under v1A. |
| S-016 | Daemon Binary Crate Init + CLI Subcommands | done | 4 | EPIC-04 | mark-done-historical | Delivered. `monocle daemon start/stop` CLI structure unchanged; v1A adds session subcommands on top. Valid under v1A. |
| S-024 | TUI Core Types: AppMode, Action, FocusSnapshot | done | 4 | EPIC-06 | mark-done-historical | Delivered. AppMode/Action/FocusSnapshot types are the foundation for v1A EmbeddedTerminal mode. S-044 adds the new AppMode variants. Valid under v1A. |
| S-030 | Config Crate Foundation | done | 4 | EPIC-07 | mark-done-historical | Delivered. Config schema v1 unchanged; v1A profile-picker logic (S-031) already done on top. Valid under v1A. |
| S-017 | Daemon Start Sequence + Hook Tmpfile | done | 5 | EPIC-04 | mark-done-historical | Delivered. Start sequence steps 1-8a unchanged; step-8b rediscover_sessions is a v1A addition (S-036). Valid historical baseline. |
| S-018 | Hook Routing + Bounded Event Bus | done | 5 | EPIC-04 | mark-done-historical | Delivered. Hook endpoint routing + bounded event bus unchanged. S-032 adds daemon broadcast fan-out on top. Valid under v1A. |
| S-019 | Daemon Auto-Start on TUI Launch | done | 5 | EPIC-04 | mark-done-historical | Delivered. MONOCLE_NO_AUTOSTART semantics unchanged. In v1A, TUI is the entry point and auto-start remains correct. Valid under v1A. |
| S-020 | JSONL Ring Capacity and Rotation Policy | done | 5 | EPIC-04 | mark-done-historical | Delivered. Ring capacity (100MB/5 files/4096 in-memory) unchanged by pivot. Valid under v1A. |
| S-021 | UDS Server Bind + IPC Types | done | 5 | EPIC-05 | mark-done-historical | Delivered. UDS server + core IPC message types are the foundation. v1A adds new message types on top (S-046/S-047). Valid under v1A. |
| S-022 | TUI Connect + Initial State Push + Permission Prompt | done | 6 | EPIC-05 | mark-done-historical | Delivered. TUI connect + initial state push + PermissionPromptQueued message type unchanged. Valid under v1A. |
| S-023 | TUI Reconnect + SOQ-3 Overlay Clear | done | 6 | EPIC-05 | mark-done-historical | Delivered. Reconnect backoff + SOQ-3 overlay clear on disconnect unchanged. Valid under v1A. |
| S-025 | TUI Skeleton + Ctrl-\ + Sessions Panel | done | 6 | EPIC-06 | mark-done-historical | Delivered. TUI binary skeleton + Ctrl-\ popup + sessions panel are the foundation; v1A adds multi-project grouping (S-048) and embedded PTY mode on top. Valid under v1A. |
| S-026 | Permission Overlay Core | done | 6 | EPIC-06 | mark-done-historical | Delivered. VecDeque<PromptModal> permission overlay stack, decision keybindings, SOQ-3 — unchanged by pivot. Core control-center feature. Valid under v1A. |
| S-027 | Overlay Rendering + Status Bar | done | 7 | EPIC-06 | mark-done-historical | Delivered. Diff preview (similar 3), [t] stub, two-row status bar. Rendering layer unchanged by pivot. Valid under v1A. |
| S-028 | Sessions Panel Nucleo Filter + Event Ribbon | done | 7 | EPIC-06 | mark-done-historical | Delivered. Nucleo filter + event ribbon. S-048 extends sessions panel for multi-project; this story's filter logic reusable. Valid under v1A. |
| S-029 | Killer Scenario: ≤6 Keystrokes Dual Permission | done | 7 | EPIC-06 | mark-done-historical | Delivered. Killer scenario validated (HS-EXP-008 score 1.0). Unchanged by pivot. Valid under v1A. |
| S-031 | Profile Picker: Sticky + Ctrl-P Override | done | 7 | EPIC-07 | mark-done-historical | Delivered. Profile picker sticky-per-project + Ctrl-P override. CCR integration. Unchanged by pivot. Valid under v1A. |
| **Group B: Draft, pre-pivot (Wave 0 + Wave 8)** | | | | | | |
| S-032 | Daemon Event-Bus Fan-Out (HookEventReceived broadcast) | draft | 8 | EPIC-05 | **KEEP-ACTIVE** | Discharges live CONTRACT GAP markers in `lifecycle.rs` (BC-2.05.004 PC-2 timestamp_micros). The v1A session-manager broadcasts HookEventReceived to TUI clients for all active sessions — this daemon fan-out is MORE critical under v1A than the observe-only model. Dependencies (S-021/S-022/S-028) are done. See §Decision 1 above. |
| S-DAEMON-WIRE-FIX-001 | Second-Signal Exit Codes (143/130 during drain) | draft | 8 | EPIC-04 | **KEEP-ACTIVE** | Discharges live CONTRACT GAP markers in `lifecycle.rs` (BC-2.01.004 INV-4). The v1A session-host-owns-PTY model means daemon signal handling is even more important (daemon crash must not corrupt PTY-owning session-host processes). Scope may need review for v1A signal-forwarding semantics. See §Decision 2 above. |
| S-PHASE-3-PREP | spec-kit-mcp Integration Sweep | draft | 0 | EPIC-PREP | **KEEP-ACTIVE** | Tooling infrastructure story blocked on upstream vsdd-factory spec-kit-mcp rc.19+. No scope overlap with pivot. If the upstream ships, it remains a valid pre-wave-8 sweep regardless of v1A scope. No behavioral contracts authored (pending PO authorship). See §Decision 3 above. |

---

## Disposition Definitions

- **mark-done-historical**: The story was DELIVERED (status `done`, code merged to develop). No
  action needed in STORY-INDEX or sprint-state. The story file should remain as-is as a
  historical record. The code it produced is part of the v1A substrate and is valid.

- **keep-active**: The story has NOT been delivered and remains OPEN work that is still in-scope
  under the v1A pivot. No disposition change needed; it should proceed to implementation in its
  assigned wave (Wave 8). No file changes needed until Burst F wave scheduling.

- **archive** / **retire**: NOT recommended for any story in this corpus. See §Why No Stories
  Are Archived or Retired below.

---

## Why No Stories Are Archived or Retired

All 35 pre-pivot stories fall into one of two clean categories:

1. **Done (32 stories):** The implementation was already merged to `develop` (PRs #2..#37).
   Archiving or retiring `done` stories is a meaningless disposition — the code exists in the
   repo regardless of story status. The correct designation is `mark-done-historical`, which
   means "this story record accurately reflects merged production code; leave it alone."

2. **Draft with live CONTRACT GAP anchor (3 stories):** S-032, S-DAEMON-WIRE-FIX-001, and
   S-PHASE-3-PREP are draft stories that either (a) discharge `// CONTRACT GAP` markers in
   production code, or (b) are tooling infrastructure. Archiving or retiring any of these would
   orphan the CONTRACT GAP markers in `lifecycle.rs` and `event_bus.rs`, violating the
   CLAUDE.md Principle 3 requirement for a "concrete future story anchor."

There are NO purely observe-only draft stories in this corpus that were left unstarted and
are now out-of-scope. The original v1 VSDD pipeline decomposed only the Phase-1 scope into
stories (the pipeline did NOT pre-decompose Phases 2-4). The STATE.md estimate of "143 orphaned
stories" appears to have been an upper-bound estimate of what Phases 2-4 of the original observe-
only roadmap WOULD HAVE required if they had been decomposed — those stories were never written,
so there is nothing to archive.

---

## Scope Assessment: v1A Compatibility of Done Stories

Each done story's scope is entirely additive to the v1A control-center scope:

| Story Group | v1A Compatibility | Rationale |
|-------------|------------------|-----------|
| EPIC-01 (S-001..S-009): Daemon Lifecycle | FULLY COMPATIBLE | Daemon HTTP server, lock file, auth, graceful shutdown, crash recovery — all unchanged. v1A daemon gains session supervision on top. |
| EPIC-02 (S-010..S-013): Core Types and ABI | FULLY COMPATIBLE | ABI version, non-exhaustive enums, FactoryAdapter trait, HookEnvelope proto — all unchanged. v1A adds new BC-2.03.x enums on top. |
| EPIC-03 (S-014..S-015): Engine Module | FULLY COMPATIBLE | EngineModule trait definition + ClaudeCodeModule are the seam the v1A spawn path extends. S-045 builds ON TOP, not instead-of. |
| EPIC-04 (S-016..S-020): Daemon Wiring | FULLY COMPATIBLE | Binary CLI, start sequence, hook routing, ring rotation — all unchanged. S-036 adds step-8b rediscovery to the start sequence. |
| EPIC-05 (S-021..S-023): IPC | FULLY COMPATIBLE | UDS server bind, IPC transport, initial state push, reconnect — core substrate. v1A adds new message types (S-046/S-047). |
| EPIC-06 (S-024..S-029): TUI | FULLY COMPATIBLE | AppMode/Action types, TUI skeleton, sessions panel, permission overlay, killer scenario — ALL core control-center features. v1A extends with EmbeddedTerminal mode and multi-project grouping. |
| EPIC-07 (S-030..S-031): Config | FULLY COMPATIBLE | Config schema v1, atomic write, profile picker — unchanged. Profile picker is a key v1A "tune" plane feature. |
| EPIC-DTU (S-DTU-001): DTU Clone | FULLY COMPATIBLE | Claude Code hook protocol DTU clone. Hook protocol unchanged by pivot. Valid for v1A testing. |

---

## Recommended Actions After Human Ratification

If human ratifies this recommendation as-is:

1. **No changes to story files.** All 35 pre-pivot stories remain exactly as written.
2. **No changes to STORY-INDEX.md.** The index already accurately reflects story status.
3. **No changes to sprint-state.yaml.** Sprint state is accurate.
4. **State-manager (Burst G):** Record this ratification as a decision (D-NNN) in STATE.md;
   note actual count = 35 (not 143); mark Burst D complete.

If human overrides disposition on S-032, S-DAEMON-WIRE-FIX-001, or S-PHASE-3-PREP (retiring
them), state-manager must:
- Update the story file `status:` field
- Update STORY-INDEX.md Story Registry row and Wave Summary
- Update sprint-state.yaml
- For S-032 and S-DAEMON-WIRE-FIX-001: add a `// CONTRACT GAP DISPOSITION: retired (D-NNN)` comment
  in the relevant source files (`event_bus.rs`, `lifecycle.rs`) to replace the deferred marker
  with an explicit human-ratified closure note

---

## Breakdown of Recommended Dispositions

| Disposition | Count | Stories |
|-------------|-------|---------|
| mark-done-historical | 32 | S-001..S-031 + S-DTU-001 |
| keep-active | 3 | S-032, S-DAEMON-WIRE-FIX-001, S-PHASE-3-PREP |
| archive | 0 | — |
| retire | 0 | — |
| **Total pre-pivot stories** | **35** | |

---

## Ambiguous Cases Requiring Explicit Human Judgment

Three stories are genuinely ambiguous (see §Decisions Required From Human at the top):

| Story | Ambiguity | Alternatives |
|-------|-----------|--------------|
| S-032 | Wave-8 daemon fan-out: absorb into v1A IPC expansion vs keep standalone | KEEP-ACTIVE (recommended) or RETIRE with explicit absorption into S-046 |
| S-DAEMON-WIRE-FIX-001 | Second-signal exit codes: still relevant under v1A signal semantics vs defer indefinitely | KEEP-ACTIVE (recommended) or RETIRE if v1A signal handling is being fully redesigned in S-033..S-038 |
| S-PHASE-3-PREP | spec-kit-mcp: upstream may never ship vs keep as aspirational tooling | KEEP-ACTIVE / blocked (recommended) or RETIRE if spec-kit-mcp dependency is abandoned |

---

_Produced by vsdd-factory:story-writer for Burst D. Per D-305: DO NOT execute any disposition
until this document is ratified by Joshua Magady._
