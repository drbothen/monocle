# Pass 2 — Scoped Architecture: vsdd-factory

## Top-Level Architecture (Monocle Lens)

vsdd-factory is structured around three independently-versioned layers that monocle must understand:

1. **Data Layer** — `.lobster` workflow files (YAML) + STATE.md + wave-state.yaml + cycles/. These are the **observables** monocle reads.
2. **Driver Layer** — orchestrator agent + 7 workflow-execution skills + `bin/lobster-parse`. Reads (1) and dispatches work.
3. **Enforcement Layer** — Rust dispatcher binary + hooks-registry.toml + WASM hook plugins (incl. legacy-bash-adapter). Runs OUT-OF-BAND of (2): every Claude Code tool call routes through here regardless of who initiated it. Emits events to `.factory/logs/events-*.jsonl`.

Monocle observes Layer (1) directly (file reads), can interrogate Layer (2) via skill/command output, and ingests Layer (3) **only through the JSONL event log** — it does NOT touch the dispatcher binary or registry directly.

## Component Catalog

### C1. The Lobster Workflow System

**Files:** 16 `.lobster` files under `workflows/` + `workflows/phases/`.

**Format:** YAML with a single top-level `workflow:` key. Verified via `bin/lobster-parse` (a thin `yq | jq` wrapper, evidence: `bin/lobster-parse:1-51`).

**Top-level fields (observed across all 16 files):**

| Field | Type | Required | Example |
|---|---|---|---|
| `workflow.name` | string | yes | `brownfield-vsdd`, `feature-vsdd`, `per-story-delivery` |
| `workflow.description` | string (folded) | yes | Long-form prose |
| `workflow.version` | semver string | yes | `"3.0.0"`, `"2.0.0"`, `"2.1.0"` |
| `workflow.defaults` | object | optional | `{on_failure: escalate, max_retries: 2, timeout: "2h"}` |
| `workflow.steps` | list of step objects | yes | (see step schema below) |
| `workflow.inputs` | list | optional (sub-workflows) | code-delivery.lobster has `[story_id, worktree_path, feature_type, module_criticality, implementation_strategy]` |
| `workflow.cost_monitoring` | object | optional | `{enabled: true, metadata: {mode: feature}, thresholds: {warn: 0.70, pause: 0.95}, protected_agents: [adversary, ...]}` |
| `workflow.cost_tracking` | object | optional | feature.lobster:62-76 |
| `workflow.schedule` | object | optional (discovery) | `{market_research: weekly, feedback_ingestion: daily, ...}` |
| `workflow.state_fields` | list of strings | optional (discovery) | Names of STATE.md fields tracked during run |

**Step schema (verified across all 16 files):**

| Field | Type | Required | Notes |
|---|---|---|---|
| `name` | string | yes | unique within workflow |
| `type` | enum | yes | `agent`, `skill`, `command`, `gate`, `human-approval`, `sub-workflow`, `loop`, `parallel-foreach` |
| `depends_on` | list of step names | yes (may be `[]`) | DAG edges |
| `agent` | string | conditional | required if `type: agent`; agent ID like `dx-engineer`, `state-manager` |
| `skill` | string | conditional | required if `type: skill`; path `skills/<name>/SKILL.md` |
| `sub_workflow` | string | conditional | required if `type: sub-workflow`; filename like `greenfield.lobster` |
| `task` | string (folded) | conditional | natural-language task description (agent steps) |
| `condition` | string expression | optional | `"routing.choice == 'feature'"`, `"!file_exists('CLAUDE.md')"` |
| `optional` | bool | optional | `true` means skip-if-fails |
| `timeout` | duration string | optional | `"30m"`, `"4h"`, overrides `defaults.timeout` |
| `on_failure` | enum | optional | `escalate`, `retry`, `skip` (default from `defaults.on_failure`) |
| `max_retries` | int | optional | inherits from defaults |
| `gate.criteria` | list of strings | conditional | required if `type: gate`; bullet-list assertions |
| `gate.fail_action` | enum | conditional | `block`, `warn` |
| `approval.prompt` | string (folded) | conditional | required if `type: human-approval` |
| `approval.artifacts` | list of glob paths | optional | files surfaced for human review |
| `approval.timeout` | duration | optional | typically `"24h"`, `"48h"`, `"72h"` |
| `model_tier` | string | optional | `review`, `adversary` (overrides default tier) |
| `context.include` | list of glob paths | optional | files visible to agent |
| `context.exclude` | list of glob paths | optional | **information asymmetry walls** — files hidden from the spawned agent |
| `loop.max_iterations` | int | conditional | required if `type: loop` |
| `loop.exit_condition` | string expression | conditional | required if `type: loop` |
| `loop.for_each` | string | optional | `"finding in auto_fixable_findings"` |
| `loop.steps` | list of step objects | conditional | nested steps inside loop |
| `iterator` | list (with step shape) | conditional | used by `parallel-foreach` |
| `collection` | string expression | conditional | `"repos.where(mode == 'brownfield')"` |
| `cwd` | path string | optional | per-step working directory (multi-repo) |
| `config` | object | optional | step-specific config (e.g., `depth: "L1"`) |
| `inputs` | object | optional | sub-workflow input bindings |
| `wait_for_optional` | list of step names | optional | "wait for these IF they ran" |
| `description` | string | optional | secondary description |

