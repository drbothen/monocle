---
document_type: story-uncertainty-review-master-inventory
version: "1.0"
status: stage-1-complete
producer: vsdd-factory:orchestrator
project: monocle
cycle: cycle-001
timestamp: 2026-05-20T07:00:00Z
inputs:
  - .factory/stories/S-001-cargo-workspace-ci-setup.md
  - .factory/stories/S-002-healthz-endpoint.md
  - .factory/stories/S-003-status-endpoint.md
  - .factory/stories/S-004-body-size-limit.md
  - .factory/stories/S-005-graceful-shutdown.md
  - .factory/stories/S-006-lock-file-lifecycle.md
  - .factory/stories/S-007-crash-recovery-checkpoint.md
  - .factory/stories/S-008-jsonl-ring-format-version.md
  - .factory/stories/S-009-auth-token-header-validation.md
  - .factory/stories/S-010-monocle-core-abi-version.md
  - .factory/stories/S-011-non-exhaustive-enum-policy.md
  - .factory/stories/S-012-factory-adapter-trait.md
  - .factory/stories/S-013-hook-envelope-proto-wire-format.md
  - .factory/stories/S-014-engine-module-trait.md
  - .factory/stories/S-015-claude-code-module-impl.md
  - .factory/stories/S-DTU-001-claude-code-hook-clone.md
  - .factory/stories/S-PHASE-3-PREP-spec-kit-mcp-integration.md
traces_to: "monocle Phase 2 GATE PASS WITH RESIDUAL D-159 / vsdd-factory issue #150 (https://github.com/drbothen/vsdd-factory/issues/150)"
---

# Master Inventory — Story Uncertainty Review cycle-001

## Stage 1 Summary

17 stories scanned by vsdd-factory:spec-reviewer across 4 batches (CALIBRATION + BATCH 2/3/4).
Stage 1 complete as of 2026-05-20. Stages 2-4 pending orchestrator dispatch.

**Verdict: 0 PASS / 8 PASS_WITH_OBSERVATIONS / 9 NEEDS_REVISION / 1 NEEDS_RESEARCH**

No story is fully PASS. All implementation stories have at least LOW observations.
9 stories have CRITICAL or HIGH findings requiring remediation before TDD dispatch.

---

## Verdict Distribution

| Story | Title | Points | Wave | Verdict |
|-------|-------|--------|------|---------|
| S-001 | Cargo Workspace Init + CI/DevOps Setup | 5 | 1 | PASS_WITH_OBSERVATIONS |
| S-002 | Healthz Endpoint | 3 | 2 | PASS_WITH_OBSERVATIONS |
| S-003 | Status Endpoint | 5 | 2 | NEEDS_REVISION |
| S-004 | Body Size Limit | 2 | 2 | PASS_WITH_OBSERVATIONS |
| S-005 | Graceful Shutdown | 5 | 2 | NEEDS_REVISION |
| S-006 | Lock File Atomic Lifecycle | 8 | 2 | NEEDS_REVISION |
| S-007 | Crash Recovery Checkpoint | 5 | 3 | NEEDS_REVISION |
| S-008 | JSONL Ring Format Version | 5 | 3 | PASS_WITH_OBSERVATIONS |
| S-009 | Auth Token Wire Format + Header Validation | 8 | 3 | NEEDS_REVISION |
| S-010 | monocle-core Foundation + ABI Version | 5 | 2 | PASS_WITH_OBSERVATIONS |
| S-011 | Non-Exhaustive Enum Policy | 3 | 2 | PASS_WITH_OBSERVATIONS |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | 8 | 3 | NEEDS_REVISION |
| S-013 | HookEnvelope Proto Wire Format | 5 | 2 | PASS_WITH_OBSERVATIONS |
| S-014 | EngineModule Trait Definition | 5 | 2 | NEEDS_REVISION |
| S-015 | ClaudeCodeModule Implementation | 8 | 3 | NEEDS_REVISION |
| S-DTU-001 | Claude Code Hook Protocol DTU Clone | 3 | 1 | PASS_WITH_OBSERVATIONS |
| S-PHASE-3-PREP | spec-kit-mcp Integration | 3 | 0 | NEEDS_RESEARCH |

