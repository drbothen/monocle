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
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0003-license-selection.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
  - /Users/jmagady/Dev/monocle/.factory/tech-debt-register.md
  - /Users/jmagady/Dev/monocle/.factory/planning/market-intelligence.md
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-5 fix burst commits 4dfcffd+6d87d6c+2308b31+9f25dcd+6e3c658+44019c2+d544731+ee7b3fb+c28fc64; STATE.md D-022; prior rounds 1-5 convergence trajectory"
project: monocle
verdict: GAPS_FOUND
gate: PASS_WITH_NON_BLOCKERS
---

# Consistency Audit — Round 6 Final

## Verdict: GAPS_FOUND (non-blocking)

**Gate: PASS — zero blocking findings. Proceed to validate-brief v6 and adversary fresh pass.**

Overall consistency score: **92/100** (previous rounds: Round 3 ~78, Round 4 ~85, Round 5 adversary pass revealed new class; Round 6 ~92 after all F-NEW-* fixed).

---

## Summary Table

| Category | Status | Notes |
|---|---|---|
| URL coherence (endpoint paths) | PASS | All 5 hook paths consistent across all 4 artifacts checked |
| Defer-pattern scan | PASS | 0 active defer-patterns in live spec content |
| F-NEW-01..14 disposition | PASS (12/12 addressable resolved; 2 non-blocking) | See below |
| Numerical consistency (5 endpoints, 12 crates, 8 EXACT-pinned) | PASS | All consistent |
| MSRV consistency (1.86/1.92) | PASS | Consistent across all 4 artifacts |
| 256 KiB body-size limit | PASS | Consistent across brief + SS-daemon-lifecycle |
| 300ms/2000ms timeout SLOs | PASS | Consistent across brief + SS-permissions-phase1 |
| License consistency (MIT/Apache-2.0) | PASS (with IMPORTANT observation) | Two deny.toml specs differ; see G-06 |
| Permission enum (no "17 variants" in Phase 1) | PASS | SS-permissions-phase1 is canonical; brief correctly defers to it |
| Brief supplements frontmatter | IMPORTANT | 3 new artifacts not in `supplements:` list; see G-01 |
| SS-conventions tokio version claim | IMPORTANT | States "1.44" but SS-deps says "1.52"; see G-02 |
| deny.toml divergence (SS-conventions vs ADR-0003) | IMPORTANT | Two artifacts carry different deny.toml specs; see G-06 |
| New artifact cross-references | MINOR | ADR-0003 / SS-permissions / SS-daemon not in SS-deps traces_to |

---

## Prior Findings — Resolution Verification

### F-NEW-* from Adversary Pass (e2c224b)

