---
document_type: story-index
level: L4
version: "5.53"
status: active
producer: vsdd-factory:state-manager
timestamp: 2026-06-19T01:00:00Z
phase: 2
inputs:
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/domain-spec/L2-INDEX.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/dtu-assessment.md
  - .factory/specs/prd-supplements/nfr-catalog.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/architecture/SS-daemon-wiring.md
  - .factory/specs/architecture/SS-ipc.md
  - .factory/specs/architecture/SS-tui.md
  - .factory/specs/architecture/SS-config.md
input-hash: "[live-state]"
traces_to: .factory/specs/prd.md
---

# Story Index: monocle Phase 2

> **Source of truth** for all story IDs, status, points, wave assignments, and BC/VP traceability.
> Per artifact-path-registry.yaml: stories at `.factory/stories/S-{story-id}-{slug}.md`.

## Epics

| Epic ID | Name | Capability | Subsystem | Stories |
|---------|------|-----------|-----------|---------|
| EPIC-01 | Daemon Lifecycle | CAP-001 | SS-01 | S-001, S-002, S-003, S-004, S-005, S-006, S-007, S-008, S-009 |
| EPIC-02 | Core Types and ABI | CAP-002 | SS-02 | S-010, S-011, S-012, S-013 |
| EPIC-03 | Engine Module | CAP-003 | SS-03 | S-014, S-015, S-045 |
| EPIC-04 | Daemon Wiring | CAP-004 | SS-04 | S-016, S-017, S-018, S-019, S-020, S-DAEMON-WIRE-FIX-001 |
| EPIC-05 | IPC | CAP-005 | SS-05 | S-021, S-022, S-023, S-032, S-046, S-047 |
| EPIC-06 | TUI | CAP-006 | SS-06 | S-024, S-025, S-026, S-027, S-028, S-029, S-048 |
| EPIC-07 | Config | CAP-007 | SS-07 | S-030, S-031 |
| EPIC-08 | Session Manager | CAP-008 | SS-08 | S-033, S-034, S-035, S-036, S-037, S-038 |
| EPIC-09 | Embedded PTY | CAP-009 | SS-09 | S-039, S-040, S-041, S-042, S-043, S-044 |
| EPIC-DTU | Claude Code Hook Protocol Clone | CAP-001 (DTU) | — | S-DTU-001 |
| EPIC-PREP | Phase 3 Pre-Implementation Prep | — | — | S-PHASE-3-PREP |

## Story Registry

| Story ID | Title | Epic | Points | Wave | Status | Blocks |
|----------|-------|------|--------|------|--------|--------|
| S-PHASE-3-PREP | spec-kit-mcp Integration Sweep | EPIC-PREP | 3 | 0 | draft | (Phase 3 gate) |
| S-DTU-001 | Claude Code Hook Protocol DTU Clone | EPIC-DTU | 3 | 1 | done | S-009 |
| S-001 | Cargo Workspace Init + CI/DevOps Setup | EPIC-01 | 5 | 1 | done | S-002, S-003, S-004, S-005, S-006, S-009, S-010, S-013, S-016, S-030 |
| S-002 | Healthz Endpoint | EPIC-01 | 3 | 2 | done | S-003, S-005, S-018 |
| S-003 | Status Endpoint | EPIC-01 | 5 | 2 | done | S-004, S-005, S-009, S-010, S-018 |
| S-004 | Body Size Limit | EPIC-01 | 2 | 2 | done | S-009, S-018 |
| S-005 | Graceful Shutdown | EPIC-01 | 5 | 2 | done | S-DAEMON-WIRE-FIX-001 |
| S-006 | Lock File Atomic Lifecycle | EPIC-01 | 8 | 2 | done | S-005, S-007, S-008, S-009, S-016, S-017 |
| S-009 | Auth Token Wire Format + Header Validation | EPIC-01 | 8 | 3 | done | S-017, S-018 |
| S-010 | Populate monocle-core ABI Version Constant (FC-03) | EPIC-02 | 5 | 2 | done | S-011, S-012, S-014 |
| S-011 | Non-Exhaustive Enum Policy | EPIC-02 | 3 | 2 | done | S-012, S-024 |
| S-013 | HookEnvelope Proto Wire Format | EPIC-02 | 5 | 2 | done | S-021 |
| S-014 | EngineModule Trait Definition | EPIC-03 | 5 | 2 | done | S-015, S-018, S-021, S-024, S-033 |
| S-007 | Crash Recovery Checkpoint | EPIC-01 | 5 | 3 | done | — |
| S-008 | JSONL Ring Format Version | EPIC-01 | 5 | 3 | done | S-009, S-017, S-020 |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | EPIC-02 | 8 | 3 | done | S-017 |
| S-015 | ClaudeCodeModule Implementation | EPIC-03 | 8 | 3 | done | S-017, S-033, S-045 |
| S-016 | Daemon Binary Crate Init + CLI Subcommands | EPIC-04 | 5 | 4 | done | S-017, S-019, S-DAEMON-WIRE-FIX-001 |
| S-024 | TUI Core Types: AppMode, Action, FocusSnapshot, transition(), 5-Level Dispatch | EPIC-06 | 8 | 4 | done | S-025, S-026, S-031 |
| S-030 | Config Crate: Atomic Write, Schema v1, Missing/Corrupted Default, CCR Detection | EPIC-07 | 5 | 4 | done | S-025, S-031 |
| S-017 | Daemon Start Sequence (SOQ-2) + Hook Tmpfile Generation | EPIC-04 | 8 | 5 | done | S-018, S-019, S-020, S-021, S-033, S-DAEMON-WIRE-FIX-001 |
| S-018 | Hook Endpoint Routing + Bounded Event Bus with Drop Counter | EPIC-04 | 8 | 5 | done | S-022, S-029, S-DAEMON-WIRE-FIX-001 |
| S-019 | Daemon Auto-Start on TUI Launch + MONOCLE_NO_AUTOSTART | EPIC-04 | 5 | 5 | done | S-023 |
| S-020 | JSONL Ring Capacity and Rotation Policy | EPIC-04 | 5 | 5 | done | — |
| S-021 | UDS Server Bind + IPC Transport + Core Message Types | EPIC-05 | 8 | 5 | done | S-022, S-028, S-032, S-033, S-039, S-046, S-047 |
| S-022 | TUI Client Connect, Initial State Push, and Permission Message Types | EPIC-05 | 8 | 6 | done | S-023, S-025, S-026, S-029, S-032, S-047, S-048 |
| S-023 | TUI Reconnect After Daemon Restart + SOQ-3 Overlay Clear | EPIC-05 | 5 | 6 | done | S-026, S-047 |
| S-025 | TUI Binary Skeleton, Ctrl-\ Popup Integration, and Sessions Panel | EPIC-06 | 8 | 6 | done | S-027, S-028, S-031, S-039, S-048 |
| S-026 | Permission Overlay: VecDeque Stack, Decision Keybindings, Esc Hide, SOQ-3 | EPIC-06 | 13 | 6 | done | S-027, S-029 |
| S-027 | Permission Overlay Rendering, Diff Preview (similar 3), Status Bar | EPIC-06 | 8 | 7 | done | S-029 |
| S-028 | Sessions Panel Nucleo Filter + Event Ribbon Rolling Log | EPIC-06 | 5 | 7 | done | S-032, S-048 |
| S-029 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | EPIC-06 | 5 | 7 | done | — |
| S-031 | Profile Picker: Sticky-Per-Project Selection + Ctrl-P Override | EPIC-07 | 5 | 7 | done | — |
| S-032 | Daemon Event-Bus Fan-Out: Broadcast HookEventReceived with daemon timestamp_micros | EPIC-05 | 5 | 8 | draft | S-046 |
| S-DAEMON-WIRE-FIX-001 | Second-Signal Exit Codes (SigtermDuringDrain=143, SigintDuringDrain=130) | EPIC-04 | 5 | 8 | draft | — |
| S-033 | SessionManager::spawn_session — SessionHostSpawner, SessionEntry, Sidecar, SpawnAck, and SessionStateChanged{Launching} | EPIC-08 | 8 | 8 | done | S-034, S-035, S-036, S-037, S-038, S-044, S-045, S-047, S-048 |
| S-034 | SessionManager::kill_session — DaemonToHost::Kill Within 500ms; Terminating/Terminated Transitions; 12s Watchdog | EPIC-08 | 8 | 8 | done | S-036, S-037, S-047 |
| S-035 | SessionManager::attach_session and detach_session — Chunked Scrollback, SO_PEERCRED, Session-Host Stays Alive | EPIC-08 | 8 | 8 | done | S-036, S-039, S-044, S-047 |
| S-036 | SessionManager::rediscover_sessions — setsid Persistence; All States Handled Within 5s; UDS Bind Blocked | EPIC-08 | 8 | 8 | done | — |
| S-037 | SessionManager GC Task — Terminated Sessions Removed After 10s Grace Period | EPIC-08 | 3 | 8 | done | — |
| S-038 | SessionManager Hook Auto-Injection — hooks-settings.json Writer + SpawnOptions.hooks_settings_path Population | EPIC-08 | 3 | 8 | done | — |
| S-039 | PTY Output Pipeline — vt100::Parser, PseudoTerminal Render, PtyOutput IPC Handler, Auto-Attach on First Entry | EPIC-09 | 8 | 9 | draft | S-040, S-042, S-043 |
| S-040 | Full-Fidelity Keyboard Forwarding — key_event_to_pty_bytes, Kitty Protocol CSI u, and Bracketed Paste | EPIC-09 | 8 | 9 | draft | S-041, S-044 |
| S-041 | Mouse Forwarding — mouse_event_to_pty_bytes, SGR 1006 Scoped Entry/Exit, Out-of-Pane Clip | EPIC-09 | 5 | 9 | draft | S-044 |
| S-042 | PTY Resize Detection, 50ms Debounce, and ResizePane IPC | EPIC-09 | 5 | 9 | draft | S-043 |
| S-043 | Scrollback Navigation — PtyScrollUp/Down, Per-Session Offsets, Configurable Capacity | EPIC-09 | 3 | 9 | draft | — |
| S-044 | EmbeddedTerminal + SessionCreation AppMode Transitions, SessionCreation Wizard, SpawnAck, and Permission Badge+Bell | EPIC-09 | 13 | 9 | draft | — |
| S-045 | ClaudeCodeModule::spawn_recipe() — Happy Path, CCR Injection, and Error Cases (Concrete Override Only; Default Trait Impl is S-033) | EPIC-03 | 5 | 8 | draft | — |
| S-046 | PtyOutput Fan-out Broker — Bounded Channel, Backpressure, and Client Lifecycle | EPIC-05 | 5 | 8 | draft | S-047 |
| S-047 | IPC Lifecycle Variants — Spawn/Kill/Detach/Attach/Rename/Input/Resize + Scrollback Protocol | EPIC-05 | 8 | 8 | draft | S-048 |
| S-048 | Sessions Panel — Multi-Project Grouping, Lifecycle Actions, and State-Aware Blocking | EPIC-06 | 8 | 8 | draft | — |

