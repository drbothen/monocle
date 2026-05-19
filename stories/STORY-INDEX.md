---
document_type: story-index
level: L4
version: "1.4"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:30:00Z
phase: 2
inputs:
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/domain-spec/L2-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: ".factory/specs/prd.md v1.26.15"
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
| S-001 | Cargo Workspace Init + CI/DevOps Setup | EPIC-01 | 5 | 1 | draft | S-002, S-003, S-004, S-005, S-006, S-010, S-013, S-014 |
| S-002 | Healthz Endpoint | EPIC-01 | 3 | 2 | draft | S-003, S-005 |
| S-003 | Status Endpoint | EPIC-01 | 5 | 2 | draft | — |
| S-004 | Body Size Limit | EPIC-01 | 2 | 2 | draft | S-009 |
| S-005 | Graceful Shutdown | EPIC-01 | 5 | 2 | draft | — |
| S-006 | Lock File Atomic Lifecycle | EPIC-01 | 8 | 2 | draft | S-007, S-008 |
| S-009 | Auth Token Wire Format + Header Validation | EPIC-01 | 8 | 3 | draft | — |
| S-010 | monocle-core Crate + ABI Version Constant | EPIC-02 | 5 | 2 | draft | S-011, S-012, S-013, S-014 |
| S-011 | Non-Exhaustive Enum Policy | EPIC-02 | 3 | 2 | draft | S-012 |
| S-013 | HookEnvelope Proto Wire Format | EPIC-02 | 5 | 2 | draft | — |
| S-014 | EngineModule Trait Definition | EPIC-03 | 5 | 2 | draft | S-015 |
| S-007 | Crash Recovery Checkpoint | EPIC-01 | 5 | 3 | draft | — |
| S-008 | JSONL Ring Format Version | EPIC-01 | 5 | 3 | draft | — |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | EPIC-02 | 8 | 3 | draft | — |
| S-015 | ClaudeCodeModule Implementation | EPIC-03 | 8 | 3 | draft | — |

**Total stories:** 17 (15 product + 1 DTU + 1 prep)
**Total points (product):** 80 (excl. DTU 3 pts and PREP 3 pts)
**Total points (all):** 86

## Wave Summary

| Wave | Stories | Points | Description |
|------|---------|--------|-------------|
| Wave 0 | S-PHASE-3-PREP | 3 | Pre-Phase-3 gate (blocked on spec-kit-mcp rc.19+) |
| Wave 1 | S-DTU-001, S-001 | 8 | Foundation: DTU clone + workspace init |
| Wave 2 | S-002, S-003, S-004, S-005, S-006, S-010, S-011, S-013, S-014 | 41 | Core implementation (parallel-eligible within wave) |
| Wave 3 | S-007, S-008, S-009, S-012, S-015 | 34 | Dependent completions (S-009 moved per Decision 1: S-008→S-009 dependency) |

## BC Coverage Table

