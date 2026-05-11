# Pass 4 — Verification Gaps & Test Coverage: any-context/lazyclaude

## Test Inventory (Recount)

Source: `find /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal -name '*_test.go' | wc -l`

**Total `_test.go` files in repo: 119**

Distribution under `internal/`:

| Package | Test files |
|---|---|
| `internal/gui/` | 27 |
| `internal/daemon/` | 14 |
| `internal/session/` | 12 |
| `internal/server/` | 10 |
| `internal/core/tmux/` | 4 |
| `internal/gui/presentation/` | 4 |
| `internal/mcp/` | 4 |
| `internal/gui/keyhandler/` | 4 |
| `internal/plugin/` | 3 |
| `internal/adapter/tmuxadapter/` | 2 |
| `internal/gui/keymap/` | 2 |
| `internal/core/config/` | 2 |
| `internal/core/shell/` | 1 |
| `internal/profile/` | 1 |
| `internal/notify/` | 1 |
| `internal/core/model/` | 1 |
| `internal/core/lifecycle/` | 1 |
| `internal/gui/keydispatch/` | 1 |
| `internal/core/event/` | 1 |
| `internal/gui/chooser/` | 1 |
| `internal/core/choice/` | 1 |

Plus 8 tests under `cmd/lazyclaude/`:
- `mirror_test.go`, `profile_test.go`, `resolve_remote_path_test.go`, `root_test.go`, `routing_integration_test.go`, `routing_test.go`, `session_adapter_test.go`, `sessions_test.go`

Plus 6 integration tests at top-level `tests/`:
- `tmux_test.go`, `ssh_test.go`, `popup_test.go`, `popup_sendkeys_test.go`, `diff_test.go`, `fullscreen_test.go`, `server_test.go` (+helpers + testdata)

**Test density (recount):** 119 internal test files vs 362 Go files total → 33% ratio. Excluding third_party/ test files (none), the ratio in production code is higher.

## E2E Test Inventory (VHS Tapes)

Source: `vis_e2e_tests/tapes/` and `vis_e2e_tests/TEST_CATALOG.md`

| Tape | Purpose | Requires |
|---|---|---|
| `smoke.tape` | Docker smoke test; pwd/ls/echo/cat | — |
| `hero.tape` | README hero GIF; create session + fullscreen + popup + approve | Claude OAuth token |
| `profile.tape` | Profile selection on local session create | (TBD — read file) |
| `ssh_profile.tape` | Remote profile selection via SSH | SSH host + Claude token |
| `ssh_notify.tape` | Remote notification (permission popup) | SSH host + Claude token |
| `ssh_msg.tape` | Inter-session messaging over SSH | SSH host + Claude token |
| `ssh_worktree_pm.tape` | Remote worktree + PM via SSH | SSH host + Claude token |
| `worktree_pm.tape` | Local worktree + PM session | Claude token |

**Note:** TEST_CATALOG.md is in Japanese; only "smoke" and "hero" entries are visible in the first read. The remaining tapes are inferred from filenames. **Gap:** Catalog should be read in full during deepening to confirm coverage for `ssh_msg`, `ssh_notify`, `ssh_worktree_pm`, `worktree_pm`, `profile`, `ssh_profile`.

## Test Coverage by Subsystem (Qualitative)

