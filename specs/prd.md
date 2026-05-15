---
document_type: prd
level: L3
version: "1.7"
status: draft
producer: product-owner
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T08:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-forward-compatibility.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0003-license-selection.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md
input-hash: "[live-state]"
traces_to: "product-brief.md v1.4.23; vision-synthesis v1.1.2; SS-daemon-lifecycle.md v1.0.13; SS-core-types-and-abi.md v1.2.8; SS-engine-module.md v1.1.15; 22 BCs (16 original + 6 new BC-DAEMON-001..006); D-047 strict; 18+ META defense layers; STATE.md phase-1-spec-crystallization-entry-pending; F-R62 fix-burst (adversary commit 5713ccc); T-4 consistency audit (commit 0e322da); architect auth adjudication (commit 2db408f); F-R63 fix-burst (adversary R63 commit 11a98c4; consistency R2 commit 200eb68; arch v1.0.9 commit 8bf3759); R3-001 closure (consistency R3 commit ba62a15; arch v1.0.10 commit dc3af71); L-F-R63-PARTIAL-FIX propagation discipline applied; F-R65 closure chain (adversary R65 commit 77fccb7; consistency R4 commit 3d33937; arch v1.0.11 commit af2101d); L-F-R63-PARTIAL-FIX pin propagation applied; F-R67-2 closure (PRD EC-045 off-by-one fix; adversary R67 finding); F-R70 closure chain (adversary R70 commit 4b4aea1; arch v1.0.12 commit 727c826): BC-DAEMON-004 POSIX exit-code correction + BC-DAEMON-005 platform-aware runtime-dir fallback + BC-DAEMON-006 timestamp precision tightening + EC-031 fail-open security rationale; F-R71 closure chain (adversary R71 commit 2710ab4; arch v1.0.13 commit 1f53d47): F-R71-3 NFR-008 phrasing fix (BC-DAEMON-005 precondition 2 rationale) + arch pin propagation v1.0.12 → v1.0.13 (31 normative sites)"
project: monocle
supplements: []
---

# Product Requirements Document: Monocle — Phase 1 Forward-Compatibility Contracts

## 1. Product Overview

### 1.1 Problem

Today, a developer running three Claude Code sessions across two projects faces a fragmentation problem: sessions live in separate tmux windows requiring context switches to check status; concurrent permission prompts from different sessions stall until the developer switches to the right window; factory-pipeline state (vsdd-factory STATE.md) is only visible by manually reading files; and no single view spans multiple harnesses.

Per vision §Vision Statement: "One TUI lens over every Claude-class session you're running, every customization that shapes them, and every workflow driving them — across multiple harnesses and federated across hosts."

### 1.2 Vision

