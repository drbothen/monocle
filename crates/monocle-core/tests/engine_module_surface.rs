//! VP-019 AST audit: `EngineModule` trait surface, supporting types, and `HookEvent`.
//!
//! Tests in this file are the canonical verification harness for BC-2.03.001.
//! Every test is named `test_BC_2_03_001_xxx` per the TDD naming convention.
//!
//! **Red Gate discipline:** All tests in this file MUST FAIL before the implementing
//! engineer brings `engine.rs` and `hook_events.rs` to production-grade per
//! SS-engine-module.md v1.1.20.
//!
//! Probe mapping (from VP-019 §Probe Matrix):
//! - 19.a — method count == 5; names match `{id, metadata, detect, enrich, on_hook}`
//! - 19.b — no `Sealed` supertrait; `Send + Sync + 'static` only
//! - 19.c — `metadata` return type: `Result < EngineMetadata , EngineMetadataError >`
//! - 19.d — `enrich` return type: `Result < EnrichedSession , EngineMetadataError >`
//! - 19.e — `EnrichedSession::last_event_micros` is `Option < i64 >` (not bare `i64`)
//! - 19.f — all 7 supporting types are `pub` in `monocle_core::engine`
//! - 19.g — no `Sealed` token in supertrait position (AC-002)
//! - 19.h — `#[async_trait]` attribute present on trait declaration (AC-007)

// The TDD naming convention `test_BC_S_SS_NNN_xxx` uses uppercase BC-ID segments,
// which triggers the non_snake_case lint. Allow it for test names in this file.
#![allow(non_snake_case)]
// Some intermediate bindings (field, attr) are extracted for their side-effect
// (.expect()) and then consumed via derived values. Allow the unused-variable
// lint in this test file.
#![allow(unused_variables)]
// Test code legitimately uses `.expect()` / `.unwrap()` to surface test failures.
// The workspace clippy lint (expect_used = "warn") is correctly suppressed in tests.
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
// The compile-time type probe uses `assert!(true)` as a sentinel — suppress that lint.
#![allow(clippy::assertions_on_constants)]
// The `#[allow(clippy::zero_prefixed_literal)]` appears inside a function; suppress globally.
#![allow(clippy::zero_prefixed_literal)]
// `panic!()` in find_engine_module_trait is correct test-harness behavior.
#![allow(clippy::panic)]

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use syn::{ItemTrait, TraitItem};

// ──────────────────────────────────────────────────────────────────────────────
// Helper: resolve engine.rs path relative to CARGO_MANIFEST_DIR
// ──────────────────────────────────────────────────────────────────────────────

fn engine_rs_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("src").join("engine.rs")
}

fn hook_events_rs_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("src")
        .join("hook_events.rs")
}

fn parse_engine_rs() -> syn::File {
    let src =
        fs::read_to_string(engine_rs_path()).expect("cannot read engine.rs — is the path correct?");
    syn::parse_file(&src).expect("engine.rs is not valid Rust syntax")
}

fn parse_hook_events_rs() -> syn::File {
    let src = fs::read_to_string(hook_events_rs_path())
        .expect("cannot read hook_events.rs — is the path correct?");
    syn::parse_file(&src).expect("hook_events.rs is not valid Rust syntax")
}

