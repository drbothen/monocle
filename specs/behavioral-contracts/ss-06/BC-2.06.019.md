---
document_type: behavioral-contract
level: L3
version: "1.1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-01T14:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "c1ef69a"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010, F-P1D7-001, S-027-ADV-DROPS-COEXISTENCE]
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
   renders on the status bar's upper row (the breadcrumb row), right-aligned. The text
   uses `ratatui::Style::default().fg(Color::Yellow)` to indicate a degraded (but
   non-fatal) condition. See PC-7 for the coexistence guarantee: `drops: N` is NEVER
   suppressed by `status_message`.

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

6. **Positioning:** `drops: N` is part of the status bar's upper row (same row as the
   breadcrumb and mode indicator). It is right-aligned or separated from the breadcrumb
   by at least 2 spaces. The text must be visible when the terminal is at minimum width
   (80 columns).

7. **Coexistence with `status_message` — `drops: N` MUST NOT be suppressed:**
   When `App.status_message` is `Some(msg)` AND `drop_counter > 0`, BOTH pieces of
   information MUST be visible simultaneously. The canonical two-slot layout is:

   - **Upper row (breadcrumb row):** `drops: N` (in `Color::Yellow`) is rendered
     right-aligned on the upper row at all times when `drop_counter > 0`. It is NEVER
     displaced or hidden by an active `status_message`.
   - **Lower row (hint row):** `status_message` (in `Color::Yellow`) is rendered on the
     lower row when `Some`, temporarily superseding the keybinding hint line. When
     `status_message` is `None`, the lower row renders the normal keybinding hint per
     BC-2.06.021.

   This layout applies to ALL sources of `status_message`: daemon-disconnect indicators
   (BC-2.06.016), `[t]` trace-to-source stub (BC-2.06.015), and any future transient
   notification. The mutual-exclusion pattern (`if status_message { ... } else { drop_counter }`)
   is FORBIDDEN — `drops: N` is a data-loss signal that MUST remain permanently visible
   when non-zero, regardless of any transient notification.

   Rationale: `drops: N` signals that the daemon's bounded event bus is dropping events
   under load. This is an operational health indicator with production implications. A
   transient notification (which communicates a recent event) must never silently mask an
   ongoing degraded condition. Moving `status_message` to the lower row resolves the
   conflict without losing either piece of information.

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
| EC-123 | TUI terminal is 60 columns wide (below 80-column minimum) | Status bar truncates or compresses breadcrumb to fit; drop counter text `drops: N` is still rendered if counter > 0; it is NOT silently omitted due to width constraints (PC-7 reinforces this: `drops: N` is never suppressed for any reason when > 0) |
| EC-124 | `ServerToClient::DropCounterUpdate` messages arrive at 60fps (every 16ms) | Each `drop_counter` update is rendered on the next draw tick; no message is skipped in rendering (the draw loop drains IPC before drawing) |
| EC-129 | `App.status_message = Some("[disconnected] reconnecting...")` AND `drop_counter == 42` simultaneously | Upper row shows `drops: 42` in yellow (right-aligned); lower row shows `"[disconnected] reconnecting..."` in yellow (superseding keybinding hint). Both are visible. `drops: 42` is NOT hidden. |
| EC-130 | `App.status_message = Some("[t] Trace to source — Phase 2 feature (Static plane)")` AND `drop_counter == 7` simultaneously | Upper row shows `drops: 7` in yellow; lower row shows the `[t]` placeholder in yellow. Both are visible. `drops: 7` is NOT hidden. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ServerToClient::DropCounterUpdate { count: 0 }` | Status bar shows no drop counter text | happy-path |
| `ServerToClient::DropCounterUpdate { count: 1 }` | Status bar shows `drops: 1` in yellow | happy-path |
| `ServerToClient::DropCounterUpdate { count: 42 }` | Status bar shows `drops: 42` in yellow | happy-path |
| `ServerToClient::DropCounterUpdate { count: 0 }` after `count: 42` (daemon restart) | `drops: 42` disappears; status bar shows no counter | edge-case |
| Inject 1000 events/sec for 5 seconds via test harness | `drop_counter > 0`; `drops: N` visible in status bar | performance/load |
| `drop_counter = 42`, `status_message = Some("[disconnected] reconnecting...")` | Upper row: `drops: 42` in yellow; lower row: `"[disconnected] reconnecting..."` in yellow. Both visible simultaneously. | coexistence |
| `drop_counter = 7`, `status_message = Some("[t] Trace to source — Phase 2 feature (Static plane)")` | Upper row: `drops: 7` in yellow; lower row: `[t]` placeholder text in yellow. Both visible simultaneously. | coexistence |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `drop_counter == 0` renders no counter text | unit test |
| VP-TBD | `drop_counter > 0` renders `drops: N` in yellow | unit test |
| VP-TBD | Under 1000 events/sec load, `drop_counter` increments and renders within 5 seconds | load test |
| VP-TBD | Counter reflects daemon's value verbatim (no TUI-side accumulation or smoothing) | unit test |
| VP-TBD | Counter renders on every draw tick while non-zero | unit test (draw 10 frames, assert counter present on all) |
| VP-TBD | When `status_message` is `Some` AND `drop_counter > 0`, both are visible: `drops: N` on upper row, `status_message` on lower row | unit test (render with disconnect message + non-zero drops; assert both present) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the status bar drop counter rendering, which is the TUI-visible component of the "no unbounded channel" Success Criterion directly within the CAP-006 TUI scope |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — satisfied: status bar rendering is read-only display, no file writes); DI-001 (JSONL ring write before acknowledgement — not directly constrained by this BC; the drop counter reflects channel drops, not ring write failures) |
| Architecture Module | monocle-tui (status bar renderer, `App::drop_counter: u64`); monocle-ipc (IPC `ServerToClient::DropCounterUpdate` message type) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Panel Architecture §Status Bar (drop counter subsection: "receives drop_counter: u64... renders as `drops: N` when N > 0; renders nothing when N == 0") |
| Cross-Ref | BC-2.04.011 (bounded event bus + daemon-side drop counter — this BC renders what BC-2.04.011 produces); BC-2.06.020 (breadcrumb — shares the upper status bar row as the drop counter); BC-2.06.016 (disconnect indicator `"[disconnected] reconnecting..."` renders on the lower row — COEXISTS with `drops: N` per PC-7; does NOT suppress it); BC-2.06.015 (`[t]` stub placeholder renders on the lower row — COEXISTS with `drops: N` per PC-7) |
| Test File | `monocle-tui/tests/status_bar.rs` |
| Test Name | `test_BC_2_06_019_drop_counter_renders_under_load` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.011] — depends on: the daemon-side bounded event bus is the source of the drop counter this BC renders
- [BC-2.06.020] — composes with: drop counter shares the breadcrumb (upper) status bar row
- [BC-2.06.016] — composes with: disconnect message renders on the lower row per PC-7 coexistence rule; MUST NOT suppress `drops: N`
- [BC-2.06.015] — composes with: `[t]` stub message renders on the lower row per PC-7 coexistence rule; MUST NOT suppress `drops: N`

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

## §Trace v1.0.5

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): this BC has no `Overlay { stack }` references; it covers status bar drop counter only.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers status bar; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC (drop counter behavior is independent of disconnect style).
- SE-16d monotonicity: v1.0.5 timestamp 2026-05-29T00:00:00Z > v1.0.4. PASS.

## §Trace v1.1.0

**S-027-ADV-DROPS-COEXISTENCE MAJOR — Coexistence postcondition added; fabricated citation corrected** (2026-06-01T14:00:00Z):

Adversarial review of `crates/monocle-tui/src/ui/status_bar.rs` surfaced two defects:

**Defect 1 (MAJOR) — Fabricated citation "BC-2.06.016 PC-4 drop-counter-precedence":**
The implementation's module comment (status_bar.rs ~line 14 and lines 57–59) cited
"BC-2.06.016 PC-4" as the authority for suppressing `drops: N` when `status_message` is
active. BC-2.06.016 PC-4 DOES NOT contain any drop-counter-precedence rule. Its actual
content is: "Status bar renders disconnect indicator: the status bar renders the text
`"[disconnected] reconnecting..."` until the IPC reconnect sequence completes." The
citation was fabricated by the implementer to justify the mutual-exclusion pattern.

**Defect 2 (MAJOR) — `drops: N` silently suppressed when `status_message` is active:**
The implementation used `if let Some(msg) = status_message { msg } else { drop_counter_span() }`
on both the 1-row and 2-row render branches. This makes `drops: N` invisible whenever
`status_message` is `Some` — for example, pressing `[t]` in Overlay mode (BC-2.06.015)
sets `status_message`, silently hiding any active `drops: N`. This violates BC-2.06.019
PC-2 ("when drop_counter > 0, the status bar renders `drops: N`") and EC-123 ("`drops: N`
is NOT silently omitted").

**Resolution — canonical coexistence layout (PC-7 added):**
- `drops: N` is permanently right-aligned on the **upper (breadcrumb) row** when
  `drop_counter > 0`. It is never displaced by any `status_message`.
- `status_message` (all sources: disconnect, `[t]` stub, future notifications) renders
  on the **lower (hint) row**, temporarily superseding the keybinding hint line. When
  `status_message` is `None`, the lower row renders the normal keybinding hint per
  BC-2.06.021.
- The mutual-exclusion pattern is FORBIDDEN in the implementation.

**Changes in this version:**
- PC-6 positioning language updated: "second row" → "upper row" (clarifies relation to
  the two-row layout).
- PC-7 added: explicit coexistence postcondition with canonical upper/lower row layout,
  rationale, and FORBIDDEN pattern statement.
- EC-123 updated: cross-references PC-7 as additional reinforcement.
- EC-129 added: coexistence scenario — disconnect + drop_counter both active.
- EC-130 added: coexistence scenario — `[t]` stub + drop_counter both active.
- Canonical test vectors: 2 coexistence rows added.
- Verification Properties: coexistence VP added.
- Traceability Cross-Ref: BC-2.06.016 and BC-2.06.015 added with coexistence relationship.
- Related BCs: BC-2.06.016 and BC-2.06.015 added.
- BC-2.06.016 receives a one-line cross-reference to the coexistence rule (PC-4 prose
  already correctly describes WHAT the disconnect message says; a COEXISTENCE note is added
  to clarify WHERE it renders — lower row, not upper row alongside `drops: N`).
  Authority for "drops:N never suppressed" lives HERE in BC-2.06.019 PC-7.
- SE-16d monotonicity: v1.1.0 timestamp 2026-06-01T14:00:00Z > v1.0.5. PASS.
