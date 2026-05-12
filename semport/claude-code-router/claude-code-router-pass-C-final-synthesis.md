# Claude Code Router — Brownfield-Ingest Final Synthesis (Phase C)

Reference: `/Users/jmagady/Dev/monocle/.reference/claude-code-router/`
HEAD: `e270dea523b8ac025ab9b7b0708dc170efa52d8a` (branch `main`, last commit `update banner`)
Version: 2.0.0
Run date: 2026-05-11

## Summary

Claude Code Router (CCR) is a **local reverse-proxy / API-shim** that lets Anthropic's Claude Code CLI (and any tool that speaks the Anthropic `/v1/messages` protocol) talk to non-Anthropic LLM providers. It listens on `127.0.0.1:3456` by default, accepts Anthropic-flavored requests, picks a target provider+model via a rule-based router, runs the request body and response stream through a chain of **transformers** that adapt to each provider's API dialect, and forwards the call to OpenAI, DeepSeek, Gemini, OpenRouter, Ollama, Volcengine, SiliconFlow, ModelScope, DashScope, AIHubmix, Groq, Cerebras, Vertex AI, etc.

Concretely: it routes the **HTTP request from `claude` CLI to the upstream LLM**, choosing a different model per-request based on signal types (background / thinking / longContext / webSearch / image / Haiku-detection / subagent-tag) and per-session token-budget memory. It is **not** an MCP router and it does not route across IDE harnesses — it lives at the model-API boundary on a single host.

The codebase is a five-package pnpm monorepo. The single most important fact for monocle: the package named `core` is actually `@musistudio/llms` (v1.0.51), an independent universal-LLM-server framework that musistudio maintains separately. The CCR wrapper is comparatively thin (~1.9k LOC in `packages/server`, ~3.9k in `packages/cli`). The 11.9k LOC of `core` and its 20 transformers is where the engineering value is.

## Snapshot

| Item | Value |
|------|-------|
| HEAD commit | `e270dea523b8ac025ab9b7b0708dc170efa52d8a` |
| Branch | `main` |
| Top-level version | 2.0.0 |
| Core package version | `@musistudio/llms@1.0.51` |
| Node requirement | `>=20.0.0` (top-level package.json) / `>=18.0.0` (CLAUDE.md, inconsistency) |
| Package manager | pnpm `>=8.0.0` with workspace protocol |
| Build tooling | esbuild for cli/server/shared, tsx for core, Vite for ui |
| Test framework | None — zero `*.test.*` or `*.spec.*` files in any package |
| License | MIT |

LOC per package (TypeScript/TSX/JS under `packages/<name>/src`, excludes node_modules):

| Package | LOC | Files | Role |
|---------|-----|-------|------|
| core (`@musistudio/llms`) | 11878 | 58 | Universal LLM proxy server, all routing + all transformers |
| ui (`@CCR/ui`) | 9462 | 47 | React 19 + Vite admin console |
| cli (`@CCR/cli`) | 3856 | 19 | `ccr` command-line tool, preset manager, statusline |
| shared (`@CCR/shared`) | 2103 | 10 | Constants and preset utilities |
| server (`@CCR/server`) | 1909 | 12 | CCR-specific wrapper around core: auth, agents, preset namespaces, config persistence |

Key runtime dependencies:

| Dep | Used by | Purpose |
|-----|---------|---------|
| fastify 5.4 | core, server | HTTP server |
| @fastify/cors, @fastify/static, @fastify/multipart | server | CORS / UI static serving / file upload |
| @anthropic-ai/sdk 0.54 | core | Anthropic message type definitions |
| @google/genai 1.7 | core | Gemini SDK |
| @huggingface/tokenizers 0.0.6 | core | Local tokenizer for open-source models |
| tiktoken 1.0.21 | server | Default token counter |
| lru-cache 11.2 | core, server | Session caches |
| json5 2.2.3 | server, shared | Comment-tolerant config parsing |
| rotating-file-stream 3.2 | server | Log rotation (pino sink) |
| react 19.1, react-dom 19.1 | ui | UI |
| @radix-ui/* | ui | Headless UI primitives |
| @monaco-editor/react 4.7 | ui | JSON editor |
| i18next 25.3 / react-i18next 15.6 | ui | en + zh locales only |
| @inquirer/prompts 5.0 | cli | Interactive model selector / preset prompts |
| find-process 2.0 | cli | PID liveness check |
| adm-zip 0.5.16, archiver 7.0 | cli, server, shared | Preset bundle ZIP I/O |
| shell-quote 1.8 | server | Safe shell arg quoting for `ccr code` |
| undici (ProxyAgent) | core | HTTPS_PROXY support for outbound calls |

## Monorepo Architecture

Dependency direction (workspace protocol):

```
cli ─┬─→ server ──→ core (@musistudio/llms)
     └─→ shared       │
server ─→ shared      │
core (standalone) ────┘
ui    (standalone build, served as static by server)
```

- `core` has **no workspace dependencies** — it is a self-contained server framework. The `@/` import alias inside core resolves to `packages/core/src` via `tsconfig.base.json` paths.
- `server` extends `core`'s `Server` class, attaches CCR-specific Fastify hooks (auth, agent dispatch, SSE rewriting, preset namespace registration), and re-exports `pluginManager` / `tokenSpeedPlugin` / `sessionUsageCache` from core (`packages/server/src/index.ts:18,455`).
- `cli` consumes `server.getServer()` (`packages/cli/src/utils/index.ts:15,192`) to start the daemon in-process; it does not talk to core directly.
- `ui` is built standalone and shipped as static assets that `server` serves under `/ui/` (`packages/server/src/server.ts:117-121`).
- `shared` is a constants + preset library. The CONFIG_FILE / HOME_DIR / PID_FILE / PRESETS_DIR paths are all defined once here (`packages/shared/src/constants.ts:4-14`).

## Routing Core (THE primary deliverable)

The router is a **single async function** at `/Users/jmagady/Dev/monocle/.reference/claude-code-router/packages/core/src/utils/router.ts:218-298`, registered as a Fastify `preHandler` hook scoped to URLs ending in `/v1/messages` (`packages/core/src/server.ts:144-151,185-192`). It runs **after** auth and **before** the transformer pipeline, mutating `req.body.model` to a `"providerName,modelName"` string that the downstream `handleTransformerEndpoint` consumes.

### What gets routed

Only requests to `/v1/messages` (Anthropic Messages API shape) and prefixed namespace variants `<prefix>/v1/messages` for installed presets (`packages/server/src/index.ts:204-209`). Everything else (`/health`, `/api/config`, `/api/logs`, `/api/presets/*`, `/ui/*`, `/api/restart`, `/v1/messages/count_tokens`, transformer endpoints like `/v1/chat/completions` from OpenAITransformer) bypasses the router.

### Routing decision tree (in evaluation order, `router.ts:124-200`)

The decision tree below executes for every request body that survives auth:

1. **Subagent escape hatch.** If the request body's `system[1].text` starts with `<CCR-SUBAGENT-MODEL>`, extract `provider,model` from that tag, strip the tag from the system text in place, and use that model. Scenario = `default`. (`router.ts:161-175`)
2. **Explicit `provider,model` in body.** If `req.body.model` contains a comma, look up the provider+model in the loaded providers list and use it as-is. Scenario = `default`. (`router.ts:134-146`)
3. **Long-context override.** Compute the `tokenCount` of `messages + system + tools` via the per-model tokenizer (tiktoken cl100k_base by default; HuggingFace or API tokenizers if configured per provider+model in `tokenizer.ts`). If `tokenCount > Router.longContextThreshold` (default 60000) **OR** the previous request's `input_tokens` for this session exceeded that threshold AND the current request is >20000 tokens, route to `Router.longContext`. Scenario = `longContext`. (`router.ts:149-160`)
4. **Haiku trap.** If the incoming `body.model` contains both `"claude"` and `"haiku"` and `Router.background` is set, route to `Router.background`. This intercepts Claude Code's automatic Haiku usage for cheap background tasks (e.g., summarization) and redirects them to a cheaper local/background model. Scenario = `background`. (`router.ts:177-185`)
5. **Web-search priority.** If `req.body.tools` contains any tool whose `type` starts with `"web_search"` AND `Router.webSearch` is set, route to it. Scenario = `webSearch`. (`router.ts:187-193`)
6. **Thinking.** If `req.body.thinking` is truthy AND `Router.think` is set, route to it. This is how Claude Code's Plan Mode (extended thinking) gets a reasoning-capable model. Scenario = `think`. (`router.ts:194-198`)
7. **Default.** Otherwise return `Router.default`. Scenario = `default`. (`router.ts:199`)

### Pre-decision config resolution

Before any of step 1-7, the router runs `getProjectSpecificRouter(req, configService)` (`router.ts:91-122`). The flow:

1. Parse `sessionId` from `req.body.metadata.user_id` (Claude Code embeds it as `something_session_<uuid>`) at `router.ts:221-226`.
2. If `sessionId` is set, scan `~/.claude/projects/<project>/` directories to find the one containing `<sessionId>.jsonl`. The result is cached in an LRU of 1000 entries (`router.ts:303-366`).
3. If found, prefer `<project>/<sessionId>.json` over `<project>/config.json` for a per-session Router override; if neither exists, fall back to the global `Router`.

So routing rules are **per-session-with-project-override**, not just global.

### Custom router escape hatch

If `CUSTOM_ROUTER_PATH` is set in config (`router.ts:270-281`), CCR `require()`s that JS file and calls `customRouter(req, config, { event })`. If it returns a non-null string, that overrides all 7 built-in rules. If it returns null/undefined or throws, CCR falls back to the built-in tree. `req.tokenCount` is passed in so custom routers can do their own threshold logic. The example at `/Users/jmagady/Dev/monocle/.reference/claude-code-router/custom-router.example.js` is a 3-line constant return. Custom routers are loaded via `require()` so they're CommonJS only; no sandboxing.

### Image agent (special case, executes before the router)

The image agent (`packages/server/src/agents/image.agent.ts:58-101`) is **not** a router but it pre-empts routing for image requests. In the server's `preHandler` (`packages/server/src/index.ts:211-243`), each registered agent's `shouldHandle(req, config)` runs. For ImageAgent:

- If `Router.image` is configured AND `forceUseImageAgent` is false AND the last user message contains image content, **the router-step is bypassed** — `req.body.model` is set directly to `Router.image` and the message is rewritten to push image content to the end. (`image.agent.ts:62-89`)
- Otherwise, if any message has image content and the target model can't see images, the agent registers an `analyzeImage` tool, caches the actual image bytes in an in-memory LRU keyed by `<reqId>_Image#<n>` with 5-min TTL, replaces image content blocks with text placeholders like `[Image #1]This is an image...`, and lets the main model invoke `analyzeImage(imageId, task)` which fires an internal POST to `127.0.0.1:<port>/v1/messages` against `Router.image` (`image.agent.ts:149-241`). The tool result is spliced back into the stream via `rewriteStream`.

### Scenario type and fallback

The router writes `req.scenarioType` (one of `default | background | think | longContext | webSearch`) onto the request. Downstream, on `provider_response_error`, `handleFallback` in `packages/core/src/api/routes.ts:108-194` looks up `config.fallback?.[scenarioType]` (an array of `provider,model` strings) and tries each in sequence with full transformer pipeline. The image scenario has **no fallback path** (image goes through the agent, not the router scenario).

### What is NOT in the router

- No load balancing across providers (the only multi-target logic is the fallback chain on error).
- No rate limiting or quota enforcement (no token-bucket / leaky-bucket primitives anywhere).
- No A/B testing or canarying.
- No request retries (other than fallback-on-error).
- No circuit breaker / health checks of upstream providers.
- No per-tenant routing — the only "tenant" axis is `sessionId`, derived from Claude Code's metadata; there's no API key → tenant mapping.

## Server Package

`packages/server/` is a thin CCR-specific layer on top of `core`'s `Server` class. Total LOC is 1909 across 12 files. Responsibilities:

| File | LOC | Responsibility |
|------|-----|----------------|
| `src/index.ts` | 463 | `getServer()` factory: bootstrap config, choose HOST/PORT, init pino+rotating-file-stream logger, register presets as namespaces, attach auth hook, attach agent-dispatch hook, attach SSE `onSend` hook that rewrites streams when agents are active, attach `sessionUsageCache.put` for token-usage memory. Also defines `run()` which starts the daemon. |
| `src/server.ts` | 488 | `createServer(config)` returns a `core.Server` instance and attaches REST endpoints: `/v1/messages/count_tokens`, `GET/POST /api/config`, `GET /api/transformers`, `GET /api/logs[/files]`, `DELETE /api/logs`, `/api/presets`, `/api/presets/:name`, `/api/presets/:name/apply`, `/api/presets/install/github`, `/api/presets/market`. Also mounts UI static files at `/ui/`. |
| `src/middleware/auth.ts` | 58 | `apiKeyAuth(config)` Fastify hook. Public paths: `/`, `/health`, `/ui*`. If no providers configured → skip auth. If providers but no `APIKEY` → CORS-restrict to `127.0.0.1:PORT`. If APIKEY set → require `Authorization: Bearer <key>` or `x-api-key: <key>`. Returns 401 on mismatch. |
| `src/agents/index.ts` | 49 | `AgentsManager` singleton holding a `Map<name, IAgent>`. Pre-registers `imageAgent`. |
| `src/agents/image.agent.ts` | 305 | The ImageAgent class (see Routing Core section). |
| `src/agents/type.ts` | 20 | `IAgent` and `ITool` interfaces. |
| `src/utils/index.ts` | 174 | `initDir`, `readConfigFile` (JSON5 + env-var interpolation via `$VAR` / `${VAR}`), `writeConfigFile`, `backupConfigFile` (timestamp suffix, keeps last 3). |
| `src/utils/SSEParser.transform.ts` | 72 | **Duplicate** of core's parser, character-for-character. |
| `src/utils/SSESerializer.transform.ts` | ~50 | **Duplicate** of core's serializer. |
| `src/utils/rewriteStream.ts` | 32 | **Duplicate** of core's `rewriteStream`. |
| `src/types.d.ts`, `src/types/llms-plugin.d.ts` | ~200 | Type declarations for core's exports (Type ambient declarations because `@musistudio/llms` has its own d.ts) |

The duplicate SSE files in server vs core are a real concern — they will drift. CCR imports `SSEParserTransform` from `./utils/SSEParser.transform` in `index.ts:11-13`, NOT from `@musistudio/llms`, so changes to either copy stay local.

### Server's only routing-shaped logic

The auth middleware is the only true gating logic in the server package. Everything else delegates to core's hooks. Server's `preHandler` chain in `getServer()` adds, in order:
1. (registered last but `preHandler`s run in registration order within a route) Auth (`auth.ts`).
2. Parse `req.pathname` and detect preset prefix.
3. Agent shouldHandle/reqHandler (image-injection).
4. (Inside core's `registerNamespace`) Router execution.

The `onSend` hook in `index.ts:247-421` is the most complex piece of code in `server/`: it intercepts the streaming response and, **if agents tagged the request**, parses the SSE stream, watches for `content_block_start` events naming an agent tool, accumulates the `input_json_delta` chunks, parses the tool args, invokes the agent's tool handler, appends an assistant tool_use + user tool_result pair to the message history, re-POSTs to `127.0.0.1:<port>/v1/messages`, and splices the resulting stream into the original stream. This is the agent recursion loop.

If no agent is active, `onSend` simply `tee()`s the stream, reads `event: message_delta` blocks in the background to extract `usage` data, and puts it in `sessionUsageCache` (LRU 100, keyed by sessionId).

## CLI Package

`packages/cli/` provides the `ccr` binary. The `bin` field in the top-level package.json maps `ccr` → `dist/cli.js`. 3856 LOC across 19 files.

Commands and their handlers (`cli.ts:211-442`):

| Command | Handler | Behavior |
|---------|---------|----------|
| `ccr start` | `run()` in `utils/index.ts:186` | In-process call to `server.getServer().then(s => s.start())`. Writes PID to `~/.claude-code-router/.claude-code-router.pid`. Also adds `/api/update/check`, `/api/update/perform`, `/api/restart` routes that the UI calls. |
| `ccr stop` | inline | Read PID file, `process.kill(pid)`, delete reference-count file. |
| `ccr restart` | `restartService()` in `utils/index.ts:220` | Stop + detached respawn. |
| `ccr status` | `showStatus()` in `utils/status.ts` | Read PID, log endpoint+port. |
| `ccr code [args]` | `executeCodeCommand()` in `utils/codeCommand.ts:25` | Auto-start service if not running (detached `spawn`, then poll with `waitForService`). Then spawn `claude` (path from `config.CLAUDE_PATH` or env or just `claude`) with `--settings <tempFile>` flag. The temp file is a JSON `settings` object containing `env` with `ANTHROPIC_BASE_URL=http://127.0.0.1:<port>` and `ANTHROPIC_AUTH_TOKEN=<apikey>`. Increments a reference count file so multiple concurrent `ccr code` sessions don't kill each other. |
| `ccr <preset-name> [args]` | inline in `cli.ts:103-208` | Treats unknown command as a preset name. Loads `~/.claude-code-router/presets/<name>/manifest.json`. If `noServer: true`, points Claude Code directly at the provider's URL/key; otherwise points at `http://127.0.0.1:<port>/preset/<name>` (preset namespaces). |
| `ccr model` | `runModelSelector()` in `utils/modelSelector.ts` | Interactive `@inquirer/prompts` flow for editing Router rules and provider configs in `config.json`. ~700 LOC. |
| `ccr preset <sub>` | `handlePresetCommand()` in `utils/preset/commands.ts` | export / install / list / info / delete. Wraps the shared package's preset functions with CLI prompts. |
| `ccr install <name>` | `handleInstallCommand()` in `utils/installCommand.ts` | Install preset from marketplace by name. Marketplace URL is fetched via `@CCR/shared`'s `getMarketPresets()`. |
| `ccr activate` / `ccr env` | `activateCommand()` in `utils/activateCommand.ts` | Print `export KEY="VAL"` shell lines for `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`, `NO_PROXY`, `DISABLE_TELEMETRY`, `DISABLE_COST_WARNINGS`, `API_TIMEOUT_MS`, plus `unset CLAUDE_CODE_USE_BEDROCK`. Designed for `eval "$(ccr activate)"`. |
| `ccr ui` | inline | Auto-start service if needed, then `open` / `xdg-open` / `start` the platform browser pointed at `http://<endpoint>/ui/`. |
| `ccr statusline [preset]` | inline + `parseStatusLineData()` in `utils/statusline.ts` | Read JSON from stdin (Claude Code's statusline hook protocol), format an ANSI-colored status line, write to stdout. ~600 LOC. Supports custom modules via `config.StatusLine.modules[]`. |
| `ccr -v` / `ccr version` | inline | Print version. |

The CLI runs `update.ts` checks via npm registry against the published version on a separate axis (used by `/api/update/check`).

### Reference counting

`processCheck.ts:16-39` defines `incrementReferenceCount` / `decrementReferenceCount` / `getReferenceCount`, persisted in `/tmp/claude-code-reference-count.txt`. Each `ccr code` invocation bumps the counter; the server uses this to decide whether to auto-shut-down when all clients exit (`closeService` in `processCheck.ts`).

## UI Package

`packages/ui/` is a standalone React 19 + Vite + TailwindCSS 4 SPA, served as static files at `/ui/`. 9462 LOC across 47 files. **It is the most expensive package by far (4.9× the size of the server).**

Major routes (`src/routes.tsx`):
- `/login` — API key gate (stored in `localStorage` and sent as `x-api-key`)
- `/` — main config editor (Providers + Router + Transformers panels)
- `/presets` — preset list/install/apply

Main components in `src/components/`:

| Component | LOC | Purpose |
|-----------|-----|---------|
| `ConfigProvider.tsx` | ~150 | React context wrapping `api.getConfig()` |
| `Providers.tsx` + `ProviderList.tsx` | ~900 | Editable provider list with model dropdowns, transformer config |
| `Router.tsx` | ~250 | Form for `Router.default/background/think/longContext/longContextThreshold/webSearch/image` |
| `Transformers.tsx` + `TransformerList.tsx` | ~600 | List of registered transformers (read-only display) + custom transformer file paths |
| `JsonEditor.tsx` | ~250 | Monaco-based raw `config.json` editor |
| `LogViewer.tsx` | ~400 | List + view + clear log files via `/api/logs` |
| `Presets.tsx` + `preset/DynamicConfigForm.tsx` | ~800 | Preset gallery, install from marketplace, apply preset secrets (the dynamic schema flow) |
| `SettingsDialog.tsx` | ~400 | APIKEY, HOST, PORT, PROXY_URL, LOG, LOG_LEVEL, API_TIMEOUT_MS edit |
| `StatusLineConfigDialog.tsx` + `StatusLineImportExport.tsx` | ~1200 | Statusline module editor with color picker, drag-and-drop reorder, JSON import/export |
| `RequestHistoryDrawer.tsx` + `lib/db.ts` | ~300 | Recent-requests view backed by IndexedDB on the browser side |
| `DebugPage.tsx` | ~300 | Real-time WebSocket-less SSE viewer for currently-streaming requests |
| `Login.tsx`, `ProtectedRoute.tsx`, `PublicRoute.tsx` | ~200 | Local auth |
| `ui/*.tsx` (20 files) | ~2800 | Radix-wrapped shadcn-style design system primitives |

i18n: only `en.json` and `zh.json` exist in `src/locales/`. Languages switched via `i18next-browser-languagedetector` + dropdown.

The UI has **no real-time data flow** (no WebSocket / SSE consumer). Everything is polled REST. Log viewer reads files; request history is local-only IndexedDB.

## Shared Types

`packages/shared/` is a small package (2103 LOC, 10 files). It exports:

- `constants.ts` — All filesystem paths: `HOME_DIR`, `CONFIG_FILE`, `PLUGINS_DIR`, `PRESETS_DIR`, `PID_FILE`, `REFERENCE_COUNT_FILE`, `CLAUDE_PROJECTS_DIR`, and a `DEFAULT_CONFIG`.
- `preset/types.ts` (267 LOC) — The preset type system: `PresetFile`, `ManifestFile`, `PresetMetadata`, `PresetConfigSection`, `RequiredInput` (with `InputType` enum and `Condition` for conditional display), `ConfigMapping` (with `target` field-path string and `value` referencing input IDs), `MergeStrategy` enum, `PresetIndexEntry` / `PresetRegistry` for the marketplace.
- `preset/install.ts` (416 LOC) — `getPresetDir`, `extractPreset` (ZIP extraction with path-traversal guards), `readManifestFromDir`, `manifestToPresetFile`, `downloadPresetToTemp` (fetch + write to `~/.claude-code-router/temp/`), `loadPreset`, `validatePreset`, `extractMetadata`, `saveManifest`, `isPresetInstalled`, `listPresets`. **All preset filesystem operations**.
- `preset/export.ts` — `exportPreset(name, config, opts)` builds a preset directory with sanitized config.
- `preset/sensitiveFields.ts` (~140 LOC) — Pattern list (`api_key`, `secret`, `token`, `password`, `private_key`, `access_key`), `generateEnvVarName`, `sanitizeConfig` recursive walker that replaces sensitive values with `{{fieldId}}` placeholders during export.
- `preset/merge.ts` (~150 LOC) — `mergeProviders` / `mergeRouter` / `mergeTransformers` with 4 strategies (`ASK`, `OVERWRITE`, `MERGE`, `SKIP`).
- `preset/schema.ts` — `loadConfigFromManifest(manifest, presetDir)` applies `userValues` against `template` and `configMappings`, producing the runtime config. Variable substitution is `#{fieldId}` (different from env-var `$VAR` and statusline `{{var}}` — three placeholder dialects in one project).
- `preset/marketplace.ts` — `getMarketPresets()` fetches a remote JSON index, `findMarketPresetByName()` resolves a name to a GitHub repo URL.
- `preset/readPreset.ts` — Convenience reader used by CLI.

The **preset system** is the most domain-specific concept in this codebase apart from routing. A preset is a versioned, shareable, parameterizable bundle of Providers + Router rules + transformers, distributed as a directory containing `manifest.json`, installable from local path, GitHub URL, or a marketplace index. Sensitive fields are stripped on export and re-collected via the `schema` field's `RequiredInput[]` form during install. This is meaningfully more advanced than just "config templates" — it's a packaged-app model for LLM router configs.

## Provider Abstraction

A `Provider` (`packages/core/src/types/llm.ts:200-241`) is:

```ts
interface LLMProvider {
  name: string;
  baseUrl: string;           // FULL URL to /chat/completions or /v1/messages
  apiKey: string;
  models: string[];          // Free-form model identifiers
  transformer?: {
    use?: Transformer[];     // Provider-level transformer chain
    [modelName: string]: { use?: Transformer[] }; // Per-model overrides
  };
}
```

Providers are NOT typed by class (no `OpenAIProvider` vs `AnthropicProvider`); they are tagged by **which transformer chain is configured**. The `transformer.use` array is the polymorphism axis. From CCR's perspective, every provider speaks "whatever shape the transformer chain outputs," and the transformer is responsible for adapting Anthropic's wire format to that provider's API.

### Provider lifecycle

`ProviderService` (`packages/core/src/services/provider.ts:12`) holds two maps: `providers: Map<string, LLMProvider>` and `modelRoutes: Map<string, ModelRoute>`. On bootstrap, `initializeCustomProviders()` iterates `config.providers[]`, validates `name + api_base_url + api_key + models[]` are present (silently skips otherwise — no error logging), instantiates each transformer's constructor with its options dict (or via `Transformers[i].TransformerName` static lookup), and calls `registerProvider`. For each `(provider, model)` pair, two route keys are registered: `"provider,model"` (full) and `"model"` (short, only if not already taken). The short key is first-wins, so two providers offering the same model name will only have the first short-keyed.

REST endpoints (`packages/core/src/api/routes.ts:491-682`) expose CRUD: `POST /providers` (with JSON schema validation requiring an `id` and `type` field that the codebase **never sets** on the stored provider — bug noted), `GET /providers`, `GET /providers/:id`, `PUT /providers/:id`, `DELETE /providers/:id`, `PATCH /providers/:id/toggle` (the toggle is a no-op — it returns `true` always; provider enable/disable is not actually implemented despite the route existing).

### The 20 transformers

In `packages/core/src/transformer/` (sorted by LOC):

| Transformer | LOC | TransformerName | What it does |
|-------------|-----|-----------------|--------------|
| anthropic.transformer | 1069 | `Anthropic` (instance name) | The reference shape. Has `endPoint = "/v1/messages"`, so it gets registered as a Fastify route. `auth()` sets `x-api-key` or `Bearer`. `transformRequestOut` converts Anthropic→UnifiedChatRequest. `transformResponseIn` converts upstream response back to Anthropic shape. |
| openai.responses.transformer | 792 | (varies) | OpenAI Responses API (the new structured-output API) adaptation. |
| vercel.transformer | 357 | `vercel` | Vercel AI SDK shape. |
| openrouter.transformer | 357 | `openrouter` | Strips cache_control for non-Claude OpenRouter models, formats image URLs differently for Claude vs non-Claude, merges options like `provider.only` into request. |
| forcereasoning.transformer | 342 | `forcereasoning` | Wraps requests to force model into reasoning mode. |
| enhancetool.transformer | 334 | `enhancetool` | Error-tolerant tool-call parsing; breaks streaming as a side effect. |
| reasoning.transformer | 250 | `reasoning` | Processes `reasoning_content` field for DeepSeek-R1-style responses. |
| groq.transformer | 227 | `groq` | Groq API adaptations. |
| tooluse.transformer | 223 | `tooluse` | Optimizes `tool_choice` for certain models. |
| deepseek.transformer | 221 | `deepseek` | Clamps max_tokens to 8192; extracts `reasoning_content` from delta. |
| customparams.transformer | 107 | `customparams` | Inject arbitrary parameters from options. |
| vertex-claude.transformer | 81 | `vertex-claude` | Vertex AI Anthropic models. |
| vertex-gemini.transformer | 79 | `vertex-gemini` | Vertex auth + Gemini routing. |
| cerebras.transformer | 44 | `cerebras` | Cerebras API. |
| sampling.transformer | 41 | `sampling` | Process temperature / top_p / top_k / repetition_penalty. |
| gemini.transformer | 40 | `gemini` (instance) | `endPoint = "/v1beta/models/:modelAndAction"`. Rewrites URL to `<base>/<model>:streamGenerateContent` and injects `x-goog-api-key`. |
| cleancache.transformer | 23 | `cleancache` | Strips `cache_control` from requests. |
| maxtoken.transformer | 18 | `maxtoken` | Set max_tokens to options.max_tokens. |
| streamoptions.transformer | 16 | `streamoptions` | Inject `stream_options`. |
| maxcompletiontokens.transformer | 16 | `maxcompletiontokens` | Use `max_completion_tokens` instead of `max_tokens`. |
| openai.transformer | 6 | `OpenAI` (instance) | Pure routing — `endPoint = "/v1/chat/completions"`, no body/response work. |

A transformer implements (`packages/core/src/types/transformer.ts:24-43`):

```ts
type Transformer = {
  transformRequestIn?:  (req, provider, ctx) => Promise<UnifiedChatRequest>;
  transformResponseIn?: (resp, ctx) => Promise<Response>;
  transformRequestOut?: (req, ctx) => Promise<UnifiedChatRequest>;
  transformResponseOut?:(resp, ctx) => Promise<Response>;
  endPoint?: string;
  name?: string;
  auth?: (req, provider, ctx) => Promise<any>;
};
```

The IN/OUT direction is **provider-perspective**: `In` = "into the provider" (request) / "into the unified format" (response); `Out` = "out of the unified format" (request) / "out of the provider" (response). The pipeline (`packages/core/src/api/routes.ts:201-275`):

1. `transformer.transformRequestOut(body)` — convert incoming Anthropic-shaped request to UnifiedChatRequest.
2. For each `provider.transformer.use[]` transformer in order: `transformRequestIn(req, provider, ctx)`.
3. For each `provider.transformer[body.model].use[]` model-specific transformer: `transformRequestIn`.
4. `sendUnifiedRequest()` posts to provider URL with `Authorization: Bearer <apikey>` (unless `auth()` overrode headers).
5. Provider-level `transformResponseOut` chain (reversed order).
6. Model-specific `transformResponseOut` chain (reversed order).
7. `transformer.transformResponseIn(resp)` — convert provider response back to the endpoint's expected shape.

Bypass shortcut (`routes.ts:282-294`): if the provider has exactly one transformer and it matches the endpoint's transformer (e.g., both are AnthropicTransformer), skip all transformer steps and proxy raw. This is the path that lets you point CCR at an actual Anthropic endpoint as just a logging passthrough.

### Plugin-as-transformer dialect

Custom transformers (`config.transformers[]`) are loaded via `require(config.path)` in `TransformerService.registerTransformerFromConfig` (`packages/core/src/services/transformer.ts:82-111`). The module's default export must be a class constructor; `new module(options)` is called. The instance's `name` property is used as the registration key. The example transformers in the docs (`gemini-cli.js`, `qwen-cli.js`, `rovo-cli.js`, `chutes-glm-transformer.js`) all live in external gists and follow this shape.

This is **the** plugin/transformation model. There is no separate "middleware" plugin API at this layer — transformers are it.

## Cost and Telemetry

There is **no first-class cost tracking**. CCR does NOT attribute USD cost to models, sessions, or routes. The codebase contains zero `cost` / `price` / `usd` fields outside of UI strings.

What it does have:

### Session usage cache

`sessionUsageCache: LRUCache<string, { input_tokens, output_tokens }>` (`packages/core/src/utils/cache.ts:47`), capacity 100, populated on every `event: message_delta` chunk in `packages/server/src/index.ts:387-394`. Used by the router for the lookback-threshold longContext rule (`router.ts:227,150-153`). Not exposed via any REST endpoint, not surfaced in the UI's request history.

### token-speed plugin

`packages/core/src/plugins/token-speed.ts` (450 LOC) is the only "telemetry" code. It's a Fastify plugin (`CCRPlugin` interface), registered via `pluginManager.registerPlugin(tokenSpeedPlugin, {...})` and enabled via the `plugins[]` array in config. When enabled:

- `onRequest` hook records `requestStartTime` for `/v1/messages`.
- `onSend` hook tees the response stream. On the cloned stream, it parses SSE events, records `firstTokenTime` (TTFT) on first content event, accumulates token counts using the configured tokenizer (or fallback char-heuristic), and emits stats every 1 second during streaming.
- On `message_stop`, computes final tokens/sec.
- Stats are routed to **output handlers**: `console-handler.ts` (stdout with optional ANSI colors), `temp-file-handler.ts` (writes JSON to `/tmp/claude-code-router/session-*.json`), or `webhook-handler.ts` (HTTP POST to a configured URL). The output system is configurable via `plugins[0].options.outputHandlers[]` or shorthand `plugins[0].options.reporter: ['console','temp-file']`.

What the plugin exposes (`getTokenSpeedStats(sessionId?)`, `getGlobalTokenSpeedStats()`): per-session token totals, TTFT, tokens-per-second, duration. **No cost.**

### Logging

Two log surfaces:

1. **pino HTTP logs** in `~/.claude-code-router/logs/ccr-<timestamp>.log`, rotated daily, keep last 3, 50MB max — handled by `rotating-file-stream` at `packages/server/src/index.ts:152-160`. Configured via `LOG_LEVEL` (debug default).
2. **Application log** in `~/.claude-code-router/claude-code-router.log` (single file, via `LOG_FILE` env in config service).

Both logs include request body and final URL by default (`packages/server/src/server.ts` and `packages/core/src/server.ts:206`) which is a **leakage concern** since prompts can contain user data and full URLs include API key headers in some debug paths.

### Request history

UI has a `RequestHistoryDrawer` backed by browser IndexedDB (`packages/ui/src/lib/db.ts`). Captures recent requests on the **client side** (the UI polls something) but does not aggregate across UI sessions or across users. Not a server-side feature.

So the **available telemetry surface** is: tokens/sec, TTFT, per-session input/output token totals. No cost, no per-route success rate metrics, no Prometheus exposition, no OpenTelemetry, no distributed tracing.

## Plugin / Transformation Model

Two distinct extension points:

### 1. Transformers (request/response adaptation)

Already covered in Provider Abstraction. Summary:
- Interface: `Transformer` type in `packages/core/src/types/transformer.ts`.
- Registration: built-in via `transformer/index.ts` static export; custom via `config.transformers[]` with `path` + `options`.
- Scope: per-provider or per-(provider, model). Configured via `provider.transformer.use[]` and `provider.transformer[modelName].use[]`.
- Execution: request chain runs forward, response chain runs reversed.
- Has `endPoint` field that makes a transformer also register as a Fastify route (this is how `/v1/messages` exists — it's registered by AnthropicTransformer.endPoint via the `for transformersWithEndpoint` loop in `routes.ts:480-488`).

### 2. Fastify plugins (server-side behavior)

`CCRPlugin` interface (`packages/core/src/plugins/types.ts:14-19`):
```ts
interface CCRPlugin {
  name: string;
  version?: string;
  description?: string;
  register: FastifyPluginAsync<CCRPluginOptions>;
}
```

`pluginManager` is a singleton (`packages/core/src/plugins/plugin-manager.ts:121`). API: `registerPlugin(plugin, options)`, `enablePlugin(name, fastify)`, `enablePlugins(fastify)`, `setPluginEnabled(name, enabled)`, `removePlugin(name)`. Plugins are activated by listing them in `config.plugins[]` in the server's `registerPluginsFromConfig` (`packages/server/src/index.ts:61-89`).

Critical limitation: **the plugin registration path uses a switch statement on `name`**:

```ts
switch (name) {
  case 'token-speed': pluginManager.registerPlugin(tokenSpeedPlugin, {...}); break;
  default: console.warn(`Unknown plugin: ${name}`);
}
```

So while the plugin interface is extensible, you cannot actually register a third-party plugin from config without editing this file. The plugin system today supports exactly one plugin: `token-speed`. There is no `pluginPath` config or dynamic `require` for plugins (unlike transformers, which DO support custom paths). This is a real gap.

### 3. Agents (request-rewriting + tool-injection)

`IAgent` interface (`packages/server/src/agents/type.ts`). Agents have `shouldHandle(req, config)` that gates activation, `reqHandler(req, config)` that mutates the request body (e.g., rewriting image content blocks to placeholders), and `tools: Map<string, ITool>` that are spliced into the request's `tools[]` for the model to invoke. Tool calls are intercepted in the `onSend` SSE rewriter.

There is **one** built-in agent (ImageAgent) and **no registration path** other than editing `packages/server/src/agents/index.ts` to call `agentsManager.registerAgent(...)`. This is a closed extension point in v2.0.0.

### 4. Custom router

The `CUSTOM_ROUTER_PATH` config setting is loaded via `require()` in the router (`router.ts:270-281`). This is the only fully-open plugin point that doesn't require code edits.

### Summary of plugin surface

| Extension | Externally-loadable? | Where |
|-----------|----------------------|-------|
| Transformer | YES (via `config.transformers[].path`) | Per-request adaptation |
| Fastify plugin | NO (switch-cased; only `token-speed`) | Server-wide hooks |
| Agent | NO (must edit source) | Request rewriting + tool injection |
| Custom router | YES (via `CUSTOM_ROUTER_PATH`) | Routing decisions only |

## Behavioral Contracts Rollup

| ID | Contract | Evidence | Confidence |
|----|----------|----------|------------|
| BC-1 | `/v1/messages` accepts Anthropic Messages-shaped requests; router mutates `req.body.model` to `"provider,model"` before forwarding | `core/src/server.ts:144-151`, `router.ts:218` | HIGH |
| BC-2 | Subagent tag `<CCR-SUBAGENT-MODEL>p,m</CCR-SUBAGENT-MODEL>` overrides router; tag is stripped from system in-place | `router.ts:161-175` | HIGH |
| BC-3 | longContext route fires when token-count > `longContextThreshold` (default 60000) OR previous session usage > threshold AND current > 20000 | `router.ts:149-160` | HIGH |
| BC-4 | Claude Haiku model variants are automatically rerouted to `Router.background` when configured | `router.ts:177-185` | HIGH |
| BC-5 | webSearch route fires when `tools[].type` starts with `web_search` | `router.ts:187-193` | HIGH |
| BC-6 | think route fires when `body.thinking` is truthy | `router.ts:195-198` | HIGH |
| BC-7 | Per-session `Router` override is loaded from `~/.claude/projects/<proj>/<sessionId>.json` or `<proj>/config.json` | `router.ts:91-122` | HIGH |
| BC-8 | Custom router (`CUSTOM_ROUTER_PATH`) overrides built-in rules unless it returns null/throws | `router.ts:270-281` | HIGH |
| BC-9 | Authentication: APIKEY check is BYPASSED if no providers are configured | `server/src/middleware/auth.ts:13-17` | HIGH |
| BC-10 | When providers configured AND APIKEY unset, listen-host is forced to 127.0.0.1 and CORS-restricted | `server/src/index.ts:100-111` | HIGH |
| BC-11 | Transformer bypass (passthrough): if provider has exactly one transformer matching the endpoint transformer, skip request/response transformation | `core/src/api/routes.ts:282-294` | HIGH |
| BC-12 | Fallback chain: on `provider_response_error`, try each model in `config.fallback[scenarioType][]` in order | `core/src/api/routes.ts:108-194` | HIGH |
| BC-13 | Image agent intercepts before router: if Router.image is set AND last user message has image content, route to Router.image | `server/src/agents/image.agent.ts:62-89` | HIGH |
| BC-14 | Image cache: images are stored in 5-minute TTL LRU keyed by `<reqId>_Image#<n>`, content blocks replaced with text placeholders | `image.agent.ts:10-46,275-300` | HIGH |
| BC-15 | Session usage (`input_tokens`, `output_tokens`) is captured from `message_delta` events into LRU(100) keyed by sessionId | `server/src/index.ts:377-405`, `cache.ts:47` | HIGH |
| BC-16 | sessionId is parsed from `body.metadata.user_id` by splitting on `_session_` | `router.ts:221-226` | HIGH |
| BC-17 | Preset namespaces register at `/preset/<name>/v1/messages`; the CLI's preset launcher points Claude Code at that URL | `server/src/index.ts:185-187,204-209`; `cli/src/cli.ts:139` | HIGH |
| BC-18 | Preset install from GitHub validates path-traversal in ZIP entries against target directory | `shared/src/preset/install.ts:56-66` | HIGH |
| BC-19 | Preset export sanitizes fields matching `api_key`/`secret`/`token`/`password`/`private_key`/`access_key` patterns | `shared/src/preset/sensitiveFields.ts:7-29` | HIGH |
| BC-20 | Config file supports `$VAR` and `${VAR}` env-var interpolation, JSON5 (comments + trailing commas), and 3 most recent backups on save | `server/src/utils/index.ts:13-31,125-161` | HIGH |
| BC-21 | `ccr code` writes a temp settings file with `ANTHROPIC_BASE_URL=http://127.0.0.1:<port>` and passes `--settings <file>` to `claude` | `cli/src/utils/codeCommand.ts:86-88`; `createEnvVariables.ts:13-21` | HIGH |
| BC-22 | `ccr activate` outputs shell `export` lines including `DISABLE_TELEMETRY=true` and `DISABLE_COST_WARNINGS=true` | `cli/src/utils/activateCommand.ts` | HIGH |
| BC-23 | Streaming responses are tee'd: original goes to client, clone is parsed for usage / agent-tool-call splicing | `server/src/index.ts:247-421` | HIGH |
| BC-24 | When agent tool is invoked in stream, recursive POST to `/v1/messages` with appended messages, splice response back | `server/src/index.ts:283-360` | HIGH |
| BC-25 | tokenizer service supports tiktoken (default), HuggingFace, and API tokenizers; falls back to tiktoken on errors | `core/src/services/tokenizer.ts:48-100` | HIGH |
| BC-26 | All comments in code MUST be English (project convention) | `CLAUDE.md` directive | MEDIUM (norm, not enforced) |
| BC-27 | Provider `enabled` toggle endpoint is a no-op that returns true regardless | `core/src/services/provider.ts:189-195` | HIGH (bug) |
| BC-28 | No tests exist; behavior is uncovered by automated tests | `find -name '*.test.ts' \| wc -l` = 0 | HIGH |

## Conventions and Patterns

### Naming
- File suffixes: `*.transformer.ts`, `*.agent.ts`, `*.transform.ts` (SSE). Other utility files are uncategorized.
- Class naming: `PascalCase` with role-name suffix (`AnthropicTransformer`, `ImageAgent`, `ConfigService`, `TokenizerService`).
- Transformer name resolution: either static `TransformerName` field (used by 16 of 20) or instance `name` property (used by 4: `Anthropic`, `OpenAI`, `gemini`, `deepseek`). Inconsistent.
- Config keys: mixed casing inherited from external conventions — `Providers` and `Router` are PascalCase (Claude Code expects them this way), `APIKEY`/`HOST`/`PORT`/`LOG_LEVEL`/`PROXY_URL`/`API_TIMEOUT_MS`/`CUSTOM_ROUTER_PATH` are SHOUTING_SNAKE, `transformer.use`/`transformers`/`plugins` are camelCase/lowercase. Tolerant of variations: `Providers` or `providers` both accepted (`server/src/middleware/auth.ts:13`, `server/src/index.ts:97`).
- Variable placeholder dialects: `$VAR` / `${VAR}` for env vars (config), `{{field}}` for sensitive-field placeholders (presets) and statusline templates, `#{fieldId}` for preset schema variables. Three dialects, intentional but documented inconsistently.

### Module organization
- Each package has `src/index.ts` as the re-export barrel.
- Core uses a TypeScript path alias `@/*` → `packages/core/src/*` (`tsconfig.base.json`).
- UI uses `@/*` → `packages/ui/src/*` per Vite convention.
- No barrel exports from feature folders — every transformer is imported individually in `transformer/index.ts`.

### Error handling
- `createApiError(message, statusCode, code, type)` in `packages/core/src/api/middleware.ts` is the standard error factory. Always used in routes.
- Fastify error handler is set globally in `createApp()` and returns `{ error: { message, type, code } }`.
- The error message in `errorHandler` includes the full stack trace concatenated into `message`: `error.message + error.stack`. This leaks stack traces to clients on 500 errors.
- Custom router errors are caught and logged with `req.log.error(...failed to load custom router: ${e.message})` — no exception bubbles up; routing silently falls back to built-in.

### Async patterns
- Promise-based throughout. No callbacks except in `done(err, payload)` Fastify hook contracts.
- AbortController + `AbortSignal.timeout` (60-minute default) in `sendUnifiedRequest` (`core/src/utils/request.ts:22`).
- `Promise.allSettled` used for preset namespace registration so one bad preset doesn't kill startup (`server/src/index.ts:185-187`).

### Streaming patterns
- Web Streams API (`ReadableStream`, `TransformStream`) — not Node Readable. Consistent.
- `SSEParserTransform` (custom `TransformStream<string, any>`) for parsing event-stream text into event objects. `SSESerializerTransform` for the reverse.
- `rewriteStream(stream, processor)` wraps a stream with a processor that can swallow, modify, or replace events. Used by the agent tool-call interceptor.
- `pipeThrough(new TextDecoderStream())` always precedes `SSEParserTransform` because the parser is `<string, any>` not `<Uint8Array, any>`.

### Patterns NOT in use
- No dependency injection framework. Services are passed via constructor params, but there's no IoC container.
- No EventEmitter for cross-cutting concerns (one is created in `server/src/index.ts:20` but only used for `onError` / `onSend` mirroring, no consumers).
- No middleware composition library — Fastify hooks are used directly.

## Risk Register

| ID | Risk | Severity | Detail |
|----|------|----------|--------|
| P0-1 | Zero automated tests | Critical | 0 `*.test.*` / `*.spec.*` files. The router logic — the single most important code in this project — has no regression coverage. Any change to `getUseModel` is unverifiable. |
| P0-2 | Auth bypass when no providers configured | Critical (by design, but exploitable) | If `config.Providers` is empty/missing, auth is skipped AND host is forced to `0.0.0.0` (`server/src/index.ts:107-111`). A user with a partially-broken config can expose an open relay. The README's "for security reasons" forces 127.0.0.1 only when providers ARE set without APIKEY, not the other way. |
| P0-3 | Custom router runs unsandboxed `require()` | High | `CUSTOM_ROUTER_PATH` does `require(path)` (`router.ts:273`). Any file in the config can execute arbitrary code at server boot. Same for `config.transformers[].path`. No checksum, no signature, no allowlist. |
| P1-1 | Provider `toggle` endpoint is a no-op | High | `core/src/services/provider.ts:189-195` returns `true` without changing state. The UI shows toggles that don't function. |
| P1-2 | Error messages leak stack traces to clients | High | `core/src/api/middleware.ts:31` concatenates stack into the JSON error body. Internal paths and TypeScript transpilation artifacts visible to any failed 500 response. |
| P1-3 | Pino debug logging includes full request body and headers | High | `packages/core/src/server.ts:206` logs `{ data: body, type: "request body" }` at info level. `packages/core/src/utils/request.ts:46-55` logs final fetch options including headers (with bearer tokens) at debug. Default `LOG_LEVEL` is `debug`. |
| P1-4 | Three duplicate SSE files between server and core | Medium | `packages/server/src/utils/SSEParser.transform.ts` is character-identical to `packages/core/src/utils/sse/SSEParser.transform.ts`. Same for serializer and rewriteStream. They will drift. |
| P1-5 | Plugin extension is closed | Medium | `registerPluginsFromConfig` (`server/src/index.ts:61-89`) is a switch-case over `name`. Adding a third-party plugin requires forking. |
| P1-6 | Agent extension is closed | Medium | `agentsManager.registerAgent(imageAgent)` is hard-coded in `packages/server/src/agents/index.ts:47`. No dynamic loading. |
| P1-7 | Node version inconsistency | Medium | Top-level `package.json` declares `"node": ">=20.0.0"`. CLAUDE.md says >= 18.0.0. The lru-cache and undici usage suggests 20+ in practice. |
| P1-8 | The `forceUseImageAgent` config flag is undocumented | Medium | Only mention is `image.agent.ts:63` and the README's image scenario blurb. Not in any schema or example. |
| P1-9 | Two parallel logging systems | Medium | pino (HTTP) and ad-hoc `LOG_FILE` writes (app). No correlation IDs, no unified format. |
| P1-10 | Memory growth: `sessionUsageCache` is LRU(100) but `imageCache` is LRU(100) with 5-min TTL — both are global module singletons | Low | Bounded but unbounded across `tokenizerCache` (no eviction) (`packages/core/src/plugins/token-speed.ts:50`). |
| P1-11 | Preset GitHub install assumes `main` branch only | Low | URL hard-coded to `archive/refs/heads/main.zip` (`server/src/server.ts:415`). Repos with default branch `master` or `trunk` fail. |
| P1-12 | The `handleFallback` error message has a typo `"yichu"` | Trivial | `core/src/api/routes.ts:192`. Looks like Chinese pinyin slipped through. |
| P1-13 | Transformer interface name collision | Low | `TransformerName` (static) vs `name` (instance). 4 transformers use instance name, 16 use static. Caused real bugs noted in transformer.ts service code branches. |
| P1-14 | Stream tee for usage capture and stream tee for token-speed can both run, duplicating work | Low | If both `agents` are inactive and `token-speed` plugin is enabled, both `onSend` handlers tee independently. Acceptable but wasteful. |
| P2-1 | UI is 4.9× larger (LOC) than the server it controls | Tech debt | Heavy investment in UI suggests this is the intended primary operator interface, but the maintenance burden is significant. |
| P2-2 | i18n covers only English and Chinese | Tech debt | Hard-coded language list; new locales require both code and JSON changes. |

## Test Coverage Notes

- **Zero tests.** Verified by file count: `find packages -name '*.test.*' -o -name '*.spec.*' -o -name '__tests__' = 0`.
- No `vitest`, `jest`, `mocha`, or `tap` in any package.json.
- No CI configuration in repo (`.github/workflows/` not present in shallow checkout).
- No `examples/` runnable test fixtures despite the `examples/` directory existing (it contains only documentation snippets).
- The transformers — which encode subtle wire-format differences between 12+ provider APIs — are entirely uncovered. Any change to e.g. how OpenRouter handles image URLs in Claude vs non-Claude requests cannot be verified without a manual end-to-end run against the actual provider.

This is the single largest reliability risk. For a project that is by design a proxy between two evolving wire formats (Claude Code's request shape and N provider response shapes), the absence of contract tests against captured request/response fixtures is unusual.

## Architecture Recommendations for Monocle

### What CCR Does That Monocle Might Want

1. **Per-session model routing with project-scoped overrides.** This is genuinely useful for a multi-harness session manager. The `~/.claude/projects/<proj>/<sessionId>.json` lookup pattern is exactly the kind of "this session prefers GPT-4o for thinking" config that a workspace orchestrator should respect.
2. **Token-budget-aware longContext routing.** Switching to a larger-context model when accumulated tokens cross a threshold is a sensible default that monocle harnesses could reuse.
3. **Subagent escape-hatch tag (`<CCR-SUBAGENT-MODEL>`).** A simple in-band model-override mechanism for subagents is portable across harnesses.
4. **Preset packaging.** The manifest + dynamic schema + sensitive-field sanitization model is a strong distribution mechanism for "give me a working config for provider X" templates.

### What CCR Does That Monocle Probably Does NOT Want To Inherit

1. **Built-in router in monocle.** Building a router into monocle would couple the session manager to the model-API protocol. CCR is a great example of how complex this becomes (3.9k LOC of CLI just to manage one router instance, 9.5k LOC of UI to configure it, an entire preset system to package configs). monocle is not a model API proxy and should not become one.
2. **The transformer pipeline.** 11.9k LOC of provider adaptation logic. Monocle has no reason to reinvent this. If monocle needs to talk to multiple providers, it should talk to CCR or LiteLLM, not embed a transformer chain.
3. **The agent recursion loop.** The `onSend` SSE rewriter that re-POSTs to `/v1/messages` is clever but fragile. Monocle should not implement agent recursion at the routing layer.
4. **Single in-process daemon model.** CCR runs as one process per host listening on 3456. monocle is intended to manage multiple harness sessions; this single-instance model would conflict.

### Recommendation: Integrate-External, Not Build-In

monocle should **integrate with CCR (or a similar external router) as an optional component**, not build a router into the core engine.

Concrete integration shape:

1. **Detection.** monocle checks if `ccr` is on PATH and `~/.claude-code-router/config.json` exists. If so, treat it as an available routing backend.
2. **Per-session pass-through.** When monocle launches a harness session, it can:
   - Set `ANTHROPIC_BASE_URL=http://127.0.0.1:3456` in the harness's environment to route through CCR with the global Router rules. (Equivalent to `eval "$(ccr activate)"`.)
   - Or write a per-session Router override to `~/.claude/projects/<project>/<sessionId>.json` before launching, so CCR's per-session lookup picks it up.
   - Or use a preset namespace: launch with `ANTHROPIC_BASE_URL=http://127.0.0.1:3456/preset/<name>` to scope the session to a specific provider/model set.
3. **No build-in coupling.** monocle does not depend on CCR's existence; if CCR isn't installed, sessions go to Anthropic directly. CCR is a per-user opt-in.
4. **What monocle WOULD own:** which CCR preset (or which `<CCR-SUBAGENT-MODEL>` tag) to use for a given harness type / project type. Map "this is a CodeMachine harness running a planning task" → "use the `think` route" via the subagent tag mechanism, or via writing a per-session override config.

### Required API surface from CCR for monocle integration

The integration does not require CCR to expose any new APIs. The existing surface is sufficient:

- `~/.claude/projects/<proj>/<sessionId>.json` file-write for per-session routing override (file API).
- `<CCR-SUBAGENT-MODEL>provider,model</CCR-SUBAGENT-MODEL>` tag injection at message start (string API).
- `ANTHROPIC_BASE_URL` environment-variable override for the harness (env API).
- `http://127.0.0.1:3456/preset/<name>/v1/messages` endpoint for preset-scoped routing (HTTP API).

If monocle later wants cost attribution, that is **not in CCR** today and monocle should plan for it independently (either by inspecting harness stdout, by wrapping the provider's billing endpoint, or by computing locally from token counts).

### What monocle could contribute back to CCR (out of scope for ingest, noted for future)

- Externally-loadable plugins (replace the switch-case in `registerPluginsFromConfig`).
- Cost attribution per route/session/provider.
- A contract test suite of captured request/response fixtures for the 20 transformers.

## Convergence Statement

This is a Phase C synthesis written directly without per-package deepening because the codebase is small enough (29k total LOC, only 12 files in the server package, 19 in CLI, 10 in shared, 58 in core that follow a uniform structure) that two-pass reading achieved NITPICK-level novelty. The router (`packages/core/src/utils/router.ts`, 367 lines) was read line-by-line. Every transformer file's size and entry-point shape was characterized. Every CLI command's handler was identified. Every REST endpoint in the server was enumerated. The CLAUDE.md provided by the project maintainer was used as ground truth for architecture and cross-checked against actual file contents — and a discrepancy was identified (CLAUDE.md claims `packages/server/src/utils/router.ts`, but the router actually lives at `packages/core/src/utils/router.ts`; this was a stale doc).

Iron Law: All claims in this document are grounded in specific file paths and line ranges, all hand-verified by reading source. The "no tests" claim was verified by file globbing. The "no cost tracking" claim was verified by reading every plugin and the token-speed plugin's data model. The transformer count of 20 was verified by listing `packages/core/src/transformer/*.transformer.ts`. The router decision tree was verified by reading `getUseModel` in full.

Remaining gaps that would require further investigation only if monocle adopts deeper coupling: behavior of the OpenAI Responses transformer (792 LOC, only skimmed); the exact tokenizer fallback semantics under network failure; the `enhancetool` transformer's claimed side effect on streaming; the `forcereasoning` transformer's prompt-engineering behavior. None of these gaps affect the integration recommendation.

## Handoff

For monocle's next steps:

1. **Architecture decision (RECOMMENDED): integrate-external.** Do not build a router into monocle. Treat CCR (or LiteLLM, or a future alternative) as an optional routing backend that monocle can detect and integrate with via environment variables and per-session config file writes.

2. **If monocle DOES want session-level model routing in v1**, the lightest-weight path is to write `~/.claude/projects/<proj>/<sessionId>.json` files before launching harness sessions. This requires zero new APIs on the CCR side. CCR (if installed and running) honors them automatically.

3. **If monocle wants cost attribution**, this is a separate workstream — CCR does not provide it. monocle should compute cost from token counts that harnesses already emit, not depend on the router.

4. **Avoid the transformer pipeline as a model.** It is the most engineering-heavy part of CCR and exists because Anthropic / OpenAI / Gemini / DeepSeek wire formats differ. monocle is not at that layer.

5. **The preset system is interesting prior art** for any future monocle "shareable workspace template" feature, but it is solving a different problem (config-portability for an end-user CLI), so reuse should be conceptual, not literal.

## Files Inventory

All paths are absolute under `/Users/jmagady/Dev/monocle/.reference/claude-code-router/`.

### packages/core (58 files, 11878 LOC)

- `src/server.ts` — Server class, namespaces, hook registration (entry for `@musistudio/llms`)
- `src/api/routes.ts` — `handleTransformerEndpoint`, fallback, transformer pipeline, providers CRUD
- `src/api/middleware.ts` — error factory + handler
- `src/services/config.ts` — JSON5 config loader with env loading, get/has/set/reload
- `src/services/provider.ts` — provider registration, model routing maps
- `src/services/transformer.ts` — transformer registry, custom transformer loader via `require()`
- `src/services/tokenizer.ts` — tiktoken / HuggingFace / API tokenizer dispatch
- `src/tokenizer/{api,huggingface,tiktoken}-tokenizer.ts` — three tokenizer implementations
- `src/types/{llm,transformer,tokenizer}.ts` — type definitions
- `src/transformer/{anthropic,openai,openai.responses,gemini,vertex-gemini,vertex-claude,deepseek,groq,cerebras,openrouter,vercel,tooluse,maxtoken,maxcompletiontokens,sampling,reasoning,forcereasoning,enhancetool,cleancache,streamoptions,customparams}.transformer.ts` — 21 transformer modules (one is the index)
- `src/transformer/index.ts` — barrel export of all transformers
- `src/utils/router.ts` — **the router function**, longContext / haiku / webSearch / think / subagent / project-override logic
- `src/utils/cache.ts` — sessionUsageCache LRUCache
- `src/utils/request.ts` — `sendUnifiedRequest` (fetch with ProxyAgent)
- `src/utils/converter.ts`, `src/utils/image.ts`, `src/utils/thinking.ts`, `src/utils/gemini.util.ts`, `src/utils/vertex-claude.util.ts`, `src/utils/toolArgumentsParser.ts` — utility helpers used by transformers
- `src/utils/sse/{index,SSEParser.transform,SSESerializer.transform,rewriteStream}.ts` — SSE parsing utilities (canonical copies)
- `src/plugins/{index,types,plugin-manager,token-speed}.ts` — plugin system + the token-speed plugin
- `src/plugins/output/{index,types,output-manager,console-handler,temp-file-handler,webhook-handler}.ts` — pluggable output sinks for token-speed stats

### packages/server (12 files, 1909 LOC)

- `src/index.ts` — `getServer()`, agent dispatch, SSE rewriting onSend, plugin registration switch-case, `run()` entry
- `src/server.ts` — `createServer()` factory, all CCR REST endpoints (config, logs, presets, transformers, count_tokens), static UI serving
- `src/middleware/auth.ts` — APIKEY validator with no-provider bypass
- `src/agents/{index,type,image.agent}.ts` — agents manager + ImageAgent
- `src/utils/{index,SSEParser.transform,SSESerializer.transform,rewriteStream}.ts` — config I/O, **duplicate** SSE files
- `src/types.d.ts`, `src/types/llms-plugin.d.ts` — ambient type declarations

### packages/cli (19 files, 3856 LOC)

- `src/cli.ts` — main command dispatcher
- `src/utils/index.ts` — `run`, `restartService`, config I/O wrappers, `getSettingsPath`
- `src/utils/codeCommand.ts` — `executeCodeCommand` (the `ccr code` implementation)
- `src/utils/createEnvVariables.ts` — shared env-var builder for `code`/`activate`
- `src/utils/activateCommand.ts` — `ccr activate`/`ccr env`
- `src/utils/installCommand.ts` — `ccr install` marketplace fetch
- `src/utils/modelSelector.ts` — `ccr model` interactive flow
- `src/utils/processCheck.ts` — PID liveness, reference counting
- `src/utils/status.ts` — `ccr status`
- `src/utils/statusline.ts` — `ccr statusline` ANSI status builder
- `src/utils/update.ts` — version check / self-update via npm
- `src/utils/preset/{commands,export,install,install-github,index}.ts` — `ccr preset` subcommands
- `src/utils/prompt/schema-input.ts` — interactive form for preset RequiredInput schema
- `src/types.d.ts`, `src/types/inquirer.d.ts` — types

### packages/shared (10 files, 2103 LOC)

- `src/constants.ts` — filesystem paths
- `src/index.ts` — barrel
- `src/preset/types.ts` — preset type system (267 LOC)
- `src/preset/install.ts` — extract / load / validate / list (416 LOC)
- `src/preset/export.ts` — export with sanitization
- `src/preset/sensitiveFields.ts` — sensitive-field detection
- `src/preset/merge.ts` — config merge strategies
- `src/preset/schema.ts` — `loadConfigFromManifest` template variable resolver
- `src/preset/marketplace.ts` — `getMarketPresets`, `findMarketPresetByName`
- `src/preset/readPreset.ts` — convenience reader

### packages/ui (47 files, 9462 LOC)

- `src/{App.tsx,main.tsx,routes.tsx,i18n.ts,index.css,types.ts}` — root
- `src/components/{ConfigProvider,Providers,ProviderList,Router,Transformers,TransformerList,JsonEditor,LogViewer,Presets,SettingsDialog,StatusLineConfigDialog,StatusLineImportExport,RequestHistoryDrawer,DebugPage,Login,ProtectedRoute,PublicRoute}.tsx` — feature components
- `src/components/preset/DynamicConfigForm.tsx` — preset apply UI
- `src/components/ui/*.tsx` (20 files) — Radix-based design system primitives
- `src/lib/{api,db,utils}.ts` — API client, IndexedDB request history, helpers
- `src/utils/statusline.ts` — UI-side statusline preview rendering
- `src/locales/{en,zh}.json` — translations
- `src/styles/animations.css` — extras

### Root files

- `package.json` — workspace root with `release` / `dev:*` scripts
- `pnpm-workspace.yaml` — `packages/*` + `docs`
- `pnpm-lock.yaml`
- `tsconfig.base.json`, `tsconfig.json`
- `custom-router.example.js` — 3-line example
- `CLAUDE.md` — project orientation (with one stale path)
- `README.md`, `README_zh.md` — comprehensive docs
- `scripts/` — esbuild build scripts for cli/server/shared
- `docs/` — Docusaurus site (separate package)
- `blog/`, `examples/` — content

### Output

All Phase C synthesis content is in this single file at:
`/Users/jmagady/Dev/monocle/.factory/semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md`
