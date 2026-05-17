---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.01.008
module: monocle-runtime
proof_method: manual+fuzz
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

# VP-008: Auth Token — Wire Format + Constant-Time Comparison (FC-06)

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-AUTH-001 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

The lock-file `authToken` field is a 64-character lowercase hex string
(`^[0-9a-f]{64}$`) sourced from `rand::rngs::OsRng` (NOT `thread_rng`). The
wire-format token presented in `X-Monocle-Authorization` is
`"monocle-v1:" ++ authToken` (74 characters total). `validate_auth_token`
returns `true` iff the prefix is `monocle-v1:` AND the post-prefix hex
equals `expected_secret` byte-for-byte under `constant_time_eq` comparison
(NOT `==` on `&str`/`String`). Defense-in-depth: the containing
`<runtime_dir>` is `0o700` (per VP-005) and the lock file itself is
`0o600`. Fuzz target asserts NO panic and NO `true` for any input differing
from the expected secret.

## Source Contract

- **BC (primary):** BC-2.01.008 — Auth Token Wire Format (FC-06).
- **BCs (partial coverage):** BC-2.01.005 (lock-file mode + runtime-dir
  defense-in-depth pairing).
- **Postcondition/Invariant:** 64-hex token format from `OsRng`;
  `monocle-v1:`-prefixed wire format; `constant_time_eq` comparison
  (source-grep negative assertion on raw `==`); defense-in-depth with
  `0o700` runtime dir + `0o600` lock file.
- **Traces to (historical):** BC-AUTH-001 (SS-daemon-lifecycle.md §Start
  Sequence).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — finite probe set | Lock-file token regex; wire-format round-trip; bit-flip rejection; missing-prefix rejection |
| Fuzz (auxiliary) | cargo-fuzz / libFuzzer | Bounded byte-sequence universe | Arbitrary bytes never produce `true` except the exact expected secret; no daemon panic |
| Source-grep (structural) | ripgrep | N/A — static | `constant_time_eq::constant_time_eq` used in `auth.rs`; no raw `==` on hex secret strings |

## Mechanism

Integration test (primary; harness at
`monocle-runtime/tests/auth_token_lifecycle.rs` — files in `<crate>/tests/`
are cargo integration tests; PRD v1.25 §7 RTM Test Type column labels this
BC `Integration`); fuzz (auxiliary). The harness asserts the lock-file
`authToken` regex, performs round-trip valid/invalid auth probes against
the test server, asserts bit-flip rejection (single-byte-flipped hex →
401), and runs the source-grep on `monocle-runtime/src/auth.rs`. The fuzz
target sweeps arbitrary `X-Monocle-Authorization` values and asserts
panic-free behavior with `true` only for the exact expected secret.

## Pre-conditions

- Daemon has completed start sequence and written `monocle.lock`.
- `constant_time_eq ^0.3` is the project pin (per SS-deps-pin-manifest.md
  v1.1.15).
- `rand::rngs::OsRng` is the entropy source (not `thread_rng`).

## Post-conditions

1. `lock.authToken` matches `^[0-9a-f]{64}$` (exact length 64, lowercase
   hex only).
2. Presenting `monocle-v1:<lock.authToken>` to `/status` returns HTTP 200.
3. Presenting `monocle-v1:<lock.authToken with one byte flipped>` returns
   HTTP 401.
4. Presenting `<lock.authToken>` WITHOUT the `monocle-v1:` prefix returns
   HTTP 401.
5. The auth middleware's secret comparison uses `constant_time_eq`; this is
   verified by source-grep against `monocle-runtime/src/auth.rs` ensuring no
   `==` on the hex secret string appears outside `constant_time_eq`.
6. **Cross-property with VP-005 Post-condition 9:** the auth token's
   containing `<runtime_dir>` is protected by `0o700` owner-only mode
   (defense-in-depth with this VP's auth-token in-band protections —
   `monocle-v1:`-prefixed wire format + `constant_time_eq` comparison +
   64-hex `OsRng` entropy — and out-of-band protections — `0o600`
   lock-file mode per VP-005 §Post-condition 1). The `0o700` runtime-dir
   mode is the outermost ring of the defense-in-depth layering
   protecting the `lock.authToken` field: even if the lock-file mode
   bits regress, the containing directory's owner-only mode prevents
   other OS users from stat-traversing or reading the file. Per VP-005
   §Post-condition 9 the `0o700` mode is asserted on the
   runtime-dir-creation path; this VP reciprocates the cross-property
   reference (Obs-R84-2 / SE-15d cross-property reciprocity closure).

## Counter-examples

1. Switch `constant_time_eq` to `String::eq` — would still pass functional
   tests but would lose the timing-oracle property; mitigated by the
   source-grep assertion in the harness.
2. Lock file written with `tempfile::persist` interrupted mid-write —
   partial token leaves a < 64-char hex; the regex match must reject.
3. Token generation via `rand::thread_rng()` instead of `OsRng` — passes
   the format regex but fails the entropy source check (verified by
   source-grep against `monocle-runtime/src/lock.rs`).
4. Adversary submits `monocle-v1:` + 64 chars of `0` (all-zero secret) —
   must be rejected because the real secret has 256 bits of entropy.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 8.a | Read `lock.authToken` from generated lock file | matches `^[0-9a-f]{64}$` |
| 8.b | Present `monocle-v1:<lock.authToken>` to `/status` | HTTP 200 |
| 8.c | Present `monocle-v1:<bit-flipped 64-hex>` to `/status` | HTTP 401 |
| 8.d | Present `<lock.authToken>` (no prefix) to `/status` | HTTP 401 |
| 8.e | Source-grep `monocle-runtime/src/auth.rs` for `constant_time_eq` | present; raw `==` on hex secret absent |
| 8.f | Source-grep `monocle-runtime/src/lock.rs` for `OsRng` | present; `thread_rng` absent in token-gen path |
| 8.g | Cross-property: containing `<runtime_dir>` mode | `0o700` (assertion in VP-005 5.e) |
| 8.h | Fuzz: arbitrary `X-Monocle-Authorization` bytes | no panic; `true` only for exact expected secret |

**Fuzz harness:** `cargo fuzz add fuzz_auth_token_validation`. The fuzz
target constructs arbitrary byte sequences as the
`X-Monocle-Authorization` value and runs `validate_auth_token(input,
expected)` against a fixed 64-char hex secret. The fuzzer should never
produce an input that returns `true` other than the exact expected secret
with the `monocle-v1:` prefix. The target asserts NO panic and NO `true`
return for any input differing from the expected secret.

## Harness Location

- `monocle-runtime/tests/auth_token_lifecycle.rs` (integration)
- `fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz)
- Test name: `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`
  (per PRD v1.25 §BC-AUTH-001, Verification subsection — to be migrated to
  `test_BC_2_01_008_lockfile_token_format_and_auth_round_trip`).

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-AUTH-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.008.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 §Start
  Sequence (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.008 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-005 §Post-condition 9 (runtime-dir `0o700`
  defense-in-depth pairing); VP-009 (auth header validation taxonomy).
