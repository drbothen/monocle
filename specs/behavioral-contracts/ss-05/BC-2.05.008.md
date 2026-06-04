---
document_type: behavioral-contract
level: L3
version: "1.0.8"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-30T00:00:00Z
phase: phase-1-expansion
inputs: [prd-expansion-scope.md, architecture/SS-ipc.md, architecture/ARCH-INDEX.md]
input-hash: "73990b1"
traces_to: prd.md
origin: greenfield
subsystem: SS-05
capability: CAP-005
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.05.008: UDS-Only in Phase 1 (No Shared-Memory Transport)

## Description

The `monocle-ipc` crate implements only the Unix domain socket transport in Phase 1 (OQ-08
decision from the product brief). No `mmap`, `shm_open`, `shared_memory` crate, or any other
POSIX shared-memory primitive may be used in `monocle-ipc` in Phase 1. This prohibition is
enforced at three layers: the `#![forbid(unsafe_code)]` crate attribute (preventing inline
unsafe blocks that could implement shared memory), a `cargo deny` rule (blocking the import
of shared-memory crates), and a semgrep check in CI (detecting direct `libc::mmap` or
`nix::sys::mman` usage). The Phase 4 shared-memory transport is designed for via the
`Transport` trait abstraction — no structural changes to `monocle-ipc` are needed when Phase
4 arrives; only a new `ShmTransport` struct in a separate `monocle-ipc-shm` crate.

## Preconditions

1. The `monocle-ipc` crate is being built from source (Phase 1 build).
2. CI is configured with the `cargo deny`, `semgrep`, and `clippy` checks described below.

## Postconditions

**Build-time enforcement:**
1. The `monocle-ipc` crate root carries `#![forbid(unsafe_code)]`. Any attempt to add
   inline `unsafe { ... }` blocks is rejected by the Rust compiler with a hard error.
2. The `Cargo.toml` for `monocle-ipc` does NOT list any of the following crates as
   dependencies (direct or transitive):
   - `shared_memory` (any version)
   - `raw-sync` (shared-memory companion crate)
   - `ipc-channel` (uses shared memory on some platforms)
   - Any crate providing `shm_open` bindings outside the standard `libc` crate.
3. The `cargo deny` configuration includes a deny rule for `shared_memory`, `raw-sync`,
   and `ipc-channel`. CI fails if any of these crates appear in the dependency graph.
4. The semgrep CI check scans `monocle-ipc/src/**/*.rs` for any of the following patterns
   and fails if any match:
   - `libc::mmap`
   - `nix::sys::mman`
   - `shm_open`
   - `mmap_rs`
   - `memmap2` (allowed only in `monocle-ipc-shm` if that crate is created; banned in `monocle-ipc`)

**Runtime assertion:**
5. The `UdsTransport` is the sole implementation of the `Transport` trait in the
   `monocle-ipc` crate in Phase 1. The `Transport` trait definition is present in
   `monocle-ipc` (it is the abstraction point for Phase 4). No other struct implements
   `Transport` in this crate during Phase 1.

**Phase 4 forward-compat:**
6. The `Transport` trait is defined in `monocle-ipc`:
   ```rust
   #[async_trait]
   pub trait Transport: Send + Sync + 'static {
       async fn send_message(&mut self, msg: &ServerToClient) -> Result<(), IpcError>;
       async fn recv_message(&mut self) -> Result<ClientToServer, IpcError>;
   }
   ```
   When Phase 4 arrives, `ShmTransport` will implement this trait in a new crate
   (`monocle-ipc-shm`). No changes to `monocle-ipc` are required for Phase 4 transport
   addition.

## Invariants

1. `monocle-ipc` is a pure UDS transport in Phase 1. Any code review finding shared-memory
   primitives in `monocle-ipc/src/` is a CRITICAL defect under the production-grade default.
2. The `#![forbid(unsafe_code)]` attribute must not be removed or overridden with
   `#![allow(unsafe_code)]` in `monocle-ipc`. If Phase 4 `ShmTransport` requires unsafe code,
   it lives in `monocle-ipc-shm` with an explicit `#![allow(unsafe_code)]` and a security
   review gate.
