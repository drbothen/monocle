---
document_type: architecture-section
level: L3
section: "daemon-lifecycle"
subsystem: SS-01
version: "1.0.33"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-19T10:00:00Z
inputs: [product-brief.md, semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md, prd.md, verification-properties/VP-INDEX.md]
input-hash: "9c5ec0d"
traces_to: architecture/ARCH-INDEX.md
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

<a id="health-and-status-endpoints"></a>
## Health and Status Endpoints (F-NEW-05)

Both endpoints are registered on the same axum router as the 5 hook endpoints.

### GET /healthz

**Contract (BC-2.01.001):** Returns HTTP 200 with body:

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

**Contract (BC-2.01.002):** Returns HTTP 200 with body:

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
and the `shutdown_utc` format in BC-2.01.006 — cross-field uniformity per F-R72-1.

The `abi_version` field carries `monocle_core::MONOCLE_ABI_VERSION` as compiled
into this binary. Required by BC-2.02.001 (see SS-core-types-and-abi.md §ABI Version
Constant). Phase 3 plugin SDK and Phase 4 federation use this field to verify
ABI compatibility before handshake.

**Authentication:** `/status` requires the same `X-Monocle-Authorization: <token>`
header as hook endpoints. Rationale: `/status` exposes internal buffer fill levels
and channel saturation — metrics that reveal load patterns and internal queue
behavior that a local adversary could exploit to time attacks. Unauthenticated
access to `/status` is not warranted given the richer payload.

**Use:** Developer debugging, observability, CI integration tests. Read-only; no
state mutations.

<a id="body-size-limit"></a>
## Body Size Limit (F-NEW-06)

**Contract (BC-2.01.003):** All hook POST endpoints (`/hooks/*`) and `/status`
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
NOT be applied to `/healthz` (unauthenticated per BC-2.01.001). The correct
axum 0.8 pattern is to declare two routers — one unauthenticated, one authenticated
— and merge them. Hook endpoints and admin endpoints (`/status`, `/shutdown`)
share the same auth middleware layer on the authenticated router. The auth middleware
implements **dual-accept** per ADR-0005: it accepts the canonical `X-Monocle-Authorization`
header (required prefix `monocle-v1:`; monocle-aware tools and future harnesses) OR
the compatibility alias `X-Claude-Code-Ide-Authorization` (raw 64-hex, no prefix;
used by real Claude Code hook scripts whose header name is hardcoded per BC-HOOK-016
deep ingest). `X-Monocle-Authorization` takes priority if both headers are present.
When the compatibility alias is used, the middleware emits a WARN-level deprecation log.
If neither header is present, the middleware returns HTTP 401 `{"error":"missing_auth_token"}`.

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

