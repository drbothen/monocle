---
document_type: story
level: L4
story_id: S-024
epic_id: EPIC-06
version: "1.4"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-28T16:00:00Z
phase: 2
points: 8
wave: 4
tdd_mode: strict
priority: P0
depends_on: [S-011, S-014]
blocks: [S-025, S-026, S-031]
target_module: monocle-core
subsystems: [SS-06]
behavioral_contracts: [BC-2.06.001, BC-2.06.002, BC-2.06.003]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.001.md, version: "1.0.4"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.002.md, version: "1.0.4"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.003.md, version: "1.0.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.06.001 (AppMode state machine), BC-2.06.002 (FocusSnapshot: focus restored after overlay/fullscreen close), BC-2.06.003 (Action dispatch: 5-level binding precedence + transition() pure function)"
---

# S-024: TUI Core Types — AppMode, Action, FocusSnapshot, transition(), 5-Level Dispatch

## Narrative

As a TUI implementer, I want the `monocle-core` crate to define `AppMode`, `Action`,
`FocusSnapshot`, `BindingSource`, and a pure `transition()` function, so that all TUI
mode transitions, binding dispatch, and overlay lifecycle are governed by a single
correct, tested state machine with no I/O dependencies.

## Acceptance Criteria

### AC-001 (traces to BC-2.06.001 postcondition PC-1 — AppMode enum definition)
`monocle-core` exports `AppMode` as a non-exhaustive-exempt enum (AppMode is NOT
`#[non_exhaustive]`) with exactly four variants:
- `Dashboard { focused: FocusSnapshot }`
- `Filtering { panel: PanelId, query: String, prior: FocusSnapshot }`
- `Overlay { prior: FocusSnapshot }` (modal stack is carried in `App.overlay_stack: VecDeque<PromptModal>`, not in this variant)
- `Fullscreen { panel: PanelId, prior: FocusSnapshot }`

### AC-002 (traces to BC-2.06.002 precondition 1 — FocusSnapshot enum definition)
`FocusSnapshot` is a `#[non_exhaustive]` enum with at least two variants: `Sessions` and
`EventRibbon`. Phase 2+ panels (`Customizations`, `Workflow`, `Preview`) extend this enum
without breaking existing match arms (enforced by `#[non_exhaustive]`). `FocusSnapshot`
derives `Clone`, `PartialEq`, `Eq`, and `Debug`. `PanelId` is a separate `#[non_exhaustive]`
enum with variants `Sessions`, `EventRibbon`, `StaticExplorer`, `WorkflowPanel`, `HarnessPanel`.

`FocusSnapshot` also implements:
- `FocusSnapshot::cycle()` — pure method advancing focus to next panel in round-robin order;
  single-panel cycle is idempotent (returns same variant).
- `FocusSnapshot::to_panel_id()` — pure method converting a `FocusSnapshot` variant to the
  corresponding `PanelId`.

### AC-003 (traces to BC-2.06.001 postcondition PC-3 — PromptModal fields)
`PromptModal` has fields: `prompt_id: Uuid`, `session_id: String`, `tool_name: String`,
`tool_payload: ToolPayload`, `received_at: std::time::Instant`. `ToolPayload` has
variants: `Edit { old_content: String, new_content: String, path: PathBuf }`,
`Bash { command: String }`, `Read { path: PathBuf }`,
`Generic { tool_name: String, tool_input: serde_json::Value }`.

### AC-004 (traces to BC-2.06.003 postcondition PC-1 — transition() pure function)
`transition(mode: AppMode, action: Action) -> AppMode` is a pure function in
`monocle-core::tui::state` (zero I/O, zero side effects). All `AppMode` changes
in the TUI (except the PermissionPromptQueued IPC push path and BC-2.06.023 UUID
removal path) flow through `transition()`.

### AC-005 (traces to BC-2.06.003 postcondition PC-2 — empty-stack collapse invariant)
When `transition()` would produce `Overlay { prior }` while `App.overlay_stack` is empty,
it MUST instead return `Dashboard { focused: prior }`. This invariant is enforced
inside `transition()` itself — callers need not check.

### AC-006 (traces to BC-2.06.003 postcondition PC-3 — Filtering entry/exit)
`transition(Dashboard { focused }, Action::StartFilter { panel })` returns
`Filtering { panel, query: String::new(), prior: focused }`.
`transition(Filtering { panel, query, prior }, Action::CommitFilter)` returns
`Dashboard { focused: prior }`.
`transition(Filtering { .. }, Action::CancelFilter)` returns
`Dashboard { focused: prior }`.

