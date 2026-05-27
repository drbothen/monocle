---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.001: AppMode State Machine: Compile-Time Mutual Exclusion

## Description

`AppMode` is a Rust enum in `monocle-core` with four variants — `Dashboard`, `Filtering`,
`Overlay`, and `Fullscreen` — that provides compile-time mutual exclusion over which TUI
mode is active at any given time. The compiler enforces that exactly one mode is active;
there are no `Option<Panel>` fields and no `Arc<Mutex<...>>` in the transition path. All
state transitions are expressed as a pure total function
`fn transition(mode: AppMode, action: Action) -> AppMode` that lives in `monocle-core`
with no I/O, no ratatui dependency, and no runtime panics.

## Preconditions

1. The `monocle-core` crate compiles with `#[forbid(unsafe_code)]` in all SS-06 modules.
2. The `AppMode` enum is defined in `monocle-core/src/app_mode.rs` with exactly four
   variants: `Dashboard { focused: FocusSnapshot }`, `Filtering { panel: PanelId, query: String, prior: FocusSnapshot }`,
   `Overlay { stack: VecDeque<PromptModal>, prior: FocusSnapshot }`,
   `Fullscreen { panel: PanelId, prior: FocusSnapshot }`.
3. The `fn transition(mode: AppMode, action: Action) -> AppMode` function is defined in
   `monocle-core/src/transitions.rs` and is the only code path permitted to produce a new
   `AppMode` value in response to user `Action`s.
4. No `Option<Panel>` or `Option<AppMode>` fields exist anywhere in `AppMode` or in the
   `App` struct in `monocle-tui`.
5. No `Arc<Mutex<AppMode>>` or any shared-mutable-state wrapper is used in the state
   transition path.

## Postconditions

1. **Compile-time mutual exclusion enforced:** The Rust type system guarantees that code
   matching on `AppMode` must handle all four variants; missing-match-arm is a compile
   error. No runtime branch "which mode is active?" using boolean flags.
2. **Transition function is pure and total:** `transition(mode, action)` returns a new
   `AppMode` for every possible `(AppMode, Action)` pair. It never panics, never returns
   `Option`, and never touches I/O. All unrecognized `(mode, action)` pairs return the
   original `mode` unchanged (identity transition).
3. **`Overlay` variant never holds an empty `VecDeque`:** The transition function collapses
   `Overlay { stack: empty, prior }` → `Dashboard { focused: prior }` atomically. After
   any call to `transition`, if the returned `AppMode` is `Overlay`, its `stack` field
   contains at least one `PromptModal`.
4. **No `Arc<Mutex>` in transition path:** Kani proof harnesses (or equivalent property
   tests) can enumerate all reachable `(mode, action)` pairs without needing to acquire
   any lock.
5. **`monocle-core` SS-06 modules add zero I/O crate dependencies:** `AppMode`, `Action`,
   `FocusSnapshot`, `PanelId`, `PromptModal`, `BindingSource`, `Binding`, and `transition()`
   collectively introduce only `uuid 1.x` (for `PromptModal::prompt_id`, no I/O feature
   flags) and `serde` (already present) as new dependencies to `monocle-core`. No ratatui,
   crossterm, tokio, or similar I/O crates are imported by `monocle-core`.
6. **All new `monocle-core` SS-06 public types are `#[non_exhaustive]`:** `PanelId`,
   `FocusSnapshot`, `BindingSource`, `Action` are marked `#[non_exhaustive]` to allow
   Phase 2 panel additions without breaking `match` sites in `monocle-tui`.

## Invariants

1. At any point during TUI execution, exactly one `AppMode` variant is active. The type
   system enforces this; it cannot be violated at runtime.
2. The `prior: FocusSnapshot` field is always populated when entering `Overlay` or
   `Fullscreen` variants and is always restored when exiting back to `Dashboard`.
3. `transition()` is a deterministic function: the same `(mode, action)` pair always
   produces the same output `AppMode`. No hidden global state influences the result.
4. The `monocle-tui` crate calls `transition()` for every user-initiated action; it never
   mutates `app.mode` directly (i.e., no `app.mode = AppMode::Dashboard { ... }` outside
   the `transition()` call site). The only exception is the `PermissionPromptQueued` IPC
   push path, which calls `push_prompt()` directly on the `VecDeque` and then wraps in
   `Overlay { stack, prior }` — documented in BC-2.06.008.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-060 | `transition()` called with an `(Overlay, action)` pair where the resulting stack would be empty | Transition returns `Dashboard { focused: prior }`, not `Overlay { stack: vec![], prior }` — empty overlay state is impossible |
