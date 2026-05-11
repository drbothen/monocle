# Pass 2 — Conventions & Patterns: any-context/lazyclaude

## Package & File Organization

### Package layout pattern: `internal/<bucket>/<thing>/`

- **`internal/core/<thing>/`** — reusable primitives. One Go package per directory; package name matches directory: `event`, `lifecycle`, `tmux`, `config`, `model`, `shell`, `choice`, `debuglog`. No domain knowledge.
- **`internal/adapter/<thing>/`** — adapters that bridge `internal/core/` to higher-level needs. Currently one: `tmuxadapter`.
- **`internal/<thing>/`** at the second level — domain modules: `session`, `daemon`, `server`, `mcp`, `plugin`, `profile`, `notify`.
- **`internal/gui/<thing>/`** — TUI submodules under `gui/`: `chooser`, `keydispatch`, `keyhandler`, `keymap`, `presentation`. Files directly under `internal/gui/` are part of `package gui`.
- **`cmd/<binary>/`** — binary main. Each binary is one Go package main with multiple `<topic>.go` files.

### File naming

- Tests live alongside source: `manager.go` → `manager_test.go` (next to it, same package or `_test` external package).
- **External test packages** are used when the test needs only the public surface and to avoid import cycles, e.g. `package session_test` (manager_test.go:1), `package gui_test` (export_test.go variants).
- **Internal test packages** are used when private API access is needed, e.g. `package daemon` in server_test.go:1.
- `export_test.go` files exist (gui/export_test.go, session/export_test.go) — standard Go idiom to expose private symbols to external `_test` packages.
- `main_test.go` files exist when the package needs `TestMain` (server/main_test.go, session/main_test.go).
- Platform-specific files use the GOOS suffix: `proc_cwd_linux.go` vs `proc_cwd_other.go` (daemon/).
- Mock files are named `mock.go` and live with their source: `internal/core/tmux/mock.go`.

### Documentation conventions

- Every package has a `package <name>` doc comment on the first file, e.g.:
  - `// Package event provides a generic, thread-safe pub/sub event broker.` (event/broker.go:1-2)
  - `// Package daemon provides the HTTP daemon server for lazyclaude.` (daemon/server.go:1)
  - `// Package lifecycle provides a centralized cleanup registration mechanism.` (lifecycle/lifecycle.go:1-2)
- Exported types and functions carry doc comments. Many comments are **extensive** (10+ line essays embedded in code) describing rationale: e.g., daemon/server.go:617-632 documents `/profiles` HTTP semantics; daemon/remote_provider.go:34-65 documents callback concurrency contracts; root.go:805-869 documents the SessionID-hop rewrite pattern with Bug numbers.
- "Background:" prefix introduces multi-paragraph rationale comments (e.g. daemon/server_sse.go:141-148).
- Bug references are written as plain English ("See Bug 4 for the full trace.", "Bug 5 Phase B", "See #18" — root.go:813, 845; .claude/CLAUDE.md mentions "Notification popups are rendered as gocui overlays (display-popup notification mode removed in #18)").

## Naming Conventions

### Types

- **CamelCase** for exported, **camelCase** for unexported. Standard Go.
- **Suffix conventions:**
  - `Manager` — long-lived stateful service (session.Manager, daemon.LifecycleManager, plugin.Manager, mcp.Manager, daemon.MirrorManager, daemon.RemoteHostManager).
  - `Server` — listens-on-port HTTP/WS server (DaemonServer, server.Server, AskpassServer).
  - `Client` — talks to a Server (HTTPClient, ControlClient, ExecClient, MockClient).
  - `Service` — composes managers into a focused workflow (cmd/lazyclaude/SessionCommandService).
  - `Adapter` — type-shim between packages (sessionListerAdapter, sessionCreatorAdapter, pluginAdapter, mcpAdapter, guiCompositeAdapter — all in `cmd/lazyclaude/`).
  - `Provider` — implements a GUI/abstract interface from a concrete backend (LocalProvider, RemoteProvider, CompositeProvider).
  - `Executor` — runs subprocess commands (ExecSSHExecutor).
  - `Controller` / `State` — UI state holder (PopupController, FullScreenState, ScrollState, LogsState, MCPState, PluginState).
  - `Opts` (suffix) — option struct passed by value (WorktreeOpts, WorkerOpts, PMOpts, ResumeOpts, EnsureOpts, NewSessionOpts, NewWindowOpts).
  - `Option` (suffix, functional opt) — `daemon.DaemonOption`, `server.ServerOption`, `daemon.RemoteProviderOption`, `plugin.Option`. Constructed via `WithXxx(value)` helpers (e.g. `WithBroker(b)`, `WithVersion(v)`, `WithPostCreate(hook)`, `WithSSEActivity(cb)`, `WithSSEToolInfo(cb)`, `WithClaudePath(claudeAbs)`).
  - `Request` / `Response` (suffix) — JSON wire types in `daemon/api.go` and across handler files.
  - `Notification` (suffix) — event payload types in `internal/core/model/` (ToolNotification, StopNotification, SessionStartNotification, PromptSubmitNotification, ActivityNotification).
