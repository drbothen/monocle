---
document_type: behavioral-contract-index
level: L3
version: "1.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T23:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "17f342c"
traces_to: prd.md
---

# Behavioral Contract Index: monocle Phase 1

> **Source of truth** for all behavioral contract IDs, titles, priorities, and file paths.
> BC frontmatter `subsystem:`, BC body references, story `bcs:` arrays, and the PRD
> Behavioral Contracts Index (§2) MUST all use IDs and titles from this table.
>
> **Append-only:** When a BC is retired or replaced, mark it `status: retired` and add a
> `replaced_by:` column entry. Never remove a row or reuse an ID.

---

## SS-01: Daemon Lifecycle

> Architecture source: `architecture/SS-daemon-lifecycle.md`
> ARCH-INDEX subsystem: SS-01
> Capability: CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.01.001 | Healthz Endpoint (Unauthenticated Liveness Probe) | P0 | active | ss-01/BC-2.01.001.md | BC-DAEMON-001 |
| BC-2.01.002 | Status Endpoint (Authenticated Daemon State) | P0 | active | ss-01/BC-2.01.002.md | BC-DAEMON-002 |
| BC-2.01.003 | Body Size Limit (256 KiB, HTTP 413) | P0 | active | ss-01/BC-2.01.003.md | BC-DAEMON-003 |
| BC-2.01.004 | Graceful Shutdown (10-Second Drain) | P0 | active | ss-01/BC-2.01.004.md | BC-DAEMON-004 |
| BC-2.01.005 | Lock File Atomic Lifecycle (Create + Pid Check + Cleanup) | P0 | active | ss-01/BC-2.01.005.md | BC-DAEMON-005 |
| BC-2.01.006 | Crash Recovery Checkpoint | P0 | active | ss-01/BC-2.01.006.md | BC-DAEMON-006 |
| BC-2.01.007 | JSONL Ring Format Version (FC-01) | P0 | active | ss-01/BC-2.01.007.md | BC-RING-001 |
| BC-2.01.008 | Auth Token Wire Format (FC-06) | P0 | active | ss-01/BC-2.01.008.md | BC-AUTH-001 |
| BC-2.01.009 | Auth Header Validation (Missing and Invalid Token) | P0 | active | ss-01/BC-2.01.009.md | BC-AUTH-002 |
| BC-2.01.010 | Lock File Contract Version Field | P0 | active | ss-01/BC-2.01.010.md | BC-LOCK-001 |

---

## SS-02: Core Types and ABI

> Architecture source: `architecture/SS-core-types-and-abi.md`
> ARCH-INDEX subsystem: SS-02
> Capability: CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.02.001 | ABI Version in /status Endpoint (FC-03) | P0 | active | ss-02/BC-2.02.001.md | BC-ABI-001 |
| BC-2.02.002 | ABI Version Constant at Crate Root (FC-03) | P0 | active | ss-02/BC-2.02.002.md | BC-ABI-002 |
| BC-2.02.003 | Non-Exhaustive Enum Policy (FC-02) | P0 | active | ss-02/BC-2.02.003.md | BC-TYPES-001 |
| BC-2.02.004 | FactoryAdapter Trait Definition (FC-04 CRITICAL) | P0 | active | ss-02/BC-2.02.004.md | BC-FACTORY-001 |
| BC-2.02.005 | VsddFactoryAdapter Implementation | P0 | active | ss-02/BC-2.02.005.md | BC-FACTORY-002 |
| BC-2.02.006 | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | P0 | active | ss-02/BC-2.02.006.md | BC-PROTO-001a |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | P0 | active | ss-02/BC-2.02.007.md | BC-PROTO-001b |
| BC-2.02.008 | Phase 4 schema_version Validation Requirement (FC-05) | P1 | active | ss-02/BC-2.02.008.md | BC-PROTO-002 |

---

## SS-03: Engine Module

> Architecture source: `architecture/SS-engine-module.md`
> ARCH-INDEX subsystem: SS-03
> Capability: CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter")

| BC ID | Title | Priority | Status | File | Old ID (historical) |
|-------|-------|----------|--------|------|---------------------|
| BC-2.03.001 | EngineModule Trait Definition | P0 | active | ss-03/BC-2.03.001.md | BC-ENGINE-001 |
| BC-2.03.002 | ClaudeCodeModule Implementation (Strict-Basename Detect) | P0 | active | ss-03/BC-2.03.002.md | BC-ENGINE-002 |
| BC-2.03.003 | HomeUnresolvable Error Contract | P0 | active | ss-03/BC-2.03.003.md | BC-ENGINE-002-ERR |
| BC-2.03.004 | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | P0 | active | ss-03/BC-2.03.004.md | BC-ENGINE-003 |

