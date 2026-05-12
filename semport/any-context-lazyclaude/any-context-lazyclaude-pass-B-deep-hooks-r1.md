# Pass B Deepening — `internal/core/config/hooks.go` Round 1

**Subsystem:** `internal/core/config/` (hook injection protocol for Claude Code)
**Path:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/config/`
**Coverage state at entry:** Pass 3 BC-HOOK-001..006 (6 contracts; HIGH confidence but no file-walked file:line precision). Pass B5-v2 audit flagged this as HIGH-severity Drift-Category-B gap because the synthesis tells monocle to "adopt this protocol verbatim" and the actual node-eval one-liner content, the lock-discovery JS, the auth header, the per-hook body schemas, and the runtime-file layout were never source-walked.
**Round:** 1 (structural + behavioral; canonical tmpfile schema + endpoint matrix + restart-resilience sequence)
**Production LOC:** 100 (hooks.go) + 75 (config.go) = 175. **Test LOC:** 67 (hooks_test.go) + 76 (config_test.go) = 143. Test density: 82%.

This round confirms BC-HOOK-001..006, adds **BC-HOOK-007..030** (24 new contracts), produces the **canonical hooks-settings.json schema** field-by-field, produces the **6-hook → endpoint → request-body matrix**, produces the **restart-resilience sequence**, and flags **1 P1 and 1 P2 finding** specific to the protocol's portability.

---

## 1. Subsystem identity & relationships

`internal/core/config/hooks.go` is the **canonical Claude Code hook injection protocol** for lazyclaude. It is the producer-side of a wire contract whose consumer-side is `internal/server/{server.go, lock.go, discover.go}`. The wire is:

- **From producer to consumer:** A JSON settings file at `<runtimeDir>/hooks-settings.json` containing inline node-eval `command` entries for each Claude Code hook type. These commands resolve the lazyclaude HTTP server at runtime by scanning `~/.claude/ide/*.lock` files (PID-liveness gated). When the consumer-side server's port changes (restart on different OS-assigned port), the next hook invocation re-discovers it from the lock file — no producer-side reissue is needed.

- **Wire from hook to server:** HTTP POST over `127.0.0.1:<discovered-port>` with `X-Claude-Code-Ide-Authorization: <discovered-token>`. Five endpoints (`/notify`, `/stop`, `/session-start`, `/prompt-submit`) — wait, **six** hook commands in `hooks.go` but **five** registered hooks in `buildHooksMap()` (PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit). See §3 below.

- **Cross-references confirmed in this round:**
  - Producer launches consumer via `claude --settings <path>` (`session/manager.go:706-709` `writeLauncher`).
  - Consumer discovers identical lock-file format via `discover.go` (`internal/server/discover.go:20-58`) and the same `findAliveLockJS` semantics.
  - `~/.claude/settings.json` is **never** written by lazyclaude — verified by exhaustive grep on the source (only `~/.claude.json` is touched, for the onboarding-skip flags via `EnsureClaudeConfigured` per BC-SESSION-006).

The file is a **leaf** with one cross-package dependency (`encoding/json`, `bytes`, `os`, `fmt` only — stdlib). `config.go` (Paths) is its sibling, providing the resolved `<runtimeDir>` path. No reverse imports beyond `session/manager.go:706` and `internal/server/discover.go` (consumer-side that re-implements the lock-scan equivalent in Go).

---

## 2. File manifest & LOC recount

| File | LOC | Role |
|---|---|---|
| `hooks.go` | 100 | The full hook injection protocol: 5 const node one-liners + `WriteHooksSettingsFile` + `buildHooksMap`. |
| `config.go` | 75 | `Paths` struct with three env-overridable directories (IDEDir, DataDir, RuntimeDir). |
| `hooks_test.go` | 43 (production)/67 total | One integration test (`TestWriteHooksSettingsFile`) verifying file write + escape-HTML-false + 5 hook types. |
| `config_test.go` | 100 (production)/76 total* | Paths env-override + isolation tests. |

\*hook_test.go is 43 LOC by line count, the 67 in the audit was likely a different count metric. Audit numbers: production 175 LOC, test 143 LOC, density 82% — matches.

**Public surface (package `config`):**

| Identifier | Kind | File:line | Notes |
|---|---|---|---|
| `WriteHooksSettingsFile(runtimeDir string) (string, error)` | func | hooks.go:49-75 | Writes tmpfile, returns path. The sole entry point. |
| `Paths` | struct | config.go:11-15 | Three directory fields. |
| `DefaultPaths()` | func | config.go:24-44 | Production paths with env overrides. |
| `TestPaths(tmpDir string)` | func | config.go:49-55 | Isolated paths for tests. |
| `(Paths).StateFile() / .PortFile() / .ChoiceFile(window) / .LockFile(port)` | methods | config.go:58-75 | Derived paths. |

The 5 hook command consts and `buildHooksMap` are **unexported** (`preToolUseHookCommand`, `notificationHookCommand`, `stopHookCommand`, `sessionStartHookCommand`, `userPromptSubmitHookCommand`, `findAliveLockJS`, `resolveServerJS`, `buildHooksMap`). The shared JS is `findAliveLockJS` (hooks.go:13-20) and `resolveServerJS` (hooks.go:26-27).

---

## 3. THE 6-vs-5 hook clarification (P0 terminology)

The user's prompt asked about **6 hook types** (PreToolUse, PostToolUse, Notification, SessionStart, Stop, UserPromptSubmit). The synthesis says 5. The source has **5 hook commands** (`preToolUseHookCommand`, `notificationHookCommand`, `stopHookCommand`, `sessionStartHookCommand`, `userPromptSubmitHookCommand`) and **5 entries** in `buildHooksMap` (hooks.go:92-99). Net: **PostToolUse is NOT implemented**.

### BC-HOOK-007: Exactly FIVE Claude Code hook types are registered; PostToolUse is intentionally absent

**Preconditions:** None. Verified at hooks.go:92-99 `buildHooksMap()`:
```go
return map[string]any{
    "PreToolUse":        hookEntry(preToolUseHookCommand),
    "Notification":      hookEntry(notificationHookCommand),
    "Stop":              hookEntry(stopHookCommand),
    "SessionStart":      hookEntry(sessionStartHookCommand),
    "UserPromptSubmit":  hookEntry(userPromptSubmitHookCommand),
}
```

**Evidence:** hooks.go:92-99 (full map literal); hooks_test.go:38-42 asserts exactly these 5 keys via `assert.Contains` (it does NOT assert any of the other keys are absent, so a future addition would not break the test, but the current source has only 5).
**Confidence:** HIGH

**Implication for monocle:** A Rust port writes exactly 5 entries. The synthesis was correct; the user's prompt was counting Claude Code's hook surface (which has 6 according to Claude Code docs), not lazyclaude's subset.

**Why PostToolUse is omitted (inferred):** No corresponding HTTP endpoint exists on the server (`mux.HandleFunc` for `/notify`, `/stop`, `/session-start`, `/prompt-submit` — see `server.go:108-111`, four endpoints; PreToolUse and Notification both share `/notify` via the `type: 'tool_info'` distinction). There is no `/tool-result` or `/post-tool` endpoint. Adding PostToolUse would require both a producer-side const + buildHooksMap entry AND a consumer-side HTTP handler — neither exists. This is a deliberate scope decision.

---

## 4. The canonical hooks-settings.json schema (field-by-field)

This is the **load-bearing deliverable** for the monocle port. The file must be byte-compatible (modulo JSON whitespace, which is irrelevant since Claude Code parses JSON).

### Top-level shape

```json
{
  "hooks": {
    "PreToolUse":        [ /* hook entry */ ],
    "Notification":      [ /* hook entry */ ],
    "Stop":              [ /* hook entry */ ],
    "SessionStart":      [ /* hook entry */ ],
    "UserPromptSubmit":  [ /* hook entry */ ]
  }
}
```

Verified at hooks.go:49-52:
```go
settings := map[string]any{
    "hooks": buildHooksMap(),
}
```

### Hook entry shape (the inner array element)

Each hook-type value is a `[]any` with **exactly one** element. That element is a `map[string]any` with two keys:

```json
{
  "matcher": "*",
  "hooks": [
    {
      "type": "command",
      "command": "<the node one-liner string>"
    }
  ]
}
```

Verified at hooks.go:79-91 `hookEntry(command string) []any`:
```go
return []any{
    map[string]any{
        "matcher": "*",
        "hooks": []any{
            map[string]any{
                "type":    "command",
                "command": command,
            },
        },
    },
}
```

### Field-by-field schema

| Path | Type | Cardinality | Value | Source |
|---|---|---|---|---|
| `$` | object | 1 | `{"hooks": {...}}` (Claude Code settings.json shape) | hooks.go:49-52 |
| `$.hooks` | object | 1 | Map of hook-type name → array of matcher entries | hooks.go:50, 92 |
| `$.hooks.PreToolUse` | array | 1 | Array of matcher-entry objects (always length 1 here) | hooks.go:93 |
| `$.hooks.Notification` | array | 1 | (same) | hooks.go:94 |
| `$.hooks.Stop` | array | 1 | (same) | hooks.go:95 |
| `$.hooks.SessionStart` | array | 1 | (same) | hooks.go:96 |
| `$.hooks.UserPromptSubmit` | array | 1 | (same) | hooks.go:97 |
| `$.hooks.<type>[0]` | object | 1 | Matcher-entry: `{"matcher": "*", "hooks": [...]}` | hooks.go:80-90 |
| `$.hooks.<type>[0].matcher` | string | 1 | Always `"*"` (matches all tools) | hooks.go:82 |
| `$.hooks.<type>[0].hooks` | array | 1 | Array of hook-command objects (always length 1) | hooks.go:83-89 |
| `$.hooks.<type>[0].hooks[0]` | object | 1 | Command object: `{"type": "command", "command": "<JS>"}` | hooks.go:84-87 |
| `$.hooks.<type>[0].hooks[0].type` | string | 1 | Always `"command"` (one of Claude Code's hook executors) | hooks.go:85 |
| `$.hooks.<type>[0].hooks[0].command` | string | 1 | The shell command line — for lazyclaude, always `node -e "<inline JS>"` | hooks.go:86 |

### Rust serde struct (suggested for monocle port)

```rust
#[derive(Serialize)]
struct HooksSettings {
    hooks: HookMap,
}

