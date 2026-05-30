---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in a normative factory-root specs/ file IS reported (tech-debt-register.md exemption does NOT over-exclude specs/)"
expected_pol11_result: FAIL
target_subpath: specs/
---

# POL-11 Regression Fixture: specs/ Still Normative (tech-debt-register.md Exemption Control)

This fixture contains an intentionally stale version-pin literal placed under
`<factory-root>/specs/`. It proves that the tech-debt-register.md specific-file
exemption does NOT over-exclude the specs/ tree: normative spec files must still
be scanned and stale pins must still be detected.

This is the FAIL-side control for `pol11_test_exempt_tech_debt_register.md`.

## Active Normative Citation (stale — must be detected)

See SS-tui v1.5.0 §TUI skeleton panel layout for the sessions panel specification.

This version is intentionally stale (SS-tui canonical is 1.8.2). POL-11 must
detect this as a stale active version-pin literal and exit 1, proving that
only `tech-debt-register.md` at the factory root is exempt — not the specs/ tree.
