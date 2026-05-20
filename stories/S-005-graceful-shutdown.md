---
document_type: story
level: L4
story_id: S-005
epic_id: EPIC-01
version: "1.6"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001, S-002, S-003, S-006]
blocks: []
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.004]
verification_properties: [VP-004]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.004.md, version: "1.0.4"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-004-graceful-shutdown.md, version: "1.0.14"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md, version: "1.0.2"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.01.004 (Graceful Shutdown); verifies VP-004; addresses E-DAEMON-002."
---

# S-005: Graceful Shutdown (10-Second Drain)

## Narrative

As a daemon operator, I want monocle to handle `SIGTERM`/`SIGINT` signals and the
`POST /shutdown` endpoint by draining in-flight requests for up to 10 seconds before
exiting, so that hook events are not lost and harness subprocesses do not receive
broken-pipe errors mid-operation.

## Acceptance Criteria

### AC-001 (traces to BC-2.01.004 postcondition 1 — SIGTERM drain)
When the daemon receives `SIGTERM`, it:
1. Sets `AppMode` to `ShuttingDown`
2. Stops accepting new connections
3. Drains in-flight requests for up to 10 seconds
4. Exits with code 0 if drain completes within 10 seconds

Verified per VP-004 PC-5: `elapsed < 11 seconds AND exit_code == 0` (deterministic single-code
assertion; harness probe 4.e: in-flight 5-second sleep + clean drain).

### AC-002 (traces to BC-2.01.004 postcondition 1 + invariant 3 — POST /shutdown AppMode transition + dual-accept auth)
`POST /shutdown` with a valid auth header (canonical `X-Monocle-Authorization: monocle-v1:<64-hex>`
OR alias `X-Claude-Code-Ide-Authorization: <raw-64-hex>` per ADR-0005 dual-accept protocol)
returns HTTP 200 with body `{"status":"shutting_down"}` and immediately transitions AppMode to
`ShuttingDown` — the same state transition as a SIGTERM signal per BC-2.01.004 PC-1.
INV-3: unauthenticated `POST /shutdown` (neither header present) returns HTTP 401
`{"error":"missing_auth_token"}`; value-present auth failures return HTTP 401
`{"error":"invalid_auth_token"}` per BC-2.01.009 PC-1/PC-2/PC-3.

### AC-003 (traces to BC-2.01.004 postcondition 2 — hook 503 during shutdown)
Hook POST requests arriving after `ShuttingDown` is set return HTTP 503 with body
`{"error":"daemon_shutting_down"}` and header `Retry-After: 10`.

### AC-004 (traces to BC-2.01.004 postcondition 8 — 5-code POSIX exit taxonomy)
The daemon exits with exactly these exit codes, matching BC-2.01.004 PC-8 verbatim
(POSIX 128+N convention for signal-induced exits):
- `0` — graceful drain succeeded; all in-flight requests completed within the 10-second window; ring buffer flushed if applicable.
- `130` — hard-killed by SIGINT (signal 2) during drain — POSIX convention 128+2. Typical cause: user pressed Ctrl-C a second time while draining.
- `143` — hard-killed by SIGTERM (signal 15) during drain — POSIX convention 128+15. Typical cause: systemd/k8s sent a second SIGTERM after the graceful-shutdown window.
- `2` — hard-killed by a second authenticated `POST /shutdown` during drain (admin forced-stop). Monocle-specific programmatic code; chosen outside the POSIX 128+N space (which starts at 129) and distinct from startup-failure exit 1.
- `1` — daemon failed to start (startup failure — e.g., `DaemonStartError::RuntimeDirUnresolvable`, port bind failure, existing live lock file).

Exit code `3` (panic) and exit code `4` (SIGKILL) are NOT in the taxonomy. SIGKILL is
uncatchable — the OS reports a non-zero exit status but the daemon cannot set the exit
code on SIGKILL termination. Panics propagate via Rust's default panic hook and produce
an uncontrolled non-zero exit; see AC-005 for the panic logging invariant.

External monitoring systems (systemd `Restart=on-failure`, k8s `terminationGracePeriodSeconds`,
CI status parsers) MUST use exit code `143` (not `130`) to detect SIGTERM hard-kill during
drain — INV-4 per BC-2.01.004. Exit 130 encodes SIGINT (Ctrl-C second press), not SIGTERM.

