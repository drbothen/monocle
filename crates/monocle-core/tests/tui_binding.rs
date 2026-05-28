#![allow(non_snake_case)]
#![allow(clippy::expect_used)]
//! S-024: Failing tests for `resolve_binding()` — 5-level precedence dispatch.
//!
//! Covers BC-2.06.003 preconditions and postconditions for the binding resolver.
//! Tests are named `test_BC_S_SS_NNN_xxx` for full traceability.
//!
//! All calls into `resolve_binding()` use `catch_unwind` because the stub contains
//! `todo!()`. The Red Gate requirement is met: every test fails with a clear panic
//! message until the implementation is written.
#![allow(clippy::panic)]

use monocle_core::tui::binding::{BindingLayers, BindingSource, KeyCode, KeyEvent, KeyModifiers};
use monocle_core::tui::state::{AppMode, FocusSnapshot, PromptModal, ToolPayload};

use std::panic::AssertUnwindSafe;
use std::time::Instant;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn no_modifiers() -> KeyModifiers {
    KeyModifiers {
        shift: false,
        ctrl: false,
        alt: false,
    }
}

fn ctrl_modifiers() -> KeyModifiers {
    KeyModifiers {
        shift: false,
        ctrl: true,
        alt: false,
    }
}

fn char_key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: no_modifiers(),
    }
}

fn ctrl_key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: ctrl_modifiers(),
    }
}

fn special_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: no_modifiers(),
    }
}

fn dashboard_sessions() -> AppMode {
    AppMode::Dashboard {
        focused: FocusSnapshot::Sessions,
    }
}

fn filtering_mode() -> AppMode {
    AppMode::Filtering {
        panel: monocle_core::tui::state::PanelId::Sessions,
        query: String::new(),
        prior: FocusSnapshot::Sessions,
    }
}

fn make_test_modal() -> PromptModal {
    PromptModal {
        prompt_id: Uuid::new_v4(),
        session_id: "test-session".to_string(),
        tool_name: "Bash".to_string(),
        tool_payload: ToolPayload::Bash {
            command: "echo test".to_string(),
        },
        received_at: Instant::now(),
    }
}

fn overlay_mode() -> AppMode {
    // F-S025-ADV2-HIGH-003: AppMode::Overlay no longer stores the stack.
    // The prior field is sufficient to construct the Overlay mode sentinel.
    AppMode::Overlay {
        prior: FocusSnapshot::Sessions,
    }
}

/// Call `resolve_binding()` inside `catch_unwind`, converting a `todo!()` panic into an
/// explicit assertion failure. Returns `Option<(Action, BindingSource)>` or panics with a
/// Red Gate message.
macro_rules! call_resolve {
    ($key:expr, $mode:expr, $layers:expr) => {{
        let key = $key;
        let mode = $mode;
        let layers = $layers;
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            monocle_core::tui::binding::resolve_binding(&key, &mode, &layers)
        }));
        result.unwrap_or_else(|_| {
            panic!(
                "resolve_binding() panicked (likely todo!()) — Red Gate: implementation not yet written"
            )
        })
    }};
}

// ===========================================================================
// AC-010 / BC-2.06.003 Pre-1: BindingSource enum variants exist
// ===========================================================================

/// BC-2.06.003 Pre-1 / AC-010: All 5 BindingSource variants exist and can be constructed.
#[test]
fn test_BC_2_06_003_ac010_binding_source_all_variants_exist() {
    // Construct every variant — if any is missing, this will fail to compile.
    let sources: Vec<BindingSource> = vec![
        BindingSource::SearchPrompt,
        BindingSource::UserCustomCommand,
        BindingSource::PerContext,
        BindingSource::Global,
        BindingSource::Builtin,
    ];
    assert_eq!(
        sources.len(),
        5,
        "There must be exactly 5 BindingSource variants"
    );
}

