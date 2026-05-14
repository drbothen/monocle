---
document_type: verification-properties
level: L3
section: "verification-properties"
version: "1.3"
status: draft
producer: formal-verifier
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T03:30:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/prd.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-forward-compatibility.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0003-license-selection.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md
input-hash: "[live-state]"
traces_to: "22 BCs unchanged across R3-001 closure burst — 16 architecture-staged + 6 PRD-formalized daemon BCs. Architecture sources (current): SS-daemon-lifecycle v1.0.10 (commit dc3af71, R3-001 closure with version-stable §BC Summary footer — BC-DAEMON-001..006, BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001), SS-core-types-and-abi v1.2.8 (BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-FACTORY-001, BC-FACTORY-002, BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002), SS-engine-module v1.1.15 (BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003). PRD v1.3 commit d8e66c3 — current canonical BC source (31 arch-pin propagations from v1.2; content unchanged from v1.2 commit 5a49b0b, which adjudicated 4 test-name adjudications per §Trace v1.2 and remains the canonical test-name source). R3-001 closure chain: consistency-validator round 3 commit ba62a15 (R3-001 GAPS-verdict identifying arch v1.0.9 BC Summary tense oscillation); architect SS-daemon-lifecycle v1.0.10 commit dc3af71 (version-stable §BC Summary footer); product-owner PRD v1.3 commit d8e66c3 (arch v1.0.10 pin propagation). This v1.3 catalog propagates BOTH version bumps — arch v1.0.9 → v1.0.10 and PRD v1.2 → v1.3 — through every normative-current citation site (frontmatter, §Purpose narrative arch pin, §Scope, §VP Catalog Overview table, per-VP `Traces to:` lines, per-VP `Test name:` annotation citations, §Coverage Matrix, §References). Per L-F-R63-PARTIAL-FIX (cycle-001 lessons §META process-gap codification), an explicit propagation checklist was applied; no content changes — pin-only burst. Prior fix-burst context preserved in §Trace v1.2: F-R63 closures (F-R63-adv-1 + F-R63-cons-1); architect SS-daemon-lifecycle v1.0.9 commit 8bf3759 (F-R62-4 back-propagation closure); product-owner PRD v1.2 commit 5a49b0b (4 test-name adjudications + error-count correction). Prior fix-burst context preserved in §Trace v1.1: F-R62 closures (F-R62-1, F-R62-4, F-R62-5, F-R62-7, F-R62-8, F-R62-9); architect BC-AUTH-002 disposition (c) (commit 2db408f); consistency audit commit 0e322da. dtu-assessment §DTU Architecture (hook protocol clone surface); Phase 1 PRD dispatch authorization per STATE.md §Phase 1 dispatch; production-grade default per CLAUDE.md §CANONICAL PRINCIPLE"
project: monocle
---

# Verification Properties: Phase 1 Behavioral Contract Catalog

## §Purpose

This artifact authors formally-testable Verification Properties (VPs) against
the 22 Behavioral Contracts (BCs) formalized in the Phase 1 PRD v1.3 (commit
d8e66c3) and pre-staged across the Phase 1 architecture artifacts. Each VP
states a mechanical, executable property that asserts the BC holds under a
precisely scoped pre-condition. Each VP is bound to a verification mechanism —
Kani proof, fuzz harness, unit test, or mutation test — and includes
counter-example sketches that an adversary or fuzzer should generate to refute
the property.

The VP catalog is the input to Phase 6 (Formal Hardening). Every VP whose
mechanism is `unit-test` or `fuzz` is also a TDD target during Phase 3. Kani
proofs are deferred to Phase 6 but their harnesses are stubbed in this artifact
so the Phase 1 PRD can pre-stage them.

Per CLAUDE.md §CANONICAL PRINCIPLE — Production-Grade Default: every BC is
covered by at least one VP. No BC is deferred to "Phase 2 verification" or "we
can add tests later." Where a BC has both a wire-format facet and a Rust-surface
facet (BC-PROTO-001 family), each facet receives its own VP.

This v1.2 revision is the formal-verifier side of the F-R63 fix-burst. The
v1.1 catalog brought the catalog into 22-BC 1:1 correspondence (F-R62
closures) but did not align all per-VP `Test name:` annotations with the
PRD's canonical names: adversary R63 (commit 11a98c4) catalogued 4
mismatches and consistency-validator round 2 (commit 200eb68) catalogued
the same 4 mismatches plus 10 additional VPs whose harness locations were
present but whose `Test name:` lines were absent. Product-owner PRD v1.2
(commit 5a49b0b) adjudicated the canonical name per BC; this v1.2 catalog
adopts the 4 adjudicated names verbatim and adds the 10 missing `Test
name:` annotations sourced from PRD v1.2 §Section 7 RTM. This v1.2 also
propagates the SS-daemon-lifecycle.md v1.0.8 → v1.0.9 architecture bump
(architect commit 8bf3759 — F-R62-4 back-propagation closure) through
every per-VP `Traces to:` line, the §VP Catalog Overview table, the
§Coverage Matrix table, and §References.

The v1.0 catalog covered the 16 architecture-staged BCs but excluded the 6
daemon-endpoint BCs that the PRD v1.0 had not yet formalized as full contract
sections. The v1.1 revision (formal-verifier side of the F-R62 fix-burst,
commit 8454ff2) brought the catalog into 22-BC parity: VP-DAEMON-001
through VP-DAEMON-006 were added; VP-AUTH-001/002 were updated to the
collapsed two-body taxonomy; VP-PROTO-002 was reframed to be Phase 4-scoped
without fabricating Phase 1 code surface; and all test-file paths were
adopted verbatim from the PRD §7. Requirements Traceability Matrix.

---

## §Scope

In scope:

- All 22 Phase 1 BCs — 6 daemon-endpoint BCs formalized in PRD v1.3
  (BC-DAEMON-001..006) plus 16 BCs pre-staged across `SS-daemon-lifecycle.md`
  v1.0.10 (BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001),
  `SS-core-types-and-abi.md` v1.2.8 (BC-ABI-001/002, BC-TYPES-001,
  BC-FACTORY-001/002, BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002), and
  `SS-engine-module.md` v1.1.15 (BC-ENGINE-001, BC-ENGINE-002,
  BC-ENGINE-002-ERR, BC-ENGINE-003).
- Mechanical property statements (deterministic, executable assertions).
- Verification mechanism selection per VP — Kani / fuzz / unit / mutation.
- Pre-condition / post-condition pairs per VP.
- Counter-example sketches (adversarial inputs that should refute the property).
- Coverage matrix (BC → VP, one-to-one or one-to-many).
- Open verification gaps (DTU-blocked, Phase 4-deferred, etc.).

Out of scope:

- Phase 2+ BCs (deferred until Phase 2 PRD scoping).
- Performance-budget VPs (handled separately under `vsdd-factory:perf-check`).
- WCAG/accessibility VPs (TUI plane has none in Phase 1; deferred to Phase 2
  when TUI accessibility audit becomes scope).
- DTU fidelity VPs (the DTU clone itself is verified against real Claude Code;
  the DTU fidelity scoring procedure in `dtu-assessment.md` §DTU Fidelity
  Measurement Procedure is the canonical verification path, not a VP).

---

## §VP Catalog Overview

The catalog contains exactly 22 VPs, one per BC. Five VPs (VP-AUTH-001,
VP-AUTH-002, VP-FACTORY-002, VP-PROTO-002 Phase-4-deferred, VP-DAEMON-003) admit auxiliary
fuzz harnesses in addition to their primary unit-test mechanism. Four VPs
(VP-RING-001, VP-LOCK-001, VP-TYPES-001, VP-DAEMON-005) admit auxiliary
mutation-test harnesses — see §Per-VP Detail and §Coverage Matrix for the
mechanism distribution.

| VP ID | BC Source | Property Domain | Primary Mechanism | Auxiliary Mechanism |
|-------|-----------|-----------------|-------------------|---------------------|
| VP-DAEMON-001 | BC-DAEMON-001 (PRD v1.3 / SS-daemon-lifecycle v1.0.10) | `/healthz` unauthenticated 200/503 with uptime + version | unit-test | — |
| VP-DAEMON-002 | BC-DAEMON-002 (PRD v1.3 / SS-daemon-lifecycle v1.0.10) | `/status` authenticated daemon-state JSON with 10 required fields incl `abi_version: 1` | unit-test | — |
| VP-DAEMON-003 | BC-DAEMON-003 (PRD v1.3 / SS-daemon-lifecycle v1.0.10) | 256 KiB request-body limit; HTTP 413 + `payload_too_large` body on excess | unit-test | fuzz |
| VP-DAEMON-004 | BC-DAEMON-004 (PRD v1.3 / SS-daemon-lifecycle v1.0.10) | 10-second graceful drain on SIGTERM / SIGINT / `POST /shutdown`; 503 + `Retry-After: 10` on new hook POSTs during drain | unit-test | — |
| VP-DAEMON-005 | BC-DAEMON-005 (PRD v1.3 / SS-daemon-lifecycle v1.0.10) | Lock-file lifecycle atomically via `tempfile::persist`; pid-liveness gate; mode 0o600; removed on clean shutdown | unit-test | mutation-test |
| VP-DAEMON-006 | BC-DAEMON-006 (PRD v1.3 / SS-daemon-lifecycle v1.0.10) | Crash-recovery checkpoint JSON written before lock removal; 60-second TUI offer window; cleanup on accept/decline/timeout | unit-test | — |
| VP-RING-001 | BC-RING-001 (SS-daemon-lifecycle v1.0.10) | JSONL ring record format-version is first key | unit-test | mutation-test |
| VP-AUTH-001 | BC-AUTH-001 (SS-daemon-lifecycle v1.0.10) | Wire format `monocle-v1:<64-hex>`; constant-time comparison | unit-test | fuzz |
| VP-AUTH-002 | BC-AUTH-002 (SS-daemon-lifecycle v1.0.10) | Two-body taxonomy: absent header → `missing_auth_token`; any value-present failure → `invalid_auth_token` (collapsed) | unit-test | fuzz |
| VP-LOCK-001 | BC-LOCK-001 (SS-daemon-lifecycle v1.0.10) | Lock-file `contract_version: 1` first key; readers gate on field | unit-test | mutation-test |
| VP-ABI-001 | BC-ABI-001 (SS-core-types-and-abi v1.2.8) | `/status` response body contains `abi_version: 1` | unit-test | — |
| VP-ABI-002 | BC-ABI-002 (SS-core-types-and-abi v1.2.8) | `monocle_core::MONOCLE_ABI_VERSION` pub const equals `1` | unit-test | — |
| VP-TYPES-001 | BC-TYPES-001 (SS-core-types-and-abi v1.2.8) | Every pub enum in `monocle-core` carries `#[non_exhaustive]` modulo ADR-0004 exemptions | unit-test | mutation-test |
| VP-FACTORY-001 | BC-FACTORY-001 (SS-core-types-and-abi v1.2.8) | `FactoryAdapter` trait signature stable; no `private::Sealed` supertrait | unit-test | — |
| VP-FACTORY-002 | BC-FACTORY-002 (SS-core-types-and-abi v1.2.8) | `VsddFactoryAdapter::new` + self-referential detection; `None` for absent optionals | unit-test | fuzz |
| VP-PROTO-001a | BC-PROTO-001a (SS-core-types-and-abi v1.2.8) | Proto field number 1 in `HookEnvelope` is `schema_version` | unit-test | — |
| VP-PROTO-001b | BC-PROTO-001b (SS-core-types-and-abi v1.2.8) | Rust `HookEnvelope` struct exposes `pub schema_version: u32`; value `1` | unit-test | — |
| VP-PROTO-002 | BC-PROTO-002 (SS-core-types-and-abi v1.2.8) | Phase 1 verification: `schema_version` field exists at proto field 1 (structural recap of VP-PROTO-001a/001b); Phase 4 runtime: unknown `schema_version` is skipped with warning, no panic | unit-test (Phase 1 structural) | fuzz (Phase 4-only) |
| VP-ENGINE-001 | BC-ENGINE-001 (SS-engine-module v1.1.15) | `EngineModule` trait signature stable; `last_event_micros: Option<i64>`; no silent fallback | unit-test | — |
| VP-ENGINE-002 | BC-ENGINE-002 (SS-engine-module v1.1.15) | `ClaudeCodeModule::detect` strict basename match; cmdline ignored | unit-test | — |
| VP-ENGINE-002-ERR | BC-ENGINE-002-ERR (SS-engine-module v1.1.15) | `metadata`/`enrich` return `HomeUnresolvable` with all four home-env vars unset | unit-test | — |
| VP-ENGINE-003 | BC-ENGINE-003 (SS-engine-module v1.1.15) | `hook_paths()` returns exactly 5 entries — one per `HookType` variant | unit-test | — |

### §Mechanism Distribution

| Mechanism | Count (primary) | Count (auxiliary) | Total VPs touched |
|-----------|-----------------|-------------------|-------------------|
| unit-test | 22 | 0 | 22 |
| fuzz | 0 | 5 | 5 |
| mutation-test | 0 | 4 | 4 |
| Kani proof | 0 | 0 | 0 (deferred — see §Open Verification Gaps §G-1) |

Kani proof harnesses are NOT used in Phase 1 because the Phase 1 BCs do not
require model-checking — they are deterministic protocol contracts whose
verification is fully discharged by unit tests and round-trip serde fuzzing.
Phase 2 (trigger-trace state machine) and Phase 3 (wasmtime plugin host) are
the first phases where Kani's strengths (arithmetic overflow, state-machine
invariants on arbitrary inputs) become load-bearing. See §Open Verification
Gaps §G-1 for the Phase 2 trigger Kani pre-stage.

Note on VP-PROTO-002 (post-F-R62-7 reframing): the Phase 1 verification is
purely structural and is in fact a compile-time recap of VP-PROTO-001a +
VP-PROTO-001b (the `schema_version` field exists at proto-tag-1 with type
`uint32`). The runtime behavior — unknown `schema_version` is logged and
skipped, no panic, no error propagation — is exclusively a Phase 4 concern
and its dispatch surface (`monocle-ipc::dispatch`) is a Phase 4 deliverable.
This catalog does NOT mandate any Phase 1 code surface (no
`dispatch_envelope` function, no `DispatchError` type) for VP-PROTO-002. The
Phase 4 fuzz harness is documented for future implementation; it is not a
Phase 1 TDD target.

---

## §Per-VP Detail

Each VP below states: the mechanical property; the verification mechanism;
pre-conditions (test setup); post-conditions (assertions that must hold); and
counter-example sketches (adversarial inputs that, if accepted, would refute
the property). The VPs are presented in PRD §Section 7 RTM row order — daemon
endpoints first (BC-DAEMON-001..006), then the original 16 architecture-staged
BCs in the order they appear in the PRD.

### §VP-DAEMON-001 — `/healthz` Unauthenticated Liveness 200/503 with Uptime + Version

**Traces to:** BC-DAEMON-001 (PRD v1.3 §BC-DAEMON-001; SS-daemon-lifecycle.md
v1.0.10 §Health and Status Endpoints).

**Mechanical property:**

1. A `GET /healthz` request with NO `X-Monocle-Authorization` header returns
   HTTP 200 when AppMode is normal and the hook-receiver task is alive.
2. The response body is structurally a JSON object with exactly the keys
   `status`, `uptime_sec`, `version`. The `status` value is the literal string
   `"alive"`; `uptime_sec` is an integer ≥ 0; `version` is a non-empty semver
   string matching the daemon binary's compile-time `CARGO_PKG_VERSION`.
3. When AppMode is `ShuttingDown` (drain in progress), the same request
   returns HTTP 503 with body `{"status":"shutting_down"}` — exactly two keys
   in the object (no `uptime_sec`, no `version`).
4. The endpoint is registered on the unauthenticated router; it is NOT
   subject to the `DefaultBodyLimit::max(256 * 1024)` layer (BC-DAEMON-003)
   because that layer is mounted only on the authenticated router.
5. Presenting any `X-Monocle-Authorization` header (valid or invalid) does
   NOT change the response — the endpoint ignores the header entirely.

**Mechanism:** unit-test.

**Pre-conditions:**

- Daemon running with a normal AppMode (not `ShuttingDown`).
- Hook-receiver task is alive (no abnormal exit).
- `axum 0.8` is the project pin (per SS-deps-pin-manifest.md).

**Post-conditions:**

1. `GET /healthz` (no auth header) returns status code `200`.
2. Response body parsed as JSON has keys exactly `{"status", "uptime_sec",
   "version"}`. `status == "alive"`. `uptime_sec` is a JSON integer ≥ 0.
   `version` equals `env!("CARGO_PKG_VERSION")` from the daemon binary
   crate at compile time.
3. With the daemon transitioned to `ShuttingDown` (via SIGTERM or `POST
   /shutdown`), `GET /healthz` returns status code `503` and body
   `{"status":"shutting_down"}` (exactly two keys).
4. `GET /healthz` with `X-Monocle-Authorization: monocle-v1:<valid-token>`
   produces the same response as without the header (header is ignored).
5. `GET /healthz` with `X-Monocle-Authorization: garbage` produces the same
   response (no 401 — unauthenticated router does not run the auth
   middleware).
6. The two routers' construction (`unauth_router` and `auth_router`) is
   inspected: the `DefaultBodyLimit::max(256 * 1024)` layer is added to
   `auth_router` only. A `cargo expand` or source-grep test asserts this
   structural property.

**Counter-example sketches:**

1. `/healthz` mounted on the authenticated router by mistake — a no-auth
   probe would return HTTP 401 instead of 200; the test must assert 200
   on the no-auth probe.
2. Body returned as `{"status":"alive"}` only (uptime + version dropped) —
   fails the 3-key structural assertion.
3. `uptime_sec` returned as a JSON string (`"42"`) instead of integer — fails
   the integer type assertion.
4. `version` field returned as the build profile (`"debug"`) instead of
   semver — fails the semver-regex assertion.
5. Drain-state body returned as `{"status":"alive","uptime_sec":N,"version":
   "<v>","drain":true}` (4 keys with drain flag) — fails the exact two-key
   assertion under `ShuttingDown`.

**Harness location:** `monocle-runtime/tests/healthz_endpoint.rs`.

**Test name:** `test_BC_DAEMON_001_healthz_unauthenticated_alive` (per PRD
v1.2 §BC-DAEMON-001, Verification subsection).

---

### §VP-DAEMON-002 — `/status` Authenticated Daemon-State JSON with 10 Required Fields

