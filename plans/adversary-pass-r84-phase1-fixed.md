---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.13 dcae9d5 + VP v1.17 1d21fd0 + arch v1.0.17 a798d51 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 17 (R84); post-F-R83 fix-burst snapshot"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T22:00:00Z
pass_number: 1
attempt: 17
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
---

# Adversarial Review R84 — Phase 1 (D-047 Strict, Pass 1 Attempt 17 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

- 1 CRITICAL: F-R84-1 — Cross-layer parallel-dispatch coordination gap (~93 stale arch-pin sites across PRD v1.13 + VP v1.17 from F-R83 parallel-dispatch root cause). The F-R83 fix-burst dispatched PO + arch + FV agents in parallel. The architect agent bumped arch v1.0.16 → v1.0.17 (a798d51). The parallel PO and FV agents completed their bursts without knowing about the arch version bump. Result: PRD v1.13 and VP v1.17 cite arch v1.0.16 at all pin sites (frontmatter `traces_to`, body lineage citations, §Trace narratives) — approximately 93 stale arch-pin sites across both documents.
- 1 HIGH: F-R84-2 — PRD §7 RTM BC-ID column contains NFR-012 in a BC slot. NFR-012 is a Non-Functional Requirement identifier, not a Behavioral Contract identifier. The §7 RTM column is typed as "BC ID"; NFR identifiers violate the column schema. Root cause: F-R83 PO burst added NFR-012 as the PRD §4 NFR table row, but the §7 RTM propagation placed the NFR identifier into a BC ID column.
- 1 HIGH: F-R84-3 — §Purpose stale SHA recurrence (FOURTH recurrence). VP v1.17 §Purpose block still cites a stale SHA or stale version pin following the F-R83 fix-burst. This is the fourth recurrence of the §Purpose-class staleness pattern (prior: R13-001 → GAP-R19-001 → F-R81-2). The F-R81-2 META recurrence guard codified in D-071 and extended by F-R83 sub-extension explicitly added §Purpose to the propagation sweep target list. Yet §Purpose is stale again, demonstrating the recurrence guard's discipline was not applied in the F-R83 parallel PO + FV dispatch.
- 1 HIGH: F-R84-4 — VP v1.17 §Trace version citations stale. The §Trace section of VP v1.17 references arch at v1.0.16 in multiple entry headers and narrative prose, where v1.0.17 (a798d51) is the canonical current pin. This is a parallel consequence of the F-R83 parallel-dispatch root cause: the FV agent that authored VP v1.17 did not have the arch v1.0.17 pin available when composing §Trace entries.
- 1 MEDIUM: F-R84-5 — VP per-VP §Mechanism block version citations stale. Individual VP bodies (e.g., VP-DAEMON-005, VP-LOCK-001, VP-ENGINE-001) contain §Mechanism blocks with inline arch version citations. These cite arch v1.0.16 where v1.0.17 is canonical. Extension 14 VP-layer propagation target enumeration did NOT include per-VP §Mechanism blocks — only §Catalog Overview, §Auxiliary Mechanism Coverage, and §Coverage Matrix footer were enumerated. The per-VP §Mechanism block is a MISSING propagation target in Extension 14's VP-layer surface list.
- 1 MEDIUM: F-R84-6 — PRD v1.13 frontmatter `traces_to` field cites arch v1.0.16 (6bb93e2), not v1.0.17 (a798d51). This is the canonical frontmatter pin field that all downstream consumers use to identify the arch version this PRD was validated against. A stale frontmatter pin is a traceability defect — any agent doing a fresh-context read of PRD v1.13 will believe arch v1.0.16 is current.
- 1 LOW: F-R84-7 — Extension 14 propagation sweep evidence was enumerated without machine-greppable grep transcripts (Extension 13 violation). The F-R83 fix-burst codified Extension 14 in VP §Trace, but the codification block did not emit grep transcript evidence per Extension 13's machine-greppable evidence requirement. Extension 14's own codification body violates Extension 13. This creates a self-referential discipline gap: the extension that governs propagation sweeps does not itself demonstrate compliance with the evidence discipline.

**Observations (non-blocking but process-relevant):**

