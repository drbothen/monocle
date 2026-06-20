---
document_type: holdout-scenario-index
level: ops
version: "1.29"
status: active
producer: vsdd-factory:state-manager
timestamp: 2026-06-20T00:00:00Z
phase: 2
visibility: holdout-evaluator-only
inputs:
  - {path: .factory/stories/S-016-daemon-binary-cli.md, version: "1.1"}
  - {path: .factory/stories/S-017-daemon-start-sequence.md, version: "1.1"}
  - {path: .factory/stories/S-018-hook-routing-event-bus.md, version: "1.2"}
  - {path: .factory/stories/S-019-daemon-auto-start.md, version: "1.2"}
  - {path: .factory/stories/S-022-tui-connect-permission-prompt.md, version: "1.4"}
  - {path: .factory/stories/S-023-reconnect-soq3.md, version: "1.2"}
  - {path: .factory/stories/S-025-tui-skeleton-sessions.md, version: "1.14"}
  - {path: .factory/stories/S-026-permission-overlay-core.md, version: "1.11"}
  - {path: .factory/stories/S-027-overlay-rendering-status-bar.md, version: "1.10"}
  - {path: .factory/stories/S-029-killer-scenario-test.md, version: "1.3"}
  - {path: .factory/stories/S-030-config-crate-foundation.md, version: "1.1"}
  - {path: .factory/stories/S-033-session-manager-spawn.md, version: "1.9"}
  - {path: .factory/stories/S-034-session-manager-kill.md, version: "1.3"}
  - {path: .factory/stories/S-035-session-manager-attach-detach.md, version: "1.2.3"}
  - {path: .factory/stories/S-036-session-manager-rediscovery.md, version: "1.5"}
  - {path: .factory/stories/S-037-session-manager-gc.md, version: "1.0.3"}
  - {path: .factory/stories/S-038-session-manager-hook-injection.md, version: "1.5"}
  - {path: .factory/stories/S-039-pty-output-pipeline.md, version: "1.3"}
  - {path: .factory/stories/S-040-keyboard-forwarding.md, version: "1.1"}
  - {path: .factory/stories/S-041-mouse-forwarding-sgr.md, version: "1.0"}
  - {path: .factory/stories/S-042-resize-debounce-resizepane.md, version: "1.2"}
  - {path: .factory/stories/S-043-scrollback-navigation.md, version: "1.1"}
  - {path: .factory/stories/S-044-appmode-transitions-permission-badge.md, version: "1.1"}
  - {path: .factory/stories/S-045-claude-code-spawn-recipe.md, version: "1.3"}
  - {path: .factory/stories/S-046-pty-output-fan-out.md, version: "1.7"}
  - {path: .factory/stories/S-047-ipc-lifecycle-variants.md, version: "1.5"}
  - {path: .factory/stories/S-048-sessions-panel-multi-project.md, version: "1.3"}
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.44.0"}
traces_to: .factory/stories/STORY-INDEX.md
input-hash: "[pending]"
---

# Holdout Scenario Expansion: monocle Waves 4-7

> **PHASE 4 EVALUATOR ACCESS ONLY.**
> This index and all scenario files MUST NOT be shared with implementers or test-writers.
> Holdout scenarios are derived from BCs and domain invariants but are NOT mechanically
> duplicating any story AC.
>
> Evaluator: `vsdd-factory:holdout-evaluator`
> Information asymmetry: evaluator has NOT seen story ACs.

---

## Scenario Index

