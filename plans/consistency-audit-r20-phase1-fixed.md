---
document_type: consistency-report
level: ops
version: "1.0"
status: complete
producer: consistency-validator
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T06:00:00Z
round: 20
attempt: 15
policy: D-047-strict
traces_to: "PRD v1.12 (db7f50e) + VP v1.15 (3ec8ada) + arch v1.0.16 (6bb93e2) + manifest v1.1.12 (8005075) + STATE.md v5.16 (ae9b82f); post-F-R80 CRITICAL closure audit; T-45 dispatch"
input-hash: "[live-state]"
project: monocle
---

# Consistency Audit Round 20 — Phase 1 (D-047 Strict, Pass 1 Attempt 15)

## Audit Scope

Artifacts audited (all fresh-context, read from disk):

| Artifact | Version | Commit |
|----------|---------|--------|
| PRD | v1.12 | db7f50e |
| Verification Properties | v1.15 | 3ec8ada |
| Architecture (SS-daemon-lifecycle) | v1.0.16 | 6bb93e2 |
| Dependency Manifest | v1.1.12 | 8005075 |
| STATE.md | v5.16 | ae9b82f |

## Summary

| Category | Result | Findings |
|----------|--------|----------|
| F-R80 closure verification (all 7 findings + GAP-R19-001) | PARTIAL | 1 failure |
| Extension 13 compliance (machine-greppable evidence) | PARTIAL | 1 gap |
| Standard disciplines 1-16 | PASS | 0 |
| Agent-id routing existence | PASS | 0 |
| Trace audit-row integrity | PASS | 0 |
| Version pin propagation | PASS | 0 |
| ISO 8601 timestamp validity | PASS | 0 |
| BC-HOOK-022 normative-cite retirement | PARTIAL | 1 residual |
| Postcondition anchor accuracy (VP vs PRD) | PASS | 0 |

**Verdict: GAPS**

**Gap count: 3**

- GAP-R20-001 [MED] — VP §Purpose SHA still wrong (GAP-R19-001 reopen)
- GAP-R20-002 [MED] — VP §G-6 line 2199 BC-HOOK-022 residual normative framing (F-R80-2 incomplete closure)
- GAP-R20-003 [LOW] — Extension 13 evidence form gap: Extension 3 sweep uses `grep -cE` count-only output; Extension 13 requires `file:line + matched text` for full compliance

**D-047 counter: 0/3** (reset; GAPS found)

---

## F-R80 Closure Verification

### F-R80-1 [CRITICAL] — Extension 3 sweep table fabrication

**Status: CLOSED**

VP §Trace v1.15 contains the Extension 3 33-crate audit table at lines
2649-2684 with actual grep output code block at lines 2699-2764. The
table lists 33 crates matching SS-deps-pin-manifest.md v1.1.12 lines 35-66
+ 76. The 13 crates with versioned cites in the VP body all match the manifest
pin exactly: axum 0.8, tokio 1, prost 0.14, serde_json 1, serde_yaml_ng 0.10,
directories 6, tempfile 3, tracing 0.1, constant_time_eq ^0.3, nix 0.30,
serde 1, chrono 0.4, temp-env ^0.3.

The code block (lines 2699-2764) contains `grep -cE` commands with numeric
count outputs, narrative comments citing line numbers for some crates (tower
"at line 487", prost-build "at lines 1558, 1567..."), and the tower manifest
verification inline comment (`grep -nE "^\| tower " SS-deps-pin-manifest.md`
returns 0 hits). The fabricated axum 0.7 / prost 0.13 / rand 0.9 / russh 0.50
claims from v1.14 are retired. CLOSED.

**Extension 13 compliance gap noted as GAP-R20-003 (see below).**

### F-R80-2 [CRITICAL] — BC-HOOK-022 normative-cite in NFR-001/NFR-002

**Status: PARTIALLY CLOSED — GAP-R20-002 raised**

The targeted fixes were applied correctly:
- VP line 2183-2187 (NFR-001 sub-bullet): now reads "dropped per PRD §4 NFR-006
  (bounded mpsc channel with surfaced drop counter; upstream Claude Code timeout
  ceiling per gene-source BC-HOOK-022 is the reference data point for NFR-001's
  value, but NFR-006 is the Phase 1 monocle BC enforcing drop semantics)" —
  correctly framed as gene-source reference, not normative anchor.
