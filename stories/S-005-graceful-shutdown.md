---
document_type: story
story_id: S-005
epic_id: EPIC-01
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001, S-002]
blocks: [S-007]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.004]
verification_properties: [VP-004]
estimated_days: 2
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

### AC-002 (traces to BC-2.01.004 postcondition 2 — POST /shutdown endpoint)
`POST /shutdown` with a valid auth header returns HTTP 200 and initiates the same
graceful shutdown sequence as SIGTERM. Response body: `{"status":"shutting_down"}`.

### AC-003 (traces to BC-2.01.004 postcondition 3 — hook 503 during shutdown)
Hook POST requests arriving after `ShuttingDown` is set return HTTP 503 with body
`{"error":"daemon_shutting_down"}` and header `Retry-After: 10`.

### AC-004 (traces to BC-2.01.004 postcondition 4 — 5-code POSIX exit taxonomy)
The daemon exits with the following exit codes:
- 0 — clean shutdown (drain completed within 10 seconds)
- 1 — startup failure (lock file conflict, runtime dir unresolvable)
- 2 — drain timeout (10 seconds elapsed; forced exit)
- 3 — unexpected panic (tokio panic hook captures and logs before exit)
- 4 — SIGKILL received (uncatchable; OS reports non-zero but daemon cannot control)

### AC-005 (traces to BC-2.01.004 invariant 1 — dual-accept auth on /shutdown)
`POST /shutdown` with `X-Claude-Code-Ide-Authorization: <raw-64-hex>` (alias) also
initiates shutdown. Auth middleware validates both headers per ADR-0005.

### AC-006 (traces to BC-2.01.004 invariant 3 — dual-accept auth header)
The shutdown endpoint requires authentication (canonical OR alias per ADR-0005) and
returns 401 if auth is missing or invalid.

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
- [ ] Define exit codes 0–3 in `monocle-runtime/src/lifecycle.rs`
- [ ] Install tokio panic hook that logs before exit 3
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
- Exit code 2 = drain timeout; tokio `axum::serve` timeout enforced via `tokio::time::timeout`

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
