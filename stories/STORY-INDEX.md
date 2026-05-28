---
document_type: story-index
level: L4
version: "5.7"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-28T00:00:00Z
phase: 2
inputs:
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.25"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/domain-spec/L2-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.16"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
  - {path: .factory/specs/architecture/SS-daemon-wiring.md, version: "1.3.0"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.6.0"}
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.8.0"}
  - {path: .factory/specs/architecture/SS-config.md, version: "1.3.0"}
input-hash: "[live-state]"
traces_to: ".factory/specs/prd.md v1.26.15"
---

# Story Index: monocle Phase 2

> **Source of truth** for all story IDs, status, points, wave assignments, and BC/VP traceability.
> Per artifact-path-registry.yaml: stories at `.factory/stories/S-{story-id}-{slug}.md`.

## Epics

| Epic ID | Name | Capability | Subsystem | Stories |
|---------|------|-----------|-----------|---------|
| EPIC-01 | Daemon Lifecycle | CAP-001 | SS-01 | S-001, S-002, S-003, S-004, S-005, S-006, S-007, S-008, S-009 |
| EPIC-02 | Core Types and ABI | CAP-002 | SS-02 | S-010, S-011, S-012, S-013 |
| EPIC-03 | Engine Module | CAP-003 | SS-03 | S-014, S-015 |
| EPIC-04 | Daemon Wiring | CAP-004 | SS-04 | S-016, S-017, S-018, S-019, S-020 |
| EPIC-05 | IPC | CAP-005 | SS-05 | S-021, S-022, S-023 |
| EPIC-06 | TUI | CAP-006 | SS-06 | S-024, S-025, S-026, S-027, S-028, S-029 |
| EPIC-07 | Config | CAP-007 | SS-07 | S-030, S-031 |
| EPIC-DTU | Claude Code Hook Protocol Clone | CAP-001 (DTU) | — | S-DTU-001 |
| EPIC-PREP | Phase 3 Pre-Implementation Prep | — | — | S-PHASE-3-PREP |

## Story Registry

| Story ID | Title | Epic | Points | Wave | Status | Blocks |
|----------|-------|------|--------|------|--------|--------|
| S-PHASE-3-PREP | spec-kit-mcp Integration Sweep | EPIC-PREP | 3 | 0 | draft | (Phase 3 gate) |
| S-DTU-001 | Claude Code Hook Protocol DTU Clone | EPIC-DTU | 3 | 1 | done | S-009 |
| S-001 | Cargo Workspace Init + CI/DevOps Setup | EPIC-01 | 5 | 1 | done | S-002, S-003, S-004, S-005, S-006, S-009, S-010, S-013 |
| S-002 | Healthz Endpoint | EPIC-01 | 3 | 2 | draft | S-003, S-005 |
| S-003 | Status Endpoint | EPIC-01 | 5 | 2 | draft | S-004, S-005, S-009, S-010 |
| S-004 | Body Size Limit | EPIC-01 | 2 | 2 | draft | S-009 |
| S-005 | Graceful Shutdown | EPIC-01 | 5 | 2 | draft | — |
| S-006 | Lock File Atomic Lifecycle | EPIC-01 | 8 | 2 | draft | S-005, S-007, S-008, S-009 |
| S-009 | Auth Token Wire Format + Header Validation | EPIC-01 | 8 | 3 | done | — |
| S-010 | Populate monocle-core ABI Version Constant (FC-03) | EPIC-02 | 5 | 2 | draft | S-011, S-012, S-014 |
| S-011 | Non-Exhaustive Enum Policy | EPIC-02 | 3 | 2 | draft | S-012 |
| S-013 | HookEnvelope Proto Wire Format | EPIC-02 | 5 | 2 | draft | — |
| S-014 | EngineModule Trait Definition | EPIC-03 | 5 | 2 | draft | S-015 |
| S-007 | Crash Recovery Checkpoint | EPIC-01 | 5 | 3 | done | — |
| S-008 | JSONL Ring Format Version | EPIC-01 | 5 | 3 | done | S-009 |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | EPIC-02 | 8 | 3 | done | — |
| S-015 | ClaudeCodeModule Implementation | EPIC-03 | 8 | 3 | done | — |
| S-016 | Daemon Binary Crate Init + CLI Subcommands | EPIC-04 | 5 | 4 | done | S-017, S-019 |
| S-024 | TUI Core Types: AppMode, Action, FocusSnapshot, transition(), 5-Level Dispatch | EPIC-06 | 8 | 4 | done | S-025, S-026, S-031 |
| S-030 | Config Crate: Atomic Write, Schema v1, Missing/Corrupted Default, CCR Detection | EPIC-07 | 5 | 4 | done | S-025, S-031 |
| S-017 | Daemon Start Sequence (SOQ-2) + Hook Tmpfile Generation | EPIC-04 | 8 | 5 | done | S-018, S-019, S-020, S-021 |
| S-018 | Hook Endpoint Routing + Bounded Event Bus with Drop Counter | EPIC-04 | 8 | 5 | done | S-022, S-029 |
| S-019 | Daemon Auto-Start on TUI Launch + MONOCLE_NO_AUTOSTART | EPIC-04 | 5 | 5 | done | S-023 |
| S-020 | JSONL Ring Capacity and Rotation Policy | EPIC-04 | 5 | 5 | done | — |
| S-021 | UDS Server Bind + IPC Transport + Core Message Types | EPIC-05 | 8 | 5 | done | S-022, S-028 |
| S-022 | TUI Client Connect, Initial State Push, and Permission Message Types | EPIC-05 | 8 | 6 | done | S-023, S-025, S-026, S-029 |
| S-023 | TUI Reconnect After Daemon Restart + SOQ-3 Overlay Clear | EPIC-05 | 5 | 6 | done | S-026 |
| S-025 | TUI Binary Skeleton, Ctrl-\ Popup Integration, and Sessions Panel | EPIC-06 | 8 | 6 | not_started | S-027, S-028, S-031 |
| S-026 | Permission Overlay: VecDeque Stack, Decision Keybindings, Esc Hide, SOQ-3 | EPIC-06 | 13 | 6 | not_started | S-027, S-029 |
| S-027 | Permission Overlay Rendering, Diff Preview (similar 3), Status Bar | EPIC-06 | 8 | 7 | not_started | S-029 |
| S-028 | Sessions Panel Nucleo Filter + Event Ribbon Rolling Log | EPIC-06 | 5 | 7 | not_started | — |
| S-029 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | EPIC-06 | 5 | 7 | not_started | — |
| S-031 | Profile Picker: Sticky-Per-Project Selection + Ctrl-P Override | EPIC-07 | 5 | 7 | not_started | — |

**Total stories:** 33 (29 product + 1 DTU + 1 prep + 2 admin)
**Total points (product):** 189 (excl. DTU 3 pts and PREP 3 pts)
**Total points (all):** 195

## Wave Summary

| Wave | Stories | Points | Description |
|------|---------|--------|-------------|
| Wave 0 | S-PHASE-3-PREP | 3 | Pre-Phase-3 gate (blocked on spec-kit-mcp rc.19+) |
| Wave 1 | S-DTU-001, S-001 | 8 | Foundation: DTU clone + workspace init |
| Wave 2 | S-002, S-003, S-004, S-005, S-006, S-010, S-011, S-013, S-014 | 41 | Core implementation (parallel-eligible within wave) |
| Wave 3 | S-007, S-008, S-009, S-012, S-015 | 34 | Dependent completions (S-009 moved per Decision 1: S-008→S-009 dependency) |
| Wave 4 | S-016, S-024, S-030 | 18 | Foundation: daemon CLI + TUI core types + config crate (parallel-eligible) |
| Wave 5 | S-017, S-018, S-019, S-020, S-021 | 34 | Daemon integration — S-017 (serial prerequisite), then S-018, S-019, S-020, S-021 (parallel after S-017). 34 pts. |
| Wave 6 | S-022, S-023, S-025, S-026 | 34 | IPC + TUI integration — S-022 (serial prerequisite), then S-023 + S-025 (parallel after S-022), then S-026 (after S-023 + S-022). 34 pts. |
| Wave 7 | S-027, S-028, S-029, S-031 | 23 | Polish: overlay rendering, filter, killer scenario, profile picker (parallel-eligible within wave) |

## BC Coverage Table

| BC ID | Title | Covering Story | AC | Full Coverage? |
|-------|-------|---------------|----|----------------|
| BC-2.01.001 | Healthz Endpoint | S-002 | AC-001..AC-006 | YES |
| BC-2.01.002 | Status Endpoint | S-003 | AC-001, AC-005, AC-006, AC-007, AC-008 | YES |
| BC-2.01.003 | Body Size Limit | S-004 | AC-001..AC-006 | YES |
| BC-2.01.004 | Graceful Shutdown | S-005 | AC-001..AC-006 | YES |
| BC-2.01.005 | Lock File Atomic Lifecycle | S-006 | AC-001..AC-009 | YES |
| BC-2.01.006 | Crash Recovery Checkpoint | S-007 | AC-001..AC-010 | YES |
| BC-2.01.007 | JSONL Ring Format Version | S-008 | AC-001..AC-007 | YES |
| BC-2.01.008 | Auth Token Wire Format | S-006, S-009 | S-006: AC-014 (token generation); S-009: AC-001..AC-003 | YES |
| BC-2.01.009 | Auth Header Validation | S-009 | AC-004..AC-009 | YES |
| BC-2.01.010 | Lock File Contract Version Field | S-006 | AC-010..AC-013 | YES |
| BC-2.02.001 | ABI Version in /status | S-010, S-003 | S-010: AC-003, AC-005; S-003: AC-005 | YES |
| BC-2.02.002 | ABI Version Constant at Crate Root | S-010 | AC-001, AC-002, AC-004 | YES |
| BC-2.02.003 | Non-Exhaustive Enum Policy | S-011, S-014 | S-011: AC-001..AC-004; S-014: AC-003b (HookEvent #[non_exhaustive]) | YES |
| BC-2.02.004 | FactoryAdapter Trait Definition | S-012 | AC-001..AC-004 | YES |
| BC-2.02.005 | VsddFactoryAdapter Implementation | S-012 | AC-005..AC-013 | YES |
| BC-2.02.006 | HookEnvelope Proto Field Number | S-013 | AC-001, AC-006 | YES |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version | S-013 | AC-002..AC-003 | YES |
| BC-2.02.008 | Phase 4 schema_version Validation | S-013 | AC-004..AC-005 | YES |
| BC-2.03.001 | EngineModule Trait Definition | S-014, S-015 | S-014: AC-001..AC-007; S-015: AC-010 (PC-6 DI-006) | YES |
| BC-2.03.002 | ClaudeCodeModule (Strict-Basename Detect) | S-015 | AC-001..AC-004 | YES |
| BC-2.03.003 | HomeUnresolvable Error Contract | S-015 | AC-005, AC-006 | YES |
| BC-2.03.004 | ClaudeCodeModule Inherent Methods | S-015 | AC-007, AC-008, AC-009 | YES |

| BC-2.04.001 | Daemon Start Sequence: Port Bind + Lock File + Token Write (SOQ-2) | S-017 | AC-001..AC-013 | YES |
| BC-2.04.002 | Daemon Auto-Start on TUI Launch | S-019 | AC-001..AC-005 | YES |
| BC-2.04.003 | `MONOCLE_NO_AUTOSTART=1` Suppresses Auto-Start | S-019 | AC-006 | YES |
| BC-2.04.004 | `monocle daemon start` CLI Subcommand | S-016 | AC-001..AC-003 | YES |
| BC-2.04.005 | `monocle daemon stop` CLI Subcommand | S-016 | AC-004..AC-006 | YES |
| BC-2.04.006 | `directories::ProjectDirs::runtime_dir()` Fallback Chain | S-016 | AC-007..AC-009 | YES |
| BC-2.04.007 | Hook Endpoint: PreToolUse Request Routing | S-018 | AC-001..AC-004 | YES |
| BC-2.04.008 | Hook Endpoint: Notification Request Routing (2000ms Timeout) | S-018 | AC-005..AC-007 | YES |
| BC-2.04.009 | Hook Endpoint: Stop/SessionStart/PromptSubmit Routing (300ms Timeout) | S-018 | AC-008..AC-010 | YES |
| BC-2.04.010 | Hook Tmpfile Generation at `runtimeDir/hooks-settings.json` | S-017 | AC-009 | YES |
| BC-2.04.011 | Bounded Event Bus with Drop Counter | S-018 | AC-011..AC-013 | YES |
| BC-2.04.012 | JSONL Ring: Capacity and Rotation Policy | S-020 | AC-001..AC-005 | YES |
| BC-2.05.001 | UDS Server Bind at `runtimeDir/monocle.sock` | S-021 | AC-001..AC-003 | YES |
| BC-2.05.002 | TUI Client Connects to UDS and Receives Initial State Push | S-022, S-025, S-026 | S-022: AC-001..AC-005; S-025: AC-008 (Invariant 4 idempotency); S-026: AC-001 (Invariant 4 idempotency) | YES |
| BC-2.05.003 | IPC Message Types: SessionListUpdate | S-021 | AC-004..AC-006 | YES |
| BC-2.05.004 | IPC Message Types: HookEventReceived | S-021 | AC-007..AC-009 | YES |
| BC-2.05.005 | IPC Message Types: PermissionPromptQueued | S-022 | AC-006..AC-008 | YES |
| BC-2.05.006 | TUI Reconnects After Daemon Restart | S-023 | AC-001..AC-004 | YES |
| BC-2.05.007 | Overlay Stack Cleared on Daemon Disconnect (SOQ-3) | S-023 | AC-005..AC-006 | YES |
| BC-2.05.008 | UDS-Only in Phase 1 (No Shared-Memory Transport) | S-021 | AC-010..AC-011 | YES |
| BC-2.06.001 | AppMode State Machine: Compile-Time Mutual Exclusion | S-024 | AC-001..AC-004 | YES |
| BC-2.06.002 | FocusSnapshot: Focus Restored After Overlay/Fullscreen Close | S-024 | AC-005..AC-007 | YES |
| BC-2.06.003 | Action Dispatch: 5-Level Binding Precedence | S-024 | AC-008..AC-012 | YES |
| BC-2.06.004 | `Ctrl-\` Popup: Appears and Dismisses Without State Loss | S-025 | AC-001..AC-003 | YES |
| BC-2.06.005 | Sessions Panel: Session List Renders from IPC State | S-025 | AC-004..AC-006 | YES |
| BC-2.06.006 | Sessions Panel: `/` Filter with Nucleo Fuzzy Match | S-028 | AC-001..AC-004 | YES |
| BC-2.06.007 | Sessions Panel: `Enter` Transitions to Fullscreen | S-025 | AC-007..AC-008 | YES |
| BC-2.06.008 | Permission Overlay: VecDeque Stack Push on PermissionPromptQueued | S-026 | AC-001, AC-002, AC-016 | YES |
| BC-2.06.009 | Permission Overlay: `[↑↓]` Rotates Stack | S-026 | AC-013, AC-014 | YES |
| BC-2.06.010 | Permission Overlay: Diff Preview via `similar 3` | S-027 | AC-001..AC-003 | YES |
| BC-2.06.011 | Permission Overlay: Accept-Once Keybinding | S-026 | AC-003 | YES |
| BC-2.06.012 | Permission Overlay: Accept-Always Keybinding | S-026 | AC-004 | YES |
| BC-2.06.013 | Permission Overlay: Reject Keybinding | S-026 | AC-005 | YES |
| BC-2.06.014 | Permission Overlay: `[Esc]` Hides Without Rejecting | S-026 | AC-008 | YES |
| BC-2.06.015 | Permission Overlay: `[t]` Trace-to-Source Stub | S-027 | AC-004 | YES |
| BC-2.06.016 | Permission Overlay: Cleared on Daemon Disconnect | S-026 | AC-011, AC-012 | YES |
| BC-2.06.017 | Permission Response Within Hook Timeout Budget | — | — | GAP (see GAP-P2-005) |
| BC-2.06.018 | Event Ribbon Panel: Rolling Hook Event Log | S-028 | AC-006..AC-009 | YES |
| BC-2.06.019 | Status Bar: Drop Counter Renders Under Load | S-027 | AC-007..AC-008 | YES |
| BC-2.06.020 | Status Bar: Breadcrumb | S-027 | AC-009 | YES |
| BC-2.06.021 | Status Bar: Keybinding Hint Line | S-027 | AC-010 | YES |
| BC-2.06.022 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | S-029 | AC-001..AC-006 | YES |
| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | S-026 | AC-006, AC-007, AC-015 | YES |
| BC-2.06.024 | Permission Overlay: ToolPayload Body Rendering by Variant | S-026, S-027 | S-026: AC-016; S-027: AC-003, AC-004, AC-006 | YES |
| BC-2.07.001 | Config File Atomic Write via `tempfile::persist` | S-030 | AC-001..AC-002 | YES |
| BC-2.07.002 | Config Schema Version 1: Harness Profile Fields | S-030 | AC-003..AC-005 | YES |
| BC-2.07.003 | Config Missing or Corrupted: Default Applied | S-030 | AC-006..AC-008 | YES |
| BC-2.07.004 | Profile Picker: Sticky-Per-Project | S-031 | AC-001..AC-003 | YES |
| BC-2.07.005 | Profile Picker: `Ctrl-P` Override Shows Picker | S-031 | AC-004..AC-006 | YES |
| BC-2.07.006 | CCR Detection via `ccr_path` Config Field | S-030 | AC-009..AC-010 | YES |

| BC-HOOK-001 | PreToolUse Hook Fail-Open Semantics (No Server Found) | S-DTU-001 | AC-001 | YES |
| BC-HOOK-002 | Non-PreToolUse Hooks Fail-Closed (No Server Found) | S-DTU-001 | AC-001 | YES |
| BC-HOOK-003 | Notification Hook Filters on notification_type === 'permission_prompt' | S-DTU-001 | AC-001, AC-003 | YES |
| BC-HOOK-004 | Hook HTTP Requests Are Fire-and-Forget (Response Ignored) | S-DTU-001 | AC-001 | YES |
| BC-HOOK-005 | Hook HTTP Request Target is 127.0.0.1 with Port from Lock File | S-DTU-001 | AC-001, AC-002, AC-003, AC-006 | YES |
| BC-HOOK-006 | PreToolUse Always Echoes Stdin to Stdout | S-DTU-001 | AC-001, AC-003 | YES |
| BC-HOOK-007 | Exactly Five Hook Types Registered; PostToolUse Intentionally Absent | S-DTU-001 | AC-001 | YES |
| BC-HOOK-008..BC-HOOK-041 | Hooks-settings.json encoding, path, lifecycle, timeout, env, edge cases | S-DTU-001 | AC-001..AC-006 (fidelity gate) | YES |

**BC Coverage: 22/22 product BCs Waves 1-3 (100%); 49/50 product BCs Waves 4-7 (BC-2.06.017 deferred — see GAP-P2-005; BC-2.06.024 added); 41/41 DTU gene-source BCs (100%)**
**Total product BC coverage: 71/72 (BC-2.06.017 gap — GAP-P2-005; BC-2.06.024 covered by S-027)**

## VP Coverage Table

| VP ID | Title | Anchor Story | Story Where Test Lives |
|-------|-------|-------------|----------------------|
| VP-001 | Healthz Endpoint — 200/503 | S-002 | S-002 |
| VP-002 | Status Endpoint — 10 Required Fields | S-003 | S-003 |
| VP-003 | Body Size Limit — 256 KiB; HTTP 413 | S-004 | S-004 |
| VP-004 | Graceful Shutdown — 10-Second Drain | S-005 | S-005 |
| VP-005 | Lock File Lifecycle — Atomic Create + Modes | S-006 | S-006 |
| VP-006 | Crash Recovery Checkpoint | S-007 | S-007 |
| VP-007 | JSONL Ring Record — format_version First Key | S-008 | S-008 |

| VP-008 | Auth Token — Wire Format + Constant-Time | S-009 | S-009 |
| VP-009 | Auth Header Validation — Dual-Accept | S-009 | S-009 |
| VP-010 | Lock File contract_version: 1 First Key | S-006 | S-006 |
| VP-011 | ABI Version in /status Endpoint | S-003, S-010 | S-010 |
| VP-012 | MONOCLE_ABI_VERSION Pub Const Equals 1 | S-010 | S-010 |
| VP-013 | Non-Exhaustive Enum Policy | S-011 | S-011 |
| VP-014 | FactoryAdapter Trait Signature Stable | S-012 | S-012 |
| VP-015 | VsddFactoryAdapter Self-Referential Detection | S-012 | S-012 |
| VP-016 | Proto Field Number 1 = schema_version | S-013 | S-013 |
| VP-017 | HookEnvelope Rust Struct schema_version Field | S-013 | S-013 |
| VP-018 | schema_version Forward-Compat Contract | S-013 | S-013 |
| VP-019 | EngineModule Trait Signature Stable | S-014 | S-014 |
| VP-020 | ClaudeCodeModule::detect Strict Basename | S-015 | S-015 |
| VP-021 | metadata/enrich Return HomeUnresolvable | S-015 | S-015 |
| VP-022 | hook_paths() Returns Exactly 5 Entries | S-015 | S-015 |

**VP Coverage: 22/22 (100%)**

## NFR Coverage Table

| NFR ID | Category | Covering Story | Validation Method |
|--------|----------|---------------|-------------------|
| NFR-001 | Latency | Phase 3 integration test | Phase 3 story decomposition (load-test infra) |
| NFR-002 | Latency | Phase 3 integration test | Phase 3 story decomposition |
| NFR-003 | Latency | Phase 3 integration test (TUI) | Phase 3 story decomposition |
| NFR-004 | Security | S-009 | VP-008 OsRng source-grep; AC-001 |
| NFR-005 | Security | S-004 | VP-003 AC-001 |
| NFR-006 | Throughput | Phase 3 integration test | Phase 3 story decomposition (1000 events/sec) |
| NFR-007 | Build | S-001 | CI gate: rust-toolchain.toml AC-002, AC-004 |
| NFR-008 | Build | S-001 | CI gate: matrix AC-003 |
| NFR-009 | Security | S-006 | VP-005 Post-condition 1 (0o600 mode); AC-001 |
| NFR-010 | Correctness | S-009, S-003 | VP-008/VP-009 constant_time_eq source-grep |
| NFR-011 | Forward-compat | S-DTU-001 | DTU fidelity ≥0.95 fixture corpus |
| NFR-012 | Security | S-006 | VP-005 Post-condition 9 (0o700 mode); AC-006 |

**P0 NFR Coverage: 12/12 (100%)**

**NFR-001/002/003/006 deferred to Phase 3:** These NFRs validate TUI + load-test behaviors
that require Phase 3 infrastructure. They are NOT gaps — they are phased deliverables per
nfr-catalog.md §VP Probe Citations. Covered stories will be authored at Phase 3 entry.

## Error Code Coverage

| Error Code | Covering Story | AC Reference |
|-----------|---------------|--------------|
| E-AUTH-001 | S-009 | AC-004 |
| E-AUTH-002 | S-009 | AC-005, AC-006 |
| E-AUTH-003 | S-009 | AC-005; S-003 AC-002 |
| E-DAEMON-001 | S-004 | AC-001, AC-004 |
| E-DAEMON-002 | S-005 | AC-003 |
| E-DAEMON-003 | S-002 | AC-002 |
| E-DAEMON-004 | S-006 | AC-009 |
| E-LOCK-001 | S-006 | AC-003 |
| E-LOCK-002 | S-006 | AC-004 |
| E-LOCK-003 | S-006 | AC-010 |
| E-ENG-001 | S-015 | AC-005 |
| E-FACT-001 | S-012 | AC-008 |
| E-FACT-002 | S-012 | AC-008 |
| E-RING-001 | S-008 | AC-005 |
| E-PROTO-001 | S-013 | AC-004 |

**Error Code Coverage: 15/15 (100%)**

## Gap Register

| Gap ID | Level | Source | Justification | Resolution Target |
|--------|-------|--------|---------------|-------------------|
| GAP-P2-001 | L3 | NFR-001 (hook latency ≤300ms) | Requires Phase 3 load-test infrastructure not available in Phase 1; per nfr-catalog.md §VP Probe Citations | Phase 3 story decomposition at Phase 3 entry |
| GAP-P2-002 | L3 | NFR-002 (Notification latency ≤2000ms) | Same rationale as GAP-P2-001; Phase 3 infra required | Phase 3 story decomposition at Phase 3 entry |
| GAP-P2-003 | L3 | NFR-003 (TUI overlay render ≤100ms) | TUI permission overlay is Phase 3 deliverable; not Phase 1 scope | Phase 3 story decomposition at Phase 3 entry |
| GAP-P2-004 | L3 | NFR-006 (1000 events/sec throughput) | Phase 3 load-test infra required; bounded-channel DESIGN is Phase 1 (in S-008), sustained VALIDATION is Phase 3 | Phase 3 story decomposition at Phase 3 entry |
| GAP-P2-005 | L1 | BC-2.06.017 (Permission Response Within Hook Timeout Budget) | BC-2.06.017 covers render ≤100ms latency budget for tool payload rendering. PO re-anchored tool-payload rendering behavior to new BC-2.06.024; BC-2.06.017 (latency budget) has no covering story in Phase 2 waves. Latency validation requires Phase 3 integration test infrastructure. | Phase 3 story decomposition at Phase 3 entry |

L3 (NFR) gaps (GAP-P2-001..004) deferred to Phase 3 per nfr-catalog.md authoritative ruling.
GAP-P2-005 is L1 (BC clause) deferred: latency budget validation for BC-2.06.017 requires Phase 3 integration test infrastructure; behavior covered by BC-2.06.024 (tool payload rendering) is fully implemented in S-027.

## §Trace v1.0

**Phase 2 story decomposition initial burst** (2026-05-19T04:30:00Z):
- 17 stories created: 15 product stories + 1 DTU (S-DTU-001) + 1 prep (S-PHASE-3-PREP)
- 22/22 BCs covered (100%)
- 22/22 VPs covered (100%)
- 15/15 error codes covered (100%)
- 12/12 P0 NFRs covered (NFR-001/002/003/006 deferred to Phase 3 per authoritative nfr-catalog.md ruling)
- 4-wave schedule (Wave 0 = Phase 3 prep gate, Wave 1 = foundation, Wave 2 = parallel impl, Wave 3 = dependents)
- S-PHASE-3-PREP created per TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE §Future Attachment obligation
- Dependency graph is acyclic (validated via topological sort; see dependency-graph.md)

## §Trace v1.1

**Phase 2 r01 remediation burst** (2026-05-19):
- F-PHASE2-R01-01..26 and GAP-PHASE2-R01-01..11 addressed
- S-009 moved from Wave 2 to Wave 3 (Decision 1: S-008→S-009 dependency added)
- S-001 BC-2.01.007/VP-007 mis-anchors removed; BC-2.01.007 sole implementer is S-008
- S-003 behavioral_contracts updated: [BC-2.01.002, BC-2.02.001] (GAP-5 resolved)
- S-006 blocks: [S-007, S-008] (S-009 removed — S-009 now depends on S-008 not S-006 directly)
- S-005 blocks: [] (S-007 removed — S-007 depends on S-006 not S-005)
- Wave 2 points: 41 (was 49); Wave 3 points: 34 (was 26)
- All 17 stories retrofitted with inputs:/input-hash:/traces_to: per SE-22 v2

## §Trace v1.2

**Phase 2 r02 remediation burst** (2026-05-19):
- F-PHASE2-R02-01 (CRITICAL): S-005 AC-004 exit codes rewritten to BC-2.01.004 PC-8 canonical 5-code taxonomy; fabricated codes 3/4 removed; AC-005 rewritten for INV-1 hard timeout; AC-006 renamed for INV-3 dual-accept
- F-PHASE2-R02-02 (CRITICAL): EPIC-01 stories table — S-005 Depends-On corrected to S-001,S-002; S-009 Wave corrected to Wave 3 with full depends-on list
- F-PHASE2-R02-03 (HIGH): S-008 Previous Story Intelligence corrected — S-008 is PRODUCER of RingBuffer API; S-009 is consumer; no stubbing allowed
- F-PHASE2-R02-04 (HIGH): S-015 AC-010 reanchored BC-2.03.001 invariant 2 → postcondition 5 (DI-006 enforce); dep-graph matrix row updated accordingly
- F-PHASE2-R02-05 (HIGH): S-012 AC-007 + AC-009 reanchored postcondition 3 → invariant 3; dep-graph row corrected; holdout HS-W3-005 corrected
- F-PHASE2-R02-06 (HIGH): S-005 AC-005 rewritten for INV-1 hard timeout (production-grade: preserve breadth not delete)
- F-PHASE2-R02-07 (HIGH): dep-graph BC Clause Coverage Matrix swept — BC-2.01.004 PC-4→PC-8 corrected; BC-2.01.005 postcondition rows reordered monotonically; BC-2.02.005 postcondition 3→invariant 3; BC-2.03.001 INV-2(DI-006)→PC-5(DI-006); GAP-P2-005 added for BC-2.01.004 PC-6 (--persistent-events Phase 3 scope)
- F-PHASE2-R02-08 (HIGH): S-006 Previous Story Intelligence rand version pin corrected — =0.8.6 EXACT; rand 0.9 REJECTED with rationale
- F-PHASE2-R02-11+17 (MEDIUM, Orchestrator Decision 3): monocle-auth crate dropped; generate_session_token() moved to monocle-runtime::auth; swept across S-001, S-006, S-009; S-001 workspace member list corrected to 3 crates; S-001 forbidden dependencies updated
- F-PHASE2-R02-12 (MEDIUM): S-015 File Structure test path monocle-runtime/tests/ → monocle-core/tests/
- F-PHASE2-R02-13 (MEDIUM): S-009 dtu_dependencies non-canonical field removed
- F-PHASE2-R02-15 (LOW): S-015 body BC table removed (standardize: no-body-table across corpus)
- GAP-PHASE2-R02-1 (HIGH): STORY-INDEX Blocks column — S-005 S-007→"—"; S-006 S-009 removed; sweep verified other entries consistent
- GAP-PHASE2-R02-2 (MEDIUM): wave-schedule.md Wave 3 paragraph updated "all 4" → "all 5" + S-008→S-009 within-wave dep note
- GAP-PHASE2-R02-3 (MEDIUM): S-009 File Structure generate_auth_token() → generate_session_token() clarification; conflation removed
- GAP-PHASE2-R02-4 (LOW): holdout-scenarios.md frontmatter level: ops + version: "1.1" added
- F-PHASE2-R02-09 (MEDIUM): sprint-state.yaml S-015 notes updated to include BC-2.03.001
- BC-2.03.001 BC Coverage Table updated: now S-014, S-015 (S-015 AC-010 covers PC-5 DI-006)
- Version-bump rule applied: stories with AC/depends_on/blocks/behavioral_contracts changes → minor bump (+0.1)

## §Trace v1.3

**Phase 2 r03/r04/r05 remediation bursts** (2026-05-19):
- F-PHASE2-R03-01 (CRITICAL): BC-2.01.002 BC Coverage Table corrected — BC-2.01.002 AC range updated to AC-001..AC-008 (AC-008 /status during drain added); S-009 BC-2.01.009 range confirmed AC-004..AC-010b.
- F-PHASE2-R04: BC Coverage Table S-009 and S-003 row updates consistent with dep-graph r04 anchor corrections.
- F-PHASE2-R05: AC-007b orphan introduced via r05 burst (BC-2.02.001 row S-003: AC-007b) — see v1.4 for resolution.
- BC-2.03.001 BC Coverage Table confirmed: S-014 AC-001..AC-007; S-015 AC-010 (PC-6 DI-006).
- Version-bump rule applied consistently.

## §Trace v1.4

**Phase 2 r06 remediation burst** (2026-05-19):
- F-PHASE2-R06-01 (CRITICAL): BC-2.01.009 PC-2/PC-3 alias/canonical mirror swap fixed — S-009 AC-005 trace header corrected to PC-3 (alias); AC-006 trace header corrected to PC-2 (canonical); S-003 AC-002 trace header corrected to PC-3 (alias).
- F-PHASE2-R06-02 (HIGH): BC-2.02.001 BC Coverage Table row corrected — S-003: AC-007b (orphan) → S-003: AC-005 (matches body consolidation note; AC-005 subsumes AC-007b intent per S-003 body line 90-93).
- F-PHASE2-R06-03 (MEDIUM): STORY-INDEX version bumped v1.3→v1.4; sprint-state.yaml and holdout-scenarios.md traces_to_full/traces_to updated to v1.4.
- F-PHASE2-R06-04 (MEDIUM): §Trace audit-trail completed — v1.1/v1.2/v1.3/v1.4 entries added for monotonically-ascending version coverage.
- SE-22 v2 cascade: BC-INDEX v1.12→v1.13 propagated to all 19 corpus consumers. BC-2.01.001..010 and BC-2.03.001..004 version pins propagated to all story frontmatter inputs entries.
- Discipline codified: story-corpus artifacts MUST have §Trace entries in monotonically-ascending version order for every declared version.

## §Trace v1.5

**Phase 2 r07 remediation burst** (2026-05-19):
- F-PHASE2-R07-02 + GAP-R07-1 (MED): Token Budget table BC version cells updated — S-007 BC-2.01.006.md 1.0.4→1.0.5; S-015 BC-2.03.001.md 1.0.4→1.0.5, BC-2.03.002.md 1.0.3→1.0.4, BC-2.03.003.md 1.0.2→1.0.3, BC-2.03.004.md 1.0.3→1.0.4. Sibling sweep: only S-007 and S-015 had stale body-prose BC version cells; all other 15 stories confirmed clean.
- F-PHASE2-R07-03 (MED): S-015 line 121 prose version corrected — "added in BC-2.03.001 v1.0.5" → "v1.0.4" (PC-6 was authored in v1.0.4; v1.0.5 was pointer-only SE-22 cascade).
- F-PHASE2-R07-04 (MED): §Trace v1.0/v1.1 restructured — r01 remediation prose (F-PHASE2-R01-01..26, wave/dependency/SE-22 v2 changes) moved from §Trace v1.0 body into §Trace v1.1 body where it belongs; v1.1 pointer-stub eliminated.
- F-PHASE2-R07-05 / Orchestrator Decision 9 (LOW): wave-schedule.md inputs: entry added for error-taxonomy.md v1.5 to sibling-mirror STORY-INDEX/dep-graph; wave gate criteria reference E-AUTH-001/002/003 error codes.
- F-PHASE2-R07-07 (LOW): wave-schedule.md Wave 3 parallelism prose rewritten — "All 5 stories" ambiguity resolved; now correctly states 4 fully parallel + S-009 serially after S-008 per Decision 1.
- STORY-INDEX version bumped v1.4→v1.5.

## §Trace v2.4 (STORY-INDEX version)

**Phase 3 TDD — BC-HOOK-001..041 authored; S-001 done; S-DTU-001 ready** (2026-05-20T21:00:00Z):

- PO authored 41 behavioral contracts (BC-HOOK-001..BC-HOOK-041) in `.factory/specs/behavioral-contracts/ss-dtu/`.
- S-DTU-001 status updated: `draft` → `ready` (BC authorship prerequisite cleared).
- S-001 status updated: `draft` → `done` (PR #2 merged at develop @ 184f7d4).
- Story Registry table: S-DTU-001 `draft` → `ready`; S-001 `draft` → `done`.
- BC Coverage Table: 41 new BC-HOOK rows added (consolidating BC-HOOK-008..BC-HOOK-041 into a single range row for brevity); total coverage note updated.
- STORY-INDEX version bumped: 2.3 → 2.4.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z > prior 2026-05-19T12:00:00Z (v2.3). ARITHMETICALLY TRUE: PASS.

## §Trace v1.6

**Phase 2 r09 remediation burst** (2026-05-19):
- F-PHASE2-R09-01 (HIGH): Bidirectional DAG-edge asymmetry corrected per Orchestrator Decision 10:
  - S-001 Blocks column: S-009 added (S-009 directly consumes S-001's workspace + axum router foundation).
  - S-006 Blocks column: S-009 added (S-006 produces the cryptographic auth token written to the lock file; S-009 reads it from the lock file for header validation).
  - S-008 Blocks column: S-009 corrected from "—" to "S-009" (F-PHASE2-R09-02).
- F-PHASE2-R09-02 (MEDIUM): S-008 Blocks column corrected "—" → "S-009" per S-008.frontmatter blocks:[S-009] and Decision 1 (S-008→S-009 RingBuffer edge).
- Bidirectional sweep result: no additional asymmetries found beyond the 3 declared (full sweep of all 17 stories).
- SE-25 codification candidate: "Every depends_on entry must have a matching blocks entry on the depended-on story; sibling-sweep mandatory at every story-writer commit."
- STORY-INDEX version bumped v1.5→v1.6.

## §Trace v1.7

**Phase 2 r10 burst: Decision 11 applied** (2026-05-19):
- S-013, S-014 removed from S-001.blocks per consistency-validator Option A (matches S-011/S-012 corpus precedent of transitive-via-S-010). STORY-INDEX bumped 1.6→1.7 to reflect S-001 Blocks column update. F-PHASE2-R10-01 / GAP-PHASE2-R10-2 closure. Sibling-sweep: dep-graph §Trace v1.8 + holdout-scenarios §Trace v1.3 carry corresponding entries (this §Trace v1.7 closes the STORY-INDEX leg of the sibling-sweep).

## §Trace v1.8

**Phase 2 r12 fix-all burst: F-PHASE2-R12-01 + GAP-PHASE2-R12-1 closed** (2026-05-19):
- F-PHASE2-R12-01 (LOW): BC Coverage Table corrected for 9 drifted AC-range rows. Corrected by re-deriving from dep-graph BC Clause Coverage Matrix and actual story AC headers:
  - BC-2.01.002 (S-003): `AC-001..AC-007` → `AC-001, AC-005, AC-006, AC-007, AC-008` (PC-1 sub-bullets + PC-3; AC-003/AC-004 trace to BC-2.01.009)
  - BC-2.01.005 (S-006): `AC-001..AC-011` → `AC-001..AC-009` (AC-010..AC-013 anchor BC-2.01.010; AC-014 anchors BC-2.01.008)
  - BC-2.01.006 (S-007): `AC-001..AC-006` → `AC-001..AC-010` (S-007 has 10 ACs; AC-007..AC-010 cover postcondition 7 + invariants)
  - BC-2.01.009 (S-009): `AC-004..AC-010` → `AC-004..AC-009` (AC-010a anchors BC-2.01.008 PC-4; AC-010b anchors BC-2.01.002 PC-1 sub-bullet)
  - BC-2.01.010 (S-006): `AC-010..AC-011` → `AC-010..AC-013` (EC-010/EC-011/EC-012 covered by AC-010/AC-012/AC-013)
  - BC-2.02.002 (S-010): `AC-001..AC-005` → `AC-001, AC-002, AC-004` (AC-003 and AC-005 trace to BC-2.02.001)
  - BC-2.03.002 (S-015): `AC-001..AC-003, AC-009` → `AC-001..AC-004` (AC-004 = id() "claude-code" = BC-2.03.002 PC-3; AC-009 = BC-2.03.004 PC-3 preflight)
  - BC-2.03.003 (S-015): `AC-004..AC-005` → `AC-005, AC-006` (AC-004 = BC-2.03.002 PC-3; AC-005 = BC-2.03.003 PC-1; AC-006 = BC-2.03.003 PC-2)
  - BC-2.03.004 (S-015): `AC-006..AC-008` → `AC-007, AC-008, AC-009` (AC-006 = BC-2.03.003 PC-2; AC-007..AC-009 = BC-2.03.004 PC-1..PC-3)
- GAP-PHASE2-R12-1 (LOW): `level: L4` frontmatter field added to all 17 story files (S-001 through S-015, S-DTU-001, S-PHASE-3-PREP). Inserted after `document_type: story` line per STORY-INDEX/dep-graph/wave-schedule pattern.
- SE-22 v2 cascade: STORY-INDEX v1.7→v1.8; holdout-scenarios.md and sprint-state.yaml must update their traces_to_full/traces_to pins.

## §Trace v2.3

**F-PR2-R3-HIGH-1: S-001 v1.7 → v1.8 — main.rs no-op stub form** (2026-05-20):
- S-001 story spec updated: `println!("monocle-runtime stub")` removed per SS-conventions-anti-patterns.md
  v1.30.2 §Convention Checklist L503 ban. Canonical no-op stub form with `#![forbid(unsafe_code)]`
  and `#![deny(missing_docs)]` crate lints adopted. Source: PR #2 commit b7ed1e2 + adversary pass R3 HIGH-1.
- No story Registry table changes (no status/points/wave/blocks changes from this fix).
- STORY-INDEX version bumped v2.2 → v2.3.

## §Trace v2.2

**Phase 3.B Batch 6 — S-005 depends_on S-006 cascade** (2026-05-20):
- S-006 Blocks column updated: [S-007, S-008, S-009] → [S-005, S-007, S-008, S-009].
  Justification: S-005 lifecycle::exit_with() calls DaemonLock::release() from S-006
  before process termination per BC-2.01.004 PC-7 (F-E-02).
- No wave reassignment: S-005 remains Wave 2 (S-006 is also Wave 2; S-006 depends only on
  S-001 which is Wave 1; both are valid Wave 2 stories with no cross-wave inversion).
- No BC/VP/NFR coverage changes from this cascade.
- SE-22 v2 consumer-ledger: dep-graph v2.2→v2.3 (sibling).
- STORY-INDEX version bumped v2.1→v2.2.

## §Trace v2.1

**Phase 3.B Batch 3: arch-touching story remediation — STORY-INDEX cascade** (2026-05-20):
- S-013 depends_on downgraded `[S-010]` → `[S-001]` (F-E-01): monocle-proto does not consume
  monocle-core symbols; only inherits crate stub from S-001.
- Story Registry: S-001 Blocks column updated: added S-013. S-010 Blocks column updated: removed S-013.
- No wave reassignment: S-013 remains Wave 2 (depends on Wave 1 S-001; still satisfies Wave 2 constraint).
- No BC/VP/NFR coverage changes — S-013 coverage tables unaffected by depends_on edge change.
- SE-22 v2 consumer-ledger: dep-graph v2.1→v2.2 (sibling).
- STORY-INDEX version bumped v2.0→v2.1.

## §Trace v2.0

**Phase 3.B Batch 2 — Wave 2 small story cascade** (2026-05-20):
- S-003 Blocks column updated: [S-005, S-009] → [S-004, S-005, S-009, S-010] (bidirectional
  symmetry with S-004.depends_on and S-010.depends_on now including S-003).
- S-010 title updated: "monocle-core Crate + ABI Version Constant" →
  "Populate monocle-core ABI Version Constant (FC-03)" (S-001 creates the crate skeleton;
  S-010 only populates abi.rs per F-D-01).
- SE-22 v2 consumer-ledger: dep-graph v2.0→v2.1 (sibling).
- STORY-INDEX version bumped v1.9→v2.0.

## §Trace v1.9

**Phase 3.A auth-ownership decision — cascade propagation** (2026-05-20):
- S-003 F-E-03 (LOW-MED), S-005 F-E-01 (MED), S-009 F-E-01 (HIGH) closed.
- Story Registry: S-003 Blocks column updated "—" → "S-005, S-009" (bidirectional symmetry with
  S-005.depends_on and S-009.depends_on now including S-003).
- BC Coverage Table: no changes (no BC reassignments).
- No wave reassignments: S-003/S-005 both Wave 2; S-009 Wave 3. The new S-003→S-005 edge is
  an intra-Wave-2 ordering constraint; S-009 remains Wave 3. Wave point totals unchanged.
- SE-22 v2 consumer-ledger: dep-graph v1.9→v2.0 (sibling); sprint-state.yaml v1.4→v1.5 (sibling).
- STORY-INDEX version bumped v1.8→v1.9.

## §Trace v5.7

**Wave 6: S-023 flipped not_started → done + D-186** (2026-05-28):
- S-023 Story Registry row: `not_started` → `done`. PR #29 squash-merged at develop @ 7a52041 (2026-05-28T19:31:07Z).
- BC-2.05.006 (TUI reconnect backoff + lock re-read + offline mode) + BC-2.05.007 (SOQ-3 overlay clear on disconnect) fully satisfied. 15 ACs. 5 adversarial passes (3 consecutive NITPICK_ONLY convergence). 99 tests in monocle-ipc. 9/9 CI gates pass.
- F-ADV6-HIGH-001 (S-022 carry-over: slow-disconnect signal channel) production-grade resolved via commit 9bddd7b in S-023 PR.
- ADV-W4GATE-MED-001 (PATH isolation in detect_ccr) closed: migrated to temp_env::with_vars in commit 295dc1b (orchestrator-authorized scope expansion).
- Wave 6: 2/4 done (13/34 pts). S-025 in flight (parallel), S-026 still BLOCKED on S-025.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.29→v1.30 (done 25→26, not_started 7→6, points_complete 151→156); STATE.md v6.30→v6.31.
- STORY-INDEX version bumped v5.6→v5.7.

## §Trace v5.6

**F-S025-ADV4-BLOCKER-001 + BLOCKER-002 propagation** (2026-05-28):
- S-025 v1.5 → v1.6: BC-2.06.005 pin v1.0.0 → v1.0.5; AC-005 updated 6-column → 7-column (session_id added as first column).
- S-026 v1.4 → v1.5: BC-2.06.016 pin v1.0.6 → v1.0.7 (no body changes required).
- S-027 v1.2 → v1.3: BC-2.06.021 pin v1.0.0 → v1.0.4 (no body changes required).
- SS-tui.md pin v1.7.0 → v1.8.0 in STORY-INDEX and dependency-graph-expansion.md.
- BC-INDEX.md pin v1.23 → v1.25 in STORY-INDEX and dependency-graph-expansion.md.
- dependency-graph-expansion.md v1.5 → v1.6 (pin cascade).
- STORY-INDEX version bumped v5.5 → v5.6.

## §Trace v5.5

**F-S025-ADV3-BLOCKER-002 — SS-06 BC version pins propagated from PO sweep (commit 6d4fbb3)** (2026-05-28):
- 5 stories updated with inputs pin bumps and body corrections for BC-2.06.004 v1.2.0 shape change.
- S-024 v1.2 → v1.3: BC-2.06.001 v1.0.0→v1.0.4, BC-2.06.002 v1.0.0→v1.0.4 (cosmetic IPC field name sweep).
- S-025 v1.4 → v1.5: BC-2.06.004 v1.1.0→v1.2.0; AC-008 body fixed — `Overlay { stack: loaded_stack, prior }` → `Overlay { prior }` (AppMode::Overlay no longer carries stack field).
- S-026 v1.3 → v1.4: 9 BC pins updated (BC-2.06.008/009/011/012/013/014/016/023/024); AC-001 and Downstream Contract body fixed for same Overlay variant shape change.
- S-027 v1.1 → v1.2: BC-2.06.010/015/020/024 pins updated; AC-001 body fixed — `Overlay { stack, .. }` → `Overlay { .. }` with App.overlay_stack note.
- S-029 v1.0 → v1.1: BC-2.06.022 v1.0.0→v1.6.0 (no body changes required).
- STORY-INDEX version bumped v5.4→v5.5.

## §Trace v5.4

**Wave 6: S-022 flipped not_started → done** (2026-05-28):
- S-022 Story Registry row: `not_started` → `done`. PR #27 merged at develop @ c7540539.
- BC-2.05.002 + BC-2.05.005 fully satisfied. 15 ACs. 22 production-invoking integration tests across 4 test files. 15 adversarial passes (3 consecutive NITPICK_ONLY convergence). 8 implementer rounds + 2 architect interventions (Pass 2 Option B ring_tail Vec<HookEventRecord>; Pass 6 Option D at-least-once delivery + TUI prompt_id idempotency in S-025/S-026). 30+ findings closed.
- Wave 6: 1/4 done (8/34 pts). Unblocks S-023 + S-025 (parallel after S-022), S-026 (after S-023 + S-022).
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.28→v1.29 (done 24→25, not_started 8→7, points_complete 143→151); STATE.md v6.28→v6.29.
- STORY-INDEX version bumped v5.3→v5.4.

## §Trace v5.3

**F-S022-ADV8-HIGH-001 — BC-2.05.002 Invariant 4 propagated into S-025 and S-026** (2026-05-28):
- BC Coverage Table: BC-2.05.002 row updated — covering stories expanded from S-022 to S-022, S-025, S-026.
  S-025 AC-008 and S-026 AC-001 now explicitly cite BC-2.05.002 Invariant 4 (prompt_id idempotency).
- S-025 bumped v1.2 → v1.3; S-026 bumped v1.2 → v1.3.
- No story Registry table changes (no status/points/wave/blocks changes).
- SE-16d monotonicity: v5.3 timestamp 2026-05-28 >= v5.2 timestamp 2026-05-27. PASS.

## §Trace v5.2

**Wave 5 COMPLETE: S-018, S-019, S-020, S-021 flipped not_started → done** (2026-05-27):
- S-018 Story Registry row: `not_started` → `done`. PR #26 merged at develop @ 654e281. 46 tests. BC-2.04.007, BC-2.04.008, BC-2.04.009, BC-2.04.011 fully satisfied. Adversarial convergence: 3 passes, trajectory 10→4→4 (CONVERGED).
- S-019 Story Registry row: `not_started` → `done`. PR #25 merged at develop @ 11540fc. 25 tests (1 ignored). BC-2.04.002, BC-2.04.003 fully satisfied. Adversarial convergence: 3 passes, trajectory 7→2→1 (CONVERGED).
- S-020 Story Registry row: `not_started` → `done`. PR #24 merged at develop @ f69d53a. 24 tests. BC-2.04.012 fully satisfied. Fix commit 5a3eaf4 (restored ring.rs after S-018 merge conflict). Adversarial convergence: 3 passes, trajectory 12→8→0 (CONVERGED).
- S-021 Story Registry row: `not_started` → `done`. PR #23 merged at develop @ acaacb9. 49 tests. BC-2.05.001, BC-2.05.003, BC-2.05.004, BC-2.05.008 fully satisfied. New monocle-ipc crate. Adversarial convergence: 3 passes, trajectory 9→4→4 (CONVERGED).
- **WAVE 5 COMPLETE: 5/5 stories done (34/34 pts).** Wave gate pending.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.27→v1.28 (done 20→24, not_started 12→8, points_complete 109→143); STATE.md v6.22→v6.23.
- STORY-INDEX version bumped v5.1→v5.2.

## §Trace v5.1

**Wave 5: S-017 flipped not_started → done** (2026-05-27):
- S-017 Story Registry row: `not_started` → `done`. PR #22 merged at develop @ 06432cf.
- BC-2.04.001 and BC-2.04.010 fully satisfied. 29 tests. Adversarial convergence: 3 passes, trajectory 13→5→0.
- Wave 5: 1/5 done (8/34 pts). S-018 (Hook Routing), S-019 (Auto-Start), S-020 (Ring Capacity), S-021 (UDS Server) all unblocked for parallel delivery.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.26→v1.27 (done 19→20, not_started 13→12, points_complete 101→109); STATE.md v6.21→v6.22.
- STORY-INDEX version bumped v5.0→v5.1.

## §Trace v5.0

**Wave 4 COMPLETE: S-030 flipped not_started → done** (2026-05-27):
- S-030 Story Registry row: `not_started` → `done`. PR #21 merged at develop @ b8a4ab7.
- BC-2.07.001, BC-2.07.002, BC-2.07.003, BC-2.07.006 fully satisfied. 36 tests. New monocle-config crate. 3/3 adversary convergence.
- **WAVE 4 COMPLETE: 3/3 stories done (18/18 pts).** S-016 (PR #19, 87ac91f, 33 tests), S-024 (PR #20, d439c8b, 77 tests), S-030 (PR #21, b8a4ab7, 36 tests). Total 146 new tests added this wave. Wave gate pending.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.25→v1.26 (done 18→19, not_started 14→13, points_complete 96→101); STATE.md updated.
- STORY-INDEX version bumped v4.9→v5.0.

## §Trace v4.8

**Wave 4: S-016 flipped not_started → done** (2026-05-27):
- S-016 Story Registry row: `not_started` → `done`. PR #19 merged at develop @ 87ac91fc.
- BC-2.04.004, BC-2.04.005, BC-2.04.006 fully satisfied. 33 tests. 5 review findings fixed. 3/3 adversary convergence.
- Wave 4: 1/3 stories done (5/18 pts). S-024 and S-030 remaining.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.23→v1.24 (done 16→17, not_started 16→15, points_complete 83→88); STATE.md updated.
- STORY-INDEX version bumped v4.7→v4.8.

## §Trace v4.7

**Mechanical pin cascade: BC-INDEX v1.22 → v1.23 (BC-2.06.014 + BC-2.06.017 keybinding fixes)** (2026-05-27):
- BC-INDEX.md input pin updated: `"1.22"` → `"1.23"`.
- STORY-INDEX version bumped v4.6 → v4.7.
- Sibling cascade: sprint-state.yaml v1.22→v1.23, dependency-graph-expansion.md v1.4→v1.5, EVAL-INDEX.md v1.3→v1.4.

## §Trace v4.6

**Pass 4 remediation — FocusSnapshot enum syntax + BC-2.06.024 title correction** (2026-05-27):
- F-P4-MED-001: S-025 AC-006 and S-028 AC-001/AC-007 corrected — FocusSnapshot struct destructuring syntax (`FocusSnapshot { panel: PanelId::Sessions, .. }`) replaced with enum variant syntax (`FocusSnapshot::Sessions`, `FocusSnapshot::EventRibbon`) per S-024 v1.2 redefinition of FocusSnapshot as `#[non_exhaustive]` enum. S-025 bumped v1.1→v1.2; S-028 bumped v1.1→v1.2.
- F-P4-MED-002: BC Coverage Table BC-2.06.024 title corrected — "Permission Overlay: Tool Payload Rendering by Type" → "Permission Overlay: ToolPayload Body Rendering by Variant" (BC-INDEX authoritative H1 title).
- OBS-P4-001: Sprint-state S-026 notes BC list updated to include BC-2.06.024 (sprint-state.yaml v1.21→v1.22).
- STORY-INDEX version bumped v4.5→v4.6.

## §Trace v4.4

**Phase 2 Adversarial Review Pass 3 — version pin updates + S-024 FocusSnapshot correction** (2026-05-27):
- F-P3-CRIT-002: S-024 AC-002 corrected — FocusSnapshot redefined from struct (with `panel:
  PanelId` and `row: Option<usize>` fields) to `#[non_exhaustive]` enum (with variants
  `Sessions`, `EventRibbon`) per BC-2.06.002 precondition 1 and SS-tui.md v1.7.0. Methods
  `cycle()` and `to_panel_id()` added per BC-2.06.002 preconditions 3-4. Architecture Compliance
  Rules and Downstream Consumer Contract sections updated accordingly. Story version bumped v1.1→v1.2.
- F-P3-MED-002: S-024 AC-010 trace corrected `BC-2.06.002 PC-1` → `BC-2.06.003 precondition 1`
  (BindingSource lives in BC-2.06.003, not BC-2.06.002). AC-011 trace corrected `BC-2.06.002 PC-2`
  → `BC-2.06.003 postcondition PC-4`. AC-012 trace corrected `BC-2.06.002 PC-3` → `BC-2.06.003 PC-2`.
- F-P3-HIGH-004: HS-EXP-007 `source_bcs` corrected `BC-2.07.006` → `BC-2.07.001`; EVAL-INDEX BC
  Coverage Traceability table row updated; EVAL-INDEX bumped v1.1→v1.2.
- OBS-P3-001: ARCH-INDEX.md pin updated `"1.0.11"` → `"1.0.16"`.
- OBS-P3-002: SS-tui.md pin updated `"1.6.0"` → `"1.7.0"`.
- STORY-INDEX version bumped v4.3→v4.4.

## §Trace v4.3

**Phase 2 Adversarial Review Pass 2 — BC Coverage Table re-anchoring for S-026** (2026-05-27):
- F-P2ADV-P2-001/002/009/010 (CRITICAL): BC Coverage Table rows BC-2.06.008..023 updated with correct AC references after S-026 v1.2 AC re-anchoring:
  - BC-2.06.008: AC-001..AC-002 → AC-001, AC-002, AC-016 (adds conversion AC-016)
  - BC-2.06.009: AC-003..AC-004 → AC-013, AC-014 (rotation ACs are now AC-013/AC-014 after renumber)
  - BC-2.06.011: AC-005..AC-006 → AC-003 (Accept-Once is now AC-003)
  - BC-2.06.012: AC-007..AC-008 → AC-004 (Accept-Always is now AC-004)
  - BC-2.06.013: AC-009..AC-010 → AC-005 (Reject is now AC-005)
  - BC-2.06.014: AC-011..AC-012 → AC-008 (Esc hide is now AC-008)
  - BC-2.06.016: AC-013..AC-014 → AC-011, AC-012 (disconnect/reconnect now AC-011/AC-012)
  - BC-2.06.023: AC-015..AC-016 → AC-006, AC-007, AC-015 (UUID removal is AC-006; no-op is AC-007; empty-stack is AC-015)
  - BC-2.06.024: covering stories expanded S-027 → S-026, S-027 (AC-016 in S-026 handles payload_to_modal() conversion)
- OBS-P2-003: BC-INDEX version pin updated 1.19 → 1.21 in frontmatter inputs.
- STORY-INDEX version bumped v4.2 → v4.3.

## §Trace v4.2

**PO BC-anchoring resolution — S-027 + S-028 + BC-2.06.024** (2026-05-27):
- S-027 v1.0 → v1.1: Re-anchored AC-003/AC-004/AC-006 from BC-2.06.017 to BC-2.06.024 ("Permission Overlay: Tool Payload Rendering by Type"). BC-2.06.017 removed from behavioral_contracts and inputs (no AC traces to its latency postconditions). BC-2.06.024 added to behavioral_contracts and inputs. Token Budget table updated (BC-2.06.017.md row removed, BC-2.06.024.md row added). traces_to field corrected.
- S-028 v1.0 → v1.1: Removed all `ServerToClient::SessionEvents` references (no such IPC variant). Data flow corrected to: `InitialState.ring_tail` (BC-2.05.002, on connect) + `HookEventReceived` messages (BC-2.05.004, streaming) with client-side `session_id` filtering per BC-2.05.004 invariant 3. AC-006/AC-008/AC-009 rewritten. Task list updated. Previous Story Intelligence section corrected. Library table updated. BC-2.05.002 and BC-2.05.004 added to behavioral_contracts and inputs. Token Budget updated (4 BC files, ~7,600 tokens).
- BC Coverage Table: BC-2.06.024 row added (S-027, AC-003/AC-004/AC-006). BC-2.06.017 row updated (no covering story; GAP-P2-005 registered). BC coverage note updated to 71/72.
- Gap Register: GAP-P2-005 added (L1, BC-2.06.017 latency budget, deferred to Phase 3 integration test infrastructure).
- STORY-INDEX version bumped v4.1 → v4.2.

## §Trace v4.1

**Phase 2 Adversarial Review Pass 1 — mechanical fixes** (2026-05-27):
- F-P2ADV-008: Wave 5 description updated — S-017 is serial prerequisite; S-018/S-019/S-020/S-021 parallel after S-017.
- F-P2ADV-014: Wave 6 description updated — S-022 is serial prerequisite; S-023+S-025 parallel after S-022; S-026 after S-023+S-022.
- STORY-INDEX version bumped v4.0→v4.1.

## §Trace v4.0

**Phase 2 Expansion Burst: 16 new stories (S-016..S-031) integrated** (2026-05-27T00:00:00Z):

- 4 new epics added: EPIC-04 (Daemon Wiring), EPIC-05 (IPC), EPIC-06 (TUI), EPIC-07 (Config).
- 16 new stories added (S-016 through S-031), all status: not_started.
- 49/49 new BCs covered (100%): BC-2.04.001..012, BC-2.05.001..008, BC-2.06.001..023, BC-2.07.001..006.
- 4 new waves added: Wave 4 (18 pts), Wave 5 (34 pts), Wave 6 (34 pts), Wave 7 (23 pts).
- Total points: 86 (Waves 0-3) + 109 (Waves 4-7) = 195 total.
- Total stories: 33 (17 existing + 16 new).
- Dependency graph is acyclic (topological sort verified in dependency-graph-expansion.md).
- Critical path: S-001 → S-016 → S-017 → S-021 → S-022 → S-026 → S-027 → S-029.
- BC-INDEX inputs updated: v1.13 → v1.19 (expanded to 71 product BCs + 41 DTU BCs).
- sprint-state.yaml bumped v1.18 → v1.19 (16 new not_started entries; total_stories 17→33).
- dependency-graph-expansion.md created (new file for Waves 4-7 expansion dependency graph).
- STORY-INDEX version bumped v3.0 → v4.0.

## §Trace v3.0

**Wave 3 Batch B: S-009 flipped draft→done — WAVE 3 COMPLETE** (2026-05-27):
- S-009 Story Registry row: `draft` → `done`. PR #18 merged at develop @ d683c16.
- BC-2.01.008 + BC-2.01.009 fully satisfied. 26 auth tests + 2 hook integration tests. 7 adversary rounds (5→1→1→1→0→0→0, 3/3 convergence).
- Security: constant_time_eq all paths, INV-7 sentinel, WARN before validate on alias path.
- **WAVE 3 COMPLETE: all 5 Wave 3 stories done** (S-007, S-008, S-009, S-012, S-015 — 34/34 pts). Total project: 16/17 done, 83/86 pts. S-PHASE-3-PREP remains blocked on upstream (non-blocking).
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.17→v1.18 (done 15→16, not_started 1→0, points_complete 75→83); STATE.md v6.06→v6.07.
- STORY-INDEX version bumped v2.9→v3.0.

## §Trace v2.9

**Wave 3 Batch A: S-015 flipped draft→done** (2026-05-26):
- S-015 Story Registry row: `draft` → `done`. PR #17 merged at develop @ 23cfd44.
- BC-2.03.001..004 fully satisfied. 20 tests (18 detect/id/hook/paths + 2 HomeUnresolvable with E-ENG-001 log assertion). 4 adversary rounds (4→2→0→0, 3/3 convergence).
- Wave 3 Batch A COMPLETE: 4/4 stories done (S-007, S-008, S-012, S-015). S-009 is the only remaining Wave 3 story.
- No wave/points/BC coverage changes — story remains Wave 3, 8 pts, BC-2.03.001..004.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.16→v1.17 (done 14→15, not_started 2→1, points_complete 67→75); STATE.md v6.05→v6.06.
- STORY-INDEX version bumped v2.8→v2.9.

## §Trace v2.8

**Wave 3 Batch A: S-012 flipped draft→done** (2026-05-26):
- S-012 Story Registry row: `draft` → `done`. PR #16 merged at develop @ 599cd8c.
- BC-2.02.004 + BC-2.02.005 fully satisfied. 34 tests (12 AST audit + 22 integration). 4 adversary rounds (6→0→0→0, 3/3 convergence).
- No wave/points/BC coverage changes — story remains Wave 3, 8 pts, BC-2.02.004+BC-2.02.005.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.15→v1.16 (done 13→14, not_started 3→2, points_complete 59→67); STATE.md v6.04→v6.05.
- STORY-INDEX version bumped v2.7→v2.8.

## §Trace v2.7

**Wave 3 Batch A: S-008 flipped draft→done** (2026-05-26):
- S-008 Story Registry row: `draft` → `done`. PR #15 (S-008) merged at fe4db96 on develop.
- BC-2.01.007 fully satisfied. 13 integration tests. 5 adversary rounds (convergence with spec-text residual).
- S-009 dependency on S-008 is now satisfied — S-009 UNBLOCKED.
- No wave/points/BC coverage changes — story remains Wave 3, 5 pts, BC-2.01.007.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.14→v1.15 (done 12→13, not_started 4→3, points_complete 54→59); STATE.md v6.03→v6.04.
- STORY-INDEX version bumped v2.6→v2.7.

## §Trace v2.6

**Wave 3 Batch A: S-007 flipped draft→done** (2026-05-26):
- S-007 Story Registry row: `draft` → `done`. PR #14 (S-007) merged at 9982ff0 on develop.
- BC-2.01.006 fully satisfied at library level. 15 integration tests. 5 adversary rounds (3/3 convergence).
- No wave/points/BC coverage changes — story remains Wave 3, 5 pts, BC-2.01.006.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.13→v1.14 (done 11→12, not_started 5→4, points_complete 49→54); STATE.md v6.02→v6.03.
- STORY-INDEX version bumped v2.5→v2.6.

## §Trace v2.5

**F-WAVE1-003: S-DTU-001 status flipped ready→done** (2026-05-21):
- S-DTU-001 Story Registry row: `ready` → `done`. PR #3 (S-DTU-001) merged at cfeb1346 on develop.
- sprint-state.yaml was correctly updated in commit 69930c3; STORY-INDEX was missed (sibling-sweep gap surfaced by wave-1 wave-gate adversary review).
- SE-22 v2 sibling-sweep: no other artifacts carry a stale `ready` reference for S-DTU-001 (sprint-state.yaml already correct; S-DTU-001 story file status field is `done`; dep-graph and wave-schedule have no status fields; holdout-scenarios and BC-INDEX carry no story status fields).
- STORY-INDEX version bumped v2.4→v2.5. Closes F-WAVE1-003.
