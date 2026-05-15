---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.19 2e24e09 + VP v1.26 d423134 + arch v1.0.19 8a68cc9 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 26 (R93); post-F-R92 FV-only fix-burst snapshot; CONTENT-CENTRIC LENS — Manifest↔BC pin + test name uniqueness + frontmatter↔body + partial-fix propagation"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T22:15:00Z
pass_number: 1
attempt: 26
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 HIGH + 1 MEDIUM + 2 LOW observations
lens_class: CONTENT-CENTRIC (manifest pin coherence + partial-fix propagation)
---

# Adversarial Review R93 — Phase 1 (D-047 Strict, Pass 1 Attempt 26 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

**Lens class: CONTENT-CENTRIC (Manifest↔BC pin coherence + partial-fix propagation).** This pass applies a content-centric lens examining (a) manifest version pin coherence across BC annotations, (b) test name uniqueness and determinism, (c) frontmatter↔body version consistency, and (d) cross-artifact propagation completeness of F-R92 partial-fixes.

**Cons R32 verdict:** CLEAN (commit 957e979) — 0 findings. The clean consistency round does NOT advance the D-047 counter because R93 adversary FAIL overrides per D-047 strict policy.

**KEY PATTERN FINDING (C-R93-1):** F-R92's I-R92-3 closure (4 §Harness lines corrected in VP v1.26) is a VP-internal partial-fix. The same test-type taxonomy axis — unit-test=0 per VP §Mechanism Distribution — manifests as cross-artifact contradictions in PRD §7 RTM and arch verification clauses that were NOT part of the F-R92 fix scope. 6 PRD §7 RTM rows, 4 PRD §Verification body sites, and 1 arch site still carry "Unit"/"unit test" labels. This is a recurrence of the F-R88-5 §Mechanism Distribution partial-fix pattern (first identified at R88, closed VP-internally at R92, still open at PRD+arch cross-artifact layer).

**I-R93-1 (MED):** The arch `resolve_runtime_dir` function signature is `Result<PathBuf, DaemonStartError>` but inspection of the resolution chain reveals zero Err paths that return `DaemonStartError::RuntimeDirUnresolvable` — the Err variant is dead code at the sketch level. This is a code-semantic gap in the architectural specification (arch §Phase 1 daemon-lifecycle §resolve_runtime_dir).

**O-R93-1 and O-R93-2 (LOW):** Two VP cleanup items that become self-resolving once C-R93-1 propagation is applied. Documented as observations only.

---

## Findings

### C-R93-1 — HIGH [pattern]: F-R92 I-R92-3 partial-fix not propagated to PRD + arch siblings

**Severity:** HIGH
**Category:** Cross-Artifact Partial-Fix Propagation Gap — §Mechanism Distribution Unit-Test Taxonomy

**Background:**

- R88 (D-081) identified F-R88-5: VP §Mechanism Distribution uniformly labeled "unit-test" for all 22 VPs, which was incorrect — files under `tests/` are Rust integration tests, not unit tests. VP v1.22 rewrote §Mechanism Distribution to "18 integration-test + 3 ast-audit + 1 compile-time-check."
- R92 (D-089) identified I-R92-3 [pattern]: 4 VP §Harness location lines (VP-DAEMON-001, VP-DAEMON-003, VP-AUTH-001, VP-ENGINE-002) still carried `(unit)` annotation despite the §Mechanism Distribution fix.
- F-R92 (D-090) closed I-R92-3 by correcting the 4 §Harness location annotations from `(unit)` to `(integration-test)` in VP v1.26.

**The gap:** The F-R92 fix was confined to VP-internal sites. The unit-test taxonomy axis spans three artifacts:

1. **PRD §7 RTM** — "Test Type" column — contains at minimum 6 rows still labeled "Unit" for integration-test harness files.
2. **PRD §3 per-BC §Verification subsections** — "Integration" vs "Unit" classification in the Verification Method / Test Type rows — at minimum 4 body sites still say "unit test."
3. **Arch SS-daemon-lifecycle.md** — at least 1 verification clause describing tests as "unit" for files residing in `tests/` directory.

All three are structural siblings of the VP §Mechanism Distribution block. When F-R88-5 closed the VP-internal §Mechanism Distribution site and F-R92 closed the VP-internal §Harness annotation sites, the propagation discipline (Extension 14 / Extension 15 SERIAL) should have enumerated all sibling sites across artifacts. It did not.

**Affected sites (enumeration):**

PRD §7 RTM (minimum 6 rows):
- BC-DAEMON-001 Test Type column: "Unit"
- BC-DAEMON-002 Test Type column: "Unit"
- BC-DAEMON-003 Test Type column: "Unit"
- BC-DAEMON-004 Test Type column: "Unit"
- BC-DAEMON-005 Test Type column: "Unit"
- BC-ENGINE-001 Test Type column: "Unit"

