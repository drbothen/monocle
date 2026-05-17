---
document_type: verification-property-index
level: L4
version: "1.1"
status: active
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "b09f325"
traces_to: prd.md
---

# Verification Property Index: monocle Phase 1

> **Source of truth** for all verification property IDs, titles, source BCs,
> proof methods, and file paths.
> VP frontmatter `source_bc:`, VP body Source Contract sections, story
> verification entries, and the PRD Verification Properties Index MUST all
> use IDs and titles from this table.
>
> **Append-only:** When a VP is withdrawn or replaced, mark it
> `lifecycle_status: withdrawn|retired` and add a `replacement:` column entry.
> Never remove a row or reuse an ID. See §Renumbering Appendix for old → new
> ID mappings.

---

## SS-01: Daemon Lifecycle VPs (10)

> Source-contract subsystem: BC-2.01.* (see `behavioral-contracts/BC-INDEX.md` §SS-01)
> Architecture source: `architecture/SS-daemon-lifecycle.md` v1.0.25
> Capability: CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management")

| VP ID | Title | Source BC | Proof Method | File | Old ID (PG-5) |
|-------|-------|-----------|--------------|------|---------------|
| VP-001 | Healthz Endpoint — Unauthenticated Liveness 200/503 | BC-2.01.001 | manual+proptest | vp-001-healthz-endpoint.md | VP-DAEMON-001 |
| VP-002 | Status Endpoint — Authenticated 10-Field JSON | BC-2.01.002 | manual+proptest | vp-002-status-endpoint.md | VP-DAEMON-002 |
| VP-003 | Body Size Limit — 256 KiB; HTTP 413 | BC-2.01.003 | manual+fuzz | vp-003-body-size-limit.md | VP-DAEMON-003 |
| VP-004 | Graceful Shutdown — 10-Second Drain + 5-Code Exit | BC-2.01.004 | manual | vp-004-graceful-shutdown.md | VP-DAEMON-004 |
| VP-005 | Lock File Lifecycle — Atomic Create, Pid Gate, Mode 0o600/0o700 | BC-2.01.005 | manual+mutation | vp-005-lock-file-lifecycle.md | VP-DAEMON-005 |
| VP-006 | Crash Recovery Checkpoint — JSON Write, Offer, Cleanup | BC-2.01.006 | manual+mutation | vp-006-crash-recovery-checkpoint.md | VP-DAEMON-006 |
| VP-007 | JSONL Ring Format-Version First Key | BC-2.01.007 | manual+mutation | vp-007-ring-format-version.md | VP-RING-001 |
| VP-008 | Auth Token Wire Format + Constant-Time Comparison | BC-2.01.008 | manual+fuzz | vp-008-auth-token-wire-format.md | VP-AUTH-001 |
| VP-009 | Auth Header Two-Body Taxonomy | BC-2.01.009 | manual+fuzz | vp-009-auth-header-validation.md | VP-AUTH-002 |
| VP-010 | Lock File `contract_version: 1` First Key | BC-2.01.010 | manual+mutation | vp-010-lock-file-contract-version.md | VP-LOCK-001 |

---

## SS-02: Core Types and ABI VPs (8)

> Source-contract subsystem: BC-2.02.* (see `behavioral-contracts/BC-INDEX.md` §SS-02)
> Architecture source: `architecture/SS-core-types-and-abi.md`
> Capability: CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction")

| VP ID | Title | Source BC | Proof Method | File | Old ID (PG-5) |
|-------|-------|-----------|--------------|------|---------------|
| VP-011 | ABI Version in `/status` Endpoint | BC-2.02.001 | integration-test | vp-011-abi-version-status-endpoint.md | VP-ABI-001 |
| VP-012 | `MONOCLE_ABI_VERSION` Pub Const Equals `1` | BC-2.02.002 | compile-time-check | vp-012-abi-version-crate-root.md | VP-ABI-002 |
| VP-013 | Non-Exhaustive Enum Policy (Modulo ADR-0004 Exemptions) | BC-2.02.003 | ast-audit+mutation-test | vp-013-non-exhaustive-enum-policy.md | VP-TYPES-001 |
| VP-014 | `FactoryAdapter` Trait Signature Stable | BC-2.02.004 | ast-audit | vp-014-factory-adapter-trait.md | VP-FACTORY-001 |
| VP-015 | `VsddFactoryAdapter::new` + Self-Reference Detection | BC-2.02.005 | integration-test+fuzz | vp-015-vsdd-factory-adapter.md | VP-FACTORY-002 |
| VP-016 | Proto Field Number 1 = `schema_version` in `HookEnvelope` | BC-2.02.006 | integration-test | vp-016-hook-envelope-proto-field-numbers.md | VP-PROTO-001a |
| VP-017 | Rust `HookEnvelope` Struct `pub schema_version: u32 = 1` | BC-2.02.007 | integration-test | vp-017-hook-envelope-schema-version-field.md | VP-PROTO-001b |
| VP-018 | `schema_version` Forward-Compat Contract (Phase 4 Dispatch) | BC-2.02.008 | integration-test+fuzz | vp-018-phase4-schema-version-validation.md | VP-PROTO-002 |

---

## SS-03: Engine Module VPs (4)

> Source-contract subsystem: BC-2.03.* (see `behavioral-contracts/BC-INDEX.md` §SS-03)
> Architecture source: `architecture/SS-engine-module.md`
> Capability: CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter")

