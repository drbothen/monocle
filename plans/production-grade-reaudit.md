---
document_type: adversarial-re-audit-report
level: ops
version: "1.0"
status: complete
producer: adversary (fresh context, production-grade lens) — transcribed by orchestrator
phase: pre-phase-1-final-gate
timestamp: 2026-05-12T23:30:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/dependencies.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/conventions.md
  - /Users/jmagady/Dev/monocle/.factory/tech-debt-register.md
  - /Users/jmagady/Dev/monocle/.factory/plans/brief-validation-v2.md
  - /Users/jmagady/Dev/monocle/.factory/plans/brief-validation-v3.md
  - /Users/jmagady/Dev/monocle/.factory/plans/consistency-audit-pre-phase-1.md
  - /Users/jmagady/Dev/monocle/.factory/planning/market-intelligence.md
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
  - /Users/jmagady/Dev/monocle/.factory/semport/
input-hash: "[live-state]"
traces_to: "brief v1.3 commit d6a8291; consistency audit b891b78; production-grade canonical principle (orchestrator turn)"
project: monocle
verdict: MULTIPLE_DEFER_PATTERNS
---

# Adversarial Re-Audit — Production-Grade Lens

## Top-line verdict

**MULTIPLE_DEFER_PATTERNS** — 14 production-grade violations found: 5 CRITICAL, 6 IMPORTANT, 3 ADVISORY. The session work converged on procedural validity (validate-brief VALID, consistency audit GAPS_FOUND non-blocking) but the new lens reveals a consistent pattern: blockers were softened to "advisory," answerable questions were tagged "pending architect review," and stub artifacts contain explicit TODO/Placeholder defer-patterns. Under the canonical principle these are remediation items, not gate-pass conditions.

## Top 8 CRITICAL / IMPORTANT findings

