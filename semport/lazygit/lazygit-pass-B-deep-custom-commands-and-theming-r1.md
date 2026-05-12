# Pass B (deep) — Custom Commands and Theming (round 1)

## Custom Commands — the data shape (from `pkg/config/user_config.go` + the resolver chain)

A `CustomCommand` entry in the user config carries:
- `Key string` — keybinding (must be a valid key per `GetValidatedKeyBindingKey`).
- `Context string` — comma-separated list of context-keys (or `"global"`). Validates against `AllContextKeys` (`pkg/gui/context/context.go:53`).
- `Command string` — `text/template`-rendered shell command string.
- `Description string` / `LoadingText string` / `OutputTitle string`.
- `Output string ∈ {"", "terminal", "log", "logWithPty", "popup"}` — delivery mode.
- `After CustomCommandAfter` — `CheckForConflicts bool` (post-action conflict-detection hook).
- `Prompts []CustomCommandPrompt` — chained user prompts (see below).
- `CommandMenu []CustomCommand` — recursive sub-menu of commands.

The `CommandMenu` recursive shape is how users build their own menus (`pkg/gui/services/custom_commands/client.go:66-110`). Pressing a top-level menu key opens a Menu populated with sub-commands, optionally context-filtered (`client.go:80-90` — sub-commands with a `Context:` are *only* shown if the current view is in that context list).

## Prompts — the chain machinery
`pkg/gui/services/custom_commands/handler_creator.go:47-131`

The handler is built right-to-left:
- Each prompt becomes a function that calls a *resolved* prompt's UI primitive (`Prompt`, `Menu`, `Menu`, `Confirm`), passing the user-supplied response into a wrapped continuation.
- Continuations capture both `promptResponses []string` (indexed) and `form map[string]string` (keyed by `prompt.Key`).
- The final continuation runs the command template through `getResolveTemplateFn` (line 273-286) using `text/template` with `funcMap = {"quote": OS.Quote, "runCommand": Git.Custom.TemplateFunctionRunCommand}`.
- A prompt's `Condition` template is also evaluated; if it resolves to empty or `"false"`, the prompt is skipped (line 110-126).

Prompt types:
- `input` — `Prompt(...)` with optional `FindSuggestionsFunc` from a preset or shell command.
- `menu` — `Menu(...)` with predefined options.
- `menuFromCommand` — runs a shell command, then `menuGenerator.call` parses output lines with a regex `Filter` + `ValueFormat` + `LabelFormat` to produce menu items.
- `confirm` — `Confirm(...)`, no captured value; just gates continuation.

Suggestions presets (line 197-216):
```
authors, branches, files, refs, remotes, remoteBranches, tags
```

## Output delivery modes
`handler_creator.go:295-340` — the final command runs with mode-specific routing:
- `terminal` → `RunSubprocessAndRefresh` — suspends gocui, hands the terminal to a child process (full TTY).
- `log` → stream output line-by-line into the command-log view.
- `logWithPty` → same but with PTY allocated (preserves colours/CRs).
- `popup` → run, capture stdout, show in an Alert popup (with `OutputTitle` template).
- `""` (default) → run silently, refresh model afterwards.

If `After.CheckForConflicts: true` and the command errors, `MergeAndRebaseHelper.CheckForConflicts(err)` is invoked (line 320).

## Session state in templates
`session_state_loader.go` (259 LOC) — builds a `SessionState` struct exposed to templates as `.Branches`, `.Files`, `.SelectedFile`, `.Commits`, `.SelectedSubCommit`, `.SelectedRemote`, etc. This is how a user can write:

```yaml
- key: 'C'
  context: 'localBranches'
  command: 'git checkout {{.SelectedLocalBranch.Name}}'
```

Templates also have `{{ .Form.foo }}` and `{{ index .PromptResponses 0 }}` (`handler_creator.go:267-272`).

## Keybinding-creator validation
`keybinding_creator.go:25-89`

- Empty `Context:` → error with friendly message naming all valid context keys.
- Unknown `Context:` → similar error.
- A binding is generated for *each* of the comma-separated contexts.
- The `Key` is validated with `GetValidatedKeyBindingKey` (which errors at registration time, not at keypress time — a misconfigured custom command fails fast).

