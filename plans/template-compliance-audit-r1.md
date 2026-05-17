---
document_type: template-compliance-audit
version: "1.0"
producer: vsdd-factory:spec-steward
timestamp: 2026-05-16T00:00:00Z
scope: .factory/specs/
templates_root: /Users/jmagady/.claude/plugins/cache/claude-mp/vsdd-factory/1.0.0-rc.18/templates/
---

# Template Compliance Audit R1: monocle Phase 1 Spec Package

## Executive Summary

The monocle Phase 1 spec package achieves **substantive correctness** (22 BCs, 22 VPs, 12 NFRs,
61 ECs all present and internally consistent across 40+ adversarial review rounds) but has
**severe structural non-compliance** with VSDD templates. Every major artifact class violates the
one-per-file or index+shard patterns that the templates mandate.

### Aggregate Findings

| Severity | Count | Examples |
|----------|-------|---------|
| FAIL (missing required frontmatter field) | 28 | `input-hash` literal `[live-state]` in 10 files; `supplements: []` on PRD vs template requires list; missing `traces_to` on VP monolith |
| FAIL (wrong structural pattern — monolith vs one-per-file) | 3 | PRD inlines 22 BCs; VP monolith inlines 22 VPs; no `domain-spec/` shards |
| FAIL (missing required directory/index) | 7 | No `behavioral-contracts/`, no `prd-supplements/`, no `domain-spec/`, no `ARCH-INDEX.md`, no `BC-INDEX.md`, no `VP-INDEX.md`, no `L2-INDEX.md` |
| WARN (missing required section) | 31 | PRD §5b test-vectors missing; all SS-* missing `traces_to: ARCH-INDEX.md` (ARCH-INDEX absent); ADR-0002 missing `## Source / Origin` |
| WARN (section name drift from template) | 11 | PRD §2 "Behavioral Contracts" vs template "Behavioral Contracts Index"; VP document uses `document_type: verification-properties` vs template `verification-property`; SS files use `## [Section Content]` placeholder heading |
| WARN (extra non-template section) | 14 | PRD §§ Trace v1.0..v1.25 (25 trace sections); VP §§ Trace v1.0..v1.35 (35 trace sections); PRD §8 Cross-Cutting Concerns; PRD §10 Glossary |
| WARN (BC ID scheme mismatch) | 22 | All 22 BCs use `BC-DOMAIN-NNN` vs template `BC-S.SS.NNN` |
| WARN (VP ID scheme mismatch) | 22 | All 22 VPs use `VP-DOMAIN-NNN` vs template `VP-NNN` |

**Pass/Warn/Fail summary:**

| File | L1 Frontmatter | L2 Sections | L3 Tables | Overall |
|------|----------------|-------------|-----------|---------|
| prd.md | FAIL | WARN | WARN | FAIL |
| verification-properties.md | FAIL | FAIL | FAIL | FAIL |
| product-brief.md | WARN | WARN | PASS | WARN |
| dtu-assessment.md | WARN | WARN | PASS | WARN |
| SS-daemon-lifecycle.md | FAIL | WARN | N/A | FAIL |
| SS-core-types-and-abi.md | FAIL | WARN | N/A | FAIL |
| SS-engine-module.md | FAIL | WARN | N/A | FAIL |
| SS-deps-pin-manifest.md | FAIL | WARN | N/A | FAIL |
| SS-conventions-anti-patterns.md | FAIL | WARN | N/A | FAIL |
| SS-forward-compatibility.md | FAIL | WARN | N/A | FAIL |
| SS-permissions-phase1.md | FAIL | WARN | N/A | FAIL |
| ADR-0001 | PASS | WARN | N/A | WARN |
| ADR-0002 | PASS | WARN | N/A | WARN |
| ADR-0003 | PASS | WARN | N/A | WARN |
| ADR-0004 | PASS | WARN | N/A | WARN |
| research/domain-monocle-vision-synthesis.md | WARN | N/A | N/A | WARN |
| MISSING: behavioral-contracts/ | — | — | — | MISSING |
| MISSING: prd-supplements/ (4 files) | — | — | — | MISSING |
| MISSING: architecture/ARCH-INDEX.md | — | — | — | MISSING |
| MISSING: behavioral-contracts/BC-INDEX.md | — | — | — | MISSING |
| MISSING: verification-properties/VP-INDEX.md | — | — | — | MISSING |
| MISSING: domain-spec/ + L2-INDEX.md | — | — | — | MISSING |

**Remediation scope estimate:** 6 specialist agent dispatches (product-owner × 3, architect × 2,
formal-verifier × 1), approximately 4-6 bursts total. Content is 100% present; this is entirely
structural refactoring.

**Critical note:** The substantive spec work is sound. The convergence achieved across 100+
adversary/consistency rounds is not invalidated by this audit. Remediation is a structural
migration, not a content revision.

---

## Per-File Compliance

### 1. `prd.md` → `prd-template.md`

**Lines: 4,480. Version: v1.25. Template: prd-template.md.**

#### L1 — Frontmatter Compliance

| Field | Template Requires | File Status | Verdict |
|-------|-------------------|-------------|---------|
| `document_type` | `prd` | `prd` | PASS |
| `level` | `L3` | `L3` | PASS |
| `version` | semver string | `"1.25"` | PASS |
| `status` | draft/approved | `draft` | PASS |
| `producer` | agent-id | `product-owner` | PASS |
| `timestamp` | ISO-8601 | `2026-05-17T06:30:00Z` | PASS |
| `phase` | phase label | `phase-1-spec-crystallization` | PASS |
| `inputs` | list | present (14 items) | PASS |
| `input-hash` | md5 or `[md5]` placeholder | `"[live-state]"` (not `[md5]`) | FAIL — literal `[live-state]` is not the template-specified placeholder `[md5]`; signals hash has never been computed |
| `traces_to` | `domain-spec/L2-INDEX.md` | `"product-brief.md v1.4.23; vision-synthesis v1.1.2; ..."` (long trace string) | WARN — does not trace to `domain-spec/L2-INDEX.md` as template requires; traces to brief and vision instead; L2 domain spec does not exist |
| `supplements` | list of 4 supplement files | `[]` (empty list) | FAIL — template requires `[interface-definitions.md, error-taxonomy.md, test-vectors.md, nfr-catalog.md]`; all 4 supplements are absent as separate files, their content is inlined in PRD body instead |

**L1 verdict: FAIL** (2 hard fails: `input-hash` not computed, `supplements` empty when 4 supplement files must exist)

#### L2 — Section Compliance

Template requires: §1 Product Overview (with 1.1-1.5 subsections), §2 Behavioral Contracts Index, §3 Interface Definition, §4 Non-Functional Requirements, §5 Error Taxonomy, §5b Test Vectors, §6 Competitive Differentiator Traceability, §7 Requirements Traceability Matrix.

