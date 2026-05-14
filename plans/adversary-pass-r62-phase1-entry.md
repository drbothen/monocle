---
document_type: adversary-report
version: "1.0"
status: complete
producer: adversary
phase: phase-1-spec-crystallization
input-hash: "[live-state]"
traces_to: "Phase 1 PRD c69518d + VPs b7a5715; STATE.md phase-1-spec-crystallization-entry-pending; D-047 strict; 18+ META defense layers"
project: monocle
level: ops
timestamp: 2026-05-14T22:30:00Z
pass_number: 1
policy: D-047-strict
---

# Adversarial Review Pass R62 — Phase 1 Entry (D-047 Strict, Pass 1 of 3)

## Summary

**Verdict: FINDINGS — D-047 strict pass 1 FAIL.**

10 findings total: **1 CRITICAL**, **5 HIGH**, **2 MED**, **2 LOW**. Per D-047 strict, ANY finding fails the pass. The 3-clean-pass counter resets to 0. Recommended route: fix burst F-R62, then re-dispatch adversary for pass R63.

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 5 |
| MED | 2 |
| LOW | 2 |
| **TOTAL** | **10** |

The most severe finding is the PRD scope gap on BC-DAEMON-001..006 — disposition (b) below. Several PG-4 §-heading-existence violations (one of which is a falsified §Trace self-check) appear across the PRD's Section 7, Glossary, and §Trace block. PRD ↔ VP test-harness file disagreement on 3 BCs creates an unrecoverable implementer ambiguity. The VP file's frontmatter has the wrong `phase` and a contradictory `status: complete` vs §G-4's "v1.1 revision required" admission.

## BC-DAEMON-001..006 Scope Adjudication (BLOCKING)

**Verdict: Disposition (b) — PRD scope gap.** Severity: **CRITICAL**.

**Reasoning:**

1. **The architecture's authorial intent is unambiguous.** `SS-daemon-lifecycle.md` v1.0.7 §Behavioral Contract Summary (lines 494-511) lists 10 BCs in its summary table: BC-DAEMON-001..006 + BC-RING-001 + BC-AUTH-001/002 + BC-LOCK-001. Line 509-511 reads: *"The Phase 1 PRD will formalize these as full BC entries with postconditions, evidence, and verification harness stubs. This artifact pre-stages them for the Phase 1 architecture gate."* The antecedent of "these" is the full table (10 BCs), not a subset. The architect's prescriptive scope for the Phase 1 PRD is 10 BCs from this file alone.

2. **The PRD's own error taxonomy depends on undefined BCs.** PRD §Section 5 row E-DAEMON-001 (line 882) cites the source BC as "BC-DAEMON-003 (SS-daemon-lifecycle.md)". PRD EC-002 (line 126) reads: "Very large `tool_input` values (up to 256 KiB per BC-DAEMON-003)." But BC-DAEMON-003 is NOT formalized as a contract section in the PRD. The PRD is making forward references to BCs it does not define. Same applies to E-DAEMON-002 (SS-daemon-lifecycle.md §Shutdown Signal Handling, implicit BC-DAEMON-004) and E-DAEMON-003 (BC-DAEMON-001 implicit via §Health and Status Endpoints).

3. **The formal-verifier correctly surfaced this** via VP §G-4: *"BC-DAEMON-001..006 are pre-staged in SS-daemon-lifecycle.md but are NOT in the 16-BC scope of this VP catalog... The Phase 1 PRD will formalize them with the same per-BC verification-harness pattern used in this artifact."* This is a model use of Correct Agent Routing — surface the issue, route to product-owner via orchestrator.