**Step type distribution (sampled):**
- `agent` — dominant; ~60% of steps in feature/greenfield workflows
- `skill` — ~25%
- `gate` — ~10%; checkpoint/criteria evaluation
- `human-approval` — ~5%; ALWAYS has timeout, prompt, artifacts
- `sub-workflow` — used heavily by brownfield/feature/maintenance to invoke `greenfield.lobster`, `code-delivery.lobster`
- `loop` / `parallel-foreach` — used in maintenance (per-finding fix), code-delivery (adversarial convergence loop), multi-repo (per-repo Phase 0)
- `command` — rare

### C2. The Factory Dispatcher (Rust + WASM)

**Source crate:** `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/` (Rust workspace).

**Binary output (5 platform variants):** `plugins/vsdd-factory/hooks/dispatcher/bin/{darwin-arm64,darwin-x64,linux-arm64,linux-x64,windows-x64}/factory-dispatcher[.exe]`. The macOS arm64 binary is ~12 MB Mach-O 64-bit executable (verified via `file`).

**Cargo.toml key deps** (`crates/factory-dispatcher/Cargo.toml:20-39`):
- `wasmtime` + `wasmtime-wasi` — WASM host runtime
- `tokio` (current_thread flavor) — async runtime
- `serde`/`serde_json`/`toml` — registry + payload parsing
- `regex` — tool-name matching
- `anyhow`/`thiserror` — error handling
- `chrono`, `uuid`, `tracing` — telemetry
- Internal sinks: `sink-core`, `sink-datadog`, `sink-file`, `sink-honeycomb`, `sink-http`, `sink-otel-grpc`

**Pipeline (per `src/main.rs`):**
1. Build internal log → prune old entries (S-1.7 always-on telemetry).
2. Read hook envelope from stdin (`HookPayload::from_reader`, `payload.rs:77-81`).
3. Resolve registry path from `$CLAUDE_PLUGIN_ROOT/hooks-registry.toml`.
4. Load + validate `Registry` (schema_version=2 strict; fail-closed on schema mismatch).
5. Match plugins by `event_name` + `tool_name` regex (`routing.rs:37-58`).
6. Group by priority (lower fires first; same priority = parallel) (`routing.rs:63-87`).
7. Partition plugins into sync_group vs async_group (`partition.rs`).
8. Execute sync plugins, await; aggregate exit codes.
9. Spawn async plugins fire-and-forget (drained within `ASYNC_DRAIN_WINDOW_MS`).
10. Emit `plugin.invoked`, `plugin.completed`, `plugin.timeout`, `plugin.crashed` events.
11. Compute final exit code (0 continue, 2 fail-closed on registry errors E-REG-001/002/003).

**Engine details (`engine.rs:33-41`):**
- wasmtime `Engine` built once per dispatcher process (~100 ms cost).
- Epoch-interruption enabled (ticker thread bumps epoch every 10 ms).
- Fuel consumption enabled (per-plugin `fuel_cap`).
- Timeout resolution = epoch tick cadence (10 ms).

**Claude Code hook events handled** (`hooks/hooks.json.template:3-115`):
- `PreToolUse`
- `PostToolUse`
- `PermissionRequest`
- `Stop`
- `SubagentStop`
- `SessionStart` (with `"once": true` — fires once per session)
- `SessionEnd` (with `"once": true`)
- `WorktreeCreate`
- `WorktreeRemove`
- `PostToolUseFailure`

All 10 events route to the same `factory-dispatcher` binary with a 10-second timeout. The dispatcher decides which plugins fire based on the registry.

**Stdin payload shape** (`payload.rs:11-54`):
```json
{
  "event_name|hook_event_name": "PreToolUse",
  "tool_name": "Bash",
  "session_id": "<stable-per-session-id>",
  "tool_input": { ... tool-specific ... },
  "tool_response": { ... post-events only ... },
  /* event-specific extras flattened: agent_type, subagent_name, last_assistant_message, result, etc. */
}
```