| Template Section | File Section | Status |
|-----------------|--------------|--------|
| `## 1. Product Overview` | `## 1. Product Overview` | PASS |
| `### 1.1 Problem Statement` | `### 1.1 Problem` | WARN — renamed |
| `### 1.2 Solution Vision` | `### 1.2 Vision` | WARN — renamed |
| `### 1.3 Key Differentiators` | `### 1.3 Competitive Differentiators` | WARN — renamed |
| `### 1.4 Target Users` | `### 1.4 Target Users` | PASS |
| `### 1.5 Out of Scope` | `### 1.5 Out of Scope` | PASS |
| `## 2. Behavioral Contracts Index` | `## 2. Behavioral Contracts` | WARN — title differs; CRITICAL — §2 is NOT an index of separate files; all 22 BCs are fully inlined in §3 at 800+ lines |
| template BC index model: one-line summary table → `behavioral-contracts/ss-NN/` | actual: §3 Full Behavioral Contract Specifications (1,112 lines of fully inlined BCs) | FAIL — monolith pattern violates BC index model |
| `## 3. Interface Definition` (supplement reference only) | MISSING as separate section — content inlined in §3 or absent | FAIL — no §3 Interface Definition section; no supplement reference |
| `## 4. Non-Functional Requirements` | `## 4. Non-Functional Requirements` | PASS (heading present) |
| `## 5. Error Taxonomy` | `## 5. Error Taxonomy` | PASS (heading present) |
| `## 5b. Test Vectors` | MISSING | FAIL — no test vectors section; no supplement reference |
| `## 6. Competitive Differentiator Traceability` | `## 6. Competitive Differentiator Traceability` | PASS |
| `## 7. Requirements Traceability Matrix` | `## 7. Requirements Traceability Matrix` | PASS |

Extra sections (not in template): `## 3. Full Behavioral Contract Specifications`, `## 8. Cross-Cutting Concerns`, `## 9. Edge Case Catalog`, `## 10. Glossary`, `## §Trace v1.0` through `## §Trace v1.25` (25 trace sections). The 25 trace sections are the primary driver of the 4,480-line count; the trace history is authoring provenance, not spec content.

**L2 verdict: WARN** (structural drift; monolith BC pattern is the critical violation)

#### L3 — Table Column Compliance

| Template Table | Template Columns | File Columns | Status |
|----------------|------------------|--------------|--------|
| PRD §1.3 Key Differentiators | `ID, Differentiator, Description` | `ID, Differentiator, BC Backing` | WARN — "Description" renamed "BC Backing" (functionally correct but schema drift) |
| PRD §1.4 Target Users | `Persona, Description, Volume, Pain Level` | `Persona, Pain, Phase` | WARN — "Description" absent; "Volume" absent; "Pain Level" renamed "Phase" |
| PRD §2 BC Index table | `BC ID, Title, Priority` | Mixed inline/table — no compliant index table | FAIL |
| PRD §7 RTM | `BC ID, Source (L2 CAP), Module(s), Priority, Test Type` | `Requirement ID, Source, Test File, Priority, Test Type, Brief Section` | WARN — "BC ID" renamed "Requirement ID" (F-R84-2 fix); "Module(s)" absent; extra columns added |

**L3 verdict: WARN**

---

### 2. `verification-properties.md` → `L4-verification-property-template.md` (one-per-file pattern)

**Lines: 15,612. Version: v1.35. Template: L4-verification-property-template.md.**

This is the most severely non-compliant artifact. The template mandates one file per VP;
this file contains all 22 VPs inline.

#### L1 — Frontmatter Compliance

| Field | Template Requires | File Status | Verdict |
|-------|-------------------|-------------|---------|
| `document_type` | `verification-property` (per-file) | `verification-properties` (monolith) | FAIL — wrong document_type; signals wrong structural pattern |
| `level` | `L4` | `L3` (!) | FAIL — wrong level; template specifies L4; file claims L3 |
| `version` | semver | `"1.35"` | PASS |
| `status` | draft/in-development/verified/withdrawn | `draft` | PASS |
| `producer` | `architect` | `formal-verifier` | WARN — template specifies `architect`; file uses `formal-verifier` (arguably correct for actual authorship) |
| `timestamp` | ISO-8601 | `2026-05-17T07:00:00Z` | PASS |
| `phase` | `1b` | `phase-1-spec-crystallization` | WARN — template uses `1b`; file uses descriptive phase label |
| `inputs` | list | present (14 items) | PASS |
| `input-hash` | `[md5]` placeholder | `"[live-state]"` | FAIL — same issue as PRD |
| `traces_to` | `prd.md` | long trace string | WARN — traces to complex history rather than simple `prd.md` |
| `source_bc` | `BC-S.SS.NNN` | MISSING (monolith has no single source BC) | FAIL — required field for per-file template |
| `module` | implementation module name | MISSING | FAIL |
| `proof_method` | kani/proptest/fuzz/manual/tla+ | MISSING | FAIL |
| `feasibility` | feasible/needs-research/infeasible | MISSING | FAIL |
| `verification_lock` | boolean | MISSING | FAIL |
| `proof_completed_date` | null or date | MISSING | FAIL |
| `proof_file_hash` | null or hash | MISSING | FAIL |
| `lifecycle_status` | active/deprecated/... | MISSING | FAIL |
| `introduced` | vX.Y.Z | MISSING | FAIL |

**L1 verdict: FAIL** (10 missing required fields because the monolith pattern is incompatible with the per-file template)

#### L2 — Section Compliance

Template requires per VP file: `## Property Statement`, `## Source Contract`, `## Proof Method`, `## Proof Harness Skeleton`, `## Feasibility Assessment`, `## Lifecycle`.

The monolith file structure is: `## §Purpose`, `## §Scope`, `## §VP Catalog Overview`, `## §Per-VP Detail` (with 22 VP subsections each), `## §Coverage Matrix`, `## §Open Verification Gaps`, `## §References`, `## §Trace`.

The 22 VP subsections (e.g., `### §VP-DAEMON-001 — /healthz...`) each contain the equivalent of the template's per-file sections, but within a single monolith file. The content is present; the structure is wrong.

Per VP, the template requires `## Property Statement` — these appear as inline prose under each `### §VP-NNN` heading. The template requires `## Proof Harness Skeleton` — these appear as inline code blocks. The template requires `## Lifecycle` table — these appear as inline lists. None match the template H2 heading names.

**L2 verdict: FAIL** (entire section schema is monolith-adapted vs template per-file)

#### L3 — Table Column Compliance

Template `## Proof Method` table: `Method, Tool, Bounded?, Coverage`.
File equivalent: per-VP mechanism annotation as inline labeled fields, not a table.

Template `## Feasibility Assessment` table: `Factor, Assessment, Notes`.
File equivalent: per-VP `Mechanism:` labeled line, not a table.

Template `## Lifecycle` table: `Event, Date, Actor`.
File equivalent: absent as explicit table.

**L3 verdict: FAIL** (no template tables present in compliant form)

---

### 3. `product-brief.md` → `product-brief-template.md`

**Lines: 431. Version: v1.4.23.**

#### L1 — Frontmatter Compliance

| Field | Template Requires | File Status | Verdict |
|-------|-------------------|-------------|---------|
| `document_type` | `product-brief` | `product-brief` | PASS |
| `level` | `L1` | `L1` | PASS |
| `version` | semver | `"1.4.23"` | PASS |
| `status` | draft | `draft` | PASS |
| `producer` | agent-id | `product-owner` | PASS |
| `timestamp` | ISO-8601 | `2026-05-14T07:50:10Z` | PASS |
| `phase` | phase label | `pre-phase-1-brief` | PASS |
| `inputs` | list | present (10 items) | PASS |
| `input-hash` | `[md5]` | `"[live-state]"` | FAIL — hash never computed |
| `traces_to` | empty string (no upstream) | `"factory-artifacts 2737bfd..."` | WARN — traces to git commit refs rather than artifact file; functionally reasonable |