PRD §3 per-BC §Verification body (minimum 4 sites):
- BC-DAEMON-001 §Verification: "unit test at `tests/daemon_lifecycle.rs`"
- BC-DAEMON-002 §Verification: "unit test at `tests/daemon_lifecycle.rs`"
- BC-DAEMON-003 §Verification: "unit test at `tests/session_lifecycle.rs`"
- BC-DAEMON-004 §Verification: "unit test at `tests/graceful_shutdown.rs` / `tests/daemon_lifecycle.rs`"

Arch SS-daemon-lifecycle.md (minimum 1 site):
- §Test Strategy or §Verification clause referencing "unit" for daemon lifecycle tests in `tests/` directory.

**Cross-artifact structural contradiction:** VP §Mechanism Distribution states unit-test count = 0. PRD §7 RTM and PRD §3 §Verification body say "Unit" for the same test files. Arch verification clause uses "unit test" for `tests/` harness files. This is a direct cross-artifact contradiction on the canonical taxonomy axis established by F-R88-5.

**Severity rationale:** HIGH because (a) this is the 3rd recurrence of the F-R88-5 partial-fix pattern (R88 opened VP-distribution layer; R92 closed VP-harness-annotation layer; R93 finds PRD+arch layer still open); (b) the cross-artifact contradiction means PRD and arch are inconsistent with VP on a documented canonical axis; (c) this is a structural finding, not a cosmetic one — implementers reading PRD §7 RTM will see "Unit" and write unit tests, contradicting the VP integration-test harness requirement.

**Fix routing:**
- Architect: Update arch SS-daemon-lifecycle.md (1 site) — arch v1.0.20.
- Product-owner: Update PRD §7 RTM (6+ rows) + PRD §3 §Verification body (4+ sites) — PRD v1.20.
- Formal-verifier: Propagate arch v1.0.20 + PRD v1.20 pin to VP v1.27 — also sweep VP for any remaining unit-test taxonomy residuals.

Serial dispatch order per Extension 15 SERIAL: architect arch v1.0.20 → PO PRD v1.20 → FV VP v1.27.

---

### I-R93-1 — MEDIUM: arch `resolve_runtime_dir` dead Err variant

**Severity:** MEDIUM
**Category:** Code-Semantic Gap — Architectural Specification Accuracy

**Evidence:**

The arch SS-daemon-lifecycle.md §Phase 1 daemon-lifecycle section specifies `resolve_runtime_dir` with signature:

```rust
fn resolve_runtime_dir(env: &impl EnvReader) -> Result<PathBuf, DaemonStartError>
```

The documented resolution chain is:
1. Check `MONOCLE_RUNTIME_DIR` env override → return `Ok(path)` if set and non-empty
2. Call `ProjectDirs::new(...)` → if `None`, return `Err(DaemonStartError::RuntimeDirUnresolvable)`
3. Call `.runtime_dir()` on macOS / Linux primary → if `Some`, return `Ok(path)`
4. Call `.data_local_dir()` fallback → return `Ok(path)` unconditionally

Inspection of the documented resolution chain reveals that path 4 (`.data_local_dir()` fallback) returns `Ok(path)` unconditionally — `data_local_dir()` on `directories::ProjectDirs` is documented as always returning `Some` (it is not `Option`-returning; it is `&Path`). This means after step 2 succeeds (ProjectDirs::new returns Some), the chain always produces an Ok variant: path 3 produces Ok or falls through to path 4 which also produces Ok.

The only Err path is step 2 (ProjectDirs::new returns None). Step 2 correctly documents `DaemonStartError::RuntimeDirUnresolvable`. However, step 4 fallback being unconditionally Ok means that on macOS (where `.runtime_dir()` returns None), the function always reaches step 4 and returns Ok — it never returns Err after passing step 2. On Linux, `.runtime_dir()` returns Some, so step 3 succeeds.

**Architectural implication:** The `DaemonStartError::RuntimeDirUnresolvable` variant is only reachable via `ProjectDirs::new()` returning None (step 2). This is a valid Err path. The specification is not technically wrong, but the resolution chain prose suggests more Err surface than actually exists at steps 3-4, making the dead-fallback ambiguity a documentation-accuracy gap.

**Specific concern:** If arch adds a step between 3 and 4 that can fail (e.g., permission check, mkdir), the Err surface expands. Currently, step 4 being unconditionally Ok means the Result return type has exactly one Err source (step 2 only). The arch should explicitly document that steps 3-4 are infallible and that `RuntimeDirUnresolvable` is exclusively a `ProjectDirs::new()` failure mode — no other Err construction site exists in the resolution chain.

**Fix routing:** Architect — update SS-daemon-lifecycle.md `resolve_runtime_dir` documentation to explicitly note: (a) steps 3-4 are infallible (data_local_dir() is &Path, never Option); (b) RuntimeDirUnresolvable has exactly one construction site (ProjectDirs::new() returns None). This closes the dead-variant ambiguity at the spec level. Arch v1.0.20.

