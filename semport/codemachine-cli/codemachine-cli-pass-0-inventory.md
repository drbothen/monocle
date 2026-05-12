# Pass 0: Inventory — codemachine-cli

**Reference root:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/`
**Repo:** `moazbuilds/CodeMachine-CLI` @ `main`
**HEAD:** `572def63eb808e95b18ccf6c69a13d7a13fe06fd` (telemetry: replace auto-import console warnings with logger)
**Recent direction:** Active telemetry migration (~10 of last 10 commits are telemetry-related), see `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/TELEMETRY_MIGRATION_PHASE_PLAN.md`

## Tech Stack

| Aspect | Value | Citation |
|---|---|---|
| Language | TypeScript 5.4.5 (strict) + .tsx for TUI | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:111`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/tsconfig.json:1-22` |
| Runtime | Bun 1.3.3 primary; Node.js >=20.10.0 secondary | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:40-42` |
| Module system | ESM (`"type": "module"`) with `moduleResolution: "Bundler"` and custom `browser` condition | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:26`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/tsconfig.json:5-6` |
| Build system | Bun scripts (`bun scripts/build.ts`) producing per-platform binaries | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:56`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/scripts/build.ts` |
| Test framework | `bun test` declared but no tests exist in `src/` (see Verification Gap P0 in Pass 4) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:59`, `tsconfig.json:21` excludes `tests` |
| Linter | ESLint 9 flat config with typescript-eslint, import resolver | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/eslint.config.js` |
| Formatter | Prettier 3.7.4 | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:62` |
| Pre-commit | Husky 9 (`prepare: husky install`) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:67` |
| CLI parser | `commander` v14 | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:89` |
| TUI framework | `@opentui/core` + `@opentui/solid` + `solid-js` (Solid.js for terminal UI, not React) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:86-92` |
| Prompts library | `@clack/prompts` v0.11 | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:72` |
| Validation | `zod` v3.23 | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:94` |
| MCP | `@modelcontextprotocol/sdk` v1.0 | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:73` |
| Telemetry | OpenTelemetry full SDK (api, sdk-node, sdk-logs, sdk-metrics, sdk-trace-base, exporters: OTLP HTTP, Zipkin) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:74-85` |
| Embedded DB | `bun:sqlite` (built-in to Bun) for agent monitoring | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/monitoring/db/schema.ts:1` |
| File locking | `proper-lockfile` v4.1 (for log file coordination) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:91` |
| Update notification | `update-notifier` v7.3 | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:93` |
| Fuzzy search | `fuzzysort` v3.1 (TUI command palette) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:90` |

## Distribution Model

CodeMachine is distributed as a **two-package shell-wrapper system**:

1. The main `codemachine` package contains only the shell launcher `bin/codemachine.js` plus config/prompts/templates assets.
2. Platform-specific compiled binaries live in **optional dependencies**: `codemachine-{linux,darwin,windows}-{x64,arm64}`.
3. The shell wrapper detects platform/arch at runtime and `spawn()`s the correct native binary.
4. Override via env var `CODEMACHINE_BIN_PATH` (used for local dev).

Citations: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/bin/codemachine.js:14-25, 43-49, 88-127`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:27-49`.

## File Manifest (counts)

| Scope | Count | Notes |
|---|---|---|
| Total tracked files (ex .git, node_modules) | 545 | exec count via `find` |
| `.ts` files (anywhere) | 423 | |
| `.tsx` files (TUI) | 78 | exclusively under `src/cli/tui/` |
| `.ts` files in `src/` | 420 | |
| `.test.ts` / `.spec.ts` files | **0** | **P0 verification gap** |
| `.js` files (excl `.git`) | 8 | bin launcher + 4 `config/*.js` + 2 `templates/workflows/*.workflow.js` + 1 `eslint.config.js` |
| `.json` files | 8 | `package.json` + config JSONs + grafana dashboards |
| `.md` files | 15 | docs + prompts |
| Total LOC across `.ts` files (incl tsx) | 48,426 | (whole-repo `find . -name '*.ts'`) |
| Total LOC across `.js` files | 487 | |

## LOC by subsystem (`.ts` only)

