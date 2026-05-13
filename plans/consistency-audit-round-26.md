---
document_type: consistency-report
level: ops
version: "1.0"
producer: consistency-validator
phase: pre-phase-1-final-gate
timestamp: 2026-05-13T23:30:00Z
input-hash: "[live-state]"
traces_to: "round-25 fix burst commits 436d4d3 + f287592 + 3b90235 + 11185a1 + 6f43b6b"
project: monocle
---

# Consistency Audit — Round 26

**Scope:** Post-round-25-fix-burst. Validates architecture docs + brief + vision + STATE.md for
version pointer consistency, temp-env pin reference consistency, test convention rule visibility,
BC enumeration completeness, BC count reconciliation, brief ratification semantics, vision-authority
framing, cross-reference anchors, production-grade compliance, STATE.md integrity, and
frontmatter input-hash drift.

**Verdict: MEDIUM — 3 findings (0 CRITICAL, 3 MEDIUM, 0 LOW after applying production-grade lens).**

---

## Summary Table

| Check | Result | Severity |
|-------|--------|----------|
| 1. Version pointer consistency (STATE.md, brief, cross-refs) | FAIL | MEDIUM |
| 2. temp-env pin reference consistency | FAIL | MEDIUM |
| 3. Test convention rule visibility | PASS | — |
| 4. BC-ENGINE-002-ERR enumeration completeness | PASS | — |
| 5. BC count reconciliation (16 total) | PASS | — |
| 6. product-brief.md v1.4.12 ratification semantics | PASS | — |
| 7. Vision-authority framing (no stale vision-verbatim claims) | PASS | — |
| 8. Cross-reference anchors | FAIL | MEDIUM |
| 9. Production-grade compliance (no MVP/for-now/TODO-for phrases) | PASS | — |
| 10. STATE.md zero-context-resume integrity | PASS (minor note) | LOW |
| 11. Frontmatter input-hash drift | PASS | — |
| 12. Routing-precedent question framing (D-032) | PASS | — |

---

## Findings

### F-R26-1 — MEDIUM: CLAUDE.md version references stale (brief v1.4.2, vision v1.1.1)

**File:** `/Users/jmagady/Dev/monocle/CLAUDE.md`
**Lines:** 22, 47, 48

**Defective state:**

Line 22:
```
- Brief: `v1.4.2` at `.factory/specs/product-brief.md`, `validate-brief` verdict: v5 VALID.
```

Line 47:
```
6. `.factory/specs/product-brief.md` v1.4.2 — Phase 1-4 scope, …
```

Line 48:
```
7. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.1 — re-approved 2026-05-12 …
```

**Current reality:**
- `product-brief.md` is at v1.4.12 (committed 11185a1; frontmatter confirmed).
- `domain-monocle-vision-synthesis.md` is at v1.1.2 (frontmatter confirmed).

**Impact:** An agent reading CLAUDE.md §Current Pipeline State or §Architectural Authority
will see the wrong version for these two critical artifacts. The §Current Pipeline State
note instructs readers to "read `.factory/STATE.md` for live state," which partially mitigates
the risk — STATE.md correctly lists v1.4.12 and v1.1.2. However CLAUDE.md is the first
document read by every agent at session start and stale version numbers increase the probability
of an agent fetching or reasoning about wrong artifact versions.

**Production-grade fix:** Update CLAUDE.md §Current Pipeline State and §Architectural Authority
lines to reflect the current versions:
- Line 22: `v1.4.2` → `v1.4.12`
- Line 47: `product-brief.md v1.4.2` → `product-brief.md v1.4.12`
- Line 48: `domain-monocle-vision-synthesis.md v1.1.1` → `domain-monocle-vision-synthesis.md v1.1.2`

**Routing specialist:** `vsdd-factory:state-manager` (owns CLAUDE.md pipeline-state section,
same as STATE.md version bookkeeping). If the human considers §Architectural Authority
to be spec-steward territory, route to `vsdd-factory:spec-steward`.

---

