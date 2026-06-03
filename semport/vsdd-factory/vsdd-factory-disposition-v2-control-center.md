---
document_type: gene-source-disposition
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
gene_source: vsdd-factory
disposition_pass: v2 (D-236 control-center pivot)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md §5
---

# Gene-Source Disposition v2: vsdd-factory (Control-Center Lens)

## Vision Lens Applied

monocle v1 (re-baselined): full TUI control center. vsdd-factory is the Workflow plane gene
— monocle observes factory pipeline state. The pivot does NOT change monocle's relationship
to vsdd-factory: monocle remains observe-only for workflow/factory state. The Non-Goal
"Does NOT execute workflows" is confirmed.

## Original Disposition Summary

The original v1.1.2 vision adopted vsdd-factory as:
- Factory awareness (Workflow plane): read `.factory/STATE.md`, surface phase/status/blocking
  issues/convergence.
- FactoryAdapter trait: `detect(project_root)` + `read_state()` + `on_change()` returning
  `FactoryState`.
- VsddFactoryAdapter as the first concrete adapter (reads STATE.md YAML frontmatter).
- Discriminator pattern: `document_type: pipeline-state` for detection.
- Multi-repo signal: `.factory-project/` alongside `.factory/`.
- WASM plugin SDK for third-party factory adapters (Phase 3 scope).

Left behind:
- 33+ specialist agents, 116 skills, 27 enforcement hooks — monocle never runs these.
- Lobster workflow format internals (deeper than needed for display).
- factory-dispatcher WASM plugin details.

## Disposition by Capability Area

### 1. FactoryAdapter Observe-Only Pattern (originally ADOPT)

**New verdict: ADOPT (confirmed). Non-Goal explicitly CONFIRMED by pivot.**

NEXT-SESSION-PIVOT.md §2 states the pivot is about launching/managing sessions, not workflow
execution. The human's words: "We need to be able to launch, manage, and observe." The
Workflow plane remains observe-only.

Explicit confirmation of the Non-Goals:
- Does NOT execute workflows — monocle never writes STATE.md, never triggers phases.
- Does NOT dispatch agents — that is vsdd-factory's orchestrator's job.
- Does NOT write factory artifacts — read-only always.

The FactoryAdapter trait, VsddFactoryAdapter, and STATE.md parsing are unchanged.

### 2. Workflow Plane + Trigger from Sessions (new connection from pivot)

**New verdict: ENHANCE (minor) — sessions now link to factory projects.**

In the control-center model, when the user launches a session for a project that has a
`.factory/STATE.md`, the Workflow panel should automatically populate with that project's
factory state. This is the natural "session → project → workflow" linkage.

Previously this was implicit (monocle detected running sessions and their project roots via
hook events). In the control-center model, monocle explicitly knows the project root at
session launch time (it's the cwd/worktree). The Workflow plane can use this to pre-select
the correct adapter without waiting for a hook event.

Implementation: at session launch, pass the `project_root` to the `WorkflowService` which
calls `FactoryAdapter.detect(project_root)`. If a factory is detected, the Workflow panel
is pre-populated for the new session. No change to the observer pattern; just a new trigger
path.

### 3. WASM Plugin SDK for Factory Adapters (originally ADOPT, Phase 3)

**New verdict: LEAVE BEHIND for re-baselined v1.** Phase 3 WASM SDK is in the suspended
scope. VsddFactoryAdapter as a native Rust built-in is all that v1 needs. The WASM
extensibility point is preserved architecturally but not shipped in re-baselined v1.

### 4. Execute Workflows / Dispatch Agents (originally Leave-behind)

**CONFIRMED Leave-behind.** The pivot reversal was about session LAUNCHING (monocle spawns
the user's Claude Code sessions). It was NOT about factory workflow execution. monocle spawns
`claude` for the user's AI coding work; it does not spawn vsdd-factory pipeline runs.

If a user wants to run vsdd-factory phases, they do so from within a Claude Code session
(which monocle may have launched and is observing). The delineation: monocle launches coding
sessions; the user (or vsdd-factory's orchestrator via Claude Code) runs the factory pipeline.

### 5. STATE.md Schema / Frontmatter (originally ADOPT)

**New verdict: ADOPT (confirmed).** The STATE.md schema (document_type discriminator, phase/
status/blocking/convergence fields, YAML frontmatter) is the interface between vsdd-factory
and monocle. It is stable and well-defined. VsddFactoryAdapter is built (monocle-workflow
crate, S-025 wave 6).

### 6. Cost Monitoring Display (from vsdd-factory lobster workflows)

**New verdict: MODEL (display enhancement).** vsdd-factory's cost monitoring fields in
lobster workflows (thresholds, current cost, protected agents) could surface in monocle's
Workflow panel as a "cost burn" indicator alongside phase/status. This is a display-only
enhancement. Not a v1 requirement but a natural data point given monocle's token burn
display in the Runtime plane. Flagged for product-owner consideration in the revised brief.

## Summary Table

| Capability | Original Verdict | New Verdict | Change? |
|-----------|-----------------|-------------|---------|
| FactoryAdapter observe-only | ADOPT | ADOPT (confirmed; Non-Goal explicitly confirmed) | Confirmed |
| VsddFactoryAdapter + STATE.md | ADOPT (built S-025) | ADOPT (confirmed) | Confirmed |
| Workflow panel pre-population from launch | N/A | ENHANCE (minor; new trigger path from session launch) | NEW |
| WASM plugin SDK | ADOPT (Phase 3) | LEAVE BEHIND (re-baselined v1 scope) | Scoped out |
| Execute workflows / dispatch agents | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| Cost monitoring display | N/A | MODEL (display enhancement; product-owner call) | NEW |

## Net Assessment

vsdd-factory's disposition is the most unchanged of the 8 gene sources. The pivot's
"launch and manage" capability does not affect the observe-only Workflow plane. The only
new connection is the session-launch → project-root → factory-detection linkage, which
adds a trigger path but does not change the observer architecture.

The explicit confirmation of the Non-Goal ("Does NOT execute workflows") is the primary
output of this re-disposition. The pivot could have been misread as "monocle now executes
things" — it does not. monocle executes SESSION SPAWNING (launching `claude`); it does not
execute PIPELINE PHASES (launching vsdd-factory orchestrator).
