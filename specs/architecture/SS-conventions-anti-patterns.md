---
document_type: architecture-section
level: L3
section: "conventions"
version: "1.5"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-13T21:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
input-hash: "[live-state]"
traces_to: "adversary re-audit 0bd4ba9 §Top 8 CRITICAL/IMPORTANT item 3; canonical principle CLAUDE.md commit 3366d58; brief v1.4 commit 70286e1; vision v1.1 commit 0e4b0f4; adversary F-NEW-08 cargo-deny CI gate; ADR-0003 license selection; adversary F-R6-002 + consistency G-02 (round-6 bec535d); human Q-3 weekly R-001 monitoring; brief v1.4.6 §Competitive Positioning; v1.4 round-24 F-R24-adv-5: Test Conventions section added mandating temp-env for all env-mutating tests; v1.5 round-27: F-R26-adv-2 semgrep env-mutation pattern expanded (path-sensitive idioms); F-R26-adv-3 positive-coverage fixture corpus requirement added (POL-11); F-R26-adv-6 Test Conventions semgrep rule consolidated into §Semgrep Rules"
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

Write to `.semgrep.yml` at workspace root. All four rules below are authoritative; the
fourth rule (no-raw-env-mutation-in-tests) was added in v1.5 (consolidation of the §Test
Conventions CI enforcement rule to create a single source of truth). Cross-references:
§Test Conventions below cites this list as the canonical rule location.

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
  - id: monocle-no-raw-env-mutation-in-tests
    # Covers fully-qualified, module-relative, and use-alias import forms.
    # Rationale for pattern-either expansion (F-R26-adv-2):
    #   - `std::env::set_var(...)` matches only the fully-qualified path.
    #   - `use std::env; env::set_var(...)` is a common Rust idiom that writes
    #     `env::set_var(...)` — NOT matched by the fully-qualified pattern alone.
    #   - Bare-import form (`use std::env::set_var; set_var(...)`) is NOT covered
    #     because semgrep cannot disambiguate `set_var(...)` from a user-defined
    #     function of the same name without full type information. The bare-import
    #     form is documented as discouraged in §Test Conventions prose but is not
    #     enforced by semgrep (noise risk outweighs coverage). Developers using
    #     the bare-import form should use `#[allow]` sparingly only for legitimate
    #     use of a user-defined function named `set_var` or `remove_var`.
    pattern-either:
      - pattern: std::env::set_var($X, $Y)
      - pattern: std::env::remove_var($X)
      - pattern: env::set_var($X, $Y)
      - pattern: env::remove_var($X)
    paths:
      include:
        - "**/tests/**/*.rs"
        - "**/*_test.rs"
        - "**/*tests*.rs"
    message: "Raw env mutation in tests is unsafe in multi-threaded Rust harnesses (Rust 1.86+
      marks set_var/remove_var unsafe). Use temp_env::with_vars (sync) or
      temp_env::async_with_vars (async) from temp-env ^0.3. See SS-conventions-anti-patterns.md
      §Test Conventions."
    severity: ERROR
    languages: [rust]
