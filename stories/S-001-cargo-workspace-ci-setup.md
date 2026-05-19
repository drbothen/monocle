---
document_type: story
story_id: S-001
epic_id: EPIC-01
version: "1.5"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
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
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
  - {path: .factory/specs/architecture/SS-conventions-anti-patterns.md, version: "1.29.5"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
input-hash: "[live-state]"
traces_to: "Implements NFR-007 (CI green-builds, MSRV pin), NFR-008 (build-time matrix); establishes workspace structure invariants for all Phase 1 crates; enforces SS-deps-pin-manifest.md v1.1.17 EXACT-pin policy."
---

# S-001: Cargo Workspace Init + CI/DevOps Setup

## Narrative

As a developer on the monocle project, I want the Rust workspace initialized with all
crates, toolchain pinned to MSRV 1.86, and a CI matrix covering macOS + Linux
(darwin/linux × amd64/arm64), so that every subsequent story can compile, test, and
deliver in a reproducible environment.

## Acceptance Criteria

### AC-001 (NFR-007 + NFR-008 validation gate — workspace compiles on all matrix targets)
`cargo build --workspace` succeeds from the project root on both macOS (darwin/arm64) and
Linux (linux/amd64 and linux/arm64) without errors or warnings under `cargo clippy --workspace -- -D warnings`.
This establishes the workspace that all subsequent stories (S-002 through S-015) compile within.

### AC-002 (NFR-007 validation gate — devops CI artifact, not VP)
`rust-toolchain.toml` at the workspace root pins `channel = "1.86"`. `cargo check`
with this toolchain verifies MSRV 1.86 is satisfied.

### AC-003 (NFR-008 validation gate — devops CI artifact, not VP)
GitHub Actions CI workflow file exists at `.github/workflows/ci.yml` with a build matrix
of `[os: [macos-latest, ubuntu-latest]] × [target: [aarch64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu]]`
and all matrix jobs are green on first push.

### AC-004 (NFR-007 enforcement)
`Cargo.toml` workspace `rust-version = "1.86"` field is set. CI fails if
`rust-toolchain.toml` is absent or if toolchain pin does not match "1.86".

### AC-005 (workspace structure invariant — correct Phase 1 crate member list; Orchestrator Decision 3)
The workspace declares exactly these 3 crates as Phase 1 members: `monocle-core`,
`monocle-runtime`, `monocle-proto`. `monocle-auth` is NOT a separate workspace crate —
`generate_session_token() -> String` lives in `monocle-runtime::auth` module (new module
inside `monocle-runtime`, not a new crate). This decision was made because `monocle-auth`
appears in NO architectural source-of-truth; SS-deps-pin-manifest.md already declares
`runtime --> rand` as the canonical `OsRng` consumer edge; no new crate is justified for a
one-function helper. `monocle-tui` is NOT declared as a Phase 1 workspace member per
product-brief.md Phase 1 scope.

### AC-006 (dependency manifest compliance — SS-deps-pin-manifest.md v1.1.17)
`Cargo.toml` workspace `[dependencies]` table pins the following crates at exact versions
as required by the Patch-Pinning Policy:
- `axum = "=0.8.9"` (EXACT)
- `tokio = "=1.52"` (EXACT, full feature set)
- `serde_json = "=1.0.149"` (EXACT)
- `rand = "=0.8.6"` (EXACT)
All other Phase 1 crates use caret pins per SS-deps-pin-manifest.md §Phase 1 Pin Manifest.
`temp-env = { version = "^0.3", features = ["async_closure"] }` MUST be declared in
`monocle-runtime/Cargo.toml` `[dev-dependencies]` (NOT workspace dependencies; it is a
test-only crate). Pin: caret `^0.3` per SS-deps-pin-manifest.md §Phase 1 Pin Manifest
(temp-env entry: "caret pin (`^0.3`); feature `async_closure` required for `async_with_vars` API").

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~900 |
| SS-deps-pin-manifest.md v1.1.17 (full) | ~9,976 |
| SS-daemon-lifecycle.md v1.0.33 (workspace scope section) | ~2,000 |
| SS-conventions-anti-patterns.md v1.29.5 (CI enforcement section) | ~1,000 |
| Cargo.toml template + toolchain files | ~500 |
| Test scaffolding | ~300 |
| **Total estimate** | **~14,676** |

Well within 20% of 200k context window. No split required.

## Tasks

- [ ] Create `Cargo.toml` workspace manifest with member crates `monocle-core`, `monocle-runtime`, `monocle-proto`
  (monocle-auth is NOT a separate crate; Decision 3: `generate_session_token()` lives in `monocle-runtime::auth` module)
- [ ] Set `rust-version = "1.86"` in workspace `Cargo.toml`
- [ ] Create `rust-toolchain.toml` with `channel = "1.86"`
- [ ] Pin all 9 EXACT-pin security-sensitive crates in workspace `[dependencies]`
- [ ] Create `.github/workflows/ci.yml` with `[darwin, linux] × [amd64, arm64]` matrix
- [ ] Verify `cargo build --workspace` succeeds on all matrix targets
- [ ] Add `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` to CI
- [ ] Add `cargo audit` to CI (weekly scheduled job)
- [ ] Create stub `lib.rs` for each crate (empty `pub mod`) so workspace compiles
- [ ] Run `cargo check --workspace` locally and confirm zero errors

## Previous Story Intelligence

N/A — first story in monocle Phase 2. No predecessor stories in this epic.

## Architecture Compliance Rules

From `architecture/SS-deps-pin-manifest.md` v1.1.17:
- EXACT pin for 9 security-sensitive crates: `tokio`, `axum`, `serde_json`, `rand`, `prost`, `russh`, `rmcp`, `reqwest`, `wasmtime`
- Caret pin for all other Phase 1 crates
- MSRV: Rust 1.86 (ratatui 0.30 floor)
- `wasmtime 44` and `russh 0.60` declared in workspace but NOT activated in Phase 1 crate members
- `rmcp 1.6` OMITTED from Phase 1 workspace entirely (Phase 4 scope)

From `architecture/SS-conventions-anti-patterns.md` v1.29.5:
- `cargo clippy --workspace -- -D warnings` is the enforcement gate
- `cargo fmt --all` required; CI fails on format divergence

**Forbidden Dependencies:**
- `monocle-core` MUST NOT depend on `monocle-runtime` (would create a cycle)
- `monocle-runtime` MUST NOT depend on `monocle-tui` (not a Phase 1 crate)
- No crate may depend on `rmcp` in Phase 1 workspace
- `monocle-auth` MUST NOT appear as a workspace member or crate dependency (Decision 3; function is `monocle_runtime::auth::generate_session_token()` inside `monocle-runtime`)

## Library & Framework Requirements

| Crate | Version | Pin Type | Cargo.toml entry |
|-------|---------|----------|-----------------|
| axum | 0.8.9 | EXACT | `axum = "=0.8.9"` |
| tokio | 1.52 | EXACT | `tokio = { version = "=1.52", features = ["full"] }` |
| serde_json | 1.0.149 | EXACT | `serde_json = "=1.0.149"` |
| rand | 0.8.6 | EXACT | `rand = "=0.8.6"` |
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
| temp-env (dev only) | 0.3 | caret | `temp-env = { version = "^0.3", features = ["async_closure"] }` in `monocle-runtime/Cargo.toml` `[dev-dependencies]` |

## File Structure Requirements

Files to create:
- `/Cargo.toml` — workspace manifest with `[workspace]`, `[dependencies]`, `rust-version`
- `/rust-toolchain.toml` — `[toolchain] channel = "1.86"`
- `/.github/workflows/ci.yml` — CI matrix workflow
- `/monocle-core/Cargo.toml` — crate manifest
- `/monocle-core/src/lib.rs` — stub: `pub mod engine; pub mod factory; pub mod abi;`
- `/monocle-runtime/Cargo.toml` — crate manifest
- `/monocle-runtime/src/main.rs` — stub binary entry point
- `/monocle-runtime/src/lib.rs` — stub lib root
- `/monocle-proto/Cargo.toml` — crate manifest with `prost-build` build dependency
- `/monocle-proto/src/lib.rs` — stub
- `/monocle-proto/build.rs` — prost-build stub
