# Pass B.6 — Extraction Validation

For each major finding, verify by re-reading source citations and check for inferred-vs-evidenced status.

## Validation table

| Finding (short) | Cited at | Re-verified | Notes |
|---|---|---|---|
| lazy\* signature = (viewName, key, handler) | `pkg/gui/keybindings.go:448-454` | YES | gocui SetKeybinding takes viewName param verbatim. |
| GlobalContext synthesises empty-viewName | `pkg/gui/context/setup.go:9-18` + `pkg/gui/controllers/global_controller.go:23` | YES | GLOBAL_CONTEXT has `View: nil`; global controller binds with no `ViewName` so empty default in keybindings.go:415-431 path applies. |
| Custom commands prepended (precedence) | `pkg/gui/keybindings.go:405-411` | YES | `bindings = append(customBindings, bindings...)`. Comment line 410 affirms intent. |
| Search prompt isolates bindings | `pkg/gui/keybindings.go:396-403` | YES | Returns early with only the search-context bindings when SEARCH_CONTEXT_KEY is current. |
| DisabledReason 5-shape table | `pkg/gui/keybindings.go:460-479` | YES | Inspected each branch: AllowFurtherDispatching → ErrKeybindingNotHandled; ShowErrorInPanel → return error; Text non-empty → ErrorToast + nil; nil text → nil silent; nil DisabledReason → run handler. |
| Side context push wipes stack | `pkg/gui/context.go:91-95` | YES | The `lo.Filter` discards every other context; ContextStack becomes `[c]`. |
| Popup nested cascade offset (+2, +1) | `pkg/gui/controllers/helpers/confirmation_helper.go:116-122` | YES | `x0 += 2; y0 += 1; return x0, y0, x0 + panelWidth - 1, y0 + panelHeight - 1`. |
| Popup serialisation TODO | `pkg/gui/controllers/helpers/confirmation_helper.go:199-203` | YES | "ignoring create popup panel because a popup panel is already open" log + comment "The proper solution is to have a queue". |
| `@` prefix toggles key-filter | `pkg/gui/context/menu_context.go:83-91` | YES | `if self.allowFilteringKeybindings && strings.HasPrefix(filter, "@") { filterKeybindings = true; return filter[1:] }`. |
| Custom commands prompt right-fold | `pkg/gui/services/custom_commands/handler_creator.go:47-131` | YES | The outer loop iterates `reverseIdx`, building each handler that captures the previous via `g := f` closure pattern. |
| Output mode 4-way routing | `pkg/gui/services/custom_commands/handler_creator.go:295-340` | YES | terminal → RunSubprocessAndRefresh; log → StreamOutput; logWithPty → UsePty; popup → Alert(title, output). |
| Per-repo upward walk for `.lazygit.yml` | `pkg/gui/gui.go:436-457` | YES | `for dir != prevDir { prepend ConfigFile; prevDir = dir; dir = filepath.Dir(dir) }`. |
| Layered config — CustomCommands accumulate | `pkg/config/app_config.go:193-199` | YES | `existingCustomCommands := base.CustomCommands; yaml.Unmarshal(...); base.CustomCommands = append(base.CustomCommands, existingCustomCommands...)`. |
| `useFuzzySearch` reads from `Gui.FilterMode` | `pkg/gui/controllers/helpers/search_helper.go:229,246` | YES | both `SetFilter` and `ReApplyFilter` call sites pass `self.c.UserConfig().Gui.UseFuzzySearch()`. |
| Filter applies per-keystroke (no debounce) | `pkg/gui/controllers/helpers/search_helper.go:223-236` | YES | `OnPromptContentChanged` directly calls `context.SetFilter(searchString, ...)` then `c.PostRefreshUpdate(context)`. No timer / delay. |
| LogAction sets yellow, LogCommand context-coloured | `pkg/gui/command_log_panel.go:25-52` | YES | `style.FgYellow.Sprint(action)` for actions; `style.FgMagenta` if `!commandLine` else `theme.DefaultTextColor` for commands. |
| Extras autoscroll discipline | `pkg/gui/extras_panel.go:48-93` | YES | All `scrollUp*`/`pageUp*`/`goToExtrasPanelTop` set Autoscroll false; `goToExtrasPanelBottom` sets true. |
| ConfigFilePolicy 3-valued | `pkg/config/app_config.go:56-69` | YES | iota enum with 3 distinct values, each handled in `loadUserConfig` line 155-181. |
| Theme is package globals | `pkg/theme/theme.go:9-48` | YES | 12 `var` declarations at package level. `UpdateTheme` reassigns all. |
| `?` menu = 3-section telescope | `pkg/gui/controllers/options_menu_action.go:13-79` | YES | Buckets local/global/navigation, AllowFilteringKeybindings true, KeepConflictingKeybindings true, HideCancel true. |
| TabView shares one window | `pkg/gui/context/context.go:172-176` + `pkg/gui/gui.go:432-443` | YES | `viewTabMap` maps window → []TabView; `SetTabClickBinding` registers per view. |
| Window arrangement is pure function | `pkg/gui/controllers/helpers/window_arrangement_helper.go:124-498` | YES | `GetWindowDimensions(args WindowArrangementArgs)` takes a value, returns a map. `window_arrangement_helper_test.go` 729 LOC test exercises it heavily. |

22 of 22 spot-checked findings confirmed. Zero inferences. All HIGH confidence.

## Items flagged as inferred / not directly evidenced

NONE in the findings list. All claims are grounded in source citations that have been re-read.

## Items flagged as known speculations

Two minor speculations explicitly noted:
1. **Pass 5 BC-DRAFT-18 closes with "Monocle should consider debouncing"** — this is a translation recommendation, not a source claim. Acceptable as a guidance note.
2. **Pass B-popup-patterns-r1 close: "Critically, monocle should fix the missing popup queue"** — recommendation derived from BC-DRAFT-008's evidenced gap, not a claim about lazygit.

Both clearly framed as recommendations rather than findings. Acceptable.

## Risk areas (where extraction might be subtly wrong)

- **`Tag: "global"`** — Pass 5 BC-3 says global controller's bindings have empty `ViewName` *and* sometimes `Tag: "global"` per Pass B-key-dispatch-r1. The `options_menu_action.go:66` predicate `binding.ViewName == "" || binding.Tag == "global"` shows both routes work. So a binding *can* have non-empty `ViewName` but `Tag: "global"` to surface as global in the `?` menu while still being view-scoped at dispatch. Minor subtlety, worth flagging in synthesis.

- **Per-context binding precedence — last-attached wins**. Pass B-key-dispatch-r1 says this clearly, but the source comment in `base_context.go:124-127` is a little terse. Re-verified: `for i := range self.keybindingsFns { bindings = append(bindings, self.keybindingsFns[len(self.keybindingsFns)-1-i](opts)...) }`. The "first binding in the bindings array takes precedence but we want the last keybindingsFn to take precedence" comment confirms the inversion.

- **`gocui` dispatch order across `(view, "")`** — claimed view-specific wins over global. This is a gocui behaviour. The lazygit code path treats it as such (e.g., the `JumpToBlock` global numerics work because no view-specific overrides exist on `1`-`5`). Validated by lazygit's runtime behaviour and matches the gocui API contract.

## Verdict
Extraction is high-fidelity. No claim made without a source citation. Two minor subtleties (Tag-as-faceting vs ViewName-as-scope; precedence-inversion comment) noted for clarity in the final synthesis.