- VP line 2193 (NFR-002 sub-bullet): now reads "Same drop semantics per NFR-006." —
  correct.

**RESIDUAL NOT FIXED:** VP §G-6 lines 2198-2202:

> "These contracts are correctness-relevant: NFR-001 and NFR-002 directly gate
> the BC-HOOK-022 drop-on-ceiling-exceed behavior (a wrongly-tuned ceiling
> produces real user-visible event loss)..."

This sentence uses BC-HOOK-022 as a behavioral anchor ("directly gate the
BC-HOOK-022 drop-on-ceiling-exceed behavior"), implying BC-HOOK-022 is a
monocle Phase 1 BC whose behavior NFR-001/002 gate. This contradicts §G-7
(same document) which explicitly identifies BC-HOOK-022 as a gene-source
identifier from `.factory/semport/any-context-lazyclaude/*`. The F-R80-2
closure fixed the normative-anchor sub-bullets at lines 2183-2193 but missed
this descriptive framing sentence two paragraphs later.

The extended Extension 11 grep pattern (`BC-HOOK-[0-9]+`) would detect this
hit, but the extended pattern was applied to lines 1-2500 pre-§Trace only.
Lines 2198-2202 are within body range (before §Trace at line 2527) and should
have been caught.

Raised as GAP-R20-002 [MED].

### F-R80-3 [CRITICAL] — Postcondition 9 anchor corrected to Postcondition 8

**Status: CLOSED**

VP line 707 now reads: "§BC-DAEMON-005 Postcondition 8 + EC-052 + arch v1.0.16
§Start Sequence" — confirmed correct. The §Trace v1.15 Change 3 block lists all
6 correction sites with before/after and PRD grep evidence (code block at lines
2804-2813 showing PRD line 342 and 380 with actual content). CLOSED.

### F-R80-4 [HIGH] — PG-4 sweep fabricated Postcondition 9 PASS

**Status: CLOSED**

PG-4 audit row rewritten with actual PRD grep transcript citing lines 342 and
380. CLOSED.

### F-R80-5 [HIGH] — Invalid ISO 8601 timestamps

**Status: CLOSED**

VP frontmatter (line 9): `timestamp: 2026-05-16T01:30:00Z` — valid. The
"architect end-of-day notation" fabricated convention is retired. §References
intro timestamp at line 2363: `2026-05-16T01:30:00Z` — valid. CLOSED.

### F-R80-6 [MED] — Extension 11 grep pattern under-scoped

**Status: CLOSED**

Extension 11 codification (VP §Trace Extension 11 section) now includes gene-source
BC-id prefixes: `BC-HOOK-[0-9]+|BC-PERM-[0-9]+|BC-CTX-[0-9]+`. The §Trace v1.15
Change 5 block documents the updated canonical grep pattern at lines 2862-2866.
Application of the extended pattern this burst flagged the BC-HOOK-022 leak at
NFR-001 which was closed by F-R80-2. CLOSED.

**Note:** The residual GAP-R20-002 (§G-6 line 2199) was NOT flagged by Extension
11's post-F-R80-2 re-application because lines 2199-2202 appear to have been
scanned but the sentence framing escaped the BC-id prefix pattern in context
(the hit is `BC-HOOK-022` inside a behavioral description, not an explicit
normative anchor). The post-edit Extension 11 scan result reported at §Trace v1.15
lines 2873-2884 shows only two remaining BC-HOOK-022 hits:
(1) §G-7 gene-source framing — category (b) CORRECT
(2) NFR-001 line 2179-2180 gene-source reference data point — category (b) CORRECT

This scan result appears to have missed the §G-6 line 2199 hit. Either:
(a) line 2199 is outside the `awk 'NR<2501'` scan window used by Extension 11
    (lines 2199 IS within 1-2500), or
(b) the scan was applied but the $G-6 framing sentence was mistakenly classified
    as category (b) in the scan output rather than being flagged.

The line 2199 hit (`BC-HOOK-022 drop-on-ceiling-exceed behavior`) is present at
line 2199, within the NR<2501 window, and uses BC-HOOK-022 as a monocle behavior
anchor in a way that a reader would interpret as normative. This is the definition
of an Extension 11 category (c) or borderline (b→c) finding. GAP-R20-002 stands.

### F-R80-7 [MED] — 3 additional Postcondition 9 propagation sites

**Status: CLOSED**

All 3 sites confirmed corrected per §Trace v1.15 Change 3:
- Line 2057 (§Coverage Matrix footer): "Postcondition 8 lifting..."
- Line 2370 (§References item 1): "Postcondition 8 lifting..."
- Lines 2722-2724 (§Trace v1.14 Change 5): "Postcondition 8 lifting..."
CLOSED.

### GAP-R19-001 [LOW] — VP §Purpose stale SHA

**Status: NOT CLOSED — GAP-R20-001 raised**

STATE.md v5.16 (entry 19) states: "GAP-R19-001 LOW: VP §Purpose stale SHA fixed."
VP §References item 1 (line 2365) correctly shows: "`.factory/specs/prd.md` v1.12
(commit db7f50e)." §Trace v1.15 frontmatter `traces_to` claims GAP-R19-001 was
closed.

**CONFIRMED NOT FIXED:** VP §Purpose at line 35 reads:

> "the 22 Behavioral Contracts (BCs) formalized in the Phase 1 PRD v1.12 (commit
> 1f90b64)"

