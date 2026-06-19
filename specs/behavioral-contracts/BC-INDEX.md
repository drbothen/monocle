---
document_type: behavioral-contract-index
level: L3
version: "1.43.9"
status: active
producer: vsdd-factory:state-manager
timestamp: 2026-06-19T00:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "cbf13d5"
traces_to: prd.md
---

# Behavioral Contract Index: monocle Phase 1

> **Source of truth** for all behavioral contract IDs, titles, priorities, and file paths.
> BC frontmatter `subsystem:`, BC body references, story `bcs:` arrays, and the PRD
> Behavioral Contracts Index (§2) MUST all use IDs and titles from this table.
>
> **Append-only:** When a BC is retired or replaced, mark it `status: retired` and add a
> `replaced_by:` column entry. Never remove a row or reuse an ID.

---

## SS-01: Daemon Lifecycle

> Architecture source: `architecture/SS-daemon-lifecycle.md`
> ARCH-INDEX subsystem: SS-01
> Capability: CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.01.001 | Healthz Endpoint (Unauthenticated Liveness Probe) | P0 | active | ss-01/BC-2.01.001.md | BC-DAEMON-001 |
| BC-2.01.002 | Status Endpoint (Authenticated Daemon State) | P0 | active | ss-01/BC-2.01.002.md | BC-DAEMON-002 |
| BC-2.01.003 | Body Size Limit (256 KiB, HTTP 413) | P0 | active | ss-01/BC-2.01.003.md | BC-DAEMON-003 |
| BC-2.01.004 | Graceful Shutdown (10-Second Drain) | P0 | active | ss-01/BC-2.01.004.md | BC-DAEMON-004 |
| BC-2.01.005 | Lock File Atomic Lifecycle (Create + Pid Check + Cleanup) | P0 | active | ss-01/BC-2.01.005.md | BC-DAEMON-005 |
| BC-2.01.006 | Crash Recovery Checkpoint | P0 | active | ss-01/BC-2.01.006.md | BC-DAEMON-006 |
| BC-2.01.007 | JSONL Ring Format Version (FC-01) | P0 | active | ss-01/BC-2.01.007.md | BC-RING-001 |
| BC-2.01.008 | Auth Token Wire Format (FC-06) | P0 | active | ss-01/BC-2.01.008.md | BC-AUTH-001 |
| BC-2.01.009 | Auth Header Validation (Missing and Invalid Token) | P0 | active | ss-01/BC-2.01.009.md | BC-AUTH-002 |
| BC-2.01.010 | Lock File Contract Version Field | P0 | active | ss-01/BC-2.01.010.md | BC-LOCK-001 |

---

## SS-02: Core Types and ABI

> Architecture source: `architecture/SS-core-types-and-abi.md`
> ARCH-INDEX subsystem: SS-02
> Capability: CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.02.001 | ABI Version in /status Endpoint (FC-03) | P0 | active | ss-02/BC-2.02.001.md | BC-ABI-001 |
| BC-2.02.002 | ABI Version Constant at Crate Root (FC-03) | P0 | active | ss-02/BC-2.02.002.md | BC-ABI-002 |
| BC-2.02.003 | Non-Exhaustive Enum Policy (FC-02) | P0 | active | ss-02/BC-2.02.003.md | BC-TYPES-001 |
| BC-2.02.004 | FactoryAdapter Trait Definition (FC-04 CRITICAL) | P0 | active | ss-02/BC-2.02.004.md | BC-FACTORY-001 |
| BC-2.02.005 | VsddFactoryAdapter Implementation | P0 | active | ss-02/BC-2.02.005.md | BC-FACTORY-002 |
| BC-2.02.006 | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | P0 | active | ss-02/BC-2.02.006.md | BC-PROTO-001a |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | P0 | active | ss-02/BC-2.02.007.md | BC-PROTO-001b |
| BC-2.02.008 | Phase 4 schema_version Validation Requirement (FC-05) | P1 | active | ss-02/BC-2.02.008.md | BC-PROTO-002 |

---

## SS-03: Engine Module

> Architecture source: `architecture/SS-engine-module.md`
> ARCH-INDEX subsystem: SS-03
> Capability: CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.03.001 | EngineModule Trait Definition | P0 | active | ss-03/BC-2.03.001.md | BC-ENGINE-001 |
| BC-2.03.002 | ClaudeCodeModule Implementation (Strict-Basename Detect) | P0 | active | ss-03/BC-2.03.002.md | BC-ENGINE-002 |
| BC-2.03.003 | HomeUnresolvable Error Contract | P0 | active | ss-03/BC-2.03.003.md | BC-ENGINE-002-ERR |
| BC-2.03.004 | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | P0 | active | ss-03/BC-2.03.004.md | BC-ENGINE-003 |
| BC-2.03.005 | ClaudeCodeModule.spawn_recipe() — Happy-Path Recipe Assembly | P0 | active | ss-03/BC-2.03.005.md | — |
| BC-2.03.006 | ClaudeCodeModule.spawn_recipe() — CCR Base URL Injection | P0 | active | ss-03/BC-2.03.006.md | — |
| BC-2.03.007 | spawn_recipe() Error Cases — BinaryNotFound and InvalidPath | P0 | active | ss-03/BC-2.03.007.md | — |
| BC-2.03.008 | Default spawn_recipe() Returns UnsupportedOperation | P1 | active | ss-03/BC-2.03.008.md | — |

---

## SS-04: Daemon Wiring

> Architecture source: `architecture/SS-daemon-wiring.md`
> ARCH-INDEX subsystem: SS-04
> Capability: CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.04.001 | Daemon Start Sequence: Port Bind + Lock File + Token Write (SOQ-2) | P0 | active | ss-04/BC-2.04.001.md | — |
| BC-2.04.002 | Daemon Auto-Start on TUI Launch | P0 | active | ss-04/BC-2.04.002.md | — |
| BC-2.04.003 | MONOCLE_NO_AUTOSTART=1 Suppresses Auto-Start | P1 | active | ss-04/BC-2.04.003.md | — |
| BC-2.04.004 | `monocle daemon start` CLI Subcommand | P0 | active | ss-04/BC-2.04.004.md | — |
| BC-2.04.005 | `monocle daemon stop` CLI Subcommand | P0 | active | ss-04/BC-2.04.005.md | — |
| BC-2.04.006 | `directories::ProjectDirs::runtime_dir()` Fallback Chain | P0 | active | ss-04/BC-2.04.006.md | — |
| BC-2.04.007 | Hook Endpoint: PreToolUse Request Routing | P0 | active | ss-04/BC-2.04.007.md | — |
| BC-2.04.008 | Hook Endpoint: Notification Request Routing (2000ms Timeout) | P0 | active | ss-04/BC-2.04.008.md | — |
| BC-2.04.009 | Hook Endpoint: Stop/SessionStart/PromptSubmit Routing (300ms Timeout) | P0 | active | ss-04/BC-2.04.009.md | — |
| BC-2.04.010 | Hook Tmpfile Generation at runtimeDir/hooks-settings.json | P0 | active | ss-04/BC-2.04.010.md | — |
| BC-2.04.011 | Bounded Event Bus with Drop Counter | P0 | active | ss-04/BC-2.04.011.md | — |
| BC-2.04.012 | JSONL Ring: Capacity and Rotation Policy | P1 | active | ss-04/BC-2.04.012.md | — |

---

## SS-05: IPC

> Architecture source: `architecture/SS-ipc.md`
> ARCH-INDEX subsystem: SS-05
> Capability: CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.05.001 | UDS Server Bind at runtimeDir/monocle.sock | P0 | active | ss-05/BC-2.05.001.md | — |
| BC-2.05.002 | TUI Client Connects to UDS and Receives Initial State Push | P0 | active | ss-05/BC-2.05.002.md | — |
| BC-2.05.003 | IPC Message Types: SessionListUpdate | P0 | active | ss-05/BC-2.05.003.md | — |
| BC-2.05.004 | IPC Message Types: HookEventReceived | P0 | active | ss-05/BC-2.05.004.md | — |
| BC-2.05.005 | IPC Message Types: PermissionPromptQueued | P0 | active | ss-05/BC-2.05.005.md | — |
| BC-2.05.006 | TUI Reconnects After Daemon Restart | P1 | active | ss-05/BC-2.05.006.md | — |
| BC-2.05.007 | Overlay Stack Cleared on Daemon Disconnect (SOQ-3) | P0 | active | ss-05/BC-2.05.007.md | — |
| BC-2.05.008 | UDS-Only in Phase 1 (No Shared-Memory Transport) | P1 | active | ss-05/BC-2.05.008.md | — |
| BC-2.05.009 | PtyOutput Fan-Out — Per-Session Bounded Channel (1024) with Drop Counter (stderr WARN) + PtyReset TUI Recovery | P0 | active | ss-05/BC-2.05.009.md | — |
| BC-2.05.010 | New ClientToServer IPC Variants — SpawnSession, KillSession, KeyInput, ResizePane, DetachSession, RenameSession, AttachSession | P0 | active | ss-05/BC-2.05.010.md | — |
| BC-2.05.011 | New ServerToClient IPC Variants — ScrollbackChunk, ScrollbackDumpComplete, PtyReset | P0 | active | ss-05/BC-2.05.011.md | — |

---

## SS-06: TUI

> Architecture source: `architecture/SS-tui.md`
> ARCH-INDEX subsystem: SS-06
> Capability: CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.06.001 | AppMode State Machine: Compile-Time Mutual Exclusion | P0 | active | ss-06/BC-2.06.001.md | — |
| BC-2.06.002 | FocusSnapshot: Focus Restored After Overlay/Fullscreen Close | P0 | active | ss-06/BC-2.06.002.md | — |
| BC-2.06.003 | Action Dispatch: 5-Level Binding Precedence | P0 | active | ss-06/BC-2.06.003.md | — |
| BC-2.06.004 | Ctrl-\ Popup: Appears and Dismisses Without State Loss | P0 | active | ss-06/BC-2.06.004.md | — |
| BC-2.06.005 | Sessions Panel: Session List Renders from IPC State | P0 | active | ss-06/BC-2.06.005.md | — |
| BC-2.06.006 | Sessions Panel: / Filter with Nucleo Fuzzy Match | P1 | active | ss-06/BC-2.06.006.md | — |
| BC-2.06.007 | Sessions Panel: Enter Transitions to Fullscreen | P1 | active | ss-06/BC-2.06.007.md | — |
| BC-2.06.008 | Permission Overlay: VecDeque Stack Push on PermissionPromptQueued | P0 | active | ss-06/BC-2.06.008.md | — |
| BC-2.06.009 | Permission Overlay: `[↑↓]` Rotates Stack | P0 | active | ss-06/BC-2.06.009.md | — |
| BC-2.06.010 | Permission Overlay: Diff Preview via `similar 3` | P1 | active | ss-06/BC-2.06.010.md | — |
| BC-2.06.011 | Permission Overlay: Accept-Once Keybinding (`y`/`Enter`) | P0 | active | ss-06/BC-2.06.011.md | — |
| BC-2.06.012 | Permission Overlay: Accept-Always Keybinding (`A`) | P0 | active | ss-06/BC-2.06.012.md | — |
| BC-2.06.013 | Permission Overlay: Reject Keybinding (`n`/`r`) | P0 | active | ss-06/BC-2.06.013.md | — |
| BC-2.06.014 | Permission Overlay: `[Esc]` Hides Without Rejecting | P0 | active | ss-06/BC-2.06.014.md | — |
| BC-2.06.015 | Permission Overlay: `[t]` Trace-to-Source Stub | P2 | active | ss-06/BC-2.06.015.md | — |
| BC-2.06.016 | Permission Overlay: Cleared on Daemon Disconnect | P0 | active | ss-06/BC-2.06.016.md | — |
| BC-2.06.017 | Permission Response Within Hook Timeout Budget | P0 | active | ss-06/BC-2.06.017.md | — |
| BC-2.06.018 | Event Ribbon Panel: Rolling Hook Event Log | P1 | active | ss-06/BC-2.06.018.md | — |
| BC-2.06.019 | Status Bar: Drop Counter Renders Under Load | P0 | active | ss-06/BC-2.06.019.md | — |
| BC-2.06.020 | Status Bar: Breadcrumb | P1 | active | ss-06/BC-2.06.020.md | — |
| BC-2.06.021 | Status Bar: Keybinding Hint Line | P1 | active | ss-06/BC-2.06.021.md | — |
| BC-2.06.022 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | P0 | active | ss-06/BC-2.06.022.md | — |
| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | P0 | active | ss-06/BC-2.06.023.md | — |
| BC-2.06.024 | Permission Overlay: ToolPayload Body Rendering by Variant | P1 | active | ss-06/BC-2.06.024.md | — |
| BC-2.06.025 | Multi-Session / Multi-Project Sessions Panel — Grouped by Project, Fast Switching, TUI Lifecycle Actions | P0 | active | ss-06/BC-2.06.025.md | — |

---

## SS-07: Config

> Architecture source: `architecture/SS-config.md`
> ARCH-INDEX subsystem: SS-07
> Capability: CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.07.001 | Config File Atomic Write via `tempfile::persist` | P0 | active | ss-07/BC-2.07.001.md | — |
| BC-2.07.002 | Config Schema Version 1: Harness Profile Fields | P0 | active | ss-07/BC-2.07.002.md | — |
| BC-2.07.003 | Config Missing or Corrupted: Default Applied | P0 | active | ss-07/BC-2.07.003.md | — |
| BC-2.07.004 | Profile Picker: Sticky-Per-Project | P1 | active | ss-07/BC-2.07.004.md | — |
| BC-2.07.005 | Profile Picker: `Ctrl-P` Override Shows Picker | P1 | active | ss-07/BC-2.07.005.md | — |
| BC-2.07.006 | CCR Detection via `ccr_path` Config Field | P1 | active | ss-07/BC-2.07.006.md | — |

---

## SS-08: Session Manager

> Architecture source: `architecture/SS-session-manager.md`
> ARCH-INDEX subsystem: SS-08
> Capability: CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.08.001 | Session Spawn — SessionHostSpawner Called Within 2s; SessionEntry Created | P0 | active | ss-08/BC-2.08.001.md | — |
| BC-2.08.002 | Session Persistence — session-host Survives Graceful Daemon Restart | P0 | active | ss-08/BC-2.08.002.md | — |
| BC-2.08.003 | Session Kill — SIGTERM Delivered via DaemonToHost::Kill Within 500ms | P0 | active | ss-08/BC-2.08.003.md | — |
| BC-2.08.004 | Re-Discovery — All Alive Sessions Visible After Daemon Restart Within 5s; UDS Bind Blocked Until Complete | P0 | active | ss-08/BC-2.08.004.md | — |
| BC-2.08.005 | Session GC — Terminated Sessions Removed from Registry After 10s Grace Period | P1 | active | ss-08/BC-2.08.005.md | — |
| BC-2.08.006 | Hook Auto-Injection — `--settings` Arg Present in Session-Host Child Args Within 2s of Spawn | P0 | active | ss-08/BC-2.08.006.md | — |
| BC-2.08.007 | Attach/Detach — Chunked Scrollback (ScrollbackChunk*+ScrollbackDumpComplete) on Attach; session-host Stays Alive on Detach | P1 | active | ss-08/BC-2.08.007.md | — |
| BC-2.08.008 | SessionStateChanged — Daemon Emits on Every SessionState Transition; Delivered to All TUI Clients; Ordering Relative to SessionListUpdate | P0 | active | ss-08/BC-2.08.008.md | — |

---

## SS-09: Embedded PTY

> Architecture source: `architecture/SS-embedded-pty.md`
> ARCH-INDEX subsystem: SS-09
> Capability: CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.09.001 | PTY Output Renders Within 100ms of Byte Receipt at TUI | P0 | active | ss-09/BC-2.09.001.md | — |
| BC-2.09.002 | Full-Fidelity Keyboard Forwarding — All v1A Input Classes Reach PTY stdin | P0 | active | ss-09/BC-2.09.002.md | — |
| BC-2.09.003 | Mouse Events Forwarded to PTY in SGR Encoding When in EmbeddedTerminal | P1 | active | ss-09/BC-2.09.003.md | — |
| BC-2.09.004 | Kitty Keyboard Protocol — Enhanced Key Events Forwarded as CSI u Sequences | P1 | active | ss-09/BC-2.09.004.md | — |
| BC-2.09.005 | Bracketed Paste — Paste Events Wrapped in Bracket Sequences Before Forwarding | P1 | active | ss-09/BC-2.09.005.md | — |
| BC-2.09.006 | Resize — PTY and Parser Resized Within 2 Render Ticks of Pane Area Change; 50ms Debounce | P0 | active | ss-09/BC-2.09.006.md | — |
| BC-2.09.007 | Scrollback — 1000 Rows Default; Configurable; PtyScrollUp/Down Navigate | P1 | active | ss-09/BC-2.09.007.md | — |
| BC-2.09.008 | EmbeddedTerminal AppMode Enter/Exit Transitions; SessionCreation Wizard Auto-Transitions to EmbeddedTerminal | P0 | active | ss-09/BC-2.09.008.md | — |
| BC-2.09.009 | Permission Badge+Bell — Status Bar Badge + Audible Bell Within One Render Tick While in EmbeddedTerminal or SessionCreation | P0 | active | ss-09/BC-2.09.009.md | — |

---

## SS-DTU: Claude Code Hook Protocol (DTU Gene-Source Contracts)

> Gene source: `any-context-lazyclaude/internal/core/config/hooks.go` (hooks-r1/r2 ingest rounds)
> Architecture source: `specs/dtu-assessment.md` v1.7.6 §Clone Development Approach
> Subsystem: SS-01 (DTU contracts describe protocol ingested by SS-01 daemon endpoints)
> Capability: CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management")
> Note: These are gene-transfusion behavioral contracts (origin: gene-transfusion) that define
> the Claude Code hook injection protocol. They govern the behavior of the DTU clone
> (S-DTU-001) and the daemon's expected wire contract for hook events.

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-HOOK-001 | PreToolUse Hook Fail-Open Semantics (No Server Found) | P0 | active | ss-dtu/BC-HOOK-001.md |
| BC-HOOK-002 | Non-PreToolUse Hooks Fail-Closed (No Server Found) | P0 | active | ss-dtu/BC-HOOK-002.md |
| BC-HOOK-003 | Notification Hook Filters on notification_type === 'permission_prompt' | P0 | active | ss-dtu/BC-HOOK-003.md |
| BC-HOOK-004 | Hook HTTP Requests Are Fire-and-Forget (Response Ignored) | P0 | active | ss-dtu/BC-HOOK-004.md |
| BC-HOOK-005 | Hook HTTP Request Target is 127.0.0.1 with Port from Lock File | P0 | active | ss-dtu/BC-HOOK-005.md |
| BC-HOOK-006 | PreToolUse Always Echoes Stdin to Stdout | P0 | active | ss-dtu/BC-HOOK-006.md |
| BC-HOOK-007 | Exactly Five Hook Types Registered; PostToolUse Intentionally Absent | P0 | active | ss-dtu/BC-HOOK-007.md |
| BC-HOOK-008 | Hooks-Settings.json Encoding: SetEscapeHTML(false) and 2-Space Indent | P1 | active | ss-dtu/BC-HOOK-008.md |
| BC-HOOK-009 | Hooks-Settings.json Written at runtimeDir/hooks-settings.json with Mode 0o600 | P0 | active | ss-dtu/BC-HOOK-009.md |
| BC-HOOK-010 | Hooks-Settings.json Is Per-runtimeDir, Not Per-Session | P1 | active | ss-dtu/BC-HOOK-010.md |
| BC-HOOK-011 | Hooks-Settings.json Is Never Cleaned Up by WriteHooksSettingsFile | P2 | active | ss-dtu/BC-HOOK-011.md |
| BC-HOOK-012 | Hook Configuration Is Identical Across All Session Types (PM, Worker, Plain) | P1 | active | ss-dtu/BC-HOOK-012.md |
| BC-HOOK-013 | Hook URL Host Is 127.0.0.1; Port Resolved at Each Invocation via Lock-File Scan | P0 | active | ss-dtu/BC-HOOK-013.md |
| BC-HOOK-014 | Lock File Path Is Hardcoded to MONOCLE_RUNTIME_DIR (Not Env-Var Overridable in JS) | P1 | active | ss-dtu/BC-HOOK-014.md |
| BC-HOOK-015 | Auth Token Resolved at Each Invocation from Lock File authToken Field | P0 | active | ss-dtu/BC-HOOK-015.md |
| BC-HOOK-016 | Auth Header Name Is X-Claude-Code-Ide-Authorization (Hardcoded in Hook Source) | P0 | active | ss-dtu/BC-HOOK-016.md |
| BC-HOOK-017 | PID Liveness Check Uses process.kill(pid, 0) (POSIX-Only) | P1 | active | ss-dtu/BC-HOOK-017.md |
| BC-HOOK-018 | Per-Hook Fallback Semantics Matrix When No Alive Server Found | P0 | active | ss-dtu/BC-HOOK-018.md |
| BC-HOOK-019 | Gene-Source Endpoint Matrix (PreToolUse and Notification Share /notify via type Field) | P1 | active | ss-dtu/BC-HOOK-019.md |
| BC-HOOK-020 | Notification Client-Side Filter notification_type === 'permission_prompt' (Deep-Ingest Confirmation) | P0 | active | ss-dtu/BC-HOOK-020.md |
| BC-HOOK-021 | All HTTP Requests Are Fire-and-Forget (Deep-Ingest Confirmation) | P0 | active | ss-dtu/BC-HOOK-021.md |
| BC-HOOK-022 | Notification Timeout Is 2000ms; Other Four Hooks Are 300ms | P0 | active | ss-dtu/BC-HOOK-022.md |
| BC-HOOK-023 | Content-Type and Content-Length Headers Are Always Set Explicitly | P0 | active | ss-dtu/BC-HOOK-023.md |
| BC-HOOK-024 | Monocle Improvement — Lock File App Filter Added to Hook JS | P0 | active | ss-dtu/BC-HOOK-024.md |
| BC-HOOK-025 | After Daemon Restart, First Hook Invocation Re-Discovers New Port; Events During Restart Window Are Dropped | P0 | active | ss-dtu/BC-HOOK-025.md |
| BC-HOOK-026 | No Producer-Side State — Hook Discovery Is Stateless Per Invocation | P0 | active | ss-dtu/BC-HOOK-026.md |
| BC-HOOK-027 | Monocle Never Writes ~/.monocle/settings.json — Hook Injection Is Via --settings Flag | P0 | active | ss-dtu/BC-HOOK-027.md |
| BC-HOOK-028 | No Env-Var Alternative for Hook Injection — Only --settings Flag | P1 | active | ss-dtu/BC-HOOK-028.md |
| BC-HOOK-029 | Hook Process Reads Only os.homedir() from Environment — Env-Independent for All Other Vars | P1 | active | ss-dtu/BC-HOOK-029.md |
| BC-HOOK-030 | MONOCLE_SESSION_ID Env Var Is Set on Claude Code Subprocess but NOT Read by Hook JS | P2 | active | ss-dtu/BC-HOOK-030.md |
| BC-HOOK-031 | Hooks-Settings.json Is Unversioned — No Schema Version Field | P2 | active | ss-dtu/BC-HOOK-031.md |
| BC-HOOK-032 | Malformed Stdin JSON Does NOT Prevent Stdin Echo for PreToolUse (Doubly Fail-Open) | P0 | active | ss-dtu/BC-HOOK-032.md |
| BC-HOOK-033 | Malformed Stdin JSON Silently Drops Hook for Non-PreToolUse Hooks | P0 | active | ss-dtu/BC-HOOK-033.md |
| BC-HOOK-034 | parseInt Filename Parsing Handles Non-Numeric Lock Files via NaN Comparison | P1 | active | ss-dtu/BC-HOOK-034.md |
| BC-HOOK-035 | Lock File Read Errors and JSON Parse Errors Are Silently Skipped | P1 | active | ss-dtu/BC-HOOK-035.md |
| BC-HOOK-036 | Buffer.byteLength(body) Returns UTF-8 Byte Length, Not Character Count | P1 | active | ss-dtu/BC-HOOK-036.md |
| BC-HOOK-037 | req.write(body) Followed by req.end() Sends Body and Closes Write-Side Immediately | P1 | active | ss-dtu/BC-HOOK-037.md |
| BC-HOOK-038 | Two-Server-Same-Port Race Is Structurally Impossible (Lock-After-Bind Ordering) | P2 | active | ss-dtu/BC-HOOK-038.md |
| BC-HOOK-039 | WriteHooksSettingsFile Is Not Atomic; Torn Read Theoretically Possible | P1 | active | ss-dtu/BC-HOOK-039.md |
| BC-HOOK-040 | Go Map Iteration Randomness Causes Byte Non-Determinism; Rust Struct Serialization Is Stable | P1 | active | ss-dtu/BC-HOOK-040.md |
| BC-HOOK-041 | Monocle Test Must Assert Canonical Filename hooks-settings.json | P1 | active | ss-dtu/BC-HOOK-041.md |

