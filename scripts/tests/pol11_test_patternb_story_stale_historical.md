---
document_type: test-fixture
purpose: "POL-11 Pattern B fixture — story file with stale YAML inputs[] pin is HISTORICAL (not flagged)"
expected_pol11_result: PASS
target_subpath: stories/
target_filename: S-999-patternb-story-fixture.md
---

# POL-11 Pattern B Regression Fixture: Story File inputs[] YAML Pin — HISTORICAL

This fixture simulates an individual story file (`S-999-patternb-story-fixture.md`)
whose `inputs[]` YAML frontmatter contains stale version pins. Per ADR-0007
§Story inputs[] Historical Provenance, individual story files' inputs[] records
are classified HISTORICAL (authored-against provenance frozen at story authoring time)
and MUST NOT be flagged as stale.

The BLIND SPOT this closes: before Pattern B detection was added, these YAML pins
were silently ignored (not detected at all). With Pattern B, they are detected
and explicitly classified HISTORICAL — the gate is no longer blind to this form.

## Simulated Frontmatter Content

The inputs[] YAML pins below are intentionally stale but must be classified HISTORICAL
and not trigger a POL-11 failure (because this fixture is placed as S-999-*.md):

inputs:
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.5.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.004.md, version: "0.9.0"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.4.0"}

All three YAML pins above reference stale spec versions (each below the canonical
current version). However, this file is placed as an individual story file
(S-999-*.md), so all three must be classified HISTORICAL and not flagged.
POL-11 must log "[HISTORICAL] inputs[] provenance, exempt" for each and exit 0.
