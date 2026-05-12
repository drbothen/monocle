# Pass 6 — Scoped Conventions & Patterns: vsdd-factory

Conventions monocle MUST recognize when reading factory artifacts.

## File-Naming Conventions

| Pattern | Where | Meaning |
|---|---|---|
| `*.lobster` | `workflows/` and `workflows/phases/` | Lobster workflow file (YAML) |
| `phase-N-<slug>.lobster` | `workflows/phases/` | Numbered phase sub-workflow |
| `<mode>.lobster` | `workflows/` | Mode-level workflow |
| `STATE.md` | `.factory/` (and `.factory-project/` for multi-repo) | Live pipeline state |
| `SESSION-HANDOFF.md` | `.factory/` (optional) | Inter-session handoff |
| `wave-state.yaml` | `.factory/` | Wave lifecycle tracker |
| `policies.yaml` | `.factory/` (optional) | Project policy rubric |
| `cycle-manifest.md` | `.factory/cycles/<id>/` | Per-cycle manifest |
| `burst-log.md` | `.factory/cycles/<id>/` | Per-cycle burst narrative |
| `pass-N.md` | `.factory/cycles/<id>/adversarial-reviews/` | Adversarial pass N report |
| `events-*.jsonl` | `.factory/logs/` | Dispatcher event log (file sink) |
| `*-INDEX.md` | various | Sharded index (BC-INDEX, VP-INDEX, STORY-INDEX, HS-INDEX, ARCH-INDEX, L2-INDEX, EVAL-INDEX, ADV-P[N]-INDEX) |
| `BC-S.SS.NNN` | content identifiers | 4-level Behavioral Contract ID |
| `VP-NNN` | content identifiers | Verification Property ID |
| `STORY-NNN` | content identifiers | Story ID |
| `HS-NNN` | content identifiers | Holdout Scenario ID |
| `FR-NNN` | LEGACY content identifiers | Old PRD requirement ID (flagged for migration to BC-S.SS.NNN) |
| `TD-VSDD-NNN` | content identifiers | Tech debt register entry |
| `D-NNN` | content identifiers | Decision log entry |
| `R-NNN` | content identifiers | Risk register entry |
| `ASM-NNN` | content identifiers | Assumption identifier |
| `CAP-NNN` | content identifiers | L2 Domain Capability ID |
| `AC-NNN` | story content | Acceptance Criterion |
| `FIX-P[N]-NNN` | content identifiers | Fix PR identifier from phase N |
| `vX.Y.Z-<type>-<name>` | cycle directories | Cycle ID |
| `release/v<full-semver>` | git branches | Release branch (MUST target main) |

## Identifier Format Hierarchy

```
L1 Product Brief → product-brief.md (no ID; one per project)
L2 Domain Spec   → CAP-NNN (capability) under L2-INDEX.md
L3 Behavioral    → BC-S.SS.NNN (Section.Subsection.Number) under BC-INDEX.md
L4 Verification  → VP-NNN under VP-INDEX.md
Story-level      → STORY-NNN with AC-NNN children
```

## YAML Frontmatter Conventions

All persisted markdown artifacts use YAML frontmatter. Common fields:

| Field | Type | Meaning |
|---|---|---|
| `document_type` | string | Artifact type (`pipeline-state`, `cycle-manifest`, `burst-log`, etc.) |
| `level` | string | `ops`, `spec`, `impl`, etc. |
| `version` | semver string | Artifact version |
| `status` | string | `draft`, `in_progress`, `complete`, `blocked` |
| `producer` | string | Agent ID that produced it |
| `timestamp` | ISO8601 | Last update |
| `inputs` | list | Source artifacts |
| `input-hash` | string (MD5) | Hash of inputs (drift detection) |
| `traces_to` | string | Parent artifact reference |
| `project` | string | Project name |
| `mode` | string | Pipeline mode |
| `phase` | int | Phase number |
| `current_step` | string | Current step name |
| `current_cycle` | string | Active cycle ID |
| `dtu_required` | bool | DTU clones required |
| `origin` | string | `recovered` (brownfield) or `authored` (greenfield) |

## Step Type Catalog

| Type | Required fields | Optional fields |
|---|---|---|
| `agent` | `name`, `type`, `agent`, `task`, `depends_on` | `model_tier`, `context`, `condition`, `timeout`, `on_failure`, `max_retries`, `optional` |
| `skill` | `name`, `type`, `skill`, `depends_on` | `agent`, `condition`, `config`, `inputs`, `timeout`, `description` |
| `gate` | `name`, `type`, `depends_on`, `gate.criteria`, `gate.fail_action` | — |
| `human-approval` | `name`, `type`, `depends_on`, `approval.prompt`, `approval.timeout` | `approval.artifacts`, `condition` |
| `sub-workflow` | `name`, `type`, `sub_workflow`, `depends_on` | `condition`, `inputs`, `cwd`, `task`, `wait_for_optional` |
| `loop` | `name`, `type`, `depends_on`, `loop.max_iterations`, `loop.exit_condition`, `loop.steps` | `loop.for_each`, `condition` |
| `parallel-foreach` | `name`, `type`, `depends_on`, `collection`, `iterator` | `condition` |
| `command` | `name`, `type`, `depends_on`, the command itself | (rare, schema not fully canonical) |

## Path Conventions

| Path | Owner | Read/Write |
|---|---|---|
| `.factory/` | state-manager only writes | Monocle: read-only |
| `.factory-project/` | state-manager (multi-repo only) | Monocle: read-only |
| `.worktrees/STORY-NNN/` | implementer (per-story worktrees) | Monocle: should NOT read code; PR reviewer info-asymmetry wall |
| `${CLAUDE_PLUGIN_ROOT}/` | plugin-resident | Monocle: read-only reference (template/workflow lookups) |
| `${CLAUDE_PROJECT_DIR}/` | project root (capability root) | Monocle: project root |
| `docs/demo-evidence/<STORY-ID>/` | demo-recorder | Surface in session view |

