---
document_type: test-fixture
purpose: "POL-12 fixture — BC file with canonical structural claim (sessions: Vec<EnrichedSession>)"
expected_pol12_result: PASS
target_subpath: specs/behavioral-contracts/ss-99/
---

# POL-12 Test Fixture: Canonical Structural Claim in BC Postcondition Prose

This fixture simulates a behavioral-contract postcondition prose entry that correctly
cites the canonical type for `App.sessions`. POL-12 Phase 1 must PASS on this file.

ADR-0008 §CI enforcement gate Phase 1 mandates scanning
`.factory/specs/behavioral-contracts/**/*.md` for postcondition structural claims.

## Postconditions

**PC-1:** After TUI connects, `App.sessions` is populated as `Vec<EnrichedSession>`
with the session roster received from the daemon's `InitialState` message.

(CANONICAL — matches SS-tui.md §App struct App.sessions declaration exactly)