| Subsystem | Files | LOC | Notes |
|---|---|---|---|
| `src/workflows/` | 114 | 13,056 | Workflow engine, state machine, runner, directives, signals |
| `src/infra/` | 115 | 12,329 | Engine plugins (7 providers), MCP servers, process spawn |
| `src/cli/` | 84 | 8,995 | Commands + Solid.js TUI |
| `src/shared/` | 65 | 7,804 | Logging, tracing, metrics, prompts, agents config, imports |
| `src/agents/` | 33 | 4,323 | Agent runner, coordinator, monitoring, session |
| `src/runtime/` | 9 | 1,050 | CLI bootstrap, workspace init |
| **`src/` total** | **420** | **47,557** | |

## Top-Level Tree (depth 2)

```
codemachine-cli/
├── bin/codemachine.js                      # platform-dispatching shell wrapper (130 LOC)
├── config/                                 # declarative agent definitions (5 .js + 1 .json)
│   ├── main.agents.js                      # main agent registry (Ali workflow builder, etc.)
│   ├── sub.agents.js                       # sub-agent registry (currently empty)
│   ├── modules.js                          # module registry (currently empty)
│   ├── placeholders.js                     # template placeholders
│   ├── agent-characters.json               # agent personality/icon metadata
│   └── package.json                        # marks config/ as a non-module package
├── docker/observability/                   # OTel collector + Grafana + Tempo + Prometheus
├── prompts/templates/ali/                  # Ali workflow builder prompts (5 chained steps)
├── scripts/{build,publish,import-telemetry}.ts
├── templates/workflows/{ali,_example}.workflow.js  # built-in workflow templates
├── src/
│   ├── agents/                             # agent execution + monitoring + coordination
│   │   ├── chat/                           # interactive chat with engine
│   │   ├── coordinator/                    # parses multi-agent scripts (& and && composition)
│   │   ├── execution/                      # low-level exec actions
│   │   ├── monitoring/                     # SQLite-backed agent registry + log capture
│   │   │   └── db/                         # `bun:sqlite` schema + repository
│   │   ├── runner/                         # executeAgent() — main entrypoint
│   │   └── session/                        # session capture + resume helpers
│   ├── cli/
│   │   ├── commands/                       # commander-registered subcommands
│   │   │   ├── auth, export, import, mcp, run, step, templates
│   │   │   └── agents/{export, list, logs}
│   │   ├── program.ts                      # commander program assembly
│   │   ├── tui/                            # Solid.js TUI (78 .tsx files)
│   │   │   ├── app.tsx, app-shell.tsx, launcher.ts, exit.ts
│   │   │   ├── routes/{home,onboard,workflow}/
│   │   │   └── shared/{components,context,hooks,services,ui,utils}
│   │   └── utils/selection-menu.ts
│   ├── infra/
│   │   ├── engines/
│   │   │   ├── core/{base,factory,registry,types,auth}.ts  # plugin abstraction
│   │   │   └── providers/{claude,codex,cursor,opencode,auggie,mistral,ccr}/
│   │   │       └── {metadata,config,auth,telemetryParser}.ts
│   │   │       └── execution/{executor,runner,commands}.ts
│   │   │       └── mcp/{settings,adapter}.ts
│   │   ├── mcp/
│   │   │   ├── registry, writer, context, setup, types, errors
│   │   │   ├── router/                     # MCP routing layer
│   │   │   └── servers/
│   │   │       ├── workflow-signals/       # propose/approve step transition tools
│   │   │       └── agent-coordination/     # agent-to-agent messaging
│   │   └── process/spawn.ts                # Bun.spawn wrapper with abort handling
│   ├── runtime/
│   │   ├── cli-setup.ts                    # boot + tracing + lazy load (516 LOC)
│   │   ├── version.ts
│   │   └── services/workspace/             # .codemachine/ folder init
│   ├── shared/
│   │   ├── agents/{config,discovery}       # agent definition loader + discovery
│   │   ├── formatters/                     # ANSI output markers, log file formatter
│   │   ├── imports/                        # workspace package import system (auto-import)
│   │   ├── logging/                        # legacy + OTel logger
│   │   ├── metrics/                        # OTel metrics setup
│   │   ├── prompts/                        # prompt content + replacement
│   │   ├── runtime/                        # dev root detection, baseline warning suppress
│   │   ├── telemetry/                      # capture + types
│   │   ├── tracing/                        # OTel tracing setup
│   │   ├── updates/                        # update-notifier wrapper
│   │   ├── utils/{path,errors,terminal}
│   │   └── workflows/{template,index}.ts   # shared workflow state helpers
│   └── workflows/
│       ├── run.ts                          # runWorkflow() — main entrypoint
│       ├── preflight.ts                    # pre-flight validation
│       ├── mcp.ts                          # workflow-level MCP config
│       ├── runner/                         # WorkflowRunner class + mode handlers
│       │   ├── core.ts                     # FSM-state-dispatch entry point
│       │   ├── modes/{continuous,interactive,autonomous}.ts
│       │   └── actions/{advance,loop,resume,directives}.ts
│       ├── state/{machine,types}.ts        # finite state machine
│       ├── step/{run,execute,engine,hooks,skip,scenarios}.ts
│       ├── controller/                     # pre-workflow controller agent
│       ├── directives/                     # post-step actions (loop, pause, trigger, checkpoint, error)
│       ├── signals/                        # user-initiated signals (pause, skip, stop, mode-change)
│       │   ├── manager/                    # SignalManager event listener coordinator
│       │   └── mcp/                        # MCP signal detector (read agent tool calls)
│       ├── input/                          # input providers (user vs controller)
│       ├── mode/                           # autoMode/interactive mode state
│       ├── indexing/                       # StepIndexManager — single source of truth
│       ├── recovery/                       # crash recovery detection
│       ├── session/                        # StepSession per-step state
│       ├── events/                         # WorkflowEventBus + emitter
│       ├── context/                        # step context utilities
│       ├── onboarding/                     # first-run flow
│       ├── templates/                      # template loader + validator + globals
│       └── utils/                          # resolvers, separator helpers
└── images/, README.md, CONTRIBUTING.md, LICENSE, TELEMETRY_MIGRATION_PHASE_PLAN.md
```

