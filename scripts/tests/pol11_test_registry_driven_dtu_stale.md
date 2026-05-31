---
document_type: test-fixture
purpose: "POL-11 fixture — registry-driven Pattern A: dtu-assessment stale active citation"
expected_pol11_result: FAIL
target_subpath: stories/
---

# POL-11 Fixture: Registry-Driven dtu-assessment Stale Detection (F-S025-ADV31-MED-001)

This fixture contains an active inline citation of `dtu-assessment.md v1.7.5` which is stale
(canonical is v1.7.6 per registry). Before the registry-driven fix, this citation was
INVISIBLE to CI because the hardcoded Pattern A only matched SS-/BC-/ADR- prefixes.

The registry-driven matcher must detect this and fail CI.

## Normative content

The DTU scope is defined in dtu-assessment.md v1.7.5 §Clone Scope. All DTU clone
validation targets enumerated in that version are in scope for Phase 1.

Also confirms the hook protocol surface declared in dtu-assessment.md v1.7.5 §Hook Protocol.
