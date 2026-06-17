---
document_type: epic
epic_id: EPIC-05
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
subsystems: [SS-05]
capabilities: [CAP-005]
behavioral_contracts: [BC-2.05.001, BC-2.05.002, BC-2.05.003, BC-2.05.004, BC-2.05.005, BC-2.05.006, BC-2.05.007, BC-2.05.008, BC-2.05.009, BC-2.05.010, BC-2.05.011]
verification_properties: []
---

# EPIC-05: IPC

## Purpose

Implement the Unix Domain Socket (UDS) IPC transport between the monocle daemon and TUI
clients. This epic covers: the UDS server bind, core message types (SessionListUpdate,
HookEventReceived, PermissionPromptQueued), initial state push on connect, TUI reconnect
after daemon restart with SOQ-3 overlay clear, the PTY fan-out broker, all 7 new v1A
`ClientToServer` lifecycle variants (SpawnSession, KillSession, KeyInput, ResizePane,
DetachSession, RenameSession, AttachSession), and the new `ServerToClient` scrollback
protocol variants (ScrollbackChunk, ScrollbackDumpComplete, PtyReset). This epic delivers
the complete SS-05 IPC subsystem across Waves 5-8.

## Success Criteria

- All 11 BC-2.05.NNN behavioral contracts pass their verification properties
- UDS server binds at `runtimeDir/monocle.sock` with 0o700 permissions
- TUI client connects and receives initial `SessionListUpdate` + `HookEventReceived` state push
- Reconnect after daemon restart clears the permission overlay stack (SOQ-3)
- UDS-only transport for Phase 1 (no shared-memory path)
- PTY fan-out broker: bounded 1024-item channel, per-client mpsc::Sender<ServerToClient>(64), 3-strike disconnect, PtyReset on PTY writer drop
- All 7 new ClientToServer variants routed correctly with no-silent-failure invariant
- Scrollback dump protocol: contiguous ScrollbackChunk sequence + ScrollbackDumpComplete total_chunks validation
- `cargo clippy --workspace --all-targets -- -D warnings` → 0 warnings

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-021 | UDS Server Bind + IPC Transport + Core Message Types | 8 | Wave 5 | S-017, S-014, S-013 |
| S-022 | TUI Client Connect, Initial State Push, and Permission Message Types | 8 | Wave 6 | S-021, S-018 |
| S-023 | TUI Reconnect After Daemon Restart + SOQ-3 Overlay Clear | 5 | Wave 6 | S-022, S-019 |
| S-032 | Daemon Event-Bus Fan-Out: Broadcast HookEventReceived with daemon timestamp_micros | 5 | Wave 8 | S-021, S-022, S-028 |
| S-046 | PtyOutput Fan-out Broker — Bounded Channel, Backpressure, and Client Lifecycle | 5 | Wave 8 | S-021, S-032 |
| S-047 | IPC Lifecycle Variants — Spawn/Kill/Detach/Attach/Rename/Input/Resize + Scrollback Protocol | 8 | Wave 8 | S-021, S-022, S-023, S-033, S-034, S-035, S-046 |

**Total: 39 points (21 pts Waves 5-6 + 18 pts Wave 8 v1A delta)**

## Architecture Scope

- Implementing modules: `monocle-runtime` (UDS server, broker, IPC handler), `monocle-ipc` (wire types)
- Architecture source: `architecture/SS-ipc.md` v1.24.0
- Architecture dependency: `architecture/SS-session-manager.md` v2.6.1 (session lifecycle routing in S-047)
