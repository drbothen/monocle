---
document_type: plan-doc
level: L4
version: "1.2"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:30:00Z
phase: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "dependency-graph.md; implements wave execution schedule derived from topological sort"
---

# Wave Schedule: monocle Phase 2 Implementation

## Wave Overview

| Wave | Stories | Total Points | Parallelism | Gate |
|------|---------|-------------|-------------|------|
| Wave 0 | S-PHASE-3-PREP | 3 | Sequential (external dep) | spec-kit-mcp rc.19+ available + human approval; does NOT block Waves 1–3 |
| Wave 1 | S-DTU-001, S-001 | 8 | Full parallel | Wave 1 gate: both S-DTU-001 and S-001 green; CI matrix passing |
| Wave 2 | S-002, S-003, S-004, S-005, S-006, S-010, S-011, S-013, S-014 | 41 | Partial parallel (internal order below) | Wave 2 gate: all 9 stories delivered and CI green |
| Wave 3 | S-007, S-008, S-009, S-012, S-015 | 34 | 4 parallel + S-009 serial after S-008 (Decision 1) | Wave 3 gate: all 5 stories delivered; 22/22 BCs green |

**Total implementation waves: 3 (+ Wave 0 pre-Phase-3)**
**Phase 3 dispatch gate: Wave 3 complete + Wave 0 complete (S-PHASE-3-PREP)**

---

## Wave 0: Phase 3 Pre-Implementation Prep

**Timing:** Before Phase 3 dispatch. Does NOT block Phase 2 Waves 1–3.

| Story | Title | Points | Status | External Dep |
|-------|-------|--------|--------|-------------|
| S-PHASE-3-PREP | spec-kit-mcp Integration Sweep | 3 | draft (blocked) | vsdd-factory spec-kit-mcp rc.19+ |

**Gate criteria:**
- vsdd-factory upstream spec-kit-mcp rc.19+ ships
- Human approves dispatch
- `spec_kit_verify_invariants(scope="all")` → 0 violations
- POL-29 and SE-22 v1/v2 migrated to schema invariants

---

## Wave 1: Foundation

**Timing:** Phase 3 implementation start.
**Parallelism:** Full parallel (both stories have no dependencies on each other).

| Story | Title | Points | Implementer Role | Status |
|-------|-------|--------|-----------------|--------|
| S-DTU-001 | Claude Code Hook Protocol DTU Clone | 3 | devops-engineer / implementer | draft |
| S-001 | Cargo Workspace Init + CI/DevOps Setup | 5 | devops-engineer | draft |

**Wave 1 gate criteria:**
- `cargo build --workspace` green on macOS + Linux (darwin/linux × amd64/arm64)
- CI matrix `.github/workflows/ci.yml` passing
- DTU clone `dtu-claude-code-hooks-v1` Docker container builds and starts
- DTU clone responds to all 5 hook POST endpoints
- All EXACT-pin crates match SS-deps-pin-manifest.md v1.1.17

---

## Wave 2: Core Implementation

**Timing:** After Wave 1 gate passes.
**Parallelism:** Partial — some Wave 2 stories can parallelize; others have internal ordering.

### Wave 2 Internal Ordering

Stories that can start immediately at Wave 2 start (depend only on S-001):
- S-002 (Healthz — depends on S-001 only)
- S-004 (Body Size Limit — depends on S-001 only)
- S-006 (Lock File Lifecycle — depends on S-001 only)
- S-010 (monocle-core ABI — depends on S-001 only)

Stories that depend on Wave 2 predecessors (start when predecessor is green):
- S-003 (Status — depends on S-001, S-002; starts after S-002)
- S-005 (Graceful Shutdown — depends on S-001, S-002; starts after S-002)
- S-011 (Non-Exhaustive Enum — depends on S-010; starts after S-010)
- S-013 (HookEnvelope Proto — depends on S-010; starts after S-010)
- S-014 (EngineModule Trait — depends on S-010; starts after S-010)

Note: S-009 moved to Wave 3 (Decision 1 — S-008→S-009 dependency added; S-009 tasks consume RingBuffer from S-008).

| Story | Title | Points | Starts After | Status |
|-------|-------|--------|-------------|--------|
| S-002 | Healthz Endpoint | 3 | Wave 1 ✓ | draft |
| S-004 | Body Size Limit | 2 | Wave 1 ✓ | draft |
| S-006 | Lock File Atomic Lifecycle | 8 | Wave 1 ✓ | draft |
| S-010 | monocle-core Crate + ABI Version | 5 | Wave 1 ✓ | draft |
| S-003 | Status Endpoint | 5 | S-002 ✓ | draft |
| S-005 | Graceful Shutdown | 5 | S-002 ✓ | draft |
| S-011 | Non-Exhaustive Enum Policy | 3 | S-010 ✓ | draft |
| S-013 | HookEnvelope Proto Wire Format | 5 | S-010 ✓ | draft |
| S-014 | EngineModule Trait Definition | 5 | S-010 ✓ | draft |

