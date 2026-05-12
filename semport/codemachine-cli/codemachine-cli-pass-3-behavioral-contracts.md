# Pass 3: Behavioral Contracts — codemachine-cli

**Confidence caveat:** This codebase has **0 test files** (zero `.test.ts`, zero `.spec.ts` under `src/`, `tsconfig.json:21` excludes `tests/`, comment in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/bunfig.toml:5` says "Tests archived to .tests.archive/ (hidden folder, ignored by bun test)"). **All contracts below are derived from code, not test assertions.** Confidence levels:
- **HIGH** — behavior is explicit in a function with clear early-returns and named guards
- **MEDIUM** — behavior is composed across files; could miss an edge case
- **LOW** — inferred from log strings, naming, or partial reads

Contracts use the BC- prefix format. They are **DRAFT** — not lifted from tests.

## Engine plugin contract

### BC-DRAFT-001 — Engine registry rejects malformed modules but does not abort

**Preconditions:** Each provider's `index.ts` default-exports an object satisfying `EngineModule`.
**Postconditions:**
- If `isEngineModule(obj)` fails, the registry logs `console.warn('Invalid engine module:', engineModule)` and continues; other engines still register.
- If a duplicate `metadata.id` is registered, the registry logs `console.warn` and **skips** the second (first-write-wins).
**Error cases:** If `onRegister()` hook throws, caught by outer try/catch in `initialize()` and surfaced as `console.warn('Failed to register engine:', message)`. The remaining engines still register.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/registry.ts:46-58, 64-76`
**Confidence:** HIGH

### BC-DRAFT-002 — `getEngine(type)` falls back to default when type unspecified

**Preconditions:** Registry has at least one engine registered.
**Postconditions:**
- `getEngine()` with no arg → returns first engine by `metadata.order` (current default: OpenCode).
- `getEngine(id)` with known id → returns wrapped `DynamicEngine`.
**Error cases:** Empty registry → throws `'No engines registered'`. Unknown id → throws `'Unknown engine type: X. Available engines: …'`.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/factory.ts:23-50`
**Confidence:** HIGH

### BC-DRAFT-003 — `engine.run()` is a uniform process spawn + stream parse contract

**Preconditions:** `prompt` and `workingDir` are non-empty.
**Postconditions:** Returns `{stdout, stderr}` where `stdout` is the buffered raw output. Side effects via callbacks:
- `onSessionId(id)` called exactly once when first session id appears in stream.
- `onData(chunk)` called per formatted output fragment (after marker formatting).
- `onErrorData(chunk)` called per stderr fragment.
- `onTelemetry(parsed)` called when `result` event arrives in stream (and post-hoc reads from session file for Claude).
**Error cases:**
- Missing prompt or workingDir → `throw new Error('runClaude requires a prompt/working directory.')`.
- Exit code ≠ 0 OR captured error in stream → constructs `errorMessage`, calls `onErrorData(\n[ERROR] ${errorMessage}\n)`, then throws `Error(errorMessage)`.
- `ENOENT` (CLI not installed) → throws user-friendly install hint: `'claude' is not available on this system. Please install Claude Code first: npm install -g @anthropic-ai/claude-code`.
- `AbortSignal.abort()` → child gets SIGTERM (process group); promise rejects with `effectiveSignal.reason` (likely `AbortError`).
- Timeout (default 30 minutes / 1,800,000 ms) → AbortController fires, rejects with `'Process timed out after Xms'`.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts:173-383`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:171-402`
**Confidence:** HIGH

### BC-DRAFT-004 — Process abort kills the process **group**, not just the child

**Preconditions:** On Unix; `child.pid` is non-null.
**Postconditions:** `process.kill(-pid, 'SIGTERM')` issued. After 100ms grace, `process.kill(-pid, 'SIGKILL')`. Falls back to `child.kill('SIGTERM')` if process-group kill fails. On Windows: uses `child.kill('SIGTERM')` + 100ms `child.kill('SIGKILL')`.
**Rationale (per code comment):** "for wrapper scripts like CCR/Claude that spawn children" — process-group ensures grandchildren also die.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:131-149, 233-279`
**Confidence:** HIGH

### BC-DRAFT-005 — `killAllActiveProcesses()` reaches every spawned child

**Preconditions:** Some processes spawned via `spawnProcess`, `spawnQuiet`, or `spawnInteractive`.
**Postconditions:** Every member of the module-level `activeProcesses` set gets SIGTERM (group on Unix). After 1 second, surviving children get SIGKILL. Set is cleared.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:27, 128-169`
**Confidence:** HIGH

## Session and resume

### BC-DRAFT-006 — Claude session ID is captured from first stream event and reused for resume

**Preconditions:** Engine is `claude`; running fresh.
**Postconditions:** Parsing the stdout stream line-by-line, the first JSON line with a `session_id` field calls `onSessionId(json.session_id)`. The captured ID is persisted via `monitor.setSessionId(monitoringAgentId, sessionId)` and stored in `StepIndexManager.stepSessionInitialized(stepIndex, sessionId, monitoringId)`. On resume, `runClaude` is called with `resumeSessionId` → `buildClaudeExecCommand` adds `--resume <id>` flag and reads `resumePrompt` from stdin.
**Edge cases:**
- If JSON parse fails on a line, error is swallowed and the line is skipped (`catch { /* skip malformed */ }`).
- Telemetry is read **after** the `result` event arrives from the on-disk JSONL session file at `${claudeConfigDir}/projects/${slugifiedWorkingDir}/${sessionId}.jsonl` (last assistant message with `usage` block). This means token counts can be more accurate than what's in the live stream — but requires the session file to exist.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts:22-25, 30-69, 240-279`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:478-486`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/indexing/manager.ts` (sessionId tracking)
**Confidence:** HIGH

### BC-DRAFT-007 — Codex resume passes session id and prompt positionally

**Preconditions:** Engine is `codex`; `resumeSessionId` is truthy.
**Postconditions:** Command becomes `codex exec --json --skip-git-repo-check --sandbox danger-full-access --dangerously-bypass-approvals-and-sandbox -C <cwd> resume <session-id> <resumePrompt>`. **No model flag** is added on resume (model is fixed at conversation start).
**Caveat:** `args.push('resume', resumeSessionId, resumePrompt!)` uses non-null assertion — if `resumePrompt` is missing on resume, this is a latent bug.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/codex/execution/commands.ts:14-48`
**Confidence:** HIGH

### BC-DRAFT-008 — Resume mode does NOT mark the agent as completed

**Preconditions:** `executeAgent` called with `resumeSessionId !== undefined`.
**Postconditions:** After streaming finishes, the agent's monitoring status stays in its current state (e.g., `running` → `running`) instead of being marked `completed`. Comment: "During resume, the agent stays in running/awaiting state for continued conversation."
**Rationale:** A resumed agent is part of a conversational loop; closing the monitoring entry prevents further chained prompts.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:496-511`
**Confidence:** HIGH

### BC-DRAFT-009 — Engine fallback is one-shot per execute call

**Preconditions:** Requested engine is not authenticated.
**Postconditions:** `executeAgent` walks the registry (sorted by `order`), tries each via `authCache.isAuthenticated()`. First authenticated engine becomes the new `engineType`, `didFallback` set true. If none authenticated, falls back to `registry.getDefault()` regardless of auth state ("may still work — e.g., opencode only needs CLI installed"). If still none → throws `${engineName} authentication required and no fallback engine available`.
**Side effect:** When fallback happens, the agent's model preference is dropped (`didFallback ? undefined : agentConfig.model`) because the original model name may not be valid for the fallback engine.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:267-336`
**Confidence:** HIGH

### BC-DRAFT-010 — Auth cache TTL is 5 minutes per engine ID

**Preconditions:** `authCache.isAuthenticated(engineId, checkFn)` invoked.
**Postconditions:** Returns cached value if `now - cached.timestamp < 5*60*1000`. Otherwise calls `checkFn()`, caches result with current timestamp, returns. Cache is process-global, shared across all sub-agent invocations.
**Edge case:** Cache is not invalidated on `auth.clearAuth()`. After clearing, the user must wait up to 5 minutes for `isAuthenticated` to re-check, or restart CodeMachine.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:21-48`
**Confidence:** HIGH (cache logic); LOW (clearAuth invalidation behavior — not verified in deepening)

## Workflow lifecycle

### BC-DRAFT-011 — `runWorkflow()` is idempotent for workspace bootstrap

**Preconditions:** Called with `options.cwd` resolving to a project dir.
**Postconditions:**
- `ensureWorkspaceStructure({cwd})` runs every call — safe to call repeatedly; creates `.codemachine/` only if missing.
- `clearImportedAgents()` then `registerImportedAgents()` for each imported package — full refresh per call.
- `template.json` is created/updated with `activeTemplate` set to the resolved template filename (e.g., `ali.workflow.js`).
- `MonitoringCleanup.setup()` registers SIGINT/SIGTERM/`beforeExit` handlers.
- Debug log redirected to `.codemachine/logs/workflow-debug.log` if `DEBUG=true` or `LOG_LEVEL=debug`.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:43-86`
**Confidence:** HIGH

### BC-DRAFT-012 — On Ctrl+C, in-flight session state is persisted before exit

**Preconditions:** `MonitoringCleanup.registerWorkflowHandlers({onBeforeCleanup})` registered (always done in `runWorkflow`).
**Postconditions:** Before process exits, the cleanup callback:
1. Gets active agents from `AgentMonitorService.getInstance()`.
2. Filters to root agents (no parentId).
3. For each agent with a non-null `sessionId`:
   - If controller view active → save to `controllerConfig` via `saveControllerConfig`.
   - Else → save to `completedSteps` via `indexManager.stepSessionInitialized(stepIndex, sessionId, monitoringId)`.
**Effect:** Next `runWorkflow()` call finds resumable data via `getResumeInfo()` and resumes mid-conversation.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:88-124`
**Confidence:** HIGH

### BC-DRAFT-013 — Workflow steps are filtered by `tracks` and `conditions` at run time

**Preconditions:** Template has steps with optional `tracks: string[]`, `conditions: string[]` (AND), `conditionsAny: string[]` (OR).
**Postconditions:**
- Separator steps (`type === 'separator'`) are always included (visual dividers).
- Module steps included if **all** of:
  - No `tracks` set, OR `selectedTrack` is in `step.tracks`.
  - All `step.conditions` are in user's `selectedConditions`.
  - At least one of `step.conditionsAny` is in `selectedConditions` (if `conditionsAny` set).
- `visibleSteps` is the filtered array. `moduleSteps = visibleSteps.filter(type === 'module')` — module-count basis for indexing/timeline.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:189-222`
**Confidence:** HIGH

### BC-DRAFT-014 — FSM `running → delegated` requires (autoMode AND (hasController OR step.interactive===false))

**Preconditions:** State machine in `running`; `STEP_COMPLETE` event sent.
**Postconditions:**
- If `ctx.autoMode && !ctx.paused && (ctx.hasController || !step.interactive)` → transition to `delegated`. Sets `currentOutput`, `currentMonitoringId`, marks `continuationPromptSent = true`.
- Else → transition to `awaiting`. Sets `currentOutput`, `currentMonitoringId`.
**Rationale (per FSM comments):** "Interactive steps need a controller; non-interactive steps can be fully autonomous."
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts:151-188`
**Confidence:** HIGH

### BC-DRAFT-015 — Pause forces manual mode and is non-reversible by `RESUME` alone

**Preconditions:** Workflow in `running` or `delegated`; `PAUSE` event sent.
**Postconditions:** Transition to `awaiting`. Sets `autoMode = false`, `paused = true`. `continuationPromptSent = false`.
**To resume into autonomous:** User must send `DELEGATE` event (not `RESUME`). `RESUME` only takes machine back to `running` (still manual). To fully clear paused, set `autoMode=true` AND `paused=false` (the `setAutoMode(enabled=true)` path explicitly clears `paused` in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/index.ts:312-316`).
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts:216-228, 343-356`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/index.ts:307-317`
**Confidence:** HIGH