```

### Semgrep Coverage Hardening (POL-11 positive-coverage requirement)

Semgrep rules that return zero findings on every CI run are unverifiable — the rule may be
silently broken (wrong path glob, incompatible pattern for the semgrep version in use, or
path scope that never matches any file). This section specifies the fixture corpus and CI
assertion requirements that give each rule a positive signal on every run.

#### Fixture corpus

The devops-engineer creates `semgrep-fixtures/` at the project root (NOT under `tests/`,
which is the Rust integration test crate). Each semgrep rule has exactly one corresponding
fixture file containing a deliberate violation of that rule.

| Rule ID | Fixture file | Violation |
|---------|-------------|-----------|
| `monocle-no-shell-injection` | `semgrep-fixtures/shell_injection.rs` | `Command::new("sh").arg("-c").arg("echo hi");` |
| `monocle-no-naked-fs-write` | `semgrep-fixtures/naked_fs_write.rs` | `std::fs::write("/tmp/x", b"data").unwrap();` |
| `monocle-no-unbounded-channel` | `semgrep-fixtures/unbounded_channel.rs` | `tokio::sync::mpsc::unbounded_channel::<u8>();` |
| `monocle-no-raw-env-mutation-in-tests` | `semgrep-fixtures/tests/raw_env_mutation.rs` | `std::env::set_var("HOME", "/tmp");` AND `env::set_var("HOME", "/tmp");` (both patterns exercised) |

Each fixture file contains ONLY the violation pattern (plus minimal Rust syntax to make it
parse). Fixture files are NOT part of the Rust workspace (`Cargo.toml` workspace members list
does not include `semgrep-fixtures/`); they exist solely as semgrep targets.

#### CI assertions (two steps)

The following CI step specifications are normative requirements for the devops-engineer to
wire into the GitHub Actions workflow at implementation time. The exact YAML belongs in
`.github/workflows/` (devops-engineer territory); this section specifies the behavioral
contract that YAML must satisfy.

**Step 1 — Fixture corpus scan (positive-coverage assertion):**

Run semgrep against `semgrep-fixtures/` only. For each rule, assert the finding count
equals the expected value defined in the table above. Emit a log line per rule:
`Fixture corpus: N violation(s) detected for rule <rule-id> (expected N) — PASS` or
`Fixture corpus: N violation(s) detected for rule <rule-id> (expected M) — FAIL`.
Fail the CI step if any rule's actual count does not equal the expected count.

The `monocle-no-raw-env-mutation-in-tests` rule must match 2 findings in its fixture
(one for each pattern in `pattern-either`: `std::env::set_var` and `env::set_var`).
`std::env::remove_var` and `env::remove_var` are implicitly covered by the fixture — the
devops-engineer may add them to the fixture file to make the 4-pattern coverage explicit,
adjusting the expected count accordingly (2 → 4).

**Step 2 — Production scan (zero-findings assertion):**

Run semgrep against the production Rust source (`src/`, `crates/`, or equivalent workspace
source directories — NOT `semgrep-fixtures/`, NOT `tests/` for the production-code rules).
Assert zero findings for each rule. Emit a log line per rule:
`Production scan: 0 violations for rule <rule-id> (clean)` or
`Production scan: N violations for rule <rule-id> — FAIL (see semgrep output above)`.
Fail the CI step if any rule returns a non-zero count.

**Step ordering in CI workflow:**

Both steps run after `cargo clippy` and before `cargo test`. They are separate CI steps
(distinct `name:` entries) so failures are individually addressable in the GitHub Actions
UI. The fixture-corpus step runs first; if it fails (rule broken), the production scan step
is skipped to avoid a false-clean result from a non-functioning rule.

**Note on scope:** The CI wiring (actual `.github/workflows/` YAML, fixture file content,
semgrep version pin) is the devops-engineer's Phase 1 deliverable. This section specifies
the behavioral requirement with enough precision that the implementer can wire it without
round-trips to the architect.

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
- **`tokio < 1.52`**: RUSTSEC advisories were remediated starting in tokio 1.52. Any version below this floor represents a known-vulnerable async runtime. The `SS-deps-pin-manifest.md` already pins tokio at 1.52 for Phase 1 — this ban acts as a floor guard that will fire if a transitive dep drags in a pre-remediation version.
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

## R-001 Monitoring Workflow

### Purpose

Weekly scheduled check of Anthropic agent-view release notes against the 4 R-001 re-eval trigger conditions. Opens a labeled GitHub Issue when any condition matches. Per brief v1.4.6 §Competitive Positioning.

This section specifies the workflow contract. The actual implementation files (`/.github/workflows/r001-monitor.yml` and `scripts/r001-monitor.py`) are created during Phase 1 `/vsdd-factory:create-architecture` by the devops-engineer. This is a SPEC, not the implementation.

### Trigger Conditions

Four conditions, any single match opens a GitHub Issue:

| ID | Condition | Rationale |
|---|---|---|
| (a) | Anthropic announces hook-protocol ingestion as a first-class agent-view capability | Directly commoditizes monocle's hook-event pipeline (BC-HOOK-001–006) |
| (b) | Anthropic ships diff-preview or cascaded permission-queue functionality inside agent view | Directly commoditizes monocle's permission overlay plane (VecDeque\<PromptModal\>) |
| (c) | Anthropic extends agent view beyond Claude Code (e.g., supports a non-Claude harness) | Attacks monocle's multi-harness federation differentiator |
| (d) | Anthropic publishes a multi-harness session-management spec or RFC | Pre-empts monocle's Phase 4 harness-federation roadmap |

### Workflow Specification — `.github/workflows/r001-monitor.yml`

```yaml
name: R-001 Monitor (Anthropic agent-view trigger conditions)

on:
  schedule:
    - cron: '0 14 * * 1'  # Every Monday at 14:00 UTC (10am ET / 7am PT)
  workflow_dispatch:  # Allow manual trigger for ad-hoc checks

