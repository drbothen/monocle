---
document_type: behavioral-contract
level: L3
version: "1.9.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:45:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-ipc.md, architecture/SS-daemon-wiring-v2-delta.md, architecture/SS-session-manager.md, architecture/SS-engine-module-v2-delta.md]
input-hash: "2fa6c9e"
traces_to: prd.md
origin: greenfield
subsystem: SS-05
capability: CAP-005
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

# Behavioral Contract BC-2.05.010: New ClientToServer IPC Variants — SpawnSession, KillSession, KeyInput, ResizePane, DetachSession, RenameSession, AttachSession

## Description

v1A adds seven `ClientToServer` IPC message variants for session lifecycle operations and
re-attach. The TUI sends these messages to the daemon's UDS server, which routes them to
`SessionManager`. Each variant is `#[non_exhaustive]` per BC-2.02.003. The daemon handles
each variant by delegating to the corresponding `SessionManager` method and broadcasting
state updates to all TUI clients. `AttachSession` (7th variant, I3-004) was added in v1.2.0
to support TUI-initiated re-attach after `PtyReset` and explicit user re-attach of Detached
sessions. The TUI MUST NOT send `DaemonToHost::Attach` directly — that is a daemon-only
message; `ClientToServer::AttachSession` is the correct TUI→daemon message.

## Preconditions

1. TUI client is connected to the daemon's UDS.
2. The daemon's IPC handler processes `ClientToServer` messages.

## Postconditions

### SpawnSession

1. `ClientToServer::SpawnSession { opts: SpawnOptions }` is received. The TUI populates
   `project_root`, `worktree_root`, `harness_id`, `profile_id`, and `ccr_base_url`;
   the daemon IPC handler fills `session_id` and `hooks_settings_path` before calling
   `spawn_session()`. `SpawnRecipe` is daemon-internal — built inside `spawn_session()` by
   `engine_module.spawn_recipe(&opts)?` as its first step (Model A — I27-001).
2. Daemon calls `SessionManager::spawn_session(opts)` (BC-2.08.001).
3. On success: `ServerToClient::SessionStateChanged { session_id, new_state }` broadcast BEFORE
   `ServerToClient::SessionListUpdate` broadcast (ordered pair, per-client FIFO, per BC-2.08.008
   PC-3 / SS-daemon-wiring-v2-delta.md v1.11.4 §3b). The `SessionStateChanged` event reflects
   the `Launching → Running` transition; the ordering is atomically guaranteed under the
   `SessionManager` mutex hold into each client's per-client `mpsc::Sender`.
4. On failure: `ServerToClient::Error { code: <spawn-path-code>, message: ... }` sent to the
   requesting client. The specific code depends on the `SessionError` variant returned by
   `spawn_session()`, mapped via `session_error_to_code(IpcOp::Spawn, &e)` (per
   SS-session-manager.md §session_error_to_code):
   - `"binary_not_found"` — `EngineError::BinaryNotFound` (harness binary not on PATH)
   - `"invalid_spawn_arg"` — `EngineError::InvalidPath` (bad argument to spawn_recipe())
   - `"spawn_unsupported"` — `EngineError::UnsupportedOperation` (harness does not support
     monocle-controlled spawning; EC-112 defensive path — F-P44-IMP-001; ProfilePicker
     capability filtering is best-effort, so this path is REACHABLE when a non-spawning
     harness profile reaches the session creation wizard)
   - `"spawn_failed"` — `SessionError::SpawnFailed` (OS-level spawn failure after PATH resolve)
   - `"sidecar_write_failed"` — `SessionError::SidecarWriteFailed` (session-state.json write failed)
   - `"session_id_collision"` — `SessionError::SessionIdCollision` (UUID collision; exceedingly rare)
   - `"invalid_request"` — `SessionError::Io` (unexpected I/O error) and any future `EngineError`
     variant not yet explicitly mapped (`_ =>` forward-compat arm in `session_error_to_code()`;
     note: `UnsupportedOperation` now has its own dedicated `"spawn_unsupported"` arm and no
     longer falls through to this catch-all — F-P44-IMP-001)

### KillSession

1. `ClientToServer::KillSession { session_id: String }` is received.
2. Daemon calls `SessionManager::kill_session(&session_id)` (BC-2.08.003).
3. On success: `ServerToClient::SessionStateChanged { session_id, new_state }` broadcast BEFORE
   `ServerToClient::SessionListUpdate` broadcast (ordered pair, per-client FIFO, per BC-2.08.008
   PC-3 / SS-daemon-wiring-v2-delta.md v1.11.4 §3b). The `SessionStateChanged` event reflects
   the `Running → Terminating` transition (emitted immediately; `Terminating → Terminated`
   follows when the session-host confirms exit per BC-2.08.003).
4. On failure: `ServerToClient::Error { code: <kill-path-code>, message: ... }` sent to the
   requesting client per the No-silent-failure invariant (Invariant 6). Reachable failure codes,
   per `session_error_to_code(IpcOp::Kill, &e)` (SS-session-manager.md §Mapping table lines 440/445):
   - `"session_not_found"` — `SessionError::SessionNotFound` (session_id not in registry)
   - `"kill_failed"` — `SessionError::SessionHostDead` on the kill-path (session-host PID dead
     when daemon attempts to send `DaemonToHost::Kill`; `session_error_to_code(IpcOp::Kill, SessionHostDead)`
     → `"kill_failed"` via the op-aware branch)

### KeyInput

1. `ClientToServer::KeyInput { session_id: String, bytes: Vec<u8> }` is received.
2. Daemon calls `SessionManager::send_key_input(&session_id, bytes)`.
3. No broadcast — key input is fire-and-forget; no acknowledgement message sent back.
4. On failure, the error code depends on the `SessionError` variant returned by `send_key_input()`,
   mapped via `session_error_to_code(IpcOp::KeyInput, &e)` per SS-session-manager.md §session_error_to_code:
   - `"session_not_found"` — `SessionError::SessionNotFound` (session_id unknown)
   - `"attach_failed"` — `SessionError::SessionHostDead` (session exists but its session-host
     process died; `session_error_to_code(IpcOp::KeyInput, SessionHostDead)` routes to
     `"attach_failed"` per the exhaustive mapping table §session_error_to_code attach-path arm)

