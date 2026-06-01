//! ADV Pass-4 failing test for S-031 profile picker — MAJOR-1: wrapper empty-CWD guard gap.
//!
//! # Finding: MAJOR-1 — Wrapper Err-branch missing empty-CWD guard
//!
//! `commit_profile_selection` (the WRAPPER, app.rs ~1123-1154) resolves
//! `MonocleConfig::config_path()`. Its `Ok` branch correctly delegates to
//! `commit_profile_selection_with_path`, which has an early-return guard:
//!
//!   ```text
//!   if current_dir.is_empty() { ... return; }
//!   ```
//!
//! Its `Err` branch (fired when `config_path()` fails — no HOME dir on Linux etc.)
//! does NOT have an equivalent guard. Instead it proceeds to:
//!
//!   ```text
//!   if let Some(id) = selected_id {
//!       if !id.is_empty() {
//!           app.config.project_profiles.insert(current_dir.to_string(), id.clone());
//!                                               ^^^^^^^^^^^^^^^^ no guard on current_dir
//!       }
//!   }
//!   ```
//!
//! So when `current_dir == ""` AND `config_path()` returns `Err`, the Err branch inserts
//! `project_profiles[""] = id` — a silent config-corruption defect (BC-2.07.005 PC-5 /
//! INV-5 normalization contract).
//!
//! # Platform note
//!
//! On macOS, `directories::ProjectDirs::from("", "", "monocle")` always succeeds via
//! a `getpwuid_r` fallback even when `HOME` is unset. Therefore the Err branch of
//! `commit_profile_selection` is NOT triggerable via env-var manipulation on macOS —
//! `config_path()` always returns `Ok` regardless.
//!
//! On Linux CI (standard containers without XDG fallback), `HOME=""` causes
//! `ProjectDirs::from` to return `None`, `config_path()` returns
//! `Err(ConfigError::HomeUnresolvable)`, and the Err branch fires.
//!
//! The test is structured to:
//!   - Attempt to trigger the Err branch by temporarily unsetting HOME.
//!   - If the Err branch fires (Linux): assert no empty key → FAILS against current code.
//!   - If the Err branch is unavailable (macOS): assert the Ok-branch invariant (passes)
//!     AND explicitly fail with a documented RED-gate message so the platform gap is visible.
//!
//! The explicit `panic!` in the macOS path makes the test RED on macOS while keeping the
//! error message informative. The implementer's fix (hoisting the guard BEFORE
//! `config_path()`) makes the test GREEN on both platforms.
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
// Env-serialisation lock
//
// HOME manipulation is process-wide. Serialise against any parallel test thread
// that also calls code depending on HOME (e.g. config_path, directories crate).
// ---------------------------------------------------------------------------
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
// Test — MAJOR-1 / BC-2.07.005 PC-5 (RED)
//
// commit_profile_selection WRAPPER must not insert project_profiles[""]
// when current_dir is "".
//
// Red Gate on Linux CI: the Err branch fires → inserts "" key → assertion fails.
// Red Gate on macOS: the Err branch cannot fire; explicit panic documents the gap.
//
// Fix: hoist `if current_dir.is_empty() { ... return; }` to the TOP of
// commit_profile_selection, BEFORE the config_path() call. This protects both
// the Ok and Err branches with a single guard, making the test GREEN on all platforms.
// ---------------------------------------------------------------------------

/// MAJOR-1 / BC-2.07.005 PC-5 (RED):
///
/// `commit_profile_selection` (WRAPPER) must never insert `project_profiles[""] = id`
/// when `current_dir` is `""` (CWD resolution failure — INV-5 normalization contract).
///
/// **Why RED on Linux CI:** Unsetting HOME causes `config_path()` to return `Err`;
/// the wrapper's Err branch lacks the empty-CWD guard and inserts `""` key.
///
/// **Why RED on macOS:** The `directories` crate has a `getpwuid_r` fallback that
/// prevents `config_path()` from failing even with HOME unset. The Err branch is
/// unreachable in-process. The test explicitly panics with a RED-gate diagnostic
/// message to flag that the STRUCTURAL gap (no guard before config_path()) remains
/// unfixed, and the correct fix (hoist the guard) would eliminate this panic.
///
/// **After fix:** Hoisting the guard makes the wrapper reject `current_dir == ""`
/// before calling `config_path()`. Both platforms see a clean early-return, no empty
/// key, correct status_message — test GREEN on all platforms.
#[test]
fn test_BC_2_07_005_commit_wrapper_err_branch_empty_cwd_does_not_insert_empty_key() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let original_home = std::env::var("HOME").ok();

    // Temporarily unset HOME to attempt to trigger the Err branch of config_path().
    // SAFETY: ENV_MUTEX is held for the full HOME-unset window. This is the only test
    // in this file that mutates HOME. The risk of cross-file parallel races is inherent
    // to testing HOME-dependent code without a proper injection seam.
    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var("HOME");
    }

    let err_branch_triggered;
    let mut app = make_app_with_profile("cc");

    // Open the picker so commit has a selection to process.
    open_profile_picker(&mut app);
    assert!(
        app.profile_picker.is_some(),
        "picker must be open before commit"
    );

    // Check whether the Err branch is reachable in this environment.
    err_branch_triggered = MonocleConfig::config_path().is_err();

    // Call the WRAPPER with empty current_dir while HOME is unset.
    // - Err branch (Linux): config_path() → Err → Err branch fires → BUG: inserts "" key.
    // - Ok branch (macOS):  config_path() → Ok  → delegates to primary → primary guard fires.
    commit_profile_selection(&mut app, "");

    // Restore HOME unconditionally before any assertion that could panic.
    #[allow(unsafe_code)]
    unsafe {
        match &original_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
    }

    // ASSERTION: project_profiles must not contain the empty-string key on ANY branch.
    assert!(
        !app.config.project_profiles.contains_key(""),
        "MAJOR-1 / BC-2.07.005 PC-5: commit_profile_selection(\"\") must never insert \
         project_profiles[\"\"] = id. Found entry: {:?}. \
         FAILS on the Err branch (Linux CI, HOME unset) because the wrapper lacks the \
         empty-CWD guard. FIX: hoist `if current_dir.is_empty() {{ ... return; }}` to \
         the TOP of commit_profile_selection before the config_path() call.",
        app.config.project_profiles.get("")
    );

    // STRUCTURAL RED GATE for platforms where the Err branch is unavailable (macOS):
    // The Err branch unreachability means the guard gap is masked by the Ok branch
    // delegating to the already-guarded primary path. The fix — hoisting the guard to
    // the top of the wrapper — makes this unreachability irrelevant because the guard
    // fires before config_path() is ever called. Until the guard is hoisted, this
    // panic documents that the wrapper's Err branch is unprotected. After the fix,
    // remove this panic block (the test passes cleanly on all platforms without it).
    if !err_branch_triggered {
        panic!(
            "MAJOR-1 RED GATE (macOS platform): `config_path()` succeeded even with HOME \
             unset (directories crate getpwuid_r fallback). The Err branch of \
             commit_profile_selection is untriggerable in this environment, so the missing \
             empty-CWD guard is masked by the Ok branch delegating to the primary path. \
             This panic flags the structural gap. \
             FIX: hoist `if current_dir.is_empty() {{ app.status_message = ...; \
             app.profile_picker = None; return; }}` to the TOP of \
             commit_profile_selection (BEFORE the config_path() call). After the fix: \
             (1) the empty-key assertion above passes on all platforms, and \
             (2) remove this panic block — it is no longer needed."
        );
    }
}
