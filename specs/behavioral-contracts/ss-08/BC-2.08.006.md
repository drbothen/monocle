---
document_type: behavioral-contract
level: L3
version: "1.5.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/SS-engine-module-v2-delta.md]
input-hash: "80e57df"
traces_to: prd.md
origin: greenfield
subsystem: SS-08
capability: CAP-008
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.08.006: Hook Auto-Injection — `--settings` Arg Present in Session-Host Child Args Within 2s of Spawn

## Description

Every session spawned by monocle has the hooks-settings.json path injected as
`--settings <hooks_settings_path>` in the harness child process's CLI args. This injection
is automatic — no user configuration required. The `lock.app = 'monocle'` filter in the
hook JSON ensures only monocle-launched sessions trigger monocle's hook endpoints. This BC
verifies the end-to-end injection chain: daemon writes hooks-settings.json → `SpawnOptions`
carries the path → `ClaudeCodeModule::spawn_recipe()` produces `--settings <path>` in args
→ session-host builds the harness `CommandBuilder` with those args → harness child process
has `--settings` in its argv.

**Single-writer mandate (F-S038-PASS1-001):** `session_manager::write_hooks_settings_json`
is the SOLE canonical function that serializes `hooks-settings.json`. It is called from two
sites, both passing a fully-populated `HookEndpointConfig`:
1. **lifecycle step 9** (`lifecycle::write_hooks_settings` is replaced by a call to
   `session_manager::write_hooks_settings_json` with a real `HookEndpointConfig` constructed
   from the daemon's `port` and `auth_token`). This write happens before UDS bind; abort on
   failure (exit code 72).
2. **EC-182 re-write** inside `SessionManager::spawn_session()` if the file is absent at
   spawn time: calls `write_hooks_settings_json` with `self.hook_endpoint_config`, which holds
   the real config passed to `SessionManager::new()`.

`lifecycle::write_hooks_settings` (the pre-S-038 function using `HooksSettings`/`HooksMap`
structs) is removed. `HooksSettings`, `HooksMap`, and their `lock`-field-absent serialization
path are replaced by the `serde_json::json!` schema in `write_hooks_settings_json`.
`HookEndpointConfig::default()` (empty-string URLs) is used ONLY in tests that do not set up
a real daemon port; production construction always provides a real config.

## Preconditions

1. The daemon has called `SessionManager::spawn_session()` with a valid recipe.
2. `recipe.args` contains `["--settings", "<hooks_settings_path>"]` as produced by
   `ClaudeCodeModule::spawn_recipe()` (BC-2.03.005 PC-1 postcondition).
3. The hooks-settings.json file at `hooks_settings_path` exists and is readable.
4. The `monocle-session-host` process has been started.

## Postconditions

1. Within 2 seconds of `spawn_session()` being called, the `monocle-session-host` has
   built the harness `CommandBuilder` from `recipe.binary`, `recipe.args`, `recipe.env`,
   `recipe.cwd` (per SS-session-manager.md §startup sequence step 4).
2. The harness child process (`claude`) is spawned on the PTY slave with `--settings
   <hooks_settings_path>` present in its command-line args. The args are passed verbatim
   from `recipe.args` — no modification by the session-host.
3. The hooks-settings.json at `hooks_settings_path` contains (per BC-2.04.010 PC-3 /
   SS-daemon-wiring.md §Hook Tmpfile Generation):
   - `"hooks": { ... }` with exactly 4 URL-bearing keys and 2 reserved-empty keys:
     - `"PreToolUse"` → curl POST `http://127.0.0.1:<daemon_port>/hooks/pre-tool-use`
       with `X-Monocle-Authorization: monocle-v1:<64-hex>` header
     - `"Notification"` → curl POST `http://127.0.0.1:<daemon_port>/hooks/notification`
       with `X-Monocle-Authorization: monocle-v1:<64-hex>` header
     - `"Stop"` → curl POST `http://127.0.0.1:<daemon_port>/hooks/stop`
       with `X-Monocle-Authorization: monocle-v1:<64-hex>` header
     - `"UserPromptSubmit"` → curl POST `http://127.0.0.1:<daemon_port>/hooks/prompt-submit`
       with `X-Monocle-Authorization: monocle-v1:<64-hex>` header
     - `"PostToolUse": []` — reserved-empty array (forward-compat placeholder; Claude Code
       ignores hook types with empty arrays)
     - `"PreCompact": []` — reserved-empty array (forward-compat placeholder)
   - `"SessionStart"` is NOT a key in this file. Claude Code invokes
     `POST /hooks/session-start` via its own internal lifecycle mechanism regardless of
     hooks-settings.json content; monocle's axum router handles it, but it is NOT
     configured through hooks-settings.json.
   - `"lock": { "app": "monocle" }` — the filter that prevents externally-launched
     `claude` instances from sending hooks to monocle's daemon.
   Authority: BC-2.04.010 PC-3 (4-URL + 2-empty schema). Note: BC-HOOK-007 governs the
   5-key set produced by the DTU gene-source clone's `WriteHooksSettingsFile` (which
   includes `"SessionStart"` and excludes `"PreCompact"`); BC-HOOK-007 does NOT govern
   the file written by the v1A daemon.
