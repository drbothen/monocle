---
document_type: product-brief
level: L1
version: "1.1"
status: draft
producer: product-owner
phase: pre-phase-1-brief
timestamp: 2026-05-12T10:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md
  - /Users/jmagady/Dev/monocle/.factory/semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md
  - /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/lazygit/lazygit-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md
input-hash: "[live-state]"
traces_to: "factory-artifacts 2737bfd (vision-synthesis approved); 2c2b676 (8-repo full ingest)"
project: monocle
---

# Product Brief: Monocle

## What Is This?

Monocle is a Rust TUI that gives developers one `Ctrl-\` popup over every
AI coding harness session they are running — across projects, across harnesses
(Claude Code, CodeMachine, future), and across hosts. It surfaces five
information planes: live session roster with token burn and cost (Runtime),
active customizations per session (Static), workflow pipeline state for
factory-pattern projects (Workflow), per-harness profiles (Harness), and a
lazygit-style keybinding dispatch layer (TUI philosophy). Monocle is
observe-only for workflow state and session transcripts; it owns the action
layer only for permission prompts and keybinding dispatch — the two places where
context-switching today costs the developer real time and real session stalls.

Per vision §Vision Statement: "One TUI lens over every Claude-class session
you're running, every customization that shapes them, and every workflow driving
them — across multiple harnesses and federated across hosts."

## Revision History

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-12 | product-owner (direct draft from approved vision) | Initial brief — committed at factory-artifacts e8e8af1 |
| 1.1 | 2026-05-12 | product-owner (version validation revision) | Updated all crate version pins to crates.io 2026-05-12 reality; added RUSTSEC notes; refreshed wasmi/wasmtime rationale; added 11 new version pins for previously-unpinned vision tech stack crates; added OQ-11 MSRV |

## Who Is It For?

| Persona | Pain Point | Current Workaround |
|---------|-----------|-------------------|
| **Multi-session Claude Code developer** — runs 2-4 Claude Code sessions in parallel across worktrees or projects | Permission prompts from session B stall while the developer is focused on session A's window; must `Ctrl-b n` to find the right pane, read inline text, respond, switch back | Context-switch to correct tmux window; miss prompts; restart stalled sessions |
| **Factory-pattern operator** — runs vsdd-factory-style pipelines where each phase advances through a STATE.md; needs situational awareness without leaving the editor | Must `cat .factory/STATE.md`, `tree .factory/`, and mentally track which phase each session is in; blocking issues invisible until a session stalls | Manual file reads; `grep` for blocking issues; context-switch to read pipeline output |
| **Multi-harness operator** (v4 target, design must support) — runs Claude Code sessions on one task and CodeMachine sessions on another simultaneously | No unified view of cost or session health across harnesses; different UIs, different status indicators | Open separate TUI instances per harness; no aggregate cost tracking |

The killer scenario that motivates the v1 scope is the **multi-session developer**:
three sessions running (monocle project, blog, api-svc), two concurrent permission
prompts from different sessions, developer in nvim. Per vision §End-to-End Killer
Scenario: 4 keystrokes (`Ctrl-\`, `2`, `1`, `Ctrl-\`) resolves both prompts with
zero context switches vs. the current 6+ keystrokes + 2 window switches + risk of
session timeout.

## Scope

### In Scope

The scope below maps to the Phase Plan in vision §Phase Plan. Phase 1 is the
v1 delivery contract. Phases 2-4 are roadmap entries the architecture must
accommodate without breaking Phase 1 ABI.

**Phase 1 — Runtime Core (v1 delivery contract)**

- `monocle daemon start/stop`: long-lived background process that survives terminal
  closes; binds axum HTTP on `127.0.0.1:2748` (hook ingestion), optionally
  `127.0.0.1:2749` (rmcp MCP bridge, stub); writes daemon lock file with
  `{port, token}` at mode `0o600`; daemon auto-starts on first TUI launch if not
  already running (architect must decide — see Open Questions OQ-01)
- Hook ingestion endpoints: `POST /hooks/pre-tool-use`, `POST /hooks/post-tool-use`,
  `POST /hooks/stop`, `POST /hooks/permission` — schema byte-compatible with
  Claude Code's tmpfile hook protocol (verified against
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md`
  canonical schema); auth via `X-Claude-Code-Ide-Authorization` header
