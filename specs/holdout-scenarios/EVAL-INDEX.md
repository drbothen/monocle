---
document_type: holdout-scenario-index
level: ops
version: "1.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-30T00:00:00Z
phase: 2
visibility: holdout-evaluator-only
inputs:
  - {path: .factory/stories/S-016-daemon-binary-cli.md, version: "1.0"}
  - {path: .factory/stories/S-017-daemon-start-sequence.md, version: "1.0"}
  - {path: .factory/stories/S-018-hook-routing-event-bus.md, version: "1.1"}
  - {path: .factory/stories/S-019-daemon-auto-start.md, version: "1.1"}
  - {path: .factory/stories/S-022-tui-connect-permission-prompt.md, version: "1.1"}
  - {path: .factory/stories/S-023-reconnect-soq3.md, version: "1.0"}
  - {path: .factory/stories/S-025-tui-skeleton-sessions.md, version: "1.1"}
  - {path: .factory/stories/S-026-permission-overlay-core.md, version: "1.2"}
  - {path: .factory/stories/S-029-killer-scenario-test.md, version: "1.0"}
  - {path: .factory/stories/S-030-config-crate-foundation.md, version: "1.1"}
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.32"}
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

---

## Wave Coverage Summary

| Wave | Scenarios | Stories Covered | Key Invariant |
|------|-----------|----------------|---------------|
| Wave 4 | HS-EXP-007, HS-EXP-009 | S-016, S-030 | Config atomicity; runtime_dir exit code 70 |
| Wave 5 | HS-EXP-001, HS-EXP-002 | S-017, S-018 | SOQ-2 ordering; fail-open timeout |
| Wave 6 | HS-EXP-003, HS-EXP-004, HS-EXP-005, HS-EXP-006, HS-EXP-010 | S-022, S-023, S-025, S-026 | InitialState gap-free; SOQ-3 ordering; state rebuild; Ctrl-\\ survival |
| Wave 7 | HS-EXP-008 | S-026, S-027, S-029 | Killer scenario ≤6 keystrokes |

**Total expansion holdout scenarios: 10**
**Coverage: ≥1 scenario per wave (Wave 4-7 all covered)**
**Coverage: ≥1 scenario per BC grouping (SS-04, SS-05, SS-06, SS-07)**

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

---

## Relationship to Existing Holdout Scenarios

This index covers Waves 4-7 only. The existing holdout scenarios for Waves 1-3 remain in
`.factory/stories/holdout-scenarios.md` and are NOT superseded by this document.

Phase 4 holdout evaluation MUST evaluate ALL holdout scenarios:
- `.factory/stories/holdout-scenarios.md` — Waves 1-3 (HS-W1-001..HS-W3-006, 14 scenarios)
- This index + scenario files — Waves 4-7 (HS-EXP-001..HS-EXP-010, 10 scenarios)
- **Combined total: 24 holdout scenarios**

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
