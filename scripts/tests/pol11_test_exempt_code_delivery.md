---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in code-delivery/ is exempt (not reported)"
expected_pol11_result: PASS
target_subpath: code-delivery/
---

# POL-11 Regression Fixture: code-delivery/ Exemption

This fixture contains an intentionally stale version-pin literal. Because this file
lives under <factory-root>/code-delivery/, it is an at-merge PR description record
(historical, not normative) and must be skipped by POL-11.

## Stale Pin in PR Description Record

PR #27 merged implementing SS-ipc.md v1.6.0 at-least-once delivery semantics.
ADR-0006 v1.0.0 non_exhaustive struct discipline applied.
SS-conventions-anti-patterns.md v1.30.0 codified.

These pins are intentionally stale — this is a frozen code-delivery record captured
at PR merge time. POL-11 must skip this file entirely via path-level exclusion.
