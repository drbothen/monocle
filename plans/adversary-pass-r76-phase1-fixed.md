---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.10 8feecad + VP v1.10 03a1293 + arch v1.0.16 6bb93e2 + manifest v1.1.10 7d8d0de; F-R75 closure chain applied; D-047 strict pass 1 of 3 (attempt 10 — retry after R76 first attempt stream-watchdog timeout); ALL L-F-R63 Extensions + agent-id-routing-existence codified"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T20:00:00Z
pass_number: 1
attempt: 10
policy: D-047-strict
---

# Adversarial Review Pass R76 (retry) — Phase 1 (D-047 Strict, Pass 1 attempt 10 — FINDINGS)

## Summary

**Verdict:** FINDINGS — 2 HIGH + 1 process-gap observation. **Counter:** RESET to 0/3.

R76 retry after prior stream-watchdog timeout. Two HIGH findings confirmed via fresh independent re-analysis (both hint candidates from prior dispatch VERIFIED as real). Both findings are net-novel vs R73/R74/R75 axes. Both demonstrate partial-fix or §Trace-fabrication patterns that survived multiple convergence rounds.

## 22-BC ↔ 22-VP Audit

22/22 mapping intact. F-R75 closures all verified held.

## Findings

### F-R76-1 [HIGH] — §Trace audit fabrication: triple-false-positive deps-pin sweep claim

**Files:**
- VP §Trace tables at lines 2561, 2946, 3313 (v1.10, v1.9, v1.8) all contain row: `| serde | 1 (caret) | VP-RING-001 §Pre-conditions; VP-PROTO-001b §Pre-conditions | PASS — serde 1 cited verbatim |`
- Reality (a): SS-deps-pin-manifest.md v1.1.10 lines 33-65 pin table has NO bare `serde` row. Only `serde_json 1.0.149` (line 41) and `serde_yaml_ng 0.10` (line 42).
- Reality (b): VP-RING-001 line 937 cites `serde_json 1` (NOT `serde 1`)
- Reality (c): VP-PROTO-001b lines 1540-1544 cites NEITHER serde NOR serde_json
- Reality (d): grep for "serde 1" returns 0 hits in VP body outside the §Trace audit rows themselves

**Significance:** Three sweeps (v1.8/v1.9/v1.10) assert PASS on a row whose Manifest column references a crate that doesn't exist in the manifest AND whose VP-body citation references text that doesn't appear in the cited VPs. This is the false-green audit signal L-F-R63 Extension 3 was authored to prevent — and it survived three rounds of consistency-validator + adversary audits.

**Downstream impact:** When Phase 2/3 implementation reaches `monocle-runtime/Cargo.toml`, "what serde pin does the VP catalog assume" has no canonical answer — §Trace says `serde 1` but manifest doesn't list it and VPs cite `serde_json 1`. `#[derive(Serialize)]` in `HookEventRecord` (per arch lines 501-515) requires bare `serde` declared.

**Routing:**
1. architect: add bare `serde` row to manifest pin table with derive feature; add dep-graph edges `runtime → serde` and `core → serde`
2. formal-verifier: (a) add `serde 1` to VP-RING-001 §Pre-conditions where Serialize is derived; (b) audit VP-PROTO-001b for required pins; (c) re-derive §Trace Extension 3 audit row content to match actual VP-body text

### F-R76-2 [HIGH] — Workspace dep graph missing `runtime → axum` edge (F-R74-3 partial-fix regression)

**Files:**
- SS-deps-pin-manifest.md v1.1.10 lines 162-172 (workspace dep graph for runtime): edges to tokio, tracing, rand, constant_time_eq, core, proto, ipc, async_trait, tempfile, serde_json, directories, nix. NO axum.
- F-R74-3 §Trace v1.1.10 (lines 291-295) explicitly enumerates the 12 outbound edges — axum absent.
- Arch v1.0.16 SS-daemon-lifecycle.md lines 156-176 (Body Size Limit): `Router::new().route(...).layer(DefaultBodyLimit::max(256 * 1024))` — direct axum::Router invocation
- Arch v1.0.16 line 173: `axum::Router::merge` invocation in code sample
- Arch v1.0.16 lines 621-622: `axum::serve(listener, app).with_graceful_shutdown(shutdown_rx)` in Hard Shutdown section
- VP-DAEMON-001 §Pre-conditions line 211: "`axum 0.8` is the project pin (per SS-deps-pin-manifest.md)" — harness at `monocle-runtime/tests/healthz_endpoint.rs` requires monocle-runtime → axum
- Legacy `ipc → axum` edge (manifest line 174) reflects pre-decomposition assumption; monocle-ipc is Phase 4 federation russh tunnel (arch lines 279-289), distinct from Phase 1 HTTP

**Significance:** Without `runtime → axum` in the dep graph, monocle-runtime/Cargo.toml won't declare axum directly. Daemon source calling `axum::serve(...)` won't compile. This is the SECOND completion error in the same F-R74-3 partial-fix series — Extension 1 "exhaustive grep" was applied as "enumerate crates I remembered" rather than "grep all crate::* usage in daemon arch."

**Routing:** architect — (a) add `runtime → axum` to dep graph (manifest v1.1.10 → v1.1.11); (b) re-evaluate `ipc → axum` (likely redirect to runtime); (c) comprehensive grep audit of all crate-prefix usage in daemon arch against graph.

## Process-Gap Observation

**Obs-R76-1 [process-gap]:** L-F-R63-PARTIAL-FIX Extension 1 was applied to F-R74-3 closure but missed `runtime → axum` despite axum being the most-visible daemon dependency (named in 6 VPs, called directly in 3 arch code blocks). Recurrence guard codification recommended as L-F-R63 Extension 7: "Exhaustive grep" step must include automated `grep -E '\b[a-z_]+::' arch.md` against the graph, not just human-enumerated crates.

## Frozen META Catalog Status (D-054)

All 4 entries preserved.

## Novelty Assessment

**Novelty: HIGH.** F-R76-1 introduces a new attack axis (§Trace audit fabrication — false-green PASS verdict in security-grep recurrence guard). F-R76-2 extends F-R74-3 axis with second-iteration partial-fix proof. Both axes not exercised by R73/R74/R75.

## Convergence trajectory

16 attempts: 13→5→1→4→0→2→1→0→0→3→5→3→0→3→2→2. Each fresh-context lens still finds substantive defects. Counter has hit 1/3 (R66/R69/R73) but never 2/3. Asymptotic convergence persists.

## Pass 1 attempt 11 readiness

BLOCKED until F-R76 closure chain:
1. architect: manifest v1.1.10 → v1.1.11 (F-R76-1 add bare serde + F-R76-2 add runtime→axum + redirect ipc→axum)
2. formal-verifier: VP v1.10 → v1.11 (F-R76-1 §Pre-conditions + §Trace audit row reconciliation + arch pin propagation)
3. state-manager: STATE.md update + L-F-R63 Extension 7 codification (exhaustive automated grep discipline)
4. R77 + cons R16
