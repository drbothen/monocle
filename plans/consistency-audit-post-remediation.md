---
document_type: consistency-audit-report
level: ops
version: "2.0"
status: complete
producer: consistency-validator (fresh context, post-remediation, production-grade lens)
phase: pre-phase-1-final-gate-post-remediation
timestamp: 2026-05-13T01:16:15Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
  - /Users/jmagady/Dev/monocle/.factory/tech-debt-register.md
  - /Users/jmagady/Dev/monocle/.factory/planning/market-intelligence.md
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
  - /Users/jmagady/Dev/monocle/CLAUDE.md
input-hash: "[live-state]"
traces_to: "post-remediation burst close-out commit 63d5a54; production-grade re-audit 0bd4ba9; canonical principle CLAUDE.md commits b69c09f/3366d58"
project: monocle
verdict: GAPS_FOUND
---

# Consistency Audit — Post-Remediation (Production-Grade Lens) v2.0

## 1. Summary

| Category | Verdict | Findings |
|----------|---------|----------|
| Defer-pattern scan | PASS — no functional defer patterns remaining | 0 active Placeholder/Pending/TBD; 2 ADVISORY-only markers noted |
| Cross-reference integrity | GAPS_FOUND | 3 findings (1 BLOCKING, 2 IMPORTANT) |
| Numerical consistency | GAPS_FOUND | 2 findings (1 BLOCKING, 1 IMPORTANT) |
| Naming consistency | PASS | 0 findings |
| R-001 consistency | PASS | All 4 artifacts consistent |
| Frontmatter / template compliance | GAPS_FOUND | 3 findings (1 IMPORTANT, 2 ADVISORY) |
| Overall verdict | GAPS_FOUND | 2 BLOCKING, 3 IMPORTANT, 2 ADVISORY |

**Finding counts by severity:**

| Severity | Count |
|----------|-------|
| BLOCKING | 2 |
| IMPORTANT | 3 |
| ADVISORY | 2 |
| TOTAL | 7 |

Under the production-grade lens (CLAUDE.md §Canonical Principle), all findings below are assessed against whether they represent AI-introduced defects that a specialist could fix in-scope. The 2 BLOCKING findings are cross-reference breaks that would silently mislead the architect at Phase 1 entry. The 3 IMPORTANT findings are structural imprecisions that a specialist can resolve without human direction.

---

## 2. Defer-Pattern Scan Results

Scanned all 12 artifacts for: "Placeholder for architect", "Pending architect review", "pending architect", "Architect TODO", "MVP", "for now", "good enough", "we can fix later", "ship fast", "minimum viable", "TBD", "TODO".

| Pattern Category | Result | Detail |
|-----------------|--------|--------|
| "Placeholder for architect" | PASS — ZERO occurrences | Removed per remediation burst |
| "Pending architect review" | PASS — ZERO live occurrences | Historical revision-history mention in brief v1.3 row (traceability prose, not an active deferral) |
| "pending architect" | PASS — ZERO live occurrences | Same historical note |
| "Architect TODO" | PASS — ZERO occurrences | |
| "TBD" | PASS — ZERO occurrences | |
| "TODO" in spec artifacts | PASS — ZERO active TODO items | STATE.md mentions "6 TODOs resolved" and "5 TODOs" in DONE step descriptions — these are historical bookkeeping, not open items |
| "MVP" | ADVISORY — 2 occurrences | See finding F-07-A below; both are in explanatory prose that contextualizes phasing decisions, not deferral rationalizations |
| "minimum viable" | ADVISORY — 1 occurrence | See finding F-07-A; brief §Phase Plan Rationale line 353: "This is the minimum viable product for the killer scenario" — engineering explanation of phase scope, not a deferral marker |
| "for now" | PASS — ZERO occurrences | |
| "good enough" | PASS — ZERO occurrences | |
| "ship fast" | PASS — ZERO occurrences | |
| "we can fix later" | PASS — ZERO occurrences | |
| "first-pass" (scope-limit marker) | IMPORTANT — 1 occurrence | See finding F-02-I in Cross-Reference section |

**Defer-pattern verdict:** CLEAN — no functional defer patterns. 2 ADVISORY occurrences of "minimum viable" / "MVP" in explanatory prose noted, evaluated below.

---

## 3. Cross-Reference Integrity