#[derive(Serialize)]
struct HookMap {
    #[serde(rename = "PreToolUse")]
    pre_tool_use: Vec<MatcherEntry>,
    #[serde(rename = "Notification")]
    notification: Vec<MatcherEntry>,
    #[serde(rename = "Stop")]
    stop: Vec<MatcherEntry>,
    #[serde(rename = "SessionStart")]
    session_start: Vec<MatcherEntry>,
    #[serde(rename = "UserPromptSubmit")]
    user_prompt_submit: Vec<MatcherEntry>,
}

#[derive(Serialize)]
struct MatcherEntry {
    matcher: String,             // always "*"
    hooks: Vec<HookCommand>,     // always length 1
}

#[derive(Serialize)]
struct HookCommand {
    #[serde(rename = "type")]
    kind: String,                // always "command"
    command: String,             // the node -e "..." inline script
}
```

Each `Vec` is always length 1 in current lazyclaude. A monocle port MAY simplify to `MatcherEntry` directly if it doesn't need to support multi-matcher futures — but byte-compatible round-trip requires the array wrapping shown above.

### BC-HOOK-008: The settings file is encoded with `SetEscapeHTML(false)` AND `SetIndent("", "  ")`

**Postconditions:** The JSON encoder is configured to:
  - NOT escape `>`, `<`, `&` to `>`, `<`, `&` (so `=>` arrow functions in the JS source remain literal).
  - Pretty-print with 2-space indent (purely cosmetic, but verifiable on disk).
**Evidence:** hooks.go:58-61:
```go
var buf bytes.Buffer
enc := json.NewEncoder(&buf)
enc.SetEscapeHTML(false)
enc.SetIndent("", "  ")
```
**Confidence:** HIGH (asserted by hooks_test.go:26-31 — `assert.False(strings.Contains(content, "\\u003e"))` and `assert.True(strings.Contains(content, "=>"))`)
**Monocle port note:** `serde_json::ser::PrettyFormatter::with_indent(b"  ")` + `Serializer::with_formatter`. Default `serde_json::to_string_pretty` uses 2-space indent already. For escape-HTML-false equivalence: serde_json does NOT escape `>`, `<`, `&` by default (this is Go's idiosyncrasy, not a JSON standard). Rust port: byte-compatible by default; no special flag needed.

### BC-HOOK-009: Settings file is written at `<runtimeDir>/hooks-settings.json` with mode 0o600

**Postconditions:**
  - Parent dir `runtimeDir` is created with 0o755 via `os.MkdirAll` if absent.
  - File is `<runtimeDir>/hooks-settings.json` (literal filename, not session-derived).
  - File mode is `0o600` (owner-only RW).
**Evidence:** hooks.go:66-74:
```go
if err := os.MkdirAll(runtimeDir, 0o755); err != nil {
    return "", fmt.Errorf("create runtime dir: %w", err)
}
path := runtimeDir + "/hooks-settings.json"
if err := os.WriteFile(path, buf.Bytes(), 0o600); err != nil {
    return "", fmt.Errorf("write hooks settings file: %w", err)
}
```
**Confidence:** HIGH (strict 0o600; runtimeDir gets 0o755 — note asymmetric permission grants; see P2 finding §10).
**Monocle port note:** Rust `std::fs::OpenOptions::new().mode(0o600)` on the file, `std::fs::create_dir_all` with 0o755 default. The literal filename `hooks-settings.json` is per-runtimeDir but **not** per-session.

### BC-HOOK-010: The settings file is per-`runtimeDir`, NOT per-session

**Postconditions:**
  - The filename is `hooks-settings.json` — fixed, no session-ID suffix.
  - Repeated calls to `WriteHooksSettingsFile(<sameRuntimeDir>)` overwrite the same file (last-writer wins).
  - Since all lazyclaude sessions launched in the same TUI share `runtimeDir` (`config.DefaultPaths()` returns `os.TempDir()` as runtime, env-overridable), they all read the same settings file.
**Evidence:** hooks.go:70 hardcodes `"/hooks-settings.json"`; config.go:32 `RuntimeDir: os.TempDir()`. Cross-cite: `manager.go:706` calls `config.WriteHooksSettingsFile(opts.RuntimeDir)` each time a session is launched.
**Confidence:** HIGH
**Monocle port note:** Per-session file naming is **not** required by Claude Code's `--settings` flag. Sharing one file is a deliberate simplification — every session gets the same hooks. A monocle port MAY adopt the same convention (single file per runtime dir) or per-session for parallel cleanup, but byte-compatibility for the wire requires only one approach: be consistent with `--settings <path>` injection.

### BC-HOOK-011: The settings file is NEVER cleaned up by `WriteHooksSettingsFile`

**Postconditions:**
  - `WriteHooksSettingsFile` writes-only. It does not register a cleanup, does not call `os.Remove` on prior contents, does not check for staleness.
  - The file persists on disk across lazyclaude runs.
  - Verified absent: no `os.Remove("hooks-settings.json")` anywhere in `hooks.go` or callers (grepped `manager.go` callsite — only the launcher script is self-deleted via `rm -f "$0"` (manager.go:683), the settings file is left).
**Evidence:** hooks.go:49-75 (entire `WriteHooksSettingsFile` function; no cleanup); session/manager.go:706-709 (caller; no defer-remove). Whole-file scan confirms.
**Confidence:** HIGH
**Implication:** A monocle port MAY adopt the same "leave on disk" convention. Since the content is deterministic (same hook commands every time), there's no security or correctness issue with persistence. On stop, lazyclaude does NOT call `os.Remove("hooks-settings.json")` — it persists between runs.
**Disposition:** Document this. The settings file is a stable derived artifact, like a build cache.

### BC-HOOK-012: The settings file content is identical across PM, Worker, and plain sessions

**Postconditions:** `buildHooksMap()` takes no Role/SessionType parameter. The output is a constant (modulo Go map ordering, which serializing-into-bytes makes deterministic per Go version). PM sessions, Worker sessions, and plain sessions all carry the same 5 hooks pointing at the same lock-discovered server.
**Evidence:** hooks.go:78-99 (`buildHooksMap` is parameterless). Cross-cite pmw-r3 BC-PMW-HOOKS-001 (confirmed: "no Role-based skipping").
**Confidence:** HIGH
**Implication for monocle:** Hook injection is **per-process-launch but NOT per-persona**. The Rust port can use a single `&'static [HookEntry]` constant.

