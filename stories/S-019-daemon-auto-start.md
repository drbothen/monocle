---
document_type: story
level: L4
story_id: S-019
epic_id: EPIC-04
version: "1.0"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 5
wave: 5
tdd_mode: strict
priority: P0
depends_on: [S-016, S-017]
blocks: [S-023, S-025]
target_module: monocle
subsystems: [SS-04]
behavioral_contracts: [BC-2.04.002, BC-2.04.003]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.002.md, version: "1.4.0"}
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.003.md, version: "1.4.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.04.002 (daemon auto-start on TUI launch), BC-2.04.003 (MONOCLE_NO_AUTOSTART suppression)"
---

# S-019: Daemon Auto-Start on TUI Launch + MONOCLE_NO_AUTOSTART

## Narrative

As a TUI user, I want monocle to automatically start the daemon when I invoke `monocle`
without a subcommand, so that I don't need to manually run `monocle daemon start` before
opening the TUI — while also providing a `MONOCLE_NO_AUTOSTART=1` escape hatch for CI and
power users who manage the daemon lifecycle externally.

## Acceptance Criteria

### AC-001 (traces to BC-2.04.003 postcondition PC-1/PC-2 — MONOCLE_NO_AUTOSTART checked first)
`MONOCLE_NO_AUTOSTART` is read as the FIRST action in TUI mode — before any lock file
check, PID liveness check, or daemon subprocess creation.

### AC-002 (traces to BC-2.04.003 postcondition PC-3/PC-4/PC-5/PC-6 — full suppression)
When `MONOCLE_NO_AUTOSTART` is set to any non-empty string: no daemon is started, no lock
file is read, no UDS connection is attempted. The TUI launches and renders with
"daemon offline" mode active. The status bar displays `[daemon: offline]`.

### AC-003 (traces to BC-2.04.003 postcondition PC-8 — exit 0 on offline mode)
`MONOCLE_NO_AUTOSTART` suppression is a normal operating mode. The TUI exits with code 0
when the user quits. No error messages, no warnings caused by suppression itself.

### AC-004 (traces to BC-2.04.003 invariant 1 — empty string treated as unset)
`MONOCLE_NO_AUTOSTART=""` (empty string) is treated as unset. Auto-start executes normally
per BC-2.04.002. Only a non-empty value suppresses auto-start.

### AC-005 (traces to BC-2.04.003 invariant 4 / edge case EC-2.04.003-07 — subcommands unaffected)
`MONOCLE_NO_AUTOSTART=1` does NOT affect `monocle daemon start` or `monocle daemon stop`.
These subcommands are unaffected by the env var.

### AC-006 (traces to BC-2.04.002 postcondition PC-1 — runtime_dir resolved first)
When auto-start executes (no `MONOCLE_NO_AUTOSTART`), `resolve_runtime_dir()` is called first.
If resolution fails, the process exits with code 70 before rendering any TUI output.

### AC-007 (traces to BC-2.04.002 postcondition PC-2/PC-3 — liveness check)
The auto-start sequence checks for `<runtime_dir>/monocle.lock`. If the file does not exist,
proceed to PC-4 (start daemon). If the lock file exists and the PID is alive (`kill(pid, 0)`
returns 0), the TUI connects to the existing daemon without starting a new one (PC-5).

### AC-008 (traces to BC-2.04.002 postcondition PC-3 — stale lock handling)
If the lock file exists but the PID is dead (ESRCH), log `WARN: stale lock file removed`,
remove the lock file, and proceed to PC-4 (start daemon).

### AC-009 (traces to BC-2.04.002 postcondition PC-4 — daemon start + 5-second poll)
The auto-start path starts the daemon subprocess (equivalent to `monocle daemon start`)
and polls `<runtime_dir>/monocle.lock` at 100ms intervals for up to 5 seconds.
- If the lock file appears within 5 seconds: proceed to PC-5.
- If the lock file does not appear within 5 seconds: the TUI renders
  `daemon start timed out; retrying…` in the status bar and retries once (another 5-second wait).
- If the retry also fails: the TUI renders `daemon unavailable — running in offline mode`
  and continues without a daemon connection.

### AC-010 (traces to BC-2.04.002 postcondition PC-5 — UDS connection with liveness pre-check)
The TUI connects to the daemon via UDS at `<runtime_dir>/monocle.sock`. The daemon PID MUST
pass a liveness check (`kill(pid, 0)` returns 0) before the TUI attempts the UDS connection.

### AC-011 (traces to BC-2.04.002 postcondition PC-6 — no TUI content before verdict)
The TUI MUST NOT render its main content before the auto-start decision sequence completes
(steps 1-5). The main panel is rendered only after PC-5 succeeds or after offline-mode
determination at PC-4.

