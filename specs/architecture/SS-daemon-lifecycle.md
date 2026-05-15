---
document_type: architecture-section
level: L3
section: "daemon-lifecycle"
version: "1.0.18"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-15T00:01:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/prd.md
  - /Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md
input-hash: "[live-state]"
traces_to: "adversary F-NEW-05 F-NEW-06 F-NEW-07 F-NEW-09; brief v1.4.2 Phase 1 Runtime Core scope; BC-HOOK-022 timeout matrix; BC-HOOK-024 lock-file collision context; FC-01 + FC-06 from forward-compat scan 9618502; pre-Phase-1 lock-in per human authorization; v1.0.5 round-29 fix F-R28-4 HookEventRecord struct definition + constructor in monocle-runtime::ring; v1.0.6 round-30 fix F-R30-2 HookEventRecord #[non_exhaustive] attribute added; v1.0.7 round-53.1 fix F-R53-adv-1 §Analysis mis-anchor corrected to §Item P3-1 in §Trace v1.0.6 rationale sentence; v1.0.8 round-F-R62 fix F-R62-8 BC-AUTH-002 expanded to three failure modes (missing header / invalid token) — disposition (c); v1.0.9 F-R62-4 back-propagation closure (adversary R63 F-R63-adv-2 + consistency R2 F-R63-cons-3): §BC Summary footer updated past-tense + authority split (PRD v1.1 f855835); BC-AUTH-002 §Verification single-file path split to auth_header_rejection.rs; BC-AUTH-001 §Verification file path added (auth_token_lifecycle.rs); v1.0.10 consistency R3 R3-001 closure (commit ba62a15): §BC Summary footer rephrased to version-stable (oscillation prevention per L-F-R63-PARTIAL-FIX); v1.0.11 adversary R65 F-R65-1/2/3 closure: Three→Two count correction at 2 sites + Bearer disposition fix (missing_auth_token); v1.0.12 adversary R70 F-R70-1/F-R70-3 closure: macOS runtime_dir fallback chain (disposition c) + POSIX exit-code correction (disposition c, 130/143/2); v1.0.13 adversary R71 F-R71-2 + F-R71-3 + F-R71-4 closure: stale test name correction (2 sites), NFR-008 anchor correction (4 sites), tower/nix dep-pin dispositions; v1.0.14 adversary R72 F-R72-1 closure: arch JSON schema sketches tightened to mandatory millisecond precision (YYYY-MM-DDTHH:MM:SS.sssZ) — last_hook_ts (§Status endpoint), startTimeUtc (§Start Sequence step 6), shutdown_utc (§Drain step 5); cross-field uniformity achieved; L-F-R63 Extension 1 propagation discipline applied to arch JSON schema SoT; v1.0.15 adversary R74 F-R74-1 closure: hook_endpoints ellipsis placeholder replaced with canonical 5-string enumeration; L-F-R63 Extension 4: placeholder discipline extended to cover JSON array ellipsis patterns in addition to ISO8601 timestamp placeholders; v1.0.16 adversary R75 F-R75-2 closure: §Start Sequence Rationale Windows scope tightened to match PRD NFR-008 macOS+Linux primary targets; Obs-R75-1 closure: §Drain step 4 append-mode ambiguity resolved with two-phase write pattern; v1.0.17 adversary R83 F-R83-1 site 2 closure: §BC Summary footer BC-DAEMON-005 row extended with 0o700 runtime-dir mode and 0o600 lock-file mode — F-R79-3 contract lift propagated from §Start Sequence body tier to summary-table tier; v1.0.18 adversary R88 F-R88-1 closure: §Phase 4 Notes lock-file field enumeration extended from 6 to 7 fields — contract_version now explicitly enumerated as forward-compat version sentinel (FIRST key per BC-LOCK-001 Postcondition 2); Phase 4 readers MUST validate contract_version == 1 before consuming other fields"
project: monocle
---

# Architecture: Daemon Lifecycle

## [Section Content]

## Scope

Phase 1 daemon: a single-process Rust binary (`monocle daemon start`) running an
axum 0.8 HTTP server over `127.0.0.1:<OS-assigned-port>` for hook ingestion,
plus a Unix domain socket (UDS) at `<runtime_dir>/monocle.sock` for TUI client
attach/detach commands. Runtime directory is resolved via a platform-aware chain
(see §Start Sequence step 1): `MONOCLE_RUNTIME_DIR` env override first; then
`directories::ProjectDirs::runtime_dir()` on Linux (returns an XDG-compliant
`/run/user/<uid>/monocle/` path); then `directories::ProjectDirs::data_local_dir()`
on macOS (returns `~/Library/Application Support/monocle/`) and Windows (returns
`%APPDATA%/monocle/`) — because `runtime_dir()` returns `None` on those platforms
by design. NFR-008 lists macOS among the primary targets (`macOS + Linux`, darwin/linux × amd64/arm64); the fallback chain ensures
monocle starts correctly on macOS without operator intervention. All lifecycle state
(port, pid, auth token, start time) is written to a single lock file at
`<runtime_dir>/monocle.lock` using `tempfile::persist` for atomic write.

## Health and Status Endpoints (F-NEW-05)

Both endpoints are registered on the same axum router as the 5 hook endpoints.

### GET /healthz

**Contract (BC-DAEMON-001):** Returns HTTP 200 with body:

```json
{"status":"alive","uptime_sec":<N>,"version":"<semver>"}
```

where `uptime_sec` is an integer seconds-since-daemon-start and `version` is the
monocle binary semver. Returns HTTP 503 with body `{"status":"shutting_down"}` if
the daemon is in `ShuttingDown` AppMode or if the hook-receiver task has exited
abnormally.

**Authentication:** `/healthz` is unauthenticated. Rationale: liveness probes
must succeed even if the auth token has rotated (e.g., during a crash-recovery
startup). The endpoint exposes no sensitive state — uptime and version are not
secret. A local adversary that can reach `127.0.0.1:<port>` already has sufficient
OS-level access to enumerate the monocle process via `ps`.

**TUI client use:** If `/healthz` is unreachable AND the lock file exists with a
live pid (`kill(pid, 0)` succeeds), the TUI concludes the daemon is hung (accepting
TCP but not responding) and initiates a recovery flow: offer the user "Kill and
restart?" with a 10-second countdown before auto-restarting.

If `/healthz` is unreachable AND the lock file exists with a dead pid, the TUI
initiates normal auto-start (lock file is stale).

### GET /status

**Contract (BC-DAEMON-002):** Returns HTTP 200 with body:

```json
{
  "pid": <N>,
  "uptime_sec": <N>,
  "version": "<semver>",
  "abi_version": <N>,
  "lock_file": "<path>",
  "hook_endpoints": [
    "/hooks/pre-tool-use",
    "/hooks/notification",
    "/hooks/stop",
    "/hooks/session-start",
    "/hooks/prompt-submit"
  ],
  "ring_buffer_fill_pct": <0.0-100.0>,
  "channel_saturation_pct": <0.0-100.0>,
  "last_hook_ts": {
    "pre_tool_use": "<YYYY-MM-DDTHH:MM:SS.sssZ or null>",
    "notification": "<YYYY-MM-DDTHH:MM:SS.sssZ or null>",
    "stop": "<YYYY-MM-DDTHH:MM:SS.sssZ or null>",
    "session_start": "<YYYY-MM-DDTHH:MM:SS.sssZ or null>",
    "prompt_submit": "<YYYY-MM-DDTHH:MM:SS.sssZ or null>"
  },
  "tui_attached": <bool>
}
```

`last_hook_ts` values use ISO 8601 UTC format with mandatory millisecond precision
(`YYYY-MM-DDTHH:MM:SS.sssZ`). A hook type that has not fired since daemon start has
value `null` (JSON null), not an empty string. Format matches EC-044 (PRD v1.7)
and the `shutdown_utc` format in BC-DAEMON-006 — cross-field uniformity per F-R72-1.

The `abi_version` field carries `monocle_core::MONOCLE_ABI_VERSION` as compiled
into this binary. Required by BC-ABI-001 (see SS-core-types-and-abi.md §ABI Version
Constant). Phase 3 plugin SDK and Phase 4 federation use this field to verify
ABI compatibility before handshake.

**Authentication:** `/status` requires the same `X-Monocle-Authorization: <token>`
header as hook endpoints. Rationale: `/status` exposes internal buffer fill levels
and channel saturation — metrics that reveal load patterns and internal queue
behavior that a local adversary could exploit to time attacks. Unauthenticated
access to `/status` is not warranted given the richer payload.

**Use:** Developer debugging, observability, CI integration tests. Read-only; no
state mutations.

## Body Size Limit (F-NEW-06)

**Contract (BC-DAEMON-003):** All hook POST endpoints (`/hooks/*`) and `/status`
enforce `DefaultBodyLimit::max(256 * 1024)` (256 KiB = 262,144 bytes) via axum's
`DefaultBodyLimit` layer applied at router construction time.

Requests exceeding this limit receive HTTP 413 Payload Too Large:

```json
{"error":"payload_too_large","limit_bytes":262144}
```

**Rationale:** The `Notification` hook body carries an unbounded `message` string
(BC-HOOK-023 — gene-source). Claude Code production usage generates messages in
the range of 1–50 KiB (diff output, stack traces, tool output summaries). The
256 KiB ceiling accommodates 5× the 99th-percentile expected payload size while
bounding worst-case daemon memory exposure to a predictable constant:
`concurrent_requests_max × 256KiB`. With axum's default connection limit, this
is bounded well below a practical memory-exhaustion threshold on any developer
workstation.

`/healthz` carries no body; no limit applies.

**Implementation note:** axum 0.8 does NOT apply a body limit by default.
`DefaultBodyLimit` must be explicitly added as a layer. The auth middleware must
NOT be applied to `/healthz` (unauthenticated per BC-DAEMON-001). The correct
axum 0.8 pattern is to declare two routers — one unauthenticated, one authenticated
— and merge them. Hook endpoints and admin endpoints (`/status`, `/shutdown`)
share the same `X-Monocle-Authorization` middleware layer (single auth layer on the
authenticated router); the Claude Code IDE token (`X-Claude-Code-Ide-Authorization`)
is checked per-handler inside the hook handlers, not as a separate router-level layer,
because the IDE token is optional and absent on non-hook requests.

```rust
// Unauthenticated router — liveness probe only; no body limit needed (no body).
let public_router = Router::new()
    .route("/healthz", get(healthz_handler));

// Authenticated router — hook endpoints + admin endpoints.
// DefaultBodyLimit is applied here; /healthz carries no body so the limit
// is irrelevant for the public router.
let authed_router = Router::new()
    .route("/hooks/pre-tool-use", post(pre_tool_use_handler))
    .route("/hooks/notification", post(notification_handler))
    .route("/hooks/stop", post(stop_handler))
    .route("/hooks/session-start", post(session_start_handler))
    .route("/hooks/prompt-submit", post(prompt_submit_handler))
    .route("/status", get(status_handler))
    .route("/shutdown", post(shutdown_handler))
    .layer(DefaultBodyLimit::max(256 * 1024))
    .layer(auth_layer); // X-Monocle-Authorization enforced on all routes above

// Merge into a single service. axum::Router::merge combines two independent
// routers; routes in each retain their own layer stacks.
let app = public_router.merge(authed_router);
```

