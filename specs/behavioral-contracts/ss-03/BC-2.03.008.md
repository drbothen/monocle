---
document_type: behavioral-contract
level: L3
version: "1.0.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-engine-module-v2-delta.md]
input-hash: "427c948"
traces_to: prd.md
origin: greenfield
subsystem: SS-03
capability: CAP-003
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.03.008: Default spawn_recipe() Returns UnsupportedOperation

## Description

The `EngineModule` trait provides a default implementation of `spawn_recipe()` that returns
`Err(EngineError::UnsupportedOperation("spawn_recipe"))`. This default applies to all
engine implementations that do NOT override the method — including `CodeMachineModule` in
v1A scope. This establishes a safe, explicit boundary: session spawning is a capability
that must be explicitly opted into, not accidentally inherited.

## Preconditions

1. An `EngineModule` implementation exists that does NOT override `spawn_recipe()`.
2. `spawn_recipe()` is called on that implementation with any `SpawnOptions`.

## Postconditions

1. Returns `Err(EngineError::UnsupportedOperation("spawn_recipe"))` immediately. No I/O,
   no filesystem access, no `PATH` lookup is performed.
2. The error message format is: `"unsupported operation: spawn_recipe"`. Note: this is the
   raw inner string (the `EngineError::UnsupportedOperation` `Display` value). On the IPC wire,
   `SessionError::EngineError` wraps it via `#[error("engine error: {0}")]`, yielding
   `"engine error: unsupported operation: spawn_recipe"` in `ServerToClient::Error.message`.
   This is diagnostic-only — the TUI renders the FIXED banner ("Session spawn not supported
   for this harness") from PC-3 and NEVER displays `message` verbatim to the user.
3. When the daemon receives `UnsupportedOperation` from `spawn_recipe()`, it MUST surface
   an error to the TUI: `"Session spawn not supported for this harness"`. This banner is
   delivered via `ServerToClient::Error { code: "spawn_unsupported", message: "Session spawn not supported for this harness" }`
   — the `"spawn_unsupported"` wire code is the 11th entry in the `ServerToClient::Error`
   code taxonomy (SS-ipc v1.23.2; 12 codes including `session_not_ready` added F-P50-001 — `spawn_unsupported` remains the 11th), mapped from `EngineError::UnsupportedOperation` via
   `session_error_to_code(IpcOp::Spawn, EngineError::UnsupportedOperation)` →
   `"spawn_unsupported"` (SS-session-manager v2.6.0 §session_error_to_code). The session
   creation wizard MUST present this error in the UI and return to the ProfilePicker step.
4. The default impl is defined in the `EngineModule` trait body in `monocle-core/src/engine.rs`.
   It does NOT require any overriding `impl EngineModule for X` block — the default fires
   automatically for any implementor that does not override the method.
5. Adding `spawn_recipe()` with a default `Err` impl to the trait is NON-BREAKING for
   existing trait implementations. Existing `EngineModule` implementors that compiled before
   this addition continue to compile unchanged; they simply inherit the default `Err` behavior.

## Invariants

1. The default implementation fires for ALL engines that do not opt in — including any future
   WASM plugin engines loaded in Phase 3 that have not implemented `spawn_recipe()`.
2. `UnsupportedOperation` is NOT a retriable error. The caller MUST treat it as a permanent
   capability boundary for the selected engine, not a transient failure.
3. The `"spawn_recipe"` string in `UnsupportedOperation("spawn_recipe")` identifies the
   specific operation that is unsupported. This enables callers to distinguish "this engine
   cannot spawn" from other unsupported operations without string matching the full message.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-112 | User selects a CodeMachineModule profile and attempts to create a new session | **REACHABLE DEFENSIVE PATH** (F-P44-IMP-001): ProfilePicker capability filtering is best-effort and does NOT guarantee that CodeMachineModule profiles are excluded from the session creation wizard. When this path fires: daemon calls `spawn_recipe()` on `CodeMachineModule`; default impl returns `Err(UnsupportedOperation("spawn_recipe"))`; daemon surfaces `ServerToClient::Error { code: "spawn_unsupported", message: "Session spawn not supported for this harness" }`; TUI shows the error banner; wizard returns to ProfilePicker. |
| EC-113 | WASM engine loaded in Phase 3 that does not implement `spawn_recipe()` | Default `Err(UnsupportedOperation("spawn_recipe"))` fires; same error path as EC-112 |
| EC-114 | `CodeMachineModule::spawn_recipe()` called with any `SpawnOptions` including valid paths | Returns `Err(UnsupportedOperation("spawn_recipe"))` regardless of input validity — input is not inspected |

## Canonical Test Vectors

| Input | Expected Output | Edge Case | Test Type |
|-------|----------------|-----------|-----------|
| `CodeMachineModule.spawn_recipe(any_opts)` | `Err(EngineError::UnsupportedOperation("spawn_recipe"))` | — | unit |
| Any `EngineModule` impl without override called | `Err(EngineError::UnsupportedOperation("spawn_recipe"))` | — | unit |
| `spawn_session(opts, CodeMachineModule)` via IPC SpawnSession | `ServerToClient::Error { code: "spawn_unsupported", message: "Session spawn not supported for this harness" }` | EC-112 | integration |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `CodeMachineModule.spawn_recipe()` (or any non-overriding impl) returns `Err(UnsupportedOperation("spawn_recipe"))` | unit |
| VP-TBD | Trait compiles without requiring existing implementations to add `spawn_recipe()` override | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC defines the capability boundary for the engine abstraction: spawn is opt-in, not universal; the default Err impl enforces that boundary for all engines that do not explicitly support monocle-controlled session spawning |
| L2 Domain Invariants | DI-006 (EngineModule implementations must be stateless — the default impl performs no I/O and returns a constant error value, satisfying stateless detection requirement; spawn_recipe() is not a detection method but the same stateless principle applies to non-overriding impls) |
| Architecture Module | monocle-core (`EngineModule` trait default impl) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module-v2-delta.md v1.6.0 §spawn_recipe() — new trait method (default impl signature); SS-session-manager.md v2.6.0 §session_error_to_code — `EngineError::UnsupportedOperation` → `"spawn_unsupported"` arm (F-P44-IMP-001); SS-ipc.md v1.23.2 §`ServerToClient::Error` — `"spawn_unsupported"` as 11th wire code in taxonomy (12 total as of v1.23.2; `session_not_ready` is the 12th — F-P50-001; `spawn_unsupported` positional rank unchanged) |
| Cross-Ref | BC-2.03.005 (ClaudeCodeModule overrides this default with the real spawn_recipe() implementation) |
| Test Name | test_BC_2_03_008_default_spawn_recipe_unsupported_operation |

## Related BCs

- [BC-2.03.005] — contrasts with: ClaudeCodeModule overrides the default with a real implementation
- [BC-2.03.001] — depends on: EngineModule trait definition; spawn_recipe() is added to this trait

## Architecture Anchors

- `architecture/SS-engine-module-v2-delta.md#spawn_recipe-new-trait-method` — trait method signature and default impl

## Story Anchor

S-TBD — Same story as BC-2.03.005 (EngineModule trait extension with spawn_recipe() default; filled by story-writer)

## VP Anchors

VP-TBD — Default UnsupportedOperation unit test (filled after VP creation)

## §Trace v1.0.4

**P57-sweep — Architecture Source live-pin correction: SS-session-manager v2.5.0→v2.6.0; SS-ipc v1.23.0→v1.23.2** (2026-06-14):
- **Architecture Source (line 108 — live pin only):** SS-session-manager.md v2.5.0 → v2.6.0; SS-ipc.md v1.23.0 → v1.23.2. These two cascade bumps occurred at Pass-52 (F-P52-001 / D-295) and Pass-50 respectively; BC-2.03.008 was NOT in the Pass-51/52 cascade target lists, so its live Architecture Source cell was missed. §Trace lines (v1.0.3: lines 133/134; v1.0.2: lines 159/161/175) correctly record historical versions at the time of each trace entry — those are NOT stale and remain untouched.
- **POL-11 blind spot note:** `check_version_pins.py` does NOT flag the `path.md vX.Y.Z §section` pin format in Traceability table cells. This stale pin was invisible to POL-11 and was caught only by the Phase-1d→human-gate fresh-context consistency audit (Pass-57 sweep). No behavioral content changed.
- Patch bump: v1.0.3 → v1.0.4.

SE-16d monotonicity: v1.0.4 timestamp 2026-06-14 > v1.0.3 timestamp 2026-06-14. PASS.

## §Trace v1.0.3

**F-P50-001 — SS-ipc v1.22.0→v1.23.0 pin; 12th wire code (session_not_ready); spawn_unsupported positional rank confirmed; SS-session-manager v2.4.0→v2.5.0** (2026-06-14):
- **PC-3 (annotation):** `"spawn_unsupported"` is the 11th wire code; taxonomy now has 12 codes in v1.23.0 with `"session_not_ready"` added F-P50-001. `spawn_unsupported` positional rank (11th) is unchanged. Note added inline.
- **Architecture Source pins:** SS-session-manager.md v2.4.0 → v2.5.0; SS-ipc.md v1.22.0 → v1.23.0. SS-engine-module-v2-delta.md v1.5.0 unchanged (spawn_recipe() default impl is unaffected by taxonomy expansion).
- §Trace v1.0.2 "11th entry in the `ServerToClient::Error` taxonomy per SS-ipc.md v1.22.0" annotated with forward note: taxonomy grows to 12 codes in v1.23.0; `spawn_unsupported` rank unchanged.
- Patch bump: v1.0.2 → v1.0.3 (pin refresh + forward annotation; no behavioral content change).

SE-16d monotonicity: v1.0.3 timestamp 2026-06-14 > v1.0.2 timestamp 2026-06-14. PASS.

## §Trace v1.0.2 (errata)

**S-P47-002 — PC-2 clarifying note: raw EngineError Display vs wire-wrapped SessionError message** (2026-06-14):
- **Finding (S-P47-002):** PC-2 stated the error message as `"unsupported operation: spawn_recipe"`
  without distinguishing the raw `EngineError::UnsupportedOperation` Display value from the
  `SessionError::EngineError` wrapper that produces the wire-level `ServerToClient::Error.message`
  value (`"engine error: unsupported operation: spawn_recipe"`). This is harmless because
  SS-ipc mandates the TUI render the FIXED banner from PC-3 and never display `message` verbatim,
  but a reader could misunderstand which string appears on the wire.
- **PC-2 (clarifying errata):** Added one-line note explaining: the raw inner string is the
  `EngineError::UnsupportedOperation` Display value; on the wire, `SessionError::EngineError`
  wraps it via `#[error("engine error: {0}")]`; the fixed banner (not `message`) is
  what the user sees.
- **Bump disposition:** Errata-no-bump — contract behavior unchanged; note clarifies existing
  wire semantics already specified in SS-ipc. Version stays v1.0.2.

**F-P44-IMP-001 resolution — `spawn_unsupported` wire code; EC-112 reachability; integration test vector** (2026-06-14):

- **PC-3 (normative update):** The "Session spawn not supported for this harness" banner is now
  backed by the dedicated `"spawn_unsupported"` wire code (11th entry in the `ServerToClient::Error`
  taxonomy per SS-ipc.md v1.22.0; taxonomy grows to 12 codes in v1.23.0 with `session_not_ready` — F-P50-001; `spawn_unsupported` positional rank unchanged). PC-3 now cites the complete wire path:
  `EngineError::UnsupportedOperation` → `session_error_to_code(IpcOp::Spawn, …)` →
  `"spawn_unsupported"` (SS-session-manager.md v2.4.0 §session_error_to_code) →
  `ServerToClient::Error { code: "spawn_unsupported", … }`. Previously PC-3 cited only the
  user-facing banner without the wire code path.
- **EC-112 (reachability note added):** EC-112 is a REACHABLE defensive path per F-P44-IMP-001.
  ProfilePicker capability filtering is best-effort — it does NOT guarantee that non-spawning
  harness profiles are excluded from the session creation wizard at all times. The daemon's
  defense-in-depth is this BC: when the wizard reaches the daemon with a CodeMachineModule
  profile, `spawn_recipe()` returns `Err(UnsupportedOperation)` and the `"spawn_unsupported"`
  wire code is returned to the TUI. EC-112 description updated to make reachability explicit
  and to name the wire code.
- **Canonical Test Vector (new integration row):** `spawn_session(opts, CodeMachineModule)` via
  IPC SpawnSession → `ServerToClient::Error { code: "spawn_unsupported" }` (EC-112, integration).
  Exercises the full daemon-side defensive path end-to-end.
- **Architecture Source (pin bump):** SS-engine-module-v2-delta.md v1.4.1 → v1.5.0; added
  SS-session-manager.md v2.4.0 and SS-ipc.md v1.22.0 citations for the new wire code.
- Patch bump: v1.0.1 → v1.0.2.

SE-16d monotonicity: v1.0.2 timestamp 2026-06-14 > v1.0.1 timestamp 2026-06-13. PASS.

## §Trace v1.0.1

**Arch-source pin v1.4.0→v1.4.1 (architect C34-001 bump)** (2026-06-13 / D-276):
- Architecture Source updated: SS-engine-module-v2-delta.md v1.4.0 → v1.4.1.
- Reason: architect bumped SS-engine-module-v2-delta.md to v1.4.1 to correct the null-byte
  detection mechanism in spawn_recipe() (C34-001). No behavioral content in this BC changes
  — the default `UnsupportedOperation` implementation is unaffected by the detection mechanism
  correction.
- Patch bump only.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.03.008 authored for SS-03 as part of the v1A control-center pivot BC burst.
- Covers: default trait method impl returning UnsupportedOperation; non-breaking trait addition;
  CodeMachineModule v1A capability boundary.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
