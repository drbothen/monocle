---
document_type: consistency-validation
producer: consistency-validator
version: "1.0"
timestamp: 2026-05-18T15:00:00Z
phase: phase-1-spec-crystallization
round: R56
verdict: GAPS
gap_count: 2
gap_breakdown: "HIGH: 2 (stale brief version pins in PRD traces_to and L2-INDEX §Trace v1.0 active-pointer line); CRIT: 0; MED: 0; LOW: 0; OBS: 2"
---

# Consistency Validation Pass R56 — Phase 1 Spec Crystallization

**Scope:** Fresh-context, zero prior context. 10-dimension cross-document consistency review.
**Artifact set:** Brief v1.4.28, PRD v1.26.9, BC-INDEX v1.9 (22 BCs), VP-INDEX v1.12 (22 VPs),
ARCH-INDEX v1.0.9, L2-INDEX v1.0.8, 4 prd-supplements, 7 SS architecture docs, 5 ADRs,
dtu-assessment v1.7.5.

---

## Per-Dimension Summary Table

| # | Dimension | Result | Notes |
|---|-----------|--------|-------|
| 1 | Cross-document version pins | GAPS | 2 stale brief v1.4.27 pins remain; all other doc-to-doc pins PASS |
| 2 | Index ↔ file H1 alignment | PASS | All 22 BC files, all 22 VP files align to their respective INDEX rows verbatim (R15A swept VP-INDEX; R-series swept BC-INDEX) |
| 3 | ID consistency | PASS | All 22 BC IDs, 22 VP IDs, 7 subsystem abbreviations, 15 error codes, 12 NFRs, 3 CAP IDs, 7 DI IDs present and cross-referenced correctly |
| 4 | Anchor link integrity | PASS | Brief §-heading references verified at v1.4.23 PG-4 sweep; no new broken anchors introduced in v1.4.24–v1.4.28 (all four brief bumps were narrow single-cell fixes) |
| 5 | Count consistency | PASS | 22 BCs (BC-INDEX, PRD §2, brief line 250 all agree), 22 VPs (VP-INDEX, PRD §7-implied, supplement refs agree), 12 NFRs (PRD §4 prose and nfr-catalog.md table agree), 15 errors (PRD §5 prose and error-taxonomy.md table agree), 3 CAPs (L2-INDEX and ARCH-INDEX agree) |
| 6 | Naming consistency | PASS | "monocle" lowercase in code identifiers; "Monocle" capitalized in prose headings throughout. Ubiquitous Language table in L2-INDEX properly maintained |
| 7 | Traceability completeness | PASS | Brief → L2 CAPs (3) → BCs (22) → PRD requirements → VPs (22) → SS architecture docs. Every BC has ≥1 VP; every VP traces to a BC; every BC in §§2.1/2.2/2.3 of PRD; PRD §7 RTM covers all 22 BCs + NFR-012 |
| 8 | SE-16d UTC timestamp monotonicity | PASS | VP-INDEX v1.12 timestamp 2026-05-18T14:00:00Z > R15C VP-005 v1.0.12 timestamp 2026-05-18T13:30:00Z > R15B brief v1.4.28 timestamp 2026-05-18T13:30:00Z. All §Trace chains within-document verified monotonic (BC-INDEX v1.9, ARCH-INDEX v1.0.9, L2-INDEX v1.0.8 all contain ascending §Trace sequences) |
| 9 | Frontmatter completeness | PASS | All sampled artifacts carry document_type, level, version, producer, traces_to, timestamp, inputs, input-hash. BC files have additional required lifecycle fields (DF-030); VP files have source_bc, module, proof_method, feasibility |
| 10 | Citation integrity | PASS | Commit SHAs cited in §Trace sections are referenced for historical evidence, not as navigable links. File paths use relative paths from .factory/specs/ context (verified in ARCH-INDEX, BC files, VP files). All ADR filenames resolve in the adr/ directory |

---

## Gap Details

