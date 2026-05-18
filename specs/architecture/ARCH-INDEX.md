---
document_type: architecture-index
level: L3
version: "1.0.10"
status: active
producer: vsdd-factory:architect
timestamp: 2026-05-18T15:30:00Z
phase: pre-phase-1-architecture
inputs: [product-brief.md, prd.md]
input-hash: "da60462"
traces_to: prd.md
deployment_topology: single-service
project: monocle
---

# Architecture Index: monocle

> **Context Engineering:** This is a lightweight index (~200-400 tokens). Agents load
> ONLY the section files they need, not the full architecture. See the Document Map
> for per-section consumer guidance.

## Document Map

| Section | File | Tokens | Primary Consumer | Purpose |
|---------|------|--------|-----------------|---------|
| Daemon Lifecycle | SS-daemon-lifecycle.md | ~23,730 | orchestrator, implementer, test-writer | HTTP server, hooks, auth, locking, ring buffer, crash recovery |
| Core Types and ABI | SS-core-types-and-abi.md | ~10,072 | implementer, formal-verifier | Forward-compatible wire formats, factory abstractions, protocol versioning |
| Engine Module | SS-engine-module.md | ~15,013 | implementer, formal-verifier | EngineModule trait, ClaudeCodeModule adapter, harness abstraction |
| Dependency Manifest | SS-deps-pin-manifest.md | ~9,976 | implementer, devops-engineer | Version pins, MSRV policy, workspace dependency graph |
| Conventions & Anti-Patterns | SS-conventions-anti-patterns.md | ~25,794 | implementer, code-reviewer | Code conventions, forbidden patterns, clippy + semgrep enforcement |
| Forward Compatibility | SS-forward-compatibility.md | ~7,871 | architect, implementer | FC contracts P2-1..P3-N |
| Phase 1 Permissions | SS-permissions-phase1.md | ~2,661 | implementer, test-writer | Phase 1 permission enum |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| Implementation plan for daemon | SS-daemon-lifecycle.md + SS-core-types-and-abi.md + SS-deps-pin-manifest.md |
| Harness abstraction implementation | SS-engine-module.md + SS-core-types-and-abi.md |
| Verification plan for a module | SS-core-types-and-abi.md + SS-engine-module.md |
| Phase 3+ upgrade impact | SS-forward-compatibility.md + SS-deps-pin-manifest.md |
| Code review enforcement rules | SS-conventions-anti-patterns.md |

## Subsystem Registry

> **Source of truth** for subsystem names and IDs. BC frontmatter `subsystem:`,
> BC-INDEX subsystem column, story `subsystems:` fields, and PRD subsystem
> references MUST all use the exact Name from this table.

| SS ID | Name | Architecture Doc | Implementing Modules | Phase Introduced |
|-------|------|-----------------|---------------------|-----------------|
| SS-01 | Daemon Lifecycle | SS-daemon-lifecycle.md | monocle-runtime (daemon binary, HTTP server, ring buffer, lock file, auth) | Phase 1 |
| SS-02 | Core Types and ABI | SS-core-types-and-abi.md | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) | Phase 1 |
| SS-03 | Engine Module | SS-engine-module.md | monocle-core (EngineModule trait, ClaudeCodeModule adapter) | Phase 1 |

**ID format:** `SS-NN` (two-digit sequential, append-only).

**Naming rules:**
- Names are human-readable, title-case
- Names are stable — once assigned, a subsystem name does not change
- If a subsystem is retired, mark it `(retired)` in the Name column; do not remove the row

**Capability traceability:**

| SS ID | L2 Capability | Description |
|-------|--------------|-------------|
| SS-01 | CAP-001 | Daemon ingestion of Claude Code hook events; lifecycle management |
| SS-02 | CAP-002 | Forward-compatible ABI; wire format stability; factory-state abstraction |
| SS-03 | CAP-003 | Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter |

## Cross-Cutting Files

