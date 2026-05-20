---
document_type: story
level: L4
story_id: S-PHASE-3-PREP
epic_id: EPIC-PREP
version: "1.1"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-20T00:00:00Z
phase: 2
points: 3
wave: 0
tdd_mode: facade
priority: P0
depends_on: []
blocks: []
# Parallel execution: depends_on: [] and blocks: [] — this Wave-0 story does NOT block
# Waves 1-3; they MAY proceed in parallel per CLAUDE.md §Current Pipeline State.
target_module: .factory/specs
subsystems: []
behavioral_contracts: []
verification_properties: []
external_dependency:
  repo: drbothen/vsdd-factory
  semver: ">=0.19.0-rc.0"
  release-channel: "<URL TBD — fill in when vsdd-factory upstream spec-kit-mcp rc.19+ ships>"
  install-command: "<TBD — fill in when vsdd-factory upstream spec-kit-mcp rc.19+ ships>"
  package-name: "spec-kit-mcp"
  canonical-docs: "<TBD — fill in when vsdd-factory upstream spec-kit-mcp rc.19+ ships>"
# BC status: pending PO authorship — this is a pre-implementation mechanical sweep story.
# BCs are authored when spec-kit-mcp ships and scope is concrete. Cannot be ready until
# spec-kit-mcp rc.19+ is available and BCs can be grounded in actual tool APIs.
# Wave 0 = pre-Phase-3 gate. Does NOT block any Phase 2 story.
# Sources: tech-debt-register.md TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE §Future Attachment
#          tech-debt-register.md TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT §Future Attachment
inputs:
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/tech-debt-register.md, version: "current"}
assumption_validations: []
risk_mitigations: []
input-hash: "[live-state]"
traces_to: "Pre-Phase-3 prep: spec-kit-mcp rc.19+ mechanical sweep. Anchored to TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE and TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT §Future Attachment."
---

# S-PHASE-3-PREP: spec-kit-mcp Integration — Phase 3 Pre-Implementation Mechanical Sweep

## Narrative

As the Phase 3 pre-implementation gate, I want to run the `spec-kit-mcp` tool suite against
the monocle `.factory/` artifact set, so that the PRD ↔ VP-INDEX reverse-cascade asymptote
(TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE) and the Phase 2 propagation-discipline asymptote
(TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT) are resolved by schema-enforced invariants
before Phase 3 TDD implementation begins — eliminating these classes of finding permanently
and allowing all 39 prose disciplines to be migrated to spec-kit-managed invariants.

## Background

This story fulfills two §Future Attachment obligations in `tech-debt-register.md`:

1. **TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE** (ACCEPTED, MEDIUM severity): PRD ↔ VP-INDEX
   bidirectional pin coherence asymptote. Prose-rule disciplines (SE-22 v1) cannot mechanically
   enforce reverse-cascade completeness at write time. INV-005 (transitive closure with fixed-point
   iteration — i.e., spec-kit enforces that every version-pin reference is fully up-to-date by
   traversing the full dependency graph iteratively until no stale pins remain) eliminates this
   defect class mechanically.

2. **TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT** (ACCEPTED, MEDIUM severity): Phase 2 story
   corpus exhibits asymptotic propagation-discipline residuals. The two ACTIVE Phase-2 residuals are:
   - **F-PHASE2-R13-01** (ACTIVE): STORY-INDEX BC-2.01.007 row over-includes AC-005 (S-008 AC-005
     cross-anchors BC-2.01.004 EC-049 per dep-graph line 250; summary table cannot unambiguously
     attribute cross-BC ACs).
   - **GAP-PHASE2-R13-1** (ACTIVE): STORY-INDEX BC-2.01.002 row missing S-009 attribution (S-009
     AC-010b cross-anchors BC-2.01.002 PC-1 sub-bullet hook_endpoints; same cross-BC attribution
     class as F-R13-01).
   These require a NEW invariant for cross-BC AC anchor attribution in STORY-INDEX BC Coverage Table
   rows — a different invariant than the AC-range column sibling-sweep (SE-26 candidate) because it
   concerns which story's ACs are attributed to which BC.

Both residuals share the same upstream dependency: vsdd-factory upstream `spec-kit-mcp` library
shipping at rc.19+.

**Contingency:** This story is BLOCKED until vsdd-factory upstream spec-kit-mcp rc.19+ ships.
When it ships, the human must explicitly approve dispatch of this story (see AC-004). It does NOT
block any Phase 2 story or Wave 1/2/3 implementation work.

### Inline Glosses (first occurrence)

- **SE-22 v1** (sibling-sweep META, first codification): disciplines requiring that when any
  artifact's version pin is bumped, all sibling artifacts citing that version must be swept in the
  same commit. Codified R17-pre per D-142.
