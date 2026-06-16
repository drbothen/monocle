---
document_type: plan-doc
level: L4
version: "1.7"
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
| Wave 8 | S-032, S-DAEMON-WIRE-FIX-001, S-033..S-038, S-045..S-048 | 74 | S-033 is root; tiered within-wave serial ordering (see §Wave 8) | Wave 8 gate: BC-2.08.001..008 + BC-2.05.009..011 + BC-2.06.025 + BC-2.03.005..008 green; adversarial 3 clean passes |
| Wave 9 | S-039..S-044 | 42 | S-039 is root; S-040/S-042/S-043 parallel after S-039; S-041 after S-040; S-044 after S-040+S-041 | Wave 9 gate: BC-2.09.001..009 green; adversarial 3 clean passes |

**Total implementation waves: 3 (+ Wave 0 pre-Phase-3) for Phase-3 original scope. v1A adds Waves 8–9.**
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
- All EXACT-pin crates match SS-deps-pin-manifest.md v1.1.17 <!-- version-pin-historical: Wave 1 gate criterion at Wave 1 pass time -->

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
- S-005 (Graceful Shutdown — depends on S-001, S-002, S-003, S-006; starts after S-003 and S-006;
  critical-path bottleneck is S-003; S-006 depends only on S-001 so completes before S-005 can start)
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
| S-005 | Graceful Shutdown | 5 | S-003, S-006 ✓ | draft |
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
| S-009 | Auth Token Wire Format + Header Validation | 8 | S-001, S-004, S-006, S-008, S-DTU-001 (Decision 1 + Decision 10) | draft |
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

---

## Wave 8: Session Manager + IPC/TUI Delta (v1A)

**Timing:** After Wave 7 gate passes (Phase 3 v1A extension scope).
**Parallelism:** S-033 is the sequential root (all other Wave-8 EPIC-08 stories wait for S-033). S-032 and S-DAEMON-WIRE-FIX-001 can run in parallel with S-033. S-045 waits for S-033. S-046 waits for S-032. S-047 waits for S-033, S-034, S-035, and S-046. S-048 waits for S-033 and S-047.

### Wave 8 Internal Ordering

**Tier 1 (no Wave-8 predecessors — start immediately at Wave 8):**
- S-032: depends on S-021(W5), S-018(W5) — parallel with S-033
- S-DAEMON-WIRE-FIX-001: depends on S-017(W5), S-016(W4) — parallel with S-033
- S-033: depends on S-014(W2), S-015(W3), S-017(W5), S-021(W5) — Wave-8 EPIC-08 root

**Tier 2 (after S-033):**
- S-034: depends on S-033(W8)
- S-035: depends on S-033(W8)
- S-037: depends on S-033(W8) — can start in parallel with S-034/S-035
- S-038: depends on S-033(W8) — can start in parallel with S-034/S-035/S-037
- S-045: depends on S-015(W3), S-033(W8) — can start in parallel with S-034/S-035/S-037/S-038

**Tier 3 (after S-032 done):**
- S-046: depends on S-021(W5), S-032(W8)

**Tier 4 (after S-033 + S-034 + S-035 done):**
- S-036: depends on S-033(W8), S-034(W8), S-035(W8)

**Tier 5 (after S-033 + S-034 + S-035 + S-046 done):**
- S-047: depends on S-021(W5), S-022(W6), S-023(W6), S-033(W8), S-034(W8), S-035(W8), S-046(W8)

**Tier 6 (after S-033 + S-047 done):**
- S-048: depends on S-022(W6), S-025(W6), S-028(W7), S-033(W8), S-047(W8)

| Story | Title | Points | Depends On (Wave 8 deps only) | Status |
|-------|-------|--------|-------------------------------|--------|
| S-032 | Daemon Event-Bus Fan-Out | 5 | — (Wave 8 root) | draft |
| S-DAEMON-WIRE-FIX-001 | Second-Signal Exit Codes | 5 | — (Wave 8 root) | draft |
| S-033 | SessionManager::spawn_session | 8 | — (Wave 8 root) | draft |
| S-034 | SessionManager::kill_session | 8 | S-033 | draft |
| S-035 | SessionManager::attach_session and detach_session | 8 | S-033 | draft |
| S-037 | SessionManager GC Task | 3 | S-033, S-034 | draft |
| S-038 | SessionManager Hook Auto-Injection | 3 | S-033 | draft |
| S-045 | ClaudeCodeModule::spawn_recipe() | 5 | S-033 | draft |
| S-046 | PtyOutput Fan-out Broker | 5 | S-032 | draft |
| S-036 | SessionManager::rediscover_sessions | 8 | S-033, S-034, S-035 | draft |
| S-047 | IPC Lifecycle Variants | 8 | S-033, S-034, S-035, S-046 | draft |
| S-048 | Sessions Panel — Multi-Project | 8 | S-033, S-047 | draft |