**Traces to:** BC-DAEMON-002 (PRD v1.3 §BC-DAEMON-002; SS-daemon-lifecycle.md
v1.0.10 §Health and Status Endpoints).

**Mechanical property:**

1. A `GET /status` request with valid `X-Monocle-Authorization: monocle-v1:
   <64-hex>` returns HTTP 200 with a JSON body containing exactly the 10
   required fields: `pid` (integer), `uptime_sec` (integer), `version`
   (semver string), `abi_version` (integer 1), `lock_file` (absolute path
   string), `hook_endpoints` (JSON array of exactly 5 hook path strings),
   `ring_buffer_fill_pct` (float 0.0..=100.0), `channel_saturation_pct`
   (float 0.0..=100.0), `last_hook_ts` (JSON object with per-hook-type ISO
   8601 timestamps or `null`), `tui_attached` (boolean).
2. The `abi_version` value equals `monocle_core::MONOCLE_ABI_VERSION` at
   compile time (compile-time const-assert in the daemon binary crate
   ensures drift between binary and constant is impossible).
3. The `hook_endpoints` array contains exactly the 5 strings
   `["/hooks/pre-tool-use", "/hooks/notification", "/hooks/stop",
   "/hooks/session-start", "/hooks/prompt-submit"]` (order-insensitive set
   equality; the test sorts before compare).
4. A `GET /status` request without the auth header returns HTTP 401 with
   body `{"error":"missing_auth_token"}` (per VP-AUTH-002 cross-property).
5. A `GET /status` request with a malformed auth header returns HTTP 401
   with body `{"error":"invalid_auth_token"}` (per VP-AUTH-002).
6. `GET /status` continues to serve during graceful drain — the response
   includes the same 10 fields with `tui_attached` reflecting the live
   state. The endpoint is read-only and DOES NOT block on drain.

**Mechanism:** unit-test.

**Pre-conditions:**

- Daemon running with a valid lock file.
- Test client reads the auth token from the lock file before issuing the
  request (the canonical client-side pattern per SS-daemon-lifecycle.md
  §Daemon Lifecycle Protocol §Start Sequence).
- `monocle_core::MONOCLE_ABI_VERSION` equals `1` at the time the daemon
  binary is compiled.

**Post-conditions:**

1. `GET /status` with valid header → HTTP 200; JSON body parses; field set
   equals exactly the 10 keys above; `abi_version == 1`;
   `hook_endpoints.len() == 5`.
2. `GET /status` with no header → HTTP 401 + `{"error":"missing_auth_token"}`.
3. `GET /status` with `monocle-v2:<hex>` → HTTP 401 + `{"error":"invalid_auth_token"}`.
4. `last_hook_ts` is a JSON object; each value is either an ISO 8601 string
   matching `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` or JSON `null`.
5. Request body exceeding 256 KiB returns HTTP 413 + `payload_too_large`
   body (cross-check VP-DAEMON-003 since `/status` is on the authenticated
   router and inherits the body-limit layer).

**Counter-example sketches:**

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
   the post-2db408f BC-AUTH-002 contract).

**Harness location:** `monocle-runtime/tests/status_endpoint_auth.rs`.

**Test name:** `test_BC_DAEMON_002_status_endpoint_requires_auth_and_returns_abi_version`
(per PRD v1.3 §BC-DAEMON-002, Verification subsection).

---

### §VP-DAEMON-003 — 256 KiB Body Limit; HTTP 413 on Excess

**Traces to:** BC-DAEMON-003 (PRD v1.3 §BC-DAEMON-003; SS-daemon-lifecycle.md
v1.0.10 §Body Size Limit).

**Mechanical property:**

1. A POST to any of the 5 hook endpoints (`/hooks/pre-tool-use`,
   `/hooks/notification`, `/hooks/stop`, `/hooks/session-start`,
   `/hooks/prompt-submit`) with a request body of **262,145** bytes (one
   byte over the 256 KiB limit) returns HTTP 413 with body
   `{"error":"payload_too_large","limit_bytes":262144}`.
2. The same endpoints with a request body of **262,143** bytes (one byte
   under the limit) succeed (HTTP 200, no 413).
3. The same endpoints with a request body of exactly **262,144** bytes
   (the boundary value) also succeed (axum's `DefaultBodyLimit::max(N)`
   semantics: bodies strictly exceeding N bytes are rejected; bodies
   equal to N pass).
4. `/healthz` (unauthenticated, no body) is NOT subject to the limit; a
   POST `/healthz` is rejected with method-not-allowed (it is a GET-only
   endpoint, not via the body-limit layer).
5. `/status` (authenticated, GET) inherits the body-limit layer at the
   request path — though GET requests typically have no body, a manually
   crafted oversized request body to `/status` also returns HTTP 413.

**Mechanism:** unit-test (primary); fuzz (auxiliary — boundary
exploration).

**Pre-conditions:**

- Daemon running with a valid lock file.
- `axum::extract::DefaultBodyLimit::max(256 * 1024)` is the layer pinned
  to the authenticated router at construction time. The unit test
  asserts the layer is present via a `cargo expand` or source-grep
  inspection of `monocle-runtime/src/server.rs`.
- Test client holds the auth token for the positive controls.

**Post-conditions:**

1. POST 262,145-byte body to any of the 5 hook endpoints with valid auth →
   HTTP 413 with exact body
   `{"error":"payload_too_large","limit_bytes":262144}`.
2. POST 262,144-byte body (boundary) to any hook endpoint with valid auth →
   HTTP 200 (within limit; processed normally).
3. POST 262,143-byte body (one under) to any hook endpoint with valid auth →
   HTTP 200.
4. POST 262,145-byte body to `/status` with valid auth → HTTP 413
   (cross-route limit coverage).
5. Source-grep asserts `DefaultBodyLimit::max(256 * 1024)` appears
   exactly once in `monocle-runtime/src/server.rs` and is applied to the
   authenticated router only (not the `/healthz` route).

**Counter-example sketches:**

1. `DefaultBodyLimit` layer omitted — 262,145-byte body returns HTTP 200
   (the request is processed, exposing unbounded memory); the test must
   assert 413.
2. `DefaultBodyLimit::max(256 * 1024)` applied to the unauthenticated
   router by mistake — `/healthz` would reject oversized bodies but
   `/healthz` is GET-only; benign drift but still wrong; the source-grep
   asserts the layer is on the authenticated router only.
3. Limit set to `262_144` instead of `256 * 1024` (off-by-one constant) —
   functionally identical (both equal 262,144) but the literal constant
   form `256 * 1024` is preferred for readability; the source-grep
   tolerates either form.
4. Error body returns `{"error":"too_large"}` (typo / non-canonical) —
   fails the exact-body assertion.

**Fuzz harness:** `cargo fuzz add fuzz_body_size_boundary`. The fuzz
target constructs request bodies of varying lengths around the boundary
(262,140 to 262,150) and asserts the daemon returns HTTP 200 for
length ≤ 262,144 and HTTP 413 for length > 262,144. The fuzzer should
never produce an input that causes a daemon panic or unbounded memory
allocation.

**Harness location:** `monocle-runtime/tests/body_size_limit.rs` (unit);
`fuzz/fuzz_targets/fuzz_body_size_boundary.rs` (fuzz, Phase 6 deliverable).

**Test name:** `test_BC_DAEMON_003_body_size_limit_413_on_excess` (per PRD
v1.2 §BC-DAEMON-003, Verification subsection).

---

### §VP-DAEMON-004 — 10-Second Graceful Shutdown Drain

**Traces to:** BC-DAEMON-004 (PRD v1.3 §BC-DAEMON-004; SS-daemon-lifecycle.md
v1.0.10 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain).

**Mechanical property:**

1. Upon receiving SIGTERM, SIGINT, or an authenticated `POST /shutdown`,
   the daemon's AppMode transitions to `ShuttingDown` immediately
   (within < 10 ms of signal delivery).
2. After AppMode is `ShuttingDown`, any new POST to `/hooks/*` returns
   HTTP 503 with header `Retry-After: 10` and body
   `{"error":"daemon_shutting_down"}`.
3. `/healthz` returns HTTP 503 with body `{"status":"shutting_down"}`
   during drain (cross-property with VP-DAEMON-001 post-condition 3).
4. `/status` continues to serve normally during drain (read-only path is
   unaffected — cross-property with VP-DAEMON-002 post-condition 6).
5. In-flight `/hooks/*` POSTs that began before the signal continue to
   completion, bounded by a `tokio::time::timeout(Duration::from_secs(10),
   drain_inflight())`. After 10 seconds OR all in-flight requests
   complete (whichever comes first), the daemon proceeds to lock-file
   removal and exit.
6. Exit code is `0` if drain succeeded within the 10-second window
   without a hard kill; `130` if a second SIGTERM arrived during drain
   (forcing immediate exit).
7. `POST /shutdown` without a valid auth header returns HTTP 401 (per
   VP-AUTH-002) — unauthenticated shutdown requests are rejected.

**Mechanism:** unit-test.

**Pre-conditions:**

- Daemon running with a valid lock file.
- `tokio::signal::unix::signal(SignalKind::terminate())` is the SIGTERM
  receiver; `tokio::signal::ctrl_c()` is the SIGINT receiver.
- A test-only `oneshot::channel` is used to inject a synthetic shutdown
  signal (avoiding real OS signal delivery in unit tests).
- `axum 0.8`, `tokio 1`, `tower 0.5` are the project pins (per
  SS-deps-pin-manifest.md).

**Post-conditions:**

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
   cleanly (exit code 0).
6. With one synthetic in-flight POST that holds a 15-second sleep, the
   drain hits the 10-second timeout, the daemon force-exits, and the
   exit code observed via a test-harness wrapper is non-zero (test
   tolerates either 0 if the late completion still managed clean exit
   or 130 if hard-killed; the canonical assertion is `exit_code != 0`
   under the over-budget scenario).
7. `POST /shutdown` with no auth header → HTTP 401 +
   `{"error":"missing_auth_token"}` (VP-AUTH-002 cross-property).

**Counter-example sketches:**

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

**Harness location:** `monocle-runtime/tests/graceful_shutdown.rs`.

**Test name:** `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests`
(per PRD v1.3 §BC-DAEMON-004, Verification subsection).

---

### §VP-DAEMON-005 — Lock File Lifecycle: Atomic Create, Pid-Liveness Gate, Mode 0o600, Cleanup

**Traces to:** BC-DAEMON-005 (PRD v1.3 §BC-DAEMON-005; SS-daemon-lifecycle.md
v1.0.10 §Daemon Lifecycle Protocol §Start Sequence and §Hard Shutdown).

**Mechanical property:**

1. On daemon start, if no lock file exists at `<runtime_dir>/monocle.lock`,
   the daemon creates one atomically via `tempfile::persist` after binding
   its listener. The created lock file has file mode `0o600` (octal
   value: owner read+write; no group, no other).
2. On daemon start, if a lock file exists with a `pid` value for which
   `kill(pid, 0)` succeeds (process is alive), the daemon logs
   `ERROR: daemon already running at pid=<N>; exiting` and exits with
   code 1. No new lock file is written.
3. On daemon start, if a lock file exists with a `pid` value for which
   `kill(pid, 0)` returns `ESRCH` (no such process), the daemon logs
   `WARN: stale lock file removed`, removes the stale file, and proceeds
   with the atomic-write start-up path.
4. On clean shutdown (drain completes within 10 seconds or hard signal
   after drain), the daemon removes `<runtime_dir>/monocle.lock` and
   `<runtime_dir>/monocle.sock` before exiting.
5. The lock file is written via `tempfile::persist` (NOT via naked
   `std::fs::write`) — verified by source-grep asserting that
   `monocle-runtime/src/lock.rs` contains `tempfile::persist` and does
   NOT contain `std::fs::write` for the lock-file path.
6. On a hard SIGKILL (no graceful path), the lock file is NOT removed —
   the next daemon start exercises the stale-pid recovery path
   (post-condition 3).

**Mechanism:** unit-test (primary); mutation-test (auxiliary — the
0o600 mode value and the `kill(pid, 0)` gate are mutation surfaces).

**Pre-conditions:**

- Runtime directory `<runtime_dir>` exists or can be created with mode
  `0o700`.
- `tempfile 3` is the project pin (per SS-deps-pin-manifest.md).
- `nix 0.30` (for `kill(pid, Signal::None)`) is the project pin OR
  `libc 0.2` is used directly for `kill(pid, 0)` — the test asserts
  the chosen mechanism in the source.
- Tests use `tempfile::TempDir` to isolate `<runtime_dir>` per test.

**Post-conditions:**

1. Fresh start with no lock file → lock file created at
   `<temp_runtime>/monocle.lock`; `stat().mode() & 0o777 == 0o600`;
   JSON content begins with `{"contract_version":1,` (cross-property
   with VP-LOCK-001).
2. Daemon already running (mock: PID file contains current test
   process PID, which is alive) → daemon start returns exit code 1;
   stderr contains the substring `daemon already running at pid=`.
3. Stale lock file (PID file contains `1` or another known-dead PID
   for the test environment, or contains a PID that `kill(0)` ESRCHes)
   → daemon start succeeds; the old file is replaced; the new file
   has the live daemon's PID.
4. Daemon graceful shutdown via synthetic SIGTERM → after drain
   completes, `<temp_runtime>/monocle.lock` does not exist
   (`Path::exists()` returns `false`).
5. Daemon graceful shutdown → `<temp_runtime>/monocle.sock` does not
   exist (`Path::exists()` returns `false`).
6. Source-grep over `monocle-runtime/src/lock.rs`:
   - `tempfile::persist` appears at least once.
   - `std::fs::write` does NOT appear for the lock file path
     (an exception list may permit `std::fs::write` for non-lock paths,
     e.g., the recovery checkpoint file via separate path; the test
     restricts the negative match to lines mentioning `"monocle.lock"`).

**Counter-example sketches:**

1. Lock file written via naked `std::fs::write` — would expose a
   partial-write window between truncate and content-write; the
   source-grep negative assertion catches this. (This is also a
   semgrep rule per SS-conventions-anti-patterns.md §Semgrep Rules.)
2. Lock file written with mode `0o644` (group/other readable) — fails
   the `0o600` mode assertion; this is critical because the auth token
   is in the lock file and group/other readability would expose it to
   other OS users.
3. Stale-pid handling skipped (daemon refuses to start because lock
   file exists, without checking liveness) — fails post-condition 3.
4. Lock file not removed on clean shutdown — fails post-condition 4;
   subsequent starts would exercise the stale-pid path unnecessarily.
5. `tempfile::persist` argument `dest_path` set to a path that differs
   from the canonical `<runtime_dir>/monocle.lock` — fails the
   canonical-path assertion in post-condition 1.