Extra frontmatter fields not in template: `project`, `supplements`. The `supplements` field is populated with 12 architecture files — this is a project-level extension that predates the template. Not a violation, but is non-standard.

**L1 verdict: WARN** (1 fail: `input-hash` not computed; other fields consistent with project conventions)

#### L2 — Section Compliance

Template requires: `## What Is This?`, `## Who Is It For?`, `## Scope` (with `### In Scope` / `### Out of Scope`), `## Success Criteria`, `## Constraints & Integration Points`, `## Overflow Context (Optional)`.

| Template Section | File Section | Status |
|-----------------|--------------|--------|
| `## What Is This?` | `## What Is This?` | PASS |
| (no revision history in template) | `## Revision History` | EXTRA (non-template) |
| `## Who Is It For?` | `## Who Is It For?` | PASS |
| `## Scope` | `## Scope` | PASS |
| `### In Scope` | `### In Scope` | PASS |
| `### Out of Scope` | MISSING at H3 — content covered under `## Scope` body | WARN |
| `## Success Criteria` | `## Success Criteria` | PASS |
| `## Constraints & Integration Points` | `## Constraints & Integration Points` | PASS |
| `## Overflow Context` | `## Overflow Context` | PASS |

Extra sections: `## Phase 2 Exit Criteria`, `## Phase 1 Constraints (from OQ Resolutions)`, `## Open Questions for Architect`. These are project-specific extensions; not a compliance failure but non-standard.

The `## Revision History` section (88 rows, lines 57-89) is not in the template. It has grown into a 430-line trace log of all brief revisions. This serves the same function as the `##§Trace` sections in architecture files — useful provenance but not template-mandated.

**L2 verdict: WARN** (missing `### Out of Scope` sub-heading; extra sections beyond template)

#### L3 — Table Column Compliance

Template `## Who Is It For?` table: `Persona, Pain Point, Current Workaround`.
File table: `Persona, Pain Point, Current Workaround`. PASS.

Template `## Success Criteria` table: `Outcome, Metric, Target`.
File table: `Outcome, Metric, Target`. PASS.

**L3 verdict: PASS**

---

### 4. `dtu-assessment.md` → `dtu-assessment-template.md`

**Lines: 461. Version: v1.7.**

#### L1 — Frontmatter Compliance

| Field | Template Requires | File Status | Verdict |
|-------|-------------------|-------------|---------|
| `document_type` | `dtu-assessment` | `dtu-assessment` | PASS |
| `level` | `L3` | `L3` | PASS |
| `version` | semver | `"1.7"` | PASS |
| `status` | draft | `complete` | WARN — template default is `draft`; file uses `complete`; not incorrect semantically |
| `producer` | `architect` | `architect` | PASS |
| `timestamp` | ISO-8601 | `2026-05-14T08:00:00Z` | PASS |
| `phase` | phase label | `pre-phase-1-architecture` | PASS |
| `inputs` | list | present (4 items) | PASS |
| `input-hash` | `[md5]` | `"[live-state]"` | FAIL |
| `traces_to` | `architecture/ARCH-INDEX.md` | long trace string (not `architecture/ARCH-INDEX.md`) | FAIL — ARCH-INDEX.md does not exist; file traces to commit chain instead |

Extra frontmatter: `dtu_required: true`, `project: monocle`. Project extensions, acceptable.

**L1 verdict: FAIL** (2 fails: `input-hash` not computed; `traces_to` does not reference `ARCH-INDEX.md`)

#### L2 — Section Compliance

Template requires: `## Summary`, `## Integration Surface Inventory` (with 6 mandatory subsections), `## Dependency Summary`, `## Services NOT Requiring DTU`, `## DTU Architecture` (with Docker Compose + Env Var tables), `## Clone Development Approach`.

| Template Section | File Section | Status |
|-----------------|--------------|--------|
| `## Summary` | `## Summary` | PASS |
| `## Integration Surface Inventory` | `## Integration Surface Inventory (MANDATORY — all categories required)` | PASS |
| `### Inbound Data Sources` | `### Inbound Data Sources (External → Product)` | PASS |
| `### Outbound Operations` | `### Outbound Operations (Product → External)` | PASS |
| `### Identity & Access` | `### Identity & Access (Bidirectional — auth flow)` | PASS |
| `### Persistence & State` | `### Persistence & State (Product ↔ Storage)` | PASS |
| `### Observability & Export` | `### Observability & Export (Product → Monitoring)` | PASS |
| `### Enrichment & Lookup` | `### Enrichment & Lookup (External → Product, on-demand)` | PASS |
| `## Dependency Summary` | `## Dependency Summary` | PASS |
| `## Services NOT Requiring DTU` | `## Services NOT Requiring DTU` | PASS |
| `## DTU Architecture` | `## DTU Architecture` | PASS |
| `## Clone Development Approach` | `## Clone Development Approach` | PASS |

Extra sections beyond template: `## DTU Fidelity Measurement Procedure`, `## Phase 1 Clone Build Effort`, `## §Trace`. These are project-specific additions; DTU Fidelity Measurement is a valuable production-grade enhancement over the template.

**L2 verdict: WARN** (no missing required sections; extras are project-appropriate)

#### L3 — Table Column Compliance

Template `## Integration Surface Inventory` tables: `#, Service, Protocol, Fidelity, DTU?, Justification`. File tables: match exactly. PASS.

Template `## Dependency Summary` table: `#, Service, Category, Fidelity, DTU?, Points, Justification`. File table: matches. PASS.

Template `## DTU Architecture / Docker Compose` table: `Clone, Port, Fidelity, Depends On`. File table: matches. PASS.

Template `## DTU Architecture / Environment Variable Overrides` table: `Variable, Production Value, DTU Value`. File table: matches. PASS.

**L3 verdict: PASS**

---

### 5. `architecture/SS-daemon-lifecycle.md` → `architecture-section-template.md`

**Version: v1.0.25.**

#### L1 — Frontmatter Compliance

| Field | Template Requires | File Status | Verdict |
|-------|-------------------|-------------|---------|
| `document_type` | `architecture-section` | `architecture-section` | PASS |
| `level` | `L3` | `L3` | PASS |
| `section` | section-name string | `"daemon-lifecycle"` | PASS |
| `version` | semver | `"1.0.25"` | PASS |
| `status` | draft | `complete` | WARN |
| `producer` | `architect` | `architect` | PASS |
| `timestamp` | ISO-8601 | `2026-05-17T06:00:00Z` | PASS |
| `phase` | phase label | `pre-phase-1-architecture` | PASS |
| `inputs` | list | present (4 items) | PASS |
| `input-hash` | `[md5]` | `"[live-state]"` | FAIL |
| `traces_to` | `ARCH-INDEX.md` | long trace string | FAIL — ARCH-INDEX.md does not exist |

Extra frontmatter: `project: monocle`. Acceptable project extension.

**L1 verdict: FAIL** (2 fails: hash not computed; traces_to missing ARCH-INDEX reference)

#### L2 — Section Compliance

