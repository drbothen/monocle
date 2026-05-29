---
document_type: pipeline-state
level: ops
project: monocle
version: "6.40"
status: active
producer: state-manager
timestamp: 2026-05-29T02:00:00Z
phase: phase-3-wave-6-IN-PROGRESS
current_step: "S-025 Pass 17 NITPICK_ONLY-CLEAN (1 LOW: BC-2.03.001 MSRV 1.86 ref; architect sweep gap). Counter advances 0/3 → 1/3 (HOLDING pending LOW-001 fix-round landing). Product-owner LOW-001 fix dispatched in parallel. Pass 18 dispatch at post-fix HEAD."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-047..D-174 archived at cycles/cycle-001/decisions-archive.md. D-175: Wave 4 gate PASSED. D-182: Wave 5 gate PASSED (develop @ 1ce7838). D-183: Wave 6 AUTHORIZED. D-184: S-022 DELIVERED (PR #27). D-185: S-023+S-025 parallel AUTHORIZED. D-186: S-023 DELIVERED (PR #29 @ 7a52041). D-187: S-025 in flight. D-188: Pass 12 CRITICAL fix + F-S025-CI-001. D-189: Pass 13 LOW; fix dispatched. D-190: Pass 14 NIT; fix dispatched. D-191: Pass 15 CLEAN; counter 0/3→1/3. D-192: Pass 16 MED (7-round fix; counter RESET 0/3). D-193: Pass 16 round 6 BackoffState gap; F-R30-1 threshold CROSSED. D-194: Pass 16 round 7 Path B RUSTSEC-2026-0009 MSRV 1.86→1.88; CI all 9 green bfcba19. D-195: Pass 17 NITPICK_ONLY-CLEAN (1 LOW BC-2.03.001 MSRV 1.86 stale ref; PO fix dispatched); counter 0/3→1/3 HOLDING; Pass 18 at post-fix HEAD."
awaiting: "Product-owner BC-2.03.001 LOW-001 fix on factory-artifacts (BC-2.03.001 v?.?.? — PO choosing version; placeholder); then Pass 18 adversary at post-fix HEAD. After S-025 convergence (3/3): S-026 (13pts) dispatch."
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
      subject: "[S-7.02 codification trigger] F-R30-1 recurrence count crossed 3 (now 4). Codify audit-table sweep discipline."
      status: pending
      detail: "Pass 16 round 6 (D-193): F-R30-1 recurrence count crossed threshold (4 rows total: App + EventBusHookEvent + EngineModuleRegistry + BackoffState). S-7.02 codification REQUIRED. Codify in CLAUDE.md or VSDD.md: 'When a new crate is added or merged from a separate branch, the architect MUST run git ls-tree <merge-base>..HEAD + per-file #[non_exhaustive] pub struct sweep before declaring audit-table sync complete.' Anchored to Task #9 post-merge sweep, batched with story-writer for follow-up story creation. Route orchestrator Task #9 batch to story-writer + CLAUDE.md documentation update."
      blocking: false
    - id: "F-S025-PATH-B-CLAUDE-MD"
      subject: "CLAUDE.md line 18 cites MSRV 1.86; Path B bumped Phase 1 MSRV to 1.88 — human action required"
      status: pending
      detail: "CLAUDE.md line 18 reads: 'MSRV: Phase 1 = Rust 1.86 (ratatui 0.30 floor). Phase 3 = Rust 1.92 (wasmtime 44 requirement).' Architect Path B work (D-194) bumped Phase 1 MSRV to 1.88 (time 0.3.47 floor per RUSTSEC-2026-0009 mitigation). CLAUDE.md is human-maintained (outside agent write scope). Human action: update line 18 to: 'MSRV: Phase 1 = Rust 1.88 (time 0.3.47 floor per RUSTSEC-2026-0009 mitigation; original ratatui 0.30 floor was 1.86). Phase 3 = Rust 1.92 (wasmtime 44 requirement).' Anchored to next human review; non-blocking for S-025."
      blocking: false
    - id: "F-S025-ADV17-LOW-001"
      subject: "BC-2.03.001 stale 'MSRV 1.86 stable Rust' reference — product-owner 2-line fix"
      status: pending
      detail: "Pass 17 (D-195): BC-2.03.001 lines 35 + 61 say 'MSRV 1.86 stable Rust' — stale project-MSRV claim after Path B bumped Phase 1 MSRV to 1.88. Architect MSRV sweep (D-194) updated 6 artifacts but missed this BC. Fix: replace 'MSRV 1.86 stable' → 'MSRV 1.88 stable' at lines 35 + 61 + BC version bump + §Trace. Routing: product-owner. BC-2.03.001 version to be assigned by PO (placeholder v?.?.?). Counter advance holds pending fix-round landing; Pass 18 at post-fix HEAD. Non-blocking for S-025 overall."
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
next_session_resume_protocol: |
  S-025 PASS 17 DONE (D-195) — PASS 18 PENDING PO FIX — STATE v6.40

  STATE: develop @ 7a52041. 26/33 done (156/195 pts). 852+ tests. S-025 HEAD bfcba19 (CI all 9 green).
  COUNTER: 1/3 HOLDING (Pass 17 NITPICK_ONLY-CLEAN; 1 LOW dispatched to PO in parallel).
  TRAJECTORY: 5→4→3→2→4→H→M→0→M→M→H→C→L→N(14)→C(15)→M(16)→N(17).
  MSRV: Phase 1 = 1.88 (time 0.3.47 floor). Phase 3 = 1.92. CLAUDE.md line 18 needs human update (F-S025-PATH-B-CLAUDE-MD).

  IMMEDIATE NEXT ACTIONS:
    1. CHECK: git -C .factory log origin/factory-artifacts --oneline -3 for PO BC-2.03.001 fix commit.
    2. Once landed: pull/rebase factory-artifacts; confirm BC-2.03.001 final version; update v?.?.? placeholder in STATE.md awaiting + D-195.
    3. Dispatch Pass 18 adversary at bfcba19 (unchanged — PO touched specs only). Focus: verify BC-2.03.001 fix + full re-sweep all 17 prior axes.
    4. If Pass 18 CLEAN → counter 2/3. Dispatch Pass 19.

  PASS 17 LOW FINDING (F-S025-ADV17-LOW-001):
    BC-2.03.001 lines 35+61: "MSRV 1.86 stable Rust" — stale after Path B bump.
    Architect sweep (D-194) updated 6 artifacts; missed this BC. PO fix: 2 lines + version bump.
    Full report: cycles/cycle-001/S-025/adversarial-pass-17.md.

  CRITICAL FILES FOR PASS 18 ADVERSARY (read in order):
    1. .factory/STATE.md (v6.40); 2. adversarial-pass-17.md; 3. adversarial-pass-16.md;
    4. adversarial-pass-15.md; 5. adversarial-pass-14.md; 6. adversarial-pass-12.md;
    7. architect-decisions-pass-1.md; 8. architect-decisions-pass-2.md;
    9. text-style-adjudication.md; 10. red-gate-log.md;
    11. .factory/stories/S-025-tui-skeleton-sessions.md (v1.6);
    12. .factory/specs/behavioral-contracts/ss-03/BC-2.03.001.md (verify fix);
    13. CLAUDE.md (project principles — production-grade default).
    All files at: .factory/cycles/cycle-001/S-025/

  ARTIFACT VERSIONS (Pass 17; BC-2.03.001 placeholder):
    BC-INDEX v1.27 (113 BCs). PRD v1.27.3. SS-engine-module v1.1.26. SS-deps-pin-manifest v1.2.0.
    SS-tui v1.8.2. ADR-0006 v1.2. S-025 v1.6. S-026 v1.7. BC-2.03.001 v?.?.? (pending PO fix).

  AFTER CONVERGENCE (3/3 NITPICK_ONLY-CLEAN):
    Rebase S-025 → develop. Resolve TODO(S-023-merge) at app.rs:586-615+630. Demo-recorder (10 ACs).
    PR-manager (PR #28 draft → merge). State-manager D-187 closure. Dispatch S-026 (13 pts).
    Task #9 post-merge: F-S025-ADV16-CODIFY-001 (story-writer + CLAUDE.md), NIT-003/004 (PO).

  KEY LESSONS (full details: cycles/cycle-001/lessons.md L-W6-S025-001..007):
    L-001: Propagation sweeps = BC bodies + SS docs + story fm + body + input-hashes (all 5).
    L-002: Assertion must trace to EXACT production code path (not TestBackend-local copy).
    L-003: pub const extraction eliminates vacuous-mirror class structurally.
    L-004: Premature-clean signal confirmed (Pass 15 clean → Pass 16 MED). Max skepticism.
    L-007: Sweep wider than the finding — catch ALL class siblings.

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
| 3 TDD Implementation | IN PROGRESS — Wave 6 2/4 done; S-025 Pass 17 NITPICK_ONLY-CLEAN (1 LOW BC-2.03.001); counter 0/3 → 1/3 HOLDING; LOW fix-round in flight | 2026-05-28 | Wave 1+2+3 DONE (83 pts, 447 tests, all 6 gates). Wave 4 GATE PASSED (D-175): 634 tests. Wave 5 GATE PASSED (D-182): 753 tests, 0 failures, clippy clean, fmt clean. Wave 6: 2/4 done (S-022 8pts + S-023 5pts). 26/33 stories done (156/195 pts). S-025 Pass 17 NITPICK_ONLY-CLEAN (1 LOW: BC-2.03.001 MSRV 1.86 stale ref; PO fix-round in parallel; counter 0/3 → 1/3 HOLDING). S-026 blocked on S-025. Trajectory: 5→4→3→2→4→H→M→0→M→M→H→C→L→N(14)→C(15)→M(16)→N(17). |
| 4-7 | not-started | — | |

## Wave 5 — GATE PASSED (D-182)

| Story | Points | Status | Notes |
|-------|--------|--------|-------|
| S-017 Daemon Start Sequence + Hook Tmpfile | 8 | done | PR #22, 06432cf, 29 tests, adv 13→5→0 (3 passes) |
| S-018 Hook Endpoint Routing + Event Bus | 8 | done | PR #26, 654e281, 46 tests, adv 10→4→4 (CONVERGED) |
| S-019 Daemon Auto-Start + MONOCLE_NO_AUTOSTART | 5 | done | PR #25, 11540fc, 25 tests, adv 7→2→1 (CONVERGED) |
| S-020 JSONL Ring Capacity and Rotation | 5 | done | PR #24, f69d53a, 24 tests, adv 12→8→0 (CONVERGED) |
| S-021 UDS Server + IPC Transport + Core Message Types | 8 | done | PR #23, acaacb9, 49 tests, adv 9→4→4 (CONVERGED) |

develop @ 7a52041. 852+ tests, 0 failures. 26/33 stories done, 156/195 pts (80%). Wave 5 gate PASSED (D-182). Wave 6 in progress: S-022 DONE (D-184, PR #27 @ c7540539), S-023 DONE (D-186, PR #29 @ 7a52041). S-025 Pass 17 NITPICK_ONLY-CLEAN (1 LOW: BC-2.03.001 MSRV 1.86 stale ref; counter 0/3 → 1/3 HOLDING; PO fix-round in parallel; HEAD bfcba19; CI all 9 green; D-195). S-026 blocked on S-025. F-R30-1 codification threshold CROSSED (4/3).

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
| D-195 | S-025 Pass 17 NITPICK_ONLY-CLEAN — 1 LOW (BC-2.03.001 MSRV 1.86 stale ref; PO fix dispatched in parallel). Zero CRITICAL/HIGH/MED. Counter 0/3 → 1/3 HOLDING. BC-2.03.001 v?.?.? (PO placeholder). Pass 18 at post-fix HEAD. | 2026-05-29 | state-manager |

## Key Tech Stack

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14,
serde_yaml_ng 0.10, wasmtime 44, directories 6, notify 8, russh 0.60, rmcp 1.6,
reqwest 0.13, nucleo 0.5, nix 0.30, serde 1 (derive), chrono 0.4, serde_json =1.0.149 (EXACT), rand =0.8.6 (EXACT), time 0.3.47 (RUSTSEC-2026-0009 floor).
28 pinned production deps. **manifest v1.2.0**. **PRD v1.27.3**. **BC-INDEX v1.27** (72 numbered BCs + 41 DTU BCs = 113 total). **ARCH-INDEX v1.0.16** (7 subsystems). **SS-tui v1.8.2**. **SS-engine-module v1.1.26**. **ADR-0006 v1.2**. **STORY-INDEX v5.9** (33 stories, 195 pts). **sprint-state v1.30** (26/33 done, 156/195 pts, 80%). MSRV: Rust 1.88 (Phase 1-2, time 0.3.47 floor per RUSTSEC-2026-0009; original ratatui floor was 1.86); Rust 1.92 (Phase 3, wasmtime 44). 39 codified disciplines (SE-1..SE-23 + SE-40 candidate). Workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask.

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

## §Trace v6.40 (D-195 — Pass 17 NITPICK_ONLY-CLEAN; 1 LOW BC-2.03.001 MSRV gap; counter 0/3 → 1/3 HOLDING; PO fix-round dispatched)

**S-025 PASS 17 RESULT: NITPICK_ONLY-CLEAN** (2026-05-29): 17 adversarial passes; counter advances 0/3 → 1/3 (HOLDING pending LOW-001 fix-round landing).

**Pass 17 outcome (D-195):** Dispatched against HEAD bfcba19 (Path B: MSRV 1.88, time 0.3.47). Comprehensive verification of all 7 Pass 16 fix rounds PASSED. Angles I-N exercised with zero defects. One LOW finding: F-S025-ADV17-LOW-001 — BC-2.03.001 lines 35+61 say "MSRV 1.86 stable Rust"; architect MSRV propagation sweep (D-194) updated 6 artifacts but missed this BC. Semantic distinction: SS-conventions "Rust 1.86+" (language history, correct on 1.88) vs BC-2.03.001 "MSRV 1.86 stable" (project MSRV claim, stale). Fix: 2-line update + BC version bump. Routed to product-owner in parallel. BC-2.03.001 v?.?.? (placeholder — fill after PO commit lands on factory-artifacts). Zero CRITICAL/HIGH/MED. Counter advances 0/3 → 1/3 HOLDING. Pass 18 dispatch at post-PO-fix HEAD.

**Trajectory:** Pass 17: NITPICK_ONLY-CLEAN (1 LOW: BC-2.03.001 MSRV 1.86 reference; architect sweep gap; fix-round dispatched in parallel; counter advances 0/3 → 1/3 HOLDING pending fix-round landing).

**Phase 3 trajectory shorthand:** 5→4→3→2→4→H→M→0→M→M→H→C→L→N(14)→C(15)→M(16)→N(17).

**Convergence forecast:** 3/3 at Pass 19 if Passes 18 + 19 also clean. WARN: L-W6-S025-004 (premature-clean signal) applies. Maximum skepticism at every counter-advance moment.

**Artifact versions bumped this burst (D-195):** STATE v6.39→v6.40. No spec bumps this burst (BC-2.03.001 bump owned by product-owner).
Full Pass 17 report: `cycles/cycle-001/S-025/adversarial-pass-17.md`.

§Trace v6.39 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.38 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.37 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.36 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.35 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.34 archived to `cycles/cycle-001/burst-log.md`.
§Trace v6.29 through v6.32 archived to `cycles/cycle-001/burst-log.md` (D-188 compaction).
§Trace v6.22 through v6.28 archived to `cycles/cycle-001/burst-log.md`.
