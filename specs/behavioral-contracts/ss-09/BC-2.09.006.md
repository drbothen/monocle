---
document_type: behavioral-contract
level: L3
version: "1.3.0"
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

# Behavioral Contract BC-2.09.006: Resize — PTY and Parser Resized Within 2 Render Ticks of Pane Area Change; 50ms Debounce

## Description

When the TUI's Preview pane area dimensions change (user resizes terminal, panel layout
changes), monocle detects the new dimensions, debounces for 50ms, then sends
`ClientToServer::ResizePane { session_id, rows, cols }` to the daemon. The daemon forwards
to the session-host as `DaemonToHost::Resize`, which calls `pty.resize()` and
`parser.set_size()`. The TUI's local `vt100::Parser` is also resized. Both the PTY and
the parser must reflect the new size within 2 render ticks of the first dimension change.

## Preconditions

1. `AppMode::EmbeddedTerminal { session_id }` is active.
2. The Preview pane's `Rect` area has changed since the last rendered size.
3. The 50ms debounce window has elapsed since the first size change detected.

## Postconditions

1. Size change detection: at each render cycle, the TUI checks
   `area.rows != parser.screen().size().0 || area.cols != parser.screen().size().1`.
   If different AND 50ms has elapsed since the first detected change, a resize is triggered.
2. `ClientToServer::ResizePane { session_id, rows: area.rows, cols: area.cols }` is sent
   over IPC within the same render cycle as the size change detection.
3. The TUI calls `pty_parsers[session_id].set_size(area.rows, area.cols)` locally to
   keep the parser in sync with the displayed area.
4. The daemon routes `ResizePane` to `SessionManager::resize_session(session_id, rows, cols)`.
5. `SessionManager` sends `DaemonToHost::Resize { rows, cols }` to the session-host.
6. The session-host calls `pty.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })`
   and `parser.set_size(rows, cols)`.
7. The harness child (Claude Code) receives `SIGWINCH` (if PTY master sends it, which
   `portable-pty` does on resize) and adjusts its output formatting.
8. All of steps 2-6 complete within 2 render ticks (≈ 33ms at 60fps) of the first size
   change. Total end-to-end resize latency from user terminal resize to PTY resize: ≤ 100ms
   (50ms debounce + ~50ms for IPC round-trip and PTY resize).

## Invariants

1. The 50ms debounce prevents excessive resize messages during drag operations. Only one
   `ResizePane` message is sent per 50ms window; intermediate sizes are discarded.
2. The TUI tracks `last_sent_size: Option<(u16, u16)>`. A resize message is sent ONLY when
   `pending_size != last_sent_size` AND the 50ms debounce has elapsed.
3. Resize is sent for the FOCUSED session only. If the user resizes the terminal while
   looking at a non-focused session in the sessions panel, no `ResizePane` is sent until the
   session is focused in EmbeddedTerminal mode.
4. `parser.set_size()` in the TUI is called synchronously (not debounced) to keep the local
   rendering correct on the NEXT render tick. The IPC `ResizePane` message is debounced but
   the local parser is updated immediately.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-235 | User rapidly resizes terminal (continuous drag) | Debounce fires; only the final stable size within each 50ms window is sent; multiple intermediate sizes coalesced |
| EC-236 | Resize while `AppMode::Dashboard` (not in EmbeddedTerminal) | No `ResizePane` sent; local parsers are NOT resized; they will be resized when the session is next entered in EmbeddedTerminal mode |
| EC-237 | Resize to same size as current | `area.rows == parser.screen().size().0 && area.cols == parser.screen().size().1`; no-op; no IPC message sent |
| EC-238 | Session-host dies while resize IPC is in-flight | Resize message arrives at dead socket; daemon handles `SessionError` from `resize_session()`; session transitions to Terminated; TUI receives `SessionStateChanged::Terminated` |
| EC-239 | TUI pane area collapses to 0 rows or 0 cols (degenerate layout) | The TUI detects area.rows==0 or area.cols==0 as a degenerate case and does NOT send ResizePane (no-op; same as "resize to same size as current"). The daemon's IPC handler also clamps zero dimensions to minimum 1 as a defense-in-depth fallback (BC-2.05.010 EC-282 Invariant 5). If ResizePane(rows=0, cols=0) somehow reaches the daemon, the daemon clamps to 1x1 and resizes without error. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Pane resizes from 24×80 to 30×100; 50ms debounce elapsed | `ResizePane { rows: 30, cols: 100 }` sent; session-host PTY resized to 30×100 | happy-path |
| Continuous resize: 24×80 → 25×82 → 26×84 (within 50ms) | Only one `ResizePane` for 26×84 sent after 50ms | edge-case |
| Resize to current size (no change) | No IPC message sent | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `ResizePane` sent after 50ms debounce with correct dimensions | unit (tokio::time::pause) |
| VP-TBD | Local parser resized immediately on detection | unit |
| VP-TBD | Rapid resize coalesced (only final size sent per 50ms window) | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — resize propagation is a core PTY widget capability that ensures the harness child renders at the correct dimensions in the embedded terminal pane |
| Architecture Module | monocle-tui (resize detection, debounce, ResizePane send); monocle-runtime (SessionManager resize_session); monocle-session-host (PTY resize) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md §Pane area and resize (detection logic, debounce, SIGWINCH) |
| Test Name | test_BC_2_09_006_pty_and_parser_resized_within_2_render_ticks |

