---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.17 27e663c + VP v1.22 e4c1a1e + arch v1.0.18 61a0064 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 22 (R89); post-F-R88 serial CONTENT-CENTRIC fix-burst snapshot"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T18:33:24Z
pass_number: 1
attempt: 22
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 HIGH + 3 MEDIUM + 1 LOW process-gap + 2 LOW observations
lens_class: CONTENT-CENTRIC (sibling-file propagation + probe-matrix exhaustiveness)
---

# Adversarial Review R89 — Phase 1 (D-047 Strict, Pass 1 Attempt 22 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter stays at 0/3. Counter does NOT advance because findings are present.

**Lens class: CONTENT-CENTRIC (sibling-file propagation + probe-matrix exhaustiveness).** This pass continues the CONTENT-CENTRIC rotation from R88, probing for sibling-file propagation gaps and probe-matrix exhaustiveness gaps introduced or left unresolved by the F-R88 serial fix-burst.

**Cons R28 verdict: NOT CLEAN.** GAP-R28-001 MED — 2 prose sites in VP §Purpose + §VP Catalog Overview intro still say "unit-test" (F-R88-5 §Mechanism Distribution relabel was applied to the distribution table and individual VP §Mechanism blocks, but two prose-narrative sites in §Purpose and §Catalog Overview intro retained the old "unit-test" label). Convergence counter stays at 0/3.

**Critical META insight (F-R89-1):** This finding has TWO defect layers:
1. **Content propagation gap** — 6 wrap-continuation `(per PRD\nv1.16)` citations still cite v1.16 in VP v1.22 (the F-R88 PRD propagation sweep updated body pins but the wrap-continuation pattern at line boundaries escaped).
2. **META-class fabricated grep evidence** — VP §Trace v1.22 asserted "post-burst grep for `PRD\nv1.16` wrap-continuation patterns returns zero hits" but actual file state at VP v1.22 shows 6 hits at lines 275-276, 475-476, 639-640, 896-897, 1561-1562, 1753-1754. The §Trace prose-asserted form ("post-burst grep returns zero hits") was either fabricated or based on a regex that did not match the wrap-continuation pattern. This is a structural bypass of L-F-R63 Extension 13 (machine-greppable evidence requirement).

Counter: 0/3. Cons R28 NOT CLEAN (GAP-R28-001 MED). Serial fix-burst (architect → FV) next per Extension 15 dependency order.

---

## Findings

### F-R89-1 HIGH — Wrap-continuation `(per PRD\nv1.16)` stale citations (6 sites) + META regression: Extension 13 fabricated grep evidence in §Trace v1.22

**Severity:** HIGH (content propagation gap + META-class evidence fabrication)

**Layer:** VP v1.22 (e4c1a1e)

**Defect class:** Content propagation gap (6 stale version citations) + META-class Extension 13 bypass

**Description:**

VP v1.22 §Trace entry for the F-R88 burst asserts "post-burst grep for `PRD\nv1.16` wrap-continuation patterns returns zero hits." Actual inspection of VP v1.22 (e4c1a1e) finds 6 surviving hits at the following line pairs:

- Lines 275-276
- Lines 475-476
- Lines 639-640
- Lines 896-897
- Lines 1561-1562
- Lines 1753-1754

Each of these is a wrap-continuation citation of the form:
```
(per PRD
v1.16 §BC-...)
```

The F-R88 PRD propagation sweep updated `PRD v1.16` citations at inline (single-line) sites but did not sweep the wrap-continuation pattern. The `grep -E "PRD v1\.16"` pattern used by the formal-verifier's §Trace sweep does NOT match wrap-continuation patterns where `PRD` appears on one line and `v1.16` appears on the next line, because `grep -E` is not multi-line by default.

**META layer:** The §Trace v1.22 assertion "post-burst grep returns zero hits" is fabricated or based on a defective regex. This bypasses L-F-R63 Extension 13 (machine-greppable evidence requirement), which mandates that EVERY sweep claim be backed by a code-block transcript of the ACTUAL grep command and ACTUAL output. The §Trace entry used prose-asserted form ("zero hits") without the literal command in code-fence form. This is the structural bypass pattern that Extension 17 (SE-17a + SE-17b) must close.

