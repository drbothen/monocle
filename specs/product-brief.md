---
document_type: product-brief
level: L1
version: "2.0.2"
status: draft
producer: product-owner
phase: pivot-delta-brief
timestamp: 2026-06-03T23:30:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md
  - semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md
  - semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md
  - semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md
  - semport/zellij/zellij-pass-8-final-synthesis.md
  - semport/lazygit/lazygit-pass-8-final-synthesis.md
  - semport/claude-squad/claude-squad-pass-8-deep-synthesis.md
  - semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md
  - planning/oq-research.md
  - semport/DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md
  - specs/research/embedded-pty-evaluation.md
  - STATE.md
input-hash: "13e1215"
traces_to: >
  factory-artifacts 2737bfd (vision-synthesis v1.0 approved); 2c2b676 (8-repo full ingest);
  b3c68ca (OQ research); vision-synthesis v2.1 (D-238 approved 2026-06-03);
  D-236 (pivot); D-237 (v1 scope ratified); D-238 (vision v2.1 approved + session-host-owns-PTY)
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
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/embedded-pty-evaluation.md
  - /Users/jmagady/Dev/monocle/.factory/semport/DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md
---

# Product Brief: Monocle (v2.0.2 — I18-001 §Out-of-Scope Consistency Correction)

> **D-236/D-237/D-238 RE-BASELINE (2026-06-03).**  
> The observe-only framing of v1.4.x is RETIRED. Monocle is now a full TUI control center.
> The Phase-1 substrate (daemon, hook ingestion, permission overlay, EngineModule/FactoryAdapter
> traits, proto, ring, TUI rendering — 1514 tests, 9 crates) is preserved and extended, not rebuilt.

## §Trace — Revision History

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-12 | product-owner | Initial brief — observe-only scope, Phase 1–4 roadmap. |
| 1.1–1.4.34 | 2026-05-12–2026-05-30 | product-owner + architect | Incremental tightening: OQ/SOQ resolutions, forward-compatibility contracts, DTU clone, adversary finding closures, version-pin back-cascades. Full history preserved in git (factory-artifacts branch). Normative decisions from this lineage that survive into v2.0 are cited in §Preserved Substrate below. |
| **2.0.0** | **2026-06-03** | **product-owner** | **D-236/D-237/D-238 CONTROL-CENTER RE-BASELINE.** Retired: observe-only constraint and all residual "observe-only" framing in scope/non-goals/roadmap. Retired: Phase 1–4 observe-only roadmap. Added: LAUNCH, EMBEDDED PTY, MULTI-SESSION/MULTI-PROJECT, INTERACTIVE TUNE as first-class v1 capabilities. Replaced roadmap with v1A/v1B wave plan. Replaced success criteria with control-center bar. Replaced competitive positioning. session-host-owns-PTY persistence model (D-238). Hook auto-injection on spawn. Preserved: all normative decisions from v1.4.x lineage that remain valid (OQ resolutions, forward-compatibility contracts, non-goals that are still non-goals). Traces to vision-synthesis v2.1 (approved D-238 by Joshua Magady 2026-06-03). Status: draft — pending adversarial review and human gate before architecture delta proceeds. |
| **2.0.1** | **2026-06-03** | **product-owner** | **Consistency propagation of v2.1 session-host model + architect rulings; no scope change.** Applied ADR-0009 and SS-08: `portable-pty` and `vt100` crate locations corrected to `monocle-session-host` in §Tech Direction table (CRIT-2); `PtySpawner`/`MockPtySpawner` replaced with `SessionHostSpawner`/`MockSessionHostSpawner` throughout (CRIT-3), including §Success Criteria "verified via integration test with MockSessionHostSpawner"; `VsddFactoryAdapter`/`FactoryAdapter` extraction source corrected to `monocle-core` in §Crate Workspace Layout (IMP-1); `monocle-session-host` added as v1A new crate in §Crate Workspace Layout; permission badge+bell guarantee for EmbeddedTerminal mode added with BC-pending note; v1B pre-emption open item recorded (SUG-3). Status: draft (consistency-only; no scope change). |
| **2.0.2** | **2026-06-04** | **product-owner** | **I18-001 consistency correction — §Out-of-Scope daemon-owned framing aligned with ratified D-238 session-host-owns-PTY model; no scope change.** §Out of Scope "Does NOT replace the user's general-purpose terminal multiplexer" bullet: "daemon-owned PTYs" → "session-host-owned PTYs (daemon-coordinated)". This is a consistency correction to the already-ratified session-host-owns-PTY decision (ADR-0009, D-238); it does NOT change any approved decision. Status: draft (consistency-only; no scope change). |

---

## What Is This?

