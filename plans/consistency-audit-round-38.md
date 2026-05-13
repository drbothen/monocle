---
document_type: consistency-report
level: ops
version: "1.0"
producer: consistency-validator
timestamp: 2026-05-13T22:00:00Z
round: 38
traces_to: "round-37 fix burst commits 17373a3 (adv-r36 persist) + ee3f8ab (conventions v1.9) + ddc18b1 (brief v1.4.18)"
---

# Consistency Audit — Round 38 (Post Round-37 Fix Burst)

**Scope:** Final projected convergence round. Validates all 12 specified checks against the round-37 fix burst (F-R36-1 + F-R36-2 fixes).

**Gate verdict at end of report.**

---

## Summary Table

| # | Check | Result | Severity if Failed |
|---|-------|--------|--------------------|
| 1 | F-R36-1 resolution: brief line 249 cites v1.1.10 | PASS | IMPORTANT |
| 2 | F-R36-2 full propagation: delimiter sweep (spec files) | PASS | MEDIUM |
| 2a | Plans/cycle files contain delimiter strings in historical reports | OBSERVATION | INFO (in-scope per convention) |
| 3 | §Trace narrative preservation (v1.6 SS-conventions + v1.4.16/v1.4.17 brief) | PASS | — |
| 4 | STATE.md Critical Artifacts list: v1.9 conventions + v1.4.18 brief | PASS | — |
| 5 | BC count = 16; BC-ENGINE-002-ERR in all lists | PASS | — |
| 6 | Audit table integrity: 17 structs + HTML delimiters intact | PASS | — |
| 7 | Phase 1 Gate Questions: 3 items + Pending Human Direction (O-R36-1) | PASS | — |
| 8 | Vision-authority framing: consistent | PASS | — |
| 9 | CLAUDE.md staleness (Q-3): still flagged, stale text precisely captured | PASS | — |
| 10 | CLAUDE.md production-grade compliance: no rationalization phrases | PASS | — |
| 11 | Cross-reference anchors: intact | PASS | — |
| 12 | Frontmatter input-hash drift | PASS (live-state) | — |

**Overall result: CLEAN — zero blocking findings.**

---

## Detailed Findings Per Check

### Check 1: F-R36-1 Resolution — Brief v1.1.10 Citation

**Status: PASS**

`product-brief.md` line 250 (Forward-compatibility Success Criteria table) reads:
> "Per `SS-core-types-and-abi.md`, `SS-daemon-lifecycle.md` v1.0.6, and `SS-engine-module.md` v1.1.10."

The citation has been updated from v1.1.9 to v1.1.10 as required by F-R36-1. The brief frontmatter version is `1.4.18` and the v1.4.18 revision-history entry explicitly documents this fix. Confirmed.

### Check 2: F-R36-2 Full Propagation — Verbatim Delimiter Sweep

**Status: PASS (spec files)**

The convention rule (SS-conventions-anti-patterns.md §Semgrep Coverage Hardening clause 4) prohibits verbatim quoting of the audit-table delimiter strings in "§Trace prose or any spec narrative." Scope = `.factory/specs/` files.

**Spec file grep results (`<!-- BEGIN: Cross-Crate Constructor Audit Table -->` and `<!-- END: Cross-Crate Constructor Audit Table -->`):**

| File | Lines | Classification | Status |
|------|-------|----------------|--------|
| `SS-engine-module.md` | 1108 (BEGIN), 1128 (END) | Canonical delimiters — the real HTML markers wrapping the audit table | CORRECT — these are the real delimiters, not quotes |
| `SS-conventions-anti-patterns.md` | 131–132 (Python comment), 331–332 (regex constants), 365–369 (edge-case contract), 377–378 (regex constant block), 797–798 (§Trace v1.8 reference) | Normative regex constant definitions + error-message specifications + §Trace v1.8 backward reference | CORRECT — regex constant definitions are the canonical occurrence; §Trace v1.8 references the regex by name (BEGIN_DELIMITER_REGEX/END_DELIMITER_REGEX) in all but two locations (lines 797–798 which embed the full strings in the context of describing the regex specification itself) |

**Deep check on SS-conventions lines 797–798:**

Lines 797–798 read:
```
  BEGIN_DELIMITER_REGEX = r'^<!-- BEGIN: Cross-Crate Constructor Audit Table -->$'
  END_DELIMITER_REGEX = r'^<!-- END: Cross-Crate Constructor Audit Table -->$'
```

