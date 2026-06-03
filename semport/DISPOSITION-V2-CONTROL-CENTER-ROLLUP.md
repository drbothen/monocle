---
document_type: gene-source-disposition-rollup
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
disposition_pass: v2 (D-236 control-center pivot; D-237 human ratification)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md (all sections), embedded-pty-evaluation.md
individual_dispositions:
  - claude-squad/claude-squad-disposition-v2-control-center.md
  - zellij/zellij-disposition-v2-control-center.md
  - any-context-lazyclaude/any-context-lazyclaude-disposition-v2-control-center.md
  - nikiforovall-lazyclaude/nikiforovall-lazyclaude-disposition-v2-control-center.md
  - lazygit/lazygit-disposition-v2-control-center.md
  - claude-code-router/claude-code-router-disposition-v2-control-center.md
  - codemachine-cli/codemachine-cli-disposition-v2-control-center.md
  - vsdd-factory/vsdd-factory-disposition-v2-control-center.md
---

# Gene-Source Disposition v2: Master Rollup (Control-Center Pivot)

## The Vision Delta

The D-236 pivot (human-ratified D-237) transforms monocle from:
> "Observe-only TUI over existing sessions you launched elsewhere"

to:
> "Full TUI control center — launch + manage + observe + tune + control; many sessions, many
> projects; never leave the TUI. Daemon owns PTYs. Session persistence across restart is hard v1."

The original vision explicitly rejected: "inherit PM/Worker orchestration", "execute workflows",
"replace terminal multiplexer". These rejections were correct and are CONFIRMED by this pass.

What the pivot reverses: the stance that LAUNCHING and OWNING sessions is out of scope.
monocle now spawns and supervises AI coding sessions from the daemon. The TUI is a client
that streams PTY output and forwards keystrokes over the existing UDS IPC.

## Reversal Scorecard

### REVERSED (launch/manage genes that were previously out-of-scope or rejected)

| Gene Source | Originally | Now | Reversal Type |
|------------|-----------|-----|---------------|
| claude-squad: session launch mechanism | Out of scope | MODEL (portable-pty, not tmux) | Fully reversed |
| claude-squad: instance lifecycle state machine | Out of scope | MODEL + ADAPT | Fully reversed |
| zellij: PTY/pane architecture | Out of scope (ingest scope) | MODEL (architecture only) | Fully reversed |
| zellij: multiplexer role | LEAVE BEHIND | PARTIALLY reversed — monocle IS a mux for AI sessions; does NOT replace user's terminal mux | Partially reversed |
| any-context: hook inject-on-spawn | N/A (couldn't inject; didn't own sessions) | ADOPT (`--settings` inject at spawn) | New capability |
| codemachine: EngineModule spawn_recipe | Out of scope | ADOPT + EXTEND (new `spawn_recipe()` method) | Extended |

### CONFIRMED (genes that were ADOPT before the pivot and remain ADOPT)

