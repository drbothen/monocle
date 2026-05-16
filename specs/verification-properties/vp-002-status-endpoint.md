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
source_bc: BC-2.01.002
module: monocle-runtime
proof_method: manual+proptest
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

# VP-002: Status Endpoint — Authenticated Daemon-State JSON with 10 Required Fields

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-002 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

`GET /status` is mounted on the authenticated router and, given a valid
`X-Monocle-Authorization: monocle-v1:<64-hex>` header, returns HTTP 200 with a
JSON body containing exactly 10 required fields: `pid`, `uptime_sec`,
`version`, `abi_version`, `lock_file`, `hook_endpoints`,
`ring_buffer_fill_pct`, `channel_saturation_pct`, `last_hook_ts`,
`tui_attached`. Each field has a precise type and range; `abi_version` equals
the compile-time `monocle_core::MONOCLE_ABI_VERSION` constant; the
`hook_endpoints` array is exactly the 5-element canonical hook-path set. The
endpoint continues to serve unmodified during graceful drain. Authentication
follows the BC-2.01.009 two-body taxonomy (missing → `missing_auth_token`;
malformed → `invalid_auth_token`).

## Source Contract

- **BC (primary):** BC-2.01.002 — Status Endpoint (Authenticated Daemon State).
- **BCs (partial coverage):** BC-2.01.003 (256 KiB body-limit cross-route
  inheritance), BC-2.01.009 (two-body auth taxonomy on `/status`),
  BC-2.01.004 (drain-state read-only invariant).
- **Postcondition/Invariant:** BC-2.01.002 Postcondition 1 (numeric/range
  probes — `pid ≥ 1`, `uptime_sec ≥ 0`, percentage 0.0..=100.0; semver
  regex on `version`; absolute-path on `lock_file`; boolean on
  `tui_attached`); exact 10-field set; `abi_version` equality with
  compile-time const; `hook_endpoints.len() == 5`.
- **Traces to (historical):** BC-DAEMON-002 (PRD v1.25 §BC-DAEMON-002;
  SS-daemon-lifecycle.md v1.0.25 §Health and Status Endpoints).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — finite probe set | Status-code, JSON field set, per-field type+range probes, cross-route auth taxonomy probes |
| Proptest (auxiliary) | proptest | Bounded property quantification | Property-based exhaustive enumeration of random valid and malformed auth header values; uniform 401 response |
| Const-assert (compile-time) | static_assertions / const _: () = assert!(...) | N/A — compile-time | `abi_version` literal equals `monocle_core::MONOCLE_ABI_VERSION` |

## Mechanism

