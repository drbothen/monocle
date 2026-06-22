---
document_type: behavioral-contract
level: L3
version: "1.4.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "3e74bba"
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
   `vt100::Parser::new(rows, cols, scrollback_rows)`. For parsers created on session arrival
   (before attach), `rows` = `PTY_DEFAULT_ROWS` (24) and `cols` = `PTY_DEFAULT_COLS` (80)
   per SS-embedded-pty.md §Parser initialization (F-S039-P2-004 ruling). These placeholder
   dims are replaced by real PTY dims on first attach via `ScrollbackDumpComplete`.
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
   bar (`[scrolled back N rows]` or equivalent). This indicator is **persistent viewport state**
   and is rendered concurrently with all other status bar badges. Specifically:
   - The `[scrolled back N rows]` indicator is NOT suppressed by any transient diagnostic badge
     (`[dump: N drops]`, `[N pending permission(s)]`, `[reconnecting...]`, or similar).
   - When the user is scrolled back AND a transient warning is active, BOTH are rendered in the
     status bar simultaneously. The status bar accommodates multiple concurrent badges.
   - Rationale: the scrollback indicator reflects **persistent viewport state** that the user must
     always be able to see (they may not realize they are scrolled back). Transient diagnostics do
     not supersede it. Suppression would cause the user to believe they are at live tail when they
     are not — a silent correctness failure.
5. New PTY output received while scrolled back does NOT force the viewport to jump to the
   bottom. The `pty_scroll_offsets[focused_session_id]` value is **content-anchored**: when
   new output arrives, the offset is incremented by the number of new rows processed so that
   the viewport stays pinned to the same content rows the user is currently viewing.
   Specifically:
   - `on_pty_output(session_id, bytes)` when `pty_scroll_offsets[session_id] > 0`:
     call `parser.process(&bytes)`, then add the number of new scrollback rows generated
     by that process call to `pty_scroll_offsets[session_id]`, then clamp the result to
     `min(parser.screen().scrollback_len(), new_offset)` (upper-bound clamp; no negative
     clamping needed because new rows only increase the offset).
   - When `pty_scroll_offsets[session_id] == 0` (live tail): `process(&bytes)` is called
     normally; the offset stays at 0 (no adjustment). Live tail is never disturbed.
   - The user must explicitly `PtyScrollDown` to return to live output from a scrolled-back
     position.
   - Rationale: vt100 `set_scrollback(N)` is bottom-relative; a static N causes the viewport
     to drift toward newer content as lines arrive. Content-anchored semantics match the
     behavior of all mainstream terminal emulators (iTerm2, tmux, kitty, wezterm, Alacritty)
     and is the expected UX for a production-grade TUI.

## Invariants

1. Default `scrollback_rows = 1000`. The configured value is read from
   `~/.monocle/config.json:pty_scrollback_rows`. Two distinct cases apply:
   - **Absent (key missing, or config falls back to default):** `scrollback_rows = 1000`.
     `pty_scrollback_rows` is typed `Option<u32>`; a `None` deserialize result → 1000 default.
   - **Present (key exists with a parseable u32 value):** the value is clamped to [1, 10000].
     Out-of-range values are NOT defaulted to 1000 — they are clamped (see EC-242, EC-243).
     E.g. `0 → 1` (clamped to minimum; per EC-243); `20000 → 10000` (clamped to maximum;
     per EC-242). A present non-integer cannot occur — serde would fail the whole config load,
     resulting in default config (`None`) → 1000.
   The phrase "missing or invalid → 1000" does NOT apply to out-of-range values; it applies
   only to absent/unparseable config. See BC-2.09.001 Invariant 4 for the cross-reference.
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
| EC-244 | New output arrives while scrolled back | Parser processes bytes; offset is incremented by new-row count (content-anchored); viewport stays pinned to same content rows; user sees `[scrolled back N rows]` indicator with updated count |
| EC-245 | Both scrolled-back AND dump-drop warning active simultaneously | Status bar renders BOTH `[scrolled back N rows]` AND `[dump: N drops]` concurrently; neither suppresses the other |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 1100 lines of output; PtyScrollUp × 10 (session "s1" focused) | `pty_scroll_offsets["s1"] = 10`; rows 1090-1100 visible (scrolled 10 rows back); `pty_scroll_offsets` for other sessions unchanged | happy-path |
| PtyScrollDown when `pty_scroll_offsets[focused_session_id] = 0` | Offset stays 0; no error | edge-case |
| Focus switch from "s1" (offset=10) to "s2" (offset=0) | `pty_scroll_offsets["s1"] = 10` preserved; `pty_scroll_offsets["s2"] = 0`; render uses `pty_scroll_offsets["s2"]` for new focused session | happy-path |
| Config `pty_scrollback_rows: 500` | Parser initialized with `scrollback_rows = 500` | happy-path |
| Config `pty_scrollback_rows: 15000` | Parser initialized with `scrollback_rows = 10000` (clamped) | edge-case |
| Scrolled to offset=10; 5 new rows of output arrive (PtyOutput) | `pty_scroll_offsets["s1"] = 15` (incremented by 5 — content-anchored); viewport rows unchanged; `[scrolled back 15 rows]` shown | content-anchored |
| Scrolled to offset=0 (live tail); 5 new rows of output arrive | `pty_scroll_offsets["s1"] = 0` (unchanged — live tail never adjusted); live output visible | content-anchored |
| Scrolled to offset=990 (near max of 1000); 20 new rows arrive | offset clamped to `min(1000, 990+20) = 1000`; no overflow, no error | content-anchored edge |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `vt100::Parser` initialized with configured scrollback_rows | unit |
| VP-TBD | `PtyScrollUp/Down` adjusts `pty_scroll_offsets[focused_session_id]` and clamps correctly | unit |
| VP-TBD | Focus switch preserves per-session scroll offsets (I7: no cross-session contamination) | unit |
| VP-TBD | No IPC message sent for scroll actions | unit |
| VP-TBD | Content-anchored: new PtyOutput while scrolled back increments offset by new-row count | unit |
| VP-TBD | Status bar renders `[scrolled back N rows]` AND `[dump: N drops]` concurrently when both conditions are true | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — scrollback is part of the embedded PTY widget capability; it enables users to review previous output without leaving EmbeddedTerminal mode |
| Architecture Module | monocle-tui (`App::pty_scroll_offsets`, `pty_parsers`, PtyScrollUp/Down action handlers) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md §Scrollback navigation; §Parser ownership in TUI; §Parser initialization (PTY_DEFAULT_ROWS/COLS, F-S039-P2-004) |
| Test Name | test_BC_2_09_007_scrollback_1000_default_configurable |