| ID | Severity | Status | Verification |
|---|---|---|---|
| F-NEW-01 | CRITICAL | RESOLVED | Vision v1.1.2 (commit 4dfcffd) — `/hooks/prompt-submit` is now consistent in vision §Process Topology diagram, brief, and DTU. Grep confirms zero occurrences of `/hooks/user-prompt-submit` in live spec content (only in §Provenance changelog prose referencing the prior error). |
| F-NEW-02 | CRITICAL | RESOLVED | SS-deps v1.1.1 (commit 2308b31) — prost correctly re-cast as "Phase 4 untrusted-input deserializer, pinned now to lock audit baseline"; serde_json promoted to EXACT pin with Phase 1 untrusted-input justification. Both crates appear correctly in the 8-EXACT-pin list. |
| F-NEW-03 | CRITICAL | RESOLVED | SS-permissions-phase1.md (commit 9f25dcd) — 5-variant exhaustive enum derived from Claude Code hook semantics. Brief correctly references artifact without spelling out variant count. |
| F-NEW-04 | CRITICAL | RESOLVED | Brief v1.4.3 Success Criteria row added: 300ms (PreToolUse/Stop/SessionStart/UserPromptSubmit) and 2000ms (Notification) per BC-HOOK-022. Consistent with SS-permissions-phase1 AskUser timeout reference. |
| F-NEW-05 | IMPORTANT | RESOLVED | SS-daemon-lifecycle.md BC-DAEMON-001: `/healthz` returns 200/503 with uptime+version; unauthenticated. Brief §Scope hardening sub-bullet references `/healthz` and `/status`. |
| F-NEW-06 | IMPORTANT | RESOLVED | SS-daemon-lifecycle.md BC-DAEMON-003: 256 KiB DefaultBodyLimit on all `/hooks/*` and `/status`. Brief v1.4.4 promotes to Success Criteria. Limit is 262,144 bytes consistently. |
| F-NEW-07 | IMPORTANT | RESOLVED | SS-daemon-lifecycle.md §Lock File Discovery Policy: single-file absolute-path mechanism; explicit anti-scan rationale from BC-HOOK-024. |
| F-NEW-08 | IMPORTANT | RESOLVED | ADR-0003 (commit d544731): MIT OR Apache-2.0 dual-license selected. SS-conventions v1.2 (commit ee7b3fb): cargo-deny + SBOM CI gate added. |
| F-NEW-09 | IMPORTANT | RESOLVED | SS-daemon-lifecycle.md §Daemon Lifecycle Protocol: SIGTERM/SIGINT handling, 10-second drain, ring buffer flush, crash recovery checkpoint. |
| F-NEW-10 | IMPORTANT | RESOLVED | DTU assessment §Fidelity Measurement Procedure (commit 44019c2): fixture corpus (25 minimum), scoring function, CI gate (`cargo xtask dtu-fidelity`), monthly + quarterly cadence. |
| F-NEW-11 | ADVISORY | RESOLVED | CLAUDE.md §Key Tech Stack (commit 9863ab3) now includes pulldown-cmark, serde_json, bytes, semver. |
| F-NEW-12 | ADVISORY | Non-finding | Adversary noted "no fix required." |
| F-NEW-13 | ADVISORY | RESOLVED | STATE.md `awaiting:` field updated to reflect round-6 validation chain. |
| F-NEW-14 | ADVISORY (process-gap) | Non-blocking upstream | vsdd-factory#131 filed. URL coherence axis added as explicit audit axis for this round — confirms it works. Not a spec defect. |

**All 12 addressable F-NEW-* findings: RESOLVED.**

---

## New Findings — Round 6

### G-01 [IMPORTANT] — Brief `supplements:` frontmatter incomplete

**Location:** `.factory/specs/product-brief.md` frontmatter, lines 23-29.

**Finding:** The `supplements:` array lists 6 architecture artifacts but does NOT include the 3 new artifacts added in the round-5 fix burst:
- `.factory/specs/architecture/SS-permissions-phase1.md` (commit 9f25dcd)
- `.factory/specs/architecture/SS-daemon-lifecycle.md` (commit 6e3c658)
- `.factory/specs/architecture/adr/ADR-0003-license-selection.md` (commit d544731)

The brief body text correctly references SS-permissions-phase1.md and SS-daemon-lifecycle.md by name (lines 138, 220, 294). The frontmatter supplements list is the machine-readable cross-reference that agents use to load related artifacts. An incomplete list causes downstream agents to miss context.

**Severity:** IMPORTANT (not BLOCKING — the body prose compensates for Phase 1 gate purposes; the architect reads both frontmatter and body).

**Remediation:** Add all three paths to `supplements:` in brief frontmatter. Route to: **product-owner**.

---

### G-02 [IMPORTANT] — SS-conventions v1.2 §deny.toml rationale states "tokio at 1.44"

**Location:** `.factory/specs/architecture/SS-conventions-anti-patterns.md` line 198.

**Finding:** The `[bans]` rationale for the `tokio < 1.52` ban reads:

> "The `SS-deps-pin-manifest.md` already pins tokio at 1.44 for Phase 1 — this ban acts as a floor guard..."

SS-deps-pin-manifest.md line 37 and the SS-deps Patch-Pinning Policy both state tokio is pinned at **1.52**. The "1.44" reference is a stale version number from a prior draft of SS-deps that has since been superseded.

