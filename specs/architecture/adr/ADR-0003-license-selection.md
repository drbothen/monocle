---
document_type: adr
adr_id: ADR-0003
status: accepted
date: 2026-05-12
subsystems_affected: []
supersedes: null
superseded_by: null
level: L3
section: "adr"
version: "1.0.2"
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-17T16:30:00Z
inputs: [SS-deps-pin-manifest.md, product-brief.md]
input-hash: "28869ce"
traces_to: "adversary F-NEW-08 (IMPORTANT); human Q-license (MIT/Apache-2.0 dual); devops-engineer companion: cargo-deny CI gate"
project: monocle
---

# ADR-0003: MIT OR Apache-2.0 Dual-License Selection

## Status

Accepted

## Context

monocle is a greenfield Rust project with 24+ pinned dependencies across Phase 1–4.
The dependency chain includes crates with non-trivial licenses:

| Crate | License | Copyleft type |
|-------|---------|---------------|
| russh 0.60 | Apache-2.0 AND MIT | None |
| wasmtime 44 | Apache-2.0 | None |
| nucleo 0.5 | MIT | None |
| axum 0.8 | MIT | None |
| tokio 1.52 | MIT | None |
| rmcp 1.6 | MIT | None (Anthropic-canonical) |
| ratatui 0.30 | MIT | None |
| prost 0.14 | Apache-2.0 | None |

The only dependency with weak copyleft is `nucleo 0.5` (MPL-2.0). MPL-2.0 is a
**file-level** copyleft — it requires modifications to nucleo's source files to be
shared under MPL-2.0, but does NOT propagate to monocle's own codebase when nucleo
is used as a compiled dependency (dynamic or static linking). MPL-2.0 is compatible
with Apache-2.0 and MIT for binary distribution.

The vision and product brief are silent on monocle's own license. No license file,
no SPDX identifier, no `license` field in any hypothetical `Cargo.toml` exists
as of this ADR.

Production-grade enterprise/developer-tool projects must declare their license
before shipping code. CISA (SBOM guidance 2023) and the EU Cyber Resilience Act
(CRA, effective 2027) both require SBOM inclusion for open-source software
dependencies used in commercial contexts. The Rust ecosystem `cargo-deny` tool
enforces license compliance in CI and generates SBOM data; it requires a declared
workspace license.

The human decision (Q-license) is: **MIT OR Apache-2.0 dual-license**.

## Decision

monocle is dual-licensed under **MIT OR Apache-2.0** at the user's option.

This is expressed as:

- `license = "MIT OR Apache-2.0"` in the workspace `Cargo.toml` `[package]`
  section and in each crate's `[package]` section.
- SPDX identifier in every source file header:
  `// SPDX-License-Identifier: MIT OR Apache-2.0`
- `LICENSE-MIT` and `LICENSE-APACHE` files at the repository root.
- `cargo-deny` configuration (`deny.toml`) at workspace root, with license
  allowlist covering the full dependency chain.

## Rationale

### MIT OR Apache-2.0 is the canonical Rust ecosystem default