### ResizePane

1. `ClientToServer::ResizePane { session_id: String, rows: u16, cols: u16 }` is received.
2. Daemon's IPC handler clamps zero dimensions to 1 (`rows = max(rows, 1); cols = max(cols, 1)`)
   per Invariant 5 before calling `resize_session()`.
3. Daemon calls `SessionManager::resize_session(&session_id, rows, cols)`.
4. No broadcast — resize is fire-and-forward; no `ServerToClient::Error` sent on failure.
5. **No-silent-failure EXCEPTION (ResizePane carve-out):** `ResizePane` is the ONLY
   `ClientToServer` variant exempt from the No-silent-failure invariant (see §No-silent-failure
   invariant below). Failures from `resize_session()` are WARN-logged and dropped — no
   `ServerToClient::Error` is sent to the requesting client. Rationale: after the zero-dimension
   clamp, the reachable `resize_session()` failure paths are (F-P51-001):
   - `SessionError::SessionNotFound` — benign race: the session terminated between the TUI sending
     the resize and the daemon processing it. Expected at session teardown; surfacing it would
     create spurious "Session not found" popups during normal session exit.
   - `SessionError::SessionNotReady` — session is in Launching state with `host_conn: None`
     (post-spawn monitor not yet connected). This is an untrusted-client path; the official TUI
     prevents sending `ResizePane` during Launching via state guards. Surfacing it would create
     spurious popups during normal session startup.
   Both paths are WARN-dropped. The WARN-drop carve-out covers ALL `resize_session()` errors
   without enumeration — F-P51-001 strengthens, not narrows, this invariant. Per
   SS-session-manager.md v2.6.1 §ResizePane special rule (lines 572-576).

### DetachSession

1. `ClientToServer::DetachSession { session_id: String }` is received.
2. Daemon calls `SessionManager::detach_session(&session_id)` (BC-2.08.007).
3. On success: `ServerToClient::SessionStateChanged { session_id, new_state: Detached }` broadcast
   BEFORE `ServerToClient::SessionListUpdate` broadcast (ordered pair, per-client FIFO, per
   BC-2.08.008 PC-3 / SS-daemon-wiring-v2-delta.md v1.11.4 §3b).
4. On failure: `ServerToClient::Error { code: <detach-path-code>, message: ... }` sent to the
   requesting client. `DetachSession` is user-initiated — failures MUST surface (not dropped) per
   the No-silent-failure invariant below. The reachable failure codes, per
   `session_error_to_code(IpcOp::Detach, &e)` (SS-session-manager.md v2.6.1 §Mapping table line 440):
   - `"session_not_found"` — `SessionError::SessionNotFound` (session_id not in registry or
     already terminated). Note: `SessionHostDead` is NOT reachable on the detach-path; `detach_session()`
     aborts the proxy task rather than attempting a new connection to the session-host.
   - `"session_not_ready"` — `SessionError::SessionNotReady` (session in Launching state; `host_conn: None`;
     post-spawn monitor not yet connected). Defensive: the official TUI never sends `DetachSession`
     during Launching (BC-2.08.007 Precondition 1 requires `state: Running`; BC-2.06.025 guards).
     Reachable from untrusted clients with socket access. (F-P51-001)
   See EC-284b.

### RenameSession

1. `ClientToServer::RenameSession { session_id: String, new_name: String }` is received.
2. Daemon calls `SessionManager::rename_session(&session_id, new_name)`.
3. On success: `ServerToClient::SessionListUpdate` broadcast. NOTE: `SessionStateChanged` is
   NOT emitted for rename — rename is not a `SessionState` transition (per BC-2.08.008 PC-4a).
4. On failure: `ServerToClient::Error { code: <rename-path-code>, message: ... }` sent to the
   requesting client per the No-silent-failure invariant below. Reachable failure codes, per
   `session_error_to_code(IpcOp::Rename, &e)` (SS-session-manager.md §Mapping table):
   - `"session_not_found"` — `SessionError::SessionNotFound` (session_id not in registry)
   - `"rename_failed"` — `SessionError::InvalidSessionName` (empty name or name exceeding
     length limit; per SS-session-manager.md line 446 + EC-283)

### AttachSession

1. `ClientToServer::AttachSession { session_id: String }` is received.
2. Daemon calls `SessionManager::attach_session(&session_id)` (BC-2.08.007).
3. On success: the session-host streams a fresh `HostToDaemon::ScrollbackChunk*` +
   `HostToDaemon::ScrollbackDumpComplete` sequence; the daemon fans these out as
   `ServerToClient::ScrollbackChunk` / `ServerToClient::ScrollbackDumpComplete` to all
   connected TUI clients (per BC-2.05.011). If state transitions from `Detached → Running`,
   `ServerToClient::SessionStateChanged { session_id, new_state: Running }` is broadcast
   BEFORE `ServerToClient::SessionListUpdate`.
4. On failure: `ServerToClient::Error { code: <attach-path-code>, message: ... }` sent to the
   requesting client per the No-silent-failure invariant below. Reachable failure codes, per
   `session_error_to_code(IpcOp::Attach, &e)` (SS-session-manager.md §Mapping table lines 440/444):
   - `"session_not_found"` — `SessionError::SessionNotFound` (session_id not in registry)
   - `"attach_failed"` — `SessionError::SessionHostDead` (session-host PID dead when daemon
     attempts attach; `session_error_to_code(IpcOp::Attach, SessionHostDead)` → `"attach_failed"`)
