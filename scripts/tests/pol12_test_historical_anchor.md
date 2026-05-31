---
document_type: test-fixture
purpose: "POL-12 fixture — structural claims that ARE historical anchors (exempt)"
expected_pol12_result: PASS
---

# POL-12 Test Fixture: Historical Anchor Exemptions

This fixture contains structural claims that are all historical anchors.
POL-12 must classify them all as exempt and exit 0.

## Form 1: explicit structural-claim-historical annotation

<!-- structural-claim-historical: this block records the pre-refactor API shape at S-001 authoring time -->
```rust
// Historical — before EnrichedSession was introduced
pub struct App {
    pub sessions: Vec<SessionState>,
}
```

## Form 2: inline annotation

The following has the marker on the preceding line:
<!-- structural-claim-historical -->
```rust
pub struct App {
    pub sessions: Vec<SessionState>,
    pub overlay_stack: Vec<PromptModal>,
}
```

## Form 3: time qualifier in comment

```rust
// S-025-introduced fields at S-025 authoring time (illustrative subset, not a structural claim)
pub struct App {
    pub sessions: Vec<SessionState>,
}
```

## Form 4: §Trace section

## §Trace v1.0.0

Historical record — structural claims inside §Trace are exempt:

```rust
// Early prototype — sessions was Vec<SessionState> before EnrichedSession
pub struct App {
    pub sessions: Vec<SessionState>,
}
```
