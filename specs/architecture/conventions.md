---
document_type: architecture-conventions
level: L3
section: "conventions"
version: "1.0"
status: stub
producer: product-owner (extracted from brief v1.1)
phase: pre-phase-1-architecture
timestamp: 2026-05-12T16:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
input-hash: "[live-state]"
traces_to: "factory-artifacts ee09833 (brief v1.1)"
project: monocle
---

# Architecture: Code Conventions

## [Section Content]

This file records code-review enforcement rules, triple-confirmed across 8
gene-source ingest syntheses. These conventions are non-negotiable in Phase 1;
violations block PR merge. The architect is expected to wire enforcement tooling
(clippy, semgrep, PR checklist) during `/vsdd-factory:create-architecture`.

## Anti-Patterns to Reject in Code Review

The following patterns were observed as failure modes in gene-source repositories
and are explicitly forbidden in monocle's codebase:

- **Shell injection via template strings**: No `Command::new("sh").arg("-c").arg(template_string)` or equivalent `shell=True` pattern. Use `Command::new(binary).args([...])` arg-array form. Shell interpolation of user-controlled strings is a command-injection vector regardless of runtime context.

- **Naked config file writes**: No `std::fs::write` / `write_text` for config files. Use `tempfile::persist` (write to temp → atomic rename). Direct writes leave a corruption window on crash between open and close.

- **Unbounded event channels**: No `tokio::sync::mpsc::unbounded_channel`. Use bounded channel with a drop counter surfaced in the status bar. Unbounded channels mask backpressure and lead to unbounded memory growth under high-frequency hook events.

- **Package-level mutable globals for theme/config**: No `static mut` or `once_cell::Lazy<Mutex<Theme>>` at package level. Use `Arc<RwLock<Theme>>` threaded through the application context. Globals for visual state cause race conditions in multi-thread renders and make theme hot-reload impossible.

- **Single-popup overlay field**: No `Option<PromptModal>` field for the permission overlay. Use `VecDeque<PromptModal>` to support concurrent prompts without silent drop. The single-Option pattern causes the second concurrent prompt to replace the first with no acknowledgment.

## Gene-Source Citations

| Anti-Pattern | Gene-Source Evidence |
|---|---|
| Shell injection | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §mixins-r1: Python `subprocess(shell=True)` with template string in CLAUDE.md injection path |
| Naked config file writes | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §services-r1..r3: atomic-write gap finding; `write_text` with no temp-file intermediary |
| Unbounded channels | `any-context-lazyclaude-pass-8-final-synthesis-v2.md` §broker-r1: BC-BROKER-003 documents unbounded channel as a confirmed failure mode in the broker subsystem |
| Theme globals | `lazygit-pass-8-final-synthesis.md` §pkg/gui: package-level theme globals causing render-thread contention |
| Single-popup overlay | `lazygit-pass-8-final-synthesis.md` §pkg/gui popup: `Option<Popup>` drop-on-concurrent pattern; concurrent modal opens silently drop the pending prompt |

## Test-Time Enforcement

Placeholder for architect. Expected enforcement mechanisms:

- `cargo clippy` with `disallowed_methods` lint configured in `.cargo/config.toml`
- Custom semgrep rules for shell injection and naked write patterns
- PR checklist item: "All channel instantiations use `mpsc::channel(N)` with explicit bound N"
- Integration test: synthetic 1000 events/sec load with drop counter assertion

## Architect TODO

- [ ] Add `clippy::disallowed_methods` entries for `unbounded_channel`, `std::fs::write`
- [ ] Write semgrep rule for `Command::new("sh")` / `Command::new("bash")` patterns
- [ ] Add PR template checklist section "Monocle Convention Checklist"
- [ ] Confirm `tempfile::persist` is the only write path for `monocle-config`
- [ ] Wire enforcement into CI (block merge on clippy deny + semgrep findings)