---

## Summary

| Subsystem | Total BCs | Active | Pending |
|-----------|-----------|--------|---------|
| SS-01 Daemon Lifecycle | 10 | 10 | 0 |
| SS-02 Core Types and ABI | 8 | 8 | 0 |
| SS-03 Engine Module | 4 | 4 | 0 |
| **Total** | **22** | **22** | **0** |

---

## Renumbering Map (Old ID → New ID)

> Append-only ID protection per audit §663-714. Old IDs are preserved here for
> cross-reference from git history, test names, and PRD §7 Requirements Traceability Matrix.
> Old IDs are NOT reused. New IDs follow the BC-S.SS.NNN scheme.

| Old ID | New ID | Title | Subsystem |
|--------|--------|-------|-----------|
| BC-DAEMON-001 | BC-2.01.001 | Healthz Endpoint (Unauthenticated Liveness Probe) | SS-01 |
| BC-DAEMON-002 | BC-2.01.002 | Status Endpoint (Authenticated Daemon State) | SS-01 |
| BC-DAEMON-003 | BC-2.01.003 | Body Size Limit (256 KiB, HTTP 413) | SS-01 |
| BC-DAEMON-004 | BC-2.01.004 | Graceful Shutdown (10-Second Drain) | SS-01 |
| BC-DAEMON-005 | BC-2.01.005 | Lock File Atomic Lifecycle (Create + Pid Check + Cleanup) | SS-01 |
| BC-DAEMON-006 | BC-2.01.006 | Crash Recovery Checkpoint | SS-01 |
| BC-RING-001 | BC-2.01.007 | JSONL Ring Format Version (FC-01) | SS-01 |
| BC-AUTH-001 | BC-2.01.008 | Auth Token Wire Format (FC-06) | SS-01 |
| BC-AUTH-002 | BC-2.01.009 | Auth Header Validation (Missing and Invalid Token) | SS-01 |
| BC-LOCK-001 | BC-2.01.010 | Lock File Contract Version Field | SS-01 |
| BC-ABI-001 | BC-2.02.001 | ABI Version in /status Endpoint (FC-03) | SS-02 |
| BC-ABI-002 | BC-2.02.002 | ABI Version Constant at Crate Root (FC-03) | SS-02 |
| BC-TYPES-001 | BC-2.02.003 | Non-Exhaustive Enum Policy (FC-02) | SS-02 |
| BC-FACTORY-001 | BC-2.02.004 | FactoryAdapter Trait Definition (FC-04 CRITICAL) | SS-02 |
| BC-FACTORY-002 | BC-2.02.005 | VsddFactoryAdapter Implementation | SS-02 |
| BC-PROTO-001a | BC-2.02.006 | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | SS-02 |
| BC-PROTO-001b | BC-2.02.007 | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | SS-02 |
| BC-PROTO-002 | BC-2.02.008 | Phase 4 schema_version Validation Requirement (FC-05) | SS-02 |
| BC-ENGINE-001 | BC-2.03.001 | EngineModule Trait Definition | SS-03 |
| BC-ENGINE-002 | BC-2.03.002 | ClaudeCodeModule Implementation (Strict-Basename Detect) | SS-03 |
| BC-ENGINE-002-ERR | BC-2.03.003 | HomeUnresolvable Error Contract | SS-03 |
| BC-ENGINE-003 | BC-2.03.004 | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | SS-03 |

---

## §Trace v1.1

**Template compliance Dispatch 3 of 7+** (2026-05-17T12:00:00Z):
- SS-02 section: 8 BC rows flipped from `pending-dispatch-3` to `active`.
  Files created at `.factory/specs/behavioral-contracts/ss-02/` (BC-2.02.001..BC-2.02.008).
- SS-03 section: 4 BC rows flipped from `pending-dispatch-3` to `active`.
  Files created at `.factory/specs/behavioral-contracts/ss-03/` (BC-2.03.001..BC-2.03.004).
