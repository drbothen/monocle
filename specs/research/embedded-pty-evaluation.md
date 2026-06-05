---
document_type: research
research_type: general
project: monocle
producer: research-agent
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
topic: "Embedding a live AI-harness terminal session (PTY) inside monocle's ratatui TUI"
inputs:
  - NEXT-SESSION-PIVOT.md
  - .factory/specs/architecture/SS-engine-module.md
  - .factory/specs/architecture/SS-deps-pin-manifest.md
  - .factory/semport/zellij/ (final synthesis + architecture passes)
  - .factory/semport/claude-squad/ (session-tmux-git + patterns-for-monocle passes)
traces_to: NEXT-SESSION-PIVOT.md §4 items 1-3 (D-236 control-center pivot)
---

# Research: Embedded PTY Terminal Inside monocle's ratatui TUI

> **SR-002 SUPERSESSION NOTE (2026-06-03T23:30:00Z — D-237 architecture delta):**
>
> This document was produced before the D-237 human ratification (2026-06-03) of the full
> keyboard fidelity scope. Two positions in this document are SUPERSEDED by the D-237
> ratification and the subsequent architecture delta (vision v2.1, ADR-0011, SS-09):
>
> 1. **§3.3 point 3 (keyboard scope narrow):** States "Phase-1-correct scope: cover the keys a
>    Claude Code session actually needs (printable, Enter, arrows, Ctrl-C, Ctrl-D, Backspace,
>    Tab, Esc) to production grade; defer exotic protocols to a later wave." This is SUPERSEDED.
>    D-237 human ratification (2026-06-03) placed full keyboard fidelity IN v1A scope: printable
>    + control + arrows + Backspace + Tab + Esc + Enter + **mouse events + Kitty keyboard protocol**.
>    Bracketed paste is also v1A. No deferral. See vision §Open Questions Q-6 (RESOLVED) and
>    SS-embedded-pty.md §Full-Fidelity Keyboard Encoding.
>
> 2. **§8 Q5 (keyboard fidelity open question):** The question "Which key/input protocols are in
>    v1 production scope?" is RESOLVED — full fidelity as described in point 1 above. The human
>    decision note "HUMAN DECISION NEEDED" was resolved at D-237. No further human input needed.
>
> All other analysis and recommendations in this document remain valid and inform the architecture
> delta. The PTY stack selection (portable-pty 0.9.0 + vt100 0.16.2 + tui-term 0.3.4) is
> confirmed by ADR-0011. The Option A UDS channel recommendation (§8 Q4) is confirmed by ADR-0010.
>
> Authoritative keyboard scope: SS-embedded-pty.md §Full-Fidelity Keyboard Encoding (v1A).
> Authoritative PTY stack: ADR-0011 + SS-deps-pin-manifest-v2-delta.md.
> Authoritative session persistence model: ADR-0009 + SS-session-manager.md.

> **SR-003 SUPERSESSION NOTE (2026-06-04 — S18-001 Phase-1d adversarial finding, ADR-0009):**
>
> This research evaluation was produced BEFORE the D-238 persistence escalation and the
> ratification of ADR-0009 (session-host-owns-PTY). One ownership-model position in this
> document is SUPERSEDED:
>
> **The "native in-process PTY embedding" / daemon-owned recommendation** (Executive Summary,
> §3.5 daemon-owned children, §7.1) assumed monocle-runtime (the in-process daemon) would own
> PTY masters and harness children. This model is SUPERSEDED by ADR-0009, which retired
> in-process/daemon-owned PTY in favor of **session-host-owns-PTY**: PTY masters and harness
> child processes live in detached, per-session `monocle-session-host` processes that survive
> TUI restarts independently. See ADR-0009 §Decision and the canonical vision §Process Topology.
>
> **What remains valid:** The PTY stack selection (portable-pty 0.9.0 + vt100 0.16.2 +
> tui-term 0.3.4) is confirmed by ADR-0011 and is unchanged. The multi-session architecture
> patterns (§6), compatibility analysis (§2), and approach trade-off table (§4.3) remain
> informative. Only the in-process OWNERSHIP assumption is superseded — not the crate stack.

## Executive Summary

The D-236 pivot requires that a user "never leave the TUI" while a running Claude Code
session is **visible and interactive inside** monocle. This is the single biggest new
unknown. After evaluating three approaches against monocle's actual stack (ratatui 0.30 +
crossterm 0.29 + tokio 1.52 + Rust 1.88 MSRV), the finding is decisive:

> **Primary recommendation: native in-process PTY embedding using `portable-pty` 0.9.0
> (spawn) + `vt100` 0.16.2 (parse) + `tui-term` 0.3.4 (render as a ratatui widget), with a
> blocking PTY-reader thread bridged into the tokio event loop over a bounded channel.**
> _(Stack confirmed; in-process OWNERSHIP superseded by ADR-0009 — see SR-003 above.)_

