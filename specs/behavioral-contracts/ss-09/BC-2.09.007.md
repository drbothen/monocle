---
document_type: behavioral-contract
level: L3
version: "1.1.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "32bbb0a"
traces_to: prd.md
origin: greenfield
subsystem: SS-09
capability: CAP-009
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

# Behavioral Contract BC-2.09.007: Scrollback — 1000 Rows Default; Configurable; PtyScrollUp/Down Navigate

## Description

`vt100::Parser` is initialized with a scrollback buffer of 1000 rows by default. In
`AppMode::EmbeddedTerminal`, `PtyScrollUp` and `PtyScrollDown` actions adjust
`App::pty_scroll_offsets[focused_session_id]` without sending `ResizePane` IPC messages.
`pty_scroll_offsets` is a `HashMap<String, usize>` keyed by `session_id`; each session's
offset is independent. The TUI passes the per-session scrollback viewport offset to the PTY
widget renderer. Scrollback capacity is configurable via
`~/.monocle/config.json:pty_scrollback_rows`, capped at 10000.

## Preconditions

1. A `vt100::Parser` is initialized for the session via
   `vt100::Parser::new(rows, cols, scrollback_rows)`.
2. `AppMode::EmbeddedTerminal` is active.
3. The session has produced enough output to fill the visible area (scroll is possible).

## Postconditions

1. `vt100::Parser` is initialized with `scrollback_rows` equal to the configured value
   (default 1000). The scrollback buffer stores up to `scrollback_rows` lines of output
   beyond the current visible screen.
2. In `AppMode::EmbeddedTerminal`:
   a. `Action::PtyScrollUp` increments `App::pty_scroll_offsets[focused_session_id]` by one
      scroll step (scroll toward older output).
      `pty_scroll_offsets[focused_session_id]` = number of rows scrolled BACK from the current
      bottom (0 = live tail).
      `PtyScrollUp`: `pty_scroll_offsets[focused_session_id] += scroll_step` (toward older lines).
      `PtyScrollDown`: `pty_scroll_offsets[focused_session_id] -= scroll_step` (toward newer lines; min 0).
   b. `Action::PtyScrollDown` decrements `pty_scroll_offsets[focused_session_id]` toward 0
      (toward current output).
   c. Both actions clamp: `pty_scroll_offsets[focused_session_id]` cannot exceed the number of
      available scrollback rows in the parser, and cannot go below 0.
   d. Each session's offset is independent. Switching focus preserves each session's offset in its
      own `pty_scroll_offsets` entry; focus switch does NOT reset the incoming session's offset.
3. No `ResizePane` or `KeyInput` IPC message is sent for scroll actions — scrollback is a
   TUI-side viewport operation only.
4. When `pty_scroll_offsets[focused_session_id] > 0`, a visual indicator is shown in the status
   bar (`[scrolled back N rows]` or equivalent).
5. New PTY output received while scrolled back does NOT force the viewport to jump to the
   bottom. The user must explicitly `PtyScrollDown` to return to live output.

## Invariants

1. Default `scrollback_rows = 1000`. The configured value is read from
   `~/.monocle/config.json:pty_scrollback_rows`; if missing or invalid, 1000 is used.
2. Maximum `scrollback_rows = 10000`. Values above this cap are silently clamped.
   Memory bound (per SS-embedded-pty.md §O4): the `vt100` crate stores each cell as
   `(char, fg_color, bg_color, attrs_bitmask)` — approximately `1 (char) + 4 (fg color enum) +
   4 (bg color enum) + 1 (attrs bitmask) + padding ≈ 16 bytes/cell` on 64-bit systems.
   `10000 × 80 × ~16 bytes/cell ≈ 12.8 MB per session × 8 sessions ≈ 102 MB`
   — acceptable on a workstation with ≥ 8 GB RAM.
   See BC-2.09.001 Invariant 4 for the same bound with default (1000-row) analysis.
3. `pty_scroll_offsets[session_id]` is reset to 0 when:
   a. A `ResizePane` IPC event fires for that session (resize reflows content; old offset is
      meaningless against new layout; snapping to live tail is least-surprising behavior).
   b. The session transitions to `Terminated` (`pty_scroll_offsets.remove(session_id)` per
      SS-embedded-pty.md §Parser ownership in TUI §Scrollback offset invariants).
4. Scrollback is a TUI-local operation. The session-host's `vt100::Parser` (which owns the
   PTY master side) is independent. Scrollback in the TUI does not affect the harness child.
5. `pty_scroll_offsets` is a `HashMap<String, usize>` keyed by `session_id` (NOT a singular
   shared field). This is the I7 fix per SS-embedded-pty.md §Parser ownership in TUI: a
   shared single offset caused focus-switch to show the wrong session's scrollback position.
   Per-session offsets are initialized to 0 when a session is added to `pty_parsers`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-240 | Scroll up past beginning of scrollback buffer | `pty_scroll_offsets[focused_session_id]` clamped to max available rows; no error |
