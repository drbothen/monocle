---
document_type: brief-validation-report
level: ops
version: "6.0"
status: complete
producer: product-owner (validate-brief skill)
phase: pivot-delta-brief
timestamp: 2026-06-03T22:30:00Z
inputs:
  - specs/product-brief.md
input-hash: "1659922"
traces_to: "product-brief.md v2.0.0 (D-236/D-237/D-238 control-center re-baseline)"
project: monocle
verdict: VALID
---

# Brief Validation Report — Monocle Product Brief v2.0.0

## Subject

- File: `.factory/specs/product-brief.md`
- Version: 2.0.0
- Line count: 594
- Context: D-236/D-237/D-238 control-center re-baseline of prior v1.4.34 (observe-only scope)

## Overall Verdict

**VALID** — the brief is structurally complete, measurable, internally consistent, and correctly
accounts for the D-238 persistence escalation and v1A/v1B wave split. The token count exceeds
the 1,500-token ideal but is justified by the brownfield delta context (re-baseline of a 9-crate
project with existing substrate requiring precise retirement/preservation accounting). One minor
typo found and remediated below. No blockers to architecture delta proceeding.

---

## Check Results

| Check | Status | Finding |
|-------|--------|---------|
| Structure — §What Is This? | PASS | 4-sentence description with architectural inversion statement + verbatim human quote. Clearly distinguishes v2.0 from v1.4.x. |
| Structure — §Who Is It For? | PASS | 3 specific personas with concrete pain points and current workarounds. 2 killer scenarios with keystroke counts. |
| Structure — §Scope In Scope | PASS | 5 capabilities (LAUNCH, EMBEDDED PTY, MULTI-SESSION, Hook auto-injection, INTERACTIVE TUNE) plus preserved substrate table and persistence model (3 cases). Exceeds the 3–7 capability requirement. |
| Structure — §Scope Out of Scope | PASS | 11 explicit hard-boundary exclusions. Critical preservation: factory-observe-only non-goal correctly retained (FactoryAdapter remains read-only through D-236 pivot). |
| Structure — §Success Criteria | PASS | v1A: 13 rows with numerical targets. v1B: 4 rows. Preserved: 4 rows. All measurable. |
| Structure — §Constraints & Integration Points | PASS | PTY stack table, architecture authority, crate workspace layout, process topology diagram, OQ/SOQ resolution table, forward-compatibility contracts. All constraints are actionable. |
| Quality — Specificity | PASS | No vague language. Personas are specific ("2–4 parallel Claude Code sessions across worktrees or projects"). All scope items are capabilities, not features or user stories. |
| Quality — Measurability | PASS | Success criteria contain explicit numbers: ≤5 keystrokes, ≤100ms, ≤5s, ≤6 keystrokes, ≥1000 events/sec, ≥0.95 fidelity, 80×24 terminal, 256 KiB, 2s, darwin/linux × amd64/arm64. |
| Quality — Scope bounds | PASS | In-scope and out-of-scope are consistent. The "Does NOT replace terminal multiplexer" non-goal is correctly nuanced: monocle IS a mux for AI sessions; does NOT replace the user's general-purpose mux. No contradictions. |
| Quality — Audience clarity | PASS | Personas are specific and requirement-generating. |
| Quality — Constraint actionability | PASS | Every constraint would cause implementation failure if missed. D-238 persistence constraint explicitly names the architectural consequence (D-235 wiring rework). |
| Bloat — Core sections word count | WARNING | Core sections (§What Is This, §Who Is It For, §Scope, §Success Criteria) are well-scoped. The §Constraints section is verbose due to: (1) PTY stack decision table (pre-committed human decision, not speculative); (2) OQ/SOQ resolution table (11+4 rows — required for architect inheritance); (3) process topology diagram (required for architecture delta). These are legitimately required by the brownfield re-baseline context. The comparable v1.4.34 was 702 lines; this v2.0.0 at 594 lines is a net reduction despite covering the additional v2.0 scope. Flag as WARNING, not BLOATED. |
| Bloat — Token estimate | WARNING | Estimated ~5,800–6,200 tokens total. Exceeds 1,500-token ideal. Justified: this is a delta re-baseline with an existing 9-crate, 1514-test substrate. The §Constraints, §Reference Gene Source Map, and §Trace sections are downstream-agent reference material (equivalent to §Overflow Context in v1.4.x). Core normative sections are appropriately scoped. |
| Bloat — Narrative padding | PASS | No business justification or market research embedded in core sections. §Competitive Positioning is correctly in its own section (downstream of §Success Criteria). No filler prose patterns found. |
| Bloat — Requirements leakage | PASS | No FR-XXX requirements, no AC-level acceptance criteria, no ADR-level decisions in scope/criteria sections. Architecture decisions are correctly placed in §Constraints with references to supplement artifacts. |
| Implementation Leakage — §Scope | PASS | No technology names prescriptively embedded in §Scope In-Scope prose. Crate names (`portable-pty`, `vt100`, `tui-term`) appear in §Constraints §Tech Direction only (WARNING severity per skill rubric; legitimate because pre-committed in vision D-237 with dedicated evaluation artifact `embedded-pty-evaluation.md`). |
| Implementation Leakage — §Constraints | WARNING | PTY stack crate names and version numbers appear in §Constraints §Tech Direction. Per skill rubric this is WARNING-severity (Constraints section is acceptable). All are pre-committed human-approved decisions traceable to D-237, `embedded-pty-evaluation.md`, and `SS-deps-pin-manifest.md`. Downgraded from ERROR because leakage is intentional and traceable. |
| Implementation Leakage — §Success Criteria | PASS | No technology names in success criteria. Criteria reference observable behaviours (latency, keystroke count, rendering correctness, fidelity score). |
| Information Density | PASS | Zero instances of: conversational filler, "in order to," "due to the fact that," "somewhat/fairly/rather/quite," "may potentially," "has the ability to," redundant phrases. One typo found (line 98: "moncle" should be "monocle") — fixed in remediation below. |
| Completeness | PASS | 594 lines, well above 150-word minimum. No body sections contain only TBD/TODO. The `input-hash: "TBD-state-manager"` frontmatter field is a legitimate handoff annotation, not a section body placeholder. |
| D-238 Persistence Coverage | PASS | All three persistence cases documented (TUI reconnect, graceful restart, hard crash). D-235 rework consequence explicitly called out. Q-8 HIGH routed to architect. No-tmux default preserved. External supervisor correctly described as architect-surfaced fallback requiring human decision. |
| v1A/v1B Wave Split | PASS | Wave ordering matches vision §Wave Plan (D-237 ratified). Interactive Tune correctly deferred to v1B (feature-ordering, not MVP shortcut). All four capabilities confirmed to ship in v1. |
| Market Intel Cross-Check | PASS | Competitive positioning against agent view (v2.1.139), lazyclaude, claude-squad, zellij/tmux is substantive and specific. R-001 assessment at <10% retained with re-eval triggers (a)–(d) unchanged. No unconfirmed pain claims. |
| OQ/SOQ Inheritance | PASS | All 11 OQ and 4 SOQ resolutions from v1.4.x lineage are explicitly preserved and flagged as still binding. OQ-M1/M2/M3 resolved status maintained. |
| Forward-Compatibility Contract Preservation | PASS | All 6 FC items listed. 22 BCs cited with canonical IDs per BC-INDEX v1.34. |