3. The `Transport` trait is stable for Phase 4. Adding a `ShmTransport` does not require
   changing `Transport`'s method signatures.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Developer accidentally adds `memmap2` as a `monocle-ipc` dependency | `cargo deny` fails in CI with: "crate `memmap2` is explicitly denied". PR blocked. |
| EC-002 | Developer adds an inline `unsafe` block to `monocle-ipc/src/framing.rs` | Rustc fails to compile with: "error: usage of an `unsafe` block (forbidden by `forbid(unsafe_code)`)". |
| EC-003 | Dependency audit identifies a transitive dependency of `monocle-ipc` that includes `libc::mmap` calls | `cargo deny` (or a security scan) flags the transitive dependency. The crate must be replaced or the `monocle-ipc` feature flags adjusted to exclude the shared-memory path. This is a MEDIUM severity finding under the production-grade default. |
| EC-004 | Phase 4 development begins; contributor adds `ShmTransport` to `monocle-ipc/src/` | Code review catches the violation of BC-2.05.008 (wrong crate). The contributor is directed to create `monocle-ipc-shm` as a separate crate per the architecture doc. |
| EC-005 | `Transport` trait signature change proposed (e.g., adding a new method) | Any change to the `Transport` trait is a breaking ABI change for all implementors. It requires a new BC revision and an ADR documenting the rationale. The trait is intentionally minimal (send/recv) to minimize Phase 4 friction. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `cargo build --package monocle-ipc` with no shared-memory deps | Build succeeds; no warnings about shared-memory | happy-path |
| `cargo deny check` on `monocle-ipc` Cargo.lock | No denied crates; exit code 0 | happy-path |
| Add `memmap2 = "2"` to `monocle-ipc/Cargo.toml` and run `cargo deny check` | `cargo deny` fails: `memmap2` is denied; exit code non-zero | error |
| Add `unsafe { libc::mmap(...) }` to `monocle-ipc/src/lib.rs` | Rustc compile error: `unsafe` block forbidden; exit code non-zero | error |
| `grep -r "libc::mmap\|shm_open\|nix::sys::mman" monocle-ipc/src/` | Zero matches | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `#![forbid(unsafe_code)]` present in `monocle-ipc/src/lib.rs` | static analysis (grep) |
| VP-TBD | `cargo deny check` passes with shared-memory crates in deny list | CI (cargo deny) |
| VP-TBD | No `libc::mmap`, `shm_open`, or `nix::sys::mman` in `monocle-ipc/src/**` | CI (semgrep) |
| VP-TBD | `Transport` trait has exactly 2 methods: `send_message` and `recv_message` | static analysis |
| VP-TBD | `UdsTransport` is the only `Transport` implementor in `monocle-ipc` | static analysis (rustc trait implementations) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability §SS-05 — this BC constrains the transport implementation to UDS-only, which defines the Phase 1 boundary of the internal transport capability and provides the `Transport` trait abstraction point for Phase 4 |
| L2 Domain Invariants | DI-007 (monocle must not write to harness-owned files — shared-memory primitives could theoretically be used to violate this; the prohibition on shared-memory in monocle-ipc upholds DI-007's spirit at the transport layer) |
| Architecture Module | monocle-ipc (Transport trait, UdsTransport, `#![forbid(unsafe_code)]`) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-ipc.md v1.17.0 §Transport Layer §Transport Trait; SS-ipc.md v1.17.0 §Phase 1 Transport Constraint |
| Cross-Ref | SS-deps-pin-manifest.md §cargo-deny rules (shared-memory deny list); SS-conventions-anti-patterns.md §Forbidden Patterns (shared-memory primitives) |
| Test File | CI enforcement (cargo deny, semgrep, rustc compile gate) — not an integration test |
| Test Name | `test_BC_2_05_008_uds_only_constraint` (static analysis CI job) |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.02.003] — composes with: non-exhaustive enum policy applies to `Transport` trait implementors in future phases

## Architecture Anchors