/// BC-2.06.003 Pre-1 / AC-010: BindingSource derives Clone, PartialEq, Eq, Debug.
#[test]
fn test_BC_2_06_003_ac010_binding_source_derives() {
    let a = BindingSource::SearchPrompt;
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(BindingSource::SearchPrompt, BindingSource::Builtin);
    let _ = format!("{:?}", BindingSource::Global);
}

/// BC-2.06.003 INV-1 / AC-010: Priority ordering — SearchPrompt is highest priority.
/// This is verified implicitly by the resolve_binding tests, but we assert the conceptual
/// ordering through BindingSource variant identity.
#[test]
fn test_BC_2_06_003_ac010_binding_source_priority_identity() {
    // Verify each variant is distinct (no accidental equality)
    assert_ne!(
        BindingSource::SearchPrompt,
        BindingSource::UserCustomCommand
    );
    assert_ne!(BindingSource::UserCustomCommand, BindingSource::PerContext);
    assert_ne!(BindingSource::PerContext, BindingSource::Global);
    assert_ne!(BindingSource::Global, BindingSource::Builtin);
}

// ===========================================================================
// AC-011 / BC-2.06.003 PC-4: resolve_binding returns None for unregistered key
// ===========================================================================

/// BC-2.06.003 PC-4 / AC-011: Unregistered key with empty layers returns None.
#[test]
fn test_BC_2_06_003_ac011_resolve_binding_none_on_unregistered_key_empty_layers() {
    let key = char_key('z');
    let layers = BindingLayers::empty();
    let mode = dashboard_sessions();

    let result = call_resolve!(key, mode, layers);
    assert!(
        result.is_none(),
        "resolve_binding with empty layers must return None for any key (BC-2.06.003 PC-4)"
    );
}

/// BC-2.06.003 PC-4 / AC-011: Unknown key (e.g., not a printable char, not special) returns None.
/// Uses an F12-like scenario: a key not present in any default binding table.
#[test]
fn test_BC_2_06_003_ac011_resolve_binding_none_on_unknown_key() {
    // Up arrow is unlikely to be in any default binding in the empty-layers case
    let key = special_key(KeyCode::Up);
    let layers = BindingLayers::empty();
    let mode = dashboard_sessions();

    let result = call_resolve!(key, mode, layers);
    assert!(
        result.is_none(),
        "Unknown key with empty layers must return None (BC-2.06.003 PC-4)"
    );
}

/// BC-2.06.003 PC-4 / AC-011: No-match keypress in Dashboard mode returns None.
/// Tests the EC-070 case: Char('y') in Dashboard has no match.
#[test]
fn test_BC_2_06_003_ac011_ec070_char_y_in_dashboard_returns_none() {
    let key = char_key('y');
    let layers = BindingLayers::empty();
    let mode = dashboard_sessions();

    let result = call_resolve!(key, mode, layers);
    assert!(
        result.is_none(),
        "Char('y') in Dashboard must return None — no overlay bindings in dashboard mode (EC-070)"
    );
}

// ===========================================================================
// AC-012 / BC-2.06.003 PC-2: SearchPrompt captures printable keys in Filtering mode
// ===========================================================================

/// BC-2.06.003 PC-2 / AC-012: Printable char in Filtering mode resolves via SearchPrompt.
#[test]
fn test_BC_2_06_003_ac012_printable_char_in_filtering_resolves_search_prompt() {
    let key = char_key('a');
    let layers = BindingLayers::empty();
    let mode = filtering_mode();

    let result = call_resolve!(key, mode, layers);
    let (action, source) = result.expect(
        "Printable char in Filtering mode must NOT return None — SearchPrompt must capture it",
    );

    assert_eq!(
        source,
        BindingSource::SearchPrompt,
        "Printable char in Filtering mode must resolve from SearchPrompt layer (AC-012)"
    );
    // Verify it maps to FilterType('a')
    match action {
        monocle_core::tui::state::Action::FilterType(c) => {
            assert_eq!(c, 'a', "Printable char 'a' must map to FilterType('a')");
        }
        _ => panic!("Printable char in Filtering must map to Action::FilterType(char)"),
    }
}

