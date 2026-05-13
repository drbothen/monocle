---
document_type: product-brief
level: L1
version: "1.4.4"
status: draft
producer: product-owner
phase: pre-phase-1-brief
timestamp: 2026-05-12T12:00:00Z
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
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
input-hash: "[live-state]"
traces_to: "factory-artifacts 2737bfd (vision-synthesis approved); 2c2b676 (8-repo full ingest); b3c68ca (OQ research)"
project: monocle
supplements:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/tech-debt-register.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
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
| 1.2 | 2026-05-12 | product-owner (Option A bloat remediation + OQ/SOQ/JC decisions) | Trimmed core to ~200 lines; moved version manifest + RUSTSEC + ADR + conventions to architecture stubs; applied 11 OQ defaults + 4 SOQs + JC-1/2/3 + EX-1/2 resolutions; full traceability preserved |
| 1.3 | 2026-05-12 | product-owner (competitive positioning revision + OQ-M1/OQ-M3) | Competitive Positioning revised to acknowledge Anthropic's `claude agents` (agent view, v2.1.139, shipped 2026-05-11). Repositioned monocle's differentiation on mechanism and depth (hook-protocol ingestion, VecDeque overlay, diff preview, trigger-trace, workflow plane, multi-harness, external overlay) rather than exclusivity over the session-list surface. R-001 acceptance stated explicitly. Added OQ-M1 (agent-view IPC coexistence) and OQ-M3 (`PermissionRequest` as 6th endpoint) to the Open Questions table as `pending architect review`. No scope changes. Resolves B-1 from `.factory/plans/brief-validation-v2.md`. |
| 1.4 | 2026-05-12 | product-owner (production-grade defect fixes per adversary re-audit 0bd4ba9) | CRITICAL production-grade defect fixes per adversary re-audit (commit 0bd4ba9). Crate count typo 13→12. OQ-M1/M2/M3 resolved in-scope (no longer Pending architect review): OQ-M1 = no agent-view IPC collision; OQ-M2 = claude-manager not hook-protocol; OQ-M3 = stay at 5 endpoints via JC-2 parity. OQ-M2 row added to table (was absent in v1.3). F-07/F-08 citation parentheticals added. R-001 mitigation reframe HOLD pending human Q-B confirmation (v1.4 shipped with HOLD marker in place). No scope changes. |
| 1.4.1 | 2026-05-12 | product-owner (R-001 probability finalized per human Q-B response) | R-001 risk assessment finalized at <10% probability per human Q-B response. Removed the elaborate mitigation framing (was 'ship Phase 1 fast' in v1.3, became HOLD in v1.4 pending human answer). R-001 is now noted as informational background only — at <10%, the production-grade depth monocle is already shipping IS the response; no separate mitigation scaffolding required. Competitive Positioning section simplified to 3-4 sentences replacing the HOLD block. No scope changes. No other content changes. |
| 1.4.2 | 2026-05-12 | product-owner (Rule 1 violation fix per validate-brief v4) | §Phase Plan Rationale — replaced 'minimum viable product' phrase (Rule 1 violation per CLAUDE.md §Canonical Principle) with production-grade phrasing. Substantive meaning unchanged. Resolves the single blocker from validate-brief v4 (commit 38b8e8f). |
| 1.4.3 | 2026-05-12 | product-owner (adversary findings e2c224b: F-NEW-04, R-001 re-eval, F-NEW-03, F-NEW-05/06/09) | F-NEW-04 CRITICAL: hook ingestion timeout budget added to Success Criteria (300ms PreToolUse/Stop/SessionStart/UserPromptSubmit, 2000ms Notification per BC-HOOK-022); R-001 re-eval trigger paragraph added (4 conditions matching ADR-0002 pattern; <10% probability stands until any condition materializes); F-NEW-03 CRITICAL: permission token enum reference updated; brief no longer claims 17 zellij-borrowed variants for Phase 1; points at architect-produced SS-permissions-phase1.md canonical artifact; F-NEW-05/06/09 IMPORTANT: hook receiver hardening note added to Scope (body size limit, /healthz, /status, graceful shutdown). No scope removals; all additions are production-grade tightening, not new features. |
| 1.4.4 | 2026-05-12 | product-owner (architect-surfaced follow-on from round 5 fix burst) | Body-size limit (256 KiB) added to Success Criteria as a measurable Phase 1 acceptance criterion, cross-referencing BC-DAEMON-003 in `SS-daemon-lifecycle.md`. Resolves the architect-surfaced follow-on from round 5 fix burst — v1.4.3 added the hardening sub-bullet to Scope but did not promote the limit to a measurable Success Criterion. No new scope; just promotes existing scope item to measurable criterion. |

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
  closes; binds axum HTTP on OS-assigned port written to lock file (OQ-04/JC-3
  closed); writes daemon lock file with `{port, token, contract_version}` at mode
  `0o600` (SOQ-1); daemon auto-starts on first TUI launch with `MONOCLE_NO_AUTOSTART=1`
  escape hatch for CI/power users (OQ-01 hybrid)