## Entry Points

| Layer | Entry | Path |
|---|---|---|
| Shell wrapper (user-facing `codemachine` or `cm`) | `bin/codemachine.js` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/bin/codemachine.js:1` |
| Bundled JS entry (after platform-binary resolve) | the compiled platform binary built from `src/runtime/cli-setup.ts` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:1`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/scripts/build.ts` |
| Runtime CLI bootstrap | `runCodemachineCli()` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:349-472` |
| Default action (no subcommand) | Launches TUI via `startTUI()` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:401-441`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/cli/tui/launcher.ts` |
| Subcommand mode | `registerCli(program)` (commander) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:444-454`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/cli/index.ts:1-3` |
| Workflow execution | `runWorkflow()` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:43-361` |
| Single-agent execution | `executeAgent()` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:196-546` |
| Engine plugin contract | `EngineModule` interface, registered via `registry.initialize()` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/base.ts:67-97`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/registry.ts:28-59` |
| MCP signal server (stdio child process) | `workflow-signals` MCP server | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/index.ts:48-58, 253-264` |
| MCP coordination server | `agent-coordination` MCP server | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/agent-coordination/index.ts` |

## Top-Level CLI Surface

Commands registered via `src/cli/commands/index.ts:1-9`:

| Command | Purpose | File |
|---|---|---|
| (default, no subcommand) | Launch TUI | `src/runtime/cli-setup.ts:374-442` |
| `codemachine run <script>` | Execute agent script (single agent or `&`/`&&` composition) | `src/cli/commands/run.command.ts:14-115` |
| `codemachine auth ...` | Per-engine authentication management | `src/cli/commands/auth.command.ts` |
| `codemachine step ...` | Run a single workflow step manually | `src/cli/commands/step.command.ts` |
| `codemachine templates ...` | List/select workflow templates | `src/cli/commands/templates.command.ts` |
| `codemachine agents {list,logs,export}` | Inspect agent registry and logs | `src/cli/commands/agents/{list,logs,export}.ts` |
| `codemachine import ...` | Install imported workflow package | `src/cli/commands/import.command.ts` |
| `codemachine export ...` | Export workflow data | `src/cli/commands/export.command.ts` |
| `codemachine mcp ...` | Manage MCP server configuration | `src/cli/commands/mcp.command.ts` |
| `codemachine {engine-id} run ...` | Engine-specific run shortcut (dynamically registered per registered engine) | `src/cli/commands/run.command.ts:61-93` |

