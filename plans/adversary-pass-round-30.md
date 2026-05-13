---
document_type: adversarial-review-report
level: ops
version: "1.0"
status: complete
producer: adversary (fresh context, round 30, production-grade lens) — transcribed by state-manager during round-30 durability
phase: pre-phase-1-final-gate-round-30-complete
timestamp: 2026-05-14T01:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md  # v1.1.8
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md  # v1.2.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md  # v1.0.5
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md  # v1.1.7
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md  # v1.5
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md  # v1.4.15
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-29 fix burst commits 0b3f89d (adv-r28 persist) + dc719cd (engine v1.1.8) + 09642de (daemon-lifecycle v1.0.5) + 03f08ad (brief v1.4.14) + 1427f4d (brief v1.4.15) + dac0ebd (state); validates F-R28-1/2/3/4/5/6 resolution; surfaces 1 HIGH + 2 MEDIUM + 1 LOW continuing audit-table-completeness pattern"
project: monocle
verdict: NEEDS_ONE_MORE
---

# Adversarial Pass — Round 30

## Verdict

NEEDS_ONE_MORE — 0 CRITICAL + 1 HIGH + 2 MEDIUM + 1 LOW. Round-29 fix burst RESOLVED F-R28-1/2/3/4/5/6 correctly. The Cross-Crate Constructor Audit table introduced as F-R28-2's codification has ITS OWN completeness gap (audit-mechanism-incompleteness pattern recurrence) plus an enforcement gap (passive-document risk).

## Disposition of Round-28 Findings (all RESOLVED)

- F-R28-1 HIGH (EnrichedSession epoch sentinel): RESOLVED — Option<i64> in place; rustdoc updated; BC text updated.
- F-R28-2 HIGH (3 more constructors + HookEventRecord): RESOLVED — constructors for SpawnArgs/SessionHandle/EngineVersion; HookEventRecord defined as real struct.
- F-R28-3 MEDIUM (HookResponse pub-mutation): RESOLVED — `with_diagnostic`/`with_redirect` builder methods added; rustdoc shows chain form.
- F-R28-4 MEDIUM (HookEventRecord undefined): RESOLVED — real struct definition.
- F-R28-5 LOW (v1.1.5 trace): RESOLVED — supersession annotation corrected.
- F-R28-6 LOW (brief row order): RESOLVED — table monotonically ascending.

## Important Findings

### F-R30-1 HIGH — Cross-Crate Constructor Audit table is incomplete vs its own stated invariant

File: SS-engine-module.md lines 1079-1129 (§Cross-Crate Constructor Audit).

The table intro states "Every `#[non_exhaustive]` struct in the monocle workspace requires a `pub fn new(...)` ... [and] every architect spec change that adds `#[non_exhaustive]` to a struct MUST update this table." But the table enumerates only 7 structs. Missing:
- `FactoryDetection` (SS-core-types-and-abi.md line 343)
- `FactoryState` (line 370)
- `BlockingIssue` (line 412)
- `ConvergenceMetrics` (line 435)
- 5 HookEvent inner structs (in a separate sub-section rather than the main table)
- `HookEventRecord` (which separately lacks the `#[non_exhaustive]` attribute — see F-R30-2)

This is exactly the same audit-completeness pattern recurrence the round-28 adversary surfaced. The mechanism designed to prevent the bug from recurring itself fails its own completeness invariant.

Production-grade fix: Expand the audit table to cover ALL `#[non_exhaustive]` structs in the workspace specs. Either fold HookEvent inner structs into the same table (with a "construction path: serde-deserialize-only" notes column), OR split the table into two clearly labeled tables.

Routing: vsdd-factory:architect.

## Medium Findings

### F-R30-2 MEDIUM — HookEventRecord is NOT `#[non_exhaustive]` despite trace text citing the E0639 rationale

File: SS-daemon-lifecycle.md lines 340-394 (struct def) + line 540 (trace).

Line 340 declares `#[derive(Debug, Clone, Serialize, Deserialize)] pub struct HookEventRecord { ... }` — no `#[non_exhaustive]`. Trace at line 540 justifies the constructor with the E0639 rationale, which is inapplicable since the attribute is missing.

Either (a) `#[non_exhaustive]` is a Phase 2 forward-compat oversight — adding fields to an exhaustive `pub` struct is a SemVer-major breaking change for any downstream crate; or (b) intentional omission, in which case trace text is wrong.

Production-grade fix: Add `#[non_exhaustive]` to HookEventRecord. Add to Cross-Crate Constructor Audit table. The Phase 2 trigger-trace ingestion already documented at lines 306-313 implies field evolution is expected.

Routing: vsdd-factory:architect.

### F-R30-3 MEDIUM [process-gap] — Audit-table codification rule lacks CI/lint enforcement; passive document risk

File: SS-engine-module.md lines 1090-1093 (audit table invariant statement).

The policy statement "Every architect spec change that adds `#[non_exhaustive]` to a struct MUST update this table" is unenforced. F-R30-1 demonstrates this pattern can recur even with the table present — the audit mechanism is a passive document with no automatic verification. Same pattern class as F-R28-2.

Production-grade fix: Add semgrep rule to SS-conventions-anti-patterns.md (parallel to monocle-no-raw-env-mutation-in-tests in v1.5) matching `#[non_exhaustive]` on `pub struct` definitions and asserting (via fixture-corpus per POL-11) each matched struct is enumerated in the audit table. Or: add a Rust integration test that uses `syn` to parse source files and assert audit-table completeness.

Routing: vsdd-factory:architect (spec) + vsdd-factory:devops-engineer (CI wiring at implementation time).

## Low Findings

### F-R30-4 LOW — Brief revision history same-day entries lack time precision

File: product-brief.md lines 77-80. v1.4.12 through v1.4.15 all dated 2026-05-13 with no time component.

Production-grade fix: Switch Date column to ISO-8601 timestamps (`2026-05-13T22:00:00Z`) for any same-day successive revisions. Apply prospectively from v1.4.16+.

Routing: vsdd-factory:product-owner.

## Severity Trajectory

| Round | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| R20 | 0 | - | 2 | 1 |
| R22 | 0 | - | 3 | 0 |
| R24 | 0 | - | 3 | 2 |
| R26 | 1 | - | 2 | 3 |
| R28 | 0 | 2 | 2 | 2 |
| R30 | 0 | 1 | 2 | 1 |

Trajectory converging. Round-31 fix burst should resolve all four findings; round-32 validation likely converges to 0+0+0+0.

## Convergence Verdict

NEEDS_ONE_MORE.

## Phase 1 Implementation Readiness (Pass E)

`cargo build` and `cargo test` would BOTH succeed with the current spec set. Narrow caveat: a future synthesis-style test of FactoryState would hit E0639 since FactoryDetection/FactoryState/BlockingIssue/ConvergenceMetrics have no constructors. F-R30-1 closes this latent risk.

## Process-Gap Lessons (codification candidates, NOT tech-debt entries)

1. Audit-mechanism codification requires both the audit document AND its enforcement (semgrep rule + fixture corpus). Passive documents alone are insufficient — round-28 demonstrated this, round-30 confirmed the lesson with a recurrence.
2. The "X is what prevents bug class Y" claim must itself be subject to automated verification.