### F-R26-2 — MEDIUM: Stale `temp-env = "^0.2"` in SS-engine-module.md §Trace (v1.1.4 history block)

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md`
**Lines:** 786–792

**Defective state:**

```
v1.1.4 changes (round-22 fixes F-R22-1/F-R22-2/F-R22-3):
…
  `monocle-runtime/tests/engine_module.rs`. Test isolation strategy: `temp-env = "^0.2"`
  (new `[dev-dependencies]` pin in SS-deps-pin-manifest.md v1.1.6). …
  Variables cleared: HOME, USERPROFILE, XDG_DATA_HOME, XDG_CONFIG_HOME, XDG_CACHE_HOME,
  XDG_RUNTIME_DIR (all env vars that could allow `BaseDirs::new()` to succeed).
```

**Problem:** This historical trace entry for v1.1.4 retains the superseded `^0.2` pin and the
superseded XDG_* variable list. The round-24 fix (v1.1.6) corrected these in the live spec body
but did not update the v1.1.4 trace history to reflect that it was later superseded.

This is a documentation-accuracy issue that misleads implementers who scan the trace for
historical rationale. An implementer reading this section sequentially sees:
1. v1.1.4: "use `^0.2`, clear HOME/USERPROFILE/XDG_*"  (stale — still says ^0.2 and XDG_*)
2. v1.1.6: "split sync/async halves, bump to ^0.3, remove XDG_*" (correct)

The v1.1.4 trace does not carry a "(superseded by v1.1.6)" annotation, so a reader who stops
at v1.1.4 leaves with the wrong pin and wrong variable list.

**Production-grade fix:** The v1.1.4 trace block should add a supersession note at its
conclusion:
```
  (NOTE: `temp-env = "^0.2"` and the XDG_* variable list above were superseded in v1.1.6
   by `temp-env = "^0.3"` with `features = ["async_closure"]`, and by the corrected
   four-variable list HOME/USERPROFILE/HOMEDRIVE/HOMEPATH. See v1.1.6 changes below.)
```

Alternatively, the v1.1.4 trace block can be updated inline to cross-reference v1.1.6.
No behavioral content changes; the live §Behavioral Contracts section is correct.

**Routing specialist:** `vsdd-factory:architect` (owner of SS-engine-module.md).

---

### F-R26-3 — MEDIUM: product-brief.md Forward-compatibility Success Criteria cites SS-engine-module v1.1.5 (stale; current is v1.1.6)

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/product-brief.md`
**Line:** 244

**Defective state:**

```
Per `SS-core-types-and-abi.md`, `SS-daemon-lifecycle.md` v1.0.4, and `SS-engine-module.md` v1.1.5.
```

**Current reality:** `SS-engine-module.md` is now at v1.1.6. This line was correct at the time
of v1.4.11 authorship (round-23), but round-25 bumped SS-engine-module to v1.1.6. The brief's
v1.4.12 changelog entry notes the "F-R24-cons-3 citation refresh" for daemon-lifecycle (v1.0.3 →
v1.0.4) but did not update the engine-module version reference in this same sentence.

**Impact:** The Forward-compatibility Success Criteria table row references a version that no
longer exists as the current spec. A Phase 1 implementer validating conformance against the cited
version will open v1.1.5 — which does not exist as a distinct file (only v1.1.6 is present) —
and may be confused. More practically, this makes the brief's citation chain incomplete for audit
purposes.

**Production-grade fix:** Update line 244:
```
Per `SS-core-types-and-abi.md`, `SS-daemon-lifecycle.md` v1.0.4, and `SS-engine-module.md` v1.1.6.
```

**Routing specialist:** `vsdd-factory:product-owner` (owner of product-brief.md).

---

## Checks Passed — Evidence

### Check 2: temp-env pin reference consistency

**Canonical pin (SS-deps-pin-manifest.md v1.1.7 Dev Dependencies table):**
```toml
temp-env = { version = "^0.3", features = ["async_closure"] }
```