## Daemon Lifecycle Protocol (F-NEW-09)

### Start Sequence

1. Resolve `runtime_dir` via the following platform-aware chain (F-R70-1 closure):

   **Resolution chain (evaluated in order; first `Some` result wins):**

   a. `MONOCLE_RUNTIME_DIR` environment variable — if set and non-empty, use as
      the runtime directory path verbatim. This is the operator escape hatch for
      containers, NixOS, and any deployment where platform defaults are
      inappropriate. Log `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var`.

   b. `directories::ProjectDirs::runtime_dir()` — returns `Some` on Linux (XDG
      `$XDG_RUNTIME_DIR/monocle`, e.g., `/run/user/1000/monocle`); returns `None`
      on macOS and Windows by design of the platform ABI. If `Some`, use this path.
      Log `INFO: runtime_dir from ProjectDirs::runtime_dir()`.

   c. `directories::ProjectDirs::data_local_dir()` — platform fallback for macOS
      and Windows (and any Linux environment where `XDG_RUNTIME_DIR` is not set):
      macOS → `~/Library/Application Support/monocle/`; Windows →
      `%APPDATA%/monocle/`. If `Some`, use this path. Log
      `INFO: runtime_dir fallback to data_local_dir (platform: <os>)`.

   d. If all three resolution paths return `None` (e.g., no home directory AND no
      `MONOCLE_RUNTIME_DIR`), exit 1 with:
      `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`.
      This is the fail-fast path for genuinely unresolvable environments.

   **Rationale:** `ProjectDirs::runtime_dir()` returns `None` on macOS and Windows
   by design — not due to misconfiguration. macOS is among the primary target platforms
   (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64). A fail-fast-only approach would require every macOS user to set
   `MONOCLE_RUNTIME_DIR` before starting monocle, which violates the zero-config
   startup requirement. The `data_local_dir()` fallback provides a correct,
   standards-compliant runtime state location on macOS (`~/Library/Application Support/monocle/`).
   Windows is a secondary build target per PRD §8.7; the same `data_local_dir()`
   fallback resolves to `%APPDATA%/monocle/` on Windows but Phase 1 CI does not
   formally validate Windows behavior per NFR-008's `macOS + Linux` target scope.
   The env override preserves operator
   control for non-standard deployments without burdening default users. The
   asymmetry with `BC-ENGINE-002-ERR` (which fail-fasts on `BaseDirs::new() == None`)
   is correct: `BaseDirs::new()` returns `None` only when there is no home directory
   at all — a genuine system-configuration failure; `ProjectDirs::runtime_dir()`
   returns `None` on macOS as a platform design choice, not a failure.

   Implementation:

   ```rust
   fn resolve_runtime_dir(project_dirs: &directories::ProjectDirs) -> Result<PathBuf, DaemonStartError> {
       // (a) Operator env override
       if let Ok(env_path) = std::env::var("MONOCLE_RUNTIME_DIR") {
           if !env_path.is_empty() {
               tracing::info!(source = "MONOCLE_RUNTIME_DIR", "runtime_dir resolved");
               return Ok(PathBuf::from(env_path));
           }
       }
       // (b) XDG runtime dir (Linux only in practice)
       if let Some(rd) = project_dirs.runtime_dir() {
           tracing::info!(source = "ProjectDirs::runtime_dir()", "runtime_dir resolved");
           return Ok(rd.to_path_buf());
       }
       // (c) data_local_dir fallback (macOS / Windows / XDG-less Linux)
       let fallback = project_dirs.data_local_dir().to_path_buf();
       tracing::info!(
           source = "ProjectDirs::data_local_dir()",
           platform = std::env::consts::OS,
           "runtime_dir fallback resolved"
       );
       Ok(fallback)
   }
   ```

   If `ProjectDirs::new("monocle", "monocle", "monocle")` itself returns `None`
   (which requires no home directory), the daemon exits with
   `DaemonStartError::RuntimeDirUnresolvable` before `resolve_runtime_dir` is called.
   This is the fail-fast path (d) above.

   Create the resolved directory with mode `0o700` if absent.
2. Check for existing lock file at `<runtime_dir>/monocle.lock`. If it exists:
   a. Parse the JSON lock content.
   b. Send `kill(pid, 0)` to the pid in the lock. If the pid is alive, log
      `ERROR: daemon already running at pid=<N>; exiting` and exit 1.
   c. If the pid is dead (stale lock), log `WARN: stale lock file removed` and
      proceed.
3. Generate a cryptographically random 32-byte auth token, hex-encoded (64 chars).
   Use `rand::rngs::OsRng` — not `thread_rng`.

   **Token format (FC-06 resolution):** the auth token written to the lock file
   and presented in the `X-Monocle-Authorization` header is
   `monocle-v1:<64-char-hex>` — the literal prefix `monocle-v1:` followed by a
   64-character lowercase hex string (32 bytes of `OsRng`-generated entropy).
   Total token length: 74 characters.

   The prefix versions the auth model. Phase 4 federation introduces OAuth2
   bearer tokens; the prefix allows the daemon's auth middleware to dispatch
   on token type without ambiguity for the LOCAL auth surface:

   | Prefix | Auth model | Header | Phase introduced |
   |--------|-----------|--------|-----------------|
   | `monocle-v1:` | Local shared secret (32-byte OsRng entropy) | `X-Monocle-Authorization` | Phase 1 |

   **Phase 4 OAuth2 clarification (F-FC-I005):** Phase 4 federation does NOT
   extend `X-Monocle-Authorization` to carry OAuth2 tokens. Phase 4 federation
   tokens use the STANDARD `Authorization: Bearer <token>` header on a SEPARATE
   `monocle-ipc` federation channel (russh tunnel), which is distinct from the
   Phase 1 HTTP hook-ingestion channel. The Phase 1 daemon's auth middleware:

   - Inspects only `X-Monocle-Authorization` (never `Authorization: Bearer`).
   - Rejects any `Authorization: Bearer` header with HTTP 401 on Phase 1 routes
     (the header is not a recognized auth mechanism for Phase 1 endpoints).

   Phase 4 daemon adds a separate federation middleware path on the russh/`monocle-ipc`
   channel gated by a `federation` feature flag. The Phase 1 HTTP routes remain
   `X-Monocle-Authorization`-only with no Bearer support. BC-AUTH-002 applies
   only to the Phase 1 `X-Monocle-Authorization` surface.

   Auth middleware validation rules in Phase 1 (applied in this order):

   1. **Missing header:** if the `X-Monocle-Authorization` header is absent
      entirely, return HTTP 401 `{"error":"missing_auth_token"}` immediately.
      This is a structural precondition failure, not an authentication attempt.

   2. **Format check:** if the header is present but its value does NOT begin
      with `monocle-v1:`, return HTTP 401 `{"error":"invalid_auth_token"}`
      before any secret comparison occurs. This prevents timing-oracle attacks
      where an attacker probes whether a non-prefixed string matches the secret.

   3. **Secret comparison:** strip the `monocle-v1:` prefix, then perform a
      constant-time comparison of the remaining hex part against the stored
      secret. If the comparison fails for any reason (wrong hex value, empty
      suffix, length mismatch), return HTTP 401 `{"error":"invalid_auth_token"}`.

   Rules 2 and 3 deliberately return the SAME error body (`invalid_auth_token`).
   This collapses the "malformed format" and "correct format but wrong value"
   failure modes into a single indistinguishable response, blocking an attacker
   from determining whether their token had the structurally correct prefix even
   if they could not read the lock file directly.

   **Security rationale (threat model):** The monocle daemon binds exclusively
   to `127.0.0.1`. All callers are local processes running as the same OS user.
   An adversary co-located as the same user can read `monocle.lock` directly
   (0o600, same-user read access). Enumeration via distinct format-vs-mismatch
   error bodies provides zero marginal attack capability for a same-user
   adversary. However, defence-in-depth is applied: collapsing Rules 2 and 3
   costs nothing (both are auth failures) and prevents any information leak to
   an attacker who has gained unexpected network access to 127.0.0.1 but has
   NOT gained file-system access (e.g., a compromised subprocess with a
   restricted sandbox).

   The `missing_auth_token` body for absent headers (Rule 1) is deliberately
   distinct because: (a) absence of the header is a client-configuration error,
   not an authentication attempt — the attacker who omits the header has
   revealed nothing about knowledge of the secret; (b) the distinct body
   provides actionable diagnostics for developers debugging hook integration.

   Auth middleware implementation:

   ```rust
   const TOKEN_PREFIX: &str = "monocle-v1:";

   /// Extract and validate the monocle auth token from the request headers.
   ///
   /// Returns:
   /// - `Ok(())` if the token is present, well-formed, and matches the secret.
   /// - `Err(AuthError::Missing)` if the `X-Monocle-Authorization` header is absent.
   /// - `Err(AuthError::Invalid)` if the header is present but fails validation
   ///   for any reason (bad prefix, bad format, or secret mismatch). These cases
   ///   are intentionally collapsed into a single error variant to prevent
   ///   information disclosure about which check failed.
   fn validate_auth_header(
       headers: &HeaderMap,
       expected_secret: &str,
   ) -> Result<(), AuthError> {
       let Some(header_value) = headers.get("X-Monocle-Authorization") else {
           return Err(AuthError::Missing);
       };
       let Ok(presented) = header_value.to_str() else {
           return Err(AuthError::Invalid);
       };
       let Some(hex_part) = presented.strip_prefix(TOKEN_PREFIX) else {
           return Err(AuthError::Invalid); // bad prefix — not an auth attempt
       };
       // Constant-time comparison to prevent timing oracle on the hex secret.
       if constant_time_eq::constant_time_eq(hex_part.as_bytes(), expected_secret.as_bytes()) {
           Ok(())
       } else {
           Err(AuthError::Invalid) // format OK but token mismatch — same body
       }
   }

   #[derive(Debug)]
   enum AuthError {
       Missing,  // → HTTP 401 {"error":"missing_auth_token"}
       Invalid,  // → HTTP 401 {"error":"invalid_auth_token"} (all other failures)
   }
   ```

   The `expected_secret` stored in memory (and written to the lock file's
   `authToken` field) is the bare 64-char hex string WITHOUT the prefix. The
   prefix is stripped from the presented token before comparison. This design
   keeps the lock-file value unambiguous (always a raw hex secret for Phase 1)
   while the wire format is always prefixed.

   Lock file `authToken` field value: `<64-char-hex>` (no prefix — the prefix
   is a wire-format concern, not a storage concern).

   **Behavioral contracts:**

   - **BC-AUTH-001:** The auth token written to the lock file has format
     `monocle-v1:<64-hex>` when read back from the lock file and presented to
     the daemon. The lock file `authToken` field stores only the 64-char hex
     part. Verification: integration test in
     `monocle-runtime/tests/auth_token_lifecycle.rs` reads the lock file after
     daemon start and asserts `authToken` matches `/^[0-9a-f]{64}$/`; presents
     `monocle-v1:<authToken>` to `/status` and asserts HTTP 200.
     Test name: `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`
     (PRD v1.1 §7 RTM canonical path; F-R62-4).

   - **BC-AUTH-002:** Two auth failure modes are specified:

     | Failure mode | Header state | HTTP body |
     |---|---|---|
     | Missing header | `X-Monocle-Authorization` absent | `{"error":"missing_auth_token"}` |
     | Invalid token | Header present; value fails for any reason (bad prefix, bad format, secret mismatch, or empty suffix) | `{"error":"invalid_auth_token"}` |

     All "invalid token" failures return the same body regardless of whether
     the format check or the secret comparison failed — this is a deliberate
     security choice (see security rationale above).

     Phase 4 OAuth2 federation tokens use `Authorization: Bearer` on a separate
     federation channel and are NOT valid on Phase 1 HTTP endpoints; they
     receive HTTP 401 `{"error":"missing_auth_token"}` (no `X-Monocle-Authorization`
     header present; `Authorization: Bearer` is a different, unrecognized header —
     Phase 4 OAuth2 uses a separate federation channel and does not reuse the
     Phase 1 HTTP endpoints).

     Verification: integration test in
     `monocle-runtime/tests/auth_header_rejection.rs` (rejection probes;
     F-R62-4 canonical path per PRD v1.1 §7 RTM). Round-trip happy-path covered
     in `monocle-runtime/tests/auth_token_lifecycle.rs` per BC-AUTH-001
     verification above.
     Test name: `test_BC_AUTH_002_auth_header_validation_all_failure_modes`
     - No header → HTTP 401 `{"error":"missing_auth_token"}`
     - `X-Monocle-Authorization: baretoken` → HTTP 401 `{"error":"invalid_auth_token"}`
     - `X-Monocle-Authorization: monocle-v2:abc` → HTTP 401 `{"error":"invalid_auth_token"}`
     - `X-Monocle-Authorization: monocle-v1:` (empty suffix) → HTTP 401 `{"error":"invalid_auth_token"}`
     - `Authorization: Bearer fake` (wrong header name) → HTTP 401 `{"error":"missing_auth_token"}`
     - `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` → HTTP 401 `{"error":"invalid_auth_token"}`

   Note: `constant_time_eq` crate is added to the Phase 1 dependency manifest
   (caret pin `^0.3`; no untrusted-input deserialization; timing-safety is its
   only function). This matches the canonical pin in SS-deps-pin-manifest.md
   Phase 1 pin table (authoritative source).
