---
document_type: domain-spec-index
level: L2
version: "1.0.10"
status: active
producer: vsdd-factory:business-analyst
timestamp: 2026-05-18T22:30:00Z
phase: 1a
inputs:
  - product-brief.md
  - research/domain-monocle-vision-synthesis.md
input-hash: "494c12d"
traces_to: product-brief.md
sections:
  - CAP-001-daemon-lifecycle.md
  - CAP-002-forward-compat-wire-formats.md
  - CAP-003-multi-harness-adapter.md
---

# L2 Domain Specification: monocle

> **Sharded artifact (DF-021).** This index provides navigation and summary.
> Detail lives in per-capability section files listed below. Each section
> targets 800-1,200 tokens for optimal LLM consumption. This L2 spec describes
> the **problem domain** — what monocle addresses, what entities exist, and
> what business invariants hold. Implementation details live in BCs (L3) and
> architecture (L3).

## Domain Summary

monocle addresses the fragmentation problem facing developers who run multiple
AI coding harness sessions concurrently: permission prompts stall while the
developer is in another window, customizations are scattered across project
trees, and workflow state is invisible without manual file reads. monocle
provides a single `Ctrl-\` popup that collapses all of this into one observable
surface — observe-only for state, action-only for permission overlays and
keybinding dispatch — without requiring the developer to leave their editor.

## Document Map

| Section | File | Tokens | Primary Consumer | Purpose |
|---------|------|--------|-----------------|---------|
| Daemon Lifecycle | CAP-001-daemon-lifecycle.md | ~900 | product-owner, architect, story-writer | CAP-001: hook ingestion, JSONL ring, lock file, crash recovery, graceful shutdown |
| Forward-Compatible Wire Formats | CAP-002-forward-compat-wire-formats.md | ~900 | architect, product-owner | CAP-002: FC-01..FC-06 schemas, ABI versioning, protobuf field numbering |
| Multi-Harness Adapter Surface | CAP-003-multi-harness-adapter.md | ~900 | architect, story-writer | CAP-003: EngineModule trait, ClaudeCodeModule, FactoryAdapter trait |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| BC creation input | CAP-001-daemon-lifecycle.md + CAP-002-forward-compat-wire-formats.md + CAP-003-multi-harness-adapter.md |
| Architecture design input | All CAP-NNN files + ARCH-INDEX.md |
| Story decomposition input | CAP-001-daemon-lifecycle.md + CAP-003-multi-harness-adapter.md |
| Forward-compat contract authoring | CAP-002-forward-compat-wire-formats.md + ARCH-INDEX.md |
| Full domain review | All CAP-NNN files |

## Capabilities Registry

| CAP ID | Name | Priority | Subsystem | BC Operationalizations |
|--------|------|----------|-----------|------------------------|
| CAP-001 | Daemon Lifecycle | P0 | SS-01 | BC-2.01.001..BC-2.01.010 (10 BCs) |
| CAP-002 | Forward-Compatible Wire Formats | P0 | SS-02 | BC-2.02.001..BC-2.02.008 (8 BCs) |
| CAP-003 | Multi-Harness Adapter Surface | P0 | SS-03 | BC-2.03.001..BC-2.03.004 (4 BCs) |

## Domain Entities Registry

| Entity | Owning Capability | Section File |
|--------|------------------|-------------|
| HookEvent | CAP-001 | CAP-001-daemon-lifecycle.md |
| HookEventRecord (JSONL) | CAP-001 | CAP-001-daemon-lifecycle.md |
| DaemonLockFile | CAP-001 | CAP-001-daemon-lifecycle.md |
| CrashCheckpoint | CAP-001 | CAP-001-daemon-lifecycle.md |
| HookEnvelope (proto) | CAP-002 | CAP-002-forward-compat-wire-formats.md |
| AbiVersionConst | CAP-002 | CAP-002-forward-compat-wire-formats.md |
| AuthToken | CAP-002 | CAP-002-forward-compat-wire-formats.md |
| EngineModule (trait) | CAP-003 | CAP-003-multi-harness-adapter.md |
| ClaudeCodeModule | CAP-003 | CAP-003-multi-harness-adapter.md |
| FactoryAdapter (trait) | CAP-003 | CAP-003-multi-harness-adapter.md |
| VsddFactoryAdapter | CAP-003 | CAP-003-multi-harness-adapter.md |

## Domain Invariants Summary

Full invariant text lives in each capability file. Summary:

| DI ID | Statement | Owning Capability |
|-------|-----------|------------------|
| DI-001 | Every hook event received by the daemon MUST be written to the JSONL ring before any acknowledgement is returned to the harness | CAP-001 |
| DI-002 | The daemon lock file MUST be present and contain a valid port and auth token before any hook endpoint accepts connections | CAP-001 |
| DI-003 | The auth token MUST be written to the lock file after the port is bound — never before | CAP-001 |
| DI-004 | All public wire types MUST carry a version discriminant as their first field so that readers can detect format evolution without parsing the full record | CAP-002 |
| DI-005 | A monocle daemon MUST NOT accept an auth token that does not begin with the canonical prefix for its version | CAP-002 |
| DI-006 | Every EngineModule implementation MUST be stateless with respect to process detection — `detect()` must not perform I/O and must not mutate shared state | CAP-003 |
| DI-007 | monocle MUST NOT write to any file owned by a harness or factory workflow system | CAP-003 |

## Ubiquitous Language

Terms shared across all monocle artifacts. Use these exact terms in BCs, architecture, and stories.

| Term | Definition | Do NOT say |
|------|------------|------------|
| harness | An AI coding tool (Claude Code, CodeMachine, or future equivalent) that monocle observes | platform, vendor, provider |
| hook | A lifecycle event fired by a harness subprocess via HTTP POST to the daemon's endpoint | event, signal, notification (when referring specifically to the HTTP hook mechanism) |
| daemon | The long-lived background process that receives hooks and brokers them to TUI clients | server, service (acceptable in implementation docs, but "daemon" is the domain term) |
| lock file | The file at `runtime_dir/monocle.lock` that carries `{port, token, contract_version}` for hook-script consumption | pid file, socket file |
| hook event | The structured data payload carried by a single hook HTTP POST | hook payload, hook message |
| JSONL ring | The hybrid RAM + async-flush append-only log of hook events with rotation policy | event log, audit log |
| TUI client | The ratatui process that connects to the daemon and renders the popup | frontend, UI process |
| overlay | The floating permission-prompt panel rendered as a VecDeque-backed cascade | popup, dialog, modal (acceptable in implementation; "overlay" is the domain term) |
| factory | A project that uses a `document_type: pipeline-state` STATE.md workflow file (vsdd-factory or compatible) | vendor, framework |
| session | One running harness subprocess identified by a unique ID in hook events | process, instance |
| observe-only | monocle reads harness state; it does not write to harness files or trigger harness actions | read-only (for workflow plane), passive |
| tee invariant | Every hook event seen by a harness subprocess reaches the daemon — no silent drops | N/A (this is a named invariant, not a synonym) |
| ABI version | The `MONOCLE_ABI_VERSION: u32` constant that identifies the daemon's public contract version | API version, protocol version |

## ID Registry Summary

| ID Format | Count | Section |
|-----------|-------|---------|
| CAP-NNN | 3 | Capabilities Registry (above) + section files |
| DI-NNN | 7 | Domain Invariants Summary (above) + section files |

## Priority Distribution

| Priority | Count | Items |
|----------|-------|-------|
| P0 (must-have for Phase 1) | 3 | CAP-001, CAP-002, CAP-003 |
| P1 (should-have) | 0 | N/A — all monocle Phase 1 capabilities are P0 |
| P2 (nice-to-have) | 0 | N/A — Phase 2+ capabilities are roadmap, not in this L2 |

## Architecture Cross-Reference

| Capability | Architecture Doc | ARCH-INDEX Subsystem |
|------------|-----------------|---------------------|
| CAP-001 | SS-daemon-lifecycle.md | SS-01 |
| CAP-002 | SS-core-types-and-abi.md | SS-02 |
| CAP-003 | SS-engine-module.md | SS-03 |

See `ARCH-INDEX.md` for the full subsystem registry and ADR list.

## BC Cross-Reference

See `behavioral-contracts/BC-INDEX.md` for the full BC registry. All 22 active
BCs are operationalizations of the 3 capabilities in this L2 spec.

## §Trace v1.0

**Template compliance Dispatch 6 of 7-8** (2026-05-17T14:00:00Z):
- Created as new artifact. Directory `.factory/specs/domain-spec/` populated.
- 3 capabilities extracted from product-brief.md v1.4.29 + vision-synthesis v1.1.2.
- Capability anchors grounded: CAP-001 from brief §Scope / In Scope / Phase 1 — Runtime Core + vision §JC-1 hook capture;
  CAP-002 from brief §Forward-compatibility contracts FC-01..FC-06; CAP-003 from
  brief §Phase 1 dual-engine + vision §Engine Module + §FactoryAdapter.
- 7 domain invariants extracted. Ubiquitous Language table (14 terms) grounded in
  brief + vision §Vision Statement + §Explicit Non-Goals.
- All section files: CAP-001-daemon-lifecycle.md, CAP-002-forward-compat-wire-formats.md,
  CAP-003-multi-harness-adapter.md.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T14:00:00Z >= chain high-water
  2026-05-17T13:30:00Z (Dispatch 5b VP-INDEX).
- Audit reference: `.factory/plans/template-compliance-audit-r1.md §MISS-05`.
- Next: Dispatch 7 (Architect runs compute-input-hash across all artifacts).

## §Trace v1.0.1

**RETROACTIVE BACKFILL — GAP-R44-5 closure** (2026-05-17T03:44:04Z, historical):
- v1.0.1 was NEVER a distinct commit. In commit 0af206a (2026-05-16 22:44:04 -0500 =
  2026-05-17T03:44:04Z UTC) the RES-01 input-hash normalization pass bumped L2-INDEX
  directly from v1.0 → v1.0.2 (one-level skip). No v1.0.1 artifact ever existed on disk
  or in git. This entry documents the skip for audit-trail completeness.
- SE-17f evidence: commit message reads "L2-INDEX 1.0→1.0.2" — confirming the intermediate
  v1.0.1 was never authored.
- This retroactive entry closes the v1.0.1 audit-trail gap per GAP-R44-5.

## §Trace v1.0.2

**RETROACTIVE BACKFILL — GAP-R44-5 closure** (2026-05-17T03:44:04Z, historical):
- Commit: 0af206a "spec(arch): RES-01 input-hash normalization + RES-04 ARCH-INDEX
  Tokens column" (2026-05-16 22:44:04 -0500 = 2026-05-17T03:44:04Z UTC).
- Changes to L2-INDEX.md (SE-17f before/after evidence from git diff 2a852d1..0af206a):
  - BEFORE `version: "1.0"` → AFTER `version: "1.0.2"`
  - BEFORE `timestamp: 2026-05-17T14:00:00Z` → AFTER `timestamp: 2026-05-17T16:30:00Z`
  - BEFORE `input-hash: "[live-state]"` → AFTER `input-hash: "494c12d"`
  - All other content unchanged (no section or prose edits).
- Context: RES-01 normalization pass ran compute-input-hash --update across 19 artifacts;
  L2-INDEX received its first real hash (`494c12d`), replacing the placeholder.
- This retroactive entry closes the v1.0.2 audit-trail gap per GAP-R44-5.

## §Trace v1.0.3

**F-R105-1 BA closure — HookEventRecord schema alignment** (2026-05-17T17:00:00Z):
- CAP-001-daemon-lifecycle.md bumped v1.0 → v1.1.
- HookEventRecord entity table corrected from 5-field opaque-blob schema to
  7-field canonical schema per BC-2.01.007 Postcondition 4 (verbatim field order:
  format_version, session_id, timestamp_micros, pid, hook_type, tool_name, tool_input).
- Surrounding prose updated to describe structured optional tool_name/tool_input fields.
- L2-INDEX version bumped 1.0.2 → 1.0.3; timestamp advanced.
- CAP-002 and CAP-003: no HookEventRecord schema table present — no changes required.
- SE-16d monotonicity PASS: 2026-05-17T17:00:00Z > prior 2026-05-17T16:30:00Z (v1.0.2).

## §Trace v1.0.4

**F-R105-6 + GAP-R44-2 BA closure — auth header rename cascade** (2026-05-17T18:00:00Z):
- CAP-001-daemon-lifecycle.md bumped v1.1 → v1.2.
- Auth header renamed in CAP-001 §P2 Hook Event Ingestion:
  `X-Claude-Code-Ide-Authorization` → `X-Monocle-Authorization` (1 occurrence).
- CAP-002 and CAP-003: zero auth header name occurrences — no changes required.
- L2-INDEX version bumped 1.0.3 → 1.0.4; timestamp advanced.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.3).

## §Trace v1.0.5

**F-R105-14 + GAP-R44-5 BA closure — anchor fix + retroactive trace backfill** (2026-05-17T19:00:00Z):
- F-R105-14 anchor fix: §Trace v1.0 prose citation corrected.
  - BEFORE: `CAP-001 from brief §Tier 1 + vision §JC-1 hook capture`
  - AFTER: `CAP-001 from brief §Scope / In Scope / Phase 1 — Runtime Core + vision §JC-1 hook capture`
  - Rationale: product-brief.md has no `## Tier 1` section. The correct section anchor is
    `## Scope → ### In Scope → **Phase 1 — Runtime Core**`. "§Tier 1" was an invention;
    "§Scope / In Scope / Phase 1 — Runtime Core" is the exact heading path in the brief.
    Confirmed by grep: no match for "Tier 1" as a heading anywhere in product-brief.md.