1. **CRITICAL** — `dependencies.md` lines 70-73 §"Phase 2/3/4 Additions" reads "Placeholder for architect" — textbook forbidden defer-pattern; Phase 2/3/4 additions are knowable now from oq-research §Theme-3 trait seams and brief Phase 2-4 scope bullets.
2. **CRITICAL** — `dependencies.md` lines 120-127 §"Architect TODO" lists 6 items (dual-MSRV strategy, patch-version pinning policy, `bytes` direct-pin verification, version-bump policy, rmcp omission confirmation, dep-graph diagram); 4 of 6 are answerable in-scope and 2 are AI-content the AI deferred (`patch-version pinning policy` and `bump policy` are policy questions resolvable by selecting the conventional choice with rationale).
3. **CRITICAL** — `conventions.md` lines 52-58 §"Test-Time Enforcement" reads "Placeholder for architect"; lines 61-67 §"Architect TODO" lists 5 items (clippy lint entries, semgrep rule, PR template, write-path confirmation, CI wiring) — all are concrete and codifiable now from the anti-patterns table already in the same file.
4. **CRITICAL** — Brief v1.3 OQ table lines 297-298 carries OQ-M1 and OQ-M3 with Resolution = "Pending architect review (market intel)". Both have answerable production-grade resolutions: OQ-M1 (agent-view IPC) is researchable from public Anthropic docs cited in market-intelligence.md line 222; OQ-M3 (`PermissionRequest` as 6th endpoint) is resolvable by re-reading the brief's own JC-2 rationale — if Phase 1 omits `PostToolUse` for parity with the 5-endpoint canonical matrix, the same parity argument resolves OQ-M3 to "stay at 5; revisit if Phase 2 trigger-trace needs the extra signal."
5. **CRITICAL** — `tech-debt-register.md` TD-001 (nucleo dormant) was not human-directed per the user's stated rule that the register is for HUMAN-DIRECTED deferrals only. TD-001 was AI-introduced from the OQ research narrative. Under the new lens TD-001 must be either resolved now (pin nucleo-picker 0.11 fork OR frizbee 0.9 with explicit ADR) or migrated to a Phase 2 story with a re-eval gate. The current "Phase 2 re-eval" Due column with no story anchor is exactly the unpinned-deferral the rule forbids.
6. **IMPORTANT** — `STATE.md` Skip Log line 85: "DTU Assessment | pending | Deferred until architecture complete" — DTU is for external-surface fidelity tracking. The hook protocol IS an external surface (Claude Code's contract). DTU should be RUN NOW for the 5 hook-endpoint schemas with a flag for `PermissionRequest` per OQ-M3. The defer-until-architecture-complete framing is the wrong default.
7. **IMPORTANT** — Brief v1.3 lines 96-99 Process-Topology supersession note ("vision diagram is non-authoritative for endpoint enumeration") is the right semantic but the wrong remediation pattern: the vision document itself was approved verbatim 2026-05-11 and is now demonstrably stale on the 5-vs-4 endpoint set AND on 10+ tech-stack version pins. The supersession-notes-in-downstream-artifacts pattern shifts the burden onto every future reader of the vision. A v1.1 re-approval would fix it once.
8. **IMPORTANT** — Brief v1.3 line 226: "13 crates total" but enumeration on lines 226-229 yields 11 named crates + binary = 12. The consistency audit caught this (F-10) and tagged it ADVISORY with "architect must reconcile." Under the new lens this is a numerical defect in an L1 artifact that MUST be resolved by the AI that introduced it. The vision §Workspace Layout (line 76 onward) also says 11 named crates + 1 binary. The brief's "13" is just wrong; correct it to "12."

## Additional IMPORTANT findings (9–14)

9. **IMPORTANT** — `brief-validation-v2.md` line 216 (B-4): non-canonical stub paths tagged "ADVISORY / LOW / architect migrates on first touch." Under the new lens this is a defer-pattern: the migration is mechanical (mv + frontmatter `section:` field update), in-scope, and answerable now without architect input.
10. **IMPORTANT** — `oq-research.md` frontmatter line 19 `brief_version: "1.1"` is stale. Consistency audit F-09 tagged ADVISORY with "add a comment at top" — the production-grade fix is to bump the field to "1.3" with a one-line resolution-validity note.
11. **IMPORTANT** — Brief v1.3 §Competitive Positioning line 325-326: "Mitigation: ship Phase 1 fast" is an MVP-shaped phrase masquerading as risk mitigation. R-001 mitigation should be a concrete artifact (e.g., "trigger-trace BC anchors in Phase 2 PRD; workflow-plane FactoryAdapter trait stability ADR") not a velocity exhortation. "Ship fast" is not a mitigation; it is a hope.
12. **ADVISORY** — Reference-ingest pass-naming inconsistency: 8 final-synthesis files, 5 use `*-pass-8-final-synthesis.md`, 2 use `*-pass-8-final-synthesis-v2.md` (any-context, nikiforovall — re-run), 1 uses `*-pass-8-deep-synthesis.md` (claude-squad), 1 uses `*-pass-C-final-synthesis.md` (claude-code-router — non-numeric phase). Process-gap; brownfield-ingest protocol does not enforce a single final-synthesis filename convention. All 8 content-substantive. Severity: keep ADVISORY with `[process-gap]` tag.
13. **ADVISORY** — `tech-debt-register.md` missing `level`, `status`, `inputs`, `traces_to` frontmatter fields. Production-grade requires complete metadata.
14. **ADVISORY** — `oq-research.md` missing `traces_to` frontmatter and uses non-canonical `level: pre-architecture`.

## My 5 questions — defensible or my-defer?

| # | Question | Defensible? | One-sentence reason |
|---|---|---|---|
| Q1 | 5 vs 6 endpoints (`PermissionRequest`) | **NO — my-defer** | OQ-M3 is researchable from the cited Anthropic docs and the JC-2 parity rationale already in the brief; the answer is "5 — same Claude-Code-gene-source parity argument that closed JC-2 also closes M3, with explicit revisit at Phase 2 trigger-trace if signal gap surfaces." |
| Q2 | `dependencies.md` authority over vision §Tech Stack | **YES — defensible** | Cross-artifact supersession is a versioning/governance decision with two legitimate human-decidable paths (re-version vision v1.1 vs supersession-note pattern). |
| Q3 | R-001 risk acceptance (25-40%) | **YES — defensible** | Risk-acceptance probability is a human business decision; AI can recommend but not unilaterally accept. |
| Q4 | Non-canonical stub paths | **NO — my-defer** | The migration (mv + frontmatter `section:` field update + path-registry entry) is mechanical, in-scope, and does not require architect adjudication. |
| Q5 | DTU assessment reopening | **NO — my-defer** | The hook protocol is an external surface with a public reference schema (any-context hooks-r1); DTU on those 5 endpoints can be run now without architectural prerequisites. |

## Production-grade resolutions for OQ-M1 / OQ-M2 / OQ-M3

- **OQ-M1 (agent-view IPC coexistence):** Research-in-scope. Per Anthropic docs link in market-intelligence.md line 222 (`https://code.claude.com/docs/en/agent-view`), agent view dispatches via Claude Code's own internal IPC (not hook protocol POSTs); monocle's daemon on an OS-assigned port + `X-Claude-Code-Ide-Authorization` header cannot collide because the agent-view surface does not bind a TCP port. Recommendation: resolve OQ-M1 as "No collision; agent view operates inside Claude Code's process tree, monocle ingests via outbound hook POSTs from Claude Code subprocesses to monocle's daemon. No shared port or auth surface." *(Confidence: MEDIUM — would benefit from a 5-minute WebFetch to confirm against current Anthropic docs.)*
- **OQ-M2 (claude-manager hook protocol use):** Research-in-scope via cited libs.rs URL. From market intel gap-matrix line 50 (`claude-manager... hook-overlay: NO`), claude-manager does NOT use hook protocol — it uses tmux + worktrees per market intel description on line 64. Recommendation: resolve OQ-M2 as "claude-manager uses tmux pane management, NOT hook protocol — the hook-native architectural moat is intact."
- **OQ-M3 (`PermissionRequest` as 6th endpoint):** Resolvable in-scope by reading the existing JC-2 rationale. JC-2 omitted `PostToolUse` per Claude Code gene-source parity (any-context BC-HOOK-007 canonical 5-endpoint set). The same parity argument resolves M3 to "stay at 5; the `PermissionRequest` event is upstream of `PreToolUse` and the brief's existing VecDeque overlay receives all permission-relevant signal via `PreToolUse` + `Notification`. Revisit if Phase 2 trigger-trace UX testing demonstrates a needed signal gap." Confidence HIGH. Decision is final unless human red-lines.

## Vision re-versioning recommendation

**YES — re-version to v1.1, request human re-approval.** The vision is demonstrably stale on two load-bearing axes — 4-vs-5 endpoints AND 10+ version pins — and the supersession-note-in-downstream-artifacts pattern requires every future reader to chase three artifacts to assemble correct state, which is the inverse of production-grade source-of-truth discipline.

Counter-position: the supersession pattern preserves the original human-approval event and avoids re-approval friction. This is the cheaper path. But under the canonical principle "default = correct path, suggest cheaper paths," the correct path is v1.1 re-approval, capturing the human's existing intent (the human red-lined PostToolUse + asked for full endpoint parity via EX-2, and approved the OQ pin updates via JC closures) into a stable, single-source-of-truth vision document. The work is ~10 minutes of edits; the readability win is permanent.

## Orchestrator recommendation

**(b) Run a remediation burst BEFORE re-presenting the Phase 1 gate.** Specifically:

1. **Fix the AI-introduced defects in-scope:**
   - Crate count: brief 13 → 12 (verify count, fix both brief and vision Workspace Layout)
   - Resolve OQ-M1/M2/M3 via the research/parity arguments above (in-scope where possible; WebFetch for OQ-M1 confirmation)
   - Resolve all 6 `dependencies.md` and 5 `conventions.md` Architect-TODO items by writing the conventional production answers (caret-pinning, semgrep rules from anti-pattern table already in the same file, etc.)
   - Remove "Placeholder for architect" sections from both stubs; replace with concrete content
   - Migrate stub files to canonical artifact-path-registry paths
   - Update `oq-research.md` frontmatter to `brief_version: "1.3"` and add `traces_to` field
2. **Re-version vision to v1.1** with EX-2 endpoint set, refreshed Tech Stack table pointing to `dependencies.md` as authority, and the JC-2/EX-2 closure log. Request human re-approval.
3. **Re-anchor TD-001:** either commit to nucleo via explicit ADR-0002 with re-eval trigger, or migrate to a Phase 2 story with a story-ID anchor. Current "Phase 2 re-eval" with no story anchor is exactly the unpinned-deferral the rule forbids.
4. **Re-classify R-001 mitigation** from "ship Phase 1 fast" to concrete Phase 2/3 BC anchors (trigger-trace BCs, workflow-plane FactoryAdapter trait ADR).
5. **Run DTU assessment NOW** on the 5 hook endpoints, not after architecture.

After the remediation burst, only Q2 (deps.md authority + vision re-versioning approach) and Q3 (R-001 acceptance) remain for human decision. Q1, Q4, Q5 are removed from the question set.

## Re-classification of consistency-audit findings under the new lens

| ID | Old severity | New severity | Disposition |
|---|---|---|---|
| F-01 (STATE.md brief v1.2 stale) | IMPORTANT | FIX-NOW | mechanical update; AI defect (already fixed via state-manager) |
| F-02 (STATE.md current_step stale) | IMPORTANT | FIX-NOW | already fixed |
| F-03 (JC-2/OQ-M3 ambiguity) | IMPORTANT | FIX-NOW + close OQ-M3 in-scope | the ambiguity goes away when OQ-M3 is resolved to "stay at 5" |
| F-04 (vision tech-stack drift) | IMPORTANT | PROMOTE → CRITICAL | drives the v1.1 re-versioning recommendation |
| F-05 (Monocle vs monocle convention) | ADVISORY | FIX-NOW | one-liner in conventions.md |
| F-06 (claude agents label) | ADVISORY | KEEP (no fix needed) | confirmation only |
| F-07 (D-012 archive note) | ADVISORY | FIX-NOW | trivial parenthetical |
| F-08 (R-001 origin pointer) | ADVISORY | FIX-NOW | trivial parenthetical |
| F-09 (oq-research.md brief_version 1.1) | ADVISORY | PROMOTE → IMPORTANT FIX-NOW | stale frontmatter on a load-bearing artifact misleads consumers |
| F-10 (13 vs 12 crates) | ADVISORY | PROMOTE → CRITICAL FIX-NOW | numerical defect in L1 brief is not advisory under production-grade lens |
| F-11 (vision diagram endpoints) | IMPORTANT | PROMOTE → CRITICAL via v1.1 vision re-version | best fixed at the source, not by supersession note |

## validate-brief v2 / v3 critique

- **v2 (NEEDS_WORK)**: Blocker enumeration is mostly correct. B-1 (agent view) was correctly HIGH. But B-2/B-3 ADVISORY framings of OQ-M1/M3 are under-classified — both have in-scope answers, so they should have been "fix-now blockers." B-4 (non-canonical stub paths) is the textbook defer-pattern.
- **v3 (VALID)**: Flipped Competitive Positioning to PASS based on the rewrite — under the new lens the rewrite is minimally compliant. The "ship Phase 1 fast" mitigation is not production-grade. v3 also carried forward all 4 ADVISORY blockers (B-2/B-3/B-4) without re-evaluating them, which is exactly the propagation-of-deferral pattern the new lens forbids.

## Reference-ingest pass-naming process gap

8 final-synthesis files exist with inconsistent naming:
- 5 use `*-pass-8-final-synthesis.md`
- 2 use `*-pass-8-final-synthesis-v2.md` (any-context, nikiforovall — re-run)
- 1 uses `*-pass-8-deep-synthesis.md` (claude-squad)
- 1 uses `*-pass-C-final-synthesis.md` (claude-code-router — non-numeric phase)

All 8 files are content-substantive (cross-checked against frontmatter where present and citations from oq-research.md), but the naming drift makes consistency checks harder. Severity: ADVISORY. Tagged `[process-gap]` per cycle-closing checklist. The brownfield-ingest skill in vsdd-factory should enforce a single final-synthesis filename convention.

## Production-readiness verdict

The spec package is NOT production-ready under the new lens. Procedurally valid (VALID + GAPS_FOUND non-blocking) but substantively carrying defer-patterns the canonical principle rejects. Recommended path: remediation burst (above), then re-present Phase 1 gate with only Q2 and Q3 as open questions.