| ID | Title | Wave | Stories | Severity |
|----|-------|------|---------|---------|
| HS-EXP-001 | SOQ-2 Ordering: hooks-settings.json Never Written Before Lock File | 5 | S-017 | must-pass |
| HS-EXP-002 | PreToolUse Daemon Timeout Returns Allow (Fail-Open) with Correct HTTP Body | 5 | S-018 | must-pass |
| HS-EXP-003 | IPC InitialState Captures Pending Prompts for Late-Connecting TUI | 6 | S-022 | must-pass |
| HS-EXP-004 | SOQ-3 Overlay Clear on Daemon Disconnect — VecDeque Empty Before First Reconnect | 6 | S-023, S-026 | must-pass |
| HS-EXP-005 | IPC State Fully Rebuilds from InitialState After Daemon Restart | 6 | S-023 | must-pass |
| HS-EXP-006 | Ctrl-\\ Popup: Permission Prompts Survive Hide/Show Cycle Without Corruption | 6 | S-025, S-026 | must-pass |
| HS-EXP-007 | Config Atomic Write: tempfile::persist Leaves No Partial Config | 4 | S-030 | must-pass |
| HS-EXP-008 | Killer Scenario: Dual Prompt Resolved in 6 Keystrokes via ratatui TestBackend | 7 | S-029, S-026, S-027 | must-pass |
| HS-EXP-009 | Daemon Binary: runtime_dir Level 4 Fail-Fast Produces Exit Code 70 Not 1 | 4 | S-016 | must-pass |
| HS-EXP-010 | Permission Overlay Lifecycle: Queue → Timeout-Resolved → Clear from Both Paths | 6 | S-022, S-026 | must-pass |
| HS-EXP-011 | Session Survives Graceful Daemon Restart — PTY Stream Re-Attached, SessionEntry Visible | 8 | S-036 | must-pass |
| HS-EXP-012 | Re-Discovery Completes Before UDS Bind — No TUI Connection Accepted During Discovery Window | 8 | S-036 | must-pass |
| HS-EXP-013 | Permission Badge+Bell While in EmbeddedTerminal — SUG-3 Guarantee: Prompt Never Silently Queued | 9 | S-044 | must-pass |
| HS-EXP-014 | Hook Auto-Injection Under Concurrent Spawns — Shared hooks-settings.json Not Clobbered; All Sessions Get Correct `--settings` Arg | 8 | S-033, S-038 | must-pass |
| HS-EXP-015 | Full-Fidelity Keyboard Forwarding — Kitty + SGR Mouse + Bracketed Paste Reach PTY stdin | 9 | S-040, S-041 | must-pass |

---

## Wave Coverage Summary

| Wave | Scenarios | Stories Covered | Key Invariant |
|------|-----------|----------------|---------------|
| Wave 4 | HS-EXP-007, HS-EXP-009 | S-016, S-030 | Config atomicity; runtime_dir exit code 70 |
| Wave 5 | HS-EXP-001, HS-EXP-002 | S-017, S-018 | SOQ-2 ordering; fail-open timeout |
| Wave 6 | HS-EXP-003, HS-EXP-004, HS-EXP-005, HS-EXP-006, HS-EXP-010 | S-022, S-023, S-025, S-026 | InitialState gap-free; SOQ-3 ordering; state rebuild; Ctrl-\\ survival |
| Wave 7 | HS-EXP-008 | S-026, S-027, S-029 | Killer scenario ≤6 keystrokes |
| Wave 8 | HS-EXP-011, HS-EXP-012, HS-EXP-014 | S-036, S-033, S-038 | Session persistence; re-discovery ordering; hook injection concurrency |
| Wave 9 | HS-EXP-013, HS-EXP-015 | S-044, S-040, S-041 | SUG-3 badge+bell (EmbeddedTerminal); full-fidelity keyboard forwarding |

**Total expansion holdout scenarios: 15**
**Coverage: ≥1 scenario per wave (Wave 4-9 all covered)**
**Coverage: ≥1 scenario per BC grouping (SS-04, SS-05, SS-06, SS-07, SS-08, SS-09)**

---

## BC Coverage Traceability

