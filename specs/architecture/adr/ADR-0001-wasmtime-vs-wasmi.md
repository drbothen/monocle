---
document_type: adr
adr_id: ADR-0001
status: accepted
date: 2026-05-12
subsystems_affected: []
supersedes: null
superseded_by: null
level: L3
version: "1.0.3"
producer: product-owner (extracted from brief v1.1)
phase: pre-phase-1-architecture
timestamp: 2026-05-17T16:30:00Z
inputs: [research/domain-monocle-vision-synthesis.md, product-brief.md]
input-hash: "6c55009"
traces_to: "factory-artifacts ee09833 (brief v1.1); consistency-audit 0f28619; validate-brief v4 38b8e8f"
project: monocle
---

# ADR-0001: wasmtime vs wasmi for WASM Plugin Runtime

## Status

Accepted

## Context

monocle's plugin SDK (Phase 3+) needs a WASM runtime to host third-party
factory-adapter binaries. Plugin authors write `EngineModule` and `FactoryAdapter`
implementations in a guest language compiled to WASM; monocle's host loads and
executes these binaries in an isolated sandbox. The choice of WASM runtime
affects: binary size, cold-start latency, JIT throughput for non-trivial logic,
security advisory cadence, and Phase 1 MSRV constraints.

The two realistic options in the Rust WASM ecosystem as of 2026-05-12 are
wasmtime (Bytecode Alliance, JIT-based, WASI-first) and wasmi (now 1.0 with
mature WASI support, interpreter-based).

## Decision

**wasmtime 44** is selected as the WASM runtime for monocle's plugin SDK.

## Rationale

wasmtime 44 is preferred over wasmi. wasmi 1.0 is now mature with WASI support,
so the historical "WASI gap" rationale no longer applies. Monocle prefers
wasmtime for two reasons: (1) JIT throughput for factory adapters that may
execute non-trivial pipeline logic, and (2) actively-maintained security posture
— wasmtime's Bytecode Alliance publishes security advisories on a tight cadence
(multiple advisories in 2026 alone) and ships patches promptly. wasmi remains a
future fallback if binary-size pressure becomes a release constraint.

## Consequences

### Positive

- JIT throughput adequate for factory adapters executing non-trivial pipeline logic
- Bytecode Alliance security advisory cadence provides timely patches on CVEs
- Guest WASM ABI is runtime-agnostic; wasmi swap possible without breaking plugins

### Negative / Trade-offs

- Phase 3 MSRV bumps from 1.86 to 1.92 due to wasmtime requirements
- Binary size increases ~12MB; acceptable for desktop developer tool
- Brings wasmtime RUSTSEC advisory surface (pre-44 majors carry multiple CVEs — pin to 44)

### Status as of 2026-05-12

Pre-implementation (Phase 3 deliverable). Decision accepted at brief v1.1 (at time of
ADR authoring); rationale extracted to this ADR during the brief v1.2 revision (also at
time of ADR authoring). wasmtime 44 pinned in SS-deps-pin-manifest.md; no code shipped yet.

## Alternatives Considered

- **wasmi 1.0:** Rejected (Phase 1) — Interpreter-based; higher latency for non-trivial pipeline logic; slower advisory cadence than Bytecode Alliance; binary size advantage insufficient to outweigh throughput cost.
- **wasmer:** Not evaluated — LGPL licensing ambiguity; not required for monocle's use case.

## Source / Origin

- **Master design doc:** vision §Tech Stack (human-approved, D-012) specifies wasmtime as the WASM runtime
- **Product brief:** `/Users/jmagady/Dev/monocle/.factory/specs/product-brief.md` v1.1 at time of ADR authoring, Constraints & Integration Points, wasmtime vs wasmi rationale paragraph
- **Dependencies manifest:** `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md` §Phase 1 Pin Manifest row wasmtime 44
- **Gene source:** `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md` §plugin (zellij-tile WASM plugin SDK model)
- **RUSTSEC context:** RUSTSEC-2026-0114, 0095, 0096, 0006, 0020 on pre-44 wasmtime majors

## Amendment History

v1.0.2 changes (round-57.1 PG-5 ADR-class sweep):
- PG-5 sweep: §Consequences `### Status as of 2026-05-12` cited `brief v1.1` (bare version,
  neither current nor explicitly historical). Fix: Form 2 historical-anchor applied —
  "at brief v1.1 (at time of ADR authoring)". §Source / Origin cited
  `product-brief.md` v1.1 without historical qualifier. Fix: "v1.1 at time of ADR
  authoring" added. Both citations are provenance records; historical framing is accurate.
  `traces_to` frontmatter (also cites brief v1.1) is exempt per PG-5 Option B carve-out
  (frontmatter exempted in SS-conventions-anti-patterns.md v1.25).