**SS-engine-module.md v1.1.6 §Behavioral Contracts BC-ENGINE-002-ERR (live spec):**
- Line 624: "temp-env ^0.3 (feature async_closure) is pinned in SS-deps-pin-manifest.md"
- Line 627: "`temp_env::async_with_vars` — async closure (requires `features = ["async_closure"]`)"
- Pre-Staging table line 719: "temp-env ^0.3 (features=["async_closure"])"
- §Trace v1.1.6 line 741: "bumped from `^0.2` to `{ version = "^0.3", features = ["async_closure"] }`"

**SS-conventions-anti-patterns.md v1.4 §Test Conventions:**
- Line 391: `temp-env = { version = "^0.3", features = ["async_closure"] }`

**Result:** All live spec references to temp-env use `^0.3` with `async_closure`. The stale
`^0.2` in SS-engine-module.md is confined to the v1.1.4 historical trace block (captured in
F-R26-2 above). No live-spec location says `^0.2`.

### Check 3: Test convention rule visibility

**SS-conventions-anti-patterns.md v1.4 §Test Conventions verified:**

1. **Semgrep rule name:** `monocle-no-raw-env-mutation-in-tests` — consistent with the
   project naming convention used for all other rules (`monocle-no-shell-injection`,
   `monocle-no-naked-fs-write`, `monocle-no-unbounded-channel`).

2. **Forbidden pattern syntactic validity:**
   ```rust
   std::env::set_var("HOME", "/tmp/test-home");
   std::env::remove_var("HOME");
   ```
   Both are valid Rust expressions; semgrep can match them.

3. **Required pattern (sync) syntactic validity:**
   ```rust
   temp_env::with_vars(
       [("HOME", None::<&str>)],
       || { /* test body */ },
   );
   ```
   Valid Rust. The type annotation `None::<&str>` is correct for the `Option<impl AsRef<OsStr>>`
   parameter.

4. **Required pattern (async) syntactic validity:**
   ```rust
   temp_env::async_with_vars(
       [("HOME", None::<&str>)],
       async { /* async test body */ },
   ).await;
   ```
   Valid Rust. The `async {}` block satisfies `F: Future<Output=R>` per the confirmed
   `async_with_vars` signature in SS-deps trace.

5. **Cargo.toml declaration:**
   ```toml
   temp-env = { version = "^0.3", features = ["async_closure"] }
   ```
   Matches the SS-deps-pin-manifest.md v1.1.7 canonical pin form exactly.

6. **BC-ENGINE-002-ERR cross-reference:** Line 399:
   "Canonical usage example: BC-ENGINE-002-ERR test in
   `monocle-runtime/tests/engine_module.rs` (see SS-engine-module.md §Behavioral Contracts)."
   Correct cross-reference; points to the live canonical usage.

7. **Inline temp-env version reference:** Line 349: "MUST use `temp-env 0.3+`". Consistent
   with the ^0.3 canonical pin.

### Check 4: BC-ENGINE-002-ERR enumeration completeness

All enumeration sites verified:

| Location | Present? |
|----------|---------|
| SS-engine-module.md §Behavioral Contracts | YES (line 615) |
| SS-engine-module.md §Phase 1 PRD BC Pre-Staging table | YES (line 719, 4th row) |
| SS-forward-compatibility.md reserved-ID table | YES (line 251, 15th row) |
| SS-core-types-and-abi.md §Phase 1 PRD BC Pre-Staging closing text | YES (line 1035: "BC-ENGINE-002-ERR") |
| product-brief.md v1.4.12 Forward-compatibility BC list | YES (line 244: "BC-ENGINE-001/002/002-ERR/003") |
| STATE.md | STATE.md does not enumerate BC IDs; not required. |

### Check 5: BC count reconciliation

| Artifact | BC IDs | Count |
|----------|--------|-------|
| SS-engine-module.md | BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003 | 4 |
| SS-core-types-and-abi.md | BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-FACTORY-001, BC-FACTORY-002, BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002 | 8 |
| SS-daemon-lifecycle.md | BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001 | 4 |
| SS-permissions-phase1.md | 0 (Phase 1 Permission BCs are captured under BC-TYPES-001 in SS-core-types-and-abi.md) | 0 |
| SS-forward-compatibility.md | 0 authored here; lists all 16 by reference | 0 |
| **Total** | | **16** |

