---
document_type: prd-supplement-interface-definitions
level: L3
version: "1.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T12:30:00Z
phase: 1a
inputs: [prd.md]
input-hash: "742464a"
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
**Auth:** `X-Monocle-Authorization: monocle-v1:<64-hex-token>`

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
**Auth:** `X-Monocle-Authorization: monocle-v1:<64-hex-token>`

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

## Exit Code Semantics (Daemon Process)

| Code | Meaning | Trigger |
|------|---------|---------|
| 0 | Clean exit | Normal graceful shutdown after drain completes |
| 1 | Fatal error | `DaemonStartError::RuntimeDirUnresolvable` (E-DAEMON-004); lock file conflict (E-LOCK-001); lock file write failure (EC-051) |
| 2 | Forced stop | Second `POST /shutdown` during drain (EC-050 — admin forced-stop) |

---

## Authentication Header Format

**Contract:** BC-2.01.008, BC-2.01.009
**Header name:** `X-Monocle-Authorization`
**Value format:** `monocle-v1:<64-hex-lowercase>`
**Token entropy:** 32 bytes from `rand::rngs::OsRng` encoded as 64-character lowercase hex
**Token regex:** `^monocle-v1:[0-9a-f]{64}$`

Token is written to lock file on daemon start. Token rotates on every daemon restart (BC-2.01.008). Hook scripts read the lock file to obtain the current token.

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
  "auth_token": "<64-hex-lowercase>",
  "runtime_dir": "<absolute-path>",
  "started_at": "<ISO8601>"
}
```

**Field Constraints:**
| Field | Type | Constraint |
|-------|------|------------|
| `contract_version` | integer | `1`; MUST be first key in serialized JSON |
| `pid` | integer | ≥ 1 |
| `port` | integer | OS-assigned ephemeral port (> 1024) |
| `auth_token` | string | 64-char lowercase hex |
| `runtime_dir` | string | absolute path to runtime directory |
| `started_at` | string | ISO 8601 UTC `YYYY-MM-DDTHH:MM:SS.sssZ` |

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