**Wave 2 gate criteria (BCs validated by Wave 2 stories per traces_to):**
- BC-2.01.001 (Healthz) — S-002 tests green
- BC-2.01.002 (Status) — S-003 tests green
- BC-2.01.003 (Body Limit) — S-004 tests green
- BC-2.01.004 (Graceful Shutdown) — S-005 tests green
- BC-2.01.005 (Lock File Lifecycle) — S-006 tests green
- BC-2.01.010 (Lock File Contract Version) — S-006 tests green
- BC-2.02.001 (ABI Version in /status) — S-003 + S-010 tests green
- BC-2.02.002 (ABI Version Constant) — S-010 tests green
- BC-2.02.003 (Non-Exhaustive Enum) — S-011 tests green
- BC-2.02.006, BC-2.02.007, BC-2.02.008 (HookEnvelope) — S-013 tests green
- BC-2.03.001 (EngineModule Trait) — S-014 tests green
- VP-001 through VP-013, VP-016..VP-019 probes green
- NFR-005 (body limit) + NFR-007 (CI) + NFR-008 (matrix) + NFR-009 (0o600) + NFR-012 (0o700) all verified

---

## Wave 3: Dependent Completions

**Timing:** After Wave 2 gate passes.
**Parallelism:** 4 stories parallel (S-007, S-008, S-012, S-015 are fully independent and can run concurrently). S-009 runs serially after S-008 completes within Wave 3 per Decision 1 (S-008 → S-009 RingBuffer dependency).

| Story | Title | Points | Depends On | Status |
|-------|-------|--------|-----------|--------|
| S-007 | Crash Recovery Checkpoint | 5 | S-006 (Wave 2) | draft |
| S-008 | JSONL Ring Format Version | 5 | S-006 (Wave 2) | draft |
| S-009 | Auth Token Wire Format + Header Validation | 8 | S-001, S-004, S-006, S-008 (Decision 1) | draft |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | 8 | S-010, S-011 (Wave 2) | draft |
| S-015 | ClaudeCodeModule Implementation | 8 | S-014 (Wave 2) | draft |

**Wave 3 gate criteria (= Phase 2 completion gate):**
- All 22 BCs covered by passing tests (22/22) — S-009 completing BC-2.01.008 + BC-2.01.009
- All 22 VPs probed green (22/22) — VP-008, VP-009 (auth) also validated in Wave 3
- All 15 error codes exercised in tests (15/15) — E-AUTH-001/002/003 validated by S-009
- DTU fidelity ≥0.95 (S-DTU-001 gate)
- NFR-004 (OsRng) + NFR-010 (constant-time) validated by S-009 in Wave 3
- NFR-011 (DTU fidelity) validated by dtu-validator
- `cargo clippy --workspace -- -D warnings` → 0 warnings
- `cargo audit` → 0 critical/high advisories
- Adversarial review pass on full implementation (Phase 5 entry)

---

## Holdout Scenarios Reference

Holdout scenarios are in `holdout-scenarios.md` (hidden from implementers per Phase 4 protocol).

---

## Phase 3 Dispatch Checklist

Before dispatching Phase 3 (TDD Implementation):
- [ ] Wave 1 gate PASS
- [ ] Wave 2 gate PASS
- [ ] Wave 3 gate PASS
- [ ] S-PHASE-3-PREP dispatched and complete (Wave 0)
- [ ] Adversarial review of Phase 2 story corpus complete (consistency-validator + adversary passes)
- [ ] Human approval: "Phase 2 story corpus approved; proceed to Phase 3 TDD"

---

## §Trace v1.0

**Phase 2 story decomposition wave schedule** (2026-05-19T04:30:00Z):
- 4 waves defined: Wave 0 (prep), Wave 1 (foundation), Wave 2 (core), Wave 3 (dependent completions)
- Wave 0 does NOT block Waves 1–3; it is the Phase 3 dispatch pre-condition
- DTU story correctly placed in Wave 1 (no product deps; product stories depend on it)
- S-PHASE-3-PREP correctly placed in Wave 0 (pre-Phase-3, not blocking Phase 2)
- Topological sort verified in dependency-graph.md (ACYCLIC PASS)

## §Trace v1.1

**Phase 2 r02 remediation** (2026-05-19):
- GAP-PHASE2-R02-2: Wave 3 parallelism paragraph corrected from "all 4 stories" to "all 5 stories"; S-008→S-009 within-wave dependency noted

## §Trace v1.2

**Phase 2 r06 remediation** (2026-05-19):
- F-PHASE2-R06-03 (MEDIUM): BC-INDEX version pin bumped v1.12→v1.13 per SE-22 v2 forward consumer-ledger sweep (15 BCs bumped by PO commit d7c860a).
- F-PHASE2-R06-04 (MEDIUM): §Trace v1.2 entry added; discipline: story-corpus artifacts MUST have §Trace entries in monotonically-ascending version order for every declared version.