4. **The product-owner did NOT surface or fix the gap.** STATE.md's T-1 dispatch prompt literally enumerated 16 BCs. The product-owner followed the dispatch literally without surfacing the architecture-vs-dispatch discrepancy, then silently emitted a PRD with forward references to 3 undefined BCs (BC-DAEMON-001, BC-DAEMON-003, BC-DAEMON-004). This is the exact anti-pattern that the Correct Agent Routing companion principle blocks: the product-owner should NOT silently ship a PRD with forward references to undefined BCs — it should surface the dispatch-prompt-vs-source-of-truth conflict to the orchestrator.

5. **Per CLAUDE.md §CANONICAL PRINCIPLE rule 4 + Correct Agent Routing rule 3:** the right path is "Surface (production-grade): Agent A finds an issue → routes to orchestrator with 'this needs specialist B' → orchestrator dispatches specialist B → specialist B fixes in scope → original work proceeds." The fix is: product-owner adds 6 more BC sections to the PRD (BC-DAEMON-001..006), updates the count claim from 16 → 22, expands the error taxonomy traceability rows, expands the edge-case catalog. The VP file's §G-4 is then re-conciled.

**Routing for the fix:** orchestrator → product-owner (PRD v1.1 expansion to 22 BCs) → concurrent dispatch to formal-verifier (VP v1.1 expansion to 22 VPs with VP-DAEMON-001..006). Adversary re-dispatch for R63 after fixes.

## Findings Table

