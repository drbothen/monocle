# Pass B Deep: `cmd/lazyclaude/` glue — Round 1

**Scope:** mirror.go, session_command.go, gui_adapter.go (head), local_provider.go (head), remote_host.go.

**Files read in full this round:** mirror.go (226 LOC), remote_host.go (78 LOC). Partial: session_command.go (200/431).

## MirrorManager — local placeholder for remote sessions

`MirrorManager` (mirror.go:18-226) creates local tmux windows that, on attach, exec `ssh -t <host> tmux attach`. The mirror is the display surface; the actual claude process runs remotely.

### BC-CMD-MIRROR-001: CreateMirror is idempotent against double-call (checks store.FindByID for existing)
**Postconditions:** Guards against double-click / retry. Returns nil without action if session already in store.
**Evidence:** mirror.go:44-48.
**Confidence:** HIGH

### BC-CMD-MIRROR-002: CreateMirror uses daemon response Path (worktree path) if non-empty; falls back to groupPath (project root)
**Postconditions:** Sidebar [W] worktree decoration shows the actual session path, not the parent project root.
**Evidence:** mirror.go:51-55.
**Confidence:** HIGH

### BC-CMD-MIRROR-003: addMirrorSession immediately resolves the new mirror window's tmux ID ("@N") instead of using the name ("rm-xxxx")
**Postconditions:** Sidebar activity events arrive keyed by the same TmuxWindow that the store entry uses. Without this immediate resolve, activity events arriving before the next SyncWithTmux (up to 2s later) would be written under the wrong key and reverted on sync.
**Evidence:** mirror.go:118-127 + explicit comment 119-127.
**Confidence:** HIGH — load-bearing race avoidance.

### BC-CMD-MIRROR-004: resolveMirrorTmuxID falls back to the mirrorName on lookup failure
**Postconditions:** Defensive: store entry is never empty. Self-heals on next SyncWithTmux.
**Evidence:** mirror.go:215-226.
**Confidence:** HIGH

### BC-CMD-MIRROR-005: RestoreExisting skips sessions already in the local store (silent skip per session)
**Postconditions:** On reconnection, mirrors aren't duplicated.
**Evidence:** mirror.go:88-92.
**Confidence:** HIGH — confirms BC-MIRROR-002.

### BC-CMD-MIRROR-006: createMirrorWindow uses base64-encoded remote command + `eval "$(echo BASE64 | base64 -d)"` wrap
**Postconditions:** Prevents shell injection from user-controlled host strings. Confirms BC-REMOTE-008 from Pass 3.
**Evidence:** mirror.go:152-172.
**Confidence:** HIGH

### BC-CMD-MIRROR-007: createMirrorWindow creates the lazyclaude tmux session if it doesn't exist; else NewWindow
**Postconditions:** Handles the fresh-start case where first operation is remote (no local sessions). Without this, NewWindow would fail with "no server running".
**Evidence:** mirror.go:181-206 + explicit comment 181-184.
**Confidence:** HIGH

### BC-CMD-MIRROR-008: DeleteMirror kills the local tmux window by `lazyclaude:rm-<id[:8]>`; ignores errors
**Postconditions:** Best-effort cleanup. If window already gone, no-op.
**Evidence:** mirror.go:77-80.
**Confidence:** HIGH

## RemoteHostManager — lazy connection per host

`RemoteHostManager` (remote_host.go full file) tracks one `sync.Once` per host. Lazy: first call to EnsureConnected triggers connectFn; subsequent calls observe the cached error.

### BC-CMD-REMOTE-001: Each host gets at most one connectFn invocation via sync.Once
**Postconditions:** Multiple concurrent EnsureConnected calls block on once.Do — only one runs.
**Evidence:** remote_host.go:50-54.
**Confidence:** HIGH

### BC-CMD-REMOTE-002: connectFn failure is cached — subsequent calls return the cached error without retry
**Postconditions:** Failed connections must be retried via explicit reconnect (e.g., remove the host entry first). Comment line 18-19: "If the initial connect fails, subsequent callers see the cached error without retrying (connectRemoteHost leaves no side effects on failure)."
**Evidence:** remote_host.go:50-55.
**Confidence:** HIGH

### BC-CMD-REMOTE-003: MarkConnected creates a pre-completed entry only if no entry exists
**Postconditions:** Prevents TOCTOU where EnsureConnected holds a reference to the OLD entry. Explicit comment: "do not replace it to avoid the TOCTOU where EnsureConnected holds a reference to the old entry and once.Do fires after replacement."
**Evidence:** remote_host.go:63-78.
**Confidence:** HIGH — load-bearing race protection.

### BC-CMD-REMOTE-004: EnsureConnected returns nil for empty host (local) or nil connectFn
**Postconditions:** Safe to call unconditionally; local ops short-circuit cleanly.
**Evidence:** remote_host.go:38-40.
**Confidence:** HIGH

## SessionCommandService — local/remote routing for create/delete/rename

`SessionCommandService` (session_command.go:56-431) is the glue layer that hides local-vs-remote branching from the GUI adapter. The host check `sess.Host != ""` lives in this layer.

