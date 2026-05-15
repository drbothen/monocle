---
document_type: consistency-audit
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.7 3024bd3 + VP v1.7 296b044 + arch v1.0.13 1f53d47 + manifest v1.1.9 1f53d47 + STATE.md v5.8 eac0cf7; F-R71 closure chain applied; Extension 3 Enforcement applied"
level: ops
producer: consistency-validator
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T02:54:43Z
round: 11
---

# Consistency Audit — Round 11
## Post-F-R71 Phase 1 Spec Package

**Artifacts audited:**
- PRD v1.7 (commit 3024bd3)
- VP catalog v1.7 (commit 296b044)
- SS-daemon-lifecycle.md v1.0.13 (commit 1f53d47)
- SS-deps-pin-manifest.md v1.1.9 (commit 1f53d47)
- STATE.md v5.8 (commit eac0cf7)

**Extensions applied:** L-F-R63 Extension 1 (arch pin propagation) + Extension 2 (intra-block consistency) + Extension 3 Enforcement (deps-pin-manifest sweep, Obs-R71-1 codification)

---

## Executive Summary

**VERDICT: CLEAN — 0 gaps found.**

All F-R71 closure items (F-R71-1 through F-R71-5) are confirmed propagated correctly and consistently across all four artifacts. The mandatory deps-pin sweep across 25 crates + 1 dev-dep produces zero stale normative pins. BC↔VP 1:1 coverage holds. Counts (22 BCs, 14 errors, 59 ECs, 23 test names) are consistent. The standard 16 cross-artifact checks produce no findings. No routing actions required.

---

## 1. F-R71 Closure Verification

### F-R71-1: `directories 5` → `directories 6` (HIGH)

**Finding (R71):** VP-DAEMON-005 §Pre-conditions referenced `directories 5 (or pinned equivalent)` — stale pin; manifest v1.1.9 pin is `directories 6`. Hedge language `(or pinned equivalent)` violated Principle 6.

**Closure verification:**

| Site | Artifact | Expected | Found | Status |
|------|----------|----------|-------|--------|
| VP-DAEMON-005 §Pre-conditions (line 643) | VP v1.7 | `directories 6 (per SS-deps-pin-manifest.md v1.1.9)` | `directories 6 (per SS-deps-pin-manifest.md v1.1.9)` | PASS |
| §Trace v1.6 historical entry (line 2562) | VP v1.7 | `directories 6 (now ... per F-R71 v1.7 closure; was directories 5 at v1.6 authoring time)` | `now directories 6 per F-R71 v1.7 closure; was directories 5 at v1.6 authoring time` | PASS (PG-5 historical annotation correct) |
| Hedge language `(or pinned equivalent)` | VP v1.7 | ABSENT in normative body | Not present in VP-DAEMON-005 §Pre-conditions | PASS |
| SS-deps-pin-manifest.md Phase 1 Pin Manifest | manifest v1.1.9 | `directories | 6 | ...` | `directories | 6 | XDG-compliant config/data/runtime dirs | caret pin` | PASS — canonical source correct |

**F-R71-1 STATUS: CLOSED.**

---

### F-R71-2: Stale test name `test_BC_DAEMON_004_exit_codes` (HIGH)

**Finding (R71):** SS-daemon-lifecycle.md v1.0.12 contained a stale test name reference at 2 sites in the arch body. Correct canonical name per PRD §RTM: `test_BC_DAEMON_004_exit_codes_posix_distinct`.

**Closure verification:**

| Site | Artifact | Expected | Found | Status |
|------|----------|----------|-------|--------|
| §Hard Shutdown / BC-DAEMON-004 verification note | arch v1.0.13 (line 635) | `test_BC_DAEMON_004_exit_codes_posix_distinct` | `test_BC_DAEMON_004_exit_codes_posix_distinct` | PASS |
| §BC Summary verification cross-ref | arch v1.0.13 (line 731) | `test_BC_DAEMON_004_exit_codes_posix_distinct` | `test_BC_DAEMON_004_exit_codes_posix_distinct` | PASS |
| VP-DAEMON-004 §Test names (line 557) | VP v1.7 | `test_BC_DAEMON_004_exit_codes_posix_distinct` | `test_BC_DAEMON_004_exit_codes_posix_distinct` | PASS |
| PRD §BC-DAEMON-004 Verification (line 306) | PRD v1.7 | `test_BC_DAEMON_004_exit_codes_posix_distinct` | `test_BC_DAEMON_004_exit_codes_posix_distinct` | PASS |
| PRD §7 RTM BC-DAEMON-004 row | PRD v1.7 (line 1264) | `monocle-runtime/tests/daemon_lifecycle.rs` | present in test file column | PASS |

