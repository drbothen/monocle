---
document_type: test-fixture
purpose: "POL-11 fixture — version-pin literals that ARE historical anchors (exempt)"
expected_pol11_result: PASS
---

# POL-11 Test Fixture: Historical Anchor Exemptions

This fixture contains version-pin literals that are all historical anchors.
POL-11 must classify them all as exempt and exit 0.

## Form 1: §Trace section exemption

## §Trace v1.0.0

Historical content — any version pins here are frozen historical provenance:
- BC-2.06.005 v0.9.0 was the version at initial story authoring (historical record).
- SS-tui.md v1.0.0 — initial architecture reference.

## Form 2: explicit historical-anchor annotation

The following cites a stale version but is explicitly annotated:
Implemented against SS-tui.md v1.5.0 <!-- version-pin-historical --> at implementation time.

## Form 3: time qualifier

The following contain time qualifiers that make them historical anchors:

- at S-001 authoring time, SS-tui.md v1.5.0 was the canonical TUI spec.
- BC-2.06.005 v0.9.0 was the baseline at spec authoring time.
- Implemented against SS-engine-module.md v1.1.20 at S-014 authoring time (TDD red-gate authoring baseline).
- SS-deps-pin-manifest.md v1.0.0 at time of initial dependency pinning.
- SS-ipc.md v1.4.0 at T-NNN dispatch time.