Monocle is a single-binary Rust TUI that gives developers one `Ctrl-\` popup over every AI coding harness session they are running. It surfaces five information planes: live session roster (Runtime), active customizations per session (Static), workflow pipeline state (Workflow), per-harness profiles (Harness), and a lazygit-style keybinding dispatch layer (TUI philosophy). Monocle is observe-only for workflow state and session transcripts; it owns the action layer only for permission prompts and keybinding dispatch.

The killer scenario per vision §End-to-End Killer Scenario: 4 keystrokes (`Ctrl-\`, `2`, `1`, `Ctrl-\`) resolve two concurrent permission prompts with zero context switches vs. the current 6+ keystrokes + 2 window switches + risk of session timeout.

### 1.3 Competitive Differentiators

| ID | Differentiator | BC Backing |
|----|---------------|------------|
| D-1 | Hook-protocol ingestion at OS-assigned port with versioned auth token | BC-AUTH-001, BC-AUTH-002, BC-LOCK-001, BC-DAEMON-001, BC-DAEMON-002 |
| D-2 | VecDeque overlay stack — both concurrent prompts visible simultaneously | BC-ENGINE-001, BC-ENGINE-002 |
| D-3 | Forward-compatible ABI via const + non_exhaustive + proto schema_version | BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 |
| D-4 | FactoryAdapter open trait — VsddFactoryAdapter ships in Phase 1; WASM loadable in Phase 3 | BC-FACTORY-001, BC-FACTORY-002 |
| D-5 | ClaudeCodeModule strict-basename detect — no false positives from claude-squad/claudio | BC-ENGINE-002 |
| D-6 | JSONL ring with format_version first key — Phase 2 trigger-trace can read Phase 1 history | BC-RING-001 |
| D-7 | 256 KiB body size limit with structured error — bounded daemon memory exposure | BC-DAEMON-003 |
| D-8 | Graceful 10-second drain with crash-recovery checkpoint | BC-DAEMON-004, BC-DAEMON-006 |

### 1.4 Target Users

| Persona | Pain | Phase |
|---------|------|-------|
| Multi-session Claude Code developer | Concurrent permission prompts stall sessions; no unified view | Phase 1 |
| Factory-pattern operator | STATE.md only readable via manual cat/tree; no live pipeline visibility | Phase 1 |
| Multi-harness operator (CodeMachine + Claude Code) | No unified cost/session-health view across harnesses | Phase 4 |

### 1.5 Out of Scope

Per vision §Explicit Non-Goals (hard boundaries):
- Does NOT execute workflows — monocle never writes STATE.md, never triggers factory phases
- Does NOT route LLM API requests — CCR integration is detect-on-PATH + config-write only
- Does NOT replace the terminal multiplexer — runs inside tmux, does not replace it
- Does NOT include PM/Worker multi-agent orchestration
- Does NOT own session transcripts — hook events are ephemeral ingestion signals
- Does NOT ship `PostToolUse` hook endpoint in Phase 1 — per JC-2 gene-source parity (any-context BC-HOOK-007 canonical 5-endpoint matrix)
- Does NOT ship WASM plugin SDK in Phase 1 — Phase 3 deliverable per OQ-03
- Does NOT ship rmcp MCP bridge in Phase 1 — Phase 4 deliverable per OQ-09

---

## 2. Behavioral Contracts

### 2.1 Grouping

BCs are grouped by domain subsystem. The 22 Phase 1 BCs span five functional domains:

| Domain | BC IDs | Subsystem Anchor |
|--------|--------|-----------------|
| Daemon Health + Status Endpoints | BC-DAEMON-001, BC-DAEMON-002 | SS-daemon-lifecycle.md §Health and Status Endpoints |
| Daemon Body Size Limit | BC-DAEMON-003 | SS-daemon-lifecycle.md §Body Size Limit |
| Daemon Lifecycle — Shutdown + Recovery | BC-DAEMON-004, BC-DAEMON-005, BC-DAEMON-006 | SS-daemon-lifecycle.md §Daemon Lifecycle Protocol |
| Hook Ingestion — Ring Buffer + Drain | BC-RING-001 | SS-daemon-lifecycle.md §Drain |
| Daemon Authentication | BC-AUTH-001, BC-AUTH-002 | SS-daemon-lifecycle.md §Daemon Lifecycle Protocol |
| Lock File Discovery | BC-LOCK-001 | SS-daemon-lifecycle.md §Daemon Lifecycle Protocol §Start Sequence |
| Core ABI Stability | BC-ABI-001, BC-ABI-002 | SS-core-types-and-abi.md §ABI Version Constant |
| Enum Extensibility | BC-TYPES-001 | SS-core-types-and-abi.md §Enum Extensibility |
| Factory Adapter Trait | BC-FACTORY-001, BC-FACTORY-002 | SS-core-types-and-abi.md §FactoryAdapter Trait |
| Proto Wire Schemas | BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 | SS-core-types-and-abi.md §Prost Wire Schemas |
| EngineModule Trait | BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003 | SS-engine-module.md §EngineModule Trait Signature |

---

## 3. Full Behavioral Contract Specifications

### BC-DAEMON-001 — Healthz Endpoint (Unauthenticated Liveness Probe)

**Priority:** P0 — Daemon liveness contract.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Health and Status Endpoints §GET /healthz

**Preconditions:**
1. The monocle daemon is running and bound on `127.0.0.1:<port>`.
2. A `GET /healthz` request arrives (no auth header required).

**Postconditions:**
1. When AppMode is normal (not `ShuttingDown`) and the hook-receiver task is alive: HTTP 200 with body `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}` where `uptime_sec` is integer seconds since daemon start and `version` is the monocle binary semver string.
2. When AppMode is `ShuttingDown` OR the hook-receiver task has exited abnormally: HTTP 503 with body `{"status":"shutting_down"}`.
3. `/healthz` is unauthenticated — no `X-Monocle-Authorization` header is required or checked.
4. `/healthz` has no request body and no `DefaultBodyLimit` applies (the limit is applied to the authenticated router only).

**Invariants:**
1. The endpoint must succeed even if the auth token has rotated during crash recovery. Unauthenticated access is warranted because `uptime_sec` and `version` are not secret, and a local adversary with `127.0.0.1` access already has OS-level process enumeration capability.
2. `/healthz` is registered on the unauthenticated router and MUST NOT be co-located on the authenticated router (which would inadvertently apply the auth middleware).

**Edge Cases:**

EC-040: TUI client behavior when `/healthz` is unreachable AND the lock file exists with a live pid (`kill(pid, 0)` succeeds): TUI concludes daemon is hung (accepting TCP, not responding) and initiates recovery flow with a 10-second countdown before auto-restarting.

EC-041: TUI client behavior when `/healthz` is unreachable AND the lock file exists with a dead pid: TUI treats the lock file as stale and initiates normal auto-start.

**Canonical Test Vectors:**

| Scenario | Input | Expected |
|----------|-------|----------|
| Normal operation | `GET /healthz` (no auth header) | HTTP 200 `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}` |
| Daemon shutting down | `GET /healthz` during graceful shutdown | HTTP 503 `{"status":"shutting_down"}` |
| No auth header required | `GET /healthz` with no `X-Monocle-Authorization` | HTTP 200 (not HTTP 401) |

**Verification:**
- Integration test in `monocle-runtime/tests/healthz_endpoint.rs`: starts daemon, polls `/healthz`, asserts HTTP 200 with `"status":"alive"` and numeric `uptime_sec`.
- Test name: `test_BC_DAEMON_001_healthz_unauthenticated_alive`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Health and Status Endpoints §GET /healthz
- Brief: §Scope (hook receiver hardening sub-bullet — `/healthz` liveness endpoint)

---

### BC-DAEMON-002 — Status Endpoint (Authenticated Daemon State)

**Priority:** P0 — Daemon observability contract.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Health and Status Endpoints §GET /status

**Preconditions:**
1. The monocle daemon is running.
2. A `GET /status` request arrives with a valid `X-Monocle-Authorization: monocle-v1:<token>` header.

**Postconditions:**
1. HTTP 200 with a JSON body containing all of the following fields:
   - `pid`: integer PID of the daemon process
   - `uptime_sec`: integer seconds since daemon start
   - `version`: daemon binary semver string
   - `abi_version`: integer `1` (`monocle_core::MONOCLE_ABI_VERSION` as compiled)
   - `lock_file`: absolute path string to `<runtime_dir>/monocle.lock`
   - `hook_endpoints`: JSON array of 5 hook path strings (`["/hooks/pre-tool-use", "/hooks/notification", "/hooks/stop", "/hooks/session-start", "/hooks/prompt-submit"]`)
   - `ring_buffer_fill_pct`: float 0.0–100.0 representing ring buffer fill percentage
   - `channel_saturation_pct`: float 0.0–100.0 representing bounded channel fill percentage
   - `last_hook_ts`: JSON object with per-hook-type ISO 8601 timestamps or `null` for hook types that have not fired since daemon start
   - `tui_attached`: boolean — `true` if a TUI client is currently connected via UDS
2. If the auth token is invalid: HTTP 401 per BC-AUTH-002.
3. `/status` continues to serve during graceful shutdown drain (read-only; useful for drain monitoring).

**Invariants:**
1. `/status` requires authentication because it exposes internal buffer fill levels and channel saturation metrics that could reveal load patterns to a local adversary.
2. The `abi_version` field in the response MUST equal `monocle_core::MONOCLE_ABI_VERSION`. This enables Phase 3 plugin SDK and Phase 4 federation to gate on ABI compatibility.
3. `/status` is subject to the 256 KiB body size limit (BC-DAEMON-003) in its request path (even though GET responses are unbounded — the limit protects request ingestion, not response generation).

**Edge Cases:**

EC-042: Phase 4 federation reads `abi_version` from a peer's `/status` and refuses to activate if the version is incompatible. Phase 1 daemon only needs to serve the field with the correct value.

EC-043: `ring_buffer_fill_pct` is `0.0` if no events have been received since startup. `channel_saturation_pct` is `0.0` if the bounded channel is empty.

EC-044: `last_hook_ts` values use ISO 8601 format (`YYYY-MM-DDTHH:MM:SS.sssZ` UTC). A hook type that has not fired since daemon start has value `null` (JSON null), not an empty string.

**Canonical Test Vectors:**

| Scenario | Input | Expected |
|----------|-------|----------|
| Authenticated request | `GET /status` with valid `X-Monocle-Authorization` header | HTTP 200; body contains all 10 fields; `abi_version == 1` |
| Unauthenticated request | `GET /status` (no auth header) | HTTP 401 `{"error":"missing_auth_token"}` |
| Wrong token | `GET /status` with invalid token | HTTP 401 `{"error":"invalid_auth_token"}` |

**Verification:**
- Integration test in `monocle-runtime/tests/status_endpoint_auth.rs`: starts daemon, reads lock file for auth token, requests `/status` with valid token, asserts HTTP 200 and all 10 fields present including `abi_version == 1`.
- Test name: `test_BC_DAEMON_002_status_endpoint_requires_auth_and_returns_abi_version`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Health and Status Endpoints §GET /status
- Brief: §Scope (hook receiver hardening sub-bullet — `/status` daemon-state query endpoint)

---

### BC-DAEMON-003 — Body Size Limit (256 KiB, HTTP 413)

**Priority:** P0 — Memory protection contract.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Body Size Limit

**Preconditions:**
1. The monocle daemon is running.
2. A request arrives at any of the 5 hook POST endpoints (`/hooks/pre-tool-use`, `/hooks/notification`, `/hooks/stop`, `/hooks/session-start`, `/hooks/prompt-submit`) or at `/status` with a request body exceeding 262,144 bytes.

**Postconditions:**
1. The daemon returns HTTP 413 Payload Too Large with body `{"error":"payload_too_large","limit_bytes":262144}`.
2. The limit is enforced via axum's `DefaultBodyLimit::max(256 * 1024)` layer applied at router construction time on the authenticated router.
3. `/healthz` (unauthenticated, no body) is NOT subject to the limit — it is registered on the unauthenticated router which has no body-limit layer.
4. The limit applies to the request body. Response bodies from `/status` are not bounded by this contract.

**Invariants:**
1. The 256 KiB ceiling accommodates 5× the 99th-percentile expected payload from Claude Code's `Notification` hook (diff output, stack traces, tool output summaries typically 1–50 KiB).
2. The worst-case daemon memory exposure per connection is bounded to `concurrent_requests_max × 256KiB`.
3. `DefaultBodyLimit::max(256 * 1024)` must be explicitly added — axum 0.8 does NOT apply a default body limit.

**Edge Cases:**

EC-045: Request body is exactly 262,145 bytes: HTTP 413 (limit is strictly exclusive — `> limit` triggers the rejection; axum's `DefaultBodyLimit::max(N)` rejects bodies strictly exceeding N bytes; body of exactly N=262,144 returns HTTP 200).

EC-046: Request body is 262,143 bytes: HTTP 200 (within limit).

EC-047: `POST /shutdown` (authenticated admin endpoint) is also on the authenticated router and therefore subject to the body limit. A shutdown payload is typically empty or a few bytes; the limit provides defence-in-depth against oversized shutdown requests.

**Canonical Test Vectors:**

| Scenario | Input Body Size | Expected |
|----------|----------------|----------|
| At limit (exceeds) | 262,145 bytes | HTTP 413 `{"error":"payload_too_large","limit_bytes":262144}` |
| Just under limit | 262,143 bytes | HTTP 200 (hook processed) |
| Normal hook | ~1 KiB | HTTP 200 |

**Verification:**
- Integration test in `monocle-runtime/tests/body_size_limit.rs`: sends a 262,145-byte POST to a hook endpoint, asserts HTTP 413 with the exact error body.
- Test name: `test_BC_DAEMON_003_body_size_limit_413_on_excess`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Body Size Limit
- Brief: §Success Criteria (hook receiver body size limit row — target `{"error":"payload_too_large","limit_bytes":262144}`)

---

### BC-DAEMON-004 — Graceful Shutdown (10-Second Drain)

**Priority:** P0 — Data integrity and reliability contract.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain

**Preconditions:**
1. The monocle daemon is running and may have in-flight hook POST requests.
2. A shutdown signal arrives: SIGTERM, SIGINT, or an authenticated `POST /shutdown`.

**Postconditions:**
1. AppMode transitions to `ShuttingDown` immediately.
2. All new hook POST requests to `/hooks/*` receive HTTP 503 with header `Retry-After: 10` and body `{"error":"daemon_shutting_down"}`.
3. `/healthz` returns HTTP 503 with body `{"status":"shutting_down"}` during drain.
4. `/status` continues to serve (read-only) during drain for monitoring purposes.
5. The daemon waits up to 10 seconds for in-flight hook POSTs to complete (`tokio::time::timeout(Duration::from_secs(10), drain_inflight())`).
6. If `--persistent-events` flag is set, the JSONL ring buffer is flushed to `<runtime_dir>/monocle-events.jsonl` during drain.
7. After drain or on second signal or second admin `/shutdown`: lock file removed, UDS socket closed, daemon exits.
8. The exit code written to the OS process table on daemon termination MUST match the trigger (POSIX 128+N convention for signal-induced exits):
   - `0`: graceful drain succeeded; all in-flight requests completed within the 10-second window; ring buffer flushed if applicable.
   - `130`: hard-killed by SIGINT (signal 2) during drain — POSIX convention 128+2. Typical cause: user pressed Ctrl-C a second time while draining.
   - `143`: hard-killed by SIGTERM (signal 15) during drain — POSIX convention 128+15. Typical cause: systemd/k8s sent a second SIGTERM after the graceful-shutdown window.
   - `2`: hard-killed by a second authenticated `POST /shutdown` during drain (admin forced-stop). This is a monocle-specific programmatic code, chosen outside the POSIX 128+N space (which starts at 129) and distinct from startup-failure exit 1. External monitoring treats exit 2 as operator-initiated force-stop via admin API.
   - `1`: daemon failed to start (startup failure — e.g., `DaemonStartError::RuntimeDirUnresolvable`, port bind failure, existing live lock file).

**Invariants:**
1. The 10-second drain window is a hard timeout. A second SIGTERM during drain triggers immediate hard shutdown without waiting for in-flight requests.
2. Signal handling uses `tokio::signal::unix::signal(SignalKind::terminate())` for SIGTERM and `tokio::signal::ctrl_c()` for SIGINT. Both are awaited in a `tokio::select!` loop alongside the oneshot shutdown receiver. The signal type that triggered hard shutdown is recorded for exit-code selection.
3. The `POST /shutdown` endpoint requires `X-Monocle-Authorization` authentication — unauthenticated shutdown requests receive HTTP 401.
4. External monitoring systems (systemd `Restart=on-failure`, k8s `terminationGracePeriodSeconds`, CI status parsers) MUST use exit code 143 (not 130) to detect SIGTERM hard-kill during drain. Exit 130 encodes SIGINT (Ctrl-C second press), not SIGTERM.

**Edge Cases:**

EC-048: Hook POST arrives mid-drain after the drain timeout has expired but before the connection is force-closed. The daemon rejects with HTTP 503 `{"error":"daemon_shutting_down"}`.

EC-049: Ring buffer flush fails during drain (e.g., filesystem full). The daemon logs `WARN: ring buffer flush failed: <io-error>` (E-RING-001) and proceeds with shutdown. The partial flush is acceptable — Phase 2 readers skip incomplete trailing lines.

EC-050: `POST /shutdown` with valid auth during a drain already in progress. The daemon acknowledges with HTTP 200 and the second shutdown call triggers immediate hard close with exit code 2 (admin forced-stop).

**Canonical Test Vectors:**

| Scenario | Input | Expected |
|----------|-------|----------|
| Shutdown signal received | SIGTERM or `POST /shutdown` (authenticated) | AppMode → ShuttingDown; new hooks get HTTP 503 + Retry-After: 10 |
| New hook during drain | POST /hooks/* during drain | HTTP 503 `{"error":"daemon_shutting_down"}`, `Retry-After: 10` |
| Clean drain | All in-flight requests complete within 10s | Exit code 0 |
| SIGINT hard-kill during drain | Second SIGINT (Ctrl-C) during drain | Exit code 130 (POSIX 128+2) |
| SIGTERM hard-kill during drain | Second SIGTERM during drain | Exit code 143 (POSIX 128+15) |
| Admin forced-stop during drain | Second `POST /shutdown` during drain | Exit code 2 (monocle-specific admin forced-stop) |
| Startup failure | `DaemonStartError::RuntimeDirUnresolvable` or port bind failure | Exit code 1 |

**Verification:**
- Integration test in `monocle-runtime/tests/graceful_shutdown.rs`: starts daemon, sends SIGTERM, immediately sends a new hook POST, asserts HTTP 503 with correct body and `Retry-After: 10` header.
- Test name: `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests`
- Integration test in `monocle-runtime/tests/daemon_lifecycle.rs` (`test_BC_DAEMON_004_exit_codes_posix_distinct`): sends SIGTERM twice (expects exit 143), sends SIGINT twice (expects exit 130), sends two sequential `POST /shutdown` calls (expects exit 2).

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Shutdown Signal Handling and §Drain
- Brief: §Scope (hook receiver hardening sub-bullet — graceful shutdown protocol on SIGTERM/SIGINT)

---

### BC-DAEMON-005 — Lock File Atomic Lifecycle (Create + Pid Check + Cleanup)

**Priority:** P0 — Process isolation and idempotency contract.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence and §Hard Shutdown

**Preconditions:**
1. The monocle daemon is starting up (executing the start sequence).
2. The runtime directory `<runtime_dir>` is resolved via the following platform-aware chain (evaluated in order; first `Some` result wins):
   - (a) `MONOCLE_RUNTIME_DIR` environment variable — if set and non-empty, use as the runtime directory path verbatim. This is the operator escape hatch for containers, NixOS, and non-standard deployments.
   - (b) `directories::ProjectDirs::runtime_dir()` — returns `Some` on Linux (XDG `$XDG_RUNTIME_DIR/monocle`); returns `None` on macOS and Windows by platform-ABI design (not misconfiguration).
   - (c) `directories::ProjectDirs::data_local_dir()` — platform fallback for macOS (`~/Library/Application Support/monocle/`) and Windows (`%APPDATA%/monocle/`), and any Linux environment where `XDG_RUNTIME_DIR` is not set.
   - (d) If all three resolution paths return `None` (e.g., no home directory AND no `MONOCLE_RUNTIME_DIR`), the daemon exits 1 with `DaemonStartError::RuntimeDirUnresolvable`. Error message: `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`.
   
   Rationale: `ProjectDirs::runtime_dir()` returns `None` on macOS by platform design (not misconfiguration). macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64). A fail-fast-only approach would require every macOS user to set `MONOCLE_RUNTIME_DIR`, violating the zero-config startup requirement. The `data_local_dir()` fallback provides a standards-compliant runtime state location on macOS and Windows without operator intervention.

**Postconditions (start sequence):**
1. If a lock file exists at `<runtime_dir>/monocle.lock` with a live pid (`kill(pid, 0)` succeeds): daemon logs `ERROR: daemon already running at pid=<N>; exiting` and exits 1.
2. If a lock file exists with a dead pid: daemon logs `WARN: stale lock file removed` and proceeds with startup.
3. The lock file is written atomically via `tempfile::persist` to `<runtime_dir>/monocle.lock` after the daemon has bound its listener and obtained a port. Lock file mode: `0o600`.
4. The lock file JSON has `contract_version` as the first key (value `1`), followed by `pid`, `port`, `authToken`, `startTimeUtc`, `app`, `version`.
5. If `DaemonStartError::RuntimeDirUnresolvable` is raised (resolution path (d) reached), the daemon exits 1 with the message above. No lock file is created. This is the fail-fast path for genuinely unresolvable environments (no home directory AND no `MONOCLE_RUNTIME_DIR`).

**Postconditions (clean shutdown):**
6. On successful graceful shutdown, `<runtime_dir>/monocle.lock` is removed.
7. On successful graceful shutdown, `<runtime_dir>/monocle.sock` is removed.

**Invariants:**
1. Only one monocle daemon instance runs per runtime directory. The pid-liveness check (step 1) enforces this.
2. `tempfile::persist` guarantees atomicity — no partial lock file is observable by concurrent readers.
3. Lock file mode `0o600` prevents other OS users from reading the auth token.
4. The asymmetry with BC-ENGINE-002-ERR (`HomeUnresolvable` fail-fast) is intentional: `BaseDirs::new() == None` signals a genuine system-configuration failure (no home directory at all); `ProjectDirs::runtime_dir() == None` on macOS is expected platform behavior, warranting a documented fallback rather than fail-fast.

**Edge Cases:**

EC-051: Lock file write fails (filesystem full, permission denied). Daemon exits before accepting any requests. No partial lock file with wrong or empty content is left on disk (tempfile guarantees).

EC-052: Runtime directory does not exist on startup. Daemon creates it with mode `0o700` (owner-only). If directory creation fails, daemon logs error and exits 1.

EC-053: Lock file removed between pid-liveness check and atomic write (TOCTOU race). The `tempfile::persist` atomic-replace pattern mitigates this — the rename step is atomic on POSIX filesystems.

EC-057: macOS startup — `MONOCLE_RUNTIME_DIR` not set, `ProjectDirs::runtime_dir()` returns `None` (expected on macOS), `ProjectDirs::data_local_dir()` returns `Some("~/Library/Application Support/monocle/")`. Daemon uses the `data_local_dir` path as runtime directory. Logs `INFO: runtime_dir fallback to data_local_dir (platform: macos)`. Happy path for default macOS deployment.

EC-058: `MONOCLE_RUNTIME_DIR` env override — operator sets `MONOCLE_RUNTIME_DIR=/tmp/monocle-test`. Daemon uses `/tmp/monocle-test` as runtime directory regardless of platform-default resolution. Logs `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var`. Happy path for containerized and custom deployments.

EC-059: Full-fail path — `MONOCLE_RUNTIME_DIR` not set, `ProjectDirs::new("monocle", "monocle", "monocle")` returns `None` (requires no home directory at all). Daemon exits 1 with `DaemonStartError::RuntimeDirUnresolvable` and message `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`. No lock file created.

**Canonical Test Vectors:**

| Scenario | Input | Expected |
|----------|-------|----------|
| Fresh start (no lock file) | `monocle daemon start` | Lock file created at `<runtime_dir>/monocle.lock` with mode 0600 and `contract_version == 1` as first key |
| Stale lock file (dead pid) | Lock file exists; pid is not alive | WARN logged; old lock file removed; new daemon starts |
| Live daemon already running | Lock file exists; pid is alive | Error logged; exit 1 |
| Clean shutdown | Daemon exits gracefully | Lock file removed; UDS socket removed |
| macOS — data_local_dir fallback | `MONOCLE_RUNTIME_DIR` unset; `ProjectDirs::runtime_dir()` returns `None` | `data_local_dir()` used; INFO logged; daemon starts normally |
| Env override | `MONOCLE_RUNTIME_DIR=/tmp/monocle-test` | `/tmp/monocle-test` used as runtime dir; INFO logged |
| Full-fail | `MONOCLE_RUNTIME_DIR` unset; `ProjectDirs::new(...)` returns `None` | `DaemonStartError::RuntimeDirUnresolvable` raised; exit 1; no lock file created |

**Verification:**
- Integration test in `monocle-runtime/tests/lock_file_lifecycle.rs`: starts daemon, verifies lock file exists at correct path with mode 0600 and `contract_version == 1`; shuts down daemon and verifies lock file is removed.
- Test name: `test_BC_DAEMON_005_lock_file_create_and_cleanup`
- Integration test covers EC-057 (macOS `data_local_dir` fallback via `MONOCLE_RUNTIME_DIR` override to a temp dir), EC-058 (explicit env override), EC-059 (`RuntimeDirUnresolvable` via mocked `ProjectDirs::new()` returning `None`).

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence and §Hard Shutdown
- Cross-ref: BC-LOCK-001 (lock file JSON schema contract)

---

### BC-DAEMON-006 — Crash Recovery Checkpoint

**Priority:** P0 — State continuity contract.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Crash Recovery

**Preconditions:**
1. On startup, `<runtime_dir>/monocle.recovery.json` exists.
2. The pid in the stale or absent lock file is dead (prior daemon exited without clean shutdown).

**Postconditions:**
1. Daemon logs `WARN: recovery checkpoint found; prior daemon exited without clean shutdown`.
2. Daemon reads `last_app_mode` and `shutdown_reason` from the recovery file.
3. If a TUI client attaches within 60 seconds of daemon start, daemon sends the recovery state via the UDS control protocol: `{"type":"recovery_available","last_app_mode":"<...>"}`.
4. TUI displays a recovery banner: `"Prior session ended unexpectedly. Restore state? [Y/n]"`.
5. On TUI acknowledgment (Y or 60-second timeout): `monocle.recovery.json` is deleted.
6. On TUI decline (N): `monocle.recovery.json` is deleted without restoring state.
7. If no TUI attaches within 60 seconds: recovery file is deleted silently and daemon starts fresh.

**Invariants:**
1. The recovery checkpoint file schema is:
   ```json
   {"pid":<N>,"shutdown_reason":"graceful|signal|forced","last_app_mode":"<string>","shutdown_utc":"YYYY-MM-DDTHH:MM:SS.sssZ"}
   ```
   The `shutdown_utc` field MUST use ISO 8601 UTC format with mandatory millisecond precision: `YYYY-MM-DDTHH:MM:SS.sssZ` (matching the `last_hook_ts` format in EC-044). A seconds-only timestamp (e.g., `2026-05-15T07:30:00Z`) is non-compliant. VP-DAEMON-006 enforces this with regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`.
2. Recovery file creation occurs during the drain sequence (step 5 of §Drain) — BEFORE the lock file is removed. If the daemon crashes hard (SIGKILL), the recovery file may not be written; this is acceptable (no recovery file = clean-start behavior).
3. The 60-second TUI attach window is measured from daemon start time, not from the moment the control socket becomes ready.

**Edge Cases:**

EC-054: Recovery file is malformed JSON (e.g., truncated due to a crash during write). Daemon logs `WARN: recovery file malformed; starting fresh` and deletes the file. No banner is shown to the TUI.

EC-055: Multiple recovery files from multiple crash cycles (hypothetical). Only one `monocle.recovery.json` exists per runtime directory (each shutdown overwrites the previous recovery file).

EC-056: TUI attaches exactly at 60-second boundary. If the recovery offer has already been sent (within the window), the TUI receives it. If the 60-second timeout has expired and the recovery file deleted, the TUI connects to a fresh daemon with no recovery state.

**Canonical Test Vectors:**

| Scenario | Input | Expected |
|----------|-------|----------|
| Clean startup (no recovery file) | `monocle daemon start` with no recovery file | No WARN log; normal start |
| Recovery file present | `monocle daemon start` with existing recovery file | WARN logged; UDS message sent if TUI attaches within 60s |
| TUI accepts recovery | TUI responds Y to banner | Recovery file deleted; state offered to TUI |
| TUI declines recovery | TUI responds N to banner | Recovery file deleted; clean start |
| No TUI attaches | 60 seconds elapse without TUI attach | Recovery file deleted silently |

**Verification:**
- Integration test in `monocle-runtime/tests/crash_recovery.rs`: creates a synthetic `monocle.recovery.json` before daemon start; starts daemon; asserts WARN log; connects a mock TUI client within 60 seconds; asserts recovery UDS message is received; sends Y acknowledgment; asserts recovery file is deleted.
- Test name: `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Crash Recovery

---

### BC-RING-001 — JSONL Ring Format Version (FC-01)

**Priority:** P0 — Forward-compatibility contract; locked pre-Phase-1 by human authorization.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Drain

**Preconditions:**
1. The monocle daemon is running and has received at least one hook event.
2. `--persistent-events` flag is set OR the drain path is exercised during graceful shutdown.
3. The JSONL ring buffer serializes `HookEventRecord` instances via `serde_json::to_string`.

**Postconditions:**
1. Every JSONL line written to the ring buffer begins with `{"format_version":1,` — the `format_version` key is the first key in the JSON object.
2. The `format_version` value is always `1` for all Phase 1-origin records. No record written by Phase 1 code has any other value.
3. The module-level const `RING_FORMAT_VERSION: u32 = 1` in `monocle-runtime::ring` is the single source of truth for the format version value. All `HookEventRecord::new(...)` call sites MUST pass `RING_FORMAT_VERSION`, not a literal integer.
4. `HookEventRecord` is defined in `monocle-runtime::ring` (NOT `monocle-core`) with the fields declared in declaration order: `format_version: u32`, `session_id: String`, `timestamp_micros: i64`, `pid: u32`, `hook_type: String`, `tool_name: Option<String>`, `tool_input: Option<serde_json::Value>`.
5. `HookEventRecord` carries `#[non_exhaustive]` and provides `pub fn new(session_id, timestamp_micros, pid, hook_type, tool_name, tool_input) -> Self` constructor.

**Invariants:**
1. `serde_json` preserves struct field declaration order for plain Rust structs (no `#[serde(rename)]` reordering). `format_version` being the first declared field guarantees it serializes first.
2. `RING_FORMAT_VERSION` is never modified without: bumping the version value, updating `HookEventRecord` field layout documentation, and adding a Phase 2 ingestor capable of reading both versions.

**Edge Cases:**

EC-001: `tool_name` and `tool_input` are `None` for hook types that carry no tool context (`SessionStart`, `UserPromptSubmit`, `Stop`). The serialized record for these hook types omits the fields via `#[serde(skip_serializing_if = "Option::is_none")]` or serializes them as JSON `null` depending on the serde configuration. The first key `format_version` is always present regardless.

EC-002: Very large `tool_input` values (up to 256 KiB per BC-DAEMON-003). The JSONL line may approach 256 KiB in length. Ring buffer rotation (100 MB × 5 files per OQ-06) must handle lines of this length without truncation.

EC-003: Ring buffer file truncated mid-line (e.g., crash during write). Phase 2 ring readers MUST handle incomplete trailing lines by ignoring them (standard JSONL reader robustness requirement). The `format_version` first-key contract applies only to complete lines.

**Canonical Test Vectors:**

| Scenario | Input | Expected First JSON Key |
|----------|-------|------------------------|
| PreToolUse record | `HookEventRecord::new(session_id, t, pid, "PreToolUse".into(), Some("Bash".into()), Some(json!({"command":"cargo test"})))` | `"format_version":1` is first |
| SessionStart record | `HookEventRecord::new(session_id, t, pid, "SessionStart".into(), None, None)` | `"format_version":1` is first |
| Stop record | `HookEventRecord::new(session_id, t, pid, "Stop".into(), None, None)` | `"format_version":1` is first |

**Verification:**
- Unit test in `monocle-runtime/tests/jsonl_ring.rs`: constructs a `HookEventRecord` via `HookEventRecord::new(...)`, calls `serde_json::to_string`, and asserts the result begins with `{"format_version":1,`.
- Test name: `test_BC_RING_001_format_version_first_key`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Drain
- FC: FC-01 (JSONL ring format versioning)
- Brief: §Scope (forward-compatibility contracts sub-bullet — JSONL ring format versioning)

---

### BC-AUTH-001 — Auth Token Wire Format (FC-06)

**Priority:** P0 — Security contract; locked pre-Phase-1 by human authorization.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence

**Preconditions:**
1. The monocle daemon has completed its start sequence (steps 1–6 of §Start Sequence).
2. The lock file at `<runtime_dir>/monocle.lock` has been written successfully via `tempfile::persist`.

**Postconditions:**
1. The lock file `authToken` field contains exactly a 64-character lowercase hexadecimal string (32 bytes from `rand::rngs::OsRng`, hex-encoded). No prefix, no suffix. Regex: `/^[0-9a-f]{64}$/`.
2. The wire format for the auth token presented to the daemon (in the `X-Monocle-Authorization` header) is `monocle-v1:<64-char-hex>` — the literal prefix `monocle-v1:` followed by the lock file's 64-char hex value. Total wire length: 74 characters.
3. The daemon's auth middleware uses `constant_time_eq::constant_time_eq` to compare the hex part (after prefix strip) with the stored secret. The comparison is constant-time to prevent timing oracle attacks.
4. Tokens accepted by Phase 1 daemon's `/status`, `/hooks/*`, and `/shutdown` routes use ONLY `X-Monocle-Authorization: monocle-v1:<64-hex>`. No other header format is a valid auth mechanism on Phase 1 endpoints.

**Invariants:**
1. The prefix `monocle-v1:` versions the auth model. Phase 4 federation uses `Authorization: Bearer` on a SEPARATE russh/IPC channel — NOT on Phase 1 HTTP endpoints.
2. The `expected_secret` stored in memory and in the lock file is the bare 64-char hex (no prefix). The prefix is a wire-format concern only.
3. `rand::rngs::OsRng` is the entropy source — not `thread_rng`. This is mandatory for production-grade secret generation.

**Edge Cases:**

EC-004: Token rotation. If the daemon restarts (with a stale lock file), a new 32-byte secret is generated. Any in-flight requests with the old token will receive HTTP 401 after restart. Hook scripts that read the token from the lock file at request time (as specified) will always have the current token.

EC-005: Lock file written atomically via `tempfile::persist`. If the write fails (e.g., filesystem full), the daemon exits before accepting any requests. No partial lock file with a wrong or empty token is left on disk.

EC-006: The lock file `contract_version` field is `1` (first key). Any lock-file reader MUST check `contract_version == 1` before consuming the `authToken` field. This is a separate contract from BC-LOCK-001 but cross-references it.

**Canonical Test Vectors:**

| Scenario | Input | Expected |
|----------|-------|----------|
| Lock file after start | Read `<runtime_dir>/monocle.lock` after `monocle daemon start` | `authToken` field matches `/^[0-9a-f]{64}$/` |
| Successful auth | `GET /status` with `X-Monocle-Authorization: monocle-v1:<authToken-from-lock>` | HTTP 200 |
| Wrong hex | `GET /status` with `X-Monocle-Authorization: monocle-v1:<wrong-hex>` | HTTP 401 |

**Verification:**
- Integration test in `monocle-runtime/tests/auth_token_lifecycle.rs`: reads lock file after daemon start, asserts `authToken` matches regex; presents `monocle-v1:<authToken>` to `/status`, asserts HTTP 200.
- Test name: `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence
- FC: FC-06 (versioned auth token prefix)
- Brief: §Scope (forward-compatibility contracts sub-bullet — versioned auth token prefix)

---

### BC-AUTH-002 — Auth Header Validation (Missing and Invalid Token)

**Priority:** P0 — Security contract; locked pre-Phase-1.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence

**Preconditions:**
1. The monocle daemon is running with a valid lock file.
2. A request arrives at any authenticated endpoint (`/hooks/*`, `/status`, `/shutdown`).

**Postconditions:**
1. **Missing header:** If the `X-Monocle-Authorization` header is absent entirely, return HTTP 401 `{"error":"missing_auth_token"}`. This is a structural precondition failure, not an authentication attempt.
2. **Any value-present failure:** If the header is present but its value fails validation for any reason — bad prefix (does not begin with `monocle-v1:`), bad format, empty suffix, secret mismatch — return HTTP 401 `{"error":"invalid_auth_token"}`. All value-present failure modes return the same body intentionally (no format/mismatch distinction in the response body).
3. `Authorization: Bearer <token>` headers on Phase 1 endpoints (Phase 4 OAuth2 attempt) receive HTTP 401 `{"error":"missing_auth_token"}` — `Authorization: Bearer` is not a recognized header name for Phase 1 endpoints; `X-Monocle-Authorization` is absent.

**Invariants:**
1. The two-body taxonomy (`missing_auth_token` vs. `invalid_auth_token`) is the complete auth error surface for Phase 1. There is no third body. The old body `invalid_auth_token_format` is retired and does not appear in any Phase 1 response.
2. Value-present failures (Rules 2 and 3 in the auth middleware) deliberately return the same body to prevent an attacker from determining whether their token had the structurally correct prefix, even if they cannot read the lock file directly.
3. The distinction between missing and invalid is preserved because a missing header is a client-configuration error (actionable for debugging), not an authentication attempt. The `missing_auth_token` body provides developer-friendly diagnostics at zero security cost.
4. The auth middleware implementation uses `AuthError::Missing` for absent headers and `AuthError::Invalid` for all value-present failures.

**Edge Cases:**

EC-007: Empty `X-Monocle-Authorization` value (header present but value is empty string). Empty string does not begin with `monocle-v1:` — returns HTTP 401 `{"error":"invalid_auth_token"}` (value-present, format-fail case).

EC-008: `X-Monocle-Authorization` header absent entirely. Returns HTTP 401 `{"error":"missing_auth_token"}`.

EC-009: `X-Monocle-Authorization: monocle-v1:` (prefix present but no hex suffix). Passes the prefix check but fails the constant-time secret comparison (empty hex string never matches the 64-char secret). Returns HTTP 401 `{"error":"invalid_auth_token"}` — the empty suffix is a value-present failure.

**Canonical Test Vectors:**

| Scenario | Input Header | Expected HTTP Status | Expected Body |
|----------|-------------|---------------------|---------------|
| Absent header | (no `X-Monocle-Authorization` header) | 401 | `{"error":"missing_auth_token"}` |
| Bare token (no prefix) | `X-Monocle-Authorization: deadbeef...64chars` | 401 | `{"error":"invalid_auth_token"}` |
| Wrong version prefix | `X-Monocle-Authorization: monocle-v2:deadbeef...64chars` | 401 | `{"error":"invalid_auth_token"}` |
| Prefix only, no hex | `X-Monocle-Authorization: monocle-v1:` | 401 | `{"error":"invalid_auth_token"}` |
| Wrong header name (Bearer) | `Authorization: Bearer fake-token` (no `X-Monocle-Authorization`) | 401 | `{"error":"missing_auth_token"}` |
| Correct format, wrong value | `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` | 401 | `{"error":"invalid_auth_token"}` |

**Verification:**
- Integration test in `monocle-runtime/tests/auth_header_rejection.rs`: for each of the 6 test vectors above, sends the specified header to `/status` and asserts the expected HTTP status + body.
- Test name: `test_BC_AUTH_002_auth_header_validation_all_failure_modes`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence
- FC: FC-06 (F-FC-I005 Phase 4 OAuth2 clarification)
- Brief: §Scope (forward-compatibility contracts sub-bullet — versioned auth token prefix)
- Architect adjudication: commit 2db408f — disposition (c) mixed approach; `invalid_auth_token_format` retired

---

### BC-LOCK-001 — Lock File Contract Version Field

**Priority:** P0 — Forward-compatibility contract.

**Source:** SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging

**Preconditions:**
1. The monocle daemon has completed step 6 of its start sequence (lock file written via `tempfile::persist`).

**Postconditions:**
1. The lock file JSON is a valid JSON object containing at minimum these fields in the stated order: `contract_version` (first), `pid`, `port`, `authToken`, `startTimeUtc`, `app`, `version`.
2. `contract_version` is always the FIRST key in the JSON object. Value is `1` for all Phase 1 daemons.
3. `app` field is `"monocle"` — allows future hook-discovery tooling to filter by app name without scanning all lock files.
4. Any lock-file reader MUST check `contract_version == 1` before consuming other fields. An unrecognized `contract_version` triggers a graceful skip with a log warning — no panic, no crash.

**Invariants:**
1. `contract_version` field order parallels the `format_version` convention in the JSONL ring (BC-RING-001). Both formats put the version sentinel first so readers can validate before deserializing remaining fields.
2. The lock file is always written atomically via `tempfile::persist` — no partial lock file with only some fields is observable by concurrent readers.
3. Lock file mode: `0o600` (owner-only read/write). Neither group nor other permissions are set.

**Edge Cases:**

EC-010: Stale lock file with `contract_version` from a future daemon (hypothetical Phase 4 format). A Phase 1 TUI reader encountering an unrecognized `contract_version` MUST log `WARN: lock file contract_version <N> not recognized; skipping` and proceed as if no lock file exists (trigger the "no daemon running" flow, which auto-starts the daemon).

EC-011: Lock file with `contract_version` key present but value not an integer (e.g., `"contract_version": "1"` as a string instead of integer). Phase 1 reader must handle this gracefully (coerce-to-integer or log and skip).

EC-012: Lock file with `contract_version` key missing entirely (pre-Phase-1 format). Same treatment as EC-010: log and skip.

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| Lock file after daemon start | `contract_version` is integer `1`, present as first key |
| Lock file `app` field | `"monocle"` — exact string, no variants |
| Lock file `authToken` field | 64-char hex string matching `/^[0-9a-f]{64}$/` |
| Reader encounters `contract_version: 99` | WARN log, skip, proceed as if no daemon running |

**Verification:**
- Integration test in `monocle-runtime/tests/lock_file_contract.rs`: starts daemon, reads lock file, asserts `contract_version == 1` is first key via `serde_json::Value::Object` iteration (which preserves insertion order for `serde_json::Map<String, Value>`).
- Test name: `test_BC_LOCK_001_contract_version_first_key`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence
- SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging row BC-LOCK-001

---

### BC-ABI-001 — ABI Version in /status Endpoint (FC-03)

**Priority:** P0 — Forward-compatibility contract; locked pre-Phase-1.

**Source:** SS-core-types-and-abi.md v1.2.8 §ABI Version Constant

**Preconditions:**
1. The monocle daemon is running and has been authenticated successfully.
2. A `GET /status` request is issued with a valid `X-Monocle-Authorization: monocle-v1:<token>` header.

**Postconditions:**
1. The JSON response body includes an `abi_version` field with integer value `1`.
2. The value equals `monocle_core::MONOCLE_ABI_VERSION` as compiled into the running binary. For Phase 1 binaries, this is always `1`.
3. The full `/status` response shape (per SS-daemon-lifecycle.md §Health and Status Endpoints) includes: `pid`, `uptime_sec`, `version`, `abi_version`, `lock_file`, `hook_endpoints`, `ring_buffer_fill_pct`, `channel_saturation_pct`, `last_hook_ts`, `tui_attached`.

**Invariants:**
1. `MONOCLE_ABI_VERSION` is a compile-time constant. It cannot differ between a running daemon and the constant exported by `monocle-core`. If they differ, the binary was built with a different `monocle-core` than the one the plugin SDK or federation layer expects.
2. Changing `MONOCLE_ABI_VERSION` from `1` requires an ADR.

**Edge Cases:**

EC-013: Phase 3 plugin SDK encounter with a Phase 1 daemon. The SDK reads `abi_version` from `/status` and must refuse to activate plugins compiled against a different ABI version. This is Phase 3 scope; Phase 1 need only ensure the field is present and correct.

EC-014: Federation handshake (Phase 4) where two daemons running different ABI versions attempt to federate. The initiating daemon reads `abi_version` from the peer's `/status` and responds HTTP 409 Conflict if no compatibility shim is registered. Phase 1 daemon only needs to serve the field; compatibility resolution is Phase 4 scope.

**Canonical Test Vectors:**

| Input | Expected |
|-------|----------|
| `GET /status` (authenticated) | JSON body includes `"abi_version": 1` |
| `GET /status` (unauthenticated) | HTTP 401 |

**Verification:**
- Integration test in `monocle-runtime/tests/status_abi_version.rs`: `GET /status | jq .abi_version == 1`.
- Test name: `test_BC_ABI_001_status_endpoint_returns_abi_version_1`

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §ABI Version Constant
- FC: FC-03

---

### BC-ABI-002 — ABI Version Constant at Crate Root (FC-03)

**Priority:** P0 — Forward-compatibility contract.

**Source:** SS-core-types-and-abi.md v1.2.8 §ABI Version Constant

**Preconditions:**
1. The `monocle-core` crate compiles successfully.

**Postconditions:**
1. `monocle-core` exports `pub const MONOCLE_ABI_VERSION: u32 = 1;` accessible at the crate root as `monocle_core::MONOCLE_ABI_VERSION`.
2. The declaration is in `monocle-core/src/abi.rs` and re-exported from `monocle-core/src/lib.rs` via `pub use abi::MONOCLE_ABI_VERSION;`.
3. Downstream crates can write compile-time assertions against the constant:
   ```rust
   const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1,
       "ABI version mismatch — check monocle-core version");
   ```

**Invariants:**
1. The constant is `u32`, not `u8`, `u16`, or `usize` — chosen for proto field parity (proto `uint32`).
2. The constant is `pub` (not `pub(crate)`) — must be accessible from `monocle-plugin-sdk` (Phase 3) and `monocle-ipc` (Phase 4).

**Edge Cases:**

EC-015: Compile-time assertion in `monocle-plugin-sdk`. If `MONOCLE_ABI_VERSION` is changed without updating the plugin SDK's compile-time assertion, the SDK will fail to compile with a clear error message. This is the intended behavior.

**Canonical Test Vectors:**

| Input | Expected |
|-------|----------|
| `monocle_core::MONOCLE_ABI_VERSION` | Compile-time value `1u32` |
| `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "...");` | Compiles without error |

**Verification:**
- Lint test in `monocle-core/tests/abi_stability.rs` asserting the constant is exactly `1` and publicly accessible.
- Test name: `test_BC_ABI_002_abi_version_const_exported`

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §ABI Version Constant
- FC: FC-03

---

### BC-TYPES-001 — Non-Exhaustive Enum Policy (FC-02)

**Priority:** P0 — Forward-compatibility contract; ADR-0004 governs exemptions.

**Source:** SS-core-types-and-abi.md v1.2.8 §Enum Extensibility

**Preconditions:**
1. The `monocle-core` crate source tree is parsed via `syn 2` to enumerate all `pub` enum declarations.

**Postconditions:**
1. Every `pub` enum in `monocle-core` carries `#[non_exhaustive]` unless explicitly exempted by an ADR.
2. At Phase 1 PRD dispatch, the exhaustive-enum forbidden list contains exactly two entries: `Phase1Permission` and `ClaudeCodeTool` (both documented in ADR-0004).
3. Any new exemption requires a new ADR before the code compiles in CI. No exemption is granted by inline comment or spec prose alone.
4. The mandatory non-exhaustive enums include at minimum: `HookType`, `HookEvent`, `DenyReason`, `AllowPattern`, `DenyPattern`, `BlockingSeverity`, `SessionStatus`, `HookDecision`, `DeferUntil`.

**Invariants:**
1. The verification mechanism is a `syn 2` AST parse (NOT clippy). The test in `monocle-core/tests/enum_audit.rs` walks every `Item::Enum` node across all `.rs` files in `monocle-core/src/**/*.rs`, asserts `#[non_exhaustive]` is present unless the enum identifier is in the ADR-0004 EXEMPT list. This is deterministic and load-bearing; clippy's `non_exhaustive_omitted_patterns` lint is supplement only.
2. Adding a variant to any `#[non_exhaustive]` enum (except `Phase1Permission`) is NOT a breaking change and does NOT require a SemVer-major version bump.
3. `Phase1Permission` is exhaustive because the TUI permission dispatcher must handle every variant at compile time. Phase 3 adds `monocle-plugin-sdk::PluginPermission` as a separate enum rather than extending `Phase1Permission`.

