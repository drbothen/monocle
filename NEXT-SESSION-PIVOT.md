# NEXT SESSION — START HERE: monocle pivots to a full Control Center

> Written 2026-06-03 at the end of a long session, by the outgoing context, for the
> incoming fresh-context session. The human deliberately asked to clear context and
> restart so the next session can plan this properly. **Read this whole file first,
> then CLAUDE.md, then `.factory/STATE.md`.**

---

## 1. THE DECISION (the human's words, verbatim intent)

> *"We need to be able to launch, manage, and observe — a better lazyclaude, a better
> claude-squad. We should never have to leave the TUI and we should be able to manage
> as many sessions and as many projects with sessions as we need to. We need to be able
> to run, launch, manage, observe, tune, control — everything from the TUI."*

monocle is **no longer observe-only.** It becomes the **control center** from which the
user does everything:

- **Launch** Claude (and future harness) sessions — spawn them *from* the TUI.
- **Manage** many concurrent sessions across **many projects** — switch, attach/detach, kill, rename, group by project.
- **Observe** — live session state, transcripts, event stream (what Phase 1 already does).
- **Tune** — customizations / bindings / profiles / model routing, interactively (not just browse).
- **Control** — permission prompts, keybindings, session lifecycle — all in-TUI.
- **Never leave the TUI** — the user lives inside monocle; the running session is *visible inside it* (embedded terminal / PTY), not in a separate window.

This **supersedes** the original "observe-only for state, action-only via overlays"
principle (vision synthesis v1.1.2, approved 2026-05-11). That principle explicitly
*rejected* launching/orchestration ("inherit PM/Worker orchestration — rejected";
"execute workflows — rejected — observe-only"). **Those rejections are now reversed.**

## 2. WHY (the gap that triggered this)

Phase 1 was built faithfully to the approved vision: a passive observer that sits over
your editor where **you** run Claude, receiving hook POSTs and overlaying permission
prompts. Functionally correct — but it's a *narrower* product than the reference repos
it was born from. The human realized: "we were building a better lazyclaude, not a
worse one." You cannot even launch Claude from inside monocle today. **None of the
roadmap Phases 2-4 fixed this** — they all extended observe-only. So this is a
vision-level course correction, not a missing phase.

## 3. CURRENT STATE — what is ALREADY BUILT (do NOT rebuild; most of it is reusable)

