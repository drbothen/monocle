---
document_type: prd-supplement-interface-definitions
level: L3
version: "1.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T01:00:00Z
phase: 1a
inputs: [prd.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]
input-hash: "680be97"
traces_to: prd.md
---

# Interface Definitions: Monocle Phase 1

> PRD supplement — created during v1.26 restructure.
> Phase 1 scope note: Monocle Phase 1 delivers a daemon binary (`monocle-daemon`) with an HTTP
> API surface, not a standalone CLI with flags. The CLI interface for the TUI shell wrapper
> is a Phase 1b deliverable. This document covers the HTTP API interface (the primary Phase 1
> interface surface) and the lock file schema.
>
> Primary consumers: implementer, test-writer, devops-engineer.

## Phase 1 Interface Summary

Phase 1 defines three interface surfaces:

1. **HTTP API** — daemon-side REST endpoints (axum router, `127.0.0.1:<port>`)
2. **Lock File** — JSON schema for IPC discovery between TUI client and daemon
3. **JSONL Ring Buffer** — append-only event log schema for Phase 2 trigger-trace

Full behavioral contracts for each endpoint are in `behavioral-contracts/ss-01/` and `behavioral-contracts/ss-02/`.

---

## HTTP API

### Endpoint: GET /healthz (Unauthenticated)

**Contract:** BC-2.01.001
**Router:** Unauthenticated router (no `DefaultBodyLimit`, no auth middleware)
**Auth:** None required

**Request:**
```
GET /healthz HTTP/1.1
Host: 127.0.0.1:<port>
```

**Response (200 — daemon alive):**
```json
{
  "status": "alive",
  "uptime_sec": <N>,
  "version": "<semver>"
}
```

**Response (503 — shutting down or hook-receiver task dead):**
```json
{
  "status": "shutting_down"
}
```

**Field Constraints:**
| Field | Type | Constraint |
|-------|------|------------|
| `status` | string | `"alive"` or `"shutting_down"` |
| `uptime_sec` | integer | seconds since daemon start; ≥ 0 |
| `version` | string | semver 2.0 regex `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$`; no leading `v` |

---

### Endpoint: GET /status (Authenticated)

**Contract:** BC-2.01.002
**Router:** Authenticated router (subject to 256 KiB `DefaultBodyLimit`)
**Auth:** Canonical `X-Monocle-Authorization: monocle-v1:<64-hex-token>` **or** alias `X-Claude-Code-Ide-Authorization: <64-hex>` (ADR-0005 dual-accept applies; WARN log emitted on alias path)

**Request:**
```
GET /status HTTP/1.1
Host: 127.0.0.1:<port>
X-Monocle-Authorization: monocle-v1:<64-hex-token>
```

**Response (200 — authenticated):**
```json
{
  "pid": <integer>,
  "uptime_sec": <integer>,
  "version": "<semver>",
  "abi_version": 1,
  "lock_file": "<absolute-path>",
  "hook_endpoints": [
    "/hooks/pre-tool-use",
    "/hooks/notification",
    "/hooks/stop",
    "/hooks/session-start",
    "/hooks/prompt-submit"
  ],
  "ring_buffer_fill_pct": <float>,
  "channel_saturation_pct": <float>,
  "last_hook_ts": {
    "pre_tool_use": "<ISO8601>" | null,
    "notification": "<ISO8601>" | null,
    "stop": "<ISO8601>" | null,
    "session_start": "<ISO8601>" | null,
    "prompt_submit": "<ISO8601>" | null
  },
  "tui_attached": <boolean>
}
```

**Response (401 — missing header):**
```json
{"error":"missing_auth_token"}
```

**Response (401 — invalid header):**
```json
{"error":"invalid_auth_token"}
```

**Field Constraints:**
| Field | Type | Constraint |
|-------|------|------------|
| `pid` | integer | ≥ 1 (POSIX; PID 0 is scheduler) |
| `uptime_sec` | integer | ≥ 0 |
| `version` | string | semver 2.0 regex (same as /healthz) |
| `abi_version` | integer | `1` (equals `monocle_core::MONOCLE_ABI_VERSION`) |
| `lock_file` | string | absolute path to `<runtime_dir>/monocle.lock` |
| `hook_endpoints` | array[string] | exactly 5 elements in the order shown |
| `ring_buffer_fill_pct` | float | 0.0–100.0 inclusive |
| `channel_saturation_pct` | float | 0.0–100.0 inclusive |
| `last_hook_ts` | object | per-hook ISO 8601 `YYYY-MM-DDTHH:MM:SS.sssZ` or JSON `null` |
| `tui_attached` | boolean | `true` if TUI client is connected via UDS |

