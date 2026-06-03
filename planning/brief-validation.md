---
document_type: brief-validation-report
level: ops
version: "1.0"
status: complete
producer: orchestrator (validate-brief skill)
phase: pre-phase-1-brief
timestamp: 2026-05-12T11:00:00Z
inputs:
  - specs/product-brief.md
input-hash: "f32471c"
traces_to: "factory-artifacts ee09833 (brief v1.1)"
project: monocle
verdict: NEEDS_WORK
---

# Brief Validation Report — Monocle Product Brief v1.1

## Subject

- File: `.factory/specs/product-brief.md`
- Version: 1.1
- Commit: ee09833
- Line count: 341
- Byte count: 24,981

## Overall Verdict

**NEEDS_WORK** — the brief is functionally complete and traceable but exceeds the recommended bloat threshold and contains one quality contradiction (success criterion vs. scope) that requires the JC-1 human red-line decision to resolve.

## Check Results

| Check | Status | Finding |
|-------|--------|---------|
| Structure (Required Sections) | PASS | All required sections present: What Is This?, Who Is It For?, Scope (In + Out), Success Criteria, Constraints & Integration Points, plus optional sections (Revision History, Supply Chain RUSTSEC, Open Questions, Overflow Context) |
| Quality — Specificity | PASS | No vague language; specific personas; concrete metrics |
| Quality — Measurability | PASS | 5 of 7 success criteria have hard numbers (<=6 keystrokes, <=100ms, 1000 events/sec, CI green, zero missing) |
| Quality — Scope bounds | WEAK | JC-1 contradiction unresolved: line 167 "All 7 customization types render in Static plane on filter All" is a Phase 1 success criterion, but Static plane is explicitly Phase 2 scope (lines 106-116). One of these must move — either the criterion to Phase 2 exit, or Static work back into Phase 1 in-scope |
| Quality — Audience clarity | PASS | Personas are specific enough to generate requirements: "Multi-session Claude Code developer running 2-4 sessions across worktrees" |
| Quality — Constraint actionability | PASS | Anti-patterns documented with file:line cost-of-not-knowing (e.g., shell=True silent no-op success cited from triple-confirmed gene sources) |
| Bloat — Word count | BLOATED | Core sections (excluding Overflow Context) total ~2,870 words. Skill threshold: <500 recommended, flag at >800. Brief is 3.6x the flag threshold. The Supply Chain RUSTSEC section (~350 words) and detailed Tech Stack version pin list (~800 words) are PRD/architecture-level material that belongs in architecture.md or prd.md, not the brief. |
| Bloat — Narrative padding | PASS | No business justification, market research, or competitive analysis in core sections (these are properly placed in Overflow Context) |
| Bloat — Requirements leakage | PASS | No FR-XXX numbered requirements, no architectural decisions phrased as ADRs. The brief defers to PRD/architecture for these. |
| Bloat — Token estimate | OVER | ~6,250 tokens (estimated from byte count). Skill threshold: 1,500 token max recommendation. Brief is 4.2x over. |
| Implementation Leakage — Scope/In-Scope | WARNING | Specific crates named in Phase 1 In-Scope bullets: nucleo-matcher, similar 3, tempfile::persist, axum HTTP, rmcp MCP bridge. Per skill rubric, technology names in Scope are Error-severity. However, the brief explicitly traces tech stack to approved vision D-012 ("Tech stack is fixed by vision §Tech Stack — the architect inherits these picks as Phase 1 constraints; they are not up for re-selection in Phase 1"). Downgrade severity to WARNING because leakage is INTENTIONAL and traceable. |
| Implementation Leakage — Constraints section | WARNING | The Tech Stack list in Constraints and Integration Points (~30 crate pins with versions) is explicit implementation prescription. Acceptable in Constraints per skill rubric (Warning-severity), and again traces to vision D-012. |
| Implementation Leakage — Overflow Context | PASS | Tech references in Overflow Context (Reference Gene Source Map) are descriptive references to source repos, not prescriptive new choices. Info-severity allowed. |
| Information Density | PASS | Zero instances of conversational filler; zero wordy-phrase patterns ("in order to," "due to the fact that"); zero redundant phrases; hedge words used only where precision-appropriate |
| Completeness | PASS | 341 lines far above 150 word minimum; no TBD/TODO placeholders; not a title-only stub |
| Market Intel Cross-Check | WARNING | Market intelligence assessment (Task #8 in orchestrator task list) has NOT been run. Pain claims on personas (line 54-56) are vision-derived but not market-validated. Vision approved by human (D-012) so pain claims have human-as-authority validation, but external market validation is pending. |

## Bloat Score

- **Core word count:** ~2,870 (target <500, warn >800) — OVER by 3.6x
- **Token estimate:** ~6,250 (target <=1,500) — OVER by 4.2x
- **Status:** BLOATED / OVER_SPECIFIED

## Quality Score

- 5 of 6 quality sub-checks PASS
- 1 WEAK (JC-1 scope contradiction)
- **Status:** NEEDS_WORK (until JC-1 resolved)

## Remediation Actions

The orchestrator presents these as ranked options; the human (and architect, if dispatched in parallel) should choose:

### Action A — Tighten bloat (recommended)

1. **Move Supply Chain and RUSTSEC Notes section to Overflow Context** (or extract to a separate `.factory/specs/supply-chain.md` artifact and reference from the brief in a single line). The audit cadence policy and per-crate advisory list are architect/security-reviewer concerns, not brief content.
2. **Compress the Tech Stack pin list in Constraints**. Replace the 30-row detailed table with a single sentence: "All version pins are tracked in `.factory/specs/version-manifest.md` (commit ee09833); see Supply Chain notes for advisory context. Pins are inherited from approved vision D-012 and are not up for re-selection in Phase 1." Then extract the detailed pin table to `.factory/specs/version-manifest.md`.
3. **Estimated reduction:** ~1,400 words (~3,000 tokens) removed from core; brief becomes ~1,470 core words (~3,250 tokens). Still above the 500/1,500 ideal but no longer "becoming a PRD".

### Action B — Resolve JC-1 (required regardless of bloat decision)

- **Option B1:** Move "All 7 customization types render" to a Phase 2 exit criterion in a separate Phase 2 Exit Criteria block (or in a roadmap document)
- **Option B2:** Add a Phase 1 light-static stub (the 7 parsers exist but the Customizations panel is feature-flagged off in v1; the panel ships in Phase 2)
- **Option B3:** Promote the Customizations panel into Phase 1 in-scope (expands v1 contract; may delay v1 ship)

### Action C — Accept current bloat as documented trade-off

Document explicitly in Revision History: "v1.1 intentionally exceeds the validate-brief bloat threshold to keep the version-pin manifest and RUSTSEC audit context co-located with scope for architect efficiency. The brief functions as a brief + supply-chain checklist combined. This is a deliberate trade-off; if downstream agent dispatches in Phase 1 incur context budget pressure, revisit Action A."

This option preserves single-source-of-truth at the cost of brief-purity.

## Cross-Reference Validation

- Frontmatter `traces_to: factory-artifacts 2737bfd (vision-synthesis approved); 2c2b676 (8-repo full ingest)` — both SHAs exist on factory-artifacts and are reachable from current HEAD ee09833.
- Frontmatter `inputs` lists 9 source documents; all 9 verified present on disk (semport syntheses + vision-synthesis).
- D-001 through D-013 referenced in brief — all present in STATE.md Decisions Log (D-014 and D-015 added in this commit).
- Open Questions OQ-01 through OQ-11 are internally consistent (OQ-04 references port 2748, which is also called out in JC-3 — appropriate cross-reference).
- 5 judgment calls (JC-1..3, EX-1..2) explicitly listed at the end of Open Questions with "awaiting human red-line" note. PASS.

## Pre-Phase-1 Blockers

1. **JC-1 must resolve before validate-brief PASS.** This is the only quality contradiction in the document.
2. **Bloat decision must be made.** Action A, B, or C above.
3. **Market intelligence (Task #8) should run before /vsdd-factory:phase-1-spec-crystallization** to validate the persona pain claims against external market evidence. Not a blocker for brief PASS but a blocker for confident Phase 1 entry.

## Verdict Summary

| Field | Value |
|-------|-------|
| Status | NEEDS_WORK |
| Blocker count | 1 (JC-1 contradiction) |
| Bloat decision count | 1 (Action A/B/C) |
| Sequential gate count | 1 (market intel before Phase 1 entry) |
| Leakage severity | WARNING (intentional, vision-traceable) |
| Recommended next action | Human red-lines JC-1..3 + EX-1..2 + selects bloat-remediation action → product-owner revises to v1.2 → re-run validate-brief |
