---
document_type: market-intelligence-assessment
level: L1
version: "1.0"
status: complete
producer: business-analyst
phase: pre-phase-1-market-gate
timestamp: 2026-05-12T18:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md
  - /Users/jmagady/Dev/monocle/.factory/semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md
  - /Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/claude-squad/claude-squad-pass-8-deep-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/zellij/zellij-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/lazygit/lazygit-pass-8-final-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md
input-hash: "[live-state]"
traces_to: "factory-artifacts 6ac4279 (brief v1.2)"
project: monocle
recommendation: CAUTION
---

# Market Intelligence Assessment: Monocle

## Recommendation

**CAUTION — proceed with revised brief assumptions before Phase 1.**

Anthropic shipped `claude agents` (agent view, research preview, v2.1.139) on May 11, 2026 — the day before this assessment — directly commoditizing monocle's most visible Phase 1 surface: "see all sessions in one list, respond to the one that needs you." This does not kill monocle's differentiation, but it does shift the brief's positioning claim from "no one does this" to "Anthropic does a thin version; monocle goes far deeper." The customer pain (permission prompt interruptions, factory-pattern situational awareness, customization visibility, multi-harness cost tracking) is validated as real and persistent by 30+ open GitHub issues, Reddit threads, and a dense field of third-party tools that emerged to fill it. The brief's core differentiators — hook-protocol ingestion (not file polling), VecDeque permission overlay with diff preview, trigger-trace to settings.json, workflow-plane factory awareness, multi-harness OTel cost panel — are not replicated by any tool in the current landscape, including the just-shipped agent view. Go once the brief explicitly repositions monocle against agent view and makes the deeper-integration argument front and center.

---

## Competitive Landscape

### Gap Matrix

| Tool | session-mgmt | customization-explore | workflow-awareness | multi-harness | cost-tracking | hook-overlay |
|------|-------------|----------------------|-------------------|---------------|---------------|-------------|
| any-context/lazyclaude | YES (hook-native, pm/worker) | NO | NO | NO | NO | PARTIAL (PreToolUse only) |
| NikiforovAll/lazyclaude | NO (static viewer) | YES (7 parsers) | NO | NO | NO | NO |
| claude-squad | YES (tmux+worktree) | NO | NO | SHALLOW (program string) | NO | NO |
| claude-code-router (CCR) | NO | NO | NO | YES (7 engines, routing) | NO | NO |
| codemachine-cli | NO (execution-only) | NO | YES (workflow FSM) | YES (7 engines) | YES (OTEL) | NO |
| zellij | NO (multiplexer) | NO | NO | NO | NO | NO |
| lazygit | NO (git TUI) | NO | NO | NO | NO | NO |
| **Recon** (NEW — Rust, Cargo) | YES (tmux poll) | NO | NO | NO | NO | NO |
| **claude-manager** (NEW — Rust/ratatui) | YES (tmux+worktree) | NO | NO | NO | NO | NO |
| **CCManager** (NEW — Go) | YES (multi-harness) | NO | NO | PARTIAL (claude/gemini/codex) | NO | NO |
| **claude-picker** (NEW — Rust/ratatui) | YES (session history) | NO | NO | NO | YES (cost/audit) | NO |
| **Nimbalyst** (NEW — desktop GUI) | YES (kanban) | NO | NO | YES (claude+codex) | NO | NO |
| **ur-dashboard** (NEW — npm) | YES (real-time) | NO | NO | YES (multi-provider) | YES (per-model) | NO |
| **Anthropic agent view** (NEW — built-in) | YES (native, built-in) | NO | NO | NO | NO | NO |
| **monocle (target)** | YES (hook-native) | YES (trigger-trace) | YES (factory-adapter) | YES (Phase 4) | YES (Phase 4 OTel) | YES (VecDeque+diff) |

**Key observations from the gap matrix:**

1. **Anthropic's agent view** (shipped May 11, 2026) covers session-mgmt at the list-and-respond level only. It has no customization plane, no workflow awareness, no diff preview, no cost tracking, no hook-protocol overlay. It dispatches from within Claude Code itself — not as an external overlay. It is a thin feature, not a dashboard product.

