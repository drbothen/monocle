# monocle — Project Operating Instructions

> Read this file first. Every other doc in this repository operates under the principle stated below.

## Project Identity

**monocle** is a single-binary Rust TUI that manages AI coding harness sessions (Claude Code first, future CodeMachine, others). Five planes:

1. **Runtime** — live session state and harness lifecycle.
2. **Static** — customization explorer (5-level binding precedence: SearchPrompt > UserCustomCommand > PerContext > Global > Builtin).
3. **Workflow** — factory awareness (`.factory/STATE.md` ingestion, phase visibility).
4. **Harness** — `EngineModule` trait abstraction over Claude Code today, CodeMachine + others tomorrow.
5. **TUI** — lazy* signature (lazygit-philosophy): single `Ctrl-\` tmux popup over the user's editor, observe-only for state, action-only for permission overlays + keybinding dispatch.

Mode: greenfield-with-reference-ingest. 8 reference repos already ingested into `.factory/semport/` (any-context-lazyclaude, nikiforovall-lazyclaude, vsdd-factory, codemachine-cli, zellij, lazygit, claude-squad, claude-code-router).

Vision approved verbatim by the human on 2026-05-11. Canonical vision: `.factory/specs/research/domain-monocle-vision-synthesis.md`.

## Current Pipeline State

Read `.factory/STATE.md` for live state. As of last commit on this branch:
- Brief: `v1.4.29` at `.factory/specs/product-brief.md`, `validate-brief` verdict: v5 VALID.
- Phase: `pre-phase-1-final-gate-post-fix-burst` (round-4 consistency audit complete; adversary fresh pass pending).
- Mode: greenfield-with-reference-ingest.

## Build / Test / Lint

The Rust workspace will be initialized during `/vsdd-factory:create-architecture`. Until then there is no `Cargo.toml` at the root. Once the workspace exists, the canonical commands are:

- Build: `cargo build --workspace`
- Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace -- -D warnings`
- Security audit: `cargo audit` (must run in CI on every PR; weekly scheduled `cargo audit --json` against latest RUSTSEC DB)
- Format: `cargo fmt --all`

MSRV: Phase 1 = Rust 1.86 (ratatui 0.30 floor). Phase 3 = Rust 1.92 (wasmtime 44 requirement). Single-workspace MSRV strategy is canonical; see `.factory/specs/architecture/SS-deps-pin-manifest.md` §"MSRV Policy".

## Architectural Authority — Source of Truth

When two artifacts disagree, the LATER, MORE-SPECIFIC artifact wins:

1. `.factory/specs/architecture/SS-deps-pin-manifest.md` — canonical version pins, MSRV policy, patch-pinning policy, security-advisory response policy, workspace dependency graph (supersedes vision §Tech Stack).
2. `.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md` — wasmtime 44 over wasmi for Phase 3 plugin SDK.
3. `.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md` — nucleo 0.5 dormancy risk accepted with explicit re-eval trigger (retires TD-001).
4. `.factory/specs/architecture/SS-conventions-anti-patterns.md` — code conventions, anti-patterns, clippy + semgrep + PR-template + CI enforcement specs.
5. `.factory/specs/dtu-assessment.md` — DTU clone scope for hook protocol surface (DTU_REQUIRED: true for Phase 1).
6. `.factory/specs/product-brief.md` v1.4.29 — Phase 1-4 scope, success criteria, competitive positioning vs Anthropic agent view. R-001 (Anthropic commoditization risk) reassessed at <10% probability; informational only, no mitigation scaffolding required.
7. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 — re-approved 2026-05-12 with refreshed endpoint set (canonical 5) and tech-stack pointer to `SS-deps-pin-manifest.md`. Captures human intent including all JC/EX/OQ-M closures. v1.1.2 is a surgical patch of v1.1 (path refs + frontmatter date); substantive content unchanged.
8. `.factory/tech-debt-register.md` — tech debt register (see Principle 3 below; not for AI-driven deferrals).

---

## CANONICAL PRINCIPLE — Production-Grade Default

This principle binds every AI agent operating on this project. It overrides any default behavior in agent prompts, skills, or templates that conflicts with it.

