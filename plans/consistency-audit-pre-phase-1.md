---
document_type: consistency-audit-report
level: ops
version: "1.0"
status: complete
producer: consistency-validator (fresh context)
phase: pre-phase-1-final-gate
timestamp: 2026-05-12T23:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/dependencies.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/conventions.md
  - /Users/jmagady/Dev/monocle/.factory/tech-debt-register.md
  - /Users/jmagady/Dev/monocle/.factory/plans/brief-validation-v3.md
  - /Users/jmagady/Dev/monocle/.factory/planning/market-intelligence.md
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
input-hash: "[live-state]"
traces_to: "brief v1.3 commit d6a8291; validation v3 commit b3d9560"
project: monocle
verdict: GAPS_FOUND
---

# Consistency Audit — Pre-Phase-1 Final Gate

## 1. Summary

**Verdict: GAPS_FOUND**

No BLOCKING issues. Findings are IMPORTANT or ADVISORY. The spec package is
internally coherent on the primary axes (cross-reference existence, ADR-to-dep
alignment, naming conventions, vision-to-brief semantic fidelity). The gaps
found are correctable without brief revision — most are STATE.md stale fields
and minor cross-artifact discrepancies that the architect should be aware of
before Phase 1 crystallization begins.

| Severity | Count |
|----------|-------|
| BLOCKING | 0 |
| IMPORTANT | 4 |
| ADVISORY | 6 |
| **Total** | **10** |

**Recommendation:** Proceed to human Phase 1 approval gate. The 4 IMPORTANT
findings should be fixed in-place (STATE.md update + brief OQ-M3 note) before
dispatching `/vsdd-factory:create-domain-spec`. No brief re-validation required;
fixes are either STATE.md metadata corrections or advisory notes for the
architect. None of the IMPORTANT findings alter scope or architectural decisions.

---

## 2. Findings Table

