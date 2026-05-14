---
document_type: consistency-audit
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.1 f855835 + VP v1.1 8454ff2 + arch v1.0.8 2db408f + STATE.md v5.1 9d00138; F-R62 fix-burst applied"
level: ops
producer: consistency-validator
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-14T23:45:00Z
round: 2
---

# Consistency Audit — Round 2 (Post F-R62 Fix-Burst)

**Artifacts under audit:**
- PRD v1.1 (commit f855835)
- Verification Properties v1.1 (commit 8454ff2)
- SS-daemon-lifecycle.md v1.0.8 (commit 2db408f)

**Cross-document anchors checked:**
- product-brief.md v1.4.23 (actual: v1.4.23) ✓
- domain-monocle-vision-synthesis.md v1.1.2 (actual: v1.1.2) ✓
- SS-daemon-lifecycle.md v1.0.8 (actual: v1.0.8) ✓
- SS-core-types-and-abi.md v1.2.8 (actual: v1.2.8) ✓
- SS-engine-module.md v1.1.15 (actual: v1.1.15) ✓
- SS-deps-pin-manifest.md (actual: v1.1.8)
- SS-conventions-anti-patterns.md v1.28 (actual: v1.28) ✓
- SS-permissions-phase1.md (actual: v1.4)
- SS-forward-compatibility.md (actual: v1.2.13)
- dtu-assessment.md v1.7 (actual: v1.7) ✓
- STATE.md v5.1 (actual: v5.1) ✓

---

## Verdict

**GAPS — 3 findings (1 HIGH, 2 MEDIUM)**

D-047 strict (0 findings of any severity) requires 3 consecutive clean passes. This is pass 1; GAPS blocks advancement to pass 2.

---

## Audit Results Table

| # | Check | Result | Evidence |
|---|-------|--------|----------|
| 1 | BC inventory coherence (22-BC) | PASS | PRD RTM: 22 rows ✓; VP §Coverage Matrix: 22 rows ✓; SS-daemon-lifecycle §BC Summary: 10 rows (daemon-lifecycle BCs only) ✓; STATE.md: "22 BCs" ✓ |
| 2 | BC↔VP 1:1 ID coherence (22-VP) | PASS — see matrix below | All 22 BCs have exactly one VP; zero orphans either direction |
| 3 | Test-file path coherence (PRD RTM vs VP Coverage Matrix) | PASS on file paths, FAIL on test names | File paths: all 22 identical. Test names: 3 mismatches + 10 missing from VPs (see F-R63-1) |
| 4 | Version-pin coherence (PG-5) | PASS (versioned pins correct; non-versioned references acceptable per PG-5) | See version-pin drift table below; all cited versions match actuals |
| 5 | §-anchor resolution (PG-4) | PASS | PRD §Trace v1.1 PG-4 sweep verified all anchors; VP §Trace PG-4 sweep verified all anchors; SS-daemon-lifecycle anchors in PRD/VP verified against actual headings |
| 6 | Count coherence (PG-2) | PARTIAL PASS — see F-R63-2 | 22 BCs ✓; 22 VPs ✓; 5 hook endpoints ✓; 56 edge cases (EC-001..EC-056) ✓; 13 error codes in §5 actual vs "14 error codes" in §Trace v1.0 historical reference (not corrected to 13 in v1.1 §Trace) |
| 7 | §Trace chain integrity | PASS | PRD §Trace v1.1 present; VP §Trace present; both use §-anchor refs, no bare L-numbers, no directional qualifiers; PG-3 compliant |
| 8 | Error taxonomy cross-check | PASS | BC-AUTH-002 two-body taxonomy consistent across PRD §5 (E-AUTH-001, E-AUTH-002), PRD BC-AUTH-002 postconditions, VP-AUTH-002, and SS-daemon-lifecycle.md v1.0.8 BC-AUTH-002 table. `invalid_auth_token_format` confirmed absent from all three artifacts |
| 9 | Scope-boundary coverage | PASS | All 10 daemon-lifecycle BCs from SS-daemon-lifecycle §BC Summary are formalized in PRD v1.1; BC-DAEMON-001..006 + BC-RING-001 + BC-AUTH-001 + BC-AUTH-002 + BC-LOCK-001 = 10 ✓ |
| 10 | Frozen META catalog status | PASS | F-R55-adv-1, F-R55-adv-3, F-R61-adv-1, F-R61-2 — none present in PRD v1.1 or VP v1.1 normative content |
| 11 | Naming convention (CLAUDE.md) | PASS | "Monocle" in headings, "monocle" in code/crate identifiers; consistent throughout all three artifacts |
| 12 | Forbidden phrases | PASS | No "for now," "good enough," "MVP," "minimum viable," "TODO for architect," "Pending architect review" in normative content of PRD v1.1, VP v1.1, or SS-daemon-lifecycle.md v1.0.8 |
| 13 | VP frontmatter `phase` and `status` | PASS | `phase: phase-1-spec-crystallization` ✓ (F-R62-5 fix confirmed); `status: draft` ✓ (F-R62-5 fix confirmed) |
| 14 | STATE.md coherence | PASS | STATE.md v5.1 reflects: 22-BC count ✓; Phase 1 in progress ✓; T-1/T-2/T-3/T-4 complete ✓; T-7..T-13 queued ✓; F-R62 fix-burst complete ✓ |