---

### Endpoint: POST /hooks/* (Authenticated, 5 endpoints)

**Contract:** BC-2.01.001 through BC-2.01.010 (hook ingestion surface)
**Router:** Authenticated router (subject to 256 KiB `DefaultBodyLimit`)
**Auth:** Canonical `X-Monocle-Authorization: monocle-v1:<64-hex-token>` **or** alias `X-Claude-Code-Ide-Authorization: <64-hex>` (ADR-0005 dual-accept applies; WARN log emitted on alias path)

The 5 hook endpoints match Claude Code's canonical hook protocol (JC-2 gene-source parity):

| Path | Hook Type | Priority |
|------|-----------|----------|
| `POST /hooks/pre-tool-use` | PreToolUse | P0 |
| `POST /hooks/notification` | Notification | P0 |
| `POST /hooks/stop` | Stop | P0 |
| `POST /hooks/session-start` | SessionStart | P0 |
| `POST /hooks/prompt-submit` | UserPromptSubmit | P0 |

**Body size limit:** 262,144 bytes (256 KiB). Bodies exceeding this return HTTP 413 (BC-2.01.003, E-DAEMON-001).

**NOT included in Phase 1:** `PostToolUse` — per JC-2 joint closure (gene-source parity with any-context-lazyclaude BC-HOOK-007 canonical 5-endpoint matrix).

---

### Endpoint: POST /shutdown (Authenticated, Admin)

**Contract:** BC-2.01.004, BC-2.01.008, BC-2.01.009
**Router:** Authenticated router (subject to 256 KiB `DefaultBodyLimit`)
**Auth:** Canonical `X-Monocle-Authorization: monocle-v1:<64-hex-token>` **or** alias `X-Claude-Code-Ide-Authorization: <64-hex>` (ADR-0005 dual-accept applies; WARN log emitted on alias path)

**Request:**
```
POST /shutdown HTTP/1.1
Host: 127.0.0.1:<port>
X-Monocle-Authorization: monocle-v1:<64-hex-token>
Content-Length: 0
```
Request body: empty or JSON `null`. Daemon ignores the body entirely.

**Response (200 — drain initiated):**
```json
{"status":"shutting_down"}
```
The daemon begins the 10-second graceful drain window (BC-2.01.004). In-flight hook requests in the drain window complete normally. New hook arrivals during drain receive HTTP 503 with `Retry-After: 10` (E-DAEMON-002).

**Response (503 — already draining, EC-050 second /shutdown):**
```json
{"error":"daemon_shutting_down"}
```
A second `POST /shutdown` received **during** the active drain window triggers EC-050: daemon forces exit 2 immediately without waiting for the drain to complete. The HTTP 503 response is sent before the forced exit.

**Field Constraints:**
| Field | Type | Constraint |
|-------|------|------------|
| `status` | string | `"shutting_down"` (200 response only) |
| `error` | string | `"daemon_shutting_down"` (503 response only) |

**Edge Cases:**

| Scenario | Behavior |
|----------|----------|
| `POST /shutdown` without auth | HTTP 401 per BC-2.01.009 (auth middleware runs before shutdown handler) |
| `POST /shutdown` during drain (EC-050) | HTTP 503 `{"error":"daemon_shutting_down"}`; daemon forces exit 2 |
| `POST /shutdown` with alias header only | HTTP 200 + drain initiated + WARN deprecation log (ADR-0005 alias path) |

---

## Exit Code Semantics (Daemon Process)

| Code | Meaning | Trigger |
|------|---------|---------|
| 0 | Clean exit | Normal graceful shutdown after drain completes |
| 1 | Fatal error | `DaemonStartError::RuntimeDirUnresolvable` (E-DAEMON-004); lock file conflict (E-LOCK-001); lock file write failure (EC-051) |
| 2 | Forced stop | Second `POST /shutdown` during drain (EC-050 — admin forced-stop) |