| VP ID | Title | Source BC | Proof Method | File | Old ID (PG-5) |
|-------|-------|-----------|--------------|------|---------------|
| VP-019 | `EngineModule` Trait Signature Stable; `last_event_micros: Option<i64>` | BC-2.03.001 | ast-audit | vp-019-engine-module-trait.md | VP-ENGINE-001 |
| VP-020 | `ClaudeCodeModule::detect` Strict Basename Match | BC-2.03.002 | integration-test | vp-020-claude-code-module-impl.md | VP-ENGINE-002 |
| VP-021 | `metadata`/`enrich` Return `HomeUnresolvable` (All Four Home-Env Vars Unset) | BC-2.03.003 | integration-test | vp-021-home-unresolvable-error.md | VP-ENGINE-002-ERR |
| VP-022 | `hook_paths()` Returns Exactly 5 Entries — One per `HookType` Variant | BC-2.03.004 | integration-test | vp-022-claude-code-module-inherent-methods.md | VP-ENGINE-003 |

---

## Summary

- **SS-01 (Daemon Lifecycle):** 10 VPs sharded into individual files (Dispatch 5a).
- **SS-02 (Core Types and ABI):** 8 VPs sharded into individual files (Dispatch 5b — this dispatch).
- **SS-03 (Engine Module):** 4 VPs sharded into individual files (Dispatch 5b — this dispatch).
- **Total active Phase 1 VPs:** 22 (all sharded; monolithic
  `verification-properties.md` retired in Dispatch 5b).
- **Pending:** 0
- **Withdrawn:** 0
- **Retired:** 0

---

## Renumbering Appendix (Append-Only Protection)

This appendix preserves the PG-5 historical VP IDs against the new per-file
canonical IDs introduced in Dispatch 5a and Dispatch 5b.
Old IDs are NEVER reused for new properties; this table is the canonical
mapping for traceability into pre-Dispatch-5a artifacts.

| Old ID (PG-5 historical) | New ID | New File | Dispatch | Status |
|---------------------------|--------|----------|----------|--------|
| VP-DAEMON-001 | VP-001 | vp-001-healthz-endpoint.md | 5a | sharded |
| VP-DAEMON-002 | VP-002 | vp-002-status-endpoint.md | 5a | sharded |
| VP-DAEMON-003 | VP-003 | vp-003-body-size-limit.md | 5a | sharded |
| VP-DAEMON-004 | VP-004 | vp-004-graceful-shutdown.md | 5a | sharded |
| VP-DAEMON-005 | VP-005 | vp-005-lock-file-lifecycle.md | 5a | sharded |
| VP-DAEMON-006 | VP-006 | vp-006-crash-recovery-checkpoint.md | 5a | sharded |
| VP-RING-001 | VP-007 | vp-007-ring-format-version.md | 5a | sharded |
| VP-AUTH-001 | VP-008 | vp-008-auth-token-wire-format.md | 5a | sharded |
| VP-AUTH-002 | VP-009 | vp-009-auth-header-validation.md | 5a | sharded |
| VP-LOCK-001 | VP-010 | vp-010-lock-file-contract-version.md | 5a | sharded |
| VP-ABI-001 | VP-011 | vp-011-abi-version-status-endpoint.md | 5b | sharded |
| VP-ABI-002 | VP-012 | vp-012-abi-version-crate-root.md | 5b | sharded |
| VP-TYPES-001 | VP-013 | vp-013-non-exhaustive-enum-policy.md | 5b | sharded |
| VP-FACTORY-001 | VP-014 | vp-014-factory-adapter-trait.md | 5b | sharded |
| VP-FACTORY-002 | VP-015 | vp-015-vsdd-factory-adapter.md | 5b | sharded |
| VP-PROTO-001a | VP-016 | vp-016-hook-envelope-proto-field-numbers.md | 5b | sharded |
| VP-PROTO-001b | VP-017 | vp-017-hook-envelope-schema-version-field.md | 5b | sharded |
| VP-PROTO-002 | VP-018 | vp-018-phase4-schema-version-validation.md | 5b | sharded |
| VP-ENGINE-001 | VP-019 | vp-019-engine-module-trait.md | 5b | sharded |
| VP-ENGINE-002 | VP-020 | vp-020-claude-code-module-impl.md | 5b | sharded |
| VP-ENGINE-002-ERR | VP-021 | vp-021-home-unresolvable-error.md | 5b | sharded |
| VP-ENGINE-003 | VP-022 | vp-022-claude-code-module-inherent-methods.md | 5b | sharded |

---

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Source monolith (retired): `.factory/specs/verification-properties.md`
  v1.35 was the predecessor (commit 842402c). The monolith was deleted
  from the working tree in Dispatch 5b; per PG-5 historical preservation
  policy, the full content remains accessible via
  `git show 842402c:.factory/specs/verification-properties.md` and
  earlier commits. VP-INDEX.md is now the canonical entry point for
  Phase 1 verification properties.
- BC index: `behavioral-contracts/BC-INDEX.md` v1.1 (Dispatch 3 commit f259ade).
- PRD: `.factory/specs/prd.md` v1.26 (Dispatch 4 commit 1030c65).
- Architecture index: `architecture/ARCH-INDEX.md`.
- Template: `templates/L4-verification-property-template.md`.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md`.
