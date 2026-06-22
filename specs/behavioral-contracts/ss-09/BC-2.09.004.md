---
document_type: behavioral-contract
level: L3
version: "1.0.11"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "3bf5d37"
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
2. The terminal supports Kitty keyboard enhancement (detected via
   `crossterm::terminal::supports_keyboard_enhancement()`; if not supported — function returns
   `Err(_)` or `Ok(false)` — flags were NOT pushed and standard sequences remain active).
3. `AppMode::EmbeddedTerminal` is active.

## Postconditions

1. When `key_event_to_pty_bytes(event, kitty_active: bool)` is called with `kitty_active = true`,
   `is_kitty_enhanced_key(event.code, mods, kitty_active: bool)` returns `true` for any key where:
   - `kitty_active == true`, AND
   - `mods` is non-empty (at least one modifier bit set), AND
   - `code != PtyKeyCode::Null` (i.e., the key is a recognized key code).
   These keys are CSI-u encoded per the Kitty keyboard protocol specification.

   NOTE — crossterm-0.29 reality: crossterm does NOT introduce distinct enhanced `KeyCode`
   variants when Kitty flags are active. `KeyCode::Enter`, `KeyCode::Up`, `KeyCode::Char(c)`,
   etc. are the same enum variants regardless of Kitty mode. Enhancement flags change WHICH
   events are reported (more modifier combos become visible; release events appear; modifier-only
   keys surface) and populate `KeyEventState`, but do NOT produce new `KeyCode` variants.
   Consequently, `is_kitty_enhanced_key` CANNOT determine Kitty mode from `(code, mods)` alone —
   the `kitty_active: bool` parameter threads the runtime detection result (from
   `crossterm::terminal::supports_keyboard_enhancement()` at TUI startup, stored in
   `App::kitty_active`) into the pure encoder.

2. `encode_kitty_key(event.code, mods, event.kind)` produces a CSI u sequence:
   `ESC [ <unicode_codepoint> ; <modifier_value> u`
   - `<unicode_codepoint>`: decimal codepoint of the key
   - `<modifier_value>`: computed from modifier bits + 1 (Kitty spec: shift=1, alt=2, ctrl=4, etc.)
3. The CSI u bytes are sent as `ClientToServer::KeyInput`.
4. On terminals without Kitty support (`kitty_active = false`): `is_kitty_enhanced_key()` returns
   false for all keys (because `kitty_active == false` short-circuits the check); standard VT
   sequences from BC-2.09.002 table are used instead. No Kitty-specific code path is reached.
   No panic.
5. TUI exit sends `crossterm::execute!(stdout(), PopKeyboardEnhancementFlags)` to restore
   terminal state — only if `kitty_active = true` (flags were never pushed on non-Kitty
   terminals, so there is nothing to pop).

## Invariants

1. Kitty enhancement flags are enabled GLOBALLY on TUI startup (not gated on EmbeddedTerminal
   entry). This ensures enhanced key events are available immediately when the user enters
   embedded terminal mode without a terminal state transition.
2. If `crossterm::terminal::supports_keyboard_enhancement()` returns `Err(_)` or `Ok(false)`:
   `PushKeyboardEnhancementFlags` is NOT called; `kitty_active = false`; `App::kitty_active`
   is set to false at startup and never mutated. The standard VT key table from BC-2.09.002
   handles all key classes on non-Kitty terminals.
3. `encode_kitty_key()` and `is_kitty_enhanced_key(code, mods, kitty_active)` are PURE functions —
   no I/O, no state mutation, deterministic given their inputs. Purity is preserved for
   `is_kitty_enhanced_key` because `kitty_active` is a plain input parameter (a `bool` threaded
   from the call site), not a runtime I/O read or global state access.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-225 | `Ctrl+Shift+Enter` on Kitty-capable terminal | CSI u sequence; harness receives enhanced encoding |
