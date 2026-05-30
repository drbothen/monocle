---
document_type: test-fixture
purpose: "POL-11 fixture — registry-driven Pattern A: dtu-assessment current citation passes"
expected_pol11_result: PASS
target_subpath: stories/
---

# POL-11 Fixture: Registry-Driven dtu-assessment Current Citation (F-S025-ADV31-MED-001)

This fixture contains an active inline citation of `dtu-assessment.md v1.7.6` which is the
canonical current version per registry. The registry-driven matcher must recognise this
artifact ID AND verify the version matches, resulting in a CI pass.

## Normative content

The DTU scope is defined in dtu-assessment.md v1.7.6 §Clone Scope. All DTU clone
validation targets enumerated in that version are in scope for Phase 1.
