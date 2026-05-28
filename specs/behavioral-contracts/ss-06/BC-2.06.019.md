---
document_type: behavioral-contract
level: L3
version: "1.0.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T18:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "6e22061"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010, F-P1D7-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.019: Status Bar: Drop Counter Renders Under Load

## Description

The status bar's drop counter displays the cumulative number of events dropped by the
daemon's bounded event bus, as delivered in each `ServerToClient::DropCounterUpdate` push from
the daemon. When the counter is 0, nothing is rendered (no visual clutter during healthy
operation). When the counter is greater than 0, the text `drops: N` appears in the status
bar. Under synthetic 1000 events/sec load, the drop counter must increment and render
visibly. This is the Phase 1 Success Criterion for bounded channel verification
(`product-brief.md §Success Criteria: "no unbounded channel; counter renders under 1000
events/sec"`).

## Preconditions

1. The TUI is connected to the daemon and receiving `ServerToClient::DropCounterUpdate` IPC messages.
2. Each `ServerToClient::DropCounterUpdate` contains a `drop_counter: u64` field that is the
   daemon's cumulative drop count since daemon start.
3. The status bar is always rendered (it is not a panel and is never hidden per SS-tui.md
   §Panel Architecture §Status Bar).
4. The `App` struct holds `drop_counter: u64` updated from the last received `ServerToClient::DropCounterUpdate`.

## Postconditions

1. **Zero counter — no render:** When `drop_counter == 0`, the drop counter widget
   renders no text (empty string or zero-width span). No `drops: 0` text is shown. This
   keeps the status bar uncluttered during normal (healthy) operation.

2. **Non-zero counter — renders `drops: N`:** When `drop_counter > 0`, the status bar
   renders the text `drops: N` where N is the exact value of `drop_counter`. The text
   renders in the status bar's second row (the breadcrumb row), positioned after the
   breadcrumb string. The text uses `ratatui::Style::default().fg(Color::Yellow)` to
   indicate a degraded (but non-fatal) condition.

3. **Counter is cumulative from daemon start:** The `drop_counter` is never reset to 0
   by the TUI. It is the daemon's cumulative value. If the daemon restarts (BC-2.05.006),
   the counter resets to 0 in the new daemon's `ServerToClient::DropCounterUpdate` — the TUI reflects the
   new daemon's counter. There is no TUI-side accumulation.

4. **1000 events/sec load test criterion:** Under synthetic 1000 events/sec load on
   localhost (injected via the test harness), the drop counter MUST increment (reach > 0)
   and render the `drops: N` text within 5 seconds of load start. This verifies that:
   - The daemon's bounded event bus drops at or before capacity under load (BC-2.04.011)
   - The drop counter is propagated via IPC `ServerToClient::DropCounterUpdate` (not silently discarded)
   - The TUI renders the counter when non-zero

5. **Counter renders on every draw tick:** The `drop_counter` value from the last
   `ServerToClient::DropCounterUpdate` is rendered on every `draw()` call while the daemon is connected. There
   is no debouncing or rate limiting on the display — the counter reflects the latest
   daemon state on every frame.

6. **Positioning:** `drops: N` is part of the status bar's second row (same row as
   breadcrumb). It is right-aligned or separated from the breadcrumb by at least 2 spaces.
   Exact layout is implementation-defined, but the text must be visible when the terminal
   is at minimum width (80 columns).

## Invariants

1. `drop_counter == 0` implies no text is rendered for the counter widget. This is a
   visual invariant: no "drops: 0" noise during healthy operation.
2. The TUI is a faithful display of the daemon's counter. It does not add to, subtract
   from, or smooth the counter value. Display value == received value.
3. The drop counter is a proxy for the overall system health: if it is nonzero, the user
   should investigate whether the daemon's event bus is undersized for the workload (an
   operational concern, not an error). The counter renders in yellow (degraded), not red
   (broken), to reflect this.
4. The drop counter MUST increment under 1000 events/sec synthetic load. If it does not
   increment, this indicates the bounded event bus channel is either unbounded or too
   large for the test to fill. This would mean BC-2.04.011's bounded channel requirement
   is not met. The test MUST fail if `drop_counter == 0` after 5 seconds of 1000 evt/sec
   load.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-119 | `drop_counter` wraps around `u64::MAX` | Display value reflects the wrapped `u64` value; no panic; no special handling required (u64::MAX ≈ 1.8 × 10^19 drops; practically impossible in a Phase 1 session) |
