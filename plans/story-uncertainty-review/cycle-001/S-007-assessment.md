---
document_type: story-uncertainty-assessment
story_id: S-007
story_version: "1.1"
story_title: Crash Recovery Checkpoint
assessment_batch: BATCH-3
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_REVISION
---

# Story Assessment: S-007

## Verdict

**NEEDS_REVISION** — One CRITICAL finding (inherits from S-005): the UDS socket that S-007
depends on for `recovery_available` message dispatch is not established by any story in the
dependency chain. See S-005 assessment S005-D3-01.

## Summary

S-007 is the crash recovery checkpoint story. The 4-field schema (AC-008), VP-006 regex
validation, and atomic write pattern are all precisely specified. The critical dependency
gap is the UDS control socket origin — S-007's Previous Story Intelligence claims "UDS
control socket established in S-005" but S-005 contains no UDS socket implementation.
Until S-005-D3-01 is resolved, S-007 cannot be safely dispatched.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | `tempfile 3`, `serde_json =1.0.149`, `chrono 0.4`, `tokio =1.52` all correctly specified. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S007-D2-01 | MEDIUM | AC-003 specifies the UDS recovery message format as `{"type":"recovery_available","last_app_mode":"<value-from-file>"}`. The 60-second window is measured from "daemon start time, NOT from the moment the control socket becomes ready." This is a subtle timing requirement. The implementation must store the daemon start `Instant` at process startup — but the story does not specify WHERE this `Instant` is stored (in `AppMode` state? in a separate `DaemonContext` struct?). Without this, the implementer cannot reliably implement the 60-second window. |
| S007-D2-02 | LOW | The `ShutdownReason` enum is `#[non_exhaustive]` per AC-008 Tasks. But S-011 (Non-Exhaustive Enum Policy) does not list `ShutdownReason` in the canonical 9-enum set. Either `ShutdownReason` is exempt (it is in `monocle-runtime`, not `monocle-core`) or the non_exhaustive policy for `monocle-runtime` types is not documented. This should be clarified. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S007-D3-01 | CRITICAL | See S-005 assessment S005-D3-01. S-007 depends on a UDS control socket that is not established by S-005 (or any other story in the current corpus). This is a blocking gap. The UDS socket must be either added to S-005 or assigned to a new story before S-007 can be dispatched. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| S007-D4-01 | LOW | The test name in File Structure Requirements is `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup`. This is a good explicit name but it is only one test function — the test list in Tasks has 9 scenarios. Multiple test functions should be specified, or the single function should be documented as a macro-test with 9 sub-scenarios. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter complete. inputs versioned correctly. |

## Research Queue

None. The UDS socket gap is a spec clarity issue resolvable from SS-daemon-lifecycle.md.

## Recommended Fixes

1. S007-D3-01 (CRITICAL): Blocked on S-005 fix. After S-005's UDS socket story assignment is resolved, update S-007's Previous Story Intelligence to reference the correct story. Routing: story-writer (after architect resolves UDS socket origin).
2. S007-D2-01 (MEDIUM): Add explicit guidance on where to store the daemon start `Instant` — recommend `DaemonContext` struct or `AppState` in `state.rs`. Routing: architect (structural decision), then story-writer.
3. S007-D2-02 (LOW): Clarify whether `#[non_exhaustive]` applies to `monocle-runtime` enums or only `monocle-core` enums. If monocle-runtime enums follow the same policy, document this in S-007 Architecture Compliance Rules. Routing: architect.
