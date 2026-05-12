# Pass 6 — NFR Catalogue (scoped)

## Performance
- **Filter** uses fuzzy-patricia trie for filename suggestions (`pkg/gui/types/common.go:328` — `FilesTrie *patricia.Trie`) and `sahilm/fuzzy` for general list filtering. No debouncing — see BC-DRAFT-018.
- **Lazy view-buffer** — `pkg/tasks/ViewBufferManager` (not deepened) backs the main + secondary + extras panels. The layout calls `ReadLines(heightDiff)` on resize (`pkg/gui/layout.go:42-47`) to stream more content into the visible window.
- **Background routine manager** (`pkg/gui/gui.go:136` + `BackgroundRoutineMgr`) owns periodic background fetch + refresh + update check; each is wrapped in `OnWorker` so the UI loop stays responsive.
- **Lock detection** — `sasha-s/go-deadlock.Mutex` everywhere helpers touch shared state (`pkg/gui/types/common.go:336-346`).
- **Hash pool** — `pkg/utils.StringPool` (`pkg/gui/types/common.go:332`) deduplicates commit hash strings to avoid heap churn on huge logs.

## Security
- **Mask** field on PromptOpts/CreatePopupPanelOpts (`pkg/gui/types/common.go:178,192,205`) masks user input (used for credentials prompts via `credentials_helper.go`).
- **Custom commands run via `OS().Cmd.NewShell`** with the user's configured shell (`pkg/gui/services/custom_commands/handler_creator.go:182,295`). Templates resolve through `text/template`; the `quote` template-function (`handler_creator.go:281`) is the user's only shell-injection guard. Monocle should make this much stricter (or never inherit the shell-string approach in the first place).
- **Disabled startup popups** (`UserConfig.DisableStartupPopups`) — UX setting, not security, but mentioned because the popup version `StartupPopupVersion` (`pkg/gui/gui.go:54`) drives forced display of breaking-change notes.

## Observability
- **Command log panel** — the "extras" window is a live ledger of every action + git command issued (`pkg/gui/command_log_panel.go:25-52`). Actions are yellow, commands are default-text, "non-real" commands rendered in magenta.
- **Random tip on startup** — `getRandomTip()` (`pkg/gui/command_log_panel.go:71-189`) seeds 28 keybinding-aware tips in the log on startup.
- **Debug log path** — `pkg/config/app_config.go:731-737` — `LAZYGIT_LOG_PATH` env var override; default is the XDG state file `development.log`.
- **Toast** for non-blocking ephemeral status; **Confirmation/Alert** for blocking; **WithWaitingStatus** for spinner-guarded async work (`pkg/gui/types/common.go:139-141`).

## Reliability
- **Popup queueing is missing** — explicitly TODO in code (`confirmation_helper.go:199-203`). See BC-DRAFT-008.
- **`AllowFurtherDispatching` on disabled-reason** lets disabled bindings opt-into "let the next handler try" semantics (`pkg/gui/keybindings.go:464`).
- **Config migration** — `pkg/config/app_config.go:219-330` walks the loaded YAML and applies a sequence of `pathsToReplace` renames + bespoke transformers (null-keybinding → `"<disabled>"`, `commitPrefix` scalar → array, `subprocess` / `stream` / `showOutput` → `output:` enum). Writes back if anything changed.
- **Read-only filesystem grace** — `app_config.go:166-170,636-641` — if writing fails due to permission denied, lazygit silently continues. Avoids breaking when run as a strict user.

## Scalability
- **Multi-repo support via `RepoStateMap`** (`pkg/gui/gui.go:79`) — each repo's gui state is cached so jumping between submodules is instant.
- **`ItemOperation` per-item state** (`pkg/gui/gui.go:123`, `pkg/gui/types/common.go:352`) — multiple long-running operations can be in-flight concurrently with per-item visual feedback.

## Accessibility / Internationalization
- **i18n** — `pkg/i18n/` (out of deepening scope) holds `TranslationSet` per language; `Tr` is the field on `HelperCommon` carrying current language. Switching language reloads strings (`pkg/gui/gui.go:463-470`).
- **Border style configurable** — `pkg/gui/views.go:159-169` — single/double/rounded/hidden/bold. The "hidden" preset is a UX nicety for users on minimal terminals.
- **Nerd-font detection** — `Gui.NerdFontsVersion ∈ {"", "2", "3"}` (`pkg/config/user_config.go:152`) — old icon set can be opted out for users without the font.

## Configuration scopes
- Defaults baked at compile-time (`pkg/config/config_default_platform.go`).
- Global config: `$XDG_CONFIG_HOME/lazygit/config.yml` (legacy `jesseduffield/lazygit/config.yml` also probed).
- Override via env: `CONFIG_DIR` overrides the discovery dir; `LG_CONFIG_FILE` provides comma-separated explicit paths.
- Per-repo: `<repo>/.git/lazygit.yml` (always) + walked-upward `<dir>/.lazygit.yml` chain (`pkg/gui/gui.go:436-457`).
- State: `$XDG_STATE_HOME/lazygit/state.yml` for things like recent-repos, last-update-check, PR cache (`pkg/config/app_config.go:696-712`).

## State checkpoint
pass: 6
status: complete
