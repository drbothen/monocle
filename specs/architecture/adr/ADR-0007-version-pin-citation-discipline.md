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
version: "1.0.6"
producer: vsdd-factory:architect
phase: phase-3-wave-6
timestamp: 2026-05-29T12:00:00Z
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
   (e.g., `SS-deps-pin-manifest.md v1.1.17`, `BC-2.06.005 v1.0.5`) <!-- version-pin-historical: illustrative examples in §Species Root Anatomy --> to establish a
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

**Note — related sub-species (ADR-0008):** Pass 26 and Pass 27 adversarial reviews surfaced
a structurally distinct sub-species at the same species root: structural claims (type names
such as `Vec<SessionState>`, table column counts, enum variant lists) that drift as canonical
Rust types and BC postconditions evolve. This sub-species is NOT covered by POL-11-version-pin
(no `vN.M.P` literal present; detection requires type-name extraction and canonical-source
comparison). ADR-0008 governs structural-claim discipline and defines POL-12. Both ADRs
apply simultaneously; see ADR-0008 §Relationship to ADR-0007.

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

The CI gate applies to the NORMATIVE document classes and is EXEMPT from the EXEMPT
document classes, both defined in §Enforcement Scan Scope. Summary:
- NORMATIVE (scanned): `factory_root/stories/`, `factory_root/specs/` (all subdirs),
  `crates/`, `.github/`, `scripts/` (excl. `scripts/tests/`), root `*.toml/*.yml/*.yaml/*.md`
- EXEMPT (not scanned): `factory_root/cycles/`, `factory_root/plans/`,
  `factory_root/planning/`, `factory_root/code-delivery/`, `factory_root/STATE.md`

The CI gate is additionally exempt from the following within any scanned file:
- `§Trace` sections (historical provenance records)
- Lines annotated with `# version-pin-historical` or `<!-- version-pin-historical -->`
  (explicit historical-anchor annotation; see §Historical Anchor Classification)
- `frontmatter` `version:` field itself (this IS the canonical source)

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

## §Enforcement Scan Scope

This section formally defines the boundary of what POL-11-version-pin scans and
what it exempts. The implementation is `scripts/check_version_pins.py`
`collect_files()`. This section is authoritative; `collect_files()` must match it
exactly. Any deviation in the script is a defect in the script, not a scope change
to this ADR.

### NORMATIVE (POL-11 scans; pins must be fresh or historical-anchored)

**Factory-artifacts tree:**

| Path | Rationale |
|------|-----------|
| `factory_root/stories/` | Active story files carry version-pin literals in `inputs[]` frontmatter and body prose that must stay synchronized with canonical sources as documents evolve. |
| `factory_root/specs/` — all subdirs | Normative artifact tree: includes `architecture/`, `architecture/adr/`, `behavioral-contracts/`, `verification-properties/`, `prd-supplements/`. These are living spec documents with active version-pin citations. |
| `factory_root/specs/prd.md` | Top-level PRD is a normative artifact; includes at `specs/` path. |
| `factory_root/specs/product-brief.md` | Product brief is a normative artifact. |
| `factory_root/specs/dtu-assessment.md` | DTU assessment is a normative artifact. |
| `factory_root/specs/version-pin-registry.yaml` | The registry itself (the CI lint reads this; its own entries must be fresh). |

**Workspace repo tree (always scanned regardless of factory root):**

| Path | Rationale |
|------|-----------|
| `crates/` | Inline code comments citing SS docs and BC versions must stay synchronized. |
| `.github/` | CI workflow files may carry version references. |
| `scripts/` (excluding `scripts/tests/`) | Build scripts may reference versioned artifacts. |
| Root-level `*.toml`, `*.yml`, `*.yaml`, `*.md` | `Cargo.toml`, deny configs, CI root files. |

### EXEMPT (NOT scanned — historical/frozen or living-state)

The following document classes are excluded from POL-11 scanning. All share a
common rationale: applying pin-freshness to frozen historical records or
continuously-rewritten living-state dashboards is semantically wrong.

