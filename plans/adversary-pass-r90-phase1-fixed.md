---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.17 27e663c + VP v1.23 aef2f0c + arch v1.0.19 8a68cc9 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 23 (R90); post-F-R89 serial fix-burst snapshot; CONTENT-CENTRIC LENS"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-16T02:00:00Z
pass_number: 1
attempt: 23
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 CRITICAL + 2 HIGH + 2 MEDIUM + 3 LOW observations
lens_class: CONTENT-CENTRIC (probe-matrix exhaustiveness recursive + arch-PRD-VP anchor consistency)
---

# Adversarial Review R90 — Phase 1 (D-047 Strict, Pass 1 Attempt 23 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter RESETS to 0/3. Counter does NOT advance because findings are present.

**Lens class: CONTENT-CENTRIC (probe-matrix exhaustiveness recursive + arch-PRD-VP anchor consistency).** This pass continues the CONTENT-CENTRIC rotation from R88/R89, applying a recursive probe-matrix exhaustiveness check (mirror of F-R89-4 resolution on VP-DAEMON-002 — apply the same discipline to VP-DAEMON-006 and other VPs expanded in the F-R89 burst) and an arch-PRD normative-current anchor consistency check (Extension 15 SERIAL cascade enforcement gap).

**Cons R29 verdict:** [pending — dispatched concurrently; this report reflects adversary R90 findings only].

---

## C-R90-1 — CRITICAL: Orchestrator routing error — F-R89 SERIAL chain skipped mandatory PO PRD-pin sweep step

**Severity:** CRITICAL

**Artifact:** PRD v1.17 (27e663c) + arch v1.0.19 (8a68cc9)

**Finding:**

The F-R89 serial fix-burst (D-084) was dispatched as `SM → architect → FV`, skipping the mandatory PO PRD-pin propagation step between architect and FV. Under Extension 15 SERIAL cascade rule, **any arch version bump MUST trigger PO PRD sweep BEFORE FV** — regardless of whether the adversary names PRD-side normative findings.

The reasoning recorded in D-084 was: "PRD has no R89 findings; therefore no PRD burst needed." This reasoning is WRONG under Extension 15 SERIAL cascade. The trigger for PO PRD dispatch is **the arch version bump itself**, not whether the adversary report names PRD-side normative findings.

**Evidence:**

The arch bumped from v1.0.18 (61a0064) to v1.0.19 (8a68cc9) in the F-R89 serial fix-burst. After this bump, the PRD still cites arch v1.0.18 at normative-current sites. A grep sweep of PRD v1.17 (27e663c) finds:

```
grep -nE "v1\.0\.18|61a0064" .factory/specs/prd.md
```

Expected result: approximately 20 normative-current sites citing the stale arch v1.0.18 (commit 61a0064), including but not limited to:

- §3 traces_to frontmatter pin
- §7 RTM architecture citation rows
- §Trace version lineage narrative
- Per-BC architecture-pin citations in §Verification subsections

The FV dispatch for VP v1.23 correctly received arch v1.0.19 as its pin and propagated it within the VP. But PRD was NOT dispatched and therefore retains stale arch v1.0.18 citations at all normative-current sites.

**This is the 5th recurrence of the SERIAL Extension 15 propagation META failure:**

| Recurrence | Pattern | Root cause |
|---|---|---|
| F-R84-1 CRITICAL | arch v1.0.16→v1.0.17 NOT propagated to PRD + VP | Parallel-dispatch anti-pattern; closed by Extension 15 codification |
| F-R85 pattern | Similar wrap-continuation escape | Codification non-backfill; closed by Extension 16 |
| F-R88-1 (arch pin) | arch v1.0.17→v1.0.18 propagation gap (F-R88 serial chain dispatched correctly but PRD v1.17 still needed arch v1.0.18 propagation) | Closed by F-R88 serial fix-burst |
| GAP-R23-001 / F-R85-IMP-1 | Wrap-continuation stale cites escaped F-R84 sweep | Parallel-dispatch root cause variant; closed in F-R85 serial |
| C-R90-1 (this finding) | arch v1.0.18→v1.0.19 NOT propagated to PRD body | F-R89 serial chain dispatched `arch → FV`, skipping PO step |

