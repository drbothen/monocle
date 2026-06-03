---
document_type: gene-source-disposition
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
gene_source: any-context-lazyclaude
disposition_pass: v2 (D-236 control-center pivot)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md §5
---

# Gene-Source Disposition v2: any-context/lazyclaude (Control-Center Lens)

## Vision Lens Applied

monocle v1 (re-baselined): full TUI control center — launch + manage + observe + tune + control.
The focus areas this repo touches: hook protocol (observe/launch), session lifecycle (launch/manage),
PM/Worker orchestration (explicitly out of scope), and the broker/IPC pattern (already adopted).

## Original Disposition Summary

The v1.1.2 vision synthesis treated any-context/lazyclaude as the primary source for:
- Hook protocol specification (5 endpoints, auth header, restart-resilience sequence) — ADOPT
- Broker pattern (non-blocking pub/sub, drop counter) — ADOPT
- Session lifecycle (Manager, Store, GC, worktree session) — ADOPT
- PM/Worker layer: LEAVE BEHIND (explicitly excluded by user direction)

The pivot's primary reversal target was "execute workflows — rejected — observe-only". The
any-context gene is about hook ingestion and session management, not workflow execution per se.
The PM/Worker exclusion stands.

## Disposition by Capability Area

### 1. Hook Protocol (5 endpoints, inject-on-spawn) (originally ADOPT)

**Original verdict: ADOPT** — hook protocol was the primary gene that shaped monocle Phase 1.
This is fully built (monocle-proto, hooks-settings.json generation, DTU clone).

**New verdict: ADOPT + EXTEND (inject-on-spawn is now a v1 requirement).**

The critical addition from the pivot: NEXT-SESSION-PIVOT.md §4.6 requires hook auto-injection
when monocle LAUNCHES a session. Currently the user manually copies hooks-settings.json into
`~/.claude/settings.json`. When monocle spawns the session, it must inject the hook config
automatically.

From any-context's ingest (BC-HOOK-027): "~/.claude/settings.json is NEVER written by
lazyclaude — hooks are injected via `claude --settings <path>`."

This is the precise injection mechanism for monocle:
```
portable-pty CommandBuilder:
  .program("claude")
  .args(["--settings", &hooks_settings_json_path])
  .cwd(&worktree_path)
  .envs(&session_env)
```

The `hooks-settings.json` file already exists (written by monocle daemon at startup). The
`--settings <path>` argument injects the hook config per-session without touching the user's
global `~/.claude/settings.json`. This is exactly the any-context pattern and it is
production-grade.

Hook protocol details confirmed ADOPT verbatim:
- 5 endpoints: PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit.
- PostToolUse: intentionally absent (BC-HOOK-007).
- Auth: `X-Claude-Code-Ide-Authorization: <token>`.
- Restart-resilience: PID-liveness lock-file discovery at every hook invocation.
- Notification filter: fires only on `permission_prompt`.
- `~/.claude/settings.json` is never written by monocle (BC-HOOK-027 monocle mirror).
- Hooks-settings.json per-runtimeDir (not per-session) — all sessions in same TUI share one file.
- `SetEscapeHTML(false)` + `Indent("", "  ")` encoding discipline.
- `claude --settings <path>` injection (not global settings mutation).
- Make hooks-settings.json path configurable via `MONOCLE_IDE_DIR` env var (BC-HOOK-014 fix
  — any-context hardcodes `~/.claude/ide/`; monocle should not hardcode).

### 2. Broker Pattern (non-blocking pub/sub with drop counter) (originally ADOPT)

**Original verdict: ADOPT** — broker is built (monocle-runtime, monocle-ipc).

**New verdict: ADOPT (confirmed). Control-center adds PTY output stream but same broker.**

In the control-center model, the broker's throughput requirements increase because PTY byte
streams are a high-frequency additional event type. The existing bounded-channel discipline
(from CLAUDE.md conventions) and the drop-counter pattern remain correct. The PTY output
channel is bounded separately per-session to avoid starving hook events.

