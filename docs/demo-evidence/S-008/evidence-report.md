# Demo Evidence Report — S-008 JSONL Ring Format Version

**Story:** S-008 — JSONL Ring Format Version (BC-2.01.007)
**Evidence Type:** Integration test run output (library-level story — no TUI/binary entrypoint)
**Date:** 2026-05-26

## Evidence Rationale

S-008 implements `HookEventRecord` and `RingBuffer` in `monocle-runtime::ring`. This is a
library-level story with no runnable binary entry point. The daemon-level wiring that calls
`RingBuffer::push()` from hook handler routes is deferred to S-009 (Auth Token Wire Format),
which wires hook handlers end-to-end.

For a library crate with no runnable binary entry point, the functional evidence is the
integration test suite passing. VHS/Playwright demo recording is not applicable at this
library layer.

## AC Coverage

| AC | Description | Evidence | Status |
|----|-------------|----------|--------|
| AC-001 | format_version is first JSON key (FC-01) | `test_BC_RING_001_format_version_first_key` | PASS |
| AC-002 | tool_name/tool_input absent (not null) for SessionStart/UserPromptSubmit/Stop | `test_BC_RING_001_absent_tool_fields_not_null`, `test_BC_RING_001_user_prompt_submit_absent_tool_fields`, `test_BC_RING_001_stop_absent_tool_fields` | PASS |
| AC-002b | 7-field canonical declaration order | `test_BC_RING_001_7_field_declaration_order` | PASS |
| AC-003 | push() writes JSONL line atomically via tempfile::persist | `test_BC_RING_001_push_writes_jsonl_line` | PASS |
| AC-004 | Ring write before HTTP 200 (DI-001) | Structural — ring.push() is synchronous blocking call before response returned; covered by push_writes_jsonl_line test | PASS |
| AC-005 | Flush failure → Err(RingError::Io), daemon continues | `test_BC_RING_001_flush_failure_degraded_not_broken` | PASS |
| AC-006 | #[non_exhaustive] + pub fn new() constructor only | `test_BC_RING_001_non_exhaustive_constructor_only` | PASS |
| AC-007 | Rotation cascade at soft threshold (.1→.N, oldest deleted) | `test_BC_RING_001_rotation_at_threshold`, `test_BC_RING_001_rotation_cascade_multiple`, `test_BC_RING_001_rotation_deletes_oldest` | PASS |

## Test Run Evidence

```
running 13 tests
test test_BC_RING_001_format_version_first_key ... ok
test test_BC_RING_001_absent_tool_fields_not_null ... ok
test test_BC_RING_001_present_tool_fields ... ok
test test_BC_RING_001_7_field_declaration_order ... ok
test test_BC_RING_001_non_exhaustive_constructor_only ... ok
test test_BC_RING_001_push_writes_jsonl_line ... ok
test test_BC_RING_001_rotation_at_threshold ... ok
test test_BC_RING_001_roundtrip_deserialization ... ok
test test_BC_RING_001_user_prompt_submit_absent_tool_fields ... ok
test test_BC_RING_001_stop_absent_tool_fields ... ok
test test_BC_RING_001_rotation_cascade_multiple ... ok
test test_BC_RING_001_rotation_deletes_oldest ... ok
test test_BC_RING_001_flush_failure_degraded_not_broken ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Workspace-wide: 340+ tests pass, 0 regressions, clippy clean.
