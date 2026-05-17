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
source_bc: BC-2.01.004
module: monocle-runtime
proof_method: manual
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

# VP-004: Graceful Shutdown — 10-Second Drain + 5-Code POSIX Exit Taxonomy

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-004 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

On SIGTERM, SIGINT, or authenticated `POST /shutdown`, the daemon transitions
AppMode to `ShuttingDown` within < 10 ms. After transition, new POSTs to
`/hooks/*` return HTTP 503 with `Retry-After: 10` and body
`{"error":"daemon_shutting_down"}`; `/healthz` returns 503 with
`{"status":"shutting_down"}`; `/status` continues to serve normally. In-flight
requests drain bounded by `tokio::time::timeout(Duration::from_secs(10),
drain_inflight())` after which the daemon proceeds to lock-file removal and
exits with one of 5 deterministic POSIX-correct exit codes per trigger cause
(`0`, `130`, `143`, `2`, `1`). `POST /shutdown` without valid auth returns
HTTP 401.

## Source Contract

- **BC (primary):** BC-2.01.004 — Graceful Shutdown (10-Second Drain).
- **BCs (partial coverage):** BC-2.01.005 (lock-file removal step
  post-drain), BC-2.01.009 (auth taxonomy on `/shutdown` route).
- **Postcondition/Invariant:** AppMode transition latency bound,
  503-with-`Retry-After` invariant on `/hooks/*`, drain-completion
  bound, 5-code POSIX exit taxonomy
  (0/130/143/2/1 — Obs-R70-2 + F-R70-3 closure), auth-on-`/shutdown`
  cross-property.
- **Traces to (historical):** BC-DAEMON-004 (PRD v1.25 §BC-DAEMON-004;
  SS-daemon-lifecycle.md v1.0.25 §Shutdown Signal Handling, §Drain,
  §Hard Shutdown).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test + tokio synthetic-signal harness | Bounded — finite per-trigger probe set | AppMode transition latency, 503 + `Retry-After`, drain bound, exit-code matrix, auth-on-shutdown |
| Synthetic signal injection | tokio oneshot::channel + `trigger_signal` recording | Bounded | Per-trigger exit-code determinism (SIGTERM-vs-SIGINT distinguishability) |
| Cross-property assertion | manual review | N/A | VP-001 + VP-002 + VP-005 + VP-009 cross-property reciprocations |

## Mechanism

Integration test (harness at `monocle-runtime/tests/graceful_shutdown.rs`
and `monocle-runtime/tests/daemon_lifecycle.rs` — files in `<crate>/tests/`
are cargo integration tests; PRD v1.25 §7 RTM Test Type column labels this
BC `Integration`). The harness uses a test-only `oneshot::channel` to
inject synthetic shutdown signals tagged with `trigger_signal: SIGTERM | SIGINT
| PostShutdown` so the harness can distinguish the 130-vs-143-vs-2 exit code
paths without real OS-signal delivery. Drain completion is bounded by
asserting `elapsed < 11 seconds` for the over-budget scenario and
`exit_code == <expected-per-trigger>`.

## Pre-conditions

- Daemon running with a valid lock file.
- `tokio::signal::unix::signal(SignalKind::terminate())` is the SIGTERM
  receiver; `tokio::signal::ctrl_c()` is the SIGINT receiver. The
  signal type that triggered hard shutdown is recorded for exit-code
  selection (per arch v1.0.25 §Hard Shutdown step 6d).
- A test-only `oneshot::channel` is used to inject a synthetic shutdown
  signal (avoiding real OS signal delivery in integration tests).
  Test-harness wrappers inject SIGTERM-flavored and SIGINT-flavored
  synthetic signals to exercise the 130-vs-143 distinction without real
  OS-signal delivery.
- `axum 0.8` and `tokio 1` are the project pins (per
  SS-deps-pin-manifest.md v1.1.15); `tower` is a transitive dependency of
  `axum 0.8` (no direct workspace pin).

## Post-conditions

1. Synthetic shutdown signal injected → AppMode is `ShuttingDown` within
   10 ms (asserted via a `tokio::sync::watch` channel exposing the
   current mode).
2. POST `/hooks/pre-tool-use` after AppMode transition → HTTP 503 with
   header `Retry-After: 10` (exact integer value) and body
   `{"error":"daemon_shutting_down"}`.
3. `GET /healthz` during drain → HTTP 503 + `{"status":"shutting_down"}`.
4. `GET /status` with valid auth during drain → HTTP 200 + full 10-field
   body (read-only continues).
5. With one synthetic in-flight `/hooks/*` POST that holds a 5-second
   sleep, the drain completes within 10 seconds and the daemon exits
   cleanly with exit code `0` (deterministic; graceful drain success).
