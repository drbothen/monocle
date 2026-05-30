---
document_type: story
level: L4
story_id: S-001
epic_id: EPIC-01
version: "1.10"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-20T00:00:00Z
phase: 2
points: 5
wave: 1
tdd_mode: facade
priority: P0
depends_on: []
blocks: [S-002, S-003, S-004, S-005, S-006, S-009, S-010]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: []
verification_properties: []
estimated_days: 2
# BC status: no BC-S.SS.NNN covers CI/devops workspace setup. NFR-007 and NFR-008 are CI
# gate deliverables validated by green CI builds, not VP probes. BC-2.01.007 (JSONL ring) is
# implemented exclusively by S-008; this story only establishes the workspace that S-008 compiles in.
# S-009 included in blocks (Decision 10): S-009 directly consumes S-001's workspace + axum router
# foundation. The r01 partial-fix that removed S-009 from S-001 blocks was incomplete — the
# depends_on/blocks sibling propagation was skipped. Per SE-25 bidirectional DAG symmetry
# requirement: every depends_on entry must have a matching blocks entry on the depended-on story.
# S-013 and S-014 REMOVED from blocks (Decision 11 / F-PHASE2-R10-01): both stories depend on
# S-010 (not S-001 directly). Bidirectional check: S-013.depends_on=[S-010]; S-014.depends_on=[S-010].
# Transitive chain S-001 → S-010 → {S-013, S-014} preserves topological ordering without
# spurious direct edges. S-011/S-012 are identical pattern — absent from S-001.blocks by precedent.
inputs:
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.0"}
  - {path: .factory/specs/architecture/SS-conventions-anti-patterns.md, version: "1.30.2"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
input-hash: "[live-state]"
traces_to: "Implements NFR-007 (CI green-builds, MSRV pin), NFR-008 (build-time matrix); establishes workspace structure invariants for all Phase 1 crates; enforces SS-deps-pin-manifest.md v1.2.0 EXACT-pin policy."
---

# S-001: Cargo Workspace Init + CI/DevOps Setup

## Narrative

As a developer on the monocle project, I want the Rust workspace initialized with all
crates, toolchain pinned to MSRV 1.88, and a CI matrix covering macOS + Linux
(darwin/linux × amd64/arm64), so that every subsequent story can compile, test, and
deliver in a reproducible environment.

## Acceptance Criteria

### AC-001 (NFR-007 + NFR-008 validation gate — workspace compiles on all matrix targets)
`cargo build --workspace` succeeds from the project root on both macOS (darwin/arm64) and
Linux (linux/amd64 and linux/arm64) without errors or warnings under `cargo clippy --workspace -- -D warnings`.
This establishes the workspace that all subsequent stories (S-002 through S-015) compile within.

### AC-002 (NFR-007 validation gate — devops CI artifact, not VP)
`rust-toolchain.toml` at the workspace root pins `channel = "1.88"`. `cargo check`
with this toolchain verifies MSRV 1.88 is satisfied. Full contents:
```toml
[toolchain]
channel = "1.88"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

### AC-003 (NFR-008 validation gate — devops CI artifact, not VP)
GitHub Actions CI workflow file exists at `.github/workflows/ci.yml` using explicit
`include:` blocks (no Cartesian product matrix) as follows:
```yaml
matrix:
  include:
    - runner: macos-14
      target: aarch64-apple-darwin
    - runner: ubuntu-24.04
      target: x86_64-unknown-linux-gnu
    - runner: ubuntu-24.04-arm
      target: aarch64-unknown-linux-gnu
```
All three matrix jobs are green on first push. Runner images: `macos-14` (Apple Silicon
native), `ubuntu-24.04` (x86_64), `ubuntu-24.04-arm` (Linux ARM64 native GitHub-hosted
runner, GA 2025-05).

### AC-004 (NFR-007 enforcement)
`Cargo.toml` workspace `rust-version = "1.88"` field is set. CI includes a
`lint-toolchain` step:
```bash
test -f rust-toolchain.toml && grep -Eq '^channel = "1\.88"$' rust-toolchain.toml
```
CI fails if `rust-toolchain.toml` is absent or if toolchain pin does not match "1.88".

### AC-005 (workspace structure invariant — correct Phase 1 crate member list; Orchestrator Decision 3)
The workspace declares exactly these 3 crates as Phase 1 members: `monocle-core`,
`monocle-runtime`, `monocle-proto`. `monocle-auth` is NOT a separate workspace crate —
`generate_session_token() -> String` lives in `monocle-runtime::auth` module (new module
inside `monocle-runtime`, not a new crate). This decision was made because `monocle-auth`
appears in NO architectural source-of-truth; SS-deps-pin-manifest.md already declares
`runtime --> rand` as the canonical `OsRng` consumer edge; no new crate is justified for a
one-function helper. `monocle-tui` is NOT declared as a Phase 1 workspace member per
product-brief.md Phase 1 scope.

### AC-006 (dependency manifest compliance — SS-deps-pin-manifest.md v1.2.0)
`Cargo.toml` workspace `[workspace.dependencies]` table uses the Cargo 2021+ recommended
pattern: workspace-level declarations, member crates inherit via `{ workspace = true }`.
The following EXACT-pinned security-sensitive crates (per Patch-Pinning Policy, 9 crates)
are declared in workspace `[workspace.dependencies]`:
- `tokio = { version = "=1.52.0", features = ["full"] }` (EXACT, full SemVer triplet)
- `axum = "=0.8.9"` (EXACT)
- `serde_json = "=1.0.149"` (EXACT)
- `rand = "=0.8.6"` (EXACT)
- `prost = "=0.14.1"` (EXACT; workspace-declared for monocle-proto crate per SS-deps-pin-manifest.md v1.2.0 L33-74)
- `bytes = "1.11"` (caret; direct workspace pin overrides prost-transitive; closes RUSTSEC-2026-0007 per SS-deps-pin-manifest.md §RUSTSEC Audit Context)
- `wasmtime = "=44.0.1"` (EXACT; workspace-declared but NOT added to any Phase 1 member crate's `[dependencies]`; activated at Phase 3 plugin SDK boundary)
- `russh = "=0.60.2"` (EXACT; workspace-declared but NOT added to any Phase 1 member crate's `[dependencies]`; activated at Phase 4 federation boundary)
- `reqwest = "=0.13.0"` (EXACT; workspace-declared; crate usage activated when needed by S-009 and other stories)

NOTE: `rmcp` is OMITTED from Phase 1 workspace entirely per OQ-09 (Phase 4 only).

All other Phase 1 crates use caret pins per SS-deps-pin-manifest.md v1.2.0 L33-74 (Phase 1 Pin Manifest table).
`temp-env = { version = "^0.3", features = ["async_closure"] }` MUST be declared in
`monocle-runtime/Cargo.toml` `[dev-dependencies]` (NOT workspace dependencies; it is a
test-only crate). Pin: caret `^0.3` per SS-deps-pin-manifest.md v1.2.0 L33-74.

### AC-007 (cargo audit CI gate)
`.github/workflows/audit.yml` exists with `cron: '0 0 * * 0'` (weekly) running
`cargo audit --deny warnings`. The workflow installs `cargo-audit` via
`cargo install cargo-audit --locked` before invocation.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,100 |
| SS-deps-pin-manifest.md v1.2.0 (full) | ~9,976 |
| SS-daemon-lifecycle.md v1.0.33 (workspace scope section) | ~2,000 |
| SS-conventions-anti-patterns.md v1.29.5 (CI enforcement section) | ~1,000 | <!-- version-pin-historical: at S-001 authoring time -->
| Cargo.toml template + toolchain files | ~500 |
| Test scaffolding | ~300 |
| **Total estimate** | **~14,876** |

Well within 20% of 200k context window. No split required.

## Tasks

- [ ] Create `Cargo.toml` workspace manifest using `[workspace.dependencies]` pattern (Cargo 2021+);
  member crates inherit via `{ workspace = true }` in their individual `[dependencies]`
  (monocle-auth is NOT a separate crate; Decision 3: `generate_session_token()` lives in `monocle-runtime::auth` module)
- [ ] Set `rust-version = "1.88"` in workspace `Cargo.toml`
- [ ] Create `rust-toolchain.toml` with contents:
  ```toml
  [toolchain]
  channel = "1.88"
  components = ["clippy", "rustfmt"]
  profile = "minimal"
  ```
- [ ] Declare all 9 EXACT-pin security-sensitive crates in workspace `[workspace.dependencies]`
  per SS-deps-pin-manifest.md v1.2.0 L33-74 (Phase 1 Pin Manifest table)
- [ ] Declare `wasmtime = "=44.0.1"` and `russh = "=0.60.2"` in workspace `[workspace.dependencies]`
  but do NOT add them to any Phase 1 member crate's `[dependencies]`; they are workspace-declared
  for Phase 3 (wasmtime plugin SDK) and Phase 4 (russh federation) availability
- [ ] Declare `bytes = "1.11"` directly in workspace `[workspace.dependencies]` to close
  RUSTSEC-2026-0007 (overrides prost-transitive resolution per SS-deps-pin-manifest.md §RUSTSEC Audit Context)
- [ ] Add `prost-build = "=0.14.1"` as `[build-dependencies]` in `monocle-proto/Cargo.toml`
- [ ] Create `.github/workflows/ci.yml` with explicit `include:` matrix (not Cartesian product):
  macos-14/aarch64-apple-darwin, ubuntu-24.04/x86_64-unknown-linux-gnu, ubuntu-24.04-arm/aarch64-unknown-linux-gnu
- [ ] Add `lint-toolchain` CI step: `test -f rust-toolchain.toml && grep -Eq '^channel = "1\.88"$' rust-toolchain.toml`
- [ ] Add `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` to CI
- [ ] Create `.github/workflows/audit.yml` with weekly cron (`0 0 * * 0`), `cargo install cargo-audit --locked`, `cargo audit --deny warnings`
- [ ] Note: Native runners only — no `.cargo/config.toml` cross-linker block required for Phase 1
  (macos-14 is Apple Silicon native; ubuntu-24.04-arm is Linux ARM64 native GitHub-hosted runner)
- [ ] Create stub `lib.rs` for each crate (empty `pub mod`) so workspace compiles
  - monocle-core: `pub mod engine; pub mod factory; pub mod abi;`
    (Phase 1 modules per SS-core-types-and-abi.md v1.2.13 §Module Layout)
  - monocle-runtime: empty lib root
  - monocle-proto: empty lib root
- [ ] Create `monocle-runtime/src/main.rs` no-op stub (replaced by S-002+):
  ```rust
  //! monocle-runtime binary entry. Stub from S-001; daemon wired in S-002.
  #![forbid(unsafe_code)]
  #![deny(missing_docs)]
  fn main() {
      // Intentional no-op stub. S-002 will wire the daemon entry point.
  }
  ```
  Note: `println!("monocle-runtime stub")` is forbidden per SS-conventions-anti-patterns.md <!-- version-pin-historical: at S-001 authoring time -->
  v1.30.2 §Convention Checklist L503 ban on println! in production code paths (enforced by
  clippy.toml disallowed_methods extension). The stub is a no-op until S-002 wires daemon entry.
  `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` crate-level lints are included per
  SS-conventions production-grade default for all binary crates.
- [ ] Create `monocle-proto/build.rs` stub: `fn main() {}` (no-op; no `.proto` files in Phase 1 per S-013 which produces them)
- [ ] Run `cargo build --workspace` locally and confirm zero errors

## Previous Story Intelligence

N/A — first story in monocle Phase 2. No predecessor stories in this epic.

## Architecture Compliance Rules

From `architecture/SS-deps-pin-manifest.md` v1.2.0 L33-74 (Phase 1 Pin Manifest table):
- EXACT pin for 9 security-sensitive crates: `tokio`, `axum`, `serde_json`, `rand`, `prost`, `russh`, `rmcp`, `reqwest`, `wasmtime`
- All EXACT pins must use full SemVer triplet form (`=x.y.z`, not `=x.y`)
- Caret pin for all other Phase 1 crates
- MSRV: Rust 1.88 (ratatui 0.30 floor)
- `wasmtime 44` and `russh 0.60.2` declared in workspace `[workspace.dependencies]` but NOT activated in Phase 1 member crates
- `rmcp 1.6` OMITTED from Phase 1 workspace entirely (Phase 4 scope)
- `bytes = "1.11"` must be direct workspace dep to override prost-transitive RUSTSEC-2026-0007
- Use `[workspace.dependencies]` pattern; member crates use `{ workspace = true }`

From `architecture/SS-conventions-anti-patterns.md` v1.30.2:
- `cargo clippy --workspace -- -D warnings` is the enforcement gate
- `cargo fmt --all` required; CI fails on format divergence

From `architecture/SS-core-types-and-abi.md` v1.2.13 §Module Layout:
- monocle-core Phase 1 modules: `engine`, `factory`, `abi`

**Forbidden Dependencies:**
- `monocle-core` MUST NOT depend on `monocle-runtime` (would create a cycle)
- `monocle-runtime` MUST NOT depend on `monocle-tui` (not a Phase 1 crate)
- No crate may depend on `rmcp` in Phase 1 workspace
- `monocle-auth` MUST NOT appear as a workspace member or crate dependency (Decision 3; function is `monocle_runtime::auth::generate_session_token()` inside `monocle-runtime`)

## Library & Framework Requirements

| Crate | Version | Pin Type | Cargo.toml entry |
|-------|---------|----------|-----------------|
| axum | 0.8.9 | EXACT | `axum = "=0.8.9"` |
| tokio | 1.52.0 | EXACT | `tokio = { version = "=1.52.0", features = ["full"] }` |
| serde_json | 1.0.149 | EXACT | `serde_json = "=1.0.149"` |
| rand | 0.8.6 | EXACT | `rand = "=0.8.6"` |
| prost | 0.14.1 | EXACT | `prost = "=0.14.1"` (monocle-proto only; workspace-declared but only activated in monocle-proto crate per S-013) |
| reqwest | 0.13.0 | EXACT | `reqwest = "=0.13.0"` |
| wasmtime | 44.0.1 | EXACT | `wasmtime = "=44.0.1"` (workspace-declared; NOT activated in Phase 1 member crates) |
| russh | 0.60.2 | EXACT | `russh = "=0.60.2"` (workspace-declared; NOT activated in Phase 1 member crates) |
| bytes | 1.11 | caret | `bytes = "1.11"` (direct workspace pin; overrides prost-transitive; closes RUSTSEC-2026-0007) |
| serde | 1 | caret | `serde = { version = "1", features = ["derive"] }` |
| tracing | 0.1 | caret | `tracing = "0.1"` |
| thiserror | 2 | caret | `thiserror = "2"` |
| anyhow | 1 | caret | `anyhow = "1"` |
| tempfile | 3 | caret | `tempfile = "3"` |
| directories | 6 | caret | `directories = "6"` |
| chrono | 0.4 | caret | `chrono = "0.4"` |
| nix | 0.30 | caret | `nix = "0.30"` |
| constant_time_eq | 0.3 | caret | `constant_time_eq = "0.3"` |
| futures | 0.3 | caret | `futures = "0.3"` |
| async-trait | 0.1 | caret | `async-trait = "0.1"` |
| clap | 4.6 | caret | `clap = "4.6"` (CLI argument parsing for monocle-runtime binary entry) |
| interprocess | 2.4 | caret | `interprocess = "2.4"` (Unix domain socket IPC; activated by monocle-ipc) |
| semver | 1 | caret | `semver = "1"` (semantic version parsing; workspace-declared) |
| notify | 8 | caret | `notify = "8"` (filesystem watcher; workspace-declared; activated by Phase 3 monocle-workflow) |
| serde_yaml_ng | 0.10 | caret | `serde_yaml_ng = "0.10"` (YAML config parsing; activated by monocle-config) |
| temp-env (dev only) | 0.3 | caret | `temp-env = { version = "^0.3", features = ["async_closure"] }` in `monocle-runtime/Cargo.toml` `[dev-dependencies]` |
| syn (dev only) | 2.0 | caret | `syn = { version = "2", features = ["full"] }` in monocle-core, monocle-runtime `[dev-dependencies]` (AST audit tests; S-011/S-014) |

## File Structure Requirements

Files to create:
- `/Cargo.toml` — workspace manifest with `[workspace]`, `[workspace.dependencies]`, `rust-version = "1.88"`
- `/rust-toolchain.toml` — `[toolchain] channel = "1.88" / components = ["clippy", "rustfmt"] / profile = "minimal"`
- `/.github/workflows/ci.yml` — CI matrix workflow with explicit `include:` blocks
- `/.github/workflows/audit.yml` — weekly `cargo audit --deny warnings` scheduled workflow
- `/monocle-core/Cargo.toml` — crate manifest (workspace = true pattern)
- `/monocle-core/src/lib.rs` — stub: `pub mod engine; pub mod factory; pub mod abi;`
- `/monocle-runtime/Cargo.toml` — crate manifest (workspace = true pattern)
- `/monocle-runtime/src/main.rs` — no-op stub:
  ```rust
  //! monocle-runtime binary entry. Stub from S-001; daemon wired in S-002.
  #![forbid(unsafe_code)]
  #![deny(missing_docs)]
  fn main() {
      // Intentional no-op stub. S-002 will wire the daemon entry point.
  }
  ```
  Note: original v1.7 spec mandated `println!("monocle-runtime stub");` — removed per
  SS-conventions v1.30.2 ban on println! in production code paths. The stub is a no-op
  until S-002 wires daemon entry.
- `/monocle-runtime/src/lib.rs` — stub lib root
- `/monocle-proto/Cargo.toml` — crate manifest with `prost-build = "=0.14.1"` in `[build-dependencies]`
- `/monocle-proto/src/lib.rs` — stub
- `/monocle-proto/build.rs` — no-op stub: `fn main() {}`

## §Trace

**v1.10** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at story authoring time. No normative content changed.

**v1.9** (2026-05-29) — Path B Wave 6 MSRV propagation tail: SS-deps-pin-manifest.md v1.1.19 → v1.2.0 input pin bump; all active body MSRV 1.86 → 1.88 propagation (11 sites: narrative, AC-002 ×3, AC-004 ×3, AC-006 header, Architecture Compliance Rules, Tasks ×2, File Structure ×2; lint-toolchain grep pattern updated to "1.88"). inputs.SS-deps-pin-manifest bumped v1.1.19 → v1.2.0. traces_to manifest version updated to v1.2.0. All §Trace 1.86 entries preserved as historical records. Closes consumer-story cascade started at architect f3533ce.

**v1.8** (2026-05-20) — main.rs body spec updated to no-op stub form. Removed `println!("monocle-runtime stub")` per SS-conventions-anti-patterns.md v1.30.2 §Convention Checklist L503 ban (now enforced by clippy.toml disallowed_methods extension). Added `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]` crate lints per HIGH-2 sibling-sweep gap. inputs.SS-conventions-anti-patterns bumped v1.29.5 → v1.30.2. Source: PR #2 commit b7ed1e2 + .factory/plans/adversary-pass-PR2-round-3.md HIGH-1.

**v1.7** (2026-05-20) — Sibling-sweep update for SS-deps-pin-manifest v1.1.19 Option B (bytes pin "1.10" → "1.11" per RUSTSEC-2026-0007 fix-from = 1.11.1; production-grade default). 4 body sites updated: AC-006 bullet, Tasks declare step, Architecture Compliance Rules, Library & Framework Requirements table. inputs.SS-deps-pin-manifest bumped v1.1.18 → v1.1.19; traces_to manifest version updated to v1.1.19.

**v1.6** (2026-05-20) — Phase 3.B Batch 1 spec-reviewer remediation (F-A-01..F-D-06 findings from cycle-001 Stage-1 review). Refs: drbothen/vsdd-factory#150.
- F-A-01 CLOSED: tokio pin updated to canonical full SemVer triplet `=1.52.0` in AC-006 and Library table.
- F-A-02 CLOSED: prost `=0.14.1` EXACT pin added to AC-006 and Library table; prost-build added to monocle-proto build-dependencies task.
- F-A-03 CLOSED: `bytes = "1.10"` direct workspace caret pin added to AC-006 and Library table; RUSTSEC-2026-0007 closure noted.
- F-A-04 CLOSED: wasmtime + russh workspace-declared-not-activated mechanism made explicit in AC-006 and Tasks.
- F-A-05 CLOSED: rust-toolchain.toml full contents (channel + components + profile) specified in AC-002 and Tasks.
- F-A-06 CLOSED: clap, tracing, interprocess, semver, notify, serde_yaml_ng added to Library table with purpose notes.
- F-B-01/B-02 CLOSED: SS-deps-pin-manifest references updated to `v1.1.18 L33-74`.
- F-C-01 CLOSED: Runner images specified explicitly (macos-14, ubuntu-24.04, ubuntu-24.04-arm) in AC-003.
- F-C-02 CLOSED: CI matrix rewritten as explicit `include:` blocks (no Cartesian product) in AC-003.
- F-C-03 CLOSED: `lint-toolchain` CI step added to AC-004 and Tasks.
- F-C-04 CLOSED: AC-007 added (audit.yml with weekly cron, `cargo install cargo-audit --locked`).
- F-D-01 CLOSED: monocle-core module list cited with SS-core-types-and-abi.md v1.2.13 §Module Layout anchor.
- F-D-02 CLOSED: monocle-runtime/src/main.rs stub content specified in Tasks.
- F-D-03 CLOSED: monocle-proto/build.rs no-op stub specified in Tasks.
- F-D-04 CLOSED: `.cargo/config.toml` cross-linker block explicitly noted as not required (native runners).
- F-D-05 CLOSED: `[workspace.dependencies]` pattern mandated in AC-006 and Tasks.
- F-D-06 CLOSED: cargo-audit install step added to AC-007 and Tasks.
- inputs.SS-deps-pin-manifest bumped from v1.1.17 → v1.1.18.