| Scenario | BCs Exercised | Domain Invariant Enforced |
|----------|--------------|--------------------------|
| HS-EXP-001 | BC-2.04.001, BC-2.04.010 | SOQ-2: lock file write precedes hooks-settings.json |
| HS-EXP-002 | BC-2.04.007, BC-2.04.011 | PreToolUse fail-open; try_send non-blocking |
| HS-EXP-003 | BC-2.05.002, BC-2.05.005 | InitialState gap-free invariant (INV-3) |
| HS-EXP-004 | BC-2.05.007, BC-2.06.016 | SOQ-3: overlay clear before reconnect |
| HS-EXP-005 | BC-2.05.006 | InitialState state rebuild after daemon restart |
| HS-EXP-006 | BC-2.06.004, BC-2.06.007, BC-2.06.008 | Ctrl-\\ popup prompt survival |
| HS-EXP-007 | BC-2.07.001, BC-2.07.002 | Config atomic write; no partial write |
| HS-EXP-008 | BC-2.06.022, BC-2.06.009, BC-2.06.011 | Killer scenario; ≤6 keystrokes |
| HS-EXP-009 | BC-2.04.006, BC-2.04.004 | runtime_dir Level 4 fail-fast exit code 70 |
| HS-EXP-010 | BC-2.05.005, BC-2.06.011, BC-2.06.016 | Timeout-resolved + disconnect-cleared overlap |
| HS-EXP-011 | BC-2.08.002, BC-2.08.004, BC-2.05.006 | Session-host survives daemon restart; re-discovery + IPC reconnect integration |
| HS-EXP-012 | BC-2.08.004 | UDS bind blocked until re-discovery complete (startup ordering/race window) |
| HS-EXP-013 | BC-2.09.008, BC-2.09.009, BC-2.06.008 | SUG-3: permission badge+bell within one render tick while in EmbeddedTerminal; Esc→exit→Overlay AppMode transition |
| HS-EXP-014 | BC-2.08.006, BC-2.08.001, BC-HOOK-010 | Hook auto-injection with shared hooks-settings.json under concurrent spawns (shared-file model per BC-HOOK-010; no clobber because spawns read-only the shared file) |
| HS-EXP-015 | BC-2.09.002, BC-2.09.003, BC-2.09.004, BC-2.09.005 | Full-fidelity keyboard forwarding: all v1A input classes (Kitty + SGR + bracketed paste) |

---

## Relationship to Existing Holdout Scenarios

This index covers Waves 4-9. The existing holdout scenarios for Waves 1-3 remain in
`.factory/stories/holdout-scenarios.md` and are NOT superseded by this document.

Phase 4 holdout evaluation MUST evaluate ALL holdout scenarios:
- `.factory/stories/holdout-scenarios.md` — Waves 1-3 (HS-W1-001..HS-W3-006, 14 scenarios)
- This index + scenario files — Waves 4-7 (HS-EXP-001..HS-EXP-010, 10 scenarios)
- This index + scenario files — Waves 8-9 v1A (HS-EXP-011..HS-EXP-015, 5 scenarios: HS-EXP-011/012/014 = Wave 8; HS-EXP-013/015 = Wave 9)
- **Combined total: 29 holdout scenarios**

---

## §Trace v1.5

**POL-11 remediation: BC-INDEX active input pin v1.23 → v1.32** (2026-05-30):
- EVAL-INDEX is an `*-INDEX.md` doc (ACTIVE set per ADR-0007 closed rule); its `inputs[]` BC-INDEX pin must track canonical.
- BC-INDEX.md input pin updated: `"1.23"` → `"1.32"` (Option 1 per ADR-0007 §Decision).
- Version bumped v1.4 → v1.5.

## §Trace v1.3

**Mechanical pin cascade: BC-INDEX v1.22 → v1.23 + STORY-INDEX traces_to v4.5 → v4.7** (2026-05-27):
- BC-INDEX.md input pin updated: `"1.22"` → `"1.23"`.
- traces_to updated: `v4.5` → `v4.7`.
- Version bumped v1.3 → v1.4.

## §Trace v1.2

**Phase 2 Adversarial Review Pass 3 — HS-EXP-007 BC reference corrected** (2026-05-27):
- F-P3-HIGH-004: HS-EXP-007 `source_bcs` corrected — `BC-2.07.006` (CCR Detection) replaced
  with `BC-2.07.001` (Config File Atomic Write via tempfile::persist). This is the correct BC
  for atomic write holdout testing; BC-2.07.006 is about CCR path detection, not atomicity.
