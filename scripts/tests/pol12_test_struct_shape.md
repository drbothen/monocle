---
document_type: test-fixture
purpose: "POL-12 fixture — intentionally stale structural claim"
expected_pol12_result: FAIL
---

# POL-12 Test Fixture: Stale Structural Claim

This fixture contains an intentionally wrong type for `App.sessions`.
POL-12 Phase 1 must detect this and fail.

## Tasks

- [ ] Implement `App` struct with fields:
  - `sessions: Vec<SessionState>` (INTENTIONALLY WRONG — canonical is Vec<EnrichedSession>)
  - `mode: AppMode`

## Downstream Consumer Contract

```rust
// Intentionally wrong structural claim — POL-12 must catch this
pub struct App {
    pub sessions: Vec<SessionState>,
    pub mode: AppMode,
}
```
