---
document_type: test-fixture
purpose: "POL-11 Pattern B fixture — index doc with current YAML inputs[] pin is ACTIVE (passes)"
expected_pol11_result: PASS
target_subpath: stories/
target_filename: STORY-INDEX.md
---

# POL-11 Pattern B Regression Fixture: Index Document inputs[] YAML Pin — ACTIVE (current → PASS)

This fixture simulates a living index document (`STORY-INDEX.md`) whose `inputs[]`
YAML frontmatter contains current (non-stale) version pins. Per ADR-0007 §Story
inputs[] Historical Provenance, living index documents are classified ACTIVE and
checked against the registry — but a current pin passes the check.

## Simulated Frontmatter Content

inputs:
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.8.2"}

SS-tui v1.8.2 is the canonical current version per version-pin-registry.yaml.
Because this file is placed as STORY-INDEX.md (a living index document), the YAML
pin is classified ACTIVE and checked. The version matches canonical, so POL-11 exits 0.