4. No manual user configuration of hooks is required. The user MUST NOT edit
   `~/.monocle/settings.json` or any Claude Code settings file to receive hooks
   (BC-HOOK-027 + BC-HOOK-028).
5. A monocle-launched session (via SessionManager) is distinguishable from an
   externally-launched session because the daemon sets `spawned_by_monocle: Some(true)`
   on the `SessionSnapshot` wire record (C3-004; `SessionSnapshot` is the canonical wire
   boundary type per BC-2.08.008 Invariant 1).

## Invariants

1. The `--settings` argument is the ONLY mechanism for hook injection in monocle-launched
   sessions. There is no alternative path (BC-HOOK-028).
2. The `lock.app = 'monocle'` filter in hooks-settings.json is REQUIRED for every
   monocle-launched session. A hooks-settings.json without this filter would cause ALL
   `claude` processes on the system to route hooks to monocle, not just monocle-launched ones.
   This invariant is enforced at the function level: `session_manager::write_hooks_settings_json`
   always emits `"lock": {"app": "monocle"}` unconditionally in its `serde_json::json!` payload.
   There is no code path through which hooks-settings.json can be written without this field,
   because all writes go through this single function (F-S038-PASS1-001 single-writer mandate).
3. **Shared hooks file (BC-HOOK-010 model):** `hooks-settings.json` is a single shared file
   at `<runtime_dir>/hooks-settings.json` — NOT per-session. It is written once at daemon
   startup (per BC-2.04.010 / BC-HOOK-010). All concurrent session spawns in the same
   `runtime_dir` reference this single file. `SpawnOptions.hooks_settings_path` always
   points to `<runtime_dir>/hooks-settings.json`. Concurrent spawns are safe because all
   spawns point to the same path and the file content is deterministic (last-write-wins is
   safe because the content is identical on every write per BC-HOOK-010 PC-2).
4. The daemon writes hooks-settings.json at daemon startup, BEFORE any `spawn_recipe()` is
   called. The file exists at the path when monocle-session-host reads it. If the file is
   missing, `claude --settings` will fail to start — this is a daemon bug, not a
   session-host responsibility.
5. **Atomic-write obligation (production-grade — per SS-conventions-anti-patterns.md §anti-patterns
   table).** The daemon's write of `hooks-settings.json` to `<runtime_dir>/hooks-settings.json`
   MUST use `tempfile::persist` (i.e., `NamedTempFile::new_in(&runtime_dir)` → write JSON →
   `persist(&hooks_settings_path)`). Naked `std::fs::write` or `tokio::fs::write` is FORBIDDEN.
   **Failure behavior:** if `tempfile::persist` fails (e.g., `NamedTempFile::new_in` returns
   `Err` because `runtime_dir` is not writable, or `persist` returns `Err` due to a cross-device
   rename or permission error), the daemon MUST exit with code 72 and log
   `ERROR: failed to write hooks-settings.json: <reason>`. No partially-written file is left at
   the target path — `tempfile::persist` guarantees atomicity by using `rename(2)` only on
   success. The daemon MUST NOT proceed to step 10 (UDS bind) if step 9 (hooks-settings.json
   write) fails; no sessions can be spawned without a valid hooks-settings.json. Authority:
   BC-2.04.010 PC-1 and SS-conventions-anti-patterns.md §"Naked config file writes".