/// BC-2.06.003 PC-2 / AC-012: Multiple printable chars in Filtering mode all resolve via SearchPrompt.
#[test]
fn test_BC_2_06_003_ac012_multiple_printable_chars_in_filtering_resolve_search_prompt() {
    let printable_chars = ['a', 'z', 'A', 'Z', '0', '9', ' ', '_', '-'];

    for &c in &printable_chars {
        let key = char_key(c);
        let mode = filtering_mode();
        // BindingLayers does not impl Copy, so reconstruct each iteration.
        let layers = BindingLayers::empty();

        let result = call_resolve!(key, mode, layers);
        let (action, source) = result.unwrap_or_else(|| {
            panic!(
                "Printable char '{c}' in Filtering must resolve (not None) — SearchPrompt captures all printable chars"
            )
        });

        assert_eq!(
            source,
            BindingSource::SearchPrompt,
            "Char '{c}' in Filtering must resolve from SearchPrompt"
        );
        match action {
            monocle_core::tui::state::Action::FilterType(resolved_c) => {
                assert_eq!(resolved_c, c, "FilterType char must match the input char");
            }
            _ => panic!("Printable char '{c}' in Filtering must map to FilterType"),
        }
    }
}

/// BC-2.06.003 EC-075 / AC-012: Ctrl-modified key in Filtering falls through SearchPrompt.
/// Ctrl-P is NOT a printable char and must NOT be captured by the SearchPrompt layer.
#[test]
fn test_BC_2_06_003_ac012_ec075_ctrl_key_in_filtering_falls_through_search_prompt() {
    let key = ctrl_key('p');
    let layers = BindingLayers::empty();
    let mode = filtering_mode();

    let result = call_resolve!(key, mode, layers);
    // With empty layers, Ctrl-P has no binding at any level → None.
    // The important thing is it was NOT captured by SearchPrompt as FilterType.
    // If it returns Some, it must NOT be from SearchPrompt as FilterType.
    if let Some((action, source)) = result {
        // If some layer has a Ctrl-P binding (e.g., builtin), it must NOT be SearchPrompt/FilterType
        let is_search_prompt_filter_type = matches!(source, BindingSource::SearchPrompt)
            && matches!(action, monocle_core::tui::state::Action::FilterType(_));
        assert!(
            !is_search_prompt_filter_type,
            "Ctrl-P must NOT be captured by SearchPrompt as FilterType (EC-075)"
        );
    }
    // If None, that's correct — Ctrl-P has no binding in empty layers.
}

// ===========================================================================
// BC-2.06.003 PC-3: Overlay mode SearchPrompt captures permission decision keys
// ===========================================================================

/// BC-2.06.003 PC-3 / EC-071: Char('y') in Overlay mode resolves to PermissionAcceptOnce via SearchPrompt.
#[test]
fn test_BC_2_06_003_pc3_char_y_in_overlay_resolves_permission_accept_once() {
    let key = char_key('y');
    let layers = BindingLayers::empty();
    let mode = overlay_mode();

    let result = call_resolve!(key, mode, layers);
    let (action, source) = result.expect(
        "Char('y') in Overlay must NOT return None — SearchPrompt must capture it as PermissionAcceptOnce"
    );

    assert_eq!(
        source,
        BindingSource::SearchPrompt,
        "Char('y') in Overlay must resolve from SearchPrompt (BC-2.06.003 PC-3)"
    );
    match action {
        monocle_core::tui::state::Action::PermissionAcceptOnce => {}
        _ => panic!("Char('y') in Overlay must map to PermissionAcceptOnce"),
    }
}