Monocle is a Rust TUI control center for AI coding harness sessions. A single `Ctrl-\` popup
gives you the complete session management surface: launch new Claude Code (and future harness)
sessions, watch them run in an embedded terminal pane inside monocle, manage many sessions
across many projects, resolve permission prompts without leaving the TUI, and tune your
customization configuration — all without switching terminal windows.

> *"We need to be able to launch, manage, and observe — a better lazyclaude, a better
> claude-squad. We should never have to leave the TUI and we should be able to manage
> as many sessions and as many projects with sessions as we need to. We need to be able
> to run, launch, manage, observe, tune, control — everything from the TUI."*
> — Joshua Magady, D-236 (2026-06-03), verbatim.

The architectural inversion from the observe-only scope: the monocle daemon no longer merely
receives hooks from sessions the user launched elsewhere. **The daemon now spawns and owns
sessions.** The TUI is a rich client that streams PTY output, forwards keystrokes, and manages
session lifecycle over the existing Unix domain socket IPC. The five-plane architecture,
EngineModule/FactoryAdapter traits, permission overlay, and all Phase-1 substrate remain;
they are extended, not replaced.

---

## Who Is It For?

| Persona | Pain Point | Current Workaround |
|---------|-----------|-------------------|
| **Multi-session Claude Code developer** — 2–4 parallel Claude Code sessions across worktrees or projects | Launching a new session requires opening a terminal, navigating to the project, configuring hooks manually; permission prompts from one session stall while the developer is focused on another; must context-switch tmux windows to check status | Manually open terminal windows; `cat` hook settings; `Ctrl-b n` to find the right pane; restart stalled sessions |
| **Factory-pattern operator** — vsdd-factory-style pipelines with STATE.md phase tracking | Needs situational awareness without leaving the editor; blocking issues invisible until a session stalls; launching a new factory session requires separate terminal management | Manual `cat .factory/STATE.md`; `tree .factory/`; separate terminals per session |
| **Multi-harness operator** (v2 target, design must accommodate) — Claude Code + CodeMachine sessions simultaneously | No unified launch surface, no unified status view across harnesses | Open separate TUI instances per harness; no aggregate cost view |

**The killer scenarios that motivate v1 scope** — per vision §End-to-End Killer Scenarios:

1. **Launch and enter a session:** Developer presses `+` in the TUI, selects a profile and
   project in the SessionCreation wizard, and monocle launches `claude --settings /tmp/monocle-hooks-xyz.json`
   in the worktree with hook auto-injection. Developer presses Enter to enter the embedded
   terminal pane. Sends a prompt. Returns to dashboard with Esc. No new terminal window opened.
   No hook settings file manually configured.

2. **Multi-session permission resolution (preserved from v1.1.2):** Three sessions running,
   two concurrent permission prompts. Four keystrokes (`Ctrl-\`, `2`, `1`, `Ctrl-\`) resolves
   both prompts with zero window switches, zero session stalls. The cascaded VecDeque
   overlay showing both prompts simultaneously is the core UX advantage.

---

## Scope

### In Scope — v1 Capabilities

The four capabilities below plus the already-built Observe + Control (permission overlay)
constitute v1. The two-wave ordering is feature sequencing per CLAUDE.md CANONICAL PRINCIPLE
Rule 2: each wave ships production-grade on its cycle. No capability is "MVP" or deferred for cleanup.

#### Wave v1A: Launch Wave

**LAUNCH** — the core architectural inversion.

monocle spawns and owns AI coding harness sessions from the TUI. The daemon SessionManager
spawns a native `monocle-session-host` process per session (via `SessionHostSpawner`), which
opens a `portable-pty` PTY pair, builds a CommandBuilder from `EngineModule::spawn_recipe()`
(binary + args including `--settings <hooks_settings_path>` for hook auto-injection + cwd as
git worktree root + CCR env vars if applicable), and owns the harness child process. The daemon
coordinator tracks session-host processes and proxies PTY bytes to TUI clients. Session
lifecycle: `Created → Launching → Running → Detached → Terminated`.

The `EngineModule` trait is extended with:

```rust
fn spawn_recipe(&self, opts: &SpawnOptions) -> Result<SpawnRecipe, EngineError>
```

`ClaudeCodeModule` implements this. The `SpawnRecipe` carries `binary`, `args`, `env`, and `cwd`.
Default impl returns `Err(UnsupportedOperation)` for engines that do not yet support monocle-spawned
sessions.

**EMBEDDED PTY** — never leave the TUI.

The running session is visible and interactive inside monocle. The TUI Preview pane slot hosts a
`tui-term::PseudoTerminal` widget that renders the `vt100::Screen` from the TUI-side `vt100::Parser`
(fed by `PtyOutput` IPC bytes proxied by the daemon from the session-host). The TUI sends keystrokes
to the daemon as `KeyInput` IPC messages; the daemon forwards them to the session-host which writes
PTY-encoded bytes to the master. The user reads and responds to the Claude session inside monocle
without switching windows.

PTY byte stack: `portable-pty 0.9.0` (in `monocle-session-host`) + `vt100 0.16.2` (in
`monocle-session-host` and `monocle-tui`) + `tui-term 0.3.4` (in `monocle-tui`). All MIT, no RUSTSEC,
MSRV 1.88 compatible, ratatui 0.30 compatible. See §Tech Direction and vision §Tech Stack.

Input fidelity (v1A): full fidelity — printable keys + control keys (Ctrl-C, Ctrl-D, Ctrl-Z) +
arrows + Backspace + Tab + Esc + Enter + mouse events + Kitty keyboard protocol. Bracketed paste
included. No input class is deferred. Human-ratified at D-237. Architect routes implementation
details (crossterm Kitty-enhancement flags + mouse capture → PTY byte translation → portable-pty
master write) in the architecture delta.

AppMode is extended with `EmbeddedTerminal { session_id, prior }` and
`SessionCreation { step, prior }` variants. The Preview pane transitions from session detail to
the live PTY widget when the user enters embedded terminal mode.

**MULTI-SESSION / MULTI-PROJECT**

List, switch, create, kill, rename, and group sessions by project from the TUI. Sessions grouped
by `project_name` with collapsible header rows. Fast switching: O(1) — swap which parser the TUI
widget renders; all sessions parse in the background. Project picker overlay (SessionCreation step 2).
GC policy: Terminated sessions cleaned from registry after 10-second grace period.

`session-state.json` per session persists enough metadata to re-display terminated sessions and
offer re-launch with the same parameters (project, harness, profile).

**Hook auto-injection on spawn:** when monocle launches a session it writes the hooks settings
file to a per-session temp path and passes `--settings <path>` in the CommandBuilder args. The
`lock.app = 'monocle'` filter ensures only monocle-launched sessions trigger the monocle hook
endpoint. No manual `~/.claude/settings.json` copy required.

**Persistence model (session-host-owns-PTY; daemon coordinates/re-attaches) — three cases (D-238):**

PTY masters and harness child processes are owned by native detached per-session session-host
processes that outlive the daemon process. The daemon coordinates session-hosts and re-attaches
over UDS on restart.

1. **TUI exit and reconnect (supported in v1A):** session-host processes continue owning PTY
   masters and child handles; daemon remains running; reconnecting TUI client re-streams from
   existing `vt100::Parser` state.
2. **Graceful daemon-process restart (REQUIRED to survive in v1A — D-238):** when the daemon
   restarts gracefully, sessions MUST survive via the native session-host processes. The daemon
   re-attaches to running session-hosts on startup. Default: native implementation (no external
   multiplexer dependency; no-tmux preserved). If native proves infeasible at acceptable cost,
   the architect MUST surface the external-supervisor tradeoff for human decision — not silent
   adoption. Architecture route: Q-8 in vision §Open Questions (HIGH priority).
3. **Hard daemon crash (accepted v1A boundary):** crash → sessions lost → user re-launches. No
   clean handoff; the daemon is stable; this is exceptional. launchd/systemd or a monocle-internal
   daemon watchdog is the operational mitigation. Cross-crash PTY state serialization is explicitly
   out of v1A scope.

The already-built in-process D-235 daemon wiring placed PTY ownership inside the daemon process.
Moving PTY ownership to session-host processes requires reworking the SessionManager abstraction
boundary. This is a known architectural consequence of D-238; architect routes in the architecture
delta (Q-8). Do not attempt to preserve the D-235 wiring pattern for persistence — it contradicts
D-238.

**New monocle-ipc message types (v1A):** `PtyOutput { session_id, bytes }`,
`KeyInput { session_id, bytes }`, `ResizePane { session_id, rows, cols }`. These share the
existing UDS channel with hook events and control messages (Option A per embedded-pty-evaluation.md
§8 Q4 recommendation); the architect confirms or revises at the architecture delta with a
pre-v1A-gate PTY-bytes throughput benchmark.

#### Wave v1B: Interactive Tune Wave

**INTERACTIVE TUNE**

The Static plane becomes interactive. The user edits keybindings, profile definitions, and CCR
routing slots directly in the TUI; changes take effect via atomic `tempfile::persist` writes
then hot-reload. Every destructive Tune action follows the NikiforovAll Modal-Confirm-Callback
3-phase pattern.

New crate: `monocle-static`. CRUD activation: enable/disable customizations, move between scopes.
Profile management: create/edit/delete profiles in `~/.monocle/config.json`. CCR routing slot
editor: select model per routing scenario, write CCR config, inform user of any restart
requirements. New Action variants: `TuneEditBinding`, `TuneApplyProfile`, `TuneResetBinding`,
`TuneEditCcrSlot`.

#### Already-Built Substrate — Preserved and Extended

The Phase-1 substrate from Waves 1–7 is the foundation for v1A/v1B. It is not rebuilt.

| Capability | Status | Existing BCs |
|-----------|--------|-------------|
| Hook ingestion (5 endpoints, PID-liveness, dual-accept auth, DTU clone) | Preserved | BC-2.05.*, BC-2.06.* |
| Permission overlay (VecDeque stack, killer scenario ≤6 keystrokes) | Preserved | BC-2.06.022 |
| Sessions panel (session list, token burn rate, phase tag) | Extended (project grouping + launch/kill/detach actions) | BC-2.05.002 |
| Event ribbon (rolling hook event log with session-ID filter) | Preserved | BC-2.06.006/018 |
| Profile picker (Ctrl-P, sticky-per-project, CCR path) | Preserved + extended (used in launch wizard step 1) | BC-2.07.004/005 |
| Workflow plane (FactoryAdapter observe-only, VsddFactoryAdapter, STATE.md parsing) | Preserved + new launch trigger path | existing |
| Daemon (axum HTTP, UDS, bounded broker, JSONL ring) | Extended with SessionManager + PTY byte fan-out | BC-2.01.* |
| Forward-compatibility contracts (ABI version, non-exhaustive enums, FactoryAdapter trait, proto schema, ring format version, auth token format) | Preserved and extended | BC-2.02.* |

The D-235 convergence work (daemon now actually serves: `daemon_start_sequence + run_server +
UDS + tracing + ring-flush + 10s drain`) is the current live state of monocle-runtime. It is the
starting substrate; it is extended in v1A, not replaced — except the in-process PTY ownership
assumption which D-238 supersedes (see Q-8 architect route above).

### Out of Scope (Hard Boundaries)

These are hard boundaries, not deferred features. The factory-observe non-goal is the one that
survived the D-236 pivot intact — monocle observes factory state but never mutates it.

- **Does NOT execute vsdd-factory workflows** — monocle observes `.factory/STATE.md` (reads via
  FactoryAdapter); it never writes STATE.md, never triggers factory phases, never dispatches
  factory agents. The Workflow plane is read-only. This is the specific non-goal that the D-236
  pivot did NOT reverse.
- **Does NOT write STATE.md** — `VsddFactoryAdapter` reads; monocle never mutates.
- **Does NOT route LLM API requests** — CCR integration is detect-on-PATH + config-write +
  env-inject; monocle does not proxy or modify LLM traffic (integrate-external, D-010).
- **Does NOT replace the user's general-purpose terminal multiplexer** — monocle runs inside
  the user's tmux session; it manages AI coding sessions via its own session-host-owned PTYs
  (daemon-coordinated); it does not attempt to multiplex the user's non-AI terminal work.
- **Does NOT include PM/Worker multi-agent orchestration** — human is always the coordinator;
  sessions are independent (no inter-session bus, no automated handoff).
- **Does NOT own session transcripts** — monocle reads hook events (fine-grained, ephemeral);
  full transcript storage belongs to Claude Code's own persistence layer.
- **Does NOT build its own LLM provider abstraction** — CCR is the external router; monocle
  integrates by detecting it.
- **Does NOT include `PostToolUse` hook endpoint in v1** — JC-2 parity with Claude Code
  gene-source (any-context BC-HOOK-007, 5-endpoint canonical set). Revisit if a future wave
  requires PostToolUse data.
- **Does NOT ship the WASM plugin SDK in v1** — Phase 3 scope, suspended. v1 statically bundles
  `VsddFactoryAdapter` as the sole factory adapter.
- **Does NOT ship the rmcp MCP bridge in v1** — suspended. OQ-09.
- **Does NOT use tmux as the PRIMARY session multiplexer** — `portable-pty` native in-process
  PTY is the chosen path. tmux control-mode and other external supervisors are documented
  architect-surfaced fallbacks for human decision only; they are not default choices.
- **Does NOT include SSH federation in v1** — suspended (was Phase 4 old roadmap).

---

## Success Criteria

v1A ships when ALL of the following pass:

| Outcome | Metric | Target |
|---------|--------|--------|
| Session launch from TUI | User launches a Claude Code session from the TUI, with hooks auto-injected, without opening a new terminal window or manually configuring a settings file | Launcher wizard completes in ≤5 keystrokes from Dashboard to session Running state; `--settings` flag verified in the child process args; hook POST received at the daemon within 2s of session start |
| Embedded terminal rendering | Running session is visible and interactive in the Preview pane via tui-term PTY widget | PTY output renders within 100ms of byte receipt; no visual corruption on 80×24 terminal; full ANSI/VT100 sequence support (verified against fixture corpus from embedded-pty-evaluation.md) |
| Input fidelity | All input classes forward correctly to the PTY | Printable keys, Ctrl-C/D/Z, arrows, Backspace, Tab, Esc, Enter, mouse events, and Kitty keyboard protocol all reach the child process stdin unmodified; verified via integration test with MockSessionHostSpawner |
| Session persistence — TUI reconnect | TUI process exits and reconnects; sessions survive | Sessions remain Running after TUI process exits; reconnecting TUI client sees current parser state; no PTY byte loss during reconnect window |
| Session persistence — daemon restart | Daemon process restarts gracefully; sessions survive (D-238) | All Running sessions are present and streaming after a graceful daemon restart; session-host processes survive the daemon restart; re-attach latency ≤5s |
| Multi-session / multi-project | Multiple sessions across multiple projects manageable from the TUI | 3 sessions across 2 projects visible, grouped by project, with filter; kill/rename actions work; session-state.json persists and re-displays terminated sessions |
| Permission prompt latency | Permission overlay renders after hook fires | ≤100ms from hook POST receipt to TUI overlay render on localhost (preserved from Phase-1) |
| Hook ingestion timeout budget | Daemon responds within Claude Code's upstream timeout ceilings | ≤300ms for `PreToolUse`, `Stop`, `SessionStart`, `UserPromptSubmit`; ≤2000ms for `Notification` (gene-source BC-HOOK-022) |
| Killer scenario — multi-permission | Both prompts resolve without window switches | ≤6 keystrokes (`Ctrl-\`, `2`, `1`, `Ctrl-\`) clears 2 concurrent permission prompts (per vision killer scenario 2) |
| PTY bytes throughput (pre-gate benchmark) | PTY byte bursts do not starve hook-event delivery | Benchmark PTY bytes over UDS + bounded-channel at N concurrent sessions; confirm drop counter does not fire under terminal-refresh rate load; ≥1000 events/sec per CLAUDE.md Conventions |
| Hook protocol parity | Byte-compatible with Claude Code schema | Fixture-based parity test passes (5 endpoints; dual-accept auth: canonical `X-Monocle-Authorization` AND compatibility alias `X-Claude-Code-Ide-Authorization`; per ADR-0005) |
| Build matrix | macOS + Linux, both architectures | CI green on darwin/linux × amd64/arm64 |
| Drop counter active | Bounded event bus with visible drop counter | No unbounded channels; drop counter renders in status bar under 1000 events/sec synthetic load |

v1B ships when ALL of the following pass:

| Outcome | Metric | Target |
|---------|--------|--------|
| Keybinding edit | User edits a keybinding in the TUI and change takes effect | Atomic write via `tempfile::persist`; binding active in next key dispatch cycle; Modal-Confirm-Callback 3-phase pattern for destructive edits |
| Profile create/edit/delete | Full profile CRUD from TUI | Profiles in `~/.monocle/config.json` created/edited/deleted; hot-reload; no corruption on concurrent access |
| CCR routing slot edit | CCR model routing configurable from TUI | CCR config written; `ANTHROPIC_BASE_URL` updated; user notified of any restart requirement |
| 7 customization types render | All 7 types visible in Static plane | Zero missing types on a Claude Code project with all 7 type examples (slash commands, subagents, skills, memory files, MCP servers, hooks, LSP servers) |

**Preserved Phase-1 success criteria (still required):**

| Outcome | Metric | Target |
|---------|--------|--------|
| Hook receiver body size limit | Daemon enforces 256 KiB max on all hook POST endpoints | Exceeding returns HTTP 413 with `{"error":"payload_too_large","limit_bytes":262144}`. BC-2.01.003. |
| DTU clone fidelity | `dtu-claude-code-hooks-v1` clone exists and validates | Fidelity ≥0.95 against fixture corpus; all 5 endpoint payloads schema-valid; CI per-PR gate. Per NFR-011. DTU already validated at D-234 (fidelity 1.0000, 25/25 fixtures). |
| Forward-compatibility contracts | All 6 FC items shipped | 22 BCs active in PRD: BC-2.02.001/002 (ABI), BC-2.02.003 (Types), BC-2.02.004/005 (Factory), BC-2.02.006/007/008 (Proto), BC-2.01.007 (Ring), BC-2.01.008/009 (Auth), BC-2.03.001/002/003/004 (Engine), BC-2.01.010 (Lock). Per BC-INDEX. |
| Factory detection | vsdd-factory project detected; workflow panel populated | Self-referential integration test against monocle's own `.factory/`. |

---

## Constraints and Integration Points

### Tech Direction — PTY Stack (v1A)

New crates added in v2.0 (from `embedded-pty-evaluation.md` v1.0 §7.1, confirmed D-237):

| Crate | Version | Location | Role | License | RUSTSEC |
|-------|---------|----------|------|---------|---------|
| `portable-pty` | `"0.9"` | `monocle-session-host` | PTY pair creation, child spawn, master read/write | MIT | none (2026-06-03) |
| `vt100` | `"0.16"` | `monocle-session-host`, `monocle-tui` | ANSI/VT100 parse → in-memory screen state | MIT | none (2026-06-03) |
| `tui-term` | `"0.3"` | `monocle-tui` | ratatui widget rendering `vt100::Screen` | MIT | none (2026-06-03) |

Compatibility notes:
- `tui-term 0.3.4` depends on `ratatui-core ^0.1.0` and `ratatui-widgets ^0.3.0` — exactly what
  `ratatui 0.30.0` pins. Unifies to a single copy in the dependency graph. Cargo-init spike
  required at architecture delta: `cargo tree -d` must show zero duplicate `ratatui-core`/
  `ratatui-widgets`/`vt100` versions before committing.
- MSRV impact: `tui-term` MSRV 1.86 ≤ monocle's 1.88 floor. MSRV unchanged.
- `tui-term 0.3.4` self-describes as "work in progress." Decision-of-record: exact-pin +
  vendoring-plan-if-needed. Do NOT enable the `unstable` feature flag (the tui-term spawn
  helper); monocle spawns via `portable-pty` in the runtime. Architect confirms or revises
  vendoring posture in the architecture delta (Q-7 in vision §Open Questions).
- `tmux` is a documented fallback only. It is not a v1 default; it requires human decision
  if the architect determines native session-host ownership is infeasible for Q-8.

**Canonical version pin manifest:** `SS-deps-pin-manifest.md`. Supersedes version examples here.

### Architecture Authority — Unchanged from v1.4.x

The following architecture decisions from the Phase-1 substrate are not up for re-selection
in v1A/v1B:

- **All version pins, RUSTSEC audit, and MSRV policy:** `SS-deps-pin-manifest.md`
- **wasmtime-vs-wasmi:** `ADR-0001`
- **Anti-pattern enforcement, semgrep, conventions:** `SS-conventions-anti-patterns.md`
- **Exhaustive vs. non-exhaustive enum policy:** `ADR-0004`, `SS-core-types-and-abi.md`
- **Hook endpoint set (5 endpoints, JC-2):** locked; see Phase-1 OQ/JC resolutions below
- **Dual-accept auth header:** `ADR-0005`
- **License selection:** `ADR-0003`
- **Daemon lifecycle (axum HTTP + UDS, JSONL ring, graceful shutdown):** `SS-daemon-lifecycle.md`

When two artifacts disagree, the LATER, MORE-SPECIFIC artifact wins. Per CLAUDE.md §Architectural Authority.

### Crate Workspace Layout

As of D-232/Wave-7 gate: 9 workspace crates exist — `monocle-core`, `monocle-runtime`,
`monocle-proto`, `monocle-test-harness`, `monocle` (binary), `monocle-config`, `monocle-ipc`,
`xtask`, `monocle-tui`. The v1A/v1B delta introduces:

- **`monocle-runtime` extended**: `SessionManager` sub-module (new; coordinator role — spawns
  and tracks `monocle-session-host` processes, holds per-session UDS connections to hosts,
  re-discovers and re-attaches on daemon startup); PTY byte fan-out on existing broker.
  SessionManager is a sub-module of monocle-runtime (not a separate crate) — confirmed by
  SS-08 (architecture delta). SessionManager does NOT own PTY masters or child handles
  directly; those live in `monocle-session-host` processes (ADR-0009).
- **`monocle-session-host` (new, v1A)**: native detached per-session binary. Owns the
  `(pty_master, vt100::Parser, child_handle)` triple. Setsid'd to survive daemon restarts.
  Exposes a per-session UDS socket at `<runtime_dir>/session-<uuid>.sock`. Packaged alongside
  `monocle` in the release bundle. Expands the workspace from 9 to 10 crates.
- **`monocle-tui` extended**: `EmbeddedTerminal` AppMode variant; `tui-term` PTY widget;
  `SessionCreation` wizard.
- **`monocle-ipc` extended**: `PtyOutput`, `KeyInput`, `ResizePane` message types.
- **`monocle-proto` extended**: PTY message proto types.
- **`monocle-test-harness` extended**: `MockSessionHostSpawner` (SessionHostSpawner trait test double).
- **`monocle-static` (new, v1B)**: customization reader + writer (CLAUDE.md, settings.json,
  hooks, keybindings) with interactive CRUD activation.
- **`monocle-workflow` (extract, v1B)**: `FactoryAdapter` trait + `VsddFactoryAdapter` extracted
  from `monocle-core` to own crate. The struct/logic currently lives in `monocle-core`
  (`crates/monocle-core/src/factory/`: `mod.rs` for trait, `vsdd.rs` for struct); v1B
  extracts it to `monocle-workflow` per the vision workspace layout. monocle-runtime USES
  the adapter but does not own the definition.
- **`monocle-fuzz` (v1A or v1B)**: cargo-fuzz targets for parser and hook endpoint fuzzing.

No crate outside the binary may depend on the binary crate. `monocle-plugin-sdk` remains
suspended (Phase 3).

### Action Enum and AppMode — Extended

The `Action` enum is extended with (new in v2.0; architect adds to `SS-core-types-and-abi.md`):
`SessionCreate`, `SessionDetach`, `SessionRename`, `EnterEmbeddedTerminal`, `ExitEmbeddedTerminal`,
`ForwardKeyToPty(KeyEvent)`, `PtyScrollUp`, `PtyScrollDown`, `TuneEditBinding`, `TuneApplyProfile`,
`TuneResetBinding`, `TuneEditCcrSlot`. 5-level binding precedence and `Eq + inspectable` enum
invariant are unchanged (D-009).

`AppMode` is extended with `EmbeddedTerminal { session_id: String, prior }` and
`SessionCreation { step, prior }`. `session_id` is a `String` (UUID rendered as String at the
AppMode/IPC boundary — avoids a `uuid` dep in `monocle-core`; per SS-08 session_id canonical ruling).
Compile-time mutual exclusion and VecDeque overlay stack are unchanged.

**Permission prompts in EmbeddedTerminal / SessionCreation mode:** Incoming permission prompts
are NEVER silently queued while the user is in `AppMode::EmbeddedTerminal` or `AppMode::SessionCreation`.
Production-grade behavior: an incoming `PreToolUse` hook MUST immediately raise a status-bar badge
(e.g., `[2 prompts]`) AND an audible bell (`\x07`). The user presses Esc to exit embedded terminal
mode; the overlay presents on the prior AppMode. This guarantee will be formalized as a new BC in
the PRD delta (to be authored by product-owner). A v1B enhancement — having the permission overlay
pre-empt embedded terminal mode without requiring Esc — is a potentially desirable UX upgrade but
requires human ratification before adding to scope; recorded here as an open v1B item (not v1A scope).

### Process Topology (v2.0)

```
User's tmux server (existing)
├── pane: editor (nvim / Zed / VS Code terminal)
│   └── Ctrl-\  ─────────────────────────────────────────────┐
│                                                             │
└── pane: monocle TUI client (connects to daemon)            │
                                                             ▼
