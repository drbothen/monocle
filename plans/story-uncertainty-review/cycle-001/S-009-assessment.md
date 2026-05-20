---
document_type: story-uncertainty-assessment
story_id: S-009
story_version: "1.6"
story_title: Auth Token Wire Format + Header Validation (FC-06, ADR-0005)
assessment_batch: BATCH-3
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_REVISION
---

# Story Assessment: S-009

## Verdict

**NEEDS_REVISION** — One CRITICAL finding (inherits from S-003): auth middleware ownership
split between S-003 and S-009 requires explicit handoff documentation. Additionally, two
MEDIUM findings on the integration test file naming discrepancy.

## Summary

S-009 is the auth validation story (8 points, 10 ACs). The dual-accept protocol
(canonical `X-Monocle-Authorization` and alias `X-Claude-Code-Ide-Authorization`) is
precisely specified with correct BC citations. The constant-time comparison requirement
(AC-008, AC-009) is unambiguous. The critical finding is the `auth.rs` ownership split with
S-003 — see S003-D3-01. Additionally, S-009's File Structure Requirements lists the test
file as `auth_header_rejection.rs` but the Tasks block says "auth integration tests" with
"VP-008 source-grep" as a test task — these two names suggest different test organization
than what the BC-naming convention in S-008 established.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | `rand =0.8.6`, `constant_time_eq 0.3`, `axum =0.8.9`, `serde_json =1.0.149`, `tracing 0.1` all correctly specified as EXACT or caret pins per SS-deps-pin-manifest.md. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S009-D2-01 | MEDIUM | File Structure Requirements says the test file is `auth_header_rejection.rs` but the Tasks list integration tests that include positive cases (canonical auth → 200, alias auth → 200). A "rejection" file name implies only negative test cases. The file should be `auth_header_validation.rs` (consistent with the story title "Header Validation"). This naming inconsistency will cause confusion. |
| S009-D2-02 | MEDIUM | AC-010b specifies that hook handlers "return HTTP 200 with body `{"status":"ok"}`." This is correct. But the Tasks block does not include a test for this specific response body on a valid hook POST. The auth validation test suite focuses on auth outcomes; the hook response body assertion ({"status":"ok"}) should be a separate test case to ensure the ring write + 200 response path is covered. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S009-D3-01 | CRITICAL | S-009 "extends" `auth.rs` created by S-003. But S-003's current story spec creates a full auth middleware — not a stub. S-009 creates `validate_auth_header()` which IS the auth middleware. This means either S-003's auth middleware is a temporary stub that S-009 replaces, or S-009's `validate_auth_header()` is an addition to the S-003 middleware. The story does not explicitly specify which model applies. This must be resolved (see S003-D3-01 for the fix direction). |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | The test list is comprehensive: canonical, alias, both-present, missing-both, wrong-token (canonical), wrong-token (alias), VP-008 source-grep, VP-009 source-grep. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter complete. inputs versioned correctly including dtu-assessment.md v1.7.5. |

## Research Queue

None.

## Recommended Fixes

1. S009-D3-01 (CRITICAL): Blocked on S003-D3-01. After S-003 is updated to specify a STUB auth middleware, update S-009's Previous Story Intelligence to clarify that S-009 REPLACES the S-003 stub with the full dual-accept implementation. Routing: story-writer (after S-003 fix).
2. S009-D2-01 (MEDIUM): Rename test file from `auth_header_rejection.rs` to `auth_header_validation.rs` in File Structure Requirements. Routing: story-writer.
3. S009-D2-02 (MEDIUM): Add test case for hook POST body `{"status":"ok"}` response (positive path after auth validation passes). Routing: story-writer.