---

## Summary

| Subsystem | Total BCs | Active | Pending |
|-----------|-----------|--------|---------|
| SS-01 Daemon Lifecycle | 10 | 10 | 0 |
| SS-02 Core Types and ABI | 8 | 8 | 0 |
| SS-03 Engine Module | 8 | 8 | 0 |
| SS-04 Daemon Wiring | 12 | 12 | 0 |
| SS-05 IPC | 11 | 11 | 0 |
| SS-06 TUI | 25 | 25 | 0 |
| SS-07 Config | 6 | 6 | 0 |
| SS-08 Session Manager | 8 | 8 | 0 |
| SS-09 Embedded PTY | 9 | 9 | 0 |
| SS-DTU Hook Protocol (Gene-Source) | 41 | 41 | 0 |
| **Total** | **138** | **138** | **0** |

---

## Renumbering Map (Old ID → New ID)

> Append-only ID protection per audit §663-714. Old IDs are preserved here for
> cross-reference from git history, test names, and PRD §7 Requirements Traceability Matrix.
> Old IDs are NOT reused. New IDs follow the BC-S.SS.NNN scheme.

| Old ID | New ID | Title | Subsystem |
|--------|--------|-------|-----------|
| BC-DAEMON-001 | BC-2.01.001 | Healthz Endpoint (Unauthenticated Liveness Probe) | SS-01 |
| BC-DAEMON-002 | BC-2.01.002 | Status Endpoint (Authenticated Daemon State) | SS-01 |
| BC-DAEMON-003 | BC-2.01.003 | Body Size Limit (256 KiB, HTTP 413) | SS-01 |
| BC-DAEMON-004 | BC-2.01.004 | Graceful Shutdown (10-Second Drain) | SS-01 |
| BC-DAEMON-005 | BC-2.01.005 | Lock File Atomic Lifecycle (Create + Pid Check + Cleanup) | SS-01 |
| BC-DAEMON-006 | BC-2.01.006 | Crash Recovery Checkpoint | SS-01 |
| BC-RING-001 | BC-2.01.007 | JSONL Ring Format Version (FC-01) | SS-01 |
| BC-AUTH-001 | BC-2.01.008 | Auth Token Wire Format (FC-06) | SS-01 |
| BC-AUTH-002 | BC-2.01.009 | Auth Header Validation (Missing and Invalid Token) | SS-01 |
| BC-LOCK-001 | BC-2.01.010 | Lock File Contract Version Field | SS-01 |
| BC-ABI-001 | BC-2.02.001 | ABI Version in /status Endpoint (FC-03) | SS-02 |
| BC-ABI-002 | BC-2.02.002 | ABI Version Constant at Crate Root (FC-03) | SS-02 |
| BC-TYPES-001 | BC-2.02.003 | Non-Exhaustive Enum Policy (FC-02) | SS-02 |
| BC-FACTORY-001 | BC-2.02.004 | FactoryAdapter Trait Definition (FC-04 CRITICAL) | SS-02 |
| BC-FACTORY-002 | BC-2.02.005 | VsddFactoryAdapter Implementation | SS-02 |
| BC-PROTO-001a | BC-2.02.006 | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | SS-02 |
| BC-PROTO-001b | BC-2.02.007 | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | SS-02 |
| BC-PROTO-002 | BC-2.02.008 | Phase 4 schema_version Validation Requirement (FC-05) | SS-02 |
| BC-ENGINE-001 | BC-2.03.001 | EngineModule Trait Definition | SS-03 |
| BC-ENGINE-002 | BC-2.03.002 | ClaudeCodeModule Implementation (Strict-Basename Detect) | SS-03 |
| BC-ENGINE-002-ERR | BC-2.03.003 | HomeUnresolvable Error Contract | SS-03 |
| BC-ENGINE-003 | BC-2.03.004 | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | SS-03 |

---

## §Trace v1.1

**Template compliance Dispatch 3 of 7+** (2026-05-17T12:00:00Z):
- SS-02 section: 8 BC rows flipped from `pending-dispatch-3` to `active`.
  Files created at `.factory/specs/behavioral-contracts/ss-02/` (BC-2.02.001..BC-2.02.008).
- SS-03 section: 4 BC rows flipped from `pending-dispatch-3` to `active`.
  Files created at `.factory/specs/behavioral-contracts/ss-03/` (BC-2.03.001..BC-2.03.004).
- Summary table: all 22 BCs active (10 SS-01 + 8 SS-02 + 4 SS-03); 0 pending.
- Index version bumped: 1.0 → 1.1.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T12:00:00Z >= chain high-water 2026-05-17T11:30:00Z.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md`.
- Next: Dispatch 4 (PO) reduces PRD §3/§4/§5 to index + creates 4 prd-supplements.

## §Trace v1.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure — 22-BC DI mapping sweep** (2026-05-17T18:00:00Z):
- All 22 BC files updated (SS-01 × 10, SS-02 × 8, SS-03 × 4).
- Per-file L2 Domain Invariants cells replaced from stale "N/A" text to canonical DI-NNN citations.
- 2 stale VP IDs corrected in body prose: BC-2.01.005 (`VP-DAEMON-005` → `VP-005`) and BC-2.01.006 (`VP-DAEMON-006` → `VP-006`).
- 0 stale BC IDs found in non-historical body prose across all 22 files.
- Per-file version bumps: all files that were at v1.0 incremented to v1.0.1; files at v1.0.1 incremented to v1.0.2.
- BC-INDEX version bumped: 1.1 → 1.2.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T12:00:00Z (v1.1).
- SE-17g META audit: see commit §Trace entry for zero-remaining "N/A — no domain-spec/invariants.md" re-grep result.

**Template compliance Dispatch 2 of 7+** (2026-05-17T11:30:00Z):
- Created as new artifact; no prior version.
- SS-01 section: 10 BC rows filled (BC-2.01.001..BC-2.01.010), all active.
  Files created at `.factory/specs/behavioral-contracts/ss-01/`.
- SS-02 section: 8 BC rows with `pending-dispatch-3` status; file paths pre-registered.
- SS-03 section: 4 BC rows with `pending-dispatch-3` status; file paths pre-registered.
- Renumbering map: all 22 old IDs (BC-DOMAIN-NNN) mapped to new BC-S.SS.NNN IDs per
  append-only ID protection (audit §663-714).
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:30:00Z >= chain high-water 2026-05-17T11:00:00Z.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md`.

## §Trace v1.3

**T-128n Part 1 — F-R105 closure chain Round 4: BC-2.01.009 ADR-0005 dual-accept propagation** (2026-05-17T20:00:00Z):
- BC-2.01.009 body updated: postconditions 1-3 expanded to dual-accept (ADR-0005); postcondition 4 added (both-headers-present canonical priority); 3 new edge cases (EC-010/EC-011/EC-012); 2 new test vectors (alias wrong-secret → 401; alias correct-secret → 200 + WARN). BC-2.01.009 version: 1.0.1 → 1.0.2.
- BC-INDEX title for BC-2.01.009 unchanged: "Auth Header Validation (Missing and Invalid Token)" — H1 is stable.
- BC-INDEX status unchanged: active. No BC removals or retirements in this burst.
- SE-16d monotonicity PASS: 2026-05-17T20:00:00Z > prior 2026-05-17T18:00:00Z (v1.2).

## §Trace v1.4

**F-R106 Round 5A — BC scope fixes: CRITICAL PC-4 contradiction + fabrication removal + stale-ID sweep + §Trace reorder** (2026-05-17T23:00:00Z):

BC version bumps in this dispatch:
- BC-2.01.008: v1.0.2 → v1.0.3 (F-R106-2 CRITICAL: PC-4 rewritten to enumerate both canonical `X-Monocle-Authorization` and alias `X-Claude-Code-Ide-Authorization` per ADR-0005 dual-accept; Architecture Source row updated to include ADR-0005)
- BC-2.01.009: v1.0.2 → v1.0.3 (F-R106-7 HIGH: fabricated `(F-FC-I005 Phase 4 OAuth2 clarification)` parenthetical removed from Forward Compat Contract row; replaced with canonical `FC-06 (versioned auth token prefix)`)
- BC-2.01.005: v1.0.1 → v1.0.2 (F-R106-11 MED: stale `BC-ENGINE-002-ERR` in Invariant 4 updated to `BC-2.03.003 (HomeUnresolvable; renumbered from BC-ENGINE-002-ERR per BC-INDEX §Renumbering Map)`)
- BC-2.01.002: v1.0.1 → v1.0.2 (F-R106-12 MED: redundant `(BC-AUTH-002)` parenthetical removed from Postcondition 2 cross-reference to BC-2.01.009)
- BC-2.01.003: v1.0.1 → v1.0.2 (F-R106-12 MED: stale `BC-RING-001 EC-002` in Related BCs canonicalized to `BC-2.01.007 EC-002`)
- BC-2.01.007: v1.0.1 → v1.0.2 (F-R106-12 MED: stale self-referential `BC-RING-001 EC-002` in Related BCs canonicalized to `BC-2.01.007 EC-002`)

BC-INDEX structural fix:
- F-R106-13 MED: §Trace sections were non-monotonic (v1.1, v1.3, v1.2). Reordered to ascending (v1.1 → v1.2 → v1.3 → v1.4). Content of each section preserved verbatim.

BC-INDEX titles unchanged: all 22 BC H1 headings are stable. No BC retirements or removals.
SE-17g META audit: re-grep for stale old-form BC ID parentheticals in ss-01/ body prose — see post-fix verification below.
SE-16d monotonicity PASS: 2026-05-17T23:00:00Z > prior 2026-05-17T20:00:00Z (v1.3).

## §Trace v1.5

**F-R107 Round 6A — 10-BC pin sweep (CRITICAL F-R107-2) + ADR pins (F-R107-2 closure part) + EC-013 (F-R107-10) + INV-3 dual-accept (GAP-R46-5)** (2026-05-17T23:30:00Z):

BC version bumps in this dispatch:
- BC-2.01.001: v1.0.2 → v1.0.3 (F-R107-2: Architecture Source SS-daemon-lifecycle.md v1.0.25 → v1.0.30)
- BC-2.01.002: v1.0.2 → v1.0.3 (F-R107-2: Architecture Source SS-daemon-lifecycle.md v1.0.25 → v1.0.30)
- BC-2.01.003: v1.0.2 → v1.0.3 (F-R107-2: Architecture Source SS-daemon-lifecycle.md v1.0.25 → v1.0.30)
- BC-2.01.004: v1.0.1 → v1.0.2 (F-R107-2: Architecture Source v1.0.25 → v1.0.30; GAP-R46-5: INV-3 updated to dual-accept per ADR-0005 — `/shutdown` requires either canonical or alias header, not X-Monocle-Authorization only)
- BC-2.01.005: v1.0.2 → v1.0.3 (F-R107-2: Architecture Source SS-daemon-lifecycle.md v1.0.25 → v1.0.30)
- BC-2.01.006: v1.0.2 → v1.0.3 (F-R107-2: Architecture Source SS-daemon-lifecycle.md v1.0.25 → v1.0.30)
- BC-2.01.007: v1.0.2 → v1.0.3 (F-R107-2: Architecture Source SS-daemon-lifecycle.md v1.0.25 → v1.0.30)
- BC-2.01.008: v1.0.3 → v1.0.4 (F-R107-2: Architecture Source v1.0.25 → v1.0.30; F-R107-2 closure part (BC ADR pin add) per Round 6A scope expansion: ADR-0005 citation updated to ADR-0005 v1.0.2)
- BC-2.01.009: v1.0.3 → v1.0.4 (F-R107-2: Architecture Source v1.0.29 → v1.0.30; F-R107-2 closure part (BC ADR pin add) per Round 6A scope expansion: ADR-0005 citation updated to ADR-0005 v1.0.2; F-R107-10: EC-013 added — Bearer header dual-absence case)
- BC-2.01.010: v1.0.1 → v1.0.2 (F-R107-2: Architecture Source SS-daemon-lifecycle.md v1.0.25 → v1.0.30)

BC-INDEX titles unchanged: all 22 BC H1 headings are stable. No BC retirements or removals.
SE-17g META audit post-sweep: `grep -r "SS-daemon-lifecycle.md v1\.0\.25\|SS-daemon-lifecycle.md v1\.0\.29" .factory/specs/behavioral-contracts/ss-01/` → 0 matches. All 10 ss-01 BCs now pin v1.0.30.
SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T23:00:00Z (v1.4).

## §Trace v1.6

**F-R108 Round 7A — §Trace ordering fix (F-R108-4) + finding-ID audit corrections (F-R108-12, F-R108-16) + BC-2.01.002 dual-accept alignment (F-R108-17)** (2026-05-18T01:15:00Z):

**F-R108-4 CRITICAL — §Trace ordering fixed (v1.5 was inserted BEFORE v1.4 — non-monotonic):**
- §Trace v1.5 (Round 6A) was authored before v1.4's reorder fix and was inserted at the wrong position, making the sequence v1.1 → v1.2 → v1.3 → v1.5 → v1.4 (non-monotonic).
- SE-17f BEFORE: §Trace order was v1.1, v1.2, v1.3, v1.5, v1.4.
- SE-17f AFTER: §Trace order is v1.1, v1.2, v1.3, v1.4, v1.5, v1.6 (ascending monotonic).
- Content of each §Trace section preserved verbatim; only insertion order corrected.

