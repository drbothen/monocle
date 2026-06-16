---
document_type: behavioral-contract
level: L3
version: "1.0.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T05:12:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "b647859"
traces_to: prd.md
origin: greenfield
subsystem: SS-02
capability: CAP-002
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.02.003: Non-Exhaustive Enum Policy (FC-02)

## Description

Every `pub` enum in `monocle-core` must carry `#[non_exhaustive]` unless explicitly
exempted by an ADR. Verification is performed by a `syn 2` AST audit test in CI (NOT
clippy) that walks every `Item::Enum` node in `monocle-core/src/**/*.rs`. This policy
ensures that adding variants to public enums is never a breaking change for downstream
crates. At Phase 1, `Phase1Permission` and `ClaudeCodeTool` are the only ADR-0004-granted
exemptions.

## Preconditions

1. The `monocle-core` crate source tree is parseable via `syn 2` to enumerate all `pub` enum declarations.

## Postconditions

1. Every `pub` enum in `monocle-core` carries `#[non_exhaustive]` unless explicitly exempted by an ADR.
2. At Phase 1 PRD dispatch, the exhaustive-enum forbidden list contains exactly two entries: `Phase1Permission` and `ClaudeCodeTool` (both documented in ADR-0004).
3. Any new exemption requires a new ADR before the code compiles in CI. No exemption is granted by inline comment or spec prose alone.
4. The mandatory non-exhaustive enums include at minimum: `HookType`, `HookEvent`, `DenyReason`, `AllowPattern`, `DenyPattern`, `BlockingSeverity`, `SessionStatus`, `HookDecision`, `DeferUntil`.

## Invariants

1. The verification mechanism is a `syn 2` AST parse (NOT clippy). The test in `monocle-core/tests/enum_audit.rs` walks every `Item::Enum` node across all `.rs` files in `monocle-core/src/**/*.rs`, asserts `#[non_exhaustive]` is present unless the enum identifier is in the ADR-0004 EXEMPT list. This is deterministic and load-bearing; clippy's `non_exhaustive_omitted_patterns` lint is supplement only.
2. Adding a variant to any `#[non_exhaustive]` enum (except `Phase1Permission`) is NOT a breaking change and does NOT require a SemVer-major version bump.
3. `Phase1Permission` is exhaustive because the TUI permission dispatcher must handle every variant at compile time. Phase 3 adds `monocle-plugin-sdk::PluginPermission` as a separate enum rather than extending `Phase1Permission`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016 | New enum added in a future PR without `#[non_exhaustive]` | The `syn 2` AST audit test in CI rejects it unless an ADR is filed concurrently |
| EC-017 | `ClaudeCodeTool::Unknown(String)` catch-all variant | The enum is still exhaustive (every `match` must cover `Unknown`); the `Unknown` catch-all variant is the escape valve, not a non-exhaustive annotation |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| `syn 2` AST parse with a new `pub enum Foo { A, B }` (missing `#[non_exhaustive]`) | Test asserts error: enum `Foo` missing `#[non_exhaustive]` | edge-case |
| `syn 2` AST parse with `Phase1Permission` lacking `#[non_exhaustive]` | No error (ADR-0004 EXEMPT list) | happy-path |
| `syn 2` AST parse with `HookEvent` lacking `#[non_exhaustive]` | Test asserts error: enum `HookEvent` missing `#[non_exhaustive]` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-013 | `syn 2` AST audit in `monocle-core/tests/enum_audit.rs` rejects any `pub` enum in `monocle-core` lacking `#[non_exhaustive]` unless in ADR-0004 EXEMPT list | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability §SS-02 |
| Capability Anchor Justification | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability — this BC enforces the enum extensibility policy that is a core mechanism of the forward-compatible ABI contract |
| L2 Domain Invariants | DI-004 (all public wire types must carry a version discriminant as their first field — #[non_exhaustive] is the type-system mechanism that makes adding new variants non-breaking, which is the prerequisite for DI-004's "format evolution without parsing the full record" guarantee; enum extensibility and version discriminants are complementary halves of the forward-compatibility contract) |
| Architecture Module | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) per ARCH-INDEX Subsystem Registry SS-02 |
| Architecture Source | SS-core-types-and-abi.md v1.2.13 §Enum Extensibility |
| ADR | ADR-0004 (exhaustive-enum exemption rationale) |
| FC | FC-02 |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — non-exhaustive enum policy) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-TYPES-001 |
| Test name | test_BC_TYPES_001_non_exhaustive_enum_coverage |

## Related BCs (Recommended)

- [BC-2.02.001] — composes with: ABI version signals apply alongside enum extensibility for the full forward-compat contract
- [BC-2.03.001] — composes with: EngineModule-related enums (`HookEvent`, `HookDecision`, `DeferUntil`) are subject to this policy

## Architecture Anchors (Recommended)

- `architecture/SS-core-types-and-abi.md#enum-extensibility` — `#[non_exhaustive]` policy, ADR-0004 EXEMPT list, `syn 2` audit spec

## Story Anchor (Recommended)

S-TBD — Implement syn 2 enum audit test in monocle-core (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-013-non-exhaustive-enum-policy.md` — VP-013 enum audit lint test

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-002 per ARCH-INDEX is authoritative source`
  - After: `DI-004 ...`
  - DI-004 mapping: #[non_exhaustive] on public enums ensures new variants can be added without breaking downstream consumers. This is the type-system enforcement of DI-004's format evolution guarantee — adding a variant is equivalent to adding a new wire field, and non-exhaustive ensures consumers handle it without recompilation (wildcard arm). DI-004 and #[non_exhaustive] are complementary: DI-004 requires a version discriminant to detect change; #[non_exhaustive] ensures change is non-breaking.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T12:00:00Z (v1.0).

## §Trace v1.0.2

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.2.8 → v1.2.13** (2026-05-18T05:12:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.2.8 → v1.2.13); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-core-types-and-abi.md v1.2.8 §Enum Extensibility`
  - SE-17f AFTER: `SS-core-types-and-abi.md v1.2.13 §Enum Extensibility`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:12:00Z > prior 2026-05-17T18:00:00Z (v1.0.1). ARITHMETICALLY TRUE: 2026-05-18T05:12:00Z > 2026-05-17T18:00:00Z PASS.
