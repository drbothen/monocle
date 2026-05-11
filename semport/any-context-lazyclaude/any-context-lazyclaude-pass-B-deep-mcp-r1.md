# Pass B Deep — `internal/mcp/` — Round 1

**Subsystem:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/mcp/`
**Total LOC:** 1,708 (per pass-0 inventory) — production 641 across `manager.go` (323), `config.go` (213), `ssh.go` (80), `model.go` (29); tests 1,067 across `manager_test.go` (207), `config_test.go` (323), `ssh_test.go` (496), `model_test.go` (43).
**Verified file sizes (bytes):** manager.go 10,660 · config.go 6,157 · ssh.go 3,100 · model.go 888 · manager_test.go 5,059 · config_test.go 8,187 · ssh_test.go 15,716 · model_test.go 855.
**Round:** 1 (structural + behavioral baseline)
**Sibling deepening files:** `*-deep-daemon-r{1,2,3}.md`, `*-deep-cmd-glue-r1.md`, `*-deep-pmw-r1.md`, `*-deep-session-r{1,2}.md`, `*-deep-tmux-r1.md`, `*-deep-profile-notify-r1.md`. This file uses prefix `*-deep-mcp-` and does not collide.
**Timestamp:** 2026-05-11

---

## 0. Terminology correction up front (P0 framing fix)

The task brief uses the phrase "the built-in MCP server that any-context/lazyclaude registers with Claude Code for IDE auto-discovery via lock-file scan" and asks to deepen `internal/mcp/`. **This is a terminology mismatch in the brief**; without correcting it, the deepening would target the wrong source.

Two distinct subsystems exist in this repo:

| Concern | Location | What it does |
|---|---|---|
| In-process **MCP server** (WebSocket + hook HTTP, lock-file `~/.claude/ide/<port>.lock`, ProtocolVersion 2024-11-05, `/msg/send`, `/msg/create`, `/msg/sessions`, `/notify`, `/health`) — BC-MCPSRV-001..020 | `internal/server/` (5,525 LOC per pass-8 §line 111) | The server that Claude Code's hook injection discovers via lock-file scan and POSTs to. Started in-process at TUI launch via `tryStartInProcessServer` (`cmd/lazyclaude/root.go`, also see CLAUDE.md `### MCP server` section). |
| Claude Code MCP **registry manager** (reads `~/.claude.json` and `<proj>/.mcp.json`, edits `<proj>/.claude/settings.local.json#deniedMcpServers`) | **`internal/mcp/`** — THIS SUBSYSTEM | The TUI's "MCP tab" model. Lets the user view all MCP servers Claude Code is configured to launch (user-scope + project-scope) and toggle their `deniedMcpServers` entry per-project. SSH-aware. Pass A assigned **NO BC-IDs** to this subsystem (pass-3 grep for `internal/mcp` returns zero hits inside contracts; only `BC-GUI-MSTATE-001` is the consumer-side BC noted in pass-8 §line 115). |

**Deepening scope for this file:** `internal/mcp/` ONLY — the registry manager. The MCP server (`internal/server/`) and the `/msg/send`/`/msg/create` API (BC-MCPSRV-015..020 in `internal/server/handler_msg.go`) are explicitly out of scope here. The "cmd-glue round mentioned BC-MCPSRV-015..020" note in the brief is a red herring for this round — those BCs live in `internal/server/handler_msg.go` (verified via `find ... -name 'handler_msg.go'`). Routing recommendation: if `/msg/*` needs deeper coverage, deepen `internal/server/` in a separate sibling round.

This terminology error is **P0 for monocle planning**: a porter who reads the brief literally and starts implementing "MCP server transport" against `internal/mcp/` will find no transport, no lock file, no JSON-RPC — and will conclude the analysis is broken. The two subsystems should be tracked as **MCPRegistry** (this file) and **MCPServer** (handled elsewhere) in monocle's spec.

The remainder of this document deepens **MCPRegistry**.

---

## 1. Subsystem purpose (one paragraph)

`internal/mcp/` is a **read-and-mutate adapter over three Claude Code config files** that lets the TUI's "MCP tab" display the user's configured MCP servers and per-project allow/deny state, then toggle the deny state via an atomic write that preserves all unrelated keys. The three files are: `~/.claude.json` (user-scope `mcpServers` map; mandatory), `<project>/.mcp.json` (project-scope `mcpServers` map; optional), and `<project>/.claude/settings.local.json#deniedMcpServers` (per-project deny list; optional, key absent ≡ empty). The same `Manager` operates against **local files via `os.ReadFile`** OR against **remote files via an injected `daemon.SSHExecutor`** (no SCP — content is base64-encoded and embedded in shell commands); the mode is selected by `host == ""`. The cached server view is a sorted slice merged from both scope maps with the deny set applied. A `writeMu` serialises the read-modify-write of `ToggleDenied`; a separate `mu` `RWMutex` guards the cached fields. The package has zero network code of its own and zero MCP-protocol code — it is a JSON file editor that happens to know what an MCP server config record looks like.

---

## 2. Structural inventory (per file)

| File | LOC | Public symbols | Private symbols |
|---|---|---|---|
| `model.go` | 29 | `ServerConfig` struct, `MCPServer` struct, `(ServerConfig).EffectiveType() string` | — |
| `config.go` | 213 | `ReadClaudeJSON(path string) (map[string]ServerConfig, error)`, `ReadDeniedServers(path string) ([]string, error)`, `WriteDeniedServers(path string, denied []string) error`, `MergeServers(user, project map[string]ServerConfig, denied []string) []MCPServer` | `claudeJSON`, `settingsLocal`, `deniedEntry` structs; `parseClaudeJSON`, `parseDeniedServers`, `updateDeniedInJSON`, `atomicWriteFile` |
| `manager.go` | 323 | `Manager` struct, `NewManager(userConfig string, ssh daemon.SSHExecutor) *Manager`, `(*Manager).SetProjectDir`, `(*Manager).SetHost`, `(*Manager).SetRemote`, `(*Manager).Refresh(ctx) error`, `(*Manager).Servers() []MCPServer`, `(*Manager).ToggleDenied(ctx, name) error` | `refreshLocal`, `refreshRemote`, `toggleDeniedLocal`, `toggleDeniedRemote`, package-level `remoteUserConfigPath` const, `toggleDenied`, `removeFromSlice` |
| `ssh.go` | 80 | — | `(*Manager).sshReadFile`, `(*Manager).sshWriteFile` |

