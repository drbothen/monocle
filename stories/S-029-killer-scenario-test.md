---
document_type: story
level: L4
story_id: S-029
epic_id: EPIC-06
version: "1.3"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-28T00:00:00Z
phase: 2
points: 5
wave: 7
tdd_mode: strict
priority: P1
depends_on: [S-026, S-027, S-022, S-018]
blocks: []
target_module: monocle-tui
subsystems: [SS-06]
behavioral_contracts: [BC-2.06.022]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.022.md, version: "1.6.2"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.06.022 (killer scenario: permission prompt E2E round-trip)"
---

# S-029: Killer Scenario Integration Test — Permission Prompt E2E Round-Trip

## Narrative

As a quality engineer, I want a comprehensive integration test that exercises the
complete permission prompt lifecycle (daemon receives hook → TUI overlay appears →
user presses `y` → daemon receives Accept decision → daemon resumes hook), so that
regression coverage exists for the most critical user-facing workflow in monocle.

## Acceptance Criteria

### AC-001 (traces to BC-2.06.022 Step 1 — TUI startup and connection to daemon)
The integration test sets up: a real monocle daemon (or mock via S-DTU-001 clone)
connected to a `monocle-tui` instance via UDS. The test uses a scripted input
driver to simulate keyboard events injected into the TUI event loop without a real
terminal.

### AC-002 (traces to BC-2.06.022 Step 3 — Accept-once resolves prompt and collapses overlay to Dashboard)
Test scenario `killer_scenario_accept`:
1. Daemon receives a `PreToolUse` hook for a `Bash` tool with `command: "rm -rf /tmp/test"`.
2. Daemon emits `ServerToClient::PermissionPromptQueued { prompt_id, session_id, tool_name: "Bash", tool_payload: ToolPayload::Bash { command: "rm -rf /tmp/test" } }` to TUI.
3. TUI transitions to `Overlay` mode; `overlay_stack.len() == 1`.
4. Test driver injects keypress `y`.
5. TUI sends `ClientToServer::PermissionDecision { prompt_id, decision: PermissionDecision::Accept }` to daemon.
6. Daemon sends `ServerToClient::PermissionPromptResolved { prompt_id }` to TUI.
7. TUI removes modal from `overlay_stack`; stack is now empty; mode collapses to `Dashboard`.
8. Test asserts: `App.mode == AppMode::Dashboard { .. }` and `App.overlay_stack.is_empty()`.

### AC-003 (traces to BC-2.06.022 INV-4 — both sessions unblocked; FIFO overlay_stack ordering)
Test scenario `killer_scenario_multi_prompt`:
1. Daemon emits two `PermissionPromptQueued` messages (prompt_id_1, prompt_id_2) in rapid succession.
2. TUI `overlay_stack` has len == 2 after processing both.
3. Test driver injects `n` (reject front prompt: prompt_id_1).
4. Daemon sends `PermissionPromptResolved { prompt_id: prompt_id_1 }`.
5. TUI stack now len == 1; mode remains `Overlay`.
6. Test driver injects `y` (accept front prompt: prompt_id_2).
7. Daemon sends `PermissionPromptResolved { prompt_id: prompt_id_2 }`.
8. TUI stack is empty; mode collapses to `Dashboard`.

### AC-004 (traces to BC-2.06.022 INV-5 — empty-stack collapse is the mechanism; disconnect forces empty stack → Dashboard)
Test scenario `killer_scenario_disconnect`:
1. Two prompts queued (prompt_id_1, prompt_id_2); TUI in `Overlay` mode, stack len == 2.
2. Test driver simulates `TransportEvent::Disconnected`.
3. TUI clears `overlay_stack` to empty; transitions to `Dashboard`.
4. Test asserts: `App.mode == AppMode::Dashboard { .. }` and `App.overlay_stack.is_empty()`.
5. (Reconnect behavior is covered by S-023; this test focuses on the clear-on-disconnect invariant.)