Git log confirms:
- `1f90b64` = PRD v1.11 (GAP-R16-001 frontmatter-only housekeeping fix)
- `db7f50e` = PRD v1.12 (F-R79-1 RTM Test File + F-R79-3 BC-DAEMON-005 Postcondition lift)

The §Purpose first paragraph SHA is wrong. `1f90b64` is PRD v1.11. The correct
SHA for PRD v1.12 is `db7f50e`. This is the identical class of finding as
R13-001 (stale SHA in §Purpose). The closure claimed in the F-R80 burst
(frontmatter `traces_to` + §References item 1 + §Coverage Matrix footer) updated
downstream citation sites but left the §Purpose opening sentence uncorrected.

This is a GAP-R19-001 reopen — the finding was not actually closed.
Raised as GAP-R20-001 [MED].

**Note:** The §Purpose stale SHA was also the pre-fix state in cons R19 before
the F-R80 burst; the f-R80 burst closure narrative erroneously claimed it was
fixed without applying the actual §Purpose line 35 correction.

---

## Extension 13 Compliance Audit

**Extension 13 mandate (per D-070):** "every audit-row claim MUST be backed by
code-block grep transcript (file:line + matched text), NOT asserted PASS verdicts."

### Extension 3 sweep (F-R80-1 closure)

The §Trace v1.15 Extension 3 sweep code block at lines 2699-2764 uses:
- `grep -cE` — count-only output (e.g., `5`, `2`, `0`)
- Narrative comments with line-number references (e.g., "tower hits at line 487")
- One inline `grep -nE` command for manifest cross-check of tower

Extension 13 requires "code blocks with file:line + matched text." The count-only
form (`grep -cE` returning `5`) does not include `file:line + matched text`. The
counts are machine-verifiable (any reviewer can re-run the same `grep -cE` command)
but do not show the actual matched line content.

**Assessment:** Partial compliance. The crate-count sweep is mechanically reproducible
from the documented commands, which satisfies the spirit of Extension 13 (replaces
self-attested "PASS" with reproducible commands). However, strict reading of "file:line
+ matched text" requires `grep -n` or `grep -nE` output showing the actual line
content for each hit, not just count.

The F-R80-3/F-R80-4 PRD grep block (lines 2804-2813) fully satisfies Extension 13:

```
342:8. If `<runtime_dir>` does not exist at start, daemon creates
    it with mode `0o700` (owner-only access)...
380:- Postcondition 8 (runtime-dir mode 0o700): verified by
    VP-DAEMON-005 Post-condition 9 and probe 5.e...
```

These show `file:line:matched text` as Extension 13 requires.

**Assessment:** Extension 3 sweep satisfies the reproducibility intent of Extension 13
(commands are documented and re-runnable) but uses count-only output rather than
`file:line + matched text`. For the critical crates where count > 0 and PASS verdict
depends on ALL hits matching the pin (e.g., axum with 5 hits all claimed to be axum 0.8),
count-only output cannot be independently verified without running the commands. The
commands ARE documented, so a reviewer CAN verify by re-running — this is materially
better than the v1.14 self-attested PASS verdicts that Extension 13 was authored to
prevent.