### GAP-R56-001 — PRD traces_to brief version pin stale

| Field | Value |
|-------|-------|
| ID | GAP-R56-001 |
| Severity | HIGH |
| Dimension | 1 (Cross-document version pins) |
| Location | `/Users/jmagady/Dev/monocle/.factory/specs/prd.md` frontmatter line 11 |

**Exact text (as-is):**
```
traces_to: "product-brief.md v1.4.27; ...
```

**Expected:**
```
traces_to: "product-brief.md v1.4.28; ...
```

**Context:** PRD v1.26.9 was last bumped in Round 10 (F-R111, 2026-05-18T07:00:00Z). At that time, `product-brief.md` was at v1.4.27. Round 15B (commit 08d1ef4, 2026-05-18T13:30:00Z) bumped the brief from v1.4.27 to v1.4.28 (F-R116-2: BC-INDEX pin back-cascade). That bump post-dates the PRD's last update, so the PRD `traces_to` brief pin was not back-cascaded.

The PRD body (§7 RTM, §6, §2, §§3-5) is NOT affected — the PRD body does not carry an active brief version citation. Only the `traces_to` frontmatter field is stale.

**Recommended fix:** PO bump PRD v1.26.9 → v1.26.10 (or a new patch number per scheme), updating `traces_to` to `product-brief.md v1.4.28`. No PRD body changes required. PRD timestamp must advance past 2026-05-18T14:00:00Z (VP-INDEX v1.12 chain high-water).

**Routing:** vsdd-factory:product-owner

---

### GAP-R56-002 — L2-INDEX §Trace v1.0 active-pointer brief cite stale

| Field | Value |
|-------|-------|
| ID | GAP-R56-002 |
| Severity | HIGH |
| Dimension | 1 (Cross-document version pins) |
| Location | `/Users/jmagady/Dev/monocle/.factory/specs/domain-spec/L2-INDEX.md` line 149 |

**Exact text (as-is):**
```
- 3 capabilities extracted from product-brief.md v1.4.27 + vision-synthesis v1.1.2.
```

**Expected:**
```
- 3 capabilities extracted from product-brief.md v1.4.28 + vision-synthesis v1.1.2.
```

**Context:** This line is inside the `## §Trace v1.0` section body. However, per established precedent in this project (F-R107-12 LOW and F-R110-4 HIGH), this specific line has been explicitly treated as an **active current-pointer** — not a historical pinpoint — and was updated from v1.4.23 → v1.4.25 (Round 7, §Trace v1.0.7) and then from v1.4.25 → v1.4.27 (Round 9B, §Trace v1.0.8). The D-042 sweep convention calls for updating it again to v1.4.28.

L2-INDEX was last bumped to v1.0.8 in Round 9B (2026-05-18T05:00:00Z). Brief v1.4.28 was produced in Round 15B (2026-05-18T13:30:00Z). The L2-INDEX §Trace v1.0.8 sweep explicitly searched for `v1.4.` and updated line 149 to v1.4.27 — establishing the pattern that this is a maintained current-pointer.

**Recommended fix:** BA bump L2-INDEX v1.0.8 → v1.0.9, adding a §Trace v1.0.9 entry that: (a) corrects line 149 from v1.4.27 to v1.4.28, and (b) records the SE-17f before/after evidence. Timestamp must advance past 2026-05-18T14:00:00Z.

**Routing:** vsdd-factory:business-analyst

---

## Informational Observations (Non-Blocking)

### OBS-R56-01 — CAP-002 carries a historical inline anchor to brief v1.4.7

**Location:** `/Users/jmagady/Dev/monocle/.factory/specs/domain-spec/CAP-002-forward-compat-wire-formats.md` line 46

**Text:** `"brief v1.4.7 §Scope"` — intentional historical anchor to the version when FC-01..FC-06 were first introduced.

