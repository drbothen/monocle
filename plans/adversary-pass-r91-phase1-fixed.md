---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.18 3a18306 + VP v1.24 63b75f9 + arch v1.0.19 8a68cc9 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 24 (R91); post-F-R90 serial fix-burst snapshot; CONTENT-CENTRIC LENS — EC↔BC + lift_invariants_to_bcs"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T19:48:20Z
pass_number: 1
attempt: 24
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 CRITICAL + 6 HIGH + 1 MEDIUM + 4 LOW observations
lens_class: CONTENT-CENTRIC (EC↔BC coverage map + lift_invariants_to_bcs discipline)
---

# Adversarial Review R91 — Phase 1 (D-047 Strict, Pass 1 Attempt 24 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

**Lens class: CONTENT-CENTRIC (EC↔BC coverage map + lift_invariants_to_bcs discipline).** This pass applies the lift_invariants_to_bcs discipline in reverse — examining newly added VP §Post-condition probes (from the F-R90/R89/R88 fix-burst series) to verify that BC text has been correspondingly tightened to match the VP's constraints. Additionally applies an Anchor-Semantics audit (per BC-NNN §Postcondition/§Invariant cite resolution) and an EC↔BC coverage map.

**Cons R30 verdict:** CLEAN — 0 findings. The clean consistency round does NOT advance the D-047 counter because R91 adversary FAIL overrides per D-047 strict policy (the pair must BOTH be clean for the counter to advance for the adversary pass in the pair).

**KEY META INSIGHT:** The F-R90 fix-burst (which added VP §Post-condition probes I-R90-1/2/4 in VP v1.24) introduced VP-side tightening that exceeds the corresponding BC prose specificity. This is the lift_invariants_to_bcs discipline applied REVERSE: the VP probes now specify typed constraints (pid integer ≥ 1, shutdown_reason enum, last_app_mode non-empty string, semver regex) that the BC §Postconditions/§Invariants do NOT yet articulate. Four BC-text-lift sites (I-R91-3/4/5/6) + one fabricated BC anchor (C-R91-1) + one anchor pointing to wrong BC subsection (I-R91-2).

---

## Findings

### C-R91-1 — CRITICAL: VP-FACTORY-002 §Post-condition 7 cites fabricated BC-FACTORY-002 anchor

**Severity:** CRITICAL
**Category:** Semantic Anchoring Integrity — Fabricated Anchor

**Evidence:**

VP-FACTORY-002 §Post-condition 7 (added in VP v1.24 I-R90-4 closure) states:
```
7. After an idempotent re-initialization call with the same `cycle_id`,
   `current_cycle` remains equal to the prior value (no reset)
   (per BC-FACTORY-002 EC "idempotency invariant").
```

The anchor `BC-FACTORY-002 EC "idempotency invariant"` cites a non-existent element. Verification:

- `BC-FACTORY-002` exists in PRD §3 as the Factory State Adapter contract.
- `BC-FACTORY-002` does NOT have an EC titled or labeled "idempotency invariant."
- PRD §3 BC-FACTORY-002 edge-case enumeration: EC-028 (state-dir missing on re-init), EC-029 (concurrent write conflict), EC-030 (malformed cycle_id). None is labeled "idempotency invariant."
- The concept of idempotency is captured at BC-FACTORY-001 §Invariant level (cycle initialization is repeatable), NOT as a BC-FACTORY-002 EC.

The VP v1.24 burst introduced this anchor as newly fabricated. It does not resolve to a real BC element, violating Extension 12 (VP §Post-condition normative-tier narrative MUST anchor to a BC §Postcondition or §Invariant in PRD, not EC-only; and the cited element MUST exist).

**Impact:** C-R91-1 is CRITICAL per Extension 12 §Trace Audit mandate: fabricated anchor citations in VP §Post-conditions are the highest-severity form of anchor mis-cite because they are untraceable and unverifiable.

**Fix routing:** FV (VP v1.25 §Post-condition 7 — replace fabricated anchor with correct existing BC element or with a new BC §Invariant lift). PO may need to add a BC-FACTORY-002 §Invariant or §Postcondition for idempotency if none exists.

---

### I-R91-1 — HIGH: CLAUDE.md brief version branch sync issue

**Severity:** HIGH
**Category:** Artifact Currency — Branch Sync Gap

**Evidence:**

GAP-R29-001 fix applied CLAUDE.md brief version `v1.4.2 → v1.4.23` (D-086, state-manager scope, F-R90 fix-burst). The fix commit was 80ffdc2 (factory-artifacts branch).

