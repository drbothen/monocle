---
title: S-025 Adversarial Pass 16
pass_number: 16
counter_before: 1/3
counter_after: 0/3 (RESET — MED finding per Pass-9/10 precedent for MED-severity resets)
verdict: MED
head_sha_reviewed: 2073c89
created: 2026-05-28
---

## Summary

Pass 16 conducted fresh-context attack from 8 angles (A-H) Pass 15 did not explore. Pass 11-14 markers verified intact. Angles A-G produced no new findings (panic safety, concurrency, input validation, resource cleanup, spec-impl drift, deferred-item rigor, test-affordance leakage). **Angle H (ADR-0006 compliance) found 1 MED finding** that survived 15 prior passes.

The pattern Pass 8 false-clean → Pass 9 new class fired exactly as predicted at the 1/3 → 2/3 transition. Pass 16's angle rotation uncovered an architectural-invariant gap that no prior pass searched: monocle-tui::App is `#[non_exhaustive]` with a `pub fn new()` cross-crate constructor but was MISSING from the Cross-Crate Constructor Audit Table in SS-engine-module.md. This is the SAME defect class as F-R30-1 (the original ADR-0006 motivation: "audit table claimed completeness while 10 structs were missing"), recurring at smaller scale.

The MED finding triggered a multi-round investigation/fix that uncovered a deeper false-green vector in `scripts/check_audit_table.py` (devops-engineer investigation per [process-gap] tag). The script had been emitting "0 production structs declared — PASS" since commit 184f7d4 because it read struct names from `metavars.$NAME.abstract_content`, but semgrep OSS 1.156.0 (CI version) does not populate metavars for `pattern-either` rules. This false-green was hiding 3 missing audit-table rows (not 1).

## Verifications Performed

- [x] HEAD unchanged from Pass 15 (2073c89) at start of Pass 16
- [x] All Pass 11-14 markers verified intact (render_frame precedence, 6 const extractions, 15 lib.rs re-exports, workspace_structure.rs rename, DarkGray baseline test)
- [x] BC-2.05.002 v1.0.5 Invariant 4 wiring (apply_permission_prompt_queued + 6 dedicated tests)
- [x] BC-2.06.016 PC-4 three-branch precedence (status_message Yellow, drop_counter Yellow, DarkGray baseline) all covered
- [x] Format helpers edge-case coverage (format_cost, format_uptime_at, format_token_count)
- [x] Resource cleanup paths (install_panic_hook + restore_terminal; spawn_ipc_reader backpressure)
- [x] Bounds safety (Angle A): SelectNext/SelectPrev guarded; format_uptime_at sentinel on negative
- [x] Task #9 anchoring (Angle F): F-S025-ADV13-NIT-003 + NIT-004 in STATE.md durable_task_register with status: pending, routing notes
- [x] ADR-0006 audit-table compliance (Angle H): **NEW FINDING** — see F-S025-ADV16-MED-001

## Findings

### F-S025-ADV16-MED-001 — `App` struct missing from Cross-Crate Constructor Audit Table [process-gap]

**Severity:** MED. **Confidence:** HIGH. **Routing:** architect (content) + devops-engineer (CI rule).

**Evidence:**
- crates/monocle-tui/src/app.rs:123-124: `#[non_exhaustive] pub struct App { ... }` with `pub fn new(config: MonocleConfig) -> Self` at line 158
- Cross-crate constructors at monocle-tui/tests/startup_connect.rs:96 (17+ sites) and monocle-tui/tests/sessions_panel.rs:136 (6 sites)
- SS-engine-module.md:1166-1186 (audit table delimited block): no App row
- ADR-0006 §Audit Table Obligation (lines 108-113): "Every struct covered by this ADR MUST appear in the Cross-Crate Constructor Audit Table"
- CI semgrep rule `monocle-non-exhaustive-struct-audit-completeness` was supposed to catch this but did not

**Class identity:** F-R30-1 recurrence (ADR-0006 original motivation case).

**[process-gap] tag:** CI semgrep coverage gap — investigation routed to devops-engineer.

## Multi-Round Fix Sequence

### Round 1 — Architect (commit 98e8102 on factory-artifacts)
- SS-engine-module.md v1.1.22 → v1.1.23
- Added App row to Cross-Crate Constructor Audit Table
- ADR-0006 v1.0 → v1.1 with §Trace entry
- Sweep: only App identified at this point (devops investigation not yet complete)

