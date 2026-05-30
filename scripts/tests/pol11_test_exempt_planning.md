---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in planning/ is exempt (not reported)"
expected_pol11_result: PASS
target_subpath: planning/
---

# POL-11 Regression Fixture: planning/ Exemption

This fixture contains an intentionally stale version-pin literal. Because this file
lives under <factory-root>/planning/, it is a validation report (historical, not
normative) and must be skipped by POL-11.

## Stale Pin in Validation Report

Validation run against SS-conventions-anti-patterns.md v1.29.0 template.
BC-2.05.002 v1.0.2 was the canonical BC at validation time.

These pins are intentionally stale — this is a frozen validation report.
POL-11 must skip this file entirely via path-level exclusion.
