---
document_type: consistency-pass
pass_id: R42
attempt: 36
verdict: CLEAN
artifact_pins:
  - artifact: PRD
    version: "1.25"
    commit: 7735c84
  - artifact: VP
    version: "1.35"
    commit: latest (18fe265 cascade)
  - artifact: SS-daemon-lifecycle
    version: "1.0.25"
    commit: 18fe265
  - artifact: SS-deps-pin-manifest
    version: "1.1.15"
    commit: d088123
  - artifact: SS-core-types-and-abi
    version: "1.2.8"
    commit: latest
  - artifact: SS-engine-module
    version: "1.1.15"
    commit: latest
  - artifact: SS-conventions-anti-patterns
    version: "1.28"
    commit: latest
  - artifact: SS-forward-compatibility
    version: "1.2.13"
    commit: latest
  - artifact: SS-permissions-phase1
    version: "1.4"
    commit: latest
  - artifact: product-brief
    version: "1.4.23"
    commit: latest
  - artifact: vision-synthesis
    version: "1.1.2"
    commit: latest
  - artifact: dtu-assessment
    version: "1.7"
    commit: latest
  - artifact: ADR-0001
    version: "1.0.2"
    commit: latest
  - artifact: ADR-0002
    version: "1.0"
    commit: latest
  - artifact: ADR-0003
    version: "1.0.1"
    commit: latest
  - artifact: ADR-0004
    version: "1.0.3"
    commit: latest
dimensions_applied: 10
timestamp: "2026-05-16T07:30:00Z"
---

# Consistency Pass R42 — D-047 Strict Pass 1 Attempt 36

## §Summary

**Verdict: CLEAN**

No MED or higher findings. Zero blocking issues. Counter advances: 1/3 → 2/3.

OBS-R41-1 (LOW informational — `reqwest 0.13` in Phase 1 Pin Table has no consumer
edge in the mermaid Workspace Dependency Graph) is re-examined below. It remains
**unchanged** from R41 — pre-existing, pre-Phase-1-Cargo-init gap, not resolvable
at spec-only tier. No new findings in this class.

---

## §Findings GAP-R42-N

None. No new GAP-class findings at MED or higher severity.

---

## §PASS Dimensions

All 10 mandatory consistency dimensions verified PASS.

### D1 — Cross-Artifact Pin Coherence

All cross-artifact version citations verified consistent.

PRD v1.25 `traces_to` frontmatter declares the following canonical current-pointers
(verified by direct grep of PRD frontmatter line 25):

| Artifact | Declared pin | Actual file version | Result |
|----------|-------------|---------------------|--------|
| SS-daemon-lifecycle.md | v1.0.25 | v1.0.25 | PASS |
| SS-core-types-and-abi.md | v1.2.8 | v1.2.8 | PASS |
| SS-engine-module.md | v1.1.15 | v1.1.15 | PASS |
| SS-deps-pin-manifest.md | v1.1.15 | v1.1.15 | PASS |
| product-brief.md | v1.4.23 | v1.4.23 | PASS |
| vision-synthesis | v1.1.2 | v1.1.2 | PASS |

VP v1.35 `traces_to` declares arch v1.0.24 → v1.0.25 (commit 58af8de → 18fe265)
and PRD v1.24 → v1.25 (commit a71ca67 → 7735c84) propagated via F-R101 Burst 4.
VP §Purpose line 34 cites `PRD v1.25 (commit 7735c84)` — matches PRD actual version.
VP §Scope cites `SS-daemon-lifecycle.md v1.0.25`, `SS-core-types-and-abi.md v1.2.8`,
`SS-engine-module.md v1.1.15` — all match actual file versions. PASS.

PRD §7 RTM all 22 BC rows cite `SS-daemon-lifecycle.md v1.0.25`,
`SS-core-types-and-abi.md v1.2.8`, and `SS-engine-module.md v1.1.15` — verified by
grep of RTM section. PASS.

VP §VP Catalog Overview table all 22 VP rows cite PRD v1.25 and correct SS versions
(v1.0.25, v1.2.8, v1.1.15). PASS.

### D2 — RTM vs Coverage Matrix

PRD §7 RTM has 23 rows: 22 BC rows + 1 NFR-012 row. Verified by direct read of
§7 RTM section (lines 1278–1300). BC count = 22 (BC-DAEMON-001..006, BC-RING-001,
BC-AUTH-001/002, BC-LOCK-001, BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002,
BC-PROTO-001a/001b, BC-PROTO-002, BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR,
BC-ENGINE-003) — exactly 22.