5. Use cases: (a) TUI re-attach after `PtyReset` (BC-2.05.011 PC-3c), (b) user explicitly
   re-attaches a `Detached` session from the sessions panel. The TUI MUST NOT send
   `DaemonToHost::Attach` directly — that is a daemon→session-host message. The TUI sends
   `ClientToServer::AttachSession` to the daemon, which routes to `SessionManager::attach_session()`.
   Per SS-ipc.md v1.24.0 §`ClientToServer::AttachSession`.

## Invariants

1. All seven variants are `#[non_exhaustive]` fields per ADR-0006 (non-exhaustive structs with
   public constructors). The message enum itself is `#[non_exhaustive]` per BC-2.02.003.
2. `KeyInput` and `ResizePane` are high-frequency messages. The IPC handler MUST process
   them with minimal latency (no locking beyond `Arc<Mutex<SessionManager>>.lock()`).
3. `session_id` is a `String` (UUID rendered as string) at all IPC boundaries per
   SS-session-manager.md §session_id type ruling.
4. Unknown variants (future additions from newer TUI to older daemon) are silently ignored
   per `#[non_exhaustive]` forward-compat policy.
5. **Zero-dimension clamp (S2-004 consistency rule):** The daemon IPC handler MUST clamp
   `ResizePane.rows` and `ResizePane.cols` to a minimum of 1 before forwarding to
   `resize_session()`. This rule is applied at the daemon boundary (not in the TUI). The TUI
   (BC-2.09.006) should also prevent sending zero dimensions, but the daemon is the final
   enforcement point. Clamping prevents undefined PTY behavior without surfacing an error to
   the TUI. Cross-reference: BC-2.09.006 EC-237 (TUI-side resize no-op detection).
6. **No-silent-failure invariant:** The daemon IPC handler MUST NOT return `Ok(())` at the
   task boundary on `Err(SessionError::...)` from any lifecycle operation (SpawnSession,
   KillSession, KeyInput, DetachSession, RenameSession, AttachSession). Every error from
   `SessionManager` MUST produce `ServerToClient::Error { code, message }` to the requesting
   client over its per-client channel. This includes `EngineError`-derived errors bridged
   through `SessionError::EngineError` on the spawn path. Swallowing to `Ok(())` leaves the
   TUI in a stale `Launching` (or other) state with no user-visible feedback — a silent failure.
   **Exception — ResizePane only:** `ResizePane` is the sole carve-out. After the zero-dimension
   clamp (Invariant 5), the only remaining `resize_session()` failure is `SessionNotFound` —
   a benign race where the session terminated between the TUI sending the resize and the daemon
   processing it. These failures are WARN-logged and dropped; no `ServerToClient::Error` is sent.
   Per SS-ipc.md v1.24.0 lines 1515/389 (normative citation) and SS-session-manager.md v2.6.1 lines
   572-576 (ResizePane special rule rationale). Also cited by SS-session-manager.md line 385
   as "BC-2.05.010 §No-silent-failure invariant" — this section is the named target of that
   forward-reference.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-280 | `SpawnSession` where the harness binary (`claude`) is not on `PATH` (e.g., not installed, or `PATH` misconfigured) | Inside `spawn_session()`, `engine_module.spawn_recipe(&opts)?` calls `which::which("claude")`, which fails → `EngineError::BinaryNotFound("claude")` → `SessionError::EngineError(BinaryNotFound)` → `ServerToClient::Error { code: "binary_not_found", message: ... }` sent back. No OS process is spawned. `SpawnFailed` is NOT triggered — `SpawnFailed` is reserved for OS-level spawn failures after the binary is located on PATH (Model A — I27-001). |
| EC-281 | `KeyInput` for unknown `session_id` | `SessionError::SessionNotFound`; `ServerToClient::Error { code: "session_not_found", message: ... }` sent to requesting client |
| EC-281b | `KeyInput` for a session whose session-host process has died (session exists in registry but host dead) | `SessionError::SessionHostDead { session_id }`; `session_error_to_code(IpcOp::KeyInput, SessionHostDead)` → `"attach_failed"`; `ServerToClient::Error { code: "attach_failed", message: ... }` sent to requesting client. The session subsequently transitions to `Terminated` via the daemon's dead-host detection path. |
| EC-282 | `ResizePane` with `rows=0` or `cols=0` | The daemon's IPC handler MUST clamp each dimension to a minimum of 1 BEFORE forwarding to `resize_session()`. `rows = max(rows, 1); cols = max(cols, 1)`. The PTY and parser are resized to the clamped values. No `SessionError` is returned; the operation succeeds with clamped dimensions. Clamping is consistent with BC-2.09.006 EC-237 and the resize behavior in the TUI. A zero-dimension PTY is undefined by POSIX; clamping to 1 is the most robust behavior. |
| EC-283 | `RenameSession` with empty `new_name` | `SessionError::InvalidSessionName`; `ServerToClient::Error { code: "rename_failed", message: ... }` sent to requesting client |
| EC-284 | Concurrent `KeyInput` messages from the same TUI client | Processed in order of arrival; each forwarded to session-host in receipt order |
| EC-284b | `DetachSession` for a `session_id` that has already terminated (race: session exits between TUI dispatch and daemon processing) | `SessionError::SessionNotFound`; `session_error_to_code(IpcOp::Detach, SessionNotFound)` → `"session_not_found"`; `ServerToClient::Error { code: "session_not_found", message: ... }` sent to requesting client. Per No-silent-failure invariant (Invariant 6) — DetachSession is user-initiated and MUST surface failures. |

## Canonical Test Vectors