| Gene Source | Capability | Status |
|------------|-----------|--------|
| claude-squad | Worktree-per-session isolation (A.1) | ADOPT (extended: spawn cwd injection) |
| claude-squad | Profile selector UX (A.2, S-031) | ADOPT (already built) |
| claude-squad | Snapshot-fork concurrency (A.3) | ADOPT (extended: tokio JoinSet) |
| claude-squad | Debounced versioned filter (A.4) | ADOPT (already built, Nucleo) |
| claude-squad | Executor/PtyFactory seam (A.5) | ADOPT (extended: PtySpawner trait) |
| zellij | Client/server IPC model | ADOPT (extended: PTY byte message types) |
| zellij | Session persistence (two-file) | ADOPT (extended: daemon-owned PTY survival) |
| zellij | Per-client config overlay | ADOPT (extended: per-session params) |
| any-context | Hook protocol (5 endpoints) | ADOPT (already built, DTU clone) |
| any-context | Broker pattern (drop counter) | ADOPT (extended: per-session PTY channel) |
| any-context | Session lifecycle (Manager/Store/GC) | ADOPT (extended: spawn/kill/attach/detach) |
| nikiforovall | AppMode state machine | ADOPT (extended: EmbeddedTerminal + SessionCreation variants) |
| nikiforovall | 5-layer check_action gate | ADOPT (extended: new PTY/launch actions gated) |
| lazygit | 5-level binding precedence | ADOPT (already built, confirmed) |
| lazygit | Popup stacking VecDeque | ADOPT (already built, confirmed) |
| lazygit | Action enum dispatch | ADOPT (extended: session lifecycle + PTY + Tune actions) |
| lazygit | Panel layout (3-column) | ADOPT (extended: Preview slot hosts PTY pane) |
| CCR | Detect-on-PATH + env inject | ADOPT (already built S-031; used in spawn path) |
| codemachine | EngineModule trait | ADOPT (extended: spawn_recipe method) |
| codemachine | EngineRegistry | ADOPT (confirmed) |
| vsdd-factory | FactoryAdapter observe-only | ADOPT (confirmed) |
| vsdd-factory | VsddFactoryAdapter + STATE.md | ADOPT (already built S-025) |

### CONFIRMED LEAVE BEHIND (genes that remain out of scope)

