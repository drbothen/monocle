---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in STATE.md is exempt (not reported)"
expected_pol11_result: PASS
target_subpath: STATE.md
is_file_replacement: true
---

# POL-11 Regression Fixture: STATE.md Exemption

This fixture simulates the living STATE.md dashboard file. It contains intentionally
stale version-pin literals — the STATE.md captures pipeline state snapshots across
multiple waves and naturally accumulates historical version references.

Because this file IS <factory-root>/STATE.md (a specific file exclusion, not a
directory exclusion), POL-11 must skip it entirely.

## Live Pipeline State (simulated — stale pins are expected here)

SS-tui.md v1.5.0 at authoring baseline for Wave 3.
SS-ipc.md v1.4.0 per S-019 authoring.
ADR-0007 v1.0.0 ratified at Phase 1 gate.
BC-2.05.002 v1.0.3 as of S-022 cycle start.

These pins are intentionally stale. POL-11 must skip STATE.md via specific-file exclusion.
