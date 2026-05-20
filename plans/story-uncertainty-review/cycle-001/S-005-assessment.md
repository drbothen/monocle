---
document_type: story-uncertainty-assessment
story_id: S-005
story_version: "1.4"
story_title: "Graceful Shutdown (10-Second Drain)"
assessment_batch: BATCH-2
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_REVISION
---

# Story Assessment: S-005

## Verdict

**NEEDS_REVISION** — One CRITICAL finding: S-005 references "UDS control socket established
in S-005" in S-007's Previous Story Intelligence, but S-005's own story spec says nothing
about establishing a UDS control socket. The UDS socket is either established by S-005 (not
documented) or by a different story (gap in the dependency chain).

## Summary

S-005 (Graceful Shutdown) is one of the more complex Phase 1 stories. The exit code taxonomy
(AC-004) is exceptionally detailed and matches BC-2.01.004 PC-8 precisely. The dual-accept
auth requirement on POST /shutdown (AC-002, AC-006) is well-specified with cross-citations to
ADR-0005. The critical finding is that S-007 (Crash Recovery Checkpoint) cites "UDS control
socket established in S-005" but S-005's story spec makes no mention of a UDS socket — only
the HTTP server graceful shutdown.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | `tokio =1.52`, `axum =0.8.9`, `nix 0.30` are correctly specified. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S005-D2-01 | MEDIUM | AC-004 specifies exit code `2` as "hard-killed by a second authenticated POST /shutdown during drain." The note "(admin forced-stop)" is correct. But the story says this is "outside the POSIX 128+N space... and distinct from startup-failure exit 1." Exit code `2` in POSIX means "misuse of shell built-ins" in bash, but for a Rust binary it has no reserved meaning. This is acceptable but should be documented with a rationale note: "exit 2 is chosen for forced-stop because it is in the 1-127 range (application-defined), distinct from exit 1 (startup failure), and distinct from POSIX signal exits (128+N)." |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S005-D3-01 | CRITICAL | S-007's Previous Story Intelligence states: "UDS control socket established in S-005 (graceful shutdown signal path) — reuse the socket plumbing for the `recovery_available` message dispatch." But S-005's story spec has NO mention of a Unix Domain Socket (UDS). S-005 only covers HTTP shutdown endpoint and SIGTERM/SIGINT signal handling. If the UDS control socket is established by S-005, it must be added to S-005's ACs and File Structure Requirements. If it is established by a different story (or is implied by the HTTP axum server), this must be clarified in S-007's Previous Story Intelligence. This gap will cause S-007's implementer to receive a codebase without the UDS socket they expect. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| S005-D4-01 | LOW | Tasks specifies "POST /shutdown alias auth → 200 + WARN log + shutdown initiated" as an integration test. The WARN log text is not specified in S-005 (it is specified in BC-2.01.009 as E-AUTH-003 "WARN: hook auth via X-Claude-Code-Ide-Authorization..."). The test-writer needs the exact log text to assert against. Either add it here or cross-cite the BC explicitly in the test task. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter is complete. inputs are versioned. |

## Research Queue

None. The UDS socket origin question is a spec gap that architect or story-writer can resolve by reading the SS-daemon-lifecycle.md UDS section.

## Recommended Fixes

1. S005-D3-01 (CRITICAL): Either (a) add UDS socket establishment to S-005 ACs and File Structure Requirements, or (b) clarify in S-007 Previous Story Intelligence which story actually establishes the UDS socket. Check SS-daemon-lifecycle.md for the canonical answer. Routing: architect (to confirm UDS socket lifecycle), then story-writer.
2. S005-D2-01: Add rationale note explaining the choice of exit code 2 for forced-stop. Routing: story-writer.
3. S005-D4-01: Add exact E-AUTH-003 log text to Tasks test specification. Routing: story-writer.
