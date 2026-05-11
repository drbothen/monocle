# Pass B Deep: PM/Worker Subsystem — Round 4 (Convergence check)

Targets from r3 remaining-gaps list: daemon token file permissions, `LAZYCLAUDE_SESSION_ID` consumer audit, miscellaneous audit items. Expectation per r3: this round confirms NITPICK and closes the subsystem.

## Files spot-checked this round

- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/daemon/server.go` (writeDaemonInfo lines 753-767)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/daemon/lifecycle.go` (daemon.json read path)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/manager.go` (line 855 — LAZYCLAUDE_SESSION_ID emission)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/manager_test.go`, `launchspec_test.go` (LAZYCLAUDE_SESSION_ID assertions)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/store.go` (WindowName, line 62-69)

Exhaustive grep for `LAZYCLAUDE_SESSION_ID` across the entire codebase returns 4 matches: one writer (manager.go:855), two test asserts (manager_test.go, launchspec_test.go), zero Go readers. Confirms r3 BC-PMW-HOOKS-004.

## Minor findings (NITPICK class)

#### BC-PMW-MISC-001: Daemon writes `daemon.json` at `0o600` and `daemon.json` directory at `0o700`
**Evidence:** daemon/server.go:758, 766. Standard Unix file-permission hygiene for tokens. No anomalies.
**Confidence:** HIGH
**Novelty assessment:** NITPICK — confirms expected behavior; doesn't change the model.

#### BC-PMW-MISC-002: `LAZYCLAUDE_SESSION_ID` is set in subprocess env but has NO in-tree consumer
**Postconditions:** Exhaustive grep across the entire Go codebase finds:
- Writer: `manager.go:855` (sets env var).
- Test asserts: `manager_test.go:860, 874`, `launchspec_test.go:257` (verifies it is set).
- Readers: **zero in Go code.**

The env var is intended for consumption by tools running inside the Claude Code subprocess (slash commands, plugins, Worker-side codex review scripts). Since no in-tree Go code reads it, it cannot be characterized further without examining the codex plugin or any user-shipped `.claude/commands/*` script.
**Evidence:** Grep search summary above.
**Confidence:** HIGH
**Novelty assessment:** NITPICK — confirms r3 BC-PMW-HOOKS-004.

#### BC-PMW-MISC-003: Daemon lifecycle reads `/tmp/lazyclaude-$USER/daemon.json` via SSH `cat` — token transits the SSH connection in plaintext within the encrypted tunnel
**Postconditions:** `daemon/lifecycle.go:34, 65` use `cat /tmp/lazyclaude-$(whoami)/daemon.json` over SSH. The SSH connection encrypts the wire transport; the token is plaintext over an encrypted channel. Standard remote-daemon-discovery pattern. **No defect.**
**Evidence:** lifecycle.go:34, 65.
**Confidence:** HIGH
**Novelty assessment:** NITPICK.

#### BC-PMW-MISC-004: WindowName 8-char truncation collision space is 16^8 ≈ 4.3 billion
**Postconditions:** `WindowName()` (store.go:62-69) returns `"lc-" + ID[:8]` (or full ID if < 8 chars, which is impossible for UUIDs). UUIDv4 first 8 hex chars give a collision space of ~4.3e9. Birthday-paradox 50% collision ≈ √(4.3e9) ≈ 65,000 concurrent sessions per project. **Practical concern: zero.**
**Evidence:** store.go:62-69 + UUID v4 properties (uuid.New() is v4 per the google/uuid library).
**Confidence:** HIGH
**Novelty assessment:** NITPICK — sanity check completes.

#### BC-PMW-MISC-005: Manager.mu serialization means PM creation, Worker creation, Resume, and Delete cannot run concurrently
**Postconditions:** All PM/Worker create paths and ResumeSession/Delete acquire `m.mu.Lock()`. Under high concurrent load (e.g. CI fanning out workers), throughput is bounded by single-threaded session-creation. Each create involves git worktree add, tmux window creation, file I/O for the launcher script — likely 100-500ms wall time. So practical throughput: 2-10 worker spawns per second per project.
**Evidence:** manager.go:227-228 (Create), 333-334 (createWorktreeSession), 906-907 (CreatePMSessionOpts), 989-990 (ResumeSession), 741-742 (Delete) — all `m.mu.Lock()`.
**Confidence:** HIGH
**Novelty assessment:** NITPICK — known property of single-mutex managers.

#### BC-PMW-MISC-006: PM session's worker-list snapshot is unsorted (whatever order `p.Sessions` happens to be in)
**Postconditions:** `manager.go:925-933` iterates `p.Sessions` and `Sprintf("- %s (id=%s, path=%s)", s.Name, s.ID, s.Path)`. Ordering is insertion order (no sort, no stable filter). PM's prompt sees workers in creation-time order.
**Evidence:** manager.go:925-933.
**Confidence:** HIGH
**Novelty assessment:** NITPICK.

## Delta Summary (round 4)

- New BC contracts drafted: 6 (all NITPICK class).
- Existing items refined: BC-PMW-HOOKS-004 (LAZYCLAUDE_SESSION_ID confirmed dead var in Go), BC-PMW-SPAWN-001 (mu serialization throughput characterization), BC-PMW-LIFECYCLE-003 (workerList ordering).
- Remaining gaps after this round: **none architectural.** External-tool consumers of `LAZYCLAUDE_SESSION_ID` are outside the subsystem boundary.

## Novelty Assessment

Novelty: **NITPICK**

Justification: All 6 findings are refinements that confirm existing model facts or add boundary-condition detail. None changes how a porter would spec the system:
- File permissions: standard Unix hygiene, no surprise.
- `LAZYCLAUDE_SESSION_ID`: confirmed dead in Go, deferred to plugin layer (out of subsystem scope).
- SSH transport of token: standard discovery pattern.
- 8-char window name collision: practically infinite headroom.
- mu serialization throughput: known property already implied by `m.mu.Lock()` in r2 BC-PMW-SPAWN-001.
- Worker-list ordering: insertion order is documented.

Removing this round's findings does NOT change how a porter would spec the system. The model is complete.

## Convergence Declaration

**Pass B PMW has CONVERGED.** Rounds r1 (shallow), r2 (full deep), r3 (cross-subsystem deep), r4 (boundary refinements) collectively produced ~93 unique behavioral contracts covering:
- Persona layer (prompts, role, workflow): BC-PMW-PROMPT-001..011, BC-PMW-PERSONA-001..006.
- Worker spawn / worktree lifecycle: BC-PMW-SPAWN-001..005, BC-PMW-WORKTREE-001..006.
- /msg/* bus primitive: BC-PMW-MSGAPI-001, BC-PMW-MSG-DIV-001..002, BC-PMW-MSG-SAFETY-001..003, BC-PMW-MSG-DELIVERY-001..005, BC-PMW-MSG-AUTH-001..002.
- Topology & failure modes: BC-PMW-TOPO-001..003, BC-PMW-FAIL-001..009.
- Cross-cutting (SSH remote, hooks, GUI/daemon): BC-PMW-REMOTE-001..006, BC-PMW-HOOKS-001..006, BC-PMW-GUI-DAEMON-001..003.
- Divergence: BC-PMW-DIV-FULL-001 (three session-create endpoints, three auth headers).
- Tests + misc: BC-PMW-TEST-001..005, BC-PMW-MISC-001..006.
- Lifecycle + CLI + lifecycle from r1: BC-PMW-LIFECYCLE-001..005, BC-PMW-CLI-001..002, BC-PMW-WORKFLOW-001..009, BC-PMW-MSGCREATE-001..004 (refined into DIV-FULL-001).

## P0/P1 P2 P3 final summary (for monocle dispositioning)

The user noted that the bus primitive (Layer 2) is retained for monocle; the persona (Layer 1) is leave-behind. Findings ranked by relevance:

**Monocle MUST address if retaining the daemon `/msg/*` route:**
- **P1 BC-PMW-MSG-SAFETY-001:** Daemon `/msg/send` accepts arbitrary `type` strings → prompt-injection-via-newline.
- **P1 BC-PMW-MSG-SAFETY-002:** Daemon `/msg/send` accepts 1MB bodies (100× server limit).

**Monocle SHOULD consider (P2):**
- **BC-PMW-MSG-DIV-002:** Three auth header conventions (X-Auth-Token, X-Daemon-Authorization, X-Claude-Code-Ide-Authorization) — unify.
- **BC-PMW-MSG-AUTH-002:** No cross-check of `req.From` against caller identity → token holder can spoof any sender. Single-trust-domain model assumed; document or enforce.
- **BC-PMW-MSG-SAFETY-003:** Sender name newline-injection (low practical reach).
- **BC-PMW-MSG-DELIVERY-005:** No idempotency / dedup.
- **BC-PMW-REMOTE-006:** Daemon `/msg/*` has no production callers today, but the wiring exists.

**Monocle MAY consider (P3):**
- **BC-PMW-WORKTREE-003:** No automated worktree cleanup → disk accumulation.
- **BC-PMW-PERSONA-006:** `BuildWorktreePrompt` is dead code.
- **BC-PMW-PROMPT-011:** Custom prompt placeholder mismatch silent malform.

**Persona layer (Leave behind per directive):**
- All BC-PMW-PROMPT, BC-PMW-PERSONA, and BC-PMW-WORKFLOW contracts.
- `BuildPMPrompt`, `BuildWorkerPrompt`, Role enum, project-override prompt files.
- The `P` keybind and `ActionStartPMSession`.

**Bus-primitive layer (Retain in monocle, with P1 fixes applied):**
- `/msg/send` (server path), `/msg/sessions`, `/msg/resume` (sans Role-specific rejection).
- Generic worktree-session creation (`CreateWorktreeOpts` with Role removed).
- Session ID + tmux window naming scheme.
- Self-deleting launcher script pattern (manager.go:671-737) — useful for any tmux-attached subprocess.
- Lock-file-based server discovery (discover.go).

## State Checkpoint

```yaml
pass: B
subsystem: pmw
round: 4
status: complete
files_spot_checked:
  - internal/daemon/server.go (writeDaemonInfo, lines 753-767)
  - internal/daemon/lifecycle.go (read path)
  - internal/session/manager.go:855 (LAZYCLAUDE_SESSION_ID writer)
  - internal/session/manager_test.go, launchspec_test.go (test asserts)
  - internal/session/store.go (WindowName)
contracts_drafted_this_round: 6 (all NITPICK class)
contracts_total_after_r4: 93 (17 r1 + 50 r2 + 20 r3 + 6 r4)
timestamp: 2026-05-12T00:05:00Z
novelty: NITPICK
convergence: PASS-B-PMW CONVERGED (4 rounds total: r1 shallow + r2 substantive + r3 substantive + r4 nitpick)
next_subsystem: (per orchestration plan)
```
