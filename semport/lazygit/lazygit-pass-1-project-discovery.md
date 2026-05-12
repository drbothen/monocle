# Pass 1 — Project Discovery (SCOPED)

## Snapshot
- Repo: `/Users/jmagady/Dev/monocle/.reference/lazygit/` (jesseduffield/lazygit)
- Branch: `master`, HEAD `c4935036` ("Remove the invitation to submit PRs from the issue template (#5603)")
- Top-level files: 39 entries, ~34 MB shallow.
- Language: Go, `go 1.25.0`, module `github.com/jesseduffield/lazygit`.
- Build orchestration: `Makefile`, `justfile`, `default.nix`, `flake.nix`, `Dockerfile`, `goreleaser.yml`.
- Entry point: `/Users/jmagady/Dev/monocle/.reference/lazygit/main.go` (423 bytes; small bootstrap).
- TUI substrate: `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gocui/` (fork of `awesome-gocui/gocui`, ~8,011 LOC over 20 Go files). This is lazygit's vendored terminal cell framework.
- Layout substrate: `lazycore/pkg/boxlayout` (external dep, see `pkg/gui/controllers/helpers/window_arrangement_helper.go:9`).
- Style engine: `github.com/gookit/color` plus `lazygit/pkg/gui/style` and `lazygit/pkg/theme`.

## Why monocle ingests this
lazygit is the source TUI from which both lazyclaude reimplementations inherit: panel layout, `?` keybindings menu, `/` filter, `customCommands` framework, context-aware key dispatch, scrollback browsing (extras/log panel), telescope-style help. Monocle's ratatui app should mirror these UX conventions explicitly rather than via the lazyclaude middlemen.

## IN-SCOPE inventory (this is where the deepening happens)

| Area | Path (absolute) | Files | LOC | Why |
|---|---|---|---|---|
| TUI core (god struct, layout, views, context) | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/` (top-level only, excl. controllers/) | 25 | ~5,181 | The orchestrator. `gui.go` (1,212 LOC) wires everything. |
| Context system | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/` | 37 | ~4,135 | The panel-with-state pattern. `base_context.go`, `setup.go`, `menu_context.go`. |
| Types (cross-package contracts) | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/types/` | 13 | ~1,242 | `Binding`, `Context`, `Helpers`, `Model`, `KeybindingsOpts`. |
| Controllers (key dispatch handlers) | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/` | 64 | ~14,151 | Each controller declares `GetKeybindings` per context. **NB:** most of this body is git-specific business logic which we mark out-of-scope; we extract only the binding-registration pattern. |
| Controller helpers (cross-cutting) | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/helpers/` | 43 | ~9,267 | `search_helper.go`, `confirmation_helper.go`, `window_arrangement_helper.go`, `mode_helper.go`. |
| Popup handler | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/popup/popup_handler.go` | 1 | 152 | Confirm / Alert / Prompt / Menu / Toast / WaitingStatus surface. |
| Custom commands framework | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/services/custom_commands/` | 8 | ~1,278 | User-defined keybinding-driven actions with prompt chains. |
| Theming | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/theme/` + `pkg/gui/style/` | 4 + 6 | 222 + 571 | Colour-name resolution, runtime theme switching. |
| Config loading | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/config/` | 14 | ~4,904 | `app_config.go` layered loading, migration, `user_config.go` schema (1,104 LOC), `keynames.go` (207 LOC). |
| Status panel / app footer | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/status/`, plus `pkg/gui/options_map.go`, `pkg/gui/information_panel.go` | – | ~600 | Bottom-bar wiring and breadcrumb keybinding list. |
| Log / scrollback (extras) | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/extras_panel.go`, `command_log_panel.go` | 2 | 308 | The lineage for monocle's command log. |
| Modes | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/modes/` | 4 dirs | – | Cherry-pick / filter / diff / mark-base global modes (state, not git logic). |
| Patch exploring & merge conflicts UI | `pkg/gui/patch_exploring/`, `pkg/gui/mergeconflicts/` | – | – | View-side patch range/selection state (UX pattern, not git logic). |

## OUT-OF-SCOPE (cited once, not deepened)
- `pkg/commands/` — git command construction & shell execution. Monocle isn't a git tool.
- `pkg/commands/git_commands/`, `pkg/commands/oscommands/` — same reason.
- `pkg/integration/` — integration tests against real git fixtures.
- `vendor/` — vendored Go deps.
- `docs/`, `docs-master/`, `demo/` — narrative documentation and demo recordings.
- Specific GitHub / GitLab integration (`pkg/commands/github_pull_request.go` etc.) beyond observing it as an extensibility surface.
- `pkg/snake/` — easter-egg snake game.
- `pkg/i18n/` — translation strings; treated as a flat string source, not a deepening target.
- `pkg/cheatsheet/`, `pkg/jsonschema/` — generators for cheatsheet & config schema; mentioned as artefacts, not architecture.
- `pkg/tasks/` — async task / view buffer manager. Internal optimisation; one mention only.

## Tech-stack notable dependencies
- `github.com/gdamore/tcell/v3` — terminal cell framework (used through gocui fork).
- `github.com/jesseduffield/lazycore` — shared lazy* glue (notably `pkg/boxlayout`).
- `github.com/sahilm/fuzzy` — fuzzy filter matcher (powers `/` filter when `useFuzzySearch` enabled).
- `github.com/integrii/flaggy` — CLI flag parsing.
- `github.com/adrg/xdg` — XDG base-dir resolution for config + state files.
- `github.com/sasha-s/go-deadlock` — deadlock-detecting mutex (used throughout `gui.go` and helpers).
- `github.com/samber/lo` — collection helpers (lo.Map / lo.Filter / lo.Contains).
- `github.com/gookit/color` — ANSI colour rendering.
- `gopkg.in/ozeidan/fuzzy-patricia.v3/patricia` — trie used for filename suggestion.

## Scope statement (frozen)
For this scoped ingest, "lazygit" is treated as a TUI framework reference, not as a git client. Every dive that follows is anchored in a directory above the OUT-OF-SCOPE line. Git semantics are noted only when they intrude on TUI behaviour (e.g., the mode helper's `IsAnyModeActive` driving footer rendering).
