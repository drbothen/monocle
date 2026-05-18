---
document_type: domain-spec-section
level: L2
section: "CAP-002 Forward-Compatible Wire Formats"
capability: CAP-002
version: "1.0"
status: active
producer: vsdd-factory:business-analyst
timestamp: 2026-05-17T14:00:00Z
phase: 1a
inputs:
  - product-brief.md
  - research/domain-monocle-vision-synthesis.md
input-hash: "a6216e9"
traces_to: L2-INDEX.md
subsystem: SS-02
bcs:
  - BC-2.02.001
  - BC-2.02.002
  - BC-2.02.003
  - BC-2.02.004
  - BC-2.02.005
  - BC-2.02.006
  - BC-2.02.007
  - BC-2.02.008
---

# CAP-002: Forward-Compatible Wire Formats

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`. This section
> describes the Forward-Compatible Wire Formats domain capability at the
> problem-domain level. Implementation contracts live in
> `behavioral-contracts/ss-02/`.

## Capability Statement

CAP-002 covers the domain obligation to ship Phase 1 with wire formats, ABI
constants, and type extensibility rules that allow Phase 2, 3, and 4 software
to read Phase 1 data without breaking changes. This capability is the reason
monocle can evolve its plugin SDK, cross-host federation, and trigger-trace
tooling without forcing harness operators to restart or migrate data.

**Anchor justification:** CAP-002 covers this scope because the product brief
§Forward-compatibility contracts (FC-01 through FC-06) explicitly names six
forward-compatibility items locked pre-Phase-1 by human authorization (brief
v1.4.7 §Scope). Vision §Process Topology and §Key Abstractions establish that
monocle's value requires multi-phase continuity: Phase 3 plugin SDK, Phase 4
cross-host federation, and Phase 2 trigger-trace all depend on Phase 1 wire
types being readable by future software.

## Domain Entities

### AbiVersionConst

The `MONOCLE_ABI_VERSION: u32` constant exported from `monocle-core`. The
single authoritative number that identifies the daemon's public contract version.

| Attribute | Type | Description |
|-----------|------|-------------|
| value | u32 | Always 1 in Phase 1 |
| exposure | /status endpoint | Returned by the daemon's status endpoint so Phase 3 plugins can refuse incompatible daemons |

### PublicEnum (domain concept, not a single type)

Any public enum in `monocle-core` that forms part of the wire or plugin ABI.
Phase 2+ additions to these enums must not break existing `match` exhaustive
arms in downstream code.

| Attribute | Constraint | Description |
|-----------|------------|-------------|
| extensibility | `#[non_exhaustive]` | All public enums carry this attribute UNLESS explicitly exempted by ADR |
| exemptions | Phase1Permission, ClaudeCodeTool | These two enums are exhaustive by design (ADR-0004); new variants require explicit ADR |

The exhaustive-by-design exemption for `Phase1Permission` and `ClaudeCodeTool`
is a domain rule, not an implementation convenience: the canonical set of Claude
Code permission decisions and tool names is defined externally (by Anthropic); monocle
mirrors that set rather than extending it independently.

### AuthToken

The credential that hook scripts supply to authenticate with the daemon. Phase 4
federation may introduce OAuth2-style Bearer tokens; the Phase 1 prefix format
ensures the two token families can coexist without parser ambiguity.

| Attribute | Type | Description |
|-----------|------|-------------|
| format | `monocle-v1:<64-char-hex>` | Phase 1 canonical format (FC-06) |
| prefix | `monocle-v1:` | Non-negotiable; tokens lacking this prefix are rejected with HTTP 401 |
| hex_body | 64 hex characters | Cryptographically random; generated once per daemon startup |

### HookEnvelope (protobuf)

The protobuf wire container for hook events used in Phase 4 cross-host federation.
Defined in `monocle-proto` using prost-generated types. Phase 1 ships the type
definitions with zero runtime cost; they become active in Phase 4.

| Attribute | Type | Description |
|-----------|------|-------------|
| schema_version | uint32, field 1 | Always 1 in Phase 1; field number 1 is reserved and immutable (FC-05) |
| event_payload | oneof | One of the 5 event message types |

The field-number contract is a domain invariant: once `schema_version` is field 1
in the protobuf schema, it cannot be renumbered without breaking all Phase 4
federation endpoints that decode these messages. This constraint predates any
implementation decision about how protobuf is encoded.

### FactoryState (domain concept)

