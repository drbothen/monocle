---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T00:00:00Z
phase: phase-1-expansion
inputs:
  - {path: .factory/specs/architecture/SS-config.md, version: "1.0.0"}
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-07
capability: CAP-007
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

# Behavioral Contract BC-2.07.002: Config Schema Version 1: Harness Profile Fields

## Description

The `config.json` file written and read by `monocle-config` uses a versioned JSON schema.
Schema version 1 carries a mandatory `schema_version` field (integer value `1`) as the
first key, followed by `harness_profiles`, `ccr_path`, `binding_overrides`, and
`project_profiles`. The schema is forward-compatible: unknown fields present in a file
written by a newer version of monocle are silently ignored when read by a Phase 1 binary.
The schema_version field enables future migrations in Phase 2+ without breaking Phase 1
readers.

## Preconditions

1. A `MonocleConfig` struct has been constructed (either by `load_config()` or
   `MonocleConfig::default()`).
2. For deserialization: the file at the config path contains a valid UTF-8 JSON object.
3. The Rust struct `MonocleConfig` derives both `serde::Serialize` and `serde::Deserialize`.
   `serde(default)` is applied to `harness_profiles`, `ccr_path`, `binding_overrides`, and
   `project_profiles` fields. `#[serde(deny_unknown_fields)]` is NOT used on `MonocleConfig`
   or `HarnessProfile`.

## Postconditions

**Serialization (write path):**
1. `serde_json::to_string_pretty(&config)` produces a JSON object where `schema_version`
   appears as the first key with integer value `1`.
2. `harness_profiles` serializes as a JSON array. Each element is a JSON object with keys
   `id` (string), `display_name` (string), `binary_path` (string), and `config_dir`
   (string or `null`). The `#[serde(default)]` attribute ensures the field is present even
   when the vector is empty.
3. `ccr_path` serializes as a JSON string when `Some(<path>)`, or `null` when `None`.
4. `binding_overrides` serializes as a JSON object. The default value is an empty object `{}`.
   In Phase 1, the content is treated as opaque (round-tripped as `serde_json::Value`).
5. `project_profiles` serializes as a JSON object mapping absolute directory path strings
   to profile ID strings. The default value is an empty object `{}`.

**Deserialization (read path):**
6. A JSON file with `schema_version: 1` deserializes into `MonocleConfig` with all fields
   populated from JSON. Fields absent from the JSON but having `#[serde(default)]` receive
   their Rust default values (`Vec::new()`, `None`, empty `serde_json::Value::Object`,
   `HashMap::new()`).
7. A JSON file with `schema_version` missing is treated as schema version 1 (pre-versioning
   compatibility). The `schema_version` field has type `u32` without `#[serde(default)]`;
   a missing field causes a `serde_json` parse error, which is handled per BC-2.07.003
   (default config returned, warning logged). Rationale: development builds may have written
   an unversioned file; this is a known edge case from the evolution of the schema.
8. A JSON file containing unknown top-level keys (e.g., written by a Phase 2 binary that
   added new fields) deserializes without error. The unknown keys are silently discarded.
   This is guaranteed by the absence of `#[serde(deny_unknown_fields)]`.
9. A `HarnessProfile` element with `config_dir` absent deserializes with `config_dir: None`
   (via `#[serde(default)]`).

## Invariants

1. `schema_version` is always `1` in Phase 1 code. The field is never set to any other value
   by Phase 1 `monocle-config` code. Writing a config with any other `schema_version` value
   is a protocol violation that would confuse future migration logic.
2. Each `HarnessProfile` in `harness_profiles` has a unique `id` field within the list.
   Deduplication is the responsibility of the caller (profile picker); `monocle-config`'s
   schema contract does not enforce uniqueness at the `serde` level, but callers MUST NOT
   write duplicate IDs.
3. `binding_overrides` is stored and round-tripped as an opaque `serde_json::Value::Object`.
   Phase 1 code reads but does not parse `binding_overrides` beyond confirming it is a JSON
   object. Writing non-object values to `binding_overrides` is a protocol violation.
4. `project_profiles` values are profile IDs that MUST appear as an `id` in `harness_profiles`.
   This referential integrity constraint is not enforced by the schema at the `serde` level
   but is enforced by the profile picker logic (BC-2.07.004).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-078 | `harness_profiles` absent from JSON | Deserializes to `harness_profiles: []` (via `#[serde(default)]`) |