**F-R71-2 STATUS: CLOSED.**

---

### F-R71-3: NFR-008 sole-primary framing (MEDIUM)

**Finding (R71):** PRD BC-DAEMON-005 precondition 2 rationale and arch §Start Sequence step 1 Rationale used "macOS is the primary target (NFR-008)" — implying sole-primary. NFR-008 specifies `macOS + Linux (darwin/linux × amd64/arm64)` — coequal targets.

**Closure verification:**

| Site | Artifact | Expected | Found | Status |
|------|----------|----------|-------|--------|
| BC-DAEMON-005 precondition 2 rationale (line 328) | PRD v1.7 | "macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64)" | "macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64)" | PASS |
| §Scope para (line 35) | arch v1.0.13 | "NFR-008 lists macOS among the primary targets (`macOS + Linux`, darwin/linux × amd64/arm64)" | `NFR-008 lists macOS among the primary targets (\`macOS + Linux\`, darwin/linux × amd64/arm64)` | PASS |
| §Start Sequence step 1 Rationale (line 197) | arch v1.0.13 | "macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64)" | "macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64)" | PASS |
| NFR-008 row in PRD §4 (line 1210) | PRD v1.7 | `macOS + Linux (darwin/linux × amd64/arm64)` | `macOS + Linux (darwin/linux × amd64/arm64)` | PASS |
| Remaining "macOS is the primary target" phrases | PRD v1.7 normative body | ABSENT | No match found in §3, §4, §6, §10 normative body | PASS |

**F-R71-3 STATUS: CLOSED.**

---

### F-R71-4a: Tower fabricated workspace citation (MEDIUM)

**Finding (R71):** VP-DAEMON-004 §Pre-conditions stated "axum 0.8, tokio 1, tower 0.5 are the project pins (per SS-deps-pin-manifest.md)" — tower is NOT in the manifest; it is a transitive dep of axum 0.8.

**Closure verification:**

| Site | Artifact | Expected | Found | Status |
|------|----------|----------|-------|--------|
| VP-DAEMON-004 §Pre-conditions (lines 473-475) | VP v1.7 | `axum 0.8 and tokio 1 are the project pins (per SS-deps-pin-manifest.md); tower is a transitive dependency of axum 0.8 (no direct workspace pin)` | `axum 0.8 and tokio 1 are the project pins (per SS-deps-pin-manifest.md); tower is a transitive dependency of axum 0.8 (no direct workspace pin).` | PASS |
| `tower 0.5` in VP normative body | VP v1.7 | ABSENT (except PG-5 historical/traces_to) | Not found outside traces_to (PG-5 preserved) | PASS |
| SS-deps-pin-manifest.md Phase 1 Pin Manifest | manifest v1.1.9 | tower NOT present as workspace pin | tower absent from pin table; §Trace F-R71-4a documented transitive disposition | PASS |

**F-R71-4a STATUS: CLOSED.**

---

### F-R71-4b: nix-OR-libc OR-disjunction Principle 6 violation (MEDIUM)

**Finding (R71):** VP-DAEMON-005 §Pre-conditions contained an unresolved OR-disjunction "nix 0.30 OR libc 0.2" for POSIX signal handling — a Principle 6 violation (answerable in current scope; architect had not adjudicated). Architect in commit 1f53d47 selected `nix 0.30` as sole binding.

**Closure verification:**

