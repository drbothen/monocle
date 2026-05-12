# Pass 4 — Scoped Behavioral Contracts: vsdd-factory

These are the behavioral contracts monocle must rely on as INVARIANTS when surfacing factory state. Each is grounded in source.

## BC-MON-001: Lobster file is YAML

**Statement**: Every `.lobster` workflow file is valid YAML with a single top-level `workflow:` key. There is no custom Lobster syntax — the name "lobster" is purely conventional.

**Evidence**: `bin/lobster-parse:13-14` ("Lobster files are YAML; this helper is a thin wrapper over yq + jq") and `bin/lobster-parse:41` (`yq eval --output-format=json '.' "$FILE"`). All 16 `.lobster` files inspected start with `workflow:`.

**Confidence**: HIGH.

## BC-MON-002: Workflow steps form a DAG ordered by `depends_on`

**Statement**: Steps within a workflow form a directed acyclic graph. Execution order respects `depends_on`. No forward references. No cycles. Same-`depends_on` steps may run in parallel.

**Evidence**: `validate-workflow/SKILL.md:21-23` ("Every `depends_on` entry must name an earlier step. No cycles, no forward references, no dangling references."). `orchestrator.md:37` ("For each step, spawn the declared agent via the Agent tool with the step's task as the prompt, honoring `depends_on` ordering.").

**Confidence**: HIGH.

## BC-MON-003: Dispatcher reads hook envelope from stdin

**Statement**: The factory-dispatcher reads a JSON envelope from stdin on every invocation. The envelope contains `event_name` (or alias `hook_event_name`), `tool_name`, `session_id`, `tool_input`, optional `tool_response`, and arbitrary `extra` fields.

**Evidence**: `crates/factory-dispatcher/src/payload.rs:11-54` (struct definition with `serde(alias = "hook_event_name")`, `serde(default)` for `tool_name`/`tool_input`/`tool_response`, `serde(flatten)` for `extra`). `crates/factory-dispatcher/src/main.rs:96` (`HookPayload::from_reader(std::io::stdin().lock())`).

**Confidence**: HIGH.

## BC-MON-004: hooks-registry.toml has strict schema_version = 2

**Statement**: The dispatcher refuses to start if `schema_version != 2`. Error code E-REG-001. This is one of three named fail-closed exceptions.

**Evidence**: `registry.rs:20` (`pub const REGISTRY_SCHEMA_VERSION: u32 = 2;`). `registry.rs:30-35` (`RegistryError::SchemaVersion`). `main.rs:131-138` (E-REG-001 path emits `dispatcher.schema_mismatch` event + `exit(2)`).

**Confidence**: HIGH.

## BC-MON-005: Plugin routing is event_name + tool_name regex

**Statement**: A registry entry matches a payload iff `entry.enabled` AND `entry.event == payload.event_name` AND (`entry.tool` is `None` OR `Regex::new(entry.tool).is_match(payload.tool_name)`).

**Evidence**: `crates/factory-dispatcher/src/routing.rs:37-58` (`match_plugins` + `tool_matches`).

**Confidence**: HIGH (directly testable from `routing.rs:151-256` test suite).

## BC-MON-006: Plugins fire in priority order, lower first

**Statement**: Matched plugins are partitioned into priority tiers; tiers run sequentially in ascending priority order; within a tier plugins run in parallel (in registry order for determinism).

**Evidence**: `routing.rs:63-87` (`group_by_priority`). `routing.rs:198-211` (test `group_orders_tiers_ascending`).

**Confidence**: HIGH.

## BC-MON-007: STATE.md is the single live state file

**Statement**: Each project has exactly one `.factory/STATE.md` file representing live pipeline state. Historical content is routed to `cycles/<cycle>/*.md`. The file has size budget 200 (warn) / 500 (block).

**Evidence**: `templates/state-template.md:19-32` (`STATE.md SIZE BUDGET` HTML comment + content routing rules). `skills/check-state-health/SKILL.md:44-51` (size verdict table).

**Confidence**: HIGH.

## BC-MON-008: STATE.md mutation is single-commit on factory-artifacts

**Statement**: STATE.md edits land in exactly ONE commit on the `factory-artifacts` branch, never two. The retired Two-Commit Protocol (Stage 1 placeholder + Stage 2 backfill SHA) is forbidden. Hook `verify-sha-currency.sh` enforces `MULTI_COMMIT_CHAIN_NOT_ALLOWED`.

**Evidence**: `state-manager-checklist-template.md:87-111` (full single-commit rule). `state-manager-checklist-template.md:106-111` (regression guard logic).

**Confidence**: HIGH.

## BC-MON-009: STATE.md cannot self-cite factory-artifacts HEAD SHA

**Statement**: STATE.md (and SESSION-HANDOFF.md) MUST NOT cite the current factory-artifacts HEAD SHA in "current state" prose. Historical SHA references (in changelog rows, decisions log, cycle manifests) remain immutable. TD-VSDD-053 retired the self-cite.

