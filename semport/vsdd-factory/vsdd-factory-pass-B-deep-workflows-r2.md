# Phase B Round 2 — Deep: Workflows (Lobster Format)

## Round 1 Gaps Addressed

Round 1 noted three open gaps. All now resolved:

### Gap 1: Was `command` step type actually used anywhere?

Searched all 16 `.lobster` files. The `type: command` step type is **declared in `run-phase/SKILL.md:46`** as a supported step type ("If `type: command`, run the declared command via Bash"), but **NOT used** in any of the 16 in-tree workflows. **Status: declared-but-unused**. Monocle should accept it as a valid type but not expect to see it commonly.

### Gap 2: How is the `inputs` binding mechanism for sub-workflow invocation expressed?

Examined `code-delivery.lobster:21-26` (the only sub-workflow that declares inputs):

```yaml
inputs:
  - story_id: "STORY-NNN"
  - worktree_path: ".worktrees/STORY-NNN"
  - feature_type: "ui | backend | full-stack | infrastructure"
  - module_criticality: "CRITICAL | HIGH | MEDIUM | LOW"
  - implementation_strategy: "tdd | gene-transfusion"
```

The `inputs:` block at workflow-level is a **declaration** — typed inputs with example values. The example string serves as both a type hint and a default placeholder.

Sub-workflow **invocation** uses an `inputs:` block at step level:

```yaml
- name: deliver-fix
  type: sub-workflow
  sub_workflow: "code-delivery.lobster"
  inputs:
    story_id: "FIX-MAINT-finding.id"
    module_criticality: "finding.severity"
```

Evidence: `maintenance.lobster:325-332`. The invoker provides a name→value mapping (values may be string-interpolated `${...}` style or expression-style).

Note inconsistency: workflow-level inputs use a **list-of-maps** form (each list item is a one-entry map), while step-level inputs use a **single map**. Monocle should accept both shapes.

### Gap 3: Resolution of phase-1, phase-2, phase-6 step counts

Now read and verified:

| File | Total steps | Outermost (excluding loop bodies) | Gates | Human-approvals | Loops |
|---|---|---|---|---|---|
| phase-1-spec-crystallization.lobster | 13 outer (+2 loop body) | 13 | 1 | 1 | 1 (max 10 iter) |
| phase-2-story-decomposition.lobster | 13 outer (+2 loop body) | 13 | 1 | 1 | 1 (max 10 iter) |
| phase-3-tdd-implementation.lobster | 17 | 17 | 1 | 0 | 0 |
| phase-4-holdout-evaluation.lobster | 3 | 3 | 1 | 0 | 0 |
| phase-5-adversarial-refinement.lobster | 2 outer (+2 loop body) | 2 | 0 | 0 | 1 |
| phase-6-formal-hardening.lobster | 9 | 9 | 1 | 0 | 0 |
| phase-7-convergence.lobster | 17 | 17 | 1 | 1 | 0 |
| phase-0-codebase-ingestion.lobster | 14 | 14 | 1 | 1 | 0 |

## New Observations (Round 2)

### O-R2-1: Phase 1 and Phase 2 share an identical adversarial-loop pattern

Both `phase-1-spec-crystallization.lobster:108-145` and `phase-2-story-decomposition.lobster:109-145` use the same `type: loop` block with `max_iterations: 10`, `exit_condition: "adversary.verdict == 'CONVERGENCE_REACHED'"`, and a spawn-adversary + fix-findings sub-step pair. Phase 5 (`phase-5-adversarial-refinement.lobster:14-39`) also uses this pattern.

This is a **canonical convergence-loop template** monocle can recognize:
- Outer step name pattern: `adversarial-<scope>-review`
- Inner steps: `spawn-adversary-<scope>` (agent: adversary, model_tier: adversary) + `fix-<scope>-findings` (skill)
- Termination: 10 iterations max OR verdict `CONVERGENCE_REACHED`

When monocle surfaces "what's running now", recognizing this loop pattern lets it report "Phase N adversarial pass M of 10" rather than the raw step name.

### O-R2-2: Input-hash drift check is a universal pre-gate skill