| Site | Artifact | Expected | Found | Status |
|------|----------|----------|-------|--------|
| VP-DAEMON-005 §Pre-conditions (lines 646-648) | VP v1.7 | `nix 0.30 is the project pin (per SS-deps-pin-manifest.md v1.1.9) for the pid-liveness probe; the test asserts nix::sys::signal::kill(Pid::from_raw(pid), None) per BC-DAEMON-005 postcondition 3` | `nix 0.30 is the project pin (per SS-deps-pin-manifest.md v1.1.9) for the pid-liveness probe; the test asserts nix::sys::signal::kill(Pid::from_raw(pid), None) per BC-DAEMON-005 postcondition 3.` | PASS |
| OR-disjunction "nix 0.30 OR libc 0.2" | VP v1.7 normative body | ABSENT | Not found outside traces_to (PG-5 preserved) | PASS |
| manifest §Trace F-R71-4b (lines 258-279) | manifest v1.1.9 | nix 0.30 caret pin documented + rationale | `nix = "0.30"` in Phase 1 Pin Manifest (line 61); §Trace F-R71-4b documents binding decision | PASS |
| BC-DAEMON-005 postcondition 3 | arch v1.0.13 (line 772) | `nix::sys::signal::kill(Pid::from_raw(pid), None)` | present in arch v1.0.13 §Trace F-R71-4b | PASS |
| `libc 0.2` in VP normative body | VP v1.7 | ABSENT | Not found outside traces_to historical narrative | PASS |

**F-R71-4b STATUS: CLOSED.**

---

### F-R71-5: VP-DAEMON-006 `"pid": <int>` placeholder (LOW)

**Finding (R71):** VP-DAEMON-006 §Mechanical property item 1 JSON schema sketch used `"pid": <int>` — inconsistent with arch + PRD convention which uses `<N>` for integer placeholders.

**Closure verification:**

| Site | Artifact | Expected | Found | Status |
|------|----------|----------|-------|--------|
| VP-DAEMON-006 §Mechanical property item 1 (line 777) | VP v1.7 | `"pid": <N>,` | `"pid": <N>,` | PASS |
| `"pid": <int>` in VP normative body | VP v1.7 | ABSENT | grep returns 0 matches | PASS |

**F-R71-5 STATUS: CLOSED.**

---

## 2. Mandatory Deps-Pin Sweep (Extension 3 / Obs-R71-1)

The following 25 production crates + 1 dev-dep from SS-deps-pin-manifest.md v1.1.9 were swept across all four spec artifacts. Each crate version mention in normative body was classified against the canonical manifest row.

**Crate sweep regex applied:**
`\b(nix|libc|tower|directories|axum|tokio|prost|tempfile|tracing|temp-env|constant_time_eq|serde_json|serde_yaml_ng|rand|notify|interprocess|crossterm|ratatui|reqwest|russh|rmcp|nucleo|wasmtime|thiserror|anyhow)\s+[0-9]`

### Classification Table

