# Pass B Deep — `internal/mcp/` — Round 3

**Subsystem:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/mcp/`
**Round:** 3 (convergence sweep — r2's targets)
**Prior rounds:** `*-deep-mcp-r1.md`, `*-deep-mcp-r2.md` (this directory)
**Timestamp:** 2026-05-11

---

## 1. Round 3 scope: gaps named in r2 §13

1. Verify no other writers of `<proj>/.claude/settings.local.json` from elsewhere in the repo.
2. Verify `presentation/mcp.go` rendering doesn't mutate items.
3. Sweep `internal/gui/` for any `MCPProvider` call that bypasses `runMCPAsync`.
4. Examine `gui_adapter.go` for additional indirection.
5. Re-trace broker / event subscription topology for the MCP tab.

All five are addressed in §§2-6 below. Verdict: **CONVERGED**.

---

## 2. Settings.local.json writers across the repo (Gap r2-1)

**Method:** Searched all `.go` files for `settings.local.json`.

**Result:** All hits are inside `internal/mcp/` or its tests. No other subsystem reads or writes `<proj>/.claude/settings.local.json`. The hook injection path (mentioned in pass-8 line 26 and CLAUDE.md "Hook injection" section) writes a DIFFERENT file: `<runtime-dir>/hooks-settings.json` passed via `claude --settings <file>`, never modifying `~/.claude/settings.json` or `<proj>/.claude/settings.local.json`. Confirmed at `cmd/lazyclaude/setup.go:47-48`:

```go
// Hooks are now injected at session startup via `claude --settings`,
// so there is no need to modify ~/.claude/settings.json here.
```

**Implication:** `internal/mcp/` is the SOLE writer of `<proj>/.claude/settings.local.json` from lazyclaude. Claude Code itself also writes this file (for `permissions`, `model`, etc.) — that's the cross-tool coexistence concern documented in BC-MCPREG-005 — but no other lazyclaude subsystem competes.

**Confidence:** HIGH. **Refinement only.**

---

## 3. `presentation/mcp.go` rendering immutability (Gap r2-2)

**Method:** Read all 73 LOC of `internal/gui/presentation/mcp.go`.

**Result:** Two formatting functions:
- `FormatMCPLine(name, serverType, scope, denied, maxWidth) string` — takes value types and a primitive `int`. Returns a `string`. No mutation possible.
- `FormatMCPPreview(name, serverType, scope, denied, command, args, url) string` — same shape, plus `args []string`. The function only reads `args` (`strings.Join(args, " ")` at line 62). No mutation.

**Conclusion:** The presentation layer is **read-only** with respect to its inputs. The adapter-aliasing concern P2-MCPREG-F from r2 §7 is not realised by the presentation layer; it is a latent risk only if a future consumer is added.

**Confidence:** HIGH. **Refinement only.**

---

## 4. Unmediated `MCPProvider` calls in `internal/gui/` (Gap r2-3)

**Method:** Searched all `.go` files in `internal/gui/` for `a.mcpServers.` and `mcpServers.` patterns.

**Result:** 16 call sites, all accounted for:

| File:line | Method | Mediation |
|---|---|---|
| `app_actions.go:161` | `SetRemote` | direct (setter, not async-wrapped — fast lock acquisition only) |
| `app_actions.go:163` | `Refresh` | wrapped in `runMCPAsync` |
| `app_actions.go:224` | `SetRemote` | direct setter |
| `app_actions.go:245` | `Refresh` | wrapped in `runMCPAsync` |
| `app_actions.go:288` | `SetRemote` | direct setter |
| `app_actions.go:290` | `Refresh` | wrapped in `runMCPAsync` |
| `app_actions.go:321` | `SetRemote` | direct setter |
| `app_actions.go:338` | `Refresh` | wrapped in `runMCPAsync` |
| `app_actions.go:1115` | `ToggleDenied` | wrapped in `runMCPAsync` |
| `app_actions.go:1124` | `Refresh` | wrapped in `runMCPAsync` |
| `search.go:362` | `Servers` | direct (read-only, must be fast — runs on render path) |
| `app.go:413` | `SetMCP` (wire-up) | called once at startup |
| `render_mcp.go:12, 49` | nil-check only | no call |
| `app_actions.go:149, 156, 183, 223, 242, 267, 320, 335, 1106, 1120, 1143` | nil-check only | no call |

**Two patterns of un-`runMCPAsync`-wrapped calls:**
- **`SetRemote`** is a pure in-memory setter that acquires `mu.Lock` momentarily. It is intentionally synchronous because the `runMCPAsync` `Refresh` that immediately follows must see the new (host, projectDir) pair. Wrapping it in async would re-introduce the race window that motivated `SetRemote`'s existence.
- **`Servers()`** is a synchronous getter returning a slice copy. It runs on the render goroutine (`renderMCPList`, `renderMCPPreview`, `filteredMCPServers`). The internal `mu.RLock` is held briefly. Render must be synchronous for gocui.

Both patterns are correct and intentional. **No bypass risk.**

**Confidence:** HIGH. **Refinement only.**

---

## 5. `cmd/lazyclaude/gui_adapter.go` (Gap r2-4)

**Method:** Searched `gui_adapter.go` for any reference to `mcp` or `MCP`.

**Result:** Zero hits relevant to MCP. The `gui_adapter.go` file provides a `localProjectsProvider` adapter for the project tree (line 162: `projects := a.localMgr.Projects()`), unrelated to MCP. The MCP adapter (`mcpAdapter`) lives in `cmd/lazyclaude/root.go:745-777` and was already examined in r1 §10 and r2 §7.

**No additional indirection found.**

**Confidence:** HIGH. **Refinement only.**

---

## 6. Broker / event subscription topology for the MCP tab (Gap r2-5)

**Method:** Searched `internal/gui/` for `broker.Subscribe` and any MCP-tab-side subscription pattern.

**Result:** Exactly one `broker.Subscribe` in the GUI: `internal/gui/notify_loop.go:44` (`nl.brokerSub = broker.Subscribe(8)`). This is the **notify broker** subscription for hook events and popup delivery — NOT a subscription on the MCP registry files.

The MCP tab is **pull-only**. Triggers for `Refresh`/`ToggleDenied`:
- Explicit user keybind (`MCPRefresh` action, bound to 'r' in the plugins-tab keymap, `internal/gui/keymap/registry.go:474`).
- `MCPToggleDenied` keybind, which calls `ToggleDenied` and internally re-refreshes.
- `syncPluginProject` (triggered by tree-cursor moves, see `app_actions.go:171-340`) on selection change.
- The initial "no-sessions-yet" fallback path at `app_actions.go:148-167`.

There is **no inotify / fsnotify / SSH-side file watcher** for `~/.claude.json`, `.mcp.json`, or `settings.local.json`. If Claude Code modifies these files while the MCP tab is open, the displayed state stays stale until the user re-navigates or presses 'r'.

This was implicit in the architecture but not stated as a behavioral contract. New draft below.

### BC-MCPREG-023: MCP tab is pull-only; no event-driven refresh

**Preconditions:** The MCP tab is rendered. An external process (`claude`, the user with a text editor, another `lazyclaude` instance) modifies `~/.claude.json`, `<proj>/.mcp.json`, or `<proj>/.claude/settings.local.json`.
**Postconditions:** The TUI continues to display the cached state. No automatic re-read occurs. Refresh requires user action: keybind 'r' (MCPRefresh), tree navigation triggering `syncPluginProject`, or a deny toggle.
**Evidence:** `internal/gui/notify_loop.go:44` is the only `broker.Subscribe` in the GUI; no subscription is mediated by the MCP tab. No file-watch code path exists in `internal/mcp/` or `internal/gui/`. `mcp_state.go:31-37` has no event channel field.
**Confidence:** HIGH (proof by absence; the negative is verifiable).
**Monocle implication:** A Rust port that adds an fsnotify-based reactive refresh would be a **deliberate enhancement**, not a faithful port. If desired, it must reach across the SSH boundary (remote files), which is non-trivial — the current pull design avoids that complexity entirely. Recommend keeping pull-only for v1.

---

## 7. Final gap reconciliation

The 10 open gaps from r1, after r2 progress, after r3:

| Gap | r1 status | r2 status | r3 status |
|---|---|---|---|
| Cross-scope name collision (P1-MCPREG-C) | open | open | open — design decision for monocle, not a test gap |
| `updateDeniedInJSON` empty-existing branch | open | open | open — test gap, P2 |
| `atomicWriteFile` failure paths | open | open | open — test gap, P2 |
| Concurrent `ToggleDenied` | open | open | open — test gap, P2 |
| `Servers()` slice-inner-map aliasing | open | extended | open — defended by interface convention |
| `toggleDenied` duplicate-avoidance | open | open | open — test gap, P2 |
| Remote write non-atomicity (P1-MCPREG-A) | open | open | open — design decision for monocle |
| Non-atomic raw `Manager` setters | open | open | open — design decision (unexport in port) |
| Malformed JSON in `parseDeniedServers` (local) | open | open | open — test gap, P2 |
| `deniedEntry` schema fragility (P1-MCPREG-B) | open | confirmed | open — design decision for monocle |
| `runMCPAsync` shutdown (BC-MCPREG-022) | n/a | new | open — port-time fix |
| `EnsureClaudeConfigured` silent-wipe (P1-MCPREG-D) | n/a | new | open — cross-subsystem |
| `~/.claude.json` cross-process partial-read | n/a | new | open — accept-and-retry |
| Adapter slice aliasing (P2-MCPREG-F) | n/a | new | confirmed inert in current consumers |
| `mcpState.loading` synchronization (P2-MCPREG-H) | n/a | new | confirmed safe under gocui event-loop |
| Pull-only refresh model (BC-MCPREG-023) | n/a | n/a | new in r3 — design contract, not a defect |

**Reading the table:** Every remaining open item is now either (a) a test gap that wouldn't change spec, (b) a design decision the monocle port can make freely, or (c) a contract that's been documented as a BC. There are no "we don't know what the code does here" items left.

---

## 8. Delta Summary

- **New BCs in this round:** 1 (BC-MCPREG-023). Total across all rounds: 23.
- **New findings:** 0 of any P-tier. All r3 work was confirmation or refinement.
- **Items confirmed inert / safe:** 5 (sole settings.local.json writer; presentation layer immutable; no GUI bypass of MCPProvider; no gui_adapter indirection; no broker subscription on MCP files).
- **Items converted from "unknown" to "design decision":** all remaining opens.

## 9. Novelty Assessment

**Novelty: NITPICK**

Justification — would removing this round's findings change how I'd spec the system?
- **§§2-5 are confirmations** of structural facts (no other writers, no aliasing in practice, no bypass paths, no missing indirection). These bound the analysis and let me declare convergence with confidence, but they do not change the model.
- **§6 / BC-MCPREG-023 (pull-only refresh)** is a documentation of an absent behavior. Useful as a contract for a porter, but the absence was already implicit in the architecture (no broker subscription = no event-driven refresh). A porter reading r1+r2 would not have been surprised.

The substantive surface is fully covered by r1 (the 20 BCs + structural map + SSH command shapes + locking discipline) and r2 (the cross-subsystem EnsureClaudeConfigured discovery + shutdown cancellation + adapter aliasing). r3 closes the loop without changing the model.

## 10. Convergence Declaration

**Pass B `internal/mcp/` has converged — findings are nitpicks, not gaps.**

The subsystem now has:
- 23 drafted BCs (BC-MCPREG-001 through BC-MCPREG-023). Pass A had 0.
- 2 P0 findings (terminology in brief; host-capture pattern load-bearing).
- 4 P1 findings (remote write non-atomicity; deniedEntry schema fragility; cross-scope name collision; EnsureClaudeConfigured silent-wipe).
- 8 P2 findings.
- Verified coverage of: file-format contracts (5.1-5.6), state model (§3 r1), concurrency model (§8 r1), test coverage map (§7 r1 + §8 r2), monocle relevance (§10 r1).
- Verified isolation: zero other consumers, zero cross-subsystem file conflicts beyond EnsureClaudeConfigured.

Three rounds is appropriate for a 641-LOC production / 1,067-LOC test subsystem with a single consumer and no broker integration. Further rounds would consume tokens to produce micro-refinements.

## 11. State Checkpoint

```yaml
pass: B
subsystem: internal/mcp
round: 3
status: complete
files_read:
  - internal/gui/presentation/mcp.go (73 LOC — full read)
  - internal/gui/notify_loop.go (broker subscription site, line 44)
  - cmd/lazyclaude/setup.go lines 45-50 (hooks-not-claude-settings confirmation)
new_searches_performed:
  - "settings.local.json" repo-wide (writers — found only internal/mcp/)
  - "a.mcpServers." in internal/gui/ (16 sites — all accounted for)
  - "mcp|MCP" in cmd/lazyclaude/gui_adapter.go (0 hits — no indirection)
  - "broker.Subscribe" in internal/gui/ (1 site, unrelated to MCP tab)
  - "MCPRefresh" call sites (keybind + tests only)
total_bcs_across_rounds: 23
timestamp: 2026-05-11
novelty: NITPICK
convergence: declared
next_round_targets: none — pass converged
```