| ID | Severity | Category | Location | Description | Recommended Fix |
|----|----------|----------|----------|-------------|-----------------|
| F-01 | IMPORTANT | Cross-reference / stale metadata | `STATE.md` line 41: "Product brief — `.factory/specs/product-brief.md` v1.2" | STATE.md still shows brief at v1.2 in the Project Metadata table. The brief is at v1.3 (commit d6a8291); validation-v3 confirmed VALID. This creates a stale-pointer risk for any agent that reads only the STATE.md header to locate the brief version. | Update STATE.md Project Metadata "Product brief" row to `v1.3 (d6a8291)`. |
| F-02 | IMPORTANT | Cross-reference / stale metadata | `STATE.md` line 14: `current_step: market-intel-and-validate-v2-complete-awaiting-brief-v1.3` | The current_step field describes the state BEFORE brief v1.3 and validation-v3 landed. Brief v1.3 exists; validate-v3 is VALID. The step description is no longer accurate. | Update `current_step` to `brief-v1.3-validated-awaiting-human-phase-1-approval` and update the `awaiting` field from "brief v1.3 (competitive positioning revision) then re-validate then Phase 1" to "human Phase 1 approval gate." |
| F-03 | IMPORTANT | Numerical / count — hook endpoint tension | `brief.md` lines 93-95 (5 endpoints), OQ-M3 in Open Questions table (line 294), and Out-of-Scope bullets (lines 179-181) | OQ-M3 asks whether `PermissionRequest` should be a 6th endpoint (currently `Pending architect review`). The Out-of-Scope section at line 179 explicitly lists "Does NOT include `PostToolUse` hook endpoint in v1" as the out-of-scope boundary for JC-2, but this says nothing about `PermissionRequest`. An architect reading In-Scope (5 endpoints fixed) vs OQ-M3 (6th endpoint pending) has no explicit statement that JC-2's 5-endpoint decision does NOT pre-resolve OQ-M3. The silence creates ambiguity: does JC-2 close OQ-M3, or is OQ-M3 genuinely open? | Add a one-sentence clarification alongside OQ-M3 in the Open Questions table (or inline in the OQ preamble at lines 276-278): "JC-2 closed `PostToolUse` only; `PermissionRequest` was not in scope of JC-2 and remains open per OQ-M3." This removes architect ambiguity without changing any decision. |
| F-04 | IMPORTANT | Version drift — vision vs dependencies.md | Vision §Tech Stack (line 361): `nucleo 0.5`; vision (line 351): `ratatui 0.29`; vision (line 352): `crossterm 0.28`; vision (line 358): `russh 0.45`; vision (line 362): `similar 2.x`; vision (line 364): `notify 7.x`; vision (line 372): `directories 5.x`; vision (line 359): `wasmtime 25.x`; vision (line 357): `interprocess 2.x`; vision (line 360): `prost 0.13` — all compared against `dependencies.md` Phase 1 Pin Manifest | The approved vision's §Tech Stack table carries the v1.0 (pre-OQ-research) version pins, which have been superseded by the verified-against-crates.io versions in `dependencies.md`. Specific deltas: ratatui 0.29 (vision) vs 0.30 (deps); crossterm 0.28 vs 0.29; russh 0.45 vs 0.60; similar 2.x vs 3; notify 7.x vs 8; directories 5.x vs 6; wasmtime 25.x vs 44; prost 0.13 vs 0.14; rmcp 0.3 vs 1.6. The brief at line 218 correctly defers to `dependencies.md` as canonical ("tech stack inherited per vision D-012 — these picks are pre-committed..."). However, the vision is a separate artifact that downstream agents (e.g., disposition-pass, architect) may read for version guidance. The drift is intentional (vision was approved before OQ research updated pins) but undocumented. | Add a note at the top of the vision §Tech Stack section OR in `dependencies.md` §Phase 1 Pin Manifest stating: "Versions in this manifest supersede the vision's §Tech Stack table, which was pinned at vision approval (2026-05-11) before OQ research (2026-05-12) updated all pins to live crates.io verified values. The manifest is canonical." This is an advisory-only documentation fix; no decisions change. Severity: IMPORTANT because an architect reading the vision alone would use stale version pins. |
| F-05 | ADVISORY | Naming — capitalization | brief.md H1; vision H1; conventions.md | Product name is "Monocle" in headings, "monocle" in code/technical contexts. All artifacts consistent with this convention; it is simply unstated in conventions.md. | Add one line to conventions.md: "Product name: lowercase `monocle` in code; capitalized `Monocle` in prose headings." |
| F-06 | ADVISORY | Naming — agent view label | brief.md line 306; market-intel line 31 | Both artifacts use `` `claude agents` `` (product) with "agent view" (feature) consistently. No naming drift. | No fix required; advisory confirmation only. |
| F-07 | ADVISORY | Semantic anchoring — D-012 | brief.md line 218; burst-log.md (archived) | D-012 cited in brief as authority for tech stack approval. Resolves to archived burst-log. Not a phantom — vision Provenance section confirms human approval. | Add "(archived to cycles/cycle-001/burst-log.md)" note in brief Decisions Log Cross-Reference section alongside D-012 mention. |
| F-08 | ADVISORY | Semantic anchoring — R-001 origin | brief.md lines 318-321; market-intelligence.md line 143 | R-001 accepted in brief; formally defined in market-intelligence.md Risk Register. Cross-artifact reference is valid; source not cited inline in brief. | Add "(per market-intelligence.md §Risk Register)" parenthetical after R-001 acceptance statement in brief. |
| F-09 | ADVISORY | Stale frontmatter — oq-research.md | oq-research.md frontmatter line 19 | `brief_version: "1.1"` but resolutions applied through v1.3. No OQ decisions changed in v1.2/v1.3. Misleads agents checking OQ provenance. | Add comment at top of oq-research.md: "brief_version reflects authoring context (v1.1); resolutions valid through brief v1.3." |
| F-10 | ADVISORY | Count drift — crate count | brief.md lines 221-226; vision Workspace Layout | Brief states "13 crates total" but enumeration (brief + vision) yields 12 named crates. Off-by-one unexplained. | Architect must reconcile before create-architecture. Brief or vision should name the 13th crate explicitly or correct count to 12. |

---

## 3. Cross-Reference Integrity

### Supplements frontmatter paths (brief line 23-27)

| Referenced Path | On Disk | Status |
|----------------|---------|--------|
| `.factory/specs/architecture/dependencies.md` | YES (6,744 bytes) | PASS |
| `.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md` | YES | PASS |
| `.factory/specs/architecture/conventions.md` | YES | PASS |
| `.factory/tech-debt-register.md` | YES | PASS |

### Brief inputs paths (brief lines 10-19)