| EC-061 | `transition()` called with an `(mode, action)` pair where no arm matches (e.g., `Dashboard` + `PermissionAcceptOnce`) | Identity transition: returns the original `mode` unchanged; no panic, no error log |
| EC-062 | `monocle-core` compiled in a build environment where `VecDeque` is not in scope | Compile error (expected); `VecDeque` is `std::collections::VecDeque` and is always available in `std` |
| EC-063 | Match site in `monocle-tui` does not handle a new `AppMode` variant added in a future Phase 2 | Compile error (expected); `#[non_exhaustive]` on `AppMode` is NOT set because `AppMode` is not in the non-exhaustive list per constraint 6 (only `PanelId`, `FocusSnapshot`, `BindingSource`, `Action` are `#[non_exhaustive]`); `AppMode` variants require explicit handling |
| EC-064 | `transition()` called in a multithreaded context without synchronization | Safe: `transition()` takes ownership of `AppMode` (move semantics); Rust ownership prevents concurrent mutation without explicit wrapping |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Dashboard { focused: Sessions }`, `Action::FilterStart` | `Filtering { panel: Sessions, query: "", prior: Sessions }` | happy-path |
| `Filtering { panel: Sessions, query: "foo", prior: Sessions }`, `Action::Escape` | `Dashboard { focused: Sessions }` | happy-path |
| `Dashboard { focused: Sessions }`, `Action::Enter` | `Fullscreen { panel: Sessions, prior: Sessions }` | happy-path |
| `Fullscreen { panel: Sessions, prior: Sessions }`, `Action::Escape` | `Dashboard { focused: Sessions }` | happy-path |
| `Overlay { stack: [P1], prior: Sessions }`, `Action::PermissionAcceptOnce` | `Dashboard { focused: Sessions }` (stack collapses to empty → Dashboard) | edge-case |
| `Overlay { stack: [P1, P2], prior: Sessions }`, `Action::PermissionReject` | `Overlay { stack: [P2], prior: Sessions }` | happy-path |
| `Overlay { stack: [P1], prior: Sessions }`, `Action::Escape` | `Overlay { stack: [P1], prior: Sessions }` (Esc is no-op on stack in Overlay) | edge-case |
| `Dashboard { focused: Sessions }`, `Action::PermissionAcceptOnce` | `Dashboard { focused: Sessions }` (identity: no match arm) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `transition()` never returns `Overlay { stack: empty }` for any input | Kani proof harness (enumerate all `(AppMode, Action)` combinations) |
| VP-TBD | `transition()` never panics for any well-typed input | Kani proof harness |
| VP-TBD | `FocusSnapshot` captured in `prior` is always preserved through nested transitions | Kani proof harness |
| VP-TBD | `monocle-core` SS-06 modules have zero tokio/ratatui/crossterm imports | CI `cargo check` on `monocle-core` without those crates in `Cargo.toml` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC defines the AppMode enum and pure transition function that are the foundation of the TUI state machine, directly operationalizing the "AppMode state machine" component of CAP-006 |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — enforced here by the purity boundary: `transition()` is a pure function with no file I/O, and `monocle-core` has no I/O crate dependencies) |
| Architecture Module | monocle-core (AppMode, Action, FocusSnapshot, transition() — pure types); monocle-tui (App struct, draw loop, dispatcher) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.0.0 §AppMode State Machine; §Purity Boundary; §Constraints |
| Cross-Ref | BC-2.06.002 (FocusSnapshot restore, depends on this), BC-2.06.003 (5-level dispatch, depends on this), BC-2.06.008 (overlay push, depends on this) |
| Test File | `monocle-core/tests/app_mode_transitions.rs` |
| Test Name | `test_BC_2_06_001_appmode_compile_time_mutual_exclusion` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.002] — composes with: FocusSnapshot capture/restore is a direct postcondition of the transition function defined here
- [BC-2.06.003] — depends on: the 5-level dispatcher resolves a `KeyEvent` to an `Action`, which is then passed to `transition()` defined here
- [BC-2.06.008] — composes with: the overlay push path (IPC-driven, not via `transition()`) is the defined exception to the rule that all mode changes flow through `transition()`

## Architecture Anchors

- `architecture/SS-tui.md#appmode-state-machine` — full enum definition and transition function implementation
- `architecture/SS-tui.md#purity-boundary` — purity boundary table listing `monocle-core` vs `monocle-tui` separation
- `architecture/SS-tui.md#constraints` — constraints 1–5 directly relevant to this BC

## Story Anchor

S-TBD — Implement AppMode enum, Action enum, FocusSnapshot, PanelId, PromptModal, BindingSource in monocle-core with pure transition() function (filled by story-writer)

## VP Anchors

- VP-TBD — Kani proof harness for transition() purity and non-empty Overlay invariant

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.001 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.0.0 §AppMode State Machine, §Purity Boundary, §Constraints,
  §Rendering Architecture; prd-expansion-scope.md §3.3 BC-2.06.001 description;
  ARCH-INDEX.md §Capability Traceability SS-06.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: purity boundary eliminates all file I/O from monocle-core, satisfying the
  invariant that monocle must not write to harness-owned files.
- EC-063 corrects a subtle design decision: AppMode itself is NOT #[non_exhaustive] because
  all match sites in monocle-tui must handle all variants explicitly. Only PanelId,
  FocusSnapshot, BindingSource, Action are non_exhaustive per SS-tui.md §Constraints item 6.
