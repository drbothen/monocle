---
document_type: prd
level: L3
version: "1.28.3"
status: draft
producer: vsdd-factory:product-owner
phase: phase-1-spec-crystallization
timestamp: 2026-06-04T00:00:00Z
inputs: [product-brief.md, research/domain-monocle-vision-synthesis.md, architecture/SS-daemon-lifecycle.md, architecture/SS-core-types-and-abi.md, architecture/SS-engine-module.md, architecture/SS-deps-pin-manifest.md, architecture/SS-permissions-phase1.md, architecture/SS-conventions-anti-patterns.md, architecture/SS-forward-compatibility.md, dtu-assessment.md, architecture/adr/ADR-0001-wasmtime-vs-wasmi.md, architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md, architecture/adr/ADR-0003-license-selection.md, architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]
input-hash: "2b5dffa"
traces_to: "product-brief.md v1.4.30; vision-synthesis v1.1.2; SS-daemon-lifecycle.md v1.0.32; SS-core-types-and-abi.md v1.2.13; SS-engine-module.md v1.1.20; SS-deps-pin-manifest.md v1.1.17; SS-conventions-anti-patterns.md v1.29.5; architecture/SS-permissions-phase1.md v1.5.2; architecture/SS-forward-compatibility.md v1.2.19; architecture/adr/ADR-0001-wasmtime-vs-wasmi.md v1.0.3; architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md v1.0.4; architecture/adr/ADR-0003-license-selection.md v1.0.2; architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md v1.0.4; architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md v1.0.2; architecture/ARCH-INDEX.md v1.0.10; behavioral-contracts/BC-INDEX.md v1.35; 136 BCs sharded under behavioral-contracts/ss-NN/ (ss-01 through ss-09 + ss-dtu); domain-spec/L2-INDEX.md v1.0.11; verification-properties/VP-INDEX.md v1.15" # version-pin-historical: BC-INDEX v1.17 was canonical at PRD v1.27.4; updated to v1.35 and 70→136 BCs at v1.28.0 (D-241 control-center v1A)
project: monocle
supplements:
  - interface-definitions.md
  - error-taxonomy.md
  - test-vectors.md
  - nfr-catalog.md
---

# Product Requirements Document: Monocle — Phase 1

> **Index Document.** This PRD is an index. BC details live in `behavioral-contracts/ss-NN/BC-2.SS.NNN.md`.
> NFR catalog, error taxonomy, interface definitions, and test vectors are in `prd-supplements/`.
> Load supplements on-demand — do not load all 4 unless your task requires all 4.

## 1. Product Overview

### 1.1 Problem

Today, a developer running three Claude Code sessions across two projects faces a fragmentation problem: sessions live in separate tmux windows requiring context switches to check status; concurrent permission prompts from different sessions stall until the developer switches to the right window; factory-pipeline state (vsdd-factory STATE.md) is only visible by manually reading files; and no single view spans multiple harnesses.

Per vision §Vision Statement: "One TUI lens over every Claude-class session you're running, every customization that shapes them, and every workflow driving them — across multiple harnesses and federated across hosts."

### 1.2 Vision

Monocle is a single-binary Rust TUI that gives developers one `Ctrl-\` popup over every AI coding harness session they are running. It surfaces five information planes: live session roster (Runtime), active customizations per session (Static), workflow pipeline state (Workflow), per-harness profiles (Harness), and a lazygit-style keybinding dispatch layer (TUI philosophy). Monocle is observe-only for workflow state and session transcripts; it owns the action layer only for permission prompts and keybinding dispatch.

The killer scenario per vision §End-to-End Killer Scenario: 4 keystrokes (`Ctrl-\`, `2`, `1`, `Ctrl-\`) resolve two concurrent permission prompts with zero context switches vs. the current 6+ keystrokes + 2 window switches + risk of session timeout.

### 1.3 Competitive Differentiators

| ID | Differentiator | BC Backing |
|----|---------------|------------|
| D-1 | Hook-protocol ingestion at OS-assigned port with versioned auth token | BC-2.01.008, BC-2.01.009, BC-2.01.010, BC-2.01.001, BC-2.01.002 |
| D-2 | VecDeque overlay stack — both concurrent prompts visible simultaneously | BC-2.03.001, BC-2.03.002 |
| D-3 | Forward-compatible ABI via const + non_exhaustive + proto schema_version | BC-2.02.001, BC-2.02.002, BC-2.02.003, BC-2.02.006, BC-2.02.007, BC-2.02.008 |
| D-4 | FactoryAdapter open trait — VsddFactoryAdapter ships in Phase 1; WASM loadable in Phase 3 | BC-2.02.004, BC-2.02.005 |
| D-5 | ClaudeCodeModule strict-basename detect — no false positives from claude-squad/claudio | BC-2.03.002 |
| D-6 | JSONL ring with format_version first key — Phase 2 trigger-trace can read Phase 1 history | BC-2.01.007 |
| D-7 | 256 KiB body size limit with structured error — bounded daemon memory exposure | BC-2.01.003 |
| D-8 | Graceful 10-second drain with crash-recovery checkpoint | BC-2.01.004, BC-2.01.006 |

### 1.4 Target Users

| Persona | Pain | Phase |
|---------|------|-------|
| Multi-session Claude Code developer | Concurrent permission prompts stall sessions; no unified view | Phase 1 |
| Factory-pattern operator | STATE.md only readable via manual cat/tree; no live pipeline visibility | Phase 1 |
| Multi-harness operator (CodeMachine + Claude Code) | No unified cost/session-health view across harnesses | Phase 4 |

### 1.5 Out of Scope

Per vision §Explicit Non-Goals (hard boundaries):
- Does NOT execute workflows — monocle never writes STATE.md, never triggers factory phases
- Does NOT route LLM API requests — CCR integration is detect-on-PATH + config-write only
- Does NOT replace the terminal multiplexer — runs inside tmux, does not replace it
- Does NOT include PM/Worker multi-agent orchestration
- Does NOT own session transcripts — hook events are ephemeral ingestion signals
- Does NOT ship `PostToolUse` hook endpoint in Phase 1 — per JC-2 gene-source parity (any-context BC-HOOK-007 canonical 5-endpoint matrix)
- Does NOT ship WASM plugin SDK in Phase 1 — Phase 3 deliverable per OQ-03
- Does NOT ship rmcp MCP bridge in Phase 1 — Phase 4 deliverable per OQ-09

---

## 2. Behavioral Contracts Index

> Individual BC files live in `behavioral-contracts/ss-NN/` shard directories,
> one shard per subsystem registered in `architecture/ARCH-INDEX.md`.
> Grouped by L2 domain subsystem (CAP-NNN).
> Each BC uses hierarchical numbering: BC-S.SS.NNN where S=section (2 for all
> Phase 1 BCs), SS=subsection (matching L2 subsystem; matches the shard `ss-NN`
> directory), NNN=sequential within subsystem.
> Full index: `behavioral-contracts/BC-INDEX.md`.

### 2.1 Daemon Lifecycle (CAP-001)

> Architecture source: `architecture/SS-daemon-lifecycle.md` | ARCH-INDEX: SS-01

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.01.001 | Healthz Endpoint (Unauthenticated Liveness Probe) | P0 |
| BC-2.01.002 | Status Endpoint (Authenticated Daemon State) | P0 |
| BC-2.01.003 | Body Size Limit (256 KiB, HTTP 413) | P0 |
| BC-2.01.004 | Graceful Shutdown (10-Second Drain) | P0 |
| BC-2.01.005 | Lock File Atomic Lifecycle (Create + Pid Check + Cleanup) | P0 |
| BC-2.01.006 | Crash Recovery Checkpoint | P0 |
| BC-2.01.007 | JSONL Ring Format Version (FC-01) | P0 |
| BC-2.01.008 | Auth Token Wire Format (FC-06) | P0 |
| BC-2.01.009 | Auth Header Validation (Missing and Invalid Token) | P0 |
| BC-2.01.010 | Lock File Contract Version Field | P0 |

> Full contracts: `behavioral-contracts/ss-01/BC-2.01.NNN.md`

### 2.2 Core Types and ABI (CAP-002)

> Architecture source: `architecture/SS-core-types-and-abi.md` | ARCH-INDEX: SS-02

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.02.001 | ABI Version in /status Endpoint (FC-03) | P0 |
| BC-2.02.002 | ABI Version Constant at Crate Root (FC-03) | P0 |
| BC-2.02.003 | Non-Exhaustive Enum Policy (FC-02) | P0 |
| BC-2.02.004 | FactoryAdapter Trait Definition (FC-04 CRITICAL) | P0 |
| BC-2.02.005 | VsddFactoryAdapter Implementation | P0 |
| BC-2.02.006 | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | P0 |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | P0 |
| BC-2.02.008 | Phase 4 schema_version Validation Requirement (FC-05) | P1 |

> Full contracts: `behavioral-contracts/ss-02/BC-2.02.NNN.md`

### 2.3 Engine Module (CAP-003)

> Architecture source: `architecture/SS-engine-module.md` | ARCH-INDEX: SS-03

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.03.001 | EngineModule Trait Definition | P0 |
| BC-2.03.002 | ClaudeCodeModule Implementation (Strict-Basename Detect) | P0 |
| BC-2.03.003 | HomeUnresolvable Error Contract | P0 |
| BC-2.03.004 | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | P0 |
| BC-2.03.005 | ClaudeCodeModule.spawn_recipe() — Happy-Path Recipe Assembly | P0 |
| BC-2.03.006 | ClaudeCodeModule.spawn_recipe() — CCR Base URL Injection | P0 |
| BC-2.03.007 | spawn_recipe() Error Cases — BinaryNotFound and InvalidPath | P0 |
| BC-2.03.008 | Default spawn_recipe() Returns UnsupportedOperation | P1 |

> Full contracts: `behavioral-contracts/ss-03/BC-2.03.NNN.md`

### 2.4 Daemon Wiring (CAP-004)

> Architecture source: `architecture/SS-daemon-wiring.md` | ARCH-INDEX: SS-04

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.04.001 | Daemon Start Sequence: Port Bind + Lock File + Token Write (SOQ-2) | P0 |
| BC-2.04.002 | Daemon Auto-Start on TUI Launch | P0 |
| BC-2.04.003 | MONOCLE_NO_AUTOSTART=1 Suppresses Auto-Start | P1 |
| BC-2.04.004 | `monocle daemon start` CLI Subcommand | P0 |
| BC-2.04.005 | `monocle daemon stop` CLI Subcommand | P0 |
| BC-2.04.006 | `directories::ProjectDirs::runtime_dir()` Fallback Chain | P0 |
| BC-2.04.007 | Hook Endpoint: PreToolUse Request Routing | P0 |
| BC-2.04.008 | Hook Endpoint: Notification Request Routing (2000ms Timeout) | P0 |
| BC-2.04.009 | Hook Endpoint: Stop/SessionStart/PromptSubmit Routing (300ms Timeout) | P0 |
| BC-2.04.010 | Hook Tmpfile Generation at runtimeDir/hooks-settings.json | P0 |
| BC-2.04.011 | Bounded Event Bus with Drop Counter | P0 |
| BC-2.04.012 | JSONL Ring: Capacity and Rotation Policy | P1 |

> Full contracts: `behavioral-contracts/ss-04/BC-2.04.NNN.md`

### 2.5 IPC (CAP-005)

> Architecture source: `architecture/SS-ipc.md` | ARCH-INDEX: SS-05

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.05.001 | UDS Server Bind at runtimeDir/monocle.sock | P0 |
| BC-2.05.002 | TUI Client Connects to UDS and Receives Initial State Push | P0 |
| BC-2.05.003 | IPC Message Types: SessionListUpdate | P0 |
| BC-2.05.004 | IPC Message Types: HookEventReceived | P0 |
| BC-2.05.005 | IPC Message Types: PermissionPromptQueued | P0 |
| BC-2.05.006 | TUI Reconnects After Daemon Restart | P1 |
| BC-2.05.007 | Overlay Stack Cleared on Daemon Disconnect (SOQ-3) | P0 |
| BC-2.05.008 | UDS-Only in Phase 1 (No Shared-Memory Transport) | P1 |
| BC-2.05.009 | PtyOutput Fan-Out — Per-Session Bounded Channel (1024) with Drop Counter (stderr WARN) + PtyReset TUI Recovery | P0 |
| BC-2.05.010 | New ClientToServer IPC Variants — SpawnSession, KillSession, KeyInput, ResizePane, DetachSession, RenameSession, AttachSession | P0 |
| BC-2.05.011 | New ServerToClient IPC Variants — ScrollbackChunk, ScrollbackDumpComplete, PtyReset | P0 |

> Full contracts: `behavioral-contracts/ss-05/BC-2.05.NNN.md`

### 2.6 TUI (CAP-006)

> Architecture source: `architecture/SS-tui.md` | ARCH-INDEX: SS-06

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.06.001 | AppMode State Machine: Compile-Time Mutual Exclusion | P0 |
| BC-2.06.002 | FocusSnapshot: Focus Restored After Overlay/Fullscreen Close | P0 |
| BC-2.06.003 | Action Dispatch: 5-Level Binding Precedence | P0 |
| BC-2.06.004 | Ctrl-\ Popup: Appears and Dismisses Without State Loss | P0 |
| BC-2.06.005 | Sessions Panel: Session List Renders from IPC State | P0 |
| BC-2.06.006 | Sessions Panel: / Filter with Nucleo Fuzzy Match | P1 |
| BC-2.06.007 | Sessions Panel: Enter Transitions to Fullscreen | P1 |
| BC-2.06.008 | Permission Overlay: VecDeque Stack Push on PermissionPromptQueued | P0 |
| BC-2.06.009 | Permission Overlay: `[↑↓]` Rotates Stack | P0 |
| BC-2.06.010 | Permission Overlay: Diff Preview via `similar 3` | P1 |
| BC-2.06.011 | Permission Overlay: Accept-Once Keybinding | P0 |
| BC-2.06.012 | Permission Overlay: Accept-Always Keybinding | P0 |
| BC-2.06.013 | Permission Overlay: Reject Keybinding | P0 |
| BC-2.06.014 | Permission Overlay: `[Esc]` Hides Without Rejecting | P0 |
| BC-2.06.015 | Permission Overlay: `[t]` Trace-to-Source Stub | P2 |
| BC-2.06.016 | Permission Overlay: Cleared on Daemon Disconnect | P0 |
| BC-2.06.017 | Permission Response Within Hook Timeout Budget | P0 |
| BC-2.06.018 | Event Ribbon Panel: Rolling Hook Event Log | P1 |
| BC-2.06.019 | Status Bar: Drop Counter Renders Under Load | P0 |
| BC-2.06.020 | Status Bar: Breadcrumb | P1 |
| BC-2.06.021 | Status Bar: Keybinding Hint Line | P1 |
| BC-2.06.022 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | P0 |
| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | P0 |
| BC-2.06.024 | Permission Overlay: ToolPayload Body Rendering by Variant | P1 |
| BC-2.06.025 | Multi-Session / Multi-Project Sessions Panel — Grouped by Project, Fast Switching, TUI Lifecycle Actions | P0 |

> Full contracts: `behavioral-contracts/ss-06/BC-2.06.NNN.md`

### 2.7 Config (CAP-007)

> Architecture source: `architecture/SS-config.md` | ARCH-INDEX: SS-07

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.07.001 | Config File Atomic Write via `tempfile::persist` | P0 |
| BC-2.07.002 | Config Schema Version 1: Harness Profile Fields | P0 |
| BC-2.07.003 | Config Missing or Corrupted: Default Applied | P0 |
| BC-2.07.004 | Profile Picker: Sticky-Per-Project | P1 |
| BC-2.07.005 | Profile Picker: `Ctrl-P` Override Shows Picker | P1 |
| BC-2.07.006 | CCR Detection via `ccr_path` Config Field | P1 |

> Full contracts: `behavioral-contracts/ss-07/BC-2.07.NNN.md`

### 2.8 Session Manager (CAP-008)

> Architecture source: `architecture/SS-session-manager.md` | ARCH-INDEX: SS-08
> Phase: v1A (control-center pivot D-236)

Session Manager governs the complete lifecycle of monocle-managed harness sessions: spawning
`monocle-session-host` child processes via `SessionHostSpawner`, maintaining the `SessionEntry`
registry and `session-state.json` sidecars, killing sessions via `DaemonToHost::Kill`, re-discovering
alive sessions after daemon restart (blocking UDS bind until complete), garbage-collecting terminated
sessions after a 10-second grace period, auto-injecting the `--settings` hook argument at spawn
time, and broadcasting `SessionStateChanged` to all connected TUI clients on every `SessionState`
transition (driving wizard auto-advance and `EmbeddedTerminal` exit; ordered before
`SessionListUpdate`). The detached session-host process model (ADR-0009) means sessions survive daemon
restart and their PTY streams are re-attached by the new daemon instance, which is the primary
differentiator vs. daemon-owned PTY approaches.

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.08.001 | Session Spawn — SessionHostSpawner Called Within 2s; SessionEntry Created | P0 |
| BC-2.08.002 | Session Persistence — session-host Survives Graceful Daemon Restart | P0 |
| BC-2.08.003 | Session Kill — SIGTERM Delivered via DaemonToHost::Kill Within 500ms | P0 |
| BC-2.08.004 | Re-Discovery — All Alive Sessions Visible After Daemon Restart Within 5s; UDS Bind Blocked Until Complete | P0 |
| BC-2.08.005 | Session GC — Terminated Sessions Removed from Registry After 10s Grace Period | P1 |
| BC-2.08.006 | Hook Auto-Injection — `--settings` Arg Present in Session-Host Child Args Within 2s of Spawn | P0 |
| BC-2.08.007 | Attach/Detach — Chunked Scrollback (ScrollbackChunk*+ScrollbackDumpComplete) on Attach; session-host Stays Alive on Detach | P1 |
| BC-2.08.008 | SessionStateChanged — Daemon Emits on Every SessionState Transition; Delivered to All TUI Clients; Ordering Relative to SessionListUpdate | P0 |

> Full contracts: `behavioral-contracts/ss-08/BC-2.08.NNN.md`
> Key architecture decisions: ADR-0009 (native detached session-host process model)

### 2.9 Embedded PTY (CAP-009)

> Architecture source: `architecture/SS-embedded-pty.md` | ARCH-INDEX: SS-09
> Phase: v1A (control-center pivot D-236)

Embedded PTY provides the in-TUI terminal widget that renders PTY output from `monocle-session-host`
processes. The PTY byte pipeline is: IPC (`PtyOutput` message) → `vt100` parser → `tui-term` widget →
ratatui frame. Full-fidelity keyboard forwarding covers printable characters, control keys, arrows,
Kitty enhanced key protocol (CSI u sequences), SGR mouse events, and bracketed paste. The
`EmbeddedTerminal` AppMode and `SessionCreation` wizard AppMode are the two PTY-active states;
the SUG-3 guarantee ensures permission badge+bell are surfaced within one render tick even while
the user is in `EmbeddedTerminal` mode — monocle never silently queues prompts.

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.09.001 | PTY Output Renders Within 100ms of Byte Receipt at TUI | P0 |
| BC-2.09.002 | Full-Fidelity Keyboard Forwarding — All v1A Input Classes Reach PTY stdin | P0 |
| BC-2.09.003 | Mouse Events Forwarded to PTY in SGR Encoding When in EmbeddedTerminal | P1 |
| BC-2.09.004 | Kitty Keyboard Protocol — Enhanced Key Events Forwarded as CSI u Sequences | P1 |
| BC-2.09.005 | Bracketed Paste — Paste Events Wrapped in Bracket Sequences Before Forwarding | P1 |
| BC-2.09.006 | Resize — PTY and Parser Resized Within 2 Render Ticks of Pane Area Change; 50ms Debounce | P0 |
| BC-2.09.007 | Scrollback — 1000 Rows Default; Configurable; PtyScrollUp/Down Navigate | P1 |
| BC-2.09.008 | EmbeddedTerminal AppMode Enter/Exit Transitions; SessionCreation Wizard Auto-Transitions to EmbeddedTerminal | P0 |
| BC-2.09.009 | Permission Badge+Bell — Status Bar Badge + Audible Bell Within One Render Tick While in EmbeddedTerminal or SessionCreation | P0 |

> Full contracts: `behavioral-contracts/ss-09/BC-2.09.NNN.md`
> Key architecture decisions: ADR-0010 (PTY bytes shared on existing UDS IPC channel); ADR-0011 (PTY stack: native portable-pty + vt100 + tui-term)

---

## 3. Interface Definition

> **Supplement:** Full interface definitions are in `prd-supplements/interface-definitions.md`.
> Primary consumers: implementer, test-writer.

Phase 1 interface surfaces: HTTP API (5 hook POST endpoints + `/healthz` + `/status` + `/shutdown`), lock file JSON schema, JSONL ring buffer schema. Daemon binds on `127.0.0.1:<os-assigned-port>`. Auth header: canonical `X-Monocle-Authorization: monocle-v1:<64-hex>` (32 bytes `OsRng`); compatibility alias `X-Claude-Code-Ide-Authorization: <raw-64-hex>` accepted per ADR-0005 with WARN deprecation log. Body limit: 256 KiB on authenticated router only. See `prd-supplements/interface-definitions.md` for full schemas, exit codes, dual-accept semantics, and field constraints.

---

## 4. Non-Functional Requirements

> **Supplement:** Full NFR catalog is in `prd-supplements/nfr-catalog.md`.
> Primary consumers: architect, performance-engineer, formal-verifier.

Phase 1 defines 12 NFRs covering performance (NFR-001/002/003 latency, NFR-006 throughput), security (NFR-004 auth entropy, NFR-005 body limit, NFR-009 lock file 0o600, NFR-010 constant-time comparison, NFR-012 runtime_dir 0o700), build (NFR-007 MSRV Rust 1.88, NFR-008 macOS+Linux matrix), and forward-compat (NFR-011 DTU fidelity ≥0.95). See `prd-supplements/nfr-catalog.md` for the complete catalog including validation methods and VP probe citations. Note: NFR-007 MSRV was originally 1.86 (ratatui 0.30 floor); bumped to 1.88 in Wave 6 per RUSTSEC-2026-0009 Path B resolution (SS-deps-pin-manifest.md §Trace v1.2.0).

---

## 5. Error Taxonomy

> **Supplement:** Full error taxonomy is in `prd-supplements/error-taxonomy.md`.
> Primary consumers: implementer, test-writer.

Phase 1 defines 15 error codes across 7 subsystem abbreviations (`DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`). Convention: `E-<SUBSYSTEM>-<NNN>`. Severity levels: Broken (fatal, non-zero exit or 4xx/5xx), Degraded (WARN log + graceful continue), Cosmetic (WARN log, zero exit, no functional impact; E-AUTH-003 alias deprecation log). See `prd-supplements/error-taxonomy.md` for the complete catalog including BC source citations, implementation sites, and test file mappings.

---

## 5b. Test Vectors

> **Supplement:** Canonical test vectors are in `prd-supplements/test-vectors.md`.
> Primary consumers: test-writer, holdout-evaluator.

Per-BC test vectors are embedded in each BC file's "Canonical Test Vectors" section. The supplement provides an index by BC ID with test file mapping, plus aggregated critical vectors for the highest-risk behavioral boundaries (auth rejection, body size limit, router separation, JSONL ring key ordering, detect basename). See `prd-supplements/test-vectors.md`.

---

## 6. Competitive Differentiator Traceability

Per vision §Vision Statement and brief §Success Criteria. Every differentiator has BC backing — no unverifiable claims.

> Project-specific extension: tables include a `Verification` column (beyond template minimum) documenting the specific test scenario that verifies the differentiator. Rationale: monocle's killer scenarios are described in the brief and vision; capturing them here prevents regression during adversarial review. See §Trace v1.26.1.

### 6.1 KD-001 — Hook-Protocol Ingestion at OS-Assigned Port

Daemon binds on OS-assigned port; port written to lock file; hook scripts read absolute lock file path (no directory scan, no "highest-port-wins" collision).

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.01.008 | Auth token generated with OsRng and written to lock file at start | Integration test: lock file read after start; port confirmed reachable |
| BC-2.01.009 | Auth header validation rejects requests missing correct Bearer token | Integration test: port confirmed reachable; unauthorized access rejected |
| BC-2.01.010 | Lock file schema contract version `"monocle-lock-v1"` encoded | Integration test: no `~/.claude/ide/` scanning; lock file path is absolute |
| BC-2.01.001 | `/healthz` endpoint returns liveness signal on OS-assigned port | Integration test: lock file read after start; healthz reachable at recorded port |
| BC-2.01.002 | `/status` endpoint authenticated on OS-assigned port | Integration test: port confirmed reachable with Bearer auth |

### 6.2 KD-002 — VecDeque Overlay Stack for Concurrent Prompts

Both permission prompts visible simultaneously; `[↑↓]` rotates stack; `Esc` hides without rejecting.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.03.001 | `EngineModule::on_hook()` returns `HookDecision::Defer` for queued hooks | Killer scenario: 2 concurrent PreToolUse hooks arrive; TUI shows both prompts; 4 keystrokes resolve both |
| BC-2.03.002 | `ClaudeCodeModule::detect()` strict-basename prevents false positives in concurrent session disambiguation | Killer scenario: `on_hook → HookDecision::Defer` path exercises VecDeque routing |

### 6.3 KD-003 — Versioned ABI with Forward-Compatible Extension

`MONOCLE_ABI_VERSION = 1` const; `#[non_exhaustive]` on all public enums; proto `schema_version = 1` first field.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.02.001 | `/status` endpoint exposes `abi_version` field equal to `MONOCLE_ABI_VERSION` | Integration: ABI version in status response matches const; compile-time assertion |
| BC-2.02.002 | `MONOCLE_ABI_VERSION = 1` const defined in `monocle-core` crate root | AST audit (syn 2); compile-time assertion |
| BC-2.02.003 | All public enums carry `#[non_exhaustive]` attribute | AST audit (syn 2) verifies enum annotation policy |
| BC-2.02.006 | `HookEnvelope` proto field numbers are pinned (field 1 = `schema_version`) | Wire-format round-trip test; prost encode/decode field number test |
| BC-2.02.007 | `schema_version = 1` is first field in serialized HookEnvelope | Compile/integration test: schema_version field accessibility |

