# Phase B Deep — IPC (Round 2)

## Refinements From Round 1

| Refinement | File:line |
|---|---|
| Socket transport implementation: `ipc_bind`, `ipc_bind_async`, `ipc_connect`, `ipc_connect_reply`, `ipc_bind_reply` — five distinct helpers | `consts.rs:195-300` |
| Unix uses `interprocess::local_socket::GenericFilePath` (file-system-rooted Unix socket); Windows uses `GenericNamespaced` (named pipe) | `consts.rs:200-217` |
| Windows `ipc_bind` ALSO writes a marker file containing `std::process::id()` — used by `sessions::assert_socket` for Windows liveness probing | `consts.rs:230-238` |
| Async listener (`ipc_bind_async`) uses `create_tokio()` instead of `create_sync()` | `consts.rs:242-260` |
| `ZELLIJ_SOCK_DIR` is computed differently on Linux vs other Unix vs Windows — three `lazy_static!` branches | `consts.rs:316-380` |

## Confirmed

The Round 1 IPC catalog (20 ClientToServerMsg + 13-16 ServerToClientMsg) is complete; protobuf wire format and length-prefix framing are accurately documented. `route` thread role and `NotificationEnd` drop-pattern stand. `SessionConfiguration` overlay model stands.

## Round 2 Status

These are implementation-detail refinements (transport plumbing, OS-specific helpers). No new conceptual layer. Pass converges.

```yaml
pass: B
category: ipc
round: 2
status: complete
timestamp: 2026-05-11T21:15:00Z
classification: nitpick
```