- **Interface convention:** small interfaces with descriptive single-method-ish names — `SessionLister`, `SessionCreator`, `SessionProvider`, `NotificationCacher`, `ConnectionAware`, `RoleSessionProvider`, `WorktreeProvider`, `SessionMutator`, `SessionActioner`, `ConnectionManager`, `ClientAPI`, `SSHExecutor`, `PopupManager`, `InputForwarder`, `Popup`.

### Constants and enums

- iota-based `int` enums with `String()` method. Example: `model.ActivityState` (notification.go:8-37):
  ```go
  type ActivityState int
  const (
      ActivityUnknown ActivityState = iota
      ActivityRunning
      ActivityNeedsInput
      ActivityIdle
      ActivityError
  )
  func (s ActivityState) String() string { ... }
  ```
- `ControlEventType` (core/tmux/control.go:14-23) follows the same pattern.
- String constants are SCREAMING_SNAKE_CASE-equivalent unexported camel: `findAliveLockJS`, `preToolUseHookCommand`, `notificationHookCommand`, etc. (core/config/hooks.go:13-44).
- HTTP header constants: `AuthHeader = "X-Daemon-Authorization"` (daemon/api.go:354).
- API version is a single int constant with version-history comments: `APIVersion = 4` (daemon/api.go:25).

### Variables

- Short receiver names: `s *Server`, `m *Manager`, `rp *RemoteProvider`, `c *ControlClient`, `lc *Lifecycle`.
- Variadic option pattern: `func New(cfg, …, opts ...ServerOption) *Server` with internal apply loop `for _, opt := range opts { opt(s) }` (server.go:81-99).
- `_ = err` is rare; `//nolint:errcheck` is used for fire-and-forget JSON encoders (server.go:629, 675, 726).

## Error Handling Patterns

### Wrap idiom

- Errors are wrapped with `fmt.Errorf("context: %w", err)` virtually everywhere a public boundary is crossed. Example shown in session/manager.go:258, 273, 318, 347, 472, 498; daemon/server.go:127, 152, 261; daemon/remote_provider.go:120, 138, 145.
- Bare `fmt.Errorf("...")` is used when there's no underlying error to wrap, e.g. `fmt.Errorf("control client closed")` (core/tmux/control.go:147).
- `errors.New` is rare; `fmt.Errorf` is the default even for string-only errors.

### HTTP error responses

- Pattern: `http.Error(w, "<short reason>", http.Status<Code>); return`. Example (daemon/server.go:219, 256, 277). Status codes used:
  - 400 BadRequest — bad JSON or missing required fields.
  - 401 Unauthorized — token mismatch (constant-time compare).
  - 404 NotFound — session/window not found.
  - 405 MethodNotAllowed — wrong HTTP verb.
  - 500 InternalServerError — unexpected manager error.
  - 502 BadGateway — tmux/IO error.
  - 503 ServiceUnavailable — `sessionCreator == nil`.
  - 413 (implicit via `MaxBytesReader`).
- JSON error responses for some endpoints use `writeJSON(w, http.StatusBadRequest, MsgSendResponse{Error: "..."})` (daemon/server.go:500-501) when the response shape requires JSON for the caller.

