---
document_type: test-fixture
purpose: "POL-11 regression fixture — ADV-29 SECONDARY multi-line split historical pin (must PASS)"
expected_pol11_result: PASS
---

# POL-11 Regression Fixture: Cross-Line Split Historical Pin (Must Pass)

## Regression context

Companion to pol11_test_crossline_split_stale.md. Verifies that cross-line
split detection correctly exempts pins where EITHER adjacent line carries a
historical-anchor marker or time qualifier. Three sub-cases are covered:

Sub-case A: historical marker on the version line (annotated form with colon).
Sub-case B: time qualifier on the version line.
Sub-case C: historical marker on the artifact-ID line.

All three sub-cases must be exempt (PASS). If any is incorrectly flagged as
active, the fixture's expected PASS will not be satisfied.

## Sub-case A: marker on version line (annotated form)

This variant does not exist in the IPC type system (removed in BC-2.06.004
v1.1.0 <!-- version-pin-historical: ClientDisconnect removed in v1.1.0 revision; current canonical is v1.2.1 -->). Disconnection detected via TransportEvent.

## Sub-case B: time qualifier on version line

The protocol was updated; removed in BC-2.06.004
v1.1.0 at S-025 authoring time (historical baseline).

## Sub-case C: marker on artifact-ID line

The feature was removed in BC-2.06.004 <!-- version-pin-historical: frozen historical record -->
v1.1.0 in the IPC type system redesign.

## §Trace (unconditionally historical)

At initial authoring, BC-2.06.004 v0.9.0 was the initial draft.
