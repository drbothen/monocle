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

---

## SE-15e: Orchestrator SERIAL-cascade dispatch enforcement (Extension 15 sub-extension)

**Discovery:** C-R90-1 (R90 adversary) — 5th recurrence of the SERIAL Extension 15 pin-propagation META failure.

**Pattern history:**

| Recurrence | Finding | Pattern | Resolution |
|---|---|---|---|
| 1 | F-R84-1 CRITICAL | arch v1.0.16→v1.0.17 NOT propagated to PRD + VP; parallel-dispatch root cause | Closed by Extension 15 codification |
| 2 | F-R85 | Similar wrap-continuation escape from F-R84 sweep | Closed by Extension 16 mandatory backfill |
| 3 | F-R88-1 HIGH | arch v1.0.17→v1.0.18 propagation gap | Closed by F-R88 serial fix-burst |
| 4 | GAP-R23-001 / F-R85-IMP-1 | Wrap-continuation stale cites escaped F-R84 sweep (parallel-dispatch root cause variant) | Closed in F-R85 serial |
| 5 | C-R90-1 CRITICAL | arch v1.0.18→v1.0.19 NOT propagated to PRD body; F-R89 chain dispatched `arch → FV`, skipping PO | Requires SE-15e |

**Root cause of C-R90-1:** Orchestrator dispatched F-R89 chain as `SM → architect → FV`, skipping PO PRD pin propagation. The reasoning was "PRD has no R89 findings; therefore no PRD burst needed." This reasoning is WRONG under Extension 15 SERIAL cascade: **the trigger for PO PRD dispatch is the arch version bump itself**, not whether the adversary names PRD-side normative findings.

### SE-15e Rule (codified 2026-05-16, C-R90-1)

**When any agent dispatched in a serial chain bumps an artifact version, the orchestrator MUST dispatch ALL downstream-layer agents in dependency order BEFORE proceeding, regardless of whether the original adversary report names findings in those downstream layers.**

**Specifically for arch version bumps in a serial chain:**

- **MANDATORY:** architect bumps arch v1.0.X → v1.0.Y → orchestrator MUST dispatch PO for PRD pin propagation sweep:
  ```
  grep -nE "v1\.0\.X|commit <old-sha>" .factory/specs/prd.md
  ```
  Sweep all normative-current sites. PO commits PRD bump → orchestrator dispatches FV.
- **FORBIDDEN:** orchestrator dispatches FV directly after architect without PO step (regardless of adversary report scope).

**For PRD version bumps in a serial chain:**

- **MANDATORY:** PO bumps PRD vX.Y → vX.Z → orchestrator MUST dispatch FV for VP pin propagation sweep.
- **FORBIDDEN:** orchestrator dispatches without FV step.

**For all serial chains — pre-dispatch enforcement check:**

Before dispatching the FINAL agent in any serial chain, the orchestrator MUST run:
```
grep -nE "<predecessor-pin>|<predecessor-sha>" .factory/specs/<sibling-files>
```
and verify ZERO normative-current hits. If hits exist, dispatch the missing intermediate agent first.

**Canonical predecessor-pin grep for arch-bump chains:**
```
grep -nE "v1\.0\.(X-1)|<old-sha>" .factory/specs/prd.md | grep -v "§Trace\|§References\|Historical"
```
where `(X-1)` is the pre-bump arch version. The `grep -v §Trace|§References|Historical` exclusion filters historical-record citations that are correct to retain (they reference immutable past states). The normative-current sites must be zero.

### Companion META observation

SE-15e closes the orchestrator-side bypass of Extension 15. Prior Extensions 14/15/16/17 + sub-extensions codified spec-content and §Trace-evidence disciplines, but the meta-discipline of "orchestrator-driven serial cascade enforcement" was implicit. SE-15e makes it explicit.

Combined with Extension 17 (sweep-evidence pair-grep-command), the SERIAL Extension 15 META failure class is now structurally closed at **both** the agent-output level (Extension 17) **AND** the orchestrator-dispatch level (SE-15e).

**6th orchestration/discipline axis:**

- Extension 14 = within-layer propagation (lift_invariants_to_bcs sibling sites)
- Extension 15 = cross-layer parallel-dispatch (arch→PRD→VP serial cascade)
- Extension 16 = mandatory backfill sweep (codification-protocol non-backfill closed)
- SE-16a/b/c = in-burst-added citations + timestamp monotonicity + canonical grep target
- Extension 17 = grep-evidence command-pair discipline (SE-17a multi-line; SE-17b self-verification)
- **SE-15e = orchestrator SERIAL-cascade dispatch enforcement (this entry)**

**26 codified disciplines now in force** (was 25: Extensions 1-17 + SE-15a/b/c/d + SE-16a/b/c + SE-17a/b → + SE-15e = 26).

**Codified in:** cycle-001/lessons.md §SE-15e (this entry).

---

## SE-14b: Per-Probe BC-VP Coherence Discipline (Extension 14 Sub-Extension)

**Discovery:** R91 adversary (4-finding pattern I-R91-3/4/5/6 + 1 CRITICAL C-R91-1 + 1 HIGH I-R91-2). Attempt 24, D-047 strict pass 1. Cons R30 CLEAN.

**Root cause:** Newly added VP §Post-condition probes in VP v1.24 (I-R90-1: VP-DAEMON-006 pid integer + shutdown_reason enum + last_app_mode non-empty; I-R90-2: VP-DAEMON-001 semver regex; I-R90-4: VP-FACTORY-002 empty-string idempotency probe) introduced stronger constraints than the corresponding BC text specifies. This is the lift_invariants_to_bcs discipline applied REVERSE — BC text needs to be lifted UP to match VP's tightening.

**Extension 14 lineage:** Per Extension 14 codification body: "When a contract is lifted between tiers (e.g., EC → BC §Postcondition, NFR → BC, JC → BC), the lift MUST propagate to ALL summary-table sibling sites in the SAME architectural layer." SE-14b extends this discipline at PER-PROBE granularity:

### SE-14b Rule (codified 2026-05-15, R91 discovery)

**When a VP introduces a new §Post-condition probe that asserts a TYPED-FIELD constraint not yet specified in the corresponding BC text:**

**(a)** The probe is a VP-side tightening of a BC contract.

**(b)** The corresponding BC §Postcondition or §Invariant MUST be extended in the SAME burst (or the immediately preceding PO burst per Extension 15 SERIAL protocol) to specify the same constraint.

**(c)** If no BC element with the matching contract exists, the burst MUST include an Extension 14 BC lift (append a new BC §Postcondition or §Invariant naming the constraint).

**(d)** The VP's per-VP `Traces to:` field MUST cite the BC element that explicitly covers the matching constraint (NOT a non-existent EC or wrong subsection).

### Anchor Verification Sub-Rule (codified per C-R91-1)

Every VP §Post-condition that cites `(per BC-XXX §Postcondition N)` or `(per BC-XXX §Invariant N)` MUST resolve to a real existing element. The adversary's "Semantic Anchoring Audit" CRITICAL severity threshold applies.

**Coverage map regression prevention:** Before any FV burst lands a new VP probe, the FV agent MUST run:
```
grep -nA 5 "§Post-condition" .factory/specs/verification-properties.md | grep "per BC-"
```
to enumerate all anchor cites, then grep each cited BC element in the PRD to verify existence. Fabricated or wrong-subsection anchors are CRITICAL defects.

**SE-14b applies at the FV agent scope:** When the FV agent introduces new VP §Post-condition probes (either via probe-matrix expansion bursts like I-R90-1/2/4 or via fix bursts), the agent MUST:
1. Identify the corresponding BC element for each new probe.
2. Check that the BC text explicitly specifies the typed constraint (not just the concept).
3. If BC text does not specify the typed constraint: either (a) route to PO for BC lift first (preferred), or (b) include a BC lift in the same burst (if FV is authorized to write PRD in that burst context).

### Discipline-Axis Lineage

- Extension 14 = within-layer propagation (EC→BC lift propagates to summary table siblings)
- **SE-14b = per-probe granularity at BC-VP coherence axis** (VP tightening requires BC-text lift to match VP precision)

SE-14b is a sub-extension of Extension 14 in the same discipline family: both address the invariant that BC and VP must agree at the constraint level, not just at the conceptual level. Extension 14 addresses FORWARD lift (EC→BC); SE-14b addresses the REVERSE coherence check (VP precision → BC text equivalence).

### Finding Summary (R91 pattern)

| Finding | Site | VP Constraint | BC Gap |
|---------|------|---------------|--------|
| C-R91-1 | VP-FACTORY-002 §Post-cond 7 | idempotency invariant | Fabricated anchor `BC-FACTORY-002 EC "idempotency invariant"` — does not exist |
| I-R91-2 | VP-DAEMON-006 §Post-cond 10 | shutdown_reason enum | Cites BC-DAEMON-006 §Postcondition 1 (wrong — is §Invariant 1) |
| I-R91-3 | VP-DAEMON-006 §Post-cond 9 | pid ≥ 1 integer | BC-DAEMON-006 §Postcondition 2 says "exposed" but not "≥ 1" |
| I-R91-4 | VP-DAEMON-006 §Post-cond 11 | last_app_mode non-empty string | BC-DAEMON-006 §Postcondition 3 implies but does not specify "non-empty" |
| I-R91-5 | VP-DAEMON-001 §Post-cond 7 | semver regex `^(0\|[1-9]\d*)\.(...)$` | BC-DAEMON-001 §Invariant 2 says "semantic versioning format" without regex |
| I-R91-6 | VP-DAEMON-001 §Post-cond 8 | shutdown_reason 4-value enum | BC-DAEMON-001 §Postcondition 3 says "valid shutdown reason string" without enumeration |

All 6 sites were introduced by the F-R90 serial fix-burst VP v1.24 (I-R90-1/2/4 closures). The root cause is the same: FV added VP probe precision without simultaneously lifting BC text to match.

**27 codified disciplines now in force** (was 26: Extensions 1-17 + SE-15a/b/c/d + SE-16a/b/c + SE-17a/b + SE-15e → + SE-14b = 27).

**Codified in:** cycle-001/lessons.md §SE-14b (this entry).

**Trigger:** C-R90-1 — 5th recurrence of SERIAL Extension 15 propagation META; R90 adversary pass 1 attempt 23; F-R89 serial chain skipped PO step; 20+ normative-current PRD sites cite stale arch v1.0.18 (61a0064) when current arch is v1.0.19 (8a68cc9).

**25 codified disciplines now in force** (was 24 + Extension 17 + SE-17a + SE-17b).

---

### SE-14b Extension: AUTHORING-Discipline Mandatory Sub-Rule (R92 discovery)

**Discovery:** I-R92-5 (R92 adversary, attempt 25, D-047 strict pass 1).

**Root cause:** The F-R91 PO burst (PRD v1.19 2e24e09) lifted 5 BC content items: BC-DAEMON-001 Postcondition 1 (semver regex), BC-DAEMON-002 Postcondition 1 (pid≥1 + semver regex), BC-DAEMON-006 Invariant 1 (4 field constraints: app_mode last_app_mode shutdown_reason version), BC-FACTORY-002 EC-061 (empty-string current_cycle), EC-030 (trait-level rewording). The FV burst (VP v1.25 ba8eea4) was expected per SE-14b to add NEW BC-anchor citations to the corresponding 5 VP probes. Only 2 of 5 sites received new citations:

- VP-DAEMON-006 §Post-10 → BC-DAEMON-006 §Invariant 1: **ADDED** (correct)
- VP-FACTORY-002 §Post-7 → EC-061: **ADDED** (correct)

3 sites did NOT receive new citations:
- VP-DAEMON-001 §Post-7 semver: **MISSING** `per BC-DAEMON-001 §Postcondition 1`
- VP-DAEMON-002 §Post-7 numeric pid: **MISSING** `per BC-DAEMON-002 §Postcondition 1`
- VP-DAEMON-002 §Post-8 string-format: **MISSING** `per BC-DAEMON-002 §Postcondition 1`

**Structural diagnosis:** SE-14b was codified as VERIFICATION discipline (audit existing citations for anchor resolution) but NOT as AUTHORING discipline (mandate new citations when BC lifts happen). The FV agent applied SE-14b's verification sub-rule correctly (ran anchor-existence grep, confirmed all existing cites resolve) but had no authoring sub-rule to follow for identifying VP probes that needed NEW citations due to BC content lifts.

### SE-14b AUTHORING Sub-Rule (MANDATORY — extends SE-14b, codified 2026-05-15 R92 discovery)

**When a serial fix-burst chain includes BC content lifts (PO burst extending BC §Postcondition, §Invariant, or §Edge Case), the subsequent FV burst MUST:**

**(a)** Identify every NEW BC element introduced by the PO burst, using:
- The PO §Trace narrative (which lists what was changed), AND
- A targeted grep against the PRD comparing the new version against the previous version's known content.

**(b)** For each NEW BC element, identify the corresponding VP §Post-condition probe(s) that test the same constraint. Use:
```
grep -n "per BC-<XXX>" .factory/specs/verification-properties.md
grep -nE "§Post-condition [0-9]+" .factory/specs/verification-properties.md | grep -A2 "<constraint-keyword>"
```

**(c)** For each identified VP probe that tests the lifted BC constraint, ADD the `per BC-XXX §<Section> N` anchor citation in the VP probe text. This is the AUTHORING step — adding new citations, not just verifying existing ones.

**(d)** The §Trace v1.Y MUST embed an explicit `### SE-14b authoring audit` section enumerating:
- NEW BC elements (from PO burst) — list each BC ID + element type + new content
- MATCHING VP probes — list each VP-ID + §Post-condition N that tests the same constraint
- ANCHOR CITATIONS ADDED — list each new `per BC-XXX §Section N` citation added, OR state "no matching VP probe exists; future-Phase" with explicit rationale

### Combined SE-14b Rule (verification + authoring — BOTH mandatory)

SE-14b is now a TWO-PART discipline:

**Part 1 (VERIFICATION — codified b15effb, R91):** Every VP §Post-condition that cites `(per BC-XXX §Postcondition N)` or `(per BC-XXX §Invariant N)` MUST resolve to a real existing element. Anchor-existence grep required pre-commit.

**Part 2 (AUTHORING — codified via SE-14b extension, R92):** When BC content is freshly lifted (PO burst), identify VP probes that test the lifted constraint and ADD new BC-anchor citations. §Trace v1.Y MUST include explicit `### SE-14b authoring audit` section.

Without BOTH parts, the BC-VP coherence axis is asymmetric — BC tightens but VP doesn't trace it (authoring gap) OR VP cites non-existent anchors (verification gap).

### Discipline-Axis Extension

This extension makes SE-14b BIDIRECTIONAL at the authoring level:
- BC lift → VP citation REQUIRED (authoring sub-rule, NEW)
- VP cite → BC element MUST EXIST (verification sub-rule, prior codification b15effb)

**Codified in:** cycle-001/lessons.md §SE-14b extension (this entry, R92 discovery).

**27 codified disciplines remain in force** — SE-14b extension is a sub-rule of existing SE-14b, not a new numbered discipline.

---

## SE-17c: Final-state line-number revalidation (Extension 17 sub-extension)

**Discovery:** R95 adversary (attempt 28) surfaced a 3-finding pattern (C-R95-1 + C-R95-2 + C-R95-4) — VP v1.28 §Trace contained stale audit metadata: 7 of 10 L-number citations in the SE-16c++ supplementary grep audit-table were off-by-1 to off-by-17 from actual file state; the awk boundary `$1 < 3086` was hardcoded but the actual §Trace heading was at line 3108 (22-line drift); the frontmatter "8 sites" count claim mismatched the audit-table's 10 rows. Pattern cause: audit metadata was authored mid-burst (against partial-edit state) and not re-validated against final post-burst file state.