/// BC-2.06.003 PC-3 / EC-071: Enter in Overlay mode resolves to PermissionAcceptOnce via SearchPrompt.
#[test]
fn test_BC_2_06_003_pc3_enter_in_overlay_resolves_permission_accept_once() {
    let key = special_key(KeyCode::Enter);
    let layers = BindingLayers::empty();
    let mode = overlay_mode();

    let result = call_resolve!(key, mode, layers);
    let (action, source) = result.expect(
        "Enter in Overlay must NOT return None — SearchPrompt must capture it as PermissionAcceptOnce"
    );

    assert_eq!(
        source,
        BindingSource::SearchPrompt,
        "Enter in Overlay must resolve from SearchPrompt (BC-2.06.003 PC-3)"
    );
    match action {
        monocle_core::tui::state::Action::PermissionAcceptOnce => {}
        _ => panic!("Enter in Overlay must map to PermissionAcceptOnce"),
    }
}

/// BC-2.06.003 PC-3: Char('A') (shift+a) in Overlay mode resolves to PermissionAcceptAlways.
#[test]
fn test_BC_2_06_003_pc3_char_a_upper_in_overlay_resolves_permission_accept_always() {
    // 'A' is a Char key — the BC specifies KeyCode::Char('A') with no explicit Shift modifier
    // (the shift is encoded in the character itself as uppercase).
    let key = char_key('A');
    let layers = BindingLayers::empty();
    let mode = overlay_mode();

    let result = call_resolve!(key, mode, layers);
    let (action, source) = result.expect(
        "Char('A') in Overlay must NOT return None — SearchPrompt must capture it as PermissionAcceptAlways"
    );

    assert_eq!(
        source,
        BindingSource::SearchPrompt,
        "Char('A') in Overlay must resolve from SearchPrompt (BC-2.06.003 PC-3)"
    );
    match action {
        monocle_core::tui::state::Action::PermissionAcceptAlways => {}
        _ => panic!("Char('A') in Overlay must map to PermissionAcceptAlways"),
    }
}

/// BC-2.06.003 PC-3: Char('n') in Overlay mode resolves to PermissionReject.
#[test]
fn test_BC_2_06_003_pc3_char_n_in_overlay_resolves_permission_reject() {
    let key = char_key('n');
    let layers = BindingLayers::empty();
    let mode = overlay_mode();

    let result = call_resolve!(key, mode, layers);
    let (action, source) = result.expect(
        "Char('n') in Overlay must NOT return None — SearchPrompt must capture it as PermissionReject"
    );

    assert_eq!(
        source,
        BindingSource::SearchPrompt,
        "Char('n') in Overlay must resolve from SearchPrompt (BC-2.06.003 PC-3)"
    );
    match action {
        monocle_core::tui::state::Action::PermissionReject => {}
        _ => panic!("Char('n') in Overlay must map to PermissionReject"),
    }
}

/// BC-2.06.003 PC-3: Char('r') in Overlay mode resolves to PermissionReject.
#[test]
fn test_BC_2_06_003_pc3_char_r_in_overlay_resolves_permission_reject() {
    let key = char_key('r');
    let layers = BindingLayers::empty();
    let mode = overlay_mode();

    let result = call_resolve!(key, mode, layers);
    let (action, source) = result.expect(
        "Char('r') in Overlay must NOT return None — SearchPrompt must capture it as PermissionReject"
    );

    assert_eq!(
        source,
        BindingSource::SearchPrompt,
        "Char('r') in Overlay must resolve from SearchPrompt (BC-2.06.003 PC-3)"
    );
    match action {
        monocle_core::tui::state::Action::PermissionReject => {}
        _ => panic!("Char('r') in Overlay must map to PermissionReject"),
    }
}

