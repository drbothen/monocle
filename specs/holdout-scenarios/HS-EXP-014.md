---
scenario_id: HS-EXP-014
title: "Hook Auto-Injection Under Concurrent Spawns — Shared hooks-settings.json Not Clobbered; All Sessions Get Correct `--settings` Arg"
wave: 8
stories_tested: [S-033, S-038]
source_bcs: [BC-2.08.006, BC-2.08.001, BC-HOOK-010]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T12:00:00Z
modified: 2026-06-13T00:00:00Z
---

# HS-EXP-014: Hook Auto-Injection Under Concurrent Spawns — Shared hooks-settings.json Not Clobbered; All Sessions Get Correct `--settings` Arg

**Wave:** 8
**Source BC:** BC-2.08.006 (postconditions PC-1, PC-2), BC-2.08.001 (PC-1, PC-3), BC-HOOK-010 (PC-1, PC-3)
**Stories Tested:** S-033, S-038

## Setup

A running daemon with a `MockSessionHostSpawner` that records spawn calls. The `runtime_dir`
is a `tempfile::TempDir`. The daemon has already written the shared `<runtime_dir>/hooks-settings.json`
at startup (per BC-HOOK-010: a single shared file per runtimeDir, written once at daemon start, NOT
per-session). All sessions spawned in this test share this single file.

## Steps

### Part A: Sequential spawn — `--settings` arg present

1. Construct a `SpawnOptions` value `opts_A` carrying `harness_id: harness_A`,
   `profile_id: profile_A`, `project_root: <a test project path>`,
   `worktree_root: <same path, or a configured worktree root>`,
   `session_id: <daemon-pre-generated UUID v4 string>`,
   `hooks_settings_path: <runtime_dir>/hooks-settings.json` (the shared path written at
   daemon startup), and `ccr_base_url: None`.
   Call `SessionManager::spawn_session(opts_A)` for session S1.
   NOTE: `session_id` and `hooks_settings_path` are daemon-filled fields (the daemon IPC
   handler populates them before calling `spawn_session()`; the TUI does not set them).
   In the test harness, the `MockSessionHostSpawner` fixture populates them directly since
   there is no IPC layer.
2. Verify within 2 seconds: `MockSessionHostSpawner` was called with args that include
   `--settings <runtime_dir>/hooks-settings.json` (the single shared file path).
3. Verify: `session-state.json` for S1 was written at flat path `<runtime_dir>/session-<S1-uuid>.json`
   and contains the standard schema fields (schema_version, session_id, pid, socket_path, state,
   project_root, cwd, harness_id, profile_id, started_at, display_name, pty_rows, pty_cols).
   NOTE: The sidecar MUST NOT contain a `hook_settings_path` field — that field does not exist
   in the session-state.json schema (SS-session-manager v2.13.0 §session-state.json schema). The
   hooks path is a shared constant (`<runtime_dir>/hooks-settings.json`) baked into every
   session's spawn args via `SpawnOptions.hooks_settings_path`; it is not stored redundantly
   in the sidecar.
4. Verify: NO per-session hooks file was created (i.e., no `runtime_dir/session-<S1-uuid>-hooks.json`
   or similar; only the shared `hooks-settings.json` exists).

### Part B: Concurrent spawns — shared file survives unchanged

5. Launch 5 concurrent `spawn_session()` calls in parallel tokio tasks:
   - S2 through S6, each with a distinct `harness_id` and `profile_id`.
   - All 5 calls issued simultaneously via `tokio::join!`.

6. Wait for all 5 calls to complete (within 2 seconds each per BC-2.08.001 PC-1).

7. Verify: `hooks-settings.json` exists at `runtime_dir/hooks-settings.json` and is valid JSON
   (not a torn write or zero-byte file). Content must be byte-for-byte identical to what the
   daemon wrote at startup — no concurrent spawn modified it.

8. Verify: all 5 `MockSessionHostSpawner` calls each received `--settings <runtime_dir>/hooks-settings.json`
   (the SAME shared path). No spawn call received a different path, a per-session path, or an
   absent `--settings` arg.

9. Verify: each session S2..S6 has a `session-state.json` sidecar written successfully at its
   flat path `<runtime_dir>/session-<uuid>.json`. No sidecar is missing or zero-byte. None of
   the sidecars contain a `hook_settings_path` field (that field is not in the schema — absence
   is the correct outcome; presence would be a schema violation).

10. Verify: NO per-session hooks files were created alongside the sidecars. The only hooks
    file in runtime_dir is the shared `hooks-settings.json`.

## Expected Outcome

- Part A: `--settings <runtime_dir>/hooks-settings.json` arg is present and correct for the
  sequential spawn. No per-session hooks file is created. Sidecar does NOT contain
  `hook_settings_path` field (that field is not in the schema).
- Part B: `hooks-settings.json` content is unchanged after 5 concurrent spawns (spawns do NOT
  write to it; the file is written once at daemon startup). All 5 sessions received the shared
  `--settings <runtime_dir>/hooks-settings.json` arg. No per-session hooks files exist.
- Part B: Concurrent-safety via read-only access during spawn — because spawns only READ the
  shared file path (they do not write it), there is no concurrent-write race condition to check
  for. The production-grade concern is that no spawn accidentally writes a different file or
  omits the `--settings` arg entirely.

## Adversarial Probe