### Statement

**Default behavior is enterprise/production-grade correctness. Speed lives in feature *ordering*, not feature *completeness*.**

### Six rules

1. **No MVP-driven deferrals.** Phrases like "for now," "good enough," "we can fix later," "minimum viable," and "ship fast and iterate" are RATIONALIZATIONS, not engineering decisions. Treat them as defect-pattern smells. If a thing is worth doing in v1, it is worth doing correctly in v1.

2. **Feature order is the only acceptable speed lever.** It is acceptable to defer an entire feature (e.g., Phase 4 multi-harness federation) to a later wave. It is NOT acceptable to ship the v1 features partially or with shortcuts that need later cleanup. Each shipped feature must be production-grade, enterprise-ready, on the cycle it ships.

3. **Tech debt register (`tech-debt-register.md`) is for HUMAN-DIRECTED deferrals ONLY.** AI agents must NOT add entries to it as a default catchment for issues found during review. If an agent discovers a defect, the default action is to FIX it in-scope. Adding to the register requires:
   - Explicit human direction to defer, AND
   - A concrete future dependency that makes the deferral necessary (e.g., "this depends on Phase 3 wasmtime SDK"), AND
   - Attachment to the specific future story or wave where it will be resolved (so it cannot get lost).

4. **AI-built defects are the AI's responsibility to fix.** Every artifact in this repository was written by AI (with human approval). When an AI agent finds an issue in another AI agent's output, the default is to fix it in the current scope — even if that means expanding scope. Surfacing the issue as a question, an "advisory," a "TODO for architect," or a "pending architect review" is the WRONG default. The correct default is to fix.

5. **`Suggest` is acceptable. `Default to cheap path` is not.** Agents may propose cheaper alternatives to the human, but the agent's DEFAULT action must be the correct path. "I noticed this would be faster if we skipped X — would you like to?" is fine. Skipping X without surfacing the option is not.

6. **"Pending architect review" / "TODO for architect" / "Placeholder for architect" in spec artifacts is forbidden when the question is answerable in current scope.** If the question requires architect adjudication only because the answer needs cross-component reasoning that hasn't happened yet, that's legitimate. If the question is mechanical (path migration, version pin selection, conventional clippy lint configuration), the AI handling the spec must answer it now.

### What this means in practice

| Anti-pattern | Production-grade replacement |
|--------------|------------------------------|
| "MVP: ship without test coverage on edge case X" | Write the edge case test. Cover it now. |
| "For now we'll hardcode this value; refactor later" | Read the value from config now. Write the config schema. |
| "We can add error handling in v2" | Add error handling now. Define the error taxonomy in scope. |
| "Architect TODO: confirm patch-version pinning policy" | Pick the production-grade default (caret pin for libs, exact pin for binary deps with explicit security justification) and write the rationale inline. |
| "Pending architect review: should we support 6 hook endpoints?" | Read the gene-source canonical 5-endpoint matrix, decide based on existing parity argument, document the decision. |
| "Phase 5 deferred: add this to tech-debt-register" | First ask: did the human direct this deferral? If no, fix it now. |
| "Good enough for v1" | "Production-grade for v1." If you can't say production-grade, you're not done. |

### Self-Audit Checklist (every agent, before declaring work done)

Run this checklist as the last act of every task. If any answer is "yes" or "I'm not sure," stop and remediate before declaring done.

- [ ] Did I rationalize any decision with "MVP," "for now," "good enough," or "we can fix later"?
- [ ] Did I add a new tech-debt-register entry without explicit human direction AND a future story/wave anchor?
- [ ] Did I leave any "pending architect review," "TODO for architect," or "Placeholder for architect" in a spec artifact for a question I could have answered in scope?
- [ ] Did I find a bug or gap in another AI's output and surface it as a question/advisory instead of fixing it in scope?
- [ ] Did I default to the cheapest mechanism instead of the correct mechanism?
- [ ] If I added an ADVISORY-severity finding to a report, did I evaluate whether it should be a BLOCKER under the production-grade lens? (Most "advisories" become blockers.)

### Boundaries — what the principle does NOT mean