Every phase EXCEPT 4, 5 ends with a `check-input-drift` skill step **immediately before** human-approval. This is essential to surface as a status:
- phase-0-codebase-ingestion.lobster:128 (`input-hash-drift-check`)
- phase-1-spec-crystallization.lobster:138
- phase-2-story-decomposition.lobster:149
- phase-3-tdd-implementation.lobster:153 (after gate, no human-approval in this phase file)
- phase-7-convergence.lobster:135

Monocle: if a phase's status shows `input-hash-drift-check` in flight or `DRIFT` items present, surface that as a blocker label.

### O-R2-3: STATE.md update message format is canonical

All `state-manager` backup steps use a near-identical phrasing:
```
Update STATE.md: phase: <N>, step: <slug>, status: <complete|in-progress|blocked>.
```

Sometimes adds `story: STORY-NNN` for phase-3. Monocle can use this format as a regex/pattern to extract phase/step from STATE.md "Current Phase Steps" table entries.

### O-R2-4: Gate criteria are free-form assertions

Gate criteria are NOT machine-evaluable. They're operator-facing English sentences like:
- "Tests compile"
- "All tests fail (Red Gate)"
- "Holdout evaluator used different model family (GPT-5.4, not Claude)"
- "Mean satisfaction score >= 0.85"
- "No must-pass scenario below 0.6 satisfaction"

Some include numerical thresholds, but the dispatcher doesn't parse them — they're for humans + criteria-evaluator agents. Monocle: surface verbatim as a checklist.

### O-R2-5: A `task:` field can describe the work for both `type: agent` AND `type: skill`

Most `type: skill` steps OMIT `task`. But some include it (e.g., `multi-repo.lobster:160-173` — the `task` provides additional context for the skill invocation). Monocle should treat `task` as optional decoration on skill steps.

### O-R2-6: `condition` can reference filesystem existence

`phase-1-spec-crystallization.lobster:82`: `condition: "exists('.factory/specs/architecture-feasibility-report.md')"`.

Two variants of the same idea seen across the corpus: `file_exists('path')` (negation `!file_exists(...)`) and `exists('path')`. Inconsistent. Monocle surfaces verbatim; does not normalize.

### O-R2-7: Sub-workflow `inputs` for code-delivery includes feature_type enum

Code-delivery accepts `feature_type` from the invoking workflow with documented enum: `ui | backend | full-stack | infrastructure`. Information-asymmetry walls + Storybook step inclusion vary based on this value. Monocle: surface the feature_type for the active code-delivery instance.

## Convergence Loop Termination Vocabulary

| Token | Meaning |
|---|---|
| `CONVERGENCE_REACHED` | adversary returned cosmetic-only findings |
| `APPROVE` | pr-reviewer/human approved |
| `REQUEST_CHANGES` | pr-reviewer flagged blockers |
| `BLOCKED` | gate failed |
| `CLEAN` | adversarial pass produced zero findings |
| `REMEDIATED` | findings closed in this burst |
| `pass N+1_pending` | next clean window pending |

Monocle can use these tokens as status labels.

## Workflow Self-Documentation Pattern

Every `.lobster` file declares an extensive `description: >` (folded scalar) at the top, summarizing:
- Purpose
- DF-XXX upgrade tags (e.g., "DF-031 upgrades", "DF-035 additions", "DF-037 D11/D18")
- Routing variations
- Crash recovery semantics

These tags reference **internal RFC/upgrade documents** that monocle doesn't have access to. Monocle should NOT try to interpret `DF-XXX` tags — just preserve them in any surfaced description.

## Delta Summary

- 3 Round 1 gaps closed.
- 7 new observations on canonical patterns.
- Confirmed: `command` step type exists in skill docs but is unused.
- Confirmed: `inputs` mechanism is dual-shape (declaration vs invocation).
- Confirmed: convergence loop is a recognizable template (3 phases use it identically).

## Novelty Assessment

Novelty: SUBSTANTIVE — but at the boundary.
The new observations (convergence-loop template recognition, input-hash drift as universal pre-gate, `exists()` vs `file_exists()` inconsistency) refine but don't fundamentally change the schema. The schema itself is complete after Round 1. Round 3 is unlikely to produce substantive additions.

## Convergence Declaration

One more round (Round 3) to confirm NITPICK on edge details, then converge.

## State Checkpoint

```yaml
pass: B-deep-workflows
round: 2
status: complete
gaps_closed: 3
new_observations: 7
timestamp: 2026-05-11T22:38:00Z
novelty: SUBSTANTIVE
```
