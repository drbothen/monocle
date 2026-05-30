---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in specs/ IS reported (normative, not exempt)"
expected_pol11_result: FAIL
target_subpath: specs/
---

# POL-11 Regression Fixture: specs/ Is Normative (Must Fail)

This fixture contains an intentionally stale version-pin literal placed under
<factory-root>/specs/. The specs/ subtree is normative and must be scanned by POL-11.

This test proves that the plans/planning/code-delivery/STATE.md exemptions do NOT
over-exclude: specs/ files must still be checked.

## Active Normative Citation (stale — must be detected)

See SS-tui.md v1.5.0 §App struct for the TUI data model definition.

This version is intentionally stale. POL-11 must detect this as a stale active
version-pin literal and exit 1, proving specs/ remains in the normative scan scope.