- **It does not mean "do everything before shipping anything."** Phasing features (Phase 1 → 2 → 3 → 4) is correct. Within a phase, every shipped feature must be production-grade.
- **It does not mean "no asks of the human."** Genuine human decisions — risk acceptance, business priorities, scope vs deadline tradeoffs, versioning policy — should be surfaced. The principle forbids deferring WORK that the AI can do; it does not forbid surfacing DECISIONS that only the human can make.
- **It does not mean "infinite scope expansion."** If you find an issue, fix it. If the fix requires expanding into a new domain that requires new specs or new architecture decisions, surface it cleanly and request scope expansion. The principle requires fixing, not infinite recursion.
- **It does not override security or correctness.** If a "production-grade fix" requires a security review, run the security review.

### Conflict resolution

If this principle conflicts with a vsdd-factory agent prompt, skill, or rule, this principle wins for monocle. Upstream changes to canonicalize this principle across all VSDD projects are tracked in the `drbothen/vsdd-factory` GitHub issue tracker.

---

## Conventions (Code-Level)

Detailed conventions live in `.factory/specs/architecture/SS-conventions-anti-patterns.md`. Highlights:

- **Product name:** lowercase `monocle` in code; capitalized `Monocle` in prose headings.
- **Forbidden patterns:** shell injection via template strings; naked `std::fs::write` for config (use `tempfile::persist`); unbounded `mpsc::unbounded_channel` (use bounded with drop counter); package-level mutable globals for theme/config (use `Arc<RwLock<>>`); `Option<PromptModal>` for permission overlay (use `VecDeque<PromptModal>`).
- **Error handling:** `thiserror 2.x` for library error types; `anyhow 1` for binary-crate error propagation. Production-grade error taxonomy required in every public API.
- **Atomic writes:** all config files via `tempfile::persist`. No exceptions.
- **Logging:** `tracing 0.1` with structured fields. No `println!` in production code.
- **Channels:** bounded `mpsc::channel(N)` with surfaced drop counters in the status bar. Integration test target: 1000 events/sec with drop counter assertion.

## Git Workflow

- Main branch: `main`
- Factory artifacts branch: `factory-artifacts` (orphan branch mounted at `.factory/` via worktree)
- Commit hooks: `block-ai-attribution` (rejects "Co-Authored-By: Claude" and robot emojis), `validate-input-hash` (verifies artifact input-hash freshness), `validate-table-cell-count` (every data row pipe count must match header)
- Heredoc-in-bash sometimes blocked at large payloads — use `git commit -F /tmp/<file>` when that happens
- **NEVER** skip hooks (`--no-verify`)
- **NEVER** add `Co-Authored-By: Claude` or robot emoji to commits
- **NEVER** force push to `main`

## Tooling

- `compute-input-hash` (vsdd-factory plugin): `bin/compute-input-hash --scan .factory` for drift detection, `--update` to bump hashes after legitimate content changes
- `lobster-parse`: workflow file parser
- Factory orchestrator: invoked via `/vsdd-factory:run-phase <phase-id>` or per-skill slash commands

## Pipeline Authority

The orchestrator (`vsdd-factory:orchestrator` agent) coordinates all phases. Specialist agents do the writing. The orchestrator does not write files itself — it delegates via the `Agent` tool with `subagent_type` set to the specialist.

Phase sequence:
- Phase -1: Reference Ingest (DONE) — 8 repos, 8 final-synthesis files in `.factory/semport/`
- Phase 0.5–0.8: Brief authoring (DONE) — v1.0 → v1.3
- Phase 0.9: Market intel + validate-brief (DONE) — VALID
- **Pre-Phase-1 final gate (CURRENT)** — awaiting human approval to enter Phase 1
- Phase 1: Spec Crystallization — domain spec → PRD → architecture → adversarial review → human approval
- Phase 2: Story Decomposition
- Phase 3: TDD Implementation
- Phase 4: Holdout Evaluation
- Phase 5: Adversarial Refinement
- Phase 6: Formal Hardening
- Phase 7: Convergence

---

## Correct Agent Routing — The Production-Grade Companion Principle