monocle daemon (monocle-runtime, long-lived background process)
├── axum HTTP :<os-port>   (hook POST receiver; OS-assigned; port in lock file)
├── UDS socket             (TUI client connection: hook events + PTY bytes + control)
├── Session-host coordination (D-238 model)
│   ├── Session-host A: native detached process (PTY master + child handle + per-session UDS)
│   ├── Session-host B: native detached process
│   └── Session-host N: ...
├── Arc<Broker<Event>>     (fan-out: hook events + PTY bytes to connected TUI clients)
└── EngineModule registry  (ClaudeCodeModule: spawn_recipe() + hook handling)

Claude sessions (owned by session-host processes, coordinated by daemon)
├── Session A: claude --settings /tmp/monocle-hooks-A.json  (launched by monocle)
│   ├── Hook POSTs → POST http://localhost:<port>/hooks/*
│   └── PTY stdout ──► session-host A ──► daemon re-attach ──► UDS PtyOutput msg
└── PTY stdin ◄── UDS KeyInput msg ◄── TUI crossterm KeyEvent

TUI client
├── Receives PtyOutput { session_id, bytes }  ──► vt100::Parser.process(bytes)
├── Renders PseudoTerminal widget (tui-term 0.3.4) from parser.screen()
├── Sends KeyInput { session_id, bytes }  ──► encoded crossterm KeyEvent → PTY bytes
└── Sends ResizePane { session_id, rows, cols }  ──► daemon resizes PTY + parser
```

The exact session-host IPC re-attach protocol (per-session UDS, re-stream vt100 parser state)
is an architecture delta deliverable (Q-8 HIGH). The topology above shows the logical intent;
architect produces the binding spec.

### OQ and SOQ Resolutions — Still Binding

All 11 original open questions and 4 second-order questions from `oq-research.md` (commit
b3c68ca) remain binding. The v2.0 pivot does not reopen them. Key constraints preserved:

| Constraint | Trace |
|---|---|
| Daemon auto-start: hybrid with `MONOCLE_NO_AUTOSTART=1` escape | OQ-01 |
| Hook tmpfile: shared per-runtimeDir, mode `0o600`, atomic-replace | OQ-02 |
| WASM plugin SDK: NOT in v1 (suspended); v1 statically bundles VsddFactoryAdapter | OQ-03 |
| Port binding: OS-assigned + lock-file PID-liveness discovery | OQ-04 |
| Profile picker: sticky-per-project; `Ctrl-P` override | OQ-05 |
| Hook event retention: hybrid RAM ring + async JSONL flush, 100MB × 5 rotation | OQ-06 |
| Cross-host migration: protobuf seams in v1 (zero cost); russh transport suspended | OQ-07 |
| monocle-ipc: UDS-only in v1 | OQ-08 |
| rmcp MCP bridge: omitted in v1 (suspended) | OQ-09 |
| Daemon lock file: `directories::ProjectDirs::runtime_dir()` with fallback chain | OQ-10 |
| MSRV: Phase 1 = Rust 1.88 (RUSTSEC-2026-0009 Path B); v1B may extend; Phase 3 = 1.92 | OQ-11 |
| Lock-file schema: `contract_version: u32` from day one | SOQ-1 |
| Token rotation invariant | SOQ-2 |
| Overlay survival: clear on daemon disconnect | SOQ-3 |
| Permission token enum: see `SS-permissions-phase1.md` | SOQ-4 |

Market-intel open questions OQ-M1/M2/M3 remain resolved as in v1.4.x (no new IPC collision risk
from agent view; `claude-manager` not hook-protocol; 5-endpoint set is canonical). The R-001
Anthropic commoditization risk assessment at <10% probability stands; re-eval triggers (a)–(d)
are unchanged; weekly GitHub Actions monitoring cadence is unchanged.

### Forward-Compatibility Contracts — Preserved

All 6 forward-compatibility contracts from Phase-1 are preserved:
1. `MONOCLE_ABI_VERSION: u32 = 1` exported and exposed via `/status`.
2. Public enums in `monocle-core` carry `#[non_exhaustive]` (except `Phase1Permission` and
   `ClaudeCodeTool` per ADR-0004).