### AC-007 (traces to BC-2.06.003 postcondition PC-4 — Fullscreen toggle)
`transition(Dashboard { focused }, Action::EnterFullscreen { panel })` returns
`Fullscreen { panel, prior: focused }`.
`transition(Fullscreen { panel, prior }, Action::ExitFullscreen)` returns
`Dashboard { focused: prior }`.

### AC-008 (traces to BC-2.06.003 postcondition PC-5 — Overlay Esc is identity)
`transition(Overlay { prior }, Action::Esc)` returns
`Overlay { prior }` unchanged. Esc in Overlay mode is a no-op; it NEVER
rejects a prompt or pops the stack.

### AC-009 (traces to BC-2.06.003 postcondition PC-6 — overlay push/pop)
`transition(mode, Action::PushOverlay { modal })`: if `mode` is `Dashboard` or
`Filtering`, pushes `modal` to `App.overlay_stack` and returns `Overlay { prior: current_focus }`.
If `mode` is `Overlay { prior }`, pushes `modal` to back of `App.overlay_stack`
(the modal stack lives in `App.overlay_stack`, not in the `Overlay` variant).
`transition(Overlay { prior }, Action::PopOverlay)`: removes front of `App.overlay_stack`;
if stack becomes empty returns `Dashboard { focused: prior }`; otherwise returns
`Overlay { prior }` (stack shrinks in `App.overlay_stack`).

### AC-010 (traces to BC-2.06.003 precondition 1 — BindingSource enum)
`BindingSource` is `#[non_exhaustive]` with variants:
`SearchPrompt`, `UserCustomCommand`, `PerContext`, `Global`, `Builtin`.
Priority order is `SearchPrompt > UserCustomCommand > PerContext > Global > Builtin`.

### AC-011 (traces to BC-2.06.003 postcondition PC-4 — resolve_binding None on no match)
`resolve_binding(key: KeyEvent, mode: &AppMode, layers: &BindingLayers) -> Option<(Action, BindingSource)>`
returns the highest-priority binding for `key` in current mode, with its source.
Returns `None` if no binding is registered at any level for this key+mode combination
(BC-2.06.003 PC-4: no-match returns `None`; keypress is discarded with no error).

### AC-012 (traces to BC-2.06.003 postcondition PC-2 — SearchPrompt captures printable keys)
When `mode` is `Filtering { .. }`, the `search_prompt` table contains bindings for all
printable characters (mapped to `Action::FilterType(char)`) and for `Escape` (mapped to
`Action::Escape`). `resolve_binding` checks the `SearchPrompt` layer first when in
`Filtering` mode — any printable key returns `BindingSource::SearchPrompt` regardless
of lower-layer registrations. Non-printable modifier keys (e.g., `Ctrl-P`) are not in
`search_prompt` and fall through to `global` or `builtin` layers (BC-2.06.003 EC-075).

### AC-013 (traces to BC-2.06.001 invariant INV-1 — AppMode exhaustive)
`AppMode` is NOT `#[non_exhaustive]`. Match arms over `AppMode` must be exhaustive
without `_`. All downstream crates receive a compile error if a new variant is added
without updating their match arms. This is the intended design.

### AC-014 (traces to BC-2.06.001 invariant INV-2 — purity boundary)
`monocle-core` has zero I/O crate dependencies. `similar`, `nucleo`, `ratatui`,
and `crossterm` MUST NOT appear in `monocle-core/Cargo.toml`. The purity boundary
is enforced by the build system: adding an I/O dependency to `monocle-core` causes
`cargo clippy --workspace -- -D warnings` to fail via a custom lint or build script
check (or the dependency-graph layer enforces it at the workspace level).

### AC-015 (traces to BC-2.06.003 invariant INV-1 — transition() totality)
`transition()` is total: every `(AppMode, Action)` pair produces a valid `AppMode`.
No panic, no `unreachable!()`, no `todo!()` in the final implementation.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~2,000 |
| BC-2.06.001.md | ~1,200 |
| BC-2.06.002.md | ~1,000 |
| BC-2.06.003.md | ~1,200 |
| S-011 (non-exhaustive enum policy) | ~500 |
| monocle-core existing lib.rs, mod structure | ~600 |
| Test files (state machine tests) | ~1,500 |
| **Total estimate** | **~8,000** |

## Tasks

