---
document_type: architecture-index
level: L3
version: "1.0.2"
status: active
producer: vsdd-factory:architect
timestamp: 2026-05-17T17:00:00Z
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

**Note:** ADR-0001 covers Phase 3 wasmtime 44 adoption (not a Phase 1 runtime dependency).
ADR-0002 accepts nucleo 0.5 dormancy risk; re-eval trigger: if nucleo has no commit activity
for 6+ months by Phase 2 start, the architect must re-evaluate alternatives.

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
