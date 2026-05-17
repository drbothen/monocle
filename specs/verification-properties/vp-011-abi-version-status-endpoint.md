---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.02.001
module: monocle-runtime
proof_method: integration-test
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

# VP-011: ABI Version in `/status` Endpoint

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-ABI-001 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

A `GET /status` request with a valid `X-Monocle-Authorization` header returns
HTTP 200 with a JSON body whose top-level `abi_version` key has the integer
value `1` (equal to `monocle_core::MONOCLE_ABI_VERSION` as compiled into the
daemon binary).

## Source Contract

- **BC:** BC-2.02.001 — ABI Version (`/status` Endpoint Response).
- **Postcondition/Invariant:** BC-2.02.001 Postcondition asserting
  `abi_version: 1` integer in `/status` JSON body; cross-property with
  VP-012 §Post-condition 1 (compile-time `const _: () =
  assert!(MONOCLE_ABI_VERSION == 1)` in the binary crate ensures drift
  between binary and constant is impossible).
- **Traces to (historical):** BC-ABI-001 (SS-core-types-and-abi.md
  §ABI Version Constant; PRD v1.25 §BC-ABI-001 Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — finite probe set | Status-code + body-shape across authenticated probe; integer-equality assertion on `abi_version` field |

## Mechanism

Integration test (harness at `monocle-runtime/tests/status_abi_version.rs`
— files in `<crate>/tests/` are cargo integration tests per Rust
convention; PRD v1.25 §7 RTM Test Type column labels this BC
`Integration`). The harness constructs an axum test server, makes an
authenticated `GET /status` request, parses the JSON body, and asserts
`body["abi_version"] == 1` (integer comparison, not string).

## Pre-conditions

- Daemon running with a valid lock file.
- Authenticated client holds the lock-file secret.
- `axum 0.8` is the project pin (per SS-deps-pin-manifest.md v1.1.15).

## Post-conditions

1. HTTP 200 status code on authenticated `GET /status`.
2. Response body parsed as JSON has key `abi_version` with integer value `1`.
3. The value `1` equals `monocle_core::MONOCLE_ABI_VERSION` at compile time
   (compile-time `const _: () = assert!(MONOCLE_ABI_VERSION == 1)` in the
   binary crate ensures drift between binary and constant is impossible;
   cross-property with VP-012).

## Counter-examples

1. `/status` handler hardcodes `"abi_version": 2` — must fail the literal
   integer comparison.
2. `MONOCLE_ABI_VERSION` raised to `2` without updating the status handler —
   the compile-time assert catches drift; without the assert, the
   integration test would still catch the runtime mismatch.

## Probe Matrix

| Probe | Setup | Expected status | Expected body shape |
|-------|-------|-----------------|---------------------|
| 11.a | Authenticated `GET /status` | 200 | JSON contains `abi_version: 1` (integer) |
| 11.b | Drift simulation: handler returns `abi_version: 2` | 200 | Test FAILS integer-equality assertion |
| 11.c | Drift simulation: `MONOCLE_ABI_VERSION` raised to `2` without handler update | compile error | Compile-time `const _:` assert traps the drift |

## Harness Location

- `monocle-runtime/tests/status_abi_version.rs` (integration test)
- Test name: `test_BC_ABI_001_status_endpoint_returns_abi_version_1` (per
  PRD v1.25 §BC-ABI-001, Verification subsection — to be migrated to
  `test_BC_2_02_001_status_endpoint_returns_abi_version_1` post BC
  renumber propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-ABI-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.001.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §ABI Version Constant.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.001 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-VP: VP-012 (compile-time const equals `1`).