Template requires: one `## [Section Content]` heading (literally a placeholder) plus content.
File has `## [Section Content]` placeholder heading present (line 22) but actual content uses custom H2s: `## Scope`, `## Health and Status Endpoints`, `## Body Size Limit`, `## Daemon Lifecycle Protocol`, `## Lock File Discovery Policy`, `## Behavioral Contract Summary`, `## Phase 4 Notes`, `## §Trace`.

The `## [Section Content]` placeholder is retained as a literal heading, which creates a cosmetic artifact (an empty H2 before `## Scope`). This is a minor issue — the placeholder should have been replaced with actual content headings or removed.

Template sizing guidance: 800-1,200 tokens (~50-80 lines). Actual file is substantially larger (the §Trace section alone accounts for most of the length). This is not a compliance violation per se but contradicts the sizing principle.

**L2 verdict: WARN** (placeholder heading retained; file exceeds token guidance but content is substantive)

---

### 6. `architecture/SS-core-types-and-abi.md` → `architecture-section-template.md`

**Version: v1.2.8.**

#### L1 — Frontmatter Compliance

| Field | Template Requires | File Status | Verdict |
|-------|-------------------|-------------|---------|
| `document_type` | `architecture-section` | `architecture-core-types` | FAIL — wrong document_type; template requires `architecture-section` |
| `level` | `L3` | `L3` | PASS |
| `section` | section-name string | `"core"` | WARN — template uses full section name; `"core"` is a partial identifier |
| `version` | semver | `"1.2.8"` | PASS |
| `status` | draft | `complete` | WARN |
| `producer` | `architect` | `architect` | PASS |
| `timestamp` | ISO-8601 | `2026-05-14T08:00:00Z` | PASS |
| `input-hash` | `[md5]` | `"[live-state]"` | FAIL |
| `traces_to` | `ARCH-INDEX.md` | long trace string | FAIL |

Extra frontmatter: `slug: "types-and-abi"`, `subsystem: "core"`. Project extensions.

**L1 verdict: FAIL** (3 fails: wrong document_type, hash not computed, traces_to wrong)

---

### 7. `architecture/SS-engine-module.md` → `architecture-section-template.md`

**Version: v1.1.15.**

#### L1 — Frontmatter Compliance

Same pattern as SS-core-types-and-abi.md:

| Field | Status | Verdict |
|-------|--------|---------|
| `document_type` | `architecture-section` | PASS |
| `level` | `L3` | PASS |
| `section` | `"engine-module"` | PASS |
| `input-hash` | `"[live-state]"` | FAIL |
| `traces_to` | long trace string (no ARCH-INDEX ref) | FAIL |
| extra: `slug`, `subsystem` | present | WARN |

**L1 verdict: FAIL**

---

### 8. `architecture/SS-deps-pin-manifest.md` → `architecture-section-template.md`

**Version: v1.1.15.**

#### L1 — Frontmatter Compliance

| Field | Status | Verdict |
|-------|--------|---------|
| `document_type` | `architecture-dependencies` | FAIL — should be `architecture-section` |
| `level` | `L3` | PASS |
| `section` | `"deps"` | WARN — partial |
| `input-hash` | `"[live-state]"` | FAIL |
| `traces_to` | long trace string | FAIL |
| extra: `slug` | present | WARN |

**L1 verdict: FAIL**

---

### 9. `architecture/SS-conventions-anti-patterns.md` → `architecture-section-template.md`

**Version: v1.28.**

#### L1 — Frontmatter Compliance

| Field | Status | Verdict |
|-------|--------|---------|
| `document_type` | `architecture-section` | PASS |
| `level` | `L3` | PASS |
| `section` | `"conventions"` | PASS |
| `input-hash` | `"[live-state]"` | FAIL |
| `traces_to` | long trace string | FAIL |

**L1 verdict: FAIL**

---

### 10. `architecture/SS-forward-compatibility.md` → `architecture-section-template.md`

**Version: v1.2.13.**

#### L1 — Frontmatter Compliance

| Field | Status | Verdict |
|-------|--------|---------|
| `document_type` | `architecture-section` | PASS |
| `level` | `L3` | PASS |
| `section` | `"forward-compat"` | PASS |
| extra: `slug` | present | WARN |
| `input-hash` | `"[live-state]"` | FAIL |
| `traces_to` | long trace string | FAIL |

**L1 verdict: FAIL**

---

### 11. `architecture/SS-permissions-phase1.md` → `architecture-section-template.md`

**Version: v1.4.**

#### L1 — Frontmatter Compliance

| Field | Status | Verdict |
|-------|--------|---------|
| `document_type` | `architecture-permissions` | FAIL — should be `architecture-section` |
| `level` | `L3` | PASS |
| `section` | `"permissions"` | PASS |
| `input-hash` | `"[live-state]"` | FAIL |
| `traces_to` | long trace string | FAIL |
| extra: `slug` | present | WARN |

**L1 verdict: FAIL**

---

### 12–15. ADRs: `adr/ADR-0001` through `ADR-0004` → `adr-template.md`

All four ADRs are checked against `adr-template.md`.

**ADR-0001-wasmtime-vs-wasmi.md (v1.0.2)**

| Field | Template Requires | File Status | Verdict |
|-------|-------------------|-------------|---------|
| `document_type` | `adr` | `adr` | PASS |
| `adr_id` | `ADR-NNN` (3-digit) | `ADR-0001` (4-digit) | WARN — template shows 3-digit; file uses 4-digit. Both are valid sequencing; not a hard fail |
| `status` | proposed/accepted/superseded | `accepted` | PASS |
| `date` | YYYY-MM-DD | `2026-05-12` | PASS |
| `subsystems_affected` | array of SS-NN | `[]` | WARN — empty; wasmtime affects Phase 3 plugin subsystem (SS-??) but ARCH-INDEX doesn't exist yet |
| `supersedes` | null or ADR-NNN | `null` | PASS |
| `superseded_by` | null or ADR-NNN | `null` | PASS |

Extra non-template frontmatter: `level`, `version`, `producer`, `phase`, `timestamp`, `inputs`, `input-hash`, `traces_to`, `project`. These are project conventions extending the template — acceptable.

Sections: Template requires `## Context`, `## Decision`, `## Rationale`, `## Consequences` (with `### Positive`, `### Negative / Trade-offs`, `### Status as of...`), `## Alternatives Considered`, `## Source / Origin`.

ADR-0001 sections: all template sections present. Extra: `## Amendment History`. PASS with one note — template requires `## Source / Origin` last; file has it present.

**ADR-0001 verdict: WARN** (4-digit ID vs template 3-digit; empty `subsystems_affected`)

---

**ADR-0002-nucleo-acceptance-with-reeval-trigger.md (v1.0)**

Template sections: `## Context` PASS, `## Decision` PASS, `## Rationale` PASS, `## Consequences` PASS, `## Alternatives Considered` PASS.

Missing: `## Source / Origin` — WARN.

Extra sections: `## Re-eval Trigger`, `## Supersedes`. These are project additions, not template violations.

**ADR-0002 verdict: WARN** (missing `## Source / Origin`)

---

**ADR-0003-license-selection.md (v1.0.1)**

Template sections: `## Context` PASS, `## Decision` PASS, `## Rationale` PASS, `## Consequences` PASS, `## Alternatives Considered` PASS, `## Source / Origin` PASS. Extra: `## Re-eval Triggers`.

