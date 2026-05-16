---
document_type: architecture-index
level: L3
version: "1.0"
status: active
producer: vsdd-factory:architect
timestamp: 2026-05-17T11:00:00Z
phase: pre-phase-1-architecture
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/prd.md
input-hash: "[live-state]"
traces_to: prd.md
deployment_topology: single-service
project: monocle
---

# Architecture Index: monocle

> **Context Engineering:** This is a lightweight index (~200-400 tokens). Agents load
> ONLY the section files they need, not the full architecture. See the Document Map
> for per-section consumer guidance.

## Document Map

| Section | File | Primary Consumer | Purpose |
|---------|------|-----------------|---------|
| Daemon Lifecycle | SS-daemon-lifecycle.md | orchestrator, implementer, test-writer | HTTP server, hooks, auth, locking, ring buffer, crash recovery |
| Core Types and ABI | SS-core-types-and-abi.md | implementer, formal-verifier | Forward-compatible wire formats, factory abstractions, protocol versioning |
| Engine Module | SS-engine-module.md | implementer, formal-verifier | EngineModule trait, ClaudeCodeModule adapter, harness abstraction |
| Dependency Manifest | SS-deps-pin-manifest.md | implementer, devops-engineer | Version pins, MSRV policy, workspace dependency graph |
| Conventions & Anti-Patterns | SS-conventions-anti-patterns.md | implementer, code-reviewer | Code conventions, forbidden patterns, clippy + semgrep enforcement |
| Forward Compatibility | SS-forward-compatibility.md | architect, implementer | FC contracts P2-1..P3-N |
| Phase 1 Permissions | SS-permissions-phase1.md | implementer, test-writer | Phase 1 permission enum |

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

## §Trace v1.0

**Template compliance Dispatch 1 of 6+** (2026-05-17T11:00:00Z):
- Created as new artifact; no prior version.
- Populates Subsystem Registry with SS-01 (Daemon Lifecycle), SS-02 (Core Types and ABI),
  SS-03 (Engine Module) per audit §MISS-03 subsystem proposals.
- Cross-Cutting Files section covers SS-deps-pin-manifest, SS-conventions-anti-patterns,
  SS-forward-compatibility, SS-permissions-phase1 (not runtime subsystems).
- ADR Registry enumerates ADR-0001..ADR-0004 from `.factory/specs/architecture/adr/`.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §MISS-03.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:00:00Z >= chain high-water 2026-05-17T10:30:00Z.
