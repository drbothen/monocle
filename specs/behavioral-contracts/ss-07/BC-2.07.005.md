---
document_type: behavioral-contract
level: L3
version: "1.3.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T00:00:00Z
phase: phase-1-expansion
inputs:
  - {path: .factory/specs/architecture/SS-config.md, version: "1.0.0"}
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-07
capability: CAP-007
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D-003, F-P1D2-003, F-P1D2-010, F-P1D3-004]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.07.005: Profile Picker: `Ctrl-P` Override Shows Picker

## Description

Pressing `Ctrl-P` in any TUI `AppMode` opens the profile picker overlay, regardless
of whether a sticky profile selection exists for the current project directory. The
picker is rendered as a modal overlay over the current TUI view. When the user selects
a profile, the selection is persisted atomically to `config.json` under the current
directory's entry in `project_profiles`, and the new profile is applied to the running
session context. The `monocle-tui` crate owns the picker rendering; `monocle-config`
owns the read/write contract for the persistence step.

## Preconditions

1. The TUI is running and the user presses `Ctrl-P` (mapped to `Action::ProfilePicker`
   in any `AppMode`).
2. `load_config()` has previously returned a config with `harness_profiles` populated.
   If `harness_profiles` is empty, the picker opens but renders an empty list with a
   "No profiles configured — add profiles to config.json" message.
3. The current project directory path is available (same path resolution as BC-2.07.004
   Postcondition 1 — verbatim, not symlink-resolved).
4. The `monocle-config` write path (BC-2.07.001 `write_config`) is available to the
   picker's persistence logic.

## Postconditions

**Picker display:**
1. On `Action::ProfilePicker`, the TUI sets `app.picker = Some(ProfilePickerState { ... })`
   without changing `AppMode`. The profile picker is a transient overlay managed by
   `Option<ProfilePickerState>` in the `App` struct, separate from the `AppMode` state machine.
   It can appear over any `AppMode` (Dashboard, Fullscreen, Overlay) without creating a new
   `AppMode` variant. The profile picker widget is rendered as a modal over the current view.
2. The picker displays all profiles from `config.harness_profiles` as selectable rows.
   Each row shows `display_name` (and optionally `binary_path`).
3. If `harness_profiles` is empty, the picker renders: "No profiles configured — add
   profiles to config.json". The user can dismiss with `Esc`.
4. The picker highlights the currently active profile (the sticky entry for this directory,
   if any) as the default selection. If no sticky entry exists, the first profile in the
   list is highlighted.

**Selection and persistence:**
5. When the user confirms a selection (e.g., `Enter` keypress on a highlighted row):
   a. The selected profile's `id` is written into `config.project_profiles[current_dir]`.
   b. `write_config(&updated_config)` is called via the atomic write path (BC-2.07.001).
   c. If `write_config` returns `Err`, the TUI renders a transient error notification:
      "Config save failed: <error>". The in-memory active profile is updated regardless
      of the write failure (the user's pick takes effect for this session even if
      persistence fails).
6. The new profile is applied to the running session context. The daemon reloads the
   active harness profile from config on each new session launch (not at daemon startup),
   so the change takes effect for new sessions without daemon restart.
7. The picker overlay is dismissed after selection by setting `app.picker = None`. The
   underlying `AppMode` is unchanged throughout the picker lifecycle and requires no
   restoration.

**Dismissal without selection:**
8. If the user presses `Esc` without selecting a profile, the picker overlay is dismissed
   by setting `app.picker = None` without any change to `config.project_profiles` and without
   calling `write_config`. The underlying `AppMode` is unchanged.

**AppMode exclusivity:**
9. While the profile picker overlay is active, keybindings for other AppModes (e.g.,
   `Action::PermissionAcceptOnce`, `Action::ScrollUp`) are suppressed. Only picker-local
   keybindings are active: navigate (`↑`/`↓`), confirm (`Enter`), cancel (`Esc`).

## Invariants

1. `Ctrl-P` opens the profile picker in ALL AppModes including `Overlay` (permission prompt
   active), `Filtering`, and `Fullscreen`. There is no AppMode that suppresses `Ctrl-P`.
2. The profile picker write step uses `write_config` exclusively (BC-2.07.001). No direct
   `std::fs::write` is used for the persistence step.
3. In-memory active profile update (Postcondition 5c) is decoupled from write success.
   A failed write does not prevent the new profile from being used in the current session.
4. The picker is modeled as `Option<ProfilePickerState>` in the `App` struct, separate from
   the `AppMode` state machine. It does not create a new `AppMode` variant and does not use
   the existing `Overlay` variant. It can appear over any `AppMode` (Dashboard, Fullscreen,
   Overlay) and dismisses by setting `app.picker = None` without changing the underlying
   `AppMode`. This design keeps `AppMode` exhaustiveness unaffected by the picker and avoids
   nested-Overlay complexity.