permissions:
  contents: read
  issues: write

concurrency:
  group: r001-monitor
  cancel-in-progress: false

jobs:
  check-anthropic-release-notes:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - name: Install dependencies
        run: pip install requests beautifulsoup4
      - name: Fetch agent-view docs and release notes
        env:
          ANTHROPIC_DOCS_URL: 'https://code.claude.com/docs/en/agent-view'
          ANTHROPIC_RELEASE_NOTES_URL: 'https://docs.claude.com/en/release-notes/claude-code'
        run: python scripts/r001-monitor.py
      - name: Open issue on trigger match
        if: env.R001_TRIGGER_MATCHED == 'true'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const triggerReport = fs.readFileSync('r001-trigger-report.md', 'utf8');
            await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `[R-001 TRIGGER] Anthropic agent-view release matches re-eval condition`,
              body: triggerReport,
              labels: ['risk', 'r-001', 'competitive-monitoring']
            });
```

**Action pinning requirement:** The devops-engineer MUST resolve `actions/checkout@v4`, `actions/setup-python@v5`, and `actions/github-script@v7` to full commit SHAs at workflow creation time. The `@vN` references above are illustrative version pointers; the final file must pin to SHA per the project action-pinning requirement (see §CI Wiring).

### Companion Script Contract — `scripts/r001-monitor.py`

The script is specced here; the full Python implementation is the devops-engineer's Phase 1 deliverable.

**Inputs:** Two public URLs (set via env vars in the workflow):
- `ANTHROPIC_DOCS_URL`: `https://code.claude.com/docs/en/agent-view`
- `ANTHROPIC_RELEASE_NOTES_URL`: `https://docs.claude.com/en/release-notes/claude-code`

**Processing:**
1. Fetch both URLs via `requests`; extract visible text via BeautifulSoup HTML parsing.
2. For each of the 4 trigger conditions, evaluate keyword/regex patterns against the combined text corpus:
   - **(a)** keywords: `hook protocol`, `hook ingestion`, `hooks api`, `PreToolUse`, `PostToolUse`, `PermissionPrompt`
   - **(b)** keywords: `diff preview`, `diff overlay`, `permission queue`, `cascaded permission`, `permission stack`
   - **(c)** keywords: any harness name OTHER than `Claude Code` — e.g., `Codex`, `Aider`, `Cursor`, `Continue`, `GitHub Copilot` (case-insensitive, whole-word match to reduce false positives)
   - **(d)** keywords: `multi-harness`, `harness-agnostic`, `session management spec`, `session management RFC`, `agent-view spec`

**Outputs:**
- If ANY condition matches: set GitHub Actions env var `R001_TRIGGER_MATCHED=true` and write `r001-trigger-report.md` containing: matched condition ID(s), matched text excerpt(s) (up to 200 chars each), source URL, and fetched UTC timestamp (ISO 8601).
- Always (match or no match): append a JSON log entry to `r001-monitor-log/YYYY-MM-DD.json` recording: run timestamp, conditions checked, match results (true/false per condition), and fetch status (ok/unreachable/parse-error) per URL. This log file is committed back to a maintenance branch (`chore/r001-monitor-log`) so the historical trail is preserved even when no trigger fires.

### Failure Modes

| Failure | Behavior |
|---|---|
| URL unreachable | Log warning; do NOT set `R001_TRIGGER_MATCHED`; do NOT open trigger issue (no false positive) |
| HTML structure change (parsing yields empty or implausible content) | Log warning; open a SEPARATE maintenance issue labeled `r-001-monitor-broken` (distinct from trigger label `r-001`) |
| Cron timeout / workflow failure | Retry once via manual `workflow_dispatch` within 24h; if second run also fails, open maintenance issue labeled `r-001-monitor-broken` |

The maintenance issue label `r-001-monitor-broken` must be created in the repository at initialization time (different from the trigger label `r-001`). The devops-engineer creates both labels during Phase 1 repo setup.

### Quarterly Keyword Maintainer Review

The keyword patterns in §Companion Script Contract above are anchored to Anthropic's current terminology as of brief v1.4.6. Anthropic may introduce new wording (e.g., referring to hook-protocol ingestion under a new product name) that does not match current patterns, creating false-negative drift.

