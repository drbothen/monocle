---
document_type: pipeline-state
level: ops
project: monocle
version: "6.75"
status: active
producer: state-manager
timestamp: 2026-06-01T12:00:00Z
phase: phase-3-wave-7-IN-PROGRESS
current_step: "Wave 7 IN PROGRESS: 1/4 stories done. S-027 DELIVERED (PR #32 @ 3787ebd, D-226). 29/33 stories done (177/195 pts). Remaining: S-028 (5pts, UNBLOCKED), S-031 (5pts, UNBLOCKED), S-029 (5pts, UNBLOCKED — S-027 delivered)."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-047..D-174 archived at cycles/cycle-001/decisions-archive.md. D-175: Wave 4 gate PASSED. D-182: Wave 5 gate PASSED. D-183: Wave 6 AUTHORIZED. D-184: S-022 DELIVERED. D-185: S-023+S-025 AUTHORIZED. D-186: S-023 DELIVERED. D-188..D-221: see Decisions Log (archived in this file). D-222: S-025 DELIVERED (PR #28 @ 838477e). D-223: S-026 DELIVERED (PR #30 @ 9fb0d70) — Wave 6 COMPLETE. D-224: Wave-6 GATE PASSED (develop @ 2a51a91). D-225: STATE correction — phase-3-COMPLETE premature (Wave 7 of 7 remains); corrected to wave-7-READY; points 177→169/195 reconciled to sprint-state. D-226: S-027 DELIVERED (PR #32 @ 3787ebd) — Wave 7: 1/4 done."
awaiting: "Human authorization to begin next Wave 7 story. S-028 (5pts, UNBLOCKED) + S-031 (5pts, UNBLOCKED) are parallel-eligible. S-029 (5pts) is now UNBLOCKED (S-027 delivered). Phase 3 IN PROGRESS: 7 of 7 waves started; Wave 7 at 1/4 (29/33 stories, 177/195 pts). After all 4 Wave 7 stories merge + wave-7 gate PASSES → THEN Phase 4 (Holdout Evaluation). Pending (non-blocking): story-writer F-S025-ADV37-DEFER-001 (STORY-INDEX rows 150-153 stale BC→AC ranges + systematic sweep); architect F-S026-ADV1-LOW-002 (PermissionDecisionKind naming reconciliation)."
durable_task_register:
  outstanding:
    - id: "ADV-W5GATE-HIGH-001"
      subject: "daemon_start_sequence() doesn't wire DaemonState — integration story needed"
      status: pending
      detail: "Wave 5 gate adversarial: daemon_start_sequence() in monocle-runtime does not wire DaemonState fields (sock_file_path, last_hook_ts, ring). Requires integration story. Route to story-writer for new story in Wave 6/7."
      blocking: false
    - id: "ADV-W5GATE-HIGH-002"
      subject: "Duplicate S-009 handler dead code — cleanup needed"
      status: pending
      detail: "Wave 5 gate adversarial: S-009 HTTP handler has a duplicate code path. Dead code is non-functional. Route to implementer for cleanup fix-PR."
      blocking: false
    - id: "ADV-W5GATE-MED-001"
      subject: "S-017 UDS socket creation spurious WARN on rebind"
      status: pending
      detail: "Wave 5 gate adversarial: S-017 daemon start emits spurious WARN on rebinding already-removed socket path. Add explicit pre-remove check."
      blocking: false
    - id: "ADV-W5GATE-MED-003"
      subject: "HookEvent serde round-trip fragility — add constructors to monocle-core"
      status: pending
      detail: "Wave 5 gate adversarial: HookEvent deserialization relies on field-name stability without tagged union. Route to architect for BC update, then implementer."
      blocking: false
    - id: "#28"
      subject: "prost/reqwest exact-patch pin verification"
      status: partial
      detail: "prost = '=0.14.1' VERIFIED. reqwest = '=0.13.0' exists but latest 0.13.x is 0.13.3 — reqwest not yet activated by any Phase 1 member. Architect adjudication needed when S-009 activates reqwest."
      blocking: false
    - id: "#34"
      subject: "BC-2.03.001 PC-3 DeferUntil cleanup"
      status: pending
      detail: "BC-2.03.001 v1.0.7 PC-3 still enumerates DeferUntil in supporting types. Authority hierarchy resolves to story v1.4 + SS-engine-module v1.1.26 (no DeferUntil). PO mechanical fix."
      blocking: false
    - id: "BC-HOOK-034-typo"
      subject: "BC-HOOK-034 typo decorated_by -> deprecated_by"
      status: pending
      detail: "Cosmetic typo in BC-HOOK-034 frontmatter. Non-blocking. Maintenance sweep."
      blocking: false
    - id: "VP-DTU-001"
      subject: "VP-DTU-001 to be created by architect in Phase 4"
      status: deferred-phase-4
      detail: "All 41 BC-HOOK files cite VP-DTU-001 as verification property. Phase 4 deferral marker."
      blocking: false
    - id: "F-WAVE1-004"
      subject: "Cron schedule collision audit.yml + dtu-fidelity.yml"
      status: deferred-maintenance
      detail: "Both 0 0 * * 0 UTC. Stagger to avoid cache thrash. Low impact."
      blocking: false
    - id: "F-WAVE1-005"
      subject: "xtask not in cargo-deny [graph].targets"
      status: deferred-maintenance
      detail: "xtask is dev-tooling-only. Low likelihood Phase 1 risk. Document in deny.toml."
      blocking: false
    - id: "S-014-ADV-SS-engine-module-stale"
      subject: "SS-engine-module.md HookDecision code blocks stale"
      status: pending
      detail: "S-014 adversary surfaced stale HookDecision code blocks in SS-engine-module.md. Architect update needed."
      blocking: false
    - id: "S-011-ADV-HookArgs-divergence"
      subject: "HookArgs struct diverges from SS-permissions-phase1.md"
      status: pending
      detail: "S-011 adversary surfaced HookArgs struct definition diverging from SS-permissions-phase1.md canonical. Architect update needed."
      blocking: false
    - id: "S-013-ADV-prost-types-not-pinned"
      subject: "prost-types not registered in SS-deps-pin-manifest"
      status: pending
      detail: "prost-types is a transitive dep but not explicitly registered in SS-deps-pin-manifest.md. Architect must add explicit pin."
      blocking: false
    - id: "S-005-main-wiring"
      subject: "S-005 main.rs wiring: 10s drain timeout + second-signal detection + signal-path lock release"
      status: pending
      detail: "S-005 graceful shutdown implemented at library level. Requires main.rs wiring in integration story (Wave 3 or post-Wave-3)."
      blocking: false
    - id: "S-008-ADV-tempfile-spec"
      subject: "AC-003 tempfile::persist spec wording divergence"
      status: partial-closed
      detail: "RingError #[non_exhaustive] fixed (bef6f4b). Remaining: AC-003 tempfile::persist spec wording (story-writer); BC-2.01.007 S-TBD anchor (PO)."
      blocking: false
    - id: "S-015-ADV-tracing-test"
      subject: "tracing-test 0.2 not in SS-deps-pin-manifest"
      status: pending
      detail: "S-015 added tracing-test 0.2.6 with no-env-filter feature to monocle-runtime dev-deps. Not registered in SS-deps-pin-manifest.md. Architect must add."
      blocking: false
    - id: "SS-01-ProjectDirs-from"
      subject: "SS-daemon-lifecycle.md ProjectDirs::new → ProjectDirs::from"
      status: pending
      detail: "SS-daemon-lifecycle.md line 269 uses ProjectDirs::new('monocle','monocle','monocle') — should be ::from per SS-config.md and BC-2.04.006. Architect maintenance item."
      blocking: false
    - id: "IMPL-HookDecision-serde"
      subject: "HookDecision + HookResponse: add Serialize/Deserialize derives"
      status: deferred-wave-5
      detail: "Required for IPC wire transport. Must be done as part of S-021 or S-018 implementation."
      blocking: false
    - id: "IMPL-on-hook-Defer"
      subject: "ClaudeCodeModule::on_hook() implement Defer routing logic"
      status: deferred-wave-5
      detail: "Current implementation returns Allow unconditionally (Phase 1 placeholder). S-018 must implement the Defer path."
      blocking: false
    - id: "ADV-W3GATE-MED-001"
      subject: "last_hook_ts never written by hook handlers (future daemon wiring)"
      status: pending
      detail: "DaemonState.last_hook_ts field exists but hook handler paths never write it. Requires daemon wiring integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-002"
      subject: "Ring buffer DaemonState.ring never set to Some (main.rs stub)"
      status: pending
      detail: "DaemonState.ring is always None; ring_buffer_fill_pct always returns 0.0. Requires main.rs wiring."
      blocking: false
    - id: "ADV-W3GATE-MED-003"
      subject: "Only 1/5 hook endpoints tested in running-mode full-stack path"
      status: pending
      detail: "Full-stack integration tests cover only pre-tool endpoint. 4/5 endpoints lack coverage. Phase 4 integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-004"
      subject: "ring_buffer_fill_pct hardcoded to 0.0"
      status: pending
      detail: "Metric always 0.0 because DaemonState.ring is never set. Same root cause as ADV-W3GATE-MED-002."
      blocking: false
    - id: "ADV-W4GATE-MED-002"
      subject: "tracing::error() no-ops in monocle CLI binary (no subscriber)"
      status: pending
      detail: "monocle CLI binary does not initialize a tracing subscriber. Fix: initialize subscriber at binary entry or replace with eprintln! for startup errors."
      blocking: false
    - id: "HS-EXP-009-hint"
      subject: "Exit 70 missing stderr remediation hint for MONOCLE_RUNTIME_DIR"
      status: pending
      detail: "Daemon emits exit code 70 for invalid MONOCLE_RUNTIME_DIR but provides no stderr hint. BC-2.04.003 requires human-readable diagnostics."
      blocking: false
    - id: "PROC-SEMGREP-DECOUPLE"
      subject: "Semgrep silently skipped for Waves 1-5 when Preflight failed-fast on protoc"
      status: pending
      detail: "Decouple Semgrep from Preflight fast-fail chain so it runs independently. Route to devops-engineer. Target: Wave 7 or maintenance sweep."
      blocking: false
    - id: "PROC-GATE-SKIPPED-LOUD"
      subject: "Build+Test silently skipped for many prior CI runs — need GATE_SKIPPED loud indicator"
      status: pending
      detail: "CI gates that are skipped should emit a loud GATE_SKIPPED log line. Route to devops-engineer. Target: Wave 7 or maintenance sweep."
      blocking: false
    - id: "PROC-COMPUTE-INPUT-HASH-YAML"
      subject: "bin/compute-input-hash does NOT handle YAML-object-style inputs (path/version pairs)"
      status: pending
      detail: "compute-input-hash parser treats YAML-object inputs as plain string values. Recurring class — occurred in S-023 and S-025 cycles. Route to devops-engineer or dx-engineer."
      blocking: false
    - id: "S-025-TODO-S023-MERGE"
      subject: "Replace 2 TODO(S-023-merge) markers after S-025 rebase onto develop"
      status: pending
      detail: "app.rs has TODO(S-023-merge) at lines 586-615 and 630. After S-025 rebase onto develop, replace with real monocle_ipc imports. Mechanical substitution — implementer task post-rebase."
      blocking: false
    - id: "S-025-MAKE-MODAL-DEAD-CODE"
      subject: "Dead make_modal test helpers — defer to S-026 dispatch"
      status: pending
      detail: "Pass 8 LOW finding: make_modal test helpers in monocle-tui are dead code until S-026 implements permission overlay."
      blocking: false
    - id: "PROC-BRANCH-PROTECTION-CONTEXTS"
      subject: "Branch protection on develop has empty required-status-check contexts"
      status: pending
      detail: "develop branch protection rule has required-status-checks enabled but no specific check contexts configured (effectively a no-op). Requires admin escalation. Surface to human owner (Joshua Magady)."
      blocking: false
    - id: "F-S025-ADV13-NIT-003"
      subject: "BC-2.06.016 v1.0.9 §Trace line 230 stale 'Follow-up required' note"
      status: pending
      detail: "BC-2.06.016 line 230 note about SS-tui line 668 is stale (already uses bracketed form per 740465d). Cosmetic. Routing: product-owner. Anchored to Task #9 post-merge PO sweep."
      blocking: false
    - id: "F-S025-ADV13-NIT-004"
      subject: "BC-2.06.004 EC-079 line 104 cites non-production string 'Daemon offline'"
      status: pending
      detail: "Production has DAEMON_NOT_RUNNING_ERROR (full-screen panel) and DAEMON_OFFLINE_STATUS ('[daemon: offline]'). EC-079 ambiguous. Routing: product-owner. Anchored to Task #9 post-merge PO sweep."
      blocking: false
    - id: "F-S025-ADV20-PROC-001"
      subject: "[process-gap] Test File Documentation Standards rule (SS-conventions update)"
      status: pending
      detail: "Spec version citations in test/production file doc comments must carry disambiguation anchor (F-D-NN, §section, or parenthetical). Routing: architect (SS-conventions-anti-patterns update). Anchored to Task #9 post-S-025 merge."
      blocking: false
    - id: "F-S025-PATH-B-CLAUDE-MD"
      subject: "CLAUDE.md line 18 cites MSRV 1.86; Path B bumped Phase 1 MSRV to 1.88 — human action required"
      status: pending
      detail: "CLAUDE.md is human-maintained. Human action: update line 18 to: 'MSRV: Phase 1 = Rust 1.88 (time 0.3.47 floor per RUSTSEC-2026-0009 mitigation; original ratatui 0.30 floor was 1.86). Phase 3 = Rust 1.92 (wasmtime 44 requirement).' Non-blocking for S-025."
      blocking: false
    - id: "F-S025-ADV16-PROC-001"
      subject: "[process-gap] agents must use cargo clippy --workspace --all-targets -- -D warnings to match CI"
      status: pending
      detail: "CI uses --all-targets; lib-only mode silently misses test-code violations. Codification target: next agent-prompt-refresh cycle."
      blocking: false
    - id: "F-S025-ADV16-PROC-002"
      subject: "[process-gap] scripts/audit-table.md vendored copy and SS-engine-module.md canonical must update in same PR"
      status: pending
      detail: "scripts/audit-table.md is a vendored copy of the audit table from SS-engine-module.md. When canonical table changes, vendored copy must sync atomically in same PR."
      blocking: false
    - id: "F-S025-ADV22-PROC-001"
      subject: "[process-gap] CI-enforced bare-filename architecture anchor resolution check"
      status: pending
      detail: "POL-11 (f0926fe) covers versioned pins. Bare-filename existence-check (SS-tui-core.md style) needs separate CI enforcement. Anchored to Task #9."
      blocking: false
    - id: "F-S025-ADV24-MED-001"
      subject: "Cross-story S-026/27/28/31 inputs[] stale + done-story S-014..S-023 body prose stale"
      status: partial
      detail: "S-025 in-scope pins CLOSED (925c667). Cross-story + done-story body prose: DEFERRED to wave-gate (Task #9 anchored). POL-11 CI gate (f0926fe) will catch new instances."
      blocking: false
    - id: "F-S025-ADV24-MED-002"
      subject: "VP-body SS-deps-pin-manifest.md v1.1.17 stale across 14 VP files (45 occurrences)"
      status: pending
      detail: "14 VP files (vp-001..vp-021), 45 occurrences citing SS-deps-pin-manifest.md v1.1.17 (canonical v1.2.0). ROUTING: phase-5 system-level deferral. Task #9 anchored. POL-11 (f0926fe) will gate further instances. NOTE: POL-11 excludes VP files from freshness enforcement (normative scope only); these will be addressed at phase-5."
      blocking: false
    - id: "Task-9-m8-S028-cross-story"
      subject: "Task #9 m.8 — wave-gate sweep: S-028 lines 63+147 cross-story drift propagation (BC-5.39.002 PC2)"
      status: pending
      detail: "S-028 lines 63+147 still reference Vec<SessionState> type drift + structural-claim drift. Deferred per BC-5.39.002 PC2. story-writer sweep required at wave-gate. S-028 line 147 has structural-claim-deferrals.yaml authorized deferral entry (f0926fe)."
      blocking: false
    - id: "F-S025-ADV28-MED-001"
      subject: "S-025 §Downstream Consumer Contract struct-shape (META 10th + structural-claim #3)"
      status: closed
      detail: "CLOSED (D-207) via story-writer 344366d (Option B historical-anchor annotation at lines 225-231; tasks list line 144 clarified; S-025 v1.10→v1.11; STORY-INDEX v5.15→v5.16). System-level 3-way divergence (story 5 fields vs SS-tui.md 9 fields vs app.rs 7 fields) deferred to phase-5 as F-S025-ADV28-OBS-002."
      blocking: false
    - id: "F-S025-ADV28-MED-002"
      subject: "ADR-0008 §Canonical Source Registry off-by-2 line range (self-application defect)"
      status: closed
      detail: "CLOSED (D-207) via architect 12170b4 (ADR-0008 v1.0.1: line-range 831-864 → 833-864; §Self-Application Policy explicit; SS-conventions v1.32.2→v1.32.3 same correction)."
      blocking: false
    - id: "F-S025-ADV28-OBS-001"
      subject: "[Pattern-of-Patterns] 3rd consecutive ADR same-burst internal-consistency defect — architect protocol enhancement"
      status: pending
      detail: "ADR-0006 (Pass 16) + ADR-0007 (Pass 26) + ADR-0008 (Pass 28): 3 consecutive ADRs had internal-consistency defects discovered in the immediately-following adversarial pass. Per S-7.02 3-instance rule: codification required. Proposed: architect pre-commit self-consistency check — re-read each cited canonical line range before committing any ADR. Anchored to Task #9 m.9 (NEW). Routing: architect protocol update."
      blocking: false
    - id: "F-S025-ADV28-OBS-002"
      subject: "[worktree-vs-canonical App struct] Production app.rs 7-field App diverges from SS-tui.md §App struct 9-field canonical"
      status: pending
      detail: "3-way divergence: story v1.11 §Downstream Consumer Contract (5 fields, now historical-anchor annotated) vs canonical SS-tui.md §App struct (9 fields, v1.8.2) vs production app.rs (7 fields, 2d1188f). Story annotation resolves story layer. Spec-vs-implementation alignment requires architectural review. DEFERRED to phase-5 architectural-alignment. Not blocking for S-025 delivery."
      blocking: false
    - id: "ADR-HOOK-001"
      subject: "MECHANICAL ADR self-consistency pre-commit hook (devops, ~3pts)"
      status: pending
      detail: "Architect armed ADR self-consistency discipline in SS-conventions v1.32.4 (D-209 Pass 30 tripwire). Manual discipline is codified. Follow-up: implement a MECHANICAL pre-commit hook for ADR files that detects: (1) bold version labels outside §Trace sections, (2) unescaped `|` in backtick table-cell regexes, (3) numbered-list discontinuity in Amendment History. Route to devops-engineer. Wave 7 anchor: dispatch alongside or after S-027/S-028 delivery, before Phase 4 holdout evaluation. Story scope ~3pts."
      blocking: false
    - id: "S-025-POST-MERGE-S1"
      subject: "IpcManagerState::new() in monocle-tui/src/ipc.rs duplicates scaffolding in monocle-ipc — consolidation candidate"
      status: pending
      detail: "pr-reviewer suggestion (post-merge S-025): monocle-tui has its own IpcManagerState::new() that partially duplicates constructor scaffolding in monocle-ipc. Consolidation opportunity for a future EPIC-06 story (Wave 7 or post-Wave-6). Route to architect for scoping decision. Anchor: Wave 7 / EPIC-06 continuation."
      blocking: false
    - id: "S-025-POST-MERGE-TD1"
      subject: "Sessions panel skeleton rows for future planes (Workflow/Harness/Static) — intentional per S-025 scope"
      status: pending
      detail: "pr-reviewer tech-debt (post-merge S-025): Sessions panel has skeleton rows for Workflow, Harness, and Static planes not yet implemented. This is intentional per S-025 scope definition (skeleton only). Closed by S-026 + downstream Wave 6/7 stories. No action needed until those stories are dispatched."
      blocking: false
    - id: "F-S025-ADV37-DEFER-001"
      subject: "STORY-INDEX rows 150-153 stale S-025 BC→AC ranges (pre-renumbering draft, not propagated after §Trace v1.3/v1.4)"
      status: deferred-wave-gate
      detail: "STORY-INDEX.md rows 150-153 map S-025 BCs to STALE contiguous AC ranges from pre-renumbering drafts: BC-2.06.004→AC-001..003, BC-2.06.005→AC-004..006, BC-2.06.007→AC-007..008. Canonical (from S-025 body §Trace v1.4): BC-2.06.004←AC-002/003/004/008/010; BC-2.06.005←AC-005/006/007; BC-2.06.007←AC-001/009. Root cause: §Trace v1.3/v1.4 AC renumbering was not propagated to STORY-INDEX AC-range column. BC-INDEX has no AC-range column — confined to STORY-INDEX. Classified cross-story per BC-5.39.002 PC2 → does NOT reset Pass 37 convergence counter. FIX ANCHOR: Wave 6 gate / pre-Phase-4 — story-writer must also perform a systematic sweep of ALL other story rows in STORY-INDEX to detect whether the same pre-renumbering staleness affects other stories (systematic check required, not just S-025 rows). Per CLAUDE.md production-grade default, the human/orchestrator may elect to fix in-scope at wave-gate rather than defer further. Routing: story-writer."
      blocking: false
    - id: "F-S026-ADV6-DEFER-001"
      subject: "offline-break reconnect paths leave dead ipc_rx and hot-loop without re-attempting reconnect (availability gap)"
      status: resolved
      detail: "RESOLVED by PR #31 (merged develop @ 2a51a91). Wave-6-gate adversarial re-run elevated to CRITICAL (F-WAVE6-GATE-CRIT-001); fixed via extracted reconnect_from_offline() + 4 offline-break arms re-enter reconnect + offline_reconnect.rs mutation-verified integration test. Gate-3 re-run CLEAN. See cycles/cycle-001/blocking-issues-resolved.md."
      blocking: false
    - id: "POINTS-TALLY-RECONCILE"
      subject: "[guard] STATE running-tally points_complete drifted to 177 vs authoritative sprint-state sum 169; corrected D-225"
      status: resolved
      detail: "RESOLVED D-225 (2026-05-31): sprint-state summary.points_complete hand-incremented to 177 but authoritative per-story sum is 169 (28 done stories: Wave 1-6 complete). Also STATE prematurely declared 'Phase 3 COMPLETE' before Wave 7. Both corrected in factory-artifacts burst. GUARD: state-manager must re-sum sprint-state per-story points (never trust summary.points_complete) at every phase/wave transition. Also bump sprint-state summary.points_complete from 177→169 as correction."
      blocking: false
    - id: "HS-EXP-006-TTY-CAVEAT"
      subject: "Holdout HS-EXP-006 scored 0.85 — terminal raw-mode restore unobservable in non-TTY harness"
      status: pending
      detail: "Wave-6 holdout eval: HS-EXP-006 minimum satisfaction 0.85 (below mean 0.97) because clean terminal restore after Ctrl-\\ popup dismiss is unverifiable in a non-TTY subprocess harness. Confirm clean terminal restore in a TTY-backed demo or E2E harness in Phase 4 or at S-027. Route: e2e-tester/demo-recorder."
      blocking: false
    - id: "F-S026-ADV1-LOW-002"
      subject: "PermissionDecisionKind naming divergence vs SS-ipc/BCs PermissionDecision naming (pre-existing S-022 origin)"
      status: pending
      detail: "S-026 adversarial Pass 1 LOW finding (deferred). Impl uses PermissionDecisionKind{Allow,AcceptAlways,Deny}; SS-ipc and BCs use PermissionDecision{Accept,AcceptAlways,Reject}. Pre-existing naming divergence from S-022 origin. Route: architect at wave-6-gate (cross-story naming reconciliation). Non-blocking."
      blocking: false
    - id: "PROCESS-GAP-CI-PARITY-1"
      subject: "[process-gap] CLAUDE.md Lint line missing --all-targets flag — agents run cargo clippy without test-target coverage; human CLAUDE.md edit required"
      status: pending
      detail: "CLAUDE.md 'Build/Test/Lint' section Lint line reads 'cargo clippy --workspace -- -D warnings' (lib targets only). CI runs --all-targets. Test-code lints (unwrap/expect) slip to CI. Resolved this cycle via clippy.toml allow-*-in-tests (SS-conventions v1.32.6). FOLLOW-UP: CLAUDE.md Lint line must be updated to 'cargo clippy --workspace --all-targets -- -D warnings'. Human-maintained file — human action required. Non-blocking."
      blocking: false
    - id: "PROCESS-GAP-CI-PARITY-2"
      subject: "[process-gap] per-story delivery does not run POL-11/POL-12 locally pre-push; version-pin literal in test prose failed CI POL-11"
      status: pending
      detail: "Specialist agents do not run scripts/check_version_pins.py (POL-11) or scripts/check_structural_claims.py (POL-12) locally before declaring per-story delivery complete. A version-pin literal 'BC-2.06.024 v1.10' in test prose failed CI POL-11 and required fix-and-repush. FOLLOW-UP: per-story delivery skill should include POL-11/POL-12 local run as a pre-push gate step. Target: self-improvement epic or wave-gate codification in delivery skill."
      blocking: false
    - id: "F-S027-DOC-001"
      subject: "BC-2.06.021 PC-3 stale 'or replaced' prose vs BC-2.06.019 v1.1.0 PC-7"
      status: pending
      detail: "S-027 residual: BC-2.06.021 PC-3 body still reads 'or replaced' which conflicts with BC-2.06.019 v1.1.0 PC-7 semantic. Non-blocking doc-only fix. Routing: product-owner. Anchor: wave-7-gate sweep."
      blocking: false
    - id: "F-S027-DOC-002"
      subject: "render_frame doc-comment references legacy [dropped:N] format"
      status: pending
      detail: "S-027 residual: render_frame() doc-comment contains stale reference to legacy [dropped:N] status bar format. Current format is per BC-2.06.019 v1.1.0. Non-blocking. Routing: implementer at wave-7-gate sweep."
      blocking: false
    - id: "F-S027-DOC-003"
      subject: "overlay.rs / overlay_stub.rs stale docstrings"
      status: pending
      detail: "S-027 residual: overlay.rs and overlay_stub.rs contain docstrings referencing pre-S-027 placeholder behavior. Non-blocking cleanup. Routing: implementer at wave-7-gate sweep."
      blocking: false
    - id: "L-S027-004-PROCESS-GAP-REGISTRY-ATOMICITY"
      subject: "[process-gap] version-pin-registry.yaml must be bumped atomically with BC version bumps in same convergence-fix burst"
      status: pending
      detail: "S-027 lesson: BC version bumps (BC-2.06.015 v1.0.7, BC-2.06.016 v1.1.0, BC-2.06.019 v1.1.0, BC-2.06.020 v1.1.0, BC-2.06.021 v1.0.6) were committed in convergence-fix bursts but version-pin-registry.yaml was not always updated atomically in the same commit. Codify in convergence-fix burst checklist: bump version-pin-registry.yaml in the same commit as BC version bump. Routing: devops-engineer (checklist update). Anchor: wave-7-gate or delivery-skill update."
      blocking: false
  se_candidates:
    - id: SE-40
      occurrences: 2
      threshold: 3
      description: "Orchestrator drives deliver-story from main session only; never delegates to sub-orchestrator that cannot spawn fresh-context specialists."
      status: HELD per D-114
  process_discoveries:
    - "Full historical process_discoveries archived to cycles/cycle-001/burst-log.md (D-207 compaction)."
    - "Key disciplines: L-001..L-021 (cycle-001/lessons.md); L-W6-S025-* (Passes 21-42); L-W6-S026-001/002/003; L-W6-GATE-001/002/003."
    - "META-PATTERN BOUNDED (D-207): 18 instances. literal-pin→ADR-0007+POL-11; structural-claim→ADR-0008+POL-12. Both CI-gated (f0926fe). L-W6-S025-015 codified."
    - "STRICT-3/3-CLEAN convergence vindicated (D-221): Passes 36+37 perimeter-clean missed in-perimeter Esc/q defects; Pass 38 fresh-context source-re-derivation caught them. L-W6-S025-019/021."
    - "TALLY-GUARD (D-225): STATE running-tally must re-sum sprint-state per-story; summary.points_complete is a cache. Hand-increment drift produced +8 pts error + premature Phase-3-COMPLETE. L-W6-GATE-003."
next_session_resume_protocol: |
  ============================================================================
  ZERO-CONTEXT RESUME CHECKPOINT v6.75 (D-226) — 2026-06-01
  WAVE 7 IN PROGRESS — 1/4 DONE — S-027 DELIVERED
  ============================================================================

  YOUR FIRST 4 COMMANDS (RUN IN ORDER):

  1. Read /Users/jmagady/Dev/monocle/CLAUDE.md — production-grade-default + correct-agent-routing
     override ALL agent defaults. Read before dispatching anything.

  2. Read this STATE.md fully — especially §Trace v6.75, durable_task_register.

  3. Run worktree health check (BLOCKING per orchestrator startup protocol):
     Agent(subagent_type="vsdd-factory:devops-engineer",
           prompt="cd /Users/jmagady/Dev/monocle && run factory-worktree-health skill on this project")

  4. Execute NEXT ACTION below.

  PIPELINE STATE (as of 2026-06-01):

  Wave 7 IN PROGRESS (D-226). develop @ 3787ebd.
  Phase 3 TDD Implementation IN PROGRESS: Wave 7 at 1/4 stories done.
  29/33 stories done (177/195 pts, 91%). 3 remaining Wave 7 stories (15 pts) + 1 blocked S-PHASE-3-PREP (does NOT block).
  sprint-state v1.35.
  factory-artifacts: run git -C .factory log -1 --format='%h %s' for current HEAD.
  Factory worktree: /Users/jmagady/Dev/monocle/.factory/ (orphan branch factory-artifacts)

  NEXT ACTION:

  Obtain human authorization for next Wave 7 story.
  RECOMMENDED ORDER: S-028 + S-031 in parallel (both UNBLOCKED, independent — 5pts each).
                     S-029 after S-028/S-031 or in parallel (now UNBLOCKED — S-027 delivered).
  All 3 remaining require individual human authorization before dispatch (project norm).
  After all 4 Wave 7 stories merge → run vsdd-factory:wave-gate wave-7 → THEN Phase 4.

  WAVE 7 STORIES (final wave of Phase 3):
  * S-027 (8 pts, EPIC-06) — DONE. PR #32 @ 3787ebd (D-226).
  * S-028 (5 pts, EPIC-06) — Sessions Panel Nucleo Filter + Event Ribbon Rolling Log.
    deps [S-025 done, S-021 done] = UNBLOCKED.
    BCs: BC-2.05.002/004, BC-2.06.006/018.
  * S-031 (5 pts, EPIC-07) — Profile Picker: sticky-per-project + Ctrl-P override.
    deps [S-030 done, S-024 done, S-025 done] = UNBLOCKED.
    BCs: BC-2.07.004/005.
  * S-029 (5 pts, EPIC-06) — Killer Scenario: <=6 keystrokes dual permission resolve.
    deps [S-026 done, S-027 done, S-022 done, S-018 done] = UNBLOCKED (S-027 delivered D-226).
    BCs: BC-2.06.022. Validates holdout HS-EXP-008.

  DELIVERY (each story):
  vsdd-factory:deliver-story (per-story-delivery.md, 10 steps):
    worktree → stubs → failing tests (Red Gate) → implement →
    per-story adversarial convergence (BC-5.39.001: >=3 consecutive CLEAN, MUST precede demos) →
    demos → push → pr-manager 9-step PR → cleanup → state update.
  HUMAN AUTHORIZATION required before dispatching each story.

  CI-PARITY REQUIREMENTS (LESSONS LEARNED — DO NOT SKIP):
  1. Run `cargo clippy --workspace --all-targets -- -D warnings` (CI uses --all-targets;
     CLAUDE.md bare clippy line MISSES test targets — [HUMAN-only fix, PROCESS-GAP-CI-PARITY-1]).
  2. Run `python3 scripts/check_version_pins.py` (POL-11) AND
     `python3 scripts/check_structural_claims.py` (POL-12) locally before push.
     Both failed CI this cycle (PROCESS-GAP-CI-PARITY-2).
  3. Do NOT embed version-pin literals (e.g. "BC-X v1.2") in test prose — use version-free citations.

  ALSO PENDING (non-blocking, do at/before wave-7-gate or Phase 4):
  (a) story-writer: F-S025-ADV37-DEFER-001 — STORY-INDEX rows 150-153 stale BC→AC ranges + sweep.
  (b) e2e-tester/demo-recorder: HS-EXP-006-TTY-CAVEAT — confirm terminal restore in TTY-backed harness.
  (c) architect: F-S026-ADV1-LOW-002 — PermissionDecisionKind naming reconciliation vs SS-ipc.
  (d) devops: ADR-HOOK-001 — mechanical ADR pre-commit hook (~3 pts, Wave 7 anchor).
  (e) product-owner/implementer: F-S027-DOC-001/002/003 — S-027 residual doc fixes (wave-7-gate sweep).
  (f) devops: L-S027-004-PROCESS-GAP-REGISTRY-ATOMICITY — version-pin-registry.yaml atomicity checklist.

  ARTIFACT VERSIONS (D-226 canonical): see §Key Tech Stack below.
    SS-conventions v1.32.6 | STORY-INDEX v5.26 | BC-INDEX v1.33 (113 BCs) | sprint-state v1.35

  KNOWN-FLAKY (DO NOT FLAG):
    cli_daemon_stop, factory_self_referential, test_BC_2_07_006, wit-bindgen unmatched-skip, PATH isolation flake

  DEFERRED ITEMS — DO NOT RE-FLAG:
    F-S025-ADV12-LOW-002 + F-S025-ADV13-NIT-003/NIT-004 (BC polish)
    F-S025-ADV37-DEFER-001 (STORY-INDEX stale BC→AC ranges; pre-Phase-4 — story-writer fix)
    F-S025-PATH-B-CLAUDE-MD (MSRV CLAUDE.md human-update)
    F-S025-ADV24-MED-001 cross-story + F-S025-ADV24-MED-002 VP-body (phase-5)
    F-S025-ADV28-OBS-002 [worktree-vs-canonical App struct] (phase-5)
    ADR-HOOK-001 (Wave 7 — mechanical ADR pre-commit hook; devops)
    F-S026-ADV1-LOW-002 (PermissionDecisionKind naming divergence; architect pre-Phase-4)
    PROCESS-GAP-CI-PARITY-1 (CLAUDE.md --all-targets; human action required)
    PROCESS-GAP-CI-PARITY-2 (per-story POL-11/POL-12 local gate; devops codification)
    F-S027-DOC-001/002/003 (S-027 residual doc fixes; wave-7-gate sweep)
    L-S027-004-PROCESS-GAP-REGISTRY-ATOMICITY (version-pin-registry.yaml atomicity; devops)

  FACTORY INFRASTRUCTURE:
    .factory/ mounted at factory-artifacts orphan branch.
    Run factory-worktree-health via devops-engineer FIRST on session start.
    Commit hooks: block-ai-attribution, validate-input-hash, validate-table-cell-count — must pass.
    NEVER use --no-verify. NEVER add Co-Authored-By: Claude or robot emoji.
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
| 2 Story Decomposition | DONE (D-173 APPROVED) | 2026-05-27 | D-159 original gate: 17 stories, 86 pts. D-171: 16 stories (S-016..S-031, 109 pts) + 10 holdout scenarios. Total: 33 stories, 195 pts. D-172: adversarial story review 4 passes, trajectory 18→11→9→4. D-173: human gate APPROVED. BC-INDEX v1.23 (113 BCs). |
| 3 TDD Implementation | IN PROGRESS — Wave 7 (FINAL wave) IN PROGRESS | — | Wave 1+2+3 DONE (83 pts, 447 tests). Wave 4 GATE PASSED (D-175): 634 tests. Wave 5 GATE PASSED (D-182): 753 tests. Wave 6 GATE PASSED (D-224): develop @ 2a51a91; CRITICAL F-WAVE6-GATE-CRIT-001 fixed. Wave 7: S-027 DONE (PR #32 @ 3787ebd, D-226). 29/33 stories done (177/195 pts). Remaining: S-028/031 (UNBLOCKED), S-029 (UNBLOCKED). After wave-7 gate → Phase 4. |
| 4-7 | not-started | — | |

## Wave 5 — GATE PASSED (D-182)

| Story | Points | Status | Notes |
|-------|--------|--------|-------|
| S-017 Daemon Start Sequence + Hook Tmpfile | 8 | done | PR #22, 06432cf, 29 tests, adv 13→5→0 (3 passes) |
| S-018 Hook Endpoint Routing + Event Bus | 8 | done | PR #26, 654e281, 46 tests, adv 10→4→4 (CONVERGED) |
| S-019 Daemon Auto-Start + MONOCLE_NO_AUTOSTART | 5 | done | PR #25, 11540fc, 25 tests, adv 7→2→1 (CONVERGED) |
| S-020 JSONL Ring Capacity and Rotation | 5 | done | PR #24, f69d53a, 24 tests, adv 12→8→0 (CONVERGED) |
| S-021 UDS Server + IPC Transport + Core Message Types | 8 | done | PR #23, acaacb9, 49 tests, adv 9→4→4 (CONVERGED) |

develop @ 3787ebd (S-027 merged). Wave 5 gate PASSED (D-182). Wave 6: 4/4 done — S-022/023/025/026 (D-184/186/222/223). Wave-6 GATE PASSED (D-224): PR #31 CRITICAL fix @ 2a51a91. Wave 7 IN PROGRESS: S-027 DONE (D-226, PR #32 @ 3787ebd) — overlay rendering + diff preview + two-row status bar + [t] stub. 29/33 stories done, 177/195 pts (91%). Remaining: S-028 (UNBLOCKED), S-031 (UNBLOCKED), S-029 (UNBLOCKED as of D-226).

## Blocking Issues

None. All durable_task_register items non-blocking.

## Decisions Log (recent — D-215 through D-225)

D-047 through D-214 archived at: `cycles/cycle-001/decisions-archive.md`, `cycles/cycle-001/burst-log.md`, and earlier §Trace entries.

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-215 | S-025 Pass 36 CLEAN — counter 0/3→1/3 (ends 15-consecutive 1/3→2/3 failure run). L-W6-S025-018. STATE v6.63→v6.64. | 2026-05-30 | state-manager |
| D-216 | S-025 Pass 37 CLEAN — counter 1/3→2/3. F-S025-ADV37-DEFER-001 out-of-perimeter (wave-gate anchor). STATE v6.64→v6.65. | 2026-05-30 | state-manager |
| D-217 | S-025 Pass 38 RESET 2/3→0/3 — in-perimeter CONTENT defect: Esc/q quit key stale prose + zero test coverage. CLOSED: story-writer 8c7d693 + implementer 884401e. L-W6-S025-019. STATE v6.65→v6.66. | 2026-05-30 | state-manager |
| D-218 | S-025 Pass 39 HOLD — partial-fix regression Esc-semantics over-claim. 3-TRACK: PO 645c994 (BC-2.06.007 v1.0.5) + implementer 74585ea + story-writer 4d0fce1 (S-025 v1.14; STORY-INDEX v5.24). L-W6-S025-020. STATE v6.66→v6.67. | 2026-05-31 | state-manager |
| D-219 | S-025 Pass 40 CLEAN — counter 0/3→1/3. All 10 ACs re-derived from source. STATE v6.67→v6.68. | 2026-05-31 | state-manager |
| D-220 | S-025 Pass 41 CLEAN — counter 1/3→2/3. STATE v6.68→v6.69. | 2026-05-31 | state-manager |
| D-221 | S-025 Pass 42 CLEAN — counter 2/3→3/3 = FORMALLY CONVERGED. L-W6-S025-021. STATE v6.69→v6.70. | 2026-05-31 | state-manager |
| D-222 | S-025 DELIVERED — PR #28 @ 838477e. 27/33 done (164/195 pts). sprint-state v1.32. STATE v6.70→v6.71. | 2026-05-30 | state-manager |
| D-223 | S-026 DELIVERED — PR #30 @ 9fb0d70. BCs: BC-2.06.008/009/011..014/016/023/024. 9 adv passes, 3 CLEAN. CRITICAL Pass 5: outbound IPC unwired — fixed via ipc_tx/into_split/spawn_ipc_writer. Wave 6: 4/4 done (34/34 pts). 28/33 done. SS-conventions v1.32.6. sprint-state v1.33. STORY-INDEX v5.25. STATE v6.71→v6.72. | 2026-05-31 | state-manager |
| D-224 | Wave-6 GATE PASSED — develop @ 2a51a91. 5/6 gates green. CRITICAL F-WAVE6-GATE-CRIT-001 fixed (PR #31: reconnect_from_offline() + 4 offline-break arms). Holdout mean 0.97. sprint-state v1.34. L-W6-GATE-001/002. STATE v6.72→v6.73. | 2026-05-31 | state-manager |
| D-225 | STATE correction — phase-3-COMPLETE premature (Wave 7 of 7 remains); corrected to phase-3-wave-7-READY; points 177→169/195 reconciled to sprint-state per-story sum. L-W6-GATE-003. POINTS-TALLY-RECONCILE resolved. STATE v6.74→v6.73. Surfaced by human review. | 2026-05-31 | state-manager |
| D-226 | S-027 DELIVERED — PR #32 @ 3787ebd. Overlay rendering + diff preview (similar 3) + two-row status bar + [t] trace-to-source stub (BC-2.06.015, human-authorized). 18-pass adversarial convergence. BC bumps: BC-2.06.015 v1.0.7, BC-2.06.016 v1.1.0, BC-2.06.019/020/021 v1.1.0/v1.1.0/v1.0.6. Wave 7: 1/4 done. S-029 UNBLOCKED. 29/33 done (177/195 pts). sprint-state v1.35. STORY-INDEX v5.26. STATE v6.74→v6.75. | 2026-06-01 | state-manager |

## Key Tech Stack (D-226 canonical)

ratatui 0.30 | crossterm 0.29 | tokio 1.52 | axum 0.8 | interprocess 2.4 | prost 0.14
serde_yaml_ng 0.10 | wasmtime 44 | nucleo 0.5 | time 0.3.47 (RUSTSEC-2026-0009 floor)
serde_json =1.0.149 | rand =0.8.6 | 28 pinned deps. SS-deps-pin-manifest v1.2.0.
MSRV: Rust 1.88 (Phase 1-2); Rust 1.92 (Phase 3, wasmtime 44).
**PRD v1.27.4** | **BC-INDEX v1.33** (113 BCs) | **ARCH-INDEX v1.0.25** | **SS-tui v1.8.2**
**SS-conventions v1.32.6** | **ADR-0007 v1.0.8** | **ADR-0008 v1.0.6** | **STORY-INDEX v5.26**
**BC-2.06.015 v1.0.7** | **BC-2.06.016 v1.1.0** | **BC-2.06.019 v1.1.0** | **BC-2.06.020 v1.1.0** | **BC-2.06.021 v1.0.6**
**BC-2.06.023 v1.5.0** | **BC-2.06.024 v1.1.0** | **S-026 v1.11** | **S-027 v1.10** | **product-brief v1.4.34**
**sprint-state v1.35** (29/33 done, 177/195 pts; D-226). 59 codified disciplines.
9 workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui (S-025).

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (v5.89..v6.10) | `cycles/cycle-001/burst-log.md` |
| Decisions D-047 through D-154 | `cycles/cycle-001/decisions-archive.md` |
| Phase 1 convergence history (R62-R122) | `cycles/cycle-001/phase-1-convergence.md` |
| Task queue (T-1 through T-131) | `cycles/cycle-001/completed-tasks.md` |
| Lessons learned (all rounds) | `cycles/cycle-001/lessons.md` |
| Prior session checkpoints (through v6.56) | `cycles/cycle-001/session-checkpoints.md` |
| Adversary reports | `cycles/cycle-001/S-025/adversarial-pass-*.md` |
| CODIFY-001 sweep protocol reference (Categories 1-11) | `cycles/cycle-001/burst-log.md` (D-207 archive) |

## §Trace v6.75 (D-226 — S-027 DELIVERED; Wave-7 IN PROGRESS)

**D-226 (2026-06-01):** S-027 DELIVERED — PR #32 @ 3787ebd squash-merged to develop.
Overlay rendering + diff preview (similar 3) + two-row status bar + [t] trace-to-source stub (BC-2.06.015, human-authorized scope addition).
18-pass adversarial convergence (5 BLOCKER + 6 MAJOR + [t]-stub MAJOR + drops:N coexistence MAJOR resolved).
BC version bumps: BC-2.06.015 v1.0.7, BC-2.06.016 v1.1.0, BC-2.06.019 v1.1.0, BC-2.06.020 v1.1.0, BC-2.06.021 v1.0.6.
Wave 7: 1/4 done. S-029 UNBLOCKED. 29/33 stories done (177/195 pts). sprint-state v1.35. STORY-INDEX v5.26. v6.74→v6.75.

**D-225 (2026-05-31):** STATE correction — phase-3-COMPLETE premature (Wave 7 of 7 remains).
Corrected to phase-3-wave-7-READY. Points 177→169/195 (sprint-state per-story re-sum). L-W6-GATE-003. v6.73→v6.74.

**D-224 (2026-05-31):** Wave-6 GATE PASSED — develop @ 2a51a91 (PR #31 squash-merged). 5/6 gates green.
CRITICAL F-WAVE6-GATE-CRIT-001 fixed: offline stuck TUI — reconnect_from_offline() + 4 offline-break arms.
Tests PASS | DTU PASS (mean 1.000) | Adversarial PASS (post-remediation) | Demo Evidence PASS | Holdout PASS (mean 0.97) | Mutation SKIPPED.
L-W6-GATE-001/002. v6.72→v6.73.

Phase 3 IN PROGRESS — Waves 1-6 PASSED (D-164/D-166/D-167/D-175/D-182/D-224). Wave 7 IN PROGRESS (D-226).
29/33 stories done (177/195 pts). After wave-7 gate → Phase 4.

§Trace v6.40 through v6.72 archived to `cycles/cycle-001/burst-log.md`.
