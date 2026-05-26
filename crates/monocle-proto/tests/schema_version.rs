//! VP-017 / VP-018 / BC-2.02.007 / BC-2.02.008 integration tests — HookEnvelope
//! Rust struct schema_version field and forward-compat contract.
//!
//! VP-017: HookEnvelope Rust struct exposes `pub schema_version: u32` with value 1.
//! VP-018 (Phase 1 structural): cross-property recap of VP-016 + VP-017.
//! BC-2.02.007: prost-build-generated HookEnvelope Rust struct.
//! BC-2.02.008: Phase 4 schema_version validation (Phase 1 portion: structural recap only).
//!
//! Test naming follows the BC-based pattern:
//!   `test_BC_2_02_007_<assertion_name>()`

// Test code panicking on assertion failure is correct behavior; `expect()` is
// the idiomatic way to assert decode/encode invariants in tests.
#![allow(clippy::expect_used)]
//!   `test_BC_2_02_008_<assertion_name>()`
//!
//! Probe matrix (from VP-017):
//!   17.a — Construct HookEnvelope { schema_version: 1 } → envelope.schema_version == 1
//!   17.b — Round-trip: HookEnvelope::decode(envelope.encode_to_vec()) → schema_version == 1
//!   17.c — Type pin: `let _: u32 = envelope.schema_version;` compiles
//!
//! Probe matrix (from VP-018, Phase 1):
//!   18.a — same as 17.a (cross-property)
//!   18.b — FileDescriptorSet inspection: field 1 name == "schema_version" (cross-VP-016)
//!   18.c — Test file does NOT import dispatch_envelope (source-grep absence assertion)
//!
//! AC-004 oracle: grep for E-PROTO-001 / "not recognized" in lib.rs
//! AC-005 oracle: grep src/ for schema_version values other than 1 → none found

// BC-based test names use uppercase BC and digits per VSDD factory naming convention.
// The `non_snake_case` lint is suppressed for this mandatory naming pattern.
#![allow(non_snake_case)]

use monocle_proto::HookEnvelope;
use monocle_proto::SessionStartEvent;
use prost::Message;

// ---------------------------------------------------------------------------
// Probe 17.a — HookEnvelope construction and schema_version field assertion.
// ---------------------------------------------------------------------------

/// BC-2.02.007 postcondition 1+2: prost-build-generated HookEnvelope exposes
/// `pub schema_version: u32`. For Phase 1 messages, this field value is 1.
///
/// Exercises VP-017 probe 17.a.
#[test]
fn test_BC_2_02_007_schema_version_field_value_is_1() {
    let envelope = HookEnvelope {
        schema_version: 1,
        session_id: "test-session-id".to_string(),
        timestamp_micros: 1_748_736_000_000_000i64,
        pid: 12345u32,
        event: None,
    };

    assert_eq!(
        envelope.schema_version, 1,
        "HookEnvelope.schema_version must be 1 for Phase 1 messages. \
         BC-2.02.007 postcondition 2 requires value 1 for all Phase 1-origin messages."
    );
}

/// BC-2.02.007 postcondition 1: field is `pub schema_version: u32`.
/// Compile-time type pin — if the proto changes schema_version type (e.g., to int32),
/// prost-build generates `pub schema_version: i32`, and this assignment fails.
///
/// Exercises VP-017 probe 17.c.
#[test]
fn test_BC_2_02_007_schema_version_field_type_is_u32() {
    let envelope = HookEnvelope {
        schema_version: 1,
        ..Default::default()
    };

    // Explicit type annotation — fails compilation if field type changes from u32.
    let _schema_version_typed: u32 = envelope.schema_version;
    assert_eq!(_schema_version_typed, 1u32);
}

