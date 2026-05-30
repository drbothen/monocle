---
document_type: adr
adr_id: "ADR-0008"
status: accepted
date: 2026-05-29
subsystems_affected: []
supersedes: null
superseded_by: null
level: L3
section: "adr"
version: "1.0.4"
producer: vsdd-factory:architect
phase: phase-3-wave-6
timestamp: 2026-05-29T12:00:00Z
inputs:
  [
    product-brief.md,
    architecture/SS-conventions-anti-patterns.md,
    architecture/adr/ADR-0007-version-pin-citation-discipline.md,
    cycles/cycle-001/S-025/adversarial-pass-26.md,
    STATE.md,
  ]
input-hash: "[pending-compute]"
traces_to: "D-206 (2nd structural-spec drift tripwire — Pass 26 MED-001 + Pass 27 MED-001 — architect strategic dispatch per Task #9 m.6)"
project: monocle
---

# ADR-0008: Structural-Claim Discipline — Canonical Shape Anchors + POL-12 Detection

## Status

Accepted

## Context

### Species Root (Shared with ADR-0007)

ADR-0007 (version-pin citation discipline) addressed the literal-version-pin sub-species of
a broader META-pattern: authoring-time documentation drifts as the canonical source of truth
evolves. ADR-0007 defines a version-pin as a literal `vN.M.P` string embedded in an artifact
body; its POL-11 CI gate detects stale literals via regex and a version-pin registry.

Two adversarial passes during the S-025 convergence cycle surfaced a **distinct sub-species**
at the same species root — structural claims — with a detection mechanism that is categorically
different from version-pin literals:

### Empirical Evidence — Two Structural-Spec Drift Instances

| Pass | Layer | Instance | Status |
|------|-------|----------|--------|
| Pass 26 | `crates/monocle-tui/src/ui/sessions_panel.rs:7-16` — module-level doc-comment markdown table | Table listed 6 columns (Icon, Project, Status, Tokens, Cost, Uptime); canonical BC-2.06.005 PC-2 requires 7 columns starting with Session ID | CLOSED via implementer 2d1188f |
| Pass 27 | `.factory/stories/S-025-tui-skeleton-sessions.md:144,228` — Tasks checklist + Downstream Consumer Contract | Cites `Vec<SessionState>` for `App.sessions` field; canonical SS-tui.md:845 + BC-2.06.005 + production app.rs all use `Vec<EnrichedSession>` | PENDING story-writer fix |

**Layer escalation pattern:** Pass 26 found structural drift at the worktree code-comment layer
(module doc table). Pass 27 found the same species at the story-body spec layer (Tasks checklist
+ Downstream Consumer Contract code block). This mirrors the escalation pattern that motivated
ADR-0007 (Pass 18 worktree → Pass 23 BC-body → Pass 24 sibling-artifact-directory → Pass 25
code-citation). The orchestrator's Task #9 m.6 tripwire, armed at 1 instance (Pass 26), was
configured to fire at 2 instances based on the visible escalation pattern. The tripwire fired
correctly at Pass 27.

### Why Structural Claims Are Distinct from Version-Pin Literals

The two sub-species share the same species root but require fundamentally different detection:

| Property | Version-pin literals (ADR-0007) | Structural claims (ADR-0008) |
|----------|--------------------------------|-----------------------------|
| Pattern shape | `(SS-[a-z-]+\.md\|BC-[0-9.]+)\s+v[0-9]+\.[0-9]+` | Markdown table column counts; Rust type identifiers in prose/code-blocks; postcondition counts; enum variant lists |
| Detection approach | Regex match + version-registry lookup | Parse markdown table structure; extract type names; compare against canonical BC postcondition frontmatter |
| Staleness signal | Cited version != canonical current version | Cited column count/type name != canonical postcondition PC-N specification |
| False-positive risk | Low (literal version strings rarely appear legitimately outside historical anchors) | Medium (type names and table shapes appear in many contexts; require canonical-source lookup) |
| Implementation tool | Pre-commit hook regex (`monocle-version-pin-freshness`) | CI script with markdown table parser + type-name extractor |