Set the tokio runtime to 2 threads (reducing scheduling non-determinism) and repeat the 5-concurrent-
spawn test 10 times. All 10 runs must produce the same result: `hooks-settings.json` unchanged from
daemon-startup content; correct `--settings <runtime_dir>/hooks-settings.json` arg for all sessions;
no per-session hooks files in runtime_dir.

## Satisfaction Criteria

PASS: All concurrent spawns complete within 2 seconds; `hooks-settings.json` content is unchanged
from daemon-startup content after all spawns; every spawn received `--settings <runtime_dir>/hooks-settings.json`;
no per-session hooks files exist in runtime_dir; no session-state.json sidecar is missing or corrupt.

FAIL: `hooks-settings.json` is empty, truncated, or modified from its daemon-startup content after
concurrent spawns (indicates spawns erroneously wrote to it); any spawn call is missing the `--settings`
arg or points to a per-session path; any `session-state.json` sidecar is zero-byte or absent;
any spawn takes >2 seconds; any per-session hooks file created (violates BC-HOOK-010 PC-3);
any `session-state.json` sidecar contains a `hook_settings_path` field (schema violation — field
does not exist in session-state.json schema v3 (`schema_version: 3`) per SS-session-manager v2.13.0).

**NOT in any story AC:** The story implementing BC-2.08.006 will have ACs for the `--settings` arg
presence in a single spawn. This holdout tests the **shared-file model invariant under concurrent load**:
does the shared `hooks-settings.json` remain unmodified after 5 simultaneous spawns? And do all
sessions consistently reference the same shared path? The BC-HOOK-010 guarantee (no per-session files)
and the BC-2.08.006 invariant (spawn does not write hooks file) must hold simultaneously under
concurrency — this cannot be validated by any single AC in the implementing story.

---

**Modification note (C2-004 adversarial pass-2 fix, 2026-06-03):** Steps A3, B9, FAIL criteria
updated to remove the assertion that `session-state.json` contains a `hook_settings_path` field.
That field does not exist in the session-state.json schema v3 (`schema_version: 3`) (SS-session-manager v1.7.0 at time of C2-004 fix <!-- version-pin-historical: v1.7.0 was canonical at C2-004 authoring time --> §session-
state.json schema). The architect noted the field is redundant — the hooks path is a shared
constant (`<runtime_dir>/hooks-settings.json`) passed via spawn args, not stored per-session in
the sidecar. The holdout now asserts the ABSENCE of the field (schema compliance) and instead
verifies correct `--settings` arg injection for all concurrent sessions.

---

**§Trace v1.2 — Pass-28 I28-001: live Model-B spawn signature corrected to Model A SpawnOptions (2026-06-13)**

- **Finding (I28-001 — IMPORTANT):** Part A Step 1 (line ~31) contained the RETIRED Model-B
  three-argument `spawn_session()` signature: `SessionManager::spawn_session(recipe_A, harness_A, profile_A)`.
  Model B was superseded by Model A (I27-001 adjudication, SS-session-manager v2.0.0 <!-- version-pin-historical: v2.0.0 was canonical at Pass-28 authoring time --> §Public API,
  BC-2.08.001 v1.4.0 <!-- version-pin-historical: v1.4.0 was canonical at Pass-28 authoring time -->). Under Model A, `spawn_session()` accepts a single `SpawnOptions` struct;
  `SpawnRecipe` is daemon-internal and never appears on the call site.
- **Fix:** Step 1 rewritten to the Model A call: construct `opts_A: SpawnOptions` with
  `harness_id`, `profile_id`, `project_root`, `worktree_root`, `session_id`,
  `hooks_settings_path`, and `ccr_base_url` fields populated per the SS-session-manager v2.0.0 <!-- version-pin-historical: v2.0.0 was canonical at Pass-28 authoring time -->
  §SpawnOptions field-population split (TUI populates `project_root`/`worktree_root`/`harness_id`/
  `profile_id`/`ccr_base_url`; daemon fills `session_id`/`hooks_settings_path`), then call
  `SessionManager::spawn_session(opts_A)`.
- **Scope:** Part A Step 1 only. All other live Steps already used Model A language. §Trace
  entries are historical records — not updated.
- **Sweep of sibling holdouts HS-EXP-011/012/013/015:** grep confirmed ZERO Model-B
  spawn-signature residue in any live Steps section of those files. No further fixes required.
- Behavioral semantics unchanged: S1 is still spawned with `harness_A`/`profile_A`. Only the
  call-site notation is corrected to match the ratified v2.0.0 public API.

---

**§Trace v1.1 — Pass-9 S-P9-001: stale schema v2 label corrected to v3 (2026-06-03)**

- FAIL criterion (line ~100): `schema v2 per SS-session-manager v1.7.0 at Pass-9 authoring time` → `schema v3 (\`schema_version: 3\`) per SS-session-manager v1.7.0 at Pass-9 authoring time`.
- Modification note (lines ~113-114): same reference updated to `schema v3 (\`schema_version: 3\`)`.
- Behavioral semantics unchanged: the `hook_settings_path` absence assertion is true in both v2 and v3 schemas. The v3 label is correct because SS-session-manager v1.7.0 at Pass-9 authoring time canonically defines `schema_version: 3` (v3 added `kill_deadline_unix_ms`; no hook_settings_path field was ever present). The "schema v2" label was a stale cosmetic error from the original C2-004 modification note.