| Ref | Check | Artifact | Verdict | Detail |
|-----|-------|----------|---------|--------|
| CR-01 | Brief supplements: all 6 paths exist on disk | brief frontmatter `supplements:` | PASS | All 6 paths verified: SS-deps-pin-manifest.md, ADR-0001, SS-conventions-anti-patterns.md, tech-debt-register.md, ADR-0002, dtu-assessment.md |
| CR-02 | OQ-M1/M2/M3 consistent across brief, OQ table, vision Closure Log | brief §Open Questions table + vision §Closure Log | PASS | All three OQs resolved consistently: OQ-M1 = no IPC collision, OQ-M2 = claude-manager not hook-protocol, OQ-M3 = stay at 5 endpoints |
| CR-03 | Crate list: brief §Constraints names 11 named crates | brief line 232 | PASS | List: monocle-core, monocle-runtime, monocle-tui, monocle-static, monocle-workflow, monocle-plugin-sdk, monocle-ipc, monocle-config, monocle-proto, monocle-fuzz, monocle-test-harness = 11 named |
| CR-04 | Crate list: vision §Workspace Layout names 11 named crates | vision lines 86-96 | PASS | Same 11 crates listed in same order |
| CR-05 | ADR-0001 references `dependencies.md` path — does that file exist? | ADR-0001 lines 73, 84 | BLOCKING | `dependencies.md` referenced in ADR-0001 §Consequences and §Source/Origin. File does NOT exist. The canonical file is `SS-deps-pin-manifest.md`. Path was renamed but ADR-0001 was not updated. Architect reading ADR-0001 will follow a dead link. |
| CR-06 | Vision §Tech Stack references `dependencies.md` path | vision line 356 + inputs frontmatter line 19 + traces_to line 22 | BLOCKING | Vision frontmatter `inputs:` lists `/Users/jmagady/Dev/monocle/.factory/specs/architecture/dependencies.md` (does not exist). Vision §Tech Stack body text says "see `.factory/specs/architecture/dependencies.md` (`SS-deps-pin-manifest.md` after path migration)" — the in-body annotation is correct but the frontmatter input path is wrong. Vision `traces_to` also says "dependencies.md as canonical pin source". Three references to the dead path in a single document. |
| CR-07 | ADR-0002 traces_to references tech-debt-register — valid? | ADR-0002 frontmatter | PASS | References `tech-debt-register.md` TD-001 retirement; TD-001 correctly appears in Resolution History |
| CR-08 | Tech-debt-register Resolution History matches ADR-0002 §Supersedes | tech-debt-register + ADR-0002 | PASS | Both correctly state TD-001 retired by ADR-0002 nucleo acceptance decision |
| CR-09 | DTU assessment OQ-M1/OQ-M3 resolution consistent with brief | dtu-assessment.md §Services NOT Requiring DTU row 5 | PASS | Row 5: "Claude Code agent view IPC — Per OQ-M1 resolution: agent view uses Claude Code's internal IPC" — consistent with brief OQ-M1 resolution |
| CR-10 | DTU assessment hook endpoint count consistent | dtu-assessment.md | PASS | "Claude Code hook protocol (5 endpoints)" appears consistently throughout the assessment |
| CR-11 | oq-research.md `traces_to` contains `<forthcoming>` placeholder commit ref | oq-research.md frontmatter line 21 | IMPORTANT | `traces_to: "brief v1.4 commit <forthcoming>"` — `<forthcoming>` is a placeholder that was never resolved. The actual brief v1.4 commit is `70286e1`. An architect reading the frontmatter chain cannot follow the trace. |
| CR-12 | vision `approved_at` timestamp matches re-approval date | vision frontmatter | IMPORTANT | `approved_at: 2026-05-11T20:30:00Z` — this is the v1.0 approval timestamp. The v1.1 re-approval occurred 2026-05-12 (per STATE.md, CLAUDE.md, vision §Provenance last paragraph). The frontmatter was not updated when v1.1 was created. The timestamp is systematically misleading: anyone reading the frontmatter will believe the last approval was 2026-05-11 rather than 2026-05-12. |

---

## 4. Numerical Consistency

