---
document_type: adversarial-review-report
level: ops
version: "1.0"
status: complete
producer: adversary (fresh context, round 40, production-grade lens) — transcribed by state-manager during round-40 durability
phase: pre-phase-1-final-gate-round-40-complete
timestamp: 2026-05-13T23:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md  # v1.1.10
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md  # v1.2.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md  # v1.0.6
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md  # v1.1.7
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md  # v1.10
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-forward-compatibility.md  # v1.2.2
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md  # v1.4.18
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-39 fix burst commits 58d1320 + 7f0da23 + 8db4676 + 4a16cb9 + state-manager rollback 1d30329; resumed post-rollback convergence iteration; 0/3 clean adversary passes; surfaces 2 MEDIUM (5th-recurrence patterns at CLI-invocation layer + sibling-file dimension)"
project: monocle
verdict: NEEDS_ONE_MORE
---

# Adversarial Pass — Round 40

## Verdict
NEEDS_ONE_MORE — 0 CRITICAL + 0 HIGH + 2 MEDIUM + 0 LOW + 1 OBSERVATION. This is NOT a clean pass. Convergence count remains 0/3 consecutive clean passes since cycle start. Two genuinely novel findings at layers prior 8 rounds didn't touch: CLI-invocation paths-scope gap + sibling-file dimension of cross-artifact citation staleness.

## Disposition of Round-38 Findings

- F-R38-1 (Option B exception scope): HOLDS. Narrowly scoped to code-specification regex constants; v1.8 §Trace constant assignments fall within exception; v1.10 §Trace uses abbreviated form (compliant).
- F-R38-2 (4th-recurrence stale citation): HOLDS WITHIN FILE; FAILS AT SIBLING-FILE LAYER. Within SS-forward-compatibility v1.2.2 the three fixes are correct, but the "full sweep" was within-file only — sibling SS-engine-module.md retains stale citations (now F-R40-2).

## Important Findings

### F-R40-1 MEDIUM — Semgrep CLI --include glob doesn't match binary crate (audit-coverage-gap META-pattern at CLI layer)

File: SS-conventions-anti-patterns.md line 322 vs rule paths at lines 174-189.

The semgrep rule's `paths.include` (lines 174-189) covers 11 hyphenated workspace crates + binary crate `monocle/src/**/*.rs` (12 total). But the Step-3 Python script invocation at line 322 uses:
```
semgrep --config .semgrep.yml --json --include "monocle-*/src/**/*.rs"
```
The CLI `--include "monocle-*/src/**/*.rs"` is glob-restrictive against hyphenated names. The binary crate `monocle/` (no hyphen) does NOT match. CLI `--include` overrides rule-level `paths.include` (semgrep CLI filter applied at runner level, not union).

Consequence: any `#[non_exhaustive] pub struct` added to the binary crate `monocle/src/**` is silently dropped from the Step-3 audit-completeness check. F-R34-3 META-GAP (4 of 11 paths covered) recurring at CLI-invocation layer with different blast radius (1 crate, the binary).

POL-11 positive-coverage doesn't catch this because the fixture-corpus step exercises the rule against `semgrep-fixtures/`, not against production scope — coverage gap in production-scope glob remains undetectable.

Fix: line 322 must be one of:
- Remove `--include` flag entirely (let rule's paths.include govern scope)
- Replace with `--include "monocle*/src/**/*.rs"` (no hyphen; matches both)
- List both: `--include "monocle-*/src/**/*.rs" --include "monocle/src/**/*.rs"`

Routing: architect.

### F-R40-2 MEDIUM — SS-engine-module §Trace cites stale SS-daemon-lifecycle v1.0.5 at 2 sites (5th-recurrence; S-7.01 sibling-file gap)

File: SS-engine-module.md lines 1256 and 1267.

Current SS-daemon-lifecycle.md: v1.0.6. SS-engine-module.md §Trace cites v1.0.5:
- Line 1256: "...struct referenced in SS-daemon-lifecycle.md BC-RING-001. See SS-daemon-lifecycle.md v1.0.5." — "See X v1.0.5" is current-pointer directive, not historical pinpoint.
- Line 1267: "HookEventRecord is now defined in SS-daemon-lifecycle.md §HookEventRecord (v1.0.5)." — "now defined in (v1.0.5)" is unambiguously current-state pointer.

Applying architect's own test from SS-forward-compatibility v1.2.2 round-39: these lines fail the historical-pinpoint phrasing test. They direct readers to a superseded version. **Critically:** v1.0.6 added `#[non_exhaustive]` to HookEventRecord (F-R30-2). Reader consulting v1.0.5 per directive finds HookEventRecord WITHOUT the attribute — materially obsolete.

Why round-39 "full sweep" missed: architect's sweep was within-file only (SS-forward-compatibility). Sibling files (SS-engine-module) carrying same pattern not enumerated. Violates S-7.01 part (b) "Sibling files in same architectural layer."

D-042 manual-mitigation acceptance is challenged: the rule was added in same burst that bumped SS-forward-compatibility to v1.2.2 (round-39) but was not retroactively applied to find pre-existing stale citations in sibling files at that moment. Author discipline mitigation insufficient on first practical application.

Fix: rewrite both as historical pinpoints (recommended) or refresh to v1.0.6:
- Line 1256 → "...struct referenced in SS-daemon-lifecycle.md BC-RING-001 (introduced at v1.0.5; #[non_exhaustive] added in v1.0.6)."
- Line 1267 → "HookEventRecord was first defined at SS-daemon-lifecycle.md v1.0.5 §HookEventRecord (current v1.0.6 adds #[non_exhaustive])."

Routing: architect.

## Observations

### O-R40-1 [process-observation] — Fresh-context novelty trajectory across rounds 22-40 is essentially flat

9 adversary rounds (22, 24, 26, 28, 30, 32, 34, 36, 38, 40) each produced 1-3 substantive findings. Novelty has NOT decayed. Both R40 findings are at layers (CLI-invocation; sibling-file dimension) prior rounds did not touch — fresh-context value did not asymptote at the 3rd or 9th pass.

This challenges the "3 clean passes is achievable threshold" assumption. Convergence may require revisiting definition (e.g., "novelty decay to nitpicks only" may be unreachable for spec-set complexity at this scale). Surface for orchestrator + human consideration.

## Severity Trajectory

| Round | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| R22 | 0 | 0 | 3 | 0 |
| R24 | 0 | 0 | 3 | 2 |
| R26 | 1 | 0 | 2 | 3 |
| R28 | 0 | 2H | 2 | 2 |
| R30 | 0 | 1H | 2 | 1 |
| R32 | 0 | 0 | 2 | 2 |
| R34 | 1 | 2I | 0 | 0 |
| R36 | 0 | 0 | 2 | 1 |
| R38 | 0 | 0 | 2 | 0 |
| R40 | 0 | 0 | 2 | 0 |

Convergence count: 0/3 consecutive clean passes since cycle start.

## Recommendation

Round 41 fix burst:
1. Architect on SS-conventions v1.11: F-R40-1 CLI invocation fix.
2. Architect on SS-engine-module v1.1.11: F-R40-2 rewrite §Trace lines 1256/1267 as historical pinpoints.
3. State-manager round-41 close-out.
4. Round 42 validation. If CLEAN, this is 1-of-3 clean passes.

Round 41 fix should ALSO retroactively apply the D-042 manual workflow rule across ALL spec files (not just files being edited) to surface any other sibling-file stale citations BEFORE round-42 validation finds them.