/// BC-2.06.003 EC-070: Char('y') in Dashboard (not Overlay) returns None.
/// Permission decision bindings are NOT active outside Overlay mode.
#[test]
fn test_BC_2_06_003_ec070_char_y_in_dashboard_no_permission_binding() {
    let key = char_key('y');
    let layers = BindingLayers::empty();
    let mode = dashboard_sessions();

    let result = call_resolve!(key, mode, layers);
    assert!(
        result.is_none(),
        "Char('y') in Dashboard must return None — permission bindings only active in Overlay (EC-070)"
    );
}

// ===========================================================================
// BC-2.06.003 PC-1: First-match-wins priority order
// ===========================================================================

/// BC-2.06.003 PC-1: resolve_binding returns None with empty layers for a non-special key.
/// This validates that the resolver correctly exhausts all 5 layers when none match.
#[test]
fn test_BC_2_06_003_pc1_first_match_wins_empty_layers_returns_none() {
    let key = char_key('x');
    let layers = BindingLayers::empty();
    let mode = dashboard_sessions();

    let result = call_resolve!(key, mode, layers);
    assert!(
        result.is_none(),
        "Empty layers must return None for any non-captured key (all 5 layers exhausted)"
    );
}

// ===========================================================================
// BC-2.06.003 PC-5: resolve_binding is deterministic
// ===========================================================================

/// BC-2.06.003 PC-5: resolve_binding is deterministic — same (key, mode, layers) → same output.
#[test]
fn test_BC_2_06_003_pc5_resolve_binding_is_deterministic() {
    let key = char_key('a');
    let layers = BindingLayers::empty();

    let result1 = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::binding::resolve_binding(&key, &filtering_mode(), &layers)
    }));
    let result2 = std::panic::catch_unwind(AssertUnwindSafe(|| {
        monocle_core::tui::binding::resolve_binding(&key, &filtering_mode(), &layers)
    }));

    let out1 = result1.unwrap_or_else(|_| panic!("First call panicked — Red Gate"));
    let out2 = result2.unwrap_or_else(|_| panic!("Second call panicked — Red Gate"));

    // Both must agree on whether they return Some or None
    assert_eq!(
        out1.is_some(),
        out2.is_some(),
        "resolve_binding must be deterministic: two identical calls must both return Some or both None"
    );

    // If both returned Some, the sources must match
    if let (Some((_, source1)), Some((_, source2))) = (out1, out2) {
        assert_eq!(
            source1, source2,
            "Determinism: both calls must return the same BindingSource"
        );
    }
}

// ===========================================================================
// KeyEvent and KeyModifiers type checks
// ===========================================================================

/// AC-012 support: KeyEvent constructs with Char code and no modifiers.
#[test]
fn test_BC_2_06_003_key_event_constructs_correctly() {
    let key = KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::default(),
    };
    assert_eq!(key.code, KeyCode::Char('a'));
    assert!(!key.modifiers.ctrl);
    assert!(!key.modifiers.shift);
    assert!(!key.modifiers.alt);
}

/// KeyModifiers::default() produces all-false modifiers.
#[test]
fn test_BC_2_06_003_key_modifiers_default_is_no_modifiers() {
    let mods = KeyModifiers::default();
    assert!(!mods.ctrl);
    assert!(!mods.shift);
    assert!(!mods.alt);
}

/// KeyEvent derives Clone, PartialEq, Eq, Hash, Debug.
#[test]
fn test_BC_2_06_003_key_event_derives() {
    let a = char_key('a');
    let b = a.clone();
    assert_eq!(a, b);
    let _ = format!("{:?}", a);

    // Hash: can be used as HashMap key
    let mut map = std::collections::HashMap::new();
    map.insert(a.clone(), "action");
    assert_eq!(map.get(&a), Some(&"action"));
}

// ===========================================================================
// Modifier guard: Ctrl/Alt modified keys must not fire overlay permission actions
// ===========================================================================

