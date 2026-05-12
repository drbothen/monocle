# Pass B (deep) — Popup Patterns (round 1)

## The five popup primitives, ranked by user-visible behaviour

| Primitive | Editable? | Suggestions? | Persistent? | Typical use |
|---|---|---|---|---|
| `Alert(title, msg)` | no | no | no | One-shot notification with single-key dismiss |
| `Confirm(opts)` | no | no | no | Y/N with handler |
| `Menu(opts)` | no (but filterable via `@`) | no | no | List of actions with `j/k` nav |
| `Prompt(opts)` | yes | optional | no | Text entry, optional autocomplete |
| `WithWaitingStatus(msg, fn)` | no | no | no | Spinner during async work; not a popup view but a status surface |

The `Toast(msg)` and `ErrorToast(msg)` calls are status-bar transient messages, not popups (no view of their own beyond the right-side bottom bar).

## Anatomy of `CreatePopupPanel`
`pkg/gui/controllers/helpers/confirmation_helper.go:190-249`

The function is one path for both confirmation popups (read-only) and prompt popups (editable). The branch happens on `opts.Editable`:
- **Editable=true** → uses `Prompt` context, optionally activates `Suggestions` context underneath, sets prompt-specific keybindings (`setPromptKeyBindings`, line 259).
- **Editable=false** → uses `Confirmation` context, sets confirmation-specific keybindings (`setConfirmationKeyBindings`, line 251). **Panics if `FindSuggestionsFunc != nil`** because suggestions only work with editable.

Both paths:
1. Lock `PopupMutex`.
2. Check `CurrentPopupOpts` — if non-nil and not a loader, log error and abort (BC-DRAFT-008).
3. Clear any previously-set keybindings on Confirmation and Prompt contexts (line 208-209).
4. Prepare the panel content (text area for prompt, formatted text for confirmation).
5. Wire `OnConfirm` and `OnClose` to user-supplied handlers wrapped in popup-close logic.
6. Push the context onto the stack.

## The `Resize` callback pattern is the *only* way popups stay correct on resize
`confirmation_helper.go:315-331` — `ResizeCurrentPopupPanels` is called once per layout tick from `layout.go:190`. It walks the stack of current popups (`Context().CurrentPopup()`) and dispatches by context identity to the matching `resize*` function. It tracks the `parentPopupContext` to enable cascaded positioning.

So **popups are never explicitly resized**; every layout tick the helper recomputes their geometry. This is the "stateless layout" pattern: each tick produces fresh dimensions from the popup's content + screen size.

## Popup size limits
- `getPopupPanelWidth(maxWidth)` (line 135) — desired = `min(4*width/7, maxWidth)`. Floor at `min(width-2, 80)`. So menus tend to use 90 max width (line 338); confirms use 80; commit-message uses up to 100.
- `getPopupPanelDimensionsAux` — height capped at `3/4` of screen height (line 113-115). If the parent popup exists, offset `(x+2, y+1)` from it (line 116-122). Otherwise centred.

## The "suggestions context" is a peer of prompt, not a child
`confirmation_helper.go:170-180` — when a `FindSuggestionsFunc` is provided to `Prompt`, the suggestions view becomes visible *underneath* the prompt. The user toggles focus with `Universal.TogglePanel` (default `<tab>`). Each context has its own confirm/close handler; the prompt's `OnConfirm` uses the typed text, the suggestions' `OnConfirm` uses the selected suggestion's `Value` (line 264-270).

The suggestions title contains the toggle keybinding label (line 178): `Sprintf(self.c.Tr.SuggestionsTitle, self.c.UserConfig().Keybinding.Universal.TogglePanel)`. **This is the user-facing hint mechanism**: titles dynamically reflect the keybinding the user has configured.

## Menu: filtering, sections, keybinding shortcuts
`pkg/gui/context/menu_context.go`

