---
document_type: adversarial-review
story: S-DTU-001
round: 3
base_sha: 184f7d4
head_sha: 8a427e6
producer: vsdd-factory:adversary
verdict: PASS_WITH_OBSERVATIONS
timestamp: 2026-05-21T04:00:00Z
project: monocle
---

# Adversarial Review — S-DTU-001 Round 3

**Verdict: PASS_WITH_OBSERVATIONS**

R2 MED closures all verified. No new HIGH/CRIT defects. One MED + one LOW observation.

## R2 Closure Verification

All 3 R2 MEDs CLOSED with file:line evidence:
- MED-1 9 xtask println! → writeln!(stdout.lock()) — xtask/src/dtu_fidelity.rs:486-549
- MED-2 reqwest::Client timeout — dtu_server.rs:137-141 + xtask:332-336 (5s, BC-HOOK-022-aligned)
- MED-3 hardcoded port + "Future:" comment — dtu_server.rs:148-155 MONOCLE_DTU_LISTEN_PORT env override; "Future:" comment grep zero hits

## NEW Findings

### NEW MED — Silent port-parse-failure fallback
File: crates/monocle-test-harness/src/bin/dtu_server.rs:151-154
MONOCLE_DTU_LISTEN_PORT override silently falls back to 7860 when env var set to non-numeric value. CI operator who mistypes port receives no error; may unknowingly collide with another DTU instance.
Production-grade replacement: parse::<u16>() should Result::map_err → context → bail! when env var is set but unparseable. Set-and-empty/unset legitimately fall through; set-and-garbage should fail loudly.
Severity: MEDIUM — not regression of R2 MED-3 (original hardcoded; this improved-but-incomplete). Reasonable to defer; flagging because production-grade default applies.

### NEW LOW — args.first() ignores extra args
File: dtu_server.rs:66-89
`dtu-claude-code-hooks-v1 --help --frobnicate` exits 0 silently. Cosmetic; revisit when clap added.

## Confirmed Clean

- writeln! lock-contention no risk; stdout.lock() held across render loop
- 5s timeout regression risk none (mock daemon 500ms timeout dominates in tests)
- Cargo.lock churn limited
- dtu-fidelity.yml schedule + workflow_dispatch present
- AC-001..AC-007 still mapped (5 endpoints, alias auth header, monocle-canonical payloads, 25-fixture corpus, Rust binary, env-var overrides, cargo xtask dtu-fidelity oracle)
- Tests meaningful (integration_binary --help; integration_fidelity drive_and_capture pattern)
- Micro-commit hygiene: 4 commits / 3 fixes / 1 fmt fixup acceptable
- No "MVP/for now/good enough" in production code (1 hit in test comment Red-Gate documentation acceptable)
- tracing-subscriber duplication (R2 process-gap) still present; not blocking

## Top 3 Findings

1. MED — MONOCLE_DTU_LISTEN_PORT silent fallback on parse failure
2. LOW — args.first() ignores extra args
3. LOW carry-over — xtask auth_token "placeholder" comment (R2 deferred maintenance sweep)

## Process-gap: none new. R2 tracing-subscriber duplication observation still valid; not blocking.

## Novelty: LOW. R3 produced 1 substantive MED (env-var silent fallback) + 1 micro-LOW (multi-arg). Both downstream consequences of R2 env-config surface expansion. No fundamental gaps. Implementation converged on canonical spec.

## Confidence: HIGH on R2 closure verification + new MED finding; HIGH on story-spec alignment; MEDIUM on test meaningfulness (sampled).

## READY for demos + push + pr-manager?
YES — conditional. R2 MED closures clean. New MED non-blocking under "deferred maintenance" lens (operator-misconfig-only, not correctness regression). Recommend either: (a) push + 5-min implementer follow-up fix; (b) push + add to post-merge sibling-sweep.
