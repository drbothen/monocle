---
document_type: test-fixture
purpose: "POL-11 fixture — registry-driven Pattern A: SS-/BC-/ADR- regression (must still detect stale)"
expected_pol11_result: FAIL
target_subpath: stories/
---

# POL-11 Fixture: Registry-Driven SS-/BC-/ADR- Regression Check (F-S025-ADV31-MED-001)

This fixture verifies that the registry-driven Pattern A does NOT regress on the original
SS-/BC-/ADR- detection capability. These artifact types were covered by the hardcoded regex
before the registry-driven rewrite; they must remain covered after.

All citations below are intentionally stale and must be detected.

## Normative content

See SS-tui.md v1.5.0 §App struct for the TUI layout model.

BC-2.06.004 v1.0.0 governs the sessions panel rendering invariants.

ADR-0007 v1.0.2 ratified the version-pin citation discipline.
