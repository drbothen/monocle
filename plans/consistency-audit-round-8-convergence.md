---
document_type: consistency-audit-report
level: ops
version: "1.0"
status: complete
producer: consistency-validator
phase: pre-phase-1-final-gate-post-fix-burst
timestamp: 2026-05-12T23:59:00Z
inputs:
  - /Users/jmagady/Dev/monocle/CLAUDE.md
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0003-license-selection.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
  - /Users/jmagady/Dev/monocle/.factory/tech-debt-register.md
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-7 fix burst commits d78fc13+a22ca03+803ea63+5589849; STATE.md D-023; round-6 audit bec535d"
project: monocle
verdict: GAPS_FOUND
gate: FAIL
---

# Consistency Audit — Round 8 Convergence Check

## Verdict: GAPS_FOUND

**Gate: FAIL — two blocking findings present.**

Overall consistency score: **91/100** (round 6 was 92; two new findings discovered in the round-7 fix
burst artifacts bring the score down by 1 point).

---

## Summary Table

| Category | Status | Notes |
|---|---|---|
| G-01 Brief supplements (9 entries) | PASS | All 9 paths verified present in frontmatter |
| G-02 tokio prose "1.44" typo | PASS | SS-conventions line 198 now reads "1.52" |
| G-03 deny.toml divergence | PASS | ADR-0003 now cross-references SS-conventions as the single authoritative source; no duplicate deny.toml spec |
| serde_json concrete pin =1.0.149 | PASS | SS-deps line 41 confirmed |
| rand =0.8.6 row present | PASS | SS-deps line 45 confirmed; appears in Pin Manifest table |
| rand in mermaid diagram | PASS | SS-deps mermaid line 150: `runtime --> rand` confirmed |
| 9 EXACT-pinned crates (not 8) | PARTIAL | Header/list correct (lines 94, 96, 111, 113); ONE stale "8" in rationale prose (line 103) — see R8-002 |
| SS-conventions tokio prose "1.52" | PASS | Line 198 confirmed correct |
| /healthz unauthenticated | PASS | SS-daemon-lifecycle §Authentication: "unauthenticated" confirmed; public_router has only /healthz |
| axum::serve with_graceful_shutdown | PASS | SS-daemon-lifecycle line 225 confirmed correct idiom |
| two-router pattern in code example | PASS | public_router + authed_router merge pattern confirmed |
| ADR-0003 cross-references SS-conventions deny.toml | PASS | ADR-0003 lines 107-115 explicitly defer to SS-conventions as single authoritative source |
| Brief body-size criterion | PASS | Success Criteria row lists only 5 POST hook endpoints; /healthz and /status absent |
| URL coherence (5 hook paths + /healthz + /status) | FAIL | SS-daemon-lifecycle code example has wrong 5th hook route — see R8-001 (BLOCKING) |
| Defer-pattern scan | PASS | 0 active defer-patterns; "pending architect review" in v1.3 changelog row is historical record only |
| MSRV consistency (1.86/1.92) | PASS | Consistent across all artifacts |
| 256 KiB body size limit | PASS | Consistent; /healthz correctly excluded |
| 300ms/2000ms timeout SLOs | PASS | Consistent across brief and SS-permissions-phase1 |

---

## Prior Findings — Resolution Verification

### Round 6 Findings (G-01, G-02, G-03)

| ID | Severity | Status | Verification |
|---|---|---|---|
| G-01 | IMPORTANT | RESOLVED | Brief v1.4.5 supplements frontmatter lists 9 entries; all 3 new artifacts (SS-permissions-phase1.md, SS-daemon-lifecycle.md, ADR-0003) present. Verified at brief lines 23–32. |
| G-02 | IMPORTANT | RESOLVED | SS-conventions line 198 now reads "already pins tokio at 1.52 for Phase 1". No remaining "1.44" in any non-historical context. Verified via grep. |
| G-03 | IMPORTANT | RESOLVED | ADR-0003 §Rationale (lines 107–115) explicitly defers deny.toml content to SS-conventions as single authoritative source. No duplicate conflicting deny.toml spec in ADR-0003. Verified at ADR-0003 lines 107-115. |

