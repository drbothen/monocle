# Pass B Deepening — `internal/plugin/` Subsystem (Round 1)

**Subsystem:** `internal/plugin/` — Claude Code plugin manager (wraps `claude plugins` CLI)
**Reference path:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/plugin/`
**Total LOC:** 1,223 (production: 429, test: 794) per Pass 0 inventory
**Files (6):** `manager.go` (157 LOC), `cli.go` (197 LOC), `model.go` (77 LOC), `manager_test.go` (324 LOC), `cli_test.go` (283 LOC), `model_test.go` (187 LOC)
**Prior coverage:** Pass 8 final synthesis row at `pass-8-final-synthesis.md:65,82,116,329,346,437,671,698,702,729` flags the package as LOW direct relevance, no BC-PLUGIN-* contracts authored in Pass 3. P1-009 names plumb-cancellation. This round produces first-time, monocle-targeted deep extraction.

---

## 1. Architecture (file:line citations)

### 1.1 Three-layer composition

```
GUI (internal/gui)
  -> PluginProvider interface (gui/plugin_state.go:28-39)
  -> pluginAdapter (cmd/lazyclaude/root.go:686-744)
  -> plugin.Manager (manager.go:11-19)
  -> plugin.ExecCLI (cli.go:41-44)
  -> Runner interface -> execRunner (cli.go:13-37)
  -> `claude plugins <subcmd>` subprocess