**Total stories:** 51 (49 product + 1 DTU + 1 prep)
**Total points (product):** 305 (excl. DTU 3 pts and PREP 3 pts)
**Total points (all):** 311

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
| Wave 8 | S-032, S-DAEMON-WIRE-FIX-001, S-033, S-034, S-035, S-036, S-037, S-038, S-045, S-046, S-047, S-048 | 74 | Session Manager + delta IPC/TUI + spawn recipe: S-033 is Wave-8 root (serial prerequisite within wave); S-034/S-035/S-036/S-037/S-038/S-045/S-046/S-047/S-048 serial after their Wave-8 deps |
| Wave 9 | S-039, S-040, S-041, S-042, S-043, S-044 | 42 | Embedded PTY: S-039 is Wave-9 root (serial prerequisite within wave); S-040/S-042 parallel after S-039; S-041 after S-040; S-043 after S-042; S-044 after S-040+S-041 |

## BC Coverage Table

| BC ID | Title | Covering Story | AC | Full Coverage? |
|-------|-------|---------------|----|----------------|
| BC-2.01.001 | Healthz Endpoint | S-002 | AC-001..AC-006 | YES |
| BC-2.01.002 | Status Endpoint | S-003 | AC-001, AC-005, AC-006, AC-007, AC-008 | YES |
| BC-2.01.003 | Body Size Limit | S-004 | AC-001..AC-006 | YES |
| BC-2.01.004 | Graceful Shutdown | S-005, S-DAEMON-WIRE-FIX-001 | S-005: AC-001..AC-006 (library-level drain + exit codes 0/1/2); S-DAEMON-WIRE-FIX-001: AC-001..AC-008 (second-signal exit codes 143/130 — PC-8/INV-4 second-signal paths deferred; code carries CONTRACT GAP marker) | PARTIAL (PC-8/INV-4 second-signal paths deferred to S-DAEMON-WIRE-FIX-001 Wave 8) |
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
| BC-2.05.002 | TUI Client Connects to UDS and Receives Initial State Push | S-022, S-025, S-026, S-028 | S-022: AC-001..AC-005; S-025: AC-008 (Invariant 4 idempotency); S-026: AC-001 (Invariant 4 idempotency); S-028: AC-006 (ring_tail backfill on connect) | YES |
| BC-2.05.003 | IPC Message Types: SessionListUpdate | S-021 | AC-004..AC-006 | YES |
| BC-2.05.004 | IPC Message Types: HookEventReceived | S-021, S-028, S-032 | S-021: AC-007..AC-009 (types + struct); S-028: AC-006, AC-009 (TUI consumer — HookEventReceived streaming per INV-3); S-032: AC-001..AC-007 (daemon producer path, PC-2/INV-4 timestamp_micros) | PARTIAL (daemon producer path deferred to S-032 Wave 8) |
| BC-2.05.005 | IPC Message Types: PermissionPromptQueued | S-022 | AC-006..AC-008 | YES |
| BC-2.05.006 | TUI Reconnects After Daemon Restart | S-023 | AC-001..AC-004 | YES |
| BC-2.05.007 | Overlay Stack Cleared on Daemon Disconnect (SOQ-3) | S-023 | AC-005..AC-006 | YES |
| BC-2.05.008 | UDS-Only in Phase 1 (No Shared-Memory Transport) | S-021 | AC-010..AC-011 | YES |
| BC-2.06.001 | AppMode State Machine: Compile-Time Mutual Exclusion | S-024 | AC-001..AC-004 | YES |
| BC-2.06.002 | FocusSnapshot: Focus Restored After Overlay/Fullscreen Close | S-024 | AC-005..AC-007 | YES |
| BC-2.06.003 | Action Dispatch: 5-Level Binding Precedence | S-024 | AC-008..AC-012 | YES |
| BC-2.06.004 | `Ctrl-\` Popup: Appears and Dismisses Without State Loss | S-025 | AC-001..AC-003 | YES |
| BC-2.06.005 | Sessions Panel: Session List Renders from IPC State | S-025 | AC-004..AC-006 | YES |
| BC-2.06.006 | Sessions Panel: `/` Filter with Nucleo Fuzzy Match | S-028 | AC-001..AC-005 | YES |
| BC-2.06.007 | Sessions Panel: `Enter` Transitions to Fullscreen | S-025 | AC-007..AC-008 | YES |
| BC-2.06.008 | Permission Overlay: VecDeque Stack Push on PermissionPromptQueued | S-026 | AC-001, AC-002, AC-016 | YES |
| BC-2.06.009 | Permission Overlay: `[↑↓]` Rotates Stack | S-026 | AC-013, AC-014 | YES |
| BC-2.06.010 | Permission Overlay: Diff Preview via `similar 3` | S-027 | AC-001, AC-002, AC-005, AC-011, AC-012 | YES |
| BC-2.06.011 | Permission Overlay: Accept-Once Keybinding | S-026 | AC-003 | YES |
| BC-2.06.012 | Permission Overlay: Accept-Always Keybinding | S-026 | AC-004 | YES |
| BC-2.06.013 | Permission Overlay: Reject Keybinding | S-026 | AC-005 | YES |
| BC-2.06.014 | Permission Overlay: `[Esc]` Hides Without Rejecting | S-026 | AC-008 | YES |
| BC-2.06.015 | Permission Overlay: `[t]` Trace-to-Source Stub | S-027 | AC-013 | YES |
| BC-2.06.016 | Permission Overlay: Cleared on Daemon Disconnect | S-026 | AC-011, AC-012 | YES |
| BC-2.06.017 | Permission Response Within Hook Timeout Budget | — | — | GAP (see GAP-P2-005) |
| BC-2.06.018 | Event Ribbon Panel: Rolling Hook Event Log | S-028 | AC-006..AC-010 | YES |
| BC-2.06.019 | Status Bar: Drop Counter Renders Under Load | S-027 | AC-008, AC-012 | YES |
| BC-2.06.020 | Status Bar: Breadcrumb | S-027 | AC-008, AC-009 | YES |
| BC-2.06.021 | Status Bar: Keybinding Hint Line | S-027 | AC-008, AC-010, AC-012 | YES |
| BC-2.06.022 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | S-029 | AC-001..AC-008 | YES |
| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | S-026 | AC-006, AC-007, AC-015 | YES |
| BC-2.06.024 | Permission Overlay: ToolPayload Body Rendering by Variant | S-026, S-027 | S-026: AC-016; S-027: AC-003, AC-004, AC-006 | YES |
| BC-2.07.001 | Config File Atomic Write via `tempfile::persist` | S-030 | AC-001..AC-002 | YES |
| BC-2.07.002 | Config Schema Version 1: Harness Profile Fields | S-030 | AC-003..AC-005 | YES |
| BC-2.07.003 | Config Missing or Corrupted: Default Applied | S-030 | AC-006..AC-008 | YES |
| BC-2.07.004 | Profile Picker: Sticky-Per-Project | S-031 | AC-003, AC-004, AC-008 | YES |
| BC-2.07.005 | Profile Picker: `Ctrl-P` Override Shows Picker | S-031 | AC-001, AC-002, AC-005..AC-007, AC-009, AC-010 | YES |
| BC-2.07.006 | CCR Detection via `ccr_path` Config Field | S-030 | AC-009..AC-010 | YES |

| BC-2.03.005 | ClaudeCodeModule.spawn_recipe() — Happy-Path Recipe Assembly | S-045 | AC-001, AC-002, AC-008, AC-009 | YES |
| BC-2.03.006 | ClaudeCodeModule.spawn_recipe() — CCR Base URL Injection | S-045 | AC-003, AC-004 | YES |
| BC-2.03.007 | spawn_recipe() Error Cases — BinaryNotFound and InvalidPath | S-045 | AC-005, AC-006, AC-007 | YES |
| BC-2.03.008 | Default spawn_recipe() Returns UnsupportedOperation | S-033 | AC-009c, AC-009d | YES |
| BC-2.05.009 | PtyOutput Fan-Out — Per-Session Bounded Channel (1024) with Drop Counter (stderr WARN) + PtyReset TUI Recovery | S-046 | AC-001..AC-008 | YES |
| BC-2.05.010 | New ClientToServer IPC Variants — SpawnSession, KillSession, KeyInput, ResizePane, DetachSession, RenameSession, AttachSession | S-047 | AC-001..AC-012 | YES |
| BC-2.05.011 | New ServerToClient IPC Variants — ScrollbackChunk, ScrollbackDumpComplete, PtyReset | S-046 (PtyReset variant + broker emission), S-047 (TUI handler + scrollback protocol: AC-007..AC-010) | S-046: AC-005 (PtyReset emit); S-047: AC-007..AC-010 (scrollback + TUI reset) | YES |
| BC-2.06.025 | Multi-Session / Multi-Project Sessions Panel — Grouped by Project, Fast Switching, TUI Lifecycle Actions | S-048 | AC-001..AC-014 (AC-013: PC-1 all-5-state indicator; AC-014: PC-2 Enter-on-Detached→AttachSession→EmbeddedTerminal) | YES |
| BC-2.08.001 | Session Spawn — SessionHostSpawner Called Within 2s; SessionEntry Created | S-033 | AC-001..AC-009b | YES |
| BC-2.08.002 | Session Persistence — session-host Survives Graceful Daemon Restart | S-036 | AC-001..AC-002, AC-015 | YES |
| BC-2.08.003 | Session Kill — SIGTERM Delivered via DaemonToHost::Kill Within 500ms | S-034 | AC-001..AC-011 | YES |
| BC-2.08.004 | Re-Discovery — All Alive Sessions Visible After Daemon Restart Within 5s; UDS Bind Blocked Until Complete | S-036 | AC-003..AC-014 | YES |
| BC-2.08.005 | Session GC — Terminated Sessions Removed from Registry After 10s Grace Period | S-037 | AC-001..AC-012 | YES |
| BC-2.08.006 | Hook Auto-Injection — `--settings` Arg Present in Session-Host Child Args Within 2s of Spawn | S-038 | AC-001..AC-013 | YES |
| BC-2.08.007 | Attach/Detach — Chunked Scrollback (ScrollbackChunk*+ScrollbackDumpComplete) on Attach; session-host Stays Alive on Detach | S-035 | AC-001..AC-014 | YES |
| BC-2.08.008 | SessionStateChanged — Daemon Emits on Every SessionState Transition; Delivered to All TUI Clients; Ordering Relative to SessionListUpdate | S-033, S-034, S-035 | S-033: AC-010..AC-012 (Launching); S-034: AC-012 (Terminating/Terminated); S-035: AC-015 (Running/Detached) | YES |
| BC-2.09.001 | PTY Output Renders Within 100ms of Byte Receipt at TUI | S-039 | AC-001..AC-007 | YES |
| BC-2.09.002 | Full-Fidelity Keyboard Forwarding — All v1A Input Classes Reach PTY stdin | S-040 | AC-001..AC-005, AC-011..AC-013 | YES |
| BC-2.09.003 | Mouse Events Forwarded to PTY in SGR Encoding When in EmbeddedTerminal | S-041 | AC-001..AC-006 | YES |
| BC-2.09.004 | Kitty Keyboard Protocol — Enhanced Key Events Forwarded as CSI u Sequences | S-040 | AC-006..AC-008, AC-014 | YES |
| BC-2.09.005 | Bracketed Paste — Paste Events Wrapped in Bracket Sequences Before Forwarding | S-040 | AC-009..AC-010 | YES |
| BC-2.09.006 | Resize — PTY and Parser Resized Within 2 Render Ticks of Pane Area Change; 50ms Debounce | S-042 | AC-001..AC-012 | YES |
| BC-2.09.007 | Scrollback — 1000 Rows Default; Configurable; PtyScrollUp/Down Navigate | S-043 | AC-001..AC-014 | YES |
| BC-2.09.008 | EmbeddedTerminal AppMode Enter/Exit Transitions; SessionCreation Wizard Auto-Transitions to EmbeddedTerminal | S-044 | AC-001..AC-015 | YES |
| BC-2.09.009 | Permission Badge+Bell — Status Bar Badge + Audible Bell Within One Render Tick While in EmbeddedTerminal or SessionCreation | S-044 | AC-016..AC-021 | YES |

| BC-HOOK-001 | PreToolUse Hook Fail-Open Semantics (No Server Found) | S-DTU-001 | AC-001 | YES |
| BC-HOOK-002 | Non-PreToolUse Hooks Fail-Closed (No Server Found) | S-DTU-001 | AC-001 | YES |
| BC-HOOK-003 | Notification Hook Filters on notification_type === 'permission_prompt' | S-DTU-001 | AC-001, AC-003 | YES |
| BC-HOOK-004 | Hook HTTP Requests Are Fire-and-Forget (Response Ignored) | S-DTU-001 | AC-001 | YES |
| BC-HOOK-005 | Hook HTTP Request Target is 127.0.0.1 with Port from Lock File | S-DTU-001 | AC-001, AC-002, AC-003, AC-006 | YES |
| BC-HOOK-006 | PreToolUse Always Echoes Stdin to Stdout | S-DTU-001 | AC-001, AC-003 | YES |
| BC-HOOK-007 | Exactly Five Hook Types Registered; PostToolUse Intentionally Absent | S-DTU-001 | AC-001 | YES |
| BC-HOOK-008..BC-HOOK-041 | Hooks-settings.json encoding, path, lifecycle, timeout, env, edge cases | S-DTU-001 | AC-001..AC-006 (fidelity gate) | YES |

**BC Coverage: 22/22 product BCs Waves 1-3 (100%); 49/50 product BCs Waves 4-7 (BC-2.06.017 deferred — see GAP-P2-005; BC-2.06.024 added); 25/25 v1A BCs Waves 8-9 (100%); 41/41 DTU gene-source BCs (100%); BC-2.05.004 PARTIAL — daemon producer path (PC-2/INV-4) deferred to S-032 Wave 8; BC-2.01.004 PARTIAL — second-signal exit-code paths (PC-8/INV-4) deferred to S-DAEMON-WIRE-FIX-001 Wave 8**
**Total product BC coverage: 94/97 (BC-2.06.017 gap — GAP-P2-005; BC-2.05.004 partial — S-032 Wave 8; BC-2.01.004 partial — S-DAEMON-WIRE-FIX-001 Wave 8). Full coverage when S-032 and S-DAEMON-WIRE-FIX-001 both delivered. v1A BCs (SS-03 delta + SS-05 delta + SS-06 delta + SS-08 + SS-09): 25/25 (100%).**

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

## §Trace v5.53

**S-039 Pass-6 LOW + input-pin freshness (F-S039-P6-LOW-001, 2026-06-20):**

- S-039-pty-output-pipeline.md v1.6→v1.7. Input pins refreshed: BC-2.09.001 1.7.0→1.7.1; SS-embedded-pty 1.9.0→1.10.0.
- AC-005 step 5: `dump_in_progress.insert(session_id.clone(), false)` corrected to `dump_in_progress.remove(&session_id)` (per BC-2.09.001 Inv-5 step d; removal avoids stale entries). Same fix applied to Tasks list item.
- SE-16d monotonicity: v5.53 timestamp 2026-06-20 >= v5.52 timestamp 2026-06-20. PASS.
- SE-22 v2 sibling-sweep: version-pin-registry.yaml S-039-pty-output-pipeline 1.5→1.7; STORY-INDEX 5.52→5.53.

## §Trace v5.52

**S-039 adversarial finding remediation — F-S039-004/005/006/011 (2026-06-20):**

- S-039-pty-output-pipeline.md v1.3→v1.4. Input pins refreshed: BC-2.09.001 1.3.5→1.4.0; SS-embedded-pty 1.7.0→1.8.0.
- F-S039-005/006: AC-005 step 2 replaced — styled-cell reconstruction from ScrollbackChunk deferred to S-047 scope; S-039 does NOT read total_chunks/cursor_row/cursor_col from ScrollbackDumpComplete. S-047 extension-point comment added.
- F-S039-004: AC-005 notes enter_embedded_terminal is async; AttachSession sent via .send().await with full rollback on failure per BC-2.09.001 Inv-3.
- F-S039-011: IPC server-message handler call-site corrected throughout S-039 — PtyOutput/ScrollbackDumpComplete arms live in app.rs::handle_server_message, NOT event_loop.rs. File Structure table consolidated; Tasks updated.
- S-040 and S-041: reviewed; all event_loop.rs references in those stories are for keyboard/mouse crossterm dispatch (genuinely in event_loop.rs) — no corrections required.
- SE-16d monotonicity: v5.52 timestamp 2026-06-20 >= v5.51 timestamp 2026-06-20. PASS.
- SE-22 v2 sibling-sweep: version-pin-registry.yaml S-039-pty-output-pipeline 1.3→1.4.

## §Trace v5.51

**S-036 MERGED — Wave-8 sixth delivery, Tier-3 COMPLETE (D-339, 2026-06-20):**

- S-036 Story Registry row: `draft` → `done`. PR #46 @ d924183 (squash-merge 2026-06-20).
- 12-pass fresh-context adversarial convergence (3 consecutive CLEAN: passes 10/11/12). Security review PASS (SEC-001/002/003 all RESOLVED IN-SCOPE). 11/11 CI checks green.
- rediscover_sessions: BC-2.08.002 (setsid persistence) + BC-2.08.004 v1.4.0 (all alive sessions visible within 5s; UDS bind blocked). MED-002 RESOLVED for S-036 scope.
- SS-session-manager v2.15.0→v2.15.1; BC-2.08.004 v1.3.5→v1.4.0; EVAL-INDEX v1.26→v1.28; S-036 v1.3→v1.5.
- AC-004 spec-text drift corrected (proxy_task: Some→None). PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP recurred (caught+reset by orchestrator).
- Wave 8: 6/12 stories done (38/74 pts). 38/51 stories done (230/311 pts). develop HEAD: d924183.
- SE-16d monotonicity: v5.51 timestamp 2026-06-20 >= v5.50 timestamp 2026-06-19. PASS.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.51→v1.52 (done 37→38, draft 13→12, points_complete 222→230); STATE.md v8.03→v8.04.

## §Trace v5.50

**S-038 MERGED — Wave-8 fifth delivery, Tier-2 COMPLETE (D-338, 2026-06-19):**

- S-038 Story Registry row: `draft` → `done`. PR #44 @ 8d649ea (squash-merge 2026-06-19).
- 6-pass adversarial convergence (3 consecutive CLEAN: passes 4/5/6). Security PASS. 11/11 CI checks green.
- Single-writer mandate: lifecycle step 9 calls write_hooks_settings_json (sole canonical writer). lock.app='monocle' mandatory.
- BC-2.08.006→v1.5.0; BC-2.04.010→v1.4.0 (PC-3 lock.app added). SS-session-manager→v2.15.0; BC-2.08.007→v1.5.6 (arch-source cascade).
- SEC-001 (CWE-732) + SEC-002 (CWE-532) fixed in-scope (daeb4f2). Follow-up chore PR #45 @ 7f005af. develop HEAD: 7f005af.
- Wave 8: 5/12 stories done (30/74 pts). WAVE-8 TIER-2 COMPLETE. 37/51 stories done (222/311 pts).
- SE-16d monotonicity: v5.50 timestamp 2026-06-19 >= v5.49 timestamp 2026-06-19. PASS.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.50→v1.51 (done 36→37, draft 14→13, points_complete 219→222); STATE.md v8.01→v8.02.

## §Trace v5.49

**S-035 MERGED — Wave-8 fourth delivery (D-336, 2026-06-19):**

- S-035 Story Registry row: `draft` → `done`. PR #43 @ 270b7d4 (squash-merge 2026-06-19).
- 9-pass adversarial convergence (3 consecutive CLEAN: passes 7/8/9). Security PASS. 11 CI checks green.
- attach_session/detach_session. CRIT-001 silent Detached→Terminated fixed. Ruling L proxy_task kill-reader.
- SS-session-manager v2.11.0→v2.14.0; BC-2.08.007 v1.5.5; BC-2.08.008 v1.3.7. S-036 UNBLOCKED.
- Wave 8: 4/12 stories done (27/74 pts). 36/51 stories done (219/311 pts).
- SE-16d monotonicity: v5.49 timestamp 2026-06-19 >= v5.48 timestamp 2026-06-19. PASS.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.49→v1.50 (done 35→36, draft 15→14, points_complete 211→219); STATE.md v7.99→v8.00.

## §Trace v5.48

**S-037 MERGED — Wave-8 third delivery (D-335, 2026-06-19):**

- S-037 Story Registry row: `draft` → `done`. PR #42 @ a7e4081 (squash-merge 2026-06-19).
- 7-pass adversarial convergence (3 consecutive CLEAN: passes 5/6/7). Security PASS. 11 CI checks green.
- GC task (10s grace period) + rename_session. SEC-001 (CWE-20) + SEC-002 (CWE-706) fixed in-scope (b2d65db).
- Wave 8: 3/12 stories done (19/74 pts). 35/51 stories done (211/311 pts).
- SE-16d monotonicity: v5.48 timestamp 2026-06-19 >= v5.47 timestamp 2026-06-18. PASS.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.48→v1.49 (done 34→35, draft 16→15, points_complete 208→211); STATE.md v7.98→v7.99.

## §Trace v5.47

**S-034 MERGED — Wave-8 second delivery (D-334, 2026-06-18):**

- S-034 Story Registry row: `draft` → `done`. PR #41 @ 4dfe0db (squash-merge 2026-06-18).
- 18-pass adversarial convergence (3 consecutive CLEAN). Security PASS. 11 CI checks green (admin-override merge).
- Kill path: DaemonToHost::Kill, Terminating/Terminated, 12s watchdog. SS-session-manager v2.11.0.
- Wave 8: 2/12 stories done (16/74 pts). 34/51 stories done (208/311 pts).
- SE-16d monotonicity: v5.47 timestamp 2026-06-18 >= v5.46 timestamp 2026-06-17. PASS.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.46→v1.48 (done 33→34, draft 17→16, points_complete 200→208); STATE.md v7.95→v7.98.

## §Trace v5.46

**S-033 MERGED — Wave-8 first delivery (D-332, 2026-06-17):**

- S-033 Story Registry row: `draft` → `done`. PR #40 @ c7e10f2 (squash-merge 2026-06-17).
- 7-pass adversarial convergence (3 consecutive clean, D-331). Security PASS. 10 CI checks green.
- 10th workspace crate monocle-session-host. Wire types: SessionSidecarV3/DaemonToHost/HostToDaemon.
- Wave 8: 1/12 stories done (8/74 pts). Tier 2 unblocked: S-034/S-035/S-037/S-038/S-045.
- SE-16d monotonicity: v5.46 timestamp 2026-06-17 >= v5.45 timestamp 2026-06-16. PASS.
- SE-22 v2 sibling-sweep: sprint-state.yaml v1.46→v1.47 (done 32→33, draft 18→17, points_complete 192→200); STATE.md v7.94→v7.95.

## §Trace v5.45

**F-P25-SUG-001: BC Coverage Table AC-range corrections for S-040 and S-044** (2026-06-16):

- **BC-2.09.002 (S-040):** `AC-001..AC-005` → `AC-001..AC-005, AC-011..AC-013` (edge-case ACs AC-011/EC-210, AC-012/EC-214, AC-013/EC-215 all trace to BC-2.09.002 but were omitted from the range).
- **BC-2.09.004 (S-040):** `AC-006..AC-009` → `AC-006..AC-008, AC-014` (AC-014/EC-228 traces to BC-2.09.004; AC-009 traces to BC-2.09.005 not BC-2.09.004).
- **BC-2.09.005 (S-040):** `AC-010..AC-013` → `AC-009..AC-010` (AC-009 and AC-010 trace to BC-2.09.005; AC-011..AC-013 trace to BC-2.09.002 edge cases).
- **BC-2.09.008 (S-044):** `AC-001..AC-007` → `AC-001..AC-015` (S-044 has 15 ACs for BC-2.09.008: PC-1..PC-7, invariant 1, and edge cases EC-250..EC-253/Esc).
- **BC-2.09.009 (S-044):** `AC-008..AC-011` → `AC-016..AC-021` (S-044 permission badge ACs are AC-016..AC-021).
- SE-16d monotonicity: v5.45 timestamp 2026-06-16 >= v5.44 timestamp 2026-06-16. PASS.

## §Trace v5.44

**Phase-2 Pass-16 fix burst: BC-2.06.025 AC range AC-001..AC-012→AC-001..AC-014 (F-P16-IMP-001/002)** (2026-06-16):

- **F-P16-IMP-001 (S-033 SessionState crate-residency):** BC Coverage Table row for BC-2.06.025 updated: `AC-001..AC-012` → `AC-001..AC-014` (S-048 now has 14 ACs; AC-013 traces to BC-2.06.025 PC-1 all-5-state indicator; AC-014 traces to BC-2.06.025 PC-2 Enter-on-Detached→AttachSession→EmbeddedTerminal).
- **F-P16-IMP-002 (S-048 Detached coverage):** AC range annotation updated inline to capture AC-013 and AC-014 purpose.
- SE-16d monotonicity: v5.44 timestamp 2026-06-16 >= v5.43 timestamp 2026-06-16. PASS.

## §Trace v5.43

**Phase-2 Pass-15 fix burst: S-045 title sync (F-P15-IMP-001)** (2026-06-16):

- **F-P15-IMP-001 (S-045 Registry title sibling lag):** Registry table row title for S-045 synced to canonical H1 in S-045-claude-code-spawn-recipe.md. Stale title "Happy Path, CCR Injection, Error Cases, and Default Trait Impl" replaced with "Happy Path, CCR Injection, and Error Cases (Concrete Override Only; Default Trait Impl is S-033)". BC-2.03.008 anchor to S-033 unchanged. No other STORY-INDEX surface contained the stale title.
- **NOTE FOR STATE-MANAGER:** RESOLVED — sprint-state.yaml S-045 title synced in same Pass-15 fix burst commit (sprint-state v1.44→v1.45).
- SE-16d monotonicity: v5.43 timestamp 2026-06-16 >= v5.42 timestamp 2026-06-16. PASS.

## §Trace v5.42

**SE-25 global bidirectional dependency-symmetry reconciliation — done-story back-propagation + Wave 8 cross-edges** (2026-06-16):

- **Scope:** Complete global pass over all 51 stories. 23 stories updated; 0 depends_on changes (all changes are blocks additions/removals only). Topological sort ACYCLIC confirmed. No status/points/wave changes.
- **S-001.blocks:** Added S-013, S-016, S-030 (S-013/S-016/S-030 all have depends_on:[...,S-001]). v1.10→v1.11.
- **S-002.blocks:** Added S-018 (S-018.depends_on includes S-002). v1.1→v1.2.
- **S-003.blocks:** Added S-018 (S-018.depends_on includes S-003). v1.8→v1.9.
- **S-004.blocks:** Added S-018 (S-018.depends_on includes S-004). v1.1→v1.2.
- **S-005.blocks:** Added S-DAEMON-WIRE-FIX-001 (S-DAEMON-WIRE-FIX-001.depends_on includes S-005). v1.7→v1.8.
- **S-006.blocks:** Added S-016, S-017 (S-016/S-017.depends_on includes S-006). v1.7→v1.8.
- **S-008.blocks:** Added S-017, S-020 (S-017/S-020.depends_on includes S-008). v1.4→v1.5.
- **S-009.blocks:** Added S-017, S-018 (S-017/S-018.depends_on includes S-009). v1.8→v1.9.
- **S-010.blocks:** Removed S-013 (S-013.depends_on=[S-001] only — S-010 is NOT a direct prerequisite of S-013). v1.3→v1.4. STORY-INDEX Registry already had S-013 removed; frontmatter now matches.
- **S-011.blocks:** Added S-024 (S-024.depends_on includes S-011). v1.3→v1.4.
- **S-012.blocks:** Added S-017 (S-017.depends_on includes S-012). v1.5→v1.6.
- **S-013.blocks:** Added S-021 (S-021.depends_on includes S-013). v1.3→v1.4.
- **S-014.blocks:** Added S-018, S-021, S-024, S-033 (all have depends_on:[...,S-014]). v1.7→v1.8.
- **S-015.blocks:** Added S-017, S-033, S-045 (all have depends_on:[...,S-015]). v1.8→v1.9.
- **S-016.blocks:** Added S-DAEMON-WIRE-FIX-001 (S-DAEMON-WIRE-FIX-001.depends_on includes S-016). v1.1→v1.2.
- **S-017.blocks:** Added S-033, S-DAEMON-WIRE-FIX-001 (both have depends_on:[...,S-017]). v1.1→v1.2.
- **S-018.blocks:** Added S-DAEMON-WIRE-FIX-001 (S-DAEMON-WIRE-FIX-001.depends_on includes S-018). v1.2→v1.3.
- **S-021.blocks:** Added S-028, S-032, S-033, S-039, S-046, S-047 (all have depends_on:[...,S-021]). v1.1→v1.2.
- **S-022.blocks:** Added S-032, S-047, S-048 (all have depends_on:[...,S-022]). v1.4→v1.5.
- **S-023.blocks:** Added S-047 (S-047.depends_on includes S-023). v1.2→v1.3.
- **S-025.blocks:** Added S-039, S-048 (both have depends_on:[...,S-025]). v1.14→v1.15.
- **S-028.blocks:** Added S-032, S-048 (both have depends_on:[...,S-028]). v1.6→v1.7.
- **S-032.blocks:** Added S-046 (S-046.depends_on includes S-032). v1.0→v1.1.
- **STORY-INDEX Registry Blocks columns:** Updated for all 23 affected stories to match frontmatter exactly.
- **dependency-graph-expansion.md:** Adjacency table updated for all affected Wave 4-7 stories; Wave 8-9 adjacency table corrected for S-021 back-propagation. SE-25 §Trace v2.3 added. dep-graph bumped v2.2→v2.3.
- SE-16d monotonicity: v5.42 timestamp 2026-06-16 >= v5.41 timestamp 2026-06-16. PASS.

## §Trace v5.41

**Phase-2 Pass-9 fix burst: S-033 BC-2.03.008 PC-3 coverage (AC-009d) + BC Coverage Title normalization** (2026-06-16):

- **F-PASS9-CRIT-001 (S-033 BC-2.03.008 PC-3 gap):** Added AC-009d to S-033 covering BC-2.03.008 PC-3 — the canonical wire code `"spawn_unsupported"` mapping for `EngineError::UnsupportedOperation`. AC-009d traces to BC-2.03.008 postcondition 3 and includes the EC-112 integration test vector (`test_BC_2_03_008_EC_112_unsupported_operation_maps_to_spawn_unsupported`). The `session_error_to_code()` task entry corrected to require `EngineError::UnsupportedOperation(_) => "spawn_unsupported"` BEFORE the `_ => "invalid_request"` fallback (per SS-session-manager.md §session_error_to_code). Architecture Compliance Rule corrected identically. S-033 bumped v1.2→v1.3. BC Coverage Table BC-2.03.008 AC range updated: `AC-009c` → `AC-009c, AC-009d`. Coverage status remains YES.
- **F-PASS9-SUG-001 (BC Coverage Title normalization):** 26 BC Coverage Table Title cells normalized to verbatim BC-INDEX H1 text. Affected epics: EPIC-03 (BC-2.03.005..007, BC-2.03.008), EPIC-05 (BC-2.05.009..011), EPIC-06 (BC-2.06.025), EPIC-08 (BC-2.08.001..008), EPIC-09 (BC-2.09.001..009). No AC ranges or Coverage status changed — title-column alignment only.
- SE-16d monotonicity: v5.41 timestamp 2026-06-16 >= v5.40 timestamp 2026-06-16. PASS.

## §Trace v5.40

**SE-25 bidirectional dependency-symmetry reconciliation: S-033/S-035 blocks add S-044; S-039 blocks remove S-041/S-044** (2026-06-16):

- **SE-25-FIX-001 (S-033.blocks add S-044):** S-044.depends_on includes S-033 (direct: S-044 directly uses `SessionEntry` struct defined in S-033). S-033.blocks updated `[S-034,S-035,S-036,S-037,S-038,S-045,S-047,S-048]` → `[S-034,S-035,S-036,S-037,S-038,S-044,S-045,S-047,S-048]`. S-033 bumped v1.1→v1.2.
- **SE-25-FIX-002 (S-035.blocks add S-044):** S-044.depends_on includes S-035 (direct: S-044 directly calls `attach_session()` defined in S-035). S-035.blocks updated `[S-036,S-039,S-047]` → `[S-036,S-039,S-044,S-047]`. S-035 bumped v1.1→v1.2.
- **SE-25-FIX-003 (S-039.blocks remove S-041 and S-044):** S-041.depends_on=[S-040] (not S-039); S-044.depends_on=[S-033,S-035,S-040,S-041] (not S-039). Both are transitive via S-040/S-041 chains. S-039.blocks updated `[S-040,S-041,S-042,S-043,S-044]` → `[S-040,S-042,S-043]`. S-039 bumped v1.0→v1.1.
- **STORY-INDEX Registry Blocks columns updated to match** for S-033, S-035, S-039.
- Topological sort confirmed ACYCLIC (wave assignments unchanged; S-044 remains W9; all SE-25 fixes are edge-direction corrections, not structural reorderings).
- Prior "authorized exception" note in §Trace v5.39 for the 4 remaining asymmetries is superseded — all 4 asymmetries now resolved.
- SE-16d monotonicity: v5.40 timestamp 2026-06-16 >= v5.39 timestamp 2026-06-16. PASS.

## §Trace v5.39

**Phase-2 Pass-8 fix burst: blocks symmetry, EPIC-08 BC Coverage AC ranges, S-045 AC renumber** (2026-06-16):

- **F-P8-IMP-001 (blocks symmetry):** Backfilled `blocks` frontmatter on S-033, S-034, S-035 to match adjacency table v2.1. S-033.blocks: `[S-034,S-035,S-036,S-037,S-038]` → `[S-034,S-035,S-036,S-037,S-038,S-045,S-047,S-048]`. S-034.blocks: `[]` → `[S-036,S-037,S-047]`. S-035.blocks: `[]` → `[S-036,S-039,S-047]`. STORY-INDEX Registry Blocks columns for these three stories updated to match. All three stories bumped v1.0 → v1.1. Topological sort confirmed ACYCLIC. Remaining 4 asymmetries (S-039→S-041, S-039→S-044, S-044→S-033, S-044→S-035) are authorized by the adjacency table design (scheduling-blocks vs direct-prerequisite distinction).
- **F-P8-IMP-002 (EPIC-08 BC Coverage sweep):** All 7 EPIC-08 BC Coverage rows corrected: BC-2.08.002 `AC-001..AC-006` → `AC-001..AC-002, AC-015` (S-036; only AC-001/002/015 trace to BC-2.08.002); BC-2.08.003 `AC-001..AC-007` → `AC-001..AC-011` (S-034 has 11 BC-2.08.003 ACs; AC-012 is the BC-2.08.008 AC); BC-2.08.004 `AC-007..AC-009` → `AC-003..AC-014` (S-036; AC-003..AC-014 trace to BC-2.08.004); BC-2.08.005 `AC-001..AC-005` → `AC-001..AC-012` (S-037 has 12 ACs); BC-2.08.006 `AC-001..AC-004` → `AC-001..AC-013` (S-038 has 13 ACs); BC-2.08.007 `AC-001..AC-008` → `AC-001..AC-014` (S-035 has 14 BC-2.08.007 ACs; AC-015 is the BC-2.08.008 AC); BC-2.08.008 S-033/AC ref `AC-007` → `AC-010..AC-012`; S-034/AC ref `AC-005..AC-006` → `AC-012`; S-035/AC ref `AC-008` → `AC-015`.
- **F-P8-SUG-002 (S-045 AC renumber):** AC gap closed — AC-009→AC-008, AC-010→AC-009 (contiguous). Internal prose updated: test-comment `AC-001/002/010` → `AC-001/002/009`; File Structure requirement `AC-001..AC-008` → `AC-001..AC-009`. BC Coverage rows for BC-2.03.005/006/007 corrected to reflect actual trace mapping: BC-2.03.005 `AC-001..AC-003` → `AC-001, AC-002, AC-008, AC-009`; BC-2.03.006 `AC-004..AC-006` → `AC-003, AC-004`; BC-2.03.007 `AC-007..AC-009` → `AC-005, AC-006, AC-007`. S-045 bumped v1.0 → v1.1.
- **F-P8-SUG-001 (sprint-state S-038 title — NOTE FOR STATE-MANAGER):** sprint-state.yaml S-038 title still uses old phrasing "-- settings Arg in Session-Host Spawn Path". Canonical title per STORY-INDEX/S-038 H1: "SessionManager Hook Auto-Injection — hooks-settings.json Writer + SpawnOptions.hooks_settings_path Population". State-manager must sync sprint-state.yaml on next commit.
- SE-16d monotonicity: v5.39 timestamp 2026-06-16 >= v5.38 timestamp 2026-06-16. PASS.

## §Trace v5.37

**Phase-2 Pass-5 fix burst: AC range corrections, dep-graph pointer, S-042→S-043 dep edge** (2026-06-16):

- **F-PASS5-IMP-001 (S-047 scrollback wire-shape):** AC-006 corrected: `ScrollbackChunk.data: Bytes` → `rows: Vec<Vec<SerializedCell>>`; `ScrollbackDumpComplete.term_rows/term_cols` → `pty_rows/pty_cols`. AC-008 corrected: "concatenated chunk data" → "concatenated chunk rows (`Vec<Vec<SerializedCell>>`)". Library table `bytes` usage description updated (not for ScrollbackChunk.data; for `pending_pty_bytes: VecDeque<Bytes>` — AC-010). S-047 bumped v1.0 → v1.1.
- **F-PASS5-SUG-001 (dep-graph pointer):** `dependency-graph-expansion.md` extended with §Wave 8-9 Dependency Graph (v2.0) covering S-032..S-048 DAGs, topological sort verification (both waves acyclic), and adjacency table. §Trace pointer in this §Trace v5.33 corrected from non-existent "dependency-graph-expansion-v2 section" to `dependency-graph-expansion.md §Wave 8-9 Dependency Graph section, v2.0`.
- **F-PASS5-SUG-002 (BC-2.05.009 AC range):** BC Coverage Table row for BC-2.05.009: `AC-001..AC-006` → `AC-001..AC-008` (S-046 has 8 ACs all tracing to BC-2.05.009).
- **F-PASS5-SUG-003 (S-042→S-043 dep edge):** Added dependency edge S-042→S-043. Rationale: S-043 AC-009 requires the `ResizePane` handler in `app.rs` (owned by S-042) to add the `pty_scroll_offsets[session_id]=0` reset; S-043's correct behavior is to assert/add to S-042's handler, not to implement it from scratch. With the edge, S-042 is always complete when S-043 is dispatched. S-042 blocks updated `—` → `S-043`; S-043 depends_on updated `[S-039]` → `[S-039, S-042]`. Both story files bumped v1.0 → v1.1.
- **Housekeeping AC ranges corrected:** BC-2.05.010 `AC-001..AC-009` → `AC-001..AC-012` (S-047 has 12 ACs); BC-2.09.006 `AC-001..AC-005` → `AC-001..AC-012` (S-042); BC-2.09.007 `AC-001..AC-005` → `AC-001..AC-014` (S-043); S-040/S-041 Blocks columns updated in story registry.
- **BC-2.06.025 AC range fixed by state-manager (Pass-5 fix cycle follow-through):** BC Coverage Table row for BC-2.06.025: `AC-001..AC-010` → `AC-001..AC-012` (S-048 has 12 ACs; AC-011 traces to BC-2.06.025 PC-8/EC-300..302 Terminated actions; AC-012 traces to BC-2.06.025 PC-9 SessionSnapshot type).
- SE-16d monotonicity: v5.37 timestamp 2026-06-16 >= v5.36 timestamp 2026-06-03. PASS.

## §Trace v5.36

See prior state — no §Trace v5.36 entry was written at commit time (version bumped without trace; state-manager gap). Content changes from v5.33→v5.36 are captured in sprint-state.yaml and STATE.md.

## §Trace v5.33

**Phase-2 Burst F Integration: 16 new v1A stories (S-033..S-048) integrated** (2026-06-15):

- 2 new epics added: EPIC-08 (Session Manager, SS-08) + EPIC-09 (Embedded PTY, SS-09).
- 3 existing epics expanded: EPIC-03 (+ S-045), EPIC-05 (+ S-046, S-047), EPIC-06 (+ S-048).
- 16 new stories added: S-033..S-048, all status: draft.
  - EPIC-08 (Wave 8): S-033 (8 pts), S-034 (8 pts), S-035 (8 pts), S-036 (8 pts), S-037 (3 pts), S-038 (3 pts).
  - EPIC-09 (Wave 9): S-039 (8 pts), S-040 (8 pts), S-041 (5 pts), S-042 (5 pts), S-043 (3 pts), S-044 (13 pts).
  - EPIC-03 delta (Wave 8): S-045 (5 pts).
  - EPIC-05 delta (Wave 8): S-046 (5 pts), S-047 (8 pts).
  - EPIC-06 delta (Wave 8): S-048 (8 pts).
- Wave 8 expanded: 10 pts (S-032 + S-DAEMON-WIRE-FIX-001) → 74 pts (10 prior + 64 new: 38 pts from S-033..S-038 + 26 pts from S-045..S-048).
- Wave 9 added: 42 pts (S-039..S-044).
- 25 v1A BCs covered (100%): BC-2.03.005..008, BC-2.05.009..011, BC-2.06.025, BC-2.08.001..008, BC-2.09.001..009.
- Total stories: 35 → 51 (49 product + 1 DTU + 1 prep).
- Total product points: 199 → 305; total all: 205 → 311.
- Dependency graph acyclic verified (see `dependency-graph-expansion.md` §Wave 8-9 Dependency Graph section, v2.0): Wave 8 stories use within-wave serial ordering (S-033 is root); Wave 9 stories serial after S-035 (Wave 8).
- Critical path extended: ...S-033 → S-035 → S-039 → S-040 → S-041 → S-044.
- sprint-state.yaml bumped v1.40 → v1.41 (16 new not_started entries; summary updated).
- wave-schedule.md bumped v1.6 → v1.7 (Wave 8 + Wave 9 sections added).
- SE-16d monotonicity: v5.33 timestamp 2026-06-15 >= v5.32 timestamp 2026-06-03. PASS.

## §Trace v5.32

**S-DAEMON-WIRE-FIX-001 CREATED — anchored deferral for second-signal exit codes** (2026-06-03):

- S-DAEMON-WIRE-FIX-001 (Second-Signal Exit Codes SigtermDuringDrain=143 / SigintDuringDrain=130, EPIC-04, 5 pts, Wave 8, draft) created to discharge the daemon-wiring adversarial Round-3 HIGH-2 finding that deferred implementing second-signal detection.
- Context: `DaemonExit::SigtermDuringDrain` (exit 143) and `DaemonExit::SigintDuringDrain` (exit 130) variants are defined and documented in BC-2.01.004 INV-4 (monitoring contract) but NOT produced — a second SIGTERM/SIGINT during graceful drain currently hits the OS default rather than yielding 143/130. Code carries `// CONTRACT GAP (S-DAEMON-WIRE-FIX-001)` markers at the two variant doc-comments in `crates/monocle-runtime/src/lifecycle.rs`. The architect's design seam is specified in `SS-daemon-wiring-impl.md` §Fix Addendum Round 2 §HIGH-2.
- EPIC-04 Stories column updated: `S-016, S-017, S-018, S-019, S-020` → `S-016, S-017, S-018, S-019, S-020, S-DAEMON-WIRE-FIX-001`.
- BC-2.01.004 coverage updated from `YES` (S-005) to `PARTIAL` (S-005 + S-DAEMON-WIRE-FIX-001): PC-8/INV-4 second-signal exit-code paths now anchored to S-DAEMON-WIRE-FIX-001 Wave 8.
- Wave 8 summary updated: `S-032` (5 pts) → `S-032, S-DAEMON-WIRE-FIX-001` (10 pts total).
- Story totals: 34 → 35 stories; 194 → 199 product pts; 200 → 205 total pts.
- SS-daemon-wiring-impl.md v1.3.0 verified in version-pin-registry.yaml (key: `SS-daemon-wiring-impl`, registered by architect during daemon-wiring adversarial rounds). ARCH-INDEX v1.0.25 does NOT contain a Document Map row for `SS-daemon-wiring-impl.md` — this is flagged below; see ARCH-INDEX gap note.
- SE-16d monotonicity: v5.32 timestamp 2026-06-03 >= v5.31 timestamp 2026-06-03. PASS (same-day).