### AC-005 (traces to BC-2.06.022 INV-3 — keystrokes are counted; Esc is not a decision keystroke and must not trigger resolution)
Test scenario `killer_scenario_esc_no_reject`:
1. One prompt queued; TUI in `Overlay` mode, stack len == 1.
2. Test driver injects `Esc` three times.
3. After all three Esc presses: `overlay_stack.len() == 1` (unchanged), `App.mode` is still
   `Overlay { .. }` (unchanged). No `PermissionDecision` was sent to the daemon.

### AC-006 (traces to BC-2.06.022 Step 1 screen render — overlay renders P1 ToolPayload including Edit diff)
Test scenario `killer_scenario_edit_diff`:
1. Daemon emits `PermissionPromptQueued` with `ToolPayload::Edit { old_content: "hello\n", new_content: "hello world\n", path: "/tmp/test.txt" }`.
2. TUI renders the overlay; test captures the rendered output (ratatui `TestBackend`).
3. Assert rendered output contains a line starting with `"+"` and colored green (or
   the text `"hello world"`); assert a line starting with `" hello"` (context line).

### AC-007 (traces to BC-2.06.022 KS-001/KS-002/KS-003 canonical test vectors — each vector runs in isolation with independent state)
Each test scenario runs in isolation with its own UDS socket path (using `tempfile::TempDir`
for the socket directory). No shared state between test scenarios. Tests can run in
parallel.

### AC-008 (traces to BC-2.06.022 Step 2 — AcceptAlways resolves front prompt; P2 becomes active; overlay remains open)
Test scenario `killer_scenario_accept_always` (canonical KS-001/KS-002 driver):
1. Daemon has P1 and P2 queued; TUI starts connected; `overlay_stack.len() == 2`.
2. Test driver injects keypress `A`.
3. TUI sends `ClientToServer::PermissionDecision { prompt_id: P1.prompt_id, decision: PermissionDecision::AcceptAlways }` to daemon.
4. Daemon broadcasts `PermissionPromptResolved { prompt_id: P1.prompt_id }`.
5. TUI calls `App.overlay_stack.retain(|m| m.prompt_id != P1.prompt_id)`; P1 removed; `overlay_stack = [P2]`; `AppMode` remains `Overlay`.
6. Test driver injects keypress `y`.
7. TUI sends `ClientToServer::PermissionDecision { prompt_id: P2.prompt_id, decision: PermissionDecision::Accept }` to daemon.
8. Daemon broadcasts `PermissionPromptResolved { prompt_id: P2.prompt_id }`.
9. `overlay_stack` is empty; `AppMode` collapses to `Dashboard { focused: Sessions }`.
10. Test asserts: total user keystrokes = 2 (`A`, `y`), satisfying the ≤6 Success Criterion for the 2-session case (4 total including `[connect]`/`[disconnect]`).
This test directly exercises the BC-2.06.022 headline promise: both sessions unblocked in 4 keystrokes.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,800 |
| BC-2.06.022.md | ~900 |
| S-026 (overlay core) | ~1,000 |
| S-027 (overlay rendering) | ~800 |
| S-022 (UDS IPC types) | ~600 |
| S-018 (hook routing event bus) | ~500 |
| ratatui TestBackend API | ~300 |
| Test infrastructure patterns | ~500 |
| **Total estimate** | **~6,400** |

## Tasks

- [ ] Create `monocle-tui/tests/killer_scenario.rs` — main integration test file
- [ ] Implement `MockDaemon` struct: UDS listener that speaks `ServerToClient`/`ClientToServer`,
      scripted to emit `PermissionPromptQueued` and respond to `PermissionDecision` with `PermissionPromptResolved`
- [ ] Implement `TestInputDriver` struct: inject `crossterm::event::KeyEvent` into TUI event loop
      without a real terminal
