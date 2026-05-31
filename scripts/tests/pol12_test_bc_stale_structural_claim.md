---
document_type: test-fixture
purpose: "POL-12 fixture — BC file with stale structural claim (sessions: Vec<SessionState>)"
expected_pol12_result: FAIL
target_subpath: specs/behavioral-contracts/ss-99/
---

# POL-12 Test Fixture: Stale Structural Claim in BC Postcondition Prose

This fixture simulates a behavioral-contract postcondition prose entry that cites
a stale type for `App.sessions`. POL-12 Phase 1 MUST detect and flag this.

ADR-0008 §CI enforcement gate Phase 1 mandates scanning
`.factory/specs/behavioral-contracts/**/*.md` for postcondition structural claims.

## Postconditions

**PC-1:** After TUI connects, `App struct` fields are populated:
   - `sessions: Vec<SessionState>` — the full current session roster.

(INTENTIONALLY WRONG — canonical App.sessions type is `Vec<EnrichedSession>`)