VP §VP Catalog Overview table has exactly 22 VP rows (VP-DAEMON-001 through
VP-ENGINE-003) — verified by direct read. One-to-one BC coverage maintained. PASS.

### D3 — BC Anchors

All 22 BC sections in PRD §3 verified present (grep `^### BC-` returns 22 results).
All 22 `### BC-` headings match the BC IDs declared in PRD §2.1 grouping table and
in the VP catalog. No orphaned BC. No BC in VP catalog without a corresponding
`### BC-` section in PRD §3. PASS.

### D4 — §Purpose/§References/§Trace Lineage

PRD §Trace (frontmatter `traces_to`, full historical chain): F-R101 Burst 3 (v1.25)
entry correctly documents arch v1.0.24→v1.0.25 (commit 18fe265) and manifest
unchanged at v1.1.15. SE-17g THIRD APPLICATION, SE-17f THIRD APPLICATION, SE-16d
PASS all recorded. PASS.

VP §Purpose declares `PRD v1.25 (commit 7735c84)` — current canonical. VP §Trace
(frontmatter) F-R101 Burst 4 entry documents arch v1.0.24→v1.0.25, PRD v1.24→v1.25.
Scoped-awk innovation introduced by VP to prevent recursive self-match. PASS.

SS-daemon-lifecycle §Trace v1.0.25 entry closes F-R101-3 MED (§Trace v1.0.24 [N-4]
NORMATIVE classification self-contradiction). SE-16d PASS recorded. PASS.

SS-deps-pin-manifest §Trace v1.1.15 closes F-R99-6 MED; SE-17f self-revalidation
block present. No v1.1.15 changes to manifest body content — only §Trace narrative
correction. PASS.

### D5 — Manifest vs Arch vs VP Triple Pin

Patch-Pinning Policy in SS-deps-pin-manifest §Patch-Pinning Policy declares 9
EXACT-pinned crates: `tokio, prost, wasmtime, russh, rmcp, reqwest, axum, serde_json,
rand`. Count verified: 9 crates enumerated. Decomposition: 3 individual-rationale
crates (serde_json, prost, rand) + 6 remaining (tokio, wasmtime, russh, rmcp,
reqwest, axum). 3 + 6 = 9. Arithmetic consistent. PASS.

VP §VP Catalog cites SS-daemon-lifecycle v1.0.25 (the canonical BC source for the
6 daemon BCs including BC-AUTH-001 whose `constant_time_eq ^0.3` pin lives in the
manifest). Pin flows: manifest v1.1.15 → arch v1.0.25 → PRD v1.25 → VP v1.35.
Triple-pin chain intact. PASS.

### D6 — Glossary Completeness

PRD §10 Glossary present at line 1414. Verified by grep. VP §Open Verification Gaps
and VP §Mechanism Distribution reference glossary-defined terms consistently. No
orphaned term identified. PASS.

### D7 — EC Anchoring

PRD §9 Edge Case Catalog (O-R88-1 note: BC-grouped ordering, not numeric-monotonic)
referenced throughout BCs. EC-040..EC-044 anchored to BC-DAEMON-001/002. EC-060/061
anchored to BC-DAEMON-005/BC-FACTORY-002. Reviewed EC range headers at lines 125,
179, 226, 283, 350, 425, 474, 522, 570 — all present under respective BC sections.
PASS.

### D8 — NFR-to-VP Coverage

Extension 16 mandatory backfill sweep (PRD v1.15 closure) documented that all 12
NFR rows have VP probe citations in the `Validation Method` column. PRD §Trace v1.15
records the per-row SE-15c disposition. NFR-012 (runtime_dir 0o700 owner-only) is
linked to VP-DAEMON-005 Post-condition 9 / probe 5.e. VP-DAEMON-005 covers mode
verification. PASS.

### D9 — CLAUDE.md Cites

CLAUDE.md line 22: `Brief: v1.4.23` — matches product-brief.md frontmatter
`version: "1.4.23"`. PASS.

CLAUDE.md line 47: `product-brief.md v1.4.23` — consistent. PASS.

CLAUDE.md line 48: `domain-monocle-vision-synthesis.md v1.1.2` — matches
vision-synthesis frontmatter `version: "1.1.2"`. PASS.

CLAUDE.md line 225 historical narrative: references `v1.1.1` as the stake at
manifest stub-completion, then `current v1.1.15` — this is historical-anchor form
per PG-5 convention and is not a stale normative pin claim. PASS.