Specific additions from any-context's broker lessons:
- Per-subscriber drop counter (`Subscription::dropped_count() -> u64`) — already planned.
- GUI subscriber buffer ≥16 (not 8 as in any-context's original). PTY streams warrant a
  separate, larger buffer: PTY-output channel buffer = 256 bytes per frame × 30fps = needs
  dedicated sizing. Recommend a separate `tokio::sync::mpsc::channel(1024)` per active session
  for PTY bytes, distinct from the event broker.
- `WithBroker` / `ownsBroker` server-restart survival: already adopted.
- Broker `Close()` only from owner: already adopted.

### 3. Session Lifecycle — Manager, Store, GC, Worktree Session (originally ADOPT)

**Original verdict: ADOPT** — session lifecycle already shapes the existing Phase-1 design.

**New verdict: ADOPT + EXTEND — the daemon gains SPAWN capability as a new lifecycle action.**

The existing session lifecycle (detect via EngineModule.detect, enrich via EngineModule.enrich,
observe via hook events) remains correct. The control-center adds:

- `Session::launch(project, opts) -> SessionId` — spawn a new PTY + child process.
- `Session::kill(session_id)` — drop the PTY master; child receives SIGHUP.
- `Session::attach(session_id) -> PtyStream` — return a stream of PTY bytes to the TUI client.
- `Session::detach(session_id)` — close the TUI's subscription without killing the session.

These are new methods on the session manager (a new daemon component), not on EngineModule
(the trait remains detect/enrich/on_hook as in the original design). The EngineModule provides
the SPAWN RECIPE (binary, args, env) via a new method; the session manager handles the actual
PTY ownership.

The GC pattern (Dead sessions cleaned up after grace period; Orphaned sessions preserved) from
any-context remains directly applicable. monocle's session manager should:
- GC: sessions that are Terminated AND have no pending TUI subscriber after 10 seconds.
- Orphaned: sessions whose daemon knows about (from persistent session-state.json) but whose
  PID is gone — show as Terminated in the list, offer re-launch.

### 4. SSH Reverse Tunnel / Remote Host Support (originally MODEL for Phase 4)

**Original verdict: MODEL (Phase 4)** — remote federation was Phase 4 scope.

**New verdict: LEAVE BEHIND for re-baselined v1.** Phase 4 is suspended (D-236 pivot). The
remote host tunneling capability from any-context (`ssh -L` tunnel, CompositeProvider,
MirrorManager) is deferred until after the control-center v1 capability set ships. No change
in disposition; scope is clarified.

### 5. PM/Worker Persona (originally LEAVE BEHIND)

**Original verdict: LEAVE BEHIND** — excluded by user direction.

**CONFIRMED LEAVE BEHIND.** The pivot adds launch/manage capability but explicitly does NOT
add PM/Worker automated orchestration. The human remains the coordinator.

### 6. `/msg/*` Bus Primitive (originally partially RETAIN)

**Original verdict:** The bus primitive (inter-session messaging endpoints) was flagged for
potential retention as "BC-Bus.*" BCs with safety fixes. In the observe-only model this was
deferred because monocle didn't spawn sessions.

**New verdict: LEAVE BEHIND for re-baselined v1.** The inter-session messaging bus from
any-context is a PM/Worker coordination mechanism. monocle v1 does not have inter-session
coordination; sessions are independent. This may be revisited in a later phase if monocle
gains agent-coordination features. Not needed for launch/manage/observe/tune/control v1.

### 7. MCPServer vs MCPRegistry Distinction (originally MODEL)

**Original verdict: MODEL** — the two distinct subsystems must be spec'd separately.

**New verdict: MODEL (confirmed).** monocle already has its equivalent distinction: the
daemon's hook HTTP server (`monocle-runtime` axum server) vs the configuration/customization
reader (`monocle-static`, not yet built). This distinction is architecturally clean and confirmed.

### 8. base64-wrap-then-eval for SSH Commands (originally ADOPT)

**Original verdict: ADOPT** (for Phase 4 remote federation).

**New verdict: Deferred with Phase 4.** Not needed for re-baselined v1.

### 9. Hook JS One-Liner / App Filter (BC-HOOK-024)

**Original verdict:** Flagged as P2 — hook JS does not filter by `lock.app`.

**New verdict: ADOPT fix in v1.** When monocle spawns sessions, the hooks-settings.json
it generates should include `lock.app` filtering to avoid cross-IDE port collisions:
`if (best.lock.app && best.lock.app !== 'monocle') continue;` in the hook JS template.
This is a one-line fix with material security benefit (prevents another IDE's hook from
accidentally routing to monocle's daemon).

## Summary Table

| Capability | Original Verdict | New Verdict | Change? |
|-----------|-----------------|-------------|---------|
| Hook protocol (5 endpoints, PID-liveness) | ADOPT (built) | ADOPT + EXTEND (`--settings` inject-on-spawn) | Extended |
| Broker pattern | ADOPT (built) | ADOPT (PTY bytes get separate high-throughput channel) | Extended |
| Session lifecycle (Manager/Store/GC) | ADOPT | ADOPT + EXTEND (spawn/kill/attach/detach) | Extended |
| SSH reverse tunnel / remote federation | MODEL (Phase 4) | LEAVE BEHIND (v1 suspended) | Scoped out |
| PM/Worker persona | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| /msg/* bus primitive | Partially RETAIN | LEAVE BEHIND (v1) | Scoped out |
| MCPServer vs MCPRegistry | MODEL | MODEL (confirmed) | Confirmed |
| base64-wrap-then-eval | ADOPT (Phase 4) | Deferred | Scoped out |
| Hook JS app filter (BC-HOOK-024) | P2 | ADOPT fix in v1 | Upgraded |

## Net Assessment

any-context/lazyclaude remains the definitive hook protocol specification source. The control-
center pivot makes the `--settings <path>` inject-on-spawn pattern a v1 requirement rather than
an optional enhancement. All other previously-adopted genes remain adopted; the PM/Worker and
bus-primitive genes remain excluded.

The upgrade from P2 to v1-required for the hook JS `lock.app` filter is the only new
production-grade decision this disposition introduces beyond confirming prior adoption.