The major Rust infrastructure projects (rustc/stdlib, tokio, serde, ratatui, axum,
Anthropic's rmcp SDK) use MIT OR Apache-2.0. Developer tools that deviate from this
norm face friction in ecosystems that assume the dual-license: downstream consumers
may have policy rules that require Apache-2.0's explicit patent grant, while others
prefer MIT's brevity. Dual-licensing satisfies both camps with zero overhead.

### Apache-2.0 provides an explicit patent grant; MIT does not

For SBOM-era enterprise consumption (CRA compliance, US federal procurement, SOC 2
Type II supply-chain attestation), the Apache-2.0 patent grant is increasingly
required by legal and procurement teams. Providing only MIT exposes downstream
enterprise consumers to potential patent ambiguity. Dual-licensing gives them
Apache-2.0 when needed.

### All 24+ pinned dependencies are compatible

No dependency in the Phase 1–4 pin manifest carries GPL, AGPL, LGPL, or any
license incompatible with MIT OR Apache-2.0 binary distribution:

- MPL-2.0 (nucleo): compatible with MIT/Apache-2.0 for binary linking.
- Apache-2.0 (wasmtime, prost, russh): compatible with MIT/Apache-2.0.
- MIT (axum, tokio, ratatui, rmcp, etc.): trivially compatible.

`cargo-deny` with the canonical `deny.toml` enforces this at every PR.

**Canonical `deny.toml` content lives in
`SS-conventions-anti-patterns.md` §deny.toml configuration.** ADR-0003 captures
the license *selection* decision (MIT OR Apache-2.0 dual) that drives the license
allow-list; the `deny.toml` file content itself — including the exact `[licenses]`
allow/deny lists, `[bans]` entries, `[advisories]` settings, and MPL-2.0 rationale
for nucleo — is maintained in SS-conventions as the single authoritative source.
Any future change to the deny.toml allow-list (e.g., adding a new license for a
new dependency) is made in SS-conventions; this ADR does not need to change unless
the *license selection decision itself* changes (i.e., monocle's own license).

### Aligns with Anthropic SDK and monocle's interop story

monocle's raison d'être is Anthropic SDK integration. The Anthropic `rmcp` SDK is
Apache-2.0; Claude Code is proprietary. Dual MIT OR Apache-2.0 maximizes interop
surface with Anthropic's own licensing choices.

## Alternatives Considered

| License | Decision | Rejection Rationale |
|---------|----------|---------------------|
| Apache-2.0 only | Rejected | Less convenient for MIT-only downstream consumers; no technical advantage over dual |
| MIT only | Rejected | No explicit patent grant; insufficient for enterprise procurement in SBOM-era contexts |
| GPL-3.0 | Rejected | Incompatible with wasmtime (Apache-2.0), axum (MIT), prost (Apache-2.0); would require dropping multiple pinned dependencies with major scope impact |
| AGPL-3.0 | Rejected | Same incompatibility as GPL-3.0 plus server-side-use copyleft; directly contradicts monocle's developer-tool distribution model |
| BSL 1.1 (Business Source License) | Rejected | Non-OSI-approved; incompatible with Rust ecosystem norms; would block monocle from crates.io publication |
| Proprietary | Rejected | Human decision (Q-license) explicitly chose open-source |

## Consequences

### Implementation (devops-engineer companion work)

1. Add `license = "MIT OR Apache-2.0"` to workspace `Cargo.toml` and each crate
   `Cargo.toml` during Phase 1 Cargo workspace initialization.
2. Create `LICENSE-MIT` and `LICENSE-APACHE` at repository root (canonical text
   from OSI; year = 2026; copyright holder = "monocle contributors").
3. Add SPDX header to every new source file created in Phase 1:
   `// SPDX-License-Identifier: MIT OR Apache-2.0`
4. Add `deny.toml` to workspace root with the `cargo-deny` configuration above.
5. Add `cargo deny check licenses` to the CI matrix (devops-engineer wires this
   as part of the F-NEW-08 companion task, alongside `cargo audit`).

### SBOM

`cargo-deny` generates machine-readable license metadata as part of its check
output. For formal SBOM generation (SPDX 2.3 or CycloneDX 1.4), add `cargo-sbom`
or `cyclonedx-rust-cargo` to the CI matrix as a Phase 1 devops story. The license
declaration established by this ADR is the prerequisite for correct SBOM content.

### Future dependency additions

Any future dependency addition (across all phases) must be vetted against the
`cargo-deny` allowlist before merging. If a new dependency carries a license not
in the allowlist, the architect must evaluate compatibility and either add the
license to the allowlist (with explicit rationale) or reject the dependency.
GPL/AGPL/LGPL dependencies are blocked unconditionally.

## Re-eval Triggers

Reconsider this ADR if:

(a) monocle adds a copyleft dependency (GPL/LGPL/AGPL) in a future phase that
    cannot be isolated — a new ADR must address the resulting license compatibility
    constraint.

(b) Commercial-distribution requirements emerge that need a CLA (Contributor
    License Agreement) for IP consolidation — a new ADR must address CLA policy
    alongside the existing license.

(c) Anthropic SDK licensing changes in a way that affects binary distribution
    compatibility — re-evaluate allowlist.

None of these conditions are currently true as of 2026-05-12.

## Source / Origin

- **Adversary fresh-pass:** `/Users/jmagady/Dev/monocle/.factory/plans/adversary-pass-post-remediation.md` F-NEW-08 (IMPORTANT — no license, no SBOM, no OSS compliance strategy)
- **Human decision:** Q-license — MIT/Apache-2.0 dual selected
- **Dependency manifest:** `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md` — 24+ pinned dependencies; license compatibility verified against all Phase 1–4 pins
- **Devops companion:** cargo-deny CI gate and SBOM tooling (F-NEW-08 companion task dispatched to devops-engineer)
- **Canonical principle:** `CLAUDE.md` §Rule 1 — production-grade correctness required before Phase 1 ships; license declaration is a prerequisite for enterprise-grade distribution
