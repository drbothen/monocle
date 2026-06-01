//! ADV Pass-2 failing tests for S-031 profile picker (production path coverage).
//!
//! These tests exercise the PRODUCTION code paths — not the clean helper API
//! (`open_profile_picker_with_dir`) — and are designed to fail against the current
//! implementation until the three Pass-2 blockers are fixed.
//!
//! # Red Gate — why each test FAILS today
//!
//! 1. **test_BC_2_07_005_production_open_no_sticky_entry_pre_selects_index_0**
//!    (BLOCKER-1 / AC-002 / BC-2.07.005 PC-4):
//!    `open_profile_picker` (the Ctrl-P-wired production function) has a Step-2
//!    `.or_else()` fallback: when the process CWD has no sticky entry, it iterates
//!    `project_profiles.values()` and returns the FIRST profile it finds (HashMap
//!    order is non-deterministic). When `project_profiles` has TWO entries for OTHER
//!    directories, Step-2 fires and may select a non-zero index. The test asserts
//!    index 0 — FAILS when Step-2 picks an arbitrary other-dir profile.
//!
//! 2. **test_BC_2_07_005_active_marker_uses_per_dir_not_first_match** (BLOCKER-2 /
//!    BC-2.07.004 PC-2 / BC-2.07.005 PC-2):
//!    `render_profile_picker` determines the `*` active marker by calling
//!    `config.project_profiles.values().find(...)` — a first-match over all values.
//!    `ProfilePickerState` has no `current_dir` field, so the widget cannot compute
//!    the per-dir active profile. This test asserts the `*` marker appears on profile
//!    Y's row (the per-dir correct one), not on a decoy X row. FAILS because the
//!    widget uses first-match and `current_dir` is absent.
//!
//! 3. **test_BC_2_07_005_commit_production_empty_cwd_does_not_write_empty_dir_key**
//!    (MAJOR-2 / BC-2.07.005 PC-5):
//!    When CWD resolution fails (simulated by passing "" as `current_dir`), the
//!    production commit path must NOT write `project_profiles[""] = id`. The current
//!    `commit_profile_selection_with_path` guards on `selected_id.is_empty()` but
//!    does NOT guard on `current_dir.is_empty()`. FAILS because writing with an
//!    empty dir key proceeds unchecked.
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
use monocle_tui::app::{commit_profile_selection_with_path, open_profile_picker, App};
use monocle_tui::ui::profile_picker_widget::render_profile_picker;
use ratatui::{backend::TestBackend, Terminal};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_config_with_profiles(ids: &[&str]) -> MonocleConfig {
    let mut config = MonocleConfig::default();
    for id in ids {
        config.harness_profiles.push(HarnessProfile {
            id: id.to_string(),
            display_name: format!("Profile {id}"),
            binary_path: "/usr/local/bin/claude".to_string(),
            config_dir: None,
        });
    }
    config
}

fn make_config_with_project_profiles(
    profile_ids: &[&str],
    project_entries: &[(&str, &str)],
) -> MonocleConfig {
    let mut config = make_config_with_profiles(profile_ids);
    for (dir, pid) in project_entries {
        config
            .project_profiles
            .insert(dir.to_string(), pid.to_string());
    }
    config
}