| Check | Expected | Brief | Vision | SS-deps | SS-conv | DTU | CLAUDE.md | Verdict |
|-------|----------|-------|--------|---------|---------|-----|-----------|---------|
| Hook endpoints (Phase 1) | 5 | 5 (SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop) | 5 (diagram shows same set; prose §Process Topology correct) | N/A (not enumerated) | N/A | 5 (same list in endpoint matrix table) | 5 (per R-001 note) | PASS — all consistent |
| Named crates | 11 | 11 (named list at line 232) | 11 (workspace layout lines 86-96) | 11 named + 1 binary = 12 total (line 121) | N/A | N/A | N/A | PASS — all consistent |
| Total crate count | 12 | 12 ("11 named + 1 binary") | Closure Log EX-1 reference: "see brief v1.4 for correct count" — no explicit 12 count in workspace layout | 12 explicit (line 121) | N/A | N/A | N/A | PASS |
| MSRV Phase 1 | Rust 1.86 | 1.86 (lines 133, 275) | N/A (pointer to deps manifest) | 1.86 (MSRV table) | N/A | ADR-0001 says "Phase 3 MSRV bumps from 1.86 to 1.92" (not Phase 1 explicit, but derivable) | 1.86 (line 36) | PASS |
| MSRV Phase 3 | Rust 1.92 | 1.92 (line 155) | N/A | 1.92 (MSRV table) | N/A | ADR-0001 line 65: "Phase 3 MSRV bumps from 1.86 to 1.92 due to wasmtime requirements" | 1.92 (line 36) | PASS |
| EXACT-pinned crates count | 7 declared | N/A | N/A | BLOCKING — see F-03-B below | N/A | N/A | N/A | BLOCKING |
| EXACT-pinned crate list | tokio, axum, wasmtime, russh, rmcp, reqwest + 1 unresolved | N/A | N/A | "russh" listed TWICE; 7th slot is "to be confirmed at Cargo workspace init" | N/A | N/A | N/A | IMPORTANT |

**Finding F-03-B (BLOCKING) — EXACT-pin policy is self-contradictory with an open slot:**

SS-deps-pin-manifest.md §Patch-Pinning Policy line 93 states:
> "EXACT pin (`=x.y.z`) for the 7 security-sensitive crates: `tokio`, `axum`, `russh`, `wasmtime`, `rmcp`, `reqwest`, `russh`."

Two defects in this one sentence:
1. `russh` appears **twice** (the 7th slot is a duplicate of the 4th slot). The actual named list has only 6 distinct crates.
2. Line 95 then says: "The 7 EXACT-pinned crates are: `tokio`, `axum`, `wasmtime`, `russh`, `rmcp`, `reqwest`, and **one additional entry to be confirmed at Cargo workspace init** if any further crate handles untrusted input at the network boundary."

The "to be confirmed at Cargo workspace init" is a deferred decision in a spec that claims to be authoritative — exactly the pattern CLAUDE.md Rule 6 forbids for answerable questions. The manifest table already has a clear answer: `axum` handles untrusted network input as the HTTP server for hook ingestion. `prost` (caret-pinned) handles deserialization of untrusted network bytes from Claude Code. Looking at the manifest, `prost` is caret-pinned but receives hook POST bodies from potentially malicious Claude Code subprocesses in adversarial scenarios — it is a candidate for the 7th slot. The question is answerable now.

This is a BLOCKING finding under the production-grade lens because: (a) `russh` duplicate makes the policy list wrong, and (b) the "to be confirmed" defers an architectural security decision to a Cargo init step that has no spec gate.

**Recommended fix (route to `vsdd-factory:architect`):** Replace line 93 with the resolved 7-crate list: `tokio`, `axum`, `wasmtime`, `russh`, `rmcp`, `reqwest`, `prost` (or make an explicit decision that `prost` is caret-pinned by design with documented rationale). Remove the `russh` duplicate. Eliminate the "to be confirmed" slot.

---

## 5. Naming Consistency

| Check | Convention | Brief | Vision | SS-conv | SS-deps | CLAUDE.md | Verdict |
|-------|------------|-------|--------|---------|---------|-----------|---------|
| Product name in code | lowercase `monocle` | PASS (crate names all lowercase) | PASS | PASS (authoritative) | PASS | PASS | PASS |
| Product name in prose | capitalized `Monocle` | PASS (headings use Monocle) | PASS | PASS | N/A | PASS | PASS |
| Five-plane names: Runtime, Static, Workflow, Harness, TUI | Consistent names | PASS | PASS | N/A | N/A | PASS | PASS |
| `VsddFactoryAdapter` spelling | VsddFactoryAdapter | PASS (used consistently) | PASS | N/A | N/A | PASS | PASS |
| `FactoryAdapter` (trait) vs `VsddFactoryAdapter` (impl) | Both used correctly | PASS — trait = FactoryAdapter; impl = VsddFactoryAdapter | PASS | N/A | N/A | N/A | PASS |
| "claude agents" vs "agent view" | Both used; "claude agents" is product name, "agent view" is feature name | PASS — brief and market-intel use both correctly ("`claude agents` (agent view, v2.1.139)") | N/A | N/A | N/A | PASS ("agent view" used) | PASS |