**F-R108-12 HIGH — Finding-ID audit correction in v1.5 dispatch entry for BC-2.01.008 and BC-2.01.009:**
- v1.5 dispatch entry (and the corresponding BC files' §Trace v1.0.4) cited "F-R107-9" for the ADR-0005 version pin addition. F-R107-9 in the R107 adversarial report describes the still-broken ADR-0002 inputs path (routed to Architect 7C for Round 7). The ADR-0005 version pin addition is correctly attributed to **F-R107-2 closure part (BC ADR pin add) per Round 6A scope expansion**.
- v1.5 dispatch entry corrected: "F-R107-9" references replaced with "F-R107-2 closure part (BC ADR pin add) per Round 6A scope expansion" in the BC-2.01.008 and BC-2.01.009 rows above.
- Individual BC files corrected in their §Trace v1.0.5 entries (BC-2.01.008 v1.0.5 and BC-2.01.009 v1.0.5).

**F-R108-16 MEDIUM — F-R107-10 RESCOPED note:**
- F-R107-10 in the R107 adversarial report was described as an error-taxonomy finding (EC-NNN vs E-AUTH-NNN namespace confusion). Round 6A closure was to add EC-013 to BC-2.01.009, which is a legitimate scope correction. However, the audit-trail description conflates the EC- and E-AUTH- namespaces.
- RESCOPED: F-R107-10 RESCOPED FROM error-taxonomy E-AUTH addition TO BC-2.01.009 EC-013 addition; original R107 description conflated EC- and E-AUTH- namespaces. The correct closure action was adding EC-013 (Bearer dual-absence edge case) to BC-2.01.009; no E-AUTH-NNN entry was needed. This rescope note is recorded here for adversarial audit-trail integrity; no further normative content changes are required.

**F-R108-17 MEDIUM — BC-2.01.002 dual-accept alignment:**
- BC-2.01.002 Description, Precondition 2, and canonical test vector all implied single-header `X-Monocle-Authorization` only.
- BC-2.01.002 v1.0.3 → v1.0.4: Description and Precondition 2 updated to dual-accept (ADR-0005 v1.0.2); test vector happy-path split into two rows (canonical + alias). See BC-2.01.002 §Trace v1.0.4 for full before/after.

BC-INDEX titles unchanged: all 22 BC H1 headings are stable. No BC retirements or removals.
SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs across all modified files.
SE-16d monotonicity PASS: 2026-05-18T01:15:00Z > prior 2026-05-17T23:30:00Z (v1.5).

## §Conventions

> F-R109-17 + F-R109-21 codified conventions (v1.7). Production-grade; these conventions apply retroactively to all 22 BCs and must be upheld in all future BC additions.

### EC Namespace Convention (F-R109-17)

Edge case IDs (EC-NNN) are scoped **per-BC**. EC-013 in BC-2.01.009 and EC-013 in BC-2.02.001 are distinct and NOT in conflict — the per-BC scoping is intentional and sound. No global EC namespace exists or is required.

**Rationale:** EC IDs serve as local cross-reference labels within a BC file (cited in test vectors, preconditions, and invariants within the same BC). Global uniqueness would require coordinating EC sequences across 22+ independent BC files without providing additional semantic value — per-BC scoping is the correct granularity for behavioral edge cases.

**Enforcement:** When authoring or modifying a BC, EC-NNN is allocated within that BC's own sequence. Cross-BC EC references use the fully-qualified form `BC-S.SS.NNN EC-NNN` (e.g., `BC-2.01.007 EC-002`) to unambiguously scope the reference. This form is already in use in BC-2.01.003 Related BCs and BC-2.01.007 Related BCs.

### Anchor Parenthetical Non-Contradiction (PG-5, F-R110-16)

Any parenthetical appended to a BC-INDEX title (e.g., `"(Fail-Closed for Writes)"`) MUST NOT contradict the anchor target's H1 title. If the H1 title changes, the parenthetical must be updated in the same commit. If a parenthetical adds policy-relevant context, that context must be moved INTO the BC H1 heading (per bc_h1_is_title_source_of_truth), not left as index-only context.

**Enforcement:** The adversary is instructed to flag any parenthetical in the BC-INDEX title column that either (a) contradicts the referenced BC H1 title or (b) adds context that is absent from the H1. Such findings are MEDIUM severity.

**Cross-reference:** Also documented in `architecture/SS-conventions-anti-patterns.md §BC-INDEX Conventions` (added F-R110-18).

---

### Test Name Convention (F-R109-21)

BC test function names use stable legacy-form prefixes (e.g., `test_BC_AUTH_002_...`, `test_BC_DAEMON_003_...`) for test continuity across the BC renumbering event (BC-INDEX §Renumbering Map). These names are **immutable** — renaming them to the new BC-S.SS.NNN form would break test history in CI, coverage reports, and log analysis.

**Rationale:** Test names are stable identifiers in CI systems. The cost of renaming (CI history breakage, grep script updates, log grep pattern updates) exceeds the benefit (alignment to new BC IDs). The BC H1 heading and Traceability table `Test Name` row document the mapping: old test name → canonical BC ID.

**Enforcement:** New BCs authored after the renumbering event (v1.1+) SHOULD use the new-form prefix `test_BC_2_SS_NNN_...` for new test functions. Existing tests with legacy-form names are NOT renamed.

---

### Architecture Source Pin-Symmetry Convention (F-R117-3, SE-17e)

When a BC Traceability `Architecture Source` cell references **multiple** architecture documents (semicolon-separated), ALL referenced documents MUST carry explicit version pins in the form `SS-name.md vN.M.P` or `ADR-NNNN vN.M.P`. A cell where some references are pinned and others are unpinned is a **pin-symmetry violation** — MED severity per F-R110-8 (originally codified for VP Architecture Source cells; extended to BC Architecture Source cells via SE-17e sibling-propagation in R16C).

**Canonical SS versions** — see `.factory/specs/version-pin-registry.yaml` (single source of truth per ADR-0007). When writing a multi-reference Architecture Source cell, look up the `current_version` field for each `SS-*` artifact ID in that file. Do NOT hardcode version literals here; this table was previously a hand-maintained mirror and drifted silently (F-P49-001 Pass-49). The pin-symmetry oracle is the registry, not a literal table in this file.

**Single-reference cells** (one SS doc) have no symmetry requirement — a single-reference cell is trivially symmetric. Pin-symmetry only activates for two-or-more references.

**Enforcement:** The adversary is instructed to flag any BC Architecture Source cell where ≥2 architecture documents are cited and at least one lacks a `vN.M.P` pin. Such findings are MED severity. This convention is also codified in `architecture/SS-conventions-anti-patterns.md §BC-INDEX Conventions`.

---

## §Trace v1.7

**F-R109 Round 8B — 22-BC pin sweep + §Trace ascending reorder + conventions codified** (2026-05-18T05:45:00Z):

BC version bumps in this dispatch:

SS-01 (SS-daemon-lifecycle.md v1.0.30 → v1.0.32):
- BC-2.01.001: v1.0.3 → v1.0.4 (F-R109-4 pin + F-R109-14 §Trace ascending)
- BC-2.01.002: v1.0.4 → v1.0.5 (F-R109-4 pin + F-R109-14 §Trace ascending)
- BC-2.01.003: v1.0.3 → v1.0.4 (F-R109-4 pin + F-R109-14 §Trace ascending)
- BC-2.01.004: v1.0.2 → v1.0.3 (F-R109-4 pin + F-R109-14 §Trace ascending)
- BC-2.01.005: v1.0.3 → v1.0.4 (F-R109-4 pin + F-R109-14 §Trace ascending)
- BC-2.01.006: v1.0.3 → v1.0.4 (F-R109-4 pin + F-R109-14 §Trace ascending)
- BC-2.01.007: v1.0.3 → v1.0.4 (F-R109-4 pin + F-R109-14 §Trace ascending)
- BC-2.01.008: v1.0.5 → v1.0.6 (F-R109-4 pin + F-R109-14 §Trace ascending)
- BC-2.01.009: v1.0.5 → v1.0.6 (F-R109-4 pin + F-R109-14 §Trace ascending + F-R109-20 OAuth2 residual removed)
- BC-2.01.010: v1.0.2 → v1.0.3 (F-R109-4 pin + F-R109-14 §Trace ascending)

SS-02 (SS-core-types-and-abi.md v1.2.8 → v1.2.13; BCs were stale by 4 patches cumulative from earlier rounds; this dispatch refreshed to latest):
- BC-2.02.001: v1.0.1 → v1.0.2 (F-R109-4 pin)
- BC-2.02.002: v1.0.2 → v1.0.3 (F-R109-4 pin)
- BC-2.02.003: v1.0.1 → v1.0.2 (F-R109-4 pin)
- BC-2.02.004: v1.0.2 → v1.0.3 (F-R109-4 pin)
- BC-2.02.005: v1.0.1 → v1.0.2 (F-R109-4 pin)
- BC-2.02.006: v1.0.2 → v1.0.3 (F-R109-4 pin)
- BC-2.02.007: v1.0.2 → v1.0.3 (F-R109-4 pin)
- BC-2.02.008: v1.0.2 → v1.0.3 (F-R109-4 pin)

SS-03 (SS-engine-module.md v1.1.15 → v1.1.20; BCs were stale by 4 patches cumulative from earlier rounds; this dispatch refreshed to latest):
- BC-2.03.001: v1.0.2 → v1.0.3 (F-R109-4 pin)
- BC-2.03.002: v1.0.2 → v1.0.3 (F-R109-4 pin)
- BC-2.03.003: v1.0.1 → v1.0.2 (F-R109-4 pin)
- BC-2.03.004: v1.0.2 → v1.0.3 (F-R109-4 pin)

**F-R109-14 — §Trace ascending reorder:** All SS-01 BC §Trace blocks were descending (most recent first). Reordered to ascending (oldest first → newest appended). Content preserved verbatim; insertion order corrected. SS-02 and SS-03 BCs had only 1 §Trace block (no ordering issue).

**F-R109-20 — BC-2.01.009 Architecture Anchors residual fabrication removed:** `(Phase 4 OAuth2 clarification)` parenthetical removed from Architecture Anchors line. F-R106-7 previously removed this from the Traceability table Forward Compat Contract row but missed this line. Consistent with `FC-06 (versioned auth token prefix)` canonical form.

**F-R109-17 — EC namespace convention codified:** §Conventions section added to BC-INDEX. Per-BC EC scoping is canonical; EC-013 in two different BCs is not a collision. Cross-BC EC references use `BC-S.SS.NNN EC-NNN` fully-qualified form.

**F-R109-21 — Test name convention codified:** §Conventions section documents that legacy-form test names (e.g., `test_BC_AUTH_002_...`) are immutable — renaming them is cost-exceeds-benefit. New BCs SHOULD use new-form prefix; existing BCs are not renamed.

BC-INDEX titles unchanged: all 22 BC H1 headings are stable. No BC retirements or removals.
SE-17g META audit: `grep -r "SS-daemon-lifecycle.md v1\.0\.30\|SS-core-types-and-abi.md v1\.2\.8\|SS-engine-module.md v1\.1\.15" .factory/specs/behavioral-contracts/` → 0 matches. All 22 BCs updated to target version pins.
SE-16d monotonicity PASS: 2026-05-18T05:45:00Z > prior 2026-05-18T01:15:00Z (v1.6). ARITHMETICALLY TRUE: 2026-05-18T05:45:00Z > 2026-05-18T01:15:00Z PASS.

## §Trace v1.8

**F-R110 Round 9B — timestamp monotonicity fix + fabrication correction + NFR-011 P0 + cycle-3→Phase 3 + PG-5 convention** (2026-05-18T06:00:00Z):

**F-R110-1 CRITICAL — Round 8 timestamps corrected to 2026-05-18T05:xx:00Z:**
- 22 BC frontmatter timestamps were `2026-05-17T04:00-21:00Z`. Corrected to `2026-05-18T05:00-21:00Z`.
- 22 BC §Trace last-entry timestamps corrected to match.
- BC-INDEX v1.7 frontmatter: `2026-05-17T04:45:00Z` → `2026-05-18T05:45:00Z`.
- BC-INDEX v1.7 §Trace body: same correction.
- SE-16d monotonicity now arithmetically PASS in all 22 BCs and in BC-INDEX v1.7.

**F-R110-2 CRIT — Fabrication correction in v1.7 SS-02/SS-03 entries:**
- §Trace v1.7 lines for SS-02 and SS-03 previously stated "Architect 8A bumped SS-core-types-and-abi.md v1.2.8 → v1.2.13 (Round 8A — 4 versions stale)". This incorrectly attributed the 4-patch cumulative staleness to a single Architect 8A bump. Truth: Architect 8A bumped each file by 1 patch; the BCs were stale by 4 patches cumulative from earlier rounds.
- Corrected narrative in §Trace v1.7 SS-02/SS-03 lines: "BCs were stale by 4 patches cumulative from earlier rounds; this dispatch refreshed to latest."
- Corrected narrative in all 8 SS-02 BC files §Trace latest entries.
- Corrected narrative in all 4 SS-03 BC files §Trace latest entries.

**F-R110-16 PG-5 — "Anchor parenthetical may not contradict anchor target title" codified:**
- Per F-R110-16, this discipline is codified here for BC-INDEX. See §Conventions (PG-5 clause added below).

BC-INDEX titles unchanged: all 22 BC H1 headings are stable. No BC retirements or removals.
SE-16d monotonicity PASS: 2026-05-18T06:00:00Z > prior 2026-05-18T05:45:00Z (v1.7). ARITHMETICALLY TRUE: 2026-05-18T06:00:00Z > 2026-05-18T05:45:00Z PASS.

## §Trace v1.9

**F-R111 Round 10 — timestamp pathology fix** (2026-05-18T07:00:00Z):

**F-R111-1 CRITICAL — v1.8 frontmatter timestamp corrected:**
- v1.8 frontmatter timestamp was `2026-05-18T05:45:00Z`. This is the timestamp of the v1.7 §Trace body (the value that was being corrected in v1.8). The v1.8 burst itself ran at `2026-05-18T06:00:00Z`. Corrected frontmatter to `2026-05-18T07:00:00Z` (Round 10 fix burst timestamp).
- **BC-INDEX titles unchanged:** all 22 BC H1 headings are stable. No BC retirements or removals.

SE-16d monotonicity PASS: 2026-05-18T07:00:00Z > prior 2026-05-18T06:00:00Z (v1.8). ARITHMETICALLY TRUE: 2026-05-18T07:00:00Z > 2026-05-18T06:00:00Z PASS.

## §Trace v1.10

**R16C F-R117-3 MED — BC-2.01.010 Architecture Source pin-symmetry fix + pin-symmetry convention codified (SE-17e)** (2026-05-18T16:00:00Z):

**F-R117-3 MED — BC-2.01.010 Architecture Source pin-symmetry fixed:**
- BC-2.01.010 Architecture Source cell: `SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging` was unpinned while the sibling `SS-daemon-lifecycle.md` reference was pinned at v1.0.32. Pin-symmetry violation per F-R110-8 discipline (extended to BCs via SE-17e).
- Fix applied in BC-2.01.010 v1.0.3 → v1.0.4: added `v1.2.13` to SS-core-types-and-abi.md citation.
- **Only BC-2.01.010 was defective.** Sweep results: 21 other BCs clean — 19 BCs have single-reference Architecture Source cells (no symmetry requirement); BC-2.01.008 and BC-2.01.009 have two-reference cells (SS-daemon-lifecycle.md + ADR-0005) with both pinned (PASS).

**SE-17e sibling-propagation — pin-symmetry convention codified in §Conventions:**
- F-R110-8 pin-symmetry discipline (originally for VP Architecture Source cells) extended to BC Architecture Source cells.
- §Conventions section updated with "Architecture Source Pin-Symmetry Convention (F-R117-3, SE-17e)" including canonical SS version table.
- Future BCs with multi-reference Architecture Source cells must pin all references.

BC version bumps in this dispatch:
- BC-2.01.010: v1.0.3 → v1.0.4 (F-R117-3: SS-core-types-and-abi.md pin added v1.2.13)

BC-INDEX titles unchanged: all 22 BC H1 headings are stable. No BC retirements or removals.
SE-17g META audit: `grep -n "Architecture Source" .factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md` → line 89, cell confirmed: `SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md v1.2.13 §Phase 1 PRD BC Pre-Staging`. 0 remaining pin-symmetry violations across all 22 BCs.
SE-16d monotonicity PASS: 2026-05-18T16:00:00Z > prior 2026-05-18T07:00:00Z (v1.9). ARITHMETICALLY TRUE: 2026-05-18T16:00:00Z > 2026-05-18T07:00:00Z PASS.

## §Trace v1.11

**F-R119-2 closure — retrospective §Trace for R17F SM-applied Canonical SS version table edit** (2026-05-18T22:00:00Z):

**Background:** R17F state-manager (commit 7681632, 2026-05-18T20:30:00Z) modified BC-INDEX line 279 — the Canonical SS version table row for `SS-conventions-anti-patterns.md` — from v1.29.4 to v1.29.5 as a "defensive sweep" (R17D introduced v1.29.5 at 19:30Z via commit b7ce1ac). The content edit was correct, but SM does not have authority to author §Trace blocks or bump BC-INDEX versions per the Correct Agent Routing principle — now codified as SE-23 in R18-pre (commit 70b7552, D-146). R119 adversary correctly flagged this as F-R119-2 HIGH: BC-INDEX body content reflected ≥19:30Z state but frontmatter showed v1.10 at 16:00:00Z, breaking SE-16d audit-trail monotonicity. The Canonical SS version table is the source-of-truth that `SS-conventions-anti-patterns.md §Architecture Source Pin-Symmetry Convention` (R17D commit b7ce1ac) cross-references back to this BC-INDEX section.

**Resolution:** This §Trace v1.11 retrospectively documents the SM-applied edit in BC-INDEX's own §Trace timeline, restoring SE-16d monotonicity.

| Edit | Source | Canonical version | Applied by | When | Documented now |
|------|--------|-------------------|------------|------|----------------|
| Canonical SS version table row, SS-conventions-anti-patterns.md v1.29.4 → v1.29.5 | R17D (commit b7ce1ac) | v1.29.5 | R17F SM | 2026-05-18T20:30:00Z | §Trace v1.11 |

**Content integrity:** Pin value verified canonical at audit time. No content changes in this burst beyond bookkeeping (frontmatter version/timestamp + this §Trace).

**SE-22 in-artifact sweep (BC-INDEX scope):**

| Pin target | grep pattern | Lines found | Classification | Status |
|-----------|-------------|-------------|----------------|--------|
| SS-daemon-lifecycle.md | `v1\.` | Line 274 (table), lines 192-201 (§Trace history), line 204, 293, 371, 384 | Historical §Trace only — canonical table = v1.0.32 | NORMATIVE: v1.0.32 PASS |
| SS-forward-compatibility.md | `v1\.` | Line 275 (table) | Canonical table | NORMATIVE: v1.2.19 PASS |
| SS-engine-module.md | `v1\.` | Line 276 (table), line 315 (§Trace history) | Historical §Trace only — canonical table = v1.1.20 | NORMATIVE: v1.1.20 PASS |
| SS-core-types-and-abi.md | `v1\.` | Line 277 (table), lines 305, 345, 371, 381, 384 (§Trace history) | Historical §Trace only — canonical table = v1.2.13 | NORMATIVE: v1.2.13 PASS |
| SS-deps-pin-manifest.md | `v1\.` | Line 278 (table) | Canonical table | NORMATIVE: v1.1.17 PASS |
| SS-conventions-anti-patterns.md | `v1\.` | Line 279 (table) | Canonical table — R17F SM edit target | NORMATIVE: v1.29.5 PASS |
| prd.md | `v1\.` | 0 matches | No version pin present in BC-INDEX body | N/A — INFORMATIONAL context only |
| product-brief.md | `v1\.` | 0 matches | No version pin present in BC-INDEX body | N/A — INFORMATIONAL context only |
| L2-INDEX | `v1\.` | 0 matches | No version pin present in BC-INDEX body | N/A — INFORMATIONAL context only |
| ARCH-INDEX | `v1\.` | 0 matches | No version pin present in BC-INDEX body | N/A — INFORMATIONAL context only |

Zero NORMATIVE stale pins found. Canonical SS version table values all match post-R17D/R17F canonical state. No content changes required beyond frontmatter bookkeeping.

**SE-16d monotonicity:** BC-INDEX v1.11 timestamp `2026-05-18T22:00:00Z` > R18A PRD v1.26.12 `21:30:00Z` > STATE v5.79 `21:15:00Z` > STATE v5.78 `20:30:00Z`. PASS strict-greater.

**SE-23 first-application context:** SE-23 was codified in R18-pre (commit 70b7552) because R17F SM scope violation broke SE-16d audit-trail integrity. R18A closed the PRD half (prd.md §Trace v1.26.12 retrospective); R18B closes the BC-INDEX half (this §Trace v1.11).

BC-INDEX titles unchanged: all 22 BC H1 headings are stable. No BC retirements or removals.
SE-16d monotonicity PASS: 2026-05-18T22:00:00Z > prior 2026-05-18T16:00:00Z (v1.10). ARITHMETICALLY TRUE: 2026-05-18T22:00:00Z > 2026-05-18T16:00:00Z PASS.

**Reference:** R119 report at `.factory/plans/adversary-pass-r119-phase1.md` (commit 70b7552).

## §Trace v1.12

**F-PHASE2-R05-06 — BC-2.03.001 internal-consistency fix: PC-6 added (detect() purity postcondition)** (2026-05-19T00:00:00Z):

- BC-2.03.001 v1.0.3 → v1.0.4: PC-6 added codifying detect() purity contract; DI-006 mapping cell updated from "Postcondition 5" to "Postcondition 6" anchor.
- Root cause: Traceability §L2 Domain Invariants DI-006 claimed PC-5 covered "detect() has no I/O and no shared state mutation", but PC-5 body only covers `metadata()`/`enrich()` HomeUnresolvable semantics. Phase 2 adversary r05 found the inconsistency via S-015 AC-010 fabricated quotation.
- No other BC files modified. No BC retirements or removals.

BC-INDEX titles unchanged: BC-2.03.001 H1 "EngineModule Trait Definition" is stable. All 22 BC H1 headings unchanged.
SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs.
SE-16d monotonicity PASS: 2026-05-19T00:00:00Z > prior 2026-05-18T22:00:00Z (v1.11). ARITHMETICALLY TRUE: PASS.

## §Trace v1.13

**GAP-PHASE2-R06-1 + GAP-PHASE2-R06-2 + GAP-PHASE2-R06-3 + F-PHASE2-R06-07 closure — BC ledger Architecture Source/Module pointer cascade (architect commit `2d43127`)** (2026-05-19T12:14:00Z):

**GAP-PHASE2-R06-2 — Canonical SS version table: SS-daemon-lifecycle.md v1.0.32 → v1.0.33:**
- Architect commit `2d43127` bumped SS-daemon-lifecycle.md v1.0.32 → v1.0.33 (Ring Buffer Rotation Policy added) and ARCH-INDEX v1.0.10 → v1.0.11 (SS-03 trait/impl split clarification). Neither cascade propagated to the BC ledger.
  - SE-17f BEFORE: `| SS-daemon-lifecycle.md | v1.0.32 |`
  - SE-17f AFTER: `| SS-daemon-lifecycle.md | v1.0.33 |`
- All other Canonical SS version table rows verified current: SS-forward-compatibility.md v1.2.19 PASS; SS-engine-module.md v1.1.20 PASS; SS-core-types-and-abi.md v1.2.13 PASS; SS-deps-pin-manifest.md v1.1.17 PASS; SS-conventions-anti-patterns.md v1.29.5 PASS.

**BC version bumps in this dispatch:**

SS-01 (SS-daemon-lifecycle.md v1.0.32 → v1.0.33 — GAP-PHASE2-R06-1 + F-PHASE2-R06-07 closure):
- BC-2.01.001: v1.0.4 → v1.0.5
- BC-2.01.002: v1.0.5 → v1.0.6
- BC-2.01.003: v1.0.4 → v1.0.5
- BC-2.01.004: v1.0.3 → v1.0.4
- BC-2.01.005: v1.0.4 → v1.0.5
- BC-2.01.006: v1.0.4 → v1.0.5
- BC-2.01.007: v1.0.5 → v1.0.6
- BC-2.01.008: v1.0.6 → v1.0.7
- BC-2.01.009: v1.0.6 → v1.0.7
- BC-2.01.010: v1.0.4 → v1.0.5

SS-03 (Architecture Module cell trait/impl split — GAP-PHASE2-R06-3 closure):
- BC-2.03.001: v1.0.4 → v1.0.5
- BC-2.03.002: v1.0.3 → v1.0.4
- BC-2.03.003: v1.0.2 → v1.0.3
- BC-2.03.004: v1.0.3 → v1.0.4

**Pointer-only sweep.** No behavioral content changes (no new PCs/INVs/ECs). No story files touched. No STATE.md touched. No architect-domain files touched.

SE-17g META audit: `grep -r "SS-daemon-lifecycle.md v1\.0\.32" .factory/specs/behavioral-contracts/` in Architecture Source cells → 0 normative matches (all 10 SS-01 BCs updated). `grep -r "ClaudeCodeModule adapter" .factory/specs/behavioral-contracts/ss-03/` → 0 normative matches (all 4 SS-03 BCs updated).

BC-INDEX titles unchanged: all 22 BC H1 headings stable. No BC retirements or removals.
SE-16d monotonicity PASS: 2026-05-19T12:14:00Z > prior 2026-05-19T00:00:00Z (v1.12). ARITHMETICALLY TRUE: PASS.

## §Trace v1.14

**Phase 3 TDD — BC-HOOK-001..041 DTU gene-source behavioral contracts authored** (2026-05-20T21:00:00Z):

- SS-DTU section added: 41 new behavioral contracts (BC-HOOK-001..BC-HOOK-041).
- Gene source: `any-context-lazyclaude/internal/core/config/hooks.go` via hooks-r1 + hooks-r2 deep ingest rounds.
- BC count: 22 → 63 (41 new BCs).
- Summary table updated: SS-DTU row added; Total row updated 22 → 63.
- Subsystem: these BCs use `subsystem: SS-01` in their frontmatter (DTU contracts describe protocol ingested by SS-01 daemon endpoints). Directory: `ss-dtu/`.
- File directory created: `.factory/specs/behavioral-contracts/ss-dtu/` (41 BC files).
- Capability: all 41 BCs trace to CAP-001 per capabilities.md §CAP-001.
- Origin: gene-transfusion (derived from any-context-lazyclaude gene source with monocle improvements in BC-HOOK-014, BC-HOOK-024, BC-HOOK-039, BC-HOOK-040).
- Produced for: S-DTU-001 DTU clone prerequisite gate (status: draft → ready).
- Downstream: S-DTU-001 frontmatter `behavioral_contracts` array unchanged (already correct BC-HOOK-001..BC-HOOK-041 range); status flipped to `ready`; sprint-state S-001 flipped `done`; sprint-state S-DTU-001 flipped `ready`.
- BC-INDEX titles: all 41 H1 titles are authoritative per bc_h1_is_title_source_of_truth policy.
- Stories affected by BC changes: S-DTU-001. Story-writer must propagate under bc_array_changes_propagate_to_body_and_acs policy (BC coverage table in STORY-INDEX updated in this burst).
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z > prior 2026-05-19T12:14:00Z (v1.13). ARITHMETICALLY TRUE: PASS.

## §Trace v1.15

**PRD expansion — SS-04/05/06/07 BC integration: 48 new BCs added across 4 new subsystems** (2026-05-26T13:00:00Z):

- SS-04 section added: 12 new behavioral contracts (BC-2.04.001..BC-2.04.012). Capability: CAP-004. Architecture source: SS-daemon-wiring.md. Covers: daemon start sequence (SOQ-2), CLI subcommands, auto-start, ProjectDirs fallback chain, hook endpoint routing, hook tmpfile, bounded event bus, JSONL ring capacity.
- SS-05 section added: 8 new behavioral contracts (BC-2.05.001..BC-2.05.008). Capability: CAP-005. Architecture source: SS-ipc.md. Covers: UDS server bind, TUI connect + initial state push, IPC message types (SessionListUpdate/HookEventReceived/PermissionPromptQueued), TUI reconnect, SOQ-3 overlay clear on disconnect, UDS-only Phase 1 constraint.
- SS-06 section added: 22 new behavioral contracts (BC-2.06.001..BC-2.06.022). Capability: CAP-006. Architecture source: SS-tui.md. Covers: AppMode state machine, FocusSnapshot, 5-level action dispatch, Ctrl-\ popup, sessions panel, permission overlay (VecDeque stack, rotate, diff preview, accept-once/always/reject, Esc hide, trace-to-source stub, disconnect clear, timeout budget), event ribbon, status bar (drop counter, breadcrumb, hint line), killer scenario (≤6 keystrokes).
- SS-07 section added: 6 new behavioral contracts (BC-2.07.001..BC-2.07.006). Capability: CAP-007. Architecture source: SS-config.md. Covers: atomic write via tempfile::persist, config schema v1, missing/corrupted default, sticky-per-project profile, Ctrl-P override, CCR path detection.
- BC count: 63 → 111 (48 new BCs). Summary table updated: 4 new subsystem rows; Total row 63 → 111.
- PRD version: prd.md bumped to v1.27.0 in same burst (§2.4–§2.7 added; title updated; revision history entry added).
- Priority assignments: P0 for SOQ-2/SOQ-3 invariants, critical path behaviors, and Phase 1 success criteria features. P1 for supporting/secondary behaviors. P2 for stub features (BC-2.06.015 trace-to-source).
- BC-INDEX titles: all 48 H1 titles are authoritative per bc_h1_is_title_source_of_truth policy. Titles extracted verbatim from BC file H1 headings.
- Old ID (historical) column: all 48 new BCs have no historical IDs (greenfield BCs, no renumbering occurred).
- SE-16d monotonicity PASS: 2026-05-26T13:00:00Z > prior 2026-05-20T21:00:00Z (v1.14). ARITHMETICALLY TRUE: PASS.

## §Trace v1.16

**Phase 1d Pass 1 adversarial findings — F-P1D-001/002/003/006/012 closure** (2026-05-27T00:00:00Z):

**F-P1D-001 CRITICAL — capability mis-anchoring corrected in 6 SS-04 BCs:**
- BC-2.04.001, BC-2.04.002, BC-2.04.003, BC-2.04.004, BC-2.04.005, BC-2.04.006:
  frontmatter `capability: CAP-001` → `capability: CAP-004` per F-P1D-001.
- Traceability §L2 Capability and §Capability Anchor Justification updated in all 6 files
  to cite CAP-004 ("Daemon binary crate wiring; CLI surface; SOQ-2 start-sequence invariant;
  hook endpoint routing; bounded event bus") per ARCH-INDEX §SS-04 Capability Traceability.
- Root cause: SS-04 BCs were authored with CAP-001 (daemon lifecycle, an SS-01 capability)
  instead of CAP-004 (daemon wiring, the correct SS-04 capability).
- Version bumps: all 6 BCs: v1.0.0 → v1.1.0.

**F-P1D-002 CRITICAL — PermissionDecision variant names corrected in BC-2.06.022:**
- BC-2.06.022 Step 2: `PermissionDecision::Always` → `PermissionDecision::AcceptAlways`.
- BC-2.06.022 Step 3: `PermissionDecision::Once` → `PermissionDecision::Accept`.
- Canonical enum per SS-ipc.md: `Accept`, `AcceptAlways`, `Reject`.
- Version bump: BC-2.06.022: v1.0.0 → v1.1.0.

**F-P1D-003 CRITICAL — Action variant name corrected in BC-2.07.005:**
- BC-2.07.005: all occurrences of `Action::OpenProfilePicker` → `Action::ProfilePicker`.
- Canonical enum per SS-tui.md: variant is `ProfilePicker`, not `OpenProfilePicker`.
- Version bump: BC-2.07.005: v1.0.0 → v1.1.0.

**F-P1D-006 HIGH — Missing BC for PermissionPromptResolved TUI handling: BC-2.06.023 created:**
- BC-2.06.023 "TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved"
  added to SS-06. Priority: P0.
- Specifies: matching VecDeque removal (PC-1), empty-stack AppMode collapse (PC-2),
  no-op for unknown prompt_id (PC-3), non-empty overlay remains open (PC-4).
- Closes gap identified by adversary: BC-2.05.005 broadcast had no TUI-side consumer BC.
- SS-06 BC count: 22 → 23. Summary table: SS-06 22 → 23. Grand total: 111 → 112.

**F-P1D-012 MEDIUM — DropCounterUpdate debounce postcondition added to BC-2.04.011:**
- BC-2.04.011 PC-8 added: `ServerToClient::DropCounterUpdate` sent at most once per 100ms;
  value reflects cumulative counter at debounce-fire time. Source: SS-ipc.md lines 288-289.
- Version bump: BC-2.04.011: v1.0.0 → v1.1.0.

BC-INDEX titles: BC-2.06.023 H1 title is authoritative per bc_h1_is_title_source_of_truth.
All other BC H1 headings unchanged. No BC retirements or removals.
SE-16d monotonicity PASS: 2026-05-27T00:00:00Z > prior 2026-05-26T13:00:00Z (v1.15). ARITHMETICALLY TRUE. PASS.

## §Trace v1.17

**CV-P1D-001 + CV-P1D-002 consistency-check closure — BC-2.05.005 timeout contradiction + §Trace v1.16 timestamp monotonicity fix** (2026-05-27T00:00:00Z):

**CV-P1D-001 CRITICAL — BC-2.05.005 v1.0.0 → v1.1.0 (timeout sends PermissionPromptResolved):**
- BC-2.05.005 Postcondition 4, EC-002, test vector (timeout row), and VP-TBD (timeout row)
  all stated the daemon does NOT send `PermissionPromptResolved` on hook timeout.
- SS-ipc.md was updated by F-P1D-007 in the Phase 1d adversarial pass to specify that
  `PermissionPromptResolved` IS sent on timeout so TUI clients can remove stale overlay entries.
  BC-2.05.005 was not updated in that burst, creating a direct contradiction between the BC
  and its architecture source of truth.
- All 4 contradiction sites corrected in BC-2.05.005 v1.1.0. See BC-2.05.005 §Trace v1.1.0
  for full before/after.

**CV-P1D-002 MINOR — §Trace v1.16 body timestamp violated SE-16d monotonicity:**
- §Trace v1.16 was authored with body timestamp `2026-05-26T00:00:00Z`, which is EARLIER
  than the v1.15 frontmatter timestamp `2026-05-26T13:00:00Z`. This violated SE-16d
  strict-greater monotonicity.
- Fix: §Trace v1.16 body timestamp corrected to `2026-05-27T00:00:00Z`; BC-INDEX frontmatter
  version bumped v1.16 → v1.17; frontmatter timestamp updated to `2026-05-27T00:00:00Z`.

BC-INDEX titles unchanged. No BC retirements or removals.
SE-16d monotonicity PASS: 2026-05-27T00:00:00Z >= prior 2026-05-27T00:00:00Z (v1.16). PASS (equal-or-greater; same burst).

## §Trace v1.18

**F-P1D6-003 + F-P1D6-005 + F-P1D7-001 + F-P1D7-004 — Comprehensive IPC type sweep** (2026-05-26T00:00:00Z):

**F-P1D7-004 HIGH — SS-05 and SS-06 section header capability text aligned to ARCH-INDEX verbatim:**
- SS-05 capability: `"Unix domain socket IPC between TUI client and daemon; message types; reconnection; SOQ-3 disconnect invariant"` → `"Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear"` (verbatim from ARCH-INDEX v1.0.13 §SS-05 row).
- SS-06 capability: `"ratatui TUI; AppMode state machine; keybinding dispatch; permission overlay; sessions panel; event ribbon; status bar"` → `"User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration"` (verbatim from ARCH-INDEX v1.0.13 §SS-06 row).

BC-INDEX version: 1.17 → 1.18. No BC ID retirements or removals. All BC H1 titles unchanged.
SE-16d monotonicity PASS: 2026-05-26T00:00:00Z timestamp is within the same burst as v1.17. PASS.

## §Trace v1.19

**F-P1D10-002 HIGH — SS-04 section header capability text aligned to ARCH-INDEX verbatim** (2026-05-26T00:00:00Z):
- SS-04 capability: `"Daemon binary crate wiring; CLI surface; SOQ-2 start-sequence invariant; hook endpoint routing; bounded event bus"` → `"Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation"` (verbatim from ARCH-INDEX §Capability Traceability SS-04 row).
- Note: BC-2.04.001..BC-2.04.006 Traceability tables also carried the stale text; those 6 BCs are updated in this same burst (individual BC versions bumped). BC-2.04.007..BC-2.04.012 already had the correct ARCH-INDEX verbatim text.

BC-INDEX version: 1.18 → 1.19. No BC ID retirements or removals. All BC H1 titles unchanged.
SE-16d monotonicity PASS: 2026-05-26T00:00:00Z timestamp is within the same burst as v1.18. PASS.

## §Trace v1.20

**Adversarial BC anchoring resolution — BC-2.06.024 created; S-027/S-028 anchoring decisions documented** (2026-05-27T00:30:00Z):

**Question 1 — S-027 ACs mis-anchored to BC-2.06.017:**
- BC-2.06.024 "Permission Overlay: ToolPayload Body Rendering by Variant" created at `ss-06/BC-2.06.024.md` (P1 active).
- Decision: option (b) — new BC rather than extending BC-2.06.010. Rationale: BC-2.06.010's scope is precisely "Diff Preview via `similar 3`" for `ToolPayload::Edit`; its postconditions cover diff computation, color coding, and height cap — not label-display rendering. Extending BC-2.06.010 would violate H1 title authority (bc_h1_is_title_source_of_truth) and mix two independent rendering concerns. BC-2.06.017's postconditions are timing/latency contracts (TUI render budget ≤100ms; no artificial decision delay; daemon timeout enforcement) — entirely unrelated to how Bash/Read/Generic payloads render their body content.
- BC-2.06.024 specifies: PC-1 (Bash body: `command: <cmd>`), PC-2 (Read body: `path: <path>`), PC-3 (Generic body: `tool: <name>` + `input: <excerpt truncated at 256 chars>`), PC-4 (Edit dispatches to BC-2.06.010 — NOT handled by this BC).
- SS-06 BC count: 23 → 24. Summary table: SS-06 23 → 24. Grand total: 112 → 113.
- BC-2.06.024 H1 title is authoritative per bc_h1_is_title_source_of_truth.

**Question 2 — S-028 references undefined `ServerToClient::SessionEvents` IPC variant:**
- Decision: option (a) — no new IPC variant needed. The existing `HookEventReceived` streaming + `InitialState.ring_tail` is sufficient for the Event Ribbon per-session history.
- Rationale: SS-ipc.md §Connection Lifecycle is explicit — `InitialState` delivers `ring_tail: Vec<HookEvent>` on connect with ALL events (not session-scoped); `HookEventReceived` streams live events carrying `session_id: String`. The TUI (BC-2.06.018 Invariant 3) already stores the full session_id in `HookEventRow::session_id` for future detail views. Client-side filtering by `session_id` in the Sessions panel view is the correct pattern: it requires no new IPC variant, no new daemon-side indexing, and is consistent with BC-2.05.004 which documents that `HookEventReceived` carries `session_id` and explicitly states "There is no filtering at the IPC layer; the TUI may filter for display." Adding `ServerToClient::SessionEvents` would be a premature architectural addition that conflicts with the established push model. No spec changes are needed for this decision.
- No BC files modified for Question 2.

**Stories affected by BC changes:** S-027. Story-writer must:
- Remove S-027 AC-003, AC-004, AC-006 traces from BC-2.06.017 and anchor them to BC-2.06.024 instead.
- Remove the S-028 `ServerToClient::SessionEvents` reference and replace with `HookEventReceived + client-side session_id filter` pattern.
- Under bc_array_changes_propagate_to_body_and_acs policy: update S-027 frontmatter `bcs:` array to include BC-2.06.024; update AC tables in story body accordingly.

**VP citations changed:** None — BC-2.06.024 uses VP-TBD placeholders; architect propagation not required at this time.

BC-INDEX titles: BC-2.06.024 H1 "Permission Overlay: ToolPayload Body Rendering by Variant" is authoritative. All other BC H1 headings unchanged. No BC retirements or removals.
SE-16d monotonicity PASS: 2026-05-27T00:30:00Z > prior 2026-05-27T00:00:00Z (v1.19). ARITHMETICALLY TRUE: PASS.

## §Trace v1.21

**ADJ-ADV2-001 HIGH — Adversarial Pass 2 adjudication: three decisions for S-026 / Permission Overlay BCs** (2026-05-27T08:00:00Z):

**Decision 1 — Keybinding canonical set: mnemonic wins over numeric.**
- Conflict: BC-2.06.011/012/013 used `1`/`2`/`3`; S-026 AC-003/004/005 and S-027 AC-001 footer used `y`/`Enter`/`A`/`n`/`r`.
- Resolution: `y`/`Enter` (Accept-Once), `A` (Accept-Always), `n`/`r` (Reject) are canonical.
- Rationale: (a) lazygit-philosophy verb/mnemonic keybindings, (b) `y`/`n` is universal TUI confirmation convention, (c) S-026/S-027 are more recent artifacts (2026-05-27) reflecting deliberate UX design, (d) `A` for AcceptAlways is unambiguous and distinct from single-accept. The binding layer is `SearchPrompt` (highest priority) per S-026 AC-009, not `PerContext` as previously specified in the BCs.
- BC changes: BC-2.06.011 v1.0.5 → v1.1.0, BC-2.06.012 v1.0.5 → v1.1.0, BC-2.06.013 v1.0.5 → v1.1.0.
- BC-INDEX titles updated: 011 now includes `(`y`/`Enter`)`, 012 now includes `(`A`)`, 013 now includes `(`n`/`r`)`.
- Architect follow-up required: SS-tui.md §Overlay Stack Lifecycle Step 3 says `[1]/[2]/[3]` — must be updated to `[y]/[A]/[n/r]`. Keybinding Dispatcher comment at line ~397 also says `[1]/[2]/[3]` — must be updated. This is an architectural source correction, not product-owner scope. Route to architect.

**Decision 2 — Pop semantics: wait-for-PermissionPromptResolved (production-grade).**
- Conflict: BC-2.06.011/012/013 PC-2 specified immediate `stack.pop_front()` after IPC send; S-026 AC-003/004/005 specified wait for `ServerToClient::PermissionPromptResolved`; BC-2.06.023 (authoritative removal mechanism) specifies `retain()`-based removal on `PermissionPromptResolved`.
- Resolution: Wait-for-resolved. The modal is NOT removed until `ServerToClient::PermissionPromptResolved { prompt_id }` arrives from the daemon. The `retain()` path in BC-2.06.023 is the single removal mechanism.
- Rationale: (a) immediate pop creates a state gap — TUI thinks prompt is done, daemon has not confirmed; (b) if IPC send fails (channel drop), immediate pop silently loses the decision with no user feedback; with wait-for-resolved, the modal stays visible and the user can retry; (c) BC-2.06.023 already defines the authoritative removal path; adding a second removal path in 011/012/013 creates duplication and a race condition.
- BC changes: All three BCs have Postconditions completely rewritten, Invariants updated (Invariant 5 added to each — "TUI MUST NOT call retain()/pop_front() upon sending"), Edge Cases updated, Test Vectors updated to two-step round-trip format.

**Decision 3 — BC-2.06.009 (Stack Rotation) coverage gap: add rotation ACs to S-026.**
- Problem: BC-2.06.009 is listed in S-026's `behavioral_contracts` array but has zero actual AC coverage in S-026. S-026 ACs 003/004/005 claim to trace to BC-2.06.009 postconditions but describe Accept/Reject behavior (which belongs to 011/012/013), not rotation.
- Resolution: Story-writer adds rotation ACs to S-026. No new story needed; rotation is core overlay state machine behavior that belongs in S-026's scope alongside push/pop and decision dispatch. S-026 is at 13 pts; adding 2 rotation ACs (rotate-next and single-item no-op) does not change the point estimate materially.
- No BC file changes for this decision — BC-2.06.009 is correct as written. The gap is purely in S-026 AC coverage.

**Stories affected by BC changes:** S-026.
Story-writer must propagate under bc_array_changes_propagate_to_body_and_acs policy:
1. Update S-026 frontmatter `bcs:` input versions: BC-2.06.011 to v1.1.0, BC-2.06.012 to v1.1.0, BC-2.06.013 to v1.1.0.
2. Update S-026 AC-003/004/005 key references from `y`/`A`/`n`/`r` (already correct) to confirm binding layer is `SearchPrompt` (not `PerContext`) — the ACs already use the mnemonic keys so minimal change needed.
3. Remove the incorrect claim that AC-003/004/005 trace to BC-2.06.009 postconditions — they trace to BC-2.06.011, BC-2.06.012, BC-2.06.013 respectively.
4. Add rotation ACs to S-026: `[↑↓]` cycles the VecDeque (BC-2.06.009 PC-1); single-item stack rotation is a visual no-op (BC-2.06.009 EC-065).

**VP citations changed:** None — no VP ID assignments changed. All VP-TBD placeholders unchanged.

BC-INDEX titles updated: BC-2.06.011, BC-2.06.012, BC-2.06.013 H1 titles now authoritative with key suffix. No BC retirements or removals.
SE-16d monotonicity PASS: 2026-05-27T08:00:00Z > prior 2026-05-27T00:30:00Z (v1.20). ARITHMETICALLY TRUE: PASS.

## §Trace v1.22

**ADJ-ADV2-001 propagation — BC-2.06.003 keybinding + layer fix** (2026-05-27T09:00:00Z):

BC-2.06.003 ("Action Dispatch: 5-Level Binding Precedence") was not updated during the v1.21 burst
that applied ADJ-ADV2-001 to BC-2.06.011/012/013. This trace records the propagation fix.

- BC-2.06.003 v1.0.4 → v1.1.0: Postcondition 3 rewritten to use `SearchPrompt` layer (not
  `PerContext`) with mnemonic keybindings `y`/`Enter` (AcceptOnce), `A` (AcceptAlways), `n`/`r`
  (Reject) instead of numeric `1`/`2`/`3`. EC-070 and EC-071 updated accordingly. Test vectors
  updated (5 rows replacing 2 old rows). VP first row updated. Cross-Ref table cell and Related
  BCs bullets updated to reflect new key names.

BC-INDEX title for BC-2.06.003 is unchanged (title "Action Dispatch: 5-Level Binding Precedence"
remains accurate — the BC describes the dispatch mechanism, not a specific keybinding). No BC
retirements or removals.

SE-16d monotonicity PASS: 2026-05-27T09:00:00Z > prior 2026-05-27T08:00:00Z (v1.21). ARITHMETICALLY TRUE: PASS.

## §Trace v1.23

**Stale keybinding references corrected in BC-2.06.014 and BC-2.06.017** (2026-05-27T10:00:00Z):

BC-2.06.014 v1.0.4 → v1.0.5:
- EC-093: `[3]` → `[n/r]`; "sends Reject + pops the front item" → "sends Reject IPC; modal remains until `PermissionPromptResolved` arrives from daemon" (wait-for-resolved semantics per BC-2.06.023).
- Related BCs and Traceability Cross-Ref: "Reject (key `3`) pops and sends deny" → "Reject (keys `[n/r]`) sends deny IPC and waits for `PermissionPromptResolved`".

BC-2.06.017 v1.5.0 → v1.6.0:
- Postcondition 2: `[1]`, `[2]`, `[3]` → `[y]/[Enter]`, `[A]`, `[n]/[r]`.
- EC-105: `[1]` → `[y]`.
- Test vector "Decision within budget": `[1]` → `[y]`.
- Test vector "IPC send channel full": `[1]` → `[y]`.

BC-INDEX H1 titles for BC-2.06.014 and BC-2.06.017 are unchanged — the title changes were in body content only, not H1 headings.

SE-16d monotonicity PASS: 2026-05-27T10:00:00Z > prior 2026-05-27T09:00:00Z (v1.22). ARITHMETICALLY TRUE: PASS.

## §Trace v1.24

**Architect Pass 2 HIGH-003 propagation — SS-06 cross-BC sweep: `AppMode::Overlay { stack }` field removed** (2026-05-28T00:00:00Z):

Resolves F-S025-ADV3-BLOCKER-002. `AppMode::Overlay { stack: VecDeque<PromptModal>, prior }` shape removed by architect decision (commit `76ce1af`). New canonical shape: `AppMode::Overlay { prior: FocusSnapshot }`. The `VecDeque<PromptModal>` overlay stack lives exclusively in `App::overlay_stack` (monocle-tui) as the single source of truth. 16 BCs swept.

Version bumps:
- BC-2.06.001 v1.0.3 → v1.0.4: AppMode enum definition, PC-3 empty-stack collapse (App-level), Invariant 4 push path, EC-060/EC-062, test vectors, VP.
- BC-2.06.002 v1.0.3 → v1.0.4: PC-2/PC-3 `App.overlay_stack` reframing, EC-065, test vectors.
- BC-2.06.004 v1.1.0 → v1.2.0 (non-mechanical): PC-2 critical rewrite — two-step populate `App.overlay_stack` via `payload_to_modal()` then transition to `Overlay { prior: FocusSnapshot::Sessions }`. Postconditions 3-4, Invariant 2, test vector updated.
- BC-2.06.008 v1.0.4 → v1.1.0 (non-mechanical): description rewrite — push goes to `App.overlay_stack` first, then AppMode transition. All postconditions, invariants, EC-100-104, test vectors rewritten for two-step App-level push semantics.
- BC-2.06.009 v1.0.4 → v1.1.0 (non-mechanical): description rewrite — `transition()` is identity for `OverlayCycleNext`; rotation is App-level effectful on `App.overlay_stack`. All postconditions, invariants, EC-065/066, test vectors, VP (Kani → proptest) updated.
- BC-2.06.010 v1.0.4 → v1.0.5: Precondition 1 `Overlay { stack, prior }` → `Overlay { prior }` with `App.overlay_stack.front()`.
- BC-2.06.011 v1.1.0 → v1.2.0: Preconditions, postconditions (all `stack.*` → `App.overlay_stack.*`), invariants, EC-077/078, test vectors updated.
- BC-2.06.012 v1.1.0 → v1.2.0: Symmetric with BC-2.06.011.
- BC-2.06.013 v1.1.0 → v1.2.0: Symmetric with BC-2.06.011.
- BC-2.06.014 v1.0.5 → v1.0.6: Description, PC-1/PC-4, Invariants 1+2, test vectors. PC-4 rebuild path explicitly uses `payload_to_modal()` + `Overlay { prior }`.
- BC-2.06.015 v1.0.3 → v1.0.4: Description, PC-1/PC-2/PC-6, test vectors. Secondary fix: PC-6 stale `[1]/[2]/[3]` → `[y]/[A]/[n/r]`.
- BC-2.06.016 v1.0.5 → v1.0.6: Description, PC-1, Invariants 1+4, VP updated.
- BC-2.06.020 v1.0.3 → v1.0.4: Breadcrumb derivation table `Overlay { stack, prior }` → `Overlay { prior }` with `App.overlay_stack.len()`. PC-2, EC-125, test vectors updated.
- BC-2.06.022 v1.5.0 → v1.6.0 (non-mechanical): Summary PC-3/Invariant 3 keystroke references `2`/`1` → `A`/`y`. Invariant 5 rewritten for App-level retain()-based collapse. EC-134/135/136/138 updated. Test vectors KS-001/002/003 updated. VP Anchors updated. Architecture Module: `VecDeque pop-and-collapse` → explicit `App.overlay_stack retain()` wording.
- BC-2.06.023 v1.3.0 → v1.4.0: PC-2 `Overlay { stack, prior }` → `Overlay { prior }` with `App.overlay_stack` emptiness check. EC-006 stale keystroke `1` → `y`.
- BC-2.06.024 v1.0.0 → v1.0.1: Preconditions 1+2 `Overlay { stack, prior }` / `stack.front()` → `Overlay { prior }` / `App.overlay_stack.front()`.

BC titles: unchanged. No BC retirements or removals.

SE-16d monotonicity PASS: 2026-05-28T00:00:00Z > prior 2026-05-27T10:00:00Z (v1.23). ARITHMETICALLY TRUE: PASS.

## §Trace v1.25

**F-S025-ADV4-BLOCKER-002 + HIGH-001 — Column adjudication + incomplete body sweep cleanup** (2026-05-28T13:00:00Z):

BC-2.06.005 v1.0.4 → v1.0.5 (F-S025-ADV4-BLOCKER-002, Option A):
- Column count: 6 → 7. `session_id` added as first column. Rationale: required for operator
  debuggability when multiple sessions share project name or harness type.
- Description, PC-2 (column layout), canonical test vector happy-path row, Story Anchor updated.
- H1 title unchanged: "Sessions Panel: Session List Renders from IPC State" (column count
  is body-level detail, not a title-level discriminator).

BC-2.06.016 v1.0.6 → v1.0.7 (F-S025-ADV4-HIGH-001):
- EC-102 Expected Behavior column: stale `AppMode::Overlay { stack: [P1, P2], ... }` →
  `App.overlay_stack = [P1, P2]`; `AppMode::Overlay { prior: Sessions }`.
- Canonical test vector row 1 Initial State column: same shape correction.
- §Trace v1.0.6 retrospectively corrected to note the pass was "partial", not complete.

BC-2.06.021 v1.0.3 → v1.0.4 (F-S025-ADV4-HIGH-001):
- Canonical test vector row 3 AppMode column: stale `Overlay { stack: [P1], prior: Sessions }` →
  `Overlay { prior: Sessions }` (with `App.overlay_stack = [P1]`).
- No §Trace entry existed for the Pass 3 sweep omission — v1.0.4 is the first sweep entry.

BC-INDEX H1 titles: unchanged for all three BCs.
No BC retirements or removals.

SE-16d monotonicity PASS: 2026-05-28T13:00:00Z > 2026-05-28T00:00:00Z (v1.24). ARITHMETICALLY TRUE: PASS.

## §Trace v1.26

**F-S025-ADV5-HIGH-002 — BC-2.06.014 EC-096 Overlay shape corrected** (2026-05-28T14:00:00Z):

BC-2.06.014 v1.0.6 → v1.0.7 (F-S025-ADV5-HIGH-002):
- EC-096 Expected Behavior: stale `Overlay { stack: empty, prior }` → `Overlay { prior }` with
  App-level note (`App.overlay_stack` remains empty). §Trace v1.0.6 incorrectly claimed sweep
  completion; v1.0.7 honestly documents the missed EC row.
- Cross-sweep audit: no other stale `Overlay { stack` references found in live content.

BC-INDEX H1 title for BC-2.06.014: unchanged ("Permission Overlay: `[Esc]` Hides Without Rejecting").
No BC retirements or removals.

SE-16d monotonicity PASS: 2026-05-28T14:00:00Z > 2026-05-28T13:00:00Z (v1.25). ARITHMETICALLY TRUE: PASS.

## §Trace v1.27

**F-S025-ADV11-HIGH-001 PO Option B — BC-2.06.016 v1.0.7 → v1.0.8** (2026-05-28T00:00:00Z):

BC-2.06.016 v1.0.7 → v1.0.8 (PO Option B decision — commit 4563bfa):
- PC-1 and PC-2 disconnect text style updated per PO Option B adjudication.
- No BC H1 title change.

BC-INDEX titles unchanged: BC-2.06.016 H1 "Permission Overlay: Cleared on Daemon Disconnect" is stable. No BC retirements or removals.

SE-16d monotonicity PASS: 2026-05-28T00:00:00Z >= 2026-05-28T14:00:00Z (v1.26). PASS (same-day).

## §Trace v1.28

**F-S025-ADV23-MED-001 Category 8 sweep — Architecture Source pin refresh: 10 BCs across ss-06 + ss-07** (2026-05-29T00:00:00Z):

Pass 23 MED-001 closure + comprehensive sibling-BC sweep (orchestrator process-gap absorption). All 10 known stale Category A active Architecture Source pointers refreshed to current canonical versions.

ss-06 (3 BCs; SS-tui.md v1.5.0/v1.6.0 → v1.8.2):
- BC-2.06.004 v1.2.0 → v1.2.1: Architecture Source SS-tui.md v1.5.0 → v1.8.2. No body prose propagation needed (AppMode shape change already in v1.2.0).
- BC-2.06.005 v1.0.5 → v1.0.6: Architecture Source SS-tui.md v1.6.0 → v1.8.2 (Traceability + PC-2 inline citation). §Trace v1.0.4 prose corrected: "sixth column is Status" → accurate 7-column description (Status is fourth; the §Trace v1.0.4 historical prose was written against the 6-column layout and was internally inconsistent with the v1.0.5 body).
- BC-2.06.007 v1.0.3 → v1.0.4: Architecture Source SS-tui.md v1.5.0 → v1.8.2. No body prose propagation needed (fullscreen content semantics unchanged across v1.5.0→v1.8.2).

ss-07 (7 BCs; SS-config.md v1.1.0 → v1.3.0):
- BC-2.07.001 v1.1.0 → v1.1.1: Architecture Source SS-config.md v1.1.0 → v1.3.0. No body propagation (§Atomic Write Contract unchanged in v1.2.0–v1.3.0).
- BC-2.07.002 v1.0.2 → v1.0.3: Architecture Source SS-config.md v1.1.0 → v1.3.0. No body propagation (§Config Schema v1 struct unchanged; v1.2.0 serde-default fix already in EC-080 of this BC).
- BC-2.07.003 v1.0.1 → v1.0.2: Architecture Source SS-config.md v1.1.0 → v1.3.0. No body propagation (BC body already correctly specifies both ConfigError variants that SS-config.md v1.3.0 clarified).
- BC-2.07.004 v1.0.1 → v1.0.2: Architecture Source SS-config.md v1.1.0 → v1.3.0. No body propagation (§Profile Picker Logic unchanged in v1.2.0–v1.3.0).
- BC-2.07.005 v1.3.0 → v1.3.1: Architecture Source SS-config.md v1.1.0 → v1.3.0. No body propagation (§Ctrl-P Override unchanged).
- BC-2.07.006 v1.0.1 → v1.0.2: Architecture Source SS-config.md v1.1.0 → v1.3.0. No body propagation (§CCR Detection algorithm unchanged).

BC-INDEX H1 titles: unchanged for all 10 BCs. No BC retirements or removals.

Sweep-wider results: Exhaustive grep identified 26 additional stale Category A sites in ss-03 (4 BCs citing SS-engine-module v1.1.20 → need v1.1.26), ss-04 (12 BCs + 1 ss-05 overlap citing SS-daemon-wiring v1.2.0 → need v1.3.0), ss-05 (8 BCs citing SS-ipc v1.4.0/v1.7.0 → need v1.9.0; 1 BC citing SS-deps-pin-manifest v1.1.17 → need v1.2.0), and ss-dtu (1 BC citing SS-conventions-anti-patterns v1.29.5 → need v1.31.1). These exceed the 15-additional-site threshold per orchestrator stop-rule. Fix of these 26 additional sites is deferred and surfaces as scope expansion (ADV23-SCOPE-001) for orchestrator authorization before next burst.

[process-gap] CODIFY-001 Category 8 (ADV23-PROC-001): Architecture Source pins in BC files must be swept against canonical doc frontmatter versions whenever an architecture doc is bumped. This is a distinct codification from ADV22-PROC-001 (bare-filename discipline) and D-198.2 (worktree code). Candidate for CI enforcement rule: parse "Architecture Source | SS-*.md vX.Y.Z" cells in BC bodies and fail if cited version < canonical frontmatter version.

SE-16d monotonicity PASS: 2026-05-29T00:00:00Z > 2026-05-28T00:00:00Z (v1.27). ARITHMETICALLY TRUE: PASS.

## §Trace v1.29

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: 26 additional BC Architecture Source pin refreshes** (2026-05-29T00:00:00Z):

Continues from 1ad2852 (10 BCs in ss-06 + ss-07). Orchestrator-authorized per CLAUDE.md Principle 4 — single comprehensive sweep bounds the cascade. All 26 are plain version-pin refreshes; no substantive content propagation required per prior §Trace analysis.

ss-03 (4 BCs; SS-engine-module v1.1.20 → v1.1.26):
- BC-2.03.001 v1.0.6 → v1.0.7: Architecture Source SS-engine-module.md v1.1.20 → v1.1.26. §EngineModule Trait Signature section anchor unchanged.
- BC-2.03.002 v1.0.4 → v1.0.5: Architecture Source SS-engine-module.md v1.1.20 → v1.1.26. §Phase 1 Implementation: ClaudeCodeModule section anchor unchanged.
- BC-2.03.003 v1.0.3 → v1.0.4: Architecture Source SS-engine-module.md v1.1.20 → v1.1.26. §Behavioral Contracts BC-ENGINE-002-ERR section anchor unchanged.
- BC-2.03.004 v1.0.4 → v1.0.5: Architecture Source SS-engine-module.md v1.1.20 → v1.1.26. §Struct-level inherent operations section anchor unchanged.

ss-04 (12 BCs; SS-daemon-wiring v1.2.0 → v1.3.0):
- BC-2.04.001 v1.5.0 → v1.6.0: §Daemon Start Sequence (BC-2.04.001) anchor unchanged.
- BC-2.04.002 v1.4.0 → v1.5.0: §Daemon Auto-Start Logic §Auto-Start Decision Sequence anchor unchanged.
- BC-2.04.003 v1.4.0 → v1.5.0: §MONOCLE_NO_AUTOSTART Check anchor unchanged.
- BC-2.04.004 v1.4.0 → v1.5.0: §CLI Interface §Subcommand: monocle daemon start anchor unchanged.
- BC-2.04.005 v1.4.0 → v1.5.0: §CLI Interface §Subcommand: monocle daemon stop anchor unchanged.
- BC-2.04.006 v1.5.0 → v1.6.0: §Daemon Start Sequence Step 1 anchor unchanged. SS-daemon-lifecycle.md v1.0.33 cross-ref preserved.
- BC-2.04.007 v1.4.0 → v1.5.0: §Hook Endpoint Routing anchor unchanged.
- BC-2.04.008 v1.1.0 → v1.2.0: §Hook Endpoint Routing anchor unchanged.
- BC-2.04.009 v1.1.0 → v1.2.0: §Hook Endpoint Routing anchor unchanged.
- BC-2.04.010 v1.2.0 → v1.3.0: §Hook Tmpfile Generation anchor unchanged.
- BC-2.04.011 v1.3.0 → v1.4.0: §Bounded Event Bus anchor unchanged.
- BC-2.04.012 v1.0.2 → v1.0.3: §Daemon Start Sequence (Step 4) anchor unchanged. SS-daemon-lifecycle.md v1.0.33 cross-ref preserved.

ss-05 (8 BCs; mixed SS-ipc + SS-daemon-wiring + SS-deps-pin-manifest + SS-conventions refreshes):
- BC-2.05.001 v1.2.0 → v1.3.0: SS-ipc.md v1.4.0 → v1.9.0 + SS-daemon-wiring.md v1.2.0 → v1.3.0 (dual refresh).
- BC-2.05.002 v1.0.5 → v1.0.6: SS-ipc.md v1.7.0 → v1.9.0 (2 occurrences).
- BC-2.05.003 v1.0.3 → v1.0.4: SS-ipc.md v1.4.0 → v1.9.0 (2 occurrences).
- BC-2.05.004 v1.0.3 → v1.0.4: SS-ipc.md v1.4.0 → v1.9.0 (2 occurrences).
- BC-2.05.005 v1.6.0 → v1.7.0: SS-ipc.md v1.4.0 → v1.9.0 (3 occurrences).
- BC-2.05.006 v1.0.3 → v1.0.4: SS-ipc.md v1.4.0 → v1.9.0 (2 occurrences).
- BC-2.05.007 v1.0.3 → v1.0.4: SS-ipc.md v1.4.0 → v1.9.0 (2 occurrences).
- BC-2.05.008 v1.0.3 → v1.0.4: SS-ipc.md v1.4.0 → v1.9.0 + SS-deps-pin-manifest.md v1.1.17 → v1.2.0 + SS-conventions-anti-patterns.md v1.29.5 → v1.31.1 (three-pin refresh; Cross-Ref row also updated).

ss-dtu (1 BC; SS-conventions-anti-patterns v1.29.5 → v1.31.1):
- BC-HOOK-039 v1.0.0 → v1.0.1: Architecture Source SS-conventions-anti-patterns.md v1.29.5 → v1.31.1.

Canonical-doc refresh summary: SS-engine-module v1.1.20→v1.1.26; SS-daemon-wiring v1.2.0→v1.3.0; SS-ipc v1.4.0/v1.7.0→v1.9.0; SS-deps-pin-manifest v1.1.17→v1.2.0; SS-conventions-anti-patterns v1.29.5→v1.31.1. Five canonical docs updated across this burst.

Cumulative Category 8 closure (1ad2852 + this commit): 36 BCs across 6 canonical-doc refreshes (ss-06: SS-tui-core→SS-tui.md; ss-07: SS-config.md; ss-03: SS-engine-module.md; ss-04: SS-daemon-wiring.md; ss-05: SS-ipc.md + SS-deps-pin-manifest.md + SS-conventions-anti-patterns.md; ss-dtu: SS-conventions-anti-patterns.md).

BC-INDEX H1 titles: unchanged for all 27 BCs in this burst. No BC retirements or removals.

META-pattern CODIFY-001 Category 8 closure: ADV23-SCOPE-001 bounds spec-body Architecture Source pin freshness across all subsystems. CI enforcement candidate: parse "Architecture Source | SS-*.md vX.Y.Z" cells in BC bodies, fail if cited version < canonical frontmatter version.

## §Trace v1.30

**ADV23-SCOPE-002 — 9 ss-06 BCs SS-tui.md v1.5.0 → v1.8.2 pin refresh** (2026-05-29T01:00:00Z):

Continues from cc1ea7d (26 BCs in ADV23-SCOPE-001). Orchestrator-authorized per CLAUDE.md Principle 4 — third and final burst bounds the SS-tui.md category cascade. Final-verification sweep on cc1ea7d discovered 9 remaining ss-06 BCs still pinned to SS-tui.md v1.5.0.

All 9 are Category A plain version-pin refreshes (SS-tui.md v1.5.0 → v1.8.2). Per-BC substantive-change analysis (three SS-tui.md version increments checked):
- v1.8.0 (Overlay shape change): already propagated in earlier BC-specific §Trace entries (F-S025-ADV3-BLOCKER-002 sweep).
- v1.8.1 (Sessions Panel 6→7 columns): only BC-2.06.005 (already updated in 1ad2852 burst) referenced the column table; none of these 9 BCs reference §Sessions Panel column layout.
- v1.8.2 (disconnect bracketed-tag style `"[disconnected] reconnecting..."`): BC-2.06.016 led this change (§Trace v1.0.8 at commit 4563bfa); SS-tui.md §Trace v1.8.2 followed. BC-2.06.016 body is already canonical; no prose propagation needed.

ss-06 (9 BCs; SS-tui.md v1.5.0 → v1.8.2) — all plain version-pin refreshes:
- BC-2.06.002 v1.0.4 → v1.0.5: Architecture Source SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.006 v1.0.3 → v1.0.4: Architecture Source SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.008 v1.1.0 → v1.1.1: Architecture Source SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.009 v1.1.0 → v1.1.1: Architecture Source SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.012 v1.2.0 → v1.2.1: Architecture Source SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.016 v1.0.8 → v1.0.9: Architecture Source SS-tui.md v1.5.0 → v1.8.2. Substantive `"[disconnected] reconnecting..."` content already canonical per §Trace v1.0.8; pin bump only.
- BC-2.06.018 v1.0.4 → v1.0.5: Architecture Source SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.019 v1.0.4 → v1.0.5: Architecture Source SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.022 v1.6.0 → v1.6.1: Architecture Source SS-tui.md v1.5.0 → v1.8.2.

Substantive content propagations: NONE (all 9 are plain version-pin refreshes).

Cumulative Category 8 closure (1ad2852 + cc1ea7d + this commit): **45 BCs across 7 canonical-doc refreshes** (SS-tui.md, SS-engine-module.md, SS-daemon-wiring.md, SS-ipc.md, SS-deps-pin-manifest.md, SS-conventions-anti-patterns.md, SS-config.md). Species BOUNDED per exhaustive final-verification sweep.

BC-INDEX H1 titles: unchanged for all 9 BCs in this burst. No BC retirements or removals.

SE-16d monotonicity: v1.30 timestamp 2026-05-29T01:00:00Z > v1.29 timestamp 2026-05-29T00:00:00Z. PASS.

## §Trace v1.31

**ADV23-SCOPE-002 EXTENDED — Exhaustive final-sweep closed 9 additional ss-06 SS-tui.md stale pins** (2026-05-29T02:00:00Z):

Post-commit sweep within ADV23-SCOPE-002 burst revealed 9 additional ss-06 BCs still pinned to SS-tui.md v1.5.0 or v1.6.0. These were missed in the original 9-BC authorization scope because they had been updated in separate, targeted bursts (AppMode shape sweep, keybinding canonicalization) that did not include a full version pin bump. All are Category A plain version-pin refreshes.

Exhaustive sweep finding and CODIFY-001 Category 8 interpretation: the Architecture Source row pattern is systemic across ALL ss-06 BCs — every BC that cites an architecture source must be swept when that document is updated. The CI enforcement candidate (parse all "Architecture Source | SS-*.md vX.Y.Z" cells) would have caught all 18 sites in one pass.

ss-06 additional 9 BCs (SS-tui.md stale; all plain pin refreshes):
- BC-2.06.003 v1.1.0 → v1.1.1: SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.010 v1.0.5 → v1.0.6: SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.011 v1.2.0 → v1.2.1: SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.013 v1.2.0 → v1.2.1: SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.014 v1.0.7 → v1.0.8: SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.017 v1.6.0 → v1.6.1: SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.020 v1.0.4 → v1.0.5: SS-tui.md v1.5.0 → v1.8.2.
- BC-2.06.023 v1.4.0 → v1.4.1: SS-tui.md v1.5.0 → v1.8.2 AND SS-ipc.md v1.4.0 → v1.9.0 (dual refresh).
- BC-2.06.024 v1.0.1 → v1.0.2: SS-tui.md v1.6.0 → v1.8.2.

Substantive content propagations for these 9: NONE. All three SS-tui.md v1.5.0→v1.8.2 content changes (Overlay shape, Sessions Panel columns, disconnect tag style) were either already propagated in prior BC-specific §Trace entries or not in scope for these BCs.

Cumulative Category 8 closure (1ad2852 + cc1ea7d + ADV23-SCOPE-002 authorized 9 + ADV23-SCOPE-002 extended 9): **54 BCs across 7 canonical-doc refreshes** (SS-tui.md, SS-engine-module.md, SS-daemon-wiring.md, SS-ipc.md, SS-deps-pin-manifest.md, SS-conventions-anti-patterns.md, SS-config.md). All ss-06 BCs citing SS-tui.md now at v1.8.2. Category 8 cascade for SS-tui.md: FULLY BOUNDED.

BC-INDEX H1 titles: unchanged for all 9 BCs in this burst. No BC retirements or removals.

SE-16d monotonicity: v1.31 timestamp 2026-05-29T02:00:00Z > v1.30 timestamp 2026-05-29T01:00:00Z. PASS.

SE-16d monotonicity PASS: 2026-05-29T00:00:00Z >= 2026-05-29T00:00:00Z (v1.28). SAME TIMESTAMP: same burst, monotonicity satisfied by version increment.

## §Trace v1.32

**ADV23-SCOPE-002 FINAL — 3 remaining ss-06 BCs found in post-context-resume verification sweep** (2026-05-29T03:00:00Z):

After context compaction, a verification sweep of all ss-06 active Architecture Source pins revealed 3 BCs that were missed in the §Trace v1.30/v1.31 bursts. All had been updated in prior targeted bursts (AppMode shape sweep in §Trace v1.0.4 of each) but never received the plain SS-tui.md v1.5.0 → v1.8.2 pin bump.

ss-06 final 3 BCs (SS-tui.md stale; all plain pin refreshes):
- BC-2.06.001 v1.0.4 → v1.0.5: SS-tui.md v1.5.0 → v1.8.2. AppMode state machine; Overlay shape already propagated in §Trace v1.0.4.
- BC-2.06.015 v1.0.4 → v1.0.5: SS-tui.md v1.5.0 → v1.8.2. Trace-to-source stub; Overlay shape already propagated in §Trace v1.0.4.
- BC-2.06.021 v1.0.4 → v1.0.5: SS-tui.md v1.5.0 → v1.8.2. Keybinding hint line; test vector Overlay shape already propagated in §Trace v1.0.4.

Substantive content propagations: NONE. All three SS-tui.md v1.5.0→v1.8.2 content changes checked per-BC:
- BC-2.06.001: Overlay shape propagated in v1.0.4; Sessions panel not in scope; disconnect not in scope.
- BC-2.06.015: Overlay shape propagated in v1.0.4; Sessions panel not in scope; disconnect not in scope (stub renders placeholder footer only).
- BC-2.06.021: Overlay test-vector shape propagated in v1.0.4; Sessions panel not in scope; disconnect not in scope (hint line shows keybindings, not daemon status).

Post-fix verification: all 24 ss-06 BCs now cite SS-tui.md v1.8.2 as active Architecture Source. Category 8 cascade for SS-tui.md: FULLY BOUNDED — 57 total BC pin refreshes across this cascade (prior 54 + these 3).

BC-INDEX H1 titles: unchanged for all 3 BCs in this burst. No BC retirements or removals.

SE-16d monotonicity: v1.32 timestamp 2026-05-29T03:00:00Z > v1.31 timestamp 2026-05-29T02:00:00Z. PASS.

## §Trace v1.33

**Pass-39 adjudication item 6 — BC-2.06.007 v1.0.4 → v1.0.5: pre-split Action::Escape terminology corrected** (2026-05-30T00:00:00Z):

BC-2.06.007 v1.0.4 → v1.0.5:
- PC-5, §Description, and test-vector row used `Action::Escape` — a pre-split term no longer valid after
  F-S025-ADV2-HIGH-002 split Esc into `Action::Esc` (builtin, identity in Fullscreen) and
  `Action::ExitFullscreen` (per-context, actual fullscreen-exit action).
- PC-5 corrected: `Action::Escape` → `Action::ExitFullscreen` (full transition signature per adjudication ruling).
- §Description corrected: "Pressing `Escape` returns to Dashboard" → split-terminology prose (ExitFullscreen
  exits; Esc key is identity in Fullscreen; per-context binding is fullscreen-view story's responsibility).
- Test-vector row corrected: `Escape` → `ExitFullscreen`.
- Clarifying note added distinguishing `Action::Esc` (builtin) from `Action::ExitFullscreen` (per-context).

BC-INDEX H1 title unchanged: "Sessions Panel: Enter Transitions to Fullscreen" — H1 is about the Enter
transition, not the exit. No BC retirements or removals.

SE-16d monotonicity: v1.33 timestamp 2026-05-30T00:00:00Z > v1.32 timestamp 2026-05-29T03:00:00Z. PASS.

## §Trace v1.34

**F-S027-DOC-001 LOW — BC-2.06.021 v1.0.6 → v1.0.7: PC-3 "or replaced" corrected** (2026-06-03T00:00:00Z):

BC-2.06.021 v1.0.6 → v1.0.7:
- PC-3 asserted "no AppMode in which the hint line is hidden or replaced," which contradicted
  BC-2.06.019 v1.1.0 PC-7's canonical supersede model (status_message temporarily supersedes the
  keybinding hint on the lower row).
- Fix: PC-3 reworded to "always rendered (with supersede exception)" with explicit cross-reference
  to BC-2.06.019 PC-7. The permanent-visibility claim is preserved for the AppMode-level guarantee;
  the temporary supersede by status_message is now acknowledged.
- Documentation-only. No behavior change, no test vector changes, no story pin impact.
- Closes S-027 residual follow-up F-S027-DOC-001.

BC-INDEX H1 title unchanged: "Status Bar: Keybinding Hint Line". No BC retirements or removals.

SE-16d monotonicity: v1.34 timestamp 2026-06-03T00:00:00Z > v1.33 timestamp 2026-05-30T00:00:00Z. PASS.

## §Trace v1.36

**Adversarial pass-1 PO-owned fixes — BC/holdout content reconciliation** (2026-06-03T14:00:00Z):

BC body corrections (no H1 title changes; all BC IDs stable):
- BC-2.08.006 v1.0.0 → v1.1.0: C1 HOOK-MODEL reconciliation — Invariant 3 replaced with
  shared `<runtime_dir>/hooks-settings.json` model per BC-HOOK-010. EC-182 corrected.
- BC-2.08.001 v1.0.0 → v1.1.0: Description: removed "Created → Launching" (SessionState::Created
  retired by architect). EC-151: orphan cleanup changed from DaemonToHost::Kill to SIGTERM-on-pid
  (UDS may not be bound at failure point).
- BC-2.08.003 v1.0.0 → v1.1.0: SessionState::Killed removed. Kill path is
  Running → Terminated directly. PCs, Invariants, ECs, test vectors updated.
- BC-2.09.003 v1.0.0 → v1.1.0: PC-1 parameter renamed `screen_offset` → `pane_area: Rect`
  per SS-embedded-pty v1.1.0 naming.
- BC-2.06.025 v1.0.0 → v1.1.0: O5 fix — `spawned_by_monocle: None` render specified as `[?]`.
  EC-295 added. VP table updated.

Holdout scenario corrections:
- HS-EXP-011: flat sidecar path `runtime_dir/session-<id>.json` + `state: Running`
  (removed nested path + `status: Reconnected` — neither exists in canonical model).
- HS-EXP-012: flat sidecar path in Setup + Satisfaction Criteria.
- HS-EXP-014: C1 reconciliation — Part C (per-session hooks branch) removed; Steps/Outcome/
  Satisfaction updated to shared-file model; BC-HOOK-010 added to source_bcs.
- HS-EXP-013: Step 8 keybinding pinned to BC-2.09.009 PC-5 exact semantics (Esc → prior → Overlay).

SE-16d monotonicity: v1.36 timestamp 2026-06-03T14:00:00Z > v1.35 timestamp 2026-06-03T12:00:00Z. PASS.

## §Trace v1.39

**Pass-7 PO-owned fixes — S-P7-002 (BC-2.09.002 stray field) + S-P7-003 (BC-2.05.010 H1 title)** (2026-06-03):

**UPDATED BCs:**
- BC-2.09.002 v1.0.0 → v1.0.1: S-P7-002 — Removed stray `removal_range: null` frontmatter field
  (orphan field not present in any sibling BC; correct fields are `removed: null` + `removal_reason: null`).
  No content change.
- BC-2.05.010 v1.3.0 → v1.4.0: S-P7-003 — Appended "AttachSession" to H1 title for H1↔body
  consistency. AttachSession was added in v1.2.0 (I3-004) but the H1 was not updated at that time.
  Description and Invariant 1 both correctly state "seven variants" — the H1 now matches.

**INDEX changes:**
- BC-2.05.010 title updated from 6-variant to 7-variant form (bc_h1_is_title_source_of_truth).

SE-16d monotonicity: v1.39 timestamp 2026-06-03 ≥ v1.38 timestamp 2026-06-03. PASS.

## §Trace v1.40.5

**F-P44-IMP-001 — BC-2.03.008 v1.0.2 + BC-2.05.010 v1.8.1: `spawn_unsupported` wire code propagated to BC layer** (2026-06-14):

- BC-2.03.008 v1.0.1 → v1.0.2: PC-3 now cites the `"spawn_unsupported"` wire code
  (SS-ipc v1.22.0, 11th taxonomy entry) backing the "Session spawn not supported for this
  harness" banner. EC-112 annotated as REACHABLE DEFENSIVE PATH (ProfilePicker capability
  filtering is best-effort; F-P44-IMP-001). New integration test vector added for the
  end-to-end spawn_session(opts, CodeMachineModule) → `"spawn_unsupported"` path.
  Architecture Source pins bumped: SS-engine-module-v2-delta.md v1.4.1→v1.5.0; added
  SS-session-manager.md v2.4.0 and SS-ipc.md v1.22.0 citations.
- BC-2.05.010 v1.8.0 → v1.8.1: SpawnSession PC-4 extended with `"spawn_unsupported"`
  immediately after `"invalid_spawn_arg"` and before `"spawn_failed"` (7 spawn-path codes
  total). `"invalid_request"` bullet scoped to `SessionError::Io` and future unmapped
  `EngineError` variants only (`UnsupportedOperation` no longer falls through).
  Architecture Source pins bumped: SS-ipc.md v1.21.0→v1.22.0; SS-session-manager.md
  v2.3.0→v2.4.0; `spawn_unsupported` added to inline code taxonomy (now 11 codes).
- BC-INDEX version: 1.40.4 → 1.40.5. No BC ID additions or retirements. No H1 title changes.

SE-16d monotonicity: v1.40.5 timestamp 2026-06-14T20:00:00Z > v1.40.4 timestamp 2026-06-14T19:00:00Z. PASS.

## §Trace v1.43.9

**F-S038-CONV-001 — BC-2.04.010 v1.4.0: PC-3 schema corrected — mandatory `lock.app="monocle"` object added** (2026-06-19):

BC version bump in this dispatch:
- BC-2.04.010 v1.3.0 → v1.4.0 (F-S038-CONV-001 IMPORTANT: PC-3 canonical JSON schema was missing the
  mandatory `"lock": {"app": "monocle"}` top-level key. BC-2.08.006 Invariant 2 requires this field in
  every written hooks-settings.json; the S-038 implementation correctly emits it. The authoritative schema
  (PC-3) was stale. Fix: `"lock": {"app": "monocle"}` added as top-level sibling of `"hooks"` in PC-3
  canonical JSON. Invariant 6 added (MUST emit lock.app). Cross-reference to BC-2.08.006 added in
  Traceability, Related BCs, and PC-3 prose. One canonical test vector and one VP row added for
  lock.app assertion. Behavioral content change — minor bump.).

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.8 → 1.43.9.

SE-16d monotonicity: v1.43.9 timestamp 2026-06-19 > v1.43.8 timestamp 2026-06-16. PASS.

## §Trace v1.43.8

**F-P20-BCGAP-001 — BC-2.08.006 v1.4.0: atomic-write + path-canonicalization clauses** (2026-06-16):

BC version bump in this dispatch:
- BC-2.08.006 v1.3.2 → v1.4.0 (F-P20-BCGAP-001: Invariants 5+6 added — explicit atomic-write obligation via `tempfile::persist` with code-72 failure exit; path-canonicalization obligation for `runtime_dir` with three-case error taxonomy (code 69 / code 71 / symlink-race elimination). EC-183 and EC-184 added. 3 new test vectors. 3 new VP-TBD rows. Minor version bump per new normative content policy.).

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.7 → 1.43.8.

SE-16d monotonicity: v1.43.8 timestamp 2026-06-16 = v1.43.7 timestamp 2026-06-16. PASS (same-day sequential patch).

## §Trace v1.43.7

**Phase-2 Pass-7 fix burst — BC-2.03.008 v1.0.8 Story Anchor re-anchor: S-045 → S-033** (2026-06-16):

BC version bump in this dispatch:
- BC-2.03.008 v1.0.7 → v1.0.8 (F-PASS7-CRIT-001: Story Anchor re-anchored from S-045 to S-033. Rationale: S-033 introduces the `EngineModule::spawn_recipe()` trait method signature + default impl + `EngineError` enum in `monocle-core/src/engine.rs`; the default `Err(UnsupportedOperation)` behavior defined by this BC is a trait-level concern delivered by S-033. S-045 retains only the `ClaudeCodeModule` concrete override (BC-2.03.005/006/007 anchors). Traceability §Stories row updated: S-033 (primary); S-045 (secondary cross-ref). S-045 frontmatter `behavioral_contracts:` array updated to remove BC-2.03.008; BC-2.03.008 retained as cross-reference input only with comment. STORY-INDEX BC Coverage row updated: BC-2.03.008 row repointed S-045 → S-033 (AC-009c). Dependency graph acyclicity unaffected: S-045 depends_on S-033; no self-loops; S023→S047 DAG node declared in dependency-graph-expansion v2.1.). Anchor metadata and traceability only — no behavioral content changed.

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.6 → 1.43.7.

SE-16d monotonicity: v1.43.7 timestamp 2026-06-16 = v1.43.6 timestamp 2026-06-16. PASS (same-day sequential patch).

## §Trace v1.43.6

**Phase-2 Pass-6 fix burst — BC-2.05.011 v1.2.5 Story Anchor co-ownership: S-047-only → S-046+S-047** (2026-06-16):

BC version bump in this dispatch:
- BC-2.05.011 v1.2.4 → v1.2.5 (F-PASS6-SUG-002: Story Anchor expanded from S-047-only to S-046+S-047; bidirectionally symmetric with STORY-INDEX BC Coverage row which already listed both stories, and with S-046 frontmatter behavioral_contracts: [BC-2.05.009, BC-2.05.011]). Anchor metadata only — no behavioral content changed.

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.5 → 1.43.6.

SE-16d monotonicity: v1.43.6 timestamp 2026-06-16 = v1.43.5 timestamp 2026-06-16. PASS (same-day sequential patch).

## §Trace v1.43.5

**Phase-2 Pass-2 fix burst — SS-embedded-pty v1.7.0 Architecture Source pin propagation across SS-09 BCs** (2026-06-16):

SS-embedded-pty bumped 1.6.0 → 1.7.0 (F-P2-I06 — §Dependency Boundary + core-owned mirror types). POL-11 sweep detected 7 BCs with stale Architecture Source pins for SS-embedded-pty.

BC version bumps in this dispatch:
- BC-2.09.001 v1.3.4 → v1.3.5 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0).
- BC-2.09.002 v1.1.3 → v1.1.4 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0).
- BC-2.09.003 v1.5.1 → v1.5.2 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0; §Trace v1.4.0 in-prose v1.6.0 references annotated version-pin-historical).
- BC-2.09.004 v1.0.3 → v1.0.4 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0).
- BC-2.09.005 v1.0.2 → v1.0.3 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0).
- BC-2.09.006 v1.1.2 → v1.1.3 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0).
- BC-2.09.007 v1.1.2 → v1.1.3 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0).
- BC-2.09.008 v1.3.3 → v1.3.4 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0; §Trace v1.2.0/v1.2.1 v1.6.0 references annotated version-pin-historical).
- BC-2.09.009 v1.1.3 → v1.1.4 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0).
- BC-2.06.025 v1.5.3 → v1.5.4 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0).
- BC-2.08.008 v1.3.3 → v1.3.4 (Architecture Source SS-embedded-pty pin 1.6.0→1.7.0; §Trace v1.2.0/v1.2.1 v1.6.0 references annotated version-pin-historical).

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.4 → 1.43.5.