**Package imports (production):**
- stdlib: `context`, `encoding/base64`, `encoding/json`, `errors`, `fmt`, `os`, `path/filepath`, `sort`, `sync`
- internal: `github.com/any-context/lazyclaude/internal/core/shell` (for `shell.Quote`), `github.com/any-context/lazyclaude/internal/daemon` (for `daemon.SSHExecutor` interface)

**No CGO. No reflection. No goroutines spawned inside the package** (caller may invoke methods from goroutines; methods are concurrency-safe under the locking contract documented at `manager.go:27-30`). No global state.

---

## 3. State model and locking (P0 for porter)

`Manager` carries six fields (`manager.go:31-39`):

| Field | Type | Lifetime | Guarded by |
|---|---|---|---|
| `writeMu` | `sync.Mutex` | per-instance | self |
| `mu` | `sync.RWMutex` | per-instance | self |
| `servers` | `[]MCPServer` | per-instance | `mu` |
| `userConfig` | `string` | set once at construction; never reassigned after `NewManager` returns | none after construction |
| `projectDir` | `string` | mutated by `SetProjectDir` / `SetRemote` | `mu` |
| `host` | `string` | mutated by `SetHost` / `SetRemote` | `mu` |
| `ssh` | `daemon.SSHExecutor` | set once at construction; never reassigned | none after construction |

**Locking order (explicit in the manager.go preamble, lines 27-30):** `writeMu` MUST be acquired before `mu` when both are held. Only `ToggleDenied` ever holds both; everywhere else `mu` is taken alone.

**The `SetHost` + `SetProjectDir` race that motivated `SetRemote`** (verified at `manager.go:62-85` and `internal/gui/mcp_state.go:18-24`): two consecutive setters expose a "mixed pair" window during which a concurrent `Refresh`/`ToggleDenied` goroutine spawned from a previous selection could observe `(host=new, projectDir=old)` and write the wrong remote file. `SetRemote` installs both under a single `mu.Lock` acquisition (`manager.go:80-85`). The GUI's `mcpAdapter` exposes only `SetRemote` to the GUI (`cmd/lazyclaude/root.go:750-752`), but the underlying `Manager` still exports `SetHost` and `SetProjectDir` as part of its API surface — **anyone holding a raw `*mcp.Manager` can still create the mixed-pair window**. The GUI test (`internal/gui/plugin_remote_disabled_test.go`) only exercises the adapter, not the bare manager.

**`host` capture pattern (P0 for monocle correctness).** `Refresh` reads `m.host` once under `mu.RLock` at `manager.go:91-93`, then passes the captured value to `refreshRemote` and downstream `sshReadFile`/`sshWriteFile` calls (`manager.go:152`, `ssh.go:43`, `ssh.go:61`). The SSH helpers **never re-read `m.host`** — they take `host` as an explicit parameter. The doc comment at `manager.go:148-151` and `ssh.go:38-42` explains the rationale: if the user navigates to a different host mid-flight, the in-flight operation must continue targeting the original host. This is verified by the regression test `TestManagerRefresh_RemoteHostCaptured` (`ssh_test.go:390-426`) which uses a `blockingSSHExecutor` to suspend the SSH call, swaps the manager's host, and asserts every call targets the captured host. **For monocle in Rust:** any reimplementation must replicate this capture; passing `&self` and reading `self.host` from inside an async send would silently regress.

**`ToggleDenied`'s "deliberately re-read m.host" comment at `manager.go:240-243`** is a partial counterpoint: after writing, the cache `Refresh` is allowed to re-read the now-current host because the *write* already completed against the captured host, and the cache should reflect what the user is now looking at. Subtle but correct.

---

## 4. The data model (P1 for porter)

`ServerConfig` (`model.go:5-12`) — the on-disk JSON record:

```go
type ServerConfig struct {
    Type    string            `json:"type,omitempty"`     // "stdio" | "http" | "sse" | "" (default stdio)
    Command string            `json:"command,omitempty"`  // stdio only
    Args    []string          `json:"args,omitempty"`     // stdio only
    Env     map[string]string `json:"env,omitempty"`      // stdio only
    URL     string            `json:"url,omitempty"`      // http/sse only
    Headers map[string]string `json:"headers,omitempty"`  // http/sse only
}
```

`MCPServer` (`model.go:15-20`) — the resolved row for the TUI:

```go
type MCPServer struct {
    Name   string         // map key from claude.json
    Config ServerConfig
    Scope  string         // "user" | "project"
    Denied bool
}
```

`EffectiveType()` (`model.go:23-28`) returns `"stdio"` when `Type == ""`. **Tested at `model_test.go:13-43`** with four cases (http, sse, stdio, empty→stdio). HIGH confidence.

**The deny list on-disk shape** (`config.go:18-24`) is an array of objects, not strings:

```json
"deniedMcpServers": [
  { "serverName": "filesystem" },
  { "serverName": "memory" }
]
```

The schema has only one field today (`ServerName`). Confirmed against `config.go:22-24` and the user-supplied test data at `config_test.go:117-138`. **Porter note:** Claude Code may add other fields here (e.g. `reason`, `until`); the current code drops empty-string entries silently (`config.go:54-57`) but does not validate field presence.

**Merge semantics** (`config.go:150-183`) — `MergeServers`:
1. Build `deniedSet` from the deny list.
2. Append all user-scope entries; flag `Denied=true` iff name ∈ deniedSet.
3. Append all project-scope entries; same flag rule.
4. **Sort by `Name`** ascending (`config.go:178-180`).

