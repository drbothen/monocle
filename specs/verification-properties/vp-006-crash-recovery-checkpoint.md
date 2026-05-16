---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
traces_to: prd.md
source_bc: BC-2.01.006
module: monocle-runtime
proof_method: manual+mutation
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-006: Crash Recovery Checkpoint — JSON Write, Offer, Cleanup

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-006 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

During graceful drain (after AppMode `ShuttingDown`, before lock-file
removal), the daemon writes `<runtime_dir>/monocle.recovery.json` atomically
via `tempfile::persist` with a 4-key schema: `pid` (integer ≥ 1),
`shutdown_reason` (closed enum `"graceful" | "signal" | "forced"`),
`last_app_mode` (non-empty string), `shutdown_utc` (ISO 8601 millisecond
regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`). On next start, if
the file exists and prior exit was non-graceful, the daemon logs WARN
`recovery checkpoint found...` and reads state. The 60-second
`Instant::now()`-anchored window from daemon start offers a recovery
message to an attaching TUI client; on ACCEPT/DECLINE/timeout the recovery
file is deleted with the appropriate state transmission. Malformed recovery
files are deleted with WARN log and no UDS offer.

## Source Contract

- **BC (primary):** BC-2.01.006 — Crash Recovery Checkpoint.
- **Postcondition/Invariant:** 4-key schema with typed-field invariants
  (`pid` integer ≥ 1, `shutdown_reason` 3-element closed enum,
  `last_app_mode` non-empty string, `shutdown_utc` ISO 8601 ms regex);
  write-before-lock-remove ordering during drain; 60-second TUI offer
  window measured from `Instant::now()` at daemon start; malformed-file
  handling.
- **Traces to (historical):** BC-DAEMON-006 (PRD v1.25 §BC-DAEMON-006;
  SS-daemon-lifecycle.md v1.0.25 §Crash Recovery; F-R70-2 BC-VP alignment
  closure — millisecond-precision regex was previously tighter than the
  under-specified BC; PRD v1.6 commit 76570ac brought BC into alignment
  with VP).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test + `tokio::time::pause`/`advance` clock + mock UDS client | Bounded — finite probe set; deterministic clock | Drain-time checkpoint write; 4-key schema; TUI ACCEPT/DECLINE/timeout; malformed-file handling; ordering |
| Mutation test (auxiliary) | cargo-mutants | N/A — mutation surface | Type-flip on `pid`; enum-extension drift on `shutdown_reason`; default-init slip on `last_app_mode`; regex looseness on `shutdown_utc` |

## Mechanism

Integration test (harness at `monocle-runtime/tests/crash_recovery.rs` —
files in `<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test
Type column labels this BC `Integration`). The harness uses
`tempfile::TempDir` to isolate `<runtime_dir>` and a `tokio::time::pause()`
+ `tokio::time::advance()` clock to drive the 60-second window
deterministically. A mock UDS client attaches/declines/times-out within the
controlled clock to exercise the recovery-offer state machine.

## Pre-conditions

- Daemon binary supports the synthetic test mode where the recovery
  file is written eagerly on AppMode → `ShuttingDown` (covered by the
  drain code path).
- `tempfile 3`, `serde_json 1`, and `tokio 1` are the project pins (per
  SS-deps-pin-manifest.md v1.1.15).
- `chrono 0.4` is the project pin (per SS-deps-pin-manifest.md v1.1.15)
  for the `shutdown_utc` ISO 8601 millisecond timestamp formatter. The
  recovery checkpoint emits `shutdown_utc` via
  `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")` per
  SS-daemon-lifecycle.md §Drain step 5 and §Trace v1.0.14 F-R72-1
  rationale (cross-field uniformity with `last_hook_ts` and
  `startTimeUtc`).
- Tests use `tempfile::TempDir` to isolate the runtime directory and
  a `tokio::time::pause()` + `tokio::time::advance()` clock to drive
  the 60-second window deterministically.

## Post-conditions

1. Synthetic shutdown signal injected → after the drain code path
   completes, `<temp_runtime>/monocle.recovery.json` exists with content
   matching the 4-key schema and `shutdown_reason == "graceful"`.
2. Pre-created recovery file at daemon start (with an absent or
   stale-pid lock file) → daemon log captures the WARN
   `recovery checkpoint found; prior daemon exited without clean shutdown`.
3. Pre-created recovery file + a mock TUI client attached within 60
   simulated seconds via `tokio::time::advance` → mock TUI receives
   the UDS message `{"type":"recovery_available","last_app_mode":
   "<expected>"}`; after sending `Y`, the recovery file no longer
   exists.
4. Pre-created recovery file + mock TUI sends `N` → recovery file no
   longer exists; mock TUI did NOT receive a state-transmission
   payload after the ACK.
5. Pre-created recovery file + no TUI attaches; 61 simulated seconds
   advance → recovery file no longer exists; daemon log captures
   `WARN: recovery offer expired; deleting checkpoint`.
6. Pre-created recovery file with truncated content (e.g., the closing
   `}` removed) → daemon log captures `WARN: recovery file malformed;
   starting fresh`; recovery file no longer exists; no UDS message sent.
7. Recovery file's `shutdown_utc` value matches the strict ISO 8601
   regex above (no millisecond truncation, no missing `Z` suffix).
