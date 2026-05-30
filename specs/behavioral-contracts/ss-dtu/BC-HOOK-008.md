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

# BC-HOOK-008: Hooks-Settings.json Encoding: SetEscapeHTML(false) and 2-Space Indent

## Description

The hooks-settings.json file is JSON-encoded with HTML escaping DISABLED and
2-space pretty-print indentation. Disabling HTML escaping is critical because
the hook command strings contain JavaScript arrow functions (`=>`) that would be
mangled to `>=` by Go's default HTML-safe JSON encoder. Rust's `serde_json`
does NOT HTML-escape by default, so no special flag is needed in the Rust port.

## Preconditions

1. A hooks-settings.json is being serialized.

## Postconditions

1. Arrow function `=>` appears as literal `=>` in the output — NOT as `>=` or `>=`.
2. Output is pretty-printed with 2-space indentation (no tab indentation).
3. Angle brackets `<` and `>` in any string values appear as literal characters.
4. Ampersand `&` in any string values appears as literal `&`.

## Invariants

1. Go's `encoding/json` defaults to HTML-safe encoding (`>` → `>`). The gene source uses `SetEscapeHTML(false)` to override this.
2. Rust's `serde_json` does NOT HTML-escape by default — the Rust port is byte-compatible with the Go port without any special configuration.
3. The 2-space indent is cosmetic but verifiable; Claude Code parses the file as JSON so whitespace is irrelevant to parsing correctness. The spec matches the gene source for fidelity completeness.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Hook command string contains `=>` (arrow function) | Output contains literal `=>`, not `>=` |
| EC-002 | Hook command string contains `<` (e.g., in a comparison) | Output contains literal `<`, not `<` |
| EC-003 | Top-level object structure | One-level of pretty-print indentation; nested objects indented by 2 spaces |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Serialize hooks-settings.json | File content contains literal `=>` (no HTML escapes) | lint |
| Read file back | Valid JSON parses without error | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | Serialized hooks-settings.json contains no HTML-escaped characters for `>`, `<`, `&` | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the hooks-settings.json encoding contract is part of the hook injection mechanism that enables Claude Code to invoke the daemon's hook endpoints |
| L2 Domain Invariants | None directly (encoding is an implementation detail of the hook injection mechanism) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-008 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:58-61 (`enc.SetEscapeHTML(false); enc.SetIndent("", "  ")`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-008 (gene-source: deep-hooks-r1 §4 BC-HOOK-008) |
| Test name | test_BC_HOOK_008_json_no_html_escaping_2space_indent |

## Related BCs

- [BC-HOOK-007] — depends on: BC-HOOK-007 defines the key set that is serialized with this encoding

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:58-61 (`enc.SetEscapeHTML(false); enc.SetIndent("", "  ")`).
- Rust port note: serde_json does not HTML-escape by default; no special configuration needed.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