---

## Findings Table

### F-R63-1 — Test Name Drift: PRD vs VP (13 mismatches/gaps)

| ID | Severity | Category | File | Evidence | Recommended Route |
|----|----------|----------|------|----------|-------------------|
| F-R63-1 | HIGH | Spec Drift (test names) | VP v1.1 `verification-properties.md` | Three test names explicitly mismatched between PRD v1.1 and VP v1.1; 10 additional test names absent from VP v1.1 entirely. See detail below. | `vsdd-factory:formal-verifier` — VP v1.1 is the formal-verifier's artifact; test names must match PRD v1.1 §7 RTM as the canonical source per F-R62-4 closure. Route to `vsdd-factory:product-owner` only if PRD names need changing (prefer VP names to track PRD). |

**Explicit mismatches (PRD canonical → VP actual):**

| BC ID | PRD v1.1 Test Name | VP v1.1 Test Name | Gap Type |
|-------|-------------------|--------------------|----------|
| BC-ABI-001 | `test_BC_ABI_001_status_abi_version_field` | `test_BC_ABI_001_status_endpoint_returns_abi_version_1` | Mismatch |
| BC-ENGINE-002 | `test_BC_ENGINE_002_claude_code_module_detect` | `test_BC_ENGINE_002_claude_code_module_strict_basename_detect` | Mismatch |
| BC-ENGINE-002-ERR | `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich` | `test_BC_ENGINE_002_ERR_home_unresolvable_sync_and_async` | Mismatch |
| BC-ENGINE-003 | `test_BC_ENGINE_003_hook_paths_five_entries` | `test_BC_ENGINE_003_claude_module_inherent_hook_paths` | Mismatch |

**Absent test names in VP v1.1 (harness location present; test name omitted):**

| BC ID | PRD v1.1 Test Name | VP v1.1 §VP-NNN-* Status |
|-------|-------------------|--------------------------|
| BC-RING-001 | `test_BC_RING_001_format_version_first_key` | Harness location present; no `**Test name:**` line |
| BC-AUTH-001 | `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip` | Harness location present; no `**Test name:**` line |
| BC-LOCK-001 | `test_BC_LOCK_001_contract_version_first_key` | Harness location present; no `**Test name:**` line |
| BC-ABI-002 | `test_BC_ABI_002_abi_version_const_exported` | Harness location present; no `**Test name:**` line |
| BC-TYPES-001 | `test_BC_TYPES_001_non_exhaustive_enum_coverage` | Harness location present; no `**Test name:**` line |
| BC-FACTORY-001 | `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound` | Harness location present; no `**Test name:**` line |
| BC-FACTORY-002 | `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection` | Harness location present; no `**Test name:**` line |
| BC-PROTO-001a | `test_BC_PROTO_001a_schema_version_field_number_1` | Harness location present; no `**Test name:**` line |
| BC-PROTO-001b | `test_BC_PROTO_001b_schema_version_rust_field` | Harness location present; no `**Test name:**` line |
| BC-ENGINE-001 | `test_BC_ENGINE_001_trait_defined_all_methods_no_sealed_bound` | Harness location present; no `**Test name:**` line |

