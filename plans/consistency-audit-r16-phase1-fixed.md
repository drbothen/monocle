---
document_type: consistency-report
level: ops
version: "1.0"
producer: consistency-validator
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T23:45:00Z
input-hash: "[live-state]"
traces_to: "PRD v1.10 (8feecad); VP v1.11 (ebd50a0); arch SS-daemon-lifecycle v1.0.16 (6bb93e2); manifest SS-deps-pin-manifest v1.1.11 (7860e78); STATE.md v5.12 (618663b)"
project: monocle
---

# Consistency Audit — Round 16 (Phase 1 Post-F-R76)

**Verdict: GAPS — 2 findings (1 LOW, 1 LOW-prose)**

**Audit scope:** PRD v1.10 (8feecad) + VP v1.11 (ebd50a0) + arch SS-daemon-lifecycle
v1.0.16 (6bb93e2) + manifest SS-deps-pin-manifest v1.1.11 (7860e78) + STATE.md v5.12
(618663b).

**Checks applied:** Standard 16 checks + L-F-R63 Extensions 1+2+3+3-Enforcement+4+4-expansion+5+6+7 + agent-id-routing-existence + §Trace audit-row integrity.

---

## Summary Table

| Check | Result | Notes |
|-------|--------|-------|
| C-01 Frontmatter currency — PRD | PASS | v1.10 / 8feecad |
| C-02 Frontmatter currency — VP | PASS | v1.11 / ebd50a0 |
| C-03 Frontmatter currency — arch | PASS | v1.0.16 / 6bb93e2 |
| C-04 Frontmatter currency — manifest | PASS | v1.1.11 / 7860e78 |
| C-05 BC count coherence (22) | PASS | PRD §2.1 + §7 RTM both 22 rows; VP catalog overview 22 rows |
| C-06 Error code count coherence (14) | PASS | Unchanged across F-R76 |
| C-07 Edge case count coherence (59) | PASS | Unchanged across F-R76 |
| C-08 Test name count coherence (23) | PASS | Unchanged across F-R76 |
| C-09 VP count = 22 with 1:1 BC coverage | PASS | §Coverage Matrix 22 rows confirmed |
| C-10 F-R76-1 §Pre-conditions — VP-RING-001 serde 1 | PASS | Line 957 cites `serde 1` with `derive` feature |
| C-11 F-R76-1 §Pre-conditions — VP-PROTO-001b not-required disposition | PASS | Lines 1595-1609 document prost-only, serde NOT required |
| C-12 F-R76-2 runtime→axum edge present in manifest graph | PASS | `runtime --> axum` at graph line 175 |
| C-13 F-R76-2 ipc→axum edge absent from manifest graph | PASS | No `ipc --> axum` in mermaid graph |
| C-14 Extension 7 chrono 0.4 in manifest pin table | PASS | Chrono row present in Phase 1 Pin Manifest |
| C-15 Extension 7 chrono propagated — VP-DAEMON-002 | PASS | Line 295 `chrono 0.4 is the project pin (per SS-deps-pin-manifest.md v1.1.11)` |
| C-16 Extension 7 chrono propagated — VP-DAEMON-006 | PASS | Line 869 `chrono 0.4 is the project pin` |
| C-17 Extension 7 chrono propagated — VP-LOCK-001 | PASS | Line 1193 `chrono 0.4 is the project pin` |
| C-18 §Trace audit-row integrity — Extension 3 28-crate table has REAL evidence | PASS | Lines 2664-2709; each row has line-number evidence |
| C-19 §Trace audit-row integrity — Extension 7 grep command confirmed | PASS | §Trace v1.11 lines 2711-2718 document actual grep output |
| C-20 Manifest crate count = 32 production + 1 dev-dep | PASS | 32 rows in Phase 1 Pin Manifest table; §Trace confirms |
| C-21 Manifest runtime node = 15 outbound edges | PASS | 15 `runtime -->` lines in mermaid graph; §Trace count matches |
| C-22 Manifest core node = N outbound edges | **GAP-R16-002** | §Trace claims 6; mermaid graph has 5 (LOW — §Trace prose error) |
| C-23 PRD traces_to manifest current-pointer | **GAP-R16-001** | `SS-deps-pin-manifest.md v1.1.10` in PRD frontmatter; manifest is now v1.1.11 (LOW) |
| C-24 VP §Purpose PRD SHA current | PASS | Line 34 cites `PRD v1.10 (commit 8feecad)` |
| C-25 Agent-id-routing-existence — VP §Scope | PASS | `vsdd-factory:performance-engineer` canonical per CLAUDE.md routing table |
| C-26 Agent-id-routing-existence — VP §G-6 | PASS | `vsdd-factory:phase-f2-spec-evolution` is a skill (acceptable) |
| C-27 VP §Trace v1.11 PG-5 historical-anchor compliance | PASS | Prior §Trace entries (v1.10 including fabricated audit rows) preserved per PG-5; correction documented in v1.11 entry |
| C-28 VP §Trace v1.11 PG-2 count coherence | PASS | 22 VPs, 5 fuzz aux, 4 mutation aux, 0 Kani, 6 open gaps — all stated and consistent |
| C-29 VP §Trace v1.11 intra-block consistency (Extension 2) | PASS | 22 VPs re-verified; 0 §Mechanism vs §Post-conditions contradictions |
| C-30 PRD normative body — serde/chrono consistency with manifest v1.1.11 | PASS | PRD body does not cite bare serde or chrono; 8 crate mentions all consistent with v1.1.11 (same pins, no change) |
| C-31 §Trace audit-row integrity — no self-attestation fabrication in v1.11 Extension 3 audit | PASS | Each audit row has an explicit line-number evidence citation |

