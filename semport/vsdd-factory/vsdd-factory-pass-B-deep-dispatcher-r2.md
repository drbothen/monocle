# Phase B Round 2 — Deep: Factory Dispatcher (NITPICK round)

## Round 1 Gaps Addressed

### Gap 1: What is the resolver sub-system?

Read `resolvers-registry.toml` (18 lines, schema_version=1).

A **resolver** is a WASM plugin that injects pre-computed context into a hook's `plugin_config` before invocation. The mechanism:

1. A `[[hooks]]` entry declares `needs_context = ["wave_context"]`.
2. The dispatcher, before invoking the hook, looks up `wave_context` in `resolvers-registry.toml`.
3. The matching `[[resolvers]]` entry names a WASM module + canonical key.
4. The dispatcher invokes the resolver, reads its output, merges it into the hook's `plugin_config` under the canonical key.

The only resolver currently registered is `wave_context` (`resolvers-registry.toml:13-17`):
- Plugin: `hook-plugins/vsdd-context-resolvers.wasm`
- `context_key`: `wave_context`
- `path_allow`: `.factory/`

This means hooks that declare `needs_context = ["wave_context"]` get wave-state.yaml content automatically merged into their config without doing capability-gated reads themselves.

**Monocle implication**: monocle does not need to model resolvers. They are an internal optimization. The end-user-visible effect is identical to a hook that reads `.factory/wave-state.yaml` directly.

### Gap 2: Are there other observable side-channels besides JSONL sinks?

Per `main.rs:214-222`: the dispatcher writes a single-line trace to **stderr** on every invocation:
```
factory-dispatcher trace=<uuid> event=PreToolUse tool=Bash host_abi=1 sync_plugins=2 async_plugins=1
```

This goes to Claude Code's transcript, which is typically NOT a stable read-path. Monocle should not depend on stderr scraping.

The **internal log** is a parallel JSONL stream at `internal-log/events-*.jsonl` (separate from the user-facing sink). It's always-on and used for dispatcher self-telemetry (registry load errors, schema mismatches). The file sink (`.factory/logs/events-*.jsonl`) is the user-facing primary; the internal log is auxiliary. Monocle should focus on the file sink only.

### Gap 3: Confirm advisory-block-mode

Found inline doc comments in `hooks-registry.toml:882-886`, `:893-897`:

> "advisory-block-mode — block signal via stdout `{"outcome":"block"}` line, not via crash behavior. on_error controls crash semantics only. See `crates/hook-sdk/HOST_ABI.md` 'Advisory block-mode pattern'."

Plugins return a `{"outcome": "<verdict>"}` JSON line on stdout to signal pass/fail/block. The `on_error` field controls what happens if the plugin CRASHES (not what happens on a block verdict).

This matters for monocle: when an event is `plugin.completed` with a `block` outcome, the dispatcher MAY still exit 0 if the plugin was in async_group OR `on_error == continue` for sync. The block verdict's effect on Claude Code (whether the tool was blocked) depends on partition + on_error + verdict combination.

Monocle should surface the plugin outcome **separately** from the dispatcher exit code, because they can disagree.

## Additional Findings

### Registry Stats (from full read of hooks-registry.toml)

| Bucket | Count |
|---|---|
| Total `[[hooks]]` entries | ~56 (per header comment) |
| Native-WASM ports | ~21 |
| Legacy-bash-adapter entries | ~35 |
| Hooks with `async = true` | many (telemetry-only) |
| Hooks with `on_error = "block"` | minority (gates: red-gate, brownfield-discipline, etc.) |
| Hooks declaring `needs_context = ["wave_context"]` | (didn't explicitly enumerate; at least the wave-validate hooks) |

### Hook Naming Convention

All hooks follow pattern `<verb>-<noun>` or `<noun>-<modifier>`:
- `capture-commit-activity`, `capture-pr-activity`
- `validate-artifact-path`, `validate-wave-gate-prerequisite`, `validate-bc-title`, etc.
- `protect-vp`, `protect-bc`, `protect-secrets`
- `warn-pending-wave-gate`
- `session-start-telemetry`, `session-end-telemetry`, `session-learning`
- `worktree-hooks`
- `tool-failure-hooks`
- `red-gate`, `brownfield-discipline`, `factory-branch-guard`
- `track-agent-stop`, `update-wave-state-on-merge`
- `handoff-validator`, `pr-manager-completion-guard`
- `regression-gate`, `destructive-command-guard`, `verify-git-push`

This is informational only; monocle doesn't enforce naming.

### Priority Tier Allocation (observed)

| Range | Typical role |
|---|---|
| 100-199 | Tool-specific validators (Bash, Write, Edit) |
| 200-299 | Convergence + purity + regression checks (Edit/Write hooks) |
| 800-999 | Stop / SubagentStop hooks (session-end, wave-state, handoff validators) |
| no priority | Defaults to 500 |

Priorities allow the same event class to fire multiple ordered checks. Monocle can use the priority to render the "order of checks" if showing hook activity timelines.

### Once-only Sessions Events

`SessionStart` and `SessionEnd` in `hooks.json.template:58-81` carry `"once": true`. This is a Claude Code feature that tells the harness to only fire the hook once per session even if the dispatcher is invoked multiple times.

Monocle: assume `SessionStart` and `SessionEnd` events appear EXACTLY ONCE per session_id in the logs.

## Multi-Factory Abstraction Considerations

For monocle to be **factory-agnostic** (supporting vsdd-factory plus future factories), the dispatcher characterization above generalizes as:

| Concept | vsdd-factory | Monocle abstraction |
|---|---|---|
| Dispatcher | factory-dispatcher (Rust+WASM) | A `dispatcher_binary` per factory; monocle doesn't run it |
| Registry | hooks-registry.toml | A `registry_path` per factory; monocle CAN read for "what hooks are configured" but doesn't need to |
| Event log | .factory/logs/events-*.jsonl | A `event_log_glob` per factory; monocle reads to surface activity |
| State file | .factory/STATE.md | A `state_file_path` per factory; monocle reads + renders |
| Wave state | .factory/wave-state.yaml | A `wave_state_path` per factory (optional) |
| Workflow files | workflows/*.lobster | A `workflow_dir` per factory |
| Manifest | plugin.json `name: vsdd-factory` | A `factory_manifest_id` per factory dispatcher |

Monocle should define a "factory adapter" interface where each factory provides:
1. **Detection signal**: how to recognize this factory in a project (file present? content match?)
2. **State path**: where its STATE.md or equivalent lives
3. **Event log path**: glob for its event stream
4. **Workflow path**: where workflow files live
5. **Display schema**: what to render (mode, phase, step, gate, drift, recent activity)

vsdd-factory becomes the **first concrete adapter** for this interface.

## Delta Summary

- Resolver sub-system characterized (single resolver registered: wave_context).
- Stderr trace line + internal log location identified; both deemed secondary.
- Advisory-block-mode pattern documented.
- Registry tier allocation observed.
- Multi-factory adapter interface sketched.

## Novelty Assessment

Novelty: NITPICK.
Round 2 closed all open gaps. The resolver finding is interesting but irrelevant to monocle's read paths (resolvers are an internal dispatcher optimization). The stderr/internal-log finding confirms monocle should ignore both. The multi-factory abstraction sketch is forward-looking but grounded in current evidence.

## Convergence Declaration

Pass B (dispatcher) has converged — findings are nitpicks, not gaps.

## State Checkpoint

```yaml
pass: B-deep-dispatcher
round: 2
status: complete
timestamp: 2026-05-11T22:55:00Z
novelty: NITPICK
rounds_total: 2
```
