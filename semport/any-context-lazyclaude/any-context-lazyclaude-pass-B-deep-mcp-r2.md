# Pass B Deep — `internal/mcp/` — Round 2

**Subsystem:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/mcp/`
**Round:** 2 (gap closure — cross-tool writers, package consumers, ctx cancellation, schema completeness)
**Prior rounds:** `*-deep-mcp-r1.md` (this directory)
**Timestamp:** 2026-05-11

---

## 1. Round 2 scope: gaps named in r1 §11

R1 enumerated five remaining-gap interrogations:

1. Confirm no other consumers of `mcp.Manager` beyond `cmd/lazyclaude/root.go` and the GUI adapter.
2. Cross-check `internal/server/`'s independent file watching against this writer.
3. Verify `findClaudeBinary`/onboarding does not race the user-config read.
4. Schema completeness of partial `ServerConfig` struct.
5. `runMCPAsync` ctx cancellation on app shutdown.

All five are addressed below, plus one bonus finding from following the trail (the `mcpAdapter.Servers()` slice-aliasing pattern at the adapter boundary).

---

## 2. Gap (1) — Consumers of `mcp.Manager`

**Method:** Searched all `.go` files for `mcp.NewManager` and any `internal/mcp` import.

**Result:** Exactly two hits, both in `cmd/lazyclaude/root.go`:
- `cmd/lazyclaude/root.go:22` — `"github.com/any-context/lazyclaude/internal/mcp"` import.
- `cmd/lazyclaude/root.go:352` — `mcpMgr := mcp.NewManager(userClaudeJSON, ssh)`.

The `mcp.Manager` is **not** used anywhere else:
- Not in any other command (no `daemon`, `server`, `setup`, `sessions`, `msg`, `profile`, `askpass` subcommand path).
- Not in `internal/server/` (the in-process MCP server).
- Not in `internal/daemon/`.
- Not in `internal/plugin/`, `internal/session/`, or `internal/gui/` (the GUI talks to `MCPProvider` interface, satisfied by the `mcpAdapter` in `root.go:745-777`).

**Implication for monocle:** The wire-up is a single-call site. Porting requires re-implementing only `cmd/lazyclaude/root.go:340-360` and `cmd/lazyclaude/root.go:745-777` plus the `MCPProvider` interface (`internal/gui/mcp_state.go:24-29`). Minimal blast radius.

**Confidence:** HIGH. The search was exhaustive.

---

## 3. Gap (2) — Does `internal/server/` touch the same files?

**Method:** Searched all 18 `.go` files in `internal/server/` for any reference to `claude.json`, `.mcp.json`, `settings.local.json`, `deniedMcpServers`, or `mcpServers`.

**Result:** ZERO hits. `internal/server/` does not read, write, or watch any of the files this subsystem manages.

The in-process MCP server (`internal/server/`) only touches `~/.claude/ide/<port>.lock` (its own lock file) and the WebSocket/HTTP wire. It has no awareness of `mcpServers` (the registry it implements is keyed by hooks/`/notify` endpoints, not by the user's Claude Code MCP server list).

**Implication for monocle:** The two subsystems are **disjoint in their persistent state**. Nothing this subsystem writes can affect the MCP server, and vice versa. This is a clean port boundary — `MCPRegistry` and `MCPServer` can be specced, ported, and tested independently.

**Confidence:** HIGH.

---

## 4. Gap (3) — `EnsureClaudeConfigured` interaction with the MCP user-config read

**This is the highest-value finding of round 2.** The session manager has an `EnsureClaudeConfigured` function that writes the same `~/.claude.json` the MCP manager reads. It runs at TUI startup. Detailed analysis follows.

### 4.1 Where and when it runs

`internal/session/manager.go:186-222`:

```go
func (m *Manager) EnsureClaudeConfigured(dirPath string) {
    configPath := filepath.Join(os.Getenv("HOME"), ".claude.json")
    var cfg map[string]any
    if data, err := os.ReadFile(configPath); err == nil && len(data) > 0 {
        if json.Unmarshal(data, &cfg) != nil {
            cfg = make(map[string]any)
        }
    } else {
        cfg = make(map[string]any)
    }
    if completed, ok := cfg["hasCompletedOnboarding"].(bool); ok && completed {
        return
    }
    cfg["hasCompletedOnboarding"] = true
    cfg["numStartups"] = 10
    projects, _ := cfg["projects"].(map[string]any)
    if projects == nil {
        projects = make(map[string]any)
    }
    if abs, err := filepath.Abs(dirPath); err == nil {
        projects[abs] = map[string]any{"hasTrustDialogAccepted": true, "allowedTools": []any{}}
    }
    projects["/"] = map[string]any{"hasTrustDialogAccepted": true, "allowedTools": []any{}}
    cfg["projects"] = projects
    if out, err := json.Marshal(cfg); err == nil {
        os.WriteFile(configPath, out, 0o600)
    }
}
```

It is called from `cmd/lazyclaude/root.go:95`:
```go
mgr.EnsureClaudeConfigured(".")
```

This call site is at the **TUI startup**, BEFORE `mcp.NewManager` at line 352. Same goroutine. Sequential. **No intra-process race window for the initial read.**

### 4.2 Behavioral comparison: EnsureClaudeConfigured vs the mcp package's settings.local.json writer

| Property | `EnsureClaudeConfigured` (session/manager.go:190) | `WriteDeniedServers` (mcp/config.go:130) |
|---|---|---|
| Target file | `~/.claude.json` | `<proj>/.claude/settings.local.json` |
| Read strategy | `os.ReadFile`; unmarshal failure → restart from empty map (silently wipes existing!) | `os.ReadFile`; unmarshal failure → wrapped error (preserves existing) |
| Round-trip type | `map[string]any` | `map[string]json.RawMessage` |
| Write strategy | `os.WriteFile` (NON-atomic; truncate-then-write) | `atomicWriteFile` (CreateTemp → Rename) |
| Mode | `0o600` | `0o644` |
| Format | `json.Marshal` (compact, no indent) | `json.MarshalIndent("", "  ")` + trailing `\n` |
| Error handling | All errors silently swallowed (`if err == nil` patterns; no return value) | Errors wrapped and returned |
| Idempotency guard | Returns early if `hasCompletedOnboarding == true` | None — every call writes |

**Three substantive divergences worth flagging:**

1. **Silent-wipe-on-parse-failure** (`session/manager.go:195-197`): if `~/.claude.json` exists but is corrupt, `EnsureClaudeConfigured` restarts from an empty map and writes a new file containing only the onboarding flags. **The user's `mcpServers` and all other Claude Code settings would be lost.** This is in the session package, not the mcp package, but it directly affects what `mcp.Manager` reads on subsequent calls. **New P1 cross-subsystem finding.**

2. **Non-atomic write to `~/.claude.json`**: `os.WriteFile` truncates then writes. If a concurrent `claude` subprocess or another process reads the file mid-write, it sees partial content. Since `EnsureClaudeConfigured` only runs once at startup (before any `claude` subprocess is launched and before `mcp.NewManager`), the intra-process race is impossible — but a cross-process race with a running `claude` instance (started outside lazyclaude) is theoretically possible. P2.

3. **Mode 0600 vs the file's prior mode**: if `~/.claude.json` was 0644 before, `os.WriteFile` will set 0600 on rewrite. This may be intentional (the file contains auth tokens) but is undocumented. P2-aesthetic.

### 4.3 Does it affect mcp.Manager?

**Intra-process:** No race. `EnsureClaudeConfigured` runs at line 95, `mcp.NewManager` at line 352. Sequential, same goroutine.

**Cross-process:** If two `lazyclaude` instances start simultaneously, both call `EnsureClaudeConfigured`. The second one's parse-failure-recovery branch could be triggered if it observes a partial write from the first. **This is a known instance of P0-MCPREG-A from the architecture pass-1 line 94 description ("setup mode") but no test exercises concurrent startup.**

**With external `claude` writers:** Claude Code itself updates `~/.claude.json` (e.g. `numStartups`, project trust state). The exact write strategy used by `claude` is not visible from this repo. If `claude` uses atomic writes, the mcp.Manager's read is safe. If not, mcp.Manager could observe a partial JSON and return a wrapped parse error to the GUI's `runMCPAsync` callback (surfacing as "MCP error: parse ~/.claude.json: unexpected end of JSON input" in the GUI). P2 — runtime-recoverable via the user retrying.

### 4.4 New BC drafted

### BC-MCPREG-021: `mcp.Manager` does NOT write to `~/.claude.json`

**Preconditions:** Any operation.
**Postconditions:** The mcp package never opens `~/.claude.json` for writing. Only `<proj>/.claude/settings.local.json` is ever written.
**Evidence:** No `os.WriteFile`, `os.Create`, or SSH-write call in `internal/mcp/` ever targets `claude.json`. Confirmed by reading all four production files.
**Confidence:** HIGH.

**Implication for monocle:** A porter could naively decide that "the registry manager manages all MCP registry state" and add a `WriteMCPServers` function. Don't. The user-scope mcpServers list is owned by Claude Code; lazyclaude only edits the per-project deny list. Cross-tool coexistence depends on this asymmetry.

---

## 5. Gap (4) — `ServerConfig` schema completeness

### 5.1 The Go struct vs what Claude Code actually writes

`ServerConfig` (`internal/mcp/model.go:5-12`) declares six fields with `omitempty`: `Type`, `Command`, `Args`, `Env`, `URL`, `Headers`. All are stdlib JSON types.

**MCP server config in the wild** (from Anthropic's published examples and the test fixtures here at `manager_test.go:18-34, 76-82` and `ssh_test.go:194-200, 257`) includes only these fields. The tests cover:
- `command` + `args` + `env` (stdio with secret)
- `type: "http"` + `url`
- `type: "sse"` (only in `model_test.go:22-23` as a unit-level case for `EffectiveType`)

**What might Claude Code add that this struct drops?** Anthropic's MCP spec includes a `description` field for human-readable server descriptions, and could plausibly add `cwd`, `disabled`, or `timeout`. **None of these are in the Go struct, so any such fields would be silently dropped if this package re-marshalled the user config — but the mcp package NEVER re-marshals `~/.claude.json` (BC-MCPREG-021)**. The struct is read-only-once for display, so unknown fields are simply not surfaced to the TUI. No data loss; only display gaps.

**Confidence on completeness:** MEDIUM. The set covers everything tested and matches the visible Claude Code surface today. A monocle port should preserve the same read-only semantics and let unknown fields drop quietly.

### 5.2 The `<proj>/.mcp.json` shape

The same struct is reused for the project-level file. The test at `manager_test.go:31-34` uses `{"mcpServers": {"my-db": {"command": "node", "args": ["db.js"]}}}` — identical shape. The Anthropic MCP server project-config spec matches.

**Confidence:** HIGH (test coverage exists).

### 5.3 Schema fragility on `deniedEntry` re-examined

R1's P1-MCPREG-B flagged that `deniedEntry` has only `ServerName` and silently drops unknown per-entry fields on re-write. Re-examining: `updateDeniedInJSON` at `config.go:107-119` **re-marshals from `[]deniedEntry`** (not from raw bytes), so any new field Claude Code adds (e.g. `{serverName, reason}`) would be **lost on the next toggle**. The fix is to model `deniedMcpServers` as `[]json.RawMessage` and pass through the raw bytes for entries whose `serverName` matches an existing entry. This is a real port-time improvement opportunity.

**Confidence:** HIGH on the diagnosis, MEDIUM on the urgency (Claude Code may never extend the schema).

---

## 6. Gap (5) — `runMCPAsync` ctx cancellation on shutdown

### 6.1 The finding

`internal/gui/app_actions.go:1128-1140`:

```go
func (a *App) runMCPAsync(fn func(ctx context.Context) error) {
    a.mcpState.loading = true
    go func() {
        err := fn(context.Background())
        a.gui.Update(func(g *gocui.Gui) error {
            a.mcpState.loading = false
            ...
        })
    }()
}
```

The goroutine receives `context.Background()` — **non-cancellable**. There is no app-wide context that can be derived.

### 6.2 Shutdown implications

On TUI shutdown (`lc.Register` cleanups in `cmd/lazyclaude/root.go:105-225`):
- `notify-broker` is closed.
- `mcp-server` is stopped (5s timeout).
- `askpass-server` is stopped.
- `control-client` is closed.
- `gc.Stop` runs.

**No cleanup hook for in-flight `runMCPAsync` goroutines.** If the user quits while a remote SSH `Refresh` is in flight:
- The SSH command will run until the OS-level `exec.CommandContext` returns. `ConnectTimeout=10s` is set (`internal/daemon/ssh.go:51`), so the goroutine outlives the TUI by up to 10 seconds.
- The trailing `a.gui.Update(...)` call will run on a TUI that has already been closed; gocui's behavior here is undefined but the process is about to exit anyway, so practical impact is nil.

### 6.3 New BC drafted

### BC-MCPREG-022: `runMCPAsync` goroutines outlive TUI shutdown

**Preconditions:** A remote `Refresh` or `ToggleDenied` is in-flight when the user quits the TUI.
**Postconditions:** The goroutine is not cancelled; it completes (or times out at the 10s SSH ConnectTimeout) and attempts a `gui.Update` on the closed gocui instance.
**Evidence:** `internal/gui/app_actions.go:1128-1140` (hardcoded `context.Background()`); `cmd/lazyclaude/root.go:105-225` (no lifecycle registration for MCP async).
**Confidence:** HIGH for the code shape, LOW for any observed-failure record (no bug report cited in pass-A).

**Implication for monocle:** The Rust port should derive `runMCPAsync` contexts from a root app context that is cancelled at shutdown. Tokio `JoinHandle` + a shutdown broadcast channel is idiomatic. This is a **defensive improvement**, not a faithful port — the Go code's behavior is benign in practice but is a latent shutdown-correctness issue.

### 6.4 Adjacent observation: `loading` flag set without lock

The `a.mcpState.loading = true` write at `app_actions.go:1129` happens on the GUI goroutine (caller); the reset to `false` at line 1133 happens inside `gui.Update` callback. The render path at `internal/gui/render_mcp.go:18, 49` reads it without explicit locking. gocui's `gui.Update` provides goroutine-safety for the *callback*, but the initial `loading = true` write is on whatever goroutine invoked `runMCPAsync` — typically the GUI's keybinding goroutine. **In practice no race because keybindings and renders are both on the gocui event loop**, but the lack of explicit synchronization on `mcpState.loading` is a latent issue. P2-aesthetic.

---

## 7. Bonus finding (bench-warmer from following the trail) — `mcpAdapter.Servers()` slice aliasing

At `cmd/lazyclaude/root.go:758-773`, the adapter builds new `gui.MCPItem` rows but **aliases `Args`**:

```go
items[i] = gui.MCPItem{
    Name:    s.Name,
    Type:    s.Config.EffectiveType(),
    Scope:   s.Scope,
    Denied:  s.Denied,
    Command: s.Config.Command,
    Args:    s.Config.Args,   // <-- slice header copy, backing array shared
    URL:     s.Config.URL,
}
```

`s.Config.Args` is the same backing slice held by the manager's cached `MCPServer.Config.Args`. A GUI consumer that mutates `MCPItem.Args` would affect the manager cache. The GUI today does not mutate this field (it's a read-only render input — see `internal/gui/presentation/mcp.go` and `internal/gui/render_mcp.go:60`), but the contract is implicit and easy to break.

**Same applies to `Config.Env` and `Config.Headers`** (BC-MCPREG-019 from r1), and `MCPServer.Config` itself is a value-copy in the slice from `Manager.Servers()`, but the *maps within* `Config` share backing storage.

**P2-MCPREG-F:** Adapter-layer slice/map aliasing. Combined with BC-MCPREG-019. Defensible by interface convention ("the MCPItem slice is read-only"), but a Rust port using owned `Vec<String>`/`HashMap` would naturally fix this.

---

## 8. Updated test coverage assessment

After round 2, the open gaps from r1 §7 update as follows:

| Gap | Status after r2 | Action |
|---|---|---|
| Cross-scope name collision (P1-MCPREG-C) | unchanged — no new test discovered | retain as P1 |
| `updateDeniedInJSON` empty-existing branch | unchanged — only exercised on remote first-write | retain as P2 |
| `atomicWriteFile` failure paths | unchanged | retain as P2 |
| Concurrent `ToggleDenied` | unchanged | retain as P2 |
| `Servers()` returned-slice inner-map aliasing | extended — also affects `mcpAdapter.Servers()` (P2-MCPREG-F) | retain as P2 |
| `toggleDenied` duplicate-avoidance | unchanged | retain as P2 |
| Remote write non-atomicity (P1-MCPREG-A) | unchanged | retain as P1 |
| Non-atomic raw `Manager` setters | unchanged | retain as P2 |
| Malformed JSON in `parseDeniedServers` | unchanged | retain as P2 |
| `deniedEntry` schema fragility (P1-MCPREG-B) | **CONFIRMED via re-trace of `updateDeniedInJSON`** — re-marshals from typed slice, drops unknown fields | upgrade evidence quality; remains P1 |
| `runMCPAsync` shutdown cancellation | **NEW BC-MCPREG-022** | new P2 |
| `EnsureClaudeConfigured` silent-wipe-on-parse-failure | **NEW cross-subsystem finding** | new P1 |
| `~/.claude.json` cross-process partial-read race | **NEW** | new P2 |
| `mcpAdapter.Servers()` adapter-layer aliasing | **NEW (P2-MCPREG-F)** | new P2 |
| `mcpState.loading` flag synchronization | **NEW (P2-aesthetic)** | new P2 |

---

## 9. New BCs and findings produced in r2

### New BCs
- **BC-MCPREG-021** — `mcp.Manager` never writes `~/.claude.json`.
- **BC-MCPREG-022** — `runMCPAsync` goroutines outlive TUI shutdown (context.Background usage).

### New findings
- **P1-MCPREG-D (cross-subsystem):** `session.EnsureClaudeConfigured` silently restarts from empty map on parse failure, wiping the user's `mcpServers` and other Claude Code settings. Not in the mcp package itself but is the dominant runtime failure mode that affects what mcp.Manager reads. Should be tracked in monocle's spec because any port that includes onboarding must NOT replicate this anti-pattern.
- **P2-MCPREG-F:** Adapter-layer aliasing of `Args` slice and `Env`/`Headers` maps from `Manager.Servers()` through `mcpAdapter.Servers()` to `gui.MCPItem`. Defensive copying or owned-data types in the port would prevent.
- **P2-MCPREG-G:** `~/.claude.json` cross-process partial-read race (lazyclaude vs. external `claude` writer). Mitigation: retry on parse error.
- **P2-MCPREG-H:** `mcpState.loading` written outside `gui.Update` callback; safe in practice but unsynchronized.

### Refuted/confirmed prior speculation
- **CONFIRMED (P1-MCPREG-B):** `deniedEntry` schema fragility is real — `updateDeniedInJSON` re-marshals from the typed slice, so unknown per-entry fields would be lost. Confidence upgrade from r1.

---

## 10. Updated monocle relevance verdict

Verdict from r1 stands: **MEDIUM-HIGH retention, with split scope**.

Round 2 strengthens the recommendation:
- The "single consumer in cmd/lazyclaude/root.go" finding (§2) means a Rust port is a self-contained slice of ~641 LOC + 2 small wire-up files.
- The "disjoint from internal/server/" finding (§3) means MCPRegistry and MCPServer can be ported and tested independently.
- The "EnsureClaudeConfigured silent-wipe" finding (§4) is a **port-time improvement opportunity**, not a porting blocker — monocle should NOT replicate that anti-pattern.
- The "deniedEntry re-marshal drops unknown fields" finding (§5.3) is a **port-time correctness opportunity** — model as `[]serde_json::Value` for pass-through.

No new arguments to drop the priority. No new arguments to raise it.

---

## 11. Delta Summary

- New BCs in this round: 2 (BC-MCPREG-021, BC-MCPREG-022). Total BCs across both rounds: 22.
- New P1 findings: 1 (P1-MCPREG-D — cross-subsystem silent-wipe).
- New P2 findings: 3 (P2-MCPREG-F, -G, -H).
- Confirmation upgrades on prior findings: 1 (P1-MCPREG-B evidence quality raised from "by inspection of code" to "by re-trace of marshalling path").
- Items confirmed safe/disjoint: 2 (consumer-set bounded to root.go; internal/server/ disjoint).
- Items refined but not changed: schema completeness of ServerConfig (still MEDIUM confidence; matches Claude Code today).
- Gaps that remain genuinely open: 6 (down from 10 in r1).

## 12. Novelty Assessment

**Novelty: SUBSTANTIVE — but at the boundary.**

The cross-subsystem `EnsureClaudeConfigured` finding is genuinely new and changes how a porter should approach onboarding alongside the MCP registry — it would not have been visible from reading `internal/mcp/` alone. The `runMCPAsync`-shutdown finding affects the Rust port's task-orchestration design. The consumer-bound-to-root.go finding bounds the port surface and is decision-changing for whether to keep `internal/mcp/` as a separate crate or merge it into the GUI crate.

Two of the new findings (P2-MCPREG-G, -H) are arguably **refinement-grade** and on their own would be NITPICK. The substantive findings carry the round.

Would removing this round's findings change how I'd spec the system? **Yes** — the silent-wipe risk and the shutdown-cancellation gap would both need to be re-derived later. Worth keeping as a round.

## 13. Convergence Declaration

**Borderline — one more round recommended.**

Round 3 should target:
- Verify no other `os.WriteFile`/`os.Rename` paths target `<proj>/.claude/settings.local.json` from elsewhere in the repo (could be a hook injector — pass-8 mentions hooks-settings written to runtime dir, but a different file).
- Verify the `presentation/mcp.go` rendering doesn't mutate the items it receives (cross-check the adapter-aliasing P2-MCPREG-F).
- Sweep `internal/gui/` for any other path that calls `MCPProvider` methods we haven't catalogued, particularly any path that bypasses `runMCPAsync`.
- Examine the `gui_adapter.go` (mentioned in §5 of r1 follow-ups) for any additional indirection.
- Re-trace the `notifyBroker` and event paths to confirm the MCP tab does not subscribe to any broker stream (event-driven refresh would be a missing piece).

If round 3 finds only nitpicks, the subsystem is converged.

## 14. State Checkpoint

```yaml
pass: B
subsystem: internal/mcp
round: 2
status: complete
files_read:
  - internal/session/manager.go lines 180-225 (EnsureClaudeConfigured)
  - cmd/lazyclaude/root.go lines 80-130, 340-360, 745-805 (wire-up and adapters)
  - internal/gui/app_actions.go lines 1090-1140 (runMCPAsync)
  - internal/gui/mcp_state.go (interface)
  - internal/gui/render_mcp.go (renderer)
new_searches_performed:
  - "mcp.NewManager" / "internal/mcp" imports (consumers — found exactly 1)
  - "claude.json|.mcp.json|settings.local.json|deniedMcpServers|mcpServers" in internal/server/ (found 0)
  - "EnsureClaudeConfigured" call sites and definition (found 1 each)
  - "context.Background()" in runMCPAsync (confirmed)
  - "hasCompletedOnboarding|numStartups|hasTrustDialogAccepted|allowedTools" (schema fields touched by EnsureClaudeConfigured)
timestamp: 2026-05-11
novelty: SUBSTANTIVE
next_round_targets:
  - other settings.local.json writers across repo
  - presentation/mcp.go rendering pass (immutability check)
  - sweep gui/ for unmediated MCPProvider calls
  - gui_adapter.go indirection
  - broker/event subscription topology for MCP tab (any?)
```