This stack is **version-verified compatible** with monocle's pinned ratatui 0.30: `tui-term`
0.3.4 depends on `ratatui-core ^0.1.0` and `ratatui-widgets ^0.3.0`, which are **exactly the
versions ratatui 0.30.0 itself depends on** (verified on deps.rs). They unify to a single
copy in the dependency graph — no version conflict, no duplicate-`Buffer`-type problem. All
three crates are **MIT-licensed** (Apache-2.0/MIT for the optional `vte` fallback), carry **no
RUSTSEC advisories**, and do **not raise the Phase-1 MSRV floor** (tui-term MSRV is 1.86 <
monocle's 1.88).

> **Runner-up: tmux control mode (`tmux -CC`) via a Rust control-mode client** (the
> conceptual successor to claude-squad's `capture-pane` scraping). Rejected as primary
> because it adds a hard external runtime dependency (tmux must be installed), inherits
> claude-squad's documented fragility (tmux-server crash makes the launcher un-launchable),
> and pushes terminal-emulation fidelity through a text-scraping seam. **It remains a
> legitimate strategic fallback** for the specific capability tmux gives nearly for free:
> session persistence across a monocle restart/crash (detach/reattach). See §6.

> **Zellij-as-library: rejected for vendoring/dependency; adopt as architecture model only.**
> Zellij is a binary, not a consumable library; its `zellij-server` crate is a tightly-coupled
> internal crate (async-std + crossbeam thread-bus, not designed for embedding). Its PTY/pane
> architecture is an excellent *pattern* (the internal-ANSI-parser-as-screen-state model is
> exactly what `vt100`+`tui-term` give monocle in miniature), but its code is not transferable.
> See §7.

**The trade-off that distinguishes primary from runner-up:** native PTY embedding gives
**full terminal fidelity with zero external runtime dependencies and a clean async seam**, at
the cost that **monocle must own session persistence itself** (a crash loses live sessions
unless monocle re-spawns or daemonizes the children). tmux gives **free detach/reattach
persistence** at the cost of an **external dependency and a scrape-based fidelity ceiling**.
Production-grade default favors owning the seam in-process (native PTY) and solving
persistence explicitly, rather than outsourcing the core capability to an external process
whose failure modes monocle cannot control.

---

## 1. Context & Constraints (from monocle's actual architecture)

| Constraint | Source | Implication for this research |
|------------|--------|-------------------------------|
| ratatui 0.30 (caret pin), crossterm 0.29 | SS-deps-pin-manifest.md Phase 1 Pin Manifest | Any PTY-render widget must be ratatui-0.30-compatible; this is the gating compatibility question. |
| tokio 1.52 (exact pin), async runtime | SS-deps-pin-manifest.md | PTY I/O must integrate with an existing tokio event loop without blocking it. |
| MSRV Phase 1 = Rust 1.88 | SS-deps-pin-manifest.md MSRV Policy | A new crate that requires > 1.88 would raise the floor — flag it. |
| `thiserror 2` (lib), `anyhow 1` (bin); bounded `mpsc` with drop counter; no unbounded channels | CLAUDE.md Conventions | PTY-output channel must be bounded with a surfaced drop counter, per the project anti-pattern policy. |
| `EngineModule` trait is the harness seam, NOT sealed | SS-engine-module.md §Purpose | The pivot's `launch`/`attach`/`detach`/`kill` + PTY-stream methods extend this trait surface (or a sibling). |
| Daemon owns sessions; today it only *receives* hooks, does not *spawn* them | NEXT-SESSION-PIVOT.md §3, §4.1 | The PTY-supervising component is a new "session manager" in/near `monocle-runtime`. |
| `ProcessSnapshot.exe_path` strict-basename detection already exists | SS-engine-module.md §Supporting Types | Spawned-by-monocle sessions are now first-class — detection logic must reconcile spawned vs. externally-launched. |

**Important provenance note:** the zellij ingest in `.factory/semport/` explicitly declared
**PTY internals out-of-scope** (`zellij-pass-8-final-synthesis.md` §Scope Statement lists
`pty.rs`, `pty_writer.rs`, `terminal_bytes.rs`, `os_input_output*.rs` as "mentioned only").
So the embedded-PTY question is genuinely **new research**, not recoverable from prior
ingest. The claude-squad ingest covered tmux/PTY at the *orchestration* level (Go `creack/pty`
via tmux), not the in-process Rust rendering level. This document fills that gap.

---

## 2. Library/Package Ecosystem Analysis (version-verified)

All versions verified against the crates.io REST API and deps.rs on **2026-06-03**.

| Crate | Latest stable | Released | License | RUSTSEC | MSRV | Role |
|-------|---------------|----------|---------|---------|------|------|
| `portable-pty` | **0.9.0** | 2025-02-11 | MIT | none found | not declared (compiles on 1.88) | Cross-platform PTY pair + child spawn (part of wezterm) |
| `vt100` | **0.16.2** | 2025-07-12 | MIT | none found | not declared | ANSI/VT100 parser → in-memory screen state (cursor, colors, scrollback, alt-screen) |
| `tui-term` | **0.3.4** | 2026-04-07 | MIT | none found | **1.86** | ratatui widget that renders a `vt100::Screen`; optional `portable-pty` integration behind `unstable` feature |
| `vte` | 0.15.0 | 2025-02-02 | Apache-2.0 OR MIT | none found | n/a | Lower-level ANSI parser (alacritty); **alternative** to vt100 if you build your own screen model |
| `ratatui-core` | 0.1.0 | 2025-12-26 | MIT | none found | — | The crate ratatui 0.30 + tui-term 0.3.4 **both** depend on (`^0.1.0`) |
| `ratatui-widgets` | 0.3.0 | 2025-12-26 | MIT | none found | — | The crate ratatui 0.30 + tui-term 0.3.4 **both** depend on (`^0.3.0`) |
| `ratatui` | **0.30.0** | 2025-12-26 | MIT | none found | — | monocle's pinned TUI framework |

### 2.1 The compatibility verdict (the critical finding)

The risk with any ratatui third-party widget is **two incompatible copies of `ratatui_core`
types** (e.g. `Buffer`, `Rect`, `Style`) in the dependency graph, which produces "expected
`Buffer`, found `Buffer`" errors. This is resolved here:

- **ratatui 0.30.0 depends on** `ratatui-core ^0.1.0` and `ratatui-widgets ^0.3.0` (verified
  on deps.rs/crate/ratatui/0.30.0).
- **tui-term 0.3.4 depends on** `ratatui-core ^0.1.0` and `ratatui-widgets ^0.3.0` (verified
  on deps.rs/crate/tui-term/0.3.4 and tui-term's Cargo.toml).
- These caret ranges **unify to a single resolved version** of each crate. tui-term 0.3.4's
  widget operates on the *same* `ratatui_core::buffer::Buffer` that monocle's ratatui 0.30
  render loop hands it. **No conflict.**

This is not a coincidence: tui-term was deliberately migrated to the ratatui-0.30 split-crate
model (the `ratatui-core`/`ratatui-widgets` split landed in ratatui 0.30.0, also dated
2025-12-26). tui-term 0.3.4 (2026-04-07) is the post-0.30 release line, actively maintained.

> **Caveat (medium confidence → verify at Cargo-init):** the precise wiring is confirmed at
> the *manifest* level. The production-grade step is to add `tui-term = { version = "0.3.4",
> features = ["unstable"] }` (the `unstable` feature gates the `portable-pty` helper) to a
> throwaway crate in the monocle workspace and run `cargo build` + `cargo tree -d` to confirm
> **zero duplicate** `ratatui-core`/`ratatui-widgets`/`vt100` versions before committing the
> dependency. This is a 10-minute spike, not an open architectural risk.

### 2.2 Dormancy / maintenance assessment

- **portable-pty 0.9.0** — actively used (6.7M+ downloads), maintained as part of wezterm by
  Wez Furlong. Last release Feb 2025. The wezterm project is a flagship Rust terminal
  emulator; portable-pty is its extracted, reusable PTY layer. **Not dormant.**
- **vt100 0.16.2** — earlier monocle ADR-era notes (and nucleo's ADR-0002 context) flagged
  several "dormant since 2024" crates in the ecosystem; **vt100 is NOT one of them.** It shipped
  a fresh 0.16.0/0.16.1/0.16.2 line in **July 2025** (verified crates.io). Maintainer Jesse
  Luehrs (doy). No "unmaintained" RUSTSEC advisory. **Actively maintained through mid-2025.**
- **tui-term 0.3.4** — the **most actively maintained** of the three: 0.3.2 (Mar 3 2026),
  0.3.3 (Mar 30 2026), 0.3.4 (Apr 7 2026). Listed in `ratatui/awesome-ratatui`. The README
  self-describes as "work in progress," which is a maturity caveat (see §3.3), not a
  maintenance-risk flag.
- **vte 0.15.0** — alacritty's parser, 56M+ downloads, the most battle-tested ANSI parser in
  Rust. Relevant only if monocle builds its own screen model instead of using vt100 (not
  recommended for Phase 1; see §3.4).

---

## 3. Approach A (PRIMARY): Native In-Process PTY Embedding

### 3.1 Architecture

```
                         monocle-runtime (session manager)
  ┌───────────────────────────────────────────────────────────────────┐
  │  portable-pty: openpty() → (master, slave); spawn `claude` on slave │
  │                                                                     │
  │   ┌─ blocking reader thread (std::thread or spawn_blocking) ──────┐ │
  │   │   loop { n = master_reader.read(&mut buf);                    │ │
  │   │         bounded_tx.try_send(Bytes::copy(&buf[..n]))  }        │ │
  │   └──────────────────────────────────────────────────────────────┘ │
  │                              │ bounded mpsc (drop-counter surfaced)  │
  └──────────────────────────────┼──────────────────────────────────────┘
                                 ▼
              monocle-tui (or runtime→TUI over existing UDS IPC)
  ┌───────────────────────────────────────────────────────────────────┐
  │  tokio::select! {                                                   │
  │    Some(bytes) = pty_rx.recv()  => parser.process(&bytes);          │
  │    Some(key)   = key_rx.recv()  => master_writer.write(encode(key));│
  │    Some(sz)    = resize_rx.recv()=> pty.resize(sz);                 │
  │                                     parser.set_size(rows, cols);    │
  │  }                                                                  │
  │  terminal.draw(|f| f.render_widget(                                 │
  │      PseudoTerminal::new(parser.screen()), pane_area));             │
  └───────────────────────────────────────────────────────────────────┘
```

### 3.2 Division of labor — what each crate gives you vs. what monocle implements

| Concern | Provided by | monocle must implement |
|---------|-------------|------------------------|
| PTY pair creation, child spawn, master read/write | `portable-pty` | Choosing the spawn `CommandBuilder` (binary, args, **cwd = git worktree**, **env = injected hook config**) |
| ANSI/VT100 parse → screen state (cursor, SGR colors, alt-screen, scrollback rows) | `vt100::Parser` | Feeding bytes; sizing the parser; choosing scrollback depth |
| Render screen state as a ratatui widget (colors → ratatui `Style`, cursor cell) | `tui-term::PseudoTerminal` | Placing the widget in the layout; focus/active-pane chrome |
| Keyboard → terminal bytes | — (you own this) | Translate crossterm `KeyEvent` → terminal byte sequences (incl. modified keys, arrows, fn-keys) and write to master |
| Resize / SIGWINCH propagation | `portable-pty` exposes `.resize(PtySize)`; `vt100` exposes `.set_size()` | Detect ratatui pane-area change, call **both** PTY resize and parser resize; debounce per claude-squad's 50ms pattern |
| Async bridge | — | Blocking reader thread + **bounded** channel (CLAUDE.md anti-pattern: no unbounded mpsc; surface drop counter in status bar) |
| Scrollback UI (keyboard scroll, search) | vt100 *tracks* history | Scroll keybindings + viewport offset; optional Nucleo search over `vt100::Screen` rows (monocle already pins nucleo 0.5) |
| Session persistence across monocle restart | — (native PTY does NOT survive parent death by default) | Explicit strategy: daemon owns children + survives TUI restart, OR re-spawn policy. **This is the primary cost of Approach A — see §3.5.** |

### 3.3 Maturity caveats (honest flags)

1. **tui-term self-describes as "work in progress."** It is the right widget and it tracks
   ratatui's bleeding edge faithfully, but monocle should expect to (a) pin it exactly and
   gate upgrades through review, and (b) be prepared to fork/vendor it if a needed feature
   (e.g. custom scrollback rendering) is missing. Its small surface (one widget) makes
   vendoring cheap if ever required. **Confidence: high that it works for the core case;
   medium that it covers every advanced rendering need without local patches.**
2. **The `portable-pty` integration in tui-term is behind the `unstable` feature flag.**
   The stable surface is "give me a `vt100::Screen`, I render it." Spawning is yours via
   `portable-pty` directly — which monocle wants anyway (you need full control of cwd/env/args
   for worktree + hook injection). Treat tui-term as the **renderer**, portable-pty as the
   **spawner**, vt100 as the **parser**. Do not couple to tui-term's `unstable` spawn helper.
3. **Keyboard encoding is genuinely monocle's responsibility and is non-trivial.** Full
   fidelity (Kitty keyboard protocol, modified keys, bracketed paste, mouse) is a body of work.
   Phase-1-correct scope: cover the keys a Claude Code session actually needs (printable, Enter,
   arrows, Ctrl-C, Ctrl-D, Backspace, Tab, Esc) to production grade; defer exotic protocols to
   a later wave **as a feature-ordering decision, not a shortcut** (per CLAUDE.md Rule 2).

### 3.4 Why vt100 over building on raw `vte`

`vte` (alacritty) is the lower-level parser; it gives you the escape-sequence *events* but not
a screen *model* — you'd build the grid, scrollback, SGR-attribute tracking, and alt-screen
buffer yourself. `vt100` is `vte` + the screen model already built, and `tui-term` renders
exactly `vt100`'s model. Building on raw `vte` is re-implementing zellij's internal terminal
emulator — large scope, no Phase-1 justification. **Use vt100.** Keep `vte` in mind only as the
escape hatch if vt100's screen model proves limiting (low probability).

### 3.5 The persistence gap (Approach A's real cost)

A child spawned on a PTY by monocle is a child of monocle. If monocle's TUI process exits, the
PTY master closes and the child typically receives SIGHUP. Approaches:

- **(Recommended) Daemon-owned children.** monocle already runs a daemon (`monocle-runtime`)
  that "actually serves" (NEXT-SESSION-PIVOT.md §3). Make the **daemon** the PTY supervisor;
  the TUI is a *client* that streams PTY output over the existing UDS IPC and sends keystrokes
  back. The daemon outliving the TUI gives detach/reattach for free **within monocle's own
  process model**, with no external dependency. This aligns perfectly with the existing
  client/daemon split and the zellij "single binary, two roles" pattern monocle already mirrors.
- **(Fallback) Re-spawn policy.** If the daemon dies, sessions are lost; monocle re-launches on
  next start. Acceptable only if paired with the daemon-owned model above as the primary.

This is the decisive design consequence: **Approach A makes monocle's daemon the session owner**
— which is exactly the inversion NEXT-SESSION-PIVOT.md §3 calls "the core of the change."

---

## 4. Approach B (RUNNER-UP): tmux Control Mode Embedding

### 4.1 What claude-squad actually does (gene-source ground truth)

Per `.factory/semport/claude-squad/claude-squad-pass-7-deep-session-tmux-git.md`, claude-squad:
- Spawns **one tmux session per `Instance`** in a dedicated **git worktree** under
  `~/.claude-squad/worktrees/<sanitized>_<unixnano>/`.
- Uses **`tmux capture-pane -p -e -J`** (visible) and `-S - -E -` (full scrollback) to scrape
  pane content for both change-detection (hashing) and preview rendering.
- Drives input via `tmux send-keys`; detects trust prompts by **hardcoded substring matching**
  (`"Do you trust the files in this folder?"` → TapEnter).
- Has a documented **fragility**: a tmux-server crash makes `cs` un-launchable until `cs reset`
  (because `Restore()` = `tmux attach-session` fails the whole load → `os.Exit(1)`).

This is **`capture-pane` scraping**, NOT `tmux -CC` control mode. Control mode is the *better*
version of the same idea.

### 4.2 tmux control mode (`tmux -CC`) — the upgrade

Control mode turns tmux into a machine-readable controller: tmux emits a line-based protocol
(pane output, create/resize/focus events) over stdout and accepts commands as protocol
messages over a **single persistent connection** — no per-command subprocess spawn. iTerm2 uses
this for its native tmux integration.

**Rust libraries that exist (low confidence on maturity — niche crates):**
- `tmux-cmc` — "tmux control mode client for Rust," bidirectional `-CC` control, persistent
  connection (GitHub: ArcavenAE/tmux-cmc).
- `par-term-tmux` — integrates tmux control mode with the `par-term` terminal emulator;
  attach to sessions, render panes natively, structured I/O (docs.rs/par-term-tmux).

> **Flag (inconclusive):** these control-mode crates are small/niche. I could not verify their
> production maturity, maintenance cadence, or RUSTSEC posture to the same confidence as
> portable-pty/vt100/tui-term. If Approach B were chosen, monocle would likely **implement the
> control-mode protocol parser itself** (it is a documented, stable line protocol) rather than
> depend on an unproven crate — which is real additional scope versus Approach A.

### 4.3 Trade-off table: native PTY (A) vs tmux control mode (B)

| Dimension | A: native portable-pty + vt100 + tui-term | B: tmux control mode |
|-----------|-------------------------------------------|----------------------|
| External runtime dependency | **None** (pure Rust, statically linked) | **tmux must be installed**, version-compatible, on every host (problematic on Windows / minimal containers) |
| Terminal fidelity | **Full** — monocle owns the vt100 screen model; renders every cell/color/cursor | High in control mode, but you render tmux's view; capture-pane variant is text-scrape (fidelity ceiling) |
| Async/tokio integration | Clean: blocking reader thread → bounded channel (the documented portable-pty pattern) | Single persistent control connection; must parse the control protocol; still bridge to tokio |
| Multi-session scaling | One PTY + one parser per session; cheap; switch = swap which parser the widget renders | Control mode: **one tmux server, one connection, many panes** — actually *good* at scale; capture-pane variant pays per-`tmux`-subprocess overhead (claude-squad's weakness) |
| Detach / reattach / persistence | **monocle must build it** (daemon-owned children — §3.5) | **Free** — tmux sessions persist across monocle restart; reattach via control connection. This is tmux's single strongest advantage. |
| Failure-mode ownership | monocle controls all failure modes in-process | Inherits tmux fragility (claude-squad: server crash → un-launchable); failure modes outside monocle's control |
| Trust/permission-prompt handling | monocle already has the **hook-based** permission overlay (the "killer scenario") — no scraping needed | claude-squad scrapes pane text for prompts (brittle, breaks on UX change). monocle's hook channel is strictly better than either. |
| Implementation scope | Moderate: 3 well-maintained crates + keyboard-encoding + persistence | Moderate-high: protocol client (build or trust niche crate) + tmux lifecycle mgmt + still need keyboard mapping |
| Conventions fit (CLAUDE.md) | Native; bounded channels, in-process, no shell-out | Adds subprocess + external-binary preflight; more anti-pattern surface |

### 4.4 When Approach B wins

If **session persistence across monocle crashes** becomes a hard, top-priority requirement and
the daemon-owned-children model (§3.5) proves insufficient, tmux's free detach/reattach is a
compelling reason to adopt it — **for that capability specifically**. The production-grade
posture is to design Approach A's daemon-ownership to deliver persistence first, and treat tmux
as a documented fallback if that proves inadequate, not to default to the external dependency.

---

## 5. Approach C (REJECTED as dependency): Reuse / vendor zellij

### 5.1 What zellij gives, and why it's not consumable

Per `.factory/semport/zellij/` synthesis and direct verification:
- zellij is **one binary, two roles (client/server), N WASM plugins** (zellij-pass-2).
- Its internal terminal-emulation model is **exactly the pattern monocle wants**: instantiate a
  PTY per pane, run an **internal ANSI parser** that holds screen state, re-serialize to the
  user's screen (verified in the zellij dev video transcript via search). `vt100` + `tui-term`
  is this same architecture in miniature.
- **But:** the `zellij` crate is "not a library" (docs.rs/crate/zellij — "is not a library").
  `zellij-server` *is* published but is a deeply internal crate: it uses **async-std**, a
  bespoke **crossbeam thread-bus** (`ThreadSenders`), `interprocess` UDS + length-prefixed
  protobuf, and a `wasmi` plugin host — none of it designed for embedding in a third-party app.
  Its PTY code (`os_input_output.rs`) uses `nix::pty::openpty` + `libc::login_tty` directly
  (verified via search) — i.e., it is essentially a hand-rolled portable-pty.

### 5.2 Verdict

- **Do NOT depend on or vendor zellij crates.** The coupling, the async-std/tokio mismatch with
  monocle's tokio stack, and the binary-not-library status make it a non-starter as a dependency.
- **DO adopt zellij as the architecture model** (the ingest already recommends this): the
  PTY-per-pane + internal-parser + render-from-screen-state pattern is precisely what Approach A
  implements with `portable-pty`/`vt100`/`tui-term`. monocle gets zellij's architecture without
  zellij's code or its async-std dependency.
- If monocle ever needs to emulate a PTY at a lower level than vt100 provides (it should not in
  Phase 1), zellij's `os_input_output.rs` is the **reference to read**, not the code to import.

---

## 6. Multi-Session / Multi-Project Scaling (all approaches)

The pivot requires "as many sessions and as many projects as we need" (NEXT-SESSION-PIVOT.md §1).

**Approach A model:** one `(portable-pty master, vt100::Parser, child handle)` triple per
session, owned by the daemon's session manager, keyed by the existing session-roster key
(`EngineModule::id()` + session id, per SS-engine-module.md). Switching the active pane = the
TUI renders a different parser's `screen()`; **all sessions keep parsing in the background**
(their reader threads keep draining into bounded channels regardless of which is focused — this
is the zellij "pipe the stream to part of the screen while you keep working elsewhere" property).

- **Fast switching:** O(1) — swap the rendered parser; no spawn/attach cost.
- **Kill:** drop the child handle (portable-pty kills the child); close the channel.
- **Resource use:** each vt100 parser holds a bounded scrollback grid (configurable rows ×
  cols × cell). Dozens of sessions are cheap; surface a per-session scrollback cap. The bounded
  output channels (CLAUDE.md mandate) bound memory under burst.
- **Grouping by project:** session manager already has `project_name` on `EnrichedSession`
  (SS-engine-module.md). Group the session list by it; pair with the claude-squad
  **worktree-per-task** pattern (its A.1 HIGH-value gene) so each project/session gets an
  isolated git worktree as cwd.

**Approach B model:** one tmux server, N sessions; control mode scales well (single connection,
all panes). capture-pane scales poorly (per-subprocess overhead — claude-squad's documented
weakness at frequent polling across many sessions).

**Recommendation for scaling:** Approach A with **daemon-owned session manager** — it reuses
the existing daemon, the existing roster keying, the existing `project_name` field, and the
existing bounded-channel conventions. No new external moving parts as session count grows.

---

## 7. Recommended Technical Decision

### 7.1 Primary

Adopt **`portable-pty` 0.9.0 + `vt100` 0.16.2 + `tui-term` 0.3.4** for in-process embedded PTY
rendering. Make the **daemon (`monocle-runtime`) the PTY/session owner**; the TUI is a client
that streams PTY output over the existing UDS IPC and forwards keystrokes back.
<!-- [SUPERSEDED by ADR-0009 — session-host-owns-PTY: PTY masters + harness children live in detached per-session monocle-session-host processes, not in monocle-runtime. Stack selection (portable-pty/vt100/tui-term) remains valid.] -->

Cargo.toml additions (to be ratified by architect; caret pins per Patch-Pinning Policy — none
of these are on the 9-crate exact-pin security list, all are MIT, none touch the untrusted-input
deserialization path):

```toml
# monocle-runtime (spawner/supervisor)
portable-pty = "0.9"
vt100        = "0.16"

# monocle-tui (renderer)
tui-term     = "0.3"   # renders vt100::Screen; do NOT enable `unstable` (spawn) feature —
                       # monocle spawns via portable-pty in the runtime, not via tui-term
vt100        = "0.16"  # TUI needs the Screen type for rendering
```

MSRV impact: **none** — tui-term MSRV 1.86 ≤ monocle's 1.88; portable-pty/vt100 compile on 1.88.
License impact: **none** — all MIT, compatible with monocle's stack.
RUSTSEC impact: **none found** as of 2026-06-03 (rustsec.org search; confirm in CI via
`cargo audit` on the resolved Cargo.lock — the project already mandates this).

### 7.2 Runner-up

**tmux control mode (`tmux -CC`)** — kept as the documented fallback if (a) cross-restart
session persistence proves a hard requirement that daemon-ownership cannot satisfy
production-grade, or (b) Windows-host support via native PTY proves harder than expected.
Adopt control mode, **not** claude-squad's capture-pane scraping (which the ingest's pattern
catalog already marks "Skip — S.3 tmux as primitive" and "S.2 hardcoded prompt strings").

### 7.3 The distinguishing trade-off (one sentence)

**Native PTY (primary) gives full fidelity and zero external dependencies but makes monocle
responsible for session persistence (solved via daemon-owned children); tmux (runner-up) gives
free detach/reattach persistence but at the cost of an external runtime dependency and a
fidelity/robustness ceiling monocle cannot fully control.**

### 7.4 Genes to carry forward regardless of approach (from claude-squad)

- **A.1 Worktree-per-task isolation (HIGH)** — each session's cwd is a dedicated git worktree
  under monocle's data dir; capture base-commit SHA for diff baseline; sanitize names; append
  `_<unixnano>`. Pairs with the spawn `CommandBuilder.cwd`.
- **A.5 Executor/PtyFactory seam (MEDIUM)** — wrap PTY/process spawn behind a small trait so
  tests inject mocks (monocle already does this kind of seam via DTU clone + TestBackend).
- **Replace claude-squad's brittle prompt-scraping with monocle's existing hook-based permission
  overlay** — monocle's killer scenario already solves the trust/permission problem structurally;
  do not scrape PTY text for prompts.

### 7.5 CI/CD integration notes

- Add `cargo audit` coverage for the three new crates (already mandated on every PR).
- The Cargo-init spike (§2.1 caveat) belongs in the devops/architect Cargo-workspace step:
  `cargo tree -d` must show **single** versions of `ratatui-core`, `ratatui-widgets`, `vt100`.
- PTY tests need a TTY; use a controlled spawn (e.g. spawn `cat` or a tiny echo binary on the
  PTY) in integration tests, asserting parser screen state — mirrors claude-squad's PtyFactory
  mock seam and monocle's existing TestBackend discipline.

---

## 8. Open Questions for the Architect / Human (cross-component decisions)

1. **Session ownership locus:** Confirm the daemon (`monocle-runtime`) owns PTYs and the TUI is a
   streaming client (recommended), versus the TUI owning PTYs directly. This is the central
   architectural decision and drives the persistence story (§3.5). It also determines whether PTY
   bytes traverse the existing UDS IPC (new high-throughput message type on the `monocle-proto`
   wire) or stay in-process.
2. **Trait surface:** Do `launch`/`attach`/`detach`/`kill`/`resize`/PTY-stream become methods on
   the `EngineModule` trait (SS-engine-module.md), or on a new sibling `SessionManager` /
   inherent `ClaudeCodeModule` methods (the spec already puts operational concerns like spawning
   on the struct, not the trait — §"Struct-level inherent operations")? Recommend: lifecycle on a
   new component, with `EngineModule` providing only the spawn *recipe* (binary/args/env). Needs
   architect adjudication (cross-component: core trait vs runtime component).
3. **Persistence requirement strength:** Is cross-monocle-restart session survival a hard v1
   requirement or a later-wave feature? The answer decides whether daemon-ownership must be
   crash-durable in the first control-center wave (pushing toward, in the limit, the tmux
   fallback) or whether re-spawn-on-restart is acceptable initially (feature-ordering decision
   per CLAUDE.md Rule 2 — human/architect call, not an AI deferral).
4. **PTY-bytes over IPC throughput:** If the daemon owns PTYs, PTY output must reach the TUI over
   IPC at terminal-refresh rates across N sessions. Does the existing UDS + bounded-channel design
   (and its drop-counter convention) hold under that load, or does the embedded terminal warrant a
   dedicated streaming path? Needs a perf-engineer benchmark (1000+ events/sec target already
   exists in conventions).
5. **Keyboard fidelity scope for v1:** Which key/input protocols are in v1 production scope
   (printable + control keys + arrows) vs. deferred (Kitty keyboard protocol, mouse, bracketed
   paste)? Feature-ordering call.
6. **Hook auto-injection on spawn:** NEXT-SESSION-PIVOT.md §4.6 wants monocle to inject the hook
   config when it launches a session (per-session settings/env/wrapper) instead of the manual
   copy step. The spawn `CommandBuilder` env is the natural injection point — confirm the
   mechanism (env var vs per-session settings file) with the product-owner/architect.
7. **tui-term fork posture:** Pre-accept that tui-term (self-labeled "work in progress") may need
   a local fork/vendor for advanced scrollback rendering. Acceptable given its tiny surface — but
   confirm the architect is comfortable depending on a WIP-labeled crate for a core capability,
   or wants a vendoring plan up front.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_ask | 4 | Embedded-PTY-in-ratatui architecture + async; tui-term↔ratatui-0.30 compatibility; tmux control-mode vs capture-pane trade-offs; RUSTSEC/license/maintenance status of the crate stack |
| Perplexity perplexity_search | 1 | Whether zellij is consumable as a library / its PTY pane architecture |
| WebFetch | 8 | crates.io API version verification (portable-pty, tui-term, vt100, vte, wezterm-pty[404], ratatui, ratatui-core); deps.rs (ratatui 0.30 → ratatui-core/ratatui-widgets); tui-term GitHub + raw Cargo.toml |
| Read (local) | 8 | NEXT-SESSION-PIVOT.md; SS-engine-module.md; SS-deps-pin-manifest.md; zellij pass-2 + pass-8; claude-squad pass-7-session-tmux-git + pass-7b6-patterns |
| Glob (local) | 3 | Locate gene-source synthesis files |
| Context7 | 0 | Not used — crates.io/deps.rs gave authoritative version+dependency data directly |
| Tavily | 0 | Not needed — Perplexity + direct registry fetches cross-validated each finding |
| Training data | 2 areas | General PTY/VT100 concepts and Rust async-bridge patterns — flagged explicitly; every load-bearing claim (versions, compatibility, advisories, licenses) verified against live registries, not training data |

**Total external tool calls:** 24 (4 ask + 1 search + 8 WebFetch + 8 Read + 3 Glob)
**Training data reliance:** **low** — all version numbers, the ratatui-0.30↔tui-term compatibility
verdict, RUSTSEC posture, licenses, and gene-source facts are sourced from live registries
(crates.io, deps.rs), rustsec.org, vendor docs, and the in-repo semport ingest. The one item
explicitly flagged **medium confidence** is the manifest-level compatibility resolution, with a
concrete Cargo-init spike prescribed to convert it to verified-at-build before committing the dep.

## Sources

- portable-pty — https://crates.io/crates/portable-pty , https://docs.rs/portable-pty , https://github.com/wezterm/wezterm
- vt100 — https://crates.io/crates/vt100
- tui-term — https://crates.io/crates/tui-term , https://github.com/a-kenji/tui-term , https://deps.rs/crate/tui-term/0.3.4
- vte — https://crates.io/crates/vte , https://github.com/alacritty/vte
- ratatui 0.30 — https://crates.io/crates/ratatui , https://deps.rs/crate/ratatui/0.30.0 , https://github.com/ratatui/ratatui/releases , https://docs.rs/ratatui/latest/ratatui/widgets/index.html
- ratatui-core / ratatui-widgets — https://crates.io/crates/ratatui-core , https://docs.rs/ratatui-widgets/=0.3.0-beta.0/
- PTY-reader blocking pattern — https://github.com/wezterm/wezterm/discussions/3739
- awesome-ratatui (third-party widgets) — https://github.com/ratatui/awesome-ratatui , https://ratatui.rs/showcase/third-party-widgets/
- claude-squad — https://github.com/smtg-ai/claude-squad ; using tmux with Claude Code — https://hboon.com/using-tmux-with-claude-code/
- tmux control mode Rust — https://github.com/ArcavenAE/tmux-cmc , https://docs.rs/par-term-tmux
- zellij — https://docs.rs/zellij-server/0.43.1/zellij_server/ , https://github.com/zellij-org/zellij/blob/main/zellij-server/src/os_input_output.rs , zellij dev video (PTY/multiplexer model)
- RUSTSEC advisory DB — https://rustsec.org , https://github.com/rustsec/advisory-db
- In-repo gene-source ingest — `.factory/semport/zellij/zellij-pass-8-final-synthesis.md`, `.factory/semport/zellij/zellij-pass-2-architecture.md`, `.factory/semport/claude-squad/claude-squad-pass-7-deep-session-tmux-git.md`, `.factory/semport/claude-squad/claude-squad-pass-7b6-patterns-for-monocle.md`