| Path | Rationale |
|------|-----------|
| `factory_root/cycles/` | Frozen point-in-time records: closed adversarial cycle passes, consistency audit results, convergence proofs. These correctly cite the versions current at authoring; they are SEALED at closure and must never be retroactively updated. This exemption is the precedent for all other exemptions below. |
| `factory_root/plans/` | Point-in-time planning records: adversary-pass transcripts, consistency-audit rounds, investigation reports. Semantically identical to `cycles/` — authored and sealed at the moment of the review pass; version citations correctly document what was current at that moment. The ADV-29 CI fix revealed 1,348 findings here (88% of total); investigation confirmed all are legitimate historical references in frozen records. |
| `factory_root/planning/` | Historical planning session files. Same rationale as `plans/`. |
| `factory_root/code-delivery/` | At-merge PR descriptions and code-delivery records. These are sealed at merge time; the version citations they carry correctly describe the state at delivery. |
| `factory_root/STATE.md` | Living dashboard and log: STATE.md is rewritten by state-manager every burst. It is not a normative spec artifact — nothing traces TO it, it traces FROM it. Keeping STATE.md in scope creates version-race CI fragility: burst edits that bump a spec doc mid-session naturally leave STATE.md with a momentarily stale reference until the burst completes. The §Trace/decisions bulk of STATE.md is already exempt via the §Trace-section criterion; only transient current-state snapshot lines would fire, producing false positives with no remediation path. |

**Existing line-level exemptions within scanned files** (unchanged from §Decision):

- `§Trace` sections in any scanned file (historical provenance records)
- Lines annotated with `# version-pin-historical` or `<!-- version-pin-historical -->` (explicit historical-anchor annotation)
- `frontmatter version:` field itself (this IS the canonical source; not a citation)

### Relationship to cycles/ precedent

The `factory_root/cycles/` exemption was defined in ADR-0007 initial ratification
(D-204). The exemptions for `plans/`, `planning/`, `code-delivery/`, and `STATE.md`
apply the identical rationale: all are records of past state, not normative living
artifacts, and subjecting them to pin-freshness enforcement is semantically incorrect.
`STATE.md` adds a further fragility argument: continuous rewriting creates a
structural version-race that cycles/, plans/, planning/, and code-delivery/ do not
have (those are append-only or sealed).

### Implementation obligation

`scripts/check_version_pins.py` `collect_files()` MUST implement exactly this scope.
The devops-engineer owns the script; when this ADR scope is amended, the script must
be updated in the same burst. The script is the implementation; this ADR is the
authority. They must stay synchronized.

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

In any artifact in the NORMATIVE scan scope (see §Enforcement Scan Scope) and NOT in
a `§Trace` section:

- **Forbidden:** Active version-pin literals in artifact bodies, e.g.:
  - `SS-deps-pin-manifest.md v1.2.0` (active pointer in body prose — must be unversioned)
  - `BC-2.06.005 v1.0.6` (active pointer in body prose — must be unversioned)
  - `inputs: [{path: SS-tui.md, version: "1.8.2"}]` (active pointer in INDEX doc frontmatter — see §inputs[] Provenance Classification below)

- **Permitted:** Unversioned citations, e.g.:
  - `SS-deps-pin-manifest.md §Phase-1-Pins`
  - `BC-2.06.005 §Postconditions`
  - `inputs: [SS-tui.md]` (bare filename; no version literal)

- **Permitted:** Historical-anchor form, e.g.:
  - `at time of S-025 authoring, SS-deps-pin-manifest.md v1.2.0` (time-qualified)
  - `inputs: [{path: SS-tui.md, version: "1.8.2"}]` in an individual STORY file (historical provenance — see §inputs[] Provenance Classification below)
  - Anything inside a `§Trace` section

### inputs[] Provenance Classification

**Decision (human-approved, 2026-05-30, F-S025-ADV30-MED-001 Option A; extended 2026-05-30, closed-rule ratification):**

Individual story files' `inputs[]` frontmatter YAML pins — e.g. `{path: SS-tui.md, version: "1.8.2"}` —
are HISTORICAL PROVENANCE records, not active pointers. They record the spec versions a story was
AUTHORED/DECOMPOSED against at a specific point in time. This is inherently historical by construction:
a story is authored once, and its inputs[] faithfully captures what was current at that moment,
analogous to a §Trace entry. Individual story inputs[] pins are:

- **NOT stale** when the cited document's canonical version advances (the record is frozen at authoring).
- **EXEMPT from POL-11 CI staleness check** in individual story files — the YAML `{path, version}` form
  appearing in `.factory/stories/S-NNN-*.md` is classified HISTORICAL, not active.
- **NOT required to be updated** when a referenced spec bumps, because the historical record is correct.