| Subsystem | Coverage | Notes |
|---|---|---|
| `core/event` | EXCELLENT | 16+ test cases incl. goroutine leak (`goleak.VerifyTestMain`), concurrency under race, edge cases (zero buffer, cancel idempotent, close idempotent, publish-after-close). |
| `core/lifecycle` | EXCELLENT | 8 cases incl. panic recovery, LIFO order, concurrent register under race, register-after-close, empty close, idempotent close. |
| `core/tmux` | GOOD | control_test.go, exec_test.go, mock_test.go, pidwalk_test.go. Mock client lets higher tests avoid spawning tmux. |
| `core/config` | GOOD | config_test.go + hooks_test.go (verifies hook JSON shape and SetEscapeHTML behaviour). |
| `internal/session` | GOOD | 12 test files. `manager_test.go` 882 LOC covers Create/Delete/Rename/Sync/PM/Worker. `store_test.go` 666 LOC. `gc_test.go`, `gitcmd_test.go`, `launchspec_test.go`, `project_store_test.go`, `project_test.go`, `resolve_prompt_test.go`, `role_test.go`, `worktree_test.go`. |
| `internal/server` | GOOD | server_test.go 727 LOC + server_broker_test.go 633 LOC + handler_test.go + handler_msg_test.go 672 LOC + state_test.go + lock_test.go + discover_test.go + ensure_test.go + jsonrpc_test.go + main_test.go (TestMain). |
| `internal/daemon` | GOOD | 14 files. server_test.go, server_sse_test.go, server_capture_test.go, http_client_test.go, http_client_capture_test.go, ssh_test.go, askpass_test.go, tunnel_test.go, mock_ssh_test.go, composite_provider_test.go (685 LOC), remote_provider_test.go (1115 LOC — heaviest), connection_impl_test.go, lifecycle_test.go, proc_cwd_linux_test.go. |
| `internal/gui` | EXTENSIVE | 27 test files. Includes integration tests (`app_integration_test.go`), broker tests (`app_broker_test.go`), `popup_controller_test.go`, `popup_test.go`, `popup_types_test.go` 611 LOC, `notify_loop_test.go`, `dialog_test.go`, `layout_test.go`, `render_test.go`, `render_export_test.go`, `render_internal_test.go`, `fullscreen_test.go`, `search_test.go`, `input_test.go`, `keybind_help_test.go`, `keybindings_test.go` (via `_test.go`), `tree_test.go`, `state_test.go`, `scroll_state_test.go`, `worktree_dialog_test.go`, `profile_dialog_test.go`, `logs_state_test.go`, `choice_test.go`, `plugin_remote_disabled_test.go` 1003 LOC, `mcp_state_test.go` (via mcp_state.go siblings — actually no sibling, see render_mcp_test.go missing), `config_dir_test.go`, `export_test.go`, `sshconfig_test.go`. |
| `internal/gui/keymap` | GOOD | doc_test.go, registry_test.go |
| `internal/gui/keydispatch` | GOOD | dispatcher_test.go |
| `internal/gui/keyhandler` | GOOD | handler_test.go, panel_test.go, plugins_test.go, mock_actions_test.go |
| `internal/gui/presentation` | GOOD | diff_test.go, logline_test.go, style_test.go, tool_test.go |
| `internal/gui/chooser` | OK | chooser_test.go only |
| `internal/mcp` | OK | 4 test files (specific names not all confirmed) |
| `internal/plugin` | OK | 3 test files |
| `internal/notify` | THIN | 1 test file |
| `internal/profile` | THIN | 1 test file — high impact module, low test count |

## Tests at `tests/` Top-Level (Cross-Cutting Integration)

Real-binary integration tests that build `bin/lazyclaude-test` once (via `ensureBinary`, helpers_test.go:19-41) and spawn it:
- `tmux_test.go` — exercises tmux session creation end-to-end
- `ssh_test.go` — exercises SSH connect/disconnect flow
- `popup_test.go`, `popup_sendkeys_test.go` — popup with real key sending
- `diff_test.go` — file diff capture for Write/Edit popup variants
- `fullscreen_test.go` — fullscreen mode entry/exit + key forwarding
- `server_test.go` — server discovery + hook protocol

These are **build-then-spawn** integration tests, distinct from `internal/<pkg>/*_test.go` unit tests. They presume tmux is available on the test runner.

## Test Strategy Patterns