| Crate | Manifest Pin | Normative mentions found | Classification |
|-------|-------------|--------------------------|----------------|
| `nix` | 0.30 (caret) | VP-DAEMON-005 §Pre-conditions `nix 0.30` (x3); arch §Trace `nix 0.30` (historical-trace only); manifest row `nix | 0.30` | PASS — all normative sites = 0.30; historical traces PG-5 exempt |
| `tower` | NOT a direct workspace pin (transitive via axum 0.8) | VP-DAEMON-004 §Pre-conditions: "tower is a transitive dependency of axum 0.8 (no direct workspace pin)"; arch §Trace `tower 0.5` (historical-trace only) | PASS — normative sites correctly describe tower as transitive; no stale "tower 0.5 per manifest" citation |
| `directories` | 6 (caret) | VP-DAEMON-005 §Pre-conditions `directories 6` (x3, including v1.1.9 cite); manifest row `directories | 6`; arch §Start Sequence uses API without version cite; PRD references API without pin cite | PASS — all versioned normative sites = 6; PG-5 historical narrative (v1.6 authoring time) correctly annotated |
| `axum` | =0.8.9 (exact) | VP-DAEMON-001/004 §Pre-conditions `axum 0.8`; arch §Scope + snippets `axum 0.8`; manifest row `axum | 0.8` | PASS — `axum 0.8` is a correct current-pointer to the 0.8.x EXACT pin series; all normative sites consistent |
| `tokio` | =1.52.0 (exact) | VP-DAEMON-004/006 §Pre-conditions `tokio 1`; manifest row `tokio | 1.52` | PASS — `tokio 1` in VP Pre-conditions is a valid current-pointer to the tokio 1.x EXACT pin; no stale tokio 0.x or 1.51 reference found |
| `prost` | 0.14 (exact) | PRD §BC-PROTO-001a "prost 0.14 with an EXACT version pin"; manifest row `prost | 0.14` | PASS — single normative mention exactly matches manifest |
| `tempfile` | 3 (caret) | VP-DAEMON-005/006 §Pre-conditions `tempfile 3`; manifest row `tempfile | 3` | PASS |
| `tracing` | 0.1 (caret) | No versioned normative mentions in PRD/VP body; manifest row `tracing | 0.1` | PASS — no version cite needed; coding conventions reference tracing by name without pin annotation |
| `temp-env` | ^0.3 with `async_closure` feature (dev-dep) | VP-DAEMON-005 §Pre-conditions `temp-env ^0.3`; PRD BC-ENGINE-002-ERR precondition 2 `temp-env ^0.3 with features = ["async_closure"]`; manifest dev-dep row | PASS |
| `constant_time_eq` | 0.3 (caret) | PRD §8.4 `constant_time_eq::constant_time_eq (pinned ^0.3)`; manifest row `constant_time_eq | 0.3` | PASS |
| `serde_json` | =1.0.149 (exact) | VP §References `serde_json 1`; PRD §BC-RING-001 precondition 3 `serde_json::to_string`; manifest row `serde_json | 1.0.149` | PASS — `serde_json 1` is a valid current-pointer; no sub-series stale citation found |
| `serde_yaml_ng` | 0.10 (caret) | No versioned normative mentions; manifest row `serde_yaml_ng | 0.10` | PASS |
| `rand` | =0.8.6 (exact) | No versioned normative mentions in PRD/VP body; manifest row `rand | 0.8.6` | PASS |
| `notify` | 8 (caret) | No Phase 1 normative mentions (Phase 3 activation); manifest row `notify | 8` | PASS |
| `interprocess` | 2.4 (caret) | No versioned normative mentions in PRD/VP body; manifest row `interprocess | 2.4` | PASS |
| `crossterm` | 0.29 (caret) | No versioned normative mentions; manifest row `crossterm | 0.29` | PASS |
| `ratatui` | 0.30 (caret) | PRD §NFR-007 `ratatui 0.30 floor` (NFR table row); manifest row `ratatui | 0.30` | PASS |
| `reqwest` | 0.13 (exact) | No normative mentions in PRD/VP body; manifest row `reqwest | 0.13` | PASS |
| `russh` | 0.60 (exact) | No Phase 1 normative mentions (Phase 4 activation); manifest row `russh | 0.60` | PASS |
| `rmcp` | 1.6 (exact) | No Phase 1 normative mentions (Phase 4 activation); manifest row `rmcp | 1.6` | PASS |
| `nucleo` | 0.5 (caret) | No versioned normative mentions in PRD/VP body; manifest row `nucleo | 0.5` | PASS |
| `wasmtime` | 44 (exact) | No Phase 1 normative mentions; manifest row `wasmtime | 44` | PASS |
| `thiserror` | 2 (caret) | PRD §8.3 `thiserror 2.x`; manifest row `thiserror | 2` | PASS |
| `anyhow` | 1 (caret) | PRD §8.3 `anyhow 1`; manifest row `anyhow | 1` | PASS |
| `libc` | NOT in manifest (explicitly not chosen; nix 0.30 selected per F-R71-4b) | arch §Trace `libc 0.2` (historical-trace only); VP traces_to `libc 0.2` (PG-5 historical) | PASS — only in historical audit trail; zero normative body references to libc 0.2 as a pin |
| `bytes` | 1.10 (caret, prost-transitive-override) | manifest row `bytes | 1.10` | PASS — no normative VP/PRD body mentions required (workspace-level override, not an API surface) |

**Sweep result: 0 stale crate version pins in normative body across all 4 artifacts.**

---

## 3. Standard Cross-Artifact Coherence Checks