SS-forward-compatibility.md table lists exactly 16 rows (counted: BC-RING-001, BC-ABI-001/002,
BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-AUTH-001/002, BC-LOCK-001,
BC-ENGINE-001/002/002-ERR/003 = 16). Arithmetic confirmed.

Note: SS-core-types-and-abi.md §Trace v1.2.1 (line 1136) carries a stale historical note
"Grand total 15 = 8 (SS-core) + 4 (SS-daemon) + 3 (SS-engine) confirmed unchanged." This
reflects the count AT THE TIME of the v1.2.1 round-16 changes, before BC-ENGINE-002-ERR existed.
It is an accurate historical record for that round but could mislead a reader of the trace.
Classified as observation only (historical trace; not a live claim); the body text at line 1037
correctly states 16.

### Check 6: product-brief.md v1.4.12 ratification semantics

v1.4.12 changelog entry (lines 77) verified:
- States v1.4.11 content "stands as ratified by product-owner via this entry" — YES.
- Phase 1 gate question captured: "Should architects be permitted to mechanically propagate
  counts…" — YES, matches STATE.md Phase 1 Gate Questions §2.
- Three daemon-lifecycle citation updates documented: "§Forward-compatibility contracts / JSONL
  ring sub-bullet, §Forward-compatibility contracts / Versioned auth token sub-bullet, and the
  Forward-compatibility Success Criteria table row" — YES, confirmed via grep: both body
  sub-bullets (lines 167-168) and the Success Criteria row (line 244) now cite v1.0.4 for
  daemon-lifecycle.
- `producer: product-owner` frontmatter: YES (line 6 of product-brief.md frontmatter).

### Check 7: Vision-authority framing (no stale vision-verbatim claims post-round-25)

SS-engine-module.md v1.1.6 §EngineModule Trait Signature: The two-category provenance
split (vision-verbatim vs vision-spirit-aligned) is intact and unchanged. No file edited in
round-25 introduced a new claim that the vision is authoritative for Phase 1 trait signatures.

Verified that SS-engine-module.md v1.1.6 §EngineModule Trait Signature still reads:
"The vision is non-authoritative for this surface per CLAUDE.md §Architectural Authority."
(lines 64-66). No regression.

### Check 8: Cross-reference anchors

- SS-conventions-anti-patterns.md v1.4 §Test Conventions reference to `SS-engine-module.md
  §Behavioral Contracts` (line 399): valid — the section heading "§Behavioral Contracts" exists
  in SS-engine-module.md.
- SS-engine-module.md v1.1.6 §Trace v1.1.6 reference to "SS-deps-pin-manifest.md (v1.1.7)"
  (line 741): valid — SS-deps-pin-manifest.md is at v1.1.7.
- SS-deps-pin-manifest.md v1.1.7 §Trace reference to "SS-engine-module.md" (line 267): valid.

No round-25 edits introduced broken section anchors.

### Check 9: Production-grade compliance

Verified the four round-25-modified files for CLAUDE.md §Canonical Principle anti-patterns:

| Anti-pattern phrase | Found? | Location |
|--------------------|--------|----------|
| "for now" | NO (excluding historical changelog descriptions) | — |
| "MVP" / "minimum viable" | NO | — |
| "good enough" | NO | — |
| "we can fix later" | NO | — |
| "TODO for architect" | NO | — |
| "pending architect review" (as a current status) | NO (one historical changelog row in v1.3 history; resolved) | — |
| "Placeholder for architect" | NO | — |

Note: `todo!()` macro appears in SS-engine-module.md spec code stubs for `spawn()` and
`preflight()`. These are intentional spec placeholders in `async fn spawn(...)` and
`async fn preflight(...)` — Phase 1 story implementations are expected to fill them. This
is not a production-grade violation per the Canonical Principle; the todo!() stubs explicitly
note "The Phase 1 story for `monocle-runtime` initialization provides the full implementation.
These signatures are binding — the implementer must not alter them." This is correct usage.