The production-grade default ("fix in scope, don't defer") works ONLY when paired with correct agent routing. Otherwise it degrades into "every agent does everything," which destroys specialization and produces worse work than the defer-pattern it replaces.

### Statement

**"Fix in scope" means route the defect to the CORRECT SPECIALIST AGENT in scope of the current work cycle — not defer it, and not silently fix it with the wrong agent.**

### Rules

1. **Agents own their domain.** A spec-reviewer agent reviewing a PR does NOT silently rewrite the implementation code. An implementer does NOT silently rewrite the spec. Each specialist agent has a defined scope (see routing table below); work outside that scope is routed to the correct specialist.

2. **The orchestrator owns routing.** When a specialist agent discovers a defect outside its own domain, it surfaces the finding to the orchestrator with the proposed routing. The orchestrator then dispatches the correct specialist. This is NOT a defer-pattern — it is correct-agent-pattern. The fix still happens in scope of the same work cycle.

3. **"Surface" vs "defer" — the critical distinction:**
   - **Surface (production-grade):** Agent A finds an issue → routes to orchestrator with "this needs specialist B" → orchestrator dispatches specialist B → specialist B fixes in scope → original work proceeds. **No human round-trip required for the routing.**
   - **Defer (forbidden):** Agent A finds an issue → adds to tech-debt-register / advisory / "TODO for X" → original work declared done → issue persists across multiple cycles. **Requires human to discover and re-prioritize.**

4. **When in doubt about routing, ask the orchestrator** — not the human. The orchestrator has the routing table and can dispatch. Asking the human is for genuine human decisions, not routing decisions.

5. **The orchestrator NEVER does specialist work itself.** It coordinates, dispatches, and validates gates. If the orchestrator is tempted to write a file directly, that is a routing failure — find the correct specialist and dispatch them.

### Agent Routing Table

Use this table to determine which specialist handles which kind of work. Authoritative reference; supersedes any conflicting routing in upstream skills until the upstream vsdd-factory canonicalization lands.

| If the work is... | Route to agent ID |
|-------------------|-------------------|
| Product brief, PRD, behavioral contracts (BCs), holdout scenarios | `vsdd-factory:product-owner` |
| Market analysis, L2 domain spec, ubiquitous language | `vsdd-factory:business-analyst` |
| Architecture, ADRs, DTU assessment, gene transfusion, dependency manifest | `vsdd-factory:architect` |
| UX spec, design system, wireframes, interaction design | `vsdd-factory:ux-designer` |
| Story decomposition, dependency graph, wave schedule | `vsdd-factory:story-writer` |
| Cross-document consistency (IDs, anchors, counts, naming) | `vsdd-factory:consistency-validator` |
| Adversarial fresh-context review (specs or implementation) | `vsdd-factory:adversary` |
| Constructive spec/story review (different-model cognitive diversity) | `vsdd-factory:spec-reviewer` |
| PR diff code review (different-model cognitive diversity) | `vsdd-factory:code-reviewer` |
| Deep codebase scanning, semantic analysis, brownfield ingest | `vsdd-factory:codebase-analyzer` |
| Brownfield extraction validation (catch hallucinated dependencies) | `vsdd-factory:validate-extraction` |
| TDD test stubs and failing tests | `vsdd-factory:test-writer` |
| TDD implementation (one failing test → minimum code → micro-commit) | `vsdd-factory:implementer` |
| E2E browser tests (Playwright/Cypress) | `vsdd-factory:e2e-tester` |
| Demo recordings (VHS terminal or Playwright browser) | `vsdd-factory:demo-recorder` |
| PR lifecycle (create, review dispatch, finding triage, merge) | `vsdd-factory:pr-manager` |
| Final fresh-eyes PR diff review before merge | `vsdd-factory:pr-reviewer` |
| Formal proofs (Kani), fuzzing, mutation testing, security scan | `vsdd-factory:formal-verifier` |
| Security review / triage (CWE/CVE, OWASP) | `vsdd-factory:security-reviewer` |
| Holdout scenario evaluation against implementation (strict info asymmetry) | `vsdd-factory:holdout-evaluator` |
| DTU clone validation against real third-party services | `vsdd-factory:dtu-validator` |
| Repo setup, worktrees, CI/CD, release, Cargo workspace init | `vsdd-factory:devops-engineer` |
| Toolchain preflight, env setup, dependency installation | `vsdd-factory:dx-engineer` |
| `.factory/STATE.md` updates, `.factory/` commits, cycle bookkeeping | `vsdd-factory:state-manager` |
| Spec governance, versioning, traceability audit | `vsdd-factory:spec-steward` |
| Documentation generation from code/specs (current behavior only) | `vsdd-factory:technical-writer` |
| External research (Perplexity, Context7, Tavily MCP access) | `vsdd-factory:research-agent` |
| GitHub CLI operations on behalf of agents without shell access | `vsdd-factory:github-ops` |
| Performance benchmarks, Core Web Vitals enforcement | `vsdd-factory:performance-engineer` |
| Data schemas, migrations, pure-core / effectful-I/O boundary | `vsdd-factory:data-engineer` |
| WCAG AA/AAA accessibility audit | `vsdd-factory:accessibility-auditor` |
| Visual regression, mockup fidelity comparison | `vsdd-factory:visual-reviewer` |
| Post-pipeline analysis, lessons capture, improvement proposals | `vsdd-factory:session-reviewer` |

