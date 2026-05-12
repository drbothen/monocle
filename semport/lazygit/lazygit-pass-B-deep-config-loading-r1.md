# Pass B (deep) — Config Loading (round 1)

## The three-tier loading model

1. **Compile-time defaults** — `GetDefaultConfigForPlatform(runtime.GOOS)` (`pkg/config/config_default_platform.go` / `config_linux.go` / `config_windows.go`) returns a fully-populated `UserConfig` struct. Each field has a sane default; nothing is `nil`.
2. **Global user file(s)** — `[]*ConfigFile` from `~/.config/lazygit/config.yml` (or as specified by `LG_CONFIG_FILE`, comma-separated multi-file).
3. **Per-repo files** — discovered by walking from the repo root upward looking for `.lazygit.yml`, plus the always-present `<repo>/.git/lazygit.yml`. See `pkg/gui/gui.go:436-457`.

The loader iterates over the file list in order, each one yaml-unmarshalled on top of the running base config. This is **layered config with last-write-wins** for scalars and **append** for `CustomCommands` (BC-DRAFT-013).

## ConfigFile policies
`pkg/config/app_config.go:56-69`:
```go
ConfigFilePolicyCreateIfMissing  // default global config file
ConfigFilePolicyErrorIfMissing   // user-specified files via LG_CONFIG_FILE
ConfigFilePolicySkipIfMissing    // per-repo .lazygit.yml chain
```

This is a nice declarative pattern: each file in the chain declares how to handle its absence. Monocle's loader should adopt the same three-valued policy enum.

## Migration is YAML-rewriting, not in-memory transformation
`pkg/config/app_config.go:219-330` — `migrateUserConfig` parses YAML to a `yaml.Node` tree, applies a sequence of transformations, then re-marshals and **writes back to disk** if anything changed. This means:
- Migrations are visible to the user (they see the updated file).
- They're idempotent (running twice produces the same output).
- They use `yaml_utils` (`pkg/utils/yaml_utils/`) for path-based key rename, null→`<disabled>` rewriting, scalar-to-sequence promotion.

Specific migrations currently active (line 269-277 + bespoke transformers):
- `gui.skipUnstageLineWarning` → `skipDiscardChangeWarning`
- `keybinding.universal.executeCustomCommand` → `executeShellCommand`
- `gui.windowSize` → `screenMode`
- `keybinding.files.openMergeTool` → `openMergeOptions`
- `null` keybindings → `"<disabled>"`
- `commitPrefix` scalar → array (line 294)
- `customCommands.subprocess/stream/showOutput` → unified `output:` enum (line 304)
- `git.allBranchesLogCmd` → `git.allBranchesLogCmds` array (line 309)
- `git.paging` object → `git.pagers` array (line 314)

The pattern is admirable for backwards compat: every breaking schema change is paired with an automated migration.

## Discovery — XDG + legacy paths + env override
`pkg/config/app_config.go:589-607` — `findConfigFile`:
1. If `CONFIG_DIR` env var set → use that.
2. Try `XDG_CONFIG_HOME/jesseduffield/lazygit/config.yml` (legacy).
3. Try `XDG_CONFIG_HOME/lazygit/config.yml` (current).
4. Fallback: `xdg.ConfigHome + "/lazygit/config.yml"`.

State file resolution (`stateFilePath`, line 613-621):
1. Look for legacy path via `findConfigFile`.
2. Fall back to `XDG_STATE_HOME/lazygit/<filename>`.

This XDG-first-with-legacy-fallback is the right pattern. Monocle should use the `directories` or `etcetera` crate to mirror this on the Rust side, with the same env override precedence.

## Permission-graceful semantics
- `app_config.go:166-170` (create-if-missing) — if `os.IsPermission(err)`, silently continue. Don't fail because the user mounted their config read-only.
- `app_config.go:636-641` (state file write) — same treatment.
- `app_config.go:82-83` (config dir creation) — `os.IsPermission(err)` returns the partial state instead of erroring.

The principle: **fail open on permission errors when the operation is non-essential**. State persistence and config writes are non-essential; the app keeps running with in-memory state.

## Hot reload trigger
`pkg/gui/gui.go:351-375` — `SetFocusHandler`:
- When gocui regains focus from the terminal multiplexer or window manager:
  - Drop the git config cache.
  - Stat-compare all config files; reload if any changed.
  - Apply changes: language, theme, view properties, keybindings.
  - Call `checkForChangedConfigsThatDontAutoReload(oldConfig, newConfig)` to warn about unappliable changes.
  - Trigger a model refresh.

The focus-change strategy is brilliant because the user usually edits the config in another window, then alt-tabs back. The reload happens at the exact moment they re-enter lazygit. No filesystem watcher needed.

## State file (separate from config)
`pkg/config/app_config.go:696-729` — `AppState` is a small struct of:
- `LastUpdateCheck int64`
- `RecentRepos []string`
- `StartupPopupVersion int`
- `DidShowHunkStagingHint bool`
- `LastVersion string`
- `ShellCommandsHistory []string` (yaml: `customcommandshistory` — legacy name)
- `HideCommandLog bool`
- `GithubPullRequests map[string][]CachedPullRequest`

It's serialized via plain yaml.Marshal/Unmarshal (no migrations) into `XDG_STATE_HOME/lazygit/state.yml`. Saved on every state mutation (`SaveAppStateAndLogError`). This is the right separation:
- **Config = user intent**, edited by hand, versioned in dotfiles.
- **State = app memory**, machine-managed, not versioned.

Monocle's `monocle.toml` (intent) vs `monocle-state.json` (memory) split should follow.

## Validation
`pkg/config/user_config_validation.go` (186 LOC) — `UserConfig.Validate()` runs after each load. Validates things like:
- Numeric ranges (`SidePanelWidth: 0..1`).
- Enum values (e.g. `MainPanelSplitMode ∈ {"horizontal", "vertical", "flexible"}`).
- Cross-field constraints.

JSON schema is generated at `schema/config.json` for editor autocomplete (driven by `karimkhaleel/jsonschema` struct tags on `UserConfig`).

## Translation to monocle
- `serde` with `#[serde(default)]` on every field gets the same "always-populated" default behaviour.
- `figment` or `config-rs` crate for layered loading.
- Custom commands accumulate: use `#[serde(default)] custom_commands: Vec<CustomCommand>` and merge by concatenation.
- Migration: a `migrate_user_config(path: &Path) -> Result<()>` step before deserialization, mutating the TOML file in place.
- Focus-triggered reload: hook crossterm's `Event::FocusGained` (already supported since crossterm 0.27).
- Permission-graceful: wrap write errors in `if err.kind() == ErrorKind::PermissionDenied { warn!(); }`.

## Delta summary
- New items: 5 (`ConfigFilePolicy` 3-valued enum, YAML-rewrite migrations, fail-open permission semantics, focus-trigger reload elegance, state-vs-config separation philosophy).
- Refined: per-repo upward walk semantics.
- Remaining gaps: how exactly `Validate()` interleaves with migration (read first then validate; load order is fine).

## Round assessment
SUBSTANTIVE. Lane CONVERGED.