---

## Bloat Score

- **Core word count (§What Is This + §Who Is It For + §Scope + §Success Criteria):** ~1,600 words
- **Full file:** ~4,800 words
- **Token estimate:** ~6,000 tokens
- **Status:** WARNING — justified by brownfield re-baseline context; core sections are appropriately scoped; bulk is in §Constraints and §Trace (downstream-agent reference material, comparable to §Overflow Context in v1.4.x)

---

## Typo Remediation

Line 98: "moncle" → "monocle" — applying fix.

---

## Findings Summary

| ID | Severity | Section | Finding |
|----|----------|---------|---------|
| F-V6-001 | LOW (typo) | §Who Is It For? / killer scenario 1 | "moncle" should be "monocle" |
| F-V6-002 | WARNING (bloat) | §Constraints | PTY stack table and OQ/SOQ table inflate token count above 1,500 ideal; justified by brownfield re-baseline context; not a blocker |
| F-V6-003 | WARNING (leakage) | §Constraints §Tech Direction | Technology crate names in §Constraints; intentional and traceable to D-237 + embedded-pty-evaluation.md; not a blocker |

No blockers. No quality contradictions. No scope inconsistencies.

---

## Pre-Architecture-Delta Gate

The brief passes all structural and quality checks. The following are confirmed clear for the
architecture delta to proceed:

1. Vision v2.1 approved (D-238) — canonical basis confirmed.
2. v1A/v1B scope is unambiguous and human-ratified (D-237).
3. Persistence model (session-host-owns-PTY, D-238) is fully specified with the three-case
   breakdown and the Q-8 architect route called out.
4. D-235 rework consequence is documented — architect is explicitly informed not to preserve
   the in-process PTY ownership pattern.
5. All OQ/SOQ resolutions are inherited — architect does not need to re-litigate them.
6. PTY stack (portable-pty + vt100 + tui-term) is pre-committed — architect confirms the
   Cargo-init spike (`cargo tree -d` zero-duplicate check) and Q-7 vendoring posture.
7. Open architecture questions (Q-1, Q-2, Q-7, Q-8) are clearly enumerated with priority
   and routing.

Recommended next action: architect produces architecture delta spec (Q-8 HIGH + D-235 rework
+ crate layout extension + ADR for session-host mechanism), then story-writer decomposes new
capabilities into v1A/v1B stories.