---

## Authentication Header Format

**Contract:** BC-2.01.008, BC-2.01.009; **Dual-accept decision:** ADR-0005

### Canonical Header (Priority)

**Header name:** `X-Monocle-Authorization`
**Value format:** `monocle-v1:<64-hex-lowercase>`
**Token entropy:** 32 bytes from `rand::rngs::OsRng` encoded as 64-character lowercase hex
**Token regex:** `^monocle-v1:[0-9a-f]{64}$`

Token is written to lock file on daemon start. Token rotates on every daemon restart (BC-2.01.008). Hook scripts read the lock file to obtain the current token.

### Compatibility Alias Header (ADR-0005)

**Header name:** `X-Claude-Code-Ide-Authorization`
**Value format:** `<64-hex-lowercase>` (raw hex; **no** `monocle-v1:` prefix)
**Accepted when:** canonical `X-Monocle-Authorization` is absent
**Priority:** canonical header takes priority when both are present; alias is ignored in that case
**WARN log:** whenever the alias path is entered (header present, whether the secret matches or not), the daemon emits a `tracing::warn!` deprecation log: `"hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization"`
**Constant-time comparison:** alias-path secret comparison uses `constant_time_eq` identically to the canonical path (NFR-010, INV-7 of BC-2.01.009)
**Phase-out:** alias is a Phase 1 compatibility shim per ADR-0005; removal target is Phase 2 or on operator opt-in configuration

### Dual-Absence Semantics

When **both** `X-Monocle-Authorization` and `X-Claude-Code-Ide-Authorization` are absent, the daemon returns HTTP 401 `{"error":"missing_auth_token"}` with **no WARN log**. The missing-auth response is not an alias-path response and does not trigger the deprecation log.

### Auth Response Examples

**Both headers absent:**
```json
HTTP/1.1 401 Unauthorized
{"error":"missing_auth_token"}
```
No WARN log emitted.

**Canonical header present, wrong secret:**
```json
HTTP/1.1 401 Unauthorized
{"error":"invalid_auth_token"}
```
No WARN log emitted (canonical path, no alias deprecation).

**Alias header present, wrong secret (canonical absent):**
```json
HTTP/1.1 401 Unauthorized
{"error":"invalid_auth_token"}
```
WARN log emitted: `"hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization"`.

**Alias header present, correct secret (canonical absent):**
```json
HTTP/1.1 200 OK
<endpoint response body>
```
WARN log emitted: `"hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization"`.

---

## Lock File Schema

**Contract:** BC-2.01.010 (contract_version field); BC-2.01.005 (lifecycle)
**Path:** `<runtime_dir>/monocle.lock`
**Format:** JSON (serde_json serialized); `contract_version` MUST be first key (field declaration order preserved by `serde_json`)
**Permissions:** `0o600` (owner-only read/write; NFR-009)

```json
{
  "contract_version": 1,
  "pid": <integer>,
  "port": <integer>,
  "authToken": "<64-hex-lowercase>",
  "startTimeUtc": "<ISO8601>",
  "app": "monocle",
  "version": "<semver>"
}
```

**Field Constraints:**
| Field | Type | Constraint |
|-------|------|------------|
| `contract_version` | integer | `1`; MUST be first key in serialized JSON |
| `pid` | integer | ≥ 1 |
| `port` | integer | OS-assigned ephemeral port (> 1024) |
| `authToken` | string | 64-char lowercase hex; matches `^[0-9a-f]{64}$` |
| `startTimeUtc` | string | ISO 8601 UTC `YYYY-MM-DDTHH:MM:SS.sssZ` (millisecond precision via `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")`) |
| `app` | string | `"monocle"` (exact); allows future hook-discovery tooling to filter lock files by app name |
| `version` | string | semver 2.0 of the daemon binary; same format as `/healthz` `version` field |

**Write protocol:** Written atomically via `tempfile::persist` (SS-conventions-anti-patterns.md §Atomic Writes). No exceptions.

---

## JSONL Ring Buffer Schema

**Contract:** BC-2.01.007
**Path:** `<runtime_dir>/monocle.ring.jsonl`
**Format:** One JSON object per line; `format_version` MUST be first key

