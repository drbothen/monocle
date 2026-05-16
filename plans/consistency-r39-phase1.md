---
document_type: consistency-pass
pass_id: R39
attempt: 33
verdict: GAPS
artifact_pins:
  | Artifact | Version | Commit |
  |----------|---------|--------|
  | product-brief.md | v1.4.23 | 42ec508 (CLAUDE.md) |
  | prd.md | v1.23 | d2c0b66 |
  | verification-properties.md | v1.33 | dec90d2 |
  | SS-daemon-lifecycle.md | v1.0.23 | d088123 |
  | SS-deps-pin-manifest.md | v1.1.15 | d088123 |
  | SS-core-types-and-abi.md | v1.2.8 | (unchanged) |
  | SS-engine-module.md | v1.1.15 | (unchanged) |
  | SS-conventions-anti-patterns.md | v1.6 | (unchanged) |
  | SS-permissions-phase1.md | (not checked — not primary spec) | — |
  | SS-forward-compatibility.md | (not checked — not primary spec) | — |
  | ADR-0001..ADR-0004 | (confirmed present) | — |
  | domain-monocle-vision-synthesis.md | v1.1.2 | — |
  | CLAUDE.md | commit 42ec508 | — |
dimensions_applied:
  - "1. Cross-artifact pin coherence: every normative-current pin citation sweep"
  - "2. RTM (PRD §7) ↔ VP §Coverage Matrix: 22 BCs; Test File column; Mechanism labels"
  - "3. BC anchors: VP probe / counter-example anchors to real BC §Postcondition / §Invariant / EC"
  - "4. §Purpose / §References / §Trace lineage: SHA cites match; timestamps current; monotonicity"
  - "5. manifest ↔ arch ↔ VP triple pin: 28 dep pins coherent"
  - "6. Glossary completeness (PRD §10): every normative term has an entry"
  - "7. EC anchoring: ECs anchor to correct BC owner"
  - "8. NFR-to-VP coverage (Extension 8): all 12 NFRs have VP probe citation or Phase-deferral"
  - "9. CLAUDE.md cites: lines 22+47 cite brief v1.4.23; line 48 cites vision v1.1.2; line 225 historical pin"
  - "10. brief / vision / ADR / cross-doc consistency: internal cross-references consistent"
timestamp: 2026-05-16T04:52:00Z
---

# Consistency Pass R39 — D-047 Strict Attempt 33

## §Summary

**Verdict: GAPS**

One MED-severity finding and one LOW-severity observation detected. Per D-047
strict gate, a MED+ severity finding holds the counter at 0/3.

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MED | 1 |
| LOW | 1 (observation) |

Blocking finding: GAP-R39-001 (MED) — VP §Coverage Matrix footer opens with a
stale PRD v1.22 citation when the current canonical PRD is v1.23. Counter holds
at 0/3.

---

## §Findings

### GAP-R39-001

**Severity:** MED
**Dimension:** 2 (RTM ↔ VP Coverage Matrix cross-check) / 1 (cross-artifact
pin coherence)
**File:line:** `/Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md:2529`

**Description:**
The §Coverage Matrix body-text opens with:

```
Every test-file path matches PRD v1.22 §7. Requirements Traceability Matrix verbatim
```

The literal text `PRD v1.22` at the opening of line 2529 is a stale current-pointer
citation. The canonical PRD version is v1.23 (commit d2c0b66), advanced from v1.22
(commit d3df32e) in the F-R99 Burst 3 closure. The remainder of the same long
paragraph correctly states:

```
PRD v1.23 §Section 7 RTM and the corresponding BC `Verification` subsection verbatim
```

and:

```
PRD v1.23 is the F-R99 Burst 3 pin-propagation closure (commit d2c0b66 — current
canonical PRD source)
```

The opening sentence was not updated when the PRD pin was bumped v1.22→v1.23 at VP
v1.33 time. This is the same class as the recurring D-042 citation-staleness
META-pattern that has affected the brief/arch/manifest triple previously.

**Evidence (literal):**
- VP line 2529 opens: `test-file path matches PRD v1.22 §7.`
- VP frontmatter `traces_to` (line 25) correctly states: `PRD v1.22→v1.23 (commit d3df32e→d2c0b66)`
- VP body line 170: `Cross-reference: PRD v1.23 §7 RTM Test Type column` (correct)
- PRD frontmatter line 4: `version: "1.23"` (confirmed)
- PRD frontmatter line 25: `traces_to:` ... `PRD v1.23 is the F-R99 Burst 3 pin-propagation closure (commit d2c0b66 — current canonical PRD source)` (confirmed)

The coverage matrix footer is a normative-current site (not a historical predecessor
citation per PG-5 — it does not use historical framing or a "was v1.22" qualifier).
The opening `PRD v1.22` is presented as a factual claim about what the test-file paths
match, making it a normative-current pin that was not swept during VP v1.33's F-R99
Burst 4 pin propagation.

**Proposed routing:** formal-verifier agent (VP v1.34 burst); `PRD v1.22` → `PRD v1.23`
at the opening of line 2529. The subsequent history narrative in the same line is
correctly structured under PG-5 and must not be altered.