- `ClaudeCodeModule`: built-in `EngineModule` implementation; detects Claude Code
  processes via PID walk; enriches with token counts, cost, phase tag from hook
  events; handles hook events and produces `EnrichedSession`
- Sessions panel (TUI): live session roster showing harness icon, project name,
  phase tag, token count, cost, uptime; `/` filter (nucleo-matcher); `Enter`
  fullscreen
- Permission prompt overlay: cascaded `VecDeque<PromptModal>` — both prompts visible
  simultaneously; diff preview via `similar 3`; Accept-once / Accept-always /
  Reject keybindings; `[t]` trace-to-source stub (full trace lands in Phase 2);
  overlay survives `Ctrl-\` hide/show cycle without dropping queued prompts
- Event ribbon panel: rolling log of hook events (PreToolUse, PostToolUse, Stop,
  Permission) with session ID and latency
- `monocle-config`: reads/writes `~/.monocle/config.json` (via `tempfile::persist`
  for atomic writes); harness profile schema version 1; CCR path field; binding
  overrides stub
- Tokio mpsc **bounded** event bus with drop counter surfaced in status bar;
  no unbounded channels (triple-confirmed anti-pattern from broker-r1 §3)
- `monocle-ipc`: Unix domain socket IPC between TUI client and daemon; shared-memory
  ring buffer for high-frequency hook event stream
- macOS + Linux build targets (darwin/linux × amd64/arm64); CI matrix on GitHub Actions

**Phase 2 — Static Plane (roadmap)**

- `monocle-static` crate: reads CLAUDE.md, settings.json permission blocks, hook
  scripts, keybindings.json for the session in focus
- Customizations panel (TUI): 7 customization types from nikiforovall gene set
  (slash commands, subagents, skills, memory files, MCP servers, hooks, LSP servers);
  filter All / by type; trigger-trace `[t]` from permission prompt overlay to
  defining settings.json line
- Full AppMode state machine with FocusSnapshot enum (compile-time mutual exclusion);
  5-level binding precedence (SearchPrompt > UserCustomCommand > PerContext >
  Global > Builtin); telescope help overlay

**Phase 3 — Workflow Plane (roadmap)**

- `monocle-workflow` crate: `FactoryAdapter` trait; `VsddFactoryAdapter` (reads
  `.factory/STATE.md`, detects `document_type: pipeline-state`, parses phase/status/
  blocking-issues/convergence); `notify 8` watcher for live updates; multi-repo
  signal (`.factory-project/` directory)
- Workflow panel (TUI): phase, status, awaiting, blocking issues, cycle for focused
  session's project
- `monocle-plugin-sdk` crate: WASM ABI (`wasmtime 44`) for third-party
  `EngineModule` + `FactoryAdapter` implementations; loaded from `~/.monocle/plugins/`

**Phase 4 — Cross-plane + Multi-harness + Federation (roadmap)**

- `CodeMachineModule`: second built-in `EngineModule`
- `monocle-proto` crate: prost-generated protobuf wire format for cross-host events
- `russh 0.60` federation tunnel: TUI on host A shows sessions from host B
- OTel cost/token panel with aggregate across harnesses
- CCR integration: detect on PATH, write per-session JSON, set `ANTHROPIC_BASE_URL`
- rmcp MCP bridge (optional, port 2749): session query, prompt injection for tooling

### Out of Scope

Per vision §Explicit Non-Goals (these are hard boundaries, not deferred features):

- **Does NOT execute workflows** — monocle never writes STATE.md, never triggers
  factory phases, never dispatches agents; workflow panel is read-only observation
- **Does NOT write STATE.md** — the `VsddFactoryAdapter` reads STATE.md; monocle
  never mutates it
- **Does NOT route LLM API requests** — CCR integration is detect-on-PATH +
  config-write only; monocle does not proxy or modify LLM traffic (integrate-external,
  per D-010)
- **Does NOT replace the terminal multiplexer** — monocle runs inside tmux; it is
  not a multiplexer; zellij's multiplexer internals are a Leave-behind gene
- **Does NOT include PM/Worker multi-agent orchestration** — explicitly excluded
  by D-002; the human is always the coordinator
- **Does NOT own session transcripts** — hook events are ephemeral ingestion signals;
  full transcript storage belongs to each harness's own persistence layer
- **Does NOT build its own LLM provider abstraction** — CCR is the external router
  (D-010); monocle integrates by detecting it

## Success Criteria

v1 ships (Phase 1 complete) when ALL of the following pass:

| Outcome | Metric | Target |
|---------|--------|--------|
| Session management in popup | User can manage 3+ concurrent Claude Code sessions without leaving the editor pane | Killer scenario resolves in ≤6 keystrokes (per vision §End-to-End Killer Scenario target: 4) |
| Permission prompt latency | Permission prompt appears as overlay with diff preview after hook fires | ≤100ms from hook POST receipt to TUI overlay render on localhost |
| Hook protocol parity | Hook injection byte-compatible with Claude Code's schema | Fixture-based parity test passes against schema in any-context hooks-r1 canonical matrix (PreToolUse/PostToolUse/Stop/Permission endpoints, `X-Claude-Code-Ide-Authorization` header) |
| Customization rendering | All 7 customization types render in Static plane on filter "All" | Zero missing types when pointed at a claude-code project with all 7 type examples |
| Factory pattern detection | vsdd-factory project detected and workflow panel populated | Detection succeeds on monocle's own `.factory/` (self-referential integration test) |
| Build matrix | Builds and tests pass on macOS and Linux | CI green on darwin/linux × amd64/arm64 |
| Drop counter active | Bounded event bus with visible drop counter | No unbounded channel in codebase; drop counter renders in status bar under synthetic high-frequency load (1000 events/sec) |

## Constraints & Integration Points

**Tech stack is fixed by vision §Tech Stack** — the architect inherits these
picks as Phase 1 constraints; they are not up for re-selection in Phase 1:

- TUI: `ratatui 0.30` + `crossterm 0.29`
- Async: `tokio 1.52` (full)
- HTTP: `axum 0.8` (Cargo.toml pin: `^0.8.9`)
- IPC: `interprocess 2.4` (Unix domain socket)
- Serialization: `prost 0.14` for cross-host; `serde_json` + `serde_yaml_ng 0.10`
  (NOT `serde_yaml 0.8` — unmaintained, alias-bomb CVE); also pin `bytes` directly
  in workspace to avoid prost 0.14 transitive RUSTSEC-2026-0007 (see Supply Chain section)
- WASM: `wasmtime 44` (NOT wasmi — see rationale below)
- Fuzzy: `nucleo 0.5` (NOTE: upstream dormant since 2024-04-02; flag for Phase 2
  re-evaluation against `frizbee 0.9` / `neo_frizbee 0.10` / `nucleo-picker 0.11`
  if active maintenance becomes a release constraint)
- Diff: `similar 3`
- Temp write: `tempfile 3` via `tempfile::persist` (no naked `write_text` calls —
  triple-confirmed anti-pattern from nikiforovall atomic-write gap findings)
- Config dirs: `directories 6` (XDG-compliant)
- File watch: `notify 8`
- SSH tunnel (Phase 4): `russh 0.60`
- CLI parsing: `clap 4.6`
- Markdown rendering: `pulldown-cmark 0.13`
- Clipboard: `arboard 3`
- Tracing: `tracing 0.1`
- Semver parsing: `semver 1`
- Error handling: `thiserror 2` (NOTE: 2.x major — do NOT pin to 1.x), `anyhow 1`
- HTTP client: `reqwest 0.13` (NOTE: 0.13.x — do NOT pin to 0.11 or 0.12, both stale)
- MCP bridge: `rmcp 1.6` (Anthropic-canonical via modelcontextprotocol/rust-sdk org;
  crates.io owner alexhancock@Anthropic confirmed)

**wasmtime vs wasmi rationale**: `wasmtime 44` is preferred over `wasmi`. wasmi 1.0
is now mature with WASI support, so the historical "WASI gap" rationale no longer
applies. Monocle prefers wasmtime for two reasons: (1) JIT throughput for factory
adapters that may execute non-trivial pipeline logic, and (2) actively-maintained
security posture — wasmtime's Bytecode Alliance publishes security advisories on a
tight cadence (multiple advisories in 2026 alone) and ships patches promptly. wasmi
remains a future fallback if binary-size pressure becomes a release constraint.

**Crate workspace layout is fixed by vision §Workspace Layout and D-008:**
`monocle-core` (zero-dependency pure types), `monocle-runtime`, `monocle-tui`,
`monocle-static`, `monocle-workflow`, `monocle-plugin-sdk`, `monocle-ipc`,
`monocle-config`, `monocle-proto`, `monocle-fuzz`, `monocle-test-harness`,
`monocle` (binary). No crate outside the binary may depend on the binary crate.

**Action enum dispatch model is non-negotiable** per vision §Key Abstractions and D-009:
5-level precedence (SearchPrompt > UserCustomCommand > PerContext > Global >
Builtin); enum variants (not closures) keep bindings `Eq + inspectable` for
telescope help overlay. The dispatcher walks the stack in order and stops at the
first match.

**AppMode state machine is non-negotiable** per vision §Key Abstractions: compile-time
mutual exclusion (not `bag-of-Option` fields); `VecDeque<PromptModal>` overlay
stack (not single-popup — fixes lazygit's drop-on-concurrent anti-pattern); state
transitions are pure functions in `monocle-core`.

**Process topology**: monocle uses a separate tmux server (`-L monocle`) to host
the TUI client as a floating popup over the user's existing tmux session. Daemon
is a long-lived background process. Hook POSTs are the ingestion boundary; Claude
Code subprocesses are unmodified beyond pointing their hook scripts at
`localhost:2748`.

**CCR is integrate-external** (D-010): detect on PATH, write per-session JSON,
set `ANTHROPIC_BASE_URL`. No CCR API changes required or expected.

**Anti-patterns to enforce at code review (triple-confirmed across 8 gene sources):**

- No `Command::new("sh").arg("-c").arg(template_string)` or equivalent shell=True
  pattern — use `Command::new(binary).args([...])` arg-array form
- No naked `std::fs::write` / `write_text` for config files — use `tempfile::persist`
- No `tokio::sync::mpsc::unbounded_channel` — use bounded channel + drop counter
- No package-level mutable globals for theme/config — use `Arc<RwLock<Theme>>`
- No single `Option<PromptModal>` field for the overlay — use `VecDeque<PromptModal>`
  to support concurrent prompts without drop

## Supply Chain and RUSTSEC Notes

Validation performed 2026-05-12 against crates.io API + RUSTSEC advisory DB (Tavily + Perplexity + direct crates.io fetch). Findings the architect must respect when finalizing Cargo.toml:

### Advisories on upstream versions monocle must avoid

- `wasmtime` older majors (pre-44) carry RUSTSEC-2026-0114, RUSTSEC-2026-0095, RUSTSEC-2026-0096, RUSTSEC-2026-0006, RUSTSEC-2026-0020 (guest-controlled resource exhaustion in WASI implementations), and others. Pin to `wasmtime = "44"` (latest 44.0.1) and bind future patches via cargo update on the 44.x line.
- `russh` 0.45..0.59 transitively pulls `rsa = "0.10.0-rc.12"` which is affected by RUSTSEC-2023-0071 (timing-attack on RSA private-key operations). Pin to `russh = "0.60"` (0.60.2 latest) which moved off the affected rsa pre-release.
- `prost` 0.14 has a transitive `bytes` advisory RUSTSEC-2026-0007 affecting older `bytes` versions. Pin `bytes` directly in workspace dependencies to force a patched version (e.g. `bytes = "1.10"` or whatever the patched line is at audit time).
- `tokio` 1.x has multiple historical advisories on older minors (RUSTSEC-2025-0023, RUSTSEC-2023-0005, RUSTSEC-2023-0001, RUSTSEC-2021-0124, RUSTSEC-2021-0072). Pin to current 1.52 line to ensure all are remediated.
- `serde_yaml` 0.8 is unmaintained with alias-bomb CVE; `serde_yml` (a different fork) was archived per RUSTSEC-2025-0068. The brief's choice of `serde_yaml_ng` 0.10 (maintained fork) is correct and survives this audit.

### Re-audit cadence

The architect must enforce a `cargo audit` run in CI on every PR, plus a weekly scheduled `cargo audit --json` against the latest RUSTSEC DB. New advisories on pinned versions block merge until either (a) the version is updated to a patched release, or (b) a documented justification with mitigations is filed under `.factory/specs/risk-acceptance/`.

### Crates flagged for Phase 2 re-evaluation

- `nucleo 0.5.0` — upstream dormant since 2024-04-02 (helix-editor team's focus has shifted). Functionality is intact and adequate for Phase 1 fuzzy filtering. If maintenance becomes a release constraint before Phase 2, evaluate `frizbee 0.9` or `neo_frizbee 0.10` (actively maintained alternatives with SIMD-accelerated matching) or `nucleo-picker 0.11` (TUI-focused fork).

## Open Questions for Architect

The following questions must be resolved in Phase 1 spec crystallization before
architecture can be finalized:

| ID | Question | Options | Decision Owner |
|----|----------|---------|----------------|
| OQ-01 | Does `monocle daemon start` auto-run on first TUI launch, or require explicit invocation? | (a) Auto-start: TUI checks daemon socket, spawns daemon if absent; (b) Explicit: user must run `monocle daemon start` first; fail-fast with helpful error | Architect (UX implication: option a hides daemon lifecycle; option b makes it visible) |
| OQ-02 | Hook tmpfile per-session or shared per-runtimeDir? | any-context synthesis says shared per-runtimeDir (`~/.claude/ide/<port>.lock` pattern); verify this is the right model for monocle's hook injection, not just lazyclaude's | Architect + review hooks-r1 canonical schema |
| OQ-03 | Does v1 ship the WASM `monocle-plugin-sdk` crate or bundle `VsddFactoryAdapter` statically? | (a) v1 ships WASM ABI so third-party adapters work from day one; (b) v1 bundles statically, WASM SDK ships in Phase 3 | Architect (binary size and wasmtime startup cost implication) |
| OQ-04 | Where does the daemon's HTTP server bind — `127.0.0.1:2748` (fixed) or `127.0.0.1:0` with port written to lock file? | Fixed port (simpler, conflicts possible); OS-assigned port written to lock file (any-context pattern, restart-resilient) | Architect (hook injection must know port; OS-assigned requires lock-file discovery) |
| OQ-05 | Profile picker on session create vs sticky-per-project? | (a) Picker shown when creating a new session (interactive prompt); (b) Profile stored in per-project config, sticky across restarts | Architect + UX designer |
| OQ-06 | Hook event timeline retention: in-memory ring buffer or persisted JSONL? | In-memory ring is simpler (Phase 1); persisted JSONL enables replay and holdout evaluation; any-context broker has no persistence | Architect (test-harness implication: persisted JSONL enables deterministic fixture replay) |
| OQ-07 | Cross-host session migration scope: v1 or v4? | Vision §Phase Plan puts federation in Phase 4; confirm v1 architecture does not preclude it (russh + protobuf stubs acceptable in v1 if zero runtime cost) | Architect |
| OQ-08 | monocle-ipc: Unix domain socket only, or Unix domain socket + shared-memory ring buffer both shipped in v1? | Shared-memory ring buffer adds zero-copy throughput for high-frequency events; vision lists both; shared-mem adds complexity | Architect (vision §Tech Stack lists interprocess 2.4 for both; confirm v1 scope) |
| OQ-09 | rmcp MCP bridge (port 2749): stub with no-op handlers in v1, or omit entirely? | Including stub prevents port allocation surprises in Phase 4; omitting keeps v1 surface small | Architect |
| OQ-10 | Daemon lock file location: `~/.monocle/daemon.json` (like any-context's `<runtimeDir>/daemon.json`) or `$XDG_RUNTIME_DIR/monocle/daemon.json`? | XDG-compliant is cleaner on Linux; `~/.monocle/` is simpler and macOS-compatible | Architect + directories 6 XDG resolver |
| OQ-11 | MSRV target | Pin a specific minimum supported Rust version (e.g. 1.75 or 1.78) in the workspace `rust-version` field to align with the dependency stack; tokio 1.52, wasmtime 44, ratatui 0.30 all have MSRV constraints that must be respected | Architect (cross-reference each pinned crate's MSRV and pick the highest) |

> Five judgment calls flagged by orchestrator awaiting human red-line: JC-1 (static 7-type Phase 1 vs 2), JC-2 (PostToolUse endpoint), JC-3 (port 2748 fixed vs OS-assigned), EX-1 (workspace 11→13 crates), EX-2 (SessionStart/UserPromptSubmit hooks omitted). Architect should resolve OQ-01..OQ-11 with these judgment calls held constant until human red-line lands.

## Overflow Context

### Competitive Positioning

Monocle is not a replacement for lazygit, lazyclaude (either variant), claude-squad,
or CCR. It is the **session management and permission-prompt dispatch layer that none
of them provide**. The closest prior art:

- `any-context/lazyclaude`: Go TUI for Claude Code sessions; PM/Worker orchestration;
  hook protocol via `~/.claude/ide/<port>.lock`. Gene source for Runtime plane.
  Monocle ports the session management and hook ingestion, drops the PM/Worker
  persona, adds multi-harness and WASM plugin extensibility.
- `NikiforovAll/lazyclaude`: Python Textual TUI for customization exploration.
  Gene source for Static plane. Monocle ports the 7-parser canonical schema and
  AppMode state machine to Rust; drops the Python dependency entirely.
- `claude-squad`: Session isolation via worktrees; snapshot/fork concurrency; no
  orchestration layer (human is coordinator per D-011). Gene source for worktree
  isolation pattern in Harness plane.
- `claude-code-router`: LLM request router via HTTP reverse proxy. Integrated
  externally (D-010); monocle detects CCR on PATH and writes per-session config.

### Decisions Log Cross-Reference

All decisions that constrain this brief are logged in STATE.md §Decisions Log:
D-001 through D-012. The canonical vision approved by human is D-012.

### Phase Plan Rationale

Phase 1 ships the daemon + hook ingestion + sessions panel. This is the minimum
viable product for the killer scenario — permission prompt dispatch without
context-switching. Phase 2 adds the customization plane (trigger-trace) which
enriches the permission prompt overlay with "why did this prompt appear" context.
Phase 3 adds workflow awareness which is the factory-operator persona's core need.
Phase 4 adds multi-harness federation which serves the future multi-harness operator
persona. The ABI between phases must be stable: the `EngineModule` and
`FactoryAdapter` traits defined in Phase 1 must be forward-compatible with Phase 4
additions. No breaking changes to these traits between phases.

### Reference Gene Source Map

| Monocle Component | Primary Gene Source | Key Artifacts |
|-------------------|--------------------|-|
| EngineModule trait | codemachine-cli | pass-8-final-synthesis.md |
| Action enum + 5-level precedence | lazygit (port) | pass-8-final-synthesis.md §Action enum |
| AppMode state machine + VecDeque overlay | NikiforovAll AppMode + lazygit fix | nikiforovall pass-8-final-synthesis-v2.md §AppMode |
| Hook protocol + tmpfile schema | any-context hooks-r1/r2 | any-context pass-8-final-synthesis-v2.md §Hook protocol |
| Broker (bounded pub/sub + drop counter) | any-context broker-r1/r2 | any-context pass-8-final-synthesis-v2.md §Broker |
| Crate workspace split | zellij | zellij pass-8-final-synthesis.md §crate layout |
| Worktree isolation pattern | claude-squad | claude-squad pass-8-deep-synthesis.md |
| CCR integrate-external | claude-code-router | claude-code-router pass-C-final-synthesis.md |
| FactoryAdapter + VsddFactoryAdapter | vsdd-factory | vsdd-factory pass-8-final-synthesis.md |
| 7-parser customization schema | NikiforovAll services/parsers/ | nikiforovall pass-8-final-synthesis-v2.md §parsers |
| WASM plugin SDK ABI | zellij-tile model | zellij pass-8-final-synthesis.md §plugin |
