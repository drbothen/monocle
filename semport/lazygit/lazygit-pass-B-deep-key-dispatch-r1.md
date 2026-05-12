# Pass B (deep) — Key Dispatch (round 1)

## The lazy\* signature pattern — in one sentence
**Every keybinding is a `Binding{ViewName, Key, Handler, ...}` registered at `(viewName, key)` in gocui's key-table; a binding with `ViewName == ""` is the global fallback consulted when no view-specific binding matches.**

That is the single architectural primitive both lazyclaude reimplementations inherit.

## The full registration lifecycle
1. **Compile-time** — controllers are simply functions returning `[]*Binding` from `GetKeybindings(opts)`. They cannot reference helpers directly; they take a `*ControllerCommon` instead.
2. **Repo-load time** — `resetHelpersAndControllers` (`pkg/gui/controllers.go:20`) builds all helpers, then constructs all controllers, then calls `controllers.AttachControllers(ctx, controller...)` to register each controller's `GetKeybindings` on each context's `keybindingsFns` slice (BaseContext.AddKeybindingsFn).
3. **`resetKeybindings`** (`pkg/gui/keybindings.go:415`) — clears the gocui keybinding map, then iterates `[]*Binding` from `GetInitialKeybindingsWithCustomCommands` calling `g.SetKeybinding(viewName, key, handler)` once per binding (line 421). Mouse bindings go through `SetViewClickBinding` (line 456).
4. **Custom commands prepended** — `GetInitialKeybindingsWithCustomCommands` (`keybindings.go:392`) inspects whether the search prompt is open (if so, returns *only* the search context's bindings — see BC-DRAFT-005), otherwise prepends user custom-command bindings to default ones (line 411).
5. **Dispatch (gocui)** — when a key arrives, gocui first looks up `(currentViewName, key)`. If found, invokes its handler. If not found, falls back to `("", key)` — the global table. Lazygit's `SetKeybinding` (line 448) wraps the actual handler with `callKeybindingHandler` which is where `GetDisabledReason` is consulted.

## Sources of bindings ordered by precedence (highest first)
1. **Search prompt bindings** (when the search prompt is open) — `keybindings.go:396-403`. Nothing else fires.
2. **Custom commands** — prepended on every `resetKeybindings`. User authority wins (`keybindings.go:411`).
3. **Per-context controllers** (in *reverse* attach order — last-attached wins on collision) — `base_context.go:121-130`.
4. **Global controller** — registered with `ViewName: ""`, consulted as gocui fallback (`controllers.go:413` + `global_controller.go:23`).
5. **Built-in scroll bindings** in `GetInitialKeybindings` (`keybindings.go:78-358`) — registered with `ViewName: ""` for `Universal.ScrollUpMain` etc. Then per-view ones for confirmation/extras/main/secondary.

## Disabled-reason semantics — the four return shapes
`pkg/gui/keybindings.go:460-479`:
| `GetDisabledReason` returns | Effect |
|---|---|
| `nil` | Binding fires normally. |
| `&DisabledReason{Text:""}` | Silently swallowed. Used to hide bindings from options-map without surfacing an error. |
| `&DisabledReason{Text: msg}` | Toast `"Disabled: msg"`, swallow keystroke. |
| `&DisabledReason{ShowErrorInPanel: true, Text: msg}` | Return Go error → renders as a Confirmation popup. |
| `&DisabledReason{AllowFurtherDispatching: true}` | Return `ErrKeybindingNotHandled` so the next handler in gocui's chain (e.g. the global one for the same key) tries. |

This last is how lazygit lets a per-view binding "step aside" — e.g. if `j` is disabled in commits during loading, the global `j` (if any) still works.

## Guards vs DisabledReason — when to use which
- **Guards** (`OutsideFilterMode`, `NoPopupPanel`) are applied at *registration*. They wrap the handler before the binding is created. Use when the precondition is binary and a side-effect (showing a confirm dialog, returning nil) is the appropriate failure mode.
- **DisabledReason** is consulted at *dispatch*. Use when the binding should appear in menus / options-bar but be visually struck-through when unavailable, and when you want to communicate *why* via tooltip.

Both are needed. Guards are cheaper and don't need to render anything; disabled-reasons participate in the UI surface.

## The `Tag` field is a faceting mechanism
`pkg/gui/types/keybindings.go:28` — `Tag string` (e.g. `"navigation"`, `"global"`). The options-menu action (`options_menu_action.go:66-74`) sorts bindings into three buckets based on `Tag == "global"` or `Tag == "navigation"`; everything else is "local". This is how the keybindings menu has a stable three-section layout.

## Tag is also used by the `JumpToBlock` keybindings
`pkg/gui/keybindings.go:300,306,318` — extras-panel scroll bindings have `Tag: "navigation"`. The `JumpToBlock` array binding (`config.KeybindingUniversalConfig.JumpToBlock []string`, default `["1","2","3","4","5"]`) lets users jump directly to the corresponding side panel without sequential tab presses (see `pkg/gui/views.go:218-237` for the visible labels).

## Mouse bindings are a separate channel
`pkg/gui/types/keybindings.go` reuses `gocui.ViewMouseBinding` rather than `Binding`. Set with `g.SetViewClickBinding(binding)` (`keybindings.go:457`). The two are kept distinct because the dispatch semantics differ (mouse needs coordinate hit-testing).

## `Alternative` and `Alt1/Alt2` keybindings exist for compatibility
`pkg/gui/types/keybindings.go:27` — `Alternative string` is *documentation only* (not a second key registration). The actual alternate key registrations are separate `Binding{}` entries with the same handler but different Key — see `Universal.ScrollUpMainAlt1` and `ScrollUpMainAlt2` (`keybindings.go:102-120`). Used to cover mouse-wheel + keyboard + fn-key variants.

## Translation strategy to monocle (Action enum)
Lazygit's binding has a *handler closure* + metadata. Monocle's idiomatic Rust pattern is to dispatch to an `enum Action`:

```rust
struct Binding {
    pub view: Option<ViewId>,       // None = global
    pub key: KeyChord,
    pub action: Action,             // enum dispatched in update()
    pub description: Cow<'static, str>,
    pub short_description: Option<Cow<'static, str>>,
    pub tag: BindingTag,
    pub display_on_screen: bool,
    pub get_disabled_reason: Option<fn(&AppState) -> Option<DisabledReason>>,
    pub opens_menu: bool,
}
```

Where:
- `Action` is the data variant (not a closure) so bindings are `Copy`/`Eq` and inspectable in tests.
- Update routing is `match action { Action::ScrollUpMain => ..., Action::OpenFilter => ..., }`.
- Guards become `pub guards: BindingGuards` flags (`outside_filter`, `no_popup`) checked in `App::dispatch`.
- DisabledReason same shape as Go.

This preserves the lazy\* signature pattern (view-scoped + global) and inherits the disabled-reason richness.

## Delta summary
- New items: 4 (registration lifecycle precise ordering, alternate-key strategy, mouse channel separation, monocle Action-enum translation).
- Refined: the 5-source precedence ordering, disabled-reason 5-shape table.
- Remaining gaps: `Universal.JumpToBlock` user-config plumbing (deepened separately in custom-commands lane).

## Round assessment
SUBSTANTIVE — the lazy\* signature pattern is now fully articulated. Lane CONVERGED.