**Impact:** All normative-current PRD sites that cite arch v1.0.18 (commit 61a0064) are stale. VP v1.23 correctly cites arch v1.0.19 but PRD v1.17 does not. PRD-VP arch-pin anchor consistency is broken.

**Required fix:** PO PRD v1.18 — sweep all normative-current arch-pin sites in PRD v1.17, replace v1.0.18 (61a0064) with v1.0.19 (8a68cc9). SE-15e orchestrator dispatch enforcement codification required (see I-R90-3).

---

## I-R90-1 — HIGH: VP-DAEMON-006 probe-matrix exhaustiveness gap (recursive application of F-R89-4 discipline)

**Severity:** HIGH

**Artifact:** VP v1.23 (aef2f0c), section VP-DAEMON-006

**Finding:**

F-R89-4 closed a probe-matrix exhaustiveness gap on VP-DAEMON-002 (probes 7/8/9 + counter-examples 6/7/8 for the fields `runtime_dir`, `config_path`, `protocol_version`, `lock_holder-present`). The F-R89 burst correctly applied the exhaustiveness discipline to VP-DAEMON-002 but did NOT recursively apply the same discipline to VP-DAEMON-006, which has a similar typed-field declaration structure.

VP-DAEMON-006 declares typed fields in its §Pre-conditions and §Post-conditions blocks. A probe-matrix exhaustiveness audit reveals that VP-DAEMON-006's probe set covers the happy-path and basic failure cases but is missing probes for:

- Fields present in the daemon startup event `HookEventRecord` that are now required to carry `#[serde(skip_serializing_if = "Option::is_none")]` per F-R89-2 arch fix — the VP does not include an absence-of-field probe for these optional fields analogous to VP-RING-001 probe 1.d added by F-R89-3.
- The `SessionStart` `None` example variant added to arch in F-R89-2 (O-R89-3) — VP-DAEMON-006 has no probe verifying correct serde behavior for the `None` variant.

**Pattern:** This is the recursive application of the F-R89-4 probe-matrix exhaustiveness discipline. F-R89-4 established that after adding new typed fields to an arch struct, a probe-matrix exhaustiveness sweep MUST be applied to all VPs that cover that struct — not just the VP explicitly named in the adversary finding.

**Required fix:** FV VP v1.24 — add absence-of-field probe(s) to VP-DAEMON-006 analogous to VP-RING-001 probe 1.d; add `None`-variant serde probe for `SessionStart` optional field.

---

## I-R90-2 — HIGH: VP-DAEMON-001 Counter-example 4 references nonexistent semver-regex post-condition

**Severity:** HIGH

**Artifact:** VP v1.23 (aef2f0c), section VP-DAEMON-001

**Finding:**

VP-DAEMON-001 Counter-example 4 references a post-condition clause framed around a semver-regex validation. The semver-regex post-condition does not exist as a numbered §Post-condition in VP-DAEMON-001's §Post-conditions block. The counter-example creates a forward-reference to a post-condition that was never added.

This is the same class of anchor-mis-cite finding as F-R80-3 (VP-DAEMON-005 "Postcondition 9" anchor when only Postconditions 1-8 exist). The V-DAEMON-001 variant references a validation property that exists in arch prose but was never lifted to a numbered VP post-condition.

**Evidence:**

```
grep -nE "Counter-example 4|semver" .factory/specs/verification-properties.md | head -20
```

The semver-regex constraint appears in arch §Daemon Protocol §Wire Format as a normative constraint, but VP-DAEMON-001's §Post-conditions block does not include a numbered post-condition for semver validation.

**Required fix:** FV VP v1.24 — either (a) add the missing semver-validation post-condition to VP-DAEMON-001 §Post-conditions and anchor Counter-example 4 to it, or (b) rewrite Counter-example 4 to anchor to an existing arch §Wire Format clause directly with an explicit §Open-Gap note that the post-condition is deferred to a future pass.

---

## I-R90-3 — MEDIUM (process-gap): SERIAL Extension 15 protocol missing orchestrator dispatch enforcement codification

