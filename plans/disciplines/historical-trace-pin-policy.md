---
document_type: discipline-note
level: ops
version: "1.0"
producer: vsdd-factory:state-manager
timestamp: 2026-05-17T23:30:00Z
status: DOCUMENTATION-ONLY
codification_status: NOT-CODIFIED
codification_track: "F-R107-13 reclassification — 2nd-occurrence class (O-R107-1/O-R107-2 are 2nd; F-R107-13 is new). Escalate to codification per D-114 if R108 adversary flags this class again as a WARN or FAIL."
---

# Historical Trace Pin Policy — Discipline Note

**Status:** Documentation only. NOT a codified SE-NN discipline. Per D-114 Goodhart's law, new
disciplines require 3+ empirical recurrences before codification.

## Statement

§Trace narrative blocks in spec artifacts (BC files, VP files, architecture SS-* docs, supplements)
freeze cited artifact versions at the time the §Trace entry was authored. These version citations
are **historical context**, not stale-citation defects.

A §Trace entry that says "PRD v1.22 at commit d3df32e" in a burst narrative authored when the
current PRD was v1.22 is correct — even if the current PRD is now v1.26.5. The §Trace entry
records what was true at the time of the burst, not what is true today.

## Classification Guidance for Future Adversary Passes

When a fresh-context adversary finds version citations in §Trace narrative prose that are older than
the current canonical versions:

1. **Determine location:** Is the citation inside a `§Trace` section narrative block, or in the
   artifact body (outside §Trace)?

2. **If inside §Trace narrative block:**
   - Classify as **INFORMATIONAL** — historical context, not a defect.
   - Do NOT classify as WARN or FAIL under any finding severity class.
   - Do NOT include in finding count toward D-047 strict counter.

3. **If outside §Trace narrative block (body/frontmatter/normative sections):**
   - Classify per standard stale-citation rules (WARN, MED, or HIGH per existing discipline set).
   - These ARE stale-citation defects subject to normal remediation.

## Origin

This policy was documented in response to F-R107-13 (R107 adversary pass, 2026-05-17). The
adversary initially classified several BC §Trace historical entries as LOW findings due to stale
version pins in the narrative. The finding was reclassified as INFORMATIONAL after review:

- The cited versions were correct at the time the §Trace entries were authored.
- §Trace blocks are an append-only audit trail by design — they are NOT updated retroactively
  when an artifact they cite is subsequently bumped.
- Forcing §Trace history to track forward version bumps would require retroactively editing
  historical audit trail entries, which violates the append-only audit trail principle and
  defeats the purpose of the §Trace history.

## Boundary Conditions

| Case | Classification | Rationale |
|------|----------------|-----------|
| `§Trace` body entry: "PRD v1.22 (d3df32e) — Round 5B pin sweep" | INFORMATIONAL | Historical audit trail; correct at authoring time |
| `§References` section body: "PRD v1.22" | STALE CITATION (MED/HIGH) | Normative current-state reference; must track current |
| Frontmatter `traces_to:` field: "prd.md v1.22" | STALE CITATION (MED/HIGH) | Normative current-state reference; must track current |
| `## Source Contract` row: "PRD v1.22" | STALE CITATION (MED/HIGH) | Normative current-state reference; must track current |
| Body prose outside §Trace: "per PRD v1.22" | STALE CITATION (LOW to HIGH, context-dependent) | Normative cite; should track current |

## Relationship to Existing Disciplines

- SE-17g distinguishes NORMATIVE transcripts (literal `$ command` output) from INFORMATIONAL
  citations (labeled "informational/approximately"). This note extends that classification
  principle to §Trace historical version pins — they are analogous to INFORMATIONAL citations
  under SE-17g.

- SE-17c-d (final-state L-number revalidation) applies to NORMATIVE transcript content, not
  to §Trace narrative prose. Historical prose in §Trace is INFORMATIONAL per SE-17g; SE-17c-d
  does not require revalidation of historical-prose L-numbers.

## Codification Path

If R108 (or any subsequent adversary pass) flags §Trace historical version pins as WARN or FAIL
findings for the third time, escalate to formal codification as an SE-NN discipline via D-114
process. The codification would amend the existing SE-17g INFORMATIONAL classification rules to
explicitly enumerate §Trace historical version pins as a covered sub-case.

Until then, this document serves as the reference for adversary briefings and state-manager
record-keeping.