## Related BCs

- [BC-2.09.001] — composes with: scrollback viewport offset affects which parser rows are rendered

## Architecture Anchors

- `architecture/SS-embedded-pty.md#scrollback-navigation` — offset semantics, default/max, no-IPC-send rule

## Story Anchor

S-043 — Implement scrollback navigation in monocle-tui

## VP Anchors

VP-TBD — Scrollback offset unit tests (filled after VP creation)

## §Trace v1.4.0

**S-043 Adversarial Pass-1 product rulings — status-bar precedence + content-anchored scrollback** (2026-06-22):

Two product/intent questions surfaced by Adversarial Pass 1 on S-043 required authoritative ruling and explicit normative documentation.

### Ruling 1 — Status-bar indicator precedence (PC-4)

**Finding:** The `[scrolled back N rows]` indicator had no documented precedence rule relative to transient
diagnostic badges like `[dump: N drops]`. The adversary identified a possible implementation where the
dump-drop badge suppresses the scrollback indicator, leaving the user unaware they are scrolled back.

**Ruling:** The `[scrolled back N rows]` indicator is **persistent viewport state** and is NEVER suppressed by
any transient diagnostic. Both indicators MUST render concurrently when their respective conditions are true.
The status bar accommodates multiple concurrent badges. Suppression would cause a silent correctness failure
where the user believes they are at live tail when they are not.

**Changes:** PC-4 rewritten with explicit concurrent-badge mandate; EC-245 added.

### Ruling 2 — Content-anchored offset preservation (PC-5)

**Finding:** PC-5 previously specified that the numeric offset is "preserved, not reset to 0" on new PTY
output. Because `vt100::set_scrollback(N)` is bottom-relative, preserving the NUMERIC offset causes viewport
CONTENT to drift toward newer output as lines arrive — the rows the user is reading scroll away silently.
The adversary identified that the current BC text plus the current implementation use numeric-preserve, which
does not match expected terminal-emulator UX.

**Ruling:** The intended v1 behavior is **content-anchored preservation**: when new output arrives while the
user is scrolled back (offset > 0), the offset is incremented by the number of new rows processed, keeping
the viewport pinned to the same content rows. When the user is at live tail (offset == 0), no adjustment is
made. This matches the UX of iTerm2, tmux, kitty, wezterm, and Alacritty — the canonical production-grade
terminal-emulator behavior. Numeric-preserve is insufficient for monocle's production-grade positioning.

**Implementer impact:** The `on_pty_output` handler MUST adjust `pty_scroll_offsets[session_id]` by the
number of new rows generated by `parser.process(&bytes)` when the offset is > 0. Determining the new-row
count requires reading `parser.screen().scrollback_len()` before and after `process()` (delta = rows added).
Alternatively, parse the bytes to count newlines — but reading the scrollback_len delta is the canonical
vt100 approach and does not require inspecting raw bytes.