8. Recovery file is written BEFORE the lock file is removed during
   drain — the test traces the order of filesystem ops via a `tracing`
   subscriber configured to capture file-write events.
9. **Numeric type for `pid`:** `pid` MUST be a JSON integer ≥ 1 (positive
   process ID per POSIX). Verified by integration test parsing the crash
   recovery JSON and asserting `value["pid"].is_i64() &&
   value["pid"].as_i64().unwrap() >= 1`. Cross-property with VP-002
   §Post-condition 7 numeric-types-and-ranges probe (same `pid integer ≥ 1`
   form applied to the `/status` daemon-state JSON `pid` field).
10. **Enum-value-set for `shutdown_reason`:** `shutdown_reason` MUST be a
    JSON string equal to ONE of the 3 enum literals `"graceful"`,
    `"signal"`, or `"forced"` (per BC-2.01.006 §Invariant 1).
    Verified by integration test asserting
    `["graceful", "signal", "forced"].contains(&value["shutdown_reason"].as_str().unwrap())`.
11. **Non-empty-string for `last_app_mode`:** `last_app_mode` MUST be a
    non-empty JSON string (e.g., `"Running"`, `"ShuttingDown"`,
    `"Crashed"`). Verified by integration test asserting
    `value["last_app_mode"].is_string() &&
    !value["last_app_mode"].as_str().unwrap().is_empty()`.

## Counter-examples

1. Recovery file written AFTER lock-file removal — on a hard SIGKILL
   between the two writes, no recovery file exists; the next daemon
   start has no recovery path. The test asserts the recovery file
   comes first (post-condition 8).
2. Recovery file has an extra key (e.g., `tui_attached: false`) — fails
   the exact-4-key schema assertion.
3. `shutdown_utc` format is `YYYY-MM-DD HH:MM:SS` (space separator
   instead of `T`) — fails the strict ISO 8601 regex.
4. 60-second window started from UDS-readiness rather than daemon start
   — under high startup latency the effective window would be
   stretched; the test asserts the start-time baseline by advancing
   the clock past `start_time + 60s` and asserting expiration.
5. TUI declines (`N`) but state is still transmitted — fails
   post-condition 4.
6. **Probe-matrix exhaustiveness regression (`pid` type drift):** `pid`
   serialized as a JSON string `"12345"` instead of an integer
   `12345` — fails Post-condition 9's `is_i64()` assertion. A regression
   class where an implementer changes `pid` from `u32` to `String`
   somewhere in the recovery checkpoint emitter would slip the exact-key
   schema check (the key name `pid` is unchanged) but fail this
   integer-type probe. `cargo-mutants` mutation-test rationale: this
   probe is the leverage point for the type-mutation surface on the
   serialization path.
7. **Probe-matrix exhaustiveness regression (`shutdown_reason` enum
   drift):** `shutdown_reason` set to a value outside the 3-element
   enum, e.g., `"weird"` or `"unknown"` — fails Post-condition 10's
   closed-set membership assertion. A regression class where an
   implementer adds a 4th `shutdown_reason` variant without updating the
   serialization path's allowed-values guard would slip the exact-key
   schema check but fail this enum-value-set probe. `cargo-mutants`
   mutation-test rationale: this probe is the leverage point for the
   enum-extension-without-guard mutation surface.
8. **Probe-matrix exhaustiveness regression (`last_app_mode` empty
   string):** `last_app_mode` emitted as an empty string `""` — fails
   Post-condition 11's non-empty assertion. A regression class where a
   `Default::default()` slip emits the empty-string default for an
   uninitialized `AppMode` would slip the exact-key schema check but
   fail this non-empty-string probe. `cargo-mutants` mutation-test
   rationale: this probe is the leverage point for the
   default-initialization mutation surface.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 6.a | Synthetic shutdown signal → drain completes | recovery file exists; 4-key schema; `shutdown_reason == "graceful"` |
| 6.b | Pre-created recovery file at start + absent/stale lock | WARN log `recovery checkpoint found...` |
| 6.c | Pre-created recovery file + mock TUI ACK (`Y`) within 60s | UDS `recovery_available` msg sent; recovery file deleted after ACK |
| 6.d | Pre-created recovery file + mock TUI DECLINE (`N`) within 60s | recovery file deleted; NO state-transmission payload sent |
| 6.e | Pre-created recovery file + no TUI; clock advanced 61s | recovery file deleted; WARN `recovery offer expired` log |
| 6.f | Pre-created malformed recovery file (truncated content) | WARN `recovery file malformed` log; file deleted; no UDS msg |
| 6.g | `shutdown_utc` regex assertion | matches `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` strictly |
| 6.h | Filesystem-op order trace via `tracing` | recovery file write event precedes lock-file remove event |
| 6.i | Typed-field probe: `pid is_i64() && >= 1` | invariant holds |
| 6.j | Typed-field probe: `shutdown_reason` in closed enum set | invariant holds (3 literals only) |
| 6.k | Typed-field probe: `last_app_mode` non-empty string | invariant holds |

## Harness Location

- `monocle-runtime/tests/crash_recovery.rs` (integration)
- Test name: `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup`
  (per PRD v1.25 §BC-DAEMON-006, Verification subsection — to be migrated to
  `test_BC_2_01_006_crash_recovery_checkpoint_offer_and_cleanup`).

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-DAEMON-006 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.006.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.006 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-002 (numeric-types-and-ranges probe parity on `pid`).
