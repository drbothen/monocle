---
document_type: architecture-dependencies
level: L3
section: "dependencies"
version: "1.0"
status: stub
producer: product-owner (extracted from brief v1.1 during v1.2 revision)
phase: pre-phase-1-architecture
timestamp: 2026-05-12T16:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
input-hash: "[live-state]"
traces_to: "factory-artifacts b3c68ca (OQ research); 2737bfd (vision)"
project: monocle
---

# Architecture: Dependency Manifest

## Authority / Supersession

This document is the canonical, authoritative tech-stack version manifest for monocle. The vision document (`.factory/specs/research/domain-monocle-vision-synthesis.md`) §Tech Stack section was produced before OQ-01..OQ-11 resolution and carries pre-OQ version examples. Where this document and the vision disagree on a crate version, this document wins. Where this document and brief v1.3 disagree, this document wins. Trace: D-018 (oq-research.md commit b3c68ca), JC-1/JC-2/JC-3 resolutions.

## [Section Content]

This file is the canonical dependency manifest for monocle. The product brief
asserts inheritance of all version pins via vision D-012 — these picks are
pre-committed by human approval and are NOT up for re-selection in Phase 1.

The architect inherits this manifest as a Phase 1 constraint and is expected
to translate it into the workspace `Cargo.toml` with a dependency-graph diagram
during `/vsdd-factory:create-architecture`. Refinements (patch-version pinning
strategy, dual-MSRV considerations) are recorded in the Architect TODO section.

## Phase 1 Pin Manifest

All versions verified against crates.io REST API on 2026-05-12.

| Crate | Version | Role | Cargo.toml Note |
|-------|---------|------|-----------------|
| ratatui | 0.30 | TUI framework | MSRV floor for Phase 1 (1.86) |
| crossterm | 0.29 | Terminal backend for ratatui | |
| tokio | 1.52 | Async runtime (full feature set) | Historical advisories on older minors; 1.52 remediated |
| axum | 0.8 | HTTP server for hook ingestion | Pin as `^0.8.9` in Cargo.toml |
| interprocess | 2.4 | Unix domain socket IPC | |
| prost | 0.14 | Protobuf serialization for cross-host wire format | See RUSTSEC note on transitive `bytes` advisory |
| serde_json | (workspace) | JSON serialization | |
| serde_yaml_ng | 0.10 | YAML config parsing | NOT `serde_yaml 0.8` (unmaintained, alias-bomb CVE); NOT `serde_yml` (archived per RUSTSEC-2025-0068) |
| bytes | (direct pin) | Byte buffer utility | Pin directly in workspace to avoid prost 0.14 transitive RUSTSEC-2026-0007 |
| wasmtime | 44 | WASM runtime for Phase 3 plugin SDK | NOT wasmi — see ADR-0001; Phase 3 MSRV implication: Rust 1.92 |
| nucleo | 0.5 | Fuzzy matcher for session/filter panels | Upstream dormant since 2024-04-02; flagged in tech-debt-register TD-001 |
| similar | 3 | Diff rendering in permission prompt overlay | |
| directories | 6 | XDG-compliant config/data/runtime dirs | Used for daemon lock-file path per OQ-10 resolution |
| notify | 8 | File-system watcher for Phase 3 workflow plane | |
| russh | 0.60 | SSH tunnel for Phase 4 federation | Pin to 0.60 (0.60.2); 0.45..0.59 carry RUSTSEC-2023-0071 via transitive rsa pre-release |
| rmcp | 1.6 | MCP bridge (Phase 4 only; OMITTED in v1 per OQ-09) | Anthropic-canonical via modelcontextprotocol/rust-sdk; owner alexhancock@Anthropic on crates.io |
| tempfile | 3 | Atomic config writes via `tempfile::persist` | Required anti-pattern enforcement |
| clap | 4.6 | CLI argument parsing | |
| pulldown-cmark | 0.13 | Markdown rendering in TUI panels | |
| arboard | 3 | Clipboard integration | |
| tracing | 0.1 | Structured logging and instrumentation | |
| semver | 1 | Semantic version parsing | |
| thiserror | 2 | Error type derivation | 2.x major — do NOT pin to 1.x |
| anyhow | 1 | Error propagation in binary crate | |
| reqwest | 0.13 | HTTP client | 0.13.x only — do NOT pin to 0.11 or 0.12 (both stale) |

