# Final Synthesis — codemachine-cli

**Repo:** `moazbuilds/CodeMachine-CLI` @ `main` `572def63eb808e95b18ccf6c69a13d7a13fe06fd`
**Path:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/`
**Size:** 552 files, 27 MB. 423 `.ts` + 78 `.tsx` files, 48,426 LOC TypeScript; 487 LOC JavaScript (mainly entrypoint shim + 5 config files).
**Stack:** Bun 1.3.3 runtime (Node ≥20.10 fallback), TypeScript 5.4 strict + ESM, Solid.js TUI, Commander CLI, OpenTelemetry, bun:sqlite, MCP SDK, Zod, hand-rolled FSM.
**License:** Apache-2.0.
**Test footprint:** **ZERO** (no `.test.ts`, no `.spec.ts`, archived under hidden `.tests.archive/` not committed). All confidence below is code-derived.

This synthesis supersedes Pass 0–6 and consolidates Phase B deepening + B.5 audit + B.6 recount.

---

## 1. Executive Summary

CodeMachine is a **multi-harness AI-coding-agent orchestrator** with a **declarative workflow engine** layered over a uniform **engine-plugin** abstraction. The genes monocle would want to lift are:

1. **`EngineModule` plugin contract** — a 40-line TypeScript interface that defines what it means to "be an AI coding harness peer". CodeMachine implements 7 of them (OpenCode, Claude Code, Codex, Cursor, Mistral, Auggie, CCR). Each is ~600–1000 LOC of provider folder with identical layout.
2. **Headless-CLI-as-transport** — every engine is spawned as a child process via `Bun.spawn` with non-interactive flags; outputs are line-delimited JSON streams. No SDK use, no API calls owned by CodeMachine.
3. **MCP router** — a single in-process MCP server that fans out to in-process built-in servers (workflow-signals, agent-coordination) AND external user-defined servers. Engines see one `codemachine` MCP entry; per-step filtering via a project-cwd context file.
4. **Declarative workflow templates** as ESM JS modules with `resolveStep`/`resolveFolder`/`resolveModule`/`controller`/`separator` globals.
5. **Workflow FSM** — explicit, declarative state machine (7 states, 9 events, guarded transitions) with crash recovery via persisted session IDs.
6. **Three-axis scenario dispatcher** — `(interactive, autoMode, hasChainedPrompts) → 1 of 8 scenarios → 1 of 3 mode handlers`. The orchestration logic is data-driven.
7. **Two signal channels** — process events (user-initiated, fast) vs filesystem directive.json (agent-initiated, durable) vs MCP tool calls (agent-initiated, schema-validated).
8. **Workflow-as-package imports** — npm-installable workflow packages with manifest, registered globally to `~/.codemachine/imports/`.

The single biggest design quality: **subsystems are deliberately Decomposed By Source of Truth.** `StepIndexManager` owns step state. `WorkflowMode` owns auto-vs-manual. `MonitorService` owns session IDs (read back even by recovery, not duplicated). Comments in code make these ownership boundaries explicit.

The single biggest design weakness: **zero tests.** All behavior is uniform-pattern code; absence of tests is a P0 risk for any port.

---

## 2. Harness Architecture (User-Stated Deliverable 1)

### 2.1 The `EngineModule` Contract

From `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/base.ts:10-97`:

```ts
interface EngineMetadata {
  id: string;                  // e.g. 'claude'
  name: string;                // e.g. 'Claude Code'
  description: string;
  cliCommand: string;          // CLI namespace
  cliBinary: string;           // binary on $PATH
  installCommand: string;      // npm install -g @anthropic-ai/claude-code
  defaultModel?: string;
  defaultModelReasoningEffort?: 'low' | 'medium' | 'high';
  order?: number;              // sort order in UI/registry-default
  experimental?: boolean;
  icon?: string;
}

interface EngineAuthModule {
  isAuthenticated(opts?): Promise<boolean>;
  ensureAuth(opts?): Promise<boolean>;
  clearAuth(opts?): Promise<void>;
  nextAuthMenuAction(opts?): Promise<'login' | 'logout'>;
}

interface EngineMCPConfig {
  supported: boolean;
  configure?(workflowDir: string): Promise<void>;
  cleanup?(workflowDir: string): Promise<void>;
  isConfigured?(workflowDir: string): Promise<boolean>;
}

interface EngineRunOptions {
  prompt: string;
  workingDir: string;
  resumeSessionId?: string;
  resumePrompt?: string;
  model?: string;
  modelReasoningEffort?: 'low' | 'medium' | 'high';
  env?: NodeJS.ProcessEnv;
  onData?: (chunk: string) => void;
  onErrorData?: (chunk: string) => void;
  onTelemetry?: (telemetry: ParsedTelemetry) => void;
  onSessionId?: (sessionId: string) => void;
  abortSignal?: AbortSignal;
  timeout?: number;             // default 30 minutes
}

