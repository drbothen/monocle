# Pass 7 — Holdout Seeds (where deeper passes should mine)

For each deepening lane, these are the file:line seeds that should be re-read in the convergence phase.

## tui-architecture seeds
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/gui.go:62-150` — Gui struct fields
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/gui.go:310-434` — onSwitchToNewRepo / onNewRepo lifecycle
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/views.go:19-78` — view ordering & z-stack
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/layout.go:13-207` — layout tick
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/helpers/window_arrangement_helper.go:124-498` — boxlayout assembly
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context.go` (full file, 376 LOC) — ContextMgr
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/setup.go` — ContextTree wiring
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/base_context.go` — base context plumbing
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers.go` (442 LOC) — controller/helper attach graph

## key-dispatch seeds
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/types/keybindings.go` (full)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/types/common.go:213-225` — DisabledReason
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/keybindings.go` (full, 479 LOC) — registration + guards + handler invoke
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/global_controller.go` (full, 257 LOC)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/options_map.go` (full, 147 LOC) — bottom-bar render
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/options_menu_action.go` (full, 87 LOC) — `?` menu sectioning

## popup-patterns seeds
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/popup/popup_handler.go` (full)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/helpers/confirmation_helper.go` (full, 453 LOC)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/confirmation_context.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/prompt_context.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/menu_context.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/suggestions_context.go`

## help-overlay-and-filter seeds
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/helpers/search_helper.go` (full, 327 LOC)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/filter_controller.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/search_controller.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/search_prompt_controller.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/filtered_list.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/filtered_list_view_model.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/search_trait.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/options_menu_action.go` (the `?` overlay)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/menu_context.go:83-91` — `@`-prefix filter-keybindings detection

## custom-commands-and-theming seeds
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/services/custom_commands/client.go` (full)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/services/custom_commands/handler_creator.go` (full, 344 LOC)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/services/custom_commands/keybinding_creator.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/services/custom_commands/session_state_loader.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/services/custom_commands/resolver.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/theme/theme.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/theme/style.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/theme/gocui.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/style/basic_styles.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/style/text_style.go`

## log-viewer-and-scrollback seeds
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/extras_panel.go` (full, 119 LOC)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/command_log_panel.go` (full, 189 LOC)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/command_log_controller.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/helpers/window_arrangement_helper.go:391-404` — `getExtrasWindowSize`

## config-loading seeds
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/config/app_config.go` (full, 737 LOC)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/config/user_config.go` (full, 1,104 LOC; line 408-506 = keybinding universals)
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/config/keynames.go`
- `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/gui.go:436-457` — per-repo discovery

## State checkpoint
pass: 7
status: complete
seeds-by-lane: 7 lanes seeded
