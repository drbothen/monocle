# Pass 2 Deep — Workflows Engine (Round 2)

**Target:** Close remaining workflows gaps from round 1: step composition, input providers, coordinator parser grammar, event bus emitter.

**Files read this round:**
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/{execute,hooks}.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/input/types.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/events/{types,event-bus}.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/coordinator/{parser,service,types}.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/mcp.ts`

## Input Provider Contract

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/input/types.ts:56-83`:

```ts
interface InputProvider {
  readonly id: string;
  getInput(context: InputContext): Promise<InputResult>;
  activate?(): void;
  deactivate?(): void;
  abort?(): void;
}

type InputResult =
  | { type: 'input'; value: string; resumeMonitoringId?: number; source?: 'user' | 'controller' }
  | { type: 'skip' }
  | { type: 'stop' };
```

Implementations:
- `UserInputProvider` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/input/providers/user.ts`, 205 LOC) — TUI keyboard input via the `inputEmitter` pub-sub
- `ControllerInputProvider` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/input/providers/controller.ts`, 319 LOC) — pulls from controller agent's stream output via resume sessions; can also fall back to user via `getUserInput()`

`InputContext` carries `{step, stepOutput, stepIndex, totalSteps, promptQueue, promptQueueIndex, cwd, uniqueAgentId?}` so providers can render context-aware UX.

The `__SWITCH_TO_MANUAL__` / `__SWITCH_TO_AUTO__` magic strings (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/modes/interactive.ts:97-110`) are sentinel return values that signal mode-switch intent. Slightly hacky but functional.

## Coordinator Script Grammar (full)

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/coordinator/parser.ts:1-464` — hand-rolled quote-aware parser.

### Token / construct table

| Form | Meaning | Example |
|---|---|---|
| `agent-name 'prompt text'` | Single agent with prompt | `code-gen 'Build login feature'` |
| `agent-name 'prompt'` (single quotes) OR `agent "prompt"` (double quotes) | Either quote style works | |
| `agent-name` (no quotes/prompt) | Agent with no extra prompt (template-only) | `arch-writer` |
| `agent[options]` | Enhanced syntax with options | `system-analyst[input:spec.md,tail:100] 'analyze'` |
| `agent[input:file.md;file2.md]` | Pre-pend input file content to the prompt | `agent[input:a.md;b.md]` |
| `agent[tail:N]` | Limit agent output to last N lines | `agent[tail:50]` |
| `agent[prompt:"text"]` | Embed prompt inside options block | `agent[prompt:"do X"]` |
| `agent[key:value]` | Extensible custom options → stored on `command.options.<key>` | |
| `a & b & c` | Parallel execution (no `&&` present) | spawns 3 in parallel |
| `a && b && c` | Sequential execution | strict left-to-right, halts on first failure |
| `a && b & c` mixed | `&&` splits into sequential groups; within each group, `&` = parallel | groups = [{seq:[a]}, {par:[b,c]}] |

### Parsing precedence

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/coordinator/parser.ts:21-48`:

1. If `script.includes('&&')` AND any split-part contains `&` → mixed mode.
2. Else if `script.includes('&&')` → sequential.
3. Else if `script.includes('&')` → parallel.
4. Else → single command.

**Important nuance:** the parser uses **quote-aware splitting** (`smartSplit`, `smartSplitMultiChar`, `containsOutsideQuotes`). Apostrophes within prompts are heuristically distinguished from closing quotes by looking at the preceding/following character (line 207-218): a closing quote must be followed by space, tab, end-of-string, or the delimiter. This is good-enough but brittle for prompts like `'Bob's plan'` — the `s` apostrophe is preceded by `b`, the closing `'` is followed by ` `, so it parses correctly. But `"can't 'a' stop"` mixes both — may produce surprises.

### Parser anti-pattern note

The parser doesn't support parentheses (line 11-15 explicitly notes "For MVP, we'll support simpler syntax without parentheses"). The mixed-mode interpretation `a && b & c && d` is unambiguous only by convention: `&&` at top, `&` within each `&&`-segment.

## Coordinator Service Lifecycle

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/coordinator/service.ts:45-125`:

1. Parse via `CoordinatorParser.parse(script)` → `CoordinationPlan { groups: CommandGroup[] }`.
2. Resolve parent context for monitoring hierarchy:
   - Check `CODEMACHINE_PARENT_AGENT_ID` env var (set by `executeAgent` for child processes).
   - If not present, infer from `AgentMonitorService.getActiveAgents()` — pick most-recently-started.
3. Pass parent to `CoordinationExecutor` — agents register directly under that parent (no separate "coordination session" parent).
4. `executor.execute(plan)` → iterates groups, parallel within group via `Promise.all`, sequential between groups.
5. Print summary unless `silent: true` (used in MCP/headless calls via `run_agents` tool).

Coordinator does NOT participate in the workflow FSM. It's a parallel side-channel for the `codemachine run` CLI command AND for agent-initiated coordination via the `run_agents` MCP tool.

## Workflow Event Bus + Emitter (the seam)

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/events/event-bus.ts:51-...` provides:

- `subscribe(listener)` → all events
- `on('agent:status', typedListener)` → specific event with full type narrowing
- `emit(event)` — adds to history if `historyEnabled`, broadcasts to all + type-specific listeners

The `WorkflowEventEmitter` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/events/emitter.ts`, 503 LOC) is a typed facade over the bus: methods like `emitter.workflowStarted(name, totalSteps)` translate to `bus.emit({type: 'workflow:started', workflowName, totalSteps})`.