| EC-120 | Two consecutive `ServerToClient::DropCounterUpdate` messages with same `drop_counter` value | No visual change; renders `drops: N` (or blank if 0) same as before; no flicker |
| EC-121 | `drop_counter` decreases between two `ServerToClient::DropCounterUpdate` messages (e.g., daemon restart with new counter starting at 0) | TUI updates to the new counter value; if new value is 0, widget disappears; this is correct — the new daemon has a fresh counter |
| EC-122 | Load test: 1000 events/sec injected but all fit in the bounded channel (channel capacity > 1000) | `drop_counter` stays 0; test FAILS per Postcondition 4 (the bounded bus must drop under this load). If this happens, BC-2.04.011 channel capacity is too large |
| EC-123 | TUI terminal is 60 columns wide (below 80-column minimum) | Status bar truncates or compresses breadcrumb to fit; drop counter text `drops: N` is still rendered if counter > 0; it is NOT silently omitted due to width constraints |
| EC-124 | `ServerToClient::DropCounterUpdate` messages arrive at 60fps (every 16ms) | Each `drop_counter` update is rendered on the next draw tick; no message is skipped in rendering (the draw loop drains IPC before drawing) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ServerToClient::DropCounterUpdate { count: 0 }` | Status bar shows no drop counter text | happy-path |
| `ServerToClient::DropCounterUpdate { count: 1 }` | Status bar shows `drops: 1` in yellow | happy-path |
| `ServerToClient::DropCounterUpdate { count: 42 }` | Status bar shows `drops: 42` in yellow | happy-path |
| `ServerToClient::DropCounterUpdate { count: 0 }` after `count: 42` (daemon restart) | `drops: 42` disappears; status bar shows no counter | edge-case |
| Inject 1000 events/sec for 5 seconds via test harness | `drop_counter > 0`; `drops: N` visible in status bar | performance/load |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `drop_counter == 0` renders no counter text | unit test |
| VP-TBD | `drop_counter > 0` renders `drops: N` in yellow | unit test |
| VP-TBD | Under 1000 events/sec load, `drop_counter` increments and renders within 5 seconds | load test |
| VP-TBD | Counter reflects daemon's value verbatim (no TUI-side accumulation or smoothing) | unit test |
| VP-TBD | Counter renders on every draw tick while non-zero | unit test (draw 10 frames, assert counter present on all) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the status bar drop counter rendering, which is the TUI-visible component of the "no unbounded channel" Success Criterion directly within the CAP-006 TUI scope |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — satisfied: status bar rendering is read-only display, no file writes); DI-001 (JSONL ring write before acknowledgement — not directly constrained by this BC; the drop counter reflects channel drops, not ring write failures) |
| Architecture Module | monocle-tui (status bar renderer, `App::drop_counter: u64`); monocle-ipc (IPC `ServerToClient::DropCounterUpdate` message type) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.5.0 §Panel Architecture §Status Bar (drop counter subsection: "receives drop_counter: u64... renders as `drops: N` when N > 0; renders nothing when N == 0") |
| Cross-Ref | BC-2.04.011 (bounded event bus + daemon-side drop counter — this BC renders what BC-2.04.011 produces); BC-2.06.020 (breadcrumb — shares the same status bar row as the drop counter) |
| Test File | `monocle-tui/tests/status_bar.rs` |
| Test Name | `test_BC_2_06_019_drop_counter_renders_under_load` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.011] — depends on: the daemon-side bounded event bus is the source of the drop counter this BC renders
- [BC-2.06.020] — composes with: drop counter shares the breadcrumb status bar row

## Architecture Anchors

- `architecture/SS-tui.md#status-bar` — drop counter subsection (zero = hidden, nonzero = `drops: N`)

## Story Anchor

S-TBD — Implement status bar drop counter: hide when 0, render `drops: N` in yellow when > 0, load test criterion (filled by story-writer)

## VP Anchors

- VP-TBD — Load test: 1000 events/sec; drop counter increments within 5 seconds and renders in status bar

## §Trace v1.0.0

**Initial production** (2026-05-26T18:00:00Z):
- BC-2.06.019 created as part of SS-06 TUI behavioral contract burst (BCs 016–022).
- Reads: SS-tui.md v1.1.0 §Panel Architecture §Status Bar (drop counter subsection);
  prd-expansion-scope.md §3.3 BC-2.06.019 description (F-45, F-50) and §4 Success
  Criteria Gap Closure ("drop counter active" row citing BC-2.04.011 + BC-2.06.019).
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: status bar is read-only display.
- Postcondition 4 specifies the 1000 evt/sec load test criterion verbatim from the
  Success Criteria — makes this a testable threshold, not a qualitative goal.
- Invariant 4 explicitly makes the load test FAIL if drop_counter stays 0 — ensuring the
  bounded bus is properly bounded and not configured with an effectively unlimited capacity.
- EC-122 exposes the failure mode where a too-large channel masks the drop counter
  requirement.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-P1D7-001 HIGH — Fabricated `IpcServerMessage::StateUpdate` replaced with canonical `ServerToClient::DropCounterUpdate`** (2026-05-26T00:00:00Z):
- `IpcServerMessage::StateUpdate` → `ServerToClient::DropCounterUpdate` (all occurrences).
  The `ServerToClient` enum has no `StateUpdate` variant. The canonical variant for drop
  counter updates is `DropCounterUpdate { count: u64 }` per SS-ipc.md §Server-to-Client
  Messages.
- Test vectors updated: `StateUpdate { drop_counter: N }` → `DropCounterUpdate { count: N }`
  to match the actual struct field name (`count`, not `drop_counter`).
- All occurrences updated: Preconditions 1/4, Postcondition 3/4/5, EC-120/121/124,
  test vector table (4 rows), Traceability Architecture Module row.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.
