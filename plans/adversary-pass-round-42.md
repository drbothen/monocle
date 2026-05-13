---
document_type: adversarial-review-report
level: ops
version: "1.0"
status: complete
producer: adversary (fresh context, round 42, production-grade lens) — transcribed by state-manager during round-42 durability
phase: pre-phase-1-final-gate-round-42-complete
timestamp: 2026-05-13T23:30:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md  # v1.1.11
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md  # v1.2.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md  # v1.0.6
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md  # v1.1.7
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md  # v1.11
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-forward-compatibility.md  # v1.2.2
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md  # v1.4.18
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-41 fix burst commits 0bf426a + 6fc5ef4 + eaf4adf + fa3820c; validates F-R40-1 + F-R40-2 resolution; surfaces 1 MEDIUM (POL-11 partial-arm-coverage in 3 sibling semgrep rules — F-R32-2 dual-shape discipline never propagated to siblings); 0/3 clean adversary passes maintained"
project: monocle
verdict: NEEDS_ONE_MORE
---

# Adversarial Pass — Round 42

## Verdict
NEEDS_ONE_MORE — 0 CRITICAL + 0 HIGH + 1 MEDIUM + 0 LOW + 1 OBSERVATION. F-R40-1 and F-R40-2 genuinely resolved. New finding: POL-11 partial-arm-coverage gap in 3 of 5 semgrep rules — F-R32-2 dual-shape discipline applied to ONE rule but never propagated to siblings in the same §Semgrep Rules block (S-7.01 propagation gap).

## Disposition of Round-40 Findings

- F-R40-1 (CLI override gap): GENUINELY RESOLVED. SS-conventions v1.11 Step 3 invocation has no `--include` flag; rule paths.include (12 paths) governs scope exclusively. §Trace narrates Option-A rationale.
- F-R40-2 (5th-recurrence stale citation): GENUINELY RESOLVED. SS-engine-module v1.1.11 §Trace lines 1268-1270 rewritten as historical pinpoint with explicit current-state annotation ("introduced at v1.0.5; #[non_exhaustive] attribute added in v1.0.6 per F-R30-2").

## Important Findings

### F-R42-adv-1 MEDIUM — POL-11 partial-arm-coverage in 3 sibling semgrep rules (S-7.01 propagation gap from F-R32-2)

File: SS-conventions-anti-patterns.md.

The F-R32-2 fix applied dual-shape (Shape A + Shape B) fixture corpus discipline to `monocle-non-exhaustive-struct-audit-completeness`. That fix was NOT propagated to sibling rules in the same §Semgrep Rules block:

1. `monocle-no-shell-injection` (2 arms: `sh`, `bash`): fixture (line 214) exercises only `Command::new("sh")`. **1 of 2 arms unverified.**
2. `monocle-no-naked-fs-write` (2 arms: `std::fs::write`, `tokio::fs::write`): fixture (line 215) exercises only `std::fs::write`. **1 of 2 arms unverified.**
3. `monocle-no-raw-env-mutation-in-tests` (4 arms: set_var × 2 forms, remove_var × 2 forms): fixture (line 217) exercises 2 of 4 (set_var only). Step 1 CI assertion (lines 295-299) explicitly says expected count = 2 and labels the other 2 arms as "implicitly covered" + "may add to make explicit". This is incorrect — semgrep cannot infer remove_var arms work from a fixture containing only set_var calls.

If any rule's `remove_var`, `bash`, or `tokio::fs::write` pattern syntax accidentally breaks (semgrep version regression, YAML typo), CI emits GREEN because:
- Step 1: only asserts findings for the exercised arms — passes regardless
- Step 2: production has no violations either way — passes

This is the IDENTICAL POL-11 false-green pattern that F-R32-2 fixed for one rule. The fix was applied to 1 of 4 affected rules; siblings still carry the gap.

**Severity rationale:** S-7.01 sibling-rule propagation gap. Same-file sibling-rule propagation = MEDIUM per blast-radius convention; 3 sibling rules with same gap meets pattern-frequency threshold for HIGH-flag. Classifying MEDIUM with explicit S-7.01 propagation pattern flag.

**Fix direction:** Mandatory all-arm fixture coverage with computed expected counts. Update fixture corpus table for each of 3 rules:
- shell_injection: require both sh and bash, expected count 2
- naked_fs_write: require both std::fs::write and tokio::fs::write, expected count 2
- raw_env_mutation_in_tests: require all 4 patterns (set_var × 2, remove_var × 2), expected count 4

Replace "implicitly covered" / "may" optionality with normative "must match N findings (one per pattern-either arm)".

Routing: architect.

## Observations

### O-R42-1 [process-gap] LOW — D-042 manual sweep grep regex misses citations with intervening section anchors

File: SS-engine-module.md line ~1281 has citation "SS-daemon-lifecycle.md §HookEventRecord at v1.0.5" — the section anchor `§HookEventRecord` between doc name and version. Architect's D-042 grep pattern `SS-[name].md v` would NOT match this (regex stops at first non-`v` character after the doc name).

The citation is correctly classified as historical pinpoint with "first defined" language, so no current-pointer staleness here. But the regex has a known false-negative class. Future cross-artifact sweeps should use a more permissive pattern that tolerates intervening text.

### O-R42-2 [convergence-observation] — Novelty trajectory still non-zero at round 10

10 adversary rounds (R22-R42) produced findings counts: 3, 5, 6, 6, 4, 4, 3, 3, 2, 2. Modestly decreasing but not at zero. Each round surfaces genuinely novel findings — F-R40-1 at CLI layer, F-R42-adv-1 at sibling-rule layer — at adjacent dimensions to prior fixes.

This challenges the "3 clean passes will be achieved soon" assumption. Convergence may require more rounds than initially projected, OR a definitional acknowledgment that "novel-novelty decay to nitpicks only" is unreachable for spec-set complexity at this scale.

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
| R42 | 0 | 0 | 1 | 0+1obs |

Convergence count: 0/3 consecutive clean passes since cycle start.

## Recommendation

Round 43 fix burst:
1. Architect SS-conventions v1.12: propagate F-R32-2 dual-shape discipline to 3 sibling rules; update fixture table + Step 1 CI assertion wording.
2. Round 44 validation. If CLEAN, 1-of-3.

Per O-R42-2: orchestrator should surface convergence trajectory data to human after R44 result. If novelty remains non-zero, human may need to ratify "convergence acceptable at severity decay" rather than strict "3 zero-finding passes."
