---
document_type: behavioral-contract
level: L3
version: "1.0.6"
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

# Behavioral Contract BC-2.09.004: Kitty Keyboard Protocol — Enhanced Key Events Forwarded as CSI u Sequences

## Description

When the user's terminal supports the Kitty keyboard enhancement protocol, monocle enables
three enhancement flags at TUI startup. In `AppMode::EmbeddedTerminal`, enhanced key events
(e.g., modifier combinations like `Ctrl+Shift+Enter`, `Alt+F3`, `Shift+Tab`) are translated
to CSI u sequences and forwarded to the PTY. On terminals that do NOT support Kitty protocol,
the enhancement flags silently no-op and standard VT sequences are used as fallback.

## Preconditions

1. TUI startup has executed:
   ```
   crossterm::execute!(stdout(), PushKeyboardEnhancementFlags(
       DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES |
       REPORT_EVENT_TYPES
   ))
   ```
   REPORT_ASSOCIATED_TEXT is unavailable in crossterm-0.29 (commented-out symbol);
   REPORT_ALTERNATE_KEYS is intentionally omitted — no v1A BC depends on layout-alternate
   data. See SS-embedded-pty.md §Crossterm setup S-040 delivery ruling.
2. The terminal supports Kitty keyboard enhancement (detected via `CSI ? u` query; if not
   supported, flags were silently no-op'd and standard sequences remain active).
3. `AppMode::EmbeddedTerminal` is active.

## Postconditions

1. For key events that require Kitty protocol encoding (modifier combinations not expressible
   in standard VT sequences), `is_kitty_enhanced_key(event.code, mods)` returns `true`.
2. `encode_kitty_key(event.code, mods, event.kind)` produces a CSI u sequence:
   `ESC [ <unicode_codepoint> ; <modifier_value> u`
   - `<unicode_codepoint>`: decimal codepoint of the key
   - `<modifier_value>`: computed from modifier bits + 1 (Kitty spec: shift=1, alt=2, ctrl=4, etc.)
3. The CSI u bytes are sent as `ClientToServer::KeyInput`.
4. On terminals without Kitty support: `is_kitty_enhanced_key()` returns false for all keys
   (because Kitty-enhanced events are never generated); standard VT sequences from
   BC-2.09.002 table are used instead. No Kitty-specific code path is reached. No panic.
5. TUI exit sends `crossterm::execute!(stdout(), PopKeyboardEnhancementFlags)` to restore
   terminal state.

## Invariants

1. Kitty enhancement flags are enabled GLOBALLY on TUI startup (not gated on EmbeddedTerminal
   entry). This ensures enhanced key events are available immediately when the user enters
   embedded terminal mode without a terminal state transition.
2. If the terminal does not support Kitty protocol: `PushKeyboardEnhancementFlags` writes
   the CSI sequence; the terminal ignores it (no response to query or non-OK response); monocle
   detects this and skips the flags. The standard VT key table from BC-2.09.002 handles all
   key classes on non-Kitty terminals.
3. `encode_kitty_key()` is a PURE function — no I/O or state mutation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-225 | `Ctrl+Shift+Enter` on Kitty-capable terminal | CSI u sequence; harness receives enhanced encoding |
| EC-226 | `Shift+Tab` on Kitty-capable terminal | CSI u sequence `\x1b[9;2u` (tab codepoint=9; Kitty modifier = 1 + shift(1) = 2) |
| EC-227 | `Alt+F3` on Kitty-capable terminal | CSI u sequence; harness receives alt+F3 correctly |
| EC-228 | `Ctrl+Shift+Enter` on non-Kitty terminal | `is_kitty_enhanced_key()` returns false; falls through to standard key table (Enter → `\r`; Ctrl+Shift modifier not distinguishable); best-effort |
| EC-229 | TUI exits without `PopKeyboardEnhancementFlags` (panic/crash) | Terminal left in Kitty-enhanced mode; user sees raw Kitty sequences in their shell; next TUI launch will `Pop` on clean exit; acceptable for crash recovery |

## Canonical Test Vectors

| Input | Expected PTY bytes | Category |
|-------|-------------------|----------|
| `Ctrl+Shift+Enter` (Kitty-enhanced) | `\x1b[13;6u` (Enter codepoint=13; Kitty modifier = 1 + shift(1) + ctrl(4) = 6) | happy-path |
| Any key on non-Kitty terminal | Standard VT sequence per BC-2.09.002 table | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `encode_kitty_key()` produces correct CSI u sequences for common modifier combos | unit |
| VP-TBD | Non-Kitty terminal path: standard sequences used; no panic | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — Kitty keyboard protocol is explicitly named in CAP-009; this BC defines the CSI u encoding for Kitty-enhanced key events |
| Architecture Module | monocle-core (`encode_kitty_key()`, `is_kitty_enhanced_key()` pure functions); monocle-tui (PushKeyboardEnhancementFlags setup, PopKeyboardEnhancementFlags cleanup) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md v1.11.0 §Crossterm setup (keyboard enhancement flags; S-040 delivery ruling — three-flag set); §Translation function (Kitty branch) |
| Test Name | test_BC_2_09_004_kitty_protocol_csi_u_sequences |

## Related BCs

- [BC-2.09.002] — composes with: Kitty is a sub-class of full-fidelity keyboard forwarding; non-Kitty fallback uses BC-2.09.002 table

## Architecture Anchors

- `architecture/SS-embedded-pty.md#crossterm-setup` — PushKeyboardEnhancementFlags with 3 flags (DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES)
- `architecture/SS-embedded-pty.md#translation-function` — Kitty branch in key_event_to_pty_bytes

## Story Anchor

S-040 — Same story as BC-2.09.002 (keyboard encoding includes Kitty branch)

## VP Anchors

VP-TBD — Kitty encoding unit tests (filled after VP creation)

## §Trace v1.0.6

**S-040 delivery — dependency-reality correction: four-flag set → three-flag set** (2026-06-20):
- **Behavioral correction:** Precondition 1 flag enumeration updated from four flags
  (`DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES | REPORT_ASSOCIATED_TEXT`)
  to three flags (`DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES`).
- **Reason:** `REPORT_ASSOCIATED_TEXT` is a commented-out symbol in crossterm-0.29
  (`// const REPORT_ASSOCIATED_TEXT = 0b0001_0000`) and is not available as a usable constant.
  `REPORT_ALTERNATE_KEYS` is intentionally omitted — no v1A BC depends on layout-alternate data.
  Architect ruling per SS-embedded-pty.md §Crossterm setup S-040 delivery ruling (v1.11.0).
- **Architecture Source row updated:** SS-embedded-pty.md v1.10.0 → v1.11.0. Temporary
  `version-pin-historical` anchor removed; row is now a live pin.
- **Description updated:** "four enhancement flags" → "three enhancement flags".
- **Architecture Anchors updated:** "with 4 flags" → "with 3 flags (DISAMBIGUATE_ESCAPE_CODES |
  REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES)".
