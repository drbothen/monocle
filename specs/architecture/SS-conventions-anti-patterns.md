---
document_type: architecture-section
level: L3
section: "conventions"
version: "1.2"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-12T06:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
input-hash: "[live-state]"
traces_to: "adversary re-audit 0bd4ba9 §Top 8 CRITICAL/IMPORTANT item 3; canonical principle CLAUDE.md commit 3366d58; brief v1.4 commit 70286e1; vision v1.1 commit 0e4b0f4; adversary F-NEW-08 cargo-deny CI gate; ADR-0003 license selection"
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
5. `cargo deny check licenses bans advisories sources` — block on license policy violation, banned crate, active RUSTSEC advisory, or non-registry source; uses `deny.toml` at workspace root (see §deny.toml configuration below)
6. `cargo audit` — block on new RUSTSEC advisory affecting pinned versions; run weekly scheduled via `cargo audit --json`

#### deny.toml configuration

Place `deny.toml` at the workspace root. This file operationalizes ADR-0003 (MIT/Apache-2.0 dual-license selection) and enforces supply-chain hygiene.

```toml
[graph]
# Phase 1 targets — adjust at workspace init during /vsdd-factory:create-architecture
# when platform targets are pinned (e.g. ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin"])
targets = []
all-features = false

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"
ignore = []

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Unicode-3.0",
    "MPL-2.0",  # nucleo; file-level copyleft, does not propagate to binary
    "Zlib",
]
deny = ["GPL-1.0", "GPL-2.0", "GPL-3.0", "AGPL-1.0", "AGPL-3.0", "LGPL-2.0", "LGPL-2.1", "LGPL-3.0"]
copyleft = "warn"
allow-osi-fsf-free = "either"
confidence-threshold = 0.8
exceptions = []

[bans]
multiple-versions = "warn"
wildcards = "deny"
deny = [
    { name = "openssl", reason = "use rustls; openssl is a deployment-complexity liability" },
    { name = "openssl-sys", reason = "see openssl" },
    { name = "tokio", version = "<1.52", reason = "RUSTSEC remediated 1.52+" },
    { name = "russh", version = "<0.60", reason = "transitive rsa pre-release RUSTSEC-2023-0071" },
]
skip = []
skip-tree = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

**Rationale for `[bans]` deny entries:**

- **`openssl` / `openssl-sys`**: OpenSSL introduces system-library linkage that breaks hermetic builds and creates version-mismatch failures in CI containers and musl targets. monocle uses rustls (pure-Rust TLS) throughout; any transitive pull of openssl indicates a dependency tree problem to be resolved at the source, not papered over with feature flags.
- **`tokio < 1.52`**: RUSTSEC advisories were remediatedstarting in tokio 1.52. Any version below this floor represents a known-vulnerable async runtime. The `SS-deps-pin-manifest.md` already pins tokio at 1.44 for Phase 1 — this ban acts as a floor guard that will fire if a transitive dep drags in a pre-remediation version.
- **`russh < 0.60`**: russh versions below 0.60 pull in a pre-release `rsa` crate affected by RUSTSEC-2023-0071 (RSA key recovery via Marvin attack). russh is not a direct monocle dependency but may appear transiently through plugin SDK paths in Phase 3; the ban prevents accidental introduction.

**Rationale for MPL-2.0 inclusion:**

MPL-2.0 (Mozilla Public License 2.0) is file-level copyleft: it requires modifications to MPL-licensed files to be released under MPL, but does not propagate to files in different compilation units. monocle uses nucleo 0.5 (matcher/scorer library; see ADR-0002) which is MPL-2.0. Because monocle links nucleo as an unmodified library crate and does not modify nucleo source files, the file-level copyleft does not impose redistribution obligations on monocle's own code. The `copyleft = "warn"` setting ensures any new MPL-2.0 additions surface for human review.

**`targets = []` during Phase 1:**

The empty targets list instructs cargo-deny to analyze the dependency graph without restricting to a specific platform triple. This is correct during spec and early implementation phases when the exact deployment targets have not been finalized. During `/vsdd-factory:create-architecture` workspace initialization, the targets list will be populated with the pinned platform triples (e.g., `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`) to enable platform-specific dependency filtering.

#### GitHub Actions wiring

Use the `EmbarkStudios/cargo-deny-action` composite action, pinned to SHA. Place this step between `cargo test` and `cargo audit` in the CI workflow:

```yaml
- name: cargo-deny
  uses: EmbarkStudios/cargo-deny-action@v2
  with:
    log-level: warn
    command: check
    arguments: --all-features --workspace licenses bans advisories sources
```

Note: pin the `uses:` line to a full commit SHA when creating the workflow file (per the project's action-pinning requirement). The `@v2` reference above is illustrative; the devops-engineer must resolve the SHA at workflow creation time using `gh api repos/EmbarkStudios/cargo-deny-action/git/refs/tags/v2`.

### SBOM Generation

SBOM (Software Bill of Materials) generation via `cargo sbom` (or `cargo cyclonedx`) is run per-release tag in CI. Output format: CycloneDX JSON 1.6 (CISA-compliant; EU CRA-compliant). SBOM is attached to the GitHub Release artifact. Per-PR runs are NOT gated on SBOM (only on cargo-deny); SBOM is a release-time artifact, not a merge gate. License audit per merge is handled by cargo-deny (above).

## Gene-Source Citations

| Anti-Pattern | Gene-Source Evidence |
|---|---|
| Shell injection | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §mixins-r1: Python `subprocess(shell=True)` with template string in CLAUDE.md injection path |
| Naked config file writes | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §services-r1..r3: atomic-write gap finding; `write_text` with no temp-file intermediary |
| Unbounded channels | `any-context-lazyclaude-pass-8-final-synthesis-v2.md` §broker-r1: BC-BROKER-003 documents unbounded channel as a confirmed failure mode in the broker subsystem; broker drops are completely silent (no log, no metric, no counter) per BC-BROKER-006 |
| Theme globals | `lazygit-pass-8-final-synthesis.md` §pkg/gui: package-level theme globals causing render-thread contention |
| Single-popup overlay | `lazygit-pass-8-final-synthesis.md` §pkg/gui popup: `Option<Popup>` drop-on-concurrent pattern; concurrent modal opens silently drop the pending prompt |