- Obs-R84-1 — Convention back-propagation discipline. When the F-R83 fix-burst established NFR-012 as the Validation Method for a specific PRD row (NFR-012 cites VP probe), the convention should back-propagate to sibling NFR rows in the same §4 NFR section. Specifically NFR-009 (related operational security NFR) lacks a VP probe citation in its Validation Method cell. If NFR-012's citation form is the correct convention, sibling rows should conform. This is a consistency observation, not a content correctness defect — but the absence creates uneven table discipline.

- Obs-R84-2 — Cross-property reciprocity. VP-DAEMON-005 §Mechanism block cites VP-LOCK-001 in a cross-property dependency note. VP-LOCK-001 §Mechanism block does NOT reciprocate the citation. When VP-A cites VP-B in cross-property dependency, VP-B must cite VP-A back to ensure bidirectional traceability. Extension 14 does not currently address this cross-VP reciprocity surface. This is an enumeration gap in Extension 14 that was not caught by the F-R83 fix-burst.

- Obs-R84-3 — Serial dispatch protocol recommendation. The F-R83 parallel-dispatch root cause (F-R84-1) reveals a systematic orchestration vulnerability: ANY fix-burst that touches multiple architectural layers simultaneously and where one layer bumps its artifact version will silently invalidate the other layers' pin citations. The current process has no guard at the orchestration level to enforce serial dispatch when version bumps are involved.

---

## F-R84-1 — CRITICAL: Cross-layer parallel-dispatch coordination gap (~93 stale arch-pin sites)

### Root cause

The F-R83 fix-burst was dispatched in 3-agent parallel: product-owner (PRD v1.13), architect (arch v1.0.17), and formal-verifier (VP v1.17). The architect agent bumped SS-daemon-lifecycle.md from v1.0.16 to v1.0.17 (commit a798d51) as part of closing F-R83-1 site 2 (§BC Summary footer postcondition count update).

The PO agent and FV agent received their dispatch prompts before the architect agent completed and landed a798d51. Both agents operated under the assumption that arch = v1.0.16 (6bb93e2). Both agents authored their documents (PRD v1.13 and VP v1.17) with the pre-bump arch pin throughout.

### Scope of staleness

PRD v1.13 (dcae9d5) and VP v1.17 (1d21fd0) cite arch v1.0.16 / 6bb93e2 at the following surfaces:

1. **Frontmatter `traces_to`** — canonical version declaration in both PRD v1.13 and VP v1.17
2. **§Trace entry headers** — each §Trace entry records the arch version at time of fix; F-R83 §Trace entries cite v1.0.16 in both documents
3. **§Trace narrative prose** — inline arch version references in closure descriptions
4. **§Purpose block** — VP v1.17 §Purpose cites the arch version pin as part of its coverage scope declaration
5. **Per-VP §Mechanism blocks** — individual VPs with arch-version-specific coverage claims (see F-R84-5)

Estimated total stale-pin count across both documents: ~93 sites.

### Disposition

Serial fix-burst required. The architect's v1.0.17 pin (a798d51) must be propagated across ALL sites in PRD v1.13 and VP v1.17. This is the remediation scope for the F-R84 serial fix-burst:

1. **PO burst (first):** PRD v1.14 — update frontmatter `traces_to` from arch v1.0.16 (6bb93e2) to arch v1.0.17 (a798d51); sweep all §Trace entries; close F-R84-2 RTM schema violation; sweep §Purpose via Extension 14 + D-071 guards.
2. **FV burst (second, after PO lands):** VP v1.18 — with confirmed PRD v1.14 pin + arch v1.0.17 pin in hand; update frontmatter `traces_to`; sweep all §Trace entries; sweep §Purpose; sweep per-VP §Mechanism blocks (F-R84-5); apply Extension 13 grep transcripts per F-R84-7.

Serial ordering is REQUIRED because the FV burst must pin to PRD v1.14 (which the PO burst produces). Parallel dispatch is PROHIBITED for this fix-burst.

---

## F-R84-2 — HIGH: §7 RTM NFR-012 in BC ID column (schema violation)

### Finding