| Referenced Path | On Disk | Status |
|----------------|---------|--------|
| `.factory/specs/research/domain-monocle-vision-synthesis.md` | YES | PASS |
| `.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md` | (semport dir present) | ASSUMED PASS |
| `.factory/semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` | (semport dir present) | ASSUMED PASS |
| `.factory/semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md` | (semport dir present) | ASSUMED PASS |
| `.factory/semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md` | (semport dir present) | ASSUMED PASS |
| `.factory/semport/zellij/zellij-pass-8-final-synthesis.md` | (semport dir present) | ASSUMED PASS |
| `.factory/semport/lazygit/lazygit-pass-8-final-synthesis.md` | (semport dir present) | ASSUMED PASS |
| `.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md` | (semport dir present) | ASSUMED PASS |
| `.factory/semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md` | (semport dir present) | ASSUMED PASS |
| `.factory/planning/oq-research.md` | YES | PASS |

Note: All 8 semport repos confirmed present in `.factory/semport/` directory
listing (any-context-lazyclaude, nikiforovall-lazyclaude, vsdd-factory,
codemachine-cli, zellij, lazygit, claude-squad, claude-code-router). The 8-repo
count is consistent across brief frontmatter (8 inputs), vision frontmatter
(8 inputs), STATE.md phase -1 ("8 repos"), and vision §Provenance ("8 reference
repos"). PASS.

### validation-v3 inputs field

| Field | Expected | Actual | Status |
|-------|----------|--------|--------|
| `inputs[0]` — brief path | `product-brief.md` # v1.3 comment | YES, with inline comment `# v1.3` | PASS |
| `traces_to` | references brief v1.3 commit d6a8291 | "brief v1.3 commit d6a8291; brief v1.2 commit 6ac4279" | PASS |

### ADR-0001 alignment with dependencies.md

| Claim | ADR-0001 | dependencies.md | Status |
|-------|----------|-----------------|--------|
| WASM runtime choice | wasmtime 44 (accepted) | wasmtime 44 (pinned) | PASS |
| wasmi status | Rejected | Not listed (wasmi absent from manifest) | PASS |
| Phase 3 MSRV | 1.92 (ADR Consequences) | 1.92 (MSRV Constraints table) | PASS |
| RUSTSEC pre-44 context | RUSTSEC-2026-0114, 0095, 0096, 0006, 0020 | Same advisory list | PASS |

### TD-001 vs nucleo in brief/dependencies

| Claim | tech-debt-register TD-001 | brief (line 101) | dependencies.md | Status |
|-------|--------------------------|-----------------|-----------------|--------|
| nucleo dormant | YES — "upstream dormant since 2024-04-02" | Listed: "nucleo-matcher" in Sessions panel (`/` filter) | nucleo 0.5 pinned; note "Upstream dormant since 2024-04-02; flagged in tech-debt-register TD-001" | PASS — consistent across all three |
| Phase 1 scope | P1 debt (within 3 cycles) | Phase 1 scope (sessions panel filter) | Phase 1 pin | PASS — TD-001 correctly says "Functionality intact for Phase 1" |

### conventions.md anti-patterns vs brief/ADR

| Anti-Pattern | Consistency with Brief | Consistency with ADR-0001 |
|---|---|---|
| Unbounded event channels forbidden | PASS — brief line 115-116 explicitly mandates bounded mpsc + drop counter | N/A |
| Naked config file writes forbidden | PASS — brief line 113-114 mandates `tempfile::persist` for config; OQ-10 uses atomic writes | N/A |
| Single-popup overlay forbidden | PASS — brief mandates `VecDeque<PromptModal>` (lines 104-107) | N/A |
| No wasmtime-related conventions | wasmtime is Phase 3 deliverable per brief; conventions.md is Phase 1 focus | No conflict with ADR-0001 (ADR does not prescribe code conventions). PASS |

---

## 4. Naming Consistency

| Term | Brief | Vision | Market Intel | Dependencies | ADR-0001 | Conventions | Verdict |
|------|-------|--------|-------------|-------------|----------|-------------|---------|
| Product name (code) | `monocle` | `monocle` | `monocle` | `monocle` (project field) | `monocle` | `monocle` | CONSISTENT |
| Product name (prose) | "Monocle" (headings), "monocle" (technical) | "Monocle" (headings), "monocle" (technical) | "monocle" throughout | N/A | N/A | N/A | CONSISTENT (context-appropriate) |
| Five planes: Runtime | Runtime | Runtime | Runtime plane (Gap Matrix) | N/A | N/A | N/A | CONSISTENT |
| Five planes: Static | Static | Static | (implied, customization-explore) | N/A | N/A | N/A | CONSISTENT |
| Five planes: Workflow | Workflow | Workflow | Workflow plane | N/A | N/A | N/A | CONSISTENT |
| Five planes: Harness | Harness | Harness (EngineModule) | N/A | N/A | N/A | N/A | CONSISTENT |
| Five planes: TUI | TUI (lazygit-style philosophy) | "TUI philosophy" | N/A | N/A | N/A | N/A | CONSISTENT (minor label variant, not a conflict) |
| Ctrl-\ notation | `Ctrl-\` | `Ctrl-\` | N/A | N/A | N/A | N/A | CONSISTENT |
| Anthropic product name | "`claude agents` (agent view, v2.1.139)" | not referenced (predates agent view) | "`claude agents` (agent view, research preview, v2.1.139)" | N/A | N/A | N/A | CONSISTENT — brief and market intel use identical notation |
| Persona: "Multi-session Claude Code developer" | Yes (line 62) | implied in killer scenario | "Multi-session developer" | N/A | N/A | N/A | MINOR VARIANT — acceptable (not a conflict) |
| VecDeque notation | `VecDeque<PromptModal>` | `VecDeque<PromptModal>` | "VecDeque permission overlay" (abbreviated) | N/A | `VecDeque<PromptModal>` | N/A | CONSISTENT |
| 5-level binding precedence | "SearchPrompt > UserCustomCommand > PerContext > Global > Builtin" | same 5 levels, same order | N/A | N/A | N/A | N/A | CONSISTENT |
| Hook endpoints (5) | PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit | PreToolUse, PostToolUse, Stop, PermissionPrompt (vision diagram — OLD) | N/A | N/A | N/A | N/A | SEE F-11 below |

### F-11 (IMPORTANT — moved from findings): Vision Process Topology diagram endpoint mismatch

The vision §Process Topology diagram (lines 61-64) shows:
```
├── PreToolUse hook  ──► POST .../hooks/pre-tool-use
├── PostToolUse hook ──► POST .../hooks/post-tool-use
├── Stop hook        ──► POST .../hooks/stop
└── PermissionPrompt ──► POST .../hooks/permission
```

This is a 4-endpoint set that includes `PostToolUse` and `PermissionPrompt`
but omits `Notification`, `SessionStart`, and `UserPromptSubmit`.

The brief's Phase 1 scope (lines 92-95) correctly resolves to 5 endpoints:
PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit — per JC-2
(PostToolUse omitted) and EX-2 (SessionStart and UserPromptSubmit added).

The vision diagram predates OQ/JC/EX resolutions. The vision as an approved
artifact cannot and should not be revised. However, this means any downstream
agent reading the vision's Process Topology diagram will see a different
endpoint set than the brief specifies.

**Severity: IMPORTANT** — this will mislead the architect or domain-spec
agent during Phase 1 crystallization if they consult the vision diagram for
endpoint enumeration rather than the brief.

**Recommended fix:** Add a note in the brief's Constraints section (after
line 218) cross-referencing the endpoint set: "Note: the vision §Process
Topology diagram reflects the pre-OQ-research topology and lists 4 endpoints
including PostToolUse and PermissionPrompt. The canonical Phase 1 5-endpoint
set (JC-2, EX-2 resolutions) is defined above — the vision diagram is
superseded for this purpose."

| Term | Count | Consistent |
|------|-------|-----------|
| Five planes | 5 | YES — all artifacts that reference planes name the same 5 |
| Reference repos | 8 | YES — brief, vision, STATE.md, semport directory all say 8 |
| Workspace crates | 13 (brief) / 12 (enumerated) | NO — see F-10 |
| Hook endpoints (Phase 1) | 5 (brief) / 4 (vision diagram) | MISMATCH — see F-11 |
| MSRV Phase 1 | Rust 1.86 (brief, deps, ADR) | CONSISTENT |
| MSRV Phase 3 | Rust 1.92 (brief, deps, ADR) | CONSISTENT |

---

## 5. Semantic Anchoring

### Vision-to-Brief Alignment

| Vision Claim | Brief v1.3 Representation | Status |
|-------------|--------------------------|--------|
| "One TUI lens over every Claude-class session" | Lines 33-44: consistent verbatim quote from vision §Vision Statement | PASS |
| Observe-only for state, action-only via overlays | Lines 42-43: "observe-only for workflow state and session transcripts; it owns the action layer only for permission prompts and keybinding dispatch" | PASS |
| Five planes: Runtime, Static, Workflow, Harness, TUI | Lines 37-41: all five named correctly with same descriptions | PASS |
| VsddFactoryAdapter statically bundled in v1; WASM SDK Phase 3 | Lines 122-123 and Phase 3 scope: correct | PASS |
| 13-crate workspace layout | Lines 221-226: listed — but count discrepancy (see F-10) | PARTIAL (F-10) |
| Killer scenario: 4 keystrokes | Lines 68-71 and Success Criteria table (line 193): brief says "≤6 keystrokes (per vision §End-to-End Killer Scenario target: 4)" — vision says "3 keystrokes" in step description but "4 keys" in the same scenario (the vision §End-to-End paragraph says "Ctrl-\, 2, 1, Ctrl-\ = 4 keys") | PASS — the brief correctly quotes the vision's own "4 keys" formulation |
| FactoryAdapter detection canonical signal: `.factory/STATE.md` with `document_type: pipeline-state` | Phase 1 Success Criteria (line 194): "Detection succeeds on monocle's own `.factory/`" — self-referential test | PASS |
| CCR integrate-external: detect on PATH, write per-session JSON, set `ANTHROPIC_BASE_URL` | Lines 170-177 Out-of-Scope + brief lines 242-244 | PASS |

### Vision D-012 reference

D-012 is cited in the brief (line 218, 228, 244) as the source of the
human-approved tech stack commitment. The vision §Provenance (line 386-387)
says "The human approved it verbatim 2026-05-11." STATE.md Decisions Log shows
D-001..D-015 archived to `cycles/cycle-001/burst-log.md`. D-012 resolves to
the canonical vision document's human approval event — not a phantom. PASS.

### Market intel claim in brief

Brief competitive positioning (lines 307-309): "Agent view provides session
list + inline reply built into Claude Code's TUI: no hook protocol, no
external overlay, no diff preview, no cascaded permission queue, no
customization visibility, no workflow plane, no multi-harness support."

Market intel (lines 161-170): "It does not: Use hook protocol for structured
event ingestion... Show diff preview in a permission overlay... Surface
customization... Surface factory/workflow pipeline state... Support
multi-harness..."

The brief's characterization of agent view matches the market intel's
enumeration item-by-item. The claim "no cascaded permission queue" is a brief
addition not in the market intel's bullet list, but it is a reasonable
inference from "no hook protocol" + "no external overlay." PASS.

### R-001 semantic origin

R-001 is formally defined in market-intelligence.md §Risk Register (line 143):
"Anthropic ships hook-native permission overlay in agent view." Brief v1.3
accepts R-001 with probability 25-40% and mitigation. Market intel lists
R-001 severity as CRITICAL × MEDIUM = same probability range (25-40% per
brief). Consistent. PASS.

---

## 6. Frontmatter / Template Compliance

| Artifact | document_type | level | version | status | producer | timestamp | inputs | traces_to | project |
|----------|--------------|-------|---------|--------|----------|-----------|--------|-----------|---------|
| product-brief.md | product-brief | L1 | 1.3 | draft | product-owner | YES | YES | YES | monocle |
| domain-monocle-vision-synthesis.md | vision-synthesis | ops | 1.0 | approved | orchestrator | YES | YES | YES | monocle |
| dependencies.md | architecture-dependencies | L3 | 1.0 | stub | product-owner | YES | YES | YES | monocle |
| ADR-0001 | adr | L3 | 1.0 | (accepted — in body, not frontmatter `status` field) | product-owner | YES | YES | YES | monocle |
| conventions.md | architecture-conventions | L3 | 1.0 | stub | product-owner | YES | YES | YES | monocle |
| tech-debt-register.md | tech-debt-register | (missing level field) | 1.0 | (missing status field) | product-owner | (no inputs field) | (no traces_to) | monocle | PARTIAL |
| brief-validation-v3.md | brief-validation-report | ops | 3.0 | complete | product-owner | YES | YES | monocle | PASS |
| market-intelligence.md | market-intelligence-assessment | L1 | 1.0 | complete | business-analyst | YES | YES | YES | monocle |
| oq-research.md | open-questions-research | pre-architecture | 1.0 | draft | research-agent | YES | YES | (no traces_to) | monocle — PARTIAL |

**Frontmatter gaps (advisory):**

- `tech-debt-register.md`: missing `level`, `status`, `inputs`, `traces_to`
  fields. As a register document (not a spec artifact), these may be
  intentionally omitted. ADVISORY.
- `oq-research.md`: missing `traces_to` field. Level is `pre-architecture`
  (non-canonical). ADVISORY.
- `ADR-0001`: body status "Accepted" but `status: accepted` not in standard
  format for a VSDD adr (should be `status: accepted` in frontmatter — it is
  present via the `status: accepted` block, but the `date` field uses date not
  ISO datetime). ADVISORY.

---

## 7. Verdict and Recommendation

**Overall verdict: GAPS_FOUND**

**Blocking issues: ZERO**

### Consolidated finding list by priority

1. **F-11 (IMPORTANT)** — Vision Process Topology diagram shows 4 outdated
   endpoints (incl. PostToolUse, PermissionPrompt) vs brief's canonical 5-
   endpoint set. Will mislead architect if vision diagram is consulted for
   endpoint enumeration. Fix: one-sentence note in brief Constraints section.

2. **F-01 (IMPORTANT)** — STATE.md Project Metadata shows brief at v1.2.
   Stale pointer. Fix: update STATE.md "Product brief" row to v1.3.

3. **F-02 (IMPORTANT)** — STATE.md current_step and awaiting fields are
   pre-v1.3, describing a pending action that has already completed.
   Fix: update both fields to reflect post-validation-v3 state.

4. **F-04 (IMPORTANT)** — Vision §Tech Stack carries pre-OQ version pins
   (ratatui 0.29, crossterm 0.28, russh 0.45, wasmtime 25.x, etc.) diverging
   significantly from dependencies.md canonical pins. No downstream agent
   should use vision §Tech Stack for version guidance. Fix: document
   in dependencies.md that it supersedes the vision's tech stack table.

5. **F-03 (IMPORTANT)** — OQ-M3 and JC-2 relationship unclear: does JC-2
   (5-endpoint decision) also close the PermissionRequest question? Brief
   is silent. Fix: one-sentence clarification in OQ-M3 table row.

6. F-10 (ADVISORY) — Crate count: "13 crates total" but enumeration yields 12.

7. F-09 (ADVISORY) — oq-research.md `brief_version: "1.1"` stale metadata.

8. F-08 (ADVISORY) — R-001 in brief not cross-referenced to market-intel source.

9. F-07 (ADVISORY) — D-012 reference in brief not cross-referenced to archive.

10. F-05 (ADVISORY) — Monocle/monocle capitalization convention not stated
    explicitly in conventions.md (though usage is consistent).

### Gate decision

**Proceed to human Phase 1 approval gate.**

The 4 IMPORTANT findings (F-11, F-01, F-02, F-04) are:
- All correctable in minutes with targeted line edits (F-01, F-02: STATE.md
  updates; F-11, F-03: one-sentence additions to brief; F-04: note in deps.md)
- None change any architectural decision or scope
- None alter the brief's Phase 1 success criteria, constraints, or OQ resolutions

The spec package is internally coherent on all decision-bearing axes:
ADR-to-deps alignment, vision-to-brief semantic fidelity, anti-pattern
consistency, TD-001/nucleo consistency, supplements existence, 8-repo count,
five-plane naming, endpoint count (in the brief), MSRV (both phases), and
OQ/SOQ/JC resolution status.

**Recommended pre-gate actions (not gate-blocking if skipped):**
1. Update STATE.md fields (F-01, F-02) — takes 2 minutes
2. Add one-sentence note to brief Constraints about vision diagram supersession
   (F-11) — takes 1 minute
3. Add note to dependencies.md that it supersedes vision §Tech Stack (F-04)
   — takes 1 minute

**If the human approves Phase 1 without these fixes:** the architect must
be verbally briefed that vision §Process Topology diagram endpoint set is
outdated and the canonical endpoint list is in the brief.
