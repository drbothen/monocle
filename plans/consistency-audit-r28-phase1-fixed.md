---
document_type: consistency-report
level: ops
version: "1.0"
producer: consistency-validator
phase: pre-phase-1-final-gate
timestamp: 2026-05-15T23:50:00Z
traces_to: "prd.md v1.17 (27e663c); verification-properties.md v1.22 (e4c1a1e); SS-daemon-lifecycle.md v1.0.18 (61a0064); SS-deps-pin-manifest.md v1.1.12 (8005075)"
round: 28
pass: 1
attempt: 22
counter: 0/3
---

# Consistency Audit — Round 28, Phase 1, Pass 1, Attempt 22

## Summary

| Check Category | Status | Notes |
|---------------|--------|-------|
| F-R88-1 arch §Phase 4 Notes 7-field lock-file enumeration | PASS | |
| F-R88-2 PRD BC-DAEMON-005 Precondition 2(d) wording | PASS | |
| F-R88-2 VP-DAEMON-005 mirror item (d) wording | PASS | |
| F-R88-3 BC-RING-001 EC-001 serde annotation canonical form | PASS | |
| F-R88-4 EC-060 present in PRD §3 and §9; total = 60 | PASS | |
| F-R88-5 §Mechanism Distribution table (18 integration-test + 3 ast-audit + 1 compile-time-check) | PASS | |
| F-R88-5 §Coverage Matrix 22-row mechanism labels | PASS | |
| F-R88-5 §VP Catalog Overview 22-row mechanism labels (table) | PASS | |
| Cross-doc pin currency: PRD traces_to cites arch v1.0.18 (61a0064) | PASS | |
| Cross-doc pin currency: VP traces_to cites PRD v1.17 (27e663c) and arch v1.0.18 (61a0064) | PASS | |
| §Purpose META 9th-attempt: VP §Purpose cites PRD v1.17 commit 27e663c | PASS | |
| §References intro timestamp matches VP v1.22 frontmatter (2026-05-15T23:30:00Z) | PASS | |
| NFR-009 + NFR-012 sibling Validation Method VP citations (SE-15c) | PASS | |
| Extension 16 audit table: 39 rows (SE-16c canonical grep) | PASS | |
| EC count: §9 header says "EC-001 through EC-060"; body has 60 ECs | PASS | |
| PRD normative body (lines 28-1430) stale arch pin check | PASS | |
| VP normative body (lines 28-2530) stale arch pin check | PASS | |
| Git SHA verification (PRD 27e663c, VP e4c1a1e, arch 61a0064) | PASS | |
| **VP §Purpose prose "unit-test" label — NOT updated by F-R88-5** | **GAP** | **Line 43 body** |
| **VP §VP Catalog Overview intro "unit-test" prose — NOT updated by F-R88-5** | **GAP** | **Line 114 body** |

**Verdict: NOT CLEAN — 1 finding class (2 instances)**

Counter: 0/3 — does not advance (GAP found)

---

## Findings

### GAP-R28-001 — VP §Purpose + §VP Catalog Overview: Stale `unit-test` prose after F-R88-5 relabel

**Severity:** MEDIUM (internal consistency defect in the mechanism-taxonomy relabel)

**Documents affected:** `verification-properties.md` v1.22

**Finding:**

F-R88-5 relabeled all 22 VP `§Mechanism` cells and the `§Mechanism Distribution` table from `unit-test` to `integration-test` / `ast-audit` / `compile-time-check` per Rust cargo test-file-layout taxonomy. However, two prose locations that describe the mechanism types were NOT updated:

**Instance 1 — §Purpose, body line 43:**
```
The VP catalog is the input to Phase 6 (Formal Hardening). Every VP whose
mechanism is `unit-test` or `fuzz` is also a TDD target during Phase 3.
```
After F-R88-5, the correct term is `integration-test` (not `unit-test`). The TDD-target claim still applies to integration-test VPs but the label is stale.

