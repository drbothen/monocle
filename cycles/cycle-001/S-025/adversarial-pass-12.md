---
title: S-025 Adversarial Pass 12
pass_number: 12
counter_before: 0/3
counter_after: 0/3 (RESET)
verdict: CRITICAL
head_sha_reviewed: 07e207b
created: 2026-05-28
---

## Summary

Pass 12 applied maximum skepticism per L-W6-S025-004 (Pass 9 found a new class after Pass 8 clean) and identified a CRITICAL spec-implementation gap that 11 prior passes missed: `App.status_message` is set on disconnect (AC-003 / BC-2.06.016 PC-4 / BC-2.06.004 PC-2) but is NEVER read by `render_frame`. The disconnect / offline status text is unrenderable. This is exactly the same vacuous-mirror class L-W6-S025-002 was meant to catch (state-mutation tests instead of production-render tests), repeating the Pass 10 root cause for a different field. The 6 pub const extractions landed correctly; SS-tui v1.8.2 and BC-2.06.016 v1.0.8 / S-026 v1.7 pins are correct.

## Verifications Performed

- [x] Pass 11 fix-round commit landings verified (bb7bd2e, 6be3c57, 7481a73, 983d30a, 1aba802, 07e207b all present)
- [x] 6 pub const extractions present and exported via lib.rs (`DAEMON_DISCONNECT_STATUS`, `DAEMON_OFFLINE_STATUS`, `MONOCLE_STATUS_LABEL`, `TOKEN_COUNT_OVERFLOW_CAP`, `UPTIME_OVERFLOW_CAP`, `format_drop_counter` — confirmed in monocle-tui/src/lib.rs:20-30)
- [x] SS-tui frontmatter at v1.8.2 with input-hash `31b6e71`; line 668 bracketed style propagated
- [x] S-026 v1.7 frontmatter pins BC-2.06.016 at v1.0.8 (line 29)
- [x] BC-2.06.016 v1.0.8 body uses production-bracketed text throughout (Description, PC-4, VP table, 4 canonical test vector rows)
- [ ] cargo build/test/clippy/fmt — NOT EXECUTED (read-only profile)
- [ ] PR #28 CI status — NOT EXECUTED (read-only profile)
- [x] Phase-N stale-invariant scan: `ac_005_workspace_does_not_declare_monocle_auth` correctly excludes monocle-tui per F-S025-CI-001 fix; no other phase-bound member-list assertions found.

## Findings

### Finding F-S025-ADV12-CRITICAL-001 — `App.status_message` is write-only state; disconnect/offline status text never reaches the rendered buffer

**Severity:** CRITICAL
**Confidence:** HIGH
**Routing:** `vsdd-factory:implementer` (render_frame in monocle-tui/src/app.rs lines 908–978 must consume app.status_message). Test sibling fix is `vsdd-factory:test-writer` (add render-output test after on_transport_event(Disconnected)).

**Evidence:**

- monocle-tui/src/app.rs:341 — on_transport_event sets app.status_message = Some(DAEMON_DISCONNECT_STATUS.to_string()).
- Same file, lines 657, 673 — TODO-blocked reconnect arms set app.status_message = Some(DAEMON_OFFLINE_STATUS.to_string()).
- Same file, lines 922–933 (render_frame) — status_line value constructed ONLY from app.drop_counter:
  ```rust
  let status_line = if app.drop_counter > 0 {
      Line::from(vec![Span::styled(format_drop_counter(app.drop_counter), ...)])
  } else {
      Line::from(Span::styled(MONOCLE_STATUS_LABEL, ...))
  };
  ```
  No branch reads app.status_message.
- Grep for `\.status_message` returns 4 write sites (app.rs lines 341, 657, 673, comment at 642) and 2 read sites in tests (startup_connect.rs:219, 254). Zero reads in production code.
- Tests test_bc_2_06_004_pc2_ac003_on_disconnect_transitions_to_dashboard and ..._clears_overlay_stack only verify app.status_message.as_deref() == Some(DAEMON_DISCONNECT_STATUS). Neither renders a frame and asserts the buffer contains "[disconnected] reconnecting...".

**Why CRITICAL:** AC-003 (S-025 line 61): "renders a status bar notification: '[disconnected] reconnecting...'". BC-2.06.016 PC-4 (line 62): "Status bar renders the text '[disconnected] reconnecting...' until the IPC reconnect sequence completes." BC-2.06.004 PC-2 makes the same render obligation. The status text NEVER appears to the user. User sees "monocle" (DarkGray) when disconnected with zero drops — identical to running-with-no-drops state. No visual signal that daemon disconnected.

