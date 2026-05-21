---
artifact_type: adversarial-review
pr: "2"
round: 1
verdict: PASS_WITH_OBSERVATIONS
findings_crit: 0
findings_high: 0
findings_med: 2
findings_low: 3
story: S-001
phase: phase-3
date: 2026-05-20
author: vsdd-factory:adversary
---

# Adversarial Review — PR #2 (S-001 post-merge fix) Round 1

**Verdict: PASS_WITH_OBSERVATIONS**

PR is mergeable. CI is green, fixture corpus is correctly wired, bytes pin propagated cleanly. Two MEDIUM findings (spec/impl divergence on cargo-deny invocation; cargo-audit ordering inefficiency) and three LOW observations. No CRIT or HIGH findings.

## Findings

### MED-1 — cargo-deny action invocation diverges from canonical spec (spec drift)
File: `.worktrees/S-001-fix/.github/workflows/ci.yml:217-218`
Evidence: PR uses `command: check all` (no `arguments:` key). SS-conventions-anti-patterns.md v1.30.0 L637-644 mandates `command: check` + `arguments: --all-features --workspace licenses bans advisories sources`. The §Trace v1.30.0 ratifies the schema changes (A1-A4) but did NOT ratify dropping `--all-features --workspace`. Without `--all-features`, deny ignores feature-gated transitive deps (e.g., axum-tracing optional features); without `--workspace`, deny analyzes only the root package by default.
Risk: Banned-crate detection may miss crates pulled in by `--features` paths. Defeats part of the threat model that motivated tokio<1.52 / russh<0.60 / openssl bans.
Routing: devops-engineer to align ci.yml step to spec OR architect to ratify spec form. Production-grade default: route to architect for explicit ratification, then devops updates whichever side wins.
Confidence: HIGH

### MED-2 — cargo-audit runs AFTER cargo-deny: duplicate RUSTSEC scan + sequential dependency
File: `.worktrees/S-001-fix/.github/workflows/ci.yml:200-260`
Evidence: cargo-deny job (L203) checks advisories via the RUSTSEC db; audit-on-pr job (L223) reruns the same RUSTSEC scan after it. Both gate the PR, both use `--deny warnings`. Spec at L532 says cargo-audit "run weekly scheduled" — implying weekly, not per-PR. Per-PR audit is mostly redundant with deny.
Risk: Wasted minutes; sequential dependency makes merge-time longer. Not a correctness bug but a process gap.
Routing: architect to clarify spec on per-PR vs weekly audit (possible answer: belt-and-suspenders for advisory-db freshness).
Confidence: MEDIUM (pending intent verification)

### LOW-1 — Path-include regex anchoring risk in semgrep paths.include
File: `.worktrees/S-001-fix/.semgrep.yml:39-43` (env-mutation rule)
Evidence: `paths.include: - "**/*tests*.rs"` will match production files like `monocle-runtime/src/integration_tests_module.rs` if such a file ever lands as a non-test file with the substring "tests". Today no such file exists; future drift risk.
Routing: devops-engineer if architect deems narrowing desirable.
Confidence: LOW

### LOW-2 — Semgrep production scan only includes `crates/**/*.rs`, excluding root-level test files
File: `.worktrees/S-001-fix/.github/workflows/ci.yml:131-134` (Step 2 production scan)
Evidence: Step 2 scans `--include="crates/**/*.rs"` and `crates/` path. Future `xtask/` or workspace-root tests/ would not be covered.
Routing: devops-engineer if scope expands.
Confidence: LOW

### LOW-3 — unbounded_channel fixture doesn't cover turbofish form (acknowledged in fixture comment)
File: `.worktrees/S-001-fix/semgrep-fixtures/unbounded_channel.rs:7-11`
Evidence: Comment correctly documents that `tokio::sync::mpsc::unbounded_channel::<T>()` (turbofish) won't be matched by current pattern. Clippy disallowed_methods covers the gap as defense-in-depth, but semgrep rule alone has documented blind spot.
Risk: LOW because clippy covers the gap.
Routing: devops-engineer for completeness — add second pattern-either arm + fixture.
Confidence: MEDIUM (gap real; impact mitigated)

## Top 3 Findings

1. MED-1 — cargo-deny invocation diverges from spec (`command: check all` vs spec form). Bans may not cover feature-gated transitives.
2. MED-2 — Per-PR cargo-audit + cargo-deny advisories duplicates RUSTSEC scan; architect should ratify intent.
3. LOW-3 — Turbofish form `unbounded_channel::<T>()` bypasses semgrep rule; covered by clippy defense-in-depth.

## Confirmed Clean
- Cargo.toml bytes pin updated to `"1.11"`; Cargo.lock resolves bytes 1.11.1 — sync confirmed.
- workspace_structure.rs L218 test asserts `bytes = "1.11"` exactly.
- temp-env relocation: only in monocle-runtime [dev-dependencies], not in root [workspace.dependencies].
- Dependabot ignore: 9/9 EXACT-pinned crates listed; all three semver-update-types covered.
- Action SHAs: all 5 distinct actions pinned to 40-char SHAs.
- deny.toml: bans openssl/openssl-sys/tokio<1.52/russh<0.60 match spec byte-for-byte. wildcards/getrandom/wit-bindgen skips match A2/A3/A4 ratifications.
- All 5 fixture files exist; expected counts align with Step 1 assertion list.
- Semgrep ordering: fmt → clippy → semgrep → test → cargo-deny → audit matches §CI Wiring.
- 7 anti-pattern mechanisms: 5 in semgrep (covered); 2 in clippy disallowed_methods (workspace Cargo.toml wires `disallowed_methods = "deny"`).
- No `Co-Authored-By: Claude` or robot emoji in commits.

## Novelty: MEDIUM. MED-1 is substantive (cross-references workflow against spec exemplar YAML).

## Confidence: HIGH

Evidence-anchored at file:line for each finding; canonical spec versions cross-checked at SS-conventions v1.30.0 + SS-deps-pin-manifest v1.1.19.
