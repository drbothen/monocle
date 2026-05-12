# Pass 6: Holdout Seeds — codemachine-cli

This pass identifies subsystems, files, and questions that warrant deeper convergence rounds in Phase B. Each seed has a hypothesis, the broad-pass evidence backing it, and the specific gap to close in deepening.

## Convergence Order

Per the Iron Law convergence protocol, Pass 2 (domain model) and Pass 3 (behavioral contracts) deepen first; in this skill variant the relevant subsystem-level deep dives are:

### Tier 1 (deepen first — high user-stated interest)
1. **Workflows engine** — user explicitly named this as a gene to lift; 13K LOC, 114 files; only ~30% deep-read so far.
2. **Engine plugin architecture (harness)** — user's primary multi-harness interest; 12K LOC, 91 files; only Claude+Codex read closely.

### Tier 2 (deepen second)
3. **MCP subsystem** — workflow-signals + agent-coordination + router + per-engine adapters; ~25 files, only top-level read.
4. **Agents subsystem** — coordinator parsing not yet deeply traced.
5. **CLI commands & TUI shell** — TUI is 78 .tsx files (~9K LOC); only launcher signature touched.

### Tier 3 (deepen third)
6. **Shared/imports system** — workflow package manager. Plugin-distribution mechanism for monocle.
7. **Shared/telemetry/OTel wiring** — to confirm opt-in pathway is bypass-clean.
8. **Runtime/workspace bootstrap** — `.codemachine/` layout details.

### Tier 4 (cross-cutting audits in Pass B.5)
9. **Coordination MCP server vs workflow-signals MCP server** — both exist; their roles aren't fully disambiguated.
10. **Directive subsystem completeness** — 5 directive types (loop, pause, trigger, checkpoint, error) — only `pause` and `onAdvance` read; the other handlers are unknown surface area.
11. **Recovery system** — only `detect.ts` read; `restore.ts` is the actual code that performs recovery.

## Tier 1, Seed 1 — Workflows engine deep dive

