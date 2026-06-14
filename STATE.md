---
document_type: pipeline-state
level: ops
project: monocle
version: "7.46"
status: active
producer: state-manager
timestamp: 2026-06-14T23:30:00Z
phase: PIVOT-delta-in-progress
current_step: "D-296: Pass-53 FIX BURST — F-P53-001 (Important) BC-2.08.006 PC-3 authority corrected from BC-HOOK-007 (DTU gene-source clone) to BC-2.04.010 PC-3; hooks-settings.json file schema = 4 URL-bearing keys {PreToolUse,Notification,Stop,UserPromptSubmit} + 2 reserved-empty {PostToolUse:[],PreCompact:[]}; SessionStart NOT a file key (5 served endpoints vs 4-URL+2-empty file-JSON keys). Sibling closed in-scope: SS-daemon-wiring.md §SessionStart Invocation Path Note errata-no-bump. BC-2.08.006 v1.2.1→v1.3.0. No new wire codes/variants. POL-11 GREEN. POL-12 GREEN. CONSECUTIVE-CLEAN COUNTER = 0. Pass-54 next = clean candidate 1 of 3."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-047..D-174 archived at cycles/cycle-001/decisions-archive.md. D-175: Wave 4 gate PASSED. D-182: Wave 5 gate PASSED. D-183: Wave 6 AUTHORIZED. D-184: S-022 DELIVERED. D-185: S-023+S-025 AUTHORIZED. D-186: S-023 DELIVERED. D-188..D-221: see Decisions Log (archived in this file). D-222: S-025 DELIVERED (PR #28 @ 838477e). D-223: S-026 DELIVERED (PR #30 @ 9fb0d70) — Wave 6 COMPLETE. D-224: Wave-6 GATE PASSED (develop @ 2a51a91). D-225: STATE correction — phase-3-COMPLETE premature (Wave 7 of 7 remains); corrected to wave-7-READY; points 177→169/195 reconciled to sprint-state. D-226: S-027 DELIVERED (PR #32 @ 3787ebd) — Wave 7: 1/4 done. D-227: S-031 DELIVERED (PR #33 @ 8451486). D-228: S-028 DELIVERED (PR #34 @ 682e5e5) — Wave 7: 3/4 done. develop @ 682e5e5. D-229: Zero-context resume checkpoint — S-029 human-authorized 2026-06-02. develop @ 1158e24. D-230: S-029 DELIVERED (PR #35 @ 48463fb) — Wave 7 COMPLETE (4/4). 32/33 done (192/195 pts). develop @ 48463fb. D-231: Wave-7-gate prerequisite sweep complete — SS-ipc v1.11.0, BC-2.06.021 v1.0.7, BC-INDEX v1.34, STORY-INDEX v5.30, citation atomicity propagation. POL-11/POL-12 PASS. S-027/S-028 story frontmatter status fixed. D-232: Wave-7 gate PASSED — Phase 3 COMPLETE. 1514 tests, 0 failures. F-W7G3-MED-001 fixed (PR #37 @ 6811103). HS-EXP-008 score 1.0. DTU SKIP (DTU-CLONE-STORY added as Phase 4 prereq). sprint-state v1.38. D-233: Phase-3→4 consistency cleanup — EVAL-INDEX v1.9, STORY-INDEX v5.31, BC-HOOK-034 v1.0.2 (typo fix), sprint-state v1.39. 28 story-file status fields corrected. Input-hash refresh (113→0+17 bookkeeping-residual). POL-11/POL-12 PASS. All MED/LOW audit findings RESOLVED. D-234: DTU clone false-negative corrected — S-DTU-001 (cargo binary dtu-claude-code-hooks-v1) validated fidelity 1.0000 (25/25 fixtures). Gate-2 DTU-VALIDATION corrected from SKIP to PASS. DTU-CLONE-STORY closed (RESOLVED-FALSE-PREMISE). Phase 4 UNBLOCKED. dtu_clones_built updated. PROC-DTU-VALIDATE-LOCATION process gap added. D-235: Daemon-wiring convergence — monocle-runtime binary now serves (main() wires daemon_start_sequence + run_server + UDS + tracing + ring-flush + 10s drain). SS-daemon-wiring-impl v1.3.0. SS-deps-pin-manifest v1.2.1. ARCH-INDEX v1.0.26. STORY-INDEX v5.32. sprint-state v1.40. S-DAEMON-WIRE-FIX-001 Wave-8 anchor. Resolved: ADV-W5GATE-HIGH-001, ADV-W3GATE-MED-002/004, ADV-W4GATE-MED-002, S-005-main-wiring, F-DW-HIGH-001. POL-11/POL-12 PASS. D-236: PRODUCT-VISION PIVOT — observe-only RETIRED; monocle → full TUI control center. Phases 4-7 SUSPENDED. D-237: Human ratified re-baselined-v1 control-center vision scope (4 capabilities: Launch, Embedded PTY, Multi-session/multi-project, Interactive Tune + already-built Observe+Control). DAEMON-OWNS-PTY locus. Hook auto-injection v1. embedded-pty-evaluation.md v1.0: primary = portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4. NEXT: gene-source disposition → revised vision-synthesis → human gate. D-238: Vision approval gate PASSED. domain-monocle-vision-synthesis.md APPROVED at v2.1 by Joshua Magady as the canonical basis for the control-center re-baselined-v1 brief→architecture→story delta. HUMAN ESCALATION folded in at the gate: v1A persistence now REQUIRES that a graceful daemon-PROCESS restart SURVIVES (CASE 2 changed from 'lost' to 'survive'). Persistence principle renamed DAEMON-OWNS-PTY → 'session-host-owns-PTY; daemon coordinates/re-attaches': PTY masters + harness child processes owned by native detached per-session session-host processes (abduco/dtach-style) that outlive the daemon process; daemon re-attaches over UDS on restart. NO-TMUX preserved as default; external supervisor is architect-surfaced fallback only (requires human decision, not silent adoption). CASE 1 (TUI restart survives) and CASE 3 (hard crash → lost, re-launch) unchanged. New HIGH-priority architect question Q-8 (PTY-ownership-survival mechanism) added; NOTE: the already-built D-235 in-process daemon wiring will likely need rework to move PTY ownership out of the daemon process. Remaining architect-only open questions: Q-1 (PTY bytes over UDS), Q-2 (EngineModule/SessionManager surface), Q-7 (tui-term fork posture), plus PTY-throughput benchmark — all resolved during architecture delta. Architect must also reconcile the stale narrow keyboard scope in DISPOSITION-V2 rollup + embedded-pty-evaluation (superseded by full-fidelity ratification). NEXT: brief delta (product-owner) → architecture delta (architect) → story decomposition (story-writer). D-292: Pass-49 FIX BURST — F-P49-001 BC-INDEX+SS-conventions canonical SS version mirror tables converted to registry pointers (POL-11 blind spot closed); BC-INDEX→v1.40.7; SS-conventions errata-no-bump; S-P49-001 HS-EXP-011/012 cite-precision; EVAL-INDEX BC-INDEX citation updated. COUNTER = 0. Pass-50 next. D-293: Pass-50 FIX BURST — F-P50-001 host_conn-lifecycle contradiction resolved; SessionNotReady/session_not_ready 12-code taxonomy added; full cascade swept (9 spec files, 31 prose files); SS-ipc v1.23.0, SS-session-manager v2.5.0, SS-daemon-wiring-v2-delta v1.11.1, SS-engine-module-v2-delta v1.6.0, BC-2.08.003 v1.4.0, BC-2.05.010 v1.9.0, BC-2.08.007 v1.5.0, BC-2.03.008 v1.0.3, BC-INDEX v1.40.8. COUNTER = 0. Pass-51 next. D-294: Pass-51 FIX BURST — F-P51-001 session_not_ready producer = DetachSession arm only (resize WARN-dropped per Invariant 6 Exception); SS-session-manager v2.5.1, SS-ipc v1.23.1, SS-daemon-wiring-v2-delta v1.11.2, BC-2.05.010 v1.9.1, BC-2.08.007 v1.5.1, BC-2.06.025 v1.4.0 (Launching action rules Invariant 5 + EC-298/EC-299), BC-INDEX v1.40.9. POL-11 cascade ~82 literals across ~34 files. COUNTER = 0. Pass-52 next. D-295: Pass-52 FIX BURST — F-P52-001 Terminated-in-GC-grace action×state matrix closed; rename→rename_failed (backs BC-2.08.005 Inv 4), detach→idempotent Ok, kill→idempotent Ok (BC-2.08.003 Inv 2), resize→WARN-drop; no new wire codes/variants; SS-session-manager v2.6.0, SS-ipc v1.23.2, SS-daemon-wiring-v2-delta v1.11.3, BC-2.06.025 v1.5.0 (Invariant 6 + EC-300/301/302), BC-2.08.005 v1.0.2, BC-INDEX v1.41.0; POL-11 cascade 90 literals across 31 files; POL-11/POL-12 GREEN; COUNTER = 0. Pass-53 next. D-296: Pass-53 FIX BURST — F-P53-001 BC-2.08.006 PC-3 authority corrected BC-HOOK-007→BC-2.04.010; 4-URL+2-empty hooks-settings.json schema; SessionStart NOT a file key; SS-daemon-wiring errata-no-bump sibling; BC-2.08.006 v1.3.0; POL-11/POL-12 GREEN; COUNTER = 0. Pass-54 next."
awaiting: "Adversarial Pass-54 (CONSECUTIVE-CLEAN COUNTER = 0; Pass-54 = clean candidate 1 of 3). Pass-53 DONE (D-296, 0C/1I — F-P53-001 BC-2.08.006 PC-3 authority corrected BC-HOOK-007→BC-2.04.010; 4-URL+2-empty hooks-settings.json file schema; SessionStart NOT a file key (5 served endpoints vs 4-URL+2-empty JSON keys); SS-daemon-wiring sibling errata-no-bump; BC-2.08.006 v1.3.0; no new wire codes/variants; POL-11/POL-12 GREEN; COUNTER stays 0). Pass-52 DONE (D-295, 0C/1I — F-P52-001 Terminated-in-GC-grace action×state matrix closed; rename→rename_failed (backs BC-2.08.005 Inv 4 revive-not-allowed); detach→idempotent Ok(); kill→idempotent Ok (BC-2.08.003 Inv 2); resize→WARN-drop; no new wire codes/variants; SS-session-manager v2.6.0, SS-ipc v1.23.2, SS-daemon-wiring-v2-delta v1.11.3, BC-2.06.025 v1.5.0 (Invariant 6 + EC-300/301/302), BC-2.08.005 v1.0.2, BC-INDEX v1.41.0; POL-11 cascade 90 literals/31 files; POL-11/POL-12 GREEN; COUNTER stays 0). Pass-51 DONE (D-294, 0C/1I — F-P51-001 session_not_ready producer reconciliation: ONE live wire producer = DetachSession arm for not-ready sessions; resize WARN-drops ALL errors per BC-2.05.010 Invariant 6 Exception (carve-out strengthened); SS-session-manager mapping table + step-8 + doc-comments corrected to detach-only; BC-2.05.010 §DetachSession PC-4 + §ResizePane rationale expanded; BC-2.08.007 detach defensive-branch note; S-P51-001 §Trace duplicate fixed in-scope; SIBLING CLOSED IN-SCOPE: BC-2.06.025 v1.4.0 Invariant 5 (kill ALLOWED/detach BLOCKED/rename allowed during Launching) + EC-298/EC-299; SS-session-manager v2.5.1, SS-ipc v1.23.1, SS-daemon-wiring-v2-delta v1.11.2, BC-2.05.010 v1.9.1, BC-2.08.007 v1.5.1, BC-2.06.025 v1.4.0, BC-INDEX v1.40.9; POL-11 cascade ~82 literals across ~34 files; POL-11/POL-12 GREEN; COUNTER stays 0). Pass-50 DONE (D-293, 0C/1I — F-P50-001 host_conn-lifecycle contradiction for kill-during-Launching resolved; daemon establishes CONTROL connection (host_conn.writer) during Launching via post-spawn monitor; proxy_task now Option<JoinHandle<()>>; BC-2.08.003 PC-1 confirmed correct; kill-during-Launching-race PID-fallback added; SessionError::SessionNotReady (9th variant) + session_not_ready wire code (12th; taxonomy 11→12) for defensive control-ops; full cascade swept: SS-ipc v1.23.0, SS-session-manager v2.5.0, SS-daemon-wiring-v2-delta v1.11.1, SS-engine-module-v2-delta v1.6.0, BC-2.08.003 v1.4.0, BC-2.05.010 v1.9.0, BC-2.08.007 v1.5.0, BC-2.03.008 v1.0.3, BC-INDEX v1.40.8; POL-11 cascade 83 literals across 31 files; POL-12 PASS; COUNTER stays 0). Pass-49 DONE (D-292, 0C/1I — F-P49-001 BC-INDEX Canonical SS version literal table converted to registry pointer (6 rows, 4 stale: SS-forward-compatibility v1.2.19→1.2.20, SS-engine-module v1.1.20→1.1.27, SS-deps-pin-manifest v1.1.17→1.2.1, SS-conventions v1.29.5→1.32.6); SS-conventions-anti-patterns sibling mirror-table also converted (errata-no-bump); BC-INDEX→v1.40.7; POL-11 blind spot CLOSED (hand-maintained mirror table anti-pattern retired, pointers to version-pin-registry.yaml substituted); S-P49-001 HS-EXP-011/012 cite-precision BC-2.08.004 PC-3→PC-6+Invariant 1 (errata-no-bump); SS-engine-module v1.1.26 §Trace historical entries — false alarm cleared; COUNTER stays 0). Pass-48 DONE (D-291, 0C/1I — F-P48-001 stale EngineError 2-variant doc-comment corrected to 3 variants in SS-session-manager (SessionError::EngineError enum variant) + SS-daemon-wiring-v2-delta sibling (I27-001 Model A step (a) comment); architect whole-class sweep found sibling adversary missed; zero live survivors; errata-no-bump; COUNTER stays 0). Pass-47 DONE (D-290, 0C/1I+2S — F-P47-001 Detached re-discovery emission removed Option B; S-P47-001 d-kill-alias; S-P47-002 clarifying note; BC-2.08.004→v1.3.0, BC-2.08.008→v1.3.0, BC-INDEX→v1.40.6; COUNTER stays 0). Pass-46 DONE (D-289, 0C/1I — F-P46-IMP-001 stale §Architecture Anchors version pins in BC-2.08.001/008, BC-2.09.008; stripped 11 parenthetical pins (version-less corpus-standard); POL-14 closes root-cause POL-11 blind spot; errata-no-bump; COUNTER RESET 1→0). Pass-45 DONE (D-288, 0C/0I CLEAN — no spec text changed; breadth sweep sound; 2 LOW suggestions S-P45-001/S-P45-002 deferred to Phase-4 holdout-prep; counter ADVANCES 0→1). Pass-44 DONE (D-287, 0C/1I — F-P44-IMP-001 EngineError::UnsupportedOperation→spawn_unsupported wire code + distinct banner; taxonomy 10→11; all 3 EngineError variants now mapped to dedicated codes; EngineError→wire-code class fully closed; FIXED). Pass-43 DONE (D-286, 0C/1I — F-P43-IMP-001 SpawnAck step missing from SS-daemon-wiring-v2-delta §3 duplicate IPC-handler skeleton; 7-of-7 spawn-handshake sibling; SS-daemon-wiring-v2-delta v1.10.0; FIXED). Pass-1 DONE (D-242). Pass-2 DONE (D-243). Pass-3 DONE (D-244). Pass-4 DONE (D-245). Pass-5 DONE (D-246). Pass-6 DONE (D-247). Pass-7 DONE (D-248, 0C/4I/4S — FIRST zero-Critical). Pass-8 DONE (D-249, 0C/2I/1S). Pass-9 DONE (D-250, 0C/1I/1S). Pass-10 DONE (D-252, 0C/1I/4S). Pass-11 DONE (D-253, 0C/1I/4S). Pass-12 DONE (D-254, 0C/2I/2S). Pass-13 DONE (D-255, 0C/2I/2S). Pass-14 DONE (D-256, 0C/1I/1S). Pass-15 DONE (D-257, 0C/2I/1S). Pass-16 DONE (D-258, 0C/1I/1S). Pass-17 DONE (D-259, 0C/1I/2S). Pass-18 DONE (D-260, 0C/1I/2S). Pass-19 DONE (D-261, 0C/1I/1S). Pass-20 DONE (D-262, 0C/0I/2S — FIRST CLEAN PASS). Pass-21 DONE (D-263, 0C/0I/1S — SECOND CLEAN PASS). Pass-22 DONE (D-264, 0C/3I/2S — FINDINGS; counter RESET 2→0). Pass-23 DONE (D-265, 0C/2I/1S — FINDINGS; counter stays 0). Pass-24 DONE (D-266, 0C/1I/2S — FINDINGS; counter stays 0; SS-09 family fully re-derived). Pass-25 DONE (D-267, 0C/1I/1S — FINDINGS; counter stays 0; I25-001 retired SessionState::Created swept from L1 brief+vision prose+enum; brief 2.0.3→2.0.4, vision 2.2.2→2.2.3). Pass-26 DONE (D-268, 0C/1I/1S — FINDINGS; counter stays 0; I26-001 BC-2.08.007 EC-185/EC-188 non-existent AlreadyAttached/AttachTimeout variants reconciled; S26-001 #[non_exhaustive] class closed: SpawnRecipe+PermissionDecisionKind+SessionState; 4 spec versions bumped + 29-file propagation sweep). Pass-27 DONE (D-269, 0C/1I/1S — FINDINGS; counter stays 0; I27-001 spawn-path Model-A reconciliation — spawn_session(opts: SpawnOptions), SpawnOptions wire type, EngineError taxonomy reachable; 10 docs bumped + 36-file/83-stale-active propagation sweep). Pass-28 DONE (D-270, 0C/1I/1S — FINDINGS; counter stays 0; I28-001 HS-EXP-014 Step 1 live Model-B spawn_session(recipe,harness,profile) survivor FIXED → spawn_session(opts_A) with SpawnOptions; sibling holdouts clean; S28-001 SS-session-manager §Trace cross-ref DEFERRED as housekeeping). Pass-29 DONE (D-271, 1C/0I/0S — FINDINGS; C29-001 harness_id missing from SpawnOptions struct in SS-engine-module-v2-delta; errata no-bump fix). Pass-30 DONE (D-272, 2C+1I — FINDINGS; C30-001/002 + I30-001 ADR-0006 cross-crate #[non_exhaustive] constructor gap; SpawnOptions/SpawnRecipe/SessionSnapshot/SerializedCell/PermissionPromptPayload missing constructors + audit-table rows; fixed E0639 ..opts). Pass-31 DONE (D-273, 1C+1I — FINDINGS; P31-CRIT-001 EngineError enum referenced-but-never-declared → declared canonical #[non_exhaustive] enum, 3 variants; P31-HIGH-001 SpawnRecipe parity in SS-session-manager). Pass-32 DONE (D-274, 0C/3I — FINDINGS; IMP-001 session_error_to_code _=> prose self-contradiction; IMP-002 dead anchor #engineerror-additions→#engineerror-new-in-v1a; IMP-003 EngineError 'extension'→new-type framing). Pass-33 DONE (D-275, 0C/2I — FINDINGS; I33-001 PERVASIVE dead-anchor class; I33-002 §Trace over-claim; WHOLE-CLASS-NOW: built ANCHOR-LINT-TOOL scripts/check_cross_ref_anchors.py = POL-13 wired CI+pre-commit; lint found 70 dead anchors; architect added 42 explicit <a id> navigational anchors across 12 docs; PO fixed 2 defective citations; ANCHOR-LINT-TOOL CLOSED). Pass-34 DONE (D-276, 1C+1I — FINDINGS; C34-001 InvalidPath null-byte detection impossible via to_str() → two-pronged detection: to_str() + as_bytes().contains(&0); I34-001 BC-2.08.001 stale anchor version labels). Pass-35 DONE (D-277, 0C/0I/3S — FIRST CLEAN of session; 3 Suggestions fixed in-scope → RESET counter to 0; S35-001 split-pair rationale arithmetic contradiction; S35-002 KeyInput SessionHostDead→attach_failed undocumented; S35-003 mouse SGR Drag(MouseButton) missing arm + Moved Ps 32→35 + full Ps/modifier table). Pass-36 DONE (D-278, 0C/2I — FINDINGS; F-P36-IMP-001 BC-2.09.003 Invariant-3 contradicted S35-003 Moved-unreachable model; F-P36-IMP-002 BC-2.05.010 missing named No-silent-failure invariant + Detach/Resize sibling error-path gaps + dangling SS arch forward-refs; whole-class swept ALL fallible variants; BC-2.09.003→1.5.0, BC-2.05.010→1.8.0; counter stays 0). Pass-37 DONE (D-279, 0C/2I — FINDINGS; F-P37-IMP-001 BC-2.09.002 Invariant-5 globally-active mouse capture — partial-fix sibling of Pass-36 BC-2.09.003 dismissal; F-P37-IMP-002 SS-ipc:412 invalid_request mis-described as pre-call vs post-call catch-all; BC-2.09.002→1.1.2; SS-ipc descriptor errata no-bump; counter stays 0). Pass-38 DONE (D-280, 0C/0I CLEAN; S38-001 BC-2.09.008 PC-4/PC-1 partial mouse-capture restatement fixed in-scope whole-class → counter RESET to 0). Pass-39 DONE (D-281, 0C/0I CLEAN; S39-001 errata SS-embedded-pty:250 global→scoped mouse-capture prose fixed in-scope → counter RESET to 0). Pass-40 DONE (D-282, 0C/0I CLEAN; OBS-1 BC-2.09.002 §Trace stale line-numbers deferred as housekeeping; counter ADVANCES 0→1). Pass-41 DONE (D-283, 0C/1I — F-P41-IMP-001 SessionCreation→EmbeddedTerminal session_id handshake gap FIXED; SpawnAck mechanism (b) ratified; counter RESET to 0). Pass-42 DONE (D-284, 0C/1I — F-P42-IMP-001 orphan wizard_session_id field-name corrected to launching_session_id in 4 SS-ipc sites + 1 SS-session-manager site; whole-class grep zero survivors; errata-no-bump; counter stays 0). Pass-43 DONE (D-286, 0C/1I — F-P43-IMP-001 SpawnAck step missing from SS-daemon-wiring-v2-delta §3 IPC-handler skeleton; last spawn-handshake sibling 7-of-7; SS-daemon-wiring-v2-delta v1.10.0; counter stays 0). Novelty trajectory (C:5,5,4,1,2,2,0×22,1,2,1,0,0,1,1,0,0,0,0,0,0,0,0,0,0; I:8,6,9,4,4,2,4,2,1,1,1,2,2,1,2,1,1,1,1,0,0,3,2,1,1,1,1,1,0,1,1,3,2,1,0,2,2,0,0,0,1,1,1,1,0,1,1,1,1,1,1,1). CONSECUTIVE-CLEAN COUNTER = 0. Pass-54 = clean candidate 1 of 3. STRICT-3-CLEAN CRITERION ACTIVE: 3 consecutive cleans required. PATTERN WARNING — PARTIAL-FIX SIBLINGS (Passes 33-44): each fix round exposes a sibling gap in the next pass. NOTE: F-P41→F-P42 self-sibling chain demonstrates that a fix's own primary doc can be the surviving sibling (Pass-41 corrected the SpawnAck variant doc-comment; Pass-42 found that SS-ipc body prose + §Trace errata + 3 additional sites still used wizard_session_id). NOTE: F-P43 demonstrates that a CANONICAL PATTERN LOCK co-propagation can miss a sibling doc (SS-daemon-wiring-v2-delta §3 IPC-handler skeleton not updated when Pass-41 + D-285 propagated the SpawnAck pattern). MITIGATION: for every fix round, owning agent must FIX-THE-WHOLE-CLASS — grep all sibling BCs/arch docs in same subsystem for same pattern; reconcile ALL instances in one burst. Tell adversary each pass to specifically hunt for siblings of the prior round's fix. CODIFIED CYCLE CHECKLIST (D-245 SUG-003): after any RETIRED-term or corrected-magic-number fix, grep -rn .factory/specs before next adversarial pass. CODIFIED (D-246): anchor-resolution-closure check. CODIFIED (D-255): L-CWD-PROPAGATION-ATTESTATION. CODIFIED (D-258): L-VERIFICATION-ARTIFACT-FALSE-GREEN. DEP-PIN-SWEEP-RULE (D-256). POL-13 LIVE (D-275, ANCHOR-LINT-TOOL). OPEN (non-blocking, human ratification required before v1A story wave): (1) CC-TUITERM-WIP-SIGNOFF; (2) CC-GLOBAL-MOUSE-CAPTURE. DEFERRED (non-blocking): VP authoring for SS-08/SS-09 BCs; v1B Tune BCs; v1B pre-emption BC."
durable_task_register:
  outstanding:
    - id: "DEP-PIN-SWEEP-RULE"
      subject: "[process-gap] extend POL-11 (or sibling check) to grep crate-name+version literals in spec prose against SS-deps-pin-manifest(-v2-delta) — root cause of I14-001"
      status: pending
      detail: "D-256 Pass-14 S14-001: POL-11 keys on artifact-ID version rows (e.g., 'SS-session-manager v1.8.0'), NOT on crate-name+version literals in prose (e.g., 'portable-pty 0.8.x'). I14-001 was a stale portable-pty 0.8.x literal in SS-session-manager §env-inheritance that escaped all prior POL-11 sweeps. Tooling enhancement: add a sibling check that greps spec prose for crate-pin literals and validates them against SS-deps-pin-manifest(-v2-delta) canonical versions. Route to devops-engineer when a POL-extension sprint is scheduled. Non-blocking; no production impact until v1A implementation begins."
      blocking: false
    - id: "POL-14-PARENTHETICAL-ANCHOR-PIN"
      subject: "[process-gap CLOSED D-289] POL-11 blind spot for `path#anchor (vX.Y.Z)` parenthetical form in §Architecture Anchors — closed by POL-14 on develop @ 5d9d603"
      status: closed-d289
      detail: "D-289 Pass-46 F-P46-IMP-001: POL-11's ID↔version adjacency regex did not match the `path#anchor (vX.Y.Z)` form used in §Architecture Anchors sections of BC files. 11 stale pins across BC-2.08.001 (5), BC-2.08.008 (3), BC-2.09.008 (3) went undetected for multiple passes. Root-cause process-gap CLOSED by POL-14 (Pattern C detection in scripts/check_version_pins.py + CI pol-lint + lefthook, develop @ 5d9d603). Corpus-standard fix: §Architecture Anchors are now version-less navigational references; authoritative pins live only in §Architecture Source rows. Errata-no-bump (3 BCs; no version bump required). Lesson codified as POL-COVERAGE-BLIND-SPOT in Section F."
      blocking: false
    - id: "POL-11-MIRROR-TABLE-BLIND-SPOT"
      subject: "[process-gap] POL-11 is blind to version literals embedded in markdown tables whose column headers don't match the ID↔version adjacency pattern — mitigation applied D-292 (retire hand-maintained registry mirrors in favor of pointers)"
      status: mitigation-applied-d292
      detail: "D-292 Pass-49 F-P49-001: BC-INDEX.md and SS-conventions-anti-patterns.md each carried a 'Canonical SS version table' — a 6-row markdown table of SS document → version mappings. These were hand-maintained mirrors of version-pin-registry.yaml. POL-11's ID↔version adjacency regex did not match markdown table cell values (e.g., '| SS-engine-module.md | v1.1.20 |') because the table cell form has separators and whitespace that break the adjacency check. Result: 4 of 6 rows silently drifted (SS-forward-compatibility v1.2.19→1.2.20, SS-engine-module v1.1.20→1.1.27, SS-deps-pin-manifest v1.1.17→1.2.1, SS-conventions v1.29.5→1.32.6) over multiple passes without POL-11 detection. MITIGATION APPLIED (D-292): both tables removed and replaced with explicit registry pointers to version-pin-registry.yaml (the authoritative source). Pattern is now retired — do not re-introduce hand-maintained version mirror tables in spec prose. CODIFICATION PENDING: add a check for markdown table-cell version literals against the registry to POL-11, or document this anti-pattern in the Cycle-Closing Checklist. Route to devops-engineer when a POL-extension sprint is scheduled. Non-blocking."
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
    - id: "S28-001"
      subject: "SS-session-manager §Trace v2.0.0 Fix(c) cross-ref inaccuracy — states SpawnRecipe Serialize/Deserialize derives were removed in SS-engine-module-v2-delta v1.2.0 edit; they were RETAINED as harmless daemon-internal derives"
      status: pending
      detail: "D-270 Pass-28 S28-001: SS-session-manager §Trace v2.0.0 Fix(c) description states that SpawnRecipe Serialize/Deserialize derives were removed as part of the SS-engine-module-v2-delta v1.2.0 architectural change (SpawnRecipe demoted to daemon-internal). In actuality the Serialize/Deserialize derives were RETAINED on SpawnRecipe — they are harmless on a daemon-internal type (not a wire type), and removing them was not required. The §Trace changelog entry thus inaccurately describes what was changed. Non-blocking: the normative body of SS-session-manager v2.0.0 is correct; only the §Trace Fix(c) description is inaccurate. Deferred as §Trace housekeeping — a standalone SS-session-manager bump solely for a changelog annotation would trigger a disproportionate 25+ file propagation sweep. Fold into the next substantive SS-session-manager normative edit."
      blocking: false
    - id: "ENGINE-RS-DEVERSION"
      subject: "[process-gap] crates/monocle-core/src/engine.rs doc-comment version pins should be de-versioned per CLAUDE.md CI-PARITY rule 3"
      status: pending
      detail: "D-277 session: engine.rs doc-comments cite specific spec version literals (e.g., 'SS-engine-module v1.x.y') which POL-11 flags when those specs bump. These were bumped during the D-277 propagation sweep to stay POL-11 green, but the anti-pattern of versioned literals in source doc-comments persists. CLAUDE.md CI-PARITY rule 3 states: 'Do NOT embed version-pin literals in test prose or source doc-comments (de-version them; POL-11 flags stale)'. Route to implementer for a develop cleanup commit (de-version the doc-comments to bare section references). Non-blocking."
      blocking: false
    - id: "BC-INDEX-TRACE-SS08-COUNT"
      subject: "BC-INDEX §Trace line ~1337 stale 'SS-08 ... 7 BCs' (actual 8)"
      status: pending
      detail: "D-277 session: BC-INDEX §Trace at approximately line 1337 contains a historical count 'SS-08 ... 7 BCs' which reflects the pre-I16-001 state before BC-2.08.008 was added in D-258. The normative §Summary section correctly shows 8 BCs for SS-08. The §Trace entry is a historical annotation only; a standalone BC-INDEX bump to fix a §Trace count would trigger a disproportionate sweep. Fold into next substantive BC-INDEX normative edit. Non-blocking."
      blocking: false
    - id: "SS-IPC-181-REDUNDANT-HISTORICAL-MARKER"
      subject: "SS-ipc.md line ~181 active cross-ref pin to SS-daemon-wiring-v2-delta v1.9.1 has a redundant version-pin-historical marker — NOW STALE (D-286 bumped to v1.10.0)"
      status: pending
      detail: "D-277 session: during the Pass-35 propagation sweep, SS-ipc.md line ~181 received a version-pin-historical marker on an active cross-ref pin to SS-daemon-wiring-v2-delta v1.9.1. The pin was current at the time but is POL-11-exempt (version-pin-historical annotation). D-286: SS-daemon-wiring-v2-delta bumped to v1.10.0 — the line ~181 pin is now STALE (still says v1.9.1) and POL-11 cannot catch it due to the historical marker. Action required: remove the version-pin-historical marker from SS-ipc line ~181 and update the version to v1.10.0 (or convert to version-free form citing only the section anchor). Non-blocking for Phase-1d convergence but should be addressed before Phase-2 story decomposition."
      blocking: false
    - id: "OBS-1-BC2.09.002-TRACE-LINE-NUMBERS"
      subject: "[housekeeping] BC-2.09.002 Inv-5 §Trace cites stale SS-embedded-pty line numbers (254-258) for the scoped entry sequence; actual ENTRY block at 279-289"
      status: deferred-housekeeping
      detail: "D-282 Pass-40 OBS-1: navigational stale-line-number in a §Trace citation. Section-name anchors (§Crossterm-setup / §EmbeddedTerminal-ENTRY) resolve correctly and POL-13 passes; line numbers are an informal navigational aid, not the enforced anchor. No version bump required per the closed dead-anchor/navigational-reference precedent (C29-001 / D-275). Fix by updating the line-number references to 279-289 (entry) when BC-2.09.002 is next edited for a substantive reason — bumping a heavily-cited authoritative BC solely for a §Trace line-number is disproportionate. Non-blocking; zero implementation impact."
      blocking: false
    - id: "DEDUP-IPC-HANDLER-SKELETON"
      subject: "[structural] de-duplicate the two normative IPC-handler skeletons (SS-session-manager §IPC handler is canonical; SS-daemon-wiring-v2-delta §3 should become reference-plus-delta) — duplication caused F-P43 despite CANONICAL PATTERN LOCK"
      status: pending-orchestrator-action
      detail: "D-286: architect recommendation. SS-session-manager §IPC handler and SS-daemon-wiring-v2-delta §3 both carry full normative Rust handler skeletons of the same ClientToServer dispatch; the F-P41 sweep updated the canonical but missed the delta sibling (root cause of F-P43). Production-grade fix: make SS-session-manager the single authoritative skeleton and restructure SS-daemon-wiring-v2-delta §3 to cross-reference it + retain only its legitimate delta annotations (IpcOp import note, error-routing summary table, §3b emission rule). Requires architect + PO awareness (summary table has normative force). Concrete orchestrator-directed action item (NOT tech-debt-register; NOT a vague 'later'). Schedule as a consistency task before/with Phase-2 story decomposition. Non-blocking for Phase-1d convergence (both skeletons currently in sync)."
      blocking: false
    - id: "OBS-HS-PROSE-PHASE4-PREP"
      subject: "[housekeeping] two LOW holdout-prose imprecisions to tidy at Phase-4 holdout-prep — HS-EXP-014:46 omits child_pid from illustrative session-state.json field list; HS-EXP-013:54 step-9 says 'most recently added' vs BC-2.09.009 PC-5c canonical 'front=oldest'"
      status: deferred-phase4-prep
      detail: "D-288 Pass-45 S-P45-001 + S-P45-002: both are non-normative illustrative prose in holdout scenarios; neither contradicts a normative BC behaviorally (PASS/FAIL criteria are independent of the imprecise detail; S-P45-002 prose is hedged and cites the rotation-semantics BC). No implementer/evaluator is misled. Anchored to the Phase-4 holdout-evaluation prep pass (when HS scenarios get their dedicated pre-eval review): tidy HS-EXP-014:46 to include child_pid; align HS-EXP-013:54 step-9 to 'the front of the stack (oldest pending prompt) per BC-2.09.009 PC-5c'. Non-blocking for Phase-1d spec convergence."
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
  ZERO-CONTEXT RESUME CHECKPOINT v7.46 (D-271..D-296) — 2026-06-14
  PIVOT: monocle → full TUI control center — PHASE-1D ADV CONVERGENCE IN PROGRESS
  53 PASSES COMPLETE; CONSECUTIVE-CLEAN COUNTER = 0; PASS-54 NEXT (clean candidate 1 of 3)
  ============================================================================

  READ THESE FIRST (in order — before anything else):
  1. /Users/jmagady/Dev/monocle/NEXT-SESSION-RESUME.md  ← concise 1-page entry point (updated)
  2. /Users/jmagady/Dev/monocle/CLAUDE.md               ← production-grade + agent-routing rules
  3. This STATE.md fully                                 ← durable_task_register + all deferreds

  ============================================================================
  A. WHERE WE ARE (D-271..D-293, 2026-06-14)
  ============================================================================

  MODE: greenfield-with-reference-ingest.
  PHASE: VSDD Phase 1d ADVERSARIAL SPEC CONVERGENCE — IN PROGRESS.
  develop @ 8bc22a5 — UNCHANGED for production code. develop HAS had docs/version-pin/CI-wiring
  commits this session (POL-13 anchor-lint CI wiring, version-pin maintenance, etc.).
  NO v1A production code written yet. Do NOT write v1A code yet.
  factory-artifacts: run `git -C .factory log -1 --format='%h %s'` for live HEAD.

  53 adversarial passes complete on the v1A control-center spec package.
  Finding trajectory (Critical / Important per pass):
    Pass 1: 5C/8I  Pass 2: 5C/6I  Pass 3: 4C/9I  Pass 4: 1C/4I  Pass 5: 2C/4I  Pass 6: 2C/2I
    Pass 7: 0C/4I  Pass 8: 0C/2I  Pass 9: 0C/1I  Pass 10: 0C/1I Pass 11: 0C/1I Pass 12: 0C/2I
    Pass 13: 0C/2I Pass 14: 0C/1I Pass 15: 0C/2I Pass 16: 0C/1I Pass 17: 0C/1I Pass 18: 0C/1I
    Pass 19: 0C/1I Pass 20: 0C/0I (FIRST CLEAN) Pass 21: 0C/0I (SECOND CLEAN)
    Pass 22: 0C/3I (FINDINGS — counter RESET 2→0; sibling-BC cluster SS-09 scrollback/AppMode)
    Pass 23: 0C/2I  Pass 24: 0C/1I  Pass 25: 0C/1I  Pass 26: 0C/1I  Pass 27: 0C/1I  Pass 28: 0C/1I
    Pass 29: 1C/0I (D-271 — C29-001 harness_id missing from SpawnOptions; errata no-bump fix)
    Pass 30: 2C/1I (D-272 — ADR-0006 constructor gap; E0639 ..opts; constructors for 5 wire types)
    Pass 31: 1C/1I (D-273 — EngineError enum declared canonical; SpawnRecipe SS-session-manager parity)
    Pass 32: 0C/3I (D-274 — session_error_to_code _=> contradiction; dead anchor; 'extension' framing)
    Pass 33: 0C/2I (D-275 — PERVASIVE dead-anchor class; WHOLE-CLASS-NOW; POL-13/ANCHOR-LINT-TOOL live)
    Pass 34: 1C/1I (D-276 — InvalidPath null-byte detection two-pronged; BC-2.08.001 stale anchor labels)
    Pass 35: 0C/0I (D-277 — FIRST CLEAN of session; 3 Suggestions fixed in-scope → RESET counter to 0)
    Pass 36: 0C/2I (D-278 — F-P36-IMP-001 BC-2.09.003 Invariant-3 Moved-unreachable contradiction;
             F-P36-IMP-002 BC-2.05.010 missing No-silent-failure invariant + Detach/Resize gaps;
             whole-class swept all fallible variants; BC-2.09.003→1.5.0, BC-2.05.010→1.8.0)
    Pass 37: 0C/2I (D-279 — F-P37-IMP-001 BC-2.09.002 Invariant-5 globally-active mouse capture
             (partial-fix sibling of I3 scoped-capture; Pass-36 BC-2.09.003 fix dismissed sibling);
             F-P37-IMP-002 SS-ipc:412 invalid_request mis-described as pre-call;
             BC-2.09.002→1.1.2; SS-ipc descriptor errata no-bump)
    Pass 38: 0C/0I CLEAN (D-280 — S38-001 SUGGESTION: BC-2.09.008 PC-4/PC-1 partial
             mouse-capture restatement; fixed in-scope whole-class; BC-2.09.008→1.2.0,
             BC-INDEX→1.40.1; SS-09/SS-05 swept zero survivors; counter RESET to 0)
    Pass 39: 0C/0I CLEAN (D-281 — S39-001 OBSERVATION: SS-embedded-pty:250 stale
             'global EnableMouseCapture' prose↔code contradiction; errata fixed in-scope;
             whole-class arch+BC sweep zero survivors; errata-no-bump; counter RESET to 0)
    Pass 40: 0C/0I CLEAN (D-282 — OBS-1 OBSERVATION: BC-2.09.002 §Trace stale line-numbers
             254-258 vs actual 279-289; deferred as housekeeping (navigational aid, not enforced
             anchor; POL-13 green); no spec change; counter ADVANCES 0→1)
    Pass 41: 0C/1I FINDINGS (D-283 — F-P41-IMP-001: session_id handshake gap; SpawnAck
             mechanism (b) ratified; UUID locus reconciled to IPC handler;
             launching_session_id field added to AppMode::SessionCreation;
             BC-2.08.001/008, BC-2.09.008 propagated; whole-class sweep zero survivors;
             counter RESET 1→0)
    Pass 42: 0C/1I FINDINGS (D-284 — F-P42-IMP-001: orphan wizard_session_id field-name in
             SS-ipc SpawnAck doc-comment (normative MUST) + SS-session-manager skeleton comment;
             partial-fix sibling of Pass-41 SpawnAck fix; canonical is launching_session_id;
             SS-ipc body contradicted its own §Trace; 4 SS-ipc sites + 1 SS-session-manager
             site corrected; whole-class grep zero survivors; errata-no-bump; counter stays 0)
    Pass 43: 0C/1I FINDINGS (D-286 — F-P43-IMP-001: SpawnAck step missing from SS-daemon-wiring-v2-delta
             §3 duplicate IPC-handler skeleton; last spawn-handshake sibling 7-of-7; CANONICAL
             PATTERN LOCK co-propagation missed the §3 mirror skeleton; SS-daemon-wiring-v2-delta
             v1.10.0; whole-class confirmed all 3 representations now synced; counter stays 0)
    Pass 44: 0C/1I FINDINGS (D-287 — F-P44-IMP-001: EngineError::UnsupportedOperation collapsed to
             generic invalid_request wire code; BC-2.03.008 PC-3/EC-112 required a distinct banner;
             FIXED: spawn_unsupported (11th code) + banner "Session spawn not supported for this
             harness"; all 3 EngineError variants now map to dedicated codes; taxonomy fully closed;
             SS-ipc v1.22.0, SS-session-manager v2.4.0, SS-engine-module-v2-delta v1.5.0,
             SS-daemon-wiring-v2-delta v1.11.0, BC-2.03.008 v1.0.2, BC-2.05.010 v1.8.1;
             counter stays 0)
    Pass 45: 0C/0I CLEAN (D-288 — breadth sweep sound; 2 LOW suggestions S-P45-001/S-P45-002
             deferred to Phase-4 holdout-prep as OBS-HS-PROSE-PHASE4-PREP; no spec change;
             counter ADVANCES 0→1)
    Pass 46: 0C/1I FINDINGS (D-289 — F-P46-IMP-001: stale §Architecture Anchors version
             pins in BC-2.08.001 (5 pins), BC-2.08.008 (3 pins), BC-2.09.008 (3 pins);
             11 total parenthetical pins drifted from authoritative §Architecture Source rows;
             invisible to old POL-11 (ID↔version adjacency regex missed path#anchor (vX.Y.Z) form);
             FIXED: corpus-standard version-less navigational anchors adopted (errata-no-bump);
             root-cause process-gap CLOSED by POL-14 (check_version_pins.py Pattern C +
             CI pol-lint + lefthook, develop @ 5d9d603); counter RESET 1→0)
    Pass 47: 0C/1I FINDINGS (D-290 — F-P47-001: Detached re-discovery MUST NOT emit
             SessionStateChanged (Option B decision); BC-2.08.004 PC-2b + EC-172 removed
             over-specified emission; BC-2.08.008 Invariant-1 tightened to genuine-value-change
             rule; symmetry with Running/Launching re-discovery restored;
             S-P47-001 d-kill-alias added to BC-2.06.025 Terminating lists (errata-no-bump);
             S-P47-002 BC-2.03.008 PC-2 clarifying note (errata-no-bump);
             BC-2.08.004→v1.3.0, BC-2.08.008→v1.3.0, BC-INDEX→v1.40.6; counter stays 0)
    Pass 48: 0C/1I FINDINGS (D-291 — F-P48-001: stale SessionError::EngineError variant
             doc-comment enumerated only 2 EngineError variants (BinaryNotFound/InvalidPath)
             omitting UnsupportedOperation→spawn_unsupported added at Pass-44/D-287; corrected
             to 3-variant enumeration in SS-session-manager (v2.4.0 errata) + sibling fixed in
             SS-daemon-wiring-v2-delta I27-001 Model A comment step (a) (v1.11.0 errata);
             architect whole-class sweep found SS-daemon-wiring sibling adversary missed;
             normative surfaces (mapping table, exhaustiveness prose, session_error_to_code()
             body) were already correct after Pass-44; errata-no-bump; counter stays 0)
    Pass 49: 0C/1I FINDINGS (D-292 — F-P49-001: BC-INDEX Canonical SS version table (6-row
             hand-maintained mirror of version-pin-registry.yaml) had 4 stale rows
             (SS-forward-compatibility v1.2.19, SS-engine-module v1.1.20, SS-deps-pin-manifest
             v1.1.17, SS-conventions v1.29.5 — all stale vs registry); POL-11 was blind to
             markdown-table-cell version literal form; FIXED: literal tables removed from
             BC-INDEX + SS-conventions-anti-patterns, replaced with registry pointers; BC-INDEX
             →v1.40.7; SS-conventions errata-no-bump (v1.32.6 stays); POL-11 blind-spot
             pattern codified as POL-11-MIRROR-TABLE-BLIND-SPOT; S-P49-001: HS-EXP-011/012
             cite-precision BC-2.08.004 PC-3→PC-6+Invariant 1 (errata-no-bump);
             SS-engine-module v1.1.26 §Trace entries cleared as historical (false alarm);
             counter stays 0)
    Pass 50: 0C/1I FINDINGS (D-293 — F-P50-001: host_conn-lifecycle contradiction for
             kill-during-Launching resolved; daemon establishes CONTROL connection (host_conn.writer)
             during Launching via post-spawn monitor (distinct from proxy_task PTY-streaming established
             at Running); proxy_task now Option<JoinHandle<()>>; BC-2.08.003 PC-1 confirmed correct;
             kill-during-Launching-race PID-fallback path (→kill_failed) added; SessionError::SessionNotReady
             (9th variant) + session_not_ready wire code (12th; taxonomy 11→12) for defensive
             control-ops on not-ready sessions (untrusted-client wire posture; semantically distinct
             from invalid_request; no-silent-failure); full 11→12 / 8→9 cascade swept across SS-ipc
             v1.23.0, SS-session-manager v2.5.0, SS-daemon-wiring-v2-delta v1.11.1, # version-pin-historical: Pass-50 canonical versions at time of that fix
             SS-engine-module-v2-delta v1.6.0, BC-2.08.003 v1.4.0, BC-2.05.010 v1.9.0,
             BC-2.08.007 v1.5.0, BC-2.03.008 v1.0.3, BC-INDEX v1.40.8;
             POL-11 cascade 83 literals across 31 files; counter stays 0)
    Pass 53: 0C/1I FINDINGS (D-296 — F-P53-001: BC-2.08.006 PC-3 wrong authority (BC-HOOK-007 =
             DTU gene-source clone scope); corrected to BC-2.04.010 PC-3: 4-URL keys {PreToolUse,
             Notification,Stop,UserPromptSubmit} + 2 reserved-empty {PostToolUse:[],PreCompact:[]};
             SessionStart NOT a file key (5 served endpoints vs 4-URL+2-empty JSON keys);
             SS-daemon-wiring §SessionStart Invocation Path Note sibling errata-no-bump;
             whole-class sweep: 0 BC-layer + 0 arch survivors; BC-2.08.006 v1.3.0; no new wire
             codes/variants; counter stays 0)
    Pass 52: 0C/1I FINDINGS (D-295 — F-P52-001: Terminated-in-GC-grace action×state gap closed;
             rename-on-Terminated→rename_failed (backs BC-2.08.005 Inv 4 revive-not-allowed);
             detach-on-Terminated→idempotent Ok(); kill-on-Terminated→idempotent Ok (BC-2.08.003 Inv 2);
             resize→WARN-drop; no new wire codes/variants; Launching rules + 9-variant/12-code counts
             preserved; BC-2.06.025 v1.5.0 Invariant 6 + EC-300/301/302; BC-2.08.005 v1.0.2;
             SS-session-manager v2.6.0 §Terminated-in-grace matrix; errata to SS-ipc v1.23.2 +
             SS-daemon-wiring-v2-delta v1.11.3; survivor count 0; counter stays 0)
    Pass 51: 0C/1I FINDINGS (D-294 — F-P51-001: session_not_ready producer cross-doc contradiction
             resolved; ONE live wire producer = DetachSession IPC arm for not-ready (Launching, host_conn
             None) sessions (defensive, untrusted-client); Resize EXCLUDED — ResizePane WARN-drops ALL
             errors per BC-2.05.010 Invariant 6 Exception (arch constraint; no-silent-failure exception);
             SS-session-manager mapping table + step-8 + doc-comments corrected to detach-only;
             SS-ipc + SS-daemon-wiring errata traces added; BC-2.05.010 §DetachSession PC-4 gains
             session_not_ready bullet; §ResizePane rationale expanded; BC-2.08.007 detach defensive-branch
             note; S-P51-001 §Trace copy-paste duplicate fixed in-scope; SIBLING CLOSED IN-SCOPE:
             BC-2.06.025 v1.4.0 — Invariant 5 (kill ALLOWED per F-P50-001 / detach BLOCKED+no-op /
             rename allowed during Launching) + EC-298 (kill-during-Launching: PID fallback via
             host_conn.writer per BC-2.08.003) + EC-299 (detach-during-Launching: session_not_ready
             response) + VP refs backing 'BC-2.06.025 guards' cites; all 'BC-2.06.025 guards'
             citations now resolve; zero survivors; SS-session-manager v2.5.1, SS-ipc v1.23.1,
             SS-daemon-wiring-v2-delta v1.11.2, BC-2.05.010 v1.9.1, BC-2.08.007 v1.5.1,
             BC-2.06.025 v1.4.0, BC-INDEX v1.40.9;
             POL-11 cascade ~82 literals across ~34 files; POL-11/POL-12 GREEN; counter stays 0)

  Passes 7 through 52 ALL ZERO-Critical except Pass 29 (C29-001) and Pass 34 (C34-001).
  CONSECUTIVE-CLEAN COUNTER = 0. Pass-54 = clean candidate 1 of 3.
  NEXT = adversarial Pass 54.
  HUMAN DIRECTIVE: strict 3-consecutive-clean (reaffirmed; do NOT accept fewer than 3).
  STRICT-3-CLEAN VINDICATED: passes 29-51 have each surfaced a substantive defect or required
  a whole-class in-scope fix, confirming the 3-clean bar is correctly calibrated.

  PATTERN WARNING — PARTIAL-FIX SIBLINGS (Passes 33-44):
  The recurring finding pattern is PARTIAL-FIX SIBLINGS: each fix round exposes a sibling gap
  that the NEXT pass catches. Examples: fixing KeyInput error path in BC-2.05.010 (S35-002) left
  Detach/Resize sibling gaps (F-P36-IMP-002); fixing BC-2.09.003 mouse model (S35-003 + D-278)
  left BC-2.09.002 Invariant-5 scoped-capture sibling (F-P37-IMP-001); SpawnAck fix (D-283)
  left wizard_session_id orphan name in SS-ipc body/§Trace/doc-comment (F-P42-IMP-001, D-284);
  D-283/D-285 CANONICAL PATTERN LOCK propagation missed SS-daemon-wiring-v2-delta §3 mirror
  skeleton (F-P43-IMP-001, D-286).
  NOTE: F-P41→F-P42 self-sibling chain: a fix's own primary doc can be the surviving sibling.
  NOTE: F-P43 demonstrates that structural duplication across docs (canonical vs delta mirror)
  is a distinct failure mode from in-doc sibling gaps — BOTH must be checked in every sweep.
  MITIGATION (mandatory every fix round):
    1. Owning agent MUST grep all sibling BCs and arch docs in the SAME subsystem for the same
       pattern and reconcile ALL instances in ONE burst — not just the flagged instance.
    2. Tell the adversary each pass to SPECIFICALLY HUNT for siblings of the prior round's fix.
    3. Do NOT declare a fix-class closed until a whole-subsystem grep confirms no survivors.
    4. For features with CANONICAL + DELTA mirror docs (e.g., SS-session-manager vs
       SS-daemon-wiring-v2-delta §3): explicitly include the delta mirror in every sweep.

  This session's key changes (Passes 35-42):
  - Pass-35 (D-277): 0C/0I CLEAN. S35-001 (split-pair arithmetic) + S35-002 (KeyInput error path)
    + S35-003 (mouse SGR Drag/Moved/modifier table) fixed in-scope. Counter RESET to 0.
  - Pass-36 (D-278): 0C/2I FINDINGS. F-P36-IMP-001 BC-2.09.003 Invariant-3 contradicted the
    S35-003 Moved-unreachable model (Ps=35 reachable under 1003 not just 1002); F-P36-IMP-002
    BC-2.05.010 missing named "No-silent-failure invariant" + Detach/Resize error-path gaps +
    dangling SS-session-manager:385/SS-ipc:389,1515 arch forward-refs. PO-only fix burst: named
    invariant authored; whole-class swept ALL fallible variants (Kill/Rename/Attach documented
    too); ResizePane documented as WARN-drop exception. BC-2.09.003→1.5.0, BC-2.05.010→1.8.0.
  - Pass-37 (D-279): 0C/2I FINDINGS. F-P37-IMP-001 BC-2.09.002 Invariant-5 still asserted
    globally-active mouse capture — partial-fix sibling of the I2-001/I3 scoped-capture fix that
    Pass-36's BC-2.09.003 fix had incorrectly dismissed. F-P37-IMP-002 SS-ipc:412 described
    invalid_request trigger as pre-call (never-matched guard) vs canonical post-call catch-all
    (always-matched for unhandled variants). PO fixed BC-2.09.002→1.1.2 (scoped-capture
    invariant); architect fixed SS-ipc:412 descriptor errata (no version bump — wire contract
    unchanged, prose-only correction).
  - Pass-41 (D-283): 0C/1I FINDINGS. F-P41-IMP-001 session_id handshake gap — TUI had no
    deterministic way to learn daemon-assigned session_id after SpawnSession. Fixed: mechanism (b)
    deterministic SpawnAck: SS-ipc→1.21.0 (new ServerToClient::SpawnAck{session_id} variant),
    SS-embedded-pty→1.6.0 (new launching_session_id: Option<String> on AppMode::SessionCreation),
    SS-session-manager→2.3.0 (UUID locus moved to IPC handler); BC-2.08.001→1.5.0,
    BC-2.08.008→1.2.0, BC-2.09.008→1.3.0, BC-INDEX→1.40.2. Counter RESET 1→0.
  - Pass-42 (D-284): 0C/1I FINDINGS. F-P42-IMP-001 orphan field-name wizard_session_id —
    partial-fix sibling of Pass-41 SpawnAck fix. Pass-41 corrected the SpawnAck variant
    doc-comment; Pass-42 found 4 additional SS-ipc sites (body prose, §Trace errata, 2 internal
    doc-comments) + 1 SS-session-manager skeleton comment still using wizard_session_id.
    SS-ipc body contradicted its own §Trace (§Trace said launching_session_id; body used
    wizard_session_id). Fixed via architect: whole-class grep confirmed zero live survivors.
    Errata-no-bump: wire contract + canonical struct unchanged. SS-ipc stays v1.21.0,
    SS-session-manager stays v2.3.0. Counter stays 0.
  - Pass-43 (D-286): 0C/1I FINDINGS. F-P43-IMP-001 SpawnAck step missing from SS-daemon-wiring-v2-delta
    §3 duplicate IPC-handler skeleton — the §3 SpawnSession arm did not carry the 4-step SpawnAck
    sequence that D-283/D-285 propagated to SS-session-manager and SS-ipc. Root cause: CANONICAL
    PATTERN LOCK co-propagation missed the §3 mirror skeleton in SS-daemon-wiring-v2-delta (structural
    duplication between canonical SS-session-manager and delta mirror). This is the 7-of-7
    spawn-handshake sibling; all 3 normative representations now in sync. Fixed: §3 SpawnSession
    arm now carries full 4-step SpawnAck sequence + spawn-fail clear-launching_session_id note.
    SS-daemon-wiring-v2-delta v1.9.1 → v1.10.0. POL-11 16-literal sweep across 8 BC files.
    DEDUP-IPC-HANDLER-SKELETON added to durable_task_register. Counter stays 0.
  - Pass-44 (D-287): 0C/1I FINDINGS. F-P44-IMP-001 EngineError::UnsupportedOperation collapsed to
    generic invalid_request wire code — BC-2.03.008 PC-3/EC-112 mandated a DISTINCT banner
    ("Session spawn not supported for this harness") symmetric to the already-closed I12-001
    (BinaryNotFound→binary_not_found) and F-P34-IMP-001 (InvalidPath→invalid_spawn_arg).
    Fixed: SS-ipc v1.22.0 (11th wire code spawn_unsupported, session_error_to_code arm);
    SS-session-manager v2.4.0 (UnsupportedOperation arm + EC-112 defense-in-depth comment);
    SS-engine-module-v2-delta v1.5.0 (best-effort filtering note; Lines 333-334 reconciled);
    SS-daemon-wiring-v2-delta v1.11.0 (error table UnsupportedOperation row + spawn comment);
    BC-2.03.008 v1.0.2 (PC-3 + EC-112 + test vector);
    BC-2.05.010 v1.8.1 (PC-4 spawn_unsupported obligation).
    All 3 EngineError variants now map to dedicated codes — EngineError→wire-code class closed.
    POL-11 cascade sweep: 88 stale literals across 30 files updated + 1 historical annotation.
    Counter stays 0.
  - Pass-45 (D-288): 0C/0I CLEAN. Breadth sweep (PTY/scrollback/persistence/hooks/
    permission-overlay/SessionState/holdouts) sound; Pass-44 cascade verified complete/correct.
    Two LOW suggestions deferred to Phase-4 holdout-prep (OBS-HS-PROSE-PHASE4-PREP):
    S-P45-001 HS-EXP-014:46 omits child_pid from illustrative field list (non-normative;
    FAIL criterion independent); S-P45-002 HS-EXP-013:54 step-9 "most recently added" vs
    BC-2.09.009 PC-5c "front=oldest" (hedged prose; PASS-criterion independent).
    No spec text changed. Counter ADVANCES 0→1. Adversary should NOT re-flag these.

  ============================================================================
  B. THE CONVERGENCE LOOP PROCEDURE (how a fresh session runs Pass 40+)
  ============================================================================

  Step A — Dispatch vsdd-factory:adversary FRESH-CONTEXT for Pass N.
    Scope = v1A control-center spec package (full list in section D below).
    Rubric = CLAUDE.md production-grade principle.
    Adversary MUST verify all these axes:
      (a) anchor-resolution closure — every BC Architecture-Source citation resolves to a
          symbol that EXISTS in the cited doc (C5-001 class prevention)
      (b) full-lifecycle end-to-end consistency: SS ↔ BC ↔ ADR ↔ holdout
      (c) no live normative survivor of retired concepts (Created/Killed enum variants;
          Model-B spawn_session(recipe,harness,profile) signature; daemon-owned-PTY framing)
      (d) error-taxonomy completeness + no-silent-failure guarantee (9-variant SessionError;
          12-code wire taxonomy; 10-code EngineError base taxonomy reachable under Model-A)
      (e) security (keystroke injection, auth token exfil, SO_PEERCRED enforcement paths)
      (f) PARTIAL-FIX SIBLING HUNT — specifically check all BCs/SS docs in the SAME subsystem
          as the prior round's fix for the same pattern (e.g., if BC-2.09.003 was fixed last
          pass, check BC-2.09.001/002/004..009 for the same gap; if BC-2.05.010 was fixed,
          check all BC-2.05.* for analogous gaps). This closes the Passes-33-37 sibling cycle.
      (g) any new design gap
    Instruct adversary: honest CLEAN verdict if spec is sound; only Critical/Important block;
    Suggestions do NOT block but apply "fix in-scope if Important-escalation risk" discipline.

  Step B — If CLEAN (0 Critical AND 0 Important):
    Increment consecutive-clean counter.
    counter < 3: dispatch next pass (counter now N; need 3 total).
    counter = 3: CONVERGENCE COMPLETE. Proceed to human spec-package approval gate.

  Step C — If FINDINGS (any Critical or Important):
    Reset consecutive-clean counter to 0.
    Route fixes by ownership:
      architect (vsdd-factory:architect): architecture specs, ADRs, wire types, error taxonomy
      product-owner (vsdd-factory:product-owner): BC files, holdout scenarios, PRD, vision, brief
      Architect + PO may run in PARALLEL when fix sets are independent.
    After fixes: orchestrator runs CLOSURE GREP (retired-term + value survivors + anchor-resolution).
    state-manager commits the round atomically (single commit per fix burst — see commit rules).
    Dispatch Pass N+1.

    SUGGESTION handling:
      Fix in-scope if suggestion carries Important-escalation risk (cross-doc/source-of-truth
      divergence, title↔body contradiction, anchor/cite inconsistency).
      DEFER as durable housekeeping if pure cosmetic (typo, §Trace-only changelog annotation)
      where the fix would trigger a disproportionate heavily-cited-doc bump+sweep cycle.

  COMMIT RULES FOR EVERY FIX ROUND (mandatory — hooks enforce these):
    - state-manager owns version-pin-registry.yaml; architect/PO REPORT bumps only
    - single atomic commit: spec files + version-pin-registry.yaml in SAME commit (L-S027-004)
    - TARGETED staging: name each file explicitly; NEVER git add -A; never stage code-delivery/
    - run POL-11: `python3 scripts/check_version_pins.py` from REPO ROOT (not worktree)
    - run POL-12: `python3 scripts/check_structural_claims.py` from REPO ROOT
    - verify registry diff line-count matches claimed bump count before committing
    - NEVER --no-verify; NEVER add Co-Authored-By: Claude or robot emoji

  CYCLE CHECKLIST (D-245 SUG-003 + D-246):
    After any RETIRED-term / corrected-value fix: grep -rn .factory/specs — every survivor
    must be in §Trace/changelog/enforcement context, NOT live PC/Invariant/title/doc-comment.
    After any BC Architecture-Source citation fix: verify the cited symbol EXISTS in the cited doc.

  ============================================================================
  C. COMMIT RULES — EXTENDED (POL-11-driven propagation sweep)
  ============================================================================

  After any SS/ADR/BC version bump, run a targeted grep to locate all stale-active literals
  of the old version string in spec prose (BC Architecture-Source citations are ACTIVE pins).
  Sweep stale-active literals to new versions; leave §Trace/version-pin-historical alone.
  Log the sweep scope in the commit message (file count + active-literal count cleared).

  Registry atomicity (L-S027-004, recurred 3×):
    If a BC or SS spec version is bumped → version-pin-registry.yaml MUST be updated in the
    SAME commit. Committing the spec file without the registry breaks CI (POL-11 checks out
    factory-artifacts at PR-open time). Checklist item before every push:
    [ ] registry diff line-count == claimed bump count; [ ] spot-check frontmatter of 2 bumped files.

  ============================================================================
  D. FULL SPEC PACKAGE FOR PASS 40+ (adversary receives this exact list)
  ============================================================================

  NOTE: All versions below DERIVED from version-pin-registry.yaml (source of truth).
  Feed this complete list to the adversary for every pass.

  L1 documents:
    domain-monocle-vision-synthesis.md v2.2.3  (APPROVED D-238; Created/Killed enum scrubbed D-267)
    product-brief.md v2.0.4                    (validate-brief VALID; Created/Killed scrubbed D-267)
    prd.md v1.28.3                             (§2.5 7-variant; §2.8 BC-2.08.008 + cross-check)
    ARCH-INDEX v1.0.28

  Architecture subsystem specs (delta docs — all v1A pivot additions):
    SS-ipc v1.23.2                        (23 IPC variants + SpawnAck; 12-code taxonomy; spawn_unsupported + session_not_ready wire codes; SpawnSession{opts}; #[ne] PermissionDecisionKind; F-P41+F-P44+F-P50+F-P51+F-P52 errata)
    SS-session-manager v2.6.0             (spawn_session(opts); SpawnOptions split; EngineError bridge; KeyInput error path; IPC handler UUID locus + SpawnAck send; UnsupportedOperation arm; SessionNotReady arm; host_conn.writer during Launching; proxy_task Option<JoinHandle<()>>; session_not_ready producer = detach-only; resize WARN-drop carve-out; §Terminated-in-grace defensive action×state matrix)
    SS-embedded-pty v1.6.0               (Launching step opts ref; App struct fields; auto-attach; Drag(MouseButton) arm; full Ps/modifier table; launching_session_id field)
    SS-engine-module-v2-delta v1.6.0     (SpawnOptions wire type #[ne]+Serialize/Deserialize+harness_id; SpawnRecipe daemon-internal; EngineError NEW canonical enum 3-variant; best-effort filtering note; SessionNotReady rationale note)
    SS-daemon-wiring-v2-delta v1.11.3    (daemon coordinator; IPC SpawnSession arm + 4-step SpawnAck sequence; ordered-pair-split rationale; ADR-0006 constructors; UnsupportedOperation error-table row; SessionNotReady error-table row; session_not_ready producer = detach-only errata; Terminated-in-grace action×state errata)
    SS-deps-pin-manifest-v2-delta v1.0.1 (portable-pty 0.9.0; vt100 0.16.2; tui-term =0.3.4)

  ADRs:
    ADR-0009 v1.0.2  (native detached session-host process; 5s deadline no-retry)
    ADR-0010 v1.6.0  (PTY bytes on UDS IPC; SpawnSession payload = opts; snapshot-then-resume)
    ADR-0011 v1.2.1  (PTY stack: portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88)

  Behavioral Contracts:
    BC-INDEX v1.41.0  (138 BCs total; 25 new v1A BCs)
    SS-03: BC-2.03.005 v1.1.3, BC-2.03.006 v1.1.1, BC-2.03.007 v1.2.2, BC-2.03.008 v1.0.3
    SS-05: BC-2.05.009 v1.5.2, BC-2.05.010 v1.9.1, BC-2.05.011 v1.2.1
    SS-06: BC-2.06.025 v1.5.0  (permission badge+bell during EmbeddedTerminal/SessionCreation; Launching action rules Invariant 5: kill ALLOWED / detach BLOCKED / rename allowed; Terminated-in-grace panel guard Invariant 6: k/D/r blocked; EC-300/301/302)
    SS-08: BC-2.08.001 v1.5.0, BC-2.08.002 v1.2.1, BC-2.08.003 v1.4.0, BC-2.08.004 v1.2.1,
           BC-2.08.005 v1.0.2, BC-2.08.006 v1.3.0, BC-2.08.007 v1.5.1, BC-2.08.008 v1.2.1
    SS-09: BC-2.09.001 v1.3.2, BC-2.09.002 v1.1.2, BC-2.09.003 v1.5.0, BC-2.09.004 v1.0.2,
           BC-2.09.005 v1.0.1, BC-2.09.006 v1.1.1, BC-2.09.007 v1.1.1, BC-2.09.008 v1.3.1,
           BC-2.09.009 v1.1.2

  Holdout scenarios:
    EVAL-INDEX v1.15
    HS-EXP-011..015 (5 v1A scenarios; HS-EXP-014 Step 1 corrected to Model-A at D-270)

  version-pin-registry.yaml at .factory/specs/version-pin-registry.yaml — source of truth

  ============================================================================
  E. RATIFIED DECISIONS (do NOT re-litigate — tell the adversary these are closed)
  ============================================================================

  SCOPE: v1A = Launch + EmbeddedPTY + Multi-session/multi-project + persistence + hook-auto-inject.
         v1B = Interactive Tune (BCs/stories authored when v1B scheduled; not in current spec package).

  PERSISTENCE MODEL (session-host-owns-PTY, D-238):
    Native monocle-session-host binary. setsid-detached. Per-session UDS. SO_PEERCRED universal.
    Re-discovery before UDS bind. Graceful daemon-process restart SURVIVES (CASE 2 = survive).
    Hard crash → lost → re-launch (CASE 3 unchanged). NO tmux default (external supervisor is
    architect-surfaced fallback only; requires explicit human decision).

  SPAWN-PATH MODEL A (Pass-27, D-269 — the major recent decision):
    daemon-side: spawn_recipe() called INSIDE spawn_session().
    ClientToServer::SpawnSession carries opts: SpawnOptions (WIRE type with #[non_exhaustive]+
    Serialize/Deserialize). SpawnOptions fields: project_root/worktree_root/harness_id/profile_id/
    ccr_base_url = TUI-populated; session_id/hooks_settings_path = daemon-filled.
    SpawnRecipe = DAEMON-INTERNAL (never on wire, never at call site).
    EngineError::BinaryNotFound→binary_not_found / InvalidPath→invalid_spawn_arg via
    SessionError::EngineError #[from] bridge reachable daemon-side (BC-2.03.007 PC-3/PC-7).

  KEYBOARD + MOUSE:
    Full keyboard fidelity: printable + control + arrows + Alt/Meta + mouse + Kitty CSI-u + bracketed-paste.
    Mouse SGR: px = col+1, py = row+1. Kitty modifier bitfield: shift=1, alt=2, ctrl=4.
    Permission badge + per-prompt bell during EmbeddedTerminal.

  PTY STACK (ADR-0011):
    portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4 (ratatui 0.30 compatible; MSRV 1.88).
    Per-session pty_scroll_offsets HashMap (~16 bytes/cell). PTY backpressure cap 64 no-silent-drop + PtyReset.
    Terminating state + 12s watchdog.

  IPC SCHEMA:
    schema_version 3. Chunked scrollback: snapshot-then-resume (session-host NOT paused during dump).
    SessionSnapshot/SerializedCell/SerializedColor in monocle-ipc.
    ClientToServer::AttachSession re-attach. 9-variant SessionError (no AlreadyAttached/AttachTimeout;
    SessionNotReady added at Pass-50/D-293 for defensive control-ops on not-ready sessions).
    attach already-attached → idempotent Ok(). attach-timeout → SessionHostDead/attach_failed.
    RenameSession empty name → rename_failed. 12-code wire taxonomy (session_not_ready added at Pass-50/D-293;
    spawn_unsupported added at Pass-44/D-287; 10-code EngineError base taxonomy).

  SESSION LIFECYCLE STATES: Launching/Running/Detached/Terminating/Terminated.
    Created and Killed are RETIRED (removed at BC layer D-241; removed from L1 prose D-267).

  SECURITY: daemon UDS 0o600 + dir 0o700. Per-session SO_PEERCRED. #[non_exhaustive] on wire
  types except SerializedColor + TransportEvent (intentional documented exclusions).

  CONCURRENT MULTI-TUI-CLIENT: ratified FUTURE scope boundary per BC-2.05.009 Invariant 2
  (TD-MULTI-CLIENT-ATTACH-STORM-001). v1A = single TUI client or first-win semantics. Not a v1A defect.

  ENGINEERROR (Pass-31, D-273 — tell adversary this is CLOSED):
    EngineError is a NEW v1A #[non_exhaustive] enum declared in SS-engine-module-v2-delta
    §EngineError (new in v1A). Three variants: UnsupportedOperation/BinaryNotFound/InvalidPath.
    INDEPENDENT of SpawnError/PreflightError/EngineMetadataError (those are pre-existing).
    Inner cross-crate match on EngineError REQUIRES _=> (documented as forward-compat fallback,
    not a silent swallow — BC-2.03.007 Invariant-1 arm distinction codified).

  ADR-0006 CONSTRUCTORS (Pass-30, D-272 — tell adversary this is CLOSED):
    All v1A #[non_exhaustive] wire structs have ADR-0006-compliant constructors and audit-table rows.
    SpawnOptions: for_spawn_request() (TUI-side) + with_daemon_fields() (daemon-side).
    SpawnRecipe/SessionSnapshot/SerializedCell/PermissionPromptPayload: new().
    E0639 ..opts bypass workaround documented per ADR-0006 §Rationale.

  DEAD-ANCHOR REMEDIATION (Pass-33, D-275 — tell adversary this is CLOSED):
    Whole-class dead-anchor remediation via explicit <a id="anchor-name"> navigational anchors.
    NO version bumps for navigational-only anchor additions (C29-001 errata precedent).
    POL-13 (scripts/check_cross_ref_anchors.py) enforces anchor validity in CI + pre-commit.

  TWO-PRONGED INVALIDPATH DETECTION (Pass-34, D-276 — tell adversary this is CLOSED):
    InvalidPath detection uses two prongs: (1) to_str() returns None for non-UTF-8 paths;
    (2) as_bytes().contains(&0) for null bytes (null is valid UTF-8, so to_str() alone misses it).
    Both conditions independently classify as InvalidPath. Documented in BC-2.03.007 Invariant-1.

  MOUSE SGR ENCODING (Pass-35, D-277 — tell adversary this is CLOSED):
    Down/Up buttons: Ps 0/1/2 (Left/Middle/Right). Drag: 32/33/34. Moved: Ps=35 (not 32;
    unreachable on Unix when 1002 not enabled). ScrollUp/ScrollDown: 64/65. ScrollLeft/Right:66/67.
    Modifier bits: Shift |= 4, Alt |= 8, Ctrl |= 16. Terminator: 'M' for Press/Drag/Moved;
    'm' for Release. BC-2.09.003 v1.4.0. SS-embedded-pty v1.5.2.

  ORDERED-PAIR-SPLIT (Pass-35, D-277 — tell adversary this is CLOSED):
    Ordered-pair-split (half-message disconnect) triggers immediate client disconnect
    INDEPENDENT of slow-client 3-strike counter. These are two distinct error classes
    (framing integrity vs throughput). BC-2.08.008 v1.1.1. SS-daemon-wiring-v2-delta v1.10.0.

  NO-SILENT-FAILURE INVARIANT BC-2.05.010 (Pass-36, D-278 — tell adversary this is CLOSED):
    BC-2.05.010 now carries an explicit named "No-silent-failure invariant": every fallible
    operation (KeyInput/Detach/Kill/Rename/Attach/Resize) maps to a documented error code.
    ResizePane is the only documented WARN-drop exception (arch constraint). BC-2.05.010 v1.8.0.
    Whole-class sweep of ALL fallible variants completed (Kill/Rename/Attach error-code docs too).

  MOUSE SGR MOVED REACHABILITY (Pass-36, D-278 — tell adversary this is CLOSED):
    BC-2.09.003 Invariant-3 updated: Ps=35 (Moved) IS reachable under X10 extended-mouse mode
    1003 (not only 1002). Unreachable-on-Unix claim retired. BC-2.09.003 v1.5.0.

  MOUSE CAPTURE SCOPE BC-2.09.002 (Pass-37, D-279 — tell adversary this is CLOSED):
    BC-2.09.002 Invariant-5 updated: mouse capture is SCOPED (EmbeddedTerminal entry/exit)
    not globally-active. The prior "globally-active" assertion was a partial-fix sibling of
    the I2-001/I3 scoped-capture fix. BC-2.09.002 v1.1.2.

  SCOPED MOUSE-CAPTURE SEQUENCES BC-2.09.008 (Pass-38, D-280 — tell adversary this is CLOSED):
    BC-2.09.008 PC-4/PC-1 rewritten with full two-step sequences (S38-001 fixed in-scope).
    Entry: (a) crossterm::execute!(stdout(), EnableMouseCapture)? + (b) print!("\x1b[?1006h").
    Exit: (a) print!("\x1b[?1006l") + (b) crossterm::execute!(stdout(), DisableMouseCapture)?.
    Both cross-reference BC-2.09.002 Invariant-5 as authoritative owner. BC-2.09.008 v1.2.0.
    Whole-class sweep of SS-09 + SS-05 confirmed zero other partial-restatement survivors.

  GLOBAL→SCOPED MOUSE-CAPTURE PROSE (Pass-39, D-281 — tell adversary this is CLOSED):
    SS-embedded-pty:250 corrected: keyboard enhancement flags are global at TUI startup;
    EnableMouseCapture is scoped to EmbeddedTerminal entry/exit per BC-2.09.002 Invariant-5.
    Whole-class architecture+BC sweep complete; zero other live survivors confirmed.
    The ONLY allowed "global mouse capture" references remaining are §Trace/changelog history
    entries and the unadopted-option UX-tradeoff description (SS-embedded-pty:366/370).
    Mouse-capture scope-contradiction class permanently closed. Errata-no-bump (v1.5.2 unchanged).

  SPAWN session_id HANDSHAKE (Pass-41, D-283 — tell adversary this is CLOSED):
    Daemon IPC handler generates UUID (uuid::Uuid::new_v4().to_string()) BEFORE spawn_session() call.
    Sends point-to-point ServerToClient::SpawnAck{session_id} to requesting client only.
    ORDERING GUARANTEE (D-285 CV-SS-005 — tell adversary this is CLOSED):
      SpawnAck-before-SessionStateChanged{Launching} is justified by TWO properties:
      (1) Causal step ordering: IPC handler sends SpawnAck (step 2) BEFORE calling spawn_session()
          (step 4), which is BEFORE the broker emits SessionStateChanged{Launching} (step 5).
      (2) Per-client FIFO: all ServerToClient messages to a given client travel on the same
          ordered channel, so SpawnAck(step 2) arrives before Launching(step 5) by FIFO.
    TUI stores launching_session_id: Option<String> on AppMode::SessionCreation.
    Auto-advance to EmbeddedTerminal matches SessionStateChanged{Running}.session_id against
    launching_session_id (deterministic; no broadcast-race). spawn_session() receives caller-supplied
    session_id (daemon IPC handler sets it; spawn_session() does NOT generate it).
    SPAWN-FAIL PATH (D-285 CV-SS-003/CV-SS-004 — tell adversary this is CLOSED):
      On spawn failure: wizard Step 4 clears launching_session_id → returns to named step
      ProfilePicker. The return target is ALWAYS named ProfilePicker, NOT generic "Step 1".
      Canonical across SS-ipc / SS-embedded-pty / SS-session-manager / BC-2.08.008 / BC-2.09.008.
    Only SpawnSession requires a correlated ack. Kill/attach/detach/rename use TUI-known ids —
    correct by design. Spec: SS-ipc v1.22.0, SS-embedded-pty v1.6.0, SS-session-manager v2.4.0,
    BC-2.08.008 v1.2.1, BC-2.09.008 v1.3.1.
    CANONICAL wizard storage field (Pass-42, D-284 — tell adversary this is CLOSED):
    The canonical field name everywhere is AppMode::SessionCreation.launching_session_id.
    The orphan name wizard_session_id was an incorrect reference that survived in SS-ipc body
    prose + §Trace errata + doc-comments + SS-session-manager skeleton comment after Pass-41.
    All 5 live sites corrected at D-284 via architect errata-no-bump burst. wizard_session_id
    is RETIRED — do NOT reintroduce or re-flag. Whole-class grep confirmed zero survivors.

  SPAWN HANDSHAKE COMPLETE (Pass-43, D-286 — tell adversary this feature class is CLOSED):
    SpawnAck step is now present in ALL THREE normative IPC-handler representations:
      (1) SS-session-manager §IPC handler (canonical; primary normative skeleton)
      (2) SS-ipc §SpawnAck delivery-ordering (SpawnAck ordering guarantee prose)
      (3) SS-daemon-wiring-v2-delta §3 SpawnSession arm (duplicate skeleton; now synced)
    Spawn-handshake feature complete across all 7 docs that carry normative spawn-handshake
    content: SS-ipc, SS-session-manager, SS-embedded-pty, SS-daemon-wiring-v2-delta,
    BC-2.08.001, BC-2.08.008, BC-2.09.008. Root cause of F-P43: CANONICAL PATTERN LOCK
    co-propagation (D-283/D-285) updated SS-session-manager and SS-ipc but missed the
    duplicate IPC-handler skeleton in SS-daemon-wiring-v2-delta §3 (structural duplication
    was the failure mode). Architect recommendation DEDUP-IPC-HANDLER-SKELETON added to
    durable_task_register (non-blocking for Phase-1d convergence; both skeletons now in sync).
    SS-daemon-wiring-v2-delta v1.10.0.

  ENGINEERROR WIRE-CODE TAXONOMY (Pass-44, D-287; extended Pass-50, D-293 — CLOSED):
    Wire-code taxonomy: 10 base codes + spawn_unsupported (11th, D-287) + session_not_ready (12th, D-293).
    All 3 EngineError variants now map to dedicated wire codes:
      BinaryNotFound → binary_not_found   (closed I12-001)
      InvalidPath    → invalid_spawn_arg  (closed F-P34-IMP-001)
      UnsupportedOperation → spawn_unsupported  (closed F-P44-IMP-001)
    Banner for spawn_unsupported: "Session spawn not supported for this harness".
    EC-112 is REACHABLE: ProfilePicker capability filtering is best-effort; a harness that
    reports 0 capable profiles can still be selected and will return UnsupportedOperation.
    SessionError::SessionNotReady (9th variant, D-293): defensive code for control-ops (Kill/Attach/
    Detach/Rename) on sessions not yet in a control-ready state. Wire code: session_not_ready.
    Semantically distinct from invalid_request (valid variant, wrong timing) per untrusted-client
    wire posture; no-silent-failure principle requires explicit code (not catch-all).
    The inner _=> arm in session_error_to_code is forward-compat ONLY for FUTURE SessionError variants.
    Spec: SS-ipc v1.23.2, SS-session-manager v2.6.0, SS-engine-module-v2-delta v1.6.0,
    SS-daemon-wiring-v2-delta v1.11.3, BC-2.08.003 v1.4.0, BC-2.05.010 v1.9.1,
    BC-2.08.007 v1.5.1, BC-2.03.008 v1.0.3, BC-INDEX v1.41.0.
    Do NOT re-flag the EngineError→wire-code mapping, SessionNotReady variant, or _=> arm as gaps.

  SESSION_NOT_READY PRODUCER = DETACH-ONLY (Pass-51, D-294 — tell adversary this is CLOSED):
    session_not_ready has ONE live wire producer: the DetachSession IPC arm for sessions not yet
    in a control-ready state (Launching, host_conn None). This is the DEFENSIVE/UNTRUSTED-CLIENT
    path. The official TUI never sends DetachSession during Launching (BC-2.06.025 Invariant 5
    enforces the TUI guard). Resize is EXCLUDED: ResizePane WARN-drops ALL errors per BC-2.05.010
    Invariant 6 Exception (architectural constraint; does not violate no-silent-failure because
    resize failure is a best-effort advisory). KillSession on not-ready → kill_failed (PID fallback
    per BC-2.08.003, not session_not_ready). AttachSession on Launching → attach_failed (session
    host not yet answering; already documented). RenameSession works on all states. Do NOT re-flag
    resize-omission from session_not_ready as a gap. Spec: SS-session-manager v2.6.0 (mapping table
    + step-8 corrected to detach-only); BC-2.05.010 v1.9.1 (§DetachSession PC-4 + §ResizePane
    Invariant 6 Exception); BC-2.08.007 v1.5.1 (detach defensive-branch note).

  HOOKS-SETTINGS.JSON FILE SCHEMA (Pass-53, D-296 — tell adversary this is CLOSED):
    The v1A daemon-written hooks-settings.json has exactly 6 keys:
      4 URL-bearing keys: "PreToolUse", "Notification", "Stop", "UserPromptSubmit"
        (each → curl POST http://127.0.0.1:<daemon_port>/hooks/<endpoint> with X-Monocle-Authorization)
      2 reserved-empty keys: "PostToolUse": [] and "PreCompact": [] (forward-compat placeholders)
      "SessionStart" is NOT a key in this file. Claude Code fires session-start via its own internal
        lifecycle; monocle's axum router serves POST /hooks/session-start but this is NOT configured
        through hooks-settings.json.
      "lock": { "app": "monocle" } (filter — ensures only monocle-launched sessions route here)
    Distinction: 5 served HTTP endpoints (pre-tool-use, notification, stop, session-start, prompt-submit)
      vs 4 URL-bearing JSON keys + 2 empty JSON keys in the file. BC-2.08.006 PC-3 authority = BC-2.04.010
      PC-3 (NOT BC-HOOK-007, which governs the DTU gene-source clone's WriteHooksSettingsFile — different
      schema: 5 URL keys including SessionStart, no PreCompact). Do NOT re-flag the served-endpoint count
      (5) vs file-key count (4 URL + 2 empty) as a gap; this distinction is now explicitly documented.

  TERMINATED-IN-GC-GRACE ACTION×STATE MATRIX (Pass-52, D-295 — tell adversary this is CLOSED):
    Terminated sessions remain in the registry for 10s GC grace period, render [X], and may
    receive panel keystrokes. The complete action×state disposition for Terminated is:
      rename (r) on Terminated: → Err(InvalidSessionName{"session terminated"}) → rename_failed
                                  wire code. No new variant. Backs BC-2.08.005 Invariant 4
                                  (revive-not-allowed: rename semantics cannot differ across states).
      detach (D) on Terminated: → idempotent Ok(). No-op; session has no active proxy task.
                                  No wire code emitted. TUI state unchanged.
      kill (k/d) on Terminated: → idempotent Ok() per BC-2.08.003 Invariant 2 (kill on already-
                                  Terminating/Terminated is a no-op). No new wire code.
      resize on Terminated:     → WARN-drop (same as all states per BC-2.05.010 Inv 6 Exception).
    No new SessionError variants. No new wire codes. Launching rules (Invariant 5) and 9-variant/
    12-code taxonomy counts preserved. SS-session-manager v2.6.0 §Terminated-in-grace matrix closes
    ALL action×state cells. BC-2.06.025 v1.5.0 Invariant 6 + EC-300/EC-301/EC-302 + VPs.
    BC-2.08.005 v1.0.2 Inv 4 now backed by rename_failed wire code.
    Do NOT re-flag Terminated action×state coverage as a gap.

  BC-2.06.025 LAUNCHING ACTION RULES (Pass-51, D-294 — tell adversary this is CLOSED; sibling of F-P50-001):
    BC-2.06.025 v1.5.0 (supersedes v1.4.0) Invariant 5 establishes the complete Launching-phase action table:
      kill:   ALLOWED — uses host_conn.writer (CONTROL connection established during Launching per
              F-P50-001) or PID fallback; per BC-2.08.003 Invariant 3. EC-298 added.
      detach: BLOCKED / no-op — session is not yet control-ready; response is session_not_ready
              (defensive/untrusted-client path). EC-299 added. TUI guard in BC-2.06.025 Invariant 5
              ensures official TUI never sends DetachSession during Launching.
      rename: ALLOWED — rename is state-agnostic in the registry; no host interaction required.
    All 'BC-2.06.025 guards' citations in BC-2.05.010 and BC-2.08.007 now resolve to Invariant 5.
    Zero surviving unresolved 'guards' citations confirmed. Do NOT re-flag this feature class.

  OBS-1 BC-2.09.002 §TRACE LINE-NUMBERS (Pass-40, D-282 — tell adversary this is a RATIFIED LOW deferral; do NOT re-flag):
    BC-2.09.002 Inv-5 §Trace cites SS-embedded-pty lines 254-258 for the ENTRY sequence;
    actual ENTRY block is at lines 279-289. This is a navigational stale-line-number only.
    Section-name anchors resolve correctly; POL-13 green. Classified as deferred housekeeping
    per the closed dead-anchor/navigational-reference precedent (C29-001 / D-275). No version bump.
    Fix is deferred to the next substantive BC-2.09.002 edit. NOT a normative defect.

  INVALID_REQUEST TRIGGER SS-ipc (Pass-37, D-279 — tell adversary this is CLOSED):
    SS-ipc §InvalidRequest description corrected: invalid_request is a POST-CALL catch-all
    for unhandled ClientToServer variants (always-matched final arm), NOT a pre-call guard.
    Prose-only correction; wire contract unchanged; no version bump.

  VERSION-LESS §ARCHITECTURE ANCHORS (Pass-46, D-289 — CLOSED; tell adversary do NOT re-flag version-less anchors as missing pins):
    §Architecture Anchors sections in BC files are NAVIGATIONAL ONLY — they carry no version pins.
    Authoritative version pins live exclusively in §Architecture Source rows (the POL-11-enforced table).
    The old parenthetical form `path#anchor (vX.Y.Z)` in §Architecture Anchors was a corpus drift:
    these 11 pins were stale relative to §Architecture Source and invisible to POL-11's ID↔version
    adjacency regex. FIXED at D-289 (BC-2.08.001/008, BC-2.09.008; errata-no-bump).
    POL-14 (develop @ 5d9d603) now detects+validates any parenthetical anchor-pin form.
    Do NOT re-flag a version-less §Architecture Anchors citation as a missing version pin.

  HOLDOUT-PROSE LOW DEFERRALS (Pass-45, D-288 — tell adversary S-P45-001 and S-P45-002 are RATIFIED LOW deferrals; do NOT re-flag):
    S-P45-001: HS-EXP-014 line ~46 illustrative session-state.json field list omits child_pid.
    FAIL criterion at line ~200 only checks hook_settings_path; non-normative illustrative prose.
    S-P45-002: HS-EXP-013 step-9 prose says "most recently added" vs BC-2.09.009 PC-5c canonical
    "front=oldest". Prose is hedged and PASS-criterion is independent of display order;
    non-normative holdout prose that does not contradict a normative BC behaviorally.
    Both deferred to the Phase-4 holdout-evaluation prep pass as OBS-HS-PROSE-PHASE4-PREP.

  ============================================================================
  F. CODIFIED LESSONS (enforce every pass and every fix round)
  ============================================================================

  L-S027-004 REGISTRY ATOMICITY (recurred 3×): architect/PO REPORT bumps; state-manager
  OWNS registry; verify diff line-count + frontmatter spot-check before every commit.

  L-CWD-PROPAGATION-ATTESTATION (D-255, recurred 3×): changelogs must NOT attest
  propagation completeness without a whole-file fix-completeness grep. Partial propagation
  labelled "complete" recurred 3× and caused fresh Important findings in subsequent passes.

  L-VERIFICATION-ARTIFACT-FALSE-GREEN (D-258): cross-check artifacts that HAND-ENUMERATE
  expected sets (e.g. "BC-2.08.001..007") create self-falsifying false-greens. RULE: derive
  expected counts/sets from BC-INDEX §Summary; never hand-type an expected enumeration.

  PROPAGATION-CLOSURE AUDIT: after split architect+PO fix rounds, run a closure check
  confirming EVERY delegated sync landed in EVERY target file before the next pass.

  ANCHOR-RESOLUTION CLOSURE: after any BC Architecture-Source citation fix, verify the cited
  symbol EXISTS in the cited doc before committing.

  RETIRED-CONCEPT GREP SWEEP: after any RETIRED-term / corrected-value fix, grep
  .factory/specs; each survivor must be §Trace/enforcement context only.

  TARGETED STAGING: name each file explicitly. Never git add -A for factory-artifacts commits.
  Never stage code-delivery/ dirs.

  FIX-THE-WHOLE-CLASS: each Important fix must sweep its sibling class. Examples: worktree_root
  (I13), crate-pin form (I14+I19), ADR pin-symmetry (I17), L1 lifecycle prose (I25),
  #[non_exhaustive] wire types (I26), Model-B spawn residue (I27+I28), dead-anchor class (I33-001).

  POL-13 ANCHOR-LINT (D-275, CLOSED): cross-reference anchor validation enforced by
  scripts/check_cross_ref_anchors.py in CI + pre-commit. After any architectural anchor add/rename,
  run `python3 scripts/check_cross_ref_anchors.py` from repo root before committing.

  NO-VERSION-BUMP FOR NAVIGATIONAL ANCHORS (D-275, per C29-001 errata precedent): adding
  explicit <a id> navigational anchors to spec documents to satisfy POL-13 does NOT require a
  version bump. These are navigational aids, not normative content changes.

  PARTIAL-FIX-SIBLING MITIGATION (D-278/D-279, Passes 33-37 pattern — codified):
  Recurring pattern: each fix round exposes a sibling gap in the same subsystem in the next pass.
  After every fix burst the owning agent MUST grep all BC/SS docs in the SAME subsystem for the
  same pattern and reconcile ALL instances in ONE burst. The adversary MUST be explicitly told
  each pass to hunt for siblings of the prior round's fix. Do NOT declare a fix-class closed
  without a whole-subsystem grep confirming no survivors.

  PROACTIVE-CONSISTENCY-SWEEP (D-285 — codified):
  When a single new cross-doc FEATURE accrues >= 2 consecutive Important adversarial findings
  (especially self-siblings of its own fixes), inject a fresh-context consistency-validator
  scoped to that feature's whole doc-surface BEFORE the next adversarial pass. Flush the entire
  sibling class in one burst instead of discovering it one-per-adversary-pass (~250K tokens/cycle
  saved). Trigger: >= 2 consecutive Importants on the same feature cluster in the `awaiting` log.

  POL-COVERAGE-BLIND-SPOT (D-289 — codified):
  A lint that keys on a specific literal FORM can be silently blind to a sibling form. Here,
  POL-11's ID↔version adjacency regex correctly matched `SS-foo v1.2.3` style pins but MISSED
  the `path#anchor (vX.Y.Z)` parenthetical form in §Architecture Anchors sections — leaving 11
  stale pins undetected across 3 BCs for multiple passes. The form looked like prose decoration
  but was semantically a version pin. RULE: when a fresh-context finding reveals a stale pin or
  claim that "passed CI", verify the lint ACTUALLY COVERS that form; if it does not, extend the
  lint (POL-14 precedent on develop @ 5d9d603 adds Pattern C to check_version_pins.py) rather
  than merely fixing the instance. Fixing the instance without extending the lint leaves the
  process gap open for recurrence.

  ============================================================================
  G. REMAINING TASKS (in order)
  ============================================================================

  1. FINISH Phase-1d adversarial convergence (CURRENT STEP):
     53 passes done → need 3 CONSECUTIVE clean (counter = 0). Pass-54 = clean candidate 1 of 3.
     Drive to strict 3-clean per human directive. Apply PARTIAL-FIX-SIBLING mitigation every round.

  2. Human spec-package APPROVAL GATE (after convergence):
     Run /vsdd-factory:check-input-drift first.
     Present structured review questions: scope completeness, anchor correctness, coverage gaps.
     Gate items (non-blocking until story wave):
       CC-TUITERM-WIP-SIGNOFF (tui-term 0.3.4 WIP risk-acceptance, ADR-0011 §O2)
       CC-GLOBAL-MOUSE-CAPTURE (mouse capture scope — currently EmbeddedTerminal entry/exit only)

  3. Phase 2 STORY DECOMPOSITION (vsdd-factory:story-writer):
     Decompose v1A delta into stories + waves + dependency graph.
     RESOLVE all S-TBD story anchors in the 25 v1A BCs + holdout stories_tested fields.

  4. VP AUTHORING (vsdd-factory:architect) — DEFERRED to formal-hardening per VP-DTU-001
     pattern: all 25 v1A BCs cite VP-TBD; create VPs + VP-INDEX coverage at hardening.

  5. Pre-Phase-3 prerequisites:
     DTU clone: S-DTU-001 (dtu-claude-code-hooks-v1) EXISTS, fidelity 1.0 (D-234) — UNBLOCKED.
     CI/CD verification (PROC-BRANCH-PROTECTION-CONTEXTS open — human required).

  6. Phase 3 TDD IMPLEMENTATION of v1A stories (per-story-delivery.md + wave gates).
     v1B Tune BCs/stories authored when v1B scheduled (not now).

  7. PARKED HUMAN ITEMS (required before v1A story wave, not before convergence):
     CC-TUITERM-WIP-SIGNOFF. CC-GLOBAL-MOUSE-CAPTURE. v1B pre-emption ratification.

  8. Pre-existing durable_task_register backlog (ADV-W5GATE-HIGH-002 dead code,
     ADV-W5GATE-MED-001/003, S28-001 §Trace housekeeping, etc.) — non-blocking, subordinate
     to pivot; revisit during v1A implementation waves.

  ============================================================================
  H. DURABLE TASK REGISTER SUMMARY (non-blocking items — see full register above)
  ============================================================================

  S28-001:  SS-session-manager §Trace v2.0.0 Fix(c) cross-ref inaccuracy — SpawnRecipe derives
            retained not removed; fold into next substantive SS-session-manager edit. PENDING.
  ADR-0010-TRACE-256-MARKER (S15-001): ADR-0010 §Trace v1.2.0 stale 'capacity 256' no inline
            superseded-by marker; fold into next ADR-0010 normative edit. PENDING.
  SS-DAEMON-WIRING-SCROLLBACKDUMP-TERM (S19-001): bare 'ScrollbackDump' term in §Trace prose;
            fold into next SS-daemon-wiring-v2-delta normative edit. PENDING.
  ADR-0011-UPGRADE-TYPO (S20-002): doubled 'upgrade upgrade' word at line ~150; fold into next
            ADR-0011 normative edit. PENDING.
  DEP-PIN-SWEEP-RULE (S14-001): extend POL-11 to grep crate-pin literals vs SS-deps-pin. PENDING.
  PRD-COUNT-CROSSCHECK-RULE (S16-001): add structural-claim check for BC count per SS-NN. PENDING.
  TD-MULTI-CLIENT-ATTACH-STORM-001: concurrent multi-TUI-client = ratified FUTURE (v1B+). PENDING.
  DTU-CLONE-STORY: RESOLVED-FALSE-PREMISE (D-234). Phase 4 UNBLOCKED.
  CC-TUITERM-WIP-SIGNOFF: human risk-acceptance of tui-term 0.3.4 WIP-upstream before story wave.
  CC-GLOBAL-MOUSE-CAPTURE: human approval if future story needs clickable monocle panels.
  ENGINE-RS-DEVERSION: engine.rs doc-comment version pins should be de-versioned per CI-PARITY rule 3.
  BC-INDEX-TRACE-SS08-COUNT: §Trace stale 'SS-08 ... 7 BCs' (actual 8); fold into next BC-INDEX edit.
  SS-IPC-181-REDUNDANT-HISTORICAL-MARKER: line ~181 pin (v1.9.1) now STALE after D-286/D-287/D-293 bumps (SS-daemon-wiring-v2-delta now v1.11.1); POL-11-exempt due to historical marker — needs manual update to v1.11.1 or version-free form. PENDING.
  OBS-1-BC2.09.002-TRACE-LINE-NUMBERS: BC-2.09.002 §Trace stale line-numbers 254-258 vs actual ENTRY block 279-289; fold into next substantive BC-2.09.002 edit. DEFERRED-HOUSEKEEPING.
  DEDUP-IPC-HANDLER-SKELETON: de-duplicate SS-session-manager §IPC handler (canonical) and SS-daemon-wiring-v2-delta §3 (mirror); duplication caused F-P43. Schedule before/with Phase-2. PENDING-ORCHESTRATOR-ACTION.
  OBS-HS-PROSE-PHASE4-PREP: two LOW holdout-prose imprecisions (HS-EXP-014:46 child_pid omitted; HS-EXP-013:54 step-9 display-order prose vs BC-2.09.009 PC-5c); deferred to Phase-4 holdout-eval prep. DEFERRED-PHASE4-PREP.

  ============================================================================
  I. DECISION LOG ENTRIES (D-271..D-287)
  ============================================================================

  D-271 (2026-06-13): Adversarial Pass-29 = FINDINGS (1C/0I/0S).
  C29-001 (harness_id field missing from SpawnOptions struct definition in SS-engine-module-v2-delta
  — field was referenced in §SpawnOptions-fields table but absent from the struct block). Fixed as
  errata (no version bump; treated as C29-001 editorial correction per no-bump-for-navigational
  precedent). factory-artifacts committed at 5ea7395. Counter=0; Pass-30 next.

  D-272 (2026-06-13): Adversarial Pass-30 = FINDINGS (2C/1I).
  C30-001/C30-002 + I30-001 (ADR-0006 cross-crate #[non_exhaustive] constructor gap — SpawnOptions,
  SpawnRecipe, SessionSnapshot, SerializedCell, PermissionPromptPayload all missing constructors +
  audit-table rows; E0639 ..opts workaround not documented). Fixed: constructors added to all 5 types;
  for_spawn_request()+with_daemon_fields() for SpawnOptions; new() for remaining 4; audit-table rows
  added for each; E0639 bypass documented. committed ce7868c. Counter=0.

  D-273 (2026-06-13): Adversarial Pass-31 = FINDINGS (1C/1I).
  P31-CRIT-001 (EngineError enum referenced throughout spec package as a type but never declared
  — referenced as if existing but absent from all SS docs). Fixed: declared as new canonical
  #[non_exhaustive] enum in SS-engine-module-v2-delta §EngineError (new in v1A): 3 variants
  (UnsupportedOperation/BinaryNotFound/InvalidPath), independent of pre-existing error types.
  P31-HIGH-001 (SpawnRecipe parity gap in SS-session-manager — spawn_recipe() doc referenced fields
  not defined in the SpawnRecipe block). Fixed: SS-session-manager v2.2.0 updated.
  committed 927ff29. Counter=0.

  D-274 (2026-06-13): Adversarial Pass-32 = FINDINGS (0C/3I).
  IMP-001 (session_error_to_code _=> arm prose described it as a 'panic fallback' contradicting the
  'forward-compat swallow' framing elsewhere — self-contradiction). Fixed: arm documented as
  forward-compat return of internal_error code (no panic). IMP-002 (dead anchor #engineerror-additions
  in several docs → fixed to #engineerror-new-in-v1a). IMP-003 (EngineError described as 'extension
  of existing error types' — incorrect framing; it is a new independent enum). Fixed: all three.
  committed b9d3591. Counter=0.

  D-275 (2026-06-13): Adversarial Pass-33 = FINDINGS (0C/2I).
  I33-001 (PERVASIVE dead-anchor class — 70 cross-reference anchors across 12 docs resolved to
  nothing; whole class undetected because no tooling existed). Human directive: WHOLE-CLASS-NOW +
  STABLE-EXPLICIT-ANCHORS. Built scripts/check_cross_ref_anchors.py (POL-13); wired CI + pre-commit.
  Lint found 70 dead anchors; architect added 42 explicit <a id> navigational anchors across 12 docs
  (NO version bumps for navigational-only additions). I33-002 (§Trace over-claim in several docs
  attested 'propagation complete' without full sweep). PO fixed 2 defective citations:
  BC-2.08.001 typo and BC-2.06.016 path. ANCHOR-LINT-TOOL durable item CLOSED.
  factory-artifacts committed ef5b1ec; develop committed e9e9103 (CI wiring). Counter=0.

  D-276 (2026-06-13): Adversarial Pass-34 = FINDINGS (1C/1I).
  C34-001 (InvalidPath null-byte detection impossible via to_str() alone — null byte is valid UTF-8
  so to_str() does NOT return None for paths containing null bytes; to_str() only catches non-UTF-8).
  Fixed: two-pronged detection: to_str().is_none() for non-UTF-8 AND as_bytes().contains(&0) for
  null bytes. Both independently classify as InvalidPath. BC-2.03.005/006/007/008 updated.
  I34-001 (BC-2.08.001 stale anchor version labels in §Architecture-Source — labels still cited
  versions prior to Pass-33 anchor additions). Fixed: labels updated. committed 8fab81c. Counter=0.

  D-277 (2026-06-13): Adversarial Pass-35 = CLEAN (0C/0I) — FIRST CLEAN of session.
  3 Suggestions fixed in-scope per production-grade principle:
  S35-001 (split-pair rationale arithmetic contradiction in SS-daemon-wiring-v2-delta and BC-2.08.008
  — the stated disconnection trigger was inconsistent with the actual byte-pair invariant). Fixed.
  S35-002 (KeyInput→SessionHostDead→attach_failed error path in BC-2.05.010 was undocumented —
  session going dead during keystroke forwarding had no defined error-code mapping). Fixed.
  S35-003 (mouse SGR encoding: Drag(MouseButton) match arm MISSING from BC-2.09.003; Moved was
  documented as Ps=32 but canonical is Ps=35; full Ps/modifier table absent). Fixed: Drag(Left/
  Middle/Right) → Ps 32/33/34; Moved → Ps=35; full modifier table (Shift=4/Alt=8/Ctrl=16).
  SS-daemon-wiring-v2-delta v1.9.1, SS-embedded-pty v1.5.2, SS-session-manager v2.2.1,
  SS-ipc v1.20.1, SS-engine-module-v2-delta v1.4.1. committed a0d5720.
  Suggestions fixed in-scope → package changed → CONSECUTIVE-CLEAN COUNTER RESET to 0.
  Pass-35 CLEAN does NOT count toward the 3-clean streak. Pass-36 = clean candidate 1 of 3.
  Durable zero-context checkpoint written at human request (next_session_resume_protocol v7.28).

  D-278 (2026-06-13): Adversarial Pass-36 = FINDINGS (0C/2I).
  F-P36-IMP-001 (BC-2.09.003 Invariant-3 contradicted the S35-003 Moved-unreachable model —
  Invariant-3 still asserted Ps=35/Moved was unreachable on Unix, but the S35-003 fix introduced
  Ps=35 reachable under 1003 mode, making Invariant-3 a direct contradiction). Fixed: Invariant-3
  updated; Moved documented as reachable under 1003. BC-2.09.003→1.5.0.
  F-P36-IMP-002 (BC-2.05.010 missing named "No-silent-failure invariant" + Detach/Resize sibling
  error-path gaps — S35-002 documented KeyInput only; Detach/Resize had no error-path mapping;
  arch forward-refs SS-session-manager:385/SS-ipc:389,1515 were dangling). Fixed: named invariant
  authored (explicit "No-silent-failure invariant" section); whole-class sweep of ALL fallible
  variants (KeyInput/Detach/Kill/Rename/Attach documented); ResizePane documented as WARN-drop
  exception (arch constraint). BC-2.05.010→1.8.0. PO-only fix burst. Committed 33dea39.
  Counter stays 0. Pass-37 next.

  D-279 (2026-06-13): Adversarial Pass-37 = FINDINGS (0C/2I).
  F-P37-IMP-001 (BC-2.09.002 Invariant-5 still asserted globally-active mouse capture —
  partial-fix sibling of the I2-001/I3 scoped-capture fix; the Pass-36 fix of BC-2.09.003
  had correctly updated the encoding table but incorrectly dismissed BC-2.09.002's sibling
  Invariant-5, which still claimed mouse events captured in ALL TUI states not just
  EmbeddedTerminal). Fixed: BC-2.09.002 Invariant-5 updated to scoped-capture model.
  BC-2.09.002→1.1.2.
  F-P37-IMP-002 (SS-ipc §InvalidRequest description mis-described the trigger as a pre-call
  guard never matched vs canonical post-call catch-all for unhandled variants). Fixed:
  architect corrected SS-ipc:412 descriptor; wire contract unchanged; prose-only errata;
  no version bump. Committed b52dd53. Counter stays 0. Pass-38 next (DONE D-280).
  Durable zero-context checkpoint written (next_session_resume_protocol v7.29). Pass-38 DONE D-280.

  D-286 (2026-06-14): Adversarial Pass-43 = FINDINGS (0C/1I). Producer: state-manager.
  F-P43-IMP-001 (SpawnAck step missing from SS-daemon-wiring-v2-delta §3 duplicate IPC-handler
  skeleton — the §3 SpawnSession arm did not carry the 4-step SpawnAck sequence (UUID generation,
  SpawnAck send, spawn_session() call, spawn-fail clear-launching_session_id) that the D-283/D-285
  propagation sweep added to SS-session-manager and SS-ipc. Root cause: CANONICAL PATTERN LOCK
  co-propagation missed the duplicate handler skeleton in SS-daemon-wiring-v2-delta §3 — structural
  duplication between SS-session-manager (canonical skeleton) and SS-daemon-wiring-v2-delta §3
  (mirror/delta skeleton) meant the pattern-lock update path had one unvisited site.
  This is the 7-of-7 spawn-handshake sweep sibling; the feature is now complete.
  Fixed via architect: §3 SpawnSession arm now carries the full 4-step SpawnAck sequence +
  spawn-fail clear-launching_session_id note. All 3 normative IPC-handler representations synced.
  SS-daemon-wiring-v2-delta v1.9.1 → v1.10.0. POL-11 propagation sweep: 16 stale-active v1.9.1
  literals updated across 8 BC files (BC-2.05.009/010/011, BC-2.08.001/003/004/007/008).
  Registry updated atomically. POL-11/POL-12/POL-13 PASS. factory-artifacts committed at a8e0ce8.
  DEDUP-IPC-HANDLER-SKELETON added to durable_task_register (non-blocking).
  CONSECUTIVE-CLEAN COUNTER = 0. Pass-44 = clean candidate 1 of 3. STATE v7.35→v7.36.

  D-287 (2026-06-14): Adversarial Pass-44 = FINDINGS (0C/1I). Producer: state-manager.
  F-P44-IMP-001 (EngineError::UnsupportedOperation collapsed to generic invalid_request wire code
  — BC-2.03.008 PC-3/EC-112 mandated a DISTINCT error banner for session-spawn-not-supported,
  symmetric to the already-closed I12-001 (BinaryNotFound→binary_not_found) and F-P34-IMP-001
  (InvalidPath→invalid_spawn_arg). Before this fix, UnsupportedOperation shared the invalid_request
  code with InvalidPath, making TUI error display ambiguous when a harness does not support session creation).
  Fixed via architect + product-owner parallel burst:
    SS-ipc v1.22.0: 11th wire code spawn_unsupported added to error taxonomy; banner
    "Session spawn not supported for this harness"; session_error_to_code UnsupportedOperation
    arm mapped to spawn_unsupported; inner _=> arm retained as forward-compat for FUTURE variants only.
    SS-session-manager v2.4.0: UnsupportedOperation match arm added to session_error_to_code;
    defense-in-depth comment: EC-112 is REACHABLE (ProfilePicker filtering is best-effort).
    SS-engine-module-v2-delta v1.5.0: best-effort ProfilePicker capability filtering note;
    spawn_unsupported count note for 0-capable profile harnesses; Lines 333-334 reconciled.
    SS-daemon-wiring-v2-delta v1.11.0: §3 error table UnsupportedOperation→spawn_unsupported
    row added; spawn comment updated; all 3 EngineError variants now in error table.
    BC-2.03.008 v1.0.2: PC-3 authored (UnsupportedOperation MUST emit spawn_unsupported +
    distinct banner); EC-112 authored; test vector for spawn-not-supported path.
    BC-2.05.010 v1.8.1: PC-4 authored (SpawnSession must emit spawn_unsupported when harness
    does not support session creation).
  All 3 EngineError variants now map to dedicated wire codes:
    BinaryNotFound→binary_not_found / InvalidPath→invalid_spawn_arg / UnsupportedOperation→spawn_unsupported.
  EngineError→wire-code class FULLY CLOSED.
  Registry: 7 entries bumped atomically (L-S027-004). Input-hash: BC-2.03.008→427c948, BC-2.05.010→fd8e22a.
  POL-11 cascade sweep: 88 stale literals across 30 files updated; 1 historical annotation added.
  BC-INDEX v1.40.5. POL-11/POL-12/POL-13 PASS. factory-artifacts committed at d59ce5f.
  CONSECUTIVE-CLEAN COUNTER = 0. Pass-45 = clean candidate 1 of 3. STATE v7.36→v7.37.

  D-285 (2026-06-14): Post-Pass-42 PROACTIVE CONSISTENCY SWEEP. Producer: state-manager.
  Fresh-context consistency-validator scoped to spawn-handshake feature doc-surface injected by
  orchestrator to break the F-P41/F-P42 partial-fix-sibling cycle (2 consecutive Importants on
  same feature). Found 5 findings + 1 sibling, ALL fixed atomically in one burst:
  CV-SS-001: SS-ipc §Trace Fix-(c) description used orphan wizard_session_id → launching_session_id.
  CV-SS-002: SS-ipc BC-2.09.008 PC-5 cross-ref cited "PC-5" → corrected to item 7.
  CV-SS-003: SS-embedded-pty spawn-fail return target "Step 1" → named step "ProfilePicker".
  CV-SS-004: SS-embedded-pty wizard Step 4 spawn-fail missing clear-launching_session_id obligation.
  CV-SS-005: BC-2.08.008 PC-5 ordering guarantee underspecified → causal step ordering + FIFO.
  Sibling: BC-2.09.008 PC Step 4 same underspecification expanded identically.
  Bumps: BC-2.08.008 v1.2.1, BC-2.09.008 v1.3.1, BC-INDEX v1.40.4. SS-ipc/SS-embedded-pty errata-no-bump.
  POL-11 sweep: EVAL-INDEX.md BC-INDEX pin 1.40.2→1.40.4 (1 file). POL-11/POL-12/POL-13 PASS.
  factory-artifacts committed at 0569dab. CONSECUTIVE-CLEAN COUNTER = 0 (unchanged; proactive sweep,
  not an adversarial pass). Pass-43 = clean candidate 1 of 3, now against holistically
  consistency-validated spawn-handshake. STATE v7.34→v7.35.

  D-284 (2026-06-14): Adversarial Pass-42 = FINDINGS (0C/1I). Producer: state-manager.
  F-P42-IMP-001 (orphan field-name wizard_session_id — partial-fix sibling of Pass-41 SpawnAck
  fix. Pass-41 corrected the SpawnAck variant doc-comment in SS-ipc; Pass-42 found 4 additional
  SS-ipc sites (body prose, §Trace errata, 2 internal doc-comments) + 1 SS-session-manager
  skeleton comment still using wizard_session_id. SS-ipc body contradicted its own §Trace
  (§Trace said launching_session_id; body used wizard_session_id). Demonstrates F-P41→F-P42
  self-sibling chain: a fix's own primary doc can be the surviving sibling.
  Fixed via architect errata-no-bump burst: 4 SS-ipc sites + 1 SS-session-manager site corrected.
  Whole-class grep confirmed zero live survivors of wizard_session_id. Canonical field name is
  AppMode::SessionCreation.launching_session_id everywhere. wizard_session_id RETIRED.
  Errata-no-bump: wire contract + canonical struct unchanged; SS-ipc stays v1.21.0,
  SS-session-manager stays v2.3.0. version-pin-registry.yaml unchanged.
  POL-11/POL-12/POL-13 PASS. factory-artifacts committed at ff6a428. Counter stays 0.
  Pass-43 = clean candidate 1 of 3. STATE v7.33→v7.34.

  D-289 (2026-06-14): Adversarial Pass-46 = FINDINGS (0C/1I). Producer: state-manager.
  F-P46-IMP-001 (stale §Architecture Anchors version pins in BC-2.08.001, BC-2.08.008, BC-2.09.008
  — 11 parenthetical version pins of the form `path#anchor (vX.Y.Z)` in §Architecture Anchors sections
  had drifted from the authoritative §Architecture Source rows. Root cause: POL-11's ID↔version adjacency
  regex matched artifact-ID version rows (e.g., `SS-foo v1.2.3`) but was BLIND to the parenthetical
  anchor form, so these pins escaped CI detection for multiple passes.
  Fixed: PO adopted the corpus-standard version-less navigational-anchor convention — stripped all 11
  parenthetical pins from §Architecture Anchors sections (BC-2.08.001: 5 pins; BC-2.08.008: 3 pins;
  BC-2.09.008: 3 pins). Authoritative version pins remain in §Architecture Source rows only.
  ERRATA-NO-BUMP: versions unchanged (BC-2.08.001 v1.5.0, BC-2.08.008 v1.2.1, BC-2.09.008 v1.3.1).
  Root-cause process-gap CLOSED by devops: POL-14 adds Pattern C to scripts/check_version_pins.py
  (detects+validates parenthetical anchor-pin form), wired to CI pol-lint + lefthook, develop @ 5d9d603.
  Grep-zero confirmed: `grep -rnE 'path#anchor (vX.Y.Z)' .factory/specs/behavioral-contracts/` = zero.
  POL-11/POL-14 PASS, POL-12 PASS, POL-13 PASS. factory-artifacts committed at 2a95fc4.
  CONSECUTIVE-CLEAN COUNTER RESET 1→0. Pass-47 = clean candidate 1 of 3. STATE v7.38→v7.39.

  D-291 (2026-06-14): Pass-48 ERRATA FIX BURST. Producer: state-manager.
  F-P48-001 (stale SessionError::EngineError variant doc-comment enumerated only 2 EngineError variants:
  BinaryNotFound→binary_not_found and InvalidPath→invalid_spawn_arg. Pass-44/D-287/F-P44-IMP-001 added
  the 3rd variant UnsupportedOperation→spawn_unsupported and propagated it to the mapping table,
  exhaustiveness prose, and session_error_to_code() body — but the enum variant doc-comment in SS-session-manager
  §SessionError was missed and retained the stale 2-item enumeration.
  Fixed (primary): SS-session-manager §SessionError enum variant doc-comment updated to enumerate all 3
  bridged EngineError variants: BinaryNotFound (→ binary_not_found), InvalidPath (→ invalid_spawn_arg),
  and UnsupportedOperation (→ spawn_unsupported) (F-P44-IMP-001).
  Fixed (sibling): SS-daemon-wiring-v2-delta I27-001 Model A inline comment step (a) intro sentence stated
  "If this returns EngineError::BinaryNotFound or EngineError::InvalidPath" — a stale 2-item closed
  enumeration. Reachability note also updated from "These EngineError codes" to "All three EngineError variants".
  Architect whole-class sweep surfaced the SS-daemon-wiring sibling that the adversary missed; zero live
  survivors confirmed. All normative surfaces (mapping table, exhaustiveness prose, session_error_to_code()
  body) were already correct after Pass-44/D-287; these are doc-comment-only errata corrections.
  ERRATA-NO-BUMP: SS-session-manager stays v2.4.0. SS-daemon-wiring-v2-delta stays v1.11.0.
  No version-pin-registry.yaml change. POL-11/POL-12 PASS.
  CONSECUTIVE-CLEAN COUNTER = 0. Pass-49 = clean candidate 1 of 3. STATE v7.40→v7.41.

  D-292 (2026-06-14): Pass-49 FIX BURST. Producer: state-manager.
  F-P49-001 (Important): BC-INDEX.md §Conventions contained a "Canonical SS version table" — a
  6-row hand-maintained markdown literal mirror of version-pin-registry.yaml. 4 of 6 rows were stale:
  SS-forward-compatibility v1.2.19 (registry: v1.2.20), SS-engine-module v1.1.20 (registry: v1.1.27),
  SS-deps-pin-manifest v1.1.17 (registry: v1.2.1), SS-conventions-anti-patterns v1.29.5 (registry: v1.32.6).
  Root cause: POL-11's ID↔version adjacency regex does not match markdown table cell form
  (e.g., "| SS-engine-module.md | v1.1.20 |") — table cell separators and whitespace break the pattern.
  This is a distinct POL-11 blind spot from the parenthetical-anchor-pin form closed by POL-14 (D-289).
  Fix: literal mirror table removed from BC-INDEX.md §Conventions and replaced with an explicit registry
  pointer: "see .factory/specs/version-pin-registry.yaml (single source of truth per ADR-0007)".
  Sibling fix (errata-no-bump): SS-conventions-anti-patterns.md §Architecture Source Pin-Symmetry
  Convention carried an identical hand-maintained 6-row literal table (mirroring the same data, with
  equally stale rows). Removed and replaced with same registry pointer. SS-conventions-anti-patterns
  is heavily cited — a version bump would force a disproportionate POL-11 cascade sweep; errata-no-bump
  precedent (Pass-37/39/48) applied. Version stays v1.32.6. §Trace entry labeled "ERRATA under v1.32.6".
  Note: SS-conventions-anti-patterns is architect-owned; PO touched it under sibling-reconciliation
  exception (mirror-removal identical to BC-INDEX change). Routing deviation noted in commit message.
  S-P49-001 (Suggestion, fixed in-scope): HS-EXP-011 and HS-EXP-012 cited "BC-2.08.004 PC-3" for the
  UDS-bind-ordering guarantee. PC-3 is a derived consequence (all SessionEntry records in DaemonState
  before TUI connects); the primary normative specification is PC-6 (UDS bind proceeds only after
  rediscover_sessions() returns) + Invariant 1 (ordering is mandatory). Corrected in HS-EXP-011 Step 5
  + Expected Outcome + HS-EXP-012 header + body. Errata-no-bump on both holdout files.
  False alarm cleared: SS-engine-module v1.1.26 references in BC §Trace sections are immutable
  historical anchor entries (version-pin-historical annotated); no architect sweep needed.
  EVAL-INDEX.md inputs[] BC-INDEX citation updated from v1.40.6 → v1.40.7.
  Registry: BC-INDEX → v1.40.7 (sole change; SS-conventions-anti-patterns stays v1.32.6).
  POL-11/POL-12 PASS. Defensive sweep: only 1 registry line changed (BC-INDEX only, confirmed).
  POL-11-MIRROR-TABLE-BLIND-SPOT added to durable_task_register.
  CONSECUTIVE-CLEAN COUNTER = 0. Pass-50 = clean candidate 1 of 3. STATE v7.41→v7.42.

  D-296 (2026-06-14): Pass-53 FIX BURST. Producer: state-manager.
  F-P53-001 (Important): BC-2.08.006 PC-3 cited BC-HOOK-007 as the authority for the
  hooks-settings.json file schema and claimed "all 5 canonical endpoints including SessionStart,
  each with a URL". BC-HOOK-007 governs the DTU gene-source clone's WriteHooksSettingsFile
  (5 URL keys: PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit; no PreCompact) —
  it does NOT govern the v1A daemon-written hooks-settings.json.
  Fix: PC-3 rewritten to anchor on BC-2.04.010 PC-3 / SS-daemon-wiring.md §Hook Tmpfile Generation.
  Corrected schema: 4 URL-bearing keys (PreToolUse, Notification, Stop, UserPromptSubmit) +
  2 reserved-empty keys (PostToolUse:[], PreCompact:[]) + no SessionStart key.
  SessionStart explicitly excluded: Claude Code fires POST /hooks/session-start via its own internal
  lifecycle; monocle's axum router handles it, but it is not configured through hooks-settings.json.
  Precision now distinguishes 5 served HTTP endpoints from 4 URL-bearing JSON keys + 2 empty JSON keys.
  Sibling closed in-scope (errata-no-bump): SS-daemon-wiring.md §SessionStart Invocation Path Note
  conflated "5 served endpoints" with the file JSON keys and omitted PreCompact; corrected.
  Whole-class sweep: 0 BC-layer survivors + 0 arch survivors (only this file cited BC-HOOK-007 as
  v1A daemon file content authority; all other SessionStart references describe the served HTTP endpoint
  set or DTU clone scope — correct). No new wire codes/variants.
  BC-2.08.006 v1.2.1 → v1.3.0. version-pin-registry.yaml updated (1 line changed).
  POL-11 PASS. POL-12 PASS. Defensive sweep: 1 registry line changed (BC-2.08.006 only, confirmed).
  CONSECUTIVE-CLEAN COUNTER = 0. Pass-54 = clean candidate 1 of 3. STATE v7.45→v7.46.

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
| PIVOT | D-296 Pass-53 FIX DONE; Pass-54 NEXT | 2026-06-14 | D-242..D-296: Passes 1-53 ALL resolved. C trajectory: 5→5→4→1→2→2→0×22,1,2,1,0,0,1,1,0,0,0,0,0,0. I trajectory: 8→6→9→4→4→2→4→2→1×16→0→0→3→2→1→1→1→1→0→1→1→3→2→1→0,2,2,0,0,0,1,1,1,1,0,1,1,1,1,1,1,1. CONSECUTIVE-CLEAN COUNTER = 0. NEXT: Pass-54 (clean candidate 1 of 3). |
| PIVOT | D-287: Pass-44 DONE (0C/1I) — F-P44-IMP-001 FIXED; Pass-45 NEXT | 2026-06-14 | D-287: F-P44-IMP-001 EngineError::UnsupportedOperation→spawn_unsupported (11th wire code); taxonomy complete; all 3 EngineError variants mapped to dedicated codes; SS-ipc v1.22.0, SS-session-manager v2.4.0, SS-engine-module-v2-delta v1.5.0, SS-daemon-wiring-v2-delta v1.11.0, BC-2.03.008 v1.0.2, BC-2.05.010 v1.8.1, BC-INDEX v1.40.5. POL-11 cascade 88 literals. COUNTER = 0. Pass-45 = clean candidate 1 of 3. |
| PIVOT | D-288: Pass-45 DONE (0C/0I CLEAN) — counter 0→1; Pass-46 NEXT | 2026-06-14 | D-288: Pass-45 CLEAN. Breadth sweep sound. S-P45-001/S-P45-002 (2 LOW holdout-prose imprecisions) deferred to Phase-4 prep as OBS-HS-PROSE-PHASE4-PREP. No spec change. CONSECUTIVE-CLEAN COUNTER = 1. Pass-46 = clean candidate 2 of 3. |
| PIVOT | D-289: Pass-46 DONE (0C/1I) — F-P46-IMP-001 FIXED; counter RESET 1→0; Pass-47 NEXT | 2026-06-14 | D-289: F-P46-IMP-001 stale §Architecture Anchors version pins in BC-2.08.001/008, BC-2.09.008 (11 parenthetical pins drifted; invisible to old POL-11); FIXED via version-less corpus-standard anchors (errata-no-bump); POL-14 closes root-cause blind spot (develop @ 5d9d603). factory-artifacts @ 2a95fc4. COUNTER = 0. Pass-47 = clean candidate 1 of 3. |
| PIVOT | D-293: Pass-50 DONE (0C/1I) — F-P50-001 FIXED; counter stays 0; Pass-51 NEXT | 2026-06-14 | D-293: F-P50-001 host_conn-lifecycle contradiction resolved; SessionNotReady/session_not_ready (9th variant/12th wire code; taxonomy 11→12) added; full cascade swept (9 spec files bumped, 31 prose files swept); SS-ipc v1.23.0, SS-session-manager v2.5.0, SS-daemon-wiring-v2-delta v1.11.1, SS-engine-module-v2-delta v1.6.0, BC-2.08.003 v1.4.0, BC-2.05.010 v1.9.0, BC-2.08.007 v1.5.0, BC-2.03.008 v1.0.3, BC-INDEX v1.40.8. POL-11/POL-12 PASS. COUNTER = 0. Pass-51 = clean candidate 1 of 3. |
| PIVOT | D-292: Pass-49 DONE (0C/1I) — F-P49-001 FIXED; counter stays 0; Pass-50 NEXT | 2026-06-14 | D-292: F-P49-001 BC-INDEX+SS-conventions canonical SS version mirror tables converted to registry pointers; POL-11 blind spot (markdown table-cell form) closed; BC-INDEX→v1.40.7; SS-conventions errata-no-bump; S-P49-001 HS-EXP-011/012 cite-precision; EVAL-INDEX updated; SS-engine-module §Trace false alarm cleared. POL-11/POL-12 PASS. COUNTER = 0. Pass-50 = clean candidate 1 of 3. |
| PIVOT | D-291: Pass-48 DONE (0C/1I) — F-P48-001 FIXED; counter stays 0; Pass-49 NEXT | 2026-06-14 | D-291: F-P48-001 stale SessionError::EngineError variant doc-comment 2-variant enumeration corrected to 3 variants in SS-session-manager (v2.4.0 errata) + SS-daemon-wiring-v2-delta sibling I27-001 Model A step (a) comment (v1.11.0 errata); architect whole-class sweep found sibling adversary missed; zero live survivors; errata-no-bump; no version changes; no registry change. POL-11/POL-12 PASS. COUNTER = 0. Pass-49 = clean candidate 1 of 3. |

develop has had docs/version-pin/CI-wiring commits this session (POL-13 anchor-lint CI wiring, version-pin maintenance, POL-14 parenthetical-anchor-pin lint). Phase 3 COMPLETE (D-232). 32/33 stories done, 192/195 pts (98%). D-238: Vision v2.2.3 APPROVED. D-239: Architecture delta (ADR-0009/0010/0011 + 5 SS deltas; ARCH-INDEX v1.0.28). D-275: POL-13/ANCHOR-LINT-TOOL live (CI + pre-commit). D-277: Pass-35 FIRST CLEAN; S-fixes reset counter to 0. D-278: Pass-36 2I (partial-fix siblings BC-2.09.003+BC-2.05.010); BC-2.09.003→1.5.0, BC-2.05.010→1.8.0. D-279: Pass-37 2I (BC-2.09.002 scoped-capture + SS-ipc:412 errata); BC-2.09.002→1.1.2. D-280: Pass-38 0C/0I CLEAN; S38-001 BC-2.09.008→1.2.0 (whole-class fix in-scope; counter RESET to 0). D-281: Pass-39 0C/0I CLEAN; S39-001 SS-embedded-pty:250 arch prose errata fixed in-scope (errata-no-bump; counter RESET to 0). D-282: Pass-40 0C/0I CLEAN; OBS-1 BC-2.09.002 §Trace line-numbers deferred as housekeeping (no spec change; counter ADVANCES 0→1). D-283: Pass-41 0C/1I FINDINGS; F-P41-IMP-001 session_id handshake gap FIXED (SpawnAck mechanism b; SS-ipc→1.21.0, SS-embedded-pty→1.6.0, SS-session-manager→2.3.0, BC-2.08.001→1.5.0, BC-2.08.008→1.2.0, BC-2.09.008→1.3.0, BC-INDEX→1.40.2; counter RESET 1→0). VSDD Phases 4-7 (old scope) SUSPENDED. D-286: Pass-43 0C/1I FINDINGS; F-P43-IMP-001 SS-daemon-wiring-v2-delta §3 SpawnAck step; SS-daemon-wiring-v2-delta v1.10.0. D-287: Pass-44 0C/1I FINDINGS; F-P44-IMP-001 UnsupportedOperation→spawn_unsupported; taxonomy 11 codes; SS-ipc v1.22.0, SS-session-manager v2.4.0, SS-engine-module-v2-delta v1.5.0, SS-daemon-wiring-v2-delta v1.11.0, BC-2.03.008 v1.0.2, BC-2.05.010 v1.8.1, BC-INDEX v1.40.5; counter=0. D-288: Pass-45 0C/0I CLEAN; breadth sweep sound; S-P45-001/S-P45-002 deferred to Phase-4 prep (OBS-HS-PROSE-PHASE4-PREP); counter ADVANCES 0→1. D-289: Pass-46 0C/1I FINDINGS; F-P46-IMP-001 stale §Architecture Anchors version pins in BC-2.08.001/008, BC-2.09.008 (11 parenthetical pins; invisible to old POL-11); FIXED version-less anchors (errata-no-bump); POL-14 closes root-cause blind spot (develop @ 5d9d603); factory-artifacts @ 2a95fc4; counter RESET 1→0. D-291: Pass-48 0C/1I FINDINGS; F-P48-001 stale SessionError::EngineError variant doc-comment 2-variant→3-variant (SS-session-manager v2.4.0 errata + SS-daemon-wiring-v2-delta v1.11.0 errata); errata-no-bump; no registry change; counter stays 0. D-292: Pass-49 0C/1I FINDINGS; F-P49-001 BC-INDEX+SS-conventions canonical SS version mirror tables converted to registry pointers (POL-11 blind spot closed); BC-INDEX→v1.40.7; SS-conventions errata-no-bump; S-P49-001 HS-EXP-011/012 cite-precision (PC-3→PC-6+Invariant 1); EVAL-INDEX BC-INDEX citation updated; SS-engine-module v1.1.26 §Trace false alarm cleared; counter stays 0. D-293: Pass-50 0C/1I FINDINGS; F-P50-001 host_conn-lifecycle contradiction resolved; SessionNotReady/session_not_ready (9th variant/12th wire code; taxonomy 11→12); full 9-file version cascade + 31-file prose cascade; SS-ipc v1.23.0, SS-session-manager v2.5.0, SS-daemon-wiring-v2-delta v1.11.1, SS-engine-module-v2-delta v1.6.0, BC-2.08.003 v1.4.0, BC-2.05.010 v1.9.0, BC-2.08.007 v1.5.0, BC-2.03.008 v1.0.3, BC-INDEX v1.40.8; counter stays 0. NEXT: Pass-51 (candidate 1 of 3) → human gate → story decomposition.

## Blocking Issues

None. All durable_task_register items non-blocking.

## Decisions Log (Phase-1d adversarial convergence — D-242..D-281)

D-047 through D-241 archived at: `cycles/cycle-001/decisions-archive.md` and `cycles/cycle-001/burst-log.md`.

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-293 | Pass-50 FIX BURST — F-P50-001 (Important) resolved. F-P50-001: host_conn-lifecycle contradiction for kill-during-Launching — daemon now establishes the CONTROL connection (host_conn.writer) during Launching state via a new post-spawn monitor, distinct from the PTY-streaming proxy_task (established at Running; proxy_task now Option<JoinHandle<()>>). BC-2.08.003 PC-1 confirmed correct; kill-during-Launching-race PID-fallback path (→kill_failed) added to spec. SessionError::SessionNotReady (9th variant) + session_not_ready wire code (12th; taxonomy 11→12) added for defensive control-ops on sessions not yet in a control-ready state (untrusted-client wire posture; semantically distinct from invalid_request; no-silent-failure). Full 11→12 / 8→9 cascade swept across SS-ipc v1.23.0, SS-session-manager v2.5.0, SS-daemon-wiring-v2-delta v1.11.1, SS-engine-module-v2-delta v1.6.0, BC-2.08.003 v1.4.0, BC-2.05.010 v1.9.0, BC-2.08.007 v1.5.0, BC-2.03.008 v1.0.3, BC-INDEX v1.40.8. PRD + holdouts verified clean (zero count refs). Registry: 9 entries bumped atomically (L-S027-004). POL-11 cascade: 83 stale literals across 31 files cleared. POL-11/POL-12 PASS. CONSECUTIVE-CLEAN COUNTER = 0. Pass-51 = clean candidate 1 of 3. STATE v7.42→v7.43. | 2026-06-14 | state-manager |
| D-292 | Pass-49 FIX BURST — F-P49-001 (Important) + S-P49-001 resolved. F-P49-001: BC-INDEX.md Canonical SS version table (6-row hand-maintained literal mirror of version-pin-registry.yaml) had 4 stale rows (SS-forward-compatibility v1.2.19, SS-engine-module v1.1.20, SS-deps-pin-manifest v1.1.17, SS-conventions v1.29.5 — all drifted from registry). Root cause: POL-11 blind to markdown table-cell version literal form. Fix: literal table removed, replaced with registry pointer. BC-INDEX→v1.40.7. Sibling: SS-conventions-anti-patterns carried same hand-maintained mirror table — also converted to registry pointer (errata-no-bump, stays v1.32.6; heavily-cited, disproportionate to bump for non-normative mirror removal; routing deviation noted: PO touched architect-owned file under sibling-reconciliation exception). EVAL-INDEX BC-INDEX citation updated v1.40.6→v1.40.7. S-P49-001: HS-EXP-011/012 cited BC-2.08.004 PC-3 for UDS-bind-ordering; correct cite is PC-6+Invariant 1; corrected (errata-no-bump). SS-engine-module v1.1.26 §Trace references confirmed historical (false alarm cleared). POL-11-MIRROR-TABLE-BLIND-SPOT added to durable_task_register. POL-11/POL-12 PASS. Registry diff = 1 line (BC-INDEX only, confirmed). CONSECUTIVE-CLEAN COUNTER = 0. Pass-50 = clean candidate 1 of 3. STATE v7.41→v7.42. | 2026-06-14 | state-manager |
| D-291 | Pass-48 ERRATA FIX BURST — F-P48-001 (Important) resolved. F-P48-001: stale SessionError::EngineError variant doc-comment enumerated only 2 EngineError variants (BinaryNotFound→binary_not_found; InvalidPath→invalid_spawn_arg), omitting UnsupportedOperation→spawn_unsupported which was added by Pass-44/D-287/F-P44-IMP-001. Corrected in SS-session-manager: enum variant doc-comment at §SessionError now enumerates all 3 variants (BinaryNotFound/InvalidPath/UnsupportedOperation). Sibling also fixed in SS-daemon-wiring-v2-delta: I27-001 Model A comment step (a) intro sentence + reachability note corrected from 2-variant to 3-variant enumeration. Architect whole-class sweep found the SS-daemon-wiring sibling that the adversary missed; zero live survivors confirmed. All normative surfaces (mapping table, exhaustiveness prose, session_error_to_code() body) were already correct after Pass-44; this is a doc-comment-only errata correction. ERRATA-NO-BUMP: SS-session-manager stays v2.4.0, SS-daemon-wiring-v2-delta stays v1.11.0. No registry change. POL-11/POL-12 PASS. CONSECUTIVE-CLEAN COUNTER = 0. Pass-49 = clean candidate 1 of 3. STATE v7.40→v7.41. | 2026-06-14 | state-manager |
| D-290 | Pass-47 FIX BURST — F-P47-001 (Important) + S-P47-001 + S-P47-002 resolved. F-P47-001: Detached re-discovery MUST NOT emit SessionStateChanged (Option B decision) — removed over-specified SessionStateChanged{Detached} emission on Detached re-discovery from BC-2.08.004 PC-2b + EC-172; restores symmetry with Running/Launching re-discovery (all three register SessionEntry without emitting). BC-2.08.008 Invariant-1 tightened: "mutates SessionEntry.state TO A DIFFERENT VALUE" — re-discovery of unchanged persisted state (Detached→Detached) explicitly NOT a transition; removed from no-exception clause. Arch docs (§3b table, SS-session-manager) already consistent — no arch change. BC-2.08.004 v1.2.1→v1.3.0. BC-2.08.008 v1.2.1→v1.3.0. S-P47-001: added `d` kill-alias to BC-2.06.025 Terminating disabled-action lists (PC-1 + Invariant 4); errata-no-bump (v1.3.1 stays). S-P47-002: BC-2.03.008 PC-2 clarifying note distinguishing raw EngineError Display from the SessionError::EngineError-wrapped wire message form; errata-no-bump (v1.0.2 stays). BC-INDEX v1.40.5→v1.40.6. Registry: BC-2.08.004→v1.3.0, BC-2.08.008→v1.3.0, BC-INDEX→v1.40.6 (atomic L-S027-004). CONSECUTIVE-CLEAN COUNTER = 0. Pass-48 = clean candidate 1 of 3. STATE v7.39→v7.40. | 2026-06-14 | state-manager |
| D-289 | Pass-46 VERDICT FINDINGS (0C/1I). F-P46-IMP-001: stale §Architecture Anchors version pins in BC-2.08.001 (5 pins), BC-2.08.008 (3 pins), BC-2.09.008 (3 pins) — 11 total parenthetical pins drifted from authoritative §Architecture Source rows; invisible to old POL-11 (ID↔version adjacency regex missed the `path#anchor (vX.Y.Z)` form). FIXED: corpus-standard version-less navigational anchors adopted (stripped 11 pins across 3 BCs; errata-no-bump). Root-cause process-gap CLOSED by POL-14 (Pattern C in check_version_pins.py + CI pol-lint + lefthook, develop @ 5d9d603). POL-11/POL-12/POL-13/POL-14 PASS. Registry unchanged. CONSECUTIVE-CLEAN COUNTER RESET 1→0. Pass-47 = clean candidate 1 of 3. STATE v7.38→v7.39. | 2026-06-14 | state-manager |
| D-288 | Pass-45 VERDICT CLEAN (0C/0I). CONSECUTIVE-CLEAN COUNTER ADVANCES 0→1 (candidate 1 of 3). Breadth sweep (PTY/scrollback/persistence/hooks/permission-overlay/SessionState/holdouts) sound; Pass-44 propagation cascade verified complete/correct. Two LOW suggestions deferred to Phase-4 holdout-prep as OBS-HS-PROSE-PHASE4-PREP: S-P45-001 (HS-EXP-014:46 illustrative field list omits child_pid; non-normative); S-P45-002 (HS-EXP-013:54 step-9 'most recently added' vs BC-2.09.009 PC-5c 'front=oldest'; hedged; PASS-criterion independent). No spec text changed. Registry unchanged. POL-11/POL-12/POL-13 PASS (no version changes). CONSECUTIVE-CLEAN COUNTER = 1. Pass-46 = clean candidate 2 of 3. STATE v7.37→v7.38. | 2026-06-14 | state-manager |
| D-286 | Pass-43 VERDICT FINDINGS (0C/1I). F-P43-IMP-001 SpawnAck step missing from SS-daemon-wiring-v2-delta §3 duplicate IPC-handler skeleton — §3 SpawnSession arm lacked the 4-step SpawnAck sequence that D-283/D-285 propagated to SS-session-manager and SS-ipc. Root cause: CANONICAL PATTERN LOCK co-propagation missed the §3 mirror skeleton (structural duplication failure mode). Last spawn-handshake sibling (7-of-7); all 3 normative representations now synced. Fixed: §3 SpawnSession arm carries full 4-step sequence + spawn-fail clear-launching_session_id. SS-daemon-wiring-v2-delta v1.9.1 → v1.10.0. POL-11 16-literal sweep across 8 BC files (BC-2.05.009/010/011, BC-2.08.001/003/004/007/008). Registry updated atomically. POL-11/POL-12/POL-13 PASS. factory-artifacts a8e0ce8. DEDUP-IPC-HANDLER-SKELETON added to durable_task_register. Counter=0. Pass-44 = clean candidate 1 of 3. STATE v7.35→v7.36. | 2026-06-14 | state-manager |
| D-256 | Adversarial Pass-14 convergence-fix — 0 Critical / 1 Important / 1 Suggestion. I14-001 (lone stale portable-pty 0.8.x normative literal in SS-session-manager §env-inheritance prose at line 597; architect ran exhaustive crate-pin sweep across all in-scope specs confirming I14-001 was the ONLY stale live straggler — class CLOSED; SS-session-manager 1.8.0→1.8.1). S14-001 [process-gap] DEP-PIN-SWEEP-RULE: POL-11 keys on artifact-IDs not crate-pin literals → stale crate pins in prose escape CI; recorded in durable_task_register (devops-engineer tooling task, non-blocking). 1 registry entry bumped atomically (L-S027-004): SS-session-manager→1.8.1. Propagation sweep: 15 unique files updated (27 stale actives); no §Trace/historical entries touched. POL-11 PASS (319 active). POL-12 PASS. EIGHTH consecutive zero-Critical (C:...,0×8). Consecutive-clean counter = 0; Pass-15 next. STATE v7.06→v7.07. | 2026-06-04 | state-manager |
| D-257 | Adversarial Pass-15 convergence-fix — 0 Critical / 2 Important / 1 Suggestion. I15-001 (EC-283 code 'invalid_request' for RenameSession empty-name contradicts canonical session_error_to_code() which maps InvalidSessionName→'rename_failed' unconditionally — cross-doc contradiction/build collision; BC-2.05.010 1.5.0→1.5.1, EC-283 corrected to 'rename_failed', §Trace rationale updated). I15-002 (PRD §2.5 listed BC-2.05.010 as '6-variant' title missing AttachSession — S-P7-003 partial-fix straggler; class-close sweep also found entirely missing BC-2.05.011 row in §2.5; prd 1.28.1→1.28.2, 7-variant + BC-2.05.011 row added). S15-001 ADR-0010 §Trace v1.2.0 '256' lacks inline superseded-marker — DEFERRED (housekeeping; fold into next ADR-0010 substantive edit; ADR-0010-TRACE-256-MARKER recorded in durable_task_register). 2 registry entries bumped atomically (L-S027-004): BC-2.05.010→1.5.1, prd→1.28.2. POL-11 PASS (319 active). POL-12 PASS. NINTH consecutive zero-Critical (C:...,0×9). Consecutive-clean counter = 0; Pass-16 next. STATE v7.07→v7.08. | 2026-06-04 | state-manager |
| D-259 | Adversarial Pass-17 convergence-fix — 0 Critical / 1 Important / 2 Suggestions. I17-001 (Architecture-Source pin-symmetry violation: 3 BC cells cited ADR docs unpinned while SS docs pinned — violates codified BC-INDEX Pin-Symmetry Convention F-R117-3/SE-17e; sibling-inconsistent since SS-05 BCs pinned ADR-0010) FIXED via EXHAUSTIVE sweep of all 25 in-scope BC Architecture-Source cells: pinned ADR-0011 v1.2.0 (BC-2.09.001) + ADR-0009 v1.0.2 (BC-2.08.001/002) + 2 additional unpinned SS-session-manager v1.8.1 refs found in same cells; zero remaining violations confirmed. S17-002 (loose ADR §-anchors) folded in: §PTY-stack-selection/§native-detached-session-host → §Decision (verified exact headings). S17-001 (ADR-0009 risk-table '5s backoff' ambiguous vs canonical no-retry) FIXED → '5s hard deadline, one attempt, no retry per BC-2.08.004 Inv-2' (ADR-0009 1.0.1→1.0.2). BC bumps: BC-2.09.001 1.3.1, BC-2.08.001 1.3.1, BC-2.08.002 1.2.1. 4 registry entries bumped atomically (L-S027-004). POL-11 PASS (329 active). POL-12 PASS. ELEVENTH consecutive zero-Critical (C:...,0×11). Consecutive-clean counter = 0; Pass-18 next. STATE v7.09→v7.10. | 2026-06-04 | state-manager |
| D-270 | Adversarial Pass-28 FINDINGS (0C/1I/1S). I28-001 HS-EXP-014 Step 1 Model-B survivor FIXED. S28-001 §Trace housekeeping DEFERRED. POL-11 PASS (330 active). Counter=0; Pass-29 next. STATE v7.20→v7.21. | 2026-06-13 | state-manager |
| D-271 | Adversarial Pass-29 FINDINGS (1C/0I/0S). C29-001 harness_id missing from SpawnOptions struct in SS-engine-module-v2-delta — errata fix (no version bump). factory-artifacts 5ea7395. Counter=0; Pass-30 next. STATE v7.21→v7.22. | 2026-06-13 | state-manager |
| D-272 | Adversarial Pass-30 FINDINGS (2C/1I). ADR-0006 cross-crate #[non_exhaustive] constructor gap — SpawnOptions for_spawn_request()+with_daemon_fields(), SpawnRecipe/SessionSnapshot/SerializedCell/PermissionPromptPayload new(), E0639 ..opts workaround documented. Registry entries bumped. factory-artifacts ce7868c. Counter=0; Pass-31 next. STATE v7.22→v7.23. | 2026-06-13 | state-manager |
| D-273 | Adversarial Pass-31 FINDINGS (1C/1I). P31-CRIT-001 EngineError enum declared as NEW canonical #[non_exhaustive] enum in SS-engine-module-v2-delta §EngineError (new in v1A): 3 variants (UnsupportedOperation/BinaryNotFound/InvalidPath). P31-HIGH-001 SpawnRecipe parity in SS-session-manager. Registry bumped. factory-artifacts 927ff29. Counter=0; Pass-32 next. STATE v7.23→v7.24. | 2026-06-13 | state-manager |
| D-274 | Adversarial Pass-32 FINDINGS (0C/3I). IMP-001 session_error_to_code _=> prose contradiction (forward-compat, not panic); IMP-002 dead anchor #engineerror-additions→#engineerror-new-in-v1a; IMP-003 EngineError 'extension'→new-type framing. All fixed. factory-artifacts b9d3591. Counter=0; Pass-33 next. STATE v7.24→v7.25. | 2026-06-13 | state-manager |
| D-275 | Adversarial Pass-33 FINDINGS (0C/2I). I33-001 PERVASIVE dead-anchor class — human directive WHOLE-CLASS-NOW + STABLE-EXPLICIT-ANCHORS. Built scripts/check_cross_ref_anchors.py (POL-13), wired CI + pre-commit. 70 dead anchors found; 42 explicit <a id> anchors added across 12 docs (no version bumps for navigational anchors). PO fixed 2 defective citations. ANCHOR-LINT-TOOL CLOSED. factory-artifacts ef5b1ec; develop e9e9103 (CI wiring). Counter=0; Pass-34 next. STATE v7.25→v7.26. | 2026-06-13 | state-manager |
| D-276 | Adversarial Pass-34 FINDINGS (1C/1I). C34-001 InvalidPath null-byte detection impossible via to_str() alone (null is valid UTF-8) — two-pronged: to_str().is_none() + as_bytes().contains(&0). BC-2.03.005/006/007/008 updated. I34-001 BC-2.08.001 stale anchor version labels. factory-artifacts 8fab81c. Counter=0; Pass-35 next. STATE v7.26→v7.27. | 2026-06-13 | state-manager |
| D-277 | Adversarial Pass-35 CLEAN (0C/0I) — FIRST CLEAN of session. 3 Suggestions fixed in-scope per production-grade principle: S35-001 split-pair rationale arithmetic; S35-002 KeyInput SessionHostDead→attach_failed error path; S35-003 mouse Drag(MouseButton) missing arm + Moved Ps 32→35 + full Ps/modifier table. SS-daemon-wiring-v2-delta v1.9.1, SS-embedded-pty v1.5.2, SS-session-manager v2.2.1, SS-ipc v1.20.1, SS-engine-module-v2-delta v1.4.1. factory-artifacts a0d5720. S-fixes changed package → COUNTER RESET to 0; Pass-35 CLEAN does NOT count. Pass-36 = clean candidate 1 of 3. Durable checkpoint v7.28 written. STATE v7.27→v7.28. | 2026-06-13 | state-manager |
| D-278 | Adversarial Pass-36 FINDINGS (0C/2I). F-P36-IMP-001 BC-2.09.003 Invariant-3 contradicted S35-003 Moved-reachability model (Ps=35 reachable under 1003; Invariant-3 still said unreachable). F-P36-IMP-002 BC-2.05.010 missing named No-silent-failure invariant + Detach/Resize error-path gaps + dangling SS arch forward-refs. PO-only fix: named invariant authored; whole-class swept all fallible variants (Kill/Rename/Attach too); ResizePane WARN-drop exception documented. BC-2.09.003→1.5.0, BC-2.05.010→1.8.0. factory-artifacts 33dea39. Counter stays 0. Pass-37 next. STATE v7.28→v7.28a. | 2026-06-13 | state-manager |
| D-279 | Adversarial Pass-37 FINDINGS (0C/2I). F-P37-IMP-001 BC-2.09.002 Invariant-5 globally-active mouse capture — partial-fix sibling of I3 scoped-capture; Pass-36 BC-2.09.003 fix dismissed sibling BC-2.09.002. F-P37-IMP-002 SS-ipc:412 invalid_request described as pre-call guard (never matched) vs canonical post-call catch-all. PO: BC-2.09.002→1.1.2 (scoped-capture). Architect: SS-ipc:412 errata prose-only fix (no version bump — wire contract unchanged). factory-artifacts b52dd53. Counter stays 0. Pass-38 next. Durable checkpoint v7.29 written. STATE v7.28→v7.29. | 2026-06-13 | state-manager |
| D-280 | Pass-38 VERDICT CLEAN (0C/0I) — first clean candidate. Lone Suggestion S38-001 (BC-2.09.008 PC-4/PC-1 partial mouse-capture restatement: omitted EnableMouseCapture entry step and DisableMouseCapture exit step) FIXED IN-SCOPE whole-class per human directive + production-grade principle. PC-4 rewritten with full two-step entry sequence (EnableMouseCapture → SGR 1006 h); PC-1 rewritten with full two-step exit sequence (SGR 1006 l → DisableMouseCapture). Cross-references to BC-2.09.002 Invariant-5 added as authoritative owning contract. Whole-class sweep: all SS-09 (001-009) + SS-05 (001-011) BCs scanned for same partial-restatement pattern — zero other survivors. BC-2.09.008→1.2.0. BC-INDEX→1.40.1 (§Trace row added). version-pin-registry.yaml atomic (L-S027-004). POL-11 propagation: EVAL-INDEX.md inputs BC-INDEX pin updated 1.40.0→1.40.1. POL-11 PASS (330 active). POL-12 PASS. POL-13 PASS. factory-artifacts 71279a4. CONSECUTIVE-CLEAN COUNTER RESET 1→0 (spec content changed; Pass-35 precedent applies). Pass-39 = clean candidate 1 of 3. STATE v7.29→v7.30. | 2026-06-14 | state-manager |
| D-281 | Pass-39 VERDICT CLEAN (0C/0I) — counter had advanced 0→1. Lone non-blocking observation S39-001 (SS-embedded-pty:250 stale "global EnableMouseCapture" prose↔code contradiction — partial-fix-sibling survivor of the global→scoped retirement; the BC-only Pass-38 sweep fixed BC layer but did not sweep architecture prose). Fixed in-scope by architect: corrected prose at line 250 to state keyboard enhancement flags are global at TUI startup while EnableMouseCapture is scoped to EmbeddedTerminal entry per BC-2.09.002 Invariant-5. Whole-class architecture+BC sweep confirmed zero other live survivors. The only remaining "global mouse capture" references are §Trace/changelog history and unadopted-option UX-tradeoff description (SS-embedded-pty:366/370) — both correctly scoped as non-normative. Errata-no-bump: SS-embedded-pty stays v1.5.2 (normative code block unchanged; analogous to SS-ipc:412 Pass-37 prose-only errata precedent). POL-11 PASS. POL-12 PASS. POL-13 PASS. Registry unchanged. factory-artifacts a08d411. CONSECUTIVE-CLEAN COUNTER RESET 1→0 (reviewed spec text changed). Pass-40 = clean candidate 1 of 3. STATE v7.30→v7.31. | 2026-06-14 | state-manager |
| D-282 | Pass-40 VERDICT CLEAN (0C/0I). No spec text changed this pass. OBS-1 (BC-2.09.002 Inv-5 §Trace cites SS-embedded-pty lines 254-258 for the ENTRY sequence; actual ENTRY block at 279-289) DEFERRED as durable housekeeping — navigational stale-line-number; section-name anchors resolve correctly; POL-13 green; no version bump required per closed dead-anchor/navigational precedent (C29-001/D-275). Ratified LOW deferral added to Section E; OBS-1-BC2.09.002-TRACE-LINE-NUMBERS registered in durable_task_register. POL-11 PASS. POL-12 PASS. POL-13 PASS. Registry unchanged. CONSECUTIVE-CLEAN COUNTER ADVANCES 0→1 (candidate 1 of 3). Pass-41 = clean candidate 2 of 3. STATE v7.31→v7.32. | 2026-06-14 | state-manager |
| D-283 | Pass-41 VERDICT FINDINGS (0C/1I). F-P41-IMP-001: SessionCreation→EmbeddedTerminal session_id handshake gap — BC-2.08.008 PC-5 destructured a non-existent field from AppMode::SessionCreation (no launching_session_id field existed) and no correlation mechanism existed for TUI to learn the daemon-assigned session_id. ClientToServer::SpawnSession carries no correlation token; SessionStateChanged{Launching} is broadcast to ALL clients making "first unseen Launching id" a race on concurrent spawns. Fix: architect chose mechanism (b) deterministic SpawnAck. UUID locus reconciled to daemon IPC handler (generates UUID BEFORE spawn_session() call). New ServerToClient::SpawnAck{session_id} variant added to SS-ipc. New AppMode::SessionCreation.launching_session_id: Option<String> field added to SS-embedded-pty. TUI auto-advance to EmbeddedTerminal matches SessionStateChanged{Running}.session_id against launching_session_id (deterministic, no broadcast-race). IPC handler emits SpawnAck point-to-point (per-client FIFO guarantees delivery before SessionStateChanged{Launching} broadcast). PO propagated to BC-2.08.001 v1.5.0 (PC-1 UUID-locus), BC-2.08.008 v1.2.0 (PC-5 destructure + EC-303 SpawnAck mechanism), BC-2.09.008 v1.3.0 (PC Step 4/5 SpawnAck wiring). Whole-class sweep: AppMode destructures, IPC correlation flows, UUID-locus wording — zero survivors. Only SpawnSession needs correlated ack; kill/attach/detach/rename use TUI-known ids — correct by design. Bumps: SS-ipc v1.21.0, SS-embedded-pty v1.6.0, SS-session-manager v2.3.0, BC-2.08.001 v1.5.0, BC-2.08.008 v1.2.0, BC-2.09.008 v1.3.0, BC-INDEX v1.40.2. POL-11 propagation sweep: 37 files, 68 active literals cleared. POL-11 PASS (332 active). POL-12 PASS. POL-13 PASS (SpawnAck explicit anchor added). factory-artifacts bcc4f6d. CONSECUTIVE-CLEAN COUNTER RESET 1→0. Pass-42 = clean candidate 1 of 3. STATE v7.32→v7.33. | 2026-06-14 | state-manager |
| D-269 | Adversarial Pass-27 FINDINGS — consecutive-clean counter stays 0. 0 Critical / 1 Important / 1 Suggestion. I27-001 (LOAD-BEARING spawn-path contradiction: the extant spawn_session(recipe: SpawnRecipe) signature took a PRE-BUILT recipe as argument, making the asserted Model-A statement 'spawn_session internally calls spawn_recipe(&opts)' IMPOSSIBLE to satisfy — while Model-B TUI-side recipe-build rendered binary_not_found/invalid_spawn_arg ServerToClient::Error codes UNREACHABLE, defeating BC-2.03.007 PC-3/PC-7 no-silent-failure guarantee; root-cause: the I12-001 fix asserted Model-A prose without reconciling the signature/IPC payload) ADJUDICATED → MODEL A: daemon-side spawn_recipe() is the ONLY model making I12-001 taxonomy reachable + consistent with daemon-owns-EngineModule. PROPAGATED: spawn_session(opts: SpawnOptions) signature; ClientToServer::SpawnSession{opts} wire payload; SpawnOptions promoted to wire type (#[non_exhaustive]+Serialize/Deserialize+harness_id field); SpawnRecipe demoted to daemon-internal. 7 arch docs bumped (SS-session-manager 1.9.0→2.0.0 MAJOR, SS-ipc 1.18.0→1.19.0, SS-daemon-wiring-v2-delta 1.7.0→1.8.0, SS-engine-module-v2-delta 1.1.1→1.2.0, ADR-0010 1.5.0→1.6.0, SS-embedded-pty 1.5.0→1.5.1, SS-deps-pin-manifest-v2-delta 1.0.0→1.0.1). PO reconciled BCs: BC-2.08.001 Precondition-2→opts + EC-150→binary_not_found (1.3.1→1.4.0); BC-2.03.007 PC-3/PC-7 confirmed reachable (1.1.0→1.2.0); BC-2.05.010 SpawnSession PC-1/PC-2/EC-280 Model-B residue→opts (1.6.0→1.7.0). S27-001 (stale ARCH-INDEX bump instruction) annotated historical. 10 registry entries bumped atomically. Propagation sweep: 36 files (83 stale actives cleared). POL-11 PASS (327 active). POL-12 PASS. TWENTY-FIRST consecutive zero-Critical (C:...,0×21). Consecutive-clean counter = 0 (stays 0 — Important present). Pass-28 next = clean candidate 1 of 3. STATE v7.19→v7.20. | 2026-06-13 | state-manager |
| D-268 | Adversarial Pass-26 FINDINGS — consecutive-clean counter stays 0. 0 Critical / 1 Important / 1 Suggestion. I26-001 (BC-2.08.007 EC-185/EC-188 cited non-existent SessionError variants AlreadyAttached/AttachTimeout — the closed 8-variant taxonomy has neither; attach_session() cannot satisfy these ECs) FIXED via reconcile-to-existing (production-grade, no taxonomy extension): EC-185 already-attached → idempotent Ok() (matches kill/detach idempotency BC-2.08.003 EC-165 / BC-2.08.007 EC-186 + BC-2.09.001 PC-6 re-attach-triggers-fresh-dump); EC-188 attach-timeout → SessionError::SessionHostDead → attach_failed per session_error_to_code attach-path arm (1.4.0→1.4.1). S26-001 (SpawnRecipe wire struct lacked #[non_exhaustive] vs SS-ipc blanket policy + own prose) FIXED: #[non_exhaustive] added to SpawnRecipe (SS-engine-module-v2-delta 1.1.0→1.1.1, prose narrowed re SpawnOptions daemon-internal); EXHAUSTIVE wire-type sweep found 2 MORE missing: PermissionDecisionKind (SS-ipc 1.17.0→1.18.0) + SessionState (SS-session-manager 1.8.1→1.9.0); SerializedColor/TransportEvent intentional documented exclusions confirmed. 4 registry entries bumped atomically (L-S027-004): BC-2.08.007→1.4.1, SS-engine-module-v2-delta→1.1.1, SS-ipc→1.18.0, SS-session-manager→1.9.0. Propagation sweep: 69 stale active version-pin literals cleared across 29 files (POL-11-driven). POL-11 PASS (324 active). POL-12 PASS. TWENTIETH consecutive zero-Critical (C:...,0×20). Consecutive-clean counter = 0 (stays 0 — Important present). Pass-27 next = clean candidate 1 of 3. STATE v7.18→v7.19. | 2026-06-13 | state-manager |
| D-267 | Adversarial Pass-25 FINDINGS — consecutive-clean counter stays 0. 0 Critical / 1 Important / 1 Suggestion. Fresh-context Pass-25 (extra scrutiny on less-swept L1 prose / holdouts / ARCH-INDEX / SS-03/05/06 axes) confirmed SS-09 family re-derivation sound and found I25-001: retired SessionState::Created survived in THREE L1 source-of-truth locations — (1) product-brief §LAUNCH lifecycle prose 'Created → Launching → Running → Detached → Terminated' (uses retired Created, omits ratified Terminating); (2) vision §v1A-LAUNCH prose same stale string; (3) vision §SessionManager Rust code block retained `Created` and `Killed` enum variants (both removed from the ratified enum: Created was never persisted/observed, Killed was superseded by Terminating). The Created-scrub had been applied at BC-2.08.001 v1.1.0 but never propagated to L1. FIXED: all 3 locations → ratified enum/prose `Launching → Running → Detached → Terminating → Terminated` (brief 2.0.3→2.0.4; vision 2.2.2→2.2.3); consistency correction only — no decision change. POL-11 sweep also updated vision changelog BC-2.08.001 citations from v1.1.0 → v1.3.1 (canonical). 1 registry entry bumped atomically (L-S027-004): product-brief→2.0.4. S25-001 (EVAL-INDEX HS-EXP-015 4-BC/6-class count note) NO ACTION (Suggestion advisory; mapping sound). POL-11 PASS (328 active). POL-12 PASS. NINETEENTH consecutive zero-Critical (C:...,0×19). Consecutive-clean counter = 0 (stays 0 — Important present). Pass-26 next = clean candidate 1 of 3. Adversary noted package 'should converge' after this L1 fix. STATE v7.17→v7.18. | 2026-06-13 | state-manager |
| D-265 | Adversarial Pass-23 FINDINGS — consecutive-clean counter stays 0. 0 Critical / 2 Important / 1 Suggestion. Fresh-context Pass-23 (with extra SS-08/SS-09 scrutiny after the Pass-22 cluster) uncovered a mouse-coordinate off-by-one contradiction in the SS-09 encoding BCs (adjacent to but not covered by Pass-22 scrollback/AppMode sweep). I23-001 (BC-2.09.002 EC-213 mouse SGR omitted the +1 1-indexing conversion — claimed byte sequence was wrong by exactly 1 in both axes; would land every click one cell up-and-left; contradicted canonical mouse_event_to_pty_bytes() px=col+1/py=row+1 AND HS-EXP-015 step 15-16) FIXED: EC-213 corrected; PC-2 mouse row got +1 cross-reference to BC-2.09.003 §Coordinate Convention (1.0.1→1.1.0). I23-002 (BC-2.09.003 Canonical Test Vectors omitted +1 offset, contradicting own EC-220/PC-2 which correctly state 1-indexed output — three coordinate-bearing test-vector rows were wrong) FIXED → all three rows corrected with inline derivation annotations (Px=col+1=N, Py=row+1=N) for implementer clarity (1.2.0→1.3.0). S23-001 root-cause fix (no explicit pane-origin convention — caused readers to assume 0-indexed direct mapping) FIXED → §Coordinate Convention section added to BC-2.09.003 as authoritative reference for all SS-09 mouse coordinate examples; cross-references HS-EXP-015 step 15-16 as the canonical correct example. EXHAUSTIVE sweep: all SS-09 BCs (BC-2.09.001..009) + HS-EXP-011..015 reviewed; BC-2.09.002/003 confirmed as the only files with mouse-coordinate examples; keyboard encodings (Kitty/Alt/control/bracketed-paste) verified no drift; HS-EXP-015 verified already-correct. BC-INDEX §Trace v1.40.0 logs the Pass-23 SS-09 mouse-coordinate sweep (1.39.1→1.40.0). 3 registry entries bumped atomically (L-S027-004): BC-2.09.002→1.1.0, BC-2.09.003→1.3.0, BC-INDEX→1.40.0. POL-11 sweep: 2 files updated (SS-embedded-pty.md line 491 BC-2.09.003 historical-anchor reclassified; EVAL-INDEX.md inputs pin 1.39.1→1.40.0). POL-11 PASS (327 active). POL-12 PASS. SEVENTEENTH consecutive zero-Critical (C:...,0×17). Consecutive-clean counter = 0 (stays 0 — I findings present). Pass-24 next = clean candidate 1 of 3 (streak still restarted from Pass-22). STATE v7.15→v7.16. | 2026-06-13 | state-manager |
| D-266 | Adversarial Pass-24 FINDINGS — consecutive-clean counter stays 0. 0 Critical / 1 Important / 2 Suggestion. Fresh-context Pass-24 (MAXIMUM SS-09/SS-08 scrutiny after the Pass-22/23 clusters) did a per-BC re-derivation of the ENTIRE SS-09 family (BC-2.09.001..009) against canonical SS-embedded-pty encoding tables — confirmed BC-2.09.001/002/003/005/006/007/008/009 ALL CLEAN; only BC-2.09.004 (Kitty CSI-u, still v1.0.0, never previously re-derived) had a defect. I24-001 (BC-2.09.004 line-90 Ctrl+Shift+Enter derivation annotation used 'shift(2)' contradicting its own PC-2 canonical bitfield shift=1/alt=2/ctrl=4 — byte literal \x1b[13;6u correct but the annotation would mislead an implementer to encode shift as bit-2, breaking HS-EXP-015 step 12) FIXED → '1 + shift(1) + ctrl(4) = 6' canonical; full BC-2.09.004 annotation sweep confirmed no byte-literal errors (1.0.0→1.0.1). S24-002 (BC-2.09.009 §Trace v1.0.0 stale once-only bell note retained after v1.1.0 C2-001 per-prompt bell reversal — misleading audit trail) FIXED → superseded marker appended to §Trace v1.0.0 design-decision bullet (1.1.0→1.1.1). S24-001 (registry last_bump_commit descriptions for BC-2.09.004 'PTY output rendering'→Kitty keyboard protocol + BC-2.09.005 'PTY resize'→bracketed paste) FIXED. 2 registry version bumps (BC-2.09.004→1.0.1, BC-2.09.009→1.1.1) + 2 description corrections (BC-2.09.004 subject, BC-2.09.005 subject) applied atomically (L-S027-004). POL-11 PASS (327 active). POL-12 PASS. EIGHTEENTH consecutive zero-Critical (C:...,0×18). Consecutive-clean counter = 0 (stays 0 — Important present). Pass-25 next = clean candidate 1 of 3. NOTE: SS-09 family now fully re-derived clean per Pass-24 per-BC canonical-bitfield audit. STATE v7.16→v7.17. | 2026-06-13 | state-manager |
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

## Key Tech Stack (D-229 canonical)

ratatui 0.30 | crossterm 0.29 | tokio 1.52 | axum 0.8 | interprocess 2.4 | prost 0.14
serde_yaml_ng 0.10 | wasmtime 44 | nucleo 0.5 | time 0.3.47 (RUSTSEC-2026-0009 floor)
serde_json =1.0.149 | rand =0.8.6 | 28 pinned deps. SS-deps-pin-manifest v1.2.1 (D-235: +tracing-subscriber 0.3 prod, +ureq/libc dev-deps).
MSRV: Rust 1.88 (Phase 1-2); Rust 1.92 (Phase 3, wasmtime 44).
**PRD v1.28.3** | **BC-INDEX v1.40.4** (138 BCs) | **ARCH-INDEX v1.0.28** | **SS-tui v1.8.2**
**SS-ipc v1.17.0** | **SS-conventions v1.32.6** | **ADR-0007 v1.0.8** | **ADR-0008 v1.0.6** | **STORY-INDEX v5.32**
**SS-deps-pin-manifest v1.2.1** | **SS-daemon-wiring-impl v1.3.0** | **SS-session-manager v1.8.1** | **SS-embedded-pty v1.5.0**
**BC-2.05.004 v1.1.0** | **BC-2.05.009 v1.5.1** | **BC-2.05.010 v1.5.0** | **BC-2.06.006 v1.1.0** | **BC-2.06.015 v1.0.7** | **BC-2.06.016 v1.1.0**
**BC-2.06.018 v1.1.0** | **BC-2.06.019 v1.1.0** | **BC-2.06.020 v1.1.0** | **BC-2.06.021 v1.0.7** | **BC-2.06.023 v1.5.1**
**BC-2.06.024 v1.1.0** | **BC-2.07.004 v1.0.2** | **BC-2.07.005 v1.3.1** | **BC-2.09.002 v1.1.0** | **BC-2.09.003 v1.3.0** | **BC-HOOK-034 v1.0.2**
**S-026 v1.11** | **S-027 v1.10** | **product-brief v2.0.4 (draft; consistency-gate-passed)**
**EVAL-INDEX v1.15** | **ADR-0010 v1.5.0** | **SS-daemon-wiring-v2-delta v1.7.0** | **STORY-INDEX v5.32** | **sprint-state v1.40** (32/33 done, 192/195 pts; wave-7 gate PASSED D-232). **S-029 v1.3**. 64 codified disciplines. D-235: Daemon-wiring CONVERGED. D-242: Adv Pass-1 DONE. D-243: Adv Pass-2 DONE. D-244: Adv Pass-3 DONE. D-245: Adv Pass-4 DONE. D-246: Adv Pass-5 DONE (SS-ipc v1.13.0 — NOVEL C5-001: SS-ipc now complete IPC wire authority with all 11 v1A variants). D-247: Adv Pass-6 DONE (SS-ipc v1.14.0 ServerToClient::Error 13th variant + code taxonomy; SS-session-manager v1.6.0 §Error handling; SS-daemon-wiring-v2-delta v1.4.0 error routing; BC-2.05.010 v1.3.0; EVAL-INDEX v1.13). D-248: Adv Pass-7 DONE (FIRST zero-Critical). D-249: Adv Pass-8 DONE (SS-daemon-wiring-v2-delta v1.5.0 §3 all-arms canonical; EVAL-INDEX v1.14). D-250: Adv Pass-9 DONE (ADR-0010 v1.5.0 snapshot-then-resume doc-comments; EVAL-INDEX v1.15 schema v3 label). D-252: Adv Pass-10 DONE (SS-session-manager v1.7.1 re-attach ClientToServer::AttachSession naming; BC-2.05.010 v1.5.0 ordered emission sequence). D-253: Adv Pass-11 DONE (SS-embedded-pty 1.4.0 auto-attach mandate; SS-daemon-wiring-v2-delta 1.6.0 multi-client scope boundary; SS-session-manager 1.7.2; SS-ipc 1.16.0; BC-2.09.001 1.3.0). D-254: Adv Pass-12 DONE (SS-ipc 1.17.0 taxonomy 8→10; SS-session-manager 1.8.0 EngineError bridge; SS-daemon-wiring-v2-delta 1.7.0 spawn call-site; SS-embedded-pty 1.5.0 App struct; BC-2.03.005 1.1.1 worktree_root; BC-2.03.007 1.1.0 canonical codes). D-255: Adv Pass-13 DONE (BC-2.03.005 1.1.2 exhaustive worktree_root; ADR-0009 1.0.1; ADR-0011 1.2.0; L-CWD-PROPAGATION-ATTESTATION). D-256: Adv Pass-14 DONE (SS-session-manager 1.8.1 portable-pty 0.9.x lone stale crate-pin; exhaustive sweep class closed; DEP-PIN-SWEEP-RULE). D-257: Adv Pass-15 DONE (BC-2.05.010 1.5.1; prd 1.28.2). D-258: Adv Pass-16 DONE (prd 1.28.3 §2.8 BC-2.08.008 row; L-VERIFICATION-ARTIFACT-FALSE-GREEN; PRD-COUNT-CROSSCHECK-RULE). D-262: Adv Pass-20 DONE — FIRST CLEAN PASS (0C/0I/2S). BC-2.05.009 1.5.1 (S20-001 title precision). CONSECUTIVE-CLEAN COUNTER = 1. D-263: Adv Pass-21 DONE — SECOND CLEAN PASS (0C/0I/1S). BC-INDEX 1.39→1.39.1 (S21-001 SS-07 header CAP-007 text alignment). CONSECUTIVE-CLEAN COUNTER = 2; Pass-22 next (candidate 3/3 → converges if clean).
9 workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui (S-025).

## Historical Content

| Content | Location |
| Phase-1d §Trace narratives (passes 1-16, v6.90–v7.09) | `cycles/cycle-001/convergence-trajectory.md` |
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
| Phase-1d §Trace narratives (passes 1-16, v6.90–v7.09) | `cycles/cycle-001/convergence-trajectory.md` |


Historical §Trace narratives (passes 1-16, v6.90–v7.09) archived to `cycles/cycle-001/convergence-trajectory.md`.
