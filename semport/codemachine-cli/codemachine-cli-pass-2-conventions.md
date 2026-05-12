# Pass 2: Conventions & Pattern Catalog — codemachine-cli

## Language & Tooling Conventions

| Convention | Rule | Evidence |
|---|---|---|
| TS strict mode | `"strict": true` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/tsconfig.json:11` |
| ESM only | `"type": "module"` + `.js` extension in imports (`.js`-paths-for-ts) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:26`; every internal import uses `.js` even though source is `.ts` |
| Module resolution | `"Bundler"` with custom `"browser"` condition | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/tsconfig.json:5-6` — TUI uses Solid via the `browser` condition |
| Import alias | `@tui/*` → `./src/cli/tui/*` (paths) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/tsconfig.json:18-20` |
| Indent | 2 spaces, LF endings, UTF-8 | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/.editorconfig:5-10` |
| Lint | ESLint 9 flat config, `--max-warnings=0` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:58`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/eslint.config.js` |
| `_`-prefix for unused vars | enforced via `argsIgnorePattern: '^_'` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/eslint.config.js:36-43` |
| Bun built-in imports | `bun:test`, `bun:sqlite` allowed without resolver | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/eslint.config.js:45-50` |
| Pre-commit | Husky 9 | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:67`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/CONTRIBUTING.md` |
| CI matrix | 5 platforms; `--frozen-lockfile`; cross-arch via `--target` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/.github/workflows/build.yml:13-29` |
| Release flow | `release` event → `bun scripts/publish.ts --tag latest` → publishes main + 5 optional binaries | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/.github/workflows/publish.yml`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/scripts/publish.ts` |

## Naming Conventions

| Element | Convention | Examples |
|---|---|---|
| Files | `kebab-case.ts`; multi-word filenames preferred over deep nesting | `cli-setup.ts`, `agent-coordination/`, `auto-import.ts` |
| Folders | `kebab-case`; flat where possible, but workflow + TUI go deep when needed | `src/workflows/runner/modes/`, `src/cli/tui/routes/workflow/components/modals/` |
| `index.ts` per folder | Each substantial folder has a barrel `index.ts` that re-exports public surface | every subfolder under `src/workflows/`, `src/infra/engines/core/`, etc. |
| Types files | `types.ts` colocated; sometimes a per-file local types definition | `state/types.ts`, `runner/types.ts`, `runner/modes/types.ts` |
| Classes | `PascalCase` | `WorkflowRunner`, `SignalManager`, `StepIndexManager`, `EngineAuthCache`, `WorkflowMode`, `DynamicEngine` |
| Interfaces / type aliases | `PascalCase`; no `I`-prefix | `EngineModule`, `WorkflowState`, `RunnerContext`, `ChainedPrompt` |
| Functions | `camelCase`; verb-first | `runCodemachineCli`, `executeAgent`, `buildClaudeExecCommand`, `loadChainedPrompts`, `evaluateOnAdvance` |
| FSM events | `SCREAMING_SNAKE_CASE` strings | `'START'`, `'STEP_COMPLETE'`, `'INPUT_RECEIVED'`, `'DELEGATE'`, `'AWAIT'` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/types.ts:28-37`) |
| FSM states | `lowercase` strings | `'idle'`, `'running'`, `'awaiting'`, `'delegated'`, `'completed'`, `'stopped'`, `'error'` |
| Engine IDs | `lowercase` single-word | `'claude'`, `'codex'`, `'cursor'`, `'opencode'`, `'auggie'`, `'mistral'`, `'ccr'` |
| Environment variables | `SCREAMING_SNAKE_CASE`, `CODEMACHINE_*` prefix for app-owned | `CODEMACHINE_CWD`, `CODEMACHINE_TRACE`, `CODEMACHINE_PARENT_AGENT_ID` |
| Process events | `workflow:<verb>` lowercase with colon | `workflow:pause`, `workflow:skip`, `workflow:stop`, `workflow:mode-change`, `workflow:return-to-controller`, `workflow:error`, `workflow:input-received` |
| Internal/private fields | `_camelCase` underscore prefix on private getters' backing fields | `_currentStepIndex`, `_promptQueue`, `_promptQueueIndex`, `_currentSession` |
| TODOs | `// TODO: <category> - <reason>` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:6, 152, 173, 269` |

## Module Organization Patterns

### Barrel exports

Every domain folder has an `index.ts` that **re-exports** the public surface, while internal modules are imported by relative path from siblings within the folder. Example: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/index.ts` is the canonical workflow entry; siblings use `from './directives/index.js'` etc.

### Provider symmetry (template pattern)

Every engine provider folder under `src/infra/engines/providers/<id>/` has identical structure (see Pass 1 table). Adding a new provider is mechanical: copy the layout, swap CLI binary name, command-builder, and stream parser. This is the most explicit "lift template" in the codebase.

### Per-folder `types.ts` colocation

When a folder owns a public type surface, types live in a sibling `types.ts` (NOT in an `interfaces/` subfolder, NOT inline in the implementation file). Examples: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/types.ts`, `runner/types.ts`, `templates/types.ts`, `directives/types.ts`.

### "Single source of truth" comments

Code intentionally documents which module owns a piece of state to prevent drift. Examples:

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/types.ts:5-8` — "Queue state (promptQueue, promptQueueIndex) is managed by StepIndexManager. See src/workflows/indexing/ for the single source of truth."
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/indexing/manager.ts:15-23` — "This class is the single source of truth for: current step index, prompt queue state, step completion status, resume information."
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/mode/` is explicitly the source of truth for `autoMode` and provider selection (see comments at `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/index.ts:128-133`).

### Two-pass dynamic imports for boot perf

Top-level files use `await import('…')` instead of static `import` for non-critical dependencies. This is a performance pattern (lazy-load until the boot span needs it). Cited extensively in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:42-44, 49-51, 58-59, 71-83, 245-258, 296-298, 322-330`.

### Manual JSDoc, no schema/decorators

Documentation is plaintext JSDoc on public functions and classes, no decorators, no automatic schema generation outside MCP tools (where JSON Schema is mandated by MCP). Zod is reserved for MCP server input validation only (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/index.ts:34-36, 85-86, 129`).

## Design Patterns in Use

### Plugin Registry (Strategy)

Three independent registries follow the same shape:

1. **Engine registry** — `EngineRegistry` singleton, `register()` / `get()` / `getAll()` (sorted by `order`) / `has()` / `getDefault()`. Plugins implement `EngineModule`, statically imported and registered on module load. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/registry.ts:20-128`.
2. **MCP adapter registry** — `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/registry.ts` (read deeper in Pass B). Adapters self-register at import time per comment in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/types.ts:35-37`.
3. **Mode handler registry** — `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/modes/index.ts` — `getModeHandler(modeType)` selects `interactiveHandler` / `autonomousHandler` / `continuousHandler`.

Identical shape: each registry has a `Map<string, T>`, sorted accessors, type-guard `isX()`, and identity-based deduplication.

### Finite State Machine (data-driven config)

`createMachine(config)` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts:26-115`) is a **generic** machine. The workflow-specific FSM is built via `createWorkflowMachine(ctx)` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts:120-377`), which passes a declarative `MachineConfig` with states, transitions, guards, actions, and on-enter/on-exit hooks. No XState dependency — it's hand-rolled.

### Event Bus (pub-sub) + EventEmitter

Two parallel pub-sub stacks:
1. `WorkflowEventBus` (custom) — TUI ↔ runner UI events (timeline, status, log lines).
2. Node `process.on('workflow:*')` — control signals (pause/skip/stop/mode-change). Faster, signal-handler safe.

This dual layering is deliberate: `process` events handle user-control signals where TUI components emit directly to the runner; `WorkflowEventBus` handles structured updates flowing back. See `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/signals/manager/manager.ts:60-114`.

### Factory + Dynamic Wrapper

`createEngine(type)` wraps an `EngineModule` (data) in a `DynamicEngine` (behavior) implementing the `Engine` interface. The wrapper class is anonymous-ish, just bridges the two contracts. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/factory.ts:8-50`.

### Streaming Callbacks (`on*` style)

Every long-running operation accepts named-callback options instead of returning a stream. `EngineRunOptions` has `onData`, `onErrorData`, `onTelemetry`, `onSessionId` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/types.ts:28-32`). `spawnProcess` has `onStdout`, `onStderr` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:9-10`). This avoids stream-plumbing complexity and makes consumers explicit about what they want.

### Effect-free hot path / fire-and-forget side effects

Telemetry updates are explicitly **non-blocking**: `monitor.updateTelemetry(...).catch(err => …)` is fire-and-forget so stream callbacks don't await DB writes (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:466-471`, `481-485`). Same pattern in update checks (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:298-307`).

### Polymorphic input via "scenarios"

The runner's mode dispatch is via a numbered scenario table (1-8) that maps `(interactive, autoMode, hasChainedPrompts) → (modeType, inputSource)`. Each mode handler advertises which scenarios it handles via `mode.scenarios: number[]` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/modes/interactive.ts:157` shows `scenarios: [1, 2, 3, 4, 7, 8]`; `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/modes/autonomous.ts:90` shows `[5]`). Resolved by `resolveScenario()` in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/scenarios/`.

### Idempotent shutdown

Boot phase's OTel shutdown functions default to `async () => {}` no-ops, then are reassigned only if telemetry was initialized (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:31-44`). This makes the cleanup chain always safe to call.

### Lazy + cached auth

`EngineAuthCache` (5-min TTL) wraps `auth.isAuthenticated()` calls to prevent the "5-minute delay bug" when spawning many sub-agents (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:21-48`). Cache is global to the process — appropriate because the cost of being wrong (re-prompt at next call) is small.

### Process group kills

For long-running CLIs that internally spawn wrapper scripts (CCR, Claude), CodeMachine kills via `process.kill(-pid, …)` to target the process group instead of just the immediate child (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:233-255`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:131-149`). Windows fallback uses child.kill.

### Span-wrap-everything (tracing pattern)

Boot phases nest spans using `withSpan` / `withRootSpan` / `startManualSpanAsync` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/tracing/tracers.ts`). Every named phase produces an attribute-tagged span; trace tree IS the code structure.

## Engine-Specific Command Patterns

Every provider has its own conventional CLI flags, but the **shape** is identical: `buildXExecCommand` returns `{command, args}` plus stdin handling.

| Engine | Stream output flag | Non-interactive flag | Resume mechanism | Citation |
|---|---|---|---|---|
| Claude | `--output-format stream-json --verbose` | `--print` + `--dangerously-skip-permissions --permission-mode bypassPermissions` | `--resume <session-id>`, prompt via stdin | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/commands.ts:49-75` |
| Codex | `--json` | `exec --skip-git-repo-check --sandbox danger-full-access --dangerously-bypass-approvals-and-sandbox -C <cwd>` | `codex exec resume <session-id> <prompt>` (positional args), stdin prompt for new | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/codex/execution/commands.ts:14-48` |
| Others | Each per-provider commands.ts (read in Pass B) | | | |

This means: **CodeMachine's harness contract is a flag set, not an API**. Each engine's CLI must support some combination of `print-mode`, `stream-json output`, `skip-permission-prompts`, `model selection`, and `resume-by-session-id`.

## Prompt File Convention

Chained prompts are markdown files with optional YAML frontmatter:

```markdown
---
name: step-01-discovery
description: Discover user's project context and goals
---
You are a discovery agent. Begin by asking…
```

Parser: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/chained.ts:32-66`. Frontmatter is hand-rolled (not a YAML library). Only `name` and `description` keys are honored. Files are sorted alphabetically (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/chained.ts:162-163`) — convention is `NN-slug.md` numbered prefix.

Folder example: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/prompts/templates/ali/chained/step-{00..05}-*.md`.

## Workflow Template Convention

Templates are ESM JS modules (NOT JSON, NOT YAML). They `default export` a `WorkflowTemplate` object. Two helpers are injected as globals at template load: `resolveStep(agentId, opts)` and `resolveFolder(folderName, opts)` (and `resolveModule(...)`), via `ensureTemplateGlobals()` in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/globals.ts`. This lets templates be terse:

```js
export default {
  name: 'Example',
  steps: [
    resolveStep('arch-agent'),
    resolveStep('git-commit', { engine: 'claude', executeOnce: true }),
    ...resolveFolder('codemachine'),
  ],
  subAgentIds: ['frontend-dev', 'backend-dev'],
};
```

Citations: `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/templates/workflows/_example.workflow.js:5-121`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/templates/workflows/ali.workflow.js:1-100`.

Validation is a hand-rolled function returning `{valid, errors}` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/validator.ts:20-260`) — not Zod, intentionally because templates are JS modules and Zod would force a heavier transform.

## ANSI/Output Marker Convention

All engine output passes through `outputMarkers.ts` formatters before being sent to the TUI or written to log files. Markers are colon-prefixed sentinels (e.g., `__CM_THINKING_START__`) that the TUI re-renders via `renderToChalk()` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts:438-444`). The log file gets the **stripped** form via `formatForLogFile()` for grep-friendliness.

## Error Handling Idioms

- Throw `Error` with descriptive message; let it bubble.
- Custom error classes only for distinct categories (e.g., `MCPConfigError`).
- `try/catch` at top of every async I/O call; rethrow with context message: ```Failed to load template ${rel}: ${e instanceof Error ? e.message : String(e)}``` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/loader.ts:69`).
- `error.name === 'AbortError'` is **always** treated as controlled cancellation, never a failure. Caught throughout the runner.
- Process-level: `try { … } catch { exitCode = 1 } finally { …shutdown… process.exit(exitCode) }`.

## Documentation Idioms

- File-level header comment with role description for non-trivial modules (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts:1-10`).
- JSDoc on every exported function with @param, brief body.
- Inline `debug('[Component] message: %s', value)` traces are everywhere — search for `debug(` shows hundreds of call sites.
- Architectural decisions are encoded in comments near the structures they describe (e.g., "Single source of truth" comments above).
- README + CONTRIBUTING are user-facing; internal architecture is undocumented outside code comments and the `TELEMETRY_MIGRATION_PHASE_PLAN.md` (30KB doc) at repo root.

## Consistency Assessment

| Pattern | Adoption | Notes |
|---|---|---|
| Plugin registry | Consistent (engines, MCP adapters, mode handlers, directive handlers) | This is the dominant gene. Confident to lift. |
| `.js` import extension for `.ts` files | 100% consistent | Required by NodeNext + ESM; not a style choice |
| `index.ts` barrel | High | Every folder with > 2 files has one |
| FSM-driven | Workflows fully FSM; agents/runner not FSM (just imperative) | Intentional split: workflow is stateful coordination, agent run is a single transition |
| Bun-specific built-ins | Used directly (`Bun.spawn`, `Bun.which`, `Bun.build`, `bun:sqlite`) | Hard couples to Bun; Node compat is via Bun's Node compat layer |
| `debug()` everywhere | Consistent during dev; OTel migration in flight to replace with `otel_debug` | Dual-track currently |
| Telemetry markers in stream parsing | Consistent across engines | Each `telemetryParser.ts` per engine; uniform `ParsedTelemetry` output |
| Authentication caching | Consistent (single global cache) | One-of, no per-engine duplication |
| Auto-discovery via static imports | Yes for engines (hard-coded list) — not dynamic | Adding an engine needs a code edit. P1 if monocle wants plugin-installability. |
| ESM-only | 100% | No CJS in `src/`; only `config/*.js` is CJS, isolated by its own `config/package.json` (omitting `type: module`) |

### Inconsistencies

1. **Mixed CJS in `config/`**. The `config/*.js` files use `require()`/`module.exports` because they're loaded dynamically and the author wanted Node-compat without a build step. The override is explicit (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/config/package.json:3` likely defines `"type": "commonjs"` — confirmed by the lack of ESM syntax in `config/main.agents.js:1-55`).
2. **Two logging frameworks coexist** during the OTel migration: `debug()` (legacy) and `otel_debug/info/warn/error()`. Some files use both. See `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:10-23` — legacy block commented but `otel_*` is now primary.
3. **Hard-coded engine fallback id `'claude-code'`** in `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/run.ts:189` while the registry id is `'claude'`. Likely a bug; flagged in Pass 3 as a behavioral risk.
4. **Both `module` (workflow-step concept) and `Module` (registry of pre-built agents from `config/modules.js`) overload the same name**. The validator at `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/validator.ts:98-132` validates `step.module.behavior` while `config/modules.js` is a separate thing. Naming overload is intentional but confusing.

## State Checkpoint

```yaml
pass: 2
status: complete
files_scanned_for_pass: ~30
loc_read_in_depth: ~1500
timestamp: 2026-05-11T00:02:00Z
next_pass: 3
notes: |
  Conventions are unusually consistent for a ~50K-LOC codebase.
  Two trade-offs to flag for monocle:
  (1) Bun-specific (Bun.spawn, bun:sqlite) — hard to port to pure Node.
  (2) Engines registered statically — runtime plugin install would need a
      different mechanism. Both are pragmatic decisions, not failures.
```
