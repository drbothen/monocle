---
document_type: adr
level: L3
adr_id: "ADR-0009"
title: "Native Detached Session-Host Process Model for PTY Ownership"
status: accepted
producer: vsdd-factory:architect
phase: v1A-architecture-delta
version: "1.0.2"
timestamp: 2026-06-04T00:00:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - specs/product-brief.md
  - specs/research/embedded-pty-evaluation.md
  - semport/DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md
input-hash: "042e8f5"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# ADR-0009: Native Detached Session-Host Process Model for PTY Ownership

## Status

Accepted — 2026-06-03 (D-238 escalation response, Q-8 resolution)

## Context

The v2.1 vision (D-238 escalation) requires that sessions SURVIVE a graceful daemon-process
restart. The original v2.0 design placed PTY ownership inside the daemon process itself
(monocle-runtime's SessionManager). That model satisfies TUI-exit survival (CASE 1) because
the daemon remains running, but FAILS graceful-daemon-restart survival (CASE 2): when the
daemon process exits, all its child handles and PTY master file descriptors are closed by
the OS, SIGHUP-ing the Claude Code subprocesses.

D-238 (human gate, 2026-06-03) escalated CASE 2 to a hard v1A requirement: graceful daemon
restart MUST not kill running sessions. The human explicitly required a native implementation
preference (no external multiplexer as default) with an architect-surface-and-human-decide
gate if native proves infeasible.

### Three candidate approaches evaluated

**(a) Daemon double-fork / setsid per session — NOT adopted**

The POSIX double-fork pattern (fork → setsid → fork again) creates a fully detached process
that is re-parented to init/PID 1, immune to SIGHUP when the parent exits. This is the
traditional Unix daemon pattern.

*Problems for monocle:*
- `fork()` in a multi-threaded Tokio runtime is Async-signal-safe undefined behavior unless
  used with extreme care (`fork_safety` discipline: no allocations, no mutexes between fork
  and exec). Rust's standard library and Tokio do not guarantee fork-safety in async contexts.
- The double-fork pattern is inherently OS-specific (POSIX only — broken on Windows in the
  expected portability tier: macOS + Linux).
- A forked process still shares the daemon's file descriptors (including the UDS socket) at
  fork time; the child inherits FDs it must explicitly close, creating a leak surface.
- Re-attaching the daemon to a forked child over a per-session UDS socket requires the daemon
  to know the child's socket path BEFORE the fork — a chicken-and-egg bootstrapping problem.

**Verdict: infeasible cleanly. Rejected.**

**(b) Dedicated `monocle-session-host` binary per session — ADOPTED**

A separate binary `monocle-session-host` is spawned by the daemon for each session via
`std::process::Command::new("monocle-session-host")`. It is a clean, single-threaded (or
minimally-threaded) binary that:
- Opens the PTY pair via `portable-pty`
- Spawns the harness child (e.g., `claude --settings ...`) on the PTY slave
- Owns the PTY master read loop, vt100 parser, and scrollback buffer
- Exposes a per-session Unix domain socket at a deterministic path for daemon re-attachment
- Writes a `session-state.json` sidecar recording its own PID, socket path, and session metadata
- Lives as a child of init once the daemon explicitly calls `setsid()` on it before waiting

The binary is a simple supervisor — not a Tokio async server. It runs a blocking event loop:
PTY master read → forward bytes to attached clients over the per-session UDS, and conversely
receives bytes from the UDS and writes to the PTY master stdin.

**This is exactly the abduco/dtach model, but implemented natively in Rust as part of
the monocle crate family — no external dependency.**

**(c) External supervisor (tmux/abduco/dtach) — REJECTED as primary**

The embedded-pty-evaluation.md and vision both designate tmux control-mode as a documented
fallback ONLY. Using tmux or abduco as the primary session-host mechanism would introduce a
hard external runtime dependency (tmux/abduco must be installed), constrain the deployment
environments monocle targets (minimal containers, CI, systems without tmux), and violate the
no-tmux-as-primary constraint from the product brief. The human explicitly required native
as default with external supervisor only as architect-surfaced fallback.

**Verdict: tmux/abduco remain documented fallbacks; not the v1A primary.**

## Decision

**Adopt option (b): a dedicated `monocle-session-host` binary per session.**

Each session is supervised by a `monocle-session-host` process that:
1. Is spawned by the daemon via `std::process::Command` with a `setsid()` call via the
   `nix` crate before `spawn()` — making it a process group leader immune to SIGHUP when
   the daemon exits.
2. Owns `(pty_master, vt100::Parser, child_handle)` — the PTY ownership triple.
3. Listens on a per-session UDS socket at `<runtime_dir>/session-<uuid>.sock` (mode `0o600`).
4. Writes `<runtime_dir>/session-<uuid>.json` with its PID, socket path, child PID, started_at,
   project_root, harness_id, and profile_id (the session state sidecar).
5. On daemon re-attach: the daemon opens the per-session UDS, sends `Attach`, and the
   session-host streams the vt100 scrollback buffer then switches to live-streaming mode.

The daemon's `SessionManager` is redesigned from a PTY-owning in-process struct to a
session-host coordinator: it spawns and tracks session-host processes, re-discovers them on
startup from the session-state sidecars, and proxies PTY bytes to/from TUI clients over the
existing UDS IPC channel.

## Consequences

### Positive

- Graceful daemon restart (CASE 2) now survives: `monocle-session-host` processes are
  independent processes not in the daemon's process group; daemon restart does not SIGHUP them.
- Session-host is a minimal binary; its failure modes are independent of daemon complexity.
- The native UDS re-attach protocol is a clean, testable interface.
- No external runtime dependency.
- macOS and Linux support clean `setsid()` via `nix 0.30` (already pinned).

### Negative / costs

- A new crate/binary (`monocle-session-host`) is required in the workspace.
- The daemon startup re-discovery path (scan session-state sidecars, probe alive session-hosts,
  reconstruct SessionManager state) adds complexity to `daemon_start_sequence`.
- GC of orphaned session-state sidecars (session-host exited but sidecar not cleaned up) is
  required; implemented as a background sweep on daemon startup.
- The D-235 in-process SessionManager wiring must be reworked: PTY ownership moves out of
  monocle-runtime's process and into monocle-session-host.

### Hard crash boundary (CASE 3)

If `monocle-session-host` itself crashes, the PTY master closes and the Claude Code subprocess
receives SIGHUP. CASE 3 (hard crash) is the accepted v1A boundary. The session-host binary
is simple enough that crash risk is low; the launchd/systemd watchdog on the daemon is the
operational safety net.

## Cross-session UDS and POSIX contract

- Socket path: `<runtime_dir>/session-<uuid>.sock` — uuid derived from the session ID used
  on the daemon UDS wire (`session_id: String` type throughout the rest of the system).
- Socket mode: `0o600` — only the owning user.
- `setsid()` call site: `CommandExt::before_exec` (Linux/macOS) or `nix::unistd::setsid`
  invoked in a pre-exec hook. Confirmed safe in a pre-exec context (single-threaded at that
  point per Unix exec semantics). Reference: `nix::unistd::setsid()` docs; `CommandExt::pre_exec`
  safety contract.

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| Session-host binary not found on PATH at spawn time | Resolved at build time: `monocle-session-host` is built in the same workspace and packaged in the same release bundle as `monocle`; the daemon resolves its path relative to the daemon binary location using `std::env::current_exe()` parent |
| Race: daemon restarts before session-host socket is ready | Session-host writes socket path to sidecar ONLY after the UDS listener is bound; daemon startup reads sidecar and attempts connect with a 5s hard deadline (one attempt, no retry — per BC-2.08.004 Invariant 2) |
| Orphaned session-state sidecars | Daemon startup GC sweep: for each sidecar, verify PID liveness via `nix::sys::signal::kill(pid, None)`; delete stale sidecars |
| Multiple daemon instances probing same session-host | Lock-file liveness check (existing SOQ-2 invariant) prevents multiple live daemons |

## ADR Cross-References

- Supersedes: the v2.0 in-process DAEMON-OWNS-PTY persistence model in `domain-monocle-vision-synthesis.md` §Process Topology (now superseded by the session-host-owns-PTY model at v2.1).
- Extends: SS-daemon-lifecycle.md (daemon start sequence adds session-host discovery step).
- Extends: SS-ipc.md (adds per-session UDS protocol alongside existing daemon UDS).
- Requires: SS-08 Session Manager (new) — the SessionManager redesign required by this ADR AND the monocle-session-host binary spec (SS-session-manager.md).

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- ADR-0009 authored to resolve Q-8 (D-238 escalation) in the architecture delta.
- Native detached session-host process model selected; double-fork rejected (async-safety);
  external supervisor rejected as primary (no-tmux constraint).
- Q-8 VERDICT: **Native feasible at acceptable cost for v1A.** Design proceeds.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact, no prior chain entry).