| Message | Expected Daemon Response | Category |
|---------|-------------------------|----------|
| `SpawnSession { opts: valid SpawnOptions, claude on PATH }` | `SessionStateChanged{Launching}` then `SessionListUpdate` broadcast; `spawn_session(opts)` called | happy-path |
| `KillSession { session_id: "existing" }` | `SessionListUpdate` broadcast | happy-path |
| `KillSession { session_id: "nonexistent" }` | `Error { code: "session_not_found" }` to requesting client | error |
| `KeyInput { session_id: "running", bytes: [0x61] }` | `send_key_input()` called; no broadcast | happy-path |
| `ResizePane { rows: 30, cols: 120 }` | `resize_session()` called; no broadcast | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 7 variants routed to correct `SessionManager` methods (incl. AttachSession → attach_session()) | integration |
| VP-TBD | `KeyInput` and `ResizePane` generate no broadcast (fire-and-forget) | unit |
| VP-TBD | Unknown `ClientToServer` variant handled without panic | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability §SS-05 |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability — these ClientToServer variants extend the internal transport capability with session lifecycle control messages (spawn, kill, key input, resize, detach, rename, re-attach) — all transported over the existing UDS per the session/event/prompt push design |
| Architecture Module | monocle-ipc (`ClientToServer` enum new variants); monocle-runtime (IPC handler routing to SessionManager) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-daemon-wiring-v2-delta.md v1.11.4 §IPC handler — new ClientToServer variants (including AttachSession; `ClientToServer::SpawnSession { opts: SpawnOptions }` wire variant under Model A — I27-001); SS-ipc.md v1.24.0 §`ClientToServer::SpawnSession { opts: SpawnOptions }` (Model A wire variant — I27-001); SS-ipc.md v1.24.0 §`ClientToServer::AttachSession` (I3-004 — TUI re-attach; replaces incorrect "TUI sends DaemonToHost::Attach" description); SS-ipc.md v1.24.0 §`ServerToClient::Error` — Error variant + code taxonomy (`spawn_failed`, `binary_not_found`, `invalid_spawn_arg`, `spawn_unsupported`, `sidecar_write_failed`, `session_id_collision`, `session_not_found`, `attach_failed`, `kill_failed`, `rename_failed`, `invalid_request`, `session_not_ready`) — 12 codes as of v1.23.2; `spawn_unsupported` added F-P44-IMP-001; `session_not_ready` added F-P50-001; wire producer is DetachSession arm only (F-P51-001); added by architect in Pass-6 parallel track (C6-001); SS-session-manager.md v2.6.1 §session_error_to_code (spawn-path arms — Model A reachability for binary_not_found/invalid_spawn_arg confirmed I27-001; UnsupportedOperation → "spawn_unsupported" arm added F-P44-IMP-001; SessionNotReady producer-set: DetachSession arm only, resize excluded F-P51-001); SS-engine-module-v2-delta.md v1.6.0 §SessionError (9-variant enum including SessionNotReady — F-P50-001) |
| Cross-Ref | BC-2.08.001 (SpawnSession → spawn_session()); BC-2.08.003 (KillSession → kill_session()); BC-2.08.007 (DetachSession → detach_session()) |
| Test Name | test_BC_2_05_010_new_client_to_server_variants_routed |

## Related BCs

- [BC-2.08.001] — depends on: SpawnSession IPC triggers spawn_session()
- [BC-2.08.003] — depends on: KillSession IPC triggers kill_session()
- [BC-2.08.007] — depends on: DetachSession IPC triggers detach_session(); AttachSession IPC triggers attach_session()
- [BC-2.05.002] — composes with: existing IPC connection framework carries these new variants
- [BC-2.05.011] — depends on: AttachSession triggers ScrollbackChunk*/ScrollbackDumpComplete sequence fanned out by BC-2.05.011

## Architecture Anchors

- `architecture/SS-daemon-wiring-v2-delta.md#ipc-handler-new-clienttoserver-variants` — handler branches

## Story Anchor

S-047 — Implement new ClientToServer IPC variants and daemon routing

## VP Anchors

VP-TBD — IPC variant routing integration tests (filled after VP creation)

## §Trace v1.9.2

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-047** (2026-06-15):
- Story Anchor filled from Phase-2 Burst C story decomposition. No behavioral content changed.

## §Trace v1.9.0

**F-P50-001 — SessionNotReady (9th SessionError variant); 12th wire code session_not_ready; SS-ipc v1.23.0 / SS-session-manager v2.5.0 pins** (2026-06-14):

- **SessionError 9th variant (SessionNotReady):** `SessionError` now has 9 variants (SessionNotReady added F-P50-001). SessionNotReady is for defensive control-ops on not-ready sessions. `kill_session()` does NOT use it — kill on a not-yet-connected Launching session uses PID fallback → `kill_failed` (see BC-2.08.003 v1.4.0). No KillSession PC update needed for this variant.
- **12th wire code (session_not_ready):** `ServerToClient::Error` code taxonomy grows from 11 to 12 codes. `"session_not_ready"` added to the Architecture Source enumerated code list alongside `"invalid_request"` (SS-ipc.md v1.23.1 §`ServerToClient::Error`). Wire producer is the DetachSession IPC arm ONLY — see F-P51-001 errata note below.
- **Architecture Source pins bumped:** SS-daemon-wiring-v2-delta.md v1.11.0 → v1.11.1; SS-ipc.md v1.22.0 → v1.23.0 (all three inline citations); SS-session-manager.md v2.4.0 → v2.5.0; SS-engine-module-v2-delta.md v1.6.0 added (SessionError 9-variant enum source).
- §Trace v1.8.1 "now 11 codes total" annotated as historical (11 codes at v1.8.1; 12 codes at v1.9.0).
- Minor bump: v1.8.1 → v1.9.0 (normative: new wire code enumerated in Architecture Source; new SS pin citations).

SE-16d monotonicity: v1.9.0 timestamp 2026-06-14 > v1.8.1 timestamp 2026-06-14. PASS.

## §Trace v1.9.1

**F-P51-001 — session_not_ready wire producer corrected: DetachSession arm only; resize excluded; §DetachSession PC-4 and §ResizePane rationale expanded** (2026-06-14):