6. **Path-canonicalization obligation.** The `hooks_settings_path` embedded in `SpawnOptions`
   (carried from daemon to session-host as the `--settings` CLI arg value) MUST be derived from
   a canonicalized `runtime_dir`. Specifically: before the daemon constructs the
   `hooks_settings_path = runtime_dir.join("hooks-settings.json")` value (stored in
   `DaemonState.hooks_settings_path`), the `runtime_dir` MUST be resolved via
   `std::fs::canonicalize(&runtime_dir)` at daemon startup (step 1 of daemon_start_sequence).
   **Error taxonomy for non-canonicalizable runtime_dir:**
   - `runtime_dir` does not exist: daemon logs
     `ERROR: runtime_dir does not exist: <path>` and exits with code 69 (EX_UNAVAILABLE).
     The TUI receives no IPC connection (daemon never binds the UDS); the TUI MUST display
     a startup error banner: `"monocle daemon failed to start — runtime directory not found: <path>"`.
   - `runtime_dir` path component traverses a non-existent intermediate directory: same as
     above (canonicalize returns `Err(NotFound)`).
   - `runtime_dir` exists but `canonicalize` fails for any other reason (e.g., permission denied
     on an intermediate symlink): daemon logs `ERROR: failed to canonicalize runtime_dir: <reason>`
     and exits with code 71 (EX_OSERR).
   - `hooks_settings_path` derived from a non-canonicalized (symlink-containing) `runtime_dir`
     is FORBIDDEN: the path passed as `--settings` must be stable across symlink re-targets and
     must match what `spawn_recipe()` embeds in `recipe.args`; a symlink race could cause the
     session-host to pass a path to `claude` that resolves to a different location than the one
     the daemon wrote. The canonicalize-at-startup model eliminates this race entirely.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-180 | `hooks_settings_path` refers to a file that was deleted after daemon startup (e.g., runtime_dir cleaned externally) | `claude --settings <deleted_path>` fails to find the file; Claude Code exits with an error; session-host sends `StateChanged::Terminated`; TUI shows "session failed to start"; daemon GC's the entry |
