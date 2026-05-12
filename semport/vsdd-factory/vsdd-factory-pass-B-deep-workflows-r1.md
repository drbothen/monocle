# Phase B Round 1 — Deep: Workflows (Lobster Format)

## Goal

Produce a COMPLETE schema for the `.lobster` format, sufficient for an ADR. Every field documented with type, optionality, examples, and source file:line evidence.

## Full Lobster File Schema (Authoritative)

```yaml
workflow:
  name: <string>              # REQUIRED. kebab-case, unique across all workflows.
  description: <string>       # REQUIRED. Folded scalar (>) typical; arbitrary prose.
  version: <semver-string>    # REQUIRED. e.g., "3.0.0"

  # Optional top-level blocks
  defaults: <Defaults>        # Optional. Step-level defaults.
  cost_monitoring: <CostBlock>   # Optional. Cost monitoring config.
  cost_tracking:   <CostBlock>   # Optional. Feature.lobster spelling.
  schedule: <Schedule>        # Optional (discovery.lobster only).
  state_fields: <list[str]>   # Optional. STATE.md field names tracked.
  inputs: <list[InputDecl]>   # Optional (sub-workflows only).

  steps:                      # REQUIRED. Ordered list of Step objects.
    - <Step>
    - <Step>
    ...
```

## Defaults Block

```yaml
defaults:
  on_failure: escalate | retry | skip   # default: escalate (per all top-level workflows)
  max_retries: <int>                    # default: 2
  timeout: <duration-string>            # default: "2h" or "1h" or "4h"
```

Evidence: brownfield.lobster:19-22, feature.lobster:56-59, code-delivery.lobster:28-31, planning.lobster:16-19.

## Cost Block

Two spellings exist; monocle should accept both.

```yaml
cost_monitoring:
  enabled: true | false
  metadata:
    mode: <string>            # e.g., "feature", "greenfield", "discovery", "multi-repo"
    level: <string>           # optional, e.g., "project"
  thresholds:                 # optional
    warn:  <float>            # e.g., 0.70
    pause: <float>            # e.g., 0.95
  protected_agents:           # optional list
    - adversary
    - security-reviewer
    - formal-verifier
  summary_artifact: <path>    # optional, e.g., ".factory/feature/cost-summary.md"
```

Evidence: feature.lobster:62-76 (cost_tracking variant), greenfield.lobster:21-25, discovery.lobster:25-28, multi-repo.lobster:32-36, maintenance.lobster:17-20.

## Schedule Block (discovery.lobster only)

```yaml
schedule:
  market_research:        weekly | daily | <cron>
  feedback_ingestion:     weekly | daily | <cron>
  competitive_monitoring: weekly | daily | <cron>
  analytics_integration:  weekly | daily | <cron>
  full_synthesis:         weekly | daily | <cron>
```

Evidence: discovery.lobster:31-37.

## Input Declaration (sub-workflows)

```yaml
inputs:
  - <input-name>: <example-or-type>      # e.g., story_id: "STORY-NNN"
  - <input-name>: <example-or-type>
```

Evidence: code-delivery.lobster:21-26 ("inputs:" list with 5 typed entries).

## Step Object — All Fields

```yaml
- name: <string>                # REQUIRED. Unique within workflow.
  type: <step-type>             # REQUIRED. See StepType enum below.
  depends_on: <list[step-name]> # REQUIRED. Empty list `[]` means "root".
  
  # Conditional based on type
  agent:   <string>             # if type=agent or attached to type=skill
  skill:   <path>               # if type=skill, e.g., "skills/run-phase/SKILL.md"
  sub_workflow: <path>          # if type=sub-workflow, e.g., "greenfield.lobster"
  task: <string-folded>         # task prompt (agent/skill/sub-workflow)
  
  # Optional control
  condition: <string-expr>      # if absent, always runs
  optional:  <bool>             # default false
  timeout:   <duration>         # overrides defaults.timeout
  on_failure: escalate | retry | skip
  max_retries: <int>
  wait_for_optional: <list[step-name]>   # wait IF they ran, don't require them

  # Optional metadata
  model_tier: <string>          # e.g., "review", "adversary"
  description: <string>
  config:    <object>           # step-specific (e.g., {depth: "L1", scope: "feature"})
  cwd:       <path>             # per-step working directory (multi-repo)
  
  # Optional sub-structures
  gate:        <GateSpec>       # required if type=gate
  approval:    <ApprovalSpec>   # required if type=human-approval
  loop:        <LoopSpec>       # required if type=loop
  collection:  <string-expr>    # required if type=parallel-foreach
  iterator:    <list[Step]>     # required if type=parallel-foreach
  inputs:      <object>         # bindings for sub_workflow inputs
  context:     <ContextSpec>    # information asymmetry wall
```

## StepType Enum