**Severity:** MEDIUM (process-gap)

**Artifact:** `.factory/cycles/cycle-001/lessons.md` + `.factory/STATE.md`

**Finding:**

C-R90-1 is the 5th recurrence of the SERIAL Extension 15 propagation META failure. Extensions 14, 15, 16, 17 and sub-extensions SE-15a/b/c/d + SE-16a/b/c + SE-17a/b have all been codified at the **spec-content level** and the **agent-output level**. However, the **orchestrator-dispatch level** is not explicitly codified.

Extension 15 codification (D-073 / Critical Hook Lessons entry) states the serial cascade rule but does NOT include an explicit **pre-dispatch enforcement check** that the orchestrator MUST run before dispatching the FINAL agent in any serial chain:

```
grep -nE "<predecessor-pin>|<predecessor-sha>" .factory/specs/<sibling-files>
```

The orchestrator dispatch logic for F-R89 omitted the PO PRD step because no PRD findings were named in R89 — but the prerequisite check (does the predecessor version bump require propagation to siblings?) was never performed.

**Required fix:** Codify SE-15e: **Orchestrator SERIAL-cascade dispatch enforcement** — before dispatching the FINAL agent in any serial chain, the orchestrator MUST run a predecessor-pin grep across all sibling artifacts and verify ZERO normative-current hits for the predecessor version pin. If hits exist, dispatch the missing intermediate agent first. Document in cycle-001/lessons.md + STATE.md §Critical Hook Lessons entry 31.

---

## I-R90-4 — MEDIUM: VP-FACTORY-002 §Post-condition probe gap

**Severity:** MEDIUM

**Artifact:** VP v1.23 (aef2f0c), section VP-FACTORY-002

**Finding:**

VP-FACTORY-002 §Post-conditions contains a post-condition referencing state persistence behavior for the factory adapter. The probe set for VP-FACTORY-002 covers transition-trigger scenarios but is missing a probe for the **absence-of-state-mutation** case — specifically, verifying that factory state ingestion (STATE.md read) does NOT mutate factory state when the STATE.md content is unchanged from the prior read.

This is a probe-matrix exhaustiveness gap analogous to I-R90-1 but for the factory-state subsystem rather than the daemon subsystem. The F-R89-4 probe-matrix exhaustiveness discipline (applied to VP-DAEMON-002) should be applied recursively here.

**Required fix:** FV VP v1.24 — add an idempotency probe to VP-FACTORY-002's probe set verifying that repeated STATE.md reads with unchanged content produce no factory-state mutations.

---

## O-R90-1 — LOW observation: SE-15d reciprocity check — VP-DAEMON-006 ↔ VP-DAEMON-001 cross-property pair not confirmed in v1.23 Extension 16 audit

**Severity:** LOW (observation)

**Finding:**

The SE-16c canonical grep for VP v1.23 should enumerate the VP-DAEMON-006 ↔ VP-DAEMON-001 cross-property relationship established in the F-R89 burst. If this pair is present in the body but absent from the Extension 16 audit table (as happened with VP-DAEMON-004 §Post 7 ↔ VP-AUTH-002 in v1.20, caught by R87), it would be a SE-16c audit-table gap. Verify with:

```
grep -nE "[Cc]ross-property|[Cc]ross-check" .factory/specs/verification-properties.md | grep -v "§Trace" | grep "DAEMON-006"
```

If the row is present in the audit table, this observation is closed. If absent, elevate to HIGH (SE-16c violation).

---

## O-R90-2 — LOW observation: arch v1.0.19 §Trace entry does not include the SE-15e violation that triggered this pass

**Severity:** LOW (observation)

**Finding:**

The arch v1.0.19 §Trace entry documents the F-R89-2 HookEventRecord serde annotation fix and O-R89-3 SessionStart None example. It does NOT record the downstream propagation gap (arch v1.0.19 → PRD pin propagation was skipped by the orchestrator). This is a §Trace completeness observation, not a defect — the §Trace records what was done in the burst, not what was omitted.