| Pattern | Where | Notes |
|---|---|---|
| `goleak.VerifyTestMain(m)` | event_test.go:14, lifecycle_test.go:14 | Catches goroutine leaks at package exit. |
| `t.Parallel()` | Liberal across files | Most tests parallel-safe. |
| `t.TempDir()` | Anywhere needing fs | Ensures test isolation. |
| `t.Cleanup` | Per-test resource teardown (daemon_test.go:42, 53) | Replaces `defer` for test resources. |
| MockClient | `core/tmux/mock.go` | Pre-populated Sessions/Panes maps; reused across all higher tests. |
| `httptest.NewServer` | daemon_test.go:52 | HTTP tests don't spawn real binaries. |
| `_test` external packages | session_test, lifecycle_test, event_test, gui_test | Forces tests through the public API. |
| `export_test.go` files | gui, session | Standard Go idiom to expose private types. |
| Mock claude client | `cmd/mock-claude-client/main.go` | Spawned by some integration tests to simulate WS + hook traffic. Reads ~/.claude/ide/*.lock and connects. |

## Verification Gaps (Subsystems With Lower Confidence)

### Gap-VER-001: Profile loading (MEDIUM-LOW)
Only `profile_test.go` (1 file) for an entire subsystem that drives session launch. Critical paths:
- ProfileDef field validation (Name/Command required)
- ResolveDefault precedence (Default flag vs "default" name vs Builtin)
- Builtin profile constants (BuiltinDefault, BuiltinDefaultName)
- JSON unmarshal error messages (the daemon /profiles handler claims to surface "invalid JSON at line N, col M" — verify)

**Suggested verification in Pass B:** Read `internal/profile/profile.go` + profile_test.go to confirm.

### Gap-VER-002: PM/Worker prompt resolution (LOW)
`session/resolve_prompt_test.go` exists. The system has **3 layers** per commit log ("feat(session): add $HOME/.lazyclaude/prompts/ as layer 3 in resolvePrompt"):
1. Embedded `.lazyclaude/prompts/{pm,worker}.md` (project layer)
2. `prompts/` top-level repo dir (?)
3. `$HOME/.lazyclaude/prompts/` (user override)

**Verify in PMW pass:** read resolve_prompt.go + test, confirm 3 layers and precedence.

### Gap-VER-003: VHS tapes cover happy path; no failure-mode visual tests
No tapes for:
- Daemon crash mid-session
- SSH tunnel disconnect during active session
- Permission popup arrive while popup already shown (stacked popup behavior)
- Concurrent popup arrivals on different windows
- Askpass cancellation
- Profile config malformed
- Hook command timing out
- MCP server restart with persistent broker

These are all behaviorally important; lack of tape coverage means regressions are caught only by unit-level mocking.

### Gap-VER-004: `internal/notify` (file-based polling fallback) — single test
Only 1 test file. This is the **fallback path** when no broker subscriber exists (daemon mode without TUI, or TUI not yet started). If it silently drops or duplicates, the daemon-mode user loses popups. **Suggested:** verify FIFO ordering, file cleanup, concurrent enqueue safety.

### Gap-VER-005: tmuxadapter.DetectMaxOption — only 2 test files in package
DetectMaxOption is called inline by `dispatchToolNotification` (server.go:497) to pick between 2- and 3-option popups. **Failure mode:** wrong MaxOption → user sees a "1/2/3" popup when claude only offered 1/2, and presses 3 with no effect. Visual tests don't cover this. Unit tests presumably cover patterns but should be verified.

### Gap-VER-006: Daemon `/profiles` malformed-config error format
daemon/server.go:617-632 promises "invalid JSON at line N, col M: ..." but the actual error is `loadErr.Error()` — depends on `profile.Load`'s error wording. If `profile.Load` returns a Go default `*json.SyntaxError` string ("invalid character ... in object"), the daemon doc lies. **Verify in profile deep pass.**

### Gap-VER-007: SSE event ID monotonicity / replay
`sseEventID atomic.Uint64` increments on every event (daemon/server.go:55, 178). The SSE spec lets clients request replay-from-id via `Last-Event-ID` header. Lazyclaude's daemon does NOT honor this header (no inspection in server_sse.go). **Result:** reconnects always do a `full_sync` from scratch, never partial replay. This is intentional but should be made explicit in any documentation.

### Gap-VER-008: Mirror window lifecycle on remote SSH drop
When the SSH connection drops, the remote tunnel + SSE stream die. The mirror window remains in local tmux until cleanup. Tests for MirrorManager cleanup on disconnect:
- Does `lc.Register("remote-conn-"+host, func() { remoteProvider.StopSSE(); remoteConn.Disconnect() })` (root.go:225-228) also remove mirrors?
- Or do mirrors live until manual `D` (PurgeOrphans)?

Reading mirror.go in Pass B will confirm.

### Gap-VER-009: PM/Worker resume after session GC (PMW)
`POST /session/resume` (API v3) takes ID + `name` (worktree fallback). If a PM/Worker is GC'd, can it resume preserving its role? `session.ResumeSession` (manager.go:521-533, root.go:521-533) preserves Role via `string(sess.Role)`. Tests in manager_test.go should cover but **verify**: does the resumed session re-receive its PM/Worker system prompt? (probably no, since prompt is sent only once at launch). The resume contract is about *worktree state*, not *prompt re-injection*.

### Gap-VER-010: cmd/mock-claude-client behavior verification
cmd/mock-claude-client connects only — does it handle openDiff RPCs back from the server? Reading main.go full content would confirm whether it implements the IDE-side of the protocol or just the initiator. **Important for understanding the hook-protocol surface area.**

### Gap-VER-011: Permission popup MaxOption=2 vs 3 detection (already noted in Gap-VER-005)

### Gap-VER-012: Daemon shutdown semantics
`/shutdown` POST returns 200 then closes shutdownCh. daemon_cmd.go:108-119 selects on shutdownCh and calls Stop with 10s context timeout. Tests should verify:
- Concurrent /shutdown requests don't double-close (server.go:719-723 guards with mu)
- Tunnel + SSE goroutines exit cleanly
- daemon.json file is removed (verified: server.go:174-176, 769-775)

### Gap-VER-013: Token rotation
No tests for token rotation. The token is generated ONCE at server start (root.go:408, daemon_cmd.go:32-34). On restart, a NEW token is generated, all hooks must rediscover (lock file scan). **Behavior under restart-during-active-popup is untested.**

### Gap-VER-014: Bracketed-paste edge cases in fullscreen mode
README.md:245 explicitly notes paste in fullscreen "does not work reliably." VHS tapes can't easily exercise paste. Manual verification only.

### Gap-VER-015: gocui patches in third_party/gocui
The README of `.claude/CLAUDE.md` notes "third_party/gocui: fork of jesseduffield/gocui. Adds paste aggregation, rawEvents pipeline, etc." Patches in `LAZYCLAUDE_PATCHES.md` (third_party/tcell). **Gap:** No automated test validates that the patches still apply against upstream — this is a vendoring/maintenance gap.

## High-Risk Untested Behaviors (P0/P1)

| Risk | Severity | Rationale |
|---|---|---|
| Profile config malformed JSON path | P1 | Daemon promises specific error format; profile/Load may not produce it. Cascading: TUI silently uses builtin default while user thinks their config is honored. |
| Mirror window orphans on SSH drop | P1 | If mirrors persist, sidebar shows dead remotes; cleanup may need manual action. |
| SSE reconnect race when remote daemon restarts | P1 | RemoteConnection lifecycle (`remoteConn.Disconnect()` vs StartSSE again) — no test. |
| Hook command resolveServerJS picking stale lock | P0 | PID-liveness check via `process.kill(pid, 0)` is the only filter. Race: PID reused between check and POST. Acceptable but worth noting. |
| Tool notification dropped when broker subscriber buffer full | P0 | Non-blocking publish is BY DESIGN; subscriber buffer size in GUI is hard-coded (see notify_loop wiring in Pass B). If buffer too small under burst load, user misses a popup. Verify buffer size. |

## Test Build Strategy

`make test`: `go test -race -cover ./...` — race detector required.
`make test-unit`: `go test -race -cover ./internal/...` — internals only.
`make test-vhs TAPE=<name>`: Docker-based; **excluded from CI default** because Docker is required.

`tests/helpers_test.go` builds `bin/lazyclaude-test` once per test run (sync.Once), allowing real-binary integration tests at top level.

## State Checkpoint

```yaml
pass: 4
status: complete
test_files_indexed: 119 (internal) + 8 (cmd) + 7 (top-level tests/) + 8 VHS tapes
gaps_documented: 15
timestamp: 2026-05-11T18:30:00Z
next_pass: 5
```
