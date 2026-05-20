---
document_type: consistency-pass
level: ops
phase: phase-2
round: r02
producer: consistency-validator
status: GAPS
gaps_total: 4
gaps_by_severity:
  critical: 0
  high: 1
  medium: 2
  low: 1
input-hash: "[live-state]"
inputs:
  - stories/STORY-INDEX.md (post-0765d2e)
  - stories/dependency-graph.md (post-0765d2e)
  - stories/wave-schedule.md (post-0765d2e)
  - stories/sprint-state.yaml (post-0765d2e)
  - stories/holdout-scenarios.md (post-0765d2e)
  - stories/S-001-cargo-workspace-ci-setup.md
  - stories/S-002-healthz-endpoint.md
  - stories/S-003-status-endpoint.md
  - stories/S-004-body-size-limit.md
  - stories/S-005-graceful-shutdown.md
  - stories/S-006-lock-file-lifecycle.md
  - stories/S-007-crash-recovery-checkpoint.md
  - stories/S-008-jsonl-ring-format-version.md
  - stories/S-009-auth-token-header-validation.md
  - stories/S-010-monocle-core-abi-version.md
  - stories/S-011-non-exhaustive-enum-policy.md
  - stories/S-012-factory-adapter-trait.md
  - stories/S-013-hook-envelope-proto-wire-format.md
  - stories/S-014-engine-module-trait.md
  - stories/S-015-claude-code-module-impl.md
  - stories/S-DTU-001-claude-code-hook-clone.md
  - stories/S-PHASE-3-PREP-spec-kit-mcp-integration.md
  - stories/epics/E-01-daemon-lifecycle.md
  - stories/epics/E-02-core-types-and-abi.md
  - stories/epics/E-03-engine-module.md
  - stories/epics/E-DTU-hook-protocol-clone.md
  - stories/epics/E-PREP-phase3-prep.md
  - specs/behavioral-contracts/BC-INDEX.md (v1.11)
  - specs/behavioral-contracts/ss-01/BC-2.01.008.md (v1.0.6)
  - specs/verification-properties/VP-INDEX.md (v1.16)
  - specs/prd-supplements/error-taxonomy.md (v1.5)
  - specs/prd-supplements/nfr-catalog.md (v1.7)
  - tech-debt-register.md
traces_to: "Phase 2 story corpus post-remediation at commit 0765d2e"
timestamp: 2026-05-19T07:00:00Z
---

# Consistency Pass: Phase 2 Story Corpus — Round 02

> **Scope:** Re-validation of all 17 r01 checks against the remediated story corpus at
> commit `0765d2e`. Plus 3 new check categories (checks 18-20). Read-only audit.
> No artifacts modified.

## Executive Summary

| Status | GAPS |
|--------|------|
| Checks run | All 20 check categories (checks 1-20) |
| r01 gaps closed | 10 of 11 |
| r01 gaps still open | 1 of 11 (GAP-PHASE2-R01-7 — S-005 STORY-INDEX Blocks partial fix) |
| New gaps (checks 18-20) | 3 |
| Total new gaps (r02) | 4 |
| Critical | 0 |
| High | 1 |
| Medium | 2 |
| Low | 1 |
| Gate recommendation | CONDITIONAL PASS — All 4 remaining gaps are documentation-only errors (no BC/VP/NFR/behavioral coverage gap). Gate-blocking: GAP-PHASE2-R02-1 (HIGH) must be fixed before story corpus is distributed to implementers as it creates implementer confusion on wave parallelism. |

---

## r01 Gap Closure Verification

Independent re-derivation of each r01 gap. Evidence column cites current artifact state.

