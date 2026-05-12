# vsdd-factory — Pass 8 Final Synthesis (Scoped Ingest)

## Summary

vsdd-factory is a Claude Code plugin that installs the Verified Spec-Driven Development (VSDD) pipeline — a "dark factory" autonomous software pipeline that drives projects through eight phases (codebase ingestion, spec crystallization, story decomposition, TDD implementation, holdout evaluation, adversarial refinement, formal hardening, convergence). The system is structured as three independently-versioned layers: a static data layer of `.lobster` YAML workflow files plus `.factory/STATE.md` state, a driver layer of one orchestrator agent plus seven workflow-execution skills, and an enforcement layer comprising a native Rust + wasmtime `factory-dispatcher` binary routing Claude Code hook events to WASM plugins. Monocle's purpose is to surface this state to humans, not to run any of it; monocle observes the data layer directly, can read-only the driver layer's outputs, and ingests the enforcement layer only through the JSONL event log it produces. The workflow-awareness scope means monocle must understand the `.lobster` schema, the dispatcher's event surface, the factory-project discriminator, the STATE.md schema, and the small set of read paths the existing workflow-execution skills use — nothing more. The deliberately decoupled discriminator (`document_type: pipeline-state` in STATE.md frontmatter) lets monocle support vsdd-factory as the first concrete adapter of a generic factory-adapter interface so future factories drop in without monocle changing. Everything outside that scope — 33+ specialist agents, 116 skills, rules, fixtures, 520+ bats tests, hook-plugin internals, extended methodology docs, and the 27 enforcement bash hooks — is deliberately left behind in this ingest because monocle does not execute or port the engine itself.

## Scope Statement

This is a SCOPED ingest. It exists to give monocle exactly the genes it needs to be workflow-aware about projects driven by vsdd-factory (and future compatible factories) without taking ownership of any of the factory's execution responsibilities.

**In-scope (deepened to NITPICK):**

- The `.lobster` workflow file format (YAML schema, step types, conditional expressions, sub-workflow composition).
- The `factory-dispatcher` binary surface monocle observes (stdin envelope, exit codes, JSONL event types, the registry contract enough to read it, never to invoke it).
- The factory-project discriminator (how to detect a vsdd-factory or compatible factory project on disk).
- The `STATE.md` schema (frontmatter fields, body sections, size budget, mutation discipline).
- The workflow-execution skills monocle MIRRORS in read paths only: `run-phase`, `next-step`, `validate-workflow`, `factory-dashboard`, `recover-state`, `check-state-health`, `state-update` (per pass-1-project-discovery.md Category 4).

**Out-of-scope (deliberately not deepened):**

- `agents/` directory (33+ specialist agents; only `orchestrator/orchestrator.md` was read for context).
- `skills/` directory (116 total; only the 7 workflow-execution skills above were read).
- `rules/` (9 orchestrator rule files), `fixtures/`, `tests/` (520+ bats tests), `docs/` (extended methodology).
- `hook-plugins/` (28+ Rust crates with individual plugin internals).
- `crates/` other than `factory-dispatcher/` (12 other crates including the sink-* family, hook-sdk, hook-sdk-macros, vsdd-context-resolvers).
- `plugins/vsdd-factory/templates/` other than the 6 STATE/cycle templates.
- `plugins/vsdd-factory/hooks/*.sh` (27 enforcement bash hooks; only the registry entries summarized, no per-hook deep-dive).

**Rationale:** monocle observes; it does not execute. The skills, agents, hook scripts, fixtures, and tests are owned by vsdd-factory and never run inside monocle. Porting them would invert the architecture monocle is meant to deliver.

## Snapshot

| Field | Value |
|-------|-------|
| Repo path | `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/` |
| Repo | `drbothen/vsdd-factory` |
| Branch | `develop` |
| HEAD | `99d2431529fa2839bc2b04702ad32404d06d0f99` |
| Plugin manifest version | `1.0.0-rc.16` |
| License | MIT |
| Total tree size | 232 MB on disk, 1,572 files |
| In-scope file count | ~46 files |
| In-scope LOC | ~13,205 (per pass-1-project-discovery.md LOC roll-up) |
| Lobster workflow LOC | ~6,609 across 16 `.lobster` files |
| Dispatcher Rust source LOC | 3,350 across 6 source files |
| hooks-registry.toml + hooks.json.template LOC | 1,112 |
| In-scope skill SKILL.md LOC (7 skills) | 588 |
| Orchestrator agent LOC | 437 |
| STATE/cycle templates LOC (7 files) | 670 |
| Top languages | Rust 2.6 MB, Shell ~1 MB, JavaScript 33 KB, PowerShell 17 KB, HTML 8 KB, Python 4 KB, TypeScript 3 KB |

Per pass-1-project-discovery.md §"Repo Identification" and §"In-Scope LOC Roll-up".

## Lobster File Format Schema

`.lobster` files are standard YAML, validated via `bin/lobster-parse` (a 51-line bash wrapper over `yq | jq` — per pass-4-behavioral-contracts.md BC-MON-001 and pass-B-deep-workflows-r1.md §"Parser Implementation"). Monocle should use a native YAML library; the bash wrapper is a convenience for orchestrator agents, not a required dependency.

### Top-level shape

```yaml
workflow:
  name:        <kebab-case-string>     # REQUIRED, unique across all workflows
  description: <string-folded>         # REQUIRED, arbitrary prose
  version:     <semver-string>         # REQUIRED, e.g. "3.0.0"
  defaults:    <Defaults>              # optional
  cost_monitoring: <CostBlock>         # optional (most workflows)
  cost_tracking:   <CostBlock>         # optional (feature.lobster spelling)
  schedule:    <Schedule>              # optional (discovery.lobster only)
  state_fields: <list[string]>         # optional (discovery)
  inputs:      <list[InputDecl]>       # optional (sub-workflows)
  steps:       <list[Step]>            # REQUIRED
```