## Theming — runtime mutable globals
`pkg/theme/theme.go:9-48` declares 12 package-level vars. Each is reassigned in `UpdateTheme(themeConfig)` (line 51-76).

The colour resolution chain:
- User config has `Gui.Theme.ActiveBorderColor []string` (a slice, e.g. `["green", "bold"]`).
- `GetGocuiStyle(colorStrings)` (`pkg/theme/gocui.go`) translates the strings into a gocui `Attribute` (bitmask).
- `GetTextStyle(colorStrings, isBackground)` (`pkg/theme/style.go`) translates into a `style.TextStyle` (the gookit/color wrapper).
- `style.TextStyle` (`pkg/gui/style/text_style.go`, 157 LOC) is the user-level type with builder methods: `SetBold()`, `SetUnderline()`, `SetStrikethrough()`, `MergeStyle(other)`, `Sprint(content)`, `Sprintf(fmt, args)`.

The `ColorMap` (`pkg/gui/style/basic_styles.go:38-52`) is the named-colour table for the eight ANSI colours, used to support template colour functions (`TemplateFuncMapAddColors`, line 62-69) — so users can write `{{ red "danger" }}` in templates.

## Per-author colours
`UserConfig.Gui.AuthorColors map[string]string` → `authors.SetCustomAuthors(colors)` (`pkg/gui/gui.go:495`) wires per-author colour overrides into the commits-view rendering. This is git-specific but worth flagging because the *mechanism* — a config map plumbed into a presentation singleton — is reusable for monocle's "per-session" or "per-thread" colour overrides.

## Theme runtime reload
The full cycle:
1. User edits `~/.config/lazygit/config.yml` while lazygit is open.
2. gocui regains focus → `SetFocusHandler` callback fires (`pkg/gui/gui.go:351`).
3. `ReloadChangedUserConfigFiles` stats every config file and compares to `modDate` cache (`pkg/config/app_config.go:559-581`).
4. If anything changed: `onUserConfigLoaded()` → `setColorScheme()` → `UpdateTheme(...)` → `configureViewProperties()` (frame runes etc.).
5. Plus `resetKeybindings()` to re-apply potentially-changed bindings.
6. `checkForChangedConfigsThatDontAutoReload(oldConfig, newConfig)` (line 363) catches changes that *can't* be applied live (e.g., language change) and shows a confirm telling the user to restart.

This is **focus-triggered hot-reload**, not filesystem-watcher-driven. Simpler and avoids fsnotify dependencies.

## Custom-commands menu = recursive
`pkg/gui/services/custom_commands/client.go:66-110` — a `customCommand` with `CommandMenu: [...]` becomes a menu, not a direct action. Sub-commands can themselves have `CommandMenu`, recursing. The `if subCommand.Context != "" && subCommand.Context != "global"` filter (line 80-90) hides items that aren't relevant to the currently-focused view, so the user only sees usable commands.

## Translation to monocle
- Custom commands map cleanly to monocle's `monocle.toml [[custom_commands]]` array.
- The `Context: "files"` mechanism translates to monocle's `ContextId` enum-or-string match.
- `text/template` becomes Rust's `tera` or `handlebars`. The minimal funcMap (`quote`, `runCommand`) is achievable.
- Session-state for templates: monocle exposes `MonocleState` with `selected_session`, `selected_thread`, `current_file`, etc.
- Theme = a struct (not globals). `theme: Arc<RwLock<Theme>>` threaded through the App for hot-reload.
- Hot-reload triggered on focus regain via crossterm focus events (already supported).

## Delta summary
- New items: 7 (the recursive CommandMenu shape, template funcMap composition, focus-triggered hot reload, four-axis Output delivery, prompt Condition templating, AuthorColors plumbing pattern, KeepConflictingKeybindings-vs-allowFiltering interplay).
- Refined: theming global-state lifecycle.
- Remaining gaps: how `runCommand` template func recurses (would deepen into git-specific paths — out of scope).

## Round assessment
SUBSTANTIVE. Lane CONVERGED.