**Name-collision behavior is not defined by `MergeServers`** — if `user["github"]` and `project["github"]` both exist, **both rows survive** with different `Scope` values and identical `Name`. The sort is stable on `Name` so the relative order of two same-name entries is iteration-order-of-input (Go map iteration is non-deterministic). **This is a real edge case** worth a P1 disposition note for monocle: the TUI list will show two "github" rows when the user has it configured in both scopes; the toggle hits a single `deniedMcpServers` list keyed by name, so the deny flag will flip for both rows simultaneously. No test covers this; `manager_test.go:276-322` uses disjoint sets.

---

## 5. File-format contracts (P1 for porter)

### 5.1 `claude.json` (user-scope `~/.claude.json` and project-scope `<proj>/.mcp.json`)

Read-only from this package's perspective. Parser (`config.go:29-41`):
- Empty input → empty map (no error).
- Unmarshal failure → wrapped error.
- Missing `mcpServers` key → empty map (no error).
- Only the `mcpServers` field is unmarshaled (`config.go:13-15`); all other top-level keys (e.g. `oauthAccount`, `numStartups`, `hasCompletedOnboarding`) are simply ignored.

**Project-level `.mcp.json` is optional.** `refreshLocal` (`manager.go:131-135`) and `refreshRemote` (`manager.go:170-174`) both swallow read/parse errors for project-scope and fall back to `nil`. Comment at `manager.go:132` makes this explicit.

### 5.2 `settings.local.json` (deny list)

This file lives at `<project>/.claude/settings.local.json` (`manager.go:139`, `manager.go:249`). The package writes it via `updateDeniedInJSON` (`config.go:100-125`), which:
1. Unmarshals existing content into `map[string]json.RawMessage` — so **all unrelated top-level keys (`permissions`, `hooks`, `model`, etc.) survive byte-for-byte**.
2. If `denied` is empty: `delete(existingMap, "deniedMcpServers")` (the key is REMOVED, not set to `[]`).
3. Else: marshal `[]deniedEntry{...}` into `RawMessage`, store at `deniedMcpServers`.
4. `json.MarshalIndent(map, "", "  ")` + trailing newline.

**This is the key behavioral contract for cross-tool coexistence.** Claude Code itself writes `permissions`, `hooks`, etc. to the same file; if `mcp.Manager` round-tripped the whole struct via a typed unmarshal/remarshal, those keys would be lost. The `RawMessage` choice is deliberate and **tested directly** at `config_test.go:190-225` and (for the remote path) `ssh_test.go:293-361`.

**Subtle gotcha 1** (`updateDeniedInJSON` at `config.go:107-108`): when `denied == []string{}` (empty slice but non-nil), `len(denied) == 0` → the key is deleted. There is no way to write an explicit empty `deniedMcpServers: []` via this API. **For monocle:** this matches Claude Code's "absent ≡ allow all" convention.

**Subtle gotcha 2** (`config.go:120-124`): `json.MarshalIndent` does NOT preserve key order. When the existing file had `{"permissions": ..., "deniedMcpServers": ...}`, the rewritten file may swap them based on Go's map iteration. Currently no test asserts order preservation; Claude Code itself is presumed not to care. **For monocle (Rust):** if using `serde_json::Map` (preserves insertion order) the behavior diverges from Go and may be *more* stable, which is fine.

**Atomic write** (`config.go:187-212`) — `atomicWriteFile`:
1. `os.CreateTemp(filepath.Dir(path), ".settings-*.tmp")`
2. Write data.
3. `Close()`, then `os.Chmod(tmpName, perm)` (perm = `0o644`).
4. `os.Rename(tmpName, path)`.
5. All error paths `os.Remove(tmpName)`.

**Behavioral guarantees:** rename is atomic on POSIX same-filesystem. **A reader observing the file mid-write sees either the old or new content, never partial.** Permission set AFTER write (matching the typical Go atomic-write idiom). Parent directory creation: `WriteDeniedServers` calls `os.MkdirAll(filepath.Dir(path), 0o755)` at `config.go:131-133` BEFORE `atomicWriteFile` — verified by `TestWriteDeniedServers/creates parent directory if needed` (`config_test.go:257-273`).

### 5.3 Remote settings.local.json

The remote write does NOT use SCP and does NOT use `os.Rename`. It runs (`ssh.go:69-74`):

```
mkdir -p "$(dirname <remotePath>)" && printf '%s' '<base64>' | base64 -d > <remotePath>
```

This is **NOT atomic** — `>` truncates then writes. A concurrent remote reader (e.g. another `lazyclaude` instance, or `claude` itself) could observe an empty or partial file. **This is a behavioral divergence from the local path and is currently undocumented in pass-A.** P1 finding for monocle.

### 5.4 Remote read

```
if [ -f <remotePath> ]; then cat <remotePath>; fi
```

Missing file → empty output, exit 0 (treated as "key absent"). SSH failure → non-zero exit, wrapped error (`ssh.go:46-48`).

### 5.5 Path quoting rules (P0 for porter — they are non-obvious)

`remotePath` arguments to `sshReadFile`/`sshWriteFile` are **pre-quoted by the caller**. Two forms are valid (per the contract at `ssh.go:22-36`):

- `shell.Quote(absolutePath)` → single-quoted, e.g. `'/tmp/proj/.mcp.json'`. Safe against shell metacharacters because `shell.Quote` doubles single quotes as `'\''` (`internal/core/shell/quote.go:8-10`).
- The literal Go string `` `"$HOME/.claude.json"` `` (double-quoted) — used ONLY for the user-level config because `$HOME` expansion must happen on the remote side. Defined as the package constant `remoteUserConfigPath` at `manager.go:13-16`.