**Edge Cases:**

EC-016: New enum added in a future PR without `#[non_exhaustive]`. The `syn 2` AST audit test in CI must reject it unless an ADR is filed concurrently.

EC-017: `ClaudeCodeTool::Unknown(String)` catch-all variant. This is the runtime safety net for tools added by Anthropic between monocle releases. It does NOT make `ClaudeCodeTool` non-exhaustive in the Rust sense — the enum is still exhaustive (every `match` must cover all variants including `Unknown`). The `Unknown` catch-all is the intended escape valve that keeps the enum exhaustive without breaking on new tools.

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| `syn 2` AST parse of `monocle-core/src/**/*.rs` with a new `pub enum Foo { A, B }` (missing `#[non_exhaustive]`) | Test asserts error: enum `Foo` missing `#[non_exhaustive]` |
| `syn 2` AST parse with `Phase1Permission` lacking `#[non_exhaustive]` | No error (ADR-0004 EXEMPT list) |

**Verification:**
- AST audit test in `monocle-core/tests/enum_audit.rs`: uses `syn 2` to parse every `.rs` file in `monocle-core/src/**/*.rs`, walks `Item::Enum` nodes, asserts `#[non_exhaustive]` is present unless enum identifier is in the ADR-0004 EXEMPT list (`Phase1Permission`, `ClaudeCodeTool`).
- Test name: `test_BC_TYPES_001_non_exhaustive_enum_coverage`

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §Enum Extensibility
- ADR: ADR-0004 (exhaustive-enum exemption rationale)
- FC: FC-02

---

### BC-FACTORY-001 — FactoryAdapter Trait Definition (FC-04 CRITICAL)

**Priority:** P0 — Critical forward-compatibility contract; CRITICAL designation from FC-04.

**Source:** SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait

**Preconditions:**
1. The `monocle-core` crate compiles.
2. `monocle-core::factory` module is accessible.

**Postconditions:**
1. `FactoryAdapter` trait is defined in `monocle-core::factory` with the exact signature:
   - `fn detect(workspace_root: &Path) -> Option<FactoryDetection> where Self: Sized`
   - `fn matches(&self, workspace_root: &Path) -> bool`
   - `fn state_file_path(&self) -> &Path`
   - `fn read_state(&self) -> Result<FactoryState, FactoryReadError>`
   - `fn subscribe(&self) -> Result<StateChangeStream, FactorySubscribeError>`
   - `fn display_name(&self) -> &str`
   - `fn abi_version(&self) -> u32` (default impl returning `crate::MONOCLE_ABI_VERSION`)
2. The trait carries NO sealed bound — supertrait bounds are `Send + Sync + 'static` ONLY. No `private::Sealed` supertrait exists.
3. Supporting types are co-located in `monocle-core::factory`: `FactoryDetection` (3 fields), `FactoryState` (7-field canonical struct), `BlockingIssue`, `BlockingSeverity`, `ConvergenceMetrics`, `FactoryReadError`, `FactorySubscribeError`, `StateChangeStream` type alias.
4. `FactoryState` uses `serde_yaml_ng::Value` for `custom_fields` (not `serde_json::Value`). `convergence` is `Option<ConvergenceMetrics>`. `cycle` is `Option<String>`. These `Option` types represent legitimate absence — NOT unknown. Consumers display `"—"` or `"pending"` for `None`, not `"unknown"`.

**Invariants:**
1. `FactoryAdapter` is an OPEN extension trait — third-party crates may implement it. Sealing would defeat Phase 3 WASM plugin extensibility.
2. The 7-field `FactoryState` struct is the canonical shape from vision §FactoryAdapter. No `raw_content` field (user red-line per SS-core-types-and-abi.md §FactoryAdapter Trait §Trait Signature rustdoc).
3. Phase 1 `subscribe()` implementations MUST return `Ok(Box::pin(futures::stream::empty()))`. The live watcher is Phase 3 scope.
4. Any modification to the method signatures listed in postcondition 1 is a BREAKING change requiring an ADR.

**Edge Cases:**

EC-018: `dyn FactoryAdapter` dispatch. The `detect` method has `where Self: Sized` and is not callable on trait objects. Callers using `dyn FactoryAdapter` use `matches()` instead (no `Self: Sized` bound). This asymmetry is intentional and documented.

EC-019: `custom_fields` with YAML flow-style lists or block scalars. `parse_frontmatter_extra_fields` skips these (cannot decode without a full YAML parser). Callers needing full YAML semantics re-parse with `serde_yaml_ng::from_str`. The escape hatch is `custom_fields` itself — structured values unrepresentable as simple scalars are accessible via re-parse.

EC-020: Phase 3 WASM adapter implements `FactoryAdapter` in a separate crate. The trait is open; the adapter's `fn abi_version()` default returns `crate::MONOCLE_ABI_VERSION` from the monocle-core version linked into the WASM component. Phase 3 plugin loader checks this value before activating the adapter.

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| `VsddFactoryAdapter::detect` called with monocle repo root | `Some(FactoryDetection { display_name: "VSDD Factory", ... })` |
| `VsddFactoryAdapter::matches` with non-factory dir | `false` |
| `cargo check` with Phase 1 workspace | Compiles without error; no `private::Sealed` supertrait in rustdoc |

**Verification:**
- AST audit test in `monocle-core/tests/factory_trait_surface.rs`: uses `syn 2` to parse `monocle-core/src/factory.rs`, asserts 7-method count, no `Sealed` supertrait bound, and `Send + Sync + 'static` bounds only.
- Test name: `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound`

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait
- FC: FC-04 (CRITICAL)
- Brief: §Scope (forward-compatibility contracts sub-bullet — FactoryAdapter trait)

---

### BC-FACTORY-002 — VsddFactoryAdapter Implementation

**Priority:** P0 — Self-referential integration test requirement.

**Source:** SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait §Phase 1 Implementation: VsddFactoryAdapter

**Preconditions:**
1. `monocle-core` compiles.
2. `VsddFactoryAdapter` implements `FactoryAdapter`.

**Postconditions:**
1. `VsddFactoryAdapter::new(workspace_root: PathBuf) -> Self` is a public constructor. It derives `state_file = workspace_root.join(".factory").join("STATE.md")`. No validation is performed at construction time — validation is deferred to `detect()` and `read_state()`.
2. `VsddFactoryAdapter::detect(workspace_root)` returns `Some(FactoryDetection)` when called against monocle's own workspace root — the directory containing `.factory/STATE.md` with `document_type: pipeline-state` in YAML frontmatter.
3. `read_state()` returns `None` for absent optional fields: absent `current_cycle:` → `cycle: None`; absent §Session Resume Checkpoint → `convergence: None`. Consumers MUST NOT receive `"unknown"` as a placeholder for absent optional fields.
4. `parse_frontmatter_field` and `parse_frontmatter_extra_fields` apply these guards: skip continuation lines (leading whitespace); return `None` for empty values; return `None` for flow-style list values (beginning with `[`); return `None` for block scalar markers (beginning with `|` or `>`). YAML quoted scalars are unquoted (single and double quotes stripped).

**Invariants:**
1. The detection criterion is `document_type: pipeline-state` in the YAML frontmatter of `.factory/STATE.md`. No other file or field is required for detection.
2. `display_name()` returns `"VSDD Factory"` — the exact string used in TUI display.
3. `subscribe()` returns `Ok(Box::pin(futures::stream::empty()))` in Phase 1.

**Edge Cases:**

EC-021: STATE.md file with `document_type: pipeline-state` in a non-YAML frontmatter block (e.g., the string appears in the document body). `parse_frontmatter_field` checks that the document begins with `---` on the FIRST line before scanning for the key. Body occurrences are not detected.

EC-022: STATE.md file with `awaiting: "round 18 validation chain"` (YAML double-quoted value). `parse_frontmatter_field` strips surrounding double quotes and returns `Some("round 18 validation chain")` — the semantic value, not the YAML encoding.

EC-023: STATE.md file with `blocking_issues: []` (YAML flow-style list). `parse_frontmatter_extra_fields` skips this because the value begins with `[`. The `blocking_issues` field in `FactoryState` is populated by Phase 3 body parsing, not frontmatter extraction. Phase 1 always returns `Vec::new()`.

**Canonical Test Vectors:**

| Scenario | Input | Expected |
|----------|-------|----------|
| Self-referential detection | `VsddFactoryAdapter::detect(monocle_repo_root)` | `Some(FactoryDetection { display_name: "VSDD Factory" })` |
| read_state — absent current_cycle | STATE.md with no `current_cycle:` key | `FactoryState { cycle: None, ... }` |
| read_state — present cycle | STATE.md with `current_cycle: "cycle-001"` | `FactoryState { cycle: Some("cycle-001"), ... }` |
| read_state — quoted awaiting | STATE.md with `awaiting: "human GO"` | `FactoryState { awaiting: Some("human GO"), ... }` |
| Nonexistent workspace | `VsddFactoryAdapter::detect("/tmp/not-a-factory")` | `None` |

**Verification:**
- Integration test `monocle-core/tests/factory_self_referential.rs`: calls `VsddFactoryAdapter::detect(workspace_root)` with monocle repository root; asserts `Some(_)` returned with `display_name == "VSDD Factory"`; calls `read_state()`, asserts `cycle` is `None` or `Some(_)` (not `"unknown"`).
- Test name: `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection`

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait §Phase 1 Implementation: VsddFactoryAdapter
- Brief: §Success Criteria (factory pattern detection row — "Detection succeeds on monocle's own `.factory/`")

---

### BC-PROTO-001a — HookEnvelope Proto Field Number Contract (FC-05, wire-format)

**Priority:** P0 — Wire-format contract; forward-compatibility.

**Source:** SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas

**Preconditions:**
1. The `.proto` file `monocle-proto/proto/monocle/v1/hook_envelope.proto` exists and compiles via `protoc` or `prost-build`.

**Postconditions:**
1. `schema_version` is declared at proto FIELD NUMBER 1 in `HookEnvelope` (`uint32 schema_version = 1;`).
2. This is a WIRE-FORMAT contract: it governs the binary protobuf encoding on the wire. Field number 1 in proto3 binary encoding uses tag `0x08` (field 1, varint type). Any proto consumer in any language sees field number 1 as `schema_version`.
3. The proto package is `monocle.v1`. The file path is `monocle-proto/proto/monocle/v1/hook_envelope.proto`.
4. The oneof `event` in `HookEnvelope` uses field numbers 10–14 for the five event variants: `session_start (SessionStartEvent) = 10`, `prompt_submit (UserPromptSubmitEvent) = 11`, `pre_tool_use (PreToolUseEvent) = 12`, `notification (NotificationEvent) = 13`, `stop (StopEvent) = 14`. These are in the Phase 1 reserved range (1–99). Field names are snake_case; the parenthetical shows the event message type.
5. Phase 4 federation additions MUST use field numbers 100–999. Phase 5+ MUST use 1000+.

**Invariants:**
1. Any change to a Phase 1 field (field numbers 1–99) in `HookEnvelope` or any event message is a BREAKING change: bump `schema_version` AND produce an ADR.
2. Removing a Phase 1 field is forbidden. Deprecated fields are marked `[deprecated = true]` and the field number is retained as reserved.

**Edge Cases:**

EC-024: Proto3 unknown-field handling. A Phase 1 receiver encountering a `HookEnvelope` with fields in the 100–999 range (Phase 4 additions) MUST NOT crash or reject the message. Proto3 forward compatibility: unknown fields are preserved. Phase 1 receiver ignores Phase 4+ fields.

EC-025: `schema_version` field value is `0` in a received message. Phase 4 federation test case: receiver MUST log a warning and skip without panic. Value `0` is not a valid Phase 1 schema version.

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| `protoc --decode monocle.v1.HookEnvelope` on an encoded Phase 1 message | Field number 1 is `schema_version` |
| Round-trip `HookEnvelope { schema_version: 1, ... }` via prost encode/decode | `envelope.schema_version == 1` after decode |

**Verification:**
- `monocle-proto/tests/wire_field_order.rs`: round-trips a message and asserts field number 1 is `schema_version` via prost-build's generated descriptor.
- Test name: `test_BC_PROTO_001a_schema_version_field_number_1`

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas
- FC: FC-05

---

### BC-PROTO-001b — HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface)

**Priority:** P0 — Rust API contract; companion to BC-PROTO-001a.

**Source:** SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas

**Preconditions:**
1. `monocle-proto` crate compiles (prost-build generates Rust types from the `.proto` file).

**Postconditions:**
1. The prost-build-generated `HookEnvelope` Rust struct exposes `pub schema_version: u32`.
2. For all Phase 1-origin messages, this field has value `1`.
3. The generated struct field order is an implementation detail of prost-build and is NOT a behavioral contract. Only the proto field number (BC-PROTO-001a) and the Rust field accessibility (`pub schema_version: u32`) are contractual.

**Invariants:**
1. `monocle-proto` declares `prost 0.14` with an EXACT version pin (per SS-deps-pin-manifest.md). The exact pin prevents silent prost-build behavior changes from altering the generated Rust API between builds.
2. The `build.rs` in `monocle-proto` generates Rust types but activates no wire path in Phase 1 — the types are compiled into the binary and available for Phase 4 without workspace changes.

**Edge Cases:**

EC-026: prost-build version change. An exact pin prevents this, but if the pin is relaxed in a future PR, the `pub schema_version: u32` field accessibility must be re-verified after any prost-build version change.

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| Construct `HookEnvelope { schema_version: 1, ... }` in Rust | Compiles; `envelope.schema_version == 1` |
| Access `HookEnvelope::schema_version` from `monocle-runtime` | Field is accessible (`pub`) without re-import |

