---
scenario_id: HS-EXP-009
title: "Daemon Binary: runtime_dir Level 4 Fail-Fast Produces Exit Code 70 Not 1"
wave: 4
stories_tested: [S-016]
source_bcs: [BC-2.04.006, BC-2.04.004]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-009: Daemon Binary — runtime_dir Level 4 Fail-Fast Produces Exit Code 70 Not 1

**Wave:** 4
**Source BC:** BC-2.04.006 (postconditions PC-12, PC-13), BC-2.04.004 (postcondition PC-8)
**Stories Tested:** S-016

## Setup

An environment where `ProjectDirs::from("", "", "monocle")` returns `None`. This is achieved by
unsetting all known home-directory environment variables:
- `HOME` (POSIX)
- `USERPROFILE` (Windows — not applicable on macOS/Linux but included for hygiene)
- `XDG_RUNTIME_DIR`
- `XDG_DATA_HOME`
- `XDG_CONFIG_HOME`

Also unset `MONOCLE_RUNTIME_DIR` to ensure Level 1 bypass does not hide the failure.

## Steps

1. Run `monocle daemon start` with no home-dir env vars and no `MONOCLE_RUNTIME_DIR`.
2. Capture the exit code.
3. Capture stderr output.
4. Run `monocle daemon stop` in the same environment.
5. Capture the exit code and stderr.

## Expected Outcome

- Step 2: exit code is `70` (not `1`, not `71`, not any other value).
- Step 3: stderr contains exactly:
  `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`
  (verbatim, no trailing newline variation, no extra context lines that could confuse shell callers).
- No lock file or daemon process is created (the fail-fast fires before any subprocess spawn).
- Step 4-5 (`monocle daemon stop`): exit code is also `70` (the stop command also calls
  `resolve_runtime_dir()` first).

## Satisfaction Criteria

PASS: Exit code 70 on both `start` and `stop`; exact stderr message; no orphaned processes.

FAIL: Exit code 1 instead of 70 (most likely failure — implementer maps all errors to exit 1);
missing or incorrect stderr message; any daemon subprocess spawned before the error fires;
`monocle daemon stop` returns a different exit code than 70 in this environment.

**NOT in any story AC:** S-016 AC-004 tests that `resolve_runtime_dir()` returns
`Err(DaemonStartError::RuntimeDirUnresolvable)` and the CLI exits with code 70. This holdout
tests the OBSERVABLE exit code from the compiled binary in a real no-home-dir shell environment
(not a unit test mock), AND tests that `monocle daemon stop` also returns 70 in the same condition
(stop calls `resolve_runtime_dir()` too — this is an easy-to-miss symmetry). The exact stderr
message byte-for-byte is also verified, which no AC requires.
