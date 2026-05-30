---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in a normative stories/ file IS reported (sprint-state.yaml exemption does NOT over-exclude other stories/)"
expected_pol11_result: FAIL
target_subpath: stories/
---

# POL-11 Regression Fixture: stories/ Remains Normative (sprint-state.yaml Exemption Control)

This fixture contains an intentionally stale version-pin literal placed in a
normative story file under `<factory-root>/stories/`. It proves that the
sprint-state.yaml specific-file exemption does NOT over-exclude: other files in
stories/ must still be scanned and stale pins must still be detected.

This is the FAIL-side control for `pol11_test_exempt_sprint_state_yaml.md`.

## Active Normative Citation (stale — must be detected)

See SS-tui v1.5.0 for the TUI skeleton panel layout and session list rendering.

This version is intentionally stale (SS-tui canonical is 1.8.2). POL-11 must
detect this as a stale active version-pin literal and exit 1, proving that
only `stories/sprint-state.yaml` is exempt — not all of stories/.