- All other behavioral content (postconditions, invariants, edge cases, test vectors, traceability)
  remains correct — the three-flag set fully satisfies all v1A behavioral contracts per
  SS-embedded-pty.md §Crossterm setup S-040 delivery ruling.
- SE-16d monotonicity: v1.0.6 timestamp 2026-06-20 >= v1.0.5 timestamp 2026-06-20. PASS (same-day sequential patch).

## §Trace v1.0.5

**Arch-source pin: SS-embedded-pty.md v1.7.0 → v1.10.0** (2026-06-20):
- S-039 adversarial convergence bumped SS-embedded-pty to v1.10.0. This BC's Architecture Source
  row is updated to reflect the current version. No behavioral content changed.
- SE-16d monotonicity: v1.0.5 timestamp 2026-06-20 >= v1.0.4 timestamp 2026-06-20. PASS (same-day sequential patch).

## §Trace v1.0.3

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-040** (2026-06-15):
- Story Anchor filled from Phase-2 Burst B story decomposition (clusters with BC-2.09.002). No behavioral content changed.

## §Trace v1.0.2

**Arch-source pin v1.5.1→v1.5.2** (2026-06-13 / D-277):
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.0.1

**I24-001 annotation correction — Kitty modifier bitfield coherence** (2026-06-13):
- Finding I24-001 (Phase-1d Pass 24, IMPORTANT): Canonical Test Vectors row for `Ctrl+Shift+Enter`
  used `shift(2)` in the annotation, contradicting PC-2's canonical bitfield `shift=1, alt=2, ctrl=4`.
  The byte literal `\x1b[13;6u` was CORRECT; only the derivation annotation was wrong.
- Fix: Rewrote Ctrl+Shift+Enter annotation to `(Enter codepoint=13; Kitty modifier = 1 + shift(1) + ctrl(4) = 6)`.
- Sweep: EC-226 annotation `modifier=shift+1=2` also clarified to canonical form
  `(tab codepoint=9; Kitty modifier = 1 + shift(1) = 2)` — result unchanged, notation now
  unambiguously matches PC-2 and the `1 + sum(bits)` formula.
- All other Kitty derivation annotations in this file verified against canonical bitfield: no
  further annotation errors found. All byte literals confirmed correct.
- No normative postconditions, invariants, or byte literals changed. Patch version bump only.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.004 authored for SS-09 as part of the v1A control-center pivot BC burst.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