3. `FactoryAdapter` trait defined; `VsddFactoryAdapter` implements it (not wired inline).
4. `monocle-proto` HookEnvelope + 5 event messages with `schema_version = 1` as first field.
5. Every JSONL ring record carries `format_version: u32 = 1` as first key.
6. Auth token format `monocle-v1:<64-char-hex>`; dual-accept per ADR-0005.

The 22 BCs covering these contracts (per BC-INDEX) are preserved. New v1A BCs will be
added by the product-owner in the PRD delta to cover LAUNCH, EMBEDDED PTY, MULTI-SESSION, and
PERSISTENCE. New v1B BCs will cover INTERACTIVE TUNE.

---

## Competitive Positioning

Monocle is positioned as "a better lazyclaude AND a better claude-squad" — collapsing the
session-management gap that neither single tool fills:

- **vs. any-context/lazyclaude:** lazyclaude observes sessions you launched elsewhere; monocle
  launches and owns them. lazyclaude has no embedded terminal pane; monocle streams the live
  session without a window switch. lazyclaude is Go; monocle is Rust with production-grade
  hook-protocol ingestion, DTU clone, and forward-compatibility ABI.
- **vs. claude-squad:** claude-squad launches sessions via tmux multiplexing (external dep;
  fidelity ceiling); monocle uses native portable-pty (no external dep; full Kitty keyboard
  protocol). claude-squad has no TUI management surface — you interact with the raw tmux session;
  monocle provides a full dashboard, permission overlay, profile picker, and event ribbon.
  claude-squad has no hook-protocol depth; monocle has the VecDeque cascaded overlay, diff
  preview, and 5-endpoint hook ingestion.
