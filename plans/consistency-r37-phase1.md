---
document_type: consistency-pass
pass_id: R37
attempt: 31
timestamp: "2026-05-15T17:00:00Z"
artifact_pins:
  - artifact: prd.md
    version: "1.21"
    sha: 0f124a9
    verified: true
  - artifact: verification-properties.md
    version: "1.31"
    sha: a3a68a4
    verified: true
  - artifact: SS-daemon-lifecycle.md
    version: "1.0.21"
    sha: 42504b4
    verified: true
  - artifact: SS-deps-pin-manifest.md
    version: "1.1.13"
    sha: 42504b4
    verified: true
  - artifact: product-brief.md
    version: "1.4.23"
    verified: true
  - artifact: domain-monocle-vision-synthesis.md
    version: "1.1.2"
    verified: true
dimensions_applied:
  - "1. Cross-artifact pin coherence (canonical version pins across all spec files)"
  - "2. RTM (PRD §7) ↔ VP §Coverage Matrix coherence (22 BCs, test files, mechanism labels)"
  - "3. Behavioral Contract anchors (VP probe/counter-example BC anchor resolution)"
  - "4. VP §Purpose SHA cite / §References intro timestamp / §Trace lineage monotonicity"
  - "5. Manifest ↔ arch ↔ VP triple pin coherence (28 dep pins)"
  - "6. Glossary completeness (PRD §10: terms used normatively)"
  - "7. EC anchoring (60+ ECs, BC owner correctness)"
  - "8. NFR-to-VP coverage (12 NFRs, VP probe citations or documented Phase deferral)"
  - "9. CLAUDE.md brief cite (lines 22 + 47) and vision cite (line 48)"
  - "10. Brief / vision / ADR consistency (cross-reference coherence)"
---

# Consistency Pass R37 — Phase 1 Spec Package

## §Summary

**Verdict: GAPS**

**Finding count by severity:**

| Severity | Count |
|----------|-------|
| HIGH     | 1     |
| MED      | 0     |
| LOW      | 0     |

**Total findings: 1 (HIGH)**

One confirmed gap: CLAUDE.md §Architectural Authority line 48 cites vision
`v1.1.1` but the vision document's frontmatter shows `version: "1.1.2"` as
the canonical version. This is a stale version citation in a normative
architectural authority list.

All other dimensions audited CLEAN.

---

## §Artifact Pin Verification

All four canonical artifact pins verified against actual frontmatter:

| Artifact | Claimed Version | Actual Frontmatter Version | SHA Claim | Status |
|----------|-----------------|---------------------------|-----------|--------|
| prd.md | v1.21 | "1.21" | 0f124a9 | MATCH |
| verification-properties.md | v1.31 | "1.31" | a3a68a4 | MATCH |
| SS-daemon-lifecycle.md | v1.0.21 | "1.0.21" | 42504b4 | MATCH |
| SS-deps-pin-manifest.md | v1.1.13 | "1.1.13" | 42504b4 | MATCH |
| product-brief.md | v1.4.23 | "1.4.23" | — | MATCH |
| domain-monocle-vision-synthesis.md | v1.1.2 | "1.1.2" | — | MATCH |

CLAUDE.md lines 22 and 47 cite `brief v1.4.23` — both MATCH the brief frontmatter.

---

## §Findings

### GAP-R37-001

**Severity:** HIGH
**Dimension:** 9 (CLAUDE.md vision cite) and 10 (brief/vision/ADR consistency)
**File:Line:** `/Users/jmagady/Dev/monocle/CLAUDE.md:48`

**Evidence (literal):**

```
Line 48: 7. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.1 — re-approved 2026-05-12 with refreshed endpoint set (canonical 5) and tech-stack pointer to `SS-deps-pin-manifest.md`. Captures human intent including all JC/EX/OQ-M closures. v1.1.1 is a surgical patch of v1.1 (path refs + frontmatter date); substantive content unchanged.
```

**Canonical authority:** Vision document frontmatter at
`/Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md`
line 4 reads:

```
version: "1.1.2"
```

and line 29 reads:

```
# Monocle Vision Synthesis (v1.1.2, approved 2026-05-12)
```