The following architecture documents are not assigned an SS-NN runtime subsystem ID.
They define conventions, constraints, and cross-cutting concerns that apply to all subsystems.

| File | Purpose |
|------|---------|
| SS-conventions-anti-patterns.md | Code conventions, forbidden patterns, clippy + semgrep + PR-template + CI enforcement |
| SS-deps-pin-manifest.md | Canonical dependency pins, MSRV policy, security-advisory response, workspace graph |
| SS-forward-compatibility.md | FC contracts for Phase 2/3/4 forward-compatibility surface |
| SS-permissions-phase1.md | Phase 1 `Phase1Permission` enum definition and exhaustive-enum policy |

## ADR Registry

| ADR ID | Title | Status | File |
|--------|-------|--------|------|
| ADR-0001 | wasmtime vs wasmi for WASM Plugin Runtime | accepted | adr/ADR-0001-wasmtime-vs-wasmi.md |
| ADR-0002 | Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger | accepted | adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md |
| ADR-0003 | MIT OR Apache-2.0 Dual-License Selection | accepted | adr/ADR-0003-license-selection.md |
| ADR-0004 | Exhaustive Enums — `Phase1Permission` and `ClaudeCodeTool` | accepted | adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md |
| ADR-0005 | Auth Header Dual-Accept — Canonical `X-Monocle-Authorization` with `X-Claude-Code-Ide-Authorization` Compatibility Alias | accepted | adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md |

**Note:** ADR-0001 covers Phase 3 wasmtime 44 adoption (not a Phase 1 runtime dependency).
ADR-0002 accepts nucleo 0.5 dormancy risk; re-eval trigger: if nucleo has no commit activity
for 6+ months by Phase 2 start, the architect must re-evaluate alternatives.
ADR-0005 resolves the auth header interop gap between monocle's canonical header and real
Claude Code's hardcoded `X-Claude-Code-Ide-Authorization` (BC-HOOK-016); dual-accept at the
router-level auth middleware.

## §Trace v1.0.2

**T-128e audit-trail reconciliation** (2026-05-17T17:00:00Z):
- NORMATIVE: §Trace v1.0.1 body corrected: hash citation `561ef4d` → `ee1f76a` to match
  frontmatter `input-hash: ee1f76a` (commit `0af206a`). No frontmatter change — frontmatter
  was always correct; only the §Trace narrative diverged.
- INFORMATIONAL: Version bump 1.0.1 → 1.0.2 records audit-trail correction; no content changes.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T17:00:00Z >= chain high-water 2026-05-17T16:30:00Z.
- Audit reference: `.factory/plans/adversary-cycle-001/R105-findings.md` F-R105-5 (HIGH).

**Audit R2 residual fix RES-04 + RES-01 fix-pass** (2026-05-17T16:30:00Z):
- RES-04: Added `Tokens` column to Document Map per architecture-index-template.md.
  Token counts computed as word_count × 1.3 (approximate), using `wc -w` per section file.
  All seven section files enumerated with `~N` token estimates.
- RES-01: Normalized `inputs:` field in ARCH-INDEX.md from absolute paths to relative
  paths (inline array format) resolvable by compute-input-hash. input-hash updated to
  `ee1f76a` (reflecting [product-brief.md, prd.md] content at fix-pass time). Path
  normalization also applied to 18 other [live-state] placeholder files in the same pass.
- version: 1.0 → 1.0.1; timestamp: 2026-05-17T11:00:00Z → 2026-05-17T16:30:00Z.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T16:30:00Z >= chain high-water 2026-05-17T16:00:00Z.
- Audit references: `.factory/plans/template-compliance-audit-r2.md` RES-01, RES-04.
- **T-128e reconciliation** (2026-05-17T17:00:00Z): §Trace originally cited `561ef4d`; corrected
  to `ee1f76a` to match frontmatter line 10 (actual value written by commit `0af206a`). Root
  cause: §Trace narrative was authored with an intermediate hash computed before the final
  compute-input-hash write; frontmatter received the definitive `ee1f76a` in the same commit.
  SE-17c body-scope grep evidence: `grep "561ef4d\|ee1f76a"` in §Trace body (lines ≥ 96)
  returned 1 match (`561ef4d`) prior to this correction — confirming the divergence between
  audit trail and artifact state (defect F-R105-5). Frontmatter `input-hash: ee1f76a` is
  authoritative; §Trace narrative now aligned. No frontmatter change required.
  Audit reference: `.factory/plans/adversary-cycle-001/R105-findings.md` F-R105-5 (HIGH).

