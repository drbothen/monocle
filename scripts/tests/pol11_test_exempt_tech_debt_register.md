---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in tech-debt-register.md is exempt (not reported)"
expected_pol11_result: PASS
target_subpath: tech-debt-register.md
target_filename: tech-debt-register.md
---

# POL-11 Regression Fixture: tech-debt-register.md Exemption

This fixture simulates the living tech-debt-register.md file placed at
`<factory-root>/tech-debt-register.md`. It contains intentionally stale
version-pin literals — the tech-debt-register captures deferred items with
version references frozen at deferral time, which are historical by definition.

Because this file IS `<factory-root>/tech-debt-register.md` (a specific file
exclusion at the factory root), POL-11 must skip it entirely. The exemption is
implemented as a `factory_root_name + "/tech-debt-register.md"` entry in
`extra_prefixes` inside `collect_files()` (ADR-0007 v1.0.8 §Living-State
Exemption Set).

## Simulated Debt Register Content (stale pins expected here)

### TD-041: TUI panel layout alignment (deferred at S-025 authoring)

Deferred against SS-tui v1.5.0 (current at deferral time). Requires human
direction per canonical principle §3 before scheduling. Blocked on S-026
completion.

These version references are intentionally stale (SS-tui canonical is 1.8.2).
POL-11 must skip this file via specific-file exclusion and exit 0.