## Naming Conventions

- **Workflows** use kebab-case names ending in a domain suffix: `brownfield-vsdd`, `feature-vsdd`, `per-story-delivery`, `phase-0-codebase-ingestion`.
- **Phase sub-workflows** use the prefix `phase-N-` literally.
- **Skill directories** are kebab-case under `skills/<name>/` with `SKILL.md` inside.
- **Agent files** use either `agents/<name>.md` or `agents/<name>/<name>.md`.
- **Workflow versions** use semver (`"3.0.0"`, `"2.1.0"`, `"2.0.0"`).
- **Plugin manifest version** uses semver with optional pre-release tag: `"1.0.0-rc.16"`.

## Patterns

### P1. Sub-workflow Composition

Top-level mode workflows invoke sub-workflows via `type: sub-workflow`:
- `brownfield.lobster` → `greenfield.lobster` (after Phase 0)
- `brownfield.lobster`/`greenfield.lobster`/`feature.lobster`/`maintenance.lobster` → `code-delivery.lobster` (per-story)
- `discovery.lobster` → `planning.lobster` (approved product ideas) → `greenfield.lobster`
- `planning.lobster` → `greenfield.lobster`
- `multi-repo.lobster` → `brownfield.lobster` (per-repo Phase 0)

### P2. State-Manager Sandwich

Almost every "real work" step is followed by a `state-manager` step that commits artifacts. E.g., `phase-0-codebase-ingestion.lobster:18-30` shows the recurring pattern:
```yaml
- name: source-acquisition
  type: skill
  ...
- name: backup-source-acquisition
  type: agent
  agent: state-manager
  depends_on: [source-acquisition]
  task: > Commit source acquisition artifacts...
```

### P3. Gate + Human-Approval Pair

End of each phase: a `gate` step (criteria check) followed by a `human-approval` step.

### P4. Information Asymmetry Wall Block

```yaml
context:
  include:
    - "<files-the-agent-needs>"
  exclude:
    # ▓ WALL: <wall name>
    - ".factory/cycles/**/adversarial-reviews/**"
    - ".factory/semport/**"
```

The `▓ WALL:` comment marker is used consistently (`code-delivery.lobster:127, 129, 132, 134`).

### P5. Cost Monitoring Inline

```yaml
cost_monitoring:
  enabled: true
  metadata: {mode: feature}
  thresholds: {warn: 0.70, pause: 0.95}
  protected_agents: [adversary, security-reviewer, formal-verifier]
```

Or:
```yaml
cost_tracking:
  enabled: true
  metadata: {mode: feature, feature: "${feature.name}", phase: "${current_phase}"}
  summary_artifact: ".factory/feature/cost-summary.md"
```

(Both spellings appear: `cost_monitoring` in most workflows, `cost_tracking` in feature.lobster:62. Monocle should accept either.)

### P6. Conditional Execution

`condition` is a string expression evaluated by the orchestrator/runner. Examples:
- `"!file_exists('CLAUDE.md')"`
- `"routing.choice == 'feature'"`
- `"feature_type in ['ui', 'full-stack']"`
- `"state.has_holdout_scenarios == true"`
- `"any_repo_mode == 'brownfield'"`
- `"discovery.approved_products.count > 0"`

There is NO documented expression grammar; the convention is "natural-looking boolean expression". Monocle should NOT attempt to evaluate these — only surface them as labels.

### P7. The `${VAR}` interpolation

Some fields contain `${...}` placeholders:
- `${CLAUDE_PLUGIN_ROOT}` — installed plugin root
- `${CLAUDE_PROJECT_DIR}` — current project root
- `${feature.name}` — workflow context variable
- `${current_phase}` — runtime variable
- `${repo.name}` — iterator variable (multi-repo)

Monocle should NOT interpolate these; they are runtime concerns.

## Anti-Patterns Observed (and avoided in latest version)

| Anti-pattern | Mitigation |
|---|---|
| Two-commit STATE.md burst | Retired (TD-VSDD-053). Single-commit only. |
| Self-citing factory-artifacts HEAD SHA | Retired. Use `git -C .factory log -1` instead. |
| Outcome-presumptive language ("Pass N — 1st of 3 required clean passes") | Use outcome-neutral ("if CLEAN…if BLOCKED…"). |
| Past-tense for in-progress work | Use "REMEDIATED" voice from burst start. |
| Tense-flip after Stage 1 commit | Obviated by single-commit rule. |
| `backfill` token in two consecutive commits | Blocked by `verify-sha-currency.sh` (`MULTI_COMMIT_CHAIN_NOT_ALLOWED`). |
| Compound phase values (`2-story-decomposition-patch-cycle`) | Flagged by check-state-health; phase MUST be 0-7 integer. |
| Multi-burst story creation > 8 artifacts | Split into create+integrate sub-bursts (`orchestrator.md:130-131`). |

## Markdown Table Discipline

Per the user instruction "Markdown table cell counts MUST match header" and the `validate-table-cell-count.sh` hook (`hooks/validate-table-cell-count.sh`, listed in Pass 1): every table must have header-count = body-row-cell-count.

I confirm every table in this pass set has been authored with that discipline.

## State Checkpoint

```yaml
pass: 6
status: complete
patterns_cataloged: 7
identifier_formats: 13
timestamp: 2026-05-11T22:22:00Z
next_pass: 7-synthesis
```
