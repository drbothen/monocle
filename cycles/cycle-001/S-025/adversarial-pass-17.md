---
title: S-025 Adversarial Pass 17
pass_number: 17
counter_before: 0/3
counter_after: 1/3 (ADVANCE; HOLDING pending LOW-001 fix-round landing, then re-confirm)
verdict: NITPICK_ONLY (1 LOW + 2 sub-NIT observations; no MED/HIGH/CRITICAL)
head_sha_reviewed: bfcba19
created: 2026-05-29
---

## Summary

Pass 17 dispatched against HEAD bfcba19 (devops Path B: rust-toolchain 1.88 + time 0.3.47 + RUSTSEC-2026-0009 mitigation). Comprehensive verification of all 7 Pass 16 fix rounds passed. Angles I-N (new 1.88 surfaces) exercised with zero defects. One LOW-severity stale doc reference surfaced in BC-2.03.001 lines 35 + 61 ("MSRV 1.86 stable Rust" — architect's MSRV propagation sweep updated 6 artifacts but missed this BC). Counter advances 0/3 → 1/3.

## Verifications Performed (key items)

- [x] rust-toolchain.toml channel = "1.88"
- [x] Cargo.toml workspace rust-version = "1.88"
- [x] Cargo.lock time = 0.3.47, deranged = 0.5.8, num-conv = 0.2.2, time-core = 0.1.8
- [x] deny.toml ignore = [] (empty)
- [x] CI workflow line 43 grep '^channel = "1\.88"$'; lines 49/267/345 toolchain = "1.88"
- [x] Renamed test ac_004_workspace_cargo_pins_rust_version_1_88 at workspace_structure.rs:117 + companion ac_004_ci_yml_has_lint_toolchain_step at line 126
- [x] check_audit_table.py message-field fallback (lines 66-72) + safety assertion (lines 79-86)
- [x] 4 op_ref violations gone from startup_connect.rs
- [x] Audit table 21 rows in canonical SS-engine-module v1.1.26 + byte-identical vendored
- [x] App, EventBusHookEvent, EngineModuleRegistry, BackoffState rows at canonical 1186-1189
- [x] HookEventRecord row attributes monocle-ipc crate at canonical line 1176
- [x] io::Error::other migration in lifecycle.rs:541/688/791 + lock.rs:173/190 (no remaining ErrorKind::Other)
- [x] uninlined_format_args = "allow" at Cargo.toml:100 with documented rationale
- [x] ratatui feature trim preserved: ["crossterm", "underline-color", "macros", "layout-cache"]
- [x] Pass 11-14 fixes preserved: render_frame precedence (app.rs:941-953), 10 pub consts, 15 lib.rs re-exports, DarkGray baseline test (startup_connect.rs:1383-1490)
- [x] Architect MSRV sweep verified: SS-deps-pin-manifest v1.2.0, prd v1.27.3, nfr-catalog v1.8, product-brief v1.4.31, ADR-0001, risk-acceptance — all correctly updated. SS-conventions "Rust 1.86+" correctly preserved (Rust language history fact, not project MSRV)
- [ ] cargo build/test/clippy/fmt on worktree — NOT EXECUTED (adversary read-only)

## Findings

### F-S025-ADV17-LOW-001 — BC-2.03.001 stale "MSRV 1.86 stable Rust" reference (pending intent verification)

**Severity:** LOW. **Confidence:** HIGH. **Routing:** product-owner (BC owner) — 2-line update.

**Evidence:**
- File: .factory/specs/behavioral-contracts/ss-03/BC-2.03.001.md
- Line 35 (Description): "...does not yet provide ergonomic dyn-compatibility on MSRV 1.86 stable Rust."
- Line 61 (Invariant 3): "Native async fn in traits (stable since Rust 1.75) does NOT yet provide both properties ergonomically on MSRV 1.86 stable."

**Pattern:** Architect Path B MSRV propagation sweep updated 6 artifacts but missed BC-2.03.001. The "intentionally kept" note in architect report referred to SS-conventions ("Rust 1.86+" Rust-language-history fact, correctly preserved) — BC-2.03.001 is the different case ("MSRV 1.86 stable" = present-tense project MSRV claim).

**Semantic analysis:**
- Technical claim (native async fn ergonomics for dyn-compat) is MSRV-independent — correct on both 1.86 and 1.88
- The cited MSRV number "1.86" is a stale project-MSRV reference — project now ships on 1.88
- Comparison: SS-conventions lines 155/812/2423 say "Rust 1.86+" = "starting from 1.86" (correct on 1.88 since 1.88 ≥ 1.86); BC-2.03.001 says "MSRV 1.86 stable" = "the project's MSRV is 1.86" (incorrect on 1.88)

**Severity rationale:** LOW because no implementation impact (BC technical content correct); descriptive comment, not postcondition.

**Suggested resolution:** Two-line change BC-2.03.001:35 + :61 replacing "MSRV 1.86 stable" → "MSRV 1.88 stable". BC version bump + §Trace entry.

## Observations (sub-NIT — not flagged)

- **OBS-001 [process-gap, informational]:** Cargo.toml:100 `uninlined_format_args = "allow"` rationale mentions "will be addressed in a dedicated cleanup pass" but is not anchored to a specific task. However, the workspace-level allow correctly covers test-file format args; production code has zero inline-able candidates (the one `format!("claude-{}", proc.pid)` at engine/claude_code.rs:380 uses field-access, non-inlineable). So the deferred "cleanup pass" is technically vacuous. Informational only.

- **OBS-002 [informational]:** BackoffState row in audit table at SS-engine-module canonical 1189 + vendored 44 references a struct that doesn't exist in S-025 worktree (S-025 branched before S-023 merged). Architect's exhaustive develop-branch sweep added the row in anticipation of post-merge state. CI script computes gaps = production_struct_names - table_struct_names (semgrep scans worktree, finds 14 structs ⊆ 21-row table = no gaps). Forward-looking inclusion works as designed. Informational only.

## Angles Attacked

| Angle | Result |
|-------|--------|
| I — New 1.88 clippy regressions | PASS (io_other_error fixed 6 sites; uninlined_format_args workspace-allowed with rationale) |
| J — time 0.3.47 transitive callers | PASS (no monocle crate uses deranged/num-conv/time API; semver-major bumps in deranged/num-conv have zero production impact) |
| K — ratatui defense-in-depth feature trim | PASS (monocle-tui imports only Block/Borders/List/ListItem/ListState/Paragraph/StatefulWidget/Widget — all baseline widgets not gated by widget-calendar) |
| L — CI AC-004 trip-wire coverage | PASS (renamed test runs via cargo test --workspace at ci.yml:297; assertion in test body at workspace_structure.rs:120 verifies "rust-version = \"1.88\""; companion test verifies workflow itself) |
| M — Multi-spec MSRV consistency | PARTIAL (architect propagated to 6 artifacts; BC-2.03.001 missed — F-S025-ADV17-LOW-001 above; SS-conventions correctly preserved as Rust language history fact) |
| N — Re-verify Pass 1-15 axes | PASS (all prior fixes preserved; no regressions) |

## Class-Sibling Sweep

Grep "MSRV 1.86" pattern across spec artifacts:
- behavioral-contracts/: only BC-2.03.001 (LOW-001)
- architecture/: no present-tense "MSRV 1.86" claim
- prd*.md + product-brief.md: updated to 1.88 with historical preservation
- SS-conventions-anti-patterns.md: "Rust 1.86+" correctly preserved as language history fact

No class-sibling proliferation. Single-file finding.

Audit-table sweep: 14 src-side #[non_exhaustive] pub struct matches verified against 21-row table (gap-free for current worktree). BackoffState forward-looking.

## Counter Decision

ADVANCES 0/3 → 1/3 per consistent rubric: zero CRITICAL/HIGH/MED. LOW finding is `(pending intent verification)` tagged; intent is unambiguous on inspection (architect's own sweep rule applies); 2-line fix + version bump. Per CLAUDE.md Principle 4 (fix in scope), the LOW fix-round is dispatched in parallel with Pass 17 persistence. Counter advance holds pending fix-round landing — Pass 18 dispatch at the post-fix HEAD.

## Defense of the Search

Pass 17 attacked 6 new angles (I-N) targeting the 7-round Pass 16 closure pattern + re-verification of all prior Pass 1-16 surfaces. Architect's MSRV sweep proved comprehensive across 6 spec artifacts with one stale reference missed (BC-2.03.001) that fresh-context cognitive diversity surfaced. The finding is grounded in specific file:line evidence + textual comparison with architect's other propagation targets ("Rust 1.86+" vs "MSRV 1.86 stable" semantic distinction).

No CRITICAL/HIGH/MED defects detected. Convergence trajectory advances 0/3 → 1/3.
