---
document_type: behavioral-contract
level: L3
version: "1.0.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-20T21:00:00Z
phase: 1a
inputs:
  - {path: .factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md, version: "r1"}
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

# BC-HOOK-023: Content-Type and Content-Length Headers Are Always Set Explicitly

## Description

Every hook HTTP POST sets `Content-Type: application/json` and
`Content-Length: <byte-length>` explicitly. The Content-Length is the UTF-8 byte
length of the body (not the character count). Both headers are set on all five hook
types with identical code.

## Preconditions

1. An HTTP POST request body has been serialized to a JSON string.

## Postconditions

1. `Content-Type: application/json` header is set.
2. `Content-Length: <N>` header is set where `<N>` is the byte length of the body string in UTF-8.
3. Both headers are present on ALL five hook types.

## Invariants

1. Content-Length uses UTF-8 byte count, not character count. For ASCII-only bodies (typical for hook payloads), they are equal. For bodies with non-ASCII characters (e.g., CJK in tool_input), byte count exceeds character count.
2. The server parser uses Content-Length to determine end-of-body. Incorrect Content-Length causes server parse failure or request truncation.
3. Setting Content-Length explicitly prevents chunked transfer encoding.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Body contains non-ASCII characters (emoji in tool_input) | Content-Length = UTF-8 byte length (may be > char count); server parser receives correct byte count |
| EC-002 | Body is an empty object `{}` | Content-Length = 2 |
| EC-003 | Body is deeply nested JSON (100 nested objects) | Content-Length = full byte serialization length |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ASCII body `{"tool_name":"Bash"}` (20 chars) | Content-Length: 20 | happy-path |
| Body with emoji: `{"message":"👍"}` (unicode) | Content-Length = byte length of UTF-8 encoding of `{"message":"👍"}` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone sets Content-Type and Content-Length on all 5 hook POSTs | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — correct Content-Type and Content-Length headers ensure the daemon can correctly parse hook event bodies; incorrect headers cause ingestion failures |
| L2 Domain Invariants | DI-001 (tee invariant — a malformed Content-Length causes the daemon to misparse the body, preventing ring write; explicit Content-Length headers are a correctness requirement for DI-001) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-023 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31,35,38,41,44 (`'Content-Type':'application/json','Content-Length':Buffer.byteLength(body)`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-023 (gene-source: deep-hooks-r1 §6 BC-HOOK-023) |
| Test name | test_BC_HOOK_023_content_type_content_length_headers |

## Related BCs

- [BC-HOOK-036] — composes with: BC-HOOK-036 covers the Buffer.byteLength vs character count distinction

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (and identical at 35, 38, 41, 44) — `'Content-Type':'application/json','Content-Length':Buffer.byteLength(body)`.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
