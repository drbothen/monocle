---
scenario_id: HS-EXP-001
title: "SOQ-2 Ordering: hooks-settings.json Is Never Written Before Lock File"
wave: 5
stories_tested: [S-017]
source_bcs: [BC-2.04.001, BC-2.04.010]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-001: SOQ-2 Ordering — hooks-settings.json Is Never Written Before Lock File

**Wave:** 5
**Source BC:** BC-2.04.001 (postconditions PC-15, PC-16), BC-2.04.010 (postcondition PC-4)
**Stories Tested:** S-017

## Setup

A clean `<runtime_dir>` with no existing lock file, no existing `hooks-settings.json`, and
no running daemon. Filesystem `inotify` (Linux) or `kqueue` (macOS) watch placed on
`<runtime_dir>` before daemon start.

Alternatively: two integration test assertions on file modification timestamps.

## Steps

1. Start the daemon via `daemon_start_sequence()`.
2. Record the precise mtime of `monocle.lock` at the instant it is first observed on disk.
3. Record the precise mtime of `hooks-settings.json` at the instant it is first observed on disk.
4. (Optional adversarial probe) Attempt to inject a race: kill the daemon process between step 8
   (lock write) and step 9 (hooks-settings write). Verify that when the daemon exits mid-sequence,
   `hooks-settings.json` does NOT exist (lock file cleanup invariant 6 removes `monocle.lock`).

## Expected Outcome

- `monocle.lock` mtime is strictly less than or equal to `hooks-settings.json` mtime.
  (`lock_mtime <= hooks_settings_mtime` — same-second writes are acceptable only if the
  filesystem clock resolution permits it; the liveness test is that `hooks-settings.json`
  is NEVER older than `monocle.lock`.)
- `hooks-settings.json` is NEVER present without a corresponding `monocle.lock`.
- When the daemon is killed between step 8 and step 9: both files are absent after process exit
  (invariant 6 cleanup removes `monocle.lock`; `hooks-settings.json` was never written).
- `hooks-settings.json` contains exactly the schema from BC-2.04.010 PC-3:
  non-empty `PreToolUse`, `Notification`, `Stop`, and `UserPromptSubmit` hook arrays;
  empty `PostToolUse: []`; empty `PreCompact: []`; `SessionStart` key is ABSENT.

## Satisfaction Criteria

PASS: All three assertions hold in 10 consecutive daemon-start runs (no flakiness).

FAIL: Any run where `hooks-settings.json` exists with an older mtime than `monocle.lock`, OR
where `hooks-settings.json` exists but `monocle.lock` is absent, OR where `SessionStart` appears
in `hooks-settings.json`.

**NOT in any story AC:** S-017 ACs (AC-008, AC-012) test that the write functions are called
in the correct step order within `daemon_start_sequence()`. This holdout tests the observable
*on-disk artifact ordering* and the mid-sequence-kill invariant (no partial state on disk) —
neither is mechanically stated in the AC corpus.