**Root cause:** VP v1.1 F-R62-4 closure propagated file paths from PRD v1.1 RTM but did not propagate test names for the 10 original architecture-staged BCs. The 6 new VP-DAEMON-NNN entries do carry test names (matching PRD exactly). The 4 mismatched names for ABI-001, ENGINE-002, ENGINE-002-ERR, ENGINE-003 suggest the VP was authored before the PRD test names were finalized for those entries.

**Impact:** During Phase 3 TDD delivery, test-writer agent resolves test names from the VP. If VP test names diverge from PRD, the implementing test may use the VP name, diverging from the contract spec. Under D-047 strict, this is a blocking gap.

---

### F-R63-2 — §Trace v1.0 Historical Error Count Not Updated in v1.1 (13 actual, 14 stated)

| ID | Severity | Category | File | Evidence | Recommended Route |
|----|----------|----------|------|----------|-------------------|
| F-R63-2 | MEDIUM | Count Coherence (PG-2) | PRD v1.1 `prd.md` §Trace | §Trace v1.0 states "Error taxonomy: 14 error codes". F-R62-8 retired `invalid_auth_token_format` reducing the taxonomy to 13 error codes. PRD §5 has 13 rows (E-AUTH-001, E-AUTH-002, E-DAEMON-001..003, E-LOCK-001..003, E-ENG-001, E-FACT-001..002, E-RING-001, E-PROTO-001). §Trace v1.1 narrates the retirement of `invalid_auth_token_format` but does not explicitly state the updated count is 13. | `vsdd-factory:product-owner` — add a PG-2 correction in §Trace v1.1 explicitly stating the updated error code count: "Error taxonomy updated: 14 → 13 error codes (invalid_auth_token_format retired per F-R62-8)." |

**Evidence detail:**
- §Trace v1.0 (historical): "Error taxonomy: 14 error codes covering all error surfaces across 6 subsystem abbreviations."
- §Trace v1.1: documents `invalid_auth_token_format` retirement but omits updated count.
- Actual §5 row count: 13 rows.
- PG-2 requires that narrative count claims be updated when the underlying table changes. The §Trace v1.0 entry is a historical record, but the v1.1 §Trace's F-R62-8 resolution block should carry the corrected count.

---

### F-R63-3 — SS-daemon-lifecycle §BC Summary Footer: Stale Forward-Looking Language

| ID | Severity | Category | File | Evidence | Recommended Route |
|----|----------|----------|------|----------|-------------------|
| F-R63-3 | MEDIUM | Artifact Staleness | SS-daemon-lifecycle.md v1.0.8 §Behavioral Contract Summary | Lines 586-588: "The Phase 1 PRD will formalize these as full BC entries with postconditions, evidence, and verification harness stubs. This artifact pre-stages them for the Phase 1 architecture gate." — This is future-tense language describing what PRD v1.1 (f855835) has already done. The prediction is now fulfilled; the text is stale. | `vsdd-factory:architect` — update §BC Summary footer from forward-looking to present-tense: e.g., "The Phase 1 PRD v1.1 (commit f855835) formalizes these as full BC entries with postconditions, evidence, and verification harness stubs." Bump SS-daemon-lifecycle.md to v1.0.9 with the §Trace entry for this change. |

**Evidence detail:**
- SS-daemon-lifecycle.md v1.0.8 line 586: "The Phase 1 PRD will formalize these as full BC entries..."
- PRD v1.1 commit f855835 has formalized BC-DAEMON-001..006 with full preconditions, postconditions, invariants, edge cases, canonical test vectors, and verification specification.
- The forward-looking language is now retrospectively false: it describes the future state that has been achieved.
- Additionally: SS-daemon-lifecycle.md v1.0.8 §Start Sequence BC-AUTH-002 verification block (line 318) cites `monocle-runtime/tests/auth.rs` as the test file; PRD v1.1 §7 RTM (F-R62-4 canonical path authority) names `monocle-runtime/tests/auth_header_rejection.rs`. This is a secondary stale reference in the same artifact (arch doc pre-dates the F-R62-4 canonicalization).

