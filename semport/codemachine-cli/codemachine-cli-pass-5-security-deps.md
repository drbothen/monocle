# Pass 5: Security & Dependencies — codemachine-cli

## Dependency Inventory

### Runtime dependencies (23, from `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:71-95`)

| Package | Version range | Role | Risk profile |
|---|---|---|---|
| `@clack/prompts` | `^0.11.0` | Interactive prompts (used in onboarding, auth flows) | LOW — well-known, no security issues |
| `@modelcontextprotocol/sdk` | `^1.0.0` | MCP server + client | LOW — official Anthropic SDK; new (v1) but actively maintained |
| `@opentelemetry/api` | `^1.9.0` | OTel API surface | LOW |
| `@opentelemetry/api-logs` | `^0.211.0` | Logs API (pre-1.0; volatile) | MED — `0.x` semver implies breaking-change risk |
| `@opentelemetry/exporter-logs-otlp-http` | `^0.211.0` | Logs OTLP HTTP exporter | MED — same volatile-version-range concern |
| `@opentelemetry/exporter-metrics-otlp-http` | `^0.211.0` | Metrics OTLP HTTP exporter | MED |
| `@opentelemetry/exporter-trace-otlp-http` | `^0.211.0` | Traces OTLP HTTP exporter | MED |
| `@opentelemetry/exporter-zipkin` | `^2.5.0` | Zipkin trace exporter | LOW |
| `@opentelemetry/resources` | `^2.5.0` | OTel resource attributes | LOW |
| `@opentelemetry/sdk-logs` | `^0.211.0` | Logs SDK | MED |
| `@opentelemetry/sdk-metrics` | `^2.5.0` | Metrics SDK | LOW |
| `@opentelemetry/sdk-node` | `^0.211.0` | Node SDK bundle | MED |
| `@opentelemetry/sdk-trace-base` | `^2.5.0` | Base trace SDK | LOW |
| `@opentelemetry/semantic-conventions` | `^1.39.0` | OTel attribute name constants | LOW |
| `@opentui/core` | `0.1.73` | Terminal UI core (pinned exact, not `^`) | MED — early-version pin; ecosystem-thin |
| `@opentui/solid` | `^0.1.73` | Solid.js renderer for OpenTUI | MED — same; `^0.1.x` allows minor bumps |
| `chalk` | `^5.6.2` | ANSI color helpers | LOW |
| `commander` | `^14.0.1` | CLI parsing | LOW |
| `fuzzysort` | `^3.1.0` | Fuzzy search for TUI command palette | LOW |
| `proper-lockfile` | `^4.1.2` | Cross-process file lock for log files | LOW |
| `solid-js` | `^1.9.10` | Solid reactive primitives | LOW |
| `update-notifier` | `^7.3.1` | npm-update awareness | LOW |
| `zod` | `^3.23.0` | JSON Schema validation for MCP tool inputs | LOW |

### Dev dependencies (16)

| Package | Version | Role |
|---|---|---|
| `@eslint/js` | `^9.36.0` | ESLint core configs |
| `@types/{babel__core,bun,node,proper-lockfile,update-notifier}` | various | type defs |
| `@typescript-eslint/eslint-plugin` | `^8.49.0` | TS ESLint rules |
| `@typescript-eslint/parser` | `^8.49.0` | TS ESLint parser |
| `eslint` | `^9.0.0` | linter |
| `eslint-config-prettier` | `^10.1.8` | disable conflicting rules |
| `eslint-import-resolver-typescript` | `^4.4.4` | TS path resolution for import plugin |
| `eslint-plugin-import` | `^2.29.1` | import lints |
| `husky` | `^9.0.7` | git hooks |
| `prettier` | `^3.7.4` | formatter |
| `typescript` | `^5.4.5` | TS compiler |
| `typescript-eslint` | `^8.49.0` | TS ESLint flat-config helper |

### Optional dependencies (5)

