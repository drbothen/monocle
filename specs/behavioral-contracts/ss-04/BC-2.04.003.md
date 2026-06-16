---
document_type: behavioral-contract
level: L3
version: "1.5.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:02:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "32ab659"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D-001, F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.003: MONOCLE_NO_AUTOSTART=1 Suppresses Auto-Start

## Description

The `MONOCLE_NO_AUTOSTART` environment variable is an escape hatch for CI environments and
power users who manage the daemon lifecycle externally. When the variable is set to any
non-empty string, the entire daemon auto-start logic (BC-2.04.002) is skipped. The TUI
launches immediately in "daemon offline" mode without reading the lock file, without starting
a daemon subprocess, and without connecting to a UDS socket. The status bar renders a
`[daemon: offline]` indicator. This check is the FIRST action in TUI mode — it precedes all
other daemon state checks.

## Preconditions

1. `monocle` is invoked without any subcommand (TUI mode).
2. `MONOCLE_NO_AUTOSTART` is set in the environment to a non-empty string value.
   The canonical value is `1` but any non-empty string triggers suppression.

## Postconditions

PC-1. The environment variable `MONOCLE_NO_AUTOSTART` is read before any lock file check,
      any PID liveness check, or any daemon subprocess creation.
PC-2. If `MONOCLE_NO_AUTOSTART` is non-empty, the entire auto-start decision sequence
      (BC-2.04.002 PC-1 through PC-5) is skipped entirely.
PC-3. No daemon process is started. No `monocle daemon start` invocation occurs.
PC-4. No lock file is read (the lock file path is not accessed; no attempt is made to
      parse the PID or port from it).
PC-5. No UDS socket connection is attempted.
PC-6. The TUI launches and renders with "daemon offline" mode active. The status bar displays
      the indicator `[daemon: offline]`.
PC-7. The TUI remains functional in offline mode: it renders the sessions panel (empty, no
      sessions), the event ribbon (empty), and the status bar with the offline indicator.
      Permission overlay dispatch is unavailable (no daemon to receive decisions).
PC-8. No error messages, no warnings, and no exit codes other than 0 are produced as a result
      of the suppression. The suppression is a normal operating mode, not an error condition.

## Invariants

1. **Empty string treated as unset.** If `MONOCLE_NO_AUTOSTART` is set to the empty string
   (e.g., via `export MONOCLE_NO_AUTOSTART=$UNDEFINED_VAR`), it is treated as unset and
   auto-start proceeds normally per BC-2.04.002. Only a non-empty value suppresses auto-start.
2. **Suppression is total.** There is no partial auto-start when `MONOCLE_NO_AUTOSTART` is
   set. The escape hatch is binary: either auto-start runs fully (BC-2.04.002) or it is
   skipped entirely (this BC).
3. **TUI must render in offline mode.** The TUI process does not exit when auto-start is
   suppressed; it renders with degraded but functional state.
4. **Lock file state is irrelevant.** Whether a live daemon is running externally is not
   checked. The TUI renders offline regardless of actual daemon state when
   `MONOCLE_NO_AUTOSTART` is set. A power user who manages the daemon externally and also
   sets `MONOCLE_NO_AUTOSTART` must connect manually or via a different invocation path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-2.04.003-01 | `MONOCLE_NO_AUTOSTART=1` (canonical value) | Auto-start suppressed; TUI renders `[daemon: offline]` |