- **Repo:** `~/Dev/monocle`, branch `develop` @ `fcd42f04` (as of this session's end).
  `.factory/` is the factory-artifacts orphan-branch worktree.
- **9 workspace crates** (under `crates/`): `monocle-core`, `monocle-runtime` (daemon),
  `monocle-proto`, `monocle-test-harness`, `monocle` (CLI binary), `monocle-config`,
  `monocle-ipc`, `xtask`, `monocle-tui`.
- **The daemon now ACTUALLY SERVES** (fixed this session — it was a sleep-loop stub):
  `monocle daemon start` → binds an OS HTTP port (hook ingestion) + a UDS socket (TUI) +
  tracing + durable ring-flush shutdown; writes `monocle.lock`, `hooks-settings.json`,
  `monocle.sock` into `$MONOCLE_RUNTIME_DIR`. Generates the Claude Code hooks config.
  See `USING-WITH-CLAUDE-CODE.md` for the working flow. Verified end-to-end:
  hook POST → `{"decision":"allow"}` → event persisted to `monocle-events.jsonl`.
- **TUI** (`monocle-tui`): sessions panel, event ribbon (rolling hook-event log),
  **permission overlay** (the `Ctrl-\` popup — y/n/A resolve, the "killer scenario"),
  profile picker (Ctrl-P, sticky-per-project, CCR path), two-row status bar.
  Rendered via ratatui; tested via `TestBackend`.
- **Reusable seams already in place** (the architecture left forward-compat hooks):
  - `EngineModule` trait — abstracts "the harness" (Claude Code today, CodeMachine/others
    tomorrow). Runtime plane is named "live session state **and harness lifecycle**."
    **This is the seam to extend with launch/spawn/attach/kill lifecycle methods.**
  - `FactoryAdapter` trait (vsdd factory awareness), `monocle-proto` wire schemas
    (`HookEnvelope` + 5 events, `schema_version`), JSONL ring (`format_version`),
    `monocle-v1:` auth token format, daemon lock protocol.
  - DTU clone `dtu-claude-code-hooks-v1` (in `monocle-test-harness`) — behavioral clone
    of the Claude Code hook protocol for tests; fidelity 1.0.
- **Pipeline status:** VSDD Phase 3 complete, Phase 4 holdout PASSED (0.99). Phases 5-7
  (adversarial / hardening / convergence) were pending when we pivoted. Product was
  "Phase 1 (Runtime Core)" of a 4-phase plan; Phases 2-4 were roadmap.
- **The hard truth this exposes about the model:** the daemon currently only *receives*
  hooks from sessions the user launched elsewhere. It does **not own or spawn** any
  session. That inversion is the core of the change.

## 4. WHAT MUST CHANGE (the scope — to be detailed/architected next session)

This is **additive** to Phase 1, not a rewrite. The good parts (hook ingestion,
permission overlay, EngineModule/FactoryAdapter traits, proto, ring, TUI rendering)
stay. We add the control-center layer on top:

1. **Session lifecycle ownership.** monocle must *spawn and own* harness sessions —
   process/PTY management, one per (project, session). The daemon (or a new "session
   manager" component) supervises child processes. Study how **claude-squad** does this
   (it uses **tmux sessions + git worktrees** per session) and how **zellij** manages
   PTYs/panes (zellij is already a gene source — it's a terminal multiplexer; the
   "never leave the TUI with the session visible inside" requirement is essentially an
   embedded-multiplexer problem).
2. **`EngineModule` (and the daemon) gain lifecycle:** `launch(project, opts)`,
   `attach`/`detach`, `list`, `kill`, `rename`, status — and a way to stream the
   session's terminal output into the TUI (PTY → TUI pane).
3. **Embedded terminal in the TUI** — the user watches/interacts with the running Claude
   session *inside* monocle (a PTY pane), not a separate terminal. This is the biggest
   new TUI capability. (zellij / a PTY widget — research the right Rust approach:
   `portable-pty`, `vt100`/`tui-term`, or embedding via tmux control mode.)
4. **Multi-session, multi-project management UX:** a session list/switcher grouped by
   project; create/launch/kill from the TUI; fast switching; the existing sessions panel
   evolves into a real session manager.
5. **"Tune/control" surfaces:** the Static plane (customizations / 5-level binding
   precedence) becomes **interactive** — edit/apply bindings, profiles, model routing
   (CCR is already integrated for profiles) from the TUI, not just browse.
6. **Hook wiring becomes automatic.** Today the user manually copies `hooks-settings.json`
   into `~/.claude/settings.json`. If monocle *launches* the session, it should inject
   the hook config itself (per-session settings, env, or a wrapper) — no manual step.
7. **Re-evaluate "auto-start daemon on first TUI launch"** (already specced as
   `MONOCLE_NO_AUTOSTART`) — in the control-center model the TUI is the entry point.

## 5. GENE SOURCES TO RE-STUDY (through the NEW launcher/manager lens)

The original vision did a "disposition pass" over 8 ingested reference repos and chose
to *leave behind* the launcher/orchestration genes. **That disposition must be redone.**
The repos are in `.factory/semport/` (final-synthesis files). Re-study especially:
- **claude-squad** — the closest prior art: launches & manages multiple Claude sessions
  (tmux + git worktrees). This is the primary gene source for the new direction.
- **zellij** — terminal multiplexer / PTY & pane management → the "embedded session in
  the TUI" model.
- **lazyclaude variants** (`any-context-lazyclaude`, `nikiforovall-lazyclaude`) — the
  "lazy*" launch+manage UX.
- **lazygit** — the interaction-philosophy gene (already heavily used).
- **claude-code-router (CCR)** — model/profile routing (already integrated; central to
  "tune").
- **codemachine-cli** — second harness (validates the multi-harness `EngineModule`).
- **vsdd-factory** — factory awareness (workflow plane).

## 6. METHODOLOGY for the next session

This is a **major vision revision** → it runs through the VSDD pipeline again as a delta:
1. **Facilitate a revised vision** with the human (this file is the seed). Capture
   launch/manage/observe/tune/control as first-class capabilities; explicitly retire the
   observe-only constraint; define the multi-session/multi-project model and the
   embedded-PTY requirement.
2. **Redo the disposition pass** on the gene sources (claude-squad, zellij first).
3. **Brief delta → architecture delta → stories** (the orchestrator + specialist agents,
   same as Phase 1). Decide: is this "Phase 1.5 / a new Runtime-plane capability set," or
   a re-baselined v1? Likely the latter given the scope.
4. **Preserve & extend** the working Phase-1 substrate; don't discard it.

## 7. KEY ARTIFACTS / POINTERS

- `CLAUDE.md` (repo root) — project operating instructions, agent routing, production-grade principle.
- `.factory/STATE.md` — live pipeline state (will carry a pointer to this file).
- `.factory/specs/research/domain-monocle-vision-synthesis.md` — the ORIGINAL (observe-only) vision to be revised. Read it to see exactly what was chosen and rejected.
- `.factory/specs/product-brief.md` — Phase 1-4 plan (observe-only).
- `.factory/specs/architecture/` — SS-* subsystem specs, ADRs, `SS-engine-module.md` (the trait to extend), `SS-daemon-wiring-impl.md` (how the daemon now serves), `SS-deps-pin-manifest.md`.
- `.factory/semport/` — the 8 ingested gene-source repos' synthesis.
- `USING-WITH-CLAUDE-CODE.md` (repo root) — current working flow (observe-only).
- The orchestrator agent (`vsdd-factory:orchestrator`) drives the pipeline; specialists do the writing.

## 8. FIRST COMMANDS FOR THE NEW SESSION

1. Read this file, then `CLAUDE.md`, then `.factory/STATE.md` (look for the pivot pointer + `next_session_resume_protocol`).
2. Skim `.factory/specs/research/domain-monocle-vision-synthesis.md` (what observe-only chose/rejected) and `.factory/semport/` for **claude-squad** and **zellij**.
3. Do NOT resume VSDD Phase 5 (adversarial refinement of the old scope) — the scope is changing. Confirm with the human, then start the **vision-revision** facilitation for the control-center direction.
4. Reuse, don't rebuild: the daemon, hook ingestion, permission overlay, EngineModule/FactoryAdapter, proto, ring, and TUI rendering are assets.

---

**One-line summary for the next session:** monocle is becoming a full TUI control center
(launch + manage + observe + tune + control; many sessions, many projects; never leave the
TUI — a better lazyclaude AND claude-squad). The observe-only constraint is retired. Phase 1
substrate is built and reusable. Start by revising the vision with the human, redoing the
gene-source disposition (claude-squad + zellij first), then deltaing brief → architecture → stories.
