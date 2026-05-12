# Pass 7 (Deepening): Daemon + Web — claude-squad

## Scope

Verify the daemon model (the user wanted "what's IPC?") and definitively classify the web/ directory.

## Daemon Subsystem — Verified Full Picture

### Files

- `/daemon/daemon.go` — 167 LOC, all of RunDaemon, LaunchDaemon, StopDaemon
- `/daemon/daemon_unix.go` — 14 LOC, `getSysProcAttr` returning `Setsid: true`
- `/daemon/daemon_windows.go` — 15 LOC, `getSysProcAttr` with `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`

### Spawn Mechanism (`LaunchDaemon`)

```
execPath = os.Executable()                        // path of the running cs binary
cmd = exec.Command(execPath, "--daemon")          // re-invoke self
cmd.Stdin/Stdout/Stderr = nil                     // detach from parent's std streams
cmd.SysProcAttr = getSysProcAttr()                // Setsid on unix
cmd.Start()                                       // non-blocking
WriteFile("~/.claude-squad/daemon.pid", pid)
return (parent does NOT wait)
```

`--daemon` is a hidden flag (`MarkHidden`). Only the program itself invokes it.

### Daemon Loop (`RunDaemon`)

```
log.Initialize(daemon=true)                       // prefixes logs with "[DAEMON]"
state = config.LoadState()
storage = session.NewStorage(state)
instances = storage.LoadInstances()
for inst in instances: inst.AutoYes = true        // forced on

pollInterval = cfg.DaemonPollInterval ms          // default 1000
everyN = log.NewEvery(60 * time.Second)           // rate-limit per-iteration error logs

go {
    ticker = NewTimer(pollInterval)
    loop:
        for inst in instances:
            if inst.Started() && !inst.Paused():
                _, hasPrompt = inst.HasUpdated()  // captures pane, hash-checks
                if hasPrompt:
                    inst.TapEnter()
                    inst.UpdateDiffStats()        // updates inst.diffStats in place
        
        select {
        case <-stopCh: return
        default:
        }
        <-ticker.C
        ticker.Reset(pollInterval)
}

sigChan = SIGINT | SIGTERM
<-sigChan                                          // block
close(stopCh)
wg.Wait()
storage.SaveInstances(instances)
```

**Notable:** the daemon polls `HasUpdated` (which captures pane + hash-compares) every interval. Each capture is `tmux capture-pane -p -e -J -t <name>` — a tmux subprocess invocation. For 10 instances on a 1s interval, that's 10 tmux invocations per second. Not free but manageable.

**Stop mechanism:** SIGINT/SIGTERM. The parent app's `StopDaemon` does `proc.Kill()` which sends SIGKILL — so daemon doesn't get to save instances on parent-initiated stop. The daemon's signal handler only catches the SIGINT/SIGTERM cases for user-direct stop.

### IPC — Verified None

The daemon and the TUI share state via:
- **state.json on disk** (instances)
- **PID file** (daemon presence)
- **tmux server** (the actual agent processes)

There is **no socket**, **no pipe**, **no shared memory**, **no signaling beyond Kill**. The daemon doesn't notify the TUI of anything; the TUI doesn't notify the daemon of anything. They cooperate by being eventually consistent through tmux + filesystem.

Specifically: if the TUI's autoyes was just enabled and instances list changed, the daemon won't see new instances until it's restarted (or instances become Paused → not polled). The daemon's `instances` slice is fixed at startup. This is **another fragility**.

### Stale-PID Behavior

`StopDaemon` (`/daemon/daemon.go:131-167`):
1. Read PID file (no file → return nil)
2. `os.FindProcess(pid)` — on Unix this never fails; on Windows it might
3. `proc.Kill()` — sends SIGKILL on unix
4. Delete PID file

If the PID file points to a stale PID:
- Unix: `os.FindProcess` succeeds; `proc.Kill` returns ESRCH; code returns the wrapped error and the PID file is NOT cleaned up
- This means the next `cs` launch tries to kill a non-existent process and fails

The defensive fix would be `signal(syscall.Signal(0))` first to check liveness, but it's not present.

### Cross-OS Differences

- Unix: `Setsid: true` puts the daemon in its own session, untying it from the parent's controlling terminal so it survives parent exit
- Windows: `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP` — child has no console, can't be signaled by parent's Ctrl+C

## Web Subsystem — Verified Classification

`/web/` contains a Next.js 15.3.2 + React 19 marketing site. Concrete contents:

| File | Purpose |
|------|---------|
| `package.json` | declares: `react@19`, `react-dom@19`, `next@15.3.2`, devDeps for TypeScript/ESLint |
| `next.config.ts` | empty/default config |
| `eslint.config.mjs` | ESLint defaults |
| `tsconfig.json` | TS config |
| `src/app/layout.tsx` | (not read, but typical Next.js root layout) |
| `src/app/page.tsx` | The marketing homepage — `<h1>claude squad</h1>`, install instructions, demo video, feature list |
| `src/app/components/CopyButton.tsx` | Click-to-copy `brew install` snippet |
| `src/app/components/ThemeToggle.tsx` | Dark/light theme toggle |
| `src/app/globals.css` | Theme variables |
| `src/app/page.module.css` | Page styling |
| `public/*.svg` | Vercel template SVGs (file, globe, vercel, window — leftovers from create-next-app) |

**Definitive verdict:** `/web/` is the source for the `smtg-ai.github.io/claude-squad` GitHub Pages site (the URL is in the README's first line as `https://smtg-ai.github.io/claude-squad/`). It exists to give the project a homepage.

**It is NOT:**
- An API server
- A control plane for the Go binary
- A web UI for managing instances remotely
- Anything the Go binary communicates with at runtime

It compiles and deploys completely separately. There is no Go ↔ Next bridge in the codebase. The Vercel template SVGs in `public/` confirm this is a `create-next-app` scaffolding that was lightly customized into a landing page.

**Implication for monocle:** ignore `/web/` entirely. It's not architecturally interesting. If monocle wants a marketing site, design separately. claude-squad's choice to colocate the site in the same repo is a minor anti-pattern (mixes Go monorepo with Node ecosystem) but harmless.

## Daemon + Web Together — No Coupling

The web site does not call the daemon. The daemon does not serve HTTP. There's no admin port, no health endpoint, nothing. Daemon is purely local-process to local-filesystem.

## Delta Summary

- New items added: full daemon spawn/loop trace; the daemon-stops-with-SIGKILL detail; the "daemon sees instances frozen at startup" fragility; stale-PID bug; comprehensive web/ inventory confirming it's a Next.js marketing site
- Existing items refined: daemon IPC = "none" confirmed at code level
- Remaining gaps: none

## Novelty Assessment

Novelty: **NITPICK**

Justification: The broad sweep correctly identified the daemon as a polling autoyes loop with PID-file presence detection, and the web/ as not-relevant-to-the-Go-binary. This round verified, no model change.

## Convergence Declaration

Daemon + Web deepening has converged.

## State Checkpoint

```yaml
pass: 7
subsystem: daemon-web
round: 1
status: complete
novelty: NITPICK
timestamp: 2026-05-11T19:55:00Z
```