- **Errata (F-P51-001):** The v1.9.0 trace implied both `detach_session()` and `resize_session()` were wire producers of `session_not_ready`. This is incorrect. The wire producer is the DetachSession IPC arm ONLY: when `detach_session()` returns `Err(SessionError::SessionNotReady)`, the IPC handler emits `ServerToClient::Error { code: "session_not_ready" }`. `resize_session()` may also return `Err(SessionNotReady)` at the session-manager level when the session is in Launching state with `host_conn: None`; however, the ResizePane IPC arm WARN-drops ALL `resize_session()` errors (Invariant 6 Exception — §No-silent-failure invariant carve-out). No `ServerToClient::Error` is ever emitted from the ResizePane arm regardless of which `SessionError` variant is returned. SS-session-manager.md v2.5.0 → v2.5.1 (producer-set errata).
- **§DetachSession PC-4 expanded:** Added `"session_not_ready"` bullet covering `SessionError::SessionNotReady` (session in Launching state; `host_conn: None`; post-spawn monitor not yet connected). Defensive path — the official TUI never sends `DetachSession` during Launching (BC-2.08.007 Precondition 1 requires `state: Running`; BC-2.06.025 guards). Reachable from untrusted clients with socket access. (F-P51-001.)
- **§ResizePane rationale expanded:** PC-5 rationale updated to cover BOTH `SessionNotFound` (benign race — session terminated between TUI dispatch and daemon processing) AND `SessionNotReady` (session in Launching state, `host_conn: None` — untrusted-client path; the official TUI prevents this via state guards). Both are WARN-dropped. Surfacing either would create spurious popups during normal session teardown or startup. (F-P51-001.) WARN-drop carve-out (Invariant 6 Exception) preserved and strengthened, not narrowed.
- **Architecture Source pins bumped:** SS-session-manager.md v2.5.0 → v2.5.1; SS-daemon-wiring-v2-delta.md v1.11.1 → v1.11.2; SS-ipc.md v1.23.0 → v1.23.1 (all three inline citations).
- **§Trace v1.9.0 12th wire code bullet:** Second "session_not_ready" duplicate corrected to "invalid_request" (copy-paste error; `invalid_request` is the 11th code, `session_not_ready` the 12th).
- Patch bump: v1.9.0 → v1.9.1.

SE-16d monotonicity: v1.9.1 timestamp 2026-06-14 > v1.9.0 timestamp 2026-06-14. PASS.

## §Trace v1.8.1

**F-P44-IMP-001 resolution — `spawn_unsupported` added to SpawnSession PC-4; `invalid_request` scoped** (2026-06-14):

- **SpawnSession PC-4 — `"spawn_unsupported"` (new, 3rd in spawn-path list):** Added immediately
  after `"invalid_spawn_arg"` and before `"spawn_failed"`. Covers `EngineError::UnsupportedOperation`
  (harness does not support monocle-controlled spawning). This is the EC-112 defensive path per
  F-P44-IMP-001: ProfilePicker capability filtering is best-effort and does NOT guarantee that
  non-spawning harness profiles are excluded from the wizard. The daemon surfaces
  `ServerToClient::Error { code: "spawn_unsupported" }` when this path fires.
- **SpawnSession PC-4 — `"invalid_request"` (bullet updated):** `UnsupportedOperation` previously
  fell through to this catch-all arm. As of F-P44-IMP-001 / SS-session-manager.md v2.4.0,
  `UnsupportedOperation` has a dedicated `"spawn_unsupported"` arm. The `"invalid_request"` bullet
  now correctly covers only `SessionError::Io` (unexpected I/O error) and future `EngineError`
  variants not yet explicitly mapped in the `_ =>` forward-compat arm.
- **Architecture Source (pin bump):** SS-ipc.md v1.22.0 → v1.22.0 (across all three SS-ipc.md
  citations); SS-session-manager.md v2.4.0 → v2.4.0. `spawn_unsupported` added to the inline
  code taxonomy list (now 11 codes as of v1.8.1; 12 codes as of v1.9.0 — see §Trace v1.9.0).
- Patch bump: v1.8.0 → v1.8.1.

SE-16d monotonicity: v1.8.1 timestamp 2026-06-14 > v1.8.0 timestamp 2026-06-13. PASS.

## §Trace v1.8.0

**F-P36-IMP-002 — No-silent-failure invariant authored; DetachSession + ResizePane + RenameSession + AttachSession failure-path completeness** (2026-06-13 / D-278):
- F-P36-IMP-002: SS-session-manager.md line 385 cited "BC-2.05.010 §No-silent-failure invariant"
  as a forward-reference to a named section that did not exist in this BC, creating a dangling
  citation. SS-ipc.md lines 1515 and 389 state the canonical invariant verbatim. This fix
  authors the named target section so all three arch citations resolve.
- **Invariant 6 (new — §No-silent-failure invariant):** Authors the named invariant section with
  the general rule (every fallible SessionManager call's Err MUST produce ServerToClient::Error
  to the requesting client; the IPC handler MUST NOT swallow to Ok(())), and the explicit
  ResizePane carve-out (WARN-drop only, benign session-not-found race, zero-dim clamp already
  applied). The section heading "No-silent-failure invariant" is the exact forward-reference
  target cited in SS-session-manager.md:385 / SS-ipc.md:389.
- **ResizePane PC (rewritten):** Added explicit step for the zero-dimension clamp (moving Invariant 5
  detail into the PC flow for implementer clarity), updated PC-4 to state no ServerToClient::Error
  on failure, and added PC-5 documenting the no-silent-failure EXCEPTION with rationale cross-
  referencing SS-session-manager.md §ResizePane special rule lines 572-576.
- **DetachSession PC-4 (new):** Documents the `"session_not_found"` failure code for
  `SessionError::SessionNotFound` with explicit note that `SessionHostDead` is NOT reachable
  on the detach-path (detach aborts the proxy task, does not attempt a new host connection).
  EC-284b added for the race-condition case.
