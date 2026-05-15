---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.12 db7f50e + VP v1.16 b0dd7b6 + arch v1.0.16 6bb93e2 + manifest v1.1.12 8005075; D-047 strict pass 2 attempt 1 (R83); pre-fix-burst snapshot"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T20:00:00Z
pass_number: 2
attempt: 1
policy: D-047-strict
verdict: FINDINGS
counter_before: 1/3
counter_after: 0/3
---

# Adversarial Review R83 — Phase 1 (D-047 Strict, Pass 2 attempt 1 — FINDINGS)

## Summary

**Verdict:** FINDINGS — counter RESET 1/3 → 0/3.

- 1 HIGH: F-R83-1 — 4-site propagation gap for the BC-DAEMON-005 Postcondition 8 / 0o700 invariant lift (F-R79-3). The contract was lifted to §Postcondition tier in PRD v1.12 but the lift was NOT propagated to all sibling summary tables in the architectural layer.
- 1 LOW: F-R83-2 — §References intro "current-as-of" timestamp in VP v1.16 remained pinned to `2026-05-15T18:00:00Z` while the frontmatter `timestamp` was bumped to `2026-05-15T21:00:00Z` as part of the F-R81 + GAP-R20 closure burst (b0dd7b6). Timestamp drift across two "current-as-of" anchors in the same document.
- 1 LOW (observation): Obs-R83-1 — PRD NFR-012 §Verification DirBuilder snippet uses the `DirBuilder::new().recursive(true).create(path)?` form. The canonical pattern in SS-daemon-lifecycle.md §Postcondition 8 context uses `std::fs::create_dir_all(path)?` (simpler, same semantics for the non-XDG-path creation branch). Not a correctness defect — both forms are valid — but an aesthetics/consistency observation for the implementer context.

This is a HIGH-novelty finding class. The 4-site propagation pattern for `lift_invariants_to_bcs` (F-R79-3 introduced BC-DAEMON-005 Postcondition 8) had not been identified in any prior adversary pass, despite 22 prior attempts and F-R83-1's 4 sites all being summary-table locations that other extensions (Extensions 8, 9, 10) had touched. The pattern escaped because prior disciplines focused on content-correctness axes (§Mechanism, §Post-conditions, §Trace, §Coverage Matrix footer); the sibling summary-table propagation axis had not been enumerated.

---

## F-R83-1 — HIGH: 4-site 0o700 propagation gap (lift_invariants_to_bcs sibling-site)

### Root cause

F-R79-3 lifted the EC-052 0o700 runtime-dir mode invariant from edge-case tier to BC-DAEMON-005 Postcondition 8 (PRD v1.12, db7f50e). The lift added one new §Postcondition row to the BC body. It did NOT propagate to the four sibling summary-table sites in the same architectural layer where the BC count and postcondition inventory are independently tracked.

### 4 evidenced sites

**Site 1 — PRD §4 NFR table (NFR-012 row):**

PRD v1.12 §4 (Non-Functional Requirements) NFR table did not contain a row for NFR-012 covering the 0o700 runtime-dir mode enforcement. The BC-DAEMON-005 Postcondition 8 text references a security invariant that has a corresponding NFR footprint (permission enforcement = operational security NFR). The NFR table was not updated to reflect this postcondition's existence as a non-functional requirement.

**Site 2 — arch §BC Summary footer (SS-daemon-lifecycle.md):**

SS-daemon-lifecycle.md §Behavioral Contract Summary footer tabulates postcondition counts per BC. BC-DAEMON-005 row postcondition count did not reflect the addition of Postcondition 8 (EC-052 0o700 lift from F-R79-3). The footer remained at the pre-F-R79-3 count for that BC row.

**Site 3 — VP §Catalog Overview row (VP v1.16):**

VP v1.16 §Catalog Overview table provides a per-VP summary row including postcondition count and coverage scope. VP-DAEMON-005 row did not reflect the addition of VP §Post-condition 9 (added in F-R75-1) and the corresponding BC-DAEMON-005 Postcondition 8 anchor. The coverage description retained the pre-F-R79-3 framing.

**Site 4 — VP §Auxiliary Mechanism Coverage row (VP v1.16):**

VP v1.16 §Auxiliary Mechanism Coverage table lists security-critical postconditions and their verification coverage. The 0o700 mode enforcement entry (VP-DAEMON-005 probe 5.e + BC-DAEMON-005 Postcondition 8) was not reflected in this table after the F-R79-3 lift.

### Disposition

Route to 3-agent parallel fix-burst:
- Product-owner: PRD v1.13 — add NFR-012 row to §4 NFR table + close Obs-R83-1 DirBuilder snippet form.
- Architect: arch v1.0.17 — update §BC Summary footer BC-DAEMON-005 postcondition count.
- Formal-verifier: VP v1.17 — update §Catalog Overview VP-DAEMON-005 row + §Auxiliary Mechanism Coverage 0o700 row.

---

## F-R83-2 — LOW: §References timestamp drift

