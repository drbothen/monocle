---
document_type: vision-synthesis
level: ops
version: "2.2.2"
status: approved
producer: product-owner
phase: pre-phase-0-vision
timestamp: 2026-06-03T23:30:00Z
inputs:
  - semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md
  - semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md
  - semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md
  - semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md
  - semport/zellij/zellij-pass-8-final-synthesis.md
  - semport/lazygit/lazygit-pass-8-final-synthesis.md
  - semport/claude-squad/claude-squad-pass-8-deep-synthesis.md
  - semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md
  - product-brief.md
  - architecture/SS-deps-pin-manifest.md
  - planning/oq-research.md
  - NEXT-SESSION-PIVOT.md
  - semport/DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md
  - specs/research/embedded-pty-evaluation.md
input-hash: "020ce73"
traces_to: >
  v1.0 commit 2c2b676 (8-repo full-protocol ingest); JC-2/EX-2 closures via
  oq-research.md; SS-deps-pin-manifest.md as canonical pin source; adversary
  re-audit 0bd4ba9 vision-re-versioning recommendation;
  D-236 product-vision pivot (2026-06-03) human-directed;
  D-237 human ratification of re-baselined v1 control-center scope (2026-06-03);
  DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md v1.0;
  embedded-pty-evaluation.md v1.0;
  D-238 human gate escalation (2026-06-03) — graceful daemon-process restart
  must survive (session-host-owns-PTY model, native preferred, no-tmux preserved)
project: monocle
approved_by: Joshua Magady
approved_at: 2026-06-03
approved_at_v1_0: 2026-05-11T20:30:00Z
approved_at_v1_1: 2026-05-12T00:00:00Z
---

# Monocle Vision Synthesis (v2.2 — Consistency Propagation, Approved)

## Amendment History

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-11 | orchestrator | Initial observe-only vision. Human-approved ("I agree with this fully"). |
| 1.1 | 2026-05-12 | business-analyst | JC/EX/OQ-M closures; version-pin updates to SS-deps-pin-manifest.md. Re-approved by human 2026-05-12. |
| 1.1.1 | 2026-05-12 | business-analyst | Surgical frontmatter and §Tech Stack pointer fixes. |
| 1.1.2 | 2026-05-12 | business-analyst | Surgical path fix — `/hooks/prompt-submit` wire correction. |
| **2.0** | **2026-06-03** | **product-owner** | **D-236/D-237 CONTROL-CENTER PIVOT.** Retired: observe-only constraint; all specific "rejected" non-goals that blocked launching/orchestration. Added: LAUNCH, EMBEDDED PTY, MULTI-SESSION/MULTI-PROJECT, INTERACTIVE TUNE as first-class v1 capabilities. DAEMON-OWNS-PTY persistence model. Hook auto-injection on spawn. v1A/v1B wave ordering. See §Retired Constraints and §v1 Capability Set. Status: draft — pending human approval gate before architecture delta proceeds. Reconciled to human-ratified decisions 2026-06-03: full keyboard fidelity (mouse + Kitty keyboard protocol) IN v1A; Q-3/Q-5/Q-6 in §Open Questions converted to RESOLVED; wave-split adjudication note at §Wave Plan updated to reflect human ratification. Applied spec-review patch (SR-001..SR-005, SR-007): keyboard scope supersession pointer added; stale softener sentence replaced with affirmation; Q-4 merged into Q-1 benchmark gate; persistence model three-case disambiguation; workspace legend exists-today vs. aspirational; tui-term Q-7 softened to confirm-posture framing. |
| **2.1** | **2026-06-03** | **product-owner** | **D-238 (human gate, 2026-06-03): v1A persistence ESCALATED — graceful daemon-process restart must SURVIVE.** Changed: CASE 2 (graceful daemon restart) now requires sessions survive via a native detached per-session session-host mechanism that owns PTY masters + child processes and outlives the daemon process (abduco/dtach-style); daemon coordinates and re-attaches on restart. Persistence principle renamed from DAEMON-OWNS-PTY to session-host-owns-PTY; daemon coordinates/re-attaches (no-tmux preserved as default; external supervisor only as architect-surfaced fallback for human decision). CASE 1 (TUI exit) unchanged; CASE 3 (hard crash → lost) unchanged. §Open Questions: new HIGH-priority item Q-8 for PTY-ownership-survival mechanism. Doc APPROVED by Joshua Magady to proceed to brief→architecture→story delta. |
| **2.2** | **2026-06-03** | **product-owner** | **Consistency propagation of v2.1 session-host model + architect rulings; no scope change.** Applied ADR-0009 (native session-host process model) and SS-session-manager.md (SS-08) to all stale descriptions: §Process Topology ASCII diagram updated to show session-host processes owning PTYs + daemon coordinating; §SessionManager description corrected from in-process PTY ownership to session-host coordinator; `PtySpawner`/`RealPtySpawner`/`MockPtySpawner` replaced throughout with `SessionHostSpawner`/`RealSessionHostSpawner`/`MockSessionHostSpawner` (SS-08 canonical trait); `portable-pty` and `vt100` crate locations corrected to `monocle-session-host` (not `monocle-runtime`) per SS-deps-pin-manifest-v2-delta; `VsddFactoryAdapter`/`FactoryAdapter` extraction source corrected to `monocle-core` (not `monocle-runtime`) per IMP-1 (live codebase: `crates/monocle-core/src/factory/`); `EmbeddedTerminal { session_id }` typed as `String` (not `Uuid`) per SS-08 session_id canonical ruling (IMP-2); `Detached` state comment corrected for session-host ownership (SUG-1); §Wave Plan updated to name `monocle-session-host` binary as the v1A deliverable (SUG-2); permission badge+bell guarantee for embedded-terminal mode documented; v1B embedded-terminal pre-emption noted as open item for human ratification (SUG-3). Status: APPROVED (no scope change). |

