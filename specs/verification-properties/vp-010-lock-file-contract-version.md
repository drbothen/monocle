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
source_bc: BC-2.01.010
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

# VP-010: Lock File `contract_version: 1` First Key

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-LOCK-001 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

The JSON content written to `<runtime_dir>/monocle.lock` is structurally a
JSON object whose first key (per `serde_json` declaration-order
serialization) is `contract_version` with integer value `1`. Any
lock-file reader (e.g., TUI client) MUST inspect `contract_version` before
consuming other fields; on mismatch the reader returns
`Err(LockFileError::UnsupportedContractVersion(N))`; on absence the reader
returns `Err(LockFileError::MissingContractVersion)`. Neither error path
panics. The first-key invariant is the consumer-side reciprocal of
VP-005's producer-side joint mode-and-content assertion (`0o600` + JSON
prefix).

## Source Contract

- **BC (primary):** BC-2.01.010 — Lock File Contract Version Field.
- **BCs (partial coverage):** BC-2.01.005 (producer-side joint
  mode-and-content assertion).
- **Postcondition/Invariant:** declaration-order serialization with
  `contract_version` first; round-trip parse via `LockFile` struct
  preserves `contract_version == 1`; version-mismatch returns a typed
  error (not a panic); absent-`contract_version` returns
  `MissingContractVersion`; cross-property reciprocation with VP-005
  §Post-condition 1.
- **Traces to (historical):** BC-LOCK-001 (SS-daemon-lifecycle.md §Start
  Sequence).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test | Bounded — finite probe set | First-key prefix; round-trip; v2-mismatch error variant; missing-field error variant |
| Mutation test (auxiliary) | cargo-mutants | N/A — mutation surface | `contract_version: u32 = 1` value mutation; reader-side gate condition |

## Mechanism

Integration test (primary; harness at
`monocle-runtime/tests/lock_file_contract.rs` — files in `<crate>/tests/`
are cargo integration tests; PRD v1.25 §7 RTM Test Type column labels this
BC `Integration`); mutation-test (auxiliary). The harness asserts the
written file's literal prefix, performs round-trip via
`serde_json::from_str::<LockFile>`, constructs synthetic `v2` and
`absent-version` files, and asserts the typed-error returns (NOT panics).
`cargo-mutants` is configured to mutate the writer's `contract_version`
literal `1` to `0` and to mutate the reader's gate-condition; both
mutations must be caught.

## Pre-conditions

- Daemon start sequence completes; lock file is written via
  `tempfile::persist`.
- Lock-file reader code is
  `pub fn read_lock_file(path: &Path) -> Result<LockFile, LockFileError>`.
- `chrono 0.4` is the project pin (per SS-deps-pin-manifest.md v1.1.15)
  for the lock-file `startTimeUtc` ISO 8601 millisecond timestamp
  formatter. The daemon emits `startTimeUtc` via
  `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")` per
  SS-daemon-lifecycle.md §Start Sequence step 6 and §Trace v1.0.14
  F-R72-1 rationale (cross-field uniformity with `last_hook_ts` and
  `shutdown_utc`). Although VP-010 primarily asserts the
  `contract_version` first-key invariant, the `startTimeUtc` field is
  one of the lock file's normative fields and the round-trip read-back
  (post-condition 2 via `serde_json::from_str::<LockFile>`) requires the
  deserialization path to accept the chrono-emitted format.

## Post-conditions

1. `std::fs::read_to_string(&lock_path).unwrap().starts_with("{\"contract_version\":1,")`.
2. `serde_json::from_str::<LockFile>(&content).unwrap().contract_version == 1`.
3. With a synthetic lock file where `contract_version = 2`, the reader logs
   a warning and returns
   `Err(LockFileError::UnsupportedContractVersion(2))` — NOT a panic and
   NOT a silent acceptance of unknown fields.
4. With a synthetic lock file where `contract_version` is absent entirely,
   the reader returns `Err(LockFileError::MissingContractVersion)`.
5. **Cross-property with VP-005 §Post-condition 1** (lock-file 0o600 mode
   + `contract_version` JSON content): VP-005 asserts the lock-file file
   mode is `0o600` AND the JSON content begins with
   `{"contract_version":1,`; this VP asserts the same `contract_version`
   first-key invariant from the consumer side (reader-facing schema
   assertion) and is consumed by VP-005's mode-and-content joint
   post-condition.

## Counter-examples

1. Lock file written with `serde_json::to_value(&lock).to_string()` (which
   alphabetizes) — would place `app` before `contract_version`; the prefix
   assertion must fail.
2. Reader implements `serde_json::from_str` without an explicit
   `contract_version` check before field access — a future v2 lock file
   would be silently misparsed; the integration test must construct a
   synthetic v2 file and assert the version-gate error.
3. Lock-file writer omits `contract_version` (regression) — readers MUST
   reject; covered by post-condition 4.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 10.a | Daemon start writes lock file | `read_to_string` starts with `{"contract_version":1,` |
| 10.b | `serde_json::from_str::<LockFile>(&content)` | `.contract_version == 1` |
| 10.c | Synthetic `contract_version: 2` lock file | `Err(LockFileError::UnsupportedContractVersion(2))`; no panic |
| 10.d | Synthetic absent-`contract_version` lock file | `Err(LockFileError::MissingContractVersion)`; no panic |
| 10.e | Mutation: writer emits `contract_version = 0` | integration test 10.a fails |
| 10.f | Mutation: reader skips gate check | integration test 10.c or 10.d slips silently — caught by `cargo-mutants` |

**Mutation-test rationale:** the `contract_version` integer value `1` is a
prime mutation target. `cargo-mutants` will attempt to mutate the writer
to `contract_version = 0` and the reader's gate condition; both must be
caught by the integration test.

## Harness Location

- `monocle-runtime/tests/lock_file_contract.rs` (integration)
- Test name: `test_BC_LOCK_001_contract_version_first_key` (per PRD v1.25
  §BC-LOCK-001, Verification subsection — to be migrated to
  `test_BC_2_01_010_contract_version_first_key`).

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-LOCK-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.010.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 §Start
  Sequence (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.010 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-005 §Post-condition 1 (producer-side joint
  mode-and-content assertion).