### C3. hooks-registry.toml Protocol

**Schema** (`crates/factory-dispatcher/src/registry.rs:18-260`):

```toml
schema_version = 2  # MANDATORY; dispatcher refuses anything else

# Registry-wide defaults (all optional)
[defaults]
timeout_ms = 5000
fuel_cap = 10_000_000
on_error = "continue"  # or "block"
priority = 500

# A plugin entry
[[hooks]]
name = "capture-commit-activity"     # unique
event = "PostToolUse"                # Claude Code event
tool = "Edit|Write"                  # optional regex; None = all tools
plugin = "hook-plugins/foo.wasm"     # relative to CLAUDE_PLUGIN_ROOT
priority = 110                       # lower = fires first
enabled = true                       # default true
timeout_ms = 5000                    # per-plugin override
fuel_cap = 10_000_000                # per-plugin override
on_error = "continue"                # "continue" or "block"
async = true                         # default false; true = fire-and-forget
needs_context = []                   # resolver names for context injection

[hooks.config]                       # plugin-defined config table
script_path = "hooks/foo.sh"

[hooks.capabilities]                 # deny-by-default
env_allow = ["PATH", "HOME", ...]

[hooks.capabilities.exec_subprocess]
binary_allow = ["bash", "jq"]
shell_bypass_acknowledged = "..."    # opt-in for shell interpreters
cwd_allow = []
env_allow = [...]

[hooks.capabilities.read_file]
path_allow = [".factory/...", ...]

[hooks.capabilities.write_file]
path_allow = [...]
max_bytes_per_call = 65536
```

**Hard invariants (enforced fail-closed at load time):**
- `E-REG-001`: schema_version must == 2.
- `E-REG-002`: `on_error = "block"` + `async = true` is forbidden.
- `E-REG-003`: duplicate (name, event, tool) tuple forbidden.

**Live registry**: `plugins/vsdd-factory/hooks-registry.toml` is 996 lines, ~56 entries. Per the file's header comment (`hooks-registry.toml:1-9`): "21 native-WASM ports coexist with 35 legacy-bash-adapter entries" — most entries currently use `plugin = "hook-plugins/legacy-bash-adapter.wasm"` with `[hooks.config] script_path = "hooks/<name>.sh"` to wrap unported bash hooks.

### C4. The .factory/ Directory (the surface monocle reads)

Per `templates/state-template.md`, `agents/orchestrator/orchestrator.md`, and `skills/recover-state/SKILL.md`, a vsdd-factory project owns a `.factory/` directory mounted as a git worktree on an orphan branch `factory-artifacts`. Standard subtree:

```
.factory/
├── STATE.md                          # Pipeline state (frontmatter + body)
├── SESSION-HANDOFF.md                # Optional inter-session handoff
├── wave-state.yaml                   # Wave lifecycle tracker
├── policies.yaml                     # Project policy rubric (optional)
├── current-cycle                     # symlink or text file pointing to active cycle
├── tech-debt-register.md
├── logs/
│   └── events-*.jsonl                # Dispatcher event log (monocle's primary read)
├── phase-0-ingestion/                # Brownfield mode
│   ├── project-context.md
│   ├── recovered-architecture.md
│   ├── conventions.md
│   ├── security-audit.md
│   ├── adversarial-review-0.md
│   ├── validation-report.md
│   ├── verification-gap-analysis.md
│   └── behavioral-contracts/
│       └── BC-*.md
├── specs/                            # Phase 1 outputs
│   ├── product-brief.md
│   ├── prd.md
│   ├── dtu-assessment.md
│   ├── module-criticality.md
│   ├── domain-spec/L2-INDEX.md
│   ├── behavioral-contracts/BC-INDEX.md
│   ├── architecture/ARCH-INDEX.md
│   └── verification-properties/VP-INDEX.md
├── stories/                          # Phase 2 outputs
│   ├── STORY-INDEX.md
│   └── sprint-state.yaml
├── holdout-scenarios/                # Hidden test scenarios
│   └── HS-INDEX.md
├── cycles/                           # Per-cycle archives
│   └── <cycle-name>/
│       ├── cycle-manifest.md
│       ├── burst-log.md
│       ├── convergence-trajectory.md
│       ├── session-checkpoints.md
│       ├── lessons.md
│       ├── blocking-issues-resolved.md
│       ├── adversarial-reviews/pass-*.md
│       └── implementation/
├── planning/                         # Adaptive planning outputs
│   ├── artifact-inventory.md
│   ├── gap-analysis.md
│   ├── routing-decision.md
│   ├── market-intel.md
│   └── ...
├── feature/                          # Feature mode outputs
├── maintenance/                      # Maintenance sweep outputs
├── discovery/                        # Discovery mode outputs
├── session-reviews/
├── demo-evidence/
├── design-system/
└── ui-quality/
```