- BC Coverage Traceability table row for HS-EXP-007 updated: `BC-2.07.002, BC-2.07.006` →
  `BC-2.07.001, BC-2.07.002`.
- BC-INDEX.md pin updated: `"1.19"` → `"1.21"`.
- traces_to updated: `v4.0` → `v4.4`.
- Version bumped v1.1→v1.2.

## §Trace v1.1

**Phase 2 Adversarial Review Pass 2 — stale story version pins updated** (2026-05-27):
- F-P2ADV-P2-008: Story version pins updated to match actual story file versions after Pass 1/Pass 2 remediation:
  - S-018: "1.0" → "1.1"; S-019: "1.0" → "1.1"; S-022: "1.0" → "1.1"
  - S-025: "1.0" → "1.1"; S-026: "1.0" → "1.2"; S-030: "1.0" → "1.1"
  - S-016: remains "1.0" (no Pass 1/2 changes); S-017: remains "1.0"; S-023: remains "1.0"; S-029: remains "1.0"
- Version bumped v1.0→v1.1.

## §Trace v1.0

**Phase 2 Expansion burst** (2026-05-27T00:00:00Z):
- 10 expansion holdout scenarios created for Waves 4-7 (S-016 through S-031 scope).
- Scenarios derive from BC body invariants and domain ordering constraints not mechanically
  repeated in story ACs. Each scenario tests a property that crosses AC boundaries or
  exercises an interaction between stories that no single AC captures.
- Focus areas: SOQ-2 (HS-EXP-001), fail-open timeout (HS-EXP-002), InitialState gap-free
  (HS-EXP-003), SOQ-3 ordering (HS-EXP-004), state rebuild (HS-EXP-005), Ctrl-\\ survival
  (HS-EXP-006), config atomicity (HS-EXP-007), killer scenario (HS-EXP-008), runtime_dir
  exit code (HS-EXP-009), lifecycle overlap (HS-EXP-010).
## §Trace v1.6 — POL-11 version-pin remediation (2026-05-30)

**Bump:** 1.5 → 1.6.
**Scope:** `traces_to:` field: `STORY-INDEX.md v4.7` → `STORY-INDEX.md v5.20` (Option 1 per ADR-0007 §Decision; EVAL-INDEX is an active INDEX document; its traces_to must reflect canonical current STORY-INDEX version).
**SE-16d PASS:** 2026-05-30 >= 2026-05-30 (patch; no normative behavioral change).

## §Trace v1.19 — F-GATE-ADV-003: S-033..S-048 added to inputs[]; BC-INDEX pin cascade to 1.43.8 (2026-06-16)

**Bump:** 1.18 → 1.19.
**Scope:** `inputs[]` array extended with 16 new v1A story entries covering Waves 8-9 (S-033..S-048).
BC-INDEX input pin updated: `"1.43.7"` → `"1.43.8"` (BC-INDEX bumped in this burst for F-P20-BCGAP-001).
Rationale: EVAL-INDEX is the Phase 4 holdout-evaluator's input manifest. HS-EXP-011..015 test properties of
the Wave 8-9 stories (S-033..S-048). The evaluator needs these story ACs as inputs to evaluate the holdout
scenarios. Their absence from inputs[] (F-GATE-ADV-003) left the evaluator blind to the ACs for all 5
new holdout scenarios. No holdout scenario IDs, titles, BCs, or behavioral semantics changed.
**SE-16d PASS:** 2026-06-16 >= 2026-06-16 (same day as v1.18). PASS (same-day sequential patch).

## §Trace v1.29 — POL-11 pin cascade: BC-INDEX input pin v1.43.8 → v1.44.0 (2026-06-20)