Raised as GAP-R20-003 [LOW] — process gap; does not constitute a fabrication-class
failure; does NOT block convergence but should be noted for future sweeps.

---

## Standard Disciplines Assessment

### Discipline 1 — L-F-R63 Extension 1 (version pin propagation)

PRD frontmatter `traces_to` cites SS-deps-pin-manifest.md v1.1.12. Manifest
frontmatter version: "1.1.12". Consistent. PASS.

### Discipline 2 — L-F-R63 Extension 2 (intra-block consistency)

VP §Trace v1.15 PG-2 count coherence (lines 2897-2907): 22 VPs unchanged;
mechanism distribution unchanged; Coverage Matrix 22 rows; Open Verification Gaps 7.
The F-R80 closure is VP-only with zero semantic VP changes. PASS.

### Discipline 3 — L-F-R63 Extension 3 (deps-pin-manifest enforcement sweep)

33-crate audit table present (lines 2649-2684). All 13 versioned cites match manifest.
See Extension 13 gap note (GAP-R20-003) for evidence form. PASS on substance.

### Extension 3 Enforcement (Extension 3-Enforcement discipline)

The sweep ran post-F-R80-1/2/3/7 edits (documented in §Trace v1.15 Change 1 sweep
mechanic description). Applied to `/tmp/vp-body-2500-post.txt` (VP body lines 1-2500
post-edit). PASS.

### Discipline 4 — L-F-R63 Extension 4 (ellipsis placeholder patrol)

VP does not introduce any `<X>` or `"..."` ellipsis placeholders in normative content.
Arch v1.0.16 (UNCHANGED this burst) had F-R74-1 closure applying this discipline. PASS.

### Extension 4-expansion

Covers `"..."` 3-dot ellipsis pattern. No new instances in VP v1.15 burst. PASS.

### Discipline 5 — L-F-R63 Extension 5 (VP-coverage-vs-BC-EC-security-property sweep)

VP-DAEMON-005 contains Post-condition 9 (0o700 runtime-dir mode verification,
probe 5.e, counter-example 10, mutation rationale) per F-R75-1 closure.
BC-DAEMON-005 Postcondition 8 (PRD line 342) is the BC anchor; the VP anchor
citation is corrected to "Postcondition 8 + EC-052" per F-R80-3 closure. PASS.

### Discipline 6 — L-F-R63 Extension 6 (rationale-prose-vs-NFR-canonical-contract sweep)

VP probe 5.c (Windows secondary target rationale) and §G-6 NFR-001/002 description
correctly scope to macOS + Linux primary CI. No Windows-primary claims in VP v1.15. PASS.

### Discipline 7 — L-F-R63 Extension 7 (exhaustive crate-prefix grep against arch)

VP §Trace v1.15 notes "chrono:: confirmed as only Extension 7 finding (arch v1.0.16
unchanged)" per the extension 7 re-application in the burst context. PASS.

### Discipline 8 — L-F-R63 Extension 8 (NFR-to-VP exhaustive coverage audit)

VP §G-6 covers NFR-001/002/003 (deferred Phase 3 with concrete future-attachment).
VP §G-7 covers NFR-006 (deferred Phase 3). NFR-004/005/007/008/009/010/011 verified
per earlier bursts (Extensions 8 preemption in v1.12). PASS.

### Discipline 9 — L-F-R63 Extension 9 (Coverage Matrix footer narrative consistency)

§Coverage Matrix footer (lines 2049-2067 area) updated with "F-R79-3 new BC-DAEMON-005
Postcondition 8 lifting..." per F-R80-7 correction at line 2057. PASS.

### Discipline 10 — L-F-R63 Extension 10 (PRD §3 §Verification → §7 RTM propagation)

PRD v1.12 §7 RTM BC-DAEMON-004 Test File column lists both `graceful_shutdown.rs`
and `daemon_lifecycle.rs` per F-R79-1 closure (PRD unchanged at v1.12 this burst). PASS.