For multi-repo projects: a sibling `.factory-project/` directory (worktree on `factory-project-artifacts`) holds project-level coordination state (per `templates/factory-project-structure-template.md:7-34`).

### C5. Workflow Execution Skills

Seven SKILL.md files form the workflow-execution surface that monocle would mirror:

| Skill | Reads | Writes | Side effects |
|---|---|---|---|
| `validate-workflow` | a `.lobster` file | nothing | exit code |
| `next-step` | STATE.md + workflow file | nothing | prints proposal |
| `run-phase` | workflow file + STATE.md | STATE.md (append after each step) | spawns agents/skills |
| `check-state-health` | STATE.md | nothing | report |
| `recover-state` | `.factory/*` artifacts | STATE.md (rebuilt) | one-shot reconstruction |
| `state-update` | STATE.md | STATE.md + git commit | internal-only |
| `factory-dashboard` | STATE.md + wave-state.yaml + logs/events-*.jsonl | nothing | renders markdown |

These are the **read-paths monocle should mirror exactly** to be workflow-aware without taking ownership of state mutation.

## Layered Diagram (Monocle's perspective)

```
┌────────────────────────────────────────────────────────────────────────┐
│ Layer 3 — ENFORCEMENT (out-of-band)                                    │
│                                                                        │
│  Claude Code session                                                   │
│     │ tool call                                                        │
│     ▼                                                                  │
│  hooks/hooks.json.<platform>  ──fork──>  factory-dispatcher (Rust)     │
│                                              │                         │
│                                              ▼                         │
│                                       hooks-registry.toml              │
│                                              │                         │
│                                              ▼                         │
│                                   WASM plugins (sync + async)          │
│                                              │                         │
│                                              ▼                         │
│                                   .factory/logs/events-*.jsonl  ◄── monocle reads
└────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────┐
│ Layer 2 — DRIVER (in-band)                                             │
│                                                                        │
│  Orchestrator agent  ◄── reads workflows/*.lobster via bin/lobster-parse│
│         │                                                              │
│         ├── /vsdd-factory:run-phase <id>                               │
│         ├── /vsdd-factory:next-step                                    │
│         ├── /vsdd-factory:validate-workflow                            │
│         ├── /vsdd-factory:factory-dashboard      ◄── monocle should mirror
│         ├── /vsdd-factory:check-state-health     ◄── monocle should mirror
│         └── /vsdd-factory:recover-state                                │
│                                                                        │
│         delegates to specialist agents via Agent tool                  │
└────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────┐
│ Layer 1 — DATA (passive)                                               │
│                                                                        │
│  workflows/*.lobster          (workflow definitions)                   │
│  workflows/phases/*.lobster   (phase sub-workflows)                    │
│  .factory/STATE.md            (live state)                             │
│  .factory/wave-state.yaml     (wave tracker)                           │
│  .factory/policies.yaml       (governance rubric, optional)            │
│  .factory/cycles/<id>/...     (per-cycle archive)                      │
│  .factory/logs/events-*.jsonl (dispatcher event log)                   │
│                                                                        │
│  ◄── monocle reads this layer directly                                 │
└────────────────────────────────────────────────────────────────────────┘
```

## Cross-Cutting Concerns

- **Versioning of artifacts**: every persisted artifact has a YAML frontmatter `version` and `input-hash` field (input-hash drift is enforced by `hooks/validate-input-hash.sh`). Monocle should NOT attempt to recompute these.
- **Information asymmetry walls**: workflows declare `context.include`/`context.exclude` globs on agent steps. Critical for adversarial review: the adversary CANNOT see implementer notes, prior adversarial passes, semport history, or holdout scenarios. This is enforced by the agent dispatcher, not by the file system.
- **Branching model** (per root `CLAUDE.md:11-20`): `develop` (integration), `main` (releases only), `factory-artifacts` (orphan; `.factory/` worktree).
- **STATE.md is on `factory-artifacts`, not `develop`** — so STATE.md edits are committed independently of code changes. Monocle reads STATE.md from the working tree (no need to switch branches).
- **STATE.md size discipline**: warn at 200 lines, block at 500 (per `state-template.md:21-22` and `hooks/validate-state-size.sh`). Historical content routes to `cycles/<cycle>/*.md`.

## State Checkpoint

```yaml
pass: 2
status: complete
components_cataloged: 5
timestamp: 2026-05-11T22:05:00Z
next_pass: 3-domain-model
```
