---
document_type: adr
adr_id: "ADR-0007"
status: accepted
date: 2026-05-29
subsystems_affected: []
supersedes: null
superseded_by: null
level: L3
section: "adr"
version: "1.0.1"
producer: vsdd-factory:architect
phase: phase-3-wave-6
timestamp: 2026-05-29T08:00:00Z
inputs:
  [
    product-brief.md,
    architecture/SS-conventions-anti-patterns.md,
    cycles/cycle-001/S-025/adversarial-pass-23.md,
    cycles/cycle-001/S-025/adversarial-pass-24.md,
    STATE.md,
  ]
input-hash: "[pending-compute]"
traces_to: "D-204 (state-manager logs in parallel); architect-escalation tripwire armed Pass 24 D-203"
project: monocle
---

# ADR-0007: Version-Pin Citation Discipline — Semantic Anchors + CI Registry Enforcement

## Status

Accepted

## Context

### The META-Pattern: Empirical Evidence

Seven adversarial review passes across the S-025 convergence cycle uncovered the same
defect species at progressively deeper architectural layers:

| Pass | Layer | Instance |
|------|-------|----------|
| Pass 9 | test-assertion | Vacuous-mirror test assertions |
| Pass 16 | struct-metadata | ADR-0006 audit-table compliance |
| Pass 18 | impl-code | SS-deps-pin-manifest worktree pointers (17 occurrences, 10 files) |
| Pass 22 | spec-filename | Bare-filename anchors (SS-tui-core.md → SS-tui.md; 9 sites) |
| Pass 23 | BC-body→arch-doc | BC-body Architecture Source pins (57+ BCs; D-202.1 BOUNDED) |
| Pass 24 | sibling-artifact-directory | Story `inputs[]` + VP-body architecture citations (story + 14 VPs / 45 occurrences) |
| Pass 25 | code-citation BC-version pin | Worktree code/test BC-version pin doc-citations + story body prose BC-version pins |

**Root cause (invariant across all 7 instances):** A version literal embedded in an
artifact body drifts silently when the source-of-truth document bumps. The version pin
is correct at the moment of authoring and becomes stale as the canonical document
evolves. No enforcement mechanism existed to detect the drift at commit time.

**Codification escalation:** CODIFY-001 grew from 1 sweep category to 13 across this
cycle. Each codification added an enumeration category but did not address the species
root — it described how to manually sweep for drift rather than preventing drift from
occurring.