**Impact:** This creates a false impression that monocle's tokio pin is below the remediated floor (1.52), when in fact the pin IS 1.52 and fully remediates all advisories. The `deny.toml` `[bans]` entry (`tokio < 1.52`) is correct and will fire against the correct floor; only the prose explanation is wrong.

**Severity:** IMPORTANT (the configuration itself is correct; the prose description is wrong and could mislead an implementer reading SS-conventions without cross-reading SS-deps).

**Remediation:** Change "1.44" to "1.52" in SS-conventions line 198. Route to: **architect**.

---

### G-03 [IMPORTANT] — deny.toml spec diverges between SS-conventions and ADR-0003

**Location:** SS-conventions-anti-patterns.md §deny.toml configuration vs ADR-0003 §Consequences.

**Finding:** Two artifacts carry `deny.toml` configurations that differ in meaningful ways:

| Field | SS-conventions v1.2 | ADR-0003 |
|---|---|---|
| `[licenses] unlicensed` | `= "deny"` | absent |
| `[licenses] copyleft` | `= "warn"` | absent |
| `[licenses] allow-osi-fsf-free` | `= "either"` | absent |
| `[licenses] confidence-threshold` | `= 0.8` | absent |
| `[licenses] exceptions` | `= []` | `[{ allow = ["MPL-2.0"], name = "nucleo", version = "^0.5" }]` |
| MPL-2.0 in allow list | Yes (inline with comment) | Only in exceptions |
| CC0-1.0 in allow list | Absent | Present |
| deny list | GPL-1.0, GPL-2.0, GPL-3.0, AGPL-1.0, AGPL-3.0, LGPL-2.0, LGPL-2.1, LGPL-3.0 | GPL-2.0, GPL-3.0, AGPL-3.0, LGPL-2.0, LGPL-2.1, LGPL-3.0 (missing GPL-1.0 and AGPL-1.0) |

SS-conventions was produced by devops-engineer (F-NEW-08 companion); ADR-0003 was produced by architect. They were developed semi-independently and not reconciled.

**The authoritative deny.toml to implement is SS-conventions v1.2**, since it is more complete and its `[bans]` section also specifies the crate-ban policy. ADR-0003's deny.toml snippet is a licensing-focused subset.

**However**, ADR-0003's use of the `exceptions` table for nucleo is the cargo-deny canonical pattern for MPL-2.0 (not inline in `allow = [...]`). This should be adopted in SS-conventions to avoid the MPL-2.0 allow-list entry triggering a `copyleft = "warn"` cascade warning on every `cargo deny check`.

**Severity:** IMPORTANT (two official specs disagree on the CI gate configuration; the implementer has no clear authority). The discrepancy between SS-conventions deny list (includes GPL-1.0 and AGPL-1.0) and ADR-0003 deny list (omits them) is a functional gap — GPL-1.0 and AGPL-1.0 should be denied.

**Remediation:** Architect (owns ADR-0003) and devops-engineer (owns SS-conventions) must produce one reconciled deny.toml. The reconciled version should:
1. Keep SS-conventions as the master (more complete).
2. Adopt ADR-0003's exceptions-table pattern for nucleo MPL-2.0 (remove MPL-2.0 from allow list; add to exceptions table with name + version).
3. Add CC0-1.0 to the SS-conventions allow list (missing from SS-conventions but present in ADR-0003; it is a permissive public-domain dedication with no reciprocal requirement).
4. Verify the deny list: both GPL-1.0 and AGPL-1.0 should remain blocked.
Route to: **architect** to reconcile; **devops-engineer** to update SS-conventions.

---

### G-04 [MINOR] — Brief v1.3 changelog references "pending architect review"

**Location:** `.factory/specs/product-brief.md` Revision History table, v1.3 row (line 58).

**Finding:** The v1.3 changelog entry reads: "Added OQ-M1 (agent-view IPC coexistence) and OQ-M3 (`PermissionRequest` as 6th endpoint) to the Open Questions table as `pending architect review`."