**Bump:** 1.28 → 1.29.
**Scope:** `inputs[]` BC-INDEX pin updated: `"1.43.8"` → `"1.44.0"` (BC-INDEX was bumped in the same burst to record pty_scrollback_rows schema extension: BC-2.07.002 v1.0.3→v1.1.0; SS-config.md v1.3.0→v1.4.0). No holdout scenario IDs, BCs, or behavioral semantics changed. EVAL-INDEX is an active INDEX document per ADR-0007 §Active-set rules; its input pins must track canonical current versions.
**SE-16d PASS:** 2026-06-20 >= 2026-06-20 (same day as v1.28). PASS.

## §Trace v1.28 — POL-11 pin cascade: S-036 input pin v1.4 → v1.5 (2026-06-20)

**Bump:** 1.27 → 1.28.
**Scope:** `inputs[]` S-036 story pin updated: `"1.4"` → `"1.5"` (S-036 v1.5 corrects AC-004 proxy_task over-specification — `proxy_task: Some(handle)` → `proxy_task: None` at re-discovery; spec-text fix only; no behavioral semantics of holdout scenarios HS-EXP-011/HS-EXP-012 changed). No holdout scenario IDs, BCs, or behavioral semantics changed.
**SE-16d PASS:** 2026-06-20 >= 2026-06-19 (v1.27 last-recorded bump date). PASS.

## §Trace v1.27 — POL-11 pin cascade: S-036 input pin v1.3 → v1.4 (2026-06-19)

**Bump:** 1.26 → 1.27.
**Scope:** `inputs[]` S-036 story pin updated: `"1.3"` → `"1.4"` (Option 1 per ADR-0007 §Decision — EVAL-INDEX is an active INDEX document; its story input pins must track canonical current versions). S-036 was bumped to v1.4 in commit b5e522e (BC-2.08.004 null/absent kill_deadline_unix_ms case added to PC-2b) but EVAL-INDEX was not cascaded at that time. No holdout scenario IDs, BCs, or behavioral semantics changed.
**SE-16d PASS:** 2026-06-19 >= 2026-06-16 (v1.26 last-recorded bump date). PASS.

## §Trace v1.18 — F-P20-SUG-001: HS-EXP-013 and HS-EXP-015 wave corrected from 8 → 9 (2026-06-16)

**Bump:** 1.17 → 1.18.
**Scope:** Scenario Index table and Wave Coverage Summary corrected.
- HS-EXP-013: Wave column `8` → `9`. Stories tested: S-044, which is Wave 9 per STORY-INDEX (EPIC-09, Wave 9 row: S-039/S-040/S-041/S-042/S-043/S-044).
- HS-EXP-015: Wave column `8` → `9`. Stories tested: S-040, S-041, both Wave 9 per STORY-INDEX (EPIC-09 Embedded PTY scope).
- Wave Coverage Summary: Wave 8 row revised to HS-EXP-011/012/014 only (S-036/S-033/S-038). Wave 9 row added: HS-EXP-013, HS-EXP-015 (S-044/S-040/S-041).
- Coverage statement updated: "Wave 4-8 all covered" → "Wave 4-9 all covered".
- HS-EXP-013 scenario file: v1.1 → v1.2 (wave: 8 → 9 + §Trace entry). HS-EXP-015 scenario file: v1.2 → v1.3 (wave: 8 → 9 + §Trace entry).
- The `wave:` field convention is confirmed as **tested-story wave** (the wave of the stories the holdout exercises), not authoring wave. Evidence: EVAL-INDEX §Trace v1.10 set `wave: 8` concurrently with `stories_tested: [S-TBD-session-manager]` for Wave 8 stories, and EVAL-INDEX Wave Coverage Summary groups holdouts by tested-story wave throughout.
**SE-16d PASS:** 2026-06-16 >= 2026-06-16 (same day as v1.17). PASS.

## §Trace v1.17 — F-P14-SUG-002: HS-EXP-013 BC Coverage Traceability row adds BC-2.09.008 (2026-06-16)