**Evidence**: `state-manager-checklist-template.md:161-163`, `:175-177`, `:248-253` (verification grep that must return empty).

**Confidence**: HIGH.

## BC-MON-010: Workflow files are immutable at runtime

**Statement**: `.lobster` files are data, not working documents. Skills/agents read them; they do NOT write them.

**Evidence**: `run-phase/SKILL.md:52` ("Do not edit the workflow file itself. It's data, not a working document."). `validate-workflow/SKILL.md:44` ("Do not fix the file. Report problems; the user fixes them.").

**Confidence**: HIGH.

## BC-MON-011: factory-dispatcher routes 10 Claude Code event types

**Statement**: The dispatcher handles `PreToolUse, PostToolUse, PermissionRequest, Stop, SubagentStop, SessionStart, SessionEnd, WorktreeCreate, WorktreeRemove, PostToolUseFailure`. All route to the same binary with 10s timeout.

**Evidence**: `hooks/hooks.json.template:3-115` (full listing).

**Confidence**: HIGH.

## BC-MON-012: Async plugin classification is per-entry boolean

**Statement**: `entry.async` (TOML key) defaults to `false`. `true` means fire-and-forget (verdict never affects dispatcher exit code). `async = true` combined with `on_error = "block"` is forbidden (E-REG-002, fail-closed).

**Evidence**: `registry.rs:228-245` (`async_flag` field, defaults, semantics). `registry.rs:46-51` (`AsyncBlockConflict`). `main.rs:140-149` (E-REG-002 fail-closed path).

**Confidence**: HIGH.

## BC-MON-013: Plugin capabilities are deny-by-default

**Statement**: A plugin without a `[hooks.capabilities]` block has no host-function access beyond always-on APIs (`log`, `emit_event`, `session_id`). Read/write/exec capabilities require explicit `path_allow` / `binary_allow` lists.

**Evidence**: `registry.rs:80-141` (Capabilities, ReadFileCaps, WriteFileCaps, ExecSubprocessCaps types). `registry.rs:84-92` (doc comment: "Deny-by-default — a missing block means the plugin cannot use the corresponding host function at all.").

**Confidence**: HIGH.

## BC-MON-014: lobster-parse is the canonical workflow read path

**Statement**: All reads of `.lobster` files go through `bin/lobster-parse`. The helper requires both `yq` and `jq` on PATH and exits with informative error if either is missing.

**Evidence**: `bin/lobster-parse:17-24` (preflight check). `orchestrator.md:32-38` (orchestrator usage examples). `run-phase/SKILL.md:30-32`, `next-step/SKILL.md:23-26`, `validate-workflow/SKILL.md:29-31`.

**Confidence**: HIGH.

## BC-MON-015: STATE.md frontmatter encodes minimal mode + phase

**Statement**: STATE.md frontmatter MUST contain `document_type: pipeline-state`, `project: <name>`, `mode: <greenfield|brownfield|feature|maintenance|discovery|multi-repo>`, `phase: <integer 0-7>`, `status: <in_progress|complete|blocked>`, `current_step: <string>`. Compound phase values like `2-story-decomposition-patch-cycle` are FLAGGED as drift.

**Evidence**: `skills/check-state-health/SKILL.md:28-41` (validation table).

**Confidence**: HIGH.

## BC-MON-016: `.factory/` is a git worktree on `factory-artifacts` orphan branch

**Statement**: The `.factory/` directory is mounted as a git worktree on an orphan branch named `factory-artifacts`. This branch shares no history with `develop` or `main`.

**Evidence**: `CLAUDE.md:11-20` (root). `state-manager-checklist-template.md:163` ("`git -C .factory log -1`"). `feature.lobster:84-98` (factory-worktree-health step + gate).

**Confidence**: HIGH.

## BC-MON-017: Multi-repo projects have `.factory-project/` AND per-repo `.factory/`

**Statement**: A multi-repo project has BOTH a project-level `.factory-project/` worktree (on `factory-project-artifacts` branch) AND per-repo `.factory/` worktrees (each on its own `factory-artifacts` branch).

**Evidence**: `templates/factory-project-structure-template.md:7-42`. `multi-repo.lobster:108-122`.

**Confidence**: HIGH.

## BC-MON-018: Cycle directory holds historical content

**Statement**: `.factory/cycles/<cycle-name>/` archives per-cycle: burst-log, convergence-trajectory, session-checkpoints, lessons, blocking-issues-resolved, adversarial-reviews/, implementation/. The cycle directory is created at pipeline start and finalized at cycle close.

**Evidence**: `state-template.md:27-32` (content routing rules). `feature.lobster:107-114` (`feature-cycle-init`). `cycle-manifest-template.md:1-59`.

**Confidence**: HIGH.

## BC-MON-019: Wave gate prerequisite enforced by PreToolUse hook

**Statement**: A `PreToolUse` hook `validate-wave-gate-prerequisite.sh` blocks dispatch of stories into wave N+1 while wave N's gate is `pending`.

**Evidence**: `wave-state-template.yaml:6-7` (header comment names this hook).