## Phase 2/3/4 Additions

Placeholder for architect. Known incoming additions:
- Phase 3: `monocle-plugin-sdk` WASM ABI (wasmtime 44 already pinned above; MSRV bumps to 1.92)
- Phase 4: russh transport (already pinned above), prost wire format (already pinned), rmcp MCP bridge (already pinned)
- Additional dependencies to be specified during `/vsdd-factory:create-architecture` for each phase

## RUSTSEC Audit Context

Validation performed 2026-05-12 against crates.io API + RUSTSEC advisory DB
(Tavily + Perplexity + direct crates.io fetch). Findings the architect must
respect when finalizing Cargo.toml:

### Advisories on upstream versions monocle must avoid

- `wasmtime` older majors (pre-44) carry RUSTSEC-2026-0114, RUSTSEC-2026-0095,
  RUSTSEC-2026-0096, RUSTSEC-2026-0006, RUSTSEC-2026-0020 (guest-controlled resource
  exhaustion in WASI implementations), and others. Pin to `wasmtime = "44"` (latest
  44.0.1) and bind future patches via `cargo update` on the 44.x line.
- `russh` 0.45..0.59 transitively pulls `rsa = "0.10.0-rc.12"` which is affected by
  RUSTSEC-2023-0071 (timing-attack on RSA private-key operations). Pin to
  `russh = "0.60"` (0.60.2 latest) which moved off the affected rsa pre-release.
- `prost` 0.14 has a transitive `bytes` advisory RUSTSEC-2026-0007 affecting older
  `bytes` versions. Pin `bytes` directly in workspace dependencies to force a patched
  version (e.g. `bytes = "1.10"` or whatever the patched line is at audit time).
- `tokio` 1.x has multiple historical advisories on older minors (RUSTSEC-2025-0023,
  RUSTSEC-2023-0005, RUSTSEC-2023-0001, RUSTSEC-2021-0124, RUSTSEC-2021-0072). Pin to
  current 1.52 line to ensure all are remediated.
- `serde_yaml` 0.8 is unmaintained with alias-bomb CVE; `serde_yml` (a different fork)
  was archived per RUSTSEC-2025-0068. The choice of `serde_yaml_ng` 0.10 (maintained
  fork) is correct and survives this audit.

## Re-audit Cadence

The architect must enforce a `cargo audit` run in CI on every PR, plus a weekly
scheduled `cargo audit --json` against the latest RUSTSEC DB. New advisories on
pinned versions block merge until either:
- (a) the version is updated to a patched release, or
- (b) a documented justification with mitigations is filed under
  `.factory/specs/risk-acceptance/`

## MSRV Constraints

| Phase | MSRV | Binding Factor |
|-------|------|---------------|
| Phase 1 | Rust 1.86 | ratatui 0.30 floor |
| Phase 3 | Rust 1.92 | wasmtime 44 requirement |

The workspace `Cargo.toml` `rust-version` field must be set to `"1.86"` for Phase 1.
Phase 3 work begins in a separate workspace or with a workspace-wide MSRV bump
decision documented as an ADR.

## Architect TODO

- [ ] Confirm dual-MSRV strategy vs single workspace bump at Phase 3 boundary
- [ ] Set patch-version pinning policy (exact pin `=1.52.x` vs caret `^1.52`)
- [ ] Confirm `bytes` direct-pin resolves prost RUSTSEC-2026-0007 at current patch level
- [ ] Version bump policy for security advisories (automated Dependabot PRs vs manual?)
- [ ] Confirm rmcp 1.6 omission from Phase 1 workspace (OQ-09 resolution: no stub)
- [ ] Add dependency-graph diagram showing crate-level `Cargo.toml` edges