**Bump:** 1.16 → 1.17.
**Scope:** BC Coverage Traceability table row for HS-EXP-013 updated: `BC-2.09.009, BC-2.06.008` → `BC-2.09.008, BC-2.09.009, BC-2.06.008`. The Esc→exit→Overlay AppMode-transition mechanic in HS-EXP-013 step 8 is owned by BC-2.09.008 PC-1 (not BC-2.09.009 PC-5a/PC-5b, which is a restatement). Domain Invariant column updated to reflect BC-2.09.008's contribution. No other rows changed.
**SE-16d PASS:** 2026-06-16 >= 2026-06-15 (v1.16). PASS.

## §Trace v1.16 — Phase-2 Burst G: HS-EXP-011..015 S-TBD anchors resolved (2026-06-15)

**Bump:** 1.15 → 1.16.
**Scope:** Scenario Index table and Wave Coverage Summary HS-EXP-011..015 story placeholders resolved to canonical story IDs.
- HS-EXP-011: `S-TBD-session-manager` → `S-036` (SessionManager::rediscover_sessions — setsid persistence + state handling)
- HS-EXP-012: `S-TBD-session-manager` → `S-036` (same rediscovery story; both holdouts test BC-2.08.004 properties)
- HS-EXP-013: `S-TBD-embedded-pty` → `S-044` (EmbeddedTerminal + SessionCreation AppMode transitions, permission badge+bell)
- HS-EXP-014: `S-TBD-session-manager` → `S-033, S-038` (spawn + hook auto-injection stories)
- HS-EXP-015: `S-TBD-embedded-pty` → `S-040, S-041` (keyboard forwarding + mouse forwarding stories)
Wave Coverage Summary Wave 8 row updated accordingly.
BC-INDEX input pin updated: `"1.41.1"` → `"1.42.0"` (BC-INDEX bumped in Burst G).
The `stories_tested` frontmatter fields in the individual HS-EXP-011..015 scenario files were resolved by Burst E (product-owner) and contain the canonical story IDs.
The historical note in §Trace v1.0 describing `stories_tested: [S-TBD-session-manager]` documents the original authoring state and is intentionally preserved as historical context.
**SE-16d PASS:** 2026-06-15 >= 2026-06-15 (same day as v1.15 authoring; Burst G is same-session continuation).

## §Trace v1.15 — Pass-9 S-P9-001: HS-EXP-014 stale schema v2 label corrected to v3 (2026-06-03)

**Bump:** 1.14 → 1.15.
**Scope:** HS-EXP-014.md cosmetic stale-label fix only. No Scenario Index table, BC Coverage Traceability table, Wave Coverage Summary rows, or behavioral semantics changed.
**Correction:** Two occurrences of "schema v2 per SS-session-manager v1.7.0" updated to "schema v3 (`schema_version: 3`) per SS-session-manager v1.7.0" — in the FAIL criterion and in the C2-004 modification note. The `hook_settings_path` absence assertion is true in both v2 and v3; the stale label was purely cosmetic. SS-session-manager v1.7.0 canonically defines `schema_version: 3` (v3 added `kill_deadline_unix_ms`).
**SE-16d PASS:** 2026-06-03 >= 2026-06-03 (same-day cosmetic fix; no holdout scenario behavioral change).

## §Trace v1.14 — Pass-8 S-P8-001: HS-EXP-015 input-class count normalized to 6 (2026-06-03)

**Bump:** 1.13 → 1.14.
**Scope:** HS-EXP-015 Expected Outcome corrected: "5 input classes" → "6 input classes" (cosmetic count
drift; enumerated set was always six). Two further "all five" → "all six" occurrences fixed in the
NOT-in-any-story-AC paragraph. Satisfaction Criteria PASS was already correct at "All 6". No Scenario
Index, BC Coverage Traceability, or Wave Coverage Summary rows changed.
See HS-EXP-015.md §Trace v1.2 for full detail.
**SE-16d PASS:** 2026-06-03 >= 2026-06-03 (same-day cosmetic fix; no holdout scenario behavioral change).

