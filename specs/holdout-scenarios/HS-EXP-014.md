---
scenario_id: HS-EXP-014
title: "Hook Auto-Injection Under Concurrent Spawns — No hooks-settings.json Clobber; Each Session Gets Correct `--settings` Arg"
wave: 8
stories_tested: [S-TBD-session-manager]
source_bcs: [BC-2.08.006, BC-2.08.001]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T12:00:00Z
---

# HS-EXP-014: Hook Auto-Injection Under Concurrent Spawns — No hooks-settings.json Clobber; Each Session Gets Correct `--settings` Arg

**Wave:** 8
**Source BC:** BC-2.08.006 (postconditions PC-1, PC-2), BC-2.08.001 (PC-1, PC-3)
**Stories Tested:** S-TBD-session-manager

## Setup

A running daemon with a `MockSessionHostSpawner` that records spawn calls. The `runtime_dir`
is a `tempfile::TempDir`. The `hooks-settings.json` file is the shared per-runtime-dir hook
injection file (per BC-HOOK-010: per-runtimeDir, not per-session).

## Steps

### Part A: Sequential spawn — `--settings` arg present

1. Call `SessionManager::spawn_session(recipe_A, harness_A, profile_A)` for session S1.
2. Verify within 2 seconds: `MockSessionHostSpawner` was called with args that include
   `--settings <path_to_hooks-settings.json>` where the path resolves to
   `runtime_dir/hooks-settings.json`.
3. Verify: `session-state.json` for S1 was written and contains the `hook_settings_path` field
   pointing to the same `hooks-settings.json`.

### Part B: Concurrent spawns — no clobber

4. Launch 5 concurrent `spawn_session()` calls in parallel tokio tasks:
   - S2 through S6, each with a distinct `harness_id` and `profile_id`.
   - All 5 calls issued simultaneously via `tokio::join!`.

5. Wait for all 5 calls to complete (within 2 seconds each per BC-2.08.001 PC-1).

6. Verify: `hooks-settings.json` exists at `runtime_dir/hooks-settings.json` and is valid JSON
   (not a torn write or zero-byte file).

7. Verify: all 5 `MockSessionHostSpawner` calls each received `--settings runtime_dir/hooks-settings.json`.
   No spawn call received a different path or an absent `--settings` arg.

8. Verify: each session S2..S6 has a `session-state.json` sidecar written successfully.
   No sidecar is missing or zero-byte.

### Part C: Per-hooks-file sessions (if per-session hook paths are supported)

9. If the implementation uses per-session hook file paths (e.g., `runtime_dir/sessions/S1/hooks-settings.json`),
   verify: no two sessions share the same file path; each session's `--settings` arg points to its
   own file; concurrent writes to different files do not race.

## Expected Outcome

- Part A: `--settings` arg is present and correct for the sequential spawn.
- Part B: `hooks-settings.json` is intact (valid JSON, non-zero size) after 5 concurrent spawns.
  No file corruption from concurrent writes. All 5 sessions received the correct `--settings` arg.
- Part B: Atomic write invariant — if the implementation uses `tempfile::persist` for the
  hooks-settings.json, there is no window where a spawn reads a partially-written file.

## Adversarial Probe

Set the tokio runtime to 2 threads (reducing scheduling non-determinism) and repeat the 5-concurrent-
spawn test 10 times. All 10 runs must produce the same result: valid JSON, correct `--settings` arg
for all sessions.

## Satisfaction Criteria

PASS: All concurrent spawns complete within 2 seconds; `hooks-settings.json` is valid JSON after
all spawns; every spawn received the correct `--settings` arg; no session-state.json sidecar is
missing or corrupt.

FAIL: `hooks-settings.json` is empty, truncated, or invalid JSON after concurrent spawns; any spawn
call is missing the `--settings` arg; any `session-state.json` sidecar is zero-byte or absent;
any spawn takes >2 seconds.

**NOT in any story AC:** The story implementing BC-2.08.006 will have ACs for the `--settings` arg
presence in a single spawn. This holdout tests the **concurrent spawn race condition**: does
`hooks-settings.json` survive 5 simultaneous spawn calls with no torn writes? And does each session's
`--settings` arg point to the correct file? The concurrent mutation of the shared per-runtime-dir
hooks-settings.json (or per-session files if that model is used) is an implementation-level race
that cannot be validated by any single AC in the implementing story.
