---
document_type: story
level: L4
story_id: S-038
epic_id: EPIC-08
version: "1.0"
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
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.006.md, version: "1.3.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.6.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.0"}
input-hash: "[pending]"
traces_to: "Implements BC-2.08.006 (hook auto-injection in session-host spawn path: --settings arg, hooks-settings.json with 4 URL-bearing + 2 reserved-empty hook entries, lock.app=monocle, shared-file lifecycle)"
# BC status: BC-2.08.006 v1.3.1 — non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-038: SessionManager Hook Auto-Injection — --settings Arg in Session-Host Spawn Path

## Narrative

As the monocle daemon, I want to automatically inject the Claude Code hook settings file
(`hooks-settings.json`) into the session-host's child argv when spawning a new session —
so that every monocle-managed Claude Code session reports lifecycle events (pre-tool, post-tool,
stop, notification, etc.) back to monocle via the hook protocol — without requiring the user to
configure hooks manually in their Claude Code user settings.

## Acceptance Criteria

### AC-001 (traces to BC-2.08.006 postcondition 1 — --settings arg appended within 2s of spawn_session call)

Within 2 seconds of `spawn_session()` being called, the monocle-session-host child process
is launched with `--settings <path-to-hooks-settings.json>` appended to its argv.
The 2-second window is the same spawn latency budget as BC-2.08.001 PC-1.

### AC-002 (traces to BC-2.08.006 postcondition 2 — hooks-settings.json content: 4 URL-bearing keys)

`hooks-settings.json` MUST contain exactly the following 4 URL-bearing hook keys, each set
to the monocle daemon's local IPC endpoint URL:
1. `PreToolUse` → `"http://localhost:<hook_port>/hooks/pre-tool-use"` (or equivalent UDS hook URL)
2. `PostToolUse` → `"http://localhost:<hook_port>/hooks/post-tool-use"`
3. `Stop` → `"http://localhost:<hook_port>/hooks/stop"`
4. `Notification` → `"http://localhost:<hook_port>/hooks/notification"`

The exact key names, URL format, and port are defined in BC-2.04.010 (the hook protocol
authority). The daemon's hook endpoint MUST be running before `spawn_session()` is called.

### AC-003 (traces to BC-2.08.006 postcondition 3 — hooks-settings.json content: 2 reserved-empty keys)

`hooks-settings.json` MUST contain exactly 2 reserved-empty hook keys (value: empty string `""`):
1. `PreCompact` → `""`
2. `PostCompact` → `""`

These keys are reserved for future use per BC-2.08.006 PC-3. Their presence ensures the
settings file schema is stable across Claude Code versions that may check for key existence.

### AC-004 (traces to BC-2.08.006 postcondition 4 — lock.app set to "monocle")

The `hooks-settings.json` MUST include a `lock.app` field set to `"monocle"`:

```json
{
  "lock": {
    "app": "monocle"
  },
  "hooks": {
    "PreToolUse": "...",
    "PostToolUse": "...",
    "Stop": "...",
    "Notification": "...",
    "PreCompact": "",
    "PostCompact": ""
  }
}
```

The exact JSON schema (field names, nesting depth) is defined in BC-2.04.010.

### AC-005 (traces to BC-2.08.006 postcondition 5 — shared file; NOT per-session)

`hooks-settings.json` is a SHARED file written once per daemon startup, NOT once per
session spawn. All sessions spawned by the daemon share the same `hooks-settings.json`
at `<runtime_dir>/hooks-settings.json`. The file is NOT re-written on each `spawn_session()`
call unless the daemon's hook endpoint URL changes.

### AC-006 (traces to BC-2.08.006 postcondition 6 — atomic write via tempfile::persist)

The initial write of `hooks-settings.json` (and any update) MUST use `tempfile::persist`
for atomic rename-based write. No partial write is visible to spawned session-hosts.
File mode: readable by owner only (0o600 on Unix).

### AC-007 (traces to BC-2.08.006 postcondition 7 — file written before first spawn_session call)

`hooks-settings.json` MUST exist and be fully written before the FIRST `spawn_session()` call
is dispatched. The daemon startup sequence MUST write the file during initialization
(before the IPC listen socket is bound, so no client can issue `SpawnSession` before the file
exists). If the write fails, the daemon MUST log a WARN and abort the startup sequence.

### AC-008 (traces to BC-2.08.006 postcondition 8 — SpawnRecipe includes --settings arg)