```

The package is a **pure shell-out adapter**. It has no understanding of plugin file formats, no manifest schema, no on-disk paths it manages itself. All state lives in the `claude` CLI. The Go package exists purely to:

1. Convert TUI actions into `claude plugins ...` argv vectors (`cli.go:82-197`).
2. Cache the last JSON result so the GUI can render synchronously without re-shelling on every paint (`manager.go:16-19,75-101`).
3. Hold the current "project directory" so subprocesses run with the right `cmd.Dir` for project-scope CLI semantics (`cli.go:43,49-51`, `manager.go:34-36`).

### 1.2 Wiring (single instance, global, no daemon)

`cmd/lazyclaude/root.go:340-347` constructs one `*plugin.Manager` per TUI process. There is no plugin daemon; everything is in-process around the gocui GUI. The Manager is handed to `pluginAdapter` (root.go:686-744) which is registered with `app.SetPlugins(...)` (root.go:347).

The adapter performs three type conversions only:
- `InstalledPlugin` -> `gui.PluginItem` (strips `InstallPath`, `LastUpdated`) at `root.go:702-710`
- `AvailablePlugin` -> `gui.AvailablePluginItem` (strips `Source`) at `root.go:717-725`
- `Install` always passes `"project"` scope (root.go:730), ignoring the `scope` arg that would otherwise be passed through. Marketplace install is project-scope-only by GUI design.

### 1.3 No marketplace integration in this Go package

The `claude` CLI handles marketplace fetch / add / remove / update entirely. The plugin package only knows the CLI verbs (`cli.go:169-197`). Marketplace state (which marketplaces are configured, install location, etc.) comes back as `MarketplaceInfo` JSON (`model.go:61-66`).

---

## 2. Plugin discovery model

**The plugin package never reads `~/.claude/plugins/` or any plugin path directly.** All discovery is delegated:

| Discovery path | How retrieved | Returned shape |
|---|---|---|
| Installed plugins (any scope) | `claude plugins list --json` (`cli.go:83`) | `[]InstalledPlugin` with `InstallPath` populated by CLI (`model.go:11`) |
| Marketplace-available plugins | `claude plugins list --available --json` (`cli.go:97`) | `ListResult{Installed, Available}` |
| Configured marketplaces | `claude plugins marketplace list --json` (`cli.go:111`) | `[]MarketplaceInfo` with `InstallLocation` populated by CLI (`model.go:65`) |

**Concrete InstallPath shape** (from the JSON fixture in `model_test.go:35`):
```
/Users/test/.claude/plugins/cache/claude-plugins-official/lua-lsp/1.0.0
```
i.e. `~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/`.

**Concrete marketplace InstallLocation** (`model_test.go:164`):
```
/Users/test/.claude/plugins/marketplaces/claude-plugins-official
```
i.e. `~/.claude/plugins/marketplaces/<name>/`.

These paths are returned for display only — the Go package never `os.Stat`s, reads, writes, or watches them.

---

## 3. Scope precedence and semantics

### 3.1 Scope is a CLI passthrough string

The Go layer treats scope as an opaque string. Per `model.go:9` the documented values are `"user"`, `"project"`, `"local"`. The CLI invocations (`cli.go:125,134,143,152`) pass it verbatim as `--scope <value>`. No validation, no enum, no Go-side precedence logic.

### 3.2 The de facto scope rules emerge in the GUI/adapter, not the package

| Operation | Scope source | Where enforced |
|---|---|---|
| `Install` | Hardcoded `"project"` | `cmd/lazyclaude/root.go:730` (adapter) |
| `Uninstall` | `p.Scope` from cached InstalledPlugin | `gui/app_actions.go:1009`; but uninstall is gated to scope=="project" only at `app_actions.go:1001-1006` ("Only project-scoped plugins can be uninstalled") |
| `Enable`/`Disable` (Toggle) | `p.Scope` from cached InstalledPlugin | `gui/app_actions.go:1026` (no scope restriction — user/local/project all toggleable) |
| `Update` | No scope arg (per CLI design) | `cli.go:160-166` |

**Behavioral inversion worth noting:** uninstall is *more* restrictive than enable/disable. You can flip an enabled state on a user-scoped plugin from the TUI, but you cannot uninstall it. This is enforced in the GUI layer; the `plugin.Manager.Uninstall` will happily uninstall any scope if called directly.

### 3.3 Project context switching

`Manager.SetProjectDir(dir)` -> `ExecCLI.SetProjectDir(dir)` (`manager.go:34-36`, `cli.go:49-51`) stores `projectDir`. Each subsequent `runner.Run(ctx, c.projectDir, ...)` (`cli.go:24-26`) sets `cmd.Dir`. Project-scope semantics are realized by `claude` itself reading the cwd; the Go layer just sets `Dir`.

This means: **the same `Manager` instance can be retargeted across sessions** by changing the project dir. The GUI does this via `pluginAdapter.SetProjectDir` (root.go:691-693), driven by the Sessions panel cursor (per Pass B daemon/session deepenings).

---

## 4. Toggle/enable/disable persistence

The Go package does not persist enabled state. It tracks an `Enabled bool` field on the cached `InstalledPlugin` (`model.go:10`), but mutations go through:

1. `Manager.ToggleEnabled` reads cached state to decide direction (`manager.go:122-131`).
2. Invokes `c.cli.Enable(...)` or `c.cli.Disable(...)` (`manager.go:137-145`).
3. Calls `Manager.Refresh(ctx)` to repopulate cache from CLI (`manager.go:147`).

The toggle decision is **read-from-cache, not read-from-CLI**. A stale cache could send the wrong subcommand. Mitigation: every successful op ends with `Refresh`. But concurrent operations could race here — see Section 7 (concurrency).

---

## 5. Install / uninstall / update flow

### 5.1 Install (manager.go:105-110, cli.go:124-130)

```
Manager.Install(ctx, pluginID, scope)
  -> ExecCLI.Install(ctx, pluginID, scope)
     -> runner.Run(ctx, projectDir,
          "plugins", "install", pluginID, "--scope", scope)
        -> exec.CommandContext(claudePath, ...) with cmd.Dir = projectDir
  -> on success: Manager.Refresh(ctx)  // single global refresh