The parsed representation of a factory-pattern project's workflow state, as
understood by monocle's Workflow plane. A `FactoryAdapter` produces a
`FactoryState` from the factory's files.

| Attribute | Type | Description |
|-----------|------|-------------|
| phase | string | Current pipeline phase |
| status | string | Active / blocked / awaiting |
| awaiting | string or absent | What the pipeline is waiting for (e.g., "human GO") |
| blocking_issues | list of issues | Issues preventing phase advance |
| convergence | optional | Convergence trajectory summary |

## Domain Processes

### P5: ABI Version Check (Phase 3+)

1. A Phase 3 WASM plugin or Phase 4 federation peer reads the daemon's `/status`
   endpoint.
2. The endpoint returns `{"abi_version": 1, ...}`.
3. The plugin checks `abi_version` against its own compatibility matrix.
4. If incompatible, the plugin refuses to load and reports the version mismatch.
5. This process is defined in Phase 1 because the `/status` endpoint and the
   `MONOCLE_ABI_VERSION` constant must both exist in Phase 1 for the Phase 3
   check to work.

### P6: Enum Variant Addition (Phase 2+)

1. A future monocle version adds a new variant to a `#[non_exhaustive]` public
   enum (e.g., a new `HookType`).
2. Phase 1 code that `match`es on `HookType` without a wildcard arm will not
   compile against the new SDK — the `#[non_exhaustive]` attribute ensures the
   compiler requires a `_ =>` wildcard.
3. Phase 1 plugin binaries compiled against the Phase 1 SDK will have wildcard
   arms and will not crash when receiving an unknown variant at runtime.
4. This process is defined in Phase 1 because the `#[non_exhaustive]` attribute
   must be present from day one; it cannot be added retroactively without breaking
   existing match arms that were written exhaustively against Phase 1.

### P7: Ring Record Reading (Phase 2 Trigger-Trace)

1. Phase 2 trigger-trace reads Phase 1 JSONL ring entries.
2. Reader checks `format_version` (first key in every record, FC-01).
3. If `format_version == 1`, reader applies the Phase 1 schema.
4. If `format_version > 1` (future), reader applies the corresponding schema
   or skips the record with a warning.
5. This process is defined in Phase 1 because the `format_version` key must be
   first in every record from Phase 1 day one (DI-004).

## Domain Invariants

### DI-004: Version-First Wire Rule

All public wire types MUST carry a version discriminant as their first field so
that readers can detect format evolution without parsing the full record.

**Justification:** DI-004 is a business invariant because monocle's multi-phase
roadmap (Phase 1 → 4) depends on Phase 2+ tools being able to read Phase 1 data.
If version fields are added later or placed non-first, readers cannot cheaply
skip or branch on unknown formats. Source: brief §Forward-compatibility contracts
FC-01 (JSONL ring), FC-05 (protobuf HookEnvelope), both locked pre-Phase-1.

### DI-005: Auth Token Prefix Invariant

A monocle daemon MUST NOT accept an auth token that does not begin with the
canonical version prefix for its phase.

**Justification:** DI-005 is a business invariant because the prefix scheme is
the only mechanism that allows Phase 4 federation to introduce OAuth2 Bearer
tokens on the same HTTP interface without parser ambiguity. Accepting prefix-free
tokens would make the two token families indistinguishable at the parsing layer.
Source: brief §Forward-compatibility contracts FC-06, brief §Success Criteria
"Forward-compatibility contracts" row.

## BC Cross-References

All 8 BCs in SS-02 operationalize CAP-002. See `behavioral-contracts/BC-INDEX.md`
§SS-02 for the full list.

| BC ID | Title | Operationalizes |
|-------|-------|-----------------|
| BC-2.02.001 | ABI Version in /status Endpoint (FC-03) | AbiVersionConst entity, P5 ABI Version Check |
| BC-2.02.002 | ABI Version Constant at Crate Root (FC-03) | AbiVersionConst entity |
| BC-2.02.003 | Non-Exhaustive Enum Policy (FC-02) | PublicEnum entity, P6 Enum Variant Addition |
| BC-2.02.004 | FactoryAdapter Trait Definition (FC-04 CRITICAL) | FactoryState entity (trait produces it) |
| BC-2.02.005 | VsddFactoryAdapter Implementation | FactoryState entity (built-in impl) |
| BC-2.02.006 | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | HookEnvelope entity, DI-004 |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | HookEnvelope entity |
| BC-2.02.008 | Phase 4 schema_version Validation Requirement (FC-05) | HookEnvelope entity, P5 ABI Version Check |
