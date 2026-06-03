---
document_type: story
level: L4
story_id: S-DAEMON-WIRE-FIX-001
epic_id: EPIC-04
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-03T08:30:00Z
phase: 2
points: 5
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-016, S-017, S-018, S-005]
blocks: []
target_module: monocle-runtime
subsystems: [SS-01, SS-04]
behavioral_contracts: [BC-2.01.004]
verification_properties: [VP-004]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.004.md, version: "1.0.4"}
  - {path: .factory/specs/architecture/SS-daemon-wiring-impl.md, version: "1.3.0"}
  - {path: .factory/specs/architecture/SS-daemon-wiring.md, version: "1.3.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
input-hash: "[pending]"
traces_to: "Anchored deferral target for daemon-wiring adversarial Round-3 HIGH-2 finding: DaemonExit::SigtermDuringDrain (exit 143) and SigintDuringDrain (exit 130) variants are defined + documented in BC-2.01.004 INV-4 monitoring contract but NOT produced — a second SIGTERM/SIGINT during graceful drain currently hits the OS default. Code carries CONTRACT GAP (S-DAEMON-WIRE-FIX-001) markers at crates/monocle-runtime/src/lifecycle.rs variant doc-comments."
# BC status: BC-2.01.004 behavioral_contracts array is non-empty. Status must remain
# draft until PO confirms dispatch priority (post-Phase-3 convergence-fix batch).
# Anchored deferral: established by architect at SS-daemon-wiring-impl.md §Fix Addendum Round 2
# §HIGH-2 (version 1.2.0, 2026-06-03).
---

# S-DAEMON-WIRE-FIX-001: Second-Signal Exit Codes (SigtermDuringDrain=143, SigintDuringDrain=130)

## Narrative

As a system operator or CI test harness monitoring the monocle daemon, I want a second
SIGTERM sent during the graceful drain window to cause the daemon to exit with code 143
(POSIX 128+15) and a second SIGINT to cause exit code 130 (POSIX 128+2), so that
process supervisors (systemd `Restart=on-failure`, k8s `terminationGracePeriodSeconds`,
CI status parsers) can distinguish second-signal hard-kills from graceful shutdown (exit 0)
and from admin forced-stop (exit 2), per BC-2.01.004 Invariant 4.

## Background: Anchored Deferral from Daemon-Wiring Adversarial Round 3

During adversarial convergence on `feat/daemon-wire-serve` (SS-daemon-wiring-impl.md Round
2, finding HIGH-2), the architect determined that:

- `DaemonExit::SigtermDuringDrain` (exit 143) and `DaemonExit::SigintDuringDrain` (exit
  130) variants ARE defined in `crates/monocle-runtime/src/lifecycle.rs` and ARE documented
  in BC-2.01.004 PC-8 / INV-4.
- However, these variants are NOT produced by the runtime path: a second SIGTERM or SIGINT
  during graceful drain currently hits the OS signal default (abrupt termination, no controlled
  exit code path) rather than yielding 143/130 via `exit_with(DaemonExit::SigtermDuringDrain)`.
- Code carries `// CONTRACT GAP (S-DAEMON-WIRE-FIX-001)` markers at the two variant
  doc-comments in `crates/monocle-runtime/src/lifecycle.rs` as code-level deferral anchors.

The architect's design seam is specified in
`.factory/specs/architecture/SS-daemon-wiring-impl.md` §Fix Addendum Round 2 §HIGH-2:
a two-phase signal loop using `DaemonState.second_sigterm: AtomicBool`, where `main()` reads
the flag after the `run_server` / drain-timeout sequence and selects the correct `DaemonExit`
variant.

This story is the explicit anchored deferral target. It MUST NOT be confused with the
`DaemonExit::AdminForceStop` path (exit 2), which IS already implemented via the
`state.force_exit` AtomicBool.

## Acceptance Criteria

### AC-001: Second SIGTERM during drain yields exit 143 (traces to BC-2.01.004 postcondition 8, invariant 4)

When the daemon has received its first SIGTERM (entering graceful drain) and a second SIGTERM
arrives before the drain completes:

- The daemon MUST set `DaemonState.second_sigterm` (AtomicBool) to `true` on the second
  SIGTERM signal.
- After `run_server` returns (or the 10s drain timeout fires), `main()` reads
  `state.second_sigterm.load(Ordering::SeqCst)` and, if true, calls
  `exit_with(DaemonExit::SigtermDuringDrain)`.
- The process table exit code is 143 (POSIX 128+15).
- The cleanup tail (UDS socket cleanup, lock release, hooks-settings removal, ring flush
  drain) MUST run before `exit_with` is called — no cleanup is skipped on second-signal
  paths.

### AC-002: Second SIGINT during drain yields exit 130 (traces to BC-2.01.004 postcondition 8, invariant 4)

When the daemon has received its first shutdown signal and a second SIGINT (Ctrl-C) arrives
during the drain window:

- The daemon MUST set `DaemonState.second_sigint` (AtomicBool, analogous to
  `second_sigterm`) to `true` on the second SIGINT signal.
- `main()` reads `state.second_sigint.load(Ordering::SeqCst)` and, if true, calls
  `exit_with(DaemonExit::SigintDuringDrain)`.
- The process table exit code is 130 (POSIX 128+2).
- Cleanup tail runs before `exit_with`.

### AC-003: First-signal graceful path unchanged — exit 0 (traces to BC-2.01.004 postcondition 8)

When only one shutdown signal arrives and all in-flight requests drain within 10 seconds:

- `DaemonState.second_sigterm` and `DaemonState.second_sigint` remain `false`.
- `state.force_exit` remains `false` (no second POST /shutdown).
- `main()` calls `exit_with(DaemonExit::Graceful)`, yielding exit code 0.
- This verifies the second-signal detection path does not regress the happy path.

### AC-004: AdminForceStop path (exit 2) unchanged (traces to BC-2.01.004 postcondition 8)

A second authenticated `POST /shutdown` during drain continues to set `state.force_exit`
and yield exit code 2 (`DaemonExit::AdminForceStop`). The second-signal AtomicBools must
not interfere with this path. Priority: if both `second_sigterm` and `force_exit` are set,
`force_exit` (admin forced-stop) takes precedence per BC-2.01.004 postcondition 8 ordering.

### AC-005: E2E integration test — second SIGTERM → exit 143 (traces to BC-2.01.004 invariant 4, canonical test vectors row "Second SIGTERM during drain")

An integration test in `crates/monocle-runtime/tests/daemon_e2e_serve.rs` (or a new file
`second_signal_exit_codes.rs`) MUST:

1. Spawn the real `monocle-runtime` binary with `MONOCLE_RUNTIME_DIR=<tmpdir>`.
2. Wait for `monocle.lock` to appear (max 5 seconds, 50ms poll).
3. Use `MONOCLE_HOOK_DELAY_MS` (cargo feature `e2e-test-affordances`, per
   SS-daemon-wiring-impl.md §HIGH-1 resolution) to inject a slow in-flight request that
   will still be in-flight when the second SIGTERM arrives.
4. Send SIGTERM #1 to trigger graceful drain.
5. While the slow request keeps the server draining: send SIGTERM #2.
6. Assert the daemon exits with code 143 within 15 seconds of SIGTERM #1.
7. Assert cleanup ran: `monocle.lock` and `hooks-settings.json` are absent.

### AC-006: E2E integration test — second SIGINT → exit 130 (traces to BC-2.01.004 invariant 4, canonical test vectors row "Second SIGINT during drain")

Companion test mirroring AC-005 but with SIGINT:

1. Same spawn + slow-request injection as AC-005.
2. Send SIGTERM #1 (or SIGINT #1) to trigger graceful drain.
3. Send SIGINT #2 during drain.
4. Assert exit code 130.
5. Assert cleanup ran.

### AC-007: AtomicBool fields present on DaemonState; DaemonState::new() initializes to false (traces to BC-2.01.004 invariant 4)

`DaemonState` gains two new public fields:

```rust
/// Set to true if a second SIGTERM arrives during the graceful drain window.
/// Read by main() after run_server returns to select DaemonExit::SigtermDuringDrain.
/// // CONTRACT GAP (S-DAEMON-WIRE-FIX-001) — see lifecycle.rs variant doc-comment.
pub second_sigterm: AtomicBool,

/// Set to true if a second SIGINT arrives during the graceful drain window.
pub second_sigint: AtomicBool,
```

`DaemonState::new()` (unit-test constructor) initializes both to `false`. Unit tests
confirm that the fields are present and initialized correctly.

### AC-008: Signal loop wired in run_server or main() — two-phase select! (traces to BC-2.01.004 invariant 2, invariant 4)

The signal handling implementation MUST use a two-phase approach:

**Phase 1 (first signal → initiate drain):** The existing `tokio::select!` loop in
`run_server` (or wherever shutdown is currently signalled) fires on SIGTERM or SIGINT and
initiates graceful drain exactly as today.

**Phase 2 (second signal during drain → set AtomicBool):** While `run_server` is serving
(after the first signal), a second signal handler must be active. The two-phase design per
SS-daemon-wiring-impl.md §HIGH-2:

- After initiating drain (`state.shutdown_tx.send(true)`), spawn a task (or reuse the
  existing signal future) that listens for SIGTERM and SIGINT again.
- On second SIGTERM: `state.second_sigterm.store(true, Ordering::SeqCst)`.
- On second SIGINT: `state.second_sigint.store(true, Ordering::SeqCst)`.
- The task exits when `run_server` returns (listener drops, scope exits).
- `main()` reads the AtomicBools after `run_server` returns to select the exit variant.

The architect's canonical design from SS-daemon-wiring-impl.md §HIGH-2 must be followed
exactly. Any deviation requires architect re-approval.

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `DaemonState` (new fields) | monocle-runtime | `crates/monocle-runtime/src/state.rs` | Pure (data struct) |
| Second-signal loop | monocle-runtime | `crates/monocle-runtime/src/main.rs` or `server.rs` | Effectful (signal I/O) |
| Exit-code selection in main() | monocle (binary) | `crates/monocle-runtime/src/main.rs` | Effectful (process::exit) |
| E2E test assertions | monocle-runtime tests | `crates/monocle-runtime/tests/daemon_e2e_serve.rs` (or new file) | Effectful (subprocess spawn) |

Architecture reference: `architecture/SS-daemon-wiring-impl.md` §Fix Addendum Round 2 §HIGH-2
(second-signal design seam).

Architecture compliance: SS-01 (Daemon Lifecycle) + SS-04 (Daemon Wiring) per ARCH-INDEX
Subsystem Registry. BC-2.01.004 INV-4 is the behavioral anchor for the monitoring contract.

## Behavioral Contracts

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-2.01.004 | Graceful Shutdown (10-Second Drain) | AC-001 (PC-8/INV-4), AC-002 (PC-8/INV-4), AC-003 (PC-8), AC-004 (PC-8), AC-005 (INV-4 e2e), AC-006 (INV-4 e2e), AC-007 (INV-4 state fields), AC-008 (INV-2/INV-4 signal loop) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Both second_sigterm AND force_exit set (race: POST /shutdown + SIGTERM #2 arrive simultaneously) | AdminForceStop (exit 2) takes precedence per BC-2.01.004 PC-8 ordering |
| EC-002 | Second SIGTERM arrives after drain completes and exit_with already called | OS delivers signal after process::exit — no observable effect; process already exited |
| EC-003 | Second SIGINT arrives after drain completes (same as EC-002) | No observable effect; process already exited |
| EC-004 | Second SIGTERM and second SIGINT both set (very fast double-signal) | SIGTERM takes precedence (exit 143); SIGINT (exit 130) is a less severe classification |
| EC-005 | `e2e-test-affordances` cargo feature not enabled in production build | `MONOCLE_HOOK_DELAY_MS` env var is not read; feature flag gate prevents accidental production exposure (per SS-daemon-wiring-impl.md §HIGH-1) |
| EC-006 | Slow request finishes before second signal — drain completes cleanly | `second_sigterm` / `second_sigint` remain false; exit 0; second-signal task exits cleanly (listener dropped) |

## Tasks

- [ ] **T-001**: Add `second_sigterm: AtomicBool` and `second_sigint: AtomicBool` to `DaemonState`
  struct in `crates/monocle-runtime/src/state.rs`. Initialize both to `false` in
  `DaemonState::new()`.
- [ ] **T-002**: Write RED test `test_BC_2_01_004_second_sigterm_exit_143` — E2E subprocess test
  spawning the binary, injecting a slow request (MONOCLE_HOOK_DELAY_MS), sending SIGTERM #1 +
  SIGTERM #2, asserting exit code 143. Test MUST fail before T-004 (TDD red gate).
- [ ] **T-003**: Write RED test `test_BC_2_01_004_second_sigint_exit_130` — mirror of T-002 for
  SIGINT, asserting exit code 130.
- [ ] **T-004**: Implement two-phase signal loop per AC-008 and SS-daemon-wiring-impl.md §HIGH-2
  design seam. Wire `state.second_sigterm.store(true)` on second SIGTERM;
  `state.second_sigint.store(true)` on second SIGINT.
- [ ] **T-005**: Update exit-code selection logic in `main()` to read `second_sigterm` /
  `second_sigint` AtomicBools and call the correct `DaemonExit` variant. Remove the
  `// CONTRACT GAP (S-DAEMON-WIRE-FIX-001)` markers from `lifecycle.rs` variant doc-comments.
- [ ] **T-006**: Verify AC-003 regression safety: run the full E2E graceful-shutdown test suite
  (`daemon_e2e_serve.rs` AC-E2E-001..007) and confirm exit 0 still passes.
- [ ] **T-007**: Verify AC-004 regression safety: confirm `AdminForceStop` path (exit 2 on second
  POST /shutdown) still passes.
- [ ] **T-008**: Unit tests for `DaemonState` new fields (`AtomicBool` init false, store/load
  round-trip).
- [ ] **T-009**: CI parity pre-push:
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `python3 scripts/check_version_pins.py`,
  `python3 scripts/check_structural_claims.py`,
  `cargo test --workspace`.

## Previous Story Intelligence

**From S-005 (Graceful Shutdown):** The library-level graceful shutdown was delivered with an
explicit deferral note: "Deferred: 10s drain timeout + second-signal detection + signal-path
lock release (requires main.rs wiring — tracked in durable_task_register S-005-main-wiring)."
The drain timeout was addressed in SS-daemon-wiring-impl.md C2. This story closes the
second-signal detection half of that deferral.

**From daemon-wiring adversarial convergence (feat/daemon-wire-serve):**
- The `DaemonExit::AdminForceStop` pattern (AtomicBool on DaemonState, read by main() after
  `run_server` returns) is the proven, correct architecture for the exit-code selection seam.
  The second-signal fix MUST follow the same pattern — do not invent a new mechanism.
- The `e2e-test-affordances` cargo feature (SS-daemon-wiring-impl.md §HIGH-1) provides
  `MONOCLE_HOOK_DELAY_MS` injection for E2E slow-request tests. This feature is already
  decided; use it without modification.
- The `DaemonState.force_exit: AtomicBool` field is the reference implementation for the
  `second_sigterm` / `second_sigint` fields in T-001. Replicate the field type and ordering
  discipline exactly.

**Critical wiring lesson (CLAUDE.md S-029 section):** Tests MUST drive the REAL production
path through the spawned binary. Do not test isolated helpers only — green isolated tests
mask dead code. The wiring gap (signal not connected to AtomicBool to main() exit-code
selector) is the exact defect class this story closes. Use subprocess E2E tests (AC-005,
AC-006) as the primary correctness gate.

## Architecture Compliance Rules

Extracted from `architecture/SS-daemon-wiring-impl.md` §Fix Addendum Round 2 §HIGH-2 and
CLAUDE.md conventions:

1. **AtomicBool pattern**: `DaemonState.second_sigterm` and `DaemonState.second_sigint` must
   be `AtomicBool` (not `Mutex<bool>`), consistent with `force_exit`. Load with
   `Ordering::SeqCst` in `main()` to preserve happens-before with the signal handler store.
2. **Cleanup tail always runs**: Both the second-signal paths and the graceful path must
   execute the full cleanup tail (UDS cleanup → lock release → hooks removal → ring flush
   drain) before calling `exit_with`. No cleanup may be skipped.
3. **`exit_with` is the sole `process::exit` call-site**: All process exit paths call
   `exit_with(DaemonExit::*)`. No direct `std::process::exit` calls.
4. **No unbounded signal wait**: The second-signal listener task must exit cleanly when
   `run_server` returns. It must not hold resources that prevent runtime shutdown.
5. **SS-04 boundary**: Signal handling code lives in `monocle-runtime` (SS-04 composition
   root scope). Signal wiring must not bleed into `monocle-core` (SS-02) or `monocle-ipc`
   (SS-05).
6. **`e2e-test-affordances` feature gate**: `MONOCLE_HOOK_DELAY_MS` env var reading MUST be
   gated by `#[cfg(feature = "e2e-test-affordances")]`. Production builds must not expose
   this affordance.

## Library and Framework Requirements

Use ONLY the versions pinned in `architecture/SS-deps-pin-manifest.md` v1.2.1. Do NOT
consult training data for version numbers.

| Dependency | Usage | Pin Source |
|------------|-------|------------|
| tokio | `tokio::signal::unix::signal`, `AtomicBool`, `select!`, spawn | SS-deps-pin-manifest.md |
| `std::sync::atomic::{AtomicBool, Ordering}` | Second-signal flags on DaemonState | stdlib |
| nix | Signal constants (SIGTERM=15, SIGINT=2) for test assertions | SS-deps-pin-manifest.md |
| `cargo feature: e2e-test-affordances` | MONOCLE_HOOK_DELAY_MS gate | SS-daemon-wiring-impl.md §HIGH-1 |

**Forbidden dependencies**: This story must NOT introduce any new crate-level dependencies.
All required types (`AtomicBool`, `tokio::signal`, `nix`) are already in the workspace
dependency graph. If a new crate is needed, escalate to architect.

## File Structure Requirements

| File | Action | Change |
|------|--------|--------|
| `crates/monocle-runtime/src/state.rs` | Modify | Add `second_sigterm: AtomicBool` and `second_sigint: AtomicBool` fields; update `DaemonState::new()` |
| `crates/monocle-runtime/src/lifecycle.rs` | Modify | Remove `// CONTRACT GAP (S-DAEMON-WIRE-FIX-001)` markers from `SigtermDuringDrain` and `SigintDuringDrain` doc-comments after implementation |
| `crates/monocle-runtime/src/main.rs` | Modify | Add second-signal listener task; update exit-code selection to read `second_sigterm` / `second_sigint` |
| `crates/monocle-runtime/src/server.rs` | Modify (possibly) | If the two-phase signal loop lives in `run_server`, add second-signal branch; otherwise leave unchanged |
| `crates/monocle-runtime/tests/daemon_e2e_serve.rs` | Modify | Add AC-005 and AC-006 E2E tests for second-signal exit codes (or create `tests/second_signal_exit_codes.rs`) |

**Files the implementer must NOT change:**
- `crates/monocle-runtime/src/lock.rs` — lock lifecycle unchanged.
- `crates/monocle-runtime/src/event_bus.rs` — event bus unchanged by this story.
- Any TUI crates — signal handling is daemon-only.

## Token Budget Estimate

| Context Component | Estimated Tokens |
|-------------------|-----------------|
| This story spec | ~3,500 |
| BC-2.01.004.md (full) | ~4,000 |
| SS-daemon-wiring-impl.md (full — §HIGH-2 + surrounding context) | ~12,000 |
| SS-daemon-wiring.md | ~4,800 |
| state.rs (current DaemonState struct) | ~2,500 |
| lifecycle.rs (DaemonExit enum + variant doc-comments) | ~2,000 |
| main.rs (current exit-code selection and signal wiring) | ~1,500 |
| server.rs (run_server signal select loop) | ~1,500 |
| daemon_e2e_serve.rs (existing E2E tests for reference) | ~3,000 |
| SS-deps-pin-manifest.md (relevant dependency section) | ~1,500 |
| **Total estimated context** | **~36,300 tokens** |

Estimate: ~36k tokens. Well within 20-30% of a 200k-token agent context window. No split
required.

## Verification Properties

VP-004 (Graceful Shutdown — 10-Second Drain) covers this story's second-signal exit-code
assertions. The existing VP-004 integration test file (`monocle-runtime/tests/graceful_shutdown.rs`)
is the anchor; new second-signal tests extend VP-004's coverage to the second-signal paths.

| VP-NNN | Property | Proof Method | Anchor Story |
|--------|----------|-------------|-------------|
| VP-004 | Second SIGTERM during drain → exit 143; second SIGINT during drain → exit 130; first-signal graceful → exit 0 | E2E subprocess integration (AC-005, AC-006, AC-003) | S-DAEMON-WIRE-FIX-001 |
---

_Subsystem anchor justification:_ SS-01 owns this story because BC-2.01.004 (Graceful
Shutdown) is a SS-01 lifecycle contract and `DaemonExit` variants are defined in `monocle-runtime`
(the SS-01 implementing module). SS-04 co-owns because the composition-root wiring in
`main.rs` (SS-04 scope) is where the second-signal loop and exit-code selection must live,
per SS-daemon-wiring-impl.md §HIGH-2 design seam.

_Dependency anchor justification:_ S-DAEMON-WIRE-FIX-001 depends on S-016 (daemon CLI init,
provides the binary crate structure), S-017 (daemon start sequence, provides `daemon_start_sequence`
return type and DaemonState), S-018 (bounded event bus, provides DaemonState.event_bus_tx
field basis — DaemonState struct must be in its final shape before adding AtomicBool fields),
and S-005 (graceful shutdown library — the library-level `DaemonExit` enum and `exit_with`
function must exist before this story wires them into the second-signal path). All four
depend-on stories are done (Wave 2-5, Phase-3 complete).
