---
document_type: consistency-check
level: ops
version: "1.0.0"
producer: vsdd-factory:consistency-validator
timestamp: 2026-05-26T00:00:00Z
scope: phase-1d-adversarial-fix-pass-1
---

# Cross-Document Consistency Check — Phase 1d Adversarial Fix Pass 1

**Date:** 2026-05-26
**Scope:** Files modified by Phase 1d Pass 1 adversarial fix pass
**Result:** PARTIAL PASS — 1 CRITICAL inconsistency, 2 MINOR findings

---

## Check 1: BC-INDEX SS-06 count and grand total

**Result: PASS**

BC-INDEX.md (v1.16) SS-06 section contains exactly 23 rows:
- BC-2.06.001 through BC-2.06.023 are all present in the table.
- `ss-06/` directory contains exactly 23 files (BC-2.06.001.md through BC-2.06.023.md).
- BC-INDEX summary table: `SS-06 TUI | 23 | 23 | 0` — correct.
- BC-INDEX summary table grand total: `**112** | **112** | **0**` — arithmetic correct
  (10+8+4+12+8+23+6+41 = 112).

---

## Check 2: SS-04 capability fields

**Result: PASS**

All 12 SS-04 BC files have `capability: CAP-004` in frontmatter:

| File | capability field |
|------|-----------------|
| BC-2.04.001.md | CAP-004 |
| BC-2.04.002.md | CAP-004 |
| BC-2.04.003.md | CAP-004 |
| BC-2.04.004.md | CAP-004 |
| BC-2.04.005.md | CAP-004 |
| BC-2.04.006.md | CAP-004 |
| BC-2.04.007.md | CAP-004 |
| BC-2.04.008.md | CAP-004 |
| BC-2.04.009.md | CAP-004 |
| BC-2.04.010.md | CAP-004 |
| BC-2.04.011.md | CAP-004 |
| BC-2.04.012.md | CAP-004 |

Note: BC-INDEX §Trace v1.16 records that only 6 BCs (001–006) had the CAP-001 mis-anchor
fixed. BCs 007–012 also show CAP-004 in frontmatter, consistent with not having had the
mis-anchor in the first place.

---

## Check 3: Priority consistency

**Result: PASS (with scope note)**

The BC frontmatter schema does NOT include a `priority:` field — priority is stored only
in BC-INDEX. This is by design: the BC files do not duplicate priority in frontmatter.

Priority verification was performed by cross-referencing BC-INDEX (the source of truth)
against PRD §7 Requirements Traceability Matrix references:

| BC ID | BC-INDEX Priority | PRD §7 Priority | Consistent? |
|-------|-------------------|-----------------|-------------|
| BC-2.04.003 | P1 | P1 | PASS |
| BC-2.04.012 | P1 | P1 | PASS |
| BC-2.06.006 | P1 | P1 | PASS |
| BC-2.06.015 | P2 | P2 | PASS |

BC-2.06.023 is P0 in BC-INDEX. It does not yet appear in PRD §7 (the PRD was not updated
in this burst to add the new BC to the RTM). See Finding CV-P1D-003 below.

---

## Check 4: Enum variant consistency — BC-2.06.022

**Result: PASS**

BC-2.06.022 v1.1.0 now uses:
- `PermissionDecision::AcceptAlways` (Step 2) — matches SS-ipc.md `PermissionDecision` enum
- `PermissionDecision::Accept` (Step 3) — matches SS-ipc.md `PermissionDecision` enum

SS-ipc.md canonical enum (line 262–266):
```
pub enum PermissionDecision {
    Accept,
    AcceptAlways,
    // Reject (also present)
}
```

The §Trace v1.1.0 in BC-2.06.022 explicitly records the fix with before/after for both
variants. No residual occurrences of `::Always` or `::Once` found in BC-2.06.022 body.

---

## Check 5: BC-2.07.005 Action::ProfilePicker

**Result: PASS**

BC-2.07.005 v1.1.0 now uses `Action::ProfilePicker` throughout. The §Trace v1.1.0 records:
"All occurrences of `Action::OpenProfilePicker` replaced with `Action::ProfilePicker`.
Canonical enum in SS-tui.md defines the variant as `ProfilePicker`, not `OpenProfilePicker`."

The body text verified at line 44 shows `Action::ProfilePicker` and at line 179 the §Trace
documents the replacement. No residual `OpenProfilePicker` found in BC-2.07.005.

---

## Check 6: BC-2.06.023 ↔ SS-ipc.md consistency on timeout behavior

**Result: FAIL — CRITICAL inconsistency**

### Finding CV-P1D-001 (CRITICAL)

**BC-2.05.005 Postcondition 4 contradicts SS-ipc.md v1.1.0 §Trace on timeout behavior.**

