---
document_type: story
story_id: S-002
epic_id: EPIC-01
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 3
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001]
blocks: [S-003, S-005]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.001]
verification_properties: [VP-001]
estimated_days: 1
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.11"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.001.md, version: "1.0.4"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-001-healthz-endpoint.md, version: "1.0.14"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.10"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.32"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.01.001 (Healthz Endpoint); verifies VP-001; covers EC-040, EC-041; addresses E-DAEMON-003."
---

# S-002: Healthz Endpoint (Unauthenticated Liveness Probe)

## Narrative

As a TUI client or external health monitor, I want to probe `GET /healthz` on the monocle
daemon without an auth token, so that I can determine whether the daemon is alive or shutting
down even during auth-token rotation in crash recovery scenarios.

## Acceptance Criteria

### AC-001 (traces to BC-2.01.001 postcondition 1)
When the daemon AppMode is normal and the hook-receiver task is alive, `GET /healthz` returns
HTTP 200 with body `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}` where `uptime_sec`
is an integer seconds-since-start (not floating point) and `version` matches
`^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$` (SemVer 2.0; no leading `v`).

### AC-002 (traces to BC-2.01.001 postcondition 2)
When the daemon AppMode is `ShuttingDown` OR the hook-receiver task has exited abnormally,
`GET /healthz` returns HTTP 503 with body `{"status":"shutting_down"}`.

### AC-003 (traces to BC-2.01.001 postcondition 3)
`GET /healthz` with no `X-Monocle-Authorization` header returns HTTP 200 (not HTTP 401).
The endpoint is registered on the unauthenticated axum router.

### AC-004 (traces to BC-2.01.001 postcondition 4)
`GET /healthz` accepts no request body. `DefaultBodyLimit` is NOT applied to this endpoint.
Sending a body to `/healthz` does not cause a 413 response.

### AC-005 (traces to BC-2.01.001 invariant 2)
`/healthz` is NOT registered on the authenticated router. The auth middleware does NOT
intercept GET /healthz requests.

### AC-006 (traces to BC-2.01.001 edge case EC-040 — TUI hung-daemon detection)
Integration test documents: if `/healthz` is unreachable AND the lock file contains a live
PID (`kill(pid, 0)` returns Ok), the TUI flow initiates a recovery dialog.
(Phase 1 scope: verify daemon-side healthz response only; TUI recovery flow is Phase 3.)

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~700 |
| BC-2.01.001.md | ~600 |
| VP-001 file | ~500 |
| SS-daemon-lifecycle.md (health endpoints section, ~100 lines) | ~1,500 |
| axum router scaffolding from S-001 | ~800 |
| Test file skeleton | ~500 |
| **Total estimate** | **~4,600** |

Well within 20% context budget. No split required.

## Tasks

- [ ] Define `AppMode` enum in `monocle-runtime/src/state.rs`: variants `Running`, `ShuttingDown`
- [ ] Create `monocle-runtime/src/handlers/healthz.rs` with `get_healthz` axum handler
- [ ] Handler reads `AppMode` from shared `Arc<RwLock<AppMode>>` and start time `Instant`
- [ ] Build unauthenticated axum router in `monocle-runtime/src/router.rs` with `GET /healthz` route
- [ ] Serialize response as `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}`
- [ ] Read binary version from `env!("CARGO_PKG_VERSION")` for `version` field
- [ ] Add integration test `monocle-runtime/tests/healthz_endpoint.rs`
  - Test: normal state → 200 + alive body + uptime integer + semver version
  - Test: ShuttingDown state → 503 + shutting_down body
  - Test: request with no auth header → 200 (not 401)
  - Test: no body limit applied (send 1MB body → 200 not 413)
- [ ] Verify VP-001 probe: integration assertion on `GET /healthz | jq .status == "alive"`

## Previous Story Intelligence

S-001 (Wave 1): Cargo workspace is initialized. axum 0.8.9 pinned. `monocle-runtime/src/lib.rs` stub exists.
Use axum's `Router::new().route("/healthz", get(get_healthz))` pattern.
Do NOT use `DefaultBodyLimit::max()` on the unauthenticated router.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.32 §Health and Status Endpoints:
- `/healthz` is registered on the UNAUTHENTICATED router
- `DefaultBodyLimit` is applied to the authenticated router ONLY
- `AppMode` enum drives the 200/503 split — use `Arc<RwLock<AppMode>>`

From `architecture/SS-conventions-anti-patterns.md` v1.29.5:
- No `println!` in production code — use `tracing::info!` / `tracing::warn!`
- Error types use `thiserror 2`; handler returns `impl IntoResponse`

**Forbidden Dependencies:**
- `monocle-runtime` handler code MUST NOT import from `monocle-tui` (not a Phase 1 crate)
- `GET /healthz` MUST NOT import or use `constant_time_eq` (auth path only)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| axum | =0.8.9 | Router, handler, IntoResponse, StatusCode |
| tokio | =1.52 | async runtime |
| serde_json | =1.0.149 | JSON body serialization |
| tracing | 0.1 | structured logging |
| serde | 1 | derive Serialize for response structs |

## File Structure Requirements

Files to create:
- `monocle-runtime/src/state.rs` — `AppMode` enum, shared state types
- `monocle-runtime/src/handlers/healthz.rs` — `get_healthz` handler
- `monocle-runtime/src/handlers/mod.rs` — module declaration
- `monocle-runtime/src/router.rs` — router construction (unauthenticated + authenticated split)

Files to modify:
- `monocle-runtime/src/lib.rs` — add `pub mod handlers; pub mod router; pub mod state;`
- `monocle-runtime/tests/healthz_endpoint.rs` — integration test (create)