These appear in the v1.8 §Trace entry body, but they are regex constant assignment statements (code blocks), not narrative prose quoting. This is the same form as lines 331–332 and 377–378 — the convention's exception is "regex constant definitions + normative error-message specifications." These are definitional, not quotational. No violation.

**Brief sweep:** No delimiter strings found anywhere in `product-brief.md`. The v1.4.16 and v1.4.17 revision-history entries (lines 81–82) have been de-quoted. The v1.4.16 entry now refers to "HTML delimiter boundary markers (defined in SS-engine-module.md §Cross-Crate Constructor Audit Table)" and v1.4.17 refers to "the correct delimiter strings as documented in SS-conventions-anti-patterns.md §Semgrep Coverage Hardening clause 4" — both reference by name only.

**Observation 2a (INFO — not blocking):** The `.factory/plans/` and `.factory/cycles/` directories contain historical audit reports and adversary passes that include verbatim delimiter strings in context of documenting the findings themselves:
- `cycles/cycle-001/burst-log.md` line 1089, 1129
- `adversary-pass-round-34.md` lines 26, 41
- `consistency-audit-round-34.md` lines 41, 76–77, 80–81
- `consistency-audit-round-32.md` lines 45, 76–77, 80, 94–95
- `consistency-audit-round-36.md` lines 57, 82, 85, 94, 112, 192–193

These are historical evidence artifacts in `.factory/plans/` and `.factory/cycles/`, not spec narrative files. The convention rule's stated scope is "§Trace prose or any spec narrative." Historical round reports are not spec narrative. No remediation required.

The line-anchored regex (`^<!-- BEGIN: Cross-Crate Constructor Audit Table -->$`) correctly excludes these occurrences because all instances in the plan/cycle files are mid-line (backtick-wrapped, indented, or table-cell-embedded) — confirmed by the round-36 audit which ran this exact validation. The CI Python script will not false-positive on them.

### Check 3: §Trace Narrative Preservation

**Status: PASS**

**SS-conventions v1.6 §Trace (rewritten by v1.9 fix):**

The v1.6 §Trace point (1) now reads: "SS-engine-module.md v1.1.9: HTML BEGIN/END delimiter markers (whose canonical line-anchored regex patterns are specified in BEGIN_DELIMITER_REGEX and END_DELIMITER_REGEX in clause 4 of §Semgrep Coverage Hardening) wrap the audit table rows, enabling a CI Python script to machine-parse the declared struct list."

Historical meaning preserved: a reader understands that (1) the HTML delimiter markers exist in SS-engine-module.md, (2) their canonical form is defined in SS-conventions §Semgrep Coverage Hardening, (3) the purpose is CI machine-parsing. The narrative is unambiguous. The de-quoting removed only the verbatim HTML comment text, not any semantic content.

**Brief v1.4.16/v1.4.17 entries (rewritten by v1.4.18):**

The v1.4.16 entry refers to "HTML delimiter boundary markers (defined in SS-engine-module.md §Cross-Crate Constructor Audit Table)." The v1.4.17 entry refers to "the correct delimiter strings as documented in SS-conventions-anti-patterns.md §Semgrep Coverage Hardening clause 4." A Phase 1 implementer reading these entries can still understand: v1.4.16 introduced the HTML-delimited audit table, v1.4.17 corrected the wrong marker names that v1.4.16 quoted. Historical causality chain intact. No semantic loss.

The v1.4.18 entry itself explicitly documents the de-quoting rationale and confirms "The historical narrative is fully preserved." Correct.

### Check 4: STATE.md Critical Artifacts Version Pointers

**Status: PASS**

STATE.md Critical Artifacts (lines 124–135) lists:
- Item 3: `.factory/specs/product-brief.md` **v1.4.18** (commit ddc18b1) ✓
- Item 9: `.factory/specs/architecture/SS-conventions-anti-patterns.md` **v1.9** ✓

Both match the round-37 fix burst outputs. The `traces_to` frontmatter field (line 14) also references both commits: `ee3f8ab (conventions v1.9) + ddc18b1 (brief v1.4.18)`. Confirmed.

### Check 5: BC Count = 16; BC-ENGINE-002-ERR Enumeration

**Status: PASS**

