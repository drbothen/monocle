---
document_type: gene-source-disposition
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
gene_source: zellij
disposition_pass: v2 (D-236 control-center pivot)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md §5, embedded-pty-evaluation.md §5 (Approach C), §7
---

# Gene-Source Disposition v2: zellij (Control-Center Lens)

## Vision Lens Applied

monocle v1 (re-baselined): full TUI control center — never leave the TUI; running session
visible and interactive inside monocle (embedded PTY pane). Daemon owns PTYs.
Session persistence is a hard v1 requirement.

## Original Disposition (to be selectively reversed)

The v1.1.2 vision-synthesis §Explicit Non-Goals stated:
- "Does NOT replace the terminal multiplexer — monocle runs inside tmux, does not replace it;
  zellij's multiplexer internals are a Leave-behind gene."

The original Pass-8 synthesis recommended:
- Inherit: crate split (zellij-utils/client/server/tile pattern), IPC model (length-prefixed
  protobuf + UDS), plugin SDK (WASM + single host import), session persistence (two-file per
  session, dirty detection), config hot-reload, theme system, per-client config overlay.
- Leave behind: PTY internals (out of scope for the original ingest), modal keymap (14 InputMode
  variants — monocle uses single-mode), multiplexer role itself.

The pivot reverses the "PTY internals are leave-behind" stance. The EMBEDDED PTY requirement
makes zellij's internal PTY/pane architecture the most relevant pattern.

## Disposition by Capability Area

### 1. PTY/Pane Architecture (originally out-of-scope for ingest, now PRIMARY)

**Original verdict:** PTY internals declared out-of-scope in the ingest (`pty.rs`,
`pty_writer.rs`, `terminal_bytes.rs`, `os_input_output*.rs` listed as "mentioned only").
Not dispositioned — simply left aside as irrelevant to observe-only monocle.

**REVERSED by pivot. New verdict: MODEL (architecture pattern, not code).**

The embedded-pty-evaluation.md §5 has already delivered the specific verdict:
- zellij's PTY/pane architecture (PTY-per-pane + internal ANSI parser as screen state +
  re-serialize to user terminal) is EXACTLY the pattern monocle adopts.
- But zellij code is NOT consumable as a library: async-std + crossbeam thread-bus +
  interprocess UDS + wasmi plugin host; binary not library; tightly coupled internal crate.
- The implementation monocle uses: `portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4`
  (the zellij architecture in miniature, with monocle's tokio stack).

What monocle models from zellij's PTY architecture:
- **One PTY pair + one parser per pane/session.** Each session in monocle's daemon has its own
  `(portable-pty master, vt100::Parser, child handle)` triple.
- **Internal ANSI parser holds screen state.** vt100::Parser is monocle's equivalent of
  zellij's internal terminal emulator. The parser is NOT zellij's code — it is a separate crate
  that implements the same conceptual role.
- **Render from screen state.** tui-term renders `vt100::Screen` to a ratatui Buffer — exactly
  analogous to zellij serializing its internal screen model back to the user's terminal.
- **Background reading keeps all sessions fresh.** zellij "pipes the stream to part of the
  screen while you keep working elsewhere" — monocle replicates this by keeping reader threads
  alive for ALL sessions (not just the focused one), so switching sessions is O(1) (swap which
  parser the widget renders).
- **Resize propagation.** zellij sends SIGWINCH-equivalent through its server bus; monocle
  sends PTY resize via `portable-pty.resize(PtySize)` AND parser resize via `vt100::Parser.
  set_size()` on pane-area change detection.

### 2. Client/Server IPC Model (originally ADOPT, now ENHANCE)

**Original verdict: ADOPT** — already adopted in Phase 1. UDS + length-prefixed protobuf,
monocle-proto, monocle-ipc crate. This is fully built.

**New verdict: ENHANCE** — the control-center pivot adds a new high-throughput message type.

In the control-center model, PTY bytes must flow from the daemon to the TUI over the existing
UDS IPC at terminal-refresh rates across N sessions. This is a new, high-volume message type
that the original design did not include.

Implications:
- Add `PtyOutput { session_id, bytes: Bytes }` to the `ServerToClientMsg` protobuf enum in
  `monocle-proto`. This is the primary new wire message type for the embedded terminal.
- Add `KeyInput { session_id, bytes: Bytes }` to `ClientToServerMsg` for keystroke forwarding.
- Add `ResizePane { session_id, rows: u16, cols: u16 }` to `ClientToServerMsg`.
- The existing bounded-channel + drop-counter architecture (from monocle-ipc) handles this
  naturally — PTY output is just a high-frequency byte stream over the existing channel.
- zellij's `ThreadSenders` actor mesh pattern (already modeled in monocle's broker) remains the
  right IPC model. No new IPC mechanism is needed; the existing design extends cleanly.

