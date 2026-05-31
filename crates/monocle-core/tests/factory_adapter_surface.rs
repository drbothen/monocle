//! VP-014 AST audit tests for `FactoryAdapter` trait surface (BC-2.02.004, S-012).
//!
//! These tests use `syn` to parse the factory module source and assert structural
//! properties that cannot be verified at runtime:
//! - exactly 7 methods (detect, matches, state_file_path, read_state, subscribe,
//!   display_name, abi_version)
//! - no `Sealed` supertrait bound
//! - `Send + Sync + 'static` supertraits present
//! - `FactoryState` has exactly 7 canonical fields (no `raw_frontmatter`)
//! - `FactoryDetection` has exactly 3 fields
//!
//! Canonical test-function names pinned per VP-014 probe matrix and BC-2.02.004.

// BC-based test names use uppercase segments.
#![allow(non_snake_case)]
// Test code uses `.expect()` / `.unwrap()` to surface assertion failures.
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
// `panic!` is correct test-harness behavior.
#![allow(clippy::panic)]
// `assert!(true)` used as a compile-time sentinel.
#![allow(clippy::assertions_on_constants)]

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use syn::{ItemTrait, TraitItem};

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Absolute path to `monocle-core/src/factory/mod.rs`.
///
/// The factory module lives at `src/factory/mod.rs` (subdirectory form), not
/// `src/factory.rs` (flat form).
fn factory_mod_rs_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("src")
        .join("factory")
        .join("mod.rs")
}

fn parse_factory_mod_rs() -> syn::File {
    let src = fs::read_to_string(factory_mod_rs_path())
        .expect("cannot read factory/mod.rs — check CARGO_MANIFEST_DIR");
    syn::parse_file(&src).expect("factory/mod.rs is not valid Rust syntax")
}

