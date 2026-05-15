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