**Compensating control for not-started stories:** Individual story inputs[] are historical at the time
of story authoring but may become stale relative to the spec at implementation time. The existing
spec-freshness gate (`vsdd-factory:remove-uncertainty` skill + Phase 3 wave dispatch) requires
implementers to verify spec currency before implementing each story. This gate is the compensating
control: story inputs[] records what the story was written against; the implementer gate ensures
the story is implemented against current specs. The historical inputs[] record is not updated when
the gate runs — the gate may produce a story revision with updated body content, but inputs[] remains
the original authoring-time record.

**Closed rule — default HISTORICAL, active set ENUMERATED and CLOSED:**

`inputs[]` version pins are HISTORICAL (authored-against provenance, exempt from POL-11 staleness) BY
DEFAULT for ALL document classes. The only documents whose `inputs[]` is classified ACTIVE are a
CLOSED, ENUMERATED set of living traceability index documents:

> **ACTIVE set (closed):** Any file whose basename matches `*-INDEX.md` (e.g. STORY-INDEX.md,
> BC-INDEX.md, ARCH-INDEX.md, VP-INDEX.md, EVAL-INDEX.md, L2-INDEX.md) PLUS `prd.md`.
> These documents exist specifically to maintain current-state enumeration; their `inputs[]` is a
> live declaration of what they currently reflect, not an authoring-time provenance record. A stale
> `inputs[]` in an index document is a real defect (the index is wrong), not a frozen historical fact.

Every other document class — SS-NNN architecture specs, BC-NNN/BC-HOOK-NNN behavioral contracts,
dependency-graph.md, prd-expansion-scope.md, prd-supplements, verification-property files, ADR files,
and any future document class not explicitly listed in the active set — is classified HISTORICAL and
is exempt from POL-11 inputs[] staleness checks.

**The active set is CLOSED.** Adding a new index document to the active set requires an explicit ADR
amendment (bump ADR-0007, add an §Trace entry documenting the addition). Silent inclusion of a new
document class in the active set without an ADR amendment is not permitted. This closure prevents
the meta-pattern recurrence: under a default-ACTIVE rule, every newly encountered document class
requires individual adjudication, which is itself the classification-recursion the project has fought
across multiple adversarial passes. A default-HISTORICAL + closed-active-set rule means no document
class is ever "unclassified" — the safe default applies automatically.

**Rationale for the default-HISTORICAL direction:** Consistent with Option A (human-approved
2026-05-30): `inputs[]` is honest historical provenance of what an artifact was authored against.
Minimizing churn is correct — a document bumping its own inputs[] every time any cited spec advances
is noise, not signal. Only the index documents, whose literal purpose is current-state reflection,
justify ACTIVE classification.

| Artifact type | inputs[] classification | POL-11 treatment |
|---------------|------------------------|-----------------|
| Individual story files (`stories/S-NNN-*.md`) | HISTORICAL (authored-against provenance) | EXEMPT — not scanned for staleness |
| Living index docs — basename `*-INDEX.md` or `prd.md` (closed set; see above) | ACTIVE POINTER | SCANNED — must match canonical current version |
| All other document classes (SS-*, BC-*, ADR-*, dep-graph, prd-supplements, etc.) | HISTORICAL (authored-against provenance) | EXEMPT — not scanned for staleness |

**POL-11 implementation requirement for inputs[] YAML form:**

The CI gate MUST NOT be blind to the YAML `{path:, version:}` form. The gate must:
1. Detect YAML-form pins: `{path: <artifact>, version: "<semver>"}` in any scanned file's frontmatter.
2. Apply the closed-rule classification:
   - File path matches `stories/S-[0-9]+-*.md` → classify as HISTORICAL → skip staleness check.
   - File basename matches `*-INDEX.md` regex OR equals `prd.md` → classify as ACTIVE → check version against registry.
   - All other files → classify as HISTORICAL → skip staleness check.
3. For active YAML-form pins, fail with: `version-pin staleness: <file>: inputs[].version cites <artifact> v<cited> but canonical is v<canonical>`.

The silent blind spot (YAML form not detected at all) is the defect to close. The handling must be
explicit and intentional: HISTORICAL (skip with rationale) or ACTIVE (check). Never silently
ignored. The classification rule is closed: no new document class is ever "unclassified."

### Convention changes in SS-conventions-anti-patterns.md

SS-conventions-anti-patterns.md v1.31.1 <!-- version-pin-historical: version at ADR-0007 initial ratification time --> → v1.32.0 gains a new §Citation Discipline
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
| Phase 7 (convergence) | Final CI gate clean-run with zero stale active pins in the NORMATIVE scan scope (§Enforcement Scan Scope); EXEMPT document classes remain outside scope by design | Phase 7 gate criterion |

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