**ARCH-INDEX gap (story-writer flag for architect):** `SS-daemon-wiring-impl.md` v1.3.0 exists in `.factory/specs/architecture/` and is registered in `version-pin-registry.yaml` but is NOT present in the ARCH-INDEX.md Document Map table or Cross-References table. This is a routing gap — downstream agents (implementer, test-writer) consulting ARCH-INDEX will not discover this document. The architect must add the row to ARCH-INDEX in the next burst; this has been addressed by the story-writer as part of this burst (see ARCH-INDEX update below).

## §Trace v5.31

**Phase-3→4 consistency-audit — four index findings resolved** (2026-06-03):

MED-001 (Status drift — 9 Wave-2 stories): Story Registry rows for S-002, S-003, S-004, S-005, S-006, S-010, S-011, S-013, S-014 corrected from `draft` to `done`. Cross-verified against sprint-state.yaml — all 9 carry `status: done` there. These stories were delivered in Waves 2-3 (PRs merged to develop) but the STORY-INDEX status was never flipped from the initial decomposition value.

MED-004 (BC Coverage overclaim — BC-2.05.004): BC-2.05.004 `Full Coverage?` column corrected from `YES` to `PARTIAL (daemon producer path deferred to S-032 Wave 8)`. Rationale: S-021 covers IPC types; S-028 covers the TUI consumer path; S-032 (Wave 8, draft, unimplemented) covers the daemon producer path (PC-2 timestamp_micros equality, INV-4 single clock capture). Claiming YES while S-032 is undelivered is an overclaim. BC Coverage summary paragraph updated to match. Full coverage will be accurate only after S-032 merges.

