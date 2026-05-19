---
document_type: story-index
level: L4
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:30:00Z
phase: 2
inputs:
  - specs/prd.md
  - specs/behavioral-contracts/BC-INDEX.md
  - specs/verification-properties/VP-INDEX.md
  - specs/domain-spec/L2-INDEX.md
  - specs/architecture/ARCH-INDEX.md
  - specs/dtu-assessment.md
  - tech-debt-register.md
input-hash: "[pending-compute-input-hash]"
traces_to: specs/prd.md
---

# Story Index: monocle Phase 2

> **Source of truth** for all story IDs, status, points, wave assignments, and BC/VP traceability.
> Per artifact-path-registry.yaml: stories at `.factory/stories/S-{story-id}-{slug}.md`.

## Epics

| Epic ID | Name | Capability | Subsystem | Stories |
|---------|------|-----------|-----------|---------|
| EPIC-01 | Daemon Lifecycle | CAP-001 | SS-01 | S-001, S-002, S-003, S-004, S-005, S-006, S-007, S-008, S-009 |
| EPIC-02 | Core Types and ABI | CAP-002 | SS-02 | S-010, S-011, S-012, S-013 |
| EPIC-03 | Engine Module | CAP-003 | SS-03 | S-014, S-015 |
| EPIC-DTU | Claude Code Hook Protocol Clone | CAP-001 (DTU) | — | S-DTU-001 |
| EPIC-PREP | Phase 3 Pre-Implementation Prep | — | — | S-PHASE-3-PREP |

## Story Registry

| Story ID | Title | Epic | Points | Wave | Status | Blocks |
|----------|-------|------|--------|------|--------|--------|
| S-PHASE-3-PREP | spec-kit-mcp Integration Sweep | EPIC-PREP | 3 | 0 | draft | (Phase 3 gate) |
| S-DTU-001 | Claude Code Hook Protocol DTU Clone | EPIC-DTU | 3 | 1 | draft | S-009 |
| S-001 | Cargo Workspace Init + CI/DevOps Setup | EPIC-01 | 5 | 1 | draft | S-002..S-006, S-009..S-014 |
| S-002 | Healthz Endpoint | EPIC-01 | 3 | 2 | draft | S-003, S-005 |
| S-003 | Status Endpoint | EPIC-01 | 5 | 2 | draft | — |
| S-004 | Body Size Limit | EPIC-01 | 2 | 2 | draft | S-009 |
| S-005 | Graceful Shutdown | EPIC-01 | 5 | 2 | draft | S-007 |
| S-006 | Lock File Atomic Lifecycle | EPIC-01 | 8 | 2 | draft | S-007, S-008, S-009 |
| S-009 | Auth Token Wire Format + Header Validation | EPIC-01 | 8 | 2 | draft | S-008 |
| S-010 | monocle-core Crate + ABI Version Constant | EPIC-02 | 5 | 2 | draft | S-011, S-012, S-013, S-014 |
| S-011 | Non-Exhaustive Enum Policy | EPIC-02 | 3 | 2 | draft | S-012 |
| S-013 | HookEnvelope Proto Wire Format | EPIC-02 | 5 | 2 | draft | — |
| S-014 | EngineModule Trait Definition | EPIC-03 | 5 | 2 | draft | S-015 |
| S-007 | Crash Recovery Checkpoint | EPIC-01 | 5 | 3 | draft | — |
| S-008 | JSONL Ring Format Version | EPIC-01 | 5 | 3 | draft | — |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | EPIC-02 | 8 | 3 | draft | — |
| S-015 | ClaudeCodeModule Implementation | EPIC-03 | 8 | 3 | draft | — |

**Total stories:** 18 (16 product + 1 DTU + 1 prep)
**Total points (product):** 80 (excl. DTU 3 pts and PREP 3 pts)
**Total points (all):** 86

## Wave Summary

| Wave | Stories | Points | Description |
|------|---------|--------|-------------|
| Wave 0 | S-PHASE-3-PREP | 3 | Pre-Phase-3 gate (blocked on spec-kit-mcp rc.19+) |
| Wave 1 | S-DTU-001, S-001 | 8 | Foundation: DTU clone + workspace init |
| Wave 2 | S-002, S-003, S-004, S-005, S-006, S-009, S-010, S-011, S-013, S-014 | 49 | Core implementation (parallel-eligible within wave) |
| Wave 3 | S-007, S-008, S-012, S-015 | 26 | Dependent completions |

## BC Coverage Table