### Constant-time secret comparison

`subtle.ConstantTimeCompare([]byte(token), []byte(s.config.Token)) != 1` is the universal auth check (daemon/server.go:190, server.go:278, server/handler_msg.go:93, 197, 328, 401). Never `==` for tokens.

### Panic policy

- Cleanup functions are recovered defensively in `lifecycle.runCleanup` (lifecycle/lifecycle.go:75-82). Panic in one cleanup does not stop subsequent ones.
- The TUI process must never panic — errors flow through `app.ScheduleError(err)` (root.go:195, 303, app.go references).
- Test code uses `require.NoError(t, err)` and `t.Fatal` on setup errors.

## Concurrency Patterns

### Mutex hygiene

- `sync.Mutex` for write-mostly. `sync.RWMutex` when reads dominate (e.g. server.Server.activityMu, server.Server.mu, session.Manager.profilesMu).
- "Acquires the manager mutex to prevent races with Create/Delete" — explicit doc on session.Manager.Sync (manager.go:137-138).
- Lock-then-snapshot pattern when calling out to expensive code:
  ```go
  lc.mu.Lock()
  snapshot := make([]entry, len(lc.entries))
  copy(snapshot, lc.entries)
  lc.closed = true
  lc.mu.Unlock()
  // Run in reverse (LIFO) order without holding the lock.
  ```
  (lifecycle/lifecycle.go:48-63)
- Snapshot-before-cleanup pattern in PopupController.DismissAll (popup_controller.go:121-131).

### Channels

- `chan T` with bounded buffer + `select default` for drop-on-full. Examples:
  - Broker subscribers: default fixed size, drop on full (event/broker.go:69-74).
  - keyQueue: `chan keyCmd` capacity 1024 in FullScreenState (fullscreen.go:9, 25).
  - `gEvents` channel cap 20 in third_party/gocui (per .claude/CLAUDE.md "Bracketed paste is aggregated at the pollEvent level into a single eventPasteContent sent to gEvents").
- `done chan struct{}` for goroutine lifecycle signaling (control.go:121, daemon/tunnel.go:74, daemon/remote_provider.go:148).
- Context-aware select in all blocking reads:
  ```go
  select {
  case <-ctx.Done():
      return ctx.Err()
  case <-shutdownCh:
      return
  case evt := <-sub.Ch():
      ...
  }
  ```
  (daemon/server_sse.go:48-65)

### Goroutine leak prevention

- `go.uber.org/goleak` is imported (go.mod:16) — checked in tests.
- AskpassServer.handleConn sets read deadline to prevent leak from clients that connect and never send (daemon/askpass.go:169-170).
- ControlClient.Close has a 3-second timeout, then `Process.Kill()` if readLoop hangs (core/tmux/control.go:233-238).
- RemoteProvider.stopAndWaitSSE always waits for the goroutine to exit (daemon/remote_provider.go:170-184).
- Tunnel.waitForPort uses `context.WithTimeout` with deadline (daemon/tunnel.go:117-118).

### Non-blocking pubsub

The broker explicitly drops events when a subscriber's buffer is full, with rationale: "Subscriber buffer full; drop to preserve non-blocking guarantee." (event/broker.go:73). This is foundational to the hook handlers staying snappy under load — they must return quickly to the calling claude process.

## Testing Patterns

### Test helpers

- `func newTestManager(t *testing.T)`, `func newTestServer(t *testing.T)` — package-scoped factories return ready instances (session/manager_test.go:21-29, daemon/server_test.go:24-56).
- `t.Helper()` is the first line of every helper (manager_test.go:22, 75; daemon/server_test.go:25).
- `t.TempDir()` for filesystem isolation. `t.Cleanup(...)` for teardown registration (daemon/server_test.go:42, 53).
- `t.Parallel()` is liberally applied; tests are designed to be parallel-safe (manager_test.go:32, 51, 97, 111, 119).

### testify usage

