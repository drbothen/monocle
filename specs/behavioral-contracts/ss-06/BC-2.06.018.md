---
document_type: behavioral-contract
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T18:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "ee4d690"
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

# Behavioral Contract BC-2.06.018: Event Ribbon Panel: Rolling Hook Event Log

## Description

The Event Ribbon panel renders a scrollable, newest-first log of hook events received via
`ServerToClient::HookEventReceived` messages. Each row shows four columns: timestamp
(HH:MM:SS.mmm), hook type (display name), session ID (first 8 chars), and latency-ms. A
fifth `Status` column shows `PENDING` in yellow for unresolved `PreToolUse` events; blank
otherwise. N is determined by the visible panel height — no artificial cap beyond the
screen. The panel scrolls via `Action::ScrollUp` / `Action::ScrollDown`. This is the TUI
rendering contract; it is distinct from the daemon-side JSONL ring buffer (BC-2.01.007)
and the daemon-side bounded event bus (BC-2.04.011).

## Preconditions

1. The TUI is connected to the daemon and receiving `HookEventReceived` IPC messages.
2. The `App` struct holds a `VecDeque<HookEventRow>` (the Event Ribbon's in-memory log).
3. The Event Ribbon panel is visible (AppMode is `Dashboard` with either panel focused, or
   `Filtering { panel: EventRibbon, ... }`). In `Fullscreen` or `Overlay` modes the Event
   Ribbon is not rendered in the main area, but events continue to accumulate in the
   `VecDeque`.
4. The panel layout allocates 40% of terminal width to the Event Ribbon (per SS-tui.md
   §Rendering Architecture §Draw Function Dispatch).

## Postconditions

1. **Column layout per row:**

   | Column | Source | Width |
   |--------|--------|-------|
   | Timestamp | `HookEventRow::received_at` formatted as `HH:MM:SS.mmm` | 12 chars |
   | Hook type | `HookType` display name (e.g., "PreToolUse", "Notification") | 16 chars |
   | Session ID | `HookEventRow::session_id` first 8 characters | 10 chars |
   | Latency | `HookEventRow::latency_ms` formatted as `NNNms` | 8 chars |
   | Status | `PENDING` if unresolved PreToolUse; blank otherwise | 8 chars |

2. **Newest-first ordering:** New events are prepended to the top of the displayed list.
   The most recent event is row 0 (the top row of the panel).

3. **Rolling window:** The `VecDeque<HookEventRow>` holds at most `panel_height` entries
   (determined at render time). When the `VecDeque` is full and a new event arrives, the
   oldest event (back) is popped before the new event is pushed to the front. This is a
   fixed-size sliding window, not an infinite log.

4. **`PENDING` status in yellow:** An unresolved `PreToolUse` event (one for which a
   `PromptModal` has been pushed but no `ClientToServer::PermissionDecision` yet sent) renders `PENDING` in
   the Status column using `ratatui::Style::default().fg(Color::Yellow)`. When the
   `ClientToServer::PermissionDecision` is sent (or the prompt is auto-resolved by timeout), the Status
   column for that row reverts to blank (default color).

5. **Scroll behavior:** When `Action::ScrollUp` is dispatched in `Dashboard` mode with
   focus on `PanelId::EventRibbon`, the visible window scrolls up (revealing older events).
   `Action::ScrollDown` scrolls toward the newest events. The scroll offset is bounded:
   cannot scroll past the first event (top) or below the last event (bottom).

6. **Panel 40% width layout:** In the standard Dashboard layout, the Event Ribbon
   occupies the right 40% of the terminal. This is a render-time constraint; the
   `VecDeque<HookEventRow>` itself is not width-constrained.

7. **Continuous accumulation:** Events continue to be added to the `VecDeque` while the
   TUI is in any AppMode (including Overlay and Fullscreen). The draw loop drains IPC
   messages before rendering; no events are dropped by the TUI due to panel visibility.
   Drop events (if any) are tracked by the daemon-side bounded event bus (BC-2.04.011)
   and surfaced via the drop counter (BC-2.06.019).

## Invariants

1. The Event Ribbon `VecDeque<HookEventRow>` is structurally separate from the daemon's
   JSONL ring buffer (BC-2.01.007) and the daemon's bounded event bus (BC-2.04.011). The
   ribbon holds TUI-side render state only; it is rebuilt from the daemon's IPC pushes.
2. The Event Ribbon is NOT the same as the permission overlay. Hook events appear in the
   ribbon for all hook types. `PermissionPromptQueued` events produce BOTH a ribbon entry
   (showing the hook event) AND a `PromptModal` pushed to the overlay stack.
3. The `panel_height`-based cap is dynamic: terminal resize causes `panel_height` to
   change; the `VecDeque` is trimmed to the new height on the next draw cycle.
4. Session IDs in the ribbon are truncated to 8 characters for display only; the full
   session ID is stored in `HookEventRow::session_id` for future detail views.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-112 | Terminal is 80 columns wide; all columns cannot fit without wrapping | Columns are rendered with fixed widths per Postcondition 1; if terminal is too narrow, rightmost columns are clipped or omitted — no panic; no line wrap that breaks column alignment |
| EC-113 | 1000 events arrive in rapid succession (1000 events/sec load test) | `VecDeque` is bounded to `panel_height`; old events are popped; no memory growth; drop counter in status bar reflects any daemon-side drops (BC-2.04.011); TUI never drops events independently — it only trims the render window |
| EC-114 | No events received since TUI start | Panel renders empty state: blank panel body or "No events yet" placeholder text (implementation choice, but must not panic) |
| EC-115 | A `PreToolUse` event's `PromptModal` is resolved while the user is scrolled down (viewing older events) | The resolved event's Status column changes from `PENDING` (yellow) to blank; the change is visible when the user scrolls back to that event's position |
| EC-116 | User scrolls up past the oldest event | Scroll offset stays at max (clamped to last event index); no crash; no out-of-bounds access on `VecDeque` |
| EC-117 | TUI receives `HookEventReceived` for a session ID not in the current sessions list | Event is still rendered in the ribbon with the truncated session ID; no lookup failure; sessions panel and event ribbon are independent data sources |
| EC-118 | Latency field is `None` (daemon did not measure latency for this event type) | Latency column renders as `—` (em-dash) or blank; not `0ms` (which would be misleading) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 5 `HookEventReceived` events arrive in order (E1 oldest, E5 newest) | Panel renders E5 at top, E1 at bottom | happy-path |
| Panel height is 10; 15 events arrive | First 5 (oldest) are dropped; 10 newest remain in `VecDeque` | edge-case |
| `PreToolUse` event arrives; overlay pushed | Status column shows `PENDING` in yellow for that row | happy-path |
| `ClientToServer::PermissionDecision` sent for the `PreToolUse` event above | Status column for that row reverts to blank | happy-path |
| `ScrollUp` dispatched when at top of ribbon | No change to scroll offset; no error | edge-case |
| `ScrollDown` dispatched at bottom (most recent event visible) | No change; no error | edge-case |
| `Notification` event arrives | Renders in ribbon; NO overlay push; Status blank | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Event Ribbon renders newest event at row 0 | unit test |
| VP-TBD | `VecDeque` is bounded to `panel_height`; no unbounded growth under load | unit test (inject panel_height=10, add 20 events, assert len==10) |
| VP-TBD | `PreToolUse` unresolved events render `PENDING` in yellow | unit test (inspect ratatui `Style` on row) |
| VP-TBD | `PENDING` status reverts to blank after `ClientToServer::PermissionDecision` | unit test |
| VP-TBD | Under 1000 events/sec synthetic load, TUI renders without crash | integration/load test |
| VP-TBD | Scroll offset is clamped — no out-of-bounds panic | unit test (scroll past end) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC directly specifies the "event ribbon" component of CAP-006, the rolling hook event log that is one of the product's three Phase 1 panels |
| L2 Domain Invariants | DI-001 (every hook event received by the daemon MUST be written to the JSONL ring — the TUI Event Ribbon is a render view derived from IPC pushes, not from the ring directly; DI-001 is enforced by the daemon before the IPC push, not by this TUI BC); DI-007 (monocle MUST NOT write to files owned by a harness — satisfied: ribbon is read-only rendering) |
| Architecture Module | monocle-tui (Event Ribbon panel renderer, `VecDeque<HookEventRow>`, scroll state); monocle-core (PanelId::EventRibbon, FocusSnapshot::EventRibbon) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Panel Architecture §Event Ribbon Panel (column layout, scroll, `PENDING` status, panel height cap, newest-first ordering) |
| Cross-Ref | BC-2.01.007 (JSONL ring buffer — daemon-side storage; DISTINCT from this TUI render state); BC-2.04.011 (bounded event bus — daemon-side drop counter; TUI ribbon is downstream of this); BC-2.05.004 (IPC HookEventReceived — the IPC message type this BC processes); BC-2.06.019 (drop counter in status bar — shows daemon-side drops visible alongside ribbon); BC-2.06.017 (Notification hooks: appear in ribbon but never defer) |
| Test File | `monocle-tui/tests/event_ribbon_panel.rs` |
| Test Name | `test_BC_2_06_018_event_ribbon_rolling_log` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.004] — depends on: IPC `HookEventReceived` message is the data source for the ribbon
- [BC-2.01.007] — DISTINCTION: JSONL ring is the daemon-side persistent log; this ribbon is TUI-side ephemeral render state
- [BC-2.04.011] — composes with: daemon bounded event bus drop counter informs the status bar (BC-2.06.019); ribbon is the visual complement
- [BC-2.06.017] — composes with: Notification hooks appear in ribbon but never defer; this BC handles the ribbon rendering aspect

