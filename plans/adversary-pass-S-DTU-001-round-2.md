---
document_type: adversarial-review
story: S-DTU-001
round: 2
base_sha: 184f7d4
worktree_sha: 49f2b40
producer: vsdd-factory:adversary
verdict: PASS_WITH_OBSERVATIONS
timestamp: 2026-05-21T03:30:00Z
project: monocle
---

# Adversarial Review — S-DTU-001 Round 2

**Verdict: PASS_WITH_OBSERVATIONS**

R1 critical defects genuinely closed. Implementation production-grade across major axes. No CRIT-new findings. Several MEDIUM concerns identified, none blocking demos+push+pr-manager.

## R1 Closure Verification

All 10 R1 findings CLOSED with file:line evidence:
- CRIT-1 binary main() — dtu_server.rs:60-165 full async main implemented
- CRIT-2 xtask crate — exists; workspace member; clap derive CLI
- CRIT-3 dtu-fidelity.yml — exists with SHA-pinned actions
- CRIT-4 tautological fidelity tests — integration_fidelity.rs drives clone router + mock daemon
- CRIT-5 MONOCLE_RUNTIME_DIR — lock_reader.rs:247-263 + dtu_server.rs:116-117
- HIGH-1 silent failures — no unwrap_or_default in src/
- HIGH-2 hook env vars — server.rs:165-170 reads process.env.MONOCLE_DTU_PORT
- HIGH-3 tracing — dtu_server.rs:108-113 EnvFilter from RUST_LOG; structured fields
- HIGH-4 file mode race — tempfile::Builder::new().permissions(0o600)
- MED-1 kill shell-out — nix::sys::signal::kill with proper ESRCH handling

## NEW Findings

### MED-1 — 9 println! in xtask/src/dtu_fidelity.rs
Lines 490-524. clippy doesn't catch macros so it passes -D warnings. CLAUDE.md says "No println! in production code". xtask has tracing-subscriber initialized. Fix: tracing::info! OR writeln! to stdout with explicit tooling exemption comment.

### MED-2 — reqwest::Client lacks timeout
dtu_server.rs:137 + xtask/src/dtu_fidelity.rs:332 use Client::new() with default settings. spawn_daemon_post fires tokio::spawn → if daemon hangs, task leaked. Fix: Client::builder().timeout(Duration::from_secs(5)).build().

### MED-3 — hardcoded listen port + "Future:" comment violation
dtu_server.rs:144 hardcodes 127.0.0.1:7860 with comment "Future: accept MONOCLE_DTU_LISTEN_PORT env override (AC-006 extensibility)." Production-Grade Default Rule 1 forbids "Future:" deferred-work language. Multi-binary scenarios cannot work without env override. Node.js hook template already reads MONOCLE_DTU_PORT.

### LOW-1 — xtask:329 auth_token "placeholder" rationalization comment
Cosmetic fix; rewrite to "Auth token value irrelevant — mock daemon accepts all requests."

### LOW-2 — dtu-fidelity.yml no push trigger
Only pull_request + schedule + workflow_dispatch. Weekly cron catches regressions. Not blocking.

### LOW-3 — duplicate workspace_structure.rs (cross-story scope)
monocle-runtime + monocle-test-harness both have one. Cross-story; defer to wave-gate.

## Top 3 NEW Findings
1. MED-3 — hardcoded listen port + "Future:" comment (Production-Grade Rule 1)
2. MED-2 — reqwest::Client no timeout
3. MED-1 — xtask println! (CLAUDE.md convention)

## Observations
- Implementer commits (fb6533c + e18bbc8 + 40d7b4f + 49f2b40) = 4 logical groups; acceptable batching
- 135 tests pass locally; spot-check confirms non-tautological
- CRIT-4 closure validated: handlers forward body.to_vec() raw; notification handler type-parses only for routing filter
- temp-env dev-dep correctly scoped (not in workspace.dependencies)
- tracing-subscriber duplicated in monocle-test-harness + xtask (consolidation candidate; not blocking)

## Novelty: MEDIUM. R1 critical findings closed; new findings are refinements at production-grade boundary, not architectural defects.

## Confidence: HIGH on closure verifications + MED-2/MED-3. MEDIUM on MED-1.

## Ready for demos + push + pr-manager?
YES, with caveat: fix MED-3 before push (1-line; adversary will re-flag). MED-1/MED-2 are reasonable post-merge follow-ups but Production-Grade Default suggests fixing all 3.