Additional bound names: `cm` is an alias for `codemachine`. See `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:27-30`.

## Dependency Graph (high-level)

```mermaid
graph TD
    bin[bin/codemachine.js shell wrapper] --> binary[platform-specific binary]
    binary --> runtime[src/runtime/cli-setup.ts]
    runtime --> tui[src/cli/tui/launcher.ts]
    runtime --> commands[src/cli/commands/*]
    tui --> workflow[src/workflows/run.ts runWorkflow]
    commands --> workflow
    commands --> agent[src/agents/runner/runner.ts executeAgent]
    workflow --> runner[WorkflowRunner core.ts]
    runner --> fsm[state/machine.ts FSM]
    runner --> step[step/run.ts runStep]
    step --> agent
    agent --> registry[infra/engines/core/registry.ts]
    registry --> provider1[providers/claude]
    registry --> provider2[providers/codex]
    registry --> provider3[providers/cursor]
    registry --> providerN[providers/{opencode,auggie,mistral,ccr}]
    provider1 --> spawn[infra/process/spawn.ts]
    provider2 --> spawn
    provider3 --> spawn
    providerN --> spawn
    spawn --> childproc[Bun.spawn child process]
    agent --> mcp[infra/mcp/writer.ts]
    mcp --> mcpcfg[per-engine MCP settings.json]
    workflow --> signals[workflows/signals manager]
    signals --> mcpserver[infra/mcp/servers/workflow-signals stdio MCP server]
    workflow --> directives[workflows/directives reader]
    directives --> directivefile[.codemachine/memory/directive.json]
    agent --> monitor[agents/monitoring SQLite]
    monitor --> sqlitedb[.codemachine/monitor.db]
    runtime --> otel[shared/{tracing,metrics,logging} OpenTelemetry]
```

## Persisted On-Disk State (`.codemachine/` layout discovered)

The runtime maintains a `.codemachine/` workspace inside the user's project directory:

| Path | Purpose | Citation |
|---|---|---|
| `.codemachine/logs/` | Engine stdout/stderr per-agent | `src/runtime/cli-setup.ts:18` (legacy reference) + `src/agents/monitoring/db/schema.ts:14` (log_path field) |
| `.codemachine/logs/workflow-debug.log` | Workflow runner debug log (when DEBUG=true) | `src/workflows/run.ts:77` |
| `.codemachine/logs/app-debug.log` | Early app debug log (legacy/disabled) | `src/runtime/cli-setup.ts:18` |
| `.codemachine/memory/directive.json` | Agent-written directive for workflow advance behavior | `src/workflows/directives/reader.ts:13` |
| `.codemachine/agents/agents-config.json` | Workspace-local agent overrides | `src/agents/runner/runner.ts:183` |
| `.codemachine/inputs/specifications.md` | Workflow input spec (referenced, currently deferred) | `src/runtime/cli-setup.ts:270` (TODO comment) |
| `.codemachine/artifacts/` | Agent-produced artifacts (e.g., `prd.md`) | `src/infra/mcp/servers/workflow-signals/tools.ts:36` |
| `.codemachine/inputs/` | Spec/input files | (referenced in TODOs) |
| (SQLite DB path — TBD per repo) | Agent monitoring DB | `src/agents/monitoring/db/connection.ts` (not read yet, deepen in Pass B) |
| `~/.codemachine/claude/` | Per-user Claude auth config dir override (CLAUDE_CONFIG_DIR) | `src/infra/engines/providers/claude/execution/runner.ts:185-187` |
| `~/.codemachine/mcp/workflow-signals/` | Global MCP workflow-signal storage | `src/infra/mcp/servers/workflow-signals/index.ts:15` |
| `~/.codemachine/logs/debug.log` | MCP server debug log | `src/infra/mcp/servers/workflow-signals/index.ts:29` |