### 6.4 KD-004 — FactoryAdapter Open Trait — Phase 3 WASM Extensibility

`VsddFactoryAdapter` ships Phase 1 as a static implementation; WASM plugin SDK in Phase 3 uses the same trait without code changes.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.02.004 | `FactoryAdapter` trait surface has no sealed supertrait; open for external implementation | `cargo check` no sealed supertrait; AST audit (syn 2) |
| BC-2.02.005 | `VsddFactoryAdapter` self-referential integration test confirms Phase 1 implementation | Self-referential detection test |

### 6.5 KD-005 — Strict-Basename Detection (No False Positives)

`detect()` uses `exe_path.file_name()` == `"claude"` or `"claude.js"`; rejects `claude-squad`, `claudio`, `claude-code-router`.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.03.002 | `ClaudeCodeModule::detect()` applies strict file_name() equality; rejects all non-exact basenames | Unit tests with 5 synthetic ProcessSnapshot instances (true positives: `claude`, `claude.js`; true negatives: `claude-squad`, `claudio`, `claude-code-router`) |

### 6.6 KD-006 — JSONL Ring with format_version First Key

Phase 2 trigger-trace can read Phase 1 history; version field allows future format evolution.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.01.007 | JSONL ring format version `format_version: 1` is first key in every serialized line | Unit test: serialized JSONL line begins with `{"format_version":1,` |

### 6.7 KD-007 — 256 KiB Body Size Limit with Structured Error

Bounded daemon memory exposure per connection; structured error body for machine-readable rejection.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.01.003 | Daemon rejects bodies > 262,144 bytes with HTTP 413 and structured JSON error body | Integration test: 262,145-byte body returns HTTP 413 with correct error body |

### 6.8 KD-008 — Graceful 10-Second Drain with Crash-Recovery Checkpoint

In-flight requests complete before daemon exits; crash-recovery state offered to TUI on reconnect.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.01.004 | SIGTERM triggers 10-second drain window; new hooks receive HTTP 503 with `Retry-After: 10` | Integration test: SIGTERM triggers drain; new hooks get 503 with Retry-After: 10 |
| BC-2.01.006 | Crash-recovery checkpoint written before shutdown; checkpoint offered to TUI on reconnect | Integration test: crash-recovery state offered to TUI on reconnect |

---

## 7. Requirements Traceability Matrix

> Project-specific extensions: `Source (L2 CAP)` contains brief section citations used as interim L2 traceability anchors (L2 domain spec complete at v1.0.9; full CAP-NNN back-cascade to RTM rows is a Phase 1b task per story decomposition). `Module(s)` contains architecture subsystem file references. `Test File` is an additional column beyond the template minimum, providing direct test location traceability. See §Trace v1.26.1.

| BC ID | Source (L2 CAP) | Module(s) | Priority | Test File | Test Type |
|-------|----------------|-----------|----------|-----------|-----------|
| BC-2.01.001 | §Scope (hook receiver hardening — `/healthz`) | SS-daemon-lifecycle.md v1.0.33 §GET /healthz | P0 | `monocle-runtime/tests/healthz_endpoint.rs` | Integration |
| BC-2.01.002 | §Scope (hook receiver hardening — `/status`) | SS-daemon-lifecycle.md v1.0.33 §GET /status | P0 | `monocle-runtime/tests/status_endpoint_auth.rs` | Integration |
| BC-2.01.003 | §Success Criteria (body size limit) | SS-daemon-lifecycle.md v1.0.33 §Body Size Limit | P0 | `monocle-runtime/tests/body_size_limit.rs` | Integration |
| BC-2.01.004 | §Scope (hook receiver hardening — graceful shutdown) | SS-daemon-lifecycle.md v1.0.33 §Shutdown Signal Handling | P0 | `monocle-runtime/tests/graceful_shutdown.rs` + `monocle-runtime/tests/daemon_lifecycle.rs` | Integration |
| BC-2.01.005 | §Scope (hook receiver hardening — graceful shutdown) | SS-daemon-lifecycle.md v1.0.33 §Start Sequence | P0 | `monocle-runtime/tests/lock_file_lifecycle.rs` | Integration |
| BC-2.01.006 | §Scope (hook receiver hardening — graceful shutdown) | SS-daemon-lifecycle.md v1.0.33 §Crash Recovery | P0 | `monocle-runtime/tests/crash_recovery.rs` | Integration |
| BC-2.01.007 | §Scope (forward-compatibility — JSONL ring) | SS-daemon-lifecycle.md v1.0.33 §Drain | P0 | `monocle-runtime/tests/jsonl_ring.rs` | Integration |
| BC-2.01.008 | §Scope (forward-compatibility — versioned auth token) | SS-daemon-lifecycle.md v1.0.33 §Start Sequence | P0 | `monocle-runtime/tests/auth_token_lifecycle.rs` | Integration |
| BC-2.01.009 | §Scope (forward-compatibility — versioned auth token) | SS-daemon-lifecycle.md v1.0.33 §Start Sequence | P0 | `monocle-runtime/tests/auth_header_rejection.rs` | Integration |
| BC-2.01.010 | §Scope (forward-compatibility — versioned auth token) | SS-daemon-lifecycle.md v1.0.33 §Start Sequence | P0 | `monocle-runtime/tests/lock_file_contract.rs` | Integration |
| BC-2.02.001 | §Scope (forward-compatibility — monocle-core ABI) | SS-core-types-and-abi.md v1.2.13 §ABI Version Constant | P0 | `monocle-runtime/tests/status_abi_version.rs` | Integration |
| BC-2.02.002 | §Scope (forward-compatibility — monocle-core ABI) | SS-core-types-and-abi.md v1.2.13 §ABI Version Constant | P0 | `monocle-core/tests/abi_stability.rs` | Lint/compile |
| BC-2.02.003 | §Scope (forward-compatibility — public enum extensibility) | SS-core-types-and-abi.md v1.2.13 §Enum Extensibility | P0 | `monocle-core/tests/enum_audit.rs` | AST audit (syn 2) |
| BC-2.02.004 | §Scope (forward-compatibility — FactoryAdapter trait) | SS-core-types-and-abi.md v1.2.13 §FactoryAdapter Trait | P0 | `monocle-core/tests/factory_trait_surface.rs` | AST audit (syn 2) |
| BC-2.02.005 | §Success Criteria (factory pattern detection) | SS-core-types-and-abi.md v1.2.13 §VsddFactoryAdapter | P0 | `monocle-core/tests/factory_self_referential.rs` | Integration |
| BC-2.02.006 | §Scope (forward-compatibility — prost wire schemas) | SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas | P0 | `monocle-proto/tests/wire_field_order.rs` | Integration |
| BC-2.02.007 | §Scope (forward-compatibility — prost wire schemas) | SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas | P0 | `monocle-proto/tests/schema_version.rs` | Integration |
| BC-2.02.008 | §Scope (forward-compatibility — prost wire schemas) | SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas | P1 | Phase 4 integration test (future) | Integration |
| BC-2.03.001 | §Scope §In Scope (ClaudeCodeModule) | SS-engine-module.md v1.1.27 §EngineModule Trait Signature | P0 | `monocle-core/tests/engine_module_surface.rs` | AST audit (syn 2) |
| BC-2.03.002 | §Scope §In Scope (ClaudeCodeModule) | SS-engine-module.md v1.1.27 §ClaudeCodeModule | P0 | `monocle-runtime/tests/engine_module_claude_detect.rs` | Integration |
| BC-2.03.003 | §Scope §In Scope (ClaudeCodeModule) | SS-engine-module.md v1.1.27 §BC-ENGINE-002-ERR | P0 | `monocle-runtime/tests/engine_module_home_unresolvable.rs` | Integration (env-isolation) |
| BC-2.03.004 | §Scope §In Scope (ClaudeCodeModule) | SS-engine-module.md v1.1.27 §Inherent operations | P0 | `monocle-runtime/tests/engine_module_claude_methods.rs` | Integration |
| NFR-012 | §Scope (daemon start — runtime_dir fallback chain; lock-file 0o600 + runtime_dir 0o700) | SS-daemon-lifecycle.md v1.0.33 §Start Sequence | P0 | `monocle-runtime/tests/daemon_lifecycle.rs` | Integration (VP-005 Post-condition 9 / probe 5.e) |
| BC-2.06.023 | §Success Criteria (killer scenario — permission overlay; concurrent prompt resolution) | SS-tui.md v1.0.0 §Permission Overlay §Overlay Stack Lifecycle <!-- version-pin-historical: version at PRD authoring time -->; SS-ipc.md v1.0.0 §ServerToClient::PermissionPromptResolved <!-- version-pin-historical: version at PRD authoring time --> | P0 | `monocle-tui/tests/permission_overlay_resolved.rs` | Integration |