Checking factory-artifacts live filesystem: CLAUDE.md lines 22 and 47 currently show `v1.4.23` on the working tree. However, the orchestrator note for this pass states "the adversary reading the file system sees v1.4.2 still — branch sync issue." The discrepancy may be resolved by the time this report is persisted. State-manager must confirm:

```
grep -n "v1\.4\." /Users/jmagady/Dev/monocle/CLAUDE.md
```

Expected post-fix state (factory-artifacts working tree):
- Line 22: `- Brief: \`v1.4.23\` at \`.factory/specs/product-brief.md\``
- Line 47: `6. \`.factory/specs/product-brief.md\` v1.4.23`

If the factory-artifacts working tree still shows `v1.4.2` at either site, state-manager must re-apply the fix and commit. If both sites show `v1.4.23`, I-R91-1 is closed.

**Fix routing:** State-manager — confirm and re-apply if needed.

---

### I-R91-2 — HIGH: VP-DAEMON-006 §Post-condition 10 anchor mis-cite (wrong BC subsection)

**Severity:** HIGH
**Category:** Semantic Anchoring Integrity — Wrong Subsection

**Evidence:**

VP-DAEMON-006 §Post-condition 10 (added in VP v1.24 I-R90-1 closure) states:
```
10. `shutdown_reason` field is one of the enumerated values:
    `GracefulShutdown | AdminShutdown | SignalTermination | Crash`
    (per BC-DAEMON-006 §Postcondition 1)
```

Verification against PRD §3 BC-DAEMON-006:
- BC-DAEMON-006 §Postcondition 1 is the **health-endpoint response** postcondition: "The `/status` GET response body matches the `DaemonStatusResponse` schema with all required fields present."
- BC-DAEMON-006 does NOT specify the `shutdown_reason` enum values in §Postcondition 1.
- The `shutdown_reason` enum is specified in BC-DAEMON-006 **§Invariant 1**: "The daemon maintains a monotonically valid `shutdown_reason: ShutdownReason` enum value throughout its lifecycle, initialized to `None` at startup and set exactly once at shutdown trigger."
- `GracefulShutdown | AdminShutdown | SignalTermination | Crash` is the enumeration from BC-DAEMON-006 §Invariant 1, not §Postcondition 1.

The VP cites the correct BC but the wrong subsection. Extension 12 requires precise BC element anchoring (§Postcondition vs §Invariant subsection).

**Fix routing:** FV (VP v1.25 §Post-condition 10 — change `§Postcondition 1` to `§Invariant 1`).

---

### I-R91-3 — HIGH: VP-DAEMON-006 §Post-condition 9 stronger than BC-DAEMON-006 §Postcondition/§Invariant text (pid ≥ 1 not in BC)

**Severity:** HIGH
**Category:** lift_invariants_to_bcs REVERSE — VP stronger than BC

**Evidence:**

VP-DAEMON-006 §Post-condition 9 (added in VP v1.24 I-R90-1 closure) states:
```
9. `pid` field is an integer ≥ 1 representing the daemon process ID
   (per BC-DAEMON-006 §Postcondition 2)
```

Verification against PRD §3 BC-DAEMON-006 §Postcondition 2:
- BC-DAEMON-006 §Postcondition 2 states: "The daemon PID is exposed via the `/status` endpoint `pid` field when the daemon is running."
- The BC text says PID is "exposed" and "represents the daemon process ID" — but does NOT specify the constraint `≥ 1`.

The VP probe introduces a typed constraint (`integer ≥ 1`) that the BC text does not articulate. Under SE-14b, this is a BC-text-lift required: BC-DAEMON-006 §Postcondition 2 must be extended to specify `pid: integer ≥ 1` to match the VP's precision.

**Fix routing:** PO (PRD BC-DAEMON-006 §Postcondition 2 — extend text to specify `pid integer ≥ 1`). FV (VP v1.25 — retain probe; update anchor after PO lift).

---

### I-R91-4 — HIGH: VP-DAEMON-006 §Post-condition 11 stronger than BC-DAEMON-006 §Postcondition text (last_app_mode non-empty not in BC)

**Severity:** HIGH
**Category:** lift_invariants_to_bcs REVERSE — VP stronger than BC

**Evidence:**

VP-DAEMON-006 §Post-condition 11 (added in VP v1.24 I-R90-1 closure) states:
```
11. `last_app_mode` field is a non-empty string when the daemon has processed
    at least one session start event (per BC-DAEMON-006 §Postcondition 3)
```