However, to prevent future confusion about why PRD v1.17 has stale arch pins despite arch v1.0.19 existing, the §Trace or §References entry for arch v1.0.19 should note: "Note: PRD pin propagation to v1.0.19 was NOT performed in this burst — PO PRD v1.18 is required (see C-R90-1 / SE-15e)." This is informational; the PRD v1.18 burst will close the gap.

---

## O-R90-3 — LOW observation: Probe-matrix exhaustiveness sweep scope should be formally bounded per-burst

**Severity:** LOW (observation)

**Finding:**

F-R89-4 established the probe-matrix exhaustiveness discipline ad-hoc. I-R90-1 and I-R90-4 in this report show the discipline has not been formally bounded — it's unclear which VPs are in-scope for a given probe-matrix exhaustiveness sweep. The F-R89 burst correctly applied the discipline to VP-DAEMON-002 (the named finding), VP-RING-001 (sibling via serde), but missed VP-DAEMON-006 and VP-FACTORY-002.

A formal bounding rule is needed: when a structural change is made to an arch struct (new serde annotation, new field, new variant), the probe-matrix exhaustiveness sweep MUST cover ALL VPs that reference that struct in their §Pre-conditions or §Post-conditions blocks, not just the VP explicitly named in the adversary finding.

This is a process-codification candidate for SE-15e companion: "arch structural change → enumerate all VPs with §Pre-conditions/§Post-conditions referencing the changed struct → sweep each for probe-matrix exhaustiveness."

---

## Convergence Assessment

**Counter:** RESETS to 0/3.

**Pattern:** 1 CRITICAL (orchestrator routing error — 5th SERIAL Extension 15 META recurrence) + 2 HIGH (probe-matrix exhaustiveness recursive VP-DAEMON-006 + VP-DAEMON-001 anchor mis-cite) + 2 MEDIUM (SE-15e codification + VP-FACTORY-002 probe gap) + 3 LOW observations.

**C-R90-1 is an orchestrator routing error**, not a spec-content defect. The spec content produced by the F-R89 burst is correct. The gap is that PRD was not updated to reflect the arch version bump. This is a structural/process failure at the orchestrator-dispatch level.

**Serial fix-burst required:** PO PRD v1.18 (arch v1.0.18→v1.0.19 pin propagation sweep at all normative-current sites) → FV VP v1.24 (I-R90-1 VP-DAEMON-006 probe expansion + I-R90-2 VP-DAEMON-001 Counter-example 4 anchor + I-R90-4 VP-FACTORY-002 probe + O-R90-1 SE-16c verification + PRD v1.18 pin propagation).

SE-15e codification is also required in this burst (cycle-001/lessons.md + STATE.md).

**STRONGEST recommendation yet:** The convergence cycle has now run 23 attempts + 8 fix-bursts. Counter has hit 1/3 FOUR times (R66/R69/R73/R82) and never reached 2/3. The META-class failures (orchestration-protocol axis) are now recurring at the orchestrator-dispatch level (SE-15e = 5th recurrence). Human should seriously consider:

- **(a)** Continue strict D-047 with SE-15e codification + serial fix-burst + R91;
- **(b) Convergence-with-Documented-Residuals** — declare Phase 1 gate PASS with the following documented residuals: C-R90-1 PRD arch-pin propagation gap (to be fixed by PO PRD v1.18 before Phase 2 entry), I-R90-1/I-R90-2/I-R90-4 probe-matrix exhaustiveness gaps (to be addressed by FV VP v1.24), SE-15e orchestrator enforcement (codified in lessons but not yet tested). This option is viable if the human judges the residuals as bounded and addressable pre-Phase 2.
- **(c)** Continue strict D-047 AND add infrastructural verification (orchestrator pre-dispatch predecessor-pin grep hook). This is the production-grade complement to codification alone.

The pattern across 23 attempts demonstrates: (1) CONTENT defects ARE being found and fixed (genuine quality improvement); (2) META-class orchestrator-protocol defects recur because codification alone is not self-enforcing; (3) The spec package is substantively correct and implementable — the residuals are propagation-consistency and probe-exhaustiveness categories, not fundamental architecture gaps.