5. The `current_dir` key used for `project_profiles` write (Postcondition 5a) uses the
   same path string as the read path (BC-2.07.004 Postcondition 1). Identical normalization
   at write time and read time is required for the sticky selection to work.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-106 | `Ctrl-P` pressed when `harness_profiles` is empty | Picker opens; renders "No profiles configured" message; `Esc` dismisses; no write |
| EC-107 | `Ctrl-P` pressed while permission overlay is active (AppMode = Overlay with PromptModal) | `app.picker = Some(...)` set; AppMode remains Overlay; picker renders over the permission overlay; permission prompts remain queued; picker dismiss sets `app.picker = None`; AppMode (Overlay) unchanged; permission overlay remains visible |
| EC-108 | User selects same profile as current sticky entry | `write_config` is still called (idempotent); no error; picker dismisses normally |
| EC-109 | `write_config` fails during picker selection persistence (e.g., disk full) | In-memory profile updated; TUI shows transient "Config save failed: <error>" notification; no crash; picker dismissed |
| EC-110 | User presses `Ctrl-P` twice (picker already open) | Second `Ctrl-P` is a no-op when `app.picker` is already `Some(...)`. Only one picker instance is active at a time; the second keypress does not replace or re-open the picker. |
| EC-111 | Profile selected via picker has `binary_path` that does not exist | Picker accepts the selection (monocle-config does not validate binary existence at persistence time); error surfaces at engine spawn time when the session is launched |
| EC-112 | `harness_profiles` updated by an external edit to config.json while picker is open | Picker uses the in-memory snapshot from TUI startup; newly added external profile is NOT visible in this picker invocation; it appears on next TUI launch or daemon restart |
| EC-113 | `Esc` pressed after navigating the picker but before confirming | Picker dismissed; `project_profiles` unchanged; prior AppMode restored |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Ctrl-P` in `Dashboard` mode with 2 profiles | `app.picker = Some(ProfilePickerState { ... })`; picker widget renders as modal; AppMode remains Dashboard; 2 profile rows rendered | happy-path |
| User presses `↓` then `Enter` on second profile | Second profile `id` written to `project_profiles[current_dir]`; `write_config` called; `app.picker = None`; AppMode unchanged | happy-path |
| User presses `Esc` after navigating | `app.picker = None`; `project_profiles` unchanged; no `write_config` call; AppMode unchanged | happy-path |
| `write_config` returns `Err` | In-memory profile updated; error notification rendered; no crash | error |
| `Ctrl-P` with empty `harness_profiles` | Picker opens; "No profiles configured" rendered; `Esc` dismisses | edge-case |
| `Ctrl-P` while permission overlay is showing | `app.picker = Some(...)` set; AppMode remains Overlay (permission prompts remain queued); picker modal renders over permission overlay; Esc sets `app.picker = None`; AppMode (Overlay) unchanged | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `Ctrl-P` sets `app.picker = Some(ProfilePickerState { ... })` without changing AppMode | integration (synthetic key event; assert picker field populated; assert AppMode unchanged) |
| VP-TBD | Profile selection triggers `write_config` with updated `project_profiles` | integration (intercept write_config call; assert new entry) |
| VP-TBD | `Esc` sets `app.picker = None` without calling `write_config` and without changing AppMode | integration (assert picker field None after Esc; assert no write_config call) |
| VP-TBD | Failed `write_config` does not crash TUI; error notification rendered | integration (mock write_config to return Err; assert TUI continues) |
| VP-TBD | `Ctrl-P` sets `app.picker = Some(...)` in all AppMode variants without changing AppMode | integration (parametrize over AppMode variants; assert picker populated and AppMode unchanged) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability §SS-07 |
| Capability Anchor Justification | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability — this BC specifies the Ctrl-P override that is the user-initiated profile management action, including the persistence contract that makes profile changes durable |
| L2 Domain Invariants | No domain-spec/invariants.md exists for this project; authority is ARCH-INDEX §SS-07 and SS-config.md §Profile Picker Logic §Ctrl-P Override |
| Architecture Module | monocle-tui (picker rendering, AppMode transition); monocle-config (write_config persistence) per ARCH-INDEX Subsystem Registry SS-06 and SS-07 |
| Architecture Source | SS-config.md v1.4.0 §Profile Picker Logic §Ctrl-P Override (BC-2.07.005) |
| Cross-Ref | BC-2.07.001 (write_config atomic write used by persistence step); BC-2.07.002 (project_profiles field schema); BC-2.07.004 (sticky read that consumes the profile ID written here); BC-2.06.003 (Action dispatch 5-level precedence that routes Ctrl-P to ProfilePicker) |
| Brief Features | F-57 (profile picker Ctrl-P override and persistence) |
| Test File | `monocle-tui/tests/profile_picker_overlay.rs` |
| Test Name | `test_BC_2_07_005_ctrl_p_opens_picker_and_persists` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.07.001] — depends on: write_config (atomic write) is used for the selection persistence step
- [BC-2.07.002] — depends on: project_profiles schema field is the write target
- [BC-2.07.004] — composes with: sticky read (BC-2.07.004) consumes the project_profiles entry written here
- [BC-2.06.003] — depends on: Action dispatch must route Ctrl-P to ProfilePicker in all AppModes

## Architecture Anchors

- `architecture/SS-config.md#profile-picker-logic` — Ctrl-P override algorithm, separation of concerns (monocle-tui owns rendering, monocle-config owns persistence)
- `architecture/SS-config.md#atomic-write-contract` — write_config used in persistence step