## Related BCs

- [BC-2.09.001] — composes with: after resize, PTY output is rendered at new dimensions via the parser pipeline
- [BC-2.05.010] — depends on: ResizePane IPC variant; EC-282/Invariant 5 defines zero-dimension clamp at daemon boundary

## Architecture Anchors

- `architecture/SS-embedded-pty.md#pane-area-and-resize` — detection logic, 50ms debounce, resize sequence

## Story Anchor

- **S-042** — Full end-to-end resize: TUI-side (resize detection, 50ms debounce,
  `ClientToServer::ResizePane` send, local `vt100::Parser` immediate resize) + daemon
  routing leg (`ClientToServer::ResizePane` IPC dispatch arm in `ipc_server.rs`,
  `SessionManager::resize_session()` implementation, zero-dimension clamp,
  `DaemonToHost::Resize` forwarding) + session-host leg (`DaemonToHost::Resize` handler:
  `pty.resize()` + `parser.set_size()`) [All Postconditions 1–8; EC-238 daemon host-dead
  error handling; EC-239 daemon zero-dim clamp]. Wave 9.
- **S-047** — IPC lifecycle variants (BC-2.05.010/BC-2.05.011): `KeyInput`, `RenameSession`,
  scrollback protocol, fan-out. S-047 does NOT own the `ResizePane` IPC arm or
  `resize_session()` — these are S-042 scope per human ruling 2026-06-21.

**Postcondition ownership (all S-042):**

| PC | Description (summary) | Owning Story |
|----|----------------------|-------------|
| PC-1 | Size change detection per render cycle | S-042 |
| PC-2 | `ClientToServer::ResizePane` sent on debounce expiry | S-042 |
| PC-3 | Local `vt100::Parser` resized immediately (not debounced) | S-042 |
| PC-4 | Daemon routes `ResizePane` → `SessionManager::resize_session()` | S-042 |
| PC-5 | `SessionManager` sends `DaemonToHost::Resize` to session-host | S-042 |
| PC-6 | Session-host calls `pty.resize()` and `parser.set_size()` | S-042 (session-host binary) |
| PC-7 | Harness child receives SIGWINCH | S-042 (session-host binary, via `portable-pty`) |
| PC-8 | End-to-end latency ≤ 100ms | S-042 |

## VP Anchors

VP-TBD — Resize debounce timing tests (filled after VP creation)

## §Trace v1.3.0

**Human ruling: full end-to-end resize belongs to S-042; ResizePane/resize_session removed from S-047 scope** (2026-06-21):

- **Root cause of correction:** The v1.2.0 Architect ruling (S-042/S-047 split) was based on two
  false assumptions: (a) `ClientToServer` is `#[non_exhaustive]` with a wildcard `_ =>` arm so an
  unrecognised `ResizePane` variant would be silently dropped — this is FALSE; `ClientToServer`
  has no `#[non_exhaustive]` and no wildcard arm; adding `ResizePane` without a matching arm is
  a compile error; (b) S-047 ships before S-042 (Wave 8 before Wave 9) — this is FALSE; S-047 is
  `status: draft`, Wave 8, and its deps (S-046 ← S-032) are undelivered. Leaving `resize_session()`
  as `todo!()` while S-042's TUI sends `ResizePane` ships a live compile failure and an
  end-to-end-inert feature, violating the production-grade principle.
- **Human ruling (authoritative 2026-06-21):** Expand S-042 to end-to-end. The RESIZE-specific
  daemon leg (ipc_server.rs `ResizePane` dispatch arm, `session_manager.resize_session()`,
  zero-dim clamp, `DaemonToHost::Resize` forwarding) belongs to S-042, NOT S-047.