**Class identity:** SAME vacuous-mirror class as L-W6-S025-002. Pass 10 caught _verbatim_match substring-on-local-buffer; Pass 12 catches status_message mutation-test pretending to be render-test.

**Sweep wider:** DAEMON_OFFLINE_STATUS (lines 657, 673) — same defect. Even after S-023 merge, reconnect arm's app.status_message = None won't take effect because render_frame still ignores the field. CLASS bug, not single-line.

**Proposed remediation (implementer scope):**
```rust
let status_line = if let Some(msg) = app.status_message.as_deref() {
    Line::from(Span::styled(msg, Style::default().fg(Color::Yellow)))
} else if app.drop_counter > 0 {
    Line::from(vec![Span::styled(format_drop_counter(app.drop_counter), Style::default().fg(Color::Yellow))])
} else {
    Line::from(Span::styled(MONOCLE_STATUS_LABEL, Style::default().fg(Color::DarkGray)))
};
```

Orchestrator note: precedence (message wins over drop_counter) is grounded in BC-2.06.016 PC-4 ("renders" unconditionally on disconnect — hard render contract). No architect adjudication required per CLAUDE.md production-grade-default.

**Test sibling fix (test-writer scope):** Add startup_connect.rs tests using TestBackend that render after on_transport_event(Disconnected) and assert buffer contains DAEMON_DISCONNECT_STATUS. Mirror parallel test for DAEMON_OFFLINE_STATUS.

### Finding F-S025-ADV12-MED-001 — Pass 11 fix-round commit IDs unverifiable from worktree HEAD (read-only profile limitation)

**Severity:** MED
**Confidence:** MEDIUM
**Routing:** `vsdd-factory:state-manager` (verify commit lineage; informational).

Adversary verified artifact CONTENT consistency: BC-2.06.016 v1.0.8 §Trace timestamp matches, SS-tui frontmatter input-hash 31b6e71 matches architect commit description, S-026 v1.7 BC pin matches story-writer description. Content-consistent with claimed fix round.

### Finding F-S025-ADV12-LOW-001 — S-025 frontmatter pins BCs only, not SS-tui directly (prompt expectation mismatch)

**Severity:** LOW
**Confidence:** MEDIUM
**Routing:** `vsdd-factory:story-writer` for adjudication (intent).

S-025 inputs[] pins 4 BCs + SS-deps-pin-manifest, no direct SS-tui pin. S-026 follows same pattern (BCs + SS-deps-pin-manifest, no SS-XX direct pin). Convention appears to be "stories pin BCs, not subsystem architecture docs." Pass 12 verification request was prompt-construction error; not a defect.

### Finding F-S025-ADV12-LOW-002 — BC-2.06.016 v1.0.8 §Trace stale "Follow-up required" note

**Severity:** LOW
**Confidence:** HIGH
**Routing:** `vsdd-factory:product-owner` (BC-2.06.016 §Trace polish).

BC-2.06.016 line 230: "Follow-up required (architect scope): SS-tui.md line 668 still cites prose form — LOW severity." But SS-tui line 668 already reads bracketed form (architect commit 740465d). §Trace note is stale; sweep to mark "RESOLVED in architect commit 740465d (SS-tui v1.8.1 → v1.8.2)".

## Class-Sibling Sweep

For CRITICAL-001: swept all uses of app.status_message (5 grep hits — all writes in app.rs, 2 reads in tests, ZERO reads in production). Swept render_frame call sites (1 prod, 1 test, neither reads). Sibling SessionsPanel::render explicitly reserves status_bar_area without consuming status_message per F-S025-ADV2-MED-002. Spec contracts violated: AC-003, BC-2.06.004 PC-2, BC-2.06.016 PC-4.

For MED-001: single-instance, not a class pattern.

For LOW-001: S-026 frontmatter confirms BC-only pinning convention.

For LOW-002: one-off documentation lag, no class pattern.

## Counter Decision

**RESET to 0/3.** CRITICAL finding triggers at-or-above-HIGH reset rule.

## Recommended Next Action

Route F-S025-ADV12-CRITICAL-001 to two specialists:
1. `vsdd-factory:test-writer` — write failing render-output tests FIRST (TDD red gate)
2. `vsdd-factory:implementer` — update render_frame to consume status_message; verify tests pass

Then dispatch Pass 13. Optional: LOW-002 polish via product-owner.

## Defense of the Search

Re-derived from artifacts cold; applied L-W6-S025-002 to a different field (status_message); traced render_frame top-to-bottom; exhaustive grep of `.status_message` (0 production reads); verified sibling SessionsPanel::render intentionally doesn't consume (per F-S025-ADV2-MED-002). Survived 11 passes because prior passes focused on state-mutation handler or drop-counter render, never on "what happens when status_message is Some?".
