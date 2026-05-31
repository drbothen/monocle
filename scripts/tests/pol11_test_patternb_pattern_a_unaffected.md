---
document_type: test-fixture
purpose: "POL-11 Pattern B — Pattern A (inline prose) behavior unchanged alongside Pattern B YAML pins"
expected_pol11_result: FAIL
target_subpath: stories/
target_filename: STORY-INDEX.md
---

# POL-11 Pattern B Regression Fixture: Pattern A Still Works When Pattern B Present

This fixture confirms that adding Pattern B detection does not break Pattern A
(inline prose form) detection. The file contains BOTH forms:

- Pattern B (YAML inputs[] form): a current pin → ACTIVE, passes registry check.
- Pattern A (inline prose form): a stale active prose citation → must still be caught.

The fixture is placed as STORY-INDEX.md (living index doc), so Pattern B pins are ACTIVE.

## Pattern B (YAML inputs[] — current, should pass registry check)

inputs:
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.8.2"}

## Pattern A (inline prose — stale active citation, must still be flagged)

The TUI subsystem architecture is defined in SS-tui.md v1.5.0 (stale active citation —
no historical anchor marker, no time qualifier).

This file must FAIL because Pattern A detects the stale prose citation even though
the Pattern B pin is current. The two patterns operate independently.