- GAP-R44-5 retroactive backfill: added §Trace v1.0.1 (documents the version skip — no
  distinct v1.0.1 commit ever existed) and §Trace v1.0.2 (RES-01 input-hash normalization,
  commit 0af206a, 2026-05-17T03:44:04Z). Both entries include SE-17f before/after diff
  evidence. §Trace v1.0.3 was already present in the prior file but was positioned
  out-of-sequence (between §Trace v1.0 and v1.0.4, before the retroactive v1.0.1 / v1.0.2
  entries). This edit reordered the chain to restore sequential monotonicity.
- SE-17c-d body-scope grep: searched L2-INDEX.md body for remaining stale §Tier 1
  references — none found beyond the §Trace v1.0 line corrected above.
- L2-INDEX version bumped 1.0.4 → 1.0.5; timestamp advanced.
- SE-16d monotonicity PASS: 2026-05-17T19:00:00Z > prior 2026-05-17T18:00:00Z (v1.0.4).

## §Trace v1.0.6

**T-128o ADR-0005 alias note cascade — CAP-001 v1.2 → v1.3** (2026-05-17T20:00:00Z):
- CAP-001-daemon-lifecycle.md bumped v1.2 → v1.3.
- Change: §P2 Hook Event Ingestion step 1 augmented with dual-accept alias note for
  `X-Claude-Code-Ide-Authorization` (Claude Code Phase 1 compatibility alias) per
  ADR-0005. Canonical `X-Monocle-Authorization` remains primary; alias accepted with
  WARN-level deprecation log; canonical takes priority when both are present.
