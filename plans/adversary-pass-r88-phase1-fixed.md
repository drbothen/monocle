---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.16 cd6541f + VP v1.21 6ecb79a + arch v1.0.17 a798d51 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 21 (R88); post-F-R87 fix-burst snapshot; CONTENT-CENTRIC LENS"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T23:00:00Z
pass_number: 1
attempt: 21
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 HIGH + 4 MEDIUM + 3 LOW observations
lens_class: CONTENT-CENTRIC (substantive spec correctness)
---

# Adversarial Review R88 — Phase 1 (D-047 Strict, Pass 1 Attempt 21 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

**CRITICAL SHIFT: CONTENT-CENTRIC LENS.** This pass rotates away from the META-axis lens (audit-discipline, orchestration protocol, codification backfill) that has dominated R83–R87 and applies a CONTENT-CENTRIC lens — interrogating the substantive correctness of spec semantics. The result is 5 substantive content defects that 22+ prior META-axis passes missed entirely. This confirms the critical finding: META-axis lens rotations and CONTENT-axis lens rotations are not interchangeable. Alternate lens axes across passes to ensure both classes converge.

- **1 HIGH:** F-R88-1 — arch §Phase 4 Notes (line 747) lock-file field enumeration lists 6 fields as "stable across Phase 1 → Phase 4" but the Phase 1 schema defined in SS-daemon-lifecycle.md has 7 fields including `contract_version` as a forward-compatibility sentinel. The enumeration omits `contract_version`, creating a forward-compatibility contract contradiction: the very field introduced to signal schema evolution to future readers is absent from the stability guarantee list.

- **1 MEDIUM:** F-R88-2 — BC-DAEMON-005 Precondition 2(d) states "all three resolution paths return `None`" as the condition triggering `DaemonStartError::RuntimeDirUnresolvable`. The API description is inaccurate: `directories::ProjectDirs::data_local_dir()` never returns `Option` — it returns `&Path` (a reference, always present when `ProjectDirs::new()` succeeds). The actual failure condition is `ProjectDirs::new()` returning `None`. The wording `data_local_dir()` returns `None` is an API-inaccuracy defect that will produce incorrect implementation.

- **1 MEDIUM:** F-R88-3 — BC-RING-001 EC-001 states the `tool_name` and `tool_input` fields' serialization behavior "depends on serde configuration" without pinning the specific annotation. CLAUDE.md Production-Grade Default Rule 1 forbids this: unresolved serde configuration is not production-grade. The correct annotation is `#[serde(skip_serializing_if = "Option::is_none")]`. This must be pinned explicitly in the BC.

- **1 MEDIUM:** F-R88-4 — BC-DAEMON-005 error-code catalog is missing an edge case for `MONOCLE_RUNTIME_DIR=""` (empty string). The existing EC-058 covers `MONOCLE_RUNTIME_DIR` set to a non-existent path; EC-059 covers `MONOCLE_RUNTIME_DIR` set to a path the process cannot write. Neither covers the empty-string form, which is a distinct failure mode: an explicitly set but empty env var should fail with a clear error, not silently fall through to the ProjectDirs resolution chain. This is an EC-060 candidate.

- **1 MEDIUM:** F-R88-5 — VP verification-properties.md §Mechanism column uniformly labels test harness entries as "unit-test" across multiple VPs, but most test files reside in `tests/` directories (Rust integration tests, not unit tests). PRD §7 RTM correctly labels these as "Integration". The VP §Mechanism mislabeling is a content defect that creates a false impression of test isolation and contradicts PRD §7 RTM.

**Observations (process-relevant):**

- O-R88-1 — Arch §Phase 4 Notes section header (line ~744) lacks a cross-reference to the lock-file schema definition in SS-forward-compatibility.md. A reader encountering the Phase 4 Notes section has no canonical pointer to where the full 7-field schema is authoritatively defined. Adding a section-header note pointing to SS-forward-compatibility.md §Lock-File Schema would prevent future drift recurrences (the omission of `contract_version` from this section may have originated from this missing pointer).

- O-R88-2 — The lens-rotation discipline gap is now empirically confirmed: 22 prior META-axis adversary passes did not surface F-R88-1 through F-R88-5. These are substantive content correctness findings (API accuracy, forward-compatibility contracts, production-grade annotation pins, missing error codes, test-harness classification labels) that are structurally invisible to META-axis review. Recommend: formally codify lens-rotation discipline as Extension 17 candidate — alternate between META-axis and CONTENT-axis passes, tracking which lens class each pass applied.

