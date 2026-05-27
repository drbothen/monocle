---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T04:00:00Z
phase: phase-1-expansion
inputs: [prd-expansion-scope.md, architecture/SS-ipc.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-05
capability: CAP-005
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

# Behavioral Contract BC-2.05.005: IPC Message Types: PermissionPromptQueued

## Description

When a `PreToolUse` hook POST arrives with `decision_required: true`, the daemon creates a
pending-decision entry and broadcasts a `ServerToClient::PermissionPromptQueued` message to
all connected TUI clients. The message contains the full payload needed for the TUI to render
the permission overlay, including diff content for file-mutation tools. The daemon holds the
HTTP response open (via a `oneshot::channel`) until a TUI client sends a `ClientToServer::PermissionDecision`
or the hook timeout expires. When one TUI client resolves a prompt, the daemon broadcasts
`ServerToClient::PermissionPromptResolved` to all other clients so they can remove the stale
overlay entry.

## Preconditions

1. The daemon has received a `PreToolUse` hook POST with `decision_required: true` in the
   request body (BC-2.04.007 — PreToolUse request routing).
2. The daemon has created a pending-decision entry in its registry:
   - A unique `prompt_id: Uuid` is generated for this prompt.
   - A `oneshot::Sender<PermissionDecision>` is stored in the registry, keyed by `prompt_id`.
   - The HTTP response future is suspended, awaiting resolution of the `oneshot`.
3. At least one TUI client is connected (otherwise the message cannot be delivered; the
   daemon holds the HTTP response open until timeout regardless).

## Postconditions

1. The daemon serializes and sends `ServerToClient::PermissionPromptQueued` to all currently
   connected TUI clients. The message fields are:
   - `prompt_id: Uuid` — stable identifier for this prompt, generated at creation time.
   - `session_id: String` — the session identifier from the hook POST body.
   - `tool_name: String` — the name of the tool requesting permission (e.g., `"Edit"`, `"Bash"`).
   - `tool_input: serde_json::Value` — the full tool input object from the hook POST body.
   - `old_content: Option<String>` — present when `tool_name` is `"Edit"` (or another
     file-mutation tool); contains the file content before the proposed edit.
   - `new_content: Option<String>` — present when `old_content` is present; contains the
     proposed new file content. The TUI uses these two fields to compute a unified diff via
     `similar::TextDiff` (BC-2.06.010).
2. The `prompt_id` is stable for the lifetime of the pending decision. It does not change
   between `PermissionPromptQueued` and the corresponding `PermissionPromptResolved`.
3. When a TUI client sends `ClientToServer::PermissionDecision { prompt_id, decision }`:
   - The daemon looks up `prompt_id` in its pending-decision registry.
   - If found: resolves the `oneshot::Sender` with the decision; sends the HTTP response to
     Claude Code; removes the entry from the registry; broadcasts
     `ServerToClient::PermissionPromptResolved { prompt_id }` to ALL other connected TUI
     clients (not to the resolving client, though broadcasting to all clients including the
     resolver is also acceptable).
   - If NOT found (second resolution attempt, race condition): the `PermissionDecision` message
     is silently discarded. No error is returned to the TUI client.
4. When the hook timeout expires (300ms for PreToolUse per BC-2.04.007 and BC-HOOK-022) before
   any TUI client resolves the prompt:
   - The daemon resolves the pending hook response with the fail-open or fail-closed semantics
     per BC-HOOK-001 / BC-HOOK-002 (no decision was made; Claude Code's default applies).
   - The daemon does NOT send `PermissionPromptResolved` to TUI clients. The stale overlay
     entry remains visible until the TUI next connects (or until a periodic cleanup sweep).
   - The `prompt_id` entry is removed from the daemon registry after timeout resolution.

## Invariants

1. Every `PermissionPromptQueued` has a unique `prompt_id`. No two pending prompts share the
   same `prompt_id` in the daemon's registry at the same time. (UUIDs are unique with
   overwhelming probability; no deduplication check is required in practice.)
2. The `oneshot::channel` per prompt enforces at-most-one resolution: the first
   `PermissionDecision` to arrive resolves the channel; all subsequent decisions for the same
   `prompt_id` are silently discarded. This prevents double-approval races when multiple TUI
   clients view the same prompt.
3. The daemon never sends `PermissionPromptResolved` without a corresponding prior
   `PermissionPromptQueued` for the same `prompt_id` (within the lifetime of a single
   daemon process). TUI clients can safely use `prompt_id` as an exact key into their
   local overlay stack.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two TUI clients both press Accept simultaneously for the same `prompt_id` | First `PermissionDecision` resolves the oneshot and removes the registry entry. Second `PermissionDecision` finds no entry and is silently discarded. `PermissionPromptResolved` is broadcast after the first resolution; the second client's overlay entry is already gone by the time its own decision arrives at the daemon. |
| EC-002 | PreToolUse hook times out (300ms) before any TUI client responds | Daemon resolves the pending hook response using Claude Code's default fallback (fail-open per BC-HOOK-001). No `PermissionPromptResolved` sent. The TUI overlay entry for this `prompt_id` becomes stale. On next TUI reconnect, the `InitialState` will not include this prompt (it was removed from the registry). |
| EC-003 | No TUI clients are connected when `PermissionPromptQueued` would be sent | Message is not sent (empty subscriber list). The daemon holds the HTTP response open waiting for a decision that can only arrive when a TUI client connects and sends `PermissionDecision`. On TUI connect, `InitialState.overlay_stack` contains this prompt. |
| EC-004 | `tool_input` is a very large JSON object (e.g., 200 KiB) | If the `PermissionPromptQueued` message exceeds 256 KiB, the daemon cannot send it (IpcError::MessageTooLarge). The daemon logs an error and does NOT send the message; the prompt remains in the registry and is visible in `InitialState.overlay_stack` for connecting clients. The 256 KiB limit acts as a practical guard against pathological tool inputs. |
| EC-005 | TUI client disconnects while viewing a `PermissionPromptQueued` overlay | SOQ-3 (BC-2.05.007) clears the TUI's overlay stack on disconnect. On reconnect, `InitialState.overlay_stack` re-delivers any still-pending prompts. The `prompt_id` is stable across reconnects. |
| EC-006 | TUI sends `PermissionDecision` with a `prompt_id` that has already timed out | No entry in registry; `PermissionDecision` silently discarded. Daemon does not return an error to the TUI. Claude Code has already processed the default fallback; the decision has no effect. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| PreToolUse hook arrives (`decision_required: true`, `tool_name: "Edit"`, with old_content + new_content); 1 TUI client connected | `PermissionPromptQueued { prompt_id: <uuid>, session_id: "abc", tool_name: "Edit", tool_input: {...}, old_content: Some("<old>"), new_content: Some("<new>") }` sent to client | happy-path |
| PreToolUse hook arrives (`tool_name: "Bash"`, no old/new content); 1 TUI client connected | `PermissionPromptQueued { ..., tool_name: "Bash", old_content: None, new_content: None }` sent | happy-path |
| Client sends `PermissionDecision { prompt_id: <uuid>, decision: Accept }` | Daemon resolves oneshot; sends HTTP response to Claude Code; broadcasts `PermissionPromptResolved { prompt_id }` to other clients | happy-path |
| Two clients both send `PermissionDecision` for same `prompt_id` simultaneously | First decision resolves; second silently discarded; `PermissionPromptResolved` broadcast once | edge-case |
| PreToolUse hook times out (300ms) with no decision | Daemon resolves fail-open/closed; no `PermissionPromptResolved` sent; registry entry removed | edge-case |
| `PermissionDecision` sent for unknown `prompt_id` | Silently discarded; no error response to TUI | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `PermissionPromptQueued` broadcast on `decision_required: true` PreToolUse arrival | integration |
| VP-TBD | `prompt_id` is stable across the prompt lifecycle (queued → resolved) | unit |
| VP-TBD | Second `PermissionDecision` for same `prompt_id` is silently discarded | integration |
| VP-TBD | `PermissionPromptResolved` broadcast to all clients except (optionally) resolver after first decision | integration |
| VP-TBD | Timeout path: registry entry removed; no `PermissionPromptResolved` sent | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability §SS-05 — this BC specifies the permission decision routing path, which is the highest-stakes behavioral contract in the IPC subsystem: it connects the hook caller's 300ms window to the user's keypress |
| L2 Domain Invariants | DI-001 (hook event written before ACK — this BC's PreToolUse POST is held open pending decision, not ACK'd immediately; DI-001 applies when the ACK is finally sent with the decision); DI-002 (lock file and auth token required before endpoints accept connections — Precondition 1 cites BC-2.04.007, which depends on the lock file being valid) |
| Architecture Module | monocle-ipc (ServerToClient::PermissionPromptQueued, ClientToServer::PermissionDecision, oneshot registry) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-ipc.md v1.0.0 §Message Types §Server-to-Client Messages; SS-ipc.md v1.0.0 §Connection Lifecycle §Phase 2 Streaming Updates; SS-ipc.md v1.0.0 §Risk Mitigations §Multiple TUI Clients Resolving the Same Prompt |
| Cross-Ref | BC-2.04.007 (PreToolUse hook routing — produces the event that triggers this BC); BC-HOOK-001 (fail-open semantics on timeout); BC-HOOK-002 (fail-closed semantics on timeout); BC-2.05.007 (SOQ-3 — overlay cleared on disconnect); BC-2.06.011..BC-2.06.013 (TUI keybindings that send PermissionDecision) |
| Test File | `monocle-ipc/tests/permission_prompt.rs` |
| Test Name | `test_BC_2_05_005_permission_prompt_queued_and_resolved` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.007] — depends on: PreToolUse routing produces the hook event this BC is triggered by
- [BC-2.05.007] — composes with: SOQ-3 clears overlay on disconnect; reconnect InitialState re-delivers pending prompts
- [BC-2.06.011] — composes with: TUI Accept-Once sends PermissionDecision that this BC processes
- [BC-2.06.012] — composes with: TUI Accept-Always sends PermissionDecision
- [BC-2.06.013] — composes with: TUI Reject sends PermissionDecision