---

## Cross-File BC↔VP Matrix (22 rows)

| BC ID | VP ID | PRD Test File | VP Test File | Test Files Match | Test Name (PRD) | Test Name (VP) | Test Names Match |
|-------|-------|--------------|--------------|-----------------|-----------------|-----------------|-----------------|
| BC-DAEMON-001 | VP-DAEMON-001 | `monocle-runtime/tests/healthz_endpoint.rs` | `monocle-runtime/tests/healthz_endpoint.rs` | ✓ | `test_BC_DAEMON_001_healthz_unauthenticated_alive` | `test_BC_DAEMON_001_healthz_unauthenticated_alive` | ✓ |
| BC-DAEMON-002 | VP-DAEMON-002 | `monocle-runtime/tests/status_endpoint_auth.rs` | `monocle-runtime/tests/status_endpoint_auth.rs` | ✓ | `test_BC_DAEMON_002_status_endpoint_requires_auth_and_returns_abi_version` | `test_BC_DAEMON_002_status_endpoint_requires_auth_and_returns_abi_version` | ✓ |
| BC-DAEMON-003 | VP-DAEMON-003 | `monocle-runtime/tests/body_size_limit.rs` | `monocle-runtime/tests/body_size_limit.rs` | ✓ | `test_BC_DAEMON_003_body_size_limit_413_on_excess` | `test_BC_DAEMON_003_body_size_limit_413_on_excess` | ✓ |
| BC-DAEMON-004 | VP-DAEMON-004 | `monocle-runtime/tests/graceful_shutdown.rs` | `monocle-runtime/tests/graceful_shutdown.rs` | ✓ | `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests` | `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests` | ✓ |
| BC-DAEMON-005 | VP-DAEMON-005 | `monocle-runtime/tests/lock_file_lifecycle.rs` | `monocle-runtime/tests/lock_file_lifecycle.rs` | ✓ | `test_BC_DAEMON_005_lock_file_create_and_cleanup` | `test_BC_DAEMON_005_lock_file_create_and_cleanup` | ✓ |
| BC-DAEMON-006 | VP-DAEMON-006 | `monocle-runtime/tests/crash_recovery.rs` | `monocle-runtime/tests/crash_recovery.rs` | ✓ | `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup` | `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup` | ✓ |
| BC-RING-001 | VP-RING-001 | `monocle-runtime/tests/jsonl_ring.rs` | `monocle-runtime/tests/jsonl_ring.rs` | ✓ | `test_BC_RING_001_format_version_first_key` | (absent) | MISSING |
| BC-AUTH-001 | VP-AUTH-001 | `monocle-runtime/tests/auth_token_lifecycle.rs` | `monocle-runtime/tests/auth_token_lifecycle.rs` | ✓ | `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip` | (absent) | MISSING |
| BC-AUTH-002 | VP-AUTH-002 | `monocle-runtime/tests/auth_header_rejection.rs` | `monocle-runtime/tests/auth_header_rejection.rs` | ✓ | `test_BC_AUTH_002_auth_header_validation_all_failure_modes` | `test_BC_AUTH_002_auth_header_validation_all_failure_modes` | ✓ |
| BC-LOCK-001 | VP-LOCK-001 | `monocle-runtime/tests/lock_file_contract.rs` | `monocle-runtime/tests/lock_file_contract.rs` | ✓ | `test_BC_LOCK_001_contract_version_first_key` | (absent) | MISSING |
| BC-ABI-001 | VP-ABI-001 | `monocle-runtime/tests/status_abi_version.rs` | `monocle-runtime/tests/status_abi_version.rs` | ✓ | `test_BC_ABI_001_status_abi_version_field` | `test_BC_ABI_001_status_endpoint_returns_abi_version_1` | MISMATCH |
| BC-ABI-002 | VP-ABI-002 | `monocle-core/tests/abi_stability.rs` | `monocle-core/tests/abi_stability.rs` | ✓ | `test_BC_ABI_002_abi_version_const_exported` | (absent) | MISSING |
| BC-TYPES-001 | VP-TYPES-001 | `monocle-core/tests/enum_audit.rs` | `monocle-core/tests/enum_audit.rs` | ✓ | `test_BC_TYPES_001_non_exhaustive_enum_coverage` | (absent) | MISSING |
| BC-FACTORY-001 | VP-FACTORY-001 | `monocle-core/tests/factory_trait_surface.rs` | `monocle-core/tests/factory_trait_surface.rs` | ✓ | `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound` | (absent) | MISSING |
| BC-FACTORY-002 | VP-FACTORY-002 | `monocle-core/tests/factory_self_referential.rs` | `monocle-core/tests/factory_self_referential.rs` | ✓ | `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection` | (absent) | MISSING |
| BC-PROTO-001a | VP-PROTO-001a | `monocle-proto/tests/wire_field_order.rs` | `monocle-proto/tests/wire_field_order.rs` | ✓ | `test_BC_PROTO_001a_schema_version_field_number_1` | (absent) | MISSING |
| BC-PROTO-001b | VP-PROTO-001b | `monocle-proto/tests/schema_version.rs` | `monocle-proto/tests/schema_version.rs` | ✓ | `test_BC_PROTO_001b_schema_version_rust_field` | (absent) | MISSING |
| BC-PROTO-002 | VP-PROTO-002 | Phase 4 (no Phase 1 harness) | Phase 4 (no Phase 1 harness) | ✓ | `test_BC_PROTO_002_schema_version_validation_skip_unknown` (Phase 4) | (Phase 4 deferred; no Phase 1 name) | N/A — deferred |
| BC-ENGINE-001 | VP-ENGINE-001 | `monocle-core/tests/engine_module_surface.rs` | `monocle-core/tests/engine_module_surface.rs` | ✓ | `test_BC_ENGINE_001_trait_defined_all_methods_no_sealed_bound` | (absent) | MISSING |
| BC-ENGINE-002 | VP-ENGINE-002 | `monocle-runtime/tests/engine_module_claude_detect.rs` | `monocle-runtime/tests/engine_module_claude_detect.rs` | ✓ | `test_BC_ENGINE_002_claude_code_module_detect` | `test_BC_ENGINE_002_claude_code_module_strict_basename_detect` | MISMATCH |
| BC-ENGINE-002-ERR | VP-ENGINE-002-ERR | `monocle-runtime/tests/engine_module_home_unresolvable.rs` | `monocle-runtime/tests/engine_module_home_unresolvable.rs` | ✓ | `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich` | `test_BC_ENGINE_002_ERR_home_unresolvable_sync_and_async` | MISMATCH |
| BC-ENGINE-003 | VP-ENGINE-003 | `monocle-runtime/tests/engine_module_claude_methods.rs` | `monocle-runtime/tests/engine_module_claude_methods.rs` | ✓ | `test_BC_ENGINE_003_hook_paths_five_entries` | `test_BC_ENGINE_003_claude_module_inherent_hook_paths` | MISMATCH |

