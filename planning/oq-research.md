---
document_type: open-questions-research
level: pre-architecture
version: "1.0"
status: draft
producer: research-agent
phase: pre-phase-1-architecture
timestamp: 2026-05-12T14:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md
  - /Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/lazygit/lazygit-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md
project: monocle
brief_version: "1.1"
target_skill: /vsdd-factory:create-architecture
---

# Open Questions Research — Monocle v1 Architecture Inputs

## Document scope and method

This research document resolves the 11 open questions (OQ-01..OQ-11) raised by the
monocle product brief v1.1 (`/Users/jmagady/Dev/monocle/.factory/specs/product-brief.md`,
lines 273-285). Each question receives a Recommended Default with rationale tied to
monocle's specific constraints (multi-harness, factory-aware, observe-only,
lazygit-philosophy, Ctrl-\ popup over an editor). The Architect should consume this
document as the primary input for spec crystallization and `/vsdd-factory:create-architecture`.

Method per OQ:
1. State the design tension in 2-3 sentences.
2. Enumerate concrete options.
3. Cite prior art from gene-source synthesis files and external Rust ecosystem.
4. Estimate Phase 1 implementation cost.
5. List operational implications (failure modes, observability, UX).
6. Pick one Recommended Default with confidence rating.
7. Define a concrete architect action.

Source verification rule: every crate version claim has been verified against
the live `crates.io` REST API as of 2026-05-12. Every behavioral claim that ties
to a gene-source repo has a file:line citation that resolves on disk.

## OQ-01: Does monocle daemon start auto-run on first TUI launch, or require explicit invocation?

### Trade-off summary

The trade-off pits user-facing simplicity against daemon-lifecycle visibility. Auto-start
hides the daemon process from a developer who just wants to "press Ctrl-\ and see my
sessions"; explicit start makes the lifecycle visible and forces the user to think
about background processes (which is honest engineering but bad onboarding UX for a
popup-over-editor tool).

### Options under evaluation

- **Option A — auto-start with handshake.** TUI checks for live daemon (lock file + PID
  liveness probe), spawns daemon as detached child process if absent, waits on a
  startup-notify channel until the daemon binds its socket, then connects.
- **Option B — explicit start.** TUI prints a helpful error ("monocle daemon is not
  running; run `monocle daemon start` first") and exits with code 75. User wires daemon
  into their shell init or systemd-user / launchd unit.
- **Option C — hybrid.** Auto-start by default; honor `MONOCLE_NO_AUTOSTART=1` env
  var to fall back to explicit mode for power users and CI.

### Prior art evidence

- **sccache** (Mozilla) implements auto-spawn with `SCCACHE_STARTUP_NOTIFY` socket-path
  notification pattern. The client spawns the server process, sets the env var with a
  socket path, listens for the server to connect and report startup status. Default
  listen address `127.0.0.1:4226`. Sources:
  `https://github.com/mozilla/sccache`,
  `https://docs.rs/sccache/latest/src/sccache/commands.rs.html`. The hardened pattern
  in production for ~10 years.
- **zellij** is the opposite — `zellij` first invocation does NOT auto-spawn a
  background daemon in any traditional sense. Instead, the binary forks and runs both
  client and server inside its own process tree, with the server backgrounded.
  Subsequent `zellij attach` invocations connect to the same server. Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:91-93`
  ("zellij runs as one binary, two roles, N plugins in one process. The client/server
  split is a logical-process boundary within a single OS process most of the time").
- **any-context/lazyclaude** has a daemon subsystem with explicit `lazyclaude daemon
  start`. The brief notes "lazyclaude daemon stop referenced but not in CLI inventory"
  (P1-005,
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:293`)
  — even an explicit-start design ships with operational gaps.
- **lazygit / claude-squad** — single-process TUIs with no daemon, so not directly
  comparable.
- **tmuxinator** — explicit; user runs `tmuxinator start project`. No background daemon.
- **mprocs / helix** — no daemon model.
- **rust-analyzer** — auto-spawned by editor LSP client; pattern is "client owns
  lifecycle" via stdio pipe. Different topology (one daemon per editor instance, not
  one daemon per machine).

### Implementation cost

- **Option A (auto-start):** ~150-300 LOC for the spawn-and-handshake helper. Requires:
  (1) lock-file probe with `kill(pid, 0)` PID-liveness check (the same logic OQ-04
  needs); (2) detached child spawn — `std::process::Command::new(current_exe()).arg("daemon")
  .arg("start").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())` plus
  `setsid` on Unix (via `nix::unistd::setsid` or `daemonize` crate); (3) startup-notify
  channel — an ephemeral Unix socket path passed via env var that the daemon connects
  back to after binding its real socket. Crate options: `daemonize 0.5` (sync, no tokio
  conflict if called pre-runtime), or roll our own with `nix` + tokio.
- **Option B (explicit):** ~30 LOC for the missing-daemon error path. Plus operational
  cost: documentation for systemd-user / launchd setup; ~200 LOC of unit files and
  install logic if monocle wants to ship a `monocle install service` helper.
- **Option C (hybrid):** Option A + 5 LOC env-var check. Negligible additional cost.

### Operational implications

- **Auto-start (A or C)** introduces an observability gap: the user does not know the
  daemon exists, so does not know to look at its logs. Mitigation: monocle TUI status
  bar must show daemon PID, uptime, and (if not already running) the path to the daemon
  log file. The any-context synthesis confirms this:
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:148`
  ("CompositeProvider routes per-session ops"; the daemon-side complexity is real).
- **Explicit (B)** introduces an onboarding cliff: first-time users will hit the
  fail-fast error and abandon the tool unless the error message is exceptionally good
  ("Run `monocle daemon start` once. Add this to your shell init: ..." with copy-paste
  shell snippet).
- **Failure modes** to handle in either case: (1) stale lock file with dead PID — must
  unlink and proceed; (2) port already in use — must surface the conflict cleanly
  (relates to OQ-04); (3) daemon crash during handshake — must time out the connect
  attempt at ~3 seconds and report the failure path.
- **Killer scenario** (vision §End-to-End Killer Scenario, brief lines 58-63) requires
  the TUI to be reachable in ≤4 keystrokes. If the daemon is not running and explicit-
  start is required, that 4-keystroke target is impossible to hit consistently.

### Recommended default

**Option C (hybrid: auto-start by default, MONOCLE_NO_AUTOSTART=1 escape hatch).**

Rationale: the killer scenario demands sub-second TUI launch from a cold shell — the
developer presses Ctrl-\ and expects the popup *now*. Auto-start with a startup-notify
handshake (the sccache pattern) achieves this. The env-var escape hatch covers CI
environments and power users running monocle under systemd-user / launchd who want
explicit lifecycle control. Implementing auto-start now also forces the architecture
to confront the lock-file PID-liveness pattern (OQ-04, OQ-10) up front rather than as
a Phase 2 retrofit.

### Confidence

**HIGH.** sccache has run this pattern in production for a decade. The implementation
ingredients (PID liveness, detached spawn, startup notify) are all available in
mainstream Rust crates and are exercised by the any-context hook-protocol discovery
pattern that monocle already inherits.

### Architect action

Define a `monocle-runtime::daemon::auto_start` module that exports
`fn ensure_daemon_running(config: &Config) -> Result<DaemonHandle>`. The function
reads the lock file (per OQ-04 outcome), probes PID liveness with `nix::unistd::Pid::kill(0)`,
spawns `Command::new(current_exe()).args(["daemon", "start", "--notify-socket", &path])`
as detached, and waits on the notify socket with a 3-second tokio timeout. Honor
`MONOCLE_NO_AUTOSTART=1` by returning a structured error with the suggested command.
Surface the spawn in tracing logs at INFO level so the user can correlate.

---

## OQ-02: Hook tmpfile per-session or shared per-runtimeDir?

### Trade-off summary

Claude Code reads hook configuration from a settings file path passed via `--settings
<path>`. If two concurrent Claude Code processes share one settings file, they share
the same hook script targets (the same daemon URL and token); if they each get their
own settings file, the daemon has to manage N tmpfiles instead of one. The any-context
v2 synthesis pins this to shared-per-runtimeDir at
`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:154`
(`<runtimeDir>/hooks-settings.json` mode 0o600 written by `WriteHooksSettingsFile`).
Monocle has to decide whether to inherit that pattern verbatim or to swing per-session.

### Options under evaluation

- **Option A — shared per-runtimeDir.** One `<runtimeDir>/monocle/hooks-settings.json`
  file at mode 0o600. All `claude --settings <path>` invocations get the same path.
  Hooks point at the daemon's HTTP endpoint (per OQ-04). Identical to any-context.
- **Option B — per-session.** Each session gets a unique temp file under
  `<runtimeDir>/monocle/sessions/<session-id>/hooks-settings.json`. Allows per-session
  hook overrides (e.g. "this session has a custom PostToolUse hook for logging
  to a remote service"). Cleanup on session end.
- **Option C — hybrid.** Default to shared. Allow per-session override via config
  (`harness.profile.hook_overrides` field).

### Prior art evidence

- **any-context/lazyclaude — shared per-runtimeDir, verified.** Path:
  `<runtimeDir>/hooks-settings.json` mode `0o600`. Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:154`
  (`internal/core/config/hooks.go:49-75`, `WriteHooksSettingsFile`). The file uses
  `SetEscapeHTML(false)` to preserve `=>` arrow-function literals
  (BC-HOOK-008,
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:154`).
- **Claude Code `--settings` precedence (documented).** Per Anthropic's public docs
  `https://code.claude.com/docs/en/settings`, the precedence order from lowest to
  highest is: User > Project > Local > `--settings` flag > Managed. The `--settings`
  flag is "high precedence; below only Managed" and "accepts absolute paths or
  ~/-prefixed paths". Critically: `--settings` is "not accepted from project or
  local settings, since a cloned repository could supply either file to redirect
  memory writes to sensitive locations". Multiple concurrent processes with different
  `--settings` paths is a documented use case.
- **Race condition risk.** A separate ecosystem bug shows that Claude Code's underlying
  state files (claude-task-master) had no file locking, causing corruption with
  concurrent sessions
  (`https://github.com/eyaltoledano/claude-task-master/issues/1567`). Monocle's
  hooks-settings.json must not exhibit similar races. Mitigation: write via
  `tempfile::persist` atomic rename (brief line 99 mandate).
- **Concurrent `--settings` is safe by design.** Each Claude Code subprocess gets its
  own `argv[]`; the OS gives them isolated file handles to the settings file on read.
  Settings are loaded once at startup and not re-read on every tool invocation (per
  the precedence docs). So shared-per-runtimeDir does not exhibit a read-side race —
  the only race is write-side, which monocle controls via atomic-replace.

### Implementation cost

- **Option A (shared):** ~50-80 LOC. Single helper that writes the JSON via
  `tempfile::persist`. One path constant per runtime dir.
- **Option B (per-session):** ~200-300 LOC. Adds: per-session tmpfile lifecycle
  (create on session detect, delete on session end), per-session path map in the
  registry, cleanup on daemon shutdown (RAII), garbage collection for orphaned
  files. Plus integration test cost for the lifecycle.
- **Option C (hybrid):** ~100 LOC. Defaults to A; per-session is a config-driven
  override path triggered only when the profile schema declares it.

### Operational implications

- **Shared file (A)** is restart-resilient: the daemon writes the file once at startup,
  and on restart writes a new file (which atomic-replaces the old). Concurrent Claude
  Code processes that started with the old file keep using the old (cached at startup)
  daemon URL/token until they restart — which is the lock-file-discovery problem from
  OQ-04 in different clothing. The any-context fix is that hooks NEVER cache env vars
  and read the lock file every time
  (`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:184`,
  BC-HOOK-001). Monocle inherits this.
- **Per-session (B)** decouples hook routing from daemon restart but adds an
  observability surface: when a hook payload arrives, the daemon must trace it back
  to which session-specific file targeted it. Possible but adds cognitive load.
- **Hybrid (C)** preserves the shared-default simplicity. The override path activates
  only when explicitly configured, which is Phase 2+ territory.
- **Security:** mode `0o600` is non-negotiable — the hook file contains the daemon
  auth token, which is also the lock-file token (OQ-04 dependency).
  any-context-lazyclaude verifies this at
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:24`
  (`lock.go:56`, `lock_test.go:242-254` `TestLockManager_FilePermissions`).

### Recommended default

**Option A (shared per-runtimeDir, mode 0o600, atomic-replace).**

Rationale: the only Phase 1 use case is "hooks POST to one daemon". Per-session hook
files are speculative Phase 2 fodder (custom hook scripts per session profile, which
is not in the brief's Phase 1 scope). Shared-per-runtimeDir matches the any-context
pattern verbatim, which already has 41 behavioral contracts (BC-HOOK-001..041) and
gap-fill round verification. Doing anything else here is invented-here syndrome.

### Confidence

**HIGH.** Verified at file-line precision against any-context's hooks-r1 deepening
round
(`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:152-174`).

### Architect action

In `monocle-config::hook_settings`, define a single function
`fn write_hook_settings(runtime_dir: &Path, port: u16, token: &str) -> Result<PathBuf>`.
The function constructs the JSON via the canonical schema from
any-context-lazyclaude-pass-8-final-synthesis-v2.md §"Hook protocol", writes via
`tempfile::persist` for atomicity, and sets mode `0o600` after rename. Return the
absolute path so the caller can pass it to `claude --settings <path>` in profile
launches. Add an integration test using
`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md`
schema as the fixture.

---

## OQ-03: Does v1 ship the WASM monocle-plugin-sdk crate, or bundle VsddFactoryAdapter statically?

### Trade-off summary

WASM ABI gives third-party adapters from day one and aligns with zellij's gene
(`/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:138-186`).
But wasmtime 44 adds ~2.1MB minimum to the binary (verified against
`https://docs.wasmtime.dev/examples-minimal.html`) and requires Rust 1.92 MSRV (per
crates.io API check), which pushes monocle's whole workspace MSRV forward by years.
Statically bundling `VsddFactoryAdapter` keeps the v1 binary lean and defers the
WASM cost to Phase 3 where it natively belongs per the brief's phase plan
(brief lines 122-126, Phase 3).

