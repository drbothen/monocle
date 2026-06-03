---
document_type: story
level: L4
story_id: S-031
epic_id: EPIC-07
version: "1.2"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 5
wave: 7
tdd_mode: strict
priority: P1
depends_on: [S-030, S-024, S-025]
blocks: []
target_module: monocle-tui
subsystems: [SS-07]
behavioral_contracts: [BC-2.07.004, BC-2.07.005]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-07/BC-2.07.004.md, version: "1.0.2"}
  - {path: .factory/specs/behavioral-contracts/ss-07/BC-2.07.005.md, version: "1.3.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.07.004 (profile picker widget), BC-2.07.005 (profile switch + config save)"
---

# S-031: Profile Picker — Profile Selection Widget and Config Save

## Narrative

As a daemon operator managing multiple Claude Code environments, I want to press `Ctrl-P`
in the TUI (in any AppMode) to open a profile picker, select from defined profiles, and
have my selection saved to `config.json` atomically, so that I can switch between
project contexts without manually editing configuration files.

## Acceptance Criteria

### AC-001 (traces to BC-2.07.005 postcondition PC-1 — profile picker entry via Ctrl-P)
In ANY `AppMode`, pressing `Ctrl-P` dispatches `Action::ProfilePicker`
via `resolve_binding()` (registered in the `Global` layer, fires in all AppModes per
BC-2.07.005 INV-1). The TUI calls `open_profile_picker(app)` which sets
`app.profile_picker = Some(ProfilePickerState { .. })` without changing `app.mode`.
The profile picker is NOT an `AppMode` variant and does NOT use `AppMode::Overlay`.
The `App.mode` is unchanged (e.g., remains `Dashboard`) while the picker is open.

### AC-002 (traces to BC-2.07.005 postcondition PC-2 — profile list rendering)
When `App.profile_picker` is `Some(ProfilePickerState)`, a centered modal is rendered
over the current view (using `ratatui::widgets::Clear` + `Block`). The modal lists all
profiles from `App.config.harness_profiles` as a scrollable list, showing each profile's
`display_name`. The profile pre-selected (highlighted) on open is the sticky profile for
the current working directory (`resolve_profile_for_dir(&config, current_dir)`); if no
sticky entry exists, the first profile in the list is highlighted. If `harness_profiles`
is empty, the modal renders: `"No profiles configured — add profiles to config.json"`
(per BC-2.07.005 PC-3).

### AC-003 (traces to BC-2.07.004 postcondition PC-3 — profile picker navigation)
While `App.profile_picker` is `Some(ProfilePickerState)`:
- `j` / `↓` moves selection down one row (wraps to top)
- `k` / `↑` moves selection up one row (wraps to bottom)
- `Enter` selects the highlighted profile (triggers AC-005)
- `Esc` closes the picker without changes: sets `App.profile_picker = None`

### AC-004 (traces to BC-2.07.004 postcondition PC-4 — picker modal keyboard isolation)
While `App.profile_picker` is `Some(ProfilePickerState)`, ALL key events are consumed
by the picker. Session nav keys (`Tab`, `Enter` on sessions, `j`/`k` for session scroll)
do NOT fire while the picker is open. The picker keyboard handler takes precedence over
`resolve_binding()` for standard session navigation.

### AC-005 (traces to BC-2.07.005 postcondition PC-5 — profile switch saves config atomically)
On `Enter` in the profile picker, the TUI:
1. Writes the selected profile's `id` into `config.project_profiles[current_dir]`
   (BC-2.07.005 PC-5a; `current_dir` is the verbatim working directory path per
   BC-2.07.004 INV-1 / BC-2.07.005 INV-5)
2. Calls `write_config(&updated_config, &config_path)` — atomic write via `tempfile::persist`
   (BC-2.07.005 PC-5b / BC-2.07.001)
3. Sets `app.profile_picker = None` (closes picker, per BC-2.07.005 PC-7)
4. Logs `INFO: profile switched to <id>`
5. Calls `detect_ccr(&config)` and logs `INFO: ccr_path resolved to <path>` if Some, or
   `WARN: ccr_path not found for profile <id>` if None

### AC-006 (traces to BC-2.07.005 postcondition PC-5c — config save error display, in-memory update decoupled)
If `write_config()` returns `Err`, the TUI renders a transient error notification in the
status bar: `"Config save failed: <error>"`. The in-memory profile selection IS applied
regardless of the write failure — the selected profile's `id` remains in
`config.project_profiles[current_dir]` in memory for the duration of the current session
(BC-2.07.005 PC-5c / INV-3: in-memory update is decoupled from write success). The
persisted `config.json` file is unchanged. `app.profile_picker = None` (picker closes).

### AC-007 (traces to BC-2.07.005 postcondition PC-5 — detect_ccr on switch)
After a profile selection (whether persistence succeeded or failed), `detect_ccr(&config)`
is called and its result is stored in `App.ccr_path: Option<PathBuf>`. This value is used
by subsequent sessions panel rendering to display the active CCR path in the status bar
footer (e.g., `"CCR: ~/.claude/"` or `"CCR: none"`). The `App.ccr_path` field is also
initialized at TUI startup via `detect_ccr(&config)` so the status bar shows the CCR path
from first render.

### AC-008 (traces to BC-2.07.004 invariant INV-1 — picker is not AppMode::Overlay)
The profile picker MUST NOT use `AppMode::Overlay`. It MUST be modeled as
`Option<ProfilePickerState>` in `App`. The reason: permission overlays (VecDeque<PromptModal>)
and the profile picker are orthogonal features; using `AppMode::Overlay` for the picker
would incorrectly conflate them with the permission overlay stack.

### AC-009 (traces to BC-2.07.005 invariant INV-2 — atomic write required)
`write_config()` MUST use `tempfile::persist` (from `monocle-config::write_config()`).
Any code path that calls `std::fs::write` directly for config is a forbidden pattern
and MUST trigger the `monocle-no-direct-config-write` semgrep rule.

### AC-010 (traces to BC-2.07.005 postcondition PC-1, PC-7, PC-8 — integration: key dispatch, render, navigation, dismiss)
The profile picker MUST be fully integrated into the TUI event loop and render path:
1. `dispatch_key_event` routes `Action::ProfilePicker` to `open_profile_picker(app)` —
   verifiable by a synthetic `Ctrl-P` key event test asserting `app.profile_picker.is_some()`.
2. `render_frame` renders the picker modal when `app.profile_picker.is_some()` — verifiable
   by asserting the picker widget appears in the render output (using a test backend).
3. While the picker is open, `dispatch_key_event` routes picker-local keys (`↑`/`↓`, `Enter`,
   `Esc`) to their picker handlers before `resolve_binding()` runs for session navigation.
4. `Esc` calls `close_profile_picker(app)` and sets `app.profile_picker = None` without
   calling `write_config` (BC-2.07.005 PC-8).
5. `App.ccr_path` is set at TUI startup via `detect_ccr(&config)` and shown in the status
   bar footer; the status bar displays `"CCR: <path>"` or `"CCR: none"` from first render.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,800 |
| BC-2.07.004.md | ~800 |
| BC-2.07.005.md | ~800 |
| S-030 (MonocleConfig, detect_ccr, save) | ~700 |
| S-024 (AppMode, resolve_binding, Action) | ~600 |
| S-025 (App struct, TUI layout) | ~500 |
| Test files | ~900 |
| **Total estimate** | **~6,100** |

## Tasks

- [ ] Define `ProfilePickerState` struct in `monocle-tui/src/app.rs`:
      `{ selected_index: usize, profiles: Vec<String>, current_dir: String }` — snapshot of
      profile ids at open time; `current_dir` is the verbatim working-directory path for
      `project_profiles` write-key (BC-2.07.005 INV-5)
- [ ] Add `App.profile_picker: Option<ProfilePickerState>` field
- [ ] Add `App.ccr_path: Option<PathBuf>` field; set via `detect_ccr(&config)` at startup and
      after every profile selection (whether or not `write_config` succeeds)
- [ ] Register `Ctrl-P` in `Global` binding layer → `Action::ProfilePicker`
      (fires in ALL AppModes per BC-2.07.005 INV-1; no AppMode guard)
- [ ] Implement `Action::ProfilePicker` handler: populate `ProfilePickerState` from
      `config.harness_profiles` ids, sorted alphabetically by `display_name`; pre-select the
      sticky profile via `resolve_profile_for_dir(&config, &current_dir)`; set
      `App.profile_picker = Some(state)` (no-op if picker already open per EC-110)
- [ ] Implement picker keyboard handler (evaluated before `resolve_binding()` when picker is open):
      `j`/`↓` → increment selected_index (wrap), `k`/`↑` → decrement (wrap), `Enter` → select, `Esc` → close
- [ ] Implement profile selection on `Enter`:
      1. Write selected id to `config.project_profiles[current_dir]` (in memory)
      2. Call `write_config(&config, &MonocleConfig::config_path()?)` — atomic write
      3. If write fails: render status bar error `"Config save failed: <error>"`; in-memory
         change persists for this session (BC-2.07.005 PC-5c / INV-3)
      4. Set `app.profile_picker = None`
      5. Call `detect_ccr(&config)`, update `App.ccr_path`
- [ ] Create `monocle-tui/src/ui/profile_picker_widget.rs` — centered modal list widget
      using `ratatui::widgets::Clear` + `Block` + `List`; highlight pre-selected entry;
      render "No profiles configured — add profiles to config.json" when harness_profiles empty
- [ ] Wire profile picker widget into `render_frame` (rendered on top of layout when
      `app.profile_picker.is_some()`)
- [ ] Add `"CCR: <path>"` or `"CCR: none"` to status bar (extend S-027 status bar); initialize
      `App.ccr_path` at startup with `detect_ccr(&config)` before first render
- [ ] Integration test: synthetic `Ctrl-P` key event → `dispatch_key_event` sets
      `app.profile_picker.is_some()`; AppMode unchanged; `render_frame` outputs picker widget
- [ ] Unit tests `monocle-tui/tests/profile_picker.rs` — open/close, navigation wrap, Enter selects,
      Esc closes without change, empty harness_profiles message, Ctrl-P no-op when already open
- [ ] Unit tests `monocle-tui/tests/profile_switch.rs` — write_config called on Enter,
      write error shows "Config save failed: ..." and in-memory profile updated,
      detect_ccr called after selection, sticky pre-selection via resolve_profile_for_dir

## Previous Story Intelligence

S-030 (config foundation): `write_config()` is implemented and tested. The atomic write via
`tempfile::persist` is in `monocle-config::write_config`. The `detect_ccr()` function and
`resolve_profile_for_dir()` are also from `monocle-config`. This story calls them from
`monocle-tui/src/app.rs` via
`use monocle_config::{MonocleConfig, write_config, detect_ccr, resolve_profile_for_dir, ConfigError}`.
Note: the config write API is `write_config(&config, &path)`, NOT `config.save(...)` — there
is no `save()` method on `MonocleConfig`.

S-024 (TUI core types): `Action::ProfilePicker` (NOT `OpenProfilePicker`) was canonicalized in
F-P1D-003 / BC-2.07.005 v1.1.0 at S-024 authoring time. <!-- version-pin-historical: historical record of canonicalization event; not a navigation pointer --> The variant is already named `ProfilePicker` in
`monocle-core/src/tui/state.rs`. Because `Action` is `#[non_exhaustive]`, all existing
`match action` arms with `_` catch-all continue to work.

S-025 (TUI skeleton): `App` struct is defined in `monocle-tui/src/app.rs`. Adding
`profile_picker: Option<ProfilePickerState>` and `ccr_path: Option<PathBuf>` extends
the existing struct without breaking its consumers.

## Architecture Compliance Rules

From `architecture/SS-config.md` and `architecture/SS-tui.md`:
- Profile picker is `Option<ProfilePickerState>` in `App` — NOT an `AppMode` variant,
  NOT `AppMode::Overlay`, NOT `AppMode::Fullscreen`
- `Ctrl-P` → `Action::ProfilePicker` is registered in the `Global` binding layer and fires
  in ALL AppModes (BC-2.07.005 INV-1); no AppMode guard is applied to this action
- All config writes via `monocle_config::write_config(&config, &path)` (which uses
  `tempfile::persist`) — never `std::fs::write` directly in `monocle-tui`
- `detect_ccr(&config)` called after every profile selection (success or failure) and at
  TUI startup — no stale CCR path
- Picker keyboard handler takes priority over `resolve_binding()` when picker is open —
  implement as a pre-check in `dispatch_key_event` before calling `resolve_binding()`
- `Action::ProfilePicker` (NOT `OpenProfilePicker` — renamed in F-P1D-003) is the
  canonical variant in `monocle-core` `Action` enum (it is `#[non_exhaustive]`)
- `resolve_profile_for_dir(&config, &current_dir)` is a pure function (no I/O) for sticky
  pre-selection at picker open time; lives in `monocle-config`

**Forbidden Dependencies:**
- Direct `std::fs::write` for config — all writes through `monocle_config::write_config()`
- Profile picker as `AppMode::Overlay` — architectural violation, incorrect model
- `detect_ccr` implemented in `monocle-tui` directly — use `monocle_config::detect_ccr()`
- `Action::OpenProfilePicker` — this variant does not exist; use `Action::ProfilePicker`

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| monocle-config | workspace path | `write_config()`, `detect_ccr()`, `resolve_profile_for_dir()`, `ConfigError` |
| monocle-core | workspace path | `AppMode`, `Action::ProfilePicker`, `resolve_binding()` |
| ratatui | workspace pin | `Clear`, `Block`, `List`, `ListItem`, `ListState` for picker widget |
| tracing | 0.1 | INFO on profile switch, WARN if ccr not found |

## File Structure Requirements

Files to create:
- `monocle-tui/src/ui/profile_picker_widget.rs` — picker modal widget
- `monocle-tui/tests/profile_picker.rs` — picker open/close/navigation tests
- `monocle-tui/tests/profile_switch.rs` — config save and detect_ccr tests

Files to modify:
- `monocle-tui/src/app.rs` — add `ProfilePickerState` (with `current_dir: String` field),
  `profile_picker: Option<ProfilePickerState>`, `ccr_path: Option<PathBuf>` fields;
  add picker keyboard handler in `dispatch_key_event`; wire `Action::ProfilePicker` dispatch
  to `open_profile_picker(app)`
- `monocle-tui/src/ui/mod.rs` — declare `profile_picker_widget` module
- `monocle-tui/src/ui/status_bar.rs` (from S-027) — add CCR path display; initialize
  `App.ccr_path` at startup
- `monocle-core/src/tui/state.rs` (from S-024) — `Action::ProfilePicker` already present
  (renamed from `OpenProfilePicker` in F-P1D-003); no new variant needed

## Downstream Consumer Contract

No new public API produced by this story. The profile picker is an internal TUI feature.
This is a leaf story — nothing depends on it in the current phase.

## §Trace v1.2

**ADV S-031 Pass-1 — symbol drift reconciliation and integration AC** (2026-06-01):
- BLOCKER-3 symbol drift corrected throughout:
  - `Action::OpenProfilePicker` → `Action::ProfilePicker` (canonical per BC-2.07.005 §Trace
    v1.1.0 / F-P1D-003; affected AC-001, Tasks, Previous Story Intelligence, Architecture
    Compliance Rules, Library table, File Structure).
  - `config.profiles` / `config.profiles.keys()` → `config.harness_profiles` (actual field on
    `MonocleConfig`; affected AC-002, Tasks).
  - Removed all references to `config.active_profile` (no such field); replaced with
    `config.project_profiles[current_dir]` write and `resolve_profile_for_dir()` for pre-selection.
  - `config.save()` / `MonocleConfig::save()` → `write_config(&config, &path)` (actual API;
    affected AC-005, AC-009, Tasks, Architecture Compliance Rules, Library table).
- MAJOR-1 sticky pre-selection wording: AC-002 now correctly states picker pre-selects the
  sticky profile FOR THE CURRENT DIRECTORY via `resolve_profile_for_dir(&config, current_dir)`,
  not a first-match across all `project_profiles`.
- AC-006 aligned to BC-2.07.005 PC-5c: on write failure the in-memory profile IS applied
  (was: "profile NOT switched, old active_profile restored" — wrong); error message corrected
  from "Error saving config: <e>" to "Config save failed: <error>".
- AC-007 updated: detect_ccr called after selection whether or not write succeeds; also called
  at startup (not only on switch).
- AC-009 corrected: `write_config()` → canonical API (was: `config.save()`); invariant
  reference corrected to BC-2.07.005 INV-2.
- AC-010 added (integration-render + key-dispatch): dispatch_key_event → open_profile_picker,
  render_frame renders picker when open, navigation/commit/close routed while open, ccr_path
  set at startup via detect_ccr shown in status bar. Closes dead-code integration gap.
- Narrative updated: `press \`p\`` → `press \`Ctrl-P\`` (fires in any AppMode per BC-2.07.005 INV-1).
- Frontmatter input pins updated: BC-2.07.004 1.0.0→1.0.2, BC-2.07.005 1.0.0→1.3.1.
- SE-16d monotonicity: v1.2 timestamp 2026-06-01 >= v1.1 timestamp 2026-05-29. PASS.

## §Trace v1.1

**F-S025-ADV22-MED-001 sibling propagation — SS-tui-core.md → SS-tui.md (line 150)** (2026-05-29):
- Architecture Compliance Rules header: `architecture/SS-config.md` and `architecture/SS-tui-core.md` → `architecture/SS-config.md` and `architecture/SS-tui.md`.
- Systematic EPIC-06 story-writing burst defect; canonical anchor is `SS-tui.md` per BC-2.06.005 §Architecture Source + audit-table.md row 41.
- SE-16d monotonicity: v1.1 timestamp 2026-05-29 >= v1.0 timestamp 2026-05-27. PASS.