| ID | Severity | Domain | File:Line/Heading | Description | Recommended Route |
|----|----------|--------|-------------------|-------------|------------------|
| F-R62-1 | CRITICAL | PRD scope | `.factory/specs/prd.md` §3 (entire), §5 line 882, §9 EC-002 | BC-DAEMON-001..006 missing from PRD; PRD's error taxonomy + edge cases forward-reference undefined BC-DAEMON-003 / BC-DAEMON-004 / BC-DAEMON-001. Architecture (SS-daemon-lifecycle.md line 509-511) prescribes 10 BCs for PRD, not 16. | product-owner (PRD v1.1 + 6 BCs) → formal-verifier (VP v1.1 + 6 VPs) |
| F-R62-2 | HIGH | PG-4 anchor | `.factory/specs/prd.md` §6 Competitive Diff, §7 Traceability Matrix, §10 Glossary, §Trace | `brief §Forward-compatibility contracts` cited 7+ times; the named string is a bullet bold-label in `## Scope` at brief line 173, NOT a heading. PG-4 violation multi-site. Also `§Scope §ClaudeCodeModule` (bullet in `### In Scope`, not heading), `§Success Criteria §Factory pattern detection` (row label in table, not heading). | product-owner (PRD fix-burst — re-anchor to actual headings per PG-4 anti-pattern table) |
| F-R62-3 | HIGH | PG-4 META | `.factory/specs/prd.md` §Trace | Falsified PG-4 §-heading-existence self-check. §Trace declares `brief §Forward-compatibility contracts ✓` (and 2 other sub-anchors) but those anchors do not resolve to actual headings. F-R51-adv-1 pattern — falsified self-checks shipped as PASS. NEW pattern; not in D-054 frozen residual catalog. | product-owner (PRD §Trace correction + actual sweep) |
| F-R62-4 | HIGH | PRD ↔ VP drift | `.factory/specs/prd.md` lines 187, 236, 284, 326 vs `.factory/specs/verification-properties.md` lines 237, 287, 339, 375 | PRD and VP disagree on test-harness file paths for 4 BCs: BC-AUTH-001 (PRD `auth.rs` vs VP `auth_token_lifecycle.rs`); BC-AUTH-002 (PRD `auth.rs` vs VP `auth_header_rejection.rs`); BC-LOCK-001 (PRD `daemon_lock.rs` vs VP `lock_file_contract.rs`); BC-ABI-001 (PRD vague vs VP `status_endpoint.rs`). Implementer cannot resolve canonical file. | product-owner + formal-verifier (joint reconciliation: pick one canonical set; both files updated atomically) |
| F-R62-5 | HIGH | VP frontmatter | `.factory/specs/verification-properties.md` line 6, line 8 | VP frontmatter has `phase: pre-phase-1-architecture` — wrong. STATE.md asserts Phase 1 entry; T-2 dispatch was Phase 1. PRD correctly says `phase: phase-1-spec-crystallization`. Also `status: complete` contradicted by §G-4 "VP catalog SHOULD be extended in a v1.1 revision". | formal-verifier (VP frontmatter fix: phase → `phase-1-spec-crystallization`; status → `draft` until §G-4 closed) |
| F-R62-6 | HIGH | PRD trait drift | `.factory/specs/prd.md` vs `.factory/specs/verification-properties.md` | PRD claims test type "Clippy" for BC-TYPES-001 and "Compile/rustdoc" for BC-ENGINE-001. VP specifies concrete test files using `syn 2` AST parsing. PRD has materially lower verification rigor than VP. Implementer cannot reconcile. | product-owner + formal-verifier (joint reconciliation) |
| F-R62-7 | MED | VP fabrication | `.factory/specs/verification-properties.md` lines 716-749 | VP-PROTO-002 mandates `monocle-proto` export `pub fn dispatch_envelope(env: &HookEnvelope) -> Result<(), DispatchError>` as a Phase 1 stub. NOT specified in `SS-core-types-and-abi.md` or any other architecture artifact. PRD BC-PROTO-002 explicitly says "Phase 4 integration test (out of Phase 1 scope)". VP fabricates Phase 1 code surface. | architect (decide whether to add to SS-core-types-and-abi.md as Phase 1 stub OR formal-verifier reframes VP-PROTO-002 as Phase 4-only) |
| F-R62-8 | MED | PRD invention | `.factory/specs/prd.md` §5 lines 880-881 (E-AUTH-002, E-AUTH-003) | PRD invents error bodies `{"error":"missing_auth_token"}` and `{"error":"invalid_auth_token"}` not specified in SS-daemon-lifecycle.md. Architecture only defines `invalid_auth_token_format`. PRD adds contract surface beyond architecture without architect routing. | product-owner (route to architect for SS-daemon-lifecycle.md update OR remove inventions from PRD) |
| F-R62-9 | LOW | VP stale projection | `.factory/specs/verification-properties.md` §G-4 lines 1061-1077 | VP §G-4 says "SCOPED — covered by Phase 1 PRD verification-harness stubs". Now FALSE: the PRD does NOT include BC-DAEMON-001..006. Stale-projection error. | formal-verifier (VP §G-4 update once PRD v1.1 lands) |
| F-R62-10 | LOW | PRD ↔ Architecture | `.factory/specs/prd.md` BC-PROTO-001a postcondition 4 (line 540) | PRD describes proto oneof field-number assignment as "SessionStart=10, UserPromptSubmit=11..." but architecture uses field names `session_start = 10`, `prompt_submit = 11`. PRD conflates field name with event type name. Cosmetic but inaccurate. | product-owner (PRD prose disambiguation) |

## Per-Finding Detail