### Immediate (D-ADV30 — F-S025-ADV30 remediation burst)

4. Update `ADR-0007` v1.0.4 → v1.0.5 with §inputs[] Provenance Classification policy,
   §Trace v1.0.3 split, pipe-escape fix, and ADR self-consistency discipline.
5. Update `ADR-0008` v1.0.3 → v1.0.4 with §Trace v1.0.2 restoration (mis-inserted entry
   extracted from normative numbered list and placed in correct §Trace section).
6. Update `SS-conventions-anti-patterns.md` with §ADR Authoring Discipline section codifying
   the pre-commit ADR self-consistency checklist (see below).

### Immediate (closed-rule ratification burst)

7. Update `ADR-0007` v1.0.5 → v1.0.6: §inputs[] Provenance Classification closed rule
   (default HISTORICAL, active set = `*-INDEX.md` OR `prd.md`, set is CLOSED). Pattern B
   devops spec updated to match the three-branch exhaustive classification. `version-pin-registry.yaml`
   ADR-0007 entry bumped to v1.0.6. ARCH-INDEX.md Note for ADR-0007 updated.

**ADR Self-Consistency Checklist (pre-commit discipline — codified D-ADV30):**

The §Trace-escaping-into-normative-content defect has appeared 4 times (ADR-0006 indirect
path; ADR-0007 Pass 26 HIGH-001; ADR-0008 Pass 28 MED-002; ADR-0008 Pass 30 HIGH-001).
The pattern: §Trace entry prose is accidentally inserted inside a normative section (numbered
list, table, or body paragraph) rather than in a dedicated `## §Trace vN.M.P` section.

Before committing any ADR, the author MUST verify:

1. **§Trace section header ↔ entry label match:** Every `## §Trace vN.M.P` header must
   contain an entry labeled `**N.M.P**` or `**vN.M.P**` (or a descriptive title). No
   `## §Trace v1.0.2` section with a `**1.0.3**` labeled entry.
2. **No §Trace prose inside normative sections:** Grep the file for `**[0-9]\+\.[0-9]\+`
   (bold version labels). Every match must be either (a) inside a `## §Trace` section, or
   (b) inside a code block, or (c) an annotated historical-anchor. If a match is inside a
   numbered list, table, or body prose, it has escaped and must be extracted.
3. **Table cell pipe escape:** Any regex pattern inside backticks that contains a `|`
   alternation operator must escape it as `\|` to prevent `validate-table-cell-count` from
   counting it as a structural pipe.
4. **Numbered list continuity:** After any ADR edit, verify numbered lists read 1, 2, 3, ...
   without gaps. A gap (1, then 3 with no 2) indicates an insertion removed item 2's text
   but left its number, or a §Trace entry consumed a list item's position.
5. **Line-level self-references verified:** Before citing a specific line number in the same
   file (e.g., "see lines 121-125"), re-verify those lines exist and contain the referenced
   content. Off-by-N defects are a recorded failure mode (ADR-0008 Pass 28 MED-002).

This checklist is added to `SS-conventions-anti-patterns.md §ADR Authoring Discipline` in
the same burst. Architect runs it mentally before every ADR write; the conventions doc
serves as the durable reference.

### Next session dispatches

| Priority | Dispatch | Instructions |
|----------|----------|-------------|
| 1 (HIGH) | devops-engineer | Implement `monocle-version-pin-freshness` pre-commit hook (`scripts/check_version_pins.py`). Reads `.factory/specs/version-pin-registry.yaml`. **Pattern A — prose form:** For each `.md`, `.rs`, `.toml`, `.yml`, `.yaml` file in the staged diff, greps for patterns matching `(SS-[a-z-]+\.md\|BC-[0-9.]+)\s+v[0-9]+\.[0-9]+(\.[0-9]+)?`. Classifies each match as a historical anchor if ANY ONE of: (a) the line is inside a `§Trace` block, (b) the line contains a `version-pin-historical` annotation, or (c) the line contains a time qualifier ("at time of", "at S-NNN authoring time", "at T-NNN dispatch time", "at spec authoring time", "at time of ratification", "at initial authoring", or equivalent). Any match not meeting at least one of these criteria is classified as active. **Pattern B — YAML frontmatter form:** Additionally detect YAML-form pins `{path: <artifact>, version: "<semver>"}` in file frontmatter. Apply the closed-rule classification per §inputs[] Provenance Classification: (a) if the containing file path matches `stories/S-[0-9]+-*.md` → classify HISTORICAL → skip; (b) if the containing file's basename matches `*-INDEX\.md` regex OR equals `prd.md` → classify ACTIVE → check version against registry; (c) all other files → classify HISTORICAL → skip. The three branches are exhaustive and CLOSED — no file falls through to "unclassified." The gate must never silently skip the YAML form — handling must be explicit (HISTORICAL or ACTIVE, never unhandled). For each active match (Pattern A or B), looks up the artifact ID in the registry and compares the cited version to `current_version`. Fails with: Pattern A: `version-pin staleness: <file>:<line> cites <artifact> v<cited> but canonical is v<canonical>`. Pattern B: `version-pin staleness: <file>: inputs[].version cites <artifact> v<cited> but canonical is v<canonical>`. Add CI step after `cargo clippy` per §CI Wiring ordering. |
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