**No outer `sh -c` wrapper is used.** This is explicitly asserted in tests (`ssh_test.go:100-102`, `ssh_test.go:218-220`) and documented at `ssh.go:33-36`. The rationale: SSH itself runs the command via the remote shell; an outer `sh -c '<single-quoted-stuff>'` would collide with `shell.Quote`'s single-quoting. This is a non-obvious but provably correct pattern. **Pass-8 §confirmed (line 12) refuted a Pass-A P0 suspicion that `shell.Quote` was misused inside SSH — that refutation was correct.**

### 5.6 Base64-encoded write payload

`sshWriteFile` (`ssh.go:61-79`):
1. `base64.StdEncoding.EncodeToString([]byte(content))` — the encoded payload is ASCII-safe (`A-Za-z0-9+/=`).
2. Command: `mkdir -p "$(dirname <path>)" && printf '%s' <shell.Quote(encoded)> | base64 -d > <path>`.
3. `printf '%s' '<encoded>'` uses **explicitly single-quoted `'%s'`** for portability across POSIX printf (`ssh.go:65-68`); some busybox variants interpret an unquoted `%s` literally.

**Why base64 and not heredoc / single-quote-the-payload?** Heredoc would require escape gymnastics for shell-special terminators; raw single-quoting fails on payloads containing `'`. Base64 sidesteps both. The round-trip is verified at `ssh_test.go:165-172`.

---

## 6. Draft behavioral contracts (BC-MCPREG-*)

Pass A assigned zero BC-IDs to this subsystem. Drafting them here. Confidence per BC reflects test evidence per the "tests as first-class specs" rule.

### BC-MCPREG-001: `EffectiveType` defaults stdio when `Type == ""`

**Preconditions:** A `ServerConfig` value with any combination of fields.
**Postconditions:** Returns `Type` unchanged when non-empty; returns `"stdio"` when empty.
**Evidence:** `model.go:23-28`; `model_test.go:13-43` (4 cases).
**Confidence:** HIGH (table test).

### BC-MCPREG-002: `parseClaudeJSON` empty/missing-key/parse-failure semantics

**Preconditions:** `data []byte`, may be empty, may be `{}` with no `mcpServers`, may be malformed JSON, may be valid.
**Postconditions:**
- Empty `data` → `(map{}, nil)`.
- `{"foo":"bar"}` → `(map{}, nil)`.
- Malformed JSON → `(nil, wrapped error)`.
- Valid `{"mcpServers": {...}}` → `(map{...}, nil)`.
**Evidence:** `config.go:29-41`; `config_test.go:70-93, 95-107`.
**Confidence:** HIGH.

### BC-MCPREG-003: `parseDeniedServers` filters empty server names

**Preconditions:** `settings.local.json` bytes; may have `deniedMcpServers` entries with empty `serverName`.
**Postconditions:** Returns `[]string` containing only non-empty names; preserves order of input.
**Evidence:** `config.go:45-60` (filter at lines 54-57); `config_test.go:113-138`.
**Confidence:** MEDIUM — the empty-name filter is in code but **no test exercises a `{"serverName":""}` entry**. Behavior is grep-able from source but unvalidated.

### BC-MCPREG-004: `MergeServers` is deterministic by name-sort

**Preconditions:** `user`, `project` maps and `denied` slice.
**Postconditions:** Returns slice sorted by `Name` ascending; each entry has `Scope` from its source map and `Denied = (name ∈ denied)`.
**Evidence:** `config.go:150-183`; `config_test.go:276-322` (merge with 2 user + 1 project + 1 denied), `manager_test.go:11-69` (end-to-end via `Refresh`).
**Confidence:** HIGH for disjoint inputs; **MEDIUM-LOW for name collisions** — duplicate-name behavior (same name in both user and project) is uncovered. The code path appends both rows.

### BC-MCPREG-005: `WriteDeniedServers` preserves unrelated top-level keys

**Preconditions:** Existing `settings.local.json` containing `permissions`, `hooks`, etc.
**Postconditions:** The output file's top-level map contains exactly the same keys as the input PLUS or MINUS `deniedMcpServers` per the toggle direction; unrelated key values are byte-preserved (via `json.RawMessage`).
**Evidence:** `config.go:100-125`; `config_test.go:190-225` (asserts `permissions` survives); `ssh_test.go:293-361` (remote path, asserts `permissions` survives a base64 round-trip).
**Confidence:** HIGH (both local and remote paths tested).

### BC-MCPREG-006: Empty deny list REMOVES `deniedMcpServers` key (does not write `[]`)

**Preconditions:** Existing file has a non-empty `deniedMcpServers` array.
**Postconditions:** After `WriteDeniedServers(path, []string{} or nil)`, the file does NOT contain `deniedMcpServers`.
**Evidence:** `config.go:107-108` (delete branch); `config_test.go:227-255`; `manager_test.go:125-133`.
**Confidence:** HIGH.

### BC-MCPREG-007: `WriteDeniedServers` is atomic (local path)

**Preconditions:** A pre-existing target file; concurrent reader present.
**Postconditions:** Reader observes either the old content or the new content; never partial. Final mode is 0o644. Parent directory is created if missing.
**Evidence:** `config.go:187-212` (CreateTemp → Write → Close → Chmod → Rename → cleanup on error); `config_test.go:257-273` (parent directory creation tested).
**Confidence:** HIGH for code; **MEDIUM** for the "concurrent reader" guarantee — no race-detector test exercises a simultaneous read/write.

### BC-MCPREG-008: `WriteDeniedServers` remote path is NOT atomic

**Preconditions:** Remote target file exists; concurrent remote reader present.
**Postconditions:** The shell pipeline truncates then writes; a reader can observe an empty file between `>` and `printf | base64 -d` completion.
**Evidence:** `ssh.go:69-74`; no test covers the non-atomicity directly (asymmetric with local path).
**Confidence:** LOW for behavior intent, HIGH for code shape. **P1 finding — this divergence from the local atomic write deserves explicit disposition in monocle.**

### BC-MCPREG-009: `Manager.Refresh` selects local vs remote by `host == ""`