## Architecture Anchors

- `architecture/SS-tui.md#event-ribbon-panel` — column layout, scroll behavior, PENDING status, newest-first ordering
- `architecture/SS-tui.md#rendering-architecture` — dashboard layout (40% Event Ribbon)

## Story Anchor

S-TBD — Implement Event Ribbon panel: column layout, newest-first prepend, bounded VecDeque, PENDING status, scroll (filled by story-writer)

## VP Anchors

- VP-TBD — Load test: 1000 events/sec; VecDeque bounded; no memory growth; no panic

## §Trace v1.0.0

**Initial production** (2026-05-26T18:00:00Z):
- BC-2.06.018 created as part of SS-06 TUI behavioral contract burst (BCs 016–022).
- Reads: SS-tui.md v1.1.0 §Panel Architecture §Event Ribbon Panel (full section);
  §Rendering Architecture §Draw Function Dispatch (40% width allocation);
  prd-expansion-scope.md §3.3 BC-2.06.018 description (F-42, F-43, F-44).
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-001 cited: ring write is upstream; ribbon is a derived view. DI-007 cited: read-only.
- Invariant 2 explicitly separates Event Ribbon (all hook types) from the permission
  overlay (PreToolUse only) — a common source of confusion in lazyclaude reference code.
- EC-113 covers the 1000 events/sec load criterion from product-brief.md §Success Criteria.
- EC-118 covers `None` latency — avoids misleading `0ms` display for unmetered events.
- Postcondition 3 defines the rolling window as panel_height-bounded — no artificial cap,
  but no infinite growth either.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-P1D7-001 HIGH — Fabricated `IpcServerMessage` type name replaced with canonical `ServerToClient`** (2026-05-26T00:00:00Z):
- `IpcServerMessage::HookEventReceived` → `ServerToClient::HookEventReceived`. The canonical
  server-to-client enum is `ServerToClient` per SS-ipc.md §Server-to-Client Messages.
- `DecisionResponse` → `ClientToServer::PermissionDecision` (3 occurrences: Postcondition 4,
  test vector table, VP table).
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.0.5

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): this BC has no `Overlay { stack }` references; it covers Event Ribbon rendering only.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers Event Ribbon panel; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
- SE-16d monotonicity: v1.0.5 timestamp 2026-05-29T00:00:00Z > v1.0.4. PASS.