### Check 10: STATE.md zero-context-resume integrity

**Immediate Next Action (line 109):** Describes round-26 validation scope with correct
specifics — BC count reconciliation, temp-env pin, version pointers, and adversary scope.
Executable from fresh context. PASS.

**Phase 1 Gate Questions (lines 175-181):** Two questions present and well-formed:
1. Vision-vs-architecture authority (D-031) — precise and answerable (binary ratify/reject).
2. Architect-brief-routing precedent (D-032) — precise; presents both options explicitly.
Both are human-decision questions, not AI-actionable items. PASS.

**Critical Artifacts list (lines 111-122):** All 10 artifacts listed with correct versions:
- product-brief.md v1.4.12 — correct
- domain-monocle-vision-synthesis.md v1.1.2 — correct
- SS-core-types-and-abi.md v1.2.3 — correct
- SS-engine-module.md v1.1.6 — correct
- SS-daemon-lifecycle.md v1.0.4 — correct
- SS-permissions-phase1.md v1.1 — correct
- SS-deps-pin-manifest.md v1.1.7 — correct
- SS-conventions-anti-patterns.md v1.4 — correct
- SS-forward-compatibility.md v1.2.1 — correct
All correct. PASS.

**LOW observation:** STATE.md §Task Queue Snapshot "Resumption protocol for fresh-context
session" (line 159) says "Mark `#39` as `in_progress` when you dispatch the round 24 validation
chain." This should reference #40 (the round-26 task in the queue above). This is a stale
protocol instruction that a fresh-context agent would follow and be confused by. However, given
the STATE.md instructs agents to read the Immediate Next Action section first (which is correct),
this stale protocol line is unlikely to cause actual harm. Classified LOW. Routing: state-manager
to update at round-26 close-out.

### Check 11: Frontmatter input-hash drift

All examined architecture artifacts carry `input-hash: "[live-state]"`, consistent with
project policy for pre-implementation artifacts. No computed hashes requiring verification.
PASS.

### Check 12: Routing-precedent question framing (D-032)

STATE.md line 181: "Does the human accept a narrow exemption for mechanical count-propagation
across artifact boundaries, or should every cross-boundary edit route through the destination
owner even when content is mechanical?"

The question is precise: it presents both options (narrow exemption vs. always-route), identifies
the specific table row in CLAUDE.md (line 188) that would be affected, and notes no CLAUDE.md
change is made pre-answer. The human can answer with "narrow exemption accepted" or "always route."
PASS.

---

## Novelty Signal

Round 26 findings are CONTINUATION of known patterns rather than new defect classes:

- **F-R26-1** (CLAUDE.md stale versions): This class of finding (CLAUDE.md version refs lagging
  behind artifact bumps) has appeared in rounds 22 and 24 for other files. Same root cause:
  CLAUDE.md is not in the update path when architecture artifacts bump versions.

- **F-R26-2** (historical trace block retaining superseded pin): This is a new variant of the
  "stale content in §Trace" defect class. Prior rounds caught stale content in live spec sections;
  this is the first time the trace section itself has retained superseded values without a
  supersession annotation. Small expansion of known class.

- **F-R26-3** (brief Success Criteria cites stale engine-module version): Direct repetition of
  the F-R24-cons-3 pattern — a citation refresh hit daemon-lifecycle but missed engine-module in
  the same sentence. Same root cause: multi-artifact citations in a single sentence where only
  one was updated.

**Novelty decay signal:** Round 26 finds no new defect classes. All three findings are variants
of already-known patterns. This is consistent with convergence. The fix burst for these findings
should be brief (3 targeted edits). Severity trajectory: R20 0+2+1 → R22 0+3+0 → R24 0+3+2 →
R26 0+3+0. No escalation; trajectory holding at low MEDIUM counts with zero CRITICALs.

---

## Gate Result

**FAIL — 3 MEDIUM findings block Phase 1 gate.**

Gate passes after architect fixes F-R26-2, product-owner fixes F-R26-3, and state-manager fixes
F-R26-1. All three are mechanical edits with no behavioral content changes.