### Routing examples (from this project's recent history)

- **Brief defect found by consistency-validator:** correct routing is `product-owner` (owner of brief content), NOT consistency-validator-fixes-it. Example: the F-03/F-04/F-11 fixes this session went through product-owner via the orchestrator after consistency-validator surfaced them.
- **Tech-stack version pin needed:** correct routing is `architect` (with input from `research-agent` for version verification), NOT product-owner copying from a generic best-practices list. The `SS-deps-pin-manifest.md` stub was correctly extracted by product-owner but its production version (v1.1.1 at architect's stub-completion; current v1.1.17) was completed by architect.
- **TDD red-gate violation found by test-writer:** route back to product-owner (if the BC is the problem) or to the human (if the spec is genuinely contradictory). DO NOT have the test-writer modify the BC silently.
- **Security finding found by security-reviewer:** triage classification is security-reviewer's job. The FIX is implementer's job (with security-reviewer re-running to confirm). Use the `fix-pr-delivery` skill.
- **Out-of-scope finding (legitimate scope-boundary defer):** still route to orchestrator. Orchestrator records the deferral with explicit future-story attachment per Principle 3 of the canonical principle. The deferral target must be a real story ID, not "Phase X" or "later."

### When the routing is unclear

If a defect doesn't obviously map to a specialist:

1. **Ask the orchestrator first.** The orchestrator has the routing table loaded; let it route.
2. **If the orchestrator is uncertain, the orchestrator asks the human.** This is the legitimate use of human time — routing-table extensions, not domain-fixes-by-wrong-agent.
3. **Default fallback for unmapped work: research → architect.** Most truly novel work that doesn't fit a specialist needs external research first, then architectural decision.

### Anti-patterns this principle blocks

- ❌ Adversary rewrites failing tests "to make them pass" (wrong: route to test-writer or implementer).
- ❌ State-manager writes spec content (wrong: route to product-owner or architect).
- ❌ Consistency-validator silently edits brief frontmatter (wrong: route to product-owner).
- ❌ Implementer adds a new BC to fix a TDD red-gate (wrong: route to product-owner; implementer cannot author specs).
- ❌ Orchestrator writes the artifact itself when a specialist's output is unsatisfactory (wrong: re-dispatch the specialist with better instructions, or escalate to human).
- ❌ Any agent edits `.factory/STATE.md` directly (wrong: state-manager owns STATE.md).

### Conflict with upstream

If a vsdd-factory agent prompt or skill defines a different routing than the table above, this table wins for monocle. The upstream canonicalization issue (filed against `drbothen/vsdd-factory`) tracks bringing upstream into alignment.

---

## When in Doubt

If you are an AI agent and you are uncertain whether the production-grade default applies in a specific case, the answer is YES. The principle is the default. Ask only if you have a concrete reason to suspect this case is an exception.

If you are a human reviewing this file and you want to change the principle, edit this file and commit. The principle becomes whatever this file says.
