---
title: S-025 Adversarial Pass 22
pass_number: 22
counter_before: 1/3
counter_after: 0/3 (RESET — MED finding F-S025-ADV22-MED-001 + sibling drift in 7 EPIC-06 stories)
verdict: MED
head_sha_reviewed: ef7f4c62
created: 2026-05-29
---

## Summary

Pass 22 is the 4th consecutive convergence-attempt reset at the 1/3 → 2/3 transition (Passes 9, 16, 18, 22). The architect-escalation tripwire was armed for Path-B-propagation species recurrence — but Pass 22's finding is a DIFFERENT class one architectural layer deeper than the canonical-anchored version-pin sweep (D-198.2).

**F-S025-ADV22-MED-001: `SS-tui-core.md` is cited in S-025 worktree code AND 7 EPIC-06 story files but the document DOES NOT EXIST.** Canonical is `SS-tui.md` v1.8.2. The canonical-anchored sweep protocol (D-198.2) audits versioned citations (`SS-X.md vY.Z`) but does NOT audit unversioned bare-filename references that fail to resolve.

This is a **META-GAP in the codified sweep protocol itself.** The defect class is "bare-filename architecture anchor resolution" — distinct from version-pointer staleness.

Path-B-propagation species remains BOUNDED (no recurrence). Tripwire NOT FIRED in the literal sense (different class). Counter resets per MED rule.

## Verifications Performed

- [x] Pass 21 CLEAN advance re-validation: all version-pinned citations correctly classified per D-198.2
- [x] All Pass 11-20 closures verified at ef7f4c62 (engine.rs:143 v1.1.26, engine_module_surface.rs:8 parenthetical anchor, lib.rs 15 re-exports, audit table 21 rows, rust 1.88, time 0.3.47 + bytes 1.11.1)
- [x] Build/test/lint deferred to orchestrator (adversary read-only)
- [x] CI on ef7f4c62: 9/9 SUCCESS (orchestrator-confirmed)
- [ ] **Bare-filename architecture anchor resolution audit** — **FAIL: SS-tui-core.md cited but doesn't exist (9 sites: 2 worktree + 7 stories)**

## Findings

### F-S025-ADV22-MED-001 — Broken architecture anchor SS-tui-core.md cited but doesn't exist

**Severity:** MEDIUM. **Confidence:** HIGH. **Routing:** implementer (S-025 in-scope 2 sites) + story-writer (out-of-scope 7 sibling sites on factory-artifacts).

**Evidence (HEAD ef7f4c62):**

In-scope (S-025 newly-authored code):
| File | Line | Citation |
|------|------|----------|
| crates/monocle-tui/src/lib.rs | 7 | `//! # Architecture boundary (SS-tui-core.md)` |
| crates/monocle-tui/Cargo.toml | 19 | `# TUI / terminal rendering — effectful boundary (SS-tui-core.md)` |

Out-of-scope sibling drift (story-spec layer on factory-artifacts):
- stories/S-024-tui-core-types.md:183 (S-024 already merged)
- stories/S-025-tui-skeleton-sessions.md:176 (S-025 spec)
- stories/S-026-permission-overlay-core.md:257
- stories/S-027-overlay-rendering-status-bar.md:167
- stories/S-028-sessions-filter-event-ribbon.md:166
- stories/S-029-killer-scenario-test.md:143
- stories/S-031-profile-picker.md:150

**Verification of intent:**
- `ls .factory/specs/architecture/SS-tui*` returns ONLY `SS-tui.md` (no SS-tui-core.md)
- BC-2.06.005 §Architecture Source pins `SS-tui.md v1.6.0 §Panel Architecture §Sessions Panel` (correct anchor exists in BCs)
- audit-table.md row 41 (`App | monocle-tui | SS-tui.md`) — correct anchor
- Text adjacent to citations ("Architecture boundary", "effectful boundary") matches SS-tui.md §Scope lines 27-40 v1.8.2

**Class identity:** META-GAP in CODIFY-001 D-198.2. The canonical-anchored sweep audits `SS-X.md vY.Z` patterns. Unversioned bare-filename references like `SS-tui-core.md` slip through because there's no version to mismatch.

**Why this was missed for 21 passes:** All prior canonical-anchored sweeps audit versioned citations. SS-tui-core.md is a bare-filename reference — passes the "Category B preserved" classifier because there's no version. This is genuinely novel angle ββ (bare-filename architecture anchor resolution).

**Severity rationale:**
- Blast radius 9 files (2 in-scope code + 7 sibling stories) → would be HIGH on systematic axis
- But in-S-025-scope fix is only 2 mechanical edits
- 7 story-spec sibling defects are pre-existing (EPIC-06 story-writing burst pattern), NOT introduced by S-025
- Final: MED for S-025; sibling fix dispatched to story-writer in parallel per Principle 4

**[process-gap] dimension:** F-S025-ADV16-CODIFY-001 needs a 7th sweep category: **bare-filename architecture anchor resolution audit** — every `SS-X.md` / `ADR-NNNN-*.md` reference (versioned or unversioned) must resolve to a real file in `.factory/specs/architecture/`. Origin: TD-S025-PASS22-PROC-001.

## Angles Attacked

| Angle | Result |
|-------|--------|
| θθ — Cargo workspace metadata-level consistency | PASS (pre-existing tracing-subscriber duplication is not S-025 finding) |
| ιι — lib.rs re-export semantics (consumption, ordering, dead-export) | PASS (15 re-exports for S-026/S-027/S-028 forward-compat per inline comment) |
| κκ — Test-naming convention consistency | PASS (no stale names in S-025 test files) |
| λλ — Error-path coverage (panic propagation, signal handling, queue overflow) | PASS (bounded by daemon-side BC-2.05.002 Inv 4 idempotency) |
| μμ — SS-tui v1.8.2 → current consistency | PASS for version axis; FAIL for anchor-resolution axis (see MED-001) |
| νν — BC body-content drift (BC-2.06.005 PC-2) | PASS (engine.rs:143 cites v1.0.5 PC-2 = current canonical) |
| ξξ — Pass 1-21 axes re-verification | PASS |
| **ββ (new) — Broken architecture anchor resolution** | **FAIL — SS-tui-core.md does not exist; MED-001** |

## Counter Decision

**RESETS 1/3 → 0/3** per MED rule. 4th consecutive 1/3 → 2/3 transition failure but DIFFERENT class than Pass 16/18/20 Path-B species (which is bounded by D-198.2).

## Defense of the Search

Pass 22 attacked 8 angles. The novelty is genuine:
1. Concrete evidence: 2 specific worktree files + line numbers
2. Negative existence proof: Glob returns only `SS-tui.md`
3. Positive intent proof: BC + audit-table both correctly use `SS-tui.md`
4. Not duplicate of any prior pass (zero "SS-tui-core" hits in cycle history)
5. Pattern-consistent with prior 1/3 reset events (each found defect class one layer deeper)

The META-GAP for canonical-anchored sweep (D-198.2): protocol enumerates classification rules for version-pinned references, NOT for bare-filename references that don't resolve. This is the codification target for TD-S025-PASS22-PROC-001.

## Recommended Next

1. **Implementer (S-025 in-scope):** Replace SS-tui-core.md → SS-tui.md in 2 worktree files
2. **Story-writer (sibling drift):** Sweep 7 EPIC-06 story files on factory-artifacts
3. **State-manager:** Persist + extend CODIFY-001 with 7th sweep category

After fix: Pass 23 at post-fix HEAD targets 0/3 → 1/3 (5th convergence attempt begins fresh).
