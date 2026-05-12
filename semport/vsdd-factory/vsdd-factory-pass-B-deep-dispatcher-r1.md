# Phase B Round 1 — Deep: Factory Dispatcher

## Goal

Characterize the factory-dispatcher as if monocle were writing an integration ADR titled "How monocle observes vsdd-factory dispatch state without coupling to it".

## Identity & Lineage

- **Binary name**: `factory-dispatcher` (per-platform variants ending in `.exe` for Windows).
- **Source**: `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/` — Cargo crate inside the vsdd-factory workspace.
- **Cargo package name**: `factory-dispatcher`, lib name `factory_dispatcher`, bin `factory-dispatcher`.
- **Description (from Cargo.toml:9)**: "vsdd-factory v1.0 dispatcher binary — routes Claude Code hook events to WASM plugins"
- **`publish = false`** — not on crates.io.
- **License + repo** inherited from workspace (MIT, drbothen/vsdd-factory).
- **Lib surface** (`src/lib.rs:21-37`): 17 public modules — `aggregator`, `engine`, `executor`, `host`, `internal_log`, `invoke`, `partition`, `payload`, `plugin_loader`, `registry`, `resolver`, `resolver_classify_trap`, `resolver_loader`, `routing`, `sinks`.

## Implementation Language

**Rust**, no unsafe code allowed (`lib.rs:19`: `#![deny(unsafe_code)]`). Single async runtime: tokio `current_thread` flavor (`main.rs:79`). Workspace deps centralized via `Cargo.toml [workspace]` (versions resolved at workspace level).

Key external deps:
- `wasmtime` + `wasmtime-wasi` — WASM host runtime (preview-1 WASI)
- `tokio` — async runtime
- `serde`/`serde_json`/`toml` — registry + payload parsing
- `regex` — tool-name matching
- `anyhow`/`thiserror` — errors
- `uuid` — trace IDs
- `chrono` — timestamps
- `tracing` — internal logging

Internal sink crates (`Cargo.toml:33-38`): `sink-core`, `sink-datadog`, `sink-file`, `sink-honeycomb`, `sink-http`, `sink-otel-grpc`. The file sink is what writes `.factory/logs/events-*.jsonl`.

## Process Lifecycle

The dispatcher is a **short-lived, fork-on-demand process**. Every Claude Code hook event spawns a new `factory-dispatcher` process (no daemon, no IPC, no shared state across calls). Lifecycle:

1. **Boot** (~ms): allocate, build engine, start epoch ticker (`main.rs:233-245`).
2. **Read stdin**: parse `HookPayload` (~ms).
3. **Load registry**: `Registry::load(&registry_path)` from `$CLAUDE_PLUGIN_ROOT/hooks-registry.toml` (`main.rs:98-99`).
4. **Match + group**: filter to relevant plugins, partition sync vs async, group sync by priority (`main.rs:205-212`).
5. **Load resolvers**: `ResolverLoader::load_registry("$CLAUDE_PLUGIN_ROOT/resolvers-registry.toml")` (`main.rs:307-351`).
6. **Sync execution**: `execute_tiers(inputs, sync_tiers).await` — sequential tiers, parallel within tier (`main.rs:366`).
7. **Aggregate exit code**: `aggregate_exit_code(...)` from sync results.
8. **Async dispatch**: spawn async plugins fire-and-forget (`main.rs:368-`).
9. **Drain**: wait up to `ASYNC_DRAIN_WINDOW_MS = 100ms` for async plugins to emit terminal events (`lib.rs:100-105`).
10. **Flush**: in debug builds, dump events to `VSDD_SINK_FILE` for test harness.
11. **Exit**: code 0 (continue) or 2 (fail-closed registry error).

## Transport

**stdin → JSON envelope → routing → WASM plugin invocations → stdout/stderr-line for Claude Code + sink writes.**

Inputs:
- **stdin**: hook envelope JSON. Required fields: `event_name` or `hook_event_name`, `session_id`. Optional: `tool_name`, `tool_input`, `tool_response`, plus arbitrary extras (flattened).
- **`$CLAUDE_PLUGIN_ROOT/hooks-registry.toml`**: plugin registry.
- **`$CLAUDE_PLUGIN_ROOT/resolvers-registry.toml`**: context resolver registry (optional, fail-open if absent per BC-1.13.001 INV2).
- **Environment**: `$CLAUDE_PROJECT_DIR` (project root for capability anchoring), `$VSDD_SINK_FILE` (debug builds only), `$VSDD_ASYNC_DRAIN_WINDOW_MS` (debug-only override).

Outputs:
- **stderr**: human-readable trace line like `factory-dispatcher trace=<uuid> event=PreToolUse tool=Bash host_abi=1 sync_plugins=N async_plugins=M` (`main.rs:214-222`).
- **Sink events** (JSONL): plugin lifecycle events written to whatever sinks are configured (file sink writes `.factory/logs/events-*.jsonl`).
- **Internal log**: always-on, `internal-log/events-*.jsonl` (separate from user-facing sink; for dispatcher self-telemetry).
- **Exit code**: 0 continue, 2 fail-closed.

