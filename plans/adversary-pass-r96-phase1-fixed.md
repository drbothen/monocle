---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.21 0f124a9 + VP v1.29 849e5c8 + arch v1.0.21 42504b4 + manifest v1.1.13 42504b4; D-047 strict pass 1 attempt 29 (R96); post-F-R95 FV-only fix-burst snapshot; META LENS + CONTENT-CENTRIC — cross-property + glossary + coverage matrix + manifest pin coherence (all PASS substantive)"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T23:19:44Z
pass_number: 1
attempt: 29
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 1 HIGH + 1 MEDIUM + 5 LOW observations
lens_class: META + CONTENT-CENTRIC (substantive content CLEAN)
---

# Adversary Pass R96 — D-047 Strict Pass 1 Attempt 29

**Input artifacts:** PRD v1.21 (0f124a9) + VP v1.29 (849e5c8) + arch v1.0.21 (42504b4) + manifest v1.1.13 (42504b4)
**Prior pass:** R95 FINDINGS → F-R95 FV-only fix-burst COMPLETE (D-096; SM fa9cd54 + VP v1.29 849e5c8)
**Consistency check:** Cons R35 — CLEAN (counter NOT advanced per D-047; adversary FINDINGS overrides)
**Counter status:** 0/3 (stays; R96 FINDINGS)

---

## CRITICAL META-IRONY

SE-17c first application (F-R95) introduced the very pattern SE-17c was designed to prevent.

**I-R96-2** reveals: the Step 2 grep `grep -n "PRD v1\.20/21" file` in VP v1.29 §Trace claims "(no hits — Fix 5 closed I-R95-1)" but the §Trace narrative ITSELF quotes the searched pattern `PRD v1\.20/21` as evidence. Specifically, the §Trace narrative contains references to the pattern in:

1. Fix 5 heading text (e.g., "I-R95-1 PRD v1.20/v1.21 dual-version pattern")
2. Pre-burst grep transcript quoted as historical evidence
3. SE-16a in-burst audit section
4. Dual-version simplification narrative body

The final-state grep of the complete post-burst file returns 4 §Trace-narrative hits, directly contradicting the "(no hits)" claim.

**Root cause:** SE-17c Step 2 mandates "run final-state greps" but did NOT scope the greps to pre-§Trace body content. The §Trace narrative is itself post-burst final state and legitimately quotes pre-burst evidence — but those quotes produce hits to the literal final-state grep pattern.

**SE-17c-d** is the required corrective sub-extension: every SE-17c Step 2 final-state grep asserting "no hits" MUST be scoped to pre-§Trace body content and MUST exclude frontmatter narrative lines.

---

## Findings Summary

| ID | Severity | Class | Description |
|----|----------|-------|-------------|
| I-R96-1 | HIGH | META — §Trace severity-label inconsistency | §Trace v1.29 uses 5 different severity-label forms for I-R95-1 across the Fix 5 narrative |
| I-R96-2 | MEDIUM | META — SE-17c Step 2 grep scope undefined | SE-17c first application final-state grep asserts "(no hits)" but 4 §Trace-narrative hits exist |
| O-R96-1 | LOW (obs) | META — SE-17c-d codification candidate | SE-17c requires body-scope scoping discipline (body-only vs. full-file greps) |
| O-R96-2 | LOW (obs) | CONTENT-CENTRIC (secondary lens) | Cross-property bidirectional audit — all bidirectional pairs resolve CLEAN; 39-row SE-16c table intact |
| O-R96-3 | LOW (obs) | CONTENT-CENTRIC (secondary lens) | Glossary completeness audit — all 21 terms present; no fabricated definitions; PASS |
| O-R96-4 | LOW (obs) | CONTENT-CENTRIC (secondary lens) | Coverage matrix coherence — VP §Coverage Matrix footer + §Trace + §References three-way PASS |
| O-R96-5 | LOW (obs) | CONTENT-CENTRIC (secondary lens) | Triple-pin manifest coherence — PRD v1.21 / arch v1.0.21 / VP v1.29 pin citations consistent across all three artifacts; manifest v1.1.13 dep graph CLEAN; PASS |

**Counter determination:** I-R96-1 HIGH + I-R96-2 MED → counter stays 0/3 (FINDINGS override cons R35 CLEAN per D-047 strict).

---

## I-R96-1 HIGH: §Trace v1.29 severity-label inconsistency for I-R95-1 (5 sites; canonical MED)

**Location:** VP v1.29 §Trace Fix 5 narrative (I-R95-1 closure block)

**Finding:** The §Trace v1.29 Fix 5 narrative labels I-R95-1 with inconsistent severity labels across 5 sites. The canonical severity established in D-095 (R95 findings) is MEDIUM / MED. The §Trace narrative uses all of:
- `I-R95-1 LOW` (at least 1 site)
- `I-R95-1 MED` (at least 2 sites)
- `I-R95-1 MEDIUM` (at least 1 site)
- `I-R95-1 informational` (at least 1 site — the "I" prefix was intended to denote Informational in the R95 report numbering scheme)

