# Pass 4: Verification Gaps — codemachine-cli

This pass enumerates the lack of automated verification and the resulting unknowns.

## P0 Finding — Zero Tests in the Repository

**Quantitative claim:**

```
find /Users/jmagady/Dev/monocle/.reference/codemachine-cli -name '*.test.ts' -not -path '*/node_modules/*' -not -path '*/.git/*'  → 0 files
find /Users/jmagady/Dev/monocle/.reference/codemachine-cli -name '*.spec.ts' -not -path '*/node_modules/*' -not -path '*/.git/*'  → 0 files
grep -l 'describe(' src/**/*.ts                                                                                          → no matches
grep -l 'expect('   src/**/*.ts                                                                                          → no matches (except mock UI adapter)
find /Users/jmagady/Dev/monocle/.reference/codemachine-cli -type d -name tests                                            → none
find /Users/jmagady/Dev/monocle/.reference/codemachine-cli -type d -name '.tests.archive'                                  → none
```

**Documented intent:**

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:59` defines `"test": "bun test"`, `"test:watch": "bun test --watch"`, `"test:coverage": "bun test --coverage"`.
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/tsconfig.json:21` excludes `"tests"`.
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/eslint.config.js:6` includes `'tests/**/*.{ts,tsx}'` in the lint glob.
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/bunfig.toml:5` says: "# Tests archived to .tests.archive/ (hidden folder, ignored by bun test)" — but **the `.tests.archive/` folder is not present** in the cloned repo.

**Interpretation:** Tests existed historically and were removed (archived to a hidden folder that is not committed). The lint config, tsconfig, and package.json scripts still reference the old test paths but they're inert. The repo currently ships with **no executable verification of any kind**.

**Severity:** P0 for confidence in any contract drawn from code-reading. Every BC-DRAFT in Pass 3 could be subtly wrong without showing up in any local check beyond `bun run typecheck` and `bun run lint`.

## CI-Level Verification

