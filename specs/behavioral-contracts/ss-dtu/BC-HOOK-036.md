---
document_type: behavioral-contract
level: L3
version: "1.0.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-20T21:00:00Z
phase: 1a
inputs:
  - {path: .factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r2.md, version: "r2"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
input-hash: "[live-state]"
traces_to: prd.md
origin: gene-transfusion
subsystem: SS-01
capability: CAP-001
dtu_service: claude-code-hook-protocol
gene_source: any-context-lazyclaude/internal/core/config/hooks.go
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-HOOK-036: Buffer.byteLength(body) Returns UTF-8 Byte Length, Not Character Count

## Description

The `Content-Length` header is computed using `Buffer.byteLength(body)`, which returns
the UTF-8 byte length of the string. For ASCII-only bodies (typical), byte length
equals character count. For bodies containing non-ASCII characters (emoji, CJK
characters in `tool_input`), byte length exceeds character count. The Rust port
using `body.len()` on a `String` (which is already UTF-8 bytes) is correct.

## Preconditions

1. A hook POST body is being constructed as a JSON string.
2. The body may contain non-ASCII characters.

## Postconditions

1. `Content-Length: N` where `N` = UTF-8 byte count of the body string.
2. For ASCII-only bodies: `N` = character count.
3. For non-ASCII bodies: `N` > character count.
4. The HTTP server receives the correct number of bytes to read.

## Invariants

1. Using `body.chars().count()` (character count) in the Rust port would produce incorrect Content-Length for non-ASCII payloads — this is a bug.
2. `String.len()` in Rust returns byte count (UTF-8 encoded), which is the correct value.
3. `body.len()` in Rust is equivalent to `Buffer.byteLength(body)` in Node.js.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | tool_input contains emoji `{"tool_input":{"text":"👍"}}` | UTF-8 bytes for 👍 is 4 bytes; Content-Length = char_length + 3 extra bytes |
| EC-002 | tool_input contains CJK characters `{"tool_input":{"q":"中"}}` | 中 = 3 bytes in UTF-8; Content-Length includes 3 bytes for 中 |
| EC-003 | Pure ASCII body (typical case) | byte count == char count; Content-Length is correct either way |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Body = `{"tool_name":"Bash"}` (ASCII) | Content-Length = 20 | happy-path |
| Body containing 👍 | Content-Length = actual UTF-8 byte count | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone Content-Length is computed as UTF-8 byte length (body.len() in Rust) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — correct Content-Length computation is required for successful HTTP body delivery to the daemon's hook ingestion endpoints |
| L2 Domain Invariants | DI-001 (tee invariant — incorrect Content-Length causes HTTP parse failure at the daemon; the daemon cannot write the ring entry for an unparseable body) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-036 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (`'Content-Length':Buffer.byteLength(body)` — UTF-8 byte length) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-036 (gene-source: deep-hooks-r2 §2 BC-HOOK-036) |
| Test name | test_BC_HOOK_036_content_length_utf8_byte_count |

## Related BCs

- [BC-HOOK-023] — depends on: BC-HOOK-023 covers the requirement to set Content-Length; this BC covers the computation method

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (`Buffer.byteLength(body)` — UTF-8 byte length per Node.js default).
- Rust port note: `body.len()` on `String` is UTF-8 byte count — correct; `body.chars().count()` would be wrong for non-ASCII.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