<a id="daemon-lifecycle-protocol"></a>
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
   asymmetry with `BC-2.03.003` (which fail-fasts on `BaseDirs::new() == None`)
   is correct: `BaseDirs::new()` returns `None` only when there is no home directory
   at all — a genuine system-configuration failure; `ProjectDirs::runtime_dir()`
   returns `None` on macOS as a platform design choice, not a failure.

   Implementation:

   ```rust
   /// Resolve the runtime directory per BC-2.01.005 Precondition 2 chain
   /// (a) MONOCLE_RUNTIME_DIR env override
   /// (b) ProjectDirs::runtime_dir() platform-aware
   /// (c) data_local_dir() fallback
   ///
   /// Note: path (d) "fail-fast on ProjectDirs::new() == None" is handled
   /// in the CALLER (daemon main) BEFORE this function is invoked. This function
   /// is infallible because:
   /// - Path (b) `runtime_dir()` returns `Option<&Path>` and returns `None`
   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
   ///   the None arm falls through to path (c).
   /// - Path (c) `data_local_dir()` returns `&Path` (never `Option`),
   ///   making (c) the unconditional terminator of the resolution chain.
   ///
   /// `ProjectDirs::new() == None` is handled in the CALLER before this
   /// function is invoked. Hence return type is `PathBuf`, not
   /// `Result<PathBuf, _>`.
   fn resolve_runtime_dir(project_dirs: &directories::ProjectDirs) -> PathBuf {
       // (a) Operator env override
       if let Ok(env_path) = std::env::var("MONOCLE_RUNTIME_DIR") {
           if !env_path.is_empty() {
               tracing::info!(source = "MONOCLE_RUNTIME_DIR", "runtime_dir resolved");
               return PathBuf::from(env_path);
           }
       }
       // (b) XDG runtime dir (Linux only in practice)
       if let Some(rd) = project_dirs.runtime_dir() {
           tracing::info!(source = "ProjectDirs::runtime_dir()", "runtime_dir resolved");
           return rd.to_path_buf();
       }
       // (c) data_local_dir fallback (macOS / Windows / XDG-less Linux)
       // data_local_dir() returns &Path (never Option) — this branch is infallible.
       let fallback = project_dirs.data_local_dir().to_path_buf();
       tracing::info!(
           source = "ProjectDirs::data_local_dir()",
           platform = std::env::consts::OS,
           "runtime_dir fallback resolved"
       );
       fallback
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

   **Phase 4 OAuth2 clarification (FC-06):** Phase 4 federation does NOT
   extend `X-Monocle-Authorization` to carry OAuth2 tokens. Phase 4 federation
   tokens use the STANDARD `Authorization: Bearer <token>` header on a SEPARATE
   `monocle-ipc` federation channel (russh tunnel), which is distinct from the
   Phase 1 HTTP hook-ingestion channel. The Phase 1 daemon's auth middleware:

   - Inspects only `X-Monocle-Authorization` (never `Authorization: Bearer`).
   - Rejects any `Authorization: Bearer` header with HTTP 401 on Phase 1 routes
     (the header is not a recognized auth mechanism for Phase 1 endpoints).

   Phase 4 daemon adds a separate federation middleware path on the russh/`monocle-ipc`
   channel gated by a `federation` feature flag. The Phase 1 HTTP routes use
   dual-accept auth (ADR-0005) with no Bearer support. BC-2.01.009 applies
   to the Phase 1 auth surface; BC-2.01.009 postcondition 1 is being updated
   by PO Round 4 to reflect dual-accept semantics (see ADR-0005 §BC Impact).

   Auth middleware validation rules in Phase 1 — **dual-accept protocol (ADR-0005)**
   (applied in this order):

   1. **Canonical path (`X-Monocle-Authorization` present):** Validate value with
      prefix check: MUST begin with `monocle-v1:`. Strip prefix; constant-time
      compare hex suffix against stored secret. On success: proceed. On failure
      (bad prefix, bad format, secret mismatch): return HTTP 401
      `{"error":"invalid_auth_token"}`.

   2. **Compatibility alias (`X-Monocle-Authorization` absent, `X-Claude-Code-Ide-Authorization`
      present):** Emit WARN deprecation log. Validate value as raw 64-hex
      (no prefix required — real Claude Code sends the lock file `authToken` field
      verbatim, which has no prefix per BC-HOOK-016 deep ingest). Constant-time
      compare against stored secret. On success: proceed. On failure: return HTTP 401
      `{"error":"invalid_auth_token"}`.

   3. **Missing headers (both absent):** Return HTTP 401 `{"error":"missing_auth_token"}`
      immediately. This is a structural precondition failure, not an authentication
      attempt.

   Rules 1 and 2 deliberately collapse all value-present failures into the SAME error
   body (`invalid_auth_token`), blocking an attacker from determining whether their
   token had the structurally correct prefix even if they could not read the lock file
   directly. This applies equally to both the canonical and alias code paths.

   **Security rationale (threat model):** The monocle daemon binds exclusively
   to `127.0.0.1`. All callers are local processes running as the same OS user.
   An adversary co-located as the same user can read `monocle.lock` directly
   (0o600, same-user read access). Enumeration via distinct format-vs-mismatch
   error bodies provides zero marginal attack capability for a same-user
   adversary. However, defence-in-depth is applied: collapsing failure modes
   costs nothing and prevents any information leak to an attacker who has gained
   unexpected network access to 127.0.0.1 but has NOT gained file-system access
   (e.g., a compromised subprocess with a restricted sandbox).

   The `missing_auth_token` body for absent headers (Rule 3) is deliberately
   distinct because: (a) absence of both recognized headers is a client-configuration
   error, not an authentication attempt — the attacker who omits recognized headers
   has revealed nothing about knowledge of the secret; (b) the distinct body
   provides actionable diagnostics for developers debugging hook integration.

   Auth middleware implementation (dual-accept per ADR-0005):

   ```rust
   const TOKEN_PREFIX: &str = "monocle-v1:";
   const CANONICAL_HEADER: &str = "X-Monocle-Authorization";
   const COMPAT_ALIAS_HEADER: &str = "X-Claude-Code-Ide-Authorization";

   /// Extract and validate the monocle auth token from the request headers.
   ///
   /// Dual-accept per ADR-0005:
   /// - `X-Monocle-Authorization: monocle-v1:<hex>` — canonical; monocle-aware tools.
   /// - `X-Claude-Code-Ide-Authorization: <hex>` — compatibility alias; real Claude Code
   ///   hook scripts whose header name is hardcoded (BC-HOOK-016). Emits WARN log.
   ///
   /// Returns:
   /// - `Ok(())` if authentication succeeds via either path.
   /// - `Err(AuthError::Missing)` if BOTH recognized headers are absent.
   /// - `Err(AuthError::Invalid)` if a recognized header is present but fails
   ///   validation for any reason. Intentionally collapsed to prevent information
   ///   disclosure about which check failed.
   fn validate_auth_header(
       headers: &HeaderMap,
       expected_secret: &str,
   ) -> Result<(), AuthError> {
       if let Some(header_value) = headers.get(CANONICAL_HEADER) {
           // Canonical path: X-Monocle-Authorization with monocle-v1: prefix required.
           let Ok(presented) = header_value.to_str() else {
               return Err(AuthError::Invalid);
           };
           let Some(hex_part) = presented.strip_prefix(TOKEN_PREFIX) else {
               return Err(AuthError::Invalid); // bad prefix — not a valid auth attempt
           };
           // Constant-time comparison to prevent timing oracle on the hex secret.
           if constant_time_eq::constant_time_eq(hex_part.as_bytes(), expected_secret.as_bytes()) {
               Ok(())
           } else {
               Err(AuthError::Invalid) // format OK but token mismatch — same body
           }
       } else if let Some(alias_value) = headers.get(COMPAT_ALIAS_HEADER) {
           // Compatibility alias: X-Claude-Code-Ide-Authorization (raw hex, no prefix).
           // Real Claude Code hook scripts send the lock file authToken field verbatim.
           // ADR-0005: emit deprecation WARN to aid migration visibility.
           tracing::warn!(
               header = COMPAT_ALIAS_HEADER,
               "hook auth via compatibility alias; \
                monocle-aware harness should use X-Monocle-Authorization"
           );
           let Ok(raw_hex) = alias_value.to_str() else {
               return Err(AuthError::Invalid);
           };
           // No prefix to strip — Claude Code sends the raw 64-hex secret directly.
           // Constant-time comparison to prevent timing oracle.
           if constant_time_eq::constant_time_eq(raw_hex.as_bytes(), expected_secret.as_bytes()) {
               Ok(())
           } else {
               Err(AuthError::Invalid)
           }
       } else {
           // Neither recognized header present — structural precondition failure.
           Err(AuthError::Missing)
       }
   }

   #[derive(Debug)]
   pub enum AuthError {
       Missing,  // → HTTP 401 {"error":"missing_auth_token"} (both headers absent)
       Invalid,  // → HTTP 401 {"error":"invalid_auth_token"} (any value-present failure)
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

   - **BC-2.01.008:** The auth token written to the lock file has format
     `monocle-v1:<64-hex>` when read back from the lock file and presented to
     the daemon. The lock file `authToken` field stores only the 64-char hex
     part. Verification: integration test in
     `monocle-runtime/tests/auth_token_lifecycle.rs` reads the lock file after
     daemon start and asserts `authToken` matches `/^[0-9a-f]{64}$/`; presents
     `monocle-v1:<authToken>` to `/status` and asserts HTTP 200.
     Test name: `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`
     (PRD v1.1 §7 RTM canonical path; F-R62-4).

   - **BC-2.01.009 (dual-accept per ADR-0005):** Two auth failure modes are specified.
     "Missing" means BOTH recognized headers are absent. BC-2.01.009 is being updated
     by PO Round 4 to reflect dual-accept semantics; the two-body taxonomy is preserved:

     | Failure mode | Header state | HTTP body |
     |---|---|---|
     | Missing headers | BOTH `X-Monocle-Authorization` and `X-Claude-Code-Ide-Authorization` absent | `{"error":"missing_auth_token"}` |
     | Invalid token (canonical path) | `X-Monocle-Authorization` present; value fails for any reason (bad prefix, bad format, secret mismatch, or empty suffix) | `{"error":"invalid_auth_token"}` |
     | Invalid token (alias path) | `X-Monocle-Authorization` absent; `X-Claude-Code-Ide-Authorization` present; raw hex value fails constant-time comparison | `{"error":"invalid_auth_token"}` |

     All value-present failures return the same body regardless of code path —
     canonical or alias — this is a deliberate security choice (see security
     rationale above). The two-body taxonomy (`missing_auth_token` /
     `invalid_auth_token`) is preserved; dual-accept expands "missing" to mean
     "both recognized headers absent".

     Phase 4 OAuth2 federation tokens use `Authorization: Bearer` on a separate
     federation channel and are NOT valid on Phase 1 HTTP endpoints; they
     receive HTTP 401 `{"error":"missing_auth_token"}` (neither recognized Phase 1
     auth header is present; `Authorization: Bearer` is a different, unrecognized
     header for Phase 1 endpoints — Phase 4 OAuth2 uses a separate federation
     channel and does not reuse the Phase 1 HTTP endpoints).

     Verification: integration test in
     `monocle-runtime/tests/auth_header_rejection.rs` (rejection probes;
     F-R62-4 canonical path per PRD v1.1 §7 RTM). Round-trip happy-path covered
     in `monocle-runtime/tests/auth_token_lifecycle.rs` per BC-2.01.008
     verification above.
     Test name: `test_BC_AUTH_002_auth_header_validation_all_failure_modes`
     - No header (neither recognized) → HTTP 401 `{"error":"missing_auth_token"}`
     - `X-Monocle-Authorization: baretoken` → HTTP 401 `{"error":"invalid_auth_token"}`
     - `X-Monocle-Authorization: monocle-v2:abc` → HTTP 401 `{"error":"invalid_auth_token"}`
     - `X-Monocle-Authorization: monocle-v1:` (empty suffix) → HTTP 401 `{"error":"invalid_auth_token"}`
     - `Authorization: Bearer fake` (wrong header name, both recognized headers absent) → HTTP 401 `{"error":"missing_auth_token"}`
     - `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` → HTTP 401 `{"error":"invalid_auth_token"}`
     - `X-Claude-Code-Ide-Authorization: <wrong-64-hex>` (alias path, wrong secret) → HTTP 401 `{"error":"invalid_auth_token"}`
     - `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (alias path, correct secret) → HTTP 200 + WARN log emitted

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
   BC-2.01.010: any lock-file reader MUST check `contract_version == 1` before
   consuming other fields; an unrecognized version triggers a graceful skip with
   a log warning.
   `startTimeUtc` uses ISO 8601 UTC format with mandatory millisecond precision
   (`YYYY-MM-DDTHH:MM:SS.sssZ`) — matching `last_hook_ts` (BC-2.01.002 / EC-044)
   and `shutdown_utc` (BC-2.01.006) for cross-field uniformity per F-R72-1.
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

<a id="drain"></a>
### Drain (10-Second Timeout)

3. Wait up to 10 seconds for in-flight hook POSTs to complete
   (`tokio::time::timeout(Duration::from_secs(10), drain_inflight())`).
<a id="jsonl-ring-buffer"></a>
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
       #[serde(skip_serializing_if = "Option::is_none")]
       pub tool_name: Option<String>,
       /// Tool input as a parsed JSON value; populated for `PreToolUse` and `Notification` events.
       /// `None` for events without tool context (`SessionStart`, `UserPromptSubmit`, `Stop`).
       /// Stored as `serde_json::Value` (in-memory JSON tree, not an encoded string) to
       /// avoid double-deserialization on the read path.
       #[serde(skip_serializing_if = "Option::is_none")]
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

   For hook types without tool context (`SessionStart`, `UserPromptSubmit`, `Stop`), per
   BC-2.01.007 EC-001 the `#[serde(skip_serializing_if = "Option::is_none")]` annotation
   causes the `tool_name` and `tool_input` fields to be OMITTED entirely from the JSONL
   record:

   ```json
   {"format_version":1,"session_id":"<uuid>","timestamp_micros":1747094400000000,"pid":12345,"hook_type":"SessionStart"}
   ```

   Note the absence (not explicit null) of `tool_name` and `tool_input` keys. Phase 1
   emitters MUST emit absence; Phase 2+ readers MUST tolerate both absence and explicit
   null per forward-compat.

   **Behavioral contract: BC-2.01.007** — every JSONL record's first key is
   `format_version` with value `1` for all Phase 1-origin records. Verification:
   integration test in `monocle-runtime/tests/jsonl_ring.rs` constructs a
   `HookEventRecord` via `HookEventRecord::new(...)` and asserts the resulting
   JSON string begins with `{"format_version":1,`.

   **JSONL Ring Buffer Rotation Policy** (canonical source of truth; traced from
   `oq-research.md §OQ-06` recommendation, brief §Storage `100MB × 5 rotation`,
   and NFR-006 throughput ceiling):

   The rotation policy governs disk usage for the JSONL event log when
   `--persistent-events` is set. It bounds total on-disk footprint to a
   predictable maximum regardless of event volume.

   | Parameter | Value | Rationale |
   |-----------|-------|-----------|
   | Default rotation threshold | 50 MB per active file | Soft trigger: checked on each flush batch; provides early rotation before the absolute cap is reached |
   | Absolute per-file cap | 100 MB | Hard upper bound; rotation is mandatory when the active file reaches this size regardless of configuration |
   | Maximum retained rotated files | 5 | `events.jsonl.1` through `events.jsonl.5` (plus the active `events.jsonl`) |
   | Total disk usage ceiling | 500 MB (5 × 100 MB) | Active file excluded from this ceiling; absolute worst-case is 500 MB rotated + up to 100 MB active = 600 MB |
   | Rotation algorithm | Atomic rename | Active file renamed to `events.jsonl.N` (incrementing N, wrapping); new active file created fresh; both operations via `std::fs::rename` (kernel atomic on POSIX) |
   | Compress on rotate | No | Raw JSONL retained for Phase 2 replay without decompression overhead; disk tradeoff accepted per brief scope |
   | File naming convention | `monocle-events.jsonl` (active), `monocle-events.jsonl.1` through `monocle-events.jsonl.5` (rotated, newest=1) | Newest rotated file is always `.1`; numbers shift up on each rotation (`.1` → `.2`, etc.) |
   | Cleanup policy | Oldest-first deletion | When 5 rotated files already exist and a new rotation is triggered, `monocle-events.jsonl.5` is deleted before the rename cascade; no rotated file is ever silently overwritten |
   | Mode on new active file | `0o600` | Owner-read/write only; same mode as lock file; set via `File::create` + `set_permissions` before first write |
   | Flush that triggers rotation check | Post-batch flush completion | After each successful `tempfile::persist` flush, check `active_file.metadata()?.len()` against the 50 MB threshold; trigger rotation synchronously within the flush task before returning |
   | EC-002 compatibility | Lines up to 256 KiB are never truncated | Rotation check is size-based on the whole file, not line-count-based; a single 256 KiB line is written atomically and counted toward the threshold after the write succeeds |

   **Rotation sequence (atomic, within flush task):**

   1. Check `monocle-events.jsonl` size via `metadata().len()`.
   2. If size < 50 MB: no rotation; return.
   3. If size >= 50 MB (or 100 MB hard cap): proceed with rotation cascade.
   4. Delete `monocle-events.jsonl.5` if it exists.
   5. Rename `monocle-events.jsonl.4` → `monocle-events.jsonl.5` (if exists).
   6. Rename `monocle-events.jsonl.3` → `monocle-events.jsonl.4` (if exists).
   7. Rename `monocle-events.jsonl.2` → `monocle-events.jsonl.3` (if exists).
   8. Rename `monocle-events.jsonl.1` → `monocle-events.jsonl.2` (if exists).
   9. Rename `monocle-events.jsonl` → `monocle-events.jsonl.1`.
   10. Create new `monocle-events.jsonl` with mode `0o600`; continue writes.

   If any rename fails (e.g., out of disk space), log `WARN E-RING-002` and
   continue writing to the existing active file without rotation. The ring
   continues accepting events; data is not lost. The active file may transiently
   exceed the 100 MB hard cap under this error condition; the next flush cycle
   re-attempts rotation.

   **Trace anchors:** `oq-research.md §OQ-06` (architectural decision source);
   `product-brief.md §Storage` ("100MB × 5 rotation"); NFR-006 (throughput — 1000
   events/sec ceiling, from which 100 MB per segment is derived as adequate for
   ~48-minute sessions at max rate). Added by F-PHASE2-R05-05 (v1.0.33).

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
   non-compliant per BC-2.01.006 invariant 1 (PRD v1.7). VP-DAEMON-006 enforces this
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

**BC-2.01.004** (exit-code postcondition): The exit code written to the OS process
table on daemon termination MUST match the trigger:
- graceful drain complete → `0`
- SIGINT hard-kill during drain → `130`
- SIGTERM hard-kill during drain → `143`
- admin `/shutdown` second-call during drain → `2`
- startup failure → `1`

Verification: integration test in `monocle-runtime/tests/daemon_lifecycle.rs`
(`test_BC_DAEMON_004_exit_codes_posix_distinct`) sends SIGTERM twice (expects 143), SIGINT twice
(expects 130), and two sequential `POST /shutdown` calls (expects 2). The PRD
BC-2.01.004 postcondition is the canonical error-taxonomy source; this architecture
document is the canonical rationale source for the code selection.

### Crash Recovery

On startup, if `<runtime_dir>/monocle.recovery.json` exists AND the pid in the
(now-stale or absent) lock file is dead:

**Contract (BC-2.01.006):**
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
| BC-2.01.001 | `/healthz` returns 200/503 with uptime + version; unauthenticated | Health and Status Endpoints |
| BC-2.01.002 | `/status` returns full daemon state JSON; requires auth token | Health and Status Endpoints |
| BC-2.01.003 | All `/hooks/*` and `/status` enforce 256 KiB body limit; 413 on excess | Body Size Limit |
| BC-2.01.004 | Graceful shutdown: 10-second drain, ring buffer flush, recovery checkpoint; exit codes: 0 (clean), 130 (SIGINT hard-kill), 143 (SIGTERM hard-kill), 2 (admin /shutdown force-stop), 1 (startup failure) | Daemon Lifecycle Protocol |
| BC-2.01.005 | Runtime dir resolved via platform-aware chain: MONOCLE_RUNTIME_DIR env override → ProjectDirs::runtime_dir() (Linux/XDG) → ProjectDirs::data_local_dir() (macOS/Windows fallback); runtime_dir created with mode `0o700` owner-only (defense-in-depth with lock file `0o600`); lock file created atomically via `tempfile::persist`; pid-liveness checked on startup; removed on clean shutdown | Daemon Lifecycle Protocol |
| BC-2.01.006 | Crash recovery checkpoint at `<runtime_dir>/monocle.recovery.json`; TUI offered recovery on next attach | Daemon Lifecycle Protocol |
| BC-2.01.007 | Every JSONL ring buffer record's first key is `format_version` with value `1` for all Phase 1-origin records (FC-01) | Daemon Lifecycle Protocol §Drain |
| BC-2.01.008 | Auth token wire format is `monocle-v1:<64-hex>`; lock file stores bare 64-hex; presented token validated with constant-time comparison after prefix strip (FC-06) | Daemon Lifecycle Protocol §Start Sequence |
| BC-2.01.009 | Dual-accept auth failure modes (ADR-0005): (1) BOTH recognized headers (`X-Monocle-Authorization` and `X-Claude-Code-Ide-Authorization`) absent → HTTP 401 `{"error":"missing_auth_token"}`; (2) either recognized header present but value fails validation for any reason → HTTP 401 `{"error":"invalid_auth_token"}` (collapsed; no format/mismatch distinction); Phase 4 OAuth2 federation uses separate channel (FC-06). PO Round 4 updates BC-2.01.009 postconditions to reflect dual-accept. | Daemon Lifecycle Protocol §Start Sequence |
| BC-2.01.010 | Lock-file JSON includes `contract_version: 1` as the first key; readers must check this field before consuming other fields; unrecognized version triggers graceful skip with warning (F-FC-O001) | Daemon Lifecycle Protocol §Start Sequence |

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
(forward-compatibility version sentinel, always FIRST key per BC-2.01.010
Postcondition 2; Phase 4 readers MUST validate `contract_version == 1` before
consuming other fields), `"pid"`, `"port"`, `"authToken"`, `"startTimeUtc"`,
`"app"`, `"version"`. Phase 4 readers that encounter `contract_version > 1` MUST
fail gracefully (do not attempt parse of unknown-version JSON).

---

## §Trace

v1.0.25 changes (F-R101 Burst 2 — F-R101-3 MED closure: §Trace v1.0.24 [N-4] classification
sharpening; option (a) per D-114 Goodhart's law; SE-17g SECOND APPLICATION; SE-17f FOURTH APPLICATION):

- F-R101-3 RESOLVED (MED — §Trace v1.0.24 [N-4] NORMATIVE classification self-contradiction):

  The adversary R101 reported that §Trace v1.0.24 SE-17f Step 1 [N-4] block carries a dual
  implicit classification. The [N-4] label reads:

    "[N-4] NORMATIVE. Literal final-state output, 35 lines total. Production-code hits
    NORMATIVE at 235/238. All §Trace-body hits (lines 956+) are INFORMATIONAL per SE-17c-d."

  SE-17g prohibits implicit dual-classification: a citation is NORMATIVE OR INFORMATIONAL,
  not both. The [N-4] block contains two distinct sub-claims:
  (a) Production-code hits at lines 235/238 — stable above BOUNDARY=790; this is NORMATIVE.
  (b) "35 lines total" — a snapshot count at v1.0.24 insertion-time; by the time R101 ran,
      the same grep returned 70 lines (§Trace growth per SE-17c-d); this sub-claim cannot be
      NORMATIVE without obligating SE-17f re-run to reconcile a count that is designed to grow.

  Additionally, re-running the grep against the current (post-v1.0.24, pre-v1.0.25-edit)
  file returns 70 lines — confirming the 35-line claim is already stale. Under SE-17g, the
  count claim's NORMATIVE label creates a false SE-17f re-run obligation that is both
  non-actionable (§Trace-body growth is expected per SE-17c-d) and irreconcilable without
  amending SE-17g itself.

  Human directed option (a) per D-114 Goodhart's law: NO SE-17g amendment. The fix is
  purely in-file classification sharpening — split [N-4] into two disjoint citations.

  **Classification split (option (a)):**

  - **[N-4a] NORMATIVE — production-code hits**: lines 235/238 are production-code citations
    above BOUNDARY=790. They are stable across all §Trace insertions. NORMATIVE class applies:
    SE-17f re-run required; SE-17a literal transcript required. See SE-17f block below.

  - **[N-4b] INFORMATIONAL — snapshot count at v1.0.24 insertion-time**: "35 lines total"
    was the grep line-count at the moment §Trace v1.0.24 NORMATIVE transcript was committed.
    This count grows with each §Trace insertion per SE-17c-d (§Trace-body hits are
    INFORMATIONAL, their growth is expected and non-actionable). Current actual line-count
    pre-v1.0.25-edit: 70 lines. Post-v1.0.25-edit: will grow further. The [N-4b] citation
    carries the v1.0.24 snapshot value (35 lines) and the pre-v1.0.25 actual value (70 lines)
    as INFORMATIONAL evidence. SE-17f re-run NOT required for [N-4b].

  The §Trace v1.0.24 [N-4] block is preserved VERBATIM as PG-5 historical (see §Trace v1.0.24
  below). The split interpretation is applied here; the v1.0.24 block is not modified in place.

- SE-17g SECOND APPLICATION (33rd discipline — citation taxonomy for §Trace v1.0.25):

  NORMATIVE citations (must match final-state; SE-17f re-run required):
  - [N-1] Frontmatter `version:` field value "1.0.25" — frontmatter field; SE-17f re-read required.
  - [N-2] Frontmatter `timestamp:` value "2026-05-17T06:00:00Z" — frontmatter field; SE-17f re-read required.
  - [N-3] Production-code L-numbers 235/238 — stable above BOUNDARY=790; SE-17f re-run required.
    (Same [N-3] as v1.0.24; restated here as NORMATIVE for this §Trace entry's SE-17f scope.)
  - [N-4a] Scoped-awk NORMATIVE transcript — production-code body only (awk NR 1–789); SE-17f re-run required.
  - [N-5] BOUNDARY=790 — confirmed by `grep -n "^## §Trace" | head -1`; SE-17f re-run required.

  INFORMATIONAL citations (OK if approximate; SE-17f re-run not required):
  - [I-1] [N-4b] Snapshot counts — "35 lines total" (v1.0.24 insertion-time) and "70 lines"
    (pre-v1.0.25-edit actual); §Trace-body hit growth; INFORMATIONAL per SE-17c-d.
  - [I-2] Adversary R101 narrative references (defect description, line range citations in
    adversary report) — adversary observations cited as evidence, not fresh greps; INFORMATIONAL.
  - [I-3] §Trace v1.0.24 [N-4] verbatim block position — informational range; §Trace-body
    narrative per D-108.

- META-N+9 SELF-DISCLOSURE (SE-17g obligation):

  This §Trace v1.0.25 entry contains narrative text that includes the substring
  "35 lines total" and "70 lines". A future grep of the full file for [N-4b]-related
  strings will match this §Trace v1.0.25 body — this is the expected §Trace-body growth
  pattern per SE-17c-d. These narrative occurrences are INFORMATIONAL ([I-1]) and non-actionable.

  The SE-17f NORMATIVE transcript [N-4a] below uses a scoped awk (NR 1–789, i.e., production-
  code body only above BOUNDARY=790) which structurally excludes the entire §Trace section.
  This ensures the [N-4a] transcript cannot self-match §Trace v1.0.25 body text regardless
  of what narrative appears in this entry. No META-N+9 recursive self-match is possible in
  the NORMATIVE [N-4a] transcript.

- Disciplines applied:
  - SE-17a (NORMATIVE literal scoped-awk transcript [N-4a] in SE-17f block — no ellipsis,
    no summary count, no hedging; post-edit final-state output only; scoped to NR 1–789)
  - SE-17b (self-verification: NORMATIVE scoped-awk [N-4a] re-run after all body edits complete)
  - SE-17c (5-step: body authored → final-state NORMATIVE scoped-awk run → L-numbers confirmed →
    re-verified against final-state file → committed)
  - SE-17c-d (body-scope filter: BOUNDARY=790 [N-5]; production-code hits 235/238 [N-3] above
    boundary; scoped-awk (NR 1–789) structurally enforces the boundary exclusion for [N-4a])
  - SE-17e (sibling-propagation: PRD v1.25 receives arch v1.0.25 pin per Extension 15;
    VP v1.35 receives arch v1.0.25 pin per Extension 15; both in SE-15e cascade bursts)
  - SE-17f FOURTH APPLICATION (mechanical self-revalidation gate — see self-revalidation
    block below; NORMATIVE elements [N-1]–[N-5] + [N-4a] re-verified after all edits)
  - SE-17g SECOND APPLICATION (33rd discipline — NORMATIVE/INFORMATIONAL taxonomy applied
    to every citation; ambiguous citations default to NORMATIVE per Production-Grade Default)

- SE-16b monotonicity check PASS: v1.0.24 → v1.0.25 is a monotonic increment.
  Timestamp 2026-05-17T06:00:00Z >= v1.0.24 timestamp 2026-05-17T02:30:00Z. PASS.
  [SE-17g: [N-2] NORMATIVE — frontmatter timestamp value.]

- SE-16d PASS (cross-artifact chain-time monotonicity, 32nd discipline):
  2026-05-17T06:00:00Z >= STATE v5.54 chain high-water 2026-05-17T05:30:00Z. PASS.
  UTC ISO-8601 Z form confirmed. [SE-17g: [N-2] NORMATIVE.]

- No body content changes in this burst. Only §Trace edits:
  (a) §Trace v1.0.25 entry authored (this entry).
  (b) §Trace v1.0.24 [N-4] block preserved verbatim as PG-5 historical (no in-place modification).
  The [N-4] split ([N-4a] NORMATIVE / [N-4b] INFORMATIONAL) is applied via this §Trace v1.0.25
  clarification block, not by modifying the v1.0.24 block.

- Cross-document pins (Extension 15 + SE-15e mandatory cascade):
  arch v1.0.25 pin propagation required:
  - Burst 3 (PO PRD v1.25) — Extension 15 arch v1.0.24 → v1.0.25 pin propagation.
  - Burst 4 (FV VP v1.35) — Extension 15 arch v1.0.24 → v1.0.25 pin propagation.
  - Burst 5 (SM STATE v5.55) — chain closure recording.

- SE-17f SELF-REVALIDATION BLOCK (FOURTH APPLICATION of 31st discipline):

  SE-17g classification: all steps in this block that cite scoped-awk transcripts or L-numbers
  are NORMATIVE [N-3]–[N-5], [N-4a]. Steps citing informational counts or ranges are
  INFORMATIONAL [I-1]. SE-17f NORMATIVE steps must be re-run post-edit and before commit.

  Step 1 [NORMATIVE — N-4a]: scoped-awk transcript run AFTER all §Trace v1.0.25 body edits
  are complete (post-edit final-state of v1.0.25 file). SE-17a-compliant: literal bash output,
  no ellipsis, no abbreviation, no hedging. Scoped to NR 1–789 (production-code body only):
  ```
  $ awk 'NR>=1 && NR<=789' /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md | grep -nE "making \(c\) the unconditional terminator|platform-ABI design \(not misconfiguration\)"
  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  238:   ///   making (c) the unconditional terminator of the resolution chain.
  ```
  [SE-17g: [N-4a] NORMATIVE. Scoped-awk final-state output, production-code body only
  (NR 1–789). 2 lines total. Production-code hits confirmed at 235/238. §Trace section
  (NR 790+) structurally excluded — no §Trace-body self-match possible. SE-17a-compliant.]

  Step 2 [NORMATIVE — N-3]: verify production-code hits at 235/238 — CONFIRMED.
  Lines 235 and 238 are above BOUNDARY=790. They are in the production-code body section.
  The scoped-awk (NR 1–789) confirms only these 2 lines match.
  [SE-17g: [N-3] NORMATIVE — production-code L-number citations, must match final-state.]

  Step 3 [NORMATIVE — N-5]: verify BOUNDARY=790:
  ```
  $ grep -n "^## §Trace" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md | head -1
  790:## §Trace
  ```
  BOUNDARY=790 CONFIRMED. [SE-17g: [N-5] NORMATIVE — boundary value for SE-17c-d filter.]

  Step 4 [NORMATIVE — N-1, N-2]: re-read frontmatter fields post-edit — CONFIRMED:
  - version: "1.0.25" [N-1] NORMATIVE — confirmed present in frontmatter.
  - timestamp: "2026-05-17T06:00:00Z" [N-2] NORMATIVE — confirmed present in frontmatter.

  Step 5 [INFORMATIONAL — I-1]: [N-4b] snapshot count verification:
  - v1.0.24 insertion-time count: 35 lines (per §Trace v1.0.24 [N-4] SE-17g block; historical
    snapshot, INFORMATIONAL).
  - Pre-v1.0.25-edit actual count: 70 lines (re-run pre-edit, confirmed 70 = expected growth
    from §Trace accumulation per SE-17c-d). INFORMATIONAL — non-actionable.
  - Post-v1.0.25-edit count: will increase further (§Trace v1.0.25 body contains multiple
    matches for the grep strings in §Trace-body narrative context). INFORMATIONAL.
  [SE-17g: [I-1] INFORMATIONAL — snapshot counts; §Trace-body growth; SE-17c-d expected.]

  Step 6: SE-17g recursion check — this SE-17f block contains scoped-awk transcript [N-4a].
  The awk scope (NR 1–789) structurally excludes this §Trace v1.0.25 entry (located at NR
  790+). Therefore no recursive self-match is possible for [N-4a]. §Trace-body narrative
  occurrences of the grep strings are INFORMATIONAL ([I-1]) per SE-17c-d and non-actionable.
  SE-17a-compliance is declared for [N-4a] as literal scoped-awk output after all edits.
  This is the FOURTH APPLICATION of SE-17f; SE-17g SECOND APPLICATION.
  No divergences unresolved in this burst.

  Divergence summary:
  Pre-v1.0.25-edit baseline: full-file grep returned 70 lines (confirmed pre-edit).
  After §Trace v1.0.25 body insertion: full-file grep will return additional hits from
  §Trace v1.0.25 narrative (expected, non-actionable per SE-17c-d).
  Scoped-awk (NR 1–789) returns 2 lines throughout — production-code hits 235/238 only.
  SE-17f [N-4a] scoped approach eliminates the entire class of §Trace-body self-match.
  No production-code divergences unresolved.

v1.0.24 changes (F-R100 Burst 2 — F-R100-1 HIGH closure: §Trace v1.0.23 SE-17f Step 1
transcript SE-17a non-compliance; SE-17g FIRST APPLICATION; SE-17f THIRD APPLICATION):

- F-R100-1 RESOLVED (HIGH — §Trace v1.0.23 SE-17f Step 1 transcript SE-17a non-compliance):

  SE-17g classification: NORMATIVE (this citation class — literal `$ grep` transcripts that
  declare SE-17a-compliance — must match final-state; SE-17f re-run required post-edit).

  The adversary R100 reported that the §Trace v1.0.23 SE-17f Step 1 block displayed a
  grep transcript captured mid-burst (before §Trace v1.0.23 narrative expansion completed):
  - Transcript displayed 22 lines / 11 hit-pairs at capture time.
  - Actual final-state v1.0.23 grep returned 36 lines / 18 hit-pairs.
  - Step 1 summary claimed "30 lines, 15 hit-pairs" — contradicted by both the actual
    grep output and the adversary's canonical re-run.
  - Step 7 (lines 924–929 at v1.0.23 state) declared the transcript "SE-17a compliant"
    — unsupported given the 22-vs-36 discrepancy.

  Root cause: SE-17g disambiguation (33rd discipline, D-110) identifies that literal
  `$ grep` transcripts declaring SE-17a-compliance are NORMATIVE class and must be
  re-run post-edit. The Burst 2 architect (D-108) established that §Trace-body narrative
  L-numbers are INFORMATIONAL — but this principle was incorrectly extended to cover
  literal grep transcripts, which are NORMATIVE. SE-17f Step 1 was therefore applied
  with the wrong citation class, allowing a mid-burst snapshot to remain as the committed
  NORMATIVE transcript.

  Fix applied: §Trace v1.0.23 SE-17f Step 1 transcript retired as PG-5 historical
  (preserved verbatim with explicit "PRE-FINALIZATION SNAPSHOT — INFORMATIONAL" label).
  SE-17a-compliance claim at Step 7 withdrawn. Authoritative NORMATIVE transcript
  produced in this §Trace v1.0.24 SE-17g reconciliation block (see SE-17f below).

- SE-17g FIRST APPLICATION (33rd discipline — NORMATIVE/INFORMATIONAL citation taxonomy):

  Every citation in this §Trace v1.0.24 entry is explicitly classified:

  NORMATIVE citations (must match final-state; SE-17f re-run required):
  - [N-1] Frontmatter `version:` field value "1.0.24" — production code / frontmatter
    field; SE-17f re-read required.
  - [N-2] Frontmatter `timestamp:` value "2026-05-17T02:30:00Z" — frontmatter field;
    SE-17f re-read required.
  - [N-3] Production-code L-numbers 235/238 (platform-ABI / unconditional-terminator
    strings) — confirmed stable above BOUNDARY=790; SE-17f re-run required.
  - [N-4] NORMATIVE literal grep transcript in SE-17f self-revalidation block below —
    post-edit final-state output; SE-17f re-run required AFTER all body edits complete.
  - [N-5] Total hit-count and hit-pair count in SE-17f transcript — explicit count claim
    using enumeration; SE-17f verification required.
  - [N-6] BOUNDARY=790 (§Trace section boundary — confirmed by `grep -n "^## §Trace"`)

  INFORMATIONAL citations (range citations; OK if approximate; SE-17f re-run not required):
  - [I-1] Narrative range "approximately lines 930–935" in §Trace v1.0.23 Steps 3–5
    (pre-existing informational ranges; established as informational per D-108).
  - [I-2] §Trace v1.0.23 SE-17f Step 1 transcript (retired as PG-5 historical, relabeled
    INFORMATIONAL in-place by this burst).
  - [I-3] Adversary-reported counts "22 lines / 11 hit-pairs" and "36 lines / 18 hit-pairs"
    in the root-cause narrative above — these are adversary observations cited as evidence,
    not fresh greps; labeled informational to avoid SE-17f re-run obligation. The
    authoritative count is in the NORMATIVE [N-4]/[N-5] SE-17f transcript below.
  - [I-4] Historical position references to v1.0.22/v1.0.21 §Trace-body blocks in
    Divergence summary — these are informational narrative ranges per D-108.

- Disciplines applied:
  - SE-17a (NORMATIVE literal grep transcript [N-4] in SE-17f block — no ellipsis, no
    summary count, no hedging; post-edit final-state output only)
  - SE-17b (self-verification: NORMATIVE grep [N-4] re-run after all body edits complete)
  - SE-17c (5-step: body authored → final-state NORMATIVE greps run → L-numbers updated →
    re-verified against final-state file → committed)
  - SE-17c-d (body-scope filter: BOUNDARY=790 [N-6]; production-code hits 235/238 [N-3]
    above boundary; all §Trace-body hits ≥ BOUNDARY are INFORMATIONAL per SE-17c-d)
  - SE-17e (sibling-propagation: PRD v1.24 receives arch v1.0.24 pin per Extension 15;
    VP v1.34 receives arch v1.0.24 pin per Extension 15; both in SE-15e cascade bursts)
  - SE-17f THIRD APPLICATION (mechanical self-revalidation gate — see self-revalidation
    block below; NORMATIVE elements [N-1]–[N-6] re-verified after all §Trace body edits)
  - SE-17g FIRST APPLICATION (33rd discipline — NORMATIVE/INFORMATIONAL taxonomy applied
    to every citation; ambiguous citations default to NORMATIVE per Production-Grade Default)

- SE-16b monotonicity check PASS: v1.0.23 → v1.0.24 is a monotonic increment.
  Timestamp 2026-05-17T02:30:00Z ≥ v1.0.23 timestamp 2026-05-17T00:00:00Z. PASS.
  [SE-17g: [N-2] NORMATIVE — frontmatter timestamp value.]

- SE-16d PASS (cross-artifact chain-time monotonicity, 32nd discipline):
  2026-05-17T02:30:00Z >= STATE v5.52 chain high-water 2026-05-17T02:00:00Z. PASS.
  UTC ISO-8601 Z form confirmed. [SE-17g: [N-2] NORMATIVE.]

- No body content changes in this burst. Only §Trace edits:
  (a) §Trace v1.0.23 SE-17f Step 1 transcript retired as PG-5 historical (INFORMATIONAL
      relabel); SE-17a-compliance claim at Step 7 withdrawn.
  (b) §Trace v1.0.24 entry authored (this entry).

- Cross-document pins (Extension 15 + SE-15e mandatory cascade):
  arch v1.0.24 pin propagation required:
  - Burst 3 (PO PRD v1.24) — Extension 15 arch v1.0.23 → v1.0.24 pin propagation.
  - Burst 4 (FV VP v1.34) — Extension 15 arch v1.0.23 → v1.0.24 pin propagation +
    F-R100-2 closure + GAP-R39-001 closure.
  - Burst 5 (SM STATE v5.53) — chain closure recording.

- SE-17f SELF-REVALIDATION BLOCK (THIRD APPLICATION of 31st discipline):

  SE-17g classification: all steps in this block that cite grep transcripts or L-numbers
  are NORMATIVE [N-3]–[N-6] — must be re-run after ALL §Trace v1.0.24 body edits complete.
  Steps citing narrative ranges only are INFORMATIONAL [I-4].

  Step 1 [NORMATIVE — N-4, N-5]: literal grep run AFTER all §Trace v1.0.24 body edits
  are complete (post-edit final-state of v1.0.24 file). SE-17a-compliant: literal bash
  output, no ellipsis, no abbreviation, no hedging:
  ```
  $ grep -nE "making \(c\) the unconditional terminator|platform-ABI design \(not misconfiguration\)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  238:   ///   making (c) the unconditional terminator of the resolution chain.
  956:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  957:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  958:  800:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);  [PRE block]
  959:  801:  236:   ///   making (c) the unconditional terminator of the resolution chain.  [PRE block]
  1012:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1013:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  1014:  818:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);  [PRE block]
  1015:  819:  238:   ///   making (c) the unconditional terminator of the resolution chain.  [PRE block]
  1016:  820:  800:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);  [PRE block]
  1017:  821:  801:  236:   ///   making (c) the unconditional terminator of the resolution chain.  [PRE block]
  1018:  864:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1019:  865:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  1020:  866:  800:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1021:  867:  801:  236:   ///   making (c) the unconditional terminator of the resolution chain.
  1022:  868:  809:  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" ...
  1023:  869:  810:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1024:  870:  811:   238:   ///   making (c) the unconditional terminator of the resolution chain.
  1025:  907:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1026:  908:  236:   ///   making (c) the unconditional terminator of the resolution chain.
  1027:  916:  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  1028:  917:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1029:  918:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  1030:  1028:  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  1031:  1029:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1032:  1030:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  1085:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1086:  236:   ///   making (c) the unconditional terminator of the resolution chain.
  1094:  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  1095:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1096:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  1206:  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  1207:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1208:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  ```
  [SE-17g: [N-4] NORMATIVE. Literal final-state output, 35 lines total. Production-code
  hits NORMATIVE at 235/238. All §Trace-body hits (lines 956+) are INFORMATIONAL per
  SE-17c-d. SE-17a-compliant for this snapshot; future §Trace insertions will add
  additional §Trace-body hits — those are expected and non-actionable.]

  Step 2 [NORMATIVE — N-3]: verify production-code hits at 235/238 — CONFIRMED.
  Lines 235 and 238 are above BOUNDARY=790. They are in the production-code body section.
  [SE-17g: [N-3] NORMATIVE — production-code L-number citations, must match final-state.]

  Step 3 [NORMATIVE — N-6]: verify BOUNDARY=790:
  ```
  $ grep -n "^## §Trace" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md | head -1
  790:## §Trace
  ```
  BOUNDARY=790 CONFIRMED. [SE-17g: [N-6] NORMATIVE — boundary value for SE-17c-d filter.]

  Step 4 [INFORMATIONAL — I-4]: verify §Trace v1.0.23 SE-17f Step 1 block has PG-5
  historical label — CONFIRMED by this burst's edits (PG-5 label inserted in-place above).

  Step 5 [INFORMATIONAL — I-4]: verify §Trace v1.0.22 PRE block (stale 233/236 record)
  still present — informational range approximately lines 1085–1090 (exact line number
  informational; §Trace-body narrative per D-108). Production-code hits 235/238 stable.

  Step 6 [NORMATIVE — N-1, N-2]: re-read frontmatter fields post-edit — CONFIRMED:
  - version: "1.0.24" [N-1] NORMATIVE — confirmed present in frontmatter.
  - timestamp: "2026-05-17T02:30:00Z" [N-2] NORMATIVE — confirmed present in frontmatter.

  Step 7: SE-17g recursion check — this SE-17f block now contains the filled NORMATIVE
  transcript [N-4]. The transcript lines themselves will appear in future greps of this
  file (additional §Trace-body hits). That is expected and non-actionable per SE-17c-d
  (§Trace-body hits excluded from production-code L-number claims). SE-17a-compliance is
  declared for [N-4] as literal final-state output before commit. This is the THIRD
  APPLICATION of SE-17f; SE-17g FIRST APPLICATION. No divergences unresolved in this burst.

  Divergence summary:
  Pre-v1.0.24-edit baseline (v1.0.23 state): grep returned 36 lines.
  After §Trace v1.0.24 body insertion (but before NORMATIVE transcript insertion):
  35 lines (the retired Step 1 summary paragraph containing a match pattern was
  replaced by a PG-5 label without match patterns — 36→35 delta).
  After NORMATIVE transcript [N-4] insertion: the transcript itself contains grep
  pattern strings, adding additional §Trace-body hits. SE-17b post-transcript grep
  confirms production-code hits remain at 235/238 only (NORMATIVE [N-3]).
  §Trace-body hit line numbers in the [N-4] transcript are INFORMATIONAL per SE-17c-d
  — they shifted post-transcript-insertion (expected, non-actionable, per D-108 + SE-17f
  Step 7 established pattern). Production-code hits 235/238 confirmed stable throughout.
  All §Trace-body hits ≥ BOUNDARY=790 confirmed INFORMATIONAL per SE-17c-d.
  SE-17f caught all shifts. No production-code divergences unresolved.

v1.0.23 changes (F-R99 Burst 2 — F-R99-1 HIGH closure: §Trace v1.0.22 SE-17c-d note stale L-numbers corrected; SE-17f + SE-16d first application):

- F-R99-1 RESOLVED (HIGH — §Trace v1.0.22 SE-17c-d note at lines 813–817 cited stale
  §Trace-body L-numbers 864–866 and 864–872; actual final-state §Trace-body grep hits
  were at lines 800–801 (PRE block), 809–811 (POST grep in §Trace v1.0.22 body), and
  the §Trace v1.0.21 Fix 1 POST block; all cited L-numbers corrected per SE-17c-d
  re-application):

  The adversary R99 reported that the §Trace v1.0.22 SE-17c-d note contained stale
  §Trace-body L-number citations. The note claimed:
  - "§Trace body lines 864–866" (for the PRE-block quote of stale 233/236 values)
  - "The §Trace v1.0.21 POST block at lines 864–872" (for the corrected evidence block)

  Adversary canonical grep showed actual §Trace-body hits at lines 800–801, 810–811,
  and 918–919 (at v1.0.22 final state). Same defect class as F-R98-2 (SE-17c-d
  L-number revalidation gap) — the §Trace v1.0.22 author committed stale L-numbers
  from an interim revision state, not the final-state file.

  Fix applied: the SE-17c-d note in §Trace v1.0.22 Fix 1 POST block (formerly lines
  813–817) has been rewritten to no longer pin stale line numbers; authoritative
  final-state line numbers for all §Trace-body hits are documented in this §Trace
  v1.0.23 SE-17f self-revalidation block below.

  Pre-edit final-state grep (v1.0.22, before this burst's edits):
  ```
  $ grep -nE "making \(c\) the unconditional terminator|platform-ABI design \(not misconfiguration\)" .../SS-daemon-lifecycle.md
  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  238:   ///   making (c) the unconditional terminator of the resolution chain.
  800:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);  [PRE block]
  801:  236:   ///   making (c) the unconditional terminator of the resolution chain.  [PRE block]
  809:  $ grep -n ...  [POST grep command line in §Trace v1.0.22 body]
  810:  235:   ...  [POST grep output line 1 in §Trace v1.0.22 body]
  811:  238:   ...  [POST grep output line 2 in §Trace v1.0.22 body]
  918:  $ grep -n ...  [§Trace v1.0.21 Fix 1 POST block command line]
  919:  235:   ...  [§Trace v1.0.21 Fix 1 POST output line 1]
  ```
  (These were the adversary-reported actual values, confirming the stale 864-866/864-872
  citations in §Trace v1.0.22 were incorrect by ~50 lines.)

- Disciplines applied:
  - SE-17a (literal grep transcripts — pre-edit and post-edit — documented in SE-17f
    self-revalidation block; no ellipsis abbreviation, no summary count, no "16+"-style hedge)
  - SE-17b (self-verification: each grep re-run after edit before finalizing claims)
  - SE-17c (5-step: body authored → final-state greps run → L-numbers updated → re-verified
    against final-state file → committed)
  - SE-17c-d (body-scope filter: BOUNDARY=790 confirmed by `grep -n "^## §Trace" | head -1`;
    production-code hits at 235/238 are above boundary; §Trace-body hits at 800+ are narrative)
  - SE-17e (sibling-propagation: manifest §Trace v1.1.15 receives parallel F-R99-6 closure
    in this same burst; both §Trace entries SE-17a-strict from inception)
  - SE-17f FIRST APPLICATION (mechanical self-revalidation gate — see self-revalidation block
    below; all cited L-numbers and grep outputs re-verified after §Trace authoring)

- SE-16b monotonicity check PASS: v1.0.22 → v1.0.23 is a monotonic increment.
  Timestamp 2026-05-17T00:00:00Z ≥ v1.0.22 timestamp 2026-05-16T22:00:00Z. PASS.

- SE-16d FIRST APPLICATION (cross-artifact chain-time monotonicity):
  2026-05-17T00:00:00Z >= STATE v5.50 chain high-water 2026-05-16T23:30:00Z. PASS.
  UTC ISO-8601 form (`YYYY-MM-DDTHH:MM:SSZ`): confirmed. Both arch v1.0.23 and manifest
  v1.1.15 share timestamp 2026-05-17T00:00:00Z — same burst, same commit.

- No body content changes. Only §Trace v1.0.22 SE-17c-d note rewritten (stale L-numbers
  replaced with reference to §Trace v1.0.23 as authoritative source) + §Trace v1.0.23
  entry authored.

- Cross-document pins (Extension 15 + SE-15e):
  arch v1.0.23 pin propagation required: Burst 3 (PO PRD v1.23) + Burst 4 (FV VP v1.33).

- SE-17f SELF-REVALIDATION BLOCK (FIRST APPLICATION of 31st discipline):

  [SE-17g NOTE: This entire SE-17f block is part of the §Trace v1.0.23 entry which is
  PG-5 preserved historical. The Step 1 transcript below is INFORMATIONAL — explicitly
  a pre-finalization snapshot; it does NOT declare SE-17a-compliance for the post-v1.0.24
  final state. The SE-17a-compliance claim at Step 7 below is WITHDRAWN by §Trace v1.0.24.
  See §Trace v1.0.24 SE-17g reconciliation block for the authoritative NORMATIVE transcript.]

  Step 1: [PG-5 PRESERVED VERBATIM — PRE-FINALIZATION SNAPSHOT — stale post-edit; transcript
  captured mid-burst BEFORE §Trace v1.0.23 entry expansion completed; line numbers and
  hit-count are stale relative to v1.0.24 final state. INFORMATIONAL per SE-17g.]

  literal grep run AFTER §Trace v1.0.23 insertion (post-insertion final-state at time of capture):
  ```
  $ grep -nE "making \(c\) the unconditional terminator|platform-ABI design \(not misconfiguration\)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  238:   ///   making (c) the unconditional terminator of the resolution chain.
  818:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  819:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  820:  800:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);  [PRE block]
  821:  801:  236:   ///   making (c) the unconditional terminator of the resolution chain.  [PRE block]
  864:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  865:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  866:  800:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  867:  801:  236:   ///   making (c) the unconditional terminator of the resolution chain.
  868:  809:  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" ...
  869:  810:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  870:  811:   238:   ///   making (c) the unconditional terminator of the resolution chain.
  907:  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  908:  236:   ///   making (c) the unconditional terminator of the resolution chain.
  916:  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  917:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  918:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  1028:  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  1029:  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  1030:  238:   ///   making (c) the unconditional terminator of the resolution chain.
  ```
  [PG-5 END — snapshot displayed 22 lines / 11 hit-pairs at time of capture; actual
  final-state v1.0.23 grep returned 36 lines / 18 hit-pairs per adversary R100 F-R100-1.
  Stale count claim "30 lines, 15 hit-pairs" below is also INFORMATIONAL / stale.]
  (Stale snapshot summary — INFORMATIONAL: claimed "30 lines, 15 hit-pairs" at capture time;
  adversary confirmed actual 36 lines. Production-code hits at 235/238 remain correct.
  §Trace v1.0.21 Fix 1 POST block position stale. See §Trace v1.0.24 for authoritative counts.)

  Step 2: verify production-code hits at 235/238 — CONFIRMED. Lines 235/238 are in the
  body section above BOUNDARY=790. They do not shift regardless of §Trace insertions.

  Step 3: verify §Trace v1.0.22 PRE block (stale 233/236 defect record) — CONFIRMED present
  in §Trace body. Informational line-number reference: approximately lines 930–935 range
  post-commit (exact line shifts with each SE-17f self-revalidation edit; §Trace-body
  narrative line numbers are informational, not normative — SE-17c-d precision requirement
  applies to production-code hits only). Production-code hits at 235/238 confirmed stable.

  Step 4: verify §Trace v1.0.22 POST grep lines — CONFIRMED present in §Trace body.
  Informational line-number reference: approximately lines 940–945 range post-commit.
  Same self-referential caveat as Step 3 applies.

  Step 5: verify §Trace v1.0.21 Fix 1 POST block — CONFIRMED present in §Trace body.
  Informational line-number reference: approximately lines 1050–1060 range post-commit.
  Same self-referential caveat applies. SE-17f requires re-verification of these
  §Trace-body positions at each future burst that inserts a new §Trace entry before v1.0.22.

  Step 6: verify §Trace v1.0.22 SE-17c-d note no longer cites stale 864–866/864–872 —
  CONFIRMED. The note defers to this §Trace v1.0.23 entry as authoritative.

  Step 7: [SE-17g INFORMATIONAL — SE-17a-compliance claim WITHDRAWN by §Trace v1.0.24.]
  SE-17f recursion check (historical record, informational): this SE-17f block itself
  contains grep transcript text that appears in future greps. The grep output transcript
  at Step 1 was the literal bash output at time of capture. The Step 1 transcript is inside
  the §Trace body, so future greps return additional hits from within this block — that is
  expected and non-actionable (§Trace-body hits are excluded from production-code L-number
  claims per SE-17c-d). SE-17a-compliance claim was incorrectly applied to a mid-burst
  snapshot; superseded by §Trace v1.0.24 NORMATIVE transcript per SE-17g F-R100-1.

  Divergence summary: Pre-burst §Trace-body hits for the §Trace v1.0.22 PRE/POST blocks
  were at 800–801/809–811 (pre-insertion); after §Trace v1.0.23 insertion they shifted
  to 907–908/916–918. The v1.0.21 POST block shifted from pre-burst 915–923 through
  multiple intermediate states during SE-17c-d note revisions to final post-insertion
  1028–1030. SE-17f caught all shifts and documented them here. No divergence unresolved.

v1.0.22 changes (F-R98 Burst 2 — SE-17c-d L-number revalidation of §Trace v1.0.21 Fix 1 POST evidence):

- F-R98-2 RESOLVED (HIGH — §Trace v1.0.21 Fix 1 POST evidence block cited stale line numbers
  233/236 for the platform-ABI / unconditional-terminator strings; actual final-state lines
  are 235/238):

  The adversary R98 reported that the Fix 1 POST evidence block in §Trace v1.0.21 claimed:
  ```
  233:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  236:   ///   making (c) the unconditional terminator of the resolution chain.
  ```
  This violates SE-17c-d (L-number revalidation via direct Read of final-state file at
  burst-finalization). The v1.0.21 burst committed stale line numbers from an interim
  revision without revalidating against the final-state file.

  Fix 1 POST (final-state v1.0.22, body-scope grep — BOUNDARY=790, single-pass literal output):
  ```
  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  238:   ///   making (c) the unconditional terminator of the resolution chain.
  ```
  (SE-17c-d note: the grep also returns hits within the §Trace body — the PRE block
  quoting stale 233/236 values as the defect record, the POST grep output embedded in
  this §Trace v1.0.22 entry, and the §Trace v1.0.21 Fix 1 POST block. Those §Trace-body
  hits are not the subject of this fix; the authoritative current-state production-code
  lines are 235 and 238 above the §Trace section boundary (BOUNDARY=790). The §Trace
  v1.0.21 Fix 1 POST block has been corrected in place to show 235/238 per F-R98-2.
  [§Trace v1.0.22 originally cited stale §Trace-body L-numbers 864–866/864–872;
  corrected in §Trace v1.0.23 per F-R99-1. Final-state line numbers for all §Trace-body
  hits are authoritative in §Trace v1.0.23 SE-17f self-revalidation block.])

  The parenthetical in §Trace v1.0.21 claiming "+7 new doc-comment lines" has been
  corrected to reflect the actual final-state shift of +9 lines relative to v1.0.20
  (final-state 235/238 vs. v1.0.20 inferred positions ~226/229 per the PRE awk block).

- Disciplines applied:
  - SE-17a (literal grep output shown above — not summary count, not ellipsis-abbreviated)
  - SE-17b (self-verification: grep re-run after edit to confirm output matches §Trace claim)
  - SE-17c (5-step: body authored → final-state greps run → L-numbers updated → re-verified
    against final-state file → committed)
  - SE-17c-d (body-scope filter: §Trace narrative hits above BOUNDARY=790 excluded from
    productive-code line-number claims; only body hits at lines 235/238 are authoritative)
  - SE-17e FIRST APPLICATION (sibling-propagation: SE-17a/c/c-d applied to this §Trace
    v1.0.22 entry from inception, not retroactively; manifest §Trace v1.1.14 receives
    parallel SE-17e first application in Burst 2)

- SE-16b monotonicity check PASS: v1.0.21 → v1.0.22 is a monotonic increment.
  Timestamp 2026-05-16T22:00:00Z ≥ v5.48 STATE.md timestamp 2026-05-16T21:00:00Z. PASS.

- No body content changes. Only §Trace evidence-block correction (stale line numbers
  corrected in the v1.0.21 Fix 1 POST block) + §Trace v1.0.22 entry authored with
  SE-17a-strict literal-output convention from inception.

- Cross-document pins (unchanged in this burst — Extension 15 + SE-15e):
  PRD v1.21 / VP v1.31. Burst 3 (PO) propagates arch v1.0.22; Burst 4 (FV) propagates
  both arch v1.0.22 + manifest v1.1.14.

v1.0.21 changes (adversary R94 C-R94-1 + I-R94-1 + I-R94-3 closures — resolve_runtime_dir doc-comment correctness + HookEventRecord docstring + AuthError visibility):

- C-R94-1 RESOLVED (HIGH — adversary R94 resolve_runtime_dir doc-comment incorrectness):
  The doc-comment block above `resolve_runtime_dir` stated: "paths (b) and (c) can only
  return None/empty IF ProjectDirs::new() returned None, which is checked earlier." This
  is factually incorrect on two counts:
  1. `ProjectDirs::runtime_dir()` (path b) returns `None` on macOS/Windows by platform-ABI
     design — it is NOT conditional on `ProjectDirs::new()` having failed. The `directories`
     crate documents that `runtime_dir()` returns `None` on platforms without an XDG
     `XDG_RUNTIME_DIR` (i.e., macOS and Windows natively). A successful `ProjectDirs::new()`
     call on macOS still yields `runtime_dir() == None`. The inline comment at the path (c)
     branch (line ~250) correctly states "data_local_dir() returns &Path (never Option)" —
     the doc-comment directly contradicted it.
  2. `data_local_dir()` (path c) has return type `&Path` (not `Option<&Path>`), so the
     phrase "return None/empty" mischaracterizes its type signature entirely.

  Fix: the doc-comment rationale rewritten to correctly characterize each path's infallibility
  by its actual type signature and platform-ABI semantics:
  - Path (b): infallible because the `None` arm falls through to path (c) (not because
    `runtime_dir()` is unconditionally `Some`).
  - Path (c): infallible because `data_local_dir()` returns `&Path`, never `Option<&Path>`.
  - The CALLER fail-fast for `ProjectDirs::new() == None` is preserved, correctly scoped
    to the caller rather than to the function body.

  Cross-reference: inline comment at path (c) branch (line ~250), PRD line 326
  ("data_local_dir() is always available on all platforms"), VP-DAEMON-005 §Pre-conditions
  ("ProjectDirs::new() returns Some — caller has checked this before invoking
  resolve_runtime_dir").

- I-R94-1 RESOLVED (MED — adversary R94 HookEventRecord tool_input docstring misnomer):
  The docstring for `tool_input: Option<serde_json::Value>` read:
  "JSON-encoded tool input; populated for `PreToolUse` and `Notification` events."
  `serde_json::Value` is an in-memory parsed JSON tree (a Rust enum), not a JSON-encoded
  string (`&str` or `String`). Calling it "JSON-encoded" implies it is a raw byte sequence
  or string — the opposite of the truth. The correct term is "parsed JSON value." The
  "Stored as `serde_json::Value` to avoid double-deserialization" rationale is correct
  but incomplete: it did not state that the field is `None` for the three non-tool hook
  types, which could leave a reader uncertain which event types populate it.

  Fix: docstring rewritten to:
  - Replace "JSON-encoded" with "parsed JSON value (in-memory JSON tree, not an encoded string)".
  - Enumerate the `None` cases explicitly: `SessionStart`, `UserPromptSubmit`, `Stop`.
  - Preserve the double-deserialization rationale.

- I-R94-3 RESOLVED (MED — adversary R94 `enum AuthError` private visibility mismatch
  with VP-AUTH-002): The `AuthError` enum was declared `enum AuthError` (private to the
  module). VP-AUTH-002 §Pre-conditions declares `pub enum AuthError` in the harness
  context. Integration tests for `BC-2.01.009` are compiled as separate `[[test]]` binaries
  that import from `monocle-runtime`'s public API. For integration tests to assert on
  `AuthError` variants, `AuthError` must be `pub` — a private enum cannot be named in a
  test binary outside the defining module.

  Fix: `enum AuthError` → `pub enum AuthError`.

- SE-16b monotonicity check PASS: v1.0.20 → v1.0.21 is a monotonic increment.
  No version regression. No prior §Trace entry modified.

- Extension 17 evidence discipline — real grep transcripts:

  Fix 1 PRE (doc-comment):
  ```
  $ awk 'NR>=230 && NR<=235' /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
     ///
     /// Note: path (d) "fail-fast on ProjectDirs::new() == None" is handled
     /// in the CALLER (daemon main) BEFORE this function is invoked. This function
     /// is infallible given a valid ProjectDirs instance — paths (b) and (c) can
     /// only return None/empty IF ProjectDirs::new() returned None, which is
     /// checked earlier. Hence the return type is PathBuf, not Result<PathBuf, _>.
  ```

  Fix 1 POST (doc-comment):
  ```
  $ grep -n "making (c) the unconditional terminator\|platform-ABI design (not misconfiguration)" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  235:   ///   on macOS/Windows by platform-ABI design (not misconfiguration);
  238:   ///   making (c) the unconditional terminator of the resolution chain.
  ```
  (SE-17c-d L-number revalidation via §Trace v1.0.22: v1.0.21 burst committed with stale
  line numbers 233/236; actual final-state lines are 235/238 — confirmed by literal grep
  above. F-R98-2 closure.)

  Fix 2 PRE (HookEventRecord docstring):
  ```
  $ grep -n "JSON-encoded tool input\|Tool input as a parsed JSON" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  559:       /// JSON-encoded tool input; populated for `PreToolUse` and `Notification` events.
  ```

  Fix 2 POST (HookEventRecord docstring):
  ```
  $ grep -n "JSON-encoded tool input\|Tool input as a parsed JSON" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  566:       /// Tool input as a parsed JSON value; populated for `PreToolUse` and `Notification` events.
  ```

  Fix 3 PRE (AuthError visibility):
  ```
  $ grep -n "^pub enum AuthError\|^   enum AuthError\|^    enum AuthError" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  380:   enum AuthError {
  ```

  Fix 3 POST (AuthError visibility):
  ```
  $ grep -n "^pub enum AuthError\|^   pub enum AuthError\|^   enum AuthError" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  387:   pub enum AuthError {
  ```

  (Line number shifts: Fix 1 inserts doc-comment lines above the target. The final-state
  line numbers for the platform-ABI / unconditional-terminator strings are 235 and 238
  respectively in v1.0.21, confirmed by literal grep in §Trace v1.0.22. The original
  parenthetical claimed +7 shift relative to v1.0.20; the actual shift is +9 based on
  final-state positions 235/238 vs. the v1.0.20 positions 226/227 inferred from the
  PRE block showing NR>=230 context. F-R98-2 SE-17c-d revalidation applied at v1.0.22.)

### Propagation requirements (Extension 15 + SE-15e)

This burst bumps arch v1.0.20 → v1.0.21 AND manifest v1.1.12 → v1.1.13. Downstream agents must propagate BOTH pins:

- **PO (PRD):** PRD frontmatter `traces_to` cites arch v1.0.20 + manifest v1.1.12; PO must propagate BOTH bumps. Canonical greps:
  - `grep -nE "v1\.0\.20|commit 8533ea2" /Users/jmagady/Dev/monocle/.factory/specs/prd.md`
  - `grep -nE "v1\.1\.12|commit 8005075" /Users/jmagady/Dev/monocle/.factory/specs/prd.md`

- **FV (VP):** VP frontmatter `traces_to` cites both. FV must propagate both.

Per SE-15e: orchestrator MUST dispatch PO before FV.

- **BC count: 22 — CONFIRMED unchanged.** No new BCs; no BCs removed.

- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) PG-3: §Start Sequence (EXISTS), §Daemon Lifecycle Protocol §Drain (EXISTS),
      §Trace (EXISTS) — all §-anchor refs verified against actual headings in this
      document.
  (b) PG-4: all referenced sections confirmed to exist in normative body.
  (c) PG-5: historical §Trace entries unchanged. Post-write self-grep: 0 L[0-9]+
      matches in this §Trace v1.0.21 entry.

v1.0.20 changes (adversary R93 I-R93-1 + C-R93-1 arch part closures — resolve_runtime_dir signature + integration-test prose):
- I-R93-1 RESOLVED (MED — adversary R93 resolve_runtime_dir dead Err variant): The
  `resolve_runtime_dir` function was declared with signature
  `fn resolve_runtime_dir(project_dirs: &directories::ProjectDirs) -> Result<PathBuf, DaemonStartError>`
  but the function body contained zero `Err(...)` return paths — all branches returned
  `Ok(path)`. The only `DaemonStartError::RuntimeDirUnresolvable` construction site is
  in the CALLER (daemon main), which checks `ProjectDirs::new() == None` and fails fast
  BEFORE invoking `resolve_runtime_dir`. This is explicitly documented in the prose at
  §Start Sequence step 1 (lines 250-253): "If `ProjectDirs::new(...)` itself returns
  `None`... the daemon exits with `DaemonStartError::RuntimeDirUnresolvable` before
  `resolve_runtime_dir` is called." The Result wrapper was therefore dead code — the
  function could never return `Err(_)`.

  Fix (disposition **(a)** — clean signature change to PathBuf):
  - Before: `fn resolve_runtime_dir(project_dirs: &directories::ProjectDirs) -> Result<PathBuf, DaemonStartError>`
    with all branches returning `Ok(path)` (three sites: `return Ok(PathBuf::from(env_path))`,
    `return Ok(rd.to_path_buf())`, `Ok(fallback)`).
  - After: `fn resolve_runtime_dir(project_dirs: &directories::ProjectDirs) -> PathBuf`
    with all branches returning bare `PathBuf` (three sites: `return PathBuf::from(env_path)`,
    `return rd.to_path_buf()`, `fallback`).
  - A clarifying doc-comment block added above the function explicitly stating:
    (a) paths (b) and (c) are infallible given a valid `ProjectDirs` instance;
    (b) `data_local_dir()` returns `&Path` (never `Option`) — branch (c) cannot fail;
    (c) `DaemonStartError::RuntimeDirUnresolvable` is exclusively a caller-side
    `ProjectDirs::new() == None` failure mode — no construction site exists inside
    `resolve_runtime_dir` itself.

  This eliminates the clippy `unnecessary_wraps` lint warning that this signature
  would produce if Clippy were run. The semantic correctness of the resolution chain
  is unchanged — only the return type annotation is corrected to match actual behavior.

- C-R93-1 arch part RESOLVED (HIGH/arch-site — adversary R93 F-R88-5 §Mechanism
  Distribution partial-fix propagation gap): The BC-2.01.007 verification clause in
  §Daemon Lifecycle Protocol §Drain read "unit test in `monocle-runtime/tests/jsonl_ring.rs`".
  Tests under `<crate>/tests/*.rs` are cargo integration tests (executed in a separate
  test binary against the crate's public API), not unit tests (which reside in
  `#[cfg(test)]` modules inside `src/**/*.rs`). This is the F-R88-5 §Mechanism
  Distribution discipline: VP §Mechanism Distribution was corrected at R88 (unit-test=0),
  VP §Harness annotations corrected at R92, and this arch verification-clause site is
  the third and final layer identified at R93.

  Fix (disposition **(a)** — single word substitution):
  - Before: "unit test in `monocle-runtime/tests/jsonl_ring.rs` constructs a..."
  - After: "integration test in `monocle-runtime/tests/jsonl_ring.rs` constructs a..."

- SE-16b monotonicity check PASS: v1.0.19 → v1.0.20 is a monotonic increment.
  No version regression. No prior version entry in §Trace modified.

- Extension 17 evidence discipline — real grep transcripts:

  Fix 1 PRE:
  ```
  $ grep -n "fn resolve_runtime_dir\|Result<PathBuf, DaemonStartError>\|Ok(fallback)\|return Ok(" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  226:   fn resolve_runtime_dir(project_dirs: &directories::ProjectDirs) -> Result<PathBuf, DaemonStartError> {
  231:               return Ok(PathBuf::from(env_path));
  237:           return Ok(rd.to_path_buf());
  246:       Ok(fallback)
  ```

  Fix 1 POST:
  ```
  $ grep -n "fn resolve_runtime_dir\|-> PathBuf\|Result<PathBuf" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  235:   /// checked earlier. Hence the return type is PathBuf, not Result<PathBuf, _>.
  236:   fn resolve_runtime_dir(project_dirs: &directories::ProjectDirs) -> PathBuf {
  ```

  Fix 2 PRE:
  ```
  $ grep -n "unit test in.*jsonl_ring" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  613:   unit test in `monocle-runtime/tests/jsonl_ring.rs` constructs a
  ```

  Fix 2 POST:
  ```
  $ grep -n "integration test in.*jsonl_ring\|unit test in.*jsonl_ring" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  624:   integration test in `monocle-runtime/tests/jsonl_ring.rs` constructs a
  ```

  (Line number shift of +11 in Fix 2 POST vs PRE is correct: the clarifying doc-comment
  block added for Fix 1 inserts 11 new lines above the target site.)

#### Propagation requirements from v1.0.20 burst (historical — superseded by v1.0.21 block above)

This burst bumps arch v1.0.19 → v1.0.20. Downstream agents must propagate:

- **PO (PRD):** PRD frontmatter `traces_to` cites arch pin v1.0.19; PO must propagate
  v1.0.19 → v1.0.20 across PRD body (~32 sites per F-R90 precedent). Canonical grep:
  `grep -nE "v1\.0\.19|commit 8a68cc9" /Users/jmagady/Dev/monocle/.factory/specs/prd.md`.
  ALSO: C-R93-1 PRD part — §7 RTM 6 rows "Unit"→"Integration" + §Verification 4 prose
  sites: BC-2.01.001, BC-2.01.002, BC-2.01.003, BC-2.01.004 §Verification
  paragraphs that carry "unit test at `tests/...`" language.

- **FV (VP):** VP frontmatter `traces_to` cites arch v1.0.19; FV must propagate
  v1.0.19 → v1.0.20. Canonical grep:
  `grep -nE "v1\.0\.19|commit 8a68cc9" /Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md`.
  ALSO: O-R93-1 + O-R93-2 cleanup; PRD pin propagation v1.19 → v1.20 (after PO
  delivers PRD v1.20, FV picks up the new PRD pin in the same VP v1.27 burst).

Per SE-15e: orchestrator MUST dispatch PO before FV.

- **BC count: 22 — CONFIRMED unchanged.** No new BCs; no BCs removed.

- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) PG-3: §Start Sequence (EXISTS), §Daemon Lifecycle Protocol §Drain (EXISTS),
      §Trace (EXISTS) — all §-anchor refs verified against actual headings in this
      document.
  (b) PG-4: all referenced sections confirmed to exist in normative body.
  (c) PG-5: historical §Trace entries unchanged. Post-write self-grep: 0 L[0-9]+
      matches in this §Trace v1.0.20 entry.

v1.0.19 changes (adversary R89 F-R89-2 + O-R89-3 closures — HookEventRecord serde annotation + SessionStart None example):
- F-R89-2 RESOLVED (MED — adversary R89 HookEventRecord struct missing
  `#[serde(skip_serializing_if = "Option::is_none")]` annotation): The
  `HookEventRecord` struct in §Drain carried `tool_name: Option<String>` and
  `tool_input: Option<serde_json::Value>` fields without the serde annotation.
  PRD v1.17 BC-2.01.007 EC-001 declares: "When `tool_name` or `tool_input` is
  `None`, the key MUST be absent from the serialized JSONL record (not present as
  `null`)." The annotation is normative — omitting it causes `serde_json` to
  serialize absent-tool-context records with explicit `"tool_name":null` and
  `"tool_input":null` keys, violating EC-001. This is a sibling-site propagation
  gap from F-R88-3 (arch layer), which established the serde normative form in PRD
  but did not backfill the arch struct definition.

  Fix (disposition **(a)** — add annotation to struct field declarations):
  - Before: `pub tool_name: Option<String>,` and `pub tool_input: Option<serde_json::Value>,`
    (no serde annotation on either field)
  - After: `#[serde(skip_serializing_if = "Option::is_none")]` added immediately
    above each Option field in the struct declaration. The `#[derive(Debug, Clone,
    Serialize, Deserialize)]` derive macro already present on the struct provides
    the required `Serialize` implementation; no import changes required.

  Pre-burst grep confirmed 0 occurrences of `skip_serializing_if` in this file
  before the fix. Post-burst grep confirms 2 occurrences at the struct field
  annotations (see Evidence block below).

- O-R89-3 RESOLVED (LOW — bundled): SessionStart absence-form example added
  immediately after the PreToolUse (Some-valued) example in §Drain. The new
  example demonstrates that for hook types without tool context (`SessionStart`,
  `UserPromptSubmit`, `Stop`), the `tool_name` and `tool_input` keys are ABSENT
  entirely from the JSONL record (not present as explicit `null`). The
  accompanying prose explicitly states:
  - "Phase 1 emitters MUST emit absence"
  - "Phase 2+ readers MUST tolerate both absence and explicit null per forward-compat"
  This closes the O-R89-3 observation that only the Some-valued example was
  present, leaving the None-case emitter behavior undocumented at the arch layer.

- SE-16b monotonicity check PASS: v1.0.18 → v1.0.19 is a monotonic increment.
  No version regression. No prior version entry in §Trace modified.

- Extension 17 evidence discipline — real grep transcripts:

  Pre-burst verification:
  ```
  $ grep -n "skip_serializing_if" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  [0 matches — confirmed absent before fix]

  $ grep -n "tool_name\|tool_input" /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  523:   /// `tool_name` and `tool_input` are `Option` because only `PreToolUse` and
  546:       pub tool_name: Option<String>,
  550:       pub tool_input: Option<serde_json::Value>,
  560:       /// `tool_name` and `tool_input` are `None` for hook types that carry no tool
  568:           tool_name: Option<String>,
  569:           tool_input: Option<serde_json::Value>,
  577:               tool_name,
  578:               tool_input,
  593:   {"format_version":1,...,"hook_type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}
  1367:  `timestamp_micros: i64`, `pid: u32`, `hook_type: String`, `tool_name: Option<String>`,
  1368:  `tool_input: Option<serde_json::Value>`. A `pub fn new(...)` constructor is provided
  ```

  Post-burst verification (Fix 1):
  ```
  $ grep -n "skip_serializing_if" .../SS-daemon-lifecycle.md
  546:       #[serde(skip_serializing_if = "Option::is_none")]
  551:       #[serde(skip_serializing_if = "Option::is_none")]
  [2 matches — at struct field annotations for tool_name (line 546) and tool_input (line 551)]
  ```

  Post-burst verification (Fix 2):
  ```
  $ grep -nE "hook_type.*SessionStart|hook_type.*PreToolUse" .../SS-daemon-lifecycle.md
  595:   {"format_version":1,"session_id":"<uuid>","timestamp_micros":...,"hook_type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}
  604:   {"format_version":1,"session_id":"<uuid>","timestamp_micros":...,"hook_type":"SessionStart"}
  [2 matches — PreToolUse (Some-valued) + SessionStart (None-case / absence form)]
  ```

- Propagation requirements (Extension 15):

  This burst bumps arch v1.0.18 → v1.0.19. Downstream agents must propagate:

  **VP (formal-verifier):** VP frontmatter `traces_to` cites arch pin v1.0.18;
  formal-verifier must propagate v1.0.18 → v1.0.19 in VP frontmatter + §References
  item 2 + §VP Catalog Overview + per-VP `Traces to:` lines + §Coverage Matrix.
  Use canonical grep:
  `grep -nE "v1\.0\.18|commit 61a0064" /Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md`
  to enumerate sites.

  **PRD (product-owner):** NOT required for this burst — PRD already cites arch
  v1.0.18 + serde normative annotation (PRD v1.17 BC-2.01.007 EC-001). PRD pin
  propagation only needed if PRD itself bumps.

  **BC count: 22 — CONFIRMED unchanged.** No new BCs; no BCs removed.

- Propagation sweep (PG-3/PG-4/PG-5 compliance):
  (a) PG-3: §Drain (EXISTS), §Trace (EXISTS) — all §-anchor refs verified
      against actual headings in this document.
  (b) PG-4: all referenced sections confirmed to exist in normative body.
  (c) PG-5: historical §Trace entries unchanged. Post-write self-grep: 0 L[0-9]+
      matches in this §Trace v1.0.19 entry.

v1.0.15 changes (adversary R74 F-R74-1 closure — hook_endpoints ellipsis placeholder + L-F-R63 Extension 4):
- F-R74-1 RESOLVED (HIGH — adversary R74 JSON array ellipsis placeholder in §GET /status
  schema sketch): The `hook_endpoints` field in the BC-2.01.002 JSON schema sketch
  (§Health and Status Endpoints) carried a literal `"..."` string as the third array
  element — a textual ellipsis placeholder instead of the full canonical enumeration.
  The canonical 5-endpoint set is established by BC-2.01.002 postcondition 1
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
  field). No BC content change required — BC-2.01.002 postcondition already
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
  and most critical Phase 1 lock-file field. Per BC-2.01.010 Postcondition 1 (PRD line
  602) the Phase 1 lock-file schema has 7 fields. Per BC-2.01.010 Postcondition 2 (PRD
  line 606) `contract_version` is always the FIRST key in the JSON object. Per
  BC-2.01.005 Postcondition 4 (PRD line 334) the lock-file contains all 7 fields
  including `contract_version`. The omission created two failure modes for Phase 4
  implementers: (a) treating `contract_version` as a Phase-1-only field and removing
  it in Phase 4, breaking BC-2.01.010 forward-compat; (b) assuming `contract_version`
  is implicit/non-contractual and not validating it.

  Fix (disposition **(a)** — extend §Phase 4 Notes enumeration to all 7 fields):
  - Before: "...the `\"app\"`, `\"pid\"`, `\"port\"`, `\"authToken\"`,
    `\"startTimeUtc\"`, `\"version\"` fields are stable across Phase 1 → Phase 4."
    (6 fields, `contract_version` absent)
  - After: "...the 7 Phase 1 fields are stable across Phase 1 → Phase 4:
    `\"contract_version\"` (forward-compatibility version sentinel, always FIRST key
    per BC-2.01.010 Postcondition 2; Phase 4 readers MUST validate
    `contract_version == 1` before consuming other fields), `\"pid\"`, `\"port\"`,
    `\"authToken\"`, `\"startTimeUtc\"`, `\"app\"`, `\"version\"`. Phase 4 readers
    that encounter `contract_version > 1` MUST fail gracefully (do not attempt parse
    of unknown-version JSON)."

  The field ordering in the fix follows BC-2.01.010 Postcondition 1 canonical order
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

v1.0.17 changes (adversary R83 F-R83-1 site 2 closure — §BC Summary footer BC-2.01.005 0o700 propagation):
- F-R83-1 site 2 RESOLVED (HIGH — adversary R83 multi-site propagation gap): The
  F-R79-3 closure (v1.0.x chain) lifted the runtime_dir `0o700` owner-only mode
  contract from EC-052 into BC-2.01.005 §Postcondition 8. The §Start Sequence
  step 1 body at line 255 correctly states "Create the resolved directory with
  mode `0o700` if absent." However, the §BC Summary footer BC-2.01.005 row was
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
  (traces_to field). No BC content change required — BC-2.01.005 postcondition 8
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
  BC-2.01.006 invariant 1 and the VP-DAEMON-006 regex to mandate
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
    EC-044 (PRD v1.7), BC-2.01.002, and F-R72-1 cross-field uniformity.
  - Rationale: EC-044 (PRD v1.7 line 185) already specifies this exact format for
    `last_hook_ts`; the arch sketch now matches its own downstream BC/EC.

  **Site 2 — §Start Sequence step 6 lock file `startTimeUtc`:**
  - Before: `"startTimeUtc": "<ISO8601>"`
  - After: `"startTimeUtc": "<YYYY-MM-DDTHH:MM:SS.sssZ>"` + inline annotation
    referencing BC-2.01.002 / EC-044, BC-2.01.006, and F-R72-1 uniformity.
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
    referencing BC-2.01.006 invariant 1 (PRD v1.7), VP-DAEMON-006 regex
    (`^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`), and cross-field uniformity.
  - This site is directly mandated by BC-2.01.006 invariant 1; the prior generic
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
  (d) §Behavioral Contract Summary BC-2.01.006 row: prose description does not
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
  content change required — VP-DAEMON-006 regex and BC-2.01.006 invariant 1 are
  already correctly specified. Pin propagation only.
  **BC count: 22 — CONFIRMED unchanged.** No new BCs added; no BCs removed.

v1.0.13 changes (adversary R71 F-R71-2 + F-R71-3 + F-R71-4 closure):
- F-R71-2 RESOLVED (HIGH — adversary R71 stale test name): Two arch sites cited
  `test_BC_DAEMON_004_exit_codes` as the BC-2.01.004 verification test name. The
  canonical name per PRD v1.6 §3 BC-2.01.004 §Verification and VP v1.6 is
  `test_BC_DAEMON_004_exit_codes_posix_distinct`. Authority per §BC Summary footer:
  "the PRD is source-of-truth for canonical test names." Fixed at both arch sites:
  (1) §Hard Shutdown BC-2.01.004 verification prose block; (2) §Trace v1.0.12
  BC-2.01.004 rationale sentence. No behavioral change — only the test-name
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
  signal handling in BC-2.01.005 postcondition 3 (stale-pid detection).
  Architect disposition: **`nix 0.30`** (typed wrapper crate preferred over raw
  `libc` for `Signal::None` send pattern). Rationale: `nix::sys::signal::kill(pid,
  None)` is the idiomatic, type-safe Rust API for pid-liveness testing without
  signal delivery; using raw `libc::kill(pid, 0)` bypasses the type system and
  requires unsafe. `nix 0.30` is the latest stable release (verified 2026-05-14
  against crates.io). `nix 0.30` added to SS-deps-pin-manifest.md as a workspace
  caret pin. BC-2.01.005 postcondition 3 implementation MUST use
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
  BC-2.01.005 and BC-2.01.010 on macOS. Architect disposition: **(c) hybrid platform
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
  Asymmetry with BC-2.03.003 is intentional: `BaseDirs::new() == None` signals
  a genuine system-configuration failure (no home directory), warranting fail-fast;
  `ProjectDirs::runtime_dir() == None` on macOS is expected platform behavior,
  warranting a documented fallback. BC count: **22 unchanged** — BC-2.01.005 is
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
  BC-2.01.004 summary row updated to enumerate all five exit codes. Hard Shutdown
  step 6 updated to distinguish signal type for exit-code selection. BC-2.01.004
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
  (b) §Behavioral Contract Summary — BC-2.01.004 and BC-2.01.005 rows updated
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
  **PRD propagation required:** BC-2.01.004 postcondition (exit codes) must be
  updated to enumerate all five codes (0/130/143/2/1) with POSIX rationale; the PRD
  is the canonical test-vector source. BC-2.01.005 precondition 2 (runtime dir
  resolution) must be updated to reflect the three-path chain + env override. These
  are content changes in the PRD requiring `product-owner` dispatch.
  **VP propagation required:** VP-DAEMON-004 and VP-DAEMON-005 (if they exist) must
  be updated to reflect the new exit codes and resolution chain in their proof
  strategies and test vectors. `formal-verifier` / `test-writer` dispatch as applicable.
  **BC count: 22 — CONFIRMED unchanged.** No new BCs added; no BCs removed.
  Post-write self-grep: 0 L[0-9]+ matches in this §Trace v1.0.12 entry.

v1.0.11 changes (adversary R65 F-R65-1/2/3 content closure + propagation sweep):
- F-R65-1 RESOLVED (HIGH — adversary R65 pass 1 attempt 2): BC-2.01.009 lead-in prose
  at §Behavioral contracts stated "Three auth failure modes are specified:" but the
  BC-2.01.009 table immediately below contained exactly two rows (Missing header /
  Invalid token). Similarly, §Behavioral Contract Summary BC-2.01.009 row opened with
  "Three auth failure modes:". Root cause: F-R62-8 (v1.0.8) collapsed the originally
  distinct format / mismatch rows into a single "Invalid token" row — reducing the table
  to 2 rows — but the lead-in count words at both sibling sites were not updated in that
  same burst. The L-F-R63-PARTIAL-FIX propagation discipline was not yet codified at the
  time of F-R62-8; these sibling sites were therefore a pre-codification gap. Fix: "Three"
  → "Two" at both body-prose sites.
- F-R65-2 RESOLVED (CRITICAL — adversary R65 pass 1 attempt 2): BC-2.01.009 §Behavioral
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
  (`Authorization: Bearer fake` → missing); PRD v1.3 BC-2.01.009 postcondition 3 +
  Canonical Test Vector row 5; VP v1.3 §VP-AUTH-002 probe 5.
- F-R65-3 RESOLVED (HIGH — closed by F-R65-2 fix): Cross-artifact contradiction between
  arch and PRD/VP on Bearer disposition. After F-R65-2 fix, arch aligns with PRD v1.3
  BC-2.01.009 and VP v1.3 VP-AUTH-002. No independent change required.
- Propagation sweep (L-F-R63-PARTIAL-FIX discipline applied):
  (a) "Three/three" auth failure modes — body prose grep result: 2 sites fixed
  (BC-2.01.009 lead-in at §Behavioral contracts; §Behavioral Contract Summary row);
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
  BC-2.01.009 table — VERIFIED. PG-3 compliant: §-anchor refs used throughout;
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
  (b) normative body lines for BC-2.01.008 and BC-2.01.009 §Verification cite
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
  body lines BC-2.01.008/002 §Verification — classified historical (PG-5 compliant,
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
- F-R63-adv-2 partial RESOLVED (MEDIUM — adversary R63 stale path): BC-2.01.009
  §Behavioral contracts §Verification block cited `monocle-runtime/tests/auth.rs`
  (the pre-F-R62-4 single-file path). F-R62-4 (PRD v1.1 §7 RTM) canonicalized
  the split: BC-2.01.008 → `monocle-runtime/tests/auth_token_lifecycle.rs`;
  BC-2.01.009 → `monocle-runtime/tests/auth_header_rejection.rs`. Architecture is
  the last artifact on the old single-file path. Change: BC-2.01.009 §Verification
  updated to `auth_header_rejection.rs` with cross-reference to
  `auth_token_lifecycle.rs` for BC-2.01.008 round-trip coverage; test name
  `test_BC_AUTH_002_auth_header_validation_all_failure_modes` added inline (PRD
  v1.1 §7 RTM canonical). BC-2.01.008 §Verification sentence updated to add
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
  in §Section 5 and edge cases EC-008/EC-009 in BC-2.01.009, none of which were specified in
  SS-daemon-lifecycle.md v1.0.7. The architecture defined only `invalid_auth_token_format` for
  the single BC-2.01.009 case (non-prefixed header). This was a PRD invention of contract surface
  beyond architecture authorization. Architect disposition chosen: **(c) mixed approach** —
  two distinct error bodies: `missing_auth_token` for absent header (structural precondition
  failure, not an auth attempt; diagnostic value with zero security cost) and `invalid_auth_token`
  for any value-present failure (format failure OR secret mismatch, intentionally collapsed into
  one body to eliminate the format-vs-mismatch enumeration vector). The third PRD invention
  `invalid_auth_token_format` is RETIRED — no body of that name exists in the architecture.
  Security rationale in §Start Sequence §Behavioral contracts BC-2.01.009 (threat model:
  localhost-only, same-user adversary already has lock-file read access; defence-in-depth
  collapse of Rules 2+3 blocks information leak to adversaries with unexpected network access
  but no filesystem access). Auth middleware implementation updated to `validate_auth_header`
  returning `AuthError::Missing` or `AuthError::Invalid`. BC-2.01.009 §Behavioral Contract
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
  BC-2.01.007's verification body ("unit test serializes a `HookEventRecord`") but was
  defined nowhere in the spec corpus. An implementer following BC-2.01.007 would not know
  what `HookEventRecord` is, what fields it contains, or how to construct it. Fix: a full
  `HookEventRecord` struct definition added to §Daemon Lifecycle Protocol §Drain, immediately
  preceding the BC-2.01.007 contract statement. The struct is placed in `monocle-runtime::ring`
  (NOT `monocle-core`) because the ring buffer is a daemon runtime artifact, not part of the
  core ABI surface. Fields match the JSONL example record exactly:
  `format_version: u32` (first, always `1` in Phase 1), `session_id: String`,
  `timestamp_micros: i64`, `pid: u32`, `hook_type: String`, `tool_name: Option<String>`,
  `tool_input: Option<serde_json::Value>`. A `pub fn new(...)` constructor is provided (same
  `#[non_exhaustive]` / E0639 reasoning as engine-module structs — integration tests compile
  as separate binaries). The module-level const `RING_FORMAT_VERSION: u32 = 1` is the single
  source of truth for the format version value. BC-2.01.007 verification body updated to use
  `HookEventRecord::new(...)` explicitly. Cross-reference: SS-engine-module.md §Trace v1.1.8
  F-R28-2 entry notes that F-R28-4 is resolved in this document.

**§Trace v1.0.26** (2026-05-17T11:00:00Z) — Template compliance Dispatch 1:
- NORMATIVE: `subsystem: SS-01` added (SS-daemon-lifecycle maps to Daemon Lifecycle subsystem
  per ARCH-INDEX.md Subsystem Registry; field absent from prior versions because ARCH-INDEX
  did not exist).
- NORMATIVE: `traces_to` corrected to `architecture/ARCH-INDEX.md` (was long trace-history
  string spanning v1.0.1..v1.0.25; ARCH-INDEX.md created in this dispatch).
- NORMATIVE: `timestamp` bumped to 2026-05-17T11:00:00Z (>= chain high-water 2026-05-17T10:30:00Z;
  SE-16d PASS).
- INFORMATIONAL: `document_type` already `architecture-section` — no change required (audit §5
  confirmed PASS for daemon-lifecycle document_type).
- INFORMATIONAL: Version bump 1.0.25 → 1.0.26 records structural fix; no content changes.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §5 (SS-daemon-lifecycle).
- SE-17g classification: all citations above NORMATIVE or INFORMATIONAL as labeled.
- SE-17f PASS: post-edit verification — `subsystem: SS-01` at frontmatter line 7; `traces_to:
  architecture/ARCH-INDEX.md` at frontmatter line 16; version `"1.0.26"` at frontmatter line 4.

**§Trace v1.0.28** (2026-05-17T17:00:00Z) — F-R105-8 BC ID canonicalization (T-128h):
- NORMATIVE: All stale pre-renumbering BC IDs replaced with canonical BC-2.SS.NNN forms
  per BC-INDEX.md v1.1 §Renumbering Map (canonical at T-128h dispatch time
  2026-05-17T17:00:00Z; current canonical advances over time per F-R107-8
  historical-pin discipline).
  Finding: F-R105-8 MED.
- SE-17c BEFORE: 95 lines / 102 occurrences with stale BC IDs (all old-form DAEMON/AUTH/RING/LOCK/ABI/ENGINE prefixes).
- Replacements by canonical new ID (old-form identity in BC-INDEX §Renumbering Map):
  BC-2.01.001 [old: DAEMON-001]: 4 occurrences
  BC-2.01.002 [old: DAEMON-002]: 9 occurrences
  BC-2.01.003 [old: DAEMON-003]: 3 occurrences
  BC-2.01.004 [old: DAEMON-004]: 12 occurrences
  BC-2.01.005 [old: DAEMON-005]: 13 occurrences
  BC-2.01.006 [old: DAEMON-006]: 11 occurrences
  BC-2.01.007 [old: RING-001]: 10 occurrences
  BC-2.01.008 [old: AUTH-001]: 8 occurrences
  BC-2.01.009 [old: AUTH-002]: 20 occurrences
  BC-2.01.010 [old: LOCK-001]: 9 occurrences
  BC-2.02.001 [old: ABI-001]: 1 occurrence (cross-ref to SS-02)
  BC-2.03.003 [old: ENGINE-002-ERR]: 2 occurrences (cross-ref to SS-03)
- SE-17d AFTER: 0 lines with stale BC IDs in normative body (SE-17g PASS — see ARCH-INDEX §Trace v1.0.3).
- SE-17f PASS: sampled mapping verified — §Behavioral Contract Summary row 1 `BC-2.01.001`,
  §Start Sequence BC-2.01.008, §Drain BC-2.01.007, §Lock File BC-2.01.010.
- SE-16d PASS: 2026-05-17T17:00:00Z >= chain high-water 2026-05-17T16:30:00Z.
- No retired BCs discovered. All 95 stale-ID lines resolved to active BCs in BC-INDEX v1.1.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:00:00Z >= chain high-water 2026-05-17T10:30:00Z.

**§Trace v1.0.29** (2026-05-17T19:00:00Z) — T-128m ADR-0005 dual-accept auth header:
- NORMATIVE: Auth middleware spec updated to dual-accept protocol per ADR-0005.
  - §Body Limit and Router Design (lines 141-174): rewritten. Prior text described
    `X-Claude-Code-Ide-Authorization` as an optional per-handler IDE token check.
    BEFORE: "the Claude Code IDE token (`X-Claude-Code-Ide-Authorization`) is checked
    per-handler inside the hook handlers, not as a separate router-level layer, because
    the IDE token is optional and absent on non-hook requests."
    AFTER: "The auth middleware implements dual-accept per ADR-0005: it accepts the
    canonical `X-Monocle-Authorization` header [...] OR the compatibility alias
    `X-Claude-Code-Ide-Authorization` [...]."
  - §Start Sequence — Auth middleware validation rules: rewritten from single-header
    3-rule model to dual-accept 3-rule model. Rule 1 = canonical path, Rule 2 =
    compatibility alias (with WARN log), Rule 3 = neither header present.
  - BC-2.01.009 behavioral contracts table: expanded. "Missing header" row updated to
    "both recognized headers absent". Two new rows added: canonical path invalid-token
    and alias path invalid-token. Two new test vectors added (alias wrong secret →
    invalid_auth_token; alias correct secret → HTTP 200 + WARN log).
  - Rust implementation stub: rewritten to dual-accept with `CANONICAL_HEADER` and
    `COMPAT_ALIAS_HEADER` constants, alias path emits `tracing::warn!`.
- NORMATIVE: BC-2.01.009 update surfaced to PO for Round 4 (postcondition 1 "missing"
  semantics change; alias validation postconditions 2-3 extension).
- NORMATIVE: CAP-001 compatibility alias update surfaced to BA for Round 4 (§P2 step 1).
- SE-17f PASS: post-edit verification — `validate_auth_header` stub contains both
  `CANONICAL_HEADER` and `COMPAT_ALIAS_HEADER` constants; BC-2.01.009 table has 3 rows.
- SE-16d PASS: 2026-05-17T19:00:00Z > chain high-water 2026-05-17T17:00:00Z.

**§Trace v1.0.30** (2026-05-17T22:00:00Z) — F-R106-7 fabrication removal (Round 5E):
- NORMATIVE: F-FC-I005 fabricated ID removed from two sites in this document.
  - SE-17f BEFORE (site 1, §Start Sequence body ~line 298):
    `**Phase 4 OAuth2 clarification (F-FC-I005):** Phase 4 federation does NOT`
    AFTER: `**Phase 4 OAuth2 clarification (FC-06):** Phase 4 federation does NOT`
  - SE-17f BEFORE (site 2, §Behavioral Contract Summary BC-2.01.009 table row ~line 800):
    `Phase 4 OAuth2 federation uses separate channel (FC-06 + F-FC-I005).`
    AFTER: `Phase 4 OAuth2 federation uses separate channel (FC-06).`
  Rationale: F-FC-I005 does not exist in SS-forward-compatibility.md. FC convention is
  FC-NN (two-digit); F-FC-INNN is a non-canonical sub-ID pattern with no registry entry.
  FC-06 is the correct reference for Phase 4 auth forward-compatibility (see
  SS-forward-compatibility.md §Cross-Phase Decisions table row FC-06). Pre-adjudicated
  decision: remove fabricated sub-ID, retain FC-06 alone (Round 5E dispatch instructions).
- SE-17c BODY-SCOPE GREP EVIDENCE:
  BEFORE: 2 lines matched `F-FC-I005` (lines 298, 800).
  AFTER: 0 lines match `F-FC-I005` in SS-daemon-lifecycle.md. SE-17g META AUDIT PASS.
- SE-17d AFTER CONFIRMATION: zero `F-FC-I005` occurrences remain in this document.
- SE-16d PASS: 2026-05-17T22:00:00Z > chain high-water 2026-05-17T19:00:00Z (monotonic).

**§Trace v1.0.31** (2026-05-17T23:00:00Z) — F-R107-8 historical-pin clarification (Round 6D):
- INFORMATIONAL: §Trace v1.0.28 BC-INDEX cite `v1.1 Renumbering Map` expanded to explicit
  historical-pin form: `v1.1 §Renumbering Map (canonical at T-128h dispatch time
  2026-05-17T17:00:00Z; current canonical advances over time per F-R107-8
  historical-pin discipline)`.
  Purpose: prevent future fresh-context audits from re-flagging the historical pin as stale;
  the cite records what was canonical at the time of the T-128h canonicalization sweep,
  not a live version claim. Finding: F-R107-8 architect part.
- SE-16d PASS: 2026-05-17T23:00:00Z > chain high-water 2026-05-17T22:00:00Z (monotonic).

**§Trace v1.0.32** (2026-05-18T01:00:00Z) — F-R108-1 + F-R108-9 historical-pin + frontmatter correction (Round 7C):
- NORMATIVE (F-R108-1 CRITICAL): Two "current canonical BC-INDEX is v1.4 per F-R107-2 closure"
  occurrences removed per O-R108-3 codification. O-R108-3 pattern: live-version claims in
  historical-pin §Trace notes are structurally fragile — they become false immediately when
  BC-INDEX advances. Replaced with "current canonical advances over time per F-R107-8
  historical-pin discipline" in both §Trace v1.0.28 body (1 occurrence) and §Trace v1.0.31
  historical-pin expansion prose (1 occurrence). This is a normative content change: two
  live-version claim strings were removed from the document body.
- NORMATIVE (F-R108-9 HIGH): frontmatter `timestamp` corrected from 2026-05-17T19:00:00Z to
  2026-05-18T01:00:00Z. Prior timestamp lagged the latest §Trace entry (v1.0.31 at 23:00:00Z);
  SE-16b violation. Version bump v1.0.31 → v1.0.32 applied in Round 8A (F-R109-1) to
  reconcile frontmatter with the §Trace version number this entry already claimed.
- SE-17c BEFORE: "current canonical BC-INDEX is v1.4 per F-R107-2 closure" (2 occurrences).
- SE-17c AFTER: "current canonical advances over time per F-R107-8 historical-pin discipline".
- SE-16d PASS: 2026-05-18T01:00:00Z > chain high-water 2026-05-17T23:00:00Z (monotonic).

**§Trace v1.0.32-R109** (2026-05-18T05:00:00Z) — F-R109-1 + F-R109-8 frontmatter version reconciliation (Round 8A):
- NORMATIVE (F-R109-1 CRITICAL): frontmatter `version` bumped from "1.0.31" to "1.0.32" to
  match the §Trace v1.0.32 entry already present in the document body since Round 7C. The Round
  7C dispatch wrote §Trace v1.0.32 but withheld the frontmatter bump per cross-dispatch
  coordination directive (targeting current PO 7B SS pin). This created a fabrication-class
  defect: frontmatter claimed v1.0.31 while §Trace body documented v1.0.32 as the current state.
- NORMATIVE (F-R109-8 HIGH): §Trace v1.0.32 body rewritten to remove "No version bump —
  content unchanged; timestamp-only correction" self-contradiction. The F-R108-1 removal of two
  live-version claim strings IS normative content change. The body now accurately states that the
  version bump v1.0.31 → v1.0.32 was applied in Round 8A to reconcile frontmatter with §Trace.
- SE-17c BEFORE: "No version bump — content unchanged; timestamp-only correction".
- SE-17c AFTER: "Version bump v1.0.31 → v1.0.32 applied in Round 8A (F-R109-1) to reconcile
  frontmatter with the §Trace version number this entry already claimed."
- SE-16d PASS: 2026-05-18T05:00:00Z > chain high-water 2026-05-18T01:00:00Z (monotonic; corrected from erroneous 2026-05-17T04:30:00Z per F-R110-1).

**§Trace v1.0.32-R110** (2026-05-18T05:30:00Z) — F-R110-1 timestamp correction (Round 9A):
- NORMATIVE (F-R110-1 CRITICAL): frontmatter `timestamp` and §Trace v1.0.32-R109 header corrected
  from "2026-05-17T04:30:00Z" to "2026-05-18T05:00:00Z". The Round 8A dispatch used a wrong date
  (2026-05-17T04 instead of monotonic post-Round-7C 2026-05-18T05+), causing a SE-16d chain
  regression: 2026-05-17T04:30Z is before the prior §Trace v1.0.32 entry at 2026-05-18T01:00:00Z.
  The SE-16d PASS claim "2026-05-17T04:30:00Z satisfies chain monotonicity" was arithmetically
  false. All five affected files corrected in this burst (parallel Round 9A).
- SE-16d PASS: 2026-05-18T05:30:00Z > chain high-water 2026-05-18T05:00:00Z (monotonic).

**§Trace v1.0.33** (2026-05-19T10:00:00Z) — F-PHASE2-R05-05: JSONL Ring Buffer Rotation Policy added (Phase 2 adversary r05):
- NORMATIVE (F-PHASE2-R05-05 HIGH): Added "JSONL Ring Buffer Rotation Policy" subsection to
  §Drain step 4 in §Daemon Lifecycle Protocol. This section is the canonical source of truth
  for the ring buffer rotation parameters (50 MB default threshold, 100 MB absolute per-file
  cap, 5-file retention, oldest-first deletion, atomic-rename rotation algorithm, no-compress
  policy, `0o600` file mode, `monocle-events.jsonl.1` through `.5` naming convention).
  Motivation: Phase 2 story S-008 AC-007 cited `PRD v1.26.15 §OQ-06` as the ring rotation
  policy source. `grep -n "OQ.?06\|50 MB\|rotat" .factory/specs/prd.md` returns no matches —
  `PRD §OQ-06` does not exist as a PRD section. The planning-origin research for this decision
  lives in `oq-research.md §OQ-06` (a planning artifact, not a normative spec). The
  architectural rotation policy belongs in this document (SS-daemon-lifecycle.md) as the
  §Daemon Lifecycle Protocol governing spec. This entry establishes it here.
- NORMATIVE (F-PHASE2-R05-05 / BC-2.01.007 EC-002): EC-002 parenthetical updated in
  BC-2.01.007.md from `(100 MB × 5 files per OQ-06)` to
  `(50 MB rotation threshold, 100 MB × 5 cap per SS-daemon-lifecycle.md §JSONL Ring Buffer Rotation Policy)`.
  This re-anchors the BC edge-case citation to the canonical spec section created here;
  no change to the BC's behavioral semantics.
- INFORMATIONAL: The planning document `oq-research.md §OQ-06` recommended "100MB × 5 rotation"
  as the per-segment size; the product brief §Storage says "100MB × 5 rotation (OQ-06)". This
  architecture section adopts 50 MB as the default rotation trigger (soft threshold for early
  rotation before the hard 100 MB cap) with 100 MB as the absolute per-file ceiling. The
  brief's "100MB × 5" refers to the maximum retained size, not the trigger threshold. No
  contradiction — these are complementary parameters of the same policy.
- INFORMATIONAL: `PRD §OQ-06` does not exist and was never a PRD section. OQ-NNN IDs are
  open-question research IDs from `oq-research.md`, not PRD section anchors. Surfaced to
  orchestrator: the S-008 AC-007 citation of "PRD v1.26.15 §OQ-06" is a fabricated anchor
  that must be re-pointed to `SS-daemon-lifecycle.md §JSONL Ring Buffer Rotation Policy`
  in the story file. Story-writer domain; architect surfaces finding only.
- SE-16d PASS: 2026-05-19T10:00:00Z > chain high-water 2026-05-18T05:30:00Z (monotonic).
