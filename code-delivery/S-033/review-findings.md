# S-033 Review Convergence Tracking

**PR:** #40 — feat(S-033): SessionManager::spawn_session
**Branch:** story/S-033-session-manager-spawn
**Merged:** 2026-06-17T19:21:09Z
**Merge SHA:** c7e10f2326bbe5c53388a92b86124844c8cf1257

---

## Convergence Summary

| Cycle | Source | Findings | Blocking | Fixed | Remaining |
|-------|--------|----------|----------|-------|-----------|
| Pre-PR (adversarial) | adversary (passes 1-4) | 30 | 5 | 30 | 0 |
| Pre-PR (security gate) | security-reviewer | 3 (SEC-001/003/005) | 3 | 3 | 0 |
| 1 | security-reviewer (post-PR) | 4 | 0 | 0 | 4 (tracked) |
| 1 | pr-reviewer | 2 | 0 | 0 | 2 (non-blocking nits) |

**Verdict:** APPROVE in cycle 1. Zero blocking findings.

---

## Security Findings

| ID | Severity | CWE | Status |
|----|----------|-----|--------|
| SEC-001 | CRITICAL | CWE-602 | CLOSED (fixed pre-PR: no unsafe pre_exec) |
| SEC-003 | HIGH | CWE-22 | CLOSED (fixed pre-PR: UUID path validation) |
| SEC-005 | HIGH | CWE-59 | CLOSED (fixed pre-PR: UDS remove-before-bind) |
| SEC-006 | MEDIUM | CWE-20/93 | OPEN — tracked for S-045 (ccr_base_url validation before ClaudeCodeModule::spawn_recipe activates full CCR URL path) |
| SEC-007 | LOW | CWE-770 | OPEN — informational (1 MiB allocation cap; revisit before S-047 PTY streaming) |
| SEC-008 | LOW | CWE-367 | OPEN — informational (sidecar TOCTOU; correctness relies on daemon mutex guard) |
| SEC-009 | LOW | CWE-73 | OPEN — informational (hooks_settings_path /tmp fallback; consider cfg-gating for production) |

---

## PR Review Findings

| Finding | Severity | Status |
|---------|----------|--------|
| Test name "exactly_5_methods" is stale (now 6 methods) | NIT | DEFERRED — non-blocking, future cycle |
| Demo binary size (~13 MB GIF/WEBM) | DECISION | SURFACED to orchestrator — binaries are in develop after squash merge |

---

## CI Fix Cycles

| Cycle | Failure | Fix Commit | Status |
|-------|---------|------------|--------|
| 1 | Semgrep: naked fs::write in B-005 test | 987f0dd | FIXED |
| 1 | Preflight: cargo fmt failure (fmt fix) | 538e223 | FIXED |
| 1 | POL-11: 53 stale version-pin citations in factory-artifacts | 579f077 (factory-artifacts) | FIXED |
| 2 | POL-11: 4 stale version-pin citations in mod.rs source comments | db40ffc | FIXED |
| 3 | Build+Test: monocle-session-host binary path in unit test | b8fdd30 | PARTIAL — integration test still failing |
| 4 | Build+Test: session-host binary path in red gate + claude-binary guard for B-002/B-002b | cda4f4a | FIXED — all 10 CI checks GREEN |
