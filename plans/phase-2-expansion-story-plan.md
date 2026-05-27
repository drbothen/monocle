---
document_type: planning-artifact
level: planning
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
phase: phase-2-expansion
timestamp: 2026-05-27T00:00:00Z
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.19"}
  - {path: .factory/specs/architecture/SS-daemon-wiring.md, version: "1.3.0"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.6.0"}
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.6.0"}
  - {path: .factory/specs/architecture/SS-config.md, version: "1.3.0"}
  - {path: .factory/stories/STORY-INDEX.md, version: "3.0"}
---

# Phase 2 Expansion Story Plan

> This plan covers the decomposition of 49 new BCs across SS-04, SS-05, SS-06, and
> SS-07 into implementable stories starting from S-016. All IDs are sequential,
> continuing from the existing 17 stories (S-001 through S-015, S-DTU-001,
> S-PHASE-3-PREP). Waves 4+ continue from where Waves 1-3 left off.

---

## Epic Table

| Epic ID | Name | Capability | Subsystem | Stories |
|---------|------|-----------|-----------|---------|
| EPIC-04 | Daemon Wiring | CAP-004 | SS-04 | S-016, S-017, S-018, S-019, S-020 |
| EPIC-05 | IPC | CAP-005 | SS-05 | S-021, S-022, S-023 |
| EPIC-06 | TUI | CAP-006 | SS-06 | S-024, S-025, S-026, S-027, S-028, S-029 |
| EPIC-07 | Config | CAP-007 | SS-07 | S-030, S-031 |

**Total new stories: 16**

---

## Proposed Story List

### EPIC-04: Daemon Wiring (SS-04)

---

#### S-016: Daemon Binary Crate Init + CLI Subcommands

**Title:** Daemon Binary Crate Init + CLI Subcommands (`monocle daemon start/stop`)

**Epic:** EPIC-04

**BCs covered:**
- BC-2.04.004 — `monocle daemon start` CLI Subcommand (P0)
- BC-2.04.005 — `monocle daemon stop` CLI Subcommand (P0)
- BC-2.04.006 — `directories::ProjectDirs::runtime_dir()` Fallback Chain (P0)

**Points:** 5

**Dependencies:** S-001 (Cargo workspace init), S-006 (Lock file — daemon stop reads lock file)

**Wave:** 4

**Rationale:**
BC-2.04.004 and BC-2.04.005 are the CLI entry points for the `monocle daemon` subcommand
family. They both depend on the runtime_dir fallback chain (BC-2.04.006), which is also used
by the existing S-006 lock-file code and must be wired into the binary. These three BCs form
the CLI surface layer and are naturally implemented together because:
1. BC-2.04.006 (runtime_dir fallback) is a shared prerequisite for both subcommands.
2. `daemon start` blocks on the lock file, which S-006 implements — this story consumes S-006.
3. `daemon stop` reads the lock file and sends SIGTERM — also a consumer of S-006.
4. The binary crate (`monocle`) is scaffolded here for the first time with `clap`.
No dependency on new SS-04 stories (start sequence is in S-017); Wave 4 is appropriate.

---

#### S-017: Daemon Start Sequence + Hook Tmpfile Generation

**Title:** Daemon Start Sequence (SOQ-2) + Hook Tmpfile Generation

**Epic:** EPIC-04

**BCs covered:**
- BC-2.04.001 — Daemon Start Sequence: Port Bind + Lock File + Token Write (SOQ-2) (P0)
- BC-2.04.010 — Hook Tmpfile Generation at `runtimeDir/hooks-settings.json` (P0)

**Points:** 8

**Dependencies:** S-016 (binary crate init; this story implements the daemon_start_sequence
called by the CLI), S-006 (lock file), S-008 (ring buffer), S-009 (auth token), S-015
(ClaudeCodeModule for EngineModule registration), S-012 (VsddFactoryAdapter)

**Wave:** 5

**Rationale:**
The 13-step daemon start sequence (BC-2.04.001) is the most complex single piece of SS-04
wiring. It requires every foundational component to be built: the lock file (S-006), the
auth token generator (S-009), the ring buffer (S-008), and the ClaudeCodeModule (S-015) for
EngineModule registry registration. Hook tmpfile generation (BC-2.04.010) is step 9 of
the same start sequence and writes `hooks-settings.json` immediately after the lock file
(SOQ-2 ordering invariant). These two BCs share step 8 (lock file write) and step 9
(hooks-settings.json write) as an ordered pair — splitting them would require testing the
SOQ-2 invariant across stories, creating a dependency gap. The 8-point estimate reflects
the integration complexity: 13 ordered steps with multiple I/O operations, SOQ-2
invariant, atomic writes, and the EngineModule registry wiring.

---

#### S-018: Hook Endpoint Routing + Bounded Event Bus

**Title:** Hook Endpoint Routing + Bounded Event Bus with Drop Counter

**Epic:** EPIC-04

**BCs covered:**
- BC-2.04.007 — Hook Endpoint: PreToolUse Request Routing (P0)
- BC-2.04.008 — Hook Endpoint: Notification Request Routing (2000ms Timeout) (P0)
- BC-2.04.009 — Hook Endpoint: Stop/SessionStart/PromptSubmit Routing (300ms Timeout) (P0)
- BC-2.04.011 — Bounded Event Bus with Drop Counter (P0)

**Points:** 8

**Dependencies:** S-017 (daemon start sequence creates the event bus), S-002 (healthz),
S-003 (status endpoint — auth middleware), S-004 (body size limit middleware), S-009 (auth
token/header validation), S-014 (EngineModule trait — hook handlers call on_hook)

**Wave:** 5

**Rationale:**
The hook endpoint routing BCs (BC-2.04.007, BC-2.04.008, BC-2.04.009) are the request-path
handlers that sit on top of the existing middleware stack (auth, body-limit) built in Waves
1-3. They form a coherent group because they share the same handler pattern:
deserialize HookEnvelope → call on_hook → publish to event bus → return response.
The bounded event bus (BC-2.04.011) is inseparable from hook routing because:
- The event bus is the consumer of every processed hook event.
- The drop counter (AtomicU64 in DaemonState) is incremented by try_send in the handler.
- Testing hook routing without the event bus would require mocking the bus out, which is more
  work than implementing it.
Splitting hook routing from the event bus would create an artificial seam where handlers can
route but have no consumer.

---

#### S-019: Daemon Auto-Start on TUI Launch

**Title:** Daemon Auto-Start on TUI Launch + MONOCLE_NO_AUTOSTART

**Epic:** EPIC-04

**BCs covered:**
- BC-2.04.002 — Daemon Auto-Start on TUI Launch (P0)
- BC-2.04.003 — `MONOCLE_NO_AUTOSTART=1` Suppresses Auto-Start (P1)

**Points:** 5

**Dependencies:** S-016 (binary CLI), S-017 (daemon start sequence — auto-start calls it)

**Wave:** 5

**Rationale:**
BC-2.04.002 and BC-2.04.003 are companion behaviors for the default TUI launch path.
BC-2.04.002 is the positive case (start daemon if not running); BC-2.04.003 is the escape
hatch (MONOCLE_NO_AUTOSTART=1 skips the liveness check entirely). Both modify the same
code path in `main.rs` before TUI initialization. Implementing BC-2.04.002 without BC-2.04.003
would leave the auto-start path unguarded for CI environments. These are naturally a pair.
Points are 5 because the 5-step decision sequence (lock file check, PID liveness, start
subprocess, wait for lock file, retry) is well-specified but involves subprocess management.

---

#### S-020: JSONL Ring Capacity and Rotation Policy

**Title:** JSONL Ring Capacity and Rotation Policy (BC-2.04.012)

**Epic:** EPIC-04

**BCs covered:**
- BC-2.04.012 — JSONL Ring: Capacity and Rotation Policy (P1)

**Points:** 5

**Dependencies:** S-008 (JSONL Ring Format), S-017 (daemon start sequence creates the RingBuffer)

**Wave:** 5

**Rationale:**
BC-2.04.012 specifies the rotation policy: 100MB file cap, 5 rotation files on disk, 4,096
in-memory RAM ring. S-008 built the JSONL ring and its format; this story adds the capacity
and rotation behavior on top. The rotation logic is a distinct feature from the base format
(S-008) and requires integration with the start sequence (S-017) to configure capacity
correctly. This is P1 (versus P0 hook routing) and can be in the same wave as the other
SS-04 stories. The 5-point estimate reflects that the rotation logic is non-trivial (file
rename chain, maximum-rotation-files enforcement) but is self-contained within monocle-runtime.

---

### EPIC-05: IPC (SS-05)

---

#### S-021: UDS Server Bind + IPC Transport and Message Types

**Title:** UDS Server Bind + IPC Transport + Core Message Types

**Epic:** EPIC-05

**BCs covered:**
- BC-2.05.001 — UDS Server Bind at `runtimeDir/monocle.sock` (P0)
- BC-2.05.003 — IPC Message Types: SessionListUpdate (P0)
- BC-2.05.004 — IPC Message Types: HookEventReceived (P0)
- BC-2.05.008 — UDS-Only in Phase 1 (No Shared-Memory Transport) (P1)

**Points:** 8

**Dependencies:** S-017 (daemon start sequence — UDS socket is step 10), S-014
(EngineModule — EnrichedSession type used in SessionListUpdate), S-013 (HookEnvelope —
HookType used in HookEventReceived)

**Wave:** 5

**Rationale:**
BC-2.05.001 (UDS server bind) is the foundation: without it, the client cannot connect.
It is implemented as step 10 of the daemon start sequence (already specified in S-017 at
the orchestration level; this story actually builds the monocle-ipc crate).
BC-2.05.003 (SessionListUpdate) and BC-2.05.004 (HookEventReceived) are the first two
message types that flow outward from the daemon to the TUI after the initial state push.
They are grouped with UDS bind because:
1. All three live in `monocle-ipc`: the Transport trait, framing protocol, and message enums.
2. Implementing the server side without defining the message types leaves an incomplete crate.
3. The framing protocol (4-byte LE length prefix, 256 KiB limit, serde_json) is tested via
   SessionListUpdate and HookEventReceived — they provide the first real exercise of framing.
BC-2.05.008 (UDS-only Phase 1 constraint) is a compile-time/CI enforcement rule (cargo deny,
semgrep for mmap/shm imports). It belongs here because it is part of the monocle-ipc crate
structure (`#![forbid(unsafe_code)]`, no libc::mmap dependency). Adding it separately would
require revisiting the same crate.

---

#### S-022: TUI Connect + Initial State Push + Permission Prompt Message Types

**Title:** TUI Client Connect, Initial State Push, and Permission Message Types

**Epic:** EPIC-05

**BCs covered:**
- BC-2.05.002 — TUI Client Connects to UDS and Receives Initial State Push (P0)
- BC-2.05.005 — IPC Message Types: PermissionPromptQueued (P0)

**Points:** 8

**Dependencies:** S-021 (UDS server bind + core message types — client connects to
already-bound server), S-018 (event bus + hook routing — PermissionPromptQueued is
triggered by PreToolUse with decision_required)

**Wave:** 6

**Rationale:**
BC-2.05.002 (TUI client connect + initial state push) depends on the server being bound
(BC-2.05.001 in S-021). The initial state push delivers all five ServerToClient message
fields: sessions, ring_tail, overlay_stack, drop_counter — requiring the full message
enum to be in place. BC-2.05.005 (PermissionPromptQueued) is the third major message type
and is closely tied to BC-2.05.002 because:
- The initial state push sends overlay_stack: Vec<PermissionPromptPayload> — the same
  payload type used in PermissionPromptQueued.
- Testing the initial state push requires the PermissionPromptPayload struct to exist.
- The `oneshot::channel` per-prompt pending-decision registry (referenced in SS-04 and SS-05)
  must be exercised in the PermissionPromptQueued path.
Wave 6 is correct because this depends on S-021 (Wave 5) for the UDS server.

---

#### S-023: TUI Reconnect + SOQ-3 Overlay Clear on Disconnect

**Title:** TUI Reconnect After Daemon Restart + SOQ-3 Overlay Clear (BC-2.05.006/007)

**Epic:** EPIC-05

**BCs covered:**
- BC-2.05.006 — TUI Reconnects After Daemon Restart (P1)
- BC-2.05.007 — Overlay Stack Cleared on Daemon Disconnect (SOQ-3) (P0)

**Points:** 5

**Dependencies:** S-022 (TUI client connect — reconnect is a re-invocation of the connect
path), S-019 (daemon auto-start — reconnect may trigger a daemon re-start)

**Wave:** 6

**Rationale:**
BC-2.05.007 (SOQ-3 overlay clear) is P0 and logically paired with BC-2.05.006
(reconnect) because they are part of the same failure-recovery path:
1. Daemon crashes → UDS connection drops.
2. SOQ-3: TUI clears local VecDeque<PromptModal>.
3. TUI enters reconnect loop (exponential backoff, 5-second window).
4. Successful reconnect → daemon sends fresh InitialState push → TUI rebuilds state.
These cannot be split because SOQ-3 is the first action in the reconnect sequence. Testing
SOQ-3 without the reconnect loop would require simulating only a partial disconnect path.
Together they validate the complete disconnect-recover-rebuild cycle.

---

### EPIC-06: TUI (SS-06)

The TUI epic is the largest (23 BCs). The decomposition groups BCs by architectural layer
to minimize intra-wave dependencies.

---

#### S-024: TUI Core Types + AppMode State Machine + Action Dispatch

**Title:** TUI Core Types: AppMode, Action, FocusSnapshot, transition(), 5-Level Dispatch

**Epic:** EPIC-06

**BCs covered:**
- BC-2.06.001 — AppMode State Machine: Compile-Time Mutual Exclusion (P0)
- BC-2.06.002 — FocusSnapshot: Focus Restored After Overlay/Fullscreen Close (P0)
- BC-2.06.003 — Action Dispatch: 5-Level Binding Precedence (P0)

**Points:** 8

**Dependencies:** S-011 (Non-Exhaustive Enum Policy — AppMode, Action, BindingSource,
PanelId all carry #[non_exhaustive] per BC-2.02.003), S-014 (EngineModule trait — monocle-core
is the crate where AppMode/Action live; must be initialized)

**Wave:** 4

**Rationale:**
These three BCs define the pure-core data types and transition function that underpin the
entire TUI. They live entirely in `monocle-core` (no ratatui, no crossterm, no I/O).
BC-2.06.001 (AppMode enum with compile-time mutual exclusion) is the prerequisite for
all other TUI stories — without the AppMode enum and transition() function, no TUI
rendering can be tested.
BC-2.06.002 (FocusSnapshot) is an invariant on the transition function (prior field is
always preserved) — it is tested in the same test module as BC-2.06.001.
BC-2.06.003 (5-level binding precedence) defines the Dispatcher struct and BindingSource
enum, which also live in monocle-core. The Dispatcher is constructed with 5 HashMap tables;
testing it requires AppMode to exist (for update_context()).
These are Wave 4 because they depend only on S-011 (Wave 2) and S-014 (Wave 2) from
prior waves. Pure-core work, no IPC, no ratatui — appropriate earliest wave.

---

#### S-025: TUI Binary Skeleton + Ctrl-\ Popup + Sessions Panel

**Title:** TUI Binary Skeleton, Ctrl-\ Popup Integration, and Sessions Panel

**Epic:** EPIC-06

**BCs covered:**
- BC-2.06.004 — `Ctrl-\` Popup: Appears and Dismisses Without State Loss (P0)
- BC-2.06.005 — Sessions Panel: Session List Renders from IPC State (P0)
- BC-2.06.007 — Sessions Panel: `Enter` Transitions to Fullscreen (P1)

**Points:** 8

**Dependencies:** S-024 (AppMode/Action types), S-022 (TUI client connect — IPC state
arrives via initial push, sessions are from SessionListUpdate), S-030 (Config — profile
picker selection at startup reads config)

**Wave:** 6

**Rationale:**
BC-2.06.004 (Ctrl-\ popup) is the top-level TUI entry point. It tests that the daemon
state survives TUI process restart (monocle spawned → connects → receives InitialState →
AppMode set from overlay_stack). This BC requires the full IPC connect path (S-022) to
be in place.
BC-2.06.005 (sessions panel) is the first rendered output — it depends on IPC
SessionListUpdate messages arriving and being rendered to the terminal. It requires the
ratatui draw loop and the monocle-tui crate to be wired to the IPC receiver.
BC-2.06.007 (Enter → Fullscreen) is grouped here rather than with the filter story
because it is a simple transition() invocation (Dashboard → Fullscreen) that is naturally
exercised when the sessions panel is rendered. Splitting it would create an artificial
boundary in the sessions panel test coverage.
Wave 6 is necessary because S-022 (IPC client connect, Wave 6) must be ready before
the TUI can receive session data.

---

#### S-026: Permission Overlay Core (Stack, Decisions, Esc)

**Title:** Permission Overlay: VecDeque Stack, Decision Keybindings, Esc Hide, SOQ-3

**Epic:** EPIC-06

**BCs covered:**
- BC-2.06.008 — Permission Overlay: VecDeque Stack Push on PermissionPromptQueued (P0)
- BC-2.06.009 — Permission Overlay: `[↑↓]` Rotates Stack (P0)
- BC-2.06.011 — Permission Overlay: Accept-Once Keybinding (P0)
- BC-2.06.012 — Permission Overlay: Accept-Always Keybinding (P0)
- BC-2.06.013 — Permission Overlay: Reject Keybinding (P0)
- BC-2.06.014 — Permission Overlay: `[Esc]` Hides Without Rejecting (P0)
- BC-2.06.016 — Permission Overlay: Cleared on Daemon Disconnect (P0)
- BC-2.06.023 — TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved (P0)

**Points:** 13

**Dependencies:** S-024 (AppMode transition function handles Overlay variants),
S-022 (PermissionPromptQueued IPC message arrives via SS-05), S-023 (SOQ-3 overlay clear
on disconnect — BC-2.06.016 is the TUI side of BC-2.05.007)

**Wave:** 6

**Rationale:**
All 8 BCs in this story are P0 and form the core permission overlay system — the product's
primary competitive differentiator. They cannot be split without creating artificial partial
states:
- BC-2.06.008 (push) and BC-2.06.009 (rotate) are the write and read paths of the VecDeque.
- BC-2.06.011/012/013 (accept-once/always/reject) are the three decision keybindings that
  send ClientToServer::PermissionDecision to the daemon and pop from the VecDeque.
- BC-2.06.014 (Esc hides without popping) is the complement: the VecDeque is preserved.
- BC-2.06.016 (cleared on daemon disconnect) is SOQ-3's TUI side — the VecDeque is cleared
  when TransportEvent::Disconnected arrives. It is inseparable from the overlay lifecycle
  tests.
- BC-2.06.023 (TUI removes prompt on PermissionPromptResolved) is the IPC-initiated
  removal path — the handle_ipc_message() VecDeque retain() path. This is the "another TUI
  resolved it" or "hook timeout" notification path.
These 8 BCs exercise the same VecDeque<PromptModal> data structure from 8 different
angles. Testing one without the others would leave invariants untestable (e.g., you cannot
test that Esc preserves the stack without also testing that decide pops it).
13 points reflects the 8 P0 BCs, the IPC integration, and the transition() test coverage
requirements. This is a dense story but cannot be split further without breaking invariant
coverage.

---

#### S-027: Permission Overlay Rendering + Diff Preview + Status Bar

**Title:** Permission Overlay Rendering, Diff Preview (similar 3), Status Bar

**Epic:** EPIC-06

**BCs covered:**
- BC-2.06.010 — Permission Overlay: Diff Preview via `similar 3` (P1)
- BC-2.06.015 — Permission Overlay: `[t]` Trace-to-Source Stub (P2)
- BC-2.06.017 — Permission Response Within Hook Timeout Budget (P0)
- BC-2.06.019 — Status Bar: Drop Counter Renders Under Load (P0)
- BC-2.06.020 — Status Bar: Breadcrumb (P1)
- BC-2.06.021 — Status Bar: Keybinding Hint Line (P1)

**Points:** 8

**Dependencies:** S-026 (overlay VecDeque stack — diff preview renders the front
PromptModal's ToolPayload::Edit), S-025 (TUI binary skeleton — status bar is rendered
by draw_status_bar())

**Wave:** 7

**Rationale:**
This story completes the visual rendering layer of the permission overlay and status bar.
BC-2.06.017 (timeout budget) is P0 because it verifies that the 300ms PreToolUse
budget is not violated by TUI rendering latency — the overlay must appear within 100ms
of PermissionPromptQueued arrival. It is placed here (with S-026's overlay stack already
built) because it is a performance assertion on the rendering pipeline, not a behavioral
assertion on the overlay data structure.
BC-2.06.010 (diff preview via similar 3) is P1 because it only activates for
ToolPayload::Edit — a specific tool type. It renders the unified diff in the overlay.
BC-2.06.015 (trace-to-source stub) is P2 — a reserved keybinding with a placeholder
message. It is grouped here because it is part of the overlay rendering, not the
overlay state machine.
The status bar BCs (019, 020, 021) are rendering-only: drop counter (reads from IPC
StateUpdate), breadcrumb (derived from AppMode), and hint line (context-sensitive one-liner).
They are grouped here because they all render in draw_status_bar() and are mutually
co-located.
Wave 7 because this depends on S-026 (Wave 6) for the overlay stack.

---

#### S-028: Sessions Panel Filter + Event Ribbon Panel

**Title:** Sessions Panel Nucleo Filter + Event Ribbon Rolling Log

**Epic:** EPIC-06

**BCs covered:**
- BC-2.06.006 — Sessions Panel: `/` Filter with Nucleo Fuzzy Match (P1)
- BC-2.06.018 — Event Ribbon Panel: Rolling Hook Event Log (P1)

**Points:** 5

**Dependencies:** S-025 (sessions panel and TUI binary skeleton), S-021 (IPC
HookEventReceived message type used by event ribbon)

**Wave:** 7

**Rationale:**
BC-2.06.006 (nucleo filter) and BC-2.06.018 (event ribbon) are both P1 secondary
features that build on top of the primary panels from S-025. They are grouped together
because:
- Both are pure rendering enhancements that do not modify the core overlay lifecycle.
- Both are P1 (not on the critical path for the killer scenario).
- The nucleo filter is exercised via the Filtering AppMode transition already tested
  in S-024; this story wires it to the actual sessions data and renders highlights.
- The event ribbon simply renders HookEventReceived messages from the IPC receive buffer.
Wave 7 because both depend on S-025 (Wave 6) for the TUI skeleton.

---

#### S-029: Killer Scenario Integration Test

**Title:** Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve (BC-2.06.022)

**Epic:** EPIC-06

**BCs covered:**
- BC-2.06.022 — Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve (P0)

**Points:** 5

**Dependencies:** S-026 (permission overlay VecDeque + decisions), S-027 (overlay
rendering — must be renderable to validate the scenario), S-022 (IPC — two
PermissionPromptQueued messages must arrive), S-018 (hook routing + event bus — the
two PreToolUse hooks arrive and are published)

**Wave:** 7

**Rationale:**
BC-2.06.022 is the product's success criterion: 4 keystrokes (or ≤6 maximum) to resolve
two concurrent permission prompts. This is an end-to-end integration test, not a unit
test. It requires the full stack to be working:
1. Daemon receives two PreToolUse hooks (S-018).
2. Daemon publishes two PermissionPromptQueued IPC messages (S-022).
3. TUI pushes both to VecDeque<PromptModal> (S-026).
4. TUI overlay is rendered (S-027).
5. User presses 2 keys (AcceptAlways + AcceptOnce) → two decisions.
6. Both HTTP responses are sent to Claude Code.
The 5-point estimate reflects that the behavior itself is proven by S-026 unit tests;
this story adds the integration test harness that simulates the full flow end-to-end
(daemon running, two concurrent hooks, TUI connected, keystrokes simulated).
Wave 7 because all three upstream stories (S-026, S-027, S-022) must be complete.

---

### EPIC-07: Config (SS-07)

---

#### S-030: Config Crate Foundation + Atomic Write + Schema v1

**Title:** Config Crate: Atomic Write, Schema v1, Missing/Corrupted Default, CCR Detection

**Epic:** EPIC-07

**BCs covered:**
- BC-2.07.001 — Config File Atomic Write via `tempfile::persist` (P0)
- BC-2.07.002 — Config Schema Version 1: Harness Profile Fields (P0)
- BC-2.07.003 — Config Missing or Corrupted: Default Applied (P0)
- BC-2.07.006 — CCR Detection via `ccr_path` Config Field (P1)

**Points:** 5

**Dependencies:** S-001 (Cargo workspace init — monocle-config is a new workspace member)

**Wave:** 4

**Rationale:**
These four BCs are all foundational behaviors of the `monocle-config` crate that must exist
before the crate can be used by any other crate (monocle-runtime, monocle-tui, monocle binary).
BC-2.07.001 (atomic write) is the write contract — all writes must use tempfile::persist.
BC-2.07.002 (config schema v1) defines the data model — without this, there is nothing to
write or read.
BC-2.07.003 (missing/corrupted → default) is the resilience contract — without this, daemon
startup fails on first run.
BC-2.07.006 (CCR detection) is P1 but is placed here because it is a pure-core function
(`detect_ccr()`) with no dependency on TUI or daemon wiring. It uses `which::which("ccr")`
and is straightforward to implement alongside the other pure-core config logic.
Wave 4 is appropriate because monocle-config has NO dependencies on any new SS-04/05/06
stories — it only depends on S-001 (workspace init, Wave 1). This story can be implemented
in parallel with S-016 (daemon CLI) in Wave 4.

---

#### S-031: Profile Picker Sticky Selection + Ctrl-P Override

**Title:** Profile Picker: Sticky-Per-Project Selection + Ctrl-P Override

**Epic:** EPIC-07

**BCs covered:**
- BC-2.07.004 — Profile Picker: Sticky-Per-Project (P1)
- BC-2.07.005 — Profile Picker: `Ctrl-P` Override Shows Picker (P1)

**Points:** 5

**Dependencies:** S-030 (config schema + load/write — project_profiles map is the storage
for sticky selection), S-024 (Action::ProfilePicker defined in monocle-core — BC-2.07.005
uses this Action variant), S-025 (TUI binary skeleton — the picker is rendered by monocle-tui
using Option<ProfilePickerState>)

**Wave:** 7

**Rationale:**
BC-2.07.004 (sticky-per-project) and BC-2.07.005 (Ctrl-P) are companion P1 features that
both involve the profile picker interaction:
- BC-2.07.004 reads project_profiles from config.json at TUI startup to pre-select the profile.
- BC-2.07.005 opens the picker on Ctrl-P, writes the selection to project_profiles, and applies
  the profile to the running daemon session.
These are P1 (not blocking earlier waves) because they are convenience features (pre-selected
profile) and override behavior. The sticky selection (BC-2.07.004) is tested during daemon
startup, which requires monocle-runtime to be wired (S-017). The Ctrl-P override (BC-2.07.005)
requires the TUI binary to be running (S-025).
Wave 7 because both S-024 (Wave 4) and S-025 (Wave 6) must be complete for TUI rendering,
and S-030 (Wave 4) must be complete for config persistence.

---

## Wave Assignments

| Wave | Stories | Points | Description |
|------|---------|--------|-------------|
| Wave 4 | S-016, S-024, S-030 | 18 | Foundation layer: binary CLI + config crate + TUI core types (all depend only on Wave 1-3) |
| Wave 5 | S-017, S-018, S-019, S-020, S-021 | 34 | Daemon wiring + IPC server foundation (S-017 unlocks daemon; S-018/S-021 build on it) |
| Wave 6 | S-022, S-023, S-025, S-026 | 34 | IPC client + TUI rendering + permission overlay core |
| Wave 7 | S-027, S-028, S-029, S-031 | 23 | Rendering polish + integration scenario + profile picker |

**Total points:** 109 (across 16 new stories)
**Running total with Waves 1-3:** 83 + 109 = 192 points

---

## BC Coverage Matrix (All 49 New BCs)

| BC ID | Title | Covering Story |
|-------|-------|---------------|
| BC-2.04.001 | Daemon Start Sequence: Port Bind + Lock File + Token Write (SOQ-2) | S-017 |
| BC-2.04.002 | Daemon Auto-Start on TUI Launch | S-019 |
| BC-2.04.003 | `MONOCLE_NO_AUTOSTART=1` Suppresses Auto-Start | S-019 |
| BC-2.04.004 | `monocle daemon start` CLI Subcommand | S-016 |
| BC-2.04.005 | `monocle daemon stop` CLI Subcommand | S-016 |
| BC-2.04.006 | `directories::ProjectDirs::runtime_dir()` Fallback Chain | S-016 |
| BC-2.04.007 | Hook Endpoint: PreToolUse Request Routing | S-018 |
| BC-2.04.008 | Hook Endpoint: Notification Request Routing (2000ms Timeout) | S-018 |
| BC-2.04.009 | Hook Endpoint: Stop/SessionStart/PromptSubmit Routing (300ms Timeout) | S-018 |
| BC-2.04.010 | Hook Tmpfile Generation at `runtimeDir/hooks-settings.json` | S-017 |
| BC-2.04.011 | Bounded Event Bus with Drop Counter | S-018 |
| BC-2.04.012 | JSONL Ring: Capacity and Rotation Policy | S-020 |
| BC-2.05.001 | UDS Server Bind at `runtimeDir/monocle.sock` | S-021 |
| BC-2.05.002 | TUI Client Connects to UDS and Receives Initial State Push | S-022 |
| BC-2.05.003 | IPC Message Types: SessionListUpdate | S-021 |
| BC-2.05.004 | IPC Message Types: HookEventReceived | S-021 |
| BC-2.05.005 | IPC Message Types: PermissionPromptQueued | S-022 |
| BC-2.05.006 | TUI Reconnects After Daemon Restart | S-023 |
| BC-2.05.007 | Overlay Stack Cleared on Daemon Disconnect (SOQ-3) | S-023 |
| BC-2.05.008 | UDS-Only in Phase 1 (No Shared-Memory Transport) | S-021 |
| BC-2.06.001 | AppMode State Machine: Compile-Time Mutual Exclusion | S-024 |
| BC-2.06.002 | FocusSnapshot: Focus Restored After Overlay/Fullscreen Close | S-024 |
| BC-2.06.003 | Action Dispatch: 5-Level Binding Precedence | S-024 |
| BC-2.06.004 | `Ctrl-\` Popup: Appears and Dismisses Without State Loss | S-025 |
| BC-2.06.005 | Sessions Panel: Session List Renders from IPC State | S-025 |
| BC-2.06.006 | Sessions Panel: `/` Filter with Nucleo Fuzzy Match | S-028 |
| BC-2.06.007 | Sessions Panel: `Enter` Transitions to Fullscreen | S-025 |
| BC-2.06.008 | Permission Overlay: VecDeque Stack Push on PermissionPromptQueued | S-026 |
| BC-2.06.009 | Permission Overlay: `[↑↓]` Rotates Stack | S-026 |
| BC-2.06.010 | Permission Overlay: Diff Preview via `similar 3` | S-027 |
| BC-2.06.011 | Permission Overlay: Accept-Once Keybinding | S-026 |
| BC-2.06.012 | Permission Overlay: Accept-Always Keybinding | S-026 |
| BC-2.06.013 | Permission Overlay: Reject Keybinding | S-026 |
| BC-2.06.014 | Permission Overlay: `[Esc]` Hides Without Rejecting | S-026 |
| BC-2.06.015 | Permission Overlay: `[t]` Trace-to-Source Stub | S-027 |
| BC-2.06.016 | Permission Overlay: Cleared on Daemon Disconnect | S-026 |
| BC-2.06.017 | Permission Response Within Hook Timeout Budget | S-027 |
| BC-2.06.018 | Event Ribbon Panel: Rolling Hook Event Log | S-028 |
| BC-2.06.019 | Status Bar: Drop Counter Renders Under Load | S-027 |
| BC-2.06.020 | Status Bar: Breadcrumb | S-027 |
| BC-2.06.021 | Status Bar: Keybinding Hint Line | S-027 |
| BC-2.06.022 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | S-029 |
| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | S-026 |
| BC-2.07.001 | Config File Atomic Write via `tempfile::persist` | S-030 |
| BC-2.07.002 | Config Schema Version 1: Harness Profile Fields | S-030 |
| BC-2.07.003 | Config Missing or Corrupted: Default Applied | S-030 |
| BC-2.07.004 | Profile Picker: Sticky-Per-Project | S-031 |
| BC-2.07.005 | Profile Picker: `Ctrl-P` Override Shows Picker | S-031 |
| BC-2.07.006 | CCR Detection via `ccr_path` Config Field | S-030 |

**Coverage: 49/49 BCs covered (100%)**

---

## Dependency Graph (Text Format)

```
Wave 4 (parallel-eligible within wave):
  S-016 (daemon CLI)         ← S-001, S-006
  S-024 (TUI core types)     ← S-011, S-014
  S-030 (config crate)       ← S-001

Wave 5 (parallel-eligible within wave; all depend on Wave 4):
  S-017 (daemon start seq)   ← S-016, S-006, S-008, S-009, S-015, S-012
  S-018 (hook routing + bus) ← S-017, S-002, S-003, S-004, S-009, S-014
  S-019 (auto-start)         ← S-016, S-017
  S-020 (ring rotation)      ← S-008, S-017
  S-021 (UDS server + types) ← S-017, S-014, S-013

Wave 6 (parallel-eligible within wave; all depend on Wave 5):
  S-022 (TUI connect + push) ← S-021, S-018
  S-023 (reconnect + SOQ-3)  ← S-022, S-019
  S-025 (TUI binary + sess)  ← S-024, S-022, S-030
  S-026 (overlay core)       ← S-024, S-022, S-023

Wave 7 (parallel-eligible within wave; all depend on Wave 6):
  S-027 (overlay render + bar) ← S-026, S-025
  S-028 (filter + ribbon)     ← S-025, S-021
  S-029 (killer scenario)     ← S-026, S-027, S-022, S-018
  S-031 (profile picker)      ← S-030, S-024, S-025
```

### Topological Sort Verification

Wave 4 has no dependencies on new stories (only Wave 1-3). Wave 5 depends on Wave 4.
Wave 6 depends on Wave 5. Wave 7 depends on Wave 6. No cycles detected.

**Critical path:** S-001 → S-016 → S-017 → S-021 → S-022 → S-026 → S-027 → S-029

---

## Notable Decisions and Constraints

### Why S-026 is 13 points

BC-2.06.016 (overlay cleared on daemon disconnect) and BC-2.06.023 (TUI removes prompt
on PermissionPromptResolved) must live in the same story as the rest of the overlay
stack lifecycle because they test invariants of the same VecDeque<PromptModal>:
- Invariant: the VecDeque is never left in an inconsistent state.
- BC-2.06.016 verifies the SOQ-3 clear path (VecDeque emptied).
- BC-2.06.023 verifies the IPC-initiated retain() path (specific prompt_id removed).
If these were separate stories, the VecDeque invariant coverage would be split, making
it impossible to write a comprehensive property test that covers all state transitions.
13 points is under the 13-point maximum and is the correct atomic boundary.

### SS-04 Wave 5 Cohesion

S-017 (daemon start sequence) is Wave 5, NOT Wave 4, because it depends on:
- S-016 (Wave 4) for the binary crate that calls daemon_start_sequence().
- S-006 (Wave 2) for the lock file implementation.
- S-008 (Wave 3) for the ring buffer.
- S-009 (Wave 3) for the auth token.
- S-015 (Wave 3) for ClaudeCodeModule.
- S-012 (Wave 3) for VsddFactoryAdapter.
All Wave 3 prerequisites are done. S-017 is Wave 5 because S-016 (Wave 4) must exist first.

### Config Crate Is Wave 4 (Not Later)

`monocle-config` (S-030) has NO dependency on any new SS-04/05/06 stories. It only
requires the workspace (S-001, Wave 1). It can be built in Wave 4 alongside S-016 and
S-024, which is important because S-025 (Wave 6) depends on S-030 for profile resolution
at TUI startup. Building config in Wave 4 creates a 2-wave buffer before it is needed.

### TUI Core Types Before IPC

S-024 (AppMode, Action, transition() — Wave 4) is placed before IPC stories because
monocle-core types are consumed by monocle-ipc (S-021) and monocle-tui (S-025). The
types must exist as compiled Rust before the crates that use them can be built.

### SOQ-3 Split Across SS-05 and SS-06

SOQ-3 spans two BCs in two subsystems:
- BC-2.05.007 (IPC layer — signals TransportEvent::Disconnected) → S-023
- BC-2.06.016 (TUI layer — clears VecDeque on TransportEvent::Disconnected) → S-026
Both are in Wave 6. The IPC side (S-023) does not depend on S-026; S-026 depends on
S-023 (one-directional). The split reflects the clean monocle-ipc / monocle-tui crate
boundary.

---

## Summary

| Metric | Value |
|--------|-------|
| New epics | 4 (EPIC-04, EPIC-05, EPIC-06, EPIC-07) |
| New stories | 16 |
| Total new BCs covered | 49/49 (100%) |
| New story points | 109 |
| Running total (Waves 1-7) | 192 pts |
| Waves | 4, 5, 6, 7 |
| Wave 4 points | 18 (S-016=5, S-024=8, S-030=5) |
| Wave 5 points | 34 (S-017=8, S-018=8, S-019=5, S-020=5, S-021=8) |
| Wave 6 points | 34 (S-022=8, S-023=5, S-025=8, S-026=13) |
| Wave 7 points | 23 (S-027=8, S-028=5, S-029=5, S-031=5) |
| Max story points | 13 (S-026) — at the allowed ceiling |
| Stories at ceiling (13pts) | 1 (S-026 — justified above) |
| Dependency cycles | 0 (topological sort verified) |
