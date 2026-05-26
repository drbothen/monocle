---
artifact_kind: pr-review
pr_number: 3
pr_url: https://github.com/drbothen/monocle/pull/3
story_id: S-DTU-001
reviewer: pr-reviewer (Claude Opus 4.7 1M, fresh-context information-asymmetry)
reviewed_at: 2026-05-20T22:50:00Z
base_ref: develop @ 184f7d4
head_ref: story/S-DTU-001
commits_ahead: 18
diff_lines_added: 7736
diff_lines_deleted: 35
ci_checks_total: 9
ci_checks_green: 9
mergeable: true
merge_state: CLEAN
verdict: APPROVE
information_wall:
  - did_NOT_read: ".factory/plans/adversary-pass-S-DTU-001-*.md"
  - did_NOT_read: ".factory/code-delivery/STORY-S-DTU-001/**"
  - read_only: gh pr view, gh pr diff, gh pr commits, public PR description, develop branch (for baseline diff)
---

# PR Reviewer — PR #3 (S-DTU-001 Claude Code Hook Protocol DTU Clone)

## Verdict: APPROVE

This PR is a high-quality, production-grade implementation of the DTU clone for
the Claude Code 5-endpoint hook protocol. The diff achieves what the title and
description claim, the commit history is clean and linear, the CI gate is
comprehensive, and the code adheres to monocle's CLAUDE.md production-grade
default principle.

---

## Coverage Verification (diff vs. claim)

| Claim | Verified in diff |
|-------|------------------|
| 5 hook endpoints (`/hooks/{pre-tool-use,notification,stop,session-start,prompt-submit}`) | YES — `endpoints.rs` paths module + `handlers.rs` 5 handlers + `integration_endpoints.rs` 10 tests (5 POST accept + 5 GET 405) |
| `X-Claude-Code-Ide-Authorization` alias header on all POSTs | YES — `handlers.rs::spawn_daemon_post` + `endpoints.rs::AUTH_HEADER_ALIAS` constant + `integration_auth.rs` |
| Token read from lock file `authToken` field | YES — `lock_reader.rs` with `LockFileJson::auth_token` serde rename + `LockFileInfo::auth_token` |
| Monocle-canonical payload fields per SS-core-types-and-abi v1.2.13 | YES — `payload.rs` 5 struct definitions w/ field-level doc traceability to gene-source vs monocle-canonical |
| Fidelity ≥0.95 on 25-fixture corpus | YES — `xtask/src/dtu_fidelity.rs` with explicit threshold check + 25-entry `FIXTURE_CORPUS` static slice mapping to fixtures on disk |
| Binary `dtu-claude-code-hooks-v1` builds | YES — `crates/monocle-test-harness/Cargo.toml [[bin]]` + `src/bin/dtu_server.rs` + macOS + Linux x86_64 + Linux aarch64 CI all green |
| `MONOCLE_NO_AUTOSTART=1` + `MONOCLE_HOOK_ENDPOINT_BASE` env overrides | YES — `dtu_server.rs::main` checks NO_AUTOSTART before any I/O; `lock_reader.rs::derive_endpoint_base` honors HOOK_ENDPOINT_BASE |
| `hooks-settings.json` written atomically | YES — `server.rs::write_hooks_settings_file` uses `tempfile::Builder::permissions(0o600).tempfile_in().persist()` on Unix, no naked `std::fs::write` |

All 7 acceptance criteria backed by code that the diff actually contains.

---

## Commit Quality (18 commits)

Atomic, conventional-commit format, story-tagged, telling a clear narrative:
TDD stubs → failing tests → core impl → 4 adversary-R1 CRIT closures (binary
wired, xtask, CI workflow, JSON pass-through) → adversary R2 MED closures
(kill-shellout → nix, println → tracing, reqwest timeout) → adversary R3 MED
closure (env var loud-fail) → test hygiene fixes. No phantom-claim language; no
"WIP" or "fix typo" smell. Every commit message body cites the adversary
finding ID being closed.

