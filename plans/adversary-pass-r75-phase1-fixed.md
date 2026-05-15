---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.9 32927f6 + VP v1.9 eb6eb93 + arch v1.0.15 + manifest v1.1.10 7d8d0de; F-R74 closure chain applied; D-047 strict pass 1 of 3 (attempt 9); ALL L-F-R63 Extensions + agent-id-routing-existence codified"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-14T22:30:00Z
pass_number: 1
policy: D-047-strict
---

# Adversarial Review Pass R75 — Phase 1 (D-047 Strict, Pass 1 attempt 9 — FINDINGS)

## Summary

**Verdict:** FINDINGS — 2 MEDIUM + 2 LOW observations.

F-R74 closure verified GREEN on all 3 sub-items + R13-001 GREEN. New lens rotation: security-critical defaults + atomic-write semantics + §Trace propagation discipline + verification-coverage completeness vs BC postconditions.

## Findings

### F-R75-1 [MEDIUM] — VP-DAEMON-005 lacks 0o700 runtime-dir mode probe

**File:** VP v1.9 §VP-DAEMON-005 lines 569-748
**Cross-reference:** BC-DAEMON-005 EC-052 (PRD line 351): "Runtime directory created with mode 0o700 (owner-only). If directory creation fails, daemon logs error and exits 1." Arch SoT at SS-daemon-lifecycle.md line 252.
**Defect:** VP-DAEMON-005 §Post-conditions verifies 0o600 for lock file but ZERO probes for the runtime-dir 0o700 mode. Grep `0o700` across VP returns ZERO matches.

**Impact:** Phase 3 TDD implementer writes `std::fs::create_dir_all(&runtime_dir)?` (umask default ~0o755). VP test passes because no mode probe exists. Runtime dir becomes world-readable → information leak (other OS users can stat the dir and enumerate monocle paths).

**Fix:** Add VP-DAEMON-005 postcondition: "On runtime-dir creation (paths b/c when dir absent), `stat(&runtime_dir).mode() & 0o777 == 0o700`." Add counter-example for `create_dir_all` default umask.

**Routing:** formal-verifier.

### F-R75-2 [MEDIUM] — Windows scope drift at 4 normative-current sites vs NFR-008 canonical contract

**Files:**
- PRD line 328 BC-DAEMON-005 precondition 2 Rationale: "macOS and Windows without operator intervention"
- arch SS-daemon-lifecycle.md lines 207-213 §Start Sequence Rationale: same Windows claim
- VP line 678 probe 5.c: "happy-path on macOS/Windows"
- PRD line 1210 NFR-008 (canonical): "macOS + Linux" only — Windows NOT listed
- PRD line 1320 §8.7: explicitly "Windows is a secondary build target"

**Defect:** Rationale prose at 4 sites overstates Windows support vs NFR-008. PRD §8.7's downgrade only covers BC-ENGINE-002-ERR's `HomeUnresolvable` test, not BC-DAEMON-005. NFR-008 is the canonical platform contract.

**Impact:** Phase 3 TDD ambiguity (implementer following BC adds Windows CI; NFR-008 doesn't require). Phase 6 security audit scope ambiguity (auditor cannot decide whether Windows ACLs are in scope).

**Fix:** Disposition (a) production-grade default — tighten rationale at 4 sites: "macOS (and Windows as secondary-target per §8.7, best-effort only)". Production-grade per CLAUDE.md: answer in scope; brief §Scope established darwin/linux primary; human approved.

**Routing:** product-owner (PRD line 328) + architect (arch lines 207-213) + formal-verifier (VP line 678) — per L-F-R63 Extension 3.

## Observations

### Obs-R75-1 [LOW]
Arch §Drain step 4 mixes "append mode" + `tempfile::persist` — semantically ambiguous (persist is atomic-replace, not append). Clarity gap. Severity LOW.

### Obs-R75-2 [process-gap]
R75 surfaced defects surviving 11+ bursts. Suggest codifying new review axes:
- VP-coverage-vs-BC-EC-mode-bits (security-relevant mode/atomicity/constant-time properties)
- Rationale-vs-NFR-canonical (BC rationale prose claims vs canonical NFR/§Scope/brief)

**Routing:** state-manager (post-fix codification).

## Frozen META Catalog Status (D-054)

All 4 entries preserved. None re-litigated.

## Pass 1 attempt 10 Readiness

BLOCKED until F-R75-1 + F-R75-2 closure chain:
1. architect: arch v1.0.15 → v1.0.16 (F-R75-2 4-site rationale tightening + Obs-R75-1 optional clarification)
2. product-owner: PRD v1.9 → v1.10 (F-R75-2 + arch pin propagation)
3. formal-verifier: VP v1.9 → v1.10 (F-R75-1 0o700 probe + F-R75-2 + arch/PRD pin propagation)
4. state-manager: STATE.md update + new META axes codification
5. R76 + cons R15

## Convergence trajectory note

15 attempts: 13→5→1→4→0→2→1→0→0→3→5→3→0→3→2. Each fresh-context lens rotation continues to find substantive defects. Counter never reaches 2/3.