- **RenameSession PC-4 (new):** Documents both reachable failure codes: `"session_not_found"`
  (SessionNotFound) and `"rename_failed"` (InvalidSessionName per EC-283), per
  `session_error_to_code(IpcOp::Rename, &e)` mapping table.
- **AttachSession PC-4 (rewritten for precision):** Previously said `code: "attach_failed"` for
  both "session not found" and "session-host dead" failures — incorrect. SessionNotFound →
  `"session_not_found"`; SessionHostDead (attach-path) → `"attach_failed"`. These are distinct
  codes with distinct TUI banner texts. Rewritten with the complete two-code enumeration per
  `session_error_to_code(IpcOp::Attach, &e)` SS-session-manager.md §Mapping table lines 440/444.
- **Whole-class variant status (fix-the-whole-class):**
  - SpawnSession: COMPLETE — PC-4 full 6-code spawn-path enumeration (per §Trace v1.6.0).
  - KillSession: NOW COMPLETE — PC-4 rewritten to enumerate both reachable failure codes:
    `"session_not_found"` (SessionNotFound) and `"kill_failed"` (SessionHostDead on kill-path via
    op-aware `session_error_to_code(IpcOp::Kill, SessionHostDead)` branch). Previously only
    documented `"session_not_found"`. Fix-the-whole-class discipline applied in same burst.
  - KeyInput: COMPLETE — PC-4 full two-code enumeration (per §Trace v1.7.2).
  - ResizePane: NOW COMPLETE — explicit exception documented in PC-5 (this fix).
  - DetachSession: NOW COMPLETE — PC-4 added (this fix).
  - RenameSession: NOW COMPLETE — PC-4 added (this fix).
  - AttachSession: NOW COMPLETE (precision fix) — PC-4 rewritten from single-code to two-code (this fix).
- Minor bump: 1.7.2 → 1.8.0 (minor: new named invariant section; multiple PCs extended with
  error-path content; new EC-284b).

## §Trace v1.7.2

**S35-002 + arch-source pin v1.9.0→v1.9.1 — KeyInput reachable error code completeness** (2026-06-13 / D-277):
- S35-002: KeyInput PC-4 previously enumerated only `"session_not_found"` as the failure
  response. This is incomplete: `send_key_input()` can also return
  `SessionError::SessionHostDead` when the session exists in the registry but its session-host
  process has died. `session_error_to_code(IpcOp::KeyInput, SessionHostDead)` routes to
  `"attach_failed"` per the exhaustive mapping table in SS-session-manager.md
  §session_error_to_code attach-path arm. The omission was a no-silent-failure completeness
  gap — an implementer seeing only `"session_not_found"` would not know to handle the dead-host
  path at the IPC layer.
  - **KeyInput PC-4:** rewritten from single `"session_not_found"` to enumerated error table
    with both reachable codes: `"session_not_found"` (SessionNotFound) and `"attach_failed"`
    (SessionHostDead), each with triggering variant and routing citation.
  - **EC-281b (new):** documents the SessionHostDead → `"attach_failed"` KeyInput edge case
    with full behavior description including post-error session Terminated transition.
  - No other KeyInput behavior changed. Fire-and-forget postconditions (PC-3) and Invariant 2
    (high-frequency / low-latency path) are unchanged.
- Arch-source pin: SS-daemon-wiring-v2-delta.md v1.9.0 → v1.9.1 (all active citations).
- Patch bump: 1.7.1 → 1.7.2.

## §Trace v1.7.1

**Pass-32 — SUG-001: imprecise InvalidPath parenthetical corrected** (2026-06-13):

- **SUG-001 (SpawnSession PC-4 error table):** The `"invalid_spawn_arg"` row described
  `EngineError::InvalidPath` as "(bad argument in SpawnRecipe)". `SpawnRecipe` is the
  daemon-internal output struct built by `spawn_recipe()` — `InvalidPath` is not "in" it.
  `InvalidPath` is produced from a bad argument TO `spawn_recipe()`, specifically when
  `opts.hooks_settings_path.to_str()` returns `None` (non-UTF-8 / null bytes in the
  `SpawnOptions` field before the recipe can be built). Corrected to "(bad argument to
  spawn_recipe())" for precision.

- No behavioral content changed (the error code, triggering condition, and propagation
  chain are unchanged). Patch bump only.

## §Trace v1.7.0

**I27-001 (Model A) — SpawnSession wire variant: SpawnOptions replaces SpawnRecipe on the wire** (2026-06-13):

Under the I27-001 Model A adjudication, `ClientToServer::SpawnSession` carries
`opts: SpawnOptions` (not `recipe: SpawnRecipe`). `SpawnRecipe` is now daemon-internal —
built inside `SessionManager::spawn_session()` by `engine_module.spawn_recipe(&opts)?` as
its first step. This BC is updated to reflect the wire variant change and remove all residual
Model B (TUI-builds-recipe) text.

**Changes in this version:**

- **SpawnSession PC-1 (normative):** Updated from `ClientToServer::SpawnSession { recipe: SpawnRecipe }` to `ClientToServer::SpawnSession { opts: SpawnOptions }`. Added explicit field population split: TUI populates `project_root`, `worktree_root`, `harness_id`, `profile_id`, `ccr_base_url`; daemon IPC handler fills `session_id` and `hooks_settings_path`. Added Model A note that `SpawnRecipe` is daemon-internal.

- **SpawnSession PC-2 (normative):** Updated from `SessionManager::spawn_session(recipe, ...)` to `SessionManager::spawn_session(opts)`.

- **EC-280 (normative):** Description updated from "SpawnSession with a SpawnRecipe where binary is empty path" to "SpawnSession where the harness binary (claude) is not on PATH." Under Model A there is no `binary` field in the TUI-sent message; binary discovery happens entirely daemon-side via `which::which("claude")` inside `spawn_recipe()`. The expected behavior (`"binary_not_found"`) remains unchanged and correct.