- **SE-22 v2** (sibling-sweep Consumer-Ledger Extension): forward consumer-ledger cascades — when
  artifact A bumps, artifact B (A's consumer) must also bump. Codified R18E per D-149.
- **SE-23** (SM Defensive-Sweep Prohibition): state-manager agent must NOT perform defensive sweeps
  of spec artifacts; routing must go to the correct specialist agent. Codified R18-pre per D-146.
- **SE-25 candidate** (bidirectional DAG symmetry): every depends_on entry must have a matching
  blocks entry on the depended-on story. HELD pending spec-kit-mcp rc.19+.
- **SE-26 candidate** (STORY-INDEX BC Coverage Table AC-range sibling-sweep): AC-range column must
  be kept in sync with story ACs. HELD pending spec-kit-mcp rc.19+.
- **POL-29** (version-pin staleness policy): governs when a version pin is considered stale and
  requires update. Will live in `.factory/policies.yaml` (to be created during upstream
  spec-kit-mcp rc.19+ integration; that file does not currently exist).
- **INV-005** (transitive closure with fixed-point iteration): spec-kit invariant that enforces all
  version pins are up-to-date by traversing the full dependency graph until no stale pins remain.
- **NORMATIVE** (per SS-conventions-anti-patterns.md v1.29.5 §Pin-Symmetry policy): an annotation
  applied to changes that are binding constraints (as opposed to INFORMATIONAL which is context only).
  Stale pins on any artifact carrying a NORMATIVE annotation must be fixed before the spec is valid.

## Pre-Conditions

State of canonical artifact versions at story dispatch (from CLAUDE.md §Current Pipeline State
and STATE.md latest commit):

| Artifact | Version at Story Start |
|----------|----------------------|
| product-brief.md | v1.4.30 |
| prd.md | v1.26.15 |
| BC-INDEX.md | v1.13 |
| VP-INDEX.md | v1.16 |
| STORY-INDEX.md | v1.8 |
| dep-graph.md | v1.9 |
| wave-schedule.md | v1.4 |
| ARCH-INDEX.md | v1.0.11 |
| SS-daemon-lifecycle.md | v1.0.33 |
| SS-engine-module.md | v1.1.20 |
| SS-deps-pin-manifest.md | v1.1.18 |
| SS-conventions-anti-patterns.md | v1.29.5 |
| SS-core-types-and-abi.md | v1.2.13 |
| tech-debt-register.md | active residuals: F-PHASE2-R13-01 (ACTIVE), GAP-PHASE2-R13-1 (ACTIVE) |

## Post-Conditions

Expected state changes upon story completion:

| Artifact | Expected Change |
|----------|----------------|
| tech-debt-register.md TD-VSDD-PHASE-1 | F-R121/F-R122 class residual: CLOSED |
| tech-debt-register.md TD-VSDD-PHASE-2 | F-PHASE2-R13-01: CLOSED; GAP-PHASE2-R13-1: CLOSED |
| .factory/policies.yaml | New file created; POL-29 entry migrated from prose to schema invariant |
| SE-22 v1/v2, SE-25 candidate, SE-26 candidate | Migrated to spec-kit invariant schema entries; candidate SEAs promoted to codified if invariant covers them |
| All .factory/specs/*.md NORMATIVE pins | Zero stale NORMATIVE pins after cascade-tail sweep |
| 39 prose disciplines | Count stable or reduced (disciplines covered by INV-005 retired from prose catalog) |

## Acceptance Criteria

### AC-001 (resolves TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE — spec_kit_verify_invariants)
`spec_kit_verify_invariants(scope="all")` exits with code 0 AND stdout matches
`violations: 0` when run against the monocle `.factory/` artifact set after Phase 3 entry.
The oracle is mechanical: exit code 0 AND `violations: 0` in stdout — both conditions
must hold.

### AC-002 (resolves residual pin staleness — spec_kit_bump_artifact cascade)
`spec_kit_bump_artifact()` cascade-tail closes any remaining PRD ↔ VP-INDEX pin staleness
that persisted from the Phase 1 asymptote. Zero NORMATIVE stale pins remain after this sweep
(NORMATIVE = stale pins on any artifact carrying a NORMATIVE annotation per
SS-conventions-anti-patterns.md v1.29.5 §Pin-Symmetry policy). `spec_kit_verify_invariants`
re-run is required until fixed-point: re-run until `violations: 0` holds across all artifacts.

### AC-003 (prose rule migration — POL-29 / SE-22 to schema invariants)
POL-29 (version-pin staleness policy — will live in `.factory/policies.yaml` to be created
during this story; the file does not currently exist) and SE-22 v1/v2 prose rules are
migrated to schema-enforced invariants in the spec-kit schema file.

Oracle: verified by attempting to write a stale-version-pin into any consumer artifact;
the pre-commit hook (`vsdd-spec-kit-validator.wasm`) MUST reject the commit with non-zero
exit and a structured error message.

The adversarial review loop no longer needs to catch version-pin staleness manually —
the spec-kit pre-commit hook blocks it at write time.

### AC-004 (human approval of spec-kit-mcp rc.19+ availability — dispatch gate)
Before dispatch of this story, the human confirms: "vsdd-factory upstream spec-kit-mcp rc.19+
is available on the release channel." This confirmation is the dispatch gate. The install
command, package name, release-channel URL, and canonical docs URL are TBD placeholders
in `external_dependency` frontmatter — fill in when rc.19+ ships.

### AC-005 (resolves F-PHASE2-R13-01 — new invariant for cross-BC AC anchor attribution)
A NEW invariant is defined in the spec-kit schema that mechanically enforces cross-BC AC
anchor attribution in STORY-INDEX BC Coverage Table rows. Specifically: an AC that
cross-anchors a secondary BC must be attributed ONLY to the primary BC in the Coverage Table
(not the secondary). The invariant blocks AC-over-attribution at write time.

NOTE: This AC requires a NEW invariant to be defined upstream in spec-kit-mcp rc.19+ — the
new invariant cannot be authored here because the invariant schema is not yet available.
This story remains BLOCKED until spec-kit-mcp rc.19+ ships with support for cross-BC
attribution invariants.

### AC-006 (resolves GAP-PHASE2-R13-1 — STORY-INDEX BC-2.01.002 attribution closure)
After the new cross-BC AC anchor attribution invariant (AC-005) is in place:
`spec_kit_verify_invariants(scope="story-index")` must flag as a violation the
BC-2.01.002 row missing S-009 attribution (S-009 AC-010b cross-anchors BC-2.01.002
PC-1 sub-bullet hook_endpoints). The Story Writer then adds S-009 to the BC-2.01.002
row per the invariant's resolution guidance.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,100 |
| tech-debt-register.md (TD-VSDD-PHASE-1 + TD-VSDD-PHASE-2 entries) | ~1,500 |
| spec-kit-mcp tool documentation (when available) | ~TBD |
| monocle .factory/ artifact inventory | ~500 |
| SS-conventions-anti-patterns.md v1.29.5 (Pin-Symmetry section) | ~300 |
| **Total estimate** | **~3,400 + spec-kit docs** |

Well within 20% of 200k context window. No split required.

## Tasks

- [ ] GATE: Confirm vsdd-factory spec-kit-mcp rc.19+ is shipped and available; record install
  command + release-channel URL in `external_dependency` frontmatter
- [ ] Install spec-kit-mcp in monocle factory environment per `external_dependency` install-command
- [ ] Run `spec_kit_verify_invariants(scope="all")` against monocle `.factory/`;
  confirm exit code 0 AND `violations: 0` in stdout (mechanical oracle per AC-001)
- [ ] Run `spec_kit_bump_artifact()` cascade-tail for any remaining stale NORMATIVE pins;
  re-run `spec_kit_verify_invariants` until fixed-point (`violations: 0` holds)
- [ ] Create `.factory/policies.yaml` with POL-29 entry migrated to spec-kit invariant schema
- [ ] Migrate SE-22 v1/v2 prose rules to spec-kit invariant schema entries
- [ ] Verify pre-commit hook (`vsdd-spec-kit-validator.wasm`) blocks stale-version-pin writes:
  attempt to write a stale pin → hook must reject with non-zero exit + structured error message
- [ ] Define new cross-BC AC anchor attribution invariant (per AC-005); confirm spec-kit-mcp rc.19+
  supports this invariant class before authoring
- [ ] Apply STORY-INDEX BC-2.01.002 fix: add S-009 attribution per AC-006 resolution guidance
- [ ] Apply STORY-INDEX BC-2.01.007 fix: remove AC-005 over-attribution per AC-005 invariant
- [ ] Run `spec_kit_verify_invariants(scope="all")` final pass — MUST return `violations: 0`
- [ ] Invoke `/vsdd-factory:adversarial-review` skill against changed artifacts to confirm
  no new version-pin findings
- [ ] Add `spec_kit_verify_invariants` to CI on every PR (analogous to `cargo audit`);
  verify workflow file exists at `.github/workflows/spec-kit-verify.yml`
- [ ] Specify spec-kit-mcp output persistence path under `.factory/spec-kit/`:
  - Verification log: `.factory/spec-kit/verify-invariants.log`
  - Cascade-tail diff artifact: `.factory/spec-kit/cascade-tail-diff.md`

## Previous Story Intelligence

N/A — Wave 0 story; precedes all Phase 3 implementation.
Dependency: vsdd-factory upstream (external dependency not under monocle's control).

Note: `depends_on: []` and `blocks: []` — this Wave-0 story runs in parallel with all Waves.
Waves 1/2/3 MAY proceed in parallel; this story does NOT gate them.

## Architecture Compliance Rules

From `tech-debt-register.md` TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE §Required for Resolution:
- Target library: `vsdd-spec-kit-core` (typed graph + invariants)
- Target MCP server: `spec-kit-mcp` (agent-callable mutation tools)
- Target invariant: INV-005 (transitive closure with fixed-point iteration)
- Dispatcher pre-commit hook: `vsdd-spec-kit-validator.wasm`

From `tech-debt-register.md` TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT §Required for Resolution:
- Requires NEW invariant for cross-BC AC anchor attribution in STORY-INDEX BC Coverage Table
  (distinct from SE-26 AC-range column sibling-sweep invariant)
- ACTIVE residuals F-PHASE2-R13-01 + GAP-PHASE2-R13-1 closed by AC-005 + AC-006

**Forbidden Dependencies:**
- This story MUST NOT modify any `.factory/specs/behavioral-contracts/` BC files
- This story MUST NOT add new BCs or modify existing BC IDs

## Library & Framework Requirements

| Tool | Version | Usage |
|------|---------|-------|
| spec-kit-mcp | `>=0.19.0-rc.0` (external, vsdd-factory upstream; `drbothen/vsdd-factory`) | Invariant verification and cascade-tail bump; install command TBD when rc.19+ ships |

## File Structure Requirements

Files to modify:
- `.factory/specs/*.md` — NORMATIVE pin corrections as needed by cascade-tail sweep
- `.factory/stories/STORY-INDEX.md` — BC-2.01.007 row: remove AC-005 over-attribution; BC-2.01.002 row: add S-009 attribution (per AC-005 + AC-006)

Files to create:
- `.factory/policies.yaml` — new file; POL-29 migrated to spec-kit invariant schema
- `.factory/spec-kit/verify-invariants.log` — spec-kit-mcp verification output log
- `.factory/spec-kit/cascade-tail-diff.md` — cascade-tail sweep diff artifact
- `.github/workflows/spec-kit-verify.yml` — CI gate: `spec_kit_verify_invariants` on every PR

Files to install (tool, not a repo file):
- spec-kit pre-commit hook: `vsdd-spec-kit-validator.wasm` (installed by spec-kit-mcp tooling)

## §Trace

**v1.1** (2026-05-20) — Phase 3.B Batch 1 spec-reviewer remediation (F-A-01..F-E-03 findings from cycle-001 Stage-1 review). Refs: drbothen/vsdd-factory#150.
- F-B-01 CLOSED: TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT added to inputs frontmatter and Background section. F-PHASE2-R13-01 and GAP-PHASE2-R13-1 named explicitly in Background.
- F-C-04 (MED) / F-B-01 CLOSED: AC-005 and AC-006 added covering closure of F-PHASE2-R13-01 + GAP-PHASE2-R13-1 via new cross-BC AC anchor attribution invariant. Story remains BLOCKED on upstream rc.19+ per original contingency.
- F-C-01 + F-C-02 CLOSED: AC-001 oracle updated to mechanical criterion (exit code 0 AND `violations: 0` in stdout). AC-002 NORMATIVE defined inline with reference to SS-conventions-anti-patterns.md v1.29.5 §Pin-Symmetry; fixed-point re-run requirement stated explicitly.
- F-C-03 CLOSED: AC-003 oracle added (attempt stale-pin write; hook must reject with non-zero exit + structured error message).
- F-B-02 CLOSED: Inline glosses added for SE-22 v1/v2, SE-23, SE-25 candidate, SE-26 candidate, POL-29, INV-005, NORMATIVE.
- F-B-03 CLOSED: POL-29 file home resolved: `.factory/policies.yaml` to be created during this story (current non-existence noted explicitly).
- F-A-01 CLOSED: spec-kit-mcp install/version updated to structured external_dependency frontmatter with TBD placeholders clearly marked (repo, semver, release-channel, install-command, package-name, canonical-docs).
- F-D-01 + F-D-02 + F-D-03 CLOSED: Files-to-modify concretized to specific paths; spec-kit output persistence paths specified (`.factory/spec-kit/`); `spec_kit_verify_invariants` CI task added.
- F-E-01 + F-E-02 CLOSED: Pre-Conditions section added (canonical artifact versions at story start). Post-Conditions section added (expected version bumps + ACTIVE→CLOSED catalog transitions).
- F-E-03 CLOSED: Parallel-execution note added to frontmatter `blocks: []` comment and Previous Story Intelligence.
- F-C-05 CLOSED: "Run adversarial review pass" task updated to reference `/vsdd-factory:adversarial-review` skill explicitly.
- external_dependency frontmatter field structured (replaces freeform `vsdd-factory-spec-kit-mcp-rc19plus` string).
- inputs: tech-debt-register.md added.