### Options under evaluation

- **Option A — ship WASM SDK in v1.** Adopt wasmtime 44 + `monocle-plugin-sdk` crate.
  Third-party `EngineModule` and `FactoryAdapter` implementations work day one.
  Binary cost ~2.1MB; MSRV jumps to 1.92.
- **Option B — static bundle for v1.** Ship `ClaudeCodeModule` and `VsddFactoryAdapter`
  as native Rust impls. Phase 3 introduces `monocle-plugin-sdk` (per brief lines
  122-126, Phase 3 already calls for it). Phase 1 architecture must define the trait
  shapes in `monocle-core` so the Phase 3 WASM ABI is forward-compatible.
- **Option C — defer wasmtime, ship trait-object plugin shim now.** Phase 1 supports
  in-process Rust plugins via `inventory` or `linkme` static-registration. Phase 3
  adds wasmtime as a second loader. Allows non-WASM plugins in v1 without binary
  bloat. (`inventory` crate has near-zero footprint.)

### Prior art evidence

- **zellij ships zellij-tile + zellij-tile-utils** — but those are SDK crates for
  third parties to compile *their* WASM plugins. zellij's binary always includes the
  wasmi host runtime. Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:32-46`
  (LOC table: zellij-tile is 3,889 LOC; zellij-server is 142,410 LOC which includes
  the wasmi host). zellij has accepted the binary-size cost because plugins are core
  to its UX (`status-bar`, `tab-bar`, `compact-bar`, `session-manager` are all WASM).
- **wasmtime 44.0.1 MSRV is 1.92.0** — verified via
  `https://crates.io/api/v1/crates/wasmtime/44.0.1` 2026-05-12. By contrast,
  ratatui 0.30 MSRV is 1.86, axum 0.8.9 MSRV is 1.80, tokio 1.52 MSRV is 1.71. The
  workspace MSRV floor is determined by the highest pinned crate — adopting wasmtime
  44 forces MSRV to 1.92.
- **Wasmtime binary-size data:** minimum embedding `libwasmtime.so` is 2.1MB
  (`https://docs.wasmtime.dev/examples-minimal.html`). Cranelift can be disabled
  (the cargo features `cranelift` and `winch`) at the cost of requiring pre-compiled
  `*.cwasm` artifacts shipped alongside. Disabling cranelift is appropriate only if
  monocle ships AOT-compiled plugin artifacts — which is incompatible with the
  third-party-extensibility goal. Source:
  `https://github.com/bytecodealliance/wasmtime/blob/main/docs/examples-minimal.md`.
- **Wasmtime instantiation cost:** With pre-compiled (serialized) modules cached
  to disk and mmap-loaded, instantiation is dominated by deserialize time which is
  in the microseconds range for small modules. Source:
  `https://github.com/bytecodealliance/wasmtime/blob/main/docs/examples-pre-compiling-wasm.md`
  ("Faster start up: Compilation is removed from the critical path"). Vision's
  ~100ms target for TUI launch is well within this budget *if* modules are
  pre-compiled. Cold compilation of a non-trivial WASM module via cranelift is
  in the tens-of-milliseconds-to-seconds range and is unsuitable for first-launch.
- **codemachine-cli pattern (native plugin):** EngineModule is a TypeScript interface
  with 7 implementations as project folders, no WASM. Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md:18`
  ("7 of them (OpenCode, Claude Code, Codex, Cursor, Mistral, Auggie, CCR). Each is
  ~600–1000 LOC of provider folder with identical layout"). Demonstrates that
  native-language plugins scale to ~10 harnesses without needing WASM isolation.

### Implementation cost

- **Option A (WASM v1):** ~2,000-3,000 LOC for `monocle-plugin-sdk` (host loader,
  permission gate, WASI mount setup, protobuf-over-stdout shim — verbatim zellij
  pattern). Plus binary size +2.1MB. MSRV cost: 1.92 forces the entire workspace
  to Rust 1.92, which limits CI matrix to recent toolchains and constrains
  contributor onboarding.
- **Option B (static v1):** ~400-600 LOC for `VsddFactoryAdapter` (native Rust
  reading `.factory/STATE.md` per
  `/Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md:315-365`).
  Phase 3 WASM ABI is ~2,000 LOC additional then. Total LOC ~ same as A but spread
  over phases.
- **Option C (static-registration v1):** ~200 LOC for `inventory`-based plugin
  registration. Phase 3 adds wasmtime as a *second* loader. Plus interface design
  cost: the trait must work both via vtables and via WASM protobuf ABI, which
  constrains the trait to "no Rust generics, no associated types, async via Future
  not async fn in trait". This is the same constraint zellij imposes, so the
  precedent is healthy.

### Operational implications

- **A (WASM v1):** Phase 1 binary is heavier. Permission-gate UX has to ship in v1
  (the 17-permission model from zellij) — a major UI surface in a Phase 1 release.
  This contradicts the "lazygit philosophy" of minimal v1 surface.
- **B (static v1):** Cleanest Phase 1 binary. Risk: trait shapes defined in
  `monocle-core` may need breaking changes when WASM ABI is added in Phase 3 if
  the architect did not account for ABI constraints up front. Mitigation: require
  the Phase 1 trait to follow zellij's "no generics, no associated types"
  constraint set out the gate.
- **C (static-registration v1):** Allows distribution of third-party Rust crates
  that register adapters at compile time (each user rebuilds monocle with their
  custom adapter via cargo features or workspace member). Operationally clean for
  internal use; less polished UX than drop-in WASM plugins.

### Recommended default

**Option B (static bundle v1, WASM SDK lands in Phase 3 verbatim per brief).**

Rationale: the brief already puts `monocle-plugin-sdk` in Phase 3 (brief lines
122-126). The brief's Open Question is whether to *accelerate* WASM into Phase 1.
Strong NO for three reasons: (1) wasmtime 44 forces MSRV to 1.92 which is harsh for
the whole stack; (2) the Phase 1 killer scenario is "session management + permission
prompt dispatch" — neither uses a plugin; (3) the brief's Phase 3 already
acknowledges WASM. Monocle's vision (§Phase Plan, lines 376-382) is the same.
The architect should design the `EngineModule` and `FactoryAdapter` traits in
`monocle-core` with zellij's ABI constraints (no generics, no associated types,
protobuf-friendly types) so the Phase 3 WASM SDK is a non-breaking extension.

### Confidence

**HIGH.** The brief explicitly defers `monocle-plugin-sdk` to Phase 3. Wasmtime
MSRV 1.92 is mechanically verified. Binary-size 2.1MB is documented in wasmtime
official docs. The architectural insight (trait shape constrained at v1 to be
WASM-portable in v3) is the only contribution this OQ needs.

### Architect action

In `monocle-core::plugin`, define `EngineModule` and `FactoryAdapter` traits with
these constraints declared in module rustdoc:
1. No generic methods (all methods take concrete types, return
   `Box<dyn Future<Output = ...> + Send>` not `impl Future`).
2. No associated types beyond a static `&'static str` ID.
3. Method arguments are protobuf-encoded types from `monocle-proto` if cross-phase
   stability is needed (or `serde`-derivable structs in `monocle-core` for v1-only).
4. Permission requirements declared as a static method
   `fn required_permissions(&self) -> &'static [Permission]`.
The Phase 1 deliverable is `ClaudeCodeModule` and `VsddFactoryAdapter` as
in-binary native Rust impls. The Phase 3 WASM SDK becomes a second `dyn` source.

---

## OQ-04: Where does the daemon's HTTP server bind — 127.0.0.1:2748 (fixed) or 127.0.0.1:0 (OS-assigned)?

### Trade-off summary