**Rule:** Every §Trace audit-table or audit-claim that cites line numbers, boundaries, or counts MUST be re-validated by re-running the cited grep/awk command as the LAST step of the burst, AFTER all other edits (including the §Trace narrative's own additions) are settled. Specifically:

- **(a) Line-number revalidation:** every `L<N>` citation in audit-table prose MUST be verified by Read of actual line N against the current file state.
- **(b) Boundary derivation:** every awk boundary filter `$1 < <BOUNDARY>` MUST derive `<BOUNDARY>` at burst-finalization time via `grep -n "^## §Trace" file.md | head -1 | cut -d: -f1`, NOT hardcode from a prior burst.
- **(c) Count consistency:** every frontmatter narrative count claim ("N sites surfaced") MUST equal the audit-table row count in the body.

**SE-17c application order:**
1. Author all body content + §Trace narrative.
2. Run all final-state greps with current line numbers.
3. Update audit-table L-numbers + awk boundary + counts to match final-state.
4. Re-verify all claims against the final-state.
5. Commit.

This is the third META audit-discipline codification under Extension 17 (after SE-17a multi-line patterns, SE-17b self-verification). The discipline class matures via observed defect patterns: C-R95-1 (L-numbers), C-R95-2 (awk boundary), C-R95-4 (count claim) are three distinct forms of the same root cause (mid-burst authoring without final-state revalidation).

**Context:** Cons R34 was CLEAN. Counter stays at 0/3. C-R95-3 MED (VP line 910 stale `per C-R91-1` attribution) is a separate finding class from the SE-17c cluster. I-R95-1 (PRD v1.20/v1.21 dual-version pattern) is informational. arch↔PRD round-trip secondary lens: all 11 sampled BCs resolve CLEAN — strong empirical signal that substantive content is converged; remaining defects are exclusively META audit-discipline.

**Codified in:** cycle-001/lessons.md §SE-17c (this entry, R95 discovery).

---

## SE-17c-d: Body-scope grep convention (Extension 17 sub-extension)

**Discovery:** I-R96-2 (R96 adversary, attempt 29) — F-R95 FV burst applied SE-17c first application, embedded a Step 2 final-state grep `grep -n "PRD v1\.20/21" file` and claimed "(no hits — Fix 5 closed I-R95-1)". But the §Trace narrative ITSELF quotes the searched pattern `PRD v1\.20/21` as evidence (in Fix 5 heading, in pre-burst grep transcript, in SE-16a in-burst audit, in dual-version simplification narrative). The final-state grep returns 4 §Trace-narrative hits, contradicting the "(no hits)" claim.

The SE-17c discipline correctly mandated "re-run final-state greps" but did NOT scope the greps to pre-§Trace body content. The §Trace narrative is ITSELF post-burst final state, and it correctly quotes pre-burst evidence — but those quotes show up as hits to the literal final-state grep.

**Rule:** Every SE-17c Step 2 final-state grep that asserts "no hits" or "N hits" for a string-class fix MUST be scoped to pre-§Trace body content via:

```bash
$ BOUNDARY=$(grep -n "^## §Trace" file.md | head -1 | cut -d: -f1)
$ grep -n "<PATTERN>" file.md | awk -F: -v B="$BOUNDARY" '$1 < B && $1 != 25'
```

where `<DERIVED_BOUNDARY>` is computed via `grep -n "^## §Trace" file.md | head -1 | cut -d: -f1` (per SE-17c-b boundary derivation) and `$1 != 25` excludes frontmatter narrative line.

The §Trace narrative may legitimately quote the searched pattern as PG-5 historical evidence — those quotes are NOT defects. They simply must be excluded from the "no-hits" count.

**Alternative formulation:** if a fix INTENDS to leave §Trace narrative quotes in place (per PG-5 framing), the post-burst grep should report "N hits in pre-§Trace body; M hits in §Trace narrative-evidence blocks per PG-5" rather than asserting "(no hits)".

This is the 4th sub-rule of Extension 17 (after SE-17a multi-line patterns, SE-17b self-verification, SE-17c final-state revalidation). It refines SE-17c by adding scope-discipline.

**CRITICAL META-IRONY:** R96 demonstrates third-order META-irony: the first application of META-discipline N+1 introduces a META-N+2 gap. The pattern is genuinely asymptotic at the META-discipline-codification layer. However: R96 secondary lenses (substantive content — cross-property + glossary + coverage matrix + triple pin coherence) ALL PASS, indicating substantive content layer is structurally converged. The remaining defects are exclusively META audit-narrative consistency.

**29 codified disciplines in force** after SE-17c-d (was 28 + SE-17c-d as 4th sub-rule of Extension 17).

**Codified in:** cycle-001/lessons.md §SE-17c-d (this entry, R96 discovery).

**28 codified disciplines now in force** (was 27 + SE-17c). SE-17c is a sub-extension of Extension 17, not a new numbered discipline.

---

## SE-18: Commit-Burst Hygiene (CODIFIED — 2026-05-18)

**Discovery:** 3rd occurrence per D-114 Goodhart's law trigger threshold. Three prior recurrences:

1. **Round 3 commit 932f4e0** (F-R105 closure chain): commit co-mingled FV T-128k work + architect T-128m ADR-0005 dual-accept into a single commit; sandbox-denied `git reset HEAD~1` left co-mingling unrecoverable without force.
2. **Round 6 commit d92e4a7** (F-R107 Round 6A+6B): PO 6A + PO 6B work co-mingled in a single commit; worktree state was already at staging boundary when discovered; recovered by proceeding rather than splitting.
3. **Round 7 cross-dispatch coordination failure**: Architect 7C edited SS doc content (F-R108-1 "current canonical" removal), which bumped SS document versions (v1.0.31→v1.0.32, v1.2.11→v1.2.13, v1.1.18→v1.1.20, v1.2.16→v1.2.17, v1.0.6→v1.0.7 dtu-assessment). PO 7B and FV 7D were dispatched with pin targets citing the pre-bump versions because the coordination directive did not anticipate that "content correction" in Architect 7C would cascade into version bumps. Result: 10 BC arch-source rows + PRD traces_to + brief line 247 + 22 VPs + VP-INDEX SS pins all cite stale pre-bump versions. Carryforward for R109 to surface and Round 8 to fix.

**Discipline:**

Before any parallel-dispatch round, the orchestrator MUST:

1. **Identify cross-dispatch artifact-version dependencies.** For each dispatch scope, ask: "Does this dispatch's edit bump a version that another dispatch in the same round cites?"

2. **If dispatch A's edit bumps an artifact version that dispatch B cites, EITHER:**
   - **(a) Serialize A → B** with explicit version target in B's prompt (B reads A's output and uses the post-bump version), OR
   - **(b) Use "commit pending — dispatch A" placeholder convention** with mandatory SM resolution pass that sweeps and corrects all stale citations before committing.

3. **SM closure burst MUST sweep all "commit pending" placeholders** in the artifact set and verify all cross-dispatch pins before committing. The sweep uses the Defensive Sweep Discipline (S-7.02): grep for the old version string across STATE.md, all INDEX files, all prd-supplements, and all BCs/VPs that cite the bumped artifacts.

4. **Architect MUST surface any anticipated version bump** from "content-correcting" work (including §Trace fixups, fabrication removals, schema corrections) BEFORE accepting dispatch scope. Version bumps from content corrections are NOT zero-impact — they cascade to every artifact that cites the bumped document.

**Application:** This discipline applies to all future Round-N closure chains. Each dispatch report MUST include a section: "Cross-dispatch version-bump dependencies identified: [list or NONE]."

**Related codification candidates (held per D-114 — not yet at 3-occurrence threshold):**
- O-R107-1 (input-hash path-existence hook): 2nd occurrence — needs 1 more.
- O-R107-2 (semantic anchor existence check): 2nd occurrence — needs 1 more.
- O-R108-3 ("current canonical" §Trace fabrication): 2nd occurrence as process-gap (but manifested as 2× CRIT findings F-R107-1 + F-R108-1) — approaching threshold.

**O-R108-1 (commit-pending resolution mechanism):** This SE-18 codification absorbs O-R108-1 as its Step 3 (SM closure burst sweeps "commit pending" placeholders). The O-R108-1 pattern (commit-pending placeholders not resolved before SM burst commit) is a specific instance of SE-18 commit-burst hygiene — the root cause is the same cross-dispatch coordination failure that leaves stale pins unresolved at commit time.

**34 codified disciplines now in force** (was 33 + SE-18).

**Codified in:** cycle-001/lessons.md §SE-18 (this entry, Round 7 cross-dispatch coordination failure — 2026-05-18 SM closure burst v5.68).

---

## SE-19: NFR-VP Phantom-Anchor Audit (CODIFIED — 2026-05-18)

**Discovery:** 3rd occurrence per D-114 Goodhart's law trigger threshold (3-occurrence rule). Three occurrences:

1. **R107 (F-R107-phantom-nfr):** NFR-007, NFR-008, NFR-011 in nfr-catalog.md cited VP files by number (VP-NNN) in their Verification column. Adversary R107 found that the cited VPs did not contain probes actually covering the NFR's stated latency/performance/capability requirement — only that VP-NNN existed. Classified as phantom-anchor finding.

2. **R109 (F-R109-phantom-anchor):** NFR-001 and NFR-002 cited VP-001 and VP-002 respectively in nfr-catalog.md Verification column. Adversary R109 found that while VP-001 and VP-002 exist, neither VP contained a probe row asserting the specific NFR-001/002 startup-latency and permission-overlay-latency budget requirements. The VPs had generic daemon-start probes but not NFR-specific latency-budget probes.

3. **R110 (F-R110-phantom-anchor):** Same NFR-001/002 pattern recurred in Round 10 audit — Round 9 closure had not yet been verified. This 3rd occurrence triggered D-114 codification threshold.

**Discipline (SE-19):**

Every NFR row in nfr-catalog.md that cites `VP-NNN` in its Verification column MUST be verified by adversary (and by FV on each nfr-catalog touch) to confirm that:

1. **VP-NNN exists** on disk (not a phantom file reference).
2. **VP-NNN contains a probe row** whose assertion text covers the NFR's stated behavioral requirement — not merely a generic probe for the same system component.
3. **The probe's pass criterion** is traceable to the specific metric or threshold stated in the NFR (e.g., if NFR-001 says "startup within 2 seconds," VP-NNN must have a probe with pass criterion `startup_elapsed_ms < 2000` or equivalent).

The adversary's NFR-VP anchor check is NOT satisfied by confirming VP-NNN exists. It requires confirming that VP-NNN contains a semantically covering probe for the NFR's stated requirement.

**Adversary audit recipe:**

```bash
# For each NFR row citing VP-NNN in nfr-catalog.md:
# 1. Confirm VP file exists:
ls .factory/specs/verification-properties/VP-NNN-*.md
# 2. Extract NFR requirement text; grep VP file for coverage:
grep -n "<key_metric_from_NFR>" .factory/specs/verification-properties/VP-NNN-*.md
# If grep returns 0 hits: PHANTOM ANCHOR — cite as finding.
```

**Application:** This discipline applies to every adversary pass AND to every FV dispatch that touches nfr-catalog.md. FV must include an explicit SE-19 compliance attestation in its dispatch report: "SE-19 NFR-VP anchor check: [list of NFR→VP pairs verified with grep evidence]."

**Related observations held per D-114:**
- O-R109-C: phantom NFR-VP anchor — this SE-19 codification absorbs O-R109-C as its formal rule.

**35 codified disciplines now in force after SE-19** (was 34 + SE-19 = 35; SE-20 follows in the same burst, bringing total to 36).

**Codified in:** cycle-001/lessons.md §SE-19 (this entry, 3rd occurrence NFR-001/002 R107+R109+R110 — 2026-05-18 SM closure burst v5.71 D-134).

---

## SE-20: Timestamp Monotonicity Hook (CODIFIED — 2026-05-18)