**Template compliance Dispatch 1 of 6+** (2026-05-17T11:00:00Z):
- Created as new artifact; no prior version.
- Populates Subsystem Registry with SS-01 (Daemon Lifecycle), SS-02 (Core Types and ABI),
  SS-03 (Engine Module) per audit §MISS-03 subsystem proposals.
- Cross-Cutting Files section covers SS-deps-pin-manifest, SS-conventions-anti-patterns,
  SS-forward-compatibility, SS-permissions-phase1 (not runtime subsystems).
- ADR Registry enumerates ADR-0001..ADR-0004 from `.factory/specs/architecture/adr/`.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §MISS-03.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:00:00Z >= chain high-water 2026-05-17T10:30:00Z.

## §Trace v1.0.3

**T-128h BC ID canonicalization — F-R105-8 closure** (2026-05-17T17:00:00Z):
- NORMATIVE: All stale pre-renumbering BC IDs propagated to canonical BC-2.SS.NNN forms
  across 3 SS architecture documents per BC-INDEX.md v1.1 §Renumbering Map (canonical
  at T-128h dispatch time 2026-05-17T17:00:00Z; current canonical advances over time
  per F-R107-8 historical-pin discipline).
  Scope: SS-daemon-lifecycle.md, SS-engine-module.md, SS-core-types-and-abi.md.
- SE-17g META AUDIT — final re-grep confirms zero stale IDs remaining across all 3 docs
  (grep pattern: old-form DAEMON/AUTH/RING/LOCK/ABI/TYPES/FACTORY/PROTO/ENGINE prefixes):
  SS-daemon-lifecycle.md: 0 lines match (was 95 lines / 102 occurrences)
  SS-engine-module.md: 0 lines match (was 31 lines / 33 occurrences)
  SS-core-types-and-abi.md: 0 lines match (was 39 lines / 46 occurrences)
  Grand total replaced: 181 occurrences across 165 lines. SE-17g PASS: 165 → 0.
- DISCOVERED: PROTO-001 (bare pre-split form, old-style) — 2 occurrences in historical §Trace
  prose in SS-core-types-and-abi.md. This ID is retired by split (F-FC-O004); it has no
  canonical new-form entry in BC-INDEX §Renumbering Map (only the a/b split variants are
  mapped to BC-2.02.006 and BC-2.02.007). Resolved: historical §Trace prose rewritten to
  descriptive form; stale ID removed from SS doc body. Record preserved in BC-INDEX §Renumbering
  Map per append-only policy.
- SS doc versions bumped: SS-daemon-lifecycle.md 1.0.27 → 1.0.28; SS-engine-module.md
  1.1.17 → 1.1.18; SS-core-types-and-abi.md 1.2.10 → 1.2.11.
- ARCH-INDEX does not carry per-SS doc version numbers — no Document Map changes required.
- INFORMATIONAL: Version bump 1.0.2 → 1.0.3 records SE-17g META audit; no content changes
  to ARCH-INDEX body.
- SE-16d PASS: 2026-05-17T17:00:00Z >= chain high-water 2026-05-17T17:00:00Z (same burst;
  ARCH-INDEX and SS docs updated in the same T-128h dispatch — monotonicity satisfied).

## §Trace v1.0.4

