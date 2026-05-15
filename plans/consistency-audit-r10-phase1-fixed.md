---
document_type: consistency-audit
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.6 76570ac + VP v1.6 7ba155a + arch v1.0.12 727c826 + STATE.md v5.7 e456529; F-R70 closure chain applied"
level: ops
producer: consistency-validator
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T10:00:00Z
round: 10
---

# Consistency Audit — Round 10 (Phase 1 Post-F-R70)

**Verdict: GAPS**
**Gap count: 2**
**Severity: 1 MEDIUM + 1 LOW**

---

## Audit Scope

| Artifact | Version | Commit |
|----------|---------|--------|
| PRD | v1.6 | 76570ac |
| Verification Properties | v1.6 | 7ba155a |
| SS-daemon-lifecycle.md | v1.0.12 | 727c826 |
| STATE.md | v5.7 | e456529 |

Auditor: consistency-validator (fresh context, round 10)
Prior round: R9 (d8a61f2) — CLEAN

---

## Summary Table

| Check | Result | Notes |
|-------|--------|-------|
| BC-DAEMON-004 5-code POSIX exit taxonomy (PRD postcondition 8) | PASS | 0/130/143/2/1 consistent across PRD §3, VP §Post-condition 6, arch §Hard Shutdown |
| BC-DAEMON-004 exit taxonomy in arch BC summary row | PASS | Arch §BC Summary line enumerates all 5 codes |
| BC-DAEMON-004 test name PRD ↔ VP alignment | PASS | Both cite `test_BC_DAEMON_004_exit_codes_posix_distinct` |
| BC-DAEMON-004 test name PRD/VP ↔ arch body alignment | **GAP** | Arch body/§Trace cite `test_BC_DAEMON_004_exit_codes`; PRD/VP cite `test_BC_DAEMON_004_exit_codes_posix_distinct` |
| BC-DAEMON-005 4-path runtime-dir chain (PRD ↔ VP ↔ arch) | PASS | Paths a/b/c/d consistent across all three artifacts |
| BC-DAEMON-006 millisecond timestamp format (PRD invariant 1) | PASS | PRD BC-DAEMON-006 invariant 1 specifies `YYYY-MM-DDTHH:MM:SS.sssZ`; VP regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` matches |
| E-DAEMON-004 RuntimeDirUnresolvable propagation | PASS | PRD §5 error taxonomy row present; VP-DAEMON-005 coverage matrix updated |
| EC-057/058/059 anchored in PRD §3 and §9 | PASS | All three ECs appear in BC-DAEMON-005 §Edge Cases and §9 Edge Case Index |
| 14 error codes count | PASS | PRD §5 table row count = 14; PG-2 count coherence claim matches |
| 59 edge cases count | PASS | PRD §9 table row count = 59; PG-2 count coherence claim matches |
| BC count 22 unchanged | PASS | PRD, VP, arch all confirm 22 BCs |
| `directories` crate version pin PRD ↔ arch ↔ deps-manifest | PASS | PRD and arch do not cite a numeric version; deps-manifest = 6 (canonical) |
| `directories` crate version pin VP-DAEMON-005 ↔ deps-manifest | **GAP** | VP-DAEMON-005 §Pre-conditions cites `directories 5`; canonical pin is `directories 6` |
| STATE.md version markers (v1.6/v1.6/v1.0.12) | PASS | STATE.md §Phase 1 Entry inventory and §Blocking Issues both cite correct versions and commits |
| Previous closures intact (F-R65, F-R67, R3-001, R7-001) | PASS | No regression detected |
| D-047 strict convergence state | PASS | Counter reset to 0/3 after F-R70; STATE.md §Session Resume Checkpoint reflects this |

---

## Findings

### R10-001 — MEDIUM: Arch body and §Trace v1.0.12 use stale test name `test_BC_DAEMON_004_exit_codes`

**Location:** SS-daemon-lifecycle.md v1.0.12

- Line 635: `(test_BC_DAEMON_004_exit_codes) sends SIGTERM twice...`
- Line 770: `(test_BC_DAEMON_004_exit_codes in monocle-runtime/tests/daemon_lifecycle.rs)`

**Expected (canonical per PRD v1.6 and VP v1.6):** `test_BC_DAEMON_004_exit_codes_posix_distinct`

**Evidence:**
- PRD v1.6 line 306: `` `test_BC_DAEMON_004_exit_codes_posix_distinct` ``
- PRD v1.6 §Trace v1.6 line 1714: "New test added: `test_BC_DAEMON_004_exit_codes_posix_distinct`"
- VP v1.6 line 556: `test_BC_DAEMON_004_exit_codes_posix_distinct`
- VP v1.6 §Test name annotation (line 549 block): `test_BC_DAEMON_004_exit_codes_posix_distinct`

**Root cause:** The arch §Trace v1.0.12 was authored before PRD v1.6 adjudicated the canonical name with the `_posix_distinct` suffix (per PRD §Trace v1.6 change log). The arch §Trace at line 790 correctly notes "PRD is the canonical test-vector source" but the arch body was not back-propagated to reflect the name the PRD chose.

**Impact:** A test-writer following the arch body would produce a test named `test_BC_DAEMON_004_exit_codes`, which would diverge from the PRD/VP canonical name `test_BC_DAEMON_004_exit_codes_posix_distinct`. This is the exact drift class that produced F-R63-adv-1 (test name divergence). Per L-F-R63-PARTIAL-FIX Extension 2 discipline, test name canonicalization must propagate to ALL artifacts.

**Severity:** MEDIUM — same class as F-R63-adv-1 findings.

**Routing:** `vsdd-factory:architect` — arch body and §Trace are architect-owned. The fix is a two-site rename in SS-daemon-lifecycle.md (lines 635 and 770): `test_BC_DAEMON_004_exit_codes` → `test_BC_DAEMON_004_exit_codes_posix_distinct`.

---

### R10-002 — LOW: VP-DAEMON-005 cites `directories 5`; canonical pin is `directories 6`

**Location:** verification-properties.md v1.6

- Line 642: `- \`directories 5\` (or pinned equivalent) is the project pin for`
- Line 2197: `§Pre-conditions expanded to include the \`directories 5\` pin`

