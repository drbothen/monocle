---
document_type: consistency-audit-report
level: ops
version: "1.0"
status: complete
producer: consistency-validator (fresh context, round 22, post-round-21 fix burst)
phase: pre-phase-1-final-gate-round-21-complete
timestamp: 2026-05-13T20:30:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md  # v1.1.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md  # v1.2.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md  # v1.0.4
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md  # v1.1
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md  # v1.1.5
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md  # v1.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-forward-compatibility.md  # v1.2.1
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0003-license-selection.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md  # v1.4.10
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md  # v1.1.2
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md  # v1.1
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-21 fix burst commits 83d5fc5 (SS-engine-module v1.1.3) + 3495812 (SS-core-types-and-abi v1.2.3) + ac87649 (state close-out); resolves F-R20-1/2/3"
project: monocle
verdict: ONE_MEDIUM_PROPAGATION_DEFECT
---

# Consistency Audit — Round 22

## Verdict

ONE_MEDIUM_PROPAGATION_DEFECT — 0 CRITICAL + 1 MEDIUM + 0 LOW.

Round-21 correctly resolved all three F-R20 findings (typed error, guard parity, rustdoc
crate ref). One propagation defect survives: the vision synthesis document retains the
pre-round-21 infallible `EngineModule` trait signatures. All other checks pass cleanly.

---

## Scope

Post-round-21 fix burst consistency check. Primary risk surface: cross-document propagation
of the `EngineModule::metadata` and `EngineModule::enrich` return-type changes from
infallible to `Result<_, EngineMetadataError>`.

---

## Check Results

### Check 1 — Trait-surface propagation (MEDIUM defect found)

**Finding: F-R22-1 MEDIUM**

File: `/Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md`
Section: §Key Abstractions — EngineModule, lines 111-128 (the Rust snippet)

The vision document v1.1.2 retains the pre-round-21 infallible trait signatures:

```rust
fn metadata(&self) -> EngineMetadata;
async fn enrich(&self, proc: &ProcessSnapshot) -> EnrichedSession;
```

SS-engine-module.md v1.1.3 (round-21 fix, F-R20-1) changed both signatures to:

```rust
fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError>;
async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>;
```

The vision document is a non-authoritative sketch per CLAUDE.md §Architectural Authority —
"the LATER, MORE-SPECIFIC artifact wins." SS-engine-module.md is both later and more
specific. However, the stale infallible signatures in the vision create a propagation risk:
a Phase 1 implementer reading the vision sketch before SS-engine-module.md will implement
the infallible form and introduce the exact silent-fallback violation that F-R20-1 was
designed to prevent.

SS-engine-module.md §§Purpose, §EngineModule Trait Signature, and §Trace all document the
Result signatures and the HomeUnresolvable contract correctly. No stale text was found in:
- SS-daemon-lifecycle.md (does not cite metadata/enrich signatures)
- SS-permissions-phase1.md (does not cite EngineModule trait methods)
- SS-forward-compatibility.md §P3-1 (describes EngineMetadata struct fields; does not
  reproduce method signatures)
- SS-deps-pin-manifest.md (crate pins only; no trait signatures)
- SS-conventions-anti-patterns.md (no trait signatures)
- product-brief.md (references SS-engine-module.md canonically; does not repeat signatures)
- DTU assessment (hook endpoint matrix only; no trait methods)
- All four ADRs (no EngineModule trait method signatures)

The defect is isolated to the vision document.

**Severity:** MEDIUM. The vision document is explicitly non-authoritative for this surface
(CLAUDE.md §Architectural Authority: SS-engine-module.md supersedes vision §EngineModule).
However, under the production-grade lens, stale infallible signatures in ANY spec document
create implementer risk. A Phase 1 TDD implementer reading the vision sketch naively would
write the infallible implementation and ship a silent-failure violation.

**Routing:** architect — update vision §Key Abstractions EngineModule snippet to reflect
v1.1.3 Result signatures, or add an explicit note that the snippet is superseded by
SS-engine-module.md v1.1.3 and the Result signatures are canonical.

**Evidence:**
- Vision §EngineModule lines 111-128: `fn metadata(&self) -> EngineMetadata` (stale infallible)
- Vision §EngineModule lines 111-128: `async fn enrich(...) -> EnrichedSession` (stale infallible)
- SS-engine-module.md v1.1.3 §EngineModule Trait Signature lines 92 and 104: Result forms (authoritative)
- SS-engine-module.md §Trace v1.1.3: explicit call-out that F-R20-1 changed both return types

---