**T-128m ADR-0005 auth header dual-accept — F-R105 closure chain Round 3** (2026-05-17T19:00:00Z):
- NORMATIVE: ADR-0005 authored and registered. Decision: dual-accept (option a).
  File: `adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md`.
  Resolves interop gap surfaced by BA in T-128f: real Claude Code hook scripts have
  `X-Claude-Code-Ide-Authorization` hardcoded per BC-HOOK-016 deep ingest; they cannot
  send `X-Monocle-Authorization`. ADR-0005 directs the daemon auth middleware to
  dual-accept both headers with `X-Monocle-Authorization` as canonical priority.
- NORMATIVE: ADR Registry updated — ADR-0005 row added.
- NORMATIVE: SS-daemon-lifecycle.md v1.0.28 → v1.0.29 — auth middleware spec updated
  to dual-accept; Rust stub rewritten; BC-2.01.009 table expanded; §Trace added.
- NORMATIVE: dtu-assessment.md v1.7.2 → v1.7.3 — ADR-0005 auth header rationale block
  added to endpoint matrix preamble; 10 `X-Claude-Code-Ide-Authorization` occurrences
  confirmed correct (DTU tests compatibility alias path).
- NORMATIVE: BC-2.01.009 update surfaced to PO for Round 4 (postcondition 1 "missing"
  semantics; alias path postconditions 2-3 extension).
- NORMATIVE: CAP-001 §P2 step 1 compatibility alias note surfaced to BA for Round 4.
- INFORMATIONAL: product-brief.md occurrences (lines 116 and 239) out of scope for this
  dispatch; surfaced to PO for Round 4 as noted.
- SE-16d PASS: 2026-05-17T19:00:00Z > chain high-water 2026-05-17T17:00:00Z.

## §Trace v1.0.5

**F-R106 Round 5E — ADR-0005 path fix + SS-daemon-lifecycle F-FC-I005 removal** (2026-05-17T22:00:00Z):
- NORMATIVE: ADR-0005 v1.0.1 → v1.0.2 — frontmatter `inputs:` third entry normalized;
  spurious `specs/` prefix removed from `behavioral-contracts/ss-01/BC-2.01.009.md`.
  §Trace v1.0.2 added to ADR-0005.
- NORMATIVE: SS-daemon-lifecycle.md v1.0.29 → v1.0.30 — F-FC-I005 fabricated ID removed
  from two sites: §Start Sequence body (~line 298) and §Behavioral Contract Summary
  BC-2.01.009 table row (~line 800). Replaced with FC-06 alone (canonical reference).
  SE-17g META AUDIT PASS: zero F-FC-I005 occurrences remain in SS-daemon-lifecycle.md.
  §Trace v1.0.30 added.
- INFORMATIONAL: ARCH-INDEX ADR Registry table does not carry per-ADR version numbers;
  no content change to ADR-0005 row required. ARCH-INDEX Document Map does not carry
  per-SS doc version numbers (confirmed §Trace v1.0.3). Only §Trace version bumped here.
- SE-16d PASS: 2026-05-17T22:00:00Z > chain high-water 2026-05-17T19:00:00Z (monotonic).

## §Trace v1.0.6

**F-R107 Round 6D — BC ID canonicalization + historical-pin clarification** (2026-05-17T23:00:00Z):
- NORMATIVE (F-R107-5 HIGH / SS-forward-compatibility.md v1.2.14 → v1.2.16): All stale
  pre-renumbering BC IDs in FC table and BC-mapping table canonicalized to BC-2.SS.NNN forms
  per BC-INDEX.md v1.4 §Renumbering Map. FC-04 body prose updated. BC-mapping table
  restructured: "Old-Form ID (retired)" column added; all 16 rows carry canonical new IDs as
  primary. Notes paragraph updated with old→new cross-references. SE-17g META AUDIT: zero
  normative stale BC IDs remain in SS-forward-compatibility.md. [Note: this entry originally
  cited "v1.2.15 → v1.2.16" — corrected to "v1.2.14 → v1.2.16" by F-R109-13 Round 8A.
  Version v1.2.15 was never a real intermediate state; git evidence confirms the file went
  directly from v1.2.14 to v1.2.16 in commit 98396fe. See SS-forward-compatibility.md
  §Trace v1.2.17-R109 for full disposition.]
