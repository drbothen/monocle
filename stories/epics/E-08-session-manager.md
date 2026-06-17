---
document_type: epic
epic_id: EPIC-08
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
subsystems: [SS-08]
capabilities: [CAP-008]
behavioral_contracts: [BC-2.08.001, BC-2.08.002, BC-2.08.003, BC-2.08.004, BC-2.08.005, BC-2.08.006, BC-2.08.007, BC-2.08.008]
verification_properties: []
---

# EPIC-08: Session Manager

## Purpose

Implement the `SessionManager` in `monocle-runtime`: spawn session (SessionHostSpawner,
SessionEntry, sidecar write, SpawnAck, SessionStateChanged{Launching}), kill session
(SIGTERM within 500ms, 12s watchdog), attach/detach (chunked scrollback dump,
SO_PEERCRED, session-host stays alive), re-discover sessions after daemon restart
(all alive sessions visible within 5s, UDS bind blocked until complete), session GC
(Terminated sessions removed after 10s grace period), hook auto-injection
(hooks-settings.json writer, SpawnOptions.hooks_settings_path population), and
SessionStateChanged broadcast on every state transition. This epic delivers the complete
SS-08 Session Manager subsystem as the v1A control-center Phase 3 foundation.

## Success Criteria

- All 8 BC-2.08.NNN behavioral contracts pass their verification properties
- `spawn_session()` calls `SessionHostSpawner::spawn()` within 2s; `SessionEntry{Launching}` in registry; sidecar written atomically
- `kill_session()` delivers SIGTERM within 500ms; 12s watchdog SIGKILL fallback; Terminating/Terminated idempotent
- `attach_session()` triggers chunked scrollback dump (ScrollbackChunk* + ScrollbackDumpComplete); session-host stays alive on `detach_session()`
- `rediscover_sessions()` surfaces all alive sessions within 5s after daemon restart; UDS bind blocked until complete
- GC task removes Terminated sessions from registry after 10s grace period
- `hooks-settings.json` written once at daemon startup (before IPC bind) via `tempfile::persist` (0o600); `SpawnOptions.hooks_settings_path` populated before every spawn
- `SessionStateChanged` broadcast to all TUI clients on every state transition (Launching/Running/Detached/Terminating/Terminated); delivered before `SessionListUpdate`; SpawnAck before Launching
- `cargo clippy --workspace --all-targets -- -D warnings` → 0 warnings

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-033 | SessionManager::spawn_session — SessionHostSpawner, SessionEntry, Sidecar, SpawnAck, and SessionStateChanged{Launching} | 8 | Wave 8 | S-014, S-015, S-017, S-021 |
| S-034 | SessionManager::kill_session — DaemonToHost::Kill Within 500ms; Terminating/Terminated Transitions; 12s Watchdog | 8 | Wave 8 | S-033 |
| S-035 | SessionManager::attach_session and detach_session — Chunked Scrollback, SO_PEERCRED, Session-Host Stays Alive | 8 | Wave 8 | S-033 |
| S-036 | SessionManager::rediscover_sessions — setsid Persistence; All States Handled Within 5s; UDS Bind Blocked | 8 | Wave 8 | S-033, S-034, S-035 |
| S-037 | SessionManager GC Task — Terminated Sessions Removed After 10s Grace Period | 3 | Wave 8 | S-033, S-034 |
| S-038 | SessionManager Hook Auto-Injection — hooks-settings.json Writer + SpawnOptions.hooks_settings_path Population | 3 | Wave 8 | S-033 |

**Total: 38 points (all Wave 8 v1A)**

## Architecture Scope

- Implementing module: `monocle-runtime` (session_manager submodule)
- Architecture source: `architecture/SS-session-manager.md` v2.6.1
- Architecture dependency: `architecture/SS-engine-module-v2-delta.md` v1.6.0 (SpawnOptions, SpawnRecipe)
- Architecture dependency: `architecture/SS-daemon-wiring-v2-delta.md` v1.11.4 (daemon startup sequence integration)
- Architecture dependency: `architecture/SS-ipc.md` v1.24.0 (SessionStateChanged, SessionListUpdate wire types)
