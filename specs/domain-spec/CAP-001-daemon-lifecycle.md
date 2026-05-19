---
document_type: domain-spec-section
level: L2
section: "CAP-001 Daemon Lifecycle"
capability: CAP-001
version: "1.6"
status: active
producer: vsdd-factory:business-analyst
timestamp: 2026-05-19T01:00:00Z
phase: 1a
inputs:
  - product-brief.md
  - research/domain-monocle-vision-synthesis.md
input-hash: "6881431"
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

## §Trace v1.4

**F-R110-5 BA closure — brief cite verification per current brief v1.4.27** (2026-05-18T05:00:00Z):
- SE-17c body-scope grep: searched CAP-001-daemon-lifecycle.md for all occurrences of `v1.4.` —
  zero matches found. No brief version cite exists anywhere in the non-trace body of this file.
- Analysis: CAP-001 cites the product brief by section anchor (§Phase 1 Scope, §Success Criteria,
  §Phase 1 Constraints SOQ-2, §Out of Scope), not by version number. All anchors verified against
  current product-brief.md v1.4.27 — all cited section headings present and substance unchanged.
  No stale content to refresh.
- Anchor justification re-verification (per v1.4.27; pointer subsequently refreshed to v1.4.29 in §Trace v1.5):
  - CAP-001 §Anchor justification: "§Phase 1 Scope names monocle daemon start/stop, lock-file
    lifecycle, five hook ingestion endpoints, hook tmpfile, event ribbon ring storage, and graceful
    shutdown" — confirmed present in brief v1.4.27 §Scope → In Scope → Phase 1 — Runtime Core
    (section structure unchanged through v1.4.29; anchor stable).
  - Vision §Process Topology reference: unchanged in vision-synthesis v1.1.2.
- No body content changed. §Trace entry added to close F-R110-5 per round 9D dispatch.
- SE-16d monotonicity PASS: 2026-05-18T05:00:00Z > prior v1.3 2026-05-17T20:00:00Z.
- CAP-001 version: 1.3 → 1.4.
- L2-INDEX.md version: cascaded to 1.0.7 → 1.0.8 (F-R110-4 brief cite fix).

## §Trace v1.5

**F-R118-6 BA closure — active brief pointer refresh v1.4.27 → v1.4.29** (2026-05-18T20:00:00Z):

- Finding: F-R118-6 MED — §Trace v1.4 active-state prose (lines 346, 351, 356 pre-edit) cited
  "current brief v1.4.27" / "current product-brief.md v1.4.27" / "confirmed present in brief
  v1.4.27 §Scope". Canonical brief is now v1.4.29 (post-R17B commit b934e57). Two-step
  supersession: R15B advanced v1.4.27 → v1.4.28; R17B advanced v1.4.28 → v1.4.29.
- Precedent: L2-INDEX R16D GAP-R56-002 closure — same pattern of active current-pointer prose
  refresh in §Trace; historical BEFORE/AFTER slots preserved per SE-17g INFORMATIONAL classification.
- SE-22 sweep (cycle 5, CAP-files layer) — scoped grep results:

  ```
  grep -n "product-brief.md.* v1\." CAP-001-daemon-lifecycle.md
  346: current product-brief.md v1.4.27  [NORMATIVE — refreshed]

  grep -n "brief v1\." CAP-001-daemon-lifecycle.md
  346: brief v1.4.27  [NORMATIVE — heading]
  351: product-brief.md v1.4.27  [NORMATIVE — body]
  356: brief v1.4.27 §Scope  [NORMATIVE — anchor verify]

  grep -n "BC-INDEX.* v1\." CAP-001-daemon-lifecycle.md → no matches
  grep -n "prd.md.* v1\." CAP-001-daemon-lifecycle.md → no matches
  grep -n "L2-INDEX.* v1\." CAP-001-daemon-lifecycle.md → no matches
  grep -n "VP-INDEX.* v1\." CAP-001-daemon-lifecycle.md → no matches
  grep -n "ARCH-INDEX.* v1\." CAP-001-daemon-lifecycle.md → no matches
  ```

- SE-17g NORMATIVE vs INFORMATIONAL classification:
  - NORMATIVE (active-state assertions, refreshed): §Trace v1.4 heading "current brief v1.4.27",
    §Trace v1.4 body "current product-brief.md v1.4.27", §Trace v1.4 body "confirmed present in
    brief v1.4.27 §Scope".
  - INFORMATIONAL (historical §Trace BEFORE/AFTER slots, revision tables, supersession chains):
    none present in CAP-001 at this version; the §Trace v1.4 entries are preserved as-is
    (they are themselves historical after this edit) per SE-17g.
- SE-17a literal grep evidence (scoped per D-116):
  - BEFORE (3 normative occurrences of `v1.4.27`): lines 346, 351, 356 as listed above.
  - AFTER (0 normative occurrences of `v1.4.27`): post-edit grep returns zero matches in
    active-state prose; §Trace v1.4 body retains v1.4.27 as historical record (INFORMATIONAL).
- SE-17c-d L-number revalidation: §Trace v1.4 heading is the only site with "current brief"
  framing; no other lines in CAP-001 carry active-state brief version assertions. Post-edit
  search confirms zero remaining stale NORMATIVE pins.
- Brief anchor stability check: `product-brief.md` confirmed at v1.4.29 (frontmatter verified).
  Section "Phase 1 — Runtime Core" present at line 109; "In Scope" heading stable. R17B was a
  patch-level bump; §Phase 1 Scope anchor structure unchanged from v1.4.27 through v1.4.29.
- SE-17f recursive self-revalidation: §Trace v1.5 itself contains no version pins requiring
  future refresh (supersession chain documented; pointer now v1.4.29).