- O-R88-3 — The CONTENT-CENTRIC lens confirms that VP §Mechanism "unit-test" vs "integration-test" labeling (F-R88-5) is a SYSTEMIC pattern across multiple VPs, not a single-VP defect. A full sweep of all 22 VP §Mechanism blocks for harness-type accuracy is warranted as part of the F-R88-5 fix-burst.

---

## F-R88-1 — HIGH: arch §Phase 4 Notes lock-file field enumeration omits `contract_version`

### Description

Architecture document SS-daemon-lifecycle.md §Phase 4 Notes (approximately line 747) includes a stability annotation for the lock-file schema fields that remain stable across Phase 1 → Phase 4. The enumeration lists 6 fields: `pid`, `socket_path`, `started_at`, `version`, `hostname`, and `capabilities_hash`. It does not list `contract_version`.

The Phase 1 lock-file schema (defined in SS-daemon-lifecycle.md §Lock-File Schema and cross-referenced in SS-forward-compatibility.md) has 7 fields. The 7th field — `contract_version` — is explicitly documented as a forward-compatibility sentinel: it exists so that future Phases can detect whether the running daemon speaks an older or newer protocol version, enabling graceful degradation and migration paths.

By omitting `contract_version` from the "stable across Phase 1 → Phase 4" field list, the Phase 4 Notes section creates a contradiction: the forward-compatibility contract is stated to cover 6 fields when it must cover 7, and the one omitted field is the one whose specific purpose is to signal schema evolution across phases.

### Impact

Any Phase 3/4 implementation that reads the Phase 4 Notes stability guarantee will conclude that `contract_version` is NOT guaranteed to be present in lock files written by Phase 1 daemons, and may not read or validate it. This directly contradicts the forward-compatibility design and will cause Phase 3/4 daemons to fail graceful-degradation detection when encountering Phase 1 lock files.

This is a forward-compatibility contract contradiction — the severity is HIGH because the defect is not detectable at Phase 1 implementation time but surfaces as a runtime failure when Phase 3 or Phase 4 code attempts to interpret Phase 1 lock files.

### Fix routing

- **Architect:** SS-daemon-lifecycle.md §Phase 4 Notes — add `contract_version` to the 6-field enumeration to make it a 7-field list. Cross-reference SS-forward-compatibility.md §Lock-File Schema. Optionally add O-R88-1 section-header pointer (see Observation O-R88-1).
- **PRD:** Propagate arch version pin after arch fix.
- **VP:** Propagate arch version pin after PRD fix.

---

## F-R88-2 — MEDIUM: BC-DAEMON-005 Precondition 2(d) `data_local_dir()` API-inaccurate wording

### Description

BC-DAEMON-005 Precondition 2 specifies the conditions under which the daemon must fail with `DaemonStartError::RuntimeDirUnresolvable`. Precondition 2(d) states that the failure is triggered when "all three resolution paths return `None`" and specifically describes the `directories::ProjectDirs::data_local_dir()` fallback as "returning `None`."

This is API-inaccurate. The `directories` crate's `ProjectDirs::data_local_dir()` method signature is:

```
pub fn data_local_dir(&self) -> &Path
```

It returns `&Path`, not `Option<&Path>`. It NEVER returns `None`. The method is only callable on a `ProjectDirs` instance, which itself can only be obtained if `ProjectDirs::new()` succeeds. If `ProjectDirs::new()` returns `None` (which happens when the home directory cannot be determined), the caller never has a `ProjectDirs` instance and therefore cannot call `data_local_dir()` at all.

The actual failure condition for the fallback path is: `ProjectDirs::new()` returns `None`, not `data_local_dir()` returning `None`.

### Impact

An implementer reading BC-DAEMON-005 Precondition 2(d) will write code that calls `data_local_dir()` and checks its return value for `None`. This will not compile (the return type is `&Path`, not `Option<&Path>`), causing the implementer to either (a) misread the API signature and introduce a type error, or (b) silently rewrite the precondition to match the actual API, diverging from the spec without updating it.

### Fix routing

- **PRD (product-owner):** BC-DAEMON-005 Precondition 2(d) — rewrite to accurately describe the failure condition: `ProjectDirs::new()` returns `None` (home directory unresolvable), causing all three resolution paths (MONOCLE_RUNTIME_DIR, runtime_dir(), data_local_dir() fallback) to be unavailable. The language "returns `None`" should be applied to `ProjectDirs::new()`, not to `data_local_dir()`.
- **VP:** Propagate if VP-DAEMON-005 §Pre-conditions mirrors this wording.