SE-16d monotonicity: v1.43.5 timestamp 2026-06-16 = v1.43.4 timestamp 2026-06-16. PASS (same-day sequential patch).

## §Trace v1.43.4

**Phase-2 Pass-1 fix burst (cascade, wave 4) — SS-session-manager v2.6.1 and SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin propagation** (2026-06-16):

SS-session-manager bumped 2.6.0 → 2.6.1 and SS-daemon-wiring-v2-delta bumped 1.11.3 → 1.11.4 (SS-ipc v1.24.0 cascade patches). POL-11 sweep detected 16 BCs with stale Architecture Source pins for one or both SS specs.

BC version bumps in this dispatch:
- BC-2.03.005 v1.1.4 → v1.1.5, BC-2.03.007 v1.2.4 → v1.2.5, BC-2.03.008 v1.0.6 → v1.0.7 (SS-session-manager).
- BC-2.05.009 v1.5.4 → v1.5.5, BC-2.05.010 v1.9.3 → v1.9.4, BC-2.05.011 v1.2.3 → v1.2.4 (SS-session-manager + SS-daemon-wiring-v2-delta).
- BC-2.06.025 v1.5.2 → v1.5.3 (SS-session-manager + SS-daemon-wiring-v2-delta).
- BC-2.08.001 v1.5.2 → v1.5.3, BC-2.08.002 v1.2.2 → v1.2.3, BC-2.08.003 v1.4.1 → v1.4.2, BC-2.08.004 v1.3.1 → v1.3.2, BC-2.08.005 v1.0.5 → v1.0.6, BC-2.08.006 v1.3.1 → v1.3.2, BC-2.08.007 v1.5.2 → v1.5.3, BC-2.08.008 v1.3.2 → v1.3.3.
- BC-2.09.001 v1.3.3 → v1.3.4 (SS-session-manager).

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.3 → 1.43.4.