| EC-181 | hooks-settings.json missing `lock.app` filter (regression) | All external `claude` processes on the system would route hooks to monocle; this is a data-integrity defect; this regression is prevented by the single-writer mandate (Description §Single-writer mandate): only `write_hooks_settings_json` writes the file, and it always emits `lock.app = 'monocle'` unconditionally. No code path can produce a hooks-settings.json without this field. |
| EC-182 | hooks-settings.json file deleted between daemon startup (step 9) and a `spawn_session()` call (e.g., external process cleans runtime_dir) | `SessionManager::spawn_session()` checks for file existence before spawning. If absent, it calls `write_hooks_settings_json(&self.hook_endpoint_config, &self.hooks_settings_path)` to re-write the file. The re-written file contains REAL curl URLs (real daemon port + auth_token) and `lock.app = 'monocle'` because `self.hook_endpoint_config` was populated with the real config at `SessionManager::new()` construction (lifecycle step 9 passes the real `HookEndpointConfig` — see Description §Single-writer mandate). A re-write that uses `HookEndpointConfig::default()` (empty URLs) would produce non-functional hooks; this is FORBIDDEN in production. If the re-write fails, `spawn_session()` logs ERROR and continues; the session-host will fail when `claude --settings` cannot read the file. |
| EC-182b | Two sessions spawned concurrently | Both reference the SAME `<runtime_dir>/hooks-settings.json`; no file-level conflict; each `claude` process has `--settings <runtime_dir>/hooks-settings.json` in its argv; the shared file is read-only after daemon startup (no concurrent writes during spawn under normal conditions) |
| EC-183 | `tempfile::persist` fails during step 9 (e.g., `runtime_dir` exists but a cross-device rename is attempted, or filesystem is mounted read-only after daemon start) | Daemon logs `ERROR: failed to write hooks-settings.json: <reason>` and exits with code 72. No partial file is left at the target path (tempfile guarantees this via `rename(2)` semantics — the temp file is only unlinked, never partially moved). No UDS bind happens; the TUI never receives an IPC connection; TUI displays a startup error banner (same path as binary-not-found startup failure). Invariant 5 enforces this hard-exit; a daemon that proceeds without a hooks-settings.json would spawn sessions whose hook injection silently fails. |
| EC-184 | `runtime_dir` is a symlink and the symlink target changes between canonicalize (step 1) and hooks-settings.json write (step 9) | The `hooks_settings_path` embedded in `DaemonState` and all subsequent `SpawnOptions` continues to point to the canonicalized (real) path resolved at step 1, not the symlink. The re-targeted symlink is irrelevant: `hooks-settings.json` was written to the canonical path; the `--settings` arg carries the canonical path; the file is accessible by `claude`. No error; no divergence. Invariant 6 (canonicalize-at-startup) is the mechanism that eliminates this race. |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Single session spawn via `MockSessionHostSpawner` | `CommandBuilder.args` contains `["--settings", "<runtime_dir>/hooks-settings.json"]` | happy-path |
| Session spawned via SessionManager; inspect spawned process `argv` | `argv` contains `--settings <runtime_dir>/hooks-settings.json` | integration |
| Two concurrent spawns | Both spawns reference the SAME `<runtime_dir>/hooks-settings.json`; no distinct hooks files; file is valid JSON after both spawns complete | happy-path |
| Daemon startup with `runtime_dir` pointing to a symlink (Invariant 6) | `DaemonState.hooks_settings_path` equals `std::fs::canonicalize(runtime_dir).unwrap().join("hooks-settings.json")`; symlink does NOT appear in the stored path | unit |
| `tempfile::persist` injected to return `Err(...)` in step 9 (Invariant 5 — EC-183) | Daemon exits with code 72; no UDS socket created; `hooks-settings.json` is absent or unchanged at the target path | unit (daemon startup) |
| `runtime_dir` does not exist at daemon startup (Invariant 6 — EC-184 precursor) | Daemon exits with code 69; logs `ERROR: runtime_dir does not exist: <path>`; no UDS bind | unit (daemon startup) |
| hooks-settings.json written by lifecycle step 9 (production path, real port+token) | File parses as valid JSON; contains `"lock": {"app": "monocle"}`; PreToolUse/Notification/Stop/UserPromptSubmit arrays each contain one hook object with a non-empty `"command"` curl string pointing to `127.0.0.1:<port>/hooks/<endpoint>`; PostToolUse and PreCompact are empty arrays; no `"SessionStart"` key | unit (lifecycle hook-file writer) |
| hooks-settings.json deleted at spawn time (EC-182 re-write path) | `spawn_session()` re-writes the file via `write_hooks_settings_json(&self.hook_endpoint_config, ...)`; re-written file has `lock.app = 'monocle'` and non-empty curl URLs (real port+token from `hook_endpoint_config`); spawn proceeds normally | unit (spawn_session EC-182 path) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `CommandBuilder` produced by session-host has `--settings <runtime_dir>/hooks-settings.json` in args | unit (session-host startup sequence) |
| VP-TBD | hooks-settings.json content includes `lock.app = 'monocle'` | unit (daemon hook-file writer) |
| VP-TBD | Concurrent spawns all reference the shared `<runtime_dir>/hooks-settings.json`; no per-session file created | unit |
| VP-TBD | hooks-settings.json write uses `tempfile::persist`; injected persist failure → daemon exits code 72 with no partial file at target path (Invariant 5) | unit (daemon startup; fault injection) |
| VP-TBD | `DaemonState.hooks_settings_path` is derived from canonicalized `runtime_dir`; symlink runtime_dir → real path stored (Invariant 6) | unit (daemon startup) |
| VP-TBD | Non-existent `runtime_dir` at startup → daemon exits code 69; no UDS bind (Invariant 6 error path) | unit (daemon startup) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — hook auto-injection on spawn is explicitly named in CAP-008; this BC defines the complete injection chain from daemon hook-file write through to child process argv |
| L2 Domain Invariants | DI-007 (monocle must not write to any file owned by a harness — hooks-settings.json is written to monocle's runtime_dir, NOT to Claude Code's config directory; the `--settings` flag mechanism ensures monocle does not touch `~/.monocle/settings.json` or any Claude Code-owned path) |
| Architecture Module | monocle-runtime (SessionManager spawn; daemon hook-file writer); monocle-session-host (CommandBuilder construction from recipe) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v2.18.0 §SpawnRecipe integration with EngineModule (single-writer mandate, HookEndpointConfig construction); SS-engine-module-v2-delta.md v1.6.0 §Hook auto-injection invariant; SS-daemon-wiring-v2-delta.md v1.12.0 §lifecycle step 9 single-writer call; BC-HOOK-027; BC-HOOK-028 |
| Cross-Ref | BC-2.03.005 (spawn_recipe() produces --settings arg); BC-HOOK-027 (monocle never writes ~/.monocle/settings.json); BC-HOOK-028 (no env-var alternative for hook injection); BC-2.04.010 (hook tmpfile generation — writes shared per-runtimeDir hooks-settings.json at daemon startup); BC-HOOK-010 (authoritative: hooks-settings.json is per-runtimeDir, not per-session) |
| Test Name | test_BC_2_08_006_hook_auto_injection_settings_arg_in_child_argv |