**Mutation-test rationale:** the `0o600` literal in the lock-file
creation call and the `kill(pid, 0)` syscall result check are
prime mutation targets. `cargo-mutants` will attempt to mutate the
mode to `0o644` (passing functional tests that don't check mode) and
to flip the `kill` result interpretation; both must be caught.

**Harness location:** `monocle-runtime/tests/lock_file_lifecycle.rs`.

**Test name:** `test_BC_DAEMON_005_lock_file_create_and_cleanup` (per PRD
v1.2 §BC-DAEMON-005, Verification subsection).

---

### §VP-DAEMON-006 — Crash-Recovery Checkpoint JSON: Write, Offer, Cleanup

**Traces to:** BC-DAEMON-006 (PRD v1.3 §BC-DAEMON-006; SS-daemon-lifecycle.md
v1.0.10 §Daemon Lifecycle Protocol §Crash Recovery).

**Mechanical property:**

1. During graceful drain (after AppMode transition to `ShuttingDown`
   but before lock-file removal), the daemon writes
   `<runtime_dir>/monocle.recovery.json` atomically via `tempfile::persist`
   with content matching the schema:
   ```json
   {
     "pid": <int>,
     "shutdown_reason": "graceful" | "signal" | "forced",
     "last_app_mode": "<string>",
     "shutdown_utc": "<ISO8601 string matching ^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}\\.\\d{3}Z$>"
   }
   ```
   The 4 keys above are the complete schema — no additional keys.
2. On the next daemon startup, if `<runtime_dir>/monocle.recovery.json`
   exists AND the pid in the lock file (or absence of a lock file) is
   consistent with a non-graceful prior exit, the daemon logs
   `WARN: recovery checkpoint found; prior daemon exited without clean
   shutdown` and reads `last_app_mode` + `shutdown_reason` into memory.
3. If a TUI client attaches via the UDS control socket within 60 seconds
   of daemon start, the daemon sends a recovery-offer message:
   `{"type":"recovery_available","last_app_mode":"<string>"}` and awaits
   the TUI's acknowledgment.
4. On TUI ACCEPT (`Y`): the daemon deletes `monocle.recovery.json` and
   transmits the recovery state to the TUI via the same UDS channel.
5. On TUI DECLINE (`N`): the daemon deletes `monocle.recovery.json`
   without transmitting state.
6. If no TUI attaches within 60 seconds of daemon start: the daemon
   silently deletes `monocle.recovery.json` and proceeds with normal
   operation.
7. The 60-second window is measured from daemon start time (not from
   UDS-readiness time). The clock source is `std::time::Instant::now()`
   captured at daemon start.
8. Recovery file with malformed JSON (truncated, mismatched braces,
   non-UTF-8 bytes) → daemon logs `WARN: recovery file malformed;
   starting fresh` and deletes the file; no UDS recovery offer is sent.

**Mechanism:** unit-test.

**Pre-conditions:**

- Daemon binary supports the synthetic test mode where the recovery
  file is written eagerly on AppMode → `ShuttingDown` (covered by the
  drain code path).
- `tempfile 3`, `serde_json 1`, and `tokio 1` are the project pins.
- Tests use `tempfile::TempDir` to isolate the runtime directory and
  a `tokio::time::pause()` + `tokio::time::advance()` clock to drive
  the 60-second window deterministically.

**Post-conditions:**

1. Synthetic shutdown signal injected → after the drain code path
   completes, `<temp_runtime>/monocle.recovery.json` exists with content
   matching the 4-key schema and `shutdown_reason == "graceful"`.
2. Pre-created recovery file at daemon start (with an absent or
   stale-pid lock file) → daemon log captures the WARN
   `recovery checkpoint found; prior daemon exited without clean shutdown`.
3. Pre-created recovery file + a mock TUI client attached within 60
   simulated seconds via `tokio::time::advance` → mock TUI receives
   the UDS message `{"type":"recovery_available","last_app_mode":
   "<expected>"}`; after sending `Y`, the recovery file no longer
   exists.
4. Pre-created recovery file + mock TUI sends `N` → recovery file no
   longer exists; mock TUI did NOT receive a state-transmission
   payload after the ACK.
5. Pre-created recovery file + no TUI attaches; 61 simulated seconds
   advance → recovery file no longer exists; daemon log captures
   `WARN: recovery offer expired; deleting checkpoint`.
6. Pre-created recovery file with truncated content (e.g., the closing
   `}` removed) → daemon log captures `WARN: recovery file malformed;
   starting fresh`; recovery file no longer exists; no UDS message sent.
7. Recovery file's `shutdown_utc` value matches the strict ISO 8601
   regex above (no millisecond truncation, no missing `Z` suffix).
8. Recovery file is written BEFORE the lock file is removed during
   drain — the test traces the order of filesystem ops via a `tracing`
   subscriber configured to capture file-write events.

**Counter-example sketches:**

1. Recovery file written AFTER lock-file removal — on a hard SIGKILL
   between the two writes, no recovery file exists; the next daemon
   start has no recovery path. The test asserts the recovery file
   comes first (post-condition 8).
2. Recovery file has an extra key (e.g., `tui_attached: false`) — fails
   the exact-4-key schema assertion.
3. `shutdown_utc` format is `YYYY-MM-DD HH:MM:SS` (space separator
   instead of `T`) — fails the strict ISO 8601 regex.
4. 60-second window started from UDS-readiness rather than daemon start
   — under high startup latency the effective window would be
   stretched; the test asserts the start-time baseline by advancing
   the clock past `start_time + 60s` and asserting expiration.
5. TUI declines (`N`) but state is still transmitted — fails
   post-condition 4.

**Harness location:** `monocle-runtime/tests/crash_recovery.rs`.

**Test name:** `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup`
(per PRD v1.3 §BC-DAEMON-006, Verification subsection).

---

### §VP-RING-001 — JSONL Ring Record Format-Version First Key

**Traces to:** BC-RING-001 (SS-daemon-lifecycle.md §Drain).

**Mechanical property:** For every `HookEventRecord` constructed via
`HookEventRecord::new(...)`, `serde_json::to_string(&record)` produces a JSON
string whose first non-whitespace character after the opening `{` is the key
`"format_version"` with value `1`. Formally:

```
forall record: HookEventRecord constructed via HookEventRecord::new(...),
  serde_json::to_string(&record).unwrap().starts_with("{\"format_version\":1,")
```

**Mechanism:** unit-test (primary); mutation-test (auxiliary).

**Pre-conditions:**

- `RING_FORMAT_VERSION` const equals `1` (loaded from `monocle-runtime::ring`).
- `HookEventRecord` carries `#[non_exhaustive]` (otherwise the constructor
  contract is moot — see VP-TYPES-001 for the orthogonal exhaustive-enum
  property).
- `serde_json 1` is the project pin (per SS-deps-pin-manifest).

**Post-conditions:**

1. `record.format_version == 1` after construction.
2. Serialized prefix is exactly `{"format_version":1,` (literal string match).
3. Round-trip preservation: `serde_json::from_str::<HookEventRecord>(&s).unwrap().format_version == 1`.

**Counter-example sketches (adversary should attempt):**

1. Reorder struct field declarations in `HookEventRecord` such that
   `session_id` precedes `format_version` — must cause the unit test to fail
   because serde respects declaration order.
2. Change `RING_FORMAT_VERSION` to `0` or `2` — must cause the literal-prefix
   assertion to fail.
3. Use `serde_json::Value`-wrapped serialization that re-orders keys
   alphabetically (e.g., `serde_json::to_value(&record).unwrap().to_string()`)
   — this would order `format_version` after `hook_type` etc.; the unit test
   MUST use direct `to_string(&record)`, not `to_value` round-trip.
4. Replace `#[derive(Serialize)]` with a hand-written impl that emits fields in
   alphabetical order — must cause the unit test to fail.

**Harness location:** `monocle-runtime/tests/jsonl_ring.rs`.

**Test name:** `test_BC_RING_001_format_version_first_key` (per PRD v1.3
§BC-RING-001, Verification subsection).

**Mutation-test rationale:** the `format_version: u32` field value `1` is a
prime mutation target (off-by-one, sign-flip). Mutation testing with
`cargo-mutants` ensures the assertion is value-discriminating, not just
key-discriminating.

---

### §VP-AUTH-001 — Auth Token Wire Format and Constant-Time Comparison

**Traces to:** BC-AUTH-001 (SS-daemon-lifecycle.md §Start Sequence).

**Mechanical property:**

1. The lock-file `authToken` field, when read back as a string, matches the
   regex `^[0-9a-f]{64}$` (bare 64-char lowercase hex).
2. The wire-format token presented in `X-Monocle-Authorization` is exactly
   `"monocle-v1:" ++ authToken` (74 characters total: 11-char prefix + 64-char
   hex).
3. `validate_auth_token(presented, expected_secret)` returns `true` iff
   `presented` has the `monocle-v1:` prefix AND the post-prefix hex equals
   `expected_secret` byte-for-byte.
4. The comparison is performed via `constant_time_eq::constant_time_eq` — NOT
   via `==` on `&str` or `String`.

**Mechanism:** unit-test (primary); fuzz (auxiliary).

**Pre-conditions:**

- Daemon has completed start sequence and written `monocle.lock`.
- `constant_time_eq ^0.3` is the project pin (per SS-deps-pin-manifest).
- `rand::rngs::OsRng` is the entropy source (not `thread_rng`).

**Post-conditions:**

1. `lock.authToken` matches `^[0-9a-f]{64}$` (exact length 64, lowercase hex only).
2. Presenting `monocle-v1:<lock.authToken>` to `/status` returns HTTP 200.
3. Presenting `monocle-v1:<lock.authToken with one byte flipped>` returns HTTP 401.
4. Presenting `<lock.authToken>` WITHOUT the `monocle-v1:` prefix returns HTTP 401.
5. The auth middleware's secret comparison uses `constant_time_eq`; this is
   verified by source-grep against `monocle-runtime/src/auth.rs` ensuring no
   `==` on the hex secret string appears outside `constant_time_eq`.

**Counter-example sketches:**

1. Switch `constant_time_eq` to `String::eq` — would still pass functional
   tests but would lose the timing-oracle property; mitigated by the
   source-grep assertion in the harness.
2. Lock file written with `tempfile::persist` interrupted mid-write — partial
   token leaves a < 64-char hex; the regex match must reject.
3. Token generation via `rand::thread_rng()` instead of `OsRng` — passes the
   format regex but fails the entropy source check (verified by
   source-grep against `monocle-runtime/src/lock.rs`).
4. Adversary submits `monocle-v1:` + 64 chars of `0` (all-zero secret) —
   must be rejected because the real secret has 256 bits of entropy.

**Fuzz harness:** `cargo fuzz add fuzz_auth_token_validation`. The fuzz target
constructs arbitrary byte sequences as the `X-Monocle-Authorization` value
and runs `validate_auth_token(input, expected)` against a fixed 64-char hex
secret. The fuzzer should never produce an input that returns `true` other
than the exact expected secret with the `monocle-v1:` prefix. The target asserts
NO panic and NO `true` return for any input differing from the expected secret.

**Harness location:** `monocle-runtime/tests/auth_token_lifecycle.rs` (unit);
`fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz).

**Test name:** `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`
(per PRD v1.3 §BC-AUTH-001, Verification subsection).

---

### §VP-AUTH-002 — Auth Header Two-Body Taxonomy: `missing_auth_token` vs `invalid_auth_token`

**Traces to:** BC-AUTH-002 (PRD v1.3 §BC-AUTH-002; SS-daemon-lifecycle.md
v1.0.10 §Daemon Lifecycle Protocol §Start Sequence; architect adjudication
commit 2db408f — disposition (c) collapsed error taxonomy; F-R62-4
back-propagation closure landed in arch v1.0.9 commit 8bf3759).

**Mechanical property:**

1. **Absent header:** If the `X-Monocle-Authorization` header is absent
   entirely on an authenticated route (`/hooks/*`, `/status`, `/shutdown`),
   the daemon responds with HTTP 401 and the JSON body
   `{"error":"missing_auth_token"}`.
2. **Any value-present failure:** If the `X-Monocle-Authorization` header
   IS present but its value fails validation for ANY reason — wrong prefix
   (not `monocle-v1:`), malformed format, empty suffix, length mismatch,
   or correct-format-but-wrong-secret — the daemon responds with HTTP 401
   and the JSON body `{"error":"invalid_auth_token"}`. All value-present
   failure modes return the same body intentionally; the daemon does NOT
   distinguish format-fail from secret-mismatch in the response (preventing
   a timing- or body-oracle).
3. **Bearer header without X-Monocle-Authorization:** An `Authorization:
   Bearer <anything>` header with NO `X-Monocle-Authorization` header is
   treated as absent header → HTTP 401 + `{"error":"missing_auth_token"}`.
   (Phase 4 federation uses Bearer on a separate channel; Phase 1 routes
   do not recognize Bearer.)
4. **Retired body:** The body `{"error":"invalid_auth_token_format"}` from
   v1.0 of this catalog is RETIRED per architect commit 2db408f. It MUST
   NOT appear in any Phase 1 daemon response. The auth middleware's
   `AuthError` enum has exactly two variants: `Missing` and `Invalid`.

**Mechanism:** unit-test (primary); fuzz (auxiliary).

**Pre-conditions:**

- Daemon is running with a valid `monocle-v1:` secret in the lock file.
- Authenticated test client has access to the secret for the positive
  control (probe 7).
- The auth middleware's `AuthError` enum is defined as exactly:
  ```rust
  pub enum AuthError {
      Missing,  // → HTTP 401 {"error":"missing_auth_token"}
      Invalid,  // → HTTP 401 {"error":"invalid_auth_token"}
  }
  ```
  No third variant exists.

**Post-conditions (per probe):**

| Probe | Header | Expected status | Expected body |
|-------|--------|-----------------|---------------|
| 1 | (no `X-Monocle-Authorization` header) | 401 | `{"error":"missing_auth_token"}` |
| 2 | `X-Monocle-Authorization: deadbeef...64chars` (bare token, no prefix) | 401 | `{"error":"invalid_auth_token"}` |
| 3 | `X-Monocle-Authorization: monocle-v2:deadbeef...64chars` (wrong version prefix) | 401 | `{"error":"invalid_auth_token"}` |
| 4 | `X-Monocle-Authorization: monocle-v1:` (prefix only, no hex suffix) | 401 | `{"error":"invalid_auth_token"}` |
| 5 | `Authorization: Bearer fake-token` with no `X-Monocle-Authorization` (wrong header name) | 401 | `{"error":"missing_auth_token"}` |
| 6 | `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` (correct format, wrong secret) | 401 | `{"error":"invalid_auth_token"}` |
| 7 | `X-Monocle-Authorization: monocle-v1:<correct-64-hex>` (positive control) | 200 | (route's normal body) |

**Counter-example sketches:**

1. Auth middleware accepts `Authorization: Bearer` as a fallback path —
   probe 5 would return 200; the unit test must assert 401 +
   `missing_auth_token`.
2. Auth middleware uses `presented.contains("monocle-v1:")` instead of
   `strip_prefix("monocle-v1:")` — probe `X-Monocle-Authorization:
   junk-monocle-v1:abc` would be accepted; the unit test asserts strict
   `strip_prefix` behavior (returns 401 + `invalid_auth_token` for any
   value not starting with the literal prefix).
3. Auth middleware returns the retired `invalid_auth_token_format` body for
   probe 2/3/4 — fails the exact-body assertion (the retired taxonomy is
   forbidden post-2db408f).
4. Auth middleware returns `invalid_auth_token` for probe 1 (absent header
   treated as invalid) — fails the missing-vs-invalid distinction; the
   structural precondition (header absence) must produce the
   diagnostic-friendly `missing_auth_token` body.
5. Auth middleware returns `missing_auth_token` for probe 6 (correct-format
   wrong-secret) — fails the value-present unification; secret mismatch
   must produce `invalid_auth_token`, not `missing_auth_token` (an attacker
   probing the secret space must not learn that their format was correct).

**Fuzz harness:** the `fuzz_auth_token_validation` target shared with
VP-AUTH-001 is updated to assert the post-2db408f two-body taxonomy. The
fuzzer constructs arbitrary byte sequences as the `X-Monocle-Authorization`
value (including the absent-header case via `Option<Vec<u8>>`) and asserts:

- No panic.
- If header is absent: response body is exactly
  `{"error":"missing_auth_token"}`.
- If header is present but token validation fails for any reason: response
  body is exactly `{"error":"invalid_auth_token"}`.
- Response body is NEVER `{"error":"invalid_auth_token_format"}` (the
  retired body — fuzz harness asserts this body string never appears in
  any response).
- The fuzzer should never produce an input that returns 200 except for
  the exact expected secret with the `monocle-v1:` prefix.

**Harness location:** `monocle-runtime/tests/auth_header_rejection.rs`
(unit); `fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz, shared
with VP-AUTH-001).

**Test name:** `test_BC_AUTH_002_auth_header_validation_all_failure_modes`
(per PRD v1.3 §BC-AUTH-002, Verification subsection).

---

### §VP-LOCK-001 — Lock File `contract_version: 1` First Key

**Traces to:** BC-LOCK-001 (SS-daemon-lifecycle.md §Start Sequence).

**Mechanical property:**

1. The JSON content written to `<runtime_dir>/monocle.lock` is structurally a
   JSON object whose first key (per `serde_json` declaration-order
   serialization) is `contract_version` with integer value `1`.
2. Any lock-file reader (e.g., the TUI client's lock-file ingestion path)
   MUST inspect `contract_version` before consuming other fields; reading code
   asserts `contract_version == 1` and on mismatch logs a warning and skips
   the file gracefully (no panic).

**Mechanism:** unit-test (primary); mutation-test (auxiliary).

**Pre-conditions:**

- Daemon start sequence completes; lock file is written via `tempfile::persist`.
- Lock-file reader code is `pub fn read_lock_file(path: &Path) -> Result<LockFile, LockFileError>`.

**Post-conditions:**

1. `std::fs::read_to_string(&lock_path).unwrap().starts_with("{\"contract_version\":1,")`.
2. `serde_json::from_str::<LockFile>(&content).unwrap().contract_version == 1`.
3. With a synthetic lock file where `contract_version = 2`, the reader logs
   a warning and returns `Err(LockFileError::UnsupportedContractVersion(2))`
   — NOT a panic and NOT a silent acceptance of unknown fields.
4. With a synthetic lock file where `contract_version` is absent entirely, the
   reader returns `Err(LockFileError::MissingContractVersion)`.

**Counter-example sketches:**

1. Lock file written with `serde_json::to_value(&lock).to_string()` (which
   alphabetizes) — would place `app` before `contract_version`; the prefix
   assertion must fail.
2. Reader implements `serde_json::from_str` without an explicit
   `contract_version` check before field access — a future v2 lock file would
   be silently misparsed; the unit test must construct a synthetic v2 file
   and assert the version-gate error.
3. Lock-file writer omits `contract_version` (regression) — readers MUST
   reject; covered by post-condition 4.

**Mutation-test rationale:** the `contract_version` integer value `1` is a
prime mutation target. `cargo-mutants` will attempt to mutate the writer to
`contract_version = 0` and the reader's gate condition; both must be caught
by the unit test.

**Harness location:** `monocle-runtime/tests/lock_file_contract.rs`.

**Test name:** `test_BC_LOCK_001_contract_version_first_key` (per PRD v1.3
§BC-LOCK-001, Verification subsection).

---

### §VP-ABI-001 — `/status` Response Body Contains `abi_version: 1`

**Traces to:** BC-ABI-001 (SS-core-types-and-abi.md §ABI Version Constant).

**Mechanical property:** A `GET /status` request with a valid
`X-Monocle-Authorization` header returns HTTP 200 with a JSON body whose
top-level `abi_version` key has the integer value `1` (equal to
`monocle_core::MONOCLE_ABI_VERSION` as compiled into the daemon binary).

**Mechanism:** unit-test.

**Pre-conditions:**

- Daemon running with a valid lock file.
- Authenticated client holds the lock-file secret.

**Post-conditions:**

1. HTTP 200 status code on `GET /status`.
2. Response body parsed as JSON has key `abi_version` with integer value `1`.
3. The value `1` equals `monocle_core::MONOCLE_ABI_VERSION` at compile time
   (compile-time `const _: () = assert!(MONOCLE_ABI_VERSION == 1)` in the
   binary crate ensures drift between binary and constant is impossible).

**Counter-example sketches:**

1. `/status` handler hardcodes `"abi_version": 2` — must fail the literal
   integer comparison.
2. `MONOCLE_ABI_VERSION` raised to `2` without updating the status handler —
   the compile-time assert catches drift; without the assert, the unit test
   would still catch the runtime mismatch.

**Harness location:** `monocle-runtime/tests/status_abi_version.rs`.

**Test name:** `test_BC_ABI_001_status_endpoint_returns_abi_version_1` (per
PRD v1.3 §BC-ABI-001, Verification subsection).

---

### §VP-ABI-002 — `monocle_core::MONOCLE_ABI_VERSION` Pub Const Equals `1`

**Traces to:** BC-ABI-002 (SS-core-types-and-abi.md §ABI Version Constant).

**Mechanical property:**

1. `monocle_core::MONOCLE_ABI_VERSION` is publicly accessible at the crate
   root (no `pub use` from a private module that fails to re-export).
2. Its type is `u32`.
3. Its value is `1`.
4. The constant is usable in const contexts — i.e.,
   `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1);` compiles.

**Mechanism:** unit-test (specifically a compile-time test in
`monocle-core/tests/abi_stability.rs`).

**Pre-conditions:**

- `monocle-core` is the project pinned crate.
- `cargo check --tests` is the verification driver.

**Post-conditions:**

1. The `tests/abi_stability.rs` file contains `const _: () =
   assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "ABI version drift");` and
   compiles cleanly.
2. A runtime assertion `assert_eq!(monocle_core::MONOCLE_ABI_VERSION, 1u32);`
   passes.
3. The type assertion `let _: u32 = monocle_core::MONOCLE_ABI_VERSION;`
   compiles (catches accidental promotion to `u64` or demotion to `u8`).

**Counter-example sketches:**

1. `MONOCLE_ABI_VERSION` re-typed as `u64` — fails the type-pinning let-binding.
2. `MONOCLE_ABI_VERSION` defined as `pub static` instead of `pub const` — fails
   the const-context assertion (statics cannot be used in `const _:` blocks).
3. `MONOCLE_ABI_VERSION` moved into a private module without `pub use` —
   fails to compile because `monocle_core::MONOCLE_ABI_VERSION` is unresolved.

**Harness location:** `monocle-core/tests/abi_stability.rs`.

**Test name:** `test_BC_ABI_002_abi_version_const_exported` (per PRD v1.3
§BC-ABI-002, Verification subsection).

---

### §VP-TYPES-001 — Every Pub Enum in `monocle-core` Carries `#[non_exhaustive]` Modulo ADR-0004 Exemptions

**Traces to:** BC-TYPES-001 (SS-core-types-and-abi.md §Enum Extensibility).

**Mechanical property:** For every `pub enum E` defined in any source file of
the `monocle-core` crate, exactly one of the following holds:

1. `E` carries `#[non_exhaustive]`, OR
2. `E` is listed in the ADR-0004 exemption set
   `{ "Phase1Permission", "ClaudeCodeTool" }`.

No other exemption is allowed without a new ADR superseding ADR-0004.

**Mechanism:** unit-test (primary, via a `cargo clippy` lint configuration);
mutation-test (auxiliary).

**Pre-conditions:**

- `monocle-core` source tree is the audit scope.
- The exempt-list constant in the test harness is
  `EXEMPT: &[&str] = &["Phase1Permission", "ClaudeCodeTool"]`.

**Post-conditions:**

1. A test harness in `monocle-core/tests/enum_audit.rs` parses every
   `monocle-core/src/**/*.rs` file via `syn 2`, walks all `Item::Enum` nodes,
   and asserts that for each enum either `#[non_exhaustive]` is present in
   the attribute list OR the enum's identifier is in `EXEMPT`.
2. The test fails with a descriptive error listing every offending enum if
   the property is violated.
3. The `cargo clippy --workspace -- -D warnings` invocation passes with the
   project-local lint `non_exhaustive_omitted_patterns` deny-listed for
   `#[allow]` (per SS-conventions-anti-patterns.md).

**Counter-example sketches:**

1. A new contributor adds `pub enum NewError { ... }` to `monocle-core/src/`
   without `#[non_exhaustive]` and not in the exempt list — must fail the
   audit.
2. A contributor sneaks `#[allow(non_exhaustive_omitted_patterns)]` into a
   match site — must fail the clippy step (semgrep rule co-enforces, per
   SS-conventions-anti-patterns.md §Semgrep Rules).
3. A contributor adds `pub enum Phase2Permission` to `monocle-core/` (NOT in
   `monocle-plugin-sdk`) without `#[non_exhaustive]` and not in the exempt
   list — must fail the audit. (Even though ADR-0004 contemplates a parallel
   `Phase 3` enum in the plugin SDK, that enum is in a different crate and
   the audit is `monocle-core`-scoped.)
4. Exempt list expanded silently to add a third enum without an ADR
   superseding ADR-0004 — covered by an orthogonal consistency check that
   greps for the EXEMPT constant length and asserts it equals the count of
   exhaustive enums documented in ADR-0004 (currently 2).

**Mutation-test rationale:** mutating the `EXEMPT` constant length (e.g.,
adding a stray entry) or the `#[non_exhaustive]` attribute presence check
(e.g., flipping `has_attr` to `!has_attr`) must be caught by the audit
harness — this is a high-leverage mutation surface.

**Harness location:** `monocle-core/tests/enum_audit.rs`.

**Test name:** `test_BC_TYPES_001_non_exhaustive_enum_coverage` (per PRD
v1.2 §BC-TYPES-001, Verification subsection).

---

### §VP-FACTORY-001 — `FactoryAdapter` Trait Signature Stable; No Sealed Bound

**Traces to:** BC-FACTORY-001 (SS-core-types-and-abi.md §FactoryAdapter Trait).

**Mechanical property:**

1. The trait `monocle_core::factory::FactoryAdapter` exists with the exact
   method set: `detect`, `matches`, `state_file_path`, `read_state`,
   `subscribe`, `display_name`, `abi_version`.
2. The trait's super-bounds are exactly `Send + Sync + 'static` — no
   `private::Sealed` (or any other sealing) supertrait appears.
3. The supporting types
   `{FactoryDetection, FactoryState, BlockingIssue, BlockingSeverity,
   ConvergenceMetrics, FactoryReadError, FactorySubscribeError,
   StateChangeStream}` are all `pub` and accessible from
   `monocle_core::factory::*`.
4. `FactoryState` has the 7 canonical fields:
   `{ phase: String, status: String, awaiting: Option<String>,
   blocking_issues: Vec<BlockingIssue>,
   convergence: Option<ConvergenceMetrics>, cycle: Option<String>,
   custom_fields: HashMap<String, serde_yaml_ng::Value> }`.

**Mechanism:** unit-test (specifically a `cargo check` + `syn 2` parse over
the public trait surface).

**Pre-conditions:**

- `monocle-core` builds cleanly.
- `rustdoc` JSON output is available via `cargo +nightly rustdoc -- -Z unstable-options --output-format json` OR equivalent stable `cargo doc` parsing.

**Post-conditions:**

1. `cargo check --workspace` passes.
2. A `monocle-core/tests/factory_trait_surface.rs` test uses `syn 2` to
   parse `monocle-core/src/factory.rs`, locates the `trait FactoryAdapter`
   item, and asserts:
   - method count equals 7;
   - method names match the canonical set (HashSet equality);
   - super-trait bounds equal `Send + Sync + 'static` (token-stream match);
   - no `Sealed` identifier appears anywhere in the trait declaration.
3. A `FactoryState` field-name check asserts the HashSet of field identifiers
   equals the 7-field canonical set above.

**Counter-example sketches:**

1. A future refactor adds a `Sealed` supertrait — must fail the substring
   check.
2. A method is renamed (e.g., `display_name` → `name`) — must fail the
   canonical-method-set HashSet equality.
3. A new method `priority` is added without a default body — must fail the
   method count check; a method added WITH a default body is permitted
   per SS-core-types-and-abi.md §Forward Compatibility Guarantees, so the
   audit must distinguish defaulted vs non-defaulted methods (the
   `has_block` check on the `TraitItemFn` syn node distinguishes them).
4. A `FactoryState` field is renamed (e.g., `phase` → `pipeline_phase`) —
   must fail the field-name HashSet equality.

**Harness location:** `monocle-core/tests/factory_trait_surface.rs`.

**Test name:** `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound`
(per PRD v1.3 §BC-FACTORY-001, Verification subsection).

---

### §VP-FACTORY-002 — `VsddFactoryAdapter::new` + Self-Referential Detection; `None` for Absent Optionals

**Traces to:** BC-FACTORY-002 (SS-core-types-and-abi.md §FactoryAdapter Trait).

**Mechanical property:**

1. A public constructor `VsddFactoryAdapter::new(workspace_root: PathBuf) ->
   Self` exists.
2. The constructor performs NO validation (it does not panic, does not error,
   does not stat the filesystem); validation happens in `detect()` and
   `read_state()`.
3. The static method `VsddFactoryAdapter::detect(<monocle repo root>)`
   returns `Some(FactoryDetection)` where `display_name == "VSDD Factory"`
   (self-referential test).
4. For a `FactoryState` produced from a STATE.md file lacking
   `current_cycle:`, `state.cycle == None` (NOT `Some("unknown")` or any
   placeholder string).
5. For a `FactoryState` produced from a STATE.md file lacking a §Session
   Resume Checkpoint section, `state.convergence == None`.

**Mechanism:** unit-test (primary); fuzz (auxiliary).

**Pre-conditions:**

- `monocle-core` builds cleanly.
- Test fixture STATE.md files are checked in under
  `monocle-core/tests/fixtures/`:
  - `state_minimal.md` — has `current_cycle:` and §Session Resume Checkpoint.
  - `state_no_cycle.md` — lacks `current_cycle:` frontmatter key.
  - `state_no_checkpoint.md` — has `current_cycle:` but lacks §Session
    Resume Checkpoint section.

**Post-conditions:**

1. `VsddFactoryAdapter::new(PathBuf::from("/nonexistent/path"))` returns a
   value without error or panic.
2. `VsddFactoryAdapter::detect(&monocle_repo_root)` returns
   `Some(d)` where `d.display_name == "VSDD Factory"` and `d.state_file ==
   monocle_repo_root.join(".factory/STATE.md")`.
3. For fixture `state_no_cycle.md`, `state.cycle.is_none()`.
4. For fixture `state_no_checkpoint.md`, `state.convergence.is_none()`.
5. For fixture `state_minimal.md`, `state.cycle == Some("cycle-001".into())`
   (or whatever the fixture declares) — proves the `None` cases are
   discriminating, not a vacuous default.
6. `parse_frontmatter_field` returns `None` for: empty values, flow-style
   lists (`[...]`), block scalars (`|` or `>` lead), and continuation lines.

**Counter-example sketches:**

1. The constructor stats `workspace_root` and panics on absent path — fails
   post-condition 1.
2. `read_state` substitutes `"unknown"` for absent `current_cycle:` — fails
   post-condition 3.
3. `parse_frontmatter_field` accepts `awaiting: [a, b]` and returns
   `Some("[a, b]")` — fails post-condition 6 (the v1.2.3 round-20 fix).
4. `parse_frontmatter_field` accepts `phase: |` block-scalar marker and
   returns `Some("|")` — fails post-condition 6.
5. Self-referential detect fails because the `document_type:
   pipeline-state` substring check is too strict (e.g., requires exact
   line equality including trailing whitespace) — fails post-condition 2.

**Fuzz harness:** `cargo fuzz add fuzz_state_md_parser`. The fuzz target
feeds arbitrary UTF-8 byte sequences into `parse_frontmatter_field(content,
"phase")` and `parse_frontmatter_extra_fields(content, &known_keys)` and
asserts: no panic; no allocation > 1 MiB; flow-style and block-scalar inputs
produce `None` (frontmatter_field) or are skipped (extra_fields). The fuzzer
is seeded with `state_minimal.md`, `state_no_cycle.md`, and adversarial
malformed corpora (truncated frontmatter, mismatched quotes, Unicode
direction overrides, deep nesting markers).

**Harness location:** `monocle-core/tests/factory_self_referential.rs` (unit);
`fuzz/fuzz_targets/fuzz_state_md_parser.rs` (fuzz).

**Test name:** `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection`
(per PRD v1.3 §BC-FACTORY-002, Verification subsection).

---

### §VP-PROTO-001a — Proto Field Number 1 in `HookEnvelope` is `schema_version`

**Traces to:** BC-PROTO-001a (SS-core-types-and-abi.md §Prost Wire Schemas).

**Mechanical property:** In `monocle-proto/proto/monocle/v1/hook_envelope.proto`,
the `HookEnvelope` message's field assigned to proto-tag-number `1` has the
field name `schema_version` and type `uint32`. The wire-level invariant is
verified by encoding a `HookEnvelope` and decoding the first field tag.

**Mechanism:** unit-test.

**Pre-conditions:**

- `monocle-proto` build script (`build.rs`) compiles the `.proto` files via
  `prost-build`.
- `prost-reflect` or direct `prost::encoding` is available in `[dev-dependencies]`.

**Post-conditions:**

1. Encoding a `HookEnvelope { schema_version: 1, ... }` via `prost::Message::
   encode_to_vec(&envelope)` produces a byte stream whose first wire-tag
   decodes to field number 1 with wire type `Varint` (proto3 `uint32` =
   varint).
2. A `prost-build`-generated descriptor inspection (via
   `prost_reflect::DescriptorPool::decode(...)` over the FileDescriptorSet
   emitted by `build.rs`) confirms field number 1 is named `schema_version`.

**Counter-example sketches:**

1. The `.proto` file is edited so `schema_version = 5;` — must fail the
   wire-tag decode (the first tag would decode to field 5 instead of 1).
2. A new field `string trace_id = 1;` is inserted, displacing
   `schema_version` to a new number — must fail the field-name lookup.

**Harness location:** `monocle-proto/tests/wire_field_order.rs`.

**Test name:** `test_BC_PROTO_001a_schema_version_field_number_1` (per PRD
v1.2 §BC-PROTO-001a, Verification subsection).

---

### §VP-PROTO-001b — Rust `HookEnvelope` Struct Exposes `pub schema_version: u32` with Value `1`

**Traces to:** BC-PROTO-001b (SS-core-types-and-abi.md §Prost Wire Schemas).

**Mechanical property:** The prost-build-generated Rust type
`monocle_proto::v1::HookEnvelope` exposes a public field
`schema_version: u32`. For all Phase 1-origin messages (those constructed
inside Phase 1 monocle code), the value of `schema_version` is `1`.

**Mechanism:** unit-test.

**Pre-conditions:**

- `monocle-proto` builds cleanly.
- The `pub use monocle::v1` re-export is present so callers can access
  `monocle_proto::v1::HookEnvelope`.

**Post-conditions:**

1. A unit test constructs a `HookEnvelope` with `schema_version: 1` and any
   `oneof event` variant (e.g., `SessionStartEvent { cwd: "/", transcript_path:
   "" }`) and asserts `envelope.schema_version == 1`.
2. Round-trip serialize/deserialize preserves `schema_version`:
   `HookEnvelope::decode(envelope.encode_to_vec().as_slice()).unwrap()
   .schema_version == 1`.
3. The Rust struct field declaration order is NOT asserted (per BC-PROTO-001b
   normative carve-out — the proto-tag-number is the wire contract, not the
   Rust field declaration order).

**Counter-example sketches:**

1. The `.proto` file changes `uint32 schema_version = 1;` to
   `int32 schema_version = 1;` — would change the Rust type to `i32`; the
   `pub schema_version: u32` type check fails.
2. A constructor helper in `monocle-proto` defaults `schema_version` to `0`
   — fails post-condition 1.

**Harness location:** `monocle-proto/tests/schema_version.rs`.

**Test name:** `test_BC_PROTO_001b_schema_version_rust_field` (per PRD v1.3
§BC-PROTO-001b, Verification subsection).

---

### §VP-PROTO-002 — `schema_version` Forward-Compat Contract (Phase 1 Structural Recap; Phase 4 Runtime Dispatch)

**Traces to:** BC-PROTO-002 (PRD v1.3 §BC-PROTO-002; SS-core-types-and-abi.md
v1.2.8 §Prost Wire Schemas).

**Reframing rationale (F-R62-7):** v1.0 of this catalog required
`monocle-proto` to export a Phase 1 stub
`pub fn dispatch_envelope(env: &HookEnvelope) -> Result<(), DispatchError>`
with a Phase 1 runtime semantics. That requirement fabricated a Phase 1
code surface — neither `SS-core-types-and-abi.md` nor any other
architecture artifact specifies a Phase 1 dispatcher; PRD v1.3
§BC-PROTO-002 explicitly classifies the runtime test as Phase 4
(BC-PROTO-002 unchanged from PRD v1.1 → v1.2 → v1.3; classification
preserved across both v1.2 test-name adjudication burst and v1.3 arch-pin
propagation burst).

This v1.1 reframing splits the VP into a Phase 1 structural contract
(verifiable now without fabricating new code surface) and a Phase 4
runtime-dispatch contract (verifiable when the Phase 4 IPC dispatcher
exists).

**Phase 1 Mechanical property (structural):**

1. The compiled proto schema has a field named `schema_version` at proto
   field number `1` of `HookEnvelope` with type `uint32`. This is the
   structural precondition for any future runtime dispatcher.
2. The generated Rust struct `monocle_proto::v1::HookEnvelope` exposes
   `pub schema_version: u32` and the value `1` is the Phase 1 canonical
   value.

These two properties are already covered by VP-PROTO-001a (wire-format)
and VP-PROTO-001b (Rust surface). VP-PROTO-002's Phase 1 verification is
therefore a structural recap that asserts these two properties IN
COMBINATION — both must hold for any future dispatcher to function. The
Phase 1 unit test simply re-invokes the two cross-property assertions in
a single harness file to make the cross-property dependency explicit and
greppable.

**Phase 4 Mechanical property (runtime dispatch — deferred):**

1. When Phase 4's `monocle-ipc` crate exists with its dispatcher (to be
   designed in Phase 4 architecture), a `HookEnvelope` message with
   `schema_version = 0` or any unrecognized value other than `1` MUST be
   processed by:
   - Emitting a `tracing::warn!` event with the structured field
     `schema_version = <unknown_value>` and a descriptive message.
   - Returning success (skip) without panic and without propagating an
     error to the caller. The exact dispatcher API (function signature,
     error type, return type) is a Phase 4 design decision.
