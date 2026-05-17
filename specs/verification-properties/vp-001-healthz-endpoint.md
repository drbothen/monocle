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
source_bc: BC-2.01.001
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

# VP-001: Healthz Endpoint — Unauthenticated Liveness 200/503 with Uptime + Version

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-001 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

`GET /healthz` is mounted on the unauthenticated router and returns a structured
JSON body documenting daemon liveness. In normal AppMode the response is HTTP
200 with exactly three keys (`status`, `uptime_sec`, `version`); during
`ShuttingDown` AppMode it is HTTP 503 with exactly two keys
(`{"status":"shutting_down"}`). The endpoint ignores any
`X-Monocle-Authorization` header (valid or invalid) and is NOT subject to the
authenticated router's 256 KiB body-size limit. The `version` field matches the
semver regex `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$`.

## Source Contract

- **BC:** BC-2.01.001 — Healthz Endpoint (Unauthenticated Liveness Probe).
- **Postcondition/Invariant:** BC-2.01.001 Postcondition 1 (semver-regex
  constraint on `version`), the structural 3-key vs 2-key shape requirement,
  and the unauthenticated-router placement invariant.
- **Traces to (historical):** BC-DAEMON-001 (PRD v1.25 §BC-DAEMON-001;
  SS-daemon-lifecycle.md v1.0.25 §Health and Status Endpoints).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — finite probe set | Status-code + body-shape across normal and drain modes; header-ignored probes; router structural probe |
| Proptest (auxiliary) | proptest | Bounded property quantification | Property-based semver-regex matching against `version` field across randomized valid/invalid token forms |
| Source-grep (structural) | ripgrep | N/A — static | Asserts `DefaultBodyLimit::max(256 * 1024)` appears only on the authenticated router construction |

## Mechanism

Integration test (harness at `monocle-runtime/tests/healthz_endpoint.rs` —
files in `<crate>/tests/` are cargo integration tests per Rust convention;
PRD v1.25 §7 RTM Test Type column labels this BC `Integration`). The harness
constructs an axum test server, hits `/healthz` with and without auth
headers, transitions AppMode to `ShuttingDown` via the synthetic
shutdown channel, and asserts the response status + JSON body shape for
each probe. A complementary source-grep asserts the body-limit layer is
mounted on the authenticated router only.

## Pre-conditions

- Daemon running with a normal AppMode (not `ShuttingDown`).
- Hook-receiver task is alive (no abnormal exit).
- `axum 0.8` is the project pin (per SS-deps-pin-manifest.md v1.1.15).

## Post-conditions

1. `GET /healthz` (no auth header) returns status code `200`.
2. Response body parsed as JSON has keys exactly `{"status", "uptime_sec",
   "version"}`. `status == "alive"`. `uptime_sec` is a JSON integer ≥ 0.
   `version` equals `env!("CARGO_PKG_VERSION")` from the daemon binary
   crate at compile time.
3. With the daemon transitioned to `ShuttingDown` (via SIGTERM or `POST
   /shutdown`), `GET /healthz` returns status code `503` and body
   `{"status":"shutting_down"}` (exactly two keys). Cross-property with
   VP-004 §Mechanical property item 3 (drain-state 503 invariant):
   VP-004 asserts the AppMode transition to `ShuttingDown` is the
   trigger; this VP asserts the resulting HTTP 503 response shape on
   `/healthz`.
4. `GET /healthz` with `X-Monocle-Authorization: monocle-v1:<valid-token>`
   produces the same response as without the header (header is ignored).
5. `GET /healthz` with `X-Monocle-Authorization: garbage` produces the same
   response (no 401 — unauthenticated router does not run the auth
   middleware).
6. The two routers' construction (`unauth_router` and `auth_router`) is
   inspected: the `DefaultBodyLimit::max(256 * 1024)` layer is added to
   `auth_router` only. A `cargo expand` or source-grep test asserts this
   structural property.
7. **Semver-regex format for `version` field** (per BC-2.01.001
   Postcondition 1 — semver regex constraint lifted in F-R91 / PRD
   v1.25 commit 7735c84 via I-R91-3 HIGH closure):
   `version` MUST match the regex
   `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$`
   (cross-VP uniformity with VP-002 §Post-condition 8
   string-format probe for the same `version` field on the `/status`
   endpoint — both VPs anchor to the same semver regex form). Verified
   by integration test asserting the regex match against
   `value["version"].as_str().unwrap()`.

## Counter-examples

1. `/healthz` mounted on the authenticated router by mistake — a no-auth
   probe would return HTTP 401 instead of 200; the test must assert 200
   on the no-auth probe.
2. Body returned as `{"status":"alive"}` only (uptime + version dropped) —
   fails the 3-key structural assertion.
3. `uptime_sec` returned as a JSON string (`"42"`) instead of integer — fails
   the integer type assertion.
4. `version` field returned as the build profile (`"debug"`) instead of
   semver — fails the semver-regex assertion (anchored to numbered
   §Post-condition 7).
5. Drain-state body returned as `{"status":"alive","uptime_sec":N,"version":
   "<v>","drain":true}` (4 keys with drain flag) — fails the exact two-key
   assertion under `ShuttingDown`.

## Probe Matrix

| Probe | Setup | Expected status | Expected body |
|-------|-------|-----------------|---------------|
| 1.a | Normal AppMode; no auth header | 200 | `{"status":"alive","uptime_sec":N,"version":"<semver>"}` (3 keys exact) |
| 1.b | Normal AppMode; valid auth header | 200 | same as 1.a (header ignored) |
| 1.c | Normal AppMode; garbage auth header | 200 | same as 1.a (header ignored; no 401 because unauth router) |
| 1.d | AppMode = `ShuttingDown`; no auth header | 503 | `{"status":"shutting_down"}` (2 keys exact) |
| 1.e | Source-grep: `DefaultBodyLimit::max(256 * 1024)` placement | N/A | layer present on `auth_router` only |
| 1.f | Regex assertion on `version` field | N/A | `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$` matches |

## Harness Location

- `monocle-runtime/tests/healthz_endpoint.rs` (integration test)
- Test name: `test_BC_DAEMON_001_healthz_unauthenticated_alive` (per PRD
  v1.25 §BC-DAEMON-001, Verification subsection — to be migrated to
  `test_BC_2_01_001_healthz_unauthenticated_alive` post BC renumber
  propagation into source).

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-DAEMON-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.001.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.001 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