### Round 2 — Devops-engineer (commit 390d04d on feature/S-025-tui-skeleton-sessions)
- Investigated semgrep rule + CI script
- **Root cause: hypothesis (b) false-green in scripts/check_audit_table.py**
- The rule was correctly authored; the Python script's `parse_semgrep_json()` read struct names from `metavars.$NAME.abstract_content` which semgrep OSS 1.156.0 does not populate for `pattern-either` rules
- False-green active since commit 184f7d4
- Hidden 2 additional missing rows: EventBusHookEvent + EngineModuleRegistry
- Fix: added message-field regex fallback (Path 2) + safety assertion (exit 1 if N findings but 0 names extracted)

### Round 3 — Architect round 2 (commit b504026 on factory-artifacts)
- SS-engine-module.md v1.1.23 → v1.1.24
- Added EventBusHookEvent + EngineModuleRegistry rows (the 2 earlier-wave gaps surfaced by devops-engineer's fix)
- ADR-0006 v1.1 → v1.2
- Sweep confirmed: exactly 3 total missing rows, all 3 now closed in canonical doc

### Round 4 — Test-writer (commit 47b9ba9 on feature/S-025-tui-skeleton-sessions) [process-gap]
- 4 clippy `op_ref` violations in startup_connect.rs color assertion loops (Pass 13 NIT-002 + Pass 14 NIT-001 introduced these; agents missed because they ran `cargo clippy --workspace -- -D warnings` without `--all-targets`; CI uses `--all-targets`)
- Fixed Option A: dropped `&` on right operand (semantics preserved via `PartialEq<String> for str` deref coercion)
- All 4 sites: lines 1160, 1254, 1364, 1474
- [process-gap]: agents should default to `--all-targets` clippy locally to match CI

### Round 5 — Architect round 3 (commit 76690aa on feature/S-025-tui-skeleton-sessions + commit fd00508 on factory-artifacts)
- scripts/audit-table.md (vendored copy): synced with SS-engine-module v1.1.24, added 3 rows + fixed HookEventRecord crate-column (monocle-runtime → monocle-ipc, post-S-022 relocation)
- SS-engine-module.md v1.1.24 → v1.1.25 with HookEventRecord canonical correction + §Trace v1.1.25
- ADR-0006 v1.2 (no further bump — round 5 is content sync only)

## Class-Sibling Sweep

- Audit-table completeness (Angle H sweep): exactly 3 missing rows identified (App, EventBusHookEvent, EngineModuleRegistry); all closed
- HookEventRecord crate-column drift: surfaced and fixed
- L-W6-S025-007 wider sweep: only 4 `op_ref` sites in test code (all fixed); no production code sites
- Other angles A-G: no new findings

## Counter Decision

**RESET 1/3 → 0/3.** MED finding per Pass-9 + Pass-10 MED-resets precedent. Multi-round fix sequence has closed all 5 dimensions:
1. Canonical audit-table content (rounds 1+3): 3 rows added
2. CI false-green vector (round 2): script + safety assertion fixed
3. clippy --all-targets (round 4): 4 op_ref violations closed
4. Vendored copy sync (round 5): scripts/audit-table.md aligned with canonical
5. HookEventRecord stale-crate column (round 5): canonical + vendored both corrected

## Defense of the Search

Pass 15's defense was strong on App-field re-sweep, class-symmetry to NIT-001, L-W6-S025-003 pub const re-export, production-path render verification, forward-compat deferrals, fullscreen `_` arm reachability, spec/input-hash, deferred-task confirmation. Pass 15 did NOT defend the ADR-0006 audit-table angle.

Pass 16 deliberately rotated to Angle H because:
1. S-025 introduces NEW crate (monocle-tui) with NEW `#[non_exhaustive]` struct (App)
2. ADR-0006 was ratified in S-022 cycle — audit table authored before monocle-tui existed
3. CI semgrep rule is exactly the regression-detector class that emits invisible success signals (CI-as-Code review axis)
4. F-R30-1 was the original ADR-0006 motivation; modern recurrence at smaller scale was a real risk

Angles A-G produced no new findings — consistent with Pass 15 maturity. The Angle H finding is the canonical example of fresh-context cognitive diversity value: novel attack vectors uncover real architectural-invariant gaps that 15 prior passes missed because they were not searching this perimeter.

Recurrence count: 2 (F-R30-1 + F-S025-ADV16-MED-001). One more triggers mandatory S-7.02 codification.

### Round 6 — Architect round 4 (commit ee9742b on factory-artifacts + commit 8929e24 on feature/S-025-tui-skeleton-sessions) — Exhaustive sweep closure

After rounds 1-5 closure, PR #28 CI Step 3 semgrep job still failed with:
> Audit table gap: following structs carry #[non_exhaustive] but are absent from the Cross-Crate Constructor Audit Table: `BackoffState`.

Root cause: prior rounds swept only the S-025 worktree (branched before S-023 merged). CI's pull_request merge ref combines S-025 + develop, exposing develop-introduced gaps (BackoffState in monocle-ipc::reconnect from S-023).

Architect dispatch with exhaustive `git ls-tree origin/develop` + per-file `git show` produced the authoritative count of 21 `#[non_exhaustive] pub struct` in the merge state. Exactly 1 gap: BackoffState.

Changes:
- SS-engine-module v1.1.25 → v1.1.26 (BackoffState row added; §Trace v1.1.26 documents exhaustive sweep methodology)
- vendored scripts/audit-table.md synced (delimited block byte-identical to canonical)
- All other 20 structs confirmed correctly listed (no further sweep gaps)

This closes Pass 16 multi-round F-S025-ADV16-MED-001 for the FULL accumulated debt (4 audit-table rows total: App + EventBusHookEvent + EngineModuleRegistry + BackoffState).

### Round 7 — Path B closure (architect commit f3533ce on factory-artifacts + devops-engineer commit bfcba19 on feature/S-025-tui-skeleton-sessions) — RUSTSEC-2026-0009 mitigation via Phase 1 MSRV bump

After rounds 1-6 closed all 4 audit-table gaps (App, EventBusHookEvent, EngineModuleRegistry, BackoffState), PR #28 CI cargo-deny job surfaced RUSTSEC-2026-0009: "Denial of Service via Stack Exhaustion" in `time` crate < 0.3.47 (RFC 2822 parser). Time is transitive via ratatui→ratatui-widgets→time (optional, calendar feature).

Human decision (orchestrator AskUserQuestion): Path B selected over Path A (deny.toml ignore) and Path C (defer to Phase 3 MSRV 1.92). Rationale: root-cause removal preferred when MSRV bump cost is acceptable. Time 0.3.47 requires Rust 1.88.

Changes (architect f3533ce on factory-artifacts):
- SS-deps-pin-manifest.md v1.1.22 → v1.2.0 (minor bump — MSRV interface change): Phase 1 MSRV 1.86 → 1.88; ratatui Cargo.toml Note updated; which Cargo.toml Note updated; MSRV Policy section rewritten; MSRV Constraints table updated; §Trace v1.2.0 documents Path B + SE-16b/SE-16d monotonicity PASS
- Propagation: prd.md v1.27.2→v1.27.3, nfr-catalog v1.7→v1.8, product-brief v1.4.30→v1.4.31, ADR-0001 (Phase 3 sentence "1.86→1.92" → "1.88→1.92"), RUSTSEC-2026-0009 risk-acceptance doc (status: accepted→resolved)

Changes (devops-engineer bfcba19 on feature/S-025-tui-skeleton-sessions):
- rust-toolchain.toml: channel 1.86 → 1.88
- CI AC-004 check updated in 4 locations: .github/workflows/ci.yml (lines 49/267/345 + grep assertion), .github/workflows/audit.yml, .github/workflows/dtu-fidelity.yml, monocle-runtime/tests/workspace_structure.rs (renamed: ac_004_workspace_cargo_pins_rust_version_1_88)
- Cargo.lock: time 0.3.37→0.3.47, deranged 0.3.11→0.5.8, num-conv 0.1.0→0.2.2, time-core 0.1.2→0.1.8
- deny.toml: RUSTSEC-2026-0009 ignore entry REMOVED (clean ignore=[])
- Cargo.toml: ratatui defense-in-depth feature trim KEPT with updated comment (no longer for RUSTSEC mitigation; now for compile-time/binary-size)
- 1.88 new clippy lint fixes: io_other_error (4 sites: lifecycle.rs, lock.rs); uninlined_format_args (production fixed; test sites allowed at workspace lints with rationale)

CI on bfcba19: ALL 9 JOBS SUCCESS (Preflight, DTU, Semgrep audit-table completeness, audit-table-drift, Build+Test x3, cargo deny, cargo audit).

This closes Pass 16 multi-round F-S025-ADV16-MED-001 in its FULL accumulated scope (7 rounds, 4 audit-table rows + CI script false-green + vendored sync + clippy --all-targets + RUSTSEC-2026-0009 + MSRV 1.86→1.88).