Cite: BC-2.01.004 PC-8; BC-2.01.004 INV-4; SS-daemon-lifecycle.md §Hard Shutdown (step 6 + Exit codes list) and §BC-2.01.004 verification block.

### AC-005 (traces to BC-2.01.004 invariant 1 — hard-timeout drain budget exhaustion)
The 10-second drain window is a HARD timeout per BC-2.01.004 INV-1. If the drain has not
completed within 10 seconds, the daemon forces immediate shutdown regardless of remaining
in-flight requests. A second SIGTERM during drain also triggers immediate hard shutdown
without waiting for in-flight requests to complete.

Verified per VP-004 PC-6: forced exit on 10-second timeout; assert ring-buffer-flush log
line present in test output (over-budget scenario: in-flight 15-second sleep, no second
signal → drain timeout fires → `elapsed < 11 seconds` asserted).

### AC-006 (traces to BC-2.01.004 invariant 3 — dual-accept auth on /shutdown)
The shutdown endpoint requires authentication (canonical OR alias per ADR-0005) and
returns 401 if auth is missing or invalid. `POST /shutdown` with
`X-Claude-Code-Ide-Authorization: <raw-64-hex>` (alias) also initiates shutdown.
Auth middleware validates both headers per ADR-0005 v1.0.2 dual-accept protocol.

### AC-007 (traces to BC-2.01.004 postcondition 7 — lock file release on clean shutdown)
On clean shutdown completion, `lifecycle::exit_with(DaemonExit::Graceful)` invokes
`lock_file::release()` from S-006 (`monocle-runtime/src/lock.rs`) BEFORE
`std::process::exit(0)`. Verified via integration test asserting lock file is absent
on the filesystem after graceful shutdown completes (BC-2.01.004 PC-7 + BC-2.01.005 PC-6
lock+sock removal invariant).

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~800 |
| BC-2.01.004.md | ~700 |
| VP-004 file | ~500 |
| SS-daemon-lifecycle.md (shutdown section, ~80 lines) | ~1,200 |
| tokio signal + axum shutdown docs | ~400 |
| Test file | ~700 |
| **Total estimate** | **~4,300** |

## Tasks

- [ ] Add `POST /shutdown` route to authenticated router
- [ ] Implement `post_shutdown` handler in `monocle-runtime/src/handlers/shutdown.rs`
- [ ] Wire `tokio::signal::unix::signal(SignalKind::terminate())` for SIGTERM
- [ ] Wire `tokio::signal::ctrl_c()` for SIGINT
- [ ] Implement 10-second drain with `axum::serve(...).with_graceful_shutdown(signal)`
- [ ] Set AppMode to `ShuttingDown` on signal/shutdown endpoint trigger
- [ ] Hook handlers return 503 + Retry-After: 10 when AppMode is ShuttingDown
- [ ] Define `DaemonExit` enum with `to_exit_code(&self) -> i32` returning 0/1/2/130/143 per
  POSIX convention in `monocle-runtime/src/lifecycle.rs` (BC-2.01.004 PC-8); `lifecycle::exit_with(reason: DaemonExit) -> !`
  is the SOLE call-site for `std::process::exit` per SS-conventions-anti-patterns v1.29.5
  ('No `std::process::exit()` in handler code')
- [ ] Install tokio panic hook that logs structured panic info to stderr; propagates default Rust panic exit behavior (no custom exit code)
- [ ] Integration tests `monocle-runtime/tests/graceful_shutdown.rs`:
  - SIGTERM → drain → exit 0
  - POST /shutdown canonical auth → 200 + shutdown initiated
  - POST /shutdown alias auth → 200 + WARN log + shutdown initiated
  - Hook POST during shutdown → 503 + Retry-After: 10
  - POST /shutdown no auth → 401

## Implementation Notes

Panic-hook structured logging to stderr is implementation-recommended for diagnostic
visibility but is NOT a behavioral acceptance criterion in this story. No BC clause
mandates panic-hook installation; daemon process death from panic propagates Rust's
default panic exit behavior without setting a custom exit code.

## Previous Story Intelligence