SS-ipc.md was updated to v1.1.0 (§Trace entry for F-P1D-007) with the following explicit
decision:

> `PermissionPromptResolved` IS sent to all connected TUI clients on hook timeout (in addition
> to user-decision resolution). This reverses a prior implicit "not sent" reading of the spec.

However, BC-2.05.005 (at version 1.0.0, unmodified by the Pass 1 fix burst) still states
the opposite in Postcondition 4 (lines 79-80):

> The daemon does NOT send `PermissionPromptResolved` to TUI clients. The stale overlay entry
> remains visible until the TUI next connects (or until a periodic cleanup sweep).

This contradiction also propagates into:
- BC-2.05.005 EC-002: "No `PermissionPromptResolved` sent." — incorrect per SS-ipc.md v1.1.0
- BC-2.05.005 Test Vector row: "Daemon resolves fail-open/closed; no `PermissionPromptResolved`
  sent; registry entry removed" — incorrect per SS-ipc.md v1.1.0
- BC-2.05.005 VP-TBD: "Timeout path: registry entry removed; no `PermissionPromptResolved`
  sent" — incorrect per SS-ipc.md v1.1.0

BC-2.06.023 (the new BC authored in this burst) correctly reflects the SS-ipc.md v1.1.0
decision: EC-001 and PC-3(c) reference "timeout fail-open" as a triggering case for
`PermissionPromptResolved`, and the Description says "by any means (user decision, timeout
fail-open, or another TUI client)". This is internally consistent with SS-ipc.md v1.1.0.

**The inconsistency:** BC-2.05.005 says timeout → no message; SS-ipc.md v1.1.0 + BC-2.06.023
say timeout → message IS sent. A developer implementing the daemon using BC-2.05.005 as the
behavioral spec would NOT send `PermissionPromptResolved` on timeout, causing BC-2.06.023
EC-002 to be untestable (the "post-reconnect no-op" case it describes would never occur via
timeout).

**Impacted artifacts:**
- `.factory/specs/behavioral-contracts/ss-05/BC-2.05.005.md` — PC-4 (line 79), EC-002 (line
  102), test vector (line 116), VP-TBD row (line 127)
- SS-ipc.md v1.1.0 §Trace (F-P1D-007) — source of truth (CORRECT)
- BC-2.06.023 — consistent with SS-ipc.md v1.1.0 (CORRECT)

**Required remediation (owner: product-owner):**
BC-2.05.005 must be updated (v1.0.0 → v1.1.0) to align with SS-ipc.md v1.1.0:
- PC-4 bullets 2–3: replace "does NOT send `PermissionPromptResolved`" with "DOES send
  `PermissionPromptResolved { prompt_id }` to all connected TUI clients, identical to the
  user-decision path."
- EC-002 expected behavior: remove "No `PermissionPromptResolved` sent." and replace with
  "Daemon sends `PermissionPromptResolved { prompt_id }` to all connected TUI clients."
- Test vector row: update expected output from "no `PermissionPromptResolved` sent" to
  "`PermissionPromptResolved { prompt_id }` sent to all connected clients."
- VP-TBD row: update property description to reflect the new expected behavior.
- Invariant 3 (line 92): "The daemon never sends `PermissionPromptResolved` without a
  corresponding prior `PermissionPromptQueued`" remains valid — this invariant is not
  contradicted by the timeout change. No change needed here.
- §Trace entry added at v1.1.0 citing F-P1D-007 (SS-ipc.md §Trace).

---

## Check 7: `which` crate in SS-deps-pin-manifest.md

**Result: PASS**

`which 7` is present in SS-deps-pin-manifest.md at line 65 (Phase 1 Pin Manifest table):

```
| which | 7 | PATH search for CCR binary detection in `monocle-config::detect_ccr()` ... |
  caret pin (`^7`); `which 7.x` is the current stable series as of 2026-05 (crates.io
  verified); MSRV 1.70 — well within Phase 1 floor of Rust 1.86; ... F-P1D-008 closure
```

The §Trace closure note `F-P1D-008 closure (missing from manifest, found in adversarial
review Pass 1)` is inline. The crate version, MSRV note, role description, and Cargo.toml
note are all present. SS-deps-pin-manifest.md is at v1.1.21.

---

## Additional Finding: BC-2.06.023 Missing from PRD §7 RTM

### Finding CV-P1D-003 (MINOR)

BC-2.06.023 is a new active P0 BC for SS-06, but it does not yet appear in PRD §7
Requirements Traceability Matrix. The PRD §7 RTM lists BCs by BC-INDEX entry as they existed
at the time of prd.md last update (v1.27.0, per §Trace v1.15 burst). The BC was added in the
§Trace v1.16 burst which did not include a PRD §7 RTM update.

