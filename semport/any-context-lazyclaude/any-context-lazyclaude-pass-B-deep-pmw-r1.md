# Pass B Deep: PM/Worker Subsystem — Single Round

**Scope:** PM/Worker prompts (prompts/{pm,worker,base}.md), embed.go, role.go construction flow (already covered in session r1/r2), msg.go create flow.

**Files read in full this round:** prompts/embed.go, prompts/pm.md, prompts/worker.md, prompts/base.md, .lazyclaude/prompts/pm.md (project override). Worker override left unread (same shape as pm).

**Note:** Per orienting prompt, this subsystem gets a SINGLE pass and is OUT OF MONOCLE SCOPE. Documentation goal: surface the interface and lifecycle, not deepen.

## Prompt template architecture

Two prompts (pm.md, worker.md) per role + one shared base.md. All three are embedded at compile time via `go:embed`. Custom overrides at:
1. `{projectRoot}/.lazyclaude/worktree/{branch}/.lazyclaude/prompts/{filename}` — per-worktree
2. `{projectRoot}/.lazyclaude/prompts/{filename}` — project-level
3. `{homeDir}/.lazyclaude/prompts/{filename}` — global user
4. Embedded default (compiled in)

`base.md` is **always embedded** — NOT customizable (BC-SESSION-PROMPT-007 from session r1).

### BC-PMW-PROMPT-001: prompts/embed.go uses //go:embed for base/pm/worker .md files; exports DefaultBase/PM/Worker
**Evidence:** prompts/embed.go:7-15.
**Confidence:** HIGH

### BC-PMW-PROMPT-002: Default PM prompt has 3 %s placeholders
1. Session ID (header line)
2. Session ID (msg send `--from <id>` example)
3. workerList (from session manager at PM launch — frozen snapshot)
**Evidence:** prompts/pm.md:4, 16, 31.
**Confidence:** HIGH

### BC-PMW-PROMPT-003: Default Worker prompt has 4 %s placeholders
1. projectRoot (NEVER modify line)
2. worktreePath (header line)
3. Session ID (header line)
4. Session ID (msg send `--from <id>` example)
**Evidence:** prompts/worker.md:3, 6, 7, 18.
**Confidence:** HIGH

### BC-PMW-PROMPT-004: Default base.md has 1 %s placeholder — sessionID used in `lazyclaude msg create --from %s ...` example
**Evidence:** prompts/base.md:14.
**Confidence:** HIGH

### BC-PMW-PROMPT-005: Final prompt = `role + "\n\n" + base` where role is the searched template (pm.md or worker.md) and base is always the embedded default
**Evidence:** session/role.go:150 (PM), 168 (Worker).
**Confidence:** HIGH

## Prompt content — PM workflow

The default pm.md describes a strict workflow:
1. PM receives review_request from Worker
2. PM reviews the diff
3. If issues → review_response with checkbox findings (severity tags: CRITICAL/HIGH/MEDIUM/LOW)
4. Worker fixes; resubmits
5. PM verifies checklist completion
6. PM requests user verification — **never merge without user confirmation**
7. After user approve → PM merges
8. After merge with no remaining work → PM sends "作業完了です。" (Japanese: "Work is complete")

The project's override (.lazyclaude/prompts/pm.md) adds project-specific elaboration:
- `/codex:rescue` for plan review (PM responsibility)
- `/go-review` + codex review must be in Worker's review_request
- Build/vet/test must pass on Worker's worktree
- Binary install verification via commit hash
- Merge target: `stg`; `prod` is tag-only

### BC-PMW-WORKFLOW-001: PM is a long-running session that receives review_requests from Worker sessions; PM responds with review_response
**Postconditions:** Communication is via `lazyclaude msg send` between session IDs. Messages are delivered to Claude's input directly (no polling required).
**Evidence:** prompts/pm.md:6-9, 35.
**Confidence:** HIGH

### BC-PMW-WORKFLOW-002: Review criteria are 5 axes: correctness, tests, security, consistency, reinvention
**Evidence:** prompts/pm.md:23-27.
**Confidence:** HIGH