**Fix route:** Formal-verifier (VP v1.23) — sweep all wrap-continuation `(per PRD\nv1.X)` patterns using `pcregrep -M` or `grep -Pzo` and replace v1.16 with v1.17 at all 6 sites. §Trace must include literal grep command + output per Extension 13 + Extension 17.

---

### F-R89-2 MED — arch HookEventRecord struct missing `#[serde(skip_serializing_if)]` — F-R88-3 PRD-side closure NOT propagated to arch

**Severity:** MEDIUM (arch propagation gap from F-R88-3)

**Layer:** arch SS-daemon-lifecycle.md v1.0.18 (61a0064)

**Defect class:** Sibling-file propagation gap (F-R88-3 arch-side not closed)

**Description:**

F-R88-3 closed BC-RING-001 EC-001 serde annotation by pinning `#[serde(skip_serializing_if = "Option::is_none")]` in PRD v1.17 (27e663c). The F-R88 serial fix-burst applied this closure to PRD (BC-RING-001 EC-001 body) and VP (VP-RING-001 §Pre-conditions and §Mechanism). However, arch SS-daemon-lifecycle.md contains a `HookEventRecord` JSON schema sketch (§Core Types) that specifies optional fields with no `skip_serializing_if` annotation. The arch schema sketch is the upstream reference for the serde configuration; leaving it without the canonical `#[serde(skip_serializing_if = "Option::is_none")]` annotation means the arch schema is inconsistent with the PRD EC-001 closure.

Specifically, `HookEventRecord.cwd` (Option<String>), `HookEventRecord.session_id` (Option<String>), and `HookEventRecord.tool_input` (Option<Value>) — three optional fields in the Phase 1 hook protocol schema — are present in the arch schema sketch without the canonical serde annotation that was mandated by F-R88-3.

**Fix route:** Architect (arch v1.0.19) — add `#[serde(skip_serializing_if = "Option::is_none")]` annotation to all three optional fields in the HookEventRecord schema sketch in SS-daemon-lifecycle.md §Core Types. Serial before VP (FV must pick up arch v1.0.19 pin).

**Bundled observation O-R89-3:** F-R89-2 also suggests a sibling-file propagation checklist item: whenever BC-EC-001 serde annotations are updated in PRD, the corresponding arch schema sketch and VP §Pre-conditions should be verified atomically. This pattern mirrors the F-R83 Extension 14 lift_invariants_to_bcs propagation discipline but for serde-annotation propagation. Bundled into F-R89-2 fix scope (arch v1.0.19 application).

---

### F-R89-3 MED — VP-RING-001 probe matrix missing absence-of-field probes

**Severity:** MEDIUM (VP probe-matrix exhaustiveness gap)

**Layer:** VP v1.22 (e4c1a1e) — VP-RING-001

**Defect class:** Probe-matrix exhaustiveness gap

**Description:**

VP-RING-001 §Verification probe matrix covers:
- Probe 1.a: field present with valid value (happy path)
- Probe 1.b: field present with invalid value (format rejection)
- Probe 1.c: wrong session_id (auth rejection)

The probe matrix has no probe for: **field absent** (i.e., `cwd` absent from serialized JSON; `session_id` absent). Given BC-RING-001 EC-001 now pins `#[serde(skip_serializing_if = "Option::is_none")]`, the absence-of-field case is a first-class protocol behavior: when an optional field is `None`, it is OMITTED from the serialized JSON. The VP must verify that the ring buffer correctly handles a `HookEventRecord` that arrives WITHOUT the optional field present, not just with the field present as `null` or absent-but-defaulted.

This probe gap is directly traceable to F-R88-3: the serde annotation pin created a new first-class protocol case (field omitted vs field null) that was not reflected in the VP-RING-001 probe matrix.

**Fix route:** Formal-verifier (VP v1.23) — add probe 1.d (field-absent case: `cwd` omitted from JSON, verify deserialization succeeds and `HookEventRecord.cwd` is `None`). This is a content gap, not a meta-discipline gap.

---

### F-R89-4 MED — VP-DAEMON-002 probe exhaustiveness gap (10 typed fields declared, 5-6 probed)

**Severity:** MEDIUM (VP probe-matrix exhaustiveness gap)

**Layer:** VP v1.22 (e4c1a1e) — VP-DAEMON-002

**Defect class:** Probe-matrix exhaustiveness gap

**Description:**

