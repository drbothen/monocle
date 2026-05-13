---
document_type: architecture-section
level: L3
section: "daemon-lifecycle"
version: "1.0.6"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-13T22:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md
input-hash: "[live-state]"
traces_to: "adversary F-NEW-05 F-NEW-06 F-NEW-07 F-NEW-09; brief v1.4.2 Phase 1 Runtime Core scope; BC-HOOK-022 timeout matrix; BC-HOOK-024 lock-file collision context; FC-01 + FC-06 from forward-compat scan 9618502; pre-Phase-1 lock-in per human authorization; v1.0.5 round-29 fix F-R28-4 HookEventRecord struct definition + constructor in monocle-runtime::ring; v1.0.6 round-30 fix F-R30-2 HookEventRecord #[non_exhaustive] attribute added"
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

   Validation rule in Phase 1: any `X-Monocle-Authorization` value that does
   NOT begin with `monocle-v1:` is rejected immediately with HTTP 401
   `{"error":"invalid_auth_token_format"}` before any secret comparison occurs.
   This prevents timing-oracle attacks where an attacker probes whether a
   non-prefixed string matches the secret.

   Auth middleware implementation:

   ```rust
   const TOKEN_PREFIX: &str = "monocle-v1:";

   fn validate_auth_token(presented: &str, expected_secret: &str) -> bool {
       let Some(hex_part) = presented.strip_prefix(TOKEN_PREFIX) else {
           return false; // Rejected before any secret comparison.
       };
       // Constant-time comparison to prevent timing oracle on the hex secret.
       constant_time_eq::constant_time_eq(hex_part.as_bytes(), expected_secret.as_bytes())
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
     part. Verification: integration test reads the lock file after daemon start
     and asserts `authToken` matches `/^[0-9a-f]{64}$/`; presents
     `monocle-v1:<authToken>` to `/status` and asserts HTTP 200.

   - **BC-AUTH-002:** Any `X-Monocle-Authorization` value not beginning with
     `monocle-v1:` receives HTTP 401 `{"error":"invalid_auth_token_format"}`.
     Phase 4 OAuth2 federation tokens use `Authorization: Bearer` on a separate
     federation channel and are NOT valid on Phase 1 HTTP endpoints.
     Verification: integration test sends `Authorization: Bearer fake`,
     `X-Monocle-Authorization: baretoken`, and
     `X-Monocle-Authorization: monocle-v2:abc` and asserts all receive HTTP 401.

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
| BC-AUTH-002 | Any `X-Monocle-Authorization` value not beginning `monocle-v1:` receives HTTP 401 `{"error":"invalid_auth_token_format"}`; Phase 4 OAuth2 federation uses separate channel not this header (FC-06 + F-FC-I005) | Daemon Lifecycle Protocol §Start Sequence |
| BC-LOCK-001 | Lock-file JSON includes `contract_version: 1` as the first key; readers must check this field before consuming other fields; unrecognized version triggers graceful skip with warning (F-FC-O001) | Daemon Lifecycle Protocol §Start Sequence |

The Phase 1 PRD will formalize these as full BC entries with postconditions,
evidence, and verification harness stubs. This artifact pre-stages them for
the Phase 1 architecture gate.

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
  production-grade forward-compat mechanism per SS-forward-compatibility.md §Analysis.
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
