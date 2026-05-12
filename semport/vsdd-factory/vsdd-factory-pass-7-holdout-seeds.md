# Pass 7 — Scoped Holdout Seeds: vsdd-factory

Holdout scenarios are hidden test scenarios used by Phase 4. For monocle's purposes, this pass instead enumerates the **test cases monocle must pass to validate its factory-awareness implementation**.

## H1. Detect a vsdd-factory project (positive)

**Given** a project directory containing `.factory/STATE.md` with frontmatter `document_type: pipeline-state`
**When** monocle opens the session
**Then** monocle MUST emit signal `is_factory_project = true` and surface mode + phase

**Source**: `templates/state-template.md:1-17`.

## H2. Detect a vsdd-factory project (multi-repo)

**Given** a project directory containing `.factory-project/STATE.md` AND per-repo `.factory/STATE.md`
**When** monocle opens the session
**Then** monocle MUST recognize the multi-repo coordination layer AND surface project-level state in addition to per-repo state

**Source**: `templates/factory-project-state-template.md`, `multi-repo.lobster:108-122`.

## H3. Detect a non-factory project (negative)

**Given** a project with NO `.factory/` directory
**When** monocle opens the session
**Then** monocle MUST NOT claim it is a factory project

## H4. Parse a Lobster workflow file

**Given** a workflow file at `${CLAUDE_PLUGIN_ROOT}/workflows/brownfield.lobster`
**When** monocle reads it via the same parse path the orchestrator uses
**Then** monocle MUST extract `workflow.name`, `workflow.version`, `workflow.steps[].name`, and `workflow.steps[].depends_on`

**Source**: `bin/lobster-parse:31-51`.

## H5. Recognize sub-workflow composition

**Given** `brownfield.lobster` invokes `greenfield.lobster` via `type: sub-workflow`
**When** monocle reports "current workflow"
**Then** monocle MUST be able to report BOTH the parent and the child workflow names if execution is inside a sub-workflow

**Source**: `brownfield.lobster:335-339`.

## H6. Surface wave state

**Given** `.factory/wave-state.yaml` exists with `current_wave: wave_2` and `wave_1.gate_status: passed`
**When** monocle renders the dashboard
**Then** monocle MUST surface: current wave, last passed wave, next gate (if pending)

**Source**: `wave-state-template.yaml:1-29`.

## H7. Block on stale wave gate

**Given** wave N's gate status is `pending` (stories merged, gate not run)
**When** monocle surfaces "what's next"
**Then** monocle MUST flag the pending gate as a blocker BEFORE proposing N+1 work

**Source**: `wave-state-template.yaml:6-7` (validate-wave-gate-prerequisite.sh).

## H8. Surface a recent hook event

**Given** `.factory/logs/events-2026-05-11.jsonl` contains a recent `plugin.completed` event
**When** monocle's "recent activity" panel is requested
**Then** monocle MUST include the event (timestamp, plugin name, outcome)

**Source**: `factory-dashboard/SKILL.md:30-35`.

## H9. Surface drift items

**Given** STATE.md has a `Drift Items` table (referenced in `orchestrator.md:401-405`) OR `tech-debt-register.md` exists
**When** monocle renders the session view
**Then** monocle MUST surface open drift items

**Source**: `orchestrator.md:401-405`, `templates/tech-debt-register-template.md` (inventoried).

## H10. Detect a compatible factory (forward-compatible)

**Given** a project with `.factory/STATE.md` whose `document_type: pipeline-state` but `mode:` value is unknown to monocle
**When** monocle opens the session
**Then** monocle MUST treat it as a factory project (rendering generic phase/step), NOT discard it

This is the multi-factory abstraction test — future factory dispatchers should still work.

## H11. Handle missing STATE.md gracefully

**Given** `.factory/` exists but `STATE.md` is missing or corrupted
**When** monocle opens the session
**Then** monocle MUST NOT crash; it SHOULD surface "pipeline state unavailable, recoverable via /vsdd-factory:recover-state"

**Source**: `skills/recover-state/SKILL.md:11-19`.

## H12. Honor STATE.md size budget

**Given** STATE.md is between 200 and 500 lines
**When** monocle reads it
**Then** monocle MUST report the size status (HEALTHY/WARN/NEEDS-COMPACT) consistent with check-state-health verdicts

**Source**: `check-state-health/SKILL.md:44-51`.

## H13. Detect phase numbering drift

**Given** STATE.md frontmatter has `phase: 3.5` or `phase: 2-story-decomposition-patch-cycle`
**When** monocle reads it
**Then** monocle SHOULD flag the drift (but still surface the value to the human)

**Source**: `check-state-health/SKILL.md:55-63`.

## H14. Treat `.lobster` files as immutable

**Given** monocle has read a `.lobster` workflow
**When** monocle generates a recommendation
**Then** monocle MUST NOT propose edits to the workflow file; workflow edits are an out-of-band operator concern

**Source**: `run-phase/SKILL.md:52`, `validate-workflow/SKILL.md:44`.

## H15. Respect information asymmetry walls when surfacing

**Given** the adversary agent is currently running
**When** monocle's session view is being constructed for the adversary
**Then** monocle MUST NOT include excluded artifacts (`.factory/cycles/**/adversarial-reviews/**`, `.factory/semport/**`, `.factory/holdout-scenarios/**`, etc.)

**Source**: `code-delivery.lobster:118-135`.

## State Checkpoint

```yaml
pass: 7
status: complete
holdout_scenarios: 15
timestamp: 2026-05-11T22:25:00Z
next_pass: B-deep-workflows
```
