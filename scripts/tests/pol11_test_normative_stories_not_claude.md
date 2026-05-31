---
document_type: test-fixture
purpose: "POL-11 fixture — stale pin in a normative stories/ file IS reported (CLAUDE.md exemption does NOT over-exclude stories/)"
expected_pol11_result: FAIL
target_subpath: stories/
---

# POL-11 Regression Fixture: stories/ Not Exempt via CLAUDE.md Rule (CLAUDE.md Exemption Control)

This fixture contains an intentionally stale version-pin literal placed in a
normative story file under `<factory-root>/stories/`. It proves that the
CLAUDE.md basename exemption (via `_EXCLUDED_ROOT_FILES`) does NOT accidentally
exclude non-CLAUDE.md files in stories/ or any other scanned directory.

This is the FAIL-side control for `pol11_test_exempt_claude_md.md`.

## Active Normative Citation (stale — must be detected)

See SS-tui v1.5.0 §TUI sessions panel for the layout specification used in
this story's acceptance criteria.

This version is intentionally stale (SS-tui canonical is 1.8.2). POL-11 must
detect this as a stale active version-pin literal and exit 1, proving that
the CLAUDE.md exemption affects only CLAUDE.md at the workspace root.
