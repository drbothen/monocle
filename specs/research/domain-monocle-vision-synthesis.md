---
document_type: vision-synthesis
level: ops
version: "1.1.3"
status: approved
producer: orchestrator
phase: pre-phase-0-vision
timestamp: 2026-05-17T16:30:00Z
inputs: [semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md, semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md, semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md, semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md, semport/zellij/zellij-pass-8-final-synthesis.md, semport/lazygit/lazygit-pass-8-final-synthesis.md, semport/claude-squad/claude-squad-pass-8-deep-synthesis.md, semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md, product-brief.md, architecture/SS-deps-pin-manifest.md, planning/oq-research.md]
input-hash: "e04ad7b"
traces_to: "v1.0 commit 2c2b676 (8-repo full-protocol ingest); JC-2/EX-2 closures via oq-research.md; SS-deps-pin-manifest.md as canonical pin source; adversary re-audit 0bd4ba9 vision-re-versioning recommendation"
project: monocle
approved_by: human
approved_at: 2026-05-12T00:00:00Z
approved_at_v1_0: 2026-05-11T20:30:00Z
---

# Monocle Vision Synthesis (v1.1.2, approved 2026-05-12)

## Vision Statement

One TUI lens over every Claude-class session you're running, every customization that shapes them, and every workflow driving them — across multiple harnesses and federated across hosts. Observe-only for state, action-only via overlays. You never leave your editor.

Today, a developer running three Claude Code sessions across two projects faces a fragmentation problem that no single tool solves. Sessions live in tmux windows you must context-switch between to check status. Customizations (CLAUDE.md files, settings.json permission lists, hook scripts, keybindings) are scattered across project trees and `~/.claude/` — you `cat` them to read them and lose track of which one is active for which session. Workflows (vsdd-factory STATE.md files, sprint-state.yaml, wave-gate status) sit in `.factory/` directories you `tree` to discover, with no unified view of what is blocking. Permission prompts demand you be in the right tmux window at exactly the right moment or the session stalls. Monocle collapses all of this into one `Ctrl-\` popup: a five-plane dashboard that never disrupts your editor focus, never forks a new terminal window, and never requires you to remember where anything lives.

## Five Planes

| Plane | Source genes | What it does |
|-------|-------------|--------------|
| Runtime | any-context/lazyclaude + zellij | Multi-harness session roster: lists all live Claude Code / CodeMachine / future-harness sessions, shows token burn rate, cost, wall-time, phase tag. Rust IPC via Unix domain socket + axum HTTP. WASM plugin SDK (zellij-tile model) for third-party session integrations. |
| Static | NikiforovAll/lazyclaude | Customization explorer: reads CLAUDE.md files, settings.json permission blocks, hook scripts, keybindings.json. Shows which customizations are active for the focused session. Trigger-trace from popup to the defining file — jump to the line that granted or denied a tool. |
| Workflow | vsdd-factory | Factory-awareness (observe-only): detects `.factory/STATE.md` + `document_type: pipeline-state` discriminator; surfaces phase, wave, blocking issues, and convergence trajectory for the focused session's project. Multi-repo signal: `.factory-project/`. First concrete adapter: VsddFactoryAdapter. Third-party adapters via WASM plugin (same SDK as runtime plane). |
| Harness | codemachine-cli + claude-squad + claude-code-router | EngineModule abstraction: each harness (Claude Code, CodeMachine, future) registers a profile in `~/.monocle/config.json`. Worktree-isolation pattern (claude-squad gene). CCR integration: detect on PATH, write per-session JSON, set `ANTHROPIC_BASE_URL` — integrate-external, not build-in. |
| TUI philosophy | lazygit | The canonical lazy* signature: context-aware Action enum dispatch, 5-level binding precedence (search-prompt > user-custom-commands > per-context > global > builtin), telescope help overlay, modal cascade with VecDeque stack, compile-time AppMode state machine replacing bag-of-Option fields. |

## Process Topology

```
User's tmux server (existing)
├── pane: editor (nvim / Zed / VS Code terminal)
│   └── Ctrl-\  ──────────────────────────────────────────────┐
│                                                              │
└── pane: monocle TUI client (connects to daemon)             │
                                                              ▼
