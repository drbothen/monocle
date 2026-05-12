# Pass 2 Deep — Harness Architecture (Round 2)

**Target:** Close remaining gaps from r1: Cursor, Auggie, Mistral command shapes; `agent-coordination` MCP server tools; `eventBus` schema; `workflows/mcp.ts` setup orchestration.

**Files read this round (not previously in depth):**
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/{cursor,auggie,mistral}/execution/commands.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/agent-coordination/tools.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/mcp.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/events/{types,event-bus}.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/{execute,hooks}.ts`

## All 7 Engines — Final Command Inventory

| Engine | Command + base args (sans model/resume) | Stream output mode | Prompt input | Resume flag |
|---|---|---|---|---|
| OpenCode | `opencode run --format json` | newline-JSON | stdin | `--session <id>` |
| Claude | `claude --print --output-format stream-json --verbose --dangerously-skip-permissions --permission-mode bypassPermissions` | newline-JSON | stdin | `--resume <id>` |
| Codex | `codex exec --json --skip-git-repo-check --sandbox danger-full-access --dangerously-bypass-approvals-and-sandbox -C <cwd>` | newline-JSON | stdin (or positional on resume) | `resume <id> <prompt>` (positional, sub-command) |
| Cursor | `cursor-agent -p --force --output-format stream-json` | newline-JSON | stdin | `--resume=<id>` (note `=`) |
| Mistral Vibe | `vibe -p '<prompt>' --auto-approve --output streaming` | newline-JSON | **positional** to `-p` | `--resume <id>` |
| Auggie | `auggie --print --quiet --output-format json` | JSON | stdin (assumed) | `--resume <id>` |
| CCR | `ccr code --print --output-format stream-json --verbose --dangerously-skip-permissions --permission-mode bypassPermissions` | newline-JSON | stdin | `--resume <id>` |

**Insights:**
- 6 of 7 engines accept prompt via stdin; Mistral is the outlier (prompt as `-p <arg>` positional).
- Cursor uses `--resume=<id>` syntax (with `=`); others use `--resume <id>` (space-separated).
- Codex uses a subcommand for resume (`exec resume`), not a flag.
- All produce newline-delimited JSON stdout (with various event schemas).
- Bypass-permission flags are explicit per engine; the same dangerous-permission concept under different names.

## Model Mapping per Engine

Each engine has its own `MODEL_MAP: Record<string, string>` that translates "common" model names to the engine's native ones. For monocle, this is a **named feature**: workflow authors specify a logical model in the workflow template, and each engine translates.

| Common name | Cursor target | CCR/Claude target | Mistral target |
|---|---|---|---|
| `gpt-5-codex` | `gpt-5-codex` | `sonnet` | `devstral-2` |
| `gpt-4` | `gpt-5` | `sonnet` | `mistral-large` |
| `gpt-3.5-turbo` | `cheetah` | `haiku` | `mistral-small` |
| `sonnet` | `sonnet-4.5` | `sonnet` (passthrough) | (no mapping) |
| `opus` | `opus-4.1` | `opus` (passthrough) | (no mapping) |

The mapping table is per-engine in `commands.ts`. Unknown models pass through OR are dropped (each engine has its own policy).

## Agent Coordination MCP Server (the second built-in server)

4 tools, exposed to every agent via the MCP router:

| Tool | Purpose | Schema highlights |
|---|---|---|
| `run_agents(script, [working_dir], [timeout_ms])` | Execute a coordinator script (single agent, parallel `&`, sequential `&&`) from inside an agent | `script` string (min 1); `timeout_ms` 1000-3600000 (default 600000=10min) |
| `get_agent_status({agent_id?, name?, status?, limit?})` | Query the SQLite monitoring DB | `status` enum: `['running','completed','failed','paused','skipped']`; `limit` 1-100 (default 10) |
| `list_active_agents()` | Convenience: all agents with status `'running'` | no args |
| `list_available_agents([working_dir])` | Discover the catalog of agent definitions (main + sub + workflows) | no args except `working_dir` |

Citations: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/agent-coordination/tools.ts:16-146`.

**Implication:** A running agent can call `run_agents` to spawn parallel/sequential sub-agents synchronously. This is **agents-launching-agents-via-tools** — different from the workflow runner's "next step" mechanism. The workflow runner has its own scheduling; `run_agents` is a side-channel for ad-hoc orchestration.