**Required cadence:** Review and update keyword patterns quarterly (every 13 weeks). Track via a recurring maintenance issue tagged `r-001-keywords-review`, created automatically by the workflow at each calendar quarter boundary (1 January, 1 April, 1 July, 1 October). The quarterly creation logic is embedded in `scripts/r001-monitor.py` using the run date.

**Review checklist:**
- [ ] Check Anthropic release notes for new terminology covering hook-protocol, diff-preview, multi-harness, or session-management surface areas.
- [ ] Update keyword lists in `scripts/r001-monitor.py` to cover new wording.
- [ ] Run workflow manually (`workflow_dispatch`) after update to verify no spurious matches on prior release notes.
- [ ] Commit updated script and close the `r-001-keywords-review` issue.

### Cost

GitHub Actions free tier covers weekly cron (`0 14 * * 1`) plus light Python compute (typically under 30 seconds per run). No external API costs — both source URLs are public Anthropic documentation. No GitHub API rate-limit risk: one issue creation per trigger event, well within the 5,000 requests/hour ceiling for `GITHUB_TOKEN`.

## Test Conventions

### Environment Variable Mutation in Tests

Tests that mutate `std::env` MUST use `temp-env 0.3+` (`with_vars` for sync closures;
`async_with_vars` for async closures). Raw `std::env::set_var` / `std::env::remove_var` is
**forbidden** in tests — it is unsound in multi-threaded Rust test harnesses (Rust 1.86+
marks these functions `unsafe` for exactly this reason), and lacks panic-safe cleanup.
`temp-env` provides RAII cleanup that fires on both normal return and panic exit.

**Forbidden pattern:**

```rust
// FORBIDDEN: raw env mutation — race-prone, leaks on panic
std::env::set_var("HOME", "/tmp/test-home");
// ... test body ...
std::env::remove_var("HOME"); // never executes if test panics
```

**Required pattern (sync closure):**

```rust
// REQUIRED: RAII cleanup on normal return AND panic
temp_env::with_vars(
    [("HOME", None::<&str>)],
    || {
        // test body — env is restored when closure exits
    },
);
```

**Required pattern (async closure — requires `features = ["async_closure"]`):**

```rust
// REQUIRED for async test bodies
temp_env::async_with_vars(
    [("HOME", None::<&str>)],
    async {
        // async test body — env is restored when future completes or panics
    },
).await;
```

**Dependency declaration** (`monocle-runtime/Cargo.toml` `[dev-dependencies]`):

```toml
temp-env = { version = "^0.3", features = ["async_closure"] }
```

The `async_closure` feature must always be enabled even if only sync closures are used in
a given crate's tests, to keep the dev-dependency declaration consistent across all crates
that add `temp-env`.

**Canonical usage example:** BC-ENGINE-002-ERR test in
`monocle-runtime/tests/engine_module.rs` (see SS-engine-module.md §Behavioral Contracts).

**CI enforcement:** Rule `monocle-no-raw-env-mutation-in-tests` is the 4th semgrep rule
in §Semgrep Rules above — it is the canonical single-source-of-truth location for this
rule. The rule uses `pattern-either` to cover both the fully-qualified form
(`std::env::set_var`, `std::env::remove_var`) and the module-relative form
(`env::set_var`, `env::remove_var`), scoped to test file paths only via `paths.include`.
See §Semgrep Rules for the full rule definition and §Semgrep Coverage Hardening for the
positive-coverage fixture corpus requirement (POL-11).

**Bare-import form** (`use std::env::set_var; set_var(...)`): not enforced by semgrep
(cannot disambiguate from a user-defined function without full type resolution). This form
is also discouraged; developers using it must use `temp_env::with_vars` anyway. If a
false-positive suppression is needed for a legitimate user-defined `set_var` function,
add a `# nosemgrep: monocle-no-raw-env-mutation-in-tests` comment — NOT `#[allow]`.

## Gene-Source Citations

| Anti-Pattern | Gene-Source Evidence |
|---|---|
| Shell injection | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §mixins-r1: Python `subprocess(shell=True)` with template string in CLAUDE.md injection path |
| Naked config file writes | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §services-r1..r3: atomic-write gap finding; `write_text` with no temp-file intermediary |
| Unbounded channels | `any-context-lazyclaude-pass-8-final-synthesis-v2.md` §broker-r1: BC-BROKER-003 documents unbounded channel as a confirmed failure mode in the broker subsystem; broker drops are completely silent (no log, no metric, no counter) per BC-BROKER-006 |
| Theme globals | `lazygit-pass-8-final-synthesis.md` §pkg/gui: package-level theme globals causing render-thread contention |
| Single-popup overlay | `lazygit-pass-8-final-synthesis.md` §pkg/gui popup: `Option<Popup>` drop-on-concurrent pattern; concurrent modal opens silently drop the pending prompt |
| Raw env mutation in tests | `monocle` round-21 adversary trace + round-24 F-R24-adv-1: `std::env::set_var`/`remove_var` is unsound in multi-threaded test harnesses; data-race on `HOME` between concurrent test threads causes non-deterministic failures; cleanup leaks on panic; `temp-env` is the canonical RAII fix |