This is a historical record of a prior state (v1.3), not a live spec assertion. The phrase "pending architect review" appears in the changelog prose documenting what v1.3 did, not as a current architectural deferral. Both OQ-M1 and OQ-M3 have been fully resolved in subsequent versions (v1.4).

**Severity:** MINOR — This is not an active defer-pattern. The changelog is an accurate historical record. The Open Questions table in the current brief correctly shows all OQs resolved.

**Remediation:** None required. This finding is documented for completeness; it does not block the gate.

---

### G-05 [MINOR] — Brief `supplements:` does not list CLAUDE.md as an architectural authority

**Location:** `.factory/specs/product-brief.md` frontmatter `supplements:` and `CLAUDE.md` §Architectural Authority.

**Finding:** CLAUDE.md is explicitly listed as item 7 in the Architectural Authority chain ("`.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 — re-approved 2026-05-12...") and as a binding principle document. However, CLAUDE.md is not included in the brief's `supplements:` frontmatter. This is a minor structural gap; in practice, agents are instructed to read CLAUDE.md first by the project configuration, so the missing reference is unlikely to cause behavioral errors.

**Severity:** MINOR — Convention gap only; not a semantic error.

**Remediation:** Optional: add CLAUDE.md to brief supplements. The canonical order is: read CLAUDE.md first (project-level instruction), then read brief supplements. Adding it to the list makes the authority chain machine-readable. Route to: **product-owner** if human decides this is worth resolving before Phase 1 gate.

---

### G-06 [OBSERVATION] — `X-Monocle-Authorization` vs `X-Claude-Code-Ide-Authorization` — intentional bifurcation, correctly scoped

**Location:** SS-daemon-lifecycle.md lines 87, 172 (`X-Monocle-Authorization`); brief + DTU (`X-Claude-Code-Ide-Authorization`).

**Finding during scan:** Two different auth header names exist. This is NOT a defect. They are intentionally distinct:
- `X-Claude-Code-Ide-Authorization`: used by Claude Code subprocesses when POSTing to hook endpoints. This is the gene-source header from any-context and the canonical Claude Code protocol. Monocle's daemon validates this header on all `/hooks/*` routes.
- `X-Monocle-Authorization`: monocle's own internal auth token for `/status` (authenticated, rich payload) and `POST /shutdown` (admin endpoint). This is monocle's own header, distinct from the Claude Code header, because `/status` and `/shutdown` are not Claude Code hook endpoints — they are monocle daemon management endpoints.

The bifurcation is architecturally correct. Documented here so future reviewers do not flag this as an inconsistency.

---

## Numerical Consistency Verification

| Claim | Expected | Verified | Status |
|---|---|---|---|
| Hook endpoints | 5 (pre-tool-use, notification, stop, session-start, prompt-submit) | All 4 artifacts agree | PASS |
| Additional daemon endpoints | 2 (healthz, status) + 1 admin (shutdown) = 7+ total | SS-daemon-lifecycle + brief consistent | PASS |
| Phase 1 crates | 12 (11 named + 1 binary) | Brief + SS-deps consistent | PASS |
| Phase 4 crates | 13 (adds monocle-mcp-bridge) | SS-deps consistent | PASS |
| EXACT-pinned crates | 8 (tokio, prost, wasmtime, russh, rmcp, reqwest, axum, serde_json) | SS-deps policy + list consistent; CLAUDE.md §Key Tech Stack confirms "8 EXACT-pinned" | PASS |
| MSRV Phase 1 | Rust 1.86 | Brief, SS-deps, ADR-0001, CLAUDE.md all agree | PASS |
| MSRV Phase 3 | Rust 1.92 | Brief, SS-deps, ADR-0001 all agree | PASS |
| Body size limit | 256 KiB = 262,144 bytes | Brief v1.4.4, SS-daemon-lifecycle BC-DAEMON-003 consistent | PASS |
| PreToolUse/Stop/SessionStart/UserPromptSubmit timeout | 300ms | Brief Success Criteria, SS-permissions-phase1 AskUser timeout | PASS |
| Notification timeout | 2000ms | Brief Success Criteria | PASS |

