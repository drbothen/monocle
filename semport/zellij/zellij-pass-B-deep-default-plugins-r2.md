# Phase B Deep — Default Plugins Survey (Round 2)

## Refinements From Round 1

| Refinement | File:line |
|---|---|
| `ColoredElements` struct in status-bar uses `palette.text_unselected.background` and `.base` directly — confirms the semantic-token model is the API plugins actually consume | `default-plugins/status-bar/src/main.rs:80-120` |
| status-bar uses `style!` macro from `zellij-tile-utils` to build ansi_term `Style`s | `default-plugins/status-bar/src/main.rs:6-19` |
| `register_plugin!(State)` is the canonical entry; every plugin's `main.rs` has it as a top-level invocation, NOT inside a function. The macro must generate the wasm `_start` and `load`/`update`/`pipe`/`render` exports | universal |
| tab-bar's `InitialKeybinds` handling logic shows the optimization clearly: subscribe → cache → fold cached into incoming ModeUpdate when empty | `default-plugins/tab-bar/src/main.rs:60-80` |
| session-manager has 5 sibling files: `new_session_info.rs`, `resurrectable_sessions.rs`, `session_list.rs`, `single_screen.rs`, `ui/` — module-per-screen pattern | `default-plugins/session-manager/src/` |
| configuration plugin has only 5 files (`main.rs`, `presets_screen.rs`, `presets.rs`, `rebind_leaders_screen.rs`, `ui_components.rs`); the `Screen` enum dispatches between two sub-screens — clean two-pane UI architecture | `default-plugins/configuration/src/` |

## Confirmed

Round 1 four-archetype classification (decoration, fs-permission, stateful UI, reconfigure UI) covers all 13 plugins. The 4-widget UI library (Text/Table/Ribbon/NestedList) is the canonical rendering surface.

## Round 2 Status

Refinements name specific helper structs and sibling-file layout patterns. No new archetypes. Pass converges.

```yaml
pass: B
category: default-plugins-survey
round: 2
status: complete
timestamp: 2026-05-11T21:15:00Z
classification: nitpick
```