---

## CRITICAL Findings Catalog

13 CRITICAL findings identified across the corpus.

| Finding ID | Story | Dimension | Summary |
|------------|-------|-----------|---------|
| S014-D2-01 | S-014 | API Accuracy | `HookEvent` vs `HookType` enum conflation — two distinct types; relationship undefined in story |
| S014-D3-01 | S-014 | Cross-Story | `HookType` declaration origin unresolved — S-011 references it, S-014 doesn't declare it, S-015 uses it |
| S003-D3-01 | S-003 | Cross-Story | `auth.rs` ownership split: S-003 creates auth middleware, S-009 extends/replaces — handoff undocumented |
| S009-D3-01 | S-009 | Cross-Story | Inherits S003-D3-01 — S-009 "extends" S-003 auth module without explicit stub/replace contract |
| S005-D3-01 | S-005 | Cross-Story | UDS control socket referenced by S-007 as "established in S-005" but S-005 contains no UDS implementation |
| S007-D3-01 | S-007 | Cross-Story | Inherits S005-D3-01 — S-007 recovery_available dispatch depends on a UDS socket not established by any story |
| S006-D1-01 | S-006 | Version Pin | `indexmap` crate referenced in Tasks but not in SS-deps-pin-manifest.md and not in Library table |
| S012-D1-01 | S-012 | Version Pin | `serde_yaml_ng 0.10` used by S-012 but absent from SS-deps-pin-manifest.md v1.1.17 |
| S012-D3-01 | S-012 | Cross-Story | `parse_frontmatter_field` unit tests mixed with VP-015 integration tests in single file — test org collision |
| S015-D2-01 | S-015 | API Accuracy | Inherits S014-D2-01 — `HookType` import path unknown until S-014 disambiguation resolved |
| S010-D3-01 | S-010 | Cross-Story | S-010 modifies S-003's handler file; both are Wave 2 but S-010 `depends_on` doesn't list S-003 |
| S004-D3-01 | S-004 | Cross-Story | MEDIUM promoted: authenticated router origin (S-003) not stated as prerequisite in S-004 dependency chain |
| S011-D2-01 | S-011 | API Accuracy | `EngineMetadataError` public enum not addressed by non-exhaustive policy; 9-enum list may be incomplete |

---

## Severity Distribution (approximate)

| Severity | Count |
|----------|-------|
| CRITICAL | 13 |
| HIGH | ~5 (S-PHASE-3-PREP D1-01, D2-01, D2-02) |
| MEDIUM | ~18 |
| LOW | ~15 |
| **Total** | **~51** |

---

## Cross-Cutting Patterns

Five patterns appear repeatedly across the corpus:

### Pattern 1: Auth-middleware ownership ambiguity (S-003 ↔ S-009)

S-003 creates `auth.rs` with middleware; S-009 extends `auth.rs` with full dual-accept
validation. The "stub vs complete" handoff is not documented in either story. Fix requires
both stories to describe the same handoff contract from their respective perspectives.

Affected: S-003, S-009.

### Pattern 2: `inputs` frontmatter missing referenced specs

Multiple stories cite specs in their body text (Architecture Compliance Rules, Tasks,
AC citations) but do not list those specs in the `inputs` frontmatter. This means the
implementer's context load for those stories will be incomplete.

Affected: S-014 (missing SS-engine-module.md), S-015 (missing SS-core-types-and-abi.md),
S-DTU-001 (missing SS-deps-pin-manifest.md).

### Pattern 3: Test function and test file naming drift

The corpus has inconsistent test naming: S-008 establishes `test_BC_RING_001_*` format in
the test function name; S-007 uses `test_BC_DAEMON_006_*` in the file spec; S-009 uses
`auth_header_rejection.rs` (mismatch with the story title "auth header VALIDATION");
S-004 doesn't specify test function names at all. A canonical test naming convention should
be established and applied uniformly.

Affected: S-002, S-004, S-009, and several others.

