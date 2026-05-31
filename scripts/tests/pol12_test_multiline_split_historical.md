---
document_type: test-fixture
purpose: "POL-12 regression fixture — Form (c) multi-line split with historical marker on EITHER line must PASS"
expected_pol12_result: PASS
target_subpath: specs/behavioral-contracts/ss-99/
---

# POL-12 Regression Fixture: Multi-Line Split — Historical Anchor Exemption (Must PASS)

## Regression context (F-S025-ADV33-MED-001)

Verifies that cross-line Path A detection correctly exempts split claims where EITHER
adjacent line carries a historical-anchor marker or time qualifier. Three sub-cases are
covered. All three must be classified as HISTORICAL and produce no findings (PASS).

## Sub-case A: historical marker on the field-name line (annotated reason form)

The prior impl used `App.overlay_stack: <!-- structural-claim-historical: pre-VecDeque form at S-001 authoring time -->
VecDeque<WrongType>` before the current canonical shape was adopted.

(The historical-anchor marker is on the same line as the App.field: reference.
Either line carries exemption — the split pair is historical. PASS.)

## Sub-case B: time qualifier on the type line

The original declaration was `App.overlay_stack:
Vec<PromptModal>` at S-001 authoring time, revised when VecDeque was adopted.

(Time qualifier "at S-001 authoring time" is on the type continuation line.
Either line carries exemption — PASS.)

## Sub-case C: historical marker on the type line

The architectural shape was captured as `App.overlay_stack:
Vec<PromptModal>` <!-- structural-claim-historical: initial prototype before canonical container --> in the original design.

(Marker is on the type continuation line. Either line carries exemption — PASS.)
