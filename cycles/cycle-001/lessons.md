---
document_type: lessons-learned
level: ops
project: monocle
cycle: cycle-001
version: "1.0"
producer: state-manager
timestamp: 2026-05-14T06:00:00Z
input-hash: "[live-state]"
---

# Lessons Learned — Cycle 001

Per S-7.02 cycle-closing checklist. [codified] entries indicate defense layers that were
formally written into spec artifacts (SS-conventions-anti-patterns.md or similar).

---

## R47 Codifications

### [codified] PG-1 — §Schema-Fact Citation Convention (R47)

**Pattern:** Schema-fact claims (e.g., "session_id present in all 5 monocle-canonical schemas")
must cite the source document and version that was actually inspected. Claims without
attestation become stale silently as schemas evolve.

**Codified in:** SS-conventions-anti-patterns.md §Schema-Fact Citation Convention (R47 fix burst,
commit 1cbab1e).

**Trigger:** F-R46-1 HIGH — DTU schema-citation drift across dtu-assessment.md, SS-core-types-and-abi.md,
SS-forward-compatibility.md. Root cause: schema claims made without explicit source citation.

---

### [codified] PG-2 — META Rule: Count-Verification Sweep (R47; generalized R49)

**Pattern (R47 form):** Numbered/counted lists in spec prose must be verified against actual
element counts whenever the spec is modified. Any change that adds/removes sections, rules,
or steps must trigger a count-verification grep.

**Evolved to noun-agnostic form (R49):** The META grep recipe must match any ordinal or
word-count expression (seven, five, three, etc.) near any structural element, regardless of
the specific noun used ("mechanisms", "steps", "rules", "subsections", etc.). Enumerated-noun
forms escape when new counting nouns are introduced.

**Codified in:** SS-conventions-anti-patterns.md §Count-Verification META Rule (R47 initial,
generalized in R49 fix burst commit 07c1259).

**Trigger:** F-R46-2/3 in R46; F-R48-adv-1 refined the generalization scope.

---

### [codified] PG-3 — §Cross-Section Directional Reference Convention (R47.2; expanded R49)

**Pattern (R47.2 form):** Within §Trace prose, cross-document references must use
position-free section names (e.g., `§EngineModule`) rather than L-number pinpoints
(e.g., "line 654"). Historical L-numbers in §Trace entries are exempt (they record
what existed when the fix was made).

**Expanded scope (R49):** Scope expanded from §Trace-prose to ALL spec prose in any section.
No cross-doc L-number pinpoints permitted in main-body prose of any section; position-free
section references required everywhere.

**Codified in:** SS-conventions-anti-patterns.md §Cross-Section Directional Reference Convention
(R47.2 commit 42b0007; §Trace sub-rule R47.3 commit 83cd93f; all-prose expansion R49
commit 07c1259).

**Trigger:** F-R48-adv-2 revealed §Trace-only scope was insufficient; main-body prose at
4 sites in SS-engine-module.md + SS-core-types-and-abi.md also used L-numbers.

---

### [codified] PG-3 §Trace-prose Sub-Rule (R47.3)

**Pattern:** §Trace entries specifically: when writing a new §Trace entry that refers to prior
versions, use "at time of this fix" + "subsequently bumped" qualifiers to distinguish historical
pinpoints from current-state claims. This prevents M-TRACE-HISTORICAL-VS-CURRENT false positives.

**Codified in:** SS-conventions-anti-patterns.md §Trace sub-rule (commit 83cd93f).

**Trigger:** R47.3 §Trace L-pinpoint sweep revealed the need for explicit temporal qualification
language in §Trace entries.

---

## R49 Codifications

### [codified] PG-2 Generalized to Noun-Agnostic Syntactic Shape (R49)

**Pattern:** See PG-2 entry above (R47 initial + R49 generalization merged). The R49
generalization changed the grep recipe from noun-enumerated to syntactic-shape — any
ordinal/count-word in proximity to a countable structural element triggers verification.

**Codified in:** SS-conventions-anti-patterns.md §Count-Verification META Rule (commit 07c1259).

**Trigger:** F-R48-adv-1 LOW — PG-2 used "mechanisms" (not in enumerated noun list) at L51.
Seven subsections were actually present and the count was correct, but the PG-2 grep would
have missed it if the count had drifted.

---

### [codified] PG-3 Expanded to All-Prose Scope (R49)

**Pattern:** See PG-3 entry above. The R49 expansion changed scope from §Trace-prose to
all spec prose. Already covered in the PG-3 merged entry above.

**Codified in:** SS-conventions-anti-patterns.md §Cross-Section Directional Reference Convention
(commit 07c1259, all-prose scope statement).

**Trigger:** F-R48-adv-2 LOW.

---

### [codified] PG-D042-BURST-SKIP Closure — D-042 Scope Codified as .factory/specs/ Recursive (R49)

**Pattern:** D-042 grep-before-version-bump rule must use `.factory/specs/` recursive scope
(not a narrower pattern). A prior burst skipped the D-042 scope verification (PG-D042-BURST-SKIP
process gap) because the scope was not formally codified. R49 fix burst closed this by explicitly
writing `.factory/specs/` as the canonical grep scope in the D-042 convention text.

**Codified in:** SS-conventions-anti-patterns.md §D-042 Scope Codification (commit 07c1259).

**Trigger:** F-R48-adv-3 LOW and the PG-D042-BURST-SKIP finding from earlier rounds. The scope
hole allowed burst-skip gaps where the recursive grep was not applied.

---

## Convergence Pattern Lessons

### Trajectory decay is meaningful signal

Trajectory: R44 4f, R46 3f (1H+1M+1L), R48 3f LOW only, R50 ZERO. The severity decay
from HIGH/MEDIUM to LOW-only in R48, and then to zero in R50, is the expected pattern when
defense layers are closing root-cause coverage. Novelty assessment at R50: ZERO. This is
the correct convergence signal — not just count reduction but novelty exhaustion.

### Defense layer verification requires inter-layer compatibility check

When codifying a new defense layer, explicitly check it against ALL prior layers, not just
the gap it closes. The R47 PG-2 enumerated-noun form was correct for the R46 finding but
incompatible with the next novelty probe (F-R48-adv-1). Inter-layer compatibility check
should be added to the PG codification checklist.

### [process-gap] tagging enables root-cause closing

All R48 findings were tagged [process-gap] because they were meta-layer completeness gaps
in the defense codification, not new spec content errors. This tagging allowed R49 to address
root causes (PG-2 generalization, PG-3 expansion, D-042 scope codification) rather than
surface fixes. The [process-gap] tag correctly predicted that root-cause closure would
yield a ZERO-finding R50.

---

## R51 Codifications

### [codified] PG-4 — §Section-Anchor Citation Convention (R51.1)

**Pattern:** Cross-document §<Name> references MUST point to an actual `#/##/###/####` heading
in the target file. Inline prose mentions, bold-label paragraphs (**Name:**), or paragraph
prefixes do not satisfy the §-anchor convention. A reader must be able to navigate to the
cited section by heading search alone.

**Initial recipe (SS-only, later expanded by D-052):** `grep -rn "§" .factory/specs/architecture/SS-*.md`

**Anti-pattern:** `§Option A` citing SS-permissions-phase1.md when the actual heading is `§Trace`.

**Codified in:** SS-conventions-anti-patterns.md §Section-Anchor Citation Convention (R51.1
fix burst, commit 562b54c).

**Trigger:** F-R51-adv-1 MEDIUM — gene-source qualifier in SS-engine-module.md L655 cited
`§Option A` in SS-permissions-phase1.md but no such heading exists; attestation is under §Trace.
The mis-anchor was introduced by the F-R48-adv-3 fix in R49.

---

## R52 Codifications

### [codified] PG-3-TRACE-NEW-ENTRY META-rule Reflexivity Discipline (R52.1)