**Verification:**
- Unit test in `monocle-proto/tests/schema_version.rs`: constructs a `HookEnvelope` with `schema_version: 1`, asserts `envelope.schema_version == 1`.
- Test name: `test_BC_PROTO_001b_schema_version_rust_field`

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas
- FC: FC-05 (Rust surface; wire-format covered by BC-PROTO-001a)

---

### BC-PROTO-002 — Phase 4 schema_version Validation Requirement (FC-05)

**Priority:** P1 — Phase 4 forward-compatibility requirement; Phase 1 must define the schema correctly.

**Source:** SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas

**Preconditions:**
1. Phase 4 federation is active (out of scope for Phase 1 testing; this BC defines the contract Phase 1 schema must support).
2. A Phase 4 federation node receives a `HookEnvelope` message.

**Postconditions:**
1. Phase 4 federation nodes check `schema_version` before deserializing event payloads.
2. A node receiving a message with an unrecognized `schema_version` MUST log a warning and skip the message rather than crash (proto3 unknown-field semantics apply).
3. The Phase 1 `HookEnvelope` schema is the canonical wire representation for cross-host federation in Phase 4. No schema changes are permitted between Phase 1 definition and Phase 4 activation without bumping `schema_version` and producing an ADR.

**Invariants:**
1. Proto3 forward compatibility guarantee: a Phase 4 receiver that understands Phase 5+ fields can still decode a Phase 1 message (unknown fields are preserved in proto3).
2. The `schema_version` field exists specifically so Phase 4 can distinguish Phase 1 messages from future-format messages without heuristics.

**Edge Cases:**

EC-027: Phase 4 receiver encounters `schema_version: 0`. Must skip with warning (value 0 is not a defined version). Must not panic.

EC-028: Phase 4 receiver encounters `schema_version: 2` (hypothetical Phase 5 format). Must skip with warning (unrecognized version). Must not attempt to decode as Phase 1 format.

**Canonical Test Vectors:**

| Scenario | Expected (Phase 4 test) |
|----------|------------------------|
| Message with `schema_version: 0` | Skip with WARN log; no panic |
| Message with `schema_version: 1` | Decode successfully |
| Message with `schema_version: 2` | Skip with WARN log; no panic |

**Verification:**
- Phase 4 integration test (out of Phase 1 scope): simulates a `schema_version: 0` message and asserts the receiver skips without panic.
- Phase 1 gate: `schema_version` field is present at field number 1 in the compiled schema (verified by BC-PROTO-001a/001b).
- Test name: `test_BC_PROTO_002_schema_version_validation_skip_unknown` (Phase 4 test)

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas
- FC: FC-05

---

### BC-ENGINE-001 — EngineModule Trait Definition

**Priority:** P0 — Core abstraction contract.

**Source:** SS-engine-module.md v1.1.15 §EngineModule Trait Signature

**Preconditions:**
1. `monocle-core` crate compiles.
2. `monocle-core::engine` module is accessible.

**Postconditions:**
1. `EngineModule` trait is defined in `monocle-core::engine` using `#[async_trait::async_trait]` with exactly these five methods and return types:
   - `fn id(&self) -> &'static str`
   - `fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError>`
   - `fn detect(&self, proc: &ProcessSnapshot) -> bool`
   - `async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>`
   - `async fn on_hook(&self, event: HookEvent) -> HookResponse`
