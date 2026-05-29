---
title: S-025 Adversarial Pass 18
pass_number: 18
counter_before: 1/3
counter_after: 0/3 (RESET — MEDIUM finding F-S025-ADV18-MED-001)
verdict: MED
head_sha_reviewed: bfcba19
created: 2026-05-29
---

## Summary

Pass 18 dispatched against HEAD bfcba19 at the literal Pass-9 / Pass-16 equivalent counter position (Pass 17 NITPICK_ONLY-CLEAN advanced 0/3 → 1/3). Required verifications 1, 2, 4 PASS. Angles O, P, Q, R, T PASS. **Angle S (cargo workspace-level invariants after Path B) surfaces a sibling-sweep gap: the Path B propagation cascade closed at the spec/story layer (factory-artifacts commits c7ae560 + e2944d3) but DID NOT propagate to 17 occurrences across 10 implementation-worktree files that pin "SS-deps-pin-manifest v1.1.19" as their documented source-of-truth.** The current canonical SS-deps-pin-manifest is v1.2.0. Counter RESETS 1/3 → 0/3 per partial-fix regression discipline.

Pattern reproduced (Pass 8 → 9; Pass 15 → 16; Pass 17 → 18): prior-pass clean → next-pass new-class surface via fresh-context cognitive diversity at a different architectural layer.

## Verifications Performed (key results)

- [x] BC-2.03.001 v1.0.6 lines 35+61 "MSRV 1.88 stable" — PASS
- [x] S-014 v1.5 + S-015 v1.7 + S-001 v1.9 + S-003 v1.8 + holdout-scenarios v1.5 + STORY-INDEX v5.11 — all PASS
- [x] ZERO non-§Trace "MSRV 1.86" hits in .factory/specs/ — PASS
- [x] verification-properties scope (Angle Q) — PASS (zero matches)
- [x] BC layer scope (Angle P) — PASS (only BC-2.03.001 had MSRV body refs)
- [x] rust-toolchain.toml 1.88 + Cargo.lock time 0.3.47 + deny.toml clean — PASS
- [x] AC-004 trip-wire test, audit table 21 rows, 15 lib.rs re-exports, render_frame precedence — PASS (Pass 11-17 fixes preserved)
- [x] Audit trail consistency across 7 §Trace entries (Angle T) — PASS
- [ ] **Cargo.toml/deny.toml/dependabot.yml/audit.yml/×6 Cargo.toml/workspace_structure.rs SS-deps-pin-manifest version pointer (Angle S)** — **FAIL — 17 occurrences across 10 files still pin v1.1.19**

## Findings

### F-S025-ADV18-MED-001 — Path B propagation cascade tail-gap: SS-deps-pin-manifest v1.1.19 doc-pointers in 10 implementation-worktree files (17 occurrences)

**Severity:** MEDIUM. **Confidence:** HIGH. **Routing:** devops-engineer (same role that landed Path B bfcba19; mechanical text replacement).

**Evidence (HEAD bfcba19, worktree `/Users/jmagady/Dev/monocle/.worktrees/S-025`):**

| File | Line(s) | Count |
|------|---------|-------|
| Cargo.toml | 25, 27, 49, 79 | 4 |
| deny.toml | 1 | 1 |
| .github/dependabot.yml | 3, 15, 48 | 3 |
| .github/workflows/audit.yml | 45 | 1 |
| xtask/Cargo.toml | 37 | 1 |
| crates/monocle-test-harness/Cargo.toml | 21, 24, 30 | 3 |
| crates/monocle/Cargo.toml | 48 | 1 |
| crates/monocle-proto/Cargo.toml | 26 | 1 |
| crates/monocle-runtime/Cargo.toml | 40 | 1 |
| crates/monocle-runtime/tests/workspace_structure.rs | 207 (panic message) | 1 |

**Total: 17 occurrences across 10 files.**