Fixed port (2748) is simpler — hooks know where to POST. OS-assigned (port 0) is
restart-resilient — old daemon dies, new one binds a different port, hooks
re-discover via lock file. any-context-lazyclaude ships OS-assigned with PID-liveness
lock file probing
(`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:184`).
The brief lines 273-285 surface this as OQ-04 because the vision uses 2748/2749/2750
in its process-topology diagram (vision §Process Topology, brief lines 45-69) but
the hooks-r1/r2 pattern reads from a lock file.

### Options under evaluation

- **Option A — fixed port 2748 (hooks), 2749 (rmcp), 2750 (russh).** Hooks POST to a
  known URL. Port conflicts surface at daemon startup with a clear error.
- **Option B — OS-assigned port + lock-file discovery.** Daemon binds `127.0.0.1:0`,
  reads back the kernel-assigned port, writes `<runtimeDir>/monocle/daemon.json` with
  `{pid, port, authToken, contract_version}` at mode `0o600`. Hook scripts read the
  lock file, filter by PID liveness via `process.kill(pid, 0)`, pick the highest-port
  alive, POST.
- **Option C — hybrid.** Default to fixed 2748. If bind fails (EADDRINUSE), fall back
  to OS-assigned and write the lock file.

### Prior art evidence

- **any-context/lazyclaude — OS-assigned + lock file, verified.** Path:
  `~/.claude/ide/<port>.lock`. Format: JSON `{pid, port, authToken, transport, app}`
  at mode `0o600`. Hook discovery sequence (hooks-r1/r2 deepening):
  1. Read directory `~/.claude/ide/`.
  2. Filter `*.lock`.
  3. For each lock: parse JSON, parse port from filename, `process.kill(lk.pid, 0)`,
     track best (highest port among alive).
  4. If `best != null`: `srvPort = best.port`, `srvToken = best.lock.authToken`.
  5. Else: per-hook fallback (PreToolUse echoes stdin; others return).
  6. Build POST body; fire-and-forget with timeout.
  Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:168-174`.
  BC-HOOK-001 mandates "hooks NEVER cache env vars" — they re-discover on every fire.
- **Daemon's own daemon.json.** any-context-lazyclaude's daemon (different from the
  MCP server) writes its own `daemon.json` with `{Port, Token}` to stdout AND to
  `<runtimeDir>/daemon.json` at mode `0o600`. Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:150`.
- **Port 2748/2749/2750 conflict risk.** Direct check against the IANA registry
  excerpt at `https://en.wikipedia.org/wiki/List_of_TCP_and_UDP_port_numbers` shows
  that ports 2748-2758 are not listed for any common-known service. Searching the
  IANA service-names database returned no canonical assignment that would conflict.
  Port 2748 *is* used by some Dell PowerConnect 2748 switches as a management
  interface (`https://dcomcomputers.com/dell-powerconnect-2748-network-switch.html`)
  but that is a hardware product name not a network service.
- **rust-analyzer / LSP-stdio pattern (counterexample).** LSPs use stdio not TCP, so
  the analogy is limited. But the principle holds: LSP clients don't hard-code
  TCP ports; they fork the server and inherit pipes. Monocle can't quite do this
  because Claude Code subprocesses are spawned by the user, not by monocle, so
  monocle has to advertise its endpoint via the filesystem.
- **OS-assigned port in axum.** `axum::serve(TcpListener::bind("127.0.0.1:0").await?
  .local_addr()?.port())` works straightforwardly. Source:
  `https://docs.rs/axum/latest/axum/`. Pattern used widely in Rust tests.

### Implementation cost

- **A (fixed):** ~10-20 LOC. Just bind a constant. Error path on EADDRINUSE.
- **B (OS-assigned + lock):** ~150-200 LOC. Lock-file writer, lock-file reader,
  PID-liveness probe, directory scan with `*.lock` glob, JSON parse. ~80 LOC are
  the discovery sequence on the hook-script side (but that's Node JS in
  any-context — for monocle's parity test the same logic shipped as JS in the
  hook settings file body).
- **C (hybrid):** ~100 LOC. A + B + a feature flag. Operationally the worst —
  half the time hooks find the port via the fixed default, half the time via
  the lock file. Debugging hell.

### Operational implications

- **A (fixed):** When 2748 is already in use (e.g. user has another monocle daemon
  on a different runtime dir, or unrelated dev service), startup fails. Error
  recovery is a manual `monocle daemon start --port <other>` invocation, which
  the user then has to teach their Claude Code hooks about — by which point you
  have re-invented OS-assigned anyway.
- **B (OS-assigned):** Hook scripts must re-read the lock file on every hook fire
  (BC-HOOK-001). Adds ~5ms per hook for FS stat + read + JSON parse + kill(0)
  probe. Acceptable for hooks at the 100-1000/sec event rate.
  Restart resilience: kill the daemon, restart, hooks find the new daemon on
  next fire. No client restart needed.
- **C (hybrid):** Inherits B's complexity and A's confusion. Reject.
- **Token-rotation story:** with OS-assigned + lock file, the token is rotated
  on every daemon restart (because a new lock file is written). With fixed port,
  the token persists across restart only if the daemon reads the previous lock
  file on startup — which is an extra design step.

### Recommended default

**Option B (OS-assigned port + lock-file discovery, verbatim from any-context-lazyclaude).**

Rationale: monocle's hook protocol is byte-compatible with Claude Code's hook
schema (brief lines 82-85). Hook scripts must do PID-liveness lock-file discovery
anyway because that is the canonical pattern Claude Code's surrounding ecosystem
(any-context, lazyclaude, downstream tooling) has standardized on. Fixed port is
neither faster nor more debuggable; OS-assigned is strictly more restart-resilient
with the same operational footprint once you have a lock file (which you need
for the daemon auto-start handshake from OQ-01 regardless).

Drop the brief's "2748/2749/2750" port-number conventions — they were aspirational
diagrams, not architectural commitments. The lock file is the source of truth.

### Confidence

**HIGH.** any-context-lazyclaude verifies the lock-file-discovery pattern across 41
hook contracts (BC-HOOK-001..041) plus 77 server contracts (BC-MCPSRV-001..077),
including `TestLockManager_FilePermissions`
(`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:24`).
The pattern is well-trodden in the Rust HTTP ecosystem.

### Architect action

In `monocle-runtime::daemon::bind`, define
`fn bind_local() -> Result<(TcpListener, u16, Token)>` that binds `127.0.0.1:0`,
extracts the kernel-assigned port via `listener.local_addr()?.port()`, generates a
fresh token (32 bytes from `rand::thread_rng()`, base64url-encoded), and returns
the triple. In `monocle-runtime::daemon::lock`, define
`fn write_lock_file(path: &Path, lock: &LockFile) -> Result<()>` that writes via
`tempfile::persist`, then `chmod 0o600`. The lock-file schema follows
`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:184`
exactly: `{ pid: u32, port: u16, authToken: String, transport: "http", app: "monocle",
contract_version: 1 }`. The hook-side discovery JS (embedded in the settings file
body via the OQ-02 writer) follows the any-context pattern verbatim.

---

## OQ-05: Profile picker on session create vs sticky-per-project?

### Trade-off summary

When the user opens a new harness session, does monocle prompt for a profile
("Claude Code default / Claude Code high-context / CodeMachine") via picker UI,
or does it remember the last-used profile per project and silently reuse? Picker
respects power users with many profiles; sticky respects the killer scenario where
the user just wants to launch and go.

### Options under evaluation

- **Option A — picker on every session create.** Show `ProfilePicker` overlay
  when user invokes `monocle session new`. Default selection is the most recently
  used profile (or the first listed).
- **Option B — sticky per project.** Monocle stores `~/.monocle/state.json` with
  `{project_path => last_profile_id}` map. New session in project P silently picks
  the recorded profile. If no record, falls back to default profile from config.
- **Option C — sticky per project with manual override.** Default to B. Add a
  flag/keystroke to invoke the picker (`monocle session new --profile <id>` or
  `Ctrl-P` from picker preview).

### Prior art evidence

- **claude-squad — `ProfilePicker` overlay on instance creation.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md:265-273`:
  "If multiple profiles are defined, a `ProfilePicker` appears at instance creation
  time. The default profile is determined by `DefaultProgram` matching a profile's
  `Name`. There is no per-instance config." The data model is intentionally minimal:
  `Profile = {Name, Program}`. claude-squad picks at create-time, no sticky behavior.
  Patterns to inherit: "default-first ordering convention"
  (`/Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md:437`).
- **codemachine — engine override resolution.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md:33-50`.
  Engine resolution precedence: `step.engine ?? agent.engine ?? registry.default()`.
  Per-step override is the explicit mechanism; no picker UI — the workflow YAML
  declares the engine declaratively. Different domain (declarative workflow vs
  interactive TUI), so the pattern is informative but not directly transferable.
