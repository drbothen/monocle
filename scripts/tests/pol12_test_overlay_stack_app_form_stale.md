---
document_type: test-fixture
purpose: "POL-12 regression fixture — overlay_stack App-form stale (Vec<PromptModal>, wrong container) must FAIL"
expected_pol12_result: FAIL
target_subpath: specs/behavioral-contracts/ss-99/
---

# POL-12 Regression Fixture: overlay_stack App-Form Stale — Must FAIL

## Regression context (F-S025-ADV33-MED-002)

The type-aware homonym disambiguation must classify `overlay_stack: Vec<PromptModal>`
as NEITHER an IPC form (wrong type argument — IPC uses PermissionPromptPayload) NOR
canonical App (wrong container — App uses VecDeque, not Vec). Therefore it is a stale
App-form claim and POL-12 MUST detect it.

## Postconditions

**PC-1:** The app struct modal stack is maintained:
   - `overlay_stack: Vec<PromptModal>` — INTENTIONALLY WRONG (canonical is VecDeque<PromptModal>).
   - `sessions: Vec<EnrichedSession>` — the current session roster.

(overlay_stack Vec<PromptModal> is not the IPC prefix AND mismatches canonical VecDeque<PromptModal>.
POL-12 must FAIL and report the stale App-form structural claim.)