- `architecture/SS-ipc.md#phase-1-transport-constraint` — `#![forbid(unsafe_code)]`, cargo deny rules, semgrep check, ShmTransport separate crate plan
- `architecture/SS-ipc.md#transport-layer` — `Transport` trait definition for Phase 4 extensibility

## Story Anchor

S-TBD — Implement Transport trait with UdsTransport; configure cargo deny and semgrep for shared-memory prohibition (filled by story-writer)

## VP Anchors

VP-TBD — UDS-only constraint static analysis verification (filled after VP creation)

## §Trace v1.0.0

**1.0.5** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers and time qualifiers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at authoring time. No normative content changed.

**Initial production** (2026-05-26T04:00:00Z):
- BC-2.05.008 authored for SS-05 IPC subsystem per `prd-expansion-scope.md §3.2` and
  `SS-ipc.md §Transport Layer §Transport Trait + §Phase 1 Transport Constraint`.
- Covers: `#![forbid(unsafe_code)]` crate attribute, `cargo deny` rule for shared-memory
  crates (shared_memory, raw-sync, ipc-channel), semgrep CI pattern matching (libc::mmap,
  nix::sys::mman, shm_open, mmap_rs, memmap2), `Transport` trait definition for Phase 4
  abstraction, UdsTransport as sole Phase 1 implementor, ShmTransport in separate
  `monocle-ipc-shm` crate for Phase 4.
- 5 edge cases documented (EC-001..EC-005).
- Priority P1 (not P0) because this is a constraint enforcement contract, not a user-visible
  behavioral feature; it is enforced entirely at build/CI time rather than at runtime.
- SE-16d PASS: 2026-05-26T04:00:00Z is the production timestamp for this wave.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.0.0` → `SS-ipc.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-004 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.1.0` (2 occurrences) → `SS-ipc.md v1.3.0` per F-P1D4-004 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.3.0` (2 occurrences) → `SS-ipc.md v1.4.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: three-pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-ipc.md v1.4.0 §Transport Layer §Transport Trait` → `SS-ipc.md v1.9.0 §Transport Layer §Transport Trait`; `SS-ipc.md v1.4.0 §Phase 1 Transport Constraint` → `SS-ipc.md v1.9.0 §Phase 1 Transport Constraint`.
- Cross-Ref row: `SS-deps-pin-manifest.md v1.1.17 §cargo-deny rules` → `SS-deps-pin-manifest.md v1.2.0 §cargo-deny rules`. Plain pin refresh — cargo-deny shared-memory deny list content unchanged.
- Cross-Ref row: `SS-conventions-anti-patterns.md v1.29.5 §Forbidden Patterns` → `SS-conventions-anti-patterns.md v1.31.1 §Forbidden Patterns`. Plain pin refresh — §Forbidden Patterns shared-memory primitives entry unchanged.
- All three refreshes are plain version-pin refreshes; no substantive content propagation required.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.0.6

**POL-11 remediation: SS-conventions-anti-patterns Cross-Ref pin v1.32.3 → v1.32.4** (2026-05-30):
- Cross-Ref row: `SS-conventions-anti-patterns.md v1.32.3 §Forbidden Patterns` → `SS-conventions-anti-patterns.md §Forbidden Patterns` (Option 1 per ADR-0007 §Decision — active navigation pointer, not historical provenance).
- SS-conventions canonical version is v1.32.4 per `version-pin-registry.yaml`.
- Version bumped v1.0.5 → v1.0.6.
- SE-16d monotonicity: v1.0.6 timestamp 2026-05-30 >= v1.0.4 timestamp 2026-05-29. PASS.
## §Trace 1.0.7 — POL-11 cascade remediation (2026-05-30)

**Bump:** 1.0.6 → 1.0.7.
**Scope:** Cross-Ref table: `SS-conventions-anti-patterns.md v1.32.4 §Forbidden Patterns` → `SS-conventions-anti-patterns.md §Forbidden Patterns` (Option 2 version-free; cascade from SS-conventions-anti-patterns v1.32.4 → v1.32.5 bump in same remediation burst; version-free permanently prevents re-staling).
**SE-16d PASS:** 2026-05-30 >= 2026-05-30 (same-day patch).