| EC-2.04.003-02 | `MONOCLE_NO_AUTOSTART=true` | Non-empty string; auto-start suppressed; same behavior as EC-01 |
| EC-2.04.003-03 | `MONOCLE_NO_AUTOSTART=yes` | Non-empty string; auto-start suppressed |
| EC-2.04.003-04 | `MONOCLE_NO_AUTOSTART=0` | Non-empty string (the value "0" is not empty); auto-start suppressed. Users who want to enable auto-start must unset the variable entirely, not set it to "0". |
| EC-2.04.003-05 | `MONOCLE_NO_AUTOSTART=` (empty string) | Treated as unset; auto-start proceeds normally per BC-2.04.002 |
| EC-2.04.003-06 | `MONOCLE_NO_AUTOSTART` unset | Auto-start proceeds normally per BC-2.04.002 |
| EC-2.04.003-07 | `MONOCLE_NO_AUTOSTART=1` and `monocle daemon start` subcommand | Suppression only applies to TUI mode (no-subcommand invocation). `monocle daemon start` is NOT affected by `MONOCLE_NO_AUTOSTART`. |
| EC-2.04.003-08 | `MONOCLE_NO_AUTOSTART=1` and `monocle daemon stop` subcommand | Same as EC-07: `daemon stop` is NOT affected by `MONOCLE_NO_AUTOSTART`. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `MONOCLE_NO_AUTOSTART=1 monocle` | TUI renders with `[daemon: offline]`; no daemon process started; exit 0 when TUI exits | happy-path |
| `MONOCLE_NO_AUTOSTART=0 monocle` | Auto-start suppressed (non-empty value); TUI renders `[daemon: offline]` | edge-case |
| `MONOCLE_NO_AUTOSTART= monocle` (empty string) | Auto-start executes normally | edge-case |
| `MONOCLE_NO_AUTOSTART=1 monocle daemon start` | Daemon start executes normally (env var does not affect daemon subcommand) | edge-case |
| `unset MONOCLE_NO_AUTOSTART; monocle` | Auto-start executes normally | baseline |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | No daemon process is started when `MONOCLE_NO_AUTOSTART` is non-empty | integration |
| VP-TBD | TUI renders `[daemon: offline]` indicator in status bar | integration |
| VP-TBD | Empty `MONOCLE_NO_AUTOSTART` does not suppress auto-start | integration |
| VP-TBD | `MONOCLE_NO_AUTOSTART=0` suppresses auto-start (non-empty string semantics) | integration |
| VP-TBD | `monocle daemon start/stop` subcommands are unaffected by the env var | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — `MONOCLE_NO_AUTOSTART` is a CLI surface concern that gates the daemon auto-start path; "CLI surface" and "daemon auto-start" are both named CAP-004 responsibilities; this BC specifies the env-var escape hatch that modifies auto-start behavior for CI and power users |
| L2 Domain Invariants | DI-002 (lock file must be present before hook endpoints accept connections — this BC deliberately allows TUI to start without verifying DI-002, which is correct: when `MONOCLE_NO_AUTOSTART` is set, the daemon may be externally managed and TUI is offline-only; the DI-002 invariant governs daemon-side behavior, not TUI-offline behavior) |
| Architecture Module | `monocle` binary crate + `monocle-runtime` per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §Daemon Auto-Start Logic §MONOCLE_NO_AUTOSTART Check (BC-2.04.003) |
| Cross-Ref | BC-2.04.002 (daemon auto-start — this BC is the precondition gate that prevents BC-2.04.002 from executing); BC-2.04.004 (daemon start subcommand — unaffected by this env var); BC-2.04.005 (daemon stop subcommand — unaffected) |
| Test File | `monocle/tests/no_autostart_env.rs` |
| Test Name | `test_BC_2_04_003_no_autostart_suppresses_daemon` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.002] — this BC is a gate that prevents BC-2.04.002 from executing when `MONOCLE_NO_AUTOSTART` is non-empty
- [BC-2.04.004] — related: `monocle daemon start` is not affected by this env var (EC-2.04.003-07)
- [BC-2.04.005] — related: `monocle daemon stop` is not affected by this env var (EC-2.04.003-08)

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#monocle_no_autostart-check-bc-2.04.003` — env var check, offline mode behavior
- `architecture/SS-daemon-wiring.md#daemon-auto-start-logic` — broader auto-start context

## Story Anchor

S-TBD — Implement MONOCLE_NO_AUTOSTART env var gate with offline-mode TUI rendering (filled by story-writer)

## VP Anchors

VP-TBD — MONOCLE_NO_AUTOSTART integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T12:02:00Z):
- BC-2.04.003 created as new artifact for SS-04 per prd-expansion-scope.md §3.1 and
  SS-daemon-wiring.md §Daemon Auto-Start Logic §MONOCLE_NO_AUTOSTART Check.
- Covers: non-empty string semantics, empty string treated as unset, subcommand exclusion,
  8 edge cases (including EC-04 for `MONOCLE_NO_AUTOSTART=0`), 5 test vectors, 5 VPs.
- EC-2.04.003-04 documents the non-obvious `MONOCLE_NO_AUTOSTART=0` behavior to prevent
  developer surprise. This is the correct POSIX-env-var interpretation.
- input-hash: [pending] — to be populated by compute-input-hash after human review.
- SE-16d PASS: 2026-05-26T12:02:00Z > prior 2026-05-26T12:01:00Z (BC-2.04.002).

## §Trace v1.1.0

**F-P1D-001 CRITICAL — capability mis-anchor corrected** (2026-05-26T00:00:00Z):
- Frontmatter `capability: CAP-001` → `capability: CAP-004` per F-P1D-001.
- Traceability §L2 Capability and §Capability Anchor Justification updated to cite CAP-004
  ("Daemon binary crate wiring; CLI surface; SOQ-2 start-sequence invariant; hook endpoint
  routing; bounded event bus") per ARCH-INDEX §SS-04 Capability Traceability.
- SE-16d monotonicity: v1.1.0 timestamp >= v1.0.0. PASS.

## §Trace v1.2.0

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.0.0` → `SS-daemon-wiring.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.4.0

**F-P1D10-002 HIGH — CAP-004 capability text corrected to ARCH-INDEX verbatim** (2026-05-26T00:00:00Z):
- L2 Capability and Capability Anchor Justification: stale text → ARCH-INDEX verbatim
  `"Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation"`.
- SE-16d monotonicity: v1.4.0 timestamp >= v1.3.0. PASS.

## §Trace v1.3.0

**F-P1D4-003 LOW — Architecture Source pin updated from v1.1.0 to v1.2.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.1.0` → `SS-daemon-wiring.md v1.2.0` per F-P1D4-003 bulk update.
- SE-16d monotonicity: v1.3.0 timestamp >= v1.2.0. PASS.

## §Trace v1.5.0

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-daemon-wiring.md v1.2.0 → v1.3.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §Daemon Auto-Start Logic §MONOCLE_NO_AUTOSTART Check (BC-2.04.003)` → `SS-daemon-wiring.md v1.3.0 §Daemon Auto-Start Logic §MONOCLE_NO_AUTOSTART Check (BC-2.04.003)`.
- Plain version-pin refresh. No substantive content propagation required — §MONOCLE_NO_AUTOSTART Check section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.5.0 timestamp >= v1.4.0. PASS.