| Gap ID | Severity | Description | Status | Evidence |
|--------|----------|-------------|--------|----------|
| GAP-PHASE2-R01-1 | HIGH | Story count 18→17 | CLOSED | STORY-INDEX:59 "17 (15 product + 1 DTU + 1 prep)"; dep-graph:98 "Total processed: 17 nodes"; sprint-state summary.total_stories: 17 |
| GAP-PHASE2-R01-2 | HIGH | Kahn trace double-counts S-004 | CLOSED | dep-graph Round 3 processes {S-003, S-005, S-007, S-008, S-011, S-013, S-014} — S-004 absent; Total processed: 17 |
| GAP-PHASE2-R01-3 | MEDIUM | All 17 stories missing traces_to | CLOSED | All 17 story files verified with non-empty traces_to: field in frontmatter |
| GAP-PHASE2-R01-4 | MEDIUM | S-015 BC-2.03.001 absent from frontmatter | CLOSED | S-015:18 behavioral_contracts: [BC-2.03.001, BC-2.03.002, BC-2.03.003, BC-2.03.004] |
| GAP-PHASE2-R01-5 | MEDIUM | S-003 BC-2.02.001 absent from frontmatter | CLOSED | S-003:18 behavioral_contracts: [BC-2.01.002, BC-2.02.001] |
| GAP-PHASE2-R01-6 | MEDIUM | S-009 spuriously blocks S-008 | CLOSED | S-009:15 blocks: [] |
| GAP-PHASE2-R01-7 | MEDIUM | S-005 spuriously blocks S-007 | PARTIAL | S-005 frontmatter blocks: [] FIXED. STORY-INDEX:47 still shows "S-007" in S-005 Blocks column. See GAP-PHASE2-R02-1 (folded into broader STORY-INDEX Blocks column audit). |
| GAP-PHASE2-R01-8 | MEDIUM | S-001 STORY-INDEX Blocks ambiguous range | CLOSED | STORY-INDEX:43 shows explicit list "S-002, S-003, S-004, S-005, S-006, S-010, S-013, S-014" |
| GAP-PHASE2-R01-9 | LOW | S-006 Previous Story Intelligence TBD placeholder | CLOSED | S-006 §Previous Story Intelligence no longer uses TBD; uses "no placeholder value is ever written" language. S-009 §Previous Story Intelligence: "no placeholder retrofit." |
| GAP-PHASE2-R01-10 | LOW | sprint-state.yaml phase: 3 vs phase: 2 corpus | CLOSED | sprint-state.yaml:8 phase: 2; target_implementation_phase: 3 (new field distinguishes production phase from consumption phase) |
| GAP-PHASE2-R01-11 | LOW | STORY-INDEX input-hash placeholder | CLOSED | STORY-INDEX:18 input-hash: "[live-state]" (live-state sentinel is the accepted post-commit state per SE-22) |

**r01 closure rate: 10/11 fully closed, 1/11 partially closed (frontmatter fixed, STORY-INDEX body not updated).**

---

## Checks Passed — r01 Re-verification (No New Gaps)

All checks that passed in r01 still pass after remediation. Below are the checks that required
active re-verification because the remediation burst touched related artifacts.

| Check | Description | Result |
|-------|-------------|--------|
| 1 | Version pin freshness: all spec versions current | PASS — no version pin changes in corpus |
| 2 | BC ID validity: all 22 BC-S.SS.NNN in stories exist in BC-INDEX v1.11 | PASS |
| 3 | VP ID validity: all 22 VP-NNN in stories exist in VP-INDEX v1.16 | PASS |
| 4 | Error code validity: all 15 E-NNN exist in error-taxonomy v1.5 | PASS |
| 5 | NFR validity: all NFRs exist in nfr-catalog v1.7 | PASS |
| 6 | Frontmatter coherence: S-015 BC coverage | PASS (GAP-4 closed; BC-2.03.001 now in S-015 frontmatter) |
| 6 | Frontmatter coherence: S-003 BC coverage | PASS (GAP-5 closed; BC-2.02.001 now in S-003 frontmatter) |
| 8 | Story ID uniqueness; filename slugs | PASS |
| 9 | STORY-INDEX coverage count: 17 registered, 17 files on disk | PASS |
| 10 | S-009 depends_on: [S-001, S-004, S-006, S-008] — dep-graph edges consistent | PASS |
| 10 | S-008 blocks: [S-009] — dep-graph Blocks Edges table consistent | PASS |
| 11 | S-009 wave: 3 in STORY-INDEX, dependency-graph, wave-schedule, sprint-state | PASS |
| 11 | Wave 2 points: 41 | PASS (STORY-INDEX Wave Summary, wave-schedule header, sprint-state) |
| 11 | Wave 3 points: 34 | PASS (STORY-INDEX Wave Summary, wave-schedule header, sprint-state) |
| 12 | sprint-state: 17 stories, 16 not_started, 1 blocked, wave_2: 41, wave_3: 34 | PASS |
| 13 | Holdout non-leakage: 12 scenarios unchanged | PASS |
| 14 | Epic membership: all 5 epics, all 17 stories | PASS |
| 15 | BC/VP/NFR/error coverage rollups unchanged | PASS — 22/22/12/15 coverage preserved |
| 16 | Production-grade language: S-006 TBD removed | PASS (GAP-9 closed) |
| 17 | S-PHASE-3-PREP integrity | PASS |