**What D-236/D-237 retired (precise list):**
- The vision statement phrase "Observe-only for state, action-only via overlays"
- The Non-Goal "Does NOT execute workflows — monocle is observe-only"
- The Non-Goal "Does NOT replace the terminal multiplexer" (partially: monocle IS a mux for AI sessions; does not replace the user's terminal mux — see §Non-Goals)
- The Provenance statement "execute workflows (rejected — observe-only)"
- The Provenance statement "inherit PM/Worker orchestration (rejected by user direction)" — these were correct for the original scope but are reversed for the control-center inversion: monocle now SPAWNS and OWNS sessions
- All residual "observe-only" qualifiers in the Five Planes descriptions
- Phase Plan Phases 2-4 (the old roadmap) — superseded by the re-baselined v1 wave plan

**What D-236/D-237 preserved (unchanged from v1.1.2):**
- The five-plane architecture (updated descriptions only)
- EngineModule + FactoryAdapter traits (extended, not replaced)
- The killer scenario (4-keystroke permission resolution) — now amplified by launch ownership
- The lazygit interaction philosophy (5-level binding precedence, VecDeque popup stack, Action enum)
- The 5-endpoint hook protocol and DTU clone (dtu-claude-code-hooks-v1)
- AppMode state machine (extended with new variants)
- All Phase-1 substrate (daemon, hook ingestion, permission overlay, TUI rendering, profile picker)
- CCR integration (detect-on-PATH + config-write; now also used in spawn path)
- Gene-source LEAVE-BEHIND verdicts for: PM/Worker orchestration, capture-pane scraping, zellij-as-library, inter-session bus, SSH federation (Phase 4 suspended)

---

## Vision Statement

One TUI control center for every Claude-class session you need to run — launch it, watch it, talk to it, tune it, and resolve its permission prompts without ever leaving the TUI. Many sessions, many projects, fully managed from a single `Ctrl-\` popup.

> *"We need to be able to launch, manage, and observe — a better lazyclaude, a better
> claude-squad. We should never have to leave the TUI and we should be able to manage
> as many sessions and as many projects with sessions as we need to. We need to be able
> to run, launch, manage, observe, tune, control — everything from the TUI."*
> — Human direction, D-236 (2026-06-03), verbatim.

Today, a developer running three Claude Code sessions across two projects faces a fragmentation problem that no single tool solves. Sessions live in tmux windows you must context-switch between to check status. Customizations (CLAUDE.md files, settings.json permission lists, hook scripts, keybindings) are scattered across project trees and `~/.claude/` — you `cat` them to read them and lose track of which one is active for which session. Workflows (vsdd-factory STATE.md files, sprint-state.yaml, wave-gate status) sit in `.factory/` directories you `tree` to discover, with no unified view of what is blocking. Permission prompts demand you be in the right tmux window at exactly the right moment or the session stalls. **You cannot even launch a new Claude session from inside the tool that monitors all of them.**

Monocle collapses all of this into one `Ctrl-\` popup: a five-plane control center that never disrupts your editor focus, never forks a new terminal window, and never requires you to remember where anything lives. You spawn sessions from here. You watch them run inside the same panel. You resolve permissions here. You tune your configuration here. You never leave.

The architectural inversion from v1.1.2: monocle's daemon no longer merely *receives* hooks from sessions you launched elsewhere. **The daemon now spawns and owns sessions.** The TUI is a rich client that streams PTY output, forwards keystrokes, and manages session lifecycle over the existing UDS IPC. This is the core of the change.

---

## Retired Constraints (D-236 — explicit reversals)

The following constraints appeared in v1.1.2 and are **explicitly retired** as of this version. They were correct for the observe-only scope; the control-center pivot reverses them.

| Retired constraint | Where it appeared in v1.1.2 | Why reversed |
|-------------------|-----------------------------|--------------|
| "Observe-only for state, action-only via overlays" | Vision Statement, Five Planes | The fundamental inversion: monocle now LAUNCHES and OWNS sessions, not just observes them |
| "inherit PM/Worker orchestration — rejected" | §Provenance | The pivot is not PM/Worker orchestration; it is direct session spawning. The human is still the coordinator. Monocle spawns sessions on demand; it does not orchestrate multi-agent pipelines. This was not a reversal of the "human is coordinator" principle — it was a reversal of the "cannot spawn" constraint. |
| "execute workflows — rejected — observe-only" | §Provenance | monocle still does NOT execute vsdd-factory workflows or dispatch factory agents. The reversal is: monocle can spawn AI harness sessions (user-initiated). Factory adapter remains observe-only (reads STATE.md, never mutates it). |
| "Does NOT execute workflows — monocle is observe-only for factory/workflow state" | §Non-Goals | Retired as a blanket statement. The specific non-goal (never mutate STATE.md, never dispatch factory agents) is preserved but the "observe-only" label is retired. |
| "Does NOT replace the terminal multiplexer" | §Non-Goals | Partially revised: monocle IS a multiplexer for AI coding sessions (manages multiple PTYs). It still does NOT replace the user's general-purpose terminal multiplexer (tmux/zellij for non-monocle work). |

**Confirming what the pivot does NOT reverse:**
- PM/Worker inter-session orchestration — still out of scope; the human is always the coordinator
- vsdd-factory workflow dispatch — still out of scope; FactoryAdapter is still read-only
- LLM API routing / CCR proxy — monocle detects CCR and injects config; does not proxy traffic
- Full-transcript storage — belongs to Claude Code's own persistence layer
- LLM provider abstraction — CCR is the external router
- zellij-as-library — architecture model only; not a code dependency

---

## Five Planes

| Plane | Source genes | What it does (v2.0 — control-center model) |
|-------|-------------|---------------------------------------------|
| Runtime | any-context/lazyclaude + zellij + claude-squad | **Session lifecycle center**: spawn, attach, detach, kill, and rename harness sessions from the TUI. One PTY per session, owned by a detached `monocle-session-host` process (session-host-owned; daemon coordinates). Streams terminal output into the embedded PTY pane. Shows live session roster: token burn rate, cost, wall-time, phase tag. Rust IPC via Unix domain socket (PTY bytes + hook events + control messages) + axum HTTP (hook ingestion). WASM plugin SDK (zellij-tile model) deferred to Phase 3 (suspended). |
| Static | NikiforovAll/lazyclaude | **Interactive customization center** (v1B Tune wave): reads AND edits CLAUDE.md files, settings.json permission blocks, hook scripts, keybindings.json. Shows which customizations are active for the focused session. Edit bindings in-place, apply profiles, manage CCR routing slots — interactive, not just browse. Trigger-trace from popup to the defining file — jump to the line that granted or denied a tool. |
| Workflow | vsdd-factory | Factory-awareness (observe-only, unchanged): detects `.factory/STATE.md` + `document_type: pipeline-state` discriminator; surfaces phase, wave, blocking issues, and convergence trajectory for the focused session's project. Pre-populated from session launch `project_root` (new trigger path). Multi-repo signal: `.factory-project/`. First concrete adapter: VsddFactoryAdapter. Third-party adapters via WASM plugin (Phase 3, suspended). |
| Harness | codemachine-cli + claude-squad + claude-code-router | EngineModule abstraction: each harness (Claude Code, CodeMachine, future) registers a profile and implements `spawn_recipe()` — the binary, args, and env needed to launch a session. worktree-per-session isolation (claude-squad gene). CCR integration: detect on PATH, write per-session JSON, set `ANTHROPIC_BASE_URL` — integrate-external, not build-in. Hook auto-injection on spawn: `--settings <hooks_settings_path>` in the launch args; no manual copy required. |
| TUI philosophy | lazygit | The lazygit control-center signature: context-aware Action enum dispatch, 5-level binding precedence (search-prompt > user-custom-commands > per-context > global > builtin), telescope help overlay, modal cascade with VecDeque stack, compile-time AppMode state machine. The Preview pane slot hosts the embedded PTY widget (tui-term) in EmbeddedTerminal mode. Single `Ctrl-\` popup; the user lives and acts inside it. |

---

## Process Topology

```
User's tmux server (existing)
├── pane: editor (nvim / Zed / VS Code terminal)
│   └── Ctrl-\  ──────────────────────────────────────────────┐
│                                                              │
└── pane: monocle TUI client (connects to daemon)             │
                                                              ▼
monocle daemon (monocle-runtime, long-lived background process)
├── axum HTTP :<os-port>   (hook POST receiver; OS-assigned; port in lock file)
├── UDS socket             (TUI client connection: hook events + PTY bytes + control)
├── SessionManager         (coordinator: spawns/tracks session-host processes; holds
│   │                       session metadata + per-session UDS connections to hosts;
│   │                       re-discovers and re-attaches on daemon startup)
│   ├── → session-host A: monocle-session-host process  ──► session-<uuid-A>.sock
│   │     (owns: pty_master, vt100::Parser, child_handle, session-state.json)
│   ├── → session-host B: monocle-session-host process  ──► session-<uuid-B>.sock
│   │     (owns: pty_master, vt100::Parser, child_handle, session-state.json)
│   └── → session-host N: ...
├── Arc<Broker<Event>>     (fan-out: hook events + PTY bytes to all connected TUI clients)
└── EngineModule registry  (ClaudeCodeModule: spawn_recipe() + hook handling)

Session-host processes (each is a native detached monocle-session-host binary)
├── session-host A: monocle-session-host --session-id <uuid-A> ...  (setsid'd; outlives daemon)
│   ├── Owns PTY master A  ──► vt100::Parser A  ──► per-session UDS A
│   ├── Owns child handle: claude --settings /tmp/monocle-hooks-A.json
│   │   ├── SessionStart hook      ──► POST http://localhost:<port>/hooks/session-start
│   │   ├── UserPromptSubmit hook  ──► POST http://localhost:<port>/hooks/prompt-submit
│   │   ├── PreToolUse hook        ──► POST http://localhost:<port>/hooks/pre-tool-use
│   │   ├── Notification hook      ──► POST http://localhost:<port>/hooks/notification
│   │   └── Stop hook              ──► POST http://localhost:<port>/hooks/stop
│   ├── PTY stdout/stderr ──► blocking reader thread ──► HostToDaemon::PtyBytes ──► daemon
│   └── PTY stdin  ◄── DaemonToHost::KeyInput ◄── daemon ◄── UDS KeyInput msg ◄── TUI
└── session-host B: similar structure

TUI client
├── Receives PtyOutput { session_id, bytes }  ──► vt100::Parser.process(bytes)
├── Renders PseudoTerminal widget (tui-term 0.3.4) from parser.screen()
├── Sends KeyInput { session_id, bytes }  ──► encoded crossterm KeyEvent → PTY bytes
└── Sends ResizePane { session_id, rows, cols }  ──► daemon → session-host resize
```

**The architectural inversion:** in v1.1.2, the daemon received hooks from sessions the user launched elsewhere. In v2.0/v2.1, the daemon coordinates sessions that are owned by per-session `monocle-session-host` processes (ADR-0009). Each session-host is setsid'd and outlives a daemon restart. The TUI is a client that streams PTY output and forwards keystrokes. Hook events continue to arrive via HTTP and are fan-out as before.

**Persistence model (session-host-owns-PTY; daemon coordinates/re-attaches) — three cases:**
_(Previously called DAEMON-OWNS-PTY. Renamed at v2.1 / D-238 escalation. The daemon still coordinates all session lifecycle and streams PTY bytes to TUI clients; the change is that PTY ownership now resides in per-session session-host processes that outlive a daemon restart, rather than inside the daemon process itself.)_

1. **TUI exit and reconnect (supported in v1A):** Sessions survive a monocle TUI process exit. The per-session session-host processes continue owning PTY masters and child handles; the daemon remains running; a reconnecting TUI client re-streams from the existing per-session `vt100::Parser` state. Unchanged from v2.0.

2. **Graceful daemon-process restart (REQUIRED to survive in v1A — D-238 escalation):** When the daemon process restarts gracefully, sessions MUST survive. This requires PTY masters and harness child processes to be owned by a component that outlives the daemon process — a native detached per-session "session-host" process (abduco/dtach-style ownership). The daemon coordinates session-hosts and re-attaches to them on restart. A planned `DaemonRestart` therefore performs a clean reconnect/handoff, not a session teardown. Persistence principle: the session-host owns the PTY master + child process; the daemon is a coordinator that can detach and re-attach. **Default: native implementation (no external multiplexer dependency, no-tmux constraint preserved).** If native daemon-restart-survival proves infeasible at acceptable cost or complexity for v1A, the architect MUST surface the external-supervisor (tmux/abduco/dtach) tradeoff for human decision rather than adopting it silently. Architecture route: Q-8 in §Open Questions.

3. **Daemon crash (accepted v1A boundary — unchanged):** Hard crash → in-flight sessions lost → user re-launches. This is an unplanned death; no clean handoff completes. The daemon is stable; crash is exceptional; launchd/systemd or a monocle-internal daemon watchdog provides the operational mitigation. Cross-crash PTY state serialization remains explicitly out of v1A scope.

`session-state.json` persists enough metadata to re-display terminated sessions and offer re-launch with the same parameters (project, harness, profile).

The canonical Phase 1 hook set is 5 endpoints (unchanged from v1.1.2, locked by JC-2). Hook auto-injection: when monocle launches a session, it passes `--settings <hooks_settings_path>` to the `claude` binary — no manual `settings.json` copy required. The `lock.app = 'monocle'` filter in the hook JS ensures only monocle-launched sessions trigger the monocle hook endpoint.

---

## Workspace Layout

Legend: **[EXISTS]** = crate present in the current 9-crate workspace (as of D-232/Wave-7 gate). **[v1A]** / **[v1B]** = introduced by that wave. **[Phase-3]** = deferred/suspended.

```
monocle/
├── Cargo.toml                    # workspace manifest
├── xtask/                        # [EXISTS] cargo xtask build-time helpers (currently omitted from crates/ layout; lives at workspace root)
└── crates/
    ├── monocle-core/             # [EXISTS] pure types: Event, Action, EngineMetadata, AppMode, SessionState; no I/O
    ├── monocle-runtime/          # [EXISTS] async daemon: axum server, broker, EngineModule registry;
    │                             #   SessionManager sub-module [v1A NEW]: session-host coordinator,
    │                             #   spawn/kill/attach/detach; proxies PTY bytes from session-hosts to TUI
    ├── monocle-session-host/     # [v1A NEW] per-session native detached binary; owns the
    │                             #   (pty_master, vt100::Parser, child_handle) triple; exposes
    │                             #   per-session UDS socket; setsid'd to outlive daemon restarts
    ├── monocle-tui/              # [EXISTS] ratatui renderer: panels, overlays, keybinding dispatch;
    │                             #   EmbeddedTerminal mode [v1A NEW]: tui-term PTY widget
    ├── monocle-ipc/              # [EXISTS] Unix domain socket + shared-memory ring buffer;
    │                             #   extended with PtyOutput/KeyInput/ResizePane message types [v1A]
    ├── monocle-config/           # [EXISTS] ~/.monocle/config.json: harness profiles, binding overrides, CCR path
    ├── monocle-proto/            # [EXISTS] prost-generated protobuf types; extended with PTY message types [v1A]
    ├── monocle-test-harness/     # [EXISTS] integration test scaffolding: fake Claude Code subprocess, fake hooks;
    │                             #   MockSessionHostSpawner [v1A NEW]: SessionHostSpawner trait test double
    ├── monocle/                  # [EXISTS] binary crate: clap CLI, daemon entrypoint, TUI entrypoint
    ├── monocle-static/           # [v1B] customization reader+writer: CLAUDE.md, settings.json,
    │                             #   hooks, keybindings — interactive CRUD activated in v1B
    ├── monocle-workflow/         # [v1B] factory-awareness: STATE.md parser, FactoryAdapter trait, VsddFactoryAdapter
    │                             #   (struct/logic exists in monocle-core today; extracted to own crate in v1B)
    ├── monocle-fuzz/             # [v1A or v1B] cargo-fuzz targets for parser and hook endpoint fuzzing
    └── monocle-plugin-sdk/       # [Phase-3 SUSPENDED] WASM plugin ABI: EngineModule + FactoryAdapter for third-party plugins
```

Note: SessionManager lives as a sub-module of `monocle-runtime` (not a separate crate) — confirmed by SS-08 (architecture delta). Rationale: session coordination is intrinsically a daemon responsibility; SessionManager shares daemon-internal types (DaemonState, Arc<Broker<Event>>); no other crate depends on SessionManager directly (the proto/IPC wire is the interface); the SessionHostSpawner trait provides the test seam regardless of crate structure. FactoryAdapter trait and VsddFactoryAdapter currently live in `monocle-core` (`crates/monocle-core/src/factory/`; extracted from monocle-core to monocle-workflow in v1B).

---

## Key Abstractions

### EngineModule (extended in v2.0)

The multi-harness gene from codemachine-cli. Extended with `spawn_recipe()` for the control-center inversion:

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

    /// NEW (v2.0): Return the recipe needed to spawn a session under monocle.
    /// Default impl returns Err(UnsupportedOperation) for engines that do not
    /// support monocle-spawned sessions. ClaudeCodeModule implements this.
    fn spawn_recipe(&self, opts: &SpawnOptions) -> Result<SpawnRecipe, EngineError> {
        Err(EngineError::UnsupportedOperation("spawn_recipe"))
    }
}

/// NEW (v2.0): The spawn recipe produced by an EngineModule.
/// SessionManager uses this to build the portable-pty CommandBuilder.
pub struct SpawnRecipe {
    pub binary: PathBuf,          // absolute path to the harness binary
    pub args: Vec<String>,        // includes --settings <hooks_settings_path>
    pub env: HashMap<String, String>, // injected hook config, CCR env vars
    pub cwd: PathBuf,             // git worktree path (per claude-squad A.1 pattern)
}

pub struct EngineMetadata {
    pub display_name: &'static str,
    pub icon: char,           // single char shown in sessions panel
    pub config_path: PathBuf, // e.g. ~/.claude/ or ~/.codemachine/
    pub hook_schema: &'static str, // JSON schema for hook payloads
}
```

Built-in: `ClaudeCodeModule` (implements `spawn_recipe()`). Second built-in: `CodeMachineModule` (returns `UnsupportedOperation` in v1; implemented in a later wave). Third-party: WASM plugin implementing the same ABI via `monocle-plugin-sdk` (Phase 3, suspended).

### SessionManager (new in v2.0 — redesigned for v2.1 session-host-owns-PTY model)

Daemon-side sub-module of `monocle-runtime` (location: `crates/monocle-runtime/src/session_manager/mod.rs`; not a separate crate). **Coordinator role**: the daemon's SessionManager holds session metadata and per-session UDS connections to the session-hosts; it re-discovers and re-attaches on daemon startup. It does NOT own the `(pty_master, vt100::Parser, child_handle)` triple — those are owned by each per-session `monocle-session-host` process (ADR-0009).

_(Pre-v2.1 note: earlier drafts said "Daemon-side sub-module of monocle-runtime. Owns the (pty_master, vt100::Parser, child_handle) triple per session." That is the old v2.0 in-process model, superseded by ADR-0009 and the v2.1 session-host-owns-PTY escalation.)_

```rust
/// Session lifecycle state (adapted from claude-squad A.3 instance lifecycle).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Created,      // spawn_session() called; monocle-session-host not yet started
    Launching,    // session-host process spawned; waiting for its socket to appear
    Running,      // session-host alive; PTY streaming
    Detached,     // TUI disconnected; the session-host process still owns the PTY
                  // and child; the daemon may or may not be attached
    Terminated,   // child process exited; GC in 10s grace period
    Killed,       // user-initiated kill; SIGTERM sent to session-host
}
```

The `SessionHostSpawner` trait provides the test seam for session-host process spawning:

```rust
/// Seam for session-host process spawning.
/// Mirrors the PtyFactory concept from claude-squad A.5 pattern.
pub trait SessionHostSpawner: Send + Sync + 'static {
    /// Spawn a monocle-session-host process with the given session ID and recipe.
    /// Returns the child PID and expected socket path.
    async fn spawn(
        &self,
        session_id: &str,
        recipe: &SpawnRecipe,
        runtime_dir: &Path,
    ) -> Result<SpawnedHostHandle, SessionError>;
}

// RealSessionHostSpawner: spawns the monocle-session-host binary via
//   std::process::Command with pre_exec setsid() — making the process a
//   group leader immune to SIGHUP when the daemon exits.
// MockSessionHostSpawner: in-memory mock host — does NOT open a real PTY;
//   used in integration tests.
```

### Action enum (extended in v2.0)

```rust
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // Navigation (unchanged)
    FocusPanel(PanelId),
    ScrollUp, ScrollDown, PageUp, PageDown,
    FilterToggle, FilterType(char),

    // Session management (v1.1.2 stubs + new v2.0 actions)
    SessionSelect,
    SessionKill,
    SessionAttach,
    SessionCreate,            // NEW v2.0: open launch wizard
    SessionDetach,            // NEW v2.0: detach TUI from running session
    SessionRename,            // NEW v2.0: rename a session

    // Embedded terminal (NEW v2.0)
    EnterEmbeddedTerminal,    // switch Preview pane to PTY widget for focused session
    ExitEmbeddedTerminal,     // return to normal dashboard view
    ForwardKeyToPty(KeyEvent),// route crossterm KeyEvent to PTY stdin
    PtyScrollUp, PtyScrollDown,

    // Permission prompt (unchanged)
    PermissionAcceptOnce,
    PermissionAcceptAlways,
    PermissionReject,
    PermissionTraceToSource,

    // Overlay control (unchanged)
    OverlayOpen(OverlayKind),
    OverlayClose,
    OverlayCycleNext,

    // Tune actions (v1B wave)
    TuneEditBinding,          // edit a keybinding in-place
    TuneApplyProfile,         // switch active profile
    TuneResetBinding,         // reset binding to builtin
    TuneEditCcrSlot,          // edit a CCR routing slot

    // System (unchanged + additions)
    DaemonRestart,
    ConfigReload,
    Quit,
}
```

### AppMode state machine (extended in v2.0)

```rust
#[derive(Clone, PartialEq, Eq)]
pub enum AppMode {
    /// Normal dashboard view.
    Dashboard { focused: FocusSnapshot },

    /// Telescope-style filter input over the focused panel.
    Filtering { panel: PanelId, query: String, prior: FocusSnapshot },

    /// Modal overlay (permission prompt, detail view, help).
    /// VecDeque<PromptModal> enables concurrent prompt queuing.
    Overlay { stack: VecDeque<PromptModal>, prior: FocusSnapshot },

    /// Full-screen view of a single panel.
    Fullscreen { panel: PanelId, prior: FocusSnapshot },

    /// NEW v2.0: Preview pane hosts the tui-term PTY widget for the focused session.
    /// Keyboard events are forwarded to the daemon SessionManager as KeyInput IPC messages.
    /// session_id is a String (UUID rendered as String at AppMode/IPC boundary — avoids
    /// a uuid dep in monocle-core and eliminates UUID/String conversion friction; per SS-08
    /// session_id canonical ruling).
    EmbeddedTerminal { session_id: String, prior: FocusSnapshot },

    /// NEW v2.0: Launch wizard — multi-step modal for creating a new session.
    /// Steps: profile picker → project picker → worktree confirmation → launch.
    SessionCreation { step: SessionCreationStep, prior: FocusSnapshot },
}

#[derive(Clone, PartialEq, Eq)]
pub enum FocusSnapshot {
    Sessions,
    Preview,
    Workflow,
    Customizations,
    EventRibbon,
}
```

### FactoryAdapter (unchanged)

The workflow-plane plugin contract remains observe-only. Detection canonical signal: `document_type: pipeline-state` in `.factory/STATE.md`. The adapter reads; monocle never writes STATE.md.

---

## TUI Layout (v2.0 — control-center model)

```
┌─────────────────────────────────────────────────────────────────────┐
│  monocle  [project: monocle]  3 sessions  2 workflows  Ctrl-\ hide  │
│  [+] New session    [k] Kill    [d] Detach    [Enter] View terminal  │
├────────────────────────┬──────────────────┬────────────────────────┤
│  [1] Sessions          │  [2] Preview     │  [3] Workflow          │
│                        │                  │                        │
│  monocle/              │  session detail  │  vsdd-factory          │
│  ● CC  monocle  phase0 │  token: 142k     │  phase: PIVOT          │
│  ● CC  blog     wave-2 │  cost: $0.83     │  status: active        │
│                        │  hooks: 47       │  awaiting: vision gate │
│  api-svc/              │  uptime: 00:42   │  blocking: 0           │
│  ● CM  api-svc  idle   │                  │  cycle: cycle-001      │
│                        │  [Enter] Enter   │                        │
│  / filter  + new  ? help  terminal view   │                        │
│                        │                  │                        │
├────────────────────────┴──────────────────┴────────────────────────┤
│  [4] Customizations / Tune                                          │
│  CLAUDE.md: /Users/jmagady/Dev/monocle/CLAUDE.md  (active)         │
│  settings:  allowedTools [Bash,Read,Edit,Write,...] + 3 hooks       │
│  keybinds:  12 custom  /  48 builtin     [e] Edit  [r] Reset       │
├─────────────────────────────────────────────────────────────────────┤
│  [5] Events                                                          │
│  20:29:01  PreToolUse    Bash  monocle-session-1                    │
│  20:29:00  Notification  info  monocle-session-1  12ms              │
│  20:28:58  PreToolUse    Edit  blog-session-2  PENDING              │
├─────────────────────────────────────────────────────────────────────┤
│  Tab: cycle panels  Enter: view terminal  +: new session  q: quit  │
│  breadcrumb: Dashboard > Sessions                                   │
└─────────────────────────────────────────────────────────────────────┘
```

Embedded terminal mode (AppMode::EmbeddedTerminal):

```
┌─────────────────────────────────────────────────────────────────────┐
│  monocle  blog-session-2  [EMBEDDED TERMINAL]  Esc: exit terminal   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  $ claude --settings /tmp/monocle-hooks-abc.json                    │
│  Claude Code 1.x.x                                                  │
│  ✓ Reading CLAUDE.md (3 files)                                      │
│  ✓ Hooks configured (monocle: http://localhost:54321)               │
│  > Working on wave-2 story S-012...                                 │
│                                                                     │
│  [all keystrokes forwarded to PTY stdin]                            │
│  [↑↓]: scroll history  Ctrl-D: end session  Esc: exit terminal mode │
│                                                                     │
│  ▓ (cursor)                                                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

Session launch wizard (AppMode::SessionCreation):

```
┌─ New Session  [1/3: Profile] ───────────────────────────────────────┐
│  Select a harness profile:                                          │
│                                                                     │
│  > Claude Code (default)              claude 1.x.x                 │
│    Claude Code + CCR (background)     background model routing      │
│    Claude Code + CCR (think)          extended thinking enabled     │
│                                                                     │
│  [↑↓] navigate  [Enter] select  [Esc] cancel                       │
└─────────────────────────────────────────────────────────────────────┘
```

Permission prompt popup overlay (unchanged from v1.1.2 — the "killer scenario" is preserved):

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

---

## v1 Capability Set (D-237 — human-ratified)

All four capabilities ship in v1. The two-wave split is feature-ordering per CLAUDE.md Rule 2 — each wave ships production-grade on its cycle.

### v1A: Launch Wave

**LAUNCH — the core inversion**

monocle spawns and owns Claude Code (and future harness) sessions from the TUI. The daemon SessionManager spawns a native `monocle-session-host` process per session (via `SessionHostSpawner`), which opens a `portable-pty` PTY pair, builds a `CommandBuilder` from `EngineModule::spawn_recipe()` (binary + args including `--settings <hooks_settings_path>` for hook auto-injection + cwd as git worktree + CCR env vars), and owns the child process. The daemon coordinator tracks session-host processes and proxies PTY bytes to TUI clients. Session lifecycle: `Created → Launching → Running → Detached → Terminated`.

This is the architectural reversal from v1.1.2: monocle no longer waits for hook POSTs from sessions the user launched elsewhere. It is now the launcher.

**EMBEDDED PTY — never leave the TUI**

The running session is visible and interactive inside monocle. The TUI Preview pane slot hosts a `tui-term::PseudoTerminal` widget that renders the `vt100::Screen` from the TUI-side `vt100::Parser` (fed by `PtyOutput` IPC bytes proxied from the session-host). The TUI sends keystrokes to the daemon as `KeyInput` IPC messages; the daemon forwards them to the session-host which writes PTY-encoded bytes to the master. The user reads and interacts with the Claude session inside monocle without switching windows.

PTY byte stack: `portable-pty 0.9.0` (spawn + master read/write, in `monocle-session-host`) + `vt100 0.16.2` (ANSI parse → screen state, in `monocle-session-host` and `monocle-tui`) + `tui-term 0.3.4` (render screen as ratatui widget, in `monocle-tui`). All MIT, no RUSTSEC, MSRV 1.88 compatible, ratatui 0.30 compatible (verified at manifest level — Cargo-init spike required at architecture delta step).

Input fidelity (v1A): printable keys + control keys (Ctrl-C, Ctrl-D, Ctrl-Z) + arrows + Backspace + Tab + Esc + Enter + mouse events + Kitty keyboard protocol (full support IN v1A scope — human-ratified D-237 2026-06-03). Bracketed paste is included as part of full fidelity; implementation details (crossterm Kitty-enhancement flags + mouse capture → PTY byte translation wiring) are architect-routed items resolved at architecture delta. Full fidelity is the v1A target; no input class is deferred. This full-fidelity scope supersedes the narrower keyboard scope described in DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md and embedded-pty-evaluation.md, which predate the D-237 ratification and will be reconciled during the architecture delta.

**Permission prompts while in EmbeddedTerminal or SessionCreation mode:** While the user is in `AppMode::EmbeddedTerminal` or `AppMode::SessionCreation`, incoming permission prompts are NEVER silently queued without notification. The production-grade behavior (architect-confirmed): an incoming `PreToolUse` hook MUST immediately raise a status-bar badge (e.g., `[2 prompts]` indicator) AND an audible bell (`\x07`). The user can press Esc to exit embedded terminal mode, at which point the overlay presents on the prior AppMode. This guarantee will be formalized as a dedicated behavioral contract (to be authored by product-owner in the PRD delta). A v1B enhancement — pre-emption of embedded terminal mode by the overlay itself (i.e., the overlay slides in without requiring the user to Esc first) — is a potentially desirable UX upgrade but requires human ratification before it is added to scope; it is recorded here as an open v1B item (not v1A).

**MULTI-SESSION / MULTI-PROJECT**

List, switch, create, kill, rename, and group sessions by project from the TUI. Sessions grouped by `project_name` with collapsible header rows. Fast switching: O(1) — swap which parser the TUI widget renders; all sessions parse in the background. Project picker overlay (SessionCreation step 2). GC policy: Terminated sessions cleaned from registry after 10-second grace period.

Persistence: session-host-owns-PTY; daemon coordinates/re-attaches (three cases — see §Process Topology for full detail): (1) TUI exit/restart → session-host processes still running, daemon still running → TUI reconnects and re-streams (supported in v1A); (2) graceful daemon-process restart → sessions SURVIVE via native detached session-host ownership → daemon re-attaches on restart (required in v1A — D-238 escalation; native default, no-tmux preserved; external supervisor only as architect-surfaced fallback for human decision); (3) daemon crash → sessions lost → re-launch (unplanned death, no clean handoff, accepted v1A boundary). `session-state.json` per session for re-display and parameter-based re-launch.

**Hook auto-injection on spawn**

When monocle launches a session, it writes the hooks settings file to a per-session temp path and passes `--settings <path>` in the `CommandBuilder` args. No manual `~/.claude/settings.json` copy required. Launch ownership carries hook-wiring ownership.

### v1B: Interactive Tune Wave

**INTERACTIVE TUNE**

The Static plane becomes interactive. The user edits keybindings, profile definitions, and CCR routing slots directly in the TUI and the changes take effect (atomic `tempfile::persist` writes, then hot-reload). Profile management (create/edit/delete profiles in `~/.monocle/config.json`). CCR routing slot editor (select model per routing scenario, write CCR config, inform user of any restart requirements). monocle-static CRUD activation (enable/disable customizations, move between scopes). Every destructive Tune action follows the NikiforovAll Modal-Confirm-Callback 3-phase pattern.

### Already-built, preserved + extended (from Phase 1 substrate)

| Capability | Status | BCs |
|-----------|--------|-----|
| Hook ingestion (5 endpoints, PID-liveness, DTU clone) | Preserved | BC-2.05.*, BC-2.06.* |
| Permission overlay (VecDeque stack, killer scenario ≤6 keystrokes) | Preserved | BC-2.06.022 |
| Sessions panel (session list, token burn rate, phase tag) | Extended (project grouping + launch/kill/detach actions) | BC-2.05.002 |
| Event ribbon (rolling hook event log with filter) | Preserved | BC-2.06.006/018 |
| Profile picker (Ctrl-P, sticky-per-project, CCR path) | Preserved + extended (used in launch wizard) | BC-2.07.004/005 |
| Workflow plane (FactoryAdapter, VsddFactoryAdapter, STATE.md parsing) | Preserved + new trigger path | existing |

---

## End-to-End Killer Scenarios

### Scenario 1: Launch and enter a new session (new in v2.0)

Developer opens monocle (`Ctrl-\`), sees sessions panel. Presses `+` to create a session. SessionCreation wizard opens: selects "Claude Code + CCR (background)" profile, selects project `blog`, confirms worktree path. monocle daemon spawns: `claude --settings /tmp/monocle-hooks-abc.json` in the blog worktree, with CCR env vars. Session appears as "Launching" then "Running" in the sessions panel. Developer presses Enter to enter the embedded terminal: the Preview pane shows the live Claude session. Developer types a prompt; keystrokes forward to the PTY. Claude responds. Developer presses Esc to return to Dashboard. Total: started a session, sent a prompt, and returned to the dashboard overview — without opening a new terminal window or manually configuring any hook.

### Scenario 2: Multi-session permission resolution (preserved from v1.1.2)

Setup: three sessions running. Two permission prompts arrive concurrently from different sessions.

1. `blog-session-2` fires a `PreToolUse` hook — Edit on `draft.md`. Daemon queues `PromptModal`.
2. `api-svc-session-3` fires a `PreToolUse` hook — Bash `cargo build --release`. Second `PromptModal` pushed. TUI badge shows `2 prompts`.
3. Developer presses `Ctrl-\` from nvim. Overlay opens: both prompts visible.
4. Developer presses `2` (Accept always). `blog-session-2` unblocks. Stack pops.
5. Developer presses `1` (Accept once). `api-svc-session-3` unblocks. Stack empty; Dashboard restored.
6. Developer presses `Ctrl-\` to dismiss. Returns to nvim.
7. Total: 4 keystrokes. 0 context switches between tmux windows. 0 sessions stalled.

---

## Explicit Non-Goals

- Does NOT execute vsdd-factory workflows — monocle observes factory state (reads STATE.md); it never writes STATE.md, never triggers factory phases, never dispatches factory agents
- Does NOT write STATE.md — the FactoryAdapter reads STATE.md; monocle never mutates it
- Does NOT route LLM API requests — CCR integration is detect-on-PATH + config-write + env-inject; monocle does not proxy or modify LLM traffic
- Does NOT replace the user's general-purpose terminal multiplexer — monocle runs inside the user's tmux session; it manages AI coding sessions via its own session-host-owned PTYs (daemon-coordinated); it does not attempt to multiplex the user's non-AI terminal work
- Does NOT include PM/Worker multi-agent orchestration — human is always the coordinator; sessions are independent (no inter-session bus, no automated handoff)
- Does NOT own session transcripts — monocle reads hook events (fine-grained, ephemeral); full transcript storage belongs to Claude Code's own persistence layer
- Does NOT build its own LLM provider abstraction — CCR is the external router; monocle integrates by detecting it, not by reimplementing it
- Does NOT use tmux as the PRIMARY session multiplexer — native detached per-session `monocle-session-host` PTY ownership (portable-pty inside each session-host process) is the chosen approach; tmux control-mode is a documented fallback if native session-host persistence proves insufficient (not a v1 default)

---

## Tech Stack (v2.0 additions)

**Canonical version pin manifest:** see `.factory/specs/architecture/SS-deps-pin-manifest.md`, which carries live-crates.io-verified pins and the RUSTSEC audit context. The pin manifest supersedes version examples in this document.

**New crates added in v2.0 (from embedded-pty-evaluation.md §7.1):**

| Crate | Version | Location | Role | License | RUSTSEC |
|-------|---------|----------|------|---------|---------|
| `portable-pty` | `"0.9"` | `monocle-session-host` | PTY pair creation, child spawn, master read/write | MIT | none (2026-06-03) |
| `vt100` | `"0.16"` | `monocle-session-host`, `monocle-tui` | ANSI/VT100 parse → in-memory screen state | MIT | none (2026-06-03) |
| `tui-term` | `"=0.3.4"` | `monocle-tui` | ratatui widget rendering `vt100::Screen` | MIT | none (2026-06-03) |

Compatibility notes:
- `tui-term 0.3.4` depends on `ratatui-core ^0.1.0` and `ratatui-widgets ^0.3.0` — exactly what `ratatui 0.30.0` pins. Unifies to a single copy in the dependency graph (verified at manifest level on deps.rs 2026-06-03). Cargo-init spike required at architecture delta step: `cargo tree -d` must show zero duplicate `ratatui-core`/`ratatui-widgets`/`vt100` versions before committing.
- MSRV impact: none — `tui-term` MSRV 1.86 ≤ monocle's 1.88 floor.
- `tui-term` self-describes as "work in progress." **Decision-of-record (this section):** Pin it exactly; gate upgrades through review. Vendoring is cheap if needed (small surface) — apply if the WIP label becomes a stability concern. Do NOT enable the `unstable` feature flag (the tui-term spawn helper); monocle spawns via `portable-pty` directly in the runtime. See Q-7 in §Open Questions for the architect confirmation step.

All architectural intent from v1.1.2 for existing crates is preserved (ratatui for TUI, crossterm as backend, tokio for async runtime, axum for hook ingestion, interprocess for IPC, prost for cross-host wire format, serde_yaml_ng for config, nucleo for fuzzy matching, similar for diff preview, directories for XDG paths, notify for FS watching, thiserror+anyhow for error handling, tempfile for atomic writes, clap for CLI, tracing for instrumentation, etc.). See `SS-deps-pin-manifest.md` for live verified pins.

---

## Wave Plan (re-baselined v1)

This replaces the Phase Plan from v1.1.2. The two-wave split is feature-ordering (CLAUDE.md Rule 2) — both waves ship in v1; each feature is production-grade on the wave it ships.

| Wave | Name | Core capabilities | Key new work | Preserved/extended |
|------|------|-------------------|--------------|-------------------|
| v1A | Launch Wave | LAUNCH + EMBEDDED PTY + MULTI-SESSION/MULTI-PROJECT | SessionManager + monocle-session-host binary (portable-pty + vt100 live in the session-host); EmbeddedTerminal AppMode; SessionCreation wizard; session-state.json; EngineModule.spawn_recipe(); monocle-proto PTY message types; hook auto-injection; SessionHostSpawner trait + MockSessionHostSpawner | All Phase-1 observe/control capabilities; profile picker (used in launch wizard); event ribbon; workflow plane |
| v1B | Tune Wave | INTERACTIVE TUNE | monocle-static CRUD activation; keybinding editor; CCR routing slot editor; profile management UI; TuneEdit* Action variants | Everything from v1A |

**Human ratified (D-237, 2026-06-03): v1A/v1B split confirmed.** All four capabilities ship in v1; v1A = Launch + Embedded PTY + Multi-session/project (+ persistence + hook auto-inject); v1B = Interactive Tune. Both waves production-grade on their cycles.

---

## Gene-Source Disposition Summary (v2.0)

Full individual dispositions: `.factory/semport/DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md`

| Gene Source | v1.1.2 stance | v2.0 stance | Change |
|------------|---------------|-------------|--------|
| claude-squad: session launch | Out of scope | MODEL (portable-pty, not tmux) | Fully reversed |
| claude-squad: lifecycle state machine | Out of scope | MODEL + ADAPT | Fully reversed |
| claude-squad: worktree-per-session | ADOPT | ADOPT (extended) | Confirmed |
| claude-squad: capture-pane / tmux-primary | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| claude-squad: AutoYes / polling daemon | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| zellij: PTY/pane architecture | Out of scope | MODEL (architecture pattern) | Reversed |
| zellij: client/server IPC | ADOPT | ADOPT (extended: PTY byte types) | Confirmed |
| zellij: zellij-as-library | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| any-context: hook inject-on-spawn | N/A (couldn't inject) | ADOPT (`--settings` flag) | New |
| nikiforovall: AppMode state machine | ADOPT | ADOPT (extended: EmbeddedTerminal + SessionCreation) | Extended |
| nikiforovall: writer/CRUD genes | Not yet active | ADOPT (v1B) | Extended |
| lazygit: 5-level precedence + Action enum | ADOPT | ADOPT (extended: PTY + launch actions) | Extended |
| CCR: detect-on-PATH + env inject | ADOPT | ADOPT (extended: used in spawn path) | Extended |
| codemachine: EngineModule trait | ADOPT | ADOPT (extended: spawn_recipe()) | Extended |
| vsdd-factory: FactoryAdapter observe-only | ADOPT | ADOPT (confirmed observe-only) | Confirmed |
| PM/Worker orchestration (any-context) | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| SSH federation | ADOPT (Phase 4) | SUSPENDED (re-baselined v1 scope) | Suspended |
| WASM plugin SDK | Phase 3 | SUSPENDED | Suspended |

---

## Open Questions for the Architecture Delta

These are genuine cross-component architecture decisions that require architect adjudication. They are NOT deferrals of mechanical questions (per CLAUDE.md Rule 6 anti-pattern). They are captured here for the orchestrator to route to the architect after this vision is approved.

Human-ratified questions are marked RESOLVED; they are retained for traceability but require no human action.

---

**RESOLVED — Q-8: PTY-ownership-survival mechanism for graceful daemon-process restart (CASE 2) — ADR-0009**

Design a native detached per-session session-host that owns the PTY master and harness child process and survives a daemon-process restart, with the daemon coordinating and re-attaching over UDS.

**Q-8 VERDICT (ADR-0009, 2026-06-03): Native feasible at acceptable cost for v1A.** Option (b) adopted: dedicated `monocle-session-host` binary per session, spawned by the daemon via `std::process::Command` with a pre-exec `setsid()` call. Session-host process owns `(pty_master, vt100::Parser, child_handle)`; exposes per-session UDS at `<runtime_dir>/session-<uuid>.sock`; writes `session-state.json` sidecar. Daemon re-attaches on startup via `SessionManager::rediscover_sessions()` (reads sidecars, probes PID liveness, connects to alive hosts). Option (a) (double-fork inside the daemon) rejected: fork() in async Tokio runtime is unsafe. Option (c) (external supervisor) rejected: no-tmux constraint. SessionManager abstraction boundary reworked from PTY-owning (old v2.0 model) to session-host coordinator (v2.1 model): `SessionHostSpawner` trait replaces `PtySpawner`; `RealSessionHostSpawner` spawns the session-host binary; `MockSessionHostSpawner` is the in-memory test double. See ADR-0009 and SS-session-manager.md (SS-08) for full design. SS-deps-pin-manifest-v2-delta.md for crate placement (`portable-pty` and `vt100` in `monocle-session-host`; `tui-term` in `monocle-tui`). Retained here for traceability; no further architect action required.

**Default design constraint (no-tmux preserved):** The native detached session-host approach is the chosen primary. tmux/abduco remain documented fallbacks only (Q-8 architecture decision closed — native feasible).

---

**RESOLVED — Q-3: Daemon crash persistence posture for v1A (human-ratified 2026-06-03, D-237)**
Daemon crash → sessions lost → user re-launches. This is the accepted v1A boundary. The daemon is stable; crash is exceptional; launchd/systemd or a monocle-internal daemon watchdog provides operational mitigation. Cross-crash PTY state serialization remains out of v1A scope. TUI reconnect to a running daemon (session-host-owns-PTY; daemon coordinates/re-attaches — formerly called DAEMON-OWNS-PTY; renamed at v2.1) is the survival path for TUI exit/restart (CASE 1); this is already the design in §Process Topology. Note: CASE 2 (graceful daemon-process restart) was escalated at D-238 and is now REQUIRED to survive — see Q-8. Architect follow-on (implementation only): session-state.json schema for re-display and parameter-based re-launch of terminated sessions.

**RESOLVED — Q-5: v1A / v1B wave boundary (human-ratified 2026-06-03, D-237)**
v1A = Launch + Embedded PTY + Multi-session/Multi-project (+ persistence + hook auto-injection); v1B = Interactive Tune. All four capabilities ship in v1. The two-wave split is confirmed feature-ordering (CLAUDE.md Rule 2), not an MVP shortcut. Interactive Tune does NOT merge into v1A.

**RESOLVED — Q-6: Keyboard fidelity scope for v1A (human-ratified 2026-06-03, D-237)**
Full fidelity is IN v1A scope: printable keys + control keys (Ctrl-C, Ctrl-D, Ctrl-Z) + arrows + Backspace + Tab + Esc + Enter + mouse events + Kitty keyboard protocol. Bracketed paste is included as part of full fidelity. No deferral. Architect implementation sub-question (routed to architect, NOT human): how to wire crossterm Kitty-enhancement flags and mouse capture → PTY byte translation → portable-pty master write. This is an implementation decision in the architecture delta, not a scope question. Note: this full-fidelity scope supersedes the narrower keyboard scope still present in DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md and embedded-pty-evaluation.md, which predate D-237 ratification; the architect will reconcile those documents during the architecture delta.

---

**Q-1: PTY bytes over existing UDS IPC vs. dedicated streaming path (architect)**
Should `PtyOutput { session_id, bytes }` messages share the existing UDS IPC channel with hook events and command messages, or warrant a dedicated high-throughput streaming path per session?
- Option A (shared channel with per-session PTY byte buffer sized at 1024): simpler; risk of PTY byte bursts starving hook-event delivery — mitigated by separate per-session bounded buffers and drop-counter surfacing.
- Option B (dedicated per-session stream): cleaner throughput isolation; adds socket-per-session complexity or a multiplexed streaming protocol.
Recommendation from embedded-pty-evaluation.md §8 Q4: Option A.
- **v1A Gate Deliverable (benchmark):** Before v1A gate: benchmark PTY bytes over the chosen UDS + bounded-channel design at N concurrent sessions. Confirm or disprove that Option A holds at terminal-refresh rates (target: 1000+ events/sec per CLAUDE.md Conventions). The drop-counter convention surfaces starvation, but a benchmark is required before the gate. This is a pre-gate verification deliverable, not an open design question — the architect routes to performance-engineer once the Option A/B decision is made.

**Q-2: EngineModule.spawn_recipe() vs. SessionManager lifecycle on the EngineModule trait (architect)**
Should `launch`/`attach`/`detach`/`kill`/`resize`/PTY-stream become methods on the `EngineModule` trait, or should the trait provide only `spawn_recipe()` (binary/args/env) and the SessionManager component own all lifecycle? SS-engine-module.md already places operational concerns (spawning) on struct-level inherent operations, not the trait. Recommendation: lifecycle on SessionManager; EngineModule provides the recipe only.

**Q-7: tui-term fork posture (architect)**
tui-term 0.3.4 self-describes as "work in progress." Confirm the §Tech Stack default posture: exact-pin + vendoring-plan-if-needed is the production-grade default already decided; architect to confirm whether immediate vendoring before any v1A work is preferred over deferred-on-need. The small surface (one widget) makes vendoring cheap either way.

---

## Provenance

This vision synthesis was originally produced by the orchestrator agent after the full-protocol brownfield-ingest of 8 reference repos completed (factory-artifacts commit 2c2b676). The human approved v1.0 verbatim 2026-05-11 with the statement "I agree with this fully."

v1.1 was drafted by the business-analyst agent on 2026-05-12 to capture JC/EX/OQ-M closures and version-pin updates. Re-approved by the human on 2026-05-12.

v1.1.1 (2026-05-12): Surgical frontmatter and §Tech Stack pointer fixes.

v1.1.2 (2026-05-12): Surgical path fix — `/hooks/prompt-submit` wire correction.

**v2.0 (2026-06-03):** Major revision by the product-owner agent in response to the D-236 human-directed vision pivot and D-237 human ratification of the re-baselined v1 control-center scope. This revision:
- Retires the observe-only constraint and the specific rejections it codified
- Adds LAUNCH, EMBEDDED PTY, MULTI-SESSION/MULTI-PROJECT, and INTERACTIVE TUNE as first-class v1 capabilities
- Captures the DAEMON-OWNS-PTY persistence model (ratified D-237; renamed to session-host-owns-PTY at v2.1)
- Captures the embedded-PTY tech direction (portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4; from embedded-pty-evaluation.md v1.0)
- Documents the v1A/v1B wave ordering (feature-ordering, not MVP shortcut)
- Enumerates the preserved Phase-1 substrate as an asset set (not to be rebuilt)
- Captures open architecture questions for the architect in the architecture delta phase
- Sources: NEXT-SESSION-PIVOT.md, DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md v1.0, embedded-pty-evaluation.md v1.0, STATE.md decisions D-236 and D-237
- Status at v2.0: DRAFT — pending human approval gate.

**v2.1 (2026-06-03):** D-238 escalation applied by product-owner agent. Persistence model escalated: CASE 2 (graceful daemon-process restart) now REQUIRES session survival via native detached session-host processes (session-host-owns-PTY model). Persistence principle renamed from DAEMON-OWNS-PTY to session-host-owns-PTY; daemon coordinates/re-attaches. No-tmux default preserved; external supervisor only as architect-surfaced fallback for human decision. Q-8 (HIGH priority) added to §Open Questions for architect. Doc **APPROVED** by Joshua Magady (2026-06-03) to proceed to brief → architecture → story delta.

**v2.2.2 (2026-06-04):** I19-001 consistency correction — §Tech Stack table tui-term version form corrected from caret `"0.3"` to exact-pin `"=0.3.4"`. The exact-pin contract was already ratified in ADR-0011 §Q-7 and recorded in SS-deps-pin-manifest-v2-delta; the table row was a stale caret form contradicting the §Tech Stack compatibility note "Decision-of-record: Pin it exactly." No decision change — consistency correction only. Status: APPROVED (consistency-only; no scope change).

**v2.2.1 (2026-06-04):** I18-001 consistency correction — aligns §Non-Goals and §Five Planes with the already-approved §Process Topology (ADR-0009 / D-238 session-host-owns-PTY model). Three surviving stale assertions fixed: (1) §Five Planes Runtime row "daemon-owned" → "owned by a detached monocle-session-host process (session-host-owned; daemon coordinates)"; (2) §Explicit Non-Goals line 588 "daemon-owned PTYs" → "session-host-owned PTYs (daemon-coordinated)"; (3) §Explicit Non-Goals line 592 "portable-pty native in-process PTY is the chosen approach; tmux control-mode is a documented fallback if daemon-owned persistence proves insufficient" → "native detached per-session monocle-session-host PTY ownership (portable-pty inside each session-host process) is the chosen approach; tmux control-mode is a documented fallback if native session-host persistence proves insufficient". This is a consistency correction to already-ratified decisions; it does NOT change any approved decision. Status: APPROVED (consistency-only; no scope change).

**v2.2 (2026-06-03):** Consistency propagation of v2.1 session-host model + architect rulings; no scope change. Applied ADR-0009 and SS-08 (SS-session-manager.md): §Process Topology ASCII diagram and §SessionManager description rewritten from in-process PTY ownership to session-host coordinator model. `PtySpawner`/`RealPtySpawner`/`MockPtySpawner` replaced with `SessionHostSpawner`/`RealSessionHostSpawner`/`MockSessionHostSpawner` (SS-08 canonical). `portable-pty` and `vt100` crate locations corrected to `monocle-session-host` per SS-deps-pin-manifest-v2-delta. `VsddFactoryAdapter`/`FactoryAdapter` extraction source corrected to `monocle-core` per live codebase (IMP-1). `EmbeddedTerminal { session_id }` typed as `String` per SS-08 ruling (IMP-2). `Detached` state comment updated for session-host ownership (SUG-1). §Wave Plan now names `monocle-session-host` binary as the v1A crate deliverable (SUG-2). Permission badge+bell guarantee documented; v1B embedded-terminal pre-emption open item recorded (SUG-3). Status: APPROVED (no scope change; consistency-only propagation).

The vision is the synthesis lens for disposition decisions: every subsystem in every reference repo gets sorted through THIS vision. The D-236/D-237 reversal of the launch/manage genes is captured in DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md. If future vision-doc changes invalidate prior dispositions, those dispositions must be re-run.

v1.1.2 trace: the observe-only record is preserved in git history on the `factory-artifacts` branch. The git history is the canonical preservation mechanism — this file supersedes v1.1.2 in place.
