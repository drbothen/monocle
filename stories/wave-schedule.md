---
document_type: plan-doc
level: L4
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:30:00Z
phase: 2
traces_to: dependency-graph.md
---

# Wave Schedule: monocle Phase 2 Implementation

## Wave Overview

| Wave | Stories | Total Points | Parallelism | Gate |
|------|---------|-------------|-------------|------|
| Wave 0 | S-PHASE-3-PREP | 3 | Sequential (external dep) | spec-kit-mcp rc.19+ available + human approval; does NOT block Waves 1–3 |
| Wave 1 | S-DTU-001, S-001 | 8 | Full parallel | Wave 1 gate: both S-DTU-001 and S-001 green; CI matrix passing |
| Wave 2 | S-002, S-003, S-004, S-005, S-006, S-009, S-010, S-011, S-013, S-014 | 49 | Partial parallel (internal order below) | Wave 2 gate: all 10 stories delivered and CI green |
| Wave 3 | S-007, S-008, S-012, S-015 | 26 | Full parallel | Wave 3 gate: all 4 stories delivered; 22/22 BCs green |

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
- S-009 (Auth Token — depends on S-001, S-004, S-006; starts after S-004 AND S-006)
- S-011 (Non-Exhaustive Enum — depends on S-010; starts after S-010)
- S-013 (HookEnvelope Proto — depends on S-010; starts after S-010)
- S-014 (EngineModule Trait — depends on S-010; starts after S-010)

| Story | Title | Points | Starts After | Status |
|-------|-------|--------|-------------|--------|
| S-002 | Healthz Endpoint | 3 | Wave 1 ✓ | draft |
| S-004 | Body Size Limit | 2 | Wave 1 ✓ | draft |
| S-006 | Lock File Atomic Lifecycle | 8 | Wave 1 ✓ | draft |
| S-010 | monocle-core Crate + ABI Version | 5 | Wave 1 ✓ | draft |
| S-003 | Status Endpoint | 5 | S-002 ✓ | draft |
| S-005 | Graceful Shutdown | 5 | S-002 ✓ | draft |
| S-009 | Auth Token + Header Validation | 8 | S-004 ✓, S-006 ✓ | draft |
| S-011 | Non-Exhaustive Enum Policy | 3 | S-010 ✓ | draft |
| S-013 | HookEnvelope Proto Wire Format | 5 | S-010 ✓ | draft |
| S-014 | EngineModule Trait Definition | 5 | S-010 ✓ | draft |

**Wave 2 gate criteria:**
- All 22 BC-2.01.001–BC-2.01.010 and BC-2.02.001–BC-2.02.003 covered by passing tests
- BC-2.03.001 (EngineModule trait) tests green
- VP-001 through VP-014 probes green
- NFR-004 (OsRng) + NFR-005 (body limit) + NFR-009 (0o600) + NFR-012 (0o700) + NFR-010 (constant-time) all verified

---

## Wave 3: Dependent Completions

**Timing:** After Wave 2 gate passes.
**Parallelism:** Full parallel (all 4 stories are independent of each other).

| Story | Title | Points | Depends On | Status |
|-------|-------|--------|-----------|--------|
| S-007 | Crash Recovery Checkpoint | 5 | S-006 (Wave 2) | draft |
| S-008 | JSONL Ring Format Version | 5 | S-006 (Wave 2) | draft |
| S-012 | FactoryAdapter Trait + VsddFactoryAdapter | 8 | S-010, S-011 (Wave 2) | draft |
| S-015 | ClaudeCodeModule Implementation | 8 | S-014 (Wave 2) | draft |

**Wave 3 gate criteria (= Phase 2 completion gate):**
- All 22 BCs covered by passing tests (22/22)
- All 22 VPs probed green (22/22)
- All 15 error codes exercised in tests (15/15)
- DTU fidelity ≥0.95 (S-DTU-001 gate)
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
