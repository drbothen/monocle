---
document_type: architect-decision
level: ops
cycle: cycle-001
story: S-025
pass: 1
version: "1.0"
status: binding
producer: vsdd-factory:architect
timestamp: 2026-05-28T00:00:00Z
phase: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.005.md, version: "1.0.4"}
  - {path: .factory/specs/architecture/SS-engine-module.md, version: "1.1.22"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.8.0"}
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.7.0"}
  - {path: crates/monocle-core/src/engine.rs, lines: "139-181"}
  - {path: crates/monocle-runtime/src/ring.rs, ref: "RAM_RING_CAPACITY=4096"}
  - {path: .worktrees/S-025/crates/monocle-tui/src/app.rs, lines: "50-90,196-220"}
input-hash: "[live-state]"
traces_to: "Resolves F-S025-ADV1-HIGH-001, F-S025-ADV1-HIGH-002, F-S025-ADV1-HIGH-003 from S-025 adversarial Pass 1."
---

# S-025 Architect Decisions — Pass 1

## Decision 1 — F-S025-ADV1-HIGH-001 (EnrichedSession field expansion)

**Chosen option:** C (Hybrid — expand struct, defensible v1 defaults)

**Rationale:** SS-engine-module.md v1.1.22 already specifies the full expanded shape
(`project_name`, `started_at`, `token_count`, `cost_usd`). The spec is ahead of the code.
The durable-task-register entry `IMPL-EnrichedSession-fields` was never closed. Option A
(full population) is not actionable in S-025 scope alone because `enrich()` in
`monocle-runtime::engine::claude_code` would need transcript-parsing logic to populate
`project_name` and `started_at` — that belongs to a richer enrichment story. But Option B
(hardcode em-dashes, defer BC) violates CLAUDE.md Principle 1 (no "for now" deferrals)
and Principle 4 (AI-built defect is AI's responsibility). Option C is the production-grade
synthesis: the struct fields MUST exist and MUST be wire-serializable so BC-2.06.005
PC-2 is satisfied and the formatter functions in `sessions_panel.rs` wire to real data
paths — the daemon just happens to emit zero/None for these fields in Phase 1 until
a richer enrichment story provides real values. The `"—"` render for `None`/zero is
correct behavior per BC-2.06.005 Invariant 3 and EC-084/EC-085, NOT a workaround.

**Required `engine.rs` `EnrichedSession` shape:**

```rust
/// `#[non_exhaustive]` per ADR-0006. All fields must derive Serialize + Deserialize
/// for IPC wire transport (SessionListUpdate and InitialState).
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichedSession {
    /// Engine-specific session identifier.
    pub session_id: String,
    /// Harness type identifier (e.g., "claude-code").
    pub harness_type: String,
    /// Absolute path to the engine-specific transcript file, if known.
    pub transcript_path: Option<PathBuf>,
    /// Absolute path to the engine-specific config file, if known.
    pub config_path: Option<PathBuf>,
    /// Session lifecycle status.
    pub status: SessionStatus,
    /// Timestamp of the last received hook event, in microseconds since the Unix epoch (UTC).
    /// `None` means no hook events have been received for this session yet.
    /// `Some(0)` is the Unix epoch, NOT a sentinel — using `0` as "no events" is forbidden.
    pub last_event_micros: Option<i64>,
    /// Human-readable project name derived from the transcript directory name
    /// (the immediate parent directory of `transcript_path`).
    /// `None` when `transcript_path` is unknown or parsing fails.
    /// Phase 1 daemon populates this during `enrich()`; zero-value is `None`.
    pub project_name: Option<String>,
    /// UTC timestamp of the first `SessionStart` hook event for this session.
    /// `None` until the daemon receives the first hook event carrying session-start data.
    /// The TUI computes uptime as `now - started_at` at render time.
    /// Phase 1 daemon defaults to `None`; populated when enrichment reads start time.
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Cumulative input + output token count reported by the harness hook stream.
    /// Defaults to `0` when the daemon has not yet received token-count hook data.
    /// Phase 1: zero is the sentinel-free default (the TUI renders `0` not `"—"`).
    pub token_count: u64,
    /// Cumulative cost in USD as reported by the harness hook stream.
    /// `None` when the daemon has not received cost data for this session.
    /// `Some(0.0)` is a valid zero-cost session — not a sentinel.
    pub cost_usd: Option<f64>,
}
```

**Type rationale:**
- `started_at: Option<chrono::DateTime<chrono::Utc>>` — matches SS-engine-module.md v1.1.22
  exactly. `chrono::DateTime<Utc>` round-trips correctly through `serde_json` with the
  `chrono/serde` feature (already in workspace deps per SS-deps-pin-manifest.md). DO NOT
  use `std::time::SystemTime` or `std::time::Instant` — neither is timezone-aware and
  `Instant` is not serializable.
- `cost_usd: Option<f64>` — `f64` is sufficient for Phase 1 cost display (two decimal
  places per BC-2.06.005 PC-2). `rust_decimal::Decimal` is NOT required and is not in
  the workspace deps manifest; adding it for display-only cost is disproportionate.
- `token_count: u64` — exact match to BC-2.06.005 PC-2 and SS-engine-module.md.

**`EnrichedSession::new()` constructor update:** Add the four new fields as parameters
after `last_event_micros`. Because the struct is `#[non_exhaustive]`, all construction
is via `new()` (per ADR-0006). The `new()` signature becomes:

```rust
pub fn new(
    session_id: String,
    harness_type: String,
    transcript_path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    status: SessionStatus,
    last_event_micros: Option<i64>,
    project_name: Option<String>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    token_count: u64,
    cost_usd: Option<f64>,
) -> Self
```

All existing call sites in `monocle-runtime::engine::claude_code` pass `None, None, 0, None`
for the four new parameters until a richer enrichment story populates them.

**SS-engine-module.md update required:** NO — SS-engine-module.md v1.1.22 already
specifies this exact shape (§EnrichedSession, lines 300-435 of the spec file as read).
The code is behind the spec. The fix is to bring the code up to spec, not change the spec.

**Affected BCs to bump:** None — BC-2.06.005 v1.0.4 already specifies all four fields
in PC-2. The BC is correct. The code was wrong.

**Implementer directive:**

1. In `crates/monocle-core/src/engine.rs`, expand `EnrichedSession` struct with the four
   fields above (exact types as specified). Update `new()` with four new parameters.
2. Add `chrono` to `monocle-core`'s `Cargo.toml` dependency if not present (check
   `SS-deps-pin-manifest.md` for the pinned version and feature flags required for serde).
3. Fix all `EnrichedSession::new(...)` call sites in `monocle-runtime` — add
   `project_name: None, started_at: None, token_count: 0, cost_usd: None` as trailing args.
4. Fix all `EnrichedSession::new(...)` call sites in test files similarly.
5. In `crates/monocle-tui/src/ui/sessions_panel.rs`, wire `format_token_count` and
   `format_cost` to `session.token_count` and `session.cost_usd` respectively in the
   production render path (replacing the `"—"` hardcodes). The `"—"` sentinel for
   `None` on `cost_usd`, `project_name`, and `started_at` is CORRECT rendering behavior
   per BC-2.06.005 Invariant 3 — it signals "daemon has no data yet", not a bug.
6. Verify `cargo build --workspace` and `cargo test --workspace` pass.
7. Verify that `format_token_count` unit tests pass against real `session.token_count` values.

**Cross-story impact:** Closes durable_task_register entry `IMPL-EnrichedSession-fields`
(status: resolved). State-manager to mark resolved in STATE.md after this story merges.

---

## Decision 2 — F-S025-ADV1-HIGH-002 (ring_tail handling)

**Chosen option:** A (S-025 expands `App` to include the ring_tail buffer now)

**Rationale:** BC-2.05.002 PC-5 is unambiguous: "TUI renders its initial state from this
message without polling… all subsequent state changes arrive as push messages." If
`ring_tail` is silently discarded, the TUI starts blind to events already in the daemon's
ring. This is a functional data-loss defect, not a UI deferral. The widget that renders
this state (event ribbon) belongs to S-027 — that is an acceptable feature-order deferral.
But the *state holder* belongs in S-025 because: (a) S-025 defines `App` and its
initialization logic, (b) wiring `on_initial_state` to populate `App::event_ring` is
trivially in-scope, and (c) deferring state to S-027 creates a structural regression
where S-027 must retroactively mutate `App` across a story boundary, creating a merge
dependency that does not exist if we do it now. CLAUDE.md Principle 2 (feature order is
the speed lever) applies: the event-ribbon WIDGET is deferred to S-027; the STATE HOLDER
is implemented now. Option B requires human escalation per CLAUDE.md Principle 3 and is
not chosen.

**Required `App` shape change:**

```rust
#[non_exhaustive]
pub struct App {
    pub mode: AppMode,
    pub config: MonocleConfig,
    pub sessions: Vec<EnrichedSession>,
    pub drop_counter: u64,
    pub overlay_stack: VecDeque<PromptModal>,
    pub status_message: Option<String>,
    /// Recent hook events from the daemon RAM ring, seeded from `InitialState::ring_tail`
    /// and extended by subsequent push messages (S-027).
    ///
    /// Bounded to [`EVENT_RING_CAPACITY`] entries — same as the daemon's RAM ring
    /// (BC-2.04.012 PC-1, `ring.rs::RAM_RING_CAPACITY = 4096`). Oldest entries are
    /// evicted on overflow; drop events are NOT counted in `App::drop_counter`
    /// (that counter tracks IPC channel drops, not ring evictions).
    pub event_ring: VecDeque<HookEventRecord>,
}
```

**Bound size and source of truth:** `EVENT_RING_CAPACITY = 4096`. This matches
`monocle-runtime::ring::RAM_RING_CAPACITY` (4096, defined in `ring.rs` line 53, per
BC-2.04.012 PC-1). The TUI-side ring should not exceed the daemon-side ring size —
there is no value in holding more events than the daemon can produce. Define a
`pub const EVENT_RING_CAPACITY: usize = 4096;` in `monocle-tui::app` module, with a
doc-comment citing `monocle_runtime::ring::RAM_RING_CAPACITY` as the canonical bound.
Do NOT import the runtime constant directly (that would create a monocle-tui → monocle-runtime
dependency not currently in the dep graph).

**Overflow policy:** FIFO drop of the oldest entry on push when `len() == capacity`.
This matches the daemon's own RAM ring policy (ring.rs `append()` implementation). The
TUI event ring drop is an internal rendering concern — do NOT increment `App::drop_counter`
on overflow (that counter tracks IPC channel packet drops per SS-ipc.md, not ring
evictions). No status-bar notification is required for TUI-side ring overflow in Phase 1
(the ring is large enough that overflow during normal operation is not expected; if it
overflows, the oldest events are the least relevant).

**`App::new()` initialization:** `event_ring: VecDeque::with_capacity(EVENT_RING_CAPACITY)`.

**`on_initial_state` fix:**

```rust
pub fn on_initial_state(
    app: &mut App,
    sessions: Vec<EnrichedSession>,
    ring_tail: Vec<HookEventRecord>,
    overlay_stack: Vec<PromptModal>,
    drop_counter: u64,
) {
    app.sessions = sessions;
    app.drop_counter = drop_counter;
    // Seed the event ring from the daemon's ring snapshot.
    // Bounded to EVENT_RING_CAPACITY; ring_tail from daemon is already bounded
    // to RAM_RING_CAPACITY (4096) so no overflow is expected, but enforce the
    // bound defensively.
    app.event_ring.clear();
    for record in ring_tail {
        if app.event_ring.len() == EVENT_RING_CAPACITY {
            app.event_ring.pop_front();
        }
        app.event_ring.push_back(record);
    }
    // overlay_stack handling (existing logic)
    for modal in overlay_stack {
        apply_permission_prompt_queued(&mut app.overlay_stack, modal);
    }
}
```

**Implementer directive:**

1. Add `pub const EVENT_RING_CAPACITY: usize = 4096;` to `crates/monocle-tui/src/app.rs`
   with a doc-comment citing `monocle_runtime::ring::RAM_RING_CAPACITY` as the source.
2. Add `pub event_ring: VecDeque<HookEventRecord>` field to `App` struct per the shape
   above. Import `HookEventRecord` from `monocle_ipc::types` (already imported in app.rs).
3. Initialize `event_ring: VecDeque::with_capacity(EVENT_RING_CAPACITY)` in `App::new()`.
4. Update `on_initial_state` to populate `app.event_ring` from `ring_tail` as shown above
   (replace the `_ring_tail` discard).
5. Write a unit test: `on_initial_state` with `ring_tail` of N > 0 records leaves
   `app.event_ring.len() == N` (and `== EVENT_RING_CAPACITY` when N > capacity).
6. Write a unit test: overflow eviction — seeding with `EVENT_RING_CAPACITY + 1` records
   leaves `app.event_ring.len() == EVENT_RING_CAPACITY` and the oldest record is gone.
7. S-027 will add: (a) push-message handler that appends to `app.event_ring` with same
   overflow policy, and (b) the event-ribbon widget that renders `app.event_ring`. S-025
   does NOT need to implement either of those.

---

## Decision 3 — F-S025-ADV1-HIGH-003 (InitialState re-invocation)

**Production-grade default in two sentences for the implementer:**

When `handle_server_message` receives a second `ServerToClient::InitialState` on an
already-initialized connection, log an error at `tracing::error!` level with the
session context and immediately close the IPC connection (return an error that the
transport loop treats as fatal, triggering the reconnect path). The `ignore` alternative
is ruled out because a duplicate `InitialState` signals daemon-side state machine
corruption or a protocol violation per BC-2.05.002 Invariant 1 — silent continuation
would cause the TUI's session list and event ring to silently diverge from daemon reality.

---

## Cross-Story Coordination

**BCs to bump after implementer lands the fix:**
- BC-2.06.005 does NOT need a version bump — the spec was already correct; the code fix
  brings the implementation into conformance.
- BC-2.05.002 does NOT need a version bump — Invariant 1 already covers the duplicate
  InitialState prohibition; HIGH-003 is a code bug against an existing invariant.

**SS spec updates:**
- SS-engine-module.md: No content change required (v1.1.22 already specifies the expanded
  shape). The implementer should verify the `new()` signature in the spec matches the
  implementation after the fix and flag any divergence to the architect.
- SS-ipc.md: No change required. `ring_tail: Vec<HookEventRecord>` type is already correct
  per S-022 adversarial resolution (SS-ipc v1.8.0).
- SS-tui.md: The `App` struct documentation should be updated to reflect `event_ring` field.
  This is an implementer-side doc update (inline Rust doc on the field) — no SS-tui.md
  version bump is required for an internal field addition.

**Durable task register:**
- Close `IMPL-EnrichedSession-fields`: resolved in S-025 cycle (HIGH-001 fix).
- State-manager to update STATE.md durable_task_register after S-025 merges.

**Merge order:** No new merge-order constraints introduced. S-025 and S-023 remain
authorized parallel. S-026 remains blocked on S-023 + S-025 both merging.