- INFORMATIONAL (F-R107-8 / ARCH-INDEX §Trace v1.0.3): BC-INDEX cite `v1.1 §Renumbering Map`
  expanded to explicit historical-pin form: `v1.1 §Renumbering Map (canonical at T-128h
  dispatch time 2026-05-17T17:00:00Z; current canonical advances over time per F-R107-8
  historical-pin discipline)`. Purpose: prevent future fresh-context audits from re-flagging
  the historical pin. [Note: original "current canonical BC-INDEX is v1.4" language corrected
  to historical-pin-only form by F-R108-1 Round 7C.]
- INFORMATIONAL (F-R107-8 / SS-engine-module.md v1.1.18 → v1.1.19): Same historical-pin
  expansion applied to §Trace v1.1.18. `v1.1 §Renumbering Map (canonical at T-128h dispatch
  time 2026-05-17T17:00:00Z; current canonical advances over time per F-R107-8
  historical-pin discipline)`. [Original live-version claim corrected by F-R108-1 Round 7C.]
- INFORMATIONAL (F-R107-8 / SS-daemon-lifecycle.md v1.0.30 → v1.0.31): Same historical-pin
  expansion applied to §Trace v1.0.28. [Original live-version claim corrected by F-R108-1 Round 7C.]
- INFORMATIONAL (F-R107-8 / SS-core-types-and-abi.md v1.2.11 → v1.2.12): Same historical-pin
  expansion applied to §Trace v1.2.11. [Original live-version claim corrected by F-R108-1 Round 7C.]
- SE-16d PASS: 2026-05-17T23:00:00Z > chain high-water 2026-05-17T22:00:00Z (monotonic).

## §Trace v1.0.7

**F-R108 Round 7C — historical-pin live-version removal + frontmatter timestamp correction** (2026-05-18T01:00:00Z):
- NORMATIVE (F-R108-1 CRITICAL): All "current canonical BC-INDEX is v1.4 per F-R107-2 closure"
  live-version claims removed from 5 arch docs per O-R108-3 codification. Sites corrected:
  - SS-forward-compatibility.md: 1 occurrence in BC-mapping notes (pre-§Trace body).
  - SS-daemon-lifecycle.md: 2 occurrences — §Trace v1.0.28 body + §Trace v1.0.31 prose.
  - SS-engine-module.md: 2 occurrences — §Trace v1.1.18 body + §Trace v1.1.19 prose.
  - SS-core-types-and-abi.md: 2 occurrences — §Trace v1.2.11 body + §Trace v1.2.12 prose.
  - ARCH-INDEX.md (this file): 3 occurrences — §Trace v1.0.6 (2 occurrences, corrected below)
    + §Trace v1.0.3 (1 additional occurrence discovered during O-R108-3 corpus application).
  Replacement language: "current canonical advances over time per F-R107-8 historical-pin discipline".
- NORMATIVE (F-R108-9 HIGH): frontmatter `timestamp` corrected on all 5 files from stale values
  to 2026-05-18T01:00:00Z (matching chain high-water). SE-16b violation resolved on each.
  No version bumps applied to SS docs — content-change §Trace entries written; frontmatter bumps
  deferred to Round 8A per cross-dispatch coordination directive (F-R109-1 closure).
  SS docs that received new §Trace entries (content change): SS-daemon-lifecycle (v1.0.32 added),
  SS-engine-module (v1.1.20 added), SS-core-types-and-abi (v1.2.13 added),
  SS-forward-compatibility (v1.2.17 added).
- NORMATIVE (F-R108-10 HIGH): ADR-0002 v1.0.2 → v1.0.3. frontmatter `inputs:` path fix:
  `tech-debt-register.md` → `../tech-debt-register.md`; `plans/production-grade-reaudit.md`
  → `../plans/production-grade-reaudit.md`. Both paths now resolve from `.factory/specs/`
  context. §Trace v1.0.3 added to ADR-0002.
