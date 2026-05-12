# Phase B Round 3 — Deep: Workflows (NITPICK round)

Round 3 hunts for last-mile details and validates convergence.

## Edge Cases Re-Verified

### EC-1. `depends_on: []` vs missing `depends_on`

`run-phase/SKILL.md:36` says "For each step, resolve `depends_on` and execute topologically." `validate-workflow/SKILL.md:21-23` requires every `depends_on` entry to name an earlier step.

Examining files: `phase-0-codebase-ingestion.lobster:21` shows `depends_on: []` explicitly. `code-delivery.lobster:42` also uses `[]`. Some steps omit `depends_on` entirely (e.g., `phase-1-spec-crystallization.lobster:21` for `create-brief`).

Conclusion: empty list and missing field are both valid — both mean "no predecessors". Monocle should normalize to empty list internally.

### EC-2. Mixed `depends_on` semantics with `wait_for_optional`

`brownfield.lobster:301-313` shows:
```yaml
- name: brownfield-to-greenfield-transition
  type: agent
  agent: state-manager
  depends_on: [brownfield-market-review]
  wait_for_optional: [semport-validation-gate, brownfield-design-system-approval]
  ...
```

Semantics: `depends_on` is hard (must succeed), `wait_for_optional` is soft (wait IF they ran, don't require). Monocle: do not collapse these.

### EC-3. The `optional: true` step modifier

`greenfield.lobster:75`, `brownfield.lobster:107`:
```yaml
- name: scaffold-claude-md
  type: skill
  skill: "skills/scaffold-claude-md/SKILL.md"
  depends_on: [factory-worktree-gate]
  condition: "!file_exists('CLAUDE.md')"
  optional: true
```

`optional: true` is a step-level flag meaning "skip if it fails". Different from `condition` (which means "don't even start"). Monocle: surface optional steps with a visual hint that failure won't block.

### EC-4. `task:` may be omitted on `type: skill` even when the skill normally accepts parameters

Most `type: skill` steps omit `task` because the skill carries its own contract. But some include `task` to override or specialize behavior. Monocle: do not assume `task` is always present.

### EC-5. Cost block fields are inconsistent

| Field | feature.lobster | greenfield.lobster | discovery.lobster |
|---|---|---|---|
| Top-level key | `cost_tracking` | `cost_monitoring` | `cost_monitoring` |
| `enabled` | yes | yes | yes |
| `metadata.mode` | yes | yes | yes |
| `metadata.feature` | yes | no | no |
| `metadata.phase` | yes | no | no |
| `metadata.wave` | yes | no | no |
| `thresholds.warn/pause` | yes | no | no |
| `protected_agents` | yes | no | no |
| `summary_artifact` | yes | no | no |

The richer block is in feature.lobster. Monocle: render whatever fields are present; do not require any specific subset.

### EC-6. Description block uses YAML folded scalar (`>`)

All workflow `description:` and many `task:` fields use `>` (folded scalar — newlines collapse to spaces). Some use `|` (literal). YAML semantics differ. Monocle's parser should rely on the YAML lib to handle both correctly.

### EC-7. The `version` field on workflows is independent of artifact versions

Workflow `version: "3.0.0"` describes the WORKFLOW DEFINITION's version (semantic semver of the file's schema/contract), not the project being built. Monocle: do not confuse with `STATE.md frontmatter version` (artifact version of STATE.md itself).

### EC-8. Phase 3 `phase-3-tdd-implementation.lobster` does NOT contain a human-approval step

Verified: 0 human-approvals. Phase 3 runs per-story and the human approval (if any) happens at wave-gate level, not per-story. Monocle: should not surface Phase 3 as "awaiting human" unless the wave-gate is active.

### EC-9. Some workflows have BOTH a `gate` AND a `human-approval` at the end

Phase 0, 1, 2, 7 — each has a gate followed by human-approval. The pattern is: gate validates machine-checkable criteria, human-approval validates judgment calls.

### EC-10. The orchestrator's "step" tracking is by step `name`, not position

Two workflows can have steps with the same name only if they're in different workflows. Within a workflow, names are unique (per `validate-workflow/SKILL.md:25`). Monocle: STATE.md `current_step` is just the step name (no namespace prefix observed).

## Reading-Path Audit

`bin/lobster-parse` is bash-only. The contract is simple enough that monocle can:
1. Use a YAML lib in its native language (Go, Rust, JS, Python — all have mature YAML parsers).
2. Directly read the JSON-equivalent shape.
3. NOT depend on shelling out to `yq`/`jq`/`lobster-parse`.

This is a deliberate decoupling: monocle should treat `.lobster` files as portable YAML, not as a vsdd-factory-specific format.

## Workflow Parsing Pseudocode (for monocle)

```python
def load_workflow(path):
    raw = yaml.safe_load(read(path))
    wf = raw['workflow']
    return Workflow(
        name=wf['name'],
        description=wf.get('description', ''),
        version=wf['version'],
        defaults=wf.get('defaults', {}),
        cost=wf.get('cost_monitoring') or wf.get('cost_tracking'),
        schedule=wf.get('schedule'),
        state_fields=wf.get('state_fields', []),
        inputs=wf.get('inputs', []),
        steps=[parse_step(s) for s in wf['steps']],
    )

def parse_step(s):
    return Step(
        name=s['name'],
        type=s['type'],            # enum
        depends_on=s.get('depends_on') or [],
        agent=s.get('agent'),
        skill=s.get('skill'),
        sub_workflow=s.get('sub_workflow'),
        task=s.get('task'),
        condition=s.get('condition'),
        optional=s.get('optional', False),
        timeout=s.get('timeout'),
        on_failure=s.get('on_failure'),
        max_retries=s.get('max_retries'),
        wait_for_optional=s.get('wait_for_optional', []),
        model_tier=s.get('model_tier'),
        description=s.get('description'),
        config=s.get('config', {}),
        cwd=s.get('cwd'),
        gate=s.get('gate'),
        approval=s.get('approval'),
        loop=s.get('loop'),
        collection=s.get('collection'),
        iterator=s.get('iterator'),
        inputs=s.get('inputs'),
        context=s.get('context'),
    )
```

## Delta Summary

- 10 edge cases re-verified.
- Workflow parsing pseudocode produced (ADR-grade).
- Confirmed: monocle should use native YAML; not depend on bash + yq + jq.

## Novelty Assessment

Novelty: NITPICK.
This round produced no new structural findings — only edge case confirmations, mild inconsistency notes (`exists()` vs `file_exists()`, `cost_monitoring` vs `cost_tracking`), and a parsing pseudocode that codifies what Round 1+2 already documented. The schema is settled.

## Convergence Declaration

Pass B (workflows) has converged — findings are nitpicks, not gaps.

## State Checkpoint

```yaml
pass: B-deep-workflows
round: 3
status: complete
timestamp: 2026-05-11T22:43:00Z
novelty: NITPICK
rounds_total: 3
```