## Related BCs

- [BC-2.03.005] — depends on: spawn_recipe() constructs the --settings arg
- [BC-HOOK-027] — depends on: hook injection is via --settings only; ~/.monocle/settings.json never written
- [BC-2.04.010] — depends on: hooks-settings.json written by daemon before spawn

## Architecture Anchors

- `architecture/SS-engine-module-v2-delta.md#hook-auto-injection-invariant` — lock.app filter + --settings arg mechanism
- `architecture/SS-session-manager.md#spawnrecipe-integration-with-enginemodule` — daemon hook-file writer obligation

## Story Anchor

S-038 — Implement hook auto-injection in session spawn path

## VP Anchors

VP-TBD — Hook injection end-to-end tests (filled after VP creation)


## §Trace 1.5.4

**SS-session-manager arch-source pin cascade v2.17.0→v2.17.1 (F-S042-ADV-MED-001 ownership-drift cleanup)** (2026-06-21):
- Architecture Source pin updated: SS-session-manager.md v2.17.0 → v2.17.1. No behavioral content changed.
- SE-16d monotonicity: 1.5.4 > 1.5.3. PASS.

## §Trace 1.5.3

**SS-session-manager arch-source pin cascade v2.16.0→v2.17.0** (2026-06-21):
- Architecture Source pin updated: SS-session-manager.md v2.16.0 → v2.17.0. No behavioral content changed.
- SE-16d monotonicity: 1.5.3 > 1.5.2. PASS.

## §Trace v1.5.2

**SS-session-manager v2.15.1 → v2.16.0 Architecture Source pin cascade (Ruling A errata)** (2026-06-21):
- Architecture Source pin updated. No behavioral content changed.
- SE-16d monotonicity: v1.5.2 > v1.5.1. PASS.

## §Trace v1.5.0

**F-S038-PASS1-001 BLOCKER + F-S038-PASS1-002 HIGH + F-S038-PASS1-005 MED — Single-writer mandate + HookEndpointConfig real-config construction + EC-182 real-config re-write** (2026-06-19):

- **Root problem (F-001 BLOCKER):** Two divergent writers of `hooks-settings.json` existed in S-038:
  (1) `lifecycle::write_hooks_settings` — production writer, called at step 9 before UDS bind. Uses
  `HooksSettings`/`HooksMap`/`HookEntry` types that have NO `lock` field → production file lacked
  `lock.app = 'monocle'` → BC-2.08.006 Invariant 2 / EC-181 violated.
  (2) `session_manager::write_hooks_settings_json` — S-038's new function, emits `lock.app` correctly
  but used `HookEndpointConfig::default()` (empty URLs) in `SessionManager::new()` under an
  `if !path.exists()` guard. Since the lifecycle file already existed, S-038's writer was dead code
  in production.
- **Decision (single-writer mandate):** `session_manager::write_hooks_settings_json` is the SOLE
  canonical writer. `lifecycle::write_hooks_settings` (with its `HooksSettings`/`HooksMap` types)
  is removed. Lifecycle step 9 calls `session_manager::write_hooks_settings_json` with a
  `HookEndpointConfig` constructed from the daemon's real `port` and `auth_token`.
