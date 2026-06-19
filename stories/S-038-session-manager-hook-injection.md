---
document_type: story
level: L4
story_id: S-038
epic_id: EPIC-08
version: "1.5"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 3
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-033]
blocks: []
target_module: monocle-runtime
subsystems: [SS-08]
behavioral_contracts: [BC-2.08.006]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.006.md, version: "1.5.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.6.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.08.006 (hook auto-injection in session-host spawn path: --settings arg, hooks-settings.json with 4 URL-bearing + 2 reserved-empty hook entries, lock.app=monocle, shared-file lifecycle)"
# BC status: BC-2.08.006 v1.5.0 — non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-038: SessionManager Hook Auto-Injection — hooks-settings.json Writer + SpawnOptions.hooks_settings_path Population

## Narrative

As the monocle daemon, I want to write the Claude Code hook settings file
(`hooks-settings.json`) once at startup and populate `SpawnOptions.hooks_settings_path`
before every session spawn — so that `ClaudeCodeModule::spawn_recipe()` (S-045) can append
`--settings <path>` to the session-host argv, ensuring every monocle-managed Claude Code
session reports lifecycle events (pre-tool, post-tool, stop, notification, etc.) back to
monocle via the hook protocol without requiring the user to configure hooks manually.

## Acceptance Criteria

### AC-001 (traces to BC-2.08.006 postcondition 1 — --settings arg appended within 2s of spawn_session call)

Within 2 seconds of `spawn_session()` being called, the monocle-session-host child process
is launched with `--settings <path-to-hooks-settings.json>` appended to its argv.
The 2-second window is the same spawn latency budget as BC-2.08.001 PC-1.

### AC-002 (traces to BC-2.08.006 postcondition 2 — hooks-settings.json content: 4 URL-bearing keys)

`hooks-settings.json` MUST contain exactly the following 4 URL-bearing hook keys (authority:
BC-2.04.010 PC-3 / SS-daemon-wiring.md §Hook Tmpfile Generation):
1. `"PreToolUse"` → curl POST `http://127.0.0.1:<daemon_port>/hooks/pre-tool-use` with `X-Monocle-Authorization: monocle-v1:<64-hex>` header
2. `"Notification"` → curl POST `http://127.0.0.1:<daemon_port>/hooks/notification` with `X-Monocle-Authorization: monocle-v1:<64-hex>` header
3. `"Stop"` → curl POST `http://127.0.0.1:<daemon_port>/hooks/stop` with `X-Monocle-Authorization: monocle-v1:<64-hex>` header
4. `"UserPromptSubmit"` → curl POST `http://127.0.0.1:<daemon_port>/hooks/prompt-submit` with `X-Monocle-Authorization: monocle-v1:<64-hex>` header

The daemon's hook endpoint MUST be running before `spawn_session()` is called.

### AC-003 (traces to BC-2.08.006 postcondition 3 — hooks-settings.json content: 2 reserved-empty ARRAY keys)

`hooks-settings.json` MUST contain exactly 2 reserved-empty hook keys, each set to an
**empty array** (NOT an empty string):
1. `"PostToolUse": []` — reserved-empty array (forward-compat placeholder; Claude Code ignores hook types with empty arrays)
2. `"PreCompact": []` — reserved-empty array (forward-compat placeholder)

The value MUST be an empty JSON array `[]`, NOT an empty string `""`. These keys are reserved for
future use per BC-2.04.010 PC-3. Claude Code ignores hook types whose value is an empty array.

### AC-004 (traces to BC-2.08.006 postcondition 3 — lock.app set to "monocle"; SessionStart is NOT a key)