**Discovery:** 1st occurrence triggering codification as defensive measure (Production-Grade Default Rule 1 — not MVP-defer). R110 adversary surfaced a systematic SE-16d violation in the Round 8 closure chain: Round 8 commits (159d123 Arch 8A, 3334fb6 PO 8B, 72d863e FV 8C, 5ee763d SM 8D) all used timestamps rooted at the real-clock date 2026-05-17 (commit author's local date) rather than the chain high-water date 2026-05-18 (established by STATE v5.69 at 2026-05-18T03:00:00Z). This caused at least 4 §Trace timestamps in Round 8 artifacts to violate cross-artifact monotonicity — they pre-dated the chain's established high-water mark.

**Root cause:** SE-16d requires cross-artifact timestamp monotonicity but does not specify a hook-time enforcement mechanism. Agents authoring §Trace entries during a closure burst use the real-clock date unless explicitly instructed otherwise. When a burst spans midnight UTC or when the agent's local clock date differs from the chain high-water date, silent violations occur.

**Discipline (SE-20):**

Every §Trace timestamp in any spec artifact MUST satisfy:

1. **Intra-artifact monotonicity (existing SE-16b):** Each §Trace entry timestamp > all prior §Trace entry timestamps in the same artifact.

2. **Cross-artifact chain-time monotonicity (existing SE-16d):** Each artifact's newest §Trace entry timestamp >= all prior-burst artifact timestamps in the same closure chain.

3. **Chain high-water floor (NEW — SE-20):** The chain high-water is defined as `max(STATE.md frontmatter timestamp at burst start, all prior-burst artifact §Trace timestamps)`. Every §Trace entry authored in a closure burst MUST have timestamp >= chain high-water floor, regardless of the authoring agent's real-clock date.

**Hook-time enforcement (proposed):**

Add to `.claude/settings.json` hooks a `validate-timestamp-monotonicity` pre-commit hook that:

```bash
# For each modified spec file with §Trace section:
# 1. Extract newest §Trace timestamp from the file.
# 2. Extract chain high-water from STATE.md frontmatter timestamp field.
# 3. If file_newest_trace_ts < chain_high_water: REJECT commit with message:
#    "SE-20 VIOLATION: <file> §Trace timestamp <X> < chain high-water <Y>.
#     Bump §Trace timestamp to >= <Y> before committing."
```

Until the hook is implemented, the SM closure burst MUST include an explicit SE-20 compliance check: read STATE.md frontmatter timestamp (chain high-water), then verify that every §Trace timestamp in every artifact touched in the current burst is >= that value.

**Application:** This discipline applies to every closure burst. The SM must record the SE-20 cross-chain monotonicity matrix in the Session Resume Checkpoint (as SE-16d is already recorded) — specifically noting the chain high-water value and confirming all burst artifacts are >= it.

**Codification rationale:** R110 demonstrated that SE-16d is easy to violate silently when real-clock date diverges from chain high-water. A single-occurrence codification is justified under CLAUDE.md Production-Grade Default Rule 1 ("no MVP-driven deferrals") — this is not a "wait for 3 occurrences" case; it is a structural enforcement gap that will recur systematically until the hook exists. Codifying at 1st occurrence is the production-grade default.

**36 codified disciplines now in force** (was 35 after SE-19 + SE-20 = 36 total. SE-18 was 34th; SE-19 is 35th; SE-20 is 36th).

**Codified in:** cycle-001/lessons.md §SE-20 (this entry, R110 Round 8 timestamp pathology — 2026-05-18 SM closure burst v5.71 D-134).

---

## Critical Hook Lessons (Archived from STATE.md v6.01 on 2026-05-26 during compaction)

**Source:** STATE.md lines 947–1034 (v6.01), extracted during STATE.md v6.02 compaction.

### Human gate decision recorded

**OPTION (a) — Continue strict D-047** (recorded D-101; R98-R104 cycle completed under this directive, achieving D-047 strict convergence at D-120 against monolithic structure; convergence retired per D-122; new cycle underway against restructured artifacts). **OPTION A full closure chain** selected for new session per D-127 (context-clear durable checkpoint).

### Hook integration and process lessons

- `validate-input-hash`: update frontmatter to `[live-state]` BEFORE editing a cycle file with computed hash
- `block-ai-attribution`: rejects "Co-Authored-By: Claude" + robot emoji in commits
- Use `git commit -F /tmp/<file>` for messages over 2KB
- FC items LOCKED: read SS-core-types-and-abi.md; do NOT re-derive
- 4-entry permanent META catalog (D-054) is frozen — do NOT add to it during Phase 1+
- Phase 1+ adversarial review reverts to D-047 strict (0 findings x 3 consecutive passes)
- 18+ codified META rules in SS-conventions-anti-patterns.md v1.28 guard Phase 1 work
- **Orchestrator dispatch-prompt scope verification (codified per F-R62-1 META-finding):** When authoring dispatch prompts that enumerate BC/VP/story scope, the orchestrator MUST verify the enumeration against the architecture's source-of-truth section (e.g., §Behavioral Contract Summary) rather than copying from secondary references like STATE.md task queues. STATE.md scope claims can drift from architecture; the architect's §Behavioral Contract Summary is the authoritative count. Codified after F-R62-1 (16-BC dispatch under-enumerated the architect's prescribed 22-BC PRD scope).
- **Partial-fix regression discipline (codified per F-R63 META-findings):** When a fix-burst reconciles cross-artifact drift (e.g., test paths, names, taxonomies), the fix MUST propagate to ALL affected artifacts in scope — including the architecture if architecture cited the prior values. F-R62-4 reconciled PRD ↔ VP test paths but did NOT propagate to architecture (S-7.01 violation); F-R63 propagated arch v1.0.9 to close the loop. Codification: every fix-burst dispatch must include an explicit propagation checklist for ALL artifact dimensions touched (paths, names, taxonomies, versions, counts) AND a sweep step that verifies all sibling artifacts that might cite the changed values.
- **Semantic propagation sweep (codified per F-R65 finding cluster):** When a fix-burst changes a count, enumeration size, or taxonomy boundary, the propagation checklist MUST include SEMANTIC sweeps for: (a) prose lead-in count statements; (b) cross-paragraph consistency on changed elements; (c) related test vectors; (d) sibling sections that summarize the count. Metadata-only sweeps (paths, names, versions) are INSUFFICIENT. Codified after F-R65 "Three" count surviving 3 passes because F-R62-8 dispatch only enumerated metadata propagation.
- **Intra-block / intra-artifact same-ID consistency sweep (codified per F-R67 Obs-1):** When a fix-burst updates any element of a multi-section block (BC/VP §Mechanism, §Post-conditions, §Verification, §Edge Cases, etc.) OR a multi-instance ID (EC-NNN, BC-NNN, VP-NNN that appears in both catalog/index AND body prose), the propagation checklist MUST include intra-block AND intra-artifact same-ID consistency verification. Codified after F-R67-1 (VP-TYPES-001 §Mechanism vs §Post-conditions contradiction) and F-R67-2 (PRD §3 EC-045 vs §9 catalog row numeric mismatch).
- **Cross-platform + POSIX-convention sweep discipline (codified per F-R70-1 + F-R70-3):** Spec authors and review agents MUST verify (a) every dependency-crate platform-behavior claim against the crate's documented platform-specific behavior (`directories::runtime_dir()` returns None on macOS/Windows; OS-specific paths differ for cache/data/runtime dirs); (b) every signal-handling exit-code claim against POSIX 128+N convention; (c) every cross-platform-invariant claim against NFR platform targets. Codified after F-R70-1 (macOS runtime_dir() blocker) and F-R70-3 (exit code 130 misencoding SIGTERM as SIGINT origin).
- **Mandatory deps-pin-manifest sweep in formal-verifier dispatch templates (codified per Obs-R71-1 enforcement gap):** Every formal-verifier dispatch (and any agent doing pin propagation) MUST include a REAL grep sweep against SS-deps-pin-manifest.md as a mandatory checklist item BEFORE commit. Pattern: `grep -nE "\b(nix|libc|tower|directories|axum|tokio|prost|tempfile|tracing|temp-env|constant_time_eq|serde_json|serde_yaml_ng|rand|notify|interprocess|crossterm|ratatui|reqwest|russh|rmcp|nucleo|wasmtime|thiserror|anyhow)\s+[0-9]" .factory/specs/<artifact>.md`. For each match, classify against SS-deps-pin-manifest.md (current version). Document the classification table in the burst's §Trace entry. Converts L-F-R63 Extension 3 from codified-but-unenforced (R70→R71 gap) to codified-and-enforced. Application precedent: VP burst (commit 296b044) applied the discipline and caught 3 stale PRD v1.5/v1.6 annotations that would otherwise have been R72 findings.
- **Agent-id-routing-existence sweep (codified per Obs-R72-1):** Every `vsdd-factory:*` agent ID reference in any spec artifact (PRD, VP, arch, manifest, STATE.md, lessons.md, plans/) MUST resolve to an entry in CLAUDE.md Agent Routing Table. Codified after Obs-R72-1 caught VP §Scope citing non-existent `vsdd-factory:perf-check` (canonical: `vsdd-factory:performance-engineer`). Review agents include this as a sweep checklist item; orchestrator dispatch prompts to spec authors include "no-fabricated-agent-IDs" reminder. The authoritative routing table is §Agent Routing Table in `/Users/jmagady/Dev/monocle/CLAUDE.md`.
- **Placeholder-discipline covers `"..."` ellipsis pattern (codified per F-R74-1 / L-F-R63 Extension 4 expansion):** The placeholder-discipline sweep must cover BOTH `<X>` generic-placeholder forms AND `"..."` / `[..., "...", ...]` ellipsis-shorthand forms in array/JSON fields. F-R74-1 caught arch `GET /status` hook_endpoints using `[..., "...", ...]` (3-element ellipsis) instead of canonical 5-string enumeration. Extension 4 (schema-sketch precision propagation) now explicitly includes ellipsis pattern as a forbidden shorthand. All JSON schema sketches in arch must use actual values, not placeholders of any form.
- **VP-coverage-vs-BC-EC-security-property sweep (codified per Obs-R75-2 part 1 / L-F-R63 Extension 5):** For every BC EC-NNN that mandates a security-relevant property (mode bits, atomicity, constant-time, length, ordering), grep the VP catalog for a probe verifying that property. Missing probe = MEDIUM finding minimum (CRITICAL if privilege-related). Adversary and consistency-validator dispatch prompts include this as a mandatory sweep checklist item. F-R75-1 caught VP-DAEMON-005 missing 0o700 probe despite BC-DAEMON-005 EC-052 mandating it.
- **Rationale-prose-vs-NFR-canonical-contract sweep (codified per Obs-R75-2 part 2 / L-F-R63 Extension 6):** For every BC rationale prose that claims platform/scope/CI support, grep against NFRs/§Scope/brief for canonical contract evidence. Ungrounded claim = MEDIUM finding. F-R75-2 caught BC-DAEMON-005 rationale claiming Windows "zero-config" behavior — not grounded in NFR-008 (macOS+Linux CI scope only). Adversary and consistency-validator dispatch prompts include this as a mandatory sweep checklist item.
- **Automated crate-prefix grep audit + §Trace audit-row integrity (codified per F-R76 / Obs-R76-1 / L-F-R63 Extension 7):** Every fix-burst touching workspace dep graph or making deps-pin claims MUST (a) run automated grep `grep -oE '\b[a-z_][a-z_0-9]*::' arch.md` over all arch files and classify every result against the dep graph (PASS / N/A-stdlib / N/A-sub-module / MISSING); (b) include REAL grep evidence per audit row in §Trace (file:line citation of what was actually grepped, not self-attestation). Codified after F-R76 discovered fabricated §Trace PASS verdicts (`serde 1 cited verbatim` triple-false-positive) AND missed dep-graph edges (runtime→axum + runtime→chrono). Extension 7 IMMEDIATELY found a net-new gap on first application: chrono 0.4 was missing from manifest despite §Trace v1.0.14 using chrono::Utc::now().
- **NFR-to-VP exhaustive coverage audit + fabrication-pattern META class (codified per F-R77-3 / L-F-R63 Extension 8):** Every fresh-context adversarial pass MUST enumerate all NFR-NNN identifiers in PRD and verify each has either (a) VP coverage in catalog, or (b) explicit §Open-Gap entry with concrete future-attachment. §Open-Gap entries claiming a BC/VP "is independently verified by its own VP" or similar self-attestation MUST include grep-able evidence (file:line) of the cited VP's existence. False-positive §Open-Gap verification claims are HIGH-severity fabrication-pattern findings. The fabrication-pattern META class has now recurred at three axes: F-R76-1 §Trace audit-rows, F-R77-3 §Open-Gap entries, and potentially others. Recurrence-guard: every verification-claim cell in any audit table or gap entry MUST cite REAL grep evidence, not self-attestation.
- **§Coverage Matrix footer + closure-chain narrative audit (codified per F-R78-1 / L-F-R63 Extension 9):** Every fresh-context adversarial pass MUST perform a three-way consistency check between (1) §Coverage Matrix footer closure-chain narrative, (2) §Trace v(current) closure-chain narrative, and (3) §References item 1 historical lineage. Any divergence is a HIGH-severity fabrication-pattern finding. This is the FOURTH recurrence of the fabrication-pattern META class (F-R76-1 §Trace audit-row; F-R77-3 §Open-Gap; GAP-R17-001 §Trace closure-narrative; F-R78-1 §Coverage Matrix footer). The definitive recurrence guard is REAL grep evidence (file:line citation of independently-verified state) for EVERY claim cell in EVERY audit table — never accept self-attestation. Apply to ALL audit-table style claims: §Coverage Matrix, §Scope, §Purpose, §References lineage, §Mechanism Distribution, §Open Verification Gaps.
- **PRD §3 Verification → §7 RTM Test File propagation (codified per F-R79-1 / L-F-R63 Extension 10):** For every BC in PRD §3, the §3 §Verification test-file list MUST match the §7 RTM Test File column exactly. When any closure burst adds a test file to a BC's §3 Verification subsection, the §7 RTM Test File column MUST be updated atomically in the same burst. VP §Coverage Matrix is NOT a substitute for the RTM column — each document independently requires accurate test-file enumeration. Dispatch prompts for PRD fix-bursts MUST include a 22-row §3↔§7 RTM audit step. F-R79-1 caught BC-DAEMON-004 §7 RTM listing only `graceful_shutdown.rs` while VP §Coverage Matrix correctly listed both `graceful_shutdown.rs` + `daemon_lifecycle.rs`.
- **BC-vs-Brief/Vision JC-closure alignment audit (codified per F-R79-2 / L-F-R63 Extension 11):** Every VP-side normative description of an NFR or BC surface MUST cross-verify hook variant names against the canonical 5-endpoint matrix (BC-ENGINE-003 + BC-DAEMON-002). Gene-source variants (PostToolUse, PermissionRequest, etc.) appearing in normative-current VP prose MUST be either (a) explicitly marked as JC-N-OMITTED with PRD §1.5 + brief §Explicit Non-Goals citation, or (b) removed. Fabricated surfaces (NFR descriptions that claim hook coverage never scoped in the canonical 5-endpoint matrix) are HIGH-severity. Dispatch prompts for VP fix-bursts MUST include a PostToolUse/PermissionRequest audit step. F-R79-2 caught VP §G-6 NFR-002 description introducing `post-tool-use` surface contrary to JC-2-OMITTED.
- **VP-to-BC §Postcondition anchor audit (codified per F-R79-3 / L-F-R63 Extension 12):** Every VP §Post-condition with normative-tier narrative (defense-in-depth rationale, mutation-test rationale, counter-example sketch) MUST anchor to a BC §Postcondition or §Invariant in PRD, not EC-only. EC-NNN-only anchoring is insufficient for security-critical invariants. When a VP §Post-condition is added for a security property, the corresponding BC MUST have a matching §Postcondition entry. Dispatch prompts for VP security-property fix-bursts MUST verify that the BC has been lifted from EC-only anchoring to §Postcondition tier. F-R79-3 caught VP-DAEMON-005 Post-condition 9 anchored only to EC-052 — the 0o700 runtime-dir mode contract was a defense-in-depth security invariant requiring BC-DAEMON-005 Postcondition 8 lift.
- **Machine-greppable evidence requirement — META-discipline integrity (codified per F-R80 CRITICAL / L-F-R63 Extension 13):** EVERY audit-row claim across ALL L-F-R63 Extensions (Extension 3, Extension 7, Extension 8, Extension 9, Extension 11, Extension 12, PG-4, and any future extensions) MUST be backed by a code-block transcript of the ACTUAL grep command + ACTUAL output (file:line + matched text). NOT asserted PASS verdicts. NOT self-attested counts ("12 PASS verbatim") without evidence. NOT fabricated convention narratives. The §Trace block MUST include the grep transcript inline as a verifiable artifact that a downstream verifier or fresh-context adversary can re-run and observe the same output. F-R80 discovered prior bursts (v1.12, v1.13, v1.14) had fabricated Extension 3 audit-row PASS verdicts — axum 0.7 claimed "12 times PASS verbatim" but ZERO `axum 0.7` strings exist (canonical: axum 0.8); tower NOT in manifest at all; prost 0.13 vs prost 0.14; rand 0.9 vs rand =0.8.6 EXACT. ≥14 of ~30 rows had wrong manifest values. This is the deepest META-class finding: the recurrence-guard discipline had ITSELF become the fabrication site. The DEFINITIVE recurrence guard requires INFRASTRUCTURAL support (state-manager hooks that run the asserted greps and write verifiable evidence into §Trace as part of the commit pipeline; lint rules; runtime verification). Spec-side discipline narrative alone is necessary but NOT sufficient.
- **§Purpose SHA propagation — META recurrence guard (codified per F-R81-2/GAP-R20-001 / D-071):** §Purpose is a THIRD recurrence point for stale-SHA (R13-001 → GAP-R19-001 → F-R81-2). VP v1.16 adds §Purpose to the explicit propagation grep target list alongside §Trace and §References. Every fix-burst that bumps a PRD or arch version pin MUST explicitly grep §Purpose for the old SHA alongside §Trace and §References — 3-document sweep, not 2. Extension 13 evidence form is `grep -nE` (produces file:line + matched text) NOT `grep -cE` (count-only); count-only output fails Extension 13's verifiability requirement.
- **Extension 14 lift_invariants_to_bcs sibling-site propagation discipline (codified per F-R83-1 / D-072):** When a contract is lifted between tiers (EC → BC §Postcondition, NFR → BC, JC → BC), the lift MUST propagate to ALL sibling summary tables in the same architectural layer: (1) PRD §4 NFR table row, (2) arch §BC Summary footer row, (3) VP §Catalog Overview row, (4) VP §Auxiliary Mechanism Coverage row. EC-only or body-only lift without summary-table propagation is a HIGH-severity finding minimum. Sub-extension (from F-R83-2): §References intro current-as-of timestamp is a FOURTH "current-as-of" recurrence guard target alongside §Purpose SHA, §Trace version citations, and §References lineage SHA. Every fix-burst that bumps a frontmatter timestamp MUST sweep all four targets. F-R83-1 caught 4-site propagation gap after F-R79-3 BC-DAEMON-005 Postcondition 8 lift; F-R83-2 caught §References timestamp 3-hour drift after F-R81+GAP-R20 burst. PRD v1.13 (dcae9d5) + arch v1.0.17 (a798d51) + VP v1.17 (1d21fd0) applied the discipline. [SE-15a EXTENDS Extension 14 VP-layer enumeration: add per-VP §Mechanism block as FOURTH VP-layer propagation target.]
- **Extension 15 cross-layer parallel-dispatch coordination (codified per F-R84-1 CRITICAL / D-073):** ORCHESTRATOR-level discipline: (1) Before parallel-dispatching sibling-layer bursts, orchestrator MUST determine whether ANY burst will bump an artifact's version. If yes, that agent is dispatched FIRST (serial cascade); sibling agents dispatched only AFTER the version-bump burst lands, with new version pin in their prompt. (2) Serial cascade for multi-bump fix-bursts: serialize in dependency order (arch → PRD → VP). (3) Default to serial for ambiguous bursts — parallel is the exception, not the default. (4) Parallel-safe bursts only when ALL sibling agents do in-place edits with NO version bumps; even then, agents must re-read sibling frontmatter at burst-end. SE-15a: Extension 14 VP-layer enumeration extended to include per-VP §Mechanism block (FOURTH VP-layer target). SE-15b: Extension 13 evidence requirement retroactively inherited by ALL propagation-sweep extensions (including Ext14) — every sweep MUST emit grep -nE transcripts before + after for each propagation target. SE-15c: Convention back-propagation — when a new citation form is established on one row of a homogeneous table, sweep ALL sibling rows for the same convention. SE-15d: Cross-property VP reciprocity — when VP-A cites VP-B in §Mechanism cross-property, VP-B MUST cite VP-A back; sweep at codification time. F-R84-1 CRITICAL caught ~93 stale arch-pin sites across PRD v1.13 + VP v1.17 from F-R83 parallel-dispatch anti-pattern (architect bumped v1.0.16 → v1.0.17 mid-parallel-burst; PO and FV completed with stale pin). This is the first ORCHESTRATOR-behavior discipline (all prior extensions addressed spec-content rules).
- **F-R84 serial fix-burst — Extension 15 + SE-15a/b/c/d PROVEN effective (per D-074 / 2026-05-15):** F-R84 serial fix-burst (SM v5.19 8d8ec3d → PO PRD v1.14 4997354 → FV VP v1.18 6915b5d) is the first fix-burst to apply Extension 15 serial protocol end-to-end. All 7 substantive findings + 3 observations closed with zero parallel-dispatch anti-pattern recurrence. Critical lesson: SE-15b (real grep transcript discipline) was PROVEN effective when explicitly applied at burst time — §Purpose 4th recurrence (F-R84-3) was closed with actual grep output rather than asserted PASS verdicts. SE-15d (cross-property VP reciprocity) was applied at codification time: VP-AUTH-001 now explicitly reciprocates VP-DAEMON-005 §Mechanism cross-property citation. SE-15c (convention back-propagation) was applied: §7 RTM sibling rows swept for Requirement ID convention. Cumulative Extension 15 operationalization outcome: the orchestration-protocol defect axis (first identified at R84) is now structurally closed at the process-discipline level. Future bursts with multi-layer version bumps MUST follow the serial cascade; parallel dispatch is the exception requiring explicit pre-authorization.
- **Extension 16 codification-protocol mandatory backfill sweep (per R85 / 2026-05-15):** R85 adversary identified that SE-15c and SE-15d were each applied only to their trigger site without backfill sweep of pre-existing instances — producing F-R85-IMP-2 (NFR-004/005/010 missing SE-15c convention) and F-R85-IMP-3 (4+ cross-property pairs missing SE-15d reciprocity). L-F-R63 Extension 16 closes this at the codification-protocol level: every new Extension/SE codification MUST trigger a mandatory backfill sweep of all pre-existing sites in the same artifact class, with SE-15b grep transcripts in a dedicated §Trace backfill section. Orchestrator dispatch prompts for trigger fix-bursts MUST explicitly enumerate backfill targets. State-manager logs adherence. This is the SECOND orchestration-protocol-axis discipline (after Extension 15 cross-layer parallel-dispatch) — the ratchet has now closed the codification-protocol non-backfill axis. Also: Obs-R85-2 adjudicated — SE-15d applies uniformly to BOTH `cross-property` AND `cross-check` citation forms.
- **F-R85 serial fix-burst — Extension 16 PROVEN effective (per D-076 / 2026-05-15):** F-R85 serial fix-burst (SM v5.21 32fb5ee → PO PRD v1.15 80bfe86 → FV VP v1.19 022ce3c) is the first fix-burst to apply Extension 16 mandatory backfill sweep end-to-end. All 4 substantive findings + 2 observations closed with zero codification-protocol non-backfill recurrence. Critical lesson: Extension 16 backfill sweep produced ADDITIONAL FINDING DISCOVERY beyond the trigger findings — 1 extra wrap-continuation site at line 568 was found during the IMP-1 backfill sweep that R85's original 5-site listing missed. This proves the backfill-sweep discipline is a finding-amplification tool, not merely a compliance checkbox. Cumulative Extension 16 operationalization outcome: the codification-protocol defect axis (first identified at R85) is now structurally closed at the process-discipline level. Every future codification burst MUST include mandatory backfill sweep with SE-15b grep transcripts.
- **SE-16a (Extension 16 sub-extension): In-burst-added citation discipline (per C-R86-1 / R86 / 2026-05-15):** Extension 16's mandatory backfill sweep rule covers "pre-existing sites" but did NOT cover citations added WITHIN the same burst. C-R86-1 caught VP-DAEMON-005 → VP-DAEMON-004 §Mech 5 unidirectional citation introduced by the F-R85 burst itself — described as a "reciprocation" in §Trace v1.19 but VP-DAEMON-004 §Mech 5 had no reciprocal citation. SE-16a amends Extension 16: the mandatory sweep MUST cover BOTH (a) pre-existing sites (original Extension 16 scope) AND (b) in-burst-added citations (NEW — SE-16a). Every burst that adds cross-property/cross-check citations MUST include a `### Burst-end re-audit: in-burst-added SE-15d citations` section in §Trace, with grep -nE transcripts verifying bidirectionality for each citation added in the burst. META-pattern META-3: "apply audit rule to pre-existing instances, not to in-burst-added instances" — closed by SE-16a. F-R86 FV burst (VP v1.20) is first application of SE-16a.
- **SE-16b (Extension 16 sub-extension): Frontmatter timestamp monotonicity guard (per O-R86-1 / R86 / 2026-05-15):** VP v1.18 → v1.19 timestamp regression (v1.18 was future-dated `2026-05-16T08:00:00Z`; v1.19 corrected to `2026-05-15T20:00:00Z`, which is earlier). This breaks monotonic ordering for tooling that sorts by frontmatter timestamp. SE-16b rule: every burst's frontmatter timestamp must be (a) monotonically increasing (`≥` predecessor), or (b) if a correction is required (predecessor was future-dated), the §Trace MUST include a `### Timestamp monotonicity correction` section explicitly stating: predecessor version, predecessor timestamp, predecessor-was-future-dated status, new timestamp, regression introduced, and justification. F-R86 FV burst (VP v1.20 §Trace) applies SE-16b to document the v1.18 future-date correction retrospectively.
- **F-R86 serial fix-burst — SE-16a first application CONFIRMED + backfill discipline finding-amplification (per D-078 / 2026-05-15):** F-R86 serial fix-burst (SM v5.23 7224e58 → PO PRD v1.16 cd6541f → FV VP v1.20 f94c499) is the first fix-burst to apply SE-16a (in-burst-added citation re-audit) end-to-end: burst-end re-audit section PASS (VP-DAEMON-004 §Mech 5 reciprocation confirmed bidirectional, Extension 16 full audit table 10 citations all bidirectional). SE-16b PASS (v1.20 timestamp 20:30:00Z ≥ v1.19 20:00:00Z). Critical finding: generic Per-PRD-v1.\d+ grep enhancement (backfill discipline improvement applied to I-R86-1) discovered 2 ADDITIONAL stale-pin sites (lines 2279 + 2369) beyond R86's adversary-listed line 1867. This continues the pattern established in F-R85 (1 additional wrap-continuation site) — the backfill discipline is EMPIRICALLY PROVEN as a finding-amplification tool. Rule: when an adversary identifies a stale-citation finding, the backfill grep MUST use a VERSION-AGNOSTIC pattern (e.g., `Per-PRD-v1.\d+` not `Per-PRD-v1.10`) to catch ALL sibling instances, not just the listed site. 23 codified disciplines remain in force; no new discipline from F-R86 itself (SE-16a + SE-16b were established at R86 and applied here for the first time).
- **SE-16c (Extension 16 sub-extension): Canonical Extension 16 audit grep target (per I-R87-1 / O-R87-2 / R87 / 2026-05-15):** R87 adversary discovered VP v1.20 Extension 16 audit table dropped the v1.19 pre-existing row for VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002 (META-4 third-order failure). Root cause: audit-table enumeration was manual (burst author re-enumerates every burst), making it vulnerable to row-dropping. SE-16c rule: the Extension 16 mandatory backfill sweep audit table MUST be generated by running a canonical grep target: `grep -nE "[Cc]ross-property|[Cc]ross-check" .factory/specs/verification-properties.md | grep -v "§Trace"`. The `grep -v "§Trace"` exclusion filters historical-snapshot citations in §Trace that do not reflect current body state. Every VP fix-burst that produces an Extension 16 audit table MUST include a `### SE-16c canonical grep transcript` section in §Trace showing the actual grep output used. This is the 4th orchestration/discipline-protocol axis sub-extension (SE-16a = in-burst-added; SE-16b = timestamp monotonicity; SE-16c = canonical grep target). META-4 closes the enumeration-mechanism axis: the discipline ratchet has closed audit-scope gaps at four levels: cross-session (Extension 13), cross-burst (Extension 16), intra-burst (SE-16a), and enumeration-mechanism (SE-16c). 24 codified disciplines in force.
- **Extension 17 sweep-evidence command-pair discipline (codified per F-R89-1 + O-R89-1 / L-F-R63 Extension 17 / 2026-05-15):** R89 adversary discovered VP v1.22 §Trace asserted "post-burst grep returns zero hits" for wrap-continuation `PRD\nv1.16` patterns — but actual file state has 6 hits at lines 275-276, 475-476, 639-640, 896-897, 1561-1562, 1753-1754. This is Extension 13 bypass via prose-asserted form. Extension 17 rule: every grep transcript in a §Trace forensic block MUST include (a) the literal grep command in code-fence AND (b) the actual output in code-fence. NEVER use "post-burst grep returns zero hits" without literal command. SE-17a: multi-line patterns must use `grep -P` with `\n` or `pcregrep -M` — single-line `grep -E` does NOT match cross-line patterns. SE-17b: before asserting ANY sweep-completion claim in §Trace, agent MUST run the command (literally) and paste the output into §Trace. F-R89-1 demonstrates Extension 13 codification was bypassable via prose-asserted form; Extension 17 adds the literal-command requirement to close the bypass. 25 codified disciplines in force (was 24 + Extension 17 + SE-17a + SE-17b).
- **SE-16c PROVEN effective at F-R87 fix-burst — first application confirms mechanical superiority over manual enumeration (per D-080 / 2026-05-15):** F-R87 FV-only fix-burst (VP v1.21 6ecb79a) is the first fix-burst to apply SE-16c end-to-end. Critical confirmation: SE-16c canonical-grep enumeration produced 39 rows vs v1.20 manual table's 10, surfacing the dropped v1.19 row 6 (VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002) PLUS PROSE continuations and recap rows that manual enumeration had missed entirely. This is the strongest empirical validation yet in the Extensions series: the discipline was applied for the first time, and it immediately surfaced a class of omissions (continuation-line rows, historical recap rows) that manual enumeration is structurally incapable of catching. Lesson: mechanical reproducibility > manual enumeration is NOT merely theoretical — it is an empirically observed finding-amplification differential of 3.9× (39 vs 10 rows) on first application. Every future VP burst MUST use SE-16c canonical grep for Extension 16 audit; manual enumeration is permanently retired for this purpose.
- **CONTENT-CENTRIC lens rotation produces structurally different finding-class than META-axis lens rotations (per R88 / D-081 / 2026-05-15):** R88 applied a CONTENT-CENTRIC lens (interrogating substantive spec correctness: API accuracy, forward-compat contracts, production-grade annotation pins, missing error codes, test-type labels) instead of the META-axis lens (audit-discipline, orchestration protocol, codification backfill) that dominated R83–R87. R88 found 5 substantive content defects that 22 prior META-axis passes missed ENTIRELY. This is the empirically confirmed lens-rotation gap: META-axis and CONTENT-axis lens rotations are not substitutes; they find structurally different defect classes. MANDATORY LENS-ROTATION DISCIPLINE (Extension 17 candidate): adversary dispatch prompts MUST alternate between META-axis and CONTENT-axis lenses to ensure both classes converge simultaneously. A sequence of N consecutive META-axis passes leaves content defects undetected. A sequence of N consecutive CONTENT-axis passes leaves META-discipline gaps undetected. The correct discipline is: after each pass, state the lens class used, and explicitly rotate lens class on the next dispatch. Historical R88 lens-class summary: R62–R82 = CONTENT-dominant (finding spec content defects); R83–R87 = META-dominant (finding protocol/discipline defects); R88 = CONTENT-CENTRIC (first explicit content rotation after 5-pass META streak). All R88 findings are novel to the CONTENT axis and were invisible to the prior META-axis streak.
- **F-R88 demonstrates CONTENT-CENTRIC lens rotation produces structurally different substantive findings than META-axis lens rotations; production-grade default fix-in-scope correctly applied via 4-agent serial chain (per D-082 / 2026-05-15):** F-R88 serial fix-burst (SM v5.27 0b217ad → architect arch v1.0.18 61a0064 → PO PRD v1.17 27e663c → FV VP v1.22 e4c1a1e) is the definitive proof that CONTENT-CENTRIC lens rotation finds defects invisible to META-axis passes: F-R88-5 (VP §Mechanism uniform "unit-test" mislabel for integration tests) had been latent across 21+ prior META-axis adversary passes. Production-grade default (fix in scope, not defer) was correctly applied — all 5 substantive content defects fixed in the same work cycle via specialist-routing chain. The 4-agent serial chain (state-manager bookkeeper + architect + product-owner + formal-verifier) demonstrates correct agent routing: each specialist fixed only their domain. Lesson for dispatch prompts: after a run of META-dominant passes, explicitly instruct adversary to rotate to CONTENT-CENTRIC lens before the next pass. Extension 17 codification (lens-rotation discipline) is a candidate for the R89 dispatch.

- **F-R89 Extension 17 first application PROVEN effective — fabricated grep evidence pattern structurally prevented (per D-084 / 2026-05-16):** F-R89 serial fix-burst (SM v5.29 752d391 → architect arch v1.0.19 8a68cc9 → FV VP v1.23 aef2f0c) demonstrates Extension 17 first application PROVEN effective. The Python re.MULTILINE evidence in §Trace v1.23 (replacing v1.22's prose-asserted form "post-burst grep returns zero hits") is mechanically re-runnable — the META-class fabricated grep evidence regression that R89 surfaced is now structurally prevented. SE-17a (multi-line pattern grep discipline via Python re.MULTILINE / pcregrep -M) and SE-17b (self-verification before §Trace assertion) are the structural preventions. F-R89-3 + F-R89-4 also represent first probe-matrix exhaustiveness expansions of the cycle — VP coverage now operationalized for previously-undertested typed fields (VP-RING-001 absence-of-field probe 1.d; VP-DAEMON-002 probes 7/8/9 for runtime_dir/config_path/protocol_version/lock_holder-present). Lesson: Extension 17's literal-command-pair requirement converts a prose-bypass surface (codified discipline stated but not executed) into a machine-verifiable artifact. Future VP bursts MUST pair every grep assertion with its literal command + output in §Trace; prose-asserted sweep claims are permanently retired per SE-17b.

- **F-R90 serial fix-burst — SE-15e first application PROVEN effective; SERIAL Extension 15 META failure class structurally closed at dual layer (per D-086 / 2026-05-16):** F-R90 serial fix-burst (SM v5.31 c9d77a9 + PO PRD v1.18 3a18306 + FV VP v1.24 63b75f9) is the first fix-burst to apply SE-15e (orchestrator SERIAL-cascade dispatch enforcement pre-dispatch predecessor-pin grep) end-to-end. Critical confirmation: the 5th recurrence of the SERIAL Extension 15 META failure (C-R90-1) is now structurally prevented at the orchestrator-dispatch level — combined with Extension 17 (agent-output level sweep-evidence discipline), the SERIAL Extension 15 META failure class has DUAL-LAYER structural enforcement. This is the first time any META failure class has achieved dual-layer enforcement in this project (codification + orchestrator-dispatch-level verification). Lesson: META failure recurrences follow a surface-closing pattern: Extension 15 closed the spec-content surface, SE-15a/b/c/d closed the sub-extension surfaces, and SE-15e closes the orchestrator-dispatch surface. No further recurrences at the SERIAL Extension 15 axis are architecturally possible given the pre-dispatch grep check. The 26 codified disciplines now represent a mature, battle-tested META defense stack built across 32 adversary passes and 9 fix-bursts.

- **SE-15e: Orchestrator SERIAL-cascade dispatch enforcement — 5th recurrence of SERIAL Extension 15 META failure closes the orchestrator-dispatch bypass (per C-R90-1 / D-085 / 2026-05-16):** R90 adversary (pass 1 attempt 23) found C-R90-1 CRITICAL: F-R89 serial chain dispatched `arch → FV`, skipping the mandatory PO PRD-pin propagation step between them. The orchestrator reasoning "PRD has no R89 findings; therefore no PRD burst needed" violates Extension 15 SERIAL cascade — the trigger for PO PRD dispatch is the arch version bump itself, not whether the adversary names PRD-side findings. This is the 5th recurrence of the SERIAL Extension 15 META failure (F-R84-1 / F-R85 / F-R88-1 / GAP-R23-001 / C-R90-1). SE-15e rule: before dispatching the FINAL agent in any serial chain, the orchestrator MUST run `grep -nE "<predecessor-pin>|<predecessor-sha>" .factory/specs/<sibling-files>` and verify ZERO normative-current hits; if hits exist, dispatch the missing intermediate agent first. For arch-bump chains specifically: `grep -nE "v1\.0\.(X-1)|<old-sha>" .factory/specs/prd.md | grep -v "§Trace\|§References\|Historical"`. Combined with Extension 17 (agent-output level sweep-evidence discipline), the SERIAL Extension 15 META failure class is now structurally closed at BOTH the agent-output level AND the orchestrator-dispatch level. 26 codified disciplines in force (was 25 + SE-15e). Full codification in `cycles/cycle-001/lessons.md §SE-15e`.

- **F-R91 serial fix-burst — SE-14b first application PROVEN effective; BC-VP coherence axis structurally closed (per D-088 / 2026-05-15):** F-R91 serial fix-burst (SM v5.33 b15effb → PO PRD v1.19 2e24e09 → FV VP v1.25 ba8eea4) demonstrates SE-14b first application PROVEN effective. The systematic 4-finding BC-stronger-than-VP pattern (I-R91-3/4/5/6) was closed via PO BC lifts in PRD v1.19; C-R91-1 CRITICAL (fabricated anchor `BC-FACTORY-002 EC "idempotency invariant"`) was closed via EC-061 creation (empty-string current_cycle) + VP re-anchor to EC-061. Anchor verification grep-target is now a per-burst FV checklist item (SE-14b sub-rule). Substantive content produced: BC-DAEMON-001/002 Postcondition 1 (pid≥1 + semver regex constraints), BC-DAEMON-006 Invariant 1 (4 explicit field constraints: app_mode last_app_mode shutdown_reason version), EC-061 (61 ECs total), 21 glossary terms (MONOCLE_RUNTIME_DIR + DaemonStartError::RuntimeDirUnresolvable). SE-16b monotonicity PASS throughout. §Purpose META 12th-attempt application. Critical lesson: the REVERSE direction of BC-VP coherence (VP precision → BC text lift) was the invisible gap axis for 34 attempts; SE-14b mechanically closes it by requiring the BC to match VP probe precision in the SAME burst (or immediately preceding PO burst per Extension 15 SERIAL). 27 codified disciplines remain in force; no new discipline from F-R91 itself (SE-14b was established at R91 and applied here for the first time).

- **SE-14b extension: AUTHORING-discipline mandatory sub-rule (per I-R92-5 / R92 / 2026-05-15):** R92 demonstrates that SE-14b codified as VERIFICATION discipline can fail at AUTHORING. The F-R91 PO burst lifted 5 BC content items; the FV burst (VP v1.25) correctly verified that existing anchor citations resolve (SE-14b verification PASS) but did NOT add new anchor citations for the newly-lifted BC elements (SE-14b authoring FAIL). 3 VP probes (VP-DAEMON-001 §Post-7 semver, VP-DAEMON-002 §Post-7 numeric, VP-DAEMON-002 §Post-8 string-format) were left without `per BC-XXX §Postcondition N` citations for the freshly-lifted BC content. SE-14b is now AUTHORING-mandatory: PO BC lifts MUST trigger FV to add new VP anchor citations in the same serial chain. §Trace v1.Y MUST embed explicit "SE-14b authoring audit" section enumerating: NEW BC elements, MATCHING VP probes, ANCHOR CITATIONS ADDED. Without authoring, the BC-VP coherence axis is asymmetric — BC tightens but VP doesn't trace it. Full extension text in cycles/cycle-001/lessons.md §SE-14b extension. 27 disciplines remain in force (SE-14b authoring is a sub-rule extension, not a new numbered discipline).

- **SE-14b AUTHORING first application PROVEN effective (per F-R92 / D-090 / 2026-05-15):** The SE-14b codification extension (b15effb + 68bf374) was applied for the first time in VP v1.26 (d423134). Result: 3 new BC-anchor citations added (VP-DAEMON-001 §Post-7 ← BC-DAEMON-001 Postcondition 1 semver regex; VP-DAEMON-002 §Post-7 ← BC-DAEMON-002 Postcondition 1 pid≥1; VP-DAEMON-002 §Post-8 ← BC-DAEMON-002 Postcondition 1 string-format last_app_mode). Combined verification + authoring discipline now operationally enforces BC-VP coherence in BOTH directions: (1) verification — VP BC-anchor cites must resolve (proven F-R91); (2) authoring — PO BC lifts must trigger FV anchor-citation additions in same serial chain (proven F-R92). The BC-VP coherence axis is now structurally closed at both application-time layers. No new discipline created (SE-14b authoring is a sub-rule extension, not a new numbered discipline). 27 disciplines remain in force.

- **C-R93-1 demonstrates that F-R88-5 §Mechanism Distribution discipline propagation was incomplete at the cross-artifact (PRD + arch) layer (per R93 / D-091 / 2026-05-15):** F-R92 closed only VP-internal sibling sites (4 §Harness location annotations). The pattern: a within-artifact partial-fix (VP-internal) leaves cross-artifact siblings (PRD §7 RTM + PRD §3 §Verification body + arch verification clauses) unfixed. Future propagation discipline applications MUST explicitly enumerate ALL cross-artifact sibling sites for the canonical taxonomy axis being fixed. For the unit-test/integration-test taxonomy axis specifically: the sibling sites are (1) VP §Mechanism Distribution block, (2) VP §Harness location annotations per-VP, (3) PRD §7 RTM Test Type column rows, (4) PRD §3 per-BC §Verification body type labels, (5) arch verification clause text. A fix is not complete until all 5 tiers are updated atomically in one serial burst. This pattern is an application failure of Extension 14 (lift_invariants_to_bcs sibling-site propagation) — the Extension 14 sibling-site enumeration must include cross-artifact tiers, not just within-VP summary tables. No new codification class needed — SE-14b authoring discipline + Extension 14 propagation discipline cover this case. The gap is application, not codification.

- **R94 surfaces an ORCHESTRATOR-induced defect (C-R94-1): the F-R93 dispatch prompt authored an imprecise doc-comment rationale. The orchestrator's dispatch language IS spec content when copied verbatim into agent output. Lesson: orchestrator prompts that include normative content text (rationale, doc-comment text, API claims) must apply the same Extension 14 propagation discipline + SE-14b verification as agent-authored content. Codify this in a future SE (SE-18 candidate) if recurrence. Currently a process-gap with no immediate codification (O-R94-2). (per C-R94-1 / D-093 / 2026-05-15)**

- **SE-17c: Final-state line-number revalidation discipline (codified per C-R95-1/2/4 / R95 / 2026-05-15):** R95 adversary (attempt 28) surfaced 3-finding same-class pattern: VP §Trace audit-table stale metadata — 7 of 10 L-number citations off-by-1 to off-by-17 from actual file state; awk boundary `$1 < 3086` hardcoded when actual §Trace heading was at line 3108 (22-line drift); frontmatter count claim "8 sites" mismatched audit-table 10 rows. Root cause: §Trace metadata authored mid-burst against partial-edit state, not re-validated against final post-burst file state. SE-17c rule (5-step application order): (1) Author all body content + §Trace narrative; (2) Run all final-state greps with current line numbers; (3) Update audit-table L-numbers + awk boundary + counts to match final-state; (4) Re-verify all claims; (5) Commit. Sub-rules: (a) L-number revalidation via Read of actual line N; (b) awk boundary derivation via `grep -n "^## §Trace" file.md | head -1 | cut -d: -f1` at burst-finalization time, NOT hardcoded; (c) frontmatter count == body row count. This is the third META audit-discipline sub-extension under Extension 17 (SE-17a multi-line grep patterns; SE-17b self-verification before assertion; SE-17c final-state revalidation). R95 secondary lens (arch↔PRD round-trip): ZERO substantive content defects on 11 sampled BCs — strong empirical convergence signal: substantive content layer is converged; META audit-discipline continues maturing. 28 codified disciplines in force (was 27 + SE-17c). Full codification in `cycles/cycle-001/lessons.md §SE-17c`.

- **SE-17c-d: Body-scope grep convention — SE-17c Step 2 final-state greps MUST be scoped to pre-§Trace body content (per I-R96-2 / R96 / 2026-05-15):** R96 adversary (attempt 29) found SE-17c first application in F-R95 embedded a `grep -n "PRD v1\.20/21"` final-state grep asserting "(no hits)" — but the §Trace narrative ITSELF quotes this pattern at 4 sites as PG-5 historical evidence. Root cause: SE-17c Step 2 ("run all final-state greps") did NOT scope greps to pre-§Trace body. SE-17c-d rule: every SE-17c Step 2 final-state grep asserting "no hits" MUST use `awk -F: -v B="$BOUNDARY" '$1 < B && $1 != 25'` scoped to pre-§Trace body. The §Trace narrative may legitimately quote the pattern as PG-5 evidence; those hits are NOT defects and MUST NOT be counted as "(no hits)" violations. **META-IRONY:** R96 demonstrates that the first application of META-discipline N+1 introduces a META-N+2 gap — genuine asymptotic convergence at the META-codification layer. R96 substantive secondary lenses ALL PASS (cross-property + glossary + coverage matrix + triple pin). 29 codified disciplines in force (was 28 + SE-17c-d as 4th sub-rule of Extension 17). Full codification in `cycles/cycle-001/lessons.md §SE-17c-d`.

- **META-asymptote pattern CONFIRMED at META-N+2 — SE-17c-d codification (29-discipline) FIRST application introduces SE-17a defects (per D-099 / R97 / 2026-05-15):** R97 adversary (D-047 strict pass 1 attempt 30 — META-asymptote test) confirmed the pattern empirically across 4 consecutive cycles. SE-17c-d was designed to enforce body-scope grep convention. Its FIRST application (VP v1.30 §Trace Fix 1 + Fix 2 transparency blocks) introduced 4 new SE-17a evidence-fidelity defects: (1) `<N>` placeholder unfilled in Fix 2; (2) Fix 1 "full-file grep" claims 8 of 16 actual hits; (3) Fix 1 evidence rows self-reference §Trace line numbers; (4) Fix 2 count "4 in-§Trace MED sites" stale post-Fix-1. Pattern confirmed: each codification of META-discipline N enables META-discipline N+1 defects on FIRST application (SE-17c → SE-17c-d gap R96; SE-17c-d → SE-17a violations R97). The substantive spec content (22 BCs / 22 VPs / arch / manifest) remains stable across ALL META-discipline iterations — no regression in sampled prior closures (F-R88/R91/R93/R94 all HOLDING). ADVERSARY R97 EXPLICITLY RECOMMENDS option (b) Convergence-with-Documented-Residuals. This is the first time the adversary itself issued an explicit gate recommendation. State-manager concurs. After F-R97 fix-burst (VP v1.31), human gate decision pending. Full codification in `cycles/cycle-001/lessons.md §META-asymptote`. 29 codified disciplines remain in force.
- **Human gate decision recorded as OPTION (a) (per D-101 / 2026-05-16):** Human directed OPTION (a) Continue strict D-047. Context cleared after STATE.md v5.47 written. State-manager resumes from this STATE.md v5.47 in next session. Orchestrator will dispatch R98 (adversary D-047 strict pass 1 attempt 31) + cons R37 (consistency-validator round 37) in parallel as first action after context restore. Counter at 0/3. All 29 codified disciplines remain in force. Artifact set locked: PRD v1.21 (0f124a9) + VP v1.31 (a3a68a4) + arch v1.0.21 (42504b4) + manifest v1.1.13 (42504b4). No further unilateral surfacing of option (b) — human chose (a). See §Session Resume Checkpoint — DURABLE (v5.47) for full resume instructions.

- **SE-17e: Cross-Artifact Sibling-Propagation of SE-17a/c-d Discipline — 30th codified discipline (per O-R98-1 / R98 / D-103 / 2026-05-16):** R98 adversary (pass 1 attempt 31) surfaced that sibling artifacts (manifest v1.1.13 §Trace at F-R98-3; arch v1.0.21 §Trace at F-R98-2; PRD v1.21 §Trace at O-R98-1) carried pre-codification audit-block formats that became defects under the v1.31 R97 SE-17a strict codification. SE-17e rule: A fix-burst that touches ANY of the 4 canonical artifacts (PRD, VP, arch, manifest) MUST apply SE-17a (literal-grep evidence) + SE-17c (final-state L-number revalidation) + SE-17d (body-scope filter scope definition) disciplines to that artifact's §Trace transparency blocks, regardless of whether the artifact has historically been SE-17a-strict. No artifact gets a grandfathered exemption from new META disciplines on next touch. Rationale: META-asymptote at META-N+3 (R98) demonstrated that sibling-artifact bursts (manifest v1.1.13, arch v1.0.21, PRD v1.21) carried pre-codification audit-block formats that became defects under post-codification audit lenses. Closing requires forward-propagation on next touch. Per CLAUDE.md Production-Grade Default Rule 1+4, O-R98-1 IS in scope for current cycle (no MVP defer); PO PRD v1.22 will include §Trace retro-fix. 30 codified disciplines in force (was 29 + SE-17e). Full codification in `cycles/cycle-001/lessons.md §SE-17e`.

- **SE-17f: §Trace Evidence-Block Self-Revalidation Gate — 31st codified discipline (per O-R99-1 / R99 / D-106 / 2026-05-16):** R99 adversary (pass 1 attempt 32) returned META-N+5 findings demonstrating that SE-17a/c/c-d/e alone cannot prevent §Trace transparency blocks from introducing new evidence-fidelity defects. F-R99-1 (arch §Trace L-number drift), F-R99-2 (VP §Trace arithmetic contradiction), and F-R99-4 (VP enumeration arithmetic mismatch) all show that §Trace blocks contain evidence claims that were not mechanically re-verified against final-state before commit. SE-17f rule: every §Trace entry that contains a literal grep transcript or L-number citation MUST be followed by a self-revalidation step at burst-finalization: re-run the literal grep(s) cited in the §Trace AND re-Read each cited line N AND compare output against the §Trace claim. Any divergence must either (a) be reconciled before commit or (b) be explicitly documented in a SE-17f self-disclosure block within the same §Trace. The self-revalidation is required for every grep claim AND every L-number citation in the §Trace itself, recursively (the SE-17f block is part of the §Trace; its own L-numbers and outputs must also be revalidated post-edit). Codified per CLAUDE.md Production-Grade Default Rule 5+6 in-scope codification. 31 disciplines in force (was 30 + SE-17f). First empirical test: R100 pass against arch v1.0.23 + manifest v1.1.15 + PRD v1.23 + VP v1.33 §Trace blocks that each apply SE-17f.

- **SE-17g: §Trace Transcript-vs-Narrative Normativity Disambiguation — 33rd codified discipline (per O-R100-1 / R100 / D-110 / 2026-05-17):** R100 adversary (pass 1 attempt 33) process-gap observation O-R100-1 identified that the Burst 2 architect principle "§Trace-body narrative L-numbers are INFORMATIONAL" (D-108) was incorrectly extended to literal `$ grep ...` transcripts displayed inside §Trace SE-17f blocks — enabling both F-R100-1 and F-R100-2. SE-17g rule: inside §Trace body, distinguish two citation classes. **NORMATIVE (must match final-state; SE-17f re-run required):** literal `$ <command>\n<output>` transcripts that declare SE-17a-compliance; production-code L-number citations; frontmatter value citations; explicit `wc -l = N` / `count = N` / enumeration lists labeled "literal" or "final-state." **INFORMATIONAL (range citations; OK if approximate; SE-17f not required):** narrative paragraph references to L-numbers using "approximately lines NNN-NNN" or "around L-NNN"; range-citations explicitly self-labeled "informational" or "narrative range." Burst authoring must explicitly label each citation as NORMATIVE or INFORMATIONAL; ambiguous citations default to NORMATIVE (per CLAUDE.md Production-Grade Default Rule 1 — no MVP deferrals). SE-17f mechanical self-revalidation MUST be re-run on every NORMATIVE element after burst-finalization. INFORMATIONAL elements need not be re-run. Codified per CLAUDE.md Production-Grade Default Rule 5+6 in-scope codification. 33 disciplines in force (was 32 + SE-17g). First empirical test: R101 pass against arch §Trace (Burst 2) + VP §Trace (Burst 3) blocks that each apply SE-17g + SE-17f jointly.

- **SE-16d: Cross-Artifact Chain-Time Monotonicity — 32nd codified discipline (per O-R99-2 / F-R99-3 / R99 / D-106 / 2026-05-16):** R99 adversary (pass 1 attempt 32) finding F-R99-3 proved that per-artifact SE-16b is insufficient: VP v1.32 was authored with a local CDT timestamp that resolved to UTC `2026-05-16T03:15:00Z` — earlier than arch v1.0.22 (`22:00:00Z`), manifest v1.1.14 (`22:00:00Z`), and PRD v1.22 (`23:00:00Z`) timestamps in the same F-R98 serial fix-burst chain. SE-16b checks each artifact's version monotonicity within itself; it does NOT check cross-artifact chain-time ordering. SE-16d rule: every serial fix-burst chain MUST satisfy cross-artifact frontmatter timestamp monotonicity. Each artifact in chain order (Burst 1 → Burst N) must have a frontmatter timestamp >= all prior bursts in the chain, regardless of timezone of authorship. Use UTC ISO-8601 (`YYYY-MM-DDTHH:MM:SSZ`) as the canonical form. Cross-artifact monotonicity is checked at chain-completion (burst N) by state-manager: scan frontmatter timestamps of every artifact touched in the chain; verify Burst-N >= Burst-(N-1) >= ... >= Burst-1. Any inversion must be reconciled (timestamp bump on the later burst) before chain-completion commit. **SE-16d first application (STATE v5.50 / D-106):** cross-artifact chain timestamp matrix for F-R99 chain: arch v1.0.22 = 2026-05-16T22:00:00Z; manifest v1.1.14 = 2026-05-16T22:00:00Z; PRD v1.22 = 2026-05-16T23:00:00Z; VP v1.32 = 2026-05-16T03:15:00Z (UTC; DEFECT — F-R99-3; to be fixed in VP v1.33); STATE v5.49 = 2026-05-16T21:30:00Z. Chain high-water = PRD v1.22 at 2026-05-16T23:00:00Z. STATE v5.50 timestamp = 2026-05-16T23:30:00Z >= 2026-05-16T23:00:00Z — SE-16d PASS for Burst 1. VP v1.33 must use UTC timestamp >= 2026-05-16T23:00:00Z to satisfy SE-16d at chain-completion. Codified per CLAUDE.md Production-Grade Default Rule 5+6 in-scope codification. 32 disciplines in force (was 31 + SE-16d).

- **SE-17c-d FIRST application PROVEN effective — discipline chain Extension 13 → 17 → SE-17a/b/c/d now closes body-scope grep convention gap (per F-R96 / D-098 / 2026-05-15):** F-R96 FV-only fix-burst (SM v5.43 63b5151 + VP v1.30 40248d4) is the first fix-burst to apply SE-17c-d end-to-end. I-R96-1: 5 severity-label sites normalized to MED (correcting I-R95-1 tracking inconsistency across §Trace narrative sites). I-R96-2: SE-17c Step 2 grep evidence rephrased per PG-5 framing — body-scope awk filter derives boundary 3110, correctly excludes 3 §Trace PG-5 historical quotes as expected-by-construction. §Purpose META 17th-attempt; SE-16b PASS. R97 adversary subsequently confirmed: SE-17c-d FIRST application introduced SE-17a defects in Fix 1 + Fix 2 transparency blocks. META-asymptote continues at META-N+2. 29 codified disciplines remain in force.

- **FINAL CONVERGENCE STATEMENT — Phase 1 pipeline at HUMAN GATE (entry 46 / D-100 / 2026-05-15):** After 30 adversary passes + 17 fix-bursts, the Phase 1 Spec Crystallization pipeline has reached the following EMPIRICALLY VERIFIED state: (1) Substantive Phase 1 spec content (22 BCs, 22 VPs, 12 NFRs, 61 ECs, 14 error codes, 21 glossary terms, 23 test names, all architectural anchors) is STRUCTURALLY CONVERGED — R88-R97 secondary content lenses ALL PASS, prior-closure sample (F-R88/R91/R93/R94) ALL HOLDING. (2) META audit-discipline framework has produced 29 codifications (Extensions 1-17 + SE-14b/15a-e/16a-c/17a-d + agent-id-routing + §Trace integrity + §Purpose META guard + §References timestamp guard) — all internally consistent. (3) The remaining defect class is exclusively META audit-narrative evidence-fidelity, which is process-gap (not content) class. (4) R97 META-asymptote test CONFIRMED the pattern continues — each META codification produces META-N+2 findings on first application. (5) Three independent voices (R97 adversary, state-manager, orchestrator) recommend option (b) Convergence-with-Documented-Residuals. Pipeline awaits human decision.

- **SE-17c first application PROVEN effective — codification chain Extension 13 → 17 → SE-17a/b/c now fully closes META audit-evidence class (per F-R95 / D-096 / 2026-05-15):** F-R95 FV-only fix-burst (SM v5.41 fa9cd54 + FV VP v1.29 849e5c8) is the first fix-burst to apply SE-17c (final-state line-number revalidation) end-to-end. Critical confirmation: derived boundary 3110, 17 audit-table rows, frontmatter+body count match — the three-distinct-form same-class pattern (C-R95-1/2/4) is now structurally prevented at burst-finalization. Combined with SE-14b verification+authoring, SE-15e orchestrator dispatch, and SE-16a/b/c canonical-grep + SE-16c++ supplementary — the META discipline stack is mature. R95 secondary lens (arch↔PRD round-trip): ZERO substantive content defects on 11 sampled BCs — the strongest convergence signal in the cycle. Substantive content is CONVERGED; META audit-discipline is STRUCTURALLY MATURE at 28 codified disciplines. §Purpose META 16th-attempt. SE-16b PASS. D-047 strict pass 1 attempt 29 (R96 + cons R35) is the FINAL pre-gate verification candidate.

- **F-R94 demonstrates 4-step serial cascade executing flawlessly under SE-15e; SE-16c++ supplementary grep proven (per D-094 / 2026-05-15):** F-R94 serial fix-burst (SM v5.39 ca90269 → architect arch v1.0.21 + manifest v1.1.13 42504b4 → PO PRD v1.21 0f124a9 → FV VP v1.28 a6a0976) executed under Extension 15+16+17+SE-14b+SE-15e with zero skipped agents and zero propagation gaps. The 4-step dual-file cascade (architect owning both SS-daemon-lifecycle v1.0.21 + SS-deps-pin-manifest v1.1.13 in a single commit, then PO single-file pin propagation, then FV with triple pin propagation + 3 content fixes) validates the deepest dispatch-chain in the cycle. SE-16c++ supplementary grep PROVEN effective: catches standalone version+SHA pairs without PRD prefix — a finding class that escaped prior SE-16c canonical grep on Extension 16 audit tables. Incremental refinement of existing grep-pattern discipline; no new SE codification required. The convergence pattern remains asymptotic — each pass discovers genuinely novel defects on new lens rotations — but substantive content is now extraordinarily mature (22 BCs with explicit constraints, 61 ECs, 21 glossary terms, 39+ bidirectional cross-property citations, 27 codified disciplines). D-047 strict convergence is increasingly unlikely without structural intervention; option (c) hybrid gate recommended at next CLEAN pass.

- **F-R93 demonstrates the deepest serial cascade observed in this cycle — 4-step chain validates SE-15e dispatch enforcement (per D-092 / 2026-05-15):** F-R93 serial fix-burst (SM v5.37 4039afa → architect arch v1.0.20 8533ea2 → PO PRD v1.20 9371348 → FV VP v1.27 202e15c) is the first 4-agent serial chain in the cycle (prior chains were 3-agent: arch → PO → FV). The C-R93-1 closure required all 3 artifact-owners to act: arch for the verification clause at 1 site; PO for PRD §7 RTM 6 rows + §3 §Verification 4 body sites; FV for VP cleanup (O-R93-1/2) + pin propagation. SE-15e dispatch enforcement was applied: no agents were skipped. Critical lesson: SE-15e is proven effective even for the deepest cascade depth encountered to date. The F-R88-5 partial-fix pattern (3rd recurrence across R88 → R92 → R93) is now definitively closed: all 5 taxonomy tiers (VP §Mechanism Distribution, VP §Harness annotations, PRD §7 RTM, PRD §3 §Verification body, arch clause) are aligned on "Integration" taxonomy. SE-14b AUTHORING audit in VP v1.27 returned no-op (no new BC content lifts in this serial chain) — confirming the chain is content-stable at the VP layer; only cleanup (O-R93-1/2) and pin propagation were required. SE-16b monotonicity PASS throughout the 4-step chain.

- **SE-14b: Per-probe BC-VP coherence discipline — VP §Post-condition probes MUST match BC text precision; Extension 14 sub-extension (per I-R91-3/4/5/6 / C-R91-1 / R91 / 2026-05-15):** R91 adversary (pass 1 attempt 24) found SYSTEMATIC 4-finding pattern: VP v1.24 (I-R90-1/2/4 closures) added VP §Post-condition probes with typed constraints (pid ≥ 1, last_app_mode non-empty string, shutdown_reason 4-value enum, semver regex) without simultaneously lifting BC text to specify the same constraint. Additionally: C-R91-1 CRITICAL — VP-FACTORY-002 §Post-cond 7 cites fabricated anchor (`BC-FACTORY-002 EC "idempotency invariant"`) that does not exist; I-R91-2 HIGH — VP-DAEMON-006 §Post-cond 10 cites §Postcondition 1 (wrong — shutdown_reason enum is in §Invariant 1). SE-14b rule: when a VP introduces a §Post-condition probe asserting a TYPED-FIELD constraint not yet in the corresponding BC text, the BC text MUST be lifted in the SAME burst (or the immediately preceding PO burst per Extension 15 SERIAL). The VP `Traces to:` MUST cite a BC element that explicitly covers the matching constraint. Anchor verification sub-rule: every `per BC-XXX §Postcondition N` or `per BC-XXX §Invariant N` cite MUST resolve to a real existing element. SE-14b is Extension 14 sub-extension (within-layer propagation family). 27 codified disciplines in force (was 26 + SE-14b). Full codification in `cycles/cycle-001/lessons.md §SE-14b`.

---

## Wave 6 S-023 Cycle Lessons (D-186 / 2026-05-28)

### L-W6-S023-001: Architect-decision BC/SS propagation requires dedicated orchestrator dispatch

**Date:** 2026-05-28
**Severity:** process-gap
**Origin:** S-023 cycle adversarial pass 2 + S-025 cycle adversarial pass 4 (2 BLOCKERs for BC-2.06.005 + BC-2.06.021 stale pins)

**Pattern:** When an architect makes a decision that bumps a BC or SS document (e.g., BC-2.06.005 v1.0.0 → v1.0.5, SS-tui v1.7.0 → v1.8.0), the propagation to downstream story bodies requires a dedicated orchestrator dispatch to the story-writer agent. The implementer is NOT responsible for spec propagation — the implementer writes code, not spec documents. If the orchestrator dispatches only implementer + adversary without a parallel story-writer + consistency-validator sweep, the BC version pins in story bodies go stale and are caught by a later adversarial pass as BLOCKERs.

**Rule:** When an architect commits a BC or SS version bump mid-delivery-cycle, orchestrator MUST dispatch a parallel story-writer sweep of all downstream stories that cite the bumped artifact. Do not defer to the next pass; the stale pin is a BLOCKER by the time the adversary sees it.

---

### L-W6-S023-002: Multi-pass adversarial convergence novelty-spike confirms 3-consecutive-NITPICK_ONLY threshold

**Date:** 2026-05-28
**Severity:** process validation
**Origin:** S-025 adversarial pass 4 found 2 BLOCKERs (BC-2.06.005 stale pin + AC-005 6→7 column) that were invisible to Passes 1-3

**Pattern:** Adversarial passes do not monotonically decay in severity. Pass N+1 can find higher-severity findings than Pass N if: (a) the spec was modified to close a prior finding and the modification introduced a new gap, or (b) the adversary rotated lens focus. The S-025 cycle demonstrated: Passes 1-3 were HIGH-PRIORITY but closed their respective findings, Pass 4 found 2 fresh BLOCKERs from the Pin bump propagation that Passes 1-3 did not audit. This confirms the 3-consecutive-NITPICK_ONLY convergence criterion is necessary and sufficient — any earlier declaration of convergence would have been wrong.

**Rule:** Never declare convergence at a single NITPICK_ONLY pass. Do not lower the threshold below 3 consecutive passes under any schedule pressure. A novelty-spike at Pass N means the spec surface was not yet stable at Pass N-1.

---

### L-W6-S023-003: CI gate decoupling — downstream gates must not silently skip when upstream gates fail

**Date:** 2026-05-28
**Severity:** process-gap
**Origin:** PROC-SEMGREP-DECOUPLE + PROC-GATE-SKIPPED-LOUD discoveries in S-023 cycle

**Pattern:** When a CI gate fails early (e.g., Preflight fails on missing protoc), subsequent gates in the pipeline may silently not run rather than emitting a clear GATE_SKIPPED signal. This caused Semgrep to silently not run for 24 stories (Waves 1-5). The CI log showed "steps skipped" but not "SEMGREP_GATE_SKIPPED — protoc missing" with a clear indication that security scanning was absent. A human reviewer reading the CI summary would reasonably conclude all gates ran.

**Rule:** Every CI gate that is skipped due to upstream failure MUST emit a loud GATE_SKIPPED log line (not a silent omission) with: (a) which gate was skipped, (b) why (upstream failure), (c) what risk was accepted. Gates must be independently runnable where toolchain permits. Route devops-engineer to decouple Semgrep from Preflight fast-fail chain.

---

### L-W6-S023-004: Architecture-source documents (SS-*) must be swept alongside BCs during spec propagation

**Date:** 2026-05-28
**Severity:** process-gap
**Origin:** F-S025-ADV4-BLOCKER-001 — BC-2.06.005 pin stale in S-025; root cause was SS-tui.md v1.7.0 → v1.8.0 bump not propagated to story bodies

**Pattern:** When PO sweeps BCs for a version bump, the routing table assigns SS-* documents to the architect. If the orchestrator dispatches only PO (for BCs) without simultaneously dispatching architect (for SS-*), the SS-* version pins in story inputs: sections go stale. Story-writer reads both BCs and SS-* documents — both must be current. The adversary correctly caught the BC pin as a BLOCKER but the root cause was the SS-* document version bump not being propagated atomically with the BC sweep.

**Rule:** Any burst that bumps a BC MUST check: does this BC depend on an SS-* document that was also bumped in the same cycle? If yes, dispatch architect for SS-* propagation in the same serial chain as the PO BC sweep. PO cannot be the sole propagation agent for a BC that derives from an SS-* document.

---

### L-W6-S023-005: Pre-existing technical debt can be cleared in-scope of a fix PR when CI cascade-fails — orchestrator-authorized scope expansion per CLAUDE.md Principle 4

**Date:** 2026-05-28
**Severity:** process codification
**Origin:** S-023 PR #29 scope expansion: ADV-W4GATE-MED-001 (PATH isolation) cleared in-scope

**Pattern:** S-023 CI first run triggered a cascade failure that surfaced ADV-W4GATE-MED-001 (PATH env mutation in detect_ccr tests, tracked since Wave 4). The implementer surfaced the finding to the orchestrator rather than silently deferring. Orchestrator invoked CLAUDE.md Principle 4 ("AI-built defects are the AI's responsibility to fix") and authorized in-scope scope expansion: commit 295dc1b migrated the detect_ccr tests to temp_env::with_vars. This cleared a Wave-4 durable task register item without requiring a separate fix PR.

**Rule:** When a story's CI run cascades into a pre-existing durable task register item that can be fixed in 1-2 commits, the correct default is orchestrator-authorized in-scope expansion (CLAUDE.md Principle 4). The implementer surfaces the finding + proposed fix; orchestrator authorizes or defers based on scope risk. This is preferable to accumulating fix PRs for mechanical test-isolation fixes. The tech-debt-register must be updated atomically in the same state burst as the fix.

---

## Wave 6 S-025 Cycle Lessons (D-187 PENDING / 2026-05-28)

### L-W6-S025-001: Architect-decision propagation requires dedicated dispatches covering ALL five artifact classes

**Date:** 2026-05-28
**Severity:** process-gap
**Origin:** S-025 adversarial passes 1-11 recurring class: architect decisions on BC shapes and SS-* versions not propagated to all downstream artifacts

**Pattern:** When an architect makes a decision that changes a BC or SS-* document, the complete propagation surface is: (1) BC body text, (2) SS-* architecture source documents, (3) story frontmatter pins (inputs: version fields), (4) story body language (AC prose, BC references), (5) input-hashes for affected SS-* documents. Missing any ONE of these five artifact classes creates a finding in a later adversarial pass. Passes 3, 4, 5 in the S-025 cycle each found a distinct class of propagation miss: Pass 3 missed SS-ipc; Pass 4 missed BC-2.06.005 pin in story; Pass 5 missed SS-tui architecture source document. Pass 11 found that BC-2.06.016 PC-4 prose had not been propagated to match the bracketed production style.

**Rule:** When the orchestrator dispatches an architect-decision propagation burst, the dispatch prompt MUST enumerate ALL five classes explicitly with grep targets. A BC-only sweep is incomplete. An SS-only sweep is incomplete. A story-pin-only sweep is incomplete. All five classes must appear in the dispatch checklist.

---

### L-W6-S025-002: Sweep-audit must trace assertion to the EXACT production code path

**Date:** 2026-05-28
**Severity:** process-gap
**Origin:** S-025 adversarial passes 9-10: Pass 9 found NaN/inf and canonical text untested; Pass 10 found Pass 9 sweep was vacuous-mirror (assertion on TestBackend local canonical_msg, not production-rendered path)

**Pattern:** A sweep that examines whether a canonical string is tested may conclude "PASS" when the test asserts against a TestBackend-rendered local copy, not against the value produced by the actual production code path. This is the vacuous-mirror-test pattern generalized: the test passes because the test itself produced the expected value in test scope, not because the production path produces it. Pass 9 adversary correctly identified that AC-003 canonical text was untested; Pass 10 adversary correctly identified that the Pass 9 "fix" was itself a vacuous-mirror sweep.

**Rule:** Every sweep that validates "this canonical string is tested" must trace the assertion to the EXACT production function or constant that will be called at runtime. If the test constructs the expected value locally (via a local variable, TestBackend render, or test-scope helper), and that value is not the same object as the production-path value, the test is vacuous. The production-grade assertion is: `assert_eq!(production_function_output, CANONICAL_CONST)` where CANONICAL_CONST is the same const used by production code.

---

### L-W6-S025-003: pub const extraction is the production-grade default for canonical strings shared by production and tests

**Date:** 2026-05-28
**Severity:** codification
**Origin:** S-025 Pass 11 fix round: implementer extracted 6 pub const + 1 format_drop_counter(n) helper to eliminate the vacuous-mirror class structurally

**Pattern:** When production code and test code must agree on a canonical string value (status text, label, overflow cap, etc.), the correct pattern is: (1) declare the string as a `pub const` in production code, (2) import the same `pub const` in tests. This eliminates the vacuous-mirror failure mode at the structural level — if the production code changes the string, the test immediately breaks. The pattern also eliminates the sibling-extraction gap finding class: once a const exists for one usage site, all usage sites in the codebase can import it.

**Rule:** Any canonical string that appears in both production code and test assertions MUST be extracted to a `pub const`. Re-export from `lib.rs` so that external test crates and integration test files can import without path gymnastics. Add this as a pre-commit checklist item: "Does any test assert a literal string that also appears in production code? Extract to pub const."

---

### L-W6-S025-004: Multi-pass convergence: premature-clean signal at counter-advance moments is a structural property

**Date:** 2026-05-28
**Severity:** process codification
**Origin:** S-025 adversarial pass 8 = 1/3 CLEAN, then Pass 9 found NEW class (NaN/inf + canonical text untested); counter reset to 0/3

**Pattern:** In a multi-pass adversarial convergence cycle, the FIRST clean pass (counter 1/3) is the moment of maximum risk for premature declaration of convergence. The first clean pass confirms only that the finding class from the PREVIOUS passes was closed. It does not confirm that no new class will appear. The S-025 cycle demonstrated: Pass 8 was NITPICK_ONLY (confirmed Passes 1-7 finding classes were closed); Pass 9 immediately found two NEW classes (float edge cases, canonical text coverage). The 3-consecutive-NITPICK_ONLY rule exists precisely to catch this: a single clean pass is insufficient evidence of structural convergence.

**Rule:** At every counter-advance moment (Pass N advancing counter from K to K+1 with K < 3), the adversary dispatch prompt MUST include: "MAXIMUM SKEPTICISM MODE: counter is now K/3. S-022 and S-025 cycle history shows premature-clean signals at counter-advance moments. Look harder, not softer. Rotate to unexplored lens axes." Counter resets to 0/3 immediately on any non-NITPICK_ONLY pass.

---

### L-W6-S025-005: IEEE 754 subtleties for f64-typed BC-format fields

**Date:** 2026-05-28
**Severity:** codification
**Origin:** S-025 adversarial pass 9: NaN/inf edge cases in format_cost; Pass 11 LOW-001 removal of redundant cost < 0.0 guard

**Pattern:** f64-typed BC fields require explicit IEEE 754 edge-case handling: (a) NaN: `value.is_nan()` must be checked FIRST in guard chains because NaN comparisons return false for all `<`, `>`, `==`, `<=`, `>=` operators — NaN < 0.0 is false; (b) infinity: `value.is_infinite()` returns true for both positive and negative infinity; (c) negative zero: `-0.0 == 0.0` evaluates to true in Rust, so `cost < 0.0` does NOT catch negative zero — `is_sign_negative()` is the correct check. A guard chain `if cost < 0.0 { cap } else if cost.is_sign_negative() { ... }` is redundant because `is_sign_negative()` subsumes `< 0.0` (negative-finite values are covered by both). The correct minimal guard chain is: `is_nan()` → `is_infinite()` → `is_sign_negative()` in that order.

**Rule:** Any BC that specifies format or display behavior for an f64-typed field MUST enumerate NaN, positive infinity, negative infinity, and negative zero as edge cases in the AC or EC section. The corresponding test MUST assert the correct output for each of these four values using `f64::NAN`, `f64::INFINITY`, `f64::NEG_INFINITY`, and `-0.0_f64`. Guard chains in production code MUST check `is_nan()` first.

---

### L-W6-S025-006: SS-* architecture source documents are required propagation targets alongside BC bodies

**Date:** 2026-05-28
**Severity:** process-gap
**Origin:** S-025 adversarial pass 5: SS-tui v1.8.0 had BC-2.06.005 column count still at 6; architect-decision propagation sweep had covered BCs but missed SS-tui architecture source

**Pattern:** BC documents derive from SS-* architecture source documents. When an architect decision changes the canonical shape defined in an SS-* document, the propagation surface includes the SS-* document body (not just the BC files). If the orchestrator dispatches only the PO for BC propagation and the story-writer for pin propagation, the SS-* architecture source document retains the stale shape. The adversary correctly catches this as a BLOCKER because the SS-* document is the normative architecture reference; a story implementer reading it will implement the wrong shape.

**Rule:** L-W6-S025-001 enumerates SS-* as artifact class (2). This lesson provides the specific operational trigger: when an architect decision changes a shape (column count, struct fields, enum variants), the architect is responsible for propagating to SS-* documents in the SAME burst as the decision, not as a follow-up. If the decision and the SS-* propagation are separated across bursts, the window between bursts creates a stale-spec risk.

---

### L-W6-S025-007: Production-grade sweep audits must expand beyond the flagged targets

**Date:** 2026-05-28
**Severity:** codification (CLAUDE.md Principle 4 extension)
**Origin:** S-025 Pass 11 implementer found 3 additional pub const class siblings beyond the 3 flagged by Pass 11; S-025 Pass 7 test-writer found 2 additional drift instances beyond the 1 flagged

**Pattern:** When an adversarial pass flags N instances of a finding class, the production-grade fix is to find and fix ALL instances of that class in the codebase — not just the N flagged instances. This is a direct implication of CLAUDE.md Principle 4 ("AI-built defects are the AI's responsibility to fix in scope"). The adversary's N flagged instances are a floor, not a ceiling. A sweep that addresses only the flagged N instances is a partial fix that will produce a sibling-extraction finding in the next pass.

**Rule:** When an implementer or test-writer fixes a class of finding (e.g., literal-string-in-test, missing-pub-const, stale-doc-comment), the fix workflow MUST: (1) apply the fix to all N flagged instances, (2) grep the ENTIRE codebase for the same pattern signature, (3) apply the fix to all grep hits not already addressed. Report the total hits found vs flagged count in the commit message (e.g., "Fixed 6 instances: 3 flagged by adversary + 3 additional found by sweep"). A fix that reports 3-of-3 when grep would have found 6-of-6 is incomplete.

---

### L-W6-S025-008: Enforcement gates must themselves be verified to actually scan the intended corpus

**Date:** 2026-05-30
**Severity:** codified (process-gap — closed in scope D-208)
**Origin:** S-025 adversarial Pass 29 [process-gap]: POL-11 was live in CI but collect_files() was hardcoded to `.factory` instead of reading the `--factory-root` flag value, causing it to scan ZERO files on the CI runner (where factory-artifacts are checked out to `.factory-spec`). First CI run of POL-11 scanned 0 files and reported PASS vacuously.

**Pattern:** When a new CI enforcement gate is deployed for the first time, its "PASS" result is ambiguous: it may pass because there are no violations, or because it is scanning an empty or wrong corpus. The devops engineer who implements a gate and the adversary who reviews the next pass must both verify the gate's scope empirically — not just that the gate runs without error. A gate that reports 0 findings after scanning 0 files is indistinguishable from a gate that legitimately finds no violations without an explicit scope-verification step.

In the S-025 cycle: POL-11 deployed at Pass 28 (f0926fe). Pass 28 adversary ran with POL-11 reporting "PASS" but did not verify file count. Pass 29 adversary [process-gap] correctly flagged this. Resolution required: devops scope fix (adaf9d2), architect ADR-0007 v1.0.4 (§Enforcement Scan Scope ratified; exempt dirs listed), corpus sweep (0d190a5). This was the 11th META-pattern instance — the enforcer itself had a scope bug on first deployment.

**Rule:** When a CI enforcement gate is deployed or modified, the deployment commit MUST include a dry-run output line reporting the number of files scanned (e.g., "541 files scanned, 0 findings"). The adversarial pass that follows a new gate deployment MUST verify the scanned-file count is non-zero and plausible for the corpus size. If the scanned-file count is zero or suspiciously low, this is a BLOCKER finding — not a LOW or NIT. The gate is not live until scope is empirically verified.

---

### L-W6-S025-009: Green CI proves the gate ran, not that it detects the input form [codified]

**Date:** 2026-05-30
**Severity:** codified (process-gap — closed in scope D-209, Pass 30 remediation)
**Origin:** S-025 adversarial Pass 30 F-S025-ADV30-MED-001: POL-11 was BLIND to YAML `inputs[]` form (e.g., `{path: ..., version: "1.5"}`). The `check_version_pins.py` regex was matching only prose form `EVAL-INDEX v1.5` and `EVAL-INDEX.md v1.5` but not object-style `EVAL-INDEX.md, version: "1.5"`. CI reported PASS — 264 active, 0 findings — but this was a false-green because the detector never saw the input form. devops added Pattern B detection (feature branch e38c9d0); architect ratified in ADR-0007 v1.0.6.

**Pattern:** A CI gate can report 0 findings while being structurally blind to entire classes of input. L-W6-S025-008 established that a gate scanning 0 files is a BLOCKER. This lesson extends the principle: a gate scanning the correct file count but with regex patterns that do not cover all input forms can produce identical "0 findings" output. The false-green is undetectable without explicit detector-coverage verification: run the gate against a synthetic file that CONTAINS the uncovered input form and confirm a finding is raised.

**Rule:** When a CI enforcement gate is deployed or modified, detector coverage MUST be verified against real input shapes — not just prose "vN.N" form, but all structural forms the detector is expected to catch (YAML object form, inline table form, frontmatter form, etc.). Add a test fixture with each input form to the gate's test suite. A coverage-miss in a structural input form is a false-green and must be classified as a BLOCKER. "CI was green" is NOT sufficient evidence that a gate covers the input shape — only explicit detector-coverage probing is.

---

### L-W6-S025-010: Classification policies need a CLOSED rule with a safe default [codified]

**Date:** 2026-05-30
**Severity:** codified (architectural — closed in scope D-209, Pass 30 remediation)
**Origin:** S-025 adversarial Pass 30 F-S025-ADV30-MED-001 root-cause analysis: the long tail of META-pattern recurrences (11 instances across Passes 18-29) was enabled by an open-ended ACTIVE classification. Every time a new document class appeared (BC body, story inputs[], VP body, EVAL-INDEX inputs[]), the adversary correctly asked "is this active or historical?" and POL-11 enforcement was partial until the boundary was re-argued. Human approved Option A (historical provenance) to permanently close the set: the ACTIVE set is `{*-INDEX.md + prd.md}` by closed rule; everything else is HISTORICAL by default. ADR-0007 v1.0.6 codified this as a closed rule.

**Pattern:** A policy that classifies artifacts as ACTIVE or HISTORICAL without a closed enumeration of the ACTIVE set will re-trigger the same boundary dispute for each new document class encountered. The META-pattern in S-025 (12 instances in 30 passes) was enabled by an open-ended ACTIVE boundary. Each new document class (BC body pins → story inputs[] pins → VP body pins → EVAL-INDEX inputs[] pins) required a fresh adversarial challenge, a fresh architectural ruling, and a fresh enforcement fix. An open ACTIVE rule produces O(document-class-count) recurrences. A CLOSED active rule (enumerated finite set) produces O(1) recurrences (only when the closed set itself is updated).

**Rule:** Any enforcement policy that must distinguish between two classification states (ACTIVE vs HISTORICAL, normative vs informational, required vs optional) MUST specify:
1. A CLOSED enumeration of the minority class (the stricter-enforcement class — here: ACTIVE = `{*-INDEX.md, prd.md}`).
2. An explicit SAFE DEFAULT for all non-enumerated artifacts (here: HISTORICAL).
3. The procedure for adding to the closed set (here: ADR-0007 amendment with human approval).
Without a closed rule, each new document class triggers a new boundary dispute. The safe-default + closed-set pattern eliminates the long tail structurally.

---

### L-W6-S025-011: Hardcoded detector vocabularies re-trigger the META-pattern per artifact-class; make detection REGISTRY-DRIVEN [codified]

**Date:** 2026-05-30
**Severity:** codified (architectural — closed in scope D-210, Pass 31 remediation)
**Origin:** S-025 adversarial Pass 31 F-S025-ADV31-MED-001: POL-11 Pattern-A regex matched only artifacts with SS-/BC-/ADR- prefixes. Artifacts with no recognized prefix (dtu-assessment.md, ARCH-INDEX.md, STORY-INDEX.md, EVAL-INDEX.md, product-brief.md, etc.) produced false-greens — their stale citations were invisible to the detector. This was the 13th META-pattern instance; first vocabulary-blind-spot sub-species. ROOT FIX: registry-driven Pattern-A — the detector now reads version-pin-registry.yaml and matches ANY artifact ID present in the registry, eliminating the prefix hard-coding entirely.

**Pattern:** A detector that hard-codes artifact-class prefixes (e.g., only `SS-.*\.md`, `BC-.*\.md`, `ADR-.*\.md`) will produce a false-green for any artifact whose prefix is not in the hard-coded list. Each time a new artifact class enters the corpus (dtu-assessment, story files, VP files, EVAL-INDEX, product-brief, etc.), the detector silently fails to enforce against it. The hard-coded list will always lag behind the actual artifact-class population. The vocabulary-blind-spot is structurally guaranteed for any novel artifact class. Making detection DATA-DRIVEN (keyed off the registry that is authoritative for what artifacts exist) eliminates the vocabulary blind-spot permanently: any artifact registered in version-pin-registry.yaml is automatically detected regardless of naming convention.

**Rule:** CI enforcement gates that detect references to versioned artifacts MUST derive the set of artifact IDs to detect from the canonical registry (e.g., `version-pin-registry.yaml`), NOT from a hard-coded prefix list or enumeration in the detector script. When a new artifact is added to the registry, detection coverage automatically extends to that artifact — no detector update required. Hard-coded prefix lists are permanently forbidden as the primary detection mechanism. They may be used only as a performance optimization (pre-filter before registry lookup), never as the sole gating condition.

---

### L-W6-S025-012: Improving a gate's coverage surfaces the full pre-existing debt at once — budget for the cascade; version-free (Option 2) is the permanent no-re-stale fix for navigation pointers [codified]

**Date:** 2026-05-30
**Severity:** codified (process — closed in scope D-210, Pass 31 remediation)
**Origin:** S-025 Pass 31 remediation: registry-driven Pattern-A was deployed and immediately surfaced 207 project-wide stale citations that the prefix-filtered detector had never seen. Post-exemption (3 living-state files: sprint-state.yaml, tech-debt-register.md, CLAUDE.md): 164 findings. story-writer fixed 154 .factory findings; implementer fixed 10 feature-branch findings. For 41 BC-HOOK navigation pointers (e.g., `BC-HOOK-039 v1.0.3` inline citation), Option 2 (version-free bare name) was applied — eliminating the possibility of the pointer going stale on future BC-HOOK bumps.

**Pattern (cascade budget):** When a CI enforcement gate is made more comprehensive — by extending its detection scope, adding new patterns, or switching from prefix-filtered to registry-driven — it will surface ALL pre-existing violations that were previously invisible. The surfaced count can be large (here: 207 active stale citations across 538 files). This is not a regression in content quality; it is correct gate behavior. The remediation cycle must be resourced appropriately: budget for the full cascade, not just the newly-detected count. Declaring "the gate is now fixed" without completing the cascade leaves the corpus in a known-unclean state and guarantees a finding in the immediately following adversarial pass.

**Pattern (Option 2 — permanent fix for navigation pointers):** Navigation pointers are references whose sole purpose is to identify an artifact (e.g., "see BC-HOOK-039 for the full rule"). These pointers do not make a versioned claim — they are not asserting the artifact is at a specific version. Encoding a version number in a navigation pointer (e.g., "BC-HOOK-039 v1.0.3") creates a stale pointer on every future bump of BC-HOOK-039. Option 2 (version-free: "see BC-HOOK-039") is correct for navigation pointers and is permanently no-re-stale. Option 1 (version bump cascade) is correct for versioned claims (e.g., a §Trace entry that records "fixed in BC-HOOK-039 v1.0.3 at commit X" — this is a historical anchor and should remain version-pinned).

**Rule:** When a gate's coverage is expanded (new patterns, wider scope, registry-driven), plan for the full cascade before deploying. After deploying the expanded gate: (1) run it against the full corpus immediately, (2) triage all findings by option: Option 2 (version-free) for navigation pointers, Option 3 (historical-anchor) for §Trace/historical records, Option 1 (bump) for active live claims, (3) fix ALL findings to fixpoint before declaring the remediation cycle closed. A partial fix (only the directly-flagged files) will produce a CI failure on the next run and a finding in the next adversarial pass.

---

### L-W6-S025-013: When you fix a scope/form/vocabulary gap in one enforcement gate, audit the sibling gate for the same gap-class proactively [codified]

**Date:** 2026-05-30
**Severity:** codified (process-gap — closed in scope D-211, Pass 32 remediation)
**Origin:** S-025 adversarial Pass 32 F-S025-ADV32-MED-001: POL-12 (check_structural_claims.py) scanned only stories/ while ADR-0008 §Phase 1 mandates stories/ + behavioral-contracts/ — a scope gap. This is the same gap-class as POL-11 Pass-29 (enforcer scanning ZERO files, scope bug). POL-11's scope gap was fixed in Pass-29 (D-208, devops aa0a5d6). POL-12 was not audited for the same gap-class at that time, allowing the sibling gate to carry the identical defect through Passes 30 and 31 undetected. devops 92fe2f8 (feature branch) fixed POL-12: collect_bc_files() added; 152 files scanned (38 stories + 114 BCs); 3 BC regression fixtures + IPC-homonym false-positive guards; 30/30 fixtures pass; 0 stale structural claims.

**Pattern:** Enforcement gates deployed to enforce adjacent concerns (POL-11: version-pin citations; POL-12: structural claims) share structural properties: both scan a corpus of normative files, both use pattern-matching logic, both have scope definitions in their corresponding ADRs. When a scope/form/vocabulary gap is found and fixed in one gate, the same gap-class is likely to exist in sibling gates because they were authored by the same process with the same structural template. The adversary will find it in a subsequent pass if the sibling audit is not performed proactively. Each unaudited sibling gate costs one adversarial pass and one counter HOLD.

**Rule:** When a gap-class is found and fixed in an enforcement gate (scope gap, form-coverage gap, vocabulary gap), the devops engineer who authors the fix MUST audit all sibling enforcement gates for the same gap-class in the same PR. The audit must be explicit: (1) identify all sibling gates (other CI lint scripts enforcing VSDD policies), (2) for each sibling gate, verify that the gap-class is absent — check the scan scope against the authoritative ADR, check all pattern forms, check all vocabulary coverage, (3) either confirm absence or apply the equivalent fix. The audit result must be documented in the commit message or PR description. A sibling audit that is not documented is not an audit.

---

### L-W6-S025-014: Suppression guards added to fix false-positives can introduce false-negatives — disambiguate by TYPE/value, never by blanket name-exclusion [codified]

**Date:** 2026-05-30
**Severity:** codified (architectural — closed in scope D-212, Pass 33 remediation)
**Origin:** S-025 adversarial Pass 33 F-S025-ADV33-MED-001 + F-S025-ADV33-MED-002: The IPC-homonym guards added in Pass 32 (devops 92fe2f8) to suppress false-positives from field names that coincidentally matched structural-claim patterns introduced two false-negative vectors. MED-001: guards were single-line matchers, so a multi-line claim assembly (pending_app split across two lines) bypassed detection entirely. MED-002: the `overlay_stack` field-name Path-B exclusion was over-broad — it excluded ALL structural claims containing "overlay_stack", blinding the gate to App-form claims in that field context. ROOT FIX: devops 10cdb0b (feature) — multi-line cross-line assembly detection (parity with POL-11 Pattern B); TYPE-AWARE IPC-homonym disambiguation (exclude only IPC-type homonyms, never App-type structural claims).

**Pattern:** When a false-positive suppression guard is added to an enforcement gate, it narrows the gate's detection surface. Blanket name-based exclusions ("skip any claim containing field name X") are structurally dangerous: they exclude the false-positive case but also exclude any genuine structural claim that happens to mention the same field name. The guard silently blinds the gate to an entire form category. Two failure modes compound: (1) guards written for single-line patterns miss multi-line claim assembly (a form-coverage gap); (2) value-agnostic name exclusions suppress by name rather than by type context (a type-disambiguation gap). Both reduce coverage while appearing to keep the gate PASSING.

**Rule:** Suppression guards added to fix false-positives MUST: (1) be TYPE-AWARE — exclude only the specific type/context that is a false-positive (e.g., "field name X in IPC-struct context") rather than blanket name exclusion ("field name X anywhere"); (2) be form-complete — if the gate detects single-line forms, the guard must also handle multi-line forms; (3) be accompanied by a §Form-Coverage Matrix (see L-W6-S025-015) that documents every structural form the gate is expected to detect and which forms the guard covers or exempts. A guard that is not TYPE-AWARE and form-complete is a latent false-negative.

---

### L-W6-S025-015: A §Form-Coverage Matrix is the fixpoint artifact that ends form-gap whack-a-mole; version-free inputs[] permanently ends active-index re-stale cascades [codified]

**Date:** 2026-05-30
**Severity:** codified (architectural + process — closed in scope D-212, Pass 33 remediation)
**Origin:** S-025 adversarial Pass 33 remediation. Two structural problems resolved simultaneously. (1) After D-212 architect 3a83365 (ADR-0008 v1.0.5): multi-line + type-aware homonym normative rules added; §Form-Coverage Matrix added to BOTH POL-11 and POL-12 covering 35/35 fixtures — zero silent skips. Adversary confirmed: third gate (audit-table) SOUND; POL-11 fixpoint; IPC-marker guard sound; ADRs clean; S-025 content converged. GATE-COMPLETENESS STREAK extended to 5. (2) story-writer 3d2190e converted STORY-INDEX inputs[]/traces_to to VERSION-FREE bare-filename form (ADR-0007 Option 2), permanently ending the active-index re-stale cascade: future bumps of ARCH-INDEX, STORY-INDEX, EVAL-INDEX no longer produce stale STORY-INDEX inputs[]. Also fixed 3 downstream traces_to (EVAL-INDEX, dependency-graph-expansion, holdout-scenarios).

**Pattern — §Form-Coverage Matrix:** An enforcement gate that detects structural forms (single-line, multi-line, field-name variants, type-context variants) has an open-ended detection surface. Each new form discovered by the adversary requires a fix + re-verify cycle (whack-a-mole). The whack-a-mole ends when the gate's detection surface is exhaustively enumerated: a §Form-Coverage Matrix maps every known form → checked (detected), exempt (excluded with justification), or not-applicable. A gate with an exhaustive §Form-Coverage Matrix has no silent detection gaps; any newly-discovered form becomes a matrix addition with explicit classification. The matrix IS the fixpoint artifact — not the number of adversarial passes.

**Pattern — version-free inputs[]:** Index documents (STORY-INDEX, EVAL-INDEX, ARCH-INDEX) that cite frequently-bumped artifacts in their inputs[] or traces_to fields will re-stale on every bump of those artifacts. Converting inputs[]/traces_to citations from versioned form ("ARCH-INDEX.md v1.0.24") to version-free bare-filename form ("ARCH-INDEX.md") eliminates the re-stale mechanism permanently. The document still traces its inputs; it simply does not pin the version, which is correct because index documents describe current state, not a snapshot.

**Rule (§Form-Coverage Matrix):** Every VSDD enforcement gate (POL-N CI script) MUST maintain a §Form-Coverage Matrix in its governing ADR. The matrix has columns: Form | Example | Detected? | Notes. Rows cover every structural form the gate is designed to detect. When a new form is found in an adversarial pass, the first remediation step is to add it to the matrix with its detection status — before writing the fix code. A gate without an exhaustive §Form-Coverage Matrix is not at fixpoint, regardless of the current adversarial pass result. Pass 33 establishes both POL-11 and POL-12 have exhaustive §Form-Coverage Matrices (35/35 fixtures covered), making them fixpoint candidates for Pass 34 audit.

**Rule (version-free inputs[]):** For index documents that trace inputs to frequently-bumped artifacts, prefer version-free (bare-filename) form in inputs[]/traces_to fields (ADR-0007 Option 2). Reserve versioned form (Option 1) for body prose that makes a claim about a specific version. This distinction prevents index-document re-stale cascades from propagating across the corpus on every artifact bump.

---

### L-W6-S025-016: A coverage matrix needs a DEFERRED/NOT-ENFORCED value — otherwise a deferred form gets mislabeled CHECKED and the matrix lies about gate completeness [codified]

**Date:** 2026-05-30
**Severity:** codified (architectural — closed in scope D-213, Pass 34 remediation)
**Origin:** S-025 adversarial Pass 34 F-S025-ADV34-MED-001: ADR-0008 §Form-Coverage Matrix carried the row "module-level doc-comment table" with status CHECKED. However, the Phase-1 gate (check_structural_claims.py) does NOT scan crates/**/*.rs — that coverage is explicitly deferred to Phase 2 (unit-level ADR) and Phase 5 (adversarial hardening). The matrix labeled a not-yet-enforced form as CHECKED, making the matrix's own invariant ("no silent-blindness") false. This is the 17th META instance — a new sub-species: matrix-vs-code self-consistency. ROOT FIX: architect 42eb74c — DEFERRED gate-treatment value added to §Form-Coverage Matrix; the row was relabeled DEFERRED (Phase 2/Phase 5) with explicit rationale; the 'no silent-blindness' invariant was qualified TRUE (with deferred exception documented). ADR-0008 v1.0.5→v1.0.6; ARCH-INDEX v1.0.24→v1.0.25. No feature-branch change (ADR doc only). Adversary confirmed: empirical test CLEAN; matrix self-consistent; Pass 35 is genuine fixpoint candidate.

**Pattern:** A §Form-Coverage Matrix that has only binary values (CHECKED / EXEMPT / NOT-APPLICABLE) cannot represent enforcement that is correct but deferred to a later phase. A form that is genuinely in scope but whose gate implementation is deferred will be mislabeled — either as CHECKED (false-positive: claims enforcement that doesn't exist) or EXEMPT (false-negative: implies the form is intentionally out of scope forever). Both mislabelings make the matrix lie. The matrix becomes the defect instead of the defense.

**Rule:** Every §Form-Coverage Matrix MUST support a DEFERRED gate-treatment value alongside CHECKED, EXEMPT, and NOT-APPLICABLE. The DEFERRED row must specify: (1) which phase or gate is responsible for enforcement, (2) the reason enforcement is not present in the current gate, (3) any preconditions for the deferred enforcement to activate. A DEFERRED row is not a gap — it is an honest acknowledgment that the form is known, out-of-scope for the current phase, and tracked for future enforcement. When authoring a §Form-Coverage Matrix, audit every row against the actual gate implementation: if the gate script does not scan a given path or pattern, the row MUST NOT be labeled CHECKED. Apply DEFERRED or NOT-APPLICABLE with justification instead. A §Form-Coverage Matrix is only self-consistent when every CHECKED row is empirically verified against what the gate actually scans.

---

### L-W6-S025-017: Enforcement tooling's own self-description (script docstrings citing the policy ADR) drifts on ADR bumps and is invisible to the gate that governs other files — version-free §-anchor citations in tooling docstrings make this drift class structurally impossible [codified]

**Date:** 2026-05-30
**Severity:** codified (process-gap — closed in scope D-214, Pass 35 remediation; LIGHT cycle, feature-branch only)
**Origin:** S-025 adversarial Pass 35 F-S025-ADV35-LOW-001: the POL gate scripts' POLICY SUMMARY docstrings at the top of check_version_pins.py and check_structural_claims.py cited pinned ADR versions — "ADR-0008 v1.0.4" and "ADR-0007 v1.0.7" — that contradicted the v1.0.6 and v1.0.8 policy the scripts actually implement. The defect was invisible to both POL-11 and POL-12 because .py files are excluded from their normative-file scan scope (POL-11 is a Python script, not a .factory artifact; it cannot govern its own citations). This is the 18th META instance — a new sub-species: gate-self-description drift. ROOT FIX: devops 4a88e5f (feature branch) — COMPREHENSIVE sweep: ALL 11 pinned ADR/spec citations across scripts/ converted to VERSION-FREE §-anchor form (e.g., "per ADR-0008 §Structural-Claim Discipline" instead of "per ADR-0008 v1.0.4"). No logic change. 35/35 fixtures pass; both gates PASS 0/0. Zero remaining pinned citations in .py files. Adversary confirmed: both gates at BEHAVIORAL + DOCUMENTARY fixpoint; empirical test CLEAN; Pass 36 is the genuine advance candidate with no remaining drift surface.

**Pattern:** An enforcement gate (CI lint script) necessarily enforces OTHER files; it cannot scan itself for version-pin drift without infinite recursion. When the gate script's own docstring contains a versioned ADR citation (e.g., "Implements ADR-0007 v1.0.4"), every ADR bump that changes the version number creates a stale citation in the script — and the script's own gate cannot detect it. The adversary finds it in the next pass. The meta-pattern is: the tooling that enforces drift-freedom is itself subject to drift in its self-description, but is blind to that drift. This is a structural property of all self-governing enforcement systems, not a one-off authoring error.

**Rule:** VSDD enforcement gate scripts (POL-N CI scripts) MUST use version-free §-anchor citations in their own docstrings and POLICY SUMMARY headers when referencing governing ADRs and specs. The correct form is a §-anchor reference ("per ADR-0008 §Structural-Claim Discipline") rather than a versioned pin ("per ADR-0008 v1.0.4"). This makes the docstring permanently accurate regardless of ADR version bumps — the §-anchor resolves to whatever the current ADR contains. Versioned citations in .py script docstrings are a drift class that is structurally invisible to the very gate that would catch them elsewhere; the version-free form eliminates the drift class entirely. When an ADR version bumps, no docstring update is required and no stale-citation can arise. This rule applies to: POLICY SUMMARY headers, inline "# per ADR-NNN" comments, and any other prose in .py enforcement scripts that cites a governing VSDD document by version.

---

### L-W6-S025-018: Gate-completeness defects decay monotonically toward a fixpoint when each fix closes the whole CLASS rather than the instance — the streak terminates rather than recurring forever [codified]

**Date:** 2026-05-30
**Severity:** codified (convergence pattern — observed at D-215, Pass 36 CLEAN)
**Origin:** S-025 adversarial Pass 36 verdict: ZERO findings. This was the FIRST counter advance (0/3 → 1/3) after 15 consecutive 1/3→2/3 failures and a 7-pass GATE-COMPLETENESS STREAK (Passes 29-35). The adversary independently re-derived: App-struct canonical (SS-tui §App struct 9 fields), registry currency, both §Form-Coverage matrices match scripts, cascade consistent, both ADRs self-consistent, no residual drift surface in README/CHANGELOG/CI-comments/gate-docstrings. "The 7-pass gate-completeness decay (P29 scan-scope → P35 docstring version-free) has terminated."

**Pattern:** The GATE-COMPLETENESS STREAK (7 consecutive passes, Passes 29-35) appeared to be non-terminating from inside the streak — each pass found a new sub-species of gate-completeness defect (scope bug → YAML blind-spot → vocabulary blind-spot → sibling-gate scope gap → suppression-guard false-negatives → matrix-vs-code mislabel → gate-self-description drift). However, the streak was finite because each fix addressed the WHOLE CLASS of the defect (registry-driven vocabulary, closed-rule classification, exhaustive §Form-Coverage matrix, version-free tooling docstrings), not just the instance found in that pass. Class-level fixes are structurally non-recurring: once the §Form-Coverage Matrix is exhaustive and version-free §-anchors are universal, there is no remaining surface of that type for the adversary to probe. The streak terminated when all remaining sub-species were closed structurally.

By contrast, instance-level fixes (patch one stale citation without addressing the citation discipline) produce whack-a-mole: the adversary finds a different instance of the same class in the next pass. The S-025 cycle accumulated 18 META instances across 36 passes because early fixes (Passes 9-28) were often instance-level. The Passes 29-35 gate-completeness streak demonstrates that class-level fixes (ADRs + CI enforcement + exhaustive matrices) break the recurrence loop.

**Rule:** When an adversarial pass surfaces a defect in an enforcement gate (scope gap, vocabulary gap, form-coverage gap, self-description drift), the remediation MUST close the WHOLE CLASS of that defect — not just the specific instance found:
1. Scope gaps: fix the scan scope to match the authoritative governing ADR's mandate; run sibling-gate audit for the same gap-class in the same PR.
2. Vocabulary gaps: make detection vocabulary data-driven (registry-driven) rather than hardcoded list; add to §Form-Coverage Matrix.
3. Form-coverage gaps: add the new form to the §Form-Coverage Matrix with CHECKED/EXEMPT/DEFERRED/NOT-APPLICABLE classification; write a fixture.
4. Self-description drift: convert all version-pinned citations in tooling docstrings to version-free §-anchor form; verify zero remaining pinned citations in .py files.
A fix that patches the specific instance without closing the class guarantees a recurrence in the next adversarial pass. Class-level closure is recognizable by its permanence: after the fix, no instance of that defect class can arise structurally, not just "no instance was found today."

**Convergence implication:** A long gate-completeness streak (7+ passes) is not evidence that the gates cannot converge; it is evidence that prior fixes were instance-level rather than class-level. Once the class is closed, the streak terminates in a single clean pass. The adversary's role in the streak is to probe sub-species until the class boundary is reached. The correct response to a streak is class-level diagnosis ("what is the root defect class?"), not skepticism that convergence is possible.

---

### L-W6-S025-019: A long gate-perimeter streak can mask an unfixed CONTENT defect — perimeter-anchored "clean" passes do not confirm story content unless the pass independently re-derives implementation behavior from source [codified]

**Date:** 2026-05-30
**Severity:** codified (convergence integrity pattern — observed at D-217, Pass 38 RESET)
**Origin:** S-025 adversarial Pass 38 verdict: RESET 2/3 → 0/3. The fixpoint did NOT hold. The THIRD independent fresh-context pass found a genuine in-perimeter S-025 content defect cluster that 37 prior passes AND both preceding "clean" passes (36, 37) had missed. F-S025-ADV38-HIGH-001: AC-001/AC-009/Tasks prose + app.rs:546 doc-comment all claimed "exits cleanly when user presses `q` or `Esc` from Dashboard" — stale since F-S025-ADV2-HIGH-002 deliberately made Esc context-sensitive (only `q` quits from Dashboard; Esc returns to full-screen). F-S025-ADV38-MED-001: the `q`→Quit primary exit path had ZERO test coverage despite being the sole quit key in Dashboard view. Passes 36+37 validated the enforcement-gate perimeter (POL-11/12 fixpoint, ADR self-consistency, cascade cleanliness) but did not re-derive the quit behavior from the binding layers. Pass 38 independently re-derived the quit path from dispatch_key_event → found the stale claim and the coverage gap simultaneously.

**Pattern:** When a convergence counter advances on passes that are "perimeter-anchored" (validating gates, ADR consistency, cascade cleanliness), those passes confirm the enforcement infrastructure is correct but do NOT confirm the story's content unless the adversary independently re-derives implementation behavior from source. A perimeter-anchored pass answers: "does the gate pass?" A content-re-derivation pass answers: "does the code actually do what the spec says?" These are orthogonal questions. Two "clean" perimeter passes can coexist with a stale spec-vs-implementation claim that no prior pass ever traced through the binding layers to verify.

The S-025 content defect had survived because: (1) the stale Esc-quit claim was introduced early in the cycle (F-S025-ADV2-HIGH-002 changed the design before most passes occurred); (2) the 7-pass gate-completeness streak (Passes 29-35) and the subsequent clean passes (36-37) focused adversarial attention on enforcement gates rather than story semantics; (3) the claim was in prose (not in a formal gate-checked constraint), so POL-11 and POL-12 could not detect it; (4) zero test coverage for the `q`→Quit path meant no CI signal existed.

**Rule:** Strict 3-consecutive-CLEAN-with-FRESH-CONTEXT convergence discipline is what makes this class of defect detectable. The "fresh context" and "information asymmetry" requirements are not formalities — they force each pass to re-approach the story without anchoring on the prior pass's attack angles. An adversary who reads "Pass 37 was CLEAN on the enforcement-gate perimeter" and then validates only the enforcement-gate perimeter has NOT provided an independent third confirmation; they have replicated the prior pass's scope. A genuine independent pass re-derives the story's behavioral claims from source (binding tables → dispatch_key_event → actual code path), independent of what the prior pass examined.

Operationally: when dispatching Pass N after two consecutive CLEAN passes, the adversary briefing MUST NOT anchor on "the gates passed cleanly" as the primary lens. The primary lens must be: "does the story content match the implementation?" with the gates serving as supporting evidence only.

**Codified implication:** The 3-consecutive threshold is the minimum that makes implementation-content re-derivation likely to occur at least once in the run. Two clean passes can both be perimeter-anchored. A third pass with a different attack angle (independent re-derivation from binding layers) is what caught this defect. The threshold cannot be reduced without increasing the probability of a perimeter-only run that misses content defects.

**Remediation:** story-writer 8c7d693 (S-025 v1.12→v1.13: AC-001/AC-009/Tasks corrected to reflect q-only quit from Dashboard; Esc context-sensitive behavior documented; STORY-INDEX v5.22→v5.23) + implementer 884401e (app.rs:546 doc-comment fixed; 3 production-dispatch tests added: positive q→Quit PASS, negative Esc-in-Dashboard ≠ Quit, q-in-Overlay ≠ Quit; 32/32 tests; clippy/fmt clean). Both gates PASS 0/0.
