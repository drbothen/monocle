---
document_type: story
level: L4
story_id: S-013
epic_id: EPIC-02
version: "1.4"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001]
blocks: [S-021]
target_module: monocle-proto
subsystems: [SS-02]
behavioral_contracts: [BC-2.02.006, BC-2.02.007, BC-2.02.008]
verification_properties: [VP-016, VP-017, VP-018]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.006.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.007.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.008.md, version: "1.0.3"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-016-hook-envelope-proto-field-numbers.md, version: "1.0.12"}
  - {path: .factory/specs/verification-properties/vp-017-hook-envelope-schema-version-field.md, version: "1.0.13"}
  - {path: .factory/specs/verification-properties/vp-018-phase4-schema-version-validation.md, version: "1.0.12"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/architecture/SS-forward-compatibility.md, version: "1.2.19"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.02.006 (HookEnvelope Proto Field Number), BC-2.02.007 (HookEnvelope prost-build-generated Rust Struct schema_version), BC-2.02.008 (Phase 4 schema_version Validation); verifies VP-016, VP-017, VP-018; addresses E-PROTO-001; depends_on [S-001] (monocle-proto crate stub, no monocle-core symbols consumed)."
---

# S-013: HookEnvelope Proto Wire Format (FC-05)

## Narrative

As a Phase 4 federation layer consumer, I want the `HookEnvelope` protobuf message to
have `schema_version` as field number 1 and the corresponding Rust struct to expose
`pub schema_version: u32` with value `1`, so that Phase 4 can version-dispatch on the
first decoded field without parsing the full protobuf message.

## Acceptance Criteria

### AC-001 (traces to BC-2.02.006 postcondition 1 — proto field number 1 = schema_version)
In the `HookEnvelope` protobuf message definition (`monocle-proto/proto/monocle/v1/hook_envelope.proto`),
`schema_version` is field number 1. This field number is IMMUTABLE per FC-05 (immutability invariant
defined in `SS-forward-compatibility.md §FC-05`) — any change is a BREAKING change requiring
an ADR.

### AC-002 (traces to BC-2.02.007 postcondition 1 — Rust struct schema_version field)
The generated/hand-written `HookEnvelope` Rust struct in `monocle-proto` exposes
`pub schema_version: u32`. In Phase 1, `schema_version` value is always `1`.

### AC-003 (traces to BC-2.02.007 postcondition 2 — schema_version value 1 in Phase 1)
Integration test: create a `HookEnvelope { schema_version: 1, ... }` and assert the field
value equals `1`. No Phase 1 code path sets `schema_version` to any other value.

### AC-004 (traces to BC-2.02.008 postcondition 1 — Phase 4 dispatch contract documented)
`monocle-proto/src/lib.rs` includes a rustdoc comment on `HookEnvelope` documenting the
Phase 4 schema_version dispatch contract: unknown `schema_version` values (>1) must trigger
`WARN: HookEnvelope schema_version <N> not recognized; skipping` (E-PROTO-001).
Executable oracle: a `grep`-based test in `monocle-proto/tests/schema_version.rs` asserts the
E-PROTO-001 warning string substring (e.g., `"not recognized"` OR `"E-PROTO-001"`) appears in the
rustdoc comment on the prost-build-generated or hand-annotated `HookEnvelope` in `monocle-proto/src/lib.rs`.
Alternatively: `cargo doc --no-deps` build assertion confirms rustdoc compilation succeeds and
the warning text is present in the generated HTML output.

### AC-005 (traces to BC-2.02.008 postcondition 2 — Phase 1 structural recap; negative oracle)
Phase 1 does not use protobuf encoding for any runtime wire path (hook POSTs use JSON).
`monocle-proto` declares `prost 0.14` as a dependency but NO Phase 1 code path invokes
protobuf serialization. The proto schema is declared now to lock the field number contract
before Phase 4 activation.
Negative oracle (executable): `grep -r "schema_version" --include='*.rs' monocle-proto/src/` asserts
that ONLY literal `1` appears as the `schema_version` value in Phase 1 source (no `schema_version: 0`,
`schema_version: 2`, etc.). Alternatively: a clippy-deny lint or test assertion confirms the constant
`HOOK_ENVELOPE_SCHEMA_VERSION` (if declared) equals `1u32`.

### AC-006 (traces to BC-2.02.006 invariant 1 — field number immutability)
`monocle-proto/tests/wire_field_order.rs` integration test (canonical name per SS-core-types-and-abi.md
line 982: "a `monocle-proto/tests/wire_field_order.rs` test round-trips a message and asserts the field
number assignment via prost-build's generated descriptor") builds via prost-build and decodes an encoded
`HookEnvelope` using the prost-build generated `FileDescriptor` (or `prost-reflect`), asserting:
`descriptor.field_by_number(1).name() == "schema_version"`.
Text-parse of the `.proto` source is NOT the canonical oracle — the descriptor decode is. This test
fails if the proto is regenerated with `schema_version` at any field number other than 1.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~700 |
| BC-2.02.006.md | ~500 |
| BC-2.02.007.md | ~500 |
| BC-2.02.008.md | ~500 |
| VP-016 + VP-017 + VP-018 files | ~1,200 |
| SS-core-types-and-abi.md (§Prost Wire Schemas, ~80 lines) | ~1,200 |
| SS-forward-compatibility.md §FC-05 (~20 lines) | ~300 |
| prost-build documentation | ~400 |
| Test files (wire_field_order.rs + schema_version.rs) | ~800 |
| **Total estimate** | **~6,100** |

## Tasks

- [ ] Create `monocle-proto/proto/monocle/v1/hook_envelope.proto` with canonical schema
  (versioned package dir per SS-core-types-and-abi.md §Prost Wire Schemas (FC-05 resolution) line 906;
  F-D-02 + F-D-04 fix — was unversioned path `monocle-proto/proto/hook_envelope.proto`):
  ```protobuf
  syntax = "proto3";
  package monocle.v1;

  // HookEnvelope is the canonical wire message for every hook event.
  // Phase 1 defines this schema; Phase 4 activates the wire path.
  // Field numbers 1-99: stable Phase 1 core fields.
  // Field numbers 100-999: reserved for Phase 4 federation additions.
  // Field numbers 1000+: reserved for Phase 5+.
  message HookEnvelope {
    uint32 schema_version  = 1;  // Always 1 for Phase 1 messages.
    string session_id      = 2;  // Claude Code session identifier.
    int64  timestamp_micros = 3; // Event timestamp, UTC microseconds since Unix epoch.
    uint32 pid             = 4;  // PID of the Claude Code process that fired the hook.

    oneof event {
      SessionStartEvent    session_start  = 10;
      UserPromptSubmitEvent prompt_submit = 11;
      PreToolUseEvent      pre_tool_use   = 12;
      NotificationEvent    notification   = 13;
      StopEvent            stop           = 14;
    }
  }

  message SessionStartEvent {
    string cwd             = 1;
    string transcript_path = 2;
  }

  message UserPromptSubmitEvent {
    string prompt = 1;
  }

  message PreToolUseEvent {
    string tool_name  = 1;
    bytes  tool_input = 2;
  }

  message NotificationEvent {
    string notification_type = 1;
    string tool_name         = 2;
    bytes  tool_input        = 3;
    string message           = 4;
  }

  message StopEvent {
    string stop_reason = 1;
  }
  ```
  (VERBATIM from SS-core-types-and-abi.md v1.2.13 lines 915-960; was 3-field sketch — F-D-01/F-D-02 CRITICAL fix)

- [ ] Create `monocle-proto/build.rs` invoking prost-build codegen (F-D-01 + F-D-03 fix — drop hand-written struct):
  ```rust
  fn main() {
      prost_build::Config::new()
          .compile_protos(
              &["proto/monocle/v1/hook_envelope.proto"],
              &["proto/"],
          )
          .expect("prost-build codegen failed");
  }
  ```
  (prost-build generates Rust types at build time; Phase 1 compiles types into binary but activates no
  wire path; Phase 4 activates wire path without any Phase 4 workspace changes to `monocle-proto`)

- [ ] Create `monocle-proto/src/lib.rs`:
  - Include the prost-build generated module: `pub mod monocle { pub mod v1 { include!(concat!(env!("OUT_DIR"), "/monocle.v1.rs")); } }`
  - Add rustdoc comment on `HookEnvelope` re-export documenting Phase 4 dispatch contract (E-PROTO-001)
    and FC-05 immutability invariant (`schema_version: u32 = 1` is immutable across Phase 1 minor versions)
  - NOTE: Workspace must already declare `bytes = "1.11"` (caret) per S-001 to neutralize RUSTSEC-2026-0007
    in prost transitive resolution (F-A-02)

- [ ] Create `monocle-proto/tests/wire_field_order.rs` (VP-016 / BC-2.02.006 verification; F-D-05 fix):
  - Build via prost-build, decode an encoded `HookEnvelope` using generated `FileDescriptor`
    (or `prost-reflect`), assert `descriptor.field_by_number(1).name() == "schema_version"`
  - (was `proto_field_numbers.rs` — canonical name per SS-core-types-and-abi.md line 982)

- [ ] Create `monocle-proto/tests/schema_version.rs` (VP-017/VP-018 / BC-2.02.007 verification):
  - Construct prost-generated `HookEnvelope` with `schema_version: 1` and assert field value == 1
  - Assert E-PROTO-001 warning string substring present in rustdoc (AC-004 executable oracle)

## Previous Story Intelligence

S-001 (Wave 1): `monocle-proto` crate stub created as part of workspace init. `prost 0.14` is pinned
in workspace (EXACT pin). `bytes 1.11` (caret) declared in workspace per S-001 to neutralize
RUSTSEC-2026-0007 in prost's transitive dependency resolution. `monocle-proto` does NOT consume any
`monocle-core` symbols — it only depends on `prost` and `prost-build`. Dependency is therefore
`depends_on: [S-001]` not `[S-010]` (F-E-01 fix: S-010 creates monocle-core; monocle-proto
does not import from monocle-core; no monocle-core symbols consumed here).
In Phase 1, prost-build generates Rust types at compile time but no wire path is activated at runtime.
Phase 4 activates the wire path without any Phase 4 changes to `monocle-proto`.

## Architecture Compliance Rules

From `architecture/SS-core-types-and-abi.md` v1.2.13 §Prost Wire Schemas (FC-05 resolution) line 884:
- Field number 1 = `schema_version` is IMMUTABLE — changing it is a BREAKING change requiring ADR
  (FC-05 immutability invariant: `schema_version: u32 = 1` is immutable across Phase 1 minor versions;
  defined in `SS-forward-compatibility.md §FC-05`)
- Phase 1: prost-build generates types at compile time but NO proto encoding/decoding occurs at runtime
- Phase 4: prost activates for cross-host federation wire format decoding of untrusted input

**Field Number Reservation Scheme** (SS-core-types-and-abi.md lines 894-901):
- Fields 1-99: envelope routing fields (stable Phase 1 core; immutable)
- Fields 100-999: reserved for Phase 4 federation additions (e.g., `peer_origin_host = 100`)
- Fields 1000+: reserved for Phase 5+
- Phase 4 federation MUST respect these ranges; violations require an ADR

**FC-05 Immutability** (SS-forward-compatibility.md §FC-05):
- `schema_version: u32 = 1` field at proto field number 1 is immutable across Phase 1 minor versions.
  Any change to a Phase 1 field (numbers 1-99) is a BREAKING change: bump `schema_version` AND produce an ADR.

**Downstream Impact** (Phase 4 federation consumer):
- Phase 4 federation consumer (no Phase 1 story) depends on this schema's field-number lock.
- Any change post-merge requires bumping `schema_version` and producing an ADR.
- Phase 4 deserialization of Phase 1 messages MUST succeed even if Phase 4 receiver knows additional fields
  (proto3 forward compatibility guarantee: unknown fields are preserved).

**blocks: []** — verified: no Phase 1 story consumes the proto types directly.
`monocle-runtime` and `monocle-tui` in Phase 1 use JSON hook POSTs, not protobuf wire encoding.
The proto crate compiles into the binary as a type library only; Phase 4 activates the wire path.

**Forbidden Dependencies:**
- `monocle-proto` MUST NOT depend on `monocle-runtime` or `monocle-tui`
- `monocle-proto` MUST NOT depend on `monocle-core` (no monocle-core symbols consumed here)
- Phase 1 runtime MUST NOT call any prost encoding/decoding functions

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| prost | 0.14 (EXACT) | `[dependencies]`; generates prost Message derive; no Phase 1 runtime wire usage |
| prost-build | =0.14.1 | `[build-dependencies]`; codegen in `build.rs`; aligns with prost 0.14 minor |
| bytes | 1.11 (caret) | `[dependencies]`; neutralizes RUSTSEC-2026-0007 in prost transitive resolution; declared by S-001 |

## File Structure Requirements

Files to create:
- `monocle-proto/proto/monocle/v1/hook_envelope.proto` — canonical proto schema (versioned dir per
  SS-core-types-and-abi.md §Prost Wire Schemas line 906; F-D-04 fix — was unversioned path)
- `monocle-proto/build.rs` — prost-build codegen (F-D-01/F-D-03 fix — required by SS-core-types-and-abi)
- `monocle-proto/src/lib.rs` — generated module include + rustdoc on HookEnvelope (E-PROTO-001 warning)
- `monocle-proto/tests/wire_field_order.rs` — VP-016 / BC-2.02.006 descriptor-decode field number oracle
  (F-D-05 + F-C-01 fix — canonical name per SS-core-types-and-abi.md line 982; was proto_field_numbers.rs)
- `monocle-proto/tests/schema_version.rs` — VP-017/VP-018 / BC-2.02.007 struct tests + AC-004 rustdoc oracle

Files to modify:
- `monocle-proto/Cargo.toml` — add `prost = "=0.14.x"` to `[dependencies]`; add `prost-build = "=0.14.1"`
  to `[build-dependencies]`; add `bytes = "1.11"` to `[dependencies]` if not already declared from workspace

## §Trace

**v1.2** (2026-05-20) — Sibling-sweep update for SS-deps-pin-manifest v1.1.19 Option B (bytes pin "1.10" → "1.11" per RUSTSEC-2026-0007 fix-from = 1.11.1; production-grade default). 4 body sites updated: Tasks NOTE, Previous Story Intelligence, Library & Framework Requirements table, Files to modify.

**v1.1 — Phase 3.B Batch 3: arch-touching story remediation** (2026-05-20):
- F-E-01: `depends_on` downgraded `[S-010]` → `[S-001]` — `monocle-proto` only inherits crate stub from
  S-001; it does NOT consume any `monocle-core` symbols; no monocle-core types imported.
- F-D-01 (CRITICAL): Hand-written struct decision overridden — rewritten to use prost-build codegen per
  SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas (FC-05 resolution) lines 884-892.
- F-D-02 (CRITICAL): `.proto` schema rewritten from 3-field sketch to canonical envelope (4 fields) +
  oneof event (5 inner messages) verbatim from SS-core-types-and-abi.md lines 915-960.
- F-D-03 (HIGH): `monocle-proto/build.rs` task added with explicit `prost_build::Config::new()` invocation.
- F-D-04 (MEDIUM): Proto file path updated to versioned directory `monocle-proto/proto/monocle/v1/`.
- F-D-05 (MEDIUM): Test files renamed/split: `wire_field_order.rs` (BC-2.02.006 / VP-016) +
  `schema_version.rs` (BC-2.02.007/BC-2.02.008 / VP-017/VP-018); was single `schema_version.rs`.
- F-A-01 (HIGH): `prost-build =0.14.1` added to Library table as `[build-dependencies]`.
- F-A-02 (LOW): `bytes 1.10` precondition noted — RUSTSEC-2026-0007 neutralization.
- F-B-01 (HIGH): Architecture Compliance Rules updated to use canonical anchor `§Prost Wire Schemas (FC-05 resolution)`.
- F-B-02 (HIGH): FC-05 traceability added in Architecture Compliance Rules citing SS-forward-compatibility.md §FC-05.
- F-B-03 (MEDIUM): Field number reservation scheme added (lines 894-901).
- F-C-01 (CRITICAL): AC-006 oracle rewritten from text-regex parse to prost-reflect descriptor decode.
- F-C-02 (HIGH): AC-004 executable oracle added (grep test for E-PROTO-001 warning substring).
- F-C-03 (MEDIUM): AC-005 negative oracle added (grep asserts only literal 1 as schema_version value).
- F-E-02 (MEDIUM): Downstream impact statement added in Architecture Compliance Rules.
- F-E-03 (LOW): `blocks: []` explicitly verified — no Phase 1 story consumes proto types.
## §Trace 1.3 — POL-11 cascade remediation (2026-05-30)

**Bump:** 1.2 → 1.3.
**Scope:** AC/Architecture body (3 occurrences): `SS-forward-compatibility.md v1.2.19 §FC-05` → `SS-forward-compatibility.md §FC-05` (Option 2 version-free; cascade from SS-forward-compatibility v1.2.19 → v1.2.20 bump in same remediation burst; version-free permanently prevents re-staling).
**SE-16d PASS:** 2026-05-30 >= prior date (cascade patch).