**Naming verdict: PASS — zero naming consistency violations.**

---

## 6. R-001 Consistency

| Artifact | R-001 Probability | Source Attribution | Status |
|----------|------------------|-------------------|--------|
| market-intelligence.md §Risk Register | MEDIUM (25–40%) | Historical snapshot from pre-human-Q-B | PASS — expected as point-in-time historical artifact; explicitly noted in audit scope as "DO NOT FLAG" |
| vision §Closure Log | <10% | "human Q-B response (2026-05-12)"; attributes to market-intel as original source | PASS |
| brief v1.4.1 §Competitive Positioning | <10% | "(per `.factory/planning/market-intelligence.md` §Risk Register, originally assessed at 25–40%; human red-line at v1.4.1 brief gate revised this to <10%)" | PASS |
| CLAUDE.md §Architectural Authority | <10% | "R-001 (Anthropic commoditization risk) reassessed at <10% probability; informational only" | PASS |

**R-001 verdict: PASS — all four artifacts consistent with human Q-B resolution. Market-intel correctly preserved as historical snapshot; brief correctly cross-references it with the revision note.**

---

## 7. Frontmatter / Template Compliance

| Artifact | document_type | level | version | status | producer | timestamp | inputs | input-hash | traces_to | project | Verdict |
|----------|--------------|-------|---------|--------|----------|-----------|--------|------------|-----------|---------|---------|
| product-brief.md | product-brief | L1 | 1.4.1 | draft | product-owner | present | present | [live-state] | present | monocle | PASS — note: `status: draft` is expected for a brief at pre-phase-1 |
| vision-synthesis.md | vision-synthesis | ops | 1.1 | approved | orchestrator | present | present | [live-state] | present | monocle | IMPORTANT — see F-05-I below (approved_at stale) |
| SS-deps-pin-manifest.md | architecture-dependencies | L3 | 1.1 | complete | architect | present | present | [live-state] | present | monocle | PASS — has `[Section Content]` heading (historical template artifact, not a defer marker) |
| SS-conventions-anti-patterns.md | architecture-section | L3 | 1.1 | complete | architect | present | present | [live-state] | present | monocle | PASS — same `[Section Content]` heading noted |
| ADR-0001 | adr | L3 | 1.0 | accepted | product-owner (extracted from brief v1.1) | present | present | [live-state] | present | monocle | ADVISORY — producer is "product-owner (extracted from brief v1.1)"; after remediation, ADR-0001 was not re-attributed to architect despite being an architectural decision. Minor; does not affect correctness. |
| ADR-0002 | adr | L3 | 1.0 | accepted | architect | present | present | [live-state] | present | monocle | PASS |
| dtu-assessment.md | dtu-assessment | L3 | 1.0 | complete | architect | present | present | [live-state] | present | monocle | PASS |
| tech-debt-register.md | tech-debt-register | ops | 1.0 | active | product-owner | present via `last_updated` field (not `timestamp` field) | MISSING | [live-state] would apply | present | monocle | ADVISORY — `inputs:` and `timestamp:` fields absent; `last_updated` substitutes but differs from canonical frontmatter schema. `inputs: []` (empty) would be appropriate since this is a register, not a derived artifact. |
| market-intelligence.md | market-intelligence-assessment | L1 | 1.0 | complete | business-analyst | present | present | [live-state] | present | monocle | PASS (historical snapshot; out of scope for correction per audit instructions) |
| oq-research.md | open-questions-research | ops | 1.0 | draft | research-agent | present | present | [live-state] | IMPORTANT — see F-06-I | monocle | IMPORTANT |
| STATE.md | pipeline-state | ops | 2.0 | active | state-manager | present | inputs: [] (correct for state doc) | [live-state] | empty string (traces_to: "") | monocle | ADVISORY — `traces_to: ""` is an empty string; convention allows this for state docs but should be a note or "N/A" rather than empty |
| CLAUDE.md | Not a VSDD artifact — project instructions file | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A | PASS — CLAUDE.md is not subject to VSDD frontmatter requirements |

---

## 8. Findings Table