- No CAP registry table row changes required (CAP-001 version is not tracked in the
  Capabilities Registry table; section file version lives in CAP-001 frontmatter only).
- SE-17c-d body-scope grep: searched L2-INDEX.md for any reference to auth header
  naming — none found in this index file; all auth header specifics live in
  CAP-001-daemon-lifecycle.md. No index-level prose changes required beyond version bump.
- L2-INDEX version bumped 1.0.5 → 1.0.6; timestamp advanced.
- SE-16d monotonicity PASS: 2026-05-17T20:00:00Z > prior 2026-05-17T19:00:00Z (v1.0.5).

## §Trace v1.0.7

**F-R107-12 BA closure — brief version cite refresh v1.4.23 → v1.4.25** (2026-05-17T23:00:00Z):
- SE-17f before/after:
  - BEFORE: `3 capabilities extracted from product-brief.md v1.4.23 + vision-synthesis v1.1.2.`
  - AFTER:  `3 capabilities extracted from product-brief.md v1.4.25 + vision-synthesis v1.1.2.`
- Rationale: PO Round 5C commit 56c11fe bumped product-brief.md v1.4.23 → v1.4.25. The
  §Trace v1.0 prose in L2-INDEX retained the stale v1.4.23 cite. F-R107-12 (LOW) flagged
  this discrepancy. This edit refreshes the cite to match the current brief version.