/// Collect all cell symbols from a `Terminal<TestBackend>` buffer as a single string.
fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer().clone();
    let area = buf.area();
    (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Test 1 — BLOCKER-1 / AC-002 / BC-2.07.005 PC-4
//
// PRODUCTION open_profile_picker with no sticky entry for CWD → pre-select index 0.
//
// The PRODUCTION `open_profile_picker` resolves CWD internally via
// std::env::current_dir(). In the test process, CWD will NOT be "/fake-dir-zeta".
// Therefore:
//   - Step-1 (resolve_profile_for_dir for the actual CWD) → None (miss).
//   - Step-2 (fallback) fires: iterates project_profiles.values() — EXACTLY ONE
//     entry exists: "/fake-dir-zeta" → "zzz" (sorted index 2). Since there is only
//     one entry, HashMap iteration is deterministic: it ALWAYS returns "zzz" (index 2).
//
// BC-2.07.005 PC-4: "if no sticky entry exists, the FIRST profile in the list is
// highlighted" = index 0 (first in alphabetically-sorted snapshot).
//
// Because Step-2 deterministically returns index 2 and the correct answer is index 0,
// this assertion is deterministically RED on any machine.
//
// TO IMPLEMENTER: Remove the Step-2 `.or_else()` fallback entirely.
// `open_profile_picker` must delegate to `open_profile_picker_with_dir(app, &cwd)`
// (which already does the correct `unwrap_or(0)` with NO Step-2 fallback).
// ---------------------------------------------------------------------------

/// BLOCKER-1 / AC-002 / BC-2.07.005 PC-4 (RED):
///
/// When the process CWD has no sticky entry in `project_profiles`, the PRODUCTION
/// `open_profile_picker` must pre-select index 0 (first profile in sorted list).
///
/// Config has EXACTLY ONE project_profiles entry for a directory OTHER than the test's CWD:
///   "/fake-dir-zeta" → "zzz" (sorted index 2 — alphabetically LAST)
/// Profiles sorted: ["aaa", "mmm", "zzz"] → aaa=0, mmm=1, zzz=2.
///
/// The test process CWD is not "/fake-dir-zeta", so Step-1 returns None.
/// Step-2 fires: `project_profiles.values()` has EXACTLY ONE entry ("zzz"), so it
/// deterministically returns index 2. The assertion `index == 0` therefore FAILS
/// on every machine (not a HashMap-ordering fluke).
///
/// Correct behavior (AC-002 / BC-2.07.005 PC-4): no sticky entry → index 0.
///
/// FAILS NOW: Step-2 or_else fallback fires and returns index 2 ("zzz").
#[test]
fn test_BC_2_07_005_production_open_no_sticky_entry_pre_selects_index_0() {
    // ONE project_profiles entry for a directory OTHER than the process CWD.
    // Maps to "zzz" — sorted index 2 (alphabetically last).
    // With exactly one HashMap entry, Step-2 iteration is deterministic: always "zzz".
    let config = make_config_with_project_profiles(
        &["aaa", "mmm", "zzz"],
        &[
            ("/fake-dir-zeta", "zzz"), // sorted index 2 — the ONLY entry
        ],
    );

    let mut app = App::new(config);

    // Precondition: verify the process CWD is not the fake dir.
    let process_cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    assert!(
        process_cwd != "/fake-dir-zeta",
        "Test precondition: process CWD must not be '/fake-dir-zeta'. \
         Got: {:?}. If this fires, the test environment is unusual.",
        process_cwd
    );

    // Call the PRODUCTION open_profile_picker (resolves CWD internally).
    // Step-1: resolve_profile_for_dir(config, &process_cwd) → None (not in map).
    // Step-2 (WRONG): fallback to project_profiles.values() → ONLY entry is "zzz" (index 2).
    // Correct behavior (AC-002 / BC-2.07.005 PC-4): no sticky entry → unwrap_or(0) → index 0.
    open_profile_picker(&mut app);

    let state = app
        .profile_picker
        .as_ref()
        .expect("profile_picker must be Some after open_profile_picker");

    assert_eq!(
        state.selected_index, 0,
        "BLOCKER-1 / AC-002 / BC-2.07.005 PC-4: when the process CWD has no sticky entry, \
         open_profile_picker must pre-select index 0 (first profile in sorted list). \
         Got index {} (expected 0). FAILS because the Step-2 or_else fallback fires: \
         project_profiles.values() has exactly one entry ('zzz', index 2) and returns it. \
         FIX: remove the Step-2 or_else block; delegate to open_profile_picker_with_dir \
         which correctly uses unwrap_or(0).",
        state.selected_index
    );
}

// ---------------------------------------------------------------------------
// Test 2 — BLOCKER-2 / BC-2.07.004 PC-2 / BC-2.07.005 PC-2
//
// `*` active marker in render_profile_picker must use per-directory lookup,
// NOT first-match over all project_profiles.values().
//
// Current render_profile_picker code (profile_picker_widget.rs ~92-99):
//
//   let active_profile_id: Option<&str> = config
//       .project_profiles
//       .values()
//       .find(|profile_id| config.harness_profiles.iter().any(|p| &&p.id == profile_id))
//       .map(|s| s.as_str());
//
// This finds the FIRST valid project_profiles value in HashMap iteration order —
// it ignores which directory the picker was opened for.
//
// Additionally, `ProfilePickerState` has no `current_dir: String` field, so the
// widget cannot compute the correct per-dir active profile even if it wanted to.
//
// Setup (deterministic RED):
//   - "/dir-x" → "xxx" (the DECOY, sorted index 0 — alphabetically first)
//   - "/dir-y" → "yyy" (the TARGET, sorted index 2 — alphabetically last)
//   Profiles sorted: ["mmm", "xxx", "yyy"] → mmm=0, xxx=1, yyy=2.
//
// We open the picker for "/dir-y" (using open_profile_picker_with_dir so that
// selected_index is correctly 2). We then render with a config where "/dir-y" → "yyy".
// The `*` marker must appear on "yyy"'s row.
//
// FAILS because render_profile_picker uses first-match over values(): it may return
// "xxx" (decoy, from "/dir-x") depending on HashMap order, placing `*` on the wrong row.
//
// TO IMPLEMENTER:
//   1. Add `current_dir: String` to `ProfilePickerState`.
//   2. In `render_profile_picker`, pass `state.current_dir` to
//      `resolve_profile_for_dir(&config, &state.current_dir)` to get the active profile.
//   3. Remove the first-match `project_profiles.values()` code from render_profile_picker.
// ---------------------------------------------------------------------------

/// BLOCKER-2 / BC-2.07.004 PC-2 / BC-2.07.005 PC-2 (RED):
///
/// The `*` active marker in the rendered picker modal must mark the profile that is
/// sticky for the CURRENT DIRECTORY — not an arbitrary first-match from all
/// `project_profiles.values()`.
///
/// Config:
///   "/dir-x" → "xxx" (DECOY — sorted index 1; alphabetically before "yyy")
///   "/dir-y" → "yyy" (TARGET — sorted index 2; the per-dir correct active profile)
///   Profiles sorted: ["mmm", "xxx", "yyy"] → mmm=0, xxx=1, yyy=2.
///
/// We open the picker for "/dir-y" and render with config above. The `*` marker
/// must appear on "yyy"'s row. FAILS because the widget uses first-match over
/// project_profiles.values() (may return "xxx" from /dir-x) AND ProfilePickerState
/// has no `current_dir` field to convey which directory was opened for.
///
/// FAILS NOW: render_profile_picker cannot distinguish per-dir active profile.
#[test]
fn test_BC_2_07_005_active_marker_uses_per_dir_not_first_match() {
    // Profiles sorted: ["mmm", "xxx", "yyy"] → mmm=0, xxx=1, yyy=2.
    // "/dir-x" → "xxx" (decoy, sorted index 1)
    // "/dir-y" → "yyy" (target, sorted index 2)
    let config = make_config_with_project_profiles(
        &["mmm", "xxx", "yyy"],
        &[
            ("/dir-x", "xxx"), // decoy
            ("/dir-y", "yyy"), // target: active profile for /dir-y
        ],
    );

    let mut app = App::new(config);

    // Open picker for "/dir-y". open_profile_picker_with_dir correctly sets
    // selected_index = 2 ("yyy"). We use the _with_dir helper here because
    // this test targets BLOCKER-2 (render path), not BLOCKER-1 (open path).
    monocle_tui::app::open_profile_picker_with_dir(&mut app, "/dir-y");
    assert!(app.profile_picker.is_some(), "picker must be open");

    let state = app.profile_picker.as_ref().expect("picker must be Some");

    // Render the picker into a TestBackend buffer.
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal");

    terminal
        .draw(|frame| {
            render_profile_picker(state, &app.config, frame.area(), frame.buffer_mut());
        })
        .expect("render_profile_picker must not panic");

    let all_text = buffer_text(&terminal);

    // The `*` marker must appear on "yyy"'s row. We check that "* yyy" appears in
    // the rendered buffer (the active prefix "* " immediately before the profile id).
    assert!(
        all_text.contains("* yyy"),
        "BLOCKER-2 / BC-2.07.004 PC-2 / BC-2.07.005 PC-2: the '* ' active marker must appear \
         on 'yyy' (the per-dir active profile for '/dir-y'). \
         FAILS because render_profile_picker uses first-match over project_profiles.values() \
         instead of looking up the per-directory active profile via current_dir. \
         Additionally, ProfilePickerState has no current_dir field. \
         Rendered buffer excerpt (first 200 chars): {:?}",
        &all_text[..all_text.len().min(200)]
    );

    // Safety: ensure the marker is NOT on "xxx" (decoy). If "* xxx" appears in the buffer,
    // the first-match bug is clearly visible.
    assert!(
        !all_text.contains("* xxx"),
        "BLOCKER-2: '* xxx' (decoy) must NOT be the active marker; \
         the decoy '/dir-x'→'xxx' entry must not influence the active marker for '/dir-y'."
    );
}

// ---------------------------------------------------------------------------
// Test 3 — MAJOR-2 / BC-2.07.005 PC-5
//
// Empty-CWD commit guard: when current_dir is "" (CWD resolution failure in the
// Enter handler), commit_profile_selection_with_path must NOT write
// project_profiles[""] = id and must set app.status_message.
//
// The Enter handler in dispatch_key_event resolves CWD and then calls
// commit_profile_selection(app, &cwd_string). If current_dir fails:
//   cwd_string = std::env::current_dir()
//       .map(|p| p.to_string_lossy().into_owned())
//       .unwrap_or_default()   ← yields ""
//
// commit_profile_selection_with_path currently checks `if selected_id.is_empty()`
// (line ~1239 in app.rs) but does NOT check `if current_dir.is_empty()`. So when
// current_dir = "" and selected_id = "cc", it writes project_profiles[""] = "cc"
// — a silent data-corruption defect.
//
// TO IMPLEMENTER:
//   Add a guard at the top of commit_profile_selection_with_path:
//     if current_dir.is_empty() {
//         app.status_message = Some("Config save failed: CWD resolution failed".to_string());
//         app.profile_picker = None;
//         return;
//     }
// ---------------------------------------------------------------------------

/// MAJOR-2 / BC-2.07.005 PC-5 (RED):
///
/// When `current_dir` is `""` (CWD resolution failure), `commit_profile_selection_with_path`
/// must NOT write `project_profiles[""] = id` and must set `app.status_message`.
///
/// Simulation: pass `""` as `current_dir` directly (mimics what the Enter handler does
/// when `std::env::current_dir().unwrap_or_default()` yields an empty string).
///
/// FAILS because `commit_profile_selection_with_path` does not guard against an
/// empty `current_dir` key — it proceeds to insert `project_profiles[""] = "cc"`.
#[test]
fn test_BC_2_07_005_commit_production_empty_cwd_does_not_write_empty_dir_key() {
    use tempfile::TempDir;

    let config = make_config_with_profiles(&["cc"]);
    let mut app = App::new(config);

    // Open picker so there is a selection to commit.
    open_profile_picker(&mut app);
    assert!(app.profile_picker.is_some(), "picker must be open");
    // Profiles: ["cc"] → selected_index = 0 → selected_id = "cc" (non-empty).
    assert_eq!(
        app.profile_picker.as_ref().unwrap().profiles,
        vec!["cc".to_string()]
    );

    // Write to a real (but temporary) config path to isolate the test from the user's config.
    let tmp = TempDir::new().expect("tempdir");
    let config_path = tmp.path().join("config.json");

    // Call the production commit path with an empty current_dir ("") — simulates CWD failure.
    // The selected profile is "cc" (non-empty), so the only guard that should fire is
    // the empty-dir guard (which does NOT exist today).
    commit_profile_selection_with_path(&mut app, "", &config_path);

    // ASSERTION 1: project_profiles must NOT have an entry with the empty-string key.
    // An empty-key entry is a data-corruption defect: resolve_profile_for_dir will never
    // find it (real CWDs are non-empty), but it permanently pollutes the config file.
    let empty_key_entry = app.config.project_profiles.get("");
    assert!(
        empty_key_entry.is_none(),
        "MAJOR-2 / BC-2.07.005 PC-5: commit with empty current_dir must NOT write \
         project_profiles[\"\"] = id. Found entry: {:?}. \
         FAILS because commit_profile_selection_with_path does not guard against \
         current_dir.is_empty(). FIX: add an early-return guard on empty current_dir.",
        empty_key_entry
    );

    // ASSERTION 2: app.status_message must be set to communicate the failure.
    // An empty-dir commit is a detectable error condition — the user should be notified.
    assert!(
        app.status_message.is_some(),
        "MAJOR-2 / BC-2.07.005 PC-5: commit with empty current_dir must set \
         app.status_message to notify the user of the CWD resolution failure. \
         Got: None. FAILS because the empty-dir guard does not exist yet."
    );

    // ASSERTION 3: picker must close regardless (no dangling open picker).
    assert!(
        app.profile_picker.is_none(),
        "BC-2.07.005 PC-5: picker must close after commit (even on empty-dir failure)"
    );
}