4. Bind HTTP listener on `127.0.0.1:0` (OS-assigned port). Retrieve the actual
   port via `listener.local_addr()`.
5. Bind UDS at `<runtime_dir>/monocle.sock` with mode `0o600`.
6. Write lock file atomically via `tempfile::persist`:
   ```json
   {
     "contract_version": 1,
     "pid": <N>,
     "port": <N>,
     "authToken": "<64-char hex>",
     "startTimeUtc": "<YYYY-MM-DDTHH:MM:SS.sssZ>",
     "app": "monocle",
     "version": "<semver>"
   }
   ```
   The `contract_version` field is always the first key (parallel to the JSONL
   ring `format_version` convention). Phase 4 and future tooling check this field
   before parsing remaining lock-file fields. Value `1` is the Phase 1 contract.
   BC-LOCK-001: any lock-file reader MUST check `contract_version == 1` before
   consuming other fields; an unrecognized version triggers a graceful skip with
   a log warning.
   `startTimeUtc` uses ISO 8601 UTC format with mandatory millisecond precision
   (`YYYY-MM-DDTHH:MM:SS.sssZ`) — matching `last_hook_ts` (BC-DAEMON-002 / EC-044)
   and `shutdown_utc` (BC-DAEMON-006) for cross-field uniformity per F-R72-1.
   Lock file mode: `0o600` (owner-only read/write).
7. Spawn hook-receiver task (axum server on the bound listener).
8. Spawn UDS control task.
9. Log `INFO: monocle daemon started pid=<N> port=<N>`.

The `app: "monocle"` field allows future hook-discovery code to filter by
app name, avoiding the BC-HOOK-024 cross-IDE collision (see §Lock File Discovery
Policy below).

### Shutdown Signal Handling

The daemon listens for SIGTERM, SIGINT, and an authenticated `POST /shutdown`
request (admin endpoint, `X-Monocle-Authorization` required).

On receiving any shutdown signal:

1. Set AppMode to `ShuttingDown`.
2. Stop accepting new hook POSTs: axum returns HTTP 503 with header
   `Retry-After: 10` and body `{"error":"daemon_shutting_down"}` for all
   `/hooks/*` routes. `/healthz` returns 503 `{"status":"shutting_down"}`.
   `/status` continues to serve (read-only; useful during drain monitoring).

### Drain (10-Second Timeout)

3. Wait up to 10 seconds for in-flight hook POSTs to complete
   (`tokio::time::timeout(Duration::from_secs(10), drain_inflight())`).
4. If `--persistent-events` flag is set, flush the JSONL ring buffer to disk at
   `<runtime_dir>/monocle-events.jsonl`. The flush uses a two-phase write: read
   the existing file content (if any) into memory, append the in-memory ring
   buffer records, then write the combined content via `tempfile::persist` over
   the destination path. This preserves prior events while guaranteeing an
   atomic replace — `tempfile::persist` is not append-mode; it is an atomic
   rename from a temp file, so the two-phase pattern is required to retain
   existing file content.

   **Format versioning (FC-01 resolution):** every JSONL event record carries a
   top-level `format_version: u32 = 1` field as the first key. Phase 2
   trigger-trace ingests Phase 1 ring history; the version field allows Phase 2
   to detect and refuse incompatible records (e.g., if a future Phase 5 changes
   the record shape, Phase 2's reader checks `format_version` and falls back to
   a migration path). Any future change to the record shape requires bumping
   `format_version` AND adding a Phase 2 ingestor capable of reading both
   versions. The field is serialized first in the JSON object — before
   `session_id`, `timestamp_micros`, or any event-type field — so readers can
   parse and validate the version without deserializing the full record.

   **`HookEventRecord` struct (defined in `monocle-runtime::ring`):**

   The concrete Rust type written to the JSONL ring buffer. Defined in
   `monocle-runtime` (NOT `monocle-core`) because the ring buffer is a daemon
   runtime artifact — it is not part of the core ABI surface. The struct uses
   `#[derive(serde::Serialize, serde::Deserialize)]` for JSON round-trips.
   Field ordering in the serialized JSON is governed by serde's default (field
   declaration order) combined with the `format_version` field being first.
   `serde_json::to_string` preserves declaration order for standard structs.

   ```rust
   // monocle-runtime/src/ring.rs

   use serde::{Serialize, Deserialize};

   /// A single event record written to the JSONL ring buffer.
   ///
   /// `format_version` is declared first to guarantee it serializes first in the
   /// JSON object (`serde_json` preserves struct field declaration order). Phase 2
   /// ring-buffer ingestors check this field before deserializing the rest of the
   /// record. The value is always `1` for Phase 1-origin records.
   ///
   /// `tool_name` and `tool_input` are `Option` because only `PreToolUse` and
   /// `Notification` hook events carry tool context; `SessionStart`, `Stop`, and
   /// `UserPromptSubmit` events set both to `None`.
   #[non_exhaustive]
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct HookEventRecord {
       /// Format version for this ring record shape. Always `1` in Phase 1.
       /// Declared and serialized first so ingestors can version-check without
       /// deserializing the full record.
       pub format_version: u32,
       /// Claude Code's session UUID (matches `EnrichedSession::session_id`).
       pub session_id: String,
       /// Event timestamp as microseconds since the Unix epoch (UTC).
       pub timestamp_micros: i64,
       /// PID of the Claude Code subprocess that generated this hook event.
       pub pid: u32,
       /// Hook event type as a string (matches `HookType` variant names:
       /// "SessionStart", "UserPromptSubmit", "PreToolUse", "Notification", "Stop").
       /// Stored as `String` (not `HookType`) to avoid pulling `monocle-core` into
       /// deserialization paths that only need the raw JSONL record.
       pub hook_type: String,
       /// Tool name; populated for `PreToolUse` and `Notification` events.
       /// `None` for `SessionStart`, `UserPromptSubmit`, and `Stop`.
       pub tool_name: Option<String>,
       /// JSON-encoded tool input; populated for `PreToolUse` and `Notification` events.
       /// `None` for events without tool context.
       /// Stored as `serde_json::Value` to avoid double-deserialization.
       pub tool_input: Option<serde_json::Value>,
   }

   impl HookEventRecord {
       /// Construct a ring record from a parsed hook event.
       ///
       /// `format_version` is always `1` in Phase 1 — callers must NOT pass any
       /// other value. The const `RING_FORMAT_VERSION: u32 = 1` is defined in
       /// this module and MUST be used at all `HookEventRecord::new` call sites.
       ///
       /// `tool_name` and `tool_input` are `None` for hook types that carry no tool
       /// context (`SessionStart`, `UserPromptSubmit`, `Stop`). For `PreToolUse`
       /// and `Notification`, both are `Some`.
       pub fn new(
           session_id: String,
           timestamp_micros: i64,
           pid: u32,
           hook_type: String,
           tool_name: Option<String>,
           tool_input: Option<serde_json::Value>,
       ) -> Self {
           Self {
               format_version: RING_FORMAT_VERSION,
               session_id,
               timestamp_micros,
               pid,
               hook_type,
               tool_name,
               tool_input,
           }
       }
   }

   /// Ring buffer format version constant for Phase 1.
   ///
   /// Increment this constant AND add a Phase 2 ingestor capable of reading both
   /// versions before changing the `HookEventRecord` field layout.
   pub const RING_FORMAT_VERSION: u32 = 1;
   ```

   Example serialized record (`serde_json::to_string` with field declaration order):

   ```json
   {"format_version":1,"session_id":"<uuid>","timestamp_micros":1747094400000000,"pid":12345,"hook_type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}
   ```

   **Behavioral contract: BC-RING-001** — every JSONL record's first key is
   `format_version` with value `1` for all Phase 1-origin records. Verification:
   unit test in `monocle-runtime/tests/jsonl_ring.rs` constructs a
   `HookEventRecord` via `HookEventRecord::new(...)` and asserts the resulting
   JSON string begins with `{"format_version":1,`.
