---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.7 3024bd3 + VP v1.7 296b044 + arch v1.0.13 1f53d47 + manifest v1.1.9 1f53d47; F-R71 closure chain applied; D-047 strict pass 1 of 3 (attempt 7); L-F-R63 Extensions 1+2+3+3-Enforcement codified"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T16:00:00Z
pass_number: 1
policy: D-047-strict
---

# Adversarial Review Pass R72 — Phase 1 (D-047 Strict, Pass 1 attempt 7 — FINDINGS)

## Summary

**Verdict:** FINDINGS (1 HIGH + 1 MEDIUM + 1 process-gap)
**Counter:** RESET to 0/3.

## 22-BC ↔ 22-VP Audit + F-R71 closure verification

ALL F-R71 closures VERIFIED HELD:
- F-R71-1: VP cites directories 6 (not 5) — VERIFIED
- F-R71-2: arch test name `test_BC_DAEMON_004_exit_codes_posix_distinct` — VERIFIED at 3 arch sites + PRD + VP
- F-R71-3: NFR-008 "among the primary target platforms" phrasing — VERIFIED at arch + PRD
- F-R71-4a: tower transitive (not "per manifest") — VERIFIED at VP-DAEMON-004
- F-R71-4b: nix 0.30 sole binding + manifest has entry — VERIFIED at VP + manifest
- F-R71-5: VP placeholder `<N>` (not `<int>`) — VERIFIED

25-crate deps-pin sweep against manifest v1.1.9: ALL normative-current matches PASS.

22 BCs / 22 VPs / 14 error codes / 59 edge cases / 23 test names — all unchanged.

## Findings

### F-R72-1 [HIGH] — Arch JSON schema sketches use generic `<ISO8601>`; F-R70-2 BC tightening (millisecond mandatory) didn't propagate to arch SoT

**Files:**
- arch line 586 §Drain step 5: `"shutdown_utc": "<ISO8601>"` (generic)
- arch lines 86-90 §Health and Status Endpoints `/status`: `<ISO8601 or null>` for `last_hook_ts`
- PRD line 408 BC-DAEMON-006 invariant 1: tightened to `YYYY-MM-DDTHH:MM:SS.sssZ` mandatory millisecond
- VP-DAEMON-006 lines 780, 842: enforces `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` regex

**Problem:** F-R70-2 tightened the BC; F-R70 closure burst propagated to PRD body but did NOT update arch JSON schema sketches. Arch is the §BC Summary footer-declared "source-of-truth for invariants" — but the schema sketch is loose. Implementer following arch alone would write seconds-only timestamps; VP test fails. Partial-fix regression of F-R70-2 to L-F-R63 Extension 1 propagation discipline.

**Required fix:**
- arch line 586: `<ISO8601>` → `<YYYY-MM-DDTHH:MM:SS.sssZ>` + cross-reference "VP-DAEMON-006 regex enforced"
- arch lines 86-90: same tightening for `last_hook_ts` per EC-044 precedent
- arch line 427 (§Start Sequence step 6 `startTimeUtc`): assess cross-field consistency — tighten or document seconds-only allowance explicitly

**Routing:** architect (arch v1.0.13 → v1.0.14).

### F-R72-2 [MEDIUM] — NFR-001/002/003 latency targets have ZERO Phase 1 VP coverage; not enumerated in §Open Verification Gaps

**Files:**
- PRD lines 1203-1205: NFR-001 (≤300ms hook latency), NFR-002 (≤2000ms Notification), NFR-003 (≤100ms permission overlay)
- VP §VP Catalog Overview: 22 VPs; no LATENCY VP
- VP §Open Verification Gaps §G-1..§G-5: no NFR-001/002/003 gap entry
- VP line 99 §Scope "Out of scope": "Performance-budget VPs (handled separately under `vsdd-factory:perf-check`)" — but `perf-check` is NOT in CLAUDE.md Agent Routing Table

**Problem:** NFR-001/002 are CORRECTNESS contracts (per brief BC-HOOK-022: exceeding ceilings causes Claude Code to silently drop events = data loss). They are not aspirational performance preferences. The hand-wave to "vsdd-factory:perf-check" routes to a non-existent agent. Two principled options:
- (a) Author VP-LATENCY-001 (NFR-001+002) + VP-LATENCY-002 (NFR-003); expand to 24 VPs
- (b) Add §G-6 entry deferring to concrete future story with correct agent routing (`vsdd-factory:performance-engineer`)

**Production-grade reasoning:** Latency bench infrastructure (criterion, baseline measurement, regression detection) is plausibly Phase 3 implementation work — option (b) is acceptable IF deferral cites a CONCRETE future story (not "Phase 2 generic").

**Routing:** formal-verifier (decide a/b; close).

## Observation

### Obs-R72-1 [process-gap] — VP references non-existent agent `vsdd-factory:perf-check`

**File:** VP line 99
**Defect:** `vsdd-factory:perf-check` is not in CLAUDE.md Agent Routing Table; canonical is `vsdd-factory:performance-engineer`.
**Recommendation:** Codify a new META rule: **agent-id-routing-existence sweep** — every `vsdd-factory:*` reference in spec artifacts must resolve to CLAUDE.md routing table entry.

**Routing:** formal-verifier (rename) + state-manager (codify new META rule).

## Frozen META Catalog Status

All 4 D-054 entries preserved. None re-litigated.

## Novelty Assessment

**Novelty: HIGH.** 3 NEW defect classes:
1. Schema-sketch-precision-divergence (BC tight, arch sketch loose) — partial-fix regression class extension
2. NFR-latency-testability (correctness contracts uncovered)
3. Agent-id-routing-existence (process-gap class)

Each lens-rotation continues to find new substantive defects. R72 trajectory: 3 findings (down from R71's 5, up from R69's 0). Pattern remains: each pass with new lens finds new defects.

## Pass 1 attempt 8 readiness

BLOCKED on F-R72 closure:
1. architect: F-R72-1 arch v1.0.13 → v1.0.14 (schema sketch timestamp propagation)
2. formal-verifier: F-R72-2 (option a or b) + Obs-R72-1 + arch pin propagation v1.0.13 → v1.0.14
3. state-manager: STATE.md update + agent-id-routing-existence META rule codification
4. R73 + cons R12 dispatch
