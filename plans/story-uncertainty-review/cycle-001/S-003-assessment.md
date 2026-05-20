---
document_type: story-uncertainty-assessment
story_id: S-003
story_version: "1.4"
story_title: Status Endpoint (Authenticated Daemon State)
assessment_batch: BATCH-2
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_REVISION
---

# Story Assessment: S-003

## Verdict

**NEEDS_REVISION** — One CRITICAL finding: S-003 implements auth middleware in `auth.rs` but
S-009 also implements auth middleware in the same file (`monocle-runtime/src/auth.rs`). The
ownership of `auth.rs` is split across two stories. This creates a TDD collision where S-003
creates `auth.rs` with auth middleware and S-009 later extends/replaces it — the handoff
contract is not specified.

## Summary

S-003 is the status endpoint story. The 10-field response is well-specified and all BC
citations are accurate. The critical finding is an architectural ownership ambiguity: S-003
says "Build auth middleware as a tower Layer or axum middleware::from_fn" and creates
`monocle-runtime/src/auth.rs`. But S-009 says "S-009 extends this module with
`validate_auth_header()` and the auth middleware" — implying S-003 creates a preliminary
auth that S-009 supersedes. This pattern, if not explicitly specified in both stories, will
cause implementers to create conflicting or overlapping implementations.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | `constant_time_eq 0.3`, `chrono 0.4`, `axum =0.8.9` all correctly specified. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S003-D2-01 | MEDIUM | AC-001 specifies `ring_buffer_fill_pct` and `channel_saturation_pct` as "float 0.0–100.0". The Rust type is not specified. Using `f32` vs `f64` affects JSON serialization precision. `serde_json` serializes `f64` as a full-precision float, `f32` as a truncated float. For a percentage field, `f32` is sufficient but `f64` is more conventional in Rust JSON APIs. The story should specify the type. |
| S003-D2-02 | MEDIUM | AC-007 specifies `last_hook_ts` uses `Option<String>` serialized as ISO 8601 via chrono. But the 5-field structure within `last_hook_ts` (one per hook type) is not specified in the story. How is the `last_hook_ts` JSON object structured? The BC citation says "object with 5 nullable timestamp fields" but the field names (pre_tool_use, notification, stop, session_start, prompt_submit?) are not defined in this story. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S003-D3-01 | CRITICAL | S-003 creates `monocle-runtime/src/auth.rs` with auth middleware. S-009 also creates/extends `monocle-runtime/src/auth.rs` with `validate_auth_header()`. The handoff is noted in S-009's Previous Story Intelligence ("S-009 extends this module") but NOT in S-003's File Structure Requirements or Previous Story Intelligence. S-003 must explicitly document that it creates a STUB auth middleware (sufficient for /status testing) and that S-009 will extend it with the full dual-accept validation. Without this, the S-003 implementer may create a complete auth implementation that conflicts with S-009. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| S003-D4-01 | LOW | Test list specifies "`hook_endpoints` array == exactly 5 paths in spec order" but does not specify the spec order. The 5 paths are defined in BC-2.01.002, not in the story text. The test-writer must read the BC to determine the order. Add the explicit ordered list to the Tasks test specification. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter is complete. All inputs are versioned. |

## Research Queue

None. The `auth.rs` ownership issue is a spec clarity gap, not an external research question.

## Recommended Fixes

1. S003-D3-01 (CRITICAL): Add a section to S-003 File Structure Requirements and Previous Story Intelligence explicitly stating: "S-003 creates a STUB auth middleware in `auth.rs` that validates only the canonical `X-Monocle-Authorization` header for /status testing. S-009 extends this module with the full dual-accept (canonical + alias) validation. The S-003 stub will be replaced/extended by S-009." Add the corresponding note to S-009's Previous Story Intelligence (already partially present). Routing: story-writer.
2. S003-D2-01: Specify `f64` for `ring_buffer_fill_pct` and `channel_saturation_pct`. Routing: story-writer.
3. S003-D2-02: Define the 5 field names for `last_hook_ts` JSON object (pre_tool_use, notification, stop, session_start, prompt_submit). Routing: architect (confirm field names from SS-daemon-lifecycle.md), then story-writer.
4. S003-D4-01: Add ordered 5-path list to test specification in Tasks. Routing: story-writer.