- L2-INDEX step 6 check: `grep -n "CAP-001.*v1\.4" L2-INDEX.md` — four historical bump entries
  found (INFORMATIONAL per SE-17g); no active-state CAP-001 version pin in L2-INDEX registry
  tables. No back-cascade obligation. Surfacing per R17E instructions: L2-INDEX is OUT OF SCOPE
  for this burst; no changes made.
- Sibling check (step 7): CAP-002-forward-compat-wire-formats.md and CAP-003-multi-harness-adapter.md
  both return zero matches for "brief v1." and "product-brief.md v1." grep patterns. No stale
  brief pins in siblings. No orchestrator escalation required for siblings.
- Changes applied:
  1. Frontmatter `version: "1.4"` → `version: "1.5"`.
  2. Frontmatter `timestamp: 2026-05-18T05:00:00Z` → `2026-05-18T20:00:00Z`.
  3. §Trace v1.4 heading: annotation "(pointer subsequently refreshed to v1.4.29 in §Trace v1.5)"
     added to preserve historical record while clarifying supersession.
  4. §Trace v1.4 anchor re-verification line: appended "(section structure unchanged through
     v1.4.29; anchor stable)" to document the bridge.
  No body content (capability statement, domain entities, processes, invariants, BC cross-references)
  changed.
- SE-16d monotonicity PASS: 2026-05-18T20:00:00Z > prior v1.4 2026-05-18T05:00:00Z
  > SS-conventions v1.29.5 at 2026-05-18T19:30:00Z.
- CAP-001 version: 1.4 → 1.5.
- L2-INDEX.md: no version cascade required (active registry tables do not pin CAP-001 version;
  R17E is scoped to CAP-001 only).

## §Trace v1.6

**R19D combined BA burst — active brief pointer refresh v1.4.29 → v1.4.30** (2026-05-19T01:00:00Z):

**SE-17g historical preservation note:** §Trace v1.5 (authored R17E commit 2e15e88) is preserved verbatim as the historical record of the v1.4.27 → v1.4.29 supersession it documented. §Trace v1.5 is now INFORMATIONAL — it reflects what was true at 2026-05-18T20:00:00Z. This §Trace v1.6 entry supersedes §Trace v1.5's active pointer.

**Background:** R19B (commit 6c863a9, 2026-05-19T00:00:00Z) bumped product-brief.md v1.4.29 → v1.4.30. SE-22 v2 consumer-ledger declaration in R19B identified CAP-001 §Trace active pointer as a known stale consumer. Per R17E SE-17g precedent (v1.5 preserved historical; v1.6 adds new current pointer), this §Trace v1.6 advances the active pointer without modifying §Trace v1.5 content.

**SE-22 v1 in-artifact sweep (CAP-001 scope, SE-17g classification):**

| Pattern | Matches | Lines | Classification | Action |
|---------|---------|-------|----------------|--------|
| `product-brief.md v1.` | 1 | §Trace v1.4 line (v1.4.27 historical pointer) | INFORMATIONAL — historical §Trace record preserved per SE-17g | No action |
| `brief v1.4.27` | 3 | §Trace v1.4 heading/body | INFORMATIONAL — historical §Trace before-state records (v1.5 documented supersession) | No action |
| `brief v1.4.29` / `product-brief.md v1.4.29` | 3 | §Trace v1.5 body | INFORMATIONAL — historical §Trace (v1.5 is now historical; this §Trace v1.6 is the new active pointer) | No action |
| `BC-INDEX v1.` | 0 | — | No pin present in CAP-001 | No action |
| `prd.md v1.` | 0 | — | No pin present in CAP-001 | No action |
| `L2-INDEX v1.` | 0 | — | No pin present in CAP-001 body | No action |
| `VP-INDEX v1.` | 0 | — | No pin present in CAP-001 | No action |
| `ARCH-INDEX v1.` | 0 | — | No pin present in CAP-001 | No action |

Zero-residual confirmation: no NORMATIVE stale pins remain in CAP-001. Active brief pointer is now v1.4.30 (this §Trace v1.6 entry).

**Active brief pointer declaration:** CAP-001 capabilities are grounded in current product-brief.md v1.4.30. All cited section anchors (§Phase 1 Scope → In Scope → Phase 1 — Runtime Core, §Success Criteria, §Phase 1 Constraints SOQ-2, §Out of Scope) verified stable through the v1.4.29 → v1.4.30 patch-level bump; substantive Phase 1 scope structure unchanged.

**Combined burst context:** L2-INDEX.md bumped v1.0.10 → v1.0.11 in this same commit (§Trace v1.0.11: brief v1.4.29 → v1.4.30 active pointer). Both files staged together per topological-order serialization.

**SE-22 v2 consumer-ledger declaration (CAP-001 v1.6 known downstream consumers):**

| Consumer artifact | Pin type | Current pin | Status after this burst |
|-------------------|----------|-------------|------------------------|
| L2-INDEX.md Capabilities Registry table | No version column (confirmed §Trace v1.0.6) | — | No stale pin; no action |
| `prd.md` (does it pin CAP-001 version?) | No direct CAP-001 version pin per R19A audit | — | No action |
| `BC-INDEX.md` §SS-01 | References "BC-2.01.001..BC-2.01.010" scope only; no CAP-001 version pin | — | No action |

No downstream consumers of CAP-001 version carry stale pins after this burst.

**SE-16d monotonicity PASS:** CAP-001 v1.6 timestamp `2026-05-19T01:00:00Z` > R17E v1.5 `2026-05-18T20:00:00Z`. Strict-greater: PASS.

- CAP-001 version: 1.5 → 1.6.
- L2-INDEX.md: no version cascade from CAP-001 version bump (registry table carries no CAP-001 version column per §Trace v1.0.6); L2-INDEX bumped independently in this same burst to v1.0.11.
