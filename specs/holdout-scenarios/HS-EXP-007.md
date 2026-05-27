---
scenario_id: HS-EXP-007
title: "Config Atomic Write: tempfile::persist — No Partial Config Under Concurrent Save Failure"
wave: 4
stories_tested: [S-030]
source_bcs: [BC-2.07.002, BC-2.07.006]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-007: Config Atomic Write — tempfile::persist Leaves No Partial Config Under Concurrent Save Failure

**Wave:** 4
**Source BC:** BC-2.07.006 (postcondition PC-1), BC-2.07.002 (invariant INV-1)
**Stories Tested:** S-030

## Setup

An existing `config.json` at the canonical path with valid content:
```json
{"schema_version": 1, "active_profile": null, "profiles": {}, "binding_overrides": {}}
```

## Steps

### Part A: Simulated mid-write failure

1. Call `MonocleConfig::save()` with a mock that panics after `NamedTempFile::new_in()` but
   before `persist()` is called (simulating an I/O error after write, before rename).
2. After the panic is caught (via `std::panic::catch_unwind`), read the config file at `path`.
3. Verify the original content is intact.

### Part B: Direct `std::fs::write` detection

4. Run `grep -r "std::fs::write" monocle-config/src/` in the repo.
5. Verify zero matches (the semgrep rule `monocle-no-direct-config-write` is enforced structurally).

### Part C: binding_overrides null-coercion

6. Create a JSON file with `"binding_overrides": null` at the config path.
7. Call `MonocleConfig::load(path)`.
8. Assert the result is either an error (`Err`) OR `Ok` with `binding_overrides` coerced to `{}`.
   (The invariant is that `binding_overrides` MUST NOT be `Value::Null` after load; a `null`
   in the file must never silently produce `Value::Null` in the struct.)

### Part D: Atomicity across rename

9. Create a test that starts `save()` and, while the tempfile exists but `persist()` has not
   yet been called, reads the original config path. Verify the original file is untouched.
10. Allow `persist()` to complete. Verify the new config content is now at `path`.

## Expected Outcome

- Part A: Original config file is intact after the simulated mid-write failure. No zero-byte
  or partial-write file at `path`.
- Part B: Zero matches for `std::fs::write` in `monocle-config/src/`.
- Part C: `MonocleConfig::load()` on a file with `"binding_overrides": null` does NOT return
  `Ok(config)` where `config.binding_overrides == Value::Null`. Either rejected or coerced to `{}`.
- Part D: Config path is either the original content or the new content — never empty, never partial.

## Satisfaction Criteria

PASS: All four parts pass. No partial write at any point.

FAIL: Part A leaves a partial or empty file; Part B finds any `std::fs::write` usage;
Part C silently accepts `binding_overrides: null` as `Value::Null`; Part D shows a window
where the config path is absent or corrupted.

**NOT in any story AC:** S-030 ACs test `save()` uses tempfile (AC-005), parent dir creation (AC-006),
and binding_overrides invariant (AC-012). This holdout tests the `Value::Null` coercion behavior
explicitly (the spec says "rejected or coerced to {}" — which behavior does the implementation choose?)
and the mid-write crash recovery observable from the filesystem. The direct-write detection (Part B)
is a structural enforcement test not covered by any AC.