2. **Recon** is the closest structural analog to monocle's Phase 1 sessions panel — Rust, tmux-native, reads Claude Code's own files. Its approach is tmux-pane-scraping (brittle) vs monocle's hook POST ingestion (structured). Recon has no permission overlay, no daemon, no cross-harness plan.

3. **claude-manager** (ratatui, active releases as of May 2026) is a direct Rust/ratatui competitor for the sessions + tmux management surface. It launched 30 releases with the most recent on May 6, 2026. No hook protocol, no permission overlay, no customization, no workflow plane.

4. **claude-picker** is the only tool with a cost audit feature approaching monocle's Phase 4 OTel panel, but it operates on historical JSONL, not live hook events.

5. **The customization plane, workflow plane, and hook-native permission overlay remain unoccupied** across all discovered tools as of May 12, 2026.

### Competitive Density Score

**MEDIUM-HIGH** for session roster display; **LOW** for monocle's combined differentiator surface (hook-native overlay + customization trace + workflow awareness).

---

## Customer Pain Validation

| Persona | Asserted Pain | Validation Source | Confidence |
|---------|--------------|-------------------|-----------|
| Multi-session developer — permission prompts stall session B while in session A | Permission prompts interrupt parallel workflows; "Cannot step away for more than a few seconds during non-trivial workflows" (GitHub #11380, 160+ comments, filed Aug 2025); "A project that should take 1 hour is taking 36 hours" (Facebook Vibe Coding group, multiple +1s); "Permissions matching is fundamentally broken — 30+ open issues, no staff engagement" (GitHub #30519, Mar 2026) | https://github.com/anthropics/claude-code/issues/11380, https://github.com/anthropics/claude-code/issues/30519, https://www.facebook.com/groups/vibecodinglife/posts/1894363367818858/ | HIGH |
| Multi-session developer — no unified view of which session is blocked | "Running several agents in parallel across a tmux server means tabbing between panes to find out which sessions are blocked" (Recon announcement, agent-wars.com, Mar 2026); `claude agents` doc: "No more hunting across tabs to find what's blocked" (Anthropic's own framing for agent view, May 2026) — Anthropic's own marketing language validates the pain | https://agent-wars.com/news/2026-03-14-recon-tmux-tui-claude-code-sessions, https://code.claude.com/docs/en/agent-view | HIGH |
| Factory-pattern operator — situational awareness without leaving the editor | vsdd-factory STATE.md is used in production by the monocle project itself; the brief was authored in a live vsdd-factory pipeline; no external third-party validation found for this specific workflow — it is a niche that exists but is narrow | Internal (self-referential: the monocle product brief was authored inside a vsdd-factory pipeline) | MEDIUM (real pain, narrow population) |
| Multi-harness operator — no unified cost view across Claude Code + CodeMachine | Multi-agent token burn at scale creating "quota gone in days" (GitHub #41930: "1 Opus PM + 6-8 Sonnet workers ... 20% of monthly quota consumed overnight"); ur-dashboard and claude-picker exist specifically to fill this gap | https://github.com/anthropics/claude-code/issues/41930, https://lib.rs/crates/claude-picker, https://dev.to/thestack_ai/ur-dashboard | MEDIUM (Phase 4, not v1) |

**Pain Confirmed: YES** for the multi-session developer persona. **PARTIAL** for the factory-pattern operator (real but niche). **MEDIUM** for multi-harness cost (real but Phase 4).

---

## Market Size Estimate

### Method

- Claude Code adoption: 28% primary adoption among developers surveyed by Digital Applied (Q1 2026, n=2,847 across 320 organizations); 54% any-use adoption; ~18% at work in JetBrains Jan 2026 survey. Claude Code holds "54% of the AI coding market" per analyst forums. (Source: https://www.digitalapplied.com/blog/ai-coding-tool-adoption-2026-developer-survey, https://byteiota.com/ai-coding-tools-2026/)
- Estimated global developer population: ~27M (Sacra, Feb 2025). Active Claude Code users: ~4M–8M extrapolating from adoption surveys and comparison to Cursor's 1M+ paying customers.
- Multi-session power users (monocle's SAM): developers running 2+ concurrent Claude Code sessions. Based on usage patterns in GitHub issues and forum threads, conservatively 10–20% of active Claude Code users = 400K–1.6M developers.
- Factory-pattern operator niche: vsdd-factory users + comparable structured workflows. Estimated 1–5% of Claude Code power users = 40K–200K.
- Multi-harness operators: currently a smaller cohort; likely 5–10% of power users by Phase 4 = 20K–100K currently, growing.

### Estimates

| Segment | Est. Population | Notes |
|---------|----------------|-------|
| TAM: All Claude Code users | ~4M–8M | Survey-based extrapolation |
| SAM: Multi-session Claude Code power users (monocle's Phase 1 core) | ~400K–1.6M | 10–20% of TAM; validated by depth of third-party tool demand |
| SOM: Reachable in Year 1 (terminal-native, tmux users, vsdd-factory operators) | ~10K–50K | Conservative; cargo install + word-of-mouth; no marketing budget |
| Factory-operator niche | ~40K–200K | Grows as structured AI pipeline adoption expands |

**Market Maturity:** Nascent-to-growing. The segment was essentially non-existent before 2025; in 18 months it generated ~15+ third-party tools and a first-party Anthropic response (agent view). This is a fast-moving but thin market relative to Claude Code's base.

**Market verdict:** The TAM is real. The SAM is meaningful. The SOM for a Rust open-source CLI tool is modest in Year 1 but positions well for when structured multi-agent workflows become mainstream. This is not a large-market bet — it is a developer-tool infrastructure play targeting a high-value power-user cohort.

---

## Differentiation

### What makes monocle defensible

**1. Hook-protocol ingestion, not file polling.**
Every competing tool (Recon, claude-manager, claude-picker) reads `~/.claude/` files or polls tmux pane text. Monocle receives structured POST events from Claude Code's hook protocol — the same low-level signal that Anthropic's own agent view does NOT use. Hook ingestion gives monocle sub-100ms latency, structured data (tool type, diff content, session ID), and the ability to participate in permission decisions before Claude Code acts. This is a genuine architectural moat against file-polling competitors.

**2. VecDeque permission overlay with diff preview.**
No tool in the landscape — including agent view — offers a cascaded permission overlay that queues multiple simultaneous prompts, shows a diff preview, and accepts/rejects without switching terminals. The agent view dispatches by "peeling off" to attach to a session; monocle handles prompts without leaving the current context. This is monocle's killer scenario directly addressed and unaddressed by the field.

**3. Trigger-trace: permission prompt to settings.json line.**
The `[t]` trace-to-source capability (Phase 2) — connecting a permission prompt to the specific settings.json rule (or gap) that caused it — is not implemented by any tool in the landscape. Given that 30+ GitHub issues describe the permission system as "fundamentally broken" and undebugable, this is a high-value feature in a market with documented, unmet demand.

**4. Workflow-plane factory awareness.**
No tool surfaces factory/pipeline workflow state (phase, blocking issues, convergence) alongside live session data. The VsddFactoryAdapter and FactoryAdapter trait are unique to monocle. The near-zero population of factory-pattern operators today does not diminish this differentiator — it is a wedge into the premium power-user segment that monocle can own from day one.

**5. Multi-harness architecture.**
The EngineModule abstraction (from codemachine-cli gene) positions monocle as the only hook-native TUI that can serve Claude Code, CodeMachine, and future harnesses from a single daemon. All current competitors are Claude Code-specific.

**6. Rust + ratatui quality signal.**
In a field of Go TUIs (claude-squad, cs), npm packages (CCManager, ur-dashboard), and Python tools (NikiforovAll), a Rust binary with ratatui, bounded event bus, and formal hook-parity tests is a quality and performance differentiator that matters to the developer audience.

**Defensibility verdict:** Monocle's differentiation is real and currently unoccupied. The risk is execution speed — the window before Anthropic deepens agent view is unknowable.

---

## Risk Register

| Risk ID | Risk | Severity | Probability | Mitigation | Impact if Realized |
|---------|------|----------|-------------|------------|-------------------|
| R-001 | Anthropic ships hook-native permission overlay in agent view (direct feature overlap with monocle's killer scenario) | CRITICAL | MEDIUM (25–40%) | Ship Phase 1 fast; lead with workflow plane and trigger-trace as second moat — these are harder for Anthropic to replicate; OSS positioning creates community lock-in | Monocle's Phase 1 value proposition is commoditized; pivot to Phase 2/3 differentiators or fold |
| R-002 | Anthropic ships first-party factory/workflow panel (Phase 3 moat erosion) | HIGH | LOW (5–15%) | Anthropic has no vsdd-factory relationship and no structured pipeline-state read layer; the FactoryAdapter trait's openness is a defensibility accelerator | Phase 3 positioning weakened; monocle becomes workflow-agnostic observer only |
| R-003 | Claude Code hook protocol breaks backward compatibility (existential for hook ingestion model) | HIGH | LOW-MEDIUM (10–25%) | Monitor changelog weekly; the hook protocol has been stable since its introduction; the 5-endpoint set has not changed; `X-Claude-Code-Ide-Authorization` auth header is stable | Monocle's ingestion layer requires significant rework; could cause multi-month delays |
| R-004 | Market fragments into GUI desktop apps (Nimbalyst, Opcode) rather than terminal TUI | MEDIUM | MEDIUM (30–45%) | The terminal-native segment is a loyal cohort; lazygit has ~35K+ GitHub stars despite GUI alternatives; monocle's target user is explicitly terminal-first | Smaller but still meaningful SAM; monocle becomes a niche power-user tool rather than a mainstream one |
| R-005 | Recon or claude-manager ships hook-protocol ingestion (technical convergence) | MEDIUM | LOW-MEDIUM (15–25%) | Both tools are actively maintained (claude-manager: 30 releases through May 2026); hook protocol ingestion is not a patent-protected moat; Recon reads Claude's own session files | Architectural moat weakened; must compete on feature depth (workflow plane, trigger-trace) |
| R-006 | Cursor IDE / VS Code captures the multi-session management market via editor integration | MEDIUM | LOW (5–10%) | Terminal-native developers do not want IDE-bound session management; editor integrations (Nimbalyst, Opcode) appeal to a different persona than monocle's target | Smaller addressable market than projected; terminal-native segment grows more slowly |
| R-007 | Claude Code's hook protocol expands incompatibly (e.g., new required fields, auth rotation breaks) | MEDIUM | MEDIUM (20–35%) | Brief includes `contract_version` field and SOQ-2 token rotation invariant specifically to handle this; any-context's hook-parity test suite provides regression coverage | Hook ingestion stops working until patched; daemon auth broken |
| R-008 | Factory-pattern operator market fails to materialize at scale | LOW | MEDIUM (35–50%) | Phase 1 does not require factory operators; it stands alone on the multi-session/permission-overlay value; Phase 3 is gated roadmap | Phase 3 has lower ROI than projected; monocle's workflow plane remains niche |

**Top risks ranked by adjusted severity:** R-001 (CRITICAL × MEDIUM) > R-003 (HIGH × LOW-MEDIUM) > R-007 (MEDIUM × MEDIUM) > R-002 (HIGH × LOW).

---

## GO/CAUTION/STOP Justification

### Why CAUTION, not STOP

The agent view shipped yesterday is a research preview that dispatches sessions and shows a list — nothing more. It does not:
- Use hook protocol for structured event ingestion (it attaches to running sessions via the TUI's own plumbing)
- Show diff preview in a permission overlay
- Surface customization (CLAUDE.md, settings.json permissions, keybindings)
- Surface factory/workflow pipeline state
- Support multi-harness (CodeMachine, future)
- Provide OTel cost aggregation across sessions or harnesses
- Work as an external overlay over an existing tmux setup

Monocle's value proposition was always "deeper than a session list." The agent view validates that Anthropic agrees the session list problem is worth solving, which is positive market validation. It does not solve the full problem monocle targets.

The field of ~15 third-party tools that emerged in under 18 months — all filling gaps that prior art left open — confirms that the customer pain is real, persistent, and not satisfied by incremental improvements to Claude Code itself. The permission system has 30+ open GitHub issues with no Anthropic response; trigger-trace addresses a documented gap that Anthropic has not prioritized.

### Why not GO immediately

The brief's competitive positioning section states "Monocle is the session management and permission-prompt dispatch layer that none of them provide." This was accurate as of the brief's authored date (May 12, 2026 morning) but is now partially inaccurate: agent view provides the session list component. The brief's headline positioning claim needs one revision pass before Phase 1 specs crystallize, so the architect, product-owner, and downstream agents work from accurate competitive assumptions.

The CAUTION is not about the product concept — it is about the brief needing a targeted update before spec work proceeds.

### Why not STOP

- Customer pain is HIGH-confidence validated across multiple independent sources
- Monocle's true differentiators (hook ingestion, VecDeque overlay, trigger-trace, workflow plane) remain unoccupied
- The market is growing rapidly; Claude Code at 28% primary adoption with 54% any-use means the denominator of potential monocle users is large and expanding
- The risk of Anthropic building the full monocle feature set is low because: (a) Anthropic focuses on Claude Code as an AI coding agent, not an AI session manager; (b) the workflow plane (factory awareness) requires Anthropic to build a generic pipeline observer that benefits third-party workflows — unlikely; (c) the trigger-trace feature requires parsing third-party settings.json semantics — not Anthropic's product scope

---

## Conditions for GO

The following conditions upgrade CAUTION to GO. All are actionable brief edits, not product scope changes:

1. **Brief revision: Reposition against agent view explicitly.** Add a "vs agent view" comparison to the Competitive Positioning section that explains: agent view = session list + inline reply; monocle = hook-native overlay + diff preview + customization trace + workflow awareness + multi-harness + cost panel. Position agent view as a market validator, not a competitor.

2. **Brief revision: Clarify the Phase 1 killer scenario is NOT the session list.** The brief's killer scenario (Ctrl-\, `2`, `1`, Ctrl-\) is specifically about resolving concurrent permission prompts with diff preview — which agent view cannot do. Make this explicit to avoid the architect designing around the session-list feature as the primary value.

3. **Brief revision: Acknowledge R-001 explicitly.** Add a risk acceptance entry: "Anthropic agent view is a research preview addressing session visibility; monocle's differentiation is the hook-protocol permission overlay and trigger-trace, which agent view does not implement." This binds downstream agents to the correct competitive frame.

4. **Optional: Validate that claude-manager (Rust/ratatui, active May 2026) does not already implement hook-protocol ingestion.** A quick review of its GitHub codebase would confirm whether the architectural moat is intact. If it does, escalate to STOP for Phase 1 sessions panel; monocle must lead with Phase 2 trigger-trace instead.

---

## Open Questions Raised

The following market intelligence findings surface questions the brief should address:

1. **OQ-M1 — Agent view hook protocol relationship**: Does Anthropic's agent view use Claude Code's hook protocol or a different mechanism? If it uses hook protocol, monocle's `X-Claude-Code-Ide-Authorization` auth scheme must ensure monocle and agent view can coexist on the same host without port collision or auth conflict. Research needed: read https://code.claude.com/docs/en/agent-view for IPC mechanism.

2. **OQ-M2 — claude-manager architectural review**: claude-manager (lib.rs/crates/claude-manager, ratatui, 30 releases through May 2026) uses tmux + worktrees for session management. Does it use file polling or hook protocol? If hook protocol, the architectural moat claim requires qualification. GitHub: https://lib.rs/crates/claude-manager.

3. **OQ-M3 — Hook protocol event count**: The brief specifies 5 endpoints (PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit) per JC-2. The May 2026 Claude Code docs list 25 lifecycle hook events including `PermissionRequest` as a distinct hook event separate from PreToolUse. The brief's 5-endpoint set may be incomplete. Does monocle need to add `PermissionRequest` as a sixth endpoint for cleaner permission overlay UX? (See https://ofox.ai/blog/claude-code-hooks-subagents-skills-complete-guide-2026/ — `PermissionRequest` hook fires when Claude Code is about to show a permission dialog and can auto-approve or deny.)

4. **OQ-M4 — Recon fragility**: Recon detects permission-blocked status by reading tmux pane status bar text ("escape to cancel"). This is brittle — any Claude Code UI change breaks it. Monocle's hook-ingestion approach is structurally superior. The brief should explicitly call out the tmux-pane-scraping fragility of all polling-based competitors as a long-term moat argument.

5. **OQ-M5 — Factory-operator SOM**: The brief targets factory-pattern operators as Persona 2. The market search found no third-party tools for this use case — it may be a genuinely unmet need with a small current population. Should the v1 killer scenario be broadened to include a simpler factory-awareness demo (detect .factory/ STATE.md + show phase) even in Phase 1, to differentiate from agent view from day one?

---

## References

1. Anthropic agent view announcement: https://gigazine.net/gsc_news/en/20260512-claude-code-agent-view-aws/ (May 12, 2026)
2. Claude Code agent view docs: https://code.claude.com/docs/en/agent-view (May 11, 2026)
3. Claude Code 2.1.139 release notes: https://releasebot.io/updates/anthropic/claude-code (May 11, 2026)
4. Claude Code agents parallelism docs: https://code.claude.com/docs/en/agents (May 11, 2026)
5. Best Claude Code session managers 2026 (Nimbalyst blog): https://nimbalyst.com/blog/best-session-managers-for-claude-code-and-codex/ (March 30, 2026)
6. Recon Rust TUI dashboard announcement: https://agent-wars.com/news/2026-03-14-recon-tmux-tui-claude-code-sessions (March 14, 2026)
7. claude-manager (Rust/ratatui): https://lib.rs/crates/claude-manager (May 6, 2026)
8. claude-picker (Rust/ratatui, cost audit): https://lib.rs/crates/claude-picker (April 18, 2026)
9. CCManager (Go, multi-harness): https://github.com/kbwo/ccmanager (June 2025 — ongoing)
10. ur-dashboard (npm, real-time multi-agent): https://dev.to/thestack_ai/i-run-5-claude-code-agents-at-once-i-had-no-idea-what-they-were-doing-273p (March 2026)
11. Nimbalyst (desktop GUI with iOS app): https://nimbalyst.com (ongoing)
12. GitHub issue — permission matching fundamentally broken (30+ issues): https://github.com/anthropics/claude-code/issues/30519 (March 2026)
13. GitHub issue — Always Allow not working (160+ comments): https://github.com/anthropics/claude-code/issues/11380 (August 2025)
14. GitHub issue — permission context lost from Claude's memory: https://github.com/anthropics/claude-code/issues/21503 (January 2026)
15. GitHub issue — Show statusline during permission prompts: https://github.com/anthropics/claude-code/issues/40248 (April 2026)
16. GitHub issue — quota drain from multi-agent permission bug: https://github.com/anthropics/claude-code/issues/41930 (April 2026)
17. Facebook Vibe Coding group — permission prompt frustration: https://www.facebook.com/groups/vibecodinglife/posts/1894363367818858/ (ongoing)
18. JetBrains AI Pulse Jan 2026 survey — Claude Code 18% at work: https://www.getpanto.ai/blog/cursor-ai-statistics (April 2026)
19. Digital Applied Q1 2026 AI coding tool survey — Claude Code 28% primary: https://www.digitalapplied.com/blog/ai-coding-tool-adoption-2026-developer-survey (April 2026)
20. Claude Code 2.1.7 Agent Teams, Mux Terminal: https://www.idlen.io/news/claude-code-2-new-features-anthropic-2026/ (March 2026)
21. Claude Code hooks 25 lifecycle events guide: https://ofox.ai/blog/claude-code-hooks-subagents-skills-complete-guide-2026/ (April 2026)
22. Stanislas TUI for indexing coding agent sessions: https://stanislas.blog/2026/01/tui-index-search-coding-agent-sessions/ (January 2026)
23. claude-session-manager-tui (Go, borball): https://libraries.io/go/github.com%2Fborball%2Fclaude-session-manager-tui (April 2026)
24. cs — Claude Session Manager (Go, no tmux): https://libraries.io/go/github.com%2Fdakaneye%2Fclaude-session-manager (April 2026)
25. wzcc — WezTerm Claude Code session manager (Rust): https://lib.rs/crates/wzcc (February 2026)
26. Cursor statistics 2026 ($2B ARR): https://www.gradually.ai/en/cursor-statistics/ (April 2026)
27. Claude Code Desktop redesign + Routines (April 14, 2026): https://www.mikegingerich.com/blog/anthropic-revamps-claude-code-with-multi-session-desktop/ (April 2026)
28. Claude Code changelog — breaking changes and hook evolution: https://www.getaiperks.com/en/articles/claude-code-changelog (2026)