- **EC-182 real-config obligation (F-002 HIGH + F-005 MED):** `SessionManager::new()` now receives
  a caller-provided `HookEndpointConfig` (populated by lifecycle from real port+token). The
  `if !path.exists()` write-on-absent guard in `new()` is removed — lifecycle is the sole step-9
  writer; `new()` only stores the config for EC-182 re-writes. The EC-182 re-write in
  `spawn_session()` uses `self.hook_endpoint_config` which now holds real curl URLs + lock.app.
- **Abort-on-failure ownership:** Lifecycle step 9 calls `write_hooks_settings_json` and propagates
  `Err` → `DaemonStartError::HooksSettingsWriteFailure` → daemon exits code 72. `SessionManager`'s
  EC-182 re-write logs ERROR and continues (non-fatal: spawn may still fail if file truly absent,
  which is the correct behavior; daemon has already started successfully at that point).
- **Changes to this BC:** Description updated with §Single-writer mandate. Invariant 2 updated to
  name the enforcement mechanism. EC-181 updated (regression path is now impossible by construction).
  EC-182 rewritten to describe the file-deleted-at-spawn scenario with real-config re-write
  obligation. EC-182b added for concurrent-spawns (previously mislabeled EC-182). Test vectors
  updated: two new rows for production-writer lock.app assertion and EC-182 real-content assertion.
  Architecture Source pin updated to SS-session-manager v2.15.0 + SS-daemon-wiring-v2-delta v1.12.0.
- SE-16d monotonicity: v1.5.0 timestamp 2026-06-19 >= v1.4.1 timestamp 2026-06-19. PASS (same-day
  sequential minor bump).

## §Trace v1.4.1

**SS-session-manager v2.13.0 → v2.14.0 Architecture Source pin cascade (F-S035-PASS5-MED-001)** (2026-06-19T00:00:00Z):
- Architecture Source pin: SS-session-manager.md v2.13.0 → v2.14.0 (v2.14.0 adds EC-188
  timeout → Terminated subpath (d) in the attach_session Detached cell of the action×state
  matrix — F-S035-PASS5-MED-001). No behavioral content changes to this BC.
- SE-16d monotonicity: v1.4.1 timestamp 2026-06-19 >= v1.4.0 timestamp auto. PASS.

## §Trace v1.3.1

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-038** (2026-06-15):
- Story Anchor filled from Phase-2 Burst A story decomposition. No behavioral content changed.

## §Trace v1.3.0

**F-P53-001 IMPORTANT — PC-3 authority corrected from BC-HOOK-007 to BC-2.04.010** (2026-06-14):
- PC-3 rewrote to anchor on BC-2.04.010 PC-3 / SS-daemon-wiring.md §Hook Tmpfile Generation
  instead of BC-HOOK-007.
- BC-HOOK-007 governs the DTU gene-source clone's `WriteHooksSettingsFile` (5 keys: PreToolUse,
  Notification, Stop, SessionStart, UserPromptSubmit; no PreCompact). It does NOT govern the
  v1A daemon-written hooks-settings.json.
- BC-2.04.010 PC-3 is the authoritative schema for the v1A daemon file: 4 URL-bearing keys
  (PreToolUse, Notification, Stop, UserPromptSubmit) + PostToolUse:[] + PreCompact:[] +
  no SessionStart key.
- SessionStart is explicitly excluded from the file: Claude Code fires this event via its own
  internal lifecycle; monocle's axum router handles it but it is not configured through
  hooks-settings.json.
- Lock object description retained unchanged.
- No other BC files require changes (whole-class sweep: only this file cited BC-HOOK-007 as
  the v1A daemon file content authority; all other SessionStart references in BC layer describe
  the served HTTP endpoint set or DTU clone scope).
- Minor bump: v1.2.1 → v1.3.0 (postcondition rewrite).

## §Trace v1.2.1