S-002 (Wave 2): `AppMode` enum and `Arc<RwLock<AppMode>>` established in `state.rs`.
S-003 (Wave 2): Authenticated router + auth middleware established for `/status`.
`monocle-runtime/src/auth.rs` is created by S-003 with the canonical `X-Monocle-Authorization`
validation path (ADR-0005). S-005 reuses the auth middleware layer for the authenticated
`POST /shutdown` endpoint — no new auth code needed. The `/shutdown` endpoint requires canonical
OR alias auth per ADR-0005 dual-accept protocol (BC-2.01.004 INV-3).
S-006 (Wave 2): `DaemonLock` lifecycle established; S-005 invokes `DaemonLock::release()` from
`lifecycle::exit_with()` immediately before process termination per BC-2.01.004 PC-7
(lock+sock removed on clean shutdown). S-005 depends on S-006 for the `lock_file::release()`
call in the graceful-shutdown code path.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.33 §Graceful Shutdown, §Drain (10-Second Timeout), and §Hard Shutdown:
- 10-second drain timeout is hard-coded (not configurable in Phase 1)
- `AppMode::ShuttingDown` gates the 503 response on hook handlers
- Drain timeout (10s per INV-1) triggers force-shutdown via in-process abort; drain-timeout-forced-shutdown exits 0 (SIGTERM originator, graceful attempt completed within deadline). Exit code 130 = second SIGINT during drain; exit code 143 = second SIGTERM during drain; exit code 2 = second authenticated `POST /shutdown` during drain (admin forced-stop, NOT drain timeout). BC-2.01.004 PC-8 + INV-1 + INV-4.

From `architecture/SS-conventions-anti-patterns.md` v1.29.5:
- Use `nix 0.30` for signal handling: `nix::sys::signal::kill` for pid-liveness; tokio signal API for SIGTERM/SIGINT
- No `std::process::exit()` in handler code — propagate through lifecycle module

**Forbidden Dependencies:**
- Signal handling MUST NOT use `libc::signal()` directly — use `tokio::signal` or `nix`

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| axum | =0.8.9 | serve().with_graceful_shutdown(), StatusCode |
| tokio | =1.52 | signal handling, timeout |
| nix | 0.30 | POSIX signal types (SignalKind) |
| tracing | 0.1 | panic hook logging |

## File Structure Requirements

Files to create:
- `monocle-runtime/src/handlers/shutdown.rs` — `post_shutdown` handler
- `monocle-runtime/src/lifecycle.rs` — daemon start/stop lifecycle, exit code definitions
- `monocle-runtime/tests/graceful_shutdown.rs` — integration tests

Files to modify:
- `monocle-runtime/src/handlers/mod.rs` — add `pub mod shutdown;`
- `monocle-runtime/src/router.rs` — add `POST /shutdown` route
- `monocle-runtime/src/state.rs` — `AppMode::ShuttingDown` variant gates 503

## §Trace v1.6

**Phase 3.B Batch 6 — residual NON-AUTH findings** (2026-05-20):
- F-E-02 (MED): S-006 added to `depends_on` ([S-001, S-002, S-003] → [S-001, S-002, S-003,
  S-006]). S-006 establishes `DaemonLock`; S-005 invokes `DaemonLock::release()` from
  `lifecycle::exit_with()` before process termination per BC-2.01.004 PC-7.
- AC-007 added: `lifecycle::exit_with(DaemonExit::Graceful)` invokes `lock_file::release()`
  from S-006 BEFORE `std::process::exit(0)`.
- §Previous Story Intelligence: S-006 paragraph added.
- F-C-01 + F-C-02 (LOW): VP-004 PC-5/PC-6 oracle assertions inlined in AC-001 and AC-005.
- F-D-02 (LOW): `DaemonExit` enum spec refined — `to_exit_code() -> i32`, sole call-site rule,
  SS-conventions-anti-patterns v1.29.5 citation.
- Cascade: dep-graph + STORY-INDEX updated (S-005 depends_on adds S-006).
- version bumped 1.5 → 1.6.

## §Trace v1.5

**Phase 3.A auth-ownership decision** (2026-05-20):
- S-005 F-E-01 (MED) closed: S-003 missing from `depends_on` corrected.
- depends_on: [S-001, S-002] → [S-001, S-002, S-003].
- inputs: added ADR-0005 v1.0.2.
- AC-004 cite: line-number citation replaced with section-name citation per dispatch spec.
- §Architecture Compliance Rules: section heading updated to include §Drain (10-Second Timeout).
- §Previous Story Intelligence: S-003 dependency context added — auth middleware in auth.rs
  created by S-003; S-005 reuses it for the authenticated /shutdown endpoint.
- version bumped 1.4 → 1.5.