Three menu superpowers:
1. **Per-item keybindings** — `MenuItem.Key` (`pkg/gui/types/common.go:264`) lets a menu item declare a single keystroke. Pressing it directly inside the menu invokes that item. `MenuContext.GetKeybindings` (line 205-232) collects them. The `keybindingsTakePrecedence` flag decides whether menu-item keys override base list nav (BC-DRAFT-023).
2. **Inline filter** — menus are filterable lists; typing into the menu invokes the filter trait. `allowFilteringKeybindings` (line 122) flips the filter target from `LabelColumns` to `LabelForKey(item.Key)` when prefix `@` is typed (line 83-91).
3. **Sections** — `MenuItem.Section *MenuSection` lets items group. Pointer-equality decides whether two items belong to the same section (line 280-284 docstring). Section headers are rendered as injected non-model rows in green (`menu_context.go:194-198`).

## DisabledMenuItem rendering
`menu_context.go:136-139` — disabled items are struck-through using `style.FgDefault.SetStrikethrough().Sprint(displayStrings[0])`. The first label column is mutated *in place* before return — this is a small mutation hazard but tolerable because the menu items live in the temporary popup state.

When the user presses Enter on a disabled item, `OnMenuPress` (line 235-241) toasts the disabled reason rather than invoking the handler. If `ShowErrorInPanel: true` the reason becomes an error popup.

## Tooltip view is bound to the menu
`pkg/gui/controllers/helpers/confirmation_helper.go:344-352` — when a menu is resized, an auxiliary tooltip view is sized just below it. The tooltip content comes from `TooltipForMenuItem` (line 444-453) which concatenates the menu item's `Tooltip` with the disabled-reason text (if any). This is the only popup that owns a sidecar view.

## Prompt's `HandleDeleteSuggestion` and `AllowEditSuggestion` are unique
`PromptOpts.HandleDeleteSuggestion func(int) error` (`pkg/gui/types/common.go:204`) — invoked when the user presses the delete-suggestion key (default `<c-d>`) while focused on a suggestion. The branch-pick prompt uses this to allow deleting a branch from the suggestions.

`AllowEditSuggestion bool` — when true and a suggestion is highlighted, the user can press a key to edit the suggestion text inline (e.g. paste a branch name then refine).

## Toast routing
`pkg/gui/popup/popup_handler.go:62-71` — `Toast` and `ErrorToast` both delegate to `toastFn(message, kind)`. The actual rendering happens via `AppStatusHelper` (`pkg/gui/controllers/helpers/app_status_helper.go`, 157 LOC) which manages the right-side bottom-bar slot. Toast colour is `theme.ToastColor` (status) vs red (error). They auto-expire after a configurable duration.

## Translation to monocle
ratatui already supports clean overlay rendering via separate render passes. The translation map:

```
lazygit.PopupHandler.Confirm   ─→ monocle.app.show_modal(Modal::Confirm { ... })
lazygit.PopupHandler.Prompt    ─→ monocle.app.show_modal(Modal::Prompt { ... })
lazygit.PopupHandler.Menu      ─→ monocle.app.show_modal(Modal::Menu { items, sections })
lazygit.PopupHandler.Alert     ─→ Modal::Confirm with no handler
lazygit.PopupHandler.Toast     ─→ monocle.app.push_toast(ToastKind::Status, msg)
WithWaitingStatus              ─→ monocle.app.with_spinner(msg, async { ... })
```

Suggestions become an opt-in `suggestion_source: Box<dyn FnMut(&str) -> Vec<Suggestion>>` on the `Prompt` modal variant.

**Critically**: monocle should fix the missing popup queue from BC-DRAFT-008 by accepting modal pushes into a `VecDeque<Modal>` and only showing one at a time.

## Delta summary
- New items: 6 (sidecar tooltip view, suggestion edit/delete keys, menu @ filter mode, section pointer-equality, popup width formula, dynamic title with config key interpolation).
- Refined: nested popup positioning offsets.
- Remaining gaps: how exactly `getPromptInput` is invoked async from the helper (irrelevant detail).

## Round assessment
SUBSTANTIVE — six new findings reify the popup pattern. Lane CONVERGED.