```json
{"format_version":1,"session_id":"<uuid>","timestamp_micros":<i64>,"pid":<u32>,"hook_type":"<HookType>","tool_name":"<string>"|null,"tool_input":"<object>"|null}
```

**Field Constraints:**
| Field | Type | Constraint |
|-------|------|------------|
| `format_version` | u32 | `1`; MUST be first key in serialized JSON line |
| `session_id` | String | Claude Code session UUID |
| `timestamp_micros` | i64 | microseconds since Unix epoch; monotonically non-decreasing per session |
| `pid` | u32 | daemon PID at time of event; ≥ 1 |
| `hook_type` | String | one of: `PreToolUse`, `Notification`, `Stop`, `SessionStart`, `UserPromptSubmit` |
| `tool_name` | Option\<String\> | present for `PreToolUse`; field absent (not explicit null) for non-tool hook types |
| `tool_input` | Option\<serde_json::Value\> | present for `PreToolUse`; field absent (not explicit null) for non-tool hook types |

**Write protocol:** Appended atomically per event. Phase 2 trigger-trace reads this file by scanning for lines beginning with `{"format_version":1,`.

---

## Runtime Directory Resolution Chain

**Contract:** BC-2.01.005 Precondition 2

Resolution priority (highest to lowest):
1. `MONOCLE_RUNTIME_DIR` env var — if set and non-empty, use verbatim
2. `ProjectDirs::runtime_dir()` from `directories` crate — platform-native runtime dir
3. `ProjectDirs::data_local_dir()` — fallback if runtime_dir() returns None (macOS case; EC-057)
4. All three return None/empty → `DaemonStartError::RuntimeDirUnresolvable` (E-DAEMON-004)

`MONOCLE_RUNTIME_DIR=""` (empty string) is treated as unset — falls through to platform default (EC-060).

**Permissions:** runtime_dir created with `0o700` (owner-only; NFR-012).

---

## §Trace

### F-R105-1 PO closure — 2026-05-17T18:00:00Z

**Finding:** F-R105-1 CRITICAL — 3-way HookEventRecord schema divergence. `interface-definitions.md` §JSONL Ring Buffer Schema used 6 fields including `received_at` instead of the canonical 7-field schema from BC-2.01.007.

**Canonical source (BC-2.01.007 Postcondition 4):**
> `HookEventRecord` is defined in `monocle-runtime::ring` (NOT `monocle-core`) with the fields declared in declaration order: `format_version: u32`, `session_id: String`, `timestamp_micros: i64`, `pid: u32`, `hook_type: String`, `tool_name: Option<String>`, `tool_input: Option<serde_json::Value>`.

**SE-17c — Before (body-scope grep evidence):**

```
grep result — §JSONL Ring Buffer Schema inline JSON (pre-fix):
{"format_version":1,"session_id":"<uuid>","hook_type":"<HookType>","received_at":"<ISO8601>","tool_name":"<string>"|null,"tool_input":"<object>"|null}

grep result — §JSONL Ring Buffer Schema field table (pre-fix): 6 rows
| format_version | integer | `1`; MUST be first key in serialized JSON line |
| session_id     | string  | Claude Code session UUID |
| hook_type      | string  | one of: PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit |
| received_at    | string  | ISO 8601 UTC with millisecond precision |
| tool_name      | string or null | present for PreToolUse; null for non-tool hook types |
| tool_input     | object or null | present for PreToolUse; null for non-tool hook types |
```

**SE-17d — After (body-scope grep evidence):**

```
grep result — §JSONL Ring Buffer Schema inline JSON (post-fix):
{"format_version":1,"session_id":"<uuid>","timestamp_micros":<i64>,"pid":<u32>,"hook_type":"<HookType>","tool_name":"<string>"|null,"tool_input":"<object>"|null}

grep result — §JSONL Ring Buffer Schema field table (post-fix): 7 rows
| format_version   | u32                    | `1`; MUST be first key in serialized JSON line |
| session_id       | String                 | Claude Code session UUID |
| timestamp_micros | i64                    | microseconds since Unix epoch; monotonically non-decreasing per session |
| pid              | u32                    | daemon PID at time of event; >= 1 |
| hook_type        | String                 | one of: PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit |
| tool_name        | Option<String>         | present for PreToolUse; field absent (not explicit null) for non-tool hook types |
| tool_input       | Option<serde_json::Value> | present for PreToolUse; field absent (not explicit null) for non-tool hook types |
```

