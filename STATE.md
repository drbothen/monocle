---
document_type: pipeline-state
level: ops
project: monocle
version: "6.49"
status: active
producer: state-manager
timestamp: 2026-05-29T06:00:00Z
phase: phase-3-wave-6-IN-PROGRESS
current_step: "S-025 Pass 22 MED REMEDIATED (D-201.1) — F-S025-ADV22-MED-001 CLOSED: implementer e5ebc43 (lib.rs:7 + Cargo.toml:19) + story-writer 0cea089 (7 EPIC-06 stories + STORY-INDEX v5.12 + BONUS SS-forward-compat.md anchor → SS-conventions v1.31.1). CI 9/9 green on e5ebc43. Pass 23 adversary dispatched. Counter 0/3; convergence-attempt #6. META-PATTERN watch: does next-layer class surface?"
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-047..D-174 archived at cycles/cycle-001/decisions-archive.md. D-175: Wave 4 gate PASSED. D-182: Wave 5 gate PASSED (develop @ 1ce7838). D-183: Wave 6 AUTHORIZED. D-184: S-022 DELIVERED (PR #27). D-185: S-023+S-025 parallel AUTHORIZED. D-186: S-023 DELIVERED (PR #29 @ 7a52041). D-187: S-025 in flight. D-188: Pass 12 CRITICAL fix + F-S025-CI-001. D-189: Pass 13 LOW; fix dispatched. D-190: Pass 14 NIT; fix dispatched. D-191: Pass 15 CLEAN; counter 0/3→1/3. D-192: Pass 16 MED (7-round fix; counter RESET 0/3). D-193: Pass 16 round 6 BackoffState gap; F-R30-1 threshold CROSSED. D-194: Pass 16 round 7 Path B RUSTSEC-2026-0009 MSRV 1.86→1.88; CI all 9 green bfcba19. D-195: Pass 17 NITPICK_ONLY-CLEAN (1 LOW BC-2.03.001 MSRV 1.86 stale ref; PO fix dispatched); counter 0/3→1/3 HOLDING; Pass 18 at post-fix HEAD. D-196: Pass 17 LOW-001 FULLY CLOSED — Path B propagation tail 2 cascade rounds (c7ae560 story-writer: S-014/S-015/STORY-INDEX v5.10; e2944d3 story-writer: S-001/S-003/holdout-scenarios/STORY-INDEX v5.11). Zero MSRV 1.86 non-§Trace hits. Counter 1/3 CONFIRMED. Pass 18 ready. D-197: Pass 18 MED-001 RESET counter 1/3 → 0/3 — Path B propagation cascade extends to worktree implementation layer (17 occurrences in 10 files still pin SS-deps-pin-manifest v1.1.19; devops fix-round dispatched in parallel). ADV16-CODIFY-001 extended with 6th sweep target. Pass 19 pending post-fix HEAD. D-197.1: MED-001 CLOSED at devops 9fcfd49 (18 replacements/11 files; wider sweep found types.rs:48 v1.1.20 additional). ADV16-CODIFY-001 6-category enumeration finalized. CI queued. D-198: Pass 19 MED — F-S025-ADV19-MED-001: SS-conventions-anti-patterns v1.30.2 stale active pointers in clippy.toml + deny.toml. Counter HOLDS 0/3. Orchestrator preemptive comprehensive sweep dispatched (devops — all 7 canonical docs + SS-engine-module/SS-ipc active-vs-historical adjudication). ADV16-CODIFY-001 generalized from SS-deps-pin-manifest-specific to ALL concurrent doc bumps. Convergence-attempt #3 stalled at floor (4 consecutive attempts failed to advance past 0/3). D-199: S-025 Pass 20 MED-001 + LOW-001 CLOSED — devops ef7f4c62 (engine.rs:143 v1.1.22→v1.1.26; 72 citations swept canonical-anchored) + test-writer dc229db (engine_module_surface.rs:6-8 Option B anchor). CODIFY-001 D-198.2 canonical-anchored sweep protocol codified. ADV20-PROC-001 added (Test File Documentation Standards). 5 consecutive convergence-attempt stalls; counter HOLDS 0/3. CI pending 10/10 on ef7f4c62; architect-escalation tripwire armed for Pass 21. D-200: Pass 21 NITPICK_ONLY-CLEAN; counter ADVANCES 0/3 → 1/3; 5-stall pattern ENDED; species BOUNDED. D-201: Pass 22 MED — F-S025-ADV22-MED-001 SS-tui-core.md broken anchor (9 sites: 2 worktree + 7 EPIC-06 story files); META-GAP in D-198.2 (bare-filename refs not covered); CODIFY-001 7th sweep category added; ADV22-PROC-001 added (CI-enforced bare-filename anchor resolution); counter RESETS 1/3 → 0/3; 4th 1/3→2/3 failure (DIFFERENT class); implementer + story-writer parallel dispatched. D-201.1: MED-001 CLOSED at e5ebc43 + 0cea089; BONUS SS-forward-compat.md catch → SS-conventions v1.31.1; STORY-INDEX v5.12; CI 9/9 green; Pass 23 dispatched."
awaiting: "Pass 23 adversary (background, dispatched) — convergence attempt #6 first pass; target counter 0/3 → 1/3; META-pattern watch: does another deeper-layer class surface?"
durable_task_register:
  outstanding:
    - id: "F-S022-ADV15-LOW-001"
      subject: "Story S-022 AC-002 ring_tail type doc drift"
      status: closed
      detail: "CLOSED: Story v1.2→v1.3 ring_tail type doc drift fixed by story-writer commit 545b634 during S-023 cycle. AC-002 now correctly cites Vec<HookEventRecord>."
      blocking: false
    - id: "ADV-W5GATE-HIGH-001"
      subject: "daemon_start_sequence() doesn't wire DaemonState — integration story needed"
      status: pending
      detail: "Wave 5 gate adversarial: daemon_start_sequence() in monocle-runtime does not wire DaemonState fields (sock_file_path, last_hook_ts, ring). Daemon runs but TUI state queries return stale/default values. Requires integration story. Route to story-writer for new story in Wave 6/7."
      blocking: false
    - id: "ADV-W5GATE-HIGH-002"
      subject: "Duplicate S-009 handler dead code — cleanup needed"
      status: pending
      detail: "Wave 5 gate adversarial: S-009 HTTP handler has a duplicate code path introduced during Wave 5 IPC routing work. Dead code is non-functional but increases maintenance burden and confusion. Route to implementer for cleanup fix-PR."
      blocking: false
    - id: "ADV-W5GATE-MED-001"
      subject: "S-017 UDS socket creation spurious WARN on rebind"
      status: pending
      detail: "Wave 5 gate adversarial: S-017 daemon start emits spurious WARN log when rebinding an already-removed socket path. Confuses log readers. Add explicit pre-remove check before bind to eliminate the warning."
      blocking: false
    - id: "ADV-W5GATE-MED-003"
      subject: "HookEvent serde round-trip fragility — add constructors to monocle-core"
      status: pending
      detail: "Wave 5 gate adversarial: HookEvent deserialization relies on field-name stability without tagged union. Adding constructors to monocle-core would enforce correct shape at build time. Route to architect for BC update, then implementer."
      blocking: false
    - id: "#28"
      subject: "prost/reqwest exact-patch pin verification"
      status: partial
      detail: "prost = '=0.14.1' VERIFIED on crates.io + resolved in Cargo.lock. reqwest = '=0.13.0' exists but latest 0.13.x is 0.13.3 — reqwest not yet activated by any Phase 1 member. Architect adjudication needed when S-009 activates reqwest."
      blocking: false
    - id: "#34"
      subject: "BC-2.03.001 PC-3 DeferUntil cleanup"
      status: pending
      detail: "BC-2.03.001 v1.0.5 PC-3 still enumerates DeferUntil in supporting types. Authority hierarchy resolves to story v1.4 + SS-engine-module v1.1.20 (no DeferUntil). PO mechanical fix."
      blocking: false
    - id: "BC-HOOK-034-typo"
      subject: "BC-HOOK-034 typo decorated_by -> deprecated_by"
      status: pending
      detail: "Cosmetic typo in BC-HOOK-034 frontmatter. Non-blocking. Maintenance sweep."
      blocking: false
    - id: "VP-DTU-001"
      subject: "VP-DTU-001 to be created by architect in Phase 4"
      status: deferred-phase-4
      detail: "All 41 BC-HOOK files cite VP-DTU-001 as verification property. VP-DTU-001 is Phase 4 deferral marker; architect creates when holdout-evaluator scope is operational."
      blocking: false
    - id: "F-WAVE1-004"
      subject: "Cron schedule collision audit.yml + dtu-fidelity.yml"
      status: deferred-maintenance
      detail: "Both 0 0 * * 0 UTC. Stagger to avoid cache thrash. Low impact."
      blocking: false
    - id: "F-WAVE1-005"
      subject: "xtask not in cargo-deny [graph].targets"
      status: deferred-maintenance
      detail: "xtask is dev-tooling-only. Targets limit cargo-deny to 3 CI runner triples. Low likelihood Phase 1 risk. Document in deny.toml."
      blocking: false
    - id: "F-WAVE1-006"
      subject: "monocle-proto prost-build no-op build cost"
      status: closed-S-013-delivered
      detail: "build.rs no-op stub resolved — S-013 (PR#12, 8653977) wired real .proto files. Closed."
      blocking: false
    - id: "Wave-2-gate-dep-cleanup"
      subject: "Wave-2 gate dep hygiene — 4 findings fixed in closeout e2898be"
      status: closed
      detail: "Removed monocle-proto from monocle-runtime deps; removed futures+semver from monocle-core deps; added sock_file_path to DaemonState. develop @ e2898be. Closed by wave-gate PASS_WITH_OBSERVATIONS (2026-05-26T18:00:00Z)."
      blocking: false
    - id: "S-014-ADV-SS-engine-module-stale"
      subject: "SS-engine-module.md HookDecision code blocks stale"
      status: pending
      detail: "S-014 adversary surfaced stale HookDecision code blocks in SS-engine-module.md. Architect update needed. Sourced from S-014 adversarial review R2."
      blocking: false
    - id: "S-011-ADV-HookArgs-divergence"
      subject: "HookArgs struct diverges from SS-permissions-phase1.md"
      status: pending
      detail: "S-011 adversary surfaced HookArgs struct definition diverging from SS-permissions-phase1.md canonical definition. Architect update needed. Sourced from S-011 adversarial review."
      blocking: false
    - id: "S-013-ADV-prost-types-not-pinned"
      subject: "prost-types not registered in SS-deps-pin-manifest"
      status: pending
      detail: "S-013 adversary noted prost-types is a transitive dependency but not explicitly registered in SS-deps-pin-manifest.md. Architect must add explicit pin. Sourced from S-013 adversarial review."
      blocking: false
    - id: "S-005-main-wiring"
      subject: "S-005 main.rs wiring: 10s drain timeout + second-signal detection + signal-path lock release"
      status: pending
      detail: "S-005 graceful shutdown implemented at library level (BC-2.01.004 satisfied). Requires main.rs wiring for: (a) 10-second drain timeout enforcement, (b) second-signal detection (force-kill escalation), (c) signal-path lock release handoff. Deferred from S-005 scope (requires main.rs which belongs to TUI integration story). Route to integration story in Wave 3 or post-Wave-3."
      blocking: false
    - id: "S-008-ADV-tempfile-spec"
      subject: "AC-003 tempfile::persist spec wording divergence"
      status: partial-closed
      detail: "RingError #[non_exhaustive] fixed in-scope during wave-3-gate adversarial review (bef6f4b). Remaining: AC-003 tempfile::persist spec wording (story-writer); BC-2.01.007 S-TBD anchor (PO)."
      blocking: false
    - id: "S-015-ADV-tracing-test"
      subject: "tracing-test 0.2 not in SS-deps-pin-manifest"
      status: pending
      detail: "S-015 added tracing-test 0.2.6 with no-env-filter feature to monocle-runtime dev-deps. Not registered in SS-deps-pin-manifest.md. Architect must add."
      blocking: false
    - id: "S-012-self-ref-test-fix"
      subject: "3 self-referential tests fail — workspace root detection"
      status: closed-fixed-in-gate
      detail: "Fixed in ceebd2d (wave-3-gate): monocle_repo_root uses ancestors().find(.git) instead of cargo metadata. All 3 self-referential tests now pass. 447 tests total."
      blocking: false
    - id: "S-009-ADV-non-utf8"
      subject: "Non-UTF-8 canonical header treated as absent"
      status: accepted-observation
      detail: "Non-UTF-8 X-Monocle-Authorization falls through to alias path. HTTP headers are ASCII per RFC 7230; axum rejects non-ASCII before middleware. Zero security impact."
      blocking: false
    - id: "ADV-W3GATE-MED-001"
      subject: "last_hook_ts never written by hook handlers (future daemon wiring)"
      status: pending
      detail: "Wave 3 gate adversarial: DaemonState.last_hook_ts field exists but hook handler paths never write it. Observation-only — requires daemon wiring integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-002"
      subject: "Ring buffer DaemonState.ring never set to Some (main.rs stub)"
      status: pending
      detail: "Wave 3 gate adversarial: DaemonState.ring is always None; ring_buffer_fill_pct always returns 0.0. Requires main.rs wiring in integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-003"
      subject: "Only 1/5 hook endpoints tested in running-mode full-stack path"
      status: pending
      detail: "Wave 3 gate adversarial: Full-stack integration tests cover only pre-tool endpoint. 4/5 endpoints lack running-mode full-stack coverage. Phase 4 integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-004"
      subject: "ring_buffer_fill_pct hardcoded to 0.0"
      status: pending
      detail: "Wave 3 gate adversarial: Metric always returns 0.0 because DaemonState.ring is never set. Tracked with ADV-W3GATE-MED-002 (same root cause)."
      blocking: false
    - id: "SS-01-ProjectDirs-from"
      subject: "SS-daemon-lifecycle.md ProjectDirs::new → ProjectDirs::from"
      status: pending
      detail: "SS-daemon-lifecycle.md line 269 uses ProjectDirs::new('monocle','monocle','monocle') — should be ::from('','','monocle') per SS-config.md and BC-2.04.006 v1.5.0 correction. Architect maintenance item."
      blocking: false
    - id: "IMPL-EnrichedSession-fields"
      subject: "EnrichedSession implementation: add 4 TUI fields + serde derives"
      status: closed-pending-S-025-merge
      detail: "CLOSED (pending S-025 merge): Fields added in-scope via S-025 commit 9b84ef3 (architect Pass 1 Option C directive). Will be confirmed merged when S-025 PR merges to develop."
      blocking: false
    - id: "IMPL-HookDecision-serde"
      subject: "HookDecision + HookResponse: add Serialize/Deserialize derives"
      status: deferred-wave-5
      detail: "engine.rs HookDecision and HookResponse need Serialize/Deserialize. Required for IPC wire transport. Must be done as part of S-021 or S-018 implementation."
      blocking: false
    - id: "IMPL-on-hook-Defer"
      subject: "ClaudeCodeModule::on_hook() implement Defer routing logic"
      status: deferred-wave-5
      detail: "Current implementation returns Allow unconditionally (Phase 1 placeholder). S-018 (Hook Endpoint Routing) must implement the Defer path for permission overlay activation. BC-2.04.007 PC-3 specifies the full routing."
      blocking: false
    - id: "ADV-P1D-pin-staleness"
      subject: "Architecture Source pin staleness (cosmetic)"
      status: accepted-cosmetic
      detail: "SS-05 BCs pin SS-ipc.md v1.4.0 (current v1.6.0); SS-04 BCs pin v1.2.0 (current v1.3.0); SS-07 BCs pin v1.1.0 (current v1.3.0). Content is correct; only metadata pins lag. Will be swept at story implementation time."
      blocking: false
    - id: "ADV-W4GATE-MED-001"
      subject: "PATH env mutation in detect_ccr tests (test isolation)"
      status: closed
      detail: "CLOSED: Migrated to temp_env::with_vars in commit 295dc1b (S-023 PR #29, orchestrator-authorized scope expansion). Test isolation confirmed production-grade by Pass 5 adversary."
      blocking: false
    - id: "ADV-W4GATE-MED-002"
      subject: "tracing::error() no-ops in monocle CLI binary (no subscriber)"
      status: pending
      detail: "Wave 4 gate adversarial (P01): monocle CLI binary does not initialize a tracing subscriber, so tracing::error!() calls in main.rs are silent no-ops. Fix: initialize subscriber at binary entry or replace with eprintln! for startup errors. Sourced from ADV-W4GATE-P01-MED-002."
      blocking: false
    - id: "HS-EXP-009-hint"
      subject: "Exit 70 missing stderr remediation hint for MONOCLE_RUNTIME_DIR"
      status: pending
      detail: "Wave 4 gate holdout (HS-EXP-009 score 0.8): daemon emits exit code 70 for invalid MONOCLE_RUNTIME_DIR but provides no stderr hint to the user. BC-2.04.003 requires human-readable diagnostics. Fix: print actionable remediation message to stderr before exit. Sourced from HS-EXP-009 holdout evaluation."
      blocking: false
    - id: "F-ADV6-HIGH-001"
      subject: "S-022 slow-disconnect signal channel missing for subscribers"
      status: closed
      detail: "CLOSED: Production-grade slow-disconnect signal channel added to IPC subscribers via commit 9bddd7b in S-023 PR #29. Pass 5 adversary confirmed production-grade. Carry-over commit lineage annotated in PR #29 reviewer notes."
      blocking: false
    - id: "PROC-SEMGREP-DECOUPLE"
      subject: "Semgrep silently skipped for Waves 1-5 when Preflight failed-fast on protoc"
      status: pending
      detail: "24 stories (Waves 1-5) ran without Semgrep scan because CI Preflight failed-fast on protoc, silently skipping Semgrep. Decouple Semgrep from Preflight fast-fail chain so it runs independently. Route to devops-engineer. Target: Wave 7 or maintenance sweep."
      blocking: false
    - id: "PROC-GATE-SKIPPED-LOUD"
      subject: "Build+Test silently skipped for many prior CI runs — need GATE_SKIPPED loud indicator"
      status: pending
      detail: "CI gates that are skipped (due to upstream gate failure or missing tooling) should emit a loud GATE_SKIPPED log line so future reviewers know what ran vs was silently skipped. Route to devops-engineer. Target: Wave 7 or maintenance sweep."
      blocking: false
    - id: "PROC-COMPUTE-INPUT-HASH-YAML"
      subject: "bin/compute-input-hash does NOT handle YAML-object-style inputs (path/version pairs)"
      status: pending
      detail: "compute-input-hash parser treats YAML-object inputs as plain string values rather than structured path/version pairs. Discovered during S-023 cycle; fixed manually. Tooling gap remains — route to devops-engineer or dx-engineer for proper parser fix. Recurring class — occurred again during S-025 cycle. Target: Wave 7+ maintenance."
      blocking: false
    - id: "S-025-TODO-S023-MERGE"
      subject: "Replace 2 TODO(S-023-merge) markers after S-025 rebase onto develop"
      status: pending
      detail: "app.rs has TODO(S-023-merge) at lines 586-615 and 630. After S-025 is rebased onto develop (which includes S-023), replace with real monocle_ipc::events::TransportEvent + monocle_ipc::reconnect::reconnect_with_backoff imports. Instructions inline in the TODO blocks. Mechanical substitution — implementer task post-rebase."
      blocking: false
    - id: "S-025-MAKE-MODAL-DEAD-CODE"
      subject: "Dead make_modal test helpers — defer to S-026 dispatch"
      status: pending
      detail: "Pass 8 LOW finding: make_modal test helpers in monocle-tui are dead code now that permission overlay is deferred to S-026. Defer cleanup to S-026 dispatch when the overlay is implemented and the helpers become live. Confirmed LOW/non-blocking by Pass 8 adversary."
      blocking: false
    - id: "PROC-BRANCH-PROTECTION-CONTEXTS"
      subject: "Branch protection on develop has empty required-status-check contexts"
      status: pending
      detail: "develop branch protection rule has required-status-checks enabled but no specific check contexts configured (empty list). This means the rule is effectively a no-op — any CI status (or no CI) satisfies it. Requires admin escalation to configure required context names. Surface to human owner (Joshua Magady) for GitHub repo settings change."
      blocking: false
    - id: "F-S025-ADV13-NIT-003"
      subject: "BC-2.06.016 v1.0.8 §Trace line 230 stale 'Follow-up required' note (SS-tui propagation completed in architect commit 740465d)"
      status: pending
      detail: "BC-2.06.016 line 230 §Trace note reads 'Follow-up required (architect scope): SS-tui.md line 668 still cites prose form' — but SS-tui line 668 already uses bracketed form per architect commit 740465d. Note is stale. Routing: product-owner. Deferral rationale: cosmetic documentation aging; BC bump triggers full propagation chain (BC v1.0.9 → story-writer S-026 frontmatter bump → BC-INDEX/STORY-INDEX bump → consistency-validator re-run) for a §Trace text change. Anchored to Task #9 post-merge PO sweep."
      blocking: false
    - id: "F-S025-ADV13-NIT-004"
      subject: "BC-2.06.004 EC-079 line 104 cites non-production string 'Daemon offline' (production has DAEMON_NOT_RUNNING_ERROR full-screen panel for the AC-002 path)"
      status: pending
      detail: "BC-2.06.004 EC-079 (line 104): 'TUI starts but cannot connect to daemon; renders \"Daemon offline\" status message; no crash.' Production has DAEMON_NOT_RUNNING_ERROR (full-screen panel, AC-002 path) and DAEMON_OFFLINE_STATUS ('[daemon: offline]', S-023 reconnect-exhaust path). Neither is literally 'Daemon offline.' EC-079 is ambiguous; the path described uses DAEMON_NOT_RUNNING_ERROR (full-screen, NOT a status message). Routing: product-owner. Deferral rationale: EC-079 is informational (not a hard test contract); production behavior is correct. Anchored to Task #9 post-merge PO sweep."
      blocking: false
    - id: "F-S025-ADV16-PROC-001"
      subject: "[process-gap] agents must use `cargo clippy --workspace --all-targets -- -D warnings` to match CI Preflight"
      status: pending
      detail: "Pass 16 round 4 (test-writer): 4 clippy op_ref violations in startup_connect.rs color assertion loops (introduced by Pass 13 NIT-002 + Pass 14 NIT-001 fix rounds) were not caught locally because agents ran `cargo clippy --workspace -- -D warnings` without `--all-targets`. CI uses `--all-targets`. The `--all-targets` flag includes integration test crates; lib-only mode silently misses test-code violations. Codification target: next agent-prompt-refresh cycle or process-rules update. Recurrence count: 1."
      blocking: false
    - id: "F-S025-ADV16-PROC-002"
      subject: "[process-gap] scripts/audit-table.md vendored copy and SS-engine-module.md canonical must update in same PR"
      status: pending
      detail: "Pass 16 round 5 (architect): scripts/audit-table.md is a vendored copy of the audit table from SS-engine-module.md. When the canonical table changes, the vendored copy must be synced atomically in the same PR. The HookEventRecord crate-column drift (monocle-runtime → monocle-ipc, post-S-022 relocation) persisted unnoticed across 16 passes because the vendored copy was not included in propagation sweeps. Codification target: include in pre-commit hook or PR template checklist. Recurrence count: 1."
      blocking: false
    - id: "F-S025-ADV16-CODIFY-001"
      subject: "[S-7.02 codification trigger] F-R30-1 recurrence count crossed 3 (now 4). Codify audit-table sweep + MSRV-bump playbook discipline (extended D-196, D-197, FINALIZED D-197.1, GENERALIZED D-198, REFINED D-198.1, CANONICAL-ANCHORED D-198.2)."
      status: pending
      detail: "Pass 16 round 6 (D-193): F-R30-1 recurrence count crossed threshold (4 rows total: App + EventBusHookEvent + EngineModuleRegistry + BackoffState). S-7.02 codification REQUIRED. Codify in CLAUDE.md or VSDD.md: 'When a new crate is added or merged from a separate branch, the architect MUST run git ls-tree <merge-base>..HEAD + per-file #[non_exhaustive] pub struct sweep before declaring audit-table sync complete.' EXTENDED (D-196) per PO + story-writer process-gap observation: MSRV-bump playbook scope must include ALL layers — architecture/ (SS docs, ADRs, risk-acceptance), behavioral-contracts/ (BC bodies), stories/ inputs[] pins AND body content, planning artifacts (holdout-scenarios.md HS-* scenarios). Verification sweep command: grep -rn \"MSRV X.YY\\|Rust X.YY stable\\|channel = \\\"X.YY\\\"\\|rust-version = \\\"X.YY\\\"\" .factory/ | grep -v \"§Trace\". Success criterion: zero non-§Trace hits remaining. FINALIZED (D-197.1) — MSRV-bump playbook + version-pointer-sweep playbook 6-category target enumeration (codified from F-S025-ADV18-MED-001 closure 9fcfd49): (1) .factory/ spec artifacts (.md files) [architect f3533ce sweep covered this]; (2) Root Cargo.toml + deny.toml [devops 9fcfd49 sweep target]; (3) Member crate Cargo.toml files (under crates/ and xtask/) [devops 9fcfd49 sweep target]; (4) .github/workflows/**.yml files [devops 9fcfd49 sweep target]; (5) .github/dependabot.yml [devops 9fcfd49 sweep target]; (6) src/**/*.rs production code (comments, panic messages, doc strings) [devops 9fcfd49 wider-sweep gap discovery — crates/monocle-runtime/src/types.rs:48 carried v1.1.20, hidden from Passes 16+17 because they searched .toml/.yml only]. GENERALIZED (D-198) — F-S025-ADV19-MED-001 (SS-conventions-anti-patterns v1.30.2 in clippy.toml+deny.toml) proves the 6-category sweep was doc-name-specific (SS-deps-pin-manifest only). REFINED (D-198.1) — devops 0aba808 adjudication-protocol-driven sweep is the concrete codification template. After ANY canonical policy/spec doc minor-or-patch version bump (SS-deps-pin-manifest, SS-conventions-anti-patterns, SS-engine-module, SS-ipc, SS-tui, SS-config, SS-daemon-wiring, ARCH-INDEX, PRD, etc.), sweep ALL implementation-worktree files for active-pointer citations matching the doc-name pattern. Canonical sweep command: grep -rn -E '(SS-[a-z-]+|ARCH-INDEX|BC-INDEX|STORY-INDEX|VP-INDEX|prd|product-brief)\\.md.*v[0-9]+\\.[0-9]+' --include='*.toml' --include='*.yml' --include='*.rs' --include='*.py' --include='*.md' . | grep -v '\\.factory/\\|target/\\|node_modules/\\|\\.git/\\|cycles/'. For each hit, classify: CATEGORY A — ACTIVE POINTER (FIX, bump to canonical): 'Source: <Doc>.md v<X>'; 'Policy source of truth: <Doc>.md v<X>'; 'Pin policy source of truth: <Doc>.md v<X>'; 'See <Doc>.md v<X> §<section>' (implies look at current spec); 'Per <Doc>.md v<X>, <current-behavior-claim>' (no F-D-NN tag, no line anchor); 'conformance to <Doc>.md v<X>'; module-doc citations as implementation source; bare top-of-file version cites without §section or line anchor. CATEGORY B — HISTORICAL ANCHOR (PRESERVE): 'per <Doc>.md v<X> §<section> (F-D-NN)' — F-D-NN finding tag; 'per <Doc>.md v<X> §<section> (lines NNN-NNN)' — explicit line anchor; 'first specified in <Doc>.md v<X>'; '<Doc>.md v<X> §Trace'; 'introduced in v<X>' / 'as of v<X>' — when-anchored. CATEGORY EDGE (judgment call): Citations with §section anchor but no F-D-NN tag — verify §section content unchanged between cited version and canonical. If unchanged → Category B. If wording refined → Category A. If semantics changed → CRITICAL flag. Verification: zero non-historical-anchor stale pointers. Template commit: devops 0aba808 commit body documents per-site adjudication for all 5 active-pointer sites + 8+ historical-anchor preservations. Recurrence count: F-R30-1 = 4/3 (audit-table) + F-S025-ADV17-LOW-001 = 1/3 (MSRV-bump scope) + F-S025-ADV18-MED-001 = 1/3 (version-pointer-sweep impl-layer) + F-S025-ADV19-MED-001 = 1/3 (sibling-doc generalization) + F-S025-ADV20-MED-001 = 1/3 (sibling-version orphaned-intermediate). CANONICAL-ANCHORED (D-198.2) — F-S025-ADV20-MED-001 (engine.rs:143 v1.1.22 orphaned-intermediate between Pass 19 floor v1.1.20 and canonical v1.1.26) proves D-198.1 stale-literal sweep was insufficient. Sweep regex must be CANONICAL-ANCHORED: extract cited_version from each hit, compare against canonical from .factory/specs/architecture/<Doc>.md frontmatter. Any cited_version != canonical_version is Category A regardless of how many intermediate versions exist. Canonical-anchored sweep command (D-198.2): grep -rn -E '(SS-[a-z-]+|ARCH-INDEX|BC-INDEX|STORY-INDEX|VP-INDEX|prd|product-brief|dtu-assessment)\\.md( +|\\s+v|v)[0-9]+\\.[0-9]+(\\.[0-9]+)?' --include='*.toml' --include='*.yml' --include='*.rs' --include='*.py' . | grep -v '\\.factory/\\|target/\\|node_modules/\\|\\.git/\\|cycles/'. For each hit: extract cited_version; look up canonical_version from frontmatter; if cited_version != canonical_version AND no F-D-NN/line-anchor/§Trace → Category A (fix). This supersedes D-198.1 stale-literal approach for ALL future sweep dispatches. Template commit: devops ef7f4c62 (72 citation hits; 9 Category B preserved; 62 canonical-match; 1 Category A bump). Pattern codified. Anchored to Task #9 post-merge sweep, batched with story-writer for follow-up story creation. 7th SWEEP CATEGORY (D-201 — TD-S025-PASS22-PROC-001): BARE-FILENAME ARCHITECTURE ANCHOR RESOLUTION AUDIT. For every SS-*.md / ADR-NNNN-*.md / BC-N.NN.NNN.md reference (versioned OR unversioned) in worktree code, story specs, and architecture docs — verify the file exists at canonical location: SS-*.md at .factory/specs/architecture/<name>.md; ADR-NNNN-*.md at .factory/specs/architecture/adr/<name>.md; BC-N.NN.NNN.md at .factory/specs/behavioral-contracts/ss-NN/<name>.md. Sweep command: grep -rh -oE 'SS-[a-z-]+\\.md|ADR-[0-9]+-[a-z-]+\\.md|BC-[0-9]+\\.[0-9]+\\.[0-9]+\\.md' --include='*.toml' --include='*.yml' --include='*.rs' --include='*.py' --include='*.md' . 2>/dev/null | grep -v '/cycles/' | sort -u — then for each unique reference, verify file exists at canonical location. Distinguish active references (FIX) from historical narrative (PRESERVE). Codification candidate: build.rs script or semgrep rule for CI enforcement. Root cause of D-201: D-198.2 canonical-anchored sweep audits versioned patterns (SS-X.md vY.Z) but bare-filename references (SS-tui-core.md, no version) slip through entirely — a META-GAP in the sweep protocol itself."
      blocking: false
    - id: "F-S025-ADV22-MED-001"
      subject: "SS-tui-core.md broken anchor in 2 worktree files + 7 EPIC-06 story files (canonical: SS-tui.md)"
      status: closed
      detail: "CLOSED (D-201.1): implementer e5ebc43 (feature/S-025-tui-skeleton-sessions) — lib.rs:7 + Cargo.toml:19 fixed; sweep verified all 9 unique SS-*.md references in code; only SS-tui-core.md was broken. Story-writer 0cea089 (factory-artifacts) — 7 EPIC-06 story files (S-024 v1.5, S-025 v1.7, S-026 v1.8, S-027 v1.4, S-028 v1.3, S-029 v1.2, S-031 v1.1) + STORY-INDEX v5.12. BONUS: story-writer sweep-wider caught SS-forward-compat.md (non-existent) at SS-conventions-anti-patterns.md:1043 — fixed in-scope per CLAUDE.md Principle 4; SS-conventions v1.31.0 → v1.31.1. CI 9/9 green on e5ebc43 (Preflight, DTU, Semgrep, audit-table-drift, 3x Build+Test, cargo deny, cargo audit). Final-verification grep: zero active-content broken SS-tui-core.md references."
      blocking: false
    - id: "F-S025-ADV22-PROC-001"
      subject: "[process-gap] CI-enforced bare-filename architecture anchor resolution check (build.rs or semgrep)"
      status: pending
      detail: "Pass 22 META-GAP: D-198.2 canonical-anchored sweep catches versioned SS-X.md vY.Z references but NOT bare-filename references (SS-tui-core.md) that fail to resolve. Need CI enforcement: enumerate all SS-*.md / ADR-NNNN-*.md references in worktree, verify each resolves to a real file at canonical path. Implementation candidates: build.rs script or semgrep rule. Anchored to Task #9."
      blocking: false
    - id: "F-S025-ADV20-PROC-001"
      subject: "[process-gap] SS-conventions 'Test File Documentation Standards' rule — spec version citations in test files need disambiguation anchor"
      status: pending
      detail: "Pass 20 test-writer Option B adjudication surfaces a codification gap: spec version citations in test/production file doc comments must carry one of (1) F-D-NN fix tag, (2) §section anchor, (3) parenthetical '(TDD red-gate authoring baseline; current canonical is vX.Y.Z)' or equivalent disambiguation. Bare version numbers without anchors are flagged as Category B smell during canonical-anchored sweep. Engine_module_surface.rs:6-8 was the concrete instance (v1.1.20 bare → Option B anchor added in dc229db). Routing: architect (SS-conventions-anti-patterns update). Anchored to Task #9 SS-conventions update post-S-025 merge."
      blocking: false
    - id: "F-S025-PATH-B-CLAUDE-MD"
      subject: "CLAUDE.md line 18 cites MSRV 1.86; Path B bumped Phase 1 MSRV to 1.88 — human action required"
      status: pending
      detail: "CLAUDE.md line 18 reads: 'MSRV: Phase 1 = Rust 1.86 (ratatui 0.30 floor). Phase 3 = Rust 1.92 (wasmtime 44 requirement).' Architect Path B work (D-194) bumped Phase 1 MSRV to 1.88 (time 0.3.47 floor per RUSTSEC-2026-0009 mitigation). CLAUDE.md is human-maintained (outside agent write scope). Human action: update line 18 to: 'MSRV: Phase 1 = Rust 1.88 (time 0.3.47 floor per RUSTSEC-2026-0009 mitigation; original ratatui 0.30 floor was 1.86). Phase 3 = Rust 1.92 (wasmtime 44 requirement).' Anchored to next human review; non-blocking for S-025."
      blocking: false
    - id: "F-S025-ADV17-LOW-001"
      subject: "BC-2.03.001 stale 'MSRV 1.86 stable Rust' reference — FULLY CLOSED (D-196)"
      status: closed
      detail: "FULLY CLOSED (D-196): PO commit 5006528 (BC-2.03.001 v1.0.6) replaced lines 35+61. Story-writer cascade round 1 (c7ae560): S-014 v1.5 + S-015 v1.7 + STORY-INDEX v5.10. Story-writer cascade round 2 (e2944d3): S-001 v1.9 + S-003 v1.8 + holdout-scenarios v1.5 + STORY-INDEX v5.11. Final grep confirms ZERO non-§Trace 'MSRV 1.86' hits across all .factory/ artifacts. Counter 1/3 CONFIRMED. Pass 18 ready at bfcba19."
      blocking: false
    - id: "F-S025-ADV18-MED-001"
      subject: "Path B propagation cascade tail-gap: SS-deps-pin-manifest v1.1.19 doc-pointers in implementation-worktree files"
      status: closed
      detail: "CLOSED (D-197.1) at devops commit 9fcfd49 on feature/S-025-tui-skeleton-sessions. Pre-fix count: 17 occurrences (Pass 18 reported) + 1 additional (crates/monocle-runtime/src/types.rs:48 v1.1.20, found by devops wider sweep — hidden from Passes 16+17 which searched .toml/.yml only, not production .rs). Total replacements: 18 across 11 files. Local: cargo build/test/clippy/fmt clean. CI: queued/running at time of push. Counter 0/3; Pass 19 pending CI green."
      blocking: false
    - id: "F-S025-ADV19-MED-001"
      subject: "Path B sibling-doc tail-gap: SS-conventions-anti-patterns.md v1.30.2 stale active pointers (clippy.toml + deny.toml)"
      status: closed
      detail: "CLOSED (D-198.1) at devops commit 0aba808 on feature/S-025-tui-skeleton-sessions. 5 total active-pointer bumps: clippy.toml:2 (SS-conventions-anti-patterns v1.30.2→v1.31.0), deny.toml:1 (SS-conventions-anti-patterns v1.30.2→v1.31.0) — Pass 19 MED-001 targets; engine.rs:4 (SS-engine-module v1.1.20→v1.1.26), engine_module_surface.rs:1197 (SS-engine-module v1.1.20→v1.1.26), engine_module_surface.rs:1223 (SS-engine-module v1.1.20→v1.1.26) — devops adjudication-protocol-driven wider-sweep catches (Pass 19 adversary missed). 8+ Category B historical anchors preserved with per-site rationale documented in 0aba808 commit message. 4 other canonical docs verified clean (SS-deps-pin-manifest v1.2.0, SS-daemon-lifecycle v1.0.33, SS-core-types-and-abi v1.2.13, SS-forward-compatibility v1.2.19). CI: 8/10 SUCCESS at snapshot (Preflight, DTU, Semgrep, audit-table-drift, 2/3 Build+Test); macOS Build+Test + cargo deny + cargo audit pending."
      blocking: false
    - id: "F-S025-ADV20-MED-001"
      subject: "engine.rs:143 SS-engine-module v1.1.22 stale active pointer (orphaned intermediate version)"
      status: closed
      detail: "CLOSED (D-199) at devops commit ef7f4c62 on feature/S-025-tui-skeleton-sessions. engine.rs:143 cited 'SS-engine-module.md v1.1.22' — bare active pointer (no F-D-NN, §-anchor, line-anchor); orphaned intermediate version between Pass 19's targeted floor (v1.1.20) and canonical ceiling (v1.1.26). 1 Category A bump applied: engine.rs:143 v1.1.22→v1.1.26. Canonical-anchored sweep: 72 total citation hits across all worktree .rs/.toml/.yml/.md files; 9 Category B preserved; 62 canonical-match; 8 canonical docs verified (SS-engine-module v1.1.26, SS-ipc v1.9.0, SS-daemon-lifecycle v1.0.33, SS-core-types-and-abi v1.2.13, SS-deps-pin-manifest v1.2.0, SS-forward-compatibility v1.2.19, SS-conventions-anti-patterns v1.31.0, dtu-assessment v1.7.5). Convergence-stall species RESOLVED: sweep now catches ALL Doc.md vX.Y.Z cites in one pass. CI pending 10/10."
      blocking: false
    - id: "F-S025-ADV20-LOW-001"
      subject: "engine_module_surface.rs:6-8 Red-Gate v1.1.20 bare citation (pending-intent disambiguation)"
      status: closed
      detail: "CLOSED (D-199) at test-writer commit dc229db on feature/S-025-tui-skeleton-sessions. Option B adjudication: engine_module_surface.rs:6-8 disambiguated with parenthetical anchor 'per SS-engine-module.md v1.1.20 (TDD red-gate authoring baseline; current canonical is v1.1.26)'. Justified via comparison: engine.rs:4 + lines 1197/1223 at v1.1.26 (current pointers); lines 548/581/594/624 at v1.1.20 with F-D-NN anchors; bare v1.1.20 at line 8 was the lone outlier. Process-gap recommendation surfaced as ADV20-PROC-001."
      blocking: false
  se_candidates:
    - id: SE-40
      occurrences: 2
      threshold: 3
      description: "Orchestrator drives deliver-story from main session only; never delegates to sub-orchestrator that cannot spawn fresh-context specialists."
      status: HELD per D-114
  process_discoveries:
    - "Tautological-test risk pattern: test-writer compared fixture.clone() to fixture (identity = false-green)"
    - "Single-giant-commit-hides-todo: implementer R1 claimed completion but left todo!() in binary entrypoint"
    - "Production-Grade Default Rule 1 'Future:' comment violations caught 3x"
    - "Sibling-sweep gaps in 3-place status tracking (sprint-state/STORY-INDEX/story-frontmatter)"
    - "clippy --all-targets vs --workspace scope gap (test code violations invisible in lib-only mode)"
    - "Semgrep silently skipped for Waves 1-5 (24 stories) when Preflight failed-fast on protoc; decouple recommended (PROC-SEMGREP-DECOUPLE)"
    - "Build+Test silently skipped for many prior CI runs; need GATE_SKIPPED loud indicator (PROC-GATE-SKIPPED-LOUD)"
    - "Architect-decision binding propagation to spec bodies requires dedicated orchestrator dispatches — the BC sweep can be incomplete if the architecture SOURCE document (SS-tui) is missed"
    - "Pass 4 S-025 adversary found 2 BLOCKERs missed by Passes 1-3; novelty-spike pattern confirms multi-pass convergence rigor and the 3-consecutive-NITPICK_ONLY threshold"
    - "bin/compute-input-hash does NOT handle YAML-object-style inputs (path/version pairs) — fixed manually but tooling gap remains (PROC-COMPUTE-INPUT-HASH-YAML)"
    - "S-025 adversarial cycle (11 passes): architect-decision propagation requires DEDICATED dispatches sweeping ALL artifacts — BC bodies, SS docs, story frontmatter pins, story body language, input-hashes. Missing any one creates Pass N+M findings (L-W6-S025-001)"
    - "S-025 Pass 9 sweep was vacuous-mirror: assertion on TestBackend-local canonical_msg, NOT production-rendered path. Sweep-audit must trace assertion to EXACT production code path (L-W6-S025-002)"
    - "pub const extraction is the production-grade default for canonical strings shared by production + tests — eliminates vacuous-mirror class structurally (L-W6-S025-003)"
    - "Multi-pass convergence: premature-clean signal at counter-advance moments confirmed (Pass 8 clean → Pass 9 NEW class). 3-consecutive-NITPICK_ONLY rule exists precisely to catch class-sibling regressions (L-W6-S025-004)"
    - "IEEE 754: -0.0 == 0.0 so cost < 0.0 does not catch negative zero; is_sign_negative() required. NaN check must come FIRST in guard chains (L-W6-S025-005)"
    - "Architect-decision propagation missed SS-tui in Pass 5 because routing assigned SS to architect but SS-tui was overlooked during BC sweep; SS docs are ALSO propagation targets (L-W6-S025-006)"
    - "Production-grade sweep should expand BEYOND the flagged targets — Pass 11 implementer found 3 additional class siblings; Pass 7 found 2 additional. CLAUDE.md Principle 4 (fix in scope) implies sweep-wider-than-the-finding (L-W6-S025-007)"
    - "Stale-literal-anchored sweep is insufficient for version citation staleness — orphaned intermediate versions (between targeted floor and canonical ceiling) are invisible to regex targeting known stale literals. CANONICAL-ANCHORED sweep required: extract cited_version per hit, compare vs frontmatter canonical (D-198.2). 5 consecutive convergence stalls in same species before structural resolution."
    - "Test file doc comment spec version citations need explicit disambiguation anchor (F-D-NN tag, §section, or parenthetical) — bare version numbers in test files are indistinguishable from Category A active-pointers during automated sweep (ADV20-PROC-001)"
    - "META-PATTERN (D-201): S-025 convergence has failed 4× at the 1/3→2/3 transition. Each failure is a DIFFERENT class ONE LAYER DEEPER in the stack: Pass 9 = vacuous-mirror (test assertion layer); Pass 16 = audit-table/op_ref (struct-metadata layer); Pass 18 = version-pointer propagation (implementation-worktree layer); Pass 22 = bare-filename anchor resolution (spec-filename layer). This is not recurrence within a bounded species — it is a genuine escalation ladder where each sweep improvement reveals the next blind spot. Two-sweep protocol now required: (1) D-198.2 canonical-anchored versioned sweep; (2) D-201 bare-filename existence-check sweep. Anticipate that Pass 23 may reveal a new class if the two-sweep protocol still has a blind spot."
next_session_resume_protocol: |
  S-025 PASS 22 MED REMEDIATED (D-201.1) — COUNTER 0/3 — PASS 23 DISPATCHED — STATE v6.49

  STATE: develop @ 7a52041. 26/33 done (156/195 pts). 852+ tests. S-025 branch HEAD e5ebc43 on feature/S-025-tui-skeleton-sessions.
  COUNTER: 0/3 (reset at Pass 22 MED; REMEDIATED via e5ebc43 + 0cea089). Pass 23 in flight.
  TRAJECTORY: 5→4→3→2→4→H→M→0→M→M→H→C→L→N(14)→C(15)→M(16)→N(17)→M(18)→M(19)→M(20)→C(21)→M(22)→REMEDIATED.
  TOTAL RESETS: 7 (Passes 8, 9, 10, 12, 16, 18, 22). CONVERGENCE-ATTEMPT #6 ACTIVE.
  MSRV: Phase 1 = 1.88 (time 0.3.47 floor). Phase 3 = 1.92. CLAUDE.md line 18 needs human update (F-S025-PATH-B-CLAUDE-MD).

  PASS 22 OUTCOME (D-201) — REMEDIATED (D-201.1):
    VERDICT: MED — F-S025-ADV22-MED-001: SS-tui-core.md cited in 9 files but document does not exist.
    REMEDIATION: COMPLETE. implementer e5ebc43 (2 worktree sites) + story-writer 0cea089 (7 story sites + BONUS).
    BONUS CATCH (0cea089): SS-conventions-anti-patterns.md:1043 cited SS-forward-compat.md (non-existent);
      canonical is SS-forward-compatibility.md. Story-writer fixed in-scope per CLAUDE.md Principle 4 sweep-wider.
      SS-conventions v1.31.0 → v1.31.1.
    CI: 9/9 green on e5ebc43 (Preflight, DTU, Semgrep, audit-table-drift, 3x Build+Test, cargo deny, cargo audit).
    COUNTER: 1/3 → 0/3 (MED reset) → holding 0/3. Pass 23 dispatched at post-fix HEAD e5ebc43.
    PATH-B-PROPAGATION SPECIES: BOUNDED (no recurrence). Tripwire NOT FIRED (different class).
    DEFECT CLASS: "Bare-filename architecture anchor resolution" — META-GAP in D-198.2 canonical-anchored sweep.
    4TH 1/3→2/3 TRANSITION FAILURE: Pattern = "each 1/3 transition surfaces NEW-class defect ONE LAYER DEEPER."
      Pass 9: vacuous-mirror (test layer). Pass 16: audit-table+op_ref (struct-metadata layer).
      Pass 18: version-pointer propagation (implementation-worktree layer). Pass 22: bare-filename resolution (spec-filename layer).
    META-PATTERN CODIFIED: Genuine escalation ladder, not recurrence within a bounded species.
    CODIFY-001 EXTENDED with 7th sweep category + PROVEN canonical sweep command (story-writer 0cea089).
    ADV22-PROC-001 ADDED: CI-enforced bare-filename anchor resolution check.

  IMMEDIATE NEXT ACTIONS:
    1. AWAIT Pass 23 adversary result. Focus: all Pass 1-22 axes + NEW angle (bare-filename anchor resolution).
       Target: NITPICK_ONLY-CLEAN → counter 0/3 → 1/3.
    2. META-pattern watch: does a 5th new-class find surface? Anticipate if two-sweep protocol still has blind spots.
    3. Convergence forecast: 3/3 at Pass 25 if Passes 23+24+25 all NITPICK_ONLY-CLEAN.

  CRITICAL FILES FOR PASS 23 ADVERSARY (read in order):
    1. .factory/STATE.md (v6.49 post-fix); 2. adversarial-pass-22.md; 3. adversarial-pass-21.md;
    4. adversarial-pass-20.md; 5. adversarial-pass-19.md; 6. adversarial-pass-18.md;
    7. adversarial-pass-16.md; 8. adversarial-pass-15.md; 9. adversarial-pass-12.md;
    10. architect-decisions-pass-1.md; 11. architect-decisions-pass-2.md;
    12. text-style-adjudication.md; 13. red-gate-log.md;
    14. .factory/stories/S-025-tui-skeleton-sessions.md (v1.7);
    15. .factory/specs/behavioral-contracts/ss-03/BC-2.03.001.md (verify v1.0.6 correct);
    16. CLAUDE.md (project principles — production-grade default).
    All cycle files at: .factory/cycles/cycle-001/S-025/

  ADV16-CODIFY-001 7-CATEGORY SWEEP (D-198.2 + D-201 + D-201.1 PROVEN):
    CATEGORY 1-6 (CANONICAL-ANCHORED): grep versioned SS-X.md vY.Z hits; extract cited_version vs canonical.
      grep -rn -E "(SS-[a-z-]+|ARCH-INDEX|BC-INDEX|STORY-INDEX|VP-INDEX|prd|product-brief|dtu-assessment)\.md( +|\s+v|v)[0-9]+\.[0-9]+(\.[0-9]+)?" \
        --include='*.toml' --include='*.yml' --include='*.rs' --include='*.py' \
        . | grep -v "\.factory/\|target/\|node_modules/\|\.git/\|cycles/"
    CATEGORY 7 (BARE-FILENAME RESOLUTION — D-201, PROVEN D-201.1): grep ALL SS-*.md / ADR-NNNN-*.md / BC-N.NN.NNN.md bare refs; verify each resolves.
      Proven sweep command (story-writer 0cea089 burst):
        grep -rh -oE "SS-[a-z-]+\.md|ADR-[0-9]+-[a-z-]+\.md|BC-[0-9]+\.[0-9]+\.[0-9]+\.md" \
          .factory/ 2>/dev/null | sort -u
      For each reference, verify file exists at canonical location.
      Distinguish active references (FIX) from historical narrative (PRESERVE).
      Historical narrative includes:
        - /cycles/ directory (cycle reports, decisions-archive)
        - §Trace entries documenting old→new migrations
        - /plans/ historical audit reports
        - spec body placeholder examples (e.g., "SS-foo.md")
      Proven catch (0cea089): SS-conventions-anti-patterns.md:1043 cited SS-forward-compat.md (non-existent);
        canonical is SS-forward-compatibility.md. BONUS fix → SS-conventions v1.31.1.

  RECURRENCE WATCH:
    Path-B-propagation defect species: BOUNDED (Pass 21 D-198.2 validated; no recurrence at Pass 22).
    Bare-filename-anchor species (new): FIRST INSTANCE at Pass 22. Fix-round dispatched.
    1/3→2/3 transition failure count: 4 (Pass 9 MED, Pass 16 MED, Pass 18 MED, Pass 22 MED).
    META-PATTERN: Each 1/3 transition fires a NEW-class defect ONE LAYER DEEPER in the stack.
      This pattern itself is now codified — see process_discoveries (last entry).

  ARTIFACT VERSIONS (D-201.1 / Pass 23 entry point — post-fix, confirmed):
    BC-INDEX v1.27 (113 BCs). PRD v1.27.3. SS-engine-module v1.1.26. SS-deps-pin-manifest v1.2.0.
    SS-conventions-anti-patterns v1.31.1. SS-tui v1.8.2. ADR-0006 v1.2. S-025 v1.7. S-026 v1.8.
    BC-2.03.001 v1.0.6. S-001 v1.9. S-003 v1.8. S-014 v1.5. S-015 v1.7.
    S-024 v1.5. S-027 v1.4. S-028 v1.3. S-029 v1.2. S-031 v1.1.
    holdout-scenarios v1.5. STORY-INDEX v5.12.
    Post-fix: implementer e5ebc43 (feature/S-025-tui-skeleton-sessions) + story-writer 0cea089 (factory-artifacts). CI 9/9 green.

  AFTER CONVERGENCE (3/3 NITPICK_ONLY-CLEAN):
    Rebase S-025 → develop. Resolve TODO(S-023-merge) at app.rs:586-615+630. Demo-recorder (10 ACs).
    PR-manager (PR #28 draft → merge). State-manager D-187 closure. Dispatch S-026 (13 pts).
    Task #9 post-merge: F-S025-ADV16-CODIFY-001 (story-writer + CLAUDE.md incl. D-198.2+D-201 protocols),
      ADV20-PROC-001 (architect SS-conventions update), ADV22-PROC-001 (devops CI anchor check), NIT-003/004 (PO).

  KEY LESSONS:
    L-001: Propagation sweeps = BC bodies + SS docs + story fm + body + input-hashes + worktree policy-pointer comments.
    L-002: Assertion must trace to EXACT production code path (not TestBackend-local copy).
    L-003: pub const extraction eliminates vacuous-mirror class structurally.
    L-004: Premature-clean signal vigilance at ALL counter positions (not just 0/3 floor).
    L-007: Sweep wider than the finding — catch ALL class siblings at EVERY architectural layer.
    L-NEW (D-198.1): Devops adjudication-protocol-driven sweep catches Category A sites that adversary misses.
    L-NEW (D-198.2): Stale-literal-anchored sweep is insufficient — CANONICAL-ANCHORED sweep required.
    L-NEW (D-201): Canonical-anchored sweep audits versioned refs only — bare-filename refs (no version) require
      a SEPARATE existence-check sweep. Two-sweep protocol now required: (1) versioned D-198.2; (2) bare-filename D-201.

  FACTORY: .factory/ on factory-artifacts. Run factory-worktree-health first. NEVER --no-verify.
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: pending
dtu_services: [hook-endpoints-x5]
current_cycle: cycle-001
---

# Pipeline State: Monocle — ZERO-CONTEXT RESUME GUIDE

## Phase Progress

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| -1 Reference Ingest | DONE | 2026-05-11 | 8 repos; semport/ |
| 0.5-0.9 Brief + arch stubs | DONE | 2026-05-14 | |
| Pre-Phase-1 Final Gate | DONE | 2026-05-14 | D-054. 26 adv rounds. 22 BCs. |
| 1 Spec Crystallization | DONE (expansion complete, D-169 APPROVED) | 2026-05-27 | D-155 original gate. D-168: PRD 22→70 BCs. D-169: Phase 1d CONVERGED (15 passes, trajectory 15→0). D-170: human gate APPROVED. BC-INDEX v1.19 (112 BCs). |
| 2 Story Decomposition | DONE (D-173 APPROVED) | 2026-05-27 | D-159 original gate: 17 stories, 86 pts. D-170: re-entry for 48 new BCs. D-171: 16 stories (S-016..S-031, 109 pts) + 10 holdout scenarios (HS-EXP-001..010) produced. Total: 33 stories, 195 pts. D-172: adversarial story review 4 passes, trajectory 18→11→9→4 (0 CRIT/HIGH at Pass 4). D-173: human gate APPROVED. BC-INDEX v1.23 (113 BCs). STORY-INDEX v4.7. |
| 3 TDD Implementation | IN PROGRESS — Wave 6 2/4 done; S-025 Pass 22 REMEDIATED (D-201.1); counter 0/3; Pass 23 dispatched | 2026-05-28 | Wave 1+2+3 DONE (83 pts, 447 tests, all 6 gates). Wave 4 GATE PASSED (D-175): 634 tests. Wave 5 GATE PASSED (D-182): 753 tests, 0 failures, clippy clean, fmt clean. Wave 6: 2/4 done (S-022 8pts + S-023 5pts). 26/33 stories done (156/195 pts). S-025 MED-001 CLOSED: e5ebc43 (2 worktree sites) + 0cea089 (7 stories + BONUS SS-forward-compat.md → SS-conventions v1.31.1). CI 9/9 green. Pass 23 dispatched; convergence-attempt #6. S-026 blocked on S-025. Trajectory: 5→4→3→2→4→H→M→0→M→M→H→C→L→N(14)→C(15)→M(16)→N(17)→M(18)→M(19)→M(20)→C(21)→M(22). |
| 4-7 | not-started | — | |

## Wave 5 — GATE PASSED (D-182)

| Story | Points | Status | Notes |
|-------|--------|--------|-------|
| S-017 Daemon Start Sequence + Hook Tmpfile | 8 | done | PR #22, 06432cf, 29 tests, adv 13→5→0 (3 passes) |
| S-018 Hook Endpoint Routing + Event Bus | 8 | done | PR #26, 654e281, 46 tests, adv 10→4→4 (CONVERGED) |
| S-019 Daemon Auto-Start + MONOCLE_NO_AUTOSTART | 5 | done | PR #25, 11540fc, 25 tests, adv 7→2→1 (CONVERGED) |
| S-020 JSONL Ring Capacity and Rotation | 5 | done | PR #24, f69d53a, 24 tests, adv 12→8→0 (CONVERGED) |
| S-021 UDS Server + IPC Transport + Core Message Types | 8 | done | PR #23, acaacb9, 49 tests, adv 9→4→4 (CONVERGED) |

develop @ 7a52041. 852+ tests, 0 failures. 26/33 stories done, 156/195 pts (80%). Wave 5 gate PASSED (D-182). Wave 6 in progress: S-022 DONE (D-184, PR #27 @ c7540539), S-023 DONE (D-186, PR #29 @ 7a52041). S-025 Pass 22 REMEDIATED (D-201.1): F-S025-ADV22-MED-001 CLOSED at e5ebc43 + 0cea089. BONUS: SS-conventions v1.31.1. CI 9/9 green. Pass 23 dispatched; convergence-attempt #6; counter 0/3. S-026 blocked on S-025. Resets: Pass 8, 9, 10, 12, 16, 18, 22 = 7 total.

## Blocking Issues

None. All durable_task_register items non-blocking.

## Decisions Log (recent — D-188 through D-195)

D-047 through D-187 archived at: `cycles/cycle-001/decisions-archive.md`

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-188 | S-025 Pass 12 CRITICAL — App.status_message write-only (render_frame never consumes); vacuous-mirror class L-W6-S025-002. Counter RESET 0/3. F-S025-CI-001 fix landed (07e207b). | 2026-05-28 | state-manager |
| D-189 | S-025 Pass 13 LOW — 2 LOW + 4 NITPICK. Pass 12 fix verified. Counter holds 0/3. Fix dispatched. | 2026-05-28 | state-manager |
| D-190 | S-025 Pass 14 NITPICK_ONLY — DarkGray baseline branch missing render+color test. Counter holds 0/3. Fix dispatched. | 2026-05-28 | state-manager |
| D-191 | S-025 Pass 15 NITPICK_ONLY-CLEAN — zero findings. Counter ADVANCES 0/3 → 1/3. Pattern decay confirmed. | 2026-05-28 | state-manager |
| D-192 | S-025 Pass 16 MED — ADR-0006 audit-table gap (App+EvBus+EMR+BackoffState) + false-green CI + 4 op_ref + vendored copy. 5-round fix. SS-engine-module v1.1.22→v1.1.25. Counter RESET 1/3 → 0/3. | 2026-05-28 | state-manager |
| D-195 | S-025 Pass 17 NITPICK_ONLY-CLEAN — 1 LOW (BC-2.03.001 MSRV 1.86 stale ref; PO fix landed 5006528 as BC-2.03.001 v1.0.6). Zero CRITICAL/HIGH/MED. Counter 0/3 → 1/3 CONFIRMED. Pass 18 ready at bfcba19. | 2026-05-29 | state-manager |
| D-197 | S-025 Pass 18 MED — F-S025-ADV18-MED-001: Path B propagation cascade tail-gap in implementation-worktree layer (17 occurrences across 10 files still pin SS-deps-pin-manifest v1.1.19). Counter RESET 1/3 → 0/3. ADV16-CODIFY-001 extended with 6th sweep target (implementation-worktree policy-pointer comments). Devops fix-round dispatched in parallel. Pass 19 pending post-fix HEAD. | 2026-05-29 | state-manager |
| D-197.1 | MED-001 CLOSED at devops 9fcfd49 (feature/S-025-tui-skeleton-sessions). Total: 18 replacements/11 files (NOT 17/10 — wider sweep found types.rs:48 v1.1.20 additional). Local cargo clean. CI queued. ADV16-CODIFY-001 6-category enumeration finalized. Counter 0/3; Pass 19 pending CI green. | 2026-05-29 | state-manager |
| D-198 | S-025 Pass 19 MED — F-S025-ADV19-MED-001: SS-conventions-anti-patterns.md v1.30.2 stale active pointers in clippy.toml:2 + deny.toml:1 (canonical v1.31.0). Counter HOLDS 0/3. Orchestrator preemptive comprehensive sweep dispatched (devops — all 7 canonical docs + SS-engine-module v1.1.20/SS-ipc v1.4.0 active-vs-historical adjudication). ADV16-CODIFY-001 GENERALIZED: 6th playbook category broadened from SS-deps-pin-manifest-specific to ALL concurrent doc bumps. Convergence-attempt #3 stalled at 0/3 floor (4 consecutive attempts). Devops comprehensive sweep SHA: PENDING follow-up burst (D-198.1). | 2026-05-29 | state-manager |
| D-198.1 | MED-001 CLOSED at devops 0aba808 (feature/S-025-tui-skeleton-sessions). 5 total active-pointer bumps: (1) clippy.toml:2 SS-conventions v1.30.2→v1.31.0; (2) deny.toml:1 SS-conventions v1.30.2→v1.31.0 — Pass 19 MED-001 targets; (3) engine.rs:4 SS-engine-module v1.1.20→v1.1.26; (4) engine_module_surface.rs:1197 SS-engine-module v1.1.20→v1.1.26; (5) engine_module_surface.rs:1223 SS-engine-module v1.1.20→v1.1.26 — devops wider-sweep catches (Pass 19 adversary missed). 8+ Category B historical anchors preserved (F-D-NN tagged + line-anchored + when-anchored sites). 4 other canonical docs verified clean (SS-deps-pin-manifest v1.2.0, SS-daemon-lifecycle v1.0.33, SS-core-types-and-abi v1.2.13, SS-forward-compatibility v1.2.19). CI: 8/10 SUCCESS at snapshot time; macOS Build+Test + cargo deny + cargo audit pending. F-S025-ADV19-MED-001: CLOSED. ADV16-CODIFY-001 REFINED with per-site adjudication protocol (Category A/B/Edge taxonomy) — template: 0aba808 commit body. Counter HOLDS 0/3; Pass 20 pending CI 10/10 green. | 2026-05-29 | state-manager |
| D-199 | S-025 Pass 20 MED-001 + LOW-001 CLOSED — devops ef7f4c62: engine.rs:143 v1.1.22→v1.1.26 (1 Category A bump; 72 citations canonical-anchored swept; 9 Category B preserved; 8 canonical docs verified). Test-writer dc229db: engine_module_surface.rs:6-8 Option B anchor (TDD red-gate authoring baseline parenthetical). CODIFY-001 D-198.2 canonical-anchored sweep protocol codified (supersedes D-198.1 stale-literal approach; extract cited_version, compare vs frontmatter canonical). ADV20-PROC-001 added: SS-conventions Test File Documentation Standards rule. 5 consecutive convergence-attempt stalls; counter HOLDS 0/3. CI pending 10/10 on ef7f4c62. Architect-escalation tripwire armed for Pass 21. | 2026-05-29 | state-manager |
| D-200 | S-025 Pass 21 NITPICK_ONLY-CLEAN — counter ADVANCES 0/3 → 1/3 (FIRST CLEAN after 5 consecutive convergence-attempt stalls). Tripwire NOT fired. Canonical-anchored sweep (D-198.2) independently re-executed: zero Category A stale active pointers across all 8 canonical docs; 9 Category B historical anchors preserved correctly. Path-B-propagation defect species BOUNDED. 5-stall pattern ENDED. Pass 22 dispatch under standard adversarial cadence; target 1/3 → 2/3. Convergence forecast: 3/3 at Pass 23 if Passes 22+23 both NITPICK_ONLY-CLEAN. | 2026-05-29 | state-manager |
| D-201 | S-025 Pass 22 MED — F-S025-ADV22-MED-001: SS-tui-core.md cited in 2 worktree files (lib.rs:7 + Cargo.toml:19) + 7 EPIC-06 story files (S-024 through S-031 range) but document DOES NOT EXIST (canonical: SS-tui.md v1.8.2). META-GAP in D-198.2 canonical-anchored sweep: protocol audits versioned SS-X.md vY.Z patterns only; bare-filename references that don't resolve are invisible. Counter RESETS 1/3 → 0/3 per MED rule. 4th consecutive 1/3→2/3 transition failure — DIFFERENT class than Path-B-propagation species (bounded by D-198.2). CODIFY-001 extended with 7th sweep category (bare-filename architecture anchor resolution audit). ADV22-PROC-001 added (CI-enforced anchor resolution check via build.rs or semgrep). META-pattern codified: each 1/3 transition surfaces NEW-class defect one layer deeper. Implementer (2-site worktree) + story-writer (7-site factory-artifacts) dispatched in parallel. Pass 23 pending post-fix HEAD. | 2026-05-29 | state-manager |

## Key Tech Stack

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14,
serde_yaml_ng 0.10, wasmtime 44, directories 6, notify 8, russh 0.60, rmcp 1.6,
reqwest 0.13, nucleo 0.5, nix 0.30, serde 1 (derive), chrono 0.4, serde_json =1.0.149 (EXACT), rand =0.8.6 (EXACT), time 0.3.47 (RUSTSEC-2026-0009 floor).
28 pinned production deps. **manifest v1.2.0**. **PRD v1.27.3**. **BC-INDEX v1.27** (72 numbered BCs + 41 DTU BCs = 113 total). **ARCH-INDEX v1.0.16** (7 subsystems). **SS-tui v1.8.2**. **SS-engine-module v1.1.26**. **SS-conventions-anti-patterns v1.31.1**. **ADR-0006 v1.2**. **BC-2.03.001 v1.0.6**. **STORY-INDEX v5.12** (33 stories, 195 pts). **sprint-state v1.30** (26/33 done, 156/195 pts, 80%). **S-001 v1.9**. **S-003 v1.8**. **S-014 v1.5**. **S-015 v1.7**. **S-024 v1.5**. **S-025 v1.7**. **S-026 v1.8**. **S-027 v1.4**. **S-028 v1.3**. **S-029 v1.2**. **S-031 v1.1**. **holdout-scenarios v1.5**. MSRV: Rust 1.88 (Phase 1-2, time 0.3.47 floor per RUSTSEC-2026-0009; original ratatui floor was 1.86); Rust 1.92 (Phase 3, wasmtime 44). 39 codified disciplines (SE-1..SE-23 + SE-40 candidate). Workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask.

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (v5.89..v6.10) | `cycles/cycle-001/burst-log.md` |
| Decisions D-047 through D-154 | `cycles/cycle-001/decisions-archive.md` |
| Phase 1 convergence history (R62-R122) | `cycles/cycle-001/phase-1-convergence.md` |
| Task queue (T-1 through T-131) | `cycles/cycle-001/completed-tasks.md` |
| Lessons learned (all rounds) | `cycles/cycle-001/lessons.md` |
| Prior session checkpoints (through v5.88) | `cycles/cycle-001/session-checkpoints.md` |
| Adversary reports | `.factory/plans/adversary-pass-*.md` |

## §Trace v6.49 (D-201.1 — MED-001 CLOSED; e5ebc43 + 0cea089; BONUS SS-forward-compat.md catch; SS-conventions v1.31.1; CI 9/9; Pass 23 dispatched)

**Pass 22 MED REMEDIATED** (2026-05-29, D-201.1): F-S025-ADV22-MED-001 CLOSED. Implementer e5ebc43 (feature/S-025-tui-skeleton-sessions): lib.rs:7 + Cargo.toml:19 fixed; sweep verified all 9 unique SS-*.md references in code — only SS-tui-core.md was broken. Story-writer 0cea089 (factory-artifacts): 7 EPIC-06 story files updated (S-024 v1.4→v1.5, S-025 v1.6→v1.7, S-026 v1.7→v1.8, S-027 v1.3→v1.4, S-028 v1.2→v1.3, S-029 v1.1→v1.2, S-031 v1.0→v1.1) + STORY-INDEX v5.11→v5.12.

**BONUS sweep-wider catch (0cea089, CLAUDE.md Principle 4):** Story-writer sweep found SS-conventions-anti-patterns.md:1043 citing `SS-forward-compat.md` (non-existent; canonical is `SS-forward-compatibility.md`). Fixed in-scope. SS-conventions-anti-patterns v1.31.0→v1.31.1. This is architect-domain work done by story-writer per CLAUDE.md Principle 4 (fix in scope, don't defer) + sweep-wider-than-finding discipline (L-W6-S025-007).

**CODIFY-001 7th sweep category PROVEN (D-201.1):** Canonical sweep command validated by story-writer 0cea089: `grep -rh -oE "SS-[a-z-]+\.md|ADR-[0-9]+-[a-z-]+\.md|BC-[0-9]+\.[0-9]+\.[0-9]+\.md" .factory/ 2>/dev/null | sort -u`. Scope: .factory/ (broader than worktree-only); catches spec-body and story-file bare-filename refs that the worktree-only grep misses. Historical-narrative exclusion (cycles/, §Trace, plans/) applied post-grep by human judgment.

**CI green:** 9/9 SUCCESS on e5ebc43 (Preflight, DTU, Semgrep, audit-table-drift, 3x Build+Test, cargo deny, cargo audit).

**Pass 23 dispatched.** Convergence-attempt #6; counter 0/3. META-pattern watch: 5th new-class defect one layer deeper? Two-sweep protocol (D-198.2 versioned + D-201 bare-filename) now operationally proven.

**Artifact versions bumped (D-201.1):** STATE v6.48→v6.49. SS-conventions-anti-patterns v1.31.0→v1.31.1. STORY-INDEX v5.11→v5.12. Stories: S-024 v1.5, S-025 v1.7, S-026 v1.8, S-027 v1.4, S-028 v1.3, S-029 v1.2, S-031 v1.1.

§Trace v6.48 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.42 through v6.44 archived to `cycles/cycle-001/burst-log.md` (D-198.1 compaction).
§Trace v6.40 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.39 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.38 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.37 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.36 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.35 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.34 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.29 through v6.32 archived to `cycles/cycle-001/burst-log.md` (D-188 compaction).
§Trace v6.22 through v6.28 archived to `cycles/cycle-001/burst-log.md`.