**Arch-source pin v1.4.0→v1.4.1 (architect C34-001 bump)** (2026-06-13 / D-276):
- Architecture Source updated: SS-engine-module-v2-delta.md v1.4.0 → v1.4.1.
- Reason: architect bumped SS-engine-module-v2-delta.md to v1.4.1 to correct the null-byte
  detection mechanism in spawn_recipe() (C34-001). The hook auto-injection invariant in this
  BC is unaffected — it governs the `--settings` arg assembly and `lock.app` filter content,
  not the null-byte detection path.
- Patch bump only.

## §Trace v1.2.0

**SUG-002 adversarial pass-4 fix (PO half) — PC-5: EnrichedSession → SessionSnapshot wire type** (2026-06-03):
- PC-5: "the daemon sets `spawned_by_monocle: Some(true)` on the `EnrichedSession` record" →
  "on the `SessionSnapshot` wire record (C3-004)". `SessionSnapshot` is the canonical wire
  boundary type per BC-2.08.008 Invariant 1; `EnrichedSession` is the internal aggregate type
  that does not cross the IPC boundary. The `spawned_by_monocle` field is the same field —
  only the type name is corrected to match the wire boundary.

## §Trace v1.1.0

**C1 HOOK-MODEL reconciliation — shared hooks file (BC-HOOK-010 wins)** (2026-06-03):
- Invariant 3 revised: removed per-session `hooks-<session_id>.json` model; replaced with
  shared `<runtime_dir>/hooks-settings.json` per BC-HOOK-010 (the winning architecture
  decision from adversarial pass 1). Concurrent spawns all reference the single shared file;
  no clobbering because the file is written once at daemon startup and is read-only during spawn.
- EC-182 corrected: two concurrent spawns now correctly describe the shared-file outcome
  (same path, identical content, last-write-wins safe) rather than the now-retired
  per-session path model.
- Test vectors and VP table updated to reference `<runtime_dir>/hooks-settings.json`.
- Cross-Ref updated: BC-HOOK-010 added as authoritative reference.
- Architecture Source updated to reflect BC-2.04.010 shared-file semantics (not extended).

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.006 authored for SS-08 as part of the v1A control-center pivot BC burst.
- NOTE: Invariant 3 in v1.0.0 incorrectly specified per-session hooks paths. This was
  superseded by the C1 architecture decision (BC-HOOK-010 model wins) in v1.1.0.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).

## §Trace v1.3.2

**Phase-2 Pass-1 fix burst — SS-session-manager v2.7.3 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.3.2 timestamp >= v1.3.1. PASS.

## §Trace v1.4.0

**F-P20-BCGAP-001 — Atomic-write + path-canonicalization clauses added** (2026-06-16):
- **Finding:** BC-2.08.006 lacked dedicated atomic-write and path-canonicalization clauses despite
  SS-conventions-anti-patterns.md §"Naked config file writes" requiring `tempfile::persist` for all
  config/state writes and the `hooks_settings_path` being derived from externally-supplied path
  inputs.
- **Invariant 5 added:** Atomic-write obligation — the daemon's write of `hooks-settings.json`
  MUST use `tempfile::persist`; failure exits the daemon with code 72; no partial file is left
  at target. Authority: BC-2.04.010 PC-1 + SS-conventions-anti-patterns.md.
- **Invariant 6 added:** Path-canonicalization obligation — `runtime_dir` MUST be canonicalized
  via `std::fs::canonicalize` at daemon startup step 1 before `hooks_settings_path` is
  constructed; three-case error taxonomy defined (does not exist → exit 69; other canonicalize
  failure → exit 71; symlink race eliminated by design).
- **EC-183 added:** `tempfile::persist` failure path (step 9) → code 72, no partial file, no UDS
  bind.
- **EC-184 added:** Symlink runtime_dir retarget after canonicalize → path in `DaemonState` is
  stable; no divergence.
- **Test vectors added:** 3 new rows covering symlink canonicalization, persist fault injection,
  and missing runtime_dir.
- **Verification properties added:** 3 new VP-TBD rows for Invariants 5 and 6.
- Minor bump: v1.3.2 → v1.4.0 (new normative content in Invariants, Edge Cases, Test Vectors,
  Verification Properties).
- SE-16d monotonicity: v1.4.0 timestamp 2026-06-16 >= v1.3.2 timestamp 2026-06-16. PASS (same-day sequential minor bump).