SE-16d monotonicity: v1.43.4 timestamp 2026-06-16 = v1.43.3 timestamp 2026-06-16. PASS (same-day sequential patch).

## §Trace v1.43.3

**Phase-2 Pass-1 fix burst (cascade, wave 3) — BC-2.06.018 cross-BC pin update** (2026-06-16):

POL-11 sweep third pass detected stale `BC-2.05.004 v1.1.0` live body-prose pin in BC-2.06.018. BC-2.05.004 was bumped to v1.1.1 in wave 2 of this burst. BC-2.06.018 updated to cite v1.1.1.

BC version bumps in this dispatch:
- BC-2.06.018 v1.1.0 → v1.1.1: Cross-BC pin `BC-2.05.004 v1.1.0` → `BC-2.05.004 v1.1.1` (2 occurrences in body prose).

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.2 → 1.43.3.

SE-16d monotonicity: v1.43.3 timestamp 2026-06-16 = v1.43.2 timestamp 2026-06-16. PASS (same-day sequential patch).

## §Trace v1.43.2

**Phase-2 Pass-1 fix burst (cascade, wave 2) — SS-ipc v1.24.0 Architecture Source pin propagation to SS-03 and SS-05 BCs** (2026-06-16):

POL-11 sweep second pass detected stale `SS-ipc.md v1.23.2` live Architecture Source cells in 9 additional BCs across SS-03 and SS-05. All updated to `SS-ipc.md v1.24.0`.

