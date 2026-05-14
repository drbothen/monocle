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
