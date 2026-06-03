---
document_type: pipeline-state
level: ops
project: monocle
version: "6.88"
status: active
producer: state-manager
timestamp: 2026-06-03T23:00:00Z
phase: PIVOT-delta-in-progress
current_step: "D-238 delta: product-brief.md v2.0.0 COMMITTED (control-center re-baseline, status draft; validate-brief verdict VALID; input-hash 7e4f4f4). NEXT = architecture delta (architect): resolve open questions Q-1 (PTY-over-UDS), Q-2 (EngineModule/SessionManager surface), Q-7 (tui-term fork posture), Q-8 (HIGH — native session-host persistence feasibility; if infeasible, surface external-supervisor tradeoff for human decision); design SessionManager/session-host/embedded-PTY subsystem; scope D-235 in-process daemon-wiring rework; reconcile stale narrow-keyboard-scope in DISPOSITION-V2 rollup + embedded-pty-evaluation. Then story decomposition."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-047..D-174 archived at cycles/cycle-001/decisions-archive.md. D-175: Wave 4 gate PASSED. D-182: Wave 5 gate PASSED. D-183: Wave 6 AUTHORIZED. D-184: S-022 DELIVERED. D-185: S-023+S-025 AUTHORIZED. D-186: S-023 DELIVERED. D-188..D-221: see Decisions Log (archived in this file). D-222: S-025 DELIVERED (PR #28 @ 838477e). D-223: S-026 DELIVERED (PR #30 @ 9fb0d70) — Wave 6 COMPLETE. D-224: Wave-6 GATE PASSED (develop @ 2a51a91). D-225: STATE correction — phase-3-COMPLETE premature (Wave 7 of 7 remains); corrected to wave-7-READY; points 177→169/195 reconciled to sprint-state. D-226: S-027 DELIVERED (PR #32 @ 3787ebd) — Wave 7: 1/4 done. D-227: S-031 DELIVERED (PR #33 @ 8451486). D-228: S-028 DELIVERED (PR #34 @ 682e5e5) — Wave 7: 3/4 done. develop @ 682e5e5. D-229: Zero-context resume checkpoint — S-029 human-authorized 2026-06-02. develop @ 1158e24. D-230: S-029 DELIVERED (PR #35 @ 48463fb) — Wave 7 COMPLETE (4/4). 32/33 done (192/195 pts). develop @ 48463fb. D-231: Wave-7-gate prerequisite sweep complete — SS-ipc v1.11.0, BC-2.06.021 v1.0.7, BC-INDEX v1.34, STORY-INDEX v5.30, citation atomicity propagation. POL-11/POL-12 PASS. S-027/S-028 story frontmatter status fixed. D-232: Wave-7 gate PASSED — Phase 3 COMPLETE. 1514 tests, 0 failures. F-W7G3-MED-001 fixed (PR #37 @ 6811103). HS-EXP-008 score 1.0. DTU SKIP (DTU-CLONE-STORY added as Phase 4 prereq). sprint-state v1.38. D-233: Phase-3→4 consistency cleanup — EVAL-INDEX v1.9, STORY-INDEX v5.31, BC-HOOK-034 v1.0.2 (typo fix), sprint-state v1.39. 28 story-file status fields corrected. Input-hash refresh (113→0+17 bookkeeping-residual). POL-11/POL-12 PASS. All MED/LOW audit findings RESOLVED. D-234: DTU clone false-negative corrected — S-DTU-001 (cargo binary dtu-claude-code-hooks-v1) validated fidelity 1.0000 (25/25 fixtures). Gate-2 DTU-VALIDATION corrected from SKIP to PASS. DTU-CLONE-STORY closed (RESOLVED-FALSE-PREMISE). Phase 4 UNBLOCKED. dtu_clones_built updated. PROC-DTU-VALIDATE-LOCATION process gap added. D-235: Daemon-wiring convergence — monocle-runtime binary now serves (main() wires daemon_start_sequence + run_server + UDS + tracing + ring-flush + 10s drain). SS-daemon-wiring-impl v1.3.0. SS-deps-pin-manifest v1.2.1. ARCH-INDEX v1.0.26. STORY-INDEX v5.32. sprint-state v1.40. S-DAEMON-WIRE-FIX-001 Wave-8 anchor. Resolved: ADV-W5GATE-HIGH-001, ADV-W3GATE-MED-002/004, ADV-W4GATE-MED-002, S-005-main-wiring, F-DW-HIGH-001. POL-11/POL-12 PASS. D-236: PRODUCT-VISION PIVOT — observe-only RETIRED; monocle → full TUI control center. Phases 4-7 SUSPENDED. D-237: Human ratified re-baselined-v1 control-center vision scope (4 capabilities: Launch, Embedded PTY, Multi-session/multi-project, Interactive Tune + already-built Observe+Control). DAEMON-OWNS-PTY locus. Hook auto-injection v1. embedded-pty-evaluation.md v1.0: primary = portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4. NEXT: gene-source disposition → revised vision-synthesis → human gate. D-238: Vision approval gate PASSED. domain-monocle-vision-synthesis.md APPROVED at v2.1 by Joshua Magady as the canonical basis for the control-center re-baselined-v1 brief→architecture→story delta. HUMAN ESCALATION folded in at the gate: v1A persistence now REQUIRES that a graceful daemon-PROCESS restart SURVIVES (CASE 2 changed from 'lost' to 'survive'). Persistence principle renamed DAEMON-OWNS-PTY → 'session-host-owns-PTY; daemon coordinates/re-attaches': PTY masters + harness child processes owned by native detached per-session session-host processes (abduco/dtach-style) that outlive the daemon process; daemon re-attaches over UDS on restart. NO-TMUX preserved as default; external supervisor is architect-surfaced fallback only (requires human decision, not silent adoption). CASE 1 (TUI restart survives) and CASE 3 (hard crash → lost, re-launch) unchanged. New HIGH-priority architect question Q-8 (PTY-ownership-survival mechanism) added; NOTE: the already-built D-235 in-process daemon wiring will likely need rework to move PTY ownership out of the daemon process. Remaining architect-only open questions: Q-1 (PTY bytes over UDS), Q-2 (EngineModule/SessionManager surface), Q-7 (tui-term fork posture), plus PTY-throughput benchmark — all resolved during architecture delta. Architect must also reconcile the stale narrow keyboard scope in DISPOSITION-V2 rollup + embedded-pty-evaluation (superseded by full-fidelity ratification). NEXT: brief delta (product-owner) → architecture delta (architect) → story decomposition (story-writer)."
awaiting: "Architecture delta (architect): Q-1 PTY-over-UDS, Q-2 EngineModule/SessionManager surface, Q-7 tui-term fork posture, Q-8 HIGH native session-host persistence feasibility (if infeasible → surface external-supervisor tradeoff for human decision); design SessionManager/session-host/embedded-PTY subsystem; scope D-235 in-process daemon-wiring rework; reconcile stale narrow-keyboard-scope in DISPOSITION-V2 rollup + embedded-pty-evaluation. Then story decomposition. product-brief.md v2.0.0 committed (draft; awaiting adversarial + human gate during/after architecture delta)."
durable_task_register:
  outstanding:
    - id: "DTU-CLONE-STORY"
      subject: "DTU clone false-negative — RESOLVED D-234 (RESOLVED-FALSE-PREMISE)"
      status: resolved-false-premise
      detail: "RESOLVED D-234: the D-232 Gate-2 SKIP and this task's blocking: true classification were based on a false premise. The DTU clone EXISTS as S-DTU-001 (status: done, facade mode, wave 1, BC-HOOK-001..041). Binary dtu-claude-code-hooks-v1 lives in crates/monocle-test-harness/src/bin/dtu_server.rs (built at target/release/dtu-claude-code-hooks-v1). Validated on develop @ 90ae584: fidelity mean 1.0000 (25/25 fixtures, threshold 0.95), all 5 endpoints covered (pre-tool-use, notification, stop, session-start, prompt-submit), X-Claude-Code-Ide-Authorization header correct, BC-HOOK-034 filter passes, clippy + semgrep CLEAN. Gate-2 DTU-VALIDATION corrected to PASS (see wave-7-gate-report.md D-234 annotation). Phase 4 holdout-eval gate is UNBLOCKED. Root cause: dtu-validator tooling looked for .factory/dtu-clones/ docker dir and missed the cargo-binary clone location. Process gap tracked as PROC-DTU-VALIDATE-LOCATION."
      blocking: false
    - id: "PROC-DTU-VALIDATE-LOCATION"
      subject: "[process-gap] DTU validation must check cargo-binary clone location, not only .factory/dtu-clones/ docker dir"
      status: pending
      detail: "D-234: two independent agents (wave-7 Gate-2 dtu-validator and Phase-3→4 consistency-audit HIGH-001) produced false-negative DTU-missing verdicts by looking for a .factory/dtu-clones/ docker-style directory. They MISSED the cargo-binary clone delivered as S-DTU-001 (crates/monocle-test-harness/src/bin/dtu_server.rs, binary target dtu-claude-code-hooks-v1). DTU validation tooling must check the actual clone artifact location as recorded in the DTU story (target_module field) rather than assuming a fixed .factory/dtu-clones/ path. Routing: agent-prompt improvement for vsdd-factory:dtu-validator and vsdd-factory:consistency-validator. Target: self-improvement epic or upstream vsdd-factory issue. Non-blocking."
      blocking: false
    - id: "ADV-W5GATE-HIGH-001"
      subject: "daemon_start_sequence() doesn't wire DaemonState — RESOLVED D-235"
      status: resolved
      detail: "RESOLVED D-235: main() now wires daemon_start_sequence + run_server + UDS listener + tracing subscriber + durable ring-flush shutdown + 10s drain timeout. DaemonState fields (sock_file_path, ring) are wired. 16+ adversarial passes over 6 fix rounds, converged. Code on feat/daemon-wire-serve (PR pending merge)."
      blocking: false
    - id: "ADV-W5GATE-HIGH-002"
      subject: "Duplicate S-009 handler dead code — cleanup needed (re-confirmed D-235)"
      status: pending
      detail: "Wave 5 gate adversarial: S-009 HTTP handler has a duplicate code path. Dead code is non-functional. Re-confirmed still present during D-235 daemon-wiring adversarial review. Route to implementer for cleanup fix-PR."
      blocking: false
    - id: "F-DW-HIGH-001"
      subject: "CI false-green — daemon integration test was a sleep loop — RESOLVED D-235"
      status: resolved
      detail: "RESOLVED D-235: daemon-e2e-affordances CI job added on feat/daemon-wire-serve branch catches the real serve path. The pre-daemon-wiring test suite was green because the daemon binary ran a sleep loop with no assertions on actual service behavior. Resolved by implementing the real main() wiring + wiring test coverage."
      blocking: false
    - id: "HIGH-2-SECOND-SIGNAL-DEFER"
      subject: "Second-signal exit codes (143/130) during drain — DEFERRED to S-DAEMON-WIRE-FIX-001 (Wave 8)"
      status: deferred-wave-8
      detail: "D-235 daemon-wiring adversarial Round-3 HIGH-2: DaemonExit::SigtermDuringDrain (exit 143) and SigintDuringDrain (exit 130) variants are defined in BC-2.01.004 INV-4 monitoring contract but NOT produced — a second SIGTERM/SIGINT during graceful drain currently hits the OS default. Explicit human-authorized deferral per CANONICAL PRINCIPLE rule 3 (future story anchor required): S-DAEMON-WIRE-FIX-001 (Wave 8, P1, 5pts, EPIC-04). CONTRACT GAP markers in crates/monocle-runtime/src/lifecycle.rs. NOT a loose item."
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
      subject: "BC-HOOK-034 typo decorated_by -> deprecated_by — RESOLVED D-233"
      status: resolved
      detail: "RESOLVED: product-owner fixed deprecated_by typo in D-233 Phase-3→4 consistency cleanup. BC-HOOK-034 bumped v1.0.1→v1.0.2."
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
      subject: "S-005 main.rs wiring: 10s drain timeout + second-signal detection + signal-path lock release — PARTIALLY RESOLVED D-235"
      status: resolved
      detail: "RESOLVED D-235: main() wires daemon_start_sequence + run_server + UDS + tracing + durable ring-flush + 10s drain timeout. Signal-path lock release wired. Second-signal exit codes (143/130) deferred to S-DAEMON-WIRE-FIX-001 (Wave 8) as documented HIGH-2 anchored deferral in SS-daemon-wiring-impl v1.3.0."
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
      subject: "Ring buffer DaemonState.ring never set to Some — RESOLVED D-235"
      status: resolved
      detail: "RESOLVED D-235: main() wires durable ring-flush shutdown + DaemonState.ring set to Some. ring_buffer_fill_pct now reflects actual ring state. Code on feat/daemon-wire-serve."
      blocking: false
    - id: "ADV-W3GATE-MED-003"
      subject: "Only 1/5 hook endpoints tested in running-mode full-stack path"
      status: pending
      detail: "Full-stack integration tests cover only pre-tool endpoint. 4/5 endpoints lack coverage. Phase 4 integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-004"
      subject: "ring_buffer_fill_pct hardcoded to 0.0 — RESOLVED D-235"
      status: resolved
      detail: "RESOLVED D-235: same fix as ADV-W3GATE-MED-002 — ring wired in main(). ring_buffer_fill_pct now reflects real ring state."
      blocking: false
    - id: "ADV-W4GATE-MED-002"
      subject: "tracing::error() no-ops in monocle CLI binary (no subscriber) — RESOLVED D-235"
      status: resolved
      detail: "RESOLVED D-235: monocle-runtime main() now initializes a tracing subscriber (tracing-subscriber 0.3 added as prod dep in SS-deps-pin-manifest v1.2.1). CLI binary startup errors now emit to the subscriber."
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
    - id: "S-029-PROCESS-GAP-PC-LABEL-DRIFT"
      subject: "S-029 story PC-N label drift (process-gap) — RESOLVED by story-writer v1.3"
      status: resolved
      detail: "S-029 adversarial review surfaced PC-N label drift in story spec. story-writer v1.3 corrected labels. RESOLVED prior to PR #35 merge. Closed D-230."
      blocking: false
    - id: "F-S025-ADV28-MED-001"
      subject: "S-025 §Downstream Consumer Contract struct-shape — CLOSED (D-207)"
      status: closed
      detail: "See cycles/cycle-001/blocking-issues-resolved.md"
      blocking: false
    - id: "F-S025-ADV28-MED-002"
      subject: "ADR-0008 §Canonical Source Registry off-by-2 — CLOSED (D-207)"
      status: closed
      detail: "See cycles/cycle-001/blocking-issues-resolved.md"
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
      subject: "STORY-INDEX rows 150-153 stale BC→AC ranges — RESOLVED D-231"
      status: resolved
      detail: "RESOLVED: story-writer fixed in D-231 wave-7-gate sweep. STORY-INDEX v5.29→v5.30 with corrected AC ranges for BC-2.06.004/005/007 per §Trace v1.4 canonical: BC-2.06.004←AC-002/003/004/008/010; BC-2.06.005←AC-005/006/007; BC-2.06.007←AC-001/009. Systematic sweep of all other STORY-INDEX rows also completed; no other pre-renumbering staleness found."
      blocking: false
    - id: "F-S026-ADV6-DEFER-001"
      subject: "offline-break reconnect paths — RESOLVED (PR #31 @ 2a51a91, D-224)"
      status: resolved
      detail: "See cycles/cycle-001/blocking-issues-resolved.md"
      blocking: false
    - id: "POINTS-TALLY-RECONCILE"
      subject: "STATE running-tally drift — RESOLVED D-225"
      status: resolved
      detail: "See cycles/cycle-001/blocking-issues-resolved.md. GUARD: always re-sum sprint-state per-story points."
      blocking: false
    - id: "HS-EXP-006-TTY-CAVEAT"
      subject: "Holdout HS-EXP-006 scored 0.85 — terminal raw-mode restore unobservable in non-TTY harness"
      status: pending
      detail: "Wave-6 holdout eval: HS-EXP-006 minimum satisfaction 0.85 (below mean 0.97) because clean terminal restore after Ctrl-\\ popup dismiss is unverifiable in a non-TTY subprocess harness. Confirm clean terminal restore in a TTY-backed demo or E2E harness in Phase 4 or at S-027. Route: e2e-tester/demo-recorder."
      blocking: false
    - id: "F-S026-ADV1-LOW-002"
      subject: "PermissionDecisionKind naming divergence vs SS-ipc/BCs PermissionDecision naming — RESOLVED D-231"
      status: resolved
      detail: "RESOLVED: architect reconciled in D-231 wave-7-gate sweep. SS-ipc bumped to v1.11.0 with PermissionDecisionKind naming aligned. Citation propagation complete (BC-2.05.001-008, BC-2.06.023 Architecture Source rows updated). POL-11 PASS."
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
      subject: "BC-2.06.021 PC-3 stale 'or replaced' prose — RESOLVED D-231"
      status: resolved
      detail: "RESOLVED: product-owner fixed BC-2.06.021 PC-3 'or replaced' text in D-231 wave-7-gate sweep. BC-2.06.021 bumped to v1.0.7. BC-INDEX bumped to v1.34. Registry updated."
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
      subject: "[process-gap] version-pin-registry atomicity — RESOLVED D-231 (lesson absorbed)"
      status: resolved
      detail: "RESOLVED: D-231 sweep enforced registry atomicity for all four bumped artifacts (SS-ipc, BC-2.06.021, BC-INDEX, STORY-INDEX). Lesson codified: all future burst commits MUST update version-pin-registry.yaml in the same commit as the document version bump. Routing for checklist codification: devops-engineer delivery-skill update (tracked as delivery-skill process-gap, non-blocking)."
      blocking: false
    - id: "F-S028-NIT-001"
      subject: "S-028 ScrollUp/ScrollDown empty-sessions asymmetry — RESOLVED D-231 (confirmed no-op pre-gate sweep)"
      status: resolved
      detail: "RESOLVED D-231: reviewed in wave-7-gate sweep. Fix-PR delivered and merged prior to this sweep (part of the wave-7-gate sweep artifacts committed by pr-manager). Closed."
      blocking: false
    - id: "F-S028-NIT-002"
      subject: "S-028 filter-mode ribbon selected_sid index-space mismatch — RESOLVED D-232 (fixed as F-W7G3-MED-001)"
      status: resolved
      detail: "RESOLVED D-232: surfaced at integration scope as F-W7G3-MED-001 during wave-7 adversarial gate review. Fixed in scope via PR #37 @ 6811103 — render_sessions_filter now returns the highlighted session_id from the filtered entry with index-space remap + render test. pr-reviewer CLEAN. Security-reviewer CLEAN. 9 CI checks green."
      blocking: false
    - id: "FLAKY-TIMING-5MS"
      subject: "test_BC_2_06_010 5ms timing threshold — RESOLVED D-231"
      status: resolved
      detail: "RESOLVED: implementer widened threshold to 10ms in D-231 wave-7-gate sweep fix-PR. No more boundary flakes on loaded CI runners."
      blocking: false
    - id: "PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP"
      subject: "[process-gap] architect committed implementation code directly to develop instead of spec+guidance — codify architect-spec-only role"
      status: pending
      detail: "S-028 cycle: architect committed implementation code directly to develop (commit 32f319e) instead of producing spec + routing to implementer-in-worktree. Reset + re-routed to implementer corrected the artifact. CODIFY: architect role = spec production only; all code must be written by implementer in an isolated worktree. Anti-pattern: architect writes Rust code in main session and commits to develop. Routing: orchestrator + delivery-skill update."
      blocking: false
    - id: "PROCESS-GAP-TMP-COMMIT-MSG-MIXUP"
      subject: "[process-gap] parallel implementers reused /tmp commit-message files causing cross-branch mislabeled commits"
      status: pending
      detail: "S-028+S-031 parallel cycle: parallel implementers reused /tmp/commit-msg paths causing cross-branch mislabeled commits (881cde2 S-028 with S-031 message; 71e9426/5266acd misleading messages). CODIFY: each dispatch must use a unique per-story /tmp path (e.g., /tmp/commit-msg-S-028-<timestamp>) to prevent cross-contamination. Routing: orchestrator + delivery-skill update."
      blocking: false
    - id: "PROCESS-GAP-PRMANAGER-EARLY-RETURN"
      subject: "[process-gap] pr-manager returned mid-process (after step 4/5) twice, requiring re-dispatch to complete merge"
      status: pending
      detail: "S-028 cycle: pr-manager agent returned control to orchestrator after step 4-5 of the 9-step PR workflow twice, requiring two re-dispatch cycles to complete the merge. Root cause: pr-manager does not have a completion-signal discipline. CODIFY: pr-manager must complete all 9 steps and emit an explicit COMPLETE signal before returning. Routing: orchestrator + pr-manager skill update."
      blocking: false
    - id: "ADR-HOOK-001-WIRING"
      subject: "ADR self-consistency hook script delivered but NOT wired — HUMAN action required"
      status: pending
      detail: "D-231 wave-7-gate sweep: scripts/validate_adr_self_consistency.sh delivered on the fix-PR branch. Agent scope is blocked from wiring it into .claude/settings.json or the plugin hooks-registry — requires human to add it to project settings. Non-blocking. Routing: human action (Joshua Magady). Anchor: before Phase 4 start or at next human session."
      blocking: false
    - id: "SIGTERM-TERMINAL-RESTORE"
      subject: "monocle-tui has no SIGTERM handler — terminal not restored on external kill"
      status: pending
      detail: "D-231: monocle-tui does not register a SIGTERM handler, so if the process is killed externally the terminal raw-mode is not restored. HS-EXP-006 residual caveat. Pre-existing architectural gap (crossterm does not install signal handlers by default). Phase 4+ quality item. Routing: architect + implementer. Non-blocking for wave-7-gate."
      blocking: false
    - id: "F-S028-NIT-002-DEFERRED"
      subject: "S-028 event-ribbon selected_sid filter-index-space mismatch — RESOLVED D-232 (fixed as F-W7G3-MED-001 in scope)"
      status: resolved
      detail: "RESOLVED D-232: the D-231 deferred maintenance item surfaced at integration scope during wave-7 adversarial gate review as F-W7G3-MED-001 (MEDIUM severity). Fixed in scope via PR #37 @ 6811103. Both F-S028-NIT-002 and F-S028-NIT-002-DEFERRED are now closed."
      blocking: false
    - id: "PIVOT-CONTROL-CENTER"
      subject: "PRODUCT-VISION PIVOT — monocle becomes full TUI control center (D-236); vision APPROVED D-238; delta in progress"
      status: active-delta-in-progress
      detail: "D-236 (2026-06-03): Human-directed vision pivot. monocle evolves from observe-only to full TUI control center: launch + manage + observe + tune + control; many sessions, many projects; never leave the TUI. A better lazyclaude AND claude-squad. Observe-only constraint (vision-synthesis v1.1.2, approved 2026-05-11) RETIRED. VSDD Phase 4/5/6/7 of the old observe-only scope SUSPENDED. Phase-1 substrate is built and reusable: daemon (now actually serves), hook ingestion, permission overlay (VecDeque<PromptModal>), EngineModule/FactoryAdapter traits, proto/ring, TUI rendering. D-237 (2026-06-03): Human ratified v1 capability scope and DAEMON-OWNS-PTY persistence locus. D-238 (2026-06-03): Vision APPROVED — domain-monocle-vision-synthesis.md v2.1 status:approved. Persistence principle renamed to 'session-host-owns-PTY; daemon coordinates/re-attaches'. CASE 2 survival now required (graceful daemon-PROCESS restart survives). Q-8 HIGH (PTY-ownership-survival) added for architect. D-235 daemon wiring likely needs rework. NEXT: brief delta (product-owner) → architecture delta (Q-8 HIGH, D-235 rework, input-doc reconciliation) → story decomposition. Remains active until delta lands."
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
  ZERO-CONTEXT RESUME CHECKPOINT v6.88 (D-238-delta) — 2026-06-03
  PIVOT: monocle → full TUI control center — VISION v2.1 APPROVED — BRIEF COMMITTED
  ============================================================================

  YOUR FIRST 3 COMMANDS (RUN IN ORDER — BEFORE ANYTHING ELSE):

  1. Read /Users/jmagady/Dev/monocle/NEXT-SESSION-PIVOT.md — pivot handoff context.

  2. Read /Users/jmagady/Dev/monocle/CLAUDE.md — production-grade-default + correct-agent-routing
     override ALL agent defaults.

  3. Read this STATE.md fully — especially §Trace v6.88 (D-238-delta), durable_task_register,
     and PIVOT-CONTROL-CENTER entry.

  CRITICAL ORIENTATION — D-238-delta: BRIEF COMMITTED, ARCHITECTURE DELTA NEXT:

  product-brief.md v2.0.0 committed to factory-artifacts (draft; input-hash 7e4f4f4).
  validate-brief verdict VALID (planning/brief-validation.md v6.0).
  Vision v2.1 APPROVED (D-238). Persistence = 'session-host-owns-PTY; daemon re-attaches'.
  CASE 2 (daemon-PROCESS restart) REQUIRED to SURVIVE. Q-8 HIGH open for architect.
  D-235 in-process daemon wiring LIKELY NEEDS REWORK.

  DO NOT run vsdd-factory:phase-4-holdout-evaluation.
  DO NOT resume adversarial refinement (Phase 5), formal hardening (Phase 6), or
  convergence (Phase 7) of the old observe-only scope.

  WHAT THE NEW SESSION MUST DO:

  Step 1 (DONE): Brief delta — product-brief.md v2.0.0 committed. validate-brief VALID.
  Step 2: Architecture delta — architect handles:
          - Q-8 HIGH: PTY-ownership-survival mechanism (abduco/dtach-style session-host
            processes vs in-process daemon ownership; likely reworks D-235 wiring)
          - Q-1 (PTY bytes over UDS), Q-2 (EngineModule/SessionManager surface),
            Q-7 (tui-term fork posture), PTY-throughput benchmark
          - Input-doc reconciliation: DISPOSITION-V2 rollup + embedded-pty-evaluation
            stale narrow keyboard scope (superseded by full-fidelity ratification)
  Step 3: Story decomposition — story-writer decomposes delta into new waves.

  WHAT IS ALREADY BUILT (REUSE — DO NOT REBUILD):

  - Daemon (now actually serves: HTTP hook ingestion + UDS socket + tracing + ring).
  - Hook ingestion (5 endpoints, proto, JSONL ring, dtu-claude-code-hooks-v1 clone).
  - Permission overlay (VecDeque<PromptModal>, Ctrl-\ popup, y/n/A resolve, IPC write-back).
  - EngineModule / FactoryAdapter traits — the seam to extend with launch/spawn/attach/kill.
  - monocle-proto wire schemas, monocle-ipc, monocle-config, monocle-core.
  - TUI rendering (ratatui + crossterm, sessions panel, event ribbon, profile picker, status bar).
  - 1514 passing tests. 9 workspace crates on develop.

  PIPELINE STATE (as of 2026-06-03 D-238-delta):

  Phase 3 TDD Implementation COMPLETE (D-232). All 7 waves delivered and gated.
  D-235: Daemon-wiring CONVERGED (feat/daemon-wire-serve merged as PR #39 @ fcd42f0).
  develop: check git log -1 for live HEAD.
  factory-artifacts: run git -C .factory log -1 --format='%h %s' for live HEAD.
  32/33 stories done (192/195 pts, 98%). Phase 4-7 SUSPENDED per D-236 pivot.
  Vision v2.1 APPROVED (D-238). product-brief.md v2.0.0 COMMITTED (draft, D-238-delta).
  NEXT: architecture delta (Q-8 HIGH + SessionManager/session-host + D-235 rework).
  Factory worktree: /Users/jmagady/Dev/monocle/.factory/ (orphan branch factory-artifacts)

  KEY ARTIFACTS FOR ARCHITECTURE DELTA:

  - /Users/jmagady/Dev/monocle/NEXT-SESSION-PIVOT.md — pivot handoff (read FIRST)
  - .factory/specs/product-brief.md v2.0.0 — control-center re-baseline (COMMITTED, draft)
  - .factory/specs/research/domain-monocle-vision-synthesis.md v2.1 — APPROVED canonical basis
  - .factory/specs/architecture/SS-engine-module.md — EngineModule trait (extend with launch/lifecycle)
  - .factory/specs/research/embedded-pty-evaluation.md — embedded PTY research (input for architect)
  - .factory/semport/DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md — gene disposition (stale keyboard scope — reconcile)
  - .factory/STATE.md durable_task_register entry PIVOT-CONTROL-CENTER

  WAVE-8 BACKLOG (valid, subordinate to pivot):
    S-032 (5 pts, Wave 8, EPIC-05) — daemon fan-out (BC-2.05.004 v1.1.0 PC-2 obligation)
    S-DAEMON-WIRE-FIX-001 (Wave 8, P1, 5pts) — second-signal exit codes 143/130

  KNOWN-FLAKY (DO NOT FLAG):
    cli_daemon_stop, factory_self_referential, test_BC_2_07_006, wit-bindgen unmatched-skip,
    PATH isolation flake.

  FACTORY INFRASTRUCTURE:
    .factory/ mounted at factory-artifacts orphan branch.
    Run factory-worktree-health via devops-engineer FIRST on session start.
    Commit hooks: block-ai-attribution, validate-input-hash, validate-table-cell-count — must pass.
    NEVER use --no-verify. NEVER add Co-Authored-By: Claude or robot emoji.
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: 2026-05-28
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
| 3 TDD Implementation | COMPLETE — Wave-7 GATE PASSED (D-232) | 2026-06-03 | Wave 1+2+3 DONE (83 pts). Wave 4 GATE PASSED (D-175). Wave 5 GATE PASSED (D-182). Wave 6 GATE PASSED (D-224) @ 2a51a91. Wave 7 GATE PASSED (D-232): S-027 (D-226), S-031 (D-227), S-028 (D-228), S-029 (D-230). F-W7G3-MED-001 fixed PR #37 @ 6811103. 1514 tests, 0 failures. HS-EXP-008 score 1.0. 32/33 done (192/195 pts). develop @ 6811103. NEXT: Phase 3→4 transition gate → Phase 4. |
| 4-7 | SUSPENDED — vision pivot D-236 | — | Old observe-only scope retired. Do NOT run phase-4-holdout-evaluation until vision revision complete. |
| PIVOT | DELTA-IN-PROGRESS (D-238-delta) | 2026-06-03 | Vision v2.1 APPROVED (D-238). product-brief.md v2.0.0 committed (draft, D-238-delta). session-host-owns-PTY. Q-8 HIGH. D-235 rework likely. NEXT: architecture delta → story decomposition. |

develop @ fcd42f04 (NEXT-SESSION-PIVOT.md + CLAUDE.md D-236 banner). Phase 3 COMPLETE (D-232). 32/33 stories done, 192/195 pts (98%). D-236: PRODUCT-VISION PIVOT — monocle becomes full TUI control center. D-238: Vision v2.1 APPROVED. D-238-delta: product-brief.md v2.0.0 COMMITTED (draft). session-host-owns-PTY. Q-8 HIGH. VSDD Phases 4-7 (old scope) SUSPENDED. NEXT: architecture delta (architect, Q-8+D-235 rework+SessionManager) → story decomposition.

## Blocking Issues

None. All durable_task_register items non-blocking.

## Decisions Log (recent — D-222 through D-238-delta)

D-047 through D-221 archived at: `cycles/cycle-001/decisions-archive.md`, `cycles/cycle-001/burst-log.md`, and earlier §Trace entries.

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-222 | S-025 DELIVERED — PR #28 @ 838477e. 27/33 done (164/195 pts). sprint-state v1.32. STATE v6.70→v6.71. | 2026-05-30 | state-manager |
| D-223 | S-026 DELIVERED — PR #30 @ 9fb0d70. BCs: BC-2.06.008/009/011..014/016/023/024. 9 adv passes, 3 CLEAN. CRITICAL Pass 5: outbound IPC unwired — fixed via ipc_tx/into_split/spawn_ipc_writer. Wave 6: 4/4 done (34/34 pts). 28/33 done. SS-conventions v1.32.6. sprint-state v1.33. STORY-INDEX v5.25. STATE v6.71→v6.72. | 2026-05-31 | state-manager |
| D-224 | Wave-6 GATE PASSED — develop @ 2a51a91. 5/6 gates green. CRITICAL F-WAVE6-GATE-CRIT-001 fixed (PR #31: reconnect_from_offline() + 4 offline-break arms). Holdout mean 0.97. sprint-state v1.34. L-W6-GATE-001/002. STATE v6.72→v6.73. | 2026-05-31 | state-manager |
| D-225 | STATE correction — phase-3-COMPLETE premature (Wave 7 of 7 remains); corrected to phase-3-wave-7-READY; points 177→169/195 reconciled to sprint-state per-story sum. L-W6-GATE-003. POINTS-TALLY-RECONCILE resolved. STATE v6.74→v6.73. Surfaced by human review. | 2026-05-31 | state-manager |
| D-226 | S-027 DELIVERED — PR #32 @ 3787ebd. Overlay rendering + diff preview (similar 3) + two-row status bar + [t] trace-to-source stub (BC-2.06.015, human-authorized). 18-pass adversarial convergence. BC bumps: BC-2.06.015 v1.0.7, BC-2.06.016 v1.1.0, BC-2.06.019/020/021 v1.1.0/v1.1.0/v1.0.6. Wave 7: 1/4 done. S-029 UNBLOCKED. 29/33 done (177/195 pts). sprint-state v1.35. STORY-INDEX v5.26. STATE v6.74→v6.75. | 2026-06-01 | state-manager |
| D-227 | S-031 DELIVERED — PR #33 squash-merged to develop @ 8451486. Profile Picker: sticky-per-project + Ctrl-P override + CCR path status bar. BCs: BC-2.07.004 (sticky-per-project), BC-2.07.005 (Ctrl-P override). 9-pass adversarial convergence (3 consecutive CLEAN). Wave 7: 2/4 done. STATE v6.75→v6.76. | 2026-06-01 | state-manager |
| D-228 | S-028 DELIVERED — PR #34 squash-merged to develop @ 682e5e5. Sessions Panel Nucleo Filter + Event Ribbon Rolling Log. BCs: BC-2.05.002, BC-2.05.004 (TUI consumer), BC-2.06.006, BC-2.06.018. 10-pass adversarial convergence + S-031 integration-merge + 3-cycle PR review. Human-authorized scope expansion: timestamp_micros to HookEventReceived IPC (daemon emit deferred to S-032) + EnrichedSession.display_name. BC bumps: BC-2.05.004 v1.1.0, BC-2.06.006 v1.1.0, BC-2.06.018 v1.1.0, SS-ipc v1.10.0. Wave 7: 3/4 done. S-029 UNBLOCKED (sole remaining). 31/33 done (187/195 pts). sprint-state v1.36. STORY-INDEX v5.28. STATE v6.76→v6.77. | 2026-06-01 | state-manager |
| D-229 | Zero-context resume checkpoint v6.78. S-029 human-authorized 2026-06-02 — dispatch now. develop @ 1158e24 (CLAUDE.md durable checkpoint). factory-artifacts @ 47d6a39. BC-2.07.004/005 versions added to artifact table. STATE v6.77→v6.78. | 2026-06-02 | state-manager |
| D-230 | S-029 DELIVERED — PR #35 squash-merged to develop @ 48463fb (mergedAt 2026-06-03T04:35:15Z). Killer Scenario: <=6 keystrokes dual permission resolve. BC-2.06.022. Validates holdout HS-EXP-008. 3 consecutive CLEAN adversarial passes (BC-5.39.001). Security review PASS (0 findings). Wave 7 COMPLETE (4/4: S-027 #32, S-031 #33, S-028 #34, S-029 #35). 32/33 stories done (192/195 pts, 98%). sprint-state v1.36→v1.37. STORY-INDEX v5.28→v5.29. S-029 spec v1.3. STATE v6.78→v6.79. | 2026-06-03 | state-manager |
| D-231 | Wave-7-gate prerequisite sweep complete. SS-ipc v1.10.0→v1.11.0 (architect, F-S026-ADV1-LOW-002 PermissionDecisionKind naming reconciliation). BC-2.06.021 v1.0.6→v1.0.7 (PO, F-S027-DOC-001 PC-3 'or replaced' fix). BC-INDEX v1.33→v1.34. STORY-INDEX v5.29→v5.30 (story-writer, F-S025-ADV37-DEFER-001 AC ranges corrected + systematic sweep). Citation atomicity propagation: BC-2.05.001-008 + BC-2.06.023 SS-ipc Architecture Source rows; EVAL-INDEX + product-brief BC-INDEX rows. S-027/S-028 story frontmatter status → done. version-pin-registry.yaml updated. POL-11 PASS (246 active, 575 files). POL-12 PASS. RESOLVED: F-S026-ADV1-LOW-002, F-S027-DOC-001, F-S028-NIT-001, F-S025-ADV37-DEFER-001, FLAKY-TIMING-5MS, L-S027-004. New residual: ADR-HOOK-001-WIRING (HUMAN action), SIGTERM-TERMINAL-RESTORE, F-S028-NIT-002-DEFERRED. STATE v6.79→v6.80. | 2026-06-03 | state-manager |
| D-232 | Wave-7 gate PASSED — Phase 3 TDD Implementation COMPLETE. Gate results: gate-1 PASS (1514 tests, 0 failures; clippy+fmt CLEAN), gate-2 SKIP (no DTU clone story — DTU-CLONE-STORY added as Phase 4 prereq; zero hook-boundary files touched), gate-3 PASS (0 CRIT/HIGH, 1 MED F-W7G3-MED-001 fixed in scope via PR #37 @ 6811103), gate-4 PASS (all 4 wave-7 demos), gate-5 PASS (HS-EXP-008 score 1.0), gate-6 PASS (this update), mutation-testing SKIP (strict-mode only). F-S028-NIT-002/NIT-002-DEFERRED both RESOLVED. sprint-state v1.37→v1.38 (wave_7_gate_status: passed). POL-11 PASS (248 active, 578 files). POL-12 PASS. NEXT: Phase 3→4 transition gate (consistency audit + human approval) → Phase 4. STATE v6.80→v6.81. | 2026-06-03 | state-manager |
| D-233 | Phase-3→4 consistency cleanup — all MED/LOW audit findings RESOLVED. EVAL-INDEX v1.8→v1.9 (PO, S-027 input added). BC-HOOK-034 v1.0.1→v1.0.2 (PO, deprecated_by typo). STORY-INDEX v5.30→v5.31 (story-writer, 9 Wave-2 stories draft→done + BC-2.05.004 coverage PARTIAL + story-count fix + EPIC-05 S-032). sprint-state v1.38→v1.39 (S-032 exclusion note + LOW-004 bulk flip note). 28 story-file status fields corrected (15 draft→done, 12 not_started→done, 1 in_progress→done). version-pin-registry: EVAL-INDEX 1.9 + STORY-INDEX 5.31 anchors set + S-027 added. Input-hash refresh: 113+1 stale → 114 updated, 17 residual bookkeeping-class (convergence limit — not content drift; POL-11/POL-12 PASS). BC-HOOK-034-typo RESOLVED. STATE v6.81→v6.82. | 2026-06-03 | state-manager |
| D-234 | DTU clone false-negative corrected — S-DTU-001 clone (dtu-claude-code-hooks-v1 cargo binary, crates/monocle-test-harness/src/bin/dtu_server.rs) validated on develop @ 90ae584: fidelity mean 1.0000 (25/25 fixtures, threshold 0.95), all 5 endpoints covered (pre-tool-use/notification/stop/session-start/prompt-submit), X-Claude-Code-Ide-Authorization header correct, BC-HOOK-034 filter passes, clippy + semgrep CLEAN. GATE-2 DTU-VALIDATION corrected from SKIP (false-negative) to PASS. Wave-7-gate-report.md Gate-2 annotated with D-234 correction. DTU-CLONE-STORY closed (RESOLVED-FALSE-PREMISE — blocking: false). dtu_clones_built updated to 2026-05-28. Phase 4 holdout-eval gate UNBLOCKED. Process gap: dtu-validator and consistency-validator searched for .factory/dtu-clones/ docker dir and missed cargo-binary clone location (S-DTU-001). PROC-DTU-VALIDATE-LOCATION added. POL-11/POL-12 PASS. STATE v6.82→v6.83. | 2026-06-03 | state-manager |
| D-235 | Daemon-wiring integration CONVERGED — monocle-runtime binary now actually serves (was a sleep-loop stub). main() wires daemon_start_sequence + run_server + UDS listener + tracing subscriber (tracing-subscriber 0.3) + durable ring-flush shutdown + 10s drain timeout. 16+ adversarial passes over 6 fix rounds, converged to CLEAN. Code on feat/daemon-wire-serve (PR pending merge). Spec artifacts: SS-daemon-wiring-impl v1.3.0 (new — architect's implementation plan + Round 1/2/3 fix addenda), SS-deps-pin-manifest v1.2.1 (tracing-subscriber 0.3 prod dep + ureq/libc dev-deps), ARCH-INDEX v1.0.26 (SS-daemon-wiring-impl row), STORY-INDEX v5.32, sprint-state v1.40 (S-032 + S-DAEMON-WIRE-FIX-001 Wave-8 entries formalized). HIGH-2 second-signal exit codes (143/130) explicitly deferred to S-DAEMON-WIRE-FIX-001 (Wave 8, P1, 5pts). RESOLVED: ADV-W5GATE-HIGH-001 (DaemonState wiring), ADV-W3GATE-MED-002/004 (ring never Some), ADV-W4GATE-MED-002 (no tracing subscriber), S-005-main-wiring (partial), F-DW-HIGH-001 (CI false-green). ADV-W5GATE-HIGH-002 (duplicate dead handler) re-confirmed open. factory-artifacts pushed before daemon-wire code PR to satisfy CI POL-11. POL-11/POL-12 PASS. STATE v6.83→v6.84. | 2026-06-03 | state-manager |
| D-236 | PRODUCT-VISION PIVOT — monocle becomes a full TUI control center (launch/manage/observe/tune/control; multi-session, multi-project; never leave the TUI). "A better lazyclaude AND a better claude-squad." Observe-only / no-orchestration principle from vision-synthesis v1.1.2 (approved 2026-05-11) RETIRED — specifically reverses: "inherit PM/Worker orchestration — rejected" and "execute workflows — rejected — observe-only." Phase-1 substrate (daemon-now-serves, hook ingestion, permission overlay, EngineModule/FactoryAdapter, proto, ring, TUI rendering) is BUILT and REUSABLE. VSDD Phases 4-7 (old observe-only scope) SUSPENDED. Next session: facilitate vision revision (NEXT-SESSION-PIVOT.md is the seed), redo gene-source disposition (claude-squad + zellij first), then delta brief→architecture→stories. Handoff canonical: /Users/jmagady/Dev/monocle/NEXT-SESSION-PIVOT.md. S-032 + S-DAEMON-WIRE-FIX-001 (Wave 8) and all non-blocking durable_task items remain valid but subordinate to pivot. STATE v6.84→v6.85. | 2026-06-03 | human+state-manager |
| D-237 | Human ratified the re-baselined-v1 control-center vision scope (following D-236 pivot). DECISIONS: (1) Re-baselined v1 (not additive Phase-1.5) — control-center IS the real v1; Phase-1 substrate preserved and extended. (2) v1 capability scope = ALL FOUR: Launch (monocle spawns and OWNS harness sessions from the TUI), Embedded PTY pane (running session visible/interactive INSIDE monocle via tui-term), Multi-session/multi-project management (list/switch/create/kill/rename, grouped by project), Interactive Tune (Static plane becomes interactive: edit/apply bindings, profiles, CCR model routing) — plus the already-built Observe + Control (permission overlay). (3) Cross-restart session PERSISTENCE is a HARD v1 requirement (sessions survive monocle/daemon restart; detach/reattach) — constrains architecture toward DAEMON-OWNS-PTY locus (daemon owns PTYs; PTY bytes traverse existing UDS IPC). Architect to formalize via ADR. (4) Hook auto-injection on spawn is v1 (launch ownership carries hook-wiring ownership; removes today's manual settings.json copy step). EMBEDDED-PTY RESEARCH (.factory/specs/research/embedded-pty-evaluation.md v1.0): primary recommendation = native in-process PTY (portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4); ratatui 0.30 compatibility verified at manifest level (tui-term 0.3.4 shares ratatui-core ^0.1.0 / ratatui-widgets ^0.3.0); MIT, no RUSTSEC, does NOT raise Phase-1 MSRV floor 1.88; runner-up = tmux control-mode (claude-squad style, external dep + fidelity ceiling); zellij = architecture-model-only. Architect-routed open questions deferred to architecture delta: PTY-vs-tmux ADR, trait-vs-SessionManager component, PTY-over-IPC throughput benchmark, EngineModule lifecycle extension. NEXT: gene-source disposition pass (claude-squad + zellij first) → revised vision-synthesis doc → human vision gate → delta brief → architecture → stories. STATE v6.85→v6.86. | 2026-06-03 | human+state-manager |
| D-238 | Vision approval gate PASSED. domain-monocle-vision-synthesis.md APPROVED at v2.1 by Joshua Magady as the canonical basis for the control-center re-baselined-v1 brief→architecture→story delta. HUMAN ESCALATION folded in at the gate: v1A persistence now REQUIRES that a graceful daemon-PROCESS restart SURVIVES (CASE 2 changed from 'lost' to 'survive'). Persistence principle renamed DAEMON-OWNS-PTY → 'session-host-owns-PTY; daemon coordinates/re-attaches': PTY masters + harness child processes owned by native detached per-session session-host processes (abduco/dtach-style) that outlive the daemon process; daemon re-attaches over UDS on restart. NO-TMUX preserved as default; external supervisor is architect-surfaced fallback only (requires human decision, not silent adoption). CASE 1 (TUI restart survives) and CASE 3 (hard crash → lost, re-launch) unchanged. New HIGH-priority architect question Q-8 (PTY-ownership-survival mechanism) added; NOTE: the already-built D-235 in-process daemon wiring will likely need rework to move PTY ownership out of the daemon process. Remaining architect-only open questions: Q-1 (PTY bytes over UDS), Q-2 (EngineModule/SessionManager surface), Q-7 (tui-term fork posture), plus PTY-throughput benchmark — all resolved during architecture delta. Architect must also reconcile the stale narrow keyboard scope in DISPOSITION-V2 rollup + embedded-pty-evaluation (superseded by full-fidelity ratification). NEXT: brief delta (product-owner) → architecture delta (architect) → story decomposition (story-writer). STATE v6.86→v6.87. | 2026-06-03 | human+state-manager |
| D-238-delta | product-brief.md v2.0.0 COMMITTED to factory-artifacts (control-center re-baseline, status draft; validate-brief v6.0 verdict VALID). input-hash 7e4f4f4 written. planning/brief-validation.md v6.0 (input-hash 1659922) committed alongside. Draft-commit: spec package goes through adversarial + human gate during/after architecture delta. Part of the D-238 delta progression (no new heavyweight D-number). STATE v6.87→v6.88. | 2026-06-03 | state-manager |

## Key Tech Stack (D-229 canonical)

ratatui 0.30 | crossterm 0.29 | tokio 1.52 | axum 0.8 | interprocess 2.4 | prost 0.14
serde_yaml_ng 0.10 | wasmtime 44 | nucleo 0.5 | time 0.3.47 (RUSTSEC-2026-0009 floor)
serde_json =1.0.149 | rand =0.8.6 | 28 pinned deps. SS-deps-pin-manifest v1.2.1 (D-235: +tracing-subscriber 0.3 prod, +ureq/libc dev-deps).
MSRV: Rust 1.88 (Phase 1-2); Rust 1.92 (Phase 3, wasmtime 44).
**PRD v1.27.4** | **BC-INDEX v1.34** (113 BCs) | **ARCH-INDEX v1.0.26** | **SS-tui v1.8.2**
**SS-ipc v1.11.0** | **SS-conventions v1.32.6** | **ADR-0007 v1.0.8** | **ADR-0008 v1.0.6** | **STORY-INDEX v5.32**
**SS-deps-pin-manifest v1.2.1** | **SS-daemon-wiring-impl v1.3.0** (NEW)
**BC-2.05.004 v1.1.0** | **BC-2.06.006 v1.1.0** | **BC-2.06.015 v1.0.7** | **BC-2.06.016 v1.1.0** | **BC-2.06.018 v1.1.0**
**BC-2.06.019 v1.1.0** | **BC-2.06.020 v1.1.0** | **BC-2.06.021 v1.0.7** | **BC-2.06.023 v1.5.1** | **BC-2.06.024 v1.1.0**
**BC-2.07.004 v1.0.2** | **BC-2.07.005 v1.3.1** | **BC-HOOK-034 v1.0.2** | **S-026 v1.11** | **S-027 v1.10** | **product-brief v2.0.0 (draft)**
**EVAL-INDEX v1.9** | **STORY-INDEX v5.32** | **sprint-state v1.40** (32/33 done, 192/195 pts; wave-7 gate PASSED D-232). **S-029 v1.3**. 62 codified disciplines. D-235: Daemon-wiring CONVERGED. D-233: Phase-3→4 consistency cleanup COMPLETE.
9 workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui (S-025).

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (v5.89..v6.77) | `cycles/cycle-001/burst-log.md` |
| Decisions D-047 through D-221 | `cycles/cycle-001/decisions-archive.md` |
| Phase 1 convergence history (R62-R122) | `cycles/cycle-001/phase-1-convergence.md` |
| Task queue (T-1 through T-131) | `cycles/cycle-001/completed-tasks.md` |
| Lessons learned (all rounds) | `cycles/cycle-001/lessons.md` |
| Prior session checkpoints (through v6.56) | `cycles/cycle-001/session-checkpoints.md` |
| Adversary reports | `cycles/cycle-001/S-025/adversarial-pass-*.md` |
| Resolved blocking issues | `cycles/cycle-001/blocking-issues-resolved.md` |
| CODIFY-001 sweep protocol reference (Categories 1-11) | `cycles/cycle-001/burst-log.md` (D-207 archive) |

## §Trace v6.88 (D-238-delta — product-brief.md v2.0.0 committed; architecture delta next)

**D-238-delta (2026-06-03):** product-brief.md v2.0.0 COMMITTED to factory-artifacts
(control-center re-baseline, status draft). validate-brief verdict VALID (planning/brief-validation.md
v6.0). input-hash written: 7e4f4f4 (product-brief.md), 1659922 (brief-validation.md).
Draft-commit — spec package goes through adversarial + human gate during/after architecture
delta. Part of the D-238 delta progression. NEXT: architecture delta (architect) →
Q-1/Q-2/Q-7/Q-8 HIGH + SessionManager/session-host/embedded-PTY subsystem design +
D-235 rework scope + input-doc reconciliation → story decomposition. v6.87→v6.88.

**D-238 (2026-06-03):** Vision approval gate PASSED. domain-monocle-vision-synthesis.md
APPROVED at v2.1 by Joshua Magady as the canonical basis for the control-center
re-baselined-v1 brief→architecture→story delta. HUMAN ESCALATION folded in at the gate:
v1A persistence now REQUIRES that a graceful daemon-PROCESS restart SURVIVES (CASE 2
changed from 'lost' to 'survive'). Persistence principle renamed DAEMON-OWNS-PTY →
'session-host-owns-PTY; daemon coordinates/re-attaches': PTY masters + harness child
processes owned by native detached per-session session-host processes (abduco/dtach-style)
that outlive the daemon process; daemon re-attaches over UDS on restart. NO-TMUX preserved
as default; external supervisor is architect-surfaced fallback only (requires human
decision, not silent adoption). CASE 1 (TUI restart survives) and CASE 3 (hard crash →
lost, re-launch) unchanged. New HIGH-priority architect question Q-8
(PTY-ownership-survival mechanism) added; NOTE: the already-built D-235 in-process daemon
wiring will likely need rework to move PTY ownership out of the daemon process. Remaining
architect-only open questions: Q-1 (PTY bytes over UDS), Q-2 (EngineModule/SessionManager
surface), Q-7 (tui-term fork posture), plus PTY-throughput benchmark — all resolved during
architecture delta. Architect must also reconcile the stale narrow keyboard scope in
DISPOSITION-V2 rollup + embedded-pty-evaluation (superseded by full-fidelity ratification).
NEXT: brief delta (product-owner) → architecture delta (architect) → story decomposition
(story-writer). v6.86→v6.87.

**D-237 (2026-06-03):** Human ratified the re-baselined-v1 control-center vision
scope (following D-236 pivot). DECISIONS: (1) Re-baselined v1 (not additive Phase-1.5)
— control-center IS the real v1; Phase-1 substrate preserved and extended. (2) v1
capability scope = ALL FOUR: Launch (monocle spawns and OWNS harness sessions from the
TUI), Embedded PTY pane (running session visible/interactive INSIDE monocle via
tui-term), Multi-session/multi-project management (list/switch/create/kill/rename,
grouped by project), Interactive Tune (Static plane becomes interactive: edit/apply
bindings, profiles, CCR model routing) — plus the already-built Observe + Control
(permission overlay). (3) Cross-restart session PERSISTENCE is a HARD v1 requirement
(sessions survive monocle/daemon restart; detach/reattach) — constrains architecture
toward DAEMON-OWNS-PTY locus (daemon owns PTYs; PTY bytes traverse existing UDS IPC).
Architect to formalize via ADR. (4) Hook auto-injection on spawn is v1 (launch
ownership carries hook-wiring ownership; removes today's manual settings.json copy
step). EMBEDDED-PTY RESEARCH (.factory/specs/research/embedded-pty-evaluation.md v1.0):
primary recommendation = native in-process PTY (portable-pty 0.9.0 + vt100 0.16.2 +
tui-term 0.3.4); ratatui 0.30 compatibility verified at manifest level (tui-term 0.3.4
shares ratatui-core ^0.1.0 / ratatui-widgets ^0.3.0); MIT, no RUSTSEC, does NOT raise
Phase-1 MSRV floor 1.88; runner-up = tmux control-mode (claude-squad style, external
dep + fidelity ceiling); zellij = architecture-model-only. Architect-routed open
questions deferred to architecture delta: PTY-vs-tmux ADR, trait-vs-SessionManager
component, PTY-over-IPC throughput benchmark, EngineModule lifecycle extension. NEXT:
gene-source disposition pass (claude-squad + zellij first) → revised vision-synthesis
doc → human vision gate → delta brief → architecture → stories. v6.85→v6.86.

**D-236 (2026-06-03):** PRODUCT-VISION PIVOT. monocle pivots from observe-only to
a full TUI control center: launch + manage + observe + tune + control; many sessions,
many projects; never leave the TUI. "A better lazyclaude AND a better claude-squad."
Human decision. The observe-only / no-orchestration constraint (vision-synthesis v1.1.2,
approved 2026-05-11) is RETIRED — specifically reverses the two rejected genes:
"inherit PM/Worker orchestration — rejected" and "execute workflows — rejected —
observe-only." Phase-1 substrate (daemon-now-serves, hook ingestion, permission overlay,
EngineModule/FactoryAdapter traits, proto, ring, TUI rendering) is BUILT and REUSABLE.
VSDD Phases 4-7 of the old observe-only scope SUSPENDED pending vision revision.
Handoff: /Users/jmagady/Dev/monocle/NEXT-SESSION-PIVOT.md committed to develop.
CLAUDE.md updated with D-236 pivot banner. durable_task_register: PIVOT-CONTROL-CENTER
added (active). S-032 and S-DAEMON-WIRE-FIX-001 (Wave 8) remain valid but subordinate.
STATE v6.84→v6.85.

**D-235 (2026-06-03):** Daemon-wiring integration CONVERGED. monocle-runtime binary
now actually serves (was a sleep-loop stub). main() wires daemon_start_sequence + run_server
+ UDS listener + tracing subscriber (tracing-subscriber 0.3) + durable ring-flush shutdown
+ 10s drain timeout. 16+ adversarial passes over 6 fix rounds, converged to CLEAN.
Code on feat/daemon-wire-serve (PR pending merge). Spec artifacts committed: SS-daemon-wiring-impl
v1.3.0 (NEW — architect's implementation plan + Round 1/2/3 fix addenda); SS-deps-pin-manifest
v1.2.1 (tracing-subscriber 0.3 as prod dep, ureq/libc as dev-deps); ARCH-INDEX v1.0.26
(SS-daemon-wiring-impl Document Map row); STORY-INDEX v5.32 + sprint-state v1.40
(S-DAEMON-WIRE-FIX-001 Wave-8 deferral anchor formalized, S-032 Wave-8 entry formalized).
HIGH-2 second-signal exit codes (DaemonExit::SigtermDuringDrain exit 143 / SigintDuringDrain
exit 130) explicitly deferred to S-DAEMON-WIRE-FIX-001 (Wave 8, P1, 5pts, EPIC-04) per
CANONICAL PRINCIPLE rule 3. CONTRACT GAP markers in lifecycle.rs. RESOLVED: ADV-W5GATE-HIGH-001
(DaemonState wiring), ADV-W3GATE-MED-002/004 (ring never Some), ADV-W4GATE-MED-002 (no tracing
subscriber), S-005-main-wiring (partial), F-DW-HIGH-001 (CI false-green). ADV-W5GATE-HIGH-002
(duplicate dead handler) re-confirmed still open. factory-artifacts pushed before daemon-wire code
PR to satisfy CI POL-11 (SS-deps-pin-manifest v1.2.1 must be on factory-artifacts before CI runs).
POL-11/POL-12 PASS. v6.83→v6.84.

**D-234 (2026-06-03):** DTU clone false-negative corrected. S-DTU-001 (cargo binary
dtu-claude-code-hooks-v1, crates/monocle-test-harness/src/bin/dtu_server.rs) validated on
develop @ 90ae584: fidelity mean 1.0000 (25/25 fixtures, threshold 0.95), all 5 endpoints
covered (pre-tool-use, notification, stop, session-start, prompt-submit),
X-Claude-Code-Ide-Authorization header correct, BC-HOOK-034 filter passes, clippy + semgrep
CLEAN. Gate-2 DTU-VALIDATION corrected from SKIP to PASS in wave-7-gate-report.md (D-234
annotation). DTU-CLONE-STORY closed: RESOLVED-FALSE-PREMISE (blocking: false). dtu_clones_built
updated to 2026-05-28. Phase 4 holdout-eval gate UNBLOCKED. Process gap: dtu-validator and
consistency-validator searched for .factory/dtu-clones/ docker dir and missed cargo-binary clone
location. PROC-DTU-VALIDATE-LOCATION added. POL-11/POL-12 PASS. v6.82→v6.83.

**D-233 (2026-06-03):** Phase-3→4 consistency cleanup (D-233). All MED/LOW audit findings
RESOLVED. EVAL-INDEX v1.8→v1.9 (PO: added S-027 as input). BC-HOOK-034 v1.0.1→v1.0.2 (PO:
deprecated_by typo fixed). STORY-INDEX v5.30→v5.31 (story-writer: 9 Wave-2 stories draft→done,
BC-2.05.004 coverage PARTIAL, story-count fix, EPIC-05 S-032 added). sprint-state v1.38→v1.39
(S-032 exclusion note + LOW-004 bulk flip note). 28 story-file status fields corrected
(15 draft→done S-001..S-015, 12 not_started→done S-016..S-026/S-030/S-031, 1 in_progress→done
S-022). version-pin-registry: EVAL-INDEX 1.9 + STORY-INDEX 5.31 D-233 anchors + S-027 added.
Input-hash refresh: 113+1=114 updated, 17 residual bookkeeping-class (convergence limit —
not content drift). POL-11 PASS (251 active, 578 files). POL-12 PASS. BC-HOOK-034-typo
RESOLVED. v6.81→v6.82.

**D-232 (2026-06-03):** Wave-7 integration gate PASSED. Phase 3 TDD Implementation COMPLETE
(all 7 waves delivered and gated). Gate results: gate-1 PASS (1514 tests, 0 failures; clippy
--all-targets + fmt CLEAN on develop @ 6811103), gate-2 SKIP (DTU clone story not decomposed
in Phase 2; wave-7 touched zero hook-boundary files; DTU-CLONE-STORY added to durable_task_register
as Phase 4 prereq with blocking: true for Phase 4 holdout-eval gate), gate-3 PASS (cross-story
wave-diff adversarial review: 0 CRIT/HIGH/1 MED F-W7G3-MED-001 FIXED IN SCOPE via PR #37 @
6811103 — render_sessions_filter index-space remap + render test; pr-reviewer + security-reviewer
CLEAN; 9 CI checks green), gate-4 PASS (all 4 wave-7 stories demoed; S-029 @ fdf1a31;
S-027/S-031 @ b2c8635; S-028 @ 70418e7; all ACs covered), gate-5 PASS (HS-EXP-008 score 1.0;
black-box info-asymmetric; killer scenario ≤6 keystrokes validated), gate-6 PASS (this state
update; sprint-state v1.38; CLAUDE.md D-232 checkpoint committed to develop), mutation-testing
SKIP (all wave-7 stories tdd_mode: strict; no facade stories). F-S028-NIT-002 and
F-S028-NIT-002-DEFERRED both RESOLVED (fixed in scope as F-W7G3-MED-001). POL-11 PASS
(248 active, 578 files). POL-12 PASS. sprint-state v1.37→v1.38. develop @ 6811103. v6.80→v6.81.

§Trace v6.40 through v6.82 archived to `cycles/cycle-001/burst-log.md`.