**Preconditions:** A `Manager` constructed via `NewManager(userConfig, ssh)`.
**Postconditions:**
- If `host == ""`: reads files from `os.ReadFile`. `ssh` is not invoked.
- Else: dispatches all three reads through `ssh.Run`.
- `host` is captured under `mu.RLock` at the top of `Refresh` and passed down explicitly.
**Evidence:** `manager.go:89-118`; `manager_test.go:11-181` (local); `ssh_test.go:192-251` (remote); `ssh_test.go:443-471` (round-trip remote → local).
**Confidence:** HIGH.

### BC-MCPREG-010: Mandatory vs optional file reads

**Preconditions:** `Refresh` is called.
**Postconditions:**
- User-scope `~/.claude.json` (or `"$HOME/.claude.json"` over SSH): mandatory. Read failure → wrapped error → `Refresh` returns error → cached `servers` NOT updated.
- Project `.mcp.json`: optional. Read failure → `projectServers = nil`. No error.
- Project `settings.local.json`: optional. Read failure → `denied = nil`. No error.
**Evidence:** `manager.go:120-144` (local), `manager.go:146-183` (remote); `manager_test.go:156-181` (no project), `ssh_test.go:253-275` (remote optional files missing), `ssh_test.go:277-291` (remote user-config SSH failure surfaces).
**Confidence:** HIGH.

### BC-MCPREG-011: `ToggleDenied` performs serialised RMW under `writeMu`

**Preconditions:** Two goroutines call `ToggleDenied` simultaneously on the same server name.
**Postconditions:** The two toggles run sequentially; the final on-disk state reflects two flips (back to start). Per `manager.go:198-203` doc: "two simultaneous toggles will serialise cleanly instead of racing on the filesystem / remote shell."
**Evidence:** `manager.go:204-244` (writeMu acquisition at line 205); `manager_test.go:71-134` (sequential toggle), but **no concurrent toggle test exists**.
**Confidence:** MEDIUM — locking is correct by inspection, but unvalidated by `-race`.

### BC-MCPREG-012: `ToggleDenied` fails on unknown server name

**Preconditions:** `serverName` not present in the cached `servers` slice.
**Postconditions:** Returns `fmt.Errorf("server not found: %s", serverName)`. No file write occurs.
**Evidence:** `manager.go:222-224`; `manager_test.go:136-154`.
**Confidence:** HIGH.

### BC-MCPREG-013: `ToggleDenied` fails when no project directory is set

**Preconditions:** `projectDir == ""`.
**Postconditions:** Returns `fmt.Errorf("no project directory set")`. No file write.
**Evidence:** `manager.go:225-227`; **no direct test** — `TestManagerRefresh_no_project` exercises Refresh-only, not Toggle.
**Confidence:** MEDIUM (code present, no test).

### BC-MCPREG-014: `ToggleDenied` re-reads `m.host` (not the captured value) for the post-write cache refresh

**Preconditions:** After a successful local or remote write, the user has already navigated to a different host.
**Postconditions:** The trailing `Refresh(ctx)` call at `manager.go:243` uses the now-current `host`, so the cache reflects what the user is looking at — NOT the host whose file was just mutated.
**Evidence:** `manager.go:239-243` (comment explicitly justifies this departure from the capture pattern).
**Confidence:** HIGH (intentional, documented).
**Porter note:** This is the inverse of BC-MCPREG-009's capture pattern. Both are correct in context. The capture protects the write target; the re-read serves the cache for the new selection.

### BC-MCPREG-015: SSH commands carry NO `sh -c` wrapper

**Preconditions:** Any remote operation.
**Postconditions:** The command string handed to `ssh.Run` does NOT begin with `sh -c '...'`.
**Evidence:** `ssh.go:33-36` (rationale), `ssh.go:44, 69-74` (assembly); `ssh_test.go:100-102, 152-153, 218-220` (negative assertions).
**Confidence:** HIGH.

### BC-MCPREG-016: SSH host capture across an in-flight `SetHost`

**Preconditions:** `Refresh` is running on goroutine G1; `SetHost("host-B")` is called from goroutine G2 between `Refresh`'s host-capture and the SSH call's actual dispatch.
**Postconditions:** Every SSH call made by G1's `Refresh` targets the captured `host-A`. `m.host` is never re-read inside the SSH helpers.
**Evidence:** `manager.go:89-93, 152` + `ssh.go:43, 61`; `ssh_test.go:390-426` (`blockingSSHExecutor` + `waitForCallCount`).
**Confidence:** HIGH (dedicated regression test).

### BC-MCPREG-017: `SetRemote` is the atomic alternative to `SetHost` + `SetProjectDir`

**Preconditions:** GUI navigates from selection A (host_a, dir_a) to selection B (host_b, dir_b) while an async `Refresh` from A is in flight.
**Postconditions:** Using `SetRemote(host_b, dir_b)`, no concurrent observer of `(m.host, m.projectDir)` can see `(host_a, dir_b)` or `(host_b, dir_a)`. Using `SetHost` followed by `SetProjectDir`, that mixed window exists.
**Evidence:** `manager.go:71-85` (single `mu.Lock` covers both writes); `internal/gui/mcp_state.go:18-29` (interface exposes only `SetRemote`); `cmd/lazyclaude/root.go:750-752` (adapter only forwards `SetRemote`).
**Confidence:** HIGH for the GUI consumer path; MEDIUM as a general contract — the bare `Manager` API still permits the non-atomic pair.

### BC-MCPREG-018: Remote `ToggleDenied` preserves unrelated keys through base64 round-trip

**Preconditions:** Remote `settings.local.json` contains `{"permissions": {...}}`.
**Postconditions:** After a `ToggleDenied`, the written bytes decoded from the embedded base64 still parse as a JSON object containing both `permissions` and the new `deniedMcpServers`.
**Evidence:** `manager.go:268-291` + `ssh.go:61-79`; `ssh_test.go:293-361` (decodes the written payload and asserts both keys present).
**Confidence:** HIGH.