Per pass-B-deep-workflows-r1.md §"Full Lobster File Schema" and §"Top-Level Fields".

### Defaults block

```yaml
defaults:
  on_failure:  escalate | retry | skip   # default escalate
  max_retries: <int>                     # default 2
  timeout:     <duration-string>         # default "2h" (some workflows "1h"/"4h")
```

Evidence: brownfield.lobster:19-22, feature.lobster:56-59, code-delivery.lobster:28-31, planning.lobster:16-19 (per pass-B-deep-workflows-r1.md).

### Cost block (dual spelling — monocle accepts both)

```yaml
cost_monitoring:        # OR cost_tracking
  enabled: true|false
  metadata: {mode: <string>, feature: <string>, phase: <string>, wave: <string>}
  thresholds: {warn: <float>, pause: <float>}
  protected_agents: [adversary, security-reviewer, formal-verifier, ...]
  summary_artifact: <path>
```

Per pass-6-conventions.md §"P5. Cost Monitoring Inline" and pass-B-deep-workflows-r3.md §"EC-5. Cost block fields are inconsistent". feature.lobster uses `cost_tracking`; greenfield/discovery/multi-repo/maintenance use `cost_monitoring`.

### Step object — all fields

| Field | Type | Required when | Notes |
|---|---|---|---|
| `name` | string | always | unique within workflow |
| `type` | StepType enum | always | see enum below |
| `depends_on` | list[step-name] | always (may be `[]` OR omitted) | empty/missing both mean root |
| `agent` | string | type=agent | agent ID, kebab-case |
| `skill` | path string | type=skill | e.g. `skills/run-phase/SKILL.md` |
| `sub_workflow` | path string | type=sub-workflow | e.g. `greenfield.lobster` |
| `task` | string-folded | agent steps; optional on skill | natural-language prompt |
| `condition` | string-expr | optional | surface verbatim, do NOT evaluate |
| `optional` | bool | optional | default false; true = skip-if-fails |
| `timeout` | duration | optional | overrides defaults.timeout |
| `on_failure` | enum | optional | inherits defaults |
| `max_retries` | int | optional | inherits defaults |
| `wait_for_optional` | list[step-name] | optional | soft predecessors |
| `model_tier` | string | optional | `review`, `adversary` |
| `description` | string | optional | secondary description |
| `config` | object | optional | step-specific (e.g. `{depth: "L1"}`) |
| `cwd` | path | optional | per-step working dir (multi-repo) |
| `gate` | GateSpec | type=gate | criteria + fail_action |
| `approval` | ApprovalSpec | type=human-approval | prompt + timeout + artifacts |
| `loop` | LoopSpec | type=loop | max_iterations + exit_condition + steps |
| `collection` | string-expr | type=parallel-foreach | iterable expression |
| `iterator` | list[Step] | type=parallel-foreach | per-item sub-steps |
| `inputs` | object | optional | sub-workflow input bindings |
| `context` | ContextSpec | optional | information-asymmetry wall |

Per pass-2-architecture.md §"Step schema" + pass-B-deep-workflows-r1.md §"Step Object" + pass-B-deep-workflows-r2.md §"Round 1 Gaps Addressed" (Gap 2 clarified the dual-shape `inputs` mechanism).

### StepType enum

| Value | Required companion fields | Usage frequency |
|---|---|---|
| `agent` | `agent`, `task` | ~60% of steps |
| `skill` | `skill` | ~25% |
| `gate` | `gate.criteria`, `gate.fail_action` | ~8-10% |
| `human-approval` | `approval.prompt`, `approval.timeout` | ~5% |
| `sub-workflow` | `sub_workflow` | ~2% |
| `loop` | `loop.max_iterations`, `loop.exit_condition`, `loop.steps` | rare |
| `parallel-foreach` | `collection`, `iterator` | rare (multi-repo) |
| `command` | (shell command) | declared in run-phase/SKILL.md:46 but UNUSED in any in-tree workflow per pass-B-deep-workflows-r2.md Gap 1 |

### Examples table (sub-structures)

| Sub-structure | Required fields | Evidence |
|---|---|---|
| GateSpec | `criteria` (list of assertion strings), `fail_action` (block or warn) | brownfield.lobster:151-167, feature.lobster:94-98, phase-7-convergence.lobster:117-130 |
| ApprovalSpec | `prompt`, `timeout` (typical 24h/48h/72h); optional `artifacts` glob list | brownfield.lobster:170-189, planning.lobster:101-112 |
| LoopSpec | `max_iterations`, `exit_condition`, `steps`; optional `for_each` | code-delivery.lobster:103-145 (per-story adversarial review loop, max 10) |
| ContextSpec | `include` glob list, `exclude` glob list — the information-asymmetry wall | code-delivery.lobster:118-135 (the canonical wall site) |

### Convergence loop template (canonical pattern)

Per pass-B-deep-workflows-r2.md §"O-R2-1. Phase 1 and Phase 2 share an identical adversarial-loop pattern", phases 1, 2, and 5 use a near-identical convergence loop:

- Outer step: `type: loop`, `max_iterations: 10`, `exit_condition: "adversary.verdict == 'CONVERGENCE_REACHED'"`.
- Inner steps: `spawn-adversary-<scope>` (agent: adversary, model_tier: adversary) plus `fix-<scope>-findings` (skill).

Monocle UI should label this as "adversarial pass M of 10" rather than the raw step name.

### Conditional expression vocabulary