Verification against PRD §3 BC-DAEMON-006 §Postcondition 3:
- BC-DAEMON-006 §Postcondition 3 states: "The `last_app_mode` field reflects the most recent session's application mode string when at least one session has started."
- The BC text says "reflects the most recent session's application mode string" — it implies non-empty (since an app mode is a string), but does NOT explicitly specify "non-empty string" as a testable constraint.

The VP probe introduces a typed constraint (`non-empty string`) that the BC text does not explicitly assert. SE-14b requires BC-DAEMON-006 §Postcondition 3 to be extended to explicitly state "non-empty string when at least one session start event has been processed."

**Fix routing:** PO (PRD BC-DAEMON-006 §Postcondition 3 — add explicit "non-empty string" constraint). FV (VP v1.25 — retain probe).

---

### I-R91-5 — HIGH: VP-DAEMON-001 §Post-condition 7 semver regex constraint not in BC-DAEMON-001

**Severity:** HIGH
**Category:** lift_invariants_to_bcs REVERSE — VP stronger than BC

**Evidence:**

VP-DAEMON-001 §Post-condition 7 (corrected in VP v1.24 I-R90-2 closure from nonexistent post-condition reference) states:
```
7. `protocol_version` field conforms to semantic versioning regex
   `^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$`
   (per BC-DAEMON-001 §Invariant 2)
```

Verification against PRD §3 BC-DAEMON-001 §Invariant 2:
- BC-DAEMON-001 §Invariant 2 states: "The lock file's `protocol_version` field uses semantic versioning format."
- "Uses semantic versioning format" is a prose description — it does NOT include the specific regex `^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$`.

The VP probe anchors to BC-DAEMON-001 §Invariant 2 (correct subsection) but specifies a typed regex constraint that the BC text does not articulate. SE-14b requires BC-DAEMON-001 §Invariant 2 to be extended to include the specific semver regex pattern.

**Fix routing:** PO (PRD BC-DAEMON-001 §Invariant 2 — add semver regex). FV (VP v1.25 — retain probe).

---

### I-R91-6 — HIGH: VP-DAEMON-001 §Post-condition 8 (shutdown_reason enum values) not propagated to BC

**Severity:** HIGH
**Category:** lift_invariants_to_bcs REVERSE — VP stronger than BC

**Evidence:**

VP-DAEMON-001 §Post-condition 8 states (from VP v1.24):
```
8. `shutdown_reason` is one of: `null | "GracefulShutdown" | "AdminShutdown" |
   "SignalTermination" | "Crash"` (per BC-DAEMON-001 §Postcondition 3)
```

Verification against PRD §3 BC-DAEMON-001 §Postcondition 3:
- BC-DAEMON-001 §Postcondition 3 states: "On daemon stop, the lock file's `shutdown_reason` field is set to a valid shutdown reason string before the lock file is removed."
- The BC text says "valid shutdown reason string" — it does NOT enumerate the four canonical values `GracefulShutdown | AdminShutdown | SignalTermination | Crash`.

The VP probe specifies the typed enumeration that the BC text does not articulate. SE-14b requires BC-DAEMON-001 §Postcondition 3 to enumerate the four canonical `ShutdownReason` variants.

**Fix routing:** PO (PRD BC-DAEMON-001 §Postcondition 3 — add enum list). FV (VP v1.25 — retain probe).

---

### I-R91-7 — MEDIUM: EC-030 anchored to wrong BC (BC-FACTORY-001 vs BC-FACTORY-002)

**Severity:** MEDIUM
**Category:** EC↔BC coverage map — Wrong BC Anchor

**Evidence:**

PRD §9 edge-case catalog EC-030 is listed under BC-FACTORY-001 in the §9 grouping. EC-030 describes: "Malformed or missing `cycle_id` field in `FactoryStateAdapter::init()` call — returns `FactoryInitError::InvalidCycleId`."

Verification:
- `FactoryStateAdapter::init()` is the entry-point operation of BC-FACTORY-002 (which governs the initialization contract and error types for the adapter).
- BC-FACTORY-001 governs the FactoryStateAdapter READ contract (read-only state observation for the Workflow plane).
- EC-030 describes an ERROR from `init()` — a WRITE/initialization operation — which falls under BC-FACTORY-002's contract scope.

EC-030 is cataloged under the wrong BC. It should be associated with BC-FACTORY-002.