**Changes made:**
- Removed `received_at` field (was: string, ISO 8601 UTC)
- Added `timestamp_micros: i64` (microseconds since Unix epoch)
- Added `pid: u32` (daemon PID at time of event)
- Corrected field type language from plain-English to Rust type signatures matching BC-2.01.007
- Corrected `tool_name`/`tool_input` nullability prose: "field absent (not explicit null)" matches BC-2.01.007 EC-001 and the `#[serde(skip_serializing_if = "Option::is_none")]` invariant
- Version bumped: `1.0` → `1.1`

**Scope:** PO-only. No changes to BC-2.01.007, CAP-001-daemon-lifecycle.md, PRD top-level, or any other artifact. BA parallel track (CAP-001) untouched.

---

### F-R105-10/11 + GAP-R44-3 PO closure — 2026-05-17T19:00:00Z

**Finding:** F-R105-10/11 + GAP-R44-3 MED — Lock file schema used `auth_token` (snake_case) and was missing fields `startTimeUtc`, `app`, `version`. BC-2.01.010 Postcondition 1 is the authoritative field list and order: `contract_version`, `pid`, `port`, `authToken`, `startTimeUtc`, `app`, `version`.

**Canonical source (BC-2.01.010 Postcondition 1):**
> The lock file JSON is a valid JSON object containing at minimum these fields in the stated order: `contract_version` (first), `pid`, `port`, `authToken`, `startTimeUtc`, `app`, `version`.

**SE-17c — Before (body-scope grep evidence):**

```
§Lock File Schema JSON block (pre-fix):
{
  "contract_version": 1,
  "pid": <integer>,
  "port": <integer>,
  "auth_token": "<64-hex-lowercase>",
  "runtime_dir": "<absolute-path>",
  "started_at": "<ISO8601>"
}

§Lock File Schema field table (pre-fix): 6 rows
| contract_version | integer | `1`; MUST be first key in serialized JSON |
| pid              | integer | >= 1 |
| port             | integer | OS-assigned ephemeral port (> 1024) |
| auth_token       | string  | 64-char lowercase hex |
| runtime_dir      | string  | absolute path to runtime directory |
| started_at       | string  | ISO 8601 UTC `YYYY-MM-DDTHH:MM:SS.sssZ` |
```

**SE-17d — After (body-scope grep evidence):**

```
§Lock File Schema JSON block (post-fix):
{
  "contract_version": 1,
  "pid": <integer>,
  "port": <integer>,
  "authToken": "<64-hex-lowercase>",
  "startTimeUtc": "<ISO8601>",
  "app": "monocle",
  "version": "<semver>"
}

§Lock File Schema field table (post-fix): 7 rows
| contract_version | integer | `1`; MUST be first key in serialized JSON |
| pid              | integer | >= 1 |
| port             | integer | OS-assigned ephemeral port (> 1024) |
| authToken        | string  | 64-char lowercase hex; matches `^[0-9a-f]{64}$` |
| startTimeUtc     | string  | ISO 8601 UTC `YYYY-MM-DDTHH:MM:SS.sssZ` (millisecond precision via chrono) |
| app              | string  | `"monocle"` (exact) |
| version          | string  | semver 2.0 of the daemon binary |
```

**Changes made:**
- `auth_token` (snake_case) → `authToken` (camelCase) — per project convention (Claude Code IDE standard) and BC-2.01.010 Postcondition 1
- `runtime_dir` → removed (not in BC-2.01.010 Postcondition 1 canonical field list)
- `started_at` → renamed to `startTimeUtc` (camelCase; canonical field name per BC-2.01.010 Postcondition 1 and vp-010-lock-file-contract-version.md §Pre-conditions)
- Added `app: "monocle"` — per BC-2.01.010 Postcondition 3 (hook-discovery tooling filter field)
- Added `version: "<semver>"` — per BC-2.01.010 Postcondition 1 canonical field list
- `startTimeUtc` constraint updated with chrono format string per vp-010 §Pre-conditions
- Version bumped: `1.1` → `1.2`