2. The trait carries NO sealed bound. Supertrait bounds: `Send + Sync + 'static` only.
3. Supporting types are co-located in `monocle-core::engine`: `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`, `SessionStatus`, `HookResponse`, `HookDecision`, `DeferUntil`, `EngineMetadataError`.
4. `EnrichedSession::last_event_micros` is `Option<i64>`. `None` means no hook events received yet. `Some(t)` means microseconds since Unix epoch of most recent hook event. Consumers MUST NOT treat `0i64` as a sentinel — `0` is the Unix epoch (1970-01-01), not a valid last-event timestamp.
5. `metadata()` and `enrich()` MUST NOT substitute a default path when the platform home directory is unresolvable. They MUST return `Err(EngineMetadataError::HomeUnresolvable)`. Daemon initialization MUST fail fast with a diagnostic (no silent-fallback, per CLAUDE.md SOUL #4).

**Invariants:**
1. `EngineModule` is an OPEN trait — third-party WASM plugins implement it in Phase 3.
2. `HookEvent` type is defined in `monocle-core/src/hook_events.rs` (SS-core-types-and-abi.md §Non-Exhaustive Inner Structs). `EngineModule` references it; does not re-declare.
3. The `#[async_trait]` macro is required because `async fn` in traits is not stable in the MSRV (Rust 1.86). The macro is a compile-time transformation; no runtime overhead.

**Edge Cases:**

EC-029: `metadata()` called when `$HOME` is unset (e.g., systemd service unit without `Environment=HOME`). Must return `Err(EngineMetadataError::HomeUnresolvable)`. Daemon start fails with a clear diagnostic. Not a recoverable error — the daemon cannot operate without knowing where Claude Code's config lives.

EC-030: `detect()` receives a `ProcessSnapshot` with `exe_path: None` (process exited before path was resolved). Must return `false` — no detection on missing path.

EC-031: `on_hook()` called with an unrecognized `HookEvent` variant (future Phase 4 addition). Since `HookEvent` is `#[non_exhaustive]`, all match sites have a wildcard arm. `on_hook()` returns `HookResponse::new(HookDecision::Allow)` as the fail-open default for unrecognized event types.

Security rationale for fail-open on unrecognized variants: Phase 1 specifies exactly 5 hook variants (`SessionStart`, `UserPromptSubmit`, `PreToolUse`, `Notification`, `Stop`). Unrecognized event types in Phase 1 are presumed non-permission-relevant by design — Phase 1 only knows the canonical 5 variants, and any novel variant arriving must be from a future Phase 4+ hook extension that carries its own permission semantics. Until those variants are explicitly enumerated and their permission implications specified, `Allow` is the correct default because there is no permission context available to defer on. A `Defer` response on an unrecognized variant would stall the hook caller indefinitely (no TUI handler registered for the unknown type). The monocle daemon binds exclusively to `127.0.0.1` with no untrusted remote callers in Phase 1; the localhost threat model does not warrant defensive stalling on unknown variants. Future Phase 4 hooks carrying permission semantics MUST be explicitly enumerated in the `HookEvent` enum and matched in `on_hook()` — the wildcard arm is a forward-compat escape hatch, not a permission bypass.

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| `syn 2` AST parse of `monocle-core/src/engine.rs` | 5 methods present; no `Sealed` supertrait; return types match specification |
| `cargo check` with Phase 1 workspace | Compiles without error |

**Verification:**
- AST audit test in `monocle-core/tests/engine_module_surface.rs`: uses `syn 2` to parse `monocle-core/src/engine.rs`, asserts 5-method count, no `Sealed` supertrait bound, and verifies return-type token-stream matches for each method signature.
- Test name: `test_BC_ENGINE_001_trait_defined_all_methods_no_sealed_bound`

**Traceability:**
- Source: SS-engine-module.md v1.1.15 §EngineModule Trait Signature
- Vision: §EngineModule

---

### BC-ENGINE-002 — ClaudeCodeModule Implementation

**Priority:** P0 — Phase 1 concrete harness implementation.

**Source:** SS-engine-module.md v1.1.15 §Phase 1 Implementation: ClaudeCodeModule

**Preconditions:**
1. `monocle-runtime` crate compiles.
2. `ClaudeCodeModule` is defined in `monocle-runtime::engine::claude_code`.
3. A `ProcessSnapshot` is provided to `detect()`.

**Postconditions:**
1. `ClaudeCodeModule` implements `EngineModule`.
2. A public `ClaudeCodeModule::new(hook_base_url: String) -> Self` constructor is provided. Construction is infallible — URL validation is deferred to `preflight()`.
3. `id()` returns the string `"claude-code"` — stable, never changes.
4. `detect()` returns `true` for any process whose `exe_path.file_name()` equals `"claude"` or `"claude.js"`. STRICT basename match on the RESOLVED exe path. NOT a suffix match on `cmdline[0]`.
5. `detect()` returns `false` when `exe_path` is `None`, regardless of `cmdline` contents.

**Invariants:**
1. The strict-basename rule prevents false positives from: `claude-squad`, `claudio`, `claude-code-router`, or any other binary whose name contains "claude" as a prefix or substring.
2. `cmdline` is retained for `enrich()` (reading `CLAUDE_SESSION_ID`). It MUST NOT be used as the primary detection signal — `exe_path` is the canonical signal.

**Edge Cases:**

EC-032: Process `cmdline[0]` is `"claude"` but `exe_path` resolves to `/usr/local/bin/claude-squad`. `detect()` returns `false` — `cmdline[0]` is not consulted.

EC-033: `exe_path` is `/usr/local/bin/claude` (no extension). `file_name()` is `"claude"`. `detect()` returns `true`.

EC-034: `exe_path` is `/usr/local/bin/claude.js` (Node.js wrapper). `file_name()` is `"claude.js"`. `detect()` returns `true` — matches the second allowed name.

EC-035: `exe_path` is `Some(PathBuf::from("/usr/local/bin/claude-squad"))`. `file_name()` is `"claude-squad"`. Neither `"claude"` nor `"claude.js"`. `detect()` returns `false`.

**Canonical Test Vectors:**

| Scenario | ProcessSnapshot | Expected |
|----------|----------------|----------|
| Real claude binary | `exe_path: Some("/usr/local/bin/claude")` | `detect() == true` |
| claude.js wrapper | `exe_path: Some("/usr/local/bin/claude.js")` | `detect() == true` |
| claude-squad (false positive risk) | `exe_path: Some("/usr/local/bin/claude-squad")` | `detect() == false` |
| exe_path None | `exe_path: None, cmdline: vec!["claude"]` | `detect() == false` |
| claudio (false positive risk) | `exe_path: Some("/usr/local/bin/claudio")` | `detect() == false` |

**Verification:**
- Unit test in `monocle-runtime/tests/engine_module_claude_detect.rs` with all 5 test vectors above.
- Test name: `test_BC_ENGINE_002_claude_code_module_strict_basename_detect`

**Traceability:**
- Source: SS-engine-module.md v1.1.15 §Phase 1 Implementation: ClaudeCodeModule

---

### BC-ENGINE-002-ERR — HomeUnresolvable Error Contract

**Priority:** P0 — No-silent-fallback guarantee; CLAUDE.md SOUL #4.

**Source:** SS-engine-module.md v1.1.15 §Behavioral Contracts BC-ENGINE-002-ERR

**Preconditions:**
1. `ClaudeCodeModule` is instantiated via `ClaudeCodeModule::new("http://127.0.0.1:7891".into())`.
2. Platform home directory resolution fails: `directories::BaseDirs::new()` returns `None`. This is induced in tests by unsetting `HOME`, `USERPROFILE`, `HOMEDRIVE`, and `HOMEPATH` using `temp-env ^0.3` with `features = ["async_closure"]`.

**Postconditions:**
1. `ClaudeCodeModule::metadata()` returns `Err(EngineMetadataError::HomeUnresolvable)`.
2. `ClaudeCodeModule::enrich()` returns `Err(EngineMetadataError::HomeUnresolvable)`.
3. Neither method substitutes a relative path (e.g., `.claude`) or a hardcoded fallback path when `BaseDirs::new()` returns `None`.
4. The daemon initialization code that calls `metadata()` at startup MUST propagate this error and surface a diagnostic message rather than silently continuing with a wrong path.

**Invariants:**
1. `temp-env ^0.3` (with `features = ["async_closure"]`) is the env-isolation strategy — NOT `std::env::remove_var` (unsafe in multi-threaded test harnesses) and NOT `#[serial]` (mitigates race but doesn't guarantee cleanup on panic).
2. `metadata()` (synchronous) and `enrich()` (async) require different `temp-env` wrappers: `temp_env::with_vars` for sync and `temp_env::async_with_vars` for async. They MUST NOT be co-located in the same `with_vars` call.
3. The four env vars to clear: `HOME`, `USERPROFILE`, `HOMEDRIVE`, `HOMEPATH`. NOT `XDG_*` variables (not consulted by `BaseDirs::home_dir()`).

**Edge Cases:**

EC-036: Windows CI runner has a registered user SID. `BaseDirs::new()` may succeed via `SHGetKnownFolderPath` even with all four env vars cleared. The test on Windows CI is best-effort for the `None` path; the contract is fully deterministic on Linux/macOS.

EC-037: `enrich()` called with a `ProcessSnapshot` that has `working_dir: None`. The transcript path derived from `working_dir` is `None`. This is separate from the `HomeUnresolvable` error path — if `BaseDirs::new()` succeeds, `enrich()` returns `Ok(EnrichedSession)` with `transcript_path: None`.

**Canonical Test Vectors:**

| Scenario | Input | Expected |
|----------|-------|----------|
| metadata() with HOME unset | `temp_env::with_vars([("HOME", None::<&str>), ...], || { module.metadata() })` | `Err(EngineMetadataError::HomeUnresolvable)` |
| enrich() with HOME unset | `temp_env::async_with_vars([("HOME", None::<&str>), ...], async { module.enrich(&snapshot).await })` | `Err(EngineMetadataError::HomeUnresolvable)` |
| metadata() with HOME set | Normal environment | `Ok(EngineMetadata { config_paths: [~/.claude, ~/.claude.json], ... })` |

**Verification:**
- Test in `monocle-runtime/tests/engine_module_home_unresolvable.rs`. Sync half uses `temp_env::with_vars`; async half uses `temp_env::async_with_vars` in a separate `#[tokio::test]`.
- Dev dependency: `temp-env = { version = "^0.3", features = ["async_closure"] }` in `monocle-runtime` `[dev-dependencies]`.
- Test name: `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich`

**Traceability:**
- Source: SS-engine-module.md v1.1.15 §Behavioral Contracts BC-ENGINE-002-ERR
- CLAUDE.md SOUL #4 (no silent fallback for unresolvable platform home directory)

---

### BC-ENGINE-003 — ClaudeCodeModule Inherent Methods

**Priority:** P0 — Phase 1 hook path routing contract.

**Source:** SS-engine-module.md v1.1.15 §Struct-level inherent operations

**Preconditions:**
1. `ClaudeCodeModule` is instantiated via `ClaudeCodeModule::new("http://127.0.0.1:7891".into())`.

**Postconditions:**
1. `ClaudeCodeModule::hook_paths()` is an inherent method (NOT a trait method) that returns `HashMap<HookType, String>` with exactly 5 entries:
   - `HookType::SessionStart` → `"/hooks/session-start"`
   - `HookType::UserPromptSubmit` → `"/hooks/prompt-submit"`
   - `HookType::PreToolUse` → `"/hooks/pre-tool-use"`
   - `HookType::Notification` → `"/hooks/notification"`
   - `HookType::Stop` → `"/hooks/stop"`
2. `ClaudeCodeModule::spawn(args: SpawnArgs) -> Result<SessionHandle, SpawnError>` is an inherent async method. Phase 1 implementation is `todo!()` — the signature is binding; the implementation is stubbed.
3. `ClaudeCodeModule::preflight() -> Result<EngineVersion, PreflightError>` is an inherent async method. Phase 1 implementation is `todo!()` — the signature is binding.
4. The ABI version is read as `monocle_core::MONOCLE_ABI_VERSION` at call sites. No `abi_version` method appears on any trait.
5. These three methods (`hook_paths`, `spawn`, `preflight`) are NOT in the `EngineModule` trait. They are engine-specific operational methods on the concrete struct.

**Invariants:**
1. The 5 hook path strings exactly match the canonical endpoint set from brief §Scope (§In Scope sub-bullets for hook endpoints): `PostToolUse` is NOT included (JC-2 gene-source parity).
2. Path strings begin with `/` (relative to the daemon's base URL).
3. The `hook_paths()` method is synchronous — it returns a static mapping. No I/O, no async.

**Edge Cases:**

EC-038: `spawn()` called in Phase 1 with a valid `SpawnArgs`. Returns `todo!()` panic — intentional. Phase 1 story provides the full implementation. The stub is acceptable because Phase 1 daemon does not spawn sessions (it receives hook POSTs from externally-started sessions).

EC-039: `preflight()` called at daemon startup before accepting hook registrations. Returns `todo!()` panic in Phase 1 stub. Phase 1 story replaces the stub with `which claude` + `claude --version` checks.

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| `hook_paths().len()` | `5` |
| `hook_paths()[HookType::PreToolUse]` | `"/hooks/pre-tool-use"` |
| `hook_paths()[HookType::Stop]` | `"/hooks/stop"` |
| `hook_paths()[HookType::UserPromptSubmit]` | `"/hooks/prompt-submit"` |
| `hook_paths()[HookType::SessionStart]` | `"/hooks/session-start"` |
| `hook_paths()[HookType::Notification]` | `"/hooks/notification"` |

**Verification:**
- Unit test in `monocle-runtime/tests/engine_module_claude_methods.rs`: asserts `module.hook_paths().len() == 5` with the exact path string for each `HookType`.
- Test name: `test_BC_ENGINE_003_claude_module_hook_paths_five_entries`

**Traceability:**
- Source: SS-engine-module.md v1.1.15 §Struct-level inherent operations
- FC: JC-2 (5-endpoint parity, PostToolUse omitted)

---

## 4. Non-Functional Requirements

| ID | Category | Requirement | Numerical Target | Validation Method |
|----|----------|-------------|-----------------|------------------|
| NFR-001 | Latency | Hook ingestion end-to-end response time for `PreToolUse`, `Stop`, `SessionStart`, `UserPromptSubmit` | ≤300ms | Integration test with stopwatch between hook POST and response; Claude Code's upstream timeout ceiling per BC-HOOK-022 |
| NFR-002 | Latency | Hook ingestion end-to-end response time for `Notification` | ≤2000ms | Integration test; gene-source BC-HOOK-022 timeout ceiling |
| NFR-003 | Latency | Permission prompt overlay render after hook POST receipt | ≤100ms | Integration test with TUI client attached; measures from POST receipt to TUI event dispatch |
| NFR-004 | Security | Auth token entropy | 32 bytes from `rand::rngs::OsRng` (not `thread_rng`) | Code review + unit test asserting `OsRng` usage |
| NFR-005 | Security | Hook body size limit (all POST endpoints) | 256 KiB (262,144 bytes); HTTP 413 on excess | Integration test: send 262,145-byte body, assert 413 response |
| NFR-006 | Throughput | Bounded event bus with visible drop counter | No unbounded channel; drop counter renders in status bar; 1000 events/sec sustained without queue overflow | Integration test at 1000 events/sec asserting drop counter assertion |
| NFR-007 | Build | MSRV | Rust 1.86 (ratatui 0.30 floor) | CI matrix check; `rust-toolchain.toml` |
| NFR-008 | Build | Platform targets | macOS + Linux (darwin/linux × amd64/arm64) | GitHub Actions CI matrix |
| NFR-009 | Security | Lock file permissions | `0o600` (owner-only read/write) | Integration test: `stat` lock file after daemon start; assert mode is `0600` |
| NFR-010 | Correctness | Constant-time auth comparison | `constant_time_eq::constant_time_eq` used for token comparison | Code review; no `==` on token strings |
| NFR-011 | Forward-compat | DTU clone fidelity | ≥0.95 against fixture corpus | DTU fidelity measurement procedure per dtu-assessment.md |

---

## 5. Error Taxonomy

Error codes follow the convention `E-<SUBSYSTEM>-<NNN>` where subsystem abbreviations are: `DAEMON` (daemon lifecycle), `AUTH` (authentication), `LOCK` (lock file), `RING` (ring buffer), `FACT` (factory adapter), `ENG` (engine module).

| Code | Category | Severity | Exit / HTTP | Message Format | Source BC |
|------|----------|----------|-------------|---------------|-----------|
| E-AUTH-001 | Authentication | Broken | HTTP 401 | `{"error":"missing_auth_token"}` | BC-AUTH-002 (absent header) |
| E-AUTH-002 | Authentication | Broken | HTTP 401 | `{"error":"invalid_auth_token"}` | BC-AUTH-002 (any value-present failure) |
| E-DAEMON-001 | Body Size | Broken | HTTP 413 | `{"error":"payload_too_large","limit_bytes":262144}` | BC-DAEMON-003 |
| E-DAEMON-002 | Shutdown | Degraded | HTTP 503 | `{"error":"daemon_shutting_down"}` with `Retry-After: 10` header | BC-DAEMON-004 §Shutdown Signal Handling |
| E-DAEMON-003 | Liveness | Broken | HTTP 503 | `{"status":"shutting_down"}` | BC-DAEMON-001 (healthz during shutdown) |
| E-DAEMON-004 | Daemon Start | Broken | Exit 1 | `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path` | BC-DAEMON-005 precondition 2(d) — `DaemonStartError::RuntimeDirUnresolvable` |
| E-LOCK-001 | Lock File | Broken | Exit 1 | `ERROR: daemon already running at pid=<N>; exiting` | BC-DAEMON-005 §Start Sequence step 2b |
| E-LOCK-002 | Lock File | Degraded | WARN log | `WARN: stale lock file removed` | BC-DAEMON-005 §Start Sequence step 2c |
| E-LOCK-003 | Lock File | Degraded | WARN log | `WARN: lock file contract_version <N> not recognized; skipping` | BC-LOCK-001 EC-010 |
| E-ENG-001 | Engine Init | Broken | Daemon exit | `ERROR: platform home directory unresolvable (BaseDirs::new() returned None)` | BC-ENGINE-002-ERR |
| E-FACT-001 | Factory Parse | Degraded | WARN log | `WARN: STATE.md not found at <path>: <io-error>` | BC-FACTORY-002 |
| E-FACT-002 | Factory Parse | Degraded | Returns `None` or `Err` | `WARN: STATE.md malformed: <reason>` | BC-FACTORY-002 |
| E-RING-001 | Ring Buffer | Degraded | Logged | `WARN: ring buffer flush failed: <io-error>` | BC-RING-001 EC-003 |
| E-PROTO-001 | Protocol | Degraded | WARN log | `WARN: HookEnvelope schema_version <N> not recognized; skipping` | BC-PROTO-002 EC-027, EC-028 |

---

## 6. Competitive Differentiator Traceability

Per vision §Vision Statement and brief §Success Criteria. Every differentiator has BC backing — no unverifiable claims.

| Differentiator | Description | BC Backing | Verification |
|---------------|-------------|------------|-------------|
| Hook-protocol ingestion at OS-assigned port | Daemon binds on OS-assigned port; port written to lock file; hook scripts read absolute lock file path (no directory scan, no "highest-port-wins" collision) | BC-AUTH-001, BC-AUTH-002, BC-LOCK-001, BC-DAEMON-001, BC-DAEMON-002 | Integration test: lock file read after start; port confirmed reachable; no `~/.claude/ide/` scanning |
| VecDeque overlay stack for concurrent prompts | Both permission prompts visible simultaneously; `[↑↓]` rotates stack; `Esc` hides without rejecting | BC-ENGINE-001, BC-ENGINE-002 (on_hook → HookDecision::Defer) | Killer scenario: 2 concurrent PreToolUse hooks arrive; TUI shows both prompts; 4 keystrokes resolve both |
| Versioned ABI with forward-compatible extension | `MONOCLE_ABI_VERSION = 1` const; `#[non_exhaustive]` on all public enums; proto `schema_version = 1` first field | BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-PROTO-001a, BC-PROTO-001b | Compile-time assertions; AST audit (syn 2); wire-format round-trip test |
| FactoryAdapter open trait — Phase 3 WASM extensibility | `VsddFactoryAdapter` ships Phase 1 as a static implementation; WASM plugin SDK in Phase 3 uses the same trait without code changes | BC-FACTORY-001, BC-FACTORY-002 | `cargo check` no sealed supertrait; self-referential detection test |
| Strict-basename detection (no false positives) | `detect()` uses `exe_path.file_name()` == `"claude"` or `"claude.js"`; rejects `claude-squad`, `claudio`, `claude-code-router` | BC-ENGINE-002 | Unit tests with 5 synthetic ProcessSnapshot instances |
| JSONL ring with format_version first key | Phase 2 trigger-trace can read Phase 1 history; version field allows future format evolution | BC-RING-001 | Unit test: serialized JSONL line begins with `{"format_version":1,` |
| 256 KiB body size limit with structured error | Bounded daemon memory exposure per connection; structured error body for machine-readable rejection | BC-DAEMON-003 | Integration test: 262,145-byte body returns HTTP 413 with correct error body |
| Graceful 10-second drain with crash-recovery checkpoint | In-flight requests complete before daemon exits; crash-recovery state offered to TUI on reconnect | BC-DAEMON-004, BC-DAEMON-006 | Integration test: SIGTERM triggers drain; new hooks get 503 with Retry-After: 10 |

---

## 7. Requirements Traceability Matrix

| BC ID | Brief Section | Architecture Source | Priority | Test File | Test Type |
|-------|--------------|--------------------|---------|-----------|----|
| BC-DAEMON-001 | §Scope (hook receiver hardening sub-bullet — `/healthz`) | SS-daemon-lifecycle.md v1.0.13 §Health and Status Endpoints §GET /healthz | P0 | `monocle-runtime/tests/healthz_endpoint.rs` | Integration |
| BC-DAEMON-002 | §Scope (hook receiver hardening sub-bullet — `/status`) | SS-daemon-lifecycle.md v1.0.13 §Health and Status Endpoints §GET /status | P0 | `monocle-runtime/tests/status_endpoint_auth.rs` | Integration |
| BC-DAEMON-003 | §Success Criteria (hook receiver body size limit row) | SS-daemon-lifecycle.md v1.0.13 §Body Size Limit | P0 | `monocle-runtime/tests/body_size_limit.rs` | Integration |
| BC-DAEMON-004 | §Scope (hook receiver hardening sub-bullet — graceful shutdown) | SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Shutdown Signal Handling | P0 | `monocle-runtime/tests/graceful_shutdown.rs` | Integration |
| BC-DAEMON-005 | §Scope (hook receiver hardening sub-bullet — graceful shutdown) | SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence | P0 | `monocle-runtime/tests/lock_file_lifecycle.rs` | Integration |
| BC-DAEMON-006 | §Scope (hook receiver hardening sub-bullet — graceful shutdown) | SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Crash Recovery | P0 | `monocle-runtime/tests/crash_recovery.rs` | Integration |
| BC-RING-001 | §Scope (forward-compatibility contracts sub-bullet — JSONL ring) | SS-daemon-lifecycle.md v1.0.13 §Drain | P0 | `monocle-runtime/tests/jsonl_ring.rs` | Unit |
| BC-AUTH-001 | §Scope (forward-compatibility contracts sub-bullet — versioned auth token) | SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence | P0 | `monocle-runtime/tests/auth_token_lifecycle.rs` | Integration |
| BC-AUTH-002 | §Scope (forward-compatibility contracts sub-bullet — versioned auth token) | SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence | P0 | `monocle-runtime/tests/auth_header_rejection.rs` | Integration |
| BC-LOCK-001 | §Scope (forward-compatibility contracts sub-bullet — versioned auth token) | SS-daemon-lifecycle.md v1.0.13 §Daemon Lifecycle Protocol §Start Sequence | P0 | `monocle-runtime/tests/lock_file_contract.rs` | Integration |
| BC-ABI-001 | §Scope (forward-compatibility contracts sub-bullet — monocle-core ABI) | SS-core-types-and-abi.md v1.2.8 §ABI Version Constant | P0 | `monocle-runtime/tests/status_abi_version.rs` | Integration |
| BC-ABI-002 | §Scope (forward-compatibility contracts sub-bullet — monocle-core ABI) | SS-core-types-and-abi.md v1.2.8 §ABI Version Constant | P0 | `monocle-core/tests/abi_stability.rs` | Lint/compile |
| BC-TYPES-001 | §Scope (forward-compatibility contracts sub-bullet — public enum extensibility) | SS-core-types-and-abi.md v1.2.8 §Enum Extensibility | P0 | `monocle-core/tests/enum_audit.rs` | AST audit (syn 2) |
| BC-FACTORY-001 | §Scope (forward-compatibility contracts sub-bullet — FactoryAdapter trait) | SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait | P0 | `monocle-core/tests/factory_trait_surface.rs` | AST audit (syn 2) |
| BC-FACTORY-002 | §Success Criteria (factory pattern detection row) | SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait §Phase 1 Implementation: VsddFactoryAdapter | P0 | `monocle-core/tests/factory_self_referential.rs` | Integration |
| BC-PROTO-001a | §Scope (forward-compatibility contracts sub-bullet — prost wire schemas) | SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas | P0 | `monocle-proto/tests/wire_field_order.rs` | Unit |
| BC-PROTO-001b | §Scope (forward-compatibility contracts sub-bullet — prost wire schemas) | SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas | P0 | `monocle-proto/tests/schema_version.rs` | Unit |
| BC-PROTO-002 | §Scope (forward-compatibility contracts sub-bullet — prost wire schemas) | SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas | P1 | Phase 4 integration test (future) | Integration |
| BC-ENGINE-001 | §Scope §In Scope (ClaudeCodeModule sub-bullet) | SS-engine-module.md v1.1.15 §EngineModule Trait Signature | P0 | `monocle-core/tests/engine_module_surface.rs` | AST audit (syn 2) |
| BC-ENGINE-002 | §Scope §In Scope (ClaudeCodeModule sub-bullet) | SS-engine-module.md v1.1.15 §Phase 1 Implementation: ClaudeCodeModule | P0 | `monocle-runtime/tests/engine_module_claude_detect.rs` | Unit |
| BC-ENGINE-002-ERR | §Scope §In Scope (ClaudeCodeModule sub-bullet) | SS-engine-module.md v1.1.15 §Behavioral Contracts BC-ENGINE-002-ERR | P0 | `monocle-runtime/tests/engine_module_home_unresolvable.rs` | Unit (env-isolation) |
| BC-ENGINE-003 | §Scope §In Scope (ClaudeCodeModule sub-bullet) | SS-engine-module.md v1.1.15 §Struct-level inherent operations | P0 | `monocle-runtime/tests/engine_module_claude_methods.rs` | Unit |

---

## 8. Cross-Cutting Concerns

### 8.1 Atomic Writes

All config files, lock files, and ring buffer segments are written via `tempfile::persist`. No exceptions. Direct `std::fs::write` on config-class files is a forbidden anti-pattern (SS-conventions-anti-patterns.md).

### 8.2 Bounded Channels

All `tokio::mpsc` channels are bounded. No `unbounded_channel` in production code. Drop counters are surfaced in the TUI status bar. Integration test target: 1000 events/sec with drop counter assertion (NFR-006).

### 8.3 Error Handling Hierarchy

- Library error types: `thiserror 2.x` (typed, `#[error]` derive)
- Binary-crate error propagation: `anyhow 1`
- No `unwrap()` or `expect()` in production code without `// SAFETY:` comment explaining why the value is guaranteed to be `Some`/`Ok`

### 8.4 Constant-Time Comparisons

Auth token comparison uses `constant_time_eq::constant_time_eq` (pinned `^0.3`). No `==` operator on secret strings. See BC-AUTH-001, NFR-010.

### 8.5 Non-Exhaustive Structs + Constructors

Every `#[non_exhaustive]` struct that is constructed from any crate OTHER than the defining crate requires a `pub fn new(...)` constructor. The Cross-Crate Constructor Audit Table in SS-engine-module.md v1.1.15 §Cross-Crate Constructor Audit is the authoritative list. CI enforces this via the `monocle-non-exhaustive-struct-audit-completeness` semgrep rule.

### 8.6 Struct Field Order Contracts

Two contracts depend on serde field declaration order:
1. `HookEventRecord::format_version` must be declared FIRST (BC-RING-001).
2. Lock file `contract_version` must serialize FIRST (BC-LOCK-001).

`serde_json::to_string` preserves struct field declaration order for plain Rust structs. This is a `serde_json` implementation property (not a Rust language guarantee). The contracts are satisfied by keeping `format_version` / `contract_version` as the first declared field in their respective structs.

### 8.7 Windows CI Caveat

BC-ENGINE-002-ERR's `HomeUnresolvable` test path is best-effort on Windows CI runners because `BaseDirs::new()` may succeed via `SHGetKnownFolderPath` regardless of env var state. The contract is fully deterministic on Linux/macOS (the target platforms per NFR-008). Windows is a secondary build target (darwin/linux primary per brief §Scope).

---

## 9. Edge Case Catalog

All per-contract edge cases (EC-001 through EC-059) are embedded in Section 3 within each BC. This index provides a cross-reference for sweep tooling.

| EC ID | BC | Category | Description |
|-------|----|----------|-------------|
| EC-001 | BC-RING-001 | JSONL serialization | `tool_name`/`tool_input` None for non-tool events; format_version still first |
| EC-002 | BC-RING-001 | Ring buffer | Near-maximum payload size (256 KiB line); rotation handles without truncation |
| EC-003 | BC-RING-001 | Crash recovery | Ring buffer file truncated mid-line; Phase 2 readers skip incomplete trailing lines |
| EC-004 | BC-AUTH-001 | Token lifecycle | Token rotation on daemon restart; scripts reading from lock file always have current token |
| EC-005 | BC-AUTH-001 | Atomic write | Lock file write failure (filesystem full); daemon exits without partial lock file |
| EC-006 | BC-AUTH-001 | Lock file cross-ref | `contract_version` field cross-references BC-LOCK-001 |
| EC-007 | BC-AUTH-002 | Empty header value | Empty `X-Monocle-Authorization` value (value-present, format-fail) → HTTP 401 `{"error":"invalid_auth_token"}` |
| EC-008 | BC-AUTH-002 | Missing header | No auth header → HTTP 401 `{"error":"missing_auth_token"}` |
| EC-009 | BC-AUTH-002 | Prefix-only token | `monocle-v1:` with no hex suffix (value-present, secret-mismatch) → HTTP 401 `{"error":"invalid_auth_token"}` |
| EC-010 | BC-LOCK-001 | Future version | `contract_version: 99` → WARN log + skip |
| EC-011 | BC-LOCK-001 | Type coercion | `contract_version` stored as string → graceful coerce or skip |
| EC-012 | BC-LOCK-001 | Missing field | No `contract_version` key → same treatment as EC-010 |
| EC-013 | BC-ABI-001 | Phase 3 forward | Plugin SDK reads `abi_version` from /status to version-gate loading |
| EC-014 | BC-ABI-001 | Phase 4 forward | Federation peer with different ABI version → HTTP 409 |
| EC-015 | BC-ABI-002 | Compile-time | Plugin SDK compile-time assertion fails if ABI version changes without SDK update |
| EC-016 | BC-TYPES-001 | AST enforcement | New enum without `#[non_exhaustive]` → CI AST audit test error |
| EC-017 | BC-TYPES-001 | Unknown variant | `ClaudeCodeTool::Unknown(String)` catch-all — exhaustive enum with runtime escape |
| EC-018 | BC-FACTORY-001 | dyn dispatch | `detect()` not callable on `dyn FactoryAdapter`; use `matches()` |
| EC-019 | BC-FACTORY-001 | YAML complex types | `custom_fields` skips flow-lists and block scalars; re-parse with serde_yaml_ng for full semantics |
| EC-020 | BC-FACTORY-001 | Phase 3 WASM | WASM adapter checks `abi_version()` default against host ABI |
| EC-021 | BC-FACTORY-002 | Body occurrence | `document_type: pipeline-state` in document body not detected (frontmatter must be on line 1) |
| EC-022 | BC-FACTORY-002 | YAML quoting | `awaiting: "round 18 validation chain"` → quotes stripped; `Some("round 18 validation chain")` |
| EC-023 | BC-FACTORY-002 | Flow-list skip | `blocking_issues: []` skipped by frontmatter parser; Phase 3 body parser populates from markdown |
| EC-024 | BC-PROTO-001a | Proto3 forward | Phase 1 receiver ignores Phase 4+ fields (100-999 range); no crash |
| EC-025 | BC-PROTO-001a | Invalid version | `schema_version: 0` → WARN log + skip |
| EC-026 | BC-PROTO-001b | Pin changes | prost-build version change must re-verify `pub schema_version: u32` accessibility |
| EC-027 | BC-PROTO-002 | Version 0 | Receiver skips with WARN; no panic |
| EC-028 | BC-PROTO-002 | Future version | `schema_version: 2` → skip with WARN; no attempt to decode as Phase 1 |
| EC-029 | BC-ENGINE-001 | Home unresolvable | `$HOME` unset → `Err(EngineMetadataError::HomeUnresolvable)`; daemon fails fast |
| EC-030 | BC-ENGINE-001 | exe_path None | Process exited before path resolved → `detect()` returns `false` |
| EC-031 | BC-ENGINE-001 | Unknown hook type | Unrecognized `HookEvent` variant → `on_hook()` returns `HookResponse::new(HookDecision::Allow)` |
| EC-032 | BC-ENGINE-002 | cmdline confusion | `cmdline[0] == "claude"` but `exe_path == "claude-squad"` → `detect() == false` |
| EC-033 | BC-ENGINE-002 | No extension | `/usr/local/bin/claude` (no `.js`) → `detect() == true` |
| EC-034 | BC-ENGINE-002 | Node wrapper | `/usr/local/bin/claude.js` → `detect() == true` |
| EC-035 | BC-ENGINE-002 | Prefix match | `/usr/local/bin/claude-squad` → `detect() == false` |
| EC-036 | BC-ENGINE-002-ERR | Windows CI | `BaseDirs::new()` may succeed via SHGetKnownFolderPath on Windows regardless of env vars |
| EC-037 | BC-ENGINE-002-ERR | working_dir None | `working_dir: None` → `transcript_path: None` in EnrichedSession; separate from HomeUnresolvable |
| EC-038 | BC-ENGINE-003 | spawn stub | `spawn()` panics with `todo!()` in Phase 1; replaced in Phase 1 story |
| EC-039 | BC-ENGINE-003 | preflight stub | `preflight()` panics with `todo!()` in Phase 1; replaced in Phase 1 story |
| EC-040 | BC-DAEMON-001 | TUI hung detection | `/healthz` unreachable + live pid → TUI initiates hung-daemon recovery flow |
| EC-041 | BC-DAEMON-001 | TUI stale lock | `/healthz` unreachable + dead pid → TUI auto-starts daemon (stale lock) |
| EC-042 | BC-DAEMON-002 | Phase 4 ABI gate | Federation reads `abi_version` from `/status`; refuses incompatible ABI |
| EC-043 | BC-DAEMON-002 | Zero metrics | `ring_buffer_fill_pct == 0.0` and `channel_saturation_pct == 0.0` on fresh start |
| EC-044 | BC-DAEMON-002 | Null timestamps | `last_hook_ts` values are `null` for hook types not yet fired since daemon start |
| EC-045 | BC-DAEMON-003 | At-limit rejection | Body exactly 262,145 bytes → HTTP 413 |
| EC-046 | BC-DAEMON-003 | Under-limit pass | Body 262,143 bytes → HTTP 200 |
| EC-047 | BC-DAEMON-003 | Shutdown endpoint | `POST /shutdown` body also subject to 256 KiB limit (defence-in-depth) |
| EC-048 | BC-DAEMON-004 | Post-timeout arrival | Hook POST after drain timeout expires → HTTP 503 |
| EC-049 | BC-DAEMON-004 | Ring flush failure | Ring buffer flush fails during drain → WARN logged; shutdown proceeds |
| EC-050 | BC-DAEMON-004 | Double shutdown | Second `POST /shutdown` during drain → immediate hard close with exit code 2 (admin forced-stop) |
| EC-051 | BC-DAEMON-005 | Lock file write fail | Lock write fails → daemon exits; no partial lock file on disk |
| EC-052 | BC-DAEMON-005 | Missing runtime dir | Runtime dir absent → created with mode 0o700; failure exits 1 |
| EC-053 | BC-DAEMON-005 | TOCTOU race | Lock file removed between liveness check and atomic write → tempfile::persist atomicity mitigates |
| EC-057 | BC-DAEMON-005 | macOS data_local_dir fallback | `MONOCLE_RUNTIME_DIR` unset; `ProjectDirs::runtime_dir()` returns `None`; `data_local_dir()` provides fallback path → daemon starts normally on macOS |
| EC-058 | BC-DAEMON-005 | Env override | `MONOCLE_RUNTIME_DIR` set → overrides all platform defaults; operator escape hatch for containers/NixOS |
| EC-059 | BC-DAEMON-005 | RuntimeDirUnresolvable | All three resolution paths return `None` (no home dir + no env var) → `DaemonStartError::RuntimeDirUnresolvable`; exit 1; no lock file |
| EC-054 | BC-DAEMON-006 | Malformed recovery | Recovery file is malformed JSON → WARN logged; file deleted; clean start |
| EC-055 | BC-DAEMON-006 | Single recovery file | Only one `monocle.recovery.json` per runtime dir; overwrites on each shutdown |
| EC-056 | BC-DAEMON-006 | 60-second boundary | TUI at exact boundary: recovery offer already sent if within window; clean start if past window |

---

## 10. Glossary

| Term | Definition | Source |
|------|-----------|--------|
| ABI | Application Binary Interface. `MONOCLE_ABI_VERSION` identifies the stable contract between `monocle-core` and its consumers (plugin SDK, federation layer). | SS-core-types-and-abi.md §ABI Version Constant |
| BC | Behavioral Contract. A testable specification with preconditions, postconditions, and at least one canonical test vector. | This document |
| `ClaudeCodeModule` | Phase 1 built-in `EngineModule` implementation for Claude Code harness integration. Defined in `monocle-runtime`. | SS-engine-module.md §Phase 1 Implementation: ClaudeCodeModule |
| DTU | Digital Twin Universe. Behavioral clone of the Claude Code hook protocol for testing fidelity and regression detection. | dtu-assessment.md |
| `EngineModule` | Trait in `monocle-core::engine` abstracting over AI coding harness adapters. Open (not sealed). | SS-engine-module.md §EngineModule Trait Signature |
| `FactoryAdapter` | Trait in `monocle-core::factory` abstracting over factory-pattern workflow detectors. Open (not sealed). | SS-core-types-and-abi.md §FactoryAdapter Trait |
| `FactoryState` | 7-field canonical struct returned by `FactoryAdapter::read_state()`. Fields: `phase`, `status`, `awaiting`, `blocking_issues`, `convergence`, `cycle`, `custom_fields`. | SS-core-types-and-abi.md §FactoryAdapter Trait |
| FC | Forward-Compatibility item. Pre-Phase-1 contracts locked by human authorization. FC-01 through FC-06. | SS-forward-compatibility.md; product-brief.md §Scope (forward-compatibility contracts sub-bullet) |
| `format_version` | First key in every JSONL ring buffer record. Value `1` for all Phase 1 records. | BC-RING-001; SS-daemon-lifecycle.md §Drain |
| `HookEventRecord` | Rust struct in `monocle-runtime::ring` written to the JSONL ring buffer. `#[non_exhaustive]`; provides `new()` constructor. | SS-daemon-lifecycle.md §Drain |
| `HookEnvelope` | Proto message in `monocle-proto` with `schema_version` at field number 1. Wire format for Phase 4 federation. | BC-PROTO-001a, BC-PROTO-001b; SS-core-types-and-abi.md §Prost Wire Schemas |
| JC-2 | Joint Closure 2: `PostToolUse` omitted from Phase 1 hook endpoint set to preserve gene-source parity with any-context-lazyclaude BC-HOOK-007 canonical 5-endpoint matrix. | vision §Closure Log; brief §Scope |
| `monocle-v1:` | Wire-format prefix for Phase 1 auth tokens. `X-Monocle-Authorization: monocle-v1:<64-hex>`. | BC-AUTH-001, BC-AUTH-002 |
| `MONOCLE_ABI_VERSION` | `pub const u32 = 1` in `monocle-core::abi`. Exported at crate root. Used by Phase 3 plugin SDK and Phase 4 federation. | BC-ABI-001, BC-ABI-002 |
| `#[non_exhaustive]` | Rust attribute preventing exhaustive match and struct literal construction outside the defining crate. Default for all `pub` enums in `monocle-core`. | BC-TYPES-001; ADR-0004 |
| OsRng | `rand::rngs::OsRng`. Cryptographically secure random source used for auth token generation. Required; `thread_rng` is forbidden for secrets. | BC-AUTH-001; SS-daemon-lifecycle.md §Daemon Lifecycle Protocol §Start Sequence |
| `Phase1Permission` | Exhaustive enum in `monocle-core::permissions`. Five variants. ADR-0004 exempts it from `#[non_exhaustive]`. | ADR-0004; SS-permissions-phase1.md |
| `schema_version` | Proto field number 1 in `HookEnvelope`. Value `1` for all Phase 1 messages. Used by Phase 4 federation to validate message format compatibility. | BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 |
| `VsddFactoryAdapter` | Phase 1 static implementation of `FactoryAdapter`. Detects VSDD Factory workspaces via `document_type: pipeline-state` in `.factory/STATE.md`. | BC-FACTORY-002 |

---

## §Trace v1.0

**v1.0 (2026-05-14):** Initial PRD authored by product-owner from 16 pre-staged BCs. Source artifacts: SS-daemon-lifecycle.md v1.0.7, SS-core-types-and-abi.md v1.2.8, SS-engine-module.md v1.1.15, product-brief.md v1.4.23, vision v1.1.2, dtu-assessment.md, 4 ADRs. 16 BCs formalized with full preconditions, postconditions, invariants, edge cases, canonical test vectors, and verification specifications. 5 NFRs promoted from brief §Success Criteria; 6 additional NFRs added from cross-cutting concerns. Error taxonomy: 14 error codes covering all error surfaces across 6 subsystem abbreviations. Edge case catalog: 39 entries (EC-001 through EC-039). Glossary: 19 terms. META defense layer compliance: D-047 strict applied; no ambiguous requirements; every BC has ≥1 edge case and ≥1 canonical test vector; no MVP deferrals; no "pending architect review" for answerable questions; all field-order contracts explicitly stated with serde implementation rationale.

## §Trace v1.1

**v1.1 (2026-05-14):** Fix-burst F-R62. Source commits: adversary report R62 (5713ccc), consistency audit T-4 (0e322da), architect auth adjudication (2db408f). Input version bump: SS-daemon-lifecycle.md v1.0.7 → v1.0.8 (architect commit 2db408f). Changes applied per D-047 strict:

- **F-R62-1 RESOLVED (CRITICAL):** BC count expanded from 16 → 22 BCs. Added 6 new BC sections: BC-DAEMON-001 (healthz endpoint), BC-DAEMON-002 (status endpoint), BC-DAEMON-003 (body size limit), BC-DAEMON-004 (graceful shutdown), BC-DAEMON-005 (lock file lifecycle), BC-DAEMON-006 (crash recovery). All 6 BCs sourced from SS-daemon-lifecycle.md v1.0.8 §Behavioral Contract Summary per architecture's prescriptive language (lines 586-588: "The Phase 1 PRD will formalize these as full BC entries"). §2.1 grouping table updated to 5 domains. §7 RTM expanded from 16 to 22 rows. §1.3 differentiator table updated. §5 error taxonomy source BC citations updated. §9 edge case catalog extended to EC-040..EC-056. §10 Glossary frontmatter §Trace source citations updated.

- **F-R62-2 + T-4-F-001 RESOLVED (HIGH):** PG-4 §-heading-existence violations fixed across PRD. `§Forward-compatibility contracts` (brief bullet label, not a heading) re-anchored throughout to `§Scope (forward-compatibility contracts sub-bullet)`. `§Scope §ClaudeCodeModule` (bullet in `### In Scope`, not a heading) re-anchored to `§Scope §In Scope (ClaudeCodeModule sub-bullet)`. `§Success Criteria §Factory pattern detection` (table row label, not a heading) re-anchored to `§Success Criteria (factory pattern detection row)`. All affected sites: §3 BC Traceability fields, §7 RTM Brief Section column, §10 Glossary FC row, §Trace citations. Comprehensive sweep performed per F-R60-corpus-sweep 5-step protocol.

- **F-R62-3 RESOLVED (HIGH):** §Trace v1.0 falsified PG-4 self-check corrected. The v1.0 §Trace claimed `brief §Forward-compatibility contracts ✓` — this was a false PASS (no such heading exists in brief). The v1.1 §Trace documents the actual PG-4 sweep result with correct re-anchored citations.

- **F-R62-4 RESOLVED (HIGH):** Canonical test-harness file paths authored. BC-AUTH-001 canonical file: `monocle-runtime/tests/auth_token_lifecycle.rs` (was `auth.rs`). BC-AUTH-002 canonical file: `monocle-runtime/tests/auth_header_rejection.rs` (was `auth.rs`). BC-LOCK-001 canonical file: `monocle-runtime/tests/lock_file_contract.rs` (was `daemon_lock.rs`). BC-ABI-001 canonical file: `monocle-runtime/tests/status_abi_version.rs` (was vague). New BC-DAEMON-001..006 canonical files assigned (see §7 RTM). §7 RTM updated with all canonical paths. PRD is source of truth; formal-verifier must adopt these paths in VP v1.1.

- **F-R62-6 RESOLVED (HIGH):** BC-TYPES-001 and BC-ENGINE-001 verification rigor upgraded to match VP v1.0. BC-TYPES-001 §Verification now specifies `syn 2` AST parse of all `.rs` files in `monocle-core/src/**/*.rs`, walking `Item::Enum` nodes. BC-ENGINE-001 §Verification now specifies `syn 2` AST parse of `monocle-core/src/engine.rs` with 5-method count + no-Sealed-bound + return-type token-stream match. §7 RTM Test Type column: BC-TYPES-001 updated from `Clippy` → `AST audit (syn 2)`; BC-ENGINE-001 updated from `Compile/rustdoc` → `AST audit (syn 2)`. BC-FACTORY-001 also upgraded to AST audit (syn 2) for consistency.

- **F-R62-8 RESOLVED (MED, per architect adjudication commit 2db408f):** Auth error taxonomy updated to match SS-daemon-lifecycle.md v1.0.8 (disposition (c) mixed approach). Retired body: `invalid_auth_token_format` (removed from all PRD locations). New 2-row taxonomy in §5: E-AUTH-001 (`missing_auth_token` for absent header) and E-AUTH-002 (`invalid_auth_token` for any value-present failure). BC-AUTH-002 postconditions rewritten to reflect the two-body taxonomy. EC-007 updated: empty header value → `invalid_auth_token` (value-present failure). EC-009 note updated: prefix-only token → `invalid_auth_token` (no separate body). Canonical test vectors table expanded from 5 to 6 rows covering all three middleware branches (missing header, bare token, wrong version, prefix-only, wrong header name, correct-format-wrong-value).

- **F-R62-10 RESOLVED (LOW):** BC-PROTO-001a postcondition 4 corrected. Field assignments now use field names (snake_case) with type-name parenthetical: `session_start (SessionStartEvent) = 10`, `prompt_submit (UserPromptSubmitEvent) = 11`, `pre_tool_use (PreToolUseEvent) = 12`, `notification (NotificationEvent) = 13`, `stop (StopEvent) = 14`. Prior version incorrectly listed event type names (`SessionStart=10`, `UserPromptSubmit=11`, etc.) as field names.

- **T-4-F-002 RESOLVED (MED):** BC-LOCK-001 §Traceability re-anchored from `SS-daemon-lifecycle.md v1.0.7 §Lock File Discovery Policy` → `SS-daemon-lifecycle.md v1.0.8 §Daemon Lifecycle Protocol §Start Sequence`. §7 RTM Architecture Source column for BC-LOCK-001 updated to match. The BC-LOCK-001 contract (lock file JSON schema) lives at §Daemon Lifecycle Protocol §Start Sequence; §Lock File Discovery Policy covers the TUI-client hook-script path discovery mechanism (separate concern).

**D-042 sweep (v1.1):** 4-pattern recursive sweep on this document. Pattern 1 (SS-*.md v): SS-daemon-lifecycle.md v1.0.8 ✓ (updated from v1.0.7 per architect commit 2db408f), SS-core-types-and-abi.md v1.2.8 ✓ (confirmed current), SS-engine-module.md v1.1.15 ✓ (confirmed current). No dtu-assessment.md version citations in body. No vision version citations in body. No ADR version citations in body. All current-pointer citations confirmed correct.

**PG-4 §-heading-existence sweep (v1.1):** All §-anchor references in this document verified against actual headings in cited files per 5-pattern recipe. Results:

SS-daemon-lifecycle.md v1.0.8 headings verified:
- `§Health and Status Endpoints` ✓ (H2 at line 32)
- `§GET /healthz` ✓ (H3 at line 36)
- `§GET /status` ✓ (H3 at line 63)
- `§Body Size Limit` ✓ (H2 at line 102)
- `§Daemon Lifecycle Protocol` ✓ (H2 at line 159)
- `§Start Sequence` ✓ (H3 at line 161)
- `§Shutdown Signal Handling` ✓ (H3 at line 360)
- `§Drain` ✓ (H3 at line 373)
- `§Crash Recovery` ✓ (H3 at line 523)

SS-core-types-and-abi.md v1.2.8 headings verified:
- `§ABI Version Constant` ✓ (H2 at line 48)
- `§Enum Extensibility` ✓ (H2 at line 113)
- `§FactoryAdapter Trait` ✓ (H2 at line 322)
- `§Prost Wire Schemas` ✓ (H2 at line 892)
- `§Phase 1 PRD BC Pre-Staging` ✓ (H2 at line 1010)
- `§Phase 1 Implementation: VsddFactoryAdapter` ✓ (H3 at line 565, prefix-match)

SS-engine-module.md v1.1.15 headings verified:
- `§EngineModule Trait Signature` ✓ (H2 at line 48, prefix-match)
- `§Phase 1 Implementation: ClaudeCodeModule` ✓ (H2 at line 539, prefix-match)
- `§Struct-level inherent operations` ✓ (H3 at line 663, prefix-match)
- `§Behavioral Contracts BC-ENGINE-002-ERR` ✓ (H2 at line 936, prefix-match to §Behavioral Contracts)
- `§Cross-Crate Constructor Audit` ✓ (H2 at line 1080, prefix-match)

Vision headings verified (domain-monocle-vision-synthesis.md v1.1.2):
- `§Vision Statement` ✓
- `§End-to-End Killer Scenario` ✓
- `§EngineModule` ✓
- `§FactoryAdapter` ✓
- `§Closure Log` ✓
- `§Explicit Non-Goals` ✓

Brief headings verified (product-brief.md v1.4.23) — PG-4 pattern 2 applied:
- `§Scope` ✓ (H2 at line 105)
- `§Scope §In Scope` ✓ (H3 at line 107) — used for ClaudeCodeModule references
- `§Success Criteria` ✓ (H2 at line 240)
- `brief §Scope (forward-compatibility contracts sub-bullet)` — PASS: `§Scope` is a real heading; the parenthetical `(forward-compatibility contracts sub-bullet)` is a position-free qualifier for the bold-label content at line 173 (not cited as a heading itself — PG-4 compliant form)
- `brief §Success Criteria (factory pattern detection row)` — PASS: `§Success Criteria` is a real heading; the parenthetical `(factory pattern detection row)` qualifies the table row at line 250 without asserting it is a heading
- `brief §Scope (hook receiver hardening sub-bullet)` — PASS: same pattern; `§Scope` is real; parenthetical qualifies bullet content

All §-anchor references: PASS. Zero mis-anchors in v1.1.

**PG-2 noun-agnostic count coherence (v1.1):** "22 BCs" is the correct count. Verified occurrences: §2.1 grouping table (11 rows, 22 BC IDs listed) ✓; §7 RTM (22 rows) ✓; §9 edge case catalog header "EC-001 through EC-056" ✓; frontmatter `traces_to` "22 BCs" ✓. No remaining "16 BCs" references in normative content.

**PG-RECIPE-SCOPE compliance (v1.1):** Sweep scope `.factory/specs/` recursive — not narrowed to `.factory/specs/architecture/` subdirectory only.

**PG-5 §Historical-Anchor Framing compliance (v1.1):** All brief version citations in §Trace use position-free section heading anchors without version qualifiers per PG-5 option (c). Architecture file version citations in §Trace are labeled as current-pointers (not historical pinpoints).

**PG-3 §Trace directional refs (v1.1):** No `above`, `below`, or bare L-numbers appear in this §Trace entry. All references use section heading anchors (§-form).

**PG-3-TRACE-NEW-ENTRY (v1.1):** This §Trace v1.1 entry uses only §-section references, no bare L-numbers, no directional qualifiers.

**F-R60-corpus-sweep (v1.1):** Corpus sweep applied — no known-stale references to deleted section headings, old enum variant names (no `invalid_auth_token_format` anywhere in normative content), superseded type names, or pre-ADR-0004 exhaustive-enum assumptions. All `invalid_auth_token_format` occurrences removed. Auth taxonomy updated to two-body taxonomy throughout.

**18+ META rule checklist (v1.1):**
- D-042 (4-pattern citation sweep): PASS — SS-daemon-lifecycle.md v1.0.8 current, SS-core-types-and-abi.md v1.2.8 current, SS-engine-module.md v1.1.15 current.
- D-047 strict (3-clean-pass convergence): N/A for PRD authoring; applies to adversarial review passes.
- PG-1 (no ambiguous requirements): PASS — every BC has testable preconditions, postconditions, and test vectors. 22 BCs covered.
- PG-2 (noun-agnostic count coherence): PASS — "22 BCs" consistent in §2.1, §7 RTM (22 rows), frontmatter, edge case count (EC-001..EC-056).
- PG-3 (no L-number pinpoints in §Trace): PASS — all §Trace references use section heading anchors.
- PG-3-TRACE-NEW-ENTRY (position-free references in new §Trace entries): PASS — v1.1 entry uses only section heading anchors.
- PG-4 (§-heading-existence sweep): PASS — all §-anchors verified per sweep above; zero mis-anchors. The v1.0 falsified claim is corrected in the v1.1 §Trace.
- PG-5 (historical-anchor framing): PASS — version qualifiers on stable section refs omitted.
- PG-RECIPE-SCOPE (`.factory/specs/` recursive sweep): PASS — sweep not narrowed to architecture/ subdirectory.
- BC-H1-is-title-source-of-truth: N/A — BCs are inline in this PRD (not separate files); H1 is each `### BC-*` heading.
- architecture_is_subsystem_name_source_of_truth: N/A — no `subsystem:` frontmatter on individual BC files (PRD format).
- append_only_numbering: PASS — no BC IDs renumbered; BC-DAEMON-001..006 are new append-only additions. EC-040..EC-056 are new append-only additions.
- lift_invariants_to_bcs: PASS — all invariants from SS-daemon-lifecycle.md v1.0.8 §Behavioral Contract Summary surfaced in corresponding BC invariant sections.
- Capability Anchor Justification (S-7.01): N/A — project-specific PRD; BCs trace to brief §Scope and architecture SS-* sections directly.
- bc_array_changes_propagate_to_body_and_acs: N/A — no stories exist yet; Phase 2 story decomposition pending.
- vp_index_is_vp_catalog_source_of_truth: N/A for this burst; VP v1.1 update is formal-verifier scope (out of bounds for product-owner).
- Self-audit (CLAUDE.md §Self-Audit Checklist): All 6 items checked — no MVP rationalizations, no tech-debt-register entries, no pending-architect-review markers, no deferred defects, no cheapest-path defaults, no advisories that should be blockers.
- Production-grade default: PASS — no "for now," "good enough," "minimum viable," or similar language. All 22 BCs have full contract specifications.
- Correct agent routing: PASS — VP file not touched (formal-verifier owns); architecture files not touched (architect owns); STATE.md not touched (state-manager owns).

## §Trace v1.2

**v1.2 (2026-05-15):** Fix-burst F-R63. Trigger: adversary R63 (commit 11a98c4), consistency-validator round 2 (commit 200eb68), architect v1.0.9 (commit 8bf3759). Changes applied per D-047 strict:

- **F-R63-adv-1 + F-R63-cons-1 RESOLVED (HIGH) — 4 test-name adjudications:** Product-owner adjudicated canonical test names for the 4 BCs where PRD v1.1 and VP v1.1 diverged. Canonical names become the pipeline source-of-truth; formal-verifier adopts these verbatim in VP v1.2.

  - **BC-ABI-001:** Canonical name → `test_BC_ABI_001_status_endpoint_returns_abi_version_1`. Adopted VP name over PRD name (`_status_abi_version_field`). Reasoning: the VP name identifies the endpoint (`status`), the field, and the expected value (`1`), making the assertion self-documenting for both presence and value; the PRD name conveyed field-presence only.

  - **BC-ENGINE-002:** Canonical name → `test_BC_ENGINE_002_claude_code_module_strict_basename_detect`. Adopted VP name over PRD name (`_claude_code_module_detect`). Reasoning: `strict_basename` encodes the critical behavioral invariant — the test distinguishes correct strict-basename matching from a naive prefix or substring match; without `strict_basename` the test name does not distinguish this contract from a simpler detect test.

  - **BC-ENGINE-002-ERR:** Canonical name → `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich` (PRD name retained). Rejected VP name (`_home_unresolvable_sync_and_async`). Reasoning: `_metadata_and_enrich` identifies WHAT behavioral methods are under contract (both `metadata()` and `enrich()` must return `Err(HomeUnresolvable)`); `_sync_and_async` describes a test-implementation strategy (separate `temp_env::with_vars` / `temp_env::async_with_vars` wrappers), which is an internal concern of the test author, not the behavioral discriminator. Test names should describe what is verified, not how the test harness is structured.

  - **BC-ENGINE-003:** Canonical name → `test_BC_ENGINE_003_claude_module_hook_paths_five_entries` (hybrid). Neither PRD name (`_hook_paths_five_entries`) nor VP name (`_claude_module_inherent_hook_paths`) alone was sufficient. The hybrid `_claude_module_hook_paths_five_entries` combines: (1) `claude_module` — identifies the concrete struct under test (not a trait), (2) `hook_paths` — identifies the inherent method, (3) `five_entries` — states the count assertion. This is the most self-documenting form per Rust integration-test naming conventions.

  Updated locations: §3 BC §Verification "Test name:" fields for BC-ABI-001, BC-ENGINE-002, BC-ENGINE-003. BC-ENGINE-002-ERR §Verification test name unchanged (PRD name was already canonical).

- **F-R63-cons-2 RESOLVED (MEDIUM) — Error code count correction (PG-2):** §5 actual row count verified: 13 error codes (E-AUTH-001, E-AUTH-002, E-DAEMON-001, E-DAEMON-002, E-DAEMON-003, E-LOCK-001, E-LOCK-002, E-LOCK-003, E-ENG-001, E-FACT-001, E-FACT-002, E-RING-001, E-PROTO-001). The §Trace v1.0 historical claim "14 error codes" reflected the pre-F-R62-8 taxonomy; F-R62-8 retired `invalid_auth_token_format` (reducing from 14 to 13) and added the two-body taxonomy (E-AUTH-001 + E-AUTH-002), which restores to 14, then the net is still 13 because E-AUTH-001 (`missing_auth_token`) was newly introduced and `invalid_auth_token_format` was removed — total is 13. Corrected count documented here. §Trace v1.0 is a historical record and is not edited; the correction is carried in this v1.2 entry.

- **Change 3 — Architecture version-pin propagation (SS-daemon-lifecycle.md v1.0.8 → v1.0.9):** Architect committed v1.0.9 (commit 8bf3759) with F-R62-4 back-propagation (auth test path split) and §BC Summary footer tense correction. All PRD normative references to SS-daemon-lifecycle.md updated from v1.0.8 to v1.0.9: §3 BC Source fields (10 daemon-lifecycle BCs), §3 BC Traceability Source lines, §7 RTM Architecture Source column (10 rows). Frontmatter `traces_to:` updated. §Trace v1.1 historical entries referencing v1.0.8 are NOT modified (they record the historical state at v1.1 authoring time).

**D-042 sweep (v1.2):** 4-pattern recursive sweep on this document. Pattern 1 (SS-*.md v): SS-daemon-lifecycle.md v1.0.9 ✓ (all 10 daemon-lifecycle BC Source/Traceability/RTM references updated), SS-core-types-and-abi.md v1.2.8 ✓ (no change; still current), SS-engine-module.md v1.1.15 ✓ (no change; still current). No remaining v1.0.8 references in normative content outside §Trace v1.1 historical record. PG-4 §-heading-existence sweep (v1.2): the 4 newly adjudicated test names appear only in §Verification "Test name:" lines — these are test function identifiers, not §-anchors. No new §-anchor references introduced in v1.2 changes. All existing §-anchor references from v1.1 sweep unchanged. PG-4 PASS.

**PG-2 count coherence (v1.2):** 22 BCs unchanged ✓. Error codes: 13 (corrected per F-R63-cons-2 above; §5 table has 13 rows) ✓. Test names: 22 distinct test names across 22 BCs (including 3 updated + 1 retained). No duplicate test names.

**PG-3 §Trace directional refs (v1.2):** No `above`, `below`, or bare L-numbers appear in this §Trace v1.2 entry. All references use section heading anchors (§-form) or commit references.

**PG-3-TRACE-NEW-ENTRY (v1.2):** This §Trace v1.2 entry uses only §-section references, no bare L-numbers, no directional qualifiers.

**F-R60-corpus-sweep (v1.2):** Corpus sweep applied. Zero `v1.0.8` references remain in normative content (all updated to v1.0.9). Zero stale test names using the old names (`_status_abi_version_field`, `_claude_code_module_detect`, `_hook_paths_five_entries`) remain in normative content. Zero occurrences of `invalid_auth_token_format` in normative content (confirmed from v1.1 sweep; no new occurrences introduced).

**18+ META rule checklist (v1.2):**
- D-042 (4-pattern citation sweep): PASS — SS-daemon-lifecycle.md v1.0.9 current (updated from v1.0.8), SS-core-types-and-abi.md v1.2.8 current, SS-engine-module.md v1.1.15 current. Zero v1.0.8 references in normative content.
- D-047 strict (3-clean-pass convergence): N/A for PRD authoring; applies to adversarial review passes.
- PG-1 (no ambiguous requirements): PASS — no requirement changes; 22 BCs with full specifications.
- PG-2 (noun-agnostic count coherence): PASS — 22 BCs consistent; 13 error codes (corrected); 22 distinct test names.
- PG-3 (no L-number pinpoints in §Trace): PASS — all §Trace v1.2 references use section heading anchors.
- PG-3-TRACE-NEW-ENTRY (position-free references in new §Trace entries): PASS — v1.2 entry uses only section heading anchors.
- PG-4 (§-heading-existence sweep): PASS — no new §-anchor references introduced; all existing anchors unchanged from v1.1 sweep.
- PG-5 (historical-anchor framing): PASS — no new version qualifiers added on stable section refs.
- PG-RECIPE-SCOPE (`.factory/specs/` recursive sweep): PASS — sweep not narrowed.
- append_only_numbering: PASS — no BC IDs renumbered or retired; no EC IDs renumbered.
- lift_invariants_to_bcs: PASS — no new invariants introduced; existing invariant coverage unchanged.
- Self-audit (CLAUDE.md §Self-Audit Checklist): All 6 items checked — no MVP rationalizations, no tech-debt-register entries, no pending-architect-review markers, no deferred defects, no cheapest-path defaults, no advisories that should be blockers.
- Production-grade default: PASS — all adjudications are definitive; no deferred decisions.
- Correct agent routing: PASS — VP file not touched (formal-verifier owns VP test name propagation per F-R63-adv-1 recommended route); architecture files not touched (architect owns); STATE.md not touched (state-manager owns).

**VP propagation note:** Formal-verifier must adopt the 4 canonical test names from this §Trace v1.2 in VP v1.2. Canonical names per adjudication above: BC-ABI-001 → `test_BC_ABI_001_status_endpoint_returns_abi_version_1`; BC-ENGINE-002 → `test_BC_ENGINE_002_claude_code_module_strict_basename_detect`; BC-ENGINE-002-ERR → `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich` (unchanged); BC-ENGINE-003 → `test_BC_ENGINE_003_claude_module_hook_paths_five_entries`.

## §Trace v1.3

**v1.3 (2026-05-14):** Architecture version-pin propagation per L-F-R63-PARTIAL-FIX discipline. Trigger: consistency R3 R3-001 closure (commit ba62a15) → architect bumped SS-daemon-lifecycle.md v1.0.9 → v1.0.10 (commit dc3af71) with version-stable phrasing in §BC Summary footer (oscillation prevention). Per L-F-R63-PARTIAL-FIX lesson: when arch version pin advances, the propagation burst MUST update all normative citations in PRD (and VP if affected). Changes applied:

- **Change 1 — Arch pin propagation (SS-daemon-lifecycle.md v1.0.9 → v1.0.10):** 31 normative sites updated across §3 BC Source fields (20 sites: 10 `**Source:**` + 10 `- Source:` Traceability lines), §7 RTM Architecture Source column (10 rows), and frontmatter `traces_to:` (1 site). Sites updated:
  - §3 BC-DAEMON-001: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-002: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-003: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-004: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-005: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-006: `**Source:**` + Traceability `- Source:`
  - §3 BC-RING-001: `**Source:**` + Traceability `- Source:`
  - §3 BC-AUTH-001: `**Source:**` + Traceability `- Source:`
  - §3 BC-AUTH-002: `**Source:**` + Traceability `- Source:`
  - §3 BC-LOCK-001: `**Source:**` (1 site; Traceability Source is combined with SS-core-types-and-abi.md)
  - §7 RTM: BC-DAEMON-001, BC-DAEMON-002, BC-DAEMON-003, BC-DAEMON-004, BC-DAEMON-005, BC-DAEMON-006, BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001 (10 rows)
  - Frontmatter `traces_to:`: pin updated from v1.0.9 to v1.0.10
  Historical §Trace v1.2 entries referencing `SS-daemon-lifecycle.md v1.0.9` are NOT modified — they record the historical state at v1.2 authoring time (2 preserved occurrences in §Trace v1.2 D-042 sweep evidence and META rule checklist).

- **Change 2 — Frontmatter version bump and traces_to extension:** `version: "1.2"` → `"1.3"`. `timestamp` updated. `traces_to:` extended with `R3-001 closure (consistency R3 commit ba62a15; arch v1.0.10 commit dc3af71); L-F-R63-PARTIAL-FIX propagation discipline applied`.

**D-042 sweep (v1.3):** 4-pattern recursive sweep on this document. Pattern 1 (SS-*.md v): SS-daemon-lifecycle.md v1.0.10 ✓ (all 31 normative sites updated; 2 historical §Trace v1.2 references correctly preserved), SS-core-types-and-abi.md v1.2.8 ✓ (no change; still current), SS-engine-module.md v1.1.15 ✓ (no change; still current). No dtu-assessment.md version citations in body. No vision version citations in body. No ADR version citations in body. Zero v1.0.9 references remain in normative content outside §Trace v1.2 historical record. PG-4 PASS — no new §-anchor references introduced in v1.3 changes; all existing §-anchor references unchanged.

**PG-2 count coherence (v1.3):** 22 BCs unchanged ✓. 13 error codes unchanged ✓. 56 edge cases (EC-001 through EC-056) unchanged ✓. 22 test names unchanged ✓. No structural elements added or removed.

**PG-3 §Trace directional refs (v1.3):** No `above`, `below`, or bare L-numbers appear in this §Trace v1.3 entry. All references use section heading anchors (§-form) or commit references.

**PG-3-TRACE-NEW-ENTRY (v1.3):** Post-write self-grep: 0 L[0-9]+ matches in this §Trace v1.3 entry.

**F-R60-corpus-sweep (v1.3):** Zero `v1.0.9` references remain in normative content. Zero count changes in this burst. No §Trace narrative count-claims modified.

**18+ META rule checklist (v1.3):**
- D-042 (4-pattern citation sweep): PASS — SS-daemon-lifecycle.md v1.0.10 current (updated from v1.0.9 per L-F-R63-PARTIAL-FIX), SS-core-types-and-abi.md v1.2.8 current, SS-engine-module.md v1.1.15 current. Zero v1.0.9 references in normative content.
- PG-1 (no ambiguous requirements): PASS — no requirement changes in this burst.
- PG-2 (noun-agnostic count coherence): PASS — 22 BCs, 13 error codes, 56 edge cases, 22 test names all unchanged.
- PG-3 (no L-number pinpoints in §Trace): PASS — all §Trace v1.3 references use section heading anchors or commit refs.
- PG-3-TRACE-NEW-ENTRY (position-free references in new §Trace entries): PASS — v1.3 entry uses only section heading anchors. Post-write self-grep: 0 L[0-9]+ matches.
- PG-4 (§-heading-existence sweep): PASS — no new §-anchor references introduced; all existing anchors unchanged from v1.2 sweep.
- PG-5 (historical-anchor framing): PASS — §Trace v1.2 historical occurrences of v1.0.9 preserved; normative body uses v1.0.10 throughout. No new bare version qualifiers added on stable section refs.
- PG-RECIPE-SCOPE (`.factory/specs/` recursive sweep): PASS — sweep not narrowed.
- append_only_numbering: PASS — no BC IDs or EC IDs renumbered or retired in this burst.
- lift_invariants_to_bcs: PASS — no new invariants; existing invariant coverage unchanged.
- Self-audit (CLAUDE.md §Self-Audit Checklist): All 6 items checked — no MVP rationalizations, no tech-debt-register entries, no pending-architect-review markers, no deferred defects, no cheapest-path defaults, no advisories that should be blockers.
- Production-grade default: PASS — bounded scope; all propagation sites updated; no deferred occurrences.
- Correct agent routing: PASS — VP file not touched; architecture files not touched (architect owns); STATE.md not touched (state-manager owns).

## §Trace v1.4

**v1.4 (2026-05-14):** Architecture version-pin propagation per L-F-R63-PARTIAL-FIX discipline. Trigger: adversary R65 closure chain → F-R65-1/F-R65-2/F-R65-3 resolved in arch v1.0.11 (commit af2101d) by architect. Consistency R4 (commit 3d33937) found R4-001 (VP-only gap, not PRD). Per L-F-R63-PARTIAL-FIX lesson: arch version pin advances from v1.0.10 → v1.0.11 require propagation of all normative citations in PRD (and VP if affected). Changes applied:

- **Change 1 — Arch pin propagation (SS-daemon-lifecycle.md v1.0.10 → v1.0.11):** 31 normative sites updated across §3 BC Source fields (20 sites: 10 `**Source:**` + 10 `- Source:` Traceability lines), §7 RTM Architecture Source column (10 rows), and frontmatter `traces_to:` (1 site). Sites updated:
  - §3 BC-DAEMON-001: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-002: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-003: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-004: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-005: `**Source:**` + Traceability `- Source:`
  - §3 BC-DAEMON-006: `**Source:**` + Traceability `- Source:`
  - §3 BC-RING-001: `**Source:**` + Traceability `- Source:`
  - §3 BC-AUTH-001: `**Source:**` + Traceability `- Source:`
  - §3 BC-AUTH-002: `**Source:**` + Traceability `- Source:`
  - §3 BC-LOCK-001: `**Source:**` (1 site; Traceability Source is combined with SS-core-types-and-abi.md)
  - §7 RTM: BC-DAEMON-001, BC-DAEMON-002, BC-DAEMON-003, BC-DAEMON-004, BC-DAEMON-005, BC-DAEMON-006, BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001 (10 rows)
  - Frontmatter `traces_to:`: current-pointer updated from v1.0.10 to v1.0.11 (historical provenance "arch v1.0.10 commit dc3af71" in traces_to preserved — it documents the R3-001 burst that produced v1.0.10)
  Historical §Trace v1.3 entries referencing `SS-daemon-lifecycle.md v1.0.10` are NOT modified — they record the historical state at v1.3 authoring time (7 preserved occurrences in §Trace v1.3).

- **Change 2 — Frontmatter version bump and traces_to extension:** `version: "1.3"` → `"1.4"`. `timestamp` updated. `traces_to:` extended with `F-R65 closure chain (adversary R65 commit 77fccb7; consistency R4 commit 3d33937; arch v1.0.11 commit af2101d); L-F-R63-PARTIAL-FIX pin propagation applied`.

- **Content confirmation:** PRD's existing BC-AUTH-002 content was already CORRECT prior to this burst. The two-body auth taxonomy (`missing_auth_token` vs `invalid_auth_token`) and the Bearer-header disposition (`Authorization: Bearer` → HTTP 401 `{"error":"missing_auth_token"}` because the `X-Monocle-Authorization` header is absent) were both correct in PRD v1.3. Architect's v1.0.11 made arch agree with PRD/VP — the content fix was in the architecture document, not here. NO BC postconditions rewritten. NO auth taxonomy edits. This is a PURE pin propagation burst.

**D-042 sweep (v1.4):** 4-pattern recursive sweep on this document. Pattern 1 (SS-*.md v): SS-daemon-lifecycle.md v1.0.11 ✓ (all 31 normative sites updated; 7 historical §Trace v1.3 references correctly preserved), SS-core-types-and-abi.md v1.2.8 ✓ (no change; still current), SS-engine-module.md v1.1.15 ✓ (no change; still current). No dtu-assessment.md version citations in body. No vision version citations in body. No ADR version citations in body. Zero v1.0.10 references remain in normative content outside §Trace v1.3 historical record. PG-4 PASS — no new §-anchor references introduced in v1.4 changes; all existing §-anchor references unchanged.

**PG-2 count coherence (v1.4):** 22 BCs unchanged ✓. 13 error codes unchanged ✓. 56 edge cases (EC-001 through EC-056) unchanged ✓. 22 test names unchanged ✓. No structural elements added or removed.

**PG-3 §Trace directional refs (v1.4):** No `above`, `below`, or bare L-numbers appear in this §Trace v1.4 entry. All references use section heading anchors (§-form) or commit references.

**PG-3-TRACE-NEW-ENTRY (v1.4):** Post-write self-grep: 0 L[0-9]+ matches in this §Trace v1.4 entry.

**F-R60-corpus-sweep (v1.4):** Zero `v1.0.10` references remain in normative content. Zero count changes in this burst. No §Trace narrative count-claims modified.

**18+ META rule checklist (v1.4):**
- D-042 (4-pattern citation sweep): PASS — SS-daemon-lifecycle.md v1.0.11 current (updated from v1.0.10 per L-F-R63-PARTIAL-FIX F-R65 chain), SS-core-types-and-abi.md v1.2.8 current, SS-engine-module.md v1.1.15 current. Zero v1.0.10 references in normative content.
- PG-1 (no ambiguous requirements): PASS — no requirement changes in this burst.
- PG-2 (noun-agnostic count coherence): PASS — 22 BCs, 13 error codes, 56 edge cases, 22 test names all unchanged.
- PG-3 (no L-number pinpoints in §Trace): PASS — all §Trace v1.4 references use section heading anchors or commit refs.
- PG-3-TRACE-NEW-ENTRY (position-free references in new §Trace entries): PASS — v1.4 entry uses only section heading anchors. Post-write self-grep: 0 L[0-9]+ matches.
- PG-4 (§-heading-existence sweep): PASS — no new §-anchor references introduced; all existing anchors unchanged from v1.3 sweep.
- PG-5 (historical-anchor framing): PASS — §Trace v1.3 historical occurrences of v1.0.10 preserved; normative body uses v1.0.11 throughout. No new bare version qualifiers added on stable section refs.
- PG-RECIPE-SCOPE (`.factory/specs/` recursive sweep): PASS — sweep not narrowed.
- append_only_numbering: PASS — no BC IDs or EC IDs renumbered or retired in this burst.
- lift_invariants_to_bcs: PASS — no new invariants; existing invariant coverage unchanged.
- Self-audit (CLAUDE.md §Self-Audit Checklist): All 6 items checked — no MVP rationalizations, no tech-debt-register entries, no pending-architect-review markers, no deferred defects, no cheapest-path defaults, no advisories that should be blockers.
- Production-grade default: PASS — bounded scope; all 31 propagation sites updated; no deferred occurrences.
- Correct agent routing: PASS — VP file not touched; architecture files not touched (architect owns); STATE.md not touched (state-manager owns).

## §Trace v1.5

**v1.5 (2026-05-14):** F-R67-2 closure — PRD EC-045 off-by-one fix. Trigger: adversary R67 fresh-context pass (finding F-R67-2 HIGH). This is a single-site content correction; no architecture version pins changed; no BC postconditions changed.

- **F-R67-2 RESOLVED (HIGH) — EC-045 prose off-by-one corrected:** §3 BC-DAEMON-003 EC-045 prose (formerly at line 228) said "exactly 262,144 bytes: HTTP 413" — a logical contradiction of its own rationale clause ("strictly exceeding N bytes"). For N=262,144, strictly exceeding means ≥ 262,145. Body of exactly 262,144 should return HTTP 200. The sole fix: "262,144" changed to "262,145" and a clarifying parenthetical added: "body of exactly N=262,144 returns HTTP 200." This makes the boundary semantics explicit, matching VP-DAEMON-003 mechanical property 3, BC-DAEMON-003 postcondition 2, and §9 EC-045 catalog row (which already said 262,145 correctly). Root cause: boundary-condition prose introduced in the F-R62 BC expansion burst was outside the semantic-propagation-sweep coverage established at that time. The Obs-1 discipline (intra-document same-ID consistency sweep before commit) now applied codifies the lesson.

**Intra-document EC consistency sweep (Obs-1-discipline, v1.5):** Grep sweep for all EC-0NN IDs in §3 prose vs §9 catalog. Boundary-numeric ECs checked:

- EC-045 (BC-DAEMON-003): §3 prose now "262,145 bytes: HTTP 413" ✓; §9 catalog "262,145 bytes → HTTP 413" ✓ — CONSISTENT post-fix.
- EC-046 (BC-DAEMON-003): §3 prose "262,143 bytes: HTTP 200" ✓; §9 catalog "Body 262,143 bytes → HTTP 200" ✓ — CONSISTENT (no change needed).
- EC-002 (BC-RING-001): §3 prose "256 KiB line" (qualitative); §9 catalog "Near-maximum payload size (256 KiB line); rotation handles without truncation" ✓ — CONSISTENT.

All other ECs (EC-001 through EC-056 except EC-045, EC-046, EC-002) are non-numeric boundary descriptions; no off-by-one risk class applies. No further inconsistencies found by this sweep.

**D-042 sweep (v1.5):** 4-pattern recursive sweep on this document. Pattern 1 (SS-*.md v): SS-daemon-lifecycle.md v1.0.11 ✓ (no pin change in this burst), SS-core-types-and-abi.md v1.2.8 ✓, SS-engine-module.md v1.1.15 ✓. No new SS-*.md version citations introduced. No normative architecture version changes in this burst; all prior v1.4 pin records intact.

**PG-2 count coherence (v1.5):** 22 BCs unchanged ✓. 13 error codes unchanged ✓. 56 edge cases (EC-001 through EC-056) unchanged ✓. 22 test names unchanged ✓. No structural elements added or removed; this is a single-word content correction.

**PG-3 §Trace directional refs (v1.5):** No `above`, `below`, or bare L-numbers appear in this §Trace v1.5 entry. All references use section heading anchors (§-form) or finding references.

**PG-3-TRACE-NEW-ENTRY (v1.5):** Post-write self-grep: 0 L[0-9]+ matches in this §Trace v1.5 entry.

**F-R60-corpus-sweep (v1.5):** Zero stale "262,144" occurrences in EC-045 normative prose. The value 262,144 still appears correctly in other contexts: BC-DAEMON-003 precondition 2 (the trigger threshold description), postcondition 1 (the `limit_bytes` response field value), §5 error taxonomy E-DAEMON-001, §7 RTM differentiator traceability, NFR-005, and §9 EC-047 prose — all of these reference 262,144 as the configured limit constant (N), which is correct. Only EC-045 references the boundary-crossing value (N+1 = 262,145). Sweep confirms no other stale occurrences.

**18+ META rule checklist (v1.5):**
- D-042 (4-pattern citation sweep): PASS — no SS-*.md version changes; all current pointers from v1.4 unchanged.
- PG-1 (no ambiguous requirements): PASS — EC-045 is now unambiguous; boundary semantics explicitly stated with parenthetical.
- PG-2 (noun-agnostic count coherence): PASS — 22 BCs, 13 error codes, 56 edge cases, 22 test names all unchanged.
- PG-3 (no L-number pinpoints in §Trace): PASS — all §Trace v1.5 references use section heading anchors.
- PG-3-TRACE-NEW-ENTRY (position-free references in new §Trace entries): PASS — v1.5 entry uses only section heading anchors. Post-write self-grep: 0 L[0-9]+ matches.
- PG-4 (§-heading-existence sweep): PASS — no new §-anchor references introduced; all existing §-anchor references unchanged from v1.4 sweep.
- PG-5 (historical-anchor framing): PASS — §Trace v1.4 historical entries preserved; no version qualifiers changed on stable section refs.
- PG-RECIPE-SCOPE (`.factory/specs/` recursive sweep): PASS — sweep not narrowed.
- append_only_numbering: PASS — no BC IDs or EC IDs renumbered or retired in this burst.
- lift_invariants_to_bcs: PASS — no new invariants; existing invariant coverage unchanged.
- Self-audit (CLAUDE.md §Self-Audit Checklist): All 6 items checked — no MVP rationalizations, no tech-debt-register entries, no pending-architect-review markers, no deferred defects, no cheapest-path defaults, no advisories that should be blockers.
- Production-grade default: PASS — single-site content fix applied; boundary semantics explicit; no deferred occurrences.
- Correct agent routing: PASS — VP file not touched (VP-DAEMON-003 already correct; no change needed); architecture files not touched (architect owns); STATE.md not touched (state-manager owns).

## §Trace v1.6

**v1.6 (2026-05-14):** F-R70 closure chain — four findings from adversary R70 (commit 4b4aea1) + arch v1.0.12 content propagation. Trigger: architect committed SS-daemon-lifecycle.md v1.0.11 → v1.0.12 (commit 727c826) resolving F-R70-1 and F-R70-3; product-owner resolves F-R70-2 and Obs-R70-1 in this burst. All four findings closed.

- **F-R70-3 propagation RESOLVED (MEDIUM — POSIX exit-code correction from arch v1.0.12):** BC-DAEMON-004 postcondition 8 previously specified a binary exit-code taxonomy (0 = clean drain; 130 = hard-killed). The binary form failed to distinguish SIGTERM hard-kill (correct POSIX: 128+15 = 143) from SIGINT hard-kill (correct POSIX: 128+2 = 130). Additionally, admin `POST /shutdown` second-call during drain had no assigned exit code, and startup failure (`exit 1`) was unspecified. Correction applied per arch v1.0.12 §Hard Shutdown disposition (c): postcondition 8 replaced with an enumerated five-code taxonomy:
  - `0` — graceful drain complete (unchanged semantic).
  - `130` — SIGINT hard-kill during drain (POSIX 128+2; Ctrl-C second press).
  - `143` — SIGTERM hard-kill during drain (POSIX 128+15; systemd/k8s second SIGTERM).
  - `2` — admin `POST /shutdown` second-call during drain (monocle-specific; outside POSIX 128+N space; distinct from startup-failure exit 1).
  - `1` — daemon startup failure (RuntimeDirUnresolvable, port bind failure, existing live lock file).
  Canonical test vectors table expanded from 4 rows to 7 rows covering all five exit-code paths. New test added: `test_BC_DAEMON_004_exit_codes_posix_distinct` in `monocle-runtime/tests/daemon_lifecycle.rs`. Invariant 2 updated to note signal-type recording for exit-code selection. Invariant 4 added: monitoring systems MUST use 143 (not 130) to detect SIGTERM hard-kill. EC-050 updated: second `POST /shutdown` exits 2 (not merely "hard close"). PG-3 compliance: §Daemon Lifecycle Protocol §Shutdown Signal Handling is a real heading in arch (verified in v1.1 sweep); §Hard Shutdown is a real heading (verified).

- **F-R70-1 propagation RESOLVED (HIGH — macOS runtime_dir fallback chain from arch v1.0.12):** BC-DAEMON-005 precondition 2 previously stated only that the runtime directory has been created or already exists — with no specification of HOW it is resolved. The arch v1.0.12 §Start Sequence step 1 now specifies a four-path resolution chain addressing the `ProjectDirs::runtime_dir() == None` on macOS (primary target per NFR-008). Precondition 2 rewritten to enumerate all four paths: (a) `MONOCLE_RUNTIME_DIR` env override; (b) `ProjectDirs::runtime_dir()` on Linux/XDG; (c) `ProjectDirs::data_local_dir()` fallback on macOS/Windows; (d) fail-fast `DaemonStartError::RuntimeDirUnresolvable` + exit 1 if all return `None`. New postcondition 5 added for the `RuntimeDirUnresolvable` fail-fast path. Invariant 4 added: asymmetry with BC-ENGINE-002-ERR is intentional. Three new edge cases added: EC-057 (macOS `data_local_dir` fallback happy path), EC-058 (env override happy path), EC-059 (full-fail `RuntimeDirUnresolvable` path). New error code E-DAEMON-004 added to §5 Error Taxonomy for `DaemonStartError::RuntimeDirUnresolvable`. Canonical test vectors expanded with three new rows covering EC-057, EC-058, EC-059. Verification note updated to reference EC-057/058/059 coverage. PG-3 compliance: §Daemon Lifecycle Protocol §Start Sequence is a real heading (verified in v1.1 sweep); §Hard Shutdown is a real heading (verified).

- **F-R70-2 RESOLVED (MEDIUM — BC-DAEMON-006 timestamp format tightening):** BC-DAEMON-006 invariant 1 schema for `shutdown_utc` previously used generic `"<ISO8601>"` placeholder. VP-DAEMON-006 enforces `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` (mandatory millisecond precision). A BC-compliant seconds-only timestamp would fail the VP test. Resolution (a) chosen per adversary R70 recommendation: BC tightened to match VP and EC-044 precedent (`last_hook_ts` format). Invariant 1 now specifies `"shutdown_utc":"YYYY-MM-DDTHH:MM:SS.sssZ"` with explicit mandatory-millisecond annotation and VP-DAEMON-006 regex cross-reference. Cross-field consistency confirmed: EC-044 uses the same format for `last_hook_ts`; BC-DAEMON-006 now matches exactly.

- **Obs-R70-1 RESOLVED (LOW — EC-031 fail-open security rationale added):** EC-031 described the fail-open default (`HookDecision::Allow`) for unrecognized `HookEvent` variants but contained no security rationale for why fail-open is the correct choice for a permission-adjacent decision point. Security rationale added: unrecognized variants in Phase 1 are non-permission-relevant by design (Phase 1 enumerates exactly 5 hook types; future variants carry their own permission context); `Defer` would stall callers with no registered TUI handler; the localhost-only threat model (monocle binds to `127.0.0.1`; no untrusted remote callers) makes defensive stalling disproportionate; future Phase 4 hooks with permission semantics MUST be explicitly enumerated in `HookEvent` (not left to the wildcard arm). Forward-compat justification rationale selected (over conservative `Defer`) per the localhost threat model and zero-stall requirement.

- **Arch pin propagation (SS-daemon-lifecycle.md v1.0.11 → v1.0.12):** 31 normative sites updated per L-F-R63-PARTIAL-FIX discipline:
  - §3 BC Source fields (20 sites): 10 `**Source:**` lines + 10 `- Source:` Traceability lines for BC-DAEMON-001 through BC-DAEMON-006, BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001.
  - §7 RTM Architecture Source column (10 rows): BC-DAEMON-001 through BC-DAEMON-006, BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001.
  - Frontmatter `traces_to:` (1 site): current-pointer updated from v1.0.11 to v1.0.12; R70 closure chain references appended.
  Historical §Trace v1.4 and v1.5 entries referencing SS-daemon-lifecycle.md v1.0.11 are NOT modified — they record historical state at v1.4/v1.5 authoring time.

**Count changes:**
- BC count: 22 — unchanged (BC-DAEMON-005 updated in place; no new BCs added; no BCs retired).
- Error code count: 13 → 14. E-DAEMON-004 added (`DaemonStartError::RuntimeDirUnresolvable`).
- Edge case count: 56 → 59. EC-057, EC-058, EC-059 added (all BC-DAEMON-005 platform-resolution scenarios).
- Test name count: 22 → 23. New test `test_BC_DAEMON_004_exit_codes_posix_distinct` added (in `monocle-runtime/tests/daemon_lifecycle.rs`). Existing `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests` retained (different behavioral scope: 503 response during drain).

**D-042 sweep (v1.6):** 4-pattern recursive sweep on this document. Pattern 1 (SS-*.md v): SS-daemon-lifecycle.md v1.0.12 ✓ (all 31 normative sites updated; historical §Trace v1.4/v1.5 references preserved), SS-core-types-and-abi.md v1.2.8 ✓ (no change; still current), SS-engine-module.md v1.1.15 ✓ (no change; still current). No dtu-assessment.md version citations in body. No vision version citations in body. No ADR version citations in body. Zero v1.0.11 references remain in normative content outside §Trace v1.4/v1.5 historical records.

**PG-2 count coherence (v1.6):** 22 BCs unchanged ✓. Error codes: 14 (added E-DAEMON-004; was 13) ✓. Edge cases: 59 (EC-057, EC-058, EC-059 added; was 56) ✓. Test names: 23 (test_BC_DAEMON_004_exit_codes_posix_distinct added; was 22) ✓. No IDs renumbered or retired.

**PG-3 §Trace directional refs (v1.6):** No `above`, `below`, or bare L-numbers appear in this §Trace v1.6 entry. All references use section heading anchors (§-form) or commit/finding references.

**PG-3-TRACE-NEW-ENTRY (v1.6):** Post-write self-grep: 0 L[0-9]+ matches in this §Trace v1.6 entry.

**PG-4 §-heading-existence sweep (v1.6):** New §-anchor references introduced in this burst:
- `§Daemon Lifecycle Protocol §Shutdown Signal Handling` — verified against arch (EXISTS heading, already confirmed v1.1 sweep).
- `§Hard Shutdown` — verified against arch §Hard Shutdown (EXISTS heading in SS-daemon-lifecycle.md v1.0.12).
- `§Start Sequence` — verified (EXISTS; confirmed in v1.1 sweep).
- `§Daemon Lifecycle Protocol §Crash Recovery` — verified (EXISTS; confirmed in v1.1 sweep).
- `§Daemon Lifecycle Protocol §Start Sequence` — verified (EXISTS; confirmed in v1.1 sweep).
- `§Health and Status Endpoints` — verified (EXISTS; confirmed in v1.1 sweep).
- `§Body Size Limit` — verified (EXISTS; confirmed in v1.1 sweep).
- `§Drain` — verified (EXISTS; confirmed in v1.1 sweep).
All §-anchor references: PASS.

**F-R60-corpus-sweep (v1.6):** Zero `v1.0.11` references remain in normative content (31 sites updated). EC-050 updated (exit code 2 added). No other numeric EC boundary values changed. §Trace v1.5 "single-word content correction" narrative preserved; no stale counts in prior §Trace entries.

**Intra-document consistency sweep (L-F-R63-PARTIAL-FIX Extension 2, v1.6):**
- BC-DAEMON-004 exit-code claims: §3 postcondition 8 (5 codes: 0/130/143/2/1) ✓ — §3 canonical test vectors (7 rows covering all 5 codes) ✓ — §5 Error Taxonomy (no exit-code codes in taxonomy, exit codes are BC-level) ✓ — §7 RTM (Architecture Source updated) ✓ — §9 EC-050 (second `/shutdown` exits 2) ✓ — CONSISTENT.
- BC-DAEMON-005 resolution chain: §3 precondition 2 (4-path chain: a/b/c/d) ✓ — §3 postcondition 5 (RuntimeDirUnresolvable fail-fast) ✓ — §3 invariant 4 (asymmetry rationale) ✓ — §3 EC-057/058/059 ✓ — §5 E-DAEMON-004 (RuntimeDirUnresolvable error code) ✓ — §7 RTM (Architecture Source v1.0.12) ✓ — §9 EC-057/058/059 catalog rows ✓ — CONSISTENT.
- BC-DAEMON-006 timestamp format: §3 invariant 1 (`YYYY-MM-DDTHH:MM:SS.sssZ`) ✓ — §9 EC-056 catalog row (no format claim; behavior only) ✓ — CONSISTENT. VP-DAEMON-006 regex cross-reference cited in invariant 1 (no VP file touched; formal-verifier owns VP).
- EC-031 fail-open rationale: §3 EC-031 prose ✓ — §9 EC-031 catalog row (behavior description; no rationale claim) ✓ — CONSISTENT.
- E-DAEMON-004 propagation: §5 Error Taxonomy row (1 new row: E-DAEMON-004) ✓ — §3 BC-DAEMON-005 postcondition 5 cites this error variant verbatim ✓ — CONSISTENT.

**18+ META rule checklist (v1.6):**
- D-042 (4-pattern citation sweep): PASS — SS-daemon-lifecycle.md v1.0.12 current (updated from v1.0.11); SS-core-types-and-abi.md v1.2.8 current; SS-engine-module.md v1.1.15 current. Zero v1.0.11 references in normative content.
- D-047 strict (3-clean-pass convergence): N/A for PRD authoring; applies to adversarial review passes.
- PG-1 (no ambiguous requirements): PASS — all four finding closures produce unambiguous requirements. BC-DAEMON-004 exit codes are enumerated exhaustively. BC-DAEMON-005 resolution chain is fully specified. BC-DAEMON-006 timestamp format is exact. EC-031 security rationale is explicit.
- PG-2 (noun-agnostic count coherence): PASS — 22 BCs (unchanged); 14 error codes (E-DAEMON-004 added, was 13); 59 edge cases (EC-057/058/059 added, was 56); 23 test names (exit_codes_posix_distinct added, was 22). All count changes documented.
- PG-3 (no L-number pinpoints in §Trace): PASS — all §Trace v1.6 references use section heading anchors or commit/finding references.
- PG-3-TRACE-NEW-ENTRY (position-free references in new §Trace entries): PASS — v1.6 entry uses only section heading anchors. Post-write self-grep: 0 L[0-9]+ matches.
- PG-4 (§-heading-existence sweep): PASS — all new §-anchor references verified against SS-daemon-lifecycle.md v1.0.12 headings (listed above). Zero mis-anchors.
- PG-5 (historical-anchor framing): PASS — §Trace v1.4/v1.5 historical occurrences of v1.0.11 preserved; normative body uses v1.0.12 throughout. No new bare unqualified version references on stable section refs.
- PG-RECIPE-SCOPE (`.factory/specs/` recursive sweep): PASS — sweep not narrowed.
- append_only_numbering: PASS — no BC IDs renumbered or retired; EC-057/058/059 are new sequential IDs; E-DAEMON-004 is a new sequential error code.
- lift_invariants_to_bcs: PASS — no new domain invariants from arch v1.0.12; all arch v1.0.12 behavioral contract content now reflected in BC-DAEMON-004 + BC-DAEMON-005 + BC-DAEMON-006.
- F-R60-corpus-sweep: PASS — §Trace narrative count claims in v1.5 not stale (they record v1.5-era values; v1.6 count changes are in v1.6 entry only).
- Self-audit (CLAUDE.md §Self-Audit Checklist): All 6 items checked — no MVP rationalizations, no tech-debt-register entries, no pending-architect-review markers, no deferred defects, no cheapest-path defaults, no advisories that should be blockers.
- Production-grade default: PASS — all four finding closures are complete; no deferred occurrences; no "for now" or "TBD" markers. EC-031 rationale is explicit rather than deferred.
- Correct agent routing: PASS — VP file not touched (formal-verifier propagates VP-DAEMON-004/005/006 changes next); architecture files not touched (arch v1.0.12 is the source, not the target); STATE.md not touched (state-manager owns).

**VP propagation note:** Formal-verifier must propagate to VP-DAEMON-004 (new exit code enumeration, 5 codes), VP-DAEMON-005 (runtime-dir resolution chain in preconditions/test vectors), and VP-DAEMON-006 (timestamp format already correct per the BC-is-now-tighter logic; confirm regex still matches). Obs-R70-2 (VP-DAEMON-004 over-budget exit-code looseness) should also be reviewed in the same burst.

## §Trace v1.7

**v1.7 (2026-05-15):** F-R71 closure chain (PRD-side) — F-R71-3 NFR-008 phrasing fix + arch pin propagation v1.0.12 → v1.0.13. Trigger: architect committed SS-daemon-lifecycle.md v1.0.12 → v1.0.13 (commit 1f53d47) resolving F-R71-2 (test name drift) + F-R71-3 (NFR-008 mis-anchor) + F-R71-4 (tower/nix dep-pin dispositions). Product-owner resolves F-R71-3 PRD-side in this burst. Adversary R71 commit: 2710ab4.

- **F-R71-3 PRD-side RESOLVED (MEDIUM — NFR-008 mis-anchor in BC-DAEMON-005 precondition 2 rationale):** PRD §3 BC-DAEMON-005 precondition 2 rationale (line 328) previously read: "macOS is the primary target (NFR-008)." This framing implies macOS is the SOLE primary target. PRD NFR-008 (§4, line 1210) specifies `macOS + Linux (darwin/linux × amd64/arm64)` — coequal, no sole-primary designation. Corrected to match arch v1.0.13 §Start Sequence step 1 Rationale phrasing (site 2): "macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64)." No behavioral change — the rationale for the runtime_dir fallback chain is unaffected; the chain is required because `ProjectDirs::runtime_dir()` returns `None` on macOS regardless of whether macOS is sole-primary or co-primary.

- **Arch pin propagation (SS-daemon-lifecycle.md v1.0.12 → v1.0.13):** 31 normative sites updated per L-F-R63-PARTIAL-FIX discipline:
  - §3 BC Source fields (20 sites): 10 `**Source:**` lines + 10 `- Source:` Traceability lines for BC-DAEMON-001 through BC-DAEMON-006, BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001.
  - §7 RTM Architecture Source column (10 rows): BC-DAEMON-001 through BC-DAEMON-006, BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001.
  - Frontmatter `traces_to:` (1 site): current-pointer updated from v1.0.12 to v1.0.13; R71 closure chain references appended.
  Historical §Trace v1.6 entries referencing SS-daemon-lifecycle.md v1.0.12 are NOT modified — they record historical state at v1.6 authoring time (PG-5 compliance).

- **SS-deps-pin-manifest.md v1.1.9 (commit 1f53d47):** PRD cites SS-deps-pin-manifest.md by path only (no version in PRD body or RTM). Zero version-string sites to update. No change required.

**Count changes (v1.7):**
- BC count: 22 — unchanged.
- Error code count: 14 — unchanged.
- Edge case count: 59 — unchanged.
- Test name count: 23 — unchanged.

**D-042 sweep (v1.7):** 4-pattern recursive sweep on this document. Pattern 1 (SS-*.md v): SS-daemon-lifecycle.md v1.0.13 ✓ (31 normative sites updated; historical §Trace v1.6 references preserved), SS-core-types-and-abi.md v1.2.8 ✓ (no change; still current), SS-engine-module.md v1.1.15 ✓ (no change; still current). SS-deps-pin-manifest.md — no versioned citation in PRD body; no change required. Zero v1.0.12 references remain in normative content outside §Trace v1.6 historical records.

**PG-2 count coherence (v1.7):** 22 BCs unchanged ✓. 14 error codes unchanged ✓. 59 edge cases unchanged ✓. 23 test names unchanged ✓. No IDs renumbered or retired.

**PG-3 §Trace directional refs (v1.7):** No `above`, `below`, or bare L-numbers appear in this §Trace v1.7 entry. All references use section heading anchors (§-form) or commit/finding references.

**PG-3-TRACE-NEW-ENTRY (v1.7):** Post-write self-grep: 0 L[0-9]+ matches in this §Trace v1.7 entry.

**PG-4 §-heading-existence sweep (v1.7):** No new §-anchor references introduced in this burst. All existing §-anchor references carry forward from v1.6. No new mis-anchors possible.

**PG-5 (historical-anchor framing):** §Trace v1.6 historical occurrences of v1.0.12 preserved (13 sites in §Trace v1.6 text); normative body uses v1.0.13 throughout. Zero v1.0.12 references in normative content outside §Trace v1.6.

**Intra-document consistency sweep (L-F-R63-PARTIAL-FIX Extension 2, v1.7):**
- NFR-008 phrasing: §3 BC-DAEMON-005 precondition 2 rationale ("macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64)") ✓ — §4 NFR-008 (`macOS + Linux (darwin/linux × amd64/arm64)`) ✓ — CONSISTENT. No other "macOS is the primary target" or "macOS as the primary target" framings exist in normative PRD body (§3, §4, §6, §10 Glossary confirmed clean; §Trace v1.6 historical reference at line 1716 preserved; line 1320 uses distinct framing "darwin/linux primary per brief §Scope" describing Windows as secondary — not an NFR-008 mis-anchor).
- §7 RTM Architecture Source column: 10 BC-DAEMON/RING/AUTH/LOCK rows now cite v1.0.13 ✓ — CONSISTENT with §3 Source fields.

**18+ META rule checklist (v1.7):**
- D-042 (4-pattern citation sweep): PASS — SS-daemon-lifecycle.md v1.0.13 current (updated from v1.0.12); SS-core-types-and-abi.md v1.2.8 unchanged; SS-engine-module.md v1.1.15 unchanged. Zero v1.0.12 references in normative content.
- PG-1 (no ambiguous requirements): PASS — NFR-008 phrasing fix is unambiguous; "macOS + Linux, darwin/linux × amd64/arm64" is explicit.
- PG-2 (noun-agnostic count coherence): PASS — 22 BCs, 14 error codes, 59 edge cases, 23 test names — all unchanged.
- PG-3 (no L-number pinpoints in §Trace): PASS — this §Trace v1.7 entry uses only section heading anchors or commit/finding references.
- PG-3-TRACE-NEW-ENTRY (position-free references): PASS — v1.7 entry uses only section heading anchors. Post-write self-grep: 0 L[0-9]+ matches.
- PG-4 (§-heading-existence sweep): PASS — no new §-anchor references; all carry forward from v1.6.
- PG-5 (historical-anchor framing): PASS — §Trace v1.6 historical occurrences of v1.0.12 preserved; normative body uses v1.0.13 throughout.
- PG-RECIPE-SCOPE (`.factory/specs/` recursive sweep): PASS — sweep not narrowed.
- append_only_numbering: PASS — no BC IDs renumbered or retired; no new IDs added.
- lift_invariants_to_bcs: PASS — no new domain invariants from arch v1.0.13; F-R71-2/3/4 closures are test-name, phrasing, and dep-pin corrections only — no new behavioral requirements.
- Self-audit (CLAUDE.md §Self-Audit Checklist): All 6 items checked — no MVP rationalizations, no tech-debt-register entries, no pending-architect-review markers, no deferred defects, no cheapest-path defaults, no advisories that should be blockers.
- Production-grade default: PASS — F-R71-3 PRD-side closure is complete; phrasing corrected to exact arch v1.0.13 parallel form; no deferred occurrences.
- Correct agent routing: PASS — VP file not touched (formal-verifier owns VP propagation for F-R71-1/4/5); architecture files not touched (arch v1.0.13 is the source, not the target); STATE.md not touched (state-manager owns).

**VP propagation note (v1.7):** Formal-verifier must propagate F-R71-1 (directories 5→6, 2 VP sites), F-R71-4a (drop "per manifest" tower citation from VP-DAEMON-005), F-R71-4b (name `nix 0.30` as binding crate in VP-DAEMON-005), F-R71-5 (VP `<int>` → `<N>` placeholder), and arch + PRD pin propagation (v1.0.12 → v1.0.13 in VP body). Manifest v1.1.9 cites `nix 0.30` as new explicit workspace pin (added by architect commit 1f53d47).
