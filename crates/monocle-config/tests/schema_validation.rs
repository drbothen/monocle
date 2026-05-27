//! Schema validation tests for BC-2.07.002: Config Schema Version 1.
//!
//! Covers:
//! - `schema_version` field: correct default value, serialization order (first key).
//! - `binding_overrides` default: must be `{}` (empty JSON object), never `null`.
//! - `HarnessProfile` round-trip with all fields including `config_dir: None`.
//! - Unknown top-level fields are silently ignored on deserialization (forward-compat).
//! - Missing `schema_version` produces a parse error (EC-084).
//! - Round-trip: serialize then deserialize produces identical struct.
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause |
//! |---------------|----|-----------|
//! | test_BC_2_07_002_default_schema_version_is_1 | story AC-001 | BC-2.07.002 INV-1 |
//! | test_BC_2_07_002_schema_version_is_first_key | story AC-001 | BC-2.07.002 PC-1 |
//! | test_BC_2_07_002_binding_overrides_default_is_empty_object | story AC-012 | BC-2.07.002 EC-080, INV-3 |
//! | test_BC_2_07_002_binding_overrides_never_null_after_default | story AC-012 | BC-2.07.002 INV-3 |
//! | test_BC_2_07_002_harness_profile_round_trip | story AC-002 | BC-2.07.002 PC-2 |
//! | test_BC_2_07_002_config_dir_none_round_trips | story AC-002 | BC-2.07.002 EC-082 |
//! | test_BC_2_07_002_unknown_top_level_fields_ignored | — | BC-2.07.002 EC-083, PC-8 |
//! | test_BC_2_07_002_missing_schema_version_is_parse_error | story AC-004 | BC-2.07.002 EC-084 |
//! | test_BC_2_07_002_harness_profiles_absent_defaults_empty | — | BC-2.07.002 EC-078 |
//! | test_BC_2_07_002_ccr_path_absent_defaults_none | — | BC-2.07.002 EC-079 |
//! | test_BC_2_07_002_project_profiles_absent_defaults_empty | — | BC-2.07.002 EC-081 |
//! | test_BC_2_07_002_full_round_trip | — | BC-2.07.002 §Canonical Test Vectors |
//! | test_BC_2_07_002_invariant_binding_overrides_is_object | story AC-012 | BC-2.07.002 INV-3 |

// Tests use expect/unwrap as assertion amplification — not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// BC-ID-encoded test names use dots-as-underscores.
#![allow(non_snake_case)]

use monocle_config::{HarnessProfile, MonocleConfig};

// ---------------------------------------------------------------------------
// BC-2.07.002 INV-1 — schema_version is always 1 in Phase 1
// ---------------------------------------------------------------------------