### BC-DRAFT-016 — Final states absorb all subsequent events silently

**Preconditions:** Machine in `completed`, `stopped`, or `error`.
**Postconditions:** Any `send(event)` is logged with `[FSM] Ignoring event %s - machine in final state %s` and no transition occurs.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts:31, 60-63`
**Confidence:** HIGH

### BC-DRAFT-017 — Crash recovery is triggered when stepData has sessionId AND no completedAt

**Preconditions:** Step data exists in `template.json` for a given step index.
**Postconditions:** `isStepResumable(stepData)` returns `true` iff `stepData.sessionId && !stepData.completedAt`. When `runStepFresh` runs, it calls `handleCrashRecovery` which (if resumable) calls `sendRecoveryPrompt` to start a resume run instead of a fresh run.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/indexing/lifecycle.ts:69-72`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/recovery/detect.ts:22-34`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/run.ts:68-93`
**Confidence:** HIGH

## Workflow signals

### BC-DRAFT-018 — User-initiated control signals are Node EventEmitter on `process`

**Preconditions:** `SignalManager.init()` has been called.
**Postconditions:** The TUI emits `process.emit('workflow:pause')` (and `:skip`, `:stop`, `:mode-change`, `:return-to-controller`). Each is handled by an async handler that:
- `pause` → calls `mode.pause()`, sets `paused=true`, sends FSM `PAUSE`.
- `skip` → calls `getAbortController()?.abort()`, marks agent skipped, sends FSM `SKIP`.
- `stop` → sets workflow status `stopped`, aborts, sends FSM `STOP`.
- `mode-change` → `setAutoMode(boolean)` based on the new mode; sends FSM `DELEGATE`/`AWAIT`.
- `return-to-controller` → switches active input provider back to controller; pauses step agent's loop.
**Cleanup:** Listeners are removed on `signalManager.cleanup()` (called from workflow `finally` block).
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/signals/manager/manager.ts:63-114, 186-194`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/signals/handlers/{pause,skip,stop,mode,return}.ts`
**Confidence:** HIGH