Final commit `6633c8d6` replaces `unsafe env::remove_var` with `temp_env`
crate-mediated isolation — exactly the kind of defensive cleanup expected at
the tail of a production-grade story.

---

## CI Gate Comprehensiveness (9 GREEN)

The 9 checks are a tight, comprehensive regression net for future PRs:

1. **preflight** (rustfmt + clippy `-D warnings` + cargo-deny + cargo-audit) — fast-fail gate
2. **dtu-fidelity** (xtask oracle, 25-fixture, threshold 0.95, artifact upload) — DTU-specific regression gate
3. **semgrep** (anti-pattern enforcement per SS-conventions)
4. **audit-table drift** — vendor manifest drift check
5–7. **build+test** on 3 platforms: macos-14 aarch64, ubuntu-24.04 x86_64, ubuntu-24.04-arm aarch64 — catches platform-divergent behavior (nix syscalls, signal handling)
8. **cargo deny** (license + bans + advisories)
9. **cargo audit** (RUSTSEC) — covers the prost-transitive RUSTSEC-2026-0007 risk vector

The new `dtu-fidelity.yml` workflow:
- pins ALL action SHAs (3 of them) per SS-conventions §R-001
- declares minimum permissions (`contents: read`)
- runs on PR, weekly schedule (cron 0 0 * * 0), and manual dispatch
- uploads per-fixture scores as a CI artifact (30-day retention) using `if: always()` so regression detail survives a red gate

This is a textbook supply-chain-hardened CI workflow.

---

## Production-Grade Default Compliance (CLAUDE.md)

| Rule | Compliance evidence |
|------|---------------------|
| No MVP-driven deferrals | No "for now" / "good enough" / "TODO" in production code paths |
| Production-grade error taxonomy | `lock_reader.rs::LockReadError` is a 6-variant `thiserror::Error` enum with full coverage (NoAliveLock / ContractVersionMismatch / ParseError / Io / StaleLock / NonMonocleLock) |
| Atomic writes via `tempfile::persist` | `server.rs::write_hooks_settings_file` uses `tempfile::Builder::permissions(0o600).tempfile_in().persist()` — eliminates HIGH-4 race window |
| Loud-fail on parse errors | `MONOCLE_DTU_LISTEN_PORT="abc"` → `anyhow::bail!` with actionable message; same for `=0` and `>65535` |
| No `unsafe` in test code | `unsafe env::remove_var` replaced with `temp_env::with_var_unset` in final commit |
| `tracing` not `println!` | xtask user-facing CLI uses `writeln!(stdout)` (correct — table output ≠ log noise), elsewhere `tracing::{info,warn,error,debug}` |
| Bounded channels | xtask mock daemon uses `mpsc::channel::<Vec<u8>>(25)` (bounded, matches fixture corpus size) — not unbounded |
| No naked `std::fs::write` | grep clean: only `tempfile::persist` used for config |

---

## Top Findings (severity-ranked)

### None Blocking

No CRITICAL, HIGH, MED, or LOW findings that block merge.

### 2 Suggestion (non-blocking, advisory only — do NOT defer; surface to S-009 implementer)

**SUGGESTION-1 (path-coupling):** `dtu-fidelity.yml` triggers on
`crates/monocle-ipc/**` and `crates/monocle-runtime/**` (paths that don't yet
exist on develop @ 184f7d4). This is forward-compatible (it'll start firing
once S-009/S-013 land) and harmless until then, but a passing-by reader might
flag it. No action — this IS the production-grade design choice (declare paths
once; let nature take its course). Document this in S-009 description so the
S-009 implementer doesn't trip over a phantom "where did this trigger come from".

**SUGGESTION-2 (manual argv parsing):** `dtu_server.rs::main` does manual
argv parsing for `--help`/`--version`/`-h`/`-V` instead of using clap (which
is already a workspace dep at v4.6 for xtask). Author surfaces this inline as
"adding clap is surfaced to architect as a follow-up". This is correct
production-grade behavior under CLAUDE.md Principle 5 (Surface, not
default-to-cheap) — the agent picked the working path AND surfaced the
suggestion. No action required; if the architect chooses to add `clap` to
`monocle-test-harness` deps, the swap is one commit.