---

## F-R88-3 — MEDIUM: BC-RING-001 EC-001 serde annotation unresolved ("depends on serde configuration")

### Description

BC-RING-001 EC-001 specifies behavior for `Option<serde_json::Value>` fields `tool_name` and `tool_input` in the hook request ring buffer entry. The BC states that when these fields are `None`, their serialization behavior "depends on serde configuration."

This is a production-grade default violation per CLAUDE.md Rule 1: phrases that defer a concrete technical decision without resolving it are RATIONALIZATIONS, not engineering decisions. The correct serde annotation for skip-on-None serialization is:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
```

This annotation MUST be pinned explicitly in the BC. An implementer reading "depends on serde configuration" has no actionable specification and will make an implementation-time decision that may or may not match the intended wire format. If the wire format matters for interoperability (and for hook protocol payloads, it does — downstream consumers may check field presence rather than null value), the serde annotation must be canonical.

### Fix routing

- **PRD (product-owner):** BC-RING-001 EC-001 — replace "depends on serde configuration" with explicit `#[serde(skip_serializing_if = "Option::is_none")]` annotation requirement. Pin the annotation in the BC text as a normative requirement, not a configuration-time decision.
- **VP:** Propagate if VP-RING-001 §Pre-conditions mirrors this wording.

---

## F-R88-4 — MEDIUM: BC-DAEMON-005 missing EC for `MONOCLE_RUNTIME_DIR=""` empty-string edge case

### Description

BC-DAEMON-005 specifies the runtime directory resolution chain. The error-code catalog for this BC includes:

- EC-058: `MONOCLE_RUNTIME_DIR` set to a non-existent path → `RuntimeDirCreationFailed`
- EC-059: `MONOCLE_RUNTIME_DIR` set to a path the process cannot write → `RuntimeDirCreationFailed`

Neither EC covers the case where `MONOCLE_RUNTIME_DIR` is explicitly set to an empty string (`""`). This is a distinct failure mode:

1. An empty string is not `None` (the env var IS set, distinguishable from unset).
2. An empty string is not a valid filesystem path.
3. The correct behavior is to fail-fast with a clear diagnostic error, NOT to fall through to the `ProjectDirs` resolution chain as if the env var were unset.

If the implementation silently treats `""` as unset (e.g., via `env::var("MONOCLE_RUNTIME_DIR").ok().filter(|s| !s.is_empty())`), the BC should specify this explicitly. If the implementation should reject `""` with a distinct error code, the BC must specify that error code.

Either way, the current BC is underspecified for this input case. This is an EC-060 candidate.

### Fix routing

- **PRD (product-owner):** BC-DAEMON-005 — add EC-060 for `MONOCLE_RUNTIME_DIR=""` (empty string). Specify whether the implementation (a) treats empty string as unset and falls through to ProjectDirs chain (with note that this is intentional), or (b) rejects empty string with `DaemonStartError::RuntimeDirInvalid` or a new error variant. Production-grade default: option (b) fail-fast with clear diagnostic.
- **VP:** VP-DAEMON-005 — add probe for EC-060 after EC-059.

---

## F-R88-5 — MEDIUM: VP §Mechanism "unit-test" mislabeling for integration-test harnesses

### Description

Across multiple VP §Mechanism blocks, the test harness type is labeled "unit-test" when the tests reside in `tests/` directories — Rust integration tests, not unit tests. This contradicts PRD §7 RTM, which correctly labels these test files as "Integration" in the Verification Method column.

Rust's module system distinguishes:
- **Unit tests:** `#[cfg(test)]` modules inside `src/` files. They have access to private items.
- **Integration tests:** Files in `tests/` directory. They only have access to public items and are compiled as separate crates.

When a VP §Mechanism block specifies `tests/daemon_lifecycle.rs`, `tests/graceful_shutdown.rs`, etc., those are integration tests. Labeling them "unit-test" is a factual error that:

1. Creates a false impression of test isolation coverage (unit tests verify private internals; integration tests verify public contract behavior).
2. Contradicts PRD §7 RTM's "Integration" label for the same test files.
3. Will cause confusion for implementers who check the VP §Mechanism to understand what kind of test harness to write.

### Scope

O-R88-3 confirms this is a systemic pattern across multiple VP §Mechanism blocks, not an isolated defect. The fix-burst must sweep ALL 22 VP §Mechanism blocks for harness-type accuracy.