### BC-DRAFT-019 — Agent-initiated workflow signals are via MCP tools

**Preconditions:** Engine is configured with MCP via `engine.mcp.configure()` and routed to CodeMachine's `workflow-signals` MCP server.
**Postconditions:** Agent (running as child CLI process) calls one of three MCP tools:
- `propose_step_completion(step_id, artifact_path, [artifact_hash], checklist, [open_questions], confidence)` → server validates with Zod, computes SHA-256 if not provided, emits to `signalQueue` (file-backed at `~/.codemachine/mcp/workflow-signals/`). Returns text confirmation including `Checklist: X/Y items complete`.
- `approve_step_transition(step_id, decision, [blockers], [notes])` → validates decision in `{approve, reject, revise}`. If `revise`, requires non-empty `blockers`. Emits to queue. Returns text confirmation with status emoji marker like `[APPROVED]`, `[REJECTED]`, `[REVISION REQUESTED]`.
- `get_pending_proposal()` → returns the latest proposal from queue, formatted with checklist as markdown.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/index.ts:83-247`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/tools.ts:16-66, 74-116, 123-134`
**Confidence:** HIGH

### BC-DRAFT-020 — Directive file is one-way signaling from agent to runner

**Preconditions:** Agent writes JSON to `.codemachine/memory/directive.json` (created by tools the runner exposes — see Pass B for MCP tool details).
**Postconditions:** On user Enter (advance), `evaluateOnAdvance(cwd)` reads the file and returns an action: `{advance|loop|stop|error|checkpoint|pause|trigger}`. The runner then dispatches to the matching directive handler. After processing, `resetDirective(cwd)` writes `{action: 'continue'}` back.
**Edge cases:**
- Missing file → `null` → treated as `'continue'` → action `'advance'`.
- Invalid JSON → logged via `console.error`, returns `null`. Workflow advances normally.
- `'trigger'` action without `triggerAgentId` → degraded to `'advance'`.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/directives/reader.ts:13-66`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/directives/onAdvance.ts:24-49`
**Confidence:** HIGH

