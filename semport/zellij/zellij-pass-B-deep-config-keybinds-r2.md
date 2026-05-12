# Phase B Deep — Configuration & Keybinds (Round 2)

## Refinements From Round 1

| Refinement | File:line |
|---|---|
| `Config::from_path` includes human-friendly error-message rewriting for specific `kdl::KdlErrorKind` variants — e.g. `Context("valid node terminator")` becomes a 4-line bullet list of likely causes | `config.rs:227-243` |
| `KdlError` accumulation: when a kdl::KdlError fires, the original config string is attached as `NamedSource` for miette span rendering | `config.rs:243-253` |
| `ConfigError` enum has 9 variants: KdlDeserializationError, KdlError, Std, IoPath, FromUtf8, PluginsError, ConversionError, DownloadError, Async | `config.rs:120-145` |
| `new_layout_kdl_error` is a separate constructor that attaches a layout-guide help URL | `config.rs:155-165` |
| `Config::config_file_path(opts)` ALSO calls `home::try_create_home_config_dir()` to ensure the dir exists before returning | `config.rs:275-285` |

## Confirmed

Round 1 architecture stands: layered priority order, per-sub-aggregate merge, PollWatcher hot-reload, per-client runtime_config overlay.

## Round 2 Status

Refinements are error-handling specifics and a config-dir-creation side effect. No new mechanism. Pass converges.

```yaml
pass: B
category: configuration-and-keybinds
round: 2
status: complete
timestamp: 2026-05-11T21:15:00Z
classification: nitpick
```