**Class identity:** Same defect class as Pass 17 LOW-001 (stale version pointer in active text after a propagation sweep). Pass 17 caught the spec-layer instance (BC-2.03.001); Pass 18 catches the implementation-layer instance. One architectural layer deeper.

**Internal-Cargo.toml inconsistency:** Line 18 says `rust-version = "1.88"` (Path B applied); line 25 says "Pin policy source of truth: SS-deps-pin-manifest.md v1.1.19" (pre-Path B). Same file, contradicting versions.

**Highest sub-case:** workspace_structure.rs:207 bakes "SS-deps-pin-manifest v1.1.19" into a runtime panic message. Future test failures on bytes 1.11 would display stale-version rationale to maintainers.

**Severity rationale:**
- Per S-7.01 Partial-Fix Regression Discipline rubric: "blast radius 2+ files: HIGH" — softened to MED because all 17 occurrences are pure-doc-pointers with zero functional/security impact; the policy content cited from v1.1.19 (EXACT-pin discipline, bytes 1.11 floor) is still present (and refined) in v1.2.0.
- Internal Cargo.toml inconsistency lifts severity above LOW.
- Final: MEDIUM.

**Suggested resolution:**
1. Mechanical text replacement: all "SS-deps-pin-manifest.md v1.1.19" → "v1.2.0" and "SS-deps-pin-manifest v1.1.19" → "v1.2.0" across the 10 files.
2. cargo build/test/clippy --all-targets/fmt verification.
3. Push + confirm CI all 9 green.
4. Pass 19 dispatches against post-fix HEAD.

**[process-gap] dimension:** F-S025-ADV16-CODIFY-001 MSRV-bump playbook scope (5 layers: architecture/, behavioral-contracts/, stories/ inputs[], stories/ body, planning artifacts) needs a 6th sweep target: **implementation-worktree policy-pointer comments** (Cargo.toml policy headers, deny.toml/dependabot.yml/audit.yml/CI policy comments, test panic messages citing policy doc version).

## Observations (sub-NIT — not flagged as blockers)

- OBS-001: §Trace 1.86 entries preserved as historical records — correct policy
- OBS-002: product-brief.md OQ-11 historical preservation — correct
- OBS-003: input-hash "[live-state]" sentinel convention — confirmed (37 stories use it)
- OBS-004: 9 pub consts (5 app.rs + 4 sessions_panel.rs) all re-exported in lib.rs — structural discipline satisfied; minor counting variance from prior pass reports (10 vs 9) is non-defect

## Angles Attacked

- **O (cascade regression risk)**: PASS
- **P (BCs other than BC-2.03.001)**: PASS
- **Q (verification-properties)**: PASS
- **R (Pass 1-17 re-verify)**: PASS
- **S (workspace-level invariants)**: **FAIL** — MED-001
- **T (§Trace audit trail consistency)**: PASS

## Class-Sibling Sweep

Sweep across worktree for SS-deps-pin-manifest v1.1.19 pattern: 17 hits across 10 files (full enumeration above). No second-class siblings found (no stale v1.1.18, v1.1.17). monocle-tui crate (S-025's own crate) is internally clean.

## Counter Decision

**RESETS 1/3 → 0/3** per MED finding. The Pass-9/Pass-16 pattern reproduced: prior-pass NITPICK_ONLY-CLEAN counter-advance immediately followed by next-pass new-class finding via fresh-context attack at a different architectural layer. Convergence not reached.

## Defense of the Search

Pass 18 attacked 6 angles (O-T) plus re-verification of all Pass 1-17 axes. Pass 17 LOW-001's spec-layer fix triggered the cascade tail closure at the .factory/ layer; fresh-context Pass 18 extended the sweep to the worktree implementation layer and found the gap. This finding is structurally NEW (not a refinement) — it represents a previously-unattacked architectural surface for the same Path B propagation defect class.

The architect + PO + story-writer scope demarcations explicitly stopped at .factory/ artifacts. The worktree implementation-layer policy-pointer comments were never enumerated as a propagation target until Pass 18's Angle S surfaced them.