**Wave 8 gate criteria:**
- BC-2.03.005..008, BC-2.05.009..011, BC-2.06.025, BC-2.08.001..008 passing tests
- BC-2.05.004 (S-032) + BC-2.01.004 PC-8/INV-4 (S-DAEMON-WIRE-FIX-001) fully discharged
- `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean
- Adversarial review on Wave 8 diff (3 consecutive clean passes required)

---

## Wave 9: Embedded PTY (v1A)

**Timing:** After Wave 8 gate passes.
**Parallelism:** S-039 is the sequential root. After S-039: S-040, S-042, S-043 can run in parallel. S-041 waits for S-040. S-044 waits for S-040 AND S-041.

### Wave 9 Internal Ordering

**Tier 1 (Wave 9 root):**
- S-039: depends on S-021(W5), S-025(W6), S-035(W8)

**Tier 2 (after S-039):**
- S-040: depends on S-039(W9)
- S-042: depends on S-039(W9) — fully parallel with S-040/S-043
- S-043: depends on S-039(W9) — fully parallel with S-040/S-042

**Tier 3 (after S-040):**
- S-041: depends on S-040(W9)

**Tier 4 (after S-040 + S-041):**
- S-044: depends on S-033(W8), S-035(W8), S-040(W9), S-041(W9)

| Story | Title | Points | Depends On (Wave 9 deps only) | Status |
|-------|-------|--------|-------------------------------|--------|
| S-039 | PTY Output Pipeline | 8 | — (Wave 9 root) | draft |
| S-040 | Full-Fidelity Keyboard Forwarding | 8 | S-039 | draft |
| S-042 | PTY Resize Detection + Debounce | 5 | S-039 | draft |
| S-043 | Scrollback Navigation | 3 | S-039 | draft |
| S-041 | Mouse Forwarding | 5 | S-040 | draft |
| S-044 | EmbeddedTerminal + SessionCreation AppMode Transitions | 13 | S-040, S-041 | draft |

**Wave 9 gate criteria:**
- BC-2.09.001..009 all passing tests
- `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean
- Scoped mouse capture enabled on EmbeddedTerminal ENTRY; disabled on EXIT (BC-2.09.003 INV-1)
- Adversarial review on Wave 9 diff (3 consecutive clean passes required)

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

## §Trace v1.7

**Phase-2 Burst F: Wave 8 (expanded) + Wave 9 added** (2026-06-15):
- Wave 8 expanded from 2 stories (S-032 + S-DAEMON-WIRE-FIX-001, 10 pts) to 12 stories (74 pts total) by adding 10 new v1A stories: S-033..S-038 (EPIC-08 Session Manager), S-045 (EPIC-03 delta), S-046..S-047 (EPIC-05 delta), S-048 (EPIC-06 delta).
- Wave 9 added: 6 stories (42 pts) — S-039..S-044 (EPIC-09 Embedded PTY).
- Within-wave serial ordering documented for both waves: S-033 is Wave-8 root (EPIC-08); S-039 is Wave-9 root (EPIC-09).
- Dependency graph confirmed acyclic. No cycles introduced. Wave assignment convention (within-wave serial deps allowed) is consistent with prior project history (Wave 5: S-017→S-018; Wave 7: S-027→S-029).
- Wave overview table updated in §Wave Overview (see STORY-INDEX.md §Wave Summary for canonical wave table).

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

## §Trace v1.3

**Phase 2 r07 remediation** (2026-05-19):
- F-PHASE2-R07-05 / Orchestrator Decision 9 (LOW): `inputs:` entry added for `error-taxonomy.md v1.5` to sibling-mirror the STORY-INDEX and dependency-graph inputs block; wave gate criteria reference E-AUTH-001/002/003 error codes which are defined in error-taxonomy.md.
- F-PHASE2-R07-07 (LOW): Wave 3 parallelism prose rewritten — "All 5 stories" ambiguity resolved; now correctly states "4 parallel + S-009 serial after S-008 (Decision 1)" in both the Wave Overview table and the Wave 3 body paragraph. Previous prose implied all 5 stories were fully parallel, contradicting Decision 1 (S-008→S-009 RingBuffer dependency added in r02).

## §Trace v1.5

**Phase 3.B Batch 6 — S-005 depends_on S-006 cascade** (2026-05-20):
- S-005 Wave 2 Internal Ordering entry updated: "depends on S-001, S-002" → "depends on
  S-001, S-002, S-003, S-006; starts after S-003 and S-006". Critical-path analysis notes
  S-006 (Wave-1-only dep) completes before S-003 (Wave-2 dep on S-002), so S-003 remains
  the bottleneck — no wave reassignment required.
- Wave 2 table S-005 "Starts After" column updated: S-002 → S-003, S-006.
- wave-schedule version bumped v1.4→v1.5.

## §Trace v1.4

**Phase 2 r09 remediation burst** (2026-05-19):
- F-PHASE2-R09-01 (HIGH): Wave 3 table S-009 Depends-On column updated to include S-DTU-001 (Decision 10: symmetric bidirectional edge required; S-DTU-001 blocks S-009 per S-DTU-001 frontmatter; S-009 now lists S-DTU-001 in depends_on).
- wave-schedule version bumped v1.3→v1.4.