- **vs. Anthropic agent view (claude agents, v2.1.139, 2026-05-11):** agent view provides a thin
  session list + inline reply inside Claude Code's TUI; no launch ownership, no embedded PTY pane,
  no diff preview, no cascaded permission queue, no customization editing, no workflow plane, no
  multi-harness. Monocle goes deeper on every dimension agent view does not touch. The R-001 risk
  at <10% probability stands (no announced hook-protocol direction, research-preview scope,
  single-harness only); re-eval trigger conditions (a)–(d) unchanged.
- **vs. zellij / tmux:** monocle is not a general-purpose multiplexer; it is a control center
  for AI coding sessions specifically. It borrows the zellij client/server IPC and session
  persistence model as architectural pattern but does not depend on zellij code. It runs inside
  the user's existing tmux session rather than replacing it.

**Competitive differentiators that v1 makes verifiable:**
1. Launch ownership + hook auto-injection — no competitor launches sessions with automatic
   hook wiring.
2. Embedded PTY pane with full keyboard fidelity including Kitty protocol — sessions interactive
   inside the TUI, not just visible.
3. VecDeque cascaded permission overlay — both prompts visible simultaneously, ≤6 keystrokes to
   clear both; no competitor offers this.
4. Factory-awareness plane (FactoryAdapter observe-only) — unique to monocle; no competitor
   surfaces vsdd-factory/STATE.md workflow state in a unified TUI.
