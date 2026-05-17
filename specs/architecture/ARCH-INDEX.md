---
document_type: architecture-index
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:architect
timestamp: 2026-05-17T19:00:00Z
phase: pre-phase-1-architecture
inputs: [product-brief.md, prd.md]
input-hash: "ee1f76a"
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
| ADR-0002 | Accept nucleo 0.5 Dormancy Risk with Explicit Re-eval Trigger | accepted | adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md |
| ADR-0003 | MIT OR Apache-2.0 Dual-License Selection | accepted | adr/ADR-0003-license-selection.md |
| ADR-0004 | Exhaustive Enums — Phase1Permission and ClaudeCodeTool | accepted | adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md |
| ADR-0005 | Auth Header Dual-Accept — Canonical X-Monocle-Authorization with X-Claude-Code-Ide-Authorization Compatibility Alias | accepted | adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md |

**Note:** ADR-0001 covers Phase 3 wasmtime 44 adoption (not a Phase 1 runtime dependency).
ADR-0002 accepts nucleo 0.5 dormancy risk; re-eval trigger: if nucleo has no commit activity
for 6+ months by Phase 2 start, the architect must re-evaluate alternatives.
ADR-0005 resolves the auth header interop gap between monocle's canonical header and real
Claude Code's hardcoded `X-Claude-Code-Ide-Authorization` (BC-HOOK-016); dual-accept at the
router-level auth middleware.

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

## §Trace v1.0.3

**T-128h BC ID canonicalization — F-R105-8 closure** (2026-05-17T17:00:00Z):
- NORMATIVE: All stale pre-renumbering BC IDs propagated to canonical BC-2.SS.NNN forms
  across 3 SS architecture documents per BC-INDEX.md v1.1 §Renumbering Map.
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