monocle tmux server  (-L monocle socket, separate from user's)
└── session: monocle-daemon  (long-lived background process)
    ├── axum HTTP  :<os-port>  (hook POST receiver; OS-assigned per OQ-04; port written to lock file)
    ├── rmcp MCP   :<os-port>  (optional MCP bridge for future tooling; Phase 4 only per OQ-09)
    ├── russh      :<os-port>  (SSH tunnel for federated multi-host; Phase 4)
    ├── Arc<Broker<Event>>  (fan-out to all connected TUI clients)
    └── EngineModule registry  (per-harness adapters)

Claude Code subprocesses  (one per session)
├── SessionStart hook      ──► POST http://localhost:<port>/hooks/session-start
├── UserPromptSubmit hook  ──► POST http://localhost:<port>/hooks/prompt-submit
├── PreToolUse hook        ──► POST http://localhost:<port>/hooks/pre-tool-use
├── Notification hook      ──► POST http://localhost:<port>/hooks/notification
└── Stop hook              ──► POST http://localhost:<port>/hooks/stop

Broker fans events to:
├── TUI client A (local)
├── TUI client B (another terminal tab)
└── TUI client C (remote host, via russh tunnel)
```

The daemon is started once (`monocle daemon start`) and survives terminal closes. TUI clients connect and disconnect freely. The hook POST endpoints are the ingestion boundary — Claude Code subprocesses are unmodified; they simply fire their existing hook scripts that POST to the daemon.

The canonical Phase 1 hook set is 5 endpoints, locked by JC-2 (`PostToolUse` omitted to preserve gene-source parity per any-context BC-HOOK-007 canonical matrix) and EX-2 (`SessionStart` and `UserPromptSubmit` added per architect extension). Note that the daemon port is OS-assigned at startup (OQ-04 resolution), not fixed at 2748 as the v1.0 diagram implied; the lock file written at `runtime_dir/monocle.lock` carries the actual port for hook-script consumption. `PermissionRequest` was considered as a 6th endpoint (OQ-M3) and resolved "stay at 5" — the `PreToolUse` + `Notification` pair captures all permission-relevant signal; revisit if Phase 2 trigger-trace UX surfaces a signal gap.

## Workspace Layout

```
monocle/
├── Cargo.toml                    # workspace manifest
└── crates/
    ├── monocle-core/             # pure types: Event, Action, EngineMetadata, AppMode; no I/O
    ├── monocle-runtime/          # async daemon: axum server, broker, EngineModule registry, russh
    ├── monocle-tui/              # ratatui renderer: panels, overlays, keybinding dispatch
    ├── monocle-static/           # customization reader: CLAUDE.md, settings.json, hooks, keybindings
    ├── monocle-workflow/         # factory-awareness: STATE.md parser, FactoryAdapter trait, VsddFactoryAdapter
    ├── monocle-plugin-sdk/       # WASM plugin ABI: EngineModule + FactoryAdapter for third-party plugins
    ├── monocle-ipc/              # Unix domain socket + shared-memory ring buffer (zero-copy event stream)
    ├── monocle-config/           # ~/.monocle/config.json: harness profiles, binding overrides, CCR path
    ├── monocle-proto/            # prost-generated protobuf types for cross-host federation wire format
    ├── monocle-fuzz/             # cargo-fuzz targets for parser and hook endpoint fuzzing
    ├── monocle-test-harness/     # integration test scaffolding: fake Claude Code subprocess, fake hooks
    └── monocle/                  # binary crate: clap CLI, daemon entrypoint, TUI entrypoint
```

Mirrors zellij's `-utils/-server/-client/-tile` split but renamed for monocle's domain. `monocle-core` is the zero-dependency pure-types crate that all other crates depend on. The binary crate depends on everything; no other crate is allowed to depend on the binary.

## Key Abstractions

### EngineModule

The multi-harness gene from codemachine-cli. Every AI coding harness (Claude Code, CodeMachine, future) is a plugin implementing this trait:

```rust
/// Implemented by each AI coding harness adapter.
pub trait EngineModule: Send + Sync + 'static {
    /// Unique stable identifier, e.g. "claude-code", "codemachine".
    fn id(&self) -> &'static str;

    /// Human-readable metadata surfaced in the sessions panel header.
    fn metadata(&self) -> EngineMetadata;

    /// Detect whether a running process is managed by this harness.
    /// Called on every new process event; must be cheap (no I/O).
    fn detect(&self, proc: &ProcessSnapshot) -> bool;

    /// Enrich a raw ProcessSnapshot with harness-specific fields
    /// (token counts, cost, phase tag). May do I/O; runs off the hot path.
    async fn enrich(&self, proc: &ProcessSnapshot) -> EnrichedSession;

    /// Handle a hook event POSTed from a subprocess managed by this harness.
    async fn on_hook(&self, event: HookEvent) -> HookResponse;
}