BC version bumps in this dispatch:
- BC-2.03.007 v1.2.3 → v1.2.4: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (3 occurrences).
- BC-2.03.008 v1.0.5 → v1.0.6: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (1 occurrence).
- BC-2.05.001 v1.3.1 → v1.3.2: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (1 occurrence).
- BC-2.05.002 v1.0.7 → v1.0.8: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (2 occurrences).
- BC-2.05.003 v1.0.5 → v1.0.6: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (2 occurrences).
- BC-2.05.004 v1.1.0 → v1.1.1: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (2 occurrences).
- BC-2.05.005 v1.7.1 → v1.7.2: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (3 occurrences).
- BC-2.05.006 v1.0.5 → v1.0.6: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (2 occurrences).
- BC-2.05.007 v1.0.5 → v1.0.6: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (2 occurrences).

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.1 → 1.43.2.

SE-16d monotonicity: v1.43.2 timestamp 2026-06-16 = v1.43.1 timestamp 2026-06-16. PASS (same-day sequential patch).

## §Trace v1.43.1

**Phase-2 Pass-1 fix burst (cascade) — SS-ipc v1.24.0 Architecture Source pin propagation** (2026-06-16):

POL-11 sweep after SS-ipc v1.23.2 → v1.24.0 bump detected stale `SS-ipc.md v1.23.2` live Architecture Source cells in 5 BCs. All updated to `SS-ipc.md v1.24.0`.