## §Trace v1.0.2

**S17-001 — Risk table "5s backoff" ambiguity resolved** (2026-06-04T00:00:00Z):
- SUGGESTION (S17-001, Phase-1d Pass 17, fixed in-scope per production-grade):
  Risk table row "Race: daemon restarts before session-host socket is ready" used the phrase
  "5s backoff" which numerically matches the canonical connect timeout but could be misread
  as "retry with backoff up to 5s" — contradicting the canonical no-retry contract.
- Fixed: "5s backoff" → "5s hard deadline (one attempt, no retry — per BC-2.08.004 Invariant 2)"
  Exact phrasing mirrors BC-2.08.004 Invariant 2: "No exponential backoff or retry — one attempt,
  5s hard deadline." Numeric value (5s) unchanged.
- SE-16d PASS: 2026-06-04T00:00:00Z > chain high-water 2026-06-04T00:00:00Z (monotonic).

## §Trace v1.0.1

**I13-002 — ADR Cross-References SS-09 mis-anchor corrected** (2026-06-04T00:00:00Z):
- NORMATIVE (I13-002 IMPORTANT): §ADR Cross-References "Requires:" list corrected.
  The original two-line form was:
  - `Requires: SS-08 Session Manager (new) — the SessionManager redesign required by this ADR.`
  - `Requires: SS-09 Session Host (new) — the monocle-session-host binary spec.`
  The second line is a mis-anchor: "SS-09 Session Host" is not a registered subsystem.
  Per ARCH-INDEX Subsystem Registry: SS-09 = "Embedded PTY" (SS-embedded-pty.md); there is
  no "SS-09 Session Host" artifact. The monocle-session-host binary spec lives in
  SS-session-manager.md under SS-08 Session Manager.
  The two-line form also created a double-attribution: SS-08 was cited for SessionManager
  redesign while the binary spec (equally part of SS-08) was misattributed to SS-09.
  Corrected to a single line: `Requires: SS-08 Session Manager (new) — the SessionManager
  redesign required by this ADR AND the monocle-session-host binary spec (SS-session-manager.md).`
  - SE-17c BEFORE: `- Requires: SS-09 Session Host (new) — the monocle-session-host binary spec.`
  - SE-17c AFTER: line removed; SS-08 line updated to cover both the redesign and the binary spec.
- SE-16d PASS: 2026-06-04T00:00:00Z > chain high-water 2026-06-03T23:00:00Z (monotonic).