PRD §7 Requirements Traceability Matrix contains a column typed "BC ID." The F-R83 PO burst propagated NFR-012 from §4 NFR table to §7 RTM, but placed the NFR identifier (NFR-012) in the BC ID column. NFR-012 is not a BC identifier. BC identifiers follow the pattern `BC-<MODULE>-<NNN>` (e.g., `BC-DAEMON-005`). NFR identifiers follow the pattern `NFR-<NNN>` and belong in a separate NFR column or in a dedicated NFR traceability row.

The §7 RTM column schema does not have a designated NFR column. The correct disposition is either:
- **(a)** Remove NFR-012 from the BC ID column and document the NFR-012 verification link in an NFR-specific annotation (no RTM column schema change)
- **(b)** Add a separate "NFR ID" column to §7 RTM to accommodate NFR-012 and any future NFR-to-test traceability (schema extension)

### Disposition

Route to PO burst (PRD v1.14). Disposition (a) is the production-grade choice: remove NFR-012 from BC ID column; add a footnote or annotation in the NFR-012 §4 row cross-referencing VP-DAEMON-005 probe 5.e as the verification evidence. Avoid schema extension unless PO determines disposition (b) is warranted.

---

## F-R84-3 — HIGH: §Purpose stale SHA (FOURTH recurrence)

### Finding

VP v1.17 §Purpose block cites a stale SHA / stale arch version pin. This is the fourth recurrence of the §Purpose-class staleness pattern:

- R13-001: First occurrence — VP §Purpose stale SHA
- GAP-R19-001: Second occurrence — VP §Purpose stale SHA (again, despite R13-001 guard)
- F-R81-2: Third occurrence — VP §Purpose stale SHA (again, triggering D-071 META recurrence guard codification)
- F-R84-3: Fourth occurrence — VP v1.17 §Purpose stale after F-R83 parallel-dispatch

Each prior occurrence triggered a META recurrence guard codification (D-071 §Purpose recurrence guard; F-R83 sub-extension adding §References intro timestamp). Yet §Purpose is stale AGAIN in VP v1.17 because the F-R83 parallel FV dispatch did not have the arch v1.0.17 pin when it authored VP v1.17.

### Disposition

Root cause is the same as F-R84-1: parallel-dispatch coordination gap. Fix is folded into FV burst (VP v1.18). The §Purpose block must cite arch v1.0.17 (a798d51) and PRD v1.14 (post-PO-burst SHA). FV must apply D-071 + F-R83 sub-extension sweep to all four propagation targets after composing VP v1.18.

---

## F-R84-4 — HIGH: §Trace version citations stale (arch v1.0.16 vs v1.0.17)

### Finding

VP v1.17 §Trace section contains entries for the F-R83 fix-burst that cite arch v1.0.16 (6bb93e2) in entry headers and narrative prose. The canonical arch version at the time those entries were authored is v1.0.17 (a798d51) — the FV burst was dispatched to close F-R83-1 sites 3+4, and site 2 was closed by the architect in the same parallel burst.

Because the FV agent did not know the architect had bumped to v1.0.17, the FV agent's §Trace entries record the fix against v1.0.16. Any downstream agent doing a §Trace provenance audit will see a contradiction: the §Trace entry for F-R83 records arch v1.0.16, but the artifact frontmatter and §Purpose should (after F-R84 remediation) record arch v1.0.17.

### Disposition

Folded into FV burst (VP v1.18). All F-R83 §Trace entries that cite arch v1.0.16 must be updated to cite arch v1.0.17 (a798d51) with appropriate temporal qualification per PG-3 §Trace sub-rule.

---

## F-R84-5 — MEDIUM: Per-VP §Mechanism block (Extension 14 enumeration gap)

### Finding

Extension 14 (codified in VP v1.17 via F-R83 fix-burst) enumerates the mandatory VP-layer propagation targets for `lift_invariants_to_bcs` events as:

> §Catalog Overview row + §Auxiliary Mechanism Coverage row + §Coverage Matrix footer row

This enumeration is INCOMPLETE. Individual VP bodies contain §Mechanism blocks that include inline arch version citations, cross-VP dependency citations, and coverage scope claims. These §Mechanism blocks are a FOURTH VP-layer propagation target that Extension 14 does not enumerate.