### Fix routing

- **VP (formal-verifier):** Sweep all 22 VP §Mechanism blocks. For each VP that specifies a test file path in `tests/`, correct "unit-test" → "integration-test". For VPs that specify test functions inside `src/` (via `#[cfg(test)]`), "unit-test" is correct. Document the classification basis in the §Trace entry. Cross-verify against PRD §7 RTM for each BC to ensure label consistency.

---

## Closure Verification — Prior Finding Classes Still Holding

### F-R80 META closure (Extension 13 — machine-greppable evidence)

Re-verified. VP v1.21 §Trace Extension 16 audit table rows contain actual grep outputs per SE-16c canonical grep. No fabricated PASS verdicts detected at the Extension 3 / Extension 13 axis. F-R80 META closure CONFIRMED HOLDING.

### SE-16c (canonical Extension 16 audit grep target)

Re-verified. VP v1.21 §Trace SE-16c canonical grep transcript section present. 39 rows enumerated. SE-16c discipline correctly applied. No regression.

### META-4 (audit-table enumeration mechanism)

Re-verified. VP v1.21 audit table generated by canonical grep (SE-16c), not manual enumeration. META-4 closure CONFIRMED HOLDING.

---

## Novelty Assessment

All 5 substantive findings (F-R88-1 through F-R88-5) are NOVEL content defects. None are recurrences of prior finding classes:

- F-R88-1: Forward-compatibility contract enumeration gap — new class. Not previously identified by any of the 22 META-axis passes or 5 content-adjacent passes prior to this session.
- F-R88-2: API-accuracy violation (library method return-type mismatch) — new class. The cross-platform/POSIX sweep (Extension 6) covers platform behavior but not API signature accuracy. This is a distinct defect axis.
- F-R88-3: Production-grade annotation deferral ("depends on configuration") — this is an application of CLAUDE.md Production-Grade Default Rule 1, but has not appeared as a BC-text finding before. Prior serde findings were at the VP §Pre-conditions axis (VP-RING-001, VP-PROTO-001b in F-R80/F-R76).
- F-R88-4: Missing error code for environmental input edge case — new class. Prior EC coverage audits (Extension 5, Extension 12) focused on security-property VPs and postcondition anchoring, not environmental-input edge-case completeness.
- F-R88-5: Test harness classification label accuracy (unit-test vs integration-test) — new class. No prior finding addressed VP §Mechanism test-type labeling accuracy vs PRD §7 RTM.

This confirms the CONTENT-CENTRIC LENS rotation produces a structurally different finding-class than META-axis rotations.

---

## Fix Routing Summary

| Finding | Severity | Owner | Target artifact |
|---------|----------|-------|----------------|
| F-R88-1 lock-file field enumeration | HIGH | architect | arch SS-daemon-lifecycle.md (v1.0.18) |
| F-R88-2 data_local_dir() API accuracy | MED | product-owner | PRD BC-DAEMON-005 Precondition 2(d) |
| F-R88-3 serde annotation unresolved | MED | product-owner | PRD BC-RING-001 EC-001 |
| F-R88-4 missing empty-string EC | MED | product-owner | PRD BC-DAEMON-005 EC-060 add |
| F-R88-5 unit-test mislabel | MED | formal-verifier | VP §Mechanism all-22-VP sweep |
| O-R88-1 arch cross-reference pointer | LOW | architect | arch SS-daemon-lifecycle.md §Phase 4 Notes |
| O-R88-2 lens-rotation discipline | LOW | state-manager | Extension 17 candidate codification |
| O-R88-3 VP mislabel systemic scope | LOW | formal-verifier | context for F-R88-5 fix-burst scope |

**Serial fix-burst dependency order per Extension 15:**

1. **Architect first:** arch v1.0.18 — F-R88-1 (lock-file field enum + `contract_version`) + O-R88-1 (section-header pointer).
2. **Product-owner second (after arch v1.0.18 lands):** PRD v1.17 — F-R88-2 (BC-DAEMON-005 2(d) API accuracy) + F-R88-3 (BC-RING-001 EC-001 serde pin) + F-R88-4 (EC-060 empty-string) + arch v1.0.18 pin propagation + O-R88-1 header note propagation.
3. **Formal-verifier third (after PRD v1.17 lands):** VP v1.22 — F-R88-2 mirror in VP-DAEMON-005 §Pre-conditions + F-R88-5 unit-test→integration-test sweep (all 22 VPs) + PRD v1.17 + arch v1.0.18 pin propagation.