**Summary:**
- File path coherence: 22/22 identical ✓ (F-R62-4 closure confirmed)
- Test name coherence: 11/22 explicitly stated + matched; 4 mismatched; 10 absent from VP; 1 deferred (BC-PROTO-002 Phase 4)
- Total test name gaps: 14 (4 mismatches + 10 absent)

---

## Version-Pin Drift Table

All version citations are in PRD v1.1 `traces_to:` frontmatter and §Trace bodies; VP v1.1 `traces_to:` frontmatter; where version is cited in body text it is verified against actual frontmatter.

| Source File | Cited Version | Actual Frontmatter Version | Status |
|-------------|--------------|---------------------------|--------|
| product-brief.md | v1.4.23 | v1.4.23 | ✓ MATCH |
| domain-monocle-vision-synthesis.md | v1.1.2 | v1.1.2 | ✓ MATCH |
| SS-daemon-lifecycle.md | v1.0.8 | v1.0.8 | ✓ MATCH |
| SS-core-types-and-abi.md | v1.2.8 | v1.2.8 | ✓ MATCH |
| SS-engine-module.md | v1.1.15 | v1.1.15 | ✓ MATCH |
| SS-deps-pin-manifest.md | (no version cited in PRD/VP body; input path only) | v1.1.8 | ✓ ACCEPTABLE — PG-5 current-pointer; no version assertion in body |
| SS-conventions-anti-patterns.md | v1.28 (cited in PRD §8.4 cross-ref only; body cite style) | v1.28 | ✓ MATCH |
| SS-permissions-phase1.md | (no version cited in body; input path only) | v1.4 | ✓ ACCEPTABLE — PG-5 current-pointer |
| SS-forward-compatibility.md | (no version cited in body; input path only) | v1.2.13 | ✓ ACCEPTABLE — PG-5 current-pointer |
| dtu-assessment.md | v1.7 (cited in traces_to) | v1.7 | ✓ MATCH |
| STATE.md | v5.1 (cited in traces_to) | v5.1 | ✓ MATCH |