### Pattern 4: Cross-story handoff contracts under-specified

When Story B modifies a file created by Story A (both in the same wave), the dependency
must be in `depends_on` or the wave-schedule must enforce ordering. Several Wave 2 stories
modify files created by other Wave 2 stories without `depends_on` entries.

Affected: S-010 (modifies S-003 output), S-009 (extends S-003 output), S-006/S-003/S-009
(three stories touch `auth.rs`).

### Pattern 5: Anchors-by-section, not anchors-by-line

Several architecture compliance citations include specific line numbers from SS-*.md files
(e.g., "SS-core-types-and-abi.md lines 364–400"). Line numbers drift as files are updated.
References should anchor to section headings, not line numbers. The citations are otherwise
correct in content — only the stability of the reference form is at risk.

Affected: S-012, S-014, S-015.

---

## Research Queue (Stage 2)

Only one story requires external research:

| Story | Research Question | Source | Blocking? |
|-------|------------------|--------|----------|
| S-PHASE-3-PREP | vsdd-factory spec-kit-mcp rc.19+ release status and API surface | vsdd-factory GitHub releases | YES — story cannot be dispatched until research completes |

All other findings are resolvable from existing specs without external research.

---

## Stage 3 Dispatch Plan

### Routing to architect (before story-writer changes)

These items require an architectural decision before stories can be updated:

1. **S014-D2-01 / S014-D3-01 / S015-D2-01** — `HookType` vs `HookEvent` disambiguation and declaration origin. Architect confirms: where is `HookType` declared? Is it separate from `HookEvent`? Update SS-engine-module.md or SS-core-types-and-abi.md with the answer.
2. **S005-D3-01 / S007-D3-01** — UDS control socket origin. Architect confirms: which story establishes the UDS socket? Is it in S-005 (undocumented), or does a new story need to be created?
3. **S003-D3-01 / S009-D3-01** — Auth middleware stub vs complete pattern. Architect confirms: S-003 creates a STUB auth middleware; S-009 replaces it with the full dual-accept implementation. Confirm this is the intended pattern.
4. **S006-D1-01** — `indexmap` vs manually ordered struct decision. Architect recommends struct approach (consistent with S-008) and removes `indexmap` option.
5. **S012-D1-01** — Add `serde_yaml_ng 0.10` to SS-deps-pin-manifest.md with version pin and pin type.
6. **S011-D2-01** — Confirm whether `EngineMetadataError` is covered by non-exhaustive policy.

### Routing to story-writer (after architect decisions)

After architectural decisions are recorded, story-writer applies fixes to all affected stories:

- S-003: Add stub auth middleware documentation
- S-004: Add cross-story prerequisite note (S-003 authenticated router)
- S-005: Add UDS socket establishment (or remove the reference from S-007)
- S-006: Remove `indexmap` option; commit to manually ordered struct
- S-007: Update Previous Story Intelligence with correct UDS socket story reference
- S-008: Add `RING_FORMAT_VERSION` const declaration; add `RingBuffer::push()` signature
- S-009: Rename test file; add hook response body test case
- S-010: Add `depends_on: [S-003]` or document wave ordering
- S-011: Update canonical enum list if `EngineMetadataError` is added
- S-012: Split test files (factory_adapter_unit.rs + factory_self_referential.rs); add `serde_yaml_ng` to inputs; define `ConvergenceMetrics` fields
- S-014: Add SS-engine-module.md to inputs; define `HookType` declaration origin; add VP-019 assertion spec
- S-015: Add SS-core-types-and-abi.md to inputs; update `HookType` import path after S-014 fix

### Routing to product-owner

- S-PHASE-3-PREP: After Stage 2 research, update ACs to reflect actual spec-kit-mcp API surface.

---

## References

- vsdd-factory upstream issue #150: https://github.com/drbothen/vsdd-factory/issues/150
- Per-story assessment files: `cycle-001/S-NNN-assessment.md` (17 files, same directory)
- Phase 2 GATE PASS declaration: `.factory/STATE.md` D-159
- Tech debt register: `.factory/tech-debt-register.md` TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT
