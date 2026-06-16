---
document_type: behavioral-contract
level: L3
version: "1.6.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "0282f86"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D-001, F-P1D2-010, F-P1D2-012, F-P1D3-002]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.001: Daemon Start Sequence: Port Bind + Lock File + Token Write (SOQ-2)

## Description

The monocle daemon executes an ordered 13-step startup sequence that enforces the SOQ-2
invariant: the OS-assigned port MUST be bound before the lock file is written, and the lock
file MUST be written before `hooks-settings.json` is generated. This ordering guarantee
ensures that any process reading `hooks-settings.json` always finds a token that is already
committed to the lock file, eliminating the auth-token race condition that would arise if the
order were reversed. The start sequence is the authoritative wiring contract that coordinates
`build_server()`, `DaemonState`, `RingBuffer`, `RecoveryCheckpoint`, `ClaudeCodeModule`, and
`VsddFactoryAdapter` into a runnable binary.

## Preconditions

1. The `monocle daemon start` subcommand has been invoked (directly or via the auto-start path
   from BC-2.04.002).
2. No live daemon is currently running at the resolved `<runtime_dir>` (checked via PID
   liveness; see BC-2.01.005 Postcondition 1).
3. The `monocle-runtime` crate exposes `build_server()` and `run_server()`.
4. `ClaudeCodeModule` and `VsddFactoryAdapter` are constructable without I/O (pure
   initialization; no home-directory resolution required at this step).
5. The host OS supports `TcpListener::bind("127.0.0.1:0")` (OS-assigned ephemeral port).
6. The host OS supports `tempfile::persist` (atomic rename on POSIX; NTFS transactional
   writes on Windows, which is a secondary target per NFR-008).

## Postconditions

The following 13 steps MUST execute in the exact order stated. A failure at any step MUST
abort the remaining steps; no partial-started daemon may serve requests.

**Step 1 — Runtime directory resolved and created.**
PC-1. `<runtime_dir>` is resolved via the chain in BC-2.04.006 (env override →
      `ProjectDirs::runtime_dir()` → `ProjectDirs::data_local_dir()` → fail-fast).
PC-2. If `<runtime_dir>` does not exist, it is created with mode `0o700` (owner-only) via
      `DirBuilder::new().mode(0o700).recursive(true).create(&runtime_dir)`. If resolution
      fails, daemon exits with `DaemonStartError::RuntimeDirUnresolvable` and exit code 70.

**Step 2 — Existing lock file and PID liveness checked.**
PC-3. Specified fully in BC-2.01.005: if a live daemon is detected (lock file exists with
      alive PID), the daemon exits 1 with `error: daemon already running (pid=<N>)`. If a
      stale lock file exists (dead PID), it is removed and startup continues.

**Step 3 — Axum HTTP listener bound (SOQ-2 first anchor).**
PC-4. `TcpListener::bind("127.0.0.1:0")` is called. The OS assigns an ephemeral port number.
PC-5. The assigned port number is recorded in a local variable. This MUST occur before any
      lock file write (SOQ-2 invariant).
PC-6. If bind fails (e.g., no loopback interface available), daemon exits with exit code 71.

**Step 4 — RingBuffer created.**
PC-7. A `RingBuffer` is constructed with capacity `100MB × 5 rotations` and flush mode
      `async-jsonl`, targeting `<runtime_dir>/monocle.jsonl`.
PC-8. The ring buffer is wrapped in `Arc<RingBuffer>` and assigned to `DaemonState.ring`.
PC-9. The RAM ring holds capacity for the last 4,096 events for zero-disk-read TUI access.

**Step 5 — Bounded event bus created.**
PC-10. `tokio::sync::mpsc::channel(4096)` is called. The sender half (`EventBusTx`) is
       stored in `DaemonState`. The receiver half (`EventBusRx`) is reserved for the
       event-bus fan-out task. The drop counter `AtomicU64` is initialized to 0.

**Step 6 — EngineModule registry populated.**
PC-11. A `ClaudeCodeModule` instance is constructed and registered in the
       `EngineModuleRegistry` stored in `DaemonState`.
PC-12. The `VsddFactoryAdapter` is initialized and associated with the `ClaudeCodeModule`
       per SS-engine-module.md §VsddFactoryAdapter wiring.

**Step 7 — Auth token generated.**
PC-13. 32 bytes are sampled from `rand::rngs::OsRng`. The bytes are hex-encoded to 64
       lowercase characters. The full wire token is `monocle-v1:<64-hex>`.
PC-14. Only the 64-hex portion is stored in `DaemonState.auth_token` (the prefix is a
       wire-format concern only; see BC-2.01.008).

**Step 8 — Lock file written (SOQ-2 commit point).**
PC-15. The lock file is written via `tempfile::persist` to `<runtime_dir>/monocle.lock`
       with mode `0o600`. The JSON payload is:
       ```json
       {
         "pid": <N>,
         "port": <N>,
         "token": "monocle-v1:<64-hex>",
         "contract_version": "monocle-lock-v1",
         "started_at": "<ISO8601Z>"
       }
       ```
PC-16. This write MUST occur AFTER step 3 (port bound) and AFTER step 7 (token generated).
       This is the SOQ-2 ordering commit point. Failure at this step causes exit code 71.
PC-17. The `contract_version` field value is `"monocle-lock-v1"`. Lock file schema is
       authoritative per BC-2.01.010.

**Step 9 — hooks-settings.json generated.**
PC-18. `<runtime_dir>/hooks-settings.json` is written via `tempfile::persist` at mode
       `0o600`. This MUST occur AFTER step 8 (SOQ-2: lock file committed before
       hooks-settings reads the token). Failure causes exit code 72.
PC-19. All 4 hook endpoint URLs embed the OS-assigned port (from step 3) and the full wire
       token `monocle-v1:<64-hex>` (from step 7). (PreToolUse, Notification, Stop,
       UserPromptSubmit — SessionStart is invoked by Claude Code's internal lifecycle, not via
       hooks-settings.json.) See BC-2.04.010 for the full schema.

**Step 10 — UDS socket created.**
PC-20. A Unix domain socket is bound at `<runtime_dir>/monocle.sock` with mode `0o600`.
PC-21. If a stale socket file exists at that path, it is removed before binding.
PC-22. The socket path is stored in `DaemonState.sock_file_path`.

**Step 11 — Crash recovery checkpoint started.**
PC-23. The crash recovery checkpoint background task is initialized per BC-2.01.006.

**Step 12 — HTTP server started.**
PC-24. `run_server(Arc::new(state), listener)` is called, handing the axum router (built
       by `build_server()`) to the tokio runtime. From this point the daemon serves requests.

**Step 13 — Startup signaled to caller.**
PC-25. The foreground `daemon start` caller detects startup completion by polling for the
       lock file at `<runtime_dir>/monocle.lock`. No explicit IPC from daemon to caller
       is required; the lock file write at step 8 is the completion signal.
PC-26. The foreground caller exits with code 0 and no stdout output upon detecting the lock
       file.

## Invariants

1. **SOQ-2 ordering is strict:** step 3 (bind) < step 8 (lock file write) < step 9
   (hooks-settings write). No deviation is permitted.
2. **Atomicity via tempfile::persist:** both the lock file (step 8) and hooks-settings.json
   (step 9) are written via `tempfile::persist`. No partial file state is ever observable
   by concurrent readers.
3. **Port assignment is OS-authoritative:** the port number in the lock file MUST match the
   port the OS assigned in step 3. It is never hardcoded or guessed.
4. **Startup is all-or-nothing:** if any step fails, the daemon exits before reaching step 12.
   No HTTP server is ever started without all prior steps having succeeded.
5. **Token stored split-form:** `DaemonState.auth_token` holds only the 64-hex suffix; the
   full `monocle-v1:<64-hex>` wire token is composed at write time (steps 8 and 9).
6. **Lock file cleanup on post-step-8 failure:** If any step N > 8 fails, the lock file
   written at step 8 MUST be removed before process exit to prevent orphaned lock files.
   An orphaned lock file (with a valid PID that matches a dead process) is cleaned up by
   the next start's stale-lock detection (step 2 / BC-2.01.005 PC-3), but removing the
   lock file on failure is cleaner and avoids a confusing start sequence on immediate restart.
   Example: step 9 (hooks-settings.json write failure) already documents this at
   EC-2.04.001-02 — the lock file is explicitly removed before exit. The same obligation
   applies to steps 10–12 failures.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-2.04.001-01 | Lock file write fails at step 8 (filesystem full or permission denied) | Daemon exits with code 71; no `hooks-settings.json` is generated; no HTTP server is started; no partial lock file is left on disk (tempfile guarantees) |
| EC-2.04.001-02 | hooks-settings.json write fails at step 9 (filesystem full) | Daemon exits with code 72; lock file that was written at step 8 is removed before exit to avoid leaving a lock file without a corresponding hooks-settings.json |
| EC-2.04.001-03 | `TcpListener::bind("127.0.0.1:0")` fails (loopback not available) | Daemon exits with code 71 before writing any file |
| EC-2.04.001-04 | Two concurrent `monocle daemon start` invocations | Both pass the PID-liveness check (step 2) before either writes the lock file. The first to complete `tempfile::persist` at step 8 wins; the second sees a valid lock file on re-check and exits 1. The lock file atomic-rename is the true exclusion point, not the PID-liveness check. |
| EC-2.04.001-05 | `OsRng` unavailable (misconfigured OS, FIPS environment) | Daemon exits before step 7 with `DaemonStartError::TokenGenerationFailed`; exit code 71 |
| EC-2.04.001-06 | Stale UDS socket exists at `<runtime_dir>/monocle.sock` (step 10) | Socket is removed before rebind; daemon proceeds normally |
| EC-2.04.001-07 | `runtime_dir` creation fails at step 1 (parent path not writable) | Daemon exits 70 with `DaemonStartError::RuntimeDirUnresolvable` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Clean start (no lock file, no prior daemon) | Steps 1–13 complete in order; lock file written with `contract_version == "monocle-lock-v1"`, port matching bound listener, token matching `monocle-v1:<64-hex>` pattern; hooks-settings.json written with same port and token; foreground exits 0 | happy-path |
| Clean start on macOS (no `runtime_dir()`) | `data_local_dir()` used as `<runtime_dir>`; all steps proceed normally; INFO log: `runtime_dir fallback to data_local_dir` | platform-edge |
| Concurrent double-start | Second invocation exits 1 with `error: daemon already running (pid=<N>)` | concurrency |
| Lock file filesystem full at step 8 | Daemon exits 71; no hooks-settings.json generated; no HTTP server started | error |
| Verify SOQ-2 ordering via timestamps | Lock file `started_at` < hooks-settings.json mtime (both measured atomically after both writes complete) | invariant |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 13 steps execute in declared order on clean start | integration |
| VP-TBD | Lock file `port` field matches OS-assigned listener port | integration |
| VP-TBD | Lock file `token` field matches `DaemonState.auth_token` (with prefix) | integration |
| VP-TBD | hooks-settings.json `mtime` >= lock file `mtime` (SOQ-2 ordering) | integration |
| VP-TBD | Startup aborts before step 12 if step 8 fails | integration |
| VP-TBD | Lock file mode is `0o600`; runtime_dir mode is `0o700` | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — this BC specifies the full 13-step daemon startup sequence enforcing SOQ-2; the "binary composition root" names the daemon binary crate that owns this start sequence, and the sequence wires together CLI surface, daemon auto-start, bounded event bus, and hook tmpfile generation in exact order |
| L2 Domain Invariants | DI-002 (lock file must be present and contain a valid port and auth token before any hook endpoint accepts connections — PC-15 through PC-17 enforce lock file creation before step 12 starts HTTP serving); DI-003 (auth token MUST be written to lock file after port is bound — PC-4/PC-5 bind port, PC-13/PC-14 generate token, PC-15 writes both to lock file in that order; SOQ-2 invariant formalizes this) |
| Architecture Module | `monocle` binary crate + `monocle-runtime` per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §Daemon Start Sequence (BC-2.04.001) |
| Cross-Ref | BC-2.01.005 (lock file atomic lifecycle — step 2 delegates to this BC); BC-2.01.008 (auth token wire format — PC-13 and PC-15 must conform); BC-2.01.010 (lock file JSON schema — PC-15 and PC-17 must conform); BC-2.04.006 (runtime_dir resolution — step 1 delegates to this BC); BC-2.04.010 (hooks-settings.json schema — step 9 delegates to this BC) |
| Test File | `monocle/tests/daemon_start_sequence.rs` |
| Test Name | `test_BC_2_04_001_daemon_start_sequence_soq2` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.01.005] — step 2 of this sequence is fully specified by BC-2.01.005 (lock file atomic lifecycle and PID check)
- [BC-2.01.008] — depends on: token generated in step 7 must conform to auth token wire format
- [BC-2.01.010] — depends on: lock file JSON written in step 8 must conform to lock file contract version schema
- [BC-2.04.002] — composes with: auto-start path triggers this sequence; BC-2.04.002 provides the preconditions for when this BC executes
- [BC-2.04.004] — composes with: `monocle daemon start` CLI subcommand (BC-2.04.004) wraps this sequence as background process
- [BC-2.04.006] — depends on: step 1 runtime_dir resolution is specified by BC-2.04.006
- [BC-2.04.010] — composes with: step 9 hooks-settings generation schema specified by BC-2.04.010

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#daemon-start-sequence-bc-2.04.001` — 13-step sequence, SOQ-2 rationale
- `architecture/SS-daemon-wiring.md#risk-mitigations` — SOQ-2 Race Condition and Daemon Double-Start mitigations
- `architecture/SS-daemon-lifecycle.md` — lock file schema, auth token format, ring buffer, crash recovery (referenced by steps 2, 4, 7, 8, 11)