## §Trace v1.0.6

**inputs[] Provenance Classification — closed-rule ratification closing long-tail over-flagging** (2026-05-30):

- NORMATIVE: §Consequences §Story inputs[] Historical Provenance (v1.0.5 name) renamed to
  §inputs[] Provenance Classification and substantially extended. The v1.0.5 "Index document boundary decision"
  sub-section described two artifact classes (individual stories = HISTORICAL; living index docs =
  ACTIVE) but left ~92 `inputs[]` pins across unclassified document classes (SS-* specs, BC-*
  contracts, BC-HOOK-* contracts, dependency-graph, prd-expansion-scope, prd-supplements, etc.)
  subject to the v1.0.5 conservative default of ACTIVE, producing over-flagging in POL-11.
- NORMATIVE: Closed rule codified: `inputs[]` version pins are HISTORICAL BY DEFAULT for ALL
  document classes. The ACTIVE set is ENUMERATED and CLOSED: files whose basename matches
  `*-INDEX.md` OR equals `prd.md`. Current members: STORY-INDEX.md, BC-INDEX.md, ARCH-INDEX.md,
  VP-INDEX.md, EVAL-INDEX.md, L2-INDEX.md, prd.md. No other document class is ACTIVE.
- NORMATIVE: The active set is CLOSED — adding a new member requires an explicit ADR amendment.
  This closes the classification recursion: under default-ACTIVE, every new document class
  requires individual adjudication (the meta-pattern the project has fought across multiple
  adversarial passes). Under default-HISTORICAL + closed-active-set, no class is ever
  "unclassified." Consistent with Option A (human-approved 2026-05-30): inputs[] is honest
  historical provenance; minimize churn.
- NORMATIVE: §Next session dispatches Priority 1 Pattern B devops spec updated to match the
  closed rule: three exhaustive branches — (a) `stories/S-[0-9]+-*.md` → HISTORICAL; (b)
  basename matches `*-INDEX\.md` or equals `prd.md` → ACTIVE; (c) all other files → HISTORICAL.
  No file can fall through to "unclassified."
- NORMATIVE: §Consequences table updated with third row: "All other document classes (SS-*, BC-*,
  ADR-*, dep-graph, prd-supplements, etc.) → HISTORICAL → EXEMPT."
- NORMATIVE: Version bump 1.0.5 → 1.0.6.
- SE-16d PASS: 2026-05-30 > chain high-water 2026-05-30 (sequential same-day patch).

## §Trace v1.0.5

**F-S025-ADV30 remediation — inputs[] historical provenance policy + ADR self-consistency discipline** (2026-05-30):

- NORMATIVE (F-S025-ADV30-MED-001 closure): §Consequences §What is forbidden amended to correctly
  classify individual story `inputs[]` YAML pins as HISTORICAL PROVENANCE (not forbidden active pointers).
  The v1.0.4 example `inputs: [SS-tui.md v1.8.2] (active pointer in frontmatter)` was WRONG under
  Option A (human-approved 2026-05-30): story inputs[] are authored-against records, historically
  frozen by construction, and NOT stale when canonical advances. The example has been corrected to
  distinguish individual story files (HISTORICAL) from living index docs (ACTIVE).
- NORMATIVE (F-S025-ADV30-MED-001 closure): New §Consequences sub-section §Story inputs[] Historical
  Provenance added. Defines: (a) individual story inputs[] = historical provenance, EXEMPT from POL-11;
  (b) living index doc inputs[] = active pointers, SCANNED; (c) compensating control (remove-uncertainty
  gate at implementation time); (d) boundary decision rationale; (e) POL-11 YAML form detection
  requirement — the gate must not be silent/blind to the YAML `{path:, version:}` form.