- [ ] Create `monocle-core/src/tui/mod.rs` — re-export `state`, `binding` modules
- [ ] Create `monocle-core/src/tui/state.rs` — define `AppMode`, `FocusSnapshot` (enum), `PanelId`, `PromptModal`, `ToolPayload`, `Action`; implement `transition()`
- [ ] Implement `FocusSnapshot::cycle()` (round-robin; single-panel idempotent) and `FocusSnapshot::to_panel_id()` (converts variant to corresponding PanelId)
- [ ] Add `#[non_exhaustive]` to `FocusSnapshot`, `PanelId`, `BindingSource`, `Action`; ensure `AppMode` is NOT `#[non_exhaustive]`
- [ ] Implement `transition()` covering all `(AppMode, Action)` branches: StartFilter, CommitFilter, CancelFilter, EnterFullscreen, ExitFullscreen, PushOverlay, PopOverlay, Esc, MoveFocus, and fallthrough identity
- [ ] Enforce empty-stack collapse invariant inside `transition()` — whenever `App.overlay_stack` is empty after a `PopOverlay` or `PushOverlay` path produces `Overlay { prior }`, collapse to `Dashboard { focused: prior }`
- [ ] Create `monocle-core/src/tui/binding.rs` — define `BindingSource`, `BindingLayers`, `resolve_binding()`
- [ ] Implement 5-level priority order in `resolve_binding()`: SearchPrompt > UserCustomCommand > PerContext > Global > Builtin; SearchPrompt layer checked first when mode is Filtering
- [ ] Add `uuid` (workspace pin) to `monocle-core/Cargo.toml` for `Uuid` in `PromptModal`
- [ ] Verify `monocle-core/Cargo.toml` has NO `similar`, `nucleo`, `ratatui`, `crossterm` dependencies
- [ ] Unit tests `monocle-core/tests/tui_state_machine.rs` — cover all `transition()` branches, empty-stack collapse, Esc-in-Overlay identity, Filtering entry/exit, Fullscreen toggle
- [ ] Unit tests `monocle-core/tests/tui_binding.rs` — cover 5-level priority resolution, SearchPrompt override, None when unregistered

## Previous Story Intelligence

S-011 (non-exhaustive enum policy): Established that `#[non_exhaustive]` applies to
public enums where variants are additive extensions; exceptions are documented inline.
`AppMode` is an explicit exception — exhaustive match is required.

S-014 (EngineModule trait): monocle-core crate structure is established. The `tui`
module is a new addition; wire it into `monocle-core/src/lib.rs` as `pub mod tui`.

## Architecture Compliance Rules

From `architecture/SS-tui-core.md` and `architecture/SS-conventions-anti-patterns.md`:
- `AppMode` MUST NOT be `#[non_exhaustive]` — exhaustive match is the compile-time
  safety mechanism for this enum
- `FocusSnapshot` is an enum (NOT a struct) — `#[non_exhaustive]` enum with variants `Sessions`, `EventRibbon` (Phase 1); Phase 2 adds more variants without breaking existing match arms
- `FocusSnapshot`, `PanelId`, `BindingSource`, `Action` MUST be `#[non_exhaustive]`
- `transition()` lives in `monocle-core` (pure) — never `monocle-tui` (effectful)
- Empty-stack collapse enforced INSIDE `transition()`, not at call sites
- Purity boundary: monocle-core = zero I/O dependencies; similar/nucleo/ratatui/crossterm
  live in monocle-tui only
- `PromptModal.received_at` uses `std::time::Instant` (pure), not `tokio::time::Instant`

**Forbidden Dependencies for monocle-core:**
- `similar` — diff rendering, lives in monocle-tui only
- `nucleo` — fuzzy matching, lives in monocle-tui only
- `ratatui` — terminal UI widgets, lives in monocle-tui only
- `crossterm` — terminal backend, lives in monocle-tui only
- If any of these appear in `monocle-core/Cargo.toml`, the build MUST fail

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| uuid | workspace pin (features=["v4","serde"]) | `Uuid` for `PromptModal.prompt_id` |
| serde | workspace pin (features=["derive"]) | `ToolPayload::Generic.tool_input: serde_json::Value` |
| serde_json | workspace pin | `Value` in `ToolPayload::Generic` |
| crossterm | workspace pin | `KeyEvent` in `resolve_binding` signature only (dev-dep or re-exported type) |

Note: If `crossterm::KeyEvent` would introduce an I/O dep into monocle-core, define a
local `KeyEvent` wrapper type in monocle-core and re-export from monocle-tui. Check
whether crossterm's `KeyEvent` type has I/O in its dependency tree — if it does, use
a newtype wrapper.

## File Structure Requirements