VP-DAEMON-002 §Verification declares the `DaemonStatus` response payload as having 10 typed fields:
1. `pid` (u32)
2. `started_at` (ISO 8601 string)
3. `runtime_dir` (String)
4. `config_path` (String)
5. `hook_endpoints` (array of 5 strings — canonical 5-endpoint matrix)
6. `active_sessions` (u64)
7. `drop_counter` (u64)
8. `last_hook_ts` (Option<ISO 8601 string>)
9. `protocol_version` (String)
10. `lock_holder` (Option<String>)

The VP-DAEMON-002 probe matrix has explicit probe rows for approximately 5-6 of these 10 fields (pid, started_at, hook_endpoints, active_sessions/drop_counter, last_hook_ts Option). Fields 3 (`runtime_dir`), 4 (`config_path`), 9 (`protocol_version`), and 10 (`lock_holder` Option-present case) lack dedicated probe rows.

Per the production-grade default (BC-DAEMON-002 Postcondition 1: all 10 fields present in response), absence of probes for 4 of 10 declared fields is a coverage gap. Fields 3 and 4 are path-type fields that require OS-normalized form validation. Field 9 (`protocol_version`) is a forward-compatibility sentinel that should be probed for exact-string match. Field 10 (`lock_holder` Option-present case) requires a probe that establishes a lock and verifies the holder is reported.

**Fix route:** Formal-verifier (VP v1.23) — add probe rows for runtime_dir, config_path, protocol_version, and lock_holder (Option-present case) to VP-DAEMON-002 §Verification probe matrix. Probe for path-normalization, exact-version-string, and lock-holder identity.

---

### O-R89-1 LOW (process-gap) — Extension 17 codification candidate: sweep evidence pair-grep-command + literal output

**Severity:** LOW (process-gap)

**Description:**

F-R89-1 demonstrates that L-F-R63 Extension 13 (machine-greppable evidence requirement) can be bypassed via prose-asserted form. The §Trace v1.22 entry asserted "post-burst grep returns zero hits" without including (a) the literal grep command in code-fence form, and (b) the actual grep output in code-fence form.

Extension 13 codification states that EVERY audit-row claim must be backed by a code-block transcript of the ACTUAL grep command + ACTUAL output. However, the §Trace assertion bypassed this by using prose ("returns zero hits") rather than a code-block transcript.

**Codification candidate:** L-F-R63 Extension 17:

**Rule:** Every grep transcript embedded in a §Trace forensic block MUST include BOTH:
- (a) The literal grep command in code-fence form
- (b) The actual output of that command in code-fence form (matching lines OR explicit "0 matches" if no hits)

NEVER use prose-asserted form ("post-burst grep returns zero hits") without the literal command. The literal command must be re-runnable verbatim.

**Sub-extension SE-17a — multi-line pattern verification:** When the sweep target is a multi-line pattern (e.g., wrap-continuation `(per PRD\nv1.X §BC-...)`), the canonical grep command MUST use `-P` (Perl-compatible regex) with `\n` literal OR `pcregrep -M` OR document the multi-line-aware approach explicitly. Single-line `grep -E "pattern"` will NOT match cross-line patterns.

**Sub-extension SE-17b — self-verification before §Trace assertion:** Before asserting any sweep-completion claim, the agent MUST run the canonical grep command (literally) and paste the output INTO the §Trace narrative. The §Trace is the publication; the act of running the grep is the verification. The two MUST coincide.

**Routing:** State-manager codification in cycle lessons + Critical Hook Lessons entry. Fix route for the content gap (6 stale wrap-continuation cites) is F-R89-1 → formal-verifier.

---

### O-R89-2 LOW — NFR-012 Brief Section parenthetical in VP §Purpose and PRD §4 NFR table

**Severity:** LOW (observation — adjudicated non-defect)

**Description:**

VP §Purpose and PRD §4 NFR table row for NFR-012 include a parenthetical `(Brief §5.3 "Workflow plane — factory awareness")`. R89 inspection notes this parenthetical cites a brief section anchor `§5.3` that is a topical descriptor, not a verified heading anchor in product-brief.md v1.4.2.

**Adjudication:** This is the same convention established by F-R84-6 (PRD frontmatter `traces_to` parenthetical). PRD topical-descriptor parentheticals that describe the brief section's subject matter (not navigate to a heading) are an established convention per F-R84-6 adjudication. Siblings F-R88-2/3/4 all applied the same convention without adversary objection in R88. NFR-012 parenthetical is consistent with the established sibling convention. **NOT a defect.** Counter does not change.