VP v1.16 §References section introductory prose contains a "current-as-of" timestamp: `current-as-of: 2026-05-15T18:00:00Z`. The document frontmatter `timestamp` field was bumped to `2026-05-15T21:00:00Z` in the b0dd7b6 burst (F-R81 + GAP-R20 consolidated closure). The §References timestamp was not updated atomically with the frontmatter timestamp bump — creating a 3-hour drift between two "current-as-of" anchors in the same document.

This is structurally the same class as F-R81-2 / GAP-R19-001 / R13-001 (§Purpose stale-SHA recurrence). The §References intro timestamp is a second "current-as-of" pointer in VP that was not added to the propagation grep target list alongside §Purpose and §Trace when D-071 codified the §Purpose META recurrence guard.

**Disposition:** Route to formal-verifier in same VP v1.17 burst. Extend propagation grep target list from §Purpose to ALSO include §References intro timestamp.

---

## Obs-R83-1 — LOW: DirBuilder snippet form (PRD NFR-012 §Verification)

PRD §4 NFR-012 §Verification specifies `DirBuilder::new().recursive(true).create(path)?` as the illustrative snippet for directory creation. The canonical form used in BC-DAEMON-005 Postcondition 8 context and SS-daemon-lifecycle.md is `std::fs::create_dir_all(path)?` (which maps directly to the POSIX `mkdir -p` semantic and avoids the `DirBuilder` method-chain form).

Both forms are semantically equivalent for this use case. The observation is: consistency between PRD §Verification snippet and arch/BC canonical form aids implementer clarity. The `create_dir_all` form is shorter and less ambiguous about the no-previous-dir case.

**Disposition:** Product-owner may address in PRD v1.13 (route alongside F-R83-1 Site 1 fix). Non-blocking; observation only.

---

## F-R81 + GAP-R20 Closure Verification (all 5 HOLDING)

| Finding | Description | Status |
|---------|-------------|--------|
| F-R81-1 HIGH | Extension 11 canonical body BC-id prefix grep pattern | HOLDING |
| F-R81-2 / GAP-R20-001 MED | §Purpose stale SHA — 3rd recurrence; META recurrence guard codified | HOLDING |
| F-R81-3 LOW | §Trace v1.15 line-number refs → section-heading anchors | HOLDING |
| GAP-R20-002 MED | §G-6 residual BC-HOOK-022 normative framing retired; NFR-006 sole Phase 1 BC | HOLDING |
| GAP-R20-003 LOW | Extension 13 evidence form refined to grep -nE (line:text) | HOLDING |

---

## F-R80 META OPERATIONALLY HOLDING

Extension 13 (machine-greppable evidence requirement) verified operationally holding in VP v1.16. The 8 grep -nE transcripts present in v1.16 §Trace are independently auditable. No fabricated PASS verdicts detected in v1.16 or v1.17 precursor content. The META-discipline-integrity axis remains an asymptotic residual class requiring infrastructural support for complete closure, but the operational discipline is taking hold at the spec-authoring level.

---

## Novelty Assessment

**HIGH.** The 4-site `lift_invariants_to_bcs` propagation pattern (F-R83-1) is a new finding class not surfaced by any prior adversary pass (22 prior attempts). The pattern is structurally distinct from prior disciplines:

- It is not a content-body inconsistency (covered by Extension 2).
- It is not a §Coverage Matrix footer fabrication (covered by Extension 9).
- It is not a §Trace audit-row fabrication (covered by Extension 13).
- It is not a test-file/RTM mismatch (covered by Extension 10).
- It is specifically a **sibling summary-table propagation gap** for BC/VP tier-lift operations (lift_invariants_to_bcs class).

The F-R83-2 finding (§References timestamp drift) is a re-instance of the §Purpose stale-SHA recurrence pattern at a SECOND anchor in the same document — directly analogous to the D-071 codification finding that §Purpose was a third recurrence point. §References intro timestamp is now the fourth recurrence point.

---

## Verdict: FINDINGS — Counter RESET 1/3 → 0/3

Counter was at 1/3 after R82 CLEAN. F-R83-1 (HIGH) is a substantive finding that resets the counter per D-047 strict policy.

---

## Suggested R84 Lens Rotation

R83 found the `lift_invariants_to_bcs` sibling-site propagation axis. After the F-R83 fix-burst applies Extension 14, R84 should rotate to:

1. **Post-fix correctness verification:** Verify all 4 F-R83-1 sites are correctly closed (PRD NFR-012 row, arch §BC Summary footer, VP §Catalog Overview, VP §Auxiliary Mechanism Coverage).
2. **§References timestamp propagation audit:** Verify F-R83-2 is closed and §References intro timestamp is now on the META recurrence guard grep target list (alongside §Purpose and §Trace per D-071).
3. **Deeper lift_invariants_to_bcs sweep:** Check whether any other BCs in PRD §3 have had invariants recently lifted (BC-DAEMON-004 exit-code postconditions, BC-AUTH-002 taxonomy collapse) and verify all sibling summary tables are consistent.
4. **Obs-R83-1 DirBuilder form:** Confirm snippet form resolved or explicitly accepted as alternate.