| BC ID | Title | Covering Story | AC | Full Coverage? |
|-------|-------|---------------|----|----------------|
| BC-2.01.001 | Healthz Endpoint | S-002 | AC-001..AC-006 | YES |
| BC-2.01.002 | Status Endpoint | S-003 | AC-001..AC-007 | YES |
| BC-2.01.003 | Body Size Limit | S-004 | AC-001..AC-005 | YES |
| BC-2.01.004 | Graceful Shutdown | S-005 | AC-001..AC-006 | YES |
| BC-2.01.005 | Lock File Atomic Lifecycle | S-006 | AC-001..AC-011 | YES |
| BC-2.01.006 | Crash Recovery Checkpoint | S-007 | AC-001..AC-006 | YES |
| BC-2.01.007 | JSONL Ring Format Version | S-008 | AC-001..AC-007 | YES |
| BC-2.01.008 | Auth Token Wire Format | S-006, S-009 | S-006: AC-014 (token generation); S-009: AC-001..AC-003 | YES |
| BC-2.01.009 | Auth Header Validation | S-009 | AC-004..AC-010 | YES |
| BC-2.01.010 | Lock File Contract Version Field | S-006 | AC-010..AC-011 | YES |
| BC-2.02.001 | ABI Version in /status | S-010, S-003 | S-010: AC-003, AC-005; S-003: AC-005 | YES |
| BC-2.02.002 | ABI Version Constant at Crate Root | S-010 | AC-001..AC-005 | YES |
| BC-2.02.003 | Non-Exhaustive Enum Policy | S-011, S-014 | S-011: AC-001..AC-004; S-014: AC-003b (HookEvent #[non_exhaustive]) | YES |
| BC-2.02.004 | FactoryAdapter Trait Definition | S-012 | AC-001..AC-004 | YES |
| BC-2.02.005 | VsddFactoryAdapter Implementation | S-012 | AC-005..AC-013 | YES |
| BC-2.02.006 | HookEnvelope Proto Field Number | S-013 | AC-001, AC-006 | YES |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version | S-013 | AC-002..AC-003 | YES |
| BC-2.02.008 | Phase 4 schema_version Validation | S-013 | AC-004..AC-005 | YES |
| BC-2.03.001 | EngineModule Trait Definition | S-014, S-015 | S-014: AC-001..AC-007; S-015: AC-010 (PC-6 DI-006) | YES |
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
- 17 stories created: 15 product stories + 1 DTU (S-DTU-001) + 1 prep (S-PHASE-3-PREP)
- 22/22 BCs covered (100%)
- 22/22 VPs covered (100%)
- 15/15 error codes covered (100%)
- 12/12 P0 NFRs covered (NFR-001/002/003/006 deferred to Phase 3 per authoritative nfr-catalog.md ruling)
- 4-wave schedule (Wave 0 = Phase 3 prep gate, Wave 1 = foundation, Wave 2 = parallel impl, Wave 3 = dependents)
- S-PHASE-3-PREP created per TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE §Future Attachment obligation
- Dependency graph is acyclic (validated via topological sort; see dependency-graph.md)

**Phase 2 r01 remediation burst** (2026-05-19):
- F-PHASE2-R01-01..26 and GAP-PHASE2-R01-01..11 addressed
- S-009 moved from Wave 2 to Wave 3 (Decision 1: S-008→S-009 dependency added)
- S-001 BC-2.01.007/VP-007 mis-anchors removed; BC-2.01.007 sole implementer is S-008
- S-003 behavioral_contracts updated: [BC-2.01.002, BC-2.02.001] (GAP-5 resolved)
- S-006 blocks: [S-007, S-008] (S-009 removed — S-009 now depends on S-008 not S-006 directly)
- S-005 blocks: [] (S-007 removed — S-007 depends on S-006 not S-005)
- Wave 2 points: 41 (was 49); Wave 3 points: 34 (was 26)
- All 17 stories retrofitted with inputs:/input-hash:/traces_to: per SE-22 v2

## §Trace v1.1

**Phase 2 r01 remediation burst** (2026-05-19):
- See narrative in §Trace v1.0 above (inline in the initial burst trace).

## §Trace v1.2

**Phase 2 r02 remediation burst** (2026-05-19):
- F-PHASE2-R02-01 (CRITICAL): S-005 AC-004 exit codes rewritten to BC-2.01.004 PC-8 canonical 5-code taxonomy; fabricated codes 3/4 removed; AC-005 rewritten for INV-1 hard timeout; AC-006 renamed for INV-3 dual-accept
- F-PHASE2-R02-02 (CRITICAL): EPIC-01 stories table — S-005 Depends-On corrected to S-001,S-002; S-009 Wave corrected to Wave 3 with full depends-on list
- F-PHASE2-R02-03 (HIGH): S-008 Previous Story Intelligence corrected — S-008 is PRODUCER of RingBuffer API; S-009 is consumer; no stubbing allowed
- F-PHASE2-R02-04 (HIGH): S-015 AC-010 reanchored BC-2.03.001 invariant 2 → postcondition 5 (DI-006 enforce); dep-graph matrix row updated accordingly
- F-PHASE2-R02-05 (HIGH): S-012 AC-007 + AC-009 reanchored postcondition 3 → invariant 3; dep-graph row corrected; holdout HS-W3-005 corrected
- F-PHASE2-R02-06 (HIGH): S-005 AC-005 rewritten for INV-1 hard timeout (production-grade: preserve breadth not delete)
- F-PHASE2-R02-07 (HIGH): dep-graph BC Clause Coverage Matrix swept — BC-2.01.004 PC-4→PC-8 corrected; BC-2.01.005 postcondition rows reordered monotonically; BC-2.02.005 postcondition 3→invariant 3; BC-2.03.001 INV-2(DI-006)→PC-5(DI-006); GAP-P2-005 added for BC-2.01.004 PC-6 (--persistent-events Phase 3 scope)
- F-PHASE2-R02-08 (HIGH): S-006 Previous Story Intelligence rand version pin corrected — =0.8.6 EXACT; rand 0.9 REJECTED with rationale
- F-PHASE2-R02-11+17 (MEDIUM, Orchestrator Decision 3): monocle-auth crate dropped; generate_session_token() moved to monocle-runtime::auth; swept across S-001, S-006, S-009; S-001 workspace member list corrected to 3 crates; S-001 forbidden dependencies updated
- F-PHASE2-R02-12 (MEDIUM): S-015 File Structure test path monocle-runtime/tests/ → monocle-core/tests/
- F-PHASE2-R02-13 (MEDIUM): S-009 dtu_dependencies non-canonical field removed
- F-PHASE2-R02-15 (LOW): S-015 body BC table removed (standardize: no-body-table across corpus)
- GAP-PHASE2-R02-1 (HIGH): STORY-INDEX Blocks column — S-005 S-007→"—"; S-006 S-009 removed; sweep verified other entries consistent
- GAP-PHASE2-R02-2 (MEDIUM): wave-schedule.md Wave 3 paragraph updated "all 4" → "all 5" + S-008→S-009 within-wave dep note
- GAP-PHASE2-R02-3 (MEDIUM): S-009 File Structure generate_auth_token() → generate_session_token() clarification; conflation removed
- GAP-PHASE2-R02-4 (LOW): holdout-scenarios.md frontmatter level: ops + version: "1.1" added
- F-PHASE2-R02-09 (MEDIUM): sprint-state.yaml S-015 notes updated to include BC-2.03.001
- BC-2.03.001 BC Coverage Table updated: now S-014, S-015 (S-015 AC-010 covers PC-5 DI-006)
- Version-bump rule applied: stories with AC/depends_on/blocks/behavioral_contracts changes → minor bump (+0.1)

## §Trace v1.3

**Phase 2 r03/r04/r05 remediation bursts** (2026-05-19):
- F-PHASE2-R03-01 (CRITICAL): BC-2.01.002 BC Coverage Table corrected — BC-2.01.002 AC range updated to AC-001..AC-008 (AC-008 /status during drain added); S-009 BC-2.01.009 range confirmed AC-004..AC-010b.
- F-PHASE2-R04: BC Coverage Table S-009 and S-003 row updates consistent with dep-graph r04 anchor corrections.
- F-PHASE2-R05: AC-007b orphan introduced via r05 burst (BC-2.02.001 row S-003: AC-007b) — see v1.4 for resolution.
- BC-2.03.001 BC Coverage Table confirmed: S-014 AC-001..AC-007; S-015 AC-010 (PC-6 DI-006).
- Version-bump rule applied consistently.

## §Trace v1.4

**Phase 2 r06 remediation burst** (2026-05-19):
- F-PHASE2-R06-01 (CRITICAL): BC-2.01.009 PC-2/PC-3 alias/canonical mirror swap fixed — S-009 AC-005 trace header corrected to PC-3 (alias); AC-006 trace header corrected to PC-2 (canonical); S-003 AC-002 trace header corrected to PC-3 (alias).
- F-PHASE2-R06-02 (HIGH): BC-2.02.001 BC Coverage Table row corrected — S-003: AC-007b (orphan) → S-003: AC-005 (matches body consolidation note; AC-005 subsumes AC-007b intent per S-003 body line 90-93).
- F-PHASE2-R06-03 (MEDIUM): STORY-INDEX version bumped v1.3→v1.4; sprint-state.yaml and holdout-scenarios.md traces_to_full/traces_to updated to v1.4.
- F-PHASE2-R06-04 (MEDIUM): §Trace audit-trail completed — v1.1/v1.2/v1.3/v1.4 entries added for monotonically-ascending version coverage.
- SE-22 v2 cascade: BC-INDEX v1.12→v1.13 propagated to all 19 corpus consumers. BC-2.01.001..010 and BC-2.03.001..004 version pins propagated to all story frontmatter inputs entries.
- Discipline codified: story-corpus artifacts MUST have §Trace entries in monotonically-ascending version order for every declared version.
