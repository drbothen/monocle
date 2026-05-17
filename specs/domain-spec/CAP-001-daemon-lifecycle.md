---
document_type: domain-spec-section
level: L2
section: "CAP-001 Daemon Lifecycle"
capability: CAP-001
version: "1.3"
status: active
producer: vsdd-factory:business-analyst
timestamp: 2026-05-17T20:00:00Z
phase: 1a
inputs:
  - product-brief.md
  - research/domain-monocle-vision-synthesis.md
input-hash: "a7a709c"
traces_to: L2-INDEX.md
subsystem: SS-01
bcs:
  - BC-2.01.001
  - BC-2.01.002
  - BC-2.01.003
  - BC-2.01.004
  - BC-2.01.005
  - BC-2.01.006
  - BC-2.01.007
  - BC-2.01.008
  - BC-2.01.009
  - BC-2.01.010
---

# CAP-001: Daemon Lifecycle

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`. This section
> describes the Daemon Lifecycle domain capability at the problem-domain level.
> Implementation contracts live in `behavioral-contracts/ss-01/`.

## Capability Statement

CAP-001 covers the full lifecycle of the monocle daemon: startup, hook-event
ingestion from AI coding harness subprocesses, in-memory and persistent ring
storage, crash-recovery checkpointing, graceful shutdown, and the lock-file
coordination mechanism that allows hook scripts to discover the daemon's
dynamically-assigned port and auth token.

**Anchor justification:** CAP-001 covers this scope because the product brief
§Phase 1 Scope names `monocle daemon start/stop`, lock-file lifecycle, five
hook ingestion endpoints, hook tmpfile, event ribbon ring storage, and graceful
shutdown as the core v1 deliverable. Vision §Process Topology establishes the
daemon as the permanent background process that is the sole ingestion boundary
for all harness hook events (reference: vision §Process Topology diagram and
§"The daemon is started once...").

## Domain Entities

### HookEvent

A structured message fired by an AI coding harness subprocess when a lifecycle
point is reached. The hook event is the atomic unit of information monocle
ingests and brokers.

| Attribute | Type | Description |
|-----------|------|-------------|
| hook_type | HookType enum | Which lifecycle point fired (PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit) |
| session_id | string | Stable identifier for the harness subprocess that fired the event |
| payload | structured map | Hook-type-specific fields (tool name, command, notification text, etc.) |
| decision_required | bool | True when the harness is blocking and awaiting a response |
| received_at | timestamp | Wall-clock time at which the daemon received the POST |

Phase 1 supports exactly 5 hook types (JC-2 resolution, EX-2 resolution). The
`PostToolUse` type is explicitly absent in Phase 1 per gene-source parity (brief
§Out of Scope, any-context BC-HOOK-007).

### HookEventRecord (JSONL ring entry)

The persisted form of a HookEvent as written to the async JSONL ring. Every record
carries a format version discriminant as its first key so Phase 2+ readers can
detect format evolution. Hook-type-specific context is carried in two structured
optional fields (`tool_name`, `tool_input`) rather than an opaque payload blob;
these fields are absent (not explicit-null) for hook types that carry no tool
context (e.g., SessionStart, UserPromptSubmit, Stop).

| Attribute | Type | Description |
|-----------|------|-------------|
| format_version | u32 | Always 1 in Phase 1. First field in declaration order; serializes first (FC-01). |
| session_id | String | Stable harness subprocess identifier. From HookEvent. |
| timestamp_micros | i64 | Microseconds since epoch at time of daemon receipt. |
| pid | u32 | Daemon process ID at time of record emission. |
| hook_type | String | Serialized HookType (one of the 5 Phase 1 hook types). |
| tool_name | Option&lt;String&gt; | Tool name for hook types that carry tool context (e.g., PreToolUse); absent otherwise. |
| tool_input | Option&lt;serde_json::Value&gt; | Structured tool arguments; absent for hook types with no tool context. |

### DaemonLockFile

The file at `runtime_dir/monocle.lock` (XDG `runtime_dir`, falling back through
`state_dir` and `data_dir` to `~/.monocle` per OQ-10). Written atomically at
daemon startup; removed at clean shutdown. The lock file is the coordination
primitive that connects hook scripts to the daemon.

| Attribute | Type | Description |
|-----------|------|-------------|
| port | u16 | OS-assigned TCP port bound by the daemon's axum HTTP server |
| token | string | Auth token in `monocle-v1:<64-char-hex>` format |
| contract_version | u32 | Lock file schema version (always 1 in Phase 1, SOQ-1) |
| pid | u32 | Daemon process ID for stale-lock detection |

### CrashCheckpoint

A recoverable snapshot written by the daemon before processing a sequence of hook
events. If the daemon crashes mid-sequence, the checkpoint enables it to restart
without losing committed ring entries.

| Attribute | Type | Description |
|-----------|------|-------------|
| ring_offset | u64 | Byte offset of the last fully-written JSONL record |
| session_ids | set of strings | Sessions known at checkpoint time |
| checkpoint_at | timestamp | Wall-clock time of the checkpoint write |

## Domain Processes

### P1: Daemon Startup

1. Daemon binds axum HTTP on an OS-assigned TCP port (OQ-04).
2. Daemon writes `DaemonLockFile` atomically at `runtime_dir/monocle.lock`
   with mode `0o600` (SOQ-1, SOQ-2).
3. Daemon writes the auth token to the lock file AFTER the port is bound —
   never before (token rotation invariant, SOQ-2).
4. Daemon opens the JSONL ring file for append; restores ring offset from
   the most recent `CrashCheckpoint` if one exists (DI-002 precondition met
   once these steps are complete).
5. Daemon signals readiness; TUI clients may now connect.

### P2: Hook Event Ingestion

1. A harness subprocess fires an HTTP POST to `POST /hooks/<type>` with the
   `X-Monocle-Authorization` header set to the token read from the lock file.
   Note: For the Claude Code harness (Phase 1 primary), hook scripts send the
   compatibility alias header `X-Claude-Code-Ide-Authorization: <raw-64-hex>`
   (header name hardcoded in Claude Code's source per BC-HOOK-016 deep ingest).
   The daemon dual-accepts both `X-Monocle-Authorization` (canonical, monocle-aware
   harnesses) and `X-Claude-Code-Ide-Authorization` (compatibility alias) per
   ADR-0005; canonical takes priority if both are present, and a WARN-level
   deprecation log is emitted when the alias is used.
2. Daemon validates the auth header (DI-005); rejects non-prefixed tokens with
   HTTP 401; rejects bodies over 256 KiB with HTTP 413 (BC-2.01.003).
3. Daemon deserializes the hook payload into a `HookEvent`.
4. Daemon writes a `HookEventRecord` to the JSONL ring — this write MUST complete
   before any acknowledgement is returned to the harness (DI-001, "tee invariant").
5. Daemon fans the event to all connected TUI clients via the bounded event bus.
6. If `decision_required` is true, daemon queues a `PromptModal` for the overlay
   stack and returns the harness-supplied response within the hook timeout budget
   (≤300ms for PreToolUse/Stop/SessionStart/UserPromptSubmit; ≤2000ms for
   Notification — brief §Success Criteria, BC-HOOK-022 gene source).

### P3: Graceful Shutdown

1. Daemon receives SIGTERM or SIGINT.
2. Daemon stops accepting new hook connections.
3. Daemon drains in-flight requests within a 10-second window (BC-2.01.004).
4. Daemon flushes any buffered JSONL ring entries to disk (OQ-06).
5. Daemon closes the Unix domain socket (UDS) used by TUI clients.
6. Daemon removes or marks the lock file with a shutdown marker.

### P4: Crash Recovery

1. Daemon starts and detects a stale `DaemonLockFile` (PID is dead).
2. Daemon removes the stale lock file.
3. Daemon reads the most recent `CrashCheckpoint` and restores ring offset.
4. Daemon resumes normal startup from P1 step 1.

## Domain Invariants

### DI-001: Tee Invariant

Every hook event received by the daemon MUST be written to the JSONL ring
before any acknowledgement is returned to the harness.

**Justification:** DI-001 is a business invariant because monocle's promise to
the developer is full observability of every harness lifecycle event. Silent
ring-write failures would cause gaps in the event ribbon and break trigger-trace
in Phase 2. Source: brief §Phase 1 Scope event ribbon, brief §Success Criteria
"Hook protocol parity".

### DI-002: Lock File Precondition

The daemon MUST NOT accept hook connections before the `DaemonLockFile` is fully
written with a valid port and token.

**Justification:** DI-002 is a business invariant because hook scripts discover
the daemon by reading the lock file; any race between lock-file write and port
binding would cause hook scripts to POST to a non-listening address, silently
stalling the harness. Source: brief §Phase 1 Scope, SOQ-2.

### DI-003: Token Write Order

The auth token MUST be written to the lock file after the port is bound — never
before.

**Justification:** DI-003 is a business invariant (the "token rotation invariant")
because hook scripts that read a token before the port is bound would attempt to
authenticate against a port that is not yet accepting connections. Source: brief
§Phase 1 Constraints SOQ-2.

## BC Cross-References

All 10 BCs in SS-01 operationalize CAP-001. See `behavioral-contracts/BC-INDEX.md`
§SS-01 for the full list with titles and file paths.

| BC ID | Title | Operationalizes |
|-------|-------|-----------------|
| BC-2.01.001 | Healthz Endpoint | DaemonLockFile precondition visibility |
| BC-2.01.002 | Status Endpoint | DaemonLockFile state query |
| BC-2.01.003 | Body Size Limit | Hook ingestion hardening |
| BC-2.01.004 | Graceful Shutdown | P3 Graceful Shutdown process |
| BC-2.01.005 | Lock File Atomic Lifecycle | DaemonLockFile entity + DI-002 + DI-003 |
| BC-2.01.006 | Crash Recovery Checkpoint | CrashCheckpoint entity + P4 Crash Recovery |
| BC-2.01.007 | JSONL Ring Format Version (FC-01) | HookEventRecord entity + DI-001 |
| BC-2.01.008 | Auth Token Wire Format (FC-06) | AuthToken format in DaemonLockFile |
| BC-2.01.009 | Auth Header Validation | DI-005 (see CAP-002) applied at ingestion |
| BC-2.01.010 | Lock File Contract Version Field | DaemonLockFile.contract_version |

## §Trace v1.1

**F-R105-1 BA closure** (2026-05-17T17:00:00Z):

- Finding: F-R105-1 CRITICAL — HookEventRecord schema divergence between CAP-001 (5-field,
  opaque `payload_json`) and BC-2.01.007 canonical 7-field schema.
- SE-17f before/after evidence:

  BEFORE (v1.0, lines ~79-84):
  ```
  | format_version | u32, first key | Always 1 in Phase 1. Enables forward evolution. |
  | hook_type      | string         | Serialized HookType                              |
  | session_id     | string         | From HookEvent                                   |
  | received_at_micros | i64        | Microseconds since epoch                         |
  | payload_json   | string         | Full hook payload as JSON                        |
  ```

  AFTER (v1.1, lines ~79-87):
  ```
  | format_version   | u32                       | Always 1 in Phase 1. First field; serializes first (FC-01). |
  | session_id       | String                    | Stable harness subprocess identifier. From HookEvent.       |
  | timestamp_micros | i64                       | Microseconds since epoch at time of daemon receipt.         |
  | pid              | u32                       | Daemon process ID at time of record emission.               |
  | hook_type        | String                    | Serialized HookType (one of the 5 Phase 1 hook types).     |
  | tool_name        | Option<String>            | Tool name for hook types that carry tool context; absent otherwise. |
  | tool_input       | Option<serde_json::Value> | Structured tool arguments; absent for hook types with no tool context. |
  ```

- Changes applied:
  1. Field count: 5 → 7 (added `pid: u32`; split `payload_json` into `tool_name` + `tool_input`).
  2. Field renamed: `received_at_micros` → `timestamp_micros` (BC-2.01.007 canonical spelling).
  3. Field order corrected to declaration order per BC-2.01.007 Postcondition 4.
  4. `hook_type` moved from position 2 → position 5 (BC-2.01.007 canonical order).
  5. Surrounding prose updated: replaced "opaque payload blob" with structured optional fields description.
  6. `Option<String>` and `Option<serde_json::Value>` semantics documented (absent vs explicit null).

- SE-16d monotonicity PASS: 2026-05-17T17:00:00Z > prior v1.0 creation 2026-05-17T14:00:00Z.
- Scope: BA-only. interface-definitions.md and BC-2.01.007.md not touched.
- CAP-002 and CAP-003: no HookEventRecord table found — see report for details.
- L2-INDEX.md version: bumped from 1.0.2 → 1.0.3 (§Trace entry added, entity registry unchanged).

## §Trace v1.2

**F-R105-6 + GAP-R44-2 BA closure — auth header rename** (2026-05-17T18:00:00Z):

- Finding: F-R105-6 + GAP-R44-2 MED — hook ingestion auth header named
  `X-Claude-Code-Ide-Authorization` but canonical project header is
  `X-Monocle-Authorization`. monocle is not Claude Code; adopting their header
  verbatim is a naming error at the domain level.
- SE-17c before grep (CAP-001 only — confirmed unique occurrence):
  ```
  CAP-001-daemon-lifecycle.md:134: `X-Claude-Code-Ide-Authorization` header set to the token read from the lock file.
  ```
- SE-17d after grep (post-rename — no remaining occurrences in CAP files):
  ```
  CAP-001-daemon-lifecycle.md:134: `X-Monocle-Authorization` header set to the token read from the lock file.
  ```
- Change applied: P2 Hook Event Ingestion step 1 — `X-Claude-Code-Ide-Authorization`
  → `X-Monocle-Authorization` (1 occurrence, 1 file).
- CAP-002 scan: no auth header name occurrences found. No change.
- CAP-003 scan: lines 59 and 89 contain the string `"claude-code"` as the stable
  `id()` return value for `ClaudeCodeModule` — this is a domain identifier for the
  Claude Code harness, NOT an HTTP header name. No rename applies.
- Out-of-scope files with `X-Claude-Code-Ide-Authorization` surfaced for specialist
  routing (not edited): `dtu-assessment.md` (×10), `product-brief.md` (×2),
  `architecture/SS-daemon-lifecycle.md` (×1). These carry the header string as
  the *wire value that Claude Code actually sends* — the correct rename there is an
  architectural/PO decision, not a BA CAP fix. Surfaced to orchestrator.
- Security-adjacent note: the header rename at the domain level does not change the
  auth token format (`monocle-v1:<64-char-hex>`) — only the header name.
  Token format security properties are unchanged. No security-reviewer escalation
  required beyond the out-of-scope file list above.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior v1.1 2026-05-17T17:00:00Z.
- CAP-001 version: 1.1 → 1.2.
- L2-INDEX.md version: 1.0.3 → 1.0.4.

## §Trace v1.3

**T-128o ADR-0005 alias note propagation — dual-accept auth header clarification** (2026-05-17T20:00:00Z):

- Finding: Architect ADR-0005 (commit 932f4e0) established dual-accept for auth
  headers: canonical `X-Monocle-Authorization` (monocle-aware harnesses) AND
  compatibility alias `X-Claude-Code-Ide-Authorization` (real Claude Code hook
  scripts, whose header name is hardcoded in Go source per BC-HOOK-016 deep ingest).
  CAP-001 §P2 step 1 (at v1.2) only mentioned the canonical header, leaving the
  alias undocumented at the domain level. This T-128o dispatch propagates the
  dual-accept behavior into the L2 domain spec.
- SE-17f before/after evidence:

  BEFORE (v1.2, §P2 step 1):
  ```
  1. A harness subprocess fires an HTTP POST to `POST /hooks/<type>` with the
     `X-Monocle-Authorization` header set to the token read from the lock file.
  ```

  AFTER (v1.3, §P2 step 1 with alias note):
  ```
  1. A harness subprocess fires an HTTP POST to `POST /hooks/<type>` with the
     `X-Monocle-Authorization` header set to the token read from the lock file.
     Note: For the Claude Code harness (Phase 1 primary), hook scripts send the
     compatibility alias header `X-Claude-Code-Ide-Authorization: <raw-64-hex>`
     (header name hardcoded in Claude Code's source per BC-HOOK-016 deep ingest).
     The daemon dual-accepts both `X-Monocle-Authorization` (canonical, monocle-aware
     harnesses) and `X-Claude-Code-Ide-Authorization` (compatibility alias) per
     ADR-0005; canonical takes priority if both are present, and a WARN-level
     deprecation log is emitted when the alias is used.
  ```

- SE-17c body-scope grep: searched CAP-001 for all auth header name references beyond
  §P2 step 1. Findings:
  - §P2 step 2: "Daemon validates the auth header (DI-005)" — names no specific header;
    no change needed.
  - BC-2.01.009 cross-reference row: "Auth Header Validation" title — no header name
    spelled out; no change needed.
  - HookEventRecord and DaemonLockFile entity tables: no auth header name reference;
    no change needed.
- SE-17d after grep: no remaining references to auth header naming that lack the
  alias clarification. §P2 step 1 is the sole location where header naming is
  specified at the domain level.
- SE-17c-d body-scope: one location updated; all other CAP-001 sections clean.
- SE-16d monotonicity PASS: 2026-05-17T20:00:00Z > prior v1.2 2026-05-17T18:00:00Z.
- CAP-001 version: 1.2 → 1.3.
- L2-INDEX.md version: cascaded to 1.0.5 → 1.0.6 (CAP-001 version bump in §CAP Files registry).