- **Canonical Test Vectors:** SpawnSession happy-path row updated from `SpawnSession { recipe: valid }` to `SpawnSession { opts: valid SpawnOptions, claude on PATH }`. Expected output updated to note `SessionStateChanged{Launching}` precedes `SessionListUpdate` (per §Trace v1.5.0 / BC-2.08.008 PC-3).

- **Architecture Source:** Version pins updated to current canonical versions:
  SS-daemon-wiring-v2-delta.md `v1.7.0` → `v1.8.0`;
  SS-ipc.md `v1.18.0` → `v1.19.0`;
  Added SS-ipc.md v1.19.0 §`ClientToServer::SpawnSession { opts: SpawnOptions }` citation;
  Added SS-session-manager.md v2.0.0 §session_error_to_code citation confirming spawn-path
  code reachability under Model A.
  Full spawn-path code set already enumerated correctly (per §Trace v1.6.0); no code change needed.

- PC-4 spawn-path error table and EC-280 behavior (both `"binary_not_found"`) already
  correct from §Trace v1.6.0; only the trigger description was updated (not the behavior).

## §Trace v1.6.0

**S22-001 — SpawnSession PC-4 + EC-280: complete spawn-path error code set** (2026-06-13):
- S22-001 (Phase-1d Pass 22 SUGGESTION): SpawnSession PC-4 listed only `code: "spawn_failed"`
  as the failure response. This is incomplete. The 10-code taxonomy introduced in Pass-12
  (Per BC-2.03.007 PC-3/PC-7 and SS-session-manager.md §session_error_to_code) added distinct
  spawn-path codes: `"binary_not_found"` (EngineError::BinaryNotFound) and `"invalid_spawn_arg"`
  (EngineError::InvalidPath), plus `"sidecar_write_failed"` (SidecarWriteFailed) and
  `"session_id_collision"` (SessionIdCollision). The canonical `session_error_to_code(IpcOp::Spawn, &e)`
  function in SS-session-manager.md (line ~469) maps each SessionError variant exhaustively.
  - **SpawnSession PC-4**: updated from single `"spawn_failed"` to the full spawn-path code
    enumeration cross-referenced to `session_error_to_code(IpcOp::Spawn, &e)`. All 6 spawn-path
    codes documented with their triggering SessionError/EngineError variants.
  - **EC-280**: `SpawnSession` with empty `binary` path was mapped to `SessionError::SpawnFailed`
    / `"spawn_failed"`. This is incorrect: an empty binary path fails at the PATH-lookup stage
    (`which("")` → not found) → `EngineError::BinaryNotFound` → `"binary_not_found"`.
    `SessionError::SpawnFailed` is for OS-level spawn failures AFTER the binary is located on
    PATH (e.g., permission denied). Corrected: EC-280 now maps to `"binary_not_found"` with
    explanatory note.
  - Verified against SS-session-manager.md §session_error_to_code (line ~469): the exhaustive
    match for `EngineError::BinaryNotFound(_) => "binary_not_found"` is canonical and unambiguous.
- Version bump: 1.5.1 → 1.6.0 (minor: normative spawn-path code set extended and EC-280
  behavior corrected).

## §Trace v1.5.1

**I15-001 — Correct EC-283 error code from `invalid_request` to `rename_failed`** (2026-06-04):
- I15-001 (Phase-1d Pass 15 IMPORTANT): EC-283 specified `SessionError::InvalidSessionName` →
  `ServerToClient::Error { code: "invalid_request" }`. This is internally impossible: the canonical
  exhaustive `session_error_to_code()` function in SS-session-manager.md (line ~484) maps
  `SessionError::InvalidSessionName { .. } => "rename_failed"` UNCONDITIONALLY — no op-aware or
  content-aware branch routes any `InvalidSessionName` to `"invalid_request"`. Additionally,
  SS-ipc.md line ~405 defines `"invalid_request"` as "Validation failure BEFORE the SessionManager
  call", but an empty-name failure surfaces as `SessionError::InvalidSessionName` returned BY
  `rename_session()` — a post-call result, not a pre-call validation failure.
- **EC-283:** `code: "invalid_request"` corrected to `code: "rename_failed"`.
- **§Trace v1.3.0:** The rationale claiming `"invalid_request"` was correct for validation failures
  has been corrected. It now accurately states that `session_error_to_code()` maps `InvalidSessionName`
  unconditionally to `"rename_failed"` and that no pre-call validation layer exists that could route
  to `"invalid_request"`.
- Whole-file sweep performed: no other EC/PC/Invariant carries an inconsistent `invalid_request`/
  `rename_failed` mapping for `InvalidSessionName`. Architecture Source line (line ~166) correctly
  enumerates both `rename_failed` and `invalid_request` as codes in the taxonomy — this is accurate
  (both codes exist in the taxonomy; only the EC-283 assignment of `invalid_request` was wrong).

## §Trace v1.5.0

**S10-002 — Add ordered SessionStateChanged→SessionListUpdate emission to SpawnSession-PC-3, KillSession-PC-3, DetachSession-PC-3** (2026-06-04):
- S10-002 (Phase-1d Pass 10 SUGGESTION): SpawnSession PC-3, KillSession PC-3, and DetachSession
  PC-3 stated only "On success: `ServerToClient::SessionListUpdate` broadcast", omitting the
  required `SessionStateChanged` emission that precedes it for every state transition.
- Per BC-2.08.008 PC-3 (verified): "`SessionStateChanged` is enqueued BEFORE `SessionListUpdate`
  into each client's per-client FIFO channel … Both `.try_send()` calls are made while holding
  the `SessionManager` mutex, into the same per-client `mpsc::Sender` in the correct sequence."
  Per SS-daemon-wiring-v2-delta.md v1.5.0 §3b emission table: spawn/kill/detach each trigger
  the ordered pair (SessionStateChanged then SessionListUpdate).