## §Trace v1.13 — Pass-6 I6-001: HS-EXP-015 phantom MouseInput removal (2026-06-03T23:45:00Z)

**Bump:** 1.12 → 1.13.
**Scope:** HS-EXP-015 Setup corrected (scenario-file internal fix). No Scenario Index table, BC Coverage
Traceability table, or Wave Coverage Summary rows changed. EVAL-INDEX version bumped for auditability only.
**Correction:** HS-EXP-015 Setup referenced a phantom `MouseInput` IPC variant that does not exist.
Mouse events are SGR-encoded and forwarded as `KeyInput` per SS-embedded-pty SS-09 §Mouse support.
Setup rewritten to reference `KeyInput` only; Part D (Steps 15-16) was already correct.
See HS-EXP-015.md §Trace v1.1 for full detail.
**SE-16d PASS:** 2026-06-03T23:45:00Z > 2026-06-03T14:00:00Z (v1.12). PASS.

## §Trace v1.12 — BC-INDEX pin cascade from adversarial pass-1 (2026-06-03T14:00:00Z)

_(Previously unlabeled — retroactively assigned version number for §Trace ordering consistency. Content is as-committed.)_

## §Trace v1.11 — Adversarial pass-1 PO-owned fixes: holdout scenario data-model reconciliation (2026-06-03T14:00:00Z)

**Bump:** 1.10 → 1.11.
**Scope:** Four holdout scenarios corrected per adversarial pass-1 PO-owned findings.

- HS-EXP-011: C4 fix — flat sidecar path `<runtime_dir>/session-<id>.json` (NOT nested `runtime_dir/sessions/<id>/session-state.json`); field corrected to `state: "Running"` (NOT `status: Reconnected`). The `Reconnected` state does not exist; re-discovered sessions are `Running`. Setup, Steps 6+9, Expected Outcome, Satisfaction Criteria all corrected.
- HS-EXP-012: C4 fix — flat sidecar path in Setup + Satisfaction Criteria. Re-discovery scans `runtime_dir/session-*.json` (flat glob, not nested directory).
- HS-EXP-013: O3 fix — Step 8 keybinding pinned to `Esc` per BC-2.09.009 PC-5 exact semantics (Esc → prior → Overlay); removed ambiguous "or a dedicated key" wording.
- HS-EXP-014: C1 reconciliation — title updated to reflect shared-file model; Part C (per-session hooks branch) removed; Steps revised to validate single shared `<runtime_dir>/hooks-settings.json` not modified by spawns; BC-HOOK-010 added to source_bcs.

Scenario Index table: HS-EXP-014 title updated. BC Coverage Traceability: HS-EXP-014 cross-refs updated. BC-INDEX input pin: 1.35 → 1.36.

**SE-16d PASS:** 2026-06-03T14:00:00Z > 2026-06-03T12:00:00Z (v1.10). PASS.

## §Trace v1.10 — D-241 control-center v1A: 5 new holdout scenarios HS-EXP-011..HS-EXP-015 (2026-06-03T12:00:00Z)

**Bump:** 1.9 → 1.10.
**Scope:** Wave 8 v1A holdout scenarios added for SS-08 (Session Manager) and SS-09 (Embedded PTY) BCs introduced by the D-236 control-center pivot.

New scenarios:
- HS-EXP-011 (BC-2.08.002/004/05.006): session survives graceful daemon restart; PTY re-attached; SessionEntry visible after reconnect. Tests the integration timing property between BC-2.08.002 (session persistence) and BC-2.08.004 (UDS bind blocked until re-discovery) and BC-2.05.006 (TUI reconnect).
- HS-EXP-012 (BC-2.08.004): re-discovery completes before UDS bind; concurrent TUI connect attempts during discovery window all fail. Tests the race-window ordering property (startup sequencing invariant).
- HS-EXP-013 (BC-2.09.009/2.06.008): permission badge+bell within one render tick while in EmbeddedTerminal (SUG-3 guarantee). Tests the concurrent PTY output + permission prompt surface property that no AC captures in isolation.
- HS-EXP-014 (BC-2.08.006/001/BC-HOOK-010): hook auto-injection under 5 concurrent spawns; shared hooks-settings.json not clobbered. Tests the shared-file model invariant under concurrent load (v1.10 description had per-session path model — superseded by C1 reconciliation in v1.11).
- HS-EXP-015 (BC-2.09.002/003/004/005): all v1A input classes (printable, control, arrows, Kitty CSI u, SGR mouse, bracketed paste) forwarded correctly in a single running EmbeddedTerminal session with adversarial 100-keystroke flood.