6. **5-code POSIX exit taxonomy probe matrix (per PRD v1.25 §BC-DAEMON-004
   canonical test vectors; Obs-R70-2 closure):**

   | Scenario | Synthetic input | Expected exit code |
   |----------|-----------------|--------------------|
   | Clean drain | All in-flight POSTs complete within 10s | `0` |
   | SIGINT hard-kill during drain | Second synthetic-SIGINT delivered during drain | `130` (POSIX 128+2) |
   | SIGTERM hard-kill during drain | Second synthetic-SIGTERM delivered during drain | `143` (POSIX 128+15) |
   | Admin forced-stop during drain | Second authenticated `POST /shutdown` during drain | `2` (monocle-specific) |
   | Startup failure | `DaemonStartError::RuntimeDirUnresolvable` (cross-property with VP-005 post-condition 5) | `1` |

   Each row is a deterministic single-code assertion. No tolerance range.
   The over-budget scenario (in-flight 15-second sleep with no second
   signal) reaches the 10-second drain timeout and exits `143`-or-`130`
   depending on which signal originally triggered drain — NOT a tolerance,
   but a per-cause deterministic outcome captured by the test-harness's
   recorded `trigger_signal` field. The harness asserts `elapsed < 11
   seconds` AND `exit_code == <expected-per-trigger>`.
7. `POST /shutdown` with no auth header → HTTP 401 +
   `{"error":"missing_auth_token"}` (VP-009 cross-property).

## Counter-examples

1. New hook POSTs during drain return HTTP 200 (drain logic not
   short-circuiting accepts) — fails post-condition 2.
2. `Retry-After` header omitted or set to a different value (e.g., `5`) —
   fails the exact-value assertion.
3. `/status` blocks during drain (returns no response or 503) — fails
   post-condition 4.
4. Drain timeout not enforced (in-flight 15-second sleep allowed to
   complete) — fails the 10-second bound; the test must assert
   `elapsed < 11 seconds` for the over-budget scenario.
5. `POST /shutdown` accepted without auth — fails post-condition 7
   (auth middleware must run on this route).
6. **Exit code 130 returned for a SIGTERM hard-kill scenario** — fails
   the POSIX 128+N convention (128+15 = 143 for SIGTERM, not 130).
   External monitoring (systemd `Restart=on-failure`, k8s
   `terminationGracePeriodSeconds`) would misinterpret the trigger.
   The test harness asserts `exit_code == 143` for the SIGTERM
   second-signal path and `exit_code == 130` for the SIGINT
   second-signal path; conflating the two — i.e., returning `130` for
   both — must be caught (Obs-R70-2 + F-R70-3 closure).
7. **Exit code 2 collides with startup failure** — if implementer sets
   the admin-forced-stop exit to `1` (overlapping with startup
   failure), monitoring systems cannot distinguish operator-initiated
   force-stop from daemon-start failure. The probe matrix asserts
   `exit_code == 2` for the second-`POST /shutdown` path and
   `exit_code == 1` for the `RuntimeDirUnresolvable` startup-failure
   path; identical codes for these two distinct triggers must fail.
8. **Single-binary exit-code (any non-zero accepted) regression** —
   the prior-burst v1.5.1 tolerance (exit code `0` OR `130` for the
   over-budget 15-second scenario) is RETIRED. A harness that accepts
   any non-zero exit code as "hard-killed pass" without distinguishing
   130 vs 143 vs 2 vs 1 fails the new per-cause deterministic-outcome
   assertion. This counter-example sketch is the formal recurrence
   guard against the over-budget BC-vs-VP drift Obs-R70-2 documented.

## Probe Matrix

| Probe | Setup | Expected status | Expected outcome |
|-------|-------|-----------------|------------------|
| 4.a | Synthetic SIGTERM injected | N/A | AppMode → `ShuttingDown` within 10 ms |
| 4.b | POST `/hooks/*` post-drain entry | 503 | `Retry-After: 10` header; body `{"error":"daemon_shutting_down"}` |
| 4.c | GET `/healthz` during drain | 503 | `{"status":"shutting_down"}` (cross VP-001) |
| 4.d | GET `/status` (valid auth) during drain | 200 | full 10-field body (cross VP-002) |
| 4.e | In-flight 5s sleep + clean drain | exit `0` | drain completes within 10s |
| 4.f | Second synthetic-SIGINT during drain | exit `130` | POSIX 128+2; `elapsed < 11s` |
| 4.g | Second synthetic-SIGTERM during drain | exit `143` | POSIX 128+15; `elapsed < 11s` |
| 4.h | Second authenticated `POST /shutdown` during drain | exit `2` | monocle-specific admin force-stop |
| 4.i | `DaemonStartError::RuntimeDirUnresolvable` start | exit `1` | startup-failure code (cross VP-005) |
| 4.j | `POST /shutdown` with no auth header | 401 | `{"error":"missing_auth_token"}` (cross VP-009) |

## Harness Location

- `monocle-runtime/tests/graceful_shutdown.rs` (primary HTTP 503 / `Retry-After`
  probes)
- `monocle-runtime/tests/daemon_lifecycle.rs` (exit-code 5-code POSIX taxonomy
  probes per PRD v1.25 §BC-DAEMON-004 canonical test vectors)
- Test names:
  - `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests`
  - `test_BC_DAEMON_004_exit_codes_posix_distinct`
  (per PRD v1.25 §BC-DAEMON-004, Verification subsection — to be migrated to
  `test_BC_2_01_004_*` post BC renumber propagation into source.)

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-DAEMON-004 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.004.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.004 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-001 (`/healthz` 503), VP-002 (`/status` drain-state
  read-only), VP-005 (lock-file removal post-drain + startup-failure exit
  `1`), VP-009 (auth-on-shutdown taxonomy).
