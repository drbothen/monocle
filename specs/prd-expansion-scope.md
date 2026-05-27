---
document_type: plan-doc
level: L3
version: "1.0"
status: draft
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
phase: phase-1-expansion
inputs:
  - {path: .factory/specs/product-brief.md, version: "1.4.30"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.14"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/research/domain-monocle-vision-synthesis.md, version: "1.1.3"}
input-hash: "[pending]"
traces_to: "product-brief.md v1.4.30 §Phase 1 scope"
project: monocle
---

# PRD Expansion Scope Document: Monocle Phase 1 — Gap Analysis

> **Purpose.** This document maps every Phase 1 feature from the product brief against the
> current PRD (v1.26.15) behavioral contract coverage. It identifies which features are
> fully covered, partially covered, or entirely missing from the current 22 BCs + 41 DTU BCs.
> It proposes new subsystems (SS-04 through SS-07) and outlines the BCs needed to bring the
> PRD into full coverage of the Phase 1 delivery contract.
>
> **DO NOT create BC files yet.** This document is the scope definition that drives the PRD
> expansion. BC files will be created after human review of the proposals herein.
>
> **DO NOT modify the existing PRD.** This document is a SEPARATE planning artifact.

---

## Section 1: Feature Coverage Matrix

The Phase 1 delivery contract is defined in `product-brief.md v1.4.30` §Phase 1 scope
(lines 110-176) and §Success Criteria (lines 239-252). The table below enumerates every
discrete Phase 1 feature, cites the brief reference, maps current BC coverage, and
classifies gap status.

Coverage levels:
- **COVERED** — one or more active BCs fully specify the behavior
- **PARTIAL** — the feature appears in BCs but key behaviors, preconditions, or edge cases are absent
- **MISSING** — no BC exists for this feature; implementation cannot be tested against contract

### 1.1 Infrastructure and Daemon

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-01 | `monocle daemon start/stop` CLI command | §Phase 1 line 112 | None | MISSING |
| F-02 | Daemon binds axum HTTP on OS-assigned port | §Phase 1 line 112 | BC-2.01.001, BC-2.01.002 (test endpoints exist but daemon startup contract missing) | PARTIAL |
| F-03 | Lock file written at `runtime_dir` with `{port, token, contract_version}` at mode `0o600` | §Phase 1 lines 112-118 | BC-2.01.005, BC-2.01.010 (file created and contract version present; SOQ-2 token rotation invariant not specced) | PARTIAL |
| F-04 | `directories::ProjectDirs::runtime_dir()` fallback chain (`runtime_dir → state_dir → data_dir → ~/.monocle`) | §Phase 1 lines 117-118 | None | MISSING |
| F-05 | Daemon auto-start on first TUI launch with `MONOCLE_NO_AUTOSTART=1` escape hatch | §Phase 1 line 116 | None | MISSING |
| F-06 | Hook ingestion: 5 POST endpoints (pre-tool-use, notification, stop, session-start, prompt-submit) | §Phase 1 lines 120-128 | BC-2.01.003 (body size), BC-2.01.009 (auth) — endpoints exist but no schema validation, response contract, or routing BCs | PARTIAL |
| F-07 | Hook receiver: 256 KiB body size limit (HTTP 413) | §Phase 1 lines 129-130 | BC-2.01.003 | COVERED |
| F-08 | Hook receiver: `/healthz` liveness endpoint | §Phase 1 line 130 | BC-2.01.001 | COVERED |
| F-09 | Hook receiver: `/status` daemon state endpoint | §Phase 1 line 130 | BC-2.01.002 | COVERED |
| F-10 | Hook receiver: graceful shutdown on SIGTERM/SIGINT (10-second drain) | §Phase 1 lines 130-134 | BC-2.01.004 | COVERED |
| F-11 | Hook receiver: dual-accept auth header (canonical + alias) | §Phase 1 lines 120-128 | BC-2.01.008, BC-2.01.009 | COVERED |
| F-12 | Hook tmpfile: shared per-runtimeDir, mode `0o600`, atomic-replace | §Phase 1 line 135 | None (BC-HOOK-009 covers write path from Claude Code side, not daemon side) | MISSING |
| F-13 | JSONL ring buffer: hybrid RAM ring + async JSONL flush, 100MB × 5 rotation | §Phase 1 lines 148-150 | BC-2.01.007 (format version only; no ring capacity, rotation, or flush contract) | PARTIAL |
| F-14 | Lock file contract version field `"monocle-lock-v1"` | §Phase 1 lines 112-118 | BC-2.01.010 | COVERED |
| F-15 | Lock file: PID liveness check (no stale daemon) | §Phase 1 line 112 | BC-2.01.005 | COVERED |
| F-16 | Crash recovery checkpoint | §Phase 1 line 130 | BC-2.01.006 | COVERED |
| F-17 | Auth token wire format `monocle-v1:<64-hex>` | §Phase 1 line 176 | BC-2.01.008 | COVERED |
| F-18 | ABI version const `MONOCLE_ABI_VERSION = 1` | §Phase 1 line 171 | BC-2.02.001, BC-2.02.002 | COVERED |
| F-19 | Public enum `#[non_exhaustive]` policy | §Phase 1 line 172 | BC-2.02.003 | COVERED |
| F-20 | `FactoryAdapter` trait + `VsddFactoryAdapter` implementation | §Phase 1 line 173 | BC-2.02.004, BC-2.02.005 | COVERED |
| F-21 | `monocle-proto` prost wire schemas with `schema_version = 1` | §Phase 1 lines 174 | BC-2.02.006, BC-2.02.007, BC-2.02.008 | COVERED |
| F-22 | SOQ-2 token rotation invariant (bind → write lock-file → write token → hooks-settings reads token) | §Phase 1 lines 118-119 | None | MISSING |
| F-23 | `MONOCLE_NO_AUTOSTART=1` env var checked before daemon auto-start | §Phase 1 line 116 | None | MISSING |

### 1.2 IPC Layer

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-24 | `monocle-ipc`: Unix domain socket IPC between TUI client and daemon | §Phase 1 line 157 | None | MISSING |
| F-25 | UDS-only in v1 (shared-memory ring deferred to Phase 4) | §Phase 1 line 157 | None (OQ-08 decision documented in brief only) | MISSING |
| F-26 | IPC message types for TUI-daemon communication (session list, hook events, permission prompts) | Vision §Key Abstractions | None | MISSING |
| F-27 | IPC reconnection behavior (TUI disconnects and reconnects; daemon continues) | Vision §Process Topology | None | MISSING |
| F-28 | Overlay clear on daemon disconnect (SOQ-3) | §Phase 1 line 145 | None | MISSING |

### 1.3 TUI — Sessions Panel

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-29 | Sessions panel: live session roster (harness icon, project name, phase tag, token count, cost, uptime) | §Phase 1 lines 139-141 | BC-2.03.001, BC-2.03.002 (EngineModule trait and ClaudeCodeModule detect; no panel rendering contract) | PARTIAL |
| F-30 | Sessions panel: `/` filter with nucleo-matcher | §Phase 1 line 140 | None | MISSING |
| F-31 | Sessions panel: `Enter` fullscreen | §Phase 1 line 140 | None | MISSING |
| F-32 | Sessions panel: harness icon column | Vision §TUI Layout | None | MISSING |
| F-33 | `Ctrl-\` tmux popup: popup appears over editor, hides/shows without state loss | Brief §Who Is It For + Vision §Process Topology | None | MISSING |

### 1.4 TUI — Permission Overlay

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-34 | Permission prompt overlay: `VecDeque<PromptModal>` stack (both prompts visible simultaneously) | §Phase 1 lines 141-145 | BC-2.03.001 (EngineModule returns `HookDecision::Defer` — precondition for queuing; no overlay rendering or stack management contract) | PARTIAL |
| F-35 | Permission overlay: diff preview via `similar 3` | §Phase 1 line 143 | None | MISSING |
| F-36 | Permission overlay: Accept-once / Accept-always / Reject keybindings | §Phase 1 lines 142-144 | None | MISSING |
| F-37 | Permission overlay: `[t]` trace-to-source stub | §Phase 1 line 144 | None | MISSING |
| F-38 | Overlay survives `Ctrl-\` hide/show cycle without dropping queued prompts | §Phase 1 line 145 | None | MISSING |
| F-39 | Overlay clears on daemon disconnect (SOQ-3) | §Phase 1 line 145 | None | MISSING |
| F-40 | `[↑↓]` rotates overlay stack; `[Esc]` hides without rejecting | Vision §TUI Layout overlay diagram | None | MISSING |
| F-41 | Permission response sent to Claude Code within hook timeout budget (≤300ms for PreToolUse) | §Success Criteria line 244-245 | None | MISSING |

### 1.5 TUI — Event Ribbon Panel

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-42 | Event ribbon panel: rolling log of hook events (type, session ID, latency) | §Phase 1 lines 148-150 | None | MISSING |
| F-43 | Event ribbon: shows PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit | §Phase 1 line 149 | None | MISSING |
| F-44 | Event ribbon: latency column (time from hook POST to daemon ACK) | §Phase 1 line 149 | None | MISSING |
| F-45 | Drop counter rendered in status bar under synthetic 1000 events/sec load | §Success Criteria line 249 | None | MISSING |

### 1.6 TUI — AppMode State Machine and Keybinding Dispatch

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-46 | AppMode state machine: compile-time mutual exclusion (Dashboard, Filtering, Overlay, Fullscreen) | Vision §Key Abstractions + brief line 288 | None | MISSING |
| F-47 | `FocusSnapshot` enum: explicit focus tracking across mode transitions | Vision §Key Abstractions | None | MISSING |
| F-48 | Action enum dispatch: 5-level binding precedence (SearchPrompt > UserCustomCommand > PerContext > Global > Builtin) | Brief lines 282-286 + Vision §Key Abstractions | None | MISSING |
| F-49 | State transitions are pure functions in `monocle-core` (no `Arc<Mutex<Option<...>>>`) | Brief line 291 | None | MISSING |

### 1.7 TUI — Status Bar

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-50 | Bounded event bus with drop counter in status bar | §Phase 1 line 154; §Success Criteria line 249 | None | MISSING |
| F-51 | Status bar renders breadcrumb (e.g., "Dashboard > Sessions") | Vision §TUI Layout | None | MISSING |
| F-52 | Status bar renders keybinding hint line | Vision §TUI Layout | None | MISSING |

### 1.8 Config

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-53 | `monocle-config`: reads/writes `~/.monocle/config.json` via `tempfile::persist` | §Phase 1 lines 151-153 | None | MISSING |
| F-54 | Config: harness profile schema version 1 | §Phase 1 line 152 | None | MISSING |
| F-55 | Config: CCR path field | §Phase 1 line 152 | None | MISSING |
| F-56 | Config: binding overrides stub | §Phase 1 line 153 | None | MISSING |
| F-57 | Profile picker: sticky-per-project with `Ctrl-P` override | §Phase 1 lines 146-147 | None | MISSING |
| F-58 | Config file atomic write via `tempfile::persist` (no `std::fs::write`) | Vision §Tech Stack + SS-conventions-anti-patterns.md | None | MISSING |

### 1.9 Daemon Wiring (Binary Crate Entry Point)

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-59 | `monocle` binary crate: `clap` CLI with `daemon start`, `daemon stop` subcommands | Brief line 112 + Vision §Workspace Layout | None | MISSING |
| F-60 | `monocle` binary: TUI launch with auto-start daemon if not running | Brief line 116 | None | MISSING |
| F-61 | `monocle` binary: `MONOCLE_NO_AUTOSTART=1` env var check before auto-start | Brief line 116 | None | MISSING |
| F-62 | Hook tmpfile generation and injection into Claude Code `--settings` flag | Brief line 135; BC-HOOK-027 (gene-source) | None (BC-HOOK-027 covers Claude Code side; monocle side unspecced) | MISSING |
| F-63 | Tokio mpsc bounded event bus with drop counter (no unbounded channels) | §Phase 1 line 154; Anti-pattern in SS-conventions | None | MISSING |
| F-64 | CI matrix: macOS + Linux × amd64/arm64 | §Phase 1 line 167; §Success Criteria | None (build constraint, not a BC per se — tracked as NFR-007/NFR-008 in nfr-catalog.md) | COVERED (NFR) |

### 1.10 Forward-Compatibility Contracts

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-65 | FC-01: JSONL `format_version = 1` first key | §Phase 1 line 175 | BC-2.01.007 | COVERED |
| F-66 | FC-02: `#[non_exhaustive]` on public enums | §Phase 1 line 172 | BC-2.02.003 | COVERED |
| F-67 | FC-03: `MONOCLE_ABI_VERSION = 1` const + `/status` exposure | §Phase 1 line 171 | BC-2.02.001, BC-2.02.002 | COVERED |
| F-68 | FC-04: `FactoryAdapter` trait + `VsddFactoryAdapter` | §Phase 1 line 173 | BC-2.02.004, BC-2.02.005 | COVERED |
| F-69 | FC-05: `monocle-proto` HookEnvelope + 5 event messages, `schema_version = 1` | §Phase 1 line 174 | BC-2.02.006, BC-2.02.007, BC-2.02.008 | COVERED |
| F-70 | FC-06: auth token format + non-prefix rejection | §Phase 1 line 176 | BC-2.01.008, BC-2.01.009 | COVERED |

### 1.11 ClaudeCodeModule

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-71 | `ClaudeCodeModule`: detect Claude Code processes via PID walk + strict-basename | §Phase 1 lines 136-138 | BC-2.03.002 | COVERED |
| F-72 | `ClaudeCodeModule`: enrich with token counts, cost, phase tag from hook events | §Phase 1 line 137 | BC-2.03.001 (on_hook → HookDecision::Defer), BC-2.03.004 (inherent methods) — no enrichment data-flow contract | PARTIAL |
| F-73 | `ClaudeCodeModule`: `hook_paths`, `spawn`, `preflight` inherent methods | §Phase 1 line 138 | BC-2.03.004 | COVERED |
| F-74 | `HomeUnresolvable` error contract | §Phase 1 line 137 | BC-2.03.003 | COVERED |
| F-75 | `EngineModule` trait definition | §Phase 1 lines 136-138 | BC-2.03.001 | COVERED |

### 1.12 DTU Clone

| # | Feature | Brief Reference | Current BCs | Gap Status |
|---|---------|----------------|-------------|------------|
| F-76 | `dtu-claude-code-hooks-v1` DTU clone exists | §Phase 1 line 169; §Success Criteria line 251 | BC-HOOK-001..BC-HOOK-041 (gene-source contracts; DTU clone BCs) | COVERED |
| F-77 | DTU clone fidelity ≥0.95 against fixture corpus | §Success Criteria line 251 | NFR-011 | COVERED (NFR) |

---

### 1.13 Coverage Summary

| Status | Feature Count | Percentage |
|--------|--------------|------------|
| COVERED | 29 | 38% |
| PARTIAL | 10 | 13% |
| MISSING | 38 | 50% |

**Conclusion:** The current PRD covers the forward-compatibility contracts and the infrastructure substrate in full, but the product-facing layer — TUI, IPC, config, daemon wiring, and event bus — has zero BC coverage. 50% of Phase 1 features are entirely unspecced. This is why the product brief describes a working TUI application but the PRD covers only "Phase 1 Forward-Compatibility Contracts."

---

## Section 2: Proposed New Subsystems

Based on the gap analysis, four new subsystems are required to cover the missing Phase 1 features. These map directly to the crate boundaries defined in vision §Workspace Layout and the brief §Constraints.

### SS-04: Daemon Wiring

**Architecture document:** `SS-daemon-wiring.md` (to be created by architect)

**Implementing crate(s):**
- `monocle` (binary crate — `main.rs`, `clap` CLI, daemon entrypoint, TUI entrypoint)
- `monocle-runtime` (hook tmpfile generation, bounded event bus, `MONOCLE_NO_AUTOSTART` check)

**Capabilities covered from the brief:**
- `monocle daemon start/stop` CLI subcommands (F-01, F-59)
- Daemon auto-start on first TUI launch with `MONOCLE_NO_AUTOSTART=1` escape hatch (F-05, F-23, F-60, F-61)
- `directories::ProjectDirs::runtime_dir()` fallback chain (F-04)
- SOQ-2 token rotation invariant (F-22)
- Hook tmpfile generation and injection via `--settings` flag (F-62)
- Tokio mpsc bounded event bus with drop counter (F-50, F-63)
- Hook endpoint routing: request deserialization, session dispatch, response formation (F-06 partial)
- Ring buffer init: capacity, rotation policy, JSONL flush trigger (F-13 partial)

**Estimated BCs needed:** 12

**Rationale:** The `monocle` binary crate is the composition root — it wires together monocle-runtime, monocle-tui, monocle-config, and monocle-ipc. It owns the CLI surface (`clap`), the daemon lifecycle coordination (start/stop/auto-start), and the event bus backbone. These behaviors are production-critical and need explicit behavioral contracts: the auto-start path is exercised on every developer's first `monocle` invocation; the token rotation invariant (SOQ-2) prevents auth races at daemon startup; the bounded event bus with drop counter directly satisfies a Phase 1 success criterion.

### SS-05: IPC

**Architecture document:** `SS-ipc.md` (to be created by architect)

**Implementing crate(s):**
- `monocle-ipc` (UDS client + server, message types, framing)

**Capabilities covered from the brief:**
- `monocle-ipc` Unix domain socket IPC between TUI client and daemon (F-24, F-25)
- IPC message types: session list push, hook event push, permission prompt push (F-26)
- IPC reconnection behavior (F-27)
- Overlay clear on daemon disconnect (SOQ-3) (F-28, F-39)

**Estimated BCs needed:** 8

**Rationale:** `monocle-ipc` is the internal transport between the TUI client (which the user sees) and the daemon (which processes hook events). Without IPC BCs, there is no contract for what the TUI receives from the daemon, how it reconnects, or how it handles the SOQ-3 disconnect requirement. The UDS-only constraint (OQ-08) must be enforced by contract to prevent Phase 4 shared-memory transport from being accidentally shipped in Phase 1.

### SS-06: TUI

**Architecture document:** `SS-tui.md` (to be created by architect)

**Implementing crate(s):**
- `monocle-tui` (ratatui renderer, panel layout, AppMode state machine, keybinding dispatch)
- `monocle-core` (AppMode, Action, FocusSnapshot, BindingSource types — pure data)

**Capabilities covered from the brief:**
- `Ctrl-\` tmux popup: appears over editor, hides/shows without state loss (F-33)
- Sessions panel: roster rendering, filter, fullscreen (F-29, F-30, F-31, F-32)
- Permission overlay: VecDeque stack, diff preview, keybindings, trace-to-source stub (F-34–F-41)
- Event ribbon panel: rolling hook event log, latency column (F-42–F-44)
- Status bar: drop counter, breadcrumb, keybinding hint (F-50–F-52)
- AppMode state machine: compile-time mutual exclusion (F-46, F-47)
- Action enum dispatch: 5-level precedence (F-48)
- Pure state transitions in `monocle-core` (F-49)
- Permission response sent to Claude Code within hook timeout budget (F-41)

**Estimated BCs needed:** 22

**Rationale:** The TUI is the user-facing layer — the "one `Ctrl-\` popup" that is the entire product value proposition. The killer scenario (4 keystrokes, zero context switches) is entirely unspecced at the behavioral contract level. The VecDeque overlay stack (competitive differentiator D-2 in the PRD) has a precondition in BC-2.03.001 but no contract for the actual overlay behavior, diff preview, or keybinding response. The AppMode state machine is a Phase 1 architectural constraint but has no BCs enforcing compile-time correctness, purity of transitions, or the FocusSnapshot restore behavior. Without TUI BCs, Phase 1 cannot be considered "done" — the success criterion "resolve two concurrent permission prompts in ≤6 keystrokes" has no testable specification.

### SS-07: Config

**Architecture document:** `SS-config.md` (to be created by architect)

**Implementing crate(s):**
- `monocle-config` (config.json reader/writer, harness profile schema, profile picker logic)

**Capabilities covered from the brief:**
- `monocle-config`: reads/writes `~/.monocle/config.json` via `tempfile::persist` (F-53, F-58)
- Harness profile schema version 1 (F-54)
- CCR path field in config (F-55)
- Binding overrides stub in config (F-56)
- Profile picker: sticky-per-project with `Ctrl-P` override (F-57)

**Estimated BCs needed:** 6

**Rationale:** `monocle-config` is the configuration persistence layer. It is referenced by both the daemon (harness profiles, CCR detection) and the TUI (profile picker, binding overrides). The requirement for `tempfile::persist` (no naked `std::fs::write`) is an anti-pattern prohibition in `SS-conventions-anti-patterns.md` that must be enforced by a BC — the test-writer cannot write a test for this without a contract. The profile picker's sticky-per-project behavior is a Phase 1 user-test target with MEDIUM confidence (per OQ-05) and needs a BC to allow the holdout evaluator to validate it.

---

## Section 3: Proposed BC Outline

### 3.1 SS-04: Daemon Wiring (BC-2.04.NNN)

| Proposed BC ID | Title | Priority | Description | Brief Feature |
|----------------|-------|----------|-------------|---------------|
| BC-2.04.001 | Daemon Start Sequence: Port Bind + Lock File + Token Write (SOQ-2) | P0 | Daemon binds OS-assigned port, then writes lock file with `{port, pid, token, contract_version}` at mode `0o600`, then the hooks-settings file reads the token. Any failure in write order causes startup abort. Enforces SOQ-2 invariant. | F-02, F-03, F-22 |
| BC-2.04.002 | Daemon Auto-Start on TUI Launch | P0 | When `monocle` (TUI mode) launches and no daemon is running, it starts a daemon subprocess before rendering the TUI. The daemon PID must pass liveness check before TUI connection attempt. | F-05, F-60 |
| BC-2.04.003 | `MONOCLE_NO_AUTOSTART=1` Suppresses Auto-Start | P0 | When `MONOCLE_NO_AUTOSTART=1` env var is set, `monocle` (TUI mode) does NOT start a daemon. TUI renders with "daemon offline" state. | F-23, F-61 |
| BC-2.04.004 | `monocle daemon start` CLI Subcommand | P0 | `monocle daemon start` starts the daemon in background (detached), writes lock file, exits with code 0. If daemon is already running (lock file exists + PID alive), exits code 1 with structured error. | F-01, F-59 |
| BC-2.04.005 | `monocle daemon stop` CLI Subcommand | P0 | `monocle daemon stop` sends SIGTERM to the PID in the lock file. Waits up to 15 seconds for graceful shutdown. If PID does not exist, exits code 1. | F-01, F-59 |
| BC-2.04.006 | `directories::ProjectDirs::runtime_dir()` Fallback Chain | P0 | Lock file path resolves via `runtime_dir()` → `state_dir()` → `data_dir()` → `~/.monocle`. First non-None result is used. All fallback levels must be tested. | F-04 |
| BC-2.04.007 | Hook Endpoint: PreToolUse Request Routing | P0 | POST `/hooks/pre-tool-use` with valid auth and valid body is deserialized, dispatched to the registered `EngineModule::on_hook()`, and responds within 300ms with the `HookResponse` decision. | F-06 |
| BC-2.04.008 | Hook Endpoint: Notification Request Routing | P0 | POST `/hooks/notification` with valid auth and valid body dispatches to `EngineModule::on_hook()` and responds within 2000ms. | F-06 |
| BC-2.04.009 | Hook Endpoint: Stop/SessionStart/PromptSubmit Routing | P0 | POST `/hooks/stop`, `/hooks/session-start`, `/hooks/prompt-submit` — each dispatches to `EngineModule::on_hook()` and responds within 300ms. | F-06 |
| BC-2.04.010 | Hook Tmpfile Generation at `runtimeDir/hooks-settings.json` | P0 | Daemon generates `hooks-settings.json` at `runtimeDir/hooks-settings.json` with mode `0o600` using `tempfile::persist`. File contains all 5 hook endpoint URLs. File is regenerated on daemon restart (new port). | F-12, F-62 |
| BC-2.04.011 | Bounded Event Bus with Drop Counter | P0 | The daemon uses `mpsc::channel(N)` (bounded, N defined at startup). When the channel is full, the oldest event is dropped and the drop counter increments by 1. Drop counter value is published to all TUI clients on each state push. | F-63, F-50 |
| BC-2.04.012 | JSONL Ring: Capacity and Rotation Policy | P0 | The ring buffer retains at most 100MB of JSONL data per rotation file. When a rotation file reaches 100MB, it is renamed `.1`/`.2`... up to 5 rotations; the oldest (`.5`) is deleted. RAM ring holds last N events in memory for instant TUI access. | F-13 |

### 3.2 SS-05: IPC (BC-2.05.NNN)

| Proposed BC ID | Title | Priority | Description | Brief Feature |
|----------------|-------|----------|-------------|---------------|
| BC-2.05.001 | UDS Server Bind at `runtimeDir/monocle.sock` | P0 | Daemon creates Unix domain socket at `runtimeDir/monocle.sock` mode `0o600` on startup. Socket is removed on graceful shutdown. If socket exists at startup (stale), it is removed before rebind. | F-24 |
| BC-2.05.002 | TUI Client Connects to UDS and Receives Initial State Push | P0 | When TUI connects to `monocle.sock`, daemon immediately pushes the current session list, ring tail, and any queued overlay stack. Connection uses length-prefixed framing. | F-24, F-26 |
| BC-2.05.003 | IPC Message Types: SessionListUpdate | P0 | Daemon pushes `SessionListUpdate` message to all connected TUI clients when the session roster changes (new session detected, session ended, session enriched). Message contains full current session list. | F-26 |
| BC-2.05.004 | IPC Message Types: HookEventReceived | P0 | Daemon pushes `HookEventReceived` message to all connected TUI clients when a hook event is ingested. Message contains hook type, session ID, payload excerpt, latency-ms. | F-26 |
| BC-2.05.005 | IPC Message Types: PermissionPromptQueued | P0 | Daemon pushes `PermissionPromptQueued` message to all connected TUI clients when a `PreToolUse` hook with `decision_required: true` is received. Message contains full payload for diff preview. | F-26 |
| BC-2.05.006 | TUI Reconnects After Daemon Restart | P0 | When daemon restarts (new port), TUI detects connection loss, re-reads lock file, connects to new UDS socket within 5 seconds. During reconnect window, TUI renders "reconnecting..." indicator. | F-27 |
| BC-2.05.007 | Overlay Stack Cleared on Daemon Disconnect (SOQ-3) | P0 | When the TUI loses the UDS connection unexpectedly, all entries in the `VecDeque<PromptModal>` overlay stack are cleared. Rationale: Claude Code subprocesses time out delayed responses; cleared prompts prevent ghost approvals. | F-28, F-39 |
| BC-2.05.008 | UDS-Only in Phase 1 (No Shared-Memory Transport) | P1 | The `monocle-ipc` crate implements only the UDS transport in Phase 1. No `mmap`, `shm_open`, or shared-memory primitives are used. Phase 4 shared-memory transport is a future extension per OQ-08. | F-25 |

### 3.3 SS-06: TUI (BC-2.06.NNN)

| Proposed BC ID | Title | Priority | Description | Brief Feature |
|----------------|-------|----------|-------------|---------------|
| BC-2.06.001 | AppMode State Machine: Compile-Time Mutual Exclusion | P0 | `AppMode` is an enum with variants `Dashboard`, `Filtering`, `Overlay`, `Fullscreen`. State transitions are pure functions in `monocle-core`: `fn transition(mode: AppMode, action: Action) -> AppMode`. No `Option<Panel>` fields; no `Arc<Mutex<...>>` in transition logic. | F-46, F-49 |
| BC-2.06.002 | FocusSnapshot: Focus Restored After Overlay/Fullscreen Close | P0 | When a `Overlay` or `Fullscreen` mode is entered, the prior `FocusSnapshot` is captured. When the overlay or fullscreen closes, `AppMode` restores to `Dashboard { focused: <prior snapshot> }`. | F-47 |
| BC-2.06.003 | Action Dispatch: 5-Level Binding Precedence | P0 | The keybinding dispatcher walks `SearchPrompt > UserCustomCommand > PerContext > Global > Builtin` and stops at the first matching binding. If no binding matches, the keypress is discarded. Dispatcher is deterministic (same key always resolves same action in same AppMode). | F-48 |
| BC-2.06.004 | `Ctrl-\` Popup: Appears and Dismisses Without State Loss | P0 | `Ctrl-\` makes the monocle TUI pane visible (via tmux `display-popup` or equivalent). Second `Ctrl-\` hides it. The TUI continues receiving IPC pushes from daemon while hidden. AppMode state is preserved across hide/show cycles. | F-33, F-38 |
| BC-2.06.005 | Sessions Panel: Session List Renders from IPC State | P0 | The sessions panel renders one row per `EnrichedSession` received via IPC `SessionListUpdate`. Each row shows: harness icon, project name, phase tag (if available), token count, cost, uptime. Empty state shows "No sessions detected". | F-29, F-32 |
| BC-2.06.006 | Sessions Panel: `/` Filter with Nucleo Fuzzy Match | P0 | Pressing `/` in sessions panel activates `Filtering` AppMode. Typed characters are sent to nucleo-matcher. Only sessions whose project name or harness name fuzzy-match the query are shown. `Esc` clears filter and returns to `Dashboard`. | F-30 |
| BC-2.06.007 | Sessions Panel: `Enter` Transitions to Fullscreen | P0 | Pressing `Enter` on a focused session row transitions AppMode to `Fullscreen { panel: Sessions, prior: Dashboard { focused: Sessions } }`. Fullscreen view shows session detail: token history, cost breakdown, hook event count. | F-31 |
| BC-2.06.008 | Permission Overlay: VecDeque Stack Push on PermissionPromptQueued | P0 | When the TUI receives `PermissionPromptQueued` IPC message, a `PromptModal` is pushed to the back of the `VecDeque<PromptModal>`. AppMode transitions to `Overlay { stack: [...], prior: <current focus> }`. The overlay badge in the status bar increments. | F-34, F-38 |
| BC-2.06.009 | Permission Overlay: `[↑↓]` Rotates Stack | P0 | In `Overlay` AppMode, `Action::OverlayCycleNext` rotates the `VecDeque`: the front `PromptModal` moves to the back, exposing the next queued prompt. The overlay renders the current front item. | F-40 |
| BC-2.06.010 | Permission Overlay: Diff Preview via `similar 3` | P0 | When the `PromptModal` payload contains `tool: Edit` with `old_content` and `new_content` fields, the overlay renders a unified diff computed via `similar::TextDiff`. Lines prefixed with `-` render in red; `+` in green. | F-35 |
| BC-2.06.011 | Permission Overlay: Accept-Once Keybinding | P0 | In `Overlay` AppMode, pressing `1` (or configured binding) triggers `Action::PermissionAcceptOnce`. The daemon sends `{"decision": "accept"}` to the waiting Claude Code hook response. The front `PromptModal` is popped from the stack. | F-36, F-41 |
| BC-2.06.012 | Permission Overlay: Accept-Always Keybinding | P0 | In `Overlay` AppMode, pressing `2` triggers `Action::PermissionAcceptAlways`. The daemon sends `{"decision": "always"}` to the hook response and records the pattern for future auto-accept. Front `PromptModal` popped. | F-36, F-41 |
| BC-2.06.013 | Permission Overlay: Reject Keybinding | P0 | In `Overlay` AppMode, pressing `3` triggers `Action::PermissionReject`. The daemon sends `{"decision": "deny"}` to the hook response. Front `PromptModal` popped. | F-36, F-41 |
| BC-2.06.014 | Permission Overlay: `[Esc]` Hides Without Rejecting | P0 | In `Overlay` AppMode, `Esc` hides the overlay (closes `Ctrl-\` popup) without popping any `PromptModal`. Prompts remain queued. Next `Ctrl-\` shows the overlay again with the same stack. | F-38, F-40 |
| BC-2.06.015 | Permission Overlay: `[t]` Trace-to-Source Stub | P1 | Pressing `[t]` in `Overlay` AppMode renders a "Trace to source: Phase 2 feature" placeholder message. Does NOT navigate. Stub is required in Phase 1 so the keybinding exists and is discoverable. | F-37 |
| BC-2.06.016 | Permission Overlay: Cleared on Daemon Disconnect | P0 | When the TUI receives a daemon disconnect notification (from IPC layer BC-2.05.007), the `VecDeque<PromptModal>` is cleared. AppMode transitions back to `Dashboard`. | F-39 |
| BC-2.06.017 | Permission Response Within Hook Timeout Budget | P0 | The time between TUI receiving `PermissionPromptQueued` and the daemon sending the decision response (after user keypress) must not stall the hook response beyond Claude Code's timeout. The daemon holds the HTTP response open until a decision is made OR the hook timeout (300ms for PreToolUse) is reached. On timeout, daemon returns fail-open or fail-closed per BC-HOOK-001/BC-HOOK-002. | F-41 |
| BC-2.06.018 | Event Ribbon Panel: Rolling Hook Event Log | P0 | The event ribbon panel renders the last N hook events received via IPC `HookEventReceived` messages. Each row shows: timestamp, hook type, session ID, latency-ms. New events prepend to the top. The panel scrolls via `Action::ScrollUp/ScrollDown`. | F-42, F-43, F-44 |
| BC-2.06.019 | Status Bar: Drop Counter Renders Under Load | P0 | The status bar renders the current drop counter value received from the daemon. Under synthetic 1000 events/sec load, the drop counter increments and renders. A drop counter of 0 renders as nothing (no visual clutter when healthy). | F-45, F-50 |
| BC-2.06.020 | Status Bar: Breadcrumb | P1 | The status bar renders the current AppMode as a breadcrumb string (e.g., "Dashboard > Sessions", "Dashboard > Sessions > Fullscreen", "Dashboard > Overlay [2 prompts]"). | F-51 |
| BC-2.06.021 | Status Bar: Keybinding Hint Line | P1 | The status bar renders a context-sensitive hint line showing the available actions for the current AppMode (e.g., "Tab: cycle  Enter: fullscreen  /: filter  ?: help  q: quit"). | F-52 |
| BC-2.06.022 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | P0 | Starting from nvim with 2 queued prompts: `Ctrl-\` (1), `2` (Accept-always prompt 1, 2), `1` (Accept-once prompt 2, 3), `Ctrl-\` (4) = 4 keystrokes. TUI returns to hidden state. Both Claude Code sessions unblock. No tmux window switches. | F-29, F-34, F-36, F-41 |

### 3.4 SS-07: Config (BC-2.07.NNN)

| Proposed BC ID | Title | Priority | Description | Brief Feature |
|----------------|-------|----------|-------------|---------------|
| BC-2.07.001 | Config File Atomic Write via `tempfile::persist` | P0 | All writes to `~/.monocle/config.json` use `tempfile::persist` (write to temp, rename atomically). Naked `std::fs::write` to config path is forbidden and enforced by semgrep rule. | F-53, F-58 |
| BC-2.07.002 | Config Schema Version 1: Harness Profile Fields | P0 | `config.json` schema includes: `schema_version: 1`, `harness_profiles: [...]` (array of `{id, display_name, binary_path, config_dir}`), `ccr_path: Option<String>`, `binding_overrides: {}` (empty stub). Unknown fields are ignored (forward-compat). | F-53, F-54, F-55, F-56 |
| BC-2.07.003 | Config Missing or Corrupted: Default Applied | P0 | If `~/.monocle/config.json` does not exist or fails JSON parse, `monocle-config` returns the built-in default config (empty `harness_profiles`, `ccr_path: None`, empty `binding_overrides`). No panic; no fatal error. | F-53 |
| BC-2.07.004 | Profile Picker: Sticky-Per-Project | P1 | When `monocle` starts for a project directory, it reads `config.json` for the last-used profile for that directory. If found, that profile is pre-selected without showing the picker. | F-57 |
| BC-2.07.005 | Profile Picker: `Ctrl-P` Override Shows Picker | P1 | Pressing `Ctrl-P` in any AppMode opens the profile picker overlay regardless of sticky selection. The user can change the active profile. The new selection is persisted to `config.json` for that project directory. | F-57 |
| BC-2.07.006 | CCR Detection via `ccr_path` Config Field | P1 | If `ccr_path` is set in `config.json`, `monocle` verifies the binary exists at that path before use. If `ccr_path` is None, `monocle` searches `PATH` for `ccr`. Detection result is surfaced in the TUI status bar. | F-55 |

---

## Section 4: Success Criteria Gap Closure

The following maps each Phase 1 Success Criterion from `product-brief.md §Success Criteria`
(lines 239-252) to the new BCs that would satisfy it, in addition to any existing BCs.

| Success Criterion | Current Coverage | Closing BCs (proposed) |
|-------------------|-----------------|------------------------|
| Session management: user can manage 3+ concurrent Claude Code sessions without leaving editor pane; killer scenario ≤6 keystrokes | BC-2.03.001 (HookDecision::Defer — necessary but not sufficient) | BC-2.06.004 (Ctrl-\ popup), BC-2.06.005 (sessions panel), BC-2.06.008 (overlay push), BC-2.06.009 (stack rotate), BC-2.06.011..013 (accept/reject), BC-2.06.022 (killer scenario E2E) |
| Permission prompt latency: ≤100ms from hook POST receipt to TUI overlay render on localhost | None | BC-2.04.007 (hook routing timing), BC-2.05.005 (PermissionPromptQueued IPC push), BC-2.06.008 (overlay push on receive) — the 100ms budget spans all three hops |
| Hook ingestion timeout budget: ≤300ms for PreToolUse/Stop/SessionStart/PromptSubmit; ≤2000ms for Notification | BC-2.01.003 (body limit enforces memory bound, not timing) | BC-2.04.007 (≤300ms routing), BC-2.04.008 (≤2000ms Notification routing), BC-2.06.017 (decision within budget) |
| Hook protocol parity: fixture-based parity test passes | BC-2.01.008, BC-2.01.009 (auth), BC-HOOK-001..BC-HOOK-041 (DTU) | BC-2.04.010 (hooks-settings.json generation) |
| Factory pattern detection: vsdd-factory detected on monocle's own `.factory/` | BC-2.02.004, BC-2.02.005 (FactoryAdapter trait + VsddFactoryAdapter self-referential test) | No new BCs needed — COVERED |
| Build matrix: CI green on darwin/linux × amd64/arm64 | NFR-007, NFR-008 | No new BCs needed — NFR-covered |
| Drop counter active: no unbounded channel; counter renders under 1000 events/sec | None | BC-2.04.011 (bounded bus + counter), BC-2.06.019 (counter renders in status bar) |
| Hook receiver body size limit: HTTP 413 on >256KiB | BC-2.01.003 | No new BCs needed — COVERED |
| DTU clone fidelity ≥0.95 | NFR-011, BC-HOOK-001..041 | BC-2.04.010 (hooks-settings.json generation — monocle side; completes DTU round-trip) |
| Forward-compatibility contracts: all 6 FC items | BC-2.02.001..008, BC-2.01.007..010 | No new BCs needed — COVERED |

---

## Section 5: Dependency Analysis

### 5.1 New Subsystem Dependencies on Existing Subsystems

```
SS-07 Config
  └── No runtime dependency on other new subsystems

SS-04 Daemon Wiring
  ├── depends on SS-01 (Daemon Lifecycle: HTTP server, auth, graceful shutdown)
  ├── depends on SS-02 (Core Types and ABI: FactoryAdapter, ABI version)
  ├── depends on SS-03 (Engine Module: EngineModule trait, ClaudeCodeModule)
  └── depends on SS-07 (Config: reads harness profiles, CCR path)

SS-05 IPC
  ├── depends on SS-04 (Daemon Wiring: UDS socket created at runtimeDir by daemon)
  └── depends on SS-03 (Engine Module: session data types come from EnrichedSession)

SS-06 TUI
  ├── depends on SS-05 (IPC: all TUI data arrives via IPC messages)
  ├── depends on SS-07 (Config: profile picker reads/writes config.json)
  └── depends on SS-04 (Daemon Wiring: daemon start triggered by TUI on first launch)
```

### 5.2 New BC Dependencies on Existing BCs

| New BC | Depends On (existing) | Nature of Dependency |
|--------|-----------------------|----------------------|
| BC-2.04.001 (Daemon Start Sequence) | BC-2.01.005 (Lock File Atomic Lifecycle) | SOQ-2 token ordering is a constraint on the BC-2.01.005 lock-file-write sequence |
| BC-2.04.001 (Daemon Start Sequence) | BC-2.01.008 (Auth Token Wire Format) | Token written during start must conform to `monocle-v1:<64-hex>` format |
| BC-2.04.007..009 (Hook Endpoint Routing) | BC-2.01.003 (Body Size Limit) | Body limit enforced before routing; must be checked first in middleware chain |
| BC-2.04.007..009 (Hook Endpoint Routing) | BC-2.01.009 (Auth Header Validation) | Auth middleware runs before routing; both headers accepted per ADR-0005 |
| BC-2.04.007..009 (Hook Endpoint Routing) | BC-2.03.001 (EngineModule Trait) | Routing dispatches to `EngineModule::on_hook()` |
| BC-2.04.010 (Hook Tmpfile) | BC-HOOK-009 (hooks-settings.json mode 0o600) | Monocle's file must match the expected path and mode from the gene-source contract |
| BC-2.04.011 (Bounded Event Bus) | BC-2.04.007..009 (Hook Routing) | Events produced by hook routing are the source for the event bus |
| BC-2.05.001 (UDS Bind) | BC-2.04.001 (Daemon Start Sequence) | UDS socket created as part of daemon start, after lock file written |
| BC-2.05.005 (PermissionPromptQueued) | BC-2.04.007 (PreToolUse Routing) | The PreToolUse hook routing produces the event that triggers the IPC push |
| BC-2.05.007 (Overlay Clear on Disconnect) | BC-2.06.016 (Overlay Cleared on Disconnect) | IPC layer signals disconnect; TUI BC responds to it |
| BC-2.06.008 (Overlay Push) | BC-2.05.005 (PermissionPromptQueued) | Overlay push is the TUI-side handler of the IPC message |
| BC-2.06.017 (Hook Timeout Budget) | BC-HOOK-022 (Notification 2000ms, Others 300ms) | Timeout values are the gene-source canonical ceilings |
| BC-2.06.017 (Hook Timeout Budget) | BC-HOOK-001 (PreToolUse Fail-Open) | On decision timeout, daemon must apply fail-open semantics from gene-source |
| BC-2.07.001 (Atomic Write) | (none — foundational) | Anti-pattern enforcement; no BC dependency |

### 5.3 Story Wave Assignment (Preliminary)

The new BCs suggest the following wave ordering, following the TDD dependency chain:

| Wave | New Subsystem Focus | Rationale |
|------|--------------------|-----------| 
| Wave 4 (next) | SS-04 (Daemon Wiring) + SS-07 (Config) | Daemon wiring is the integration layer over the existing SS-01..SS-03 code already shipped. Config is independent and small. Both can proceed in parallel. |
| Wave 5 | SS-05 (IPC) | Requires the daemon UDS socket (SS-04) to be in place. 8 BCs of moderate complexity. |
| Wave 6 | SS-06 TUI: AppMode, Sessions Panel, Event Ribbon, Status Bar | IPC must be complete so TUI has real data to render. State machine first, then panels. |
| Wave 7 | SS-06 TUI: Permission Overlay (VecDeque stack, diff preview, keybindings) | Most complex TUI feature; requires sessions panel to already work for context. |

---

## Section 6: Summary Metrics

| Metric | Value |
|--------|-------|
| Phase 1 features identified | 77 |
| Currently COVERED | 29 (38%) |
| Currently PARTIAL | 10 (13%) |
| Currently MISSING | 38 (50%) |
| Existing BCs (Phase 1 PRD) | 22 |
| Existing DTU BCs | 41 |
| Proposed new BCs (SS-04) | 12 |
| Proposed new BCs (SS-05) | 8 |
| Proposed new BCs (SS-06) | 22 |
| Proposed new BCs (SS-07) | 6 |
| **Total proposed new BCs** | **48** |
| **Projected total Phase 1 PRD BCs** | **70** |
| New subsystems proposed | 4 (SS-04 through SS-07) |

---

## §Trace v1.0

**Initial production** (2026-05-27T00:00:00Z):
- PRD expansion scope document created as a standalone planning artifact.
- Reads: product-brief.md v1.4.30, prd.md v1.26.15, BC-INDEX.md v1.14, ARCH-INDEX.md v1.0.11, vision-synthesis v1.1.3.
- Identified 77 Phase 1 features; 38 (50%) have zero BC coverage.
- Proposed 4 new subsystems (SS-04 through SS-07) with 48 new BCs.
- Does NOT create BC files. Does NOT modify the existing PRD. Awaits human review.
- Path note: this document is written to `.factory/specs/prd-expansion-scope.md` per
  explicit task instruction. The artifact-path-registry.yaml classifies `plan-doc` under
  `.factory/plans/`. If the orchestrator wants this relocated to `.factory/plans/`, use the
  `vsdd-factory:relocate-artifact` skill before creating the BC files.