**Note on `ver` vs `version`:** The task brief cited the missing field as `ver`, but BC-2.01.010 Postcondition 1 uses `version` as the authoritative field name. BC-2.01.010 is the canonical source of truth; `version` is used here per BC authority, not the brief's informal note.

**Scope:** PO-only. No changes to BC-2.01.010, VP-010, or any other artifact. PRD top-level bumped to v1.26.2 in same burst (F-R105-7 manifest pin).

---

### F-R107-1 PO closure — 2026-05-17T23:00:00Z

**Finding:** F-R107-1 CRITICAL — fabricated ADR-0005 path in frontmatter `inputs:`.

**SE-17f before/after evidence:**

**Before:** `inputs: [prd.md, architecture/adr/ADR-0005-dual-accept-auth-header.md]`
**After:** `inputs: [prd.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]`

Canonical filename verified via ARCH-INDEX and disk. Version bumped: 1.3 → 1.4; timestamp refreshed.

**Scope:** Frontmatter `inputs:` only. No body content changed.

---

### F-R106-5 + F-R106-6 PO closure — 2026-05-17T22:05:00Z

**Findings:**
- F-R106-5 HIGH — §Authentication Header Format only referenced canonical `X-Monocle-Authorization`; no dual-accept semantics, no WARN log behavior, no alias-path response examples per ADR-0005.
- F-R106-6 HIGH — `POST /shutdown` endpoint missing from interface-definitions despite BC-2.01.004, BC-2.01.008, BC-2.01.009, VP-004, and VP-009 all citing it.

**Canonical sources:** BC-2.01.009 v1.0.2 (INV-6 WARN log, INV-7 constant-time on both paths, EC-010 alias-path behavior, EC-011 canonical priority); BC-2.01.004 (10-second drain, EC-050 second-shutdown forced exit 2); ADR-0005 (dual-accept decision, alias header name, format constraint, phase-out rationale).

**SE-17c — Before (§Authentication Header Format — single-header, no dual-accept):**

```
## Authentication Header Format

**Contract:** BC-2.01.008, BC-2.01.009
**Header name:** `X-Monocle-Authorization`
**Value format:** `monocle-v1:<64-hex-lowercase>`
**Token entropy:** 32 bytes from `rand::rngs::OsRng` encoded as 64-character lowercase hex
**Token regex:** `^monocle-v1:[0-9a-f]{64}$`

Token is written to lock file on daemon start. Token rotates on every daemon restart (BC-2.01.008).
Hook scripts read the lock file to obtain the current token.
```

**SE-17d — After (§Authentication Header Format — dual-accept with canonical priority, alias WARN, dual-absence semantics, 4 response examples):**

```
## Authentication Header Format

**Contract:** BC-2.01.008, BC-2.01.009; **Dual-accept decision:** ADR-0005

### Canonical Header (Priority)
...`X-Monocle-Authorization: monocle-v1:<64-hex>` (unchanged spec)...

### Compatibility Alias Header (ADR-0005)
**Header name:** `X-Claude-Code-Ide-Authorization`
**Value format:** `<64-hex-lowercase>` (raw hex; no monocle-v1: prefix)
**Accepted when:** canonical absent. **Priority:** canonical wins when both present.
**WARN log:** emitted whenever alias path is entered (success or failure).
**Constant-time comparison:** alias path uses constant_time_eq identically to canonical.
**Phase-out:** Phase 2 or operator opt-in configuration.

### Dual-Absence Semantics
Both headers absent → HTTP 401 {"error":"missing_auth_token"}; no WARN log.

### Auth Response Examples
[4 examples: both-absent, canonical-wrong, alias-wrong+WARN, alias-correct+WARN]
```

**SE-17c — Before (/shutdown endpoint — absent from §HTTP API):**

```
§HTTP API endpoints documented: GET /healthz, GET /status, POST /hooks/* (5 endpoints).
POST /shutdown: mentioned only in Exit Code Semantics table (line 170) as the trigger for exit code 2.
No request/response schema, no field constraints, no edge cases.
```

**SE-17d — After (/shutdown endpoint — full §Endpoint: POST /shutdown (Authenticated, Admin) section):**