| BC ID | Title | Covering Story | AC | Full Coverage? |
|-------|-------|---------------|----|----------------|
| BC-2.01.001 | Healthz Endpoint | S-002 | AC-001..AC-006 | YES |
| BC-2.01.002 | Status Endpoint | S-003 | AC-001..AC-007 | YES |
| BC-2.01.003 | Body Size Limit | S-004 | AC-001..AC-005 | YES |
| BC-2.01.004 | Graceful Shutdown | S-005 | AC-001..AC-006 | YES |
| BC-2.01.005 | Lock File Atomic Lifecycle | S-006 | AC-001..AC-011 | YES |
| BC-2.01.006 | Crash Recovery Checkpoint | S-007 | AC-001..AC-006 | YES |
| BC-2.01.007 | JSONL Ring Format Version | S-001 (workspace), S-008 (impl) | AC-001..AC-006 | YES |
| BC-2.01.008 | Auth Token Wire Format | S-009 | AC-001..AC-003 | YES |
| BC-2.01.009 | Auth Header Validation | S-009 | AC-004..AC-010 | YES |
| BC-2.01.010 | Lock File Contract Version Field | S-006 | AC-010..AC-011 | YES |
| BC-2.02.001 | ABI Version in /status | S-010, S-003 | AC-003, AC-005 | YES |
| BC-2.02.002 | ABI Version Constant at Crate Root | S-010 | AC-001..AC-005 | YES |
| BC-2.02.003 | Non-Exhaustive Enum Policy | S-011 | AC-001..AC-004 | YES |
| BC-2.02.004 | FactoryAdapter Trait Definition | S-012 | AC-001..AC-004 | YES |
| BC-2.02.005 | VsddFactoryAdapter Implementation | S-012 | AC-005..AC-009 | YES |
| BC-2.02.006 | HookEnvelope Proto Field Number | S-013 | AC-001, AC-006 | YES |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version | S-013 | AC-002..AC-003 | YES |
| BC-2.02.008 | Phase 4 schema_version Validation | S-013 | AC-004..AC-005 | YES |
| BC-2.03.001 | EngineModule Trait Definition | S-014 | AC-001..AC-007 | YES |
| BC-2.03.002 | ClaudeCodeModule (Strict-Basename Detect) | S-015 | AC-001..AC-003, AC-009 | YES |
| BC-2.03.003 | HomeUnresolvable Error Contract | S-015 | AC-004..AC-005 | YES |
| BC-2.03.004 | ClaudeCodeModule Inherent Methods | S-015 | AC-006..AC-008 | YES |

**BC Coverage: 22/22 (100%)**

## VP Coverage Table

| VP ID | Title | Anchor Story | Story Where Test Lives |
|-------|-------|-------------|----------------------|
| VP-001 | Healthz Endpoint — 200/503 | S-002 | S-002 |
| VP-002 | Status Endpoint — 10 Required Fields | S-003 | S-003 |
| VP-003 | Body Size Limit — 256 KiB; HTTP 413 | S-004 | S-004 |
| VP-004 | Graceful Shutdown — 10-Second Drain | S-005 | S-005 |
| VP-005 | Lock File Lifecycle — Atomic Create + Modes | S-006 | S-006 |
| VP-006 | Crash Recovery Checkpoint | S-007 | S-007 |
| VP-007 | JSONL Ring Record — format_version First Key | S-008 | S-008 |
| VP-008 | Auth Token — Wire Format + Constant-Time | S-009 | S-009 |
| VP-009 | Auth Header Validation — Dual-Accept | S-009 | S-009 |
| VP-010 | Lock File contract_version: 1 First Key | S-006 | S-006 |
| VP-011 | ABI Version in /status Endpoint | S-003, S-010 | S-010 |
| VP-012 | MONOCLE_ABI_VERSION Pub Const Equals 1 | S-010 | S-010 |
| VP-013 | Non-Exhaustive Enum Policy | S-011 | S-011 |
| VP-014 | FactoryAdapter Trait Signature Stable | S-012 | S-012 |
| VP-015 | VsddFactoryAdapter Self-Referential Detection | S-012 | S-012 |
| VP-016 | Proto Field Number 1 = schema_version | S-013 | S-013 |
| VP-017 | HookEnvelope Rust Struct schema_version Field | S-013 | S-013 |
| VP-018 | schema_version Forward-Compat Contract | S-013 | S-013 |
| VP-019 | EngineModule Trait Signature Stable | S-014 | S-014 |
| VP-020 | ClaudeCodeModule::detect Strict Basename | S-015 | S-015 |
| VP-021 | metadata/enrich Return HomeUnresolvable | S-015 | S-015 |
| VP-022 | hook_paths() Returns Exactly 5 Entries | S-015 | S-015 |

**VP Coverage: 22/22 (100%)**

## NFR Coverage Table