Bundling POL-12 structural-claim detection into POL-11's pre-commit regex hook would produce
an incoherent implementation: two unrelated pattern classes in one script, each requiring
different parsing strategies, with no natural shared abstraction. ADR-0008's POL-12 is a
separate, purpose-built CI step that operates after `cargo test` (structural claim verification
requires knowledge of the compiled codebase's canonical types, unlike regex-based ADR-0007).

### Scope of Affected Structural-Claim Types

Based on the two-instance empirical record, structural claims appear in:

**Type 1 — Table shape claims:** Markdown tables in code-comment doc strings (`//! ` blocks)
or story body sections that enumerate column names, postcondition counts, enum variant lists,
or field inventories. A table with N columns is a structural claim that the canonical source
has N such items.

**Type 2 — Type identifier claims:** Prose or code-block references to a Rust type name
(`Vec<SessionState>`, `VecDeque<PromptModal>`, `AppMode::Overlay`) as the type of a field,
parameter, or return value. These appear in:
- Story Tasks checklists (`impl App struct with fields: sessions: Vec<...>`)
- Story Downstream Consumer Contract code blocks (`pub sessions: Vec<...>`)
- BC postcondition prose (`App.sessions is populated as Vec<...>`)
- Architecture section prose (`App.sessions: Vec<...>`)

**Type 3 — Count claims (anticipated, not yet empirically observed):** Prose that asserts a
count of items ("BC-2.06.005 has 4 postconditions", "7 columns", "5 hook endpoints"). A
count claim drifts when postconditions are added or removed in the canonical BC.

## Decision

**Option B is selected: Ratify ADR-0008 as a distinct architectural record for
structural-claim discipline, separate from ADR-0007 (version-pin citation discipline).**

### Rationale for Option B over the alternatives

**Against Option A (extend ADR-0007 §Scope):** ADR-0007's §Decision, §Rationale, and
§Implementation Plan are all organized around version-pin literals and the version-pin
registry. Extending its scope to cover structural claims would require inserting a second,
unrelated detection algorithm (markdown table parser + type-name extractor vs. regex) into
the same implementation plan. The cognitive coherence of ADR-0007 as a self-contained design
record for the literal-pin species would be lost. Option A risks turning ADR-0007 into a
catch-all for documentation-drift species, making both the record and the implementation
harder to reason about.

**Against Option C (hybrid cross-reference):** Option C defers the detection algorithm
to ADR-0008 while having ADR-0007 note the related species in §Scope. This is the correct
relationship to document — but it still produces two ADRs. Given that two ADRs are required
in any non-D option, the question is whether ADR-0007 §Scope should be amended (Option C)
or whether ADR-0007 remains unchanged and ADR-0008 stands alone with cross-references to
ADR-0007 (Option B). Option B is preferable: amending ADR-0007 §Scope creates a §Trace
entry on ADR-0007 for something ADR-0007 didn't actually decide. ADR-0008's own §Context
(this section, above) explicitly documents the shared species root and why ADR-0007 does
not cover structural claims. This is a cleaner factoring.

**Against Option D (defer, wait for 3rd instance):** The Task #9 m.6 tripwire was explicitly
armed at "2 more structural-spec drift instances → dispatch architect." Pass 27 delivers
that 2nd instance. The tripwire condition is met. Deferring under a stricter S-7.02
3-instance reading contradicts the tripwire's explicit arm condition. More importantly: the
2-instance escalation pattern (code-comment → story-body) is a visible signal that the
species is spreading across artifact layers, exactly as the literal-pin species did across
7 passes. The production-grade default is to codify discipline at the first credible
recurrence signal, not at the N-th occurrence. Two instances at two distinct artifact layers
is a credible recurrence signal.

**For Option B:** Distinct species, distinct detection algorithm, distinct ADR, distinct
policy ID (POL-12). Clean separation. ADR-0007 remains unmodified and coherent for its
target species. ADR-0008 carries the full rationale, classification, and implementation
contract for structural-claim discipline.

### Adopted Policy: POL-12-structural-claim (applying immediately post-D-206)

**For all NEW artifacts created going forward (post-D-206):**

When authoring a structural claim in any artifact body, the author MUST anchor the claim
to the canonical source by using one of the following forms:

**Permitted — Source-anchored claim:**
```
// Per BC-2.06.005 §Postconditions PC-2: Session ID | Project | Status | Tokens | Cost | Uptime | Drop (7 columns)
pub sessions: Vec<EnrichedSession>  // per SS-tui.md §App-struct canonical field
```

**Permitted — Unqualified form (for types/values with stable, unambiguous canonical sources):**
```
pub sessions: Vec<EnrichedSession>
```
When the type name matches the canonical source exactly, no additional annotation is required.
The canonical source is SS-tui.md §App struct for `App` fields; BC postconditions for
column/count claims in doc-comments; monocle-core types module for type names.

**Forbidden — Stale structural claim:**
```
sessions: Vec<SessionState>                     ← wrong type; canonical is EnrichedSession
// Icon | Project | Status | Tokens | Cost | Uptime  ← 6 columns; canonical PC-2 has 7
```

The distinction from ADR-0007: this policy governs **what the claim says** (type name,
column count, field name), not **what version a document is at** (vN.M.P literal).

**For existing artifacts (pre-D-206):**

Existing artifacts containing stale structural claims are remediated opportunistically:
when any artifact is touched for another reason, verify its structural claims against
canonical sources in the same edit. This mirrors ADR-0007's opportunistic migration
strategy for legacy active-pointer literals.

**CI enforcement gate (POL-12-structural-claim, Phase 3 deliverable):**

A new CI script (`monocle-structural-claim-check`) verifies structural claims in
spec artifacts against canonical sources. The script operates in two phases:

**Phase 1 — Type-identifier extraction (story + BC files):**

Scan `.factory/stories/` and `.factory/specs/behavioral-contracts/` for Rust type
identifiers in Tasks checklists, Downstream Consumer Contract code blocks, and
postcondition prose. For each `App.sessions` / `App.<field>` reference, compare the
cited type against the canonical `App` struct definition in `SS-tui.md §App struct`.

Detection pattern (story Tasks + Consumer Contract code blocks):
```
grep -n 'Vec<\|VecDeque<\|Option<' .factory/stories/*.md | grep -v '§Trace'
```
For each match, extract the type argument and verify against canonical source.

**Phase 2 — Markdown table shape extraction (worktree doc-comments):**

Scan `crates/**/*.rs` for module-level doc-comment markdown tables (`//! | ... |` lines).
For each table found adjacent to a `BC-N.NN.NNN` reference (same doc-comment block),
extract the column count and verify against the cited BC's postcondition PC-N column list.

Detection pattern:
```
grep -n '//! |' crates/**/*.rs | grep -E '\|.*\|.*\|'
```
Compare column count against the cited BC's frontmatter `postconditions:` field or
the canonical column list in the BC's PC-N postconditon body.

**Canonical source registry (structural claims):**

> **Self-application:** This §Canonical Source Registry table is itself subject to POL-12. Stale entries (citing the wrong line range, deprecated section anchors, or removed canonical sources) will be detected by POL-12 against the cited canonical document's actual content. The architect dispatch in §Implementation Plan row 4 ("When a new canonical type is added to `App` struct in a future story, add it to the canonical source registry table") explicitly includes registry-maintenance as a POL-12 closure dependency.

| Structural claim type | Canonical source | Lookup method |
|-----------------------|-----------------|---------------|
| `App` struct field types | `SS-tui.md §App struct` (lines 833-864) | Read field declarations; compare cited type |
| Sessions panel column list | `BC-2.06.005 §Postconditions PC-2` | Read PC-2 column table; count columns |
| `AppMode` variants | `monocle-core::tui::AppMode` enum | Read enum definition; compare variant list |
| `Action` variants | `monocle-core::tui::Action` enum | Read enum definition; compare variant list |
| Hook endpoint count | `BC-HOOK-007 §Postconditions PC-1` | Read endpoint enumeration; count |

**CI gate scope (structural claim check):**

The CI gate applies to:
- `.factory/stories/*.md` — Tasks checklists + Downstream Consumer Contract code blocks
- `.factory/specs/behavioral-contracts/**/*.md` — postcondition prose
- `crates/**/*.rs` — module-level doc-comment tables

Exempt from the CI gate:
- `.factory/cycles/` — closed adversarial cycle records; sealed at closure
- `§Trace` sections — provenance records; not structural claims about current behavior
- Lines annotated with `<!-- structural-claim-historical -->` (explicit historical-anchor)

**Scope note:** POL-12's scan is narrower than POL-11's by design. POL-12 Phase 1
targets `.factory/stories/*.md` and `.factory/specs/behavioral-contracts/**/*.md`
specifically; it does NOT scan `plans/`, `planning/`, `code-delivery/`, or `STATE.md`.
Those paths do not contain the story-Tasks and Consumer-Contract structural claim
forms that POL-12 detects, so no additional exemptions are needed for those directories.
ADR-0007 §Enforcement Scan Scope formally defines the broader POL-11 exemptions
(including `plans/`, `planning/`, `code-delivery/`, and `STATE.md`) that were added
in v1.0.4 to address the ADV-29 scope issue. The two policies have independent
`collect_files()` implementations and must be kept in sync with their respective ADRs.

### Historical Anchor Classification for Structural Claims

A structural claim is a historical anchor (frozen, exempt from CI check) when it
meets at least ONE of:

1. It appears inside a `## §Trace` section.
2. It is annotated with `<!-- structural-claim-historical -->` on the same line or
   the adjacent line.
3. It contains a time qualifier establishing this as a record of past state:
   "at S-NNN authoring time", "at T-NNN dispatch time", "as of v1.0.0", or equivalent.

If a structural claim does not meet any criterion above, it is classified as an active
claim subject to the CI check.

## Consequences

### What is forbidden going forward (post-D-206)

In any artifact NOT in `.factory/cycles/` and NOT in a `## §Trace` section:

**Forbidden:** Active structural claims with incorrect type/count/shape:
- `sessions: Vec<SessionState>` (wrong type; canonical is `Vec<EnrichedSession>`)
- `// Icon | Project | Status | ...` (6 columns; canonical BC-2.06.005 PC-2 has 7)

**Required:** Claims must match the canonical source exactly. When in doubt, read the
canonical source (SS-tui.md §App struct for `App` fields; the relevant BC postcondition
for column lists) before authoring.

### Convention changes in SS-conventions-anti-patterns.md

SS-conventions-anti-patterns.md v1.32.1 <!-- version-pin-historical: version at ADR-0008 initial ratification time --> → v1.32.2 gains a new §Structural-Claim
Discipline section (produced in the same burst as this ADR) codifying the
permitted/forbidden forms, historical-anchor classification, canonical source registry,
and CI gate contract.

### Relationship to ADR-0007

ADR-0007 governs version-pin literal citations (the `vN.M.P` form). ADR-0008 governs
structural claims (type names, column counts, variant lists). Both address the same species
root — authoring-time documentation that drifts as canonical sources evolve — but require
different detection mechanisms:

| Concern | Governed by | CI gate |
|---------|------------|---------|
| Version-pin literal staleness | ADR-0007 | POL-11 `monocle-version-pin-freshness` |
| Structural claim staleness | ADR-0008 | POL-12 `monocle-structural-claim-check` |

Neither ADR supersedes the other. Both apply simultaneously. A single artifact may
contain both version-pin literals (flagged by POL-11) and structural claims (flagged
by POL-12).

### ADR Registry

ARCH-INDEX.md ADR Registry gains ADR-0008 row (produced in the same burst).

### Migration plan for legacy corpus

| Phase | Scope | Timing |
|-------|-------|--------|
| Immediate (D-206) | Enumerate known structural-claim sites: S-025:144,228 (Vec<SessionState>); S-028:63,147 (cross-story propagation, deferred per BC-5.39.002 PC2) | This burst: story-writer fixes S-025 in parallel |
| Opportunistic | Convert stale structural claims in any artifact touched for other reasons | Per-touch obligation from D-206 onward |
| Wave-gate sweeps | Each wave-gate sweep includes a structural-claim spot-check: "did any story/BC touched in this wave introduce a stale type identifier or column count?" | Per-wave-gate |
| Phase 5 (formal hardening) | Full corpus scan of remaining structural claims in BCs and VPs | Phase 5 scope |
| Phase 7 (convergence) | Final CI gate clean-run with zero exemptions outside §Trace and `.factory/cycles/` | Phase 7 gate criterion |

## Alternatives Considered

**Option A (extend ADR-0007 §Scope):** Rejected. ADR-0007 is coherent around the
version-pin literal detection algorithm (regex + registry). Extending its scope to cover
structural claims would require adding a second, incompatible detection algorithm to the
same implementation plan. The cognitive coherence of the ADR as a decision record and the
CI hook as an implementation would both degrade. See §Decision rationale above.

**Option C (hybrid: ADR-0007 §Scope cross-reference + ADR-0008 for detection):** The
cross-reference value is fully achieved by ADR-0008 §Context and ADR-0008 §Relationship
to ADR-0007. Amending ADR-0007 §Scope produces a §Trace entry on ADR-0007 for something
ADR-0007 didn't decide — a documentation anti-pattern. Option B achieves the same
cross-referencing without modifying ADR-0007.

**Option D (defer to 3rd instance):** Rejected. Task #9 m.6 tripwire was armed at
"2 more structural-spec drift instances." The 2-instance condition is met with Pass 26 +
Pass 27. The layer-escalation pattern (code-comment → story-body) matches the literal-pin
escalation (impl-code → BC-body → sibling-artifact) that justified ADR-0007. Deferring
here contradicts the explicit tripwire condition and the production-grade default.

## Implementation Plan

### Immediate (this burst — D-206)

1. Write ADR-0008 (this file).
2. Update SS-conventions-anti-patterns.md v1.32.1 <!-- version-pin-historical: version at ADR-0008 initial ratification time --> → v1.32.2 with §Structural-Claim
   Discipline section.
3. Update ARCH-INDEX.md with ADR-0008 row in ADR Registry.
4. Story-writer fixes S-025:144,228 `Vec<SessionState>` → `Vec<EnrichedSession>` in
   parallel (tactical fix; story-writer parallel dispatch per three-agent strategy).

### Next session dispatches

| Priority | Dispatch | Instructions |
|----------|----------|--------------|
| 1 (HIGH) | devops-engineer | Implement `monocle-structural-claim-check` CI script. Phase 1 scope: (a) scan `.factory/stories/*.md` Tasks checklists + Downstream Consumer Contract blocks for `Vec<`, `VecDeque<`, `Option<` patterns; extract type arguments; compare against SS-tui.md §App struct canonical declarations; fail on mismatch with: `structural-claim mismatch: <file>:<line> cites <App.field> as <cited-type> but canonical SS-tui.md §App struct declares <canonical-type>`. Phase 2 scope (deferred to Phase 5): module-level doc-comment table shape extraction from `crates/**/*.rs`. Add CI step after `cargo test` per §CI Wiring step ordering in SS-conventions. |
| 2 (MEDIUM) | story-writer | Sweep all in-flight + Wave 6 stories for structural claims about `App` field types. For each story that mentions `App.sessions`, `App.events`, `App.overlay_stack`, `App.mode`, `App.drop_counter`: verify type matches SS-tui.md §App struct canonical declaration. Fix any mismatch in same wave-gate sweep. S-028 lines 63+147 are the known deferred instance (deferred per BC-5.39.002 PC2 cross-story; fix in wave-gate sweep). |
| 3 (MEDIUM) | story-writer | Add §Structural-Claim Discipline to story template: "App struct field types MUST match SS-tui.md §App struct declaration exactly. Before citing `Vec<X>` in Tasks or Consumer Contract blocks, read SS-tui.md §App struct and confirm type name." |
| 4 (LOW) | architect | If a new canonical type is added to `App` struct in a future story, add it to the canonical source registry table in ADR-0008 §CI enforcement gate and SS-conventions §Structural-Claim Discipline. This is a per-story-cycle obligation for the implementing architect. |

### Cross-story propagation (S-028)

S-028 lines 63 + 147 carry the same `Vec<SessionState>` drift (surfaced at Pass 27
as cross-story propagation). Per BC-5.39.002 PC2, cross-story structural-claim fixes
are deferred to wave-gate sweep (not blocking S-025 convergence). Story-writer is
dispatched to fix S-028 in the next wave-gate sweep post-S-025 merge.

## §Trace v1.0.4

**F-S025-ADV30-HIGH-001 internal-consistency correction — §Trace-escaping-into-normative-content** (2026-05-30):

- NORMATIVE: §Historical Anchor Classification for Structural Claims numbered list (items 1/2/3)
  restored to correct sequence. The `**1.0.2**` §Trace entry was mis-inserted between items 1 and 2
  of the normative numbered list, corrupting the list structure and burying the §Trace prose inside
  a normative section. The entry has been extracted from the normative list and placed in its correct
  location as a standalone `## §Trace v1.0.2` section (between v1.0.3 and v1.0.1 in the §Trace chain).
  The numbered list is now correctly ordered: 1, 2, 3 with no embedded §Trace prose.
- NORMATIVE: This is the 4th recorded instance of the ADR same-burst §Trace-escaping-into-normative-
  content defect class. The corrective discipline (pre-commit ADR self-consistency checklist) is
  codified in ADR-0007 v1.0.5 §Implementation Plan and SS-conventions-anti-patterns.md §ADR Authoring
  Discipline, added in the same burst (D-ADV30).
- NORMATIVE: Version bump 1.0.3 → 1.0.4 (structural correction; no operative rule change).
- SE-16d PASS: 2026-05-30 > chain high-water 2026-05-30T00:00:00Z — same calendar day, sequential pass.

## §Trace v1.0.3

**ADV-29 scope cross-reference — ADR-0007 §Enforcement Scan Scope alignment** (2026-05-30T00:00:00Z):

- NORMATIVE: §CI enforcement gate scope note added. Explains that POL-12's scan is
  narrower than POL-11's by design and does not require exemptions for `plans/`,
  `planning/`, `code-delivery/`, or `STATE.md` because those paths do not contain
  the story-Tasks and Consumer-Contract structural claim forms that POL-12 detects.
  Cross-references ADR-0007 v1.0.4 §Enforcement Scan Scope (added in same burst).
- NORMATIVE: Confirms independence of `collect_files()` implementations between POL-11
  and POL-12 — each is governed by its own ADR and must stay synchronized with it.
- NORMATIVE: Version bump 1.0.2 → 1.0.3 (informational cross-reference + explicit
  scope confirmation; no operative detection-rule change).
- SE-16d PASS: 2026-05-30T00:00:00Z > chain high-water 2026-05-30 (monotonic;
  v1.0.2 was same-day patch with no explicit timestamp — this entry establishes chain).

## §Trace v1.0.2

**1.0.2** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers and time qualifiers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at authoring time. No normative content changed.

## §Trace v1.0.1

**Pass 28 F-S025-ADV28-MED-002 closure — same-burst self-inconsistency correction** (2026-05-29):

- NORMATIVE: §Canonical Source Registry `App` struct field types row line range corrected: `(lines 831-864)` → `(lines 833-864)`. Lines 831-832 of SS-tui.md are the code-block fence (` ```rust`) and the filename comment (`// monocle-tui/src/app.rs`) — not struct content. `pub struct App {` begins at line 833; closing `}` is at line 864. Off-by-2 at start.
- NORMATIVE: Same defect propagated to SS-conventions-anti-patterns.md v1.32.2:1702 — corrected to `(lines 833-864)` in that file's §Structural-Claim Discipline §Canonical Source Registry (becomes v1.32.3).
- NORMATIVE: Explicit self-application policy added to §Canonical Source Registry: the registry table is itself subject to POL-12 (stale entries — wrong line range, deprecated anchors, removed canonical sources — are detected by POL-12 against cited canonical document content). This was implicit via §Exempt list exclusion; now explicit.
- SE-16d PASS: 2026-05-29 > chain high-water 2026-05-29T12:00:00Z — same calendar day, sequential pass.
- DEFECT CLASS: Same-burst self-inconsistency (authored structural claim citing lines 831-864 but canonical document has struct body at 833-864). Third instance of newly-authored ADR same-burst internal-consistency defect caught by fresh-context adversarial review (ADR-0006 indirect path, ADR-0007 Pass 26 HIGH-001, ADR-0008 Pass 28 MED-002).

## §Trace v1.0.0

**ADR-0008 initial ratification — D-206 structural-spec drift tripwire closure** (2026-05-29T12:00:00Z):

- NORMATIVE: ADR-0008 authored. Decision: Option B (distinct ADR for structural-claim
  discipline). Tripwire fired at Pass 27 (2nd structural-spec drift instance, story-body
  type-name layer) per Task #9 m.6 arm condition (2 instances → dispatch architect).
- NORMATIVE: Codifies POL-12-structural-claim as complement to ADR-0007 POL-11-version-pin.
  Both address authoring-time documentation drift; different detection algorithms.
- Produces concurrent updates (same burst): SS-conventions-anti-patterns.md v1.32.1→v1.32.2
  (§Structural-Claim Discipline section added); ARCH-INDEX.md ADR Registry row added.
- NORMATIVE: Story-writer dispatched to fix S-025:144,228 Vec<SessionState>→Vec<EnrichedSession>
  in parallel. S-028 deferred per BC-5.39.002 PC2 cross-story deferral.
- SE-16d PASS: 2026-05-29T12:00:00Z — initial ratification, no prior chain.