- `require.NoError(t, err)` for setup checks where failure is fatal.
- `require.NotNil(t, x)`, `require.NoError(t, err)` — stop the test.
- `assert.Equal(t, want, got)` for non-fatal assertions.
- `assert.Empty(t, x)`, `assert.Len(t, x, n)` for collection checks.
- Imported as `"github.com/stretchr/testify/assert"` and `"github.com/stretchr/testify/require"`.

### Mocking

- The tmux client has a dedicated `MockClient` (core/tmux/mock.go) with exported fields (e.g. `mock.Sessions["lazyclaude"] = ...`) so tests can pre-populate state (manager_test.go:60, daemon/server_test.go:29).
- For higher-level testing, mocks are often inline anonymous structs implementing an interface (e.g. SessionLister, SessionCreator).
- httptest.NewServer is used for HTTP API tests (daemon/server_test.go:52).
- Process-level tests use real `exec.Command` to start subprocesses (e.g. ControlClient tests).

### Naming

- Pattern: `TestStruct_Method_Condition` — e.g. `TestManager_Create_FirstSession`, `TestManager_Create_SecondSession`, `TestManager_Delete_NotFound` (manager_test.go).
- Pattern: `TestFunc_Condition` for free functions — e.g. `TestHealth_NoAuth`, `TestAuth_Unauthorized` (daemon/server_test.go).
- Test files often include integration-leaning tests in `*_integration_test.go` (gui/app_integration_test.go, cmd/lazyclaude/routing_integration_test.go).

## Design Patterns In Use

| Pattern | Where | Why |
|---|---|---|
| **Functional options** | server.WithBroker, daemon.WithVersion, daemon.WithPostCreate, plugin.WithClaudePath, daemon.RemoteProviderOption | Forward-compatible config without breaking constructors |
| **Strategy** | PopupManager (PopupController impl), SessionProvider (LocalProvider vs RemoteProvider vs CompositeProvider), tmux.Client (ExecClient vs ControlClient vs MockClient), SSHExecutor | Swap implementations without changing call sites |
| **Adapter** | sessionListerAdapter, sessionCreatorAdapter (cmd/lazyclaude/root.go:467-555), pluginAdapter (root.go:687-743), mcpAdapter (root.go:746-777), guiCompositeAdapter | Bridge package types across boundaries without leaking |
| **Composite** | CompositeProvider dispatches to local or named-remote provider based on host (daemon/composite_provider.go) | Single SessionProvider surface for mixed local/remote |
| **Observer** | event.Broker[T] with Subscribe/Publish | Decouple hook emitters from consumers |
| **Command queue** | FullScreenState.keyQueue + RunKeyForwarder + dispatchBatch (fullscreen.go) | Order-preserving async input forwarding with literal batching |
| **Discriminated union** | model.Event with nullable pointer fields per variant (Notification, StopNotification, etc.) | Single Subscribe channel carries all event kinds |
| **Lazy connection** | RemoteHostManager.EnsureConnected (cmd/lazyclaude/remote_host.go) defers SSH until first use | Don't pay SSH cost on startup if user never uses remote |
| **Lifecycle registry** | core/lifecycle.Lifecycle with `lc.Register(name, fn)` + `defer lc.Close()` | LIFO cleanup, panic-tolerant, ordered |
| **Two-phase init** | server.New() → server.SetSessionLister(...) → server.Start() after adapters are wired | Decouple construction from dependency wiring |
| **Hooks-as-data** | core/config/hooks.go writes JSON file rather than embedding command strings in args | Avoids shell quoting; `SetEscapeHTML(false)` keeps `=>` literal |
| **Constant-time compare for secrets** | All token checks use `subtle.ConstantTimeCompare` | Timing-attack-resistant |
| **Deprecation markers** | `// Deprecated: Use CreateWorktreeOpts.` (session/manager.go:425-426, 453-454) | Backwards-compat shims for the older API |

## Anti-Patterns / Code Smells (Observed)