/// Regression guard: Ctrl+Y in Overlay mode must NOT resolve to PermissionAcceptOnce.
///
/// Without the modifier guard, `KeyCode::Char('y')` with `ctrl: true` would match
/// the `'y'` arm and fire `PermissionAcceptOnce`. This is incorrect — the user
/// pressed Ctrl+Y, not plain 'y'.
///
/// Traces to: PR #20 review finding (MEDIUM); binding.rs overlay modifier guard.
#[test]
fn test_ctrl_y_in_overlay_does_not_resolve_accept() {
    let key = ctrl_key('y');
    let layers = BindingLayers::empty();
    let mode = overlay_mode();

    let result = call_resolve!(key, mode, layers);
    // Ctrl+Y must NOT resolve to PermissionAcceptOnce or any permission action.
    // With empty layers and the modifier guard, it must fall through to None.
    if let Some((action, _source)) = result {
        match action {
            monocle_core::tui::state::Action::PermissionAcceptOnce => {
                panic!(
                    "Ctrl+Y in Overlay must NOT resolve to PermissionAcceptOnce \
                     — modifier guard must filter Ctrl-modified keys"
                )
            }
            monocle_core::tui::state::Action::PermissionAcceptAlways => {
                panic!(
                    "Ctrl+Y in Overlay must NOT resolve to PermissionAcceptAlways \
                     — modifier guard must filter Ctrl-modified keys"
                )
            }
            monocle_core::tui::state::Action::PermissionReject => {
                panic!(
                    "Ctrl+Y in Overlay must NOT resolve to PermissionReject \
                     — modifier guard must filter Ctrl-modified keys"
                )
            }
            _ => {
                // Some other action from a lower layer is acceptable.
            }
        }
    }
    // None is the expected result with empty layers and the modifier guard active.
}

/// Regression guard: Alt+N in Overlay mode must NOT resolve to PermissionReject.
///
/// Mirror of the Ctrl+Y test for Alt modifier on 'n'.
///
/// Traces to: PR #20 review finding (MEDIUM); binding.rs overlay modifier guard.
#[test]
fn test_alt_n_in_overlay_does_not_resolve_reject() {
    let key = KeyEvent {
        code: KeyCode::Char('n'),
        modifiers: KeyModifiers {
            shift: false,
            ctrl: false,
            alt: true,
        },
    };
    let layers = BindingLayers::empty();
    let mode = overlay_mode();

    let result = call_resolve!(key, mode, layers);
    if let Some((monocle_core::tui::state::Action::PermissionReject, _source)) = result {
        panic!(
            "Alt+N in Overlay must NOT resolve to PermissionReject \
             — modifier guard must filter Alt-modified keys"
        );
    }
}

/// Sanity check: plain 'y' (no modifiers) in Overlay must still work after the modifier guard.
///
/// The modifier guard must not break the normal permission-accept path.
///
/// Traces to: PR #20 review finding (MEDIUM) — regression guard for the fix.
#[test]
fn test_plain_y_in_overlay_still_resolves_accept_after_modifier_guard() {
    let key = char_key('y');
    let layers = BindingLayers::empty();
    let mode = overlay_mode();

    let result = call_resolve!(key, mode, layers);
    let (action, source) = result.expect(
        "Plain 'y' (no modifiers) in Overlay must still resolve to PermissionAcceptOnce \
         after the modifier guard is applied",
    );
    assert_eq!(
        source,
        BindingSource::SearchPrompt,
        "Plain 'y' in Overlay must still resolve from SearchPrompt after modifier guard"
    );
    match action {
        monocle_core::tui::state::Action::PermissionAcceptOnce => {}
        _ => panic!("Plain 'y' in Overlay must still map to PermissionAcceptOnce"),
    }
}

// ===========================================================================
// BindingLayers::empty() returns a valid (non-panicking) value
// ===========================================================================

/// AC-011: BindingLayers::empty() constructs without panicking.
#[test]
fn test_BC_2_06_003_binding_layers_empty_constructs() {
    let _layers = BindingLayers::empty();
    // No assertion needed — if construction panics, the test fails.
}