## Template lifecycle

### BC-DRAFT-021 — Template loading busts the import cache per call

**Preconditions:** `loadWorkflowModule(modPath)` called.
**Postconditions:**
- For `.cjs`/`.cts`: deletes `require.cache[require.resolve(modPath)]` then re-`require()`.
- For `.js`/`.mjs`: appends `?ts=<timestamp>` to the file URL, forcing ESM cache miss.
- Returns `mod.default ?? mod`.
**Rationale:** Allows hot-reload during dev/test without restarting the process.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/loader.ts:16-34`
**Confidence:** HIGH

### BC-DRAFT-022 — Template validation produces aggregated errors, not first-fail

**Preconditions:** `validateWorkflowTemplate(value)` called.
**Postconditions:** Returns `{valid: false, errors: [string]}` with ALL detected errors (not just the first). Validator iterates each step, each conditionGroup, each child group, and collects errors. `valid` is `errors.length === 0`. Error messages identify the offending path like `Step[3].agentId must be a string` or `conditionGroups[1].children.full-workflow.conditions.brainstorming.label must be a string`.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/validator.ts:20-260`
**Confidence:** HIGH

### BC-DRAFT-023 — Chained prompts honor frontmatter > filename fallback

**Preconditions:** `loadChainedPrompts(path, projectRoot, selectedConditions, selectedTrack)` called.
**Postconditions:** For each `.md` file:
- Parse YAML frontmatter (`---`-delimited block at top, hand-rolled parser; only `name` and `description` keys supported, with optional quote-stripping).
- If frontmatter `name` present → `name = frontmatter.name`. Else `name = filename without .md`.
- If frontmatter `description` present → `label = frontmatter.description`. Else `label = TitleCased filename without 'NN-' prefix`.
- Body (post-frontmatter) is the `content`.
**Path filtering:**
- Conditional path entries (`{path, conditions?, conditionsAny?, tracks?}`) are filtered: a path is included iff all of `conditions` are in user's selected set AND at least one of `conditionsAny` is in selected set (if specified) AND `selectedTrack` is in `tracks` (if specified).
- String entries are always included.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/chained.ts:32-66, 234-259, 279-313`
**Confidence:** HIGH

## Pre-flight & onboarding

### BC-DRAFT-024 — Specification file is required iff template declares it

**Preconditions:** `template.specification === true`.
**Postconditions:** `validateSpecification(specificationPath)` is called. If file missing/empty → throws `ValidationError`. Default path: `.codemachine/inputs/specifications.md` (overridable via `CODEMACHINE_SPEC_PATH` env var).
**If template.specification !== true:** spec check is skipped entirely.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/preflight.ts:101-122`
**Confidence:** HIGH