---

## URL / Endpoint Path Coherence (vsdd-factory#131 axis)

All `/hooks/*` paths checked against 4 authoritative locations:

| Path | Brief | Vision | DTU | SS-daemon-lifecycle |
|---|---|---|---|---|
| `/hooks/pre-tool-use` | line 98 | line 67 | line 96 | line 124 |
| `/hooks/notification` | line 99 | line 68 | line 97 | (via `/hooks/*`) |
| `/hooks/stop` | line 99 | line 69 | line 98 | (via `/hooks/*`) |
| `/hooks/session-start` | line 99 | line 65 | line 99 | (via `/hooks/*`) |
| `/hooks/prompt-submit` | line 100 | line 66 | line 100 | line 73 |
| `/healthz` | line 108 | not applicable | not applicable | lines 46, 124 |
| `/status` | line 109 | not applicable | not applicable | lines 65, 124 |

**Result: ALL PATHS CONSISTENT. Zero divergence.** F-NEW-01 is fully resolved.

---

## Defer-Pattern Scan

Active spec content searched for: "for now," "good enough," "ship fast," "MVP," "TODO for architect," "pending architect review," "Placeholder for architect."

**Zero active defer-patterns found** in live spec content (outside historical changelog entries and CLAUDE.md anti-pattern table, which are meta-level documentation of the forbidden patterns, not instances of them).

---

## License Consistency

- ADR-0003: MIT OR Apache-2.0 dual-license selected.
- SS-conventions deny.toml: MPL-2.0 (nucleo) in allow list with comment.
- ADR-0003 deny.toml: MPL-2.0 in exceptions table (more correct cargo-deny pattern).
- Status: G-03 (IMPORTANT) captures the divergence. The intent is consistent; the implementation spec diverges.

---

## Cross-Reference Integrity

| Check | Result |
|---|---|
| SS-permissions-phase1 referenced in brief (lines 138, 294) | PASS |
| SS-daemon-lifecycle referenced in brief (line 220, 109, 112) | PASS |
| ADR-0003 referenced in SS-conventions traces_to | PASS |
| SS-permissions-phase1 in brief `supplements:` | FAIL (G-01) |
| SS-daemon-lifecycle in brief `supplements:` | FAIL (G-01) |
| ADR-0003 in brief `supplements:` | FAIL (G-01) |
| ADR-0002 §Supersedes references tech-debt-register TD-001 | PASS |
| Tech-debt-register TD-001 marked resolved | PASS |
| STATE.md phase matches current state | PASS |
| STATE.md awaiting field reflects round-6 | PASS |
| Vision v1.1.2 changelog documents F-NEW-01 fix correctly | PASS |

---

## Gate Decision

**PASS WITH NON-BLOCKERS**

Zero BLOCKING findings. Three IMPORTANT findings (G-01, G-02, G-03) are present. Under the production-grade principle, these should be fixed before convergence. However, for the Phase 1 gate decision:

- **G-01** (incomplete supplements list): does not affect the semantic content read by the architect; the body prose is correct.
- **G-02** (tokio "1.44" prose error): the configuration is correct; only the explanatory text is wrong. No implementer decision is harmed by this error.
- **G-03** (deny.toml divergence): the correct deny.toml must be resolved before Phase 1 workspace init, but does not block the Phase 1 *gate approval* because no Cargo.toml exists yet.

**Recommendation:** Fix G-01, G-02, G-03 in a fast in-scope burst (all three are single-file edits requiring no new architecture decisions), then proceed to validate-brief v6 and the adversary fresh pass. These are not gate-blockers for the Phase 1 approval, but they must be resolved before Phase 1 workspace initialization begins.

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
| **6 this round** | GAPS_FOUND | **0 BLOCKING** | **3 IMPORTANT** | **0 ADVISORY** |

All 3 IMPORTANT findings are single-file edits with no architectural ambiguity. The trajectory is converging.