Specific evidence: VP-DAEMON-005 §Mechanism block, VP-LOCK-001 §Mechanism block, and VP-ENGINE-001 §Mechanism block all contain arch v1.0.16 citations where v1.0.17 is canonical. None of these sites are §Catalog Overview, §Auxiliary Mechanism Coverage, or §Coverage Matrix footer — they are per-VP body sections that Extension 14 failed to enumerate.

### Disposition

Folded into FV burst (VP v1.18). FV must:
1. Update all per-VP §Mechanism blocks that cite arch v1.0.16 to v1.0.17
2. Extend Extension 14 VP-layer enumeration to add: "per-VP §Mechanism block" as a FOURTH VP-layer propagation target

---

## F-R84-6 — MEDIUM: PRD v1.13 frontmatter `traces_to` stale arch pin

### Finding

PRD v1.13 (dcae9d5) frontmatter `traces_to` field reads:

> "Phase 1 PRD v1.13 ... + arch v1.0.16 6bb93e2 ..."

The canonical arch version is v1.0.17 (a798d51). This is the primary traceability declaration for the document — every fresh-context agent that reads PRD v1.13 will conclude arch v1.0.16 is the validated arch version.

This is a direct consequence of F-R84-1 (parallel-dispatch coordination gap). The PO agent that authored PRD v1.13 did not have the arch v1.0.17 pin when composing the `traces_to` field.

### Disposition

Folded into PO burst (PRD v1.14). Frontmatter `traces_to` must cite arch v1.0.17 (a798d51) once available.

---

## F-R84-7 — LOW: Extension 14 codification body violates Extension 13 (enumeration-without-evidence)

### Finding

VP v1.17 §Trace contains the Extension 14 codification block (introduced by FV in the F-R83 fix-burst). The codification narrative enumerates the 4 mandatory propagation targets for `lift_invariants_to_bcs` and describes the discipline. However, the codification block does NOT emit machine-greppable grep transcripts per Extension 13's machine-greppable evidence requirement.

Extension 13 states:
> "Every audit-row claim ... MUST be backed by: Code-block transcript of the actual grep command + actual output (file:line + matched text)"

Extension 14's codification body makes enumeration claims ("4 mandatory summary-table targets") without grep evidence demonstrating that these targets were actually verified in the F-R83 fix-burst. This is structurally the same pattern that Extension 13 was designed to prevent — the discipline's own documentation omits the evidence discipline.

### Disposition

Folded into FV burst (VP v1.18). Extension 14 codification block must be amended to include grep transcript evidence demonstrating that all 4 propagation target sites were verified in the F-R83 burst, per Extension 13 discipline.

---

## Observations

### Obs-R84-1 — Convention back-propagation (NFR table sibling rows)

When the F-R83 PO burst established NFR-012 with "Validation Method: VP-DAEMON-005 probe 5.e" in §4 NFR table, this citation form became the convention for security-critical NFRs with VP probe coverage. NFR-009 (related operational security NFR for daemon signal handling) has a Validation Method cell that does not cite the corresponding VP probe. If NFR-012's citation form is the canonical form for NFR rows with VP probe coverage, then NFR-009 and any other sibling rows with equivalent VP coverage should adopt the same form.

This observation identifies a convention back-propagation gap: a new convention established on one row in a homogeneous table should be audited against sibling rows. Extension 14 does not currently include sibling-row convention back-propagation as a check. This surface is a candidate for Sub-extension SE-15c.

### Obs-R84-2 — Cross-property VP reciprocity

VP-DAEMON-005 §Mechanism block contains a cross-property dependency citation: "cross-property dependency: VP-LOCK-001 (lock acquisition ordering)." VP-LOCK-001 §Mechanism block does not reciprocate with a citation back to VP-DAEMON-005. When VP-A normatively cites VP-B in a cross-property block, VP-B should cite VP-A back to ensure bidirectional traceability and prevent asymmetric dependency documentation.

This is a structural gap in the VP authoring discipline. Extension 14 does not enumerate cross-VP reciprocity as a propagation target. Candidate for Sub-extension SE-15d.