interface EngineModule {
  metadata: EngineMetadata;
  auth: EngineAuthModule;
  run: (options: EngineRunOptions) => Promise<EngineRunResult>;
  syncConfig?: (opts?) => Promise<void>;
  onRegister?: () => void;
  onLoad?: () => void;
  mcp?: EngineMCPConfig;
}
```

Singleton `EngineRegistry` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/registry.ts:20-128`) provides:
- `register(engine)` — first-write-wins on duplicate ID, log+continue on validation fail
- `get(id)`, `getAll()` (sorted by `order`), `getAllIds()`, `getAllMetadata()`, `has(id)`, `getDefault()` (first by order)
- Auto-initialization at module load via static imports of all 7 providers (lines 9-15)

### 2.2 Engine Inventory (full)

| id | CLI binary | Default model | Order | Experimental | Base command + flags | Resume mechanism | Stream format |
|---|---|---|---|---|---|---|---|
| `opencode` | `opencode` | `opencode/big-pickle` | 1 | no | `opencode run --format json` | `--session <id>` | newline-JSON, `sessionID` field |
| `claude` | `claude` | `opus` | 2 | yes | `claude --print --output-format stream-json --verbose --dangerously-skip-permissions --permission-mode bypassPermissions` | `--resume <id>` | newline-JSON, `session_id` field, telemetry from `.jsonl` file post-result |
| `codex` | `codex` | `gpt-5.2-codex` | 3 | no | `codex exec --json --skip-git-repo-check --sandbox danger-full-access --dangerously-bypass-approvals-and-sandbox -C <cwd>` | `codex exec resume <id> <prompt>` (positional sub-command) | newline-JSON |
| `cursor` | `cursor-agent` | `auto` | 4 | yes | `cursor-agent -p --force --output-format stream-json` | `--resume=<id>` (equals sign) | newline-JSON |
| `mistral` | `vibe` | `devstral-2` | 5 | yes | `vibe -p '<prompt>' --auto-approve --output streaming` | `--resume <id>` | newline-JSON; prompt is **positional**, not stdin |
| `auggie` | `auggie` | (none) | 6 | no | `auggie --print --quiet --output-format json` | `--resume <id>` | JSON |
| `ccr` | `ccr` | `sonnet` | 7 | no | `ccr code --print --output-format stream-json --verbose --dangerously-skip-permissions --permission-mode bypassPermissions` | `--resume <id>` | newline-JSON (Claude-compatible) |

Citations: per-engine `metadata.ts` and `execution/commands.ts` files in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/<id>/`.

### 2.3 Process Spawning

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:171-402` — `spawnProcess({command, args, cwd, env, onStdout, onStderr, signal, stdioMode, timeout, stdinInput})` is the single low-level routine. Key behaviors:

- Stdin: `TextEncoder().encode(stdinInput)` (or `'inherit'` for TUI mode).
- Stdout/stderr: streamed via `getReader()` + decoder loop; abort cancels readers cleanly.
- Abort: `AbortSignal` direct or internal `AbortController` for timeout. On abort, Unix kills **process group** (`process.kill(-pid, SIGTERM)`) with 100ms SIGKILL escalation. Windows: `child.kill('SIGTERM')` + escalation.
- Tracking: module-level `activeProcesses: Set<Subprocess>` — every spawn is registered. `killAllActiveProcesses()` (lines 128-169) iterates this set in SIGINT/SIGTERM handlers.
- Command resolve: `Bun.which(command)` → `$PATH` lookup; special `'bun'` → `process.execPath`.

### 2.4 Session Model

CodeMachine **does not own session state**. Each engine CLI maintains its own conversation transcript on disk (`~/.claude/projects/...`, `~/.codemachine/opencode/data/...`, etc., with per-engine env override possible).

CodeMachine records:
- **Session ID** in workspace `.codemachine/template.json` → `completedSteps[stepIdx].sessionId`
- **Session ID** in SQLite monitoring DB → `agents.session_id` column
- **Telemetry** parsed from stream events (live for OpenCode; post-result file-read for Claude)

Resume = a **new process spawn** with the engine's resume flag. There is no IPC, no daemon, no persistent connection. The next turn's prompt is sent via stdin (or positional, for Mistral).

### 2.5 Authentication

Two strategies in use:

1. **Credential file** (Claude, Codex, others): `isAuthenticated()` checks for `.credentials.json` or `$ANTHROPIC_API_KEY`. `ensureAuth()` runs the engine's interactive `login` subcommand via `spawnInteractive`. `clearAuth()` deletes the credential file.
2. **CLI-installed-is-enough** (OpenCode): `isAuthenticated() = Bun.which(cliBinary) !== null`. Auth lives in OpenCode's own home.

Shared helpers: `checkCliInstalled(command, {versionFlag, timeout})`, `displayCliNotInstalledError`, `isCommandNotFoundError`, `ensureAuthDirectory`, etc., at `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/auth.ts`.

A 5-minute global TTL cache (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:21-48`) sits in front of every `auth.isAuthenticated()` call to "fix the 5-minute delay bug when spawning multiple subagents" (per comment). Known wart: not invalidated on `clearAuth()`.

### 2.6 MCP Topology — The Killer Feature

**One MCP server per engine.** That server is `codemachine mcp router` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/router/config.ts:68-80`), which is the same binary CodeMachine ships, in a different mode.