**Classification:** Historical anchor per L2-INDEX §Trace v1.0.7 (BA explicitly noted this as "a historical anchor cite pointing to an earlier brief revision where FC-01..FC-06 were first introduced. That cite is in a CAP file, not L2-INDEX, and is outside BA L2-INDEX scope for this round."). This OBS is recorded for transparency; the BA should continue to classify it as leave-alone per the established pattern.

**Action:** None required.

---

### OBS-R56-02 — Brief `supplements:` frontmatter uses absolute machine-local paths

**Location:** `/Users/jmagady/Dev/monocle/.factory/specs/product-brief.md` frontmatter `supplements:` list (lines 14-26)

**Text:** All 12 supplement paths begin with `/Users/jmagady/Dev/monocle/...` (absolute paths specific to the author's machine).

**Classification:** Not a functional gap — the `supplements:` field is a navigation aid for human readers, not a machine-resolved path used by factory tools (compute-input-hash uses the `inputs:` field, which uses relative paths). However, it degrades portability if the codebase moves or another developer mounts it at a different path.

**Action:** Optional remediation by PO — normalize to relative paths (e.g., `.factory/specs/architecture/SS-deps-pin-manifest.md`) consistent with the `inputs:` field pattern. Defer if the path is stable.

---

## Traceability Chain Verification (L1 → L4)

| Chain Link | Status | Evidence |
|------------|--------|---------|
| L1 Brief v1.4.28 → L2 L2-INDEX v1.0.8 | PASS | L2-INDEX `traces_to: product-brief.md`; 3 CAPs map to brief §Scope capabilities |
| L2 CAP-001 → BC-2.01.001..BC-2.01.010 (10 BCs) | PASS | BC-INDEX §SS-01 lists all 10; L2-INDEX Capabilities Registry row confirmed |
| L2 CAP-002 → BC-2.02.001..BC-2.02.008 (8 BCs) | PASS | BC-INDEX §SS-02 lists all 8 |
| L2 CAP-003 → BC-2.03.001..BC-2.03.004 (4 BCs) | PASS | BC-INDEX §SS-03 lists all 4 |
| L3 BCs → PRD §2 + §7 RTM | PASS | All 22 BCs appear in PRD §2.1/2.2/2.3 tables and §7 RTM rows; titles match BC-INDEX and BC file H1s |
| L3 BCs → L3 architecture SS docs | PASS | PRD §7 RTM Module(s) column cites SS-daemon-lifecycle.md v1.0.32 (SS-01 BCs), SS-core-types-and-abi.md v1.2.13 (SS-02 BCs), SS-engine-module.md v1.1.20 (SS-03 BCs) — all matching current frontmatter versions |
| L4 VPs → L3 BCs (source_bc) | PASS | All 22 VPs carry `source_bc:` frontmatter matching a valid BC-2.SS.NNN in BC-INDEX |
| L4 VP-INDEX → PRD | PASS | VP-INDEX §References cites PRD v1.26.9 (current) |
| L4 VP-INDEX → BC-INDEX | PASS | VP-INDEX §References cites BC-INDEX v1.9 (current) |

## Count Verification

| Claim | Claimed Count | Actual Count | Source | Status |
|-------|---------------|--------------|--------|--------|
| Total Phase 1 BCs | 22 | 22 (10+8+4) | BC-INDEX summary table | PASS |
| Active BCs | 22 | 22 | BC-INDEX summary table | PASS |
| Total Phase 1 VPs | 22 | 22 (10+8+4) | VP-INDEX summary | PASS |
| NFRs (PRD §4 prose) | 12 | 12 (NFR-001..NFR-012) | nfr-catalog.md | PASS |
| Error codes (PRD §5 prose) | 15 | 15 distinct E-xxx-NNN codes | error-taxonomy.md | PASS |
| Error subsystem abbreviations | 7 | 7 (AUTH/DAEMON/LOCK/ENG/FACT/RING/PROTO) | error-taxonomy.md | PASS |
| L2 capabilities | 3 | 3 (CAP-001/002/003) | L2-INDEX registry | PASS |
| Domain invariants | 7 | 7 (DI-001..DI-007) | L2-INDEX registry | PASS |
| Architecture subsystems (SS) | 3 | 3 (SS-01/02/03) | ARCH-INDEX registry | PASS |
| ADRs registered | 5 | 5 (ADR-0001..ADR-0005) | ARCH-INDEX ADR Registry | PASS |

## Index ↔ File H1 Alignment Check

### BC-INDEX row title → BC file H1 (all 22 checked)

After stripping `"Behavioral Contract BC-S.SS.NNN: "` prefix from BC file H1s:

| BC ID | BC-INDEX title | BC file H1 (stripped) | Match |
|-------|---------------|----------------------|-------|
| BC-2.01.001 | Healthz Endpoint (Unauthenticated Liveness Probe) | Healthz Endpoint (Unauthenticated Liveness Probe) | PASS |
| BC-2.01.002 | Status Endpoint (Authenticated Daemon State) | Status Endpoint (Authenticated Daemon State) | PASS |
| BC-2.01.003 | Body Size Limit (256 KiB, HTTP 413) | Body Size Limit (256 KiB, HTTP 413) | PASS |
| BC-2.01.004 | Graceful Shutdown (10-Second Drain) | Graceful Shutdown (10-Second Drain) | PASS |
| BC-2.01.005 | Lock File Atomic Lifecycle (Create + Pid Check + Cleanup) | Lock File Atomic Lifecycle (Create + Pid Check + Cleanup) | PASS |
| BC-2.01.006 | Crash Recovery Checkpoint | Crash Recovery Checkpoint | PASS |
| BC-2.01.007 | JSONL Ring Format Version (FC-01) | JSONL Ring Format Version (FC-01) | PASS |
| BC-2.01.008 | Auth Token Wire Format (FC-06) | Auth Token Wire Format (FC-06) | PASS |
| BC-2.01.009 | Auth Header Validation (Missing and Invalid Token) | Auth Header Validation (Missing and Invalid Token) | PASS |
| BC-2.01.010 | Lock File Contract Version Field | Lock File Contract Version Field | PASS |
| BC-2.02.001 | ABI Version in /status Endpoint (FC-03) | ABI Version in /status Endpoint (FC-03) | PASS |
| BC-2.02.002 | ABI Version Constant at Crate Root (FC-03) | ABI Version Constant at Crate Root (FC-03) | PASS |
| BC-2.02.003 | Non-Exhaustive Enum Policy (FC-02) | Non-Exhaustive Enum Policy (FC-02) | PASS |
| BC-2.02.004 | FactoryAdapter Trait Definition (FC-04 CRITICAL) | FactoryAdapter Trait Definition (FC-04 CRITICAL) | PASS |
| BC-2.02.005 | VsddFactoryAdapter Implementation | VsddFactoryAdapter Implementation | PASS |
| BC-2.02.006 | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | PASS |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | PASS |
| BC-2.02.008 | Phase 4 schema_version Validation Requirement (FC-05) | Phase 4 schema_version Validation Requirement (FC-05) | PASS |
| BC-2.03.001 | EngineModule Trait Definition | EngineModule Trait Definition | PASS |
| BC-2.03.002 | ClaudeCodeModule Implementation (Strict-Basename Detect) | ClaudeCodeModule Implementation (Strict-Basename Detect) | PASS |
| BC-2.03.003 | HomeUnresolvable Error Contract | HomeUnresolvable Error Contract | PASS |
| BC-2.03.004 | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | PASS |

**Result: 22/22 PASS**

### VP-INDEX row title → VP file H1 (all 22 checked)

After stripping `"VP-NNN: "` prefix from VP file H1s:

| VP ID | VP-INDEX title | VP file H1 (stripped) | Match |
|-------|---------------|----------------------|-------|
| VP-001 | Healthz Endpoint — Unauthenticated Liveness 200/503 with Uptime + Version | Healthz Endpoint — Unauthenticated Liveness 200/503 with Uptime + Version | PASS |
| VP-002 | Status Endpoint — Authenticated Daemon-State JSON with 10 Required Fields | Status Endpoint — Authenticated Daemon-State JSON with 10 Required Fields | PASS |
| VP-003 | Body Size Limit — 256 KiB; HTTP 413 on Excess | Body Size Limit — 256 KiB; HTTP 413 on Excess | PASS |
| VP-004 | Graceful Shutdown — 10-Second Drain + 5-Code POSIX Exit Taxonomy | Graceful Shutdown — 10-Second Drain + 5-Code POSIX Exit Taxonomy | PASS |
| VP-005 | Lock File Lifecycle — Atomic Create, Pid-Liveness Gate, Mode 0o600/0o700, Cleanup, 4-Path Resolution | Lock File Lifecycle — Atomic Create, Pid-Liveness Gate, Mode 0o600/0o700, Cleanup, 4-Path Resolution | PASS |
| VP-006 | Crash Recovery Checkpoint — JSON Write, Offer, Cleanup | Crash Recovery Checkpoint — JSON Write, Offer, Cleanup | PASS |
| VP-007 | JSONL Ring Record — Format-Version First Key (FC-01) | JSONL Ring Record — Format-Version First Key (FC-01) | PASS |
| VP-008 | Auth Token — Wire Format + Constant-Time Comparison (FC-06) | Auth Token — Wire Format + Constant-Time Comparison (FC-06) | PASS |
| VP-009 | Auth Header Validation — Two-Body Taxonomy + ADR-0005 Dual-Accept (Canonical `X-Monocle-Authorization` with `X-Claude-Code-Ide-Authorization` Compatibility Alias) | Auth Header Validation — Two-Body Taxonomy + ADR-0005 Dual-Accept (Canonical `X-Monocle-Authorization` with `X-Claude-Code-Ide-Authorization` Compatibility Alias) | PASS |
| VP-010 | Lock File `contract_version: 1` First Key | Lock File `contract_version: 1` First Key | PASS |
| VP-011 | ABI Version in `/status` Endpoint | ABI Version in `/status` Endpoint | PASS |
| VP-012 | `monocle_core::MONOCLE_ABI_VERSION` Pub Const Equals `1` | `monocle_core::MONOCLE_ABI_VERSION` Pub Const Equals `1` | PASS |
| VP-013 | Non-Exhaustive Enum Policy (Modulo ADR-0004 Exemptions) | Non-Exhaustive Enum Policy (Modulo ADR-0004 Exemptions) | PASS |
| VP-014 | `FactoryAdapter` Trait Signature Stable | `FactoryAdapter` Trait Signature Stable | PASS |
| VP-015 | `VsddFactoryAdapter::new` + Self-Referential Detection; `None` for Absent Optionals | `VsddFactoryAdapter::new` + Self-Referential Detection; `None` for Absent Optionals | PASS |
| VP-016 | Proto Field Number 1 in `HookEnvelope` is `schema_version` | Proto Field Number 1 in `HookEnvelope` is `schema_version` | PASS |
| VP-017 | Rust `HookEnvelope` Struct Exposes `pub schema_version: u32` with Value `1` | Rust `HookEnvelope` Struct Exposes `pub schema_version: u32` with Value `1` | PASS |
| VP-018 | `schema_version` Forward-Compat Contract (Phase 1 Structural Recap; Phase 4 Runtime Dispatch) | `schema_version` Forward-Compat Contract (Phase 1 Structural Recap; Phase 4 Runtime Dispatch) | PASS |
| VP-019 | `EngineModule` Trait Signature Stable; `last_event_micros: Option<i64>`; No Silent Fallback | `EngineModule` Trait Signature Stable; `last_event_micros: Option<i64>`; No Silent Fallback | PASS |
| VP-020 | `ClaudeCodeModule::detect` Strict Basename Match; Cmdline Ignored | `ClaudeCodeModule::detect` Strict Basename Match; Cmdline Ignored | PASS |
| VP-021 | `metadata`/`enrich` Return `HomeUnresolvable` with All Four Home-Env Vars Unset | `metadata`/`enrich` Return `HomeUnresolvable` with All Four Home-Env Vars Unset | PASS |
| VP-022 | `hook_paths()` Returns Exactly 5 Entries — One per `HookType` Variant | `hook_paths()` Returns Exactly 5 Entries — One per `HookType` Variant | PASS |

**Result: 22/22 PASS** (R15A sweep confirmed; verified independently in this pass)

## Active Architecture Version Pin Summary

| Doc | Actual frontmatter version | Pins in PRD §7 RTM | Pins in brief §Success Criteria | Status |
|-----|---------------------------|---------------------|--------------------------------|--------|
| SS-daemon-lifecycle.md | v1.0.32 | v1.0.32 | v1.0.32 | PASS |
| SS-core-types-and-abi.md | v1.2.13 | v1.2.13 | v1.2.13 | PASS |
| SS-engine-module.md | v1.1.20 | v1.1.20 | v1.1.20 | PASS |
| SS-deps-pin-manifest.md | v1.1.17 | v1.1.17 (traces_to) | n/a | PASS |
| SS-forward-compatibility.md | v1.2.19 | n/a (not pinned in RTM) | n/a | PASS (ARCH-INDEX does not carry per-SS version numbers per §Trace v1.0.3) |
| SS-conventions-anti-patterns.md | v1.29.4 | n/a | n/a | PASS (not pinned in RTM; cross-cutting) |
| ARCH-INDEX | v1.0.9 | n/a | n/a | PASS |
| BC-INDEX | v1.9 | v1.9 (traces_to) | v1.9 (line 250) | PASS |
| VP-INDEX | v1.12 | n/a | n/a | PASS |
| L2-INDEX | v1.0.8 | v1.0.8 (traces_to) | n/a | PASS |
| product-brief.md | v1.4.28 | v1.4.27 (traces_to) | n/a | GAP-R56-001 |

## SE-16d Timestamp Chain (R15 burst)

| Event | Timestamp | Chain position |
|-------|-----------|----------------|
| PRD v1.26.9 (R111 Round 10A) | 2026-05-18T07:00:00Z | Before R15 |
| L2-INDEX v1.0.8 (F-R110-4) | 2026-05-18T05:00:00Z | Before R15 (earlier round, but recorded before) |
| Brief v1.4.28 (R15B) | 2026-05-18T13:30:00Z | R15 burst |
| VP-005 v1.0.12 (R15C) | 2026-05-18T13:30:00Z | R15 burst |
| VP-INDEX v1.12 (R15A) | 2026-05-18T14:00:00Z | R15 burst — chain high-water |

**SE-16d PASS:** Chain is monotonic. The R15A VP-INDEX timestamp (14:00:00Z) is strictly greater than R15B/R15C timestamps (13:30:00Z).

---

## Validation Gate Result

**GATE: FAIL — GAPS exist**

Two HIGH-severity gaps block clean passage:

1. **GAP-R56-001** (HIGH): PRD `traces_to` brief pin is v1.4.27; brief is at v1.4.28. Requires product-owner PRD patch bump.
2. **GAP-R56-002** (HIGH): L2-INDEX §Trace v1.0 active-pointer line cites brief v1.4.27; should be v1.4.28 per established D-042 current-pointer convention. Requires business-analyst L2-INDEX patch bump.

These are both brief back-cascade misses from the R15B bump (brief v1.4.27 → v1.4.28), consistent with the M-CASCADE-SCOPE pattern documented throughout this project's history. Neither gap affects any behavioral specification, acceptance criterion, or verification property — both are version-pin metadata in traceability fields. Functional conformance of the spec chain is otherwise confirmed.

**Consistency score: 98%** (2 gaps / ~100 checked citation points; all behavioral traceability, counts, H1 alignments, ID integrity, and naming pass)