```
### Endpoint: POST /shutdown (Authenticated, Admin)

**Contract:** BC-2.01.004, BC-2.01.008, BC-2.01.009
**Router:** Authenticated router (256 KiB DefaultBodyLimit applies)
**Auth:** Canonical or alias (ADR-0005 dual-accept; WARN on alias)
**Request body:** empty or JSON null (ignored)
**Response 200:** {"status":"shutting_down"} — drain initiated
**Response 503 (EC-050):** {"error":"daemon_shutting_down"} — second /shutdown during drain → forced exit 2
**Edge cases table:** unauthenticated (→ 401), second /shutdown during drain (→ 503 + exit 2), alias header (→ 200 + WARN)
```

**Changes made:**
- §Authentication Header Format restructured into ### Canonical Header / ### Compatibility Alias Header / ### Dual-Absence Semantics / ### Auth Response Examples subsections
- Added `X-Claude-Code-Ide-Authorization` alias documentation (ADR-0005 scope)
- Added WARN log behavior documentation (BC-2.01.009 INV-6)
- Added constant-time note for alias path (BC-2.01.009 INV-7)
- Added dual-absence semantics paragraph (both-absent → missing_auth_token, no WARN)
- Added 4 auth response examples covering all auth-path branches
- Added full `### Endpoint: POST /shutdown (Authenticated, Admin)` section
- Added EC-050 (second /shutdown → exit 2) to /shutdown edge cases table
- Version bumped: 1.2 → 1.3; timestamp refreshed; ADR-0005 added to inputs

**Scope:** PO-only. No changes to BC-2.01.004, BC-2.01.008, BC-2.01.009, ADR-0005, or any other artifact.

---

### F-R108-2 + GAP-R47-1 PO closure — 2026-05-18T01:00:00Z

**Findings resolved:**
- F-R108-2 CRITICAL — `/status` and `/hooks/*` Auth specifications were single-header only. Per ADR-0005 §Decision: "router-level auth middleware for the authenticated router (hook endpoints + /status + /shutdown)" — all three endpoint groups share the same authenticated router middleware. `/shutdown` was corrected in Round 6B; this round extends dual-accept to `/status` and `/hooks/*`.
- GAP-R47-1 HIGH (PO part) — WARN log string in `§Compatibility Alias Header` and both auth response examples used String B (`"X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization"`) instead of canonical String A from BC-2.01.009 INV-6 (`"hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization"`). BC-2.01.009 is the canonical source per CLAUDE.md hierarchy.

**SE-17f — F-R108-2 before/after (Auth field, /status):**

**Before:** `**Auth:** \`X-Monocle-Authorization: monocle-v1:<64-hex-token>\``
**After:** `**Auth:** Canonical \`X-Monocle-Authorization: monocle-v1:<64-hex-token>\` **or** alias \`X-Claude-Code-Ide-Authorization: <64-hex>\` (ADR-0005 dual-accept applies; WARN log emitted on alias path)`

**SE-17f — F-R108-2 before/after (Auth field, /hooks/*):**

**Before:** `**Auth:** \`X-Monocle-Authorization: monocle-v1:<64-hex-token>\``
**After:** `**Auth:** Canonical \`X-Monocle-Authorization: monocle-v1:<64-hex-token>\` **or** alias \`X-Claude-Code-Ide-Authorization: <64-hex>\` (ADR-0005 dual-accept applies; WARN log emitted on alias path)`

**SE-17f — GAP-R47-1 before/after (WARN log string — 3 occurrences replaced):**

**Before (String B):** `"X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization"`
**After (String A, canonical per BC-2.01.009 INV-6):** `"hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization"`

Occurrences updated: §Compatibility Alias Header WARN log description (line 236); §Alias header present, wrong secret example (line 265); §Alias header present, correct secret example (line 272).

**Changes made:**
- `/status` endpoint Auth field: single-header → dual-accept per ADR-0005
- `/hooks/*` endpoint Auth field: single-header → dual-accept per ADR-0005
- WARN log string: String B → String A (canonical per BC-2.01.009 INV-6) at 3 body locations
- Version bumped: 1.4 → 1.5; timestamp refreshed to 2026-05-18T01:00:00Z

**Scope:** PO-only. No changes to BC files, VP files, ADR-0005, or ARCH-INDEX.
