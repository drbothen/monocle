---
document_type: pipeline-state
level: ops
project: monocle
version: "7.15"
status: active
producer: state-manager
timestamp: 2026-06-04T03:00:00Z
phase: PIVOT-delta-in-progress
current_step: "D-264: Adversarial Pass-22 FINDINGS (COUNTER RESET 2→0). 0 Critical / 3 Important / 2 Suggestion. Fresh-context Pass-22 uncovered a partial-fix sibling CLUSTER in SS-09 BCs that all 21 prior passes (incl. the 2 clean ones) missed — I7/Killed-removal/O4 fixes propagated to SS-embedded-pty + BC-2.09.001 but never reached sibling BCs. I22-001 (BC-2.09.007 retained retired singular pty_scroll_offset throughout — would reintroduce focus-switch-wrong-scrollback bug) FIXED → per-session HashMap (1.0.0→1.1.0). I22-002 (BC-2.09.008 Inv-1 retained retired Killed + missing Terminating + Detached-no-op contradicting own PC-2) FIXED → reconciled per-state no-op set, Detached attachable, PC-4 HashMap (1.0.0→1.1.0). I22-003 (BC-2.09.007 Inv-2 stale ~4 bytes/cell memory bound) FIXED → ~16 bytes/cell, 12.8MB/session, ~102MB/8 per O4 (in 1.1.0). S22-001 (BC-2.05.010 PC-4 incomplete spawn-code set) FIXED → 6-code set, EC-280→binary_not_found (1.5.1→1.6.0). S22-002 (BC-2.08.005 Killed prose) FIXED (1.0.0→1.0.1). Exhaustive 4-class sweep (pty_scroll_offset / Killed-state / ~4-bytes / spawn-code) across all in-scope BCs confirmed zero remaining live survivors; closure grep verified all residuals are §Trace/canonical-negation. 4 registry entries bumped atomically. POL-11 PASS (328 active). POL-12 PASS. SIXTEENTH consecutive zero-Critical (C:...,0×16). CONSECUTIVE-CLEAN COUNTER = 0 (RESET — streak restarted). Pass-23 next = clean candidate 1 of 3."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-047..D-174 archived at cycles/cycle-001/decisions-archive.md. D-175: Wave 4 gate PASSED. D-182: Wave 5 gate PASSED. D-183: Wave 6 AUTHORIZED. D-184: S-022 DELIVERED. D-185: S-023+S-025 AUTHORIZED. D-186: S-023 DELIVERED. D-188..D-221: see Decisions Log (archived in this file). D-222: S-025 DELIVERED (PR #28 @ 838477e). D-223: S-026 DELIVERED (PR #30 @ 9fb0d70) — Wave 6 COMPLETE. D-224: Wave-6 GATE PASSED (develop @ 2a51a91). D-225: STATE correction — phase-3-COMPLETE premature (Wave 7 of 7 remains); corrected to wave-7-READY; points 177→169/195 reconciled to sprint-state. D-226: S-027 DELIVERED (PR #32 @ 3787ebd) — Wave 7: 1/4 done. D-227: S-031 DELIVERED (PR #33 @ 8451486). D-228: S-028 DELIVERED (PR #34 @ 682e5e5) — Wave 7: 3/4 done. develop @ 682e5e5. D-229: Zero-context resume checkpoint — S-029 human-authorized 2026-06-02. develop @ 1158e24. D-230: S-029 DELIVERED (PR #35 @ 48463fb) — Wave 7 COMPLETE (4/4). 32/33 done (192/195 pts). develop @ 48463fb. D-231: Wave-7-gate prerequisite sweep complete — SS-ipc v1.11.0, BC-2.06.021 v1.0.7, BC-INDEX v1.34, STORY-INDEX v5.30, citation atomicity propagation. POL-11/POL-12 PASS. S-027/S-028 story frontmatter status fixed. D-232: Wave-7 gate PASSED — Phase 3 COMPLETE. 1514 tests, 0 failures. F-W7G3-MED-001 fixed (PR #37 @ 6811103). HS-EXP-008 score 1.0. DTU SKIP (DTU-CLONE-STORY added as Phase 4 prereq). sprint-state v1.38. D-233: Phase-3→4 consistency cleanup — EVAL-INDEX v1.9, STORY-INDEX v5.31, BC-HOOK-034 v1.0.2 (typo fix), sprint-state v1.39. 28 story-file status fields corrected. Input-hash refresh (113→0+17 bookkeeping-residual). POL-11/POL-12 PASS. All MED/LOW audit findings RESOLVED. D-234: DTU clone false-negative corrected — S-DTU-001 (cargo binary dtu-claude-code-hooks-v1) validated fidelity 1.0000 (25/25 fixtures). Gate-2 DTU-VALIDATION corrected from SKIP to PASS. DTU-CLONE-STORY closed (RESOLVED-FALSE-PREMISE). Phase 4 UNBLOCKED. dtu_clones_built updated. PROC-DTU-VALIDATE-LOCATION process gap added. D-235: Daemon-wiring convergence — monocle-runtime binary now serves (main() wires daemon_start_sequence + run_server + UDS + tracing + ring-flush + 10s drain). SS-daemon-wiring-impl v1.3.0. SS-deps-pin-manifest v1.2.1. ARCH-INDEX v1.0.26. STORY-INDEX v5.32. sprint-state v1.40. S-DAEMON-WIRE-FIX-001 Wave-8 anchor. Resolved: ADV-W5GATE-HIGH-001, ADV-W3GATE-MED-002/004, ADV-W4GATE-MED-002, S-005-main-wiring, F-DW-HIGH-001. POL-11/POL-12 PASS. D-236: PRODUCT-VISION PIVOT — observe-only RETIRED; monocle → full TUI control center. Phases 4-7 SUSPENDED. D-237: Human ratified re-baselined-v1 control-center vision scope (4 capabilities: Launch, Embedded PTY, Multi-session/multi-project, Interactive Tune + already-built Observe+Control). DAEMON-OWNS-PTY locus. Hook auto-injection v1. embedded-pty-evaluation.md v1.0: primary = portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4. NEXT: gene-source disposition → revised vision-synthesis → human gate. D-238: Vision approval gate PASSED. domain-monocle-vision-synthesis.md APPROVED at v2.1 by Joshua Magady as the canonical basis for the control-center re-baselined-v1 brief→architecture→story delta. HUMAN ESCALATION folded in at the gate: v1A persistence now REQUIRES that a graceful daemon-PROCESS restart SURVIVES (CASE 2 changed from 'lost' to 'survive'). Persistence principle renamed DAEMON-OWNS-PTY → 'session-host-owns-PTY; daemon coordinates/re-attaches': PTY masters + harness child processes owned by native detached per-session session-host processes (abduco/dtach-style) that outlive the daemon process; daemon re-attaches over UDS on restart. NO-TMUX preserved as default; external supervisor is architect-surfaced fallback only (requires human decision, not silent adoption). CASE 1 (TUI restart survives) and CASE 3 (hard crash → lost, re-launch) unchanged. New HIGH-priority architect question Q-8 (PTY-ownership-survival mechanism) added; NOTE: the already-built D-235 in-process daemon wiring will likely need rework to move PTY ownership out of the daemon process. Remaining architect-only open questions: Q-1 (PTY bytes over UDS), Q-2 (EngineModule/SessionManager surface), Q-7 (tui-term fork posture), plus PTY-throughput benchmark — all resolved during architecture delta. Architect must also reconcile the stale narrow keyboard scope in DISPOSITION-V2 rollup + embedded-pty-evaluation (superseded by full-fidelity ratification). NEXT: brief delta (product-owner) → architecture delta (architect) → story decomposition (story-writer)."
awaiting: "Adversarial Pass-23 (Pass-22 FINDINGS — 0C/3I/2S; CONSECUTIVE-CLEAN COUNTER = 0 RESET; Pass-23 = clean candidate 1 of 3). Pass-1 DONE (D-242). Pass-2 DONE (D-243). Pass-3 DONE (D-244). Pass-4 DONE (D-245). Pass-5 DONE (D-246). Pass-6 DONE (D-247). Pass-7 DONE (D-248, all 0C/4I/4S — FIRST zero-Critical). Pass-8 DONE (D-249, all 0C/2I/1S). Pass-9 DONE (D-250, all 0C/1I/1S). Pass-10 DONE (D-252, all 0C/1I/4S — FOURTH consecutive zero-Critical). Pass-11 DONE (D-253, all 0C/1I/4S — FIFTH consecutive zero-Critical). Pass-12 DONE (D-254, all 0C/2I/2S — SIXTH consecutive zero-Critical). Pass-13 DONE (D-255, all 0C/2I/2S — SEVENTH consecutive zero-Critical). Pass-14 DONE (D-256, all 0C/1I/1S — EIGHTH consecutive zero-Critical). Pass-15 DONE (D-257, all 0C/2I/1S — NINTH consecutive zero-Critical). Pass-16 DONE (D-258, all 0C/1I/1S — TENTH consecutive zero-Critical). Pass-17 DONE (D-259, all 0C/1I/2S — ELEVENTH consecutive zero-Critical). Pass-18 DONE (D-260, all 0C/1I/2S — TWELFTH consecutive zero-Critical). Pass-19 DONE (D-261, all 0C/1I/1S — THIRTEENTH consecutive zero-Critical). Pass-20 DONE (D-262, all 0C/0I/2S — FOURTEENTH consecutive zero-Critical; FIRST CLEAN PASS). Pass-21 DONE (D-263, all 0C/0I/1S — FIFTEENTH consecutive zero-Critical; SECOND CLEAN PASS). Pass-22 DONE (D-264, all 0C/3I/2S — SIXTEENTH consecutive zero-Critical; FINDINGS — counter RESET 2→0). Novelty trajectory (C:5→5→4→1→2→2→0×16; I:8→6→9→4→4→2→4→2→1→1→1→2→2→1→2→1→1→1→1→0→0→3). CONSECUTIVE-CLEAN COUNTER = 0 (RESET by Pass-22 Importants). Pass-23 dispatches fresh-context adversary on updated spec package (BC-2.09.007 v1.1.0 + BC-2.09.008 v1.1.0 + BC-2.08.005 v1.0.1 + BC-2.05.010 v1.6.0). STRICT-3-CLEAN CRITERION VINDICATED: fresh-context Pass-22 at candidate-3 caught a sibling-BC partial-fix cluster that all 21 prior passes missed — the I7/Killed/O4 fixes propagated to SS-embedded-pty+BC-2.09.001 but never reached sibling BCs BC-2.09.007/008. CODIFIED CYCLE CHECKLIST (D-245 SUG-003): after any RETIRED-term or corrected-magic-number fix, grep -rn .factory/specs before next adversarial pass — every survivor must be in §Trace/changelog/enforcement, NOT live PC/Invariant/title. CODIFIED (D-246): anchor-resolution-closure check. CODIFIED (D-255): L-CWD-PROPAGATION-ATTESTATION. CODIFIED (D-258): L-VERIFICATION-ARTIFACT-FALSE-GREEN. CODIFIED: propagation-closure consistency audit after every split architect+PO fix round (D-244 PROCESS WIN). DEP-PIN-SWEEP-RULE (D-256): extend POL-11 to also grep crate-name+version literals in spec prose. OPEN (non-blocking, human ratification required before v1A story wave): (1) CC-TUITERM-WIP-SIGNOFF; (2) CC-GLOBAL-MOUSE-CAPTURE. DEFERRED (non-blocking): VP authoring for SS-08/SS-09 BCs; v1B Tune BCs; v1B pre-emption BC."
durable_task_register:
  outstanding:
    - id: "DEP-PIN-SWEEP-RULE"
      subject: "[process-gap] extend POL-11 (or sibling check) to grep crate-name+version literals in spec prose against SS-deps-pin-manifest(-v2-delta) — root cause of I14-001"
      status: pending
      detail: "D-256 Pass-14 S14-001: POL-11 keys on artifact-ID version rows (e.g., 'SS-session-manager v1.8.0'), NOT on crate-name+version literals in prose (e.g., 'portable-pty 0.8.x'). I14-001 was a stale portable-pty 0.8.x literal in SS-session-manager §env-inheritance that escaped all prior POL-11 sweeps. Tooling enhancement: add a sibling check that greps spec prose for crate-pin literals and validates them against SS-deps-pin-manifest(-v2-delta) canonical versions. Route to devops-engineer when a POL-extension sprint is scheduled. Non-blocking; no production impact until v1A implementation begins."
      blocking: false
    - id: "ADR-0010-TRACE-256-MARKER"
      subject: "ADR-0010 §Trace v1.2.0 (and SS-daemon-wiring-v2-delta §Trace v1.2.0) narrate historical 'capacity 256' without an inline '[superseded by 64 in v1.3.0]' marker"
      status: pending
      detail: "D-257 Pass-15 S15-001: ADR-0010 §Trace v1.2.0 and SS-daemon-wiring-v2-delta §Trace v1.2.0 reference the historical 'capacity 256' value in changelog prose without an inline superseded-by marker. Normative bodies correctly carry 64. The missing marker is a changelog-legibility gap only — no normative contradiction. Deferred as housekeeping: bumping heavily-cited ADR-0010 standalone for a §Trace annotation is disproportionate. Fold into the next ADR-0010 edit (e.g., when a substantive normative change is required). Non-blocking Suggestion S15-001."
      blocking: false
    - id: "PRD-COUNT-CROSSCHECK-RULE"
      subject: "[process-gap] add a structural-claim check (POL-12 sibling) asserting 'for each SS-NN, PRD §2.NN BC-table row count == BC-INDEX §Summary active count for SS-NN'"
      status: pending
      detail: "D-258 Pass-16 S16-001: PRD §2.8 omitted BC-2.08.008 (active P0 BC) for 15 adversarial passes because the Pass-15 cross-check verification artifact hand-typed the expected BC set as 'BC-2.08.001..007' (7 rows) instead of deriving from BC-INDEX §Summary (8 active BCs for SS-08). The hand-typed set was off by one and produced a false-green that cleared Pass-15. Root cause: cross-check artifacts that hand-enumerate expected sets are self-falsifying — an off-by-one in the hand-typed enumeration passes as CLEAN and hides a real omission for many passes. TOOLING ENHANCEMENT: add a check asserting that for each SS-NN, the PRD §2.NN BC-table row count equals the BC-INDEX §Summary active count for that subsystem — making a P0 BC missing from the PRD index a hard CI failure rather than a silent pass. Route to devops-engineer when a POL-extension sprint is scheduled. PROCESS RULE: PO must derive expected sets from BC-INDEX counts in future §2 sync sweeps — never hand-enumerate. Non-blocking tooling enhancement."
      blocking: false
    - id: "TD-MULTI-CLIENT-ATTACH-STORM-001"
      subject: "concurrent multi-TUI-client attach-reset storm refinement — ratified-FUTURE scope boundary (BC-2.05.009 Invariant 2)"
      status: ratified-future
      detail: "D-253 Pass-11 I11-001 PRONG B adjudication: concurrent multi-TUI-client attach-storm (per-client unicast scrollback or attach-scoped fan-out instead of broadcast-reset-all) is explicitly ratified as out-of-v1A scope per BC-2.05.009 Invariant 2 ('v1A: single TUI client or first-win semantics for concurrent attach'). The current broadcast fan-out is forward-compatible infrastructure. Scope-boundary note added to SS-daemon-wiring-v2-delta v1.6.0. Specify the unicast/per-client protocol refinement when the multi-TUI-client capability is formally scheduled (v1B or later); current infra does not need rework. NOT a v1A defect."
      blocking: false
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
      subject: "[process-gap] version-pin-registry atomicity — RECURRENCE at D-242 (BC-2.06.025)"
      status: pending
      detail: "RESOLVED D-231 (first instance). RECURRENCE D-242-fix: BC-2.06.025 was bumped to v1.1.0 in the BC file as part of the D-242 O5 fix (spawned_by_monocle None→[?] + EC-295) but its registry entry remained at 1.0.0. Root cause: state-manager burst verification did not line-by-line diff the claimed 15-entry bump list against actual registry edits before pushing. ADDITIONAL PROCESS-GAP: state-manager checklist must include an explicit step — count claimed registry bumps vs. actual lines changed in version-pin-registry.yaml before commit. Routing: devops-engineer delivery-skill + state-manager checklist update."
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
    - id: "CC-TUITERM-WIP-SIGNOFF"
      subject: "Human risk-acceptance of tui-term 0.3.4 WIP-upstream for v1A embedded-terminal widget — required before v1A story wave begins"
      status: pending
      detail: "ADR-0011 §O2: tui-term 0.3.4 is noted as WIP-upstream. Human must explicitly accept the risk before the v1A story wave begins. If tui-term 0.3.4 is promoted to stable before the story wave, this item closes automatically. Non-blocking until story decomposition dispatched."
      blocking: false
    - id: "CC-GLOBAL-MOUSE-CAPTURE"
      subject: "Human approves global mouse capture if a future story needs clickable monocle panels — v1A scopes mouse capture to EmbeddedTerminal entry/exit only"
      status: pending
      detail: "SS-embedded-pty I3 (D-242): mouse capture is scoped to EmbeddedTerminal entry (enable) and exit (disable). If a future v1B story requires clickable monocle panel UI (e.g., sessions list), human must explicitly approve global mouse capture before that BC is authored. Non-blocking for v1A."
      blocking: false
    - id: "SS-DAEMON-WIRING-SCROLLBACKDUMP-TERM"
      subject: "SS-daemon-wiring-v2-delta.md lines ~195/~822 use bare retired noun 'ScrollbackDump' as parenthetical generic dump label"
      status: pending
      detail: "D-261 Pass-19 S19-001: SS-daemon-wiring-v2-delta.md at lines ~195 and ~822 use the bare retired noun 'ScrollbackDump' as a parenthetical generic label ('ScrollbackDump (ScrollbackChunk* + ScrollbackDumpComplete sequence)'), providing disambiguation but still leading with the retired term. Not a live normative survivor — the parenthetical makes the canonical types clear. Replace with 'ScrollbackChunk* + ScrollbackDumpComplete sequence' to eliminate the retired noun entirely. Deferred: bumping the heavily-cited SS-daemon-wiring-v2-delta standalone for this terminology polish is disproportionate. Fold into the next substantive SS-daemon-wiring-v2-delta edit. Non-blocking Suggestion S19-001."
      blocking: false
    - id: "ADR-0011-UPGRADE-TYPO"
      subject: "ADR-0011 §Pin-policy line ~150 doubled word 'upgrade upgrade' — fold into next substantive ADR-0011 edit"
      status: pending
      detail: "D-262 Pass-20 S20-002: ADR-0011 §Pin-policy prose contains a doubled word 'upgrade upgrade' at approximately line 150. Non-blocking Suggestion only (typo cannot escalate to a correctness defect). Deferred as housekeeping: creating a standalone ADR-0011 bump + citation-sweep solely for a typo is disproportionate. Fold into the next substantive normative ADR-0011 edit."
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
  ZERO-CONTEXT RESUME CHECKPOINT v7.15 (D-264) — 2026-06-04
  PIVOT: monocle → full TUI control center — PHASE-1D ADV CONVERGENCE IN PROGRESS
  22 PASSES COMPLETE; CONSECUTIVE-CLEAN COUNTER = 0 (RESET); PASS-23 NEXT
  ============================================================================

  READ THESE FIRST (in order — before anything else):
  1. /Users/jmagady/Dev/monocle/NEXT-SESSION-RESUME.md  ← concise 1-page entry point
  2. /Users/jmagady/Dev/monocle/CLAUDE.md               ← production-grade + agent-routing rules
  3. This STATE.md fully                                 ← durable_task_register + PIVOT-CONTROL-CENTER

  ============================================================================
  WHERE WE ARE (D-264, 2026-06-04)
  ============================================================================

  MODE: greenfield-with-reference-ingest.
  PHASE: VSDD Phase 1d ADVERSARIAL SPEC CONVERGENCE — IN PROGRESS.
  develop @ 8bc22a5 — UNCHANGED. All pivot work is SPEC-ONLY on factory-artifacts.
  NO production code written yet for v1A. Do NOT write code for v1A.
  factory-artifacts: run `git -C .factory log -1 --format='%h %s'` for live HEAD.

  22 adversarial passes complete on the v1A control-center spec package.
  Finding trajectory: Critical 5,5,4,1,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0 / Important 8,6,9,4,4,2,4,2,1,1,1,2,2,1,2,1,1,1,1,0,0,3
  Passes 7 through 22 ALL ZERO-Critical (SIXTEEN consecutive). Pass 20 = 0C/0I (FIRST CLEAN). Pass 21 = 0C/0I (SECOND CLEAN).
  Pass 22 = 0C/3I/2S (FINDINGS — sibling-BC partial-fix cluster caught at candidate-3).
  CONSECUTIVE-CLEAN COUNTER = 0 (RESET by Pass-22 3 Importants — streak RESTARTED).
  NEXT = adversarial Pass 23 (clean-streak candidate 1 of 3 — streak restarted).
  SPEC PACKAGE VERSIONS UPDATED (D-264): BC-2.09.007 v1.1.0 + BC-2.09.008 v1.1.0 + BC-2.08.005 v1.0.1 + BC-2.05.010 v1.6.0.
  BC-INDEX v1.39.1 (D-263); BC-2.05.009 v1.5.1 (D-262); vision v2.2.2 + brief v2.0.3 + ADR-0011 v1.2.1 unchanged.
  HUMAN DIRECTIVE: strict 3-consecutive-clean (chosen at Pass-6 checkpoint over
  the "1 more pass then decide" option — do NOT accept less than 3).
  STRICT-3-CLEAN VINDICATED: Pass-22 at candidate-3 caught a sibling-BC cluster all 21 prior passes missed.

  ============================================================================
  PIVOT HISTORY (context — do not re-litigate)
  ============================================================================

  D-236: VISION PIVOT — observe-only RETIRED; monocle becomes full TUI control center.
  D-237: Human ratified v1 scope (Launch + EmbeddedPTY + Multi-session/project + Tune; v1A first).
  D-238: Vision domain-monocle-vision-synthesis.md v2.2 APPROVED (Joshua Magady).
         session-host-owns-PTY persistence model. graceful daemon-process restart MUST survive.
  D-239: Architecture delta COMMITTED — ADR-0009/0010/0011 + 5 SS deltas.
  D-240: Consistency gate PASSED — ARCH-INDEX v1.0.28; vision v2.2 + brief v2.0.1 VALID.
  D-241: BC/PRD delta — 25 new v1A BCs; PRD v1.28.2; 5 holdout scenarios HS-EXP-011..015.
  D-242 through D-252: Adversarial Passes 1-10, all findings resolved. D-253: Pass-11 convergence-fix — I11-001 PRONG A auto-attach mandate (SS-embedded-pty 1.4.0 + BC-2.09.001 1.3.0) + PRONG B scope-boundary note (SS-daemon-wiring-v2-delta 1.6.0 + TD-MULTI-CLIENT-ATTACH-STORM-001); S11-001 phantom §Trace fix (SS-session-manager 1.7.2); SS-ipc 1.16.0 doc-comment extension; 35-file propagation sweep; POL-11/POL-12 PASS. D-254: Pass-12 convergence-fix — I12-001 EngineError→ServerToClient::Error taxonomy gap (taxonomy 8→10 codes; SS-ipc 1.17.0, SS-session-manager 1.8.0, SS-daemon-wiring-v2-delta 1.7.0, BC-2.03.007 1.1.0); I12-002 BC-2.03.005 worktree_root regression (1.1.1); S12-001 SS-embedded-pty App struct fields (1.5.0); S12-002 §Scope cross-ref. 35-file propagation sweep; POL-11/POL-12 PASS. D-255: Pass-13 convergence-fix — I13-001+S13-001 BC-2.03.005 exhaustive worktree_root (VP row-1+EC-102+§Trace false-attestation; 3rd recurrence, whole-file grep; 1.1.1→1.1.2); I13-002 ADR-0009 SS-08 mis-anchor + ADR-0011 SS-08/SS-09 sibling sweep (1.0.0→1.0.1, 1.1.0→1.2.0); S13-002 no-change; L-CWD-PROPAGATION-ATTESTATION codified; POL-11/POL-12 PASS. D-256: Pass-14 convergence-fix — I14-001 portable-pty 0.8.x→0.9.x lone stale normative literal in SS-session-manager §env-inheritance; exhaustive crate-pin sweep confirmed class CLOSED (SS-session-manager 1.8.0→1.8.1); 15-file propagation sweep (27 stale actives). S14-001 DEP-PIN-SWEEP-RULE process-gap. POL-11/POL-12 PASS. EIGHTH consecutive zero-Critical. D-257: Pass-15 convergence-fix — I15-001 EC-283 code invalid_request→rename_failed per canonical session_error_to_code (BC-2.05.010 1.5.0→1.5.1); I15-002 PRD §2.5 6-variant→7-variant +AttachSession + missing BC-2.05.011 row (prd 1.28.1→1.28.2). S15-001 deferred (ADR-0010-TRACE-256-MARKER). POL-11/POL-12 PASS. NINTH consecutive zero-Critical. D-258: Pass-16 convergence-fix — I16-001 PRD §2.8 omitted active P0 BC-2.08.008 (undetected 15 passes due to hand-typed false-green cross-check; BC-2.08.008 row added; SS-08=7→8 attestation corrected; derived count cross-check added; prd 1.28.2→1.28.3). S16-001 PRD-COUNT-CROSSCHECK-RULE process-gap. L-VERIFICATION-ARTIFACT-FALSE-GREEN codified. POL-11/POL-12 PASS. TENTH consecutive zero-Critical. D-259: Pass-17 convergence-fix — I17-001 ADR pin-symmetry exhaustive sweep (25 BC Architecture-Source cells; ADR-0009 v1.0.2/ADR-0011 v1.2.0 pinned + 2 unpinned SS-session-manager v1.8.1 refs fixed; BC-2.09.001 1.3.1 + BC-2.08.001 1.3.1 + BC-2.08.002 1.2.1); S17-001 ADR-0009 no-retry wording (1.0.1→1.0.2); S17-002 §-anchors folded in. POL-11/POL-12 PASS. ELEVENTH consecutive zero-Critical.
  D-260: Pass-18 convergence-fix — I18-001 exhaustive daemon-owned-PTY survivor sweep (vision+brief; 4 live survivors at vision lines 122/588/592 + brief line 259 → session-host-owned framing; vision 2.2→2.2.1, brief 2.0.1→2.0.2); S18-001 embedded-pty-evaluation.md superseded banner (research doc — not registry-tracked; analysis preserved); S18-002 LEFT (BC-2.09.009 §Trace v1.0.0 once-only bell — exempt historical §Trace, correctly superseded by v1.1.0). 1 registry entry bumped: product-brief→2.0.2. POL-11/POL-12 PASS. TWELFTH consecutive zero-Critical.
  D-261: Pass-19 convergence-fix — I19-001 tui-term pin FORM drift in ADR-0011 §Decision + brief/vision Tech-Stack tables (caret '0.3' → exact '=0.3.4'; ADR-0011 1.2.0→1.2.1; brief 2.0.2→2.0.3; vision 2.2.1→2.2.2); BC-2.09.001 ADR-0011 citation swept v1.2.0→v1.2.1 (mechanical); pin-form class CLOSED (exhaustive in-scope arch doc sweep). S19-001 (SS-daemon-wiring-v2-delta bare ScrollbackDump term) DEFERRED (housekeeping; SS-DAEMON-WIRING-SCROLLBACKDUMP-TERM task registered). 2 registry entries bumped: ADR-0011→1.2.1, product-brief→2.0.3. POL-11/POL-12 PASS. THIRTEENTH consecutive zero-Critical.

  ============================================================================
  CONVERGENCE LOOP PROCEDURE (how the fresh session runs Pass 12+)
  ============================================================================

  Step A — Dispatch adversary (vsdd-factory:adversary) FRESH-CONTEXT for Pass N.
    Scope = the v1A control-center spec package (list below).
    Rubric = CLAUDE.md production-grade principle.
    Adversary must verify:
      (a) anchor-resolution closure — every BC Architecture-Source citation resolves
          to a symbol that EXISTS in the cited doc (C5-001 class prevention)
      (b) full-lifecycle end-to-end consistency: SS ↔ BC ↔ ADR ↔ holdout
      (c) no live normative survivor of retired concepts
      (d) error-taxonomy completeness + no-silent-failure guarantee
      (e) security (keystroke injection, auth token, SO_PEERCRED paths)
      (f) any new design gap
    Instruct: honest CLEAN verdict if sound; only Critical/Important block; Suggestions do not.

  Step B — If CLEAN:
    Increment consecutive-clean counter.
    If counter < 3: run next pass (counter now N; need 3 total).
    If counter = 3: convergence DONE. Proceed to human spec-package approval gate.

  Step C — If FINDINGS (any Critical or Important):
    Reset consecutive-clean counter to 0.
    Route fixes by ownership:
      architect (vsdd-factory:architect) — architecture specs, ADRs, wire-types, error-taxonomy
      product-owner (vsdd-factory:product-owner) — BC files, holdout scenarios, PRD, vision, brief
      Architect + PO may run in parallel when their fix sets are independent.
    After fixes: orchestrator runs CLOSURE GREP (retired-term + value survivors + anchor-resolution).
    Then state-manager commits the round atomically (see commit rules below).
    Then dispatch Pass N+1.

  COMMIT RULES FOR EVERY FIX ROUND (mandatory — enforced by hooks):
    - state-manager owns version-pin-registry.yaml; architect/PO only REPORT version bumps
    - single atomic commit: spec files + version-pin-registry.yaml in SAME commit (POL-11/L-S027-004)
    - TARGETED staging only: name each file explicitly; NEVER git add -A; never stage
      code-delivery/ dirs; never broad compute-input-hash --scan
    - run `python3 scripts/check_version_pins.py` (POL-11) and
      `python3 scripts/check_structural_claims.py` (POL-12) from REPO ROOT before commit
    - verify registry diff line-count matches claimed bump count + spot-check frontmatter
    - respect all hooks; NEVER --no-verify; no AI attribution

  CYCLE CHECKLIST (D-245 SUG-003 CODIFIED):
    After any RETIRED-term or corrected-magic-number fix:
    grep -rn .factory/specs for survivors — every survivor must be in
    §Trace/changelog/enforcement context, NOT live PC/Invariant/title/doc-comment.

  ANCHOR-RESOLUTION CLOSURE (D-246 CODIFIED):
    After any BC Architecture-Source citation fix: verify the cited symbol EXISTS
    in the cited document before committing.

  ============================================================================
  SPEC PACKAGE (feed this exact list to the adversary for Pass 10+)
  ============================================================================

  domain-monocle-vision-synthesis.md v2.2.2 (APPROVED D-238 + I18-001 consistency correction D-260 + I19-001 tui-term pin-form D-261)
  product-brief.md v2.0.3 (validate-brief VALID)
  prd.md v1.28.3 (§2.8 SessionManager + §2.9 EmbeddedPTY; §2.5 BC-2.05.010 7-variant + BC-2.05.011 row; §2.8 BC-2.08.008 row + derived count cross-check)
  ARCH-INDEX v1.0.28

  Architecture subsystem specs (delta docs):
    SS-ipc v1.17.0                       (IPC wire authority — all 21 message variants + 10-code taxonomy)
    SS-session-manager v1.8.1            (SS-08 — session-host coordinator + EngineError bridge + portable-pty 0.9.x normative)
    SS-embedded-pty v1.5.0               (SS-09 — embedded PTY widget + App struct fields)
    SS-engine-module-v2-delta v1.1.0     (spawn_recipe() + SpawnOptions)
    SS-daemon-wiring-v2-delta v1.7.0     (daemon coordinator + IPC handler + spawn_recipe call-site)
    SS-deps-pin-manifest-v2-delta v1.0.0 (portable-pty/vt100/tui-term pins)

  ADRs:
    ADR-0009 v1.0.2 (native detached session-host process model)
    ADR-0010 v1.5.0 (PTY bytes on existing UDS IPC; snapshot-then-resume scrollback)
    ADR-0011 v1.2.1 (PTY stack: portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4)

  Behavioral Contracts:
    BC-INDEX v1.39.1 (138 BCs total; 25 new v1A BCs)
    SS-03: BC-2.03.005 v1.1.2, BC-2.03.006-008 (spawn validation, hook auto-injection, invalid-path/worktree_root, re-probe); BC-2.03.007 v1.1.0
    SS-05: BC-2.05.009 v1.5.1 (D-262 S20-001 title precision); BC-2.05.010-011 (multi-project scope, ServerToClient variants)
    SS-06: BC-2.06.025 (permission badge+bell guarantee during EmbeddedTerminal/SessionCreation)
    SS-08: BC-2.08.001-008 (session spawn/kill/SIGTERM-SIGKILL-10s/re-discovery/persistence/
                             per-session-hooks/GC/SessionStateChanged)
    SS-09: BC-2.09.001-009 (PTY attach/detach/keyboard/output/resize/reconnect/AppMode/
                             scrollback/overlay-integration)

  Holdout scenarios:
    EVAL-INDEX v1.15
    HS-EXP-011..015 (5 new v1A scenarios)

  version-pin-registry.yaml — source of truth for ALL version pins

  ============================================================================
  RATIFIED DECISIONS (do NOT re-litigate)
  ============================================================================

  Re-baselined v1 control-center: Launch + Embedded PTY + Multi-session/multi-project +
  Interactive Tune (v1B) + already-built Observe + Control.
  v1A = Launch + EmbeddedPTY + Multi-session/project (+ persistence + hook-auto-inject).
  v1B = Interactive Tune (BCs/stories authored when v1B scheduled).

  Persistence: session-host-owns-PTY (native monocle-session-host binary, setsid-detached,
  per-session UDS, SO_PEERCRED, re-discovery before UDS bind).
  Graceful daemon-process restart SURVIVES (D-238 human escalation — CASE 2 is now survive).
  Hard crash → lost → re-launch (CASE 3 unchanged). NO tmux.

  Full keyboard fidelity v1A: printable + control + arrows + Alt/Meta + mouse + Kitty +
  bracketed-paste. Permission badge + per-prompt bell during EmbeddedTerminal.
  PTY backpressure: per-client buffers cap 64 (no silent drop); PtyReset recovery.
  Terminating state + 12s watchdog. SessionSnapshot/SerializedCell/SerializedColor in monocle-ipc.
  ClientToServer::AttachSession re-attach. schema_version 3.
  Chunked scrollback: snapshot-then-resume (session-host NOT paused during dump).
  Native PTY stack: portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4
  (ratatui 0.30 compatible; MSRV 1.88). Decision lineage D-236..D-250.

  ============================================================================
  REMAINING TASKS (in order)
  ============================================================================

  1. FINISH Phase-1d adversarial convergence (current step):
     Passes 18+ → need 3 CONSECUTIVE clean (counter currently 0; Pass 17 was non-clean).
     Drive to strict 3-clean per human directive. See convergence loop above.

  2. Human spec-package APPROVAL GATE (after convergence):
     Run /vsdd-factory:check-input-drift first.
     Present with structured review questions: scope completeness, anchor correctness,
     coverage gaps, convention consistency.
     Gate items (non-blocking until story wave): CC-TUITERM-WIP-SIGNOFF (tui-term 0.3.4
     WIP risk-acceptance, ADR-0011 §O2) + CC-GLOBAL-MOUSE-CAPTURE (mouse scope expansion).

  3. Phase 2 STORY DECOMPOSITION (vsdd-factory:story-writer):
     Decompose v1A delta into stories + waves + dependency graph.
     RESOLVE all S-TBD story anchors in the 25 BCs + holdout stories_tested fields.
     If >8 artifacts: split create-stories / integrate-stories across two dispatches.

  4. VP AUTHORING (vsdd-factory:architect) — DEFERRED to formal-hardening per VP-DTU-001
     pattern: all 25 v1A BCs cite VP-TBD; create VPs + VP-INDEX coverage at hardening.

  5. Pre-Phase-3 prerequisites:
     DTU clone check (S-DTU-001 dtu-claude-code-hooks-v1 EXISTS, fidelity 1.0 — D-234).
     CI/CD verification (ci.yml + branch protection — PROC-BRANCH-PROTECTION-CONTEXTS open).

  6. Phase 3 TDD IMPLEMENTATION of v1A stories (per-story-delivery.md + wave gates).
     v1B Tune BCs/stories authored when v1B scheduled.

  7. PARKED HUMAN ITEMS (must happen before v1A story wave begins):
     CC-TUITERM-WIP-SIGNOFF — tui-term 0.3.4 WIP risk acceptance (ADR-0011 §O2).
     CC-GLOBAL-MOUSE-CAPTURE — if a future story needs clickable monocle panels.
     v1B: embedded-terminal→overlay PRE-EMPTION needs human ratification before BC authoring.

  8. Pre-existing durable_task_register backlog (ADV-W5GATE-HIGH-002 dead code,
     ADV-W5GATE-MED-001/003, etc.) — non-blocking, subordinate to pivot; revisit during
     v1A implementation waves.

  ============================================================================
  CODIFIED LESSONS (enforce every pass)
  ============================================================================

  L-S027-004 REGISTRY ATOMICITY (recurred 3x): architect/PO REPORT version bumps,
  NEVER edit version-pin-registry.yaml; state-manager owns it + verifies diff line-count
  + frontmatter spot-check before every commit.

  PROPAGATION-CLOSURE AUDIT: after every split architect+PO fix round, run a closure
  check (consistency-validator or targeted grep) confirming every delegated sync landed
  in EVERY target file, BEFORE the next adversarial pass.

  ANCHOR-RESOLUTION CLOSURE: every BC Architecture-Source citation must resolve to a
  symbol that EXISTS in the cited doc (prevents C5-001 class).

  RETIRED-CONCEPT GREP SWEEP: after any RETIRED-term / corrected-value fix, grep
  .factory/specs for survivors; each must be §Trace/enforcement context, not a live
  PC/Invariant/title/doc-comment.

  TARGETED STAGING ONLY: D-249 commit accidentally broad-swept 144 files + code-delivery
  dirs. Always name files explicitly. Never git add -A for factory-artifacts commits.

  DELTA-VS-CANONICAL DRIFT: SS-*-v2-delta inline copies of patterns canonical elsewhere
  drift (recurred — SessionSnapshot in §4, §3 handler arms). Prefer reference-pointers
  + canonical-pattern-lock notes over inline copies.

  L-VERIFICATION-ARTIFACT-FALSE-GREEN (D-258): verification/cross-check artifacts that
  HAND-ENUMERATE expected sets (e.g. 'BC-2.08.001..007') create self-falsifying false-greens.
  LESSON: cross-check artifacts MUST derive expected counts/sets from BC-INDEX §Summary
  (never hand-type them); a verification record asserting CLEAN must itself be checked
  against the index. Sibling of L-CWD-PROPAGATION-ATTESTATION (false attestation class).

  ============================================================================
  ALREADY BUILT (REUSE — DO NOT REBUILD)
  ============================================================================

  - Daemon (actually serves: HTTP hook ingestion + UDS socket + tracing + ring — D-235).
  - Hook ingestion (5 endpoints, proto, JSONL ring, dtu-claude-code-hooks-v1 clone).
  - Permission overlay (VecDeque<PromptModal>, Ctrl-\ popup, y/n/A resolve, IPC write-back).
  - EngineModule / FactoryAdapter traits — seam to extend with launch/spawn/attach/kill.
  - monocle-proto wire schemas, monocle-ipc, monocle-config, monocle-core.
  - TUI (ratatui + crossterm): sessions panel, event ribbon, profile picker, status bar.
  - 1514 passing tests. 9 workspace crates. develop @ 8bc22a5.

  ============================================================================
  FACTORY INFRASTRUCTURE
  ============================================================================

  .factory/ mounted at factory-artifacts orphan branch.
  Verify worktree health: `git -C .factory rev-parse --git-dir` must succeed.
  Run factory-worktree-health via devops-engineer FIRST on session start.
  Commit hooks: block-ai-attribution, validate-input-hash, validate-table-cell-count.
  NEVER --no-verify. NEVER add Co-Authored-By: Claude or robot emoji.

  WAVE-8 BACKLOG (valid, subordinate to pivot — do not block on these):
    S-032 (5 pts, Wave 8, EPIC-05) — daemon fan-out (BC-2.05.004 v1.1.0 PC-2 obligation)
    S-DAEMON-WIRE-FIX-001 (Wave 8, P1, 5pts) — second-signal exit codes 143/130

  KNOWN-FLAKY (DO NOT FLAG as new findings):
    cli_daemon_stop, factory_self_referential, test_BC_2_07_006,
    wit-bindgen unmatched-skip, PATH isolation flake.
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
| PIVOT | ADV-PASS-22-FINDINGS (D-264); Pass-23 NEXT | 2026-06-04 | D-242..D-264: Passes 1-22 ALL resolved. C trajectory: 5→5→4→1→2→2→0×16 (SIXTEEN consecutive zero-Critical). I trajectory: 8→6→9→4→4→2→4→2→1→1→1→2→2→1→2→1→1→1→1→0→0→3. Pass-22 (D-264): FINDINGS (0C/3I/2S) — sibling-BC cluster (pty_scroll_offset / Killed / ~4-bytes / spawn-codes) caught at candidate-3. CONSECUTIVE-CLEAN COUNTER = 0 (RESET). BC-2.09.007→1.1.0, BC-2.09.008→1.1.0, BC-2.08.005→1.0.1, BC-2.05.010→1.6.0. NEXT: Pass-23 (clean candidate 1 of 3 — streak RESTARTED). |

develop @ fcd42f04 (NEXT-SESSION-PIVOT.md + CLAUDE.md D-236 banner). Phase 3 COMPLETE (D-232). 32/33 stories done, 192/195 pts (98%). D-236: PRODUCT-VISION PIVOT — monocle becomes full TUI control center. D-238: Vision v2.1 APPROVED. D-239: Architecture delta COMMITTED (ADR-0009/0010/0011 + 5 SS deltas; ARCH-INDEX v1.0.27). D-240: Consistency gate PASSED — vision v2.2 + brief v2.0.1 + arch fixes (ARCH-INDEX v1.0.28). D-261: Pass-19 done — ADR-0011 v1.2.1 + brief v2.0.3 + vision v2.2.2 (I19-001 tui-term exact-pin form; pin-form class CLOSED). VSDD Phases 4-7 (old scope) SUSPENDED. NEXT: adversarial Pass-20 (3 consecutive clean needed) → human gate → story decomposition.

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
| D-239 | Control-center architecture delta COMMITTED. 3 ADRs: ADR-0009 (native detached session-host process model for PTY ownership — Q-8 resolved native; monocle-session-host binary owns PTY masters + harness child, daemon re-attaches over UDS on restart), ADR-0010 (PTY bytes shared on existing UDS IPC channel — Q-1 resolved Option A), ADR-0011 (PTY stack: native portable-pty 0.9 + vt100 0.16 + tui-term 0.3.4 — Q-7 fork posture: exact-pin, no fork). 5 SS deltas: SS-session-manager 1.0.0 (SS-08, CAP-008 — SessionManager coordinator, monocle-session-host binary, session-state.json, re-discovery, GC), SS-embedded-pty 1.0.0 (SS-09, CAP-009 — EmbeddedTerminal AppMode, PTY widget, keyboard encoding, SessionCreation wizard), SS-engine-module-v2-delta 1.0.0 (Q-2 resolved — spawn_recipe() method, SpawnRecipe/SpawnOptions), SS-daemon-wiring-v2-delta 1.0.0 (D-235 rework scope — DaemonState.session_manager, daemon_start_sequence step 8b, PtyOutput fan-out), SS-deps-pin-manifest-v2-delta 1.0.0 (portable-pty/vt100/tui-term pins, monocle-session-host crate). ARCH-INDEX bumped to v1.0.27 (SS-08/SS-09, CAP-008/009, ADR-0009/0010/0011 rows). 8 version-pin-registry entries added atomically. input-hashes computed. POL-11 PASS, POL-12 PASS. NEXT: adversarial review (3 clean passes) → human gate → story decomposition. STATE v6.88→v6.89. | 2026-06-03 | state-manager |
| D-241 | v1A control-center BC/PRD/holdout delta — 23 new BCs authored: SS-03 BC-2.03.005-008 (spawn validation, hook auto-injection, invalid-path error, re-probe), SS-05 BC-2.05.009-010 (SessionCreation wizard, multi-project scope), SS-06 BC-2.06.025 (permission badge+bell guarantee during EmbeddedTerminal/SessionCreation), SS-08 NEW BC-2.08.001-007 (session spawn/kill/SIGTERM-SIGKILL-10s-escalation/parallel-re-discovery/state-persistence/per-session-hooks-file/GC), SS-09 NEW BC-2.09.001-009 (PTY attach/detach/keyboard/output/resize/reconnect/AppMode/scrollback/overlay-integration). BC-INDEX v1.34→v1.35 (113→136 BCs). PRD v1.27.4→v1.28.0 (§2.8 SessionManager + §2.9 EmbeddedPTY). 5 holdout scenarios HS-EXP-011..015 (EVAL-INDEX v1.9→v1.10, 24→29 scenarios). Registry atomicity (L-S027-004): 26 new/updated entries in same atomic commit. POL-11 PASS (271 active), POL-12 PASS. In-scope production-grade design decisions: SIGTERM→SIGKILL 10s escalation (BC-2.08.003); parallel re-discovery probing (BC-2.08.004); per-session hooks file to prevent concurrent-spawn clobber (BC-2.08.006). DEFERRED (architect task, VP-DTU-001 pattern): VP authoring for SS-08/SS-09 BCs (all 16 cite VP-TBD) — formal-hardening scheduling. v1B Tune BCs + embedded-terminal pre-emption BC remain out of scope (v1B scheduling; pre-emption needs human ratification). NEXT = Phase-1d adversarial convergence (3 clean passes) on full control-center spec package, then human approval gate, then v1A story decomposition. STATE v6.90→v6.91. | 2026-06-03 | state-manager |
| D-256 | Adversarial Pass-14 convergence-fix — 0 Critical / 1 Important / 1 Suggestion. I14-001 (lone stale portable-pty 0.8.x normative literal in SS-session-manager §env-inheritance prose at line 597; architect ran exhaustive crate-pin sweep across all in-scope specs confirming I14-001 was the ONLY stale live straggler — class CLOSED; SS-session-manager 1.8.0→1.8.1). S14-001 [process-gap] DEP-PIN-SWEEP-RULE: POL-11 keys on artifact-IDs not crate-pin literals → stale crate pins in prose escape CI; recorded in durable_task_register (devops-engineer tooling task, non-blocking). 1 registry entry bumped atomically (L-S027-004): SS-session-manager→1.8.1. Propagation sweep: 15 unique files updated (27 stale actives); no §Trace/historical entries touched. POL-11 PASS (319 active). POL-12 PASS. EIGHTH consecutive zero-Critical (C:...,0×8). Consecutive-clean counter = 0; Pass-15 next. STATE v7.06→v7.07. | 2026-06-04 | state-manager |
| D-257 | Adversarial Pass-15 convergence-fix — 0 Critical / 2 Important / 1 Suggestion. I15-001 (EC-283 code 'invalid_request' for RenameSession empty-name contradicts canonical session_error_to_code() which maps InvalidSessionName→'rename_failed' unconditionally — cross-doc contradiction/build collision; BC-2.05.010 1.5.0→1.5.1, EC-283 corrected to 'rename_failed', §Trace rationale updated). I15-002 (PRD §2.5 listed BC-2.05.010 as '6-variant' title missing AttachSession — S-P7-003 partial-fix straggler; class-close sweep also found entirely missing BC-2.05.011 row in §2.5; prd 1.28.1→1.28.2, 7-variant + BC-2.05.011 row added). S15-001 ADR-0010 §Trace v1.2.0 '256' lacks inline superseded-marker — DEFERRED (housekeeping; fold into next ADR-0010 substantive edit; ADR-0010-TRACE-256-MARKER recorded in durable_task_register). 2 registry entries bumped atomically (L-S027-004): BC-2.05.010→1.5.1, prd→1.28.2. POL-11 PASS (319 active). POL-12 PASS. NINTH consecutive zero-Critical (C:...,0×9). Consecutive-clean counter = 0; Pass-16 next. STATE v7.07→v7.08. | 2026-06-04 | state-manager |
| D-259 | Adversarial Pass-17 convergence-fix — 0 Critical / 1 Important / 2 Suggestions. I17-001 (Architecture-Source pin-symmetry violation: 3 BC cells cited ADR docs unpinned while SS docs pinned — violates codified BC-INDEX Pin-Symmetry Convention F-R117-3/SE-17e; sibling-inconsistent since SS-05 BCs pinned ADR-0010) FIXED via EXHAUSTIVE sweep of all 25 in-scope BC Architecture-Source cells: pinned ADR-0011 v1.2.0 (BC-2.09.001) + ADR-0009 v1.0.2 (BC-2.08.001/002) + 2 additional unpinned SS-session-manager v1.8.1 refs found in same cells; zero remaining violations confirmed. S17-002 (loose ADR §-anchors) folded in: §PTY-stack-selection/§native-detached-session-host → §Decision (verified exact headings). S17-001 (ADR-0009 risk-table '5s backoff' ambiguous vs canonical no-retry) FIXED → '5s hard deadline, one attempt, no retry per BC-2.08.004 Inv-2' (ADR-0009 1.0.1→1.0.2). BC bumps: BC-2.09.001 1.3.1, BC-2.08.001 1.3.1, BC-2.08.002 1.2.1. 4 registry entries bumped atomically (L-S027-004). POL-11 PASS (329 active). POL-12 PASS. ELEVENTH consecutive zero-Critical (C:...,0×11). Consecutive-clean counter = 0; Pass-18 next. STATE v7.09→v7.10. | 2026-06-04 | state-manager |
| D-264 | Adversarial Pass-22 FINDINGS — CONSECUTIVE-CLEAN COUNTER RESET 2→0. 0 Critical / 3 Important / 2 Suggestion. The strict-3-clean criterion paid off: fresh-context Pass-22 uncovered a partial-fix sibling CLUSTER in SS-09 BCs that all 21 prior passes (incl. the 2 clean ones) MISSED — the I7 (pty_scroll_offset→per-session pty_scroll_offsets HashMap), Killed-state-removal, and O4 (~4→~16 bytes/cell) fixes propagated to SS-embedded-pty + BC-2.09.001 but never reached sibling BCs. I22-001 (BC-2.09.007 retained retired singular pty_scroll_offset throughout — would reintroduce 'focus switch shows wrong session scrollback' bug) FIXED → per-session HashMap (1.0.0→1.1.0). I22-002 (BC-2.09.008 Inv-1 retained retired Killed + missing Terminating + Detached-no-op contradicting own PC-2) FIXED → reconciled per-state no-op set, Detached attachable, PC-4 HashMap (1.0.0→1.1.0). I22-003 (BC-2.09.007 Inv-2 stale ~4 bytes/cell memory bound) FIXED → ~16 bytes/cell, 12.8MB/session, ~102MB/8 per O4 (in BC-2.09.007 1.1.0). S22-001 (BC-2.05.010 PC-4 incomplete spawn-code set) FIXED → 6-code set, EC-280→binary_not_found (1.5.1→1.6.0). S22-002 (BC-2.08.005 Killed prose) FIXED (1.0.0→1.0.1). Exhaustive 4-class sweep (pty_scroll_offset / Killed-state / ~4-bytes / spawn-code) across all in-scope BCs confirmed zero remaining live survivors; closure grep verified all residuals are §Trace/canonical-negation. 4 registry entries bumped atomically (L-S027-004): BC-2.09.007→1.1.0, BC-2.09.008→1.1.0, BC-2.08.005→1.0.1, BC-2.05.010→1.6.0. POL-11 PASS (328 active). POL-12 PASS. SIXTEENTH consecutive zero-Critical (C:...,0×16). Consecutive-clean counter = 0; Pass-23 next = clean candidate 1 of 3 (streak RESTARTED). STATE v7.14→v7.15. | 2026-06-04 | state-manager |
| D-263 | Adversarial Pass-21 CLEAN — SECOND CONSECUTIVE CLEAN PASS. 0 Critical / 0 Important / 1 Suggestion. Independent fresh-context multi-axis adversary sweep (anchors, IPC taxonomy, SO_PEERCRED security incl. daemon-UDS-vs-per-session-UDS asymmetry re-derived as sound, PRD↔BC-INDEX counts, numeric constants, holdout integrity, BC-internal coherence) found NO Critical/Important defect. FIFTEENTH consecutive zero-Critical. Consecutive-clean counter 1→2 (need 3 consecutive per ratified human directive). S21-001 (BC-INDEX SS-07 header CAP-007 capability-text paraphrase diverged from ARCH-INDEX canonical verbatim — Suggestion only) fixed post-clean as capability-text alignment; exhaustive sweep of all 9 BC-INDEX subsystem headers confirmed SS-07 sole divergence; no CAP-ID mismatches; BC-INDEX 1.39→1.39.1 — Suggestion only, does NOT reset counter. 1 registry entry bumped atomically (L-S027-004): BC-INDEX→1.39.1 (EVAL-INDEX inputs pin also propagated: 1.39→1.39.1). Count-propagation sweep: no remaining active BC-INDEX v1.39 literals post-update. POL-11 PASS (328 active). POL-12 PASS. CONSECUTIVE-CLEAN COUNTER = 2; Pass-22 next (clean candidate 3 of 3 → Phase-1d CONVERGED if clean → next step: /vsdd-factory:check-input-drift then human spec-package approval gate CC-TUITERM-WIP-SIGNOFF + CC-GLOBAL-MOUSE-CAPTURE). STATE v7.13→v7.14. | 2026-06-04 | state-manager |
| D-262 | Adversarial Pass-20 CLEAN — FIRST CLEAN PASS. 0 Critical / 0 Important / 2 Suggestions. Multi-axis adversary sweep (anchors, IPC taxonomy, security/SO_PEERCRED, PRD↔BC-INDEX counts, numeric constants, holdout integrity, BC-internal PC↔Inv↔EC, frontmatter bidirectionality) found NO Critical/Important defect. FOURTEENTH consecutive zero-Critical. Consecutive-clean counter 0→1 (need 3 consecutive per ratified human directive). S20-001 (BC-2.05.009 H1 'Surfaced Drop Counter' title↔body tension vs PC-3 'NOT surfaced in TUI') fixed post-clean as title-precision (retitled '...with Drop Counter (stderr WARN) + PtyReset TUI Recovery'; propagated H1→BC-INDEX+PRD §2.5 per bc_h1_is_title_source_of_truth; 1.5.0→1.5.1) — Suggestion only, does NOT reset counter. S20-002 (ADR-0011 'upgrade upgrade' doubled-word typo) DEFERRED as housekeeping (typo cannot escalate; avoids disproportionate ADR-0011 bump+citation-sweep; registered as ADR-0011-UPGRADE-TYPO). 1 registry entry bumped atomically (L-S027-004): BC-2.05.009→1.5.1. POL-11 PASS (328 active). POL-12 PASS. Consecutive-clean counter = 1; Pass-21 next (clean candidate 2 of 3). STATE v7.12→v7.13. | 2026-06-04 | state-manager |
| D-261 | Adversarial Pass-19 convergence-fix — 0 Critical / 1 Important / 1 Suggestion. I19-001 (tui-term pin FORM drift: shown as caret '0.3' in ADR-0011 §Decision + brief/vision Tech-Stack tables, contradicting the ratified EXACT pin '=0.3.4' per ADR-0011 §Q-7 + deps-manifest — within-document contradiction + partial-fix regression; caret would silently absorb 0.3.5+ defeating the WIP-risk mitigation) FIXED: ADR-0011 §Decision → '=0.3.4' (1.2.0→1.2.1) + exhaustive pin-FORM sweep of all in-scope arch docs confirmed tui-term sole drift (portable-pty/vt100 correctly caret; all exact-policy crates correct); brief §Tech-Stack tui-term →'=0.3.4' (2.0.2→2.0.3) + vision §Tech-Stack →'=0.3.4' (2.2.1→2.2.2). BC-2.09.001 ADR-0011 citation swept v1.2.0→v1.2.1 (mechanical POL-11 propagation). S19-001 (SS-daemon-wiring-v2-delta lines ~195/~822 bare 'ScrollbackDump' parenthetical generic label) DEFERRED as housekeeping (Suggestion; non-live survivor; disproportionate to bump heavily-cited SS; registered as SS-DAEMON-WIRING-SCROLLBACKDUMP-TERM). 2 registry entries bumped atomically (L-S027-004): ADR-0011→1.2.1, product-brief→2.0.3. POL-11 PASS (328 active). POL-12 PASS. THIRTEENTH consecutive zero-Critical (C:...,0×13). Consecutive-clean counter = 0; Pass-20 next. STATE v7.11→v7.12. | 2026-06-04 | state-manager |
| D-260 | Adversarial Pass-18 convergence-fix — 0 Critical / 1 Important / 2 Suggestions. I18-001 (retired daemon-owned/in-process-PTY persistence model survived as LIVE normative framing in vision §Non-Goals/§Five-Planes + brief §Out-of-Scope, contradicting ratified session-host-owns-PTY ADR-0009/D-238 — partial-fix regression; the v2.1 rename reached §Process Topology but not §Non-Goals) FIXED via exhaustive sweep of vision+brief: 4 live survivors (vision 122/588/592, brief 259) → session-host-owned framing; zero live survivors confirmed; consistency correction only (no decision change). vision 2.2→2.2.1, brief 2.0.1→2.0.2. S18-001 (embedded-pty-evaluation.md pre-decision research doc = leak origin) closed via superseded banner (architect; analysis preserved; not registry-tracked). S18-002 (BC-2.09.009 §Trace v1.0.0 once-only bell) LEFT — exempt historical §Trace, correctly superseded by v1.1.0. 1 registry entry bumped atomically (L-S027-004): product-brief→2.0.2. POL-11 PASS (329 active). POL-12 PASS. TWELFTH consecutive zero-Critical (C:...,0×12). Consecutive-clean counter = 0; Pass-19 next. STATE v7.10→v7.11. | 2026-06-04 | state-manager |
| D-258 | Adversarial Pass-16 convergence-fix — 0 Critical / 1 Important / 1 Suggestion. I16-001 (PRD §2.8 omitted active P0 BC-2.08.008 SessionStateChanged/ordering contract — undetected for 15 passes because the Pass-15 cross-check artifact hand-typed SS-08 cardinality as 7 instead of 8, a self-falsifying false-green; BC-2.08.008 row added to §2.8; false §Trace line-1558 SS-08=7→8 attestation corrected; derived count cross-check recorded (SS-03=8, SS-05=11, SS-06=25, SS-08=8, SS-09=9 — all PRD §2.NN row counts == BC-INDEX Summary; orchestrator independently verified BC-2.08.008 is the ONLY in-scope gap); prd 1.28.2→1.28.3). S16-001 [process-gap] (PRD §2 sync hand-enumerated expected rows instead of deriving counts from BC-INDEX → false-green) recorded as PRD-COUNT-CROSSCHECK-RULE in durable_task_register. L-VERIFICATION-ARTIFACT-FALSE-GREEN codified in cycles/cycle-001/lessons.md. 1 registry entry bumped atomically (L-S027-004): prd→1.28.3. POL-11 PASS (319 active). POL-12 PASS. TENTH consecutive zero-Critical (C:...,0×10). Consecutive-clean counter = 0; Pass-17 next. STATE v7.08→v7.09. | 2026-06-04 | state-manager |
| D-255 | Adversarial Pass-13 convergence-fix — 0 Critical / 2 Important / 2 Suggestions. I13-001+S13-001 (BC-2.03.005 VP row-1 + EC-102 still cwd=project_root — 3rd recurrence of I2-002 worktree_root partial-fix; false §Trace/BC-INDEX attestation) FIXED EXHAUSTIVELY (whole-file grep; zero stale CWD-source survivors; §Trace attestation corrected; 1.1.1→1.1.2). I13-002 (ADR-0009 line 165 mis-anchor 'SS-09 Session Host'→SS-08 Session Manager; 1.0.0→1.0.1); sibling sweep caught inverted SS-08/SS-09 cross-refs in ADR-0011 (1.1.0→1.2.0). S13-001 folded into I13-001. S13-002 (Invariant 3 'MERGED' vs 'OVERLAY') adjudicated INTENTIONAL no-change per BC-2.03.006 §Trace v1.1.0 ratification. 3 registry entries bumped atomically (L-S027-004): BC-2.03.005→1.1.2, ADR-0009→1.0.1, ADR-0011→1.2.0. POL-11 PASS (319 active). POL-12 PASS. SEVENTH consecutive zero-Critical (C:...,0,0,0,0,0,0,0). Consecutive-clean counter = 0; Pass-14 next. L-CWD-PROPAGATION-ATTESTATION codified (cycles/cycle-001/lessons.md). STATE v7.05→v7.06. | 2026-06-04 | state-manager |
| D-254 | Adversarial Pass-12 convergence-fix — 0 Critical / 2 Important / 2 Suggestions. I12-001 (EngineError→ServerToClient::Error taxonomy gap; BinaryNotFound/InvalidPath collapsed to generic spawn_failed; BC-2.03.007 messages unsatisfiable) FIXED: spawn_recipe() call-site annotation pinned in spawn_session() first-step (SS-daemon-wiring-v2-delta 1.6.0→1.7.0); taxonomy 8→10 codes (binary_not_found, invalid_spawn_arg) + fixed banners (SS-ipc 1.16.0→1.17.0); SessionError::EngineError(#[from]) bridge + session_error_to_code arm, exhaustiveness preserved (SS-session-manager 1.7.2→1.8.0); BC-2.03.007 PC-3/PC-7 reconciled to canonical codes (1.0.0→1.1.0). I12-002 (BC-2.03.005 Description partial-fix regression: cwd still project_root) → worktree_root (1.1.0→1.1.1). S12-001: SS-embedded-pty App struct dump_in_progress/pending_pty_bytes fields + skeleton fix (1.4.0→1.5.0). S12-002: SS-ipc §Scope multi-client future-scope cross-ref to TD-MULTI-CLIENT-ATTACH-STORM-001 (part of 1.17.0 bump). Propagation sweep: 35 unique files swept (POL-11-driven). 6 registry entries bumped atomically (L-S027-004). POL-11 PASS (319 active). POL-12 PASS. SIXTH consecutive zero-Critical (C:...,0,0,0,0,0,0). Consecutive-clean counter = 0 (Important present); Pass-13 next. STATE v7.04→v7.05. | 2026-06-04 | state-manager |
| D-253 | Adversarial Pass-11 convergence-fix — 0 Critical / 1 Important / 4 Suggestions. I11-001 (multi-client late-join) adjudicated: PRONG A = REAL v1A gap (auto-attach-on-EmbeddedTerminal-entry now mandated; SS-embedded-pty 1.3.0→1.4.0 + BC-2.09.001 PC-6, 1.2.0→1.3.0); PRONG B = RATIFIED-FUTURE (BC-2.05.009 Inv-2; scope-boundary note + deferral anchor TD-MULTI-CLIENT-ATTACH-STORM-001 added to SS-daemon-wiring-v2-delta 1.5.0→1.6.0). S11-001: phantom kill_deadline_reason §Trace corrected (SS-session-manager 1.7.1→1.7.2). S11-002/003 deferred. S11-004-class → Phase-2 story-writer. SS-ipc doc-comment extended 1.15.0→1.16.0. Propagation sweep: 35 unique files swept. 5 registry entries bumped atomically: SS-embedded-pty→1.4.0, SS-daemon-wiring-v2-delta→1.6.0, SS-session-manager→1.7.2, SS-ipc→1.16.0, BC-2.09.001→1.3.0. POL-11 PASS (314 active). POL-12 PASS. FIFTH consecutive zero-Critical. Consecutive-clean counter = 0 (Important present); Pass-12 next. STATE v7.03→v7.04. | 2026-06-04 | state-manager |
| D-252 | Adversarial Pass-10 convergence-fix — 0 Critical / 1 Important / 4 Suggestions. Important + 3 answerable Suggestions fixed in-scope; S10-004 (S-TBD holdout/BC anchors) deferred to Phase-2 story-writer (tracked). I10-001: SS-session-manager re-attach wording corrected to ClientToServer::AttachSession in §Screen-state-transfer step 5a and §PTY-reader-thread step 4 (v1.7.0→v1.7.1). S10-001: SS-deps-pin-manifest-v2-delta historical annotation on registry-action block. S10-002: BC-2.05.010 success POs now name ordered SessionStateChanged→SessionListUpdate emission sequence (v1.4.0→v1.5.0). S10-003: ADR-0009 doubled-word typo. Closure grep CLEAN (BC-2.05.011:118 daemon→host DaemonToHost::Attach is correct direction, not a survivor). Stale-literal sweep: 13 BC files + HS-EXP-014 updated SS-session-manager v1.7.0→v1.7.1. 2 registry entries bumped atomically (L-S027-004): SS-session-manager→1.7.1, BC-2.05.010→1.5.0. POL-11 PASS (313 active). POL-12 PASS. FOURTH consecutive zero-Critical (C:5→5→4→1→2→2→0→0→0→0). Consecutive-clean counter = 0 (Important present); Pass-11 next. STATE v7.02→v7.03. | 2026-06-04 | state-manager |
| D-251 | Durable zero-context resume checkpoint written (next_session_resume_protocol rewritten to v7.02; NEXT-SESSION-RESUME.md created on develop) at human request — convergence paused at Pass-9-done / consecutive-clean counter = 0 / Pass-10-next. STATE v7.01→v7.02. | 2026-06-03 | state-manager |
| D-250 | Adversarial Pass-9 (control-center spec package) convergence-fix — 0 Critical / 1 Important / 1 Suggestion ALL resolved. 3rd consecutive zero-Critical pass (C:5→5→4→1→2→2→0→0→0; I:8→6→9→4→4→2→4→2→1). I-P9-001: ADR-0010 §IPC-Message-Type-Additions ScrollbackChunk/ScrollbackDumpComplete doc-comments carried the RETIRED pause-during-dump model — the last live normative I3-003 propagation residue (I-P7-004 fixed SS-session-manager lines ~717/843 but missed ADR-0010's own variant doc-comments). ScrollbackChunk doc-comment rewritten: RETIRED "TUI MUST NOT begin rendering PTY bytes until ScrollbackDumpComplete" → snapshot-then-resume with pending_pty_bytes buffer; cross-ref to §Interleaving added. ScrollbackDumpComplete doc-comment rewritten: three-step TUI procedure added (reset parser from snapshot rows; replay pending_pty_bytes in receipt order; clear buffer + set dump_in_progress=false); explicit note: session-host NOT paused during dump. Comprehensive architecture-wide sweep confirmed sole normative survivor (SS-session-manager lines ~717/793 already correct; SS-daemon-wiring-v2-delta §I3-003 fix narrative = legitimate historical §Trace; SS-session-manager line ~980 watchdog timeout = operational, unrelated; SS-engine-module "paused awaiting user decision" = permission overlay, unrelated; SS-daemon-wiring-impl "pause before writing body" = HTTP test prose, unrelated). ADR-0010 v1.4.0→v1.5.0. Stale-literal sweep: BC-2.05.009 Architecture Source row + BC-2.05.011 Architecture Source row updated v1.4.0→v1.5.0 (2 active literals). BC-2.05.011 §Trace v1.1.0 line 219 is historical anchor per ADR-0007 §Historical Anchor Classification — exempt from update, confirmed. S-P9-001: HS-EXP-014 FAIL criterion + C2-004 modification note "schema v2 per SS-session-manager v1.7.0" → "schema v3 (schema_version: 3) per SS-session-manager v1.7.0" (cosmetic stale label; assertion correct in both). EVAL-INDEX v1.14→v1.15. Adversary confirmed: anchor-resolution PASS, security PASS, no design gaps; expects Pass-10 CLEAN. STREAK: Pass-9 non-clean (Important present) → consecutive-clean counter remains 0; need 3. 2 registry entries bumped atomically (L-S027-004): ADR-0010→1.5.0, EVAL-INDEX→1.15. POL-11 PASS. POL-12 PASS. STATE v7.00→v7.01. | 2026-06-03 | state-manager |
| D-249 | Adversarial Pass-8 (control-center spec package) convergence-fix — 0 Critical / 2 Important / 1 Suggestion ALL resolved. 2nd consecutive zero-Critical pass (C:5→5→4→1→2→2→0→0; I:8→6→9→4→4→2→4→2). Both Importants were partial-fix propagation in SS-daemon-wiring-v2-delta §3 IPC handler — Pass-7 op-aware/no-silent-failure pattern landed in SS-session-manager+SS-ipc but not the delta's inline handler copy: I-P8-001 §3 KillSession/AttachSession used single-arg session_error_to_code → op-aware IpcOp::Kill/Attach; architect also caught SpawnSession hardcoding 'spawn_failed' → session_error_to_code(IpcOp::Spawn,&e). I-P8-002 §3 KeyInput/Detach/Rename bare ?-propagation (silent connection-drop) → ServerToClient::Error routing; ResizePane → WARN-continue. All 9 §3 arms now canonical; zero bare-? SessionError escapes. STRUCTURAL: canonical-pattern-lock note added to §3 (2nd delta-vs-canonical drift occurrence; cf I6-002). S-P8-001 HS-EXP-015 input-class count 5→6. SS-daemon-wiring-v2-delta v1.4.0→v1.5.0. EVAL-INDEX v1.13→v1.14. Adversary confirmed: retired-concept sweep clean, security PASS, no design gaps. STREAK: Pass-8 non-clean (Important present) → consecutive-clean counter remains 0; need 3. 2 registry entries bumped atomically (L-S027-004). Stale-literal propagation sweep: 14 active literals fixed across 9 files (SS-daemon-wiring-v2-delta v1.4.0→v1.5.0 in BC-2.05.009/010/011, BC-2.08.001/003/004/007/008, SS-ipc). POL-11 PASS. POL-12 PASS. STATE v6.99→v7.00. | 2026-06-03 | state-manager |
| D-248 | Adversarial Pass-7 (control-center spec package) convergence-fix — 0 Critical / 4 Important / 4 Suggestions ALL resolved. FIRST zero-Critical pass (C:5→5→4→1→2→2→0). Findings were side-effects of Pass-6 §Error-handling addition + I3-003 residue: I-P7-001 SessionError taxonomy gap (BC-2.08.001 EC-151/152 referenced SidecarWriteFailed/SessionIdCollision not in enum) → both added + distinct codes (sidecar_write_failed/session_id_collision; taxonomy 6→8 codes in SS-session-manager v1.6.0→v1.7.0 + SS-ipc v1.14.0→v1.15.0); I-P7-002 kill_failed dead code → op-aware session_error_to_code(IpcOp,&SessionError) so kill-path SessionHostDead→kill_failed reachable; I-P7-003 ServerToClient::Disconnected non-variant in canonical reader → IpcRead{Msg,Disconnected} wrapper enum matching shipped S-025 impl; I-P7-004 retired pause-during-dump survivors (SS-session-manager L668/L843) → snapshot-then-resume; S-P7-001 serde-on-variant removed; S-P7-002 BC-2.09.002 stray removal_range field (v1.0.0→v1.0.1); S-P7-003 BC-2.05.010 H1 +AttachSession (7 variants, v1.3.0→v1.4.0); BC-INDEX v1.38→v1.39 title sync. Adversary confirmed: anchor-resolution PASS, security PASS, no design gaps. STREAK: Pass-7 non-clean (Important present) → consecutive-clean counter remains 0; need 3. 5 registry entries bumped atomically (L-S027-004). Stale-literal propagation sweep: 50 active literals fixed across 22 files (SS-ipc v1.14.0→v1.15.0 in 15 files + SS-session-manager v1.6.0→v1.7.0 in 15 files). POL-11 PASS. POL-12 PASS. NEXT = adversarial Pass-8. STATE v6.98→v6.99. | 2026-06-03 | state-manager |
| D-247 | Adversarial Pass-6 (control-center spec package) convergence-fix — 2 Critical / 2 Important ALL resolved. Findings were dangling-reference residue EXPOSED by Pass-5 SS-ipc census consolidation: C6-001 ServerToClient::Error variant undefined though BC-2.05.010 requires it (silent-failure on launch) → added as 13th ServerToClient variant in SS-ipc v1.14.0 + code taxonomy (spawn_failed/session_not_found/attach_failed/kill_failed/rename_failed/invalid_request) + new SS-08 §Error handling mapping (SessionError→ServerToClient::Error, no Ok() at task boundary on Err) in SS-session-manager v1.6.0 + daemon-wiring error routing in SS-daemon-wiring-v2-delta v1.4.0; C6-002 uncompilable ClientToServer::ReAttach in daemon-wiring handler (I3-004 residue) removed from SS-daemon-wiring-v2-delta v1.4.0; I6-001 HS-EXP-015 nonexistent MouseInput→KeyInput (HS-EXP-015 holdout fixed + EVAL-INDEX v1.13); I6-002 daemon-wiring stale inline SessionSnapshot §4 retired → pointer to SS-ipc.md §Supporting Types (SS-daemon-wiring-v2-delta v1.4.0). BC-2.05.010 v1.3.0: error codes aligned to taxonomy. Adversary CONFIRMED all retired-concept discipline clean, security sound, no new design gaps. Novelty decaying to dangling-reference homing (C:5→5→4→1→2→2; I:8→6→9→4→4→2). 5 registry entries bumped atomically (L-S027-004): SS-ipc→1.14.0, SS-session-manager→1.6.0, SS-daemon-wiring-v2-delta→1.4.0, BC-2.05.010→1.3.0, EVAL-INDEX→1.13. Stale-literal propagation sweep: 3 active literals fixed (HS-EXP-014.md SS-session-manager v1.5.0→v1.6.0). POL-11 PASS. POL-12 PASS. Pass-6 was non-clean; convergence still needs 3 consecutive CLEAN (Pass-7 next). STATE v6.97→v6.98. | 2026-06-03 | state-manager |
| D-246 | Adversarial Pass-5 (control-center spec package) convergence-fix — 2 Critical / 4 Important ALL resolved. NOVEL structural finding (survived 4 passes): C5-001 SS-ipc.md (ARCH-INDEX-designated IPC authority, cited by SS-05 BCs as Architecture Source) was MISSING 11 v1A wire variants (PtyOutput/SessionStateChanged/ScrollbackChunk/ScrollbackDumpComplete/PtyReset/KeyInput/ResizePane/SpawnSession/KillSession/DetachSession/RenameSession) — they lived only in ADR-0010 + SS-daemon-wiring-v2-delta. FIXED: all 11 folded into SS-ipc.md v1.13.0 enum bodies (12 ServerToClient + 9 ClientToServer; orchestrator-verified all 15 cited symbols present). C5-002 SerializedCell/SerializedColor moved to monocle-ipc §Supporting Types (same class as C3-004 SessionSnapshot); SS-session-manager v1.5.0 references them. I5-001 BC-2.05.009 drop-vs-backpressure self-contradiction resolved to backpressure-no-drop (PC-2/EC-270/test-vector rewritten to match Invariants 1/3) v1.4.0→v1.5.0. I5-002 ADR-0010 throughput sizing reconciled with 256KiB MAX_MESSAGE_BYTES v1.3.1→v1.4.0. I5-003 confirmed BC-2.09.003 already uses pane_area (stale SS-embedded-pty forward-instruction settled); SS-embedded-pty v1.2.0→v1.3.0. I5-004 SS-embedded-pty pty_scroll_offset→pty_scroll_offsets prose. PROCESS-GAP CODIFIED: anchor-resolution-closure check — every BC Architecture-Source citation must resolve to a symbol that EXISTS in the cited doc (would have caught C5-001 at fix time). 5 registry entries bumped atomically (L-S027-004). 68 stale literals fixed across 32 files. POL-11 PASS (310 active). POL-12 PASS. NEXT = adversarial Pass-6 (need first of 3 consecutive clean). STATE v6.96→v6.97. | 2026-06-03 | state-manager |
| D-245 | Adversarial Pass-4 (control-center spec package) convergence-fix — 1 Critical / 4 Important / 3 Suggestions ALL resolved. Novelty DECAYING (C:5→5→4→1, I:8→6→9→4); adversary confirmed NO new architectural/security gaps — all findings were propagation residue from two Pass-3 fixes (ScrollbackDump→chunked retirement; per-client buffer 256→64). CRIT-001 BC-2.09.001 Inv-5: nonexistent ServerToClient::ScrollbackDump retired→chunked (BC-2.09.001 v1.1.0→v1.2.0). HIGH-001 BC-2.08.002 PC-4/PC-7/Inv-4: retired HostToDaemon::ScrollbackDump→chunked protocol (BC-2.08.002 v1.1.0→v1.2.0). HIGH-002 BC-2.08.007 title+Description propagated to PRD §2.8/§2.9 + BC-INDEX changelog (BC-2.08.007 v1.3.0→v1.4.0, PRD v1.28.0→v1.28.1, BC-INDEX v1.37→v1.38). HIGH-003 BC-2.05.009 PC-1b buffer 256→64 propagation residue (BC-2.05.009 v1.3.0→v1.4.0). HIGH-004 ADR-0010 line-310 + SS-daemon-wiring-v2-delta PO-edit-block CLIENT_SEND_BUFFER_SIZE 256→64 — the re-injection source retired (ADR-0010 v1.3.0→v1.3.1, SS-daemon-wiring-v2-delta v1.3.0→v1.3.1). SUG-001 vt100 0.16 reconstruction path canonicalized — Parser::process is the only path; no set_screen method (verified). SUG-002 spawned_by_monocle EnrichedSession→SessionSnapshot type (SS-session-manager v1.4.0→v1.4.1, BC-2.08.006 v1.1.0→v1.2.0). SUG-003 whole-tree grep gate applied — confirmed ZERO live normative ScrollbackDump/256 survivors (orchestrator independently verified); codified in cycle checklist. 10 registry entries bumped atomically (L-S027-004). POL-11 PASS. POL-12 PASS. CYCLE CHECKLIST CODIFIED (SUG-003): after RETIRED-term or corrected-magic-number fix, grep -rn .factory/specs before next adversarial pass — every survivor must be in §Trace/changelog/enforcement, NOT live PC/Invariant/title. NEXT = adversarial Pass-5 (Pass-4 last non-clean; need first of 3 consecutive clean). STATE v6.95→v6.96. | 2026-06-03 | state-manager |
| D-244 | Adversarial Pass-3 (control-center spec package) convergence-fix — 4 Critical / 9 Important / 4 Observations ALL resolved. Architect: C3-001 SessionStateChanged emission point (SS-daemon-wiring §3b + per-method table); C3-003 rename emits SessionListUpdate only; C3-004 SessionSnapshot wire boundary type defined (reconciles SessionEntry/EnrichedSession/SessionSnapshot); I3-001 ordering = per-client channel FIFO (not mutex) + split-pair→disconnect; I3-002 Terminating watchdog = post-bind bg task + kill_deadline persistence (schema_version 3); I3-003 dump snapshot-then-resume (no harness stall); I3-004 ClientToServer::AttachSession variant; I3-005 Detached preserved on re-discovery; I3-008 SerializedCell.attrs verified vs vt100 0.16 (5 attrs, no blink); I3-009 degraded-env surfaced to TUI; O3-004 per-client buffer 64. PO: C3-002 schema_version 3 consistent (self-deleting-sidecar bug closed); SessionStateChanged emission BCs; AttachSession BCs; Detached/Terminating BC sync; I3-006 SO_PEERCRED universal; O3-001 dup PC. PROPAGATION-CLOSURE AUDIT (pre-commit): caught F-01 (SS-ipc EnrichedSession prose) + F-02 (BC-2.05.011 buffer 256→64) before adversary. PROCESS WIN: codified — run closure audit before each adversarial pass. 13 registry entries bumped (L-S027-004 recurrence caught and corrected). 39 stale-literals fixed (SS-ipc v1.11.0→v1.12.1 across 9 SS-05/SS-06 BCs; SS-ipc v1.12.0→v1.12.1 in 4 BCs; SS-session-manager v1.3.0→v1.4.0 in 9 BCs/HS-EXP-014/registry). POL-11 PASS (309 active). POL-12 PASS. NEXT = adversarial Pass-4. STATE v6.94→v6.95. | 2026-06-03 | state-manager |
| D-243 | Adversarial Pass-2 convergence-fix — 5 Critical / 6 Important / 5 Suggestions ALL resolved. Architect: C2-002 chunked ScrollbackChunk/ScrollbackDumpComplete/PtyReset ServerToClient wire variants + buffer-then-apply interleaving (ADR-0010 v1.2.0); I2-003 per-client isolated send buffers (cap 256) so backpressure derives from durable ring not clients (SS-daemon-wiring-v2-delta v1.2.0); I2-002 worktree-per-session operationalized (worktree_root field; cwd=project_root bug fixed) (SS-engine-module-v2-delta v1.1.0, SS-session-manager v1.3.0); I2-004 Terminating transient state + 12s watchdog; C2-005 SO_PEERCRED on all per-session UDS connects incl kill-path + 2s/10s escalation reconciled; I2-006 env inheritance (SS-embedded-pty v1.2.0); C2-003 stale removal-directive reconciled; S2-002 dup Kitty arm; S2-003 scrollback memory. PO: C2-001 per-prompt-bell rule (BC↔holdout consistent); C2-004 HS-EXP-014 phantom sidecar field removed; I2-001 mouse-capture scoped BC; I2-005 Launching re-discovery synced; S2-004 zero-dim resize clamp; S2-005 NEW BC-2.08.008 SessionStateChanged emission; NEW BC-2.05.011 ServerToClient variants. BC-INDEX v1.36→v1.37 (136→138 BCs). EVAL-INDEX v1.11→v1.12. 22 registry entries bumped (21 doc versions + EVAL-INDEX). 5+17+1 = 23 files in atomic commit. POL-11 PASS (31 stale-literals fixed, 289 active). POL-12 PASS. Story impact: all S-TBD (undecomposed); story-writer picks up final BC set at decomposition. VP propagation deferred to hardening (VP-TBD). 0 consecutive clean passes so far; need 3 for human gate. NEXT = Adversarial Pass-3. STATE v6.93→v6.94. | 2026-06-03 | state-manager |
| D-242-fix | Registry-atomicity correction: BC-2.06.025 entry in version-pin-registry.yaml updated 1.0.0→1.1.0 (D-242 O5 fix — spawned_by_monocle None→[?] render + EC-295 — was bumped in BC file but registry entry missed in D-242 burst). Recurrence of L-S027-004 class (registry-atomicity miss). State-manager burst verification must confirm EVERY listed registry bump landed before push. [process-gap] — state-manager checklist must include a line-by-line registry diff against claimed bump count. POL-11 PASS (272 active). POL-12 PASS. STATE v6.92→v6.93. | 2026-06-03 | state-manager |
| D-242 | Adversarial Pass-1 (control-center spec package) convergence-fix — 5 Critical / 8 Important / 5 Observations ALL resolved. Architect: C2 lock-file/re-discovery dual-readiness-signal clarified (lock-file=step 8, UDS-bind=step 10, foreground-caller vs TUI-client contracts distinct); C3+I8 PTY backpressure (.send().await replacing silent drops) + PtyReset recovery protocol + TUI-surfaced reset indicator + ADR-0010 head-of-line priority & benchmark moved to hard pre-story-wave gate; C5 styled-cell ScrollbackDump screen-state transfer (Vec<Vec<SerializedCell>> replacing Vec<String>); I1 full mouse/keyboard encoding (SGR-1006, Alt/Meta, Shift+Tab, modified arrows); I3 mouse capture scoped to EmbeddedTerminal entry/exit; I4 dead SessionStates (Created/Killed) pruned + re-discovery state handling; I5 SO_PEERCRED + sidecar-trust on per-session UDS (closes keystroke-injection vector); I6 PID-based orphan kill pre-socket-bind; I7 per-session scroll offsets. PO: C1 SHARED hooks-file model (BC-HOOK-010 wins; BC-2.08.006 revised to shared-file per-runtimeDir; per-session hooks-file model removed); C4 holdout data-model fixed (flat session-<uuid>.json + state/Running); HS-EXP-014 shared-file; I2 BC status convention confirmed (active+S-TBD backfilled at decomposition — no change); O3/O5 fixed. BC-INDEX v1.35→v1.36 (136 BCs). EVAL-INDEX v1.10→v1.11. 15 version-pin-registry entries bumped atomically. POL-11 PASS (26 stale-literals in 21 files fixed; 268 active). POL-12 PASS. 2 new human-gate items in durable_task_register: CC-TUITERM-WIP-SIGNOFF + CC-GLOBAL-MOUSE-CAPTURE (non-blocking; required before v1A story wave). STATE v6.91→v6.92. | 2026-06-03 | state-manager |
| D-240 | Consistency convergence — fresh-context consistency-validator found 12 findings (3 Critical/5 Important/4 Suggestion) on the control-center spec package; root cause = vision/brief lagged the v2.1 session-host model introduced by ADR-0009. ALL resolved: architect fixed IMP-2 (session_id:String canonical), IMP-3 (daemon-wiring step 8b placement), IMP-5 (InvalidPath error variant; BC-2.03.007 retitled), SUG-3 (production-grade permission badge+bell during embedded-terminal, no silent suppression; v1B pre-emption flagged for human ratification), SUG-4 (ARCH-INDEX token estimates) → SS-session-manager v1.0.1, SS-embedded-pty v1.0.2, SS-engine-module-v2-delta v1.0.1, SS-daemon-wiring-v2-delta v1.0.1, ARCH-INDEX v1.0.28. product-owner propagated v2.1 session-host model + rulings into vision v2.2 and brief v2.0.1 (CRIT-1/2/3, IMP-1, SUG-1/2). IMP-4 (stale registry product-brief 1.4.34→2.0.1) fixed. NEW open item (non-blocking): v1B embedded-terminal→overlay pre-emption requires human ratification before BC authoring. NEW BC needed: permission badge+bell guarantee during EmbeddedTerminal/SessionCreation (product-owner to author in BC/PRD delta). STATE v6.89→v6.90. | 2026-06-03 | state-manager |

## Key Tech Stack (D-229 canonical)

ratatui 0.30 | crossterm 0.29 | tokio 1.52 | axum 0.8 | interprocess 2.4 | prost 0.14
serde_yaml_ng 0.10 | wasmtime 44 | nucleo 0.5 | time 0.3.47 (RUSTSEC-2026-0009 floor)
serde_json =1.0.149 | rand =0.8.6 | 28 pinned deps. SS-deps-pin-manifest v1.2.1 (D-235: +tracing-subscriber 0.3 prod, +ureq/libc dev-deps).
MSRV: Rust 1.88 (Phase 1-2); Rust 1.92 (Phase 3, wasmtime 44).
**PRD v1.28.3** | **BC-INDEX v1.39.1** (138 BCs) | **ARCH-INDEX v1.0.28** | **SS-tui v1.8.2**
**SS-ipc v1.17.0** | **SS-conventions v1.32.6** | **ADR-0007 v1.0.8** | **ADR-0008 v1.0.6** | **STORY-INDEX v5.32**
**SS-deps-pin-manifest v1.2.1** | **SS-daemon-wiring-impl v1.3.0** | **SS-session-manager v1.8.1** | **SS-embedded-pty v1.5.0**
**BC-2.05.004 v1.1.0** | **BC-2.05.009 v1.5.1** | **BC-2.05.010 v1.5.0** | **BC-2.06.006 v1.1.0** | **BC-2.06.015 v1.0.7** | **BC-2.06.016 v1.1.0**
**BC-2.06.018 v1.1.0** | **BC-2.06.019 v1.1.0** | **BC-2.06.020 v1.1.0** | **BC-2.06.021 v1.0.7** | **BC-2.06.023 v1.5.1**
**BC-2.06.024 v1.1.0** | **BC-2.07.004 v1.0.2** | **BC-2.07.005 v1.3.1** | **BC-2.09.002 v1.0.1** | **BC-HOOK-034 v1.0.2**
**S-026 v1.11** | **S-027 v1.10** | **product-brief v2.0.3 (draft; consistency-gate-passed)**
**EVAL-INDEX v1.15** | **ADR-0010 v1.5.0** | **SS-daemon-wiring-v2-delta v1.7.0** | **STORY-INDEX v5.32** | **sprint-state v1.40** (32/33 done, 192/195 pts; wave-7 gate PASSED D-232). **S-029 v1.3**. 64 codified disciplines. D-235: Daemon-wiring CONVERGED. D-242: Adv Pass-1 DONE. D-243: Adv Pass-2 DONE. D-244: Adv Pass-3 DONE. D-245: Adv Pass-4 DONE. D-246: Adv Pass-5 DONE (SS-ipc v1.13.0 — NOVEL C5-001: SS-ipc now complete IPC wire authority with all 11 v1A variants). D-247: Adv Pass-6 DONE (SS-ipc v1.14.0 ServerToClient::Error 13th variant + code taxonomy; SS-session-manager v1.6.0 §Error handling; SS-daemon-wiring-v2-delta v1.4.0 error routing; BC-2.05.010 v1.3.0; EVAL-INDEX v1.13). D-248: Adv Pass-7 DONE (FIRST zero-Critical). D-249: Adv Pass-8 DONE (SS-daemon-wiring-v2-delta v1.5.0 §3 all-arms canonical; EVAL-INDEX v1.14). D-250: Adv Pass-9 DONE (ADR-0010 v1.5.0 snapshot-then-resume doc-comments; EVAL-INDEX v1.15 schema v3 label). D-252: Adv Pass-10 DONE (SS-session-manager v1.7.1 re-attach ClientToServer::AttachSession naming; BC-2.05.010 v1.5.0 ordered emission sequence). D-253: Adv Pass-11 DONE (SS-embedded-pty 1.4.0 auto-attach mandate; SS-daemon-wiring-v2-delta 1.6.0 multi-client scope boundary; SS-session-manager 1.7.2; SS-ipc 1.16.0; BC-2.09.001 1.3.0). D-254: Adv Pass-12 DONE (SS-ipc 1.17.0 taxonomy 8→10; SS-session-manager 1.8.0 EngineError bridge; SS-daemon-wiring-v2-delta 1.7.0 spawn call-site; SS-embedded-pty 1.5.0 App struct; BC-2.03.005 1.1.1 worktree_root; BC-2.03.007 1.1.0 canonical codes). D-255: Adv Pass-13 DONE (BC-2.03.005 1.1.2 exhaustive worktree_root; ADR-0009 1.0.1; ADR-0011 1.2.0; L-CWD-PROPAGATION-ATTESTATION). D-256: Adv Pass-14 DONE (SS-session-manager 1.8.1 portable-pty 0.9.x lone stale crate-pin; exhaustive sweep class closed; DEP-PIN-SWEEP-RULE). D-257: Adv Pass-15 DONE (BC-2.05.010 1.5.1; prd 1.28.2). D-258: Adv Pass-16 DONE (prd 1.28.3 §2.8 BC-2.08.008 row; L-VERIFICATION-ARTIFACT-FALSE-GREEN; PRD-COUNT-CROSSCHECK-RULE). D-262: Adv Pass-20 DONE — FIRST CLEAN PASS (0C/0I/2S). BC-2.05.009 1.5.1 (S20-001 title precision). CONSECUTIVE-CLEAN COUNTER = 1. D-263: Adv Pass-21 DONE — SECOND CLEAN PASS (0C/0I/1S). BC-INDEX 1.39→1.39.1 (S21-001 SS-07 header CAP-007 text alignment). CONSECUTIVE-CLEAN COUNTER = 2; Pass-22 next (candidate 3/3 → converges if clean).
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

## §Trace v7.09 (D-258 — Adv Pass-16 convergence-fix COMMITTED; Pass-17 next)

**D-258 (2026-06-04):** Adversarial Pass-16 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 0 Critical / 1 Important / 1 Suggestion. TENTH consecutive zero-Critical pass
(C:...,0×10).

I16-001 (Important): PRD §2.8 Session Manager BC table omitted BC-2.08.008
(SessionStateChanged emission ordering contract — P0, active, in BC-INDEX since D-243).
This finding went undetected for 15 adversarial passes. Root cause: the Pass-15 cross-check
verification artifact hand-typed the expected SS-08 BC set as 'BC-2.08.001..007' (7 rows)
rather than deriving the count from BC-INDEX §Summary (8 active BCs for SS-08). The
off-by-one in the hand-typed enumeration produced a self-falsifying false-green: the
cross-check passed as CLEAN while the real gap persisted. Orchestrator independently
verified BC-2.08.008 was the ONLY in-scope omission.

Fixes applied:
- BC-2.08.008 row added to PRD §2.8 Session Manager table.
- False §Trace line attestation (SS-08=7) corrected to SS-08=8.
- Derived count cross-check added to PRD §2 (SS-03=8, SS-05=11, SS-06=25, SS-08=8,
  SS-09=9 — all PRD §2.NN row counts equal BC-INDEX Summary active counts).
- prd v1.28.2→v1.28.3.

S16-001 (Suggestion — process-gap): PRD §2 sync sweeps hand-enumerated expected BC rows
instead of deriving them from BC-INDEX §Summary counts. This is the structural root cause
of I16-001 persisting for 15 passes. Recorded as PRD-COUNT-CROSSCHECK-RULE in
durable_task_register: add a structural-claim check asserting 'for each SS-NN, PRD §2.NN
BC-table row count == BC-INDEX §Summary active count for SS-NN'. Route to devops-engineer.

LESSON CODIFIED (L-VERIFICATION-ARTIFACT-FALSE-GREEN): verification/cross-check artifacts
that hand-enumerate expected sets create self-falsifying false-greens. Cross-check artifacts
MUST derive expected counts/sets from BC-INDEX §Summary, never hand-type them. Appended to
cycles/cycle-001/lessons.md as a [process-gap][codified] entry.

Registry atomicity (L-S027-004): 1 entry bumped atomically: prd→1.28.3.
Registry diff = exactly 1 changed version line. Spot-checked frontmatter: prd reads v1.28.3.
No code-delivery/ dirs staged.

POL-11 PASS (319 active). POL-12 PASS. Pass-16 non-clean (Important present) →
consecutive-clean counter = 0. NEXT = Pass-17 (clean-streak candidate 1-of-3).
STATE v7.08→v7.09.

## §Trace v7.07 (D-256 — Adv Pass-14 convergence-fix COMMITTED; Pass-15 next)

**D-256 (2026-06-04):** Adversarial Pass-14 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 0 Critical / 1 Important / 1 Suggestion. EIGHTH consecutive zero-Critical pass
(C:...,0,0,0,0,0,0,0,0).

I14-001 (Important): SS-session-manager §env-inheritance prose (line 597) cited portable-pty
0.8.x as a normative literal. The canonical version in SS-deps-pin-manifest-v2-delta and
ADR-0011 is 0.9.x. Architect ran an exhaustive crate-pin sweep across ALL in-scope specs
(portable-pty, vt100, tui-term, ratatui, crossterm, nucleo, axum, tokio, prost, etc.) and
confirmed I14-001 was the ONLY stale live crate-pin straggler — class CLOSED.
SS-session-manager v1.8.0→v1.8.1.

S14-001 (Suggestion — process-gap): POL-11 keys on artifact-ID version rows (e.g.,
"SS-session-manager v1.8.0") but NOT on crate-name+version literals in prose (e.g.,
"portable-pty 0.8.x"). Stale crate pins in prose escaped all 14 prior POL-11 sweeps.
Recorded as DEP-PIN-SWEEP-RULE in durable_task_register: extend POL-11 (or a sibling
check) to grep spec prose for crate-pin literals and validate against SS-deps-pin-manifest
(-v2-delta). Route to devops-engineer when a POL-extension sprint is scheduled. Non-blocking.

Propagation sweep (S-7.02 Defensive Sweep Discipline): 15 unique files updated (27 stale
active literals). Files: BC-2.03.005, BC-2.03.007, BC-2.05.009, BC-2.05.011, BC-2.06.025,
BC-2.08.001, BC-2.08.002, BC-2.08.003, BC-2.08.004, BC-2.08.005, BC-2.08.006, BC-2.08.007,
BC-2.08.008, BC-2.09.001, HS-EXP-014. §Trace and historical entries in BC-2.03.007 §Trace
(lines 150, 162, 167 — historical record of v1.8.0 introduction) deliberately NOT updated.
No code-delivery/ dirs staged.

Registry atomicity (L-S027-004): 1 entry bumped atomically: SS-session-manager→1.8.1.
Registry diff = exactly 1 changed version line. Spot-checked frontmatter: SS-session-manager
reads v1.8.1. Count-propagation sweep: POL-11 confirmed 0 stale "SS-session-manager v1.8.0"
active literals after sweep (was 27, now 0). All survivors are in historical §Trace context.

POL-11 PASS (319 active). POL-12 PASS. Pass-14 non-clean (Important present) →
consecutive-clean counter = 0. NEXT = Pass-15 (clean-streak candidate 1-of-3).
STATE v7.06→v7.07.

## §Trace v7.05 (D-254 — Adv Pass-12 convergence-fix COMMITTED; Pass-13 next)

**D-254 (2026-06-04):** Adversarial Pass-12 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 0 Critical / 2 Important / 2 Suggestions. SIXTH consecutive zero-Critical pass
(C:...,0,0,0,0,0,0).

I12-001 (Important): EngineError→ServerToClient::Error taxonomy gap. BinaryNotFound and
InvalidPath errors were collapsed to generic `spawn_failed` code. BC-2.03.007 listed error
message banners that could not be satisfied by the taxonomy. Architect fixed:
- SS-ipc v1.16.0→v1.17.0: taxonomy 8→10 codes (`binary_not_found`, `invalid_spawn_arg`
  added); §Scope multi-client future-scope cross-ref to TD-MULTI-CLIENT-ATTACH-STORM-001.
- SS-session-manager v1.7.2→v1.8.0: `SessionError::EngineError(#[from])` bridge variant
  added; `session_error_to_code` arm for EngineError routing; exhaustiveness preserved.
- SS-daemon-wiring-v2-delta v1.6.0→v1.7.0: `spawn_recipe()` call-site annotation pinned
  in `spawn_session()` first-step in normative summary table.
- BC-2.03.007 v1.0.0→v1.1.0: PC-3/PC-7 error codes reconciled to canonical taxonomy;
  inputs frontmatter added.

I12-002 (Important): BC-2.03.005 Description partial-fix regression — cwd field still
cited `project_root` after Pass-2 I2-002 fix. Description corrected to `worktree_root`.
BC-2.03.005 v1.1.0→v1.1.1.

S12-001 (Suggestion): SS-embedded-pty App struct missing `dump_in_progress` and
`pending_pty_bytes` fields in the normative struct definition (fields used by snapshot-
then-resume protocol per ADR-0010 but not declared in the §App-struct table). Skeleton
fix: fields added with types + documentation. SS-embedded-pty v1.4.0→v1.5.0.

S12-002 (Suggestion): SS-ipc §Scope missing cross-reference to TD-MULTI-CLIENT-ATTACH-
STORM-001 deferral anchor — folded into the SS-ipc v1.17.0 bump (I12-001).

Propagation sweep (S-7.02): 35 unique files updated to new SS versions (POL-11-driven).
Files span BC-2.03.*, BC-2.05.*, BC-2.06.023/025, BC-2.08.*, BC-2.09.*, HS-EXP-014,
plus the 3 SS files themselves.

Registry atomicity (L-S027-004): 6 entries bumped atomically: SS-ipc→1.17.0,
SS-session-manager→1.8.0, SS-daemon-wiring-v2-delta→1.7.0, SS-embedded-pty→1.5.0,
BC-2.03.005→1.1.1, BC-2.03.007→1.1.0. Registry diff = exactly 6 changed version lines.
Spot-checked frontmatter: all 6 confirmed matching. No code-delivery/ dirs staged.

POL-11 PASS (319 active). POL-12 PASS. Pass-12 non-clean (Important present) →
consecutive-clean counter = 0. NEXT = Pass-13 (clean-streak candidate 1-of-3).
STATE v7.04→v7.05.

## §Trace v7.04 (D-253 — Adv Pass-11 convergence-fix COMMITTED; Pass-12 next)

**D-253 (2026-06-04):** Adversarial Pass-11 convergence-fix COMMITTED. 0C/1I/4S. FIFTH
consecutive zero-Critical. I11-001 PRONG A: auto-attach mandate (SS-embedded-pty 1.4.0 +
BC-2.09.001 1.3.0). PRONG B: RATIFIED-FUTURE scope boundary (SS-daemon-wiring-v2-delta
1.6.0 + TD-MULTI-CLIENT-ATTACH-STORM-001). S11-001: §Trace phantom kill_deadline_reason
fix (SS-session-manager 1.7.2). SS-ipc doc-comment extended 1.16.0. 35-file propagation
sweep. 5 registry entries bumped. POL-11 PASS (314 active). POL-12 PASS. STATE v7.03→v7.04.

## §Trace v7.03 (D-252 — Adv Pass-10 convergence-fix COMMITTED; Pass-11 next)

**D-252 (2026-06-04):** Adversarial Pass-10 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 0 Critical / 1 Important / 4 Suggestions — Important + 3 answerable Suggestions
fixed in-scope; S10-004 deferred to Phase-2 story-writer.

I10-001 (Important): SS-session-manager §Screen-state-transfer step 5a and §PTY-reader-thread
step 4 used the wording "re-attach command" without naming the specific IPC variant. Architect
corrected both occurrences to `ClientToServer::AttachSession` — the canonical TUI-to-daemon
attach message per SS-ipc v1.15.0. SS-session-manager v1.7.0→v1.7.1.

S10-001 (Suggestion): SS-deps-pin-manifest-v2-delta registry-action block gained a historical
annotation clarifying it records the initial pin state, not a recurrence. No version bump.

S10-002 (Suggestion): BC-2.05.010 success postconditions were silent on the ordering of
SessionStateChanged and SessionListUpdate emissions. Ordered sequence now named explicitly
in the POs. BC-2.05.010 v1.4.0→v1.5.0.

S10-003 (Suggestion): ADR-0009 contained a doubled-word typo. Fixed inline. No version bump.

S10-004 (Suggestion, deferred): S-TBD anchor resolution in 25 new v1A BCs and holdout
`stories_tested` fields deferred to Phase-2 story-writer per design intent — anchors are
correctly S-TBD until story decomposition assigns story IDs.

Closure grep CLEAN: BC-2.05.011 line 118 `daemon→host DaemonToHost::Attach` is the correct
direction for the host-to-daemon reply path, not a ClientToServer::AttachSession survivor.

Stale-literal propagation sweep (Defensive Sweep S-7.02): 13 BC files + HS-EXP-014 updated
SS-session-manager v1.7.0→v1.7.1 across all normative Architecture Source rows.
Files updated: BC-2.03.005, BC-2.05.009, BC-2.05.011, BC-2.06.025, BC-2.08.001-008,
BC-2.09.001, HS-EXP-014 (normative lines + historical anchors time-qualified).
§Trace and historical entries in HS-EXP-014 annotated with "at Pass-9 authoring time" per
ADR-0007 §Historical Anchor Classification.

Registry atomicity (L-S027-004): 2 entries bumped atomically: SS-session-manager→1.7.1,
BC-2.05.010→1.5.0. Count-propagation sweep: registry diff = exactly 2 changed version
lines; spot-checked frontmatter (SS-session-manager reads v1.7.1 ✓; BC-2.05.010 reads
v1.5.0 ✓). FOURTH consecutive zero-Critical (C:5→5→4→1→2→2→0→0→0→0;
I:8→6→9→4→4→2→4→2→1→1). Pass-10 non-clean (Important present) → consecutive-clean
counter remains 0; need 3. POL-11 PASS (313 active). POL-12 PASS. STATE v7.02→v7.03.

## §Trace v7.02 (D-251 — Durable resume checkpoint written; Pass-10 next)

**D-251 (2026-06-03):** Durable zero-context resume checkpoint written at human request.
STATE.md `next_session_resume_protocol` rewritten to v7.02 with full convergence-loop
procedure, complete spec package version list, ratified decisions, remaining tasks, and
codified lessons. NEXT-SESSION-RESUME.md created on develop as the "read first" entry
point. NEXT-SESSION-PIVOT.md updated with a D-250 status banner. Convergence paused at
Pass-9-done / consecutive-clean counter = 0 / Pass-10 is next. STATE v7.01→v7.02.

## §Trace v7.01 (D-250 — Adv Pass-9 convergence-fix COMMITTED; Pass-10 next)

**D-250 (2026-06-03):** Adversarial Pass-9 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 0 Critical / 1 Important / 1 Suggestion — ALL resolved. 3rd consecutive
zero-Critical pass (C:5→5→4→1→2→2→0→0→0; I:8→6→9→4→4→2→4→2→1).

I-P9-001: ADR-0010 §IPC-Message-Type-Additions ScrollbackChunk and ScrollbackDumpComplete
variant doc-comments carried the RETIRED pause-during-dump model — the final live normative
I3-003 propagation residue. I-P7-004 (Pass-7) correctly fixed SS-session-manager lines ~717
and ~843 ("IMMEDIATELY resumes") but did not inspect ADR-0010's own variant doc-comments.
ScrollbackChunk doc-comment rewrote "TUI MUST NOT begin rendering PTY bytes until
ScrollbackDumpComplete" to the snapshot-then-resume protocol with pending_pty_bytes buffer
and cross-ref to §Interleaving. ScrollbackDumpComplete doc-comment added the three-step
TUI procedure: (1) reset parser from snapshot rows; (2) replay pending_pty_bytes through
freshly-reset parser in receipt order; (3) clear buffer and set dump_in_progress=false;
plus explicit "session-host NOT paused during the dump" note. Comprehensive sweep of all
architecture files confirmed this was the sole normative survivor.

S-P9-001: HS-EXP-014 FAIL criterion and C2-004 modification note referenced "schema v2
per SS-session-manager v1.7.0" — corrected to "schema v3 (schema_version: 3) per
SS-session-manager v1.7.0" (cosmetic; behavioral assertion unchanged in both v2 and v3).
EVAL-INDEX v1.14→v1.15. Stale-literal sweep: BC-2.05.009 and BC-2.05.011 Architecture
Source rows updated ADR-0010 v1.4.0→v1.5.0 (2 active literals). BC-2.05.011 §Trace v1.1.0
line 219 is a historical anchor per ADR-0007 — correctly exempt. Registry: ADR-0010→1.5.0
and EVAL-INDEX→1.15 bumped atomically (L-S027-004). POL-11 PASS. POL-12 PASS.

Adversary confirmed: anchor-resolution PASS, security PASS, no design gaps; expects
Pass-10 CLEAN. consecutive-clean counter remains 0; need 3.

## §Trace v7.00 (D-249 — Adv Pass-8 convergence-fix COMMITTED; Pass-9 next)

**D-249 (2026-06-03):** Adversarial Pass-8 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 0 Critical / 2 Important / 1 Suggestion — ALL resolved. 2nd consecutive
zero-Critical pass (C:5→5→4→1→2→2→0→0).

Both Important findings were partial-fix propagation failures: Pass-7 introduced the
op-aware session_error_to_code(IpcOp,&SessionError) pattern in SS-session-manager and
SS-ipc, but SS-daemon-wiring-v2-delta §3 (the inline handler copy) was not updated
in the same burst — creating a delta-vs-canonical drift (2nd occurrence; cf. I6-002
which retired the stale inline SessionSnapshot in §4).

I-P8-001: §3 KillSession and AttachSession arms still called single-arg session_error_to_code
(pre-op-aware signature). Architect propagated op-aware session_error_to_code(IpcOp::Kill, &e)
and session_error_to_code(IpcOp::Attach, &e). Architect also caught SpawnSession hardcoding
the literal string "spawn_failed" instead of calling session_error_to_code(IpcOp::Spawn, &e).
All three spawn/kill/attach arms now use the full op-aware signature.

I-P8-002: §3 KeyInput, DetachSession, and RenameSession arms used bare ?-propagation on
SessionError — this silently drops the connection rather than sending ServerToClient::Error
to the client. Fixed: all three now send ServerToClient::Error via the canonical pattern.
ResizePane was using bare ? as well; since ResizePane is not a session error (it is a
TUI-side PTY instruction), it maps to WARN-continue rather than Error routing.

STRUCTURAL: A canonical-pattern-lock note was added to §3 of SS-daemon-wiring-v2-delta
to document the required pattern for every IPC handler arm and prevent future drift
between the canonical spec and the delta copy.

S-P8-001: HS-EXP-015 §Coverage note listed 5 input classes; ResizePane was not counted
as a distinct input class from the other keyboard variants. Updated to 6 input classes
with ResizePane enumerated explicitly. EVAL-INDEX v1.13→v1.14 (modified timestamp bump).

SS-daemon-wiring-v2-delta v1.4.0→v1.5.0. EVAL-INDEX v1.13→v1.14.

Registry atomicity (L-S027-004): 2 registry entries bumped in same atomic commit:
SS-daemon-wiring-v2-delta→1.5.0, EVAL-INDEX→1.14.

Stale-literal propagation sweep (Defensive Sweep S-7.02): 14 active SS-daemon-wiring-v2-delta
v1.4.0 literals updated to v1.5.0 across 9 files:
BC-2.05.009 (2), BC-2.05.010 (1), BC-2.05.011 (1),
BC-2.08.001 (2), BC-2.08.003 (2), BC-2.08.004 (1),
BC-2.08.007 (1), BC-2.08.008 (3), SS-ipc (1).
No other active stale literals found across .factory/specs.

POL-11 PASS. POL-12 PASS. Pass-8 was non-clean (Important present);
consecutive-clean counter remains 0; need 3. NEXT = adversarial Pass-9.
v6.99→v7.00.

## §Trace v6.98 (D-247 — Adv Pass-6 convergence-fix COMMITTED; Pass-7 next)

**D-247 (2026-06-03):** Adversarial Pass-6 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 2 Critical / 2 Important — ALL resolved.

Findings were dangling-reference residue EXPOSED by Pass-5's SS-ipc census consolidation
(which first established SS-ipc as the sole enum authority). Pass-6 adversary probed the
newly-complete enum and found that BC-2.05.010 §PC-1 requires ServerToClient::Error for
IPC-level error reporting, but no such variant existed in SS-ipc.md v1.13.0. Silent-failure
on spawn-launch would have been the runtime consequence.

C6-001: ServerToClient::Error variant missing (required by BC-2.05.010 §PC-1 IPC error
reporting obligation). Added as 13th ServerToClient variant in SS-ipc v1.13.0→v1.14.0 with
structured payload {code: ErrorCode, message: String} and a 6-code taxonomy:
spawn_failed, session_not_found, attach_failed, kill_failed, rename_failed, invalid_request.
SS-session-manager v1.5.0→v1.6.0 gained a new §Error handling section specifying the
SessionError→ServerToClient::Error mapping table (one entry per error variant), the
no-Ok()-at-task-boundary-on-Err invariant (task handler must map SessionError and send the
Error variant before returning; no silent discard), and special rules for KeyInput/ResizePane
(not session errors — these map to session_not_found if session is gone, InvalidRequest if
payload malformed). SS-daemon-wiring-v2-delta v1.3.1→v1.4.0: error routing wired into the
SpawnSession, KillSession, and AttachSession IPC handler arms (Err(e) → send ServerToClient::
Error; no bare Ok(()) on failure path).

C6-002: ClientToServer::ReAttach arm in daemon-wiring §IPC handler pattern table was an
uncompilable dead reference — this variant was removed from ClientToServer enum in I3-004
(D-244) when AttachSession superseded it, but the handler arm survived in SS-daemon-wiring-
v2-delta. Removed from SS-daemon-wiring-v2-delta v1.4.0.

I6-001: HS-EXP-015 §Steps referenced ClientToServer::MouseInput, which does not exist — the
canonical keyboard variant is ClientToServer::KeyInput (per SS-ipc.md v1.14.0 §ClientToServer).
MouseInput was never a variant; this was a typo introduced at holdout authoring. Fixed in
HS-EXP-015. EVAL-INDEX v1.12→v1.13 (modified timestamp bump).

I6-002: SS-daemon-wiring-v2-delta §4 "Supporting Types" contained an inline partial definition
of SessionSnapshot that was stale after C3-004 (D-244) moved SessionSnapshot to SS-ipc.md
§Supporting Types as the wire boundary type. The inline definition was retired; §4 now points
to SS-ipc.md §Supporting Types as the canonical location.

Registry atomicity (L-S027-004): 5 registry entries bumped atomically in same commit:
SS-ipc→1.14.0, SS-session-manager→1.6.0, SS-daemon-wiring-v2-delta→1.4.0,
BC-2.05.010→1.3.0, EVAL-INDEX→1.13. Diff-count verified: 5 current_version lines changed
in version-pin-registry.yaml — exact match to claimed 5.

Stale-literal propagation sweep (Defensive Sweep S-7.02): 3 active SS-session-manager v1.5.0
literals in HS-EXP-014.md updated to v1.6.0 (lines 38, 100, 113 — normative §Steps/FAIL
criteria cites). No other active stale literals found across .factory/specs.

POL-11 PASS. POL-12 PASS. Pass-6 was non-clean; convergence still needs 3 consecutive CLEAN.
v6.97→v6.98.

## §Trace v6.97 (D-246 — Adv Pass-5 convergence-fix COMMITTED; Pass-6 next)

**D-246 (2026-06-03):** Adversarial Pass-5 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 2 Critical / 4 Important — ALL resolved.

NOVEL STRUCTURAL FINDING (C5-001 — survived 4 passes): SS-ipc.md is the ARCH-INDEX-designated
IPC wire authority and is cited in Architecture Source rows of all SS-05 BCs. Despite this
designation, SS-ipc.md was MISSING 11 of the v1A wire variants defined in ADR-0010 +
SS-daemon-wiring-v2-delta: PtyOutput, SessionStateChanged, ScrollbackChunk,
ScrollbackDumpComplete, PtyReset (ServerToClient), KeyInput, ResizePane, SpawnSession,
KillSession, DetachSession, RenameSession (ClientToServer). These variants existed only in
ADR-0010 §Protocol detail tables and SS-daemon-wiring-v2-delta §broker fan-out. This created
a structural gap where the cited authority document did not actually contain the symbols it
was cited as the authority for. FIXED: all 11 variants folded into SS-ipc.md v1.13.0 enum
bodies. ServerToClient now has 12 canonical variants (original 1 + ScrollbackChunk +
ScrollbackDumpComplete + PtyReset + PtyOutput + SessionStateChanged + existing
PermissionPromptPayload/PermissionPromptResolved/InitialState/SessionListUpdate +
1 existing) and ClientToServer has 9 (KeyInput + ResizePane + SpawnSession + KillSession +
DetachSession + RenameSession + existing AttachSession/DetachSession/PermissionDecision
variants). Orchestrator verified all 15 symbols cited in active BC Architecture Source rows
are now present in SS-ipc.md. SS-ipc v1.12.1→v1.13.0.

C5-002: SerializedCell and SerializedColor types were defined in SS-session-manager §5 but
belong in monocle-ipc (the monocle-ipc crate is the wire boundary crate — same class of
finding as C3-004 SessionSnapshot). Moved to SS-ipc.md v1.13.0 §Supporting Types.
SS-session-manager §5 updated to reference monocle-ipc crate types. SS-session-manager
v1.4.1→v1.5.0.

I5-001: BC-2.05.009 §PC-2 and §EC-270 had a self-contradiction — Invariant 1 (§INV-1)
requires no-drop semantics (backpressure-and-wait), but PC-2 said "drop oldest and continue"
for the PTY output channel. Fixed: PC-2 rewritten to backpressure-no-drop with `.send().await`
semantics; EC-270 and the test vector rewritten to match Invariants 1 and 3. BC-2.05.009
v1.4.0→v1.5.0.

I5-002: ADR-0010 §Throughput sizing section still cited "typical 4096 bytes per message" from
the original §Transport analysis, but the MAX_MESSAGE_BYTES constant is 256 KiB — a 64×
discrepancy. Fixed: typical-vs-max per-message sizing clarified (typical PTY chunks 512B–4KB,
max = 256 KiB for scrollback transfer); the 4096-byte assumption explicitly retired.
ADR-0010 v1.3.1→v1.4.0.

I5-003: BC-2.09.003 §PC-3 contained a stale forward-instruction from SS-embedded-pty v1.2.0
referencing a pane_area field by its old name. The field in BC-2.09.003 was already correct
(pane_area). The stale instruction in SS-embedded-pty v1.2.0 was a cross-reference note that
was never converted to a settled citation after v1.0 → v1.2.0 revision. Converted to a
settled cross-reference. SS-embedded-pty v1.2.0→v1.3.0.

I5-004: SS-embedded-pty §Scrollback navigation used pty_scroll_offset (singular) in two
prose instances where the canonical struct field is pty_scroll_offsets[focused_session_id]
(per App struct §per-session scroll offsets, I7-fix from D-242). Both prose instances updated.
Folded into SS-embedded-pty v1.3.0 with I5-003.

PROCESS-GAP CODIFIED (anchor-resolution-closure check): C5-001 would have been caught at
architecture delta time (D-239) if a post-authoring check had verified that every symbol
cited in BC Architecture Source rows EXISTS as a named item in the cited document section.
Codified: after any SS or ADR version bump, run anchor-resolution-closure check — grep for
cited symbol names in the Architecture Source section of each BC that cites the bumped doc;
fail if any cited symbol is absent from the claimed §section.

Registry atomicity (L-S027-004): 5 registry entries bumped atomically in same commit:
SS-ipc→1.13.0, SS-session-manager→1.5.0, SS-embedded-pty→1.3.0, ADR-0010→1.4.0,
BC-2.05.009→1.5.0. Diff-count verified: 5 current_version lines changed in registry —
exact match to claimed 5.

Stale-literal propagation sweep (Defensive Sweep S-7.02): 68 stale active literals across
32 BC files + HS-EXP-014.md fixed before commit (SS-ipc v1.12.1→v1.13.0 in BC-2.05.001-011
+ BC-2.06.023/025 + BC-2.08.001-008 + BC-2.09.001-009; SS-session-manager v1.4.1→v1.5.0 in
BC-2.03.005 + BC-2.05.009/011 + BC-2.08.001-007 + HS-EXP-014; SS-embedded-pty v1.2.0→v1.3.0
in BC-2.09.007/008/009 + HS-EXP-014; ADR-0010 v1.3.1→v1.4.0 in BC-2.05.011). Registry
internal historical cite (BC-2.09.003 last_bump_commit field) annotated with
`# version-pin-historical` marker to suppress POL-11 false-positive.

POL-11 PASS: 310 active, 3789 historical anchors, 631 files scanned. POL-12 PASS.
0 consecutive clean passes (Pass-5 was non-clean); need 3 for human gate. v6.96→v6.97.

## §Trace v6.96 (D-245 — Adv Pass-4 convergence-fix COMMITTED; Pass-5 next)

**D-245 (2026-06-03):** Adversarial Pass-4 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 1 Critical / 4 Important / 3 Suggestions — ALL resolved. Novelty DECAYING
(C:5→5→4→1, I:8→6→9→4). Adversary confirmed NO new architectural/security gaps — all
findings were propagation residue from two Pass-3 fixes (ScrollbackDump→chunked retirement
and per-client buffer 256→64 correction).

CRIT-001: BC-2.09.001 Invariant 5 + Related-BCs [BC-2.08.007] still cited the nonexistent
ServerToClient::ScrollbackDump variant. Fixed to chunked ScrollbackChunk*/ScrollbackDumpComplete
protocol with pending_pty_bytes obligation (per BC-2.05.011 Invariant 6). BC-2.09.001
v1.1.0→v1.2.0.

HIGH-001: BC-2.08.002 PC-4/PC-7/Invariant 4 still used the retired single-message
HostToDaemon::ScrollbackDump path. Fixed to chunked protocol; cross-references updated.
BC-2.08.002 v1.1.0→v1.2.0.

HIGH-002: BC-2.08.007 title and Description still said "GC/orphan reaping" but the Pass-3
fix had changed the spec body; title+Description propagated to PRD §2.8/§2.9 capability
descriptions and BC-INDEX changelog. BC-2.08.007 v1.3.0→v1.4.0. PRD v1.28.0→v1.28.1.
BC-INDEX v1.37→v1.38.

HIGH-003: BC-2.05.009 PC-1b still said "capacity 256" — the last live normative survivor
of the I2-003 → O3-004 buffer correction. Fixed to 64. BC-2.05.009 v1.3.0→v1.4.0.

HIGH-004: ADR-0010 line 310 and the PO-edit-block in SS-daemon-wiring-v2-delta §5d both
still said CLIENT_SEND_BUFFER_SIZE=256 — this was the source that kept re-injecting the
256 value into the propagation loop across passes. Both retired to 64. ADR-0010 v1.3.0→v1.3.1.
SS-daemon-wiring-v2-delta v1.3.0→v1.3.1.

SUG-001: vt100 0.16 reconstruction path — Parser::process verified as the only correct
path; no set_screen method exists in the public API. Bytes path canonicalized in relevant
BC. Verified correct.

SUG-002: spawned_by_monocle field type — EnrichedSession→SessionSnapshot wire boundary
type correction (SS-session-manager §5 + BC-2.08.006 PC-5). SS-session-manager v1.4.0→v1.4.1.
BC-2.08.006 v1.1.0→v1.2.0.

SUG-003 (CYCLE CHECKLIST CODIFIED): whole-tree grep gate applied — orchestrator independently
verified ZERO live normative ScrollbackDump/256 survivors across .factory/specs. Every
surviving occurrence confirmed to be in §Trace/changelog/enforcement context, NOT a live
PC/Invariant/title. Codified as mandatory cycle checklist step: after any RETIRED-term or
corrected-magic-number fix, run grep -rn across .factory/specs before next adversarial pass
and confirm no live normative survivors.

Registry atomicity (L-S027-004): 10 registry entries bumped atomically in same commit.
Diff-count verified: 10 current_version lines changed in registry, 10 claimed — exact match.
Spot-check: ADR-0010 file v1.3.1 matches registry v1.3.1; BC-INDEX file v1.38 matches
registry v1.38; PRD file v1.28.1 matches registry v1.28.1; SS-session-manager file v1.4.1
matches registry v1.4.1. All 4 spot-checks PASS.

POL-11 PASS (309 active, 0 new stale literals — SUG-003 grep gate confirmed no survivors).
POL-12 PASS. 0 consecutive clean passes (Pass-4 was last non-clean); need 3 for human gate.
v6.95→v6.96.

## §Trace v6.95 (D-244 — Adv Pass-3 convergence-fix COMMITTED; Pass-4 next)

**D-244 (2026-06-03):** Adversarial Pass-3 (control-center spec package) convergence-fix
COMMITTED to factory-artifacts (single atomic commit per TD-VSDD-053 Single-Commit Burst
Protocol). 4 Critical / 9 Important / 4 Observations — ALL resolved.

Architect fixes: C3-001 SessionStateChanged emission point per-method rule — SS-daemon-wiring-v2-delta
(v1.2.0→v1.3.0) §3b adds per-method emit table (spawn, kill, rename, state-change, re-discovery,
GC) and SS-session-manager (v1.3.0→v1.4.0) §3b mirrors it. C3-003 rename() emits
SessionListUpdate (NOT SessionStateChanged) only — both specs updated. C3-004 SessionSnapshot
defined as the canonical wire boundary type in SS-ipc (v1.12.0→v1.12.1) §SessionSnapshot §2a —
resolves the SessionEntry/EnrichedSession/SessionSnapshot tri-naming confusion; ADR-0010
propagates this. I3-001 ordering guarantee = per-client channel FIFO (channels are ordered by
construction) + split-pair→disconnect on FifoViolation — no mutex needed. I3-002 Terminating
watchdog is a post-bind background task (NOT daemon startup task) — SS-session-manager §3d;
kill_deadline persisted in session-state.json schema_version 3. I3-003 dump protocol:
snapshot-then-resume (session-host snapshots current screen, sends ScrollbackChunk*, resumes
PtyBytes immediately — no harness stall). I3-004 ClientToServer::AttachSession variant
added to SS-ipc §3b + SS-daemon-wiring §3 handler; TUI sends this to trigger scrollback dump
(MUST NOT send DaemonToHost::Attach directly). I3-005 Detached state preserved on session
re-discovery (not reset to Running). I3-008 SerializedCell.attrs verified against vt100 0.16
source: 5 bool flags (bold, italic, underline, strikethrough, blink=never-set) — spec matches
implementation. I3-009 degraded-env (non-UTF8 hostname etc) surfaced to daemon in
session-state.json; daemon logs WARN; TUI shows [degraded] badge. O3-004 per-client send
buffer size corrected to 64 (not 256) in SS-daemon-wiring-v2-delta §5d constant
CLIENT_SEND_BUFFER_SIZE=64 — aligns SS-ipc + BC-2.05.011 + BC-2.08.008. ADR-0010 (v1.2.0→v1.3.0):
I3-001 ordered-pair FIFO guarantee; I3-003 snapshot-then-resume interleaving protocol replaces
prior; O3-004 buffer constant corrected.

Product-owner fixes: C3-002 schema_version 3 consistent across BC-2.08.004 (v1.1.0→v1.2.0)
and BC-2.08.003 (v1.2.0→v1.3.0) — self-deleting-sidecar bug in sidecar-content closed.
SessionStateChanged emission BCs updated: BC-2.08.007 (v1.2.0→v1.3.0) + BC-2.08.008
(v1.0.0→v1.1.0) per-method emit table mirrors architect spec. AttachSession BCs: BC-2.08.001
(v1.2.0→v1.3.0) session-lifecycle + BC-2.05.011 (v1.0.0→v1.2.0, inc F-02 buffer fix) TUI
variant. Detached/Terminating sync: BC-2.05.009 (v1.2.0→v1.3.0) + BC-2.05.010 (v1.1.0→v1.2.0)
Detached re-discovery preserved. I3-006 SO_PEERCRED universal: BC-2.06.025 (v1.2.0→v1.3.0)
permission UDS sub-channel verifies UID. O3-001 dup PC: BC-2.08.003 §PC table deduplicated.

PROPAGATION-CLOSURE AUDIT (consistency-validator, pre-commit): ran targeted sweep after
architect+PO fix round. Found 2 prose-only residues before the adversary saw them: F-01
SS-ipc §Connection Lifecycle still had Vec<EnrichedSession> prose (SessionSnapshot was the
correct term post-C3-004); F-02 BC-2.05.011 Invariant 5 said buffer 256 (was I2-003 value)
not 64 (I3-003/O3-004 corrected value). Both fixed pre-commit. PROCESS WIN: running this
closure audit before each adversarial pass breaks the propagation-leak cycle — codified in
`awaiting` and as a new SE candidate entry.

Registry atomicity (L-S027-004 recurrence): 13 claimed registry bumps. Prior agents (architect
+ product-owner) updated the BC/SS files but did NOT update version-pin-registry.yaml for 9
of 13 entries. State-manager caught all 9 mismatches via targeted python verify before commit
(BC-2.08.001/003/004/007/008 + BC-2.05.009/010/011 + BC-2.06.025 all at old versions in
registry). All 9 fixed. Diff-count verified: 13 total registry bumps confirmed by inspection.

POL-11 sweep pre-commit: 39 stale active literals detected and fixed across 21 files —
SS-ipc v1.11.0→v1.12.1 in BC-2.05.001-008 + BC-2.06.023 (9 files);
SS-ipc v1.12.0→v1.12.1 in BC-2.05.009/010/011/BC-2.06.025 (4 files);
SS-session-manager v1.3.0→v1.4.0 in BC-2.03.005/BC-2.08.002/005/006/007/BC-2.09.001/
HS-EXP-014/BC-2.05.010/011 (9 files); registry internal historical cite patched with
time qualifier. POL-11 PASS: 309 active, 3781 historical, 631 files. POL-12 PASS.
0 consecutive clean passes; need 3 for human gate. v6.94→v6.95.

## §Trace v6.94 (D-243 — Adv Pass-2 convergence-fix COMMITTED; Pass-3 next)

**D-243 (2026-06-03):** Adversarial Pass-2 convergence-fix COMMITTED to factory-artifacts
(single atomic commit per TD-VSDD-053 Single-Commit Burst Protocol). 5 Critical / 6
Important / 5 Suggestions — ALL resolved. Architect fixes: C2-002 chunked wire variants
(ScrollbackChunk/ScrollbackDumpComplete/PtyReset added as ServerToClient enum arms; daemon
broker buffers then applies; ADR-0010 v1.1.0→v1.2.0). I2-003 per-client isolated send
buffers cap 256 added to §5d of SS-daemon-wiring-v2-delta (v1.1.0→v1.2.0) so backpressure
derives from durable ring rather than client-socket latency. I2-002 worktree-per-session
operationalized: SpawnOptions.worktree_root field added to SS-engine-module-v2-delta
(v1.0.1→v1.1.0); SS-session-manager (v1.2.0→v1.3.0) §spawn_session() uses worktree_root
as cwd (not project_root — the cwd=project_root bug is fixed). I2-004 Terminating transient
state added to SessionState enum + 12s watchdog kills if SIGTERM not handled. C2-005
SO_PEERCRED uid check extended to all per-session UDS connects including kill-path (not only
keyboard passthrough); 2s/10s escalation timeline reconciled across SS-session-manager +
BC-2.08.003 (v1.1.0→v1.2.0). I2-006 env inheritance: SS-embedded-pty (v1.1.0→v1.2.0)
§session-host-process-model specifies PATH/HOME/TERM/COLORTERM inheritance from daemon env.
C2-003 stale removal-directive reconciled in SS-session-manager GC policy section. S2-002
duplicate is_kitty_enhanced_key arm removed from SS-embedded-pty translation table; precedence
documented. S2-003 scrollback memory bound (BC-2.09.006 v1.0.0→v1.1.0). Product-owner
fixes: C2-001 per-prompt-bell rule made consistent across BC-2.06.025 (v1.1.0→v1.2.0) and
HS-EXP-014 (phantom sidecar field removed, C2-004). I2-001 mouse-capture scoped BC: explicit
BC-2.05.009 (v1.1.0→v1.2.0) and BC-2.06.025 rule that mouse capture MUST NOT be re-enabled
by bell/badge events. I2-005 Launching state: BC-2.05.010 (v1.0.0→v1.1.0) + BC-2.03.005/006
(v1.0.0→v1.1.0) updated — re-discovery probe triggered when EmbeddedTerminal opens and
session is in Launching state. S2-004 zero-dim resize clamp: BC-2.09.009 (v1.0.0→v1.1.0).
S2-005 SessionStateChanged event emission: NEW BC-2.08.008 (v1.0.0) + BC-2.08.007
(v1.1.0→v1.2.0) updated. NEW BC-2.05.011 (v1.0.0): ServerToClient variant inventory BC.
BC-INDEX v1.36→v1.37 (136→138 BCs). EVAL-INDEX v1.11→v1.12 (BC-INDEX input pin updated).
22 registry entries updated atomically (21 doc version bumps + EVAL-INDEX bump). 5 architect
specs + 17 PO files = 22 spec files + version-pin-registry.yaml = 23 total files in commit.
POL-11 PASS: 31 stale active literals fixed across 24 files (SS-embedded-pty v1.1.0→v1.2.0
in 9 BC-2.09 files + BC-2.06.025; SS-session-manager v1.2.0→v1.3.0 in 8 files; SS-engine-
module-v2-delta v1.0.1→v1.1.0 in 4 BC-2.03 files + BC-2.08.006; SS-daemon-wiring-v2-delta
v1.1.0→v1.2.0 in BC-2.05.009/010; BC-INDEX v1.36→v1.37 in EVAL-INDEX + product-brief;
registry self-cite corrected). 289 active, 3750 historical, 631 files scanned. POL-12 PASS.
0 consecutive clean passes; need 3 for human gate. v6.93→v6.94.

## §Trace v6.92 (D-242 — Adv Pass-1 convergence-fix COMMITTED; Pass-2 next)

**D-242 (2026-06-03):** Adversarial Pass-1 convergence-fix COMMITTED to factory-artifacts
(single atomic commit per TD-VSDD-053 Single-Commit Burst Protocol). 5 Critical / 8
Important / 5 Observations — ALL resolved. Architect fixes: C2 lock-file/re-discovery
dual-readiness-signal clarified (lock-file write=step 8 in daemon_start_sequence; UDS
bind=step 10; foreground-caller vs TUI-client first-connection contracts are distinct).
C3+I8: PTY backpressure model changed from silent drop to `.send().await` (bounded blocking)
+ PtyReset recovery protocol on channel full + TUI-surfaced pty_reset_count indicator +
ADR-0010 head-of-line blocking mitigation (broker biased select, pty_drop_counter separate
from hook drop counter) + benchmark moved from informational to hard pre-story-wave gate.
C5: ScrollbackDump typed-cell serialization fixed from Vec<String> (no color/attrs, double-
apply bug) to Vec<Vec<SerializedCell>> with TUI parser-reset-before-apply protocol. I1:
full mouse/keyboard encoding implemented (SGR-1006, Alt/Meta, Shift+Tab, modified arrows
all specified in translation table). I3: mouse capture scoped to EmbeddedTerminal entry
(enable SGR-1006) / exit (disable) — global capture NOT enabled. I4: dead SessionStates
(Created/Killed) pruned from SessionState enum; re-discovery handles Launching edge case.
I5: SO_PEERCRED uid check + sidecar-trust model on per-session UDS (closes keystroke-
injection vector — daemon refuses attach if peer uid != daemon uid). I6: PID-based orphan
kill before socket bind (daemon_start_sequence checks for orphan session-host processes).
I7: per-session scroll offsets stored in DaemonState/App. Product-owner fixes: C1: SHARED
hooks-file model wins (BC-HOOK-010; BC-2.08.006 revised — hooks-settings.json is shared
per-runtimeDir written at daemon startup, NOT per-session; concurrent spawn safety via
identical-content last-write-wins). C4: holdout HS-EXP-011/012/013 data-model corrected
(flat session-<uuid>.json + state:Running; was incorrectly nested). HS-EXP-014 shared-file
model fixed to match BC-HOOK-010. O3/O5 formatting fixed. I2: BC status convention
confirmed (active+S-TBD filled at story decomposition; no immediate change needed).
Spec version bumps: SS-session-manager v1.0.1→v1.2.0, SS-embedded-pty v1.0.2→v1.1.0,
SS-daemon-wiring-v2-delta v1.0.1→v1.1.0, ADR-0010 v1.0.0→v1.1.0, ADR-0011 v1.0.0→v1.1.0,
BC-2.05.009/BC-2.08.001/BC-2.08.003/BC-2.08.006/BC-2.08.007/BC-2.09.001/BC-2.09.003 all
bumped to v1.1.0. BC-INDEX v1.35→v1.36 (holdout BC-2.08.006 clarification). EVAL-INDEX
v1.10→v1.11. 15 version-pin-registry entries updated atomically (L-S027-004). POL-11 PASS:
26 stale active literals fixed across 21 files (BC-2.09.002-009 SS-embedded-pty cites;
BC-2.08.001-007 + BC-2.03.005 + BC-2.05.009/010 + BC-2.06.025 SS-session-manager/
SS-daemon-wiring-v2-delta cites; ARCH-INDEX last_bump_commit historical qualifier;
product-brief.md 2× BC-INDEX v1.35→v1.36; EVAL-INDEX BC-INDEX pin). 268 active literals,
268 all current. POL-12 PASS. 2 new human-gate items: CC-TUITERM-WIP-SIGNOFF (tui-term
0.3.4 WIP-upstream risk-acceptance, ADR-0011 §O2) + CC-GLOBAL-MOUSE-CAPTURE (mouse capture
scope approval if future story needs clickable panels). v6.91→v6.92.

## §Trace v6.91 (D-241 — BC/PRD delta COMMITTED; adversarial review next)

**D-241 (2026-06-03):** v1A control-center BC/PRD/holdout delta COMMITTED to factory-artifacts.
23 new BCs authored: SS-03 extensions (BC-2.03.005 spawn-recipe validation, BC-2.03.006
hook auto-injection, BC-2.03.007 invalid-path error, BC-2.03.008 discovery re-probe);
SS-05 extensions (BC-2.05.009 SessionCreation wizard flow, BC-2.05.010 multi-project scope);
SS-06 extension (BC-2.06.025 permission badge+bell guarantee during EmbeddedTerminal
/SessionCreation); SS-08 NEW SessionManager (BC-2.08.001 spawn, BC-2.08.002 kill,
BC-2.08.003 SIGTERM-SIGKILL 10s escalation, BC-2.08.004 parallel re-discovery probing,
BC-2.08.005 state persistence across daemon restart, BC-2.08.006 per-session hooks file,
BC-2.08.007 GC/orphan reaping); SS-09 NEW EmbeddedPTY (BC-2.09.001 attach, BC-2.09.002
detach, BC-2.09.003 keyboard passthrough, BC-2.09.004 output rendering, BC-2.09.005 resize,
BC-2.09.006 reconnect after session-host restart, BC-2.09.007 AppMode entry/exit,
BC-2.09.008 scrollback, BC-2.09.009 permission overlay integration). BC-INDEX v1.34→v1.35
(113→136 BCs). PRD v1.27.4→v1.28.0 (§2.8 SessionManager + §2.9 EmbeddedPTY capabilities
added). EVAL-INDEX v1.9→v1.10 (5 new holdout scenarios HS-EXP-011..015, 24→29 scenarios).
Registry atomicity (L-S027-004): 26 entries updated atomically in same commit (23 new BC
entries at v1.0.0 + BC-INDEX/EVAL-INDEX/prd bumps). All 23 new BC input-hashes computed
and written. product-brief.md: 2 stale BC-INDEX v1.34 active citations updated to v1.35.
Defensive count-propagation sweep: STATE.md Key Tech Stack line updated (BC-INDEX v1.34/113
BCs → v1.35/136 BCs; PRD v1.27.4 → v1.28.0). POL-11 PASS (271 active, 629 files).
POL-12 PASS. HS-EXP holdout files NOT individually tracked in registry (no existing pattern
— only EVAL-INDEX is tracked). DEFERRED (architect task, non-blocking): VP authoring for
SS-08/SS-09 BCs (all 16 BCs cite VP-TBD; VP-DTU-001 pattern; formal-hardening scheduling).
OPEN (human ratification required): v1B overlay pre-emption behavior. v6.90→v6.91.

## §Trace v6.90 (D-240 — consistency gate PASSED; BC/PRD delta next)

**D-240 (2026-06-03):** Consistency gate PASSED (D-240). Fresh-context consistency-validator
found 12 findings (3 Critical/5 Important/4 Suggestion) on the control-center spec package.
Root cause: vision/brief lagged the v2.1 session-host model introduced by ADR-0009.
ALL 12 findings resolved. Architect fixes: IMP-2 (session_id:String canonical in
SS-session-manager + SS-engine-module-v2-delta), IMP-3 (daemon-wiring step 8b placement
in SS-daemon-wiring-v2-delta), IMP-5 (InvalidPath error variant added; BC-2.03.007
retitled in SS-session-manager), SUG-3 (production-grade permission badge+bell guarantee
during EmbeddedTerminal/SessionCreation, no silent suppression — added to SS-embedded-pty;
v1B pre-emption behavior flagged for human ratification before BC authoring), SUG-4
(token estimates added to ARCH-INDEX). Spec bumps: SS-session-manager v1.0.0→v1.0.1,
SS-embedded-pty v1.0.0→v1.0.2, SS-engine-module-v2-delta v1.0.0→v1.0.1,
SS-daemon-wiring-v2-delta v1.0.0→v1.0.1, ARCH-INDEX v1.0.27→v1.0.28.
product-owner fixes: CRIT-1/2/3 (v2.1 session-host model propagated to vision + brief),
IMP-1 (monocle-session-host binary referenced), SUG-1/2 (lifecycle corrections).
vision v2.1→v2.2, product-brief v2.0.0→v2.0.1. IMP-4 (stale registry product-brief
1.4.34→2.0.1) fixed in version-pin-registry.yaml. domain-monocle-vision-synthesis.md
NOT in registry (no entry to update). input-hashes refreshed: SS-session-manager 13e1215,
SS-embedded-pty 13e1215, ARCH-INDEX ac2d9a7, product-brief 13e1215 (SS-engine-module-v2-delta
and SS-daemon-wiring-v2-delta already current). POL-11 PASS, POL-12 PASS. Registry
atomicity: all 6 version-pin-registry entries updated atomically in this same commit
(L-S027-004). NEXT: BC/PRD delta (product-owner authors control-center BCs including
permission badge+bell guarantee BC) → Phase-1d adversarial convergence (3 clean passes)
on full spec package → human spec-package approval gate → story decomposition.
v6.89→v6.90.

**D-239 (2026-06-03):** Control-center architecture delta COMMITTED to factory-artifacts.
3 ADRs: ADR-0009 (native detached session-host process model — Q-8 resolved native;
monocle-session-host binary owns PTY masters + harness child processes; daemon re-attaches
over UDS on restart), ADR-0010 (PTY bytes shared on existing UDS IPC channel — Q-1 resolved
Option A), ADR-0011 (PTY stack: native portable-pty 0.9 + vt100 0.16 + tui-term 0.3.4 —
Q-7 fork posture: exact-pin, no fork needed). 5 SS deltas: SS-session-manager 1.0.0
(SS-08, CAP-008), SS-embedded-pty 1.0.0 (SS-09, CAP-009), SS-engine-module-v2-delta 1.0.0
(Q-2 resolved — spawn_recipe()), SS-daemon-wiring-v2-delta 1.0.0 (D-235 rework scope),
SS-deps-pin-manifest-v2-delta 1.0.0 (portable-pty/vt100/tui-term pins + monocle-session-host
crate). ARCH-INDEX bumped v1.0.26→v1.0.27. 8 version-pin-registry entries added atomically.
input-hashes computed per-artifact. POL-11 PASS (243 active), POL-12 PASS. DISPOSITION-V2
rollup + embedded-pty-evaluation supersession notes committed (SR-002 keyboard scope reconciled).
NEXT: consistency validation + adversarial review (3 clean passes) → human approval gate →
story decomposition (story-writer decomposes v1A delta into new waves). v6.88→v6.89.

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
