---
document_type: test-fixture
purpose: "POL-11 Pattern B fixture — index doc with stale YAML inputs[] pin is ACTIVE (flagged)"
expected_pol11_result: FAIL
target_subpath: stories/
target_filename: STORY-INDEX.md
---

# POL-11 Pattern B Regression Fixture: Index Document inputs[] YAML Pin — ACTIVE (stale → FAIL)

This fixture simulates a living index document (`STORY-INDEX.md`) whose `inputs[]`
YAML frontmatter contains stale version pins. Per ADR-0007 §Story inputs[] Historical
Provenance, living index documents are classified ACTIVE (continuously-maintained,
inputs[] is a live declaration) and MUST be checked against the registry.

A stale pin in a living index doc is a real defect — the index is claiming to reflect
a spec version it does not actually track. POL-11 must detect and fail on this.

## Simulated Frontmatter Content

inputs:
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.5.0"}

SS-tui v1.5.0 is intentionally stale (canonical: v1.8.2). Because this file is
placed as STORY-INDEX.md (a living index document), the YAML pin is classified
ACTIVE and POL-11 must fail with a version-pin staleness finding.