- INFORMATIONAL (F-R108-20 LOW): dtu-assessment.md inputs paths verified: all 4 resolve correctly
  from `.factory/specs/` context (product-brief.md, architecture/SS-deps-pin-manifest.md,
  architecture/SS-core-types-and-abi.md, semport/any-context-lazyclaude/…-final-synthesis-v2.md).
  No changes required. §Trace v1.7.4 added to dtu-assessment.md (verification record).
- INFORMATIONAL (F-R108-21 LOW): §Trace v1.0.6 above split into 4 sub-bullets per artifact,
  replacing the combined narrative. Content preserved; structure only.
- SE-16d PASS: 2026-05-18T01:00:00Z > chain high-water 2026-05-17T23:00:00Z (monotonic).

## §Trace v1.0.8

**F-R109 Round 8A — SS frontmatter version reconciliation + §Trace ordering + v1.2.15 gap disposition** (2026-05-18T05:00:00Z):
- NORMATIVE (F-R109-1 CRITICAL + F-R109-2 CRITICAL): frontmatter `version` bumped on 4 SS docs
  to reconcile with §Trace version numbers already written in Round 7C. The Round 7C cross-dispatch
  coordination directive withheld frontmatter bumps to avoid PO 7B pin staleness; this created
  fabrication-class defects where frontmatter claimed prior versions while §Trace bodies documented
  new versions. Bumps applied:
  - SS-daemon-lifecycle.md: frontmatter "1.0.31" → "1.0.32" (§Trace v1.0.32 already present).
  - SS-engine-module.md: frontmatter "1.1.19" → "1.1.20" (§Trace v1.1.20 already present).
  - SS-core-types-and-abi.md: frontmatter "1.2.12" → "1.2.13" (§Trace v1.2.13 already present).
  - SS-forward-compatibility.md: frontmatter "1.2.16" → "1.2.17" (§Trace v1.2.17 already present).
  - ARCH-INDEX.md (this file): "1.0.7" → "1.0.8" (cascade from SS doc changes + F-R109-2/9/13).
- NORMATIVE (F-R109-8 HIGH): §Trace v1.0.32/v1.1.20/v1.2.13/v1.2.17 bodies in all 4 SS docs
  rewritten to remove "No version bump — content unchanged; timestamp-only correction"
  self-contradiction. The F-R108-1 removals of live-version claim strings ARE normative content
  changes. All 4 §Trace bodies now accurately describe the version bumps as applied in Round 8A.
- NORMATIVE (F-R109-9 HIGH): §Trace ordering in ARCH-INDEX reordered from descending to ascending
  (v1.0.2 → v1.0.3 → … → v1.0.8) to match BC-INDEX pattern per F-R109-9.
- NORMATIVE (F-R109-13 MED): §Trace v1.0.6 above corrected: "v1.2.15 → v1.2.16" changed to
  "v1.2.14 → v1.2.16". Version v1.2.15 of SS-forward-compatibility.md never existed; the file
  transitioned directly from v1.2.14 to v1.2.16 in Round 6D (git commit 98396fe confirms). The
  ARCH-INDEX v1.0.6 narrative fabricated the v1.2.15 intermediate. SS-forward-compatibility.md
  §Trace v1.2.17-R109 carries the full disposition record.
- NORMATIVE (F-R109-8 / ARCH-INDEX): §Trace v1.0.7 above updated to remove "No version bumps
  applied — content unchanged" language for the SS doc frontmatter side; corrected to note that
  frontmatter bumps were deferred to Round 8A per cross-dispatch coordination directive.
- SE-17c BEFORE (F-R109-13): "SS-forward-compatibility.md v1.2.15 → v1.2.16" in §Trace v1.0.6.
- SE-17c AFTER (F-R109-13): "SS-forward-compatibility.md v1.2.14 → v1.2.16" in §Trace v1.0.6.
- SE-16d PASS: 2026-05-18T05:00:00Z > chain high-water 2026-05-18T01:00:00Z (monotonic; corrected from erroneous 2026-05-17T04:30:00Z per F-R110-1).

