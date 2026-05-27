---
document_type: story
level: L4
story_id: S-031
epic_id: EPIC-07
version: "1.0"
status: not_started
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
  - {path: .factory/specs/behavioral-contracts/ss-07/BC-2.07.004.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-07/BC-2.07.005.md, version: "1.0.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.07.004 (profile picker widget), BC-2.07.005 (profile switch + config save)"
---

# S-031: Profile Picker — Profile Selection Widget and Config Save

## Narrative

As a daemon operator managing multiple Claude Code environments, I want to press `p`
in the TUI dashboard to open a profile picker, select from defined profiles, and
have my selection saved to `config.json` atomically, so that I can switch between
project contexts without manually editing configuration files.

## Acceptance Criteria

### AC-001 (traces to BC-2.07.004 postcondition PC-1 — profile picker entry)
From `AppMode::Dashboard { .. }`, pressing `p` dispatches `Action::OpenProfilePicker`
via `resolve_binding()` (registered in the `Global` layer). The TUI sets
`App.profile_picker: Some(ProfilePickerState { .. })` — profile picker is NOT an
`AppMode` variant and does NOT use `AppMode::Overlay`. The `App.mode` remains
`Dashboard` while the picker is open.

### AC-002 (traces to BC-2.07.004 postcondition PC-2 — profile list rendering)
When `App.profile_picker` is `Some(ProfilePickerState)`, a centered modal is rendered
over the dashboard (using `ratatui::widgets::Clear` + `Block`). The modal lists all
profile names from `App.config.profiles.keys()` as a scrollable list. The currently
active profile (`config.active_profile`) is marked with a `*` prefix. If no profiles
exist, the modal renders: `"No profiles configured. Edit config.json to add profiles."`.

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

### AC-005 (traces to BC-2.07.005 postcondition PC-1 — profile switch saves config atomically)
On `Enter` in the profile picker, the TUI:
1. Sets `config.active_profile = Some(selected_name.clone())`
2. Calls `config.save(MonocleConfig::config_path()?)` — atomic write via `tempfile::persist`
3. Sets `App.profile_picker = None` (closes picker)
4. Logs `INFO: profile switched to <name>`
5. Calls `detect_ccr(&config)` and logs `INFO: ccr_path resolved to <path>` if Some, or
   `WARN: ccr_path not found for profile <name>` if None.

### AC-006 (traces to BC-2.07.005 postcondition PC-2 — config save error display)
If `config.save()` returns `Err(ConfigError::IoError(e))`, the TUI renders a transient
error notification in the status bar: `"Error saving config: <e>"`. The profile is NOT
switched (the old `active_profile` is restored). `App.profile_picker = None` (picker closes).

### AC-007 (traces to BC-2.07.005 postcondition PC-3 — detect_ccr on switch)
After a successful profile switch, `detect_ccr(&config)` is called and its result is
stored in `App.ccr_path: Option<PathBuf>`. This value is used by subsequent sessions
panel rendering to display the active CCR path in the status bar footer
(e.g., `"CCR: ~/.claude/"` or `"CCR: none"`).

### AC-008 (traces to BC-2.07.004 invariant INV-1 — picker is not AppMode::Overlay)
The profile picker MUST NOT use `AppMode::Overlay`. It MUST be modeled as
`Option<ProfilePickerState>` in `App`. The reason: permission overlays (VecDeque<PromptModal>)
and the profile picker are orthogonal features; using `AppMode::Overlay` for the picker
would incorrectly conflate them with the permission overlay stack.

### AC-009 (traces to BC-2.07.005 invariant INV-1 — atomic write required)
`config.save()` MUST use `tempfile::persist` (from `monocle-config::MonocleConfig::save()`).
Any code path that calls `std::fs::write` directly for config is a forbidden pattern
and MUST trigger the `monocle-no-direct-config-write` semgrep rule.

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
      `{ selected_index: usize, profiles: Vec<String> }` — snapshot of profile names at open time
- [ ] Add `App.profile_picker: Option<ProfilePickerState>` field
- [ ] Add `App.ccr_path: Option<PathBuf>` field; set via `detect_ccr()` on startup and on profile switch
- [ ] Register `p` in `Global` binding layer → `Action::OpenProfilePicker`
- [ ] Implement `Action::OpenProfilePicker` handler: populate `ProfilePickerState` from `config.profiles.keys()`, sorted alphabetically; set `App.profile_picker = Some(state)`
- [ ] Implement picker keyboard handler (evaluated before `resolve_binding()` when picker is open):
      `j`/`↓` → increment selected_index (wrap), `k`/`↑` → decrement (wrap), `Enter` → select, `Esc` → close
- [ ] Implement profile switch on `Enter`: update `config.active_profile`, call `config.save()`,
      handle IoError with status bar notification, call `detect_ccr()`, update `App.ccr_path`
- [ ] Create `monocle-tui/src/ui/profile_picker_widget.rs` — centered modal list widget
      using `ratatui::widgets::Clear` + `Block` + `List`; mark active profile with `*`
- [ ] Wire profile picker widget into main render loop (rendered on top of layout when `profile_picker.is_some()`)
- [ ] Add `"CCR: <path>"` or `"CCR: none"` to status bar (extend S-027 status bar)
- [ ] Unit tests `monocle-tui/tests/profile_picker.rs` — open/close, navigation wrap, Enter selects,
      Esc closes without change, empty profiles message
- [ ] Unit tests `monocle-tui/tests/profile_switch.rs` — config save called on Enter, IoError displays
      notification, detect_ccr called after switch, active profile marked in list

## Previous Story Intelligence

S-030 (config foundation): `MonocleConfig::save()` is implemented and tested. The atomic
write via `tempfile::persist` is in `monocle-config`. The `detect_ccr()` function is also
from `monocle-config`. This story calls both from `monocle-tui/src/app.rs` via
`use monocle_config::{MonocleConfig, detect_ccr, ConfigError}`.

S-024 (TUI core types): `Action` enum needs a new variant `OpenProfilePicker`. Add it to
`monocle-core/src/tui/state.rs`. Because `Action` is `#[non_exhaustive]`, this is a
non-breaking addition — all existing `match action` arms with `_` catch-all continue to work.

S-025 (TUI skeleton): `App` struct is defined in `monocle-tui/src/app.rs`. Adding
`profile_picker: Option<ProfilePickerState>` and `ccr_path: Option<PathBuf>` extends
the existing struct without breaking its consumers.

## Architecture Compliance Rules

From `architecture/SS-config.md` and `architecture/SS-tui-core.md`:
- Profile picker is `Option<ProfilePickerState>` in `App` — NOT an `AppMode` variant,
  NOT `AppMode::Overlay`, NOT `AppMode::Fullscreen`
- All config writes via `monocle-config::MonocleConfig::save()` (which uses `tempfile::persist`)
  — never `std::fs::write` directly in `monocle-tui`
- `detect_ccr()` called after every successful profile switch — no stale CCR path
- Picker keyboard handler takes priority over `resolve_binding()` when picker is open —
  implement as a pre-check in the main event loop before calling `resolve_binding()`
- `Action::OpenProfilePicker` is added to `monocle-core` `Action` enum (it is `#[non_exhaustive]`)

**Forbidden Dependencies:**
- Direct `std::fs::write` for config — all writes through `MonocleConfig::save()`
- Profile picker as `AppMode::Overlay` — architectural violation, incorrect model
- `detect_ccr` in `monocle-tui` directly — use `monocle_config::detect_ccr()`

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| monocle-config | workspace path | `MonocleConfig::save()`, `detect_ccr()`, `ConfigError` |
| monocle-core | workspace path | `AppMode`, `Action::OpenProfilePicker`, `resolve_binding()` |
| ratatui | workspace pin | `Clear`, `Block`, `List`, `ListItem`, `ListState` for picker widget |
| tracing | 0.1 | INFO on profile switch, WARN if ccr not found |

## File Structure Requirements

Files to create:
- `monocle-tui/src/ui/profile_picker_widget.rs` — picker modal widget
- `monocle-tui/tests/profile_picker.rs` — picker open/close/navigation tests
- `monocle-tui/tests/profile_switch.rs` — config save and detect_ccr tests

Files to modify:
- `monocle-tui/src/app.rs` — add `ProfilePickerState`, `profile_picker: Option<ProfilePickerState>`,
  `ccr_path: Option<PathBuf>` fields; add picker keyboard handler; add `Action::OpenProfilePicker` dispatch
- `monocle-tui/src/ui/mod.rs` — declare `profile_picker_widget` module
- `monocle-tui/src/ui/status_bar.rs` (from S-027) — add CCR path display
- `monocle-core/src/tui/state.rs` (from S-024) — add `OpenProfilePicker` variant to `Action` enum

## Downstream Consumer Contract

No new public API produced by this story. The profile picker is an internal TUI feature.
This is a leaf story — nothing depends on it in the current phase.