```

There is **no progress reporting** during install. The runner buffers stdout/stderr into `bytes.Buffer` (`cli.go:31-32`); the caller blocks for the duration of the subprocess. The GUI wraps this in `runPluginAsync` (`gui/app_actions.go:1060-1072`) which sets `loading=true`, runs in a goroutine with `context.Background()`, and clears loading + displays error on completion.

### 5.2 Force non-interactive env (cli.go:30)

```go
cmd.Env = append(os.Environ(), "TERM=dumb", "NO_COLOR=1")
```

This is a deliberate guard to prevent ANSI escape codes in JSON output that would break `json.Unmarshal`. Important contract for monocle: any subprocess wrapper must do the same, or use an alternative IPC.

### 5.3 Update has no scope (cli.go:160-166)

`Manager.Update` calls `claude plugins update <pluginID>` with no `--scope` flag. Update semantics are delegated to the CLI (which presumably figures out the scope from the install record).

### 5.4 Marketplace mutations are unwired to the GUI

`ExecCLI.MarketplaceAdd/Remove/Update` (`cli.go:169-197`) are implemented and tested (`cli_test.go:201-262`), but the `Manager` does not expose them, the `pluginAdapter` does not include them, and the `gui.PluginProvider` interface does not include them. They are **dead code from a GUI consumer perspective** — reachable only by direct package import. No keybinding exists for adding a marketplace in the registry (verified: `keymap/registry.go:446-569` defines only Install/Uninstall/ToggleEnabled/Update/Refresh actions, plus MCP-tab analogues).

This means a TUI user **cannot** add or remove a marketplace from lazyclaude. They must drop to the `claude` CLI. Important spec input for monocle if marketplace-management UX is in scope.

---

## 6. Schema, JSON parsing, polymorphic `Source`

### 6.1 `InstalledPlugin` schema (model.go:6-14)

| Field | JSON key | Notes |
|---|---|---|
| ID | `id` | Always `<plugin>@<marketplace>`; marketplace suffix may be empty (`MarketplaceName` returns "") |
| Version | `version` | |
| Scope | `scope` | `"user"`, `"project"`, `"local"` (documented values) |
| Enabled | `enabled` | bool |
| InstallPath | `installPath` | filesystem path, display-only |
| InstalledAt | `installedAt` | ISO 8601 string, no parsing |
| LastUpdated | `lastUpdated` | ISO 8601 string, no parsing |

Timestamps are kept as strings — no `time.Time` parse, no relative formatting in this package. The GUI strips both `InstallPath` and `LastUpdated` when projecting to `gui.PluginItem` (`root.go:702-710`), so they are not even displayable today.

### 6.2 Polymorphic `Source` (model.go:26-52)

The `source` JSON value can be **either**:

a. An object: `{"source":"github","repo":"...","ref":"...","sha":"..."}` or `{"source":"url","url":"..."}`
b. A bare string: `"./plugins/agent-sdk-dev"` (local path form)

The custom `UnmarshalJSON` (`model.go:38-52`) tries string-decode first; on success it normalizes to `{Source:"path", Raw:"./..."}`. On failure it falls through to standard object decode via the `type alias Source` trick (avoiding infinite recursion). The `Raw` field has `json:"-"` so it never round-trips out — it is purely an internal carrier for the string form.

Validated by `TestListResult_JSONParse` (`model_test.go:148-154`) with both forms present in a single payload.

### 6.3 Source type vocabulary (model.go:29)

Documented values: `"github"`, `"url"`, `"path"`, `"npm"`, `"git-subdir"`. The package does not branch on these; it only stores them.

### 6.4 `MarketplaceName(id)` helper (model.go:69-76)

Splits on first `@`. Returns `""` for `"no-marketplace"` or empty input. Used by the GUI to display the marketplace badge — but **the implementation does not handle multiple `@` correctly** if a plugin ID ever contains `name@ref@marketplace`. Currently no observed case; nitpick-grade concern.

### 6.5 Manifest schema is not in this package

The `claude` CLI owns plugin manifest format. There is no `plugin.json` or `manifest.toml` parsing here, no validation, no version-compatibility check. If monocle needs to read manifests directly (e.g. for richer preview), it cannot inherit anything from this package.

---

## 7. Concurrency model

### 7.1 Single mutex over caches

`Manager.mu` is a `sync.RWMutex` (`manager.go:14`). It guards `installed`, `available`, `markets` slices. The pattern:

- Writers (`Refresh` end at `manager.go:65-69`) take write lock briefly.
- Readers (`Installed`, `Available`, `Marketplaces`) take read lock and **copy** the slice before returning (`manager.go:79-81, 89-91, 99-101`). This gives callers an immutable snapshot.
- `ToggleEnabled` reads under RLock, **releases the lock**, then performs the CLI op without holding any lock (`manager.go:122-131`).

### 7.2 Operations are NOT serialized

`Install`, `Uninstall`, `ToggleEnabled`, `Update`, and `Refresh` can all execute concurrently. Each one ends with a `Refresh` that takes the write lock briefly to swap the cache. The CLI subprocesses themselves run independently.

**Race window** in `ToggleEnabled` (`manager.go:122-148`):
1. Goroutine A reads `installed[i].Enabled == true` -> decides to Disable.
2. Goroutine B refreshes cache and now `installed[i].Enabled == false`.
3. Goroutine A calls Disable on an already-disabled plugin -> the CLI may error, propagated up.

In practice the GUI funnels all plugin actions through a single keypress dispatcher and uses `runPluginAsync` which sets `loading=true`; the user cannot fire two ops simultaneously. But the package itself is **race-tolerant at the cache level, not at the action level**. `TestManager_ConcurrentRefresh` (`manager_test.go:295-324`) only exercises 10 concurrent Refreshes; it does not exercise concurrent mutation.

### 7.3 `SetProjectDir` is unsynchronized

`ExecCLI.SetProjectDir` writes `c.projectDir` (`cli.go:49-51`) without a lock. Concurrent `Run` calls could read a torn value on architectures that don't guarantee aligned word writes, though Go's string is two words so a torn read is theoretically possible. The GUI calls `SetProjectDir` from the gocui main loop and never concurrently with operations, so this is a latent issue, not an observed bug.

### 7.4 Context handling

Every public op accepts `ctx context.Context` and threads it into `exec.CommandContext` (`cli.go:25`). Cancelling the ctx kills the subprocess. However: the **GUI always uses `context.Background()`** (`gui/app_actions.go:1063`), so cancellation is never wired. This is P1-009 in pass-8.

---

## 8. Error handling

### 8.1 Wrapping convention

Every CLI method wraps with `fmt.Errorf("...: %w", err)` (`cli.go:35, 85, 90, 99, 104, 113, 118, 127, 136, 145, 154, 163, 172, 181, 194`). Stderr is captured and embedded in the error message at the runner layer (`cli.go:34-35`):

```go
return "", fmt.Errorf("claude %v: %w (stderr: %s)", args, err, stderr.String())
```

The argv vector is included verbatim in error messages — useful for debugging but could leak the plugin ID into logs (probably non-sensitive).

### 8.2 Refresh has graceful degradation (manager.go:42-72)

Two-tier fallback:
1. `ListAll` (with marketplace available list) is the happy path.
2. On `ListAll` failure, fall back to `ListInstalled` (no marketplace data) — installed plugins still display.
3. On `ListInstalled` *also* failing, return a wrapped error containing both.
4. `ListMarketplaces` failure is **silently nullified** — `markets = nil` and the function continues.

This makes "show me installed plugins" the most resilient operation, and marketplace data the most disposable. A user offline from GitHub still sees their installed plugins.

### 8.3 No retry logic, no backoff

A transient `claude` CLI failure (e.g. process killed, network blip in `ListAll`) surfaces directly to the user as a one-shot error. No retries. Spec input: monocle should decide whether to add retry for transient errors.

### 8.4 No structured logging

Manager holds a `*slog.Logger` (`manager.go:13`) but **never uses it**. Verified by absence of `m.log` references in `manager.go`. The logger is plumbed through but currently dead code. Either an oversight or scaffolding for future use.

---

## 9. Test coverage assessment

### 9.1 Coverage matrix

| Concern | Test | Type | Confidence |
|---|---|---|---|
| Refresh happy path | `TestManager_Refresh` (manager_test.go:43-89) | unit | HIGH |
| Refresh ListAll failure fallback | `TestManager_Refresh_FallbackToListInstalled` (manager_test.go:235-266) | unit | HIGH |
| Refresh marketplace failure non-fatal | `TestManager_Refresh_MarketplaceFailureNonFatal` (manager_test.go:268-293) | unit | HIGH |
| Concurrent Refresh | `TestManager_ConcurrentRefresh` (manager_test.go:295-324) | race | MEDIUM (only Refresh, not mixed ops) |
| Install + cache refresh | `TestManager_Install` (manager_test.go:91-127) | unit | HIGH |
| Uninstall + cache refresh | `TestManager_Uninstall` (manager_test.go:129-157) | unit | HIGH |
| Toggle: enabled->disable | `TestManager_ToggleEnabled` (manager_test.go:159-210) | unit | HIGH |
| Toggle: disabled->enable | same | unit | HIGH |
| Toggle: not-found error | `TestManager_ToggleEnabled_NotFound` (manager_test.go:212-233) | unit | HIGH |
| CLI argv shape (all ops) | `cli_test.go:22-262` | unit | HIGH |
| Runner error propagation | `TestExecCLI_RunnerError` (cli_test.go:167-175) | unit | HIGH |
| Invalid JSON propagates | `TestExecCLI_InvalidJSON` (cli_test.go:177-185) | unit | HIGH |
| Marketplace update with no name | `TestExecCLI_MarketplaceUpdateAll` (cli_test.go:246-262) | unit | HIGH |
| WithClaudePath option | `TestExecCLI_WithClaudePath` (cli_test.go:264-273) | unit | HIGH |
| Source polymorphic decode | `TestListResult_JSONParse` (model_test.go:79-155) | unit | HIGH |
| MarketplaceName edge cases | `TestMarketplaceName` (model_test.go:8-25) | table | HIGH |
| JSON parse with real CLI output | `TestInstalledPlugin_JSONParse`, `TestMarketplaceInfo_JSONParse` | unit | HIGH |

### 9.2 Gaps

| Gap | Impact |
|---|---|
| No test of Update + Refresh interleaving | LOW — `Update` is a thin wrapper, behaves like Install |
| No test mixing concurrent Install + Toggle + Refresh | MEDIUM — toggle-direction race in ToggleEnabled is unverified |
| No test of `execRunner` itself (real subprocess) | MEDIUM — would need a `claude` binary or shim; all tests use a fake `Runner` |
| No test of `TERM=dumb`/`NO_COLOR=1` env passthrough | LOW — value visible only via real subprocess |
| No test of `cmd.Dir = projectDir` actually being honored | LOW — same reason |
| No test of `MarketplaceAdd/Remove/Update` failure modes (only argv shape) | LOW — symmetric to Install/Uninstall |
| `MarketplaceName` does not test multi-`@` IDs | NITPICK |
| `Manager` log field has no behavioral coverage because it has no behavior | NITPICK |

### 9.3 No integration / E2E tests for plugin

No VHS tape verifies plugin install/uninstall flows. The `vis_e2e_tests/` directory mentioned in `CLAUDE.md` is the only E2E vehicle and does not exercise plugin ops (no smoke for "i" / "d" / "e" keys with plugin provider). The package is **mock-only tested.**

---

## 10. Cross-cutting concerns

### 10.1 Logging

Manager holds `*slog.Logger`, never logs. CLI does not log. Errors return up the stack and are displayed via `app.showError` (GUI layer). **Spec implication:** if monocle wants structured logging of plugin operations, it must be added — not inherited.

### 10.2 Security

- Subprocess argv is constructed from user-selected plugin IDs (`cli.go:125`, etc.). No shell interpolation — `exec.Command` with arg vector is safe against shell injection.
- `claudePath` defaults to `"claude"` and falls back to PATH resolution (`cli.go:73`). `findClaudeBinary()` (referenced at `root.go:342`) resolves an absolute path when possible. This avoids PATH-hijacking risk.
- No sandboxing of the subprocess; it runs with the user's full env (`os.Environ()` at `cli.go:30`).

### 10.3 Configuration

No environment variables, no config file. The only knobs are constructor options:
- `WithRunner(r)` — inject for tests (`cli.go:57-61`).
- `WithClaudePath(p)` — point at non-default `claude` binary (`cli.go:64-68`).

`findClaudeBinary()` lives in `cmd/lazyclaude/root.go` (referenced from `root.go:342, 779`). Not part of the plugin package's API surface.

### 10.4 Observability

No metrics, no tracing, no health probes. The package is invisible to any observability layer.

---

## 11. Behavioral contracts (NEW — BC-PLUGIN-*)

These are the first BC-PLUGIN-* contracts authored. Confidence HIGH where backed by a test; MEDIUM where inferred from code paths uniformly.

### BC-PLUGIN-001: Manager caches installed/available/markets between refreshes
- **Preconditions:** Manager constructed with a valid `*ExecCLI`.
- **Postconditions:** `Installed()`, `Available()`, `Marketplaces()` return the slices populated by the most recent `Refresh`, copied to prevent caller mutation of internal state.
- **Evidence:** `manager.go:75-101`, `TestManager_Refresh` (manager_test.go:43-89)
- **Confidence:** HIGH

### BC-PLUGIN-002: Refresh falls back from ListAll to ListInstalled on failure
- **Preconditions:** `claude plugins list --available --json` fails (e.g. no network).
- **Postconditions:** Manager populates `installed` from `claude plugins list --json`; `available` is empty; no error returned to caller.
- **Evidence:** `manager.go:46-57`, `TestManager_Refresh_FallbackToListInstalled` (manager_test.go:235-266)
- **Confidence:** HIGH

### BC-PLUGIN-003: Refresh treats marketplace listing failure as non-fatal
- **Preconditions:** `claude plugins marketplace list --json` fails.
- **Postconditions:** `markets` is set to nil; `installed`/`available` reflect the successful list; no error returned.
- **Evidence:** `manager.go:59-63`, `TestManager_Refresh_MarketplaceFailureNonFatal` (manager_test.go:268-293)
- **Confidence:** HIGH

### BC-PLUGIN-004: All mutating operations refresh the cache on success
- **Preconditions:** Mutating op (`Install`, `Uninstall`, `ToggleEnabled`, `Update`) succeeds at the CLI layer.
- **Postconditions:** A subsequent `Refresh` is invoked; cache reflects post-mutation state. On CLI failure, no Refresh is called.
- **Evidence:** `manager.go:106-109, 113-117, 138-147, 151-155`; `TestManager_Install`, `TestManager_Uninstall`, `TestManager_ToggleEnabled`
- **Confidence:** HIGH

### BC-PLUGIN-005: ToggleEnabled decides direction from cached Enabled flag
- **Preconditions:** Plugin ID exists in cache.
- **Postconditions:** If `Enabled` is true in cache, `Disable` is invoked; else `Enable`. Stale cache may produce wrong direction.
- **Evidence:** `manager.go:122-148`, `TestManager_ToggleEnabled` (manager_test.go:159-210)
- **Confidence:** HIGH

### BC-PLUGIN-006: ToggleEnabled returns error for unknown plugin ID
- **Preconditions:** Plugin ID not present in cache.
- **Postconditions:** Returns `fmt.Errorf("plugin not found: %s", pluginID)`. No CLI call made.
- **Evidence:** `manager.go:133-135`, `TestManager_ToggleEnabled_NotFound` (manager_test.go:212-233)
- **Confidence:** HIGH

### BC-PLUGIN-007: ExecCLI forces non-interactive subprocess environment
- **Preconditions:** Any CLI command is invoked through the default `execRunner`.
- **Postconditions:** Subprocess env includes `TERM=dumb` and `NO_COLOR=1` appended to `os.Environ()` to suppress ANSI escape sequences.
- **Evidence:** `cli.go:29-30`
- **Confidence:** MEDIUM (no test exercises real subprocess, but contract is explicit in code)

### BC-PLUGIN-008: ExecCLI runs subprocesses with `cmd.Dir = projectDir`
- **Preconditions:** `SetProjectDir` has been called with a non-empty path.
- **Postconditions:** Subsequent `claude plugins ...` invocations execute with that working directory, allowing project-scoped settings to apply. When `projectDir` is empty, current process cwd is used.
- **Evidence:** `cli.go:24-27, 49-51`
- **Confidence:** MEDIUM (no integration test verifies)

### BC-PLUGIN-009: Source field accepts both object and string JSON forms
- **Preconditions:** Unmarshalling marketplace plugin payload.
- **Postconditions:** Object form decodes to `Source{Source, Repo, URL, Ref, SHA}`. Bare string form decodes to `Source{Source:"path", Raw:<string>}`.
- **Evidence:** `model.go:38-52`, `TestListResult_JSONParse` (model_test.go:79-155)
- **Confidence:** HIGH

### BC-PLUGIN-010: MarketplaceUpdate omits empty name argument
- **Preconditions:** Caller passes empty string for `name`.
- **Postconditions:** Argv is `["plugins", "marketplace", "update"]` (length 3); no empty argument appended.
- **Evidence:** `cli.go:188-191`, `TestExecCLI_MarketplaceUpdateAll` (cli_test.go:246-262)
- **Confidence:** HIGH

### BC-PLUGIN-011: Installed/Available/Marketplaces return defensive copies
- **Preconditions:** Caller invokes `Installed()`, `Available()`, or `Marketplaces()`.
- **Postconditions:** Returned slice is a fresh allocation; mutation by caller does not affect cached state. Verified indirectly by the copy idiom `result := make(... ); copy(result, ...)`.
- **Evidence:** `manager.go:79-81, 89-91, 99-101`
- **Confidence:** HIGH

### BC-PLUGIN-012: Marketplace mutation APIs exist on ExecCLI but are unwired
- **Preconditions:** A consumer wants to add or remove a marketplace from the TUI.
- **Postconditions:** No path exists — `Manager` exposes no `MarketplaceAdd/Remove/Update`. The GUI provider interface omits them. Only direct `*ExecCLI` callers can reach them.
- **Evidence:** `cli.go:169-197` (impl), `manager.go` (absence), `gui/plugin_state.go:28-39` (absence in interface)
- **Confidence:** HIGH (negative-space contract)

### BC-PLUGIN-013: Manager logger field is unused
- **Preconditions:** Caller passes a `*slog.Logger` to `NewManager`.
- **Postconditions:** The logger is stored but never invoked. All errors flow upward via return values.
- **Evidence:** `manager.go:13, 22-30` (assignment but no use within `manager.go`)
- **Confidence:** HIGH

### BC-PLUGIN-014: Operations are cache-level thread-safe but not action-level serialized
- **Preconditions:** Multiple goroutines call public ops concurrently.
- **Postconditions:** Cache reads and writes are protected by `sync.RWMutex`; no data race on the cached slices. `ToggleEnabled` releases the lock before invoking the CLI, so two concurrent toggles on the same plugin may both call Enable or both call Disable.
- **Evidence:** `manager.go:14, 65-69, 77-78, 87-88, 97-98, 122-148`; `TestManager_ConcurrentRefresh` (manager_test.go:295-324) for the safe direction.
- **Confidence:** MEDIUM (concurrent mixed-op behavior is uncovered by tests)

---

## 12. Monocle relevance

### 12.1 Direct relevance

The package is a **thin adapter to an external CLI that monocle's static plane will likely need** if "MCP Server & Plugin Management" remains a feature. The translation choices:

| Decision | Options |
|---|---|
| Plugin discovery | (a) PORT-DIRECT this shell-out approach, (b) read `~/.claude/plugins/` directly, (c) skip plugins entirely in v1 |
| Marketplace integration | Currently CLI-only; monocle could either keep this constraint or read the marketplace cache dir directly |
| Scope handling | The user/project/local trichotomy is a Claude-Code concept; monocle should inherit, not redesign |
| Toggle UX | The cache-driven direction decision is a known race window; monocle could re-read fresh state before toggling |

### 12.2 What monocle inherits directly

- Concrete on-disk shape of `~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/`
- Concrete on-disk shape of `~/.claude/plugins/marketplaces/<name>/`
- The argv vectors for every plugin operation (`cli.go:82-197`)
- Polymorphic `Source` JSON handling (model.go:38-52) — this **will** recur if monocle reads the same payloads
- Non-interactive subprocess env contract (`TERM=dumb`, `NO_COLOR=1`)
- Project-scope means "subprocess cwd is the project dir"

### 12.3 What monocle should reconsider

- **Cancellation:** wire `ctx` through from UI events. lazyclaude leaves this on the floor (P1-009).
- **Action-level serialization:** consider a per-plugin lock or a single-threaded action queue.
- **Marketplace UX:** decide if add/remove marketplaces is in scope; if yes, expose the existing `ExecCLI` methods via the provider interface.
- **Manifest reading:** lazyclaude does not read manifests directly. If monocle needs a richer plugin browser (e.g. README, commands, capabilities), it must add manifest-reading logic that does not exist here.
- **Logging:** the `*slog.Logger` plumb-through is aspirational. Decide whether to wire it or remove it.
- **Update scope semantics:** `claude plugins update` takes no `--scope`. Verify whether updates honor cached scope or require explicit scope per plugin.

### 12.4 P-level recommendations for monocle

| ID | Priority | Item |
|---|---|---|
| P0-PLUGIN-A | P0 | Decide whether plugin management is in v1 scope (Pass 8 already flagged as P2 decision) |
| P1-PLUGIN-B | P1 | If kept: plumb `context.Context` through from UI cancel events (mirrors Pass-8 P1-009) |
| P1-PLUGIN-C | P1 | Add an action-serialization or per-plugin lock to prevent concurrent Enable/Disable on the same ID |
| P1-PLUGIN-D | P1 | Decide whether to expose marketplace add/remove/update in TUI or remain CLI-only (inherits gap) |
| P2-PLUGIN-E | P2 | Replace cache-driven toggle direction with a fresh read before issuing the command |
| P2-PLUGIN-F | P2 | Wire the unused `*slog.Logger` or remove it |
| P2-PLUGIN-G | P2 | Decide if manifest-level reads (for richer preview) are needed; if so, design a separate reader since this package will not give it to you |

---

## 13. Delta Summary

- **New BC-PLUGIN-* contracts authored:** 14 (BC-PLUGIN-001 through 014). Pass 3 had zero plugin contracts.
- **New negative-space findings:** marketplace mutation surface is unwired from GUI (BC-PLUGIN-012); manager logger is unused (BC-PLUGIN-013).
- **New concurrency findings:** action-level race window in ToggleEnabled (BC-PLUGIN-014); `SetProjectDir` is unsynchronized (Section 7.3).
- **New schema findings:** polymorphic `Source` form with custom UnmarshalJSON (BC-PLUGIN-009); manifest schema is not in this package (Section 6.5).
- **New scope-precedence findings:** Install hardcoded to "project" by adapter (root.go:730); Uninstall additionally gated to scope=="project" at the GUI layer (`app_actions.go:1001-1006`), but Toggle is not similarly gated.
- **New tooling findings:** No E2E coverage for plugin flows; all tests are mock-only.
- **Remaining gaps:**
  - Real-subprocess behavior of `execRunner` is uncovered by tests.
  - Concurrent mixed-op race assertions are absent.
  - Behavior of CLI for unknown subcommands / new plugin types (e.g. `npm`, `git-subdir`) is unverified.
  - Behavior when `claude` binary is missing entirely is unverified at this layer (would surface as `exec.Run` error).

## 14. Novelty Assessment

**Novelty: SUBSTANTIVE**

Justification: Pass 3 contained zero BC-PLUGIN-* contracts. This round authors 14, including non-trivial findings that change how the system would be specced:
- The cache-driven toggle direction (BC-PLUGIN-005) is a subtle correctness contract worth lifting into spec.
- The asymmetric scope gate (uninstall=project-only, toggle=any-scope) is a behavioral inversion worth surfacing.
- The unwired marketplace mutation surface (BC-PLUGIN-012) is a real product-scope question for monocle.
- The unused logger (BC-PLUGIN-013) is a code-health finding that would otherwise carry into monocle as cargo.
- The polymorphic `Source` decode (BC-PLUGIN-009) is concrete schema spec input for monocle.

Removing this round's findings would absolutely change how monocle specs its plugin plane.

## 15. Convergence Declaration

**Another round needed.** Specifically, round 2 should:

1. Verify behavior under `claude` binary absence (does `exec.LookPath` happen? what error surfaces?).
2. Cross-reference with `cmd/lazyclaude/root.go:findClaudeBinary` to document the resolution algorithm in spec terms.
3. Examine whether any callers reach `MarketplaceAdd/Remove/Update` in tests-only or dead-code-only paths.
4. Dig into how `installPath` is consumed downstream (it is stripped by the adapter — confirm no other consumer reads it).
5. Inspect whether `time.Parse` of `installedAt`/`lastUpdated` happens anywhere (current finding says no; verify).
6. Validate the multi-`@` MarketplaceName edge case against real CLI output if any documentation exists.

If round 2 turns up only confirmation and refinement, NITPICK convergence will be declared.

## State Checkpoint

```yaml
pass: B-plugin
round: 1
status: complete
files_scanned: 6
timestamp: 2026-05-11T18:05:00Z
novelty: SUBSTANTIVE
new_contracts: 14
new_gaps: 7
next_round_needed: true
```
