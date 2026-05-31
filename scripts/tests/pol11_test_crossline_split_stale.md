---
document_type: test-fixture
purpose: "POL-11 regression fixture — ADV-29 SECONDARY multi-line split stale pin (no marker)"
expected_pol11_result: FAIL
---

# POL-11 Regression Fixture: Cross-Line Split Stale Pin (No Historical Marker)

## Regression context

This fixture covers the SECONDARY gap surfaced in Adversarial Pass 29:
an artifact ID at end of line N and the version vX.Y at start of line N+1
is an artifact/version split that per-line regex cannot match. POL-11 now
implements cross-line detection (PG-SPLIT): if an artifact ID appears as
the last token on a line AND the next line opens with a version literal,
the pair is treated as a split version-pin citation and checked against the
registry. Historical-anchor exemption applies if EITHER line carries a marker
or time qualifier.

## Active Section — must FAIL (cross-line split, no marker on either line)

This variant does not exist in the IPC type system (removed in BC-2.06.004
v1.1.0 behavior was deleted from the type).