- Daemon lock-file path: `directories::ProjectDirs::runtime_dir()` with
  state_dir → data_dir → `~/.monocle` fallback chain (OQ-10); token rotation invariant:
  bind socket + write lock-file + write token THEN hooks-settings reads token (SOQ-2)
- Hook ingestion endpoints (5 total, EX-2 resolution): `POST /hooks/pre-tool-use`,
  `POST /hooks/notification`, `POST /hooks/stop`, `POST /hooks/session-start`,
  `POST /hooks/prompt-submit`; schema byte-compatible with Claude Code's tmpfile hook
  protocol; auth via `X-Claude-Code-Ide-Authorization` header; `PostToolUse` omitted
  per JC-2 (Claude Code gene-source parity BC-HOOK-007). Note: The vision document's
  §Process Topology diagram pre-dates JC-2 / EX-2 endpoint closures and depicts an
  illustrative endpoint set (with PostToolUse / PermissionPrompt); the canonical Phase 1
  endpoint set is the 5 endpoints listed above and the vision diagram is non-authoritative
  for endpoint enumeration.
  - Hook receiver hardening: body size limit ≤256KiB (RFC 7230 §3.3.2 compliant; reject
    with HTTP 413 Payload Too Large), `/healthz` liveness endpoint, `/status` daemon-state
    query endpoint, graceful shutdown protocol on SIGTERM/SIGINT (drain in-flight requests,
    flush JSONL ring per OQ-06, close UDS, persist lock file shutdown marker). See
    `.factory/specs/architecture/SS-conventions-anti-patterns.md` and the architect's
    daemon-lifecycle additions for the full BC list.
- Hook tmpfile: shared per-runtimeDir, mode `0o600`, atomic-replace (OQ-02)
- `ClaudeCodeModule`: built-in `EngineModule` implementation; detects Claude Code
  processes via PID walk; enriches with token counts, cost, phase tag from hook
  events; handles hook events and produces `EnrichedSession`
- Sessions panel (TUI): live session roster showing harness icon, project name,
  phase tag, token count, cost, uptime; `/` filter (nucleo-matcher); `Enter`
  fullscreen