There is NO formal grammar for `condition:` values. They are natural-looking boolean expressions. Common shapes (per pass-B-deep-workflows-r1.md §"Conditional Expression Vocabulary"):

- File existence: `!file_exists('CLAUDE.md')`, `exists('.factory/...')` (note inconsistency between the two spellings).
- Routing lookups: `routing.choice == 'feature'`, `routing.level in ['L1','L2']`.
- State lookups: `state.has_holdout_scenarios == true`, `state.has_ui == true`.
- Convergence checks: `adversary.verdict == 'CONVERGENCE_REACHED'`, `adversary.has_blocking_findings == true`.
- Counts: `discovery.approved_products.count > 0`.

Monocle MUST surface verbatim as labels and MUST NOT attempt to evaluate.

## Factory Dispatcher

The dispatcher is a short-lived native Rust + wasmtime binary that Claude Code spawns once per hook event. Monocle observes it ONLY through its JSONL sink output; monocle never invokes it, never reads its stderr, never parses its internal log.

### Identity

| Field | Value | Source |
|---|---|---|
| Source language | Rust, no unsafe (`#![deny(unsafe_code)]`) | pass-B-deep-dispatcher-r1.md §"Implementation Language" |
| Source path | `crates/factory-dispatcher/` | 6 source files in `src/` totaling 3,350 LOC |
| Crate package | `factory-dispatcher` | `publish = false` |
| Lib surface | 17 public modules per `src/lib.rs:21-37` | aggregator, engine, executor, host, internal_log, invoke, partition, payload, plugin_loader, registry, resolver, resolver_classify_trap, resolver_loader, routing, sinks |
| Async runtime | tokio current_thread flavor | `main.rs:79` |
| WASM runtime | wasmtime + wasmtime-wasi (preview-1 WASI) | epoch-interruption + fuel |

### Binary distribution

| Platform | Binary path |
|---|---|
| darwin-arm64 | `plugins/vsdd-factory/hooks/dispatcher/bin/darwin-arm64/factory-dispatcher` |
| darwin-x64 | `plugins/vsdd-factory/hooks/dispatcher/bin/darwin-x64/factory-dispatcher` |
| linux-arm64 | `plugins/vsdd-factory/hooks/dispatcher/bin/linux-arm64/factory-dispatcher` |
| linux-x64 | `plugins/vsdd-factory/hooks/dispatcher/bin/linux-x64/factory-dispatcher` |
| windows-x64 | `plugins/vsdd-factory/hooks/dispatcher/bin/windows-x64/factory-dispatcher.exe` |

macOS arm64 binary is ~12 MB Mach-O 64-bit executable (verified via `file` per pass-2-architecture.md §C2).

### Lifecycle and transport

Per pass-B-deep-dispatcher-r1.md §"Process Lifecycle" and §"Transport":

1. Boot (~100 ms allocation + engine build + epoch ticker start).
2. Read JSON envelope from stdin (`HookPayload::from_reader`, payload.rs:77-81).
3. Load `$CLAUDE_PLUGIN_ROOT/hooks-registry.toml`.
4. Match plugins by `event_name` + `tool_name` regex.
5. Partition sync vs async; group sync by priority (lower fires first; same-priority parallel).
6. Execute sync tiers sequentially; spawn async fire-and-forget.
7. Drain async up to `ASYNC_DRAIN_WINDOW_MS` (100ms).
8. Emit lifecycle events to sinks (`.factory/logs/events-*.jsonl` for file sink).
9. Exit 0 (continue) or 2 (fail-closed registry error).

Stdin envelope shape (per pass-2-architecture.md §C2 "Stdin payload shape" and payload.rs:11-54):

```json
{
  "event_name|hook_event_name": "<ClaudeCodeEvent>",
  "tool_name": "<ToolName>",
  "session_id": "<stable-per-session-id>",
  "tool_input":    { ... },
  "tool_response": { ... },
  /* extras flattened: agent_type, subagent_name, last_assistant_message, etc. */
}
```

### Claude Code event types handled (10)

Per `hooks/hooks.json.template:3-115` (cited in pass-2-architecture.md §C2 and pass-4-behavioral-contracts.md BC-MON-011):

| Event | Once-per-session | Purpose |
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

All 10 events route to the same binary with a 10-second timeout.

### Capability model

Deny-by-default per pass-4-behavioral-contracts.md BC-MON-013 and pass-B-deep-dispatcher-r1.md §"Capability Model". A plugin without an explicit `[hooks.capabilities]` block has only the always-on host functions (`log`, `emit_event`, `session_id`). Read, write, exec, and env-var access require explicit allowlists:

- `path_allow` lists rooted at `$CLAUDE_PROJECT_DIR` for read_file/write_file.
- `binary_allow` lists for exec_subprocess.
- `shell_bypass_acknowledged` opt-in flag required for shell interpreters.
- `max_bytes_per_call` ceiling for write_file (typical 65536).

### Exit codes

| Code | Meaning | Triggers |
|---|---|---|
| 0 | Continue | Default; even on fail-open registry errors |
| 2 | Fail-closed | E-REG-001 schema_version mismatch; E-REG-002 async + on_error=block conflict; E-REG-003 duplicate (name, event, tool) tuple |

Per pass-4-behavioral-contracts.md BC-MON-020 + main.rs:80-92 + main.rs:131-149. Plugin verdicts in sync_group can additionally produce a "block" exit code aggregated via `aggregate_exit_code` (main.rs:60).

### Constants and defaults