### BC-DRAFT-025 — Onboarding asks only for selections the template needs

**Preconditions:** `checkOnboardingRequired({cwd})` called.
**Postconditions:** Returns `{needsProjectName, needsTrackSelection, needsConditionsSelection, needsControllerSelection, controllerAgents, template}`.
- `needsTrackSelection = template.tracks?.options has keys AND !selectedTrack`.
- `needsConditionsSelection = template.conditionGroups?.length > 0 AND !hasSelectedConditions`.
- `needsControllerSelection = false` (deprecated; controller is set via `controller()` function in template).
- `needsProjectName = false` (TODO comment: temporarily disabled due to persistence bug — `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/preflight.ts:76-77`).
**`needsOnboarding(needs)`:** boolean OR of the four flags.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/preflight.ts:51-91, 143-150`
**Confidence:** HIGH

## Cleanup and lifecycle

### BC-DRAFT-026 — TUI session blocks the process on a never-resolving Promise

**Preconditions:** `runWorkflow` reaches its end; `eventBus.hasSubscribers()` is true.
**Postconditions:** Awaits `new Promise(() => {})` which never resolves. Only Ctrl+C ends the process.
**Implication:** The workflow runner does not own the process lifetime; the TUI does. For headless mode, this would need a different exit condition.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:355-360`
**Confidence:** HIGH