**Brief line 250:** "16 behavioral contracts pre-staged for Phase 1 PRD: BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-RING-001, BC-AUTH-001/002, BC-ENGINE-001/002/002-ERR/003, BC-LOCK-001."

Count verification: BC-ABI-001, BC-ABI-002 (2), BC-TYPES-001 (1), BC-FACTORY-001, BC-FACTORY-002 (2), BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 (3), BC-RING-001 (1), BC-AUTH-001, BC-AUTH-002 (2), BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003 (4), BC-LOCK-001 (1) = **16 total**. ✓

BC-ENGINE-002-ERR appears: in the brief enumeration (line 250), in the SS-engine-module §Behavioral Contracts (confirmed at line 970+), in SS-engine-module Pre-Staging table (confirmed line 1154 with full test spec), and in SS-forward-compatibility.md (confirmed per prior rounds). All consistent.

### Check 6: Audit Table Integrity — 17 Structs + HTML Delimiters

**Status: PASS**

Audit table row count (awk extraction between delimiters):
- 1 header row (`| Struct | Defining crate | ...`)
- 1 separator row (`|--------|...|`)
- 17 data rows (EngineMetadata, ProcessSnapshot, EnrichedSession, HookResponse, SpawnArgs, SessionHandle, EngineVersion, HookEventRecord, SessionStartEvent, UserPromptSubmitEvent, PreToolUseEvent, NotificationEvent, StopEvent, FactoryDetection, FactoryState, BlockingIssue, ConvergenceMetrics)

Total pipe rows between delimiters: 19. Data rows: 17. Matches claimed count. ✓

HTML delimiters at their canonical positions:
- `<!-- BEGIN: Cross-Crate Constructor Audit Table -->` at line 1108 ✓
- `<!-- END: Cross-Crate Constructor Audit Table -->` at line 1128 ✓

### Check 7: Phase 1 Gate Questions + O-R36-1 Pending Human Direction

**Status: PASS**

STATE.md §Phase 1 Gate Questions (lines 154–162) contains exactly 3 items:
1. Vision-vs-architecture authority (D-031) — routing precedent human ratification ✓
2. Architect-brief-routing precedent (D-032) — mechanical propagation exemption decision ✓
3. CLAUDE.md operational pointer refresh (F-R26-1) — Brief v1.4.2 / vision v1.1.1 stale ✓

STATE.md §Pending Human Direction (lines 164–174) contains O-R36-1 with the 3 options (a codify as Phase 1 story, b tech-debt-register, c accept manually). ✓

The Session Resume Checkpoint (line 122) correctly states: "If CLEAN: present Phase 1 gate to human with 3 standing questions (D-031, D-032, Q-3) + Pending Human Direction (O-R36-1)." ✓

### Check 8: Vision-Authority Framing

**Status: PASS**

The brief correctly frames the vision as non-authoritative for endpoint enumeration (line 125):
> "Note: The vision document's §Process Topology diagram pre-dates JC-2 / EX-2 endpoint closures and depicts an illustrative endpoint set (with PostToolUse / PermissionPrompt); the canonical Phase 1 endpoint set is the 5 endpoints listed above and the vision diagram is non-authoritative for endpoint enumeration."

CLAUDE.md §Architectural Authority rule: "later/more-specific artifact wins." The SS-engine-module.md and SS-core-types-and-abi.md are later and more specific than the vision. The framing is correct and consistent. The vision document itself (v1.1.2) is still cited as the approved human intent artifact. No inconsistency.

### Check 9: CLAUDE.md Staleness (Q-3) — Still Flagged, Text Precisely Captured

**Status: PASS**

CLAUDE.md §Current Pipeline State (lines 22–24) reads:
```
- Brief: `v1.4.2` at `.factory/specs/product-brief.md`, `validate-brief` verdict: v5 VALID.
- Phase: `pre-phase-1-final-gate-post-fix-burst` (round-4 consistency audit complete; adversary fresh pass pending).
```

And §Architectural Authority (line 47):
```
6. `.factory/specs/product-brief.md` v1.4.2 — Phase 1-4 scope, success criteria...
7. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.1 — re-approved 2026-05-12...
```

Actual current versions: brief = v1.4.18 (stale by 16 minor versions), vision = v1.1.2 (stale by one patch). The staleness is precisely captured in Q-3 of STATE.md and is correctly flagged as requiring human-directed refresh before Phase 1 entry approval. AI agents do not edit CLAUDE.md. Routing: human. The observation is accurately tracked.