### Discipline 11 — L-F-R63 Extension 11 (BC-vs-Brief JC-closure alignment audit)

Extension 11 grep pattern now includes `BC-HOOK-[0-9]+|BC-PERM-[0-9]+|BC-CTX-[0-9]+`.
Two remaining BC-HOOK-022 hits both classified as category (b) in §Trace v1.15
post-edit scan. GAP-R20-002 notes a potential missed hit at line 2199 (residual
normative framing) — see F-R80-2 partial closure above. PARTIAL — GAP-R20-002.

### Discipline 12 — L-F-R63 Extension 12 (VP-to-BC §Postcondition anchor audit)

VP normative §Post-conditions that carry BC anchor citations now reference
"Postcondition 8" per F-R80-3/F-R80-7 corrections at all 6 sites. PASS.

### Discipline 13 — L-F-R63 Extension 13 (machine-greppable evidence requirement)

DEFINED: every audit-row claim must be backed by code-block grep transcript
with file:line + matched text. APPLIED: §Trace v1.15 Change 1 provides code-block
with `grep -cE` commands (count-only) and inline comments citing line numbers.
§Trace v1.15 Change 3 (PRD grep for Postcondition 8) fully satisfies Extension 13
with `342:8. If...` and `380:- Postcondition 8...` output format.

Gap: Extension 3 sweep uses count-only. See GAP-R20-003. PARTIAL.

### Agent-id routing existence

VP §Scope item 2, §G-6, §G-7 all cite `vsdd-factory:performance-engineer` (canonical
agent ID per CLAUDE.md Agent Routing Table). No `vsdd-factory:perf-check` (retired)
in VP normative-current content. §Trace Extension discipline text at lines 4449-4459
documents the distinction. PASS.

### Trace audit-row integrity

§Trace v1.15 audit rows carry documented evidence (code blocks, before/after
verbatim, PRD line citations). No self-attested PASS verdicts for the F-R80 closure
items. Partial gap (count-only for Extension 3) addressed in GAP-R20-003. PASS on
non-count items.

---

## Cross-Document Pin Propagation Checks

### PRD v1.12 → VP v1.15 pin currency

VP frontmatter `traces_to` cites "PRD v1.12 — current canonical BC source (commit
db7f50e unchanged this burst)". VP §References item 1 cites "v1.12 (commit db7f50e)".
VP §Coverage Matrix footer cites "PRD v1.12". The bulk of VP body normative
citations (~60 sites per v1.14 propagation) cite PRD v1.12. PASS on propagation.

**Exception:** VP §Purpose line 35 cites "PRD v1.12 (commit 1f90b64)" — wrong SHA.
This is GAP-R20-001. Stale at §Purpose only.

### Arch v1.0.16 pin currency

VP §Scope cites "SS-daemon-lifecycle.md v1.0.16 (BC-RING-001, BC-AUTH-001, ...)".
VP §References cites arch v1.0.16 at commit 6bb93e2. Manifest frontmatter
`traces_to` cites "SS-daemon-lifecycle.md v1.0.16 commit 6bb93e2 unchanged". PASS.

### Manifest v1.1.12 pin currency

VP §References item 5 cites manifest v1.1.12 (commit 8005075). PRD frontmatter
`traces_to` cites "SS-deps-pin-manifest.md v1.1.12". Consistent. PASS.

---

## ISO 8601 Timestamp Validity

VP frontmatter timestamp: `2026-05-16T01:30:00Z` — hours 01, valid. (Was
`2026-05-15T25:30:00Z` in v1.14; F-R80-5 fixed.) PASS.

VP §References intro line 2363: `2026-05-16T01:30:00Z` — valid. PASS.

PRD frontmatter timestamp: `2026-05-15T23:30:00Z` — valid. PASS.

Manifest frontmatter timestamp: `2026-05-15T23:00:00Z` — valid. PASS.

---

## BC-HOOK-022 Normative-Cite Audit

Applying the updated Extension 11 pattern to VP body lines 1-2500:

Sites found:
1. **Line 2185** — "upstream Claude Code timeout ceiling per gene-source BC-HOOK-022
   is the reference data point for NFR-001's value" — category (b) gene-source
   reference with explicit framing. CORRECT.