(Full per-finding evidence and rationale preserved from adversary output; see adversary's response in resume transcript for verbatim. Critical sections summarized:)

### F-R62-1 [CRITICAL]
PRD omits BC-DAEMON-001..006 despite architecture's prescriptive language in SS-daemon-lifecycle.md line 509-511. Error taxonomy and edge cases forward-reference 3 undefined BCs. Architecture authority: 10 BCs prescribed for daemon-lifecycle file alone (not 4). Correct total Phase 1 PRD scope = 22 BCs.

### F-R62-2 / F-R62-3 [HIGH]
PG-4 violations across §6, §7, §10, §Trace. Brief lacks a heading named "Forward-compatibility contracts" (it is a bullet label in `## Scope`). PRD's §Trace falsely asserts PG-4 sweep PASS. New pattern, not in D-054 frozen catalog.

### F-R62-4 / F-R62-5 / F-R62-6 [HIGH]
Cross-artifact coordination defects between concurrent T-1 and T-2: test file paths drift on 4 BCs; VP frontmatter `phase` and `status` wrong; PRD verification rigor weaker than VP for BC-TYPES-001 and BC-ENGINE-001.

### F-R62-7 / F-R62-8 [MED]
VP fabricates Phase 1 code surface (`dispatch_envelope`) without architecture authorization; PRD invents 2 error bodies (`missing_auth_token`, `invalid_auth_token`) without architecture authorization.

### F-R62-9 / F-R62-10 [LOW]
VP §G-4 stale projection (will be reconciled after F-R62-1 fix); PRD conflates proto field name with event type name (cosmetic).

## Frozen META Residual Catalog Status

| ID | Description | Status this pass |
|----|-------------|------------------|
| F-R55-adv-1 | PG-4 em-dash separator codification gap | Not re-litigated |
| F-R55-adv-3 | PG-4 intra-document scope hole | Not re-litigated |
| F-R61-adv-1 | PG-3-CLASSIFICATION-EVIDENCE bare L-numbers in §Trace shorthand | Not re-litigated |
| F-R61-2 | §Trace-Heading-Convention ADR/vision/brief equivalents | Not re-litigated |

F-R62-2 and F-R62-3 are NEW PG-4 patterns (cross-document brief-bullet-as-§ violations, multi-site falsified self-check). They are NOT instances of F-R55-adv-1 (em-dash form) or F-R55-adv-3 (intra-document scope hole) — they are valid new findings.

## Novelty Assessment

**Novelty: HIGH.** Pass 1 of Phase 1 with newly authored artifacts. All 10 findings are genuinely novel. Most novel categories:

- **CRITICAL F-R62-1** is a brand-new META-pattern at the orchestrator dispatch-prompt authoring level (under-enumerated PRD scope vs architecture's prescriptive language).
- **HIGH F-R62-3** is a new falsified-self-check on a new PG-4 anchor pattern (cross-document, brief-bullet-as-§). The R51 / F-R55 / F-R61 META rules did not anticipate this exact failure mode.
- **HIGH F-R62-4, F-R62-5, F-R62-6** are novel cross-artifact coordination findings — demonstrate that concurrent-dispatch of product-owner and formal-verifier without an explicit interface contract (canonical test file paths, verification rigor) produces drift.

## Pass 1 Verdict and Pass 2 Readiness

**Pass 1 Verdict: FAIL.** 10 findings; D-047 strict requires 0 for 3 consecutive passes; counter resets to 0.

**Pass 2 readiness:** Required fix-burst F-R62 before pass 2:
1. **product-owner** (PRD v1.1): expand to 22 BCs (add BC-DAEMON-001..006); fix PG-4 §-anchor citations (F-R62-2); fix §Trace falsified self-check (F-R62-3); reconcile test-harness file paths with VP (F-R62-4); reconcile verification rigor (F-R62-6); reconcile error taxonomy with architecture (F-R62-8); fix proto field-name conflation (F-R62-10).
2. **formal-verifier** (VP v1.1): expand to 22 VPs (add VP-DAEMON-001..006); fix frontmatter (F-R62-5); reconcile test-harness file paths (F-R62-4); reframe VP-PROTO-002 to not require unauthorized Phase 1 code surface (F-R62-7); update §G-4 (F-R62-9).
3. **architect** (conditional): if F-R62-8 disposition is "add missing/invalid_auth_token to architecture": update SS-daemon-lifecycle.md.

After fix burst, re-dispatch **adversary R63** (fresh-context) and **consistency-validator** in parallel.

---

**Note on orchestrator-level META-finding:** F-R62-1 root cause is the dispatch prompt I (orchestrator) authored from STATE.md's "16 BCs" claim. STATE.md was wrong; my dispatch propagated the error. STATE.md should be updated to "22 BCs" as part of the fix-burst.