/// Find the `EngineModule` trait item in the parsed file.
fn find_engine_module_trait(file: &syn::File) -> &ItemTrait {
    for item in &file.items {
        if let syn::Item::Trait(t) = item {
            if t.ident == "EngineModule" {
                return t;
            }
        }
    }
    panic!("EngineModule trait not found in engine.rs");
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-019 Probe 19.a — method count and canonical name set
// (AC-001, AC-006; traces to BC-2.03.001 postcondition 1)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-019 probe 19.a and AC-001.
///
/// `EngineModule` MUST have exactly 5 trait methods with the canonical names:
/// `id`, `metadata`, `detect`, `enrich`, `on_hook`.
#[test]
fn test_BC_2_03_001_vp019_19a_exactly_5_methods_with_canonical_names() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    let method_names: HashSet<String> = trait_item
        .items
        .iter()
        .filter_map(|item| {
            if let TraitItem::Fn(m) = item {
                Some(m.sig.ident.to_string())
            } else {
                None
            }
        })
        .collect();

    let expected: HashSet<String> = ["id", "metadata", "detect", "enrich", "on_hook"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    assert_eq!(
        method_names.len(),
        5,
        "EngineModule must have exactly 5 methods; found {}: {:?}",
        method_names.len(),
        method_names
    );
    assert_eq!(
        method_names, expected,
        "EngineModule method names do not match canonical set.\nExpected: {:?}\nFound:    {:?}",
        expected, method_names
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-019 Probe 19.b — no `Sealed` supertrait; `Send + Sync + 'static` present
// (AC-002, AC-006; traces to BC-2.03.001 postcondition 2 and invariant 1)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-019 probe 19.b and AC-002.
///
/// The `EngineModule` trait MUST NOT have a `private::Sealed` or any `Sealed` supertrait.
/// Supertrait bounds must be `Send + Sync + 'static` only.
/// Implementation: syn 2 substring-absence check for the token `Sealed` in the
/// supertrait bound position.
#[test]
fn test_BC_2_03_001_vp019_19b_no_sealed_supertrait() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    // Collect all supertrait bound tokens as a string
    let supertraits_ts: String = trait_item
        .supertraits
        .iter()
        .map(|bound| quote::quote!(#bound).to_string())
        .collect::<Vec<_>>()
        .join(" + ");

    assert!(
        !supertraits_ts.contains("Sealed"),
        "EngineModule MUST NOT have a Sealed supertrait (BC-2.03.001 PC-2, AC-002). \
         Found in supertrait bounds: {supertraits_ts:?}"
    );
}

/// Exercises VP-019 probe 19.b (positive assertion): `Send + Sync + 'static` present.
#[test]
fn test_BC_2_03_001_vp019_19b_send_sync_static_supertraits_present() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    let supertraits_ts: String = trait_item
        .supertraits
        .iter()
        .map(|bound| quote::quote!(#bound).to_string())
        .collect::<Vec<_>>()
        .join(" + ");

    assert!(
        supertraits_ts.contains("Send"),
        "EngineModule must have `Send` supertrait. Found: {supertraits_ts:?}"
    );
    assert!(
        supertraits_ts.contains("Sync"),
        "EngineModule must have `Sync` supertrait. Found: {supertraits_ts:?}"
    );
    assert!(
        supertraits_ts.contains("'static"),
        "EngineModule must have `'static` supertrait. Found: {supertraits_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-019 Probe 19.c — `metadata` return type token stream
// (AC-001; traces to BC-2.03.001 postcondition 1)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-019 probe 19.c.
///
/// `metadata(&self) -> Result<EngineMetadata, EngineMetadataError>` — the return type
/// token stream must match the canonical form.
#[test]
fn test_BC_2_03_001_vp019_19c_metadata_return_type_is_result_engine_metadata_error() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    let metadata_method = trait_item.items.iter().find_map(|item| {
        if let TraitItem::Fn(m) = item {
            if m.sig.ident == "metadata" {
                return Some(m);
            }
        }
        None
    });

    let method = metadata_method.expect("metadata method not found in EngineModule trait");

    // Extract return type token string; normalize whitespace for comparison
    let ret_ts = match &method.sig.output {
        syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
        syn::ReturnType::Default => String::new(),
    };

    // Canonical: "Result < EngineMetadata , EngineMetadataError >"
    assert!(
        ret_ts.contains("EngineMetadata") && ret_ts.contains("EngineMetadataError"),
        "metadata() return type must be Result<EngineMetadata, EngineMetadataError>. \
         Found: {ret_ts:?}"
    );
    assert!(
        ret_ts.starts_with("Result"),
        "metadata() return type must start with Result. Found: {ret_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-019 Probe 19.d — `enrich` return type token stream
// (AC-001; traces to BC-2.03.001 postcondition 1)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-019 probe 19.d.
///
/// `enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>`.
#[test]
fn test_BC_2_03_001_vp019_19d_enrich_return_type_is_result_enriched_session_error() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    let enrich_method = trait_item.items.iter().find_map(|item| {
        if let TraitItem::Fn(m) = item {
            if m.sig.ident == "enrich" {
                return Some(m);
            }
        }
        None
    });

    let method = enrich_method.expect("enrich method not found in EngineModule trait");

    let ret_ts = match &method.sig.output {
        syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
        syn::ReturnType::Default => String::new(),
    };

    assert!(
        ret_ts.contains("EnrichedSession") && ret_ts.contains("EngineMetadataError"),
        "enrich() return type must be Result<EnrichedSession, EngineMetadataError>. \
         Found: {ret_ts:?}"
    );
    assert!(
        ret_ts.starts_with("Result"),
        "enrich() return type must start with Result. Found: {ret_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-019 Probe 19.e — `EnrichedSession::last_event_micros` is `Option<i64>`
// (AC-004; traces to BC-2.03.001 postcondition 4)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-019 probe 19.e and AC-004 via AST audit.
///
/// `EnrichedSession::last_event_micros` MUST be `Option<i64>` — NOT bare `i64`.
/// A bare `i64` would require `0` as a sentinel for "no events received", which is
/// explicitly forbidden (0 is the Unix epoch, 1970-01-01). BC-2.03.001 PC-4.
///
/// This AST-level test is the primary detection mechanism. The Red Gate is verified
/// when this test fails against the `i64` stub.
#[test]
fn test_BC_2_03_001_vp019_19e_enriched_session_last_event_micros_is_option_i64() {
    let file = parse_engine_rs();

    // Find the EnrichedSession struct
    let enriched_session = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "EnrichedSession" {
                return Some(s);
            }
        }
        None
    });

    let s = enriched_session.expect("EnrichedSession struct not found in engine.rs");

    let last_event_field = s.fields.iter().find(|f| {
        f.ident
            .as_ref()
            .map(|id| id == "last_event_micros")
            .unwrap_or(false)
    });

    let field =
        last_event_field.expect("EnrichedSession must have a `last_event_micros` field (AC-004)");

    let field_ty_ts = {
        let ty = &field.ty;
        quote::quote!(#ty)
    }
    .to_string();

    // Must be Option<i64> — token stream contains "Option" and "i64"
    assert!(
        field_ty_ts.contains("Option"),
        "EnrichedSession::last_event_micros must be Option<i64>, not bare i64. \
         Found type: {field_ty_ts:?} \
         (BC-2.03.001 PC-4; treating 0 as sentinel for Unix epoch is forbidden)"
    );
    assert!(
        field_ty_ts.contains("i64"),
        "EnrichedSession::last_event_micros must be Option<i64>. Found: {field_ty_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-019 Probe 19.f — supporting types are pub in `monocle_core::engine`
// (AC-003; traces to BC-2.03.001 postcondition 3)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-019 probe 19.f and AC-003.
///
/// All 7 supporting types must be declared `pub` in `monocle_core::engine`:
/// `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`, `SessionStatus`,
/// `HookResponse`, `HookDecision`, `EngineMetadataError`.
///
/// This is a compile-time type-resolution probe: if any of these types does not
/// exist as a pub item in `monocle_core::engine`, this test will fail to compile.
#[test]
fn test_BC_2_03_001_vp019_19f_supporting_types_pub_in_monocle_core_engine() {
    // Compile-time probe: use each type in a let binding so rustc verifies pub access.
    // These are type-only assertions; no values are needed.
    fn _type_probe() {
        let _: Option<monocle_core::engine::EngineMetadata>;
        let _: Option<monocle_core::engine::ProcessSnapshot>;
        let _: Option<monocle_core::engine::EnrichedSession>;
        let _: Option<monocle_core::engine::SessionStatus>;
        let _: Option<monocle_core::engine::HookResponse>;
        let _: Option<monocle_core::engine::HookDecision>;
        let _: Option<monocle_core::engine::EngineMetadataError>;
    }
    // If the function above compiles, all types exist. The runtime assertion confirms it ran.
    assert!(
        true,
        "all 7 supporting types are pub in monocle_core::engine (compile-time verified)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-019 Probe 19.g (alias: 19.b negative) — no `Sealed` identifier anywhere in trait
// (AC-002; traces to BC-2.03.001 postcondition 2)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-019 probe 19.g: verifies that the `Sealed` identifier
/// does not appear anywhere in the `EngineModule` trait declaration in engine.rs.
///
/// This is the canonical AC-002 syn 2 substring-absence check.
#[test]
fn test_BC_2_03_001_vp019_19g_sealed_token_absent_from_trait_declaration() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);
    let trait_ts = quote::quote!(#trait_item).to_string();

    assert!(
        !trait_ts.contains("Sealed"),
        "The `Sealed` identifier must not appear in the EngineModule trait declaration \
         (BC-2.03.001 PC-2, AC-002, VP-019 probe 19.g). Found in trait token stream: Sealed"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-019 Probe 19.h (alias: AC-007) — `#[async_trait]` attribute on trait declaration
// (AC-007; traces to BC-2.03.001 invariant 3)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-019 probe 19.h and AC-007.
///
/// `EngineModule` MUST have `#[async_trait::async_trait]` applied. The VP-019 harness
/// verifies this via syn 2 attribute inspection.
#[test]
fn test_BC_2_03_001_vp019_19h_async_trait_attribute_on_engine_module() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    let has_async_trait_attr = trait_item.attrs.iter().any(|attr| {
        {
            let path = attr.path();
            quote::quote!(#path)
        }
        .to_string()
        .contains("async_trait")
    });

    assert!(
        has_async_trait_attr,
        "EngineModule trait MUST carry `#[async_trait::async_trait]` attribute \
         (BC-2.03.001 invariant 3, AC-007). Not found in trait attributes."
    );
}

/// Exercises AC-007: rustdoc on `EngineModule` must contain the canonical rationale text.
///
/// BC-2.03.001 invariant 3 requires a rustdoc comment explaining why `#[async_trait]`
/// is necessary. The canonical substring is `"native async fn in traits does not provide"`.
#[test]
fn test_BC_2_03_001_ac007_async_trait_rationale_text_in_engine_module_rustdoc() {
    let src = fs::read_to_string(engine_rs_path()).expect("cannot read engine.rs");

    // The canonical rationale text per BC-2.03.001 invariant 3 must appear in
    // the source of engine.rs. AC-007 requires specifically:
    //   "native async fn in traits does not provide" (canonical BC-2.03.001 INV-3 text)
    let has_canonical_rationale = src.contains("native async fn in traits does not provide");

    assert!(
        has_canonical_rationale,
        "EngineModule trait rustdoc MUST contain the canonical async_trait rationale text: \
         'native async fn in traits does not provide' \
         (BC-2.03.001 invariant 3, AC-007). \
         This text was not found in engine.rs. Add it to the `///` rustdoc comment on the \
         `EngineModule` trait declaration explaining why #[async_trait] is required."
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-001 — exact method signatures (extends 19.a to include async-ness and arg types)
// (traces to BC-2.03.001 postcondition 1, EC-030)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-001: `id` method signature is `fn id(&self) -> &'static str`.
#[test]
fn test_BC_2_03_001_ac001_id_method_signature() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    let id_method = trait_item.items.iter().find_map(|item| {
        if let TraitItem::Fn(m) = item {
            if m.sig.ident == "id" {
                return Some(m);
            }
        }
        None
    });

    let method = id_method.expect("id method not found in EngineModule trait");

    // Must NOT be async
    assert!(
        method.sig.asyncness.is_none(),
        "id() must be a sync method (not async)"
    );

    let ret_ts = match &method.sig.output {
        syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
        syn::ReturnType::Default => String::new(),
    };

    assert!(
        ret_ts.contains("'static") && ret_ts.contains("str"),
        "id() must return &'static str. Found: {ret_ts:?}"
    );
}

/// Exercises AC-001: `detect` method signature is `fn detect(&self, proc: &ProcessSnapshot) -> bool`.
#[test]
fn test_BC_2_03_001_ac001_detect_method_signature() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    let detect_method = trait_item.items.iter().find_map(|item| {
        if let TraitItem::Fn(m) = item {
            if m.sig.ident == "detect" {
                return Some(m);
            }
        }
        None
    });

    let method = detect_method.expect("detect method not found in EngineModule trait");

    // Must NOT be async (DI-006: pure function, no I/O)
    assert!(
        method.sig.asyncness.is_none(),
        "detect() must be a sync method (not async) — DI-006 pure-function contract"
    );

    let ret_ts = match &method.sig.output {
        syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
        syn::ReturnType::Default => String::new(),
    };

    assert_eq!(
        ret_ts, "bool",
        "detect() must return bool. Found: {ret_ts:?}"
    );
}

/// Exercises AC-001: `on_hook` method is `async` and returns `HookResponse`.
#[test]
fn test_BC_2_03_001_ac001_on_hook_method_signature() {
    let file = parse_engine_rs();
    let trait_item = find_engine_module_trait(&file);

    let on_hook_method = trait_item.items.iter().find_map(|item| {
        if let TraitItem::Fn(m) = item {
            if m.sig.ident == "on_hook" {
                return Some(m);
            }
        }
        None
    });

    let method = on_hook_method.expect("on_hook method not found in EngineModule trait");

    // Must be async
    assert!(
        method.sig.asyncness.is_some(),
        "on_hook() must be async (BC-2.03.001 PC-1)"
    );

    let ret_ts = match &method.sig.output {
        syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
        syn::ReturnType::Default => String::new(),
    };

    assert_eq!(
        ret_ts, "HookResponse",
        "on_hook() must return HookResponse. Found: {ret_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-003 — supporting types co-location and field audit
// (traces to BC-2.03.001 postcondition 3)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-003: `SessionStatus` has exactly 5 canonical variants.
///
/// Canonical variants per SS-engine-module.md v1.1.20 (F-D-01 fix):
/// `Active`, `Idle`, `WaitingOnPermission`, `Stopping`, `Stopped`.
#[test]
fn test_BC_2_03_001_ac003_session_status_has_5_canonical_variants() {
    let file = parse_engine_rs();

    let session_status = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "SessionStatus" {
                return Some(e);
            }
        }
        None
    });

    let e = session_status.expect("SessionStatus enum not found in engine.rs");

    let variant_names: HashSet<String> = e.variants.iter().map(|v| v.ident.to_string()).collect();

    let expected: HashSet<String> = [
        "Active",
        "Idle",
        "WaitingOnPermission",
        "Stopping",
        "Stopped",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    assert_eq!(
        variant_names.len(),
        5,
        "SessionStatus must have exactly 5 variants (SS-engine-module.md v1.1.20 F-D-01). \
         Found {}: {:?}",
        variant_names.len(),
        variant_names
    );
    assert_eq!(
        variant_names, expected,
        "SessionStatus variant names do not match canonical set.\nExpected: {:?}\nFound:    {:?}",
        expected, variant_names
    );
}

/// Exercises AC-003: `HookResponse` has canonical 3-field set:
/// `decision`, `redirect_url: Option<String>`, `diagnostic: Option<String>`.
///
/// Per SS-engine-module.md v1.1.20 (F-D-02 fix). `DeferUntil` is NOT part of this type (F-D-03).
#[test]
fn test_BC_2_03_001_ac003_hook_response_has_canonical_3_fields() {
    let file = parse_engine_rs();

    let hook_response = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "HookResponse" {
                return Some(s);
            }
        }
        None
    });

    let s = hook_response.expect("HookResponse struct not found in engine.rs");

    let field_names: HashSet<String> = s
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
        .collect();

    let required: HashSet<String> = ["decision", "redirect_url", "diagnostic"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    for req in &required {
        assert!(
            field_names.contains(req),
            "HookResponse must have field `{req}` (SS-engine-module.md v1.1.20 F-D-02). \
             Found fields: {:?}",
            field_names
        );
    }

    assert_eq!(
        field_names.len(),
        3,
        "HookResponse must have exactly 3 fields: decision, redirect_url, diagnostic. \
         Found {}: {:?}",
        field_names.len(),
        field_names
    );
}

/// Exercises AC-003: `HookResponse::redirect_url` is `Option<String>`.
#[test]
fn test_BC_2_03_001_ac003_hook_response_redirect_url_is_option_string() {
    let file = parse_engine_rs();

    let hook_response = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "HookResponse" {
                return Some(s);
            }
        }
        None
    });

    let s = hook_response.expect("HookResponse struct not found in engine.rs");

    let redirect_field = s.fields.iter().find(|f| {
        f.ident
            .as_ref()
            .map(|id| id == "redirect_url")
            .unwrap_or(false)
    });

    let field = redirect_field.expect("HookResponse must have a `redirect_url` field (AC-003)");
    let field_ty_ts = {
        let ty = &field.ty;
        quote::quote!(#ty)
    }
    .to_string();

    assert!(
        field_ty_ts.contains("Option") && field_ty_ts.contains("String"),
        "HookResponse::redirect_url must be Option<String>. Found: {field_ty_ts:?}"
    );
}

/// Exercises AC-003: `HookResponse::diagnostic` is `Option<String>`.
#[test]
fn test_BC_2_03_001_ac003_hook_response_diagnostic_is_option_string() {
    let file = parse_engine_rs();

    let hook_response = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "HookResponse" {
                return Some(s);
            }
        }
        None
    });

    let s = hook_response.expect("HookResponse struct not found in engine.rs");

    let diagnostic_field = s.fields.iter().find(|f| {
        f.ident
            .as_ref()
            .map(|id| id == "diagnostic")
            .unwrap_or(false)
    });

    let field = diagnostic_field.expect("HookResponse must have a `diagnostic` field (AC-003)");
    let field_ty_ts = {
        let ty = &field.ty;
        quote::quote!(#ty)
    }
    .to_string();

    assert!(
        field_ty_ts.contains("Option") && field_ty_ts.contains("String"),
        "HookResponse::diagnostic must be Option<String>. Found: {field_ty_ts:?}"
    );
}

/// Exercises AC-003: `HookDecision` has exactly 3 canonical variants: `Allow`, `Block`, `Defer`.
#[test]
fn test_BC_2_03_001_ac003_hook_decision_has_3_canonical_variants() {
    let file = parse_engine_rs();

    let hook_decision = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "HookDecision" {
                return Some(e);
            }
        }
        None
    });

    let e = hook_decision.expect("HookDecision enum not found in engine.rs");

    let variant_names: HashSet<String> = e.variants.iter().map(|v| v.ident.to_string()).collect();

    let expected: HashSet<String> = ["Allow", "Block", "Defer"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    assert_eq!(
        variant_names, expected,
        "HookDecision variants must be exactly {{Allow, Block, Defer}} (AC-003). \
         Found: {:?}",
        variant_names
    );
}

/// Exercises AC-003: `EngineMetadataError` is `#[non_exhaustive]`.
#[test]
fn test_BC_2_03_001_ac003_engine_metadata_error_is_non_exhaustive() {
    let file = parse_engine_rs();

    let err_enum = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "EngineMetadataError" {
                return Some(e);
            }
        }
        None
    });

    let e = err_enum.expect("EngineMetadataError not found in engine.rs");

    let has_non_exhaustive = e.attrs.iter().any(|attr| {
        {
            let path = attr.path();
            quote::quote!(#path)
        }
        .to_string()
        .contains("non_exhaustive")
    });

    assert!(
        has_non_exhaustive,
        "EngineMetadataError must carry #[non_exhaustive] (SS-core-types-and-abi §General Rule)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-003b — HookEvent in `monocle_core::hook_events`
// (traces to BC-2.03.001 invariant 2 + BC-2.02.003 invariant)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-003b: `HookEvent` is in `monocle_core::hook_events` (not re-declared in engine).
///
/// The test verifies that `HookEvent` does NOT appear as an enum declaration in `engine.rs`.
#[test]
fn test_BC_2_03_001_ac003b_hook_event_not_declared_in_engine_rs() {
    let file = parse_engine_rs();

    let hook_event_in_engine = file.items.iter().any(|item| {
        if let syn::Item::Enum(e) = item {
            e.ident == "HookEvent"
        } else {
            false
        }
    });

    assert!(
        !hook_event_in_engine,
        "HookEvent must NOT be declared in engine.rs — it lives in hook_events.rs \
         (BC-2.03.001 invariant 2). Found a HookEvent enum declaration in engine.rs."
    );
}

/// Exercises AC-003b: `HookEvent` carries `#[non_exhaustive]` in `hook_events.rs`.
#[test]
fn test_BC_2_03_001_ac003b_hook_event_is_non_exhaustive() {
    let file = parse_hook_events_rs();

    let hook_event = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "HookEvent" {
                return Some(e);
            }
        }
        None
    });

    let e = hook_event.expect("HookEvent enum not found in hook_events.rs");

    let has_non_exhaustive = e.attrs.iter().any(|attr| {
        {
            let path = attr.path();
            quote::quote!(#path)
        }
        .to_string()
        .contains("non_exhaustive")
    });

    assert!(
        has_non_exhaustive,
        "HookEvent must carry #[non_exhaustive] (BC-2.02.003, BC-2.03.001 invariant 2)"
    );
}

/// Exercises AC-003b: `HookEvent` has exactly 5 Phase 1 variants.
///
/// Canonical Phase 1 variants: `SessionStart`, `UserPromptSubmit`, `PreToolUse`,
/// `Notification`, `Stop`. `PostToolUse` MUST NOT be present (JC-2, BC-2.03.004 invariant 1).
#[test]
fn test_BC_2_03_001_ac003b_hook_event_has_5_phase1_variants_no_post_tool_use() {
    let file = parse_hook_events_rs();

    let hook_event = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "HookEvent" {
                return Some(e);
            }
        }
        None
    });

    let e = hook_event.expect("HookEvent enum not found in hook_events.rs");

    let variant_names: HashSet<String> = e.variants.iter().map(|v| v.ident.to_string()).collect();

    let expected: HashSet<String> = [
        "SessionStart",
        "UserPromptSubmit",
        "PreToolUse",
        "Notification",
        "Stop",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    assert_eq!(
        variant_names.len(),
        5,
        "HookEvent must have exactly 5 Phase 1 variants (JC-2, BC-2.03.004 invariant 1). \
         Found {}: {:?}",
        variant_names.len(),
        variant_names
    );
    assert_eq!(
        variant_names, expected,
        "HookEvent variants do not match canonical Phase 1 set.\nExpected: {:?}\nFound:    {:?}",
        expected, variant_names
    );

    assert!(
        !variant_names.contains("PostToolUse"),
        "HookEvent must NOT have PostToolUse variant in Phase 1 (JC-2, BC-2.03.004 invariant 1)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-005 — `EngineMetadataError::HomeUnresolvable` exists
// (traces to BC-2.03.001 postcondition 5, EC-029)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-005 (EC-029): `EngineMetadataError::HomeUnresolvable` variant exists
/// and is AST-verified in the source file.
///
/// This is the fail-fast error returned by `metadata()` and `enrich()` when the
/// platform home directory is unresolvable. BC-2.03.001 PC-5.
#[test]
fn test_BC_2_03_001_ac005_engine_metadata_error_home_unresolvable_exists() {
    let file = parse_engine_rs();

    let err_enum = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "EngineMetadataError" {
                return Some(e);
            }
        }
        None
    });

    let e = err_enum.expect("EngineMetadataError not found in engine.rs");

    let has_home_unresolvable = e.variants.iter().any(|v| v.ident == "HomeUnresolvable");

    assert!(
        has_home_unresolvable,
        "EngineMetadataError must have a `HomeUnresolvable` variant (BC-2.03.001 PC-5, AC-005, EC-029)"
    );
}