5. Multi-harness extensibility substrate (EngineModule trait, FactoryAdapter trait, forward-compat
   ABI) — architecture is ready for CodeMachine and future harnesses; competitors are single-harness.

---

## Open Questions for Architecture Delta

These are genuine cross-component architecture questions that require architect adjudication.
They are NOT deferrals of mechanical questions (CLAUDE.md Rule 6). All originate from vision
§Open Questions.

| ID | Priority | Question | Route |
|----|----------|----------|-------|
| Q-8 | HIGH | PTY-ownership-survival mechanism for graceful daemon-process restart (session-host process model; re-attach protocol over per-session UDS; vt100 parser re-stream; changes to D-235 SessionManager wiring). Default constraint: native, no external multiplexer as primary. See vision §Open Questions §Q-8 for full sub-questions. | architect (must resolve before architecture delta spec finalized) |
| Q-1 | MED | PTY bytes over shared UDS IPC vs. dedicated streaming path per session. embedded-pty-evaluation.md §8 Q4 recommends Option A (shared channel). Pre-v1A-gate benchmark required. | architect → performance-engineer benchmark |
| Q-2 | MED | EngineModule.spawn_recipe() surface vs. SessionManager lifecycle ownership on the EngineModule trait. Recommendation: lifecycle on SessionManager; EngineModule provides recipe only. | architect |
| Q-7 | LOW | tui-term fork posture: confirm whether immediate vendoring before v1A work is preferred over deferred-on-need. Small surface; either is cheap. | architect |