**Gap:** CLAUDE.md §Architectural Authority item 7 cites vision `v1.1.1`. The
canonical vision document frontmatter shows the current version is `v1.1.2`.
The CLAUDE.md text additionally describes `v1.1.1` as "a surgical patch of v1.1
(path refs + frontmatter date); substantive content unchanged" — this accurately
describes the v1.1 → v1.1.1 change, but the document has since been updated to
v1.1.2. The citation is one version stale.

**Context:** The vision was at v1.1.1 per commit `aa852b9` (`docs(claude-md):
brief v1.4.1 + vision v1.1 approved; R-001 informational`) but was subsequently
updated to v1.1.2. The CLAUDE.md was not updated to reflect the vision version
bump.

**Impact:** CLAUDE.md §Architectural Authority is the canonical precedence list
read by every AI agent operating on this project. A stale version cite in this
list means an agent reading CLAUDE.md to determine vision authority may refer to
a version description that no longer matches the actual document. While the
substantive content is described as unchanged from v1.1, the version discrepancy
violates the pin coherence discipline (dimension 1) and creates a potential
confusion vector.

**Proposed routing:** `vsdd-factory:state-manager` owns CLAUDE.md per the
routing table (`.factory/STATE.md` updates, cycle bookkeeping). However, CLAUDE.md
§Architectural Authority is content that reflects human-approved decisions. The
correct routing is to surface this to the orchestrator — the fix is a mechanical
version number update (`v1.1.1` → `v1.1.2`) in CLAUDE.md line 48, which the
state-manager can execute as a single-character change. Product-owner confirmation
is not needed since this is a version-pin correction on an approved-verbatim
artifact, not a scope change.

**Remediation:** Update CLAUDE.md line 48:
- From: `v1.1.1 — re-approved 2026-05-12 with refreshed endpoint set (canonical 5) and tech-stack pointer to \`SS-deps-pin-manifest.md\`. Captures human intent including all JC/EX/OQ-M closures. v1.1.1 is a surgical patch of v1.1 (path refs + frontmatter date); substantive content unchanged.`
- To: `v1.1.2 — re-approved 2026-05-12 with refreshed endpoint set (canonical 5) and tech-stack pointer to \`SS-deps-pin-manifest.md\`. Captures human intent including all JC/EX/OQ-M closures. v1.1.2 is the current canonical version; v1.1.1 was a surgical patch of v1.1 (path refs + frontmatter date); substantive content unchanged.`

(Or simply update the version cite and remove the historical description of
v1.1.1 as unnecessary in a current-pointer field.)

---

## §PASS Dimensions

All 10 dimensions were audited. The following dimensions are confirmed CLEAN:

### Dimension 1 — Cross-Artifact Pin Coherence (CLEAN, except GAP-R37-001)

- PRD v1.21 frontmatter `traces_to` cites `SS-daemon-lifecycle.md v1.0.21`,
  `SS-core-types-and-abi.md v1.2.8`, `SS-engine-module.md v1.1.15`,
  `SS-deps-pin-manifest.md v1.1.13`, `vision-synthesis v1.1.2`,
  `product-brief.md v1.4.23` — all match actual artifact frontmatter versions.
- Python `re.MULTILINE` sweep of PRD pre-§Trace body (lines 1–1441): zero
  stale arch version pins found outside frontmatter line 25. All v1.0.21
  citations in body match the canonical current pin.
