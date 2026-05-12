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
- Brief: `v1.3` at `.factory/specs/product-brief.md` (370 lines, `validate-brief` verdict: VALID).
- Phase: `pre-phase-1-final-gate`.
- Mode: greenfield-with-reference-ingest.

## Build / Test / Lint

The Rust workspace will be initialized during `/vsdd-factory:create-architecture`. Until then there is no `Cargo.toml` at the root. Once the workspace exists, the canonical commands are:

- Build: `cargo build --workspace`
- Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace -- -D warnings`
- Security audit: `cargo audit` (must run in CI on every PR; weekly scheduled `cargo audit --json` against latest RUSTSEC DB)
- Format: `cargo fmt --all`

MSRV: Phase 1 = Rust 1.86 (ratatui 0.30 floor). Phase 3 = Rust 1.92 (wasmtime 44 requirement). Two-MSRV strategy is intentional; see `.factory/specs/architecture/dependencies.md` Architect TODO.

## Architectural Authority — Source of Truth

When two artifacts disagree, the LATER, MORE-SPECIFIC artifact wins:

1. `.factory/specs/architecture/dependencies.md` — canonical version pins (supersedes vision §Tech Stack).
2. `.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md` — first ADR (wasmtime 44 over wasmi).
3. `.factory/specs/architecture/conventions.md` — code conventions and anti-patterns (shell injection, naked writes, unbounded channels, single-popup overlay, theme globals — all forbidden).
4. `.factory/specs/product-brief.md` v1.3 — Phase 1-4 scope, success criteria, competitive positioning vs Anthropic agent view.
5. `.factory/tech-debt-register.md` — tech debt register (see Principle 3 below; not for AI-driven deferrals).

The vision document captures human intent but has known stale items (endpoint count, tech-stack version pins) superseded by the above.

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
- **It does not mean "no asks of the human."** Genuine human decisions — risk acceptance, business priorities, scope vs deadline tradeoffs, versioning policy — should be surfaced. The principle forbids deferring WORK that the AI can do; it does not forbids surfacing DECISIONS that only the human can make.
- **It does not mean "infinite scope expansion."** If you find an issue, fix it. If the fix requires expanding into a new domain that requires new specs or new architecture decisions, surface it cleanly and request scope expansion. The principle requires fixing, not infinite recursion.
- **It does not override security or correctness.** If a "production-grade fix" requires a security review, run the security review.

### Conflict resolution

If this principle conflicts with a vsdd-factory agent prompt, skill, or rule, this principle wins for monocle. Upstream changes to canonicalize this principle across all VSDD projects are tracked in the `drbothen/vsdd-factory` GitHub issue tracker.

---

## Conventions (Code-Level)

Detailed conventions live in `.factory/specs/architecture/conventions.md`. Highlights:

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

## When in Doubt

If you are an AI agent and you are uncertain whether the production-grade default applies in a specific case, the answer is YES. The principle is the default. Ask only if you have a concrete reason to suspect this case is an exception.

If you are a human reviewing this file and you want to change the principle, edit this file and commit. The principle becomes whatever this file says.