No other architecture questions are open from the v1.4.x lineage. OQ-01 through OQ-M3 are
all resolved.

---

## Reference Gene Source Map (Updated for v2.0)

| Monocle Component | Primary Gene Source | Disposition | Key Artifacts |
|-------------------|--------------------|----|---|
| SessionManager + monocle-session-host (session-host coordination; PTY owned by session-host processes) | claude-squad A.5 (PtyFactory pattern) + zellij session lifecycle | MODEL + ADAPT | claude-squad-pass-8-deep-synthesis.md; zellij-pass-8-final-synthesis.md |
| spawn_recipe() / EngineModule extension | codemachine-cli EngineModule | ADOPT + EXTEND | codemachine-cli-pass-8-final-synthesis.md |
| Embedded PTY widget (tui-term) | zellij PTY/pane architecture | MODEL (portable-pty, not zellij code) | zellij-pass-8-final-synthesis.md; embedded-pty-evaluation.md |
| Session lifecycle state machine | claude-squad instance lifecycle (A.3) | MODEL + ADAPT | claude-squad-pass-8-deep-synthesis.md §instance lifecycle |
| Hook auto-injection on spawn | any-context inject-on-spawn pattern | ADOPT | any-context-lazyclaude-pass-8-final-synthesis-v2.md |
| Action enum + 5-level precedence | lazygit | ADOPT + EXTEND (PTY + launch actions) | lazygit-pass-8-final-synthesis.md §Action enum |
| AppMode state machine + VecDeque overlay | NikiforovAll + lazygit fix | ADOPT + EXTEND (EmbeddedTerminal + SessionCreation) | nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md §AppMode |
| Hook protocol + tmpfile schema | any-context hooks-r1/r2 | ADOPT (already built) | any-context-lazyclaude-pass-8-final-synthesis-v2.md §Hook protocol |
| Broker (bounded pub/sub + drop counter) | any-context broker-r1/r2 | ADOPT + EXTEND (PTY channel) | any-context-lazyclaude-pass-8-final-synthesis-v2.md §Broker |
| Crate workspace split | zellij | ADOPT (already built, extended) | zellij-pass-8-final-synthesis.md §crate layout |
| Worktree isolation pattern | claude-squad | ADOPT + EXTEND (spawn cwd injection) | claude-squad-pass-8-deep-synthesis.md |
| CCR integrate-external | claude-code-router | ADOPT + EXTEND (used in spawn path) | claude-code-router-pass-C-final-synthesis.md |
| FactoryAdapter + VsddFactoryAdapter | vsdd-factory | ADOPT (confirmed observe-only) | vsdd-factory-pass-8-final-synthesis.md |
| 7-parser customization schema | NikiforovAll | ADOPT (v1B) | nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md §parsers |
| Interactive Tune CRUD | NikiforovAll writer/CRUD genes | ADOPT (v1B) | nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md §CRUD |

