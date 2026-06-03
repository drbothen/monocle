---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "2d6731a"
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
`App::pty_scroll_offset` without sending `ResizePane` IPC messages. The TUI passes the
viewport offset to the PTY widget renderer. Scrollback capacity is configurable via
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
   a. `Action::PtyScrollUp` decrements `App::pty_scroll_offset` by 1 (scroll toward older
      output). Minimum offset is 0 (no scroll; bottom of buffer shown).
      Wait — scroll up shows older lines, which means offset increases. Clarification:
      `pty_scroll_offset` = number of rows scrolled BACK from the current bottom.
      `PtyScrollUp`: `pty_scroll_offset += scroll_step` (scroll toward older lines).
      `PtyScrollDown`: `pty_scroll_offset -= scroll_step` (scroll toward newer lines; min 0).
   b. `Action::PtyScrollDown` increments toward 0 (toward current output).
   c. Both actions clamp: `pty_scroll_offset` cannot exceed the number of available scrollback
      rows in the parser, and cannot go below 0.
3. No `ResizePane` or `KeyInput` IPC message is sent for scroll actions — scrollback is a
   TUI-side viewport operation only.
4. When `pty_scroll_offset > 0`, a visual indicator is shown in the status bar
   (`[scrolled back N rows]` or equivalent).
5. New PTY output received while scrolled back does NOT force the viewport to jump to the
   bottom. The user must explicitly `PtyScrollDown` to return to live output.

## Invariants

1. Default `scrollback_rows = 1000`. The configured value is read from
   `~/.monocle/config.json:pty_scrollback_rows`; if missing or invalid, 1000 is used.
2. Maximum `scrollback_rows = 10000`. Values above this cap are silently clamped.
   Memory bound: `10000 × 80 × ~4 bytes/cell ≈ 3.2 MB per session × 8 sessions ≈ 25 MB`
   — acceptable for typical workloads.
3. `pty_scroll_offset` is reset to 0 when:
   a. The user exits `AppMode::EmbeddedTerminal` (resets on next entry).
   b. The session is killed or terminates.
4. Scrollback is a TUI-local operation. The session-host's `vt100::Parser` (which owns the
   PTY master side) is independent. Scrollback in the TUI does not affect the harness child.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-240 | Scroll up past beginning of scrollback buffer | `pty_scroll_offset` clamped to max available rows; no error |
| EC-241 | Scroll down when already at bottom (offset = 0) | `pty_scroll_offset` stays at 0; no error |
| EC-242 | `pty_scrollback_rows: 20000` in config | Clamped to 10000 at parser initialization |
| EC-243 | `pty_scrollback_rows: 0` in config | Clamped to 1 (minimum 1-row scrollback; 0 would mean no scrollback which is confusing) |
| EC-244 | New output arrives while scrolled back | Parser updates; viewport stays scrolled; user sees `[scrolled back N rows]` indicator |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 1100 lines of output; PtyScrollUp × 10 | `pty_scroll_offset = 10`; rows 1090-1100 visible (scrolled 10 rows back) | happy-path |
| PtyScrollDown when at offset=0 | Offset stays 0; no error | edge-case |
| Config `pty_scrollback_rows: 500` | Parser initialized with `scrollback_rows = 500` | happy-path |
| Config `pty_scrollback_rows: 15000` | Parser initialized with `scrollback_rows = 10000` (clamped) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `vt100::Parser` initialized with configured scrollback_rows | unit |
| VP-TBD | `PtyScrollUp/Down` adjusts `pty_scroll_offset` and clamps correctly | unit |
| VP-TBD | No IPC message sent for scroll actions | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — scrollback is part of the embedded PTY widget capability; it enables users to review previous output without leaving EmbeddedTerminal mode |
| Architecture Module | monocle-tui (`App::pty_scroll_offset`, `pty_parsers`, PtyScrollUp/Down action handlers) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md v1.1.0 §Scrollback navigation; §Parser ownership in TUI |
| Test Name | test_BC_2_09_007_scrollback_1000_default_configurable |

## Related BCs

- [BC-2.09.001] — composes with: scrollback viewport offset affects which parser rows are rendered

## Architecture Anchors

- `architecture/SS-embedded-pty.md#scrollback-navigation` — offset semantics, default/max, no-IPC-send rule

## Story Anchor

S-TBD — Implement scrollback navigation in monocle-tui (filled by story-writer)

## VP Anchors

VP-TBD — Scrollback offset unit tests (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.007 authored for SS-09 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): Clarified PtyScrollUp increases offset (scrolls toward older lines),
  PtyScrollDown decreases toward 0 (newer lines). The description clarification is production-grade
  — the offset direction must be unambiguous for implementers.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