## Notable Design Genes (preliminary, deepened in later passes)

1. **Engine-as-plugin registry** — every AI coding CLI (Claude Code, Codex, Cursor, OpenCode, Auggie, Mistral, CCR) is implemented as an `EngineModule` with identical contract: `metadata` + `auth` + `run` + optional `mcp` + optional `syncConfig`. Auto-registered at module load via `registry.initialize()`. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/registry.ts:35-59`.

2. **Headless CLI invocation as transport** — agents are NOT spoken to via SDK/API. They're spawned as child processes with non-interactive flags (`--print`, `--output-format stream-json`, etc.) and streamed via stdout. See `buildClaudeExecCommand` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/commands.ts:45-75`) and `buildCodexCommand` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/codex/execution/commands.ts:14-48`).

3. **Session model is engine-provided** — CodeMachine doesn't own session IDs. It captures them from each engine's stream output (Claude `session_id` field) and passes them back via engine-specific resume flags (e.g., `claude --resume <id>`, `codex resume <id>`). `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts:245-280`.

4. **Workflow templates are JS modules, not JSON** — `templates/workflows/*.workflow.js` use a typed `WorkflowTemplate` interface; loaded dynamically via `import()`. Validation by hand-rolled validator (no Zod for templates). `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/loader.ts:16-34, 37-71`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/validator.ts:20-260`.

5. **Workflow execution = explicit FSM** — finite state machine with states `idle → running → {awaiting | delegated} → ... → {completed|stopped|error}`. Each state has explicit transition guards. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/types.ts:15-22`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts:120-377`.

6. **Two-way signaling via MCP** — workflow signals (propose/approve step transitions) are exposed to agents as MCP tools. Replaces "fragile text-based signals like 'ACTION: NEXT'" with schema-validated tool calls. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/index.ts:6-15, 64-247`.

7. **Crash recovery via persisted step data** — `StepIndexManager` saves session IDs to `template.json`; on Ctrl+C, the cleanup handler captures the in-flight session before exit so the next run can resume mid-conversation. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:88-124`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/recovery/detect.ts:22-34`.

8. **OpenTelemetry-first observability** — every boot phase, span, and metric is instrumented; trace export is OTLP/Zipkin, but only activated when `CODEMACHINE_TRACE` is set (opt-in to avoid latency). `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:27-64`.

9. **Workspace is a `.codemachine/` folder in user project** — not a global daemon; everything is colocated with the user's code. Permits `git`-friendly persistence (though `.codemachine/` is typically gitignored).

10. **Imports system (workflow package manager)** — workflows/agents can be imported from npm-distributed packages, resolved at template load time. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/imports/auto-import.ts`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/imports/resolver.ts`.

## Reference-Repo Anomalies / Risks Spotted

- **No tests anywhere**. `find . -name '*.test.ts' -o -name '*.spec.ts'` returns 0 results. `tsconfig.json:21` excludes `tests/` but no such folder exists. **All behavioral confidence must come from code-reading, not test fixtures.** Carried to Pass 3 as a P0.
- **Pre-release version 0.8.0** (`package.json:3`). Reflects "we ship to users" but interfaces may still churn.
- **CCR engine** — provider id `ccr` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/ccr/`) — based on file structure appears to be Claude Code Router (community proxy), not a separate model vendor. Deepen in Pass B.
- **`agents-coordination` MCP server** is shipped but its purpose vs `workflow-signals` is not yet clear from cursory read. Deepen in Pass B.
- **TUI uses Solid.js, not React** — meaningful for monocle if it intends to lift TUI patterns; Solid signals don't match React state mental model.
- **Mass commented-out code in `cli-setup.ts`** — telemetry refactor in flight; legacy `appDebug` callsites pending removal (lines 6-21, 152-194, 269-271, 309-320). Risk of dual-codepath confusion during further reads.

## State Checkpoint

```yaml
pass: 0
status: complete
files_scanned: 545
loc_scanned_directly: ~4500
timestamp: 2026-05-11T00:00:00Z
next_pass: 1
notes: |
  Pass 0 captures stack, layout, and entry points. Subsystem-level structure
  is established. No tests exist; gaps will surface in Pass 3.
```