/// Exercises AC-005 (invariant): `EngineMetadataError::HomeUnresolvable` is constructable
/// and pattern-matchable — verifies type-system participation.
#[test]
fn test_BC_2_03_001_ac005_home_unresolvable_is_usable_in_match() {
    use monocle_core::engine::EngineMetadataError;

    // Construct the error directly; verify it pattern-matches to HomeUnresolvable.
    let err = EngineMetadataError::HomeUnresolvable;
    assert!(
        matches!(err, EngineMetadataError::HomeUnresolvable),
        "EngineMetadataError::HomeUnresolvable must be a valid, matchable variant (AC-005)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-004 (AST + runtime) — `last_event_micros: Option<i64>` semantics
// (traces to BC-2.03.001 postcondition 4)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-004 (runtime): verifies that `last_event_micros` being `Option<i64>`
/// allows `None` and `Some(0)` to be distinct.
///
/// **Red Gate note:** This test asserts the TYPE returned from `last_event_micros`
/// can be compared as `Option<i64>`. If the field is bare `i64` in the stub, the
/// compile-time type checks in `test_BC_2_03_001_vp019_19e_*` will catch it; this
/// runtime test verifies the semantic contract once the type is corrected.
///
/// We test using AST inspection here so the test compiles against both the stub
/// (which has `i64`) and the final implementation (which will have `Option<i64>`).
/// The failing assertion in `test_BC_2_03_001_vp019_19e_*` is the primary Red Gate test.
#[test]
fn test_BC_2_03_001_ac004_last_event_micros_field_type_verified_by_ast() {
    // This test mirrors 19.e — explicitly named for AC-004 traceability.
    let file = parse_engine_rs();

    let enriched_session = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "EnrichedSession" {
                return Some(s);
            }
        }
        None
    });

    let s = enriched_session.expect("EnrichedSession not found in engine.rs");

    let field = s.fields.iter().find(|f| {
        f.ident
            .as_ref()
            .map(|id| id == "last_event_micros")
            .unwrap_or(false)
    });

    let f = field.expect("last_event_micros field must exist on EnrichedSession (AC-004)");
    let ty_ts = {
        let ty = &f.ty;
        quote::quote!(#ty)
    }
    .to_string();

    assert!(
        ty_ts.contains("Option"),
        "AC-004: last_event_micros must be Option<i64> (None = no events; \
         Some(0) = Unix epoch, NOT a sentinel). Found type: {ty_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// EC-031 — wildcard arm required in `on_hook` match sites for #[non_exhaustive] HookEvent
// (traces to BC-2.03.001 edge case EC-031)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises EC-031: `HookDecision::Allow` is the fail-open default for unrecognized
/// `HookEvent` variants. This test verifies that `HookResponse::new(HookDecision::Allow)`
/// constructs correctly and is the canonical no-match response.
#[test]
fn test_BC_2_03_001_ec031_hook_response_allow_is_valid_fail_open_default() {
    use monocle_core::engine::{HookDecision, HookResponse};

    let fail_open = HookResponse::new(HookDecision::Allow);
    assert!(
        matches!(fail_open.decision, HookDecision::Allow),
        "HookResponse::new(HookDecision::Allow) must produce an Allow decision (EC-031 fail-open)"
    );
}

/// Exercises EC-031: a wildcard match arm on `HookEvent` compiles and resolves to Allow.
///
/// `HookEvent` must be deserialized from JSON to avoid `#[non_exhaustive]` E0639.
/// Inner structs (SessionStartEvent etc.) are serde-deserialize-only per
/// SS-engine-module.md §Cross-Crate Constructor Audit.
#[test]
fn test_BC_2_03_001_ec031_wildcard_match_arm_on_hook_event_produces_allow() {
    use monocle_core::engine::{HookDecision, HookResponse};
    use monocle_core::hook_events::HookEvent;

    // Deserialize HookEvent from JSON — the canonical construction path for
    // #[non_exhaustive] event structs outside monocle-core (E0639 applies to
    // struct literal; serde internal construction is exempt per audit table).
    let json = r#"{
        "SessionStart": {
            "cwd": "/tmp",
            "transcript_path": "/tmp/transcript",
            "session_id": "test-session",
            "pid": 12345
        }
    }"#;
    let event: HookEvent = serde_json::from_str(json)
        .expect("HookEvent must deserialize from canonical JSON (serde-deserialize-only path)");

    // Wildcard arm is required because HookEvent is #[non_exhaustive].
    let response = match event {
        HookEvent::SessionStart(_) => HookResponse::new(HookDecision::Allow),
        HookEvent::UserPromptSubmit(_) => HookResponse::new(HookDecision::Allow),
        HookEvent::PreToolUse(_) => HookResponse::new(HookDecision::Allow),
        HookEvent::Notification(_) => HookResponse::new(HookDecision::Allow),
        HookEvent::Stop(_) => HookResponse::new(HookDecision::Allow),
        // Required wildcard arm for Phase 4 additions (EC-031):
        _ => HookResponse::new(HookDecision::Allow),
    };

    assert!(
        matches!(response.decision, HookDecision::Allow),
        "Wildcard arm must produce Allow (fail-open) per EC-031"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-006 — trait is OPEN (no Sealed bound; third-party crates may implement)
// (traces to BC-2.03.001 invariant 1)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises AC-006: `EngineModule` is an open trait (no sealed bound).
///
/// A downstream-crate struct can implement `EngineModule`. This test verifies
/// via a local mock implementation that the trait is implementable from outside
/// `monocle-core`.
#[test]
fn test_BC_2_03_001_ac006_engine_module_is_open_trait_implementable_from_outside() {
    use monocle_core::engine::EngineModule;
    use monocle_core::engine::{
        EngineMetadata, EngineMetadataError, EnrichedSession, HookDecision, HookResponse,
        ProcessSnapshot, SessionStatus,
    };
    use monocle_core::hook_events::HookEvent;

    struct MockEngine;

    #[async_trait::async_trait]
    impl EngineModule for MockEngine {
        fn id(&self) -> &'static str {
            "mock-engine"
        }

        fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError> {
            Ok(EngineMetadata::new("Mock Engine", '\u{25A1}', vec![], 1))
        }

        fn detect(&self, _proc: &ProcessSnapshot) -> bool {
            false
        }

        async fn enrich(
            &self,
            proc: &ProcessSnapshot,
        ) -> Result<EnrichedSession, EngineMetadataError> {
            // NOTE: Once last_event_micros is corrected to Option<i64> (AC-004),
            // this value should be changed to None. The 0 value is a stub-compat
            // placeholder; the type error from None is caught by VP-019 probe 19.e.
            #[allow(clippy::zero_prefixed_literal)]
            Ok(EnrichedSession::new(
                format!("pid-{}", proc.pid),
                self.id().to_string(),
                None,
                None,
                SessionStatus::Idle,
                0, // stub-compat: must be None (Option<i64>) in production (AC-004)
            ))
        }

        async fn on_hook(&self, _event: HookEvent) -> HookResponse {
            HookResponse::new(HookDecision::Allow)
        }
    }

    let engine = MockEngine;
    assert_eq!(engine.id(), "mock-engine");
    let proc = ProcessSnapshot::new(1, None, vec![], 0);
    assert!(!engine.detect(&proc), "mock engine detects nothing");
}

// ──────────────────────────────────────────────────────────────────────────────
// Invariant: `#[non_exhaustive]` on all supporting struct types
// (traces to SS-core-types-and-abi §General Rule, BC-2.02.003)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises the non-exhaustive policy for all structs in `engine.rs`.
///
/// `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`, `HookResponse` must ALL
/// carry `#[non_exhaustive]` per SS-core-types-and-abi §General Rule.
#[test]
fn test_BC_2_03_001_invariant_non_exhaustive_on_all_supporting_structs() {
    let file = parse_engine_rs();
    let required_structs = [
        "EngineMetadata",
        "ProcessSnapshot",
        "EnrichedSession",
        "HookResponse",
    ];

    for struct_name in required_structs {
        let struct_item = file.items.iter().find_map(|item| {
            if let syn::Item::Struct(s) = item {
                if s.ident == struct_name {
                    return Some(s);
                }
            }
            None
        });

        let s =
            struct_item.unwrap_or_else(|| panic!("{struct_name} struct not found in engine.rs"));

        let has_non_exhaustive = s.attrs.iter().any(|attr| {
            {
                let path = attr.path();
                quote::quote!(#path)
            }
            .to_string()
            .contains("non_exhaustive")
        });

        assert!(
            has_non_exhaustive,
            "{struct_name} must carry #[non_exhaustive] (SS-core-types-and-abi §General Rule)"
        );
    }
}

/// Exercises the non-exhaustive policy for all enum types in `engine.rs`.
///
/// `SessionStatus`, `HookDecision`, `EngineMetadataError` must ALL carry `#[non_exhaustive]`.
#[test]
fn test_BC_2_03_001_invariant_non_exhaustive_on_all_supporting_enums() {
    let file = parse_engine_rs();
    let required_enums = ["SessionStatus", "HookDecision", "EngineMetadataError"];

    for enum_name in required_enums {
        let enum_item = file.items.iter().find_map(|item| {
            if let syn::Item::Enum(e) = item {
                if e.ident == enum_name {
                    return Some(e);
                }
            }
            None
        });

        let e = enum_item.unwrap_or_else(|| panic!("{enum_name} enum not found in engine.rs"));

        let has_non_exhaustive = e.attrs.iter().any(|attr| {
            {
                let path = attr.path();
                quote::quote!(#path)
            }
            .to_string()
            .contains("non_exhaustive")
        });

        assert!(
            has_non_exhaustive,
            "{enum_name} must carry #[non_exhaustive] (SS-core-types-and-abi §General Rule)"
        );
    }
}
