---
document_type: test-fixture
purpose: "POL-14 fixture — parenthetical anchor-pin form with stale version (Pattern C)"
expected_pol11_result: FAIL
---

# POL-14 Test Fixture: Stale Parenthetical Anchor-Pin

## Background

This fixture covers the process-gap surfaced in F-P46-IMP-001 (Pass-46): the form
`architecture/SS-ipc.md#servertoClientspawnack` (v1.21.0) was invisible to Pattern A
because the `#anchor` + space + `(` separated the artifact ID from the version literal,
preventing the `<id>[ \t]+v<version>` match from firing. Three BCs accrued stale
anchor-section pins that drifted from their §Architecture Source rows and passed
POL-11 for many passes.

POL-14 (Pattern C in check_version_pins.py) must detect and fail on this form.

## Architecture Anchors (active — must FAIL)

The following anchor-pin citation uses a deliberately stale version. There is no
historical-anchor marker or time qualifier, so POL-14 must flag it.

`architecture/SS-tui.md#appstruct` (v1.5.0) — stale anchor-pin citation that survived
previous POL-11 sweeps due to the ID-version separation by #fragment + parens.

## §Trace

At initial authoring, `architecture/SS-tui.md#appstruct` (v0.1.0) was the version
referenced. This is a historical record and is exempt from POL-14.