**Subdirectories not yet fully read:**

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/modes/continuous.ts` — third mode handler; scenarios?
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/actions/{loop, advance, resume, directives}.ts` — mutation primitives
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/scenarios/` — full scenario table (1-8 mentioned, only 5 documented)
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/{execute,engine,hooks,skip}.ts` — step lifecycle
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/input/{providers/, emitter, types}` — input providers full contract
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/controller/{init, helper, view, config}.ts` — controller agent
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/onboarding/` — first-run flow
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/events/{bus, emitter}.ts` — event-bus shape
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/context/` — step context helpers
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/globals.ts` — `resolveStep`/`resolveFolder`/`resolveModule` definitions
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/utils/resolvers/` — resolver helpers
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/mcp.ts` — workflow-level MCP integration
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/indexing/{persistence, lifecycle, debug, types}.ts` — `template.json` schema deep-detail
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/recovery/restore.ts` — actual recovery routine
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/session/` — `StepSession` class

**Open questions:**

1. What is `continuous` mode? It's referenced in `getModeHandler(modeType)` paths but I only read interactive and autonomous handlers.
2. The full 8-scenario table — what is the precise mapping `(interactive ∈ {true|false|undef}, autoMode ∈ {true|false}, hasChainedPrompts ∈ {true|false}) → (scenarioId, modeType, inputSource)`?
3. Loop directive: what is the precise rewind semantics? `module.behavior.steps: number` (rewind N) — does it skip-while-rewinding? `loopMaxIterations` enforced where?
4. Trigger directive: agent-A calls trigger to run agent-B as a sub-step — is B's output piped back to A?
5. Checkpoint directive: what blocks/unblocks the workflow?
6. Controller view UI: how does the user "return to controller" via `c` key? Where does the controller's conversation history live?
7. The `template.json` full schema — what keys live there beyond `activeTemplate`, `completedSteps`, `controllerConfig`, `autonomousMode`, `selectedTrack`, `selectedConditions`?
8. Onboarding flow — is the project name persistence bug (TODO at `preflight.ts:76-77`) still active?

## Tier 1, Seed 2 — Engine plugin architecture (harness)

**Subdirectories not yet fully read:**

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/auggie/*` — Auggie CLI
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/opencode/*` — OpenCode (the DEFAULT engine, never read)
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/cursor/*` — Cursor (read metadata only)
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/mistral/*` — Mistral
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/ccr/*` — Claude Code Router
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/{claude, codex}/auth.ts` — full file
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/*/telemetryParser.ts` — every engine's stream-parse logic
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/core/auth.ts` — shared auth helpers
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/*/mcp/settings.ts` — settings-file location per engine

**Open questions:**

1. How does OpenCode resume/session work? It's the default engine — its semantics dominate. (BC-DRAFT-006 / -007 are Claude/Codex specific.)
2. CCR is "Claude Code Router" — does it speak a Claude-compatible stream format, or does it own its own protocol?
3. What's the `syncConfig` hook used for? Mentioned in registry but only stub-traced (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:333-342`).
4. What does each engine put in `settings.json` to enable MCP routing? The Claude adapter writes `data.mcpServers[settings.ROUTER_ID] = settings.getMCPRouterConfig()`. What is `getMCPRouterConfig()`? What's `ROUTER_ID`? — Read `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/mcp/settings.ts`.
5. Auth flow per engine: where is the OAuth/API-key dance handled? Does CodeMachine spawn the underlying CLI's `login` subcommand interactively?
6. What's the per-engine "experimental" flag semantic? UI label only?
7. Is there a hook for adding custom stop-tokens / system-message manipulation, or is the engine fully responsible for prompt assembly?

## Tier 2, Seed 3 — MCP subsystem

**Not yet read:**

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/registry.ts` — MCP adapter registry
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/router/{index, config, backend}.ts` — the MCP router
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/setup.ts` — orchestration
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/writer.ts` — full file (`ensureMCPConfig`)
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/context.ts` — context file management
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/errors.ts` — error taxonomy
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/agent-coordination/{executor, handler, validator, config, schemas, tools}.ts` — entire server
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/queue.ts` — signal queue file format

**Open questions:**

1. What does `agent-coordination` MCP server provide that `workflow-signals` does not? Is one for human-in-the-loop and one for agent-to-agent?
2. What is the MCP router architecture? Comment in `claude/mcp/adapter.ts` says "Manages workflow-signals MCP server configuration in Claude's settings". Is the router a *proxy* in front of multiple downstream MCP servers, or is it just config glue?
3. How is MCP filtering (`MCPConfigEntry.only`/`exclude`/`targets`) actually enforced — is it in the router middleware?
4. What's the signal queue file format? File names? Concurrency story?

## Tier 2, Seed 4 — Agents coordinator

**Not yet read:**

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/coordinator/{parser, service, execution, types}.ts` — full parser
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/execution/{actions, run, telemetry, types}.ts` — lower-level execution

**Open questions:**

1. Coordinator script grammar — full spec? Is there a published BNF? Operator precedence: `a && b & c` could mean `(a && b) & c` or `a && (b & c)`.
2. `[input:file1.md;file2.md,tail:100]` syntax — full grammar? Other annotations?
3. Are coordinator-run agents subject to the same FSM workflow lifecycle, or do they bypass it entirely?

## Tier 2, Seed 5 — TUI and CLI commands

**Not yet read in depth:** All 78 `.tsx` files plus `src/cli/commands/{auth, export, import, mcp, step, templates}.command.ts`.

**Open questions:**

1. Does the TUI subscribe to OTel spans for live status, or to the `WorkflowEventBus`?
2. The "modal stack" in the workflow route — how is z-ordering managed?
3. What does `codemachine step <step-name>` do? Run a single step out of the current template?
4. Auth flow UX — does it use `@clack/prompts` or Solid components?
5. The home route (`src/cli/tui/routes/home/`) — what does it show? Workflow template browser?

## Tier 3, Seed 6 — Imports system (workflow package manager)

**Not yet read:**

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/imports/{auto-import, resolver, installer, manifest, registry, paths, resolve, defaults}.ts`

**Open questions:**

1. What IS the workspace import system? Is it npm-based? Custom?
2. How does `getAllInstalledImports()` work — manifest file, npm-list, or symlink scan?
3. What `defaults` exist? Are "default packages" bundled CodeMachine workflows like Ali?
4. How does this relate to monocle's "workflow as package" needs?

## Tier 3, Seed 7 — Telemetry subsystem

**Not yet read:**

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/logging/{otel-init, otel-logger, agent-loggers, spinner-logger}.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/tracing/{init, tracers, sampler, storage, config, exporters/}` (full)
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/metrics/{init, meters, instruments/, exporters/}` (full)
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/telemetry/{capture, types, logger}.ts`

**Open questions:**

1. When OTel is opt-out (`CODEMACHINE_TRACE` unset), what does `otel_debug` actually do? Console fallback? No-op?
2. Sampler strategy — head-based or tail-based? Trace-context propagation across MCP IPC?
3. `LOGGER_NAMES` enum — full list?
4. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/TELEMETRY_MIGRATION_PHASE_PLAN.md` is 714 lines — what's the migration status by phase?

## Tier 3, Seed 8 — Runtime/workspace

**Not yet read:**

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/services/workspace/{init, discovery, fs-utils}.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/services/validation.ts`
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/shared/workflows/template.ts` — the workspace persistence layer (`getSelectedTrack`, `setActiveTemplate`, etc.)

**Open questions:**

1. Full `.codemachine/` directory structure created on first run?
2. Does it write a `.gitignore` or leave that to the user?
3. What does `validateSpecification(path)` actually check — file exists? Non-empty? Specific format?
4. What's the workspace discovery walk — is `.codemachine` found by walking up?

## Files Worth a Spot-Check During Convergence

Files >300 LOC that are likely important and not yet read in detail (LOC count from earlier `find … wc -l` runs, partial list):

| File | Approx LOC | Why |
|---|---|---|
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts` | 516 | Read in full but heavy commented-out content — deserves a second pass once telemetry migration completes |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/run.ts` | 363 | Read in full but cross-references many sub-modules |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/runner.ts` | 547 | Read in full but error-fallback paths are dense |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts` | 383 | Read; deepen the JSON-stream parse cases (system/init, error, result, assistant.error) |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts` | 402 | Read in full but signal-race ordering deserves a property test mental model |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/state/machine.ts` | 377 | Read in full; declarative — confirm no untested transitions |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/validator.ts` | 265 | Read in full; reasonable to lift verbatim |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/index.ts` | 340 | Read in full |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/core.ts` | 293 | Read in full |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/step/run.ts` | 382 | Read in full |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/runner/chained.ts` | 314 | Read in full |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/index.ts` | 265 | Read in full |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/loader.ts` | 106 | Read in full |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/signals/manager/manager.ts` | 196 | Read in full |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/runner/modes/{interactive,autonomous}.ts` | 215 + 211 | Read in full |
| `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/scripts/build.ts` | 231 | Read in full |

## State Checkpoint

```yaml
pass: 6
status: complete
seeds_identified: 8
tier_1: 2 (workflows engine, engine plugin architecture)
tier_2: 3 (MCP, agents coordinator, TUI+CLI)
tier_3: 3 (imports, telemetry, runtime/workspace)
files_remaining_to_read: ~280 unread .ts/.tsx files of 498 total
loc_read_so_far: ~6500 of 47557
percent_read: ~14%
timestamp: 2026-05-11T00:06:00Z
next_phase: Phase B deepening (Tier 1 first)
notes: |
  Tier 1 deepening must address the two user-stated deliverables:
  (a) Harness architecture summary (engine plugins)
  (b) Workflows system summary (templates + FSM + runner)
  Each will require 2-3 rounds before honestly hitting NITPICK.
```