**Confidence**: HIGH (validated by template comment; the hook itself is in `hooks/validate-wave-gate-prerequisite.sh` per Pass 1 inventory).

## BC-MON-020: Dispatcher exit codes have semantic meaning

**Statement**: Exit 0 = continue (default; even on fail-open registry errors). Exit 2 = fail-closed (only named exceptions: E-REG-001 schema mismatch, E-REG-002 async-block conflict, E-REG-003 duplicate entry). Plugin verdicts in sync_group can additionally produce a "block" exit code aggregated via `aggregate_exit_code` (`main.rs:60`).

**Evidence**: `main.rs:80-92` (default to 0 on dispatcher errors). `main.rs:131-149` (exit 2 paths). ADR-019 §Decision 2 cited inline.

**Confidence**: HIGH.

## BC-MON-021: Plugin events are JSONL append-only

**Statement**: Plugin-emitted events (via `host::emit_event`) are appended as JSONL to a sink path. In debug builds, the path is `VSDD_SINK_FILE`. In production, sinks route to `.factory/logs/events-*.jsonl` (file sink) and/or external backends (datadog, honeycomb, http, otel-grpc).

**Evidence**: `main.rs:23-31` (VSDD_SINK_FILE doc). `Cargo.toml:34-38` (sink crates). `skills/factory-dashboard/SKILL.md:30-35` (reads `.factory/logs/events-*.jsonl`).

**Confidence**: HIGH for the file sink path; MEDIUM for the JSONL format (inferred from "JSONL" in `main.rs:30` doc + dashboard reading `events-*.jsonl`).

## BC-MON-022: A vsdd-factory project is identified by `.factory/STATE.md` + a workflow file

**Statement**: Monocle can identify "this project uses vsdd-factory (or a compatible factory)" by detecting:
- (a) presence of `.factory/STATE.md` with `document_type: pipeline-state` frontmatter, OR
- (b) presence of a `.lobster` file anywhere referenced by STATE.md, OR
- (c) presence of `plugin.json` with `name: vsdd-factory` and a sibling `hooks-registry.toml`.

Combining all three signals gives high confidence.

**Evidence**: STATE.md frontmatter is the most reliable signal (`templates/state-template.md:1-17`). `plugin.json` is the canonical name discriminator BUT lives in `~/.claude/plugins/cache/...` for installed plugins, not the user's project — so per-project detection MUST rely on STATE.md.

**Confidence**: HIGH (synthesized from evidence; this is the discriminator monocle will implement).

## BC-MON-023: Adversarial review enforces information asymmetry

**Statement**: Adversary agent dispatches MUST include `context.exclude` globs for: `.factory/cycles/**/implementation/implementer-notes*`, `.factory/cycles/**/implementation/red-gate-log*`, `.factory/cycles/**/adversarial-reviews/**`, `.factory/semport/**`, `.factory/holdout-scenarios/**`. Monocle MUST NOT surface adversarial-review content to other agents WITHIN A SESSION VIEW that may be observed by adversary agents.

**Evidence**: `code-delivery.lobster:118-135` (per-story adversarial review context.exclude list).

**Confidence**: HIGH.

## BC-MON-024: Workflows are dispatched, not parsed at registration time

**Statement**: The factory-dispatcher binary does NOT parse `.lobster` files. Workflow parsing is done by the orchestrator agent + skills (via `bin/lobster-parse`). The dispatcher operates entirely below the workflow abstraction.

**Evidence**: No reference to `.lobster` anywhere in `crates/factory-dispatcher/src/`. The dispatcher's only inputs are stdin envelopes + `hooks-registry.toml`. Verified by reading `main.rs:1-150` (no workflow loading code).

**Confidence**: HIGH.

## Gaps & Low-Confidence Areas

| Area | Gap | Confidence |
|---|---|---|
| Event log filename format | The exact `events-*.jsonl` filename pattern is mentioned but I did not read the file-sink crate (`sink-file/`). | MEDIUM |
| Resolver registry | `resolvers-registry.toml` exists at 18 lines; the resolver concept (`needs_context` in registry entries) is referenced but I did NOT deep-dive resolver semantics. | LOW-MEDIUM |
| Plugin verdict format | Plugins return outcomes (`Continue`/`Block`/`Error`); the exact wire format for a `Block` verdict (does it include a reason that monocle should surface?) is referenced in `routing.rs:19-24` but the executor's parsing (`executor.rs`) is not deeply analyzed. | MEDIUM |
| Compact-state semantics | `/vsdd-factory:compact-state` is referenced but its SKILL.md was not read (out of scope). | LOW |
| Policies.yaml schema | `policies.yaml` is referenced (`orchestrator.md:244-247`) but the schema file (`templates/policies-template.yaml`) was not deep-dived. | LOW |

## State Checkpoint

```yaml
pass: 4
status: complete
contracts: 24
high_confidence: 22
medium_confidence: 2
gaps: 5
timestamp: 2026-05-11T22:15:00Z
next_pass: 5-nfr
```
