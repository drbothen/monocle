---
document_type: behavioral-contract
level: L3
version: "1.0.3"
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
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.07.004: Profile Picker: Sticky-Per-Project

## Description

When monocle starts in TUI mode, it determines the active harness profile without
requiring user interaction for projects the user has previously configured. The
`config.project_profiles` map stores the last-selected profile ID keyed by the
project's absolute directory path. If a matching entry exists, monocle loads that
profile and proceeds directly to the TUI without showing the profile picker overlay.
This zero-friction re-entry path is the primary UX mechanism ensuring users are not
interrupted by the picker on every launch.

## Preconditions

1. `load_config()` has been called and returned `Ok(config)` or `Ok(MonocleConfig::default())`.
2. The current working directory (or the project path argument if monocle was launched
   with an explicit path) is available as an `absolute path` resolved via `std::fs::canonicalize`
   or `std::env::current_dir`.
3. `config.harness_profiles` is the list of available profiles.

## Postconditions

**Step 1 — Resolve the project directory:**
1. The project directory used for lookup is the absolute path of the current working directory
   at TUI launch time (or the explicit `--project` argument if provided). Symlinks are NOT
   resolved before lookup: the path used during the previous selection is stored verbatim in
   `project_profiles`, so the same symlink path must be used for consistency.

**Step 2 — Look up the sticky profile:**
2. The resolved directory path string is used as the key into `config.project_profiles`.
3. If `config.project_profiles` contains the key, the associated value is a profile ID string.
4. The profile ID is matched against `config.harness_profiles` by the `id` field.

**Step 3 — Decision branch:**
5. **Sticky match found and profile exists in harness_profiles:** monocle loads that profile
   and enters the TUI in `Dashboard` AppMode. The profile picker overlay is NOT shown.
6. **Sticky entry found but profile ID is dangling (not in harness_profiles):** monocle
   treats this as "no match" (the profile was deleted). The profile picker is shown.
   The dangling entry is NOT immediately removed from `project_profiles` (a future write
   via BC-2.07.005 overwrites it on next selection).
7. **No sticky entry for this directory AND harness_profiles is non-empty:** the profile
   picker overlay is shown. The user selects a profile; the selection is persisted per
   BC-2.07.005 Postcondition step 1-2.
8. **No sticky entry AND harness_profiles is empty:** the TUI enters `Dashboard` AppMode
   with a "No profiles configured" notice rendered in the sessions panel area. The picker
   overlay is NOT shown (there is nothing to pick).

**Persistence contract for `resolve_profile_for_dir`:**
9. The lookup function `resolve_profile_for_dir(config: &MonocleConfig, dir: &str) -> Option<&HarnessProfile>`
   is pure (no I/O, no side effects). It takes the current config and directory string and
   returns the matching profile, if any. It belongs in the pure-core classification per
   SS-config.md §Purity Boundary.

## Invariants

1. The sticky selection map is keyed by the verbatim directory path string used during
   the last picker selection for that directory. Path normalization (canonicalization,
   trailing-slash stripping) is performed once at lookup time and must be identical to
   the normalization performed at write time (BC-2.07.005 Postcondition 1). Inconsistent
   normalization between write and read is a correctness defect.
2. `resolve_profile_for_dir` has no side effects. It does not write to the config,
   does not log, and does not perform filesystem I/O.
3. A dangling `project_profiles` entry does NOT cause a panic or error. It is silently
   treated as "no match" per Postcondition 6.
4. The profile picker is shown at most once per launch session. If no sticky profile is
   found, the picker is shown; once the user selects, the TUI enters Dashboard mode and
   the picker does not reappear unless `Ctrl-P` is pressed (BC-2.07.005).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-097 | First ever launch — no config.json exists, no project_profiles entries | Config loads as default; `harness_profiles` is empty; TUI renders "No profiles configured" notice; no picker shown |