- [ ] Implement `killer_scenario_accept` test — full 8-step happy path per AC-002
- [ ] Implement `killer_scenario_multi_prompt` test — two-prompt FIFO stacking per AC-003
- [ ] Implement `killer_scenario_disconnect` test — disconnect clears overlay per AC-004
- [ ] Implement `killer_scenario_esc_no_reject` test — Esc is identity per AC-005
- [ ] Implement `killer_scenario_edit_diff` test — diff rendered in TestBackend per AC-006
- [ ] Ensure each test uses a separate `tempfile::TempDir` for socket path isolation per AC-007
- [ ] Implement `killer_scenario_accept_always` test — AcceptAlways + Accept dual-resolve per AC-008 (BC-2.06.022 canonical KS-001/KS-002)
- [ ] Verify tests can run in parallel via `cargo test --test killer_scenario -- --test-threads=4`

## Previous Story Intelligence

S-026 (permission overlay core): All the IPC message handlers (PermissionPromptQueued,
PermissionPromptResolved, TransportEvent::Disconnected) are implemented in `app.rs`.
This story drives those handlers via a mock daemon.

S-027 (overlay rendering): ratatui `TestBackend` is the standard approach for headless
rendering tests. It provides a `Buffer` that can be inspected for cell content and style.
Use `ratatui::backend::TestBackend::new(width, height)` and `ratatui::Terminal::new(backend)`.

S-022 (UDS IPC types): `MockDaemon` must use the same serialization format as the real
daemon. Use `monocle-ipc` types directly for message construction and parsing.

S-018 (hook routing event bus): The `PreToolUse` hook path that triggers `PermissionPromptQueued`
is implemented in S-018. For this test, `MockDaemon` can emit `PermissionPromptQueued` directly
without going through the full hook routing path.

## Architecture Compliance Rules

From `architecture/SS-tui.md` and `architecture/SS-conventions-anti-patterns.md`:
- Tests use `ratatui::backend::TestBackend` for headless rendering — no real terminal
- `MockDaemon` uses real UDS sockets (via `tempfile::TempDir`) — not in-memory mocks
- `ClientToServer::ClientDisconnect` does NOT exist — `MockDaemon` must not expect it
- `TransportEvent::Disconnected` is how the TUI detects disconnect — `MockDaemon` closes
  the socket to trigger this
- Each test in a separate `tempfile::TempDir` — never reuse socket paths across tests
- `PermissionDecision` variants: `Accept`, `AcceptAlways`, `Reject` — exact names

**Forbidden Dependencies:**
- No real daemon process in tests — use `MockDaemon` to avoid test environment dependencies
- No hardcoded socket paths — all paths via `tempfile::TempDir`
- No `sleep()` for synchronization — use channel-based signaling or `tokio::sync::Notify`

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| ratatui | workspace pin | `TestBackend` for headless rendering, `Buffer` inspection |
| monocle-ipc | workspace path | `ServerToClient`, `ClientToServer`, `PermissionDecision`, `TransportEvent` |
| monocle-tui | workspace path | `App` struct, TUI event loop under test |
| tempfile | workspace pin | `TempDir` for per-test UDS socket isolation |
| tokio | workspace pin | Async runtime for test execution |
| tokio::sync::Notify | stdlib (tokio) | Channel-based synchronization between mock daemon and TUI |

## File Structure Requirements

Files to create:
- `monocle-tui/tests/killer_scenario.rs` — all 7 killer scenario test functions (AC-002..AC-007 + AC-008 AcceptAlways)
- `monocle-tui/tests/test_helpers/mock_daemon.rs` — `MockDaemon` struct (if helpers are modular)
- `monocle-tui/tests/test_helpers/input_driver.rs` — `TestInputDriver` struct

Files to modify: None (test-only story; no production code changes)

## Downstream Consumer Contract

No new public API. This is a pure test story. No other stories depend on this one.
It is the final validation gate for EPIC-06 permission overlay functionality.

## §Trace v1.3