## Workflow Event Bus — full schema

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/events/types.ts:87-154` — the complete `WorkflowEvent` sum type. **34 distinct event types**, grouped:

- **Agent lifecycle (6):** `agent:{added,status,engine,model,telemetry,reset}`
- **Controller (6):** `controller:{info,engine,model,telemetry,status,monitoring}`
- **Sub-agent (4):** `subagent:{added,batch,status,clear}`
- **Triggered agent (1):** `triggered:added`
- **Workflow state (5):** `workflow:{name,status,started,stopped,view}`
- **Loop (2):** `loop:{state,clear}`
- **Checkpoint (2):** `checkpoint:{state,clear}`
- **Input state (1):** `input:state` (unified pause/chained)
- **Chained (1):** `chained:state` (deprecated; superseded by `input:state`)
- **Message (1):** `message:log`
- **Separator (1):** `separator:add`
- **Monitoring registration (1):** `monitoring:register`
- **Progress (1):** `progress:state`
- **Onboarding (8):** `onboard:{started,step,project_name,track,condition,conditions_confirmed,completed,cancelled}`

The bus is the **single channel by which the workflow runner pushes data to ANY UI**. The TUI subscribes; a CLI adapter or test harness could subscribe similarly. The bus design intentionally allows multiple subscribers and even historical replay (`historyEnabled` flag at `event-bus.ts`).

**For monocle's headless mode:** the workflow runner can be retained verbatim; only the TUI subscriber needs replacement. The bus is the seam.

## Step Execution Composition (the final mile)

`executeStep` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/execute.ts:80-189`) is the bridge from the step-level model to the agent-level model:

1. Validate `step.type === 'module'` (separators don't execute).
2. Resolve all `promptPath`s (string OR string[]): try imports first (`resolvePromptPath`), else absolute as-is, else cwd-relative.
3. Read+concatenate all prompt files with `\n\n` separator.
4. `processPromptString(rawPrompt, cwd)` — runs placeholder substitution (e.g., `{PROJECT_NAME}`, etc.) via `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/prompts/`.
5. Resolve `timeout` (from `CODEMACHINE_AGENT_TIMEOUT` env or 30 min default).
6. Call `execute(agentId, prompt, {...})` — the unified execution layer (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/execution/index.ts`). This adds telemetry forwarding, monitoring hookup, and finally calls `executeAgent`.
7. Special post-execution: if `agentId === 'agents-builder'` or name includes "builder", create `.codemachine/agents/` and `.codemachine/plan/` scaffold (`runAgentsBuilderStep`).
8. Return `{output, monitoringId, chainedPrompts}`.

**Note:** the special-case for `agents-builder` is an in-band hack — a specific agent name triggers extra filesystem setup. Worth flagging for monocle as a "don't lift" anti-pattern; better to express via a behavior hook.

## Post-Step Directive Order (deterministic)

From `afterRun()` in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/hooks.ts:242-372`, post-step directives are evaluated in **fixed priority order**:

1. **Error directive** — if `shouldStopWorkflow`, emit `workflow:error`, set status, return `{shouldBreak: true, workflowShouldStop: true}`.
2. **Trigger directive** — if `shouldTrigger`, synchronously `executeTriggerAgent` (separate engine invocation). Continues to next directive afterward.
3. **executeOnce** marker — if `step.executeOnce === true`, mark step completed in indexManager.
4. **Checkpoint directive** — if triggered, enters a 2-handler race (`process.once('checkpoint:continue')` vs `process.once('checkpoint:quit')`) and blocks until user chooses. On quit → stop workflow. On continue → `checkpointContinued: true`.
5. **Pause directive** — returns `{pauseRequested: true, pauseReason}`.
6. **Loop directive** — calls `handleLoopLogic`. If `shouldRepeat`, updates `loopCounters`, resets agent UI for re-executed steps, returns `{newIndex, newActiveLoop}`.

**Implication for monocle:** this priority order is a behavioral contract. Lifting the directive system means committing to this ordering — error > trigger > executeOnce > checkpoint > pause > loop.

## `workflows/mcp.ts` — pre-workflow setup

`setupWorkflowMCP(template, workflowDir)` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/mcp.ts:37-77`):

1. `getWorkflowEngines(template)` walks `template.steps` and collects unique `step.engine ?? defaultEngine` IDs.
2. For each engine ID, look up the registered `EngineModule`. Skip engines without `mcp.supported`, log without erroring.
3. Call `engine.mcp.configure(workflowDir)` per engine. Track configured/failed.

Then `cleanupWorkflowMCP` (lines 82-105) tears down on workflow end. `isWorkflowMCPConfigured` (lines 110-134) gates whether configuration step is needed.

**Note:** This file is currently **not called** from `runWorkflow`. The actual MCP configuration is done lazily per-agent in `executeAgent` via `ensureMCPConfig(engineId, workingDir)` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:311-314`). The `workflows/mcp.ts` functions appear to be dead/unused code OR intended for an eager-init path (commented out in `run.ts`?). I confirmed by grepping for `setupWorkflowMCP` usage and finding none in the workflow runner; the lazy path via `ensureMCPConfig` is the live path. **Flag for monocle: don't lift `setupWorkflowMCP` until verifying current intent.**

## Anti-Patterns Surfaced

| Anti-pattern | Citation | Recommendation for monocle |
|---|---|---|
| Hard-coded fallback engine id `'claude-code'` (engine actually `'claude'`) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/hooks.ts:200`; `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/run.ts:189` | Use registry default or fail fast |
| Hardcoded check for `agentId === 'agents-builder' \|\| name.includes('builder')` in step execute | `src/workflows/step/execute.ts:177-181` | Use behavior hooks on the step definition, not name-sniffing |
| `globalThis.__workflowEventBus` for TUI ⇄ runner coupling | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:163-170` | Use explicit DI |
| Unused `workflows/mcp.ts` setup functions | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/mcp.ts` | Delete or wire up; don't lift |
| `process.once('checkpoint:continue')` race | `src/workflows/step/hooks.ts:299-308` | Acceptable but ties checkpoint UI to Node EventEmitter |

## Delta Summary

- New facts:
  - All 7 engine command builders mapped (4 not previously seen: Cursor, Auggie, Mistral, plus the per-flag-shape diff)
  - Full 34-event `WorkflowEvent` sum type with 13 categories
  - `WorkflowEventBus` is the formal seam for headless mode
  - 4-tool `agent-coordination` MCP server (run_agents, get_agent_status, list_active_agents, list_available_agents)
  - `executeStep` prompt composition (multi-file concat + placeholder replacement)
  - Post-step directive priority order (error → trigger → executeOnce → checkpoint → pause → loop)
  - Dead-code `workflows/mcp.ts` setup functions
  - Hardcoded `'builder'` name-sniff in step execution
  - Hardcoded `'claude-code'` fallback engine id (real ID is `claude`)

## Novelty Assessment

Novelty: SUBSTANTIVE

Justification: This round added the agent-coordination tool set (didn't know about `run_agents` mid-workflow ad-hoc orchestration), the full event-bus schema (necessary for monocle's headless seam), the deterministic directive priority order (a behavioral contract authors depend on), and the complete engine command builders. The dead-code finding (`workflows/mcp.ts` setup paths) prevents monocle from lifting a non-working code path. Removing this round's findings would leave the spec ambiguous on the runner-to-UI seam and on directive ordering.

## Convergence Declaration

One more round needed at the workflows side (the input/mode/onboarding subsystems still aren't fully traced); the harness architecture itself is converging. Specifically harness-side:

- Open: Auggie's full execution runner (only commands.ts read; runner.ts not yet)
- Open: Per-engine MCP settings file locations beyond Claude (`opencode/mcp/settings.ts`, etc.)
- Open: `syncConfig` hook on engines — what does each implement?

These are nitpicks. The harness architecture model is converged enough to write the synthesis. I declare:

**Harness architecture sub-pass: SUBSTANTIVE this round, but expecting NITPICK on round 3.** Workflows sub-pass needs one more round on input/onboarding/coordinator (separate file).

## State Checkpoint

```yaml
pass: 2
round: 2
status: complete
target: harness-architecture
timestamp: 2026-05-11T00:09:00Z
novelty: SUBSTANTIVE
files_read_this_round: 8
loc_read_this_round: ~1700
```