### AC-012 (traces to BC-2.04.002 invariant 3 — max wait = 10 seconds)
The maximum wait before offline-mode fallback is 10 seconds (5 seconds + 1 retry of 5 seconds).

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,100 |
| BC-2.04.002.md | ~900 |
| BC-2.04.003.md | ~700 |
| SS-daemon-wiring.md §Daemon Auto-Start Logic | ~2,000 |
| S-016 (daemon start CLI) | ~300 |
| Test files | ~700 |
| **Total estimate** | **~5,700** |

## Tasks

- [ ] Implement `auto_start_daemon()` in `monocle/src/main.rs` (or `monocle/src/auto_start.rs`):
  - Read `MONOCLE_NO_AUTOSTART` as the FIRST action in TUI mode path
  - If non-empty: render `[daemon: offline]` status bar; skip all daemon logic
  - If empty/unset: proceed with 5-step auto-start decision sequence
- [ ] Implement step 1: `resolve_runtime_dir()` — exit 70 on failure before TUI renders
- [ ] Implement step 2: check for lock file existence
- [ ] Implement step 3: PID liveness check (`kill(pid, 0)`) — alive → go to step 5; dead → remove stale lock → go to step 4
- [ ] Implement step 4: start daemon subprocess + 100ms poll loop (5s timeout) + 1 retry + offline-mode fallback
- [ ] Implement step 5: PID liveness pre-check before UDS connection attempt
- [ ] Ensure no TUI main content renders before auto-start verdict
- [ ] Status bar rendering: `[daemon: offline]` for offline mode, `daemon start timed out; retrying…` for timeout, `daemon unavailable — running in offline mode` for double-timeout
- [ ] Integration tests `monocle/tests/daemon_auto_start.rs`:
  - Happy path: no lock file → daemon started → TUI connects
  - Already running: lock file with alive PID → TUI connects immediately
  - Stale lock file: dead PID → WARN → stale removed → new daemon started
  - `MONOCLE_NO_AUTOSTART=1`: TUI renders offline mode, no daemon started
  - `MONOCLE_NO_AUTOSTART=0`: suppressed (non-empty string semantics)
  - `MONOCLE_NO_AUTOSTART=`: NOT suppressed (empty string treated as unset)
  - Double timeout: both 5-second waits fail → offline mode
- [ ] Unit test: `monocle daemon start` unaffected by `MONOCLE_NO_AUTOSTART=1`

## Previous Story Intelligence

S-016: `resolve_runtime_dir()` and `DaemonStartError` are available from `monocle-runtime`.
`monocle daemon start` subprocess invocation pattern is established in `cmd_daemon_start()`.

S-017: `daemon_start_sequence()` is available. The auto-start path may call this directly
in-process rather than spawning a subprocess (per BC-2.04.002 invariant 4 — implementation
detail; observable behavior is that the lock file appears within 5 seconds).

## Architecture Compliance Rules

From `architecture/SS-daemon-wiring.md v1.2.0 §Daemon Auto-Start Logic`:
- `MONOCLE_NO_AUTOSTART` check is the FIRST action in TUI mode — before any filesystem access
- Empty string `MONOCLE_NO_AUTOSTART=""` is treated as UNSET — auto-start proceeds normally
- `MONOCLE_NO_AUTOSTART=0` SUPPRESSES auto-start (any non-empty string suppresses, including "0")
- Total wait = 10 seconds (5s + 5s retry); not configurable in Phase 1
- TUI MUST NOT render main content before liveness verdict
- PID liveness check MUST precede UDS connection attempt

**Forbidden Dependencies:**
- `monocle/src/auto_start.rs` (or equivalent) MUST NOT import from `monocle-ipc` directly
  (IPC connection is handled by the TUI layer after auto-start completes)
- `MONOCLE_NO_AUTOSTART` check MUST be in TUI mode path ONLY — not in `daemon start` or `daemon stop`

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| nix | 0.30 | `kill(Pid, None)` liveness check; stale lock detection |
| tracing | 0.1 | WARN on stale lock removal; INFO on offline mode entry |
| temp-env | 0.3 (features=["async_closure"]) | Test: `MONOCLE_NO_AUTOSTART` env isolation |
| tokio | =1.52.0 | Async polling loop with `time::sleep(100ms)` |

## File Structure Requirements

Files to create:
- `monocle/src/auto_start.rs` — `auto_start_daemon()` with 5-step decision sequence
- `monocle/tests/daemon_auto_start.rs` — integration tests
- `monocle/tests/no_autostart_env.rs` — MONOCLE_NO_AUTOSTART tests

Files to modify:
- `monocle/src/main.rs` — invoke `auto_start_daemon()` in the TUI launch path before TUI init
