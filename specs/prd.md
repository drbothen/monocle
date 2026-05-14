---
document_type: prd
level: L3
version: "1.0"
status: draft
producer: product-owner
phase: phase-1-spec-crystallization
timestamp: 2026-05-14T21:00:00Z
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
traces_to: "product-brief.md v1.4.23; vision-synthesis v1.1.2; SS-daemon-lifecycle.md v1.0.7; SS-core-types-and-abi.md v1.2.8; SS-engine-module.md v1.1.15; 16 pre-staged BCs; D-047 strict; 18+ META defense layers; STATE.md phase-1-spec-crystallization-entry-pending"
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
| D-1 | Hook-protocol ingestion at OS-assigned port with versioned auth token | BC-AUTH-001, BC-AUTH-002, BC-LOCK-001 |
| D-2 | VecDeque overlay stack — both concurrent prompts visible simultaneously | BC-ENGINE-001, BC-ENGINE-002 |
| D-3 | Forward-compatible ABI via const + non_exhaustive + proto schema_version | BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 |
| D-4 | FactoryAdapter open trait — VsddFactoryAdapter ships in Phase 1; WASM loadable in Phase 3 | BC-FACTORY-001, BC-FACTORY-002 |
| D-5 | ClaudeCodeModule strict-basename detect — no false positives from claude-squad/claudio | BC-ENGINE-002 |
| D-6 | JSONL ring with format_version first key — Phase 2 trigger-trace can read Phase 1 history | BC-RING-001 |

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

BCs are grouped by domain subsystem. The 16 pre-staged BCs span four functional domains:

| Domain | BC IDs | Subsystem Anchor |
|--------|--------|-----------------|
| Hook Ingestion — Ring Buffer + Drain | BC-RING-001 | SS-daemon-lifecycle.md §Drain |
| Daemon Authentication | BC-AUTH-001, BC-AUTH-002 | SS-daemon-lifecycle.md §Daemon Lifecycle Protocol |
| Lock File Discovery | BC-LOCK-001 | SS-daemon-lifecycle.md §Lock File Discovery Policy |
| Core ABI Stability | BC-ABI-001, BC-ABI-002 | SS-core-types-and-abi.md §ABI Version Constant |
| Enum Extensibility | BC-TYPES-001 | SS-core-types-and-abi.md §Enum Extensibility |
| Factory Adapter Trait | BC-FACTORY-001, BC-FACTORY-002 | SS-core-types-and-abi.md §FactoryAdapter Trait |
| Proto Wire Schemas | BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 | SS-core-types-and-abi.md §Prost Wire Schemas |
| EngineModule Trait | BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003 | SS-engine-module.md §EngineModule Trait Signature |

---

## 3. Full Behavioral Contract Specifications

### BC-RING-001 — JSONL Ring Format Version (FC-01)

**Priority:** P0 — Forward-compatibility contract; locked pre-Phase-1 by human authorization.

**Source:** SS-daemon-lifecycle.md v1.0.7 §Drain

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
- Source: SS-daemon-lifecycle.md v1.0.7 §Drain
- FC: FC-01 (JSONL ring format versioning)
- Brief: §Forward-compatibility contracts / JSONL ring sub-bullet

---

### BC-AUTH-001 — Auth Token Wire Format (FC-06)

**Priority:** P0 — Security contract; locked pre-Phase-1 by human authorization.