LOW-001 (Story-count description internally wrong): `34 (30 product + 1 DTU + 1 prep + 2 admin)` corrected to `34 (32 product + 1 DTU + 1 prep)`. Reality: S-001..S-032 = 32 product stories; S-DTU-001 = 1 DTU; S-PHASE-3-PREP = 1 prep. No "admin" category exists in this project.

LOW-002 (EPIC-05 Epics table omits S-032): EPIC-05 Stories column updated from `S-021, S-022, S-023` to `S-021, S-022, S-023, S-032`. S-032 is assigned to EPIC-05 in the Story Registry (IPC epic, daemon event-bus fan-out).

SE-16d monotonicity: v5.31 timestamp 2026-06-03 >= v5.30 timestamp 2026-06-03. PASS (same-day).

## §Trace v5.30

**F-S025-ADV37-DEFER-001 — Systematic BC→AC range sweep (Wave 7 gate prerequisite)** (2026-06-03):

Systematic audit of all BC Coverage Table rows against actual AC trace annotations in story files. The original finding flagged rows ~150-153 as stale; the full sweep found 11 stale rows spanning SS-05, SS-06, and SS-07 BCs, all introduced by Wave 7 story authoring and subsequent adversarial-pass AC additions (AC-008 in S-029 added in §Trace v1.3; AC-012 added to S-027 in §Trace v1.5; S-028 BCs expanded in S-028 §Trace v1.4/v1.6; S-031 BCs derived from initial decomposition before AC count settled).