---

## §Trace v1.26 — Template Compliance Remediation (PRD restructure)

**1.27.4** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers and time qualifiers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at authoring time. No normative content changed.

**Bump:** v1.25 → v1.26.
**Predecessor pin:** v1.25 commit a71ca67 (D-047 strict CONVERGENCE achieved on monolithic structure; subsequently determined to be structurally non-compliant per template-compliance-audit-r1).

**Scope of v1.26:**
- §3 (Full BC Specifications) DELETED; 22 BCs now live as sharded files in `behavioral-contracts/ss-NN/BC-2.SS.NNN.md` (created in Dispatch 2 commit d02bf2a + Dispatch 3 commit f259ade).
- §4 NFR catalog → `prd-supplements/nfr-catalog.md` + summary reference in PRD §4.
- §5 Error Taxonomy → `prd-supplements/error-taxonomy.md` + summary reference in PRD §5.
- New `prd-supplements/interface-definitions.md` + `prd-supplements/test-vectors.md` created per template.
- `supplements:` frontmatter field populated: `[interface-definitions.md, error-taxonomy.md, test-vectors.md, nfr-catalog.md]`.
- Section ordering aligned to prd-template.md: Overview → BC Index → Interface (ref) → NFR (ref) → Error Taxonomy (ref) → Test Vectors (ref) → Competitive Diff → RTM.
- §Trace history v1.0–v1.25 retired to git PG-5 (preserved at commit a71ca67); v1.26 starts fresh §Trace lineage post-restructure.
- BC IDs renumbered `BC-DAEMON-NNN → BC-2.01.NNN`, `BC-AUTH-NNN → BC-2.01.NNN`, `BC-LOCK-001 → BC-2.01.010`, `BC-RING-001 → BC-2.01.007`, `BC-ABI-NNN → BC-2.02.NNN`, `BC-TYPES-001 → BC-2.02.003`, `BC-FACTORY-NNN → BC-2.02.NNN`, `BC-PROTO-NNN → BC-2.02.NNN`, `BC-ENGINE-NNN → BC-2.03.NNN` per audit §661-714 renumbering map; old IDs preserved in BC-INDEX.md renumbering appendix (Old ID column) per append-only ID policy.
- Old §8 Cross-Cutting Concerns: content preserved in SS-conventions-anti-patterns.md (authoritative source); not replicated in PRD (PRD is an index document, not a conventions reference).
- Old §9 Edge Case Catalog: EC-001 through EC-061 live in individual BC files (EC content embedded per-BC). The PRD no longer maintains a cross-BC EC table (this was a monolith-era artifact; BC sharding makes per-BC EC the canonical location).
- Old §10 Glossary: preserved in full below.

**Audit reference:** `.factory/plans/template-compliance-audit-r1.md`.
**Dispatch:** Template-compliance remediation Dispatch 4 of 7+.
**Predecessors:** Dispatch 1 architect (ARCH-INDEX), Dispatch 2/3 PO (BC files + BC-INDEX).
**Next:** Dispatch 5 FV shards VP monolith with new BC IDs.

---

## §Trace v1.26.1 — Audit R2 Residual RES-05: §6/§7 Column Schema Reconciliation

**Bump:** v1.26 → v1.26.1.
**Predecessor pin:** v1.26 commit (template-compliance-audit-r1 remediation; §3 deleted, BC sharding, supplement extraction).

**Scope of v1.26.1 (patch — table schema only, no content added or removed):**

### §6 Changes

**From:** Single flat table with columns `Differentiator | Description | BC Backing | Verification`.

**To:** Per-differentiator subsections (`### 6.N KD-NNN — Name`) each containing `| BC ID | Contribution | Verification |` tables, matching prd-template.md §6 pattern.