**Instance 2 — §VP Catalog Overview intro, body line 114:**
```
Five VPs (VP-AUTH-001, VP-AUTH-002, VP-FACTORY-002, VP-PROTO-002 Phase-4-deferred, VP-DAEMON-003) admit auxiliary
fuzz harnesses in addition to their primary unit-test mechanism.
```
After F-R88-5, the primary mechanism for these five VPs is `integration-test`, not `unit-test`. The table rows in the same section correctly say `integration-test` (e.g., VP-AUTH-001 = `integration-test | fuzz`), creating an internal contradiction within §VP Catalog Overview.

**Evidence:**
- VP body line 43: `mechanism is \`unit-test\` or \`fuzz\``
- VP body line 114: `their primary unit-test mechanism`
- VP §Mechanism Distribution table (lines 146-154): `unit-test | 0 | 0 | 0` — zero primary mechanism VPs labeled unit-test
- VP §VP Catalog Overview table rows (lines 121-142): all 18 primary integration-test VPs correctly labeled `integration-test`
- F-R88-5 frontmatter traces_to: "§VP Catalog Overview Mechanism column updated (22 rows)" — confirms the table rows were swept but the intro prose was not

**Impact:** Internal document inconsistency. A developer reading §Purpose is told TDD targets use `unit-test` mechanism; the actual mechanism table shows `integration-test`. The §VP Catalog Overview intro contradicts its own table on the same screen. No behavioral contract semantics affected.

**Remediation:** Route to formal-verifier agent. Fix two sites:
1. VP §Purpose line 43: change `mechanism is \`unit-test\` or \`fuzz\`` to `mechanism is \`integration-test\` or \`fuzz\``
2. VP §VP Catalog Overview line 114: change `their primary \`unit-test\` mechanism` to `their primary \`integration-test\` mechanism`

SE-16c canonical grep `grep -nE "unit-test" .factory/specs/verification-properties.md | grep -v "§Trace\|0 | 0 | 0\|unit-test | 0"` should return zero normative-body hits post-fix.

---

## F-R88 Priority Check Results

### F-R88-1: arch §Phase 4 Notes 7-field lock-file enumeration

**STATUS: PASS**

SS-daemon-lifecycle.md v1.0.18 §Phase 4 Notes (lines 746-752) enumerates:
`"contract_version"` (forward-compatibility version sentinel, always FIRST key per BC-LOCK-001 Postcondition 2; Phase 4 readers MUST validate `contract_version == 1` before consuming other fields), `"pid"`, `"port"`, `"authToken"`, `"startTimeUtc"`, `"app"`, `"version"` = 7 fields.

Count: `contract_version` (1) + `pid` (2) + `port` (3) + `authToken` (4) + `startTimeUtc` (5) + `app` (6) + `version` (7) = 7 fields. `contract_version` is explicitly the FIRST key. PASS.

### F-R88-2: PRD BC-DAEMON-005 Precondition 2(d) wording

**STATUS: PASS**

PRD v1.17 line 326 (body): "If `MONOCLE_RUNTIME_DIR` is unset or empty AND `ProjectDirs::new("monocle", "monocle", "monocle")` returns `None` (no usable home directory; rare but possible in misconfigured containers), the daemon exits 1 with `DaemonStartError::RuntimeDirUnresolvable`. ... Note: paths (b) and (c) require a valid `ProjectDirs` instance; if `ProjectDirs::new()` succeeded, path (c) `data_local_dir()` always returns a valid path (never `None`) — the fail-fast only triggers when `ProjectDirs::new()` itself returned `None`."

The wording correctly names `ProjectDirs::new()` as the point of failure (not `data_local_dir()`). PASS.

### F-R88-2 mirror: VP-DAEMON-005 §Mechanical property item (d)

**STATUS: PASS**

VP v1.22 §VP-DAEMON-005 §Mechanical property item (d) (body lines 679-690) reads: "If `MONOCLE_RUNTIME_DIR` is unset or empty AND `ProjectDirs::new("monocle", "monocle", "monocle")` returns `None` ... Note: paths (b) and (c) require a valid `ProjectDirs` instance; if `ProjectDirs::new()` succeeded, path (c) `data_local_dir()` always returns a valid path (never `None`) — the fail-fast only triggers when `ProjectDirs::new()` itself returned `None`. No lock file created."