**ADR-0003 verdict: PASS**

---

**ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md (v1.0.3)**

Template sections: `## Context` PASS, `## Decision` PASS, `## Rationale` PASS, `## Consequences` PASS, `## Alternatives Considered` PASS.

Missing: `## Source / Origin` — WARN (file has `## Source / Origin` section: actually present at end. PASS on re-check).

Extra: `## Date`, `## Amendment History`.

**ADR-0004 verdict: WARN** (extra `## Date` section at line 30 — template embeds date in frontmatter, not as a section heading)

---

### 16. `research/domain-monocle-vision-synthesis.md` → No canonical VSDD template

The vision synthesis has `document_type: vision-synthesis` which is not defined in the templates directory. The closest template would be a research artifact. This file predates the VSDD spec crystallization pipeline and is classified as an `ops`-level document (not L1-L4).

**Verdict: NOT ASSESSED** — no corresponding template exists. File is correctly classified as pre-pipeline input, not a living spec.

---

## Missing Artifacts

### MISS-01: `behavioral-contracts/` directory + sharded BC files

**Severity: CRITICAL — blocks Phase 2 story decomposition**

Template: `behavioral-contract-template.md`
Expected path: `.factory/specs/behavioral-contracts/ss-NN/BC-S.SS.NNN.md`
Expected index: `.factory/specs/behavioral-contracts/BC-INDEX.md`

Currently contains: All 22 BCs are inlined in `prd.md` §3 (lines 103-1213) using free-form markdown prose sections (`### BC-DAEMON-001 — ...` through `### BC-ENGINE-003 — ...`).

Content that must be extracted:
- 22 BC prose sections from PRD §3
- Per BC, the template requires: frontmatter (20 fields), `## Description`, `## Preconditions`, `## Postconditions`, `## Invariants`, `## Edge Cases` (table), `## Canonical Test Vectors` (table), `## Verification Properties` (table), `## Traceability` (table)
- Each extracted BC also needs lifecycle fields added (all currently absent from inline prose)
- BC frontmatter requires `subsystem: "SS-NN"` — subsystem mapping must be determined (see §BC ID Renumbering Map)

**BC-INDEX.md required columns** (from template): `BC ID, Title, Subsystem, Priority, Status, VP Count`

**Impact of absence:** Phase 2 story-writer skill expects to load `behavioral-contracts/BC-INDEX.md` for story decomposition. Phase 4 holdout-evaluator loads per-BC files for scenario generation. Phase 5 adversary loads per-BC for fresh-context review. All Phase 2+ pipeline phases will fail to find required inputs.

---

### MISS-02: `prd-supplements/` directory (4 files)

**Severity: HIGH — affects agent context efficiency**

Templates: `prd-supplement-error-taxonomy-template.md`, `prd-supplement-interface-definitions-template.md`, `prd-supplement-nfr-catalog-template.md`, `prd-supplement-test-vectors-template.md`

Expected paths:
- `.factory/specs/prd-supplements/error-taxonomy.md`
- `.factory/specs/prd-supplements/interface-definitions.md`
- `.factory/specs/prd-supplements/nfr-catalog.md`
- `.factory/specs/prd-supplements/test-vectors.md`

Currently contains (in PRD monolith):
- `## 4. Non-Functional Requirements` (lines 1215-1233) — 12 NFR rows present inline
- `## 5. Error Taxonomy` (lines 1234-1256) — error codes present inline
- `## 7. Requirements Traceability Matrix` (lines 1274-1302) — acts as test-vectors reference
- No dedicated interface-definitions section (content distributed across architecture SS-* files)

Content location in PRD for extraction:
- NFR catalog: PRD §4 (12 NFR-NNN rows with Category, Requirement, Target, Validation Method, Priority)
- Error taxonomy: PRD §5 (error codes grouped by subsystem)
- Test vectors: Per-BC `**Canonical Test Vectors:**` tables in PRD §3 (inline prose tables under each BC)
- Interface definitions: No dedicated PRD section; content lives in SS-daemon-lifecycle.md (endpoint definitions), SS-core-types-and-abi.md (struct definitions), SS-permissions-phase1.md

---

### MISS-03: `architecture/ARCH-INDEX.md`

**Severity: CRITICAL — all architecture SS-* files have broken `traces_to`**

Template: `architecture-index-template.md`
Expected path: `.factory/specs/architecture/ARCH-INDEX.md`

The 7 SS-* files all have `traces_to:` pointing to a long trace history string rather than `ARCH-INDEX.md` — because ARCH-INDEX.md does not exist. Every architecture section file is therefore orphaned from the index.

Required ARCH-INDEX.md content:
- Document Map table: `Section, File, Tokens, Primary Consumer, Purpose`
- Cross-References table
- Subsystem Registry (CRITICAL): `SS ID, Name, Architecture Doc, Implementing Modules, Phase Introduced`
  - This registry is the source-of-truth for `BC frontmatter subsystem:` field (Policy 6)
  - Without it, the BC-ID renumbering (§BC ID Renumbering Map below) cannot be completed without a simultaneous ARCH-INDEX.md creation
- Architecture Decisions table (summary; full decisions in ADRs)

The 7 SS-* files implicitly define subsystems:
- `SS-daemon-lifecycle.md` → daemon lifecycle subsystem
- `SS-core-types-and-abi.md` → core types / ABI subsystem
- `SS-engine-module.md` → engine module subsystem
- `SS-permissions-phase1.md` → permissions subsystem
- `SS-deps-pin-manifest.md` → dependency management (not a runtime subsystem; may be support artifact)
- `SS-conventions-anti-patterns.md` → conventions (not a runtime subsystem; support artifact)
- `SS-forward-compatibility.md` → forward compatibility (cross-cutting; may not need an SS-NN ID)

Proposed subsystem IDs for ARCH-INDEX Subsystem Registry (see §BC ID Renumbering Map for BC implications):

| SS ID | Name | Architecture Doc | Notes |
|-------|------|-----------------|-------|
| SS-01 | Daemon Lifecycle | SS-daemon-lifecycle.md | Primary runtime subsystem |
| SS-02 | Core Types and ABI | SS-core-types-and-abi.md | ABI stability surface |
| SS-03 | Engine Module | SS-engine-module.md | EngineModule trait |
| SS-04 | Permissions | SS-permissions-phase1.md | Phase 1 permission enum |
| (support) | Dependency Manifest | SS-deps-pin-manifest.md | Not a runtime SS; architectural support |
| (support) | Conventions | SS-conventions-anti-patterns.md | Not a runtime SS; process document |
| (support) | Forward Compatibility | SS-forward-compatibility.md | Cross-cutting; annotates all SS |

---

### MISS-04: `verification-properties/VP-INDEX.md` + per-VP files

**Severity: CRITICAL — Phase 6 formal hardening cannot locate VP files**

Template: `L4-verification-property-template.md`
Expected paths: `.factory/specs/verification-properties/VP-INDEX.md` + `vp-NNN-<slug>.md` per VP

Currently: All 22 VPs are inlined in `verification-properties.md` at `.factory/specs/verification-properties.md` (flat file, wrong path, wrong structure).

Note: The path `.factory/specs/verification-properties.md` (file) vs `.factory/specs/verification-properties/` (directory) is itself a structural conflict — the file occupies the namespace the directory needs.