### Prior F-NEW-* (from round-5 adversary, addressed in rounds 5-7)

All 12 addressable F-NEW-* findings remain RESOLVED. No regression detected across any of them.

---

## New Findings — Round 8

### R8-001 [BLOCKING] — SS-daemon-lifecycle code example registers `/hooks/post-tool-use` instead of `/hooks/prompt-submit`

**Location:** `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md` line 139.

**Finding:** The Rust code example in §Body Size Limit (F-NEW-06 implementation guidance) shows the
authenticated router with the following hook routes:

```rust
.route("/hooks/pre-tool-use", post(pre_tool_use_handler))
.route("/hooks/post-tool-use", post(post_tool_use_handler))   // LINE 139 — DEFECT
.route("/hooks/notification", post(notification_handler))
.route("/hooks/stop", post(stop_handler))
.route("/hooks/session-start", post(session_start_handler))
```

This is wrong in two ways:

1. `/hooks/post-tool-use` appears — this endpoint is **explicitly excluded** from Phase 1 by JC-2 ("omit
   PostToolUse to preserve gene-source parity per any-context BC-HOOK-007"). The brief, vision,
   and DTU all state 5 endpoints; none of the 5 is `post-tool-use`.

2. `/hooks/prompt-submit` is **absent** — this is the canonical 5th endpoint (`UserPromptSubmit` hook)
   per EX-2 resolution. It appears in every other canonical artifact (brief line 104, vision line 66,
   DTU line 100) but is missing from the SS-daemon-lifecycle code example.

The code example thus registers the wrong 5 hook routes (4 correct + 1 wrong; missing 1 correct).

**Cross-artifact URL coherence check:**

| Path | Brief | Vision | DTU | SS-daemon-lifecycle code |
|---|---|---|---|---|
| `/hooks/pre-tool-use` | line 102 | line 67 | line 96 | line 138 — PASS |
| `/hooks/notification` | line 103 | line 68 | line 97 | line 140 — PASS |
| `/hooks/stop` | line 103 | line 69 | line 98 | line 141 — PASS |
| `/hooks/session-start` | line 103 | line 65 | line 99 | line 142 — PASS |
| `/hooks/prompt-submit` | line 104 | line 66 | line 100 | **ABSENT** — FAIL |
| `/hooks/post-tool-use` | EXCLUDED | EXCLUDED | EXCLUDED | line 139 — DEFECT |

**Impact:** An implementer reading this code example would register the wrong set of hook routes.
`/hooks/prompt-submit` would never receive `UserPromptSubmit` events; `/hooks/post-tool-use` would
be registered but Claude Code never POSTs to it (PostToolUse is excluded). The daemon would silently
fail to ingest UserPromptSubmit hook events — a Phase 1 parity failure.

**Severity:** BLOCKING. URL coherence across spec artifacts is the explicit audit axis established
at vsdd-factory#131. This is a wire-protocol spec defect in the implementation example.

**Remediation:** In SS-daemon-lifecycle.md line 139, replace:

```rust
.route("/hooks/post-tool-use", post(post_tool_use_handler))
```

with:

```rust
.route("/hooks/prompt-submit", post(prompt_submit_handler))
```

Route to: **architect** (owns SS-daemon-lifecycle).

---

### R8-002 [IMPORTANT] — SS-deps-pin-manifest.md line 103 reads "8 security-sensitive crates" — stale count

**Location:** `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md` line 103.

**Finding:** The Patch-Pinning Policy rationale paragraph reads:

> "The **8 security-sensitive crates** handle untrusted network input or operate on security-critical
> protocol boundaries."

The header on line 94 correctly states "9 security-sensitive crates" and line 96 correctly lists all 9.
Line 111 and 113 in the Security Advisory Response Policy also correctly say "9 EXACT-pinned crates."
The "8" on line 103 is a stale count left over from before `rand =0.8.6` was added as the 9th
EXACT-pinned crate in the round-7 fix burst (commit d78fc13).

**Impact:** An implementer reading line 103 may believe only 8 crates require security-reviewer
dispatch, missing `rand` (the auth token generator). Given `rand` = auth token generation = security-
critical, this could result in `rand` bumps being auto-merged without the required security review.

**Severity:** IMPORTANT (not BLOCKING for the Phase 1 gate; the correct value is stated 4 times in the
same section including the headline and the Security Advisory Response Policy).

**Remediation:** SS-deps-pin-manifest.md line 103: change "The 8 security-sensitive crates" to
"The 9 security-sensitive crates". Route to: **architect**.

---

## URL / Endpoint Path Coherence (vsdd-factory#131 axis)

Full cross-artifact check:

| Path | Brief | Vision | DTU | SS-daemon-lifecycle prose | SS-daemon-lifecycle code |
|---|---|---|---|---|---|
| `/hooks/pre-tool-use` | PASS | PASS | PASS | (via /hooks/*) | PASS (line 138) |
| `/hooks/notification` | PASS | PASS | PASS | (via /hooks/*) | PASS (line 140) |
| `/hooks/stop` | PASS | PASS | PASS | (via /hooks/*) | PASS (line 141) |
| `/hooks/session-start` | PASS | PASS | PASS | (via /hooks/*) | PASS (line 142) |
| `/hooks/prompt-submit` | PASS | PASS | PASS | (via /hooks/*) | **FAIL** (absent) |
| `/hooks/post-tool-use` | excluded | excluded | excluded | excluded | **DEFECT** (line 139) |
| `/healthz` | PASS (line 112) | N/A | N/A | PASS (lines 36, 132) | PASS (line 132) |
| `/status` | PASS (line 112) | N/A | N/A | PASS (lines 65, 143) | PASS (line 143) |

**Result: ONE path failure.** The code example in SS-daemon-lifecycle has the wrong 5th hook route.
All prose references are consistent. This is a code-example-only defect but the code example is the
implementation spec for the architect and implementer.

---

## Round-7 Fix Burst Verification

| Fix | Claimed in D-023 | Verified |
|---|---|---|
| F-R6-001: serde_json concrete pin =1.0.149 | SS-deps v1.1.2 | PASS — line 41 confirmed |
| F-R6-002/G-02: tokio prose 1.44→1.52 | SS-conventions v1.2.1 line 198 | PASS — confirmed |
| F-R6-003: /healthz two-router auth split | SS-daemon-lifecycle v1.0.1 | PASS — public_router/authed_router pattern confirmed |
| F-R6-004: rand =0.8.6 EXACT-pinned | SS-deps v1.1.2 line 45 | PASS — confirmed |
| F-R6-005: axum 0.8 with_graceful_shutdown idiom | SS-daemon-lifecycle line 225 | PASS — confirmed |
| F-R6-006: /healthz removed from body-size criterion | Brief v1.4.5 Success Criteria | PASS — endpoint list is 5 POST hooks only |
| G-01: brief supplements 9 entries | Brief v1.4.5 frontmatter | PASS — 9 entries confirmed |
| G-03: deny.toml cross-ref to ADR-0003 | SS-conventions traces_to + ADR-0003 §Rationale | PASS — ADR-0003 defers to SS-conventions |

8/8 round-7 fixes verified. Two NEW defects introduced or pre-existing in the round-7 artifacts.

---

## Defer-Pattern Scan

Active spec content searched for: "for now," "good enough," "ship fast," "MVP," "TODO for architect,"
"pending architect review," "Placeholder for architect."

**Zero active defer-patterns found** in live spec content. The single hit in brief Revision History
(v1.3 row mentioning "pending architect review") is a historical record of what v1.3 did; the current
brief resolves all OQs.

The one ADR-0002 hit ("MVP-shaped risk rationalization") is prose explicitly prohibiting that pattern,
not an instance of it.

**Defer-pattern count: 0**

---

## Numerical Consistency

| Claim | Expected | Verified | Status |
|---|---|---|---|
| Hook endpoints | 5 | All prose artifacts agree | PASS (code example wrong — see R8-001) |
| Phase 1 crates | 12 (11 named + 1 binary) | Brief + SS-deps consistent | PASS |
| EXACT-pinned crates | 9 | Header/list/policy consistent; one stale "8" in rationale prose | PARTIAL (see R8-002) |
| MSRV Phase 1 | Rust 1.86 | All artifacts agree | PASS |
| MSRV Phase 3 | Rust 1.92 | All artifacts agree | PASS |
| Body size limit | 256 KiB = 262,144 bytes | Brief + SS-daemon-lifecycle consistent | PASS |
| /healthz in body-size scope | No (GET, no body) | Brief excludes it; SS-daemon-lifecycle excludes it | PASS |
| 300ms timeout (4 hooks) | 300ms | Brief Success Criteria | PASS |
| 2000ms timeout (Notification) | 2000ms | Brief Success Criteria | PASS |
| rand in mermaid graph | runtime --> rand | SS-deps line 150 | PASS |

---

## Gate Decision

**FAIL — one BLOCKING finding, one IMPORTANT finding.**

**R8-001 is BLOCKING.** A code example that registers the wrong hook routes is a wire-protocol
spec defect in the implementation guide. An implementer following this example would register
`/hooks/post-tool-use` (which Claude Code never POSTs to) and omit `/hooks/prompt-submit` (which
Claude Code fires for every UserPromptSubmit event). Under the production-grade principle
(CLAUDE.md §Rule 1), wire-protocol errors block convergence.

**R8-002 is IMPORTANT.** The stale "8" in a rationale sentence creates a documentation inconsistency
that could cause a reader to miss `rand` as a security-review-required bump. Should be fixed in the
same burst as R8-001.

**Both are single-line fixes.** The burst is: (1) change line 139 in SS-daemon-lifecycle and (2)
change line 103 in SS-deps. No architectural decisions required.

---

## Convergence Trajectory

| Round | Verdict | Blocking | Important | Advisory |
|---|---|---|---|---|
| 1 (b891b78) | GAPS_FOUND | 0 | 4 | 6 |
| 2 post-remediation (0f28619) | GAPS_FOUND | 2 | 3 | 2 |
| 3 post-fix-burst (f8bffd8) | GAPS_FOUND | 0 | 2 | 3 |
| 4 post-fix-burst (c2bf9e2) | GAPS_FOUND | 0 | 1 | 1 |
| (adversary fresh pass e2c224b) | MULTIPLE_DEFER_PATTERNS | 4 CRITICAL | 6 IMPORTANT | 4 ADVISORY |
| 5 — round-5 fix burst | all F-NEW-* fixed in 9 commits | — | — | — |
| 6 (bec535d) | GAPS_FOUND | 0 BLOCKING | 3 IMPORTANT | 0 ADVISORY |
| 7 — round-7 fix burst | G-01/G-02/G-03 fixed in 4 commits | — | — | — |
| **8 this round** | GAPS_FOUND | **1 BLOCKING** | **1 IMPORTANT** | **0 ADVISORY** |

The new findings (R8-001 BLOCKING, R8-002 IMPORTANT) are both single-line edits with zero
architectural ambiguity. They were introduced in the round-7 fix burst artifacts.

**Recommendation: one more micro-fix burst (round-8), then proceed to validate-brief v8 and
human Phase 1 approval gate.**