**Tripwire evidence:** The tripwire was armed at Pass 24 (6th instance) and fired at
Pass 25 (7th instance, within S-025's own perimeter — code-citation BC-version pins).
The species has now appeared in every artifact directory touched by Phase 3 Wave 6.

### Species Root Anatomy

The species arises from the interaction of two legitimate practices:

1. **Citation practice:** Artifacts cite their source documents with version literals
   (e.g., `SS-deps-pin-manifest.md v1.1.17`, `BC-2.06.005 v1.0.5`) to establish a
   conformance claim — "this artifact was verified against source X at version Y."

2. **Independent version evolution:** Source documents bump their versions as other
   stories and patches land. A citation that was current at authoring becomes stale
   after any subsequent bump to the cited document.

The gap: citations in artifact bodies are not verified at commit time against the
canonical version in the cited document's frontmatter. The only detection mechanism is
adversarial review — which is retrospective, runs late in the cycle, and has demonstrated
7 consecutive instances of finding the same species.

### Scope of Affected Artifact Types

Based on the 7-instance record, version-pin citations appear in:

- Worktree code files (`.rs`, `.toml`, `.yml`, `.yaml`) — inline comments citing SS-NNN.md vX.Y.Z
- Story files (`.factory/stories/`) — `inputs[]` frontmatter + body prose citations
- Behavioral contract files (`.factory/specs/behavioral-contracts/`) — Architecture Source
  sections citing SS docs and BC sibling references
- Verification property files (`.factory/specs/verification-properties/`) — body citations
- ADR files — §References and §Source sections (also carry historical-anchor citations)
- SS architecture section files — §Trace body cross-references

## Decision

**Option C (Hybrid A+B) is selected**, refined as follows:

### Adopted Option: C-Refined — Semantic Anchors for New, Registry-Enforced CI Gate for All

**For all NEW artifacts created going forward (post-D-204):**

Use unversioned semantic anchors in artifact bodies. Cite by document ID only — the
version is resolved at review/tooling time, not encoded in the body.

```
Permitted:   BC-2.06.005 §Postconditions
Permitted:   SS-deps-pin-manifest.md §Phase-1-Pins
Permitted:   SS-tui.md §AppMode-State-Machine
Forbidden:   BC-2.06.005 v1.0.6 §Postconditions
Forbidden:   SS-deps-pin-manifest.md v1.2.0
```

**One explicit exception — historical-anchor form (Form 2):**

Citations that establish a historical conformance claim — "this code/spec was written
against document X at version Y, at time T" — MUST use the historical-anchor form
documented in SS-conventions §Citation Discipline (added by this ADR). Historical anchors
are frozen at authoring time and are never updated as the cited document evolves.
This preserves the conformance record without creating drift pressure.

```
Historical anchor (frozen at authoring):
  "implemented against SS-deps-pin-manifest.md v1.2.0 at S-025 authoring time (2026-05-29)"
Active pointer (forbidden in new artifacts):
  "see SS-deps-pin-manifest.md v1.2.0"  ← this form MUST NOT appear in new artifacts
```

**For existing artifacts (pre-D-204):**

Existing artifacts containing active version-pin literals are not migrated all-at-once.
They are migrated opportunistically: when any artifact is touched for another reason
(story update, BC revision, SS-doc bump), the author converts any active version-pin
literals in the SAME edit to unversioned citations. This is a per-touch obligation, not
a big-bang sweep.

**CI enforcement gate (POL-11-version-pin, applying immediately to all artifacts):**

A new CI lint (`monocle-version-pin-freshness`) verifies that every active
version-pin literal in the repository matches the canonical version from the cited
document's frontmatter. An active version-pin literal is one that is NOT in a
historical-anchor form (see §Historical Anchor Classification below).

The CI gate applies to ALL artifact directories:
- `.factory/specs/**/*.md` (BCs, SS docs, stories, VPs, ADRs)
- `crates/**/*.rs` (inline comments)
- `*.toml`, `*.yml`, `*.yaml` (workspace and CI config files)

The CI gate is exempt from:
- `§Trace` sections (historical provenance records)
- Lines annotated with `# version-pin-historical` or `<!-- version-pin-historical -->`
  (explicit historical-anchor annotation; see §Historical Anchor Classification)
- `frontmatter` `version:` field itself (this IS the canonical source)
- `.factory/cycles/` (closed adversarial cycle records; never updated after closure)

### Historical Anchor Classification

A citation is a historical anchor (frozen, exempt from CI staleness check) when it
meets at least ONE of the following:

1. It appears inside a `§Trace` section.
2. It is annotated with `# version-pin-historical` (Rust/TOML/YAML) or
   `<!-- version-pin-historical -->` (Markdown) on the same line.
3. It contains a time qualifier: "at time of", "at S-NNN authoring time",
   "at T-NNN dispatch time", "at spec authoring time", "at time of ratification",
   "at initial authoring", or equivalent unambiguous temporal anchor.

If a citation does not meet any criterion above, it is classified as an active pointer
and subject to the CI freshness check.

### Version-Pin Registry

A machine-readable registry at `.factory/specs/version-pin-registry.yaml` enumerates
the canonical current version for every versioned document in the repository. The CI
lint reads this registry to verify active pointers.

Registry format:

```yaml
# .factory/specs/version-pin-registry.yaml
# Source of truth for canonical current version per artifact.
# Updated by state-manager whenever a versioned document is bumped.
# CI lint reads this file to verify active pointer freshness.
#
# format:
#   <artifact-id>:
#     path: <relative path from .factory/specs/ or workspace root>
#     current_version: "<semver>"
#     last_bump_commit: "<sha>"
#     last_bump_date: "<ISO-8601>"

SS-deps-pin-manifest:
  path: architecture/SS-deps-pin-manifest.md
  current_version: "1.2.0"
  last_bump_commit: "[set at D-204]"
  last_bump_date: "2026-05-29"

SS-tui:
  path: architecture/SS-tui.md
  current_version: "1.8.2"
  last_bump_commit: "[set at D-204]"
  last_bump_date: "2026-05-29"

# ... (full enumeration: devops-engineer Phase 3 deliverable)
```

**Registry update obligation:** Whenever state-manager commits a document version bump
to factory-artifacts, state-manager MUST update `version-pin-registry.yaml` in the
SAME commit. This is a Single-Commit Burst Protocol obligation — registry and document
bump are atomic.

## Rationale

### Why Option C over the alternatives

**Against Option D (accept + quarterly sweep):** The empirical record rules out Option D.
Seven consecutive passes found the same species across 9 active development weeks. A
quarterly cadence would leave 2-4 active development weeks between sweeps each quarter.
The CODIFY-001 enumeration growth (1 → 13 categories) demonstrates that reactive
codification without root-cause elimination is an escalating maintenance cost. Each
category adds cognitive overhead for every reviewer and author; the overhead now
materially impacts adversary cycle duration (13-category sweeps are expensive).

**Against Option A alone (CI enforcement only):** CI enforcement catches drift but does
not prevent it. Under Option A, every document bump still requires authors to manually
propagate the new version to all citing artifacts. The human error rate for this
propagation is empirically high — 57+ BC-body citations in D-202.1, 17 worktree
occurrences in D-197, 45 VP-body occurrences in D-203. CI enforcement reduces the
latency between introduction and detection but does not eliminate the introduction.
Over a multi-year project lifecycle, the per-bump propagation burden compounds.

**Against Option B alone (semantic anchors only):** Pure Option B eliminates new drift
but leaves a large legacy corpus (~200+ existing active version-pin literals across
stories, VPs, BCs, and code). A big-bang migration of 200+ citations introduces
non-trivial risk of mechanical errors and consumes substantial implementation bandwidth
that should go toward feature delivery. The gradual migration in Option C-Refined is
better than either big-bang migration or leaving the legacy corpus permanently excluded
from staleness enforcement.

**For Option C-Refined:** The hybrid correctly layers the two mechanisms:

- The CI gate (Option A component) provides immediate coverage on the legacy corpus
  without requiring migration — it catches drift in existing active pointers before it
  compounds into the next adversarial cycle.
- The semantic anchor discipline (Option B component) prevents new drift introduction
  in all artifacts created from D-204 onward.
- The opportunistic migration converts legacy active pointers as artifacts are touched,
  gradually eliminating CI gate overhead without a disruptive big-bang sweep.
- The historical-anchor classification provides a clean escape valve for citations that
  ARE intentionally frozen — §Trace entries, ratification records, conformance claims —
  preventing the CI gate from incorrectly flagging legitimate historical provenance.

The refinement over the adversary's Option C recommendation is the explicit YAML registry
format (machine-readable for tooling) and the formal historical-anchor classification
(prevents CI false positives on legitimate frozen citations). The adversary correctly
identified the hybrid approach; this ADR specifies it with enough precision for
implementation.

## Consequences

### What is forbidden going forward (post-D-204)

In any artifact NOT in `.factory/cycles/` (closed cycle records) and NOT in a `§Trace`
section:

- **Forbidden:** Active version-pin literals in artifact bodies, e.g.:
  - `SS-deps-pin-manifest.md v1.2.0` (active pointer)
  - `BC-2.06.005 v1.0.6` (active pointer)
  - `inputs: [SS-tui.md v1.8.2]` (active pointer in frontmatter)

- **Permitted:** Unversioned citations, e.g.:
  - `SS-deps-pin-manifest.md §Phase-1-Pins`
  - `BC-2.06.005 §Postconditions`
  - `inputs: [SS-tui.md]`

- **Permitted:** Historical-anchor form, e.g.:
  - `at time of S-025 authoring, SS-deps-pin-manifest.md v1.2.0` (time-qualified)
  - Anything inside a `§Trace` section

### Convention changes in SS-conventions-anti-patterns.md

SS-conventions-anti-patterns.md v1.31.1 → v1.32.0 gains a new §Citation Discipline
section (produced in the same burst as this ADR) codifying the permitted/forbidden forms,
historical-anchor classification, and CI gate contract.

### ADR Registry

ARCH-INDEX.md ADR Registry gains ADR-0007 row (produced in the same burst).

### Migration plan for legacy corpus

| Phase | Scope | Timing |
|-------|-------|--------|
| Immediate (D-204) | Seed `version-pin-registry.yaml` with all currently-versioned SS docs and BC IDs | This burst |
| Opportunistic | Convert active pointers in any artifact touched for other reasons | Per-touch obligation from D-204 onward |
| Wave-gate sweeps | Each wave-gate sweep includes a Category 12 check: "did any story/BC/VP touched in this wave introduce new active pointers?" | Per-wave-gate |
| Phase 5 (formal hardening) | Full corpus migration of remaining active pointers in VPs and BCs | Phase 5 scope |
| Phase 7 (convergence) | Final CI gate clean-run with zero exemptions outside §Trace and `.factory/cycles/` | Phase 7 gate criterion |

### Implementation tasks (dispatched post-D-204)

| Task | Agent | Priority | Scope |
|------|-------|----------|-------|
| POL-11-version-pin CI hook implementation | devops-engineer | HIGH | New pre-commit hook + CI step; reads `version-pin-registry.yaml`; classifies active vs historical citations; fails on stale active pointers |
| `version-pin-registry.yaml` seed population | state-manager | HIGH | Enumerate all currently-versioned SS docs (11 in Document Map) + populate current_version from each file's frontmatter |
| Story-writer template update | story-writer | HIGH | Remove `inputs:` version literals from the story template; update `inputs:` example to use bare filenames |
| BC template update | product-owner | MEDIUM | Remove Architecture Source version literals from BC template; update form guidance |
| CODIFY-001 update | story-writer | MEDIUM | Add Category 12 (new-artifact active-pointer check); retire Categories 8/9/10/11 as "covered by CI gate" once POL-11-version-pin is active in CI |
| VP file opportunistic migration | formal-verifier | LOW | During Phase 5 scope; convert 14 VP files / 45 occurrences from active pointers to unversioned citations |

## Alternatives Considered

**Option A (CI enforcement only):** Insufficient alone — detects but does not prevent.
Per-bump propagation burden compounds over multi-year project lifetime. The 57+ BC cascade
in D-202.1 and 17 worktree occurrences in D-197 demonstrate the propagation workload is
not trivially bounded by CI detection.

**Option B (semantic anchors only, big-bang migration):** Correct direction but wrong
migration strategy. 200+ citations across all artifact types is a multi-day migration that
introduces mechanical error risk and consumes Wave 6/7 bandwidth. Gradual migration in
Option C-Refined achieves the same end state with lower delivery risk.

**Option D (accept + quarterly sweep):** Ruled out by empirical record. 7 instances in
9 weeks means quarterly sweeps leave 2-4 cycles of compounding drift between sweeps.
The CODIFY-001 growth (1 → 13 categories) demonstrates that reactive enumeration
without root-cause elimination escalates indefinitely.

**Option E (architect-derived):** Option C-Refined IS the architect-derived option. The
YAML registry format (vs adversary's implicit central registry), the historical-anchor
formal classification (vs implicit exemption), and the opportunistic migration strategy
(vs the adversary's unspecified migration path) are the architect-specific refinements.

## Implementation Plan

### Immediate (this burst — D-204)

1. Write `ADR-0007` (this file) to `.factory/specs/architecture/adr/`.
2. Update `SS-conventions-anti-patterns.md` v1.31.1 → v1.32.0 with §Citation Discipline
   section covering: permitted/forbidden forms, historical-anchor classification,
   state-manager registry-update obligation.
3. Update `ARCH-INDEX.md` with ADR-0007 row in ADR Registry.

### Next session dispatches

| Priority | Dispatch | Instructions |
|----------|----------|-------------|
| 1 (HIGH) | devops-engineer | Implement `monocle-version-pin-freshness` pre-commit hook. Reads `.factory/specs/version-pin-registry.yaml`. For each `.md`, `.rs`, `.toml`, `.yml`, `.yaml` file in the staged diff, greps for patterns matching `(SS-[a-z-]+\.md|BC-[0-9.]+)\s+v[0-9]+\.[0-9]+(\.[0-9]+)?`. Classifies each match as a historical anchor if ANY ONE of: (a) the line is inside a `§Trace` block, (b) the line contains a `version-pin-historical` annotation, or (c) the line contains a time qualifier ("at time of", "at S-NNN authoring time", "at T-NNN dispatch time", "at spec authoring time", "at time of ratification", "at initial authoring", or equivalent). Any match not meeting at least one of these criteria is classified as active. For each active match, looks up the artifact ID in the registry and compares the cited version to `current_version`. Fails with: `version-pin staleness: <file>:<line> cites <artifact> v<cited> but canonical is v<canonical>`. Add CI step after `cargo clippy` per §CI Wiring ordering. |
| 2 (HIGH) | state-manager | Seed `.factory/specs/version-pin-registry.yaml` with all 11 SS docs from ARCH-INDEX Document Map + BC-INDEX current version. Each entry: artifact ID, path, current_version (from frontmatter), last_bump_commit (from git log), last_bump_date. |
| 3 (HIGH) | story-writer | Update story template (`.factory/templates/` or equivalent): remove version literals from `inputs:` example. Add note: "cite artifact by ID only; no version literals in body prose — see ADR-0007". Update STORY-INDEX template if it carries version examples. |
| 4 (MEDIUM) | product-owner | Update BC template: Architecture Source section — remove `v<version>` from example form. Add note citing ADR-0007. |
| 5 (MEDIUM) | story-writer | Update CODIFY-001 sweep protocol: add Category 12 (new-artifact active-pointer introduction check). Mark Categories 8/9/10/11 as "CI-gated once POL-11-version-pin is active; manual sweep required only before CI gate is live". |

### Migration scope estimate

Based on D-202.1 (57+ BCs), D-203 (14 VPs / 45 occurrences), and Pass 24/25 evidence:

| Artifact type | Estimated active-pointer occurrences | Migration vehicle |
|---------------|---------------------------------------|-------------------|
| Behavioral contracts (`.factory/specs/behavioral-contracts/`) | ~57 BC bodies, 1-3 active pointers each ≈ 100-170 occurrences | Opportunistic per touch + Phase 5 sweep |
| Stories (`.factory/stories/`) | ~33 stories, 4-8 `inputs[]` pins each ≈ 130-265 occurrences | story-writer per wave-gate |
| Verification properties | 14 VPs, 3-5 occurrences each ≈ 45-70 occurrences | Phase 5 formal-verifier sweep |
| Worktree code comments | ~20-40 occurrences (already swept in D-197/D-199) | Opportunistic; most already clean |
| Done-story body prose | ~10 stories × 3-5 occurrences ≈ 30-50 occurrences | Opportunistic |
| **Total estimate** | **~350-550 active-pointer occurrences** | No single migration; gradual |

A tooling script (devops-engineer deliverable alongside the CI hook) should produce
the full inventory from the registry, enabling a one-pass migration if the team
chooses to accelerate.

## §Trace v1.0.1

**Pass 26 internal-consistency correction — F-S025-ADV26-HIGH-001 + LOW-001** (2026-05-29):

- NORMATIVE (F-S025-ADV26-HIGH-001 closure): §Historical Anchor Classification rewritten
  to remove version-monotonicity criterion. Adjudication: Option B selected — at-least-one-of
  contextual marker (§Trace / `version-pin-historical` / time qualifier) is the sole criterion.
  Rationale: (1) version-monotonicity requires git-log lookup for cited doc's historical version
  at §Trace timestamp — disproportionate hook complexity; (2) contextual-marker criterion
  already provides the discriminating signal; (3) ADR-0007 §Decision body's own canonical
  example (lines 121-125) relies solely on a time qualifier, not monotonicity — the §Historical
  Anchor Classification block was internally inconsistent with the §Decision examples in
  the same burst. SS-conventions §Historical Anchor Classification (v1.32.0 at Pass 25
  authoring time) had the correct formulation; this correction aligns ADR-0007 with it.
- NORMATIVE (F-S025-ADV26-LOW-001 closure): §Implementation Plan POL-11 dispatch (Priority 1
  row) adds `.yaml` to extension list. Previously enumerated `.md`, `.rs`, `.toml`, `.yml`
  but omitted `.yaml`, while §CI gate scope (lines 143-146 at Pass 25 authoring time) and
  the registry path itself (`.factory/specs/version-pin-registry.yaml`) both use `.yaml`.
  Mechanical omission corrected.
- NORMATIVE: POL-11 dispatch historical-anchor classification description updated to match
  the at-least-one-of formulation adopted above.
- NORMATIVE: SS-conventions-anti-patterns.md v1.32.0 → v1.32.1 (§Historical Anchor
  Classification §Trace entry added; §Historical Anchor Classification body already correct
  — no body change required).
- Version bump: ADR-0007 v1.0.0 → v1.0.1 (patch: internal-consistency correction, no
  discipline change to the operative rule).

**ADR-0007 initial ratification — D-204 architect-escalation tripwire closure** (2026-05-29T08:00:00Z):

- NORMATIVE: ADR-0007 authored. Decision: Option C-Refined (hybrid semantic anchors + CI
  registry enforcement). Tripwire fired at Pass 25 (7th META-pattern instance, code-citation
  BC-version pin layer) per architect-escalation tripwire armed at D-203.
- NORMATIVE: Codifies citation discipline replacing CODIFY-001 reactive-enumeration strategy.
- Produces concurrent updates (same burst): SS-conventions-anti-patterns.md v1.31.1→v1.32.0
  (§Citation Discipline section added); ARCH-INDEX.md ADR Registry row added.
- NORMATIVE: `version-pin-registry.yaml` SEEDING dispatched to state-manager (next burst).
  devops-engineer POL-11-version-pin hook dispatched. story-writer template update dispatched.
- SE-16d PASS: 2026-05-29T08:00:00Z — initial ratification, no prior chain.