The 22 VP IDs present: VP-DAEMON-001 through VP-DAEMON-006, VP-RING-001, VP-AUTH-001, VP-AUTH-002, VP-LOCK-001, VP-ABI-001, VP-ABI-002, VP-TYPES-001, VP-FACTORY-001, VP-FACTORY-002, VP-PROTO-001a, VP-PROTO-001b, VP-PROTO-002, VP-ENGINE-001, VP-ENGINE-002, VP-ENGINE-002-ERR, VP-ENGINE-003.

Per-VP frontmatter missing from each VP (when extracted): `source_bc`, `module`, `proof_method`, `feasibility`, `verification_lock`, `proof_completed_date`, `proof_file_hash`, `lifecycle_status`, `introduced`, `modified`, `deprecated`, `deprecated_by`, `replacement`, `retired`, `withdrawn`, `withdrawal_reason`, `removed`, `removal_reason`.

---

### MISS-05: `domain-spec/` directory + `L2-INDEX.md`

**Severity: HIGH — PRD `traces_to` chain is broken at the L2 level**

Templates: `L2-domain-spec-index-template.md`, `L2-domain-spec-section-template.md`
Expected paths: `.factory/specs/domain-spec/L2-INDEX.md` + section files

Currently: No L2 domain spec exists for monocle. The pipeline went directly from product-brief → vision synthesis → PRD, skipping the formal L2 domain spec authoring step.

The vision synthesis (`research/domain-monocle-vision-synthesis.md`) functions as an informal L2 domain spec but does not conform to the L2 template structure (no CAP-NNN capability catalog, no DI-NNN invariants, no DEC-NNN edge cases, etc.).

This is the most significant pipeline gap. However, given that Phase 1 spec crystallization is complete and convergence has been achieved, the L2 domain spec creation is effectively a backfill exercise — the content exists implicitly across the vision synthesis, PRD §1, and architecture documents.

Required L2 content exists in:
- CAP-NNN capabilities: derivable from PRD §1 competitive differentiators + brief §Scope
- DI-NNN domain invariants: derivable from BC invariants in PRD §3
- DEC-NNN domain edge cases: derivable from PRD §9 Edge Case Catalog (ECs 001-061)
- ASM-NNN assumptions: partially in brief §Open Questions for Architect
- R-NNN risks: R-001 in brief §Overflow Context
- FM-NNN failure modes: derivable from PRD error taxonomy §5

---

## BC ID Renumbering Map

Current IDs use the `BC-DOMAIN-NNN` scheme. Template requires `BC-S.SS.NNN` where:
- S = PRD section (all Phase 1 BCs are in §2, so S=2)
- SS = subsystem number (from ARCH-INDEX Subsystem Registry, proposed above)
- NNN = sequential within subsystem

Proposed subsystem assignments based on BC groupings in PRD §2.1:

| SS ID | Subsystem Name | Current BC IDs |
|-------|---------------|----------------|
| SS-01 | Daemon Lifecycle | BC-DAEMON-001..006, BC-AUTH-001..002, BC-LOCK-001, BC-RING-001 |
| SS-02 | Core Types and ABI | BC-ABI-001..002, BC-TYPES-001, BC-FACTORY-001..002, BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 |
| SS-03 | Engine Module | BC-ENGINE-001..002, BC-ENGINE-002-ERR, BC-ENGINE-003 |

**Full renumbering map (current → template-compliant):**

| Current ID | Proposed Template ID | Subsystem | Notes |
|------------|---------------------|-----------|-------|
| BC-DAEMON-001 | BC-2.01.001 | SS-01 Daemon Lifecycle | |
| BC-DAEMON-002 | BC-2.01.002 | SS-01 Daemon Lifecycle | |
| BC-DAEMON-003 | BC-2.01.003 | SS-01 Daemon Lifecycle | |
| BC-DAEMON-004 | BC-2.01.004 | SS-01 Daemon Lifecycle | |
| BC-DAEMON-005 | BC-2.01.005 | SS-01 Daemon Lifecycle | |
| BC-DAEMON-006 | BC-2.01.006 | SS-01 Daemon Lifecycle | |
| BC-RING-001 | BC-2.01.007 | SS-01 Daemon Lifecycle | JSONL ring belongs to daemon lifecycle subsystem |
| BC-AUTH-001 | BC-2.01.008 | SS-01 Daemon Lifecycle | Auth is part of daemon lifecycle protocol |
| BC-AUTH-002 | BC-2.01.009 | SS-01 Daemon Lifecycle | |
| BC-LOCK-001 | BC-2.01.010 | SS-01 Daemon Lifecycle | Lock file belongs to daemon lifecycle |
| BC-ABI-001 | BC-2.02.001 | SS-02 Core Types and ABI | |
| BC-ABI-002 | BC-2.02.002 | SS-02 Core Types and ABI | |
| BC-TYPES-001 | BC-2.02.003 | SS-02 Core Types and ABI | |
| BC-FACTORY-001 | BC-2.02.004 | SS-02 Core Types and ABI | FactoryAdapter is in monocle-core |
| BC-FACTORY-002 | BC-2.02.005 | SS-02 Core Types and ABI | |
| BC-PROTO-001a | BC-2.02.006 | SS-02 Core Types and ABI | |
| BC-PROTO-001b | BC-2.02.007 | SS-02 Core Types and ABI | |
| BC-PROTO-002 | BC-2.02.008 | SS-02 Core Types and ABI | |
| BC-ENGINE-001 | BC-2.03.001 | SS-03 Engine Module | |
| BC-ENGINE-002 | BC-2.03.002 | SS-03 Engine Module | |
| BC-ENGINE-002-ERR | BC-2.03.003 | SS-03 Engine Module | Note: "-ERR" suffix non-standard; use sequential NNN |
| BC-ENGINE-003 | BC-2.03.004 | SS-03 Engine Module | |

**Append-only protection note:** The old IDs (BC-DAEMON-001 etc.) MUST be preserved in BC-INDEX.md as retired entries with `replaced_by: BC-2.01.001` etc. per the append-only ID policy. The new IDs are new artifacts; old IDs are not deleted.

**Cross-reference blast radius:** The BC ID appears in:
- prd.md §2, §3, §6, §7 (RTM): 22 BC IDs × ~5 occurrences each = ~110 sites
- verification-properties.md: every VP `source_bc` field and `## Source Contract` section = ~44 sites
- SS-*.md files: §Behavioral Contract Summary tables, §Phase 1 PRD BC Pre-Staging tables = ~30 sites
- product-brief.md: Forward-compatibility contracts BC lists = ~30 sites
- dtu-assessment.md: BC-HOOK-007 citations (different BC schema; not affected)
- ADR files: BC references in Source/Origin sections = ~10 sites

Total estimated cross-reference sites for BC ID renaming: ~224 sites across all artifacts.

---

## VP ID Renumbering Map

Current VP IDs use `VP-DOMAIN-NNN` scheme. Template specifies `VP-NNN` (sequential, no subsystem component). VP numbering is simpler than BC numbering.