| Step | What it actually verifies | Citation |
|---|---|---|
| `bun install --frozen-lockfile` | Dependency tree matches `bun.lock` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/.github/workflows/build.yml:43-44` |
| `bun run build` (per matrix platform) | Bundle compiles, no TS errors | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/.github/workflows/build.yml:46-53` |
| `grep -q '"cm"' binaries/.../package.json` | The `cm` alias exists in produced platform package | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/.github/workflows/build.yml:55-62` |
| `binary --version` smoke | Binary executes (continue-on-error: true → soft failure) | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/.github/workflows/build.yml:64-76, 84-89` |
| Cross-compile existence check | File present + `file` output for ARM cross builds | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/.github/workflows/build.yml:91-101` |
| (PR/dev) `bun run lint` `--max-warnings=0` | ESLint + import resolver + tseslint recommended | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:58`, `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/CONTRIBUTING.md` |
| (PR/dev) `bun run typecheck` | `tsc --noEmit` | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:63` |
| Husky pre-commit | Hook installed via `husky install`; specific hooks not visible in repo | `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/package.json:67` |

**Net verification at PR time:** type checks + ESLint + linker can produce a binary. **No behavior tested.**

## What This Means for Behavioral Confidence

| Subsystem | Code-derived confidence | Risk if monocle lifts this gene |
|---|---|---|
| Engine plugin abstraction (`EngineModule` + registry) | HIGH (small, declarative) | LOW — easy to verify; clear contract |
| Per-engine command builder | HIGH per provider read; MED for engines not yet deep-read (auggie, opencode, mistral, ccr) | MED — each engine's CLI flags are externally constrained but the builder logic is small |
| Process spawn + abort + group-kill | MED-HIGH | MED — race conditions on abort/exit are exactly what tests usually catch; group-kill behavior on Windows is fall-back code path |
| Workflow FSM transitions | HIGH (declarative table) | LOW — easy to formally verify via state-space enumeration |
| Workflow runner loop (state dispatch, mode handlers) | MED | HIGH — concurrent signal arrival during state transitions is the kind of bug zero-test repos hide |
| Crash recovery | LOW-MED | HIGH — requires multi-process simulation; impossible to validate without integration tests |
| Chained prompt loading and filtering | MED | MED — frontmatter parsing is hand-rolled (YAML library not used); edge cases could exist around colons in description |
| MCP tool servers | MED | LOW — interface is JSON Schema validated by Zod; the surface is small |
| Coordinator script parsing | LOW | HIGH — multi-token operator precedence parsing without tests is a common source of subtle bugs |
| TUI Solid components | LOW | LOW — TUI bugs are visible to user; monocle is unlikely to lift the TUI verbatim |
| OTel boot wiring | MED | LOW — opt-in via env var; bypassed on default path |
| SQLite migration on `session_id` column | LOW | MED — migration not exercised on every schema version path |

## TODO Density and Refactor Debt

The codebase has minimal in-source TODOs; the `grep -c TODO src/**/*.ts` returns 0 per top file. However, `cli-setup.ts` carries explicit TODO comments at:

- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:6` — "Legacy - appDebug file bootstrap is superseded by OTel telemetry. Remove after confirming no side effects."
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:152` — "Misplaced concern - boot phase metric wiring should be in a dedicated boot metrics module. Move during refactoring."
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:173` — "Misplaced concern - telemetry process lifecycle handlers should be owned by telemetry runtime initialization. Move during refactoring."
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:269` — "Move spec path handling to template level, not cli-setup level"
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/runtime/cli-setup.ts:309` — "Remove this block once confirmed unused" (workspace bootstrap legacy)
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/preflight.ts:76` — "Re-enable project name check - temporarily disabled due to persistence bug"
- `/Users/jmagady/Dev/monocle/.reference/codemachine-cli/src/workflows/templates/loader.ts` — no explicit TODOs but the dual-path conditional load (.cjs vs .js) suggests prior pain.

**Plus the entire `TELEMETRY_MIGRATION_PHASE_PLAN.md` (30KB doc)** which describes a phased refactor of the logging subsystem.

## Unknowns From This Reading

Things that I would expect to be verified with tests, but couldn't confirm from code alone:

1. **What is the exit behavior of `runWorkflow` in subcommand-only mode?** It blocks on a never-resolving Promise if `eventBus.hasSubscribers()`. In TUI mode this is fine. But `codemachine run <script>` doesn't go through `runWorkflow`, it goes through `CoordinatorService` — so the headless path bypasses this hang. Need to verify all subcommands' termination.
2. **Concurrent signal handling.** What if `workflow:skip` and `workflow:pause` arrive in the same tick? The handlers are async, registered separately, and both `.abort()` the controller. No test confirms ordering.
3. **Crash recovery across version upgrades.** `template.json` schema changes between versions — no migration framework visible (other than SQLite-column migration in `db/schema.ts:38-51`).
4. **Process-group kill on Windows.** Comment says Windows uses `child.kill('SIGTERM')` only — no test confirms that Windows wrapper-script children also die.
5. **Engine fallback chain when ALL auth checks fail with errors (not just unauthenticated).** Behavior may differ if `auth.isAuthenticated()` itself throws — not exercised in code paths I read.
6. **MCP signal queue file location.** Code says `~/.codemachine/mcp/workflow-signals/` — what happens if `$HOME` is unwritable, NFS-mounted with `noexec`, etc.?
7. **Coordinator parser precedence.** `a && b & c && d` — what does this mean? No grammar documented; needs Pass B deep-read of `src/agents/coordinator/parser.ts`.
8. **`getDevRoot()` semantics.** Used by template loader and chained-prompts loader. When does it return null? What's the fallback? Pass B deep-read needed.

## Recommendations for Monocle

If monocle adopts patterns from CodeMachine:

1. **Add tests for the FSM table first.** It's the highest-leverage testable surface; the transitions are declarative and pure functions. Property-test the state space.
2. **Integration-test crash recovery.** This is the riskiest behavior to lift.
3. **Snapshot-test command builders.** `buildClaudeExecCommand({…})` → frozen output. Catches accidental flag changes.
4. **Don't lift the auth cache as-is.** The TTL-without-invalidation is a known UX wart.
5. **Don't lift `globalThis.__workflowEventBus`.** Replace with DI.
6. **Lift the engine plugin contract verbatim.** It's clean and small enough that errors will surface in normal use.

## State Checkpoint

```yaml
pass: 4
status: complete
test_files_found: 0
ci_verification: build + lint + typecheck only (no test execution)
unknowns_carrying_to_pass_b: 8
p0_finding: confirmed (no automated verification of behavior)
timestamp: 2026-05-11T00:04:00Z
next_pass: 5
```