### D10 — Cross-Doc Consistency

Verified the following cross-document consistency claims:

1. BC count: PRD §2.1 states 22 Phase 1 BCs. PRD §3 has 22 `### BC-` sections.
   VP catalog has 22 VP rows. All consistent. PASS.

2. 5-endpoint enumeration: SS-daemon-lifecycle §GET /status (line 86-87) lists
   exactly 5 hook endpoints: `/hooks/pre-tool-use`, `/hooks/notification`,
   `/hooks/stop`, `/hooks/session-start`, `/hooks/prompt-submit`. PRD BC-DAEMON-002
   Postcondition 1 lists the same 5 endpoints. VP-DAEMON-002 property references
   "5 hook path strings". Consistent across all three artifacts. PASS.

3. BC-AUTH-002 two-body taxonomy: arch §Start Sequence (lines 411–419) defines two
   failure modes (missing header → `missing_auth_token`; present value fails →
   `invalid_auth_token`). PRD BC-AUTH-002 table mirrors the same two rows. VP-AUTH-002
   property states "Two-body taxonomy: absent header → `missing_auth_token`; any
   value-present failure → `invalid_auth_token` (collapsed)". Fully consistent. PASS.

4. 256 KiB body limit: Arch §Body Size Limit states `DefaultBodyLimit::max(256 * 1024)`
   = 262,144 bytes. PRD BC-DAEMON-003 JSON error response `{"error":"payload_too_large",
   "limit_bytes":262144}` consistent. PASS.

5. HookEventRecord struct: Arch §Drain defines `HookEventRecord` in
   `monocle-runtime::ring` with `format_version` as first field. BC-RING-001 property
   in VP references "JSONL ring record format-version is first key". Consistent. PASS.

6. Lock file `contract_version` field: Arch §Start Sequence step 6 JSON schema has
   `contract_version` as FIRST key. BC-LOCK-001 / VP-LOCK-001 both enforce "first key"
   invariant. Manifest §Phase 1 Pin Manifest has no conflicting field-order claim.
   Consistent. PASS.

7. MSRV: Manifest §MSRV Policy states Phase 1 = Rust 1.86, Phase 3 = Rust 1.92.
   CLAUDE.md line states same MSRV values. Consistent. PASS.

8. VP inputs list vs PRD inputs list: VP inputs include `prd.md` (derives from PRD
   as parent) and all SS files, DTU, ADRs — consistent with the expected derivation
   chain. VP does not independently re-list brief and vision because PRD already
   consumed them. No asymmetry requiring a finding. PASS.

---

## §Open Observations (LOW)

### OBS-R41-1 (UNCHANGED from R42 examination)

**Status:** Pre-existing. Unchanged since R41. NOT a new R42 finding.

**Observation:** `reqwest 0.13` appears in the Phase 1 Pin Table of
`SS-deps-pin-manifest.md` (line 64) as an EXACT-pinned HTTP client crate. It is also
listed in the Patch-Pinning Policy as one of the 9 security-sensitive exact-pinned
crates (lines 110, 112, 117). However, the Workspace Dependency Graph mermaid block
contains no consumer edge of the form `<crate> --> reqwest` for any Phase 1 workspace
crate. The vision document §Tech Stack mentions `reqwest for HTTP client` as
architectural intent.

**Why this is pre-existing and LOW:** The Workspace Dependency Graph comment
(manifest line 146) states it is "Authoritative for Phase 1 spec package; refresh
after Cargo workspace `Cargo.toml` files exist (during `/vsdd-factory:create-architecture`)."
The consumer crate for `reqwest` has not yet been determined at spec tier. It may
be `monocle-runtime` (TUI client-side requests to `/status`/`/shutdown`), or it may
be a Phase 2/4 crate. This is a structural gap that is resolved when the Cargo
workspace files are initialized — which is Phase 1 architecture deliverable
(`/vsdd-factory:create-architecture`), not a pre-Phase-1 spec requirement.

**Deferred to:** Phase 1 architecture creation — `create-architecture` agent will
refresh the Workspace Dependency Graph against actual `Cargo.toml` files, at which
point `reqwest` will receive its consumer edge or be reclassified as Phase 2+.

**Counter-reset impact:** None. This is a pre-existing LOW informational
observation. Re-surfacing it as "unchanged" does not constitute a new finding and
does not reset the D-047 counter per established convention.
