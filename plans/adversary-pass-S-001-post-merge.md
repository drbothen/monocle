---
document_type: adversarial-review
story: S-001
scope: "develop @ a6f119c (16 files, +2272 lines)"
producer: vsdd-factory:adversary
timestamp: 2026-05-20T20:30:00Z
input-hash: "[live-state]"
verdict: FAIL
---

# Adversarial Review — S-001 Post-Merge (develop @ a6f119c)

**Frontmatter intent:**
- document_type: adversarial-review
- story: S-001
- scope: develop @ a6f119c (16 files, +2272 lines)
- producer: vsdd-factory:adversary
- timestamp: 2026-05-20T20:30:00Z

## Verdict: FAIL

Three convention/governance gaps + one architecturally-mandated tooling omission. AC-001..AC-007 all pass syntactically; failure is on Production-Grade-Default rubric and SS-conventions-anti-patterns v1.29.5 §CI Wiring compliance.

## Top 3 Findings (most important)

### F-001 CRIT — cargo-deny + semgrep + deny.toml absent; SS-conventions §CI Wiring step list violated [HIGH confidence]
- Evidence: `SS-conventions-anti-patterns.md` §CI Wiring (lines 525–532) mandates a 6-step CI gate ordering: fmt → clippy → semgrep → test → cargo deny check licenses bans advisories sources → cargo audit. The deny.toml workspace file is fully specced (lines 538–591) with explicit `[bans]` entries (openssl, openssl-sys, tokio < 1.52, russh < 0.60) that act as floor guards for the EXACT-pinned versions. The merged `.github/workflows/ci.yml` has ONLY: fmt, clippy, build, test, cargo audit. No semgrep, no cargo-deny, no `deny.toml` at workspace root.
- Why it matters: tokio < 1.52 ban is the floor guard against RUSTSEC-2025-0023/2023-0005. openssl ban is the only mechanism preventing accidental OpenSSL linkage. Without cargo-deny, all 4 supply-chain hygiene mechanisms are inert. S-001 story spec is silent on cargo-deny/semgrep but SS-conventions §CI Wiring is the architectural source-of-truth.
- Routing: devops-engineer to add deny.toml, .semgrep.yml, CI steps.

### F-002 HIGH — Dependabot has no `ignore:` block for the 9 EXACT-pinned crates; policy violated at automation level [HIGH confidence]
- Evidence: dependabot.yml lines 21–42 declare Cargo updates with update-types [minor, patch] grouped under caret-pinned-libs — but no ignore: rule excludes the 9 EXACT-pinned crates. SS-deps-pin-manifest §Security Advisory Response Policy mandates auto-merge BLOCKED for exact-pinned crates with security-reviewer dispatch. The dependabot.yml COMMENT lines 10–14 claims "Dependabot will NOT propose updates because the manifest specifies an exact version" — factually incorrect. Dependabot DOES propose updates for =x.y.z pins.
- Routing: devops-engineer to add ignore: block + fix comment.

### F-003 HIGH — GitHub Action refs not pinned to SHA per SS-conventions §R-001 [HIGH confidence]
- Evidence: ci.yml + audit.yml use floating tag refs (@v4, @v2, @stable). SS-conventions-anti-patterns.md §R-001 line 695 mandates: "devops-engineer MUST resolve action refs to full commit SHAs at workflow creation time." Floating tag refs are mutable supply-chain attack vectors.
- Routing: devops-engineer to pin all action refs to full SHAs.

## Other Findings

### F-004 HIGH [lock-defect] — Cargo.lock resolved bytes = 1.11.1 not 1.10.x [MED confidence]
Cargo.toml declares `bytes = "1.10"` (caret = ^1.10, allowing 1.10..2.0). Cargo.lock resolved 1.11.1. SS-deps-pin-manifest says "bytes = "1.10" is the patched line resolving RUSTSEC-2026-0007" — verification was performed on 2026-05-12 against the 1.10 line. Routing: security-reviewer or research-agent to verify RUSTSEC-2026-0007 against 1.11.1; if clean, document; if not, downgrade pin to `~1.10`.

### F-005 MED — Workspace-declared deps NOT in Cargo.lock (no member crate inherits them) [HIGH confidence]
clap/reqwest/wasmtime/russh/interprocess/notify/serde_yaml_ng workspace-declared but not in lock. cargo audit will NOT scan them. The "exact-pinned + ban guard" mechanism (F-001) is the intended defense; compounds. Routing: architect to confirm intent.

### F-006 MED — audit.yml weekly job vs ci.yml audit-on-pr use divergent cargo-audit install paths [MED confidence]
audit.yml uses cargo install cargo-audit --locked; ci.yml audit-on-pr uses taiki-e/install-action@v2 prebuilt. AC-007 verbatim mandates source-install path. Divergent install paths can yield divergent cargo-audit versions. 7 days of regression window. Routing: architect to adjudicate AC-007 intent; then devops-engineer.

### F-007 MED — workspace_structure.rs tests tautological for AC-001 [HIGH confidence]
ac_005_workspace_declares_exactly_three_phase1_members only asserts "contains" — does NOT assert "and nothing else". Future 4th member would pass this test. AC-001 also mentions clippy but test doesn't assert clippy. Routing: test-writer to harden tests.

### F-008 LOW — concurrency.cancel-in-progress can cancel release-tag CI run
Routing: devops-engineer.

### F-009 LOW — Cargo.toml profile.release strip = "symbols" removes panic backtrace symbols on CI
Routing: architect to confirm intent.

## Observations

[process-gap] S-001 story spec did NOT reference SS-conventions §CI Wiring as a §Library & Framework Requirement input. When a story creates .github/workflows/, the story must trace to SS-conventions-anti-patterns.md §CI Wiring as an input.

[process-gap] Dependabot.yml has a factually-incorrect code comment about exact-pin behavior. devops-engineer prompt should include a Dependabot-exact-pin-behavior verification step.

Cargo.lock semantic correctness [lock-defect-clean for 8 pins]: tokio 1.52.0, axum 0.8.9, serde_json 1.0.149, rand 0.8.6, prost 0.14.1, prost-build 0.14.1, syn 2.0.117, temp-env 0.3.6 all match. bytes 1.11.1 flagged in F-004.

No Co-Authored-By: Claude / robot emoji in diff. No --no-verify evidence.

## Pin-Manifest Compliance Summary
- 9 EXACT-pinned security-sensitive crates declared and resolved correctly.
- Caret pins per L33-74 declared correctly.
- rmcp correctly OMITTED.
- [workspace.dependencies] pattern correctly used.
- bytes = "1.10" declared; resolved to 1.11.1 (see F-004).

## Confidence: HIGH

Full diff reviewed against SS-deps-pin-manifest v1.1.18 and SS-conventions-anti-patterns v1.29.5. Cargo.lock semantic checks performed for the 9 EXACT pins + bytes. Workflow YAMLs read in full. Crate manifests cross-checked against story AC-005/AC-006.