- SE-17c-d body-scope grep: searched L2-INDEX.md for all occurrences of `v1.4.` —
  only one match existed (line 149, now corrected). No other stale brief version cites
  in this file.
- CAP file scope check: CAP-002-forward-compat-wire-formats.md contains `v1.4.7` as a
  historical anchor cite (pointing to an earlier brief revision where FC-01..FC-06 were
  first introduced). That cite is in a CAP file, not L2-INDEX, and is outside BA L2-INDEX
  scope for this round. No CAP files touched.
- L2-INDEX version bumped 1.0.6 → 1.0.7; timestamp advanced.
- SE-16d monotonicity PASS: 2026-05-17T23:00:00Z > prior 2026-05-17T20:00:00Z (v1.0.6).

## §Trace v1.0.8

**F-R110-4 BA closure — brief version cite refresh v1.4.25 → v1.4.27** (2026-05-18T05:00:00Z):
- SE-17f before/after:
  - BEFORE: `3 capabilities extracted from product-brief.md v1.4.25 + vision-synthesis v1.1.2.`
  - AFTER:  `3 capabilities extracted from product-brief.md v1.4.27 + vision-synthesis v1.1.2.`
- Rationale: PO Round 9B bumped product-brief.md v1.4.25 → v1.4.27. The §Trace v1.0 prose
  in L2-INDEX retained the stale v1.4.25 cite. F-R110-4 (HIGH) flagged this discrepancy.
  This edit refreshes the cite to match the current brief version.