### Check 10: CLAUDE.md Production-Grade Compliance — No Rationalization Phrases

**Status: PASS**

CLAUDE.md contains no rationalization phrases ("for now," "good enough," "we can fix later," "minimum viable," "ship fast and iterate") in its own text. The canonical principle explicitly defines these as defect-pattern smells. The CLAUDE.md text is structurally and semantically compliant with the principle it defines. No issues.

### Check 11: Cross-Reference Anchors

**Status: PASS**

All cross-file references verified consistent:
- Brief cites SS-daemon-lifecycle.md v1.0.6, SS-engine-module.md v1.1.10 — both match actual frontmatter versions ✓
- SS-conventions `traces_to` frontmatter references are live-state (not version-pinned) ✓
- STATE.md Critical Artifacts list paths all match actual file locations ✓
- BC-ENGINE-002-ERR cross-references (brief ↔ SS-engine-module ↔ SS-forward-compatibility) consistent across rounds ✓

### Check 12: Frontmatter Input-Hash Drift

**Status: PASS (live-state)**

Both `product-brief.md` and `SS-conventions-anti-patterns.md` carry `input-hash: "[live-state]"`. These are actively edited spec files during the pre-phase-1 gate; live-state designation is correct and expected per the hook lesson documented in STATE.md line 143. No drift condition applies.

---

## Final Verbatim Delimiter Sweep — Full Results

The full grep across all `.factory/` files:

**Files INSIDE `.factory/specs/` (convention scope):**

| File | Match type | Location | Assessment |
|------|-----------|----------|------------|
| `SS-engine-module.md` | Real BEGIN delimiter | Line 1108 | CANONICAL — the actual HTML marker |
| `SS-engine-module.md` | Real END delimiter | Line 1128 | CANONICAL — the actual HTML marker |
| `SS-conventions-anti-patterns.md` | Python comment (prose) | Lines 131–132 | DEFINITIONAL — regex constant reference in comment |
| `SS-conventions-anti-patterns.md` | Regex constant definition | Lines 331–332 | DEFINITIONAL — normative regex constant assignment |
| `SS-conventions-anti-patterns.md` | Edge-case contract prose | Lines 365–366, 368–369 | NORMATIVE — contract specifying script behavior |
| `SS-conventions-anti-patterns.md` | Regex constant definition | Lines 377–378 | DEFINITIONAL — normative regex constant assignment |
| `SS-conventions-anti-patterns.md` | §Trace v1.8 body | Lines 797–798 | DEFINITIONAL — regex constant definition in §Trace |

All occurrences in `.factory/specs/` are either: (a) the real canonical delimiters in SS-engine-module.md, or (b) normative definitional regex constants in SS-conventions-anti-patterns.md. Zero violations.

**Files OUTSIDE `.factory/specs/` (out of convention scope):**

Historical plan/cycle reports contain verbatim delimiters in documenting prior findings. These are out of the convention scope and excluded from CI line-anchored regex detection. No action required.

**Zero spec-narrative violations found. The convention is clean.**

---

## Verdict

**GATE: PASS — CLEAN**

Zero findings of any severity in spec files. All 12 checks pass.

---

## Phase 1 Gate Status

The project is ready to present the Phase 1 gate to the human. The following items require human decision before Phase 1 entry:

**Gate Questions (must answer before Phase 1):**
1. Vision-vs-architecture authority framing (D-031): ratify explicitly?
2. Architect-brief-routing precedent (D-032): narrow exemption or strict owner routing?
3. CLAUDE.md operational pointer refresh (Q-3): human updates Brief v1.4.2→v1.4.18 and vision v1.1.1→v1.1.2.

**Pending Human Direction (must select option before Phase 1):**
- O-R36-1: Cross-artifact citation staleness prevention — option (a) Phase 1 story, (b) tech-debt-register, or (c) accept manual process.

**FINAL VERDICT: PROJECT READY FOR PHASE 1 ENTRY — YES, modulo the 3 standing gate questions + O-R36-1 codification decision (all human-directed).**

No AI-fixable blocking items remain. Spec package is internally consistent and production-grade. Adversary fresh pass (round-38 adversary thread) is the parallel companion to this consistency validation.