/// BC-2.07.002 INV-1: `MonocleConfig::default()` must have `schema_version == 1`.
///
/// This is the foundation invariant. If it fails, every other schema test is
/// meaningless.
#[test]
fn test_BC_2_07_002_default_schema_version_is_1() {
    let config = MonocleConfig::default();
    assert_eq!(
        config.schema_version,
        MonocleConfig::CURRENT_SCHEMA_VERSION,
        "MonocleConfig::default().schema_version must equal CURRENT_SCHEMA_VERSION (1) \
        per BC-2.07.002 INV-1; got: {}",
        config.schema_version
    );
    assert_eq!(
        config.schema_version, 1u32,
        "CURRENT_SCHEMA_VERSION must be 1 in Phase 1 code per BC-2.07.002 INV-1; \
        got: {}",
        config.schema_version
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.002 PC-1 — schema_version appears as the first JSON key
// ---------------------------------------------------------------------------

/// BC-2.07.002 PC-1: `serde_json::to_string_pretty(&default_config)` must have
/// `"schema_version"` as the first key in the serialized JSON object.
///
/// This is verified via raw byte-position scanning (not parsed JSON iteration),
/// because `serde_json::Map` without the `preserve_order` feature does not
/// guarantee iteration order. The struct field declaration order controls
/// serialization for named structs via serde's default behavior.
#[test]
fn test_BC_2_07_002_schema_version_is_first_key() {
    let config = MonocleConfig::default();
    let json = serde_json::to_string_pretty(&config)
        .expect("MonocleConfig::default() must serialize without error (BC-2.07.002 PC-1)");

    let sv_pos = json
        .find("\"schema_version\"")
        .expect("\"schema_version\" must appear in serialized JSON (BC-2.07.002 PC-1)");

    // All other top-level keys must appear after schema_version in the raw JSON.
    let later_keys = [
        "\"harness_profiles\"",
        "\"ccr_path\"",
        "\"binding_overrides\"",
        "\"project_profiles\"",
    ];
    for key in later_keys {
        if let Some(pos) = json.find(key) {
            assert!(
                sv_pos < pos,
                "\"schema_version\" (byte {sv_pos}) must appear BEFORE {key} (byte {pos}) \
                in serialized JSON per BC-2.07.002 PC-1. Struct field order controls \
                serde serialization; schema_version must be declared first."
            );
        }
    }
}

// ---------------------------------------------------------------------------
// BC-2.07.002 EC-080 / INV-3 — binding_overrides default is empty object
// ---------------------------------------------------------------------------

/// BC-2.07.002 EC-080: When `binding_overrides` is absent from JSON, it must
/// deserialize to an empty JSON object `{}`, NOT `null`.
///
/// Plain `#[serde(default)]` on `serde_json::Value` produces `Value::Null`, which
/// violates INV-3. The correct implementation uses `default_binding_overrides()`.
#[test]
fn test_BC_2_07_002_binding_overrides_default_is_empty_object() {
    // Minimal JSON: only schema_version, binding_overrides absent.
    let json = r#"{"schema_version": 1}"#;
    let config: MonocleConfig = serde_json::from_str(json)
        .expect("minimal JSON with just schema_version must deserialize (BC-2.07.002 EC-080)");

    assert!(
        config.binding_overrides.is_object(),
        "binding_overrides must deserialize to Value::Object when absent from JSON, \
        NOT Value::Null. Got: {:?}. \
        This indicates #[serde(default)] is used instead of \
        #[serde(default = \"default_binding_overrides\")] (BC-2.07.002 EC-080 / INV-3).",
        config.binding_overrides
    );

    let obj = config
        .binding_overrides
        .as_object()
        .expect("must be object");
    assert!(
        obj.is_empty(),
        "default binding_overrides must be an empty JSON object `{{}}` \
        per BC-2.07.002 EC-080; got non-empty: {obj:?}"
    );
}

/// BC-2.07.002 INV-3: `MonocleConfig::default()` must have `binding_overrides`
/// as an empty JSON object, never `null`.
///
/// This test is redundant with EC-080 but tests the Rust Default impl directly
/// (not deserialization). The field must be initialized to `Value::Object(Map::new())`
/// in the `Default` implementation.
#[test]
fn test_BC_2_07_002_binding_overrides_never_null_after_default() {
    let config = MonocleConfig::default();

    assert!(
        !config.binding_overrides.is_null(),
        "binding_overrides must NEVER be Value::Null in a default-constructed \
        MonocleConfig per BC-2.07.002 INV-3. Got Value::Null. \
        The Default impl must initialize binding_overrides to \
        Value::Object(Map::new()), not Value::Null."
    );

    assert!(
        config.binding_overrides.is_object(),
        "binding_overrides must be Value::Object in MonocleConfig::default() \
        per BC-2.07.002 INV-3; got: {:?}",
        config.binding_overrides
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.002 PC-2 — HarnessProfile round-trip
// ---------------------------------------------------------------------------

/// BC-2.07.002 PC-2: A config with one `HarnessProfile` (all fields populated)
/// must round-trip through serialize/deserialize with all fields intact.
#[test]
fn test_BC_2_07_002_harness_profile_round_trip() {
    let profile = HarnessProfile {
        id: "cc".to_string(),
        display_name: "Claude Code".to_string(),
        binary_path: "/usr/local/bin/claude".to_string(),
        config_dir: Some("/home/user/.claude".to_string()),
    };
    let mut config = MonocleConfig::default();
    config.harness_profiles.push(profile.clone());

    let json = serde_json::to_string_pretty(&config)
        .expect("MonocleConfig with one profile must serialize (BC-2.07.002 PC-2)");

    let deserialized: MonocleConfig = serde_json::from_str(&json)
        .expect("serialized config with one profile must deserialize (BC-2.07.002 PC-2)");

    assert_eq!(
        deserialized.harness_profiles.len(),
        1,
        "round-tripped config must have exactly 1 harness profile (BC-2.07.002 PC-2)"
    );

    let rt_profile = &deserialized.harness_profiles[0];
    assert_eq!(
        rt_profile.id, profile.id,
        "HarnessProfile.id must survive round-trip (BC-2.07.002 PC-2)"
    );
    assert_eq!(
        rt_profile.display_name, profile.display_name,
        "HarnessProfile.display_name must survive round-trip (BC-2.07.002 PC-2)"
    );
    assert_eq!(
        rt_profile.binary_path, profile.binary_path,
        "HarnessProfile.binary_path must survive round-trip (BC-2.07.002 PC-2)"
    );
    assert_eq!(
        rt_profile.config_dir, profile.config_dir,
        "HarnessProfile.config_dir must survive round-trip (BC-2.07.002 PC-2)"
    );
}

/// BC-2.07.002 EC-082: When `config_dir` is absent from a `HarnessProfile` JSON
/// element, it must deserialize as `None` (via `#[serde(default)]`).
#[test]
fn test_BC_2_07_002_config_dir_none_round_trips() {
    let json = r#"{
        "schema_version": 1,
        "harness_profiles": [
            {"id": "cc", "display_name": "Claude Code", "binary_path": "/usr/local/bin/claude"}
        ]
    }"#;

    let config: MonocleConfig = serde_json::from_str(json).expect(
        "config with HarnessProfile missing config_dir must deserialize (BC-2.07.002 EC-082)",
    );

    assert_eq!(config.harness_profiles.len(), 1);
    assert_eq!(
        config.harness_profiles[0].config_dir, None,
        "config_dir absent from JSON must deserialize to None via #[serde(default)] \
        (BC-2.07.002 EC-082); got: {:?}",
        config.harness_profiles[0].config_dir
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.002 PC-8 / EC-083 — Unknown top-level fields ignored
// ---------------------------------------------------------------------------

/// BC-2.07.002 EC-083: A JSON file containing unknown top-level keys (written by
/// a future Phase 2+ binary) must deserialize without error. Unknown keys are
/// silently discarded. This requires the absence of `#[serde(deny_unknown_fields)]`.
#[test]
fn test_BC_2_07_002_unknown_top_level_fields_ignored() {
    let json = r#"{
        "schema_version": 1,
        "future_phase2_feature": true,
        "some_new_array": [1, 2, 3],
        "harness_profiles": []
    }"#;

    let result = serde_json::from_str::<MonocleConfig>(json);
    assert!(
        result.is_ok(),
        "MonocleConfig deserialization must not fail on unknown top-level fields \
        (BC-2.07.002 EC-083 / PC-8). This indicates #[serde(deny_unknown_fields)] \
        is incorrectly applied. Error: {:?}",
        result.err()
    );

    let config = result.expect("must succeed");
    assert_eq!(
        config.schema_version, 1,
        "schema_version must be correctly parsed when unknown fields are present \
        (BC-2.07.002 EC-083)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.002 EC-084 — Missing schema_version produces parse error
// ---------------------------------------------------------------------------

/// BC-2.07.002 EC-084: A JSON file with `schema_version` absent must fail to
/// deserialize (since `schema_version: u32` has no `#[serde(default)]`).
///
/// This parse error triggers BC-2.07.003 Case 3 (parse failure → default + warn).
/// Here we test that the parse fails at the schema level, not that load_config
/// handles it (that's in config_io.rs tests).
#[test]
fn test_BC_2_07_002_missing_schema_version_is_parse_error() {
    // JSON with no schema_version field at all (per EC-084 and BC-2.07.003 EC-091).
    let json = r#"{"harness_profiles": [], "binding_overrides": {}}"#;

    let result = serde_json::from_str::<MonocleConfig>(json);
    assert!(
        result.is_err(),
        "Deserializing MonocleConfig from JSON without schema_version must fail \
        with a serde parse error (BC-2.07.002 EC-084). Got Ok — this indicates \
        #[serde(default)] was incorrectly added to schema_version."
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.002 EC-078, EC-079, EC-081 — Missing optional fields default correctly
// ---------------------------------------------------------------------------

/// BC-2.07.002 EC-078: `harness_profiles` absent from JSON deserializes to empty Vec.
#[test]
fn test_BC_2_07_002_harness_profiles_absent_defaults_empty() {
    let json = r#"{"schema_version": 1}"#;
    let config: MonocleConfig = serde_json::from_str(json)
        .expect("JSON with only schema_version must deserialize (BC-2.07.002 EC-078)");

    assert!(
        config.harness_profiles.is_empty(),
        "harness_profiles absent from JSON must default to empty Vec \
        per #[serde(default)] (BC-2.07.002 EC-078); got {} profiles",
        config.harness_profiles.len()
    );
}

/// BC-2.07.002 EC-079: `ccr_path` absent from JSON deserializes to `None`.
#[test]
fn test_BC_2_07_002_ccr_path_absent_defaults_none() {
    let json = r#"{"schema_version": 1}"#;
    let config: MonocleConfig = serde_json::from_str(json)
        .expect("JSON with only schema_version must deserialize (BC-2.07.002 EC-079)");

    assert_eq!(
        config.ccr_path, None,
        "ccr_path absent from JSON must default to None via #[serde(default)] \
        (BC-2.07.002 EC-079); got: {:?}",
        config.ccr_path
    );
}

/// BC-2.07.002 EC-081: `project_profiles` absent from JSON deserializes to empty HashMap.
#[test]
fn test_BC_2_07_002_project_profiles_absent_defaults_empty() {
    let json = r#"{"schema_version": 1}"#;
    let config: MonocleConfig = serde_json::from_str(json)
        .expect("JSON with only schema_version must deserialize (BC-2.07.002 EC-081)");

    assert!(
        config.project_profiles.is_empty(),
        "project_profiles absent from JSON must default to empty HashMap \
        per #[serde(default)] (BC-2.07.002 EC-081); got {} entries",
        config.project_profiles.len()
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.002 §Canonical Test Vectors — Full round-trip
// ---------------------------------------------------------------------------

/// BC-2.07.002 canonical test vector: `MonocleConfig::default()` serialized then
/// deserialized must equal the original struct.
///
/// Also verifies the canonical serialized form per the BC test vector table.
#[test]
fn test_BC_2_07_002_full_round_trip() {
    let original = MonocleConfig::default();
    let json = serde_json::to_string_pretty(&original)
        .expect("MonocleConfig::default() must serialize (BC-2.07.002 canonical test vector)");

    // Verify canonical expected structure: schema_version present and correct.
    let value: serde_json::Value =
        serde_json::from_str(&json).expect("serialized JSON must be valid");
    assert_eq!(
        value.get("schema_version").and_then(|v| v.as_u64()),
        Some(1u64),
        "serialized MonocleConfig must have schema_version: 1 \
        (BC-2.07.002 canonical test vector)"
    );
    assert!(
        value
            .get("harness_profiles")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false),
        "serialized default MonocleConfig must have harness_profiles: [] \
        (BC-2.07.002 canonical test vector)"
    );

    // Round-trip: deserialized struct must equal original.
    let deserialized: MonocleConfig = serde_json::from_str(&json)
        .expect("serialized MonocleConfig must deserialize (BC-2.07.002 full round-trip)");

    assert_eq!(
        deserialized, original,
        "MonocleConfig::default() must equal itself after serialize/deserialize round-trip \
        (BC-2.07.002 canonical test vector)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.002 INV-3 — binding_overrides invariant: opaque Value::Object
// ---------------------------------------------------------------------------

/// BC-2.07.002 INV-3: `binding_overrides` with arbitrary content round-trips as
/// `Value::Object` — phase 1 code treats it as opaque.
#[test]
fn test_BC_2_07_002_invariant_binding_overrides_is_object() {
    // JSON with a non-trivial binding_overrides object.
    let json = r#"{
        "schema_version": 1,
        "binding_overrides": {"ctrl_p": "profile-picker", "ctrl_q": "quit"}
    }"#;

    let config: MonocleConfig = serde_json::from_str(json)
        .expect("config with populated binding_overrides must deserialize (BC-2.07.002 INV-3)");

    assert!(
        config.binding_overrides.is_object(),
        "binding_overrides must always be Value::Object (BC-2.07.002 INV-3); \
        got: {:?}",
        config.binding_overrides
    );

    let obj = config
        .binding_overrides
        .as_object()
        .expect("must be object");
    assert_eq!(
        obj.get("ctrl_p").and_then(|v| v.as_str()),
        Some("profile-picker"),
        "binding_overrides arbitrary keys must survive round-trip (BC-2.07.002 INV-3)"
    );

    // Re-serialize and verify it's still an object (not coerced to null).
    let re_json = serde_json::to_string(&config).expect("re-serialize must succeed");
    let re_value: serde_json::Value =
        serde_json::from_str(&re_json).expect("re-serialized JSON must be valid");
    assert!(
        re_value
            .get("binding_overrides")
            .map(|v| v.is_object())
            .unwrap_or(false),
        "re-serialized config must still have binding_overrides as object \
        (BC-2.07.002 INV-3 round-trip); re_json: {re_json}"
    );
}
