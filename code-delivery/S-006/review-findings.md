---
document_type: review-findings
story_id: S-006
pr_number: 7
cycle: 1
status: converged
---

# PR #7 — S-006 Review Findings

## Convergence Table

| Cycle | Total Findings | Blocking | Non-Blocking | Fixed | Remaining | Status |
|-------|---------------|----------|--------------|-------|-----------|--------|
| 1 | 2 | 0 | 2 | 0 | 2 | APPROVE (0 blocking) |

## Cycle 1 — Findings

### Finding RF-001

- **ID:** RF-001
- **Severity:** suggestion (non-blocking)
- **Category:** description
- **Location:** `crates/monocle-runtime/src/lock.rs` line 97 — `LockFileContent` struct doc comment
- **Finding:** The doc comment states `"serde_json with the preserve_order feature (or use of an ordered map) is required to guarantee insertion order."` This is misleading. The implementation correctly uses `#[derive(Serialize)]` on a Rust struct, which guarantees field serialization in declaration order without needing the `preserve_order` feature. The `preserve_order` feature only affects deserialization into `serde_json::Value` (making the underlying Map an `IndexMap` instead of `BTreeMap`). The code path is correct; the doc comment overstates what is needed.
- **Route to:** pr-manager (description/doc-comment accuracy, no behavioral change)
- **Action:** Update the doc comment sentence to: `serde_json serializes struct fields in declaration order (a documented guarantee of serde). No preserve_order feature flag is required for struct-based serialization.`
- **Blocking?** NO — the implementation is correct; this is documentation accuracy only.

### Finding RF-002

- **ID:** RF-002
- **Severity:** suggestion (non-blocking)
- **Category:** description
- **Location:** `crates/monocle-runtime/src/lock.rs` line 96–97 — `DaemonLock` struct comment
- **Finding:** The comment above `#[allow(dead_code)]` reads: `"Fields are populated by the implementer (S-006). Stubs leave bodies as unimplemented!()."` This is a stale TDD-stub-phase comment. The implementation is complete; no stubs remain. The comment now incorrectly describes the state of the code.
- **Route to:** pr-manager (stale comment cleanup, no behavioral change)
- **Action:** Remove or replace the comment. The `#[allow(dead_code)]` may also be examinable: `path` and `sock_path` are `pub(crate)` and accessed in `release()`, so they are not dead code. If clippy `-D warnings` does not flag them, the attribute can also be removed. Suggested replacement for the comment if the attribute is kept for belt-and-suspenders: `// path and sock_path are pub(crate) to allow test-only inspection.`
- **Blocking?** NO — cosmetic/comment accuracy only.

## Triage Summary

| Finding | Severity | Category | Routed To | Status |
|---------|----------|----------|-----------|--------|
| RF-001: misleading preserve_order doc comment in LockFileContent | suggestion | description | pr-manager (inline code comment edit) | pending |
| RF-002: stale TDD stub comment on DaemonLock + stale allow(dead_code) rationale | suggestion | description | pr-manager (inline code comment edit) | pending |

## Review Verdict

**APPROVE** — 0 blocking findings.

### Positive Assessment

**Behavioral contracts:** BC-2.01.005 (8 postconditions + invariants), BC-2.01.008 (PC-1 cryptographic token), BC-2.01.010 (contract version edge cases) are all correctly implemented and fully tested.

**Security surface — all properties verified:**
- `rand::rngs::OsRng.fill_bytes()` — correct CSPRNG, EXACT `=0.8.6` pin (no 0.9 regression).
- `tempfile::NamedTempFile::persist()` — POSIX rename(2) atomicity; no `std::fs::write` anti-pattern.
- File mode `0o600` via `set_permissions` after persist — assertion uses `mode() & 0o777` (correct).
- Directory mode `0o700` via `DirBuilder::new().mode(0o700)` with `DirBuilderExt` — NOT `create_dir_all` (correct).
- Port-zero rejection guard (F-S006-ADV1-001 CRIT fix) — verified present and tested.
- `ESRCH` → stale, all other errno (including `EPERM`) → `LiveConflict` (F-S006-ADV1-002 HIGH fix) — verified.
- Token generation occurs AFTER all validation (F-S006-ADV1-003 HIGH fix) — verified in `acquire()` step ordering.
- No `unsafe` blocks anywhere in new code.

**Architecture compliance:**
- `LockFileContent` struct field declaration order = `contract_version, pid, port, authToken, startTimeUtc, app, version` — matches AC-002 schema exactly.
- Struct serialization via `#[derive(Serialize)]` guarantees declaration-order JSON output — correct.
- `nix::sys::signal::kill(Pid::from_raw(pid), None)` for pid-liveness — canonical per story spec.
- `i32::try_from(std::process::id())` with proper `map_err` — no panic on PID overflow.
- Stale lock removal failure handled gracefully (WARN log, proceeds) — correct resilience pattern.
- Sock file not-found on release treated as success — correct for the daemon-crashed-before-bind case.

**Test quality:**
- 30 tests across 2 integration test files; all 14 ACs covered.
- `write_test_fixture_to` helper uses `NamedTempFile + persist` even in test code — correct anti-pattern compliance.
- Raw string position scan for `contract_version` first-key assertion — stronger than parsed-JSON assertion alone.
- Randomness invariant test (`token_a != token_b`) — collision probability 2^-256.
- `DEAD_PID = i32::MAX - 1` — correct sentinel (exceeds kernel PID max on all platforms).

**Missing_docs compliance:** All public items in `errors.rs`, `lifecycle.rs`, `lock.rs`, and `auth.rs` (new function) carry doc comments. `#![deny(missing_docs)]` will pass.

**`forbid(unsafe_code)` compliance:** No `unsafe` blocks in any new file.

Non-blocking RF-001 and RF-002 (doc comment accuracy) do not affect merge eligibility.