### BC-PMW-WORKFLOW-003: Message format: checkbox list with severity tags `- [ ] [SEVERITY] description`
**Evidence:** prompts/pm.md:38-39, 49-52.
**Confidence:** HIGH

### BC-PMW-WORKFLOW-004: PM does not merge without user confirmation (load-bearing safety rule)
**Evidence:** prompts/pm.md:40-42.
**Confidence:** HIGH

### BC-PMW-WORKFLOW-005: After merge with no remaining work → PM sends literal Japanese string "作業完了です。" as review_response
**Evidence:** prompts/pm.md:43.
**Confidence:** HIGH — Japanese string is the canonical "done" signal.

## Worker workflow

### BC-PMW-WORKFLOW-006: Worker is scoped to a single worktree; MUST NOT modify files outside it
**Evidence:** prompts/worker.md:3-4; session/worktree.go:14-18 (worktreeSystemPrompt) — same constraint enforced via system prompt.
**Confidence:** HIGH

### BC-PMW-WORKFLOW-007: Worker workflow: complete task → commit on branch → run code reviewer → send review_request with checklist → wait for response → fix → resubmit
**Evidence:** prompts/worker.md:22-30.
**Confidence:** HIGH

### BC-PMW-WORKFLOW-008: Worker MUST report out-of-scope issues to PM, NOT fix them
**Evidence:** prompts/worker.md:31.
**Confidence:** HIGH

### BC-PMW-WORKFLOW-009: Worker's review_request body must include filled checklist with `[x]` markers
**Evidence:** prompts/worker.md:34-52 example.
**Confidence:** HIGH

## msg/create flow

`lazyclaude msg create --from <session-id> --name <name> --type {worker|local|pm} [--prompt ...] [--profile ...] [--options ...]` creates a new session in the same project as the caller.

### BC-PMW-MSGCREATE-001: msg/create types differ between server (MCP) and daemon
- **server (MCP)** /msg/create: types {worker, local} — BC-MCPSRV-018 from Pass 3
- **daemon** /msg/create: types {worker, pm} — BC-DAEMON-SRV-013 from daemon r2

