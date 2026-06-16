---
document_type: story
level: L4
story_id: S-003
epic_id: EPIC-01
version: "1.9"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001, S-002]
blocks: [S-004, S-005, S-009, S-010, S-018]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.002, BC-2.02.001]
verification_properties: [VP-002, VP-011]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.002.md, version: "1.0.6"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.001.md, version: "1.0.2"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.008.md, version: "1.0.7"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md, version: "1.0.7"}
  - {path: .factory/specs/architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md, version: "1.0.2"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-002-status-endpoint.md, version: "1.0.14"}
  - {path: .factory/specs/verification-properties/vp-011-abi-version-status-endpoint.md, version: "1.0.13"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.01.002 (Status Endpoint), BC-2.02.001 (ABI Version in /status); verifies VP-002, VP-011; addresses NFR-010 (constant-time comparison on /status auth path)."
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

### AC-002 (traces to BC-2.01.009 postcondition 3 — alias path auth + WARN log)
`GET /status` with `X-Claude-Code-Ide-Authorization: <raw-64-hex>` (alias path, no canonical header present)
returns HTTP 200 with the same body as canonical auth. A WARN log
`WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization`
is emitted (BC-2.01.009 INV-6 line 61 — exact canonical string, no ellipsis).
(BC-2.01.009 PC-3 governs alias-path behavior; BC-2.01.002 Precondition 2 delegates auth semantics to BC-2.01.009.)

### AC-003 (traces to BC-2.01.009 postcondition 1 — missing auth → 401 E-AUTH-001)
`GET /status` with no auth header (neither `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` present)
returns HTTP 401 with body `{"error":"missing_auth_token"}` (E-AUTH-001).
(BC-2.01.009 PC-1 is the canonical auth-failure locus; BC-2.01.002 PC-2 delegates to BC-2.01.009.)

### AC-004 (traces to BC-2.01.009 postcondition 2 — invalid token → 401 E-AUTH-002)
`GET /status` with an invalid auth token (format correct but token value wrong)
returns HTTP 401 with body `{"error":"invalid_auth_token"}` (E-AUTH-002).
(BC-2.01.009 PC-2 governs canonical-path wrong-value behavior; PC-3 covers alias-path value-present failure (distinct concern).)

### AC-005 (traces to BC-2.02.001 postcondition 1 — ABI version field; BC-2.01.002 postcondition 1 sub-bullet «abi_version»)
The `abi_version` field in the `/status` response equals `monocle_core::MONOCLE_ABI_VERSION`
(value `1`) as compiled into the binary. Integration test asserts `jq .abi_version == 1`.
(Covers VP-011. BC-2.02.001 PC-1 is the authoritative ABI-version-in-/status clause; BC-2.01.002 PC-1 sub-bullet `abi_version` enumerates it as one of the required 10 fields.)

### AC-006 (traces to BC-2.01.002 postcondition 1 sub-bullet «hook_endpoints»)
The `hook_endpoints` field is an array of exactly 5 paths:
`["/hooks/pre-tool-use", "/hooks/notification", "/hooks/stop", "/hooks/session-start", "/hooks/prompt-submit"]`.
(BC-2.01.002 PC-1 sub-bullet `hook_endpoints` is the canonical source; cross-cite BC-2.01.008 PC-4 which mandates these 5 endpoints on the authenticated router.)

### AC-007 (traces to BC-2.01.002 postcondition 1 sub-bullet «last_hook_ts»)
`last_hook_ts` values use ISO 8601 UTC with mandatory millisecond precision
(`YYYY-MM-DDTHH:MM:SS.sssZ`). A hook type that has not fired since daemon start has
`null` (not the string `"null"` and not `0`). (BC-2.01.002 PC-1 sub-bullet `last_hook_ts`
+ EC-044 define the null-or-ISO-ms contract.)

### AC-008 (traces to BC-2.01.002 postcondition 3 — /status serves during drain)
`/status` continues to serve HTTP 200 responses during the graceful shutdown drain window
(AppMode = ShuttingDown). It does NOT return 503 during drain. (BC-2.01.004 PC-4 cross-cites
BC-2.01.002 PC-3 as the source; read-only endpoint is exempt from the drain-503 rule.)
Test fixture note: drain harness imported from S-005 graceful-shutdown integration test
scaffolding (`monocle-runtime/tests/graceful_shutdown.rs`).

Note: AC-005 subsumes the AC-007b intent. BC-2.02.001 PC-1 + PC-2 are the canonical
source for ABI-version-in-/status. AC-005 above covers both the field presence (BC-2.02.001 PC-1)
and the compile-time equality requirement (BC-2.02.001 PC-2). S-010 provides the const;
S-003 exposes it in the response — joint coverage. VP-011 verifies the equality invariant.

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
  (field types per VP-002 probe matrix — reference citation; full type table below):

  | Field | Rust Type | Notes |
  |-------|-----------|-------|
  | `pid` | `u32` | daemon process PID |
  | `uptime_sec` | `u64` | seconds since daemon start |
  | `version` | `String` | semver string |
  | `abi_version` | `u32` | equals `monocle_core::MONOCLE_ABI_VERSION` |
  | `lock_file` | `String` | absolute path |
  | `hook_endpoints` | `Vec<String>` | exactly 5 canonical paths |
  | `ring_buffer_fill_pct` | `f64` | 0.0–100.0 |
  | `channel_saturation_pct` | `f64` | 0.0–100.0 |
  | `last_hook_ts` | `LastHookTs` (struct) | 5 nullable timestamp fields |
  | `tui_attached` | `bool` | TUI attachment state |
- [ ] `last_hook_ts` fields: use `Option<String>` serialized as ISO 8601 UTC via `chrono`
- [ ] Route `GET /status` on the AUTHENTICATED router (behind auth middleware)
- [ ] Add `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1);` in
  `monocle-runtime/src/main.rs` (or library root) — compile-time ABI drift guard per
  VP-011 §Mechanism + VP-002 §Proof Method (const_assert compile-time guard)
- [ ] Auth middleware reads token from canonical `X-Monocle-Authorization` header first,
  then falls back to `X-Claude-Code-Ide-Authorization` alias (ADR-0005); emits WARN on alias path
- [ ] Add integration tests `monocle-runtime/tests/status_endpoint_auth.rs` (BC-2.01.002
  authentication path verification per VP-002 §Mechanism):
  - Valid canonical auth → 200 + all 10 fields present
  - Valid alias auth → 200 + WARN log emitted
  - No auth header → 401 `{"error":"missing_auth_token"}`
  - Wrong token → 401 `{"error":"invalid_auth_token"}`
  - `hook_endpoints` array == exactly 5 paths in spec order
- [ ] Add integration tests `monocle-runtime/tests/status_abi_version.rs` (BC-ABI ABI-version
  verification per VP-011 §Harness Location):
  - `abi_version` field == 1 (VP-011 probe 11.a)
  - Drift guard: compile-time `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1);`
    (VP-011 PC-3; cross-property VP-012)

## Previous Story Intelligence

S-002 (Wave 2): Unauthenticated router established with `GET /healthz`. The monocle-runtime
workspace crate, `monocle-runtime/src/lib.rs` stub, clippy/audit/CI baseline, and MSRV 1.88
are all inherited from S-001 (Wave 1) via S-002.
This story adds the authenticated router with auth middleware.
The auth middleware is shared with hook endpoints (S-009 will depend on it).

**S-003 OWNS the creation of `monocle-runtime/src/auth.rs`** per the canonical
`X-Monocle-Authorization` validation path (ADR-0005). S-009 EXTENDS this file in Wave 3
with the dual-accept alias-path branch (`X-Claude-Code-Ide-Authorization`) and the 5
hook-route handlers.

Build auth middleware as a tower `Layer` or axum `middleware::from_fn`.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.33 §Status Endpoint:
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
- `monocle-runtime/tests/status_endpoint_auth.rs` — auth path integration tests (VP-002)
- `monocle-runtime/tests/status_abi_version.rs` — ABI version integration tests (VP-011)

Files to modify:
- `monocle-runtime/src/handlers/mod.rs` — add `pub mod status;`
- `monocle-runtime/src/server.rs` — add authenticated router with auth middleware + body limit
- `monocle-runtime/src/lib.rs` — add `pub mod auth;`

## §Trace v1.5

**Phase 3.A auth-ownership decision** (2026-05-20):
- S-003 F-E-03 (LOW-MED) closed: auth.rs ownership collision resolved.
- S-003 OWNS creation of `monocle-runtime/src/auth.rs` per canonical `X-Monocle-Authorization`
  validation path (ADR-0005). S-009 EXTENDS this file in Wave 3 with alias-path branch + 5
  hook-route handlers.
- inputs: added BC-2.01.008.md v1.0.7, BC-2.01.009.md v1.0.7, ADR-0005 v1.0.2.
- §Previous Story Intelligence expanded: S-001 monocle-runtime crate + lib.rs stub + CI/MSRV
  baseline inheritance explicitly noted.
- version bumped 1.4 → 1.5.

## §Trace v1.8

**Path B Wave 6 MSRV propagation tail** (2026-05-29):
- §Previous Story Intelligence: MSRV 1.86 → 1.88 body text propagation (1 site: line "MSRV 1.86 are all inherited from S-001").
- No input-pin change required (SS-deps-pin-manifest not in S-003 inputs list).
- version bumped 1.7 → 1.8. Closes consumer-story cascade started at architect f3533ce.

## §Trace v1.7

**Phase 3.B Batch 6 — residual NON-AUTH findings** (2026-05-20):
- F-D-03 (MED): Test file split — `status_endpoint.rs` → `status_endpoint_auth.rs` (VP-002
  canonical) + `status_abi_version.rs` (VP-011 canonical). Tasks + File Structure updated.
- F-D-05 (LOW-MED): Compile-time const assert task added — `const _: () =
  assert!(monocle_core::MONOCLE_ABI_VERSION == 1);` in `monocle-runtime/src/main.rs`.
- F-C-02 (LOW): AC-002 WARN string canonicalized — ellipsis replaced with full INV-6 string
  from BC-2.01.009.
- F-C-04 (SUGGESTION): AC-008 drain harness cross-link added — drain fixture from S-005.
- F-D-04 (SUGGESTION): StatusResponse field-type table added inline in Tasks.
- Pre-existing vestige: `router.rs` → `server.rs` in File Structure Requirements
  (Batch 2 naming decision follow-up; no §Previous Story Intelligence or §Architecture
  Compliance Rules prose contained this vestige — only File Structure had the stale name).
- version bumped 1.6 → 1.7.

## §Trace v1.6

**Phase 3.B Batch 2 — blocks cascade** (2026-05-20):
- S-004 added to blocks (S-004 now depends_on S-003 per auth-ownership decision: S-003 owns
  authenticated router construction in server.rs; S-004 applies DefaultBodyLimit layer to it).
- S-010 added to blocks (S-010 now depends_on S-003 per co-dependency: S-010 imports
  monocle_core::MONOCLE_ABI_VERSION into status.rs created by S-003).
- version bumped 1.5 → 1.6.