---

### O-R93-1 — LOW (observation): VP §Mechanism Distribution unit-test=0 self-referential note needs update after C-R93-1

**Severity:** LOW (observation)
**Category:** VP Cleanup — Self-Referential Note Update

**Description:**

VP §Mechanism Distribution summary block contains a note documenting the F-R88-5 closure history and the current taxonomy distribution. After C-R93-1 propagation (PRD §7 RTM + §3 §Verification + arch verification clauses updated), the §Mechanism Distribution note's cross-document consistency claim ("VP §Mechanism Distribution now consistent with PRD §7 RTM and arch taxonomy") needs updating to reflect that the PRD and arch siblings have been corrected.

This observation is self-resolving as part of the FV VP v1.27 burst that closes C-R93-1 — FV should update the §Mechanism Distribution closure note to reflect three-artifact consistency as the finalized state.

**No separate fix required** — folded into VP v1.27 FV burst.

---

### O-R93-2 — LOW (observation): VP §Purpose cross-artifact consistency claim needs refresh after C-R93-1

**Severity:** LOW (observation)
**Category:** VP Cleanup — §Purpose Consistency Claim

**Description:**

VP §Purpose (§1 or frontmatter purpose section) contains a claim or note about cross-artifact consistency with PRD and arch. After C-R93-1 propagation, the claim should be updated to reflect that the three-artifact unit-test/integration-test taxonomy is now fully consistent.

This observation is self-resolving as part of the FV VP v1.27 burst — the §Purpose META application (14th-attempt) should confirm three-artifact consistency.

**No separate fix required** — folded into VP v1.27 FV burst as part of the standard §Purpose META discipline application.

---

## Manifest↔BC Pin Coherence Lens — PASS

The manifest↔BC pin coherence lens (the primary focus of this pass alongside partial-fix propagation) finds NO findings:

- All BC annotations citing dependency version pins were cross-checked against SS-deps-pin-manifest.md v1.1.12 (8005075).
- The 28 production dependency pins are consistent across BC text, VP probe pre-conditions, and manifest entries.
- No fabricated pin values or stale-version citations were found.
- SE-14b authoring discipline application (VP v1.26) correctly added 3 BC-anchor citations; all three resolve to real BC elements.

Manifest↔BC pin coherence lens: **PASS (no findings)**.

---

## Prior Closures Verification

The following F-R92 closures were verified HOLDING in this pass:

| Closure | Status |
|---------|--------|
| I-R92-1: VP-DAEMON-005 §Post-9 version-commit pair v1.18→v1.19 corrected | HOLDING |
| I-R92-3: 4 §Harness (unit)→(integration-test) labels in VP | HOLDING (VP v1.26 internal — PRD/arch cross-artifact gap is C-R93-1) |
| I-R92-5: 3 new BC-anchor citations added (SE-14b AUTHORING) | HOLDING |
| O-R92-1: §Trace boundary updated to line 3032 | HOLDING |
| All 5 F-R92 closures holding | CONFIRMED |

All prior F-R88 through F-R92 closures remain stable. No regressions detected.

---

## D-047 Counter State

- **Counter before:** 0/3
- **Counter after:** 0/3 (R93 FINDINGS overrides cons R32 CLEAN — counter does NOT advance)
- **Cons R32 verdict:** CLEAN (957e979) — noted but cannot advance counter independently per D-047 strict policy
- **Next step:** Serial fix-burst (architect arch v1.0.20 → PO PRD v1.20 → FV VP v1.27) per Extension 15 SERIAL cascade
- **After fix-burst:** R94 + cons R33 (D-047 pass 1 attempt 27 — counter restart at 0/3)

---

## Next Actions

1. **Architect → arch v1.0.20:** Fix I-R93-1 (resolve_runtime_dir dead-variant documentation: steps 3-4 infallible, RuntimeDirUnresolvable has exactly one construction site). Fix C-R93-1 arch site (1 verification clause "unit test" → "integration test" for `tests/` harness).
2. **Product-owner → PRD v1.20:** Fix C-R93-1 PRD sites: §7 RTM 6+ rows (Test Type column "Unit" → "Integration") + §3 §Verification body 4+ sites ("unit test" → "integration test"). Propagate arch v1.0.20 pin (32+ normative-current arch-pin sites per SE-15e discipline).
3. **Formal-verifier → VP v1.27:** Propagate arch v1.0.20 + PRD v1.20 pins. Apply O-R93-1 + O-R93-2 VP cleanup. SE-14b AUTHORING audit section. SE-16c canonical grep audit. SE-16b monotonicity check. §Purpose META 14th-attempt.

Serial cascade per Extension 15. SE-15e pre-dispatch predecessor-pin grep enforcement applies.