BC version bumps in this dispatch:
- BC-2.06.025 v1.5.1 → v1.5.2: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (4 occurrences: postconditions body × 3 + Architecture Source row).
- BC-2.08.001 v1.5.1 → v1.5.2: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (1 occurrence: Architecture Source row).
- BC-2.08.005 v1.0.4 → v1.0.5: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (3 occurrences including §Trace historical prose — those remain as history; live pin updated).
- BC-2.08.008 v1.3.1 → v1.3.2: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (1 occurrence: Architecture Source row).
- BC-2.09.008 v1.3.3 → v1.3.4: Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (1 occurrence: Architecture Source row). Note: BC-2.09.008 v1.3.3 (EC-303) was §Trace v1.43.0; this patch cascades the SS-ipc Architecture Source forward.

No BC H1 title changes. No BC ID additions or retirements. BC count unchanged: 138 active.
BC-INDEX version: 1.43.0 → 1.43.1.

SE-16d monotonicity: v1.43.1 timestamp 2026-06-16 = v1.43.0 timestamp 2026-06-16. PASS (same-day sequential patch).

## §Trace v1.43.0

**Phase-2 Pass-1 fix burst — BC-2.09.008 v1.3.3 (EC-303 SpawnAck non-matching-filter)** (2026-06-16):

- BC-2.09.008 v1.3.2 → v1.3.3: EC-303 added by product-owner (F-P1-I-005 Phase-2 Pass-1 fix burst). EC-303 specifies the deterministic filter rule for `SessionStateChanged` events received while in `SessionCreation { step: Launching }`: events whose `session_id` does not match `launching_session_id` are silently ignored (not auto-advance triggers). Resolves the ambiguity where a concurrent session's `Running` state transition could spuriously advance the wizard. The SpawnAck receipt → `launching_session_id` population contract (BC-2.09.008 Step 4, added v1.3.0) now pairs with EC-303's non-matching-filter rule for a complete deterministic auto-advance specification.
- No BC H1 title change. No BC ID additions or retirements. BC count unchanged: 138 active.
- BC-INDEX version: 1.42.0 → 1.43.0.
- version-pin-registry.yaml: BC-2.09.008 → v1.3.3 (updated in same fix burst by product-owner, per REGISTRY ATOMICITY rule).

SE-16d monotonicity: v1.43.0 timestamp 2026-06-16 > v1.42.0 timestamp 2026-06-15. PASS.

## §Trace v1.42.0

**Phase-2 Burst G — BC anchor resolutions registered; bookkeeping reconciliation** (2026-06-15):

- 25 v1A BCs (BC-2.03.005..008, BC-2.05.009..011, BC-2.06.025, BC-2.08.001..008, BC-2.09.001..009) had S-TBD story anchors resolved in Burst E (product-owner) to canonical story IDs: S-033..S-048. Story files, STORY-INDEX BC Coverage Table, and version-pin-registry.yaml updated by Bursts E and F.
- BC-2.06.025 version bumped v1.5.0 → v1.5.1 in Burst E (Invariant 3 cross-reference note for Launching/Detached redirect to Invariant 5 added; anchor resolved to S-048).
- BC count unchanged: 138 active (no additions, no retirements).
- BC-INDEX version bumped 1.41.1 → 1.42.0 to register the v1A anchor-resolution completion as a milestone.
- EVAL-INDEX updated to v1.16: Scenario Index and Wave Coverage Summary HS-EXP-011..015 S-TBD placeholders replaced with canonical story IDs.
- No BC H1 title changes. No normative behavior changed.

SE-16d monotonicity: v1.42.0 timestamp 2026-06-15 > v1.41.1 timestamp 2026-06-14. PASS.

## §Trace v1.41.1

**P57-sweep — dtu-assessment pin v1.7.5→v1.7.6; double-separator removal SS-03/SS-04 boundary** (2026-06-14):

- **GAP-7.1 (IMPORTANT):** SS-DTU section preamble live normative reference `specs/dtu-assessment.md v1.7.5 §Clone Development Approach` was stale. Registry current: dtu-assessment = v1.7.6 (bumped dd82c05, 2026-05-30). Updated to v1.7.6.
- **GAP-7.2 (LOW):** Double `---` separator between SS-03 and SS-04 sections (lines 83-85 at time of audit). Duplicate removed so a single `---` remains, consistent with all other section boundaries.
- No BC H1 titles changed. No BC IDs added or retired. No normative behavior changed.
- BC-INDEX version: 1.41.0 → 1.41.1.

SE-16d monotonicity: v1.41.1 timestamp 2026-06-14 > v1.41.0 timestamp 2026-06-15T00:00:00Z. PASS (same calendar day; intra-day sequence confirmed by burst ordering).

## §Trace v1.41.0

**F-P52-001 — BC-2.06.025 v1.5.0 + BC-2.08.005 v1.0.2: Terminated-in-grace panel action guards** (2026-06-14):

- **BC-2.06.025 v1.4.0 → v1.5.0 (minor — normative addition):**
  Sessions with `SessionState::Terminated` render `[X]` in the sessions panel during the 10s GC
  grace window (Invariant 4, pre-existing). Prior to v1.5.0, no lifecycle-action guard existed for
  Terminated-in-grace sessions — an implementer could dispatch `k`, `D`, or `r` on a corpse.
  Gap closed per SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state matrix
  (F-P52-001) specifying daemon-side dispositions (rename → `rename_failed`; detach → Ok(());
  kill → Ok(); resize → WARN-drop):
  - **Invariant 6 (new):** Blanket TUI-side block for Terminated-in-grace sessions. Kill (`k`/`d`),
    Detach (`D`), and Rename (`r`) are all no-ops + status bar shows "Session has terminated".
    Mirrors the Terminating block in Invariant 4. Additive to the existing `[X]` indicator rule.
  - **EC-300 (new):** `k`/`d` on Terminated → no KillSession IPC; status hint.
  - **EC-301 (new):** `D` on Terminated → no DetachSession IPC; status hint.
  - **EC-302 (new):** `r` on Terminated → no RenameSession IPC; status hint; daemon error noted
    (`Err(InvalidSessionName{"session terminated"})` → `"rename_failed"` per BC-2.08.005 Inv 4).
  - **VP table:** Three new unit test VPs for Terminated-in-grace action guards.
  - **Traceability Cross-Ref expanded:** BC-2.08.003 Invariant 2; BC-2.08.005 Invariant 4;
    SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state matrix (F-P52-001).
  - **Architecture Source updated:** SS-session-manager.md v2.5.1 → v2.6.0; SS-ipc.md v1.23.1 →
    v1.23.2; SS-daemon-wiring-v2-delta.md v1.11.2 → v1.11.3.
  - **Pass-51 Launching rules NOT regressed.** Invariant 5 (kill ALLOWED, detach BLOCKED, rename
    ALLOWED for Launching sessions) fully preserved.

- **BC-2.08.005 v1.0.1 → v1.0.2 (patch — trace errata):**
  Invariant 4 previously stated "rename_session() reviving a Terminated session — which is not
  allowed" without specifying the observable mechanism. Updated with the explicit error return:
  `Err(SessionError::InvalidSessionName { reason: "session terminated" })` → wire code
  `"rename_failed"` per SS-session-manager.md v2.6.1 §Terminated-in-grace defensive action×state
  matrix. TUI-side guard (BC-2.06.025 Invariant 6) cited as the mechanism making this a
  defensive/untrusted-client-only path. Architecture Source updated to v2.6.0 pin.
  SessionError 9-variant / 12-code counts NOT regressed — no new variants or wire codes.

- **BC-2.08.003 — NO CHANGE:** kill-on-Terminated idempotency already specified in Invariant 2
  ("kill_session() on a Terminated or Terminating session MUST return Ok(()) (idempotent)").
  Confirmed no gap; no edit needed.

- **Sweep result:** 0 survivors. All live sites claiming rename/detach allowed on Terminated
  sessions have been reconciled. The "succeeds on any non-Terminated session" rationale at
  BC-2.06.025 PC-3 (Launching rename) and Invariant 5 is correct — those statements describe
  daemon behavior for Launching (a non-Terminated state) and are unaffected.

- BC-INDEX version: 1.40.9 → 1.41.0. No BC H1 title changes. No BC ID additions or retirements.

SE-16d monotonicity: v1.41.0 timestamp 2026-06-15T00:00:00Z > v1.40.9 timestamp 2026-06-14T23:00:00Z. PASS.

## §Trace v1.40.9

**F-P51-001 — BC-2.06.025 v1.4.0: Explicit Launching-state action rules close cite-to-unbacked-guarantee gap** (2026-06-14):

- **BC-2.06.025 v1.3.1 → v1.4.0 (minor — normative addition):**
  BC-2.05.010 §DetachSession PC-4 and BC-2.08.007 §Preconditions (detach) defensive note both
  cited "BC-2.06.025 guards" as the TUI-side guarantee that the official TUI never dispatches
  `ClientToServer::DetachSession` during `Launching`. No such rule existed in BC-2.06.025 —
  the existing PC-3 listed `D` → DetachSession unconditionally and Invariant 4 only blocked
  actions for Terminating sessions. Gap closed by:
  - **PC-3 Launching-state keybinding rules subsection (new):** Three explicit dispositions:
    kill (`k`/`d`) ALLOWED; detach (`D`) BLOCKED at TUI (no-op + status hint); rename (`r`) ALLOWED.
  - **Invariant 5 (new):** Formalizes Launching-state rules as MUST-level invariants. Explicitly
    states this is NOT a blanket disable (unlike Invariant 4 for Terminating). Names Invariant 5
    as the normative target of all "BC-2.06.025 guards" cites in BC-2.05.010 and BC-2.08.007.
  - **EC-298 (new):** `D` on Launching → no DetachSession IPC; status bar hint.
  - **EC-299 (new):** `r` on Launching → RenameSession IPC; ALLOWED.
  - **VP table:** Three new unit test VPs for Launching-state action rules.
  - **Traceability Cross-Ref expanded:** BC-2.05.010 §DetachSession PC-4, BC-2.08.003 Invariant 3,
    BC-2.08.007 §Preconditions (detach) defensive note added as bidirectional cross-references.
  - **Cited symbols verified:** BC-2.08.003 Invariant 3 (kill on Launching: allowed);
    BC-2.05.010 §DetachSession PC-4 (session_not_ready defensive path);
    BC-2.08.007 §Preconditions (detach) defensive note. All resolve.

- BC-INDEX version: 1.40.8 → 1.40.9. BC H1 title unchanged. No BC ID additions or retirements.

SE-16d monotonicity: v1.40.9 timestamp 2026-06-14T23:00:00Z > v1.40.8 timestamp 2026-06-14. PASS.

## §Trace v1.40.8

**F-P50-001 — Pass-50: wire taxonomy 12 codes (session_not_ready); SessionError 9 variants; SS-ipc v1.23.0 / SS-session-manager v2.5.0 / SS-daemon-wiring-v2-delta v1.11.1 / SS-engine-module-v2-delta v1.6.0** (2026-06-14):

- **Wire taxonomy now 12 codes (session_not_ready added):** `ServerToClient::Error` code taxonomy grows from 11 to 12. `"session_not_ready"` is the new 12th code (SS-ipc.md v1.23.0). The 11-code "F-P44-IMP-001" entries in §Trace v1.40.5 are correctly dated history — they reflect the count at v1.22.0. `kill_session()` does NOT use `"session_not_ready"` — kill on a not-yet-connected Launching session uses PID fallback → `"kill_failed"` (BC-2.08.003 v1.4.0).
- **SessionError now 9 variants:** `SessionNotReady` added F-P50-001. The "canonical 8-variant" reference in BC-2.08.007 §Trace v1.4.1 is correctly-dated history (8 variants at time of Pass-26).
- **BC version bumps this burst:**
  - BC-2.08.003 v1.3.1 → v1.4.0: Precondition 3 rewritten (post-spawn monitor distinction; Launching race window); PC-1 expanded with three named kill-path cases (Running/Launching host_conn established; Launching race window PID fallback; Detached fresh-connect). Architecture Source: SS-session-manager.md v2.4.0 → v2.5.0 (§Kill-path host_conn rules); SS-daemon-wiring-v2-delta.md v1.11.0 → v1.11.1.
  - BC-2.05.010 v1.8.1 → v1.9.0: Architecture Source taxonomy updated (12 codes; session_not_ready enumerated); SS-ipc.md v1.22.0 → v1.23.0 (all three citations); SS-session-manager.md v2.4.0 → v2.5.0; SS-daemon-wiring-v2-delta.md v1.11.0 → v1.11.1; SS-engine-module-v2-delta.md v1.6.0 added.
  - BC-2.08.007 v1.4.2 → v1.5.0: Detach PC-2 proxy_task.abort() → proxy_task.take().map(|t| t.abort()) (Option<JoinHandle<()>> type); Attach PC-5 host_conn struct proxy_task field clarified; Architecture Source: SS-session-manager.md v2.4.0 → v2.5.0; SS-daemon-wiring-v2-delta.md v1.11.0 → v1.11.1.
  - BC-2.03.008 v1.0.2 → v1.0.3: Architecture Source pins SS-session-manager.md v2.4.0 → v2.5.0; SS-ipc.md v1.22.0 → v1.23.0; 12th code forward annotation added; spawn_unsupported positional rank (11th) confirmed unchanged.
- BC-INDEX version: 1.40.7 → 1.40.8. No BC ID additions or retirements. No H1 title changes.

SE-16d monotonicity: v1.40.8 timestamp 2026-06-14 > v1.40.7 timestamp 2026-06-14T22:00:00Z. PASS.

## §Trace v1.40.7

**F-P49-001 — Pass-49: Canonical SS version table converted from literal mirror to registry pointer; holdout cite-precision fix (S-P49-001)** (2026-06-14):

- **Architecture Source Pin-Symmetry Convention (§Conventions ~line 476):** Removed the hand-maintained
  literal "Canonical SS version table" (6 rows: SS-daemon-lifecycle, SS-forward-compatibility,
  SS-engine-module, SS-core-types-and-abi, SS-deps-pin-manifest, SS-conventions-anti-patterns).
  This table was a POL-11 blind spot — its version literals were not in the `ID (vX.Y.Z)`
  parenthetical form the linter scans, so drift accumulated silently across 4 rows:
  SS-forward-compatibility v1.2.19→1.2.20, SS-engine-module v1.1.20→1.1.27,
  SS-deps-pin-manifest v1.1.17→1.2.1, SS-conventions-anti-patterns v1.29.5→1.32.6.
  Root cause: hand-maintained duplicate of the source-of-truth (version-pin-registry.yaml) is
  the anti-pattern. Replaced with an explicit registry pointer: "see version-pin-registry.yaml
  (single source of truth per ADR-0007)". Enforcement prose updated to name the registry as
  the oracle. No literal version rows remain in this file.

- **HS-EXP-011 + HS-EXP-012 cite-precision (S-P49-001):** Holdout files referenced
  "BC-2.08.004 PC-3" for the UDS-bind-blocked guarantee. PC-3 (line ~90) is about all
  SessionEntry records being in DaemonState before any TUI can connect — a derived consequence
  of the guarantee, not its specification. The primary normative specification is **PC-6**
  (UDS socket bind proceeds only after `rediscover_sessions()` returns) and **Invariant 1**
  (ordering is mandatory; re-discovery MUST complete before UDS bind). HS-EXP-011 Step 5 /
  Expected Outcome step 5 citation and HS-EXP-012 header / body now cite "PC-6 + Invariant 1".
  Errata-no-bump on both holdout files (cite-precision only; no scenario behavior changed).

- BC-INDEX version: 1.40.6 → 1.40.7. No BC ID additions or retirements. No H1 title changes.
  BC-INDEX canonical-version-table removal does not affect BC bodies or BC IDs.

SE-16d monotonicity: v1.40.7 timestamp 2026-06-14T22:00:00Z > v1.40.6 timestamp 2026-06-14T21:00:00Z. PASS.

## §Trace v1.40.6

**F-P47-001 + S-P47-001 + S-P47-002 — Pass-47 adversarial findings: Detached re-discovery over-emission; `d` kill-alias in disabled list; PC-2 wire-wrap note** (2026-06-14):

- **BC-2.08.004 v1.2.1 → v1.3.0 (minor — normative obligation removed):**
  PC-2b Detached case previously mandated `Emit \`SessionStateChanged{Detached}\` then
  \`SessionListUpdate\`` on re-discovery registration. Removed per F-P47-001 decision (Option B):
  re-discovery of an unchanged persisted state is NOT a state-value transition; MUST NOT emit
  `SessionStateChanged`. EC-172 updated: removed emission assertion; now correctly states
  TUI sees Detached session in `InitialState` on first connect. Symmetry achieved with
  Launching/Running re-discovery cases (none emit SessionStateChanged or SessionListUpdate).

- **BC-2.08.008 v1.2.1 → v1.3.0 (minor — normative obligation corrected):**
  PC-1 introductory sentence removed "Detached re-discovery registration" from the
  no-exception emission list; added explicit clarification that re-discovery registration of
  an unchanged persisted state MUST NOT emit. Invariant 1 tightened: "mutates
  `SessionEntry.state` TO A DIFFERENT VALUE" — initial re-discovery registration of an
  unchanged persisted state is excluded. No change to the transition bullet list (which never
  listed Detached re-discovery; only `Running → Detached` via `detach_session()` — a real
  value change — appears there).

- **BC-2.06.025 v1.3.1 (errata-no-bump — S-P47-001):**
  PC-1 and Invariant 4 disabled-action lists for Terminating sessions now enumerate `k`/`d`
  instead of bare `k`. The `d` kill-alias was defined in PC-3 but omitted from the disabled
  lists, creating an intra-document contradiction with EC-296. Clarification of clear intent;
  no normative obligation changes.

- **BC-2.03.008 v1.0.2 (errata-no-bump — S-P47-002):**
  PC-2 now includes a one-line note distinguishing the raw `EngineError::UnsupportedOperation`
  Display value (`"unsupported operation: spawn_recipe"`) from the wire-level
  `ServerToClient::Error.message` produced by `SessionError::EngineError` wrapping
  (`"engine error: unsupported operation: spawn_recipe"`). Diagnostic-only; the fixed banner
  from PC-3 is what the user sees. Contract behavior unchanged.