**Open question for architect/human:** What is the bounded channel size for PTY output messages?
The existing 1000 events/sec target (from conventions) may need a separate per-session PTY-byte
channel to avoid starving hook-event messages. This is an architecture adjudication question
(cross-component: IPC throughput vs. event-broker separation). Flagged for vision/architecture
revision.

### 3. WASM Plugin SDK for Factory Adapters (originally ADOPT, now CONFIRMED)

**Original verdict: ADOPT** — recommended for factory-adapter extensibility.

**New verdict: ADOPT (confirmed, scope clarified).**

The control-center pivot does not change the WASM plugin SDK recommendation for factory
adapters. However, the Phase-3 WASM SDK (`monocle-plugin-sdk`) is in the suspended Phase 4-7
scope. In the re-baselined v1 scope:
- WASM SDK for EngineModule extensions and FactoryAdapters: DEFER to Phase 2 (post-control-
  center launch capability).
- The zellij WASM model remains the correct architectural reference when that phase ships.
- Built-in adapters (VsddFactoryAdapter, ClaudeCodeModule) do not need WASM — they are native
  code. WASM is for third-party extensibility.

### 4. Session Persistence / Resurrection (originally ADOPT, now ENHANCE)

**Original verdict: ADOPT** — two-file per-session directory, is_dirty gate, background I/O.

**New verdict: ADOPT + EXTEND — persistence now covers PTY ownership, not just metadata.**

In the observe-only model, "session persistence" meant persisting monocle's knowledge of a
session (metadata). In the control-center model, it means persisting the RUNNING SESSION itself
across monocle TUI restarts (daemon survives TUI exits).

The zellij session-persistence model maps as follows:
- `~/.cache/zellij/contract_version_1/session_info/<session_name>/session-layout.kdl` →
  `~/.local/share/monocle/v1/session_info/<session_id>/session-state.json` (daemon-format JSON
  with PTY size, cwd, harness binary, args, hook-config hash, started_at).
- zellij's "dirty detection" (pane count changed?) → monocle's "session changed?" (new
  process started or stopped?).
- zellij's two-file approach: monocle uses one session-state.json (simpler; no layout
  complexity — monocle has a fixed single-pane layout per session).
- The 5-thread save-chain for offloading I/O from the render path: monocle uses the daemon's
  background task thread (already exists) to persist session state asynchronously.
- Resume on restart: if the daemon restarts and finds a `session-state.json` with a running PID
  that is still alive (via kill(pid, 0)), it reattaches. If PID is dead, the session is "orphaned"
  and shown as Terminated in the session list. User can re-launch from the session list.

This is a significant capability expansion from the original ADOPT. The architecture is clean
because the daemon is already a long-lived process that survives TUI restarts.

### 5. Configuration Model (originally ADOPT, now CONFIRMED with scope note)

**Original verdict: ADOPT** — layered loading, hot-reload via PollWatcher, source-span errors.

**New verdict: ADOPT (confirmed). Scope note: KDL is deferred.**

monocle uses JSON for config (per existing SS-deps-pin-manifest.md and brief v1.4). The
zellij KDL recommendation is acknowledged as superior for human editing but the project has
committed to JSON. The hot-reload and layered-merge patterns remain ADOPT regardless of format.

The control-center pivot adds new config surface:
- Per-session launch defaults (which EngineModule, worktree policy, initial prompt).
- These integrate into the existing `~/.monocle/config.json` structure as a new section.
- No new config format decision needed.

### 6. Theme System (originally MODEL, status unchanged)

**Original verdict: MODEL** — semantic token model is the right approach.

**New verdict: MODEL (unchanged).** The control-center pivot does not change the theme
recommendation. If monocle ships a theme system, it follows the 84-color semantic-token model.
This is deferred from the re-baselined v1 scope (not a v1 launch requirement).

### 7. Modal Keymap (14 InputMode variants) (originally Leave-behind)