- **SpawnSession PC-3**: updated to state `SessionStateChanged { session_id, new_state }` BEFORE
  `SessionListUpdate`; notes the `Launching → Running` transition and the atomicity window.
- **KillSession PC-3**: updated to state `SessionStateChanged { session_id, new_state }` BEFORE
  `SessionListUpdate`; notes the `Running → Terminating` immediate transition; references
  BC-2.08.003 for the subsequent `Terminating → Terminated` transition.
- **DetachSession PC-3**: updated to state `SessionStateChanged { session_id, new_state: Detached }`
  BEFORE `SessionListUpdate`.
- AttachSession PC-3 already specified the ordered pair (added in v1.2.0); no change needed.
- RenameSession PC-3 correctly omits `SessionStateChanged` per BC-2.08.008 PC-4a (rename is NOT
  a `SessionState` transition); no change needed.
- BC-2.08.008 PC-3 cross-reference verified against BC-2.08.008 before citation.

## §Trace v1.4.0

**S-P7-003 — Add AttachSession to H1 title for H1↔body consistency** (2026-06-03):
- H1 title listed 6 variants but Description, Invariant 1, and body all specify seven variants
  (AttachSession was added in v1.2.0 per I3-004 but the H1 was not updated at that time).
- H1 title corrected to enumerate all 7 variants: appended "AttachSession" to the comma-separated
  variant list. BC-INDEX row updated to match.
- No content change; cosmetic H1 consistency fix only.

## §Trace v1.3.0

**Pass-6 C6-001 — Align error codes with SS-ipc.md v1.14.0 architect taxonomy** (2026-06-03):
- C6-001: Architect is adding `ServerToClient::Error { code: String, message: String }` to SS-ipc.md
  (v1.14.0) with canonical code strings: `spawn_failed`, `session_not_found`, `attach_failed`,
  `kill_failed`, `rename_failed`, `invalid_request`.
- **KeyInput-PC-4:** Added explicit `code: "session_not_found"` — previously said only
  `ServerToClient::Error` with no code string. `session_not_found` is the correct code for
  unknown or dead session (matches architect taxonomy + SessionError::SessionNotFound).
- **EC-280:** Added `code: "spawn_failed"` to `ServerToClient::Error` — previously omitted code string.
- **EC-281:** Added `code: "session_not_found"` to `ServerToClient::Error` — previously omitted code string.
- **EC-283:** Added `code: "rename_failed"` to `ServerToClient::Error` — previously omitted code string.
  Per SS-session-manager.md §`session_error_to_code()` and mapping table (line ~433): `SessionError::InvalidSessionName { .. }` maps **unconditionally** to `"rename_failed"` — there is no op-aware or content-aware branch that routes to `"invalid_request"`. Per SS-ipc.md line ~405: `"invalid_request"` is reserved for "Validation failure BEFORE the SessionManager call"; an empty-name failure surfaces as `SessionError::InvalidSessionName` returned BY `rename_session()`, making it an operational (post-call) result, not a pre-call validation failure. Therefore `"rename_failed"` is the correct and only possible code for this edge case. (Note: the original v1.3.0 rationale incorrectly stated `"invalid_request"` was the correct code based on a validation/operational distinction that is NOT implemented in the canonical exhaustive `session_error_to_code()` function — corrected in v1.5.1 per I15-001.)
- **Architecture Source:** Added SS-ipc.md v1.14.0+ citation for the Error variant and code taxonomy.
  (v1.13.0 does NOT define `ServerToClient::Error`; the architect adds it in Pass-6 in parallel.)

## §Trace v1.2.0

**Adversarial Pass 3 fix — I3-004 (AttachSession 7th variant per SS-ipc.md v1.13.0)** (2026-06-03):
- I3-004: `ClientToServer::AttachSession { session_id }` added as the 7th variant. Daemon
  routes to `SessionManager::attach_session()`. Used for: (a) TUI re-attach after PtyReset
  (replaces the incorrect BC-2.05.011 PC-3c reference to "TUI sends DaemonToHost::Attach"
  — the TUI cannot send DaemonToHost messages), (b) explicit user-initiated re-attach of a
  Detached session from the sessions panel. Per SS-ipc.md v1.13.0 §ClientToServer::AttachSession.
- Invariant 1: updated "six" → "seven" variants.
- Description: updated to seven variants; clarified TUI must NOT send DaemonToHost::Attach.
- Architecture Source updated to SS-daemon-wiring-v2-delta.md v1.3.1 and SS-ipc.md v1.13.0.

## §Trace v1.1.0

**S2-004 adversarial pass-2 fix — zero-dimension clamp at daemon boundary** (2026-06-03):
- S2-004 finding: EC-282 said "if PTY rejects zero dimensions, `SessionError` returned" — this
  was inconsistent with BC-2.09.006 which has no zero-dimension handling and simply forwards
  the resize. Two BCs with different behaviors for the same condition is production-grade non-
  conformant. Resolution: clamp-to-1 at the daemon boundary, no error returned.
- EC-282: rewritten with clamp-to-1 rule at daemon IPC handler. No SessionError. Clamped values
  forwarded to resize_session(). Rationale: zero-dimension PTY is POSIX-undefined; clamping
  is more robust than rejecting (avoids unnecessary error handling in the TUI while preventing
  undefined behavior in the PTY).
- Invariant 5 added: zero-dimension clamp rule as a production-grade enforcement point at the
  daemon boundary. Cross-referenced with BC-2.09.006 EC-237 (TUI-side no-op detection).

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.05.010 authored for SS-05 as part of the v1A control-center pivot BC burst.
- Covers all 6 new ClientToServer variants from SS-daemon-wiring-v2-delta.md.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).

## §Trace v1.9.4

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.9.4 timestamp >= v1.9.3. PASS.
