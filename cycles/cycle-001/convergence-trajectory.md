# Phase-1d Adversarial Convergence Trajectory
# Extracted from STATE.md on 2026-06-14 during STATE.md compaction (health: NEEDS-COMPACT→HEALTHY)
# Contains: Phase-1d adversarial passes 1-16 historical §Trace narratives (v6.90–v7.09)
# These are the convergence-fix narratives for the control-center spec package adversarial review.
# Passes 17-37 narratives are captured in STATE.md §I Decision Log entries (D-271..D-279)
# and in the next_session_resume_protocol block (Section A, finding trajectory).

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
