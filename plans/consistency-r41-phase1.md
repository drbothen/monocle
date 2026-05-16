---
document_type: consistency-pass
pass_id: R41
attempt: 35
verdict: CLEAN
artifact_pins:
  - artifact: PRD
    version: "1.25"
    commit: 7735c84
  - artifact: verification-properties
    version: "1.35"
    commit: 842402c
  - artifact: SS-daemon-lifecycle
    version: "1.0.25"
    commit: 18fe265
  - artifact: SS-deps-pin-manifest
    version: "1.1.15"
    commit: d088123
  - artifact: SS-core-types-and-abi
    version: "1.2.8"
    commit: unchanged
  - artifact: SS-engine-module
    version: "1.1.15"
    commit: unchanged
  - artifact: product-brief
    version: "1.4.23"
    commit: current
  - artifact: vision-synthesis
    version: "1.1.2"
    commit: current
  - artifact: STATE
    version: "5.55"
    commit: b9cccd0
dimensions_applied:
  - D1: Cross-artifact pin coherence (scoped-awk; 10 dimensions)
  - D2: RTM Coverage Matrix consistency (22 BC rows)
  - D3: BC anchors in VP (22 VP sections, §Purpose/§Scope/§References)
  - D4: Lineage integrity (SHA validity, UTC monotonicity SE-16d)
  - D5: Manifest arch VP triple pin (28 dep pins)
  - D6: Glossary completeness (PRD §10, 19 terms)
  - D7: EC anchoring (61 ECs, 22 BCs)
  - D8: NFR-to-VP coverage (12 NFRs)
  - D9: CLAUDE.md cites (lines 22/47/48/225)
  - D10: Cross-doc consistency (brief/vision/ADRs/DTU)
timestamp: 2026-05-16T07:00:00Z
---

# Consistency Report R41 — Post F-R101 5-Burst Closure Chain

**Pass ID:** R41 (attempt 35)
**Verdict: CLEAN**
**MED+ findings:** 0
**LOW observations:** 1 (pre-existing; does not block counter advancement)
**Gate:** PASSES D-047 strict. Counter condition: CLEAN — advances to 1/3 if adversary R102 also CLEAN.

---

## §Summary

| Dimension | Status | Evidence |
|-----------|--------|---------|
| D1: Cross-artifact pin coherence | PASS | 0 pre-trace normative stale pins in PRD (32 arch sites all v1.0.25); 0 stale pins in VP pre-trace body; scoped-awk applied to all pattern greps |
| D2: RTM Coverage Matrix | PASS | 22 BC rows in PRD §7 RTM; 22 BC rows in VP §Coverage Matrix; Coverage Matrix opener: "PRD v1.25 §7" (correct) |
| D3: BC anchors | PASS | 22 VP sections (§VP-DAEMON-001..006, §VP-RING-001, §VP-AUTH-001/002, §VP-LOCK-001, §VP-ABI-001/002, §VP-TYPES-001, §VP-FACTORY-001/002, §VP-PROTO-001a/001b/002, §VP-ENGINE-001/002/002-ERR/003) 1:1 with 22 BCs; VP §Purpose: "PRD v1.25 (commit 7735c84)" CORRECT |
| D4: Lineage / SHA / Timestamps | PASS | All 4 commit SHAs valid: 7735c84 (PRD), 842402c (VP), 18fe265 (arch), d088123 (manifest). Cross-chain UTC monotonicity: Manifest 2026-05-17T00:00:00Z < Arch 06:00:00Z < PRD 06:30:00Z < VP 07:00:00Z — SE-16d PASS |
| D5: Manifest arch VP triple pin | PASS (1 LOW OBS) | 28 dep pins coherent across manifest/arch/VP; 1 LOW: reqwest in Phase 1 pin table has no dep graph consumer edge (see OBS-R41-1) |
| D6: Glossary completeness | PASS | 19 terms in PRD §10; all Phase 1 contracts covered including MONOCLE_RUNTIME_DIR + DaemonStartError::RuntimeDirUnresolvable (added v1.19) |
| D7: EC anchoring | PASS | 61 ECs (EC-001..EC-061) each anchored to a specific BC; EC-060 (BC-DAEMON-005) + EC-061 (BC-FACTORY-002) present; grouping-order documented in §9 header note |
| D8: NFR-to-VP coverage | PASS | 12 NFRs in PRD §4; NFR-001/002/003 Phase 3 deferral in VP §G-6; NFR-004/005 direct VP probes cited; NFR-006 Phase 3 deferral in VP §G-7; NFR-007/008 structural CI gates; NFR-009 VP-DAEMON-005 Post-condition 1 cited; NFR-010 VP-AUTH-001 §Post-condition 5 cited; NFR-011 DTU §G-2; NFR-012 VP-DAEMON-005 Post-condition 9 cited |
| D9: CLAUDE.md cites | PASS | Line 22: "v1.4.23" CORRECT; Line 47: "v1.4.23" CORRECT; Line 48: "v1.1.2" CORRECT; Line 225: historical "v1.1.1 at architect's stub-completion; current v1.1.15" — correctly qualified historical note |
| D10: Cross-doc consistency | PASS | Vision v1.1.2 frontmatter confirmed; Brief v1.4.23 frontmatter confirmed; ADR-0004 v1.0.3 consistent with PRD BC-TYPES-001/Phase1Permission/ClaudeCodeTool references; DTU dtu_required: true matches STATE.md; STATE v5.55 F-R101 chain pins match actual artifact versions |