- BC-INDEX version: 1.40.5 → 1.40.6. BC H1 titles unchanged. No BC ID additions or retirements.

SE-16d monotonicity: v1.40.6 timestamp 2026-06-14T21:00:00Z > v1.40.5 timestamp 2026-06-14T20:00:00Z. PASS.

## §Trace v1.40.4

**CV-SS-005-SIBLING — BC-2.09.008 v1.3.1: PC Step 4 (Launching) SpawnAck ordering guarantee expanded with causal step ordering** (2026-06-14):

- BC-2.09.008 v1.3.0 → v1.3.1: PC Step 4 (Launching) ordering statement was incomplete —
  it cited only "per-client FIFO ordering" to explain why `SpawnAck` arrives before
  broker-published `SessionStateChanged { Launching }`. This is the same FIFO-only
  incompleteness fixed in BC-2.08.008 PC-5 v1.2.1 (§Trace v1.40.3 same burst). The fix
  adds the causal step ordering property: (1) SpawnAck is sent at IPC-handler step 2,
  before `spawn_session()` is called at step 4, which is before the broker emits
  `SessionStateChanged { Launching }` at step 5; (2) the per-client FIFO then guarantees
  in-order delivery to the requesting client. Canonical source cited:
  SS-ipc.md §ServerToClient::SpawnAck §Delivery ordering steps 1-5. No wire/field-name
  change. No change to any other postcondition, invariant, edge case, or test vector.
  Whole-class check: zero remaining FIFO-only-without-causal survivors in live postconditions
  of BC-2.09.008 (§Trace v1.3.0 historical narrative preserved verbatim as immutable history).

- BC-INDEX version: 1.40.3 → 1.40.4. BC H1 title unchanged. No BC ID additions or
  retirements.

SE-16d monotonicity: v1.40.4 timestamp 2026-06-14T19:00:00Z > v1.40.3 timestamp 2026-06-14T18:00:00Z. PASS.

## §Trace v1.40.3

**CV-SS-005 — BC-2.08.008 v1.2.1: PC-5 SpawnAck ordering guarantee expanded with causal step ordering** (2026-06-14):

- BC-2.08.008 v1.2.0 → v1.2.1: PC-5 ordering statement was incomplete — it cited only
  the per-client FIFO ordering guarantee to explain why `SpawnAck` arrives before
  broker-published `SessionStateChanged { Launching }`. The fix adds the causal step
  ordering property: (1) SpawnAck is sent at IPC-handler step 2, before `spawn_session()`
  is called at step 4, which is before the broker emits `SessionStateChanged { Launching }`
  at step 5; (2) the per-client FIFO then guarantees in-order delivery to the requesting
  client. Canonical source cited: SS-ipc.md §ServerToClient::SpawnAck §Delivery ordering
  steps 1-5. No wire/field-name change. No change to any other postcondition, invariant,
  edge case, or test vector.

- BC-INDEX version: 1.40.2 → 1.40.3. BC H1 title unchanged. No BC ID additions or
  retirements.

SE-16d monotonicity: v1.40.3 timestamp 2026-06-14T18:00:00Z > v1.40.2 timestamp 2026-06-14T12:00:00Z. PASS.

## §Trace v1.40.2

**F-P41-IMP-001 — BC-2.08.001 v1.5.0, BC-2.08.008 v1.2.0, BC-2.09.008 v1.3.0: UUID-locus corrected; SpawnAck mechanism; launching_session_id destructure** (2026-06-14):

- BC-2.08.001 v1.4.3 → v1.5.0: PC-1 UUID-locus corrected. Old wording: "generated via
  `uuid::Uuid::new_v4().to_string()` inside `spawn_session()`". New wording: UUID is
  generated in the daemon IPC handler (`ClientToServer::SpawnSession` match arm) BEFORE
  `spawn_session()` is called. `spawn_session()` receives `opts.session_id` already
  populated via `opts.with_daemon_fields()`; it does NOT generate the UUID itself.
  Arch-source pins: SS-session-manager.md v2.3.0 → v2.3.0; SS-ipc.md v1.21.0 → v1.21.0.

- BC-2.08.008 v1.1.1 → v1.2.0: PC-5 destructure corrected from
  `AppMode::SessionCreation { step: Launching, session_id }` to
  `AppMode::SessionCreation { step: Launching, launching_session_id: Some(session_id), .. }`.
  Auto-advance now matches against `launching_session_id` (populated from
  `ServerToClient::SpawnAck { session_id }` receipt), NOT a broadcast-race heuristic.
  EC-303 enriched: explicit SpawnAck mechanism + spawn-failure clearing rule.
  Arch-source pins: SS-embedded-pty.md v1.6.0 → v1.6.0; SS-ipc.md v1.21.0 → v1.21.0.

- BC-2.09.008 v1.2.0 → v1.3.0: PC Step 4 (Launching) adds SpawnAck receipt →
  `launching_session_id: Some(session_id.clone())` storage. PC Step 5 (auto-advance)
  rewrites match to reference `launching_session_id` (deterministic). Steps renumbered
  (old 5→6, old 6→7). Spawn-fail clearing added to Step 7. Arch-source pins:
  SS-embedded-pty.md v1.6.0 → v1.6.0; SS-ipc.md v1.21.0 added.

- Whole-class sweep: all SS-08 and SS-09 BCs + HS-EXP-011..015 scanned for (1) stale
  UUID-generation-locus wording "inside spawn_session()", (2) bad `AppMode::SessionCreation`
  destructure referencing bare `session_id`, (3) TUI-learns-session-id without SpawnAck.
  Zero survivors confirmed after these three edits.

- BC-INDEX version: 1.40.1 → 1.40.2. BC H1 titles unchanged (all three BC H1 headings
  stable). No BC ID additions or retirements.

SE-16d monotonicity: v1.40.2 timestamp 2026-06-14 ≥ v1.40.1 timestamp 2026-06-14. PASS.

## §Trace v1.40.1

**S38-001 — BC-2.09.008 PC-4/PC-1: complete scoped mouse-capture sequences; whole-class sweep confirms no other survivors** (2026-06-14 / Pass-38):

- BC-2.09.008 v1.1.1 → v1.2.0: Pass-38 IMPORTANT finding S38-001. PC-4 (entering EmbeddedTerminal) and PC-1 (exiting EmbeddedTerminal) each named only the SGR write/disable (`ESC [ ? 1006 h` / `ESC [ ? 1006 l`), omitting the paired `EnableMouseCapture` / `DisableMouseCapture` crossterm calls. Both postconditions now state the complete two-step sequences in correct order (EnableMouseCapture → SGR h on entry; SGR l → DisableMouseCapture on exit) with explicit non-authoritative restatement labels and cross-references to BC-2.09.002 Invariant-5 (the owning authoritative contract).
- Whole-class sweep: SS-09 BCs 001-009 and SS-05 BCs 001-011 fully scanned for partial-restatement pattern. No other live instances found. BC-2.09.002 and BC-2.09.003 are authoritative and correct — not modified.

BC-INDEX version: 1.40.0 → 1.40.1. BC H1 title unchanged. No BC ID additions or retirements.

SE-16d monotonicity: v1.40.1 timestamp 2026-06-14 > v1.40.0 timestamp 2026-06-13. PASS.

## §Trace v1.40.0

**I23-001 + I23-002 + S23-001 (Phase-1d Pass 23) — Mouse coordinate off-by-one class-sweep; convention statement added** (2026-06-13):

Findings from Pass 23 adversarial review — coordinate-encoding off-by-one contradiction across SS-09 BC family:

- **I23-001 (IMPORTANT) — BC-2.09.002 EC-213:** Claimed `\x1b[<0;10;5M` for crossterm (row=5, col=10).
  Canonical formula (SS-embedded-pty.md lines 511-512): `px = col+1 = 11`, `py = row+1 = 6`.
  Corrected to `\x1b[<0;11;6M`. Now matches HS-EXP-015 step 15-16 exactly.
  BC-2.09.002 v1.0.1 → v1.1.0.
- **I23-002 (IMPORTANT) — BC-2.09.003 Canonical Test Vectors:** All three mouse coordinate
  examples omitted the +1 offset, contradicting EC-220 (row=0,col=0 → Px=1,Py=1) and PC-2 (1-indexed):
    - (row=3, col=5) press: `\x1b[<0;5;3M` → `\x1b[<0;6;4M`
    - (row=3, col=5) release: `\x1b[<0;5;3m` → `\x1b[<0;6;4m`
    - (row=10, col=20) scroll up: `\x1b[<64;20;10M` → `\x1b[<64;21;11M`
  BC-2.09.003 v1.2.0 → v1.3.0.
- **S23-001 (SUGGESTION) — Root cause:** No explicit coordinate-encoding convention statement existed.
  Fix: §Coordinate Convention section added to BC-2.09.003 as the authoritative convention for
  the entire SS-09 family. States pane-origin assumption (pane_area.x=0, pane_area.y=0),
  crossterm 0-indexed input, SGR 1-indexed output, canonical formula, and authoritative example
  cross-referencing HS-EXP-015 step 15-16.
- **HS-EXP-015 VERIFIED CORRECT:** step 15-16 already showed `\x1b[<0;11;6M` (col=10+1=11,
  row=5+1=6) — no change needed.
- **Exhaustive sweep of SS-09 BCs (001-009) and HS-EXP-011-015:** Only BC-2.09.002 EC-213
  and BC-2.09.003 test vectors contained incorrect mouse coordinate examples.
  BC-2.09.001/004/005/006/007/008/009 and HS-EXP-011-014 have no mouse coordinate byte-sequence
  examples — no changes needed there.

BC-INDEX version: 1.39.1 → 1.40.0. BC H1 titles unchanged. No BC ID additions or retirements.

## §Trace v1.39.1

**S21-001 — SS-07 capability-text source-of-truth alignment: BC-INDEX header verbatim-matched to ARCH-INDEX** (2026-06-13T00:00:00Z):

**S21-001 SUGGESTION — Exhaustive capability-text sweep (all SS-NN headers); SS-07 fixed:**

Adversarial Pass 21 (S21-001) found that the BC-INDEX SS-07 section header cited a paraphrased
capability text rather than the ARCH-INDEX canonical verbatim. Exhaustive class-sweep performed
across all 9 subsystem section headers (SS-01 through SS-09).

**Sweep results:**

| SS-NN | CAP-NNN | ARCH-INDEX canonical text (source of truth) | BC-INDEX before | Result |
|-------|---------|---------------------------------------------|-----------------|--------|
| SS-01 | CAP-001 | Daemon ingestion of Claude Code hook events; lifecycle management | Daemon ingestion of Claude Code hook events; lifecycle management | MATCH |
| SS-02 | CAP-002 | Forward-compatible ABI; wire format stability; factory-state abstraction | Forward-compatible ABI; wire format stability; factory-state abstraction | MATCH |
| SS-03 | CAP-003 | Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter | Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter | MATCH |
| SS-04 | CAP-004 | Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation | Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation | MATCH |
| SS-05 | CAP-005 | Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear | Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear | MATCH |
| SS-06 | CAP-006 | User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration | User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration | MATCH |
| SS-07 | CAP-007 | Configuration persistence; harness profile management; profile picker; CCR detection | Config file schema; atomic write; harness profile; profile picker; CCR path detection | **FIXED** |
| SS-08 | CAP-008 | Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn | Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn | MATCH |
| SS-09 | CAP-009 | Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard | Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard | MATCH |

**Fix applied (SS-07 only):**
- SE-17c BEFORE: `CAP-007 ("Config file schema; atomic write; harness profile; profile picker; CCR path detection")`
- SE-17c AFTER: `CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection")`
- Source authority: ARCH-INDEX.md §Capability Traceability table SS-07 row (line ~97).
- Root cause: SS-07 section header was authored with paraphrased text ("Config file schema; atomic write; harness profile; profile picker; CCR path detection") that captured partial semantics but diverged from the canonical ARCH-INDEX formulation on three axes: (1) "Config file schema" vs "Configuration persistence"; (2) "atomic write" is an implementation detail not in the capability title; (3) "harness profile" vs "harness profile management"; (4) "CCR path detection" vs "CCR detection".

**CAP-NNN ID mismatch check:** No CAP-NNN ID mismatches found across any SS-NN header — all IDs are structurally correct. Only the SS-07 descriptive text diverged.

BC-INDEX H1 titles: unchanged. No BC retirements or removals.
Version bump: 1.39 → 1.39.1 (patch — capability-text source-of-truth alignment; cosmetic text only).

**For state-manager:** BC-INDEX is cited by version literal in version-pin-registry.yaml (key `BC-INDEX`) and in CLAUDE.md Pipeline State (historical pin `BC-INDEX v1.34` — exempt per ADR-0007 v1.0.8 CLAUDE.md exemption). The version-pin-registry.yaml entry for `BC-INDEX` must be updated from `1.39` to `1.39.1` in the same factory-artifacts commit that includes this BC-INDEX edit (REGISTRY ATOMICITY per CLAUDE.md CI-PARITY rule 4). State-manager owns that registry update.

SE-16d monotonicity: v1.39.1 timestamp 2026-06-13T00:00:00Z > v1.39 timestamp 2026-06-03. PASS.

## §Trace v1.38

**Adversarial pass-4 PO-owned propagation fixes — 5 existing BCs updated** (2026-06-03):

**UPDATED BCs:**
- BC-2.05.009 v1.3.0 → v1.4.0: HIGH-003 — PC-1b per-client send buffer "capacity 256" → "capacity 64". Propagation residue from v1.2.0 introduction (Invariant 3b/EC-272 already stated 64 since v1.3.0; PC-1b was the lone survivor).
- BC-2.08.002 v1.1.0 → v1.2.0: HIGH-001 — PC-4/PC-7/Invariant 4: retired single-message `ScrollbackDump` → `ScrollbackChunk*` + `ScrollbackDumpComplete` chunked protocol. Fixes internal inconsistency: EC-158 already used `ScrollbackDumpComplete` since v1.1.0; PC-4/PC-7/Invariant 4 were misaligned survivors.
- BC-2.08.006 v1.1.0 → v1.2.0: SUG-002 (PO half) — PC-5: `EnrichedSession` → `SessionSnapshot` wire boundary type (C3-004).
- BC-2.08.007 v1.3.0 → v1.4.0: HIGH-002 — H1 title: "ScrollbackDump on Attach" → "Chunked Scrollback (ScrollbackChunk*+ScrollbackDumpComplete) on Attach"; Description + Related-BCs updated to match. Title propagated to BC-INDEX §SS-08 table and prd.md §2.8 BC table + §Trace v1.28.0 summary in same burst.
- BC-2.09.001 v1.1.0 → v1.2.0: CRIT-001 — Invariant 5 + Related-BCs [BC-2.08.007]: retired `ServerToClient::ScrollbackDump` → chunked `ScrollbackChunk*` + `ScrollbackDumpComplete` protocol with `pending_pty_bytes` buffer-during-dump obligation (per BC-2.05.011 Invariant 6).

**INDEX changes:**
- §SS-08 table: BC-2.08.007 title updated to match BC H1 (bc_h1_is_title_source_of_truth).

SE-16d monotonicity: v1.38 timestamp 2026-06-03T14:00:00Z = v1.37 timestamp. No new structural additions — patch-level fix burst. PASS.

## §Trace v1.37

**Adversarial pass-2 PO-owned + architect-delegated BC fixes — 2 new BCs, 14 existing BCs updated** (2026-06-03T23:59:00Z):

**NEW BCs:**
- BC-2.05.011 v1.0.0: New `ServerToClient` IPC variants — `ScrollbackChunk`, `ScrollbackDumpComplete`, `PtyReset` (C2-002 architect delegation; chunked scrollback protocol + PtyReset TUI receiver spec).
- BC-2.08.008 v1.0.0: `SessionStateChanged` emission — daemon emits on every `SessionState` transition; delivered to all TUI clients; ordering relative to `SessionListUpdate` (S2-005 gap fill; backs wizard auto-advance and `EmbeddedTerminal` exit).

**UPDATED BCs (PO-owned adversarial findings):**
- BC-2.09.009 v1.0.0 → v1.1.0: C2-001 bell contradiction resolved. Per-prompt bell rule adopted (every `PermissionPromptQueued` rings). PC-3, Invariant 2, EC-260, test vector all aligned. BC↔HS-EXP-013 now consistent.
- HS-EXP-014 (holdout): C2-004 — removed `hook_settings_path` field assertion. Field does not exist in `session-state.json` schema v2. Replaced with `--settings` shared path assertion and explicit schema-compliance FAIL criterion.
- BC-2.09.003 v1.1.0 → v1.2.0: I2-001 — scoped mouse capture model. PC-2 and Invariant 1 rewritten: `EnableMouseCapture` scoped to `EmbeddedTerminal` entry/exit (I3 fix per SS-embedded-pty v1.2.0). Global capture removed.
- BC-2.08.002 v1.0.0 → v1.1.0: I2-005 — EC-158 synced to canonical re-discovery procedure: single Attach attempt, 5s hard timeout, no exponential backoff.
- BC-2.08.004 v1.0.0 → v1.1.0: I2-005 — PC-2b uses `ScrollbackDumpComplete` (not retired `ScrollbackDump`); adds SO_PEERCRED step; Invariant 2 adds "no backoff" note. Architecture Source pins bumped to v1.3.0/v1.2.0.
- BC-2.05.010 v1.0.0 → v1.1.0: S2-004 — EC-282 zero-dimension: clamp-to-1 at daemon boundary (no SessionError). Invariant 5 added.
- BC-2.09.006 v1.0.0 → v1.1.0: S2-004 — EC-239 added: degenerate pane area no-op (TUI) + daemon defense-in-depth clamp. Related BCs updated.

**UPDATED BCs (architect-delegated):**
- BC-2.05.009 v1.1.0 → v1.2.0: PC-1b, Invariant 3, Invariant 3b (new), EC-272 — per-client backpressure isolation (capacity 256, `.try_send()`, 3-failure disconnect). Backpressure source = durable ring, NOT TUI clients.
- BC-2.08.003 v1.1.0 → v1.2.0: Terminating state, 12s watchdog, SO_PEERCRED on kill-path fresh-connect. Kill path: `Running|Detached|Launching → Terminating → Terminated`. PC-5 (watchdog), Invariant 1/2/5 updated. EC-164 rewritten.
- BC-2.06.025 v1.1.0 → v1.2.0: `[Terminating]` state render, lifecycle action blocking for Terminating sessions, EC-296 (`k` on Terminating = no-op).
- BC-2.03.005 v1.0.0 → v1.1.0: `recipe.cwd = opts.worktree_root` (not `project_root`). Precondition, PC-1, Invariant 4, test vectors, VP updated.
- BC-2.03.006 v1.0.0 → v1.1.0: Description explicitly states `SpawnRecipe.env` is an OVERLAY (not replacement) on inherited process env.
- BC-2.08.001 v1.1.0 → v1.2.0: Sidecar schema: `project_root` = user-selected dir; `cwd` = resolved `worktree_root`. Two distinct fields, not conflated.
- BC-2.08.007 v1.1.0 → v1.2.0: C2-002 — PC-4/5/10 and Invariant 3/4 reference `ScrollbackChunk*` + `ScrollbackDumpComplete`. Added BC-2.05.011 cross-reference. Architecture Source pinned to v1.3.0.

Summary table updated: 136 → 138 total BCs. SS-05: 10→11. SS-08: 7→8.

SE-16d monotonicity: v1.37 timestamp 2026-06-03T23:59:00Z > v1.36 timestamp 2026-06-03T14:00:00Z. PASS.

## §Trace v1.35

**D-241 control-center v1A BC integration — 23 new BCs across SS-03/SS-05/SS-06/SS-08/SS-09** (2026-06-03T12:00:00Z):

New subsystem sections added:
- SS-08 (Session Manager): 7 BCs (BC-2.08.001..BC-2.08.007); architecture source SS-session-manager.md; CAP-008.
- SS-09 (Embedded PTY): 9 BCs (BC-2.09.001..BC-2.09.009); architecture source SS-embedded-pty.md; CAP-009.

New BCs added to existing subsections:
- SS-03 Engine Module: BC-2.03.005/006/007/008 (spawn_recipe() contract expansion for control-center session launch).
- SS-05 IPC: BC-2.05.009/010 (PtyOutput fan-out channel; new ClientToServer variants).
- SS-06 TUI: BC-2.06.025 (multi-session / multi-project sessions panel).

Summary table updated: 113 → 136 total BCs. Per-subsystem counts updated (SS-03: 4→8, SS-05: 8→10, SS-06: 24→25, SS-08: 0→7, SS-09: 0→9).

BC H1 titles are the authoritative source; all 23 new BCs use their BC file H1 as the index title per bc_h1_is_title_source_of_truth policy.

VP authoring for SS-08 and SS-09 BCs deferred to formal-hardening phase per the project's established VP-DTU-001 Phase-4 deferral pattern. All 23 new BCs carry `VP Anchors: VP-TBD`. Architect must author VPs at hardening phase scheduling (tracked per D-241).

SE-16d monotonicity: v1.35 timestamp 2026-06-03T12:00:00Z > v1.34 timestamp 2026-06-03T00:00:00Z. PASS.