| ID | Severity | Category | Artifact | Location | Finding | Fix | Route To |
|----|----------|----------|----------|----------|---------|-----|----------|
| F-01-B | BLOCKING | Cross-Reference | ADR-0001 + vision | ADR-0001 lines 73, 84; vision frontmatter inputs line 19, traces_to line 22, body line 356 | Dead path `dependencies.md` referenced in ADR-0001 (2 occurrences) and vision (3 occurrences — inputs frontmatter, traces_to, body text). File does not exist. Canonical file is `SS-deps-pin-manifest.md`. Architect reading ADR-0001 §Source/Origin or vision §Tech Stack follows a dead link. This is a structural break in the spec-chain traceability that will silently mislead the Phase 1 architect. | (1) In ADR-0001 line 73: replace "dependencies.md" with "SS-deps-pin-manifest.md". (2) In ADR-0001 line 84: replace "dependencies.md" with "SS-deps-pin-manifest.md". (3) In vision frontmatter `inputs:` line 19: replace path with `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md`. (4) In vision `traces_to` line 22: replace "dependencies.md" with "SS-deps-pin-manifest.md". (5) In vision body line 356: update both references from "dependencies.md" to "SS-deps-pin-manifest.md". | architect (ADR-0001 fix) + business-analyst (vision fix); via orchestrator |
| F-02-B | BLOCKING | Numerical | SS-deps-pin-manifest | §Patch-Pinning Policy lines 93–95 | EXACT-pin list has `russh` duplicated as both 4th and 7th crate (the "7th entry" is a repeat). The policy simultaneously declares "7 crates" and leaves the 7th as "to be confirmed at Cargo workspace init" — a deferred decision in a canonical authority document. This contradicts CLAUDE.md Rule 6 (forbidden for answerable questions). The correct 7th crate is determinable now from the manifest context: `prost` deserializes untrusted hook POST bodies at the network boundary, qualifying it for EXACT-pin. | Replace §Patch-Pinning Policy lines 93–95 with resolved list: remove the `russh` duplicate; explicitly name the 7th crate or make a documented decision that prost is caret-pinned by design with rationale. If `prost` is chosen as 7th: update Phase 1 Pin Manifest prost row from "caret pin" to "EXACT pin" with policy rationale. Eliminate "to be confirmed" language. | architect |
| F-03-I | IMPORTANT | Cross-Reference | oq-research.md | frontmatter `traces_to` line 21 | `traces_to: "brief v1.4 commit <forthcoming>; ..."` — `<forthcoming>` was never resolved to an actual commit SHA. The actual brief v1.4 commit is `70286e1`. Traceability chain from oq-research to brief v1.4 is broken. | Replace `<forthcoming>` with `70286e1`. | research-agent (or any agent with write access to oq-research.md, routed via orchestrator) |
| F-04-I | IMPORTANT | Frontmatter | vision | frontmatter `approved_at` line 25 | `approved_at: 2026-05-11T20:30:00Z` is the v1.0 approval timestamp. v1.1 was re-approved by the human on 2026-05-12. The frontmatter was not updated when v1.1 was created, creating a systematic discrepancy: STATE.md, CLAUDE.md, and vision §Provenance all state "re-approved 2026-05-12" but the frontmatter says 2026-05-11. Any automated tool reading `approved_at` will report the wrong approval date. | Update `approved_at` to `2026-05-12T00:00:00Z` (or more precisely, the commit timestamp of 0e4b0f4 if known). | business-analyst (vision owner) |
| F-05-I | IMPORTANT | Cross-Reference | SS-deps-pin-manifest | §Workspace Dependency Graph header comment (line 125) | "First-pass diagram showing the 12-crate Phase 1 workspace... Mark: first-pass; refine when Cargo workspace is initialized." The "first-pass; refine" qualifier signals the diagram is not yet complete or authoritative. Under the production-grade lens, this is either: (a) an accurate scope note that the diagram is preliminary, or (b) a residual deferral marker. The diagram itself appears complete and consistent with the brief — the crate nodes match the 11 named crates. The "first-pass" qualifier is ADVISORY at worst, IMPORTANT if it causes the architect to treat the dependency graph as non-authoritative and re-derive it from scratch rather than using it as the spec input. | Evaluate whether the diagram is production-ready for Phase 1 architect consumption. If yes: remove "first-pass; refine" qualifier and the final sentence, replacing with "Authoritative for Phase 1. Update when Cargo workspace is initialized." If genuinely preliminary (e.g., edges are uncertain): document which edges are uncertain. | architect |
| F-06-A | ADVISORY | Naming (minor) | brief §Phase Plan Rationale | line 353-354 | "This is the minimum viable product for the killer scenario — permission prompt dispatch without context-switching." The phrase "minimum viable product" (MVP) appears in an explanatory context (describing what Phase 1 achieves relative to the killer scenario, not as a quality limitation). This is an engineering scope description, not a deferral rationalization. The production-grade principle flags MVP as a smell, and this occurrence should be reviewed. However, the sentence's intent is correct: Phase 1 is the minimum FEATURE SET (not minimum quality) for the core use case. | Recommended: rephrase to avoid the MVP phrase to prevent future confusion. Suggested: "Phase 1 ships the daemon + hook ingestion + sessions panel — the complete Phase 1 delivery contract for the killer scenario." This removes the MVP framing while preserving the scoping intent. | product-owner |
| F-07-A | ADVISORY | Frontmatter | tech-debt-register | frontmatter | `inputs:` field absent; `timestamp:` field absent (only `last_updated:` present). `traces_to` is present but points only to CLAUDE.md and ADR-0002, not to an input artifact. The register is a live document, not a derived artifact, so `inputs: []` is appropriate — but the field should be present for schema consistency. | Add `inputs: []` and `timestamp: 2026-05-12T00:00:00Z` (using existing `last_updated` value) to align with canonical frontmatter schema. | product-owner (register owner) |

