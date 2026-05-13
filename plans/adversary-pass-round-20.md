---
document_type: adversarial-review-report
level: ops
version: "1.0"
status: complete
producer: adversary (fresh context, round 20, production-grade lens) — transcribed by state-manager during durability close-out
phase: pre-phase-1-final-gate-round-20-complete
timestamp: 2026-05-13T14:30:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md  # v1.1.2
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md  # v1.2.2
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md  # v1.1.5
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-19 fix burst commits 4e386d9 + 33b5a0a + state close-out 1b26c54; resolves round-18 F-R18-1/2/3/4 cleanly with 3 new findings"
project: monocle
verdict: MULTIPLE_DEFER_PATTERNS
---

# Adversarial Pass — Round 20

## Verdict
MULTIPLE_DEFER_PATTERNS — 0 CRITICAL + 2 MEDIUM + 1 LOW. Round-19 fixed all 4 prior findings; the F-R18-1 remediation (BaseDirs::new()) introduced 1 new MEDIUM + 2 sibling-coherence/propagation gaps. CRITICAL severity class converged to zero across the package.

## Disposition of Round-18 Findings
- F-R18-1 CRITICAL (ProjectDirs → ~/.claude/): **RESOLVED** via `BaseDirs::new().map(|b| b.home_dir().join(".claude"))` at SS-engine-module.md lines 307-309, 343-345
- F-R18-2 MEDIUM (constructor rustdoc + InvalidHookUrl): **RESOLVED** (rustdoc on both constructors + PreflightError variant added)
- F-R18-3 MEDIUM (frontmatter parser): **RESOLVED** for `parse_frontmatter_extra_fields` (lines 758-808)
- F-R18-4 LOW (BC-ENGINE-002 wording): **RESOLVED** (line 545)

## New Defects (Round 20)

### F-R20-1 MEDIUM — Silent fallback masks $HOME-unresolvable case
File: `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md` lines 309 + 345.

`BaseDirs::new()` returns `Option<BaseDirs>`. Architect's fix uses `.unwrap_or_else(|| PathBuf::from(".claude"))` as fallback for the `None` case. Production-grade defect: silent fallback substitutes a **relative path** for the unresolvable case (container deployments, systemd `User=` units with no `Environment=HOME`, broken passwd entries). Downstream code (TUI display, DTU validator, transcript watcher) treats the returned path as absolute. Per CLAUDE.md SOUL #4 (silent failures forbidden) and Rule 1 (no MVP rationalizations).

**Correct fix:** Return a typed error. Either change `metadata()` to return `Result<EngineMetadata, EngineMetadataError>` OR add a `home_unresolvable: bool` field that downstream surfaces in `preflight()` failure. Recommendation: typed error route — add `EngineMetadataError::HomeUnresolvable` variant; `metadata()` returns Result; daemon initialization fails fast with a useful error.

**Routing:** architect (SS-engine-module + metadata trait method signature).

### F-R20-2 MEDIUM — `parse_frontmatter_field` lacks the safety guards added to its sibling
File: `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md` lines 712-739 vs 758-808.

F-R18-3 fix was applied to `parse_frontmatter_extra_fields` (lines 758-808): strips quotes; skips empty values, flow lists `[`, block scalars `|`/`>`, continuation lines. The sibling function `parse_frontmatter_field` (lines 712-739) got ONLY the quote-strip half.

Consequences:
- `phase: |` block-scalar marker → returns `Some("|")` (rustdoc at lines 703-705 promises None for block scalars — contract violation).
- `current_cycle: ` (trailing space, empty value) → returns `Some("")`, semantically distinct from `None`; BC-FACTORY-002's `Some(_) or None` assertion does not catch this.
- `awaiting: [a, b]` flow-list → returns `Some("[a, b]")` rather than skipping.

**Correct fix:** Add the same empty/list/block/continuation guards to `parse_frontmatter_field` that exist in `parse_frontmatter_extra_fields`. Return `None` for block scalars + flow lists + empty values.

**Routing:** architect (SS-core-types-and-abi).

### F-R20-3 LOW — Rustdoc references unpinned `url` crate
File: `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md` line 413.

Rustdoc for `ClaudeCodeModule::new` recommends callers `Url::parse(&hook_base_url)` from the `url` crate, but `url` is NOT in `SS-deps-pin-manifest.md`. Implementers following the doc would pull an unpinned transitive.

**Correct fix:** Either (a) remove the suggestion from rustdoc, OR (b) pin `url ^2` in SS-deps as a workspace dep. Recommendation: (a) — keep the rustdoc minimal; documentation should describe contract, not recommend specific crates implementers might use.

**Routing:** architect.

## Severity Trajectory (post-FC bursts)

| Round | CRITICAL | HIGH/MEDIUM | LOW |
|---|---|---|---|
| R12 (FC) | 4 | 6 | 4 |
| R14 | 3 | 5 | 0 |
| R16 | 1 | 4 | 0 |
| R18 | 1 | 2 | 1 |
| R20 | **0** | **2** | **1** |

CRITICAL count converged. MEDIUM count plateauing at 2-3 per round but not increasing.

## Recommendation
FIX (3 surgical fixes; ~20-30 min architect work). Then round 22 validation. F-R20-1 is the most important — silent fallback violation of CLAUDE.md SOUL #4.
