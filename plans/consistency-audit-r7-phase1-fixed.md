---
document_type: consistency-audit
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.5 d321935 + VP v1.5 6831e23 + arch SS-daemon-lifecycle v1.0.11 af2101d + STATE.md v5.5 8acabb1; F-R67 closure chain applied; L-F-R63 Extension 2 intra-block sweep discipline applied"
level: ops
producer: consistency-validator
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: "2026-05-14T10:00:00Z"
round: 7
---

# Consistency Audit — Round 7: Post-F-R67 Phase 1 Spec Package

**Artifacts under audit:**
- `.factory/specs/prd.md` v1.5 (commit d321935)
- `.factory/specs/verification-properties.md` v1.5 (commit 6831e23)
- `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.11 (commit af2101d)
- `.factory/STATE.md` v5.5 (commit 8acabb1)

**Verdict: GAPS**

**Gap count: 1**

**Severity: LOW** — single stale normative-current version pin; zero content defects; zero semantic errors.

---

## Audit Results Summary

| Check | Result | Notes |
|-------|--------|-------|
| 1. 22-BC inventory coherence | PASS | PRD §2.1, §3, §7 RTM all count 22 BCs; VP catalog counts 22 VPs; 1:1 match confirmed |
| 2. BC↔VP 1:1 ID + name + path coherence | PASS | 22/22 VP IDs match BC IDs; 22/22 test-file paths match PRD §7 RTM verbatim |
| 3. Test-file path coherence (22/22) | PASS | VP Coverage Matrix test file paths match PRD RTM test file paths exactly for all 22 BCs |
| 4. Test-name coherence (22/22) | PASS | All 22 test names match across PRD Verification subsections and VP **Test name:** lines — including the 4 F-R63-adv-1 adjudications and the 5 R4-001 sites (see Finding R7-001 for one stale citation) |
| 5. Version-pin coherence (PG-5) | GAPS | VP-DAEMON-001 **Test name** line cites "PRD v1.4" — should be "PRD v1.5" (Finding R7-001) |
| 6. §-anchor resolution (PG-4) | PASS | All §-anchor references in VP, PRD, and arch verified against actual headings in cited files; no mis-anchors |
| 7. Count coherence (PG-2) | PASS | 22 BCs, 22 VPs, 22 test names, 13 error codes, 56 edge cases (EC-001..EC-056) — all consistent across all documents |
| 8. Trace chain integrity | PASS | All 22 VPs have `Traces to:` pointing to correct BC; all BCs trace to architecture source with correct v1.0.11 pin |
| 9. Error taxonomy cross-check | PASS | 13 error codes in PRD §5 (E-AUTH-001/002, E-DAEMON-001/002/003, E-LOCK-001/002/003, E-ENG-001, E-FACT-001/002, E-RING-001, E-PROTO-001); no retired codes; two-body auth taxonomy (`missing_auth_token` / `invalid_auth_token`) consistent across arch §BC-AUTH-002, PRD BC-AUTH-002, VP-AUTH-002 |
| 10. Architecture back-propagation closure | PASS | SS-daemon-lifecycle.md v1.0.11 is current; BC-AUTH-002 Two-body count and Bearer disposition correct; BC Summary footer is version-stable per Pattern B (R3-001 closure); no stale arch pins in PRD or VP |
| 11. Scope-boundary coverage | PASS | All 22 Phase 1 BCs covered; no out-of-scope BCs added; Phase 4-deferred items (VP-PROTO-002 runtime) correctly classified |
| 12. Frozen META catalog status | PASS | F-R55-adv-1, F-R55-adv-3, F-R61-adv-1, F-R61-2 remain frozen; not extended in F-R67 burst; not reintroduced in VP v1.5 |
| 13. Naming convention | PASS | All test names follow `test_BC_<ID>_<description>` convention; no duplicates |
| 14. Forbidden patterns | PASS | No `invalid_auth_token_format` anywhere in normative content; no bare L-numbers in new §Trace entries; no directional qualifiers |
| 15. VP frontmatter | PASS | VP v1.5 frontmatter: document_type, level, section, version, status, producer, phase, timestamp, input-hash, traces_to, project — all present and correct |
| 16. STATE.md coherence | PASS | STATE.md v5.5 correctly reflects PRD v1.5 (d321935), VP v1.5 (6831e23), arch v1.0.11 (af2101d); F-R67 burst documented correctly; next actions (T-16 R68, T-17 cons R7) match actual pipeline state |
| 17. F-R67 closure verification | PASS — see detail | Both F-R67-1 and F-R67-2 confirmed closed (see §F-R67 Closure Verification Table) |
| 18. L-F-R63 Extension 2 intra-block sweep | PASS — 21/22 VPs clean; 1 fixed | VP-TYPES-001 §Mechanism vs §Post-conditions now consistent post-fix; all 22 VPs documented in intra-block sweep table in VP §Trace v1.5 |

---

## Findings Table

| ID | Severity | Artifact | Location | Description | Remediation |
|----|----------|----------|----------|-------------|-------------|
| R7-001 | LOW | `verification-properties.md` v1.5 | Line 249: VP-DAEMON-001 **Test name:** annotation | "per PRD v1.4 §BC-DAEMON-001" — the version pin is stale. PRD is now at v1.5 (commit d321935). The §Trace v1.5 F-R60-corpus-sweep claims all normative-current PRD v1.4 pins were updated, but this single site was missed. The §Trace v1.4 documents this was the R4-001 fix target (changed from v1.2 → v1.4); the v1.4 → v1.5 propagation left this site at v1.4. Content is functionally identical (PRD v1.4 → v1.5 is a single EC-045 off-by-one fix in §BC-DAEMON-003 prose; BC-DAEMON-001 content unchanged). Zero semantic impact; citation accuracy only. | Route to `vsdd-factory:formal-verifier`: change "per PRD v1.4 §BC-DAEMON-001, Verification subsection" to "per PRD v1.5 §BC-DAEMON-001, Verification subsection" at VP-DAEMON-001 Test name line. Single-character version bump. No content change needed. |

---

## Cross-File BC↔VP Matrix

All 22 BC↔VP pairs verified clean:

| BC ID | VP ID | Test Name Match | Test File Match | PRD-VP Mechanism Match |
|-------|-------|-----------------|-----------------|------------------------|
| BC-DAEMON-001 | VP-DAEMON-001 | PASS | PASS | PASS (unit-test) |
| BC-DAEMON-002 | VP-DAEMON-002 | PASS | PASS | PASS (unit-test) |
| BC-DAEMON-003 | VP-DAEMON-003 | PASS | PASS | PASS (unit-test+fuzz) |
| BC-DAEMON-004 | VP-DAEMON-004 | PASS | PASS | PASS (unit-test) |
| BC-DAEMON-005 | VP-DAEMON-005 | PASS | PASS | PASS (unit-test+mutation) |
| BC-DAEMON-006 | VP-DAEMON-006 | PASS | PASS | PASS (unit-test) |
| BC-RING-001 | VP-RING-001 | PASS | PASS | PASS (unit-test+mutation) |
| BC-AUTH-001 | VP-AUTH-001 | PASS | PASS | PASS (unit-test+fuzz) |
| BC-AUTH-002 | VP-AUTH-002 | PASS | PASS | PASS (unit-test+fuzz) |
| BC-LOCK-001 | VP-LOCK-001 | PASS | PASS | PASS (unit-test+mutation) |
| BC-ABI-001 | VP-ABI-001 | PASS | PASS | PASS (unit-test) |
| BC-ABI-002 | VP-ABI-002 | PASS | PASS | PASS (unit-test) |
| BC-TYPES-001 | VP-TYPES-001 | PASS | PASS | PASS (syn 2 AST audit + mutation + clippy supplementary) |
| BC-FACTORY-001 | VP-FACTORY-001 | PASS | PASS | PASS (unit-test/syn 2) |
| BC-FACTORY-002 | VP-FACTORY-002 | PASS | PASS | PASS (unit-test+fuzz) |
| BC-PROTO-001a | VP-PROTO-001a | PASS | PASS | PASS (unit-test) |
| BC-PROTO-001b | VP-PROTO-001b | PASS | PASS | PASS (unit-test) |
| BC-PROTO-002 | VP-PROTO-002 | PASS (Phase 4 deferred) | PASS (Phase 4 deferred) | PASS (Phase 4 deferred) |
| BC-ENGINE-001 | VP-ENGINE-001 | PASS | PASS | PASS (unit-test/syn 2) |
| BC-ENGINE-002 | VP-ENGINE-002 | PASS | PASS | PASS (unit-test) |
| BC-ENGINE-002-ERR | VP-ENGINE-002-ERR | PASS | PASS | PASS (unit-test/temp-env) |
| BC-ENGINE-003 | VP-ENGINE-003 | PASS | PASS | PASS (unit-test) |

**Note on VP-TYPES-001:** The §Mechanism prose in VP v1.5 now correctly states "unit-test (primary, via a `syn 2` AST audit at `monocle-core/tests/enum_audit.rs` per PRD v1.5 §BC-TYPES-001 invariant 1); mutation-test (auxiliary); clippy `non_exhaustive_omitted_patterns` lint configuration (supplementary)." This aligns with PRD §BC-TYPES-001 invariant 1 ("The verification mechanism is a `syn 2` AST parse (NOT clippy)") and VP-TYPES-001 §Post-conditions item 1. F-R67-1 is confirmed closed.

---

## Version-Pin Drift Table

| Artifact | Normative SS-daemon-lifecycle.md pin | Normative PRD pin | Result |
|----------|--------------------------------------|-------------------|--------|
| PRD v1.5 | v1.0.11 (at 31 sites in §3, §7 RTM) | self | PASS |
| VP v1.5 | v1.0.11 (at §Scope, §VP Catalog Overview, 10 Traces to: lines, Coverage Matrix) | v1.5 (at ~40 sites) | GAPS — see R7-001 |
| SS-daemon-lifecycle.md v1.0.11 | self | cites PRD as version-stable file pointer (Pattern B, D-057) | PASS |
| STATE.md v5.5 | af2101d (commit) | d321935 (commit) | PASS |

**Specific stale pin:** VP v1.5 line 249, VP-DAEMON-001 Test name: "per PRD v1.4" (should be "per PRD v1.5"). All other ~39 PRD v1.5 normative-current citations in VP v1.5 are correct.

**All SS-daemon-lifecycle.md version pins:** Verified at v1.0.11 in both PRD and VP normative content. No stale v1.0.10 or earlier pins found outside §Trace historical records.

**SS-core-types-and-abi.md pin:** v1.2.8 — unchanged throughout the F-R67 cycle; confirmed current in both PRD and VP.

**SS-engine-module.md pin:** v1.1.15 — unchanged throughout the F-R67 cycle; confirmed current in both PRD and VP.

---

## F-R67 Closure Verification Table

| Finding | Where to Verify | Verification Result |
|---------|----------------|---------------------|
| F-R67-1: VP-TYPES-001 §Mechanism prose (clippy→syn 2 primary) | VP v1.5 line 1080: `**Mechanism:** unit-test (primary, via a \`syn 2\` AST audit at \`monocle-core/tests/enum_audit.rs\` per PRD v1.5 §BC-TYPES-001 invariant 1); mutation-test (auxiliary); clippy \`non_exhaustive_omitted_patterns\` lint configuration (supplementary).` | CLOSED — syn 2 AST audit is primary; clippy is supplementary. Matches PRD v1.5 §BC-TYPES-001 invariant 1 verbatim. §Post-conditions item 1 also says syn 2 AST audit — intra-block consistent. |
| F-R67-2: PRD EC-045 off-by-one (262,144→262,145) | PRD v1.5 line 228: "EC-045: Request body is exactly 262,145 bytes: HTTP 413 (limit is strictly exclusive — `> limit` triggers the rejection; axum's `DefaultBodyLimit::max(N)` rejects bodies strictly exceeding N bytes; body of exactly N=262,144 returns HTTP 200)." | CLOSED — 262,145 is correct. §9 catalog row (line 1342) says "Body exactly 262,145 bytes → HTTP 413" — CONSISTENT. VP-DAEMON-003 §Mechanical property 1 says "262,145 bytes" — CONSISTENT. All three sites agree. |
| Obs-1: intra-block consistency sweep discipline applied | VP §Trace v1.5 contains 22-row intra-block sweep table (§Mechanism prose vs §Post-conditions) | CLOSED — sweep table present with real results; 1 contradiction (VP-TYPES-001) found and fixed; 21 clean. Discipline applied as preview before state-manager codification. |

---

## Intra-Block Sweep Results (22 VPs)

Results from VP §Trace v1.5 intra-block sweep table — validated against live VP body:

| VP ID | §Mechanism prose claim | §Post-conditions primary surface | Consistent? | Auditor Verification |
|-------|------------------------|----------------------------------|-------------|----------------------|
| VP-DAEMON-001 | unit-test | unit-test (HTTP probes + router-construction inspect) | YES | Confirmed — §Mechanism says unit-test; §Post-conditions 1-6 are unit-test assertions |
| VP-DAEMON-002 | unit-test | unit-test (auth + JSON-schema + 10-field probes) | YES | Confirmed |
| VP-DAEMON-003 | unit-test (primary); fuzz (auxiliary) | unit-test (boundary probes + source-grep) + fuzz harness | YES | Confirmed — boundary assertions at 262,143/262,144/262,145 present in §Post-conditions |
| VP-DAEMON-004 | unit-test | unit-test (synthetic-signal + drain-bound probes) | YES | Confirmed |
| VP-DAEMON-005 | unit-test (primary); mutation-test (auxiliary) | unit-test (mode + pid-liveness probes + source-grep) + mutation rationale | YES | Confirmed |
| VP-DAEMON-006 | unit-test | unit-test (recovery-file lifecycle + TUI offer probes) | YES | Confirmed |
| VP-RING-001 | unit-test (primary); mutation-test (auxiliary) | unit-test (literal-prefix + round-trip) + mutation rationale | YES | Confirmed |
| VP-AUTH-001 | unit-test (primary); fuzz (auxiliary) | unit-test (regex + auth-round-trip) + fuzz harness | YES | Confirmed |
| VP-AUTH-002 | unit-test (primary); fuzz (auxiliary) | unit-test (6-probe table) + fuzz harness | YES | Confirmed — 7-probe post-condition table (probe 7 is positive control) present |
| VP-LOCK-001 | unit-test (primary); mutation-test (auxiliary) | unit-test (literal-prefix + version-gate) + mutation rationale | YES | Confirmed |
| VP-ABI-001 | unit-test | unit-test (HTTP + JSON + compile-time const assert) | YES | Confirmed |
| VP-ABI-002 | unit-test (specifically a compile-time test) | unit-test (compile-time + runtime + type-pin) | YES | Confirmed |
| VP-TYPES-001 | unit-test (primary, via `syn 2` AST audit per PRD v1.5 §BC-TYPES-001 invariant 1); mutation-test (auxiliary); clippy (supplementary) | unit-test (`syn 2` AST walk) + clippy supplement + mutation rationale | YES (fixed in v1.5) | CONFIRMED FIXED — live file at line 1080 reads `syn 2 AST audit` primary, matching §Post-conditions item 1 |
| VP-FACTORY-001 | unit-test (`cargo check` + `syn 2` parse) | unit-test (cargo check + syn parse + field-name check) | YES | Confirmed |
| VP-FACTORY-002 | unit-test (primary); fuzz (auxiliary) | unit-test (constructor + detect + fixture probes) + fuzz harness | YES | Confirmed |
| VP-PROTO-001a | unit-test | unit-test (wire-tag decode + descriptor inspect) | YES | Confirmed |
| VP-PROTO-001b | unit-test | unit-test (struct field + round-trip) | YES | Confirmed |
| VP-PROTO-002 | Phase 1: unit-test (structural recap); Phase 4 (deferred): unit-test + fuzz | Phase 1: cross-property recap; Phase 4: documented future | YES | Confirmed — Phase 4-deferred carve-out preserved; Phase 1 structural recap consistent |
| VP-ENGINE-001 | unit-test (via `syn 2` parse of engine.rs) | unit-test (syn parse of trait + supporting types) | YES | Confirmed |
| VP-ENGINE-002 | unit-test | unit-test (6-probe table) | YES | Confirmed — 6-probe post-condition table present |
| VP-ENGINE-002-ERR | unit-test (with `temp-env ^0.3` env-isolation) | unit-test (sync + async halves with temp-env) | YES | Confirmed |
| VP-ENGINE-003 | unit-test | unit-test (5-entry hook-paths probe + exhaustive HookType match) | YES | Confirmed |

**Sweep result:** 0 intra-block contradictions detected in live v1.5 VP body. VP-TYPES-001 fix confirmed applied at line 1080. All 22 VPs pass this round's intra-block check.

---

## Frozen META Catalog Status

Per D-054 (human-ratified pre-Phase-1 gate PASS, 2026-05-14). Four entries frozen; audit confirms not extended in F-R67 cycle:

| ID | Description | Status |
|----|-------------|--------|
| F-R55-adv-1 | PG-4 em-dash separator codification gap | FROZEN — not reintroduced in VP v1.5 or PRD v1.5 |
| F-R55-adv-3 | PG-4 intra-document scope hole | FROZEN — not reintroduced |
| F-R61-adv-1 | PG-3-CLASSIFICATION-EVIDENCE META rule's own §Trace shorthand exception | FROZEN — not reintroduced |
| F-R61-2 | §Trace-Heading-Convention scope clause ADR/vision/brief equivalents | FROZEN — not reintroduced |

No new META-class instances found in this round.

---

## EC-045/EC-046 Intra-Document Consistency Verification

Per check 17 (F-R67 closure verification) and check 18 (intra-block sweep):

| EC ID | §3 prose value | §9 catalog value | Canonical Test Vectors table | VP-DAEMON-003 §Post-conditions | Status |
|-------|---------------|-----------------|------------------------------|-------------------------------|--------|
| EC-045 | 262,145 bytes: HTTP 413 (line 228) | 262,145 bytes → HTTP 413 (line 1342) | 262,145 bytes → HTTP 413 (line 238) | Item 1: POST 262,145-byte body → HTTP 413 | CONSISTENT — all 4 sites agree |
| EC-046 | 262,143 bytes: HTTP 200 (line 230) | Body 262,143 bytes → HTTP 200 (line 1343) | 262,143 bytes → HTTP 200 (line 239) | Item 3: POST 262,143-byte body → HTTP 200 | CONSISTENT — all 4 sites agree |

**Boundary semantics verification:** axum `DefaultBodyLimit::max(262144)` rejects bodies strictly exceeding 262,144 bytes. Bodies of exactly 262,144 pass (HTTP 200). Bodies of 262,145+ fail (HTTP 413). EC-045 (262,145 → 413) and EC-046 (262,143 → 200) are internally consistent with this semantics. BC-DAEMON-003 precondition 2 correctly states "exceeding 262,144 bytes" (trigger threshold). BC-DAEMON-003 postcondition 1's `limit_bytes:262144` response field is the configured limit constant N, not the rejection threshold — also correct. No off-by-one risk in any of the four verification points.

---

## BC-AUTH-002 Two-Body Taxonomy Verification

Per check 10 (architecture back-propagation closure):

| Site | Content | Status |
|------|---------|--------|
| SS-daemon-lifecycle.md v1.0.11 §Behavioral contracts table | 2 rows: Missing header / Invalid token | PASS (F-R65-1 fix) |
| SS-daemon-lifecycle.md v1.0.11 §Behavioral Contract Summary BC-AUTH-002 row | "Two auth failure modes:" | PASS (F-R65-1 fix) |
| PRD v1.5 BC-AUTH-002 postconditions | Items 1 and 2 (missing + any-value-present) | PASS |
| PRD v1.5 BC-AUTH-002 canonical test vectors | 6 rows; no `invalid_auth_token_format` | PASS |
| VP v1.5 VP-AUTH-002 mechanical property | Items 1-4; two-body taxonomy explicit | PASS |
| VP v1.5 VP-AUTH-002 pre-conditions | `AuthError` enum has exactly two variants | PASS |
| VP v1.5 VP-AUTH-002 post-conditions table | 7 probes (probe 5 = Bearer → missing_auth_token) | PASS |

Bearer-header disposition (`Authorization: Bearer` → HTTP 401 `{"error":"missing_auth_token"}`) is consistent across all three artifacts. F-R65-2 (arch) and pre-existing PRD/VP content confirmed aligned.

---

## Routing Recommendations

**Finding R7-001** (LOW severity — stale version citation in VP-DAEMON-001 Test name line):

Route to `vsdd-factory:formal-verifier`.

Fix: At VP v1.5 line 249, change:
```
**Test name:** `test_BC_DAEMON_001_healthz_unauthenticated_alive` (per PRD
v1.4 §BC-DAEMON-001, Verification subsection).
```
to:
```
**Test name:** `test_BC_DAEMON_001_healthz_unauthenticated_alive` (per PRD
v1.5 §BC-DAEMON-001, Verification subsection).
```

This is a single-word citation-accuracy fix. No content change to the test name, harness path, or behavioral contract. BC-DAEMON-001 content is identical between PRD v1.4 and v1.5 (v1.5 only changed EC-045 in §BC-DAEMON-003 body). The §Trace v1.5 should also document this as a missed propagation in its F-R60-corpus-sweep narrative.

**D-047 strict gate implication:** This LOW finding prevents the D-047 strict gate from passing (0 findings of any severity required). The fix is mechanical and bounded to one line. After the formal-verifier applies the fix and commits, the counter remains at 0/3 — this does NOT reset the adversary counter; it is the consistency-validator's finding surfaced ahead of adversary R68.

---

## Gate Result

**GATE: FAIL**

**Reason:** 1 LOW finding (R7-001) — VP-DAEMON-001 Test name annotation cites "PRD v1.4" when PRD is at v1.5. D-047 strict requires 0 findings of any severity.

**Blocking:** YES — under D-047 strict, any finding of any severity is blocking for the convergence counter. The fix is trivial (single word version pin) and should be applied by formal-verifier before adversary R68 is dispatched, or concurrently if the human accepts the risk that adversary R68 may independently surface the same finding.

**Recommended sequence:**
1. Route R7-001 to `vsdd-factory:formal-verifier` for fix.
2. Dispatch adversary R68 after the VP v1.5.1 fix is committed.
3. Dispatch consistency-validator round 8 if R68 produces any findings (per D-047 strict).

**Non-blocking items (none):** No observations or advisories. All other 17 checks PASS cleanly.