| EC-241 | Scroll down when already at bottom (offset = 0) | `pty_scroll_offsets[focused_session_id]` stays at 0; no error |
| EC-242 | `pty_scrollback_rows: 20000` in config | Clamped to 10000 at parser initialization |
| EC-243 | `pty_scrollback_rows: 0` in config | Clamped to 1 (minimum 1-row scrollback; 0 would mean no scrollback which is confusing) |
| EC-244 | New output arrives while scrolled back | Parser updates; viewport stays scrolled; user sees `[scrolled back N rows]` indicator |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 1100 lines of output; PtyScrollUp × 10 (session "s1" focused) | `pty_scroll_offsets["s1"] = 10`; rows 1090-1100 visible (scrolled 10 rows back); `pty_scroll_offsets` for other sessions unchanged | happy-path |
| PtyScrollDown when `pty_scroll_offsets[focused_session_id] = 0` | Offset stays 0; no error | edge-case |
| Focus switch from "s1" (offset=10) to "s2" (offset=0) | `pty_scroll_offsets["s1"] = 10` preserved; `pty_scroll_offsets["s2"] = 0`; render uses `pty_scroll_offsets["s2"]` for new focused session | happy-path |
| Config `pty_scrollback_rows: 500` | Parser initialized with `scrollback_rows = 500` | happy-path |
| Config `pty_scrollback_rows: 15000` | Parser initialized with `scrollback_rows = 10000` (clamped) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `vt100::Parser` initialized with configured scrollback_rows | unit |
| VP-TBD | `PtyScrollUp/Down` adjusts `pty_scroll_offsets[focused_session_id]` and clamps correctly | unit |
| VP-TBD | Focus switch preserves per-session scroll offsets (I7: no cross-session contamination) | unit |
| VP-TBD | No IPC message sent for scroll actions | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — scrollback is part of the embedded PTY widget capability; it enables users to review previous output without leaving EmbeddedTerminal mode |
| Architecture Module | monocle-tui (`App::pty_scroll_offsets`, `pty_parsers`, PtyScrollUp/Down action handlers) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md v1.7.0 §Scrollback navigation; §Parser ownership in TUI |
| Test Name | test_BC_2_09_007_scrollback_1000_default_configurable |

## Related BCs

- [BC-2.09.001] — composes with: scrollback viewport offset affects which parser rows are rendered

## Architecture Anchors

- `architecture/SS-embedded-pty.md#scrollback-navigation` — offset semantics, default/max, no-IPC-send rule

## Story Anchor

S-043 — Implement scrollback navigation in monocle-tui

## VP Anchors

VP-TBD — Scrollback offset unit tests (filled after VP creation)

## §Trace v1.1.2

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-043** (2026-06-15):
- Story Anchor filled from Phase-2 Burst B story decomposition. No behavioral content changed.

## §Trace v1.1.1

**Arch-source pin v1.5.1→v1.5.2** (2026-06-13 / D-277):
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.1.0

**I22-001 + I22-003 — Per-session HashMap scroll offsets (I7 fix propagation) + ~16 bytes/cell memory bound** (2026-06-13):
- I22-001 (Phase-1d Pass 22 IMPORTANT): The entire normative body used `App::pty_scroll_offset: usize`,
  a retired singular field that caused focus-switch to show the wrong session's scrollback position.
  The correct field is `pty_scroll_offsets: HashMap<String, usize>` keyed by `session_id`.
  SS-embedded-pty.md v1.5.0 (canonical reference, read-only) had already specified the per-session
  HashMap form; BC-2.09.001 Invariant 6 explicitly states "not a shared pty_scroll_offset field;
  per-session HashMap". This BC was authored before those canonical references were final and
  retained the stale singular form.
  - Description: `App::pty_scroll_offset` → `App::pty_scroll_offsets[focused_session_id]`;
    added per-session semantics paragraph and HashMap type declaration.
  - PC-2a/b/c: all `pty_scroll_offset` → `pty_scroll_offsets[focused_session_id]`; added PC-2d
    for focus-switch offset preservation (the bug that I7 fixed).
  - PC-4: `pty_scroll_offset > 0` → `pty_scroll_offsets[focused_session_id] > 0`.
  - Invariant 3: renamed from `pty_scroll_offset` to `pty_scroll_offsets[session_id]`; reset
    condition changed from "user exits EmbeddedTerminal" to ResizePane-per-session (per
    SS-embedded-pty.md §Scrollback offset invariants) and Terminated (with remove() call).
  - Invariant 5 (new): explicitly names the HashMap type and the I7 semantic fix, consistent
    with BC-2.09.001 Invariant 6.
  - EC-240/241: `pty_scroll_offset` → `pty_scroll_offsets[focused_session_id]`.
  - Canonical Test Vectors: `pty_scroll_offset = 10` → `pty_scroll_offsets["s1"] = 10`;
    added cross-session focus-switch vector.
  - VP table: updated property description; added per-session isolation VP.
  - Architecture Module: `App::pty_scroll_offset` → `App::pty_scroll_offsets`.
- I22-003 (Phase-1d Pass 22 IMPORTANT): Invariant 2 stated `10000 × 80 × ~4 bytes/cell ≈ 3.2 MB
  per session × 8 sessions ≈ 25 MB`. This figure severely underestimates real memory use.
  Per SS-embedded-pty.md §O4 (canonical source): the `vt100` crate Cell struct stores
  `(char, fg_color, bg_color, attrs_bitmask)` — approximately 16 bytes/cell on 64-bit systems.
  Updated to: `10000 × 80 × ~16 bytes/cell ≈ 12.8 MB per session × 8 sessions ≈ 102 MB`,
  with §O4 rationale quoted inline. This matches BC-2.09.001 Invariant 4 exactly.
- Version bump: 1.0.0 → 1.1.0 (minor: materially changed Invariants 2/3/5, PC-2, VP table;
  addition of per-session semantics is a normative behavioral specification enhancement).

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.007 authored for SS-09 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): Clarified PtyScrollUp increases offset (scrolls toward older lines),
  PtyScrollDown decreases toward 0 (newer lines). The description clarification is production-grade
  — the offset direction must be unambiguous for implementers.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
