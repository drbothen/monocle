---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in plans/ is exempt (not reported)"
expected_pol11_result: PASS
target_subpath: plans/
---

# POL-11 Regression Fixture: plans/ Exemption

This fixture contains an intentionally stale version-pin literal that would normally
trigger a POL-11 FAIL. However, because this file lives under <factory-root>/plans/,
it must be classified as an exempt historical/audit record and NOT reported.

## Stale Pin in Frozen Audit Record

Verified against SS-tui.md v1.5.0 for TUI data model compliance.
Also reviewed BC-2.06.005 v0.9.0 column layout. ADR-0007 v0.9.0 ratification noted.

These pins are intentionally stale — this is a frozen adversary audit record from
a prior cycle. POL-11 must skip this file entirely (path-level exclusion, not
line-level historical-anchor exemption).