| EC-098 | First launch for this project, but other projects have sticky entries | No entry for this dir; `harness_profiles` non-empty; picker shown; user selects; selection saved per BC-2.07.005 |
| EC-099 | Sticky profile ID matches a profile in harness_profiles | Picker skipped; Dashboard entered with that profile active |
| EC-100 | Sticky profile ID references a deleted profile (dangling reference) | Treated as no match; picker shown; user re-selects; new selection overwrites dangling entry |
| EC-101 | Project directory path contains symlink components | Path is NOT resolved through symlinks; verbatim path used; must match verbatim path stored at write time |
| EC-102 | Two projects with different real paths but same symlink-resolved path | Treated as two distinct entries in project_profiles (each under their own symlink path); no collision |
| EC-103 | `harness_profiles` has one entry; this dir has no sticky entry | Picker is shown with one option; user selects; stored; TUI enters Dashboard |
| EC-104 | Config parse fails at startup → `MonocleConfig::default()` used → `harness_profiles` empty | "No profiles configured" notice shown; no panic |
| EC-105 | `project_profiles` key for this dir maps to an empty string `""` | Empty string does not match any profile `id` (all valid IDs are non-empty); treated as dangling; picker shown |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `resolve_profile_for_dir(&default_config, "/home/user/project")` | `None` (no profiles, no entry) | happy-path |
| Config with one profile `id:"cc"` and `project_profiles:{"/home/user/project":"cc"}`; dir is `/home/user/project` | `Some(&profile_with_id_cc)` returned | happy-path |
| Config with one profile `id:"cc"` and `project_profiles:{"/home/user/project":"old-profile"}`; dir is `/home/user/project` | `None` (dangling ID `old-profile` not in harness_profiles) | edge-case |
| Config with one profile and no `project_profiles` entry for current dir | `None`; picker shown | happy-path |
| Dir path `/home/user/project/` (trailing slash) vs stored `/home/user/project` | Miss (different strings); demonstrates why normalization must be consistent | invariant |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `resolve_profile_for_dir` returns `Some` for matching ID | unit (construct config; call function; assert) |
| VP-TBD | `resolve_profile_for_dir` returns `None` for absent entry | unit |
| VP-TBD | `resolve_profile_for_dir` returns `None` for dangling profile ID (not in harness_profiles) | unit |
| VP-TBD | TUI startup with matching sticky entry: picker overlay NOT rendered | integration (mock IPC; assert no picker widget in render output) |
| VP-TBD | TUI startup with no sticky entry and non-empty harness_profiles: picker shown | integration |
| VP-TBD | TUI startup with empty harness_profiles: "No profiles configured" notice rendered | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability §SS-07 |
| Capability Anchor Justification | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability — this BC specifies the sticky-per-project behavior that is the primary UX mechanism for harness profile management |
| L2 Domain Invariants | No domain-spec/invariants.md exists for this project; authority is ARCH-INDEX §SS-07 and SS-config.md §Profile Picker Logic |
| Architecture Module | monocle-config (config.json reader/writer, harness profile schema, profile picker logic) per ARCH-INDEX Subsystem Registry SS-07; also monocle-tui (picker rendering) |
| Architecture Source | SS-config.md v1.4.0 §Profile Picker Logic §Sticky-Per-Project Selection (BC-2.07.004) |
| Cross-Ref | BC-2.07.002 (project_profiles field in schema); BC-2.07.003 (config load path that provides config to this logic); BC-2.07.005 (Ctrl-P override and persistence on selection) |
| Brief Features | F-57 (profile picker sticky-per-project) |
| Test File | `monocle-config/tests/profile_picker.rs` |
| Test Name | `test_BC_2_07_004_sticky_profile_selection` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.07.002] — depends on: project_profiles schema field defined in BC-2.07.002
- [BC-2.07.003] — depends on: config must be loaded (possibly as default) before resolve_profile_for_dir runs
- [BC-2.07.005] — composes with: Ctrl-P override shows picker and persists selection that this BC later reads

## Architecture Anchors

- `architecture/SS-config.md#profile-picker-logic` — sticky-per-project algorithm, symlink policy
- `architecture/SS-config.md#purity-boundary` — resolve_profile_for_dir classified as pure-core

## Story Anchor

S-TBD — Implement profile picker: sticky-per-project and Ctrl-P override (filled by story-writer)

## VP Anchors

VP-TBD — profile picker sticky logic unit and integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- Created as new artifact for SS-07 (Config subsystem) per `prd-expansion-scope.md` §3.4 BC-2.07.004
  and `SS-config.md` §Profile Picker Logic §Sticky-Per-Project Selection.
- Eight-step postcondition covers all four decision branches (sticky match, dangling, no entry
  with profiles, no entry without profiles).
- Brief feature traced: F-57.
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact; no prior chain).


## §Trace v1.0.3

**Architecture Source pin cascade: SS-config.md v1.3.0→v1.4.0** (2026-06-20):
- Architecture Source: `SS-config.md v1.3.0` → `SS-config.md v1.4.0`.
- No body propagation required: §Profile Picker Logic §Sticky-Per-Project Selection is unchanged in v1.4.0.
- SE-16d monotonicity: 2026-06-20 > v1.0.2 date 2026-05-29. PASS.

## §Trace v1.0.2

**F-S025-ADV23-MED-001 Category 8 sweep — Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-config.md v1.1.0` → `SS-config.md v1.3.0` (active pointer was stale by 2 minor versions).
- No substantive BC body prose propagation required: §Profile Picker Logic §Sticky-Per-Project Selection algorithm is unchanged in v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.0.2 timestamp 2026-05-29T00:00:00Z > v1.0.1. PASS.

## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-config.md v1.0.0` → `SS-config.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.