### Check 1: BC count (22)

| Artifact | Claimed | Actual | Status |
|----------|---------|--------|--------|
| PRD v1.7 | 22 BCs | 22 `### BC-` section headings (grep-confirmed) | PASS |
| VP v1.7 | 22 VPs | 22 VP rows in §VP Catalog Overview table | PASS |
| VP v1.7 §Coverage Matrix | 22 rows | 22 BC→VP rows (grep-confirmed) | PASS |
| STATE.md | 22 BCs implementable | 22 BCs in §Pre-Phase-1 Gate narrative | PASS |

### Check 2: Error code count (14)

| Artifact | Claimed | Actual | Status |
|----------|---------|--------|--------|
| PRD v1.7 §5 Error Taxonomy | 14 error codes | 14 `| E-` rows in error taxonomy table | PASS |
| PRD v1.7 §Trace v1.7 | "error-code count 14 unchanged" | Matches actual count | PASS |

### Check 3: Edge case count (59)

| Artifact | Claimed | Actual | Status |
|----------|---------|--------|--------|
| PRD v1.7 §9 Edge Cases | EC-001 through EC-059 | grep counts 59 `EC-[0-9]` lines in body section | PASS |
| PRD v1.7 §Trace v1.7 | "edge-case count 59 unchanged" | Matches | PASS |

### Check 4: Test name count (23)

| Artifact | Claimed | Actual | Status |
|----------|---------|--------|--------|
| PRD v1.7 BC sections | 23 distinct test names | 22 `- Test name:` annotations + 1 second test in BC-DAEMON-004 = 23 | PASS |
| VP v1.7 §Coverage Matrix | 23 `Test name:` entries | Confirmed in VP body | PASS |
| PRD v1.7 §Trace v1.7 | "test-name count 23 unchanged" | Matches | PASS |

### Check 5: BC↔VP 1:1 correspondence

All 22 BCs have exactly one primary VP. No BC has zero VPs. No VP lacks a BC source. Coverage matrix row count = 22 = VP Catalog Overview row count. PASS.

### Check 6: Arch version references in PRD normative body

All 10 BC sections referencing SS-daemon-lifecycle.md cite v1.0.13. All SS-core-types-and-abi.md references cite v1.2.8. All SS-engine-module.md references cite v1.1.15. No v1.0.12 or earlier arch ref found in normative PRD body. PASS.

### Check 7: Arch version references in VP §Coverage Matrix and §Per-VP traces

All 10 daemon-lifecycle BC rows in VP coverage matrix cite `SS-daemon-lifecycle.md v1.0.13`. All per-VP `Traces to:` headers cite `v1.0.13` for daemon-lifecycle BCs. PASS.

### Check 8: Manifest version references in VP Pre-conditions

VP-DAEMON-005 §Pre-conditions cites `SS-deps-pin-manifest.md v1.1.9` for both `directories 6` and `nix 0.30` pins. VP-DAEMON-004 references manifest correctly (axum/tokio only; tower disposition documented as transitive). PASS.

### Check 9: PRD RTM arch source column completeness

All 22 rows in §7 RTM have non-empty Architecture Source fields. All SS-daemon-lifecycle.md citations in RTM read v1.0.13. PASS.

### Check 10: VP §Coverage Matrix test file paths match PRD §7 RTM test file paths

Spot-checked 6/22 rows (BC-DAEMON-001/004/005, BC-AUTH-001/002, BC-ENGINE-002-ERR): VP matrix file paths = PRD RTM file paths verbatim. PASS.

### Check 11: STATE.md task queue consistency

STATE.md v5.8 correctly marks T-20 (R71) as COMPLETE FAIL → fix-burst applied, and T-21 (cons R10) as COMPLETE GAPS → fix-burst applied. T-22 (R72) and T-23 (cons R11) marked pending. `awaiting` field: "Adversary R72 + consistency-validator round 11 fresh-context re-review of PRD v1.7 + VP v1.7 + arch v1.0.13 + manifest v1.1.9." Current task (T-23) is consistency round 11. PASS.

### Check 12: Frontmatter version coherence

