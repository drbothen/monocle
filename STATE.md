---
document_type: pipeline-state
level: ops
project: monocle
version: "6.32"
status: active
producer: state-manager
timestamp: 2026-05-28T22:00:00Z
phase: phase-3-wave-6-IN-PROGRESS
current_step: "S-025 Pass 11 fix round COMPLETE (4563bfa, 740465d, 983d30a, 1aba802, fd6c81a). Counter 0/3. Pass 12 adversary pending dispatch. S-026 still BLOCKED on S-025 merge."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "Phase 1 GATE-PASS-WITH-RESIDUAL (D-155). Phase 2 GATE-PASS-WITH-RESIDUAL (D-159). Phase 3 Wave 1 DONE (D-164), Wave 2 GATE-PASSED (D-166), Wave 3 GATE-PASSED (D-167), Wave 4 GATE-PASSED (D-175), Wave 5 GATE-PASSED (D-182). Phase 1d CONVERGED (D-169, D-170). Phase 2 expansion adversarial CONVERGED (D-172). See cycles/cycle-001/ for full convergence history. Wave 6 AUTHORIZED (D-183). S-022 DELIVERED (D-184). S-023 DELIVERED (D-186). S-025 D-187 PENDING (Pass 11 fix round complete; Pass 12 awaiting)."
awaiting: "S-025 convergence (counter 0/3; need 3 consecutive NITPICK_ONLY). After S-025 merges: S-026 (13pts) dispatch."
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
  COLD-START RESUME GUIDE — S-025 PASS 11 COMPLETE / PASS 12 PENDING (STATE v6.32):

  SESSION CONTEXT:
    monocle Wave 6: S-022 (D-184) + S-023 (D-186) merged. S-025 in flight —
    11 adversarial passes deep, counter 0/3. S-026 blocked on S-025 merge.
    Read CLAUDE.md at the repo root for project principles before dispatching any agent.
    The production-grade-default principle in CLAUDE.md overrides all agent prompt defaults.

  PIPELINE STATE:
    develop @ 7a52041 (S-023 merge, PR #29, 2026-05-28T19:31:07Z).
    26/33 stories done (156/195 pts, 80%). 852+ tests. clippy clean. fmt clean.
    S-025 PR #28 draft. Local worktree at .worktrees/S-025/.
    S-025 HEAD: 1aba802 (test-writer assertion sweep) or later —
      verify: gh pr view 28 --json statusCheckRollup,headRefOid

  PASS 11 FIX ROUND (ALL COMPLETE — 5 commits):
    4563bfa — PO Option B: BC-2.06.016 v1.0.7→v1.0.8 (production bracketed style wins;
              rationale at .factory/cycles/cycle-001/S-025/text-style-adjudication.md)
    740465d — Architect: SS-tui v1.8.1→v1.8.2 (line 668 prose→bracketed propagated;
              input-hash 958ae3b→31b6e71)
    983d30a — Implementer: 6 pub const extractions (DAEMON_DISCONNECT_STATUS,
              DAEMON_OFFLINE_STATUS, MONOCLE_STATUS_LABEL, TOKEN_COUNT_OVERFLOW_CAP,
              UPTIME_OVERFLOW_CAP) + format_drop_counter(n) helper; LOW-001 redundant
              cost<0.0 guard removed
    1aba802 — Test-writer: 5 literal→const/helper substitutions in startup_connect.rs
    fd6c81a — Story-writer: S-026 v1.6→v1.7 (BC-2.06.016 v1.0.8 pin); BC-INDEX
              v1.26→v1.27; STORY-INDEX v5.8→v5.9; dependency-graph v1.7→v1.8

  ARTIFACT VERSIONS (post-Pass-11 fix round):
    STORY-INDEX v5.9. sprint-state v1.30 (26/33 done, 156/195 pts).
    BC-INDEX v1.27 (113 BCs). ARCH-INDEX v1.0.16. PRD v1.27.2.
    SS-tui v1.8.2. SS-ipc v1.8.0. SS-conventions v1.31.0. SS-engine-module v1.1.22.
    BC-2.06.016 v1.0.8. S-025 v1.6. S-026 v1.7.
    NOTE: verify SS-tui pin in S-025 v1.6 frontmatter is v1.8.2 (NOT v1.8.1).

  S-025 ADVERSARIAL PASS TRAJECTORY (11 passes; counter 0/3):
    Pass 1: BLOCKER — 5 BLOCKERs (render path empty, keyboard dispatch, hardcoded dash,
             EnrichedSession fields, ring_tail dropped, duplicate InitialState)
    Pass 2: BLOCKER — CRITICAL read_framed cancellation-unsafe + BLOCKER HH:MM:SS uptime.
             Architect Option B: dedicated reader task + mpsc(64).
    Pass 3: BLOCKER — SS-ipc not bumped; BC-2.06.004 PC-2 stale Overlay shape.
    Pass 4: BLOCKER — SS-tui §Sessions Panel 6 vs 7 columns + BC-2.06.005 column drift.
    Pass 5: BLOCKER — SS-tui column table missed in Pass 4 + BC-2.06.005 PC-2 drift +
             S-024 stale shape + SS-tui input-hash + BC-2.06.014 EC-096 missed.
    Pass 6: HIGH — test-name-vs-assertion drift (_verbatim_match used truncated substring).
    Pass 7: MED — Pass 6 class recurrence in _j_key_moves_selection_down + uptime doc.
    Pass 8: NITPICK_ONLY — 1/3 CLEAN (first clean pass).
    Pass 9: MED — float edge cases (NaN/inf in format_cost) + AC-003 canonical text untested.
    Pass 10: MED — Pass 9 sweep was vacuous-mirror (TestBackend local copy, not production)
              + PC-6 selected-row highlight untested. Counter reset to 0/3.
    Pass 11: HIGH — sibling-extraction gap + BC-2.06.016 PC-4 spec drift on disconnect text.
             Fix round complete (see commits above). Counter remains 0/3.

  NEXT ACTION — DISPATCH S-025 PASS 12 ADVERSARY:
    Use Agent tool, subagent_type: vsdd-factory:adversary.
    Background dispatch. Prompt must include:
      - Counter 0/3; need 3 consecutive NITPICK_ONLY for convergence
      - Pass 11 fix-round commits to verify: 4563bfa, 740465d, 983d30a, 1aba802, fd6c81a
      - Full Pass 1-11 trajectory (see above)
      - Maximum-skepticism mode
      - Premature-clean signal warning: Pass 8 clean→Pass 9 NEW class. Pass 12 must look harder.
      - Verify all 6 pub const extractions present in production code
      - Verify S-025 v1.6 frontmatter SS-tui pin is v1.8.2 (NOT v1.8.1)
      - Verify S-026 v1.7 frontmatter BC-2.06.016 pin is v1.0.8
      - Run: cargo test/clippy/fmt on the .worktrees/S-025/ worktree
      - Check PR #28 CI: gh pr view 28 --json statusCheckRollup
      - If clean: counter → 1/3, dispatch Pass 13 immediately.
      - If findings: route to fix → retry Pass 12.

  AFTER S-025 CONVERGENCE (3/3 NITPICK_ONLY):
    1. Rebase S-025 onto develop @ 7a52041 (S-023 changes).
    2. Resolve TODO(S-023-merge) markers at app.rs:586-615 and app.rs:630:
       replace with monocle_ipc::events::TransportEvent +
       monocle_ipc::reconnect::reconnect_with_backoff imports (instructions inline).
    3. Dispatch demo-recorder for 10 ACs (per-AC evidence).
    4. Dispatch pr-manager for 9-step PR lifecycle (PR #28 draft → review → merge).
    5. Dispatch state-manager for D-187 closure + STATE.md v6.33.
    6. S-026 (13 pts, EPIC-06) unblocked — dispatch immediately.

  CRITICAL FILES FOR PASS 12 ADVERSARY (read in order):
    1. .factory/STATE.md (this file — pass trajectory + fix history)
    2. .factory/cycles/cycle-001/S-025/architect-decisions-pass-1.md
    3. .factory/cycles/cycle-001/S-025/architect-decisions-pass-2.md
    4. .factory/cycles/cycle-001/S-025/text-style-adjudication.md (PO Option B)
    5. .factory/cycles/cycle-001/S-025/red-gate-log.md
    6. .factory/stories/S-025-tui-skeleton-sessions.md (v1.6; verify SS-tui pin v1.8.2)
    7. CLAUDE.md (project principles — production-grade default)

  NON-BLOCKING FOLLOW-UPS (do NOT fix unless explicitly tasked):
    See durable_task_register in this file for full list.
    S-025-specific: S-025-TODO-S023-MERGE, S-025-MAKE-MODAL-DEAD-CODE.
    S-022/S-023 carry-overs: ADV-W5GATE-HIGH-001, ADV-W5GATE-HIGH-002,
    ADV-W5GATE-MED-001, ADV-W5GATE-MED-003, ADV-W4GATE-MED-002, HS-EXP-009-hint.
    Process: PROC-SEMGREP-DECOUPLE, PROC-GATE-SKIPPED-LOUD, PROC-COMPUTE-INPUT-HASH-YAML,
    PROC-BRANCH-PROTECTION-CONTEXTS.

  S-025 CYCLE LESSONS (encode for adversary; see lessons.md L-W6-S025-001..007):
    L-W6-S025-001: Architect-decision propagation sweeps must cover BC bodies + SS docs +
      story frontmatter + story body language + input-hashes — all five.
    L-W6-S025-002: Sweep-audit must trace assertion to EXACT production code path.
      Substring on TestBackend-local buffer is NOT production-rendered.
    L-W6-S025-003: pub const extraction eliminates vacuous-mirror class structurally.
      Re-export from lib.rs for external test crates.
    L-W6-S025-004: Premature-clean signal at counter-advance moments is confirmed.
      Pass 8 clean → Pass 9 new class. Apply maximum skepticism at every counter-advance.
    L-W6-S025-005: IEEE 754: is_sign_negative() for negative-zero; NaN check first.
    L-W6-S025-006: SS docs (architecture sources) are required propagation targets alongside
      BC bodies. Pass 5 missed SS-tui — same root cause as L-W6-S025-001.
    L-W6-S025-007: Sweep wider than the finding. Principle 4 (fix in scope) implies
      catching ALL class siblings, not only the flagged instance.

  FACTORY INFRASTRUCTURE:
    .factory/ mounted at factory-artifacts branch (orphan worktree @ .factory/).
    Run factory-worktree-health via devops-engineer FIRST on session start.
    Commit hooks: block-ai-attribution, validate-input-hash, validate-table-cell-count.
    NEVER use --no-verify. NEVER add Co-Authored-By: Claude.
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
| 3 TDD Implementation | IN PROGRESS — Wave 6 2/4 done; S-025 Pass 11 → counter 0/3 | 2026-05-28 | Wave 1+2+3 DONE (83 pts, 447 tests, all 6 gates). Wave 4 GATE PASSED (D-175): 634 tests. Wave 5 GATE PASSED (D-182): 753 tests, 0 failures, clippy clean, fmt clean. Wave 6: 2/4 done (S-022 8pts + S-023 5pts). 26/33 stories done (156/195 pts). S-025 Pass 11 fix round complete (counter 0/3). S-026 blocked on S-025. |
| 4-7 | not-started | — | |

## Wave 5 — GATE PASSED (D-182)

| Story | Points | Status | Notes |
|-------|--------|--------|-------|
| S-017 Daemon Start Sequence + Hook Tmpfile | 8 | done | PR #22, 06432cf, 29 tests, adv 13→5→0 (3 passes) |
| S-018 Hook Endpoint Routing + Event Bus | 8 | done | PR #26, 654e281, 46 tests, adv 10→4→4 (CONVERGED) |
| S-019 Daemon Auto-Start + MONOCLE_NO_AUTOSTART | 5 | done | PR #25, 11540fc, 25 tests, adv 7→2→1 (CONVERGED) |
| S-020 JSONL Ring Capacity and Rotation | 5 | done | PR #24, f69d53a, 24 tests, adv 12→8→0 (CONVERGED) |
| S-021 UDS Server + IPC Transport + Core Message Types | 8 | done | PR #23, acaacb9, 49 tests, adv 9→4→4 (CONVERGED) |

develop @ 7a52041. 852+ tests, 0 failures. 26/33 stories done, 156/195 pts (80%). Wave 5 gate PASSED (D-182). Wave 6 in progress: S-022 DONE (D-184, PR #27 @ c7540539), S-023 DONE (D-186, PR #29 @ 7a52041). S-025 Pass 11 fix round complete (counter 0/3; Pass 12 adversary pending). S-026 blocked on S-025.

## Blocking Issues

None. All durable_task_register items non-blocking.

## Decisions Log (recent — D-175 through D-182)

D-047 through D-174 archived at: `cycles/cycle-001/decisions-archive.md`

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-175 | Wave 4 gate PASSED (634 tests, 0 failures, clippy/fmt clean). DTU SKIP. ADV PASS (0 CRIT/HIGH). Demo PASS (3/3). Holdout PASS (mean 0.90). develop @ b8a4ab7. | 2026-05-27 | orchestrator |
| D-176 | S-017 DELIVERED — PR #22 @ 06432cf, 29 tests, adv 13→5→0. BC-2.04.001/010 satisfied. | 2026-05-27 | orchestrator |
| D-177 | S-018 DELIVERED — PR #26 @ 654e281, 46 tests, adv 10→4→4. BC-2.04.007/008/009/011 satisfied. | 2026-05-27 | orchestrator |
| D-178 | S-019 DELIVERED — PR #25 @ 11540fc, 25 tests, adv 7→2→1. BC-2.04.002/003 satisfied. | 2026-05-27 | orchestrator |
| D-179 | S-020 DELIVERED — PR #24 @ f69d53a, 24 tests, adv 12→8→0. BC-2.04.012 satisfied. Fix: 5a3eaf4. | 2026-05-27 | orchestrator |
| D-180 | S-021 DELIVERED — PR #23 @ acaacb9, 49 tests, adv 9→4→4. BC-2.05.001/003/004/008 satisfied. New monocle-ipc crate. | 2026-05-27 | orchestrator |
| D-181 | Wave 5 COMPLETE — 5/5 stories done, 753 tests, 24/33 stories (143/195 pts). monocle-ipc crate added. | 2026-05-27 | orchestrator |
| D-182 | Wave 5 gate PASSED (753 tests, 0 failures, clippy/fmt clean). DTU SKIP. ADV PASS (0 CRIT, 0 HIGH blocking; 2 HIGH obs tracked). Demo PASS (5/5). Holdout PASS (mean 0.94, min 0.80). develop @ 1ce7838. Wave 6 unblocked. | 2026-05-27 | orchestrator |
| D-183 | Wave 6 AUTHORIZED — 4 stories (S-022, S-023, S-025, S-026), 34 pts. Execution order: S-022 serial-first → (S-023 ∥ S-025) → S-026. All dependencies satisfied. Human approval: "Approve as documented" (2026-05-27). | 2026-05-27 | orchestrator |
| D-184 | S-022 DELIVERED — PR #27 @ c7540539. BC-2.05.002 + BC-2.05.005 fully satisfied. 15 ACs. 15 adversarial passes (convergence at Pass 15). 8 implementer rounds + 2 architect interventions. BC-2.05.002 v1.0.5 (ring_tail Vec<HookEventRecord> per Option B). SS-ipc v1.8.0 (at-least-once delivery per Option D). 22 integration tests. New crate dependency: monocle-runtime now uses monocle-ipc for shared HookEventRecord. | 2026-05-28 | orchestrator |
| D-185 | Wave 6 parallel S-023 + S-025 AUTHORIZED. Human approval "Yes — parallel S-023 + S-025" (2026-05-28). Both dependencies satisfied; different crates (monocle-ipc reconnect logic vs new monocle-tui binary); independent. After both merge → S-026. | 2026-05-28 | orchestrator |
| D-186 | S-023 DELIVERED — PR #29 @ 7a52041 (2026-05-28T19:31:07Z). BC-2.05.006 (reconnect backoff + lock re-read + offline mode) + BC-2.05.007 (SOQ-3 overlay clear on disconnect) satisfied. 15 ACs. 5 adversarial passes (3 consecutive NITPICK_ONLY convergence). 99 tests in monocle-ipc. 9/9 CI gates pass. F-ADV6-HIGH-001 production-grade. ADV-W4GATE-MED-001 + F-S022-ADV15-LOW-001 closed in-cycle. | 2026-05-28 | orchestrator |

## Key Tech Stack

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14,
serde_yaml_ng 0.10, wasmtime 44, directories 6, notify 8, russh 0.60, rmcp 1.6,
reqwest 0.13, nucleo 0.5, nix 0.30, serde 1 (derive), chrono 0.4, serde_json =1.0.149 (EXACT), rand =0.8.6 (EXACT).
28 pinned production deps. **manifest v1.1.17**. **PRD v1.27.2**. **BC-INDEX v1.27** (72 numbered BCs + 41 DTU BCs = 113 total). **ARCH-INDEX v1.0.16** (7 subsystems). **SS-tui v1.8.2**. **STORY-INDEX v5.9** (33 stories, 195 pts). **sprint-state v1.30** (26/33 done, 156/195 pts, 80%). MSRV: Rust 1.86 (Phase 1-2); Rust 1.92 (Phase 3, wasmtime 44). 39 codified disciplines (SE-1..SE-23 + SE-40 candidate). Workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask.

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

## §Trace v6.32 (D-187 PENDING CHECKPOINT — S-025 Pass 11 fixes complete; Pass 12 awaiting)

**S-025 PASS 11 FIX ROUND COMPLETE** (2026-05-28): 11 adversarial passes; counter 0/3.

Pass 11 finding class: sibling-extraction gap + BC-2.06.016 PC-4 spec-impl drift on disconnect status text (production uses bracketed style; BC prose did not).

**Pass 11 fix-round commits (all landed in S-025 worktree):**
- 4563bfa — PO Option B: BC-2.06.016 v1.0.7→v1.0.8. Bracketed style wins per cross-doc consistency with [daemon: offline] and [dropped: N] patterns. Architect-decision record: `.factory/cycles/cycle-001/S-025/text-style-adjudication.md`.
- 740465d — Architect: SS-tui v1.8.1→v1.8.2. Line 668 prose→bracketed propagated. Input-hash 958ae3b→31b6e71.
- 983d30a — Implementer: 6 pub const extractions (DAEMON_DISCONNECT_STATUS, DAEMON_OFFLINE_STATUS, MONOCLE_STATUS_LABEL, TOKEN_COUNT_OVERFLOW_CAP, UPTIME_OVERFLOW_CAP) + format_drop_counter(n) helper. LOW-001 redundant cost<0.0 guard removed (subset of is_sign_negative()).
- 1aba802 — Test-writer: 5 literal→const/helper assertion substitutions in startup_connect.rs.
- fd6c81a — Story-writer: S-026 v1.6→v1.7 (BC-2.06.016 v1.0.8 pin); BC-INDEX v1.26→v1.27; STORY-INDEX v5.8→v5.9; dependency-graph v1.7→v1.8.

**Durable task register additions:** S-025-TODO-S023-MERGE, S-025-MAKE-MODAL-DEAD-CODE. PROC-COMPUTE-INPUT-HASH-YAML detail updated (recurring class).
**Artifact versions updated:** BC-INDEX v1.26→v1.27, STORY-INDEX v5.8→v5.9, SS-tui v1.8.1→v1.8.2, BC-2.06.016 v1.0.7→v1.0.8, S-026 v1.6→v1.7.
**7 new lessons codified:** L-W6-S025-001 through L-W6-S025-007 in cycles/cycle-001/lessons.md.
**next_session_resume_protocol rewritten end-to-end** for S-025 Pass 12 zero-context restart.
STATE v6.31 → v6.32.

## §Trace v6.31 (D-186 — S-023 DELIVERED, Wave 6 2/4 done)

**S-023 DELIVERED** (2026-05-28): PR #29 squash-merged at develop @ 7a52041 (2026-05-28T19:31:07Z). D-186.
- BC-2.05.006 (reconnect backoff + lock re-read + offline mode) + BC-2.05.007 (SOQ-3 overlay clear on disconnect) fully satisfied. 15 ACs. 5 adversarial passes (Pass 1-2 HIGH-PRIORITY: 3 HIGH + 4 MED + 3 LOW each; Passes 3-5 NITPICK_ONLY; 3/3 CONVERGED). 99 tests in monocle-ipc. 9/9 CI gates pass.
- F-ADV6-HIGH-001 (S-022 carry-over: slow-disconnect signal channel) closed in-scope via commit 9bddd7b. Production-grade per Pass 5 adversary.
- ADV-W4GATE-MED-001 (PATH isolation in detect_ccr) closed: temp_env::with_vars in commit 295dc1b (orchestrator-authorized scope expansion).
- F-S022-ADV15-LOW-001 (ring_tail type doc drift) closed: story v1.2→v1.3 in commit 545b634.
- IMPL-EnrichedSession-fields: closed (pending S-025 merge) via commit 9b84ef3 in S-025 branch.
- 5 process discoveries logged; 4 new durable task entries (PROC-SEMGREP-DECOUPLE, PROC-GATE-SKIPPED-LOUD, PROC-COMPUTE-INPUT-HASH-YAML, PROC-BRANCH-PROTECTION-CONTEXTS).
- 5 new lessons codified in cycles/cycle-001/lessons.md.
- sprint-state v1.29→v1.30: done 25→26, not_started 7→6, points_complete 151→156.
- STORY-INDEX v5.6→v5.7: S-023 row flipped not_started→done.
- Frontmatter: version 6.30→6.31, current_step + awaiting updated for S-025 pass 5 in flight.
STATE v6.30 → v6.31.

## §Trace v6.30 (DURABLE PAUSE CHECKPOINT — S-022 merged, Wave 6 1/4 done, S-023+S-025 parallel authorized)

**DURABLE PAUSE CHECKPOINT** (2026-05-28): Wave 6 1/4 done. S-022 merged at c7540539. Human authorized parallel S-023 + S-025. D-185.
- next_session_resume_protocol fully rewritten for zero-context fresh-session resume (see above).
- ADR-0006 (non_exhaustive constructors) + SS-conventions v1.31.0 added this cycle.
- BC-2.05.002 v1.0.5 Invariant 4 anchored in S-025 v1.3 + S-026 v1.3.
- S-022 lesson encoded in resume protocol: vacuous-mirror-test pattern, 3-consecutive NITPICK_ONLY threshold, deferral propagation verification.
- Frontmatter: version 6.29→6.30, awaiting → S-023 + S-025 parallel delivery.
STATE v6.29 → v6.30.

## §Trace v6.29 (S-022 DELIVERED — Wave 6 first delivery, D-184)

**S-022 DELIVERED** (2026-05-28): PR #27 merged at develop @ c7540539. D-184.
- BC-2.05.002 + BC-2.05.005 fully satisfied. 15 ACs. 15 adversarial passes (3 consecutive NITPICK_ONLY convergence at Pass 15). 8 implementer rounds + 2 architect interventions. 30+ findings closed. 22 production-invoking integration tests across 4 test files.
- BC-2.05.002 v1.0.5 (ring_tail Vec<HookEventRecord> per Pass 2 Option B). SS-ipc v1.8.0 (at-least-once delivery per Pass 6 Option D). TUI prompt_id idempotency anchored in S-025/S-026.
- New crate dependency: monocle-runtime now uses monocle-ipc for shared HookEventRecord.
- Unblocks: S-023 (TUI Reconnect), S-025 (TUI Skeleton), S-026 (Permission Overlay Core).
- F-S022-ADV15-LOW-001: story AC-002 ring_tail doc drift deferred to story-writer post-merge (blocking=false, future anchor: story v1.3). Durable task register entry unchanged.
- sprint-state v1.28→v1.29: done 24→25, not_started 8→7, points_complete 143→151.
- STORY-INDEX v5.3→v5.4: S-022 row flipped not_started→done. §Trace v5.4 appended.
- Frontmatter: version 6.28→6.29, phase remains phase-3-wave-6-IN-PROGRESS, awaiting → S-023 + S-025 parallel delivery.
STATE v6.28 → v6.29.

§Trace v6.27-v6.28 (S-022 Pass 13 NITPICK_ONLY + S-022 Pass 15 CONVERGED) archived to `cycles/cycle-001/burst-log.md`.

§Trace v6.22 through v6.26 (Wave 4+5 gates, Wave 6 authorization, Wave 5 complete, count correction) archived to `cycles/cycle-001/burst-log.md`.