- **lazygit `customCommands` selection** — lazygit prompts for custom-command
  arguments via the `Prompt` popup primitive when the command has placeholders.
  Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/lazygit/lazygit-pass-8-final-synthesis.md:159-186`
  ("`Prompt(opts PromptOpts)` — text entry with optional suggestions"). No
  per-project stickiness; the prompt fires every invocation. lazygit's pattern is
  not exactly comparable but reinforces "prompt-on-invocation is acceptable UX".
- **No prior art for sticky-per-project for harness profiles** in the gene set.
  The closest is codemachine's per-step engine override, which is declarative.

### Implementation cost

- **A (picker every time):** ~200-300 LOC. Picker overlay (built on lazygit's
  `Menu` primitive, ratatui adaptation). Modal state in AppMode enum. Most-recent-
  used order surfaces sensibly. Plus tests for the picker UI.
- **B (sticky per project):** ~100-150 LOC. State store (JSON), per-project lookup,
  fallback chain. No picker UI in v1.
- **C (sticky + override):** ~250-350 LOC. B + the picker from A as an opt-in
  overlay. Two code paths, but they share the picker implementation.

### Operational implications

- **A (picker):** Every session creation costs the user a keystroke or two.
  Acceptable for users with 2-3 profiles. Annoying for users with one profile.
  Mitigation: skip picker if only one profile is defined (claude-squad's behavior).
- **B (sticky):** "Surprising" behavior — user changes default profile in config,
  but the next session still uses the old per-project sticky value. Mitigation:
  on profile config change, mark all per-project records as "stale" and force
  re-pick.
- **C (sticky + override):** Closest to lazygit philosophy (smart defaults, easy
  override). Largest engineering surface for v1.
- **Persistence on B and C:** `~/.monocle/state.json` is separate from
  `~/.monocle/config.json` (config = user intent, state = program memory). This
  mirrors the claude-squad pattern
  (`/Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md:255-256`,
  "`config.json` user-facing; `state.json` program-managed") which is good
  hygiene.

### Recommended default

**Option C (sticky per project with picker override, Phase 1 minimum-viable).**

Rationale: the killer scenario (vision §End-to-End Killer Scenario) demands a
4-keystroke workflow. Forcing a picker on every session create adds 1-2 keystrokes
and breaks the "lazy" philosophy. Sticky per project is the silent-default that
lazy* tools prefer. The picker is the override path for power users (Ctrl-P or
`monocle session new --profile <id>`). Implementation cost is acceptable for
Phase 1 because the state.json store is reusable for OQ-06 event retention.

Phase 1 minimum: if user has exactly one profile, skip the picker entirely
(claude-squad's behavior). If multiple, sticky-per-project picks last-used.
Override via `Ctrl-P` from the dashboard.

### Confidence

**MEDIUM.** The split between claude-squad (picker-always) and codemachine
(declarative) is a clean line, but monocle's UX (interactive TUI for multi-session
management) sits between them. Decision is opinionated rather than strongly
evidenced; the architect should run a Phase 1 user-test with at least 3 multi-
profile users.

### Architect action

In `monocle-config::state`, define `SessionStateStore` writing
`~/.monocle/state.json` via `tempfile::persist`. Schema:
```json
{
  "version": 1,
  "projects": {
    "/abs/path/to/project": {
      "last_profile_id": "claude-code-default",
      "last_used_at": "2026-05-12T14:00:00Z"
    }
  }
}
```
In `monocle-tui::session::create`, resolve the profile via:
1. `--profile <id>` CLI flag wins.
2. Sticky lookup for `cwd()` in state store.
3. `Ctrl-P` to invoke picker.
4. Fall back to default profile from config.
The picker UI is a `Menu` overlay (per the AppMode VecDeque<PromptModal> stack
in brief lines 246-247).

---

## OQ-06: Hook event timeline retention — in-memory ring buffer or persisted JSONL?

### Trade-off summary

The hook event ribbon (brief line 95) shows the last N events. If in-memory only,
the ribbon resets on daemon restart and there is no replayable test fixture for
holdout-evaluation. If persisted JSONL, restart-resilience is preserved and tests
can replay deterministic event streams, but disk I/O cost at 100-1000 events/sec
must be quantified.

### Options under evaluation

- **Option A — in-memory ring buffer only.** `VecDeque<Event>` capped at e.g. 1000
  events, eviction on push when full. Lost on daemon restart.
- **Option B — persisted JSONL append-only.** `<XDG_STATE_HOME>/monocle/events.jsonl`
  rolling log. One line per event. Tail-N for the ribbon. Rotation/truncation policy
  needed.
- **Option C — hybrid (RAM ring + periodic JSONL flush).** Same in-memory ring as A
  for hot path. Background task flushes to JSONL every N seconds or M events.
  Restart loads recent JSONL into the ring buffer.

### Prior art evidence

- **any-context broker — no persistence.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:198-206`
  ("Broker dispatch (broker-r1/r2 verified). Single sync.Mutex. Non-blocking publish
  (select with empty default). Drops are completely silent. Publish fan-out runs
  entirely under the mutex"). The broker is RAM-only; subscribers buffer 8 (GUI)
  / 64 (daemon SSE). No JSONL or disk persistence.
- **zellij — no event-stream persistence.** Plugin events are RAM-only and lost
  on restart. Session layout is persisted (5-thread save chain) but that is
  state-not-events. Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:314-353`.
- **Observability tool patterns (vector.dev, OpenTelemetry collector).** External
  evidence from `https://oneuptime.com/blog/post/2026-01-25-high-throughput-data-ingestion-pipeline-rust/view`:
  "Writers batch and persist data to storage (databases, object stores, queues)
  using bounded channels. Batching aggressively is recommended because network
  and disk I/O have high per-operation overhead." This argues for hybrid (RAM
  for hot path, batched flush to disk).
- **Disk cost of JSONL append at 1000 events/sec.** A typical hook event JSON
  is 200-500 bytes. 1000/sec × 350 bytes ≈ 350 KB/sec or 21 MB/min. Sustained over
  a 30-minute session that's ~600 MB — too much without rotation. With rotation
  at 100 MB and 5-file retention, peak disk use is 500 MB. Acceptable for a dev
  machine. Append-only writes are fast (sequential disk I/O, ~1ms per batch via
  buffered writer).
- **Holdout-evaluation requirement.** Per the brief, vsdd-factory holdout
  evaluators need replayable test fixtures. The brief explicitly flags this in
  OQ-06's option text (brief line 280): "persisted JSONL enables replay and
  holdout evaluation". This is the differentiator — A cannot satisfy this, B/C can.

### Implementation cost

- **A (RAM ring):** ~50 LOC. `tokio::sync::Mutex<VecDeque<Event>>` with cap.
- **B (JSONL):** ~200-300 LOC. Async writer task, line-buffered, rotation policy
  (`tempfile::persist` for atomic rotation), tail-N reader for ribbon, JSONL
  parser for replay. Plus 4-6 integration tests for rotation and crash
  consistency.
- **C (hybrid):** ~250-350 LOC. A + B + a batching flusher. Most engineering
  surface but the right architectural shape.

### Operational implications

- **A:** Status bar shows event count from session start; ribbon resets on
  daemon restart. Acceptable for "kid you not, I just want to see what's happening
  right now" UX. Inacceptable for "replay this session's events to debug a
  customer issue".
- **B:** Disk space management is a real concern — must rotate. Privacy concern:
  hook events include tool inputs which may contain code/secrets. JSONL on disk
  at mode 0o600 is necessary. Encryption at rest is out of scope for v1.
- **C:** Same concerns as B for the on-disk side. Hot-path latency is A's profile
  (no synchronous disk I/O on event publish).

### Recommended default

**Option C (hybrid: RAM ring + async JSONL flush, with 100MB × 5 rotation).**