Platform-specific binaries (`codemachine-{linux-x64, linux-arm64, darwin-arm64, darwin-x64, windows-x64}` all at `0.8.0`). These ARE the actual compiled CLI — the main package's `bin/codemachine.js` selects one at runtime.

### Bun-Specific Built-ins (Not in package.json)

Used directly via Bun's auto-resolved built-ins; ESLint resolver ignores `^bun:` prefix (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/eslint.config.js:45-50`):

| Built-in | Usage |
|---|---|
| `bun:sqlite` | Monitoring DB (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/agents/monitoring/db/schema.ts:1`) |
| `bun:test` | Used in archived tests (no current usage) |
| `Bun.spawn`, `Bun.which`, `Bun.build`, `Bun.file` | Process spawn, command resolve, build, file read |
| `Bun.main` | Module-as-main detection (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:480-490`) |
| `Bun.$` | Shell in scripts (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/scripts/build.ts:7`) |

This is **a hard runtime dependency on Bun ≥ 1.3.3** (`package.json:41`).

## Security-Relevant Patterns

### S1 — Dangerous flags passed to engine CLIs

CodeMachine deliberately passes "bypass permissions/sandbox" flags to every engine to enable headless automation. This is **a deliberate trust delegation**: the user is running CodeMachine, so CodeMachine acts with the user's full file/system permissions, and the engines run within that same trust boundary.

| Engine | Bypass flag(s) | Citation |
|---|---|---|
| Claude | `--dangerously-skip-permissions`, `--permission-mode bypassPermissions` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/commands.ts:53-57` |
| Codex | `--sandbox danger-full-access`, `--dangerously-bypass-approvals-and-sandbox`, `--skip-git-repo-check` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/codex/execution/commands.ts:20-26` |
| Others | TBD per Pass B deep read |

**Implication for monocle:** Adopting this pattern adopts the same trust model — agents run with no in-engine sandboxing. The user's safety net is process-level isolation (run inside a container/VM, etc.), not engine-level review.

### S2 — Environment variable passthrough is broad

`runClaude` constructs `mergedEnv = {...process.env, ...env, CLAUDE_CONFIG_DIR, MCP_TIMEOUT, MCP_TOOL_TIMEOUT}` then conditionally adds `ANTHROPIC_*` and `CLAUDE_OAUTH_TOKEN` if set. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts:189-210`.

Net effect: child process inherits **the full parent environment**, including secrets, AWS creds, SSH agent, etc. This is standard but means **any compromised engine CLI can exfiltrate everything CodeMachine has access to**.

### S3 — Authentication credentials

Per `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/auth.ts:38-49`:

- Claude credentials live at `${CLAUDE_CONFIG_DIR}/.credentials.json`. Default `CLAUDE_CONFIG_DIR = ~/.codemachine/claude`.
- Settings file (`settings.json`) may contain `env.ANTHROPIC_API_KEY` or `env.ANTHROPIC_AUTH_TOKEN` — these are also treated as authenticated state (`hasAuthInSettings`, line 56-77 referenced).
- File permissions: the code uses Node's `readFile/writeFile` with no explicit mode. The default umask applies — credentials are NOT explicitly `chmod 600`'d. **MED risk** on shared systems.
- Cleanup: `clearAuth` (in `core/auth.ts`) deletes the credentials path; not yet read in deepening.

### S4 — MCP signal queue is filesystem-backed at `~/.codemachine/mcp/workflow-signals/`

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/mcp/servers/workflow-signals/index.ts:14-15`. Per-user, world-writable depending on umask. Multi-user systems could see cross-user signal injection if homedirs are shared (unusual but possible).

### S5 — Stream JSON parsing is `JSON.parse` with try/catch

The Claude runner does `JSON.parse(line)` per line, catching errors silently. `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/engines/providers/claude/execution/runner.ts:101-167, 244-272`. Adversarial output (e.g., long line that crashes parser via depth) is mitigated by the line-by-line approach and Node's JSON.parse not having a depth-bomb known weakness. **LOW risk**.

### S6 — `--dir <path>` is path-traversal-unconstrained

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:79-84` accepts arbitrary directory paths from CLI. The only check is the home-dir veto (S7). User could `codemachine --dir /etc` and the CLI would `chdir` there. **By design** — running in arbitrary directories is intended.

### S7 — Home directory hard refusal

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:77-117`. Refuses to run if resolved target == resolved $HOME. This is a UX guard (so the user doesn't accidentally fill their homedir with `.codemachine/`), not a security control.

### S8 — Network endpoints

OpenTelemetry exporters default to `http://localhost:4318/v1/{traces,logs,metrics}` (OTLP HTTP standard). No outbound except `update-notifier` (npm registry) and OTel (localhost by default). **LOW** outbound surface.

`update-notifier` semantics: phones home to npm registry to check for newer version. Can be disabled via standard `NO_UPDATE_NOTIFIER=1` env var; not explicitly configured here.

### S9 — Child process command resolution

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:29-53` (`resolveCommandExecutable`):
- If command has path separator, used as-is.
- Else: `Bun.which(command)`. If null, falls through to using the bare name (relies on `$PATH` resolution by `Bun.spawn`).
- Special-case: `'bun'` → `process.execPath` (the running Bun binary).

If `$PATH` contains an attacker-writable directory before `/usr/local/bin`, an engine-named binary could be hijacked. **Same risk as any CLI**; standard mitigation (sanitized $PATH) applies.

### S10 — `Bun.spawn` stdin handling

`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/infra/process/spawn.ts:176-178, 205-213`: prompt is `TextEncoder.encode()`'d to bytes and passed as `stdin`. No shell interpolation, no command-injection vector. **LOW risk**.

### S11 — `--dangerously-skip-permissions` for Claude

Claude Code's permission prompt is bypassed. CodeMachine cannot ask the user to confirm file writes or command execution. Trust is fully delegated to the agent. By design — same trust model as S1.

## License & Distribution

- License: Apache-2.0 (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:5`).
- Author: "CodeMachine Contributors" (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:6`).
- Distributed as a hidden bundled binary (per-platform) plus a shell wrapper. **Source is not the source of truth at runtime** — the user runs a Bun-compiled native binary built via `Bun.build({ compile: ... })` (`/Users/jmagady/Dev/monocle/.reference/codemachine-cli/scripts/build.ts:158-175`).

## Dependency Risk Assessment

| Concern | Severity | Notes |
|---|---|---|
| `@opentelemetry/*-logs` at `0.x` semver | MED | Multiple OTel packages on `^0.211.0` — minor bumps can break. Active flight (telemetry migration plan) suggests team is actively tracking. |
| `@opentui/core` pinned at exact `0.1.73` | LOW-MED | Pinned for stability; early-version package; small ecosystem. Adoption risk for monocle if they want to lift the TUI. |
| MCP SDK `^1.0.0` | LOW | Recent v1 release; major versions are stable contracts in MCP. |
| No `package-lock.json` (Bun-native `bun.lock` used) | LOW | Reproducibility via `bun install --frozen-lockfile` in CI. |
| Bun runtime as hard dependency | HIGH (for portability) | Cannot run on Node.js without Bun's compat layer. Two-implementation drift risk. |
| Dev binaries shipped as optionalDependencies | LOW | Per-platform packages with `os` + `cpu` constraints prevent wrong-platform installs. |

## State Checkpoint

```yaml
pass: 5
status: complete
runtime_dependencies: 23
dev_dependencies: 16
optional_platform_binaries: 5
security_observations: 11 (S1-S11)
critical_security_issues: 0
note: |
  S1, S2, S11 are deliberate trust-delegation decisions, not bugs. Monocle
  inherits these trust assumptions if it lifts the same pattern. Documented
  in spec rather than flagged as risk.
timestamp: 2026-05-11T00:05:00Z
next_pass: 6
```