**Overall result: GAPS — 2 findings**

---

## Findings

### GAP-R16-001 [LOW] — PRD frontmatter `traces_to` still cites manifest v1.1.10

**Artifact:** `.factory/specs/prd.md` v1.10 (commit 8feecad)
**Location:** Frontmatter line 25, `traces_to:` field
**Evidence:** `SS-deps-pin-manifest.md v1.1.10` is the last manifest version cited in the PRD frontmatter.

**Description:** The F-R76 fix-burst (D-066) changed SS-deps-pin-manifest.md from v1.1.10 to v1.1.11
(commit 7860e78). That burst scoped the PRD as unchanged because the PRD normative body contains no
versioned manifest citations — only path-level references. However, the PRD `traces_to` frontmatter
field is the canonical current-pointer for the manifest in the PRD artifact and was not updated.

**Impact assessment:** LOW. The PRD normative body is entirely consistent with manifest v1.1.11:
the 8 crate-version mentions in the PRD body (axum 0.8, prost 0.14, temp-env ^0.3, ratatui 0.30,
thiserror 2.x, anyhow 1, constant_time_eq ^0.3, syn 2) are unchanged between v1.1.10 and v1.1.11.
The `serde 1` and `chrono 0.4` additions in v1.1.11 are not cited in the PRD body — they are
exclusively architecture/VP-level implementation pins. No PRD behavioral contract, precondition,
postcondition, invariant, edge case, or test vector is affected.

**Severity justification:** The PRD `traces_to` is a traceability bookkeeping field. The gap is
a version-pointer staleness finding with zero semantic impact on any PRD specification. Classified
LOW (not MED) because: (a) PRD content is correct against manifest v1.1.11, (b) the PRD was
explicitly documented as unchanged by D-066, (c) the stale pointer references the direct predecessor
version with a clear update chain.

**Remediation:** Product-owner to update PRD `traces_to` frontmatter: change
`SS-deps-pin-manifest.md v1.1.10` to `SS-deps-pin-manifest.md v1.1.11`. Append F-R76 closure
reference. Also update the PRD §Trace D-042 sweep at v1.10 to note that manifest was at v1.1.10
at time of v1.10 authoring and has since advanced to v1.1.11 via F-R76 without PRD body impact.
This is a single-field frontmatter update; no BC, NFR, or EC content changes required.

---

### GAP-R16-002 [LOW] — Manifest §Trace v1.1.11 core node edge count off by 1

