---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in sprint-state.yaml is exempt (not reported)"
expected_pol11_result: PASS
target_subpath: stories/
target_filename: sprint-state.yaml
---

# POL-11 Regression Fixture: sprint-state.yaml Exemption

This fixture simulates the living sprint-state.yaml file placed at
`<factory-root>/stories/sprint-state.yaml`. It contains intentionally stale
version-pin literals — sprint-state.yaml accumulates historical version
references across waves and delivery cycles as a normal part of its function.

Because this file IS `<factory-root>/stories/sprint-state.yaml` (a specific file
exclusion within the otherwise-normative stories/ directory), POL-11 must skip it
entirely. The exemption is implemented as a `factory_root_name + "/stories/sprint-state.yaml"`
entry in `extra_prefixes` inside `collect_files()` (ADR-0007 v1.0.8 §Living-State
Exemption Set).

## Simulated Sprint-State Content (stale pins expected here)

sprint_state_version: "1.29"

wave_6_stories:
  - id: S-022
    spec_version: "SS-tui v1.5.0"
    status: done
  - id: S-025
    spec_version_at_authoring: "SS-tui v1.5.0"
    status: in_progress

These version-pin references are intentionally stale (SS-tui canonical is 1.8.2).
POL-11 must skip this file via specific-file exclusion and exit 0.