2. The forward-compatibility contract is: a Phase 1 daemon talking to a
   future Phase 4 peer that sends an unknown `schema_version` MUST NOT
   crash; conversely, a Phase 4 daemon receiving a Phase 1
   `schema_version = 1` message MUST process it normally.

The Phase 4 mechanical property does NOT mandate a Phase 1 code surface
in `monocle-proto`. The `monocle-ipc::dispatch` crate, the
`dispatch_envelope` function signature, and the `DispatchError` type
(or equivalent) are Phase 4 deliverables and will be specified by the
Phase 4 architecture artifact. This catalog will be extended in a
Phase 4 v2.0 revision with a `VP-IPC-DISPATCH-001` (or similar) entry
to author the runtime mechanical property against the Phase 4 dispatcher.

**Mechanism:**

- **Phase 1:** unit-test (structural — cross-property recap of
  VP-PROTO-001a + VP-PROTO-001b).
- **Phase 4 (deferred):** unit-test (runtime warn-and-skip behavior) +
  fuzz (auxiliary — arbitrary `u32` value space for `schema_version`).

**Phase 1 Pre-conditions:**

- `monocle-proto` builds cleanly.
- `prost-build` emits a Rust struct for `HookEnvelope` with
  `pub schema_version: u32` (verified by VP-PROTO-001b).
- The compiled proto descriptor has `schema_version` at field number `1`
  (verified by VP-PROTO-001a).

**Phase 1 Post-conditions:**

1. The cross-property recap test instantiates a `HookEnvelope {
   schema_version: 1, event: <any oneof variant> }` and asserts
   `envelope.schema_version == 1` (cross-link to VP-PROTO-001b).
2. The same test inspects the FileDescriptorSet emitted by `build.rs` and
   asserts field number 1 is named `schema_version` (cross-link to
   VP-PROTO-001a). The test fails CLOSED if either underlying property is
   regressed — i.e., if VP-PROTO-001a or VP-PROTO-001b would fail, this
   structural recap also fails.