| Constant | Value | Source |
|---|---|---|
| `HOST_ABI_VERSION` | 1 | pass-B-deep-dispatcher-r1.md §"Delta Summary" |
| `ASYNC_DRAIN_WINDOW_MS` | 100 ms | lib.rs:100-105 |
| `EPOCH_TICK_MS` | 10 ms | engine.rs:22-23 |
| Default `priority` | 500 | registry.rs:153-161 |
| Default `timeout_ms` | 5000 | registry.rs:153-161 |
| Default `fuel_cap` | 10_000_000 | registry.rs:153-161 |
| Default `on_error` | continue | registry.rs:153-161 |

### Event types emitted (monocle's read surface)

Per pass-B-deep-dispatcher-r1.md §"Event Types (Stdout/Sink Side)" + lib.rs:45-52:

**Plugin lifecycle:** `PLUGIN_LOADED`, `PLUGIN_INVOKED`, `PLUGIN_COMPLETED`, `PLUGIN_TIMEOUT`, `PLUGIN_CRASHED`, `PLUGIN_LOAD_FAILED`.

**Dispatcher lifecycle:** `DISPATCHER_STARTED`, `DISPATCHER_SHUTTING_DOWN`.

**Internal health:** `INTERNAL_DISPATCHER_ERROR`, `INTERNAL_CAPABILITY_DENIED`, `INTERNAL_EVENT_FILTERED`, `INTERNAL_EVENT_SCHEMA_VERSION`, `INTERNAL_HOST_FUNCTION_PANIC`, `INTERNAL_SINK_CIRCUIT_CLOSED`, `INTERNAL_SINK_CIRCUIT_OPENED`, `INTERNAL_SINK_ERROR`, `INTERNAL_SINK_QUEUE_FULL`.

**Dispatcher-specific structured:** `dispatcher.schema_mismatch` (E-REG-001), `dispatcher.registry_invalid` (E-REG-002/003), `plugin.async_block_discarded`, `plugin.timeout`, `resolver.registry_loaded`, `resolver.load_warning`, `resolver.load_error`.

Plugin verdicts are signalled via stdout `{"outcome": "<verdict>"}` per pass-B-deep-dispatcher-r2.md §"Gap 3: Confirm advisory-block-mode". Monocle should surface plugin outcome separately from dispatcher exit code because they can disagree.

### hooks-registry.toml schema

| Field | Default | Notes |
|---|---|---|
| `schema_version` | required | MUST be 2; E-REG-001 fail-closed |
| `[defaults]` | implicit | timeout_ms=5000, fuel_cap=10_000_000, on_error=continue, priority=500 |
| `[[hooks]].name` | required | unique within registry |
| `[[hooks]].event` | required | Claude Code event name |
| `[[hooks]].tool` | None | optional regex; None matches all tools |
| `[[hooks]].plugin` | required | path to .wasm relative to plugin root |
| `[[hooks]].priority` | 500 | lower fires first |
| `[[hooks]].enabled` | true | false skips |
| `[[hooks]].timeout_ms` | 5000 | per-call wall-clock |
| `[[hooks]].fuel_cap` | 10_000_000 | wasmtime fuel |
| `[[hooks]].on_error` | continue | continue or block |
| `[[hooks]].async` | false | TOML key `async`; if true, fire-and-forget |
| `[[hooks]].needs_context` | `[]` | resolver names for context injection |
| `[[hooks]].capabilities` | None | deny-by-default capability declaration |
| `[[hooks]].config` | empty | plugin-defined config (legacy-bash-adapter uses `script_path`) |

Per pass-B-deep-dispatcher-r1.md §"hooks-registry.toml Field-by-Field" and pass-2-architecture.md §C3 §"Hard invariants".

Live registry stats (per pass-B-deep-dispatcher-r2.md §"Registry Stats"): hooks-registry.toml is 996 lines, ~56 total entries, split as ~21 native-WASM ports plus ~35 legacy-bash-adapter entries that wrap unported bash hooks via `plugin = "hook-plugins/legacy-bash-adapter.wasm"` with `[hooks.config] script_path = "hooks/<name>.sh"`.

## Factory Project Discriminator

### Canonical signal

`<project>/.factory/STATE.md` exists AND its YAML frontmatter has `document_type: pipeline-state`. Per pass-B-deep-factory-pattern-r1.md §"Signal 1" + `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/state-template.md:1-17` + pass-4-behavioral-contracts.md BC-MON-022.

The choice of `document_type: pipeline-state` (rather than `factory: vsdd-factory`) is deliberate decoupling — it lets future factory dispatchers adopt the same discriminator without inheriting the vsdd-factory name. This is the load-bearing finding of the factory-pattern deepening.

### Multi-repo signal

`<project>/.factory-project/STATE.md` exists at the workspace root. The multi-repo template uses a body table for `project_name/project_type/pipeline/phase` rather than a YAML frontmatter `document_type`, so multi-repo detection falls back to directory-name presence as a sufficient signal (per pass-B-deep-factory-pattern-r1.md §"Signal 2" NOTE).

Per pass-4-behavioral-contracts.md BC-MON-017, multi-repo projects have BOTH a project-level `.factory-project/` worktree (on `factory-project-artifacts` branch) AND per-repo `.factory/` worktrees (each on its own `factory-artifacts` branch).

### Detection algorithm

Per pass-B-deep-factory-pattern-r1.md §"Recommended Discriminator Algorithm":

```python
def detect_factory(project_root):
    state_md   = Path(project_root) / ".factory" / "STATE.md"
    project_md = Path(project_root) / ".factory-project" / "STATE.md"

    if project_md.exists():
        return "vsdd-multi"

    if state_md.exists():
        fm = parse_yaml_frontmatter(state_md)
        if fm.get("document_type") == "pipeline-state":
            producer = fm.get("producer", "")
            if producer == "state-manager":
                return "vsdd"
            return "unknown"   # forward-compatible factory
        return "unknown"

    return None  # not a factory project
```

### Future-factory compatibility