---

## §PASS Dimensions

The following dimensions were fully checked and found CLEAN:

**Dimension 1 — Cross-artifact pin coherence (excluding GAP-R39-001):**
- PRD v1.23 body: all 22 BC Source fields cite SS-daemon-lifecycle.md v1.0.23 ✓
- PRD v1.23 body: all SS-engine-module references cite v1.1.15 ✓
- PRD v1.23 body: all SS-core-types-and-abi references cite v1.2.8 ✓
- VP v1.33 §VP Catalog Overview: all VP-DAEMON-001..006 rows cite
  `PRD v1.23 / SS-daemon-lifecycle v1.0.23` ✓
- VP v1.33 §Coverage Matrix table (rows, not the footer sentence): all 22 rows
  cite current canonical source files ✓
- SS-daemon-lifecycle.md v1.0.23 frontmatter: confirmed actual file version ✓
- SS-deps-pin-manifest.md v1.1.15 frontmatter: confirmed actual file version ✓
- SS-engine-module.md v1.1.15 frontmatter: confirmed actual file version ✓
- SS-core-types-and-abi.md v1.2.8 frontmatter: confirmed actual file version ✓
- VP v1.33 §Purpose line 34–35: cites PRD v1.23 (commit d2c0b66) — 20th META
  recurrence guard applied substantively ✓

**Dimension 2 — RTM ↔ VP Coverage Matrix (excluding GAP-R39-001):**
- PRD §7 RTM: 22 BC rows + 1 NFR-012 row, correct Requirement ID / Architecture
  Source / Test File / Test Type columns ✓
- VP §Coverage Matrix table rows: 22 rows, 1:1 BC→VP mapping, test-file paths
  match PRD §7 RTM verbatim ✓
- Mechanism labels: VP §Mechanism Distribution taxonomy (integration-test 18,
  ast-audit 3, compile-time-check 1) matches PRD §7 RTM Test Type column ✓
- PRD §7 RTM BC-DAEMON-004 dual test files: `graceful_shutdown.rs` +
  `daemon_lifecycle.rs` present in both PRD §7 and VP §Coverage Matrix ✓

**Dimension 3 — BC anchors:**
- VP-DAEMON-001..006 all trace to PRD v1.23 §BC-DAEMON-001..006 and
  SS-daemon-lifecycle.md v1.0.23 with specific section anchors ✓
- VP-RING-001 / VP-AUTH-001 / VP-AUTH-002 / VP-LOCK-001 trace to
  SS-daemon-lifecycle.md v1.0.23 ✓
- VP-ABI-001/002 / VP-TYPES-001 / VP-FACTORY-001/002 / VP-PROTO-001a/001b/002
  trace to SS-core-types-and-abi.md v1.2.8 ✓
- VP-ENGINE-001/002/002-ERR/003 trace to SS-engine-module.md v1.1.15 ✓

**Dimension 4 — §Purpose / §References / §Trace lineage:**
- VP v1.33 frontmatter timestamp: `2026-05-17T01:00:00Z` is canonical UTC Z-form
  and ≥ PRD v1.23 high-water `2026-05-17T00:30:00Z` — SE-16d PASS ✓
- PRD v1.23 frontmatter timestamp: `2026-05-17T00:30:00Z` is canonical UTC Z-form ✓
- SS-daemon-lifecycle.md v1.0.23 frontmatter timestamp: `2026-05-17T00:00:00Z` ✓
- SS-deps-pin-manifest.md v1.1.15 frontmatter timestamp: `2026-05-17T00:00:00Z` ✓
- Cross-chain monotonicity: arch v1.0.23 (00:00Z) ≤ manifest v1.1.15 (00:00Z) ≤
  PRD v1.23 (00:30Z) ≤ VP v1.33 (01:00Z) — PASS ✓

**Dimension 5 — manifest ↔ arch ↔ VP triple pin coherence:**
- 28 dep pins in SS-deps-pin-manifest.md v1.1.15: complete pin table verified
  (ratatui 0.30, tokio 1.52, axum 0.8.9, serde_json 1.0.149, wasmtime 44, rand 0.8.6,
  nucleo 0.5, russh 0.60, rmcp 1.6, constant_time_eq 0.3, nix 0.30, etc.)
- VP traces_to frontmatter cites manifest v1.1.15 (commit d088123) ✓
- PRD traces_to frontmatter cites manifest v1.1.15 (commit d088123) ✓
- No normative crate-version citation conflicts found across PRD body §8.1-§8.5
  cross-cutting concerns vs manifest pins ✓

**Dimension 6 — Glossary completeness (PRD §10):**
- All terms used normatively in BCs (e.g., `MONOCLE_RUNTIME_DIR`,
  `DaemonStartError::RuntimeDirUnresolvable`) have §10 entries per O-R91-4 ✓
- No new normative terms introduced in v1.23 (pin-only burst) that lack glossary
  entries ✓

