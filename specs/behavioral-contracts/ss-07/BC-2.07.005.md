---
document_type: behavioral-contract
level: L3
version: "1.0.0"
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
modified: []
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

1. The TUI is running and the user presses `Ctrl-P` (mapped to `Action::OpenProfilePicker`
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
1. On `Action::OpenProfilePicker`, the TUI transitions to `Overlay { prior: <current AppMode snapshot> }`
   (or equivalent picker-specific mode). The profile picker widget is rendered as a modal.
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
7. The picker overlay is dismissed after selection. AppMode restores to the `prior`
   snapshot captured in Postcondition 1.

**Dismissal without selection:**
8. If the user presses `Esc` without selecting a profile, the picker overlay is dismissed
   without any change to `config.project_profiles` and without calling `write_config`.
   AppMode restores to the prior snapshot.

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
4. The picker does not create a new `AppMode` variant. It uses the existing `Overlay`
   variant (or a flag on the existing Overlay state); it does not introduce a new state
   machine state. This is an architectural constraint to maintain compile-time exhaustiveness
   of `AppMode` pattern matches.
5. The `current_dir` key used for `project_profiles` write (Postcondition 5a) uses the
   same path string as the read path (BC-2.07.004 Postcondition 1). Identical normalization
   at write time and read time is required for the sticky selection to work.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-106 | `Ctrl-P` pressed when `harness_profiles` is empty | Picker opens; renders "No profiles configured" message; `Esc` dismisses; no write |
| EC-107 | `Ctrl-P` pressed while permission overlay is active (AppMode = Overlay with PromptModal) | Picker opens over the permission overlay (stacked); permission prompts remain queued; picker dismiss restores back to permission overlay |
| EC-108 | User selects same profile as current sticky entry | `write_config` is still called (idempotent); no error; picker dismisses normally |
| EC-109 | `write_config` fails during picker selection persistence (e.g., disk full) | In-memory profile updated; TUI shows transient "Config save failed: <error>" notification; no crash; picker dismissed |
| EC-110 | User presses `Ctrl-P` twice (picker already open) | Second `Ctrl-P` is suppressed or treated as a no-op while picker is already in Overlay; only one picker instance is active at a time |
| EC-111 | Profile selected via picker has `binary_path` that does not exist | Picker accepts the selection (monocle-config does not validate binary existence at persistence time); error surfaces at engine spawn time when the session is launched |
| EC-112 | `harness_profiles` updated by an external edit to config.json while picker is open | Picker uses the in-memory snapshot from TUI startup; newly added external profile is NOT visible in this picker invocation; it appears on next TUI launch or daemon restart |
| EC-113 | `Esc` pressed after navigating the picker but before confirming | Picker dismissed; `project_profiles` unchanged; prior AppMode restored |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Ctrl-P` in `Dashboard` mode with 2 profiles | Picker overlay opens; 2 rows rendered; prior AppMode saved | happy-path |
| User presses `↓` then `Enter` on second profile | Second profile `id` written to `project_profiles[current_dir]`; `write_config` called; picker dismissed; Dashboard restored | happy-path |
| User presses `Esc` after navigating | Picker dismissed; `project_profiles` unchanged; no `write_config` call | happy-path |
| `write_config` returns `Err` | In-memory profile updated; error notification rendered; no crash | error |
| `Ctrl-P` with empty `harness_profiles` | Picker opens; "No profiles configured" rendered; `Esc` dismisses | edge-case |
| `Ctrl-P` while permission overlay is showing | Picker overlaid on top; permission prompts remain; Esc restores permission overlay | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `Ctrl-P` in Dashboard mode opens profile picker widget | integration (synthetic key event; assert picker widget in render frame) |
| VP-TBD | Profile selection triggers `write_config` with updated `project_profiles` | integration (intercept write_config call; assert new entry) |
| VP-TBD | `Esc` dismisses picker without calling `write_config` | integration (assert no write_config call after Esc) |
| VP-TBD | Failed `write_config` does not crash TUI; error notification rendered | integration (mock write_config to return Err; assert TUI continues) |
| VP-TBD | `Ctrl-P` opens picker in all AppMode variants | integration (parametrize over AppMode variants) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability §SS-07 |
| Capability Anchor Justification | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability — this BC specifies the Ctrl-P override that is the user-initiated profile management action, including the persistence contract that makes profile changes durable |
| L2 Domain Invariants | No domain-spec/invariants.md exists for this project; authority is ARCH-INDEX §SS-07 and SS-config.md §Profile Picker Logic §Ctrl-P Override |
| Architecture Module | monocle-tui (picker rendering, AppMode transition); monocle-config (write_config persistence) per ARCH-INDEX Subsystem Registry SS-06 and SS-07 |
| Architecture Source | SS-config.md v1.0.0 §Profile Picker Logic §Ctrl-P Override (BC-2.07.005) |
| Cross-Ref | BC-2.07.001 (write_config atomic write used by persistence step); BC-2.07.002 (project_profiles field schema); BC-2.07.004 (sticky read that consumes the profile ID written here); BC-2.06.003 (Action dispatch 5-level precedence that routes Ctrl-P to OpenProfilePicker) |
| Brief Features | F-57 (profile picker Ctrl-P override and persistence) |
| Test File | `monocle-tui/tests/profile_picker_overlay.rs` |
| Test Name | `test_BC_2_07_005_ctrl_p_opens_picker_and_persists` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.07.001] — depends on: write_config (atomic write) is used for the selection persistence step
- [BC-2.07.002] — depends on: project_profiles schema field is the write target
- [BC-2.07.004] — composes with: sticky read (BC-2.07.004) consumes the project_profiles entry written here
- [BC-2.06.003] — depends on: Action dispatch must route Ctrl-P to OpenProfilePicker in all AppModes

## Architecture Anchors

- `architecture/SS-config.md#profile-picker-logic` — Ctrl-P override algorithm, separation of concerns (monocle-tui owns rendering, monocle-config owns persistence)
- `architecture/SS-config.md#atomic-write-contract` — write_config used in persistence step

## Story Anchor

S-TBD — Implement profile picker: sticky-per-project and Ctrl-P override (filled by story-writer)

## VP Anchors

VP-TBD — profile picker overlay integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- Created as new artifact for SS-07 (Config subsystem) per `prd-expansion-scope.md` §3.4 BC-2.07.005
  and `SS-config.md` §Profile Picker Logic §Ctrl-P Override.
- Separation of concerns documented: monocle-tui owns rendering; monocle-config owns write.
- Decoupled in-memory apply from write success (write failure is non-fatal per design).
- Brief feature traced: F-57.
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact; no prior chain).