Future factories that adopt `document_type: pipeline-state` become discoverable by monocle without monocle code changes. The same algorithm classifies them as `"unknown"` (factory-shaped, non-vsdd dispatcher), which is enough for monocle to render generic phase/step views. This satisfies holdout H10 (per pass-7-holdout-seeds.md §H10).

### Discrimination confidence matrix

| Signals present | Detection | Confidence |
|---|---|---|
| `.factory/STATE.md` with `document_type: pipeline-state` AND `producer: state-manager` | vsdd-factory | HIGH |
| `.factory-project/STATE.md` + per-repo `.factory/STATE.md` | vsdd-factory multi-repo | HIGH |
| `.factory/STATE.md` without `document_type` | factory-shaped, unknown dispatcher | LOW |
| `.factory/` exists but no STATE.md | factory-shaped, uninitialized | LOW |
| `.factory/wave-state.yaml` only | factory project mid-Phase-2+ | MEDIUM |
| None of the above | not a factory project | HIGH (negative) |

Per pass-B-deep-factory-pattern-r1.md §"Discrimination Confidence Matrix".

## STATE.md Schema

### Frontmatter (REQUIRED fields)

Per `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/state-template.md:1-17` and pass-3-domain-model.md §E7 + pass-4-behavioral-contracts.md BC-MON-015:

| Field | Type | Meaning |
|---|---|---|
| `document_type` | string literal `pipeline-state` | The discriminator |
| `level` | string | typically `ops` |
| `version` | semver string | artifact version, NOT project version |
| `status` | string | `draft`, `in_progress`, `complete`, `blocked` |
| `producer` | string | agent ID (vsdd-factory: `state-manager`) |
| `timestamp` | ISO8601 | last update |
| `phase` | integer 0-7 | pipeline phase number |
| `inputs` | list | source artifacts |
| `input-hash` | string | MD5 of inputs (drift detection) |
| `traces_to` | string | parent artifact reference |
| `project` | string | project name |
| `mode` | string enum | `greenfield`, `brownfield`, `feature`, `maintenance`, `discovery`, `multi-repo` |
| `current_step` | string | current step name (unqualified) |
| `current_cycle` | string | active cycle ID |
| `dtu_required` | bool | DTU clones required |

### Pipeline field values (free-form enum)

Per pass-3-domain-model.md §"STATE.md pipeline: Field" and skills/state-update/SKILL.md:65-75:

`INITIALIZED, STARTED, RUNNING, FEATURE-CYCLE, PAUSED, BLOCKED, COMPLETE, COMPLETED`.

Monocle MUST surface whatever value is found verbatim rather than enforcing a canonical set. Future factories may introduce new values.

### Body sections (REQUIRED)

Per the STATE.md template and pass-2-architecture.md §"STATE.md size discipline":

1. Project Metadata
2. Phase Progress (8 rows, phases 0-7)
3. Current Phase Steps (keep last 5)
4. Decisions Log
5. Skip Log
6. Blocking Issues
7. Session Resume Checkpoint (only latest)
8. Historical Content (overflow → cycles/)

### Size budget

Per `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/state-template.md:21-22` + pass-4-behavioral-contracts.md BC-MON-007:

- 200 lines: warn (`validate-state-size.sh`).
- 500 lines: block.
- Overflow routed via `/vsdd-factory:compact-state`.

### Mutation discipline (monocle MUST NOT write)

Per pass-4-behavioral-contracts.md BC-MON-008/009 + state-manager-checklist-template.md:

- SINGLE-COMMIT BURST on `factory-artifacts` branch (the retired Two-Commit Protocol is forbidden).
- Past-tense voice ("REMEDIATED") from burst start.
- No factory-artifacts HEAD SHA self-cites (TD-VSDD-053).
- `MULTI_COMMIT_CHAIN_NOT_ALLOWED` guard if two consecutive commits contain "backfill" (per `verify-sha-currency.sh`).

**Monocle never writes STATE.md.** Only the `state-manager` agent mutates it. Monocle reads from the working tree (no need to switch branches; `.factory/` is a git worktree on the orphan `factory-artifacts` branch per BC-MON-016).

### Phase value drift

Real failure modes observed (per pass-7-holdout-seeds.md §H13 + check-state-health/SKILL.md:55-63):

- `phase: 3.5` (non-integer).
- `phase: "2-story-decomposition-patch-cycle"` (compound string).

Monocle SHOULD surface raw value AND flag drift. Do NOT fail-closed on unknown phase values.

## Cycle Structure

A cycle is one pass through the pipeline (or a coherent multi-phase run), archived under `.factory/cycles/<cycle-id>/`. Per pass-3-domain-model.md §E5 + pass-4-behavioral-contracts.md BC-MON-018 + `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/cycle-manifest-template.md:1-59`.

### Identity

`cycle_id` format: `vX.Y.Z-<type>-<name>` (e.g. `v1.0.0-feature-monocle-foundations`). Cycle types: `greenfield`, `feature`, `bugfix`, `deprecation`, `refactor`.

### Directory layout

```
.factory/cycles/<cycle-id>/
├── cycle-manifest.md
├── burst-log.md
├── convergence-trajectory.md
├── session-checkpoints.md
├── lessons.md
├── blocking-issues-resolved.md
├── adversarial-reviews/
│   ├── pass-N.md
│   └── ADV-PN-INDEX.md
├── implementation/
│   ├── implementer-notes*.md
│   └── red-gate-log*.md
└── wave-schedule.md
```

### Fields monocle surfaces