- Permission prompt overlay: cascaded `VecDeque<PromptModal>` — both prompts visible
  simultaneously; diff preview via `similar 3`; Accept-once / Accept-always /
  Reject keybindings; `[t]` trace-to-source stub; overlay clears on daemon disconnect
  (SOQ-3); overlay survives `Ctrl-\` hide/show cycle without dropping queued prompts
- Profile picker: sticky-per-project with `Ctrl-P` picker override (OQ-05; Phase 1
  user-test target — MEDIUM confidence)
- Event ribbon panel: rolling log of hook events (PreToolUse, Notification, Stop,
  SessionStart, UserPromptSubmit) with session ID and latency; hybrid RAM ring +
  async JSONL flush, 100MB × 5 rotation (OQ-06)
- `monocle-config`: reads/writes `~/.monocle/config.json` (via `tempfile::persist`
  for atomic writes); harness profile schema version 1; CCR path field; binding
  overrides stub
- Tokio mpsc **bounded** event bus with drop counter surfaced in status bar;
  no unbounded channels (triple-confirmed anti-pattern from broker-r1 §3)
- `monocle-ipc`: Unix domain socket IPC between TUI client and daemon; UDS-only
  in v1 — shared-memory ring deferred to Phase 4 transport variant (OQ-08)
- `monocle-proto`: prost protobuf seam in monocle-core — zero runtime cost in v1,
  enables cross-host events in Phase 4 (OQ-07)
- Permission token enum: see `.factory/specs/architecture/SS-permissions-phase1.md`
  (architect-produced canonical artifact) — small Phase-1-purpose enum derived from
  Claude Code hook permission semantics (allow/deny/ask-user decisions for the 5
  Phase 1 hook endpoint types). The zellij-style 17-variant WASM plugin permission
  enum is Phase 3 scope alongside the wasmtime plugin SDK; not in Phase 1.
  Dispatcher no-op until Phase 3 (SOQ-4); `VsddFactoryAdapter` statically bundled
  in v1 — WASM plugin SDK ships Phase 3, not v1 (OQ-03)
- macOS + Linux build targets (darwin/linux × amd64/arm64); CI matrix on GitHub
  Actions; MSRV Rust 1.86 (ratatui floor, OQ-11)

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

- `monocle-workflow` crate: `FactoryAdapter` trait; `VsddFactoryAdapter` promoted
  from static bundle to WASM-loadable; `notify 8` watcher for live updates
- Workflow panel (TUI): phase, status, awaiting, blocking issues, cycle for focused
  session's project
- `monocle-plugin-sdk` crate: WASM ABI (`wasmtime 44`) for third-party
  `EngineModule` + `FactoryAdapter` implementations; loaded from `~/.monocle/plugins/`
- MSRV bumps to Rust 1.92 (wasmtime requirement, OQ-11)

**Phase 4 — Cross-plane + Multi-harness + Federation (roadmap)**

- `CodeMachineModule`: second built-in `EngineModule`
- `russh 0.60` federation tunnel: TUI on host A shows sessions from host B
- `monocle-ipc` shared-memory ring buffer transport variant (OQ-08)
- OTel cost/token panel with aggregate across harnesses; revisit PostToolUse
  endpoint need at this point (JC-2)
- CCR integration: detect on PATH, write per-session JSON, set `ANTHROPIC_BASE_URL`
- rmcp MCP bridge (Phase 4 only, OQ-09): session query, prompt injection for tooling

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
- **Does NOT include `PostToolUse` hook endpoint in v1** — per Claude Code gene-source
  parity (any-context BC-HOOK-007 establishes the 5-endpoint set: PreToolUse,
  Notification, Stop, SessionStart, UserPromptSubmit; PostToolUse is intentionally
  absent). Revisit if Phase 4 OTel cost panel requires PostToolUse data. (JC-2)
- **Does NOT ship the WASM plugin SDK in v1** — Phase 3 deliverable per OQ-03;
  v1 statically bundles `VsddFactoryAdapter` as the sole built-in factory adapter
- **Does NOT ship the rmcp MCP bridge port in v1** — Phase 4 deliverable per OQ-09

## Success Criteria

v1 ships (Phase 1 complete) when ALL of the following pass:

| Outcome | Metric | Target |
|---------|--------|--------|
| Session management in popup | User can manage 3+ concurrent Claude Code sessions without leaving the editor pane | Killer scenario resolves in ≤6 keystrokes (per vision §End-to-End Killer Scenario target: 4) |
| Permission prompt latency | Permission prompt appears as overlay with diff preview after hook fires | ≤100ms from hook POST receipt to TUI overlay render on localhost |
| Hook ingestion timeout budget | Daemon responds within Claude Code's upstream timeout ceilings for each hook type | ≤300ms end-to-end response for `PreToolUse`, `Stop`, `SessionStart`, `UserPromptSubmit`; ≤2000ms for `Notification` — per gene-source BC-HOOK-022 (any-context-lazyclaude-pass-B-deep-hooks-r1.md). Exceeding these ceilings causes Claude Code to silently drop the event. Daemon broker architecture (event-bus, mpsc channel sizing) must be designed against these deadlines. |
| Hook protocol parity | Hook injection byte-compatible with Claude Code's schema | Fixture-based parity test passes against schema in any-context hooks-r1 canonical matrix (5 endpoints: PreToolUse/Notification/Stop/SessionStart/UserPromptSubmit; `X-Claude-Code-Ide-Authorization` header) |
| Factory pattern detection | vsdd-factory project detected and workflow panel populated | Detection succeeds on monocle's own `.factory/` (self-referential integration test) |
| Build matrix | Builds and tests pass on macOS and Linux | CI green on darwin/linux × amd64/arm64 |
| Drop counter active | Bounded event bus with visible drop counter | No unbounded channel in codebase; drop counter renders in status bar under synthetic high-frequency load (1000 events/sec) |
| Hook receiver body size limit | Daemon enforces 256 KiB max body on all hook POST endpoints (`/hooks/pre-tool-use`, `/hooks/prompt-submit`, `/hooks/notification`, `/hooks/stop`, `/hooks/session-start`) AND status endpoints (`/healthz`, `/status`) | Exceeding the limit returns HTTP 413 Payload Too Large with body `{"error":"payload_too_large","limit_bytes":262144}`. Rationale: Claude Code's Notification body carries an unbounded `message` string; 256 KiB covers expected-case bursts without exposing the daemon to memory exhaustion. Behavioral contract: BC-DAEMON-003 (per `SS-daemon-lifecycle.md`). |

## Phase 2 Exit Criteria

Phase 2 (Static plane) ships when:

| Outcome | Metric | Target |
|---------|--------|--------|
| Customization rendering | All 7 customization types render in Static plane on filter "All" | Zero missing types when pointed at a claude-code project with all 7 type examples |

Additional Phase 2 exit criteria will be defined by the architect during
`/vsdd-factory:create-architecture` and refined in PRD behavioral contracts.

## Constraints & Integration Points

**Tech stack inheritance**: All version pins, the wasmtime-vs-wasmi rationale,
anti-pattern enforcement rules, and RUSTSEC audit context are codified in
`/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md`,
`/Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md`,
and `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md`.
The architect inherits these as Phase 1 constraints (not up for re-selection);
per vision D-012 the tech stack is human-approved and architecturally pre-committed.

**Crate workspace layout** is fixed by vision §Workspace Layout + EX-1 ratification:
12 crates total (11 named workspace crates + 1 binary crate `monocle`) — `monocle-core`
(zero-dependency pure types), `monocle-runtime`, `monocle-tui`, `monocle-static`,
`monocle-workflow`, `monocle-plugin-sdk`, `monocle-ipc`, `monocle-config`,
`monocle-proto`, `monocle-fuzz`, `monocle-test-harness` (11 named), plus `monocle`
(binary). No crate outside the binary may depend on the binary crate.

**Action enum dispatch model** is non-negotiable per vision §Key Abstractions and
D-009: 5-level precedence (SearchPrompt > UserCustomCommand > PerContext > Global >
Builtin); enum variants (not closures) keep bindings `Eq + inspectable` for the
telescope help overlay.

**AppMode state machine** is non-negotiable per vision §Key Abstractions:
compile-time mutual exclusion (not `bag-of-Option` fields); `VecDeque<PromptModal>`
overlay stack (not single-popup — fixes lazygit's drop-on-concurrent anti-pattern);
state transitions are pure functions in `monocle-core`.

**Process topology**: monocle uses a separate tmux server (`-L monocle`) to host
the TUI client as a floating popup over the user's existing tmux session. Daemon
is long-lived. Hook POSTs are the ingestion boundary; Claude Code subprocesses are
unmodified beyond pointing their hook scripts at the daemon's lock-file-discovered
port.

**CCR is integrate-external** (D-010): detect on PATH, write per-session JSON,
set `ANTHROPIC_BASE_URL`. No CCR API changes required or expected.

**OQ + SOQ resolutions applied**: 11 architect open questions and 4 second-order
questions resolved per `/Users/jmagady/Dev/monocle/.factory/planning/oq-research.md`
(commit b3c68ca). See Phase 1 Constraints below.

## Phase 1 Constraints (from OQ Resolutions)

These constraints are derived from the orchestrator's accepted defaults on
`oq-research.md` and bind the architect during `/vsdd-factory:create-architecture`.

| Constraint | Trace |
|---|---|
| Daemon: hybrid auto-start with `MONOCLE_NO_AUTOSTART=1` escape hatch | OQ-01 |
| Hook tmpfile: shared per-runtimeDir, mode `0o600`, atomic-replace (any-context verbatim) | OQ-02 |
| WASM plugin SDK: NOT shipped in v1; ships in Phase 3; v1 statically bundles `VsddFactoryAdapter` | OQ-03 |
| Port binding: OS-assigned port + lock-file PID-liveness discovery (JC-3 closed by this) | OQ-04 |
| Profile picker: sticky-per-project; `Ctrl-P` picker override (Phase 1 user-test target; MEDIUM confidence) | OQ-05 |
| Hook event retention: hybrid RAM ring + async JSONL flush, 100MB × 5 rotation | OQ-06 |
| Cross-host migration: protobuf seams in v1 (zero runtime cost), russh transport Phase 4 | OQ-07 |
| monocle-ipc: UDS-only in v1; shared-memory ring deferred to Phase 4 transport variant | OQ-08 |
| rmcp MCP bridge: OMITTED in v1; Phase 4 ships real impl (no stub in v1) | OQ-09 |
| Daemon lock file: `directories::ProjectDirs::runtime_dir()` w/ state_dir → data_dir → `~/.monocle` fallback | OQ-10 |
| MSRV target: Phase 1 = Rust 1.86 (ratatui floor); Phase 3 bumps to 1.92 (wasmtime) | OQ-11 |
| Lock-file schema: `contract_version: u32` field from day one (zellij pattern) | SOQ-1 |
| Token rotation invariant: bind socket + lock-file write + token THEN hooks-settings reads token | SOQ-2 |
| Overlay survival: clear on daemon disconnect (Claude Code subprocesses time-out delayed responses) | SOQ-3 |
| Permission token enum: see `.factory/specs/architecture/SS-permissions-phase1.md` (architect-produced canonical artifact) — small Phase-1-purpose enum derived from Claude Code hook permission semantics (allow/deny/ask-user decisions for the 5 Phase 1 hook endpoint types); dispatcher no-op until Phase 3; zellij-style 17-variant WASM plugin permission enum is Phase 3 scope | SOQ-4 |

## Open Questions for Architect

All 11 original open questions have been resolved via `oq-research.md` (commit b3c68ca).
Three market-intel open questions (OQ-M1, OQ-M2, OQ-M3) were raised during brief v1.3
competitive positioning; all three are now resolved in-scope (adversary re-audit commit
0bd4ba9). The table below is preserved for traceability; OQ-01 through OQ-11 and
OQ-M1 through OQ-M3 decisions are final unless human red-lines.

| ID | Question | Resolution | Trace |
|----|----------|-----------|-------|
| OQ-01 | Daemon auto-start vs explicit? | Hybrid auto-start with `MONOCLE_NO_AUTOSTART=1` escape | oq-research.md §OQ-01 |
| OQ-02 | Hook tmpfile per-session or shared? | Shared per-runtimeDir, `0o600`, atomic-replace | oq-research.md §OQ-02 |
| OQ-03 | v1 ship WASM SDK or static bundle? | Static bundle in v1; WASM SDK Phase 3 | oq-research.md §OQ-03 |
| OQ-04 | Daemon port fixed or OS-assigned? | OS-assigned port + lock-file discovery | oq-research.md §OQ-04 |
| OQ-05 | Profile picker on create vs sticky? | Sticky-per-project; `Ctrl-P` override (MEDIUM confidence) | oq-research.md §OQ-05 |
| OQ-06 | Event retention ring or JSONL? | Hybrid RAM ring + async JSONL flush, 100MB × 5 | oq-research.md §OQ-06 |
| OQ-07 | Cross-host scope v1 or v4? | Protobuf seams v1 (zero cost); russh Phase 4 | oq-research.md §OQ-07 |
| OQ-08 | IPC: UDS only or UDS + shared-mem? | UDS-only v1; shared-mem Phase 4 | oq-research.md §OQ-08 |
| OQ-09 | rmcp stub in v1 or omit? | Omit entirely in v1 | oq-research.md §OQ-09 |
| OQ-10 | Lock-file location XDG or `~/.monocle`? | `directories::ProjectDirs::runtime_dir()` with fallback chain | oq-research.md §OQ-10 |
| OQ-11 | MSRV target? | Phase 1: Rust 1.86; Phase 3: Rust 1.92 | oq-research.md §OQ-11 |
| OQ-M1 | Does agent view use Claude Code hook protocol or different IPC? If hook protocol, can monocle daemon and agent view coexist on same host without port/auth collision? | Resolved — agent view dispatches via Claude Code's internal IPC (not hook protocol POSTs); monocle's daemon on an OS-assigned port + `X-Claude-Code-Ide-Authorization` header cannot collide because agent view does not bind a TCP port. No shared port or auth surface. Source: Anthropic docs https://code.claude.com/docs/en/agent-view referenced in market-intelligence.md line 222. | brief-validation-v2.md §OQ-M1; adversary re-audit 0bd4ba9 |
| OQ-M2 | Does `claude-manager` use the hook protocol, creating a second actor on the same hook-protocol surface as monocle? | Resolved — claude-manager uses tmux pane management + worktrees, NOT hook protocol. The hook-native architectural moat is intact. Source: market-intelligence.md §gap-matrix line 50 (`claude-manager... hook-overlay: NO`). | market-intelligence.md §gap-matrix; adversary re-audit 0bd4ba9 |
| OQ-M3 | Claude Code 2026 docs list 25 lifecycle events including `PermissionRequest` as a distinct hook event. Should monocle add `PermissionRequest` as a sixth endpoint (current JC-2 decision: 5 endpoints) for cleaner permission-overlay UX? | Resolved — stay at 5 endpoints (SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop). The `PermissionRequest` event is upstream of `PreToolUse`; the existing VecDeque overlay receives all permission-relevant signal via `PreToolUse` + `Notification`. Re-eval trigger: if Phase 2 trigger-trace UX testing surfaces a signal gap that PermissionRequest would fill, dispatch a fresh architecture review. Until then, 5 endpoints is canonical and final. | brief-validation-v2.md §OQ-M3; adversary re-audit 0bd4ba9 |

> **Judgment call resolutions (orchestrator-applied 2026-05-12)** — JC-1 → option B1
> (Phase 2 exit criterion); JC-2 → omit PostToolUse for Phase 1 (Claude Code parity);
> JC-3 → CLOSED via OQ-04; EX-1 → ratify 12-crate workspace (11 named + 1 binary); EX-2 → add SessionStart
> + UserPromptSubmit to Phase 1 (full 5-endpoint parity). All resolutions traceable to
> vision D-012 and oq-research.md commit b3c68ca. Human may red-line any of these in a
> follow-up brief revision.

## Overflow Context

### Competitive Positioning

Anthropic shipped `claude agents` (agent view, v2.1.139) on 2026-05-11 — one day before
brief v1.2 was finalized. Agent view provides session list + inline reply built into
Claude Code's TUI: no hook protocol, no external overlay, no diff preview, no cascaded
permission queue, no customization visibility, no workflow plane, no multi-harness support.
Monocle's differentiation is mechanism and depth, not exclusivity over the session-list
surface: hook-protocol ingestion (vs. file polling or pane scraping), VecDeque<PromptModal>
overlay (vs. attach-and-reply dispatch), diff preview (vs. none), trigger-trace to the
defining settings.json line (Phase 2, vs. none), workflow plane (Phase 3, vs. none),
multi-harness and external-overlay operation over the user's existing tmux + editor setup
without modifying Claude Code sessions (vs. built-in, lives inside Claude Code's TUI).
Anthropic shipping a thin version confirms the pain is real and significant enough for
a first-party response — monocle goes deeper on every dimension agent view does not touch.
The risk that Anthropic deepens agent view to commoditize monocle's hook-native overlay within 12 months was assessed at <10% probability based on agent view's current research-preview scope, single-harness focus, and absence of announced hook-protocol direction (per `.factory/planning/market-intelligence.md` §Risk Register, originally assessed at 25–40%; human red-line at v1.4.1 brief gate revised this to <10% based on additional context about agent view's roadmap and scope). At this probability, no risk mitigation scaffolding is required beyond the production-grade depth monocle is already shipping.

**R-001 re-eval trigger.** Re-open the R-001 risk assessment and reconsider the probability AND the mitigation requirement if ANY of the following occurs: (a) Anthropic announces hook-protocol ingestion as a first-class agent-view capability; (b) Anthropic ships diff-preview or cascaded permission-queue functionality inside agent view; (c) Anthropic extends agent view beyond Claude Code (e.g., supports a non-Claude harness); (d) Anthropic publishes a multi-harness session-management spec or RFC. Until any of these conditions materializes, the <10% assessment stands; monocle's defensible surface is depth + mechanism (hook-protocol ingestion, VecDeque overlay, diff preview, trigger-trace, workflow plane, multi-harness, external overlay).

The closest prior art beyond agent view:

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
D-001 through D-017. The canonical vision approved by human is D-012 (archived to `cycles/cycle-001/burst-log.md`).

### Phase Plan Rationale

Phase 1 ships the daemon + hook ingestion + sessions panel. This is the Phase 1
delivery scope for the killer scenario — permission prompt dispatch without
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