Mirror correctly matches PRD v1.17 line 326 form. PASS.

### F-R88-3: PRD BC-RING-001 EC-001 serde annotation

**STATUS: PASS**

PRD v1.17 EC-001 (body line 471): "omits these two fields entirely via the canonical `#[serde(skip_serializing_if = "Option::is_none")]` annotation on the `Option<String>` and `Option<serde_json::Value>` field types respectively. This is the NORMATIVE serialization form for BC-RING-001."

Canonical `#[serde(skip_serializing_if = "Option::is_none")]` annotation present. PASS.

### F-R88-4: EC-060 in PRD §3 and §9; total EC count = 60

**STATUS: PASS**

EC-060 found at:
- PRD body line 364 (BC-DAEMON-005 §Edge Cases): `EC-060: MONOCLE_RUNTIME_DIR="" (empty string...)`
- PRD body line 1398 (§9 catalog table): `| EC-060 | BC-DAEMON-005 | Empty MONOCLE_RUNTIME_DIR | ...`
- PRD §9 header (line 1336): "EC-001 through EC-060"

Physical count of `^EC-` lines in PRD = 60. Physical count of `^| EC-` catalog rows = 60. PASS.

### F-R88-5: §Mechanism Distribution table

**STATUS: PASS**

Table (body lines 146-154):
- integration-test | 18 | 0 | 18
- ast-audit | 3 | 0 | 3
- compile-time-check | 1 | 0 | 1
- unit-test | 0 | 0 | 0

Arithmetic: 18 + 3 + 1 = 22 primary ✓. Matches requirement exactly.

### F-R88-5: §Coverage Matrix 22-row mechanism labels

**STATUS: PASS**

All 22 rows verified:
- BC-DAEMON-001..006: `integration-test` ✓
- BC-RING-001, BC-AUTH-001/002, BC-LOCK-001: `integration-test` ✓
- BC-ABI-001: `integration-test` ✓
- BC-ABI-002: `compile-time-check` ✓
- BC-TYPES-001: `ast-audit` ✓
- BC-FACTORY-001: `ast-audit` ✓
- BC-FACTORY-002: `integration-test` ✓
- BC-PROTO-001a/001b/002: `integration-test` ✓
- BC-ENGINE-001: `ast-audit` ✓
- BC-ENGINE-002/002-ERR/003: `integration-test` ✓

Total by mechanism: integration-test = 17 table rows + 1 (structural recap) = 18; ast-audit = 3; compile-time-check = 1. Sum = 22 ✓.

**NOTE on §VP Catalog Overview intro prose: see GAP-R28-001.**

---

## Cross-Document Pin Currency

### PRD frontmatter traces_to — arch v1.0.18 citation

**STATUS: PASS**

PRD v1.17 frontmatter `traces_to` contains: `SS-daemon-lifecycle.md v1.0.18` and `SS-daemon-lifecycle.md v1.0.18 current-pointer (commit 61a0064)`.

### VP frontmatter traces_to — PRD v1.17 + arch v1.0.18 citations

**STATUS: PASS**

VP v1.22 frontmatter `traces_to` contains:
- `product-owner PRD v1.17 commit 27e663c` ✓
- `architect arch v1.0.18 commit 61a0064` ✓

### PRD normative body arch-pin currency (lines 28-1430)

**STATUS: PASS**

`awk 'NR>27 && NR<1430' prd.md | grep "v1\.0\.17"` = 0 hits. 31 `v1.0.18` references confirmed current. No stale v1.0.17 in normative body.

### VP normative body arch-pin currency (lines 28-2530)

**STATUS: PASS**

`awk 'NR>27 && NR<2530' verification-properties.md | grep "v1\.0\.17" | grep -v "→|historical|predecessor"` = 0 hits. All normative sites reference v1.0.18.

---

## §Purpose META Recurrence Guard

**STATUS: PASS**

VP §Purpose body lines 33-35:
```
This artifact authors formally-testable Verification Properties (VPs) against
the 22 Behavioral Contracts (BCs) formalized in the Phase 1 PRD v1.17 (commit
27e663c) and pre-staged across the Phase 1 architecture artifacts.
```
9th-attempt application confirmed. PRD v1.17 commit 27e663c present. PASS.

