# Pass 7: Holdout Seeds — zellij

Seed list of subsystems and questions to be deepened in Phase B. Each row maps to a planned `zellij-pass-B-deep-<name>-r1.md`.

## Deepening Targets

| Category | Open Questions | Round 1 Target File |
|---|---|---|
| **workspace** | Exact crate boundaries; which types are intentionally placed in `zellij-utils` vs each plane; feature-flag fan-out; how `xtask` orchestrates wasm-plugin builds. | `zellij-pass-B-deep-workspace-r1.md` |
| **ipc** | Full message catalog with semantics; `route` thread variant dispatch table; lifecycle of an `attach` (full sequence diagram); error/disconnect handling; `ConnStatus` probe; how is multi-client coordination done? | `zellij-pass-B-deep-ipc-r1.md` |
| **plugin-sdk** | Full inventory of `PluginCommand` variants and their permission requirements; full inventory of `Event` variants and their subscriptions; worker lifecycle; the plugin host ABI surface in `zellij_exports.rs`. | `zellij-pass-B-deep-plugin-sdk-r1.md` |
| **configuration-and-keybinds** | KDL parser entry points; merge semantics across multiple config locations; the `Reconfigure` runtime flow; how plugins like `configuration` mutate live config; theme hot-swap mechanics. | `zellij-pass-B-deep-config-keybinds-r1.md` |
| **session-persistence** | The full path of a detach: when is the layout written? what triggers it? what about pane contents? what about plugin state? how is resurrection orchestrated? what fails gracefully? | `zellij-pass-B-deep-session-persistence-r1.md` |
| **theming** | Token model; KDL theme syntax; runtime theme switching; host-terminal theme integration via CSI 2031 / DSR 997. | `zellij-pass-B-deep-theming-r1.md` |
| **default-plugins-survey** | One representative pass per plugin: status-bar (mode awareness), tab-bar (basic shape), session-manager (the bookmark-equivalent), strider (filesystem permission), configuration (Reconfigure flow), plugin-manager. | `zellij-pass-B-deep-default-plugins-r1.md` |

## Held-Out Code Paths (intentionally NOT deepened)

These were touched during the broad sweep but explicitly excluded by user scope:

- `zellij-server/src/screen.rs` (9,958 LOC of state mutation; out-of-scope for layout geometry)
- `zellij-server/src/pty.rs` + `pty_writer.rs` + `terminal_bytes.rs`
- `zellij-server/src/panes/grid.rs`, `terminal_pane.rs`, `terminal_character.rs`, `alacritty_functions.rs`, `sixel.rs`, `hyperlink_tracker.rs`, `link_handler.rs`, `selection.rs`, `search.rs`
- `zellij-server/src/panes/floating_panes/`, `panes/tiled_panes/`
- `zellij-client/src/stdin_ansi_parser.rs`, `keyboard_parser.rs`
- `zellij-client/src/remote_attach/` (SSH-style flow)
- `zellij-utils/src/vendored/termwiz/`
- `zellij-utils/src/input/mouse.rs`
- All `*_mouse*` files anywhere in the tree
- `wix/` (Windows installer)

## Known Unknowns

1. **How does the resurrected session reconnect to its plugins?** When a session is killed and resurrected, do plugin instances start fresh? What about plugin-state persistence?
2. **What happens to background workers when the plugin is reloaded?** They are spawned during `load`, so a reload presumably tears them down.
3. **What is the recovery story for a wedged plugin?** A wasm plugin that loops forever should be killable, but wasmi is single-threaded per Instance.
4. **What's the maximum sensible Render frequency?** `ServerToClientMsg::Render { content: String }` could be sent per-frame at 60Hz, which is a LOT of bytes through the socket.
5. **Multi-client conflict resolution.** If two clients send `ChangeMode` simultaneously, what wins? Is there a queue, or is it last-writer?

## State Checkpoint

```yaml
pass: 7
status: complete
timestamp: 2026-05-11T20:30:00Z
phase_a_complete: true
next_phase: B (deepening per category)
```