## Architecture Anchors

- `architecture/SS-ipc.md#message-types` — `ServerToClient::PermissionPromptQueued` and `PermissionPromptResolved` variant definitions; `ClientToServer::PermissionDecision` definition; `PermissionDecision` enum (Accept, AcceptAlways, Reject)
- `architecture/SS-ipc.md#risk-mitigations` — §Multiple TUI Clients Resolving the Same Prompt (oneshot race mitigation)
- `architecture/SS-daemon-wiring.md#pretooluse-permission-decision-hold` — oneshot::channel per-prompt pending-decision registry

## Story Anchor

S-TBD — Implement PermissionPromptQueued IPC broadcast and PermissionDecision routing (filled by story-writer)

## VP Anchors

VP-TBD — Permission prompt queued, resolved, and dual-resolution race verification (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T04:00:00Z):
- BC-2.05.005 authored for SS-05 IPC subsystem per `prd-expansion-scope.md §3.2` and
  `SS-ipc.md §Message Types + §Connection Lifecycle §Phase 2 + §Risk Mitigations §Multiple TUI Clients`.
- Covers: PermissionPromptQueued message type, full payload (old_content/new_content for diff),
  prompt_id stability, PermissionDecision routing, PermissionPromptResolved broadcast,
  oneshot race mitigation (second resolution silently discarded), timeout behavior,
  no-client-connected case, SOQ-3 reconnect re-delivery via InitialState.overlay_stack.
- 6 edge cases documented (EC-001..EC-006).
- SE-16d PASS: 2026-05-26T04:00:00Z is the production timestamp for this wave.