**Post-delivery spec reconciliation — S-029 DONE (PR #35 @ 48463fb, 2026-06-02):**

1. **Status + delivery reference (edit 1):** `status: not_started` → `status: done`. Delivery: PR #35, merge SHA 48463fb on develop.

2. **BC input pin refresh (edit 2):** `BC-2.06.022.md version: "1.6.0"` → `"1.6.2"`. BC v1.6.1 (2026-05-29) and v1.6.2 (2026-05-30) were version-pin/staleness remediation passes per their respective §Trace entries — no normative AC-affecting content changed. No body propagation required for this pin update.

3. **AC trace label reconciliation (edits 3–9):** AC-001..AC-006 cited "PC-N" labels that do not exist in BC-2.06.022. The BC's postconditions are structured as "Step 1..4" + Summary Postcondition + "Invariant 1..5" + Canonical Test Vectors, not PC-N. All trace parentheticals updated to cite the correct BC structural anchors:
   - AC-001: `PC-1` → `Step 1` (TUI startup and connection — Step 1 governs TUI spawn, connect, InitialState receipt, and AppMode transition).
   - AC-002: `PC-2` → `Step 3` (Accept-once resolves prompt and collapses overlay — `killer_scenario_accept` drives through Step 3's postcondition: `PermissionPromptResolved` → `retain()` → empty stack → `AppMode::Dashboard`).
   - AC-003: `PC-3` → `INV-4` (both sessions unblocked — multi-prompt FIFO stacking verifies INV-4: "both Claude Code sessions are unblocked after the flow").
   - AC-004: `PC-4` → `INV-5` (empty-stack collapse is the mechanism — disconnect forces the stack to empty, exercising INV-5: the automatic Dashboard return via App-level retain() and AppMode transition).
   - AC-005: `PC-5` → `INV-3` (keystrokes are counted; Esc is not a decision keystroke — INV-3 states keystrokes are counted and the total must not exceed 6; Esc must not register as a resolution keystroke).
   - AC-006: `PC-6` → `Step 1 screen render` (Edit diff rendered — Step 1's screen render row specifies "Overlay renders P1 with its ToolPayload"; Edit ToolPayload diff rendering is the ToolPayload variant under test).
   - AC-007: `INV-1` → `KS-001/KS-002/KS-003 canonical test vectors` (test isolation — each KS vector is an independent scenario; INV-1 is "editor focus preserved" in the BC, not test isolation; the canonical test vectors section is the correct structural anchor for per-test independence).

4. **AC-008 added (edit 10):** `killer_scenario_accept_always` — AcceptAlways (`A`) as the first decision in the dual-prompt KS-001/KS-002 canonical flow. This is the BC-2.06.022 headline keystroke (Step 2). The delivered test `test_BC_2_06_022_killer_scenario_accept_always` covers KS-001+KS-002 explicitly. AC-008 was absent from v1.2 despite Step 2 being the defining differentiator of the ≤6-keystroke promise. Task entry added correspondingly.

5. **File structure count fix (edit 11):** `killer_scenario.rs — all 6 killer scenario test cases` → `all 7 killer scenario test functions (AC-002..AC-007 + AC-008 AcceptAlways)`. The delivered file ships 7 test functions; the prior count of 6 was stale (predated AC-008 addition).

- SE-16d monotonicity: v1.3 timestamp 2026-06-02 >= v1.2 timestamp 2026-05-29. PASS.

## §Trace v1.2

**F-S025-ADV22-MED-001 sibling propagation — SS-tui-core.md → SS-tui.md (line 143)** (2026-05-29):
- Architecture Compliance Rules header: `architecture/SS-tui-core.md` → `architecture/SS-tui.md`.
- Systematic EPIC-06 story-writing burst defect; canonical anchor is `SS-tui.md` per BC-2.06.005 §Architecture Source + audit-table.md row 41.
- SE-16d monotonicity: v1.2 timestamp 2026-05-29 >= v1.1 timestamp 2026-05-28. PASS.

## §Trace v1.1

**F-S025-ADV3-BLOCKER-002 — SS-06 BC version pins propagated from PO sweep (commit 6d4fbb3)** (2026-05-28):
- BC-2.06.022 inputs pin updated: v1.0.0 → v1.6.0.
- No body edits required — BC-2.06.022 content changes do not affect the story's AC text.
- SE-16d monotonicity: v1.1 timestamp 2026-05-28 >= v1.0 timestamp 2026-05-27. PASS.