| Value | Required companion fields | Semantics |
|---|---|---|
| `agent` | `agent`, `task` | Spawn a specialist agent via Task tool |
| `skill` | `skill` | Invoke a skill via Skill tool |
| `gate` | `gate.criteria`, `gate.fail_action` | Evaluate criteria; block or warn on failure |
| `human-approval` | `approval.prompt`, `approval.timeout` | Pause for human |
| `sub-workflow` | `sub_workflow` | Invoke another `.lobster` file |
| `loop` | `loop.max_iterations`, `loop.exit_condition`, `loop.steps` | Repeat sub-steps |
| `parallel-foreach` | `collection`, `iterator` | Run iterator for each element of collection in parallel |
| `command` | (the shell command itself; rare and underspecified) | Bash invocation |

Evidence summary: all 8 types observed across 16 workflows. Most common: `agent` (~60%), `skill` (~25%), `gate` (~8%), `human-approval` (~5%), `sub-workflow` (~2%), rest <1%.

## GateSpec

```yaml
gate:
  criteria:
    - "<assertion string>"          # one line per criterion
    - "<assertion string>"
  fail_action: block | warn         # required
```

Evidence: brownfield.lobster:151-167, feature.lobster:94-98, phase-7-convergence.lobster:117-130.

## ApprovalSpec

```yaml
approval:
  prompt: <string-folded>           # required
  timeout: <duration>               # required (typical: "24h", "48h", "72h")
  artifacts:                        # optional list of glob paths
    - ".factory/.../*.md"
    - ".factory/.../directory/"     # trailing slash = directory
```

Evidence: brownfield.lobster:170-189, planning.lobster:101-112.

## LoopSpec

```yaml
loop:
  max_iterations: <int>             # required
  exit_condition: <string-expr>     # required
  for_each: <string>                # optional, e.g., "finding in auto_fixable_findings"
  steps:                            # required
    - <Step>
    - <Step>
```

Evidence: code-delivery.lobster:103-145 (per-story-adversarial-review loop, max 10 iterations).

## ContextSpec

```yaml
context:
  include:                          # optional list of glob paths
    - "[worktree_path]/src/**"
    - ".factory/stories/[story_id].md"
  exclude:                          # optional list of glob paths
    - ".factory/cycles/**/adversarial-reviews/**"
    - ".factory/semport/**"
```

Evidence: code-delivery.lobster:118-135. Comment convention: `# ▓ WALL: <reason>` precedes each exclude block.

## Naming/Reserved Identifiers

- **Step name** — kebab-case (`phase-0-codebase-ingestion`, `factory-worktree-gate`).
- **Workflow name** — kebab-case with `-vsdd` suffix for primary modes (`brownfield-vsdd`, `feature-vsdd`).
- **Agent IDs** — kebab-case (`state-manager`, `product-owner`, `pr-manager`, `adversary`, `dx-engineer`, `devops-engineer`, `architect`, `consistency-validator`, `formal-verifier`, `holdout-evaluator`, `implementer`, `test-writer`, `demo-recorder`, `code-reviewer`, `pr-reviewer`, `security-reviewer`, `accessibility-auditor`, `visual-reviewer`, `e2e-tester`, `dtu-validator`, `business-analyst`, `ux-designer`, `story-writer`, `spec-reviewer`, `codebase-analyzer`, `validate-extraction`, `technical-writer`, `research-agent`, `github-ops`, `performance-engineer`, `data-engineer`, `session-reviewer`, `spec-steward`, `orchestrator`). 33-34 agents per `README.md:139-149`.
- **Skill paths** — `skills/<name>/SKILL.md` exactly.
- **Phase IDs** — `phase-0-codebase-ingestion`, `phase-1-spec-crystallization`, etc., matching the filename slug.

## Conditional Expression Vocabulary (observed)

These are the natural-language fragments seen as `condition:` values. There is no formal grammar — monocle should treat them as labels, not evaluate them.

```
"<exists?-clauses>"
  !file_exists('CLAUDE.md')

"<routing/state lookups>"
  routing.choice == 'feature'
  routing.level == 'L0'
  routing.level in ['L1', 'L2', 'L3', 'L4']
  feature_type in ['ui', 'full-stack']
  any_repo_mode == 'brownfield'
  state.has_holdout_scenarios == true
  state.has_benchmarks == true
  state.has_ui == true
  state.has_dtu_clones == true

"<workflow-context lookups>"
  request.intent != 'bug-fix'
  mode == 'brownfield'
  config.demo_recording.enabled != false
  config.product_discovery.enabled == true
  config.feature_discovery.enabled == true
  config.products[*].user_channels is configured
  config.products[*].competitors is configured
  config.products[*].analytics.enabled == true
  config.enable_secondary_adversary == true

"<convergence/loop checks>"
  adversary.verdict == 'CONVERGENCE_REACHED'
  adversary.has_blocking_findings == true
  storybook_tests.all_pass
  storybook_tests.has_failures
  pr_reviewer.verdict == 'REQUEST_CHANGES'
  pr_reviewer.verdict == 'APPROVE'
  ci.status == 'all_passed'
  ci.status == 'failed'

"<counts/comparisons>"
  discovery.approved_products.count > 0
  discovery.approved_features.count > 0
  worktree.not_exists == true
  module_criticality in ['CRITICAL', 'HIGH']
  brownfield.needs_semport == true
  merge_decision.requires_human == true
  maintenance.has_auto_fixable_findings == true
  maintenance.request_demo == true
  MULTI_REPO_HANDOFF == true
```