Scenario Index table updated: 10 → 15 scenarios. Wave Coverage Summary updated to include Wave 8. BC Coverage Traceability table updated with 5 new rows. Combined total: 24 → 29 holdout scenarios. BC-INDEX input pin updated: "1.34" → "1.35".

All 5 new scenarios carry `wave: 8` and `stories_tested: [S-TBD-session-manager]` or `[S-TBD-embedded-pty]` — these will be updated to canonical story IDs after story-writer decomposes the Wave 8 stories for the v1A scope.

**SE-16d PASS:** 2026-06-03T12:00:00Z > 2026-06-03T00:00:00Z (v1.9). ARITHMETICALLY TRUE. PASS.

## §Trace v1.9 — MED-003: add S-027 to inputs[] for HS-EXP-008 evaluator coverage (2026-06-03)

**Bump:** 1.8 → 1.9.
**Scope:** `inputs[]` array: added `{path: .factory/stories/S-027-overlay-rendering-status-bar.md, version: "1.10"}` between S-026 and S-029 entries.
**Rationale:** HS-EXP-008 declares `stories_tested: [S-029, S-026, S-027]` — the holdout evaluator needs S-027's AC spec (overlay rendering + diff-preview + two-row status bar) as an input to evaluate the rendering/diff-preview expectations asserted by HS-EXP-008. S-027 was omitted from the inputs[] array at index creation time; this is a coverage gap (MED-003) that leaves the evaluator blind to S-027's ACs during Phase 4 evaluation.
**SE-16d PASS:** 2026-06-03 >= 2026-05-31 (monotonicity satisfied; no holdout scenario behavioral change — this is a metadata/input-pin correction).

## §Trace v1.8 — POL-11 cascade: story inputs[] pins updated to canonical (2026-05-31)

**Bump:** 1.7 → 1.8.
**Scope:** `inputs[]` story version pins updated to canonical current versions per version-pin-registry.yaml (Option 1 per ADR-0007 §Decision — EVAL-INDEX is an active INDEX document):
- S-016: "1.0" → "1.1"; S-017: "1.0" → "1.1"; S-018: "1.1" → "1.2"; S-019: "1.1" → "1.2"
- S-022: "1.1" → "1.4"; S-023: "1.0" → "1.2"; S-025: "1.1" → "1.14"; S-026: "1.2" → "1.11"
- S-029: "1.0" → "1.2"; S-030: "1.1" remains (no change)
**Trigger:** POL-11 gate failure on fix/WAVE6-GATE-CRIT-001-reconnect-reentry PR #31.
**SE-16d PASS:** 2026-05-31 >= 2026-05-31 (no holdout scenario behavioral change; input pins are metadata).

## §Trace v1.7 — POL-11 version-pin cascade from BC-INDEX v1.33 (2026-05-30)

**Bump:** 1.6 → 1.7.
**Scope:** `inputs[]` BC-INDEX pin: `"1.32"` → `"1.33"` (Option 1 per ADR-0007 §Decision — EVAL-INDEX is an active INDEX document; BC-INDEX pin bumped after Pass-39 adjudication item 6 corrected BC-2.06.007 Action::Escape → Action::ExitFullscreen terminology).
**SE-16d PASS:** 2026-05-30 >= 2026-05-30 (same-day patch; no holdout scenario behavioral change).
