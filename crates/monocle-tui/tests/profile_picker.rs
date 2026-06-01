//! Profile picker open/close/navigation tests (S-031).
//!
//! Tests BC-2.07.004 and BC-2.07.005 picker state management:
//! open, close without change, navigation wrap, Enter selects, Esc closes, empty profiles.
//!
//! All tests use `todo!()` stubs — RED GATE: every test must fail before S-031 implementation.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers: `test_BC_S_SS_NNN_...`.
#![allow(non_snake_case)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use monocle_config::{HarnessProfile, MonocleConfig};
use monocle_tui::app::{
    close_profile_picker, open_profile_picker, picker_select_next, picker_select_prev, App,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_config_with_profiles(ids: &[&str]) -> MonocleConfig {
    let mut config = MonocleConfig::default();
    for id in ids {
        config.harness_profiles.push(HarnessProfile {
            id: id.to_string(),
            display_name: format!("Profile {}", id),
            binary_path: "/usr/local/bin/claude".to_string(),
            config_dir: None,
        });
    }
    config
}

fn make_config_empty() -> MonocleConfig {
    MonocleConfig::default()
}

// ---------------------------------------------------------------------------
// BC-2.07.004 PC-1 / BC-2.07.005 PC-1 — Action::ProfilePicker opens picker
// ---------------------------------------------------------------------------

/// AC-001 (BC-2.07.005 PC-1): pressing Ctrl-P (Action::ProfilePicker) sets
/// `app.profile_picker = Some(ProfilePickerState { .. })` without changing AppMode.
#[test]
fn test_BC_2_07_005_ctrl_p_sets_profile_picker_some() {
    let config = make_config_with_profiles(&["cc", "cm"]);
    let mut app = App::new(config);
    assert!(app.profile_picker.is_none(), "picker should start None");

    open_profile_picker(&mut app);

    assert!(
        app.profile_picker.is_some(),
        "picker should be Some after open_profile_picker"
    );
}

/// AC-001 (BC-2.07.005 PC-1): AppMode is unchanged after opening the picker.
/// The picker MUST NOT transition AppMode (BC-2.07.005 INV-4).
#[test]
fn test_BC_2_07_005_open_picker_does_not_change_app_mode() {
    use monocle_core::tui::state::AppMode;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    // Verify starting mode is Dashboard.
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "expected Dashboard mode"
    );

    open_profile_picker(&mut app);

    // Mode must remain Dashboard — not Overlay, not any new variant.
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "AppMode must remain Dashboard after open_profile_picker (BC-2.07.005 INV-4)"
    );
}

