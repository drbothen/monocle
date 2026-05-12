# Phase B Deep — Plugin SDK (Round 2)

## Refinements From Round 1

| Refinement | File:line |
|---|---|
| `wasi_read_string`, `wasi_write_string`, `wasi_read_bytes`, `wasi_write_object` — four host-side I/O helpers for the wasi stdin/stdout pipe | `zellij_exports.rs:5137-5180` |
| `wasi_write_object` is generic over `Serialize` — used to ship response values back to plugin via wasi stdin | `zellij_exports.rs:5161` |
| `host_run_plugin_command` reads request bytes via `wasi_read_bytes(env)`, decodes ProtobufPluginCommand, dispatches | `zellij_exports.rs:161-180` |
| 5,376 LOC of `zellij_exports.rs` is overwhelmingly one giant match + one function per PluginCommand variant | structure confirmed |

## Confirmed

The Round 1 picture stands: single host import function, protobuf-over-stdout, ~120 PluginCommand variants, 17-variant PermissionType, built-in plugins bypass perms, four virtual mount paths, per-(plugin_id, client_id) state.

## Round 2 Status

Refinements at the wasi-helper level — wash-side plumbing for marshaling responses. No new conceptual surface. Pass converges.

```yaml
pass: B
category: plugin-sdk
round: 2
status: complete
timestamp: 2026-05-11T21:15:00Z
classification: nitpick
```