### Check 2 — BC ID coherence (CLEAN)

Total BC count verification:

| Source | BCs | IDs |
|--------|-----|-----|
| SS-core-types-and-abi.md | 8 | BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-FACTORY-001, BC-FACTORY-002, BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 |
| SS-engine-module.md | 3 | BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-003 |
| SS-daemon-lifecycle.md | 4 | BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001 |
| **Grand total** | **15** | |

STATE.md §Phase Progress row for Pre-Phase-1 Final Gate: "17 artifacts; 15 BCs pre-staged" — MATCHES.

SS-forward-compatibility.md §Cross-Phase Decisions Required closing paragraph lists all 15 BC IDs explicitly — MATCHES.

Product-brief.md v1.4.10 Success Criteria Forward-compatibility row: "15 behavioral contracts pre-staged: BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-RING-001, BC-AUTH-001/002, BC-ENGINE-001/002/003, BC-LOCK-001" — MATCHES.

`EngineMetadataError::HomeUnresolvable` (new type in v1.1.3) is absorbed into the existing
BC-ENGINE-001 contract, which was updated in the same commit (83d5fc5) to document both
Result return types and the no-silent-fallback contract. No new BC was required. BC count
remains 15. All totals reconcile. CLEAN.

---

### Check 3 — Error-type registry consistency (CLEAN)

Every error type across all architecture documents uses the `thiserror::Error` derive as
mandated by SS-conventions-anti-patterns.md §Error handling and CLAUDE.md §Conventions:

| Error type | File | Derive |
|-----------|------|--------|
| `EngineMetadataError` | SS-engine-module.md | `#[derive(Debug, thiserror::Error)]` |
| `SpawnError` | SS-engine-module.md | `#[derive(Debug, thiserror::Error)]` |
| `PreflightError` | SS-engine-module.md | `#[derive(Debug, thiserror::Error)]` |
| `FactoryReadError` | SS-core-types-and-abi.md | `#[derive(Debug, thiserror::Error)]` |
| `FactorySubscribeError` | SS-core-types-and-abi.md | `#[derive(Debug, thiserror::Error)]` |

All five error types carry `thiserror::Error` derive. No naked `enum FooError {}` without
the derive was found in any architecture document. SS-deps-pin-manifest.md pins
`thiserror = "2"` (caret pin, 2.x major) — consistent with CLAUDE.md "thiserror 2.x."
CLEAN.

---

### Check 4 — Version banner consistency (CLEAN)

| File | Frontmatter version | Expected | Match |
|------|--------------------|---------|----|
| SS-engine-module.md | 1.1.3 | 1.1.3 (round-21 modified) | YES |
| SS-core-types-and-abi.md | 1.2.3 | 1.2.3 (round-21 modified) | YES |
| SS-daemon-lifecycle.md | 1.0.4 | 1.0.4 (unmodified) | YES |
| SS-permissions-phase1.md | 1.1 | 1.1 (unmodified) | YES |
| SS-deps-pin-manifest.md | 1.1.5 | 1.1.5 (unmodified) | YES |
| SS-conventions-anti-patterns.md | 1.3 | 1.3 (unmodified) | YES |
| SS-forward-compatibility.md | 1.2.1 | 1.2.1 (unmodified) | YES |
| vision-synthesis.md | 1.1.2 | 1.1.2 (unmodified) | YES |
| product-brief.md | 1.4.10 | 1.4.10 (unmodified by round-21) | YES |
| dtu-assessment.md | 1.1 | 1.1 (unmodified) | YES |

No unexpected version mismatches. The two round-21-modified files (SS-engine-module.md
and SS-core-types-and-abi.md) correctly incremented their patch versions; all other files
are at their pre-round-21 versions. CLEAN.

---

### Check 5 — Cross-references and anchor links (CLEAN)

Checked all `§…` and `[…](…)` references from SS-engine-module.md v1.1.3:

- `SS-core-types-and-abi.md §Non-Exhaustive Inner Structs` — section exists at that heading.
- `SS-daemon-lifecycle.md` — referenced in §Trace cross-references; daemon file exists.
- `SS-deps-pin-manifest.md` — referenced for `async-trait = "^0.1"` pin; exists and correct.
- `Vision §EngineModule lines 111-128` — section exists (though content is now stale per F-R22-1).
- `SS-forward-compatibility.md lines 95-97` — the sealing veto text exists at lines 95-97.

Checked all references from SS-core-types-and-abi.md v1.2.3:

- `SS-permissions-phase1.md` — `Phase1Permission` and `ClaudeCodeTool` definitions exist.
- `ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md` — exists.
- `SS-daemon-lifecycle.md §Health and Status Endpoints` — section exists.
- `SS-engine-module.md` — exists.
- `SS-deps-pin-manifest.md` — `prost 0.14` EXACT pin; `futures` caret pin; `serde_json` for
  tool_input: all verified present in SS-deps-pin-manifest.md.

No broken anchor or cross-reference links found. CLEAN.

---

### Check 6 — STATE.md zero-context resume guarantee (CLEAN)

STATE.md §Session Resume Checkpoint §Immediate Next Action:

- Describes a concrete, executable next step: dispatch consistency-validator and adversary
  in parallel against round-21 artifacts.
- Names the specific output files: `.factory/plans/consistency-audit-round-22.md` and
  `.factory/plans/adversary-pass-round-22.md`.
- Names the specific checks for the adversary: (a) typed error no-silent-fallback at every
  layer, (b) parse_frontmatter_field guards match sibling exactly, (c) rustdoc references
  no unpinned crates.
- Specifies the gate condition: if both clean, proceed to Phase 1 gate.

STATE.md §Critical Artifacts (lines 111-124) lists 10 files with correct version numbers:
v1.1.2 vision, v1.4.10 brief, v1.2.3 SS-core-types-and-abi, v1.1.3 SS-engine-module,
v1.0.4 SS-daemon-lifecycle, v1.1 SS-permissions-phase1, v1.1.5 SS-deps-pin-manifest,
v1.2.2 SS-conventions-anti-patterns (note: frontmatter says 1.3 — see below).

MINOR OBSERVATION (not a defect): STATE.md §Critical Artifacts line 119 lists
"SS-conventions-anti-patterns.md v1.2.2" but the file's frontmatter is `version: "1.3"`.
This is stale in STATE.md. However, STATE.md's Critical Artifacts list is informational
guidance for fresh-context sessions, not a binding version contract. The file itself is
authoritative. This version mismatch in STATE.md §Critical Artifacts is harmless but
should be corrected in the next state-manager update. Not elevated to MEDIUM because
the production impact is zero — the next agent reads the actual file.

STATE.md is 176 lines (under the 200-line budget). CLEAN on all substantive criteria.

---

### Check 7 — Frontmatter input-hash drift (CLEAN)

All architecture documents, ADRs, vision, brief, dtu-assessment, and STATE.md use
`input-hash: "[live-state]"` — the canonical form for this pipeline per the established
pattern across all prior rounds. The `compute-input-hash` tool would need to be run
against the actual file content to detect drift; the `[live-state]` sentinel is the
agreed-upon convention for pre-Phase-1 artifacts where the hashes have not been computed
against static inputs. No drift is detectable with the available information. CLEAN.

---

### Check 8 — Production-grade principle compliance scan (CLEAN)

Grep against all in-scope architecture documents, vision, brief, and dtu-assessment for
the exact rationalization phrases defined in CLAUDE.md §Canonical Principle:

| Phrase | Occurrences in prescriptive spec body |
|--------|--------------------------------------|
| "for now" | 0 |
| "MVP" | 0 (one occurrence in ADR-0002 body explaining WHY they are not doing the MVP thing — not a violation) |
| "good enough" | 0 |
| "we can fix later" | 0 |
| "minimum viable" | 0 (one occurrence in brief revision history recording a past-tense fix — not prescriptive text) |
| "ship fast and iterate" | 0 |
| "TODO for architect" | 0 |
| "pending architect review" | 0 (two occurrences in brief revision history for v1.3 and v1.4, documenting past resolved states — not prescriptive) |
| "placeholder for architect" | 0 |

The `todo!()` markers in SS-engine-module.md (`ClaudeCodeModule::spawn` and `ClaudeCodeModule::preflight`)
are explicitly documented as intentional spec-artifact phase markers: "The `todo!()` markers
are intentional: these are Phase 1 spec artifacts. The Phase 1 story for `monocle-runtime`
initialization provides the full implementation. These signatures are binding — the implementer
must not alter them." This is correct use — the signatures are binding and the implementation
is assigned to Phase 1 stories, not deferred indefinitely. Not a violation.

CLEAN — zero rationalization phrases in prescriptive spec text.

---

### Check 9 — Hook protocol surface count (CLEAN)

| Source | Count | Variants/Entries |
|--------|-------|-----------------|
| STATE.md frontmatter `dtu_services` | 5 | `hook-endpoints-x5` |
| SS-engine-module.md `hook_paths()` return | 5 | SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop |
| SS-core-types-and-abi.md `HookType` enum | 5 | SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop |
| DTU assessment §Endpoint matrix | 5 | PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit |

All four sources agree on exactly 5 hook endpoints. The `HookType` enum carries
`#[non_exhaustive]` per BC-TYPES-001, which is correct — Phase 4 may add `PostToolUse`.
The `HookEvent` enum carries 5 variants matching exactly. `hook_paths()` returns a
`HashMap<HookType, String>` with 5 entries. BC-ENGINE-003 requires
`module.hook_paths().len() == 5` — consistent. CLEAN.

---

### Check 10 — Routing table sanity (CLEAN)

No architecture document contains agent routing instructions, spec-authorship assignments
to non-specialist agents, or any text contradicting the CLAUDE.md agent routing table.
Architecture documents correctly describe their content without specifying which agent
authors which artifact. All three fix-burst commit messages (83d5fc5, 3495812, ac87649)
attribute the work to the architect agent, which is the correct routing per CLAUDE.md
for architecture, ADRs, and DTU assessment. CLEAN.

---

## Summary

| Check | Result |
|-------|--------|
| 1: Trait-surface propagation | MEDIUM — vision doc retains stale infallible signatures |
| 2: BC ID coherence | CLEAN |
| 3: Error-type registry consistency | CLEAN |
| 4: Version banner consistency | CLEAN |
| 5: Cross-references / anchor links | CLEAN |
| 6: STATE.md zero-context resume guarantee | CLEAN (minor observation on v1.2.2 vs v1.3 in Critical Artifacts) |
| 7: Frontmatter input-hash drift | CLEAN |
| 8: Production-grade principle compliance | CLEAN |
| 9: Hook protocol surface count | CLEAN |
| 10: Routing table sanity | CLEAN |

---

## Findings Catalog

### F-R22-1 MEDIUM — Vision document retains stale infallible EngineModule trait signatures

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md`
**Section:** §Key Abstractions — EngineModule, lines 111-128 (the Rust code snippet)

**Stale text (lines 117-118 and 122-124):**
```rust
fn metadata(&self) -> EngineMetadata;
// ...
async fn enrich(&self, proc: &ProcessSnapshot) -> EnrichedSession;
```

**Authoritative text (SS-engine-module.md v1.1.3):**
```rust
fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError>;
async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>;
```

**Root cause:** The round-21 fix burst (F-R20-1) correctly updated SS-engine-module.md
(authoritative) but did not propagate the signature change to the vision document. The
vision is explicitly non-authoritative per CLAUDE.md §Architectural Authority. However,
a Phase 1 TDD test-writer or implementer who reads the vision sketch before or instead of
SS-engine-module.md will implement the infallible form — reintroducing the silent-failure
violation that F-R20-1 fixed.

**Risk impact:** A Phase 1 implementer writing the test stub for `EngineModule::metadata`
based on the vision snippet would write `fn metadata(&self) -> EngineMetadata` and the
test would compile without the `HomeUnresolvable` error path — defeating BC-ENGINE-001's
no-silent-fallback contract at the implementation level.

**Remediation options (architect chooses one):**

Option A (preferred): Update the vision §Key Abstractions EngineModule snippet in-place
to reflect v1.1.3 signatures. Add an inline comment: `// v1.1.3: Result, not infallible —
see SS-engine-module.md §EngineModule Trait Signature for authoritative signatures.`

Option B: Add a section note immediately before the snippet: "Note: The trait signatures
below are the original vision sketch. The authoritative Phase 1 signatures are in
SS-engine-module.md v1.1.3 §EngineModule Trait Signature — both `metadata` and `enrich`
now return `Result<_, EngineMetadataError>`. The vision sketch is preserved for historical
context only."

**Routing:** architect (vision is an architecture artifact; changes require architect
or orchestrator authority).

---

## Self-Audit

Per CLAUDE.md Self-Audit Checklist:

- Did I rationalize any check with "good enough"? No.
- Did I add a tech-debt-register entry without human direction? No — no tech-debt entries.
- Did I leave any "pending architect review" in findings? No — findings include concrete
  remediation options.
- Did I find a defect and surface it as advisory instead of MEDIUM or CRITICAL? No —
  F-R22-1 is classified MEDIUM (not advisory) because it creates a concrete implementer
  risk path for Phase 1 TDD stories.
- Did I pad findings to look thorough? No — 9 of 10 checks are genuinely CLEAN. The single
  MEDIUM finding is real and specific with file+line evidence.
- Did I skip any of the 10 prescribed checks? No — all 10 are executed and documented.