## Event Types (Stdin Side — Claude Code Events)

Per `hooks/hooks.json.template:3-115`:

| Event | Once-per-session? | Purpose |
|---|---|---|
| `PreToolUse` | no | Before any tool call |
| `PostToolUse` | no | After any tool call |
| `PermissionRequest` | no | Tool permission elevation prompt |
| `Stop` | no | User pressed Stop |
| `SubagentStop` | no | Spawned subagent ended |
| `SessionStart` | yes | New Claude session begins |
| `SessionEnd` | yes | Claude session ends |
| `WorktreeCreate` | no | git worktree created |
| `WorktreeRemove` | no | git worktree removed |
| `PostToolUseFailure` | no | Tool call errored |

All 10 events route to the same binary with a 10-second timeout (claude code's enforced per-hook budget).

## Event Types (Stdout/Sink Side — Plugin Lifecycle)

Per `lib.rs:45-52` (re-exports from `internal_log.rs`):

| Event constant | Meaning |
|---|---|
| `PLUGIN_LOADED` | Plugin module compiled into wasmtime |
| `PLUGIN_INVOKED` | Plugin entry function called |
| `PLUGIN_COMPLETED` | Plugin returned normally |
| `PLUGIN_TIMEOUT` | Plugin exceeded `timeout_ms` |
| `PLUGIN_CRASHED` | Plugin trapped (out of fuel, panic, etc.) |
| `PLUGIN_LOAD_FAILED` | Plugin module failed to load |
| `DISPATCHER_STARTED` | Dispatcher started (per-invocation) |
| `DISPATCHER_SHUTTING_DOWN` | Dispatcher exiting |
| `INTERNAL_DISPATCHER_ERROR` | Generic dispatcher error |
| `INTERNAL_CAPABILITY_DENIED` | Plugin attempted unauthorized capability |
| `INTERNAL_EVENT_FILTERED` | Event filtered before sink write |
| `INTERNAL_EVENT_SCHEMA_VERSION` | Event schema version negotiation |
| `INTERNAL_HOST_FUNCTION_PANIC` | Host fn panicked |
| `INTERNAL_SINK_CIRCUIT_CLOSED` | Sink circuit breaker closed |
| `INTERNAL_SINK_CIRCUIT_OPENED` | Sink circuit breaker opened |
| `INTERNAL_SINK_ERROR` | Sink write error |
| `INTERNAL_SINK_QUEUE_FULL` | Sink queue backpressure |

Plus dispatcher-specific structured events (per `main.rs:21-23`):
- `dispatcher.schema_mismatch` — E-REG-001
- `dispatcher.registry_invalid` — E-REG-002 / E-REG-003
- `plugin.async_block_discarded` — async + on_error=block conflict caught at runtime
- `plugin.timeout` (async path)
- `resolver.registry_loaded`, `resolver.load_warning`, `resolver.load_error` (`main.rs:317-348`)

## Dispatch Logic (the routing decision)

```
INPUT: HookPayload from stdin

1. Match plugins:
   matched = {entry ∈ registry.hooks :
     entry.enabled AND
     entry.event == payload.event_name AND
     (entry.tool is None OR Regex(entry.tool).is_match(payload.tool_name))}

2. Partition:
   sync_group  = {entry ∈ matched : entry.async == false}
   async_group = {entry ∈ matched : entry.async == true}

3. Group sync by priority:
   sync_tiers = [tier_0, tier_1, ..., tier_n]
     where each tier_k has all entries with priority == k_th unique priority value
     sorted ascending (lower priority fires first)
     stable order within tier (registry order)

4. Execute sync tiers sequentially:
   for tier in sync_tiers:
     run all tier entries concurrently
     await all
     aggregate per-tier outcomes
   aggregate exit code:
     if any sync plugin returned Block (and on_error=block): exit 2 / block tool call
     else: exit 0 / continue

5. Spawn async group:
   for entry in async_group:
     tokio::spawn(invoke_plugin(entry, payload))
   tokio::select! with timeout ASYNC_DRAIN_WINDOW_MS:
     - all complete: drain success
     - timeout: terminate pending, emit plugin.async_block_discarded for any block

6. Flush sinks, write events, exit.
```

## Capability Model (deny-by-default)

Per `registry.rs:80-141`:

```toml
[hooks.capabilities]
env_allow = ["PATH", "HOME", "TMPDIR", "CLAUDE_PROJECT_DIR", "CLAUDE_PLUGIN_ROOT", "VSDD_SESSION_ID"]

[hooks.capabilities.exec_subprocess]
binary_allow = ["bash", "jq"]                 # whitelist
shell_bypass_acknowledged = "..."             # opt-in flag for shell interpreters
cwd_allow = []                                # default empty = no cwd override permitted
env_allow = ["PATH", ...]

[hooks.capabilities.read_file]
path_allow = [".factory/regression-state.json"]  # rooted at CLAUDE_PROJECT_DIR

[hooks.capabilities.write_file]
path_allow = [".factory/regression-state.json"]
max_bytes_per_call = 65536
```

**Always-on host functions (no capability needed)**: `log`, `emit_event`, `session_id`, etc.

**Conditional host functions**: `read_file`, `write_file`, `exec_subprocess`, `env_var` — require explicit `[hooks.capabilities.<kind>]` block.

Missing block → all calls return `CAPABILITY_DENIED (-1)`.

## hooks-registry.toml Field-by-Field

| Field | Default | Notes |
|---|---|---|
| `schema_version` | (required) | Must be `2`. E-REG-001 if not. |
| `[defaults]` block | implicit defaults | timeout_ms=5000, fuel_cap=10_000_000, on_error=continue, priority=500 |
| `[[hooks]].name` | (required) | Unique within registry |
| `[[hooks]].event` | (required) | Claude Code event name |
| `[[hooks]].tool` | None | Regex; None matches all tools |
| `[[hooks]].plugin` | (required) | Path to .wasm relative to plugin root |
| `[[hooks]].priority` | inherits 500 | Lower fires first |
| `[[hooks]].enabled` | true | `false` skips entry |
| `[[hooks]].timeout_ms` | inherits 5000 | Per-call wall-clock |
| `[[hooks]].fuel_cap` | inherits 10_000_000 | wasmtime fuel |
| `[[hooks]].on_error` | inherits continue | `continue` or `block` |
| `[[hooks]].async` | false | TOML key `async`; if true, fire-and-forget |
| `[[hooks]].needs_context` | `[]` | Resolver names for context injection |
| `[[hooks]].capabilities` | None | Deny-by-default capability decl |
| `[[hooks]].config` | empty table | Plugin-defined config (legacy-bash-adapter uses `script_path`) |

## Legacy Bash Hook Adapter

Per `hooks-registry.toml:1-9`: 35 entries use `plugin = "hook-plugins/legacy-bash-adapter.wasm"` with `[hooks.config] script_path = "hooks/<name>.sh"`. The adapter is a single WASM module that exec()s the underlying bash script. This is how 27 bash hooks in `hooks/*.sh` get wired into the dispatcher.

This is a transition mechanism — 21 native-WASM ports coexist with the bash adapter as of this version.

## Monocle Integration Implications

| Question | Answer |
|---|---|
| Should monocle invoke the dispatcher? | NO — dispatcher is for Claude Code, not user-space tools. |
| Should monocle read `hooks-registry.toml`? | OPTIONAL — useful for understanding "which hooks could fire", but not required for state surfacing. |
| Should monocle read `.factory/logs/events-*.jsonl`? | YES — this is the canonical "recent hook activity" feed. |
| Should monocle filter events by `session_id`? | YES, when the user wants a session-scoped view. The dispatcher emits `session_id` on every event. |
| Should monocle understand WASM/wasmtime internals? | NO — the binary boundary is the sink output. |
| Should monocle know about `dispatcher_trace_id`? | OPTIONAL — useful for correlating multi-plugin events from one tool call. |

## Failure Modes Worth Surfacing

| Event | Meaning for monocle |
|---|---|
| `dispatcher.schema_mismatch` | hooks-registry.toml has wrong schema_version — dispatcher refused to start; ALL hooks bypassed |
| `dispatcher.registry_invalid` | E-REG-002 or E-REG-003; registry has structural violation |
| `plugin.timeout` | A plugin exceeded its timeout — partial enforcement |
| `plugin.crashed` | A plugin trapped — partial enforcement |
| `plugin.async_block_discarded` | A plugin tried to block from async — verdict dropped, BUG-equivalent |
| `internal.sink_circuit_opened` | Telemetry sink is broken; events may not be reaching observability backend |

Monocle should surface these as system-health signals, separate from pipeline progress.

## Delta Summary

- Dispatcher characterized as: Rust + wasmtime, short-lived per-invocation, stdin/JSON in / sink-JSONL out, exit 0/2.
- Public API surface: 17 modules, 30+ re-exports.
- Event types: 10 Claude events in, 17+ plugin/dispatcher events out.
- Registry contract: 14 plugin-entry fields, 4 capability sub-blocks.
- Failure modes: 6 named events monocle should recognize.
- Constants: `HOST_ABI_VERSION=1`, `ASYNC_DRAIN_WINDOW_MS=100ms`, `EPOCH_TICK_MS=10`, default `priority=500`, default `timeout_ms=5000`, default `fuel_cap=10_000_000`.

## Novelty Assessment

Novelty: SUBSTANTIVE.
This is the first full pass on the dispatcher. Key new findings vs Pass 1/2: ABI version constant, drain window constant, the 17-module library surface, the resolver-registry (sibling to hooks-registry), and the full plugin-lifecycle event taxonomy.

## Convergence Declaration

Another round needed — verify the resolver sub-system, confirm whether monocle needs to know anything about it, and check that there are no other observable side-channels (other than the JSONL sink).

## State Checkpoint

```yaml
pass: B-deep-dispatcher
round: 1
status: complete
timestamp: 2026-05-11T22:50:00Z
novelty: SUBSTANTIVE
```
