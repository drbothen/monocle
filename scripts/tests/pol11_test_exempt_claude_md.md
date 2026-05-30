---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in CLAUDE.md at workspace root is exempt (not reported)"
expected_pol11_result: PASS
target_subpath: root/
target_filename: CLAUDE.md
---

# POL-11 Regression Fixture: CLAUDE.md Workspace-Root Exemption

This fixture simulates the repo-root `CLAUDE.md` project operating instructions
file. It contains intentionally stale version-pin literals — CLAUDE.md captures
pipeline state snapshots (artifact versions at last checkpoint) and naturally
accumulates historical version references across delivery cycles.

Because this file is named `CLAUDE.md` at the workspace root, POL-11 must skip
it entirely. The exemption is implemented via `_EXCLUDED_ROOT_FILES` frozenset in
`collect_files()` — the root-level file walk checks `entry.name not in _EXCLUDED_ROOT_FILES`
before collecting (ADR-0007 v1.0.8 §Living-State Exemption Set).

This exemption applies ONLY to `CLAUDE.md` at the workspace root. A file named
`CLAUDE.md` inside `.factory/` would NOT be exempt (the `_EXCLUDED_ROOT_FILES`
check is only in the root-level walk, not in `_walk()` which processes subdirs).

## Simulated CLAUDE.md Content (stale pins expected here)

Artifact versions at last checkpoint (2026-05-28T08:15:32Z):
- SS-tui v1.5.0 (at S-025 authoring baseline)
- SS-ipc v1.4.0 (at S-019 authoring baseline)
- ADR-0007 v1.0.0 (at Phase 1 gate)

These version references are intentionally stale (SS-tui canonical is 1.8.2).
POL-11 must skip this file via _EXCLUDED_ROOT_FILES basename check and exit 0.
