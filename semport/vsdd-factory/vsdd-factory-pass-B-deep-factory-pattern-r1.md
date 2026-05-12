# Phase B Round 1 — Deep: Factory Project Discriminator

## The Question

How does monocle RELIABLY detect that a given project is using vsdd-factory (or a compatible factory dispatcher) so that it can switch into factory-aware mode?

## Available Signals (Strongest → Weakest)

### Signal 1: `.factory/STATE.md` with `document_type: pipeline-state` frontmatter (STRONGEST)

**Detection rule**: `<project>/.factory/STATE.md` exists AND its YAML frontmatter has `document_type: pipeline-state`.

**Evidence**: `templates/state-template.md:1-17`:
```yaml
---
document_type: pipeline-state
level: ops
version: "2.0"
status: draft
producer: state-manager
timestamp: YYYY-MM-DDTHH:MM:SS
phase: 1
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: "[project-name]"
mode: "[greenfield|brownfield|feature|maintenance|discovery|multi-repo]"
current_step: ""
current_cycle: ""
dtu_required: false
---
```

**Confidence**: HIGH. The `document_type` frontmatter field is **the canonical discriminator**. If it says `pipeline-state`, the project uses a state-managed pipeline that follows the vsdd-factory contract.

**Why this works for forward compatibility**: future factory dispatchers can adopt `document_type: pipeline-state` to be discoverable by monocle without inheriting the rest of vsdd-factory.

### Signal 2: `.factory-project/STATE.md` (multi-repo)

**Detection rule**: `<project>/.factory-project/STATE.md` exists at the workspace root.

**Evidence**: `templates/factory-project-state-template.md:1-13`:
```markdown
# .factory-project/ STATE.md

## Project

| Field | Value |
|-------|-------|
| project_name | ${project_name} |
| project_type | multi-repo |
| pipeline | ${pipeline_status} |
| phase | ${current_phase} |
| ...
```

**Confidence**: HIGH. This signals "multi-repo factory project". Combined with per-repo `.factory/STATE.md` signals, monocle gets the full picture.

NOTE: The multi-repo template uses a body table for `project_name`/`project_type`/`pipeline`/`phase`, NOT a YAML frontmatter `document_type` field. So multi-repo detection is BY DIRECTORY NAME, not by frontmatter. Monocle should fall back to "directory name === `.factory-project/`" as a sufficient signal.

### Signal 3: `.factory/wave-state.yaml`

**Detection rule**: `<project>/.factory/wave-state.yaml` exists.

**Evidence**: `templates/wave-state-template.yaml:1-29`.

**Confidence**: MEDIUM. The wave-state file exists in projects that have reached Phase 2+. Greenfield projects pre-Phase-2 may not have it.

### Signal 4: `.factory/policies.yaml`

**Detection rule**: `<project>/.factory/policies.yaml` exists.

**Evidence**: `agents/orchestrator/orchestrator.md:244-247`.

**Confidence**: LOW. Optional file; many factory projects won't have one.

### Signal 5: `.factory/` directory existence (catch-all)

**Detection rule**: `<project>/.factory/` exists as a directory.

**Confidence**: MEDIUM. A bare `.factory/` could exist for non-vsdd reasons (someone copying vsdd patterns, or other tools using the same name). Best used as a NECESSARY-but-not-SUFFICIENT signal.

### Signal 6: `hooks.json` referencing factory-dispatcher

**Detection rule**: `<project>/.claude/settings.local.json` or equivalent references `${CLAUDE_PLUGIN_ROOT}/hooks/dispatcher/bin/.../factory-dispatcher`.

**Confidence**: LOW for per-project detection — settings live in `~/.claude/plugins/cache/` for installed plugins, not in the project directory. The hooks.json template (`plugins/vsdd-factory/hooks/hooks.json.template`) lives in the PLUGIN tree, not the project tree.

### Signal 7: Cycle directory structure

**Detection rule**: `<project>/.factory/cycles/<some-id>/cycle-manifest.md` exists.

**Confidence**: MEDIUM. Strong AFTER first cycle; absent before.

## Recommended Discriminator Algorithm for Monocle