pub struct EngineMetadata {
    pub display_name: &'static str,
    pub icon: char,           // single char shown in sessions panel
    pub config_path: PathBuf, // e.g. ~/.claude/ or ~/.codemachine/
    pub hook_schema: &'static str, // JSON schema for hook payloads
}
```

Built-in: `ClaudeCodeModule`. Second built-in: `CodeMachineModule`. Third-party: WASM plugin implementing the same ABI via `monocle-plugin-sdk`.

### Action enum + Binding

The lazy* signature pattern from lazygit, adapted for Rust:

```rust
/// Every user-triggerable operation in monocle. Enum variants (not closures)
/// keep bindings Eq + inspectable for the telescope help overlay.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // Navigation
    FocusPanel(PanelId),
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    FilterToggle,
    FilterType(char),

    // Session actions
    SessionSelect,
    SessionKill,
    SessionAttach,

    // Permission prompt
    PermissionAcceptOnce,
    PermissionAcceptAlways,
    PermissionReject,
    PermissionTraceToSource,

    // Overlay control
    OverlayOpen(OverlayKind),
    OverlayClose,
    OverlayCycleNext,

    // System
    DaemonRestart,
    ConfigReload,
    Quit,
}

/// A resolved key → action mapping after applying precedence.
pub struct Binding {
    pub key: KeyEvent,
    pub action: Action,
    pub source: BindingSource, // which precedence level resolved this
}

/// 5-level precedence (highest → lowest), matching lazygit's model.
pub enum BindingSource {
    SearchPrompt,       // active filter input captures all printable keys
    UserCustomCommand,  // user-defined custom commands in config
    PerContext,         // bindings declared for the current AppMode panel
    Global,             // bindings active in all modes
    Builtin,            // factory defaults in monocle-core
}
```

The dispatcher walks the precedence stack in order and stops at the first matching binding. This gives users full control: a user custom command for `q` in the sessions panel beats the builtin `Quit` action.

### AppMode state machine

Compile-time mutual exclusion replaces NikiforovAll's bag-of-`Option` fields:

```rust
/// Exactly one AppMode is active at a time. The compiler enforces this.
#[derive(Clone, PartialEq, Eq)]
pub enum AppMode {
    /// Normal dashboard view. FocusSnapshot records which panel had focus
    /// before any overlay opened — so overlay-close restores the correct panel.
    Dashboard { focused: FocusSnapshot },

    /// Telescope-style filter input is active over the focused panel.
    Filtering { panel: PanelId, query: String, prior: FocusSnapshot },