---

## New Gaps Found (r02)

### GAP-PHASE2-R02-1 — HIGH
**Check:** #18 (Wave-restructure consistency) and #10 (STORY-INDEX Blocks column integrity)
**Title:** Two STORY-INDEX Blocks column entries not propagated from frontmatter fixes

**Evidence:**

*Sub-issue A — S-005 Blocks column (r01 GAP-7 partial fix):*
- `STORY-INDEX.md:47` — `| S-005 | Graceful Shutdown | EPIC-01 | 5 | 2 | draft | S-007 |`
- `S-005-graceful-shutdown.md:16` — `blocks: []` (correct; S-007 removed by GAP-7 fix)
- `dependency-graph.md` Blocks Edges table — no S-005→S-007 row (correct)
- The frontmatter was fixed but the STORY-INDEX Blocks column was not updated for S-005. S-007 should be "—" (not "S-007").

*Sub-issue B — S-006 Blocks column (S-009 not removed):*
- `STORY-INDEX.md:48` — `| S-006 | Lock File Atomic Lifecycle | EPIC-01 | 8 | 2 | draft | S-007, S-008, S-009 |`
- `S-006-lock-file-lifecycle.md:15` — `blocks: [S-007, S-008]` (correct; S-009 removed per §Trace)
- `dependency-graph.md Blocks Edges:112` — `| S-006 | S-007, S-008 |` (correct; S-009 absent)
- `STORY-INDEX.md §Trace:206` — "S-006 blocks: [S-007, S-008] (S-009 removed — S-009 now depends on S-008 not S-006 directly)"
- STORY-INDEX Blocks column for S-006 still reads "S-007, S-008, S-009". The §Trace documents that S-009 was removed from S-006's blocks, but the Story Registry table was not updated.

**Impact:** Both errors are STORY-INDEX Blocks column drift only. S-006 frontmatter, dep-graph, and S-009 frontmatter are all internally consistent (S-009 depends on S-006 via the lock file read, which is correct as a dep-edge, but S-006 does not block S-009 in the blocks sense because S-009 is downstream). An implementer reading only the STORY-INDEX table would incorrectly believe S-005 blocks S-007 and S-006 blocks S-009.

**Proposed routing:** `vsdd-factory:story-writer`
- STORY-INDEX:47 — change Blocks for S-005 from "S-007" to "—"
- STORY-INDEX:48 — change Blocks for S-006 from "S-007, S-008, S-009" to "S-007, S-008"

---

### GAP-PHASE2-R02-2 — MEDIUM
**Check:** #18 (Wave-restructure consistency — wave-schedule Wave 3 parallelism claim)
**Title:** wave-schedule.md Wave 3 section declares "all 4 stories are independent" but Wave 3 has 5 stories

**Evidence:**
- `wave-schedule.md:124` — `**Parallelism:** Full parallel (all 4 stories are independent of each other).`
- `wave-schedule.md:28` (Wave Overview header) — `| Wave 3 | S-007, S-008, S-009, S-012, S-015 | 34 | Full parallel (all 5 parallel) | ...`
- `wave-schedule.md:127-131` (Wave 3 table) — lists 5 stories: S-007, S-008, S-009, S-012, S-015