| Current ID | Proposed Template ID | Source BC (current) | Source BC (new) |
|------------|---------------------|---------------------|-----------------|
| VP-DAEMON-001 | VP-001 | BC-DAEMON-001 | BC-2.01.001 |
| VP-DAEMON-002 | VP-002 | BC-DAEMON-002 | BC-2.01.002 |
| VP-DAEMON-003 | VP-003 | BC-DAEMON-003 | BC-2.01.003 |
| VP-DAEMON-004 | VP-004 | BC-DAEMON-004 | BC-2.01.004 |
| VP-DAEMON-005 | VP-005 | BC-DAEMON-005 | BC-2.01.005 |
| VP-DAEMON-006 | VP-006 | BC-DAEMON-006 | BC-2.01.006 |
| VP-RING-001 | VP-007 | BC-RING-001 | BC-2.01.007 |
| VP-AUTH-001 | VP-008 | BC-AUTH-001 | BC-2.01.008 |
| VP-AUTH-002 | VP-009 | BC-AUTH-002 | BC-2.01.009 |
| VP-LOCK-001 | VP-010 | BC-LOCK-001 | BC-2.01.010 |
| VP-ABI-001 | VP-011 | BC-ABI-001 | BC-2.02.001 |
| VP-ABI-002 | VP-012 | BC-ABI-002 | BC-2.02.002 |
| VP-TYPES-001 | VP-013 | BC-TYPES-001 | BC-2.02.003 |
| VP-FACTORY-001 | VP-014 | BC-FACTORY-001 | BC-2.02.004 |
| VP-FACTORY-002 | VP-015 | BC-FACTORY-002 | BC-2.02.005 |
| VP-PROTO-001a | VP-016 | BC-PROTO-001a | BC-2.02.006 |
| VP-PROTO-001b | VP-017 | BC-PROTO-001b | BC-2.02.007 |
| VP-PROTO-002 | VP-018 | BC-PROTO-002 | BC-2.02.008 |
| VP-ENGINE-001 | VP-019 | BC-ENGINE-001 | BC-2.03.001 |
| VP-ENGINE-002 | VP-020 | BC-ENGINE-002 | BC-2.03.002 |
| VP-ENGINE-002-ERR | VP-021 | BC-ENGINE-002-ERR | BC-2.03.003 |
| VP-ENGINE-003 | VP-022 | BC-ENGINE-003 | BC-2.03.004 |

**Append-only protection note:** Same principle as BC renaming — old VP IDs preserved in VP-INDEX.md as retired entries with `replaced_by: VP-NNN`.

---

## Remediation Plan

The content is correct; the structure is wrong. Remediation is a pure structural migration.
Ordered by dependency (each step unblocks the next):

### Dispatch 1: Architect — Create ARCH-INDEX.md + Populate Subsystem Registry

**Agent:** `vsdd-factory:architect`
**Scope:**
- Create `.factory/specs/architecture/ARCH-INDEX.md` from `architecture-index-template.md`
- Populate Subsystem Registry with SS-01..SS-04 per §MISS-03 above
- Populate Document Map table with the 7 SS-* files
- Note support artifacts (SS-deps-pin-manifest, SS-conventions, SS-forward-compat) as architectural support docs, not runtime subsystem sections
- Update `traces_to:` in all 7 SS-* files to reference `ARCH-INDEX.md`
- Fix `document_type:` in SS-core-types-and-abi.md (→ `architecture-section`), SS-deps-pin-manifest.md (→ `architecture-section`), SS-permissions-phase1.md (→ `architecture-section`)

**Estimated effort:** 1 focused burst
**Blocks:** All subsequent dispatches (subsystem IDs required for BC renumbering)

---

### Dispatch 2: Product Owner — Shard PRD into BC Index + Supplement Files

**Agent:** `vsdd-factory:product-owner`
**Scope:**
1. Create `prd-supplements/` directory with 4 supplement files:
   - `error-taxonomy.md` — extract PRD §5 content
   - `nfr-catalog.md` — extract PRD §4 content (12 NFR rows)
   - `test-vectors.md` — extract per-BC canonical test vector tables from PRD §3
   - `interface-definitions.md` — synthesize from PRD §3 endpoint definitions + SS-daemon-lifecycle endpoint tables
2. Restructure `prd.md` §2 as a pure BC index table (BC ID, Title, Priority, file path) — remove all inline BC content from §3
3. Update PRD frontmatter: `supplements:` populated with 4 supplement file paths; `input-hash` computed
4. Add PRD §5b test-vectors supplement reference (currently missing)
5. Do NOT renumber BC IDs yet — that is Dispatch 4

**Estimated effort:** 2 bursts (supplement extraction is mechanical; PRD §2 restructure requires care)
**Depends on:** Dispatch 1 (subsystem names needed for index grouping)

---

### Dispatch 3: Product Owner — Create Sharded BC Files + BC-INDEX.md

**Agent:** `vsdd-factory:product-owner`
**Scope:**
1. Create `.factory/specs/behavioral-contracts/` directory structure:
   - `BC-INDEX.md` — index of all 22 BCs with new template IDs
   - `ss-01/` — 10 BC files (daemon lifecycle, ring, auth, lock)
   - `ss-02/` — 8 BC files (ABI, types, factory, proto)
   - `ss-03/` — 4 BC files (engine module)
2. Extract each BC from PRD §3 into its own file using `behavioral-contract-template.md`
3. Rename old IDs to new template IDs per §BC ID Renumbering Map
4. Add missing frontmatter to each BC: `subsystem`, `capability`, `lifecycle_status`, `introduced`, all lifecycle fields
5. Preserve old IDs as `retired` entries in BC-INDEX.md with `replaced_by:` forward links
6. Update PRD §2 index table with new BC IDs and file paths
7. Update PRD frontmatter `input-hash` and all cross-references in PRD body

**Estimated effort:** 2 bursts (22 BC files × frontmatter + lifecycle fields + cross-reference updates)
**Depends on:** Dispatch 2 (PRD §2 must be restructured before BCs are extracted)

---

### Dispatch 4: Formal Verifier — Shard VP Monolith into Per-VP Files + VP-INDEX.md

**Agent:** `vsdd-factory:formal-verifier`
**Scope:**
1. Rename/move `.factory/specs/verification-properties.md` — cannot create `.factory/specs/verification-properties/` directory while the file occupies the namespace. File must be archived first.
2. Create `.factory/specs/verification-properties/` directory
3. Create `VP-INDEX.md`
4. Extract each VP into its own file using `L4-verification-property-template.md`
5. Rename VP IDs per §VP ID Renumbering Map
6. Add missing per-VP frontmatter: `source_bc` (using new BC IDs from Dispatch 3), `module`, `proof_method`, `feasibility`, `verification_lock: false`, `proof_completed_date: null`, `proof_file_hash: null`, all lifecycle fields
7. Update `document_type: verification-property` (singular), `level: L4`
8. Preserve §Trace history in VP-INDEX.md or a companion `vp-authoring-history.md` (trace sections are provenance, not spec content)
9. Preserve old VP IDs as retired entries in VP-INDEX.md

**Estimated effort:** 2 bursts (22 VP files each with full template frontmatter; most complex dispatch due to file count and per-VP lifecycle fields)
**Depends on:** Dispatch 3 (new BC IDs required for `source_bc` frontmatter)

---

### Dispatch 5: Business Analyst — Backfill L2 Domain Spec

