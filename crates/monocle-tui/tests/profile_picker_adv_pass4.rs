//! ADV Pass-4 regression test for S-031 profile picker — MAJOR-1: wrapper empty-CWD guard.
//!
//! # Finding: MAJOR-1 — Wrapper Err-branch missing empty-CWD guard (FIXED)
//!
//! `commit_profile_selection` (the WRAPPER, app.rs ~1123-1164) previously called
//! `MonocleConfig::config_path()` before checking whether `current_dir` is empty.
//! Its `Err` branch lacked the empty-CWD guard and would insert
//! `project_profiles[""] = id` — silent config-corruption (BC-2.07.005 PC-5 /
//! INV-5 normalization contract).
//!
//! # Fix (c929848)
//!
//! The guard was hoisted to the TOP of `commit_profile_selection`, BEFORE the
//! `config_path()` call:
//!
//!   ```text
//!   if current_dir.is_empty() {
//!       app.status_message = Some("Config save failed: CWD resolution failed".to_string());
//!       app.profile_picker = None;
//!       return;
//!   }
//!   ```
//!
//! Both the `Ok` and `Err` branches of `config_path()` are now protected by a single
//! guard — the function returns before reaching either branch when `current_dir == ""`.
//!
//! # Test strategy
//!
//! The test calls the WRAPPER `commit_profile_selection(app, "")` with an empty
//! `current_dir`.  HOME is temporarily unset via `temp_env::with_vars` to probe the
//! `Err` branch of `config_path()` (reachable on Linux; masked by `getpwuid_r` fallback
//! on macOS).  In both cases the hoisted guard fires first, so:
//!
//! - `project_profiles` must NOT contain the `""` key.
//! - The picker must be closed (`profile_picker == None`).
//! - `status_message` must be set to a non-empty string (error surfaced to user).
//!
//! # Naming convention
//!
//! All test names use the `test_BC_S_SS_NNN_...` pattern per TDD spec (DF-021).
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers: `test_BC_S_SS_NNN_...`.
#![allow(non_snake_case)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use monocle_config::{HarnessProfile, MonocleConfig};
use monocle_tui::app::{commit_profile_selection, open_profile_picker, App};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_app_with_profile(id: &str) -> App {
    let mut config = MonocleConfig::default();
    config.harness_profiles.push(HarnessProfile {
        id: id.to_string(),
        display_name: format!("Profile {id}"),
        binary_path: "/usr/local/bin/claude".to_string(),
        config_dir: None,
    });
    App::new(config)
}

// ---------------------------------------------------------------------------
// Test — MAJOR-1 / BC-2.07.005 PC-5 (GREEN after fix c929848)
//
// commit_profile_selection WRAPPER must not insert project_profiles[""]
// when current_dir is "".
//
// The hoisted guard in commit_profile_selection fires before config_path()
// on ALL platforms / ALL branches, making this test GREEN everywhere.
//
// Regression value: removing or moving the guard past the config_path() call
// would cause this test to FAIL on Linux CI (where the Err branch is reachable).
// ---------------------------------------------------------------------------

/// MAJOR-1 regression / BC-2.07.005 PC-5:
///
/// `commit_profile_selection` (WRAPPER) must never insert `project_profiles[""] = id`
/// when `current_dir` is `""` (CWD resolution failure — INV-5 normalization contract).
///
/// HOME is temporarily unset via `temp_env::with_vars` to probe `config_path()`
/// reachability.  On Linux CI the Err branch fires but the hoisted guard has already
/// returned.  On macOS the Ok branch fires; the guard still fires first.  Either way
/// the empty-key must not appear.
///
/// `temp_env::with_vars` provides safe, scoped env-var mutation compliant with the
/// `monocle-no-raw-env-mutation-in-tests` Semgrep rule (SS-conventions-anti-patterns.md
/// §Test Conventions).
///
/// Assertions:
/// - `project_profiles` does not contain the `""` key.
/// - `profile_picker` is `None` (picker closed regardless of branch taken).
/// - `status_message` is `Some(_)` (error surfaced to user).
#[test]
fn test_BC_2_07_005_commit_wrapper_err_branch_empty_cwd_does_not_insert_empty_key() {
    // temp_env::with_vars provides safe scoped env mutation.
    // The None value unsets HOME for the duration of the closure.
    temp_env::with_vars([("HOME", None::<&str>)], || {
        let err_branch_triggered = MonocleConfig::config_path().is_err();
        let mut app = make_app_with_profile("cc");

        // Open the picker so commit has a selection to process.
        open_profile_picker(&mut app);
        assert!(
            app.profile_picker.is_some(),
            "picker must be open before commit"
        );

        // Call the WRAPPER with empty current_dir while HOME is unset.
        // The hoisted guard fires BEFORE config_path() on all platforms / all branches.
        commit_profile_selection(&mut app, "");

        // ASSERTION 1: project_profiles must not contain the empty-string key on ANY branch.
        assert!(
            !app.config.project_profiles.contains_key(""),
            "MAJOR-1 / BC-2.07.005 PC-5: commit_profile_selection(\"\") must never insert \
             project_profiles[\"\"] = id. Found entry: {:?}. \
             Err-branch triggered = {}. \
             REGRESSION: the hoisted empty-CWD guard in commit_profile_selection was removed \
             or moved past the config_path() call.",
            app.config.project_profiles.get(""),
            err_branch_triggered,
        );

        // ASSERTION 2: picker must be closed (guard closes it before returning).
        assert!(
            app.profile_picker.is_none(),
            "MAJOR-1 / BC-2.07.005 PC-5: picker must be closed after commit with empty CWD"
        );

        // ASSERTION 3: status_message must be set (error surfaced to user — not silent).
        assert!(
            app.status_message.is_some(),
            "MAJOR-1 / BC-2.07.005 PC-5: status_message must be Some(_) after empty-CWD \
             commit so the user sees the failure (not a silent drop)"
        );
    });
}