| NFR ID | Category | Covering Story | Validation Method |
|--------|----------|---------------|-------------------|
| NFR-001 | Latency | Phase 3 integration test | Phase 3 story decomposition (load-test infra) |
| NFR-002 | Latency | Phase 3 integration test | Phase 3 story decomposition |
| NFR-003 | Latency | Phase 3 integration test (TUI) | Phase 3 story decomposition |
| NFR-004 | Security | S-009 | VP-008 OsRng source-grep; AC-001 |
| NFR-005 | Security | S-004 | VP-003 AC-001 |
| NFR-006 | Throughput | Phase 3 integration test | Phase 3 story decomposition (1000 events/sec) |
| NFR-007 | Build | S-001 | CI gate: rust-toolchain.toml AC-002, AC-004 |
| NFR-008 | Build | S-001 | CI gate: matrix AC-003 |
| NFR-009 | Security | S-006 | VP-005 Post-condition 1 (0o600 mode); AC-001 |
| NFR-010 | Correctness | S-009, S-003 | VP-008/VP-009 constant_time_eq source-grep |
| NFR-011 | Forward-compat | S-DTU-001 | DTU fidelity ≥0.95 fixture corpus |
| NFR-012 | Security | S-006 | VP-005 Post-condition 9 (0o700 mode); AC-006 |

**P0 NFR Coverage: 12/12 (100%)**

**NFR-001/002/003/006 deferred to Phase 3:** These NFRs validate TUI + load-test behaviors
that require Phase 3 infrastructure. They are NOT gaps — they are phased deliverables per
nfr-catalog.md §VP Probe Citations. Covered stories will be authored at Phase 3 entry.

## Error Code Coverage

| Error Code | Covering Story | AC Reference |
|-----------|---------------|--------------|
| E-AUTH-001 | S-009 | AC-004 |
| E-AUTH-002 | S-009 | AC-005, AC-006 |
| E-AUTH-003 | S-009 | AC-005; S-003 AC-002 |
| E-DAEMON-001 | S-004 | AC-001, AC-004 |
| E-DAEMON-002 | S-005 | AC-003 |
| E-DAEMON-003 | S-002 | AC-002 |
| E-DAEMON-004 | S-006 | AC-009 |
| E-LOCK-001 | S-006 | AC-003 |
| E-LOCK-002 | S-006 | AC-004 |
| E-LOCK-003 | S-006 | AC-010 |
| E-ENG-001 | S-015 | AC-005 |
| E-FACT-001 | S-012 | AC-008 |
| E-FACT-002 | S-012 | AC-008 |
| E-RING-001 | S-008 | AC-005 |
| E-PROTO-001 | S-013 | AC-004 |

**Error Code Coverage: 15/15 (100%)**

## Gap Register

| Gap ID | Level | Source | Justification | Resolution Target |
|--------|-------|--------|---------------|-------------------|
| GAP-P2-001 | L3 | NFR-001 (hook latency ≤300ms) | Requires Phase 3 load-test infrastructure not available in Phase 1; per nfr-catalog.md §VP Probe Citations | Phase 3 story decomposition at Phase 3 entry |
| GAP-P2-002 | L3 | NFR-002 (Notification latency ≤2000ms) | Same rationale as GAP-P2-001; Phase 3 infra required | Phase 3 story decomposition at Phase 3 entry |
| GAP-P2-003 | L3 | NFR-003 (TUI overlay render ≤100ms) | TUI permission overlay is Phase 3 deliverable; not Phase 1 scope | Phase 3 story decomposition at Phase 3 entry |
| GAP-P2-004 | L3 | NFR-006 (1000 events/sec throughput) | Phase 3 load-test infra required; bounded-channel DESIGN is Phase 1 (in S-008), sustained VALIDATION is Phase 3 | Phase 3 story decomposition at Phase 3 entry |

All gaps are L3 (NFR) deferred to Phase 3 per nfr-catalog.md authoritative ruling.
No L1 (BC clause) gaps. No L2 (edge case) gaps.

## §Trace v1.0

**Phase 2 story decomposition initial burst** (2026-05-19T04:30:00Z):
- 18 stories created: 16 product stories + 1 DTU (S-DTU-001) + 1 prep (S-PHASE-3-PREP)
- 22/22 BCs covered (100%)
- 22/22 VPs covered (100%)
- 15/15 error codes covered (100%)
- 12/12 P0 NFRs covered (NFR-001/002/003/006 deferred to Phase 3 per authoritative nfr-catalog.md ruling)
- 4-wave schedule (Wave 0 = Phase 3 prep gate, Wave 1 = foundation, Wave 2 = parallel impl, Wave 3 = dependents)
- S-PHASE-3-PREP created per TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE §Future Attachment obligation
- Dependency graph is acyclic (validated via topological sort; see dependency-graph.md)