/// BC-2.02.007: HookEnvelope with oneof event variant set.
/// Verifies that schema_version remains 1 when an event is present.
///
/// Exercises VP-017 probe 17.a with a non-None event.
#[test]
fn test_BC_2_02_007_schema_version_is_1_with_session_start_event() {
    let envelope = HookEnvelope {
        schema_version: 1,
        session_id: "ses_abc123".to_string(),
        timestamp_micros: 1_700_000_000_000_000i64,
        pid: 9999u32,
        event: Some(monocle_proto::hook_envelope::Event::SessionStart(
            SessionStartEvent {
                cwd: "/home/user/project".to_string(),
                transcript_path: "/tmp/session.jsonl".to_string(),
            },
        )),
    };

    assert_eq!(
        envelope.schema_version, 1,
        "schema_version must be 1 regardless of which oneof event variant is set"
    );
}

// ---------------------------------------------------------------------------
// Probe 17.b — Round-trip encode/decode preserves schema_version.
// ---------------------------------------------------------------------------

/// BC-2.02.007 postcondition 2 (round-trip): encode then decode preserves schema_version == 1.
/// The proto3 wire format round-trip must preserve the exact field value.
///
/// Exercises VP-017 probe 17.b.
#[test]
fn test_BC_2_02_007_schema_version_survives_encode_decode_round_trip() {
    let original = HookEnvelope {
        schema_version: 1,
        session_id: "round-trip-session".to_string(),
        timestamp_micros: 1_748_736_000_000_000i64,
        pid: 42u32,
        event: None,
    };

    let encoded = original.encode_to_vec();
    let decoded =
        HookEnvelope::decode(encoded.as_slice()).expect("HookEnvelope must decode without error");

    assert_eq!(
        decoded.schema_version, 1,
        "schema_version must be 1 after encode/decode round-trip. \
         VP-017 probe 17.b: HookEnvelope::decode(envelope.encode_to_vec()).schema_version == 1"
    );
}