| Artifact | Frontmatter version | Consistent with body §Trace? |
|----------|--------------------|-----------------------------|
| PRD | `1.7` | §Trace v1.7 present | PASS |
| VP | `1.7` | §Trace v1.7 present | PASS |
| arch | `1.0.13` | §Trace v1.0.13 entry present | PASS |
| manifest | `1.1.9` | §Trace v1.1.9 entry present | PASS |
| STATE.md | `5.8` | phase-1-f-r71-fix-burst-complete-r72-pending | PASS |

### Check 13: BC-DAEMON-004 dual-test structure

PRD §BC-DAEMON-004 correctly documents two co-resident test names:
1. `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests` in `graceful_shutdown.rs`
2. `test_BC_DAEMON_004_exit_codes_posix_distinct` in `daemon_lifecycle.rs`

VP-DAEMON-004 §Test names lists both with matching file paths. PRD §7 RTM row for BC-DAEMON-004 lists both test files. PASS.

### Check 14: VP §Mechanism Distribution table arithmetic

| Mechanism | Primary count | Total VPs touched | Consistent? |
|-----------|--------------|-------------------|-------------|
| unit-test | 22 | 22 | PASS |
| fuzz | 0 primary | 5 auxiliary (VP-AUTH-001/002, VP-FACTORY-002, VP-PROTO-002, VP-DAEMON-003) | PASS |
| mutation-test | 0 primary | 4 auxiliary (VP-RING-001, VP-LOCK-001, VP-TYPES-001, VP-DAEMON-005) | PASS |
| Kani | 0 | 0 (deferred Phase 2+) | PASS |

### Check 15: No "pending architect review" or Principle 6 violations in normative body

Searched PRD v1.7, VP v1.7, arch v1.0.13 normative sections for: "pending", "TODO for architect", "OR libc", "OR tower". Found zero matches in normative body (all instances are in historical §Trace sections documenting closures). PASS.

### Check 16: NFR-008 cross-consistency

PRD §4 NFR-008 row: `macOS + Linux (darwin/linux × amd64/arm64)`. PRD BC-DAEMON-005 precondition 2 rationale: "macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64)". Arch §Scope: "NFR-008 lists macOS among the primary targets (`macOS + Linux`, darwin/linux × amd64/arm64)". All three sites use coequal framing. PASS.

---

## 4. Intra-Block Consistency (Extension 2)

The F-R71 content changes were confined to:
- VP-DAEMON-004 §Pre-conditions (tower phrasing)
- VP-DAEMON-005 §Pre-conditions (directories 6, nix 0.30 sole binding)
- VP-DAEMON-006 §Mechanical property item 1 (`<N>` placeholder)

For each modified VP, §Mechanism vs §Post-conditions vs §Probe-Table structural consistency was verified:

| VP | Change | §Mechanism unchanged? | §Post-conditions consistent? | §Probe-Table intact? |
|----|--------|----------------------|------------------------------|----------------------|
| VP-DAEMON-004 | §Pre-conditions tower phrasing | Yes | Yes (post-conditions reference axum/tokio, not tower) | Yes (5-code exit taxonomy intact) |
| VP-DAEMON-005 | §Pre-conditions directories 6 + nix sole binding | Yes | Yes (post-conditions probe matrix references `nix::sys::signal::kill` per sole binding) | Yes (4-path resolution matrix intact) |
| VP-DAEMON-006 | `"pid": <N>` placeholder in §Mechanical property | Yes | Yes (post-conditions use `<N>` convention throughout) | N/A (no probe table) |

**Intra-block consistency: 0 contradictions detected.**

---

## 5. Findings

**Total gaps: 0**

No blocking findings. No advisory findings. No routing actions required.

---

## 6. Gate Result

**GATE: PASS**

All F-R71 closures confirmed. All Extension 1/2/3 checks pass. Zero stale normative crate pins. Counts (22/14/59/23) consistent across artifacts. BC↔VP 1:1 holds. PRD v1.7 + VP v1.7 + arch v1.0.13 + manifest v1.1.9 are internally coherent and mutually consistent.

**Disposition:** Adversary R72 (D-047 strict pass 1 attempt 7) may proceed against the current artifact set.