Integration test (harness at `monocle-runtime/tests/status_endpoint_auth.rs` —
files in `<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test
Type column labels this BC `Integration`). The harness constructs an axum
test server, performs probes 1-7 from the Probe Matrix below, and asserts
the exact response code + body shape per probe. The `abi_version` integer
equality with `MONOCLE_ABI_VERSION` is double-guarded by a compile-time
`const_assert!` in the daemon binary crate.

## Pre-conditions

- Daemon running with a valid lock file.
- Test client reads the auth token from the lock file before issuing the
  request (the canonical client-side pattern per SS-daemon-lifecycle.md
  §Daemon Lifecycle Protocol §Start Sequence).
- `monocle_core::MONOCLE_ABI_VERSION` equals `1` at the time the daemon
  binary is compiled.
- `chrono 0.4` is the project pin (per SS-deps-pin-manifest.md v1.1.15) for
  the `last_hook_ts` ISO 8601 millisecond timestamp formatter. The daemon
  emits each per-hook-type timestamp via
  `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")` per
  SS-daemon-lifecycle.md §Trace v1.0.14 F-R72-1 rationale (cross-field
  uniformity with `startTimeUtc` and `shutdown_utc`); the regex assertion
  in §Post-conditions item 4 is exactly the format string this generator
  produces.

## Post-conditions

1. `GET /status` with valid header → HTTP 200; JSON body parses; field set
   equals exactly the 10 keys above; `abi_version == 1`;
   `hook_endpoints.len() == 5`.
2. `GET /status` with no header → HTTP 401 + `{"error":"missing_auth_token"}`.
3. `GET /status` with `monocle-v2:<hex>` → HTTP 401 + `{"error":"invalid_auth_token"}`.
4. `last_hook_ts` is a JSON object; each value is either an ISO 8601 string
   matching `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` or JSON `null`.
5. Request body exceeding 256 KiB returns HTTP 413 + `payload_too_large`
   body (cross-check VP-003 since `/status` is on the authenticated
   router and inherits the body-limit layer; reciprocates VP-003
   §Post-condition 4 — the `/status` cross-route 413 probe).
6. During daemon `ShuttingDown` AppMode (drain), `GET /status` continues
   to serve full daemon-state JSON unchanged (the `/status` read-only
   path is unaffected by drain; only POST endpoints return 503 per
   VP-004 Post-condition / Mechanical property item 4).
   Cross-property with VP-004 §Mechanical property item 4
   (drain-state read-only invariant).
7. **Numeric-type and range probes:** for the JSON response body of a
   successful `GET /status`:
   - `pid` is parsed as an integer and `pid >= 1` (PID values are
     positive non-zero in POSIX and Windows; PID 0 is reserved
     idle/system).
   - `uptime_sec` is parsed as an integer and `uptime_sec >= 0`
     (monotonic uptime cannot be negative).
   - `ring_buffer_fill_pct` is parsed as a floating-point number and
     satisfies `0.0 <= ring_buffer_fill_pct <= 100.0`.
   - `channel_saturation_pct` is parsed as a floating-point number and
     satisfies `0.0 <= channel_saturation_pct <= 100.0`.
   - `abi_version` is parsed as an integer (already covered by
     §Post-condition 1 for value equality; this probe asserts the JSON
     type kind is `Number` integer, not `String "1"` or boolean).
   The integration test parses the response body via
   `serde_json::from_str::<serde_json::Value>(&body)` and asserts each
   field's `is_i64()` / `is_f64()` discriminant plus the numeric range.
8. **String-format probes:** for the JSON response body:
   - `version` is parsed as a JSON string and matches the semver regex
     `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$` (the daemon
     binary's `CARGO_PKG_VERSION` per SS-daemon-lifecycle.md §Status
     endpoint contract).
   - `lock_file` is parsed as a JSON string and is an absolute path
     (asserted via `std::path::Path::new(&lock_file).is_absolute()`).
     On POSIX the path starts with `/`; on Windows the path matches
     a drive-letter prefix or UNC path per `Path::is_absolute` semantics.
   - `hook_endpoints[*]` each match the regex `^/hooks/[a-z-]+$`
     (extending §Post-condition 1 from a count assertion to a per-entry
     format assertion).
9. **Boolean-type probe:** the `tui_attached` field in the response body is
   parsed as a JSON boolean (`true` or `false`); the integration test
   asserts `parsed["tui_attached"].is_boolean()` and rejects any
   stringified `"true"`/`"false"` or integer `0`/`1` representation.

## Counter-examples

1. Response body has only 9 fields (one of the 10 dropped) — fails the
   exact-field-set assertion.
2. `abi_version` returned as `2` while the compiled `MONOCLE_ABI_VERSION`
   is `1` — fails the integer equality with the const.
3. `hook_endpoints` returns 4 paths (one missing) — fails the
   `.len() == 5` assertion.
4. `last_hook_ts` returns an empty string `""` instead of JSON `null` for
   hook types that have not fired — fails the null-or-iso8601 assertion.
5. Auth middleware accidentally returns `invalid_auth_token_format` body
   (the retired v1.0 taxonomy) for any case — fails because that body is
   no longer defined (the test asserts the exact two-body taxonomy from
   the post-2db408f BC-2.01.009 contract).
6. **(Numeric-type/range counter-example):** the implementer
   serializes `pid` as a JSON string (`"pid": "12345"` instead of
   `"pid": 12345`) or emits `uptime_sec` as a negative integer via
   `i64` underflow on a misbehaving monotonic clock backend — the
   §Post-condition 7 type-discriminant + range probe rejects both
   regressions. Mutation-test target: changing the `pid` field's
   `serde` type from `u32` to `String` is caught.
7. **(String-format counter-example):** the implementer
   forgets to render `lock_file` as an absolute path (e.g., emits
   `"lock_file": "monocle.lock"` because the canonicalization step
   was skipped) or emits `version` as `"v1.2.3"` with a leading `v`
   prefix violating semver — the §Post-condition 8 regex + absolute-path
   probes reject both. Mutation-test target: stripping
   `lock_file.canonicalize()` from the `/status` handler is caught.
8. **(Boolean-type counter-example):** the implementer emits
   `tui_attached` as the integer `1` (e.g., via a JS-style truthy
   coercion or a Python-style `bool` serialization mistake on a future
   non-Rust harness) or as the string `"true"` — the §Post-condition 9
   `is_boolean()` discriminant rejects both. Mutation-test target:
   changing the `tui_attached` field's Rust type from `bool` to `u8`
   in the `DaemonStatusResponse` struct is caught.

## Probe Matrix

| Probe | Setup | Expected status | Expected outcome |
|-------|-------|-----------------|------------------|
| 2.a | Valid `monocle-v1:<64-hex>` auth header; normal AppMode | 200 | 10-key JSON body; `abi_version == 1`; `hook_endpoints.len() == 5` |
| 2.b | No auth header | 401 | `{"error":"missing_auth_token"}` |
| 2.c | `monocle-v2:<hex>` (wrong prefix version) | 401 | `{"error":"invalid_auth_token"}` |
| 2.d | Numeric-type probes (pid, uptime_sec, percentages, abi_version) | 200 | per-field type-discriminant + range invariants hold |
| 2.e | String-format probes (version semver, lock_file absolute, hook_endpoints regex) | 200 | per-field regex/path-discriminant invariants hold |
| 2.f | Boolean-type probe (tui_attached) | 200 | `is_boolean()` discriminant holds |
| 2.g | `last_hook_ts` per-entry ISO 8601 ms regex | 200 | `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` or `null` |
| 2.h | Request body > 256 KiB on `/status` | 413 | `payload_too_large` body (cross-route limit inheritance; see VP-003) |
| 2.i | AppMode = `ShuttingDown`; valid auth | 200 | full 10-field body unchanged (read-only path during drain) |

## Harness Location

- `monocle-runtime/tests/status_endpoint_auth.rs` (integration test)
- Test name: `test_BC_DAEMON_002_status_endpoint_requires_auth_and_returns_abi_version`
  (per PRD v1.25 §BC-DAEMON-002, Verification subsection — to be migrated to
  `test_BC_2_01_002_status_endpoint_requires_auth_and_returns_abi_version`
  post BC renumber propagation into source).

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-DAEMON-002 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.002.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.002 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-003 (body limit), VP-004 (drain-state read-only),
  VP-009 (two-body auth taxonomy).