- VP pre-§Trace body (lines 1–3109): four hits for `v1.0.20` all confirmed as
  historical predecessor chain entries explicitly labeled per PG-5 ("now demoted
  to historical predecessor chain step per PG-5"). No normative-current stale
  pins.
- CLAUDE.md lines 22 and 47 correctly cite `brief v1.4.23`. Line 48 has the
  single finding (GAP-R37-001, vision `v1.1.1` vs actual `v1.1.2`).
- SS-deps-pin-manifest.md `traces_to` cites `brief v1.4 commit 70286e1` and
  `vision v1.1 commit 0e4b0f4` — these are authoring-time provenance references
  (PG-5 historical-anchor framing for `traces_to` fields on architecture docs),
  not current-pointer citations. Consistent with all other SS-* docs which
  uniformly use authoring-time brief references. Not a gap.
- SS-daemon-lifecycle.md `traces_to` cites `brief v1.4.2 Phase 1 Runtime Core
  scope` — same authoring-time pattern. Not a gap.

### Dimension 2 — RTM ↔ VP §Coverage Matrix Coherence (CLEAN)

- All 22 BCs appear in both PRD §7 RTM and VP §Coverage Matrix.
- Test file column matches between PRD §7 RTM and VP §Coverage Matrix for all
  22 rows, including BC-DAEMON-004's two-file citation
  (`graceful_shutdown.rs` + `daemon_lifecycle.rs`) in both documents.
- VP §Mechanism Distribution explicitly documents that VP uses file-layout
  taxonomy labels (`integration-test`, `ast-audit`, `compile-time-check`) while
  PRD §7 RTM uses conceptual test-type labels (`Integration`, `AST audit (syn 2)`,
  `Lint/compile`). This intentional vocabulary divergence is noted at VP lines
  170–174: "this VP catalog's Mechanism column is the per-VP verification
  mechanism whose label matches the file-layout taxonomy rather than the
  conceptual test-type column." Not a gap.
- NFR-012 row present in PRD §7 RTM and correctly references VP-DAEMON-005
  Post-condition 9 / probe 5.e.

### Dimension 3 — BC Anchor Resolution (CLEAN)

- VP §Purpose cites `PRD v1.21 (commit 0f124a9)` — matches canonical PRD
  frontmatter version and commit SHA.
- Spot-checked VP-RING-001 `preserve_order` precondition (added per I-R94-2):
  VP §Pre-conditions documents that `cargo tree -e features | grep preserve_order`
  must return empty, consistent with SS-deps-pin-manifest.md v1.1.13 which lists
  `serde_json 1.0.149` without `preserve_order` feature. Anchor resolves
  correctly.
- Spot-checked BC-DAEMON-005 §Postconditions postcondition 8 (0o700 runtime-dir
  mode): VP-DAEMON-005 Post-condition 9 cross-references this correctly per the
  F-R79-3 and F-R83-1 lift chain. Anchor resolves correctly.

### Dimension 4 — VP §Purpose SHA / §References Timestamp / §Trace Lineage (CLEAN)

- VP §Purpose (lines 34–35): `PRD v1.21 (commit 0f124a9)` — matches canonical
  pin exactly.
- VP §References intro (line 2834): `current as of timestamp
  2026-05-16T11:30:00Z` — matches VP frontmatter `timestamp:
  2026-05-16T11:30:00Z` exactly (Extension 14 SUB-EXTENSION compliance).
- VP §Trace lineage: v1.29 (`2026-05-16T08:30:00Z`) → v1.30
  (`2026-05-16T10:00:00Z`) → v1.31 (`2026-05-16T11:30:00Z`) — strictly
  monotonically increasing (SE-16b PASS).

### Dimension 5 — Manifest ↔ Arch ↔ VP Triple Pin Coherence (CLEAN)

- VP body cites `SS-deps-pin-manifest.md v1.1.13` at all normative dep-pin
  references (axum 0.8, chrono 0.4, directories 6, tempfile 3, nix 0.30,
  constant_time_eq ^0.3, serde 1, serde_json 1) — all match manifest frontmatter
  `version: "1.1.13"`.
- Manifest dep entries are consistent with VP §Pre-conditions dep citations:
  `serde_json 1.0.149` (EXACT pin), `axum 0.8` (pin as `=0.8.9`), `rand 0.8.6`
  (EXACT pin), `directories 6`, `nix 0.30`, `chrono 0.4` — all match.
- `preserve_order` feature: manifest lists `serde_json 1.0.149` without
  `preserve_order` feature, consistent with VP-RING-001 §Pre-conditions
  precondition requiring empty `cargo tree -e features | grep preserve_order`
  output.

### Dimension 6 — Glossary Completeness (CLEAN)

PRD §10 Glossary contains 19 terms. Spot-checked normatively-used terms against
glossary:

- `DaemonStartError::RuntimeDirUnresolvable` — present (added per O-R91-4).
- `MONOCLE_RUNTIME_DIR` — present (added per O-R91-4).
- `#[non_exhaustive]` — present.
- `VsddFactoryAdapter` — present.
- `OsRng` — present.
- `Phase1Permission` — present.
- `format_version` — present.
- All other key terms spot-checked: MONOCLE_ABI_VERSION, HookEventRecord,
  HookEnvelope, JC-2, monocle-v1:, schema_version, BC, ABI, DTU, FactoryState,
  FactoryAdapter, FC, ClaudeCodeModule, EngineModule — all present.

No normatively-used terms found in body that lack a glossary entry.

### Dimension 7 — EC Anchoring (CLEAN)

PRD §9 EC catalog contains EC-001 through EC-061 (total: 61 ECs with
append-only numbering producing non-contiguous EC-054..EC-061 ordering explained
in §9 header note). All 61 ECs verified to anchor to their correct BC owner:

- EC-001..EC-003: BC-RING-001 ✓
- EC-004..EC-006: BC-AUTH-001 ✓
- EC-007..EC-009: BC-AUTH-002 ✓
- EC-010..EC-012: BC-LOCK-001 ✓
- EC-013..EC-015: BC-ABI-001/BC-ABI-002 ✓
- EC-016..EC-017: BC-TYPES-001 ✓
- EC-018..EC-020: BC-FACTORY-001 ✓
- EC-021..EC-023: BC-FACTORY-002 ✓ (includes EC-061 added per C-R91-1)
- EC-024..EC-026: BC-PROTO-001a/001b ✓
- EC-027..EC-028: BC-PROTO-002 ✓
- EC-029..EC-031: BC-ENGINE-001 ✓
- EC-032..EC-035: BC-ENGINE-002 ✓
- EC-036..EC-037: BC-ENGINE-002-ERR ✓
- EC-038..EC-039: BC-ENGINE-003 ✓
- EC-040..EC-041: BC-DAEMON-001 ✓
- EC-042..EC-044: BC-DAEMON-002 ✓
- EC-045..EC-047: BC-DAEMON-003 ✓
- EC-048..EC-050: BC-DAEMON-004 ✓
- EC-051..EC-053, EC-057..EC-060: BC-DAEMON-005 ✓
- EC-054..EC-056: BC-DAEMON-006 ✓

No ECs anchored to wrong BC owner.

### Dimension 8 — NFR-to-VP Coverage (CLEAN)

12 NFR rows in PRD §4 table audited against Extension 16 backfill sweep
(§Trace v1.15 per-row disposition table):

| NFR | VP Coverage | Status |
|-----|-------------|--------|
| NFR-001 | Phase 3 deferred (§G-6) | CLEAN — no VP probe; documented absence per Extension 16 disposition (b) |
| NFR-002 | Phase 3 deferred (§G-6) | CLEAN — no VP probe; documented absence |
| NFR-003 | Phase 3 deferred (§G-6) | CLEAN — no VP probe; documented absence |
| NFR-004 | VP-AUTH-001 §Pre-conditions + §Post-condition 1 | CLEAN — VP probe cited in Validation Method |
| NFR-005 | VP-DAEMON-003 §Post-condition 1 | CLEAN — VP probe cited |
| NFR-006 | Phase 3 deferred (§G-7) | CLEAN — no VP probe; documented absence |
| NFR-007 | CI matrix check | CLEAN — no VP probe needed (CI gate) |
| NFR-008 | GitHub Actions CI matrix | CLEAN — no VP probe needed (CI gate) |
| NFR-009 | VP-DAEMON-005 Post-condition 1 | CLEAN — VP probe cited (Obs-R84-1 closure) |
| NFR-010 | VP-AUTH-001 §Post-condition 5 | CLEAN — VP probe cited |
| NFR-011 | dtu-assessment.md procedure | CLEAN — DTU fidelity covered by separate procedure |
| NFR-012 | VP-DAEMON-005 Post-condition 9 / probe 5.e | CLEAN — VP probe cited (F-R83-1 closure) |

The four latency/throughput NFRs (NFR-001/002/003/006) have no VP probe
citations because their VPs (VP-LATENCY-001/002/003 and VP-THROUGHPUT-001) are
explicitly deferred to Phase 3 in VP §G-6 and §G-7 with concrete
future-attachments. This is the documented correct disposition per the PRD
§Trace v1.15 Extension 16 backfill discipline.

### Dimension 9 — CLAUDE.md Brief and Vision Cites (PARTIAL — see GAP-R37-001)

- CLAUDE.md line 22: `Brief: \`v1.4.23\`` — MATCH.
- CLAUDE.md line 47: `\`.factory/specs/product-brief.md\` v1.4.23` — MATCH.
- CLAUDE.md line 48: `\`.factory/specs/research/domain-monocle-vision-synthesis.md\` v1.1.1`
  — STALE. Actual vision version is `v1.1.2`. → **GAP-R37-001**.

### Dimension 10 — Brief / Vision / ADR Consistency (CLEAN, except GAP-R37-001)

- ADR-0001 (wasmtime vs wasmi): status `accepted`; content consistent with
  manifest EXACT pin on `wasmtime 44` and brief Phase 3 WASM SDK scope. No
  contradiction.
- ADR-0002 (nucleo dormancy): status `accepted`; caret pin `nucleo 0.5` in
  manifest consistent with ADR decision; TD-001 retirement consistent with
  brief §Tech stack removing nucleo from TD register. No contradiction.
- ADR-0003 (MIT OR Apache-2.0 dual license): status `accepted`; brief does not
  contradict. No contradiction.
- ADR-0004 (exhaustive enums): status `accepted`; brief v1.4.8 cited in brief
  §Revision History correctly notes the `Phase1Permission` + `ClaudeCodeTool`
  exhaustive-enum exemption. SS-core-types-and-abi.md v1.2.8 `traces_to`
  references ADR-0004 correctly. PRD §3 BC-TYPES-001 EC-017 `ClaudeCodeTool::Unknown(String)`
  catch-all consistent with ADR-0004 exhaustive-enum exemption. No contradiction.
- Vision v1.1.2 §Five Planes and §Process Topology consistent with PRD §1.2
  Vision block and §1.4 target users. No contradiction.
- Vision `v1.1.2` is cited as `v1.1.1` in CLAUDE.md — GAP-R37-001 covers this.
  The vision content itself is internally consistent at v1.1.2.

---

## §Open Observations

None. All process-gap items resolved into the single HIGH finding above or
confirmed CLEAN.

---

## §Validation Gate Result

**GATE: FAIL (1 HIGH finding)**

GAP-R37-001 (CLAUDE.md line 48 vision version cite `v1.1.1` vs actual `v1.1.2`)
is a blocking finding. The D-047 strict gate definition requires zero findings of
any severity for a CLEAN pass; this HIGH finding prevents advancing the counter.

**D-047 strict counter: 0/3** (this pass does not advance the counter).

---

## §Consistency Score

Dimensions audited: 10 of 10
Dimensions fully clean: 9 of 10 (Dimension 9 partial — single cite stale)
Findings: 1 HIGH, 0 MED, 0 LOW

**Score: 98/100** (1 HIGH finding against 10 fully-audited dimensions)

---

## §Audit Method Notes

- Artifact pin verification: direct Read of frontmatter fields for all 6 canonical
  artifacts.
- PRD stale-pin sweep: Python `re` with `re.MULTILINE` over pre-§Trace body
  (lines 1–1441). Zero stale arch version hits outside frontmatter.
- VP stale-pin sweep: Python `re` over pre-§Trace body (lines 1–3109). All
  `v1.0.20` hits confirmed as PG-5-framed historical predecessor entries.
- CLAUDE.md vision cite: direct grep for `v1\.1\.` hits, confirmed line 48 is
  stale.
- RTM ↔ VP Coverage Matrix: side-by-side read of PRD §7 (lines 1276–1300) and
  VP §Coverage Matrix (lines 2503–2527). All 22 rows verified.
- NFR-to-VP: read PRD §4 NFR table (lines 1217–1231) and VP §G-6/§G-7 entries.
  Extension 16 disposition table in PRD §Trace v1.15 (lines 2469–2488)
  confirmed correct.
- EC anchoring: read PRD §9 EC Catalog (lines 1342–1411). All 61 ECs confirmed
  against BC owner column.
- Vision version: direct Read of vision frontmatter (line 4) and title heading
  (line 29).
