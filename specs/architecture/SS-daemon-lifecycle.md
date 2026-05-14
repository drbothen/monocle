---
document_type: architecture-section
level: L3
section: "daemon-lifecycle"
version: "1.0.11"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-14T23:30:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/prd.md
  - /Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md
input-hash: "[live-state]"
traces_to: "adversary F-NEW-05 F-NEW-06 F-NEW-07 F-NEW-09; brief v1.4.2 Phase 1 Runtime Core scope; BC-HOOK-022 timeout matrix; BC-HOOK-024 lock-file collision context; FC-01 + FC-06 from forward-compat scan 9618502; pre-Phase-1 lock-in per human authorization; v1.0.5 round-29 fix F-R28-4 HookEventRecord struct definition + constructor in monocle-runtime::ring; v1.0.6 round-30 fix F-R30-2 HookEventRecord #[non_exhaustive] attribute added; v1.0.7 round-53.1 fix F-R53-adv-1 §Analysis mis-anchor corrected to §Item P3-1 in §Trace v1.0.6 rationale sentence; v1.0.8 round-F-R62 fix F-R62-8 BC-AUTH-002 expanded to three failure modes (missing header / invalid token) — disposition (c); v1.0.9 F-R62-4 back-propagation closure (adversary R63 F-R63-adv-2 + consistency R2 F-R63-cons-3): §BC Summary footer updated past-tense + authority split (PRD v1.1 f855835); BC-AUTH-002 §Verification single-file path split to auth_header_rejection.rs; BC-AUTH-001 §Verification file path added (auth_token_lifecycle.rs); v1.0.10 consistency R3 R3-001 closure (commit ba62a15): §BC Summary footer rephrased to version-stable (oscillation prevention per L-F-R63-PARTIAL-FIX); v1.0.11 adversary R65 F-R65-1/2/3 closure: Three→Two count correction at 2 sites + Bearer disposition fix (missing_auth_token)"
project: monocle
---

# Architecture: Daemon Lifecycle

## [Section Content]

## Scope

Phase 1 daemon: a single-process Rust binary (`monocle daemon start`) running an
axum 0.8 HTTP server over `127.0.0.1:<OS-assigned-port>` for hook ingestion,
plus a Unix domain socket (UDS) at `<runtime_dir>/monocle.sock` for TUI client
attach/detach commands. Runtime directory is resolved via
`directories::ProjectDirs::runtime_dir()` per OQ-10. All lifecycle state
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
  "hook_endpoints": ["/hooks/pre-tool-use", "/hooks/notification", "..."],
  "ring_buffer_fill_pct": <0.0-100.0>,
  "channel_saturation_pct": <0.0-100.0>,
  "last_hook_ts": {
    "pre_tool_use": "<ISO8601 or null>",
    "notification": "<ISO8601 or null>",
    "stop": "<ISO8601 or null>",
    "session_start": "<ISO8601 or null>",
    "prompt_submit": "<ISO8601 or null>"
  },
  "tui_attached": <bool>
}
```

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

1. Resolve `runtime_dir` via `directories::ProjectDirs::runtime_dir()` (OQ-10).
   Create directory with mode `0o700` if absent.
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
     "startTimeUtc": "<ISO8601>",
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
   `<runtime_dir>/monocle-events.jsonl` (append mode, `tempfile::persist` for
   the current-segment file).

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
     "shutdown_utc": "<ISO8601>"
   }
   ```

### Hard Shutdown

6. After 10-second drain timeout OR on receipt of a second SIGTERM:
   a. Force-close all axum connections. In axum 0.8, `axum::Server` was removed;
      the correct idiom is `axum::serve(listener, app).with_graceful_shutdown(shutdown_rx)`
      where `shutdown_rx` is a `tokio::sync::oneshot::Receiver<()>` sent by the
      signal handler. On hard shutdown (second SIGTERM or drain timeout expiry), drop
      the sender half to unblock the receiver and trigger immediate connection close.
      Signal handling uses `tokio::signal::unix::signal(SignalKind::terminate())` for
      SIGTERM and `tokio::signal::ctrl_c()` for SIGINT; both are `async fn` futures
      awaited in a `tokio::select!` loop alongside the oneshot receiver.
   b. Close UDS socket; remove `<runtime_dir>/monocle.sock`.
   c. Remove `<runtime_dir>/monocle.lock`.
   d. Exit.

Exit codes:
- `0`: drain succeeded; all in-flight requests completed; ring buffer flushed.
- `130`: second SIGTERM received during drain; hard-killed; some in-flight
  requests may have been dropped.

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
| BC-DAEMON-004 | Graceful shutdown: 10-second drain, ring buffer flush, recovery checkpoint | Daemon Lifecycle Protocol |
| BC-DAEMON-005 | Lock file created atomically via `tempfile::persist`; pid-liveness checked on startup; removed on clean shutdown | Daemon Lifecycle Protocol |
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
but the `"app"`, `"pid"`, `"port"`, `"authToken"`, `"startTimeUtc"`, `"version"`
fields are stable across Phase 1 → Phase 4.

---

## §Trace

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