- SE-17c-d body-scope grep: searched L2-INDEX.md for all occurrences of `v1.4.` — two
  matches found: line 149 (§Trace v1.0 prose, now corrected) and §Trace v1.0.7 historical
  entries (preserved as historical record; not updated). No other stale cites.
- CAP-001 scope: see F-R110-5 closure in CAP-001-daemon-lifecycle.md §Trace v1.4.
- L2-INDEX version bumped 1.0.7 → 1.0.8; timestamp advanced.
- SE-16d monotonicity PASS: 2026-05-18T05:00:00Z > prior 2026-05-17T23:00:00Z (v1.0.7).

## §Trace v1.0.9

**R16D F-R117-4 + GAP-R56-002 BA closure — Document Map naming alignment + brief pin v1.4.27 → v1.4.28** (2026-05-18T16:30:00Z):

**Classification (SE-17g):** NORMATIVE — both fixes correct active current-state pointers.

**F-R117-4 closure (LOW) — Document Map labels aligned to canonical capability H1s:**
- SE-17a BEFORE/AFTER evidence (literal grep):
  - BEFORE line 44: `| Forward-Compat Wire Formats | CAP-002-forward-compat-wire-formats.md |`
  - AFTER  line 44: `| Forward-Compatible Wire Formats | CAP-002-forward-compat-wire-formats.md |`
  - BEFORE line 45: `| Multi-Harness Adapter | CAP-003-multi-harness-adapter.md |`
  - AFTER  line 45: `| Multi-Harness Adapter Surface | CAP-003-multi-harness-adapter.md |`
- Rationale: Document Map section labels drifted from canonical CAP-002 H1 (`Forward-Compatible Wire Formats`) and CAP-003 H1 (`Multi-Harness Adapter Surface`). Capabilities Registry (lines 62-63) used the canonical names throughout; only Document Map was inconsistent. Fixed Document Map to match capability section H1s exactly.
- CAP-001 Document Map label `Daemon Lifecycle` verified against Capabilities Registry entry `Daemon Lifecycle` — match confirmed, no change required.
- SE-17e sibling-propagation: CAP-002-forward-compat-wire-formats.md and CAP-003-multi-harness-adapter.md H1s are authoritative; only L2-INDEX Document Map was in scope for this fix. No CAP files modified.

**GAP-R56-002 closure (HIGH) — brief pin back-cascade v1.4.27 → v1.4.28:**
- SE-17a BEFORE/AFTER evidence (literal grep, §Trace v1.0 line 149):
  - BEFORE: `3 capabilities extracted from product-brief.md v1.4.27 + vision-synthesis v1.1.2.`
  - AFTER:  `3 capabilities extracted from product-brief.md v1.4.28 + vision-synthesis v1.1.2.`
- Rationale: R15B commit 08d1ef4 bumped product-brief.md v1.4.27 → v1.4.28. §Trace v1.0 is
  an active current-pointer (treated consistently with F-R107-12, F-R110-4 prior patterns in
  §Trace v1.0.7 and v1.0.8). Historical §Trace entries (v1.0.7, v1.0.8) citing v1.4.25 and
  v1.4.27 in their before/after slots are preserved as historical records — not updated.
- SE-17c-d body-scope grep: searched L2-INDEX.md for all `v1.4.` occurrences — one active
  pointer at §Trace v1.0 (now corrected to v1.4.28); all other occurrences are in historical
  §Trace before/after slots (preserved). No additional stale cites found.

**Additional sweep (Production-Grade Rule 4):**
- Swept all three Document Map labels vs Capabilities Registry names: CAP-001 match confirmed (no fix), CAP-002 fixed, CAP-003 fixed.
- Swept entire file for stale brief version pins beyond v1.4.27: none found outside historical §Trace slots.
- No additional drift detected.

