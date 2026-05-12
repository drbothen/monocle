# Phase B Deep — Session Persistence (Round 2)

## Refinements From Round 1

| Refinement | File:line |
|---|---|
| `SaveSessionToDisk` flow also tracks per-pane "external file" payloads — a `BTreeMap<String, String>` of `(filename → contents)` for things like pane scrollback snapshots | `pty.rs:797-820`, `background_jobs.rs:675-705` |
| `scan_session_list` aggregates other live sessions on the same machine — for cross-session listing (zellij ls) | `background_jobs.rs:708-730` |
| `is_default_shell(default_shell, command_name, args)` does exact command-name match — paths must match byte-for-byte | `session_layout_metadata.rs:200-220` |
| `update_default_shell` mutates the metadata in-place — any pane whose run command matches the default shell has its `run` field set to `None` so the serialized layout doesn't redundantly capture "run the default shell" | `session_layout_metadata.rs:32-60` |
| `populate_session_layout_metadata` is called by the Pty thread to fill in the per-pane CWD and the resolved command path (Pty owns this knowledge) | `pty.rs:774-776` |

## Confirmed

Round 1 architecture stands: 5-thread save chain (Screen→Plugin→Pty→BackgroundJobs→FS), two-file persistence (metadata + layout), is_dirty + file_content_changed gates, directory-traversal-based resurrection enumeration.

## Round 2 Status

Refinements clarify how the metadata flows up the chain (each thread enriches its slice). No new persistence layer. Pass converges.

```yaml
pass: B
category: session-persistence
round: 2
status: complete
timestamp: 2026-05-11T21:15:00Z
classification: nitpick
```