## §Trace

v1.5 changes (round-27 fixes F-R26-adv-2 MEDIUM / F-R26-adv-3 MEDIUM / F-R26-adv-6 LOW):
- F-R26-adv-2 RESOLVED (MEDIUM — adversary finding): The `monocle-no-raw-env-mutation-in-tests`
  semgrep rule previously matched only the fully-qualified path form `std::env::set_var(...)`.
  The common Rust idiom `use std::env; env::set_var(...)` writes `env::set_var(...)` and was
  not matched — creating a false-green where tests using the module-relative form silently
  bypassed CI enforcement. Fix: rule expanded from two `pattern-either` entries to four,
  adding `env::set_var($X, $Y)` and `env::remove_var($X)` alongside the fully-qualified
  forms. Bare-import form (`use std::env::set_var; set_var(...)`) is explicitly NOT covered
  by semgrep (documented in both §Semgrep Rules and §Test Conventions with rationale: would
  collide with user-defined functions of the same name; prose discourages it; nosemgrep
  comment is the suppression mechanism for legitimate user-defined functions).
- F-R26-adv-3 RESOLVED (MEDIUM process-gap — adversary finding): All four semgrep rules
  could produce zero findings on every CI run without indicating a problem — the rule may
  be silently broken (wrong path glob, incompatible semgrep pattern syntax for the pinned
  semgrep version, or pattern-either that matches nothing because of Rust AST differences).
  Fix: added §Semgrep Coverage Hardening subsection (between §Semgrep Rules and §PR Template
  Checklist) specifying: (1) `semgrep-fixtures/` directory with one fixture file per rule;
  (2) CI Step 1 — fixture corpus scan asserting each rule produces the expected non-zero
  finding count; (3) CI Step 2 — production scan asserting zero findings per rule; (4) log
  line format for both steps; (5) step ordering (fixture step first; skip production scan
  if fixture step fails). The CI wiring (actual workflow YAML, fixture file content, semgrep
  version pin) is the devops-engineer's Phase 1 deliverable — this section specifies the
  behavioral requirement with enough precision for implementation without architect round-trips.
- F-R26-adv-6 RESOLVED (LOW — adversary finding): The `monocle-no-raw-env-mutation-in-tests`
  semgrep rule was defined in two places: partially in §Semgrep Rules (via a trailing comment
  "Add this rule to .semgrep.yml") and fully in §Test Conventions (as a YAML block). This
  created a single-source-of-truth violation: a future editor might update one copy but not
  the other. Fix: the canonical rule definition is now in §Semgrep Rules as the 4th rule
  (complete YAML block). The §Test Conventions §CI enforcement paragraph now cross-references
  §Semgrep Rules as the canonical location rather than duplicating the YAML. The rule's
  expanded pattern-either (from F-R26-adv-2) is defined once in §Semgrep Rules only.

v1.4 changes (round-24 fix F-R24-adv-5):
- F-R24-adv-5 RESOLVED (LOW process-gap — adversary finding): BC-ENGINE-002-ERR
  established a precedent for `temp-env` usage in env-mutating tests, but the
  conventions doc did not codify the rule. Future tests for env-sensitive code paths
  (e.g., `MONOCLE_NO_AUTOSTART`, `CLAUDE_SESSION_ID`, daemon lock-file path resolution)
  could silently regress to raw `std::env::set_var`/`remove_var` — which Rust 1.86+
  marks `unsafe` in multi-threaded contexts and which lacks panic-safe cleanup. Fix:
  added §Test Conventions subsection with forbidden/required patterns, Cargo.toml
  declaration, canonical usage example pointer (BC-ENGINE-002-ERR), and a semgrep CI
  rule that rejects raw env mutation in test files. The semgrep rule is scoped to test
  file paths only to avoid false positives on production code that reads (but does not
  write) env vars (e.g., daemon start sequence `std::env::var("CLAUDE_SESSION_ID")`).
