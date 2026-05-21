---
document_type: wave-gate-review
wave: 1
scope: "main@6600585 → develop@cfeb1346"
producer: vsdd-factory:adversary
verdict: PASS_WITH_OBSERVATIONS
timestamp: 2026-05-21T05:00:00Z
project: monocle
---

# Adversarial Review — Wave 1 Gate (PR #1 + #2 + #3)

**Verdict:** PASS_WITH_OBSERVATIONS
**Wave 1 integration verdict:** Composes cleanly. Workspace, xtask, dtu-fidelity, and test-harness fit. Pin manifest hygiene sound. No CRIT findings.
**Confidence:** HIGH
**Develop READY for Wave 2 dispatch:** YES — observations documentation-quality, not blocking.

## Critical Findings: None.

## Important Findings

### F-WAVE1-001 — HIGH — Stale "Red Gate" doc-strings across S-DTU-001 test suite [process-gap]
Files: integration_payload.rs:505-528, integration_auth.rs:72-99,147,366, integration_filters.rs:46,114,257-282, integration_endpoints.rs:42, integration_bc_hooks.rs:56,68,80,106,315,333,557, integration_binary.rs:8,73,114,126, common/mod.rs:216-219. 30+ comments claim handlers contain todo!() and tests "panic at Red Gate." Handlers fully implemented. xtask/dtu_fidelity.rs:6 also claims binary contains todo!() (it doesn't).
Worst case: integration_payload.rs:512-528 `test_BC_HOOK_007_fixture_score_compute_panics_before_implementation` asserts nothing — expects FixtureScore::compute to panic but function is implemented at dtu/payload.rs:149-211. Test now passes silently (false-green for scoring function).
Routing: test-writer + implementer follow-up to sweep stale comments + fix the false-green panics_before_implementation test.

### F-WAVE1-002 — HIGH — dtu-fidelity.yml path filter references nonexistent crates/monocle-ipc/**
File: .github/workflows/dtu-fidelity.yml:7. monocle-ipc crate doesn't exist; only referenced by workflow + .semgrep.yml. Dead config that misleads readers about workspace topology.
Routing: devops-engineer to remove the path or document Phase-1 deferral.

### F-WAVE1-003 — HIGH — STORY-INDEX bookkeeping drift: S-DTU-001 status ready (now CLOSED via 06c94fb + c459b06)
Fixed during wave-gate: story-writer commit 06c94fb (STORY-INDEX v2.4 → v2.5 ready→done) + c459b06 (S-DTU-001 story frontmatter v1.2 → v1.3 ready→done). Wave-gate sibling-sweep verified all 3 status locations (sprint-state, STORY-INDEX, story frontmatter) now consistent.

## Medium Findings

### F-WAVE1-004 — MED — Cron-schedule collision: audit.yml + dtu-fidelity.yml both 0 0 * * 0
Both run weekly Sunday 00:00 UTC. Cache thrash + log aliasing.
Routing: devops-engineer to stagger (e.g., audit 0 0 * * 0, dtu-fidelity 0 6 * * 0) or add concurrency.group.

### F-WAVE1-005 — MED — xtask not in cargo-deny [graph].targets
Targets limit cargo-deny to 3 CI runner triples. xtask is dev-tooling-only. Low likelihood phase 1 risk.
Routing: defer; document in deny.toml that xtask Linux-host-only for ban-evaluation.

### F-WAVE1-006 — MED — crates/monocle-proto/build.rs no-op stub but prost-build is build-dep
Intentional per S-013 deferral. Build cost ~2-3s prost-build compile every cold-cache CI run × 3 runners. Resolved when S-013 wires real .proto files. Awareness only.

## Low Findings

### F-WAVE1-007 — LOW — temp-env in monocle-test-harness without async_closure feature
Cargo.toml:62 declares temp-env = "0.3" without features. monocle-runtime correctly has async_closure feature. Harmless — only sync with_vars used in monocle-test-harness.

### F-WAVE1-008 — LOW — audit-on-pr taiki-e fallback silent path
ci.yml:330-345. If install-action falls back to source compile, no log signal. Low blast radius (slower CI).

## Observations

### O-WAVE1-001 — Cumulative pin/manifest health (positive)
All 9 EXACT-pinned crates resolve correctly in Cargo.lock. bytes 1.11.1 (RUSTSEC-2026-0007 floor satisfied).

### O-WAVE1-002 — Three workflows compose cleanly
ci.yml + audit.yml + dtu-fidelity.yml. No overlapping responsibilities. All actions SHA-pinned.

### O-WAVE1-003 — Audit-table vendor + drift check loop closed
scripts/audit-table.md matches canonical. CI fail-loud on drift.

### O-WAVE1-004 — Workspace ↔ DTU clone composition
xtask depends on monocle-test-harness via path. Uses in-process tower::ServiceExt::oneshot driving.

### O-WAVE1-005 — Forbidden-pattern sweep clean
No println!/eprintln!/std::fs::write/unbounded_channel in crates/**. All atomic writes via tempfile::persist. dtu_server stdout.lock() pre-tracing-init justified at lines 41, 79.

### O-WAVE1-006 — Security NFR spot-check
Auth header alias only to 127.0.0.1; no token logging. signal(0) PID liveness via nix::sys::signal::kill.

## Top 3 Findings (by remediation priority)
1. F-WAVE1-001 — Sweep stale Red Gate comments + fix/remove false-green test. test-writer.
2. F-WAVE1-003 — CLOSED via 06c94fb + c459b06.
3. F-WAVE1-002 — Remove monocle-ipc path from dtu-fidelity.yml or document Phase-1 deferral. devops-engineer.

## Novelty: MED. F-WAVE1-001 + F-WAVE1-003 surfaced only at wave-level review; per-story reviews couldn't see them.