The router (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/router/index.ts:37-160`) aggregates:

- **In-process built-in servers** — `workflow-signals` + `agent-coordination`. Function calls, no IPC. (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/router/config.ts:92-105`)
- **External user-defined servers** — spawned as stdio child processes via `@modelcontextprotocol/sdk` Client. (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/router/backend.ts:40-212`)
- User-defined servers come from `~/.config/codemachine/mcp-servers.json` (global) and `.codemachine/mcp-servers.json` (project). (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/router/config.ts:125-161`)

**Per-step / per-agent tool filtering** via a project-cwd file `.codemachine/mcp/context.json`:

```ts
// /Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/types.ts:76-81
interface MCPContextFile {
  version: 1;
  activeServers: MCPServerFilterConfig[];   // {server, only?, exclude?, targets?}
  uniqueAgentId?: string;
  timestamp: number;
}
```

Filter semantics (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/router/backend.ts:494-578`):

- **Empty `activeServers` array = NO tools available.** Default-deny.
- `only` / `exclude` apply per-server; `only` wins.
- `targets` is injected into tool-call args as `_allowed_targets` for in-process servers to enforce (used by `agent-coordination` to scope which agents this caller is allowed to message).

Written by `executeAgent` before invoking the engine (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:316-326`). Read by router on every `tools/list` and `tools/call`.

### 2.7 Built-in MCP Servers (7 tools total)

**`workflow-signals`** (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/`):

| Tool | Args | Effect |
|---|---|---|
| `propose_step_completion` | `step_id, artifact_path, [artifact_hash], checklist, [open_questions], confidence` | Validate via Zod; auto-compute SHA-256 if missing; enqueue proposal to filesystem queue at `~/.codemachine/mcp/workflow-signals/`. |
| `approve_step_transition` | `step_id, decision: 'approve'|'reject'|'revise', [blockers], [notes]` | Decision gate. `revise` requires non-empty `blockers`. |
| `get_pending_proposal` | — | Returns latest proposal as formatted text. |

**`agent-coordination`** (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/agent-coordination/`):

| Tool | Args | Effect |
|---|---|---|
| `run_agents` | `script, [working_dir], [timeout_ms]` | Execute a coordinator script (`a && b & c`) synchronously from inside an agent. Default timeout 10 min. |
| `get_agent_status` | `[agent_id], [name], [status[]], [limit]` | Query SQLite monitoring DB. Status enum: `running/completed/failed/paused/skipped`. |
| `list_active_agents` | — | All `running` agents. |
| `list_available_agents` | `[working_dir]` | Catalog of agent definitions from `main.agents.js` + `sub.agents.js` + workflow templates. |

### 2.8 How Engines Talk to MCP (per-engine adapter)

Each engine has a `mcp/adapter.ts` implementing the `MCPAdapter` interface (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/types.ts:37-55`):

- `getSettingsPath(scope, projectDir?)` — engine's MCP settings file location. Claude: `${CLAUDE_CONFIG_DIR}/.claude.json` (user) or `.mcp.json` (project). OpenCode/CCR: per-engine equivalents.
- `configure(workflowDir, scope)` — writes the router entry to the settings file:
  ```json
  {
    "mcpServers": {
      "codemachine": { "command": "codemachine", "args": ["mcp", "router"] }
    }
  }
  ```
- `cleanup` / `isConfigured`

Lazy invocation: `ensureMCPConfig(engineId, workingDir)` is called per-step in `executeAgent` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:311-314`). Fast-path: check if router entry exists, skip; otherwise write.

**Compiled-binary gotcha:** when `__dirname.startsWith('/$bunfs')`, dynamic `import(`./engines/providers/${id}/mcp/index.js`)` doesn't work. Adapters must be statically imported elsewhere first to pre-register. (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/writer.ts:46-69`)

### 2.9 The Profile / Launcher Abstraction monocle Needs

For monocle to be a peer of Claude Code AND to host other engines as peers of itself, the abstraction is the **`EngineModule` contract** verbatim plus the **MCP router pattern**. Specifically:

1. Implement `monocle`'s own `EngineModule` so other harnesses can drive monocle as a child process. Required: a non-interactive flag set (think `monocle --print --output-format json`), a session ID surfaced in the stream, `--resume <id>` resume support.
2. Add monocle's harness-side: a `monocle/run.ts` that conforms to `EngineRunOptions → EngineRunResult` — launches OTHER engines (Claude, Codex, etc.) via the same plugin contract.
3. The MCP-router pattern is optional but recommended: one router fans out to multiple servers, including per-agent filtering. Monocle could either:
   - Embed CodeMachine's router as-is.
   - Roll its own with the same `MCPContextFile` filtering pattern.
   - Skip and let each engine talk directly to its MCP servers (loses per-step filtering).

The **monocle-as-peer** part is small: it's the engine plugin module (~600 LOC of provider folder). The **monocle-as-host** part is bigger: it's the workflow engine on top.

### 2.10 Comparison vs Claude Code