**Test impact:** A new unit test `test_BC_2_09_007_content_anchored_new_output` is required. See new
Canonical Test Vectors and VP rows above. The existing `test_BC_2_09_007_new_output_does_not_reset_scroll_offset`
test is INCORRECT as written (it only asserts the offset is not reset to 0; it does not assert that the
offset is adjusted by the new-row count). The test-writer MUST replace it with the content-anchored assertion.

**Changes:** PC-5 rewritten with content-anchored semantics, rationale, and precise algorithm; EC-244
updated; EC-245 added; new Canonical Test Vectors added; new VPs added.

**Routing directives (from product-owner to orchestrator):**
1. **Implementer code change REQUIRED:** `on_pty_output` handler in `monocle-tui/src/app.rs` must
   increment `pty_scroll_offsets[session_id]` by new-row delta when offset > 0. This MUST be scoped
   to story S-043 (BC-2.09.007 is its only BC).
2. **Test-writer change REQUIRED:** Replace `test_BC_2_09_007_new_output_does_not_reset_scroll_offset`
   with `test_BC_2_09_007_content_anchored_new_output` verifying the offset equals original_offset +
   new_rows (clamped). Add `test_BC_2_09_007_concurrent_status_bar_badges` verifying that scrollback
   indicator and dump-drop badge coexist.
3. **Architecture update REQUIRED:** SS-embedded-pty.md §Scrollback navigation and §Scrollback offset
   invariants updated in this same burst (see §Trace v1.15.0 there).
4. **Story inputs[] cascade:** S-043 inputs[] version for BC-2.09.007 must be updated from 1.1.3 → 1.4.0
   (story-writer responsibility per bc_array_changes_propagate_to_body_and_acs policy; orchestrator to
   dispatch story-writer with AC-008/AC-014/EC-244 rewrite reflecting content-anchored semantics).

- SE-16d monotonicity: v1.4.0 timestamp 2026-06-22 >= v1.3.2 timestamp 2026-06-20. PASS.

## §Trace v1.3.2

**Arch-source pin: SS-embedded-pty.md v1.10.0 → v1.11.0** (2026-06-20):
- S-040 delivery flag-set correction bumped SS-embedded-pty to v1.11.0. Architecture Source
  row updated. No behavioral content changed.
- SE-16d monotonicity: v1.3.2 timestamp >= v1.3.1. PASS.

## §Trace v1.3.1

**Arch-source pin: SS-embedded-pty.md v1.9.0 → v1.10.0** (2026-06-20):
- S-039 adversarial convergence bumped SS-embedded-pty to v1.10.0. This BC's Architecture Source
  row is updated to reflect the current version. No behavioral content changed.
- SE-16d monotonicity: v1.3.1 timestamp >= v1.3.0. PASS.

## §Trace v1.3.0

**F-S039-P5-001 — Invariant 1: clarify absent→1000 vs present-out-of-range→clamped semantics** (2026-06-20):

- **Invariant 1 rewritten:** Replaced the ambiguous "if missing or invalid, 1000 is used" phrase
  that incorrectly implied out-of-range values (e.g. `0`) would default to 1000. The authoritative
  behavior is:
  - ABSENT (`pty_scrollback_rows` key missing / config falls back to default → `None`) → 1000 default.
  - PRESENT (key exists with a parseable `u32`) → clamped to [1, 10000]; EC-243 (`0 → 1`) and
    EC-242 (`20000 → 10000`) remain correct and unchanged.
  - A present non-integer fails the whole config load (serde) → default config (`None`) → 1000.
  This aligns Invariant 1 prose with the EC-242/EC-243 edge cases, which already specified the
  correct clamp behavior. The bug was in the invariant wording, not the edge cases.
- Source finding: F-S039-P5-001 (S-039 adversarial Pass-5 — spec-vs-spec contradiction between
  Invariant 1 "invalid → 1000" and EC-243 "0 → 1 (clamped)").
- SE-16d monotonicity: v1.3.0 timestamp 2026-06-20 >= v1.2.0 timestamp 2026-06-20. PASS.

## §Trace v1.2.0

**F-S039-P2-004 — parser default dimensions reference added to Precondition 1** (2026-06-20):

- Precondition 1 extended: added normative note that parsers created on session arrival use
  `PTY_DEFAULT_ROWS = 24` / `PTY_DEFAULT_COLS = 80` per SS-embedded-pty.md §Parser initialization
  (F-S039-P2-004 ruling). These placeholder dims are replaced by real PTY dims on first attach
  via `ScrollbackDumpComplete`. No behavioral change — implementers now have a named constant to
  use instead of hardcoded literals.
- Architecture Source updated: SS-embedded-pty.md v1.7.0 → v1.9.0; added §Parser initialization
  anchor (F-S039-P2-004).
- SE-16d monotonicity: v1.2.0 timestamp 2026-06-20 >= v1.1.3 timestamp 2026-06-16. PASS.

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