### Obs-R84-3 — Serial dispatch protocol recommendation

The F-R83 parallel-dispatch that caused F-R84-1 through F-R84-6 reveals a systematic orchestration vulnerability. The current orchestration process has no enforcement at the dispatch level to detect:
- Whether ANY sibling agent in a parallel burst will bump its artifact's version
- Whether a version bump in one layer will invalidate pin citations in sibling layers

This is a process discipline gap at the ORCHESTRATOR level, not a spec-content defect. The fix is Extension 15 (cross-layer parallel-dispatch coordination discipline), which the state-manager will codify in cycle-001/lessons.md. The fix-burst for F-R84 will proceed SERIAL (PO → FV) to prevent recurrence.

---

## Closure Table

| Finding | Severity | Disposition | Route | Target Artifact |
|---------|----------|-------------|-------|-----------------|
| F-R84-1 | CRITICAL | Serial cascade: arch v1.0.17 propagation sweep | PO + FV (serial) | PRD v1.14 + VP v1.18 |
| F-R84-2 | HIGH | Remove NFR-012 from BC ID column; add footnote in §4 NFR-012 row | PO | PRD v1.14 |
| F-R84-3 | HIGH | §Purpose update to arch v1.0.17 + PRD v1.14 pin; apply D-071 + F-R83 sub-ext sweep | FV | VP v1.18 |
| F-R84-4 | HIGH | §Trace F-R83 entries updated to arch v1.0.17 with PG-3 temporal qualification | FV | VP v1.18 |
| F-R84-5 | MEDIUM | Per-VP §Mechanism blocks updated; Extension 14 VP-layer enumeration extended | FV | VP v1.18 |
| F-R84-6 | MEDIUM | Frontmatter `traces_to` updated to arch v1.0.17 (a798d51) | PO | PRD v1.14 |
| F-R84-7 | LOW | Extension 14 codification body amended with Extension 13 grep transcripts | FV | VP v1.18 |
| Obs-R84-1 | OBS | Convention back-propagation sweep of NFR table sibling rows | PO | PRD v1.14 (in-scope) |
| Obs-R84-2 | OBS | VP cross-property reciprocity sweep | FV | VP v1.18 (in-scope) |
| Obs-R84-3 | OBS | Extension 15 codified by state-manager; serial protocol adopted | state-manager | lessons.md + STATE.md |

---

## Lens Rotation Log

Lens rotations applied in this pass (in dispatch order):

1. **Cross-layer pin propagation lens** — examined whether arch version pins are consistent across PRD frontmatter, VP frontmatter, §Trace entries, §Purpose blocks, per-VP §Mechanism blocks. Found F-R84-1 through F-R84-6.
2. **RTM schema correctness lens** — examined whether §7 RTM column values conform to declared column types. Found F-R84-2.
3. **§Purpose recurrence lens** — explicitly checked §Purpose block per F-R81-2 recurrence guard. Found F-R84-3 (fourth recurrence).
4. **Extension 14 completeness lens** — checked whether Extension 14 VP-layer enumeration covers all VP-body sections that contain version citations. Found F-R84-5 (per-VP §Mechanism missing).
5. **Extension 13 evidence lens** — checked whether Extension 14 codification body provides grep transcripts per Extension 13. Found F-R84-7.
6. **Sibling-row convention lens** — checked whether new conventions established on one NFR table row propagate to sibling rows. Found Obs-R84-1.
7. **Cross-VP reciprocity lens** — checked whether VP cross-property citations are bidirectional. Found Obs-R84-2.

---

## Verdict

**FINDINGS — counter stays at 0/3.** 4 HIGH/CRITICAL + 3 MEDIUM/LOW findings. Root cause of 6 of 7 findings is a single process defect: F-R83 parallel-dispatch coordination gap. Serial fix-burst protocol adopted for remediation. Extension 15 to be codified by state-manager. PO burst (PRD v1.14) dispatched first; FV burst (VP v1.18) dispatched only after PRD v1.14 lands with confirmed arch v1.0.17 pin.

Cons R23 independently confirmed F-R84-3 (§Purpose staleness) via GAP-R23-001 (commit 74f4a0e).