### BC-MCPREG-019: `Servers()` returns a defensive copy

**Preconditions:** Caller mutates the returned slice.
**Postconditions:** The internal cache is unaffected.
**Evidence:** `manager.go:186-193` (allocates result then `copy(result, m.servers)`).
**Confidence:** MEDIUM — the slice header is copied, but `MCPServer` values contain map fields (`Config.Env`, `Config.Headers`); a caller mutating those maps WOULD affect the cache. No test exercises this. P2 nit.

### BC-MCPREG-020: `toggleDenied` (lowercase helper) is duplicate-safe

**Preconditions:** Caller's `currentlyDenied` view disagrees with the actual list (e.g. cached state is stale and the on-disk list already contains `serverName`).
**Postconditions:** When adding, the helper scans for an existing entry and returns a copy of the input unchanged (no duplicate).
**Evidence:** `manager.go:296-312` (lines 301-307 specifically); **no test exercises the duplicate-avoidance branch** — `manager_test.go` only toggles between empty-and-one-entry.
**Confidence:** MEDIUM (code present, untested).

---

## 7. Tests: what's covered and what's not (Pass A pass-3 gap closure)

The Pass-A pass-3 contract file marked BC-MCPSRV-* (server endpoints) as MEDIUM/LOW in some places. For **this subsystem (MCPRegistry)** no BCs existed; here is the fresh test coverage map:

### Production functions and their direct test coverage

| Function | File:line | Test(s) | Confidence |
|---|---|---|---|
| `ServerConfig.EffectiveType` | `model.go:23` | `model_test.go:5-43` | HIGH |
| `parseClaudeJSON` | `config.go:29` | covered indirectly via `ReadClaudeJSON`; direct empty-input branch (`config.go:30-32`) NOT directly tested | MEDIUM |
| `parseDeniedServers` | `config.go:45` | covered indirectly via `ReadDeniedServers`; empty-name filter at `config.go:54-57` NOT directly tested | MEDIUM |
| `ReadClaudeJSON` | `config.go:64` | `config_test.go:10-108` (4 subtests) | HIGH |
| `ReadDeniedServers` | `config.go:78` | `config_test.go:110-167` (3 subtests) | HIGH |
| `updateDeniedInJSON` | `config.go:100` | tested via `WriteDeniedServers` and ToggleDenied remote tests; the "empty input as `{}`" branch (`config.go:101-105`) NOT directly tested | MEDIUM |
| `WriteDeniedServers` | `config.go:130` | `config_test.go:169-274` (4 subtests) | HIGH |
| `MergeServers` | `config.go:150` | `config_test.go:276-322` | HIGH (disjoint names) / MEDIUM-LOW (name collisions across scopes) |
| `atomicWriteFile` | `config.go:187` | covered indirectly via `WriteDeniedServers`; failure paths (CreateTemp/Write/Close/Chmod/Rename errors) NOT directly tested | LOW |
| `NewManager` / `SetProjectDir` / `SetHost` / `SetRemote` | `manager.go:44-85` | covered via `Manager.Refresh` tests; `SetHost`+`SetProjectDir` race window NOT exercised; `SetRemote` race-protection asserted only at the interface-doc level | MEDIUM |
| `Manager.Refresh` (local) | `manager.go:89` | `manager_test.go:11-181` | HIGH |
| `Manager.Refresh` (remote) | `manager.go:89` (with `host != ""`) | `ssh_test.go:192-291, 390-471` | HIGH |
| `Manager.Servers` | `manager.go:186` | covered indirectly; **inner-map aliasing not tested** (BC-MCPREG-019) | MEDIUM |
| `Manager.ToggleDenied` (local) | `manager.go:204` | `manager_test.go:71-154` | HIGH |
| `Manager.ToggleDenied` (remote) | `manager.go:204` (with `host != ""`) | `ssh_test.go:293-361` | HIGH |
| `Manager.sshReadFile` | `ssh.go:43` | `ssh_test.go:74-134` (3 subtests including SSH error chain) | HIGH |
| `Manager.sshWriteFile` | `ssh.go:61` | `ssh_test.go:136-188` (2 subtests, base64 round-trip verified) | HIGH |
| `toggleDenied` helper duplicate-avoidance | `manager.go:300-307` | NOT tested | LOW |
| `removeFromSlice` | `manager.go:314-322` | covered indirectly via toggle-off path | MEDIUM |

### Verified gaps (uncovered behaviors)

1. **Name collision between user-scope and project-scope** — both rows survive merge; test absent. P1.
2. **`updateDeniedInJSON` with empty existing bytes** (`config.go:101-105`: "Empty input is treated as `{}`") — code path is reached during remote first-write where `existingJSON == ""`, but the local path always passes a non-empty `[]byte` because `WriteDeniedServers` ignores `os.ErrNotExist`. Code reached only via SSH. Not directly tested. P2.
3. **`atomicWriteFile` failure paths** — CreateTemp/Write/Close/Chmod/Rename errors. None tested. P2.
4. **Concurrent `ToggleDenied` calls** — sequential lock claim is asserted by inspection; no `-race`-validated concurrent test. P2.
5. **`Servers()` returned-slice inner-map aliasing** — the slice is copied but `Config.Env` / `Config.Headers` maps are shared. P2 nit.
6. **`toggleDenied` helper duplicate-avoidance branch** (`manager.go:301-307`) — defensive path against stale cache; not tested. P2.
7. **Remote `settings.local.json` write atomicity** — non-atomic by construction (BC-MCPREG-008); a porter unaware of this could ship a regression. P1 — needs explicit disposition.
8. **`SetHost` + `SetProjectDir` non-atomic pair on the raw Manager API** — protected via `SetRemote` in the adapter, but the unsafe setters remain exported. P2 (defense-in-depth nit; the GUI does the right thing today).
9. **No coverage for malformed JSON in `parseDeniedServers`** — `config.go:50-52` returns a wrapped error; the local `ReadDeniedServers` path never tests it (only valid JSON is fed in). P2.
10. **Schema evolution: `deniedEntry` has only `ServerName`** — if Claude Code adds fields (e.g. `reason`, `until`), `WriteDeniedServers` will round-trip them via `json.RawMessage` in `updateDeniedInJSON`... but the **`parseDeniedServers` reader** unmarshals into `deniedEntry`, dropping any new fields when re-emitted from `WriteDeniedServers` (because the writer re-marshals from the typed slice, not the raw bytes). **P1 schema-fragility finding** — see Section 9.

