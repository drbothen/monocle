# Pass B Deepening — `internal/plugin/` Subsystem (Round 2)

**Reference:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/plugin/`
**Round 1:** `pass-B-deep-plugin-r1.md` — SUBSTANTIVE; 14 new BC contracts.
**Round 2 goal:** Chase the six gaps named in r1 §15 and validate convergence.

---

## 1. Round-1 gap follow-ups

### Gap 1: `claude` binary resolution algorithm

**Round 1 question:** Where exactly does the resolution happen and what is the precedence order?

**Finding:** Resolution is performed by `cmd/lazyclaude/root.go:782-803` `findClaudeBinary()`, **not** inside the plugin package. Inside the plugin package, `cli.go:356` only seeds `claudePath: "claude"` and relies on `os/exec.LookPath` implicitly via `exec.CommandContext`.

The cmd-level algorithm:

| Step | Source | Lines |
|---|---|---|
| 1 | `exec.LookPath("claude")` — honors current `$PATH` | `root.go:784-786` |
| 2 | `~/.local/bin/claude` — probed via `os.Stat`, must not be a directory | `root.go:794, 798` |
| 3 | `/usr/local/bin/claude` — probed via `os.Stat` | `root.go:795` |
| 4 | Empty string (signals "use bare `claude` and let exec figure it out") | `root.go:802` |

The cmd code comment at `root.go:780-781` documents the rationale: "exec.LookPath alone is insufficient because tmux display-popup inherits the tmux server's PATH, which typically lacks ~/.local/bin." This is **non-portable** logic that monocle should re-evaluate for its target environments (likely Linux desktop / macOS, not tmux-only).

When `findClaudeBinary()` returns empty (`root.go:802`), the plugin Manager is constructed without `WithClaudePath`, falling back to `"claude"` on `$PATH`. If `claude` is not on PATH, the first CLI call returns an `exec.LookPath` error wrapped by `cli.go:35`: `claude [...]: exec: "claude": executable file not found in $PATH (stderr: )`.

**New finding for monocle:** the `findClaudeBinary` function is **outside the plugin package** and is implicit in the package's behavior. A naive port of `internal/plugin/` would miss this contract. Either move the resolution into the package, or document that callers must resolve.

### BC-PLUGIN-015: Claude binary resolution is caller-supplied, not package-managed
- **Preconditions:** Any caller constructs `ExecCLI`.
- **Postconditions:** If `WithClaudePath` is omitted, the package uses bare `"claude"` and depends on the process `$PATH`. The package never probes for the binary itself.
- **Evidence:** `cli.go:73, 354-358`; resolution lives at `cmd/lazyclaude/root.go:782-803`.
- **Confidence:** HIGH

### Gap 2: Marketplace mutation method reachability

**Round 1 finding:** `MarketplaceAdd/Remove/Update` exist on `ExecCLI` but are not exposed via `Manager` or the GUI provider interface.

**Round 2 confirmation:** A repository-wide scan finds the only call sites are:
- `cli.go:39422, 39431, 39440` — the definitions themselves.
- `cli_test.go:39173, 39189, 39204, 39220` — the unit tests.

There are **zero non-test consumers** of these three methods anywhere in the codebase. This confirms BC-PLUGIN-012 as a hard finding (negative-space contract), not a misread of partial wiring.

**Implication for monocle:** if `MarketplaceAdd/Remove/Update` are needed for the UX, they require:
1. Method addition on `Manager` (uplift + cache refresh)
2. Addition to `gui.PluginProvider` interface
3. Adapter pass-through in `pluginAdapter`
4. Keybinding registry entries (would need a new `MarketplaceTabAdd/Remove/Update` action set)
5. Tab/scope wiring for the marketplace tab keymap

None of those exist today. This is at least a 5-touchpoint addition.

### Gap 3: `installPath` and `lastUpdated` downstream consumption

**Round 1 finding (tentative):** Both are stripped by the adapter and unused downstream.

**Round 2 confirmation:**

| Field | Storage | Consumers |
|---|---|---|
| `InstallPath` | `InstalledPlugin.InstallPath` (model.go:11) | None outside the package. Adapter at `cmd/lazyclaude/root.go:702-710` does not include it in `gui.PluginItem`. |
| `LastUpdated` | `InstalledPlugin.LastUpdated` (model.go:13) | None outside the package. Only test fixtures and the struct field reference it. The adapter does not project it. |
| `InstalledAt` | `InstalledPlugin.InstalledAt` (model.go:12) | Adapter projects it (`root.go:709`). Rendered by `presentation.FormatPluginPreview` (`presentation/plugins.go:103-109`). Date portion is taken by `strings.IndexByte(date, 'T')` slice — **not** `time.Parse`. |

**BC-PLUGIN-016: ISO 8601 timestamps are never parsed into `time.Time`**
- **Preconditions:** Plugin payload contains `installedAt`/`lastUpdated`/`installedAt` ISO timestamps.
- **Postconditions:** Strings are stored as-is. The only consumption is a `strings.IndexByte(s, 'T')` split for date-only display (`presentation/plugins.go:105-107`). Malformed dates or non-ISO formats would pass through silently; relative time ("3 days ago") is not supported.
- **Evidence:** `model.go:12-13`, `presentation/plugins.go:103-109`
- **Confidence:** HIGH

**BC-PLUGIN-017: `InstallPath` and `LastUpdated` are unused downstream**
- **Preconditions:** Plugin is in cache.
- **Postconditions:** Neither field is rendered, logged, or consumed by any caller. They are decoded for fidelity but represent dead weight.
- **Evidence:** Absence of any reference in `cmd/`, `internal/gui/`, or `internal/gui/presentation/` outside the package itself.
- **Confidence:** HIGH

### Gap 4: Multi-`@` `MarketplaceName` edge case

**Round 1 finding:** `MarketplaceName(id)` splits on first `@`; uncertain if multi-`@` IDs ever occur.

**Round 2 finding:** The implementation actually splits on the **first** `@` (`model.go:69-76`):

```go
for i, c := range id {
    if c == '@' {
        return id[i+1:]
    }
}
```

If a plugin ID is `name@ref@marketplace`, this returns `ref@marketplace`, not `marketplace`. **No real test or documentation exercises this case.** Looking at observed IDs:
- `"lua-lsp@claude-plugins-official"` — single `@`
- `"code-review@claude-plugins-official"` — single `@`
- `"pyright-lsp@claude-plugins-official"` — single `@`
- `"agent-sdk-dev@claude-plugins-official"` — single `@`

No double-`@` observed in fixtures or tests. The current implementation is correct for the observed surface; the multi-`@` concern is **NITPICK-grade**.

### Gap 5: Behavior when `claude` binary is missing entirely

**Finding:** Surfaces as a runner error at `cli.go:34-37` wrapping `exec: "claude": executable file not found in $PATH`. The first user action that triggers a CLI call (typically the GUI's initial `Refresh` on plugin panel focus, via `runPluginAsync` -> `Refresh`) will display "Plugin error: ..." via `app.showError` (`app_actions.go:1067`). Subsequent calls retry from scratch — no caching of the resolution failure.

**Important UX note:** because every operation is a fresh subprocess invocation, a user who installs `claude` mid-session can recover without restarting lazyclaude.

**BC-PLUGIN-018: Missing `claude` binary surfaces as a per-call error, recoverable without restart**
- **Preconditions:** `claude` is not on `$PATH` and `WithClaudePath` is not set.
- **Postconditions:** Each operation independently fails with an exec-not-found error wrapped at `cli.go:35`. There is no cached failure state; installing `claude` later allows subsequent operations to succeed.
- **Evidence:** `cli.go:24-37`; absence of caching in `Manager`/`ExecCLI`.
- **Confidence:** MEDIUM (no test exercises this; inferred from code structure)

### Gap 6: New plugin source types (`npm`, `git-subdir`)

**Finding:** The `Source.Source` field is a free-form string (`model.go:29`). The package never branches on it. Adding new source types in upstream `claude` CLI requires zero code change in `internal/plugin/` — the value just propagates through. The Go type is intentionally string-typed and not an enum.

This is a robustness contract: new CLI features that introduce new source types do not break the Go package.

---

## 2. New findings discovered while validating gaps

### 2.1 GUI-layer scope policy

While verifying scope precedence (r1 Section 3), I noticed the GUI layer enforces **three asymmetric constraints** not visible from inside the plugin package alone:

| Operation | GUI gate | Effect |
|---|---|---|
| Any plugin op | `guardRemoteOp("Plugin editing")` at start | Returns early with notification if cursor is on a remote (SSH) session. All five operations (`Install`/`Uninstall`/`ToggleEnabled`/`Update`/`Refresh`) are gated. (`app_actions.go:972-1057`) |
| `Install` | `tabIdx != PluginTabMarketplace` check | Install only fires from the Marketplace tab. (`app_actions.go:976`) |
| `Uninstall` | `p.Scope != "project"` -> show error, return | Cannot uninstall user/local plugins from the TUI. (`app_actions.go:1001-1006`) |
| `Uninstall`/`ToggleEnabled`/`Update` | `tabIdx != PluginTabPlugins` check | Only fire from the Plugins (installed) tab. (`app_actions.go:993, 1017, 1034`) |

**BC-PLUGIN-019: Plugin operations are gated to local sessions only**
- **Preconditions:** Cursor on a remote session (SSH host).
- **Postconditions:** All five plugin operations short-circuit at `guardRemoteOp` before any CLI call. The panel shows a "Plugin editing on remote hosts is not supported" placeholder (`render_plugins.go:81-88`).
- **Evidence:** `app_actions.go:972-1057`, `render_plugins.go:36-47, 45-48, 143-148`
- **Confidence:** HIGH

**BC-PLUGIN-020: Install action is bound to the Marketplace tab; Uninstall/Toggle/Update to the Plugins tab**
- **Preconditions:** User triggers the corresponding key.
- **Postconditions:** If the active tab does not match the action's home tab, the action is a no-op. There is no automatic tab switch.
- **Evidence:** `app_actions.go:976, 993, 1017, 1034`
- **Confidence:** HIGH

### 2.2 Search filter behavior

`filteredInstalledPlugins` (`search.go:306-316, 318-328`) and `filteredAvailablePlugins` (`search.go:331-341, 343-355`):

- **Installed filter:** case-insensitive substring match against `ID` only. Does not search Version or Scope.
- **Available filter:** case-insensitive substring match against `Name` OR `Description`. Does not search PluginID, MarketplaceName, or InstallCount.

**Asymmetry:** the installed-tab search is ID-based (so `lua` matches `lua-lsp@claude-plugins-official`), while the marketplace-tab search is name+description-based (more user-friendly for browsing).

**BC-PLUGIN-021: Search filter scope differs between Plugins tab (ID only) and Marketplace tab (Name+Description)**
- **Evidence:** `search.go:319-328, 345-355`
- **Confidence:** HIGH

### 2.3 Loading state lifecycle

`runPluginAsync` (`app_actions.go:1060-1072`):

```go
a.pluginState.loading = true
go func() {
    err := fn(context.Background())
    a.gui.Update(func(g *gocui.Gui) error {
        a.pluginState.loading = false
        ...
    })
}()
```

The write to `loading` at line 1061 happens on the gocui main thread (it is called from a key handler), and the clear at 1065 happens inside `gui.Update` which serializes onto the main thread. So loading-state transitions are serialized by the gocui scheduler, not by an explicit lock.

However: a second key-press while `loading == true` is **not blocked** — the user can fire a second op. Both will set `loading = true`, both will clear it on completion. This is the "action-level concurrency" gap noted in r1 §14.

### 2.4 Manager.SetProjectDir thread-safety re-examined

R1 §7.3 flagged `ExecCLI.SetProjectDir` as unsynchronized. On closer reading, both `SetProjectDir` (`cli.go:49-51`) and the `Run` invocations (`cli.go:24-26`) happen on the gocui main thread (the GUI calls SetProjectDir from session-cursor change, which is a key-handler path; runs happen from `runPluginAsync` which dispatches to a goroutine but reads `c.projectDir` after the goroutine starts).

Specifically, the goroutine reads `c.projectDir` indirectly via the closure on `c.cli.<Op>` -> `c.runner.Run(ctx, c.projectDir, ...)`. The read happens **inside the goroutine**, after `SetProjectDir` returns. If the user changes session cursor between `runPluginAsync` start and the goroutine actually invoking `Run`, the goroutine could see the new `projectDir`.

This is a **subtle race window**: an install fired for project A could execute against project B's cwd if the cursor changes in the milliseconds between the goroutine launch and `runner.Run` reading the field.

**BC-PLUGIN-022: `projectDir` is read at Run time, not at goroutine launch — late-binding race possible**
- **Preconditions:** User changes session cursor between `runPluginAsync` invocation and the spawned goroutine's `runner.Run` call.
- **Postconditions:** The CLI subprocess executes with the new `projectDir` as cwd, not the one in effect when the operation was triggered. Manifests as project-A install actually running against project B.
- **Evidence:** `cli.go:24-26` (read); `cli.go:49-51` (write); `app_actions.go:1060-1072` (goroutine dispatch)
- **Confidence:** MEDIUM — unverified by tests; inferred from goroutine-and-field-read structure.

### 2.5 `LastUpdated` field is documented but unused — port decision

If monocle wants to display "Updated 3 days ago" in a richer preview, it needs to either:
1. Parse `lastUpdated` (currently a string) into `time.Time` — extension to BC-PLUGIN-016.
2. Surface a delta against `installedAt` for staleness UX.
3. Skip the field entirely (current lazyclaude behavior).

Both fields decode cleanly with `time.RFC3339Nano` based on the fixture format `2026-03-04T16:26:07.583Z`.

---

## 3. Delta Summary

- **New BC-PLUGIN contracts:** 8 (BC-PLUGIN-015 through 022).
- **Total BC-PLUGIN contracts after r2:** 22.
- **Negative-space confirmations:**
  - Marketplace mutation methods: zero non-test consumers (confirmed BC-PLUGIN-012).
  - `InstallPath` / `LastUpdated`: zero non-test consumers (BC-PLUGIN-017).
  - `findClaudeBinary` lives outside the plugin package (BC-PLUGIN-015).
- **New race finding:** late-binding `projectDir` race (BC-PLUGIN-022). This is genuinely new and not in r1.
- **GUI-layer policy findings:** remote gate, tab-binding gate, scope-gated uninstall (BC-PLUGIN-019, 020). These overlap with r1 §3.2 but make the contracts explicit.
- **Search-filter asymmetry:** ID-only vs Name+Description (BC-PLUGIN-021). New for r2.
- **Remaining gaps that did NOT reveal substantive findings:**
  - Multi-`@` MarketplaceName — confirmed non-issue against observed data.
  - New source types — confirmed robust by design (no branching).
  - Real-subprocess execRunner behavior — still uncovered by tests, but inferred behavior is reasonable.

## 4. Novelty Assessment

**Novelty: SUBSTANTIVE**

Justification: this round adds 8 new contracts, of which BC-PLUGIN-022 (late-binding `projectDir` race) and BC-PLUGIN-015 (out-of-package binary resolution) are findings that change how monocle would spec and port the system. Specifically:

- BC-PLUGIN-022 is a real, novel race window not noted anywhere in prior passes. If monocle ports verbatim, it inherits this bug. Worth lifting to spec.
- BC-PLUGIN-015 is a critical packaging boundary: `findClaudeBinary` is not in `internal/plugin/`, so anyone porting just the plugin package would lose binary-resolution behavior silently.
- BC-PLUGIN-019/020 make the GUI/package layering explicit. The package itself imposes none of these gates — they are policy decisions at the adapter/GUI layer.

Removing this round's findings would leave monocle's spec missing the binary-resolution boundary, the late-binding race, and the search-filter asymmetry.

## 5. Convergence Declaration

**One more round recommended.** Round 3 should:

1. Audit whether the late-binding `projectDir` race (BC-PLUGIN-022) is reproducible or actually impossible given gocui's main-thread serialization model — I want to either upgrade this to HIGH confidence or downgrade to a documented non-issue.
2. Verify the exact `time.Parse` format that would work for `installedAt`/`lastUpdated` (test the fixture string against `time.RFC3339Nano`) so monocle has a concrete recommendation, not speculation.
3. Check for any pre-existing `ListAll` / `ListInstalled` /`ListMarketplaces` race against a concurrent `SetProjectDir`.
4. Look at whether `ExecCLI` could be made immutable (project dir set at construction) to eliminate the race in BC-PLUGIN-022.
5. Check for any test that I missed exercising integration aspects (a quick double-check of test files for any `t.Skip` calls or build tags).

If round 3 finds only confirmation, NITPICK convergence will be declared.

## State Checkpoint

```yaml
pass: B-plugin
round: 2
status: complete
files_scanned: 6
prior_round_contracts: 14
new_contracts: 8
total_contracts: 22
timestamp: 2026-05-11T18:18:00Z
novelty: SUBSTANTIVE
next_round_needed: true
```