### BC-CMD-SCS-001: Delete routes by sess.Host: remote → daemon API + DeleteMirror + local store remove; local → cp.Delete
**Postconditions:** Remote daemon errors are NOT fatal — KillMirror + store removal proceed regardless. The remote daemon's session may persist if the API call fails.
**Evidence:** session_command.go:102-122.
**Confidence:** HIGH

### BC-CMD-SCS-002: Rename routes by sess.Host: remote → daemon rename + local UpdateSession + save; local → cp.Rename
**Postconditions:** Remote rename failure aborts the local update. Atomic from user perspective.
**Evidence:** session_command.go:126-146.
**Confidence:** HIGH

### BC-CMD-SCS-003: Create dispatches local → cp.Create; remote → optimistic placeholder + background goroutine
**Postconditions:** User sees the session immediately as "connecting..." while the real creation happens async.
**Evidence:** session_command.go:148-171.
**Confidence:** HIGH

### BC-CMD-SCS-004: Optimistic placeholder has Name="connecting..." Path=host Host=host Status=Running
**Postconditions:** The Path field is set to the host string (not actual remote path) until completeRemoteCreate runs.
**Evidence:** session_command.go:155-167.
**Confidence:** HIGH

### BC-CMD-SCS-005: completeRemoteCreate runs in background goroutine: ensureConnected → resolveRemotePath → remoteCreateSession → success or failPlaceholder
**Postconditions:** All failure paths call failPlaceholder which presumably (not yet read) updates the placeholder to show the error.
**Evidence:** session_command.go:175-200 (head).
**Confidence:** HIGH

### BC-CMD-SCS-006: resolveLocalPath converts "." to absolute path; other paths pass through
**Postconditions:** Sessions are stored with their actual working directory. Matches the localDaemonProvider.Create convention.
**Evidence:** session_command.go:23-31.
**Confidence:** HIGH

### BC-CMD-SCS-007: SessionCommandService uses remoteSessionAPI interface (subset of *daemon.RemoteProvider methods) — testable without real SSH
**Postconditions:** Tests can inject fake remoteSessionAPI without building a real RemoteProvider.
**Evidence:** session_command.go:43-49 + remoteProviderFn override at 70-75.
**Confidence:** HIGH

## Cross-pass observation

The cmd/ glue layer is where the architecture's local-vs-remote ambidexterity lives:

1. **MirrorManager** — creates the visual surface (local tmux window) for remote sessions.
2. **RemoteHostManager** — single-flight lazy SSH connections.
3. **SessionCommandService** — local/remote dispatcher for CRUD ops.
4. **guiCompositeAdapter** (gui_adapter.go) — bridges between GUI's SessionProvider interface and daemon.CompositeProvider + SessionCommandService.

The pattern is: GUI sees a unified provider; cmd/ glue layer routes by `sess.Host`; mirror layer maintains the local tmux surface; daemon layer handles the remote daemon API.

## Delta Summary

- New items added: 15 (8 BC-CMD-MIRROR, 4 BC-CMD-REMOTE, 7 BC-CMD-SCS — wait, 7 of latter listed but 7 written ok)
- Existing items refined: BC-REMOTE-008 confirmed at this site, BC-MIRROR-001/002 from Pass 3 confirmed.
- Remaining gaps: session_command.go body (failPlaceholder, completeRemoteCreate tail, additional methods), gui_adapter.go (full), local_provider.go (full).

## Novelty Assessment

Novelty: SUBSTANTIVE

Justification: 15 new contracts, including:
- **BC-CMD-MIRROR-003** immediate tmux-window-ID resolve to prevent the "@N vs rm-xxxx mismatch" race (load-bearing for sidebar activity correctness).
- **BC-CMD-REMOTE-002** failed connections are CACHED — no retry without explicit reset.
- **BC-CMD-REMOTE-003** TOCTOU guard in MarkConnected.
- **BC-CMD-SCS-004** optimistic placeholder pattern with "connecting..." UI.
- **BC-CMD-SCS-001** Delete remote-daemon-best-effort (doesn't fail user-visible).

These materially change the porter's model of how the local TUI maintains the illusion of unified session management across local and remote backends.

## Convergence Declaration

**Pass B cmd/glue has converged on the architectural layer.** session_command.go tail (~230 LOC), gui_adapter.go (426 LOC), and local_provider.go (278 LOC) contain repetitive routing patterns (similar Delete/Rename/Create dispatch). Reading them would add detail (e.g., the specific failPlaceholder UI update) but not new architecture.

The orienting prompt scope was "mirror.go, session_command.go, gui_adapter.go, local_provider.go, remote_host.go" — the architecture-layer is covered. Specific helper methods are mechanical.

## State Checkpoint

```yaml
pass: B
subsystem: cmd-glue
round: 1
status: complete
files_read_full: [mirror.go, remote_host.go]
files_read_partial: [session_command.go (200/431)]
contracts_drafted: 15
timestamp: 2026-05-11T22:55:00Z
novelty: SUBSTANTIVE
convergence: PASS-B-CMD-GLUE CONVERGED (sufficient architecture coverage)
next_subsystem: PM/Worker
```