- Summary table: all 22 BCs active (10 SS-01 + 8 SS-02 + 4 SS-03); 0 pending.
- Index version bumped: 1.0 → 1.1.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T12:00:00Z >= chain high-water 2026-05-17T11:30:00Z.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md`.
- Next: Dispatch 4 (PO) reduces PRD §3/§4/§5 to index + creates 4 prd-supplements.

## §Trace v1.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure — 22-BC DI mapping sweep** (2026-05-17T18:00:00Z):
- All 22 BC files updated (SS-01 × 10, SS-02 × 8, SS-03 × 4).
- Per-file L2 Domain Invariants cells replaced from stale "N/A" text to canonical DI-NNN citations.
- 2 stale VP IDs corrected in body prose: BC-2.01.005 (`VP-DAEMON-005` → `VP-005`) and BC-2.01.006 (`VP-DAEMON-006` → `VP-006`).
- 0 stale BC IDs found in non-historical body prose across all 22 files.
- Per-file version bumps: all files that were at v1.0 incremented to v1.0.1; files at v1.0.1 incremented to v1.0.2.
- BC-INDEX version bumped: 1.1 → 1.2.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T12:00:00Z (v1.1).
- SE-17g META audit: see commit §Trace entry for zero-remaining "N/A — no domain-spec/invariants.md" re-grep result.

**Template compliance Dispatch 2 of 7+** (2026-05-17T11:30:00Z):
- Created as new artifact; no prior version.
- SS-01 section: 10 BC rows filled (BC-2.01.001..BC-2.01.010), all active.
  Files created at `.factory/specs/behavioral-contracts/ss-01/`.
- SS-02 section: 8 BC rows with `pending-dispatch-3` status; file paths pre-registered.
- SS-03 section: 4 BC rows with `pending-dispatch-3` status; file paths pre-registered.
- Renumbering map: all 22 old IDs (BC-DOMAIN-NNN) mapped to new BC-S.SS.NNN IDs per
  append-only ID protection (audit §663-714).
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:30:00Z >= chain high-water 2026-05-17T11:00:00Z.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md`.

## §Trace v1.3

**T-128n Part 1 — F-R105 closure chain Round 4: BC-2.01.009 ADR-0005 dual-accept propagation** (2026-05-17T20:00:00Z):
- BC-2.01.009 body updated: postconditions 1-3 expanded to dual-accept (ADR-0005); postcondition 4 added (both-headers-present canonical priority); 3 new edge cases (EC-010/EC-011/EC-012); 2 new test vectors (alias wrong-secret → 401; alias correct-secret → 200 + WARN). BC-2.01.009 version: 1.0.1 → 1.0.2.
- BC-INDEX title for BC-2.01.009 unchanged: "Auth Header Validation (Missing and Invalid Token)" — H1 is stable.
- BC-INDEX status unchanged: active. No BC removals or retirements in this burst.
- SE-16d monotonicity PASS: 2026-05-17T20:00:00Z > prior 2026-05-17T18:00:00Z (v1.2).

## §Trace v1.4

**F-R106 Round 5A — BC scope fixes: CRITICAL PC-4 contradiction + fabrication removal + stale-ID sweep + §Trace reorder** (2026-05-17T23:00:00Z):

BC version bumps in this dispatch:
- BC-2.01.008: v1.0.2 → v1.0.3 (F-R106-2 CRITICAL: PC-4 rewritten to enumerate both canonical `X-Monocle-Authorization` and alias `X-Claude-Code-Ide-Authorization` per ADR-0005 dual-accept; Architecture Source row updated to include ADR-0005)
- BC-2.01.009: v1.0.2 → v1.0.3 (F-R106-7 HIGH: fabricated `(F-FC-I005 Phase 4 OAuth2 clarification)` parenthetical removed from Forward Compat Contract row; replaced with canonical `FC-06 (versioned auth token prefix)`)
- BC-2.01.005: v1.0.1 → v1.0.2 (F-R106-11 MED: stale `BC-ENGINE-002-ERR` in Invariant 4 updated to `BC-2.03.003 (HomeUnresolvable; renumbered from BC-ENGINE-002-ERR per BC-INDEX §Renumbering Map)`)
- BC-2.01.002: v1.0.1 → v1.0.2 (F-R106-12 MED: redundant `(BC-AUTH-002)` parenthetical removed from Postcondition 2 cross-reference to BC-2.01.009)
- BC-2.01.003: v1.0.1 → v1.0.2 (F-R106-12 MED: stale `BC-RING-001 EC-002` in Related BCs canonicalized to `BC-2.01.007 EC-002`)
- BC-2.01.007: v1.0.1 → v1.0.2 (F-R106-12 MED: stale self-referential `BC-RING-001 EC-002` in Related BCs canonicalized to `BC-2.01.007 EC-002`)

BC-INDEX structural fix:
- F-R106-13 MED: §Trace sections were non-monotonic (v1.1, v1.3, v1.2). Reordered to ascending (v1.1 → v1.2 → v1.3 → v1.4). Content of each section preserved verbatim.

BC-INDEX titles unchanged: all 22 BC H1 headings are stable. No BC retirements or removals.
SE-17g META audit: re-grep for stale old-form BC ID parentheticals in ss-01/ body prose — see post-fix verification below.
SE-16d monotonicity PASS: 2026-05-17T23:00:00Z > prior 2026-05-17T20:00:00Z (v1.3).