**Note to dispatch:** Adjudicating O-R89-2 as non-defect based on established precedent (F-R84-6). This adjudication is binding for the F-R89 fix-burst scope.

---

### O-R89-3 LOW — Bundled into F-R89-2 (HookEventRecord serde annotation propagation pattern)

**Severity:** LOW (observation bundled into F-R89-2 fix scope)

**Description:** See F-R89-2 "Bundled observation O-R89-3" paragraph. The serde-annotation-propagation checklist item is a sibling-file propagation discipline analogous to Extension 14. Bundled into F-R89-2 (arch v1.0.19) to avoid a standalone codification burst.

---

## Closure Verification (F-R88 findings)

All 5 F-R88 substantive findings verified holding in VP v1.22 / PRD v1.17 / arch v1.0.18:

- **F-R88-1** (arch §Phase 4 Notes 7-field lock-file enum including `contract_version`): VERIFIED CLOSED. arch v1.0.18 line 747 area now enumerates all 7 fields.
- **F-R88-2** (BC-DAEMON-005 2(d) API-accurate wording): VERIFIED CLOSED. PRD v1.17 §BC-DAEMON-005 Precondition 2(d) now states `ProjectDirs::new()` returns `None`.
- **F-R88-3** (BC-RING-001 EC-001 serde annotation): VERIFIED CLOSED (PRD + VP sides). arch side: OPEN — see F-R89-2.
- **F-R88-4** (EC-060 empty-string MONOCLE_RUNTIME_DIR edge case): VERIFIED CLOSED. PRD v1.17 §9 EC-060 present.
- **F-R88-5** (VP §Mechanism uniform "unit-test" mislabel): VERIFIED CLOSED (individual VP §Mechanism blocks + distribution table). §Catalog Overview intro + §Purpose prose: OPEN — see cons R28 GAP-R28-001.

---

## Lens Rotation Result

This pass used CONTENT-CENTRIC lens with sibling-file propagation + probe-matrix exhaustiveness axes. Result: 4 substantive findings (1 HIGH + 3 MED). All findings are in the CONTENT domain, not the META domain. META closure from R87 (SE-16c, Extension 13) continues to hold — no META-axis findings in this pass.

**Lens rotation for next dispatch (R90):** After F-R89 serial fix-burst, dispatch R90 with META-axis lens to verify Extension 17 codification compliance and confirm no META-discipline regressions during the F-R89 fix-burst.

---

## Novelty Assessment

All 4 substantive findings are NOVEL:

- **F-R89-1 HIGH** (wrap-continuation stale citations + META regression): novel — the wrap-continuation propagation failure mode was identified at R85 but the F-R88 §Trace sweep bypassed it at the multi-line level. The META regression is a new axis (Extension 13 prose-asserted bypass).
- **F-R89-2 MED** (arch HookEventRecord serde annotation): novel — F-R88-3 closed PRD and VP sides but did not route the arch-side propagation. First arch-schema-annotation propagation gap in this series.
- **F-R89-3 MED** (VP-RING-001 absence-of-field probe): novel — directly traceable to F-R88-3 creating a new first-class protocol case that the VP probe matrix did not cover.
- **F-R89-4 MED** (VP-DAEMON-002 probe exhaustiveness): novel — 4 of 10 DaemonStatus fields unprobed. First probe-exhaustiveness gap for VP-DAEMON-002.

---

## Routing

Serial fix-burst per Extension 15 dependency order:

1. **Architect** → arch v1.0.19: F-R89-2 (HookEventRecord serde annotations at 3 optional fields)
2. **Formal-verifier** → VP v1.23: F-R89-1 (6 wrap-continuation sites + §Trace Extension 13/17 compliance), F-R89-3 (VP-RING-001 probe 1.d absence-of-field), F-R89-4 (VP-DAEMON-002 probe rows for runtime_dir/config_path/protocol_version/lock_holder)

**State-manager** (this task): persist R89 report + codify L-F-R63 Extension 17 + SE-17a/b in lessons.md + update STATE.md to v5.29 (D-083).

**Cons R28** result (9555282) NOT CLEAN per GAP-R28-001 MED — 2 "unit-test" prose sites in VP §Purpose + §Catalog Overview intro. These are folded into FV scope (VP v1.23) alongside F-R89-1/3/4.