- L2-INDEX version bumped 1.0.8 → 1.0.9; timestamp advanced to assigned SE-18 slot 2026-05-18T16:30:00Z.
- SE-16d monotonicity PASS: 2026-05-18T16:30:00Z > prior 2026-05-18T05:00:00Z (v1.0.8).

## §Trace v1.0.10

**F-R119-3 closure — brief pin back-cascade v1.4.28 → v1.4.29 per §Trace v1.0.7/v1.0.8/v1.0.9 precedent** (2026-05-18T22:30:00Z):

**Classification (SE-17g):** NORMATIVE — §Trace v1.0 line 149 is an active current-pointer to the brief version from which capabilities were extracted.

**Background:** R17B (commit b934e57, 2026-05-18T18:30:00Z) bumped product-brief.md v1.4.28 → v1.4.29. Per the established §Trace v1.0.7 (F-R107-12), §Trace v1.0.8 (F-R110-4), and §Trace v1.0.9 (GAP-R56-002) precedent, §Trace v1.0 line 149 is a NORMATIVE active-current pointer requiring refresh on every brief bump. R17B's SE-22 sweep operated in-artifact only (per SE-22 v1 codification) and did not enumerate L2-INDEX as a known brief-pin consumer — the SE-22 v1 first-cycle partial-effectiveness gap (structural per O-R119-3).

**Resolution:**

| Edit | Stale value | Canonical value | Source commit | Applied by | When |
|------|------------|-----------------|---------------|-----------|------|
| §Trace v1.0 line 149 brief pin | v1.4.28 | v1.4.29 | R17B (b934e57) | R18C BA (this burst) | 2026-05-18T22:30:00Z |

**SE-17a BEFORE/AFTER evidence (literal grep, §Trace v1.0 line 149):**
- BEFORE: `3 capabilities extracted from product-brief.md v1.4.28 + vision-synthesis v1.1.2.`
- AFTER:  `3 capabilities extracted from product-brief.md v1.4.29 + vision-synthesis v1.1.2.`

**SE-22 in-artifact sweep (L2-INDEX scope, SE-17g classification):**

| Pattern | Matches | Lines | Classification | Action |
|---------|---------|-------|----------------|--------|
| `product-brief.md v1.` | 5 | 149, 250, 251, 269, 270 | Line 149: NORMATIVE active pointer (corrected). Lines 250/251/269/270: INFORMATIONAL historical §Trace before/after slots (preserved) | Fixed line 149 |
| `brief v1.` | 5 | 271, 301 (partial) + historical slots | INFORMATIONAL — all in §Trace historical before/after prose | No action |
| `BC-INDEX v1.` | 0 | — | No pin present in L2-INDEX | No action |
| `prd.md v1.` | 0 | — | No pin present in L2-INDEX | No action |
| `VP-INDEX v1.` | 0 | — | No pin present in L2-INDEX | No action |
| `ARCH-INDEX v1.` | 0 | — | No pin present in L2-INDEX | No action |
| `SS-conventions-anti-patterns v1.` | 0 | — | No pin present in L2-INDEX | No action |
| `CAP-001 v1.` | 0 (registry table has no version column per §Trace v1.0.6) | — | No pin present in L2-INDEX | No action |

Zero-residual confirmation: no additional NORMATIVE stale pins remain in L2-INDEX.

**SE-22 v2 codification candidate (O-R119-3, HELD per D-114):** When artifact X bumps version, SE-22 should enumerate sibling artifacts holding NORMATIVE pins to X (consumer ledger). For brief bumps, known NORMATIVE consumers include: PRD `traces_to`, VP-INDEX §References, all VP files §References, CAP-001 §Trace active prose, L2-INDEX §Trace v1.0 line 149. R17B applied SE-22 v1 in-artifact only; this structural gap is codified as O-R119-3 pending human approval per D-114.

**SE-16d monotonicity PASS:** L2-INDEX v1.0.10 timestamp `2026-05-18T22:30:00Z` > R18B BC-INDEX v1.11 `2026-05-18T22:00:00Z` > R18A PRD v1.26.12 `2026-05-18T21:30:00Z`. Strict-greater at each step: PASS.

**Reference:** R119 adversary report at `.factory/plans/adversary-pass-r119-phase1.md`.