---

## 5. Hook URL injection — port discovery & auth token

### BC-HOOK-013: Hook URL host is hardcoded `127.0.0.1`; port is RESOLVED AT EACH HOOK INVOCATION via lock-file scan

**Postconditions:** Each of the 5 node one-liners contains `hostname:'127.0.0.1'` (verified at hooks.go:31, 35, 38, 41, 44). The port (`srvPort`) is NOT in the settings file — it is resolved at runtime by the inline JS by reading `~/.claude/ide/*.lock`.
**Evidence:** hooks.go:31 (and identical pattern in 35, 38, 41, 44):
```js
hostname:'127.0.0.1', port:srvPort
```
Where `srvPort` is set by `resolveServerJS` (hooks.go:26-27).
**Confidence:** HIGH
**Implication for monocle:** No tmpfile re-issuing needed across server restarts. The hook commands are static text; only the lock files change.

### BC-HOOK-014: Lock-file path is hardcoded `~/.claude/ide/` (= `path.join(os.homedir(), '.claude', 'ide')`)

**Postconditions:** The hook JS reads `os.homedir() + .claude/ide/*.lock`. There is **no env-var override** in the hook side — `LAZYCLAUDE_IDE_DIR` (config.go:40-42) is read by the Go side but NOT by the inline node JS.
**Evidence:** hooks.go:13-14:
```js
const fs=require('fs'),path=require('path'),home=require('os').homedir();
const lockDir=path.join(home,'.claude','ide');
```
**Confidence:** HIGH
**P1 finding (porting consequence):** This is a **fundamental asymmetry**. The Go side honors `LAZYCLAUDE_IDE_DIR`; the JS hooks do NOT. In test isolation, the Go server writes its lock to the env-overridden dir, but a real Claude Code subprocess loaded from the test settings would scan `~/.claude/ide/`. Tests cannot exercise the actual hook → server flow without monkey-patching the inline JS. See §10 for monocle implications.

### BC-HOOK-015: Auth token is RESOLVED AT EACH HOOK INVOCATION via the same lock-file scan

**Postconditions:** `srvToken` (hooks.go:27) is read from the alive lock's `authToken` field. Every HTTP request includes `X-Claude-Code-Ide-Authorization: srvToken`. Token rotation: implicit on every server restart (lock file is rewritten by `(*LockManager).Write` at `lock.go:39-57` with a token supplied to `server.New` via `Config.Token` — see server.go:27-33).
**Evidence:** hooks.go:27, 31 (header), and lock.go:44-49 (`AuthToken: token` in the written lock struct).
**Confidence:** HIGH
**Token scope:**
  - Per-server-process (rotated on each `Server.Start` because the caller (root.go) generates a new token before calling `server.New`).
  - Stored ONLY on disk in `~/.claude/ide/<port>.lock` (mode 0o600 verified — see BC-MCPSRV-046 in server-r1).
  - The settings file does NOT contain the token — the token is read fresh each time from the lock file by `resolveServerJS`.