`spawn_recipe()` — the internal method that converts `SpawnOptions` into a `SpawnRecipe` —
MUST append `["--settings", "<path-to-hooks-settings.json>"]` to the `argv` vector it
builds for the session-host child process. The `hooks_settings_path` field is a property
of `SessionManager` (set during initialization; remains constant for the daemon's lifetime).

### AC-009 (traces to BC-2.08.006 invariant 1 — file path is canonicalized; no symlinks)

The path passed in `--settings` MUST be an absolute, canonicalized path (via
`std::fs::canonicalize` or equivalent). Relative paths and symlinks are NOT permitted.
This prevents path resolution issues when the session-host process changes directory.

### AC-010 (traces to BC-2.08.006 invariant 2 — authority for hook key names and URL format is BC-2.04.010)

This story does NOT define the hook key names or URL format. BC-2.04.010 is the single
authoritative source for that contract. If BC-2.04.010 and BC-2.08.006 conflict on key
names or URL format, BC-2.04.010 wins. This story implements the injection mechanism
only; the payload definition defers to BC-2.04.010.

### AC-011 (traces to BC-2.08.006 edge case EC-180 — daemon startup write fails)

If the `hooks-settings.json` write fails during daemon initialization:
- Log `tracing::error!("failed to write hooks-settings.json: {err}")`.
- The daemon MUST NOT start (return `Err` from the startup sequence).
- The error surfaces to the user via the daemon process exit code.

### AC-012 (traces to BC-2.08.006 edge case EC-181 — hooks endpoint not yet available at spawn time)

The hook endpoint is always initialized before IPC bind (per AC-007), so this race
cannot occur in normal operation. However, if the hook endpoint URL is absent or
unresolvable when `spawn_recipe()` is called, `spawn_session()` returns
`Err(SessionError::ConfigError { reason: "hook endpoint unavailable" })` → wire code
`"config_error"`. Test: simulate a `SessionManager` with no hook endpoint configured
and assert `spawn_session()` returns `Err(ConfigError)`.

### AC-013 (traces to BC-2.08.006 edge case EC-182 — hooks-settings.json deleted between daemon startup and spawn)

If `hooks-settings.json` is deleted by an external process between daemon startup and
a `spawn_session()` call, `spawn_recipe()` MUST re-write the file using the cached
in-memory hook endpoint URL before proceeding with the spawn. This ensures the
`--settings` arg always points to a valid file.
`tracing::warn!("hooks-settings.json missing at spawn time; re-writing")`.

## Tasks

- [ ] Define `HookEndpointConfig` struct in `monocle-runtime/src/session_manager/mod.rs` (or a sub-module):
  ```rust
  struct HookEndpointConfig {
      pre_tool_use: String,
      post_tool_use: String,
      stop: String,
      notification: String,
  }
  ```
- [ ] Implement `write_hooks_settings_json(config: &HookEndpointConfig, path: &Path) -> Result<(), SessionError>` using `tempfile::NamedTempFile`, `serde_json`, and `tempfile::persist`. Set file mode 0o600 via `std::os::unix::fs::PermissionsExt`.
- [ ] Call `write_hooks_settings_json()` during `SessionManager::new()` initialization (before IPC bind). Propagate error to caller.
- [ ] Add `hooks_settings_path: PathBuf` field to `SessionManager`.
- [ ] In `spawn_recipe()`: append `["--settings", hooks_settings_path.to_str().unwrap_or_default()]` to `SpawnRecipe.argv`. Guard: if `hooks_settings_path` is empty, return `Err(SessionError::ConfigError { reason: "hook endpoint unavailable".to_string() })`.
- [ ] In `spawn_recipe()`: if `hooks_settings_path` does not exist (`!path.exists()`), call `write_hooks_settings_json()` to re-write it (EC-182 re-write guard).
- [ ] Write unit test `test_BC_2_08_006_spawn_includes_settings_arg`: mock `SessionHostSpawner` captures argv; `spawn_session()` called; assert argv contains `"--settings"` followed by the absolute hooks-settings path.
- [ ] Write unit test `test_BC_2_08_006_hooks_settings_json_content`: write hooks-settings.json to tmp dir; read back; assert 4 URL-bearing keys present; assert 2 empty reserved keys present; assert `lock.app == "monocle"`.
- [ ] Write unit test `test_BC_2_08_006_hooks_settings_json_atomic_write`: verify write uses `tempfile::persist` (test via file content atomicity under mock FS or by verifying the temp file never has partial content).
- [ ] Write unit test `test_BC_2_08_006_startup_write_fail_aborts_daemon`: if `write_hooks_settings_json` returns Err, `SessionManager::new()` propagates the error; daemon start fails.
- [ ] Write unit test `test_BC_2_08_006_missing_settings_file_rewrites_at_spawn`: delete hooks-settings.json after `SessionManager::new()`; call `spawn_session()`; assert file exists again; assert WARN logged; assert spawn succeeds.
- [ ] Write unit test `test_BC_2_08_006_no_hook_endpoint_returns_config_error`: `SessionManager` constructed with no hook endpoint URL; `spawn_session()` → `Err(ConfigError { reason: "hook endpoint unavailable" })`.

## Previous Story Intelligence

- **S-033** (spawn): `spawn_recipe()` is introduced in S-033. S-038 extends `spawn_recipe()` to append the `--settings` arg. Coordinate: S-038 MUST be reviewed alongside S-033 to ensure the `SpawnRecipe.argv` extension is clean (not duplicated logic).
- The `HookEndpointConfig` values (URLs) come from the hook server started during daemon initialization (Phase 1 BC-2.04.NNN). S-038 does NOT spawn the hook server — it consumes the already-running server's URL. The exact URL format is deferred to BC-2.04.010; this story provides a `String`-typed field that the daemon's startup sequence fills in.
- `tempfile::persist` pattern is already established in the sidecar write path (S-033). Re-use the same pattern here.
- File permissions: `std::os::unix::fs::PermissionsExt::set_mode(0o600)` after persist.

## Architecture Compliance Rules

- `hooks-settings.json` written once at `SessionManager::new()`, not per spawn. Shared across all sessions.
- `spawn_recipe()` appends `--settings <path>` to argv. The path is canonicalized at initialization time (stored in `hooks_settings_path: PathBuf`).
- `write_hooks_settings_json()` MUST use `tempfile::persist` (SS-conventions-anti-patterns.md §Atomic writes policy: all config files via `tempfile::persist`; no exceptions).
- File permissions: 0o600. Session-host child processes run as the same user, so owner-read-only is sufficient.
- JSON schema authority for hook key names and URL format: BC-2.04.010. This story MUST defer any ambiguous key naming to BC-2.04.010.
- Forbidden dependency: `monocle-runtime` MUST NOT depend on `monocle-tui`.
- The `ConfigError` SessionError variant may be new (not in the existing 9-variant taxonomy from S-033). If the S-033 taxonomy does not include `ConfigError`, this story must add it and update `session_error_to_code()` to map `ConfigError` → `"config_error"`. Do NOT reuse an existing error code with wrong semantics.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tempfile` | `"3"` | Atomic write of hooks-settings.json via `NamedTempFile` + `persist()` | SS-deps-pin-manifest.md |
| `serde_json` | `=1.0.149` (exact) | Serialize hook config struct to JSON | SS-deps-pin-manifest.md |
| `serde` | `"1"` (derive) | `Serialize` derive for hook config struct | SS-deps-pin-manifest.md |
| `thiserror` | `"2"` | `SessionError::ConfigError` variant (may be new) | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/session_manager/mod.rs` | Add `HookEndpointConfig`, `write_hooks_settings_json()`, `hooks_settings_path` field on `SessionManager`, extend `spawn_recipe()` to append `--settings` arg, add EC-182 re-write guard |
| `crates/monocle-runtime/src/error.rs` (or wherever `SessionError` is defined) | Add `ConfigError { reason: String }` variant if not already present; update `session_error_to_code()` to map `ConfigError` → `"config_error"` |

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
| BC-2.08.006 | Hook Auto-Injection in Session-Host Spawn Path | v1.3.0 |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `write_hooks_settings_json()` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (filesystem write via tempfile::persist) |
| `HookEndpointConfig` struct | `monocle-runtime/src/session_manager/mod.rs` | Pure (data struct) |
| `spawn_recipe()` extension | `monocle-runtime/src/session_manager/mod.rs` | Pure extension to existing function (argv append; no I/O in happy path) |
| `SessionError::ConfigError` | `monocle-runtime/src/error.rs` | Pure (enum variant) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-180 | Daemon startup write fails | Log error; daemon MUST NOT start; error propagated |
| EC-181 | Hook endpoint not yet available at spawn time | `spawn_session()` → `Err(ConfigError { reason: "hook endpoint unavailable" })` → wire `"config_error"` |
| EC-182 | hooks-settings.json deleted between daemon startup and spawn | Re-write at spawn time; WARN logged; spawn proceeds |
| EC-183 | hooks-settings.json path contains non-UTF-8 bytes | `PathBuf::to_str()` returns None; log error; return `Err(ConfigError { reason: "hooks settings path is not valid UTF-8" })` |

## Subsystem Anchor Justifications

**SS-08 owns this story's scope** because hook auto-injection is defined in
SS-session-manager.md §Hook settings injection and is part of `spawn_recipe()` which lives
entirely in `monocle-runtime/src/session_manager/`. BC-2.04.010 defines the hook payload
schema but the injection mechanism (writing the file and appending --settings) is SS-08's
responsibility.

**Dependency Anchors:**
- STORY-038 depends on S-033 because `spawn_recipe()`, `SpawnRecipe`, `SessionHostSpawner`,
  and the spawn path must exist before the `--settings` extension can be appended.
  S-038 DOES NOT re-implement the spawn mechanism — it extends S-033's `spawn_recipe()`.

## Conflicts and Notes

- BC-2.08.006 v1.3.1 references "BC-2.04.010 (not BC-HOOK-007)" as the authority for hook
  key names and URL format. The implementer MUST read BC-2.04.010 before finalizing the
  exact hook JSON schema. This story provides the structural wrapper; the payload is defined
  by BC-2.04.010.
- If BC-2.04.010 defines a different set of hook keys (not exactly 4 URL-bearing + 2
  reserved-empty), BC-2.04.010 wins and this story's AC-002/AC-003 are superseded by
  BC-2.04.010. Surface this to the orchestrator if BC-2.04.010 is not available before
  implementation begins.
- The `ConfigError` SessionError variant: if the S-033 9-variant taxonomy already includes
  a variant that means "configuration is invalid or missing" with a `reason` field, use
  that existing variant rather than adding a new one. Do NOT reuse a wrong-semantics code.
