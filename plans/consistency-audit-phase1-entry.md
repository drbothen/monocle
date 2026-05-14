---
document_type: consistency-audit
level: ops
version: "1.0"
status: complete
producer: consistency-validator
phase: phase-1-spec-crystallization-entry
timestamp: 2026-05-14T22:00:00Z
input-hash: "[live-state]"
traces_to: "Phase 1 entry — PRD c69518d + VPs b7a5715"
project: monocle
---

# Consistency Audit: Phase 1 Entry Gate

PRD commit c69518d · VPs commit b7a5715 · Audit timestamp 2026-05-14T22:00:00Z

---

## Verdict

**GAPS** — 3 findings; none are critical blockers; 1 is MEDIUM (PG-4 violation), 2 are LOW/observation.
Gate may proceed after product-owner fixes F-001 (PG-4 anchor) and F-002 (BC-LOCK-001 anchor mismatch).
F-003 (BC-DAEMON-* scope gap) is routed to adversary T-3 for adjudication.

---

## Audit Results Table

| # | Check | Status | Evidence |
|---|-------|--------|----------|
| 1 | BC inventory coherence (STATE.md 16 / PRD 16 / VPs 16 IDs match) | PASS | PRD §3 has exactly 16 `### BC-` sections: BC-RING-001, BC-AUTH-001/002, BC-LOCK-001, BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-ENGINE-001/002/002-ERR/003. VPs catalog overview table has 16 `VP-` rows with 1:1 ID matching. STATE.md §Phase 1 Entry Artifact Inventory lists same 16 IDs. All three counts agree. |
| 2 | BC↔VP 1:1 ID coherence | PASS | Coverage matrix (VPs §Coverage Matrix) has exactly 16 rows; every BC-XXX-NNN maps to VP-XXX-NNN with matching domain. VP-RING-001/VP-AUTH-001/VP-AUTH-002/VP-LOCK-001/VP-ABI-001/VP-ABI-002/VP-TYPES-001/VP-FACTORY-001/VP-FACTORY-002/VP-PROTO-001a/VP-PROTO-001b/VP-PROTO-002/VP-ENGINE-001/VP-ENGINE-002/VP-ENGINE-002-ERR/VP-ENGINE-003. Zero orphans in either direction. |
| 3 | Version-pin coherence (PG-5) — primary BC source files | PASS | PRD `traces_to`: SS-daemon-lifecycle.md v1.0.7 ✓, SS-core-types-and-abi.md v1.2.8 ✓, SS-engine-module.md v1.1.15 ✓. VPs `traces_to` and §References items 1-3: same three files at same versions ✓. Actual frontmatter versions confirmed: SS-daemon-lifecycle v1.0.7, SS-core-types-and-abi v1.2.8, SS-engine-module v1.1.15. Zero drift on primary BC source pins. |
| 3b | Version-pin coherence — secondary files (SS-deps, SS-permissions, SS-conventions, dtu-assessment) | OBSERVATION | Neither PRD nor VPs pin versions for SS-deps-pin-manifest (v1.1.8), SS-permissions-phase1 (v1.4), SS-conventions-anti-patterns (v1.28), or dtu-assessment (v1.7). Per PG-5 option (c), secondary inputs do not require version pins when referenced only as authority/convention sources. These files are not BC sources. ACCEPTABLE per PG-5. |
| 4 | §-anchor resolution (PG-4) — spot check of 12 required §-anchors | PARTIAL PASS — see F-001, F-002 | §HookEventRecord resolves via "SS-daemon-lifecycle.md §Drain" (prefix match to "### Drain (10-Second Timeout)") ✓. §Daemon Lifecycle Protocol ✓. §Lock File Discovery Policy ✓ (heading exists but see F-002 for semantic mismatch). §ABI Version Constant ✓. §Enum Extensibility ✓. §Non-Exhaustive Inner Structs ✓ (under §Enum Extensibility). §FactoryAdapter Trait ✓. §Prost Wire Schemas ✓. §EngineModule trait → §EngineModule Trait Signature ✓. §ClaudeCodeModule → §Phase 1 Implementation: ClaudeCodeModule ✓. §EngineModule trait error types → §Behavioral Contracts BC-ENGINE-002-ERR ✓. **FAIL at F-001:** PRD Glossary cites "product-brief.md §Forward-compatibility contracts" — no such heading exists in brief (it is a bold label under ### In Scope, not a heading). |
| 5 | Count coherence — PRD | PASS | 16 BCs ✓ (16 `### BC-` sections). 14 error codes ✓ (E-AUTH-001..003, E-DAEMON-001..003, E-LOCK-001..003, E-ENG-001, E-FACT-001..002, E-RING-001, E-PROTO-001 = 14). 39 edge cases ✓ (EC-001 through EC-039 in §9 catalog). 5 hook endpoints ✓ (session-start, prompt-submit, pre-tool-use, notification, stop per BC-ENGINE-003). 11 NFRs ✓ (NFR-001..011). |
| 6 | Count coherence — VPs | PASS | 16 VPs ✓. Fuzz: 4 VPs with fuzz auxiliary (3 distinct harness files — fuzz_auth_token_validation shared by VP-AUTH-001/002, fuzz_state_md_parser, fuzz_envelope_dispatch). Mutation-test: 3 VPs (VP-RING-001, VP-LOCK-001, VP-TYPES-001). Kani proofs: 0 ✓. Open gaps: 5 (§G-1 through §G-5) ✓. Mechanism distribution table arithmetic: 16+16=32 VPs-touched rows across primary+auxiliary = correct (all 16 primary unit-test + 4 fuzz + 3 mutation = 23 VP slots; correct per table). |
| 7 | §Trace v1.0 blocks in both files | PASS | PRD has `## §Trace v1.0` at line 1046. VPs has `## §Trace` at line 1149. Both contain well-formed PG-3 compliant entries with §-section references, no bare L-numbers, no `above/below` directional qualifiers. D-042 sweep evidence present in both. PG-4 §-heading-existence sweep evidence present in PRD §Trace. |
| 8 | Error taxonomy cross-check (PRD §5 ↔ BC source contracts) | PASS | All 14 error codes trace to identified BCs or SS-* sections: E-AUTH-001/002/003 → BC-AUTH-002; E-DAEMON-001 → BC-DAEMON-003 (cross-ref to SS-daemon-lifecycle.md, not in 16-BC scope — see F-003 observation); E-DAEMON-002/003 → SS-daemon-lifecycle.md §Shutdown Signal Handling / §Health and Status Endpoints; E-LOCK-001/002/003 → BC-LOCK-001 and §Start Sequence; E-ENG-001 → BC-ENGINE-002-ERR; E-FACT-001/002 → BC-FACTORY-002; E-RING-001 → BC-RING-001; E-PROTO-001 → BC-PROTO-002. All source BCs are documentable; error codes are consistent with BC postconditions. |
| 9 | BC-DAEMON-* scope boundary | OBSERVATION | SS-daemon-lifecycle.md defines BC-DAEMON-001..006 which are NOT in the 16-BC scope. VPs §G-4 acknowledges this gap and states the PRD synthesis should have included VP-DAEMON-001..006 (or PRD-registered equivalents). PRD cross-references BC-DAEMON-003 in E-DAEMON-001 error taxonomy without formally covering it. Route to adversary T-3 for adjudication of whether this constitutes a PRD incompleteness requiring a v1.1 revision of VPs to add VP-DAEMON-001..006. |
| 10 | Frozen META catalog not re-litigated | PASS | Search for F-R55-adv-1, F-R55-adv-3, F-R61-adv-1, F-R61-2 IDs in PRD and VPs: zero matches. Search for the violation PATTERNS: no bare L-numbers in §Trace sections; no em-dash separator violations; no intra-document bold-label §-citation attempts as navigation targets in the §Trace blocks. Frozen catalog items not re-litigated and not re-introduced as new findings. |
| 11 | Naming convention compliance | PASS | No `"Monocle"` in code paths or backtick spans. No `Monocle` in identifiers. Prose headings use `Monocle` correctly ("Product Requirements Document: Monocle", "Verification Properties: Phase 1 Behavioral Contract Catalog"). `monocle` used correctly in crate names (`monocle-core`, `monocle-runtime`, `monocle-proto`). `VSDD Factory` display name string correct (not `vsdd factory`). |
| 12 | Forbidden language patterns (CLAUDE.md §CANONICAL PRINCIPLE) | PASS | Grep for "MVP", "for now", "good enough", "we can fix later", "minimum viable", "TODO for architect", "Pending architect review", "Placeholder for architect" in both files: zero hits in normative content. The word "placeholder" appears once in BC-FACTORY-002 postcondition 3 as a forbidden anti-pattern description ("Consumers MUST NOT receive 'unknown' as a placeholder"), which is correct usage. |

---

## Findings Table

| ID | Severity | Category | File | Location | Evidence | Recommended Route |
|----|----------|----------|------|----------|----------|-------------------|
| F-001 | MEDIUM | PG-4 §-anchor violation | `prd.md` | Glossary §FC row, line 1031; §Trace PG-4 sweep claim, line 1052 | PRD Glossary table cites `product-brief.md §Forward-compatibility contracts` as a §-anchor. The brief has no heading "Forward-compatibility contracts" — it is a bold label (`**Forward-compatibility contracts (locked pre-Phase-1...):**`) under `### In Scope`. Per PG-4: "Bold labels, paragraph prefixes, or any non-heading text do NOT satisfy this convention." The PRD §Trace at line 1052 claims `brief §Forward-compatibility contracts ✓` — this self-audit claim is false. Both the Glossary citation and the §Trace pass-claim need correction. Correct form: `product-brief.md §Scope` (the enclosing heading) with a position-free qualifier, e.g. `product-brief.md §Scope (forward-compatibility contracts sub-bullet)`. | product-owner: fix PRD Glossary §FC row citation + update §Trace PG-4 sweep claim. |
| F-002 | MEDIUM | §-anchor semantic mismatch | `prd.md` | BC-LOCK-001 Traceability section, line 288; RTM §7, line 918 | PRD BC-LOCK-001 Traceability (line 288) and RTM (line 918) both cite `SS-daemon-lifecycle.md v1.0.7 §Lock File Discovery Policy`. But the lock file JSON schema contract (`contract_version` first key) lives in `§Daemon Lifecycle Protocol §Start Sequence` — confirmed by (a) BC-LOCK-001 Source field at line 250 citing `§Daemon Lifecycle Protocol §Start Sequence`, (b) SS-daemon-lifecycle.md §Behavioral Contract Summary table line 507 placing BC-LOCK-001 under "Daemon Lifecycle Protocol §Start Sequence", and (c) VPs VP-LOCK-001 correctly tracing to `SS-daemon-lifecycle.md §Start Sequence`. `§Lock File Discovery Policy` is the TUI-client hook-script discovery policy (how to find the lock file by path), not the lock file JSON schema contract. Both headings exist; the PRD Traceability/RTM cite the wrong one. | product-owner: fix PRD lines 288 and 918 to cite `SS-daemon-lifecycle.md v1.0.7 §Daemon Lifecycle Protocol §Start Sequence`. |
| F-003 | OBSERVATION | Scope boundary — BC-DAEMON-* | `verification-properties.md`, `prd.md` | VPs §G-4 lines 1059-1077; PRD §5 E-DAEMON-001 line 882 | SS-daemon-lifecycle.md defines BC-DAEMON-001..006 (healthz, status endpoint, body size limit, graceful shutdown, lock file atomicity, crash recovery). These 6 BCs are outside the 16-BC pre-staged scope but are referenced in: (a) PRD error taxonomy E-DAEMON-001 citing BC-DAEMON-003, (b) PRD edge case EC-002 citing BC-DAEMON-003. VPs §G-4 acknowledges this gap, states the VP catalog "SHOULD be extended to VP-DAEMON-001..006 once the PRD lands." However, the current PRD does NOT formalize BC-DAEMON-* as full contract entries in §3, and VPs has not been extended to v1.1. This creates a dangling reference in E-DAEMON-001 (references an unformalized BC). Route to adversary T-3 for adjudication: is this an acceptable scope-phasing (BC-DAEMON-* to be formalized in a subsequent PRD iteration) or does it constitute a PRD incompleteness requiring immediate resolution? | adversary (T-3): adjudicate whether BC-DAEMON-* scope boundary constitutes a PRD v1.0 completeness gap. If yes, route to product-owner for PRD §3 addendum + VPs v1.1 extension adding VP-DAEMON-001..006. |

---

## Cross-File BC↔VP Matrix

| BC ID | BC Source File (pinned) | VP ID | VP Mechanism | Confirmed |
|-------|------------------------|-------|--------------|-----------|
| BC-RING-001 | SS-daemon-lifecycle.md v1.0.7 | VP-RING-001 | unit-test + mutation-test | ✓ |
| BC-AUTH-001 | SS-daemon-lifecycle.md v1.0.7 | VP-AUTH-001 | unit-test + fuzz | ✓ |
| BC-AUTH-002 | SS-daemon-lifecycle.md v1.0.7 | VP-AUTH-002 | unit-test + fuzz (shared target) | ✓ |
| BC-LOCK-001 | SS-daemon-lifecycle.md v1.0.7 | VP-LOCK-001 | unit-test + mutation-test | ✓ |
| BC-ABI-001 | SS-core-types-and-abi.md v1.2.8 | VP-ABI-001 | unit-test | ✓ |
| BC-ABI-002 | SS-core-types-and-abi.md v1.2.8 | VP-ABI-002 | unit-test | ✓ |
| BC-TYPES-001 | SS-core-types-and-abi.md v1.2.8 | VP-TYPES-001 | unit-test + mutation-test | ✓ |
| BC-FACTORY-001 | SS-core-types-and-abi.md v1.2.8 | VP-FACTORY-001 | unit-test | ✓ |
| BC-FACTORY-002 | SS-core-types-and-abi.md v1.2.8 | VP-FACTORY-002 | unit-test + fuzz | ✓ |
| BC-PROTO-001a | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-001a | unit-test | ✓ |
| BC-PROTO-001b | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-001b | unit-test | ✓ |
| BC-PROTO-002 | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-002 | unit-test + fuzz | ✓ |
| BC-ENGINE-001 | SS-engine-module.md v1.1.15 | VP-ENGINE-001 | unit-test | ✓ |
| BC-ENGINE-002 | SS-engine-module.md v1.1.15 | VP-ENGINE-002 | unit-test | ✓ |
| BC-ENGINE-002-ERR | SS-engine-module.md v1.1.15 | VP-ENGINE-002-ERR | unit-test | ✓ |
| BC-ENGINE-003 | SS-engine-module.md v1.1.15 | VP-ENGINE-003 | unit-test | ✓ |

**Coverage:** 16/16 (100%). Zero BCs without VP. Zero VPs without BC.

---

## Version-Pin Drift Table

| Source File | PRD Cited Version | VPs Cited Version | Actual Frontmatter Version | Drift |
|-------------|-------------------|-------------------|----------------------------|-------|
| SS-daemon-lifecycle.md | v1.0.7 (traces_to) | v1.0.7 (traces_to + §References item 1 + coverage matrix) | v1.0.7 | NONE |
| SS-core-types-and-abi.md | v1.2.8 (traces_to) | v1.2.8 (traces_to + §References item 2 + coverage matrix) | v1.2.8 | NONE |
| SS-engine-module.md | v1.1.15 (traces_to) | v1.1.15 (traces_to + §References item 3 + coverage matrix) | v1.1.15 | NONE |
| product-brief.md | v1.4.23 (traces_to) | not pinned | v1.4.23 | NONE (PRD pins; VPs references via STATE.md authority only) |
| domain-monocle-vision-synthesis.md | v1.1.2 (traces_to) | not pinned | v1.1.2 | NONE (PRD pins; VPs does not cite directly) |
| dtu-assessment.md | not pinned | not pinned | v1.7 | NOT PINNED in either (secondary input; PG-5 option (c) applies) |
| SS-deps-pin-manifest.md | not pinned | not pinned | v1.1.8 | NOT PINNED in either (secondary input; PG-5 option (c) applies) |
| SS-permissions-phase1.md | not pinned | not pinned | v1.4 | NOT PINNED in either (secondary input; PG-5 option (c) applies) |
| SS-conventions-anti-patterns.md | not pinned | not pinned | v1.28 | NOT PINNED in either (secondary input; PG-5 option (c) applies) |

**Summary:** Zero version-pin drift on all cited primary files. Three secondary files not pinned per PG-5 convention — acceptable.

---

## Frozen META Catalog Status

Per STATE.md §Pre-Phase-1 Final Gate, 4 entries are frozen and must not be re-litigated in Phase 1+:

| Frozen ID | Description | Status in PRD | Status in VPs |
|-----------|-------------|---------------|---------------|
| F-R55-adv-1 | PG-4 em-dash separator (§Item P3-1 — Verdict form accepted) | NOT referenced; §Trace uses only §-anchor references, no em-dash separator violations detected | NOT referenced; §Trace uses only §-anchor references |
| F-R55-adv-3 | PG-4 intra-document scope (bold-paragraph-label citations accepted for intra-doc) | NOT referenced; the F-001 finding is a CROSS-document brief citation, not an intra-document one — correctly classified as a new finding, not a frozen residual | NOT referenced |
| F-R61-adv-1 | PG-3-CLASSIFICATION-EVIDENCE (bare L-numbers in §Trace post-fix shorthand) | NOT referenced; no bare L-numbers found in §Trace | NOT referenced; no bare L-numbers in §Trace |
| F-R61-2 | §Trace-Heading-Convention scope doesn't document ADR/vision/brief equivalents | NOT referenced | NOT referenced |

**Conclusion:** Frozen catalog confirmed not re-litigated in PRD or VPs. F-001 finding is a new PG-4 instance on a cross-document brief citation — it is NOT covered by the frozen F-R55-adv-3 (which exempts intra-document bold-label citations, not cross-document heading navigation citations in a different file).

---

## §Trace v1.0

v1.0 (2026-05-14T22:00:00Z): Fresh-context Phase 1 entry consistency audit by consistency-validator. Inputs: PRD commit c69518d, VPs commit b7a5715, product-brief.md v1.4.23, domain-monocle-vision-synthesis.md v1.1.2, SS-daemon-lifecycle.md v1.0.7, SS-core-types-and-abi.md v1.2.8, SS-engine-module.md v1.1.15, SS-deps-pin-manifest.md v1.1.8, SS-conventions-anti-patterns.md v1.28, SS-permissions-phase1.md v1.4, dtu-assessment.md v1.7, STATE.md, ADRs 0001-0004. Verdict: GAPS. Findings: F-001 MEDIUM (PRD Glossary PG-4 brief §Forward-compatibility contracts mis-anchor — no such heading in brief), F-002 MEDIUM (PRD BC-LOCK-001 Traceability/RTM cite §Lock File Discovery Policy when contract lives at §Start Sequence), F-003 OBSERVATION (BC-DAEMON-001..006 scope boundary — PRD error taxonomy references BC-DAEMON-003 without formal contract coverage). All 12 required §-anchors spot-checked; 10 resolve correctly; 2 identified as F-001 and F-002. BC↔VP 1:1 coverage 16/16. Version-pin drift: zero on primary files. Frozen META catalog: confirmed not re-litigated. Forbidden language: zero hits. Naming convention: PASS. Production-grade compliance: PASS.