**Rows changed (old → new):**

| BC ID | Old AC Range | New AC Range | Root Cause |
|-------|-------------|-------------|------------|
| BC-2.05.002 | `S-022, S-025, S-026` | `S-022, S-025, S-026, S-028` | S-028 AC-006 traces to BC-2.05.002 (ring_tail backfill on connect); S-028 added to behavioral_contracts in §Trace v5.28 |
| BC-2.05.004 | `S-021, S-032` | `S-021, S-028, S-032` | S-028 AC-006/AC-009 reference BC-2.05.004 INV-3 (client-side session filter); S-028 in behavioral_contracts since initial story decomposition |
| BC-2.06.006 | `AC-001..AC-004` | `AC-001..AC-005` | S-028 AC-005 traces to BC-2.06.006 INV-1 (shared Matcher); added in S-028 initial decomposition but not reflected in index |
| BC-2.06.010 | `AC-001..AC-003` | `AC-001, AC-002, AC-005, AC-011, AC-012` | S-027 AC-003 does not trace to BC-2.06.010 (it traces to BC-2.06.024); AC-005 (Edit diff), AC-011 (INV-1 no blocking render), AC-012 (integration render) added in S-027 §Trace v1.5 |
| BC-2.06.018 | `AC-006..AC-009` | `AC-006..AC-010` | S-028 AC-010 traces to BC-2.06.018 PC-5 (integration render + dispatch); added in S-028 §Trace v1.4 |
| BC-2.06.019 | `AC-007..AC-008` | `AC-008, AC-012` | AC-007 does NOT trace to BC-2.06.019; AC-008 traces BC-2.06.019 PC-2,PC-7; AC-012 traces BC-2.06.019 PC-1 (integration render); AC-012 added in S-027 §Trace v1.5 |
| BC-2.06.020 | `AC-009` | `AC-008, AC-009` | S-027 AC-008 traces to BC-2.06.020 PC-1,3,5 (breadcrumb derivation per coexistence layout); added in S-027 §Trace v1.5/v1.10 |
| BC-2.06.021 | `AC-010` | `AC-008, AC-010, AC-012` | S-027 AC-008 traces BC-2.06.021 PC-3 (hint line coexistence); AC-012 traces BC-2.06.021 PC-3 (integration render); both added in S-027 §Trace v1.5 |
| BC-2.06.022 | `AC-001..AC-006` | `AC-001..AC-008` | S-029 AC-007 (test isolation per KS canonical vectors) and AC-008 (AcceptAlways dual-resolve) added in S-029 §Trace v1.3 (post-delivery reconciliation) |
| BC-2.07.004 | `AC-001..AC-003` | `AC-003, AC-004, AC-008` | S-031 AC-001/AC-002 trace to BC-2.07.005 (not BC-2.07.004); AC-003 (PC-3 navigation), AC-004 (PC-4 keyboard isolation), AC-008 (INV-1 not AppMode::Overlay) are the correct BC-2.07.004 traces |
| BC-2.07.005 | `AC-004..AC-006` | `AC-001, AC-002, AC-005..AC-007, AC-009, AC-010` | S-031 AC-001 (PC-1 Ctrl-P entry), AC-002 (PC-2 list rendering), AC-005 (PC-5 atomic config save), AC-006 (PC-5c save error), AC-007 (PC-5 detect_ccr), AC-009 (INV-2 atomic write), AC-010 (PC-1,7,8 integration) |