This is a trace-completeness gap: BC-2.06.023 has no PRD row anchoring it to a FR-NNN or
NFR-NNN requirement.

**Impacted artifacts:**
- `prd.md` §7 Requirements Traceability Matrix
- BC-2.06.023.md (input-hash is `[pending]` — also unresolved)

**Required remediation (owner: product-owner):**
Add BC-2.06.023 row to PRD §7 RTM mapping it to FR-TUI-016 (or the appropriate FR for
PermissionPromptResolved TUI handling) and SS-06 module column.
This is a bookkeeping fix, not behavioral — the BC content is correct.

---

## Additional Finding: BC-INDEX §Trace v1.16 SE-16d Monotonicity Failure

### Finding CV-P1D-002 (MINOR)

BC-INDEX §Trace v1.16 (the burst that added BC-2.06.023) has a SE-16d failure:

```
SE-16d monotonicity PASS: 2026-05-26T00:00:00Z >= prior 2026-05-26T13:00:00Z (v1.15). PASS.
```

The v1.16 §Trace body timestamp is `2026-05-26T00:00:00Z` but the prior §Trace v1.15 was
timestamped `2026-05-26T13:00:00Z`. 00:00:00Z is NOT >= 13:00:00Z — this is an arithmetic
failure. The SE-16d audit incorrectly self-reported PASS.

The BC-INDEX frontmatter timestamp is `2026-05-26T13:00:00Z` (line 8), which IS >=
13:00:00Z from v1.15. However, the §Trace v1.16 body timestamp and the frontmatter timestamp
are inconsistent with each other (00:00:00Z vs 13:00:00Z).

**Root cause:** The v1.16 §Trace body was authored with the same timestamp as BC-2.06.023
itself (00:00:00Z). The BC-INDEX frontmatter was separately updated to 13:00:00Z but the
§Trace body was not corrected.

**Required remediation (owner: product-owner or state-manager as appropriate):**
Correct the §Trace v1.16 body timestamp from `2026-05-26T00:00:00Z` to `2026-05-26T13:00:00Z`
to match the frontmatter and satisfy SE-16d. The §Trace v1.16 also records that only 6 of the
12 SS-04 BCs received the CAP fix (001–006), but the actual files show BCs 007–012 also carry
CAP-004. That is not a contradiction (BCs 007–012 were authored with CAP-004 correctly) — the
§Trace entry is precise about the finding scope.

---

## Summary Table

| Check | Criterion | Result |
|-------|-----------|--------|
| 1 | BC-INDEX SS-06 row count = 23, grand total = 112 | PASS |
| 2 | All 12 SS-04 BCs have capability: CAP-004 | PASS |
| 3 | BC-2.04.003 P1, BC-2.04.012 P1, BC-2.06.006 P1, BC-2.06.015 P2 in BC-INDEX and PRD | PASS |
| 4 | BC-2.06.022 uses AcceptAlways and Accept (not Always, Once) | PASS |
| 5 | BC-2.07.005 uses Action::ProfilePicker (not OpenProfilePicker) | PASS |
| 6 | BC-2.06.023 consistent with SS-ipc.md on PermissionPromptResolved | PARTIAL — BC-2.06.023 is correct; BC-2.05.005 is stale (CRITICAL) |
| 7 | `which` crate in SS-deps-pin-manifest.md | PASS |

---

## Blocking Findings

**CV-P1D-001 (CRITICAL):** BC-2.05.005 PC-4 / EC-002 / test vector / VP row contradict
SS-ipc.md v1.1.0 on timeout behavior. BC-2.05.005 must be updated before the Wave 3 gate
or any story implementation begins on SS-05 / BC-2.05.005 scope. A test written against the
stale BC will assert the wrong behavior and conflict with BC-2.06.023's EC-002 coverage.

**Owner:** product-owner
**Priority:** Fix before next wave gate (Wave 3 gate)
**File:** `.factory/specs/behavioral-contracts/ss-05/BC-2.05.005.md`

---

## Non-Blocking Findings

**CV-P1D-002 (MINOR):** BC-INDEX §Trace v1.16 body timestamp `2026-05-26T00:00:00Z` is
earlier than v1.15 timestamp `2026-05-26T13:00:00Z` — SE-16d self-reported PASS is
arithmetically incorrect. Frontmatter timestamp `2026-05-26T13:00:00Z` is correct.
**Owner:** product-owner
**File:** `.factory/specs/behavioral-contracts/BC-INDEX.md` §Trace v1.16

**CV-P1D-003 (MINOR):** BC-2.06.023 not yet added to PRD §7 RTM. Trace-completeness gap.
BC-2.06.023 input-hash field is `[pending]` — should be populated after compute-input-hash.
**Owner:** product-owner
**File:** `.factory/specs/prd.md` §7 Requirements Traceability Matrix
