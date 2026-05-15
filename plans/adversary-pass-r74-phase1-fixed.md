---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.8 bf11194 + VP v1.8 d80749c + arch v1.0.14 e4ce2f0 + manifest v1.1.9 1f53d47; D-047 strict pass 2 of 3"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-14T20:30:00Z
pass_number: 2
policy: D-047-strict
---

# Adversarial Review Pass R74 — Phase 1 (D-047 Strict, Pass 2 — FINDINGS)

## Summary

**Verdict:** FINDINGS — 3 HIGH severity. **Counter:** RESET to 0/3.

R73 was CLEAN (counter advanced to 1/3); R74 fresh-context with new lens rotation (BC-vs-BC cross-consistency, implementation rationale soundness, dep-graph completeness) found 3 new HIGH defects. R73's lenses (async cancellation safety + memory boundedness) re-confirmed CLEAN.

## 22-BC ↔ 22-VP Audit

22/22 mapping verified. F-R72 closures intact.

## Findings

### F-R74-1 [HIGH] — Arch §GET /status JSON schema sketch uses ellipsis placeholder in hook_endpoints array

**File:** SS-daemon-lifecycle.md v1.0.14 line 82
**Defect:** `"hook_endpoints": ["/hooks/pre-tool-use", "/hooks/notification", "..."]` — 3-element array with literal `"..."` as third element. Canonical is 5 strings per BC-DAEMON-002 PC1 + VP-DAEMON-002 mech prop 3 + arch §router construction (lines 158-162).

**Pattern:** Same class as F-R72-1 (closed v1.0.14 for `<ISO8601>` placeholders). F-R72-1 sweep was scoped to timestamps; did not catch the ellipsis pattern in the same JSON schema paragraph.

**Fix:** Replace ellipsis with canonical 5-string enumeration.

**Routing:** architect (SS-daemon-lifecycle.md v1.0.14 → v1.0.15).

### F-R74-2 [HIGH] — BC-ENGINE-001 invariant 3 factually wrong rationale for `#[async_trait]`

**File:** PRD v1.8 line 1020
**Defect:** Claims "async fn in traits is not stable in MSRV (Rust 1.86)" — FALSE. Native async fn in traits was stabilized in Rust 1.75 (Dec 2023). Phase 1 MSRV is 1.86 — far later.

**Actual reasons `#[async_trait]` is needed:**
1. Send propagation: `EngineModule` is bounded `Send + Sync + 'static` but native async fn doesn't auto-bound returned Future as Send. `#[async_trait]` desugars to `Pin<Box<dyn Future + Send + 'async_trait>>` providing explicit Send.
2. Dyn-compatibility: trait must be usable as `Box<dyn EngineModule>` (Phase 3 plugin SDK requires). dyn-AFIT and return_type_notation are still unstable in Rust 1.86.

**Failure path:** Implementer who notices MSRV 1.86 ≥ 1.75 may remove `#[async_trait]` in favor of native async fn — compiles concretely but breaks `dyn EngineModule` Send propagation in Phase 3.

**Fix:** Rewrite invariant 3 with correct technical rationale (Send propagation + dyn-compatibility).

**Routing:** product-owner (PRD v1.8 → v1.9).

### F-R74-3 [HIGH] — Workspace dependency graph missing 4 edges from monocle-runtime

**File:** SS-deps-pin-manifest.md v1.1.9 lines 162-194 (graph for `runtime` crate)
**Defect:** Graph shows 8 edges for `runtime` but missing: `tempfile`, `serde_json`, `directories`, `nix`. All four are used in `monocle-runtime` source per BC-DAEMON-005, BC-AUTH-001, BC-LOCK-001, BC-RING-001 implementation paths.

**Pattern:** F-R71-4b added `nix 0.30` to the manifest table + §Trace narrative naming `monocle-runtime` as declaring crate, but did NOT propagate to the canonical graph. Partial-fix regression S-7.01. The other 3 (`directories`, `tempfile`, `serde_json`) are original-authoring omissions.

**Failure path:** Implementer following the graph adds these to `monocle-config/Cargo.toml` only; `cargo build monocle-runtime` fails because `lock_file_lifecycle.rs` references `tempfile::persist`, etc., that aren't in `monocle-runtime/Cargo.toml`.

**Fix:** Add 4 edges: `runtime → tempfile`, `runtime → serde_json`, `runtime → directories`, `runtime → nix`.

**Routing:** architect (SS-deps-pin-manifest.md v1.1.9 → v1.1.10).

## Frozen META Catalog Status

All 4 D-054 entries preserved. None re-litigated.

## Novelty Assessment

**Novelty: HIGH.** 3 NEW defect classes (BC-vs-BC cross-consistency, factual rationale soundness, dep-graph completeness) not exercised by R62-R73.

## Convergence trajectory

13→5→1→4→0→2→1→0→0→3→5→3→0→3 across 14 attempts. Counter has hit 1/3 three times but never reached 2/3. Each pass-2 attempt has found new lens-rotation defects.

## Pass 2 Verdict and Pass 3 Readiness

**Verdict:** FAIL. Counter RESET.

**Required closure chain:**
1. architect: arch v1.0.14 → v1.0.15 (F-R74-1) + manifest v1.1.9 → v1.1.10 (F-R74-3) — single commit can cover both
2. product-owner: PRD v1.8 → v1.9 (F-R74-2 + arch pin propagation + cons R13's R13-001 fix in VP §Purpose stale SHA via formal-verifier separate dispatch)
3. formal-verifier: VP v1.8 → v1.9 (R13-001 fix + arch + PRD + manifest pin propagations + intra-block sweep)
4. state-manager: STATE.md update
5. R75 + cons R14