### BC-DRAFT-027 — Home directory is a hard veto on launch

**Preconditions:** Resolved target cwd is `realpath`-equal to user `$HOME`.
**Postconditions:** Prints multi-line styled error and `process.exit(1)`. User cannot bypass without `--dir <project>`.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:77-117`
**Confidence:** HIGH

### BC-DRAFT-028 — Telemetry init is fully opt-in; default boot path avoids OTel cost

**Preconditions:** Env var `CODEMACHINE_TRACE` is unset OR equal to `''`, `'0'`, or `'false'`.
**Postconditions:** `telemetryRequested` is false. Three shutdown handlers are no-op async functions. `tracingConfig` and `metricsEnabled` stay null/false. **No imports of OTel SDKs occur** in the boot path. The application still calls `otel_info/warn/error` everywhere, but the underlying logger is a console-fallback (verified in `shared/logging/otel-logger.ts` — read deeper in Pass B).
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:27-64`
**Confidence:** HIGH

### BC-DRAFT-029 — Mod-of-flight engine telemetry is read from session JSONL post-result

**Preconditions:** Engine is `claude`; `onTelemetry` callback provided; `result` event arrives.
**Postconditions:** `readSessionTelemetry(sessionPath)` reads the JSONL file, walks backward to find the last `assistant` message with `usage` block, returns `{tokensIn, tokensOut, cached, cacheCreationTokens, cacheReadTokens}`. The `tokensIn` field is **the sum** `input_tokens + cache_creation_input_tokens + cache_read_input_tokens` per Anthropic's prompt-caching docs.
**Edge case:** If session file is missing or unparseable → returns null → `onTelemetry` not called.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts:22-70, 261-268`
**Confidence:** HIGH

## Spawn and timeout

### BC-DRAFT-030 — Default engine timeout is 30 minutes

**Preconditions:** `engine.run({...})` invoked without explicit `timeout`.
**Postconditions:** Default 1,800,000 ms applied. Internal `AbortController` fires `'Process timed out after 1800000ms'` on expiry.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts:174, 84` (default param)
**Confidence:** HIGH

### BC-DRAFT-031 — `spawnQuiet` defaults to 10s timeout (CLI checks)

**Preconditions:** `spawnQuiet(command, args)` invoked without `timeout`.
**Postconditions:** 10,000 ms default. On timeout, rejects with `Error('Timeout')`.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:66, 80-82`
**Confidence:** HIGH

### BC-DRAFT-032 — MCP tool timeouts are explicitly set to 15 minutes for Claude

**Preconditions:** Claude engine starting a fresh run.
**Postconditions:** `mergedEnv` includes `MCP_TIMEOUT='900000'` and `MCP_TOOL_TIMEOUT='900000'` (per code comment: "for long-running agent coordination").
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts:192-196`
**Confidence:** HIGH

## Coordinator and multi-agent

### BC-DRAFT-033 — `codemachine run <script>` parses both single and composite syntax

**Preconditions:** `run` command invoked with a script string.
**Postconditions:** `CoordinatorService.execute(script, {workingDir})` parses:
- Single agent: `agent-id 'prompt'` → simple execution.
- Sequential composition: `a && b && c` → strict left-to-right; later commands skipped on prior failure.
- Parallel composition: `a & b & c` → spawned in parallel; awaits all.
- Mixed: `a && b & c` → sequential AND with parallel branches.
- Input file annotations: `agent[input:file1.md;file2.md,tail:100] 'prompt'` → reads files and tails N lines as part of prompt.
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/cli/commands/run.command.ts:47-115`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/coordinator/{parser,service,execution}.ts` (parser logic deepened in Pass B)
**Confidence:** MEDIUM (composite parsing complexity warrants deeper read in Pass B)

