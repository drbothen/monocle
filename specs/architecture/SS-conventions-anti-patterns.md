---
document_type: architecture-section
level: L3
section: "conventions"
version: "1.1"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-12T00:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
input-hash: "[live-state]"
traces_to: "adversary re-audit 0bd4ba9 §Top 8 CRITICAL/IMPORTANT item 3; canonical principle CLAUDE.md commit 3366d58; brief v1.4 commit 70286e1; vision v1.1 commit 0e4b0f4"
project: monocle
---

# Architecture: Code Conventions

## [Section Content]

This file records code-review enforcement rules, triple-confirmed across 8
gene-source ingest syntheses. These conventions are non-negotiable in Phase 1;
violations block PR merge. Enforcement is wired via clippy, semgrep, and the
PR checklist defined in this document.

## Naming Convention

Product name: lowercase `monocle` in code identifiers (crate names such as `monocle-core`,
module names, type names such as `MonocleConfig`); capitalized `Monocle` in prose
headings and documentation. This is the authoritative rule; any prior usage of
`Monocle` in code identifiers is a defect to be fixed before merge.

## Anti-Patterns to Reject in Code Review

The following patterns were observed as failure modes in gene-source repositories
and are explicitly forbidden in monocle's codebase:

- **Shell injection via template strings**: No `Command::new("sh").arg("-c").arg(template_string)` or equivalent `shell=True` pattern. Use `Command::new(binary).args([...])` arg-array form. Shell interpolation of user-controlled strings is a command-injection vector regardless of runtime context.

- **Naked config file writes**: No `std::fs::write` / `tokio::fs::write` for config files. Use `tempfile::persist` (write to temp, then atomic rename). Direct writes leave a corruption window on crash between open and close.

- **Unbounded event channels**: No `tokio::sync::mpsc::unbounded_channel`. Use bounded channel with a drop counter surfaced in the status bar. Unbounded channels mask backpressure and lead to unbounded memory growth under high-frequency hook events.

- **Package-level mutable globals for theme/config**: No `static mut` or `once_cell::Lazy<Mutex<Theme>>` at package level. Use `Arc<RwLock<Theme>>` threaded through the application context. Globals for visual state cause race conditions in multi-thread renders and make theme hot-reload impossible.

- **Single-popup overlay field**: No `Option<PromptModal>` field for the permission overlay. Use `VecDeque<PromptModal>` to support concurrent prompts without silent drop. The single-Option pattern causes the second concurrent prompt to replace the first with no acknowledgment.

## Test-Time Enforcement

All five mechanisms below are wired in CI and block merge on failure. See CI Wiring section for step ordering.

### Clippy `disallowed_methods` Configuration

Add to workspace `Cargo.toml`:

```toml
[workspace.lints.clippy]
disallowed_methods = [
  { path = "tokio::sync::mpsc::unbounded_channel", reason = "Use bounded mpsc::channel(N) with surfaced drop counter per anti-patterns table" },
  { path = "std::fs::write", reason = "Use tempfile::persist for atomic config writes per anti-patterns table" },
  { path = "tokio::fs::write", reason = "Use tempfile::persist for atomic config writes per anti-patterns table" },
]
```

### Semgrep Rules

Write to `.semgrep.yml` at workspace root:

```yaml
rules:
  - id: monocle-no-shell-injection
    pattern-either:
      - pattern: Command::new("sh")
      - pattern: Command::new("bash")
    message: "Shell injection vector. Use Command::new(binary).args([...]) arg-array form. See conventions.md anti-patterns."
    severity: ERROR
    languages: [rust]
  - id: monocle-no-naked-fs-write
    pattern-either:
      - pattern: std::fs::write(...)
      - pattern: tokio::fs::write(...)
    message: "Naked file write leaves corruption window on crash. Use tempfile::persist. See conventions.md anti-patterns."
    severity: ERROR
    languages: [rust]
  - id: monocle-no-unbounded-channel
    pattern: tokio::sync::mpsc::unbounded_channel(...)
    message: "Unbounded channel masks backpressure. Use bounded mpsc::channel(N) with drop counter. See conventions.md anti-patterns."
    severity: ERROR
    languages: [rust]
```

### PR Template Checklist

Write to `.github/PULL_REQUEST_TEMPLATE.md`:

```markdown
## Monocle Convention Checklist
- [ ] All channel instantiations use `mpsc::channel(N)` with explicit bound N
- [ ] All config writes use `tempfile::persist` (or equivalent atomic-rename pattern)
- [ ] No `Command::new("sh")` or `Command::new("bash")` — use arg-array form
- [ ] No package-level mutable globals for theme/config — use `Arc<RwLock<>>` threaded through context
- [ ] All permission overlays use `VecDeque<PromptModal>`, not `Option<PromptModal>`
- [ ] All public APIs have explicit error taxonomy via `thiserror`
- [ ] No `println!` in production code paths (use `tracing` with structured fields)
```

### Channel-Drop Integration Test Skeleton

Add to `monocle-test-harness`:

```rust
#[tokio::test]
async fn synthetic_high_frequency_hook_load_does_not_overflow() {
    let (tx, mut rx) = mpsc::channel(1024);
    let drop_counter = Arc::new(AtomicU64::new(0));
    // sustain 1000 events/sec for 30s via spawned producer
    // assert: drop_counter.load(Ordering::Relaxed) <= 50
    // assert: rx drains all non-dropped events
}
```

Target: 1000 events/sec sustained for 30 seconds; drop counter must remain at or below 50 drops. This is the integration test anchor for the bounded channel anti-pattern enforcement.

### CI Wiring

GitHub Actions step ordering (all block merge on failure):

1. `cargo fmt --check` — block merge on format deviation
2. `cargo clippy --workspace -- -D warnings` — block on any lint, including `disallowed_methods`
3. `semgrep --config .semgrep.yml --error` — block on any rule match
4. `cargo test --workspace` — block on any test failure, including channel-drop integration test
5. `cargo audit` — block on new RUSTSEC advisory affecting pinned versions; run weekly scheduled via `cargo audit --json`

## Gene-Source Citations

| Anti-Pattern | Gene-Source Evidence |
|---|---|
| Shell injection | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §mixins-r1: Python `subprocess(shell=True)` with template string in CLAUDE.md injection path |
| Naked config file writes | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §services-r1..r3: atomic-write gap finding; `write_text` with no temp-file intermediary |
| Unbounded channels | `any-context-lazyclaude-pass-8-final-synthesis-v2.md` §broker-r1: BC-BROKER-003 documents unbounded channel as a confirmed failure mode in the broker subsystem; broker drops are completely silent (no log, no metric, no counter) per BC-BROKER-006 |
| Theme globals | `lazygit-pass-8-final-synthesis.md` §pkg/gui: package-level theme globals causing render-thread contention |
| Single-popup overlay | `lazygit-pass-8-final-synthesis.md` §pkg/gui popup: `Option<Popup>` drop-on-concurrent pattern; concurrent modal opens silently drop the pending prompt |