| Aspect | Claude Code | CodeMachine |
|---|---|---|
| Outer loop | TUI conversation | Workflow runner FSM |
| Engine count | 1 (Claude itself) | 7 plugins |
| Session granularity | One per user thread | One per step + one for controller |
| Session ID source | Self-assigned | Inherited from each engine CLI |
| Cross-step memory | None (each session is independent) | Workflow-level: `.codemachine/memory/directive.json`, `.codemachine/template.json` |
| Tool model | MCP-native | MCP via router; per-step filtering via context file |
| Tool/permission gate | Per-tool prompt | `--dangerously-skip-permissions` bypass; CodeMachine acts with user's full trust |
| Auto vs manual progression | All manual | Scenario dispatcher + auto-mode |
| Hooks | Plugins via MCP | Engines (process-level) + MCP servers (tool-level) + imports (workflow package) + directives (file-level) |

---

## 3. Workflows System (User-Stated Deliverable 2)

### 3.1 Template Format

Templates are **ESM JS modules** with a default export of `WorkflowTemplate`. They use injected globals: `resolveStep`, `resolveFolder`, `resolveModule`, `separator`, `controller`.

```ts
interface WorkflowTemplate {
  name: string;                    // required
  steps: WorkflowStep[];           // required
  subAgentIds?: string[];          // sub-agents mirrored into workspace
  tracks?: TracksConfig;           // user-picks-one at onboarding
  conditionGroups?: ConditionGroup[];  // nested questions gating steps
  controller?: ControllerDefinition;   // pre-workflow conversational agent
  specification?: boolean;          // require .codemachine/inputs/specifications.md
  autonomousMode?: 'true' | 'false' | 'never' | 'always';
}

type WorkflowStep = ModuleStep | Separator;

interface ModuleStep {
  type: 'module';
  agentId: string;
  agentName: string;
  promptPath: string | string[];   // multiple files concatenated with \n\n
  model?: string;
  modelReasoningEffort?: 'low' | 'medium' | 'high';
  engine?: string;
  module?: { id: string; behavior?: LoopBehavior | TriggerBehavior | CheckpointBehavior };
  executeOnce?: boolean;            // skip if already executed
  interactive?: boolean;            // default true; false enables auto-advance
  tracks?: string[];                // step appears only for matching tracks
  conditions?: string[];            // AND of conditions
  conditionsAny?: string[];         // OR of conditions
  mcp?: MCPConfig;                  // per-step MCP filter
}
```

Citations: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/types.ts:33-103`.

Loader: `loadWorkflowModule()` dynamically imports with cache-busting (`?ts=<now>`) so dev-reload works (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/loader.ts:16-34`).