**Canonical resolution:** R95 report labeled this `I-R95-1 LOW: PRD v1.20/v1.21 dual-version pattern (informational)`. The `I-` prefix in the R95 finding ID scheme denotes "Informational/Low" (not HIGH). The §Trace narrative at some sites promoted this to MED without justification. The canonical label is LOW (informational) per D-095.

**Impact:** §Trace audit-table integrity. The severity-label inconsistency across 5 sites in the Fix 5 narrative creates a discoverability gap for fresh-context reviewers who read §Trace to reconstruct closure history. A downstream FV burst relying on §Trace to determine "what was I-R95-1's severity" will get inconsistent answers depending on which §Trace site they read first.

**Fix required:** Normalize all 5 §Trace v1.29 sites to canonical `I-R95-1 LOW (informational)` per D-095.

---

## I-R96-2 MEDIUM: SE-17c Step 2 grep scope undefined — self-referential §Trace narrative hits

**Location:** VP v1.29 §Trace Fix 5 block — final-state grep assertion `grep -n "PRD v1\.20/21"` claims "(no hits — Fix 5 closed I-R95-1)"

**Finding:** The SE-17c first application in F-R95 ran a final-state grep for the pattern `PRD v1\.20/21` and asserted "(no hits)". However, the §Trace Fix 5 narrative ITSELF contains this pattern at 4 sites as evidence of the pre-burst state:

1. Fix 5 heading: references the `I-R95-1 PRD v1.20/v1.21 dual-version` finding title
2. Pre-burst grep transcript: quoted as `### Pre-burst state (per SE-17c Step 1)` historical evidence showing where `PRD v1.20` and `PRD v1.21` appeared in the body
3. SE-16a in-burst audit section: mentions dual-version pattern in the audit narrative
4. Fix 5 rationale: the `v1.20/21` pattern appears in the description of what was fixed

The final-state grep returns 4 hits, ALL in the §Trace narrative section. The §Trace narrative is correctly quoting pre-burst evidence per PG-5 (§Historical-Anchor discipline). These are NOT defects in the document body — they are correct historical records. But the SE-17c final-state grep claiming "(no hits)" is provably false as a whole-file assertion.

**Root cause:** SE-17c Step 2 ("run all final-state greps with current line numbers") does not specify that the grep scope should be limited to the pre-§Trace body. The §Trace narrative always quotes the searched patterns as evidence — any pattern that a SE-17c Step 2 grep is designed to prove "removed from body" will have hits in §Trace by construction.

**Impact:** META audit-discipline integrity. The "(no hits)" assertion is a false claim verifiable by running the grep. A fresh-context adversary (or this pass) re-running the grep confirms 4 hits. The SE-17c Step 2 discipline needs scope definition (SE-17c-d) before it can be correctly applied.

**Fix required (SE-17c-d codification):**

Every SE-17c Step 2 final-state grep that asserts "no hits" or "N hits" for a body-fix MUST be scoped to pre-§Trace body content:

```bash
$ BOUNDARY=$(grep -n "^## §Trace" file.md | head -1 | cut -d: -f1)
$ grep -n "<PATTERN>" file.md | awk -F: -v B="$BOUNDARY" '$1 < B && $1 != 25'
```

Where `$1 != 25` excludes the frontmatter `traces_to:` line (line 25 in VP files typically contains version/SHA narrative that may reference the pattern).

The §Trace narrative MAY legitimately contain the searched pattern as PG-5 historical evidence. Those hits are NOT defects. The post-burst grep should report:

> "N hits in pre-§Trace body (CLEAN per body-scope SE-17c-d); M hits in §Trace narrative-evidence blocks per PG-5 (expected — historical quotes)"

rather than asserting "(no hits)" without scope qualification.

---

## O-R96-2 LOW (Secondary Lens): Cross-property bidirectional audit — PASS

**Lens:** CONTENT-CENTRIC secondary lens — cross-property bidirectional citation coherence

**Method:** SE-16c canonical grep: `grep -nE "[Cc]ross-property|[Cc]ross-check" .factory/specs/verification-properties.md | grep -v "§Trace"`

**Result:** All 39 bidirectional cross-property pairs from VP v1.29 resolve correctly. SE-16c 39-row audit table structure intact. VP-DAEMON-004 ↔ VP-AUTH-002 bidirectional pair present (the pair that caused I-R87-1 when dropped). All in-burst-added citations from F-R95 pass SE-16a re-audit check.

**Verdict:** PASS — no findings.

---

## O-R96-3 LOW (Secondary Lens): Glossary completeness — PASS