**Confirmed LEAVE BEHIND (unchanged from v1.1.2):** AutoYes/polling daemon; capture-pane
scraping; tmux as PRIMARY mux; PM/Worker orchestration; zellij-as-library; SSH federation
(suspended); WASM plugin SDK (suspended v1).

---

## §Trace v2.0.0 — Re-Baseline Amendment

**Amendment class:** MAJOR (control-center re-baseline; observe-only scope retired).  
**Traces to:** D-236, D-237, D-238. Vision v2.1 APPROVED by Joshua Magady (2026-06-03).

**What this re-baseline retired from v1.4.x:**

| Retired element | Where it appeared | Why retired |
|-----------------|------------------|-------------|
| "Observe-only for workflow state and session transcripts; action-only for permission prompts and keybinding dispatch" | §What Is This | D-236/D-237: monocle now launches and owns sessions |
| "Claude Code subprocesses are unmodified beyond pointing their hook scripts at the daemon's lock-file-discovered port" | §Constraints Process Topology | Superseded: monocle now spawns sessions with hook auto-injection |
| Phase 1–4 roadmap (Runtime Core, Static Plane, Workflow Plane, Cross-plane + Federation) | §Scope | Replaced by v1A/v1B wave plan |
| Phase 2/3/4 success criteria | §Phase 2/3/4 Exit Criteria | Replaced by v1B success criteria; Phase 3 WASM/federation suspended |
| "Does NOT replace the terminal multiplexer" (blanket) | §Out of Scope | Partially revised: monocle IS a mux for AI sessions; does NOT replace user's general-purpose mux |
| R-001 observe-only competitive framing | §Competitive Positioning | Replaced with control-center differentiation framing |
| v1.4.x §Trace v1.4.24 through §Trace v1.4.34 | §Trace sections | Archived to git history; not reproduced here (normative decisions from this lineage are summarized in §Preserved Substrate and §OQ/SOQ Resolutions) |

**What this re-baseline preserved (normative, unchanged):**

- All OQ-01 through OQ-M3 resolutions
- All 6 forward-compatibility contracts and their 22 BCs
- Phase-1 hook ingestion, permission overlay, event ribbon, profile picker, workflow plane,
  daemon lifecycle, auth token format, JSONL ring, DTU clone, ABI version const, forward-compat
  enum policy, FactoryAdapter trait, proto schema
- R-001 risk assessment at <10% probability and re-eval trigger conditions (a)–(d)
- Weekly GitHub Actions R-001 monitoring cadence
- All ADRs (0001–0005)
- SS-deps-pin-manifest.md version pins and MSRV policy
- CLAUDE.md CANONICAL PRODUCTION-GRADE PRINCIPLE and CORRECT-AGENT-ROUTING

**Input hash:** TBD — state-manager runs `compute-input-hash --update` after this commit to populate
the `input-hash` frontmatter field per drift detection protocol.

**SE-16d monotonicity PASS:** 2026-06-03T22:00:00Z > prior 2026-05-30 (v1.4.34). PASS.

---

## §Trace v2.0.1 — Consistency Propagation Amendment

**Amendment class:** MINOR (consistency propagation; no scope change; status remains draft).  
**Traces to:** ADR-0009, SS-session-manager.md (SS-08), SS-deps-pin-manifest-v2-delta.md.  
**Timestamp:** 2026-06-03T23:30:00Z.

**What this amendment corrected (mechanical alignment to already-approved architecture):**

| Item | Where | Correction |
|------|-------|-----------|
| CRIT-2: `portable-pty` crate location | §Tech Direction table | `monocle-runtime` → `monocle-session-host` |
| CRIT-2: `vt100` crate location | §Tech Direction table | `monocle-runtime`, `monocle-tui` → `monocle-session-host`, `monocle-tui` |
| CRIT-3: trait name | §Success Criteria (Input fidelity row) | `MockPtySpawner` → `MockSessionHostSpawner` |
| CRIT-3: trait name | §Crate Workspace Layout (monocle-test-harness) | `MockPtySpawner` / `PtySpawner` → `MockSessionHostSpawner` / `SessionHostSpawner` |
| IMP-1: FactoryAdapter extraction source | §Crate Workspace Layout (monocle-workflow) | "exists in monocle-runtime today" → "exists in monocle-core today" |
| IMP-1: session-host crate added | §Crate Workspace Layout | `monocle-session-host` new crate bullet added for v1A |
| IMP-1: SessionManager description | §Crate Workspace Layout (monocle-runtime) | "owns PTY masters" → "coordinator role; does NOT own PTY masters" |
| AppMode session_id type | §Action Enum and AppMode | `session_id` clarified as `String` per SS-08 session_id canonical ruling |
| SUG-3: permission badge+bell guarantee | §Action Enum and AppMode | Added permission behavior guarantee + v1B open item note |

**SE-16d monotonicity PASS:** 2026-06-03T23:30:00Z > prior 2026-06-03T22:00:00Z (v2.0.0). PASS.