| EC-226 | `Shift+Tab` on Kitty-capable terminal | CSI u sequence `\x1b[9;2u` (tab codepoint=9; Kitty modifier = 1 + shift(1) = 2) |
| EC-227 | `Alt+F3` on Kitty-capable terminal | CSI u sequence; harness receives alt+F3 correctly |
| EC-228 | `Ctrl+Shift+Enter` on non-Kitty terminal | `is_kitty_enhanced_key(code, mods, false)` returns false (kitty_active=false short-circuits); falls through to standard key table (Enter → `\r`; Ctrl+Shift modifier not distinguishable); best-effort |
| EC-229 | TUI exits without `PopKeyboardEnhancementFlags` (panic/crash) | Terminal left in Kitty-enhanced mode; user sees raw Kitty sequences in their shell; next TUI launch will `Pop` on clean exit; acceptable for crash recovery |
| EC-234 | Terminal does not respond to Kitty keyboard enhancement probe (`supports_keyboard_enhancement()` returns `Err(_)` or `Ok(false)`) | `kitty_active = false`; `PushKeyboardEnhancementFlags` is NOT called; TRACE log emitted; standard VT sequences used throughout session. No enhanced key encoding occurs. |

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
| Architecture Source | SS-embedded-pty.md v1.14.0 (at S-040 authoring time) §Crossterm setup (keyboard enhancement flags; three-flag set; supports_keyboard_enhancement detection); §Translation function (Kitty catch-all precedence; O-1 functional-key codepoints in v1A scope; kitty_active threading; is_kitty_enhanced_key signature); §Risk Mitigations (supports_keyboard_enhancement mandate) |
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

## §Trace v1.0.11

**Behavioral content — EC-234 detection mechanism corrected; 100ms/CSI-?u probe language removed** (2026-06-21):
- **EC-234 trigger rewritten:** Old trigger "CSI ?u query times out at startup (terminal does
  not respond within 100ms)" removed entirely. New trigger: "Terminal does not respond to Kitty
  keyboard enhancement probe (`supports_keyboard_enhancement()` returns `Err(_)` or `Ok(false)`)".
  The outcome is unchanged (`kitty_active = false`; flags not pushed; TRACE; standard VT used).
- **PC-2 (Precondition 2) updated:** Detection reference changed from "detected via `CSI ? u` query"
  to `crossterm::terminal::supports_keyboard_enhancement()` naming both `Err(_)` and `Ok(false)`
  return paths.
- **PC-1 prose (crossterm-0.29 NOTE) updated:** "from the `CSI ? u` query at TUI startup" →
  "from `crossterm::terminal::supports_keyboard_enhancement()` at TUI startup".
- **Invariant 2 updated:** Old "no response to query or non-OK response" replaced with the
  correct conditional: `supports_keyboard_enhancement()` returns `Err(_)` or `Ok(false)`;
  `PushKeyboardEnhancementFlags` is NOT called; `kitty_active = false`.
- Rationale: the mechanism is crossterm's `supports_keyboard_enhancement()` function, which
  uses crossterm's internal timeout (not a hand-rolled 100ms probe). The 100ms value was
  inaccurate and has been removed entirely per architect ruling (SS-embedded-pty.md v1.14.0
  §Risk Mitigations ADV-MED-001 PO directive).
- SE-16d monotonicity: v1.0.11 timestamp 2026-06-21 >= v1.0.10 timestamp 2026-06-21. PASS.

## §Trace v1.0.10

**Arch-source pin: SS-embedded-pty.md v1.13.0 → v1.14.0** (2026-06-21):
- S-040 adversarial pass-6 ADV-MED-001 closed. Architecture Source row updated to v1.14.0.
- PO behavioral directive recorded in SS-embedded-pty.md §Trace v1.14.0: EC-234 trigger
  description must remove "100ms" / "CSI ?u query times out" language; correct mechanism is
  supports_keyboard_enhancement() with crossterm's internal 2000ms timeout. No behavioral
  content changed in this file (PO action required per PO directives in SS-embedded-pty).