---

## 8. Concurrency model summary

| Site | Goroutine | Locks acquired | Notes |
|---|---|---|---|
| `NewManager` | caller | none | construction-time only |
| `SetProjectDir` | caller (typically GUI Update goroutine) | `mu.Lock` | writes `projectDir` |
| `SetHost` | caller | `mu.Lock` | writes `host` — NON-ATOMIC vs `SetProjectDir` if called separately |
| `SetRemote` | caller (GUI) | `mu.Lock` | writes both atomically |
| `Refresh` | typically `runMCPAsync` background goroutine (`internal/gui/app_actions.go:1128-1140`) | `mu.RLock` (capture) → release → ... → `mu.Lock` (cache write) | host captured at top; never re-read inside SSH helpers |
| `ToggleDenied` | `runMCPAsync` goroutine | `writeMu.Lock` (held entire RMW) → inside: `mu.RLock` (capture) → release → write → `Refresh` (acquires `mu.Lock` for cache) | writeMu serialises concurrent toggles; comment at `manager.go:198-203` |
| `Servers` | GUI render goroutine | `mu.RLock` | returns slice copy |

**`internal/mcp` itself spawns ZERO goroutines.** All concurrency originates with the caller (the GUI `runMCPAsync` helper). The package is goroutine-safe under documented locking order.

**Cancellation:** `Refresh` and `ToggleDenied` take a `context.Context`. The local path does not consult `ctx` (file IO is synchronous and ignores `ctx`); the remote path forwards `ctx` to `ssh.Run` and to `exec.CommandContext` (`internal/daemon/ssh.go:60`). Cancellation thus only affects in-flight SSH operations.

---

## 9. P0 / P1 / P2 findings

### P0
- **P0-MCPREG-A (terminology):** The task brief conflates `internal/mcp/` (registry manager) with `internal/server/` (the MCP server with lock-file discovery, JSON-RPC, `/msg/send`). Section 0 above resolves this. Monocle must spec **two** subsystems, not one. The lock-file scan, ProtocolVersion "2024-11-05", token, etc. live in `internal/server/`.
- **P0-MCPREG-B (host-capture pattern is load-bearing):** The "pass host explicitly through `sshReadFile`/`sshWriteFile`" pattern is the only thing preventing the GUI's host-swap race from misrouting writes. Rust port must replicate. BC-MCPREG-016.