**Fix routing:** PO (PRD §9 EC-030 — re-anchor to BC-FACTORY-002).

---

## Process-Gap Observations (LOW)

### O-R91-1 — LOW: SE-14b codification candidate (per-probe BC-VP coherence)

R91 demonstrates that VP §Post-condition probes added in fix-bursts (v1.24 I-R90-1/2/4 closures) introduced typed constraints stronger than BC text. Extension 14 (lift_invariants_to_bcs within-layer propagation) captures the FORWARD direction (EC → BC → VP), but SE-14b is needed for the REVERSE discipline: newly added VP probes MUST trigger BC-text-lift to match VP precision.

SE-14b codification: per-probe BC-VP coherence discipline — when a VP introduces a §Post-condition probe that asserts a TYPED CONSTRAINT not specified in the corresponding BC text, the BC text MUST be lifted in the SAME burst or the immediately preceding PO burst per Extension 15 SERIAL protocol. The VP's per-VP `Traces to:` field MUST cite a BC element that explicitly covers the same constraint.

**Action:** State-manager to append SE-14b to lessons.md.

### O-R91-2 — LOW: Coverage map sweep discipline

The EC↔BC coverage map should be a required step in every fresh-context adversary pass. Current dispatch prompts enumerate specific lens targets but do not mandate a systematic EC→BC reverse-map. I-R91-7 (EC-030 wrong-BC anchor) was found by this sweep.

### O-R91-3 — LOW: Anchor Semantics Audit should be a per-pass mandatory sweep step

C-R91-1 (fabricated anchor) and I-R91-2 (wrong subsection) are both findable by a per-VP §Post-condition anchor audit: enumerate all `per BC-XXX §Postcondition N` or `per BC-XXX §Invariant N` cites, then grep each cited element in PRD to verify existence and correct subsection. This is a mechanical check that can be automated.

### O-R91-4 — LOW: CLAUDE.md branch sync advisory (I-R91-1 dependency)

The I-R91-1 finding notes a potential working-tree discrepancy between `main` branch (where the 42ec508 fix landed) and `factory-artifacts` (where GAP-R29-001 fix was applied at 80ffdc2). If the state-manager confirms both sites are at `v1.4.23` on factory-artifacts, I-R91-1 is informational. If not, a re-apply is needed.

---

## Convergence Assessment

**Counter:** 0/3 — FINDINGS present. Counter does NOT advance.

**Cons R30:** CLEAN — does not override adversary FAIL.

**D-047 strict policy:** Both the adversary pass AND the consistency audit for the same attempt pair must be clean for the counter to advance. R91 FAIL + cons R30 CLEAN = counter stays 0/3.

**Quality observation:** R91 reveals a new systematic defect class: VP-stronger-than-BC at the per-probe granularity. This is SE-14b territory. The F-R90 fix-burst (I-R90-1/2/4) added VP probes with higher precision than the BCs they cite. The fix-burst correctly added VP coverage but failed to simultaneously lift BC text to match.

**SE-14b codification is the primary process defense** against this class of finding. The pattern recurs at 4 sites (I-R91-3/4/5/6) plus 1 fabricated anchor (C-R91-1) plus 1 wrong-subsection anchor (I-R91-2). The systematic nature of the pattern (all from the same I-R90 fix-burst) confirms this is a protocol gap, not an isolated oversight.

**Serial fix-burst routing (per Extension 15):**

The I-R91-3/4/5/6 findings require PO BC-text lifts FIRST (PRD v1.19), then FV VP v1.25 (anchor re-verification post-lift + C-R91-1 anchor correction + I-R91-2 wrong-subsection fix). Per SE-14b (if codified), EC-030 wrong-BC anchor fix (I-R91-7) is PO-scope (PRD §9 re-grouping).

Serial order: PO (PRD v1.19: BC lifts + EC-030 re-anchor) → FV (VP v1.25: C-R91-1 fabricated anchor fix + I-R91-2 wrong-subsection fix + anchor re-verification against lifted BCs).

**27 codified disciplines in force** after SE-14b codification (was 26 + SE-14b).

---

## Artifact Versions Reviewed

| Artifact | Version | Commit |
|----------|---------|--------|
| PRD | v1.18 | 3a18306 |
| Verification Properties | v1.24 | 63b75f9 |
| Architecture (SS-daemon-lifecycle) | v1.0.19 | 8a68cc9 |
| Dependency Manifest | v1.1.12 | 8005075 |
| Consistency Round | R30 | bceb8f0 (CLEAN) |