## Story Anchor

S-TBD — Implement profile picker: sticky-per-project and Ctrl-P override (filled by story-writer)

## VP Anchors

VP-TBD — profile picker overlay integration tests (filled after VP creation)

## §Trace v1.3.2

**Architecture Source pin cascade: SS-config.md v1.3.0→v1.4.0** (2026-06-20):
- Architecture Source: `SS-config.md v1.3.0` → `SS-config.md v1.4.0`.
- No body propagation required: §Profile Picker Logic §Ctrl-P Override is unchanged in v1.4.0.
- SE-16d monotonicity: 2026-06-20 > v1.3.1 date 2026-05-29. PASS.

## §Trace v1.3.1

**F-S025-ADV23-MED-001 Category 8 sweep — Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-config.md v1.1.0` → `SS-config.md v1.3.0` (active pointer was stale by 2 minor versions).
- No substantive BC body prose propagation required: §Profile Picker Logic §Ctrl-P Override algorithm is unchanged in v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.3.1 timestamp 2026-05-29T00:00:00Z > v1.3.0 timestamp 2026-05-26T14:00:00Z. PASS.

## §Trace v1.3.0

**F-P1D3-004 HIGH — Profile picker AppMode model corrected to Option<ProfilePickerState>** (2026-05-26T14:00:00Z):
- Architect decision: the profile picker is a transient overlay managed by
  `Option<ProfilePickerState>` in the `App` struct, NOT via AppMode. It does not use the
  `Overlay` variant and does not create a new AppMode state.
- Postcondition 1: "TUI transitions to `Overlay { prior: ... }`" → "TUI sets `app.picker =
  Some(ProfilePickerState { ... })` without changing AppMode".
- Postcondition 7: "AppMode restores to the `prior` snapshot" → "`app.picker = None`; AppMode
  unchanged".
- Postcondition 8: same restoration language corrected to `app.picker = None`.
- Invariant 4: "Uses the existing `Overlay` variant" → "Modeled as `Option<ProfilePickerState>`
  in App struct, separate from AppMode; does not create or use AppMode variant."
- Edge cases EC-107, EC-110: updated to reflect `app.picker` state management, not AppMode.
- Canonical test vectors: updated all rows mentioning "prior AppMode saved/restored" to
  reflect `app.picker` lifecycle.
- Verification properties: updated VP-1, VP-3, VP-5 to assert on `app.picker` field, not
  AppMode transitions.
- SE-16d monotonicity: v1.3.0 timestamp 2026-05-26T14:00:00Z >= v1.2.0. PASS.

## §Trace v1.2.0

**F-P1D2-003 HIGH — Remaining `OpenProfilePicker` residuals removed** (2026-05-26T00:00:00Z):
- Traceability §Cross-Ref BC-2.06.003 note: "OpenProfilePicker" → "ProfilePicker" per F-P1D2-003.
- Related BCs [BC-2.06.003] note: "OpenProfilePicker" → "ProfilePicker" per F-P1D2-003.
- These two occurrences were missed in F-P1D-003 (v1.1.0) which fixed Preconditions, Postconditions, VP, and the first Cross-Ref occurrence but left these two Traceability/Related BCs references unchanged.
- Architecture Source: `SS-config.md v1.0.0` → `SS-config.md v1.1.0` per F-P1D2-010.

SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.1.0

**F-P1D-003 CRITICAL — Action variant name corrected** (2026-05-26T00:00:00Z):
- All occurrences of `Action::OpenProfilePicker` replaced with `Action::ProfilePicker`
  per F-P1D-003 from Phase 1d Pass 1 adversarial review.
- Canonical enum in SS-tui.md defines the variant as `ProfilePicker`, not `OpenProfilePicker`.
- Affected locations: Preconditions §1, Postconditions §1, Verification Properties §VP-1,
  Traceability §Cross-Ref BC-2.06.003 note.
- SE-16d monotonicity: v1.1.0 timestamp >= v1.0.0. PASS.

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- Created as new artifact for SS-07 (Config subsystem) per `prd-expansion-scope.md` §3.4 BC-2.07.005
  and `SS-config.md` §Profile Picker Logic §Ctrl-P Override.
- Separation of concerns documented: monocle-tui owns rendering; monocle-config owns write.
- Decoupled in-memory apply from write success (write failure is non-fatal per design).
- Brief feature traced: F-57.
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact; no prior chain).