**Agent:** `vsdd-factory:business-analyst`
**Scope:**
1. Create `.factory/specs/domain-spec/` directory
2. Create `L2-INDEX.md` from `L2-domain-spec-index-template.md`
3. Create section files: `capabilities.md`, `entities.md`, `invariants.md`, `events.md`, `edge-cases.md`, `assumptions.md`, `risks.md`, `failure-modes.md`, `differentiators.md`
4. Populate from existing content:
   - capabilities.md: derive CAP-NNN from PRD §2 domain groupings (Daemon Lifecycle, Core ABI, Engine Module, Permissions = 4+ CAP groups)
   - edge-cases.md: extract EC-NNN from PRD §9 Edge Case Catalog
   - assumptions.md: derive from brief §Open Questions for Architect + resolved OQ items
   - risks.md: R-001 from brief §Overflow Context
5. Update PRD frontmatter `traces_to: domain-spec/L2-INDEX.md`

**Note:** This is a backfill, not new content authoring. Correctness is constrained by what the existing PRD and brief contain. The business-analyst should NOT introduce new BCs or change existing requirements; this is structural mapping only.

**Estimated effort:** 1 burst
**Depends on:** Dispatch 1 (subsystem assignments needed for capability-to-subsystem mapping)

---

### Dispatch 6: Architect — Compute All input-hash Values

**Agent:** `vsdd-factory:architect` (or `vsdd-factory:devops-engineer`)
**Scope:**
- Run `bin/compute-input-hash --update` to compute actual MD5 hashes for all spec artifacts
- Replace all `"[live-state]"` values with computed hashes
- This enables drift detection via `bin/compute-input-hash --scan .factory`

**Estimated effort:** 1 burst (mechanical tool invocation + frontmatter updates)
**Depends on:** All content dispatches complete (hashes must be stable after content migration)

---

## Risk Assessment

### Risk 1: Phase 2 story-writer pipeline entry will fail

**Severity: CRITICAL**
**Probability: CERTAIN if not remediated before Phase 2**

The `vsdd-factory:decompose-stories` skill (Phase 2) reads `behavioral-contracts/BC-INDEX.md` and loads individual BC files. These do not exist. The story-writer agent will either fail to start or produce empty story files. Phase 2 cannot begin until MISS-01 (behavioral-contracts/) is resolved.

### Risk 2: Phase 6 formal harness tooling cannot locate VP files

**Severity: CRITICAL**
**Probability: CERTAIN if not remediated before Phase 6**

The `vsdd-factory:formal-verify` skill and Phase 6 harnesses expect `verification-properties/vp-NNN-*.md` files. The monolith at `verification-properties.md` is not consumable by these tools. VP-DAEMON-001 through VP-ENGINE-003 are valid and complete; they simply cannot be located by path-based tooling.

### Risk 3: Append-only ID enforcement cannot be activated

**Severity: HIGH**
**Probability: CERTAIN if IDs are not migrated before BC-INDEX.md is created**

The VSDD governance tooling enforces append-only IDs. Once BC-INDEX.md is created with BC-DAEMON-001 style IDs, any subsequent attempt to renumber will create a governance violation. The renumbering must happen BEFORE BC-INDEX.md is committed, not after. The remediation plan above sequences this correctly (Dispatch 2 restructures PRD, Dispatch 3 creates files with new IDs).

### Risk 4: Cross-reference blast radius during ID migration

**Severity: HIGH**
**Probability: LOW if migration is done with automated grep-replace**

~224 cross-reference sites across 15+ files must be updated when BC IDs change. Manual editing risks missed references. The implementing agent should use `sed` or `grep -rn` sweeps per the SE-16c canonical-grep discipline already established in this project. The existing `input-hash` drift detection (once activated in Dispatch 6) will catch missed updates.

### Risk 5: verification-properties.md namespace collision

**Severity: HIGH**
**Probability: CERTAIN — must be planned explicitly**

The file `.factory/specs/verification-properties.md` occupies the path that `.factory/specs/verification-properties/` needs. Git operations must:
1. `git mv .factory/specs/verification-properties.md .factory/specs/verification-properties-archive.md` (or similar)
2. Create `.factory/specs/verification-properties/` directory
3. Commit the archive file before creating the directory

The archive file preserves the full §Trace history (35 trace sections = provenance). It should be retained as `vp-authoring-history.md` inside the new directory or moved to `.factory/cycles/`.

### Risk 6: PRD §Trace history bloat

**Severity: MEDIUM**
**Probability: INFORMATIONAL — will not block Phase 2**

The PRD contains 25 §Trace sections (v1.0 through v1.25, lines 1442-4480). These account for ~3,000 of the 4,480 lines. They are authoring provenance, not specification content. They should be migrated to `.factory/cycles/` as part of the `compact-state` pattern (`vsdd-factory:compact-state`). This does not block Phase 2 but contributes to context window pressure when agents load the full PRD.

---

## Aggregate Summary Table

| Template | Artifact(s) | Frontmatter | Sections | Tables | Overall |
|----------|-------------|-------------|----------|--------|---------|
| `prd-template.md` | `prd.md` | FAIL | WARN | WARN | FAIL |
| `behavioral-contract-template.md` | MISSING (22 files expected) | — | — | — | MISSING |
| `L4-verification-property-template.md` | `verification-properties.md` (wrong pattern) | FAIL | FAIL | FAIL | FAIL |
| `product-brief-template.md` | `product-brief.md` | WARN | WARN | PASS | WARN |
| `dtu-assessment-template.md` | `dtu-assessment.md` | FAIL | WARN | PASS | FAIL |
| `architecture-index-template.md` | MISSING (`ARCH-INDEX.md`) | — | — | — | MISSING |
| `architecture-section-template.md` | SS-daemon-lifecycle.md | FAIL | WARN | N/A | FAIL |
| `architecture-section-template.md` | SS-core-types-and-abi.md | FAIL | WARN | N/A | FAIL |
| `architecture-section-template.md` | SS-engine-module.md | FAIL | WARN | N/A | FAIL |
| `architecture-section-template.md` | SS-deps-pin-manifest.md | FAIL | WARN | N/A | FAIL |
| `architecture-section-template.md` | SS-conventions-anti-patterns.md | FAIL | WARN | N/A | FAIL |
| `architecture-section-template.md` | SS-forward-compatibility.md | FAIL | WARN | N/A | FAIL |
| `architecture-section-template.md` | SS-permissions-phase1.md | FAIL | WARN | N/A | FAIL |
| `adr-template.md` | ADR-0001 | PASS | WARN | N/A | WARN |
| `adr-template.md` | ADR-0002 | PASS | WARN | N/A | WARN |
| `adr-template.md` | ADR-0003 | PASS | PASS | N/A | PASS |
| `adr-template.md` | ADR-0004 | PASS | WARN | N/A | WARN |
| `prd-supplement-error-taxonomy-template.md` | MISSING | — | — | — | MISSING |
| `prd-supplement-nfr-catalog-template.md` | MISSING | — | — | — | MISSING |
| `prd-supplement-interface-definitions-template.md` | MISSING | — | — | — | MISSING |
| `prd-supplement-test-vectors-template.md` | MISSING | — | — | — | MISSING |
| `L2-domain-spec-index-template.md` | MISSING (`domain-spec/L2-INDEX.md`) | — | — | — | MISSING |
| `L2-domain-spec-section-template.md` | MISSING (9 section files) | — | — | — | MISSING |