- **Story Anchor revised:** Single owner S-042 for all PCs 1–8, EC-238, EC-239 daemon clamp.
  S-047 keeps non-resize IPC lifecycle variants (KeyInput, RenameSession, scrollback protocol).
- **Postcondition ownership table updated:** PC-4 and PC-5 moved from S-047 → S-042.
- **No behavioral content changed.** PC-1..PC-8 text, invariants, and edge case definitions
  are unchanged. Only ownership attribution revised.
- SE-16d monotonicity: v1.3.0 > v1.2.0. PASS.

## §Trace v1.2.0

**Architect ruling: Story Anchor split S-042 (TUI + session-host) vs S-047 (daemon leg)** (2026-06-21):

- **Root cause:** SS-session-manager.md Ruling A table had a stale row assigning the daemon-side
  `ResizePane` IPC handler and `resize_session()` to S-042. This contradicted S-047 AC-003,
  S-047's IPC Handler Arm Ownership table, STORY-INDEX BC-2.05.010 row, and SS-session-manager
  line 2946 (same Ruling A, keyboard-forwarding row which correctly says "S-047 owns KeyInput /
  ResizePane / RenameSession IPC arms"). The stale row was an authoring artifact predating S-047
  v1.1.
- **Story Anchor expanded** from single "S-042" to a two-entry split table:
  - S-042: PC-1/2/3/6/7/8 — TUI-side (detection, debounce, send, local parser, session-host handler)
  - S-047: PC-4/5 — daemon-side (IPC dispatch arm, `resize_session()`, `DaemonToHost::Resize`)
- **Postcondition ownership table added** under Story Anchor for implementer clarity.
- **No behavioral content changed.** PC-1..PC-8 text is unchanged; only ownership attribution added.
- **Implications for implementer:** S-042 implementer MUST NOT implement `resize_session()` in
  `session_manager/mod.rs`. The `todo!("S-033 (S-047 scope): ...")` marker is correct — leave it
  for S-047. S-042's daemon-side responsibility is the session-host binary's `DaemonToHost::Resize`
  match arm only.
- **EC-238 and EC-239 daemon-leg ownership:** EC-238 (`SessionError` from `resize_session()` →
  Terminated transition) and EC-239 daemon zero-dim clamp belong to S-047. S-042 owns only the
  TUI-side EC-239 no-op guard (already in AC-012 and S-042 Tasks).
- SE-16d monotonicity: v1.2.0 > v1.1.5. PASS.

## §Trace v1.1.5

**Arch-source pin: SS-embedded-pty.md v1.10.0 → v1.11.0** (2026-06-20):
- S-040 delivery flag-set correction bumped SS-embedded-pty to v1.11.0. Architecture Source
  row updated. No behavioral content changed.
- SE-16d monotonicity: v1.1.5 timestamp >= v1.1.4. PASS.

## §Trace v1.1.3

**Arch-source pin: SS-embedded-pty.md v1.7.0 → v1.10.0** (2026-06-20):
- S-039 adversarial convergence bumped SS-embedded-pty to v1.10.0. This BC's Architecture Source
  row is updated to reflect the current version. No behavioral content changed.
- SE-16d monotonicity: v1.1.3 timestamp >= v1.1.2. PASS.

## §Trace v1.1.2

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-042; BC-2.09.006 previously missing from Burst A/B anchor-fill dispatch list** (2026-06-15):
- Story Anchor filled from Phase-2 Burst B story decomposition. S-042 was confirmed after Burst C. This is the missing-anchor fix from section E spec inconsistency resolution. No behavioral content changed.

## §Trace v1.1.1

**Arch-source pin v1.5.1→v1.5.2** (2026-06-13 / D-277):
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.1.0

**S2-004 adversarial pass-2 fix — zero-dimension edge case + cross-reference** (2026-06-03):
- S2-004 finding: BC-2.09.006 had no zero-dimension handling. BC-2.05.010 EC-282 said
  "SessionError returned" — contradicting this BC which has no error path for resize. The
  two BCs gave different behaviors for the same condition.
- EC-239 added: degenerate pane area (rows=0 or cols=0) — TUI no-ops; daemon defense-in-
  depth clamp to 1 per BC-2.05.010 Invariant 5. Documents the two-layer defense without
  duplicating the daemon's rule.
- Related BCs: added [BC-2.05.010] cross-reference for the zero-dimension clamp relationship.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.006 authored for SS-09 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): local parser resized immediately (not debounced) while IPC
  resize is debounced. This is required for correct rendering during the debounce window
  (the local render uses the new pane size; the PTY will catch up after the IPC round-trip).
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