**Artifact:** `.factory/specs/architecture/SS-deps-pin-manifest.md` v1.1.11 (commit 7860e78)
**Location:** §Trace v1.1.11, line 370-371
**Evidence:** `§Trace states: "`core` node now has 6 outbound edges: thiserror, semver, futures, async_trait, serde (new)"`
**Actual graph edges:** `core --> thiserror`, `core --> semver`, `core --> futures`, `core --> async_trait[async-trait]`, `core --> serde` = 5 edges.

**Description:** The §Trace prose claims the core node has 6 outbound edges but enumerates only 5,
and the mermaid graph itself contains exactly 5 `core -->` lines. The claim "6" is a prose
arithmetic error — the count stated does not match either the enumeration immediately following it
or the actual graph.

**Impact assessment:** LOW. This is a §Trace accounting error in historical/narrative content.
The mermaid graph itself is correct (5 edges), and no behavior specification is affected. The
§Trace is descriptive prose summarizing the change — the graph is the normative dep-graph authority.
No implementer would use the §Trace edge-count claim to build a Cargo.toml; they would use the
graph diagram.

**Severity justification:** LOW (not MED) because: (a) the authoritative mermaid graph is
correct, (b) the error is in §Trace accounting prose only, (c) no behavioral contract, BC, VP,
or implementation spec is affected, (d) the enumeration following the claim (thiserror, semver,
futures, async_trait, serde) is correct.

**Remediation:** Architect to update §Trace v1.1.11 prose: change "`core` node now has 6 outbound
edges" to "`core` node now has 5 outbound edges". Single-word numeral correction in §Trace prose.
No graph changes required.

---

## F-R76 Closure Verification

All four F-R76 closure items verified:

**F-R76-1 (VP-RING-001 §Pre-conditions serde 1):** CLOSED.
- VP line 957: `serde 1 (with derive feature, declared as serde = { version = "1", features = ["derive"] }) is the project pin (per SS-deps-pin-manifest.md v1.1.11)`.
- Manifest pin table: serde row present with derive feature.
- Manifest dep graph: `runtime --> serde` and `core --> serde` present.

**F-R76-1 (VP-PROTO-001b §Pre-conditions disposition):** CLOSED.
- VP lines 1595-1609: explicit documentation that bare `serde 1` is NOT required for VP-PROTO-001b (prost-only crate path).
- Fabrication discovery documented in §Trace v1.11 with preservation of prior §Trace entries as historical record.

**F-R76-2 (runtime→axum edge added, ipc→axum edge removed):** CLOSED.
- Mermaid graph: `runtime --> axum` present at line 175.
- Mermaid graph: no `ipc --> axum` line anywhere in the graph block.

**Extension 7 chrono 0.4 net-new discovery:** CLOSED.
- Manifest pin table: chrono row present with caret pin and ISO 8601 formatting mandate.
- Manifest dep graph: `runtime --> chrono` present.
- VP-DAEMON-002 §Pre-conditions (line 295): chrono 0.4 cited.
- VP-DAEMON-006 §Pre-conditions (line 869): chrono 0.4 cited.
- VP-LOCK-001 §Pre-conditions (line 1193): chrono 0.4 cited.

**§Trace audit-row integrity discipline:** CLOSED.
- Extension 3 28-crate audit table (lines 2664-2709): every row has explicit line-number evidence.
- Extension 7 grep discipline (lines 2711-2718): grep command documented with actual output.
- No self-attestation PASS verdicts; all claims independently grep-verifiable.

---

## Extension Checks

**Extension 1 (arch + PRD + manifest pin propagation):** PASS for VP and manifest. GAP-R16-001
identifies the one remaining stale pointer (PRD frontmatter → manifest v1.1.10).

**Extension 2 (intra-block consistency sweep):** PASS — VP §Trace v1.11 confirms all 22 VPs
re-verified; 0 contradictions.

**Extension 3 (deps-pin-manifest enforcement, FULL discipline):** PASS — 28-crate audit table
with REAL grep evidence per Obs-R76-1. Zero stale pins in normative-current VP content.