5. Persist last-known AppMode to crash-recovery checkpoint:
   `<runtime_dir>/monocle.recovery.json`:
   ```json
   {
     "pid": <N>,
     "shutdown_reason": "graceful|signal|forced",
     "last_app_mode": "<string>",
     "shutdown_utc": "<YYYY-MM-DDTHH:MM:SS.sssZ>"
   }
   ```
   `shutdown_utc` MUST use ISO 8601 UTC format with mandatory millisecond precision
   (`YYYY-MM-DDTHH:MM:SS.sssZ`). A seconds-only value (e.g., `2026-05-15T07:30:00Z`) is
   non-compliant per BC-DAEMON-006 invariant 1 (PRD v1.7). VP-DAEMON-006 enforces this
   with regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`. Matches `last_hook_ts`
   format (EC-044) and `startTimeUtc` (§Start Sequence step 6) for cross-field uniformity.

### Hard Shutdown

6. After 10-second drain timeout OR on receipt of a second signal OR on a second
   authenticated `POST /shutdown` during drain:
   a. Force-close all axum connections. In axum 0.8, `axum::Server` was removed;
      the correct idiom is `axum::serve(listener, app).with_graceful_shutdown(shutdown_rx)`
      where `shutdown_rx` is a `tokio::sync::oneshot::Receiver<()>` sent by the
      signal handler. On hard shutdown (second signal, second admin `/shutdown`, or
      drain timeout expiry), drop the sender half to unblock the receiver and trigger
      immediate connection close.

      Signal handling uses `tokio::signal::unix::signal(SignalKind::terminate())` for
      SIGTERM and `tokio::signal::ctrl_c()` for SIGINT; both are `async fn` futures
      awaited in a `tokio::select!` loop alongside the oneshot receiver. The signal
      type that triggered hard shutdown is recorded for exit-code selection in step 6d.

   b. Close UDS socket; remove `<runtime_dir>/monocle.sock`.
   c. Remove `<runtime_dir>/monocle.lock`.
   d. Exit with the code appropriate to the hard-shutdown trigger (see Exit codes below).

Exit codes (F-R70-3 closure — POSIX 128+N convention; disposition c):
- `0`: graceful drain succeeded; all in-flight requests completed; ring buffer flushed.
- `130`: hard-killed by SIGINT (signal 2) during drain — POSIX convention 128+2.
  Typical cause: user pressed Ctrl-C a second time while draining.
- `143`: hard-killed by SIGTERM (signal 15) during drain — POSIX convention 128+15.
  Typical cause: systemd/k8s sent a second SIGTERM after the graceful-shutdown window.
  External monitoring (systemd `ExecStop=`, k8s `terminationGracePeriodSeconds`) MUST
  interpret exit 143, not 130, as the SIGTERM hard-kill outcome.
- `2`: hard-killed by a second authenticated `POST /shutdown` during drain (admin
  forced-stop). This is a monocle-specific programmatic code, not a POSIX signal code.
  Value 2 was chosen because it does not collide with POSIX 128+N space (which starts
  at 129) and is distinct from exit 1 (daemon start failure). External monitoring should
  treat exit 2 as "operator-initiated force-stop via admin API."
- `1`: daemon failed to start (see step 1d — `RuntimeDirUnresolvable`, port bind failure,
  existing live lock file, etc.).

**BC-DAEMON-004** (exit-code postcondition): The exit code written to the OS process
table on daemon termination MUST match the trigger:
- graceful drain complete → `0`
- SIGINT hard-kill during drain → `130`
- SIGTERM hard-kill during drain → `143`
- admin `/shutdown` second-call during drain → `2`
- startup failure → `1`

Verification: integration test in `monocle-runtime/tests/daemon_lifecycle.rs`
(`test_BC_DAEMON_004_exit_codes_posix_distinct`) sends SIGTERM twice (expects 143), SIGINT twice
(expects 130), and two sequential `POST /shutdown` calls (expects 2). The PRD
BC-DAEMON-004 postcondition is the canonical error-taxonomy source; this architecture
document is the canonical rationale source for the code selection.

### Crash Recovery

On startup, if `<runtime_dir>/monocle.recovery.json` exists AND the pid in the
(now-stale or absent) lock file is dead:

**Contract (BC-DAEMON-006):**
1. Log `WARN: recovery checkpoint found; prior daemon exited without clean shutdown`.
2. Read `last_app_mode` and `shutdown_reason` from the recovery file.
3. If a TUI client attaches within 60 seconds of daemon start, offer the recovery
   state via the UDS control protocol: `{"type":"recovery_available","last_app_mode":"<...>"}`.
   The TUI displays a banner: "Prior session ended unexpectedly. Restore state? [Y/n]"
4. On TUI acknowledgment (Y or timeout), delete `monocle.recovery.json`.
5. On TUI decline (N), delete `monocle.recovery.json` without restoring state.

If no TUI attaches within 60 seconds, the recovery file is deleted silently and
the daemon starts fresh.

## Lock File Discovery Policy (F-NEW-07)

**Contract:** monocle's hook command embeds the **literal lock file path** at
registration time. Hook scripts MUST NOT scan a directory for `*.lock` files.

**Mechanism:** `monocle hook install` generates a Claude Code hook settings file
(`<runtime_dir>/hooks-settings.json`) whose embedded node-eval JS reads
`<runtime_dir>/monocle.lock` by **absolute path** — not by scanning
`~/.claude/ide/`. The generated hook command is:

```js
const lk = JSON.parse(require('fs').readFileSync('<runtime_dir>/monocle.lock', 'utf8'));
```

**Rationale:** BC-HOOK-024 (gene-source any-context-lazyclaude-pass-B-deep-hooks-r1.md
lines 412–428) documents that Claude Code's hook discovery JS uses
"highest-port-wins" across ALL `~/.claude/ide/*.lock` files without filtering by
`lock.App` — creating P2 cross-IDE collision risk. lazyclaude relies on empirical
properties (other tools binding lower ports, `CleanAllExcept` at startup) to avoid
this collision in practice. monocle eliminates the risk entirely by:

1. Writing the lock file at `<runtime_dir>/monocle.lock` — a path OUTSIDE
   `~/.claude/ide/` — so Claude Code's own IDE-lock scan never picks it up.
2. Embedding the literal lock-file path in the generated hook command — no
   directory scan, no "highest-port-wins," no collision surface.
3. Including `"app": "monocle"` in the lock file JSON so any future hook tooling
   that does check `lock.App` will filter correctly.

**Cross-references:** OQ-10 (runtime dir via `directories::ProjectDirs`); OQ-04
(OS-assigned port); BC-HOOK-024 (cross-IDE collision risk — gene-source).

## Behavioral Contract Summary

| ID | Contract | Section |
|----|----------|---------|
| BC-DAEMON-001 | `/healthz` returns 200/503 with uptime + version; unauthenticated | Health and Status Endpoints |
| BC-DAEMON-002 | `/status` returns full daemon state JSON; requires auth token | Health and Status Endpoints |
| BC-DAEMON-003 | All `/hooks/*` and `/status` enforce 256 KiB body limit; 413 on excess | Body Size Limit |
| BC-DAEMON-004 | Graceful shutdown: 10-second drain, ring buffer flush, recovery checkpoint; exit codes: 0 (clean), 130 (SIGINT hard-kill), 143 (SIGTERM hard-kill), 2 (admin /shutdown force-stop), 1 (startup failure) | Daemon Lifecycle Protocol |
| BC-DAEMON-005 | Runtime dir resolved via platform-aware chain: MONOCLE_RUNTIME_DIR env override → ProjectDirs::runtime_dir() (Linux/XDG) → ProjectDirs::data_local_dir() (macOS/Windows fallback); runtime_dir created with mode `0o700` owner-only (defense-in-depth with lock file `0o600`); lock file created atomically via `tempfile::persist`; pid-liveness checked on startup; removed on clean shutdown | Daemon Lifecycle Protocol |
| BC-DAEMON-006 | Crash recovery checkpoint at `<runtime_dir>/monocle.recovery.json`; TUI offered recovery on next attach | Daemon Lifecycle Protocol |
| BC-RING-001 | Every JSONL ring buffer record's first key is `format_version` with value `1` for all Phase 1-origin records (FC-01) | Daemon Lifecycle Protocol §Drain |
| BC-AUTH-001 | Auth token wire format is `monocle-v1:<64-hex>`; lock file stores bare 64-hex; presented token validated with constant-time comparison after prefix strip (FC-06) | Daemon Lifecycle Protocol §Start Sequence |
| BC-AUTH-002 | Two auth failure modes: (1) absent header → HTTP 401 `{"error":"missing_auth_token"}`; (2) header present but fails for any reason (bad prefix, bad format, secret mismatch) → HTTP 401 `{"error":"invalid_auth_token"}` (collapsed; no format/mismatch distinction); Phase 4 OAuth2 federation uses separate channel (FC-06 + F-FC-I005) | Daemon Lifecycle Protocol §Start Sequence |
| BC-LOCK-001 | Lock-file JSON includes `contract_version: 1` as the first key; readers must check this field before consuming other fields; unrecognized version triggers graceful skip with warning (F-FC-O001) | Daemon Lifecycle Protocol §Start Sequence |

The Phase 1 PRD has formalized these as full BC entries with preconditions,
postconditions, invariants, edge cases, canonical test vectors, and verification
harness stubs (initial formalization: PRD v1.1, commit f855835). The current
canonical PRD is `.factory/specs/prd.md` regardless of version evolution.
Authority split: this architecture artifact is source-of-truth for invariants,
protocol decisions, and security rationale; the PRD is source-of-truth for
canonical test names, test-file paths, error taxonomy, and edge case catalog.

## Phase 4 Notes

Federation (Phase 4) may extend `/status` to report peer-daemon health
(`"peers": [{"host": "...", "status": "alive|unreachable", "uptime_sec": N}]`).
`/healthz` stays single-host by design — it is a liveness probe for the local
daemon only, not a cluster health check. The multi-host health view is `/status`
scope only.

The lock file format gains a `"peers"` array in Phase 4 (federation peer list)
but the 7 Phase 1 fields are stable across Phase 1 → Phase 4: `"contract_version"`
(forward-compatibility version sentinel, always FIRST key per BC-LOCK-001
Postcondition 2; Phase 4 readers MUST validate `contract_version == 1` before
consuming other fields), `"pid"`, `"port"`, `"authToken"`, `"startTimeUtc"`,
`"app"`, `"version"`. Phase 4 readers that encounter `contract_version > 1` MUST
fail gracefully (do not attempt parse of unknown-version JSON).

---

## §Trace

v1.0.15 changes (adversary R74 F-R74-1 closure — hook_endpoints ellipsis placeholder + L-F-R63 Extension 4):
- F-R74-1 RESOLVED (HIGH — adversary R74 JSON array ellipsis placeholder in §GET /status
  schema sketch): The `hook_endpoints` field in the BC-DAEMON-002 JSON schema sketch
  (§Health and Status Endpoints) carried a literal `"..."` string as the third array
  element — a textual ellipsis placeholder instead of the full canonical enumeration.
  The canonical 5-endpoint set is established by BC-DAEMON-002 postcondition 1
  (PRD v1.8) and the §Body Size Limit router construction sketch (lines 157–161),
  which explicitly names all five routes:
  `/hooks/pre-tool-use`, `/hooks/notification`, `/hooks/stop`,
  `/hooks/session-start`, `/hooks/prompt-submit`.
  Fix: replaced `["/hooks/pre-tool-use", "/hooks/notification", "..."]` with
  the explicit 5-element array. The serialized array format is expanded to one
  string per line for readability.

- L-F-R63 Extension 4 — placeholder discipline extended to JSON array ellipsis:
  Prior placeholder discipline (L-F-R63 Extension 1 from v1.0.14) covered only
  ISO8601 timestamp format placeholders (`<ISO8601>`). F-R74-1 reveals a second
  category: JSON array ellipsis patterns (`"..."` as a stand-in for omitted array
  elements). Extension 4 codifies: any arch JSON schema sketch that represents an
  enumerable, finite array (all elements known at spec authoring time) MUST enumerate
  all elements explicitly. The `"..."` convention is acceptable ONLY when the array
  genuinely contains variable-length content (e.g., line 727 `"peers"` list in
  §Phase 4 Notes, where peer hostnames are runtime-variable). Arch-level enumerations
  like `hook_endpoints` (whose elements are fixed by router construction) are NOT
  variable-length and must not use `"..."`.

- Placeholder sweep (L-F-R63 Extension 4 scope — `"..."` patterns in arch JSON schemas):
  Post-fix grep results for `'"..."'` in `.factory/specs/architecture/`:
  (a) `SS-daemon-lifecycle.md` line 82: FIXED by this change (the subject of F-R74-1).
  (b) `SS-daemon-lifecycle.md` line 727 (Phase 4 Notes `"peers"` array): NOT a defect.
      `"..."` represents a variable host string value in an illustrative JSON object
      (runtime peer hostnames are unknowable at spec time). This is correct notation.
  (c) `SS-engine-module.md` line 1349: `"..."` in method-chaining prose example for
      `HookResponse::new(decision).with_diagnostic("...")`. This is a diagnostic
      message placeholder in a code prose example — not a schema enumeration defect.
      The `"..."` here means "some diagnostic string"; the actual value is caller-defined.
      NOT a defect under Extension 4.
  Result: 0 defective `"..."` ellipsis patterns remain in normative-current arch JSON
  schema sketches after this fix. The 2 remaining occurrences are each correctly
  variable-content and are explicitly documented above.

- Propagation requirements for orchestrator:
  **PRD:** PRD v1.8 `traces_to` frontmatter cites arch pin `v1.0.14`; product-owner
  must propagate arch pin v1.0.14 → v1.0.15 at normative-current PRD sites (traces_to
  field). No BC content change required — BC-DAEMON-002 postcondition already
  enumerates the 5 endpoints correctly; this fix brings the arch schema sketch into
  alignment with the BC.
  **VP:** VP `traces_to` frontmatter citing arch pin `v1.0.14`; formal-verifier must
  propagate pin v1.0.14 → v1.0.15. No VP content change required.
  **BC count: 22 — CONFIRMED unchanged.** No new BCs; no BCs removed.
- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) PG-3: §Health and Status Endpoints (EXISTS), §Phase 4 Notes (EXISTS), §Trace
      (EXISTS) — all §-anchor refs verified against actual headings in this document.
  (b) PG-4: all referenced sections confirmed to exist in normative body.
  (c) PG-5: historical §Trace entries unchanged. Post-write self-grep: 0 `"..."` array
      placeholder defects remain in normative-current JSON schema sketches. 0 L[0-9]+
      matches in this §Trace v1.0.15 entry.

v1.0.18 changes (adversary R88 F-R88-1 closure — §Phase 4 Notes contract_version enumeration):
- F-R88-1 RESOLVED (HIGH — adversary R88 §Phase 4 Notes lock-file field enumeration
  incomplete): The §Phase 4 Notes paragraph stated that the fields `"app"`, `"pid"`,
  `"port"`, `"authToken"`, `"startTimeUtc"`, `"version"` are stable across Phase 1 →
  Phase 4. This 6-field enumeration omitted `contract_version`, which is the seventh
  and most critical Phase 1 lock-file field. Per BC-LOCK-001 Postcondition 1 (PRD line
  602) the Phase 1 lock-file schema has 7 fields. Per BC-LOCK-001 Postcondition 2 (PRD
  line 606) `contract_version` is always the FIRST key in the JSON object. Per
  BC-DAEMON-005 Postcondition 4 (PRD line 334) the lock-file contains all 7 fields
  including `contract_version`. The omission created two failure modes for Phase 4
  implementers: (a) treating `contract_version` as a Phase-1-only field and removing
  it in Phase 4, breaking BC-LOCK-001 forward-compat; (b) assuming `contract_version`
  is implicit/non-contractual and not validating it.

  Fix (disposition **(a)** — extend §Phase 4 Notes enumeration to all 7 fields):
  - Before: "...the `\"app\"`, `\"pid\"`, `\"port\"`, `\"authToken\"`,
    `\"startTimeUtc\"`, `\"version\"` fields are stable across Phase 1 → Phase 4."
    (6 fields, `contract_version` absent)
  - After: "...the 7 Phase 1 fields are stable across Phase 1 → Phase 4:
    `\"contract_version\"` (forward-compatibility version sentinel, always FIRST key
    per BC-LOCK-001 Postcondition 2; Phase 4 readers MUST validate
    `contract_version == 1` before consuming other fields), `\"pid\"`, `\"port\"`,
    `\"authToken\"`, `\"startTimeUtc\"`, `\"app\"`, `\"version\"`. Phase 4 readers
    that encounter `contract_version > 1` MUST fail gracefully (do not attempt parse
    of unknown-version JSON)."

  The field ordering in the fix follows BC-LOCK-001 Postcondition 1 canonical order
  exactly: `contract_version` first (sentinel), then `pid`, `port`, `authToken`,
  `startTimeUtc`, `app`, `version`.

  Pre-burst grep for `contract_version` in this file confirmed the field was present
  at §Lock File Format (lines 437, 446, 449) and §BC Summary RTM (line 728) but NOT
  in §Phase 4 Notes — the fix adds it to §Phase 4 Notes for the first time.
  Post-burst grep confirms the new occurrence at the corrected §Phase 4 Notes
  paragraph.

  Adversary R88 report: `.factory/plans/adversary-pass-r88-phase1-fixed.md`.

- Propagation requirements (Extension 15):

  This burst bumps arch from v1.0.17 → v1.0.18. Downstream agents must propagate:

  **PRD (product-owner):** PRD frontmatter `traces_to` cites arch pin v1.0.17;
  product-owner must propagate v1.0.17 → v1.0.18 in PRD frontmatter + §7 RTM
  Architecture Source column + body lineage citations. Use canonical grep target:
  `grep -nE "v1\.0\.17|commit a798d51" /Users/jmagady/Dev/monocle/.factory/specs/prd.md`
  to enumerate sites.

  **VP (formal-verifier):** VP frontmatter `traces_to` cites arch pin v1.0.17;
  formal-verifier must propagate v1.0.17 → v1.0.18 in VP frontmatter + §Catalog
  Overview + per-VP `Traces to:` lines + §Coverage Matrix + §References item 2 +
  §Trace amendment. Use canonical grep target:
  `grep -nE "v1\.0\.17|commit a798d51" /Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md`

  **BC count: 22 — CONFIRMED unchanged.** No new BCs; no BCs removed.

- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) PG-3: §Phase 4 Notes (EXISTS), §Lock File Format (EXISTS), §Trace (EXISTS) —
      all §-anchor refs verified against actual headings in this document.
  (b) PG-4: all referenced sections confirmed to exist in normative body.
  (c) PG-5: historical §Trace entries unchanged. Post-write self-grep: 0 L[0-9]+
      matches in this §Trace v1.0.18 entry.

v1.0.17 changes (adversary R83 F-R83-1 site 2 closure — §BC Summary footer BC-DAEMON-005 0o700 propagation):
- F-R83-1 site 2 RESOLVED (HIGH — adversary R83 multi-site propagation gap): The
  F-R79-3 closure (v1.0.x chain) lifted the runtime_dir `0o700` owner-only mode
  contract from EC-052 into BC-DAEMON-005 §Postcondition 8. The §Start Sequence
  step 1 body at line 255 correctly states "Create the resolved directory with
  mode `0o700` if absent." However, the §BC Summary footer BC-DAEMON-005 row was
  NOT updated in the same burst — it described only the resolution chain and
  lock-file semantics, omitting the directory permission contract.

  Fix (disposition **(a)** — extend summary row to match body tier):
  - Before: "...platform-aware chain: <chain>; lock file created atomically via
    `tempfile::persist`; pid-liveness checked on startup; removed on clean shutdown"
  - After: "...platform-aware chain: <chain>; runtime_dir created with mode `0o700`
    owner-only (defense-in-depth with lock file `0o600`); lock file created atomically
    via `tempfile::persist`; pid-liveness checked on startup; removed on clean shutdown"

  The `0o600` lock-file mode is established by §Start Sequence step 6 and
  SS-conventions-anti-patterns.md §Atomic Writes. Naming it alongside `0o700`
  in the summary row captures the layered defense-in-depth posture: even if the
  runtime_dir permissions were somehow bypassed, the lock file itself would block
  reads by other users. Both modes were already specified in the arch body;
  this change propagates them to the summary-table tier for completeness.

  Adversary R83 report: `.factory/plans/adversary-pass-r83-phase1-fixed.md`.

- Propagation requirements for orchestrator:
  **PRD:** PRD `traces_to` frontmatter cites arch pin `v1.0.16`; product-owner
  must propagate arch pin v1.0.16 → v1.0.17 at normative-current PRD sites
  (traces_to field). No BC content change required — BC-DAEMON-005 postcondition 8
  already specifies 0o700; this fix brings the summary-table footer into alignment
  with the body.
  **VP:** VP `traces_to` frontmatter citing arch pin `v1.0.16`; formal-verifier
  must propagate pin v1.0.16 → v1.0.17. No VP content change required from this
  arch fix (sibling formal-verifier agent handles VP §Catalog Overview + §Extension 14
  codification in parallel).
  **BC count: 22 — CONFIRMED unchanged.** No new BCs; no BCs removed.

- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) PG-3: §Start Sequence (EXISTS), §Behavioral Contract Summary (EXISTS),
      §Trace (EXISTS) — all §-anchor refs verified against actual headings.
  (b) PG-4: all referenced sections confirmed to exist in normative body.
  (c) PG-5: historical §Trace entries unchanged. Post-write self-grep: 0 L[0-9]+
      matches in this §Trace v1.0.17 entry.

v1.0.16 changes (adversary R75 F-R75-2 closure + Obs-R75-1 drain clarification):
- F-R75-2 RESOLVED (MEDIUM — adversary R75 §Start Sequence Rationale Windows scope
  overstatement vs PRD NFR-008): The §Start Sequence step 1(c) Rationale paragraph
  previously stated the `data_local_dir()` fallback "provides a correct,
  standards-compliant runtime state location on macOS (`~/Library/Application
  Support/monocle/`) and Windows (`%APPDATA%/monocle/`)" without qualification.
  PRD NFR-008 (line 1210) lists `macOS + Linux` as the primary platform targets
  (darwin/linux × amd64/arm64). PRD §8.7 (line 1320) explicitly states "Windows is
  a secondary build target." The unqualified Windows claim in the arch rationale
  overstated Phase 1 Windows support beyond what the PRD contracts.

  Fix (disposition **(a)** — tighten rationale to match PRD scope):
  - Before: "...standards-compliant runtime state location on macOS
    (`~/Library/Application Support/monocle/`) and Windows (`%APPDATA%/monocle/`)."
  - After: "...standards-compliant runtime state location on macOS
    (`~/Library/Application Support/monocle/`). Windows is a secondary build target
    per PRD §8.7; the same `data_local_dir()` fallback resolves to
    `%APPDATA%/monocle/` on Windows but Phase 1 CI does not formally validate
    Windows behavior per NFR-008's `macOS + Linux` target scope."

  The `data_local_dir()` fallback code itself is unchanged — Windows still resolves
  correctly at runtime; only the rationale is tightened to accurately represent the
  validation scope.

- Obs-R75-1 RESOLVED (ADVISORY — §Drain step 4 "append mode" + `tempfile::persist`
  internal contradiction): The original prose described the ring buffer flush as
  "append mode, `tempfile::persist` for the current-segment file." These two
  descriptions are contradictory: `tempfile::persist` is an atomic rename (replace),
  not append-mode I/O. A reader implementing against this spec could not determine
  which semantic was intended.

  Fix (disposition **(a)** — explicit two-phase write pattern):
  The prose now describes the correct implementation: read existing file content
  into memory, append the in-memory ring buffer records, then write the combined
  result via `tempfile::persist` over the destination path. This is atomic-replace
  semantics with preserved prior content — both properties hold, with no ambiguity.
  The `tempfile::persist` atomic-replace convention from SS-conventions-anti-patterns.md
  §Atomic Writes is preserved; the "append mode" description is retired.

- Propagation requirements for orchestrator:
  **PRD:** PRD `traces_to` frontmatter cites arch pin `v1.0.15`; product-owner
  must propagate arch pin v1.0.15 → v1.0.16 at normative-current PRD sites
  (traces_to field). No BC content change required — this fix affects only
  arch rationale prose and §Drain step 4 implementation description.
  **VP:** VP `traces_to` frontmatter citing arch pin `v1.0.15`; formal-verifier
  must propagate pin v1.0.15 → v1.0.16. No VP content change required.
  **BC count: 22 — CONFIRMED unchanged.** No new BCs; no BCs removed.

- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) PG-3: §Start Sequence (EXISTS), §Drain (EXISTS), §Trace (EXISTS) — all
      §-anchor refs verified against actual headings in this document.
  (b) PG-4: all referenced sections confirmed to exist in normative body.
  (c) PG-5: historical §Trace entries unchanged. Post-write self-grep: 0 L[0-9]+
      matches in this §Trace v1.0.16 entry.

v1.0.14 changes (adversary R72 F-R72-1 closure — partial-fix regression of F-R70-2):
- F-R72-1 RESOLVED (HIGH — adversary R72 JSON schema sketch timestamp format
  propagation gap): F-R70-2 (PRD v1.6/v1.7 + VP v1.6/v1.7 closure chain) tightened
  BC-DAEMON-006 invariant 1 and the VP-DAEMON-006 regex to mandate
  `YYYY-MM-DDTHH:MM:SS.sssZ` (mandatory millisecond precision). The §BC Summary
  footer declares this architecture document as "source-of-truth for invariants,
  protocol decisions, and security rationale." Three JSON schema sketches in this
  document still carried the generic `<ISO8601>` placeholder, leaving the arch SoT
  inconsistent with the tighter BC + VP. This constitutes a partial-fix regression:
  the BC and VP were tightened but the arch schema sketches were not propagated in
  the same burst. F-R72-1 closes the gap.

  Three sites tightened (disposition **(a)** for all — cross-field uniformity):

  **Site 1 — §Health and Status Endpoints /status `last_hook_ts` block (5 fields):**
  - Before: `"<ISO8601 or null>"` (5 occurrences, one per hook type)
  - After: `"<YYYY-MM-DDTHH:MM:SS.sssZ or null>"` + inline annotation referencing
    EC-044 (PRD v1.7), BC-DAEMON-002, and F-R72-1 cross-field uniformity.
  - Rationale: EC-044 (PRD v1.7 line 185) already specifies this exact format for
    `last_hook_ts`; the arch sketch now matches its own downstream BC/EC.

  **Site 2 — §Start Sequence step 6 lock file `startTimeUtc`:**
  - Before: `"startTimeUtc": "<ISO8601>"`
  - After: `"startTimeUtc": "<YYYY-MM-DDTHH:MM:SS.sssZ>"` + inline annotation
    referencing BC-DAEMON-002 / EC-044, BC-DAEMON-006, and F-R72-1 uniformity.
  - Disposition: **(a) tighten to millisecond precision for cross-field uniformity.**
    Rationale: no architectural reason for `startTimeUtc` to carry seconds-only
    precision while `last_hook_ts` and `shutdown_utc` both mandate milliseconds.
    Cross-field uniformity eliminates a parser asymmetry (a parser that handles all
    three fields can use a single `chrono` format string). The Phase 1 Rust
    implementation will use `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")`
    uniformly across all three timestamp fields. Seconds-only precision would require
    a distinct format string for `startTimeUtc` alone — unnecessary complexity.

  **Site 3 — §Drain step 5 crash-recovery `shutdown_utc`:**
  - Before: `"shutdown_utc": "<ISO8601>"`
  - After: `"shutdown_utc": "<YYYY-MM-DDTHH:MM:SS.sssZ>"` + inline annotation
    referencing BC-DAEMON-006 invariant 1 (PRD v1.7), VP-DAEMON-006 regex
    (`^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`), and cross-field uniformity.
  - This site is directly mandated by BC-DAEMON-006 invariant 1; the prior generic
    placeholder was the root cause of F-R72-1 (the BC was tightened but the arch
    sketch was not updated in the same burst).

- L-F-R63 Extension 1 propagation discipline applied to arch JSON schema sketches:
  All three timestamp fields in schema sketches are now the SoT-authoritative precise
  format. Future BC/VP tightening of timestamp formats MUST propagate to all schema
  sketches in the same burst (Extension 1: update all normative-current sites).
- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) PG-3 sweep: §-anchor refs used throughout; no bare L-numbers; no directional
      qualifiers (after/above/below).
  (b) PG-4 sweep evidence: §Health and Status Endpoints (EXISTS), §Start Sequence
      (EXISTS), §Drain (within §Daemon Lifecycle Protocol, EXISTS), §BC Summary
      (EXISTS), §Trace (EXISTS).
  (c) PG-5 sweep: historical §Trace entries unchanged. `<ISO8601>` grep result: 0
      remaining occurrences in normative-current body (all three sites resolved). 0
      remaining generic timestamp placeholders in any JSON schema sketch. VERIFIED.
  (d) §Behavioral Contract Summary BC-DAEMON-006 row: prose description does not
      carry a timestamp format claim (references §Daemon Lifecycle Protocol for
      detail) — no change needed to summary table.
  (e) Phase 4 Notes paragraph: references `startTimeUtc` by field name only, no
      format claim — no change needed.
  (f) Post-write self-grep: 0 `<ISO8601>` matches in normative-current body. 0
      `L[0-9]+` matches in this §Trace v1.0.14 entry.
- Propagation requirements for orchestrator:
  **PRD:** PRD v1.7 `traces_to` frontmatter cites arch pin `v1.0.13`; product-owner
  must propagate arch pin v1.0.13 → v1.0.14 at all normative-current PRD sites
  (traces_to field + any §Trace entry that cites the arch version). EC-044
  (last_hook_ts format) is already correctly specified in PRD v1.7 — no BC content
  change required.
  **VP:** VP v1.7 `traces_to` frontmatter cites arch pin `v1.0.13`; formal-verifier
  must propagate arch pin v1.0.13 → v1.0.14 at all normative-current VP sites. No VP
  content change required — VP-DAEMON-006 regex and BC-DAEMON-006 invariant 1 are
  already correctly specified. Pin propagation only.
  **BC count: 22 — CONFIRMED unchanged.** No new BCs added; no BCs removed.

v1.0.13 changes (adversary R71 F-R71-2 + F-R71-3 + F-R71-4 closure):
- F-R71-2 RESOLVED (HIGH — adversary R71 stale test name): Two arch sites cited
  `test_BC_DAEMON_004_exit_codes` as the BC-DAEMON-004 verification test name. The
  canonical name per PRD v1.6 §3 BC-DAEMON-004 §Verification and VP v1.6 is
  `test_BC_DAEMON_004_exit_codes_posix_distinct`. Authority per §BC Summary footer:
  "the PRD is source-of-truth for canonical test names." Fixed at both arch sites:
  (1) §Hard Shutdown BC-DAEMON-004 verification prose block; (2) §Trace v1.0.12
  BC-DAEMON-004 rationale sentence. No behavioral change — only the test-name
  citation corrected to match the PRD/VP canonical identifier.
- F-R71-3 RESOLVED (MEDIUM — adversary R71 NFR-008 mis-anchor): Four arch sites
  used "macOS is the primary target" / "lists macOS as the primary target" framing
  that implies macOS is the sole primary target. PRD NFR-008 (line 1210) specifies
  `macOS + Linux (darwin/linux × amd64/arm64)` — coequal, no sole-primary
  designation. Architect disposition: **(a)** rephrase all four sites to make
  explicit that NFR-008 lists macOS among the primary targets alongside Linux.
  Fixed sites:
  (1) §Scope line: "NFR-008 lists macOS as the primary target" →
      "NFR-008 lists macOS among the primary targets (`macOS + Linux`, darwin/linux × amd64/arm64)".
  (2) §Start Sequence step 1 Rationale: "macOS is the primary target platform (NFR-008)" →
      "macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64)".
  (3) §Trace v1.0.12 F-R70-1 sentence: "NFR-008 lists macOS as the primary target" →
      "NFR-008 lists macOS among the primary targets (`macOS + Linux`)".
  (4) §Trace v1.0.12 F-R70-1 rationale: "forcing every macOS user (primary target)" →
      "forcing every macOS user (a primary target, per NFR-008 `macOS + Linux`)".
  All normative prose and historical trace citations now accurately reflect
  NFR-008's coequal macOS + Linux scope. The runtime_dir fallback chain rationale
  is unaffected — the fallback is still required because `runtime_dir()` returns
  `None` on macOS regardless of whether macOS is sole-primary or co-primary.
- F-R71-4 RESOLVED (MEDIUM — adversary R71 dep-pin ambiguity):
  F-R71-4a: VP-DAEMON-005 cited `tower 0.5` with "per SS-deps-pin-manifest.md" but
  tower is not pinned in the manifest — it is a transitive dependency of axum 0.8 and
  never used directly as a workspace dependency in monocle. Architect disposition:
  **(b)** leave manifest unchanged; tower remains appropriately transitive through
  axum 0.8. The formal-verifier / test-writer must drop the "per manifest" citation
  from any VP referencing tower; tower's version is constrained by axum 0.8's
  dependency resolution, not by a direct workspace pin. No manifest change required.
  F-R71-4b: VP-DAEMON-005 contained a "pending-architect-review" Principle 6
  violation — an unresolved choice between `nix 0.30` and `libc 0.2` for POSIX
  signal handling in BC-DAEMON-005 postcondition 3 (stale-pid detection).
  Architect disposition: **`nix 0.30`** (typed wrapper crate preferred over raw
  `libc` for `Signal::None` send pattern). Rationale: `nix::sys::signal::kill(pid,
  None)` is the idiomatic, type-safe Rust API for pid-liveness testing without
  signal delivery; using raw `libc::kill(pid, 0)` bypasses the type system and
  requires unsafe. `nix 0.30` is the latest stable release (verified 2026-05-14
  against crates.io). `nix 0.30` added to SS-deps-pin-manifest.md as a workspace
  caret pin. BC-DAEMON-005 postcondition 3 implementation MUST use
  `nix::sys::signal::kill(Pid::from_raw(pid), None)`.
- Propagation requirements for orchestrator:
  PRD: NFR-008 description at PRD line 328 may carry the same sole-primary framing;
  product-owner to audit and correct if present.
  PRD: arch pin bump v1.0.12 → v1.0.13 to record in any PRD traceability matrix.
  VP: F-R71-1 (directories 5→6, 2 VP sites) — out of current arch scope; VP owner
  to resolve. F-R71-4a: formal-verifier to drop "per manifest" tower citation from
  VP-DAEMON-005. F-R71-4b: formal-verifier to update VP-DAEMON-005 to name `nix 0.30`
  as the binding crate. F-R71-5 (if any VP placeholder present) — VP owner to resolve.
  VP and PRD changes require product-owner / formal-verifier dispatch; not in arch scope.
  BC count: **22 — CONFIRMED unchanged.** No new BCs; no BCs removed.

v1.0.12 changes (adversary R70 F-R70-1 macOS runtime_dir + F-R70-3 POSIX exit-code semantics):
- F-R70-1 RESOLVED (HIGH — adversary R70 cross-platform invariant): §Scope and §Start
  Sequence step 1 mandated `directories::ProjectDirs::runtime_dir()` as the sole runtime
  directory resolution source with no fallback. `runtime_dir()` returns `None` on macOS
  and Windows by platform-ABI design, not due to misconfiguration. NFR-008 lists macOS
  among the primary targets (`macOS + Linux`). An implementer following the prior step 1 spec had no defined
  behavior when `None` was returned — bifurcated implementations would result, breaking
  BC-DAEMON-005 and BC-LOCK-001 on macOS. Architect disposition: **(c) hybrid platform
  fallback chain with env override.** Step 1 replaced with a four-path resolution chain:
  (a) `MONOCLE_RUNTIME_DIR` env override — operator escape hatch for containers and
  non-standard deployments; (b) `ProjectDirs::runtime_dir()` — XDG-compliant path used
  on Linux; (c) `ProjectDirs::data_local_dir()` — platform-appropriate fallback used on
  macOS (`~/Library/Application Support/monocle/`) and Windows (`%APPDATA%/monocle/`);
  (d) fail-fast with `DaemonStartError::RuntimeDirUnresolvable` + exit 1 if all three
  resolution paths return `None` (e.g., no home directory at all). Rationale for
  disposition (c) over (a) pure fail-fast: `runtime_dir()` returning `None` on macOS
  is a platform design choice, not a configuration failure; forcing every macOS user
  (a primary target, per NFR-008 `macOS + Linux`) to set `MONOCLE_RUNTIME_DIR` violates the zero-config startup
  requirement. Rationale for disposition (c) over (b) silent fallback: the env override
  is a necessary operator escape hatch for containerized and custom deployments.
  Asymmetry with BC-ENGINE-002-ERR is intentional: `BaseDirs::new() == None` signals
  a genuine system-configuration failure (no home directory), warranting fail-fast;
  `ProjectDirs::runtime_dir() == None` on macOS is expected platform behavior,
  warranting a documented fallback. BC count: **22 unchanged** — BC-DAEMON-005 is
  updated in place (precondition 2 + §Start Sequence step 1 elaboration); no new BC
  added. §Scope updated to describe the resolution chain. `resolve_runtime_dir()`
  implementation sketch added to step 1 for implementer clarity.
- F-R70-3 RESOLVED (MEDIUM — adversary R70 POSIX exit-code semantic correctness):
  §Hard Shutdown exit codes block specified exit `130` for "second SIGTERM during
  drain." POSIX convention: signal N → exit 128+N; SIGINT=2 → 130; SIGTERM=15 → 143.
  The prior spec encoded Ctrl-C (SIGINT) semantics for a SIGTERM hard-kill scenario.
  systemd, k8s, and CI monitoring would misinterpret exit 130 as SIGINT when the
  actual trigger was SIGTERM. Architect disposition: **(c) distinguish three hard-kill
  triggers.** Exit codes now:
  - `0` — graceful drain complete (unchanged).
  - `130` — SIGINT (signal 2) hard-kill during drain (128+2 POSIX; Ctrl-C second press).
  - `143` — SIGTERM (signal 15) hard-kill during drain (128+15 POSIX; systemd/k8s second SIGTERM).
  - `2` — admin `POST /shutdown` second-call during drain (monocle-specific programmatic
    code; chosen outside POSIX 128+N space and distinct from startup-failure exit 1).
  - `1` — daemon startup failure (unchanged semantic; now explicitly listed).
  BC-DAEMON-004 summary row updated to enumerate all five exit codes. Hard Shutdown
  step 6 updated to distinguish signal type for exit-code selection. BC-DAEMON-004
  postcondition added inline in §Hard Shutdown with verification test reference
  (`test_BC_DAEMON_004_exit_codes_posix_distinct` in `monocle-runtime/tests/daemon_lifecycle.rs`).
  Rationale for disposition (c) over (a) simple 143 substitution: distinguishing SIGINT
  vs SIGTERM vs admin-API force-stop preserves maximum diagnostic information for ops
  tooling; the code paths through the `tokio::select!` loop already distinguish SIGTERM
  vs `ctrl_c()` signals, so the only implementation delta is writing the correct exit
  code at the callsite.
- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) §Scope updated — version-stable description of resolution chain added; no bare
  L-numbers; no directional qualifiers.
  (b) §Behavioral Contract Summary — BC-DAEMON-004 and BC-DAEMON-005 rows updated
  in place; no new rows; BC count 22 verified (10 daemon-lifecycle IDs: DAEMON-001
  through DAEMON-006, RING-001, AUTH-001, AUTH-002, LOCK-001).
  (c) PG-3 sweep: §Start Sequence (EXISTS heading), §Hard Shutdown (EXISTS heading),
  §Behavioral Contract Summary (EXISTS heading), §Scope (EXISTS heading), §Trace
  (EXISTS heading) — all §-anchor refs verified against actual headings.
  (d) PG-4 sweep evidence: §Scope EXISTS, §Start Sequence EXISTS, §Hard Shutdown
  EXISTS, §Behavioral Contract Summary EXISTS, §Trace EXISTS.
  (e) PG-5 sweep: no historical normative-current claims modified; §Trace v1.0.11
  and earlier entries unchanged.
  (f) Cross-artifact propagation requirements (for orchestrator dispatch):
  **PRD propagation required:** BC-DAEMON-004 postcondition (exit codes) must be
  updated to enumerate all five codes (0/130/143/2/1) with POSIX rationale; the PRD
  is the canonical test-vector source. BC-DAEMON-005 precondition 2 (runtime dir
  resolution) must be updated to reflect the three-path chain + env override. These
  are content changes in the PRD requiring `product-owner` dispatch.
  **VP propagation required:** VP-DAEMON-004 and VP-DAEMON-005 (if they exist) must
  be updated to reflect the new exit codes and resolution chain in their proof
  strategies and test vectors. `formal-verifier` / `test-writer` dispatch as applicable.
  **BC count: 22 — CONFIRMED unchanged.** No new BCs added; no BCs removed.
  Post-write self-grep: 0 L[0-9]+ matches in this §Trace v1.0.12 entry.

v1.0.11 changes (adversary R65 F-R65-1/2/3 content closure + propagation sweep):
- F-R65-1 RESOLVED (HIGH — adversary R65 pass 1 attempt 2): BC-AUTH-002 lead-in prose
  at §Behavioral contracts stated "Three auth failure modes are specified:" but the
  BC-AUTH-002 table immediately below contained exactly two rows (Missing header /
  Invalid token). Similarly, §Behavioral Contract Summary BC-AUTH-002 row opened with
  "Three auth failure modes:". Root cause: F-R62-8 (v1.0.8) collapsed the originally
  distinct format / mismatch rows into a single "Invalid token" row — reducing the table
  to 2 rows — but the lead-in count words at both sibling sites were not updated in that
  same burst. The L-F-R63-PARTIAL-FIX propagation discipline was not yet codified at the
  time of F-R62-8; these sibling sites were therefore a pre-codification gap. Fix: "Three"
  → "Two" at both body-prose sites.
- F-R65-2 RESOLVED (CRITICAL — adversary R65 pass 1 attempt 2): BC-AUTH-002 §Behavioral
  contracts §Verification block contained two conflicting statements about the same
  scenario (an inbound request carrying `Authorization: Bearer` instead of
  `X-Monocle-Authorization`). The paragraph describing Phase 4 OAuth2 federation
  tokens stated the response body is `{"error":"invalid_auth_token"}` (bearer-as-invalid).
  The test vector bullet immediately below in the same block stated
  `Authorization: Bearer fake` → `{"error":"missing_auth_token"}` (bearer-as-missing).
  Production-grade reasoning: from the Phase 1 daemon's perspective, a request carrying
  `Authorization: Bearer` has NO `X-Monocle-Authorization` header — the structurally
  correct disposition is `missing_auth_token` (Rule 1: missing header). The Phase 4
  OAuth2 bearer header is a different, unrecognized header for Phase 1 endpoints; its
  presence does not constitute a "header present but invalid" scenario. The bearer-as-invalid
  paragraph (lines 320-321 at time of this fix) contained a logic error: it treated presence
  of `Authorization: Bearer` as equivalent to presence of `X-Monocle-Authorization`. Fix:
  bearer-as-invalid body changed from `{"error":"invalid_auth_token"}` to
  `{"error":"missing_auth_token"}`; parenthetical updated to reflect the correct semantic
  "(no `X-Monocle-Authorization` header present; `Authorization: Bearer` is a different,
  unrecognized header — Phase 4 OAuth2 uses a separate federation channel and does not
  reuse the Phase 1 HTTP endpoints)". Aligns with: test vector bullet in same block
  (`Authorization: Bearer fake` → missing); PRD v1.3 BC-AUTH-002 postcondition 3 +
  Canonical Test Vector row 5; VP v1.3 §VP-AUTH-002 probe 5.
- F-R65-3 RESOLVED (HIGH — closed by F-R65-2 fix): Cross-artifact contradiction between
  arch and PRD/VP on Bearer disposition. After F-R65-2 fix, arch aligns with PRD v1.3
  BC-AUTH-002 and VP v1.3 VP-AUTH-002. No independent change required.
- Propagation sweep (L-F-R63-PARTIAL-FIX discipline applied):
  (a) "Three/three" auth failure modes — body prose grep result: 2 sites fixed
  (BC-AUTH-002 lead-in at §Behavioral contracts; §Behavioral Contract Summary row);
  §Trace v1.0.8 contains "three-case table" and "three middleware branches" —
  HISTORICAL per PG-5 (describing what was introduced at v1.0.8 time); NOT changed.
  (b) Bearer disposition `invalid_auth_token` — body prose grep result: 1 site fixed
  (bearer-paragraph in §Behavioral contracts §Verification); §Trace v1.0.8 references
  "three-case table" — HISTORICAL; NOT changed.
  (c) `invalid_auth_token_format` RETIRED — grep result: 2 sites in §Trace v1.0.8
  body, both clearly marked RETIRED in historical context; NOT changed (PG-5 exempt).
  (d) SS-deps-pin-manifest.md — grep for "SS-daemon-lifecycle\.md v" confirmed 0
  version-pinned citations (matches prior v1.0.10 sweep finding); no update needed.
  PG-2 count-verification sweep: "Two" count matches 2 actual table rows in
  BC-AUTH-002 table — VERIFIED. PG-3 compliant: §-anchor refs used throughout;
  no bare L-numbers; no directional qualifiers. PG-4 sweep evidence: §Behavioral
  Contract Summary (EXISTS heading), §Start Sequence (EXISTS heading), §Trace
  (EXISTS heading). PG-5 sweep evidence: §Trace v1.0.8 "three" instances — classified
  HISTORICAL (PG-5 exempt); no normative-current count changes introduced. Post-write
  self-grep: 0 L[0-9]+ matches in this §Trace v1.0.11 entry.

v1.0.10 changes (consistency R3 R3-001 closure + oscillation-prevention sweep):
- R3-001 RESOLVED (MEDIUM — consistency-validator R3 finding, commit ba62a15):
  §Behavioral Contract Summary footer cited "PRD v1.1, commit f855835" as the
  source-of-truth pointer for canonical test names, test-file paths, error taxonomy,
  and edge case catalog. PRD is at v1.2 (commit 5a49b0b) and the footer's version pin
  was factually stale. Root cause: normative-current version pins in architecture body
  prose go stale whenever PRD bumps for any reason, creating an oscillation cycle
  (F-R62 → R63 → R3-001 chain). Fix: §Behavioral Contract Summary footer rephrased
  to version-stable Pattern B — historical formalization anchor preserved (PRD v1.1,
  commit f855835) plus file-path version-stable pointer (`.factory/specs/prd.md`).
  Future PRD version bumps will NOT make this sentence stale. Lesson applied:
  L-F-R63-PARTIAL-FIX (cycles/cycle-001/lessons.md) propagation discipline — the
  full propagation checklist was applied:
  (a) §Behavioral Contract Summary footer — normative-current version pin removed;
  (b) normative body lines for BC-AUTH-001 and BC-AUTH-002 §Verification cite
  "PRD v1.1 §7 RTM" as historical fix-provenance for the F-R62-4 path
  canonicalization — classified as PG-5 historical anchors, not normative-current
  claims; kept unchanged;
  (c) §Trace history entries contain PRD v1.1 references — historical per PG-5
  exemption; no changes;
  (d) SS-deps-pin-manifest.md cites SS-daemon-lifecycle.md without a version pin
  (grep confirmed zero "SS-daemon-lifecycle\.md v" matches) — no update needed.
  PG-3 compliant: §-anchor refs used; no bare L-numbers; no directional qualifiers.
  PG-4 sweep evidence: §Behavioral Contract Summary (EXISTS heading), §Start Sequence
  (EXISTS heading), §Trace (EXISTS heading). PG-5 sweep evidence: §Behavioral
  Contract Summary footer — 1 normative-current pin fixed to version-stable;
  body lines BC-AUTH-001/002 §Verification — classified historical (PG-5 compliant,
  retained); §Trace history entries — PG-5 exemption confirmed. Post-write self-grep:
  0 L[0-9]+ matches in this §Trace v1.0.10 entry.

v1.0.9 changes (F-R62-4 back-propagation closure, adversary R63 F-R63-adv-2 + consistency R2 F-R63-cons-3):
- F-R63-cons-3 RESOLVED (MEDIUM — consistency-validator R2 finding): §Behavioral
  Contract Summary footer contained future-tense language ("The Phase 1 PRD will
  formalize...") that became retrospectively false when PRD v1.1 (commit f855835)
  formalized all 10 daemon-lifecycle BCs with preconditions, postconditions,
  invariants, edge cases, canonical test vectors, and verification stubs. Footer
  updated to past-tense with explicit authority split: this architecture artifact
  remains source-of-truth for invariants, protocol decisions, and security
  rationale; PRD v1.1 is source-of-truth for canonical test names, test-file
  paths, error taxonomy, and edge case catalog.
- F-R63-adv-2 partial RESOLVED (MEDIUM — adversary R63 stale path): BC-AUTH-002
  §Behavioral contracts §Verification block cited `monocle-runtime/tests/auth.rs`
  (the pre-F-R62-4 single-file path). F-R62-4 (PRD v1.1 §7 RTM) canonicalized
  the split: BC-AUTH-001 → `monocle-runtime/tests/auth_token_lifecycle.rs`;
  BC-AUTH-002 → `monocle-runtime/tests/auth_header_rejection.rs`. Architecture is
  the last artifact on the old single-file path. Change: BC-AUTH-002 §Verification
  updated to `auth_header_rejection.rs` with cross-reference to
  `auth_token_lifecycle.rs` for BC-AUTH-001 round-trip coverage; test name
  `test_BC_AUTH_002_auth_header_validation_all_failure_modes` added inline (PRD
  v1.1 §7 RTM canonical). BC-AUTH-001 §Verification sentence updated to add
  explicit test file path `auth_token_lifecycle.rs` and test name
  `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip` (PRD v1.1 §7 RTM
  canonical; F-R62-4). Source-of-truth: PRD v1.1 §7 RTM + VP v1.1
  §Coverage Matrix (commit 8454ff2). PG-3 compliant: §-anchor refs used;
  no bare L-numbers; no directional qualifiers. PG-4 sweep evidence:
  §Behavioral Contract Summary (EXISTS ✓ heading), §Daemon Lifecycle Protocol
  (EXISTS ✓ heading), §Start Sequence (EXISTS ✓ heading). Note: "§Behavioral
  contracts" cited in §Trace v1.0.8 final sentence is a bold-prose label within
  §Start Sequence — not a heading; cited here only as a prose location reference,
  consistent with the pre-existing v1.0.8 precedent in this document.

v1.0.8 changes (fix-burst F-R62, finding F-R62-8 MED — architect adjudication):
- F-R62-8 RESOLVED (MED — adversary finding R62): PRD at commit c69518d introduced
  `E-AUTH-002 {"error":"missing_auth_token"}` and `E-AUTH-003 {"error":"invalid_auth_token"}`
  in §Section 5 and edge cases EC-008/EC-009 in BC-AUTH-002, none of which were specified in
  SS-daemon-lifecycle.md v1.0.7. The architecture defined only `invalid_auth_token_format` for
  the single BC-AUTH-002 case (non-prefixed header). This was a PRD invention of contract surface
  beyond architecture authorization. Architect disposition chosen: **(c) mixed approach** —
  two distinct error bodies: `missing_auth_token` for absent header (structural precondition
  failure, not an auth attempt; diagnostic value with zero security cost) and `invalid_auth_token`
  for any value-present failure (format failure OR secret mismatch, intentionally collapsed into
  one body to eliminate the format-vs-mismatch enumeration vector). The third PRD invention
  `invalid_auth_token_format` is RETIRED — no body of that name exists in the architecture.
  Security rationale in §Start Sequence §Behavioral contracts BC-AUTH-002 (threat model:
  localhost-only, same-user adversary already has lock-file read access; defence-in-depth
  collapse of Rules 2+3 blocks information leak to adversaries with unexpected network access
  but no filesystem access). Auth middleware implementation updated to `validate_auth_header`
  returning `AuthError::Missing` or `AuthError::Invalid`. BC-AUTH-002 §Behavioral Contract
  Summary row expanded to reflect three-case table. Verification test vectors updated to 6
  cases covering all three middleware branches.

v1.0.7 changes (round-53.1 fix F-R53-adv-1 MEDIUM):
- F-R53-adv-1 RESOLVED (MEDIUM — adversary finding R53): §Trace v1.0.6 rationale sentence
  (item 2 of the F-R30-2 rationale) cited `SS-forward-compatibility.md §Analysis` as the
  source for `#[non_exhaustive]` being the production-grade forward-compat mechanism for
  `HookEventRecord`. No `#`/`##`/`###`/`####` heading named "Analysis" exists in
  SS-forward-compatibility.md; the string appears only as bold paragraph labels
  (`**Analysis — Sealed trait:**`, `**Analysis — #[non_exhaustive] on Phase 1 enums:**`)
  within `#### Item P3-1` sections. Corrected to `§Item P3-1` — the unique heading prefix
  that covers the `#[non_exhaustive]` rationale for Phase 1 enums. Option (a) chosen over
  Option (b) `§Phase 3 Forward-Compatibility Analysis` because the citation is specifically
  about the `#[non_exhaustive]` sub-question within P3-1, and `§Item P3-1` is the most
  semantically precise enclosing heading. PG-4 §-heading-existence compliance restored.

v1.0.6 changes (round-30 fix F-R30-2 MEDIUM):
- F-R30-2 RESOLVED (MEDIUM — adversary finding): `HookEventRecord` was defined in
  §Daemon Lifecycle Protocol §Drain with `#[derive(Debug, Clone, Serialize, Deserialize)]`
  but NO `#[non_exhaustive]` attribute. The v1.0.5 trace entry for F-R28-4 stated that
  `HookEventRecord::new(...)` was added using "the same `#[non_exhaustive]` / E0639 reasoning
  as engine-module structs — integration tests compile as separate binaries." This was
  self-referentially broken: the reasoning cited the attribute's necessity but the attribute
  was absent from the struct definition. Fix: `#[non_exhaustive]` added to `HookEventRecord`
  above the `#[derive(...)]` line. Rationale: (1) `monocle-runtime/tests/jsonl_ring.rs`
  compiles as a separate `[[test]]` binary; E0639 applies — struct literal construction of
  `HookEventRecord` outside `monocle-runtime` is forbidden unless `#[non_exhaustive]` is
  absent, but a constructor is already present for exactly this reason. (2) Phase 2
  ring-buffer format evolution (adding fields to `HookEventRecord`) requires either
  `#[non_exhaustive]` or a SemVer-major version bump; `#[non_exhaustive]` is the correct
  production-grade forward-compat mechanism per SS-forward-compatibility.md §Item P3-1.
  (3) Consistency: all other structs in the constructor audit table in SS-engine-module.md
  carry `#[non_exhaustive]` (EngineMetadata, ProcessSnapshot, EnrichedSession, HookResponse,
  SpawnArgs, SessionHandle, EngineVersion); HookEventRecord should be consistent.
  The existing `HookEventRecord::new(...)` constructor is unchanged — it was already present
  to satisfy E0639; the attribute addition makes the intent explicit and the spec internally
  consistent. Cross-reference: SS-engine-module.md §Cross-Crate Constructor Audit table
  v1.1.9 now includes `HookEventRecord` as row 8.

v1.0.5 changes (round-29 fix F-R28-4 MEDIUM):
- F-R28-4 RESOLVED (MEDIUM — adversary finding): `HookEventRecord` was referenced by
  BC-RING-001's verification body ("unit test serializes a `HookEventRecord`") but was
  defined nowhere in the spec corpus. An implementer following BC-RING-001 would not know
  what `HookEventRecord` is, what fields it contains, or how to construct it. Fix: a full
  `HookEventRecord` struct definition added to §Daemon Lifecycle Protocol §Drain, immediately
  preceding the BC-RING-001 contract statement. The struct is placed in `monocle-runtime::ring`
  (NOT `monocle-core`) because the ring buffer is a daemon runtime artifact, not part of the
  core ABI surface. Fields match the JSONL example record exactly:
  `format_version: u32` (first, always `1` in Phase 1), `session_id: String`,
  `timestamp_micros: i64`, `pid: u32`, `hook_type: String`, `tool_name: Option<String>`,
  `tool_input: Option<serde_json::Value>`. A `pub fn new(...)` constructor is provided (same
  `#[non_exhaustive]` / E0639 reasoning as engine-module structs — integration tests compile
  as separate binaries). The module-level const `RING_FORMAT_VERSION: u32 = 1` is the single
  source of truth for the format version value. BC-RING-001 verification body updated to use
  `HookEventRecord::new(...)` explicitly. Cross-reference: SS-engine-module.md §Trace v1.1.8
  F-R28-2 entry notes that F-R28-4 is resolved in this document.