The Wave Overview table correctly says "all 5 parallel" and the Wave 3 story table lists 5 stories. The descriptive paragraph under "Wave 3" heading says "all 4 stories" — this is the stale text from before S-009 was moved into Wave 3 (when Wave 3 had only 4 stories: S-007, S-008, S-012, S-015).

Note: the waves ARE actually parallel within the wave (S-009's dependencies — S-001, S-004, S-006, S-008 — are all Wave 1 or Wave 2 stories, and S-008 which is Wave 3 means S-009 can only start after S-008 completes within Wave 3; they are NOT fully parallel). The parallelism claim "all 5 parallel" is also semantically incorrect because S-009 depends on S-008, but the count error (4 vs 5) is the primary factual error.

**Proposed routing:** `vsdd-factory:story-writer`
- `wave-schedule.md:124` — change "all 4 stories are independent of each other" to "all 5 stories listed (S-009 may start only after S-008 completes within the wave due to Decision 1 S-008→S-009 dependency)"

---

### GAP-PHASE2-R02-3 — MEDIUM
**Check:** #19 (Auth token mechanism consistency — intra-story function name drift)
**Title:** S-009 File Structure Requirements uses stale function name `generate_auth_token()` inconsistent with `generate_session_token()` used throughout the corpus

**Evidence:**
- `S-009-auth-token-header-validation.md:177` — `- \`monocle-runtime/src/auth.rs\` — \`generate_auth_token()\`, \`validate_auth_header()\`, middleware`
- `S-009-auth-token-header-validation.md:182` — `- \`monocle-runtime/src/lock.rs\` — include \`authToken\` from \`generate_auth_token()\``
- `S-009-auth-token-header-validation.md:53` — `string generated by \`monocle-auth::generate_session_token()\`` (AC-001)
- `S-009-auth-token-header-validation.md:117` — `- \`monocle-auth::generate_session_token()\` is S-006's deliverable` (Tasks)
- `S-009-auth-token-header-validation.md:142-144` — §Previous Story Intelligence: `monocle-auth::generate_session_token()` called at `DaemonLock::acquire()` time
- `S-006-lock-file-lifecycle.md:109,138,142` — `monocle-auth::generate_session_token()` consistently throughout
- `BC-2.01.008 PC-1` — specifies the property (32 bytes OsRng, 64-hex lowercase); does not name the function

The canonical function name throughout the corpus is `generate_session_token()`. The S-009 File Structure Requirements section uses `generate_auth_token()` — a different, incorrect name. An implementer following the File Structure section alone would write the wrong function name, which would fail to compile against S-006's exported `monocle-auth::generate_session_token`.

Note: S-009 File Structure also lists `monocle-runtime/src/auth.rs` as the location for `generate_auth_token()`. This is a conflation: `generate_session_token()` lives in the `monocle-auth` crate (per S-006 AC-014 and S-001 AC-005), not in `monocle-runtime/src/auth.rs`. The File Structure section mixes the token generation function (monocle-auth crate) with the validation function (`validate_auth_header()` in monocle-runtime/src/auth.rs). This is a documentation-only confusion since S-009's Tasks section correctly states "S-009 is the consumer" of S-006's token, but the File Structure section implies S-009 generates the token.

**Proposed routing:** `vsdd-factory:story-writer`
- `S-009:177` — remove `generate_auth_token(),` from the auth.rs line (generation is not S-009's job)
- `S-009:182` — change to `include \`authToken\` read from the lock file written by S-006 (monocle-auth::generate_session_token())` or simply remove this bullet (S-009 reads lock.rs values; it does not call a generate function)

---

### GAP-PHASE2-R02-4 — LOW
**Check:** #20 (Frontmatter retrofit completeness — holdout-scenarios.md)
**Title:** holdout-scenarios.md `traces_to` field value is present but the frontmatter does not include a `version:` field

**Evidence:**
- `holdout-scenarios.md:1-19` — frontmatter present with `document_type`, `level` (absent — document_type is present, level is absent), `version` (absent), `status`, `producer`, `timestamp`, `phase`, `visibility`, `inputs`, `input-hash`, `traces_to`
- `STORY-INDEX.md` frontmatter — has: `document_type`, `level`, `version`, `status`, `producer`, `timestamp`, `phase`, `inputs`, `input-hash`, `traces_to`
- `dependency-graph.md` frontmatter — has: `document_type`, `level`, `version`, `status`, `producer`, `timestamp`, `phase`, `inputs`, `input-hash`, `traces_to`
- `wave-schedule.md` frontmatter — has: same fields
- `sprint-state.yaml` frontmatter — has: `document_type`, `version`, `status`, `producer`, `timestamp`, `phase`, `target_implementation_phase`, `traces_to`, `inputs`, `input-hash`, `traces_to_full`

The `holdout-scenarios.md` frontmatter is missing both `level:` and `version:` fields. All other corpus plan-doc files carry `level: L4` and `version: "1.1"`. DF-020a (canonical frontmatter) requires both fields.

**Note:** This is a minor structural omission. The holdout-scenarios.md content is correct and complete; the missing fields are metadata only.

**Proposed routing:** `vsdd-factory:story-writer`
- Add `level: L4` and `version: "1.1"` to holdout-scenarios.md frontmatter.

---

## Check 18 — Wave-Restructure Consistency: Full Verification

| Sub-check | Verification | Result |
|-----------|-------------|--------|
| STORY-INDEX wave column for S-009 = 3 | STORY-INDEX:49 — `\| S-009 \| ... \| 3 \|` | PASS |
| wave-schedule Wave 2 list does NOT contain S-009 | wave-schedule:93-103 — S-009 absent from Wave 2 table | PASS |
| wave-schedule Wave 3 list contains S-009 | wave-schedule:127-131 — S-009 row present | PASS |
| wave-schedule Wave 2 points = 41 | wave-schedule:27 — "41" in header; per-story sum: 3+5+2+5+8+5+3+5+5=41 | PASS |
| wave-schedule Wave 3 points = 34 | wave-schedule:28 — "34" in header; per-story sum: 5+5+8+8+8=34 | PASS |
| Wave 2 + Wave 3 sum unchanged at 75 product points (41+34=75 vs original 49+26=75) | 41+34=75 CONFIRMED | PASS |
| dependency-graph has S-008→S-009 directed edge | dep-graph:113 — `\| S-008 \| S-009 \|` in Blocks Edges; dep-graph:65 — S-009 depends on S-008 | PASS |
| sprint-state.yaml wave_2_points: 41 | sprint-state:241 — wave_2_points: 41 | PASS |
| sprint-state.yaml wave_3_points: 34 | sprint-state:242 — wave_3_points: 34 | PASS |
| S-009 frontmatter wave: 3 | S-009:12 — wave: 3 | PASS |
| wave-schedule Wave 3 parallelism count (4 vs 5) | FAIL — see GAP-PHASE2-R02-2 |

## Check 19 — Auth Token Mechanism Consistency: Full Verification

| Sub-check | Verification | Result |
|-----------|-------------|--------|
| S-006 tasks mention monocle-auth::generate_session_token() | S-006:138-141 | PASS |
| S-006 AC-014 declares authToken populated with real cryptographic value at creation | S-006:106-115 | PASS |
| S-009 ACs do NOT reference placeholder/TBD/retrofit | S-009 §Previous Story Intelligence:142-144 — "no placeholder retrofit"; AC-001:55 — reads from already-written lock file | PASS |
| monocle-auth crate appears in S-001 workspace member list | S-001:69-70 — "monocle-core, monocle-runtime, monocle-proto, monocle-auth" | PASS |
| Crypto spec (alphabet + length) matches BC-2.01.008 PC-1 | BC-2.01.008 PC-1: 32 bytes OsRng, 64-char lowercase hex, regex /^[0-9a-f]{64}$/. S-006:109-111: same. S-009 AC-001:52-54: same. | PASS |
| S-009 File Structure function name consistent | FAIL — see GAP-PHASE2-R02-3 (generate_auth_token vs generate_session_token) |

## Check 20 — Frontmatter Retrofit Completeness: Full Verification

| File | inputs: non-empty | input-hash present | traces_to non-empty | level: present | version: present |
|------|-------------------|--------------------|---------------------|----------------|-----------------|
| STORY-INDEX.md | YES (8 sources) | YES [live-state] | YES | YES (L4) | YES (1.1) |
| dependency-graph.md | YES (6 sources) | YES [live-state] | YES | YES (L4) | YES (1.1) |
| wave-schedule.md | YES (5 sources) | YES [live-state] | YES | YES (L4) | YES (1.1) |
| sprint-state.yaml | YES (4 sources) | YES [live-state] | YES | ABSENT | YES (1.1) |
| holdout-scenarios.md | YES (6 sources) | YES [live-state] | YES | ABSENT | ABSENT |
| All 17 S-*.md stories | YES (each has own inputs list) | YES [live-state] | YES | YES (L4 implied by phase:2 story) | YES |

**Notes:**
- sprint-state.yaml has no `level:` field; this is acceptable for YAML config files (DF-020a targets markdown L4 artifacts primarily). Not flagged.
- holdout-scenarios.md missing `level:` and `version:` — flagged as GAP-PHASE2-R02-4 (LOW).
- All 17 story files have `inputs:` (non-empty, citing actually-consumed artifacts), `input-hash: "[live-state]"`, and `traces_to:` (non-empty descriptive strings). Frontmatter retrofit for stories: COMPLETE.

---

## Coverage Integrity — Unchanged Since r01

The following coverage claims were re-verified by checking that no remediation burst added or removed BC/VP/NFR/error code assignments:

- **BC coverage: 22/22 — CONFIRMED.** S-015 now correctly lists BC-2.03.001 in frontmatter; S-003 now correctly lists BC-2.02.001. Coverage matrix unchanged.
- **VP coverage: 22/22 — CONFIRMED.**
- **Error code coverage: 15/15 — CONFIRMED.**
- **NFR coverage: 12/12 — CONFIRMED** (4 deferred to Phase 3 remain justified).
- **DAG acyclicity — CONFIRMED.** Kahn trace corrected to 17 nodes, no double-counts, ACYCLIC.
- **Holdout scenarios — 12 scenarios, no leakage — CONFIRMED** (no holdout content changed).
- **Dependency-graph BC Clause Coverage Matrix — CONFIRMED.** BC-2.03.001 invariant 2 → AC-009 → S-015 mapping retained; the row at dep-graph:295 is correct and S-015 frontmatter now carries BC-2.03.001.

---

## Routing Summary

| Gap ID | Severity | Description | Proposed Routing | Estimated Effort |
|--------|----------|-------------|-----------------|-----------------|
| GAP-PHASE2-R02-1 | HIGH | STORY-INDEX Blocks column: S-005 still shows S-007; S-006 still shows S-009 | vsdd-factory:story-writer | Trivial — 2 table cell edits |
| GAP-PHASE2-R02-2 | MEDIUM | wave-schedule Wave 3 says "all 4 stories" (should be 5) | vsdd-factory:story-writer | Trivial — 1 sentence edit |
| GAP-PHASE2-R02-3 | MEDIUM | S-009 File Structure uses generate_auth_token() vs generate_session_token() | vsdd-factory:story-writer | Trivial — 2 line edits in File Structure section |
| GAP-PHASE2-R02-4 | LOW | holdout-scenarios.md missing level: and version: frontmatter fields | vsdd-factory:story-writer | Trivial — add 2 frontmatter lines |

---

## §Trace v1.0

Consistency pass r02 created 2026-05-19T07:00:00Z by `consistency-validator`.
Inputs: Phase 2 story corpus at commit `0765d2e` (r01 remediation burst).
r01 closure rate: 10/11 (91%). 1 r01 gap partially closed (S-005 frontmatter fixed, STORY-INDEX Blocks column not updated).
4 gaps found: 1 HIGH (STORY-INDEX Blocks column dual-error), 2 MEDIUM (wave parallelism count; intra-story function name), 1 LOW (holdout-scenarios.md frontmatter fields).
No behavioral coverage gaps. No BC/VP/NFR/error code validity failures. No new dependency graph errors.
