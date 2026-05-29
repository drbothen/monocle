---
document_type: test-fixture
purpose: "POL-11 fixture — intentionally stale version-pin literal"
expected_pol11_result: FAIL
---

# POL-11 Test Fixture: Stale Active Version-Pin Literal

This fixture contains an intentionally stale `SS-tui.md v1.5.0` citation
with no historical-anchor markers. POL-11 must detect and fail on this.

## Active Section (non-§Trace)

See SS-tui.md v1.5.0 §App struct for the TUI data model.

Also references BC-2.06.005 v0.9.0 for column layout — both are deliberately stale.

## §Trace v1.0.0

Historical content below is exempt from POL-11:
- Implemented against SS-tui.md v0.1.0 at initial authoring (2026-01-01).
- BC-2.06.005 v0.1.0 cited at S-001 authoring time.