Files to create:
- `monocle-core/src/tui/mod.rs` — module declarations for `state`, `binding`
- `monocle-core/src/tui/state.rs` — `AppMode`, `FocusSnapshot`, `PanelId`, `PromptModal`, `ToolPayload`, `Action`, `transition()`
- `monocle-core/src/tui/binding.rs` — `BindingSource`, `BindingLayers`, `resolve_binding()`
- `monocle-core/tests/tui_state_machine.rs` — state machine unit tests
- `monocle-core/tests/tui_binding.rs` — binding resolution unit tests

Files to modify:
- `monocle-core/src/lib.rs` — add `pub mod tui;`
- `monocle-core/Cargo.toml` — add `uuid`, verify no I/O crate leakage

## Downstream Consumer Contract

Public API produced by this story for downstream consumption:

```rust
// monocle-core::tui::state
pub enum AppMode { /* 4 variants, NOT #[non_exhaustive] */ }
#[non_exhaustive] pub enum FocusSnapshot { Sessions, EventRibbon /* Phase 2+: Customizations, Workflow, Preview */ }
impl FocusSnapshot { pub fn cycle(&self) -> FocusSnapshot; pub fn to_panel_id(&self) -> PanelId; }
#[non_exhaustive] pub enum PanelId { Sessions, EventRibbon, StaticExplorer, WorkflowPanel, HarnessPanel }
pub struct PromptModal { pub prompt_id: Uuid, pub session_id: String, pub tool_name: String, pub tool_payload: ToolPayload, pub received_at: std::time::Instant }
pub enum ToolPayload { Edit { old_content: String, new_content: String, path: PathBuf }, Bash { command: String }, Read { path: PathBuf }, Generic { tool_name: String, tool_input: serde_json::Value } }
#[non_exhaustive] pub enum Action { /* all TUI actions */ }
pub fn transition(mode: AppMode, action: Action) -> AppMode;

// monocle-core::tui::binding
#[non_exhaustive] pub enum BindingSource { SearchPrompt, UserCustomCommand, PerContext, Global, Builtin }
pub struct BindingLayers { /* per-layer binding maps */ }
pub fn resolve_binding(key: KeyEvent, mode: &AppMode, layers: &BindingLayers) -> Option<(Action, BindingSource)>;
```

S-025, S-026, S-031 all depend on these types being available in monocle-core.

## §Trace v1.4

**F-S025-ADV5-HIGH-003 — S-024 body sweep: Overlay shape annotation update (post-merge propagation)** (2026-05-28):
- Annotation-only sweep; status remains `not_started`; no behavioral change.
- Line 47: `Overlay { stack: VecDeque<PromptModal>, prior: FocusSnapshot }` → `Overlay { prior: FocusSnapshot }`
  with adjacent note that modal stack lives in `App.overlay_stack: VecDeque<PromptModal>`.
- Line 77 (AC-005): `Overlay { stack, prior } where stack is empty` → `Overlay { prior }` while `App.overlay_stack` is empty.
- Lines 96–97 (AC-008): `transition(Overlay { stack, prior }, Action::Esc) returns Overlay { stack, prior }` →
  `transition(Overlay { prior }, Action::Esc) returns Overlay { prior }`.
- Lines 101–106 (AC-009): `Overlay { stack: VecDeque::from([modal]), prior }`, `Overlay { stack, prior }`,
  `Overlay { stack: remaining, prior }` → all corrected to `Overlay { prior }` with explicit `App.overlay_stack` notation.
- Line 163 (Tasks): `Overlay { stack: empty, .. }` → `App.overlay_stack` is empty (collapse trigger corrected).
- Rationale: S-024 defined the original `Overlay { stack, prior }` shape that architect Pass 2 HIGH-003 (F-S025-ADV4-BLOCKER-001)
  retired in favour of `Overlay { prior }` + `App.overlay_stack`. Future readers referencing S-024 would otherwise
  be misled. This is an annotation-only propagation, not a behavioral revision.
- SE-16d monotonicity: v1.4 timestamp 2026-05-28T16:00:00Z >= v1.3 timestamp 2026-05-28T00:00:00Z. PASS.

## §Trace v1.3

**F-S025-ADV3-BLOCKER-002 — SS-06 BC version pins propagated from PO sweep (commit 6d4fbb3)** (2026-05-28):
- BC-2.06.001 inputs pin updated: v1.0.0 → v1.0.4.
- BC-2.06.002 inputs pin updated: v1.0.0 → v1.0.4.
- No body edits required — BC-2.06.001 and BC-2.06.002 changes (v1.0.0→v1.0.4) were
  cosmetic IPC field name cleanups and architecture source pin updates; they do not alter
  the AppMode/FocusSnapshot type definitions or transition() semantics specified here.
- SE-16d monotonicity: v1.3 timestamp 2026-05-28 >= v1.2 timestamp 2026-05-27. PASS.