```python
def detect_factory(project_root):
    """
    Returns:
      None         — not a factory project
      "vsdd"       — vsdd-factory (single-repo)
      "vsdd-multi" — vsdd-factory (multi-repo)
      "unknown"    — looks factory-shaped but unknown dispatcher
    """
    state_md   = Path(project_root) / ".factory" / "STATE.md"
    project_md = Path(project_root) / ".factory-project" / "STATE.md"

    # Multi-repo strongest signal
    if project_md.exists():
        return "vsdd-multi"

    # Single-repo strongest signal
    if state_md.exists():
        # Parse frontmatter
        fm = parse_yaml_frontmatter(state_md)
        if fm.get("document_type") == "pipeline-state":
            # Could be vsdd or any compatible factory; check producer
            producer = fm.get("producer", "")
            if producer == "state-manager":
                # vsdd-factory's state-manager agent
                return "vsdd"
            # Future factory dispatcher with producer != "state-manager"
            return "unknown"
        # No document_type → still looks factory-shaped; treat conservatively
        return "unknown"

    # No STATE.md → not a factory project
    return None
```

The choice of `document_type: pipeline-state` (not `factory: vsdd-factory`) is the deliberate decoupling — it lets new factories adopt the discriminator without claiming the vsdd-factory name.

## Discrimination Confidence Matrix

| Signals present | Detection | Confidence |
|---|---|---|
| `.factory/STATE.md` with `document_type: pipeline-state` AND `producer: state-manager` | vsdd-factory | HIGH |
| `.factory-project/STATE.md` + per-repo `.factory/STATE.md` | vsdd-factory multi-repo | HIGH |
| `.factory/STATE.md` without `document_type` | factory-shaped, unknown dispatcher | LOW |
| `.factory/` exists but no STATE.md | factory-shaped, uninitialized | LOW |
| `.factory/wave-state.yaml` only | factory project mid-Phase-2+ | MEDIUM |
| None of the above | not a factory project | HIGH (negative) |

## Multi-Factory Manifest Sketch

For monocle to support N concurrent factory dispatchers, define a **factory adapter manifest**:

```yaml
# monocle factory-adapter manifest (proposed)
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

# Reading — where state lives
paths:
  state_md:        ".factory/STATE.md"
  wave_state:      ".factory/wave-state.yaml"
  policies:        ".factory/policies.yaml"
  event_log_glob:  ".factory/logs/events-*.jsonl"
  workflows_dir:   "${CLAUDE_PLUGIN_ROOT}/workflows/"   # plugin-resident
  cycles_dir:      ".factory/cycles/"
  multi_repo_root: ".factory-project/"

# Schema — what fields to read from STATE.md frontmatter
state_schema:
  frontmatter:
    mode:           {type: string, enum: [greenfield, brownfield, feature, maintenance, discovery, multi-repo]}
    phase:          {type: integer, range: [0, 7]}
    status:         {type: string, enum: [in_progress, complete, blocked]}
    current_step:   {type: string}
    current_cycle:  {type: string, optional: true}
    project:        {type: string}
    pipeline:       {type: string, optional: true}     # free-form enum

# Display — what monocle should surface in its session view
display:
  primary:
    - "Mode: ${state.mode}"
    - "Phase: ${state.phase}"
    - "Current step: ${state.current_step}"
  secondary:
    - "Cycle: ${state.current_cycle}"
    - "Wave: ${wave_state.current_wave}"
    - "Next gate: ${wave_state.next_gate_required}"
  activity:
    - source: event_log
      filter:  "session_id == ${current_session_id}"
      limit:   20
  drift:
    - source: state_md_body_table
      table:  "Drift Items"

# Action surface (read-only, monocle never writes)
actions:
  read_only: true
```

This sketch shows what monocle needs to KNOW from a factory adapter manifest. The actual implementation can be Go/Rust/JS — the schema above is the data contract.

## Why Discriminator Reliability Matters

If monocle false-positives on factory detection, it will:
- Try to render an empty STATE.md → confuse the user.
- Suggest workflow steps that don't apply → erode trust.

If monocle false-negatives:
- Misses an active factory project → loses the entire feature value.

The `document_type: pipeline-state` frontmatter is the cleanest discriminator because:
1. It's an explicit opt-in (not coincidental directory presence).
2. It's standardizable across multiple factory dispatchers.
3. It's already enforced by `skills/check-state-health/SKILL.md:33-34`.

## Delta Summary

- 7 detection signals enumerated and ranked.
- Recommended algorithm written.
- Confidence matrix produced.
- Multi-factory adapter manifest sketched.

## Novelty Assessment

Novelty: SUBSTANTIVE.
The discriminator question hadn't been explicitly answered in earlier passes. The choice of `document_type` frontmatter as the canonical signal (vs. directory presence or plugin name) is the load-bearing finding.

## Convergence Declaration

Another round needed — verify whether there are any "lookalike" projects (other tools using `.factory/STATE.md` or similar) that would false-positive the discriminator.

## State Checkpoint

```yaml
pass: B-deep-factory-pattern
round: 1
status: complete
timestamp: 2026-05-11T23:00:00Z
novelty: SUBSTANTIVE
```
