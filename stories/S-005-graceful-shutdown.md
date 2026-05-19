---
document_type: story
story_id: S-005
epic_id: EPIC-01
version: "1.3"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001, S-002]
blocks: []
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.004]
verification_properties: [VP-004]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.11"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.004.md, version: "1.0.3"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-004-graceful-shutdown.md, version: "1.0.14"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.10"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.32"}
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

Cite: BC-2.01.004 PC-8; BC-2.01.004 INV-4; SS-daemon-lifecycle.md line 795 + lines 2117-2132.

### AC-005 (traces to BC-2.01.004 invariant 1 — hard-timeout drain budget exhaustion)
The 10-second drain window is a HARD timeout per BC-2.01.004 INV-1. If the drain has not
completed within 10 seconds, the daemon forces immediate shutdown regardless of remaining
in-flight requests. A second SIGTERM during drain also triggers immediate hard shutdown
without waiting for in-flight requests to complete. The panic hook logs structured panic
info to stderr and then propagates Rust's default panic exit behavior — no custom exit
code is assigned.

### AC-006 (traces to BC-2.01.004 invariant 3 — dual-accept auth on /shutdown)
The shutdown endpoint requires authentication (canonical OR alias per ADR-0005) and
returns 401 if auth is missing or invalid. `POST /shutdown` with
`X-Claude-Code-Ide-Authorization: <raw-64-hex>` (alias) also initiates shutdown.
Auth middleware validates both headers per ADR-0005 v1.0.2 dual-accept protocol.

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
- [ ] Define exit codes 0, 1, 2, 130, 143 in `monocle-runtime/src/lifecycle.rs` (BC-2.01.004 PC-8)
- [ ] Install tokio panic hook that logs structured panic info to stderr; propagates default Rust panic exit behavior (no custom exit code)
- [ ] Integration tests `monocle-runtime/tests/graceful_shutdown.rs`:
  - SIGTERM → drain → exit 0
  - POST /shutdown canonical auth → 200 + shutdown initiated
  - POST /shutdown alias auth → 200 + WARN log + shutdown initiated
  - Hook POST during shutdown → 503 + Retry-After: 10
  - POST /shutdown no auth → 401

## Previous Story Intelligence

S-002 (Wave 2): `AppMode` enum and `Arc<RwLock<AppMode>>` established in `state.rs`.
S-003 (Wave 2): Authenticated router with auth middleware established.
Reuse the auth middleware from S-003 for `POST /shutdown` — no new auth code needed.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.32 §Graceful Shutdown and §Hard Shutdown:
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