- SE-16d monotonicity: v1.0.10 timestamp 2026-06-21 >= v1.0.9 timestamp 2026-06-21. PASS.

## §Trace v1.0.9

**Arch-source pin: SS-embedded-pty.md v1.12.0 → v1.13.0; Kitty codepoint scope clarification** (2026-06-21):
- S-040 adversarial pass-3 architect ruling: O-1 full Kitty functional-key codepoint fidelity
  is IN v1A scope (not deferred). `encode_kitty_key` MUST use correct Kitty protocol codepoints
  for non-Unicode keys (Up=57352, Down=57353, Left=57351, Right=57354, F1=57364, etc.). Using
  placeholder codepoints (e.g., 65='A' for Up) is a data corruption bug. See SS-embedded-pty.md
  §Trace v1.13.0 O-1 ruling for full codepoint table.
- Architecture Source row updated to v1.13.0.
- No postcondition, invariant, or edge-case behavioral content changed in this BC.
  The codepoint fidelity is already implicit in PC-2 ("CSI u sequence per Kitty keyboard
  protocol specification") — this trace records that the specification means the CORRECT
  codepoints, not placeholder values.
- SE-16d monotonicity: v1.0.9 timestamp >= v1.0.8 timestamp. PASS (same-day sequential patch).

## §Trace v1.0.8

**Behavioral content — kitty_active threading; corrected crossterm model; purity invariant; EC-234** (2026-06-21):

- **PC-1 corrected:** `is_kitty_enhanced_key` now takes `kitty_active: bool` parameter; returns
  true ONLY when `kitty_active == true AND mods is non-empty AND code != Null`. The function
  signature is `is_kitty_enhanced_key(code, mods, kitty_active: bool)`.
- **PC-1 prose corrected:** Replaced incorrect claim that crossterm generates "distinct enhanced
  KeyEvent variants / distinct enhanced event types" for Kitty mode. Correct model: crossterm-0.29
  delivers the same `KeyCode` variants regardless of Kitty flag state; enhancement changes WHICH
  events are reported and populates `KeyEventState`, but does NOT introduce new `KeyCode` variants.
  The `kitty_active: bool` parameter threads the runtime detection result from the `CSI ? u` query.
- **`key_event_to_pty_bytes` signature corrected:** `key_event_to_pty_bytes(event, kitty_active: bool)`.
- **PC-4 corrected:** On non-Kitty terminals, `is_kitty_enhanced_key()` returns false because
  `kitty_active == false` short-circuits — not because "Kitty-enhanced events are never generated"
  (that was the wrong model). PopKeyboardEnhancementFlags is only called if `kitty_active = true`.
- **Invariant 3 extended:** `is_kitty_enhanced_key(code, mods, kitty_active)` is also pure;
  purity is preserved because `kitty_active` is a plain input parameter, not runtime I/O.
- **EC-228 corrected:** Updated old `is_kitty_enhanced_key()` call to include `kitty_active` param.
- **EC-234 added:** CSI ?u query timeout at startup → `kitty_active = false`; flags not pushed;
  TRACE emitted; standard VT sequences used. Next free EC id in this BC's sequence (EC-234,
  after EC-233 in BC-2.09.005; EC-230 is already taken in BC-2.09.005).
- SE-16d monotonicity: v1.0.8 timestamp >= v1.0.7 timestamp. PASS (same-day sequential patch).

## §Trace v1.0.7

**Arch-source pin: SS-embedded-pty.md v1.11.0 → v1.12.0** (2026-06-21):
- S-040 adversarial pass-2 architect ruling corrected §Translation function (Kitty branch):
  `is_kitty_enhanced_key` now takes `kitty_active: bool` parameter; `key_event_to_pty_bytes`
  takes `kitty_active: bool`; `App::kitty_active` threaded from `CSI ? u` detection at startup.
  Architecture Source row updated. No postcondition/invariant/edge-case behavioral content changed.
- SE-16d monotonicity: v1.0.7 timestamp >= v1.0.6 timestamp. PASS (same-day sequential patch).

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