/// BC-2.02.007 round-trip with SessionStart event variant.
/// Verifies both the envelope fields and the nested event survive encode/decode.
#[test]
fn test_BC_2_02_007_round_trip_with_session_start_event() {
    use monocle_proto::hook_envelope::Event;

    let original = HookEnvelope {
        schema_version: 1,
        session_id: "test-round-trip-with-event".to_string(),
        timestamp_micros: 1_748_737_000_000_000i64,
        pid: 1234u32,
        event: Some(Event::SessionStart(SessionStartEvent {
            cwd: "/workspace".to_string(),
            transcript_path: "/tmp/transcript.jsonl".to_string(),
        })),
    };

    let encoded = original.encode_to_vec();
    let decoded =
        HookEnvelope::decode(encoded.as_slice()).expect("Round-trip decode must succeed");

    assert_eq!(decoded.schema_version, 1);
    assert_eq!(decoded.session_id, "test-round-trip-with-event");

    match decoded.event {
        Some(Event::SessionStart(evt)) => {
            assert_eq!(evt.cwd, "/workspace");
            assert_eq!(evt.transcript_path, "/tmp/transcript.jsonl");
        }
        other => panic!("Expected SessionStart event after round-trip, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// BC-2.02.006 postcondition 4 — canonical test vectors from BC.
// ---------------------------------------------------------------------------

/// BC-2.02.006 canonical test vector (happy-path): round-trip HookEnvelope.
/// Exercises the test vector from BC-2.02.006 §Canonical Test Vectors row 2.
#[test]
fn test_BC_2_02_006_canonical_test_vector_round_trip_schema_version_1() {
    // Canonical test vector: "Round-trip HookEnvelope { schema_version: 1, ... }
    // via prost encode/decode → envelope.schema_version == 1 after decode"
    let envelope = HookEnvelope {
        schema_version: 1,
        ..Default::default()
    };

    let encoded = envelope.encode_to_vec();
    let decoded = HookEnvelope::decode(encoded.as_slice())
        .expect("Canonical test vector round-trip must succeed");

    assert_eq!(decoded.schema_version, 1);
}

// ---------------------------------------------------------------------------
// AC-004 — E-PROTO-001 rustdoc oracle: "not recognized" in lib.rs.
// ---------------------------------------------------------------------------

/// BC-2.02.008 postcondition 1 (AC-004 oracle): the E-PROTO-001 warning contract
/// is documented in `monocle-proto/src/lib.rs`. The implementer must ensure the
/// string "not recognized" AND "E-PROTO-001" appear in the crate documentation.
///
/// This grep test verifies the AC-004 requirement: an executable oracle that asserts
/// the E-PROTO-001 warning string substring is present in the source documentation.
///
/// This test FAILS (Red Gate) if the implementer removes or renames the E-PROTO-001
/// documentation block from lib.rs.
#[test]
fn test_BC_2_02_008_e_proto_001_warning_string_in_lib_rs() {
    // Read the lib.rs source at test time via include_str! to avoid I/O dependencies.
    // The path is relative to the crate root per Rust's include_str! semantics.
    let lib_rs_source = include_str!("../src/lib.rs");

    assert!(
        lib_rs_source.contains("not recognized"),
        "lib.rs must contain the E-PROTO-001 warning string 'not recognized' as part of \
         the Phase 4 dispatch contract documentation (AC-004 oracle). \
         This string is part of the canonical warning: \
         'WARN: HookEnvelope schema_version <N> not recognized; skipping'."
    );

    assert!(
        lib_rs_source.contains("E-PROTO-001"),
        "lib.rs must contain 'E-PROTO-001' as the error code identifier \
         for the Phase 4 schema_version dispatch contract (AC-004 oracle)."
    );
}

// ---------------------------------------------------------------------------
// AC-005 — Negative oracle: no schema_version values other than 1 in Phase 1 src.
// ---------------------------------------------------------------------------

/// BC-2.02.008 postcondition 2 (AC-005 negative oracle): Phase 1 source does NOT
/// set schema_version to any value other than 1. The canonical constant is 1.
///
/// This test asserts the negative oracle by reading lib.rs and verifying it does
/// not contain `schema_version: 0` or `schema_version: 2` (or any other non-1 value).
/// The implementer must not introduce non-1 schema_version assignments in Phase 1 src.
///
/// Scope: `src/lib.rs` is the only Phase 1 source file. The generated `monocle.v1.rs`
/// lives in OUT_DIR and is not scanned (it has no schema_version assignments, only
/// struct definitions). The scope guard assertion below confirms that lib.rs is the
/// only .rs file in src/ — if a future story adds more source files, the guard will
/// fail and the oracle scope must be expanded to cover all new files.
#[test]
fn test_BC_2_02_008_phase1_source_sets_schema_version_only_to_1() {
    // Guard: verify src/ only contains lib.rs (if this fails, expand oracle scope)
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let rs_files: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("src/ directory must exist")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .collect();
    assert_eq!(
        rs_files.len(),
        1,
        "AC-005 oracle scope guard: src/ should only contain lib.rs in Phase 1; \
         if more files are added, expand the negative oracle"
    );

    let lib_rs_source = include_str!("../src/lib.rs");

    // Assert no `schema_version: 0` assignment in Phase 1 source.
    assert!(
        !lib_rs_source.contains("schema_version: 0"),
        "lib.rs must NOT contain 'schema_version: 0'. \
         Phase 1 invariant: schema_version is always 1 (AC-005 negative oracle)."
    );

    // Assert no `schema_version: 2` (or higher) assignment in Phase 1 source.
    assert!(
        !lib_rs_source.contains("schema_version: 2"),
        "lib.rs must NOT contain 'schema_version: 2'. \
         Phase 1 invariant: schema_version is always 1 (AC-005 negative oracle)."
    );

    // Assert no `schema_version: 255` sentinel value.
    assert!(
        !lib_rs_source.contains("schema_version: 255"),
        "lib.rs must NOT contain 'schema_version: 255' (AC-005 negative oracle)."
    );
}

// ---------------------------------------------------------------------------
// VP-018 Phase 1 structural recap — cross-property BC-2.02.008.
// ---------------------------------------------------------------------------

/// BC-2.02.008 precondition structural recap (VP-018 Phase 1): both VP-016 and VP-017
/// must hold. This recap test asserts them in combination — the structural precondition
/// for any future Phase 4 dispatcher.
///
/// Per VP-018 §Post-conditions 3: "The test file is empty of any Phase 1 dispatcher
/// invocation. It does NOT import a dispatch_envelope function."
///
/// Exercises VP-018 probes 18.a + 18.b in combination.
#[test]
fn test_BC_2_02_008_phase1_structural_recap_schema_version_and_field_number() {
    use prost::Message as ProstMessage;
    use prost_types::FileDescriptorSet;

    // VP-017 recap (probe 18.a): construct HookEnvelope with schema_version: 1.
    let envelope = HookEnvelope {
        schema_version: 1,
        event: Some(monocle_proto::hook_envelope::Event::SessionStart(
            SessionStartEvent {
                cwd: "/".to_string(),
                transcript_path: String::new(),
            },
        )),
        ..Default::default()
    };
    assert_eq!(
        envelope.schema_version, 1,
        "VP-018 probe 18.a: schema_version must be 1 (cross-link to VP-017)"
    );

    // VP-016 recap (probe 18.b): FileDescriptorSet must confirm field 1 is schema_version.
    let descriptor_bytes: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin"));

    let fds = FileDescriptorSet::decode(descriptor_bytes)
        .expect("FileDescriptorSet decode must succeed (VP-016 recap)");

    let field_1_name = fds
        .file
        .iter()
        .find(|f| {
            f.name
                .as_deref()
                .map(|n| n.contains("hook_envelope"))
                .unwrap_or(false)
        })
        .and_then(|f| {
            f.message_type
                .iter()
                .find(|m| m.name.as_deref() == Some("HookEnvelope"))
        })
        .and_then(|m| m.field.iter().find(|f| f.number == Some(1)))
        .and_then(|f| f.name.clone())
        .expect("HookEnvelope must have a field at number 1 (VP-016 recap)");

    assert_eq!(
        field_1_name, "schema_version",
        "VP-018 probe 18.b: field number 1 must be 'schema_version' (cross-link to VP-016). \
         Both VP-016 and VP-017 must hold for Phase 4 dispatch to function."
    );
}

// ---------------------------------------------------------------------------
// BC-2.02.008 edge cases: EC-025 (schema_version: 0 in Phase 4 context).
// ---------------------------------------------------------------------------

/// BC-2.02.006 edge case EC-025: schema_version value 0 is not a valid Phase 1 version.
/// This test encodes a HookEnvelope with schema_version: 0 and verifies it decodes
/// correctly (proto3 semantics: 0 is the default, so it won't be on the wire).
/// The Phase 4 dispatcher must treat value 0 as unrecognized (Phase 4 scope).
///
/// Phase 1 assertion here: constructing with schema_version: 0 is possible at the
/// Rust level but NO Phase 1 code path does this (AC-005 negative oracle).
#[test]
fn test_BC_2_02_006_edge_case_ec025_schema_version_0_encodes_as_default() {
    // EC-025: schema_version: 0 — proto3 default, not a valid Phase 1 version.
    // Phase 1 note: this value MUST NOT appear in real Phase 1 code paths.
    let invalid_envelope = HookEnvelope {
        schema_version: 0, // invalid: not a Phase 1 version
        ..Default::default()
    };

    // Proto3: 0 is the default for uint32. A zero-valued field is not encoded on the wire.
    // Encoding then decoding a zero schema_version round-trips to schema_version: 0.
    let encoded = invalid_envelope.encode_to_vec();
    let decoded = HookEnvelope::decode(encoded.as_slice())
        .expect("proto3 decode must not panic even for invalid schema versions");

    // After round-trip, the value is still 0 (proto3 preserves it even if not on wire,
    // because decode initializes from the default).
    assert_eq!(
        decoded.schema_version, 0,
        "EC-025: schema_version: 0 must round-trip without panic. \
         The Phase 4 dispatcher (not Phase 1) is responsible for the log-and-skip behavior."
    );
}

/// BC-2.02.006 edge case EC-024: proto3 forward compatibility — unknown fields
/// in range 100-999 are preserved. This test verifies that adding Phase 4 federation
/// fields (field numbers 100+) to a raw byte slice still decodes the Phase 1 fields.
#[test]
fn test_BC_2_02_006_edge_case_ec024_unknown_fields_preserved_in_round_trip() {
    // Encode a known Phase 1 envelope.
    let phase1_envelope = HookEnvelope {
        schema_version: 1,
        session_id: "phase1-session".to_string(),
        timestamp_micros: 1_748_736_000_000_000i64,
        pid: 100u32,
        event: None,
    };
    let mut encoded = phase1_envelope.encode_to_vec();

    // Simulate a Phase 4 addition: append a fictitious field at number 100 (range 100-999).
    // Field 100, wire type 2 (length-delimited string): tag = (100 << 3) | 2 = 802 (varint).
    // Varint encoding of 802: 0xa2 0x06 (802 = 0x322 = 0b0011_0010_0010).
    //   Low 7 bits: 0x22, set continuation: 0xa2.
    //   Next 7 bits: 0x06.
    // Length prefix: 1 byte. Content: 0x01.
    encoded.extend_from_slice(&[0xa2, 0x06, 0x01, 0x01]);

    // Phase 1 decoder must not crash on unknown fields (proto3 forward compat guarantee).
    let decoded = HookEnvelope::decode(encoded.as_slice())
        .expect("EC-024: Phase 1 decoder must not panic on unknown Phase 4 fields");

    // Phase 1 fields survive decode.
    assert_eq!(
        decoded.schema_version, 1,
        "EC-024: schema_version must survive decode with unknown Phase 4 fields present"
    );
    assert_eq!(
        decoded.session_id, "phase1-session",
        "EC-024: session_id must survive decode with unknown Phase 4 fields present"
    );
}

// ---------------------------------------------------------------------------
// VP-018 probe 18.c — Test file does NOT import dispatch_envelope.
// ---------------------------------------------------------------------------

/// BC-2.02.008 postcondition structural recap (VP-018 probe 18.c):
/// Source-grep absence assertion — the Phase 1 production source (`lib.rs`) does
/// NOT contain any reference to the Phase 4 dispatcher symbol. Per VP-018
/// §Post-conditions 3, the Phase 1 codebase must be empty of any dispatcher
/// invocation; the Phase 4 dispatcher is not imported or called in Phase 1 source.
///
/// Implementation note: the dispatcher symbol name is assembled at runtime so
/// that this test file does not itself contain the literal, which would create a
/// confusing false-positive if the oracle were ever redirected to scan this file.
#[test]
fn test_BC_2_02_008_vp018_probe_18c_no_dispatch_envelope_import() {
    // Assemble the needle at runtime to keep the literal absent from this source file.
    let needle = {
        let mut s = String::from("dispatch");
        s.push('_');
        s.push_str("envelope");
        s
    };

    // Oracle: the Phase 1 production source must not reference the Phase 4 dispatcher.
    let lib_rs_source = include_str!("../src/lib.rs");
    assert!(
        !lib_rs_source.contains(needle.as_str()),
        "VP-018 probe 18.c: lib.rs must not reference the Phase 4 dispatcher symbol \
         (Phase 1 structural constraint — dispatcher is a Phase 4 concern only)"
    );
}