    /// A modal overlay is open (permission prompt, detail view, help).
    /// VecDeque<PromptModal> fixes lazygit's single-popup drop-on-concurrent:
    /// new prompts push_back; OverlayCycleNext rotates; Esc pops_front.
    Overlay { stack: VecDeque<PromptModal>, prior: FocusSnapshot },

    /// Full-screen view of a single panel (Enter key from Dashboard).
    Fullscreen { panel: PanelId, prior: FocusSnapshot },
}

/// Which panel had focus before an overlay/filter/fullscreen transition.
/// Explicit enum fixes NikiforovAll's gap where modal-close from MainPane
/// focus loses the MainPane fact (was stored as None in Option<Panel>).
#[derive(Clone, PartialEq, Eq)]
pub enum FocusSnapshot {
    Sessions,
    Preview,
    Workflow,
    Customizations,
    EventRibbon,
}
```

State transitions are pure functions in `monocle-core`: `fn transition(mode: AppMode, action: Action) -> AppMode`. No `Arc<Mutex<Option<...>>>` chains; no runtime panics on None unwraps.

### FactoryAdapter

The workflow-plane plugin contract. Detection canonical signal: `document_type: pipeline-state` in `.factory/STATE.md`. Multi-repo signal: `.factory-project/` directory present alongside `.factory/`.

```rust
/// Implemented by factory-pattern workflow adapters.
pub trait FactoryAdapter: Send + Sync + 'static {
    /// Stable identifier, e.g. "vsdd-factory", "custom-factory".
    fn id(&self) -> &'static str;

    /// Detect whether a project directory is managed by this factory.
    /// Canonical signal: .factory/STATE.md with document_type: pipeline-state.
    /// Must be synchronous and cheap — called on every directory scan.
    fn detect(&self, project_root: &Path) -> bool;

    /// Parse the factory state from the project directory.
    /// Returns the workflow surface monocle surfaces in the Workflow panel.
    async fn read_state(&self, project_root: &Path) -> Result<FactoryState>;

    /// Called when the workflow file changes (notify watcher event).
    async fn on_change(&self, project_root: &Path, changed: &Path) -> Result<FactoryState>;
}

pub struct FactoryState {
    pub phase: String,
    pub status: String,
    pub awaiting: Option<String>,
    pub blocking_issues: Vec<BlockingIssue>,
    pub convergence: Option<ConvergenceInfo>,
    pub cycle: Option<String>,
    pub custom_fields: HashMap<String, serde_yaml::Value>, // adapter-specific extras
}
```

Built-in: `VsddFactoryAdapter` (reads `.factory/STATE.md`, parses YAML frontmatter + Phase Progress table). Third-party adapters via WASM plugin implementing the same ABI, loaded from `~/.monocle/plugins/`.

## TUI Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│  monocle  [project: monocle]  3 sessions  2 workflows  Ctrl-\ hide  │
├────────────────────────┬──────────────────┬────────────────────────┤
│  [1] Sessions          │  [2] Preview     │  [3] Workflow          │
│                        │                  │                        │
│  ● CC  monocle  phase0 │  session detail  │  vsdd-factory          │
│  ● CC  blog     wave-2 │  token: 142k     │  phase: pre-phase-0    │
│  ● CM  api-svc  idle   │  cost: $0.83     │  status: active        │
│                        │  hooks: 47       │  awaiting: human GO    │
│  / filter  ? help      │  uptime: 00:42   │  blocking: 0           │
│                        │                  │  cycle: cycle-001      │
│                        │                  │                        │
├────────────────────────┴──────────────────┴────────────────────────┤
│  [4] Customizations                                                  │
│  CLAUDE.md: /Users/jmagady/Dev/monocle/CLAUDE.md  (active)          │
│  settings:  allowedTools [Bash,Read,Edit,Write,...] + 3 hooks        │
│  keybinds:  12 custom  /  48 builtin                                 │
├─────────────────────────────────────────────────────────────────────┤
│  [5] Events                                                          │
│  20:29:01  PreToolUse    Bash  monocle-session-1                     │
│  20:29:00  Notification  info  monocle-session-1  12ms               │
│  20:28:58  PreToolUse    Edit  blog-session-2  PENDING               │
├─────────────────────────────────────────────────────────────────────┤
│  Tab: cycle panels  Enter: fullscreen  ?: help  /: filter  q: quit  │
│  breadcrumb: Dashboard > Sessions                                    │
└─────────────────────────────────────────────────────────────────────┘
```

