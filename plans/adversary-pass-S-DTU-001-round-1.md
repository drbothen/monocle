---
document_type: adversarial-review
story: S-DTU-001
scope: "worktree story/S-DTU-001 vs develop@184f7d4"
pass: 1
producer: vsdd-factory:adversary
verdict: FAIL
timestamp: 2026-05-21T02:30:00Z
project: monocle
---

# Adversarial Review — S-DTU-001 — Pass 1 (Fresh Context)

**Verdict: FAIL** — must remediate before merge. Three CRIT spec-implementation gaps render AC-004, AC-005, and BC-HOOK-014 unmet, plus a category of tautological tests that invalidates the "all 105 tests GREEN" signal.

## Findings

### CRIT-1: Binary entrypoint is still todo!() — AC-005 unmet [HIGH]
File: crates/monocle-test-harness/src/bin/dtu_server.rs:23-25
`main()` body is `todo!("S-DTU-001 implementation pending; AC-005, AC-006, BC-HOOK-013, BC-HOOK-015")`. AC-005 requires `dtu-claude-code-hooks-v1` binary to run. S-009 depends on this binary as test driver. Commit d52e823 claim contradicted by unimplemented entrypoint.

### CRIT-2: cargo xtask dtu-fidelity oracle never created — AC-004 unmet [HIGH]
No xtask crate in workspace.members. AC-004 + Tasks explicitly require `cargo xtask dtu-fidelity` exits 0. dtu-assessment.md §Tooling L247 specifies `xtask/src/dtu_fidelity.rs`. Missing.

### CRIT-3: .github/workflows/dtu-fidelity.yml never created — AC-004 unmet [HIGH]
Only audit.yml + ci.yml exist. Story File Structure Requirements L208 mandates dtu-fidelity.yml. Missing.

### CRIT-4: Fidelity tests are tautological — AC-004 not actually verified [HIGH]
File: integration_fidelity.rs (23 tests + aggregate). Every test does `let clone_output = fixture.clone(); FixtureScore::compute(&fixture, &clone_output)`. Score trivially 1.0 for x.clone() == x. Clone server never started. POL-11 positive-coverage assertion anti-pattern at test layer. ALL fidelity tests are false-greens.

### CRIT-5: BC-HOOK-014 MONOCLE_RUNTIME_DIR env-var never honored — BC unmet [HIGH]
Grep MONOCLE_RUNTIME_DIR returns 0 source files. Only MONOCLE_HOOK_ENDPOINT_BASE honored (lock_reader.rs:228-233). BC-HOOK-014 §Invariant 3 + §EC-003 mandate the DTU clone read MONOCLE_RUNTIME_DIR. integration_auth.rs:306-343 BC-HOOK-014 tests actually test MONOCLE_HOOK_ENDPOINT_BASE — wrong env var. Test names misleading.

### HIGH-1: unwrap_or_default() swallows serde errors silently — SOUL #4 violation [HIGH]
File: handlers.rs:30. `serde::Serialize::serialize(value, &mut ser).unwrap_or_default();` → silent failure with zero-byte body propagated to spawn_daemon_post. No tracing::warn!. CLAUDE.md "no silent failures."

### HIGH-2: BC-HOOK-007 hook command doesn't reference homedir/MONOCLE_RUNTIME_DIR [MED confidence]
File: server.rs:158-163. Node.js inline hook command hardcodes `hostname:'127.0.0.1',port:7860`. Does not consult os.homedir() or process.env.MONOCLE_RUNTIME_DIR. BC-HOOK-014 §PC-1 says hook JS reads os.homedir() to construct lock file path.

### HIGH-3: No tracing instrumentation anywhere — CLAUDE.md convention violation [MED confidence]
Grep `tracing::|info!|debug!|warn!|error!` returns 0 matches in src/. tracing dep imported in Cargo.toml but never used. Active-fail-open path at handlers.rs:46-48 absorbs network errors without tracing::warn!.

### HIGH-4: hooks-settings.json filesystem mode race [MED confidence]
File: server.rs:222-241. NamedTempFile::new_in created → write_all → set_permissions(0o600) → persist. Window between temp creation and chmod = vulnerable to default temp permissions. Production-grade: `tempfile::Builder::new().mode(0o600)`. NFR-009 at risk.

### MED-1: is_pid_alive shells out via Command::new("kill") on macOS [MED]
File: lock_reader.rs:189-211. Spawns kill -0 via process exec instead of nix::sys::signal::kill. nix already in workspace deps. BC-HOOK-017 "no false negatives" risk if Err branch returns true and kill malformed.

### MED-2: Test-only unsafe std::env::set_var creates test-pollution risk [MED]
File: integration_auth.rs:316-322, 333-335. Rust 2024 marks set_var/remove_var as unsafe (process-wide mutable global). cargo test parallel default. Race: test A sets, test B reads before remove_var. Fix: temp_env crate or serial_test.

### MED-3: large-message-boundary.json + non-permission-dropped.json semantic overlap [LOW]
Both have notification_type=assistant_message. BC-HOOK-034 filter only forwards permission_prompt → large-message-boundary 200 KiB never exercised end-to-end. Should be permission_prompt to test 200 KiB wire boundary.

### MED-4: Single-commit implementation defeats TDD micro-commit discipline [MED]
Commit d52e823 covers 9 files / 800 lines / multiple BCs in one commit. implementer agent prompt mandates one-failing-test → minimum-code → micro-commit pattern. [process-gap].

### LOW-1: State cloning model in endpoints.rs:30 [LOW informational]
Fixture-corpus fidelity tests use tower::ServiceExt and never start a real server → spawn_daemon_post tokio::spawn task may be dropped at test runtime shutdown.

### LOW-1.5: Stale TDD-stub comments in tests [LOW]
test_BC_HOOK_007_endpoint_pre_tool_use_accepts_post (integration_endpoints.rs:38-41) references todo!() red gate even though implementation is green.

## Top 3 Findings (must-fix before merge)
1. CRIT-1 — Binary entrypoint is todo!() (AC-005 unmet)
2. CRIT-3 + CRIT-2 — No xtask crate, no dtu-fidelity.yml workflow (AC-004 unmet)
3. CRIT-4 — All 23 fidelity tests tautological (AC-004 false-green)

## Novelty: HIGH (all findings novel; fresh-context first pass)

## Confidence: HIGH on CRIT-1..5; MEDIUM on HIGH-2..4 + MED-1..3

## Mergeable per Production-Grade Default: NO
Three ACs unmet (AC-004, AC-005, BC-HOOK-014). AC-004 is a CI-as-Code false-green. Single giant commit conflicts with TDD micro-commit discipline.