### P1
- **P1-MCPREG-A (remote write non-atomicity):** BC-MCPREG-008. The local path uses temp-file-rename atomicity; the remote path does `> path` truncation. A remote `lazyclaude` or `claude` reading the file concurrently could observe an empty/partial state. Disposition options: (1) port verbatim and document; (2) upgrade remote write to write-to-temp-then-`mv` via SSH for parity; (3) gate remote MCP toggle behind a deferred-write strategy. Recommend (2) for monocle.
- **P1-MCPREG-B (schema-fragility on `deniedMcpServers` entry fields):** `deniedEntry` is `{ServerName string}` only. If Claude Code introduces additional per-entry fields (e.g. `reason`, `expiresAt`), `WriteDeniedServers` will silently drop them on the next toggle because the writer re-marshals from the typed slice. Monocle should consider modeling deniedMcpServers entries as a raw-JSON pass-through (similar to how top-level keys are preserved) to be schema-resilient.
- **P1-MCPREG-C (name collisions across scopes):** Section 4 + BC-MCPREG-004. When the same MCP server name appears in both `~/.claude.json` and `<proj>/.mcp.json`, two rows render in the TUI and a single toggle affects both. Worth a disposition decision: dedupe by name (project shadows user, matching Claude Code's actual resolution semantics), or surface both.

### P2
- **P2-MCPREG-A:** `Servers()` returns a slice-level copy; inner `Config.Env`/`Config.Headers` maps are shared. Document or make defensive (BC-MCPREG-019).
- **P2-MCPREG-B:** `SetHost` + `SetProjectDir` non-atomic pair remains on the public `Manager` API even though the GUI uses `SetRemote`. Could be unexported in monocle.
- **P2-MCPREG-C:** `atomicWriteFile` failure paths untested.
- **P2-MCPREG-D:** Concurrent `ToggleDenied` not exercised with `-race`.
- **P2-MCPREG-E:** `updateDeniedInJSON` does not preserve top-level key order across rewrites (Go map iteration). Cosmetic only — Claude Code is presumed not to depend on order. In Rust with `serde_json::Map`, order is preserved by default, so the port would be incidentally stricter.

---

## 10. Monocle relevance verdict

**Verdict: MEDIUM-HIGH retention, with split scope.**

Pass-8 §line 115 puts `internal/mcp` at **MEDIUM** and ties it to "consumer-side `BC-GUI-MSTATE-001`". I'd raise it to **MEDIUM-HIGH** for these reasons:

1. **It is a clean, well-bounded Rust port candidate.** Pure file IO + JSON manipulation, well-isolated SSH adapter, mature locking discipline, real test suite. Translation cost: low. Lines of behavior surface: ~641 production LOC.
2. **The deny-list edit-with-preservation contract (BC-MCPREG-005) is a real cross-tool coexistence gene.** Monocle (and any IDE-adjacent tool) needs to mutate `settings.local.json` without clobbering other tools' keys; the `map[string]json.RawMessage` pattern is the canonical solution and worth keeping verbatim.
3. **The host-capture concurrency discipline (BC-MCPREG-016) is high-signal for monocle's broader SSH story.** Any subsystem in monocle that runs ops against a "current target" the GUI can swap mid-flight needs this pattern. `internal/mcp/` is the cleanest small example to port first.
4. **The base64-payload + no-`sh -c` SSH command shape (BC-MCPREG-015, 018) is a portable convention** that should be lifted to a shared SSH helper crate in monocle, not reinvented per-subsystem.

**De-prioritisation argument:** If monocle does NOT ship an MCP-server-registry UX (no "MCP tab" equivalent), this subsystem's port is pure-overhead. In that case, drop to **LOW** and keep only the SSH conventions and the JSON-preserve trick for reuse elsewhere.

**Pass-8 retention status (§lines 696-697, 727-728):** `internal/mcp — PORT-ADAPT — if in scope`. This deepening confirms the "if in scope" conditional is correct; the contracts are now formalised so the disposition decision can be made cleanly.

---

## 11. Delta Summary

- **New BC drafts:** 20 (BC-MCPREG-001 through BC-MCPREG-020). Pass A had **zero** BCs assigned to this subsystem; all 20 are net-new.
- **Items refined from pass-1/pass-8:** the one-line description in pass-1 line 94 (mentioning `Manager`, `Refresh`, `Servers`, `ToggleDenied`, `SetRemote`) is expanded into 6 file-format contracts, 20 BCs, and a concurrency map.
- **New P0 findings:** 2 (terminology mismatch in brief; host-capture pattern as load-bearing).
- **New P1 findings:** 3 (remote write non-atomicity; deniedEntry schema fragility; cross-scope name collision).
- **New P2 findings:** 5 (inner-map aliasing, exported non-atomic setters, atomicWriteFile error paths, race-detector gap, key-order non-preservation).
- **Remaining gaps to interrogate in r2:** (1) confirm no other consumers of `mcp.Manager` beyond `cmd/lazyclaude/root.go` and the GUI adapter; (2) cross-check `internal/server/`'s independent file watching (if any) against this writer to see if a write here could disturb the in-process MCP server; (3) verify `findClaudeBinary` and onboarding flow do not also write `~/.claude.json` in a way that races with the user-config read here; (4) confirm `claude.json` shape against Claude Code's actual schema (we have the partial Go struct; is there any field this code drops that matters?); (5) confirm the `runMCPAsync` GUI loading-flag contract correctly handles ctx cancellation on app shutdown.

## 12. Novelty Assessment

**Novelty: SUBSTANTIVE**

Justification — would removing this round's findings change how I'd spec the system? **Yes**, on three independent axes:

1. The terminology correction (Section 0) **changes the deliverable structure**: monocle would now spec MCPRegistry and MCPServer as distinct components rather than rolling them up.
2. **20 net-new BC drafts** materially expand the contract surface from "documented as `Manager`, `Refresh`, `Servers`, `ToggleDenied`, `SetRemote`" (5 verbs) to a precise behavior catalog with confidence levels per contract. A porter armed with these BCs can write a Rust skeleton without re-reading the Go source.
3. The **P1 remote-write non-atomicity** and **deniedEntry schema fragility** findings are not present in any prior pass output. Both materially affect monocle's design choices (atomic remote write strategy; raw-JSON pass-through for entry fields).

## 13. Convergence Declaration

**Another round needed — substantive gaps remain.**

Specifically: the round-2 targets enumerated in §11. Of those, (4) — verifying the partial `claude.json` Go struct against Claude Code's actual schema by cross-referencing other places in the repo that read/write `~/.claude.json` — is the most likely to surface fresh substance (the `EnsureClaudeConfigured` path mentioned in pass-8 line 490 / BC-SESSION-006 writes the same file and may inform schema completeness). (1) and (3) are likely confirmation work. (2) and (5) are concurrency edge cases worth one more pass.

## 14. State Checkpoint

```yaml
pass: B
subsystem: internal/mcp
round: 1
status: complete
files_read:
  - internal/mcp/manager.go (323 LOC, 10660 bytes)
  - internal/mcp/config.go (213 LOC, 6157 bytes)
  - internal/mcp/ssh.go (80 LOC, 3100 bytes)
  - internal/mcp/model.go (29 LOC, 888 bytes)
  - internal/mcp/manager_test.go (207 LOC, 5059 bytes)
  - internal/mcp/config_test.go (323 LOC, 8187 bytes)
  - internal/mcp/ssh_test.go (496 LOC, 15716 bytes)
  - internal/mcp/model_test.go (43 LOC, 855 bytes)
  - internal/gui/mcp_state.go (consumer interface)
  - internal/gui/render_mcp.go (consumer UI)
  - internal/gui/app_actions.go lines 140-340, 1090-1140 (consumer driver)
  - cmd/lazyclaude/root.go lines 340-360, 740-780 (wire-up + adapter)
  - internal/daemon/ssh.go (SSHExecutor interface)
  - internal/core/shell/quote.go (Quote helper)
prior_context_consulted:
  - pass-0-project-discovery.md (LOC inventory, file manifest)
  - pass-1-architecture.md (line 94 single-row description)
  - pass-3-behavioral-contracts.md (confirmed ZERO BCs in this subsystem)
  - pass-8-final-synthesis.md (lines 64, 79, 81, 115, 117, 697, 728)
  - .claude/CLAUDE.md (clarified MCP server lives in internal/server/, not internal/mcp/)
timestamp: 2026-05-11
novelty: SUBSTANTIVE
next_round_targets:
  - cross-tool writers of ~/.claude.json (BC-SESSION-006 / EnsureClaudeConfigured)
  - other consumers of mcp.Manager beyond cmd/lazyclaude/root.go
  - any internal/server/ side-effects on the same files
  - schema completeness of partial ServerConfig struct (defer-able if Claude Code publishes a spec)
  - ctx cancellation behavior in runMCPAsync vs in-flight SSH
```