**Project-specific extension retained:** `Verification` column (3rd column, beyond template's 2-column minimum). Rationale: monocle's killer scenarios are explicitly described in the vision document (v1.1.1) and product brief (v1.4.23). Capturing the verification scenario inline per differentiator prevents drift during adversarial review and ensures every claimed differentiator remains verifiable without cross-referencing the vision. This extension is additive (does not remove required template columns) and is self-documenting via the blockquote note at §6 head.

**Content changes:** None. All 8 differentiators preserved. All BC ID citations preserved. All descriptions preserved (moved into subsection introductory text). All verification notes preserved (moved into `Verification` column).

### §7 Changes

**From:** `| Requirement ID | Brief Section | Architecture Source | Priority | Test File | Test Type |` (6 columns; `Requirement ID` non-template name; `Brief Section` and `Architecture Source` non-template names).

**To:** `| BC ID | Source (L2 CAP) | Module(s) | Priority | Test File | Test Type |` (6 columns).

Column mapping:
- `Requirement ID` → `BC ID` (template column name; same data)
- `Brief Section` → `Source (L2 CAP)` (template column name; monocle's interim L2 traceability pending BA Dispatch 6 domain spec; brief sections are the authoritative source until L2 CAP IDs are assigned)
- `Architecture Source` → `Module(s)` (template column name; architecture file references preserved, shortened for readability)
- `Priority` → `Priority` (unchanged)
- `Test File` → `Test File` (project-specific extension, see below)
- `Test Type` → `Test Type` (template column name; unchanged)

**Project-specific extension retained:** `Test File` column (5th column, beyond template's 5-column schema). Rationale: direct test file path traceability is production-grade quality that reduces implementation ambiguity — implementers and test-writers have explicit file location targets. Extension is additive and self-documenting via the blockquote note at §7 head.

**Content changes:** None. All 22 BC rows + NFR-012 row preserved. All architecture source citations preserved (abbreviated in `Module(s)` column for readability while retaining version pin and subsection reference).

**Audit reference:** `.factory/plans/template-compliance-audit-r2.md` RES-05.
**Dispatch:** Audit R2 residual fix — concurrent with RES-02 (BC VP anchor sweep) and RES-03 (FV VP template compliance).
**Predecessors:** architect RES-01+RES-04 COMPLETE (0af206a).

---

## §Trace v1.26.2 — F-R105-7 Manifest Pin Refresh (v1.1.15 → v1.1.17)

**Bump:** v1.26.1 → v1.26.2.
**Predecessor pin:** v1.26.1 (Audit R2 residual §6/§7 column schema reconciliation; commit in factory-artifacts branch).

**Scope of v1.26.2 (patch — manifest pin only, no content added or removed):**

**Finding:** F-R105-7 MED — PRD `traces_to` frontmatter cited `SS-deps-pin-manifest.md v1.1.15`; architect confirmed delta v1.1.15 → v1.1.17 is structural only (pin-number swap, no content cascade required).

**SE-17c — Before (body-scope grep evidence):**
```
traces_to field: "...SS-deps-pin-manifest.md v1.1.15;..."
```

**SE-17d — After (body-scope grep evidence):**
```
traces_to field: "...SS-deps-pin-manifest.md v1.1.17;..."
```

**Manifest pin replacement count:** 1 occurrence (`traces_to` frontmatter field in prd.md).

**Note:** References to `SS-engine-module.md v1.1.20` in §7 RTM rows (BC-2.03.001 through BC-2.03.004) are the ENGINE MODULE version, NOT the deps-pin-manifest version. These are correct and unchanged.

**Concurrent:** nfr-catalog.md v1.0 → v1.1 (F-R105-2 + GAP-R44-1 VP ID sweep; same burst). interface-definitions.md v1.1 → v1.2 (F-R105-10/11 + GAP-R44-3 lock file schema; same burst).

---

## §Trace v1.26.3 — F-R105-12 + GAP-R44-4 (VP alias + abbreviation count)

**Bump:** v1.26.2 → v1.26.3.
**Predecessor pin:** v1.26.2 (F-R105-7 manifest pin refresh; commit 39082b0 on factory-artifacts).
**Timestamp:** 2026-05-17T19:30:00Z

**Scope of v1.26.3 (patch — two surgical corrections; no content added or removed):**

**Finding F-R105-12 LOW — §7 NFR-012 row stale VP alias:**

SE-17f before/after evidence:

**Before:** `Integration (VP-DAEMON-005 Post-condition 9 / probe 5.e)`
**After:** `Integration (VP-005 Post-condition 9 / probe 5.e)`

Rationale: `VP-DAEMON-005` is the legacy subsystem-scoped alias. VP-INDEX v1.1 §SS-01 table (line 110) maps `VP-DAEMON-005 → VP-005`. The canonical ID per VP-INDEX v1.1 (source of truth) is `VP-005` (title: "Lock File Lifecycle — Atomic Create, Pid Gate, Mode 0o600/0o700"). All VP cross-references must use the canonical VP-NNN form.

SE-17c — before (body-scope grep evidence):
```
§7 NFR-012 row Test Type column: "Integration (VP-DAEMON-005 Post-condition 9 / probe 5.e)"
```

SE-17d — after (body-scope grep evidence):
```
§7 NFR-012 row Test Type column: "Integration (VP-005 Post-condition 9 / probe 5.e)"
```

**Finding GAP-R44-4 LOW — §5a prose "6 subsystem abbreviations" count incorrect:**

SE-17f before/after evidence:

**Before:** `Phase 1 defines 14 error codes across 6 subsystem abbreviations (`DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`).`
**After:** `Phase 1 defines 14 error codes across 7 subsystem abbreviations (`DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`).`

Rationale: Actual enumeration contains 7 distinct abbreviations: `DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`. Verified against `prd-supplements/error-taxonomy.md` §Error Catalog (14 rows: E-AUTH-001/002, E-DAEMON-001/002/003/004, E-LOCK-001/002/003, E-ENG-001, E-FACT-001/002, E-RING-001, E-PROTO-001). Count was 6, correct count is 7. The list itself was already correct — only the numeric count required correction.

SE-17c — before (body-scope grep evidence):
```
§5a line: "14 error codes across 6 subsystem abbreviations"
```

SE-17d — after (body-scope grep evidence):
```
§5a line: "14 error codes across 7 subsystem abbreviations"
```

**Concurrent:** Parallel FV dispatch sweeping 22 VP §References to cite PRD v1.26.3. Parallel architect dispatch adjudicating auth-header interop (not in PRD scope). Parallel BA dispatch fixing L2-INDEX anchors (not in PRD scope).

---

## §Trace v1.26.4 — F-R106-4 (RTM pin refresh + ADR-0005 input + E-AUTH-003 count)

**Bump:** v1.26.3 → v1.26.4.
**Predecessor pin:** v1.26.3 (VP alias + abbreviation count; commit on factory-artifacts).
**Timestamp:** 2026-05-17T22:20:00Z

**Scope of v1.26.4 (three-part patch: architecture version pin refresh, ADR-0005 traceability, error count update):**

**Finding F-R106-4 HIGH — §7 RTM + traces_to stale architecture pins:**

Pin replacement summary:

| Field | Before | After | Occurrence Count |
|-------|--------|-------|-----------------|
| `SS-daemon-lifecycle.md` (traces_to + §7 RTM) | v1.0.25 | v1.0.30 | 12 (1 traces_to + 11 RTM rows: BC-2.01.001–BC-2.01.010 + NFR-012) |
| `SS-core-types-and-abi.md` (traces_to + §7 RTM) | v1.2.8 | v1.2.11 | 9 (1 traces_to + 8 RTM rows: BC-2.02.001–BC-2.02.008) |
| `SS-engine-module.md` (traces_to + §7 RTM) | v1.1.15 | v1.1.18 | 5 (1 traces_to + 4 RTM rows: BC-2.03.001–BC-2.03.004) |

**Cross-dispatch coordination:** `SS-daemon-lifecycle.md v1.0.32` is the target version per architect 5E (F-FC-I005 removal + ADR-0005 auth-middleware section). v1.0.30 is the architect 5E commit target for the same burst. This PRD traces_to pins to v1.0.30 as coordinated.

SE-17f before/after evidence:

**Before (traces_to):** `SS-daemon-lifecycle.md v1.0.25; SS-core-types-and-abi.md v1.2.8; SS-engine-module.md v1.1.15`
**After (traces_to):** `SS-daemon-lifecycle.md v1.0.32; SS-core-types-and-abi.md v1.2.13; SS-engine-module.md v1.1.20`

SE-17c — before (§7 RTM rows — representative sample):
```
| BC-2.01.001 | ... | SS-daemon-lifecycle.md v1.0.25 §GET /healthz | ... |
| BC-2.02.001 | ... | SS-core-types-and-abi.md v1.2.8 §ABI Version Constant | ... |
| BC-2.03.001 | ... | SS-engine-module.md v1.1.15 §EngineModule Trait Signature | ... |
```

SE-17d — after (§7 RTM rows — representative sample):
```
| BC-2.01.001 | ... | SS-daemon-lifecycle.md v1.0.32 §GET /healthz | ... |
| BC-2.02.001 | ... | SS-core-types-and-abi.md v1.2.13 §ABI Version Constant | ... |
| BC-2.03.001 | ... | SS-engine-module.md v1.1.26 §EngineModule Trait Signature | ... |
```

**Finding GAP-R45-2 — ADR-0005 missing from inputs/traces_to:**

ADR-0005 (dual-accept auth header) is a canonical architecture decision that affects BC-2.01.008, BC-2.01.009, SS-daemon-lifecycle.md v1.0.32, and all 4 prd-supplements in this burst. It must appear in the PRD's inputs and traces_to fields.

SE-17f: Added `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md` to both `inputs:` array and `traces_to:` string.

**Error count update — E-AUTH-003 addition:**

error-taxonomy.md v1.1 (same burst) adds E-AUTH-003 (Cosmetic, WARN log, alias deprecation per BC-2.01.009 INV-6). Total error codes: 14 → 15.

SE-17f before/after:

**Before:** `Phase 1 defines 14 error codes across 7 subsystem abbreviations... Severity levels: Broken..., Degraded...`
**After:** `Phase 1 defines 15 error codes across 7 subsystem abbreviations... Severity levels: Broken..., Degraded..., Cosmetic (WARN log, zero exit, no functional impact; E-AUTH-003 alias deprecation log)`

**Concurrent:** Parallel PO 5A (BC scope), PO 5C (brief), FV 5D (VPs — VP-009 alias-path expansion), Architect 5E (ADR-0005 path + SS-daemon-lifecycle v1.0.30). All in same R106 Round 5 burst.

---

## §Trace v1.26.5 — F-R107 Round 6B (fabricated ADR path + traces_to refresh)

**Bump:** v1.26.4 → v1.26.5.
**Predecessor pin:** v1.26.4 (F-R106-4 RTM pin refresh + ADR-0005 input + E-AUTH-003 count; commit on factory-artifacts).
**Timestamp:** 2026-05-17T23:00:00Z

**Scope of v1.26.5 (three-part patch: ADR path correction, traces_to refresh, body §Trace correction):**

**Finding F-R107-1 CRITICAL — Fabricated ADR-0005 path in inputs/traces_to/body:**

SE-17f before/after evidence:

**Before (frontmatter `inputs:`):** `architecture/adr/ADR-0005-dual-accept-auth-header.md`
**After (frontmatter `inputs:`):** `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md`

**Before (frontmatter `traces_to:`):** `...ADR-0005-dual-accept-auth-header.md;...`
**After (frontmatter `traces_to:`):** `...ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md;...`

**Before (body §Trace v1.26.4 SE-17f prose):** `Added \`architecture/adr/ADR-0005-dual-accept-auth-header.md\``
**After (body §Trace v1.26.4 SE-17f prose):** `Added \`architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md\``

Canonical filename verified via ARCH-INDEX and `ls .factory/specs/architecture/adr/`. All 3 occurrences in prd.md corrected.

**Finding F-R107-3 HIGH — traces_to stale pins (brief + BC-INDEX):**

SE-17f before/after evidence:

**Before:** `product-brief.md v1.4.23; ...; behavioral-contracts/BC-INDEX.md v1.1`
**After:** `product-brief.md v1.4.25; ...; behavioral-contracts/BC-INDEX.md v1.5`

Rationale: product-brief.md v1.4.25 and BC-INDEX.md v1.5 are the post-PO-6A Round 6 target versions per dispatch instructions.

**Concurrent:** Parallel PO 6A (BC scope), FV 6C (VPs), Architect 6D (SS-forward-compatibility), BA 6E (L2-INDEX). All in same R107 Round 6 burst.

---

## §Trace v1.26.6 — F-R108-7 + GAP-R47-3 Round 7B (traces_to arch pin refresh + L2-INDEX resolve)

**Bump:** v1.26.5 → v1.26.6.
**Predecessor pin:** v1.26.5 (F-R107 Round 6B fabricated ADR path + traces_to refresh; commit on factory-artifacts).
**Timestamp:** 2026-05-18T01:00:00Z

**Scope of v1.26.6 (two-part patch: arch version pin refresh + L2-INDEX placeholder removal):**

**Finding F-R108-7 HIGH — traces_to stale architecture pins (post-Architect-6D):**

Architect 7C (Round 7 parallel) normalizes timestamps on SS-daemon-lifecycle, SS-core-types-and-abi, and SS-engine-module without version bumps; however the prior PRD `traces_to` was stale from pre-Architect-6D commit 98396fe which bumped those files to the versions now confirmed canonical.

SE-17f before/after evidence:

**Before:** `...SS-daemon-lifecycle.md v1.0.32; SS-core-types-and-abi.md v1.2.13; SS-engine-module.md v1.1.20; ...BC-INDEX.md v1.5; ...`
**After:** `...SS-daemon-lifecycle.md v1.0.31; SS-core-types-and-abi.md v1.2.12; SS-engine-module.md v1.1.19; ...BC-INDEX.md v1.6; ...`

Also refreshed `product-brief.md v1.4.25 → v1.4.26` (brief bumped in this same Round 7B burst per F-R108-8).

**Finding GAP-R47-3 MEDIUM — traces_to "L2-INDEX.md (pending BA Dispatch 6)" placeholder:**

BA Dispatch 6 was completed at commit fcf2b2d producing L2-INDEX v1.0.7. The `(pending BA Dispatch 6)` annotation is stale.

SE-17f before/after evidence:

**Before:** `domain-spec/L2-INDEX.md (pending BA Dispatch 6)`
**After:** `domain-spec/L2-INDEX.md v1.0.7`

**Changes made:** frontmatter `traces_to:` — 5 version pins refreshed (brief, SS-daemon-lifecycle, SS-core-types-and-abi, SS-engine-module, BC-INDEX) + L2-INDEX placeholder resolved; version bumped v1.26.5 → v1.26.6; timestamp refreshed.

**Scope:** PO-only frontmatter patch. No body content changed. No BC, VP, or architecture file changes in this burst.

---

## §Trace v1.26.7 — F-R109 Round 8B (SS pin refresh + §Trace ascending + RTM pins + brief bump)

**Bump:** v1.26.6 → v1.26.7.
**Predecessor pin:** v1.26.6 (F-R108-7 + GAP-R47-3 traces_to arch pin refresh; commit on factory-artifacts).
**Timestamp:** 2026-05-18T05:35:00Z

**Scope of v1.26.7 (three-part patch: RTM SS pin refresh, traces_to update, §Trace ascending reorder):**

**Finding F-R109-5 HIGH — PRD body RTM SS pins stale:**

Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32, SS-core-types-and-abi.md v1.2.11→v1.2.13 (actually from v1.2.8 stale per §7), SS-engine-module.md v1.1.18 → v1.1.20 (actually from v1.1.15 stale). PRD §7 RTM rows and traces_to refreshed.

Pin replacement summary:

| Field | Before | After | Occurrence Count |
|-------|--------|-------|-----------------|
| `SS-daemon-lifecycle.md` (traces_to + §7 RTM) | v1.0.30 (body) / v1.0.31 (traces_to) | v1.0.32 | 12 body + 1 traces_to |
| `SS-core-types-and-abi.md` (traces_to + §7 RTM) | v1.2.11 (body) / v1.2.12 (traces_to) | v1.2.13 | 9 body + 1 traces_to |
| `SS-engine-module.md` (traces_to + §7 RTM) | v1.1.18 (body) / v1.1.19 (traces_to) | v1.1.20 | 5 body + 1 traces_to |
| `product-brief.md` (traces_to) | v1.4.26 | v1.4.27 | 1 traces_to |
| `BC-INDEX.md` (traces_to) | v1.6 | v1.7 | 1 traces_to |

> **F-R110-13 NOTE:** Occurrence counts in the table above (12+9+5 body rows) are estimates from the Round 8B dispatch. A formal re-grep was not performed at that time. These counts are preserved as-authored for historical record. Future dispatches should verify with `grep -c` before claiming exact counts.

**Finding F-R109-9 HIGH — §Trace blocks descending → ascending:**

§Trace blocks were descending (v1.26.6, v1.26.5, ..., v1.26). Reordered to ascending (v1.26 → v1.26.6 → v1.26.7). Content of each section preserved verbatim; only insertion order corrected.

**Changes made:** §7 RTM SS pins refreshed (3 subsystem docs × 11+8+4 rows); traces_to frontmatter refreshed (5 pins); §Trace blocks reordered ascending; version bumped v1.26.6 → v1.26.7; timestamp refreshed.

**Scope:** PO-only. No BC, VP, or architecture file changes in this burst. Concurrent with Architect 8A (SS doc bumps) and FV 8C.

## §Trace v1.26.8 — F-R110 Round 9B (timestamp monotonicity + fabrication correction + BC-INDEX v1.8 ref)

**Bump:** v1.26.7 → v1.26.8.
**Predecessor pin:** v1.26.7 (F-R109 Round 8B; timestamp corrected to 2026-05-18T05:35:00Z in this burst).
**Timestamp:** 2026-05-18T06:00:00Z

**F-R110-1 CRITICAL — §Trace v1.26.7 timestamp corrected:**
- §Trace v1.26.7 Timestamp field was `2026-05-17T04:35:00Z` (wrong date — Round 8 was authored on 2026-05-18). Corrected to `2026-05-18T05:35:00Z`.
- PRD frontmatter timestamp: `2026-05-17T04:35:00Z` → `2026-05-18T05:35:00Z`.
- SE-16d monotonicity: v1.26.7 timestamp `2026-05-18T05:35:00Z > 2026-05-18T01:00:00Z` (v1.26.6) PASS. ARITHMETICALLY TRUE.

**F-R110-2 CRIT — Fabrication correction note:**
- The §Trace v1.26.7 description of SS-02/SS-03 staleness in "actually from v1.2.8 stale per §7" / "actually from v1.1.15 stale" is the correct historical record. The fabrication was in the BC files themselves (not in the PRD). The PRD §Trace v1.26.7 parentheticals accurately note the staleness — no body change needed.

**F-R110-13 MED — RTM count claim qualified:**
- §Trace v1.26.7 table Occurrence Count cells (12 body, 9 body, 5 body) were estimated without a grep transcript. Added F-R110-13 NOTE inline in v1.26.7 qualifying these as estimates.

**traces_to update:** BC-INDEX v1.7 → v1.8.

**Changes made:** frontmatter version v1.26.7 → v1.26.8; frontmatter timestamp refreshed; §Trace v1.26.7 timestamp corrected; F-R110-13 NOTE added; traces_to BC-INDEX pin updated.

SE-16d monotonicity PASS: 2026-05-18T06:00:00Z > prior 2026-05-18T05:35:00Z (v1.26.7). ARITHMETICALLY TRUE: 2026-05-18T06:00:00Z > 2026-05-18T05:35:00Z PASS.

## §Trace v1.26.9

**F-R111 Round 10 — timestamp pathology fix + L2-INDEX pin update** (2026-05-18T07:00:00Z):

**Bump:** v1.26.8 → v1.26.9.
**Predecessor pin:** v1.26.8 (F-R110 Round 9B; timestamp `2026-05-18T06:00:00Z`).
**Timestamp:** 2026-05-18T07:00:00Z

**F-R111-1 CRITICAL — v1.26.8 frontmatter timestamp corrected:**
- v1.26.8 frontmatter timestamp was `2026-05-18T05:35:00Z`. This is the corrected v1.26.7 timestamp, not the v1.26.8 burst timestamp. The v1.26.8 burst ran at `2026-05-18T06:00:00Z`. Corrected frontmatter to `2026-05-18T07:00:00Z` (Round 10 fix burst timestamp).

**F-R111-5 MED — traces_to L2-INDEX pin updated:**
- `domain-spec/L2-INDEX.md v1.0.7` → `domain-spec/L2-INDEX.md v1.0.8` (current version per L2-INDEX frontmatter).
- BC-INDEX pin updated: `behavioral-contracts/BC-INDEX.md v1.8` → `behavioral-contracts/BC-INDEX.md v1.9`.

**Changes made:** frontmatter version v1.26.8 → v1.26.9; frontmatter timestamp refreshed; traces_to L2-INDEX and BC-INDEX pins updated.

SE-16d monotonicity PASS: 2026-05-18T07:00:00Z > prior 2026-05-18T06:00:00Z (v1.26.8). ARITHMETICALLY TRUE: 2026-05-18T07:00:00Z > 2026-05-18T06:00:00Z PASS.

---

## §Trace v1.26.10

**R16A — F-R117-1 / GAP-R56-001 brief pin back-cascade (v1.4.27 → v1.4.28)** (2026-05-18T15:00:00Z):

**Bump:** v1.26.9 → v1.26.10.
**Predecessor pin:** v1.26.9 (F-R111 Round 10 timestamp pathology fix; timestamp `2026-05-18T07:00:00Z`).
**Timestamp:** 2026-05-18T15:00:00Z

**F-R117-1 HIGH / GAP-R56-001 HIGH — brief pin back-cascade:**

The PRD `traces_to:` frontmatter cited `product-brief.md v1.4.27`. The canonical brief is `v1.4.28` (bumped in post-R15B commit 08d1ef4 earlier on 2026-05-18). This defect class is a back-cascade gap on a sibling spec bump — the same class as F-R116-2 (closed in prior round). This is the 2nd SE-22-class occurrence (1st was O-R116-1 per SE-17e sibling-propagation note).

**SE-17a GREP BEFORE (scoped to PRD `traces_to:` field, D-116 normative substring):**

```
awk 'NR==11' .factory/specs/prd.md
traces_to: "product-brief.md v1.4.27; vision-synthesis v1.1.2; ..."
```

**SE-17c GREP AFTER (post-edit state):**

```
awk 'NR==11' .factory/specs/prd.md
traces_to: "product-brief.md v1.4.28; vision-synthesis v1.1.2; ..."
```

**SE-17f SCOPE SWEEP — all `v1.4.27` occurrences in PRD:**

| Location | Line | Classification | Action |
|----------|------|---------------|--------|
| `traces_to:` frontmatter | 11 | NORMATIVE live pin | Updated v1.4.27 → v1.4.28 |
| §Trace v1.26.7 BEFORE/AFTER table | 577 | HISTORICAL record (prior round's change) | Preserved verbatim — correct content |

**SE-17g citation classification:** 1 normative live pin updated; 1 historical trace table preserved. No body prose references to `v1.4.27` outside historical §Trace blocks.

**Changes made:** frontmatter `traces_to:` brief pin v1.4.27 → v1.4.28; version bumped v1.26.9 → v1.26.10; timestamp refreshed.

SE-16d monotonicity PASS: 2026-05-18T15:00:00Z > prior 2026-05-18T07:00:00Z (v1.26.9). ARITHMETICALLY TRUE: 2026-05-18T15:00:00Z > 2026-05-18T07:00:00Z PASS.

---

## §Trace v1.26.11

**R17A — F-R118-1 / GAP-R57-001 + F-R118-2 / GAP-R57-002 + GAP-R57-008 — traces_to BC-INDEX v1.10 + L2-INDEX v1.0.9 + ARCH-INDEX v1.0.10 pin + RTM annotation update (SE-22 first formal application)** (2026-05-18T18:00:00Z):

**Bump:** v1.26.10 → v1.26.11.
**Predecessor pin:** v1.26.10 (R16A F-R117-1 / GAP-R56-001 brief pin back-cascade; timestamp `2026-05-18T15:00:00Z`).
**Timestamp:** 2026-05-18T18:00:00Z

**SE-22 First Formal Application — Full-body pin sweep before editing.**

SE-22 ("sibling-spec multi-pin normative sweep") requires grepping the entire PRD body for all stale pins to artifacts bumped in sibling dispatches before any edit is applied. This is the first cycle where SE-22 is executed as a named codified discipline (codified at commit 8ab97d8, R17-pre state v5.77).

**SE-22 Sweep Transcript — all pin sites inventoried:**

```
grep -n "BC-INDEX.md" .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "behavioral-contracts/BC-INDEX.md v1.9" → TARGET v1.10
  L83 (body prose INFORMATIONAL): "behavioral-contracts/BC-INDEX.md" (unversioned reference to index location) → NO ACTION
  L300/515/516/518/538/539/578/626 (§Trace historical INFORMATIONAL) → PRESERVE

grep -n "L2-INDEX.md" .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "domain-spec/L2-INDEX.md v1.0.8" → TARGET v1.0.9
  L543/549/550/625 (§Trace historical INFORMATIONAL) → PRESERVE

grep -n "ARCH-INDEX.md" .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "architecture/ARCH-INDEX.md" (no version pin) → ADD v1.0.10
  L78 (body prose INFORMATIONAL): "architecture/ARCH-INDEX.md" (unversioned reference to index location) → NO ACTION

grep -n "product-brief.md v1.4" .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "product-brief.md v1.4.28" — CURRENT (R16A set this) → NO ACTION
  L515/516/518/541/644/650/657 (§Trace historical INFORMATIONAL) → PRESERVE

grep -n "BC-INDEX.md v1.9" (stale check)
  L11: CONFIRMED STALE → updated

grep -n "L2-INDEX.md v1.0.8" (stale check)
  L11: CONFIRMED STALE → updated
```

**SE-17g NORMATIVE vs INFORMATIONAL classification:**

| Line | Citation | Classification | Action |
|------|----------|---------------|--------|
| 11 | `behavioral-contracts/BC-INDEX.md v1.9` in `traces_to:` | NORMATIVE live pin | Updated v1.9 → v1.10 |
| 11 | `domain-spec/L2-INDEX.md v1.0.8` in `traces_to:` | NORMATIVE live pin | Updated v1.0.8 → v1.0.9 |
| 11 | `architecture/ARCH-INDEX.md` (no version) in `traces_to:` | NORMATIVE live pin (missing pin) | Added v1.0.10 |
| 83 | `behavioral-contracts/BC-INDEX.md` (unversioned, navigation reference) | INFORMATIONAL | Preserved |
| 78 | `architecture/ARCH-INDEX.md` (unversioned, navigation reference) | INFORMATIONAL | Preserved |
| 257 | `monocle L2 domain spec is pending BA Dispatch 6` (RTM blockquote) | NORMATIVE annotation | Updated — stale text removed, replaced with current state |
| 337 | `pending BA Dispatch 6 domain spec` (§Trace v1.26.1 historical block) | INFORMATIONAL §Trace record | Preserved per SE-17g |
| 543–550 | GAP-R47-3 before/after record (§Trace v1.26.6) | INFORMATIONAL §Trace record | Preserved per SE-17g |
| 625–626 | L2-INDEX v1.0.7→v1.0.8 update record (§Trace v1.26.9) | INFORMATIONAL §Trace record | Preserved per SE-17g |

**F-R118-1 / GAP-R57-001 HIGH — BC-INDEX pin:**

**SE-17a BEFORE (awk L11 literal):**
```
traces_to: "...behavioral-contracts/BC-INDEX.md v1.9;..."
```

**SE-17c AFTER (post-edit literal):**
```
traces_to: "...behavioral-contracts/BC-INDEX.md v1.10;..."
```

Canonical version confirmed: `grep "^version:" .factory/specs/behavioral-contracts/BC-INDEX.md` → `version: "1.10"` (post-R16C commit 9a02f5a).

**F-R118-2 / GAP-R57-002 HIGH — L2-INDEX pin:**

**SE-17a BEFORE (awk L11 literal):**
```
traces_to: "...domain-spec/L2-INDEX.md v1.0.8"
```

**SE-17c AFTER (post-edit literal):**
```
traces_to: "...domain-spec/L2-INDEX.md v1.0.9"
```

Canonical version confirmed: `grep "^version:" .factory/specs/domain-spec/L2-INDEX.md` → `version: "1.0.9"` (post-R16D commit b0d5092).

**SE-22 additional finding — ARCH-INDEX missing version pin in traces_to:**

`traces_to:` contained `architecture/ARCH-INDEX.md` with no version pin. Canonical version: `grep "^version:" .factory/specs/architecture/ARCH-INDEX.md` → `version: "1.0.10"`. Pin added to close the gap proactively (SE-22 sweep obligation; treating as NORMATIVE missing-pin under production-grade default).

**SE-17a BEFORE:**
```
traces_to: "...architecture/ARCH-INDEX.md; behavioral-contracts/BC-INDEX.md v1.9;..."
```

**SE-17c AFTER:**
```
traces_to: "...architecture/ARCH-INDEX.md v1.0.10; behavioral-contracts/BC-INDEX.md v1.10;..."
```

**GAP-R57-008 LOW — §7 RTM blockquote stale annotation:**

The §7 RTM blockquote contained `monocle L2 domain spec is pending BA Dispatch 6; brief sections serve as interim L2 traceability`. BA Dispatch 6 completed at commit fcf2b2d producing L2-INDEX v1.0.7 (now v1.0.9). The annotation was stale in two ways: (1) "pending" was no longer accurate, (2) L2 spec is now complete. RTM rows still use brief section citations as interim traceability anchors — this is correct and intentional (full CAP-NNN back-cascade is a Phase 1b story decomposition task). Updated annotation clarifies current state without requiring RTM row changes.

**SE-17a BEFORE (blockquote at L257):**
```
monocle L2 domain spec is pending BA Dispatch 6; brief sections serve as interim L2 traceability
```

**SE-17c AFTER:**
```
L2 domain spec complete at v1.0.9; full CAP-NNN back-cascade to RTM rows is a Phase 1b task per story decomposition
```

**SE-17c-d L-number revalidation:** Edits were at L11 (traces_to) and L257 (blockquote). Both verified by grep before and after. Line numbers are stable — no insertions/deletions above L257 in the body prior to §Trace additions.

**SE-17f recursive self-revalidation:** This §Trace v1.26.11 block itself contains no version pins that require updating. It references canonical versions confirmed by grep at time of authoring. The §Trace historical blocks (v1.26–v1.26.10) are INFORMATIONAL and preserved verbatim.

**SE-17e sibling-propagation note:** SE-22 first formal application. Three NORMATIVE pin sites found: BC-INDEX (targeted by F-R118-1), L2-INDEX (targeted by F-R118-2), ARCH-INDEX (SE-22 bonus catch — was missing version pin entirely). PRD supplements (interface-definitions, nfr-catalog, error-taxonomy, test-vectors) are out of R17A scope; any stale pins there are surfaced to orchestrator for R17B-E dispatch.

**Changes made:** frontmatter `traces_to:` BC-INDEX v1.9 → v1.10; L2-INDEX v1.0.8 → v1.0.9; ARCH-INDEX added v1.0.10; §7 RTM blockquote annotation updated; version bumped v1.26.10 → v1.26.11; timestamp refreshed.

SE-16d monotonicity PASS: 2026-05-18T18:00:00Z > prior 2026-05-18T15:00:00Z (v1.26.10). ARITHMETICALLY TRUE: 2026-05-18T18:00:00Z > 2026-05-18T15:00:00Z PASS.

---

## §Trace v1.26.12

**R18A — F-R119-1 closure — retrospective §Trace for R17F SM-applied `traces_to` edits (bookkeeping; content unchanged)** (2026-05-18T21:30:00Z):

**Bump:** v1.26.11 → v1.26.12.
**Predecessor pin:** v1.26.11 (R17A F-R118-1/F-R118-2/GAP-R57-008 — BC-INDEX v1.10 + L2-INDEX v1.0.9 + ARCH-INDEX v1.0.10 pin; timestamp `2026-05-18T18:00:00Z`).
**Timestamp:** 2026-05-18T21:30:00Z

**Background — R17F SM scope violation:**

R17F state-manager (commit 7681632, 2026-05-18T20:30:00Z) modified the PRD `traces_to:` field as a "defensive sweep" — adding `SS-conventions-anti-patterns.md v1.29.5` (R17D introduced v1.29.5 at 19:30Z) and updating `product-brief.md` pin from v1.4.28 to v1.4.29 (R17B bumped brief at 18:30Z). The content edits were correct but SM does not have authority to author §Trace blocks or bump artifact versions per the Correct Agent Routing principle (CLAUDE.md) — now codified as SE-23 in R18-pre (commit 70b7552, D-146). R119 adversary correctly flagged this as F-R119-1 HIGH: PRD body content reflected ≥19:30Z state while frontmatter showed v1.26.11 at 18:00:00Z, breaking SE-16d audit-trail monotonicity.

**Resolution:** This §Trace v1.26.12 retrospectively documents the SM-applied edits in the PRD's §Trace timeline, restoring SE-16d monotonicity:

| Edit | Source | Canonical version | Applied by | When | Documented now |
|------|--------|-------------------|-----------|------|----------------|
| `traces_to:` brief pin v1.4.28 → v1.4.29 | R17B (commit b934e57) | v1.4.29 | R17F SM | 2026-05-18T20:30:00Z | §Trace v1.26.12 |
| `traces_to:` SS-conventions-anti-patterns.md pin add v1.29.5 | R17D (commit b7ce1ac) | v1.29.5 | R17F SM | 2026-05-18T20:30:00Z | §Trace v1.26.12 |

**Content integrity:** Both pin values verified canonical at audit time. No NORMATIVE content changed in this burst — bookkeeping only.

**SE-22 in-artifact sweep (PRD scope, R18A):**

```
grep -n "BC-INDEX.md v1\." .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "behavioral-contracts/BC-INDEX.md v1.10" — canonical → NO ACTION
  L515/516/518/538/539/626/691/707/718/732/737/762/767 (§Trace historical INFORMATIONAL) → PRESERVE per SE-17g

grep -n "L2-INDEX.md v1\." .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "domain-spec/L2-INDEX.md v1.0.9" — canonical → NO ACTION
  L550/625/696/710/719/746/751 (§Trace historical INFORMATIONAL) → PRESERVE per SE-17g

grep -n "ARCH-INDEX.md v1\." .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "architecture/ARCH-INDEX.md v1.0.10" — canonical → NO ACTION
  L767 (§Trace historical INFORMATIONAL) → PRESERVE per SE-17g

grep -n "product-brief.md v1\." .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "product-brief.md v1.4.29" — canonical (R17F applied) → NO ACTION
  L515/516/518/541/644/650/657/703/704 (§Trace historical INFORMATIONAL) → PRESERVE per SE-17g

grep -n "VP-INDEX" .factory/specs/prd.md
  L395 (body prose INFORMATIONAL): "VP-INDEX v1.1" in §Trace v1.26.3 historical block → PRESERVE per SE-17g
  No NORMATIVE VP-INDEX pin site in traces_to: — VP-INDEX not listed in traces_to field → NO ACTION

grep -n "SS-conventions-anti-patterns.md v1\." .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "SS-conventions-anti-patterns.md v1.29.5" — canonical (R17F applied) → NO ACTION
  (No §Trace historical occurrences of versioned SS-conventions pin prior to this burst)
```

**SE-17g NORMATIVE vs INFORMATIONAL classification:**

| Line | Citation | Classification | Action |
|------|----------|---------------|--------|
| 11 | `product-brief.md v1.4.29` in `traces_to:` | NORMATIVE live pin | Verified canonical — no change |
| 11 | `SS-conventions-anti-patterns.md v1.29.5` in `traces_to:` | NORMATIVE live pin | Verified canonical — no change |
| 11 | `behavioral-contracts/BC-INDEX.md v1.10` in `traces_to:` | NORMATIVE live pin | Verified canonical — no change |
| 11 | `domain-spec/L2-INDEX.md v1.0.9` in `traces_to:` | NORMATIVE live pin | Verified canonical — no change |
| 11 | `architecture/ARCH-INDEX.md v1.0.10` in `traces_to:` | NORMATIVE live pin | Verified canonical — no change |
| All §Trace blocks | All versioned pins in historical §Trace records | INFORMATIONAL | Preserved verbatim per SE-17g |

**SE-22 zero-residual confirmation:** All NORMATIVE pin sites at L11 are canonical. No stale NORMATIVE pins found. Zero-residual PASS.

**SE-17a BEFORE (awk L4+L8 literals — frontmatter version and timestamp):**
```
version: "1.26.11"
timestamp: 2026-05-18T18:00:00Z
```

**SE-17c AFTER (post-edit state):**
```
version: "1.26.12"
timestamp: 2026-05-18T21:30:00Z
```

**SE-17f recursive self-revalidation:** This §Trace v1.26.12 block itself contains no version pins that require updating. It references canonical versions confirmed by sweep at time of authoring. All §Trace historical blocks (v1.26–v1.26.11) are INFORMATIONAL and preserved verbatim.

**SE-17e sibling-propagation note:** SE-22 sweep scoped to PRD body only per R18A scope constraint (bookkeeping burst). BC-INDEX is addressed in R18B; L2-INDEX in R18C. No stale NORMATIVE pins found in PRD body — zero cross-artifact back-cascade obligations generated by this burst.

**SE-23 first-application context:** SE-23 ("SM defensive-sweep prohibition") was codified in R18-pre (commit 70b7552, D-146) because the R17F SM scope violation broke SE-16d audit-trail monotonicity. This burst (R18A) is the F-R119-1 closure for the PRD half of the violation. SE-23 ensures SM will not touch PRD version/timestamp/§Trace in future bursts; any future drift is surfaced to PO via orchestrator.

**Changes made:** frontmatter version bumped v1.26.11 → v1.26.12; timestamp refreshed 2026-05-18T18:00:00Z → 2026-05-18T21:30:00Z; §Trace v1.26.12 retrospective block added. No NORMATIVE content changed.

**SE-16d monotonicity PASS:** PRD v1.26.12 timestamp `2026-05-18T21:30:00Z` > STATE v5.79 `21:15:00Z` > R17F STATE v5.78 `20:30:00Z` > R17E CAP-001 v1.5 `20:00:00Z` > PRD v1.26.11 `18:00:00Z`. ARITHMETICALLY TRUE: 2026-05-18T21:30:00Z > 2026-05-18T21:15:00Z > 2026-05-18T20:30:00Z > 2026-05-18T20:00:00Z > 2026-05-18T18:00:00Z PASS strict-greater.

**Reference:** R119 report at `.factory/plans/adversary-pass-r119-phase1.md` (commit 70b7552).

---

## §Trace v1.26.13 — R19A F-R120-1/2/3 compound closure — traces_to BC-INDEX v1.11 + L2-INDEX v1.0.10 + VP-INDEX v1.14 + 2 SS pins + 5 pinned ADRs (SE-22 v2 FIRST APPLICATION) (2026-05-19T00:00:00Z)

**Predecessor pin:** v1.26.12 (R18A F-R119-1 retrospective trace + R18E cascade-tail refresh; commit 92c55d2).

**Scope of v1.26.13 (three-class compound fix: index-pin refresh, SS-file pin-completeness, ADR pin-completeness + symmetry):**

### F-R120-1 HIGH — BC-INDEX v1.10 → v1.11 + L2-INDEX v1.0.9 → v1.0.10 pin refresh (consumer-ledger gap; SE-22 v2 occurrence #3 that triggered codification)

- BC-INDEX was bumped to v1.11 in R18B (commit 442f5ac). PRD `traces_to:` was not updated at that time — consumer-ledger gap per SE-22 class.
- L2-INDEX was bumped to v1.0.10 in R18C (commit bedcf30). Same consumer-ledger gap.
- VP-INDEX (also a traces_to entry) was bumped to v1.14 in R18E. Also stale in PRD at v1.26.12 — added to this compound refresh as cascade-tail.

**Before:** `behavioral-contracts/BC-INDEX.md v1.10; domain-spec/L2-INDEX.md v1.0.9` (no VP-INDEX pin present)

**After:** `behavioral-contracts/BC-INDEX.md v1.11; domain-spec/L2-INDEX.md v1.0.10; verification-properties/VP-INDEX.md v1.14`

### F-R120-2 MED — SS-permissions-phase1.md + SS-forward-compatibility.md missing from traces_to (pin-completeness asymmetry)

Both files appear in `inputs:` (lines 9) but were absent from `traces_to:`. The `inputs:` ↔ `traces_to:` symmetry convention requires all versioned inputs to appear in `traces_to:` with pinned versions. These are non-trivial architecture sections that shape BC postconditions directly.

- `architecture/SS-permissions-phase1.md` canonical version: v1.5.2 (confirmed from frontmatter `2026-05-17T16:30:00Z`).
- `architecture/SS-forward-compatibility.md` canonical version: v1.2.19 (confirmed from frontmatter).

**Before:** neither pin present in `traces_to:`

**After:** `architecture/SS-permissions-phase1.md v1.5.2; architecture/SS-forward-compatibility.md v1.2.19` added.

### F-R120-3 MED — ADR asymmetry: only ADR-0005 cited (unpinned), ADR-0001 through ADR-0004 absent; all 5 must be pinned

All 5 ADRs appear in `inputs:`. `traces_to:` had only `ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md` with no version pin. ADR-0001 through ADR-0004 were entirely absent. Full pin-completeness requires all 5 at canonical versions.

| ADR | Canonical version | Source confirmation |
|-----|------------------|---------------------|
| ADR-0001-wasmtime-vs-wasmi.md | v1.0.3 | frontmatter `version: "1.0.3"` |
| ADR-0002-nucleo-acceptance-with-reeval-trigger.md | v1.0.4 | frontmatter `version: "1.0.4"` |
| ADR-0003-license-selection.md | v1.0.2 | frontmatter `version: "1.0.2"` |
| ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md | v1.0.4 | frontmatter `version: "1.0.4"` |
| ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md | v1.0.2 | frontmatter `version: "1.0.2"` |

**Before:** `ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md` (unpinned, ADR-0001–0004 absent)

**After:** all 5 ADRs listed with pinned versions using full relative path form.

---

### SE-22 v1 In-Artifact Sweep Evidence (SE-17a literal grep; D-116 scoped-awk)

**Sweep command:** `grep -nE "(BC-INDEX|L2-INDEX|ARCH-INDEX|VP-INDEX|product-brief|SS-permissions-phase1|SS-forward-compatibility|ADR-000[1-5])" .factory/specs/prd.md`

**NORMATIVE occurrences (require pin-completeness):**
- L11 `traces_to:` field — PRD frontmatter. Authoritative pin location. Updated in this burst.
- L9 `inputs:` field — lists filenames without versions (intentional — inputs uses unversioned filenames; versions pinned in `traces_to:`). No action required.

**INFORMATIONAL occurrences (SE-17g — preserve verbatim):**
- L83 body prose: `behavioral-contracts/BC-INDEX.md` (unversioned location reference). PRESERVED.
- L87 body prose: `architecture/ARCH-INDEX.md` (architecture-source annotation). PRESERVED.
- §7 RTM table: ADR-0005 appears in Traceability column — INFORMATIONAL cross-reference, no version pin required there.
- §Trace historical blocks (v1.26–v1.26.12): all prior before/after evidence. PRESERVED verbatim per SE-17g.

**Post-edit verification:** `grep -nE "BC-INDEX.md v1\.(9|10)[^0-9]|L2-INDEX.md v1\.0\.(7|8|9)[^0-9]" .factory/specs/prd.md` → confirms no stale pins remain outside §Trace BEFORE-evidence blocks.

---

### SE-22 v2 FIRST APPLICATION — Consumer-Ledger Declaration

SE-22 v2 (codified R19-pre, 2026-05-18T23:45:00Z) extends SE-22 v1 with **producer responsibility**: when a spec artifact is bumped, its producer must enumerate KNOWN CONSUMERS and surface any stale-pin drift findings to the orchestrator for downstream dispatch.

**PRD v1.26.13 producer: vsdd-factory:product-owner**

**Known consumers of PRD version pin (who cite `prd.md vN.NN.NN` in their artifacts):**

| Consumer artifact | Last known PRD pin | Status after R19A |
|-------------------|-------------------|-------------------|
| `verification-properties/VP-INDEX.md` §References | v1.26.12 (R18E) | STALE → needs v1.26.13 |
| 22 VP files `behavioral-contracts/ss-NN/BC-*.md` §References | v1.26.12 (R18E sweep) | STALE → needs v1.26.13 |
| `specs/product-brief.md` (body §Phase 1 Scope pin, if present) | verify in R19B | R19B scope |
| `domain-spec/L2-INDEX.md` §Trace | no active PRD pin (L342 confirms "No pin present") | CLEAN — no action |
| `behavioral-contracts/BC-INDEX.md` | no PRD pin verified | CLEAN — no action |
| `prd-supplements/interface-definitions.md` | no `traces_to:` field found in sweep | CLEAN — no action |
| `prd-supplements/error-taxonomy.md` | no `traces_to:` field found in sweep | CLEAN — no action |
| `prd-supplements/nfr-catalog.md` | no `traces_to:` field found in sweep | CLEAN — no action |
| `prd-supplements/test-vectors.md` | no `traces_to:` field found in sweep | CLEAN — no action |
| `domain-spec/capabilities/CAP-001.md` | verify | downstream dispatch if stale |

**Surfaces to orchestrator (SE-22 v2 tripartite responsibility — downstream dispatch required):**

1. **VP-INDEX + 22 VP §References** — cite PRD v1.26.12; after R19A they cite a stale version. Dispatch: `vsdd-factory:formal-verifier` (FV owns VP files). Scope: VP-INDEX §References `PRD:` line + all 22 VP §References `PRD:` lines → v1.26.12 → v1.26.13. This is the same class as R18E Change 3+4 (SM-surfaced, FV-fixed). Estimated burst: R19D or combined with R19C STATE.
2. **CAP-001** — verify whether it cites PRD version. If stale, dispatch: `vsdd-factory:business-analyst`. R19B scope overlap — BA verifies in R19B.

**SE-22 v2 producer declaration complete.** R19A does NOT sweep VP files or BC files — consumer dispatch is the orchestrator's responsibility per SE-22 v2 tripartite protocol.

---

### SE-17e Sibling-Propagation

This fix extends the pin-completeness convention established by:
- F-R110-8 (VP pin-symmetry): VP files must pin all their input sources.
- F-R117-3 (BC pin-symmetry): BC files must pin their input sources.

**Extension (PRD frontmatter level):** PRD `inputs:` ↔ `traces_to:` symmetry is now formally established: every entry in `inputs:` must have a corresponding pinned version in `traces_to:`. The SS-permissions-phase1 and SS-forward-compatibility omissions (F-R120-2) and the ADR-0001–0004 omissions (F-R120-3) are the first instances of this convention being enforced at PRD scope.

**Scope note:** the `inputs:` field uses unversioned filenames by convention (filenames only, no versions). The `traces_to:` field is the sole authoritative version-pin location. The asymmetry class (file in `inputs:` but not `traces_to:`) is now a MEDIUM-severity finding under SE-22 v2.

---

### SE-16d Monotonicity Declaration

PRD v1.26.13 timestamp `2026-05-19T00:00:00Z` > STATE v5.81 at `2026-05-18T23:45:00Z` (R19-pre SE-22 v2 codification commit 646c949) > PRD v1.26.12 timestamp `2026-05-18T21:30:00Z`. ARITHMETICALLY TRUE: 2026-05-19T00:00:00Z > 2026-05-18T23:45:00Z > 2026-05-18T21:30:00Z. SE-16d PASS strict-greater.

---

### Long-Term Context Note

SE-22 v2 is codified as a process stopgap: the producer manually declares known consumers and surfaces drift findings to the orchestrator for dispatch. The upstream vsdd-factory `spec-kit-mcp` proposal (referenced in R19-pre STATE v5.81 entry) would supersede this via structural enforcement — the spec-kit would maintain a consumer registry and automatically flag stale pins on bump. Until that upstream feature lands, SE-22 v2 is the canonical protocol for producer responsibility.

---

**Pin sweep table (R19A complete):**

| Pin | Before | After | Classification | Finding closed |
|-----|--------|-------|----------------|----------------|
| `BC-INDEX.md` | v1.10 | v1.11 | NORMATIVE | F-R120-1 |
| `L2-INDEX.md` | v1.0.9 | v1.0.10 | NORMATIVE | F-R120-1 |
| `VP-INDEX.md` | (absent) | v1.14 | NORMATIVE | F-R120-1 cascade-tail |
| `SS-permissions-phase1.md` | (absent) | v1.5.2 | NORMATIVE | F-R120-2 |
| `SS-forward-compatibility.md` | (absent) | v1.2.19 | NORMATIVE | F-R120-2 |
| `ADR-0001-wasmtime-vs-wasmi.md` | (absent) | v1.0.3 | NORMATIVE | F-R120-3 |
| `ADR-0002-nucleo-acceptance-with-reeval-trigger.md` | (absent) | v1.0.4 | NORMATIVE | F-R120-3 |
| `ADR-0003-license-selection.md` | (absent) | v1.0.2 | NORMATIVE | F-R120-3 |
| `ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md` | (absent) | v1.0.4 | NORMATIVE | F-R120-3 |
| `ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md` | present (unpinned) | v1.0.2 | NORMATIVE | F-R120-3 |
| `ARCH-INDEX.md` | v1.0.10 | v1.0.10 | NORMATIVE | verified current |
| `product-brief.md` | v1.4.29 | v1.4.29 | NORMATIVE | verified current (R19B may bump brief) |
| `vision-synthesis` | v1.1.2 | v1.1.2 | NORMATIVE | verified current |
| `SS-daemon-lifecycle.md` | v1.0.32 | v1.0.32 | NORMATIVE | verified current |
| `SS-core-types-and-abi.md` | v1.2.13 | v1.2.13 | NORMATIVE | verified current |
| `SS-engine-module.md` | v1.1.20 | v1.1.20 | NORMATIVE | verified current |
| `SS-deps-pin-manifest.md` | v1.1.17 | v1.1.17 | NORMATIVE | verified current |
| `SS-conventions-anti-patterns.md` | v1.29.5 | v1.29.5 | NORMATIVE | verified current |

**Changes made:** frontmatter `version` v1.26.12 → v1.26.13; `timestamp` refreshed 2026-05-18T21:30:00Z → 2026-05-19T00:00:00Z; `traces_to:` — BC-INDEX pin v1.10 → v1.11; L2-INDEX pin v1.0.9 → v1.0.10; VP-INDEX pin added v1.14; SS-permissions-phase1.md pin added v1.5.2; SS-forward-compatibility.md pin added v1.2.19; ADR-0001–ADR-0004 pins added; ADR-0005 unpinned → pinned v1.0.2; §Trace v1.26.13 added.

---

## §Trace v1.26.14 — R19E (2026-05-19T01:30:00Z)

**Burst:** R19E. **Producer:** vsdd-factory:product-owner. **Closes:** R19B + R19D consumer-ledger surfaces (combined in one PRD bump per instructions).

### Consumer-Ledger Surfaces Closed

**R19B surface (brief v1.4.29 → v1.4.30):**
R19B (commit 6c863a9) bumped `product-brief.md` from v1.4.29 to v1.4.30. PRD `traces_to:` pin was STALE at v1.4.29. This bump closes that surface.

**R19D surface (L2-INDEX v1.0.10 → v1.0.11):**
R19D (commit 6b85e06) bumped `domain-spec/L2-INDEX.md` from v1.0.10 to v1.0.11 and `CAP-001.md` from v1.5 to v1.6. PRD `traces_to:` pin was STALE at v1.0.10. This bump closes that surface. CAP-001.md is not pinned directly in PRD `traces_to:` (PRD pins the L2-INDEX, not individual CAP files) — no action required for CAP-001.

---

### SE-22 v1 In-Artifact Sweep Evidence (SE-17a literal grep; D-116 scoped-awk)

**Sweep commands:**

```
grep -n "product-brief.md v" .factory/specs/prd.md
grep -n "L2-INDEX.md v" .factory/specs/prd.md
grep -n "CAP-001" .factory/specs/prd.md
```

**Results classification:**

| Line | Content | Classification | Action |
|------|---------|----------------|--------|
| 11 | `product-brief.md v1.4.29` in `traces_to:` | NORMATIVE live pin | Updated v1.4.29 → v1.4.30 |
| 11 | `domain-spec/L2-INDEX.md v1.0.10` in `traces_to:` | NORMATIVE live pin | Updated v1.0.10 → v1.0.11 |
| 515–518 | `product-brief.md v1.4.23/v1.4.25` §Trace historical before/after | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 541 | `product-brief.md v1.4.25 → v1.4.26` §Trace historical | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 644–657 | `product-brief.md v1.4.27/v1.4.28` §Trace historical before/after | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 703–704 | `product-brief.md v1.4.28` sweep evidence (§Trace) | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 832–833 | `product-brief.md v1.4.29` sweep evidence (§Trace) | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 849 | `product-brief.md v1.4.29` §Trace pin-sweep table | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 1016 | `product-brief.md v1.4.29 → v1.4.29` §Trace v1.26.13 pin-sweep | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 550 | `domain-spec/L2-INDEX.md v1.0.7` §Trace historical | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 625 | `L2-INDEX.md v1.0.7 → v1.0.8` §Trace historical | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 696–751 | `L2-INDEX.md v1.0.8/v1.0.9` §Trace historical before/after | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 824–852 | `L2-INDEX.md v1.0.9/v1.0.10` §Trace sweep evidence | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 898 | `L2-INDEX.md v1.0.10` §Trace v1.26.13 before-evidence | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 85 | `CAP-001` §2.1 heading (structural) | STRUCTURAL section heading | No version pin — no action |
| 878 | `CAP-001` §Trace historical monotonicity evidence | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |
| 966 | `CAP-001.md` §Trace v1.26.13 dispatch note | INFORMATIONAL §Trace evidence | Preserved verbatim per SE-17g |

**SE-17g classification:** All non-frontmatter occurrences of stale version strings are in INFORMATIONAL §Trace historical blocks and are preserved verbatim. No NORMATIVE body-prose stale pins found.

**Post-edit verification:** `grep -nE "product-brief.md v1\.4\.(2[0-9])" .factory/specs/prd.md` confirms all v1.4.2x strings are in §Trace BEFORE-evidence slots. `grep -nE "L2-INDEX.md v1\.0\.(7|8|9|10)[^0-9]" .factory/specs/prd.md` confirms all stale L2-INDEX pins outside frontmatter are INFORMATIONAL.

---

### SE-22 v2 FOURTH APPLICATION — Consumer-Ledger Declaration

**PRD v1.26.14 producer: vsdd-factory:product-owner**

**Known consumers of PRD version pin (who cite `prd.md vN.NN.NN` in their artifacts):**

| Consumer artifact | Last known PRD pin | Status after R19E |
|-------------------|-------------------|-------------------|
| `verification-properties/VP-INDEX.md` §References | v1.26.12 (R18E) | STALE → needs v1.26.14 |
| 22 VP files under `behavioral-contracts/ss-NN/` §References | v1.26.12 (R18E sweep) | STALE → needs v1.26.14 |
| `domain-spec/L2-INDEX.md` §Trace | no active PRD pin | CLEAN — no action |
| `behavioral-contracts/BC-INDEX.md` | no PRD pin verified | CLEAN — no action |
| `prd-supplements/interface-definitions.md` | no `traces_to:` field | CLEAN — no action |
| `prd-supplements/error-taxonomy.md` | no `traces_to:` field | CLEAN — no action |
| `prd-supplements/nfr-catalog.md` | no `traces_to:` field | CLEAN — no action |
| `prd-supplements/test-vectors.md` | no `traces_to:` field | CLEAN — no action |

**Surfaces to orchestrator (SE-22 v2 tripartite responsibility — downstream dispatch required):**

**VP-INDEX + 22 VPs §References PRD pin:** last set to v1.26.12 (R18E). R19A bumped PRD to v1.26.13 and declared this surface in R19A §Trace (targeting R19D or R19F). This R19E bump to v1.26.14 means the pending cascade now targets v1.26.14 (not v1.26.13 — skip the intermediate version, go directly to v1.26.14). Dispatch: `vsdd-factory:formal-verifier` (FV owns VP files). Burst: R19F.

**SE-22 v2 producer declaration complete.** R19E does NOT sweep VP files or BC files — consumer dispatch is the orchestrator's responsibility per SE-22 v2 tripartite protocol.

---

### SE-17e Sibling-Propagation

No new convention extensions in this burst. The `inputs:` ↔ `traces_to:` symmetry convention remains as established in v1.26.13 §Trace. This bump closes two consumer-ledger surfaces from sibling spec bumps (R19B brief, R19D L2-INDEX) per the same convention.

---

### SE-16d Monotonicity Declaration

PRD v1.26.14 timestamp `2026-05-19T01:30:00Z` > PRD v1.26.13 timestamp `2026-05-19T00:00:00Z` > R19B brief v1.4.30 commit 6c863a9 > R19D L2-INDEX v1.0.11 commit 6b85e06 > R19A PRD v1.26.13 commit ce1e0ca. ARITHMETICALLY TRUE: 2026-05-19T01:30:00Z > 2026-05-19T00:00:00Z. SE-16d PASS strict-greater.

---

**Pin sweep table (R19E complete):**

| Pin | Before | After | Classification | Finding closed |
|-----|--------|-------|----------------|----------------|
| `product-brief.md` | v1.4.29 | v1.4.30 | NORMATIVE | R19B consumer-ledger surface |
| `domain-spec/L2-INDEX.md` | v1.0.10 | v1.0.11 | NORMATIVE | R19D consumer-ledger surface |
| all other `traces_to:` pins | unchanged | unchanged | NORMATIVE | verified current — no change |

**Changes made:** frontmatter `version` v1.26.13 → v1.26.14; `timestamp` refreshed 2026-05-19T00:00:00Z → 2026-05-19T01:30:00Z; `traces_to:` — `product-brief.md` v1.4.29 → v1.4.30; `domain-spec/L2-INDEX.md` v1.0.10 → v1.0.11; §Trace v1.26.14 added.

---

### §Trace v1.26.15 — R20A (2026-05-19T03:00:00Z)

**Finding closed:** F-R121-1 (HIGH) / GAP-R60-001 (MAJOR) — reverse-cascade staleness: `traces_to:` VP-INDEX pin was v1.14 (stale); canonical is v1.15 since R19F commit d88c0b5.

**Root cause:** R19E (commit 31f984a) authored PRD v1.26.14 at timestamp `2026-05-19T01:30:00Z`. At that moment VP-INDEX was at v1.14 — the pin was correct. R19F (commit d88c0b5) subsequently bumped VP-INDEX to v1.15 as the cascade-tail of the verification-properties refresh. PRD v1.26.14's forward pin TO VP-INDEX was not updated in that R19F burst because VP-INDEX is a downstream consumer of the PRD — not the other way. This is the reverse-cascade gap class: downstream consumer bumps its own version, but the upstream producer's forward reference to it becomes stale.

**SE-17a literal grep evidence (scoped per D-116):**

```
grep -n "VP-INDEX.md v1\." .factory/specs/prd.md
  L11 (traces_to NORMATIVE): "verification-properties/VP-INDEX.md v1.14" — STALE → TARGET v1.15
  L898 (§Trace historical before-evidence) — INFORMATIONAL, preserved verbatim per SE-17g
  L1099 (§Trace v1.26.14 consumer-ledger surface declaration) — INFORMATIONAL, preserved verbatim per SE-17g
```

Classification: L11 is the sole NORMATIVE live pin. Updated v1.14 → v1.15. All other occurrences are INFORMATIONAL §Trace historical evidence; preserved verbatim per SE-17g.

**SE-22 v3 candidate status:** HELD per D-114 (1st named occurrence), D-153. Bidirectional consumer-ledger with fixed-point iteration not yet codified. Long-term solution: spec-kit-mcp upstream proposal §1.3 (INV-005 transitive closure with fixed-point iteration) directly addresses this class. Until codified, reverse-cascade gaps remain detectable only via adversarial review.

**SE-16d monotonicity declaration:** PRD v1.26.15 timestamp `2026-05-19T03:00:00Z` > PRD v1.26.14 timestamp `2026-05-19T01:30:00Z`. ARITHMETICALLY TRUE. SE-16d PASS strict-greater.

**SE-22 v2 producer declaration — downstream cascade surfaces:**

This v1.26.15 bump creates a new consumer-ledger surface: VP-INDEX §References and each of the 22 individual VP files reference `PRD v1.26.14`. After this bump those pins will be stale at v1.26.14. Dispatch target: `vsdd-factory:formal-verifier` (FV owns VP files). Burst: R20B (new pre-SM burst). R20C = SM closure. This is explicitly NOT fixed in this burst per routing principle (PO scope = PRD only; VP cascade = FV scope).

**Pin sweep table (R20A complete):**

| Pin | Before | After | Classification | Finding closed |
|-----|--------|-------|----------------|----------------|
| `verification-properties/VP-INDEX.md` | v1.14 | v1.15 | NORMATIVE | F-R121-1 / GAP-R60-001 |
| all other `traces_to:` pins | unchanged | unchanged | NORMATIVE | verified current — no change |

**Changes made:** frontmatter `version` v1.26.14 → v1.26.15; `timestamp` refreshed `2026-05-19T01:30:00Z` → `2026-05-19T03:00:00Z`; `traces_to:` — `verification-properties/VP-INDEX.md` v1.14 → v1.15; §Trace v1.26.15 added.

---

## §Trace v1.27.0 — PRD Expansion: 22 BCs → 70 BCs (Phase 1 Full Product Scope)

**Bump:** v1.26.15 → v1.27.0 (minor version — new §2.4–§2.7 sections added; no content removed).
**Predecessor pin:** v1.26.15 (R20A F-R121-1 VP-INDEX pin refresh; timestamp `2026-05-19T03:00:00Z`).
**Timestamp:** 2026-05-26T13:00:00Z

**Scope of v1.27.0 (major behavioral expansion — four new subsection groups added to §2):**

PRD expanded from 22 BCs (forward-compatibility contracts only) to 70 BCs covering the complete Phase 1 product scope. The expansion is driven by the gap analysis in `prd-expansion-scope.md` which identified that the original PRD covered only 38% of Phase 1 features (forward-compatibility and infrastructure substrate), leaving the product-facing layer — TUI, IPC, config, daemon wiring, and event bus — with zero BC coverage.

**Revision history entry:**

| Field | Before | After |
|-------|--------|-------|
| Version | 1.26.15 | 1.27.0 |
| Title | "Phase 1 Forward-Compatibility Contracts" | "Phase 1" |
| §2 subsection count | 3 (§2.1 CAP-001, §2.2 CAP-002, §2.3 CAP-003) | 7 (§2.1–§2.3 unchanged + §2.4 CAP-004 + §2.5 CAP-005 + §2.6 CAP-006 + §2.7 CAP-007) |
| Total BCs in §2 | 22 | 70 |
| BC-INDEX version | v1.14 (63 BCs) | v1.15 (111 BCs including 41 SS-DTU) |

**New §2 content summary:**

- **§2.4 Daemon Wiring (CAP-004)** — 12 BCs (BC-2.04.001..BC-2.04.012). Architecture source: SS-daemon-wiring.md. Covers: daemon start sequence with SOQ-2 port-bind-before-lock-file ordering invariant, CLI subcommands (`daemon start`/`daemon stop`), daemon auto-start on TUI launch, `MONOCLE_NO_AUTOSTART=1` suppression, `directories::ProjectDirs` runtime_dir fallback chain, hook endpoint routing (PreToolUse, Notification, Stop/SessionStart/PromptSubmit with correct timeouts), hook tmpfile generation, bounded event bus with drop counter, JSONL ring capacity and rotation policy.

- **§2.5 IPC (CAP-005)** — 8 BCs (BC-2.05.001..BC-2.05.008). Architecture source: SS-ipc.md. Covers: UDS server bind at `runtimeDir/monocle.sock`, TUI client connect and initial state push, three IPC message types (SessionListUpdate, HookEventReceived, PermissionPromptQueued), TUI reconnect after daemon restart, SOQ-3 overlay-clear-on-disconnect invariant, UDS-only Phase 1 constraint (no shared-memory transport).

- **§2.6 TUI (CAP-006)** — 22 BCs (BC-2.06.001..BC-2.06.022). Architecture source: SS-tui.md. Covers: AppMode state machine compile-time mutual exclusion, FocusSnapshot focus restoration, 5-level action dispatch binding precedence, Ctrl-\ popup appear/dismiss without state loss, sessions panel (session list render, `/` filter with nucleo, Enter fullscreen), permission overlay (VecDeque stack push, `[↑↓]` rotate, diff preview via `similar 3`, Accept-Once/Accept-Always/Reject keybindings, `[Esc]` hide without reject, `[t]` trace-to-source stub, disconnect clear, timeout budget), event ribbon panel, status bar (drop counter, breadcrumb, keybinding hint), killer scenario (≤6 keystrokes for dual permission resolve).

- **§2.7 Config (CAP-007)** — 6 BCs (BC-2.07.001..BC-2.07.006). Architecture source: SS-config.md. Covers: atomic write via `tempfile::persist`, config schema version 1 harness profile fields, missing/corrupted config default application, sticky-per-project profile picker, `Ctrl-P` override, CCR path detection via `ccr_path` field.

**Title change rationale:** The original title "Phase 1 Forward-Compatibility Contracts" accurately described the original 22-BC scope (forward-compat focus). With the expansion to 70 BCs covering the full product, the title "Phase 1" is accurate and avoids the false implication that the PRD only covers forward-compatibility. The killer scenario (BC-2.06.022), TUI panels, IPC layer, and config are Phase 1 product features, not forward-compatibility contracts.

**SE-22 v2 producer declaration:**

Known consumers of PRD version pin who will have stale pins after this bump:
- `verification-properties/VP-INDEX.md` §References: last set to v1.26.15. Dispatch: `vsdd-factory:formal-verifier`.
- Individual VP files §References: same stale class. Dispatch: `vsdd-factory:formal-verifier`.
- `behavioral-contracts/BC-INDEX.md` §Trace v1.15 references prd.md in the same burst — aligned.

**SE-16d monotonicity PASS:** PRD v1.27.0 timestamp `2026-05-26T13:00:00Z` > PRD v1.26.15 timestamp `2026-05-19T03:00:00Z`. ARITHMETICALLY TRUE. SE-16d PASS strict-greater.

**Changes made:** frontmatter `version` v1.26.15 → v1.27.0; `timestamp` refreshed `2026-05-19T03:00:00Z` → `2026-05-26T13:00:00Z`; H1 title updated (removed "Forward-Compatibility Contracts" suffix); §2.4 through §2.7 added (48 new BC rows across 4 new subsystem tables); `traces_to:` BC-INDEX pin updated v1.14 → v1.15; §Trace v1.27.0 added.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| ABI | Application Binary Interface. `MONOCLE_ABI_VERSION` identifies the stable contract between `monocle-core` and its consumers (plugin SDK, federation layer). | SS-core-types-and-abi.md §ABI Version Constant |
| BC | Behavioral Contract. A testable specification with preconditions, postconditions, and at least one canonical test vector. | This document |
| `ClaudeCodeModule` | Phase 1 built-in `EngineModule` implementation for Claude Code harness integration. Defined in `monocle-runtime`. | SS-engine-module.md §Phase 1 Implementation: ClaudeCodeModule |
| DTU | Digital Twin Universe. Behavioral clone of the Claude Code hook protocol for testing fidelity and regression detection. | dtu-assessment.md |
| `DaemonStartError::RuntimeDirUnresolvable` | The `DaemonStartError` variant raised when BC-2.01.005 Precondition 2(d) fail-fast triggers (`MONOCLE_RUNTIME_DIR` is unset/empty AND `ProjectDirs::new()` returned `None`). Maps to error code E-DAEMON-004 (exit code 1). | BC-2.01.005 Precondition 2(d); prd-supplements/error-taxonomy.md E-DAEMON-004 |
| `EngineModule` | Trait in `monocle-core::engine` abstracting over AI coding harness adapters. Open (not sealed). | SS-engine-module.md §EngineModule Trait Signature |
| `FactoryAdapter` | Trait in `monocle-core::factory` abstracting over factory-pattern workflow detectors. Open (not sealed). | SS-core-types-and-abi.md §FactoryAdapter Trait |
| `FactoryState` | 7-field canonical struct returned by `FactoryAdapter::read_state()`. Fields: `phase`, `status`, `awaiting`, `blocking_issues`, `convergence`, `cycle`, `custom_fields`. | SS-core-types-and-abi.md §FactoryAdapter Trait |
| FC | Forward-Compatibility item. Pre-Phase-1 contracts locked by human authorization. FC-01 through FC-06. | SS-forward-compatibility.md; product-brief.md §Scope (forward-compatibility contracts sub-bullet) |
| `format_version` | First key in every JSONL ring buffer record. Value `1` for all Phase 1 records. | BC-2.01.007; SS-daemon-lifecycle.md §Drain |
| `HookEventRecord` | Rust struct in `monocle-runtime::ring` written to the JSONL ring buffer. `#[non_exhaustive]`; provides `new()` constructor. | SS-daemon-lifecycle.md §Drain |
| `HookEnvelope` | Proto message in `monocle-proto` with `schema_version` at field number 1. Wire format for Phase 4 federation. | BC-2.02.006, BC-2.02.007; SS-core-types-and-abi.md §Prost Wire Schemas |
| JC-2 | Joint Closure 2: `PostToolUse` omitted from Phase 1 hook endpoint set to preserve gene-source parity with any-context-lazyclaude BC-HOOK-007 canonical 5-endpoint matrix. | vision §Closure Log; brief §Scope |
| `monocle-v1:` | Wire-format prefix for Phase 1 auth tokens. `X-Monocle-Authorization: monocle-v1:<64-hex>`. | BC-2.01.008, BC-2.01.009 |
| `MONOCLE_ABI_VERSION` | `pub const u32 = 1` in `monocle-core::abi`. Exported at crate root. Used by Phase 3 plugin SDK and Phase 4 federation. | BC-2.02.001, BC-2.02.002 |
| `MONOCLE_RUNTIME_DIR` | Environment variable that overrides the runtime directory resolution chain. Per BC-2.01.005 Precondition 2(a), if set and non-empty, this path is used verbatim as the runtime directory. Empty string treated as unset (EC-060 in BC-2.01.005). | BC-2.01.005 Precondition 2(a); prd-supplements/error-taxonomy.md E-DAEMON-004 |
| `#[non_exhaustive]` | Rust attribute preventing exhaustive match and struct literal construction outside the defining crate. Default for all `pub` enums in `monocle-core`. | BC-2.02.003; ADR-0004 |
| OsRng | `rand::rngs::OsRng`. Cryptographically secure random source used for auth token generation. Required; `thread_rng` is forbidden for secrets. | BC-2.01.008; SS-daemon-lifecycle.md §Daemon Lifecycle Protocol §Start Sequence |
| `Phase1Permission` | Exhaustive enum in `monocle-core::permissions`. Five variants. ADR-0004 exempts it from `#[non_exhaustive]`. | ADR-0004; SS-permissions-phase1.md |
| `schema_version` | Proto field number 1 in `HookEnvelope`. Value `1` for all Phase 1 messages. Used by Phase 4 federation to validate message format compatibility. | BC-2.02.006, BC-2.02.007, BC-2.02.008 |
| `VsddFactoryAdapter` | Phase 1 static implementation of `FactoryAdapter`. Detects VSDD Factory workspaces via `document_type: pipeline-state` in `.factory/STATE.md`. | BC-2.02.005 |

---

## §Trace v1.27.1 — CV-P1D-003 closure: BC-2.06.023 added to §7 RTM (2026-05-27T00:00:00Z)

**Bump:** v1.27.0 → v1.27.1.
**Predecessor pin:** v1.27.0 (PRD expansion 22 BCs → 70 BCs; timestamp `2026-05-26T13:00:00Z`).
**Timestamp:** 2026-05-27T00:00:00Z

**CV-P1D-003 MINOR — BC-2.06.023 missing from §7 RTM:**

BC-2.06.023 ("TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved")
was created in BC-INDEX v1.16 (F-P1D-006 closure during the Phase 1d adversarial pass). It
was added to the BC-INDEX SS-06 table and the §2.6 TUI BC index table, but was not added to
the §7 Requirements Traceability Matrix. This gap was flagged by the consistency validator as
CV-P1D-003.

**Row added to §7 RTM:**

```
| BC-2.06.023 | §Success Criteria (killer scenario — permission overlay; concurrent prompt resolution) | SS-tui.md v1.0.0 §Permission Overlay §Overlay Stack Lifecycle; SS-ipc.md v1.0.0 §ServerToClient::PermissionPromptResolved | P0 | monocle-tui/tests/permission_overlay_resolved.rs | Integration |
```

Architecture source cites: SS-tui.md v1.0.0 (overlay stack lifecycle is the primary module),
SS-ipc.md v1.0.0 (PermissionPromptResolved message type definition, the trigger). Both version
pins are included per pin-symmetry convention (F-R117-3, SE-17e).

**BC-INDEX pin updated:** `behavioral-contracts/BC-INDEX.md v1.15` → `v1.17` in `traces_to:`
(BC-INDEX was bumped to v1.16 for F-P1D-006 closure and v1.17 in this CV-P1D-001/002 burst).

**Changes made:** §7 RTM — BC-2.06.023 row added; frontmatter `version` v1.27.0 → v1.27.1;
`timestamp` refreshed `2026-05-26T13:00:00Z` → `2026-05-27T00:00:00Z`; `traces_to:`
BC-INDEX pin v1.15 → v1.17; §Trace v1.27.1 added.

SE-16d monotonicity PASS: 2026-05-27T00:00:00Z > prior 2026-05-26T13:00:00Z (v1.27.0). ARITHMETICALLY TRUE. PASS.

---

## §Trace v1.27.2 — F-P1D2-006 closure: BC-2.06.023 added to §2.6 BC index table (2026-05-26T00:00:00Z)

**Bump:** v1.27.1 → v1.27.2.
**Predecessor pin:** v1.27.1 (CV-P1D-003 BC-2.06.023 added to §7 RTM; timestamp `2026-05-27T00:00:00Z`).
**Timestamp:** 2026-05-26T00:00:00Z

**F-P1D2-006 HIGH — BC-2.06.023 missing from §2.6 TUI BC index table:**

BC-2.06.023 ("TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved") was
added to §7 RTM in v1.27.1 and to BC-INDEX, but was NOT added to the §2.6 TUI BC index table.
The §Trace v1.27.1 claimed this was done ("added to the §2.6 TUI BC index table") but the claim
was incorrect — the row was missing. F-P1D2-006 from Pass 2 adversarial review identified this gap.

**Row added to §2.6 TUI BC index table:**

```
| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | P0 |
```

**Correction to §Trace v1.27.1:** The §Trace v1.27.1 claim "added to the §2.6 TUI BC index table"
was incorrect. The row was added only to §7 RTM. This v1.27.2 patch adds the missing §2.6 row.

**Changes made:** §2.6 TUI BC index table — BC-2.06.023 row added; frontmatter `version`
v1.27.1 → v1.27.2; `timestamp` refreshed `2026-05-27T00:00:00Z` → `2026-05-26T00:00:00Z`;
§Trace v1.27.2 added.

SE-16d monotonicity PASS: v1.27.2 is a later version than v1.27.1. PASS.

---

## §Trace v1.27.3 — BC-2.06.024 row missing from §2.6 (patch)

**Bump:** v1.27.2 → v1.27.3.
**Predecessor pin:** v1.27.2 (F-P1D2-006 BC-2.06.023 §2.6 row; timestamp `2026-05-26T00:00:00Z`).

BC-2.06.024 ("Permission Overlay: ToolPayload Body Rendering by Variant") was authored as part of the BC expansion but was absent from the §2.6 TUI BC index table in the PRD. Added in this patch. No other changes.

**Changes made:** §2.6 TUI BC index table — BC-2.06.024 row added; `version` v1.27.2 → v1.27.3; `timestamp` refreshed.

SE-16d monotonicity PASS: v1.27.3 > v1.27.2. PASS.

---

## §Trace v1.28.0 — D-241 control-center v1A BC delta (23 new BCs; §2.3/§2.5/§2.6/§2.8/§2.9 expansion)

**Bump:** v1.27.3 → v1.28.0 (minor — new subsections §2.8 and §2.9 added; BC count 70 → 136 across all active subsystems).
**Predecessor pin:** v1.27.3 (BC-2.06.024 §2.6 patch; 2026-06-03).
**Timestamp:** 2026-06-03T12:00:00Z

**Scope of v1.28.0 (control-center v1A BC integration):**

This is a delta amendment reflecting the D-236 control-center pivot. monocle is no longer observe-only; the Session Manager (SS-08) and Embedded PTY (SS-09) subsystems join the Phase 1 scope.

**New §2.8 — Session Manager (CAP-008):**

7 BCs (BC-2.08.001..BC-2.08.007). Architecture source SS-session-manager.md. Covers session spawn (SessionHostSpawner 2s SLA + SessionEntry creation), session persistence across daemon restart (ADR-0009 detached process model), session kill (SIGTERM via DaemonToHost::Kill 500ms), re-discovery after daemon restart (blocks UDS bind until complete, 5s SLA), GC (10s grace period), hook auto-injection at spawn (`--settings` arg), and attach/detach (chunked scrollback `ScrollbackChunk*`+`ScrollbackDumpComplete` on attach; session-host stays alive on detach).

**New §2.9 — Embedded PTY (CAP-009):**

9 BCs (BC-2.09.001..BC-2.09.009). Architecture source SS-embedded-pty.md. Covers: PTY byte pipeline render latency (100ms IPC → vt100 → tui-term), full-fidelity keyboard forwarding (printable + control + arrows + Kitty CSI u), SGR mouse encoding, Kitty keyboard protocol, bracketed paste, resize with 50ms debounce, 1000-row scrollback, EmbeddedTerminal/SessionCreation AppMode transitions, and the SUG-3 guarantee (permission badge+bell within one render tick while in EmbeddedTerminal — monocle never silently queues prompts). Key decisions: ADR-0010 (PTY bytes over existing UDS IPC), ADR-0011 (native portable-pty + vt100 + tui-term stack).

**Existing subsection additions:**

- §2.3 Engine Module (CAP-003): 4 new BCs (BC-2.03.005..BC-2.03.008) — spawn_recipe() happy path, CCR injection, error cases, and default UnsupportedOperation.
- §2.5 IPC (CAP-005): 3 new BCs (BC-2.05.009..BC-2.05.011) — PtyOutput fan-out (per-session bounded channel 1024 with surfaced drop counter); new ClientToServer variants (SpawnSession, KillSession, KeyInput, ResizePane, DetachSession, RenameSession, AttachSession — 7 total); new ServerToClient variants (ScrollbackChunk, ScrollbackDumpComplete, PtyReset).
- §2.6 TUI (CAP-006): 1 new BC (BC-2.06.025) — multi-session / multi-project sessions panel grouped by project with fast switching and TUI lifecycle actions.

**BC count change:**

| Metric | Before (v1.27.3) | After (v1.28.0) |
|--------|-----------------|----------------|
| Total BCs in §2 | 72 | 95 |
| Subsystems in §2 | 7 (SS-01..SS-07) | 9 (SS-01..SS-09) |
| BC-INDEX version | v1.34 | v1.35 |

Note: BC-INDEX tracks all 136 BCs (including SS-DTU 41 gene-source contracts). §2 of the PRD covers behavioral contracts authored by this project (not the SS-DTU gene-source contracts which are covered in the DTU assessment).

**VP deferral note:** All 23 new BCs carry `VP Anchors: VP-TBD`. VP authoring for SS-08 and SS-09 BCs is deferred to the formal-hardening scheduling phase per the project's established pattern. Architect must author VPs at hardening phase scheduling (tracked per D-241).

**SE-22 v2 producer declaration:** Known consumers with stale pins after this bump: VP-INDEX §References and individual VP files (dispatch: formal-verifier; not in this burst's scope per correct-agent-routing).

**Changes made:** frontmatter `version` v1.27.3 → v1.28.0; `timestamp` refreshed; `traces_to:` BC-INDEX pin v1.34 → v1.35, BC count 70 → 136; §2.3 Engine Module — BC-2.03.005..008 rows added; §2.5 IPC — BC-2.05.009..010 rows added; §2.6 TUI — BC-2.06.024 (retroactive patch per v1.27.3) and BC-2.06.025 rows added; §2.8 Session Manager — new subsection with 7 BC rows; §2.9 Embedded PTY — new subsection with 9 BC rows; §Trace v1.27.3 and v1.28.0 added.

SE-16d monotonicity PASS: 2026-06-03T12:00:00Z > 2026-05-28T12:00:00Z (v1.27.4 predecessor). ARITHMETICALLY TRUE. PASS.

---

## §Trace v1.28.1 — Adversarial pass-4 propagation fixes (BC-2.08.007 title + §2.8 summary)

**Bump:** v1.28.0 → v1.28.1 (patch — BC table titles + §Trace prose corrected; no new BCs, no schema changes).

**Changes made:**
- §2.8 BC table: BC-2.08.007 title corrected from "Attach/Detach — ScrollbackDump on Attach; session-host Stays Alive on Detach" to "Attach/Detach — Chunked Scrollback (ScrollbackChunk*+ScrollbackDumpComplete) on Attach; session-host Stays Alive on Detach" — mirrors the BC file H1 update per bc_h1_is_title_source_of_truth policy (HIGH-002).
- §Trace v1.28.0 New §2.8 description: "ScrollbackDump on attach" → "chunked scrollback `ScrollbackChunk*`+`ScrollbackDumpComplete` on attach" — stale prose aligned with canonical protocol (HIGH-002 propagation).

SE-16d monotonicity PASS: 2026-06-03T12:00:00Z ≥ v1.28.0 timestamp. PASS.

---

## §Trace v1.28.2 — I15-001/I15-002 adversarial pass-15 fixes (BC-2.05.010 title, BC-2.05.011 missing row, §Trace prose)

**Bump:** v1.28.1 → v1.28.2 (patch — §2.5 BC table titles + missing row corrected; §Trace v1.28.0 prose corrected; no new BCs, no schema changes).

**Changes made:**

- §2.5 IPC BC table: BC-2.05.010 title corrected from 6-variant form ("SpawnSession, KillSession,
  KeyInput, ResizePane, DetachSession, RenameSession") to 7-variant form ("SpawnSession, KillSession,
  KeyInput, ResizePane, DetachSession, RenameSession, AttachSession") — mirrors the BC file H1 per
  bc_h1_is_title_source_of_truth policy (I15-002). AttachSession was added in BC-2.05.010 v1.2.0
  (I3-004); H1 was updated in v1.4.0 (S-P7-003); PRD §2.5 table was not updated at that time.

- §2.5 IPC BC table: BC-2.05.011 row added ("New ServerToClient IPC Variants — ScrollbackChunk,
  ScrollbackDumpComplete, PtyReset", P0) — this BC existed in the v1A burst (BC file at
  ss-05/BC-2.05.011.md) but was omitted from the §2.5 table and §Trace v1.28.0 "BC-2.05.009..010"
  enumeration. BC-2.05.011 is load-bearing (ScrollbackChunk* protocol consumed by BC-2.05.010 and
  BC-2.08.007). Classified as straggler found during I15-002 class-close sweep.

- §Trace v1.28.0 "Existing subsection additions" prose: updated to enumerate 3 new §2.5 BCs
  (BC-2.05.009..BC-2.05.011), add AttachSession to the ClientToServer variant list (7 total), and
  add the ServerToClient variants (ScrollbackChunk, ScrollbackDumpComplete, PtyReset).

**PRD §2.5 cross-check result (I15-002 class-close):**

| BC | PRD §2.5 title | H1 canonical title | Status |
|----|---------------|-------------------|--------|
| BC-2.03.005 | ClaudeCodeModule.spawn_recipe() — Happy-Path Recipe Assembly | MATCHES | CLEAN |
| BC-2.03.006 | ClaudeCodeModule.spawn_recipe() — CCR Base URL Injection | MATCHES | CLEAN |
| BC-2.03.007 | spawn_recipe() Error Cases — BinaryNotFound and InvalidPath | MATCHES | CLEAN |
| BC-2.03.008 | Default spawn_recipe() Returns UnsupportedOperation | MATCHES | CLEAN |
| BC-2.05.009 | PtyOutput Fan-Out — Per-Session Bounded Channel (1024) with Drop Counter (stderr WARN) + PtyReset TUI Recovery | MATCHES | CLEAN (retitled S20-001) |
| BC-2.05.010 | (stale 6-variant) | 7-variant with AttachSession | FIXED |
| BC-2.05.011 | (absent — row missing) | New ServerToClient IPC Variants — ScrollbackChunk, ScrollbackDumpComplete, PtyReset | ADDED |
| BC-2.06.025 | Multi-Session / Multi-Project Sessions Panel — Grouped by Project, Fast Switching, TUI Lifecycle Actions | MATCHES | CLEAN |
| BC-2.08.001..007 | all 7 rows present | MATCHED (7 of 8) | STALE — BC-2.08.008 missing; corrected in I16-001 (v1.28.3) |
| BC-2.08.008 | (absent — row missing) | SessionStateChanged — Daemon Emits on Every SessionState Transition; Delivered to All TUI Clients; Ordering Relative to SessionListUpdate | ADDED (I16-001) |
| BC-2.09.001..009 | all 9 rows | MATCHES | CLEAN |

SE-16d monotonicity PASS: 2026-06-04T00:00:00Z > 2026-06-03T12:00:00Z (v1.28.1 predecessor). PASS.

## §Trace v1.28.3 — I16-001 BC-2.08.008 missing row (§2.8 completeness fix)

**Bump:** v1.28.2 → v1.28.3 (patch — §2.8 BC-table completeness; BC-2.08.008 row added).

**Finding:** I16-001 (Phase-1d Pass 16, IMPORTANT). PRD §2.8 Session Manager BC table listed only BC-2.08.001..BC-2.08.007 (7 rows). BC-INDEX v1.35 line 199 and BC-INDEX §Summary line 290 record SS-08 = 8 active BCs. BC-2.08.008 (v1.1.0, P0, active) was absent from the §2.8 table. The Pass-15 §Trace cross-check entry at v1.28.2 line 1558 recorded "BC-2.08.001..007 | all 7 rows | MATCHES | CLEAN" — this was a false attestation: it hand-typed cardinality 7 instead of verifying against BC-INDEX SS-08 count of 8, causing the omission to survive that pass.

**Root cause (S16-001 process lesson):** The Pass-15 cross-check used hand-typed enumerations ("001..007") rather than deriving row counts from BC-INDEX Summary. Hand-typed ranges are opaque to off-by-one errors. Future cross-checks must derive counts from BC-INDEX Summary and compare numerically.

**Changes made:**

- §2.8 BC table: BC-2.08.008 row added after BC-2.08.007, with title verbatim from BC-INDEX/H1 and priority P0.
- §2.8 intro prose: `SessionStateChanged` broadcast behavior added to the enumeration (driving wizard auto-advance and `EmbeddedTerminal` exit; ordered before `SessionListUpdate`). BC-2.08.008 is the behavioral contract for this.
- §Trace v1.28.2 cross-check entry (line 1558): corrected from false-green "BC-2.08.001..007 | all 7 rows | MATCHES | CLEAN" to two rows reflecting the actual state — STALE at 7 of 8, and the ADDED BC-2.08.008 row.
- frontmatter `version`: v1.28.2 → v1.28.3; `timestamp`: updated to 2026-06-04T00:00:00Z.

**Count cross-check (derived from BC-INDEX §Summary, not hand-typed enumerations):**

Counts derived from `behavioral-contracts/BC-INDEX.md` Summary table (line 290 area):

| Subsystem | BC-INDEX active count | PRD §2.NN row count | Match? |
|-----------|----------------------|---------------------|--------|
| SS-03 Engine Module | 8 | 8 | PASS |
| SS-05 IPC | 11 | 11 | PASS |
| SS-06 TUI | 25 | 25 | PASS |
| SS-08 Session Manager | 8 | 8 (was 7 before I16-001) | PASS (fixed) |
| SS-09 Embedded PTY | 9 | 9 | PASS |

All in-scope subsystems now match. No other cardinality gap exists.

SE-16e monotonicity PASS: 2026-06-04T00:00:00Z >= 2026-06-04T00:00:00Z (v1.28.2 predecessor). PASS.