**Extension 3-Enforcement:** PASS — Extension 3 is explicitly applied in VP §Trace v1.11 per
full discipline requirement.

**Extension 4 (schema-sketch precision / ellipsis anti-pattern):** PASS — `GET /status`
hook_endpoints field uses canonical 5-string enumeration (not ellipsis) per F-R74-1 closure
(unchanged in this burst).

**Extension 4-expansion (ellipsis pattern coverage):** PASS — no ellipsis `[..., "...", ...]`
patterns present in any normative arch content.

**Extension 5 (VP-coverage-vs-BC-EC-security-property sweep):** PASS — VP-DAEMON-005 postcondition
9 + probe row 5.e + counter-example 10 cover 0o700 mode per BC-DAEMON-005 EC-052 (added in VP v1.10
F-R75-1 closure; unchanged in this burst).

**Extension 6 (rationale-prose-vs-NFR-canonical-contract sweep):** PASS — BC-DAEMON-005
precondition 2 rationale correctly scopes Windows as secondary target per NFR-008 macOS+Linux
scope (fixed in F-R75-2; unchanged in this burst).

**Extension 7 (automated crate-prefix grep audit + §Trace audit-row integrity):** PASS — this
round's primary validation target. All four F-R76 closure items verified (see above). §Trace
v1.11 Extension 7 section documents REAL grep output (not self-attestation).

**Agent-id-routing-existence sweep:** PASS — all `vsdd-factory:*` IDs in VP normative body resolve
against CLAUDE.md routing table: `vsdd-factory:performance-engineer` (canonical per D-063),
`vsdd-factory:phase-f2-spec-evolution` (skill reference, acceptable).

**§Trace audit-row integrity:** PASS for VP v1.11. Each §Trace Extension 3 audit-row claim
has an independent line-number citation. The fabrication pattern from v1.8/v1.9/v1.10 is
documented and corrected. GAP-R16-002 identifies one remaining §Trace prose arithmetic error
in manifest v1.1.11 (core node edge count 6 vs actual 5).

---

## Convergence Counter Impact

Both GAP-R16-001 and GAP-R16-002 are LOW severity. Under D-047 strict (0 findings of any
severity required), these findings prevent a CLEAN verdict. The D-047 counter does NOT advance.
Counter remains at 0/3.

However, the severity and scope of these findings is notably different from prior rounds:
- Neither finding affects any behavioral contract, VP property, precondition, postcondition,
  invariant, edge case, test vector, or implementation spec.
- GAP-R16-001 is a traceability frontmatter pointer that missed the v1.1.10 → v1.1.11 update.
- GAP-R16-002 is a one-word numeral error in §Trace historical/accounting prose.

Both are remediable with single-field or single-word fixes. No semantic drift.

---

## Prior Round Comparison

| Round | Verdict | Findings |
|-------|---------|----------|
| R15 (cons, deef8ad) | CLEAN | 0 |
| **R16 (this, post-F-R76)** | **GAPS** | **2 LOW** |

R15 was CLEAN on PRD v1.10 + VP v1.10 + arch v1.0.16 + manifest v1.1.10. The 2 LOW findings
in R16 are introduced by the F-R76 burst artifacts (manifest v1.1.11 + VP v1.11) — specifically:
- GAP-R16-001: the F-R76 burst correctly scoped the PRD as unchanged (no body impact) but did
  not update the PRD frontmatter pointer as a housekeeping step.
- GAP-R16-002: the F-R76 §Trace v1.1.11 introduced an arithmetic error in the core-node edge
  count claim.

Neither finding represents a defect in the fix-burst's primary closure work (F-R76-1 + F-R76-2
+ Extension 7). Both are secondary housekeeping artifacts of the burst.

---

## Remediation Routing

| Finding | Routing | Fix |
|---------|---------|-----|
| GAP-R16-001 | `vsdd-factory:product-owner` | PRD `traces_to` frontmatter: v1.1.10 → v1.1.11; no body changes |
| GAP-R16-002 | `vsdd-factory:architect` | Manifest §Trace v1.1.11 line 370: "6 outbound edges" → "5 outbound edges" |
