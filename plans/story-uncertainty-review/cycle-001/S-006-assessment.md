---
document_type: story-uncertainty-assessment
story_id: S-006
story_version: "1.4"
story_title: Lock File Atomic Lifecycle (Create + Pid Check + Cleanup)
assessment_batch: BATCH-3
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_REVISION
---

# Story Assessment: S-006

## Verdict

**NEEDS_REVISION** — One CRITICAL finding: S-006 is the most complex story in the corpus
(8 points, 14 ACs) and has an `indexmap` reference in Tasks that is not in the Library &
Framework Requirements table and not in SS-deps-pin-manifest.md. If `indexmap` is the chosen
serialization approach for ordered JSON fields, it must be pinned.

## Summary

S-006 covers the lock file atomic lifecycle including auth token generation (via
`monocle_runtime::auth::generate_session_token()`), runtime directory creation, PID liveness
checks, stale lock cleanup, and all three EC-011/EC-012/EC-013 edge cases. This is the
highest-complexity Wave 2 story. The auth token generation (`generate_session_token()`) is
fully specified and unambiguous. The critical finding is the `indexmap` dependency ambiguity
in AC-002 / Tasks.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S006-D1-01 | CRITICAL | AC-002 states "use an ordered serialization approach (use `indexmap` or manually ordered struct with `#[serde(rename_all = "camelCase")]`)". The Tasks block repeats "Use `indexmap` or manually ordered struct." `indexmap` is not listed in the Library & Framework Requirements table and is not in SS-deps-pin-manifest.md v1.1.17. If `indexmap` is used, it must be pinned. If the manually ordered struct approach is used, `indexmap` is unnecessary. The story must commit to ONE approach and remove the ambiguity. A manually ordered struct with `serde` field-declaration order is simpler (same approach as `HookEventRecord` in S-008) and requires no new dependency. Recommend removing `indexmap` option and specifying the manually ordered struct approach exclusively. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S006-D2-01 | MEDIUM | AC-008 specifies "macOS platform fallback using `data_local_dir()`" from `directories`. The actual API is `ProjectDirs::data_local_dir()` (a method on a `ProjectDirs` instance). But `ProjectDirs::runtime_dir()` returns `Option<&Path>` — if it returns `None` (on macOS), the fallback is `data_local_dir()`. The story should specify the exact call chain: `project_dirs.runtime_dir().unwrap_or_else(|| project_dirs.data_local_dir())`. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S006-D3-01 | LOW | S-006 creates `monocle-runtime/src/auth.rs` with `generate_session_token()`. S-003 also creates `monocle-runtime/src/auth.rs` (per S-003's File Structure Requirements: "auth.rs — auth middleware"). S-009 also modifies `monocle-runtime/src/auth.rs`. Three stories touch the same file. The creation/ownership sequencing is: S-006 creates `auth.rs` with `generate_session_token()` (token generation function); S-003 creates auth middleware in the same file; S-009 extends with `validate_auth_header()`. The Previous Story Intelligence in S-009 acknowledges this correctly. But S-006 should also note that `auth.rs` will be extended by S-003 and S-009 to avoid implementer surprise. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | The integration test list is comprehensive (11 test scenarios). All EC edge cases are covered. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter complete. 3 BCs in frontmatter match the traces_to text. |

## Research Queue

None. The `indexmap` vs manually ordered struct decision is an architectural choice resolvable from existing specs (S-008 uses the struct approach; recommend consistency).

## Recommended Fixes

1. S006-D1-01 (CRITICAL): Remove `indexmap` option from AC-002 and Tasks. Commit to the manually ordered struct approach (consistent with S-008's `HookEventRecord`). If `indexmap` is preferred by architect, add it to SS-deps-pin-manifest.md with a version pin first. Routing: architect (decision), then story-writer (update).
2. S006-D2-01 (MEDIUM): Add exact call chain `project_dirs.runtime_dir().unwrap_or_else(|| project_dirs.data_local_dir())` to AC-008. Routing: story-writer.
3. S006-D3-01 (LOW): Add note to S-006 File Structure Requirements that `auth.rs` will be extended by S-003 and S-009. Routing: story-writer.