**Original verdict: Leave behind** — monocle uses single-mode keybinding (not modal).

**New verdict: LEAVE BEHIND (confirmed).** The control-center pivot adds new key actions
(session launch, PTY input forwarding) but does NOT require modal keymaps. monocle's 5-level
binding precedence (lazygit gene) handles all key dispatch. The embedded terminal pane has a
special "pass-through mode" (all keys forwarded to PTY master) that is conceptually similar to
zellij's Locked mode, but implemented as an AppMode variant rather than a full modal keymap.

### 8. Multiplexer Role (originally Leave-behind)

**Original verdict: Leave behind** — monocle runs inside tmux, does not replace it.

**PARTIALLY REVERSED by pivot.**

In the control-center model, monocle IS a multiplexer — it manages multiple terminal sessions.
However, it does NOT replace the user's existing terminal multiplexer (tmux/iTerm2). The
user invokes monocle from within their existing terminal environment; monocle then manages AI
coding sessions internally via its daemon.

The key distinction:
- monocle IS a multiplexer for AI coding sessions.
- monocle does NOT multiplex the user's general terminal windows.
- The "monocle runs inside tmux" architecture from the original vision is RETAINED. monocle
  is still the `Ctrl-\` popup in the user's terminal. What changes is that monocle now ALSO
  spawns and owns PTYs for AI sessions internally, visible in the embedded terminal pane.

### 9. NOT Consumable as Library (confirmed)

**Original verdict:** zellij binary, not consumable as library.

**CONFIRMED.** The embedded-pty-evaluation.md §5 reaffirms this with additional detail:
async-std + crossbeam + wasmi, none designed for embedding. monocle does not attempt to
import zellij crates.

### 10. Per-Client Config Overlay (originally ADOPT)

**Original verdict: ADOPT** — `SessionConfiguration { runtime_config: HashMap<ClientId, Config>, saved_config: Config }` overlay model.

**New verdict: ADOPT + EXTEND** — in the control-center model, per-session launch parameters
extend this model. Each session can have its own active model profile, CCR routing config, and
hook injection parameters. The per-session config is the `runtime_config` in the overlay model.

## Summary Table

| Capability | Original Verdict | New Verdict | Change? |
|-----------|-----------------|-------------|---------|
| PTY/pane architecture (internals) | Out-of-scope (not dispositioned) | MODEL (architecture only; code via portable-pty/vt100/tui-term) | REVERSED from irrelevant |
| Client/server IPC model | ADOPT (built) | ENHANCE (add PTY byte message types) | Extended |
| WASM plugin SDK | ADOPT | ADOPT (confirmed; WASM deferred post-v1-launch) | Confirmed/scoped |
| Session persistence | ADOPT | ADOPT + EXTEND (daemon-owned PTY survival) | Extended |
| Configuration model | ADOPT | ADOPT (confirmed; KDL deferred) | Confirmed |
| Theme system | MODEL | MODEL (deferred from v1) | Confirmed/scoped |
| Modal keymap (14 InputMode) | LEAVE BEHIND | LEAVE BEHIND (PTY pass-through as AppMode variant) | Confirmed |
| Multiplexer role | LEAVE BEHIND | PARTIALLY REVERSED (monocle IS an AI-session mux; does NOT replace terminal mux) | Partially reversed |
| zellij as library | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| Per-client config overlay | ADOPT | ADOPT + EXTEND (per-session params) | Extended |

## Net Assessment

zellij is the ARCHITECTURAL MODEL for the control-center's embedded PTY design. The pivot
makes zellij more relevant, not less — specifically for the PTY/pane architecture that was
previously left out of scope.

The key insight from zellij for the control-center:
**PTY-per-session + internal ANSI parser holding screen state + render from screen state**
is the exact architecture monocle implements with `portable-pty + vt100 + tui-term`.
monocle inherits zellij's architecture without zellij's code.

The critical distinction from the original disposition: "zellij as library" remains
LEAVE BEHIND. "zellij as architectural model" is now more central than before.

## Cross-Reference

See `embedded-pty-evaluation.md` for the full technical validation of this disposition,
including version-verified crate compatibility (portable-pty 0.9.0 + vt100 0.16.2 +
tui-term 0.3.4), ratatui 0.30 compatibility proof, and the daemon-owned PTY architecture
diagram.