**Source:** SS-daemon-lifecycle.md v1.0.7 §Daemon Lifecycle Protocol §Start Sequence

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
- Integration test in `monocle-runtime/tests/auth.rs`: reads lock file after daemon start, asserts `authToken` matches regex; presents `monocle-v1:<authToken>` to `/status`, asserts HTTP 200.
- Test name: `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.7 §Daemon Lifecycle Protocol §Start Sequence
- FC: FC-06 (versioned auth token prefix)
- Brief: §Forward-compatibility contracts / Versioned auth token sub-bullet

---

### BC-AUTH-002 — Auth Token Prefix Rejection (FC-06)

**Priority:** P0 — Security contract; locked pre-Phase-1.

**Source:** SS-daemon-lifecycle.md v1.0.7 §Daemon Lifecycle Protocol §Start Sequence

**Preconditions:**
1. The monocle daemon is running with a valid lock file.
2. A request arrives at any authenticated endpoint (`/hooks/*`, `/status`, `/shutdown`) with an `X-Monocle-Authorization` header whose value does NOT begin with `monocle-v1:`.

**Postconditions:**
1. The daemon returns HTTP 401 with body `{"error":"invalid_auth_token_format"}` before performing any secret comparison.
2. The rejection is immediate upon detecting a non-`monocle-v1:` prefix — no constant-time comparison is performed for non-prefixed tokens (the prefix check acts as a fast-fail gate, not a timing oracle, because the prefix itself is not secret).
3. Phase 4 OAuth2 `Authorization: Bearer <token>` headers on Phase 1 endpoints receive HTTP 401 with the same body. Phase 4 federation tokens are valid only on the separate russh/IPC federation channel.

**Invariants:**
1. Rejection of non-prefixed tokens MUST occur before any secret comparison to prevent timing oracle attacks where an attacker determines whether a non-prefixed string matches the secret.
2. The prefix `monocle-v1:` is a public constant (`const TOKEN_PREFIX: &str = "monocle-v1:";`) in the auth middleware.
3. A `monocle-v2:` prefix (future protocol version) is also rejected with HTTP 401 on Phase 1 endpoints — Phase 1 validates only `monocle-v1:`.

**Edge Cases:**

EC-007: Empty `X-Monocle-Authorization` header. Returns HTTP 401 `{"error":"invalid_auth_token_format"}` — empty string does not begin with `monocle-v1:`.

EC-008: `X-Monocle-Authorization` header absent entirely. Returns HTTP 401 `{"error":"missing_auth_token"}` — distinct error from malformed token.

EC-009: `X-Monocle-Authorization: monocle-v1:` (prefix present but no hex part). Passes the prefix check but fails the secret comparison (empty hex string never matches the 64-char secret). Returns HTTP 401 `{"error":"invalid_auth_token"}` (distinct body to distinguish malformed-after-prefix from missing-prefix).

**Canonical Test Vectors:**

| Scenario | Input Header | Expected |
|----------|-------------|----------|
| Bearer token (Phase 4 OAuth2 attempt on Phase 1 route) | `Authorization: Bearer fake-token` | HTTP 401 `{"error":"invalid_auth_token_format"}` |
| Bare token (no prefix) | `X-Monocle-Authorization: deadbeef...64chars` | HTTP 401 `{"error":"invalid_auth_token_format"}` |
| Wrong version prefix | `X-Monocle-Authorization: monocle-v2:deadbeef...64chars` | HTTP 401 `{"error":"invalid_auth_token_format"}` |
| Missing header entirely | (no Authorization headers) | HTTP 401 `{"error":"missing_auth_token"}` |
| Prefix only, no hex | `X-Monocle-Authorization: monocle-v1:` | HTTP 401 `{"error":"invalid_auth_token"}` |

**Verification:**
- Integration test in `monocle-runtime/tests/auth.rs`: for each test vector above, sends the specified header to `/status` and asserts the expected HTTP status + body.
- Test name: `test_BC_AUTH_002_non_prefixed_token_rejection`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.7 §Daemon Lifecycle Protocol §Start Sequence
- FC: FC-06 (F-FC-I005 Phase 4 OAuth2 clarification)
- Brief: §Forward-compatibility contracts / Versioned auth token sub-bullet

---

### BC-LOCK-001 — Lock File Contract Version Field

**Priority:** P0 — Forward-compatibility contract.

**Source:** SS-daemon-lifecycle.md v1.0.7 §Daemon Lifecycle Protocol §Start Sequence; SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging

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
- Integration test in `monocle-runtime/tests/daemon_lock.rs`: starts daemon, reads lock file, asserts `contract_version == 1` is first key via `serde_json::Value::Object` iteration (which preserves insertion order for `serde_json::Map<String, Value>`).
- Test name: `test_BC_LOCK_001_contract_version_first_key`

**Traceability:**
- Source: SS-daemon-lifecycle.md v1.0.7 §Lock File Discovery Policy
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
- Integration test: `GET /status | jq .abi_version == 1`.
- Test name: `test_BC_ABI_001_status_abi_version_field`

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
1. The `monocle-core` crate compiles with `cargo clippy --workspace -- -D warnings`.

**Postconditions:**
1. Every `pub` enum in `monocle-core` carries `#[non_exhaustive]` unless explicitly exempted by an ADR.
2. At Phase 1 PRD dispatch, the exhaustive-enum forbidden list contains exactly two entries: `Phase1Permission` and `ClaudeCodeTool` (both documented in ADR-0004).
3. Any new exemption requires a new ADR before the code compiles in CI. No exemption is granted by inline comment or spec prose alone.
4. The mandatory non-exhaustive enums include at minimum: `HookType`, `HookEvent`, `DenyReason`, `AllowPattern`, `DenyPattern`, `BlockingSeverity`, `SessionStatus`, `HookDecision`, `DeferUntil`.

**Invariants:**
1. A custom clippy lint enforces this at CI time — `#[allow(non_exhaustive_omitted_patterns)]` is forbidden in monocle source files (see SS-conventions-anti-patterns.md).
2. Adding a variant to any `#[non_exhaustive]` enum (except `Phase1Permission`) is NOT a breaking change and does NOT require a SemVer-major version bump.
3. `Phase1Permission` is exhaustive because the TUI permission dispatcher must handle every variant at compile time. Phase 3 adds `monocle-plugin-sdk::PluginPermission` as a separate enum rather than extending `Phase1Permission`.

**Edge Cases:**

EC-016: New enum added in a future PR without `#[non_exhaustive]`. CI clippy lint must reject it unless an ADR is filed concurrently.

EC-017: `ClaudeCodeTool::Unknown(String)` catch-all variant. This is the runtime safety net for tools added by Anthropic between monocle releases. It does NOT make `ClaudeCodeTool` non-exhaustive in the Rust sense — the enum is still exhaustive (every `match` must cover all variants including `Unknown`). The `Unknown` catch-all is the intended escape valve that keeps the enum exhaustive without breaking on new tools.

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| `cargo clippy --workspace -- -D warnings` with a new `pub enum Foo { A, B }` in monocle-core (missing `#[non_exhaustive]`) | Compile error |
| `cargo clippy` with `Phase1Permission` lacking `#[non_exhaustive]` | No error (ADR-0004 exemption) |

**Verification:**
- `cargo clippy` with the Phase 1 workspace; `rustdoc` confirms all public enums carry `#[non_exhaustive]` except the ADR-0004 exemptions.
- Test name: `test_BC_TYPES_001_non_exhaustive_enum_coverage` (compile-time via clippy deny-list)

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
- `cargo check` with Phase 1 workspace; `rustdoc` confirms public trait surface and no sealed supertrait.
- Test name: `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound`

**Traceability:**
- Source: SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait
- FC: FC-04 (CRITICAL)
- Brief: §Forward-compatibility contracts / FactoryAdapter trait sub-bullet

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
- Brief: §Success Criteria (factory pattern detection row)

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
4. The oneof `event` in `HookEnvelope` uses field numbers 10–14 for the five event types (SessionStart=10, UserPromptSubmit=11, PreToolUse=12, Notification=13, Stop=14). These are in the Phase 1 reserved range (1–99).
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

EC-031: `on_hook()` called with an unrecognized `HookEvent` variant (future Phase 4 addition). Since `HookEvent` is `#[non_exhaustive]`, all match sites have a wildcard arm. `on_hook()` should return `HookResponse::new(HookDecision::Allow)` as the fail-open default for unrecognized event types (per BC-ENGINE-002 Phase 1 default).

**Canonical Test Vectors:**

| Scenario | Expected |
|----------|----------|
| `cargo check` with Phase 1 workspace | Compiles without error; no `private::Sealed` supertrait |
| `rustdoc` output | All 5 trait methods visible; no sealed supertrait |

**Verification:**
- `cargo check` with Phase 1 workspace; `rustdoc` confirms all types are publicly accessible and trait has no `private::Sealed` supertrait.
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
- Unit test in `monocle-runtime/tests/engine_module.rs` with all 5 (minimum 3 per spec) test vectors above.
- Test name: `test_BC_ENGINE_002_claude_code_module_detect`

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
- Test in `monocle-runtime/tests/engine_module.rs`. Sync half uses `temp_env::with_vars`; async half uses `temp_env::async_with_vars` in a separate `#[tokio::test]`.
- Dev dependency: `temp-env = { version = "^0.3", features = ["async_closure"] }` in `monocle-runtime` `[dev-dependencies]`.
- Test name: `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich`

**Traceability:**
- Source: SS-engine-module.md v1.1.15 §Behavioral Contracts BC-ENGINE-002-ERR
- CLAUDE.md SOUL #4 (no silent fallback for unresolvable platform home directory)

---

### BC-ENGINE-003 — ClaudeCodeModule Inherent Methods

**Priority:** P0 — Phase 1 hook path routing contract.

**Source:** SS-engine-module.md v1.1.15 §Phase 1 Implementation: ClaudeCodeModule §Struct-level inherent operations

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
1. The 5 hook path strings exactly match the canonical endpoint set from brief §Scope: `PostToolUse` is NOT included (JC-2 gene-source parity).
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
- Unit test in `monocle-runtime/tests/engine_module.rs`: asserts `module.hook_paths().len() == 5` with the exact path string for each `HookType`.
- Test name: `test_BC_ENGINE_003_hook_paths_five_entries`

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
| E-AUTH-001 | Authentication | Broken | HTTP 401 | `{"error":"invalid_auth_token_format"}` | BC-AUTH-002 |
| E-AUTH-002 | Authentication | Broken | HTTP 401 | `{"error":"missing_auth_token"}` | BC-AUTH-002 EC-008 |
| E-AUTH-003 | Authentication | Broken | HTTP 401 | `{"error":"invalid_auth_token"}` | BC-AUTH-002 EC-009 |
| E-DAEMON-001 | Body Size | Broken | HTTP 413 | `{"error":"payload_too_large","limit_bytes":262144}` | BC-DAEMON-003 (SS-daemon-lifecycle.md) |
| E-DAEMON-002 | Shutdown | Degraded | HTTP 503 | `{"error":"daemon_shutting_down"}` with `Retry-After: 10` header | SS-daemon-lifecycle.md §Shutdown Signal Handling |
| E-DAEMON-003 | Liveness | Broken | HTTP 503 | `{"status":"shutting_down"}` | SS-daemon-lifecycle.md §Health and Status Endpoints |
| E-LOCK-001 | Lock File | Broken | Exit 1 | `ERROR: daemon already running at pid=<N>; exiting` | BC-LOCK-001 §Start Sequence step 2b |
| E-LOCK-002 | Lock File | Degraded | WARN log | `WARN: stale lock file removed` | BC-LOCK-001 §Start Sequence step 2c |
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
| Hook-protocol ingestion at OS-assigned port | Daemon binds on OS-assigned port; port written to lock file; hook scripts read absolute lock file path (no directory scan, no "highest-port-wins" collision) | BC-AUTH-001, BC-AUTH-002, BC-LOCK-001 | Integration test: lock file read after start; port confirmed reachable; no `~/.claude/ide/` scanning |
| VecDeque overlay stack for concurrent prompts | Both permission prompts visible simultaneously; `[↑↓]` rotates stack; `Esc` hides without rejecting | BC-ENGINE-001, BC-ENGINE-002 (on_hook → HookDecision::Defer) | Killer scenario: 2 concurrent PreToolUse hooks arrive; TUI shows both prompts; 4 keystrokes resolve both |
| Versioned ABI with forward-compatible extension | `MONOCLE_ABI_VERSION = 1` const; `#[non_exhaustive]` on all public enums; proto `schema_version = 1` first field | BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-PROTO-001a, BC-PROTO-001b | Compile-time assertions; clippy lint; wire-format round-trip test |
| FactoryAdapter open trait — Phase 3 WASM extensibility | `VsddFactoryAdapter` ships Phase 1 as a static implementation; WASM plugin SDK in Phase 3 uses the same trait without code changes | BC-FACTORY-001, BC-FACTORY-002 | `cargo check` no sealed supertrait; self-referential detection test |
| Strict-basename detection (no false positives) | `detect()` uses `exe_path.file_name()` == `"claude"` or `"claude.js"`; rejects `claude-squad`, `claudio`, `claude-code-router` | BC-ENGINE-002 | Unit tests with 5 synthetic ProcessSnapshot instances |
| JSONL ring with format_version first key | Phase 2 trigger-trace can read Phase 1 history; version field allows future format evolution | BC-RING-001 | Unit test: serialized JSONL line begins with `{"format_version":1,` |

---

## 7. Requirements Traceability Matrix

| BC ID | Brief Section | Architecture Source | Priority | Test File | Test Type |
|-------|--------------|--------------------|---------|-----------|----|
| BC-RING-001 | §Forward-compatibility contracts §JSONL ring | SS-daemon-lifecycle.md v1.0.7 §Drain | P0 | `monocle-runtime/tests/jsonl_ring.rs` | Unit |
| BC-AUTH-001 | §Forward-compatibility contracts §Versioned auth token | SS-daemon-lifecycle.md v1.0.7 §Start Sequence | P0 | `monocle-runtime/tests/auth.rs` | Integration |
| BC-AUTH-002 | §Forward-compatibility contracts §Versioned auth token | SS-daemon-lifecycle.md v1.0.7 §Start Sequence | P0 | `monocle-runtime/tests/auth.rs` | Integration |
| BC-LOCK-001 | §Forward-compatibility contracts §JSONL ring (cross-ref) | SS-daemon-lifecycle.md v1.0.7 §Lock File Discovery Policy | P0 | `monocle-runtime/tests/daemon_lock.rs` | Integration |
| BC-ABI-001 | §Forward-compatibility contracts §monocle-core ABI | SS-core-types-and-abi.md v1.2.8 §ABI Version Constant | P0 | `monocle-runtime/tests/` | Integration |
| BC-ABI-002 | §Forward-compatibility contracts §monocle-core ABI | SS-core-types-and-abi.md v1.2.8 §ABI Version Constant | P0 | `monocle-core/tests/abi_stability.rs` | Lint/compile |
| BC-TYPES-001 | §Forward-compatibility contracts §Public enum extensibility | SS-core-types-and-abi.md v1.2.8 §Enum Extensibility | P0 | Clippy workspace lint | Clippy |
| BC-FACTORY-001 | §Forward-compatibility contracts §FactoryAdapter trait | SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait | P0 | `monocle-core/tests/` | Compile/rustdoc |
| BC-FACTORY-002 | §Success Criteria §Factory pattern detection | SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait §VsddFactoryAdapter | P0 | `monocle-core/tests/factory_self_referential.rs` | Integration |
| BC-PROTO-001a | §Forward-compatibility contracts §Prost wire schemas | SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas | P0 | `monocle-proto/tests/wire_field_order.rs` | Unit |
| BC-PROTO-001b | §Forward-compatibility contracts §Prost wire schemas | SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas | P0 | `monocle-proto/tests/schema_version.rs` | Unit |
| BC-PROTO-002 | §Forward-compatibility contracts §Prost wire schemas | SS-core-types-and-abi.md v1.2.8 §Prost Wire Schemas | P1 | Phase 4 integration test (future) | Integration |
| BC-ENGINE-001 | §Scope §ClaudeCodeModule | SS-engine-module.md v1.1.15 §EngineModule Trait Signature | P0 | `cargo check` + rustdoc | Compile/rustdoc |
| BC-ENGINE-002 | §Scope §ClaudeCodeModule | SS-engine-module.md v1.1.15 §ClaudeCodeModule | P0 | `monocle-runtime/tests/engine_module.rs` | Unit |
| BC-ENGINE-002-ERR | §Scope §ClaudeCodeModule | SS-engine-module.md v1.1.15 §BC-ENGINE-002-ERR | P0 | `monocle-runtime/tests/engine_module.rs` | Unit (env-isolation) |
| BC-ENGINE-003 | §Scope §ClaudeCodeModule | SS-engine-module.md v1.1.15 §Struct-level inherent operations | P0 | `monocle-runtime/tests/engine_module.rs` | Unit |

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

All per-contract edge cases (EC-001 through EC-039) are embedded in Section 3 within each BC. This index provides a cross-reference for sweep tooling.

| EC ID | BC | Category | Description |
|-------|----|----------|-------------|
| EC-001 | BC-RING-001 | JSONL serialization | `tool_name`/`tool_input` None for non-tool events; format_version still first |
| EC-002 | BC-RING-001 | Ring buffer | Near-maximum payload size (256 KiB line); rotation handles without truncation |
| EC-003 | BC-RING-001 | Crash recovery | Ring buffer file truncated mid-line; Phase 2 readers skip incomplete trailing lines |
| EC-004 | BC-AUTH-001 | Token lifecycle | Token rotation on daemon restart; scripts reading from lock file always have current token |
| EC-005 | BC-AUTH-001 | Atomic write | Lock file write failure (filesystem full); daemon exits without partial lock file |
| EC-006 | BC-AUTH-001 | Lock file cross-ref | `contract_version` field cross-references BC-LOCK-001 |
| EC-007 | BC-AUTH-002 | Empty header | Empty `X-Monocle-Authorization` value → HTTP 401 format error |
| EC-008 | BC-AUTH-002 | Missing header | No auth header → HTTP 401 missing_auth_token |
| EC-009 | BC-AUTH-002 | Prefix-only token | `monocle-v1:` with no hex → HTTP 401 invalid_auth_token |
| EC-010 | BC-LOCK-001 | Future version | `contract_version: 99` → WARN log + skip |
| EC-011 | BC-LOCK-001 | Type coercion | `contract_version` stored as string → graceful coerce or skip |
| EC-012 | BC-LOCK-001 | Missing field | No `contract_version` key → same treatment as EC-010 |
| EC-013 | BC-ABI-001 | Phase 3 forward | Plugin SDK reads `abi_version` from /status to version-gate loading |
| EC-014 | BC-ABI-001 | Phase 4 forward | Federation peer with different ABI version → HTTP 409 |
| EC-015 | BC-ABI-002 | Compile-time | Plugin SDK compile-time assertion fails if ABI version changes without SDK update |
| EC-016 | BC-TYPES-001 | Clippy enforcement | New enum without `#[non_exhaustive]` → CI compile error |
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

---

## 10. Glossary

| Term | Definition | Source |
|------|-----------|--------|
| ABI | Application Binary Interface. `MONOCLE_ABI_VERSION` identifies the stable contract between `monocle-core` and its consumers (plugin SDK, federation layer). | SS-core-types-and-abi.md §ABI Version Constant |
| BC | Behavioral Contract. A testable specification with preconditions, postconditions, and at least one canonical test vector. | This document |
| `ClaudeCodeModule` | Phase 1 built-in `EngineModule` implementation for Claude Code harness integration. Defined in `monocle-runtime`. | SS-engine-module.md §Phase 1 Implementation |
| DTU | Digital Twin Universe. Behavioral clone of the Claude Code hook protocol for testing fidelity and regression detection. | dtu-assessment.md |
| `EngineModule` | Trait in `monocle-core::engine` abstracting over AI coding harness adapters. Open (not sealed). | SS-engine-module.md §EngineModule Trait Signature |
| `FactoryAdapter` | Trait in `monocle-core::factory` abstracting over factory-pattern workflow detectors. Open (not sealed). | SS-core-types-and-abi.md §FactoryAdapter Trait |
| `FactoryState` | 7-field canonical struct returned by `FactoryAdapter::read_state()`. Fields: `phase`, `status`, `awaiting`, `blocking_issues`, `convergence`, `cycle`, `custom_fields`. | SS-core-types-and-abi.md §FactoryAdapter Trait |
| FC | Forward-Compatibility item. Pre-Phase-1 contracts locked by human authorization. FC-01 through FC-06. | SS-forward-compatibility.md; product-brief.md §Forward-compatibility contracts |
| `format_version` | First key in every JSONL ring buffer record. Value `1` for all Phase 1 records. | BC-RING-001; SS-daemon-lifecycle.md §Drain |
| `HookEventRecord` | Rust struct in `monocle-runtime::ring` written to the JSONL ring buffer. `#[non_exhaustive]`; provides `new()` constructor. | SS-daemon-lifecycle.md §Drain |
| `HookEnvelope` | Proto message in `monocle-proto` with `schema_version` at field number 1. Wire format for Phase 4 federation. | BC-PROTO-001a, BC-PROTO-001b; SS-core-types-and-abi.md §Prost Wire Schemas |
| JC-2 | Joint Closure 2: `PostToolUse` omitted from Phase 1 hook endpoint set to preserve gene-source parity with any-context-lazyclaude BC-HOOK-007 canonical 5-endpoint matrix. | vision §Closure Log; brief §Scope |
| `monocle-v1:` | Wire-format prefix for Phase 1 auth tokens. `X-Monocle-Authorization: monocle-v1:<64-hex>`. | BC-AUTH-001, BC-AUTH-002 |
| `MONOCLE_ABI_VERSION` | `pub const u32 = 1` in `monocle-core::abi`. Exported at crate root. Used by Phase 3 plugin SDK and Phase 4 federation. | BC-ABI-001, BC-ABI-002 |
| `#[non_exhaustive]` | Rust attribute preventing exhaustive match and struct literal construction outside the defining crate. Default for all `pub` enums in `monocle-core`. | BC-TYPES-001; ADR-0004 |
| OsRng | `rand::rngs::OsRng`. Cryptographically secure random source used for auth token generation. Required; `thread_rng` is forbidden for secrets. | BC-AUTH-001; SS-daemon-lifecycle.md §Start Sequence |
| `Phase1Permission` | Exhaustive enum in `monocle-core::permissions`. Five variants. ADR-0004 exempts it from `#[non_exhaustive]`. | ADR-0004; SS-permissions-phase1.md |
| `schema_version` | Proto field number 1 in `HookEnvelope`. Value `1` for all Phase 1 messages. Used by Phase 4 federation to validate message format compatibility. | BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 |
| `VsddFactoryAdapter` | Phase 1 static implementation of `FactoryAdapter`. Detects VSDD Factory workspaces via `document_type: pipeline-state` in `.factory/STATE.md`. | BC-FACTORY-002 |

---

## §Trace v1.0

**v1.0 (2026-05-14):** Initial PRD authored by product-owner from 16 pre-staged BCs. Source artifacts: SS-daemon-lifecycle.md v1.0.7, SS-core-types-and-abi.md v1.2.8, SS-engine-module.md v1.1.15, product-brief.md v1.4.23, vision v1.1.2, dtu-assessment.md, 4 ADRs. 16 BCs formalized with full preconditions, postconditions, invariants, edge cases, canonical test vectors, and verification specifications. 5 NFRs promoted from brief §Success Criteria; 6 additional NFRs added from cross-cutting concerns. Error taxonomy: 14 error codes covering all error surfaces across 6 subsystem abbreviations. Edge case catalog: 39 entries (EC-001 through EC-039). Glossary: 19 terms. META defense layer compliance: D-047 strict applied; no ambiguous requirements; every BC has ≥1 edge case and ≥1 canonical test vector; no MVP deferrals; no "pending architect review" for answerable questions; all field-order contracts explicitly stated with serde implementation rationale.

**D-042 sweep (v1.0):** 4-pattern sweep applied to this document before commit — SS-*.md v, dtu-assessment.md v, vision v, ADR v citations verified: SS-daemon-lifecycle.md v1.0.7 (confirmed current), SS-core-types-and-abi.md v1.2.8 (confirmed current), SS-engine-module.md v1.1.15 (confirmed current). All version citations in this document are current-pointers, not historical pinpoints.

**PG-4 §-heading-existence sweep (v1.0):** All §-anchor references in this document verified against actual headings in cited architecture files: SS-daemon-lifecycle.md §Drain ✓, §Daemon Lifecycle Protocol ✓, §Lock File Discovery Policy ✓, §Start Sequence ✓, §Health and Status Endpoints ✓. SS-core-types-and-abi.md §ABI Version Constant ✓, §Enum Extensibility ✓, §FactoryAdapter Trait ✓, §Prost Wire Schemas ✓, §Phase 1 PRD BC Pre-Staging ✓. SS-engine-module.md §EngineModule Trait Signature ✓, §Phase 1 Implementation: ClaudeCodeModule ✓, §Struct-level inherent operations ✓, §Behavioral Contracts ✓, §Cross-Crate Constructor Audit ✓. vision §Vision Statement ✓, §End-to-End Killer Scenario ✓, §EngineModule ✓, §FactoryAdapter ✓, §Closure Log ✓, §Explicit Non-Goals ✓. brief §Scope ✓, §Success Criteria ✓, §Forward-compatibility contracts ✓.

**PG-RECIPE-SCOPE compliance (v1.0):** Sweep scope `.factory/specs/` recursive — no citations narrowed to `.factory/specs/architecture/` subdirectory only (the D-042 scope-hole root cause documented in brief v1.4.19).

**PG-5 §Historical-Anchor Framing compliance (v1.0):** All brief version citations in §Trace are contextual historical references (e.g., "brief §Success Criteria" without a pinned version qualifier). Section heading references are stable; version qualifiers omitted per PG-5 option (c) convention.

**F-R60-corpus-sweep (v1.0):** Corpus sweep applied — no known-stale references to: deleted section headings, old enum variant names, superseded type names, or pre-ADR-0004 exhaustive-enum assumptions. `Phase1Permission` and `ClaudeCodeTool` correctly classified as exhaustive-by-ADR throughout.

**18+ META rule checklist (v1.0):**
- D-042 (4-pattern citation sweep): PASS — 3 current-pointer SS-*.md citations swept and confirmed.
- D-047 strict (3-clean-pass convergence): N/A for PRD authoring; applies to adversarial review passes.
- PG-1 (no ambiguous requirements): PASS — every BC has testable preconditions, postconditions, and test vectors.
- PG-2 (no MVP shortcuts): PASS — no "for now", "good enough", "we can fix later" rationalizations.
- PG-3 (no L-number pinpoints in §Trace): PASS — all §Trace references use section heading anchors.
- PG-3-TRACE-NEW-ENTRY (position-free references in new §Trace entries): PASS — this entry uses only section heading anchors.
- PG-4 (§-heading-existence sweep): PASS — all §-anchors verified against actual headings in cited files.
- PG-5 (historical-anchor framing): PASS — version qualifiers on stable section refs omitted.
- PG-RECIPE-SCOPE (`.factory/specs/` recursive sweep): PASS — sweep was not narrowed to architecture/ subdirectory.
- BC-H1-is-title-source-of-truth: N/A — BCs are inline in this PRD (not separate files); H1 is each `### BC-*` heading.
- architecture_is_subsystem_name_source_of_truth: N/A — no `subsystem:` frontmatter on individual BC files (PRD format).
- append_only_numbering: PASS — no BC IDs renumbered; BC-PROTO-001 split preserved as 001a/001b per architect decision.
- lift_invariants_to_bcs: PASS — all invariants from SS-*.md surfaced in corresponding BC invariant sections.
- Capability Anchor Justification (S-7.01): N/A — this is a project-specific PRD, not a standard VSDD L3 PRD with CAP-NNN domain spec; BCs trace to brief §Scope and architecture SS-* sections directly.
- bc_array_changes_propagate_to_body_and_acs: N/A — no stories exist yet; Phase 2 story decomposition pending.
- vp_index_is_vp_catalog_source_of_truth: N/A — no VPs authored yet; formal-verifier runs in parallel per STATE.md T-2.
- Self-audit (CLAUDE.md §Self-Audit Checklist): All 6 items checked — no MVP rationalizations, no tech-debt-register entries, no pending-architect-review markers, no deferred defects, no cheapest-path defaults, no advisories that should be blockers.