3. The test file is empty of any Phase 1 dispatcher invocation. It does
   NOT import a `dispatch_envelope` function (none is mandated).

**Phase 1 Counter-example sketches:**

1. `schema_version` field renumbered to `2` (proto-tag change) — fails
   the field-number assertion (cross-property regression detected here
   even if VP-PROTO-001a's primary harness was disabled).
2. `schema_version` removed from the Rust struct (e.g., made private) —
   fails the Rust-surface assertion (cross-property regression).

**Phase 4 Counter-example sketches (deferred):**

1. Phase 4 dispatcher panics on unknown version.
2. Phase 4 dispatcher propagates an error to the caller instead of
   logging + skipping.
3. Phase 4 dispatcher silently accepts unknown versions without emitting
   a `tracing::warn!` event (the "silent acceptance" regression).

**Phase 4 Fuzz harness (deferred):** when Phase 4 lands, a `cargo fuzz
add fuzz_envelope_dispatch` target will exercise arbitrary `u32`
`schema_version` values and assert the no-panic + warn-and-skip
behavior. This harness is NOT a Phase 1 deliverable.

**Open gap reference:** §G-3 catalogues the Phase 4 federation auth as
out-of-Phase-1 scope; the same out-of-scope boundary applies to the
Phase 4 runtime dispatch behavior of this VP. §G-3 is the
future-attachment anchor for both items.

**Harness location:** Phase 4 (no Phase 1 harness — the structural recap
is discharged by VP-PROTO-001a's `monocle-proto/tests/wire_field_order.rs`
and VP-PROTO-001b's `monocle-proto/tests/schema_version.rs`). Per PRD
v1.3 §Section 7 RTM, BC-PROTO-002 has no Phase 1 test file path; the
Phase 4 test file will be authored against `monocle-ipc/tests/...` when
that crate exists.

**Test name:** No Phase 1 test name — BC-PROTO-002 is Phase 4-deferred per
PRD v1.3 §BC-PROTO-002 (Phase 4 test name `test_BC_PROTO_002_schema_version_validation_skip_unknown`
documented in PRD v1.3 §BC-PROTO-002 Verification subsection for Phase 4
implementation only; not a Phase 1 deliverable).

---

### §VP-ENGINE-001 — `EngineModule` Trait Signature Stable; `last_event_micros: Option<i64>`; No Silent Fallback

**Traces to:** BC-ENGINE-001 (SS-engine-module.md §Behavioral Contracts).

**Mechanical property:**

1. The trait `monocle_core::engine::EngineModule` exists with the exact
   method set: `id`, `metadata`, `detect`, `enrich`, `on_hook`.
2. The trait has NO sealed bound (no `private::Sealed` supertrait).
3. `metadata()` returns `Result<EngineMetadata, EngineMetadataError>`;
   `enrich()` returns `Result<EnrichedSession, EngineMetadataError>` (both
   typed-error returns, not `Option<...>`-with-silent-fallback).
4. `EnrichedSession::last_event_micros` has type `Option<i64>` (NOT bare
   `i64`); `None` is distinguishable from any numeric value including the
   Unix epoch `0`.
5. Supporting types `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`,
   `SessionStatus`, `HookResponse`, `HookDecision`, `DeferUntil`,
   `EngineMetadataError` are all `pub` in `monocle_core::engine`.

**Mechanism:** unit-test (via `syn 2` parse of `monocle-core/src/engine.rs`).

**Pre-conditions:**

- `monocle-core` builds cleanly.

**Post-conditions:**

1. A `monocle-core/tests/engine_module_surface.rs` test parses the trait
   declaration and asserts:
   - method count equals 5;
   - method names match the canonical HashSet
     `{id, metadata, detect, enrich, on_hook}`;
   - super-bounds equal `Send + Sync + 'static` (no `Sealed`);
   - `metadata` return type token-stream matches
     `Result < EngineMetadata , EngineMetadataError >`;
   - `enrich` return type token-stream matches
     `Result < EnrichedSession , EngineMetadataError >`.
2. The same test asserts `EnrichedSession::last_event_micros` field type is
   `Option < i64 >` (not bare `i64`).
3. All eight supporting types resolve via `cargo check` with a probe file
   `let _: monocle_core::engine::EngineMetadata; ...`.

**Counter-example sketches:**

1. A refactor changes `metadata() -> Result<...>` to
   `metadata() -> EngineMetadata` (panicking on home-unresolvable) — fails
   the return-type token-stream match.
2. `last_event_micros` is reverted to bare `i64` with `0` as sentinel — fails
   the field-type assertion. This regression is what the v1.1.8 fix
   (F-R28-1) closed; the VP enforces it.
3. A `private::Sealed` supertrait is added — fails the no-sealed
   assertion. ADR-0004 governs the open trait property; sealing the trait
   would defeat Phase 3 plugin SDK adapter authoring.

**Harness location:** `monocle-core/tests/engine_module_surface.rs`.

**Test name:** `test_BC_ENGINE_001_trait_defined_all_methods_no_sealed_bound`
(per PRD v1.3 §BC-ENGINE-001, Verification subsection).

---

### §VP-ENGINE-002 — `ClaudeCodeModule::detect` Strict Basename Match; Cmdline Ignored

**Traces to:** BC-ENGINE-002 (SS-engine-module.md §Behavioral Contracts).

**Mechanical property:** `ClaudeCodeModule::detect(&snapshot)` returns `true`
iff `snapshot.exe_path` is `Some(p)` AND `p.file_name() == Some("claude") ||
p.file_name() == Some("claude.js")`. The method NEVER consults
`snapshot.cmdline` for identification.

**Mechanism:** unit-test.

**Pre-conditions:**

- `ProcessSnapshot::new(pid, exe_path, cmdline, start_time_secs)`
  constructor is available (per F-R26-adv-1 fix, v1.1.7).
- `ClaudeCodeModule::new("http://127.0.0.1:7891".into())` constructs a module.

**Post-conditions (per probe):**

| Probe | exe_path | cmdline | Expected `detect()` |
|-------|----------|---------|---------------------|
| (a) | `Some("/usr/local/bin/claude")` | `vec![]` | `true` |
| (b) | `Some("/usr/local/bin/claude-squad")` | `vec![]` | `false` |
| (c) | `None` | `vec!["claude".to_string()]` | `false` (exe_path=None regardless of cmdline) |
| (d) | `Some("/opt/anthropic/claude.js")` | `vec![]` | `true` |
| (e) | `Some("/usr/local/bin/claudio")` | `vec!["claude", "--debug"]` | `false` |
| (f) | `Some("/home/x/bin/claude-code-router")` | `vec![]` | `false` |

**Counter-example sketches:**

1. `detect` uses `cmdline[0].contains("claude")` — probe (c) returns `true`;
   the unit test must assert `false`.
2. `detect` uses `exe_path.starts_with("/usr/local/bin/claude")` (prefix
   match, not basename) — probe (b) returns `true`; the unit test asserts
   `false`.
3. `detect` uses `exe_path.contains("claude")` — probes (b), (e), (f) all
   return `true`; the unit test asserts `false` for each.

**Harness location:** `monocle-runtime/tests/engine_module_claude_detect.rs`.

**Test name:** `test_BC_ENGINE_002_claude_code_module_strict_basename_detect`
(per PRD v1.3 §BC-ENGINE-002, Verification subsection).

---

### §VP-ENGINE-002-ERR — `metadata`/`enrich` Return `HomeUnresolvable` with All Four Home-Env Vars Unset

**Traces to:** BC-ENGINE-002-ERR (SS-engine-module.md §Behavioral Contracts).

**Mechanical property:** When `HOME`, `USERPROFILE`, `HOMEDRIVE`, and
`HOMEPATH` are all unset (set to `None::<&str>` via `temp_env::with_vars` /
`async_with_vars`), `ClaudeCodeModule::metadata()` and
`ClaudeCodeModule::enrich(&snapshot)` both return
`Err(EngineMetadataError::HomeUnresolvable)`. The implementation MUST NOT
substitute a relative-path default, a current-directory fallback, or any
non-`HomeUnresolvable` error path.

**Mechanism:** unit-test (with `temp-env ^0.3` env-isolation, per
SS-deps-pin-manifest pin).

**Pre-conditions:**

- `temp-env = { version = "^0.3", features = ["async_closure"] }` in
  `[dev-dependencies]`.
- Test does NOT use `std::env::set_var` / `remove_var` directly; only
  `temp_env::with_vars` / `temp_env::async_with_vars` (RAII cleanup safe
  under panic and multi-threaded harness).

**Post-conditions:**

1. Sync half: inside `temp_env::with_vars([("HOME", None::<&str>),
   ("USERPROFILE", None::<&str>), ("HOMEDRIVE", None::<&str>),
   ("HOMEPATH", None::<&str>)], || { ... })`:
   - `module.metadata().is_err()` is `true`;
   - `matches!(module.metadata().unwrap_err(),
     EngineMetadataError::HomeUnresolvable)` is `true`.
2. Async half: inside `temp_env::async_with_vars([...], async { ... }).await`:
   - `module.enrich(&snapshot).await.is_err()` is `true`;
   - `matches!(module.enrich(&snapshot).await.unwrap_err(),
     EngineMetadataError::HomeUnresolvable)` is `true`.
3. Test passes on Linux and macOS CI runners deterministically. On Windows CI
   the test is best-effort (Windows may resolve `home_dir()` via
   `FOLDERID_Profile` regardless of env-var state); the Linux/macOS gates
   are the canonical assertion.

**Counter-example sketches:**

1. `metadata()` returns `Ok(EngineMetadata { home_dir: PathBuf::from("."), ... })`
   substituting `.` for unresolvable home — fails the `is_err` assertion.
2. `metadata()` returns `Err(EngineMetadataError::Io(...))` instead of
   `HomeUnresolvable` — fails the `matches!` assertion.
3. Test uses `std::env::remove_var` instead of `temp_env::with_vars` — the
   audit (a semgrep rule in SS-conventions-anti-patterns.md
   §Semgrep Rules `monocle-no-raw-env-mutation-in-tests`) fails the harness.
4. Test omits any of the four required env-vars (e.g., only clears `HOME`)
   — Windows may resolve via `USERPROFILE` even on Linux containers with
   `wine`-style env shimming; the test must clear all four.

**Harness location:** `monocle-runtime/tests/engine_module_home_unresolvable.rs`.

**Test name:** `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich`
(per PRD v1.3 §BC-ENGINE-002-ERR, Verification subsection).

**Test design (per PRD v1.2 §Trace v1.2 adjudication):** the test name
identifies the two behavioral surfaces under contract — `metadata()` and
`enrich()` — both of which must return `Err(EngineMetadataError::HomeUnresolvable)`
when all four home-env vars are unset. The internal use of
`temp_env::with_vars` for `metadata()` (sync) and `temp_env::async_with_vars`
for `enrich()` (async) is a test-implementation strategy, not the
behavioral discriminator; per the PRD v1.2 adjudication "test names should
describe what is verified, not how the test harness is structured." The
property and post-conditions above remain valid; only the naming convention
was clarified by the F-R63 adjudication.

---

### §VP-ENGINE-003 — `hook_paths()` Returns Exactly 5 Entries — One per `HookType` Variant

**Traces to:** BC-ENGINE-003 (SS-engine-module.md §Struct-level inherent operations).

**Mechanical property:** `ClaudeCodeModule::hook_paths()` returns a structure
containing exactly 5 entries, one per `HookType` variant. The path strings
are exactly:

| HookType variant | Path |
|------------------|------|
| `SessionStart` | `/hooks/session-start` |
| `UserPromptSubmit` | `/hooks/prompt-submit` |
| `PreToolUse` | `/hooks/pre-tool-use` |
| `Notification` | `/hooks/notification` |
| `Stop` | `/hooks/stop` |

**Mechanism:** unit-test.

**Pre-conditions:**

- `ClaudeCodeModule::new("http://127.0.0.1:7891".into())` constructs a module.
- `HookType` is the canonical 5-variant enum from `monocle_core::HookType`.

**Post-conditions:**

1. `module.hook_paths().len() == 5`.
2. For each `HookType` variant `v`, `module.hook_paths().get(&v)` returns
   `Some(&"/hooks/...".to_string())` matching the table above exactly.
3. No extra variants exist (the `match` over `HookType` is exhaustive in the
   harness — adding a 6th variant would fail to compile, which is the
   correct propagation given `#[non_exhaustive]` on `HookType` is for
   external consumers; the trait implementer (this crate) is internal and
   so `HookType` exhaustive matching is valid here).

**Counter-example sketches:**

1. `hook_paths()` returns 4 entries (missing one) — fails
   post-condition 1.
2. A path is typoed (`/hooks/pre_tool_use` with underscore instead of
   hyphen) — fails the exact-string match.
3. A new variant added to `HookType` (e.g., `PostToolUse`) without updating
   `hook_paths()` — the exhaustive match in the harness fails to compile,
   forcing the implementer to update.
4. `spawn()` or `preflight()` are accidentally moved into the
   `EngineModule` trait (they MUST remain inherent methods on
   `ClaudeCodeModule`) — fails an orthogonal source-grep check against
   `monocle-core/src/engine.rs`.
5. The ABI version is read via a trait method (e.g., `module.abi_version()`)
   instead of `monocle_core::MONOCLE_ABI_VERSION` const — fails an
   orthogonal source-grep check.

**Harness location:** `monocle-runtime/tests/engine_module_claude_methods.rs`.

**Test name:** `test_BC_ENGINE_003_claude_module_hook_paths_five_entries`
(per PRD v1.3 §BC-ENGINE-003, Verification subsection — hybrid name
adjudicated by product-owner combining `claude_module` (concrete struct
under test, not the trait), `hook_paths` (the inherent method), and
`five_entries` (the count assertion); see PRD v1.2 §Trace v1.2 for the
adjudication reasoning).

---

## §Coverage Matrix (BC → VP)

| BC ID | BC Source File | VP ID | Mechanism (primary) | Phase 1 Test File |
|-------|----------------|-------|---------------------|-------------------|
| BC-DAEMON-001 | PRD v1.3 / SS-daemon-lifecycle.md v1.0.10 | VP-DAEMON-001 | unit-test | `monocle-runtime/tests/healthz_endpoint.rs` |
| BC-DAEMON-002 | PRD v1.3 / SS-daemon-lifecycle.md v1.0.10 | VP-DAEMON-002 | unit-test | `monocle-runtime/tests/status_endpoint_auth.rs` |
| BC-DAEMON-003 | PRD v1.3 / SS-daemon-lifecycle.md v1.0.10 | VP-DAEMON-003 | unit-test | `monocle-runtime/tests/body_size_limit.rs` |
| BC-DAEMON-004 | PRD v1.3 / SS-daemon-lifecycle.md v1.0.10 | VP-DAEMON-004 | unit-test | `monocle-runtime/tests/graceful_shutdown.rs` |
| BC-DAEMON-005 | PRD v1.3 / SS-daemon-lifecycle.md v1.0.10 | VP-DAEMON-005 | unit-test | `monocle-runtime/tests/lock_file_lifecycle.rs` |
| BC-DAEMON-006 | PRD v1.3 / SS-daemon-lifecycle.md v1.0.10 | VP-DAEMON-006 | unit-test | `monocle-runtime/tests/crash_recovery.rs` |
| BC-RING-001 | SS-daemon-lifecycle.md v1.0.10 | VP-RING-001 | unit-test | `monocle-runtime/tests/jsonl_ring.rs` |
| BC-AUTH-001 | SS-daemon-lifecycle.md v1.0.10 | VP-AUTH-001 | unit-test | `monocle-runtime/tests/auth_token_lifecycle.rs` |
| BC-AUTH-002 | SS-daemon-lifecycle.md v1.0.10 | VP-AUTH-002 | unit-test | `monocle-runtime/tests/auth_header_rejection.rs` |
| BC-LOCK-001 | SS-daemon-lifecycle.md v1.0.10 | VP-LOCK-001 | unit-test | `monocle-runtime/tests/lock_file_contract.rs` |
| BC-ABI-001 | SS-core-types-and-abi.md v1.2.8 | VP-ABI-001 | unit-test | `monocle-runtime/tests/status_abi_version.rs` |
| BC-ABI-002 | SS-core-types-and-abi.md v1.2.8 | VP-ABI-002 | unit-test | `monocle-core/tests/abi_stability.rs` |
| BC-TYPES-001 | SS-core-types-and-abi.md v1.2.8 | VP-TYPES-001 | unit-test | `monocle-core/tests/enum_audit.rs` |
| BC-FACTORY-001 | SS-core-types-and-abi.md v1.2.8 | VP-FACTORY-001 | unit-test | `monocle-core/tests/factory_trait_surface.rs` |
| BC-FACTORY-002 | SS-core-types-and-abi.md v1.2.8 | VP-FACTORY-002 | unit-test | `monocle-core/tests/factory_self_referential.rs` |
| BC-PROTO-001a | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-001a | unit-test | `monocle-proto/tests/wire_field_order.rs` |
| BC-PROTO-001b | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-001b | unit-test | `monocle-proto/tests/schema_version.rs` |
| BC-PROTO-002 | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-002 | unit-test (structural recap) | Phase 4 (no Phase 1 harness) |
| BC-ENGINE-001 | SS-engine-module.md v1.1.15 | VP-ENGINE-001 | unit-test | `monocle-core/tests/engine_module_surface.rs` |
| BC-ENGINE-002 | SS-engine-module.md v1.1.15 | VP-ENGINE-002 | unit-test | `monocle-runtime/tests/engine_module_claude_detect.rs` |
| BC-ENGINE-002-ERR | SS-engine-module.md v1.1.15 | VP-ENGINE-002-ERR | unit-test | `monocle-runtime/tests/engine_module_home_unresolvable.rs` |
| BC-ENGINE-003 | SS-engine-module.md v1.1.15 | VP-ENGINE-003 | unit-test | `monocle-runtime/tests/engine_module_claude_methods.rs` |

**Coverage:** 22 BCs → 22 VPs (one-to-one). Zero BCs without a VP. Every
test-file path matches PRD v1.3 §7. Requirements Traceability Matrix verbatim (F-R62-4 closure carried forward in v1.2/v1.3; arch back-propagation closed by SS-daemon-lifecycle.md v1.0.9 commit 8bf3759, with version-stable phrasing landed in arch v1.0.10 commit dc3af71 per R3-001 closure). Every per-VP `Test name:` line matches PRD v1.3 §Section 7 RTM and the corresponding BC `Verification` subsection verbatim (F-R63-adv-1 + F-R63-cons-1 closure carried forward; PRD v1.3 content unchanged from v1.2 commit 5a49b0b).

### §Auxiliary Mechanism Coverage

| VP ID | Auxiliary mechanism | Rationale |
|-------|---------------------|-----------|
| VP-DAEMON-003 | fuzz | Boundary exploration around the 262,144-byte body-size cutoff; ensures no panic / no unbounded alloc across body-length space |
| VP-DAEMON-005 | mutation-test | The `0o600` file-mode literal and the `kill(pid, 0)` syscall result interpretation are high-leverage mutation targets |
| VP-RING-001 | mutation-test | `format_version: u32 = 1` is a high-leverage value-mutation target |
| VP-AUTH-001 | fuzz | Adversarial inputs to `validate_auth_token` must never produce a false-`true` |
| VP-AUTH-002 | fuzz | Same fuzz target as VP-AUTH-001 — exercises the two-body taxonomy (missing vs invalid) and asserts the retired `invalid_auth_token_format` body never appears |
| VP-LOCK-001 | mutation-test | `contract_version: u32 = 1` is a high-leverage value-mutation target |
| VP-TYPES-001 | mutation-test | EXEMPT list length and attribute-presence check are mutation surfaces |
| VP-FACTORY-002 | fuzz | `parse_frontmatter_field` was the v1.2.3 (F-R20-2) regression site; permanent fuzz harness prevents recurrence |
| VP-PROTO-002 | fuzz (Phase 4 deferred) | Unknown schema-version dispatch must never panic across `u32::MAX` value space; Phase 4 harness only |

---

## §Open Verification Gaps

This section enumerates gaps where the BC catalog and the architecture
artifacts identify properties that are NOT formally verified by a Phase 1 VP.
Per CLAUDE.md §CANONICAL PRINCIPLE rule 3, each gap is anchored to a concrete
future story or wave — gaps are NOT generic deferrals.

### §G-1 — Kani Proof Harnesses for Phase 2 Trigger State Machine

**Status:** OPEN — pre-staged for Phase 2.

**Description:** Phase 1 BCs are deterministic protocol contracts that
require no model checking. Phase 2 introduces a `trigger-trace` state machine
whose invariants (no-deadlock, no-orphan-trigger, ring-buffer-monotonicity)
are natural Kani proof targets. This artifact does NOT pre-stage Kani
harnesses because the Phase 2 state-machine specification does not yet exist
(per STATE.md current phase `phase-1-spec-crystallization-entry-pending`).

**Future-attachment:** Phase 2 architecture artifact `SS-trigger-trace.md`
(to be authored during Phase 2 spec crystallization) MUST extend this
verification-properties catalog with Kani-based VPs. The Phase 2 PRD dispatch
explicitly enumerates this as a Phase 2 deliverable.

**Compensating Phase 1 coverage:** None required — there is no Phase 1
state machine. The Phase 1 daemon-lifecycle protocol is governed by
BC-DAEMON-004 (graceful shutdown), formally verified by VP-DAEMON-004
in this catalog with a `unit-test` mechanism; the daemon lifecycle is
small enough that exhaustive unit testing suffices.

### §G-2 — DTU Fidelity Scoring

**Status:** COVERED ELSEWHERE — see `dtu-assessment.md` §DTU Fidelity Measurement Procedure.

**Description:** DTU clone fidelity (target: ≥0.95 mean field-match score
against real Claude Code 2.x fixtures) is the verification path for the hook
protocol DTU clone. This is not a BC; it is a clone-quality measurement
governed by the `dtu-validator` agent.

**Future-attachment:** Wave 1 stories per `dtu-assessment.md` §Clone
Development Approach.

### §G-3 — Phase 4 OAuth2 / Federation Auth

**Status:** OUT OF PHASE 1 SCOPE.

**Description:** BC-AUTH-002 explicitly notes that Phase 4 federation OAuth2
tokens use a separate `Authorization: Bearer` header on a `monocle-ipc`
russh channel, NOT the Phase 1 `X-Monocle-Authorization` surface.
Verification of the Phase 4 federation auth path is a Phase 4 concern and
will receive its own VP (provisionally `VP-FED-AUTH-001`).

**Future-attachment:** Phase 4 architecture artifact (to be authored during
Phase 4 spec crystallization).

### §G-4 — `BC-DAEMON-001` through `BC-DAEMON-006` Verification

**Status:** RESOLVED in VP v1.1 + PRD v1.1. The v1.0 forward-projection
that this catalog "SHOULD be extended in a v1.1 revision" is now closed:
BC-DAEMON-001..006 are formalized as VP-DAEMON-001..006 in §Per-VP Detail,
traced 1:1 to PRD v1.1 commit f855835 §BC-DAEMON-001..006, and registered
in §Coverage Matrix.

**Description (historical):** The daemon endpoints (BC-DAEMON-001 through
BC-DAEMON-006) were pre-staged in `SS-daemon-lifecycle.md` but were NOT in
the 16-BC scope of the v1.0 VP catalog (the architect's initial task
allocation focused on the 16 cross-cutting type/auth/lock/ABI/factory/engine
BCs). The F-R62-1 finding (adversary R62, commit 5713ccc) identified that
this scoping created PRD forward-references to undefined BCs and a 22-BC
gap. The F-R62 fix-burst closed the gap on both sides: PRD v1.1 formalized
BC-DAEMON-001..006 as full contract sections, and this VP v1.1 catalogs
VP-DAEMON-001..006 with mechanical properties, post-conditions, and
counter-example sketches at parity with the original 16.

**Future-attachment:** No further future work — gap closed in this v1.1
revision.

### §G-5 — Phase 1 Permission Enum Match-Site Coverage

**Status:** COVERED ELSEWHERE — by ADR-0004 + SS-permissions-phase1.md +
clippy lint configuration.

**Description:** `Phase1Permission` is exhaustive per ADR-0004. Match-site
correctness (every dispatch site covers every variant) is enforced by the
Rust compiler at compile time — no VP is required because the property is
discharged by `cargo check`. The clippy lint
`non_exhaustive_omitted_patterns` deny-listed via `#[allow(...)]` is a
separate concern covered by VP-TYPES-001's mutation-test auxiliary.

**Future-attachment:** N/A — discharged by compiler.

---

## §References (PG-5 historical-anchor framing)

The following cross-artifact references use position-free §-anchors and
either current-pointer version pinning or version-free anchors per
`SS-conventions-anti-patterns.md` §Historical-Anchor Framing Convention
(PG-5). All version pins below are current as of timestamp
`2026-05-15T03:30:00Z`.

1. `.factory/specs/prd.md` v1.3 (commit d8e66c3) — canonical BC source for
   the 22 Phase 1 BCs in this catalog and canonical test-name + test-file
   path source. Anchors: BC sections BC-DAEMON-001 through BC-ENGINE-003
   (22 sub-§s under §Behavioral Contracts), §Section 7 Requirements
   Traceability Matrix (canonical test-file path source for F-R62-4
   closure, carried forward; canonical test-name source per §Trace v1.2
   for the 4 F-R63-adv-1 / F-R63-cons-1 adjudications; PRD v1.3 content
   unchanged from PRD v1.2 commit 5a49b0b — v1.3 is a pure arch v1.0.10
   pin propagation per R3-001 closure).
2. `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.10 — source of
   BC-DAEMON-001..006, BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001.
   Anchors: §Health and Status Endpoints (BC-DAEMON-001 + BC-DAEMON-002),
   §Body Size Limit (BC-DAEMON-003), §Daemon Lifecycle Protocol
   (BC-DAEMON-004 + BC-DAEMON-005 + BC-DAEMON-006), §Drain (BC-RING-001),
   §Start Sequence (BC-AUTH-001 + BC-AUTH-002 + BC-LOCK-001),
   §Behavioral Contract Summary (10-BC table; version-stable footer phrasing
   landed in arch v1.0.10 commit dc3af71 per R3-001 closure).
3. `.factory/specs/architecture/SS-core-types-and-abi.md` v1.2.8 — source of
   BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-FACTORY-001, BC-FACTORY-002,
   BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002. Anchors:
   §ABI Version Constant, §Enum Extensibility, §FactoryAdapter Trait,
   §Prost Wire Schemas, §Phase 1 PRD BC Pre-Staging.
4. `.factory/specs/architecture/SS-engine-module.md` v1.1.15 — source of
   BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003. Anchors:
   §EngineModule Trait Signature, §Behavioral Contracts, §Phase 1
   Implementation, §Struct-level inherent operations.
5. `.factory/specs/architecture/SS-conventions-anti-patterns.md` —
   §Historical-Anchor Framing Convention (PG-5),
   §Section-Anchor Citation Convention (PG-4),
   §Cross-Section Directional Reference Convention (PG-3),
   §Schema-Fact Citation Convention (PG-1),
   §Phantom-ID Convention (PG-2),
   §META-Rule Recipe Sibling-Pattern Convention (PG-RECIPE-SCOPE),
   §Semgrep Rules, §Test Conventions.
6. `.factory/specs/architecture/SS-deps-pin-manifest.md` — canonical pins for
   `constant_time_eq ^0.3`, `temp-env ^0.3` (features = ["async_closure"]),
   `prost 0.14`, `serde_yaml_ng 0.10`, `serde_json 1`, `tracing 0.1`,
   `tempfile 3`, `axum 0.8`, `tokio 1`, `nix 0.30`.
7. `.factory/specs/architecture/SS-permissions-phase1.md` — §Phase 1
   Permission Enum, §Exhaustiveness Invariant.
8. `.factory/specs/architecture/SS-forward-compatibility.md` — §Item P3-1
   (open-trait rationale referenced by VP-FACTORY-001 and VP-ENGINE-001).
9. `.factory/specs/dtu-assessment.md` — §DTU Architecture (hook protocol
   surface), §DTU Fidelity Measurement Procedure (§G-2 deferral target).
10. `.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md` — Phase 3
    wasmtime 44 selection (informs §G-1 future Phase 2/Phase 3 Kani harness
    scope).
11. `.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md`
    — nucleo 0.5 acceptance (no Phase 1 VP impact; referenced for
    completeness).
12. `.factory/specs/architecture/adr/ADR-0003-license-selection.md` —
    license posture (no Phase 1 VP impact; referenced for completeness).
13. `.factory/specs/architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md`
    — VP-TYPES-001 exemption authority. The EXEMPT set
    `{Phase1Permission, ClaudeCodeTool}` is normatively defined here.
14. `CLAUDE.md` — §CANONICAL PRINCIPLE — Production-Grade Default,
    §Correct Agent Routing — The Production-Grade Companion Principle.

---

## §Trace

v1.3 (R3-001 closure pin propagation, 2026-05-15):

- **Trigger:** R3-001 closure chain produced two upstream version bumps that
  required propagation into VP. Consistency-validator round 3 commit ba62a15
  raised R3-001 (GAPS verdict) identifying that arch v1.0.9 §BC Summary
  footer tense oscillated across past version bumps. Architect committed
  SS-daemon-lifecycle.md v1.0.10 (commit dc3af71) with version-stable §BC
  Summary footer phrasing — content unchanged, prevents future oscillation.
  Product-owner committed PRD v1.3 (commit d8e66c3) propagating the arch
  v1.0.10 pin through 31 normative-current citation sites — PRD content
  unchanged from v1.2 commit 5a49b0b. Per L-F-R63-PARTIAL-FIX (cycle-001
  lessons §META process-gap codification), the orchestrator dispatched this
  VP burst with an explicit propagation checklist enumerating every
  citation site VP touches: frontmatter `inputs:`/`traces_to:`, §Purpose
  opening, §Scope, §VP Catalog Overview table, per-VP `Traces to:` lines,
  per-VP `Test name:` annotation citations, §Coverage Matrix table and
  footer, §References items 1 and 2. This v1.3 catalog propagates BOTH
  version bumps with NO content changes — pin-only burst.
- **Arch v1.0.9 → v1.0.10 propagation (32 normative-current sites):**
  §Scope `SS-daemon-lifecycle.md v1.0.10` (1); §VP Catalog Overview Source
  column for VP-DAEMON-001..006 (6) + VP-RING-001/AUTH-001/AUTH-002/LOCK-001
  (4); per-VP `Traces to:` for VP-DAEMON-001..006 (6) + VP-AUTH-002 (1);
  §Coverage Matrix BC Source File column for the same 10 rows (10);
  §Coverage Matrix footer arch citation (1); §References item 2 current
  pointer (1) — totals 30 row/sentence-level sites and 32 occurrences
  (some rows contain the pin twice in joined form). Historical mentions
  preserved per PG-5: line 25 frontmatter `traces_to` history bookkeeping,
  line 62 §Purpose v1.2 narrative ("propagates the v1.0.8 → v1.0.9 bump"),
  line 826 VP-AUTH-002 historical fix-provenance ("F-R62-4 back-propagation
  closure landed in arch v1.0.9 commit 8bf3759"), §Coverage Matrix footer
  historical fix-provenance, and §Trace v1.2 narrative entries — all
  unchanged.
- **PRD v1.2 → v1.3 propagation (42 normative-current sites):**
  §Purpose opening (1); §Scope (1); §VP Catalog Overview Source column
  for VP-DAEMON-001..006 (6); per-VP `Traces to:` for VP-DAEMON-001..006 +
  VP-AUTH-002 + VP-PROTO-002 (8); per-VP `Test name:` annotations across
  all 22 VPs that cite PRD (20); §VP-PROTO-002 "Per PRD §Section 7 RTM"
  and Phase 4 Test name annotation (3); §Coverage Matrix BC Source File
  column for VP-DAEMON-001..006 (6); §Coverage Matrix footer matches-PRD
  claim (2); §References item 1 (1). Historical mentions preserved per
  PG-5: line 25 frontmatter `traces_to` history bookkeeping, lines 58/61
  §Purpose v1.2 narrative, line 1368 §VP-PROTO-002 Reframing rationale
  (F-R62-7) historical, lines 1650/1656/1720 §Test design / hybrid-name
  citations referring to v1.2 adjudication record, line 1753 §Coverage
  Matrix footer historical "content unchanged from v1.2 commit 5a49b0b",
  line 1878 §References item 1 historical "PRD v1.3 content unchanged
  from PRD v1.2 commit 5a49b0b" — all unchanged. All §Trace v1.2 narrative
  entries unchanged (~80 lines).
- **Frontmatter bump:** `version: "1.2"` → `version: "1.3"`. `timestamp`
  bumped to `2026-05-15T03:30:00Z`. `inputs:` list paths unchanged (file
  paths only, no version pins per PG-5 §Frontmatter Carve-Out Option B).
  `traces_to:` rewritten to reflect the R3-001 closure chain (ba62a15 +
  dc3af71 + d8e66c3), the L-F-R63-PARTIAL-FIX lesson application, the
  pin-only nature of the burst, and the preserved prior-burst context
  for v1.2 and v1.1. `input-hash` remains `[live-state]` (computed by
  pre-commit hook).
- **PG-3 directional compliance (R3-001 closure context):** no
  `above`/`below`/L-number qualifiers used in this §Trace v1.3 entry. All
  §-anchor references position-free.
- **PG-4 §-heading-existence sweep — REAL (not falsified):** all §-anchor
  references introduced or modified in v1.3 changes resolve to actual
  headings. Sweep performed by greppable substring match against the
  pinned source-of-truth files:
  - PRD v1.3 (commit d8e66c3) §BC-DAEMON-001..006 — PASS (each `### BC-DAEMON-NNN`).
  - PRD v1.3 §BC-RING-001 — PASS.
  - PRD v1.3 §BC-AUTH-001 — PASS.
  - PRD v1.3 §BC-AUTH-002 — PASS.
  - PRD v1.3 §BC-LOCK-001 — PASS.
  - PRD v1.3 §BC-ABI-001 — PASS.
  - PRD v1.3 §BC-ABI-002 — PASS.
  - PRD v1.3 §BC-TYPES-001 — PASS.
  - PRD v1.3 §BC-FACTORY-001 — PASS.
  - PRD v1.3 §BC-FACTORY-002 — PASS.
  - PRD v1.3 §BC-PROTO-001a — PASS.
  - PRD v1.3 §BC-PROTO-001b — PASS.
  - PRD v1.3 §BC-PROTO-002 — PASS.
  - PRD v1.3 §BC-ENGINE-001 — PASS.
  - PRD v1.3 §BC-ENGINE-002 — PASS.
  - PRD v1.3 §BC-ENGINE-002-ERR — PASS.
  - PRD v1.3 §BC-ENGINE-003 — PASS.
  - PRD v1.3 §Section 7 RTM — PASS (`## 7. Requirements Traceability Matrix`).
  - SS-daemon-lifecycle.md v1.0.10 (commit dc3af71) §Health and Status
    Endpoints — PASS.
  - SS-daemon-lifecycle.md v1.0.10 §Body Size Limit — PASS.
  - SS-daemon-lifecycle.md v1.0.10 §Daemon Lifecycle Protocol — PASS.
  - SS-daemon-lifecycle.md v1.0.10 §Shutdown Signal Handling — PASS.
  - SS-daemon-lifecycle.md v1.0.10 §Drain — PASS.
  - SS-daemon-lifecycle.md v1.0.10 §Start Sequence — PASS.
  - SS-daemon-lifecycle.md v1.0.10 §Hard Shutdown — PASS.
  - SS-daemon-lifecycle.md v1.0.10 §Crash Recovery — PASS.
  - SS-daemon-lifecycle.md v1.0.10 §Behavioral Contract Summary — PASS
    (version-stable footer phrasing landed in commit dc3af71).
  - No new §-anchor references introduced for SS-core-types-and-abi.md,
    SS-engine-module.md, SS-permissions-phase1.md, SS-forward-compatibility.md,
    SS-conventions-anti-patterns.md, dtu-assessment.md, or CLAUDE.md in
    v1.3 changes. Their v1.2/v1.1 PG-4 sweep entries remain valid (no
    version bumps to these inputs since v1.2).
- **PG-5 historical-anchor compliance:** all current-pointer version pins
  in normative content updated atomically (PRD v1.3, arch v1.0.10).
  §References intro timestamp bumped to `2026-05-15T03:30:00Z`. Frontmatter
  `inputs` list uses version-free file paths per PG-5 §Frontmatter Carve-Out
  Option B. §Trace v1.2 historical entries preserved verbatim — v1.2
  citations of v1.0.9/PRD v1.2 are correct for the state at v1.2 authoring
  time. §G-4 historical resolution narrative ("RESOLVED in VP v1.1 + PRD
  v1.1") preserved. Historical fix-provenance citations within normative
  sections (line 826 VP-AUTH-002, line 1753 §Coverage footer, line 1878
  §References item 1) preserved — these record what was true at past
  fix-points and remain accurate.
- **PG-2 count coherence (v1.3):** 22 VPs unchanged (no VP added,
  retired, or renumbered). 22 `**Test name:**` lines unchanged in count
  (21 active + 1 explicit Phase 4-deferred for VP-PROTO-002). Mechanism
  distribution unchanged (22 unit-test primary, 5 fuzz auxiliary, 4
  mutation-test auxiliary, 0 Kani). Auxiliary mechanism coverage table
  unchanged (9 entries). Coverage matrix unchanged (22 rows). §G-1..§G-5
  status unchanged (§G-4 still RESOLVED, §G-1/§G-3 still OPEN/OUT-OF-SCOPE,
  §G-2/§G-5 still COVERED ELSEWHERE). Frontmatter `traces_to:` updated to
  reflect v1.3 burst.
- **F-R60-corpus-sweep (v1.3):** zero `v1.0.9` citations remain as
  normative-current pointers; the remaining v1.0.9 occurrences in normative
  sections are explicit historical fix-provenance citations (commit
  8bf3759 contexts) preserved per PG-5. Zero `PRD v1.2` citations remain
  as normative-current pointers; the remaining PRD v1.2 occurrences are
  in historical narrative or fix-provenance contexts preserved per PG-5.
  Zero `5a49b0b` citations remain as normative-current pointers (PRD
  current pin is now `d8e66c3`); historical citations of `5a49b0b`
  preserved as fix-provenance record per PG-5.
- **§Trace-Heading-Convention:** this §Trace v1.3 sub-block sits under
  the same `## §Trace` parent heading (matching the v1.2 and v1.1 entry
  pattern).
- **BC-H1-is-title-source-of-truth:** document title `# Verification
  Properties: Phase 1 Behavioral Contract Catalog` unchanged across
  v1.2 → v1.3.
- **append_only_numbering:** no VP added, retired, or renumbered. The
  22-VP catalog identifier set is identical to v1.2. Only version-pin
  citations changed; no content changes whatsoever.
- **VP-PROTO-002 Phase-4-only carve-out preserved:** VP-PROTO-002's
  classification as Phase-4-deferred-only (no Phase 1 test name, no
  Phase 1 harness) is unchanged — only the citation pin `PRD v1.2 §BC-PROTO-002`
  was bumped to `PRD v1.3 §BC-PROTO-002` in the Phase-4-deferred
  declaration line.
- **Frozen META catalog status (D-054):** F-R55-adv-1, F-R55-adv-3,
  F-R61-adv-1, F-R61-2 — none reintroduced in v1.3 changes. Frozen-residual
  discipline preserved.
- **Self-audit checklist (CLAUDE.md §CANONICAL PRINCIPLE):**
  - No MVP / for-now / good-enough / fix-later rationalizations. PASS.
  - No tech-debt-register entries added. PASS.
  - No "TODO for architect" / "pending architect review" placeholders.
    The architect's R3-001 closure is COMPLETE (commit dc3af71); the
    product-owner's pin propagation is COMPLETE (commit d8e66c3); this
    catalog consumes both as faits accomplis. PASS.
  - No silent fix outside formal-verifier scope. PRD content edits are
    product-owner work; arch content edits are architect work; state
    edits are state-manager work; this burst touched only VP. PASS.
  - No cheap-mechanism defaults. Every citation site swept via explicit
    propagation checklist per L-F-R63-PARTIAL-FIX; no implicit "fix the
    obvious thing and call it done" shortcut. PASS.
  - No advisory-severity downgrades. R3-001 was treated as the BLOCKER
    that the consistency-validator round 3 verdict identified; closure
    chain executed end-to-end (architect + product-owner + this VP
    burst). PASS.
- **Production-grade default:** every version pin in normative content
  matches the current source-of-truth (arch v1.0.10 commit dc3af71, PRD
  v1.3 commit d8e66c3). Zero MVP shortcuts, zero falsified self-checks.
  The §Trace v1.3 narrative is internally consistent with the propagation
  count claims (32 arch sites, 42 PRD sites) and the historical-pin
  preservation set is enumerated explicitly. The L-F-R63-PARTIAL-FIX
  recurrence guard is honored: this burst's dispatch prompt enumerated
  every artifact dimension to sweep, and this §Trace v1.3 entry documents
  the application of that checklist.
- **Correct agent routing (CLAUDE.md companion principle):** VP catalog
  body — formal-verifier scope (this burst). PRD — product-owner scope
  (not touched; v1.3 commit d8e66c3 is the product-owner's pin propagation
  deliverable). Architecture (SS-daemon-lifecycle.md) — architect scope
  (not touched; v1.0.10 commit dc3af71 is the architect's R3-001 closure
  deliverable). STATE.md — state-manager scope (not touched; runs after
  this burst). consistency-audit round 3 (ba62a15) — consistency-validator
  scope (not touched; the verdict triggered the closure chain). No
  cross-domain silent edits.

v1.2 (F-R63 fix-burst, 2026-05-15):

- **Trigger:** F-R63 fix-burst combining adversary R63 commit 11a98c4
  (F-R63-adv-1 HIGH test-name drift, F-R63-adv-2 MEDIUM arch back-propagation)
  and consistency-validator round 2 commit 200eb68 (F-R63-cons-1 HIGH
  13 test-name gaps = 4 mismatches + 10 missing; F-R63-cons-2 MEDIUM error-count
  narrative; F-R63-cons-3 MEDIUM arch staleness). Architect committed
  SS-daemon-lifecycle.md v1.0.9 (commit 8bf3759) closing F-R63-adv-2 +
  F-R63-cons-3 (F-R62-4 back-propagation, BC Summary tense). Product-owner
  committed PRD v1.2 (commit 5a49b0b) closing F-R63-adv-1 (4 test-name
  adjudications) and F-R63-cons-2 (error count narrative correction). This
  v1.2 catalog closes the formal-verifier-owned residuals: F-R63-adv-1
  4-name reconciliation in the VP catalog body, F-R63-cons-1 10-missing
  Test name lines added, and propagates the arch v1.0.9 + PRD v1.2 pins.
- **F-R63-adv-1 closure (HIGH) — 4 test-name reconciliations against PRD v1.2:**
  - **BC-ABI-001:** VP v1.1 already used `test_BC_ABI_001_status_endpoint_returns_abi_version_1`
    verbatim. Product-owner adjudication adopted the VP name as canonical
    (per PRD v1.2 §Trace v1.2: "the VP name identifies the endpoint
    (`status`), the field, and the expected value (`1`), making the
    assertion self-documenting for both presence and value"). VP v1.2:
    no test-name change required; annotation citation bumped to PRD v1.2.
  - **BC-ENGINE-002:** VP v1.1 already used `test_BC_ENGINE_002_claude_code_module_strict_basename_detect`
    verbatim. Product-owner adjudication adopted the VP name as canonical
    (per PRD v1.2 §Trace v1.2: "`strict_basename` encodes the critical
    behavioral invariant — the test distinguishes correct strict-basename
    matching from a naive prefix or substring match"). VP v1.2: no
    test-name change required; annotation citation bumped to PRD v1.2.
  - **BC-ENGINE-002-ERR:** VP v1.1 used `_sync_and_async`; product-owner
    adjudication rejected this name in favor of the PRD-original
    `_metadata_and_enrich` (per PRD v1.2 §Trace v1.2: "`_metadata_and_enrich`
    identifies WHAT behavioral methods are under contract (both `metadata()`
    and `enrich()` must return `Err(HomeUnresolvable)`); `_sync_and_async`
    describes a test-implementation strategy . . . which is an internal
    concern of the test author, not the behavioral discriminator. Test
    names should describe what is verified, not how the test harness is
    structured."). VP v1.2: test name updated to
    `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich`. A new
    "Test design" paragraph was added to the VP body documenting that the
    `metadata()` (sync) and `enrich()` (async) wrappers via
    `temp_env::with_vars` / `temp_env::async_with_vars` are the
    implementation strategy for verifying both behavioral surfaces — not
    the behavioral discriminator. The property and post-conditions remain
    unchanged from v1.1; both `metadata()` and `enrich()` must return
    `Err(EngineMetadataError::HomeUnresolvable)` when all four home-env
    vars are unset.
  - **BC-ENGINE-003:** VP v1.1 used `_claude_module_inherent_hook_paths`;
    product-owner adjudication produced a hybrid (per PRD v1.2 §Trace v1.2:
    "Neither PRD name (`_hook_paths_five_entries`) nor VP name
    (`_claude_module_inherent_hook_paths`) alone was sufficient. The
    hybrid `_claude_module_hook_paths_five_entries` combines: (1)
    `claude_module` — identifies the concrete struct under test (not a
    trait), (2) `hook_paths` — identifies the inherent method, (3)
    `five_entries` — states the count assertion. This is the most
    self-documenting form per Rust integration-test naming conventions.").
    VP v1.2: test name updated to
    `test_BC_ENGINE_003_claude_module_hook_paths_five_entries`.
- **F-R63-cons-1 closure (HIGH) — 10 missing `**Test name:**` lines added:**
  prior to v1.2, 11 VPs had explicit `Test name:` annotations and 11 did
  not (BC-PROTO-002 is Phase 4-deferred so legitimately has none). The
  asymmetry was a documentation gap. The following 10 VPs gained explicit
  `Test name:` lines in v1.2, with each name sourced verbatim from PRD v1.2
  §Section 7 RTM and the corresponding BC §Verification subsection:
  - VP-RING-001 → `test_BC_RING_001_format_version_first_key`
  - VP-AUTH-001 → `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`
  - VP-LOCK-001 → `test_BC_LOCK_001_contract_version_first_key`
  - VP-ABI-002 → `test_BC_ABI_002_abi_version_const_exported`
  - VP-TYPES-001 → `test_BC_TYPES_001_non_exhaustive_enum_coverage`
  - VP-FACTORY-001 → `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound`
  - VP-FACTORY-002 → `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection`
  - VP-PROTO-001a → `test_BC_PROTO_001a_schema_version_field_number_1`
  - VP-PROTO-001b → `test_BC_PROTO_001b_schema_version_rust_field`
  - VP-ENGINE-001 → `test_BC_ENGINE_001_trait_defined_all_methods_no_sealed_bound`

  VP-PROTO-002 additionally received a `**Test name:**` line explicitly
  marking it as Phase 4-deferred (no Phase 1 test name) with the Phase 4
  name documented per PRD v1.2 §BC-PROTO-002 for symmetry. Total `**Test
  name:**` lines after v1.2: 22/22 (was 11/22 in v1.1).
- **Version-pin propagation (architecture v1.0.8 → v1.0.9 and PRD v1.1 →
  v1.2):** every cross-artifact citation in normative content (frontmatter
  `inputs:`, frontmatter `traces_to:`, §Purpose, §Scope, §VP Catalog
  Overview "BC Source" column, §Per-VP Detail "Traces to" lines and "Test
  name" annotation citations, §Coverage Matrix "BC Source File" column,
  §Coverage Matrix coverage footer, §References item 1, §References item
  2, §References intro timestamp) was updated. The PRD v1.2 commit `5a49b0b`
  added the test-name adjudications and the SS-daemon-lifecycle.md v1.0.8 →
  v1.0.9 propagation; the arch v1.0.9 commit `8bf3759` is the F-R62-4
  back-propagation closure (auth test path split, BC Summary tense
  correction). §Trace v1.1 historical entries are NOT modified — they
  record the state at v1.1 authoring time per PG-5 historical-anchor
  framing convention.
- **VP-PROTO-002 reframing-rationale citation:** the v1.1 paragraph in
  §VP-PROTO-002 §Reframing rationale (F-R62-7) cited "PRD v1.1
  §BC-PROTO-002 explicitly classifies the runtime test as Phase 4." Bumped
  to PRD v1.2 with an explicit note that BC-PROTO-002 was unchanged across
  PRD v1.1 → v1.2 (classification preserved). The reframing rationale itself
  is unchanged.
- **PG-4 §-heading-existence sweep — REAL (not falsified):** all §-anchor
  references introduced or modified in v1.2 changes resolve to actual
  headings:
  - PRD v1.2 §BC-DAEMON-001..006 — PASS (each resolves to `### BC-DAEMON-NNN`).
  - PRD v1.2 §BC-RING-001 — PASS.
  - PRD v1.2 §BC-AUTH-001 — PASS.
  - PRD v1.2 §BC-AUTH-002 — PASS.
  - PRD v1.2 §BC-LOCK-001 — PASS.
  - PRD v1.2 §BC-ABI-001 — PASS.
  - PRD v1.2 §BC-ABI-002 — PASS.
  - PRD v1.2 §BC-TYPES-001 — PASS.
  - PRD v1.2 §BC-FACTORY-001 — PASS.
  - PRD v1.2 §BC-FACTORY-002 — PASS.
  - PRD v1.2 §BC-PROTO-001a — PASS.
  - PRD v1.2 §BC-PROTO-001b — PASS.
  - PRD v1.2 §BC-PROTO-002 — PASS.
  - PRD v1.2 §BC-ENGINE-001 — PASS.
  - PRD v1.2 §BC-ENGINE-002 — PASS.
  - PRD v1.2 §BC-ENGINE-002-ERR — PASS.
  - PRD v1.2 §BC-ENGINE-003 — PASS.
  - PRD v1.2 §Section 7 RTM — PASS (resolves to `## 7. Requirements
    Traceability Matrix`).
  - PRD v1.2 §Trace v1.2 — PASS (resolves to `## §Trace v1.2`).
  - SS-daemon-lifecycle.md v1.0.9 §Health and Status Endpoints — PASS.
  - SS-daemon-lifecycle.md v1.0.9 §Body Size Limit — PASS.
  - SS-daemon-lifecycle.md v1.0.9 §Daemon Lifecycle Protocol — PASS.
  - SS-daemon-lifecycle.md v1.0.9 §Drain — PASS.
  - SS-daemon-lifecycle.md v1.0.9 §Start Sequence — PASS.
  - SS-daemon-lifecycle.md v1.0.9 §Hard Shutdown — PASS.
  - SS-daemon-lifecycle.md v1.0.9 §Crash Recovery — PASS.
  - SS-daemon-lifecycle.md v1.0.9 §Behavioral Contract Summary — PASS.
  - No new §-anchor references introduced for SS-core-types-and-abi.md,
    SS-engine-module.md, SS-permissions-phase1.md, SS-forward-compatibility.md,
    SS-conventions-anti-patterns.md, dtu-assessment.md, or CLAUDE.md in
    v1.2 changes. Their v1.1 PG-4 sweep entries remain valid (no version
    bumps to these inputs since v1.1).
- **PG-2 count coherence (v1.2):** 22 VPs unchanged (no VP added or
  retired). 22 `**Test name:**` lines now present (21 active + 1 explicit
  Phase 4-deferred for VP-PROTO-002) — was 11 in v1.1; the 11-line gap
  catalogued by F-R63-cons-1 is closed. Mechanism distribution unchanged
  (22 unit-test primary, 5 fuzz auxiliary, 4 mutation-test auxiliary, 0
  Kani; per §VP Catalog Overview table). Auxiliary mechanism coverage
  table unchanged (9 entries). Coverage matrix unchanged (22 rows).
  Frontmatter `traces_to:` updated to reflect v1.2 burst.
- **PG-3 directional compliance:** no `above`/`below`/L-number qualifiers
  in v1.2 §Trace entry or in any v1.2-modified normative content. §-anchor
  references include position-free descriptions where needed (e.g., "§Trace
  v1.2", "§Section 7 RTM").
- **PG-3-TRACE-NEW-ENTRY:** this §Trace v1.2 entry uses only §-section
  references and commit SHAs (`5a49b0b`, `8bf3759`, `11a98c4`, `200eb68`,
  `2db408f`, `f855835`, `0e322da`, `5713ccc`, `8454ff2`). No bare L-numbers.
  No directional qualifiers.
- **PG-5 historical-anchor compliance:** §Trace v1.1 historical entries
  preserve their original v1.0.7/v1.0.8/v1.1 citations as a record of the
  state at v1.1 authoring time. The §G-4 historical resolution narrative
  ("RESOLVED in VP v1.1 + PRD v1.1") is preserved per PG-5 — the gap closed
  at the v1.1/PRD-v1.1 boundary; the v1.2 burst does not re-open or
  re-resolve §G-4. Current-pointer pins (PRD v1.2 commit 5a49b0b,
  SS-daemon-lifecycle.md v1.0.9 commit 8bf3759, SS-core-types-and-abi.md
  v1.2.8, SS-engine-module.md v1.1.15) all current as of timestamp
  `2026-05-15T01:00:00Z`.
- **F-R60-corpus-sweep (v1.2):** zero `v1.0.8` citations remain in normative
  content (current-pointer surfaces are all v1.0.9; historical §G-4 and
  §Trace v1.1 entries retain v1.0.8 references as historical record per
  PG-5). Zero `f855835` citations remain in normative content (current
  PRD pin is `5a49b0b`; historical §G-4 and §Trace v1.1 entries retain
  `f855835` references as historical record per PG-5). Zero stale test
  names from the pre-adjudication PRD/VP draft set (`_status_abi_version_field`,
  `_claude_code_module_detect`, `_hook_paths_five_entries`,
  `_home_unresolvable_sync_and_async`, `_claude_module_inherent_hook_paths`)
  remain in normative content (the 4 adjudicated canonical names are the
  sole authority; the rejected/superseded forms appear only in this §Trace
  v1.2 entry as historical context).
- **§Trace-Heading-Convention:** this §Trace v1.2 sub-block sits under the
  same `## §Trace` parent heading (matching the v1.1 entry pattern).
- **BC-H1-is-title-source-of-truth:** document title
  `# Verification Properties: Phase 1 Behavioral Contract Catalog` unchanged
  across v1.1 → v1.2.
- **append_only_numbering:** no VP added, retired, or renumbered. The 22-VP
  catalog identifier set is identical to v1.1. Only the test-name
  annotation strings and version-pin citations changed.
- **Frozen META catalog status (D-054):** F-R55-adv-1, F-R55-adv-3,
  F-R61-adv-1, F-R61-2 — none reintroduced in v1.2 changes. Frozen-residual
  discipline preserved.
- **Self-audit checklist (CLAUDE.md §CANONICAL PRINCIPLE):**
  - No MVP / for-now / good-enough / fix-later rationalizations. PASS.
  - No tech-debt-register entries added. PASS.
  - No "TODO for architect" / "pending architect review" placeholders.
    The product-owner adjudication is COMPLETE (commit 5a49b0b); the
    architect back-propagation is COMPLETE (commit 8bf3759); this catalog
    consumes both as faits accomplis. PASS.
  - No silent fix outside formal-verifier scope. The 4 test-name
    adjudications were product-owner work (commit 5a49b0b); this catalog
    adopts them. The arch back-propagation was architect work (commit
    8bf3759); this catalog reads the new arch version. PRD-side narrative
    corrections (F-R63-cons-2 error count) are product-owner work; not
    touched here. STATE.md is state-manager scope; not touched here. PASS.
  - No cheap-mechanism defaults. Test name additions sourced verbatim from
    PRD v1.2 RTM rather than inferred; arch v1.0.9 propagation explicit
    across every citation site. PASS.
  - No advisory-severity downgrades. F-R63-adv-1 (HIGH) and F-R63-cons-1
    (HIGH) treated as BLOCKERS and fixed in scope. PASS.
- **Production-grade default:** every test-name reconciliation is
  definitive (not deferred); every missing `**Test name:**` line is
  populated with the canonical PRD-sourced name; every version pin in
  normative content matches the current source-of-truth. Zero MVP
  shortcuts, zero falsified self-checks.
- **Correct agent routing (CLAUDE.md companion principle):** VP catalog
  body — formal-verifier scope (this burst). PRD — product-owner scope
  (not touched; v1.2 commit 5a49b0b is the product-owner's adjudication
  deliverable). Architecture (SS-daemon-lifecycle.md) — architect scope
  (not touched; v1.0.9 commit 8bf3759 is the architect's deliverable).
  STATE.md — state-manager scope (not touched; runs after this burst).
  No cross-domain silent edits.

v1.1 (F-R62 fix-burst, 2026-05-14):

- **F-R62-1 closure (CRITICAL):** expanded VP catalog from 16 → 22 VPs.
  Added VP-DAEMON-001..006 with full mechanical properties, pre/post-conditions,
  counter-example sketches, and harness locations in §Per-VP Detail in PRD
  §7. Requirements Traceability Matrix row order. Each VP-DAEMON-NNN traces
  to the corresponding PRD v1.2 §BC-DAEMON-NNN section. §VP Catalog Overview
  table, §Mechanism Distribution table, §Coverage Matrix table, and §Purpose
  count claim all updated to 22 VPs. §G-4 status updated RESOLVED.
- **F-R62-4 closure (HIGH):** test-file paths reconciled with PRD v1.1
  §7. Requirements Traceability Matrix verbatim. 6 path changes against v1.0:
  VP-ABI-001 `status_endpoint.rs` → `status_abi_version.rs`;
  VP-FACTORY-001 `factory_adapter_surface.rs` → `factory_trait_surface.rs`;
  VP-ENGINE-002 `engine_module.rs` → `engine_module_claude_detect.rs`;
  VP-ENGINE-002-ERR `engine_module.rs` → `engine_module_home_unresolvable.rs`;
  VP-ENGINE-003 `engine_module.rs` → `engine_module_claude_methods.rs`;
  VP-PROTO-002 `dispatch_unknown_version.rs` → Phase 4 (no Phase 1 harness).
  All 22 VP harness paths now match PRD v1.1 RTM exactly.
- **F-R62-5 closure (HIGH):** frontmatter `phase` changed from
  `pre-phase-1-architecture` → `phase-1-spec-crystallization` (matching the
  PRD's frontmatter); `status` changed from `complete` → `draft` (the catalog
  is `draft` until the human Phase 1 approval gate fires, regardless of
  §G-4's closure).
- **F-R62-7 closure (MED):** VP-PROTO-002 reframed to remove the v1.0
  fabrication of a Phase 1 `monocle-proto::dispatch_envelope` function and
  `DispatchError` type. The Phase 1 verification is now an explicit
  structural recap of VP-PROTO-001a + VP-PROTO-001b (no new Phase 1 code
  surface); the runtime warn-and-skip behavior is documented as a Phase 4
  deliverable bound to a future `monocle-ipc` crate (no Phase 1 harness).
  Counter-example sketches split into Phase 1 (structural) and Phase 4
  (runtime). §Coverage Matrix records VP-PROTO-002's Phase 1 test file as
  "Phase 4 (no Phase 1 harness)" matching the PRD RTM.
- **F-R62-8 closure (MED) per architect adjudication commit 2db408f:**
  VP-AUTH-002 updated to the new two-body taxonomy. Mechanical property
  rewritten: absent header → `{"error":"missing_auth_token"}`; any
  value-present failure (bad prefix, bad format, secret mismatch) →
  `{"error":"invalid_auth_token"}` (collapsed). 6-probe post-condition table
  authored to exercise all failure modes plus the positive control. Retired
  `invalid_auth_token_format` body explicitly forbidden by counter-example
  sketch 3 ("Auth middleware returns the retired `invalid_auth_token_format`
  body for probe 2/3/4 — fails the exact-body assertion"). VP-AUTH-001's
  shared fuzz harness `fuzz_auth_token_validation` updated to assert the
  new 2-body taxonomy and to assert the retired body string never appears
  in any response.
- **F-R62-9 closure (LOW):** §G-4 status updated from "SCOPED — covered by
  Phase 1 PRD verification-harness stubs" (stale v1.0 forward-projection)
  to "RESOLVED in VP v1.1 + PRD v1.1" with description of the actual
  closure path.
- **Frontmatter `inputs` updated** to add the PRD as a canonical input.
  `traces_to` rewritten to document the 22-BC source decomposition,
  the F-R62 fix-burst commits (5713ccc adversary, 2db408f architect,
  f855835 PRD, 0e322da consistency), and the v1.1 closures.
- **§References updated:** added PRD as item 1 (canonical BC source);
  SS-daemon-lifecycle.md bumped v1.0.7 → v1.0.8 (architect's new BC-AUTH-002
  taxonomy); SS-deps-pin-manifest pin list expanded with `tempfile 3`,
  `axum 0.8`, `tokio 1`, `nix 0.30` (used by new VP-DAEMON-* harnesses).
- **PG-4 §-heading-existence sweep — REAL (not falsified):**
  - SS-daemon-lifecycle.md §Health and Status Endpoints — PASS (prefix-match
    to `## Health and Status Endpoints (F-NEW-05)`).
  - SS-daemon-lifecycle.md §Body Size Limit — PASS (prefix-match to
    `## Body Size Limit (F-NEW-06)`).
  - SS-daemon-lifecycle.md §Daemon Lifecycle Protocol — PASS (prefix-match
    to `## Daemon Lifecycle Protocol (F-NEW-09)`).
  - SS-daemon-lifecycle.md §Shutdown Signal Handling — PASS
    (`### Shutdown Signal Handling`).
  - SS-daemon-lifecycle.md §Drain — PASS (prefix-match to
    `### Drain (10-Second Timeout)`).
  - SS-daemon-lifecycle.md §Start Sequence — PASS (`### Start Sequence`).
  - SS-daemon-lifecycle.md §Hard Shutdown — PASS (`### Hard Shutdown`).
  - SS-daemon-lifecycle.md §Crash Recovery — PASS (`### Crash Recovery`).
  - SS-daemon-lifecycle.md §Behavioral Contract Summary — PASS
    (`## Behavioral Contract Summary`).
  - SS-core-types-and-abi.md §ABI Version Constant — PASS (prefix-match to
    `## §ABI Version Constant (FC-03 resolution)`).
  - SS-core-types-and-abi.md §Enum Extensibility — PASS (prefix-match to
    `## §Enum Extensibility — \`#[non_exhaustive]\` Markers (FC-02 resolution)`).
  - SS-core-types-and-abi.md §FactoryAdapter Trait — PASS (prefix-match to
    `## §FactoryAdapter Trait (FC-04 resolution — CRITICAL)`).
  - SS-core-types-and-abi.md §Prost Wire Schemas — PASS (prefix-match to
    `## §Prost Wire Schemas (FC-05 resolution)`).
  - SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging — PASS
    (`## §Phase 1 PRD BC Pre-Staging`).
  - SS-core-types-and-abi.md §Forward Compatibility Guarantees — PASS
    (`## §Forward Compatibility Guarantees`).
  - SS-engine-module.md §EngineModule Trait Signature — PASS
    (`## §EngineModule Trait Signature`).
  - SS-engine-module.md §Behavioral Contracts — PASS
    (`## §Behavioral Contracts`).
  - SS-engine-module.md §Phase 1 Implementation — PASS (prefix-match to
    `## §Phase 1 Implementation: \`ClaudeCodeModule\``).
  - SS-engine-module.md §Struct-level inherent operations — PASS
    (prefix-match to `### Struct-level inherent operations (NOT trait methods)`).
  - SS-conventions-anti-patterns.md §Historical-Anchor Framing Convention —
    PASS (prefix-match to `## §Historical-Anchor Framing Convention (PG-5)`).
  - SS-conventions-anti-patterns.md §Section-Anchor Citation Convention —
    PASS (prefix-match to `## §Section-Anchor Citation Convention (PG-4)`).
  - SS-conventions-anti-patterns.md §Cross-Section Directional Reference
    Convention — PASS (`## Cross-Section Directional Reference Convention`).
  - SS-conventions-anti-patterns.md §Schema-Fact Citation Convention — PASS
    (`## Schema-Fact Citation Convention`).
  - SS-conventions-anti-patterns.md §Phantom-ID Convention — PASS
    (`## Phantom-ID Convention`).
  - SS-conventions-anti-patterns.md §META-Rule Recipe Sibling-Pattern
    Convention — PASS (prefix-match to
    `## §META-Rule Recipe Sibling-Pattern Convention (PG-RECIPE-SCOPE)`).
  - SS-conventions-anti-patterns.md §Semgrep Rules — PASS
    (`### Semgrep Rules`).
  - SS-conventions-anti-patterns.md §Test Conventions — PASS
    (`## Test Conventions`).
  - SS-permissions-phase1.md §Phase 1 Permission Enum — PASS
    (`## Phase 1 Permission Enum`).
  - SS-permissions-phase1.md §Exhaustiveness Invariant — PASS
    (`### Exhaustiveness Invariant`).
  - SS-forward-compatibility.md §Item P3-1 — PASS (prefix-match to
    `#### Item P3-1: \`monocle-core\` trait stability for WASM ABI`).
  - dtu-assessment.md §DTU Architecture — PASS (`## DTU Architecture`).
  - dtu-assessment.md §DTU Fidelity Measurement Procedure — PASS
    (`## DTU Fidelity Measurement Procedure`).
  - dtu-assessment.md §Clone Development Approach — PASS
    (`## Clone Development Approach`).
  - CLAUDE.md §CANONICAL PRINCIPLE — PASS (`## CANONICAL PRINCIPLE — Production-Grade Default`,
    prefix-match exempt under PG-4 §Scope clause for non-versioned project
    documentation with unambiguous enumerated items).
  - CLAUDE.md §Correct Agent Routing — PASS (`## Correct Agent Routing — The Production-Grade Companion Principle`,
    same exempt clause).
  - PRD §7. Requirements Traceability Matrix — PASS
    (`## 7. Requirements Traceability Matrix`).
  - PRD §BC-DAEMON-001..006 — PASS each (prefix-match to
    `### BC-DAEMON-NNN — <title>` for each of the 6).
  - PRD §BC-AUTH-002 — PASS (`### BC-AUTH-002 — Auth Header Validation...`).
  - PRD §BC-PROTO-002 — PASS (`### BC-PROTO-002 — Phase 4 schema_version...`).
  - PRD §BC-ABI-001 — PASS (`### BC-ABI-001 — ABI Version in /status...`).
  - PRD §BC-ENGINE-002 — PASS (`### BC-ENGINE-002 — ClaudeCodeModule...`).
  - PRD §BC-ENGINE-002-ERR — PASS (`### BC-ENGINE-002-ERR — HomeUnresolvable...`).
  - PRD §BC-ENGINE-003 — PASS (`### BC-ENGINE-003 — ClaudeCodeModule Inherent Methods`).
  - No `§Verification` standalone citations remain (rewritten as
    "§BC-NNN, Verification subsection" position-free descriptions per PG-4
    anti-pattern table — Verification is a bold-label inside each BC
    section, not a heading).
- **PG-2 count coherence:** 22 VPs appears in §Purpose ("22 Behavioral
  Contracts"), §Scope ("All 22 Phase 1 BCs"), §VP Catalog Overview prose
  ("exactly 22 VPs"), §VP Catalog Overview table (22 data rows),
  §Mechanism Distribution table (22 unit-test primary), §Coverage Matrix
  table (22 data rows + "22 BCs → 22 VPs (one-to-one)" footer), frontmatter
  traces_to ("22 BCs after F-R62 fix-burst"). Sweep verified by manual
  inspection of all instances.
- **PG-5 historical-anchor compliance:** §References uses current-pointer
  version pins (PRD v1.1 commit f855835, SS-daemon-lifecycle v1.0.8,
  SS-core-types-and-abi v1.2.8, SS-engine-module v1.1.15). No version-less
  cross-artifact citations in main-body prose. Frontmatter `inputs` field
  uses version-free file paths per PG-5 §Frontmatter Carve-Out (Option B).
- **PG-3 compliance:** no `above`/`below` directional qualifiers appear in
  §References or in any main-body prose. §-anchor citations include
  position-free descriptions where needed (e.g., "§BC-DAEMON-001,
  Verification subsection").
- **F-R60-corpus-sweep:** no count-drift sites identified in this revision;
  the v1.0 §G-1 reference to BC-DAEMON-004 is updated to acknowledge that
  the contract is now formally verified by VP-DAEMON-004 in this catalog.
- **§Trace-Heading-Convention:** this §Trace block uses `## §Trace` heading
  (not `## Trace`) matching the convention.
- **BC-H1-is-title-source-of-truth:** the document title `# Verification
  Properties: Phase 1 Behavioral Contract Catalog` is unchanged across v1.0
  → v1.1; the v1.1 expansion adds 6 daemon BCs which fall under "Phase 1
  Behavioral Contract Catalog" scope.
- **append_only_numbering:** VP-DAEMON-001..006 are append-only additions.
  No existing VP ID was renumbered or removed. VP-PROTO-002 reframing kept
  the ID stable; only the mechanical property and harness location changed.
- **Self-audit checklist (CLAUDE.md §CANONICAL PRINCIPLE):**
  - No MVP/for-now/good-enough/fix-later rationalizations. PASS.
  - No tech-debt-register entries added. PASS.
  - No "TODO for architect" / "pending architect review" placeholders.
    The architect adjudication is COMPLETE (commit 2db408f); this catalog
    consumes the adjudication as a fait accompli. PASS.
  - No silent fix outside formal-verifier scope. VP-PROTO-002 reframing
    REMOVES a v1.0 fabrication that was outside architect-authorized
    Phase 1 surface; this is a corrective in-scope action. F-R62-7's
    secondary disposition (architect-could-add Phase 1 stub) was NOT
    pursued because the architect did not commit that route; this VP
    revision respects the architect's commit boundary. PASS.
  - No cheap-mechanism defaults. The fuzz auxiliary for VP-DAEMON-003 is
    a Phase 6 deliverable but is documented now to ensure the boundary
    exploration is not lost; the primary unit-test is in scope for Phase 3
    TDD. PASS.
  - No advisory-severity downgrades. All F-R62 findings under this
    catalog's scope were treated as BLOCKERS and fixed. PASS.
- **Production-grade default:** every BC has a VP; every VP has a
  mechanical property, pre/post-conditions, counter-example sketches, and
  a harness location (or explicit Phase 4 deferral for VP-PROTO-002).
  Zero MVP shortcuts, zero falsified self-checks.

v1.0 (initial author, 2026-05-14):

- Authored the 16-VP catalog mapping every Phase 1 pre-staged BC to a
  formally-testable verification property.
- Selected `unit-test` as the primary mechanism for all 16 VPs. Rationale:
  Phase 1 BCs are deterministic protocol contracts (token formats, field
  presence, enum exhaustiveness, trait signatures) whose verification is
  fully discharged by deterministic unit tests. Kani model checking is not
  load-bearing in Phase 1; pre-staged for Phase 2 trigger-trace per §G-1.
- Selected 4 fuzz auxiliaries: VP-AUTH-001 + VP-AUTH-002 (single shared fuzz
  target `fuzz_auth_token_validation`), VP-FACTORY-002 (fuzz target
  `fuzz_state_md_parser` exercising the v1.2.3 F-R20-2 regression site),
  VP-PROTO-002 (fuzz target `fuzz_envelope_dispatch` over `u32::MAX`
  schema-version value space).
- Selected 3 mutation-test auxiliaries: VP-RING-001 (`format_version: u32 =
  1` mutation target), VP-LOCK-001 (`contract_version: u32 = 1` mutation
  target), VP-TYPES-001 (`EXEMPT` constant length and attribute-presence
  check mutation surface).
- Counter-example sketches per VP enumerate adversarial inputs that should
  refute the property. Each sketch maps to a concrete past adversary
  finding where one exists (e.g., VP-FACTORY-002 sketch 3 traces to
  F-R20-2 v1.2.3 fix; VP-ENGINE-001 sketch 2 traces to F-R28-1 v1.1.8 fix;
  VP-AUTH-001 sketch 1 traces to FC-06 token-format design).
- Coverage matrix: 16 BCs → 16 VPs (1:1). Open gaps catalogued with
  future-attachment per CLAUDE.md §CANONICAL PRINCIPLE rule 3.
- PG-5 historical-anchor compliance: every cross-artifact citation in
  §References pins a current version (`v1.0.7`, `v1.2.8`, `v1.1.15`) per
  the timestamp `2026-05-14T20:30:00Z`. PG-4 §-anchor compliance: every
  cited §-anchor resolves to an actual heading in the target artifact.
  PG-3 directional compliance: no `above/below` directional qualifiers
  appear in §References. PG-1 schema-fact compliance: dependency pins
  (`temp-env ^0.3`, `constant_time_eq ^0.3`, etc.) reference
  SS-deps-pin-manifest.md as the canonical source.
- This file is the formal-verifier deliverable for Phase 1 PRD pre-staging
  per orchestrator dispatch T-2 (concurrent with product-owner PRD
  synthesis T-1 per STATE.md §Phase 1 dispatch).