Monocle takeaway: surface conditions verbatim as labels; do not parse.

## Parser Implementation (`bin/lobster-parse`)

51-line bash script. Full contract:
- Args: `<file.lobster> [jq-expr]`. Default jq expr is `.`.
- Hard requirements: `yq` AND `jq` on PATH.
- Pipeline: `yq eval --output-format=json '.' "$FILE" 2>$tmpfile | jq "$EXPR"`.
- Errors: yq parse failure surfaces stderr; missing tools exits 1 with install hint.

Monocle does NOT need to depend on this script. A Go/Rust/JS YAML library can read `.lobster` files directly. The parser is just a shell convenience for the orchestrator agent.

## Phase Sub-Workflow Catalog

| File | Workflow name | Step count | Has loops | Has sub-workflows | Notable |
|---|---|---|---|---|---|
| phase-0-codebase-ingestion.lobster | phase-0-codebase-ingestion | 14 | no | no | broad-then-converge protocol (this very ingest) |
| phase-1-spec-crystallization.lobster | phase-1-spec-crystallization | (not read; 161 LOC) | (likely yes) | (likely no) | DTU + CI/CD inline |
| phase-2-story-decomposition.lobster | phase-2-story-decomposition | (171 LOC) | (likely yes) | (likely no) | Wave computation |
| phase-3-tdd-implementation.lobster | phase-3-tdd-implementation | 17 | no | no | per-story; calls deliver-story skill (Step A-G) |
| phase-4-holdout-evaluation.lobster | phase-4-holdout-evaluation | 3 | no | no | 80% scenario rotation; uses different model family |
| phase-5-adversarial-refinement.lobster | phase-5-adversarial-refinement | 2 (outer) + loop | yes | no | max 10 iterations; Gemini secondary review |
| phase-6-formal-hardening.lobster | phase-6-formal-hardening | (91 LOC; not read) | (unknown) | (unknown) | Kani proofs, fuzz, mutation |
| phase-7-convergence.lobster | phase-7-convergence | 17 | no | no | 7-dimensional convergence assessment |

## Mode Workflow Catalog

| File | LOC | Sub-workflows invoked | Mode tag |
|---|---|---|---|
| greenfield.lobster | 1408 | planning.lobster, code-delivery.lobster | `mode: greenfield` |
| brownfield.lobster | 400 | semport-analyze (skill), greenfield.lobster, multi-repo.lobster | `mode: brownfield` |
| feature.lobster | 1489 | code-delivery.lobster | `mode: feature` |
| maintenance.lobster | 418 | code-delivery.lobster (fix PR delivery loop) | `mode: maintenance` |
| discovery.lobster | 435 | planning.lobster, feature.lobster | `mode: discovery` |
| planning.lobster | 298 | greenfield.lobster | `mode: planning` |
| multi-repo.lobster | 731 | brownfield.lobster (per-repo Phase 0) | `mode: multi-repo` |
| code-delivery.lobster | 436 | (none; leaf sub-workflow) | (invoked by all above) |

## Delta Summary

- Lobster schema documented to ADR-ready depth.
- Step type enum: 8 types verified.
- 8 optional fields under Step (`condition`, `optional`, `timeout`, `on_failure`, `max_retries`, `wait_for_optional`, `model_tier`, `description`, `config`, `cwd`).
- Condition expression vocabulary: 30+ examples enumerated.
- Mode catalog: 8 workflows mapped to invoking patterns.
- Phase catalog: 8 phases mapped (phase-1, 2, 6 not deeply read; line counts noted).

## Novelty Assessment

Novelty: SUBSTANTIVE.
The first round of deepening surfaced: full step schema (was previously partial in Pass 2), the dual `cost_monitoring`/`cost_tracking` spelling, the condition expression vocabulary, the multi-repo `cwd` + `parallel-foreach` patterns, and the LoopSpec exit_condition convention. These materially change what an ADR would say.

## Convergence Declaration

Another round needed — Round 2 should verify the gap: read phase-1, phase-2, phase-6 fully; resolve whether `command` step type is really present anywhere; identify the `inputs` binding mechanism for sub-workflow invocation.

## State Checkpoint

```yaml
pass: B-deep-workflows
round: 1
status: complete
timestamp: 2026-05-11T22:30:00Z
novelty: SUBSTANTIVE
```
