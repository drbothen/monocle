---
document_type: test-fixture
purpose: "POL-12 fixture — BC IPC-homonym field (overlay_stack: Vec<PermissionPromptPayload>) in InitialState bullet list must not be flagged as App struct stale claim"
expected_pol12_result: PASS
target_subpath: specs/behavioral-contracts/ss-99/
---

# POL-12 Test Fixture: IPC-Homonym False-Positive Suppression

This fixture verifies that `overlay_stack: Vec<PermissionPromptPayload>` in a
BC postcondition bullet describing the IPC InitialState message fields is NOT
flagged as a stale App struct claim.

`overlay_stack` is an IPC-homonym: it exists in BOTH:
- `App.overlay_stack: VecDeque<PromptModal>` (TUI App struct — canonical)
- `InitialState.overlay_stack: Vec<PermissionPromptPayload>` (IPC message field — correct)

Path B must NOT flag the IPC form. Path A (App-qualified) handles App struct claims.

## Postconditions

**PC-1:** The daemon sends `ServerToClient::InitialState` upon connection. The message contains:
   - `sessions: Vec<EnrichedSession>` — the full current session roster.
   - `overlay_stack: Vec<PermissionPromptPayload>` — any currently queued permission prompts.
   - `drop_counter: u64` — the current drop counter value.

(overlay_stack here is the IPC message field, NOT App.overlay_stack. POL-12 must PASS.)