**Overall verdict: CLEAN — no MED+ findings. One LOW pre-existing observation (OBS-R41-1) documented below.**

---

## §Findings

None at MED or above severity.

---

## §Open Observations (LOW)

### OBS-R41-1 — reqwest in Phase 1 Pin Table Missing from Workspace Dep Graph

**Severity:** LOW
**Artifact:** `.factory/specs/architecture/SS-deps-pin-manifest.md` v1.1.15
**Location:** §Phase 1 Pin Manifest line 64 (pin table row) vs §Workspace Dependency Graph (lines 149-215, mermaid diagram)

**Observation:**

`reqwest 0.13` appears in the Phase 1 Pin Manifest table (line 64: "HTTP client | EXACT pin") and is cited 3 additional times in the pre-trace Patch-Pinning Policy section (lines 110, 112, 117) as one of the 9 EXACT-pinned security-critical crates. However, the Workspace Dependency Graph mermaid diagram has no `reqwest` node and no consumer edge pointing to `reqwest` from any Phase 1 crate.

The dep graph shows `bin`, `tui`, `runtime`, `ipc`, `config`, `proto`, `sdk`, `workflow`, `core`, `fuzz`, `harness` as Phase 1 crates, with `runtime → axum` (HTTP server) correctly present. No crate has `→ reqwest` (HTTP client). The SS-daemon-lifecycle spec confirms TUI-to-daemon communication uses UDS (`interprocess` via `monocle.sock`), not HTTP client calls from the TUI.

**Status:** Pre-existing since manifest v1.1.x. Not raised in R1-R40. The dep graph comment at line 147 notes it "refreshes after Cargo workspace `Cargo.toml` files exist during `/vsdd-factory:create-architecture`" — suggesting the graph may be incomplete at pre-Phase-1. This is a LOW observation, not a blocking finding, because: (a) the architect annotated the graph as provisional, (b) `reqwest` may be used by a Phase 1 crate not yet in the graph (e.g., a future TUI health-check HTTP call), or (c) `reqwest` may belong in a Phase 2/3/4 addition section rather than the Phase 1 table.

**Remediation (deferred to architect at Phase 1 architecture create):** Clarify which Phase 1 crate consumes `reqwest` and add the dep graph edge, OR move `reqwest` to the Phase 4 additions section if it is only needed for federation/bridge purposes. This does not block Phase 1 spec convergence.

---

## §PASS Dimensions (explicit list)

1. **D1 — Cross-artifact pin coherence**: PASS (scoped-awk applied; 0 stale normative pins in PRD pre-trace body, 0 stale normative pins in VP pre-trace body, 0 stale pins in arch pre-trace body)
2. **D2 — RTM Coverage Matrix**: PASS (22 BC rows in both PRD §7 RTM and VP §Coverage Matrix; Coverage Matrix opener corrected to "PRD v1.25 §7" in v1.34 per GAP-R39-001)
3. **D3 — BC anchors**: PASS (22 VP sections, 1:1 with 22 BCs; VP §Purpose, §Scope, §References items 1/2/6 all cite current canonical pins)
4. **D4 — §Purpose/§References/§Trace lineage**: PASS (SHA cites 7735c84/842402c/18fe265/d088123 all valid; UTC cross-chain monotonic; all Z-form timestamps per SE-16d)
5. **D5 — Manifest arch VP triple pin**: PASS (28 dep pins coherent; 1 LOW observation on reqwest dep graph gap)
6. **D6 — Glossary completeness**: PASS (19 terms; complete coverage for all Phase 1 contracts)
7. **D7 — EC anchoring**: PASS (61 ECs, all anchored to correct BCs; EC-054/055/056 non-monotonic ordering documented)
8. **D8 — NFR-to-VP coverage**: PASS (12 NFRs, all with documented coverage disposition; 0 uncovered NFRs)
9. **D9 — CLAUDE.md cites**: PASS (lines 22/47 = v1.4.23; line 48 = v1.1.2; line 225 = historical note correctly qualified)
10. **D10 — Cross-doc consistency**: PASS (brief/vision/ADR/DTU all internally consistent; STATE.md pins match actual file versions)

---

## §Methodology

- Scoped-awk (D-116 empirically validated): `BOUNDARY=$(grep -n "^## §Trace" <file> | head -1 | cut -d: -f1); awk -v B="$BOUNDARY" "NR<B" <file> | grep ...` applied for all substring-content greps. §Trace sections structurally excluded — META-N+9 absent by construction.
- 33 disciplines in force (SE-17h HELD per D-114 Goodhart's law — no new codification this pass).
- Artifact pins verified against `git -C .factory show --format="%H" --no-patch <SHA>` for all 4 canonical commits.
- VP §Purpose PRD SHA 7735c84 cross-checked against actual git SHA: `7735c8478a8c25dd2cb9f5e456b69ad6f00414c2` — MATCH.
- 22 BC count: PRD §3 has 22 `### BC-` headings; VP has 22 `### §VP-` headings; VP §Coverage Matrix has 22 `| BC-` rows; VP §VP Catalog Overview has 22 `| VP-` rows — all CONSISTENT.
- CLAUDE.md line numbers verified at actual file positions (line 22, 47, 48, 225) per direct Read.
