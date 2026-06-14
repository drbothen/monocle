---
document_type: test-fixture
purpose: "POL-14 fixture — parenthetical anchor-pin form with current version (Pattern C) — must PASS"
expected_pol11_result: PASS
---

# POL-14 Test Fixture: Current Parenthetical Anchor-Pin

## Background

Verifies that Pattern C does NOT produce false positives when:
  (a) The anchor-pin version matches the canonical registry version, OR
  (b) The anchor-pin appears in a §Trace section (historical anchor exemption), OR
  (c) The anchor-pin carries a time qualifier (historical anchor exemption).

All three cases below must be classified correctly so POL-14 exits 0 (PASS).

## Architecture Anchors — current version (should PASS)

This is a well-formed anchor-pin citation at the canonical version:
`architecture/SS-tui.md#appstruct` (v1.8.2) — this matches SS-tui current_version
in the registry, so POL-14 must NOT flag it as stale.

## Historical forms — exempt

### Form 1: §Trace exemption

## §Trace

Historical record — any anchor-pin citations here are frozen provenance:
`architecture/SS-ipc.md#serverspawnack` (v1.0.0) — initial version at story authoring.

### Form 2: time qualifier

At S-001 authoring time, `architecture/SS-tui.md#appstruct` (v1.2.0) was the reference.

### Form 3: explicit historical annotation

`architecture/SS-tui.md#appstruct` (v1.5.0) <!-- version-pin-historical: recorded at spec authoring -->