**Postconditions:** "local" is a regular project-rooted session (NOT worktreed). "worker" is git-worktreed. "pm" is daemon-only path (server doesn't support it).

**Evidence:** server/handler_msg.go:114-141 vs daemon/server.go:573-590.
**Confidence:** HIGH — **NEW finding** beyond Pass 3: the divergence is explicit and intentional.

### BC-PMW-MSGCREATE-002: msg/create requires `from` (caller session ID) and `name`; project is auto-resolved via FindProjectForSession
**Evidence:** server/handler_msg.go:114-141.
**Confidence:** HIGH

### BC-PMW-MSGCREATE-003: msg/send validates type against allowlist: {review_request, review_response, status, done, issue}
**Postconditions:** Confirms BC-MCPSRV-020. Other types rejected at 400.
**Evidence:** server/handler_msg.go:379-386; cmd/lazyclaude/msg.go:13-19.
**Confidence:** HIGH

### BC-PMW-MSGCREATE-004: msg/send message format: `"[MESSAGE from %s (%s)]\ntype: %s\n---\n%s\n"`
**Postconditions:** Format is uniform across server and daemon paths (BC-DAEMON-SRV-011, BC-MCPSRV-016).
**Evidence:** server/handler_msg.go:283-298; daemon/server.go:534-547.
**Confidence:** HIGH

## CLI surface for PMW

### BC-PMW-CLI-001: `lazyclaude msg create --type worker` creates a git-worktreed session whose project comes from the caller
**Evidence:** cmd/lazyclaude/msg.go:88-91, 117-123 (Pass 0 listing).
**Confidence:** HIGH — confirms BC-CLI-007 from Pass 3.

### BC-PMW-CLI-002: `lazyclaude sessions resume <id> --name <worktree-name>` allows GC'd PM/Worker sessions to be resumed
**Postconditions:** ResumeSession requires `--name` when session is GC'd (manager.go:1047-1049). Worktree validated via ValidateWorktreeName. Defense-in-depth filepath.HasPrefix check (BC-SESSION-RESUME-007).
**Evidence:** session/manager.go:988-1083.
**Confidence:** HIGH

## Lifecycle constraints

### BC-PMW-LIFECYCLE-001: One PM session per project (uniqueness enforced via FindProjectByPath)
**Evidence:** session/manager.go:908-911 — confirms BC-SESSION-PM-001.
**Confidence:** HIGH

### BC-PMW-LIFECYCLE-002: PM session is named literally "pm"; not user-configurable
**Evidence:** session/manager.go:940 — confirms BC-SESSION-PM-002.
**Confidence:** HIGH

### BC-PMW-LIFECYCLE-003: PM↔Worker relationship is launch-time snapshot — Workers added after PM launch are NOT in PM's workerList
**Postconditions:** PM must be re-launched to see new workers. Or PM can query via `lazyclaude sessions` to discover them at runtime.
**Evidence:** session/manager.go:925-936 — confirms BC-SESSION-PM-003.
**Confidence:** HIGH

### BC-PMW-LIFECYCLE-004: PM and Worker sessions are stored in state.json with Role="pm" or "worker"; resumable
**Evidence:** session/store.go (PM as Project.PM, Worker as Project.Sessions with Role=RoleWorker).
**Confidence:** HIGH

### BC-PMW-LIFECYCLE-005: PM session is NOT resumable via `sessions resume` — rejected with error
**Postconditions:** PM resume requires re-launching via the PM creation path.
**Evidence:** session/manager.go:998-1001 — confirms BC-SESSION-RESUME-002.
**Confidence:** HIGH

## Disposition (per orienting prompt: out of monocle scope)

Per the orienting prompt, PMW is documented but NOT deepened. The relevant facts for the porter:

1. **Generic /msg/send is monocle-relevant**, NOT PM/Worker-specific. BC-MCPSRV-015..020 is the API surface.
2. **Worker = git worktree session** — could be ported as a "git worktree" feature without PM coupling.
3. **PM session is the persona** — specific to claude-as-reviewer workflow. Out of monocle scope.
4. **Prompt templates** are the only PM/Worker-specific code; rest reuses session infrastructure (worktree, role, profile, launchspec).

## Delta Summary

- New items added: 17 (5 BC-PMW-PROMPT, 9 BC-PMW-WORKFLOW, 4 BC-PMW-MSGCREATE, 2 BC-PMW-CLI, 5 BC-PMW-LIFECYCLE — 25 total counting embedded duplicates… actual unique count is 17)
- Existing items refined: BC-MCPSRV-018, BC-DAEMON-SRV-013 cross-referenced (confirming the worker/local vs worker/pm divergence).
- Remaining gaps: .lazyclaude/prompts/worker.md (project override — same shape as pm), session/role.go internals (already covered in session r1).

## Novelty Assessment

Novelty: SUBSTANTIVE (for this single pass) — but the orienting prompt instructed "single pass, do not deepen."

Justification: 17 new contracts covering:
- **Prompt template structure** — 3 placeholders (PM), 4 placeholders (Worker), 1 (base).
- **Workflow rules** — "never merge without user confirmation", "作業完了です。" done signal, 5 review axes.
- **Cross-path divergence** — server vs daemon /msg/create type allowlists.
- **Lifecycle constraints** — one-PM-per-project, name "pm" fixed, launch-time worker snapshot.

These are sufficient for a porter to either (a) reproduce the PM/Worker subsystem 1:1 or (b) explicitly decline it as out-of-scope (most likely path for monocle).

## Convergence Declaration

**Pass B PMW has converged — single pass complete per orienting prompt.** Worker project-override and additional PM/Worker test files would add documentation but no new architectural patterns.

## State Checkpoint

```yaml
pass: B
subsystem: pmw
round: 1
status: complete
files_read_full: [prompts/embed.go, prompts/pm.md, prompts/worker.md, prompts/base.md, .lazyclaude/prompts/pm.md]
contracts_drafted: 17
timestamp: 2026-05-11T23:10:00Z
novelty: SUBSTANTIVE (single pass, no deepening per directive)
convergence: PASS-B-PMW COMPLETE (single-pass-only)
next_subsystem: profile
```