**Dimension 7 — EC anchoring:**
- EC-001..EC-061 all grouped by BC owner in PRD §9; cross-reference table
  confirms each EC anchors to its correct BC ✓
- EC-052 (runtime_dir creation failure) and Postcondition 8 correctly split as
  failure-path EC vs success-path postcondition per F-R79-3 ✓

**Dimension 8 — NFR-to-VP coverage (all 12 NFRs):**
- NFR-001/002/003 (latency): formal deferral in VP §G-6 with concrete Phase 3
  future-attachment ✓
- NFR-004 (auth entropy): covered by VP-AUTH-001 §Pre-conditions + Mechanical
  property item 1 ✓
- NFR-005 (body size limit): covered by VP-DAEMON-003 §Post-condition 1 ✓
- NFR-006 (bounded channel + drop counter): formal deferral in VP §G-7 with
  concrete Phase 3 future-attachment ✓
- NFR-007 (MSRV): out-of-scope structural (CI matrix enforcement); documented
  as out-of-scope in VP §Trace v1.12 ✓
- NFR-008 (platform targets): out-of-scope structural (GitHub Actions CI matrix) ✓
- NFR-009 (lock-file 0o600): covered by VP-DAEMON-005 Post-condition 1 + probe 5.e ✓
- NFR-010 (constant-time comparison): covered by VP-AUTH-001 §Post-condition 5
  (source-grep) ✓
- NFR-011 (DTU fidelity ≥0.95): covered by §G-2 deferral + dtu-assessment.md
  §DTU Fidelity Measurement Procedure ✓
- NFR-012 (runtime_dir 0o700): covered by VP-DAEMON-005 Post-condition 9 / probe
  5.e; PRD §7 RTM NFR-012 row cites `monocle-runtime/tests/daemon_lifecycle.rs`
  with VP-DAEMON-005 Post-condition 9 annotation ✓
  NOTE: The VP §Trace v1.12 historical block (line ~11118) reads "11 NFRs audited"
  — this was correct at v1.12 time (pre-NFR-012 addition at PRD v1.13). NFR-012 is
  substantively covered via VP-DAEMON-005; the "11 NFRs audited" count is a §Trace
  v1.12 historical record preserved verbatim per PG-5. No normative gap. Classified
  as LOW observation below.

**Dimension 9 — CLAUDE.md cites:**
- Line 22: `v1.4.23` at `.factory/specs/product-brief.md` ✓
- Line 47: `product-brief.md` v1.4.23 ✓ (brief text also updated correctly)
- Line 48: `domain-monocle-vision-synthesis.md` v1.1.2 ✓
- Line 225: historical manifest pin clarification `v1.1.1 at architect's stub-completion;
  current v1.1.15` ✓ (correctly describes historical v1.1.1 stub and current v1.1.15)
- Vision actual frontmatter: `version: "1.1.2"` — confirmed ✓
- Brief actual frontmatter: `version: "1.4.23"` — confirmed ✓

**Dimension 10 — brief / vision / ADR / cross-doc consistency:**
- Product brief v1.4.23 supplements list: 12 supplements including SS-daemon-lifecycle.md,
  SS-engine-module.md, ADR-0004 ✓
- PRD §1.3 Differentiators BC backing: 8 differentiators with correct BC IDs ✓
- PRD §2.1 BC grouping table: 11 domain rows, 22 BC IDs listed ✓
- No ADR version references in PRD body (PRD does not cite ADR versions in body, only
  references them by heading/decision) ✓
- Vision §Vision Statement and §End-to-End Killer Scenario quotes in PRD §1.1-1.2
  match vision v1.1.2 content ✓

---

## §Open Observations

### OBS-R39-001 (LOW — process)

**File:line:** `/Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md` §Trace v1.12 block (~line 11118)
**Description:** The VP §Trace v1.12 historical block contains "11 NFRs audited" —
accurate when written (before NFR-012 was added to PRD at v1.13). NFR-012 was
subsequently added (PRD v1.13, F-R83-1) and is mechanically covered by VP-DAEMON-005
Post-condition 9 / probe 5.e. The VP §Trace v1.12 block is preserved verbatim under
PG-5 (historical-predecessor framing). No normative coverage gap exists. This
observation flags the historical count as potentially confusing to a reader scanning
the §Trace for the NFR coverage tally, but it does not constitute a MED+ gap because
(a) the §Trace v1.12 block is explicitly a historical record, and (b) NFR-012 is
demonstrably covered at the VP body level via VP-DAEMON-005. Routing: low-priority
context item for formal-verifier awareness; may choose to annotate in a future §Trace
entry under PG-5 framing.

---

## §Gate Result

**D-047 STRICT GATE: HOLD — counter remains at 0/3**

GAP-R39-001 is MED severity and therefore a blocking finding per D-047 strict gate
rules. The consistency pass cannot advance the counter.

Required action: formal-verifier bursts VP v1.34 to update the opening sentence of
the §Coverage Matrix footer at verification-properties.md line 2529 from `PRD v1.22`
to `PRD v1.23`. Routing: formal-verifier agent.
