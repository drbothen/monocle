---
document_type: test-fixture
purpose: "POL-12 regression fixture — overlay_stack App-form canonical (VecDeque<PromptModal>) in backtick must PASS (was invisible in prior over-broad exclusion)"
expected_pol12_result: PASS
target_subpath: specs/behavioral-contracts/ss-99/
---

# POL-12 Regression Fixture: overlay_stack App-Form Canonical — Type-Aware Homonym (Must PASS)

## Regression context (F-S025-ADV33-MED-002)

Prior implementation used a blanket field-name exclusion that blinded POL-12 to
App-form claims for overlay_stack even when the cited type was correct. The fix uses
type-aware disambiguation so that:
- The IPC form (Vec of PermissionPromptPayload) is suppressed as a false positive.
- The canonical App form (VecDeque of PromptModal) is CHECKED (and must PASS when correct).

This fixture tests that the App-form with canonical type PASSES under the new logic.

## Postconditions

**PC-1:** The App struct maintains the modal stack as:
   - `overlay_stack: VecDeque<PromptModal>` — the permission overlay stack (canonical App form).
   - `sessions: Vec<EnrichedSession>` — the current session roster.

(overlay_stack here is the App struct field with its canonical type. POL-12 must PASS.)
