---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/SS-engine-module-v2-delta.md]
input-hash: "36836fc"
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
hook JS ensures only monocle-launched sessions trigger monocle's hook endpoints. This BC
verifies the end-to-end injection chain: daemon writes hooks-settings.json → `SpawnOptions`
carries the path → `ClaudeCodeModule::spawn_recipe()` produces `--settings <path>` in args
→ session-host builds the harness `CommandBuilder` with those args → harness child process
has `--settings` in its argv.

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
3. The hooks-settings.json at `hooks_settings_path` contains:
   - `"hooks": { ... }` — all 5 canonical hook endpoints (BC-HOOK-007), each pointing to
     `http://127.0.0.1:<daemon_port>/hooks/<endpoint>`.
   - `"lock": { "app": "monocle" }` — the filter that prevents externally-launched
     `claude` instances from sending hooks to monocle's daemon.
4. No manual user configuration of hooks is required. The user MUST NOT edit
   `~/.monocle/settings.json` or any Claude Code settings file to receive hooks
   (BC-HOOK-027 + BC-HOOK-028).
5. A monocle-launched session (via SessionManager) is distinguishable from an
   externally-launched session because the daemon sets `spawned_by_monocle: Some(true)`
   on the `EnrichedSession` record.

## Invariants

1. The `--settings` argument is the ONLY mechanism for hook injection in monocle-launched
   sessions. There is no alternative path (BC-HOOK-028).
2. The `lock.app = 'monocle'` filter in hooks-settings.json is REQUIRED for every
   monocle-launched session. A hooks-settings.json without this filter would cause ALL
   `claude` processes on the system to route hooks to monocle, not just monocle-launched ones.
3. The hooks-settings.json path MUST be a per-session unique path (e.g.,
   `<runtime_dir>/hooks-<session_id>.json`) to avoid clobbering when multiple sessions
   spawn concurrently. The existing OQ-02 mechanism (BC-2.04.010) is extended to write
   per-session hooks files rather than a single shared `hooks-settings.json`.
4. The daemon writes hooks-settings.json BEFORE calling `spawn_recipe()`. The file exists
   at the path when monocle-session-host reads it. If the file is missing, `claude --settings`
   will fail to start — this is a daemon bug, not a session-host responsibility.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-180 | `hooks_settings_path` refers to a file that was deleted after `spawn_recipe()` returned | `claude --settings <deleted_path>` fails to find the file; Claude Code exits with an error; session-host sends `StateChanged::Terminated`; TUI shows "session failed to start"; daemon GC's the entry |
| EC-181 | hooks-settings.json missing `lock.app` filter (regression) | All external `claude` processes on the system would route hooks to monocle; this is a data-integrity defect; the daemon MUST include `lock.app = 'monocle'` when writing the hooks file (invariant enforced by the daemon's hook-file writer, not by the session-host) |
| EC-182 | Two sessions spawned concurrently | Each has a unique hooks-settings.json path (session-ID-keyed); no clobbering; each `claude` process has its own `--settings` file |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Single session spawn via `MockSessionHostSpawner` | `CommandBuilder.args` contains `["--settings", "<hooks_path>"]` | happy-path |
| Session spawned via SessionManager; inspect spawned process `argv` | `argv` contains `--settings <hooks_path>` string | integration |
| Two concurrent spawns | Two different hooks files; each `claude` has its own `--settings` path | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `CommandBuilder` produced by session-host has `--settings <path>` in args | unit (session-host startup sequence) |
| VP-TBD | hooks-settings.json content includes `lock.app = 'monocle'` | unit (daemon hook-file writer) |
| VP-TBD | Concurrent spawns produce unique hooks-settings.json paths | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — hook auto-injection on spawn is explicitly named in CAP-008; this BC defines the complete injection chain from daemon hook-file write through to child process argv |
| L2 Domain Invariants | DI-007 (monocle must not write to any file owned by a harness — hooks-settings.json is written to monocle's runtime_dir, NOT to Claude Code's config directory; the `--settings` flag mechanism ensures monocle does not touch `~/.monocle/settings.json` or any Claude Code-owned path) |
| Architecture Module | monocle-runtime (SessionManager spawn; daemon hook-file writer); monocle-session-host (CommandBuilder construction from recipe) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v1.0.1 §SpawnRecipe integration with EngineModule; SS-engine-module-v2-delta.md v1.0.1 §Hook auto-injection invariant; BC-HOOK-027; BC-HOOK-028 |
| Cross-Ref | BC-2.03.005 (spawn_recipe() produces --settings arg); BC-HOOK-027 (monocle never writes ~/.monocle/settings.json); BC-HOOK-028 (no env-var alternative for hook injection); BC-2.04.010 (hook tmpfile generation — extended to per-session paths) |
| Test Name | test_BC_2_08_006_hook_auto_injection_settings_arg_in_child_argv |

## Related BCs

- [BC-2.03.005] — depends on: spawn_recipe() constructs the --settings arg
- [BC-HOOK-027] — depends on: hook injection is via --settings only; ~/.monocle/settings.json never written
- [BC-2.04.010] — depends on: hooks-settings.json written by daemon before spawn

## Architecture Anchors

- `architecture/SS-engine-module-v2-delta.md#hook-auto-injection-invariant` — lock.app filter + --settings arg mechanism
- `architecture/SS-session-manager.md#spawnrecipe-integration-with-enginemodule` — daemon hook-file writer obligation

## Story Anchor

S-TBD — Implement hook auto-injection in session spawn path (filled by story-writer)

## VP Anchors

VP-TBD — Hook injection end-to-end tests (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.006 authored for SS-08 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): Invariant 3 specifies that hooks-settings.json MUST be
  per-session (keyed by session_id) rather than a shared global file. This is a production-grade
  requirement for concurrent session spawning. The existing BC-2.04.010 uses a shared
  `hooks-settings.json`; this BC extends that mechanism to per-session files. This is resolved
  in-scope per CLAUDE.md production-grade principle (not deferred).
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