**Pattern:** META rules apply to their own application-documentation (§Trace entries that
describe the rule's fix), not just to the artifacts the rule governs. When authoring a §Trace
entry that documents a PG-3 fix, the entry itself must satisfy PG-3. A post-write self-audit
grep is required after every new §Trace entry is authored.

**Anti-pattern (F-R52-cons-1):** SS-conventions §Trace v1.19 entry documenting the PG-4
§-heading-existence sweep contained a bare `L487` token — a PG-3 violation in the very
prose documenting a PG-3-related fix. Ironic self-violation pattern.

**Codified in:** SS-conventions-anti-patterns.md §PG-3-TRACE-NEW-ENTRY Reflexivity Discipline
(R52.1 fix burst, commit fa3051d).

**Trigger:** F-R52-cons-1 LOW — PG-3 violation in §Trace entry that documents the R51.1
PG-4 sweep.

---

### [codified] PG-D042-DTU-SCOPE — D-042 Sibling Patterns for Non-SS Spec Artifacts (R52.2)

**Pattern:** The D-042 grep-before-version-bump rule must include sibling grep patterns for
all non-SS-prefixed versioned spec artifacts, specifically:
- `grep -rn "dtu-assessment\.md v" .factory/specs/` (dtu-assessment.md citations)
- `grep -rn "domain-monocle-vision[^ ]*\.md v" .factory/specs/` (vision doc citations)

product-brief.md is excluded from the automated recipe per D-041 routing (brief updates
are product-owner-driven, not automated grep-sweep-driven).

**Codified in:** SS-conventions-anti-patterns.md §D-042 CANONICAL SCOPE (R52.2 fix burst,
commit c20ff19).

**Trigger:** F-R52R-1 LOW — D-042 incomplete cascade: SS-forward-compat §P2-1 3 sites cited
dtu-assessment.md v1.4 but dtu-assessment.md was bumped to v1.5 in the same R52.1 burst.
The primary D-042 recipe (SS-*.md scope) did not catch dtu-assessment.md citations.

---

## R53 Codifications

### [codified] PG-RECIPE-SCOPE — META-META Rule: Every New META-Rule Recipe Must Include Sibling Patterns (R53.1)

**Pattern [META-META level]:** Every newly codified META-rule's sweep recipe MUST include
sibling grep patterns for ALL versioned spec artifact classes (SS-*.md, brief, dtu-assessment.md,
vision, ADR-*.md) AT CODIFICATION TIME — not as a follow-up burst triggered by the next
adversary pass. An SS-only scoped recipe is structurally incomplete at definition time.

**Root cause closed:** This is the 9th recurrence of the SS-only scope-hole pattern.
The pattern: a new META-rule is codified with SS-only grep scope → sibling spec artifacts
silently escape the recipe → the next adversary pass finds violations in sibling artifacts →
the recipe is expanded in a follow-up burst. PG-RECIPE-SCOPE closes this class at the
META-META level, making it structurally impossible for a new META-rule to silently inherit
the SS-only scope.

**Anti-pattern (F-R53-adv-2):** PG-4 recipe (codified R51.1) used `grep -rn "§" .factory/specs/architecture/SS-*.md`
silently excluding product-brief.md, dtu-assessment.md, vision, and ADR-*.md — the same
root-cause class as pre-PG-D042-DTU-SCOPE gap in D-042.

**Codified in:** SS-conventions-anti-patterns.md §META-Rule Recipe Sibling-Pattern Convention
(R53.1 fix burst, commit 8baec19). Also expanded PG-4 recipe per same principle (4 sibling
patterns added, Patterns 2-5).

**Trigger:** F-R53-adv-2 MEDIUM [process-gap] — PG-4 recipe was SS-only; expanded-scope probe
found mis-anchors in brief and ADR files invisible to the original recipe.

---

### [codified] PG-4 Recipe Expansion — 5-Pattern Recipe Including Sibling Artifacts (R53.1)

**Pattern:** PG-4 §-heading-existence sweep now uses a 5-pattern recipe (expanded from 1 per
PG-RECIPE-SCOPE META-META rule):
1. `grep -rn "§" .factory/specs/architecture/SS-*.md` — SS-prefixed architecture specs
2. `grep -n "§" .factory/specs/product-brief.md` — product brief
3. `grep -n "§" .factory/specs/dtu-assessment.md` — DTU assessment
4. `grep -rn "§" .factory/specs/research/` — vision and research docs
5. `grep -rn "§" .factory/specs/architecture/adr/` — ADR files

**Codified in:** SS-conventions-anti-patterns.md §Section-Anchor Citation Convention (R53.1
fix burst, commit 8baec19, recipe expansion appended to existing PG-4 section).

**Trigger:** PG-RECIPE-SCOPE META-META rule; PG-4 original recipe was Pattern 1 only.

---

## Convergence Pattern Lessons — R51-R53 Cycle

### Asymptotic META-pattern recursion at META-META level

The R51-R53 cycle confirmed asymptotic META-pattern recursion: each META-rule codification
(PG-4 in R51.1) introduced a recipe that was itself incomplete (SS-only scope), which was
caught by the next adversary cycle and required a recipe-expansion burst. The cycle then
repeated at R52.2 (PG-D042-DTU-SCOPE) and again at R53 (PG-RECIPE-SCOPE).

PG-RECIPE-SCOPE addresses this at the META-META level. No higher-order rule is needed:
the pattern is now structural — every new META-rule must include sibling patterns at
codification time. The recursion class is closed.

### Convergence under D-047 strict policy after defense-layer explosion

R50 achieved the first clean pass (both legs CLEAN). R51-R53 each found new META-pattern
class instances despite extensive prior codification. This validates the asymptotic
characterization: the defense layers are now comprehensive (12 codified), but each new
codification round's §Trace entries and recipe definitions are themselves subject to the
existing META-rules.

The trajectory is: R50 ZERO → R51 1 MED → R52 2 LOW → R53 5 findings (META-META level).
This is not divergence — it is convergence at a higher level of abstraction. PG-RECIPE-SCOPE
closes the recursion class. R54 should be the first audit to benefit from the full 12-layer
defense stack including the META-META rule.

---

## R54 Codifications

### [codified] PG-D042-WITHIN-FILE — D-042 Each-File Completeness Audit (R54.1)

**Pattern:** When a sibling-file bump triggers a D-042 cascade, the architect MUST run
a file-level grep within each citing file to verify ALL occurrences of the cited
document's version string are handled (not just previously-known ones). Selective
within-file updates are FORBIDDEN. Historical pinpoints in §Trace or Disposition columns
must be explicitly preserved and documented in §Trace.

**Anti-pattern (F-R54-adv-1 / F-R54-1):** SS-forward-compatibility.md §Verdict at line 218
was updated to SS-daemon-lifecycle.md v1.0.7 in R53.1, but FC table cells at lines 198/203
(same file, same document, 20 lines above) were not updated in the same burst. Both the
consistency leg and adversary leg independently found this as the same D-042 cascade
within-file partial miss.

**Codified in:** SS-conventions-anti-patterns.md §PG-D042-WITHIN-FILE sub-rule (R54.1 fix
burst, commit ee1fa67). This is the 10th D-042 recurrence pattern and the first
within-file-partial instance.

**Retroactive audit:** R51-R53 cascades re-audited against PG-D042-WITHIN-FILE criterion:
0 additional partial-cascade misses found. R54.1 within-file sweep: CLEAN.

---

### [codified] PG-4 Scope Clause — Aligning with PG-RECIPE-SCOPE (R54.1)

**Pattern:** META-rule rule statements must have explicit Scope clauses that align with
PG-RECIPE-SCOPE (versioned spec artifacts in .factory/specs/). Without an explicit Scope
clause, an auditor reading a META-rule cold cannot determine which document classes are
governed. CLAUDE.md enumerated-item citations are exempt from PG-4 enforcement.

**Anti-pattern (F-R54-adv-2):** PG-4 §Section-Anchor Citation Convention had no Scope
clause after initial codification (R51.1) and recipe expansion (R53.1). The absence
created potential for over-broad application to non-versioned documents.

**Codified in:** SS-conventions-anti-patterns.md §Section-Anchor Citation Convention —
Scope clause added immediately after rule paragraph (R54.1 fix burst, commit ee1fa67).

---

## R54-R55 Convergence Pattern Lessons

### Both-legs independent finding validation

R54 consistency leg and adversary leg independently surfaced the same FC table cell
staleness (F-R54-1 / F-R54-adv-1). This is expected behavior — it validates that
both auditors are reading the same corpus and the finding is genuine. The adversary's
escalation to MEDIUM (from consistency's LOW) via the within-file scope hole framing
is also expected — the adversary's role is to identify the root cause and codification
gap, not just the surface symptom.

### Scope-clause boundary recursion is a known stable class

F-R55-adv-1 and F-R55-adv-3 both exploit the scope clause added to PG-4 in R54.1.
The recursion pattern: a new rule or scope boundary introduces its own ambiguities,
which the next adversary pass exploits. This is the same class as:
R51.1 → R53 (PG-4 SS-only scope), R52.2 → R54 (PG-D042-WITHIN-FILE gap), R53.1 → R54
(PG-4 scope-clause alignment), R54.1 → R55 (em-dash separator + intra-doc scope hole).

PG-RECIPE-SCOPE closes cross-file scope recursion. D-053 option (b) closes the policy
problem: bounded LOW META residuals are acceptable; the set must not grow.

---

## R55 Bounded Residuals (per D-053)

### [bounded-residual] F-R55-adv-1 — PG-4 Em-Dash Separator Codification Gap

**Pattern:** PG-4 does not address the em-dash separator form `§HeadingName — Qualifier`
(16 sites in corpus) vs the paren form `§HeadingName (Qualifier)` shown in examples.
Neither form is explicitly authorized or prohibited. Auditors must currently decide
ad hoc whether to strip the em-dash suffix before verifying heading existence.

**Status:** BOUNDED RESIDUAL per D-053 option (b). Deferred to Phase 1 burndown OR
ratification as alternate separator form. Must not recur as a NEW finding class in
pre-Phase-1 audits; if re-flagged from this catalog entry, that is expected and allowed.

**Sites:** SS-engine-module.md, SS-daemon-lifecycle.md, SS-forward-compatibility.md
(§Trace entries and FC table column 4).

---

### [bounded-residual] F-R55-adv-3 — PG-4 Intra-Document Scope Hole

**Pattern:** PG-4 scope clause (added R54.1) scopes the rule to "cross-document"
citations. This inadvertently creates an intra-document scope hole: §-citations within
the same file to bold-paragraph labels (not headings) escape PG-4 enforcement entirely.
Sites in SS-conventions v1.23 §PG-D042-WITHIN-FILE may cite numbered-step bold labels.

**Status:** BOUNDED RESIDUAL per D-053 option (b). Deferred to Phase 1 burndown OR
PG-4 scope extension (add intra-document coverage with appropriate bold-label exemptions).
Must not recur as a NEW finding class in pre-Phase-1 audits.

---

## [policy] D-053 — Convergence Relaxation to Option (b) Phase-Scoped (2026-05-14)

**Decision:** Human (Josh Magady) ratified option (b) for pre-Phase-1 phase only.

**Criterion (pre-Phase-1):** 3 consecutive passes with:
- 0 CRIT/HIGH (hard block)
- 0 MED content-affecting findings (hard block)
- LOW META-rule-codification gaps: allowed IF within the frozen bounded residual catalog

**Scope:** Pre-Phase-1 final gate only. Subsequent phases (Phase 1+) revert to D-047
strict 3-clean-pass (0 findings of any severity).

**Trigger:** R55-gate commitment — adversary passed NEEDS_ONE_MORE with F-R55-adv-1/3
being scope-clause exploits of R54.1 rules, continuing the META-recursion class that
PG-RECIPE-SCOPE was expected to close but could not close at the within-rule boundary
level. Orchestrator surfaced O-R55-gate to human; human ratified option (b).

**Bounded residual catalog (frozen during pre-Phase-1):** F-R55-adv-1, F-R55-adv-3.
If R56+ surfaces a NEW META pattern not in this catalog, that is NEEDS_ONE_MORE.

---

## R56 Codifications

### [codified] PG-5 — §Historical-Anchor Framing Convention (R56.1, commit e5a5b5a)

**Pattern:** Version citations to external documents (brief, vision, ADRs) in body prose must
use historical-anchor Form 2 ("at time of authoring") rather than bare unqualified version
references. Bare references appear to claim the cited version is current truth.

**Codified in:** SS-conventions-anti-patterns.md §Historical-Anchor Framing Convention with
full scope clause (SS-* ×7, dtu-assessment, ADR-N ×4, vision, brief) and per-class sweep
recipe (commit e5a5b5a).

**Trigger:** F-R56-1/2 — SS-deps-pin-manifest, SS-permissions-phase1, ADR-0001, ADR-0004 body
citations without historical-anchor qualifier; R55.1 scoped fix to SS-forward-compatibility.md
only, leaving other artifact classes un-swept.

---

## R57 Codifications

### [codified] PG-5 Sweep-Evidence Checklist Requirement (R57.1, commit 9cc8205)

**Pattern:** Any §Trace entry applying PG-5 must include per-class enumerated sweep evidence
(SS-*: N files swept, N violations, N fixed; brief: N; dtu-assessment: N; vision: N; ADR-N: N).
Without enumerated evidence, auditors cannot verify PG-5 coverage from the §Trace audit trail.

**Codified in:** SS-conventions-anti-patterns.md §Historical-Anchor Framing Convention §Trace
entry requirements (commit 9cc8205).

**Trigger:** F-R57-1 — R56.1 §Trace entry for SS-conventions v1.24 missing per-class counts.
S-7.01 partial-fix irony: PG-5 codification burst omitted the sweep-evidence requirement from
its own §Trace entry.

---

### [codified] PG-5 Option B — Frontmatter Carve-Out (R57.1, commit 9cc8205)

**Pattern:** PG-5 scope clause covers body prose of versioned spec artifacts. Frontmatter fields
(`traces_to`, `commit`, `timestamp`, etc.) are operational metadata and explicitly exempt.

**Codified in:** SS-conventions-anti-patterns.md §Historical-Anchor Framing Convention scope
clause (commit 9cc8205).

**Trigger:** F-R57-2 — PG-5 scope clause as codified in R56.1 did not address frontmatter fields.

---

## R58 Codifications

### [codified] PG-3-TRACE-NEW-ENTRY Enhanced Self-Audit (R58.1, commit d00c67f)

**Pattern:** Before committing any §Trace entry, run `grep -nE 'L[0-9]+'` on the full new
§Trace entry block (not just a pattern subset). The narrow grep recipe
`\(L[0-9]+\)|paragraph at L[0-9]+|this file L[0-9]+|L[0-9]+-L[0-9]+` does not catch
space-delimited tokens ("L1408 + L1483") or bare L-number shorthands in summary lines.

**Codified in:** SS-conventions-anti-patterns.md §PG-3-TRACE-NEW-ENTRY enhanced self-audit
step (commit d00c67f).

**Trigger:** F-R58-1 — SS-permissions-phase1.md v1.2 §Trace used "§Context L28" / "§Consequences
L271" — bare L-numbers without version prefix. S-7.01 partial-fix irony: R57.1 applied PG-5
to the file and wrote a §Trace entry violating PG-3.

---

### [codified] §Trace-Heading-Convention (R58.1, commit d00c67f)

**Pattern:** All versioned spec artifacts (SS-*.md, dtu-assessment.md) must use `## §Trace` as
their versioned change-log section heading. Files without this heading must be explicitly
documented as intentionally having no §Trace equivalent.

**Codified in:** SS-conventions-anti-patterns.md §Trace-Heading-Convention with corpus audit
checklist (commit d00c67f).

**Trigger:** F-R58-adv (architecture-consistency gap — inconsistent heading forms across specs).

---

## R59 Codifications

### [codified] §Trace-Heading-Convention — Heading-Agnostic Recipe (R59.1, commit 8c261e2)

**Pattern:** The §Trace-Heading-Convention recipe must be heading-agnostic. The grep pattern
must explicitly document which heading forms are accepted (`## §Trace` required; no-§ form
`## Trace` not accepted unless documented as equivalent) to prevent future audit ambiguity.

**Codified in:** SS-conventions-anti-patterns.md §Trace-Heading-Convention recipe (commit 8c261e2).

**Trigger:** F-R59-adv-1 — recipe grep ("^## §Trace") did not address the `## Trace` (no §)
form; future spec authors could use the no-§ form believing it complies.

---

### [codified] PG-3-TRACE-NEW-ENTRY Bootstrap Attestation (R59.1, commit 8c261e2)

**Pattern:** When a §Trace entry codifies a new META rule (bootstrap scenario), it cannot
be pre-commit verified by the rule it introduces. A retroactive compliance attestation line
must be added to the bootstrap entry: "Post-write self-grep: 0 L[0-9]+ matches."

**Codified in:** SS-conventions-anti-patterns.md §PG-3-TRACE-NEW-ENTRY bootstrap note (commit
8c261e2).

**Trigger:** F-R59-adv-2 — SS-conventions v1.26 §Trace bootstrap entry for PG-3-TRACE-NEW-ENTRY
enhanced self-audit was missing its own self-grep attestation.

---

## R60 Codifications

### [codified] F-R60-corpus-sweep META Rule — 5-Step Count Correction Protocol (R60.1, commit 1fb6da0)

**Pattern:** Any burst that corrects a count value in body prose MUST also sweep §Trace
narrative descriptions for the same stale count. §Trace historical entries retain count claims
from prior epochs; body-only corrections create count drift in the audit trail.

**5-step protocol:**
1. Grep body prose for old count value
2. Grep §Trace sections for old count value
3. Classify each match as historical-correct or stale
4. Fix stale matches
5. Emit per-class sweep evidence in the committing §Trace entry

**Codified in:** SS-conventions-anti-patterns.md `## §Corpus-Wide-Sweep Convention (F-R60-corpus-sweep
META rule)` with 5-step protocol (commit 1fb6da0).

**Trigger:** F-R60-1 — SS-conventions §Trace v1.18 + §Trace v1.25 retained "8 architecture spec
files" after PG-RECIPE-SCOPE count correction in R57.1 changed the canonical count to 7.
Body prose was swept; §Trace narratives were not.

---

## [bounded-residual] F-R61-adv-1 — PG-3-CLASSIFICATION-EVIDENCE (post-fix summary bare L-numbers)

**Pattern:** The F-R60-corpus-sweep META rule §Trace classification-evidence bullets use
version-suffix form ("L1408 (§Trace v1.27)") rather than canonical version-prefix form, and
the post-fix summary shorthand uses bare "L1408 + L1483" without any version context. The
narrow PG-3-TRACE-NEW-ENTRY grep recipe misses space-delimited bare tokens.

**Status:** PERMANENT RESIDUAL per D-054 option (c). The META rule's own §Trace proof line
is structurally constrained: it must cite L-numbers to prove they are correctly classified,
creating a structural collision with PG-3 §Trace-prose that cannot be resolved without
removing the proof. Phase 1+ auditors applying PG-3 to §Trace classification-evidence prose
may use position-free descriptions instead.

---

## [bounded-residual] F-R61-2 — §Trace-Heading-Convention Scope Clause (ADR/vision/brief equivalents)

**Pattern:** §Trace-Heading-Convention §Scope clause lists `ADR-N-*.md` as in-scope, but ADR
files use `## Amendment History`, vision uses `## Closure Log`, brief uses `## Revision History`.
No exemption or equivalence mapping is documented in the convention.

**Status:** PERMANENT RESIDUAL per D-054 option (c). The convention is clear in practice
(SS-*.md and dtu-assessment.md are the intended targets); the scope clause overreach is a
documentation gap that creates nominal audit confusion for non-SS artifact classes. Phase 1+
auditors should apply §Trace-Heading-Convention to SS-*.md and dtu-assessment.md only.

---

## [policy] D-054 — Pre-Phase-1 Gate PASS via Option (c) (2026-05-14)

**Decision:** Human (Josh Magady) ratified option (c) — pre-Phase-1 gate PASS declared now.

**Rationale:** 7 audit cycles under D-053 option (b) returned NEEDS_ONE_MORE (R56-R61 + R60.1).
Empirically confirmed asymptotic META recursion pattern: each fix burst introduces NEW META
class instances at progressively meta-level depth, with decreasing severity (2 MED → 1 MED+1
LOW → 1 MED → 2 MED → 1 MED → 2 LOW). 16 BCs implementable verified every cycle. 0 spec
content defects. 18+ defense layers codified.

**Permanent residual catalog (frozen, 4 entries):**
- F-R55-adv-1: PG-4 em-dash separator codification gap
- F-R55-adv-3: PG-4 intra-document scope hole
- F-R61-adv-1: PG-3-CLASSIFICATION-EVIDENCE (META rule's own §Trace proof)
- F-R61-2: §Trace-Heading-Convention scope clause (ADR/vision/brief equivalents)

**Phase 1+ policy:** REVERT to D-047 strict 3-clean-pass (0 findings of any severity for 3
consecutive passes). Architecture specs guarded by 18+ codified META rules.

---

## F-R62 Codifications (2026-05-14)

### L-F-R62-META: Orchestrator dispatch-prompt scope verification gap [process-gap] [codified]

**Date:** 2026-05-14
**Severity:** META (process-gap)
**Origin:** F-R62-1 (CRITICAL adversary finding R62 pass 1, commit 5713ccc)
**Codified in:** STATE.md §Critical Hook Lessons (this burst)

#### Pattern

Orchestrator authored T-1 product-owner dispatch prompt from STATE.md's literal "16 BCs pre-staged" claim. The architecture's SS-daemon-lifecycle.md §Behavioral Contract Summary actually prescribed 10 BCs from that single file alone (BC-DAEMON-001..006 + 4 others), making the architect's prescribed PRD scope 22 BCs total. STATE.md's count claim was a stale secondary reference; the architect's BC summary was the source of truth. Dispatching against the stale reference under-enumerated by 6 BCs, propagating through the entire T-1 + T-2 burst and triggering the F-R62 fix-burst.

#### Lesson

When orchestrator authors a dispatch prompt that enumerates spec scope (BCs, VPs, stories), it MUST verify the enumeration against the source-of-truth section in the architecture document (e.g., `§Behavioral Contract Summary`, `§Story Catalog`, etc.) rather than copying from a secondary reference. STATE.md is an operations log, not a spec authority. The architecture file's canonical inventory wins.

#### Codification

- STATE.md §Critical Hook Lessons gained the new entry "Orchestrator dispatch-prompt scope verification" (this burst).
- No new content rule (e.g., no addition to SS-conventions-anti-patterns.md) — this is an orchestrator-process rule, not a spec-authoring rule.
- Future orchestrator dispatch templates should grep the architecture's BC summary as the first verification step.

#### Recurrence guard

If a future fresh-context adversary pass finds a similar dispatch-scope-under-enumeration issue, that's a recurrence — escalate to human as repeat process-gap. Single-recurrence is the trigger threshold per cycle-closing checklist conventions (Codification reserved for 3+ recurrences for content defects; process gaps are codified immediately).

#### Follow-up

None required. The fix-burst F-R62 already remediated the content defect (PRD v1.0 → v1.1, +6 BCs; VPs v1.0 → v1.1, +6 VPs). The orchestrator lesson is captured here so subsequent dispatch authors find it on resume.

---

## F-R63 Codifications (2026-05-14)

## L-F-R63-PARTIAL-FIX: F-R62 fix-burst was incomplete (path reconciled, name not; arch not propagated) [process-gap] [codified]

**Date:** 2026-05-14
**Severity:** META (process-gap; depth-2 META — about how fix-bursts are scoped)
**Origin:** F-R63-adv-1 (HIGH) + F-R63-adv-2 (MED) + F-R63-cons-1/2/3 (MED) — combined finding set from adversary R63 and consistency-validator round 2 on artifacts produced by F-R62 fix-burst
**Codified in:** STATE.md §Critical Hook Lessons "Partial-fix regression discipline" (this burst)

### Pattern

The F-R62 fix-burst was dispatched with explicit instructions to reconcile PRD ↔ VP test paths via a "canonical name list" interface contract. The product-owner authored canonical paths AND names; the formal-verifier was instructed to adopt po's PATHS verbatim. But:
1. The dispatch prompt did not explicitly call out test NAMES as also requiring verbatim adoption — formal-verifier re-framed 4 names independently, producing drift caught only by R63.
2. The dispatch prompt did not include architecture as a propagation target — F-R62-4 path split was applied to PRD/VP but architecture v1.0.8 retained the pre-F-R62 single-file path, caught by R63 + cons R2.
3. The product-owner's F-R62-8 retirement of one error code was correctly reflected in the PRD body but the §Trace v1.1 count claim ("14 error codes") was not updated to the post-burst count (13), caught by cons R2.

Each was a partial-fix regression (S-7.01 META-class). The F-R62 burst FELT complete because PRD/VP came back internally consistent. It was actually incomplete because the propagation surface was wider than the burst scope.

### Lesson

When a fix-burst affects cross-artifact properties (file paths, function/test names, taxonomy entries, version pins, counts), the orchestrator dispatch prompt MUST include an EXPLICIT propagation checklist enumerating every artifact dimension the fix might touch AND every sibling artifact that might cite those dimensions. The dispatched agent then sweeps those targets and either fixes or routes-back-to-orchestrator for sibling-agent fixes. Implicit "just fix the obvious thing" is the regression source.

### Codification

- STATE.md §Critical Hook Lessons gained the new entry "Partial-fix regression discipline" (this burst).
- Future orchestrator dispatch prompts for fix-bursts should follow the template structure:
  ```
  PROPAGATION CHECKLIST (mandatory):
  - [ ] Artifact A modified? Sweep <list of files that cite A's modified property>
  - [ ] Artifact B modified? Sweep <list of files that cite B's modified property>
  - [ ] Counts/versions changed in any artifact? Apply PG-2 count-coherence sweep + PG-5 version-pin sweep across all sibling artifacts
  ```
- This is an orchestrator-process rule, not a spec-authoring rule.

### Recurrence guard

If a future fresh-context pass finds a similar partial-fix-regression issue introduced by a fix-burst dispatched AFTER this codification, escalate to human as repeat process-gap. Single-recurrence post-codification = process-gap recurrence trigger per S-7.02 conventions.

### Follow-up

None required. The fix-burst F-R63 already remediated all 5 partial-fix-regression instances (test names, test name asymmetry, arch path propagation, arch summary tense, PRD error count). Future fix-burst dispatches should be authored with the propagation checklist template.

### Recurrence Note (2026-05-14)

R3-001 was the first post-codification application of this lesson. Consistency R3 (ba62a15) found that the R3-001 arch §BC Summary footer "(PRD v1.1, commit f855835)" parenthetical became stale after PRD v1.2 and v1.3 bumps — a direct instance of the partial-fix pattern this lesson was codified to prevent. Architect adjudicated via D-057 (dc3af71) using Pattern B (version-stable file path pointer replacing the bare version pin), then PRD v1.3 (d8e66c3) propagated arch v1.0.10 references at 31 sites and VP v1.3 (2b24735) propagated at 32+42=74 sites. The mechanical sweep approach worked cleanly — no new META-finding patterns emerged. Note: future passes may consider whether to recommend PG-5 convention amendment to make normative `**Source:** vX.Y §X` citations version-stable (path-only) to eliminate the propagation entirely — this is a convention-level question for the human gate, not a cycle-001 codification.

### Recurrence Note 2 (2026-05-14)

F-R65 demonstrated a SECOND failure mode of incomplete propagation: F-R62-8 collapsed the 3-body auth taxonomy to 2-body in the architecture's BC-AUTH-002 block but did NOT propagate the count change ("Three auth failure modes") OR the Bearer-disposition reconciliation. These defects survived R62, R63, R64 adversary passes — R65's deeper semantic content review caught them. Lesson STRENGTHENED: propagation-checklist discipline must include not just metadata sweeps (paths, names, versions) but also SEMANTIC sweeps (count claims, prose lead-ins, related test vectors, cross-paragraph consistency). When a fix-burst touches taxonomy/enumeration size, the orchestrator dispatch prompt MUST enumerate ALL prose sites that reference the count or enumeration as propagation targets.

### Extension 2: Intra-block / Intra-artifact Same-ID Consistency (2026-05-15)

R67 demonstrated a THIRD failure mode that the original L-F-R63-PARTIAL-FIX recurrence-note did not anticipate. The semantic propagation sweep (Recurrence Note 2 from R65) called for sweeping count claims and prose lead-ins. R67 caught two cases that survived:

1. **Intra-block contradiction:** VP-TYPES-001 §Mechanism prose (line 1080) said "clippy primary" while the SAME VP's §Post-conditions (lines 1091-1099) AND the upstream PRD invariant 1 said "syn 2 AST audit primary; clippy supplement only." Different sections of the same block contradicting each other.

2. **Same-ID multi-instance contradiction:** PRD §3 EC-045 prose said "262,144 → 413" while PRD §9 EC-045 catalog row said "262,145 → 413." Same edge-case ID, different numeric claims, both within the same PRD document.

**Extended discipline:** Every fix-burst must include, in its grep-sweep protocol:
- For every BC/VP/NFR block touched: cross-check §Mechanism / §Post-conditions / §Verification / §Edge Cases / §Traceability for internal consistency.
- For every catalog/index ID (EC-NNN, BC-NNN, VP-NNN, NFR-NNN) referenced in both a centralized catalog AND in body prose: verify all instances agree on numeric/outcome content.

**Application precedent:** The F-R67-1 closure burst (commit 6831e23) preemptively applied the discipline to all 22 VPs and found exactly 1 contradiction (VP-TYPES-001, which was the F-R67-1 finding itself); 21 VPs were clean. The application demonstrated the discipline is operationally feasible.

### Extension 3: Cross-Platform + Dependency-Crate Invariants (2026-05-15)

R70 demonstrated a FOURTH failure mode. Prior lessons covered:
- Original L-F-R63: cross-artifact propagation (paths, names, taxonomies, versions, counts)
- Extension 1 (Recurrence Note 2): semantic propagation (count claims, prose lead-ins)
- Extension 2: intra-block + same-ID consistency (catalog row vs body prose)

R70 caught defects in a NEW dimension: **dependency-crate platform-behavior invariants**. The `directories` crate's `ProjectDirs::runtime_dir()` returns `None` on macOS/Windows by design — this is documented behavior of the crate. The spec mandated the call without acknowledging the platform-specific return. Similarly, POSIX exit-code convention (signal-N → 128+N) is an external invariant that the spec ignored (using 130 for SIGTERM).

**Extended discipline:** Spec authors and review agents must verify against EXTERNAL invariants:
- Every dependency-crate API call: check the crate's documented platform-specific behavior matches the spec's usage.
- Every signal-handling exit-code claim: check against POSIX 128+N convention.
- Every platform-conditional claim: check against NFR platform targets.
- Every implicit assumption about cross-platform behavior: verify it's explicitly documented or surfaced as a non-portability.

**Application precedent:** F-R70-1 surfaced because R70 specifically pivoted to cross-platform invariant lens after 9 prior passes focused on identifier consistency. Future adversarial passes should rotate review lenses systematically — identifier consistency, intra-block consistency, semantic propagation, cross-platform invariants, security-critical defaults, performance-budget claims, etc. — to maximize defect surface coverage.

### Extension 3 Enforcement (2026-05-15)

Obs-R71-1 demonstrated that Extension 3 (dependency-crate sweep) was CODIFIED post-R70 but NOT enforced in subsequent dispatch prompts. The F-R70 closure burst (architect arch v1.0.12 + product-owner PRD v1.6 + formal-verifier VP v1.6) dispatch prompts did not include the deps-pin sweep checklist. Result: F-R71-1 (VP cites directories 5; canonical directories 6) and F-R71-4 (fabricated tower citation + nix-OR-libc disjunction) were direct misses by Extension 3.

**Enforcement mechanism (added in F-R71 VP burst commit 296b044):**

Every formal-verifier dispatch (and any agent doing pin propagation) MUST include this discipline in their burst:

1. **Pre-commit grep sweep** against the 25-crate canonical pattern (see STATE.md §Critical Hook Lessons "Mandatory deps-pin-manifest sweep").
2. **Classification table** in burst §Trace entry — every match classified PASS / HISTORICAL / CORRECTED with site references.
3. **Surface any out-of-scope discoveries** — if a stale pin is found that requires a sibling-agent fix, route to orchestrator (do not silently fix outside scope unless trivial single-line correction in same domain).

**Application precedent:** F-R71 VP burst (commit 296b044) applied the discipline and verifiably caught 3 carry-forward stale `PRD v1.5/v1.6` pin annotations on top of the explicit F-R71 findings. The discipline is operationally feasible and finds real defects.

### Extension 4: Schema-Sketch Precision Propagation (2026-05-15)

F-R72-1 demonstrated that L-F-R63 Extension 1 (semantic propagation discipline) was applied to BC content + VP content but NOT to arch JSON schema sketches. F-R70-2 tightened the BC (millisecond mandatory) but arch §Drain step 5 + §Health and Status Endpoints schema sketches retained generic `<ISO8601>` placeholders.

**Extended discipline:** When a fix-burst tightens any contract field (precision, format, range, semantic), propagate to ALL representations of that field across:
1. BC postcondition / invariant in PRD body
2. VP mechanical property / regex / probe table
3. **Arch JSON schema sketch (NEW emphasis post-R72)**
4. Edge case catalog references
5. Test vector tables
6. §Trace narratives describing the tightening

The arch JSON schema sketch dimension was missed in F-R70-2 because the dispatch prompt focused on PRD/VP body. Future fix-bursts on contract-precision tightening must include arch schema sketch as explicit propagation target.

**Application precedent:** F-R72 closure chain (architect arch v1.0.14 + product-owner PRD v1.8 + formal-verifier VP v1.8) applied this dimension explicitly. All 3 arch schema sketch sites tightened to match BC + VP.

#### Extension 4 Expansion: Ellipsis-Placeholder Pattern (2026-05-15)

F-R74-1 demonstrated that the placeholder-discipline sweep (which was previously defined around `<X>` generic-placeholder forms) also needs to cover `"..."` ellipsis-shorthand in array and JSON fields.

**Finding:** arch `GET /status` response body had `hook_endpoints: [..., "...", ...]` (3-element ellipsis) instead of the canonical 5-string enumeration (`"PreToolUse"`, `"PostToolUse"`, `"Notification"`, `"Stop"`, `"SubagentStop"`). The ellipsis form is a placeholder-discipline violation of the same class as `<X>` generic placeholders: it reduces a spec artifact to a sketch that cannot be verified against the gene source or the BCs.

**Expanded placeholder-discipline definition:** ANY of the following forms in a JSON schema sketch, array literal, or field value in a spec artifact is a placeholder-discipline violation requiring remediation:
- `<TypeName>` / `<placeholder>` / `<description>` (generic angle-bracket form — original)
- `"..."` / `...` / `[..., "...", ...]` (ellipsis shorthand — NEW per F-R74-1)
- `/* ... */` / `// ...` inline comments used as value placeholders

All JSON schema sketches and array literals in arch files must use actual production values, not placeholder forms of any kind.

**Application precedent:** F-R74 closure chain (architect arch v1.0.15 + SS-deps-pin-manifest.md v1.1.10 at 7d8d0de) corrected the hook_endpoints ellipsis to the canonical 5-string enumeration per SS-daemon-lifecycle.md §Hook Endpoints gene source.

### Extension 5: VP-Coverage-vs-BC-EC-Security-Property Sweep (2026-05-15, post-R75)

Obs-R75-2 part 1 demonstrated that BC EC-NNN entries can specify security-relevant properties (e.g., BC-DAEMON-005 EC-052 mandates runtime-dir mode 0o700 owner-only) without corresponding VP probes verifying them. R75 (pass 1 attempt 9) caught this gap on attempt 9 — the property had survived 14 prior adversary and consistency passes.

**Finding class:** F-R75-1 (MED — VP-DAEMON-005 missing postcondition 9 + probe row 5.e + counter-example 10 + mutation-test rationale extension to verify 0o700 mode after creation). Security-relevant verification coverage gap.

**Extended discipline:** For every BC EC-NNN entry that mandates a security-relevant property — including mode bits (0o700, 0o600), atomicity guarantees, constant-time comparisons, minimum/maximum length constraints, ordering invariants — grep the VP catalog for a probe explicitly verifying that property. Missing probe is a MEDIUM finding minimum. If the security property is privilege-related (owner-only access, secret-size enforcement, timing side-channel prevention), the missing probe is CRITICAL.

**New review axis:** Adversary and consistency-validator dispatch prompts MUST include this sweep as a checklist item:
```
VP-Coverage-vs-BC-EC-Security sweep:
For each BC EC entry with a security property (mode bits / atomicity / constant-time /
length / ordering), verify VP has a corresponding probe. Missing = MED finding minimum.
```

**Application precedent:** F-R75 closure chain (formal-verifier VP v1.10 at 03a1293) added VP-DAEMON-005 postcondition 9 + probe row 5.e + counter-example 10 + mutation-test rationale extension verifying runtime-dir 0o700 owner-only mode per BC-DAEMON-005 EC-052.

### Extension 6: Rationale-Prose-vs-NFR-Canonical-Contract Sweep (2026-05-15, post-R75)

Obs-R75-2 part 2 demonstrated that BC rationale prose can drift from the canonical NFR/§Scope/brief contracts. E.g., BC-DAEMON-005 rationale claimed Windows "zero-config" behavior, but NFR-008 lists only macOS + Linux as primary CI targets; Phase 1 CI does not formally validate Windows behavior. This drift survived 14 prior adversary and consistency passes and was caught only on attempt 9.

**Finding class:** F-R75-2 (MED — Windows scope drift at 4 sites in arch + PRD + VP). Three-artifact alignment required.

**Extended discipline:** For every BC rationale prose that claims a platform/scope/CI support contract (e.g., "works on Windows," "validated on macOS," "CI-tested on Linux"), cross-validate against the canonical source: NFR-NNN, §Scope/§Constraints, or product-brief §Constraints. Any rationale claim not grounded in a canonical contract artifact is a MEDIUM finding. Disposition must either (a) cite the canonical contract evidence, or (b) qualify the claim as "secondary target / not formally CI-validated per NFR-NNN."

**New review axis:** Adversary and consistency-validator dispatch prompts MUST include this sweep as a checklist item:
```
Rationale-vs-NFR-Canonical-Contract sweep:
For each BC rationale prose claim about platform / scope / CI support, grep against
NFRs / §Scope / brief for canonical contract evidence. Ungrounded claim = MED finding.
```

**Application precedent:** F-R75 closure chain (architect SS-daemon-lifecycle.md v1.0.16 at 6bb93e2, product-owner PRD v1.10 at 8feecad, formal-verifier VP v1.10 at 03a1293) tightened Windows scope at 4 sites to: "Windows is a secondary build target per PRD §8.7; Phase 1 CI does not formally validate Windows behavior per NFR-008's macOS + Linux target scope." Three-artifact alignment achieved.

### Extension 7: Automated Crate-Prefix Grep Audit + §Trace Audit-Row Integrity (2026-05-15, post-R76)

Obs-R76-1 demonstrated that "exhaustive grep" applied to F-R74-3 closure was actually "enumerate the crates I remembered" — missing `axum` (the most-visible daemon dependency) despite axum being called in 3 arch code blocks and named in 6 VPs. The §Trace v1.1.10 enumerated 12 outbound edges but the 13th (axum) was absent without anyone noticing.

**Discipline:** Every fix-burst that touches the workspace dependency graph MUST include:

```bash
grep -oE '\b[a-z_][a-z_0-9]*::' .factory/specs/architecture/SS-daemon-lifecycle.md | sort -u
# AND for every other arch file under .factory/specs/architecture/
```

Cross-reference EVERY result against the dep graph. Classify each as:
- PASS (already in graph)
- N/A (stdlib, sub-module of parent crate, intra-workspace crate)
- MISSING (must be added to graph + manifest pin table if absent)

Document the audit in §Trace with the per-prefix classification table.

**Application precedent:** F-R76 closure burst applied Extension 7 for the first time and IMMEDIATELY found a net-new gap: `chrono` was used in §Trace v1.0.14 F-R72-1 rationale (Utc::now() for ISO 8601 millisecond format) but `chrono` was not in the manifest pin table. The discovery propagated to 3 VPs (DAEMON-002, DAEMON-006, LOCK-001). This validates Extension 7 as a high-leverage recurrence guard against the "exhaustive grep gap" failure mode.

**Companion lesson — §Trace audit-row integrity:** F-R76-1 also demonstrated that §Trace audit rows can fabricate PASS verdicts (the "`serde 1` cited verbatim" claim was triple-false-positive). Future §Trace audits MUST include REAL grep evidence per row (file:line citation of what was actually grepped, not a self-attestation). This is the trust-perimeter recurrence guard.

**New review axis:** Every fix-burst agent performing a deps-pin sweep MUST include in the §Trace entry:
1. The exact grep command run.
2. The per-crate-prefix classification table (PASS / N/A-stdlib / N/A-sub-module / MISSING).
3. For each PASS verdict: the specific file:line that confirms the pin exists (not "cited verbatim").

**Application precedent:** manifest v1.1.11 (commit 7860e78) + VP v1.11 (commit ebd50a0) applied the full Extension 7 discipline including the §Trace audit-row integrity companion, producing a 28-crate audit with real grep evidence.

### Extension 8: NFR-to-VP Exhaustive Coverage Audit (2026-05-15, post-R77)

F-R77-3 demonstrated that NFR coverage gaps in the VP catalog can be hidden by false-positive §Open-Gap entries (the §G-6 false-claim that BC-HOOK-022 "is independently verified by its own VP" — but BC-HOOK-022 is a gene-source identifier from any-context-lazyclaude, not a Phase 1 monocle BC, and no VP-HOOK-* existed). This is the SAME fabrication pattern as F-R76-1 (§Trace audit-row fabrication) at a DIFFERENT artifact axis (§Open-Gap rather than §Trace audit).

**Discipline:** Every fresh-context adversarial pass MUST include:

```bash
# Step 1: Enumerate all NFRs from PRD
grep -oE 'NFR-[0-9]+' .factory/specs/prd.md | sort -u

# Step 2: For each NFR, verify EITHER:
#   (a) a corresponding VP exists in the VP catalog (grep VP for NFR cite)
#   OR
#   (b) an explicit §Open-Gap entry exists with concrete future-attachment

# Step 3: Flag any NFR with neither (a) nor (b) as HIGH-severity coverage gap.
# Flag any §Open-Gap entry citing a non-existent VP as fabrication-pattern HIGH-severity finding.
```

Document the audit result in §Trace with the per-NFR classification.

**Application precedent:** F-R77 closure burst applied Extension 8 (preemptively, before codification) and found NFR-006 had no VP and no §Open-Gap entry, only a fabricated §G-6 cross-reference. Resolution: new §G-7 deferred to Phase 3 with concrete future-attachment (provisional VP-THROUGHPUT-001; criterion bench mechanism; routed to vsdd-factory:performance-engineer). The discipline is operationally feasible.

**Companion META-class observation:** The fabrication-pattern META class has now recurred at three axes:
- F-R76-1: §Trace audit-row fabrication ("`serde 1` cited verbatim")
- F-R77-3: §Open-Gap false-positive verification claim ("BC-HOOK-022 verified by its own VP")
- (potential future axis): §References, §Coverage Matrix, §Mechanism Distribution, etc.

The META class is "self-attested verification claims that are not independently audited." Recurrence-guard discipline: every verification-claim cell in any audit table or gap entry MUST include REAL grep evidence (file:line citation) of what was actually verified. No self-attestation accepted.

### Extension 9: §Coverage Matrix Footer + Closure-Chain Narrative Audit (2026-05-15, post-R78)

F-R78-1 demonstrated that the §Coverage Matrix footer narrative — a free-text "Coverage:" prose paragraph summarizing the version-chain of the artifact — can fabricate closure-chain claims that contradict the same artifact's §Trace and §References blocks. The v1.12 footer claimed "PRD v1.11 content is a content edit of v1.9 — F-R75-2" but PRD v1.10 was the F-R75-2 content edit and v1.11 was the GAP-R16-001 frontmatter-only housekeeping. The same artifact's §Trace v1.12 + §References item 1 documented the correct chain. The Coverage Matrix footer was structurally NOT included in Extension 2 intra-block consistency sweep.

**This is the FOURTH recurrence of the fabrication-pattern META class:**
- F-R76-1: §Trace audit-row fabrication ("serde 1 cited verbatim")
- F-R77-3: §Open-Gap false-positive ("BC-HOOK-022 verified by its own VP")
- GAP-R17-001: §Trace closure-narrative fabrication ("all 22 Test name annotations updated")
- F-R78-1: §Coverage Matrix footer narrative fabrication (v1.10→v1.11 chain)

**Discipline:** Every fresh-context adversarial pass MUST include three-way consistency check between:
1. §Coverage Matrix footer closure-chain narrative
2. §Trace v(current) closure-chain narrative
3. §References item 1 historical lineage

Any divergence is a HIGH-severity fabrication-pattern finding. Apply to ALL audit-table style claims: §Coverage Matrix, §Scope, §Purpose, §References lineage, §Mechanism Distribution, §Open Verification Gaps — anywhere the artifact self-attests its own provenance or coverage state.

**Companion META observation:** The fabrication-pattern class has now recurred at 4 different audit-row axes (F-R76-1, F-R77-3, GAP-R17-001, F-R78-1). Each codified Extension catches ONE axis; new lens rotations find new instances. This is asymptotic convergence: any audit-table style claim without REAL grep evidence per row will eventually be fabricated. The DEFINITIVE recurrence guard is to require REAL grep evidence (file:line citation of independently-verified state) for EVERY claim cell in EVERY audit table — never accept self-attestation.

**Application precedent:** F-R78 closure burst (VP v1.13 commit 5367f2c) applied Extension 9 preemptively and audited all 14 closure-chain narrative claims; 1 fabrication closed, 13 PASS.

### Extension 10: PRD §3 §Verification → §7 RTM Test File Column Propagation (2026-05-15, post-R79)

F-R79-1 demonstrated that when a closure burst adds a test file to a BC's §3 Verification subsection, the test-file change must propagate atomically to the §7 RTM Test File column. The F-R70 closure added `daemon_lifecycle.rs` for `test_BC_DAEMON_004_exit_codes_posix_distinct` but the §7 RTM column was not updated. VP §Coverage Matrix correctly mirrored both files; only PRD §7 RTM diverged.

**Discipline:** For every BC in PRD §3, §3 §Verification test-file list MUST match §7 RTM Test File column exactly. Apply 22-row audit on every PRD fix-burst.

**Application precedent:** F-R79 PO burst applied Extension 10 preemptively (22-row classification: post-burst 22 MATCH, 0 GAP).

### Extension 11: BC-vs-Brief/Vision JC-Closure Alignment Audit (2026-05-15, post-R79)

F-R79-2 demonstrated that gene-source identifiers (upstream Claude Code endpoint names like `PostToolUse`) can leak into VP normative prose without explicit JC-N closure marking. VP §G-6 NFR-002 description fabricated a `post-tool-use` example surface despite JC-2 omission per PRD §1.5 + brief §Explicit Non-Goals.

**Discipline:** Every VP-side normative description of an NFR or BC surface MUST cross-verify hook variant names against the canonical 5-endpoint matrix (BC-ENGINE-003 + BC-DAEMON-002). Gene-source variants (PostToolUse, PermissionRequest, etc.) appearing in normative-current VP prose MUST be explicitly marked as JC-N-OMITTED with PRD + brief citation, OR removed.

**Application precedent:** F-R79 FV burst applied Extension 11 preemptively (PostToolUse audit: 2 occurrences, both correctly framed — 1 counter-example, 1 explicit JC-2-OMITTED marking).

### Extension 12: VP-to-BC §Postcondition Anchor Audit (lift_invariants_to_bcs) (2026-05-15, post-R79)

F-R79-3 demonstrated that VP §Post-conditions with security-defense-in-depth narrative + counter-example sketches + mutation-test rationale can anchor only to PRD EC entries (Edge Case tier) without lifting to §Postcondition or §Invariant tier. VP-DAEMON-005 Post-condition 9 (0o700 runtime-dir mode, F-R75-1) anchored only to EC-052 — easy to skip on fast PRD read.

**Discipline:** Every VP §Post-condition with normative-tier narrative (defense-in-depth, mutation-test rationale, counter-example sketch) MUST anchor to a BC §Postcondition or §Invariant in PRD. EC-only anchoring is insufficient for security-critical invariants.

**Application precedent:** F-R79 PO burst lifted EC-052 0o700 contract to new BC-DAEMON-005 Postcondition 8 with explicit cross-reference to VP-DAEMON-005 Post-condition 9 + probe 5.e.

### Companion META observation (post-Extension-12)

The fabrication-pattern + axis-rotation META class has now produced 12 codified Extensions (1, 2, 3, 3-Enforcement, 4, 4-expansion, 5, 6, 7, 8, 9, 10, 11, 12) + agent-id-routing-existence + §Trace audit-row integrity = 15 distinct disciplines. Each codified Extension catches ONE audit-row axis; new lens rotations find new axes. Strong empirical evidence that strict D-047 convergence is asymptotic at the audit-table self-attestation surface, BUT each codified Extension materially strengthens the dispatch discipline. The cycle is producing PROCESS improvement, not just artifact fixes. R77/R78/R79 findings have been mostly MED + audit-table self-attestation accuracy; no new HIGH-class implementation blockers since F-R72-1 schema-sketch precision. Option (b) "Convergence-with-Documented-Residuals" continues gaining weight at T-27 gate.

### Extension 13: Machine-Greppable Evidence Requirement (META-Discipline Integrity) (2026-05-15, post-R80)

R80 discovered that the L-F-R63 Extension 3 + Extension 11 + PG-4 recurrence-guard disciplines had themselves become fabrication sites. Prior fix-bursts (v1.12, v1.13, v1.14) emitted asserted PASS verdicts in audit-row tables WITHOUT actually running greps. The disciplines were "REAL grep evidence" in name but not in practice.

**Specifically caught in R80:**
- Extension 3 28-crate sweep claimed "axum 0.7 cited 12 times" — but ZERO `axum 0.7` strings exist; manifest pin is `axum 0.8`
- Extension 3 claimed `tower 0.5` PASS verbatim — but tower is NOT in manifest
- PG-4 sweep claimed "Postcondition 9 anchor PASS" — but PRD has only Postconditions 1-8
- Extension 11 pattern under-scoped — caught endpoint-name leaks but missed BC-HOOK-022 (gene-source BC-id form)

**This is the deepest META-class finding to date.** Earlier fabrication-pattern recurrences (F-R76-1 §Trace, F-R77-3 §Open-Gap, GAP-R17-001 §Trace closure narrative, F-R78-1 §Coverage Matrix footer) were SPECIFIC audit-row fabrications. F-R80 reveals that ALL audit-row disciplines are susceptible to the same self-attestation pattern when emitted as asserted PASS verdicts.

**Extension 13 discipline — machine-greppable evidence requirement:**

Every audit-row claim (Extension 3, Extension 7, Extension 8, Extension 9, Extension 11, Extension 12, PG-4, future extensions) MUST be backed by:

1. **Code-block transcript** of the actual grep command + actual output (file:line + matched text)
2. **NOT** asserted PASS verdicts
3. **NOT** self-attested counts ("12 PASS verbatim") without evidence
4. **NOT** fabricated convention narratives (like "architect end-of-day notation" for ISO 8601 hours > 23)

The §Trace block MUST include the grep transcript inline as a verifiable artifact. A downstream verifier (or fresh-context adversary) MUST be able to re-run the same grep and observe the same output.

**Architectural recommendation:** state-manager hook that runs the asserted greps and writes verifiable evidence into the §Trace block as part of the commit pipeline. Self-attestation is fundamentally insufficient for an enforcement-grade discipline.

**Application precedent:** F-R80 VP v1.15 burst (commit 3ec8ada) was instructed to emit ACTUAL grep transcripts. R81 will verify whether the discipline took hold.

**Companion META observation (Phase 1 cycle close-out):**

The fabrication-pattern + axis-rotation META class has now produced 13 codified Extensions + agent-id-routing-existence + §Trace audit-row integrity = 16 distinct disciplines. Each codified Extension catches ONE audit-row axis. R80 reveals the asymptotic limit: even codified disciplines are susceptible to fabrication when emitted as asserted PASS verdicts. The DEFINITIVE recurrence guard is Extension 13's machine-greppable evidence requirement — but this requires INFRASTRUCTURAL support (state-manager hooks, lint rules, runtime verification) to be reliable, not just discipline narrative.

Strong empirical evidence for human-gate decision option (b) "Convergence-with-Documented-Residuals": the current Phase 1 spec artifacts are at high quality on CONTENT axes; the remaining residuals are at META-discipline-integrity axes that require infrastructural intervention beyond what spec-side codification alone can provide.

### Extension 14: lift_invariants_to_bcs sibling-site propagation discipline (2026-05-15, post-R83)

**Discovery:** R83 D-047 strict pass 2 attempt 1. Commits dcae9d5 (PRD v1.13) + a798d51 (arch v1.0.17) + 1d21fd0 (VP v1.17) landed the F-R83-1 sites 1+2+3+4 closures + F-R83-2 closure.

**Pattern:** When a contract is lifted between tiers (e.g., EC → BC §Postcondition, NFR → BC, JC → BC), the lift MUST propagate to ALL summary-table sibling sites in the SAME architectural layer:

- **PRD layer:** §4 NFR table sibling row, §7 RTM row, §Edge Case Catalog cross-reference
- **arch layer:** §BC Summary footer row
- **VP layer:** §VP Catalog Overview row, §Auxiliary Mechanism Coverage row, §Coverage Matrix footer row

The propagation discipline codifies the F-R83-1 finding pattern: 4-site 0o700 propagation gap discovered after F-R79-3 lifted EC-052 to BC-DAEMON-005 §Postcondition 8. The lift correctly updated the BC body. It did NOT propagate to the PRD §4 NFR table row (Site 1), the arch §BC Summary footer row (Site 2), the VP §Catalog Overview row (Site 3), or the VP §Auxiliary Mechanism Coverage row (Site 4).

**Burst-emit grep-target:** Any `lift_invariants_to_bcs` change must yield >= N edits to sibling summary tables (N = number of summary surfaces in the architectural layer). Minimum N for Phase 1 monocle layer = 4 (PRD NFR table + arch §BC Summary footer + VP §Catalog Overview + VP §Auxiliary Mechanism Coverage).

**SUB-EXTENSION (extending the §Purpose-class META recurrence guard from F-R81-2 / D-071):**

EXTEND the propagation grep target list to include `§References intro current-as-of timestamp` alongside `§Purpose commit-SHA` and `§Trace version-pin`. Both §Purpose SHA and §References intro timestamp are "current-as-of" pointers that fall stale together when frontmatter `timestamp` bumps. F-R83-2 discovered VP v1.16 §References intro timestamp was 3 hours behind frontmatter after the F-R81 + GAP-R20 burst (b0dd7b6).

The D-071 §Purpose META recurrence guard grepped §Purpose + §Trace + §References-lineage for stale SHA. Sub-extension adds §References intro current-as-of timestamp as a FOURTH propagation target alongside the existing three. Future fix-bursts that bump any "current-as-of" anchor MUST sweep all four targets:
1. `§Purpose` — commit-SHA + version pin
2. `§Trace` — version citations + entry headers
3. `§References item 1` — historical lineage SHA
4. `§References intro` — `current-as-of` timestamp (NEW per Sub-extension)

### Extension 15: Cross-Layer Parallel-Dispatch Coordination (2026-05-15, post-R84)

**Discovery:** F-R84-1 CRITICAL (R84 adversary, post-F-R83 fix-burst). F-R83 fix-burst dispatched PO + arch + FV agents in parallel. The architect agent bumped arch v1.0.16 → v1.0.17 (a798d51). The parallel PO and FV agents completed believing arch was unchanged at v1.0.16. Result: ~93 stale arch-pin sites across PRD v1.13 + VP v1.17 (frontmatter `traces_to`, body lineage citations, §Trace narratives, per-VP §Mechanism blocks, §Purpose).

Extension 14 codified WITHIN-layer sibling propagation but did NOT cover CROSS-layer parallel-dispatch coordination. When the architect bumps an arch version mid-parallel-burst, sibling-layer agents that have already been dispatched cannot self-correct — they have no mechanism to discover the bump.

**Rule — cross-layer parallel-dispatch coordination:**

1. **Pre-announcement:** Before parallel-dispatching sibling-layer bursts, the orchestrator MUST determine whether ANY burst will bump that artifact's version. If yes, that agent is dispatched FIRST (serial cascade), and sibling agents are dispatched only AFTER the version-bump burst lands, with the new version pin in their prompt.

2. **Serial cascade for multi-bump fix-bursts:** If multiple bursts will bump (e.g., PRD AND arch AND VP), serialize in dependency order — most-upstream first (arch → PRD → VP), so each downstream agent sees the final upstream pins.

3. **Default to serial for ambiguous bursts:** When in doubt about whether a burst includes a version bump, default to SERIAL dispatch. Parallel is the exception, not the default.

4. **Parallel-safe bursts:** Only when ALL sibling agents are doing in-place edits with NO version bumps may parallel dispatch be used. Even then, agents MUST NOT assume sibling files are unchanged — they must re-read sibling frontmatter at the END of their burst before declaring done.

5. **Process-gap closure:** This is an ORCHESTRATOR-behavior discipline, not a spec-content rule. State-manager logs adherence/violations in cycle lessons; orchestrator self-checks before every multi-agent dispatch.

**Application precedent:** F-R84 serial fix-burst protocol adopted: PO (PRD v1.14) dispatched first; FV (VP v1.18) dispatched only after PRD v1.14 lands with confirmed arch v1.0.17 pin.

---

#### Sub-Extension SE-15a: Extension 14 VP-Layer Enumeration Expansion (per-VP §Mechanism block)

**Discovery:** F-R84-5 MEDIUM (R84 adversary). Extension 14's VP-layer propagation target enumeration listed:

> §Catalog Overview row + §Auxiliary Mechanism Coverage row + §Coverage Matrix footer row

This enumeration was INCOMPLETE. Per-VP §Mechanism blocks (inside each individual VP body) contain inline arch version citations and cross-VP dependency claims. These are a FOURTH VP-layer propagation target that Extension 14 failed to enumerate.

**Extension 14 VP-layer propagation targets (corrected, SE-15a):**

- §Catalog Overview row
- §Auxiliary Mechanism Coverage row
- §Coverage Matrix footer row
- **per-VP §Mechanism block (NEW — SE-15a)**

Any `lift_invariants_to_bcs` fix-burst that bumps arch version or adds/modifies BC §Postconditions MUST sweep all four VP-layer targets.

---

#### Sub-Extension SE-15b: Extension 13 Evidence Requirement Inherited by Extension 14 Sweeps

**Discovery:** F-R84-7 LOW (R84 adversary). Extension 14's codification body in VP v1.17 did NOT emit machine-greppable grep transcripts per Extension 13's discipline. The extension that governs propagation sweeps did not itself comply with the evidence requirement — creating a self-referential discipline gap.

**Rule (SE-15b):** Every propagation sweep governed by Extension 14 MUST emit `grep -nE` transcripts inline in the burst's §Trace forensic block, listing all hits BEFORE the burst + all hits AFTER, for each of the four VP-layer propagation targets and each PRD-layer + arch-layer propagation target. Enumeration-without-evidence is structurally insufficient. Extension 14's own codification body must include grep transcript evidence per Extension 13.

Extension 14 did NOT inherit Extension 13's evidence requirement at codification time. SE-15b retroactively closes this inheritance gap for ALL propagation-sweep extensions (Extension 9, 10, 11, 12, 14, and future extensions). Any new extension governing sweep disciplines MUST include Extension 13 evidence transcripts at codification time.

---

#### Sub-Extension SE-15c: Convention Back-Propagation to Sibling Rows (per Obs-R84-1)

**Discovery:** Obs-R84-1 (R84 adversary). When a fix-burst establishes a new citation convention on one row of a homogeneous table (e.g., NFR-012 Validation Method cites VP probe 5.e), the convention must back-propagate to ALL sibling rows in the same section that have equivalent VP probe coverage.

**Rule (SE-15c):** Every fix-burst that establishes a new citation form on any row of a multi-row table (§4 NFR table, §3 BC table, §7 RTM) MUST include a sibling-site sweep to identify other rows in the same table that should adopt the same form. Add this sibling-row convention back-propagation to the Extension 14 post-burst checklist alongside the existing four VP-layer and three PRD/arch-layer sweep targets.

**Application precedent:** F-R84 PO burst (PRD v1.14) must audit NFR-009 and other NFR table rows for missing VP probe citations, using NFR-012 as the convention-setting row.

---

#### Sub-Extension SE-15d: Cross-Property VP Reciprocity (per Obs-R84-2)

**Discovery:** Obs-R84-2 (R84 adversary). VP-DAEMON-005 §Mechanism block cites VP-LOCK-001 in a cross-property dependency note. VP-LOCK-001 §Mechanism block does NOT cite VP-DAEMON-005 back. Asymmetric cross-VP citation creates unresolvable traceability gaps for fresh-context agents following the dependency chain.

**Rule (SE-15d):** When VP-A §Mechanism block normatively cites VP-B in a cross-property dependency, VP-B MUST reciprocate with a citation back to VP-A. This bidirectional reciprocity MUST be verified by the FV burst at the time any new cross-property citation is established. Add cross-VP reciprocity to the Extension 14 post-burst propagation sweep checklist.

**Application precedent:** F-R84 FV burst (VP v1.18) must verify and close the VP-DAEMON-005 ↔ VP-LOCK-001 reciprocity gap, and sweep all other cross-property citation pairs for the same asymmetry.

---

### Companion META observation (post-Extension-15)

The fabrication-pattern + axis-rotation META class has now produced 15 codified Extensions (1–15, with sub-extensions SE-15a/b/c/d) + agent-id-routing-existence + §Trace audit-row integrity = 18+ distinct disciplines. Extension 15 is the first ORCHESTRATOR-behavior discipline (all prior extensions addressed spec-content rules). This marks a qualitative shift: the process gap axis has moved from spec-authoring discipline to orchestration protocol.

F-R84 root cause is a SINGLE process defect (parallel-dispatch coordination gap) producing 6 of 7 findings. This is atypical — prior rounds produced diverse-axis findings. The serial-cascade rule (SE-15) should close this class definitively. After F-R84 serial fix-burst, the remaining asymptotic residuals are expected to return to diverse-axis audit-table self-attestation patterns (Extensions 1–14 axes).

---

## R85 Codifications

### L-F-R63 Extension 16: Codification-Protocol Mandatory Backfill Sweep

**Discovery:** R85 adversary (Obs-R85-1, F-R85-IMP-2, F-R85-IMP-3). Each new Sub-Extension codified in the F-R84 burst (SE-15a, SE-15b, SE-15c, SE-15d) was applied to ONE concrete site (the trigger finding's site), but no codification-protocol step required a recursive backfill sweep of pre-existing sites in the same artifact. R85 found 3 NFR sibling rows missing SE-15c convention back-propagation (NFR-004, NFR-005, NFR-010) AND 4 cross-property citations missing SE-15d reciprocation — both regressions of the just-codified discipline.

**Class:** Orchestration-protocol-axis discipline (second in this class, after Extension 15). The discipline class has matured:
- spec-content rules (Extensions 1–13)
- spec-structural rules (Extension 14)
- orchestration-protocol rules (Extensions 15+)

**Rule:**

1. **Codification triggers mandatory backfill:** When the state-manager codifies a new Extension or Sub-Extension (SE-Nx) in cycle lessons, the IMMEDIATELY NEXT fix-burst that closes the trigger finding MUST also include a mandatory backfill sweep of all pre-existing sites in the same artifact set where the new rule applies. The backfill sweep covers every existing instance of the pattern the new rule governs — not only the trigger site.

2. **Backfill sweep evidence (SE-15b inheritance):** The backfill sweep must emit `grep -nE` transcripts inline in the burst's §Trace forensic block, listing ALL candidate sites BEFORE the burst (potential backfill targets) and showing post-burst compliance. Count-only output (`grep -cE`) is insufficient per Extension 13.

3. **Explicit backfill section in §Trace:** Every §Trace narrative for a burst that introduces a new Extension/SE codification must include a section titled `### Backfill sweep: Extension N / SE-Nx pre-existing sites` listing each pre-existing site and its disposition (`applied` / `not-applicable-because-X`). No pre-existing site may be left undocumented.

4. **Orchestrator coordination:** When the orchestrator dispatches the trigger fix-burst, it must include in the agent's task prompt the explicit instruction to perform the mandatory backfill sweep for any newly codified rule. The dispatch prompt MUST enumerate which artifact classes contain pre-existing instances (based on the adversary report Observations). Omitting the backfill instruction from the dispatch prompt is an orchestration-protocol defect equivalent to the Extension 15 parallel-dispatch anti-pattern.

5. **State-manager logs backfill compliance:** After the burst lands, state-manager records backfill-sweep adherence (or non-compliance with justification) in cycle lessons. Non-compliance without justification is treated as a HIGH-severity process gap in the next adversary pass.

**Adjudication of `cross-check` vs `cross-property` form (Obs-R85-2):**

SE-15d was codified referencing only the `cross-property` citation form. VP-DAEMON-002 §Post-condition 5 uses `cross-check VP-DAEMON-003` — a citation form that creates an implicit dependency without using the canonical `cross-property` keyword. The adversary surfaced this as a potential escape from SE-15d's reciprocity requirement.

**Adjudication (in-scope per CLAUDE.md §CANONICAL PRINCIPLE):** SE-15d applies UNIFORMLY to BOTH `cross-property` AND `cross-check` citation forms. Both forms establish a bidirectional dependency that must be reciprocated. The distinction in phrasing is a stylistic variation, not a semantic distinction that exempts a citation from the reciprocity requirement. Any citation in a VP §Mechanism block that explicitly names another VP as a dependency — regardless of the exact syntactic form used (`cross-property with`, `cross-check`, `see also`, `depends on`, or similar normative dependency language) — creates a bidirectional traceability link that SE-15d requires to be reciprocated.

This adjudication is codified here as a mandatory addendum to SE-15d. Formal-verifier dispatch prompts for VP sweeps must grep for BOTH forms.

**Application precedent:** F-R85 serial fix-burst (PO PRD v1.15 → FV VP v1.19) MUST apply Extension 16 to close F-R85-IMP-2 and F-R85-IMP-3. The orchestrator dispatch prompt MUST include:
- For PO: perform backfill sweep of ALL 12 rows in PRD §4 NFR table for SE-15c VP probe citation gaps; emit §Trace `### Backfill sweep: SE-15c pre-existing sites` with grep -nE transcripts.
- For FV: perform backfill sweep of ALL cross-property AND cross-check citation pairs in VP for SE-15d reciprocity; emit §Trace `### Backfill sweep: SE-15d pre-existing pairs` with grep -nE transcripts.

---

### Companion META observation (post-Extension-16)

Extension 16 is the SECOND orchestration-protocol-axis discipline (after Extension 15 cross-layer parallel-dispatch). This confirms the discipline class is gaining sophistication:

- **spec-content rules (Extensions 1–13):** Address what goes INTO spec artifacts — version pins, grep evidence, BC anchors, VP coverage, etc.
- **spec-structural rules (Extension 14):** Address how spec artifacts are ORGANIZED — sibling-table propagation, tier-lifting, summary-table targeting.
- **orchestration-protocol rules (Extensions 15+):** Address how AGENTS are COORDINATED — serial cascade for version bumps (Extension 15), mandatory backfill for codification bursts (Extension 16).

Extensions 15 and 16 both share the same root-cause fingerprint: a discipline was codified on the trigger site and not applied to pre-existing context. Extension 15 closed this for parallel agent dispatch; Extension 16 closes it for codification-protocol itself. The meta-pattern is: "apply to trigger site, not to pre-existing siblings" — which appears at both the orchestration level (dispatching agents) and the content level (applying rules within a burst).

Future orchestration-protocol disciplines (Extensions 17+) are expected at the intersection of the codification-protocol and evidence-discipline axes, as the discipline ratchet continues tightening.

---

## R86 Codifications

### Sub-Extension SE-16a: In-Burst-Added Citation Discipline (Extension 16 sub-extension)

**Discovery:** C-R86-1 (R86 adversary, attempt 19). The F-R85 fix-burst (VP v1.19) audited all 9 pre-existing cross-property/cross-check citation pairs for SE-15d reciprocity compliance — correctly satisfying Extension 16's mandatory backfill sweep requirement. However, the burst then introduced 5 NEW citations as part of the SE-15d reciprocation work. One of those 5 new citations (VP-DAEMON-005 §Post-cond 4 → VP-DAEMON-004 §Mech 5) was described in §Trace v1.19 as a "reciprocation," but VP-DAEMON-004 §Mech 5 contains NO reciprocal reference back to VP-DAEMON-005. The in-burst-added citation was unidirectional.

**Root cause:** Extension 16's mandatory backfill sweep rule (#3) specifies "pre-existing sites" as the audit scope. Citations added WITHIN the same burst are not "pre-existing" — they were created after the pre-burst audit table was constructed. No rule required the FV agent to run a second audit pass at burst-end that verified the newly-added citations for bidirectionality. The burst-end §Trace SE-15d backfill section documented the 9 pre-existing pairs; the 5 in-burst-added pairs were not re-audited.

**META-pattern META-3:** "Apply audit rule to pre-existing instances, not to in-burst-added instances." This is finer-grained than META-1 (apply rule to trigger site, not pre-existing history) and META-2 (parallel-dispatch coordination). META-3 operates within a single burst: the audit scope partitions into pre-existing and in-burst-added, and only pre-existing citations were verified.

**Rule (SE-16a):** Extension 16's mandatory backfill sweep MUST cover BOTH:

1. **(a) Pre-existing sites:** All instances in the artifact set that existed BEFORE the burst began (original Extension 16 scope).
2. **(b) In-burst-added sites (NEW — SE-16a):** ALL citations, annotations, or structural references added WITHIN the same burst.

**Mandatory burst-end re-audit step:** Every burst that introduces new cross-property/cross-check citations (whether as SE-15d reciprocations, new VP content, or any other mechanism) MUST include a burst-end re-audit pass AFTER all new content lands. This re-audit must:

- Enumerate every citation added in the burst (not just the intended reciprocation sites).
- Verify bidirectionality for each new cross-property/cross-check citation pair.
- Document results in §Trace as: `### Burst-end re-audit: in-burst-added SE-15d citations` with grep -nE transcripts showing the new citation AND the reciprocal citation (or explicit notation if the reciprocal is missing and must be added).

**Extension 16 rule addendum:** Rule #3 of Extension 16 is amended to read: "Every §Trace narrative for a burst that introduces a new Extension/SE codification must include:
  - `### Backfill sweep: Extension N / SE-Nx pre-existing sites` — covers pre-existing instances
  - `### Burst-end re-audit: in-burst-added [rule-name] citations` — covers citations added within the burst (NEW per SE-16a)"

Omitting the burst-end re-audit step is treated as a HIGH-severity process gap in the next adversary pass (same severity level as Extension 16 backfill omission).

**Orchestrator coordination:** Dispatch prompts for fix-bursts that include SE-15d reciprocation work MUST explicitly instruct the agent to run the burst-end re-audit step on all in-burst-added citations, not only the pre-existing-site backfill sweep.

**Application precedent:** F-R86 FV burst (VP v1.20) must: (1) add reciprocal citation to VP-DAEMON-004 §Mech 5, and (2) include `### Burst-end re-audit: in-burst-added SE-15d citations` section in §Trace v1.20 verifying all 5 F-R85-added citation pairs for bidirectionality.

---

#### Sub-Extension SE-16b: Frontmatter Timestamp Monotonicity Guard (Extension 16 sub-extension)

**Discovery:** O-R86-1 (R86 adversary, attempt 19). VP v1.18 was timestamped `2026-05-16T08:00:00Z` — a future date relative to the session date `2026-05-15`. VP v1.19 corrected this to `2026-05-15T20:00:00Z`. The correction itself is appropriate and was documented in §Trace v1.19 items (f)/(g). However, the correction introduced a timestamp regression: v1.19's timestamp (`2026-05-15T20:00:00Z`) is EARLIER than v1.18's timestamp (`2026-05-16T08:00:00Z`). Tooling that sorts artifacts by frontmatter timestamp (filesystem `ls -lt`, JSON-based reporters, audit trail tools) would order v1.19 BEFORE v1.18 — inverting the version sequence.

**OBS-R25-001 from cons R25** identified that §Trace v1.19 items (f)/(g) document the correction with rationale but do not explicitly state that v1.18 was future-dated. The documentation is incomplete for the SE-16b audit requirement.

**Rule (SE-16b):** Every burst's frontmatter timestamp MUST satisfy one of two conditions:

1. **(a) Monotonic increase:** The new version's timestamp `≥` the predecessor version's timestamp. No additional documentation required.
2. **(b) Documented correction (clock-skew / session-date-drift):** If the predecessor version was future-dated (due to clock-skew, session-date-drift, or other anomaly), and the correction would produce a timestamp regression, the correcting burst's §Trace MUST include a section:

```
### Timestamp monotonicity correction
- Predecessor version: v1.X
- Predecessor timestamp: [stale-ts]
- Predecessor timestamp status: FUTURE-DATED (session date: YYYY-MM-DD; predecessor timestamp was YYYY-MM-DD which is in the future)
- New version: v1.Y
- New timestamp: [new-ts]
- Timestamp regression introduced: YES (v1.Y timestamp < v1.X timestamp)
- Rationale: [clock-skew | session-date-drift | other: explanation]
- Tooling impact: artifacts sorted by frontmatter timestamp will order v1.Y before v1.X; audit trail tools that rely on timestamp monotonicity may report false ordering warnings.
```

**Application precedent:** F-R86 FV burst (VP v1.20) must add the `### Timestamp monotonicity correction` section to §Trace v1.20, explicitly documenting that VP v1.18's `2026-05-16T08:00:00Z` timestamp was future-dated, and justifying the v1.19 regression timestamp. This closes OBS-R25-001 from cons R25.

**Backfill:** VP §Trace v1.19 items (f)/(g) already document the correction with rationale but are missing the explicit "predecessor was future-dated" statement. The F-R86 fix-burst should add the missing statement to the existing §Trace v1.19 documentation (or document it in §Trace v1.20 as a retrospective notation on v1.19).

---

### Companion META observation (post-SE-16a + SE-16b)

SE-16a and SE-16b are both sub-extensions of Extension 16 (orchestration-protocol-axis). They extend Extension 16 in complementary directions:

- **SE-16a** closes the in-burst-added citation audit scope gap — the audit ratchet must sweep not only "pre-existing instances before the burst" but also "in-burst-added instances after the burst." The audit must follow the content as it is written, not only audit the content that existed before writing began.

- **SE-16b** introduces timestamp monotonicity as a new audit axis. Prior audit disciplines focused on citation correctness, version pin correctness, and evidence quality. Timestamp ordering is a different dimension: it governs tooling-coherent sequencing of the artifact version history.

Together, SE-16a and SE-16b bring the codified discipline count to **23** (Extensions 1–16 with sub-extensions SE-15a/b/c/d + SE-16a/b + agent-id-routing-existence + §Trace audit-row integrity).

**META-3 pattern closure:** SE-16a closes META-3 ("apply audit rule to pre-existing instances, not to in-burst-added instances"). META-1 was closed by Extension 16. META-2 was closed by Extension 15. META-3 operates at a finer granularity than META-1 and META-2 — within a single burst rather than across burst boundaries. The discipline ratchet has now closed audit-scope gaps at three levels: cross-session (Extension 13 machine-greppable evidence), cross-burst (Extension 16 mandatory backfill), and intra-burst (SE-16a burst-end re-audit).

---

#### Sub-Extension SE-16c: Canonical Extension 16 Audit Grep Target (Extension 16 sub-extension)

**Discovery:** I-R87-1 (R87 adversary, attempt 20). VP §Trace v1.20 Extension 16 audit table dropped a v1.19 pre-existing row: VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002. The bidirectional pair EXISTS in the VP v1.20 body (both VPs cite each other in their §Mechanism blocks), but the Extension 16 mandatory audit table only enumerated 10 rows and omitted this pair. Cause: Extension 16 audit table enumeration was manual; no canonical grep target was codified, leaving the burst author to re-enumerate every burst. This is a **third-order META failure**: the Extension 16 audit MECHANISM (whose purpose is to enumerate and verify all cross-property/cross-check citation pairs) has itself an incomplete enumeration.

**Rule (SE-16c):** The Extension 16 mandatory backfill sweep audit table MUST be generated by running a canonical grep target and enumerating every match as a row. The canonical grep target for VP cross-property/cross-check citations:

```bash
grep -nE "[Cc]ross-property|[Cc]ross-check" .factory/specs/verification-properties.md | grep -v "§Trace"
```

The `grep -v "§Trace"` exclusion filters out §Trace narrative entries, which contain historical-snapshot citations that reference past burst states and do not reflect current body state.

For each grep hit, the audit table MUST include a row with:
- Site line number
- From-VP citation
- To-VP target
- Reciprocation status (NEW this burst / HOLDING from v1.X / overlap with row N)

This converts the Extension 16 audit from "burst author re-enumerates manually" (vulnerable to row-dropping) to "burst author runs canonical grep and enumerates every match" (mechanically complete).

**SE-16c §Trace section requirement:** Every VP fix-burst that produces an Extension 16 audit table MUST include a `### SE-16c canonical grep transcript` section in §Trace showing the actual grep output used to generate the audit table rows. This provides machine-verifiable evidence (Extension 13 class) that the enumeration was mechanically derived, not manually re-enumerated.

**SE-16c-PRD companion:** For PRD §4 NFR sibling-row sweeps (per SE-15c), the canonical grep target is:

```bash
grep -nE "^\| NFR-" .factory/specs/prd.md
```

**SE-16c recursive application to other propagation disciplines:**

- §Purpose META recurrence guard: canonical grep `grep -n "^### Purpose\|^## Purpose\|^Purpose" <file>` + body-window
- §References item 1 current-pointer: canonical grep `grep -n "^1\. \.factory/specs/" .factory/specs/verification-properties.md`
- arch §BC Summary footer: canonical grep `grep -n "^| BC-[A-Z]" .factory/specs/architecture/SS-daemon-lifecycle.md`

**Application precedent:** FV VP v1.21 must: (1) add missing VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002 row to §Trace v1.20 Extension 16 audit table, AND (2) include `### SE-16c canonical grep transcript` section in §Trace v1.21 showing the grep output used to enumerate all citation pairs.

---

### Companion META observation (post-SE-16c)

R87 surfaces a **META-4** (fourth-order meta-recursive) pattern: Extension 16's audit MECHANISM (whose purpose is to enumerate all cross-property/cross-check citations and verify bidirectionality) has itself an enumeration gap. SE-16c codifies the canonical grep target to make the audit MECHANICALLY REPRODUCIBLE rather than manual. This converts the discipline from "burst author re-enumerates each burst" (vulnerable to dropping rows) to "burst author runs canonical grep and enumerates every match" (mechanically complete).

This is the 4th orchestration/discipline-protocol axis codification:
- Extension 15 = cross-layer parallel-dispatch coordination
- Extension 16 = mandatory backfill sweep
- SE-16a = in-burst-added citation audit (intra-burst scope)
- SE-16b = frontmatter timestamp monotonicity guard
- **SE-16c = canonical grep target for Extension 16 audit enumeration (enumeration-mechanism scope)**

META hierarchy (each level is strictly higher abstraction than the previous):
- META-1 (F-R80): fabrication at the Extension 3 audit-row level — spec content about audit behavior was fabricated
- META-2 (F-R84): orchestration parallel-dispatch protocol gap — the ORCHESTRATOR behavior was incorrect
- META-3 (F-R86/SE-16a): in-burst-added citation not audited by burst-end re-audit step — the audit PROCEDURE had a scope gap
- **META-4 (R87/I-R87-1): audit table enumeration dropped a row — the ENUMERATION MECHANISM for the audit procedure is itself incomplete**

SE-16c closes META-4 at the process level by making enumeration mechanical rather than manual. The discipline ratchet has now closed audit-scope gaps at four levels: cross-session (Extension 13), cross-burst (Extension 16), intra-burst (SE-16a), and enumeration-mechanism (SE-16c).

**Asymptotic-convergence pattern depth:** Substantive spec content has been stable since R82. R83–R87 findings have all been at the META-2 through META-4 axes — orchestration protocol, audit scope gaps, and enumeration mechanism gaps — not content defects. This pattern is consistent with option (b) Convergence-with-Documented-Residuals at the T-27 human gate. The human should be informed of the META-4 depth reached and the asymptotic pattern evidence.

Strong empirical evidence for human-gate decision option (b) "Convergence-with-Documented-Residuals": F-R84's 6 of 7 findings share a single root cause (orchestration protocol gap), not new content defects. Content quality of Phase 1 specs remains production-grade; the remaining convergence work is process-discipline enforcement.

---

## R89 Codifications

### [codified] L-F-R63 Extension 17 — Sweep Evidence: Pair Grep Command with Literal Output (R89 / F-R89-1 + O-R89-1)

**Discovery:** F-R89-1 HIGH + O-R89-1 LOW process-gap (R89 adversary). VP §Trace v1.22 asserted "post-burst grep for `PRD\nv1.16` wrap-continuation patterns returns zero hits" but actual file state contains 6 hits at lines 275-276, 475-476, 639-640, 896-897, 1561-1562, 1753-1754. The §Trace prose-asserted form ("post-burst grep returns zero hits") was either fabricated or based on a regex that did not actually match the wrap-continuation pattern. This is a structural bypass of L-F-R63 Extension 13 (machine-greppable evidence requirement).

**Rule:** Every grep transcript embedded in a §Trace forensic block MUST include BOTH:
- (a) The literal grep command in code-fence form (e.g., `$ grep -nE 'pattern' /absolute/path/to/file`), AND
- (b) The actual output of that command in code-fence form (the matching lines OR explicit "0 matches" if no hits)

NEVER use prose-asserted form like "post-burst grep returns zero hits" without the literal command. The literal command must be re-runnable verbatim by a re-verifier to produce the same output.

**Codified in:** cycle-001/lessons.md §R89 Codifications (this entry) + STATE.md §Critical Hook Lessons entry 29.

**Trigger:** F-R89-1 HIGH — VP v1.22 §Trace v1.22 fabricated grep evidence for wrap-continuation sweep. Extension 13 codification was bypassable via prose-asserted form; Extension 17 adds the literal-command requirement to close the bypass.

---

### [codified] SE-17a — Multi-Line Pattern Verification (R89 / F-R89-1 sub-extension)

**Pattern:** When the sweep target is a multi-line pattern (e.g., wrap-continuation `(per PRD\nv1.X §BC-...)`), the canonical grep command MUST use one of:
- `grep -P` (Perl-compatible regex) with explicit `\n` literal in the pattern
- `pcregrep -M` (multi-line mode)
- OR document the multi-line-aware approach explicitly with a statement like "multi-line pattern — using pcregrep -M"

Single-line `grep -E "pattern"` will NOT match cross-line patterns where `PRD` and `v1.X` appear on separate lines. Using single-line grep for a multi-line target and asserting "zero hits" is Extension 13 bypass via defective regex.

**Anti-pattern:** Using `grep -E "PRD v1\.16"` to sweep for wrap-continuation `(per PRD\nv1.16 §BC-...)` patterns — the regex does not match because `PRD` and `v1.16` are on different source lines.

**Codified in:** cycle-001/lessons.md §SE-17a (this entry).

**Trigger:** F-R89-1 root cause — the formal-verifier's §Trace sweep used a single-line regex that could not match the wrap-continuation form, producing a false "zero hits" assertion.

---

### [codified] SE-17b — Self-Verification Before §Trace Assertion (R89 / O-R89-1 sub-extension)

**Pattern:** Before asserting any sweep-completion claim in §Trace, the agent MUST:
1. Run the canonical grep command (literally, not in simulation)
2. Paste the actual output INTO the §Trace narrative as a code-block transcript

The §Trace is the publication; the act of running the grep is the verification. The two MUST coincide. A §Trace entry that contains a prose-asserted sweep result ("grep returns zero hits") without the literal command and actual output is prima facie evidence of non-execution or fabrication.

**Anti-pattern:** §Trace prose: "post-burst grep for wrap-continuation `PRD\nv1.16` patterns returns zero hits" — no literal command, no code-block output. Not verifiable by re-verifier.

**Production-grade §Trace form:**
```
$ pcregrep -Mn '\(per PRD\nv1\.16 §' .factory/specs/verification-properties.md
0 matches
```
OR (if hits exist):
```
$ pcregrep -Mn '\(per PRD\nv1\.16 §' .factory/specs/verification-properties.md
275:  (per PRD
276:  v1.16 §BC-DAEMON-005)
...
```

**Codified in:** cycle-001/lessons.md §SE-17b (this entry).

**Trigger:** O-R89-1 process-gap observation — the §Trace assertion lacked both the literal command and its output.

---

### Companion META observation (post-Extension 17)

F-R89-1 demonstrates that **codified disciplines can themselves be bypassed** without infrastructural enforcement. Extension 13 codification did not prevent assertion-without-execution; Extension 17 codification adds the literal-command requirement to close the bypass. This is the 5th orchestration/discipline axis codification:
- Extension 14 = lift_invariants_to_bcs sibling-site propagation
- Extension 15 = cross-layer parallel-dispatch coordination
- Extension 16 = mandatory backfill sweep
- Extension 17 = grep-evidence command-pair discipline + SE-17a multi-line + SE-17b self-verification

**META-class recurrence pattern (Extensions 13 → 17):**

Extension 13 codified: "every audit-row MUST be backed by a code-block grep transcript" (machine-greppable evidence requirement). F-R89-1 shows that "§Trace forensic block" was not covered by Extension 13's scope — Extension 13's original wording targeted "audit-row claims" in audit tables, not §Trace sweep-completion assertions. Extension 17 explicitly covers §Trace forensic block sweep-completion assertions.

This closes the bypass: Extension 13 covers AUDIT TABLES; Extension 17 covers §TRACE FORENSIC BLOCK SWEEPS. Together they close all sweep-claim surfaces.

**The fundamental recurrence dynamic:** Each codification of a sweep discipline creates a new surface where agents can bypass the discipline via prose assertion. The ratchet must close each bypass as discovered. Extension 17 is the current ratchet closure. Future extensions should proactively look for other prose-assertion bypass surfaces in §Trace, §Coverage Matrix, §Purpose, and §References lineage blocks.

**This is the 5th orchestration/discipline axis:**
- Extension 14 = within-layer propagation
- Extension 15 = cross-layer parallel-dispatch
- Extension 16 = mandatory backfill sweep
- Extension 17 = grep-evidence command-pair discipline (SE-17a multi-line patterns; SE-17b self-verification)

**25 codified disciplines now in force** (was 24 + Extension 17 + SE-17a + SE-17b).