**Headless-mode adapter:** to lift the workflow runner without the TUI, replace the Solid.js subscribers with a custom listener that prints, logs, or surfaces events programmatically. The bus IS the API; no other coupling exists.

## Step Execution Composition (recap)

Confirmed:
- Multiple `promptPath`s are concatenated with `\n\n`.
- `processPromptString(rawPrompt, cwd)` substitutes placeholders (e.g., `{PROJECT_NAME}`).
- The combined prompt is then sent to `execute(agentId, prompt, {...})` which adds telemetry/monitoring and invokes `executeAgent`.
- `executeAgent` writes the MCP context file BEFORE invoking the engine (so router-side filtering applies for tools/list and tools/call).
- Default timeout is **30 min** per step (overridable via `CODEMACHINE_AGENT_TIMEOUT` env or step-level `timeout`).
- Hardcoded special-case: agents named `agents-builder` or whose name contains "builder" trigger `runAgentsBuilderStep(cwd)` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/execute.ts:177-181`) — creates `.codemachine/{agents,plan}` dirs. Genuine wart.

## `workflows/mcp.ts` — confirmed dead code

`setupWorkflowMCP`, `cleanupWorkflowMCP`, `isWorkflowMCPConfigured` are exported but **not called from `runWorkflow`** or any other consumer I found. The live path is `executeAgent → ensureMCPConfig` per-step. These functions appear to be vestigial from an earlier eager-init design. For monocle: don't lift `setupWorkflowMCP`; use the lazy per-agent path.

## Delta Summary

- New facts:
  - `InputProvider` contract with `getInput/activate/deactivate/abort` + `InputResult` sum
  - Two implementations (UserInputProvider, ControllerInputProvider) with magic-string mode-switch sentinels
  - Complete coordinator-script grammar with quote-aware parsing rules
  - Coordinator script supports `[input:files,tail:N,prompt:text,key:custom]` enhanced syntax
  - Mixed-mode interpretation: `&&` as outer groups, `&` as inner-parallel
  - Parent-ID inference for monitoring hierarchy (env > active-agents-list)
  - `workflows/mcp.ts` is dead code (confirmed via grep — not called anywhere)
  - `WorkflowEventEmitter` is a typed facade over `WorkflowEventBus` (503 LOC of helper methods)

## Novelty Assessment

Novelty: SUBSTANTIVE

Justification: The coordinator parser grammar is a documented API that monocle workflow authors would need to either lift or reject. The dead-code finding prevents wasted effort. The input-provider contract is necessary for monocle's headless mode story. Removing this round would leave the spec without the coordinator grammar and the headless seam.

## Convergence Declaration

Round 3 would be NITPICK on workflows. Remaining gaps:
- Full per-route TUI behavior (78 .tsx files) — but these are UI-only and don't change the spec.
- Onboarding service details — `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/onboarding/service.ts` has the onboarding flow but it's UX-only.
- `coordinator/execution.ts` 339 LOC — the actual parallel/sequential execution loop; verifies but doesn't change contracts.

These are nitpicks. **Workflows pass converges next round** if needed, but at the spec-relevant level, the model is complete.

## State Checkpoint

```yaml
pass: 2
round: 2
status: complete
target: workflows
timestamp: 2026-05-11T00:10:00Z
novelty: SUBSTANTIVE
files_read_this_round: 9
loc_read_this_round: ~1800
```