Permission prompt popup overlay (cascades at parent.x+2, parent.y+1):

```
┌─ Permission Prompt  [1/2] ──────────────────────────────────────────┐
│  Session: blog-session-2  (Claude Code)                             │
│  Tool: Edit                                                         │
│  Path: /Users/jmagady/Dev/blog/posts/draft.md                      │
│                                                                     │
│  Diff preview:                                                      │
│  - old content line                                                 │
│  + new content line                                                 │
│                                                                     │
│  [1] Accept once   [2] Accept always   [3] Reject                  │
│  [t] Trace to source customization                                  │
│  [↑↓] switch prompts (2 queued)   [Esc] hide overlay               │
└─────────────────────────────────────────────────────────────────────┘
      ┌─ Permission Prompt  [2/2] ──────────────────────────────────┐
      │  Session: api-svc-session-3  (CodeMachine)                  │
      │  Tool: Bash                                                 │
      │  Command: cargo build --release                             │
      │  [1] Accept once  [2] Accept always  [3] Reject             │
      └─────────────────────────────────────────────────────────────┘
```

The VecDeque stack means both prompts are queued simultaneously. `[↑↓]` rotates the stack. `[Esc]` hides the overlay without rejecting — the prompts remain queued and re-appear on next `Ctrl-\`. `[t]` trace-to-source opens the Static plane detail showing which settings.json permission rule (or lack thereof) triggered this prompt.

## End-to-End Killer Scenario

Setup: developer has three sessions running — `monocle-session-1` (Claude Code, project `monocle`, vsdd-factory phase pre-phase-0), `blog-session-2` (Claude Code, project `blog`, wave 2), `api-svc-session-3` (CodeMachine, project `api-svc`, idle). Two permission prompts arrive concurrently from different sessions. Developer is editing code in nvim.

1. `blog-session-2` fires a `PreToolUse` hook with `decision_required: true` — Edit on `draft.md`. Daemon receives POST, queues `PromptModal` in overlay stack, sends push notification to TUI client.
2. `api-svc-session-3` fires a `PreToolUse` hook with `decision_required: true` — Bash `cargo build --release`. Second `PromptModal` pushed to stack. TUI badge shows `2 prompts`.
3. Developer presses `Ctrl-\` from nvim. Monocle popup appears (monocle tmux pane floats over editor). AppMode transitions to `Overlay { stack: [blog, api-svc], prior: Sessions }`. Both prompts visible: front prompt (blog) in focus.
4. Developer reads diff for blog Edit. Presses `2` (Accept always). Daemon sends `{"decision": "always"}` back to `blog-session-2` hook response. `blog-session-2` unblocks and continues. Overlay stack pops front; `api-svc` prompt becomes front.
5. Developer reads `cargo build --release`. Presses `1` (Accept once). Daemon sends `{"decision": "accept"}` to `api-svc-session-3`. Unblocks. Overlay stack now empty; AppMode restores to `Dashboard { focused: Sessions }`.
6. Developer presses `Ctrl-\` again to dismiss popup. Returns to nvim.
7. Total interactions: 3 keystrokes (`Ctrl-\`, `2`, `1`, `Ctrl-\` = 4 keys; 0 context switches between tmux windows; 0 `tmux select-window` commands; 0 sessions stalled due to missed prompt).

Contrast with today: developer would need to `Ctrl-b n` to find `blog-session-2`'s tmux window, read the inline permission prompt, type a response, `Ctrl-b n` again to find `api-svc-session-3`, repeat. If they missed the `api-svc` prompt and the session timed out, they would need to restart that session's task.

## Explicit Non-Goals

- Does NOT execute workflows — monocle is observe-only for factory/workflow state; it never writes STATE.md, never triggers phases, never dispatches agents
- Does NOT write STATE.md — the factory adapter reads STATE.md; monocle never mutates it
- Does NOT route LLM API requests — claude-code-router integration is detect-on-PATH + config-write; monocle does not proxy or modify LLM traffic
- Does NOT replace the terminal multiplexer — monocle runs inside tmux, does not replace it; zellij's multiplexer internals are a Leave-behind gene
- Does NOT include PM/Worker multi-agent orchestration — any-context/lazyclaude's PM/Worker subsystem is explicitly out of scope; the human is always the coordinator
- Does NOT own session transcripts — monocle reads hook events (fine-grained, ephemeral); full transcript storage belongs to Claude Code's own persistence layer
- Does NOT build its own LLM provider abstraction — CCR is the external router; monocle integrates by detecting it, not by reimplementing it

## Tech Stack

**Canonical version pin manifest:** see `.factory/specs/architecture/SS-deps-pin-manifest.md`, which carries live-crates.io-verified pins and the RUSTSEC audit context. The pin manifest supersedes the version examples that appeared in v1.0 of this vision. The architectural intent of each crate (ratatui for TUI, crossterm as backend, tokio for async runtime, axum for hook ingestion, interprocess for IPC, prost for cross-host wire format, serde_yaml_ng for config, wasmtime for Phase 3 SDK, nucleo for fuzzy matching, similar for diff preview, directories for XDG paths, notify for FS watching, russh for federation, rmcp for MCP bridge, tempfile for atomic writes, clap for CLI, arboard for clipboard, tracing for instrumentation, thiserror+anyhow for error handling, reqwest for HTTP client) remains the same as v1.0. What changed: each pin was verified against crates.io between 2026-05-11 (v1.0) and 2026-05-12 (this v1.1), and updated to the current stable major.minor with explicit RUSTSEC justification where pre-current versions had known advisories. See `SS-deps-pin-manifest.md` §RUSTSEC Audit Context for the per-crate advisory list.

## Phase Plan

| Phase | Name | Scope | Exit criterion |
|-------|------|-------|---------------|
| 1 | Runtime core | monocle-core, monocle-runtime, monocle-ipc, monocle-config, monocle binary (daemon mode), ClaudeCodeModule, axum hook endpoints, broker, Sessions panel (TUI stub) | `monocle daemon start` receives hook POSTs from a real Claude Code session; sessions panel shows live session list with token counts |
| 2 | Static plane | monocle-static, Customizations panel, trigger-trace (permission prompt → settings.json line), nikiforovall AppMode state machine, lazygit binding dispatch (5-level precedence), telescope help overlay | Permission prompt overlay works end-to-end; `[t]` trace-to-source jumps to correct settings.json line; help overlay lists all bindings with source |
| 3 | Workflow plane | monocle-workflow, FactoryAdapter trait, VsddFactoryAdapter, Workflow panel, STATE.md live watcher, monocle-plugin-sdk (WASM ABI for EngineModule + FactoryAdapter) | Workflow panel shows phase/status/blocking-issues for a vsdd-factory project in real time; WASM plugin loads a third-party FactoryAdapter |
| 4 | Cross-plane + multi-harness + federation | CodeMachineModule, monocle-proto (protobuf wire), russh federation tunnel, multi-host roster, OTel cost/token panel, CCR integration (detect + config-write), rmcp MCP bridge (optional) | Two hosts federated: monocle TUI on host A shows sessions from host B; CCR detected and per-session routing config written automatically |

## Closure Log (v1.0 to v1.1)

**What this version captures:** The human's intent from the v1.0 approval event (2026-05-11) AS REFINED by:

- JC-1 (7-customization-type scope) — moved to Phase 2 Exit Criteria, not Phase 1 (resolves contradiction in v1.0 success criteria)
- JC-2 (5 hook endpoints, `PostToolUse` omitted) — canonical endpoint set locked
- JC-3 (port 2748 fixed vs OS-assigned) — OS-assigned per OQ-04
- EX-1 (13-crate workspace → 12-crate workspace per enumeration; see brief v1.4 for correct count)
- EX-2 (`SessionStart` + `UserPromptSubmit` added to Phase 1 endpoint set)
- OQ-01..OQ-11 resolutions per `.factory/planning/oq-research.md` (b3c68ca)
- OQ-M1 (agent-view IPC coexistence): resolved — agent view uses internal Claude Code IPC, no port/auth collision with monocle's outbound hook POSTs
- OQ-M2 (claude-manager hook protocol): resolved — claude-manager uses tmux pane management, NOT hook protocol; monocle's hook-native moat is intact
- OQ-M3 (`PermissionRequest` as 6th endpoint): resolved — stay at 5 endpoints per JC-2 parity argument; revisit if Phase 2 trigger-trace UX surfaces signal gap
- R-001 (Anthropic deepens agent view): probability red-lined from market-intel's 25–40% to <10% per human Q-B response (2026-05-12). At this probability, no separate mitigation scaffolding is required beyond the production-grade depth monocle is already shipping. Noted as informational background; brief v1.4.1 reflects the revised assessment.

**What this version does NOT change:** The vision's core thesis ("one TUI lens over every Claude-class session"), the five-plane architecture, the observe-only-for-state / action-only-for-overlays principle, the killer scenario (4 keys per any concurrent permission prompt pair), the gene-transfusion methodology, or the multi-harness EngineModule trait abstraction. All architectural intent from v1.0 is preserved.

## Provenance

This vision synthesis was produced by the orchestrator agent after the full-protocol brownfield-ingest of 8 reference repos completed (factory-artifacts commit 2c2b676). The human approved it verbatim 2026-05-11 with the statement "I agree with this fully". It is the pre-brief vision document that downstream agents (product-owner for `/vsdd-factory:create-brief`, architect for `/vsdd-factory:create-architecture`, disposition-pass agents) must reference.

The vision is the synthesis lens for disposition decisions: every subsystem in every reference repo gets sorted into Model / Take-but-reimplement / Enhance / Leave-behind through THIS vision. If a future vision-doc change invalidates prior dispositions, those dispositions must be re-run.

The vision is intentionally opinionated. It does NOT enumerate every option; it states the chosen direction. Alternative directions discussed in the synthesis bursts but rejected: build-in LLM routing (rejected — integrate CCR externally), inherit PM/Worker orchestration (rejected by user direction), execute workflows (rejected — observe-only).

v1.1 was drafted by the business-analyst agent on 2026-05-12 to capture the JC/EX/OQ-M closures and version-pin updates from the OQ research and market intel work that followed the v1.0 approval.

v1.1 re-approved by the human on 2026-05-12 during the production-grade remediation burst Phase 1 gate review. R-001 probability red-lined from market-intel's 25–40% estimate to <10% during the same review; brief v1.4.1 reflects the revised assessment.

v1.1.1 (2026-05-12): Surgical frontmatter and §Tech Stack pointer fixes — `dependencies.md` references updated to canonical `SS-deps-pin-manifest.md`; `approved_at` corrected to reflect 2026-05-12 v1.1 re-approval (original v1.0 approval preserved as `approved_at_v1_0`). Substantive content unchanged. Resolves consistency audit F-01-B and F-04-I (commit 0f28619).

v1.1.2 (2026-05-12): Surgical path fix — §Process Topology diagram endpoint `/hooks/user-prompt-submit` corrected to `/hooks/prompt-submit` to match canonical brief and DTU paths. Resolves adversary F-NEW-01 (CRITICAL wire-protocol divergence across 3 artifacts). No content change.
