---
document_type: story
story_id: S-003
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
blocks: []
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.002]
verification_properties: [VP-002, VP-011]
estimated_days: 2
---

# S-003: Status Endpoint (Authenticated Daemon State)

## Narrative

As a TUI client, I want to call `GET /status` with an auth token and receive a complete
JSON snapshot of the daemon's runtime state, so that I can render the runtime plane with
accurate session, ring-buffer, and ABI information.

## Acceptance Criteria

### AC-001 (traces to BC-2.01.002 postcondition 1)
`GET /status` with a valid auth header returns HTTP 200 with a JSON body containing all
10 required fields: `pid`, `uptime_sec`, `version`, `abi_version`, `lock_file`, `hook_endpoints`
(array of 5 paths), `ring_buffer_fill_pct` (float 0.0–100.0), `channel_saturation_pct`
(float 0.0–100.0), `last_hook_ts` (object with 5 nullable timestamp fields), `tui_attached` (bool).

### AC-002 (traces to BC-2.01.002 postcondition 2 — dual-accept auth per ADR-0005)
`GET /status` with `X-Claude-Code-Ide-Authorization: <raw-64-hex>` (alias path) returns
HTTP 200 with the same body as canonical auth. A WARN log `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias)...` is emitted.

### AC-003 (traces to BC-2.01.002 postcondition 3)
`GET /status` with no auth header returns HTTP 401 with body `{"error":"missing_auth_token"}`.

### AC-004 (traces to BC-2.01.002 postcondition 4)
`GET /status` with an invalid auth token returns HTTP 401 with body `{"error":"invalid_auth_token"}`.

### AC-005 (traces to BC-2.01.002 postcondition 5 — ABI version field)
The `abi_version` field in the `/status` response equals `monocle_core::MONOCLE_ABI_VERSION`
(value `1`) as compiled into the binary. Integration test asserts `jq .abi_version == 1`.
(Covers VP-011.)

### AC-006 (traces to BC-2.01.002 postcondition 6 — hook_endpoints array)
The `hook_endpoints` field is an array of exactly 5 paths:
`["/hooks/pre-tool-use", "/hooks/notification", "/hooks/stop", "/hooks/session-start", "/hooks/prompt-submit"]`.

### AC-007 (traces to BC-2.01.002 postcondition 7 — last_hook_ts format)
`last_hook_ts` values use ISO 8601 UTC with mandatory millisecond precision
(`YYYY-MM-DDTHH:MM:SS.sssZ`). A hook type that has not fired since daemon start has
`null` (not the string `"null"` and not `0`).

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~900 |
| BC-2.01.002.md | ~700 |
| VP-002 + VP-011 files | ~900 |
| SS-daemon-lifecycle.md (status section + auth middleware, ~150 lines) | ~2,200 |
| Auth middleware from S-009 (interface reference only) | ~300 |
| BC-2.02.001 (ABI version in /status) | ~500 |
| Test file | ~800 |
| **Total estimate** | **~6,300** |

## Tasks

- [ ] Create `monocle-runtime/src/handlers/status.rs` with `get_status` axum handler
- [ ] Define `StatusResponse` struct with all 10 fields, derive `serde::Serialize`
- [ ] `last_hook_ts` fields: use `Option<String>` serialized as ISO 8601 UTC via `chrono`
- [ ] Route `GET /status` on the AUTHENTICATED router (behind auth middleware)
- [ ] Auth middleware reads token from canonical `X-Monocle-Authorization` header first,
  then falls back to `X-Claude-Code-Ide-Authorization` alias (ADR-0005); emits WARN on alias path
- [ ] Add integration tests `monocle-runtime/tests/status_endpoint.rs`:
  - Valid canonical auth → 200 + all 10 fields present
  - Valid alias auth → 200 + WARN log emitted
  - No auth header → 401 `{"error":"missing_auth_token"}`
  - Wrong token → 401 `{"error":"invalid_auth_token"}`
  - `abi_version` field == 1 (VP-011 probe)
  - `hook_endpoints` array == exactly 5 paths in spec order

## Previous Story Intelligence

S-002 (Wave 2): Unauthenticated router established with `GET /healthz`.
This story adds the authenticated router with auth middleware.
The auth middleware is shared with hook endpoints (S-009 will depend on it).
Build auth middleware as a tower `Layer` or axum `middleware::from_fn`.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.32 §Status Endpoint:
- `last_hook_ts` uses `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")` — mandatory millisecond precision
- `abi_version` reads from `monocle_core::MONOCLE_ABI_VERSION` (const import)
- Auth middleware applies `DefaultBodyLimit::max(262144)` to the authenticated router only

From `architecture/SS-conventions-anti-patterns.md` v1.29.5:
- Constant-time comparison for auth: `constant_time_eq::constant_time_eq(a, b)` — NEVER `==` on secret strings
- Both canonical and alias paths MUST use constant-time comparison (NFR-010)

**Forbidden Dependencies:**
- `GET /status` handler MUST NOT have `monocle-tui` import
- Auth middleware MUST NOT use `std::cmp::PartialEq` directly on token bytes

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| axum | =0.8.9 | Router, middleware, StatusCode, TypedHeader |
| constant_time_eq | 0.3 | Token comparison on both canonical + alias paths (NFR-010) |
| chrono | 0.4 | ISO 8601 timestamp formatting for last_hook_ts |
| serde_json | =1.0.149 | JSON response body |
| tracing | 0.1 | WARN log on alias auth path |

## File Structure Requirements

Files to create:
- `monocle-runtime/src/handlers/status.rs` — `get_status` handler, `StatusResponse` struct
- `monocle-runtime/src/auth.rs` — auth middleware (tower Layer), token validation logic
- `monocle-runtime/tests/status_endpoint.rs` — integration tests

Files to modify:
- `monocle-runtime/src/handlers/mod.rs` — add `pub mod status;`
- `monocle-runtime/src/router.rs` — add authenticated router with auth middleware + body limit
- `monocle-runtime/src/lib.rs` — add `pub mod auth;`