| Field | Source |
|---|---|
| `cycle_id` | STATE.md frontmatter `current_cycle` |
| `cycle_type` | `cycle-manifest.md` frontmatter |
| `status` | `cycle-manifest.md` frontmatter |
| `started` | `cycle-manifest.md` frontmatter |
| Latest burst entry | tail of `burst-log.md` |
| Convergence trajectory shorthand | parse `convergence-trajectory.md` (e.g. 29→24→21→7→4→3) |
| Blocking issues | `blocking-issues-resolved.md` minus resolved set |

## Multi-Factory Adapter Manifest (Proposed)

For monocle to support vsdd-factory plus future factories, the architecture defines a factory-adapter manifest. vsdd-factory becomes the first concrete adapter; monocle's core stays factory-agnostic. Per pass-B-deep-dispatcher-r2.md §"Multi-Factory Abstraction Considerations" and pass-B-deep-factory-pattern-r1.md §"Multi-Factory Manifest Sketch".

```yaml
# monocle factory-adapter manifest (proposed)
factory_adapter:
  name: vsdd-factory
  version: "1.0"
  description: "Verified Spec-Driven Development factory"

# Detection — how monocle recognizes this factory
detection:
  required:
    - path: ".factory/STATE.md"
      frontmatter:
        document_type: "pipeline-state"
    # OR for multi-repo:
    - path: ".factory-project/STATE.md"
  optional:
    - path: ".factory/wave-state.yaml"
    - path: ".factory/policies.yaml"

# Paths — templated, glob-aware
paths:
  state:        ".factory/STATE.md"
  wave_state:   ".factory/wave-state.yaml"
  event_log:    ".factory/logs/events-*.jsonl"
  cycles:       ".factory/cycles/"
  workflows:    "${CLAUDE_PLUGIN_ROOT}/workflows/"
  multi_repo:   ".factory-project/"

# Schema expectations
schema:
  frontmatter:
    mode:           {type: string}    # free-form, future-compat
    phase:          {type: integer}   # surface raw if non-int
    status:         {type: string}
    current_step:   {type: string}
    current_cycle:  {type: string, optional: true}
    project:        {type: string}
    pipeline:       {type: string, optional: true}   # free-form enum
  body_tables:
    phase_progress:    {required: true}
    decisions_log:     {required: false}
    blocking_issues:   {required: false}
    session_resume:    {required: false, single_row: true}

# Display surface
display:
  primary:   [mode, phase, current_step]
  secondary: [current_cycle, wave, next_gate, drift_count]
  activity:
    source: event_log
    scope:  current_session_id
    limit:  20
  drift:
    source: state_md.body_tables.blocking_issues

# Permissions
permissions:
  read_only: true   # monocle NEVER writes
```

## Workflow Execution Semantics Monocle Surfaces

Monocle SURFACES these fields; it does NOT execute. The dispatcher and orchestrator own execution.

| Field | Source | Read path |
|---|---|---|
| Active workflow | STATE.md mode + workflows/&lt;mode&gt;.lobster | frontmatter `mode` + load matching `.lobster`, read `workflow.name` |
| Active phase | STATE.md `phase` | frontmatter integer (surface raw on drift) |
| Active step | STATE.md `current_step` | frontmatter string |
| Active cycle | STATE.md `current_cycle` | frontmatter string |
| Step task | workflow `steps[name==current_step].task` | YAML walk |
| Pending gate | wave-state.yaml `next_gate_required` | YAML |
| Recent activity | `.factory/logs/events-*.jsonl` filtered by session_id | JSONL tail |
| Drift items | STATE.md body Blocking Issues table | markdown table parse |
| Convergence trajectory | `cycles/<current>/convergence-trajectory.md` | markdown parse |
| Information-asymmetry context | workflow `steps[].context.exclude` for current step | YAML |
| Adversarial pass progress | wave-state.yaml `gate_pass_N` OR `cycles/<current>/adversarial-reviews/pass-*.md` count | YAML + directory count |