## Monitoring

### BC-DRAFT-034 — Monitoring registers every agent execution to a SQLite row

**Preconditions:** `executeAgent` called with `disableMonitoring !== true`.
**Postconditions:** A row is inserted into `agents` table with: `name`, `engine`, `status='running'`, `parent_id`, `pid` (null initially), `start_time` (ISO), `prompt` (truncated display version), `log_path`, `engine_provider`, `model_name`. The auto-generated `id` is the `monitoringAgentId` returned to caller. Telemetry row in `telemetry` table is updated as engine reports usage.
**On completion:** `monitor.complete(id)` or `status.complete(id)` sets `end_time`, computes `duration`, sets `status='completed'`. Or `markRunning/markPaused/markSkipped/markFailed` for other states.
**Status state machine:** `running → {completed, failed, paused, skipped}` (SQL CHECK constraint).
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/monitoring/db/schema.ts:3-36`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:376-401`
**Confidence:** HIGH

### BC-DRAFT-035 — `session_id` column is added via runtime migration

**Preconditions:** Existing pre-`session_id` database file present.
**Postconditions:** On `initSchema`, the code runs `PRAGMA table_info(agents)` and `ALTER TABLE agents ADD COLUMN session_id TEXT` if the column is missing. Migration is best-effort (`try/catch` swallowing errors).
**Evidence:** `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/monitoring/db/schema.ts:38-51`
**Confidence:** HIGH

## Known Bugs / Concerns Surfaced

| Issue | Severity | Citation |
|---|---|---|
| Hard-coded fallback `'claude-code'` engine id in `runStepFresh` while registry id is `'claude'`. | MED — may silently misroute step engine on directive eval | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/run.ts:189` |
| Auth cache (5-min TTL) is not invalidated on `auth.clearAuth()`. User who logs out via CLI may see "logged in" for up to 5 minutes. | LOW — confusing UX | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:21-48` |
| `runWorkflow` blocks on `new Promise(() => {})` forever (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:355-360`). The function never returns in TUI mode; only Ctrl+C exits. | DESIGN — by design; problematic for headless invocation | `src/workflows/run.ts:355-360` |
| `runWorkflow` reads `globalThis.__workflowEventBus` — couples to TUI's global init. | DESIGN — testability hurdle | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts:163-170` |
| `needsProjectName = false` TODO ("temporarily disabled due to persistence bug"). | LOW — feature gap, not a bug | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/preflight.ts:76-77` |
| `args.push('resume', resumeSessionId, resumePrompt!)` — non-null assertion on `resumePrompt` in Codex. | LOW — latent crash if resume called without prompt | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/codex/execution/commands.ts:41` |
| Massive commented blocks in `cli-setup.ts` (lines 6-21, 152-194, 269-271, 309-320). | LOW — telemetry migration in flight; should be cleared post-migration | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts` |
| `bunfig.toml` references `.tests.archive/` — tests were intentionally archived. Reading them in deepening would give historical context but not current coverage. | INFO | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/bunfig.toml:5` |

## State Checkpoint

```yaml
pass: 3
status: complete
contracts_drafted: 35
confidence_high: 30
confidence_medium: 4
confidence_low: 1
test_coverage: ZERO (no test files anywhere)
timestamp: 2026-05-11T00:03:00Z
next_pass: 4
notes: |
  Contracts grounded in code-reading. All HIGH-confidence ones come from
  clear early-returns or explicit FSM transitions. The MEDIUM (coordinator
  parsing) and LOW (auth cache invalidation) are flagged for Pass B deepening.
  P0 carry: zero tests means every contract above could fail in production
  scenarios not exercised by code review.
```