**Expected (canonical):** `directories 6`

**Evidence:**
- SS-deps-pin-manifest.md line 48: `| directories | 6 | XDG-compliant config/data/runtime dirs | caret pin ...`
- CLAUDE.md §Key Tech Stack: `directories 6`

**Root cause:** The VP-DAEMON-005 §Pre-conditions were written during the v1.5 cycle using `directories 5`. The dep pin was updated to `directories 6` in SS-deps-pin-manifest.md but was not propagated to the VP's inline pre-condition citation. The `(or pinned equivalent)` qualifier provides partial coverage but an explicit stale version number misleads the implementer.

**Impact:** A test-writer following VP-DAEMON-005 §Pre-conditions verbatim would pin `directories 5`, conflicting with the workspace's canonical `directories 6` pin and potentially introducing a version conflict.

**Severity:** LOW — the qualifier `(or pinned equivalent)` mitigates runtime risk but the citation is stale and misleading.

**Routing:** `vsdd-factory:formal-verifier` — VP file is formal-verifier owned. Fix: update lines 642 and 2197 from `directories 5` to `directories 6`.

---

## Checks Not Applicable This Round

- UI quality checks (D1/D3/D10/D12/D16): not a UI product in Phase 1 scope.
- Sharding integrity (criteria 21-23): single-file artifacts, no shard directories for these artifacts.
- Holdout scenario alignment: Phase 4 scope, not yet authored.

---

## Previous Closures — Integrity Verification

| Closure | Status |
|---------|--------|
| F-R65-1/2/3 (arch BC-AUTH-002 Three→Two + Bearer fix) | INTACT — no regression |
| F-R67-1 (VP-TYPES-001 §Mechanism prose intra-block fix) | INTACT — no regression |
| F-R67-2 (PRD EC-045 off-by-one 262,144→262,145) | INTACT — no regression |
| R3-001 (version-stable §BC Summary footer, Pattern B) | INTACT — no regression |
| R7-001 (VP-DAEMON-001 single-line PRD v1.4→v1.5 citation) | INTACT — no regression |
| F-R70-1 (macOS runtime_dir fallback chain, arch+PRD+VP) | INTACT — arch §Start Sequence step 1 four-path chain present; PRD BC-DAEMON-005 precondition 2 enumerates (a)/(b)/(c)/(d); VP-DAEMON-005 §Mechanical property item 0 + probe matrix rows 5.a/b/c/d all present |
| F-R70-2 (BC-DAEMON-006 millisecond timestamp tightening) | INTACT — PRD invariant 1 specifies `YYYY-MM-DDTHH:MM:SS.sssZ`; VP regex matches |
| F-R70-3 (POSIX exit codes 0/130/143/2/1, arch+PRD+VP) | INTACT — all five codes enumerated consistently |
| Obs-R70-1 (EC-031 fail-open security rationale) | INTACT — present in PRD BC-DAEMON-001 §Authentication |
| Obs-R70-2 (VP-DAEMON-004 over-budget exit tolerance retired) | INTACT — VP-DAEMON-004 §Post-condition 6 now uses 5-row probe matrix with deterministic per-cause assertions |

---

## Routing Recommendations

| Finding | Routed To | Fix Scope |
|---------|-----------|-----------|
| R10-001 (MEDIUM) | `vsdd-factory:architect` | SS-daemon-lifecycle.md: rename `test_BC_DAEMON_004_exit_codes` → `test_BC_DAEMON_004_exit_codes_posix_distinct` at lines 635 and 770 |
| R10-002 (LOW) | `vsdd-factory:formal-verifier` | verification-properties.md: update `directories 5` → `directories 6` at lines 642 and 2197 |

Both fixes require a single-commit F-R10 burst with PG-3/PG-4 propagation sweep per L-F-R63-PARTIAL-FIX Extension 2 discipline. D-047 strict counter resets to 0/3 on any content change.

---

## PG-2 Count Coherence (Audit Re-verification)

| Metric | PRD claim | Verified count | Match |
|--------|-----------|----------------|-------|
| BC count | 22 | 22 | YES |
| Error code count | 14 | 14 | YES |
| Edge case count | 59 | 59 | YES |
| Test name count | 23 | 23 (confirmed PRD §Trace + VP §References) | YES |

---

## Gate Result

**GAPS — 2 findings (1 MEDIUM + 1 LOW)**

D-047 strict requires 0 findings of any severity. Both findings must be closed before the convergence counter can advance. R71 adversary pass (T-20) may run concurrently with the fix burst; however the D-047 counter does not advance until BOTH this audit round and the adversary round return 0 findings.