---

## §References Intro Timestamp

**STATUS: PASS**

VP §References intro (body line 2560): "All version pins below are current as of timestamp `2026-05-15T23:30:00Z`."

VP v1.22 frontmatter timestamp: `2026-05-15T23:30:00Z`. Match confirmed. PASS.

---

## NFR-009 + NFR-012 Sibling Validation Method Convention (SE-15c)

**STATUS: PASS**

PRD v1.17 §4 NFR table:
- NFR-009: "Integration test: `stat` lock file after daemon start; assert mode is `0600` per VP-DAEMON-005 Post-condition 1 (lock-file `0o600` mode assertion)" — cites VP probe ✓
- NFR-012: "Integration test: `stat` runtime_dir after daemon start; assert mode is `0700` per VP-DAEMON-005 Post-condition 9 / probe 5.e" — cites VP probe ✓

Sibling Security NFR rows both cite corresponding VP-DAEMON-005 post-conditions per SE-15c convention. PASS.

---

## Extension 16 Audit Table Row Count

**STATUS: PASS**

VP v1.22 §Trace v1.22 audit: "SE-16a in-burst-added citation audit: this burst introduces ZERO new cross-property / cross-check citations." "Extension 16 audit table regenerated via SE-16c canonical grep... 39 body grep matches expected (same as v1.21)."

Post-burst grep evidence in §Trace v1.22:
```
$ awk 'NR>=27 && NR<=2755' .factory/specs/verification-properties.md | grep -cE "[Cc]ross-property|[Cc]ross-check"
39
```

39 rows = 18 PRIMARY + 14 PROSE + 7 RECAP/CE. Arithmetic: 18 + 14 + 7 = 39 ✓. PASS.

---

## Git SHA Verification

| Artifact | Expected SHA | Actual SHA | Status |
|----------|-------------|-----------|--------|
| VP v1.22 | e4c1a1e | e4c1a1e | PASS |
| PRD v1.17 | 27e663c | 27e663c | PASS |
| arch v1.0.18 | 61a0064 | 61a0064 | PASS |
| manifest v1.1.12 | 8005075 | 8005075 | PASS |

All SHAs confirmed via `git log --oneline -8` in factory-artifacts branch.

---

## Verdict Summary

**VERDICT: NOT CLEAN — 1 finding class, 2 instances**

**Gap count: 1 (GAP-R28-001 — 2 instances, same root cause)**

**Counter: 0/3 — does not advance to 1/3**

### F-R88 Closure Verification Status

| Check ID | Status |
|----------|--------|
| F-R88-1 (arch §Phase 4 Notes 7 fields) | CLOSED ✓ |
| F-R88-2 (PRD Precondition 2(d) wording) | CLOSED ✓ |
| F-R88-2 mirror (VP item (d) wording) | CLOSED ✓ |
| F-R88-3 (EC-001 serde annotation) | CLOSED ✓ |
| F-R88-4 (EC-060 added; total = 60) | CLOSED ✓ |
| F-R88-5 §Mechanism Distribution table | CLOSED ✓ |
| F-R88-5 §Coverage Matrix 22-row labels | CLOSED ✓ |
| F-R88-5 §VP Catalog Overview TABLE rows | CLOSED ✓ |
| F-R88-5 §VP Catalog Overview INTRO PROSE | **NOT CLOSED — GAP-R28-001** |
| F-R88-5 §Purpose prose | **NOT CLOSED — GAP-R28-001** |

F-R88 is substantively closed on all normative contract and mechanism distribution surfaces. The remaining gap is a prose label drift in two non-normative description sentences that were not in the F-R88-5 sweep scope (the sweep targeted the Mechanism column of table rows, not surrounding prose).

### Required Action

Route GAP-R28-001 to **formal-verifier** agent (owns VP document). Fix the two prose sites in §Purpose and §VP Catalog Overview intro. No PRD or arch changes required. Commit as serial Step N+1 of the F-R88 chain; then re-run consistency audit attempt 23 to advance counter 0/3 → 1/3.