## §Trace v1.0.9

**F-R110 Round 9A — timestamp correction + ADR-0002 citation fix + ARCH-INDEX input-hash verification** (2026-05-18T05:30:00Z):
- NORMATIVE (F-R110-1 CRITICAL): §Trace v1.0.8 header and frontmatter `timestamp` corrected from
  "2026-05-17T04:30:00Z" to "2026-05-18T05:00:00Z". Round 8A dispatch used a date in the past
  relative to Round 7C output (2026-05-18T01:00:00Z), breaking the SE-16d monotonic chain across
  all 5 affected files (4 SS docs + this ARCH-INDEX). The erroneous SE-16d PASS claim
  "2026-05-17T04:30:00Z satisfies chain monotonicity" was arithmetically false: 04:30 on May 17
  precedes 01:00 on May 18 by nearly 21 hours. Corrected timestamps restore the chain:
  01:00:00Z (Round 7C) < 05:00:00Z (Round 8A corrected) < 05:30:00Z (this entry).
- NORMATIVE (F-R110-6 HIGH): ADR-0002 §Source / Origin section corrected absolute machine-local
  path to relative path for SS-deps-pin-manifest.md. §Trace v1.0.4 added to ADR-0002.
- NORMATIVE (F-R110-11 MED): ARCH-INDEX input-hash refreshed. Declared hash "ee1f76a" was stale;
  compute-input-hash recomputed to "da60462" reflecting current state of inputs [product-brief.md,
  prd.md] (both updated in Round 8B by PO scope; hash not refreshed in Round 8A arch scope).
  Frontmatter `input-hash` updated from "ee1f76a" to "da60462".
- ARCH-INDEX version: "1.0.8" → "1.0.9" (cascade from F-R110-1 normative timestamp correction).
- SE-16d PASS: 2026-05-18T05:30:00Z > chain high-water 2026-05-18T05:00:00Z (monotonic).

## §Trace v1.0.10

**R16B F-R117-2 ADR-0002 + full ADR Registry H1 sibling sweep** (2026-05-18T15:30:00Z):
- NORMATIVE (F-R117-2 HIGH — ADR-0002 row "for Phase 1" qualifier restoration):
  ADR-0002 INDEX row title corrected from `Accept nucleo 0.5 Dormancy Risk with Explicit Re-eval Trigger`
  to `Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`.
  BEFORE: `Accept nucleo 0.5 Dormancy Risk with Explicit Re-eval Trigger`
  AFTER:  `Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`
  Source authority: ADR-0002 H1 (line 21): `# ADR-0002: Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`.
  "for Phase 1" is normatively load-bearing — it scopes the acceptance decision to Phase 1 only,
  which drives the re-eval trigger semantics described in the ADR Registry Note below the table.
  Defect class: H1↔INDEX-row title drift (same class as F-R116-1 closed for VP-INDEX).
- NORMATIVE (sibling-sweep ADR-0004 backtick restoration):
  ADR-0004 INDEX row title corrected from `Exhaustive Enums — Phase1Permission and ClaudeCodeTool`
  to `Exhaustive Enums — \`Phase1Permission\` and \`ClaudeCodeTool\``.
  BEFORE: `Exhaustive Enums — Phase1Permission and ClaudeCodeTool`
  AFTER:  `Exhaustive Enums — \`Phase1Permission\` and \`ClaudeCodeTool\``
  Source authority: ADR-0004 H1 (line 21): `# ADR-0004: Exhaustive Enums — \`Phase1Permission\` and \`ClaudeCodeTool\``.
  Backtick code spans in markdown table cells are valid syntax and must be preserved verbatim.