/// Find the `FactoryAdapter` trait in the parsed file.
fn find_factory_adapter_trait(file: &syn::File) -> &ItemTrait {
    for item in &file.items {
        if let syn::Item::Trait(t) = item {
            if t.ident == "FactoryAdapter" {
                return t;
            }
        }
    }
    panic!("FactoryAdapter trait not found in factory/mod.rs");
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014 Probe: 7 methods exact + canonical name set
// (AC-001; traces to BC-2.02.004 postcondition 1)
// ──────────────────────────────────────────────────────────────────────────────

/// Exercises VP-014 (AC-001): `FactoryAdapter` has exactly 7 methods with canonical names.
///
/// Canonical methods per BC-2.02.004 PC-1:
/// `detect`, `matches`, `state_file_path`, `read_state`, `subscribe`,
/// `display_name`, `abi_version`.
///
/// This is the canonical BC-2.02.004 line 98 test function (verbatim name required).
#[test]
fn test_BC_FACTORY_001_trait_defined_open_no_sealed_bound() {
    let file = parse_factory_mod_rs();
    let trait_item = find_factory_adapter_trait(&file);

    // ── 7 methods assertion ──────────────────────────────────────────────────
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

    let expected: HashSet<String> = [
        "detect",
        "matches",
        "state_file_path",
        "read_state",
        "subscribe",
        "display_name",
        "abi_version",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    assert_eq!(
        method_names.len(),
        7,
        "FactoryAdapter must have exactly 7 methods (BC-2.02.004 PC-1); \
         found {}: {method_names:?}",
        method_names.len(),
    );
    assert_eq!(
        method_names, expected,
        "FactoryAdapter method names do not match canonical set.\nExpected: {expected:?}\nFound:    {method_names:?}",
    );

    // ── No `Sealed` supertrait ───────────────────────────────────────────────
    let supertraits_ts: String = trait_item
        .supertraits
        .iter()
        .map(|bound| quote::quote!(#bound).to_string())
        .collect::<Vec<_>>()
        .join(" + ");

    assert!(
        !supertraits_ts.contains("Sealed"),
        "FactoryAdapter MUST NOT have a Sealed supertrait (BC-2.02.004 PC-2, AC-002). \
         Found in supertrait bounds: {supertraits_ts:?}"
    );

    // ── `Send + Sync + 'static` present ─────────────────────────────────────
    assert!(
        supertraits_ts.contains("Send"),
        "FactoryAdapter must have `Send` supertrait. Found: {supertraits_ts:?}"
    );
    assert!(
        supertraits_ts.contains("Sync"),
        "FactoryAdapter must have `Sync` supertrait. Found: {supertraits_ts:?}"
    );
    assert!(
        supertraits_ts.contains("'static"),
        "FactoryAdapter must have `'static` supertrait. Found: {supertraits_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014: `Sealed` token absent from trait supertraits and where clause
// (AC-002; traces to BC-2.02.004 postcondition 2)
// ──────────────────────────────────────────────────────────────────────────────

/// VP-014: `Sealed` identifier must not appear in `FactoryAdapter` supertrait
/// bounds or where clause — it must not be used as a supertrait bound.
///
/// Note: this check intentionally excludes doc comments (`///` text). Doc comments
/// explaining "no Sealed bound" are correct and expected; what is forbidden is
/// `Sealed` appearing as an actual bound in the supertrait list or where clauses.
///
/// Traces to: BC-2.02.004 PC-2; AC-002.
#[test]
fn test_BC_FACTORY_001_sealed_token_absent_from_trait_declaration() {
    let file = parse_factory_mod_rs();
    let trait_item = find_factory_adapter_trait(&file);

    // Check supertraits (bound position)
    let supertraits_ts: String = trait_item
        .supertraits
        .iter()
        .map(|bound| quote::quote!(#bound).to_string())
        .collect::<Vec<_>>()
        .join(" + ");

    assert!(
        !supertraits_ts.contains("Sealed"),
        "The `Sealed` identifier must not appear in FactoryAdapter supertrait bounds \
         (BC-2.02.004 PC-2, AC-002). Found `Sealed` in supertrait position: {supertraits_ts:?}"
    );

    // Check where clause
    let where_ts = trait_item
        .generics
        .where_clause
        .as_ref()
        .map(|wc| quote::quote!(#wc).to_string())
        .unwrap_or_default();

    assert!(
        !where_ts.contains("Sealed"),
        "The `Sealed` identifier must not appear in FactoryAdapter where clause \
         (BC-2.02.004 PC-2, AC-002). Found `Sealed` in where clause: {where_ts:?}"
    );

    // Check each method signature (not the body, which could include comments)
    for item in &trait_item.items {
        if let TraitItem::Fn(m) = item {
            let sig_ts = quote::quote!(#(#m.sig)).to_string();
            assert!(
                !sig_ts.contains("Sealed"),
                "The `Sealed` identifier must not appear in FactoryAdapter method signatures \
                 (BC-2.02.004 PC-2, AC-002). Found `Sealed` in method `{}` signature.",
                m.sig.ident
            );
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014: `FactoryState` has exactly 7 canonical fields; NO `raw_frontmatter`
// (AC-004; traces to BC-2.02.004 postcondition 4)
// ──────────────────────────────────────────────────────────────────────────────

/// VP-014 (AC-004): `FactoryState` struct has exactly 7 canonical fields.
///
/// Canonical fields per SS-core-types-and-abi.md §FactoryState (lines 364–400):
/// `phase`, `status`, `awaiting`, `blocking_issues`, `convergence`, `cycle`, `custom_fields`.
///
/// `raw_frontmatter` is the BC-2.02.004 INV-2 red-line forbidden field.
///
/// Traces to: BC-2.02.004 PC-4; AC-004.
#[test]
fn test_BC_FACTORY_001_factory_state_exactly_7_fields() {
    let file = parse_factory_mod_rs();

    let factory_state = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "FactoryState" {
                return Some(s);
            }
        }
        None
    });

    let s = factory_state.expect("FactoryState struct not found in factory/mod.rs");

    let field_names: HashSet<String> = s
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
        .collect();

    let expected: HashSet<String> = [
        "phase",
        "status",
        "awaiting",
        "blocking_issues",
        "convergence",
        "cycle",
        "custom_fields",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    // Forbidden field check must come before count to give clearer failure
    assert!(
        !field_names.contains("raw_frontmatter"),
        "FactoryState MUST NOT have a `raw_frontmatter` field — it is the BC-2.02.004 INV-2 \
         red-line forbidden field per SS-core-types-and-abi.md §FactoryState. \
         Traces to: AC-004; BC-2.02.004 INV-2."
    );

    assert_eq!(
        field_names.len(),
        7,
        "FactoryState must have exactly 7 canonical fields (BC-2.02.004 PC-4). \
         Found {}: {field_names:?}",
        field_names.len(),
    );

    assert_eq!(
        field_names, expected,
        "FactoryState field names do not match canonical set.\nExpected: {expected:?}\nFound:    {field_names:?}",
    );
}

/// VP-014 (AC-004): `FactoryState::custom_fields` uses `serde_yaml_ng::Value`,
/// NOT `serde_json::Value` (BC-2.02.004 INV-2 secondary constraint).
///
/// Traces to: BC-2.02.004 PC-4; AC-004.
#[test]
fn test_BC_FACTORY_001_factory_state_custom_fields_uses_serde_yaml_ng_not_json() {
    let file = parse_factory_mod_rs();

    let factory_state = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "FactoryState" {
                return Some(s);
            }
        }
        None
    });

    let s = factory_state.expect("FactoryState struct not found in factory/mod.rs");

    let custom_fields_field = s.fields.iter().find(|f| {
        f.ident
            .as_ref()
            .map(|id| id == "custom_fields")
            .unwrap_or(false)
    });

    let field =
        custom_fields_field.expect("FactoryState must have a `custom_fields` field (AC-004)");

    let field_ty_ts = {
        let ty = &field.ty;
        quote::quote!(#ty)
    }
    .to_string();

    assert!(
        field_ty_ts.contains("serde_yaml_ng"),
        "FactoryState::custom_fields MUST use `serde_yaml_ng::Value` — \
         `serde_json::Value` is FORBIDDEN (BC-2.02.004 INV-2, AC-004). \
         Found type: {field_ty_ts:?}"
    );

    assert!(
        !field_ty_ts.contains("serde_json"),
        "FactoryState::custom_fields MUST NOT use `serde_json::Value` \
         (BC-2.02.004 INV-2, AC-004). Found: {field_ty_ts:?}"
    );
}

/// VP-014 (AC-004): `FactoryState::awaiting` is `Option<String>`, not bare `String`.
///
/// `None` means legitimately absent — consumers must NOT substitute `"unknown"` or
/// `"pending"` as a default (BC-2.02.004 PC-4; BC-2.02.005 PC-3).
///
/// Traces to: BC-2.02.004 PC-4; AC-004.
#[test]
fn test_BC_FACTORY_001_factory_state_awaiting_is_option_string() {
    let file = parse_factory_mod_rs();

    let factory_state = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "FactoryState" {
                return Some(s);
            }
        }
        None
    });

    let s = factory_state.expect("FactoryState struct not found in factory/mod.rs");

    let awaiting_field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().map(|id| id == "awaiting").unwrap_or(false));

    let field = awaiting_field.expect("FactoryState must have an `awaiting` field (AC-004)");

    let field_ty_ts = {
        let ty = &field.ty;
        quote::quote!(#ty)
    }
    .to_string();

    assert!(
        field_ty_ts.contains("Option"),
        "FactoryState::awaiting must be Option<String> — None means legitimately absent \
         (BC-2.02.004 PC-4, AC-004). Found type: {field_ty_ts:?}"
    );
    assert!(
        field_ty_ts.contains("String"),
        "FactoryState::awaiting must be Option<String>. Found type: {field_ty_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014: `FactoryDetection` has exactly 3 canonical fields
// (AC-003; traces to BC-2.02.004 postcondition 3)
// ──────────────────────────────────────────────────────────────────────────────

/// VP-014 (AC-003): `FactoryDetection` has exactly 3 fields:
/// `display_name: String`, `workspace_root: PathBuf`, `state_file: PathBuf`.
///
/// Per SS-core-types-and-abi.md lines 337–344. The field names are canonical:
/// NOT `state_file_path` (prohibited name), NOT `framework_version` (does not exist).
///
/// Traces to: BC-2.02.004 PC-3; AC-003.
#[test]
fn test_BC_FACTORY_001_factory_detection_3_fields() {
    let file = parse_factory_mod_rs();

    let factory_detection = file.items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            if s.ident == "FactoryDetection" {
                return Some(s);
            }
        }
        None
    });

    let s = factory_detection.expect("FactoryDetection struct not found in factory/mod.rs");

    let field_names: HashSet<String> = s
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
        .collect();

    let expected: HashSet<String> = ["display_name", "workspace_root", "state_file"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Canonical name guard — the story spec v1.5 F-D-01 fix explicitly corrected
    // these from wrong names (`state_file_path`, `framework_version`).
    assert!(
        !field_names.contains("state_file_path"),
        "FactoryDetection MUST NOT have `state_file_path` field — the canonical name is \
         `state_file` per SS-core-types-and-abi.md lines 337-344 (BC-2.02.004 PC-3, F-D-01)."
    );
    assert!(
        !field_names.contains("framework_version"),
        "FactoryDetection MUST NOT have `framework_version` field — this field does not \
         exist in the canonical 3-field definition (BC-2.02.004 PC-3, F-D-01)."
    );

    assert_eq!(
        field_names.len(),
        3,
        "FactoryDetection must have exactly 3 fields: display_name, workspace_root, state_file. \
         Found {}: {field_names:?}",
        field_names.len(),
    );

    assert_eq!(
        field_names, expected,
        "FactoryDetection field names do not match canonical set.\nExpected: {expected:?}\nFound:    {field_names:?}",
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014: Supporting types are `pub` in `monocle_core::factory`
// (AC-003; traces to BC-2.02.004 postcondition 3)
// ──────────────────────────────────────────────────────────────────────────────

/// VP-014 (AC-003): all required supporting types are `pub` in `monocle_core::factory`.
///
/// Compile-time type probe: if any type does not exist as a pub item, this will not
/// compile.
///
/// Traces to: BC-2.02.004 PC-3; AC-003.
#[test]
fn test_BC_FACTORY_001_supporting_types_pub_in_monocle_core_factory() {
    fn _type_probe() {
        let _: Option<monocle_core::factory::FactoryDetection>;
        let _: Option<monocle_core::factory::FactoryState>;
        let _: Option<monocle_core::factory::BlockingIssue>;
        let _: Option<monocle_core::factory::BlockingSeverity>;
        let _: Option<monocle_core::factory::ConvergenceMetrics>;
        let _: Option<monocle_core::factory::FactoryReadError>;
        let _: Option<monocle_core::factory::FactorySubscribeError>;
        let _: Option<monocle_core::factory::StateChangeStream>;
    }
    assert!(
        true,
        "all required supporting types are pub in monocle_core::factory (compile-time verified)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014: `detect` method has `where Self: Sized` bound
// (AC-001; traces to BC-2.02.004 postcondition 1)
// ──────────────────────────────────────────────────────────────────────────────

/// VP-014 (AC-001): `detect` has `where Self: Sized` bound preventing use on trait objects.
///
/// Traces to: BC-2.02.004 PC-1; AC-001.
#[test]
fn test_BC_FACTORY_001_detect_method_has_where_self_sized() {
    let file = parse_factory_mod_rs();
    let trait_item = find_factory_adapter_trait(&file);

    let detect_method = trait_item.items.iter().find_map(|item| {
        if let TraitItem::Fn(m) = item {
            if m.sig.ident == "detect" {
                return Some(m);
            }
        }
        None
    });

    let method = detect_method.expect("detect method not found in FactoryAdapter trait");

    // The `where Self: Sized` bound appears in the method's where clause.
    // We check the token stream of the full method signature.
    let method_ts = quote::quote!(#method).to_string();

    assert!(
        method_ts.contains("Sized"),
        "FactoryAdapter::detect must have `where Self: Sized` bound \
         (BC-2.02.004 PC-1, AC-001). Not found in method token stream: {method_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014: `abi_version` has a default impl returning `crate::MONOCLE_ABI_VERSION`
// (AC-001; traces to BC-2.02.004 postcondition 1)
// ──────────────────────────────────────────────────────────────────────────────

/// VP-014 (AC-001): `abi_version` has a default implementation body (not abstract).
///
/// The default impl returns `crate::MONOCLE_ABI_VERSION` per BC-2.02.004 PC-1.
///
/// Traces to: BC-2.02.004 PC-1; AC-001.
#[test]
fn test_BC_FACTORY_001_abi_version_has_default_impl() {
    let file = parse_factory_mod_rs();
    let trait_item = find_factory_adapter_trait(&file);

    let abi_method = trait_item.items.iter().find_map(|item| {
        if let TraitItem::Fn(m) = item {
            if m.sig.ident == "abi_version" {
                return Some(m);
            }
        }
        None
    });

    let method = abi_method.expect("abi_version method not found in FactoryAdapter trait");

    assert!(
        method.default.is_some(),
        "FactoryAdapter::abi_version must have a default implementation \
         (BC-2.02.004 PC-1, AC-001). It is currently abstract (no body)."
    );

    // The default impl must reference MONOCLE_ABI_VERSION
    let body_ts = method
        .default
        .as_ref()
        .map(|b| quote::quote!(#b).to_string())
        .unwrap_or_default();

    assert!(
        body_ts.contains("MONOCLE_ABI_VERSION"),
        "FactoryAdapter::abi_version default impl must return `crate::MONOCLE_ABI_VERSION` \
         (BC-2.02.004 PC-1, AC-001). Found body: {body_ts:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014: `FactoryReadError` and `FactorySubscribeError` are `#[non_exhaustive]`
// (AC-003; traces to BC-2.02.004 postcondition 3 + SS-core-types-and-abi §General Rule)
// ──────────────────────────────────────────────────────────────────────────────

/// VP-014 (AC-003): `FactoryReadError` carries `#[non_exhaustive]`.
///
/// Per SS-core-types-and-abi §General Rule: all public enums in monocle-core must carry
/// `#[non_exhaustive]` unless explicitly exempted by ADR.
///
/// Traces to: BC-2.02.004 PC-3; AC-003.
#[test]
fn test_BC_FACTORY_001_factory_read_error_is_non_exhaustive() {
    let file = parse_factory_mod_rs();

    let err_enum = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "FactoryReadError" {
                return Some(e);
            }
        }
        None
    });

    let e = err_enum.expect("FactoryReadError enum not found in factory/mod.rs");

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
        "FactoryReadError must carry #[non_exhaustive] (SS-core-types-and-abi §General Rule, \
         AC-003)"
    );
}

/// VP-014 (AC-003): `FactorySubscribeError` carries `#[non_exhaustive]`.
///
/// Traces to: BC-2.02.004 PC-3; AC-003.
#[test]
fn test_BC_FACTORY_001_factory_subscribe_error_is_non_exhaustive() {
    let file = parse_factory_mod_rs();

    let err_enum = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "FactorySubscribeError" {
                return Some(e);
            }
        }
        None
    });

    let e = err_enum.expect("FactorySubscribeError enum not found in factory/mod.rs");

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
        "FactorySubscribeError must carry #[non_exhaustive] \
         (SS-core-types-and-abi §General Rule, AC-003)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// VP-014: `BlockingSeverity` carries `#[non_exhaustive]`
// (AC-003; traces to BC-2.02.004 postcondition 3 + S-011)
// ──────────────────────────────────────────────────────────────────────────────

/// VP-014 (AC-003): `BlockingSeverity` carries `#[non_exhaustive]` per S-011.
///
/// Previous Story Intelligence: S-011 (Wave 2) applied `#[non_exhaustive]` to
/// `BlockingSeverity` already. This test verifies the attribute is present in the
/// factory module source.
///
/// Traces to: BC-2.02.004 PC-3; AC-003; S-011 Previous Story Intelligence.
#[test]
fn test_BC_FACTORY_001_blocking_severity_is_non_exhaustive() {
    let file = parse_factory_mod_rs();

    let severity_enum = file.items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            if e.ident == "BlockingSeverity" {
                return Some(e);
            }
        }
        None
    });

    let e = severity_enum.expect("BlockingSeverity enum not found in factory/mod.rs");

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
        "BlockingSeverity must carry #[non_exhaustive] (SS-core-types-and-abi §General Rule, \
         S-011 Previous Story Intelligence, AC-003)"
    );
}