Validator: hand-rolled, error-aggregating, returns `{valid, errors[]}` with path-qualified messages (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/validator.ts:20-260`). Not Zod — to keep templates pure JS modules.

### 3.2 Execution Model — Sequential with FSM Gates

The workflow is a **linear ordered list of steps** with the following control flow:

1. Steps are filtered by `tracks` + `conditions` + `conditionsAny` (per the user's onboarding selections).
2. The runner executes steps in order via an FSM with 7 states, 9 events.
3. At step boundaries, **directives** (5 types, evaluated in priority order) can modify control flow: rewind to a previous step (loop), branch to a separate agent (trigger), gate progression (checkpoint), pause, error-stop.

There is no DAG. There is no parallelism at the step level (parallelism only exists inside coordinator scripts via `&`).

### 3.3 The FSM (7 states, 9 events)

**States:** `idle → running → {awaiting | delegated} → … → {completed | stopped | error}`

| State | Meaning |
|---|---|
| `idle` | Not started |
| `running` | Agent executing |
| `awaiting` | Waiting for user input |
| `delegated` | Controller agent is running in autonomous mode |
| `completed` | All steps done |
| `stopped` | User stopped |
| `error` | Fatal |

**Events:** `START`, `STEP_COMPLETE`, `STEP_ERROR`, `INPUT_RECEIVED`, `RESUME`, `SKIP`, `PAUSE`, `STOP`, `DELEGATE`, `AWAIT`.

**Critical transitions:**

| From | Event | To | Guard |
|---|---|---|---|
| `running` | `STEP_COMPLETE` | `delegated` | `autoMode && !paused && (hasController || !step.interactive)` |
| `running` | `STEP_COMPLETE` | `awaiting` | (otherwise) |
| `awaiting` | `DELEGATE` | `delegated` | (no guard; sets `autoMode=true`) |
| `delegated` | `AWAIT` | `awaiting` | (no guard; sets `autoMode=false`) |
| any non-final | `PAUSE` | `awaiting` | (sets `autoMode=false, paused=true`) |

Final states absorb all subsequent events silently.

Citation: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts:120-377`.

### 3.4 The 8-Scenario Dispatcher

A step's execution branches based on three flags: `(interactive, autoMode, hasChainedPrompts)`. This produces 8 scenarios, mapping to one of 3 mode handlers:

| # | interactive | autoMode | chainedPrompts | Mode | Input source | Behavior |
|---|---|---|---|---|---|---|
| 1 | true | true | yes | interactive | controller | Controller drives with prompts |
| 2 | true | true | no | interactive | controller | Controller drives single step |
| 3 | true | false | yes | interactive | user | User drives with prompts |
| 4 | true | false | no | interactive | user | User drives each step |
| 5 | false | true | yes | autonomous | system | Fully autonomous |
| 6 | false | true | no | continuous | system | Auto-advance |
| 7 | false | false | yes | interactive (forced) | user | Forced user-driven + warning |
| 8 | false | false | no | interactive (forced) | user | Forced user-driven + warning |

When `ctx.mode.paused`, **all scenarios are coerced to interactive** regardless. (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/core.ts:218-225`)

Citation: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/scenarios/definitions.ts:29-121`.

### 3.5 Step Types (declarative behavior)

| `module.behavior.type` | Triggers handler | Effect |
|---|---|---|
| `'loop'` (`action: 'stepBack'`) | `handleLoopLogic` | When directive=loop, rewind `behavior.steps` positions, optionally skip listed agents, enforce `maxIterations` |
| `'trigger'` (`action: 'mainAgentCall'`) | `handleTriggerLogic` | When directive=trigger, synchronously run `behavior.triggerAgentId` as a side-conversation |
| `'checkpoint'` (`action: 'evaluate'`) | `handleCheckpointLogic` | Stop workflow; user choice resolves via `checkpoint:continue` / `checkpoint:quit` events |

### 3.6 Directives — Two Channels

**Channel A — On user advance (Enter press / empty input):** `evaluateOnAdvance(cwd)` reads `.codemachine/memory/directive.json` and dispatches one of `{advance, loop, stop, error, checkpoint, pause, trigger}`. After read, file is reset to `{action: 'continue'}`. (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/directives/{reader,onAdvance}.ts`)

**Channel B — Post-step (deterministic priority order):**

1. **error** — workflow stops with `error` state.
2. **trigger** — synchronously run the named trigger agent (separate `engine.run()` call); continue to next directive.
3. **executeOnce** marker — mark step completed in indexManager.
4. **checkpoint** — block on user choice (continue/quit). If continue, skip chained prompts and advance.
5. **pause** — return `{pauseRequested: true}` so mode handler enters paused state.
6. **loop** — rewind to `currentIndex - stepsBack - 1`, increment counter, reset agent UI for re-executed steps.

Citation: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/hooks.ts:242-372`.

### 3.7 Step Persistence (`template.json` schema)

```ts
interface TemplateTracking {
  activeTemplate: string;           // filename of current workflow
  lastUpdated: string;              // ISO
  completedSteps?: Record<string, StepData>;
  notCompletedSteps?: number[];
  resumeFromLastStep?: boolean;
  selectedTrack?: string;
  selectedConditions?: string[];
  projectName?: string;
  autonomousMode?: string;
  controllerConfig?: ControllerConfig;
}

interface StepData {
  sessionId: string;
  monitoringId: number;
  completedChains?: number[];      // chained-prompt indices done
  completedAt?: string;            // presence = step fully done
}

interface ControllerConfig {
  agentId: string;
  sessionId: string;
  monitoringId: number;
}
```

Migration: an older array format for `completedSteps` is detected and converted on load. (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/indexing/persistence.ts:32-50`)

### 3.8 Crash Recovery (resume after Ctrl+C / crash)

Resume decisions:
- `START_FRESH` if no template.json or no prior state.
- `RESUME_FROM_CHAIN` if newest step has `completedChains.length > 0 && !completedAt` → restart that step, re-load chained prompts, jump to `max(completedChains)+1`.
- `RESUME_FROM_CRASH` if newest step has `sessionId && !completedAt` → resume that step's CLI session.
- `CONTINUE_AFTER_COMPLETED` → start fresh at last completed + 1.

`restoreFromCrash` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/recovery/restore.ts:31-133`):
1. Register monitoringId with emitter (TUI re-attaches log panel)
2. Re-load persisted telemetry from SQLite, re-emit it
3. Set agent status to `awaiting`
4. Set machine context
5. Re-load chained prompts from agent config, build queue at `resumeIndex`

Cleanup-on-exit (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:88-124`):
- For each in-flight agent with a `sessionId`, save state to `template.json` (or `controllerConfig` for controller).
- Triggered by SIGINT/SIGTERM/beforeExit via `MonitoringCleanup.registerWorkflowHandlers({onBeforeCleanup})`.

### 3.9 Controller View — Pre-Workflow Conversational Agent

If template has `controller: controller('agent-id', {engine?, model?})`, the runner runs a separate **pre-workflow phase**:

1. Set `autonomousMode='never'`, `view='controller'`.
2. Initialize controller agent via `executeAgent` — captures session ID, saves to `controllerConfig`.
3. User has a conversation with the controller agent.
4. On user advance, set `autonomousMode='true'`, `view='executing'`, start workflow steps.
5. Even during step execution, user can press `c` to swap input back to the controller (signal `workflow:return-to-controller`).

Citation: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/controller/{view,init,helper}.ts`.

### 3.10 Workflow Events (40 types, the runner↔UI seam)

`WorkflowEventBus` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/events/event-bus.ts:51-...`) is the formal seam: typed pub-sub, supports general + type-specific subscribers, optional history replay. The runner emits 40 distinct event types in 13 categories (agent lifecycle, controller, sub-agent, triggered-agent, workflow state, loop, checkpoint, input state, message, separator, monitoring, progress, onboarding).

**For monocle's headless mode:** subscribe to the bus, render however you want. The bus is the contract; no other coupling.

Citation: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/events/types.ts:87-154`.

### 3.11 Workflow Imports — Package Distribution

Workflows can be distributed as npm-installable packages with a manifest:

```ts
// codemachine.json
{
  "name": "ali-workflow",
  "version": "0.1.0",
  "paths": {  // optional overrides
    "config": "config/",
    "workflows": "templates/workflows/",
    "prompts": "prompts/",
    "characters": "config/agent-characters.json"
  }
}
```

Installed packages live at `~/.codemachine/imports/<repo-name>/`. Registry at `~/.codemachine/imports/registry.json`. Resolution order: **imports first, local fallback** (imports can shadow local agents/workflows/prompts). (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/imports/resolve.ts:20-113`)

### 3.12 Coordinator Script Grammar (Side-channel orchestration)

Used by `codemachine run <script>` CLI and the `run_agents` MCP tool:

| Form | Meaning |
|---|---|
| `agent-name 'prompt'` | Single agent |
| `agent-name` | Agent with template-only prompt |
| `agent[input:file.md;file2.md,tail:N,prompt:"text",custom:val]` | Enhanced syntax |
| `a & b & c` | Parallel |
| `a && b && c` | Sequential (stops on first failure) |
| `a && b & c && d` mixed | `&&` makes outer groups, `&` makes inner-parallel within each group |

Parser: hand-rolled quote-aware (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/coordinator/parser.ts:1-464`). Does NOT support parentheses; mixed-mode interpretation is by convention.

### 3.13 Comparison to vsdd-factory's lobster format

The user noted in-flight vsdd-factory ingest. Without access to that ingest, I document CodeMachine's pattern only:

- **Templates are CODE, not data.** ESM JS modules with default exports. Validator runs on the imported object, not on source text.
- **Single ordered step list, no DAG.** Parallelism is in coordinator scripts only.
- **Behavior baked into step shape.** Loop/trigger/checkpoint declared on the step via `module.behavior`; agents send directives via `.codemachine/memory/directive.json` to modify control flow.
- **Persistence is one JSON file** (`template.json`) per workspace, plus SQLite agent monitoring.

---

## 4. Cross-Cutting Concerns

### 4.1 Error Handling

- Throw `Error` with descriptive message at boundaries; let it bubble.
- `error.name === 'AbortError'` is **always** treated as controlled cancellation, never a failure.
- Process-level: `try { … } catch { exitCode = 1 } finally { …shutdown… process.exit(exitCode) }` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:502-515`).
- Custom error classes minimal — only `MCPConfigError`, `ValidationError` discovered.

### 4.2 Observability

- OpenTelemetry-first; opt-in via `CODEMACHINE_TRACE` env. When unset, OTel SDK is **never imported** — zero cost.
- Spans use dotted-hierarchy names: `cli.boot`, `cli.preBoot.cli_deps_import`, `cli.lazy.engines`, etc.
- Metrics: boot-phase histogram (`boot.phase_duration` ms, label `boot.phase`).
- Local dev stack: `docker/observability/docker-compose.yml` (OTel Collector + Tempo + Prometheus + Grafana with codemachine-observability/codemachine-process-metrics/codemachine-boot-metrics dashboards).
- Currently in active flight per `TELEMETRY_MIGRATION_PHASE_PLAN.md` (714 lines); HEAD commit is telemetry-migration-related.

### 4.3 Configuration State (where things live)

| Path | Contents |
|---|---|
| `.codemachine/template.json` | Workflow state: activeTemplate, completedSteps (with sessionId, monitoringId, completedChains, completedAt), selectedTrack, selectedConditions, projectName, autonomousMode, controllerConfig |
| `.codemachine/memory/directive.json` | Agent-written directive (`{action: 'continue'|'loop'|'stop'|...|'trigger', reason?, triggerAgentId?}`) |
| `.codemachine/agents/agents-config.json` | Per-workspace agent overrides |
| `.codemachine/mcp/context.json` | Per-step MCP filter (read by router) |
| `.codemachine/logs/<agent>.<id>.log` | Per-agent stdout/stderr (proper-lockfile coordination) |
| `.codemachine/logs/workflow-debug.log` | Workflow runner debug (DEBUG=true) |
| `.codemachine/inputs/specifications.md` | Spec required if `template.specification === true` |
| `.codemachine/artifacts/` | Agent-produced artifacts (e.g., `prd.md`) |
| `~/.codemachine/claude/` | Claude auth + per-engine config (overrides via `CODEMACHINE_CLAUDE_HOME`) |
| `~/.codemachine/opencode/{config,cache,data}/` | OpenCode XDG-style home |
| `~/.codemachine/{codex,cursor,mistral,auggie,ccr}/` | Per-engine homes |
| `~/.codemachine/mcp/workflow-signals/` | MCP signal queue (file-backed) |
| `~/.codemachine/imports/<repo>/` + `registry.json` | Workflow package imports |
| `~/.config/codemachine/mcp-servers.json` | User-global MCP server definitions |

### 4.4 Environment Variables

| Var | Effect |
|---|---|
| `CODEMACHINE_CWD` | Override workspace |
| `CODEMACHINE_BIN_PATH` | Use specific platform binary (dev) |
| `CODEMACHINE_PACKAGE_ROOT` | Used by shell wrapper |
| `CODEMACHINE_TRACE` | Enable OTel telemetry (opt-in) |
| `CODEMACHINE_PARENT_AGENT_ID` | Propagated to child processes for monitoring hierarchy |
| `CODEMACHINE_AGENT_TIMEOUT` | Override 30-min step timeout |
| `CODEMACHINE_SPEC_PATH` | Override spec file path |
| `CODEMACHINE_<ENGINE>_HOME` | Per-engine config dir (e.g., `CODEMACHINE_CLAUDE_HOME`) |
| `CODEMACHINE_ANTHROPIC_BASE_URL`, `CODEMACHINE_ANTHROPIC_AUTH_TOKEN`, `CODEMACHINE_ANTHROPIC_API_KEY`, `CODEMACHINE_CLAUDE_OAUTH_TOKEN` | Anthropic API overrides (forwarded to child) |
| `MCP_TIMEOUT`, `MCP_TOOL_TIMEOUT` | Set to `900000` (15 min) when invoking Claude |
| `LOG_LEVEL`, `DEBUG` | Enable debug logging |
| `CODEMACHINE_DEBUG_TRIGGERS=1` | Verbose trigger directive logging |

---

## 5. P0 / P1 Findings for Monocle

### P0 — Zero tests anywhere

The entire 47K-LOC TypeScript codebase has zero automated tests. The `bunfig.toml:5` comment indicates tests were "archived to .tests.archive/ (hidden folder, ignored by bun test)" but the folder is not committed. CI verifies build + lint + typecheck only — no behavior is verified.

**Implication for monocle:** every behavioral contract in this brief is code-derived, not test-confirmed. The orchestration logic (FSM, recovery, signals) is the area most at risk; lift with caution and write tests as you go.

### P0 — `runWorkflow` blocks forever on TUI Promise

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:355-360`:

```ts
if (eventBus.hasSubscribers()) {
  await new Promise(() => {
    // Never resolves - Ctrl+C exits
  });
}
```

Monocle's headless mode needs a different exit condition — e.g., resolve when FSM reaches a final state and `eventBus` is in a "drain" mode. Don't lift verbatim.

### P0 — OpenCode is the default engine (not Claude)

`order: 1` in `opencode/metadata.ts:11`. If monocle expects "Claude Code is the assumed default", **registry default returns OpenCode**. Decide explicitly.

### P1 — Hard-coded `'claude-code'` engine ID fallback

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/run.ts:189` and `step/hooks.ts:200` use the literal `'claude-code'` while the registry ID is `'claude'`. Likely a bug or rename leftover. Don't propagate.

### P1 — `globalThis.__workflowEventBus`

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:163-170` — TUI-set global the workflow runner reads. Replace with DI.

### P1 — `workflows/mcp.ts` is dead code

`setupWorkflowMCP`, `cleanupWorkflowMCP`, `isWorkflowMCPConfigured` are exported but **not called from `runWorkflow`** or anywhere else in the codebase. The lazy `ensureMCPConfig` in `executeAgent` is the live path. Don't lift `setupWorkflowMCP`.

### P1 — Hardcoded `agentId.includes('builder')` post-step hook

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/execute.ts:177-181`. Name-sniffing for special behavior. Use a step behavior hook instead.

### P1 — Auth cache (5-min TTL) not invalidated on `clearAuth()`

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:21-48`. After `codemachine auth logout`, the cache will lie for up to 5 minutes. Confusing UX.

### P1 — Compiled-binary MCP adapter loading

Dynamic `import()` doesn't work inside Bun-compiled binaries; the writer falls back to "skip" mode (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/writer.ts:46-69`). Adapters must be pre-imported statically. Monocle's plugin system needs build-time static-import bundling if using compile-to-binary.

### P1 — Telemetry migration in flight

10+ recent commits are telemetry-related. `TELEMETRY_MIGRATION_PHASE_PLAN.md` describes a multi-phase refactor. Dual `debug()` (legacy) + `otel_debug()` (new) callsites coexist. The legacy code is being removed but isn't gone yet. Don't be surprised by partial-state code blocks.

---

## 6. Spec Crystallization Recommendations

For monocle's brief authoring, the design decisions to make explicit:

1. **Adopt `EngineModule` verbatim** OR define monocle's contract first and prove it's a superset/peer.
2. **Adopt the "headless CLI + stream-JSON" transport** OR define a different transport (SDK?, WebSocket?). The stream-JSON approach is simplest and engine-agnostic — every coding CLI is moving this direction.
3. **Decide on the MCP router pattern.** If yes, lift the router + per-step context.json filtering as a single design unit. If no, accept that per-step tool filtering isn't available out-of-the-box.
4. **Decide if workflows are first-class.** If yes, lift the template format (JS-module-with-globals) and the FSM. The 8-scenario dispatcher is more controversial — it conflates `interactive`, `autoMode`, `hasChainedPrompts` into a hidden state space.
5. **Decide directive signaling channels.** CodeMachine has 3 (process events, MCP tool calls, directive.json). Each is independently useful. Pick which the user can rely on.
6. **Decide on workflow-as-package distribution.** CodeMachine's imports system (`~/.codemachine/imports/`) is a real package manager mini-implementation. If monocle wants this, lift the manifest schema + resolution-order rules.
7. **TUI is OUT of scope.** 78 .tsx files of Solid.js are tied to the workflow event bus but not load-bearing for monocle's design.
8. **The trust model is explicit:** `--dangerously-skip-permissions` and equivalents are passed to every engine. CodeMachine acts with the user's full permissions, agents act within CodeMachine's process. Decide if monocle accepts this delegation (likely yes — it's the only practical headless model) and document it.

---

## 7. Convergence Report

| Pass | Round | Novelty | Status |
|---|---|---|---|
| 0 — Inventory | 1 | (broad) | SUBSTANTIVE — established stack and structure |
| 1 — Architecture | 1 | (broad) | SUBSTANTIVE — components, layers, topology |
| 2 — Conventions | 1 | (broad) | SUBSTANTIVE — patterns, idioms, consistency |
| 3 — Behavioral Contracts | 1 | (broad) | SUBSTANTIVE — 35 BC-DRAFTs from code |
| 4 — Verification Gaps | 1 | (broad) | SUBSTANTIVE — P0 (no tests) confirmed |
| 5 — Security & Deps | 1 | (broad) | SUBSTANTIVE — 11 security observations |
| 6 — Holdout Seeds | 1 | (broad) | SUBSTANTIVE — 8 deepening seeds identified |
| 2-deep workflows | 1 | SUBSTANTIVE | Filled scenario table + persistence + recovery + directives |
| 2-deep workflows | 2 | SUBSTANTIVE | Input providers + coordinator grammar + event bus + post-step priority |
| 2-deep harness | 1 | SUBSTANTIVE | EngineModule contract + MCP router + per-engine commands |
| 2-deep harness | 2 | SUBSTANTIVE | All 7 engines mapped + agent-coordination tools + event count corrected |
| B.5 — Coverage Audit | 1 | (audit) | 18 DEEP + 17 STRUCTURAL + 11 STUB subsystems. No P0 blind spots. |
| B.6 — Extraction Validation | 1 | (audit) | All metrics re-counted; 2 minor drifts documented (file count, event count) |

**Total LOC read in depth:** ~12,000 of 47,557 (25%). Coverage is concentrated on the user-stated deliverables (harness + workflows) — both at HIGH confidence.

**Total rounds before NITPICK declaration:** Harness pass needs 1 more nitpick round (Cursor/Mistral/Auggie runner.ts files); Workflows pass would have ~1 more round on UI/onboarding. Both subsystem models are spec-complete; further rounds would refine but not change the model.

---

## 8. File Manifest (output of this ingest)

All files written to `/Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli/`:

- `codemachine-cli-pass-0-inventory.md`
- `codemachine-cli-pass-1-architecture.md`
- `codemachine-cli-pass-2-conventions.md`
- `codemachine-cli-pass-3-behavioral-contracts.md`
- `codemachine-cli-pass-4-verification-gaps.md`
- `codemachine-cli-pass-5-security-deps.md`
- `codemachine-cli-pass-6-holdout-seeds.md`
- `codemachine-cli-pass-2-deep-workflows-r1.md`
- `codemachine-cli-pass-2-deep-workflows-r2.md`
- `codemachine-cli-pass-2-deep-harness-r1.md`
- `codemachine-cli-pass-2-deep-harness-r2.md`
- `codemachine-cli-pass-b5-coverage-audit.md`
- `codemachine-cli-pass-b6-extraction-validation.md`
- `codemachine-cli-pass-8-final-synthesis.md` (this file)

## 9. State Checkpoint

```yaml
synthesis: complete
total_files_written: 14
total_loc_written: ~5500 (semantic content; markdown framing extra)
references_grounded: 100+ file:line citations
user_deliverable_1_harness_architecture: COMPLETE
user_deliverable_2_workflows_system: COMPLETE
critical_findings:
  p0:
    - "Zero tests in repo (carries from Pass 4)"
    - "runWorkflow blocks forever on TUI promise — not safe for headless"
    - "OpenCode is the registry default, not Claude"
  p1:
    - "Hardcoded 'claude-code' fallback engine ID (real ID is 'claude')"
    - "globalThis.__workflowEventBus couples runner to TUI"
    - "workflows/mcp.ts setup functions are dead code"
    - "agentId.includes('builder') hardcoded name-sniff"
    - "Auth cache not invalidated on clearAuth"
    - "Compiled-binary MCP adapter loading needs static imports"
    - "Telemetry migration in flight; dual codepaths exist"
timestamp: 2026-05-11T00:13:00Z
```