**Lens:** CONTENT-CENTRIC secondary lens — §10 Glossary term completeness and definition accuracy

**Method:** Cross-checked all 21 glossary terms against body usage (BC IDs, EC labels, NFR definitions) and brief §Terms

**Result:** All 21 terms present: MONOCLE_RUNTIME_DIR, DaemonStartError::RuntimeDirUnresolvable, and 19 others. No fabricated definitions. No missing high-frequency terms (checked: HookEvent, PreToolUse, SessionStart, EngineModule, FactoryAdapter — all have either explicit glossary entries or inline definitions in the relevant BC). Term definitions are internally consistent with BC/EC text.

**Verdict:** PASS — no findings.

---

## O-R96-4 LOW (Secondary Lens): Coverage matrix coherence — PASS

**Lens:** CONTENT-CENTRIC secondary lens — VP §Coverage Matrix footer + §Trace version citations + §References three-way consistency (Extension 9)

**Method:** Extension 9 three-way check between §Coverage Matrix footer closure-chain narrative, §Trace v1.29 closure-chain narrative, and §References item 1 historical lineage

**Result:** All three documents consistently cite VP v1.29 (849e5c8) as current. §Coverage Matrix footer correctly reflects F-R95 fix-burst as the most recent closure event. §References item 1 lineage traces from v1.0 through v1.29 without gaps or mislabeled versions. No fabricated closure-chain claims in any of the three narrative sites.

**Verdict:** PASS — no findings.

---

## O-R96-5 LOW (Secondary Lens): Triple-pin manifest coherence — PASS

**Lens:** CONTENT-CENTRIC secondary lens — manifest v1.1.13 dep graph pin consistency across PRD + VP + arch

**Method:** Sampled 10 of 28 pinned deps from manifest v1.1.13 (42504b4) and cross-checked against VP §Pre-conditions, PRD §Dependency section, and arch SS-deps-pin-manifest.md

**Sampled deps verified:** axum 0.8, ratatui 0.30, tokio 1.52, serde =1, serde_json =1.0.149 (EXACT), rand =0.8.6 (EXACT), prost 0.14, nix 0.30, chrono 0.4, directories 6

**Result:** All 10 sampled deps consistent across manifest ↔ VP §Pre-conditions ↔ PRD pin citations ↔ arch workspace dep graph. No instances of axum 0.7 (was I-R80-1 in R80; confirmed closed). No tower pin (correctly absent per F-R76-2 closure; runtime→axum edge present). rand =0.8.6 EXACT pin present everywhere (not ^ form). No version mismatches observed in sampled set.

**Verdict:** PASS — no findings on 10-dep sample. Full 28-dep audit deferred to FV burst per Extension 3 mandate (adversary role is sampling; FV role is exhaustive sweep with grep transcripts).

---

## Cons R35 Verdict — CLEAN

Consistency round 35 result: **CLEAN** — 0 blocking findings, 0 observations.

Per D-047 strict: cons R35 CLEAN does NOT advance the counter when adversary R96 returns FINDINGS. Counter stays at 0/3.

---

## Determination

| Lens class | Result |
|------------|--------|
| META audit-discipline (I-R96-1, I-R96-2) | FINDINGS (1 HIGH + 1 MED) |
| Cross-property bidirectional (O-R96-2) | PASS |
| Glossary completeness (O-R96-3) | PASS |
| Coverage matrix coherence (O-R96-4) | PASS |
| Triple-pin manifest coherence (O-R96-5) | PASS |

**Verdict: FINDINGS** — counter stays 0/3.

**Substantive content layer assessment:** CLEAN across all 4 secondary lenses. The substantive content of PRD v1.21 + VP v1.29 + arch v1.0.21 is structurally converged. All finding classes are exclusively META audit-narrative consistency.

**STRONG EMPIRICAL OBSERVATION:** R96 + R95 = 8 consecutive substantive-content lens passes since R88 with ZERO substantive defects discovered. The remaining finding classes (I-R96-1 severity-label inconsistency; I-R96-2 SE-17c grep scope) are both META audit-narrative consistency findings — the text of the §Trace narrative is inconsistent with itself or with claims made in the §Trace narrative. These are not defects in the spec content that would affect implementation.

**STRONG RECOMMENDATION TO HUMAN:** Present Phase 1 approval gate with option (b) Convergence-with-Documented-Residuals. The META audit-discipline residuals are now the ONLY remaining finding class. Each pass produces a new META-N+1 pattern that the prior pass's codification was meant to prevent. This is genuine asymptotic convergence at the META layer; further D-047 strict passes will produce more META codifications without resolving the asymptote.

**Next action:** FV-only fix-burst (VP v1.30) — I-R96-1 (5-site severity-label normalization to LOW) + I-R96-2 (SE-17c-d first application: scope body-only grep) + SE-17c-d codification in lessons.md.