**Version-pin drift count: 0** — all cited versions match actual frontmatter versions.

---

## Frozen META Catalog Status

Per D-054, the 4 frozen META catalog entries must not be re-introduced in Phase 1 artifacts.

| ID | Description | Present in PRD v1.1 | Present in VP v1.1 | Present in arch v1.0.8 | Status |
|----|-------------|---------------------|-------------------|----------------------|--------|
| F-R55-adv-1 | PG-4 em-dash separator gap | No | No | No | ✓ NOT REINTRODUCED |
| F-R55-adv-3 | PG-4 intra-document scope hole | No | No | No | ✓ NOT REINTRODUCED |
| F-R61-adv-1 | PG-3-CLASSIFICATION-EVIDENCE META rule §Trace bare L-numbers | No | No | No | ✓ NOT REINTRODUCED |
| F-R61-2 | §Trace-Heading-Convention scope clause | No | No | No | ✓ NOT REINTRODUCED |

All 4 frozen META catalog entries: confirmed absent from all three audited artifacts.

---

## Supplementary Observations (Not Blocking)

These items do not constitute findings under D-047 strict but are surfaced for completeness.

**OBS-1 (SS-daemon-lifecycle §BC-AUTH-002 label):** The §Behavioral Contract Summary at line 583 and §Start Sequence BC-AUTH-002 inline contract (line 302) use the phrase "Three auth failure modes" referring to the 3 middleware rules (missing, format-fail, secret-mismatch), but the table has only 2 rows (2 distinct response bodies). The PRD consistently says "two-body taxonomy." The arch doc's "three failure modes" label is internally coherent (it refers to rules, not bodies) but could confuse readers who expect "modes" = "bodies." Not a functional gap — no incorrect contract surface.

**OBS-2 (SS-daemon-lifecycle BC-AUTH-002 old test path):** §Start Sequence BC-AUTH-002 verification (line 318) cites `monocle-runtime/tests/auth.rs` (pre-F-R62-4 path). PRD v1.1 §7 RTM is the canonical test path source; this arch doc pre-date citation is a secondary reference that implementers will supersede with the PRD. The architect's update in v1.0.8 was scoped to the contract content, not the verification path citation. This is secondary to F-R63-3 and is included in that finding's evidence detail.

---

## Gate Decision

**VERDICT: GAPS — 3 findings block D-047 strict pass 1 advancement.**

| Finding | Severity | Blocks D-047 Pass? | Routing |
|---------|----------|-------------------|---------|
| F-R63-1 Test name drift (13 gaps) | HIGH | YES | formal-verifier (VP v1.1 fixes) |
| F-R63-2 Error count not updated in §Trace v1.1 | MEDIUM | YES | product-owner (PRD §Trace update) |
| F-R63-3 Arch stale forward-looking language | MEDIUM | YES | architect (SS-daemon-lifecycle v1.0.9) |

**All 3 findings must be resolved before round 3 (consistency-validator) and R63 adversary pass 2 can proceed.**

Under CLAUDE.md Correct Agent Routing principle: do NOT fix any of these findings here. Route each to the designated specialist via the orchestrator. The consistency-validator is read-only.