- **Two parallel hook-protocol HTTP servers.** `internal/server/handler_msg.go` and `internal/daemon/server.go` BOTH implement `/msg/send`, `/msg/create`, `/msg/resume`, `/msg/sessions`. The semantics differ slightly (daemon also has /profiles, /worktree/*, /session/*, /cwd, /health, /shutdown, /notifications SSE; server has /notify, /stop, /session-start, /prompt-submit, MCP /openDiff). This is intentional duplication for two different remote topologies, but means schema drift is possible — version skew tests would need to cover both. See Pass B coverage for verification gaps.
- **Long methods.** Some files are very large:
  - `internal/gui/app_actions.go` 1481 LOC
  - `internal/gui/layout.go` 1407 LOC
  - `internal/session/manager.go` 1127 LOC
  - `internal/daemon/server.go` 784 LOC
  - `internal/gui/keybindings.go` 764 LOC
  These are above the typical Go "split file if > 500 LOC" rule of thumb. They are partitioned by concern within the file (commented sections like `// --- Session CRUD handlers ---`), not by file.
- **Comment density.** Long rationale comments are heavily used in subtle code (root.go:805-870, daemon/remote_provider.go:34-73, daemon/server.go:617-632). This is more "load-bearing prose" than "self-documenting code." Future refactors must respect them — they encode Bug 4/5/etc. landmines.
- **Magic numbers behind constants.** Mostly named (`tunnelTimeout = 10 * time.Second`, `keyQueueSize = 1024`, `syncFailThreshold = 3`, `askpassReadTimeout = 10 * time.Second`, `tunnelPollInterval = 100 * time.Millisecond`). But the 50ms sleep in dispatchToolNotification before sending the cached diff key (server.go:432) is inline. The 100ms sleep in tryStartInProcessServer (root.go:405) is inline. The 120s askpass timeout (root.go:141) is inline. Acceptable but not centralized.
- **//nolint placement.** Used sparingly with reason (`//nolint:lll` for hook one-liners, `//nolint:errcheck` for fire-and-forget). Good discipline.
- **TODOs.** Tagged with topic, e.g. `// TODO(phase-2b): extend CreateLocalSession with profile/options when daemon route adds support.` (server/handler_msg.go:139). Plain `// TODO:` also exists (control.go:177).

## Cross-Package Type Sharing

- `internal/core/model` is the universal cross-cutting type holder: ActivityState, ToolNotification, Event, etc. EVERY OTHER package imports it.
- Wire types are duplicated when their semantics differ across boundaries:
  - daemon.SessionInfo vs server.SessionInfo vs session.Session vs gui.SessionItem — all conceptually a session, all defined per-package with explicit conversion at adapter layer. Trade-off: more boilerplate vs. zero cross-package leakage.
  - daemon.WorktreeInfo, server, gui.WorktreeInfo — same pattern.
- Profile has two representations: `profile.ProfileDef` (the canonical) and `daemon.ProfileDefAPI` (wire-encoded), with explicit `profileDefToAPI` conversion (daemon/server.go:658-677).

## Consistency Assessment

| Pattern | Adopted everywhere | Adopted sporadically |
|---|---|---|
| `fmt.Errorf("...: %w", err)` wrap | Yes |  |
| `subtle.ConstantTimeCompare` for secrets | Yes |  |
| Functional options for constructors | Yes |  |
| testify require/assert | Yes |  |
| `t.Helper()` in test helpers | Yes |  |
| `t.Parallel()` | Liberal (most tests) | A few sequential tests where shared filesystem state |
| Package doc comment | Yes (every package) |  |
| Exported type doc comments | Yes |  |
| Adapter pattern at composition root | Yes (cmd/lazyclaude/root.go is the layer) |  |
| iota+String() for enums | Yes |  |
| MaxBytesReader on POST bodies | Yes (1 MB cap, 10 KB for /msg/send body specifically) |  |
| Goroutine leak protection | Mostly | A few callback callsites mutate maps without obvious synchronization — see app.go:151-153 ("All reads/writes happen on the gocui event loop goroutine") |
| Long methods broken with `// --- <Section> ---` | Yes | Some files don't (smaller files don't need it) |

## State Checkpoint

```yaml
pass: 2
status: complete
files_scanned: ~20 (test patterns, naming, type shapes)
timestamp: 2026-05-11T18:00:00Z
next_pass: 3
```