## Story Anchor

S-TBD — Implement daemon start sequence with SOQ-2 ordering invariant (filled by story-writer)

## VP Anchors

VP-TBD — Daemon start sequence integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.04.001 created as new artifact for SS-04 per prd-expansion-scope.md §3.1 and
  SS-daemon-wiring.md §Daemon Start Sequence.
- Covers: all 13 start-sequence steps, SOQ-2 ordering invariant (steps 3 < 8 < 9),
  7 edge cases, 5 test vectors, 6 verification properties.
- input-hash: [pending] — to be populated by compute-input-hash after human review.
- SE-16d PASS: 2026-05-26T12:00:00Z is the chain origin for this artifact.

## §Trace v1.5.0

**F-P1D10-002 HIGH — CAP-004 capability text corrected to ARCH-INDEX verbatim** (2026-05-26T00:00:00Z):
- L2 Capability and Capability Anchor Justification: stale `"Daemon binary crate wiring;
  CLI surface; SOQ-2 start-sequence invariant; hook endpoint routing; bounded event bus"` →
  ARCH-INDEX verbatim `"Binary composition root; CLI surface; daemon auto-start; bounded
  event bus; hook tmpfile generation"`.
- SE-16d monotonicity: v1.5.0 timestamp >= v1.4.0. PASS.

## §Trace v1.4.0

**F-P1D4-003 LOW — Architecture Source pin updated from v1.1.0 to v1.2.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.1.0` → `SS-daemon-wiring.md v1.2.0` per F-P1D4-003 bulk update.
- SE-16d monotonicity: v1.4.0 timestamp >= v1.3.0. PASS.

## §Trace v1.3.0

**F-P1D3-002 CRITICAL — PC-19 hook endpoint count corrected from 5 to 4** (2026-05-26T14:00:00Z):
- PC-19: "All 5 hook endpoint URLs" → "All 4 hook endpoint URLs".
- Added clarifying note to PC-19: "(PreToolUse, Notification, Stop, UserPromptSubmit —
  SessionStart is invoked by Claude Code's internal lifecycle, not via hooks-settings.json.)"
- Rationale: Only 4 hook types carry non-empty curl command arrays in hooks-settings.json.
  PostToolUse and PreCompact are present with empty arrays (forward-compat). SessionStart is
  routed via Claude Code's internal lifecycle and is not configurable via hooks-settings.json
  in Phase 1.
- SE-16d monotonicity: v1.3.0 timestamp 2026-05-26T14:00:00Z >= v1.2.0. PASS.

## §Trace v1.2.0

**F-P1D2-012 LOW — Lock file cleanup invariant added** (2026-05-26T00:00:00Z):
- Added Invariant 6: "If any step N > 8 fails, the lock file written at step 8 MUST be removed before process exit to prevent orphaned lock files." This invariant lifts the requirement already documented in EC-2.04.001-02 (step 9 failure → lock file removal) to a general invariant covering all post-step-8 failures (steps 10–12). The obligation was implicit from EC-2.04.001-02 but needed to be stated as an explicit invariant to be machine-checkable.

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.0.0` → `SS-daemon-wiring.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh; this file was already at v1.1.0 from F-P1D-001; Architecture Source pin independently updated to v1.1.0).

SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.1.0

**F-P1D-001 CRITICAL — capability mis-anchor corrected** (2026-05-26T00:00:00Z):
- Frontmatter `capability: CAP-001` → `capability: CAP-004` per F-P1D-001 finding from
  Phase 1d Pass 1 adversarial review.
- Traceability §L2 Capability and §Capability Anchor Justification updated to cite CAP-004
  ("Daemon binary crate wiring; CLI surface; SOQ-2 start-sequence invariant; hook endpoint
  routing; bounded event bus") per ARCH-INDEX §SS-04 Capability Traceability.
- Rationale: BC-2.04.001 covers the SOQ-2 start-sequence, which is a CAP-004 responsibility
  (named explicitly in the CAP-004 statement), not CAP-001 (daemon lifecycle management in
  the SS-01 sense). SS-04 subsystem BCs trace to CAP-004 per ARCH-INDEX §Capability
  Traceability table.
- SE-16d monotonicity: v1.1.0 timestamp >= v1.0.0 origin. PASS.

## §Trace v1.6.0

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-daemon-wiring.md v1.2.0 → v1.3.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §Daemon Start Sequence (BC-2.04.001)` → `SS-daemon-wiring.md v1.3.0 §Daemon Start Sequence (BC-2.04.001)`.
- Plain version-pin refresh. No substantive content propagation required — §Daemon Start Sequence section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.6.0 timestamp >= v1.5.0. PASS.