---

## 9. Verdict and Recommendation

### Verdict: GAPS_FOUND

**2 BLOCKING findings** prevent a clean gate pass:

1. **F-01-B** — Dead path `dependencies.md` in ADR-0001 (2 refs) and vision (3 refs). The canonical dependency manifest was renamed to `SS-deps-pin-manifest.md` but the old path was not updated in 5 locations across 2 documents. An architect entering Phase 1 will follow a dead link.

2. **F-02-B** — EXACT-pin policy in SS-deps-pin-manifest is self-contradictory: `russh` appears twice (duplicate), and the "7th crate" is left as "to be confirmed at Cargo workspace init." This violates CLAUDE.md Rule 6. The 7th slot is answerable now.

**3 IMPORTANT findings** should be fixed before Phase 1 entry but do not block the adversary fresh pass if the orchestrator accepts risk:

3. **F-03-I** — oq-research.md `traces_to` contains unresolved `<forthcoming>` placeholder for brief v1.4 commit (answer: `70286e1`).
4. **F-04-I** — vision `approved_at` frontmatter shows v1.0 date (2026-05-11) rather than v1.1 re-approval date (2026-05-12).
5. **F-05-I** — SS-deps-pin-manifest workspace dependency graph labeled "first-pass; refine" — qualifier should be resolved or removed.

**2 ADVISORY findings** are minor quality items that do not block any gate:

6. **F-06-A** — "minimum viable product" phrase in brief §Phase Plan Rationale is an engineering scope description, not a deferral rationalization. Rephrase recommended.
7. **F-07-A** — tech-debt-register missing `inputs:` and `timestamp:` frontmatter fields.

### Recommendation

**Fix the 2 BLOCKING findings before proceeding to adversary fresh pass.**

Route as follows (per CLAUDE.md §Agent Routing Table):
- F-01-B (dead `dependencies.md` path in ADR-0001 + vision) → `vsdd-factory:architect` for ADR-0001; `vsdd-factory:business-analyst` for vision frontmatter/body. Both fixes are mechanical path string substitutions — no human decision required.
- F-02-B (EXACT-pin list duplicate + unresolved 7th slot) → `vsdd-factory:architect`. Decision: either name `prost` as the 7th EXACT-pin with rationale (recommended given it deserializes untrusted hook POST bytes), or explicitly document that only 6 crates are EXACT-pinned and update the count to 6.

The IMPORTANT findings (F-03-I, F-04-I, F-05-I) should be batched into the same fix burst. F-03-I (a commit SHA fill-in) and F-04-I (a timestamp update) are single-line changes.

**After BLOCKING fixes are applied, proceed to:**
1. validate-brief v4 against brief v1.4.1 (expect VALID — no content changes required)
2. adversary fresh pass (fresh context, different model, expect PRODUCTION_READY)
3. Human Phase 1 entry approval gate

The core spec package is otherwise production-grade: all 14 defer-patterns from the prior adversary re-audit have been resolved, R-001 is consistent across all artifacts, the crate count is accurate at 12, all 5 hook endpoints are named consistently, TD-001 retirement is clean, and the tech-debt register governance is correct.