Rationale: holdout-evaluation is a v1 success criterion that requires replayable
fixtures (brief line 280 notes the test-harness implication). RAM-only fails that
test. Pure JSONL adds synchronous disk I/O to the hot publish path (or requires
the same async batcher that hybrid uses — at which point you've built C anyway).
Hybrid is the canonical observability-tool pattern (vector.dev, OTel collector)
and bounds memory and disk simultaneously. Configure rotation as 100MB × 5 files
for ~500MB cap. Add a config knob `event_retention.disk_enabled: bool` for users
who want pure-RAM (low-trust hosts, e.g. shared dev VMs).

### Confidence

**HIGH.** Holdout-evaluation requirement is explicit; the canonical observability
pattern is well-trodden; disk-cost numbers are arithmetic.

### Architect action

In `monocle-runtime::events`, define:
1. `EventRing` — bounded `VecDeque<Event>` cap 1000, `tokio::sync::RwLock`. Read
   path is `tail(n)` for ribbon render. Write path is `push_back + evict`.
2. `EventLog` — async tokio task that subscribes to the broker, batches events
   (50-event batches or 1-second tick), writes batches to
   `<XDG_STATE_HOME>/monocle/events.jsonl` via buffered writer. Rotation at 100MB
   to `events.jsonl.1` through `events.jsonl.5`; oldest evicted on rotate.
3. Mode `0o600` on the JSONL file. Config knob `events.disk_persistence: bool`
   defaulting to true. Replay API: `EventLog::replay(path) -> impl Iterator<Item =
   Event>` for test fixtures.

---

## OQ-07: Cross-host session migration scope — v1 or v4?

### Trade-off summary

Vision puts federation in Phase 4 (brief line 129; vision §Phase Plan). The
question is whether v1 architecture must include forward-compat seams (protobuf
schemas, transport abstractions) so that Phase 4 federation is an additive feature,
not an ABI-breaking retrofit.

### Options under evaluation

- **Option A — zero federation seams in v1.** Phase 1 is entirely local; protobuf
  schemas, `monocle-proto`, russh tunnel come in Phase 4 as new crates.
  Phase 4 will require trait changes to `EngineModule` and `FactoryAdapter` to
  add `host_id: HostId` fields. Breaking ABI between phases (which contradicts
  brief line 327: "the ABI between phases must be stable").
- **Option B — full federation in v1.** Ship russh tunnel + protobuf wire +
  multi-host roster in v1. Massive Phase 1 scope creep; rejects the brief's
  Phase Plan.
- **Option C — protobuf seams in v1, transport-only in v4.** Phase 1 ships
  `monocle-proto` crate with `prost`-generated types for all cross-cutting events.
  `EngineModule` and `FactoryAdapter` traits use these types from day one. Phase 4
  adds the russh transport layer that ships these types over SSH.

### Prior art evidence

- **zellij — local-only transport, but interprocess generalizes.** zellij uses
  `interprocess::local_socket::Stream` (Unix domain socket on Unix, named pipe on
  Windows). Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:93`.
  Adding remote transport in zellij would require swapping `interprocess` for
  e.g. `tokio::net::TcpStream` or a russh `Channel<Msg>`. The schema (4-byte
  length prefix + prost protobuf) survives the transport swap unchanged.
- **any-context daemon — federation via SSH composite + remote provider.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:148-149`:
  "Remote SSH composite: local TUI + remote lazyclaude daemon. CompositeProvider
  routes per-session ops to local session.Manager or per-host RemoteProvider.
  MirrorManager creates placeholder local tmux windows that SSH-attach. SSE
  notifications get Window remapped from remote-tmux to local-mirror IDs."
  The pattern is: same daemon process per host, federated via russh tunnel.
  Critical: the wire schema is the same between local and remote; only the
  transport differs.
- **russh 0.45 → 0.60 API stability.** Per
  `https://github.com/Eugeny/russh/releases` review:
  - v0.55: Named pipes (Windows support added).
  - v0.56: Server-side ping for latency. Marvin attack mitigation.
  - v0.57: Configuration accessor methods made public.
  - v0.58: `Handle::tcpip_forward` and `Handle::streamlocal_forward` now take
    `&self` instead of `&mut self` (breaking).
  - v0.58: Non-sensitive buffers no longer wrapped in `CryptoVec`; APIs now take
    `impl Into<Bytes>` (breaking).
  - v0.60.1: DoS in keyboard-interactive auth (security patch).
  Conclusion: russh has minor breaking API changes per minor version. Pin
  `russh = "0.60"` (currently 0.60.2 per crates.io) and accept that Phase 4 may
  need a russh update. The protobuf wire schema is decoupled from russh.

### Implementation cost

- **A (no seams):** Phase 1 adds 0 LOC. Phase 4 adds ~3,000 LOC plus the
  breaking-change retrofit cost (~500 LOC of trait migration and downstream
  impl updates).
- **B (full federation v1):** Phase 1 adds ~3,000 LOC and ~200 hours of testing
  surface for ranges of network failures, cross-host clock skew, partial
  failures, etc. Massively over-scoped.
- **C (protobuf seams v1):** Phase 1 adds ~500 LOC (`monocle-proto` crate with
  prost code-gen, schema for `Event`, `SessionMetadata`, `HookPayload`, etc.).
  Phase 4 adds ~2,000 LOC (russh tunnel transport, multi-host roster aggregation).
  Total ~2,500 LOC — slightly less than A's 3,500 because the trait migration
  pain is avoided.

### Operational implications

- **A:** Phase 4 retrofit hurts because the brief explicitly bans cross-phase ABI
  breaks: "No breaking changes to these traits between phases" (brief line 327).
  Reject A on that constraint alone.
- **B:** Operationally heavy in v1. SSH errors are subtle; askpass integration is
  a P1-005-flavored gap surface
  (`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:293`).
- **C:** Phase 1 architects with cross-host in mind without paying for it at
  runtime. The protobuf types compile to native Rust enums via `prost`; using
  them in v1 has near-zero overhead vs `serde`-derived structs.

### Recommended default

**Option C (protobuf seams in Phase 1, transport in Phase 4).**

Rationale: the brief explicitly bans cross-phase ABI breaks for `EngineModule`
and `FactoryAdapter`. Phase 1 must therefore commit to wire types that survive
into Phase 4. Protobuf via `prost 0.14` (already pinned in brief) is the canonical
choice and matches the zellij pattern. Phase 4 then adds russh as a transport
plug-in for those same types. monocle-proto crate is in the brief's vision
§Workspace Layout (brief line 89) — adopting it from Phase 1 honors the vision.

### Confidence

**HIGH.** ABI stability mandate is explicit; protobuf via prost is the canonical
Rust wire format; russh's API stability story is researched.

### Architect action

In `monocle-proto`, define the `.proto` schema for these messages, ship them in
v1:
- `HookEvent` (PreToolUse, PostToolUse, Stop, Permission — already in brief)
- `SessionMetadata` (id, project_path, profile_id, harness_id, started_at)
- `FactoryState` (phase, status, blocking issues)
- `Permission` (the 17-permission enum from zellij, for forward-compat)
- `HostId` field on every message — defaults to `"local"` in v1, populated by
  Phase 4 federation.
The wire serialization uses 4-byte LE length prefix + prost-encoded payload, per
zellij `ipc.rs:402-426` pattern (cited in
`/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:96`).
Phase 4's russh adapter implements `Transport` trait that swaps the underlying
stream from UDS to a russh `Channel<Msg>`.

---

## OQ-08: monocle-ipc — UDS only, or UDS + shared-memory ring buffer both in v1?

### Trade-off summary

UDS over `interprocess 2.4` is sufficient for hook events at the 100-1000/sec rate
the brief projects (brief line 170). Shared-memory ring is a Phase 4 optimization
for sub-microsecond fan-out across multiple TUI clients on the same host. The
brief lists both in the vision §Workspace Layout (brief line 87,
`monocle-ipc/`). The question is whether shared-mem ships in v1 or stays
deferred.

### Options under evaluation

- **Option A — UDS only.** `interprocess::local_socket::Stream` for TUI client ↔
  daemon. prost-encoded length-prefixed messages. Adequate for hook event rates
  and TUI render budgets.
- **Option B — UDS + shared-memory ring buffer.** UDS for control plane (session
  list, commands), shared-mem ring for high-frequency events. Adds `shmem-ipc`
  or `raw_sync` or hand-rolled `mmap` + futex coordination.
- **Option C — UDS in v1, shared-mem as Phase 4 opt-in.** Architecture supports
  pluggable transport (the Transport trait from OQ-07). Shared-mem ring lands in
  Phase 4 as a transport variant for users with the throughput need.

### Prior art evidence

- **zellij — UDS only, no shared-mem.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:92-97`.
  zellij is a far more event-heavy system (every keystroke, every PTY byte) and
  still uses UDS with prost without shared-mem. If zellij does not need shared-mem
  for its full multiplexer workload, monocle's hook-event rate (100-1000/sec, well
  below zellij's keystroke rate) does not need it either.
- **any-context broker — bounded mpsc, no shared-mem.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:201-205`:
  "Per-subscriber buffer sizes in lazyclaude: GUI notify_loop.go:44: 8;
  Daemon SSE server_sse.go:44: 64. Recommended monocle policy: 16 local / 64
  remote-fanout, plus per-subscriber atomic drop counter exposed via
  `Subscription.DroppedCount() -> u64`." Bounded mpsc + UDS is sufficient at
  any-context's scale.
- **`interprocess 2.4` UDS throughput.** Per
  `https://github.com/redhat-performance/rusty-comms` (Rust IPC benchmark suite),
  Unix domain sockets in Rust are in the hundreds-of-thousands of messages per
  second range for small payloads. monocle's projected 1000/sec is two orders of
  magnitude below saturation. Source:
  `https://docs.rs/interprocess/latest/interprocess/` confirms the API surface.
- **prost + UDS throughput context.** prost-encoded small payloads (~200-500
  bytes per hook event) at 1000/sec is ~500KB/sec or 4Mbps — trivial throughput
  on modern hardware. No shared-mem motivation.

### Implementation cost

- **A (UDS only):** ~400-600 LOC. `interprocess::local_socket::Stream` wrapper,
  prost serialize/deserialize, length-prefix framing (`zellij-utils/src/ipc.rs`
  pattern). Plus 6-10 tests for connect/disconnect/reconnect.
- **B (UDS + shared-mem):** ~1,500-2,500 LOC. Shared-mem ring (~400 LOC of mmap
  + atomics + ABA-safe ring buffer logic), control-plane UDS (~600 LOC),
  coordination layer (which channel does which message use). Plus extensive
  fuzz testing for the shared-mem path.
- **C (UDS now, shared-mem later):** Same as A for v1. Phase 4 adds the
  shared-mem variant when (if) the throughput need materializes.

### Operational implications

- **A:** Simpler debug story. `socat - UNIX-CONNECT:<sock>` works for poking
  the daemon by hand. No special tooling.
- **B:** Shared-mem rings are notorious sources of subtle bugs (ABA, lost
  wakeups, false-sharing). Even mature crates like `raw_sync` and `shared_memory`
  have had soundness issues; review their RUSTSEC history before adoption.
  Adopting in v1 buys complexity without solving a documented bottleneck.
- **C:** Phase 4 motivation would be "monocle TUI shows 10000 events/sec" which
  is unlikely from a single Claude Code session but possible with multi-harness
  burst. By Phase 4 we have real telemetry; defer the decision until then.

### Recommended default

**Option C (UDS-only in v1, shared-mem deferred to Phase 4 transport variant).**

Rationale: hook event rates are well below UDS saturation. The brief's vision
listing both transports (vision §Workspace Layout) is aspirational; the
architecture should let the `monocle-ipc` crate house both Transport
implementations under a common trait, with only UDS landing in v1. This matches
zellij's actual shipping reality and is faithful to the lazygit philosophy of
not building speculative engineering.

Note that the brief's anti-pattern enforcement (brief lines 246-247) bans
unbounded channels, mandating bounded mpsc with drop counter. That maps to UDS
+ tokio mpsc; shared-mem would re-open the unbounded discussion.

### Confidence

**HIGH.** Throughput math is arithmetic; zellij precedent is direct; any-context
broker buffer sizes are explicit guidance.

### Architect action

In `monocle-ipc`, define:
```rust
trait Transport: Send + Sync {
    type Sender: Sender;
    type Receiver: Receiver;
    fn connect(addr: &str) -> Result<(Self::Sender, Self::Receiver)>;
}
struct UdsTransport;
impl Transport for UdsTransport { /* interprocess + prost length-prefix */ }
```
Ship `UdsTransport` only in v1. Phase 4 adds a `ShmRingTransport` (or `RusshTransport`
for federation). Wire framing is 4-byte LE length prefix + prost-encoded payload
verbatim from zellij's `write_protobuf_message` /  `read_protobuf_message` pattern.
Per-subscriber buffer: 16 (local), 64 (remote). Drop counter is
`AtomicU64::DroppedCount()` exposed in TUI status bar per brief mandate (brief line
170, "drop counter renders in status bar").

---

## OQ-09: rmcp MCP bridge (port 2749) — stub with no-op handlers in v1, or omit entirely?

### Trade-off summary

The brief mentions an optional MCP bridge on port 2749 (Phase 4 feature, brief
line 80). The question is whether v1 includes a stub `ServerHandler` impl with
default no-op methods (so the port is reserved and the architecture honors the
existence of MCP), or whether v1 omits the bridge entirely. Including a stub
costs ~50 LOC and one runtime task; omitting leaves port 2749 available for any
other purpose.

### Options under evaluation

- **Option A — ship rmcp stub in v1.** `monocle-runtime::mcp::stub` exposes a
  `ServerHandler` that returns default values from all 26 trait methods
  (per Context7 lookup of `/websites/rs_rmcp_rmcp`'s `ServerHandler` trait).
  Bind to `127.0.0.1:0` (with OS-assigned port) or to the lock-file-discovered
  port (per OQ-04 resolution).
- **Option B — omit entirely.** No rmcp dep in v1. Phase 4 adds rmcp 1.6 when
  the feature lands.
- **Option C — feature-gate the stub.** `cargo features = ["mcp-stub"]` ships
  the rmcp dep behind a non-default flag. Default builds omit it.

### Prior art evidence

- **rmcp 1.6 SDK from Anthropic-canonical (modelcontextprotocol/rust-sdk).**
  Source:
  `https://github.com/modelcontextprotocol/rust-sdk`,
  `https://docs.rs/rmcp`. Context7 lookup (library ID `/websites/rs_rmcp_rmcp`)
  shows the `ServerHandler` trait has 26 methods, all with default provided
  impls returning sensible "method not supported" results.
- **Minimal stub example from Context7:**
  ```rust
  struct TimeServer;
  #[tool_router]
  impl TimeServer {
      #[tool(description = "Get current time")]
      async fn get_time(&self) -> String { "12:00".into() }
  }
  #[tool_handler]
  impl ServerHandler for TimeServer {}
  ```
  Even simpler: a unit struct with `impl ServerHandler for UnitStub {}` works
  because all methods are provided.
- **rmcp 1.6 MSRV.** Per crates.io API check 2026-05-12, `rmcp 1.6.0` lists no
  `rust_version` field. Inspecting transitive deps via `cargo tree` would
  reveal the effective floor. Conservatively: rmcp depends on tokio,
  schemars, serde_json — all MSRV ≤1.71. No MSRV pressure from rmcp.
- **rmcp binary-size impact.** rmcp depends on schemars, serde_json, tokio
  (already pinned). Net new code is ~200-500KB conservative. No new system
  libraries.

### Implementation cost

- **A (stub v1):** ~50-100 LOC for the unit-struct impl. ~50 LOC for the
  bind-on-startup logic in `monocle-runtime::daemon::start`. The rmcp dep adds
  ~2-3 second compile-time cost per clean build.
- **B (omit):** 0 LOC v1. Phase 4 adds ~500-1000 LOC for a real implementation.
- **C (feature-gated):** A's LOC + a feature flag. Default-off means most users
  don't pay the dep cost. Activates when needed.

### Operational implications

- **A:** The MCP port is "claimed" by monocle even before the feature is real.
  External MCP clients connecting to `127.0.0.1:<rmcp_port>/<lock_file_port>`
  will see a server that responds to `initialize` but returns "no tools, no
  resources, no prompts". Mildly confusing if observed; useful if it
  unblocks early integration testing.
- **B:** Port 2749 (or whatever rmcp would have used) is free for other tools.
  Less binary, less attack surface, simpler runtime.
- **C:** Best of both: ship the stub gated for opt-in, default-off keeps
  binaries lean.

### Recommended default

**Option B (omit rmcp entirely in v1).**

Rationale: the brief flags rmcp explicitly as Phase 4 (brief line 138, "rmcp
MCP bridge (optional, port 2749): session query, prompt injection for
tooling"). Shipping a stub provides zero user value in v1 — there is no
external client that benefits from a no-op MCP server. The architecture is
not constrained by omitting it: Phase 4 adds rmcp as a new dep with a real
implementation. The vision's process-topology diagram (brief lines 56-58)
shows port 2749 as optional from day one.

If the architect wants to "reserve" the port semantically, that can be done
with a comment in `monocle.toml`'s config schema:
`# port 2749 reserved for Phase 4 rmcp MCP bridge` — no code required.

### Confidence

**HIGH.** rmcp is explicitly Phase 4 per brief; no v1 user story requires it;
omitting reduces dep surface.

### Architect action

In v1 architecture: no `monocle-runtime::mcp` module, no rmcp dep in
`Cargo.toml`. Document in `monocle-core::ports` (a constants module) that
port 2749 is reserved for Phase 4 rmcp via a documentation comment. Phase 4
deliverable is the real rmcp `ServerHandler` impl with tool registry that
queries the session roster and supports prompt injection.

---

## OQ-10: Daemon lock file location — ~/.monocle/daemon.json or $XDG_RUNTIME_DIR/monocle/daemon.json?

### Trade-off summary

XDG_RUNTIME_DIR is Linux-only by spec (typically `/run/user/$UID`); macOS does
not provide it. `~/.monocle/` is portable but not XDG-compliant. The
`directories 6` crate exposes per-OS paths but its `runtime_dir()` returns
`None` on macOS and Windows (per `https://docs.rs/directories`). The architect
must pick a portable convention.

### Options under evaluation

- **Option A — `~/.monocle/daemon.json` only.** Portable. Simple. Lock file
  alongside `config.json` and `state.json`. Mirrors any-context's
  `<runtimeDir>/daemon.json` (where `runtimeDir` is the user's home subdir).
- **Option B — `$XDG_RUNTIME_DIR/monocle/daemon.json` (Linux) with fallback
  to `~/.monocle/daemon.json` on macOS and other OSes lacking runtime_dir.**
  XDG-compliant on Linux.
- **Option C — `directories::ProjectDirs::runtime_dir()` with fallback.**
  Crate handles the per-OS dispatch. Falls back to `cache_dir()` or `home_dir()`
  when `runtime_dir()` returns None.

### Prior art evidence

- **any-context-lazyclaude — uses runtimeDir.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:150`:
  "Daemon-only: remote host runs lazyclaude daemon. Spawns its own in-process
  MCP server (cmd/lazyclaude/daemon_cmd.go:67) so hooks on remote use identical
  code path. Emits daemon.json {Port, Token} to stdout AND writes to
  <runtimeDir>/daemon.json mode 0600." `runtimeDir` in any-context is
  effectively `~/.lazyclaude/runtime/` or similar; not strictly XDG.
- **zellij — XDG-compliant via `directories::ProjectDirs`.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:316-329`:
  "Filesystem layout (zellij-utils/src/consts.rs:89-110):
  ~/.cache/zellij/permissions.kdl ... ~/.cache/zellij/contract_version_1/session_info/<session_name>/..."
  zellij uses `~/.cache/zellij/` (i.e. `XDG_CACHE_HOME` on Linux,
  `~/Library/Caches/zellij` on macOS). Note: zellij does NOT use `runtime_dir`
  because that path is wiped on logout — but lock files for currently-running
  daemons want exactly that semantic on Linux.
- **lazygit — XDG_STATE_HOME for state, XDG_CONFIG_HOME for config.** Source:
  `/Users/jmagady/Dev/monocle/.factory/semport/lazygit/lazygit-pass-8-final-synthesis.md:389`:
  "`<XDG_STATE_HOME>/lazygit/state.yml` — machine-managed state (recent repos,
  last update check, GitHub PR cache, command history). Distinct from config
  (user intent)."
- **`directories 6` semantics.** Per
  `https://docs.rs/directories`:
  - `runtime_dir()` returns `Some(/run/user/$UID/<project>/)` on Linux when
    `XDG_RUNTIME_DIR` is set; `None` on macOS, Windows, and Linux without the env.
  - Spec: "the path to a directory for non-essential runtime data that is created
    when the user logs in, is only accessible by the user itself, is deleted
    when the user logs out, and supports all filesystem features of the OS."
  - For a daemon lock file: runtime_dir is semantically correct (wiped at
    logout = stale lock files cleaned up automatically) but not available
    everywhere.
- **Common Rust daemon pattern.** Production Rust daemons typically use:
  - `~/.local/share/<app>/<file>` for state (XDG_DATA_HOME on Linux).
  - `~/.cache/<app>/<file>` for cache (XDG_CACHE_HOME on Linux).
  - `$XDG_RUNTIME_DIR/<app>/<file>` for lock files when available.
  - Fall back to `~/.config/<app>/` or `~/.<app>/` on macOS.

### Implementation cost

- **A (`~/.monocle/`):** ~10-20 LOC. Simple `home_dir().join(".monocle")`.
- **B (XDG runtime_dir + fallback):** ~50-80 LOC. Per-OS dispatch in
  `monocle-config::paths`. Tests for both Linux-with-runtime-dir and
  Linux-without and macOS.
- **C (`directories::ProjectDirs`):** ~40-60 LOC. The crate handles per-OS
  dispatch; just a fallback chain when `runtime_dir()` returns None.

### Operational implications

- **A:** The lock file persists across logouts (it lives in HOME which is not
  wiped). Stale lock files from a crashed daemon survive reboot — must rely on
  PID-liveness probe for cleanup. Acceptable: PID liveness is already required
  per OQ-04.
- **B:** On Linux, runtime_dir cleanup is automatic at logout — fewer stale
  locks. On macOS, behavior is the same as A.
- **C:** Same as B operationally; crate-supplied dispatch reduces hand-rolled
  per-OS code.

### Recommended default

**Option C (`directories::ProjectDirs` with `runtime_dir()` preferred,
fallback to `state_dir()`, then `data_dir()`, then `~/.monocle/runtime/`).**

Rationale: `directories 6` is already pinned in the brief (line 192) and is the
canonical XDG abstraction. Letting it handle per-OS dispatch is the lowest-cost
correct answer. The fallback chain ensures monocle works on macOS (where
runtime_dir is None) and Linux containers without XDG_RUNTIME_DIR set.

Exact paths under Option C:
- Linux with XDG_RUNTIME_DIR: `/run/user/$UID/monocle/daemon.json`
- Linux without: `$XDG_STATE_HOME/monocle/daemon.json` (typically
  `~/.local/state/monocle/`)
- macOS: `~/Library/Application Support/monocle/runtime/daemon.json`
- Final fallback: `~/.monocle/runtime/daemon.json`

Other files (config, state, events.jsonl) use the equivalent crate methods:
- `config_dir()` → config.json
- `state_dir()` (or `data_dir()` on macOS) → state.json, events.jsonl
- `cache_dir()` → optional WASM-module cache (Phase 3)

### Confidence

**HIGH.** `directories 6` is the standard library for this. The fallback chain
is mechanical.

### Architect action

In `monocle-config::paths`, define:
```rust
pub struct MonoclePaths {
    pub runtime: PathBuf,  // daemon.json lock file
    pub state:   PathBuf,  // state.json, events.jsonl
    pub config:  PathBuf,  // config.json
    pub cache:   PathBuf,  // future: compiled wasm cache
}
impl MonoclePaths {
    pub fn discover() -> Result<Self> {
        let proj = directories::ProjectDirs::from("dev", "monocle", "monocle")
            .ok_or(MonocleError::NoHomeDir)?;
        let runtime = proj.runtime_dir().map(Path::to_path_buf)
            .or_else(|| Some(proj.state_dir().unwrap_or(proj.data_dir()).join("runtime")))
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".monocle/runtime"));
        // ... etc.
    }
}
```
Add a smoke test that exercises both with and without XDG_RUNTIME_DIR set.

---

## OQ-11: MSRV target — pin specific minimum supported Rust version

### Trade-off summary

The workspace `rust-version` field gates which Rust toolchains can compile
monocle. Pinning too high (e.g. 1.92 for wasmtime 44) limits contributor
toolchains and CI matrix. Pinning too low fails to compile against required
deps. The correct MSRV is the highest MSRV of any pinned crate — but if a
high-MSRV crate is Phase 3 (wasmtime), Phase 1 can target a lower MSRV.

### Verified per-crate MSRV (crates.io 2026-05-12)

For each crate pinned in the brief, the `rust_version` field was fetched from
the crates.io REST API at the version specified in the brief. Direct sources:
- `https://crates.io/api/v1/crates/<name>/<version>`

| Crate | Version pinned | MSRV (rust_version) | Source |
|-------|----------------|---------------------|--------|
| wasmtime | 44.0.1 | **1.92.0** | crates.io API 2026-05-12 |
| ratatui | 0.30.0 | 1.86.0 | crates.io API 2026-05-12 |
| russh | 0.60.2 | 1.85 | crates.io API 2026-05-12 |
| axum | 0.8.9 | 1.80 | crates.io API 2026-05-12 |
| notify | 8.0.0 | 1.77 | crates.io API 2026-05-12 |
| similar | 3.0.0 | 1.85 | crates.io API 2026-05-12 |
| clap | 4.6.0 | 1.85 | crates.io API 2026-05-12 |
| interprocess | 2.4.0 | 1.75 | crates.io API 2026-05-12 |
| prost | 0.14.0 | 1.71.1 | crates.io API 2026-05-12 |
| tokio | 1.52.3 | 1.71 | crates.io API 2026-05-12 |
| thiserror | 2.0.0 | 1.61 | crates.io API 2026-05-12 |
| serde_yaml_ng | 0.10.0 | 1.64 | crates.io API 2026-05-12 |
| reqwest | 0.13.0 | 1.64 | crates.io API 2026-05-12 |
| crossterm | 0.29.0 | 1.63 | crates.io API 2026-05-12 |
| pulldown-cmark | 0.13.0 | 1.71.1 | crates.io API 2026-05-12 |
| rmcp | 1.6.0 | none (defers to deps) | crates.io API 2026-05-12 |
| directories | 6.0.0 | none (defers to deps) | crates.io API 2026-05-12 |
| arboard | 3.0.0 | none (defers to deps) | crates.io API 2026-05-12 |
| nucleo | 0.5.0 | none (defers to deps) | crates.io API 2026-05-12 |
| tempfile | 3.0.0 | none (defers to deps) | crates.io API 2026-05-12 |
| anyhow | 1.0.0 | none (defers to deps) | crates.io API 2026-05-12 |
| semver | 1.0.0 | none (defers to deps) | crates.io API 2026-05-12 |
| tracing | 0.1.0 | none (defers to deps) | crates.io API 2026-05-12 |

The highest MSRV among pinned crates is **wasmtime 44.0.1 at Rust 1.92.0**.
The second-highest is ratatui 0.30 at 1.86.

### Phase-aware analysis

wasmtime is a **Phase 3** dependency (brief line 126; vision §Workspace Layout
puts `monocle-plugin-sdk` in Phase 3). Phase 1 does not need wasmtime.
Therefore:

- **Phase 1 MSRV floor:** 1.86 (driven by ratatui 0.30).
- **Phase 3 MSRV floor:** 1.92 (driven by wasmtime 44).

Resolution: pin Phase 1 workspace `rust-version = "1.86"`. When Phase 3 lands,
bump to `rust-version = "1.92"` (or whatever wasmtime's then-current MSRV
requires).

### Options under evaluation

- **Option A — pin workspace to 1.86 for Phase 1.** Aligns with the highest
  Phase 1 dep. Excludes wasmtime from Phase 1 (consistent with OQ-03 outcome).
- **Option B — pin workspace to 1.92 immediately.** Aligns with Phase 3 dep
  but locks Phase 1 to a much narrower toolchain set (Rust 1.92 was released
  ~April 2026 per the Rust 6-week cadence). Less contributor-friendly.
- **Option C — split MSRV per crate.** Each crate in the workspace declares
  its own `rust-version`. `monocle-plugin-sdk` declares 1.92; others declare
  1.86. Cargo respects per-crate MSRV in the workspace.

### Prior art evidence

- **tokio policy:** "rolling MSRV of at least 6 months, where the new Rust
  version must have been released at least six months ago." Source:
  `https://github.com/tokio-rs/tokio/pull/4457`. tokio 1.52 MSRV is 1.71.
- **wasmtime policy:** "Wasmtime supports the latest three stable releases of
  Rust, which means if the latest version is 1.72.0 then Wasmtime supports
  1.70.0, 1.71.0, and 1.72.0." Source:
  `https://docs.wasmtime.dev/contributing-coding-guidelines.html`.
  wasmtime 44.0.1 ships with MSRV 1.92 because that crate was released
  early May 2026 and the latest stable is 1.94. wasmtime's MSRV moves rapidly.
- **helix editor MSRV:** intentionally low to ease packaging. Specific 2026
  version not verified in search results. Reinforces "MSRV is a contributor
  experience knob, not a feature".
- **Per-crate MSRV in Rust workspaces:** Cargo 1.65+ supports per-crate
  `rust-version`. The workspace `rust-version` is the floor for *building the
  workspace*, but individual crates can declare higher MSRVs. Per
  `https://doc.rust-lang.org/cargo/reference/rust-version.html`.

### Implementation cost

- **A:** Workspace Cargo.toml sets `rust-version = "1.86"`. CI uses
  `rust-toolchain.toml` with `channel = "1.86"` plus a `MSRV` matrix that
  also tests on the latest stable.
- **B:** Workspace Cargo.toml sets `rust-version = "1.92"`. Same CI shape but
  the floor is much higher. Contributors on older systems (Ubuntu LTS, Debian
  stable) may not have 1.92 available without rustup.
- **C:** Per-crate `rust-version` declarations. Workspace `rust-version`
  set to 1.86. monocle-plugin-sdk crate sets `rust-version = "1.92"`. Building
  the workspace from scratch requires 1.92 because Cargo unions MSRV across
  workspace members during resolve. So C is operationally equivalent to B for
  Phase 3 onward.

### Operational implications

- **A:** Phase 1 ships against a reasonable toolchain. Phase 3 PR bumps MSRV
  in the workspace Cargo.toml; CI matrix is updated at that time. Existing
  contributors get a few months' notice via deprecation issue.
- **B:** Immediately raises the bar. Contributors must rustup to 1.92. Smaller
  contributor pool for Phase 1 features.
- **C:** Resolves correctly only after Phase 3; before then, per-crate MSRV
  costs the workspace Cargo.toml a hidden complexity for no near-term gain.

### Recommended default

**Option A (pin Phase 1 workspace `rust-version = "1.86"`; Phase 3 bumps to
match wasmtime's then-current MSRV).**

Rationale: wasmtime is deferred to Phase 3 per OQ-03. There is no reason for
Phase 1 to inherit Phase 3's MSRV. 1.86 is the floor driven by ratatui 0.30,
which is non-negotiable for the TUI. Add a CI job that runs on the pinned
MSRV (1.86) and a separate job on stable to catch regressions early.

When Phase 3 lands, the workspace MSRV bumps to whatever wasmtime's then-current
MSRV requires (1.92 today; possibly higher by Phase 3 ship date given wasmtime's
"latest three stable" policy). Document the bump in CHANGELOG.

### Confidence

**HIGH.** MSRV values are mechanically verified against crates.io. Phase
boundaries are explicit in the brief. The architect's only choice is whether to
inherit the highest Phase 3 MSRV in Phase 1 — and the answer is no.

### Architect action

In the workspace `Cargo.toml`:
```toml
[workspace.package]
rust-version = "1.86"
edition = "2024"  # 2024 edition stabilized in Rust 1.85; available in 1.86
```
Add `rust-toolchain.toml` with `channel = "1.86"` for local development
reproducibility. Add CI matrix entries for `msrv` (1.86), `stable` (latest),
`beta` (next stable). When Phase 3 lands, raise to `"1.92"` (or current
wasmtime MSRV) in a single PR with a paired CHANGELOG entry.

---

## Cross-OQ themes

Several architectural decisions appear across multiple OQs. The architect should
treat these as unified subsystems rather than per-question fragments.

### Theme 1 — The lock-file pattern (OQ-01, OQ-04, OQ-10)

The same lock-file mechanism solves three problems:
- OQ-01 (auto-start): the TUI probes the lock file to detect a running daemon.
- OQ-04 (port binding): the daemon writes the OS-assigned port + token to the
  lock file.
- OQ-10 (path location): the lock file lives at the canonical runtime path
  (`runtime_dir()` if available, fallback chain otherwise).

**Unified recommendation:** define `monocle-runtime::lock::LockFile` as a single
type owning:
- Path: `MonoclePaths::runtime.join("daemon.json")`.
- Schema: `{pid: u32, port: u16, auth_token: String, transport: "http", app:
  "monocle", contract_version: u32}`.
- Mode: `0o600` enforced post-rename.
- Writer: atomic via `tempfile::persist`.
- Reader: includes PID-liveness probe (`nix::unistd::Pid::kill(0)`).

This single subsystem feeds every OQ that touches port discovery, auto-start, or
restart resilience. Verbatim port from
`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md`
§"Hook protocol" (the canonical schema) and §"MCP server vs MCP registry" (the
PID-liveness pattern).

### Theme 2 — Atomic writes for all user-facing files (OQ-02, OQ-05, OQ-06, OQ-10)

Five files are user-readable / restart-critical:
- `daemon.json` (OQ-04/10 lock file).
- `hooks-settings.json` (OQ-02 hook tmpfile).
- `state.json` (OQ-05 sticky-per-project).
- `events.jsonl` (OQ-06 hook event log).
- `config.json` (existing in brief).

All must use `tempfile::persist` for atomic replacement. The brief mandates this
already (brief line 99, line 246). This theme reinforces that the
`monocle-config` crate should expose a single `fn atomic_write<T: Serialize>(path:
&Path, value: &T) -> Result<()>` helper, not a five-way scatter of inline temp-
file logic.

### Theme 3 — Phase 3 / Phase 4 forward-compatibility seams (OQ-03, OQ-07, OQ-08)

Phase 1 architecture must reserve seams for:
- WASM plugin loading (OQ-03): trait shapes constrained for WASM ABI portability.
- Cross-host federation (OQ-07): protobuf wire types defined in v1; russh
  transport plug-in in Phase 4.
- High-throughput transport (OQ-08): `Transport` trait abstraction with UDS as
  the only v1 implementation.

**Unified recommendation:** these three seams are *trait-shape constraints* on
`EngineModule`, `FactoryAdapter`, and `Transport`. The architect's most
load-bearing decision is to write down those constraints in `monocle-core`'s
module rustdoc:
- No generic methods.
- No associated types beyond static `&'static str`.
- All cross-cutting types live in `monocle-proto` (prost).
- Async via `Box<dyn Future<...>>`, not `impl Future` (which is incompatible
  with `dyn` trait objects and would block WASM plug-loading).

This single constraint set survives into Phase 3 and Phase 4 without breaking ABI
(per brief line 327 mandate).

### Theme 4 — XDG-aware path resolution (OQ-06, OQ-10, plus brief's existing config path)

`directories::ProjectDirs` is the canonical dispatcher. All paths in monocle
flow through `monocle-config::paths::MonoclePaths::discover()`. There is no
hand-rolled `home_dir().join(".monocle/...")` anywhere in the codebase. This
prevents per-OS path drift (the bug zellij avoids by using `ProjectDirs`,
`/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:383`).

---

## Sequencing recommendations

The 11 OQs have dependency relationships. The architect should resolve them in
this order:

1. **OQ-10 (lock file path)** — chooses where files live. Feeds every other OQ
   that touches the filesystem.
2. **OQ-04 (port binding)** — chooses OS-assigned + lock file. Locks in the
   lock-file schema needed by OQ-01 and OQ-02.
3. **OQ-02 (hook tmpfile)** — chooses shared per-runtimeDir at the path from
   OQ-10. Uses the auth token from OQ-04's lock file.
4. **OQ-01 (daemon auto-start)** — uses OQ-04's lock-file probe to decide
   whether to spawn. Cannot be designed before OQ-04 is settled.
5. **OQ-11 (MSRV target)** — choose 1.86 for Phase 1 once OQ-03 confirms
   wasmtime is deferred.
6. **OQ-03 (WASM v1 vs static)** — choose static. Confirms OQ-11's Phase 1
   floor and shapes the trait constraints for OQ-07/08.
7. **OQ-07 (cross-host scope)** — choose protobuf seams in v1. Establishes
   `monocle-proto` crate.
8. **OQ-08 (IPC channels)** — choose UDS only. Uses the `monocle-proto` types
   from OQ-07 on the wire.
9. **OQ-06 (event retention)** — choose hybrid ring + JSONL. Uses the
   `monocle-proto` types for JSONL records.
10. **OQ-05 (profile picker)** — choose sticky-per-project with override.
    Uses the state-store path from OQ-10.
11. **OQ-09 (rmcp stub)** — choose omit. No dependencies on other OQs.

Order rationale: filesystem and IPC plumbing (1-3) come first because every other
subsystem depends on them. Phase boundaries (4-7) come second because they
gate scope. Subsystem-specific choices (8-10) come last. OQ-09 is independent
and trivial.

---

## Open second-order questions surfaced by research

The research process surfaced four architectural questions not in the brief that
the architect must address during `/vsdd-factory:create-architecture`:

### SOQ-1 — Lock-file schema versioning strategy

The any-context lock file does NOT include a `contract_version` field
(`/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:184`).
zellij does
(`/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:96`,
`CLIENT_SERVER_CONTRACT_VERSION: usize = 1`). Monocle should include the version
in the lock file from day one (`contract_version: 1`) so a future schema
change (Phase 3 WASM plugin events; Phase 4 federation host_id) can be
detected by the hook script. Older hooks see an unknown version, fall through
to silent-drop or fail-closed per the per-hook policy (any-context BC-HOOK-018).

### SOQ-2 — Token rotation policy

The lock file contains an auth token. When the daemon restarts, the token
rotates (OQ-04 resolution writes a fresh token on each bind). But the
hooks-settings.json file written by OQ-02 captures a snapshot of the token at
write time. If the hooks-settings.json is written before lock-file write, and
the daemon restarts in between, the hooks have a stale token. Mitigation: the
write order must be (1) bind + lock-file-write + token-store, then (2)
hook-settings-write reads the token from the lock file. Architect must document
this ordering as a runtime invariant.

### SOQ-3 — How does the popup overlay survive daemon restart?

The brief specifies the VecDeque<PromptModal> overlay (brief lines 244-247)
survives Ctrl-\ hide/show cycles. But what about daemon restart? If the daemon
crashes while a permission prompt is pending, does the TUI keep the prompt
queued and replay the response on daemon reconnect, or does it drop the prompt?
The brief is silent. Recommendation: on daemon disconnect, the TUI displays a
"daemon disconnected" status and grays out the overlay; on reconnect, the TUI
clears the overlay (because the underlying Claude Code process has already
timed out and cannot accept a delayed response). Architect must spec this in
the AppMode state machine.

### SOQ-4 — Permission token model in Phase 1 absence of WASM

If wasmtime is deferred to Phase 3 (OQ-03 resolution), should the 17-permission
capability gate from zellij
(`/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:154-160`)
also be deferred? Recommendation: yes — Phase 1 has no untrusted code. The
permission gate is only meaningful for third-party plugins. Phase 1 has only
the static-bundle ClaudeCodeModule + VsddFactoryAdapter, both first-party.
The 17-permission enum should still be defined in `monocle-core::permissions`
(for forward compat with Phase 3) but its dispatcher (`check_permission`) is a
no-op until Phase 3 lands.

---

## Recommended default summary (one line per OQ)

| OQ | Question | Recommended default | Confidence |
|----|----------|---------------------|------------|
| OQ-01 | Daemon auto-start? | Hybrid: auto-start with `MONOCLE_NO_AUTOSTART=1` escape hatch (sccache pattern) | HIGH |
| OQ-02 | Hook tmpfile per-session or shared? | Shared per-runtimeDir, mode 0o600, atomic-replace (any-context verbatim) | HIGH |
| OQ-03 | WASM SDK in v1? | Static bundle v1; WASM SDK in Phase 3 as brief specifies | HIGH |
| OQ-04 | Fixed port or OS-assigned? | OS-assigned + lock-file discovery (any-context verbatim) | HIGH |
| OQ-05 | Profile picker vs sticky? | Sticky-per-project with Ctrl-P picker override | MEDIUM |
| OQ-06 | Event retention model? | Hybrid: RAM ring + async JSONL flush, 100MB × 5 rotation | HIGH |
| OQ-07 | Federation scope? | Protobuf seams in v1, russh transport in Phase 4 | HIGH |
| OQ-08 | IPC channels? | UDS only in v1; shared-mem deferred to Phase 4 transport variant | HIGH |
| OQ-09 | rmcp stub in v1? | Omit entirely; Phase 4 delivers real impl | HIGH |
| OQ-10 | Lock file path? | `directories::ProjectDirs::runtime_dir()` with state_dir → data_dir → ~/.monocle fallback | HIGH |
| OQ-11 | MSRV target? | Phase 1: `rust-version = "1.86"` (driven by ratatui 0.30); Phase 3 bumps to wasmtime's then-current MSRV (1.92 today) | HIGH |

### Highest MSRV found

**Rust 1.92.0** (wasmtime 44.0.1, verified at
`https://crates.io/api/v1/crates/wasmtime/44.0.1` 2026-05-12). This is the
Phase 3 floor. The Phase 1 floor is **Rust 1.86.0** (ratatui 0.30.0, verified
at `https://crates.io/api/v1/crates/ratatui/0.30.0` 2026-05-12).

### Confidence summary

- HIGH: 10 OQs (OQ-01, OQ-02, OQ-03, OQ-04, OQ-06, OQ-07, OQ-08, OQ-09, OQ-10, OQ-11)
- MEDIUM: 1 OQ (OQ-05 — profile picker UX is opinionated rather than strongly
  evidenced; recommend a Phase 1 user-test)
- LOW: 0 OQs

---

## Research methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebFetch (crates.io API) | 26 | Per-crate MSRV verification 2026-05-12 |
| WebFetch (GitHub) | 2 | wasmtime workspace MSRV check; russh release notes review |
| WebFetch (Anthropic docs) | 1 | Claude Code `--settings` flag documented behavior |
| WebFetch (Wikipedia) | 1 | IANA port 2748-2750 conflict probe |
| WebSearch | 9 | sccache auto-spawn pattern; russh API changes; wasmtime binary size; in-memory ring vs JSONL; interprocess throughput; directories XDG; daemon spawn pattern; Claude Code settings race; rmcp stub |
| Context7 resolve-library-id | 2 | wasmtime, rmcp lookup |
| Context7 query-docs | 2 | rmcp ServerHandler trait shape; wasmtime feature flags / instantiation cost |
| Local file reads | 6 | product-brief.md, vision-synthesis.md, any-context synthesis, zellij synthesis, lazygit synthesis, claude-squad synthesis |
| Local grep | 6 | Targeted extraction from large synthesis files |
| Perplexity (search/research/reason) | 0 | MCP tool was not available in this environment |
| Tavily | 0 | Not used |
| Training data | ~3 areas | Rust toolchain release cadence (6-week), generic OS daemon-spawn patterns, generic protobuf binary footprint understanding — all cross-validated against either Context7 or web sources |

**Total external tool calls:** 55 (26 crates.io + 2 github + 1 anthropic +
1 wikipedia + 9 web search + 2 context7 resolve + 2 context7 query + 6 file
reads + 6 grep)

**Training data reliance:** **low** — every load-bearing claim (MSRV numbers,
behavioral patterns, file paths, code citations) is sourced from either a live
web fetch (crates.io, github, official docs) or a file:line citation in the
gene-source synthesis files. The single area where training data was the
primary input (Rust 6-week release cadence framing for the MSRV bump narrative)
was cross-validated against the tokio rolling-MSRV policy URL and the wasmtime
"latest three stable" docs URL.

---

## Citations index (URLs and file:line)

### Crates.io API (MSRV verification, 2026-05-12)
- `https://crates.io/api/v1/crates/wasmtime/44.0.1` — wasmtime 44.0.1 rust_version: 1.92.0
- `https://crates.io/api/v1/crates/ratatui/0.30.0` — ratatui 0.30.0 rust_version: 1.86.0
- `https://crates.io/api/v1/crates/russh/0.60.2` — russh 0.60.2 rust_version: 1.85
- `https://crates.io/api/v1/crates/axum/0.8.9` — axum 0.8.9 rust_version: 1.80
- `https://crates.io/api/v1/crates/tokio/1.52.0` — tokio 1.52 rust_version: 1.71
- `https://crates.io/api/v1/crates/notify/8.0.0` — notify 8 rust_version: 1.77
- `https://crates.io/api/v1/crates/interprocess/2.4.0` — interprocess 2.4 rust_version: 1.75
- `https://crates.io/api/v1/crates/prost/0.14.0` — prost 0.14 rust_version: 1.71.1
- (additional per-crate entries in OQ-11 table)

### Documentation and ecosystem
- `https://github.com/bytecodealliance/wasmtime/blob/main/docs/examples-minimal.md` — wasmtime minimum embedding 2.1MB; cranelift/winch features
- `https://github.com/bytecodealliance/wasmtime/blob/main/docs/examples-pre-compiling-wasm.md` — wasmtime pre-compiled module loading; startup performance
- `https://docs.wasmtime.dev/contributing-coding-guidelines.html` — wasmtime "latest three stable" MSRV policy
- `https://github.com/Eugeny/russh/releases` — russh 0.45-0.60 release notes; API changes per minor
- `https://docs.rs/rmcp/latest/rmcp/handler/server/trait.ServerHandler.html` — rmcp ServerHandler trait shape (26 methods)
- `https://docs.rs/rmcp/latest/rmcp/service/fn.serve_server.html` — rmcp serve_server entrypoint
- `https://github.com/mozilla/sccache` — sccache auto-spawn daemon pattern; SCCACHE_STARTUP_NOTIFY env var
- `https://docs.rs/sccache/latest/src/sccache/commands.rs.html` — sccache client server startup notify
- `https://code.claude.com/docs/en/settings` — Claude Code `--settings` flag precedence
- `https://github.com/eyaltoledano/claude-task-master/issues/1567` — Claude Code concurrent process race (background context)
- `https://docs.rs/directories` — directories 6 API; runtime_dir() None on macOS
- `https://docs.rs/axum/latest/axum/` — axum bind pattern
- `https://docs.rs/interprocess/latest/interprocess/` — interprocess 2.4 API
- `https://github.com/redhat-performance/rusty-comms` — Rust IPC benchmark suite
- `https://oneuptime.com/blog/post/2026-01-25-high-throughput-data-ingestion-pipeline-rust/view` — high-throughput data ingestion Rust pattern
- `https://github.com/tokio-rs/tokio/pull/4457` — tokio rolling MSRV policy
- `https://en.wikipedia.org/wiki/List_of_TCP_and_UDP_port_numbers` — IANA port assignments (2748-2758 gap)

### Gene-source synthesis files (absolute paths, file:line)
- `/Users/jmagady/Dev/monocle/.factory/specs/product-brief.md:273-285` — OQ definitions
- `/Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md:376-382` — Phase Plan
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:24` — lock file mode 0o600 verified
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:150` — daemon.json runtime path
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:152-174` — Hook protocol canonical schema
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:184` — MCP server vs registry; lock file schema
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md:198-206` — Broker dispatch buffer sizes
- `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:91-97` — Client/server IPC model; UDS pattern
- `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:138-186` — Plugin SDK (WASM)
- `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:154-160` — 17-permission capability model
- `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:316-329` — Session persistence layout
- `/Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md:402-417` — Risk register
- `/Users/jmagady/Dev/monocle/.factory/semport/lazygit/lazygit-pass-8-final-synthesis.md:303-304` — Focus-trigger reload
- `/Users/jmagady/Dev/monocle/.factory/semport/lazygit/lazygit-pass-8-final-synthesis.md:377-392` — XDG discovery + state.yml location
- `/Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md:16-50` — EngineModule contract
- `/Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md:214-220` — Profile launcher abstraction
- `/Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md:265-273` — ProfilePicker UX
- `/Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md:255-256` — config.json vs state.json split
- `/Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md:437` — Default-first ordering convention
- `/Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md:315-365` — Factory project discriminator + detection algorithm

---

## End of document