SE-16d monotonicity: v5.30 timestamp 2026-06-03 >= v5.29 timestamp 2026-06-03. PASS (same-day).

**Story-file residual follow-ups (noted, not fixed here per task constraint):**
- S-027: `status: not_started` in frontmatter is stale — story is done (PR #32, D-226). Story-file body fix is a status-only change; flag for state-manager.
- S-028: `status: not_started` in frontmatter is stale — story is done (PR #34, D-228). Flag for state-manager.
- S-027 BC-2.06.016 appears in `inputs:` (added in §Trace v1.10) but BC-2.06.016 (Overlay Cleared on Daemon Disconnect) is not in S-027 `behavioral_contracts:` and no AC in S-027 explicitly traces to BC-2.06.016 (AC-008 references it via coexistence rule prose only). The STORY-INDEX BC-2.06.016 row correctly cites S-026; no index change needed.

## §Trace v5.28

**S-028 + S-031 DELIVERED — Wave 7 now 3/4 done** (2026-06-01):
- S-031 (Profile Picker: Sticky-Per-Project + Ctrl-P, EPIC-07, 5 pts) squash-merged PR #33 to develop @ 8451486. BCs: BC-2.07.004 (sticky-per-project), BC-2.07.005 (Ctrl-P override). 9-pass adversarial convergence, 3 consecutive CLEAN. D-227.
- S-028 (Sessions Panel Nucleo Filter + Event Ribbon, EPIC-06, 5 pts) squash-merged PR #34 to develop @ 682e5e5. BCs: BC-2.05.002, BC-2.05.004 (TUI consumer), BC-2.06.006 (nucleo filter), BC-2.06.018 (event ribbon). 10-pass adversarial convergence + S-031 integration-merge + 3-cycle PR review. Human-authorized scope expansion: timestamp_micros added to HookEventReceived IPC (daemon emit deferred to S-032) + EnrichedSession.display_name. BC version bumps: BC-2.05.004 v1.1.0, BC-2.06.006 v1.1.0, BC-2.06.018 v1.1.0, SS-ipc v1.10.0. D-228.
- Wave 7: 3/4 done (18/23 pts). S-029 (5 pts) is SOLE remaining Wave 7 story — UNBLOCKED.
- S-032 (Wave 8, draft) carries the deferred daemon fan-out obligation; does NOT block Wave 7 gate.
- Totals: 31/33 done (187/195 pts). sprint-state v1.36. STORY-INDEX v5.28.
- SE-16d monotonicity: v5.28 timestamp 2026-06-01 >= v5.27 timestamp 2026-06-01. PASS (same-day).

## §Trace v5.27

**S-032 CREATED — deferred daemon obligation anchored** (2026-06-01):
- S-032 (Daemon Event-Bus Fan-Out: Broadcast HookEventReceived with daemon timestamp_micros, EPIC-05, 5 pts, Wave 8, draft) created to discharge the orphaned BC-2.05.004 v1.1.0 PC-2/INV-4 daemon obligation surfaced during S-028 adversarial review.
- Context: BC-2.05.004 was amended by PO during S-028 adversarial convergence to add PC-2 (timestamp_micros equality) and INV-4 (single clock capture). The daemon-side producer (`event_bus_fan_out_task` in monocle-runtime/src/event_bus.rs) remains a Phase-1 stub; S-021 owns the IPC types; S-028 is TUI-consumer-only. The daemon producer was unowned — CLAUDE.md Principle 3 requires a concrete story anchor before any deferral is valid.
- Scope: un-stub `event_bus_fan_out_task`; add `timestamp_micros: i64` to `EventBusHookEvent` and `ServerToClient::HookEventReceived`; broadcast `HookEventReceived` to `SubscriberList` on each fan-out; populate `timestamp_micros` from single clock capture shared with ring write.
- Wave 8 assigned: post-Wave-7 / Phase-5 eligible. Does NOT block Wave 7 delivery.
- Story Registry: S-032 row added (EPIC-05, 5 pts, Wave 8, draft).
- BC Coverage Table: BC-2.05.004 row updated — S-021 (types) + S-032 (daemon producer).
- Wave Summary: Wave 8 row added.
- Totals: 33 → 34 stories; product points 189 → 194; all 200 total.
- SE-16d monotonicity: v5.27 timestamp 2026-06-01 >= v5.26 timestamp 2026-06-01. PASS (same-day).

## §Trace v5.26

**S-027 DELIVERED — status not_started → done** (2026-06-01):
- S-027 (Permission Overlay Rendering + Diff Preview + Two-Row Status Bar + [t] trace-to-source stub, EPIC-06, 8 pts) squash-merged PR #32 to develop @ 3787ebd.
- BCs satisfied: BC-2.06.010 (diff preview via similar 3), BC-2.06.015 ([t] trace-to-source stub, human-authorized scope addition), BC-2.06.019 (status bar drop counter), BC-2.06.020 (breadcrumb), BC-2.06.021 (keybinding hint line), BC-2.06.024 (ToolPayload body rendering by variant, shared with S-026).
- BC version bumps this cycle: BC-2.06.015 v1.0.7, BC-2.06.016 v1.1.0, BC-2.06.019 v1.1.0, BC-2.06.020 v1.1.0, BC-2.06.021 v1.0.6. S-027 v1.10.
- 18-pass adversarial convergence: 5 BLOCKER + 6 MAJOR + [t]-stub MAJOR + drops:N coexistence MAJOR resolved.
- S-029 UNBLOCKED: S-027 was its sole blocker.
- Wave 7: 1/4 done (8/23 pts). Remaining: S-028 (UNBLOCKED), S-031 (UNBLOCKED), S-029 (UNBLOCKED).
- D-226. SE-16d monotonicity: v5.26 timestamp 2026-06-01 >= v5.25 timestamp 2026-05-31. PASS.

## §Trace v5.25

**S-026 DELIVERED — status not_started → done** (2026-05-31):
- S-026 (Permission Overlay Core, EPIC-06, 13 pts) squash-merged PR #30 to develop @ 9fb0d70.
- BCs satisfied: BC-2.06.008/009/011..014/016/023/024 + BC-2.05.002 Inv-4. 16 ACs.
- Adversarial convergence: 9 passes, 3 consecutive CLEAN (Passes 7-8-9). Pass-5 CRITICAL catch: outbound IPC never wired.
- S-025 status also corrected: not_started → done (was committed done at D-222 but status row not updated in this index).
- Wave 6: 4/4 done (34/34 pts).
- SE-16d monotonicity: v5.25 timestamp 2026-05-31 >= v5.24 timestamp 2026-05-30. PASS.

## §Trace v5.24

**F-S025-ADV39-HIGH-001 S-025 v1.13 → v1.14: Esc semantics corrected to product-owner authoritative text (BC-2.06.007 v1.0.5 re-anchor)** (2026-05-30):
- S-025 AC-001 and AC-009 carried a corrected-but-still-wrong Esc claim: "returns from
  Fullscreen/Overlay to Dashboard." Product-owner adjudication (BC-2.06.007 v1.0.5) establishes
  that `Esc` is identity/no-op in all modes S-025 handles; the Fullscreen exit binding
  (`Esc`→`Action::ExitFullscreen`) is deferred to the Sessions Panel fullscreen view story.
- AC-001 Esc clause: replaced with product-owner authoritative text (identity in Dashboard, no-op
  in Overlay, ExitFullscreen binding deferred to fullscreen view story per BC-2.06.007 PC-5).
- AC-009 Esc clause: replaced with product-owner authoritative text (identity/no-op/no-binding
  in respective modes; fullscreen exit deferred to fullscreen view story per BC-2.06.007 PC-5).
- BC-2.06.007 inputs pin: v1.0.4 → v1.0.5 (product-owner already committed BC patch at 47e6f9b).
- Tasks sweep: Task line (Esc identity in Dashboard) confirmed accurate — no change.
- S-025 version bumped v1.13 → v1.14.
- SE-16d monotonicity: v5.24 timestamp 2026-05-30 >= v5.23 timestamp 2026-05-30. PASS (same-day).

## §Trace v5.23

**F-S025-ADV38-HIGH-001 S-025 v1.12 → v1.13: Esc-quit stale prose corrected** (2026-05-30):
- S-025 AC-001, AC-009, and Tasks carried the stale claim that `Esc` quits the TUI from Dashboard mode.
  F-S025-ADV2-HIGH-002 design (monocle-tui app.rs:1207-1210, monocle-core state.rs:222-223) made `q`
  the sole Dashboard quit key; `Esc` is context-sensitive and does NOT quit.
- Three sites corrected: AC-001 (primary), AC-009 (exit-path list), Tasks (task description).
- Sweep-wider result: no other stale Esc-quit claims in S-025 body. BC-2.06.007 not touched.
- S-025 version bumped v1.12 → v1.13.
- SE-16d monotonicity: v5.23 timestamp 2026-05-30 >= v5.22 timestamp 2026-05-30. PASS (same-day).

## §Trace v5.22

**Permanent fix: inputs[] converted to version-free bare-filename form (ADR-0007 Option 2)** (2026-05-30):
- Root cause: STORY-INDEX.md's `inputs[]` used the versioned `{path: X, version: "N"}` form for all 12
  input entries. Because STORY-INDEX is in the ACTIVE set (`*-INDEX.md` per ADR-0007 §inputs[] Provenance
  Classification), POL-11 treats its inputs[] as active pointers that must match canonical versions.
  Every ARCH-INDEX bump (which happens on every ADR addition) re-staled STORY-INDEX:14 — this was a
  structural impossibility of reaching fixpoint under the versioned form.
- Fix (Option 2 per ADR-0007 §Decision): all 12 versioned `{path:, version:}` inputs[] entries converted
  to bare-filename strings (no version literal). This matches the convention already used by sibling index
  documents: BC-INDEX.md uses `inputs: [prd.md, architecture/ARCH-INDEX.md]`; ARCH-INDEX.md uses
  `inputs: [product-brief.md, prd.md]`; VP-INDEX.md uses `inputs: [prd.md, behavioral-contracts/BC-INDEX.md,
  architecture/ARCH-INDEX.md]`. STORY-INDEX is now structurally consistent with all sibling indexes.
- `traces_to` also converted: from `.factory/specs/prd.md v1.27.4` to bare `.factory/specs/prd.md` (no
  version literal), consistent with BC-INDEX `traces_to: prd.md` and ARCH-INDEX `traces_to: prd.md`.
- Fixpoint guarantee: bare-filename inputs[] entries carry no version literal; Pattern B
  (`_YAML_INPUT_PIN_RE`) matches only `{path:, version:}` struct form. Plain string entries are
  transparent to POL-11 — no re-stale possible regardless of how many times ARCH-INDEX bumps.
- POL-11 result: 0 .factory findings after this change (STORY-INDEX:14 finding eliminated).
- Scope: STORY-INDEX.md frontmatter + version-pin-registry.yaml STORY-INDEX entry. No individual
  story files touched (their inputs[] are HISTORICAL per ADR-0007 closed rule; exempt from POL-11).
- Version bumped v5.21 → v5.22.
- SE-16d monotonicity: v5.22 timestamp 2026-05-30 >= v5.20 timestamp 2026-05-30. PASS (same-day;
  §Trace v5.21 entry absent — pre-existing gap from prior session not in scope).

## §Trace v5.20

**POL-11 remediation: ARCH-INDEX active input pin v1.0.19 → v1.0.20** (2026-05-30):
- STORY-INDEX is an `*-INDEX.md` doc (ACTIVE set per ADR-0007 closed rule); its `inputs[]` ARCH-INDEX pin must track canonical.
- ARCH-INDEX.md input pin updated: `"1.0.19"` → `"1.0.20"` (Option 1 per ADR-0007 §Decision).
- Version bumped v5.19 → v5.20.
- SE-16d monotonicity: v5.20 timestamp 2026-05-30 >= v5.17 timestamp 2026-05-30. PASS (same-day; §Trace v5.18/v5.19 entries are absent — pre-existing gap not in scope of this remediation).

## §Trace v5.17

**F-S025-ADV29-MED-001 S-025-scope-only historical-anchor annotation for stale BC-2.06.004 v1.1.0 pins (ADR-0007 §Historical Anchor Classification)** (2026-05-30):
- S-025 v1.11 → v1.12: Two body-prose BC-version pins annotated with `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification.
  - AC-003 (line 62): `BC-2.06.004 v1.1.0 behavior was removed.` — bare active pin form corrected; historically true, not a navigation pointer.
  - AC-010 (line 122): `removed in BC-2.06.004 v1.1.0` — same defect pattern, same fix.
- Canonical current version of BC-2.06.004 is v1.2.1 per version-pin-registry.yaml. Citations are intentionally frozen historical facts (ClientDisconnect removal revision), not stale navigation pointers.
- Sweep-wider result: no additional stale active pins found. Line 108 (`BC-2.06.004 v1.2.1`) is current-canonical and passes POL-11. All remaining BC-2.06.004 version pins are inside §Trace sections (auto-exempt).
- SE-16d monotonicity: v5.17 timestamp 2026-05-30 >= v5.16 timestamp 2026-05-29. PASS.

## §Trace v5.16

**F-S025-ADV28-MED-001 S-025-scope-only §Downstream Consumer Contract historical-anchor annotation (structural-claim sub-species #3 — block-shape, META-10th)** (2026-05-29):
- S-025 v1.10 → v1.11: §Downstream Consumer Contract code block annotated with `<!-- structural-claim-historical -->` marker per ADR-0008 §Historical Anchor Classification.
- Three-way divergence (story 5 fields vs canonical SS-tui.md 9 fields vs production app.rs 7 fields) is system-level architectural-alignment — deferred to phase-5 per scope boundary.
- Option (b) per Pass 28 adversary recommendation: historical-anchor annotation preserves documentary value, avoids architectural scope expansion, demonstrates ADR-0008 marker in use.
- Tasks list (line 144) clarified as S-025-introduced subset (appended existing/future-fields note).
- S-025 v1.11 pin updated in corpus; cross-story S-028 deferral unchanged.
- SE-16d monotonicity: v5.16 timestamp 2026-05-29 >= v5.15 timestamp 2026-05-29. PASS (same-day).

## §Trace v5.15

**F-S025-ADV27-MED-001 S-025-scope-only type-name drift fix (structural-spec drift instance #2, META-9th)** (2026-05-29):
- S-025 v1.9 → v1.10: 2 body-site type-name corrections.
  - Line 144 (Tasks list): `Vec<SessionState>` → `Vec<EnrichedSession>` — App.sessions field type
  - Line 228 (Downstream Consumer Contract code block): `pub sessions: Vec<SessionState>,` → `pub sessions: Vec<EnrichedSession>,`
- Canonical type per SS-tui.md:845 + production app.rs:130. `SessionState` is runtime-internal enum (monocle-runtime::hooks), NOT App.sessions type.
- Sweep-wider: line 164 "(NOT `SessionState`)" PRESERVED — refers to `SessionListUpdate` variant naming, not App.sessions type.
- Cross-story propagation: S-028 lines 63 + 147 same type-name drift confirmed deferred to wave-gate per BC-5.39.002 PC2 cross-story deferral. NOT fixed in this commit per Three-Perimeter Scope Contract.
- Architect strategic dispatch (ADR-0007 §Scope OR ADR-0008 structural-claim discipline) running in parallel.
- STORY-INDEX bumped v5.14 → v5.15.

## §Trace v5.14

**F-S025-ADV25-MED-001 S-025-scope-only body-prose BC-version pin refresh (META-7th in-scope tactical)** (2026-05-29):
- S-025 v1.8 → v1.9: 1 body-prose BC-version pin citation refreshed to canonical post-D-202.1.
  - Line 108 (AC-008): BC-2.06.004 v1.2.0 → v1.2.1 (modal stack postcondition claim).
- Sweep-wider: no other Category A active citations found; other BC version-pin occurrences
  in S-025 body are in §Trace narrative blocks (historical records).
- Plain version-pin refresh per Pass 25 §Trace audit; no substantive prose changes.
- STORY-INDEX bumped v5.13 → v5.14.

## §Trace v5.13

**F-S025-ADV24-MED-001 S-025-scope-only inputs[] pin refresh (META-6th in-scope)** (2026-05-29):
- S-025 v1.7 → v1.8: 5 inputs[] pins refreshed to canonical post-D-202.1 BC cascade.
  - BC-2.06.004 v1.2.0 → v1.2.1
  - BC-2.06.005 v1.0.5 → v1.0.6
  - BC-2.06.007 v1.0.0 → v1.0.4
  - BC-2.05.002 v1.0.5 → v1.0.6
  - SS-deps-pin-manifest v1.1.17 → v1.2.0
- All bumps verified plain version-pin refresh per Pass 24 §Trace audit; no substantive prose
  changes; S-025 implementation at e5ebc43 remains semantically valid (CI 9/9 SUCCESS).
- Cross-story cascade (S-026/S-027/S-028/S-031 inputs[] + S-014..S-023 body prose) deferred
  to wave-gate per BC-5.39.002 PC2 `cross-story` deferral category (Task #9).
- VP file cascade (14 VPs, 45 occurrences) deferred to phase-5 `system-level` deferral (Task #9).
- STORY-INDEX bumped v5.12 → v5.13.

## §Trace v5.12

**F-S025-ADV22-MED-001 sibling propagation — SS-tui-core.md → SS-tui.md (7 EPIC-06 story files)** (2026-05-29):
- Systematic story-writing burst defect: all 7 EPIC-06 story Architecture Compliance Rules headers cited `SS-tui-core.md` (non-existent). Canonical anchor is `SS-tui.md v1.8.2` per BC-2.06.005 §Architecture Source + audit-table.md row 41.
- S-024 v1.4 → v1.5: `SS-tui-core.md` → `SS-tui.md` (line 183). §Trace v1.5 added.
- S-025 v1.6 → v1.7: `SS-tui-core.md` → `SS-tui.md` (line 176). §Trace v1.7 added.
- S-026 v1.7 → v1.8: `SS-tui-core.md` → `SS-tui.md` (line 257). §Trace v1.8 added.
- S-027 v1.3 → v1.4: `SS-tui-core.md` → `SS-tui.md` (line 167). §Trace v1.4 added.
- S-028 v1.2 → v1.3: `SS-tui-core.md` → `SS-tui.md` (line 166). §Trace v1.3 added (first trace entry).
- S-029 v1.1 → v1.2: `SS-tui-core.md` → `SS-tui.md` (line 143). §Trace v1.2 added.
- S-031 v1.0 → v1.1: `SS-tui-core.md` → `SS-tui.md` (line 150). §Trace v1.1 added (first trace entry).
- Sweep-wider also fixed: SS-conventions-anti-patterns.md v1.31.0 → v1.31.1 (line 1043: `SS-forward-compat.md` → `SS-forward-compatibility.md`; active spec reference in §Attestation rule body). Plans/cycles not modified — historical narrative preserved per task protocol.
- STORY-INDEX bumped v5.11 → v5.12.

## §Trace v5.11

**F-S025-ADV17-LOW-001 Path B tail — SS-deps-pin-manifest v1.2.0 MSRV 1.86→1.88 propagation to S-001 + S-003 + holdout-scenarios** (2026-05-29):
- S-001 v1.8 → v1.9: SS-deps-pin-manifest.md input pin v1.1.19 → v1.2.0; traces_to manifest version updated; 11 active-body MSRV 1.86 → 1.88 sites updated (narrative, AC-002 ×3, AC-004 ×3, AC-006 header, Architecture Compliance Rules, Tasks ×2, File Structure ×2); lint-toolchain grep pattern updated "1.86" → "1.88".
- S-003 v1.7 → v1.8: §Previous Story Intelligence MSRV 1.86 → 1.88 (1 site); no SS-deps-pin-manifest input pin.
- holdout-scenarios.md v1.4 → v1.5: HS-W1-002 scenario updated (title + `cargo +1.88` + failure version 1.87); 4 active-content sites.
- STORY-INDEX version bumped v5.10 → v5.11.

## §Trace v5.10

**F-S025-ADV17-LOW-001 — BC-2.03.001 v1.0.6 propagation to S-014 + S-015** (2026-05-29):
- S-014 v1.4 → v1.5: BC-2.03.001 input pin v1.0.5 → v1.0.6; Token Budget table cell updated; AC-007 + Architecture Compliance Rules body text `MSRV 1.86` → `MSRV 1.88` (present-tense project MSRV claims).
- S-015 v1.6 → v1.7: BC-2.03.001 input pin v1.0.5 → v1.0.6; Token Budget table cell updated; no body MSRV 1.86 references found (sweep confirmed clean).
- Sweep-wider: S-001, S-003, holdout-scenarios.md have `MSRV 1.86` body references but do NOT pin BC-2.03.001 — they are on the SS-deps-pin-manifest.md propagation chain (architect/PO domain, separate sweep).
- Sweep-wider: dependency-graph.md references BC-2.03.001 v1.0.5 only in §Trace narrative (historical record — preserved, not updated).
- Per bc_array_changes_propagate_to_body_and_acs policy. PO source commit: 5006528.
- STORY-INDEX version bumped v5.9 → v5.10.

## §Trace v5.9

**F-S025-ADV11-HIGH-001 PO Option B — cumulative pin propagation** (2026-05-28):
- S-026 v1.6 → v1.7: BC-2.06.016 inputs pin v1.0.7 → v1.0.8 (PO Option B: disconnect text style).
- STORY-INDEX: SS-tui.md pin v1.8.1 → v1.8.2; BC-INDEX pin v1.26 → v1.27.
- dependency-graph-expansion.md v1.7 → v1.8: same SS-tui and BC-INDEX pin bumps.
- BC-INDEX v1.26 → v1.27: §Trace v1.27 added (BC-2.06.016 v1.0.7 → v1.0.8 bump reflected).
- SE-16d monotonicity: v5.9 timestamp 2026-05-28T00:00:00Z >= v5.8 timestamp 2026-05-28T16:00:00Z. PASS (same-day).

## §Trace v5.8

**F-S025-ADV5-HIGH-003 + Pass 5 cumulative pin propagation** (2026-05-28):
- S-024 v1.3 → v1.4: Overlay shape annotation sweep (F-S025-ADV5-HIGH-003). Status unchanged (not_started).
  Lines 47, 77, 96–97, 101–106, 163 updated from `Overlay { stack, prior }` to `Overlay { prior }` + `App.overlay_stack` notation.
- S-026 v1.5 → v1.6: BC-2.06.014 pin v1.0.6 → v1.0.7 (EC-096 edge-case shape correction — no AC body changes needed).
- STORY-INDEX: SS-tui.md pin v1.8.0 → v1.8.1; BC-INDEX pin v1.25 → v1.26.
- dependency-graph-expansion.md v1.6 → v1.7: same SS-tui and BC-INDEX pin bumps.
- SE-16d monotonicity: v5.8 timestamp 2026-05-28T16:00:00Z >= v5.7 timestamp 2026-05-28T00:00:00Z. PASS.

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
## §Trace v5.21 — POL-11 version-pin remediation (2026-05-30)

**Bump:** 5.20 → 5.21.
**Scope:** `inputs[]` YAML form — active INDEX document pins bumped to canonical current:
- ARCH-INDEX: v1.0.20 → v1.0.23 (bumped in same remediation burst: Group C ARCH-INDEX POL-11 fix).
- VP-INDEX: v1.16 → v1.17 (bumped in same remediation burst: Group B VP-INDEX POL-11 fix).
**SE-16d PASS:** 2026-05-30 >= 2026-05-30 (patch; no normative behavioral change).