- NORMATIVE (sibling-sweep ADR-0005 backtick restoration):
  ADR-0005 INDEX row title corrected from
  `Auth Header Dual-Accept — Canonical X-Monocle-Authorization with X-Claude-Code-Ide-Authorization Compatibility Alias`
  to
  `Auth Header Dual-Accept — Canonical \`X-Monocle-Authorization\` with \`X-Claude-Code-Ide-Authorization\` Compatibility Alias`.
  BEFORE: `Auth Header Dual-Accept — Canonical X-Monocle-Authorization with X-Claude-Code-Ide-Authorization Compatibility Alias`
  AFTER:  `Auth Header Dual-Accept — Canonical \`X-Monocle-Authorization\` with \`X-Claude-Code-Ide-Authorization\` Compatibility Alias`
  Source authority: ADR-0005 H1 (line 21): full verbatim with backtick code spans around both header names.
- INFORMATIONAL (sibling-sweep — rows confirmed MATCH, no fix required):
  ADR-0001: INDEX title `wasmtime vs wasmi for WASM Plugin Runtime` matches ADR-0001 H1 title portion. PASS.
  ADR-0003: INDEX title `MIT OR Apache-2.0 Dual-License Selection` matches ADR-0003 H1 title portion. PASS.
  Document Map "Section" column: assessed as navigation labels (deliberate short forms), not verbatim H1 matches.
  The "Architecture: " prefix is consistently omitted across all seven Document Map rows by established ARCH-INDEX convention;
  no §Trace entry has previously flagged this as drift. Convention confirmed valid; no fixes required.
  Subsystem Registry rows: Names (SS-01 Daemon Lifecycle, SS-02 Core Types and ABI, SS-03 Engine Module) are
  defined in the Subsystem Registry itself as the source of truth — they are NOT H1 copies from SS-*.md files.
  Confirmed correct per ARCH-INDEX.md §"Naming rules" (stable, title-case, human-readable). PASS.
- SE-17a LITERAL-GREP EVIDENCE (F-R117-2):
  `grep -nE "^# " .factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md`
  → `21:# ADR-0002: Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`
  Confirms "for Phase 1" present in H1; absent from prior INDEX row — defect confirmed, now closed.
- SE-17c BEFORE/AFTER (F-R117-2): captured inline in NORMATIVE bullets above.
- SE-17f SCOPED VERIFICATION (sibling sweep): all 5 ADR H1s extracted via `grep -nE "^# "` per ADR file;
  all 7 SS-*.md H1s extracted via single grep invocation. Five ADR INDEX rows cross-checked.
  Three drifted rows found; three fixed. Two matched; confirmed PASS with no change.
- SE-17g NORMATIVE vs INFORMATIONAL:
  NORMATIVE: ADR-0002 "for Phase 1" restoration (F-R117-2 HIGH); ADR-0004 backtick restoration (sibling);
  ADR-0005 backtick restoration (sibling). All three alter INDEX row content.
  INFORMATIONAL: ADR-0001, ADR-0003 match confirmations; Document Map Section label convention assessment;
  Subsystem Registry name source-of-truth confirmation. No file content changed.
- SE-17e SIBLING-PROPAGATION: F-R116-1 established H1↔INDEX-row verbatim-match discipline for VP-INDEX;
  SE-17e directs this discipline to propagate to all INDEX files. This burst (R16B) applies the discipline
  to ARCH-INDEX ADR Registry as the first sibling application. Occurrence count per SE-22:
  this is occurrence #2 of the H1↔INDEX-row drift pattern (F-R116-1 in VP-INDEX = #1; F-R117-2
  in ARCH-INDEX ADR Registry = #2). D-114 observation status maintained; SE-22 3+ threshold not yet
  reached. Pattern: agents authoring INDEX rows strip qualifiers ("for Phase 1") and code spans
  (backticks) when transcribing H1 titles — likely caused by informal paraphrasing during index creation.
- SE-16d PASS: 2026-05-18T15:30:00Z > chain high-water 2026-05-18T05:30:00Z (monotonic).