Per pass-2-architecture.md §C5 "Workflow Execution Skills" + pass-3-domain-model.md §"Bounded Contexts" (monocle's bounded context is READ-ONLY OBSERVATION).

### What monocle does NOT need to do

Per pass-B-deep-workflows-r1.md §"Conditional Expression Vocabulary" + pass-B-deep-workflows-r2.md §"O-R2-4. Gate criteria are free-form assertions" + pass-B-deep-workflows-r2.md §"Workflow Self-Documentation Pattern":

- Parse `condition:` expressions (no grammar exists).
- Evaluate `gate.criteria` bullets (free-form English).
- Render execution plans.
- Distinguish `cost_monitoring` vs `cost_tracking` semantically.
- Interpret `DF-XXX` upgrade tags.

## Risk Register

### P0 — must-have for workflow-awareness MVP

| ID | Finding | Monocle implication |
|---|---|---|
| P0-1 | `document_type: pipeline-state` is THE discriminator | Build detection around frontmatter, not directory presence. False-positive on bare `.factory/` is unacceptable. |
| P0-2 | STATE.md is read-only for monocle | NEVER write. Mutation belongs to state-manager. The single-commit burst protocol depends on this. |
| P0-3 | Dispatcher is observable through JSONL events ONLY | Do NOT invoke the dispatcher binary. Do NOT scrape stderr. Do NOT parse the internal log. |
| P0-4 | Workflows are static YAML | Use native YAML parsing in monocle's language, NOT bin/lobster-parse. |
| P0-5 | Information-asymmetry walls MUST be honored | If monocle ever feeds session context into adversary-context, exclude `.factory/cycles/**/adversarial-reviews/**`, `.factory/semport/**`, `.factory/holdout-scenarios/**`, `.factory/cycles/**/implementation/implementer-notes*`. |
| P0-6 | Multi-repo detection is by directory name | `.factory-project/` directory presence — not frontmatter. |
| P0-7 | Phase value drift is real | Surface raw value AND flag drift. Do not fail-closed on `phase: 3.5` or compound strings. |

### P1 — important for v1, not blocking

| ID | Finding | Monocle implication |
|---|---|---|
| P1-1 | Convergence-loop pattern is a recognizable template | 3+ workflows use it identically. Label as "adversarial pass M of 10" in UI. |
| P1-2 | input-hash-drift-check is universal pre-gate skill | Surface stale-input as blocker before proposing next work. |
| P1-3 | `cost_monitoring` vs `cost_tracking` are dual spellings | Accept both. Render whatever fields are present. |
| P1-4 | Wave gate pending state outranks drift items | Pending gate is a higher-priority blocker. |
| P1-5 | `dispatcher.schema_mismatch` is a SYSTEM health signal | Surface separately from pipeline state. ALL hooks bypassed when this fires. |
| P1-6 | SessionStart/SessionEnd are once-per-session | Use as session boundaries. Assume exactly one of each per session_id. |

### P2 — nice-to-have

| ID | Finding | Monocle implication |
|---|---|---|
| P2-1 | Stderr trace line exists per dispatcher invocation | Don't depend on it. Internal log is auxiliary. File sink is primary. |
| P2-2 | Resolvers are an internal dispatcher optimization | Monocle does not need to model resolvers. |
| P2-3 | Hook priority allocation suggests UI tier hints | 100-199 tool validators, 200-299 convergence/regression, 800-999 stop hooks. Can be used to order activity timeline. |

## Test Coverage Notes

vsdd-factory has 520+ bats tests (per pass-1-project-discovery.md §"OUT-OF-SCOPE"). These were not deeply read in this scoped ingest; they validate the dispatcher + hooks + skills internals which monocle does not port.

### What's tested in vsdd-factory (inferred)

- Dispatcher routing logic — direct testable via `routing.rs:151-256` test suite (pass-4-behavioral-contracts.md BC-MON-005).
- Registry validation — E-REG-001/002/003 paths tested.
- Capability denial — `INTERNAL_CAPABILITY_DENIED` event implies coverage.
- Hook scripts — bats suites in `tests/` (not read).

### Coverage gaps that affect monocle's adapter implementation

- No documented event-log SLA (per pass-5-nfr-catalog.md §"Missing NFRs"). Monocle should not assume events appear synchronously — the async drain window can defer telemetry by 100ms+.
- No documented session correlation across sessions. `session_id` is stable within a Claude session but not across.
- No public API for monocle-style introspection. The only programmatic interfaces are `bin/lobster-parse`, `bin/factory-dashboard`, and direct file reads. There is no JSON-RPC endpoint.
- `.lobster` format has no formal grammar for `condition:` expressions or `gate.criteria` assertions.

### Monocle's own test surface

The 15 holdout scenarios in pass-7-holdout-seeds.md (H1-H15) define monocle's factory-awareness acceptance suite:

- H1-H3: factory detection (positive, multi-repo, negative).
- H4-H5: lobster parse + sub-workflow composition.
- H6-H7: wave state surfacing + stale-gate blocking.
- H8-H9: hook event + drift surfacing.
- H10: forward-compatible factory detection.
- H11-H12: missing/corrupt STATE.md + size budget.
- H13: phase numbering drift.
- H14: lobster immutability.
- H15: information-asymmetry honoring.

Each maps to a P0/P1 finding above; passing all 15 is the minimum bar for factory-awareness v1.

## Architecture Recommendations

1. **Define factory-adapter interface as a versioned schema.** Use the manifest sketch above. vsdd-factory becomes the first concrete adapter; design for N more (future "ada-factory", "rust-factory", etc.).

2. **Session view layout.** Primary row: mode/phase/step/cycle. Secondary row: wave/gate/drift count. Activity panel: event log tail filtered by current `session_id`.

3. **Detection runs on session open. Cache for session.** Re-detect on `WorktreeCreate` / `WorktreeRemove` events to handle worktree-add or worktree-remove during a session.

4. **Never touch factory branches.** `factory-artifacts` and `factory-project-artifacts` are orphan branches owned by state-manager. Read from the working tree only — `.factory/` is mounted as a worktree so the working tree reflects current state without branch switching.

5. **Treat event log as append-only.** Use `tail -F` semantics. The dispatcher writes JSONL one event per line.

6. **Provide refresh affordance.** STATE.md changes when state-manager commits. Visible immediately in working tree. Monocle should detect mtime changes and re-render.

7. **Display workflow steps as a linear timeline with branch-on-condition labels.** The DAG is real but practically linear enough that a flat timeline with condition tags carries the operator value. Do not render the full DAG.

8. **Surface, do not enforce.** Monocle observes only. Never block, never warn, never propose edits to `.lobster` files (per BC-MON-010 + holdout H14).

9. **Future-proof against schema bumps.** Do not fail-closed on unknown values — `mode: ada-factory`, `pipeline: NEW_STATE`, `phase: 3.5` all must render as labels rather than errors.

10. **Multi-factory readiness from day one.** Each adapter contributes a manifest; monocle's core stays factory-agnostic. The detection algorithm returns `vsdd | vsdd-multi | unknown | None`, and `unknown` renders generic factory-shaped views.

## Convergence Statement

Per-category round count and convergence status, no padding:

| Category | Rounds to NITPICK | Status |
|---|---|---|
| Workflows (.lobster format) | 3 rounds | Converged (NITPICK declared in pass-B-deep-workflows-r3.md) |
| Factory dispatcher | 2 rounds | Converged (NITPICK declared in pass-B-deep-dispatcher-r2.md) |
| Factory project discriminator | 1 round | Provisionally converged (ingest blocked before R2; coverage in this synthesis adequate for v1) |
| STATE.md schema | inline only | Adequate — covered by Pass 4 BCs + this synthesis section; not formally deepened |
| Workflow execution skills | inline only | Adequate — covered by Pass 2 §C5 + this synthesis section; not formally deepened |

**Iron Law honesty:** the STATE.md schema and workflow-execution-skill categories were not subjected to formal multi-round deepening. The inline coverage in Pass 4 (8 BCs touching STATE.md directly) plus the dedicated section above is sufficient for monocle's MVP factory-adapter implementation but is not the same as a fully-deepened pass. If monocle's STATE.md rendering or skill mirroring reveals gaps, a follow-up deepening round may be warranted.

**Scoped ingest acknowledgment:** this is a scope-limited ingest. The deliberately-out-of-scope buckets (agents, full skill catalog, rules, fixtures, tests, docs, hook-plugins, sink crates, hook-sdk, hook-sdk-macros, vsdd-context-resolvers) are NOT covered. If monocle's product later expands to include workflow execution or hook authoring, additional scoped ingests will be needed for those buckets.

## Handoff

### For create-brief

Workflow-awareness should be positioned as a v3 monocle feature (post-v1 settings/observability MVP, post-v2 worktree-aware diff view). The brief should describe monocle as "the read-only observer for factory-driven projects" — emphasize that monocle's value comes from surfacing engine state coherently, not from running the engine.

Key user stories the brief should highlight (from the holdout seeds):

- "When I open a vsdd-factory project, monocle tells me what phase/step is running, what's blocked, and what's pending."
- "When I switch sessions, monocle remembers the factory context via stable `session_id` correlation."
- "When my factory is a different (compatible) dispatcher than vsdd-factory, monocle still shows me the basics."

### For disposition-pass

Per-subsystem disposition recommendations:

| Subsystem | Disposition | Reason |
|---|---|---|
| `.lobster` schema parser | MODEL | Implement in monocle's native language using YAML library. Do not port `bin/lobster-parse`. |
| Factory dispatcher binary | LEAVE-BEHIND | Owned by vsdd-factory. Monocle never invokes it. |
| `hooks-registry.toml` schema | MODEL (read-only) | Implement a read-only parser if monocle ever surfaces "configured hooks" — optional for MVP. |
| Event log JSONL reader | REIMPLEMENT | Native tail/parse in monocle's language. |
| STATE.md frontmatter parser | MODEL | Standard YAML frontmatter parse. |
| STATE.md body table parser | MODEL | Standard markdown table parse for Phase Progress, Blocking Issues, etc. |
| `wave-state.yaml` parser | MODEL | Standard YAML parse. |
| Cycle directory traversal | MODEL | Standard filesystem read. |
| Factory-adapter interface | ENHANCE (new in monocle) | Sketched in this synthesis; first concrete adapter is vsdd-factory. |
| 7 workflow-execution skills | LEAVE-BEHIND for execution; MIRROR read paths | Monocle does not execute. Read paths (`factory-dashboard`, `check-state-health`, `recover-state`) inform monocle's UI but stay in vsdd-factory. |
| 33+ specialist agents | LEAVE-BEHIND | Owned by vsdd-factory. |
| Bash hooks (27) | LEAVE-BEHIND | Owned by vsdd-factory. |

### For create-prd

The following Behavioral Contracts from pass-4-behavioral-contracts.md become BC-S.SS.NNN entries in monocle's PRD (4-level hierarchy per pass-6-conventions.md §"Identifier Format Hierarchy"):

- BC-MON-022 → Section 1 (Detection): the factory discriminator. Monocle PRD BC under "Project Detection" capability.
- BC-MON-015 + BC-MON-007 → Section 2 (State Reading): STATE.md frontmatter contract + size budget.
- BC-MON-001 + BC-MON-002 + BC-MON-010 + BC-MON-014 → Section 3 (Workflow Reading): YAML format, DAG ordering, immutability, parser path.
- BC-MON-021 → Section 4 (Event Surfacing): JSONL event log read.
- BC-MON-011 → Section 5 (Event Types): 10 Claude Code event types monocle recognizes.
- BC-MON-019 → Section 6 (Blocking Logic): wave-gate prerequisite as blocker.
- BC-MON-023 → Section 7 (Information Asymmetry): context.exclude honoring.
- BC-MON-017 → Section 8 (Multi-Repo): `.factory-project/` detection.
- BC-MON-018 → Section 9 (Cycle Surfacing): per-cycle archive read.

Each BC should be re-numbered into monocle's own BC-S.SS.NNN scheme during PRD authoring; the BC-MON-* IDs above are draft identifiers from this scoped ingest.

## Files in this Scoped Ingest

| Path | LOC |
|---|---|
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-1-project-discovery.md | 163 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-2-architecture.md | 344 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-3-domain-model.md | 321 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-4-behavioral-contracts.md | 223 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-5-nfr-catalog.md | 88 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-6-conventions.md | 216 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-7-holdout-seeds.md | 131 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-B-deep-workflows-r1.md | 314 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-B-deep-workflows-r2.md | 165 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-B-deep-workflows-r3.md | 163 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-B-deep-dispatcher-r1.md | 259 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-B-deep-dispatcher-r2.md | 144 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-B-deep-factory-pattern-r1.md | 251 |
| /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md | (this file) |

Total prior pass LOC: 2,782 across 13 files.

## State Checkpoint

```yaml
pass: 8
phase: C
status: complete
scope: workflow-awareness (scoped ingest)
timestamp: 2026-05-11T23:30:00Z
input_files: 13
synthesis_sections: 12
supersedes: pass-7-holdout-seeds.md (which was a synthesis-stub, not a final synthesis)
next: handoff to create-brief / disposition-pass / create-prd
```