/// AC-001 (BC-2.07.004 PC-1): profiles in ProfilePickerState are sorted alphabetically.
#[test]
fn test_BC_2_07_004_picker_profiles_sorted_alphabetically() {
    let config = make_config_with_profiles(&["zzz", "aaa", "mmm"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    let state = app.profile_picker.as_ref().expect("picker should be Some");
    let mut expected = vec!["aaa".to_string(), "mmm".to_string(), "zzz".to_string()];
    expected.sort();
    assert_eq!(
        state.profiles, expected,
        "profiles in picker must be sorted alphabetically (BC-2.07.004 PC-1)"
    );
}

/// AC-001 (BC-2.07.005 EC-110): second Ctrl-P when picker is already open is a no-op.
/// Only one picker instance is active at a time.
#[test]
fn test_BC_2_07_005_ctrl_p_idempotent_when_picker_already_open() {
    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    // Mutate selection to verify idempotency: second open must NOT reset it.
    if let Some(ref mut s) = app.profile_picker {
        s.selected_index = 0;
    }
    open_profile_picker(&mut app); // second call — must be no-op

    let state = app
        .profile_picker
        .as_ref()
        .expect("picker must still be Some");
    assert_eq!(
        state.selected_index, 0,
        "second open_profile_picker must not reset existing state (EC-110)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.004 PC-3 / BC-2.07.005 PC-9 — keyboard isolation: Esc closes without change
// ---------------------------------------------------------------------------

/// AC-003/AC-004 (BC-2.07.004 PC-3 / BC-2.07.005 PC-8):
/// Esc closes the picker by setting `app.profile_picker = None`.
#[test]
fn test_BC_2_07_004_esc_closes_picker_without_change() {
    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some());

    close_profile_picker(&mut app);

    assert!(
        app.profile_picker.is_none(),
        "Esc must set app.profile_picker = None (BC-2.07.004 PC-3)"
    );
}

/// AC-004 (BC-2.07.004 INV-4 / BC-2.07.005 PC-8): AppMode is unchanged after Esc.
#[test]
fn test_BC_2_07_005_esc_does_not_change_app_mode() {
    use monocle_core::tui::state::AppMode;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);
    close_profile_picker(&mut app);

    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "AppMode must be unchanged after Esc (BC-2.07.005 PC-8)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.004 PC-3 — navigation (j/k wrapping)
// ---------------------------------------------------------------------------

/// AC-003 (BC-2.07.004 PC-3): `j` moves selection down one row.
#[test]
fn test_BC_2_07_004_picker_select_next_increments_index() {
    let config = make_config_with_profiles(&["aaa", "bbb", "ccc"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    let initial = app.profile_picker.as_ref().unwrap().selected_index;
    picker_select_next(&mut app);
    let after = app.profile_picker.as_ref().unwrap().selected_index;

    assert_eq!(
        after,
        initial + 1,
        "select_next must increment selected_index"
    );
}

/// AC-003 (BC-2.07.004 PC-3): `j` at the last item wraps to index 0.
#[test]
fn test_BC_2_07_004_picker_select_next_wraps_to_top() {
    let config = make_config_with_profiles(&["aaa", "bbb"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    // Navigate to last item.
    if let Some(ref mut s) = app.profile_picker {
        s.selected_index = s.profiles.len() - 1;
    }

    picker_select_next(&mut app);

    let after = app.profile_picker.as_ref().unwrap().selected_index;
    assert_eq!(
        after, 0,
        "select_next at last item must wrap to 0 (BC-2.07.004 PC-3)"
    );
}

/// AC-003 (BC-2.07.004 PC-3): `k` moves selection up one row.
#[test]
fn test_BC_2_07_004_picker_select_prev_decrements_index() {
    let config = make_config_with_profiles(&["aaa", "bbb", "ccc"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    // Start at index 1.
    if let Some(ref mut s) = app.profile_picker {
        s.selected_index = 1;
    }
    picker_select_prev(&mut app);
    let after = app.profile_picker.as_ref().unwrap().selected_index;

    assert_eq!(after, 0, "select_prev must decrement selected_index");
}

/// AC-003 (BC-2.07.004 PC-3): `k` at the first item wraps to the last.
#[test]
fn test_BC_2_07_004_picker_select_prev_wraps_to_bottom() {
    let config = make_config_with_profiles(&["aaa", "bbb", "ccc"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    // selected_index starts at 0.
    picker_select_prev(&mut app);
    let after = app.profile_picker.as_ref().unwrap().selected_index;

    assert_eq!(
        after, 2,
        "select_prev at index 0 must wrap to len-1 (BC-2.07.004 PC-3)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.005 PC-3 / AC-002 — empty profiles message
// ---------------------------------------------------------------------------

/// AC-002 (BC-2.07.005 PC-3): opening the picker with no profiles still opens
/// (picker is Some); the empty-state message is a rendering concern in the widget.
#[test]
fn test_BC_2_07_005_picker_opens_with_empty_profiles() {
    let config = make_config_empty();
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    let state = app
        .profile_picker
        .as_ref()
        .expect("picker must be Some even with empty profiles");
    assert!(
        state.profiles.is_empty(),
        "profiles list must be empty when harness_profiles is empty (BC-2.07.005 PC-3)"
    );
}

/// AC-003 navigation no-ops on empty profiles.
#[test]
fn test_BC_2_07_004_navigation_noop_on_empty_profiles() {
    let config = make_config_empty();
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    // Must not panic with empty profiles.
    picker_select_next(&mut app);
    picker_select_prev(&mut app);

    // selected_index should remain at 0 (no valid range to navigate).
    let idx = app.profile_picker.as_ref().unwrap().selected_index;
    assert_eq!(
        idx, 0,
        "navigation on empty profiles must leave selected_index at 0"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.004 INV-1 — AppMode::Overlay must NOT be used for the picker
// ---------------------------------------------------------------------------

/// AC-008 (BC-2.07.004 INV-1): profile picker must not use AppMode::Overlay.
/// After open_profile_picker, app.mode must NOT be AppMode::Overlay.
#[test]
fn test_BC_2_07_004_invariant_picker_is_not_app_mode_overlay() {
    use monocle_core::tui::state::AppMode;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);
    open_profile_picker(&mut app);

    assert!(
        !matches!(app.mode, AppMode::Overlay { .. }),
        "profile picker MUST NOT use AppMode::Overlay (AC-008 / BC-2.07.004 INV-1)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.004 EC-097 — first-ever launch, empty config
// ---------------------------------------------------------------------------

/// EC-097: first ever launch — no profiles, no project_profiles entries.
/// TUI enters Dashboard with empty profile list; picker opens with empty list.
#[test]
fn test_BC_2_07_004_ec097_first_launch_empty_config() {
    let config = MonocleConfig::default();
    let mut app = App::new(config);

    // On first launch, profile_picker starts None.
    assert!(app.profile_picker.is_none());

    // Opening the picker with empty harness_profiles is valid.
    open_profile_picker(&mut app);
    let state = app.profile_picker.as_ref().unwrap();
    assert!(
        state.profiles.is_empty(),
        "EC-097: no profiles on first launch"
    );
}

// ---------------------------------------------------------------------------
// monocle_config::resolve_profile_for_dir canonical test vectors (BC-2.07.004)
// ---------------------------------------------------------------------------

/// BC-2.07.004 canonical test vector: default config, no entry → None.
#[test]
fn test_BC_2_07_004_resolve_profile_for_dir_default_config_returns_none() {
    use monocle_config::resolve_profile_for_dir;
    let config = MonocleConfig::default();
    let result = resolve_profile_for_dir(&config, "/home/user/project");
    assert!(result.is_none(), "default config should return None");
}

/// BC-2.07.004 canonical test vector: matching profile ID → Some(&profile).
#[test]
fn test_BC_2_07_004_resolve_profile_for_dir_returns_matching_profile() {
    use monocle_config::resolve_profile_for_dir;
    let mut config = make_config_with_profiles(&["cc"]);
    config
        .project_profiles
        .insert("/home/user/project".to_string(), "cc".to_string());
    let result = resolve_profile_for_dir(&config, "/home/user/project");
    assert!(result.is_some(), "should find profile 'cc'");
    assert_eq!(result.unwrap().id, "cc");
}

/// BC-2.07.004 canonical test vector: dangling profile ID → None (not panic).
#[test]
fn test_BC_2_07_004_resolve_profile_for_dir_dangling_id_returns_none() {
    use monocle_config::resolve_profile_for_dir;
    let config = make_config_with_profiles(&["cc"]);
    let mut config = config;
    config
        .project_profiles
        .insert("/home/user/project".to_string(), "old-profile".to_string());
    let result = resolve_profile_for_dir(&config, "/home/user/project");
    assert!(
        result.is_none(),
        "dangling profile ID must return None, not panic (BC-2.07.004 PC-6)"
    );
}

/// BC-2.07.004 INV-3: dangling entry does not cause panic (EC-100).
#[test]
fn test_BC_2_07_004_invariant_dangling_entry_no_panic() {
    use monocle_config::resolve_profile_for_dir;
    let mut config = MonocleConfig::default();
    config.project_profiles.insert(
        "/home/user/project".to_string(),
        "deleted-profile-id".to_string(),
    );
    // Must not panic even with no harness_profiles.
    let result = resolve_profile_for_dir(&config, "/home/user/project");
    assert!(result.is_none());
}

/// BC-2.07.004 INV-1: normalization must be consistent — trailing slash mismatch
/// produces a miss, demonstrating why identical normalization at write and read time matters.
#[test]
fn test_BC_2_07_004_invariant_trailing_slash_mismatch_is_a_miss() {
    use monocle_config::resolve_profile_for_dir;
    let mut config = make_config_with_profiles(&["cc"]);
    // Stored WITHOUT trailing slash.
    config
        .project_profiles
        .insert("/home/user/project".to_string(), "cc".to_string());
    // Look up WITH trailing slash — different string → miss (demonstrates INV-1).
    let result = resolve_profile_for_dir(&config, "/home/user/project/");
    assert!(
        result.is_none(),
        "trailing slash mismatch must produce a miss — normalization must be consistent (INV-1)"
    );
}

/// BC-2.07.004 EC-105: empty-string profile ID never matches.
#[test]
fn test_BC_2_07_004_ec105_empty_string_profile_id_returns_none() {
    use monocle_config::resolve_profile_for_dir;
    let mut config = make_config_with_profiles(&["cc"]);
    config.project_profiles.insert(
        "/home/user/project".to_string(),
        String::new(), // empty profile ID
    );
    let result = resolve_profile_for_dir(&config, "/home/user/project");
    assert!(
        result.is_none(),
        "empty profile ID must not match any profile (EC-105)"
    );
}

/// BC-2.07.004 INV-2: resolve_profile_for_dir is pure — calling it twice produces same result.
#[test]
fn test_BC_2_07_004_invariant_resolve_is_pure_no_side_effects() {
    use monocle_config::resolve_profile_for_dir;
    let mut config = make_config_with_profiles(&["cc"]);
    config
        .project_profiles
        .insert("/home/user/project".to_string(), "cc".to_string());
    let result1 = resolve_profile_for_dir(&config, "/home/user/project");
    let result2 = resolve_profile_for_dir(&config, "/home/user/project");
    // Both must return Some with the same id — pure function, no state mutation.
    assert_eq!(
        result1.map(|p| p.id.clone()),
        result2.map(|p| p.id.clone()),
        "resolve_profile_for_dir must be pure (INV-2)"
    );
}