- NORMATIVE (F-S025-ADV30-MED-001 closure): §Implementation Plan Priority 1 devops dispatch updated
  with Pattern B (YAML frontmatter form) detection specification: detect `{path:, version:}` in
  frontmatter, apply active-vs-historical classification per boundary decision, never silently skip.
- NORMATIVE (F-S025-ADV30-HIGH-001 closure): §Trace v1.0.3 section created (was missing — the `**1.0.3**`
  entry was incorrectly slotted under the v1.0.2 header, producing a header-vs-label mismatch). §Trace
  chain now correctly reads: v1.0.5, v1.0.4, v1.0.3, v1.0.2, v1.0.1.
- NORMATIVE (F-S025-ADV30-LOW-001 closure): §Implementation Plan Priority 1 regex corrected from
  unescaped `|` to `\|` inside backtick regex pattern (pre-existing since v1.0.0; first detected by
  ADV-30 comparison against ADR-0008 §Why Structural Claims Are Distinct table).
- NORMATIVE (TASK 4 — tripwire/protocol-improvement): §Implementation Plan Immediate items extended
  with ADR self-consistency pre-commit checklist discipline. The recurring §Trace-escaping-normative-
  content defect class has appeared 4 times (ADR-0006 indirect, ADR-0007 Pass 26, ADR-0008 Pass 28,
  ADR-0008 Pass 30). Discipline codified in §Implementation Plan (D-ADV30 immediate items) and
  SS-conventions-anti-patterns.md §ADR Authoring Discipline.
- NORMATIVE: Version bump 1.0.4 → 1.0.5.
- SE-16d PASS: 2026-05-30 > chain high-water 2026-05-30T00:00:00Z — same calendar day, sequential pass.

## §Trace v1.0.4

**ADV-29 scope ratification — formal §Enforcement Scan Scope added** (2026-05-30T00:00:00Z):

- NORMATIVE: §Enforcement Scan Scope section added between §Decision and §Rationale.
  Formally defines the NORMATIVE vs EXEMPT document classes for POL-11 scanning.
  NORMATIVE: `factory_root/stories/`, `factory_root/specs/` (all subdirs), and the
  repo tree (`crates/`, `.github/`, `scripts/` excl. `scripts/tests/`, root config files).
  EXEMPT (new — extends the existing `cycles/` exemption with the same rationale):
  `factory_root/plans/`, `factory_root/planning/`, `factory_root/code-delivery/`,
  and `factory_root/STATE.md`. Rationale: all four are frozen point-in-time records
  or continuously-rewritten living-state dashboards that correctly cite versions at
  authoring time; pin-freshness enforcement is semantically wrong for these classes.
  ADV-29 revealed 1,348 findings in the absence of this scope definition (88% in
  `plans/`); investigation confirmed all were legitimate historical references.
- NORMATIVE: §Enforcement Scan Scope §Implementation obligation paragraph added:
  devops-engineer obligation to keep `scripts/check_version_pins.py collect_files()`
  synchronized with this ADR.
- NORMATIVE: Version bump 1.0.3 → 1.0.4 (normative scope addition).
- SE-16d PASS: 2026-05-30T00:00:00Z > chain high-water 2026-05-30 (monotonic; v1.0.3
  was a same-day patch with no explicit timestamp; this entry establishes chain).

## §Trace v1.0.3

**1.0.3** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers and time qualifiers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at authoring time. No normative content changed.

## §Trace v1.0.2

**D-206 ADR-0008 cross-reference addition — structural-spec drift tripwire closure** (2026-05-29T12:00:00Z):

- NORMATIVE (sweep-wider L-W6-S025-007): §Scope "Note — related sub-species (ADR-0008)" paragraph
  added. Documents the structural-claim sub-species surfaced at Pass 26 (module-doc column table)
  and Pass 27 (story-body type name), explains why POL-11 does not cover it (no `vN.M.P` literal),
  and forward-references ADR-0008 + POL-12. No decision change to ADR-0007 scope or POL-11.
  ADR-0007 §Decision, §Rationale, §Consequences, and §Implementation Plan are unchanged.
- Version bump: ADR-0007 v1.0.1 → v1.0.2 (patch: informational scope-navigation note; no
  operative rule change).
- SE-16d PASS: 2026-05-29T12:00:00Z > chain high-water 2026-05-29T10:00:00Z (monotonic).

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