The `hooks-settings.json` MUST include a `lock.app` field set to `"monocle"`. The canonical
schema uses the **array-of-hook-objects** form (authority: BC-2.04.010 PC-3 — BC-2.04.010 is
the single authoritative source for the exact schema; if BC-2.04.010 and this story conflict,
BC-2.04.010 wins per AC-010):

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/pre-tool-use -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<64-hex>' -d @-"
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/notification -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<64-hex>' -d @-"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/stop -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<64-hex>' -d @-"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/prompt-submit -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<64-hex>' -d @-"
          }
        ]
      }
    ],
    "PostToolUse": [],
    "PreCompact": []
  },
  "lock": {
    "app": "monocle"
  }
}
```

Each URL-bearing hook key is an **array** containing one matcher object with a `"hooks"` array
containing one `{"type":"command","command":"<curl>"}` object. `PostToolUse` and `PreCompact`
are empty arrays `[]` (NOT empty strings). `"SessionStart"` is NOT a key in this file.
Claude Code invokes `POST /hooks/session-start` via its own internal lifecycle mechanism
regardless of hooks-settings.json content; monocle's axum router handles it, but it is NOT
configured through hooks-settings.json.
The exact JSON schema (field names, nesting depth, URL format) is defined in BC-2.04.010.

### AC-005 (traces to BC-2.08.006 invariant 3 — shared file; NOT per-session)

`hooks-settings.json` is a SHARED file written once per daemon startup, NOT once per
session spawn. All sessions spawned by the daemon share the same `hooks-settings.json`
at `<runtime_dir>/hooks-settings.json`. The file is NOT re-written on each `spawn_session()`
call unless the daemon's hook endpoint URL changes.

### AC-006 (traces to BC-2.08.006 Invariant 5 — atomic write via tempfile::persist)

The initial write of `hooks-settings.json` (and any update) MUST use `tempfile::persist`
for atomic rename-based write. No partial write is visible to spawned session-hosts.
File mode: readable by owner only (0o600 on Unix).

### AC-007 (traces to BC-2.08.006 invariant 4 — file written at daemon startup before any spawn_recipe() is called)

`hooks-settings.json` MUST exist and be fully written before the FIRST `spawn_session()` call
is dispatched. Lifecycle step 9 calls `write_hooks_settings_json()` before the UDS bind
(step 10), so no client can issue `SpawnSession` before the file exists. If the write fails,
the daemon MUST log `ERROR: failed to write hooks-settings.json: <reason>` and exit with
code 72 (BC-2.08.006 Invariant 5 / BC-2.04.010 PC-6).

### AC-008 (traces to BC-2.08.006 postcondition 2 / architecture ownership boundary — SpawnOptions.hooks_settings_path carried to session-host args; argv injection chain per BC-2.03.005 PC-1)

S-038 owns EXACTLY TWO responsibilities for hook injection:
1. **Provide `write_hooks_settings_json()`** — the single canonical writer of `hooks-settings.json`.
   This function is called by lifecycle step 9 (before UDS bind) with a real `HookEndpointConfig`,
   NOT inside `SessionManager::new()`. `SessionManager::new()` stores the caller-provided
   `HookEndpointConfig` for the EC-182 re-write path only (single-writer mandate, BC-2.08.006 Description).
2. **Populate `SpawnOptions.hooks_settings_path`**: before calling `engine_module.spawn_recipe(&opts)`,
   `SessionManager::spawn_session()` MUST set `opts.hooks_settings_path = self.hooks_settings_path.clone()`
   so that `ClaudeCodeModule::spawn_recipe()` has the path available.
   (`SpawnOptions.hooks_settings_path` is a bare `PathBuf`, not `Option<PathBuf>` — no `Some()` wrapper.)

The argv `["--settings", <path>]` injection into `SpawnRecipe.argv` is the responsibility of
**S-045 (`ClaudeCodeModule::spawn_recipe()`)**, which reads `opts.hooks_settings_path` and appends
the `--settings` arg. S-038 MUST NOT duplicate that injection — S-038 provides the path
via `SpawnOptions`; S-045 consumes it.

**Ownership boundary** (non-duplicable):
- S-038 owns: `write_hooks_settings_json()`, `hooks_settings_path: PathBuf` on `SessionManager`, setting `opts.hooks_settings_path` before calling `spawn_recipe()`.
- S-045 owns: reading `opts.hooks_settings_path` and appending `--settings <path>` to `SpawnRecipe.argv`.

### AC-009 (traces to BC-2.08.006 Invariant 6 — file path is canonicalized; no symlinks)

The path passed in `--settings` MUST be an absolute, canonicalized path (via
`std::fs::canonicalize` or equivalent). Relative paths and symlinks are NOT permitted.
This prevents path resolution issues when the session-host process changes directory.

### AC-010 (traces to BC-2.08.006 postcondition 3 — hook key names and URL format authoritative source: BC-2.04.010 PC-3)

This story does NOT define the hook key names or URL format. BC-2.04.010 is the single
authoritative source for that contract. If BC-2.04.010 and BC-2.08.006 conflict on key
names or URL format, BC-2.04.010 wins. This story implements the injection mechanism
only; the payload definition defers to BC-2.04.010.

### AC-011 (traces to BC-2.08.006 edge case EC-180 — daemon startup write fails)

If the `hooks-settings.json` write fails during daemon initialization:
- Log `tracing::error!("failed to write hooks-settings.json: {err}")`.
- The daemon MUST NOT start (return `Err` from the startup sequence).
- The error surfaces to the user via the daemon process exit code.

### AC-012 (traces to BC-2.08.006 invariant 4 / edge case EC-181 — hooks-settings.json guaranteed to exist at spawn time; EC-181 is UNREACHABLE)

EC-181 (hooks-settings.json missing `lock.app` filter) is an invariant violation at the
daemon's hook-file writer, not a spawn-time error. The file is written at daemon startup
BEFORE the IPC socket is bound (Invariant 4 of BC-2.08.006); no `SpawnSession` IPC can
arrive before the file exists — the IPC bind step follows hook-file write in the daemon
startup sequence. Therefore `hooks_settings_path` always points to an existing, valid file
when `spawn_recipe()` runs. There is NO `ConfigError` SessionError variant and NO
`"config_error"` wire code for this path.

If the hook endpoint URL is structurally invalid (non-UTF-8 path) when `spawn_recipe()`
is called, it surfaces as `EngineError::InvalidPath` → wire code `"invalid_spawn_arg"`
(canonical error code from the 12-code taxonomy — no new code needed).

### AC-013 (traces to BC-2.08.006 edge case EC-182 — hooks-settings.json deleted between daemon startup and spawn)

If `hooks-settings.json` is deleted by an external process between daemon startup and
a `spawn_session()` call, `SessionManager::spawn_session()` MUST re-write the file using
the cached in-memory hook endpoint URL before populating `opts.hooks_settings_path` and
proceeding with the spawn. `spawn_recipe()` performs NO file I/O (BC-2.03.005 PC-3 / AC-002 of S-045).
The re-write guard lives in `spawn_session()`, consistent with Tasks and AC-008.
`tracing::warn!("hooks-settings.json missing at spawn time; re-writing")`.

## Tasks

- [ ] Define `HookEndpointConfig` struct in `monocle-runtime/src/session_manager/mod.rs` (or a sub-module):
  ```rust
  /// Holds the 4 URL-bearing hook endpoint strings.
  /// PostToolUse and PreCompact are ALWAYS written as reserved-empty arrays [];
  /// they are NOT represented as String fields (BC-2.08.006 PC-3 / BC-2.04.010 PC-3).
  struct HookEndpointConfig {
      pre_tool_use: String,
      notification: String,
      stop: String,
      user_prompt_submit: String,
  }
  ```
- [ ] Implement `write_hooks_settings_json(config: &HookEndpointConfig, path: &Path) -> Result<(), SessionError>` using `tempfile::NamedTempFile`, `serde_json`, and `tempfile::persist`. Set file mode 0o600 via `std::os::unix::fs::PermissionsExt`.
  The JSON structure MUST be: `{ "hooks": { "PreToolUse": <url>, "Notification": <url>, "Stop": <url>, "UserPromptSubmit": <url>, "PostToolUse": [], "PreCompact": [] }, "lock": { "app": "monocle" } }`.
  Write `PostToolUse` and `PreCompact` as JSON arrays `[]` (not strings). `serde_json::Value::Array(vec![])` is the correct Rust representation.
- [ ] `write_hooks_settings_json()` is called by **lifecycle step 9** (before UDS bind), NOT inside `SessionManager::new()`. `SessionManager::new()` receives the caller-provided `HookEndpointConfig` (populated with real port + auth_token by lifecycle) and stores it in `self.hook_endpoint_config` for the EC-182 re-write path. No write in `new()` (single-writer mandate, BC-2.08.006 §Single-writer mandate).
- [ ] Add `hooks_settings_path: PathBuf` field to `SessionManager`.
- [ ] In `SessionManager::spawn_session()`, before calling `engine_module.spawn_recipe(&opts)`: set `opts.hooks_settings_path = self.hooks_settings_path.clone()` so `ClaudeCodeModule::spawn_recipe()` (S-045) can append `--settings <path>` to argv. (`SpawnOptions.hooks_settings_path` is a bare `PathBuf` — no `Some()` wrapper.) S-038 MUST NOT duplicate the argv append — that is S-045's responsibility. Only populate the field.
- [ ] EC-182 re-write guard (in `spawn_session()`, before populating `opts.hooks_settings_path`): if `self.hooks_settings_path` does not exist (`!self.hooks_settings_path.exists()`), call `write_hooks_settings_json()` to re-write it, log `tracing::warn!("hooks-settings.json missing at spawn time; re-writing")`, then proceed.
- [ ] Write unit test `test_BC_2_08_006_spawn_options_hooks_settings_path_populated`: call `spawn_session()` on a `SessionManager` with a valid `hooks_settings_path`; intercept the `SpawnOptions` passed to `spawn_recipe()` (via mock `EngineModule`); assert `opts.hooks_settings_path == self.hooks_settings_path` (bare `PathBuf` — no `Some()` wrapper). Do NOT assert `argv` here — that is S-045's test (`test_BC_2_03_005_spawn_recipe_happy_path_binary_args_env_cwd`, which asserts `recipe.args == ["--settings", path]` per BC-2.03.005 PC-1).
- [ ] Write unit test `test_BC_2_08_006_hooks_settings_json_content`: write hooks-settings.json to tmp dir; read back; assert 4 URL-bearing keys present; assert 2 empty reserved keys present; assert `lock.app == "monocle"`.
- [ ] Write unit test `test_BC_2_08_006_hooks_settings_json_atomic_write`: verify write uses `tempfile::persist` (test via file content atomicity under mock FS or by verifying the temp file never has partial content).
- [ ] Write unit test `test_BC_2_08_006_startup_write_fail_aborts_daemon`: if `write_hooks_settings_json` returns Err, `SessionManager::new()` propagates the error; daemon start fails.
- [ ] Write unit test `test_BC_2_08_006_missing_settings_file_rewrites_at_spawn`: delete hooks-settings.json after `SessionManager::new()`; call `spawn_session()`; assert file exists again; assert WARN logged; assert spawn succeeds.
- [ ] Write unit test `test_BC_2_08_006_non_utf8_hooks_path_returned_from_spawn_recipe`: `SessionManager` constructed with a non-UTF-8 `hooks_settings_path`; `spawn_session()` calls `spawn_recipe()`; `ClaudeCodeModule::spawn_recipe()` (S-045) encounters a non-UTF-8 path in `opts.hooks_settings_path` and returns `EngineError::InvalidPath`; IPC handler maps to wire code `"invalid_spawn_arg"`. This test verifies the end-to-end path, not S-038's internal plumbing.

## Previous Story Intelligence

- **S-033** (spawn): `spawn_recipe()` is introduced in S-033. S-038 does NOT modify `spawn_recipe()`. S-038 writes `hooks-settings.json` at daemon startup and populates `opts.hooks_settings_path` before the `spawn_recipe()` call in `spawn_session()`. The `--settings` argv injection into `SpawnRecipe.args` is owned by S-045 (`ClaudeCodeModule::spawn_recipe()`). Coordinate: S-038 and S-045 must be reviewed together to ensure the field-population (S-038) and the arg-injection (S-045) are complementary with no duplication.
- The `HookEndpointConfig` values (URLs) come from the hook server started during daemon initialization (Phase 1 BC-2.04.NNN). S-038 does NOT spawn the hook server — it consumes the already-running server's URL. The exact URL format is deferred to BC-2.04.010; this story provides a `String`-typed field that the daemon's startup sequence fills in.
- `tempfile::persist` pattern is already established in the sidecar write path (S-033). Re-use the same pattern here.
- File permissions: `std::os::unix::fs::PermissionsExt::set_mode(0o600)` after persist.

## Architecture Compliance Rules

- `hooks-settings.json` written once at daemon startup by lifecycle step 9 (which calls `write_hooks_settings_json()`), NOT inside `SessionManager::new()`. Shared across all sessions. `SessionManager::new()` stores the config for EC-182 re-writes only (single-writer mandate per BC-2.08.006 Description).
- `ClaudeCodeModule::spawn_recipe()` (S-045) appends `--settings <path>` to `SpawnRecipe.args`. S-038 provides the path by setting `opts.hooks_settings_path` before calling `spawn_recipe()`; S-038 MUST NOT append `--settings` itself. The path is canonicalized at `SessionManager::new()` initialization (stored in `hooks_settings_path: PathBuf`).
- `write_hooks_settings_json()` MUST use `tempfile::persist` (SS-conventions-anti-patterns.md §Atomic writes policy: all config files via `tempfile::persist`; no exceptions).
- File permissions: 0o600. Session-host child processes run as the same user, so owner-read-only is sufficient.
- JSON schema authority for hook key names and URL format: BC-2.04.010. This story MUST defer any ambiguous key naming to BC-2.04.010.
- Forbidden dependency: `monocle-runtime` MUST NOT depend on `monocle-tui`.
- There is NO `ConfigError` SessionError variant and NO `"config_error"` wire code in this story. The canonical 12-code taxonomy (defined in SS-ipc.md §ServerToClient::Error, closed set for Phase 1) does not include `"config_error"`. EC-181 is UNREACHABLE by invariant (hooks-settings.json is written before IPC bind). Non-UTF-8 path errors are `EngineError::InvalidPath` → `"invalid_spawn_arg"` (existing code). Do NOT add any new `SessionError` variant or wire code in this story.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tempfile` | `"3"` | Atomic write of hooks-settings.json via `NamedTempFile` + `persist()` | SS-deps-pin-manifest.md |
| `serde_json` | `=1.0.149` (exact) | Serialize hook config struct to JSON | SS-deps-pin-manifest.md |
| `serde` | `"1"` (derive) | `Serialize` derive for hook config struct | SS-deps-pin-manifest.md |
| `thiserror` | `"2"` | `SessionError` enum (existing variants only; no new variant in this story) | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/session_manager/mod.rs` | Add `HookEndpointConfig`, `write_hooks_settings_json()`, `hooks_settings_path` field on `SessionManager`, populate `opts.hooks_settings_path` before `spawn_recipe()` call in `spawn_session()`, add EC-182 re-write guard in `spawn_session()` |
| `crates/monocle-runtime/src/session_manager/mod.rs` | No new `SessionError` variant added. Verify `session_error_to_code()` exhaustive outer match still compiles after any S-033-introduced additions. (`SessionError` canonical location is `session_manager/mod.rs` per S-033.) |

Files to VERIFY (no modification expected):

| File | What to verify |
|------|---------------|
| `crates/monocle-runtime/Cargo.toml` | `tempfile = "3"` and `serde_json = { version = "=1.0.149", features = ["..."] }` are declared |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~2,500 |
| BC-2.08.006 | ~2,000 |
| SS-session-manager.md (SpawnRecipe, hook injection, spawn path) | ~3,000 |
| Existing session_manager code from S-033 | ~4,000 |
| Test files | ~2,500 |
| **Total estimate** | **~14,000** |

Estimate is comfortably within the 30% context window bound. No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.08.006 | Hook Auto-Injection — `--settings` Arg Present in Session-Host Child Args Within 2s of Spawn | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `write_hooks_settings_json()` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (filesystem write via tempfile::persist) |
| `HookEndpointConfig` struct | `monocle-runtime/src/session_manager/mod.rs` | Pure (data struct) |
| `spawn_session()` — `opts.hooks_settings_path` population | `monocle-runtime/src/session_manager/mod.rs` | Pure step inside effectful spawn (field assignment; no I/O) |
| EC-182 re-write guard in `spawn_session()` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (conditional filesystem write via `write_hooks_settings_json()`) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-180 | Daemon startup write fails | Log error; daemon MUST NOT start; error propagated |
| EC-181 | hooks-settings.json missing `lock.app` filter (regression) | INVARIANT VIOLATION at the daemon hook-file writer — not a spawn-time error. The file is written before IPC bind; no `SpawnSession` can arrive before the file exists. EC-181 is UNREACHABLE from the spawn path. The daemon's hook-file writer MUST include `lock.app = "monocle"` as an invariant (enforced by unit test `test_BC_2_08_006_hooks_settings_json_content`). |
| EC-182 | hooks-settings.json deleted between daemon startup and spawn | Re-write at spawn time; WARN logged; spawn proceeds |
| EC-183 | hooks-settings.json path contains non-UTF-8 bytes | `PathBuf::to_str()` returns `None`; `spawn_recipe()` returns `EngineError::InvalidPath` → IPC handler maps to wire code `"invalid_spawn_arg"`. No new variant needed. |

## Subsystem Anchor Justifications

**SS-08 owns this story's scope** because hook auto-injection is defined in
SS-session-manager.md §SpawnRecipe integration with EngineModule (the authoritative section
for the spawn path and `SpawnOptions` field population). S-038's responsibilities — writing
`hooks-settings.json` at daemon startup and populating `opts.hooks_settings_path` before calling
`spawn_recipe()` — are daemon-side SessionManager concerns. BC-2.04.010 defines the hook payload
schema; BC-2.08.006 defines the daemon injection mechanism. The argv `["--settings", path]`
insertion into `SpawnRecipe.argv` is owned by S-045 (ClaudeCodeModule::spawn_recipe()), not S-038.

**Dependency Anchors:**
- STORY-038 depends on S-033 because `spawn_recipe()`, `SpawnRecipe`, `SessionHostSpawner`,
  and the spawn path must exist before the `--settings` extension can be appended.
  S-038 DOES NOT re-implement the spawn mechanism — it extends S-033's `spawn_recipe()`.

## Conflicts and Notes

- BC-2.08.006 v1.5.0 references "BC-2.04.010 (not BC-HOOK-007)" as the authority for hook
  key names and URL format. The implementer MUST read BC-2.04.010 before finalizing the
  exact hook JSON schema. This story provides the structural wrapper; the payload is defined
  by BC-2.04.010.
- If BC-2.04.010 defines a different set of hook keys (not exactly 4 URL-bearing + 2
  reserved-empty), BC-2.04.010 wins and this story's AC-002/AC-003 are superseded by
  BC-2.04.010. Surface this to the orchestrator if BC-2.04.010 is not available before
  implementation begins.
- There is NO `ConfigError` SessionError variant and NO `"config_error"` wire code in this
  story. The 12-code taxonomy (closed set for Phase 1) does not include `"config_error"`.
  Non-UTF-8 path errors route to `EngineError::InvalidPath` → `"invalid_spawn_arg"`.
  EC-181 (hooks-settings.json missing) is UNREACHABLE by daemon startup invariant.

## Trace

| Version | Change | Pass |
|---------|--------|------|
| v1.5 | AC-004 schema corrected: bare-string hook values replaced with array-of-hook-objects form per BC-2.04.010 PC-3 (F-S038-AC004-SCHEMA). AC-008 + Tasks Option→PathBuf: `opts.hooks_settings_path = Some(...)` → `opts.hooks_settings_path = ...` (bare PathBuf per monocle-core/src/engine.rs; F-S038-PASS1-007). Single-writer reconciliation: AC-008 responsibility 1, Tasks write-call bullet, AC-007 error level/code, and Architecture Compliance Rules updated to reflect BC-2.08.006 v1.5.0 single-writer mandate (lifecycle step 9 owns startup write; `SessionManager::new()` stores config only). BC-2.08.006 input pin bumped 1.4.0→1.5.0. | adversarial-pass |
| v1.4 | BC-2.08.006 input pin bumped 1.4.0→1.4.1 (arch-source pin cascade for SS-session-manager at v1.4 authoring time; F-S035-PASS5-MED-001). | post-convergence |
| v1.3 | BC-2.08.006 input pin bumped 1.3.2→1.4.0 (PO added Invariants 5+6 + EC-183/184). AC-006 trace citation updated "invariant 4 / CLAUDE.md atomic-write convention — no dedicated BC clause"→"Invariant 5" (new dedicated clause). AC-009 trace citation updated "postcondition 1 / SS-conventions path handling — no dedicated BC clause"→"Invariant 6" (new dedicated clause). Body BC-table title corrected to canonical BC H1 form (F-P24-SUG-001). | post-convergence |
| v1.2 | Corpus-wide AC-trace-citation audit (F-P20-CRIT-001 class): AC-005 "postcondition 5"→"invariant 3" (shared file); AC-006 "postcondition 6"→"invariant 4 / CLAUDE.md" (atomic write; no dedicated BC clause); AC-007 "postcondition 7"→"invariant 4" (write at startup); AC-008 "postcondition 8"→"postcondition 2 / architecture boundary"; AC-009 "invariant 1"→"postcondition 1 / SS-conventions" (canonicalization; no dedicated BC clause); AC-010 "invariant 2"→"postcondition 3" (BC-2.04.010 authority). AC bodies unchanged. Genuine BC gaps: AC-006 atomic write and AC-009 path canonicalization have no dedicated clause in BC-2.08.006 — both trace to project conventions. | Phase-2 |
| v1.0 | Initial decomposition | Phase-2 |
| v1.1 | F-P11-SUG-002 cross-ref correction: Tasks test name corrected to `test_BC_2_03_005_spawn_recipe_happy_path_binary_args_env_cwd` (dangling reference fixed) | Pass-11 |