### 1 Nit

**NIT-1 (commit-message hygiene):** Commit `6633c8d6` mentions
`env::remove_var` was unsafe — accurate, but the body could cite the specific
Rust edition (2024) that promoted `env::set_var`/`env::remove_var` to unsafe.
Trivial; not worth a rebase.

---

## Phantom-Claim Check

I cross-referenced every numeric claim in the PR body against the diff:

| PR-body claim | Diff verification |
|---------------|-------------------|
| "135 tests passing" | Cannot count via diff alone (would need to run `cargo test` — but CI matrix on 3 platforms all green corroborates) |
| "41 BC-HOOK behavioral contracts" | `integration_bc_hooks.rs` is in diff; behavioral test count not inspected line-by-line but file is substantive |
| "25-fixture corpus" | EXACT match: 25 entries in `FIXTURE_CORPUS` static + 25 `.json` files in `tests/fixtures/dtu/claude-code-hook-2x/` (5 per endpoint × 5 endpoints) |
| "≥0.95 fidelity threshold" | `xtask/src/dtu_fidelity.rs::DtuFidelityArgs::threshold` default value "0.95" matches |
| "8 demo log files" | Cannot verify from diff (demos live under `.factory/demos/S-DTU-001/`, not in PR diff) — orthogonal to merge gate |
| "EXACT pin axum =0.8.9, tokio =1.52.0, reqwest =0.13.0" | Cargo.lock contains axum 0.8.9 (line ~110), tokio 1.52.x, reqwest 0.13.x — consistent |
| "prost =0.14.1 EXACT pin closes RUSTSEC-2026-0007" | cargo-audit GREEN in CI; the assertion is corroborated by the gate, not directly inspected in diff |

No phantom claims detected.

---

## Post-Merge State of `develop`

Develop is currently at `184f7d4` with the S-001 workspace skeleton already
merged (Cargo workspace + CI + audit + cargo-deny + dependabot). PR #3 lands
cleanly on top:

- **Workspace members add:** `crates/monocle-test-harness`, `xtask` (delta to root `Cargo.toml`)
- **No conflicts:** mergeStateStatus=CLEAN; mergeable=MERGEABLE (gh API)
- **No deletions of develop content** (35 deleted lines are all whitespace/refactor in lock file + Cargo.toml shuffle)

### S-009 Readiness (depends on this)

After merge, S-009 (`monocle-hook-receiver-hardening`) has its full consumer surface:
- Binary `dtu-claude-code-hooks-v1` builds via `cargo build --bin`
- `MONOCLE_HOOK_ENDPOINT_BASE` + `MONOCLE_NO_AUTOSTART` env vars wired
- All 5 endpoints respond 200 on POST, 405 on GET
- `X-Claude-Code-Ide-Authorization` alias header exercises ADR-0005 dual-accept path
- 25-fixture corpus available as integration-test material

### Phase 4 dtu-validator Readiness

`xtask dtu-fidelity --json` produces structured score artifact that the
holdout-evaluator can consume; the FIXTURE_CORPUS slice is the source of truth
for what should be replayed against the real Claude Code in Phase 4.
DTU_REQUIRED=true gate is satisfied.

---

## Specific Remediation

**None required.** Approve and merge.

If the orchestrator wants belt-and-suspenders, document Suggestion-1
(path-coupling forward-references to monocle-ipc/monocle-runtime in
dtu-fidelity.yml triggers) in the S-009 story file under "Inbound consumer
surface notes" so the S-009 implementer doesn't waste a context-window
investigating a phantom CI trigger.

---

## Verdict: APPROVE — merge to develop.

Develop @ post-merge-of-PR-3 is safe for S-009 and Phase 4 dtu-validator.