| EC-079 | `ccr_path` absent from JSON | Deserializes to `ccr_path: None` (via `#[serde(default)]`) |
| EC-080 | `binding_overrides` absent from JSON | Deserializes to `binding_overrides: {}` (via `#[serde(default)]`) |
| EC-081 | `project_profiles` absent from JSON | Deserializes to `project_profiles: HashMap::new()` (via `#[serde(default)]`) |
| EC-082 | `config_dir` absent from a `HarnessProfile` element | Deserializes to `config_dir: None` (via `#[serde(default)]`) |
| EC-083 | JSON contains unknown top-level field (e.g., `"new_phase2_field": 42`) | Field is silently ignored; `MonocleConfig` deserializes with known fields populated normally |
| EC-084 | `schema_version` field is missing from JSON (unversioned dev build) | Deserialization fails (serde_json error); per BC-2.07.003 caller gets `MonocleConfig::default()` and a `tracing::warn!` is emitted |
| EC-085 | `harness_profiles` contains a profile with `binary_path` set to a path that does not exist on disk | Schema accepts it; monocle-config does not validate binary existence at schema level — validation is at engine spawn time (BC-2.03.004) |
| EC-086 | `project_profiles` references a profile ID not in `harness_profiles` | Schema accepts it; dangling reference detected by profile picker at runtime (BC-2.07.004 Postcondition 3 handles this case) |
| EC-087 | `harness_profiles` has two entries with the same `id` | Schema accepts it; deduplication is the caller's responsibility; profile picker selects the first match by iteration order |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `MonocleConfig::default()` serialized via `serde_json::to_string_pretty` | `{"schema_version":1,"harness_profiles":[],"ccr_path":null,"binding_overrides":{},"project_profiles":{}}` (pretty-printed) | happy-path |
| JSON with one `HarnessProfile`: `{id:"cc", display_name:"Claude Code", binary_path:"/usr/local/bin/claude", config_dir:null}` | Deserializes to `MonocleConfig` with `harness_profiles.len() == 1`; profile fields match | happy-path |
| JSON with `ccr_path: "/usr/local/bin/ccr"` | Deserializes with `ccr_path: Some("/usr/local/bin/ccr".to_string())` | happy-path |
| JSON with `binding_overrides: {"ctrl_p": "profile-picker"}` | Deserializes with `binding_overrides` as `serde_json::Value::Object`; unknown key preserved in the Value | happy-path |
| JSON with extra top-level field `"future_field": true` | Deserializes without error; `future_field` silently ignored | forward-compat |
| JSON with `schema_version` field absent | `serde_json` parse error triggers BC-2.07.003 default path; no panic | edge-case |
| Round-trip: serialize `MonocleConfig` → deserialize → compare | Deserialized struct equals original | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `MonocleConfig::default()` serializes with `schema_version: 1` as first key | unit (serde_json::to_string_pretty; parse back and assert first key) |
| VP-TBD | Unknown top-level fields are silently ignored on deserialization | unit (inject known-unknown field into JSON string; assert no parse error) |
| VP-TBD | All `#[serde(default)]` fields produce correct Rust defaults when absent from JSON | unit (deserialize minimal JSON `{"schema_version":1}`; assert all defaults) |
| VP-TBD | `HarnessProfile` round-trips all four fields including `config_dir: None` | unit (serialize → deserialize; assert equality) |
| VP-TBD | `binding_overrides` is treated as opaque `serde_json::Value::Object`; arbitrary keys round-trip | unit (serialize config with rich `binding_overrides`; deserialize; assert Value equality) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability §SS-07 |
| Capability Anchor Justification | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability — this BC defines the data schema that is the carrier for harness profile management and config persistence |
| L2 Domain Invariants | No domain-spec/invariants.md exists for this project; authority is ARCH-INDEX §SS-07 and SS-config.md §Config Schema v1 |
| Architecture Module | monocle-config (config.json reader/writer, harness profile schema, profile picker logic) per ARCH-INDEX Subsystem Registry SS-07 |
| Architecture Source | SS-config.md v1.0.0 §Config Schema v1 |
| Cross-Ref | BC-2.07.001 (write_config serializes this schema atomically); BC-2.07.003 (parse failure path); BC-2.07.004 (project_profiles field consumed by profile picker); BC-2.07.006 (ccr_path field consumed by CCR detection) |
| Brief Features | F-53 (config.json schema), F-54 (harness profile schema), F-55 (ccr_path field), F-56 (binding_overrides stub) |
| Test File | `monocle-config/tests/schema_v1.rs` |
| Test Name | `test_BC_2_07_002_config_schema_v1_round_trip` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.07.001] — depends on: write_config serializes the schema defined in this BC
- [BC-2.07.003] — depends on: parse errors from this schema fall through to default handling
- [BC-2.07.004] — composes with: project_profiles field from this schema is consumed by the profile picker
- [BC-2.07.006] — composes with: ccr_path field from this schema is consumed by CCR detection

## Architecture Anchors

- `architecture/SS-config.md#config-schema-v1` — field definitions, Rust struct representation, Default impl
- `architecture/SS-config.md#forward-compatibility` — unknown field handling, schema_version migration strategy

## Story Anchor

S-TBD — Implement monocle-config crate: atomic write, schema v1, default handling (filled by story-writer)

## VP Anchors

VP-TBD — config schema v1 unit tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- Created as new artifact for SS-07 (Config subsystem) per `prd-expansion-scope.md` §3.4 BC-2.07.002
  and `SS-config.md` §Config Schema v1 and §Forward Compatibility.
- Brief features traced: F-53, F-54, F-55, F-56.
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact; no prior chain).