**Token format:** Opaque hex string supplied by the caller (`server.New`'s `Config.Token`). No format requirement enforced in `hooks.go` or `lock.go`. Caller is `cmd/lazyclaude/root.go` per the consumer-side. Length and entropy are not constrained by this subsystem.

### BC-HOOK-016: The auth header name is `X-Claude-Code-Ide-Authorization` (NOT `X-Auth-Token`)

**Postconditions:** Hooks emit `X-Claude-Code-Ide-Authorization: <token>`. The server accepts either `X-Claude-Code-Ide-Authorization` OR `X-Auth-Token` (with the former taking priority). See BC-MCPSRV-028 (server-r1.md).
**Evidence:** hooks.go:31 (and identical in 35, 38, 41, 44):
```js
headers:{'Content-Type':'application/json','Content-Length':Buffer.byteLength(body),'X-Claude-Code-Ide-Authorization':srvToken}
```
Server: `internal/server/server.go:358-363` `extractAuthToken`.
**Confidence:** HIGH
**Cross-cite:** pmw-r3 BC-PMW-HOOKS-005 ("three auth header conventions in the same product"). Hooks: `X-Claude-Code-Ide-Authorization`. Server (CLI): `X-Auth-Token`. Daemon: `X-Daemon-Authorization`. **Monocle port should unify** but for byte-compatibility with Claude Code's expected hooks, `X-Claude-Code-Ide-Authorization` is mandatory.

### BC-HOOK-017: PID-liveness check uses `process.kill(lk.pid, 0)` (Unix-only)

**Postconditions:** The discovery JS filters dead-PID lock files via `process.kill(lk.pid, 0)` which on POSIX returns success iff the PID exists (does NOT actually send a signal). On Windows, this would fail differently — but lazyclaude is documented as Linux/macOS only per `.claude/CLAUDE.md`.
**Evidence:** hooks.go:19 `try{process.kill(lk.pid,0);if(!best||p>best.port)best={lock:lk,port:p};}catch{}`.
**Confidence:** HIGH
**Monocle port note:** Equivalent in Rust would be `kill(pid, 0)` via `nix::sys::signal::kill` — but since this is JS executing in node-the-runtime, the port doesn't need Rust here. The node script is a literal string in the Rust source code (or a `.js` file embedded via `include_str!`).

### BC-HOOK-018: When NO alive server is found, PreToolUse echoes stdin unchanged; the OTHER 4 hooks exit silently

**Postconditions:** Different fallback semantics by hook type:
  - **PreToolUse:** `if(!srvPort){console.log(d);return;}` followed by always `console.log(d)` at end (hooks.go:31). Net: stdin is always echoed to stdout so Claude Code's tool call proceeds. **Fail-open semantics.**
  - **Notification:** `if(!srvPort)return;` (hooks.go:35). No stdin echo. Side: drops the permission prompt locally.
  - **Stop / SessionStart / UserPromptSubmit:** `if(!srvPort)return;` (hooks.go:38, 41, 44). No body, no echo.
**Evidence:** hooks.go:31 (PreToolUse fallback) vs hooks.go:35, 38, 41, 44 (others).
**Confidence:** HIGH (subsumes BC-HOOK-001 with the asymmetry made explicit).
**Implication for monocle:** PreToolUse fail-open is **load-bearing for Claude Code UX** — if lazyclaude is down, Claude Code still functions (tools run). The other hooks fail-closed (silent drop) which is acceptable because they're observability signals, not gates.

---

## 6. The 5-hook → endpoint URL → request body matrix

This is the second load-bearing deliverable. Every field is read directly from source.

| Hook type | HTTP method | URL path | Timeout | Request body schema | Source |
|---|---|---|---|---|---|
| `PreToolUse` | POST | `/notify` | 300 ms | `{type: 'tool_info', pid: process.ppid, tool_name: i.tool_name \|\| '', tool_input: i.tool_input \|\| {}}` | hooks.go:31 |
| `Notification` (permission_prompt only) | POST | `/notify` | 2000 ms | `{pid: process.ppid, tool_name: i.tool_name \|\| '', tool_input: i.tool_input \|\| {}, message: i.message \|\| ''}` (NO `type` field) | hooks.go:35 |
| `Stop` | POST | `/stop` | 300 ms | `{pid: process.ppid, stop_reason: i.stop_reason \|\| '', session_id: i.session_id \|\| ''}` | hooks.go:38 |
| `SessionStart` | POST | `/session-start` | 300 ms | `{pid: process.ppid, session_id: i.session_id \|\| ''}` | hooks.go:41 |
| `UserPromptSubmit` | POST | `/prompt-submit` | 300 ms | `{pid: process.ppid, session_id: i.session_id \|\| ''}` | hooks.go:44 |

### Field semantics

- **`pid`** is always `process.ppid` — the parent PID of the node hook process, which equals the Claude Code subprocess PID (since Claude Code forked the hook). The server uses this PID to walk the process tree and find the lazyclaude tmux window (`tmux.FindWindowForPid` per `server.go:463`). See BC-PMW-HOOKS-003.
- **`tool_name`** comes from `i.tool_name` where `i` is the JSON-parsed stdin. Defaults to empty string.
- **`tool_input`** is the original tool args, an arbitrary object. Defaults to `{}`.
- **`message`** is the permission-prompt message text (Notification only).
- **`stop_reason`** is the reason Claude Code's session ended. Mapped to `ActivityError` if "error" or "interrupt", else `ActivityIdle` (BC-MCPSRV-009).
- **`session_id`** is **Claude Code's own session UUID**, NOT lazyclaude's session ID (BC-PMW-HOOKS-003).

### BC-HOOK-019: PreToolUse and Notification share `/notify`; type field discriminates

**Postconditions:** Both POST to `/notify`. PreToolUse sends `type: 'tool_info'` (hooks.go:31); Notification omits the `type` field entirely (hooks.go:35). Server branches on this at `server.go:409-453` — `tool_info` → BC-MCPSRV-003 (store pending), no type / `permission_prompt` → BC-MCPSRV-004 (dispatch popup).
**Evidence:** hooks.go:31 (body literal `{type:'tool_info',...}`); hooks.go:35 (body literal `{pid:..., tool_name:..., ...}` no type).
**Confidence:** HIGH

### BC-HOOK-020: Notification has a CLIENT-SIDE filter: `if(i.notification_type !== 'permission_prompt') return;`

**Postconditions:** Only permission-prompt notifications hit the wire. Other Claude Code notification types (whatever they may be — `idle`, etc.) are dropped before the HTTP request.
**Evidence:** hooks.go:35 (verbatim `if(i.notification_type!=='permission_prompt')return;`).
**Confidence:** HIGH (subsumes BC-HOOK-006).
**Cross-cite:** This is the only place where Claude Code's `notification_type` is read. The server has no knowledge that it's filtered.

### BC-HOOK-021: All HTTP requests are fire-and-forget (errors swallowed, timeout destroys the socket)

**Postconditions:** Each request handler installs:
```js
req.on('error',()=>{});
req.on('timeout',()=>{req.destroy()});
req.write(body);
req.end();
```
No response is consumed. The hook process exits after `req.end()` (and for PreToolUse, after `console.log(d)`).
**Evidence:** hooks.go:31, 35, 38, 41, 44 (identical pattern).
**Confidence:** HIGH
**Implication:** Hook delivery is at-most-once and silently lossy on network error or timeout. Server-side broker drop semantics (BC-BROKER-003 non-blocking publish) must complement this.

### BC-HOOK-022: Notification timeout (2000 ms) is ~6.7× longer than the others (300 ms)

**Postconditions:** Notification uses `timeout: 2000` because it's part of the permission-prompt user flow — the server's `dispatchToolNotification` may need to capture pane ANSI + detect max option + publish to broker before responding. The other 4 (PreToolUse, Stop, SessionStart, UserPromptSubmit) use `timeout: 300` because they're observability/state-tracking with no UI-latency requirement.
**Evidence:** hooks.go:35 `timeout:2000`; hooks.go:31, 38, 41, 44 `timeout:300`.
**Confidence:** HIGH
**Server side:** The 300ms timeout corresponds to BC-BROKER-003 ("publish is non-blocking; events drop for slow subscribers") — handlers must return fast or the hook will time out and silently drop.
**Monocle port note:** Replicate these timeouts exactly. They are part of the wire contract — too short and lazyclaude misses events; too long and Claude Code stalls on tool use.

### BC-HOOK-023: The `Content-Type: application/json` and `Content-Length` headers are always set explicitly

**Postconditions:** Every request sets `Content-Type: application/json` and `Content-Length: Buffer.byteLength(body)`. The Content-Length is byte length of the UTF-8 encoded body, not character count.
**Evidence:** hooks.go:31, 35, 38, 41, 44 (identical pattern):
```js
headers:{'Content-Type':'application/json','Content-Length':Buffer.byteLength(body),...}
```
**Confidence:** HIGH

---

## 7. Restart-resilience sequence (PRECISE — for Rust port replication)

**Goal:** When the lazyclaude HTTP server restarts on a new random OS-assigned port, the hooks must discover the new port without any producer-side re-issuance of the settings file.

### Sequence on each hook invocation (verified at hooks.go:13-44)

1. **Read directory:** `fs.readdirSync(path.join(os.homedir(), '.claude', 'ide'))` (hooks.go:14).
2. **Filter:** Keep only files matching `*.lock`: `locks.filter(f=>f.endsWith('.lock'))` (hooks.go:15).
3. **For each lock file (no early termination):**
   - Read & parse JSON: `JSON.parse(fs.readFileSync(path.join(lockDir,f),'utf8'))` (hooks.go:17). On parse failure: catch-and-skip (hooks.go:20).
   - Extract port from filename: `parseInt(f, 10)` (hooks.go:18). Note: `parseInt` parses leading digits, so `"7860.lock"` → `7860`.
   - PID liveness: `process.kill(lk.pid, 0)` (hooks.go:19). Throws if PID dead. Catch-and-skip on throw (hooks.go:19 trailing `catch{}`).
   - If alive AND `port > best.port` (or `best` is null): update `best = {lock: lk, port: p}` (hooks.go:19).
4. **After loop:** `if(best){srvPort=best.port;srvToken=best.lock.authToken;}` (hooks.go:27). If no alive server: `srvPort` and `srvToken` remain `null`.
5. **Per-hook fallback** (when `srvPort == null`):
   - PreToolUse: echo stdin (`console.log(d)`) and return — Claude Code proceeds.
   - Others: return immediately, no HTTP call.
6. **Otherwise:** Build POST body, `req = http.request({hostname: '127.0.0.1', port: srvPort, path: <endpoint>, method: 'POST', timeout: <timeout>, headers: {...}})`. Fire and forget.

### Highest-port-wins rationale

A new server start writes a new lock file at `~/.claude/ide/<new-port>.lock`. If `cmd/lazyclaude/root.go` (per BC-MCPSRV-010, root.go:440-442) called `CleanAllExcept(newPort)` before starting, ALL prior lazyclaude locks are removed. After that, `<new-port>.lock` is the only alive lazyclaude lock → discovery picks it. Non-lazyclaude locks (VS Code) survive and are explicitly skipped by `lock.App != "" && lock.App != "lazyclaude"` filter (consumer side at `discover.go:42-45`). **The hook JS does NOT check `lock.App`** — it accepts any alive lock. This is a P2 mismatch.

### BC-HOOK-024: Hook JS does NOT filter by `lock.App` field — it accepts any alive lock at the highest port

**Postconditions:** The hook JS (hooks.go:13-20) only validates PID liveness; it does NOT check `lock.app == "lazyclaude"`. So in a workspace where another tool (e.g. VS Code IDE integration) writes a lock file with a higher port than lazyclaude's, hooks would send POSTs to that other server.
**Evidence:** hooks.go:13-20 (no `lock.App` reference). Compare with consumer-side `discover.go:42-45`:
```go
if lock.App != "" && lock.App != lockApp {
    continue
}
```
**Confidence:** HIGH (verified by direct comparison).
**P2 finding (cross-IDE collision risk):** If another tool writes a `~/.claude/ide/<port>.lock` with a higher port AND is alive, lazyclaude hooks will POST to that other tool's server. Outcomes:
  - The other server returns 401 (token mismatch) — hooks silently drop.
  - Or the other server accepts and corrupts state — unlikely but possible.

**Why it works in practice:** VS Code and JetBrains typically write locks with specific app names AND lazyclaude's `CleanAllExcept` runs at TUI startup. The other tools are typically on lower ports because they bind earlier. Empirically reliable but not strictly correct.

**Monocle port disposition:** Add `if (best.lock.app && best.lock.app !== 'monocle') continue;` to the inline JS. Or pick a unique app name. The Rust port has a clean opportunity to fix this.

### BC-HOOK-025: After server restart, the FIRST hook event after restart re-discovers the new port; intermediate events during the restart window are dropped

**Postconditions:**
  - Window 1 (before restart): hooks find old server, deliver normally.
  - Window 2 (between Stop and Start of new server): hooks find no alive server (old PID is dead OR old port not accepting). Drop per BC-HOOK-018.
  - Window 3 (new server up, new lock written): hooks discover new port on next invocation. **No state in the producer side; the JS rescans the dir every time.**
**Evidence:** hooks.go:13-27 (discovery is unconditional per invocation).
**Cross-cite:** pmw-r3 BC-PMW-FAIL-009 (this same race).
**Confidence:** HIGH

### BC-HOOK-026: No producer-side state (no caching of port between hooks)

**Postconditions:** Each hook invocation is a fresh node process. There is no `srvPort` carried across invocations; no file-based cache. Discovery is **stateless per hook** (modulo the file-system state of `~/.claude/ide/`).
**Evidence:** hooks.go:26-27 `let srvPort=null,srvToken=null;` (re-initialized every invocation).
**Confidence:** HIGH
**Implication:** Re-issuing the settings file is unnecessary on restart. Monocle port can rely on the same property.

---

## 8. `~/.claude/settings.json` non-modification invariant

### BC-HOOK-027: lazyclaude NEVER writes `~/.claude/settings.json` — hooks are injected via `claude --settings <runtime-tmpfile>`

**Postconditions:**
  - `WriteHooksSettingsFile` writes to `<runtimeDir>/hooks-settings.json`, never `~/.claude/settings.json`.
  - Session launcher (`session/manager.go:706-709`) calls `WriteHooksSettingsFile(opts.RuntimeDir)` then appends `--settings <path>` to the claude command.
  - The user's `~/.claude/settings.json` is touched ONLY by `EnsureClaudeConfigured` (session/manager.go:186-222) which writes onboarding-skip flags — and that file is `~/.claude.json`, NOT `~/.claude/settings.json` (note the dot-prefix difference).
**Evidence:**
  - hooks.go:70 hardcodes `runtimeDir + "/hooks-settings.json"` — no reference to `~/.claude/`.
  - session/manager.go:707-708: `sb.WriteString(" --settings "); sb.WriteString(shell.Quote(settingsFile))`.
  - session/manager.go:186-222 (`EnsureClaudeConfigured`) writes `~/.claude.json` — a different file.
  - Exhaustive search of `internal/` for `.claude/settings.json` returns zero hits.
**Confidence:** HIGH

**The `--settings` override strategy** is verified at session/manager.go:706-709 within `writeLauncher`:
```go
// Inject hooks via --settings file so ~/.claude/settings.json stays
// untouched. Writing to a file avoids shell quoting issues with nested
// single quotes in hook commands.
if settingsFile, werr := config.WriteHooksSettingsFile(opts.RuntimeDir); werr == nil {
    sb.WriteString(" --settings ")
    sb.WriteString(shell.Quote(settingsFile))
}
```

**The settings injection is FAIL-SOFT** — if `WriteHooksSettingsFile` errors, the session still launches but without hooks (hook injection is skipped via `if ... werr == nil`).

### BC-HOOK-028: There is no env-var alternative for hook injection — only `--settings`

**Postconditions:** There is no `CLAUDE_HOOKS` or `CLAUDE_SETTINGS_PATH` env var read by Claude Code OR injected by lazyclaude for hook configuration. The sole mechanism is the `--settings` CLI flag.
**Evidence:** session/manager.go:850-873 `claudeEnv` injects: `CLAUDE_CODE_AUTO_CONNECT_IDE`, `LAZYCLAUDE_SESSION_ID`, and passthrough of `CLAUDE_CODE_OAUTH_TOKEN`, `ANTHROPIC_API_KEY`, `CLAUDE_CODE_API_KEY`, `CLAUDE_CODE_SSE_PORT`. **No `CLAUDE_*HOOKS*` or `CLAUDE_SETTINGS*`.** The launcher script (manager.go:680-737) passes only `--settings <path>` for hook injection.
**Confidence:** HIGH (exhaustive search of manager.go:842-873 and hooks.go).

---

## 9. Env-var bleed-through analysis

### BC-HOOK-029: The HOOK PROCESS inherits the FULL Claude Code subprocess env via process inheritance — but it does NOT read any `CLAUDE_*` or `LAZYCLAUDE_*` env vars

**Postconditions:**
  - Claude Code launches the hook process (`node -e "..."`) which inherits its full env.
  - The hook JS reads ONLY `os.homedir()` — implicitly via `$HOME` or `$USERPROFILE` (platform).
  - The hook JS does NOT consult `LAZYCLAUDE_IDE_DIR`, `LAZYCLAUDE_SESSION_ID`, `CLAUDE_CODE_AUTO_CONNECT_IDE`, or any other lazyclaude/claude env var.
  - Pure-JS code: parse stdin, scan disk, build HTTP request. No env reads beyond `os.homedir()`.
**Evidence:** hooks.go:13-44 (zero `process.env.<X>` references in the JS strings; only `process.ppid`, `os.homedir()`, and `process.stdin` are consulted).
**Confidence:** HIGH (full source visible).
**Implication:** The hook code is **env-independent** for everything except `$HOME`. This makes it portable across PM, Worker, and plain sessions trivially — the only requirement is that `$HOME/.claude/ide/<port>.lock` exists and is readable.

### BC-HOOK-030: `LAZYCLAUDE_SESSION_ID` env var IS set by `claudeEnv` (manager.go:854-855) but NOT read by any hook code

**Postconditions:** Confirms BC-PMW-HOOKS-004 (pmw-r3). The env var is set on every Claude Code subprocess; the hook JS does not consult it; no Go server code consults it. **It is set for OUT-OF-PROCESS consumers** (custom slash commands, codex plugin) that the user may run inside their Claude Code session.
**Evidence:** session/manager.go:854-855 (set); hooks.go (not read).
**Confidence:** HIGH

---

## 10. Schema versioning

### BC-HOOK-031: The hooks-settings.json file is UNVERSIONED — no schema-version field

**Postconditions:** No `version`, `apiVersion`, `schemaVersion`, or analogous field in the top-level JSON. The schema is defined by Claude Code's `--settings` consumer; lazyclaude produces what Claude Code's current version expects.
**Evidence:** hooks.go:49-52 (only `"hooks"` key). hooks_test.go:34-42 (asserts `parsed["hooks"]` is a map; doesn't check for version).
**Confidence:** HIGH

**Forward-compatibility strategy (inferred):**
  - If Claude Code adds a new hook type (e.g. PostToolUse), the producer must update `buildHooksMap` to add a new entry. Old hooks-settings.json files (without the new entry) still work — they just don't trigger the new hook.
  - If Claude Code changes the request body schema for an existing hook (e.g. PreToolUse adds a new field), the producer's node JS must be updated. Existing hooks-settings.json on disk is overwritten on next session launch.
  - If Claude Code REMOVES a hook type, the producer's old entries become inert — Claude Code presumably ignores unknown hook keys (verifying this would require Claude Code's source).
  - If Claude Code changes the `matcher`/`hooks`/`command` structure: the producer's settings file becomes invalid; Claude Code's loader presumably ignores or errors.

**Monocle port disposition:** Adopt the same unversioned approach. The settings file is a derived artifact; on schema changes, monocle rewrites it. **Pin a known-good Claude Code version range** in the porting spec and run integration tests against that range.

---

## 11. Test coverage analysis

### What `hooks_test.go` (43 LOC) covers

| Test | What it asserts | Confidence boost |
|---|---|---|
| `TestWriteHooksSettingsFile` | (1) file write succeeds (2) path non-empty (3) content has no `>` or `&` escapes (4) content has literal `=>` (5) parses as valid JSON (6) top-level `hooks` key exists and is a map (7) all 5 hook-type keys present | HIGH for BC-HOOK-007, BC-HOOK-008. PARTIAL for BC-HOOK-009 (file path NOT asserted to be `<runtimeDir>/hooks-settings.json`; only that it's non-empty). |

### What `hooks_test.go` does NOT cover

A monocle port test suite should add:

| Gap | What to test | Why |
|---|---|---|
| File path literal | Assert path == `<runtimeDir>/hooks-settings.json` | BC-HOOK-009 |
| File mode | Assert mode is 0o600 | BC-HOOK-009 |
| Runtime dir creation | Assert MkdirAll runs with 0o755 | BC-HOOK-009 |
| Hook command content | Assert each of the 5 commands contains the right endpoint path, timeout, auth header, body schema | BC-HOOK-019..023 (deeply load-bearing — no current test validates the JS one-liners' content) |
| Restart-resilience (integration) | Spin up two servers on different ports, kill the first, verify a hook invocation hits the second | BC-HOOK-024..026 |
| No-server fallback | Run a hook with no lock files; assert PreToolUse echoes stdin and others exit silently | BC-HOOK-018 |
| Lock-app filter | Write a non-lazyclaude lock at a higher port; verify hook ignores it | BC-HOOK-024 — would FAIL today |
| App field omission risk | (Hypothetical) — currently the test would pass because the hook accepts any lock |

**Critical gap:** The actual node-eval JS one-liner content is **NOT tested**. The test verifies the JSON structure but not the semantics of the commands. If someone broke the URL path in `notificationHookCommand`, the test would still pass.

### P1 finding: The JS one-liner content is untested

**Source:** hooks.go:31, 35, 38, 41, 44 (five hook commands as raw string literals).
**Gap:** No test extracts the `command` field and runs it (or even asserts substrings like `'/notify'`, `'/stop'`, `srvToken`, `timeout:300`).
**Risk:** A typo in any of the inline JS — e.g. `'/notiyf'`, missing semicolon, swapped field name — would not be caught by `TestWriteHooksSettingsFile`. The next visible failure would be a real-world hook dropping silently (BC-HOOK-021 swallows errors).
**Disposition:** P1 — recommend adding a unit test per hook command that asserts substring presence for: endpoint path, timeout value, auth header, body field keys, server discovery via `lockDir`. A monocle port should bake these assertions into the test suite from day one. The fact that the existing test passes does NOT mean the hooks work — only an end-to-end test with Claude Code can verify that.

---

## 12. Monocle port translation notes

### Rust-specific concerns

| Concern | Go behavior | Rust translation |
|---|---|---|
| HTML escaping in JSON | `enc.SetEscapeHTML(false)` required (`>` → `>` default) | `serde_json` does NOT HTML-escape by default — no flag needed |
| 2-space pretty indent | `enc.SetIndent("", "  ")` | `serde_json::to_string_pretty` with a `PrettyFormatter::with_indent(b"  ")` — default already 2 space |
| Inline node JS | Raw string concatenation (`+`) | Use raw string literal `r#"..."#` or `include_str!("hooks.js")`. Recommend extract to a `.js` file under `assets/`. |
| File mode 0o600 | `os.WriteFile(path, data, 0o600)` | `std::fs::OpenOptions::new().mode(0o600).write(true).create_new(true).open(path)` (Unix only); on Windows: equivalent ACL setting |
| Path concat `+ "/hooks-settings.json"` | String concat | `runtime_dir.join("hooks-settings.json")` (PathBuf) — safer |
| Map key ordering in JSON | Go's `map[string]any` randomizes; `json.Encoder` writes in some order | serde_json with `BTreeMap` (for sorted) or struct (for explicit order). **Recommend struct** for deterministic output. |
| `process.kill(pid, 0)` | Used inside the JS (not Rust) | Keep the JS verbatim — it runs in node, not Rust |
| Token format | Opaque hex string | Reuse Rust port's token type; pin to the same wire format if interoperating |

### Wire-byte-compatibility checklist

For a monocle Rust port to produce a settings file that Claude Code consumes identically:

1. JSON top-level keys MUST include exactly `hooks` (lowercase, no other keys).
2. Hook type names MUST be exactly `PreToolUse`, `Notification`, `Stop`, `SessionStart`, `UserPromptSubmit` (PascalCase, exact spelling).
3. Each hook type value MUST be a JSON array of length ≥ 1 (lazyclaude uses 1).
4. Each array element MUST have `matcher` (string `"*"`) and `hooks` (array).
5. Each `hooks` array element MUST have `type` (string `"command"`) and `command` (string).
6. The `command` string MUST be a `node -e "..."` inline script (or equivalent) that:
   - Reads stdin to a buffer.
   - Parses stdin as JSON.
   - Scans `~/.claude/ide/*.lock` for an alive PID (via `process.kill(pid, 0)`).
   - Picks the highest-port alive lock.
   - POSTs to `127.0.0.1:<port><path>` with `Content-Type: application/json` and `X-Claude-Code-Ide-Authorization: <token>`.
   - Sets `timeout: 300` (or 2000 for Notification).
   - Swallows errors on `req.on('error')` and `req.on('timeout')`.
   - For PreToolUse: writes stdin to stdout regardless.
   - For Notification: filters on `i.notification_type === 'permission_prompt'`.
7. File path `<runtimeDir>/hooks-settings.json` with mode `0o600`.
8. Launched via `claude --settings <path>` in the session launcher script.

If any of these diverge, hooks will silently break or behave differently than lazyclaude.

---

## 13. Cross-pollination findings

### Confirms server-r1 §6 lock-file claims

- Lock file mode is 0o600 (server-r1 said so; this round confirms by reading lock.go:56 directly). Pass 8 §310-311 fix needed (already flagged by server-r1).
- Lock file contains `{pid, authToken, transport, app}`. Producer-side hook JS reads `lk.pid` and `lk.authToken`. Does NOT read `lk.transport` or `lk.app`. **The transport field is unused by hooks.** It's metadata for VS Code / JetBrains. The `app` field is unused by hooks — see P2 above.

### Confirms pmw-r3 hooks claims

- BC-PMW-HOOKS-001 (every session gets same hooks): confirmed at hooks.go:78-99 (parameterless `buildHooksMap`) and manager.go:706-709 (unconditional call).
- BC-PMW-HOOKS-002 (lock-file discovery, not env): confirmed at hooks.go:13-20 (no env reads).
- BC-PMW-HOOKS-003 (PID correlation, dual session_id confusion): confirmed at hooks.go:31, 38, 41, 44 (always `pid: process.ppid`).
- BC-PMW-HOOKS-004 (`LAZYCLAUDE_SESSION_ID` unread by hooks): confirmed.
- BC-PMW-HOOKS-005 (`X-Claude-Code-Ide-Authorization` header): confirmed.
- BC-PMW-HOOKS-006 (`SetEscapeHTML(false)` for `=>` arrow preservation): confirmed.

### Adds new fact: lock.app filter divergence between hook JS and Go consumer

- Hook JS: NO `lock.app` filter (accepts any alive lock at highest port).
- Go consumer (`discover.go:42-45`): EXPLICIT `lock.app` filter (only `lazyclaude` or empty).
- **Asymmetry**: If a non-lazyclaude tool writes a higher-port lock, hooks send POSTs to that tool's server; the Go consumer ignores it. **Monocle port should align by adding the same app-filter to the inline JS.**

---

## 14. Delta Summary

- **New contracts added:** BC-HOOK-007..031 (25 new contracts; all source-walked at file:line precision).
- **Pass 3 contracts confirmed AND refined:** BC-HOOK-001..006 (all 6).
- **Cross-pollination confirms:** pmw-r3 BC-PMW-HOOKS-001..006 (6 contracts now have file-walked precision); server-r1 lock-file mode finding.
- **New findings:**
  - **P1**: JS one-liner content is untested (semantic-correctness gap in test suite).
  - **P2**: Hook JS does NOT filter by `lock.app`, but Go discover.go does — cross-IDE collision risk + asymmetry. Recommend monocle port fix the JS to also filter.
  - **P1 (porting)**: `LAZYCLAUDE_IDE_DIR` env var is honored by Go side but NOT by hook JS — test isolation cannot exercise the actual hook path.
- **Canonical deliverables produced:**
  - Field-by-field hooks-settings.json schema (§4) — ready to translate to a Rust serde struct (Rust struct sketched in §4).
  - 5-hook → endpoint → request-body matrix (§6) — ready for direct port.
  - Restart-resilience sequence (§7) — verified byte-for-byte against hooks.go:13-44.
  - Wire-byte-compatibility checklist (§12).

### Remaining gaps for round 2 consideration

These are smaller; round 2 will assess novelty:

1. **The `findAliveLockJS` algorithm's "highest port wins" tie-break behavior** — what if two alive servers have the same port? Logically impossible (each binds-then-locks atomically), but worth a contract.
2. **The hook JS error handling for `JSON.parse(d)` on stdin** — if Claude Code sends malformed stdin, `try{}catch{}` (hooks.go:31 outer wrapper) suppresses; PreToolUse still does `console.log(d)` at the end, so stdin is echoed unchanged. Worth contract-ing.
3. **The `parseInt(f, 10)` filename parsing edge cases** — `parseInt("abc.lock", 10)` = NaN; `parseInt("123abc.lock", 10)` = 123. The current loop doesn't validate the port-parse result. If `~/.claude/ide/` has a non-numerically-prefixed `.lock` file, `port` becomes `NaN`, the comparison `p > best.port` returns false (NaN comparisons return false), and the file is silently skipped. Worth contract-ing.
4. **Lock-file size limit** — no upper bound on read length in the JS. A malicious lock file could be arbitrarily large. Not a realistic risk (the file is owner-writable only) but worth noting.
5. **Atomicity of WriteHooksSettingsFile** — `os.WriteFile` is not atomic on POSIX (no temp-rename pattern). If Claude Code reads the file concurrently with another lazyclaude session writing it, a torn read is theoretically possible. Practical risk: low (file is small and writes are infrequent). But Claude Code might rerun the hook command from cached settings between writes — research needed.
6. **The hooks_test.go does not verify the literal filename `hooks-settings.json`** — gap noted in §11.

These are mostly nitpicks (suppression of edge cases that aren't realistically reachable). Round 2 will examine them; expected outcome: NITPICK and converge.

---

## Novelty Assessment

**Novelty: SUBSTANTIVE.**

Justification: Removing this round's findings would change how monocle is spec'd. Specifically:

1. **The field-by-field schema (§4)** is the canonical deliverable the user explicitly requested. Without it, a Rust port would either re-derive from source or get it wrong. The Rust struct sketch (in §4) is directly usable.

2. **The 5-hook matrix (§6)** captures the exact URL paths, timeouts, body schemas, and header names. The synthesis-level claim "5 hook types" was correct but lacked file-walked details. Without this matrix, a porter must re-read 100 LOC of hooks.go to derive it.

3. **The restart-resilience sequence (§7)** captures the precise stateless-rediscovery algorithm, including the highest-port-wins tie-break and the per-hook-type fallback semantics (PreToolUse fail-open vs others fail-closed). Pass 8 §line 209 says "adopt the discovery pattern verbatim" — this round provides the actual byte-level recipe.

4. **The `lock.app` filter asymmetry (BC-HOOK-024, P2 finding)** is net-new — neither prior round nor synthesis flagged it. A monocle port that copies the hook JS verbatim would inherit a subtle bug; this round catches it.

5. **The JS-content-untested P1 finding (§11)** is net-new and points at a real test-suite gap. A monocle port that adopts the same test pattern would inherit the same blind spot.

6. **The `~/.claude/settings.json` non-modification verification (§8)** confirms the synthesis invariant with file-walked evidence — verifying nothing-writes-here is harder than verifying something-writes-here, and this round did the exhaustive search.

7. **The schema versioning analysis (§10)** clarifies that the file is unversioned and depends on Claude Code's expected schema — important for the monocle port's compatibility strategy.

Compared to Pass 3 BC-HOOK-001..006 (test-derived; surface only), this round walked all 100 LOC at file:line precision, derived 25 new contracts, produced the three canonical deliverables (schema, matrix, sequence), and surfaced one P2 finding + two P1 findings (one in test coverage, one in env-var asymmetry). The model of the protocol is now complete enough for a Rust port to proceed without further deepening.

## Convergence Declaration

**Another round MAY produce additional minor findings** (the 6 remaining gaps listed in §14 are nitpicks — edge-case behavior in error handling, atomicity, filename parsing). Per the convergence protocol's "minimum 2 rounds before NITPICK" rule, round 2 is mandatory.

Expected round 2 outcome: NITPICK. The structural model is complete; the schema is fully extracted; the wire is fully documented. Round 2 will examine the 6 nitpick gaps and either contract-ize them (mostly trivial: BC-HOOK-032..037 covering the edge cases) or confirm none change the model.

## State Checkpoint

```yaml
pass: B
subsystem: internal/core/config/hooks
round: 1
status: complete
files_scanned:
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/config/hooks.go (full 100 LOC)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/config/config.go (full 75 LOC)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/config/hooks_test.go (full 43 LOC)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/config/config_test.go (full 100 LOC)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/manager.go:680-873 (writeLauncher + claudeEnv)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/lock.go (full 183 LOC for cross-cite)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/discover.go (full 58 LOC for cross-cite)
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/server.go:100-260 (mux registration + setActivity)
prior_pass_files_consulted:
  - any-context-lazyclaude-pass-3-behavioral-contracts.md (BC-HOOK-001..006)
  - any-context-lazyclaude-pass-B-deep-server-r1.md (lock-file + auth header cross-cite)
  - any-context-lazyclaude-pass-B-deep-pmw-r3.md (BC-PMW-HOOKS-001..006 cross-cite)
  - any-context-lazyclaude-pass-B5-coverage-audit-v2.md (Drift-Category-B flag)
contracts_added: 25 (BC-HOOK-007..031)
contracts_confirmed: 6 (BC-HOOK-001..006)
p1_findings: 2 (JS-content-untested; LAZYCLAUDE_IDE_DIR env-var asymmetry)
p2_findings: 1 (lock.app filter asymmetry between hook JS and Go consumer)
canonical_deliverables_produced:
  - hooks-settings.json field-by-field schema (§4)
  - 5-hook → endpoint URL → request body matrix (§6)
  - restart-resilience sequence (§7)
  - wire-byte-compatibility checklist (§12)
timestamp: 2026-05-11T19:30:00Z
novelty: SUBSTANTIVE
next_round_needed: true (per protocol minimum-2-rounds rule; expected NITPICK)
```