2. **Line 2199** — "NFR-001 and NFR-002 directly gate the BC-HOOK-022
   drop-on-ceiling-exceed behavior" — uses BC-HOOK-022 as a Phase 1 behavioral
   anchor. Category (b) framing absent at THIS site; the broader §G-6 section
   frames BC-HOOK-022 as gene-source, but this sentence assigns normative gating
   semantics. This is the residual identified in GAP-R20-002.
3. **Lines 2247-2252** (§G-7) — "(a) BC-HOOK-022 is a gene-source identifier from
   .factory/semport/any-context-lazyclaude/*..." — explicit gene-source framing,
   category (b). CORRECT.

Result: 2 PASS (lines 2185, 2247-2252), 1 residual (line 2199). GAP-R20-002 confirmed.

---

## Findings

### GAP-R20-001 [MED] — VP §Purpose SHA still wrong (GAP-R19-001 reopen)

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md`
**Line:** 35
**Evidence:**
```
35: the 22 Behavioral Contracts (BCs) formalized in the Phase 1 PRD v1.12 (commit
36: 1f90b64) and pre-staged across the Phase 1 architecture artifacts.
```
Git log evidence:
- `1f90b64` = `feat(prd): v1.11 — GAP-R16-001 manifest pin housekeeping (frontmatter only)`
- `db7f50e` = `feat(prd): v1.12 — F-R79-1 RTM Test File + F-R79-3 BC-DAEMON-005 0o700 Postcondition lift`

**Impact:** §Purpose presents the wrong commit for PRD v1.12. Any reviewer using
the §Purpose SHA to checkout the authoritative BC source will get PRD v1.11
(frontmatter-only, no BC content changes) instead of PRD v1.12 (RTM Test File
column propagation + BC-DAEMON-005 Postcondition 8 — substantive changes). The
§References item 1 (line 2365) and §Coverage Matrix footer correctly cite db7f50e.
Only §Purpose is wrong.

**Routing:** formal-verifier — change `1f90b64` → `db7f50e` at VP §Purpose line 35.
Single-line fix; no semantic verification-property change required.

**History:** This is the THIRD recurrence of this exact §Purpose SHA class:
R13-001 (v1.8→v1.9), GAP-R19-001 (v1.11→v1.12, claimed closed but not applied),
GAP-R20-001 (v1.12 uncorrected). The partial-propagation pattern (§References
updated but §Purpose overlooked) is now repeatable. Recommend adding §Purpose
to Extension 10/11 sweep scope or a dedicated Extension 14 (§Purpose SHA
currency check — verify §Purpose SHA matches §References current-pointer SHA).

---

### GAP-R20-002 [MED] — VP §G-6 residual BC-HOOK-022 normative framing (F-R80-2 incomplete closure)

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md`
**Lines:** 2198-2202
**Evidence:**
```
2198: These contracts are correctness-relevant: NFR-001 and NFR-002 directly
2199: gate the BC-HOOK-022 drop-on-ceiling-exceed behavior (a wrongly-tuned
2200: ceiling produces real user-visible event loss), and NFR-003 gates the
2201: permission-overlay UX SLO (a >100 ms first-paint produces user-perceived
2202: lag on every permission decision).
```

**Impact:** The phrase "directly gate the BC-HOOK-022 drop-on-ceiling-exceed
behavior" assigns monocle Phase 1 behavioral semantics to BC-HOOK-022 — as if
BC-HOOK-022 is a monocle Phase 1 BC whose drop behavior the NFR latency ceilings
govern. This contradicts §G-7 (same document, lines 2247-2252) which identifies
BC-HOOK-022 as a gene-source identifier. A reader of §G-6 in isolation would infer
that BC-HOOK-022 is a Phase 1 monocle BC enforcing drop semantics — exactly the
misreading that F-R80-2 was authored to prevent.

The F-R80-2 fix was targeted at NFR-001 sub-bullet (line 2183) and NFR-002 line
2186, but the summary framing sentence (lines 2198-2202) was not updated. This is
a partial-fix propagation gap of the same class as F-R80-7 (3 additional
Postcondition 9 sites missed by F-R80-3).

**Routing:** formal-verifier — rewrite lines 2198-2202 to frame BC-HOOK-022 as
the gene-source reference rather than a monocle Phase 1 BC being gated. Suggested
replacement: "These contracts are correctness-relevant: NFR-001 and NFR-002
directly gate the drop-on-ceiling-exceed behavior specified by NFR-006 (the Phase
1 monocle BC enforcing bounded mpsc channel + surfaced drop counter semantics;
gene-source reference ceiling per BC-HOOK-022 upstream timeout values), and NFR-003
gates the permission-overlay UX SLO..."

---

### GAP-R20-003 [LOW] — Extension 13 evidence form gap (count-only vs file:line+text)

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md`
**Lines:** 2699-2764 (Extension 3 sweep code block)

**Issue:** Extension 13 mandates "code blocks with file:line + matched text."
The Extension 3 sweep code block uses `grep -cE` (count-only output). For crates
with versioned-cite count > 0 (axum=5, tokio=3, prost=2, serde_json=3, serde=10,
chrono=9, etc.), the count-only output cannot independently confirm that ALL
matching hits refer to the correct manifest pin without re-running the commands.

The commands ARE documented and re-runnable, which is substantially better than
the v1.14 self-attested "12 PASS verbatim" claim that Extension 13 was authored to
prevent. This is a process-refinement gap, not a fabrication-class failure.

**The PRD Postcondition grep block (lines 2804-2813) correctly satisfies Extension 13**
with `file:line:matched-text` format.

**Routing:** process-gap — no immediate fix required to unblock D-047. Recommend
that the next Extension 3 sweep application (whether in this VP or in State-manager
audit protocols) use `grep -nE` (line-numbered output showing matched text) instead
of `grep -cE` (count-only). This would allow a reviewer to read the actual VP body
lines matching each crate-pin pattern without re-running commands. Codify as
Extension 13 clarification at next formal-verifier pass.

**Note:** Does NOT constitute a META-class fabrication recurrence. The counts in the
table are externally verifiable from the documented commands. Severity: LOW.

---

## Convergence Trajectory

20 attempts: 13→5→1→4→0→2→1→0→0→3→5→3→0→3→2→2→6→2→3→7(CRIT spike)→GAPS(3)

Round 20 (this audit): GAPS — 3 findings (2 MED, 1 LOW). Counter RESET to 0/3.

---

## Closure Summary

| Finding | Source | Closed | Status |
|---------|--------|--------|--------|
| F-R80-1 CRIT Extension 3 sweep fabrication | adversary R80 | VP v1.15 | CLOSED |
| F-R80-2 CRIT BC-HOOK-022 normative-cite NFR-001/002 | adversary R80 | VP v1.15 (partial) | PARTIAL — GAP-R20-002 |
| F-R80-3 CRIT Postcondition 9 anchor (3 primary sites) | adversary R80 | VP v1.15 | CLOSED |
| F-R80-4 HIGH PG-4 fabricated Postcondition 9 PASS | adversary R80 | VP v1.15 | CLOSED |
| F-R80-5 HIGH ISO 8601 invalid timestamps | adversary R80 | VP v1.15 | CLOSED |
| F-R80-6 MED Extension 11 under-scoped | adversary R80 | VP v1.15 | CLOSED |
| F-R80-7 MED 3 additional Postcondition 9 sites | adversary R80 | VP v1.15 | CLOSED |
| GAP-R19-001 LOW VP §Purpose stale SHA | cons R19 | claimed VP v1.15 | NOT CLOSED — GAP-R20-001 |

---

## Routing for Gaps

| Gap | Severity | Route | Fix Scope |
|-----|----------|-------|-----------|
| GAP-R20-001: §Purpose SHA 1f90b64 → db7f50e | MED | formal-verifier | VP line 35 single-char substitution |
| GAP-R20-002: §G-6 line 2199 BC-HOOK-022 framing | MED | formal-verifier | VP lines 2198-2202 prose rewrite |
| GAP-R20-003: Extension 3 count-only vs file:line+text | LOW | process-gap | Codify Extension 13 clarification at next burst |

**D-047 gate result: FAIL — 2 MED findings block CLEAN. Counter reset to 0/3.**
**Dispatch: F-R20 fix-burst (formal-verifier: 3 VP fixes); adversary R81 pass 1 attempt 15 (pending fix-burst completion).**