| Gene Source | Capability | Reason |
|------------|-----------|--------|
| claude-squad | AutoYes / polling daemon (S.1) | Hook-based permission overlay is structurally superior |
| claude-squad | capture-pane scraping (S.2) | Hook protocol is the structured channel |
| claude-squad | tmux as PRIMARY multiplexer (S.3) | portable-pty + vt100 + tui-term is the native approach |
| claude-squad | PM/Worker orchestration | Human is the coordinator |
| any-context | PM/Worker persona | Out of scope by user direction (confirmed) |
| any-context | /msg/* inter-session bus | Not needed for v1 (human-driven, sessions are independent) |
| any-context | SSH reverse tunnel (Phase 4) | Re-baselined v1 scope; suspended |
| zellij | zellij as library (code import) | async-std + crossbeam + wasmi incompatible; not a library |
| zellij | 14 InputMode modal keymap | Single-mode + EmbeddedTerminal AppMode variant is sufficient |
| nikiforovall | Textual framework patterns | Python/Textual → Rust/ratatui; translate behavior, not code |
| CCR | Preset ZIP system | CCR-internal; monocle doesn't manage presets |
| CCR | Admin UI (React) | CCR's UI; monocle provides TUI-native config editing |
| codemachine | Headless-CLI spawn output handling | monocle reads PTY bytes (user sees terminal); not JSON streams |
| codemachine | Workflow templates + FSM | vsdd-factory is the workflow gene; monocle observes |
| codemachine | MCP router | Not monocle's concern |
| vsdd-factory | Execute workflows / dispatch agents | Non-Goal confirmed |
| vsdd-factory | WASM plugin SDK | Deferred from re-baselined v1 (Phase 3 suspended) |

---

## V1 Capability Map: What Monocle Builds vs. What It Models

### LAUNCH (new capability — primary reversal target)

**Genes adopted:**
- claude-squad A.1 → worktree-per-session isolation (cwd at spawn time)
- claude-squad A.5 → PtySpawner trait seam for testability
- claude-squad instance lifecycle → monocle session lifecycle (Created→Launching→Running→Detached→Killed)
- codemachine EngineModule → extended with `spawn_recipe()` method
- any-context BC-HOOK-027 → `claude --settings <hooks_settings_path>` inject at spawn
- any-context BC-HOOK-024 fix → `lock.app = 'monocle'` filter in hook JS
- zellij PTY architecture → `portable-pty + vt100 + tui-term` (model only; code via three crates)
- nikiforovall SessionCreation AppMode → launch wizard (profile picker → project picker → worktree → launch)

**What monocle builds (not reused from gene sources):**
- `SessionManager` component in `monocle-runtime` (daemon-side): owns `(pty_master, vt100::Parser, child_handle)` per session
- `PtySpawner` trait + `RealPtySpawner` (portable-pty) + `MockPtySpawner` (test)
- `CommandBuilder` construction from `EngineModule::spawn_recipe()` + worktree path + hooks config path
- Session lifecycle state machine: `SessionState` enum (Created/Launching/Running/Detached/Terminated)
- `SpawnRecipe` struct (binary, args, env) as the EngineModule → SessionManager interface

**What monocle reuses from existing Phase-1 substrate:**
- `monocle-proto` wire format (extend with PtyOutput, KeyInput, ResizePane message types)
- `monocle-ipc` UDS transport (no change; add new message types to protobuf schema)
- `EngineModule` trait base (extend; not rewrite)
- Profile/CCR integration (already in S-031)
- Hook inject mechanism (hooks-settings.json already generated; add `--settings` to spawn args)

### EMBEDDED PTY (new capability — enables "never leave the TUI")

**Genes adopted:**
- zellij PTY architecture (MODEL) → `portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4`
- lazygit panel layout → Preview slot replaced by tui-term widget in EmbeddedTerminal mode
- lazygit Action enum → `EnterEmbeddedTerminal`, `ExitEmbeddedTerminal`, `ForwardKeyToPty`, `PtyScrollUp/Down`
- nikiforovall AppMode → `EmbeddedTerminal { session_id, prior: FocusSnapshot }` new variant

**What monocle builds:**
- `portable-pty` PTY pair creation + child process spawn (in daemon SessionManager)
- Blocking PTY-reader thread → bounded `tokio::sync::mpsc::channel(N)` bridge
- `vt100::Parser` per session (owned by daemon); TUI receives PTY bytes + calls `parser.process(bytes)`
- `tui-term::PseudoTerminal` widget rendering `vt100::Screen` in the Preview pane slot
- Keyboard encoding: `crossterm::KeyEvent → terminal byte sequences` (printable, Enter, arrows, Ctrl-C, Ctrl-D, Backspace, Tab, Esc — v1 scope; exotic protocols deferred as feature ordering)
- PTY byte streaming: `PtyOutput { session_id, bytes }` message from daemon to TUI over UDS IPC
- `ResizePane` IPC message + `portable-pty.resize(PtySize)` + `vt100::Parser.set_size()` on pane area change

**Critical v1 scope decisions (from embedded-pty-evaluation.md §3.3):**
- Keyboard scope v1: printable + control keys + arrows (production-grade for Claude Code sessions)
- Keyboard scope deferred (feature ordering): Kitty keyboard protocol, mouse, bracketed paste
- tui-term fork posture: accepted as WIP-labeled; tiny surface; vendoring is cheap if needed

### MULTI-SESSION / MULTI-PROJECT (new capability)

**Genes adopted:**
- claude-squad A.3 → snapshot-fork concurrency (`Arc::clone` + `tokio::spawn` + `JoinSet`)
- claude-squad A.4 → debounced versioned filter (already built; extend to project grouping)
- claude-squad A.2 → profile selector UX (already built S-031; extend for project picker)
- zellij session persistence → `session-state.json` per session (daemon-format: PTY size, cwd, binary, started_at)
- nikiforovall → multi-project grouping in session list (group by project_name)

**What monocle builds:**
- Multi-project session list: sessions grouped by `project_name` with collapsible header rows
- Project picker overlay (new AppMode: SessionCreation step 2)
- Session kill/rename actions (SessionKill/SessionRename in Action enum)
- `session-state.json` write on session start/state change (background thread, not render path)
- Resume on daemon restart: if PID alive (kill(pid, 0)), reattach; if dead, show as Terminated
- GC policy: Terminated sessions cleaned from registry after 10-second grace period

**Persistence strategy (hard v1 requirement):**
- Daemon-owned PTY: TUI restart → TUI reconnects to daemon; daemon continues owning PTYs.
- Daemon crash: sessions lost; user re-launches. daemon.pid watchdog (supervisord/launchd/systemd
  or a built-in daemon supervisor) is the operational mitigation. This is accepted for v1.
- Session-state.json: persists enough to re-display terminated sessions in the list and offer
  re-launch with the same parameters (project, harness, profile).
- tmux control-mode: documented fallback if daemon-owned persistence proves insufficient
  (from embedded-pty-evaluation.md §7.2). Not a v1 default.

### INTERACTIVE TUNE (new capability for v1B wave)

**Genes adopted:**
- CCR routing decision tree → Tune plane routing slot display (default/background/think/longContext/webSearch)
- CCR per-session config write → interactive CCR config edit (atomic tempfile::persist write)
- nikiforovall writer/CRUD pipeline → monocle-static activation for customization editing
- nikiforovall Modal-Confirm-Callback 3-phase → every destructive Tune action follows A/B/C
- nikiforovall atomic-write fix → tempfile::persist on all config writes (already in CLAUDE.md)
- zellij per-client config overlay → per-session launch parameters (profile, model, env)

**What monocle builds (v1B Tune wave):**
- Interactive keybinding editor (list current bindings → in-line edit → write to config → hot-reload)
- CCR routing slot editor (select model per routing scenario → write CCR config → inform user of restart requirement)
- Profile management (create/edit/delete profiles in `~/.monocle/config.json`)
- monocle-static CRUD activation (enable/disable customizations, move between scopes)
- `TuneEditBinding`, `TuneApplyProfile`, `TuneResetBinding` actions in the Action enum

### OBSERVE / CONTROL (existing, preserved + extended)

**Preserved from Phase 1 (no changes to existing BCs):**
- Hook ingestion (5 endpoints, PID-liveness, DTU clone) — BC-2.05.*, BC-2.06.*
- Permission overlay (VecDeque stack, killer scenario ≤6 keystrokes) — BC-2.06.022
- Sessions panel (session list, token burn rate, phase tag) — BC-2.05.002
- Event ribbon (rolling hook event log with filter) — BC-2.06.006/018
- Profile picker (Ctrl-P, sticky-per-project, CCR path) — BC-2.07.004/005
- Workflow plane (FactoryAdapter, VsddFactoryAdapter, STATE.md parsing) — existing

**Extended by control-center pivot:**
- Sessions panel gains: Create (launch) + Kill + Detach + Enter-Terminal actions.
- Sessions panel layout: adds project grouping rows.
- Workflow plane: pre-populated from session launch project_root (new trigger path).

---

## Architecture Implications for the Vision Revision

These decisions must be captured in the revised product brief and architecture delta:

### 1. The Daemon Becomes the Session Owner (central architectural inversion)

Today: daemon receives hook POSTs from sessions the user launched elsewhere.
After pivot: daemon SPAWNS sessions (via SessionManager) and owns their PTYs.
The TUI is a client that streams PTY bytes and forwards keystrokes over existing UDS IPC.

This is the architectural center of gravity of the re-baselined v1. Everything else follows
from it.

### 2. `EngineModule` Trait Extension Required

The existing `EngineModule` trait gains `spawn_recipe(opts: &SpawnOptions) -> SpawnRecipe`.
This is a non-breaking addition (default impl: return `Err(UnsupportedOperation)` for
engines that don't support monocle-spawned sessions). ClaudeCodeModule implements it.

### 3. New Protobuf Message Types in `monocle-proto`

Three new message types on the existing UDS wire:
- `ServerToClient::PtyOutput { session_id: Uuid, bytes: Bytes }` — PTY byte stream (high-frequency)
- `ClientToServer::KeyInput { session_id: Uuid, bytes: Bytes }` — keystroke forwarding
- `ClientToServer::ResizePane { session_id: Uuid, rows: u16, cols: u16 }` — pane resize

These extend the existing proto schema (no breaking changes to existing message types).

### 4. New Crate: `monocle-session-manager` (or extension of `monocle-runtime`)

The SessionManager is the new daemon-side component that owns the `(pty_master, vt100::Parser,
child_handle)` triples. It is either a new sub-module of `monocle-runtime` or a separate crate.

Decision for architect/human: sub-module (simpler, no additional crate boundary) vs. separate
crate (cleaner separation, enables unit testing in isolation). Recommend sub-module of
`monocle-runtime` for v1A launch wave — the PTY ownership is intrinsically a daemon
responsibility. Flag as a cross-component architecture decision.

### 5. New Dependencies (ratified from embedded-pty-evaluation.md §7.1)

Three new crates to add to `monocle-runtime` and `monocle-tui`:
```toml
# monocle-runtime (spawner/supervisor)
portable-pty = "0.9"
vt100        = "0.16"

# monocle-tui (renderer)
tui-term     = "0.3"
vt100        = "0.16"
```

MSRV impact: none (tui-term MSRV 1.86 ≤ monocle's 1.88).
License: all MIT, compatible.
RUSTSEC: none found as of 2026-06-03.
Cargo-init spike required: `cargo tree -d` to verify single resolved versions of
`ratatui-core`, `ratatui-widgets`, `vt100` before committing the dep (medium-confidence
compatibility; verify at Cargo-init step per embedded-pty-evaluation.md §2.1 caveat).

### 6. v1 Delivery in Two Waves

Based on this disposition, the re-baselined v1 naturally decomposes into:

**v1A (Launch Wave):** LAUNCH + EMBEDDED PTY + MULTI-SESSION/MULTI-PROJECT
- New daemon capabilities: SessionManager, PtySpawner, PTY ownership, session state persistence
- New TUI capabilities: EmbeddedTerminal AppMode, PTY widget, session creation wizard,
  multi-project session list with create/kill/attach/detach actions
- Extended: EngineModule.spawn_recipe(), monocle-proto new message types
- Existing preserved: all Phase-1 observe/control capabilities (hook ingestion, permission
  overlay, profile picker, workflow plane, event ribbon)

**v1B (Tune Wave):** INTERACTIVE TUNE
- monocle-static writer/CRUD activation (NikiforovAll genes)
- Interactive keybinding editor
- CCR routing slot editor
- Profile management UI
- Activated from existing AppMode state machine (no new AppMode variants)

This is feature ordering per CLAUDE.md Rule 2, not an MVP shortcut. Both waves ship
production-grade on their respective cycles.

---

## Cross-Component Questions Requiring Human/Architect Adjudication

These are genuine architecture decisions that cross component boundaries or require human
input. They are NOT deferrals of mechanical questions (per CLAUDE.md anti-pattern rules).

### Q1: PTY bytes over existing UDS IPC vs. dedicated streaming path
**Decision required:** Should PTY bytes (`PtyOutput` messages) share the existing UDS IPC
channel with hook events and command messages, or should there be a dedicated high-throughput
streaming channel per session?
- Option A (shared channel): simpler; risk of PTY byte bursts starving hook-event delivery.
  Mitigated by per-session bounded channels with separate buffers.
- Option B (dedicated per-session stream): cleaner throughput isolation; adds socket-per-
  session complexity or a multiplexed streaming protocol.
**Recommendation:** Option A (shared channel with per-session PTY byte buffer sized at 1024).
The drop-counter convention surfaces any starvation. Benchmark before v1A launch gate.
**Needs:** architect adjudication (this is the Q4 item from embedded-pty-evaluation.md §8).

### Q2: SessionManager as monocle-runtime sub-module vs. separate crate
**Decision required:** See §4 above. Recommend sub-module; flag for architect confirmation.
**Needs:** architect decision only (no human input required for this mechanical choice).
**Self-adjudicated answer:** SUB-MODULE. Rationale: (a) SessionManager is a daemon responsibility
and shares daemon-internal types (Child handles, PTY masters); (b) no other crate needs to
depend on SessionManager directly (the proto wire is the interface); (c) a separate crate
boundary adds friction without enabling new test isolation (the PtySpawner trait provides the
test seam regardless of crate structure). This is a mechanical architecture decision answerable
in scope per CLAUDE.md Rule 6.

### Q3: Daemon crash persistence posture for v1A
**Decision required:** Is cross-daemon-crash session survival a hard requirement for v1A
launch, or is daemon-restart-with-re-display-and-relaunch acceptable?
**Recommendation from this analysis:** Daemon restart → sessions lost → re-launch. The daemon
is stable enough that crash recovery is exceptional; the supervisor (launchd/systemd or monocle's
own daemon watchdog) can restart it. For v1A this is acceptable. Cross-crash PTY survival
(which would require either tmux fallback or PTY-state serialization) is deferred as a later-
wave feature.
**HUMAN DECISION NEEDED:** Is this acceptable for v1A, or must the daemon survive crashes
with sessions intact? This changes the architecture significantly (toward tmux control-mode or
PTY state serialization). Human must confirm or override.
See embedded-pty-evaluation.md §8 Q3 for the full trade-off analysis.

### Q4: Keyboard fidelity scope for v1A embedded terminal
**Decision required:** Which key/input protocols are in v1A production scope?
**Recommendation:** printable + control keys (Ctrl-C, Ctrl-D, Ctrl-Z) + arrows + Backspace +
Tab + Esc + Enter. This covers everything a typical Claude Code session needs.
**Deferred as feature ordering (CLAUDE.md Rule 2):** Kitty keyboard protocol, mouse support,
bracketed paste. These are features; they ship when needed, not when possible.
**HUMAN DECISION NEEDED:** Are there specific Claude Code session interactions that require
mouse or Kitty-protocol keys that the user considers v1A requirements? If so, scope must expand.

### Q5: v1A and v1B wave boundary confirmation
**Decision required:** Is the two-wave split (Launch first, Tune second) acceptable, or does
the user require Interactive Tune in the same wave as Launch?
**Recommendation:** Two waves. The launch capability is the highest-value reversal; Tune
activation is important but secondary. Feature ordering, not MVP compromise.
**HUMAN DECISION NEEDED:** Confirm or override the two-wave split.

---

## Semport Index Note

No formal semport index file (`semport-index.md` or equivalent) exists in the
`.factory/semport/` directory. The 8 subdirectories contain their own synthesis files.
This rollup document serves as the index for the v2 disposition pass.
If a formal index is desired for the semport directory, it should be created by the
state-manager or product-owner as a separate operation — not invented here per the
"do not invent an index unprompted" constraint in the task brief.

---

## Document Map (Individual Dispositions)

| Gene Source | File | Primary New Finding |
|------------|------|---------------------|
| claude-squad | `claude-squad/claude-squad-disposition-v2-control-center.md` | Launch mechanism → MODEL (portable-pty, not tmux); lifecycle state machine MODEL |
| zellij | `zellij/zellij-disposition-v2-control-center.md` | PTY architecture → MODEL (most critical new adoption) |
| any-context/lazyclaude | `any-context-lazyclaude/any-context-lazyclaude-disposition-v2-control-center.md` | Hook inject-on-spawn: `--settings` flag + `lock.app` filter upgrade to v1 |
| nikiforovall/lazyclaude | `nikiforovall-lazyclaude/nikiforovall-lazyclaude-disposition-v2-control-center.md` | EmbeddedTerminal + SessionCreation AppMode variants; writer genes → v1B |
| lazygit | `lazygit/lazygit-disposition-v2-control-center.md` | Preview slot → PTY widget; new Action variants; no major reversals |
| claude-code-router | `claude-code-router/claude-code-router-disposition-v2-control-center.md` | Routing slot display for Tune plane; interactive config write v1B |
| codemachine-cli | `codemachine-cli/codemachine-cli-disposition-v2-control-center.md` | EngineModule.spawn_recipe() new method; headless-CLI clarified as Leave-Behind for PTY path |
| vsdd-factory | `vsdd-factory/vsdd-factory-disposition-v2-control-center.md` | Non-Goal explicitly confirmed; session-launch → project-root → factory-detection new trigger |
