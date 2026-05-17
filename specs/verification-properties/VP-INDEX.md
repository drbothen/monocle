---
document_type: verification-property-index
level: L4
version: "1.5"
status: active
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T23:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "b09f325"
traces_to: prd.md
---

# Verification Property Index: monocle Phase 1

> **Source of truth** for all verification property IDs, titles, source BCs,
> proof methods, and file paths.
> VP frontmatter `source_bc:`, VP body Source Contract sections, story
> verification entries, and the PRD Verification Properties Index MUST all
> use IDs and titles from this table.
>
> **Append-only:** When a VP is withdrawn or replaced, mark it
> `lifecycle_status: withdrawn|retired` and add a `replacement:` column entry.
> Never remove a row or reuse an ID. See §Renumbering Appendix for old → new
> ID mappings.

---

## SS-01: Daemon Lifecycle VPs (10)

> Source-contract subsystem: BC-2.01.* (see `behavioral-contracts/BC-INDEX.md` §SS-01)
> Architecture source: `architecture/SS-daemon-lifecycle.md` v1.0.30
> Capability: CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management")

| VP ID | Title | Source BC | Proof Method | File | Old ID (PG-5) |
|-------|-------|-----------|--------------|------|---------------|
| VP-001 | Healthz Endpoint — Unauthenticated Liveness 200/503 | BC-2.01.001 | manual+proptest | vp-001-healthz-endpoint.md | VP-DAEMON-001 |
| VP-002 | Status Endpoint — Authenticated 10-Field JSON | BC-2.01.002 | manual+proptest | vp-002-status-endpoint.md | VP-DAEMON-002 |
| VP-003 | Body Size Limit — 256 KiB; HTTP 413 | BC-2.01.003 | manual+fuzz | vp-003-body-size-limit.md | VP-DAEMON-003 |
| VP-004 | Graceful Shutdown — 10-Second Drain + 5-Code Exit | BC-2.01.004 | manual | vp-004-graceful-shutdown.md | VP-DAEMON-004 |
| VP-005 | Lock File Lifecycle — Atomic Create, Pid Gate, Mode 0o600/0o700 | BC-2.01.005 | manual+mutation | vp-005-lock-file-lifecycle.md | VP-DAEMON-005 |
| VP-006 | Crash Recovery Checkpoint — JSON Write, Offer, Cleanup | BC-2.01.006 | manual+mutation | vp-006-crash-recovery-checkpoint.md | VP-DAEMON-006 |
| VP-007 | JSONL Ring Format-Version First Key | BC-2.01.007 | manual+mutation | vp-007-ring-format-version.md | VP-RING-001 |
| VP-008 | Auth Token Wire Format + Constant-Time Comparison | BC-2.01.008 | manual+fuzz | vp-008-auth-token-wire-format.md | VP-AUTH-001 |
| VP-009 | Auth Header Two-Body Taxonomy | BC-2.01.009 | manual+fuzz | vp-009-auth-header-validation.md | VP-AUTH-002 |
| VP-010 | Lock File `contract_version: 1` First Key | BC-2.01.010 | manual+mutation | vp-010-lock-file-contract-version.md | VP-LOCK-001 |

---

## SS-02: Core Types and ABI VPs (8)

> Source-contract subsystem: BC-2.02.* (see `behavioral-contracts/BC-INDEX.md` §SS-02)
> Architecture source: `architecture/SS-core-types-and-abi.md` v1.2.11
> Capability: CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction")

| VP ID | Title | Source BC | Proof Method | File | Old ID (PG-5) |
|-------|-------|-----------|--------------|------|---------------|
| VP-011 | ABI Version in `/status` Endpoint | BC-2.02.001 | integration-test | vp-011-abi-version-status-endpoint.md | VP-ABI-001 |
| VP-012 | `MONOCLE_ABI_VERSION` Pub Const Equals `1` | BC-2.02.002 | compile-time-check | vp-012-abi-version-crate-root.md | VP-ABI-002 |
| VP-013 | Non-Exhaustive Enum Policy (Modulo ADR-0004 Exemptions) | BC-2.02.003 | ast-audit+mutation-test | vp-013-non-exhaustive-enum-policy.md | VP-TYPES-001 |
| VP-014 | `FactoryAdapter` Trait Signature Stable | BC-2.02.004 | ast-audit | vp-014-factory-adapter-trait.md | VP-FACTORY-001 |
| VP-015 | `VsddFactoryAdapter::new` + Self-Reference Detection | BC-2.02.005 | integration-test+fuzz | vp-015-vsdd-factory-adapter.md | VP-FACTORY-002 |
| VP-016 | Proto Field Number 1 = `schema_version` in `HookEnvelope` | BC-2.02.006 | integration-test | vp-016-hook-envelope-proto-field-numbers.md | VP-PROTO-001a |
| VP-017 | Rust `HookEnvelope` Struct `pub schema_version: u32 = 1` | BC-2.02.007 | integration-test | vp-017-hook-envelope-schema-version-field.md | VP-PROTO-001b |
| VP-018 | `schema_version` Forward-Compat Contract (Phase 4 Dispatch) | BC-2.02.008 | integration-test+fuzz | vp-018-phase4-schema-version-validation.md | VP-PROTO-002 |

---

## SS-03: Engine Module VPs (4)

> Source-contract subsystem: BC-2.03.* (see `behavioral-contracts/BC-INDEX.md` §SS-03)
> Architecture source: `architecture/SS-engine-module.md` v1.1.18
> Capability: CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter")

| VP ID | Title | Source BC | Proof Method | File | Old ID (PG-5) |
|-------|-------|-----------|--------------|------|---------------|
| VP-019 | `EngineModule` Trait Signature Stable; `last_event_micros: Option<i64>` | BC-2.03.001 | ast-audit | vp-019-engine-module-trait.md | VP-ENGINE-001 |
| VP-020 | `ClaudeCodeModule::detect` Strict Basename Match | BC-2.03.002 | integration-test | vp-020-claude-code-module-impl.md | VP-ENGINE-002 |
| VP-021 | `metadata`/`enrich` Return `HomeUnresolvable` (All Four Home-Env Vars Unset) | BC-2.03.003 | integration-test | vp-021-home-unresolvable-error.md | VP-ENGINE-002-ERR |
| VP-022 | `hook_paths()` Returns Exactly 5 Entries — One per `HookType` Variant | BC-2.03.004 | integration-test | vp-022-claude-code-module-inherent-methods.md | VP-ENGINE-003 |

---

## Summary

- **SS-01 (Daemon Lifecycle):** 10 VPs sharded into individual files (Dispatch 5a).
- **SS-02 (Core Types and ABI):** 8 VPs sharded into individual files (Dispatch 5b — this dispatch).
- **SS-03 (Engine Module):** 4 VPs sharded into individual files (Dispatch 5b — this dispatch).
- **Total active Phase 1 VPs:** 22 (all sharded; monolithic
  `verification-properties.md` retired in Dispatch 5b).
- **Pending:** 0
- **Withdrawn:** 0
- **Retired:** 0

---

## Renumbering Appendix (Append-Only Protection)

This appendix preserves the PG-5 historical VP IDs against the new per-file
canonical IDs introduced in Dispatch 5a and Dispatch 5b.
Old IDs are NEVER reused for new properties; this table is the canonical
mapping for traceability into pre-Dispatch-5a artifacts.

| Old ID (PG-5 historical) | New ID | New File | Dispatch | Status |
|---------------------------|--------|----------|----------|--------|
| VP-DAEMON-001 | VP-001 | vp-001-healthz-endpoint.md | 5a | sharded |
| VP-DAEMON-002 | VP-002 | vp-002-status-endpoint.md | 5a | sharded |
| VP-DAEMON-003 | VP-003 | vp-003-body-size-limit.md | 5a | sharded |
| VP-DAEMON-004 | VP-004 | vp-004-graceful-shutdown.md | 5a | sharded |
| VP-DAEMON-005 | VP-005 | vp-005-lock-file-lifecycle.md | 5a | sharded |
| VP-DAEMON-006 | VP-006 | vp-006-crash-recovery-checkpoint.md | 5a | sharded |
| VP-RING-001 | VP-007 | vp-007-ring-format-version.md | 5a | sharded |
| VP-AUTH-001 | VP-008 | vp-008-auth-token-wire-format.md | 5a | sharded |
| VP-AUTH-002 | VP-009 | vp-009-auth-header-validation.md | 5a | sharded |
| VP-LOCK-001 | VP-010 | vp-010-lock-file-contract-version.md | 5a | sharded |
| VP-ABI-001 | VP-011 | vp-011-abi-version-status-endpoint.md | 5b | sharded |
| VP-ABI-002 | VP-012 | vp-012-abi-version-crate-root.md | 5b | sharded |
| VP-TYPES-001 | VP-013 | vp-013-non-exhaustive-enum-policy.md | 5b | sharded |
| VP-FACTORY-001 | VP-014 | vp-014-factory-adapter-trait.md | 5b | sharded |
| VP-FACTORY-002 | VP-015 | vp-015-vsdd-factory-adapter.md | 5b | sharded |
| VP-PROTO-001a | VP-016 | vp-016-hook-envelope-proto-field-numbers.md | 5b | sharded |
| VP-PROTO-001b | VP-017 | vp-017-hook-envelope-schema-version-field.md | 5b | sharded |
| VP-PROTO-002 | VP-018 | vp-018-phase4-schema-version-validation.md | 5b | sharded |
| VP-ENGINE-001 | VP-019 | vp-019-engine-module-trait.md | 5b | sharded |
| VP-ENGINE-002 | VP-020 | vp-020-claude-code-module-impl.md | 5b | sharded |
| VP-ENGINE-002-ERR | VP-021 | vp-021-home-unresolvable-error.md | 5b | sharded |
| VP-ENGINE-003 | VP-022 | vp-022-claude-code-module-inherent-methods.md | 5b | sharded |

---

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Source monolith (retired): `.factory/specs/verification-properties.md`
  v1.35 was the predecessor (commit 842402c). The monolith was deleted
  from the working tree in Dispatch 5b; per PG-5 historical preservation
  policy, the full content remains accessible via
  `git show 842402c:.factory/specs/verification-properties.md` and
  earlier commits. VP-INDEX.md is now the canonical entry point for
  Phase 1 verification properties.
- BC index: `behavioral-contracts/BC-INDEX.md` v1.5 (commit pending — PO 6A R107 Round 6A BC scope dispatch; supersedes v1.4 commit bb088a2 — PO 5A R106 Round 5 BC-INDEX dual-accept finalization; supersedes v1.2 commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure; supersedes v1.1 Dispatch 3 commit f259ade).
- PRD: `.factory/specs/prd.md` v1.26.5 (Dispatch 4 commit 1030c65; refreshed to v1.26.5 in F-R107-3 / GAP-R46-1 closure, PO 6B R107 Round 6B dispatch — commit pending; supersedes v1.26.4 in F-R106-4 closure, PO 5B commit df5605a — PRD §7 mass pin refresh; supersedes v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
- Architecture index: `architecture/ARCH-INDEX.md`.
- Template: `templates/L4-verification-property-template.md`.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md`.

---

## §Trace v1.2 — F-R105-13 LOW: VP-INDEX §References PRD + BC-INDEX Citation Refresh

**Bump:** v1.1 → v1.2.
**Predecessor pin:** v1.1 (Dispatch 5b commit e3824ec — SS-02 + SS-03 VP files + VP-INDEX complete + retire monolith).
**Scope of v1.2 (NORMATIVE — §References citation refresh; NO per-VP-row content change; NO Renumbering Appendix cascade):**

### Change set (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before §References:**
    - `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.1 (Dispatch 3 commit f259ade).`
    - `PRD: \`.factory/specs/prd.md\` v1.26 (Dispatch 4 commit 1030c65).`
  - **After §References:**
    - `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.2 (commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure; supersedes v1.1 Dispatch 3 commit f259ade).`
    - `PRD: \`.factory/specs/prd.md\` v1.26.3 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).`
- **SE-17c-d body-scope grep:** post-edit `grep -n "BC-INDEX.md\` v1.1" VP-INDEX.md` → 0 matches (the only remaining `v1.1` cite is inside this §Trace as before-evidence); `grep -n "prd.md\` v1.26 " VP-INDEX.md` → 0 matches.
- **Renumbering Appendix:** UNCHANGED — all 22 historical→current ID mappings preserved verbatim per append-only ID protection.

### Rationale

VP-INDEX is the canonical entry point for L4 verification properties and is included in the F-R105-13 sweep scope per the task instruction "VP-INDEX final version (if cascaded)". The BC-INDEX cite became stale when BC-INDEX bumped to v1.2 in commit 61133a7 (F-R105-3 + F-R105-9 + OBS-R44-1 closure); the PRD cite became stale when PO bumped PRD to v1.26.3 in commit b2b378b (F-R105-12 + GAP-R44-4 closure). Both citation refreshes are mechanical version-string updates with no content cascade. Per CLAUDE.md Production-Grade Rule 1: fix in scope rather than deferring.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.3 (commit b2b378b).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7).
- **R105 closure chain:** F-R105-13 LOW — VP-INDEX §References citation refresh as cascade-tail of 22-VP §References PRD citation refresh sweep.
- **Concurrent dispatches (T-128k Round 3):**
  - PO: PRD v1.26.2 → v1.26.3 (F-R105-12 + GAP-R44-4) — COMPLETE (commit b2b378b).
  - architect: auth-header interop adjudication — separate scope.
  - BA: L2-INDEX anchor fixes — separate scope.
  - FV: 22 VP files (v1.0.2→v1.0.3 or v1.0.1→v1.0.2) + this VP-INDEX v1.1→v1.2.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T20:30:00Z` >= chain high-water `2026-05-17T19:30:00Z` (PRD v1.26.3 frontmatter and per-VP prior §Trace timestamps). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References BC-INDEX cite `v1.1` → `v1.2`; §References PRD cite `v1.26` → `v1.26.3`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Cascade-tail of the 22-VP sweep co-located in this dispatch rather than deferred to a separate VP-INDEX-only commit. No tech-debt entries created. Renumbering Appendix preserved unchanged per append-only ID protection.


---

## §Trace v1.3 — F-R106-9 HIGH + F-R106-18 LOW + GAP-R45-4 LOW: VP-INDEX 4-Fix Cascade (Pin Refresh + Pin Additions + BC-INDEX Citation Refresh)

**Bump:** v1.2 → v1.3.
**Predecessor pin:** v1.2 (commit 932f4e0 — F-R105-13 §References BC-INDEX + PRD citation refresh).
**Scope of v1.3 (NORMATIVE — 4 coupled cascade fixes triggered by R106 Round-4 evidence + R45 Phase-1 consistency audit):**

### Change 1 — F-R106-9 HIGH: SS-01 architecture-source pin refresh v1.0.25 → v1.0.30 (NORMATIVE)

- **SE-17f §SS-01 architecture-source line:**
  - Before: `Architecture source: \`architecture/SS-daemon-lifecycle.md\` v1.0.25`
  - After: `Architecture source: \`architecture/SS-daemon-lifecycle.md\` v1.0.30`
- **Rationale:** Architect 5E is bumping SS-daemon-lifecycle v1.0.29 → v1.0.30 in parallel (R106 Round 5) for F-FC-I005 fabricated-FC-ID removal and dual-accept consolidation. VP-INDEX SS-01 carries the canonical architecture-source pin and must be refreshed to match the parallel 10-VP per-file pin sweep.

### Change 2 — F-R106-18 LOW: SS-02 architecture-source pin addition (NEW; NORMATIVE)

- **SE-17f §SS-02 architecture-source line:**
  - Before: `Architecture source: \`architecture/SS-core-types-and-abi.md\`` (no version pin)
  - After: `Architecture source: \`architecture/SS-core-types-and-abi.md\` v1.2.11`
- **Rationale:** F-R106-18 LOW evidence flagged that VP-INDEX SS-02 header carried no architecture-source pin while SS-01 did. Adds the canonical pin matching the current `architecture/SS-core-types-and-abi.md` frontmatter version (verified via `grep ^version:` 2026-05-17T22:30:00Z = v1.2.11). Pin format follows the SS-01 precedent exactly.

### Change 3 — F-R106-18 LOW: SS-03 architecture-source pin addition (NEW; NORMATIVE)

- **SE-17f §SS-03 architecture-source line:**
  - Before: `Architecture source: \`architecture/SS-engine-module.md\`` (no version pin)
  - After: `Architecture source: \`architecture/SS-engine-module.md\` v1.1.18`
- **Rationale:** Same as Change 2 applied to SS-03. Current `architecture/SS-engine-module.md` frontmatter (verified via `grep ^version:` 2026-05-17T22:30:00Z = v1.1.18). Pin format follows the SS-01 precedent exactly.

### Change 4 — GAP-R45-4 LOW: §References BC-INDEX citation refresh v1.2 → v1.4 (NORMATIVE)

- **SE-17f §References BC index line:**
  - Before: `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.2 (commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure; supersedes v1.1 Dispatch 3 commit f259ade).`
  - After: `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.4 (commit pending — PO 5A R106 Round 5 BC-INDEX dual-accept finalization; supersedes v1.2 commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure; supersedes v1.1 Dispatch 3 commit f259ade).`
- **Rationale:** PO 5A (parallel R106 Round 5 dispatch) bumped BC-INDEX v1.3 → v1.4 in commit pending. Verified at audit time: current `behavioral-contracts/BC-INDEX.md` frontmatter = v1.4 (`grep ^version:` 2026-05-17T22:30:00Z). The §References cascade-tail is co-located in this dispatch rather than deferred to a separate VP-INDEX-only commit (per Production-Grade Rule 1+5).

### SE-17c-d body-scope grep (NORMATIVE)

- Post-edit `grep -n "v1\.0\.25" VP-INDEX.md` outside §Trace blocks → 0 matches.
- Post-edit `grep -n "v1\.0\.30" VP-INDEX.md` outside §Trace blocks → 1 match (§SS-01 architecture-source line).
- Post-edit `grep -n "v1\.2\.11" VP-INDEX.md` outside §Trace blocks → 1 match (§SS-02 architecture-source line).
- Post-edit `grep -n "v1\.1\.18" VP-INDEX.md` outside §Trace blocks → 1 match (§SS-03 architecture-source line).
- Post-edit `grep -n "BC-INDEX.md\` v1\.2" VP-INDEX.md` outside §Trace blocks → 0 matches.
- Post-edit `grep -n "BC-INDEX.md\` v1\.4" VP-INDEX.md` outside §Trace blocks → 1 match (§References BC index line).
- **Renumbering Appendix:** UNCHANGED — all 22 historical→current ID mappings preserved verbatim per append-only ID protection.

### Authoritative cross-references

- **Architecture (SS-01):** `architecture/SS-daemon-lifecycle.md` v1.0.30 (commit pending — architect 5E dispatch).
- **Architecture (SS-02):** `architecture/SS-core-types-and-abi.md` v1.2.11.
- **Architecture (SS-03):** `architecture/SS-engine-module.md` v1.1.18.
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.4 (commit pending — PO 5A R106 Round 5).
- **R106 closure chain:** F-R106-9 HIGH (SS-01 pin refresh) + F-R106-18 LOW (SS-02/SS-03 pin additions) + GAP-R45-4 LOW (BC-INDEX cite refresh) — all four cascade fixes consolidated in this single v1.3 bump.
- **Concurrent dispatches (R106 Round 5):**
  - PO 5A: BC + BC-INDEX dual-accept finalization (BC-INDEX v1.3 → v1.4) — observed COMPLETE at audit time.
  - PO 5B: PRD + supplements — separate scope.
  - PO 5C: product-brief — separate scope.
  - FV 5D: this dispatch (VP-009 v1.0.4 expansion + 10-VP pin sweep v1.0.4 + this VP-INDEX v1.3 cascade).
  - Architect 5E: ADR-0005 path normalization + SS-daemon-lifecycle v1.0.29 → v1.0.30 — separate scope (commit pending).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T22:30:00Z` >= chain high-water `2026-05-17T22:10:00Z` (BC-2.01.009 v1.0.3 frontmatter timestamp; cross-dispatch BC reference). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §SS-01 architecture-source pin refresh `v1.0.25` → `v1.0.30`; §SS-02 new architecture-source pin `v1.2.11`; §SS-03 new architecture-source pin `v1.1.18`; §References BC-INDEX cite refresh `v1.2` → `v1.4`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsections; cross-reference subsection; concurrent dispatch context.

### SE-17g META audit (NORMATIVE)

Sweep-wide post-edit re-grep across all 22 VP files for body-scope (excluding §Trace blocks): `grep -E "SS-daemon-lifecycle.md v1\.0\.25" .factory/specs/verification-properties/vp-*.md` body scope → 0 matches; `grep -E "arch v1\.0\.25" .factory/specs/verification-properties/vp-*.md` body scope → 0 matches. F-R106-10 + F-R106-9 closure verified. SS-daemon-lifecycle pin-citation stale-zero invariant restored across all VP body content.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Cascade-tail of the 4-fix VP-INDEX update co-located in this dispatch rather than deferred to separate per-fix commits. The 3 mechanical pin additions/refreshes (Changes 1-3) and the 1 mechanical citation refresh (Change 4) share the same audit-trail and re-grep evidence — bundling them into v1.3 preserves the §Trace chain continuity. No tech-debt entries created. Renumbering Appendix preserved unchanged per append-only ID protection.


---

## §Trace v1.4 — T-127''-prefix pre-R107 fix: VP-INDEX §References PRD Citation Refresh v1.26.3 → v1.26.4 + BC-INDEX Commit SHA Resolution

**Bump:** v1.3 → v1.4.
**Predecessor pin:** v1.3 (commit pending in current burst chain — FV 5D R106 Round 5 4-fix cascade: F-R106-9 + F-R106-18 + GAP-R45-4).
**Scope of v1.4 (NORMATIVE — pre-adversary-R107 mechanical coherence fix; closes spec-steward R5 audit INFO finding):**

### Change 1 — Spec-Steward R5 INFO: §References PRD pin refresh v1.26.3 → v1.26.4 (NORMATIVE)

- **SE-17f §References PRD line:**
  - Before: `PRD: \`.factory/specs/prd.md\` v1.26.3 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).`
  - After: `PRD: \`.factory/specs/prd.md\` v1.26.4 (Dispatch 4 commit 1030c65; refreshed to v1.26.4 in F-R106-4 closure, PO 5B commit df5605a — PRD §7 mass pin refresh; supersedes v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).`
- **Rationale:** Spec-steward R5 audit (just completed) returned CLEAN overall but flagged one INFO: VP-INDEX §References PRD pin cited stale v1.26.3 while current PRD frontmatter (verified `grep ^version: .factory/specs/prd.md` 2026-05-17T22:50:00Z) is v1.26.4 (bumped by PO 5B commit df5605a closing F-R106-4 PRD §7 mass pin refresh). Per CLAUDE.md Production-Grade Default Rule 1+5: fix mechanical coherence issues before adversary R107 dispatch to avoid counter reset on a known-mechanical INFO finding. Cite history chain preserved (supersession of v1.26.3) per append-only §References audit-trail convention.

### Change 2 — Co-Located Cleanup: §References BC-INDEX commit SHA resolution `pending` → `bb088a2` (NORMATIVE)

- **SE-17f §References BC-INDEX line:**
  - Before: `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.4 (commit pending — PO 5A R106 Round 5 BC-INDEX dual-accept finalization; ...).`
  - After: `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.4 (commit bb088a2 — PO 5A R106 Round 5 BC-INDEX dual-accept finalization; ...).`
- **Rationale:** While in this file fixing the PRD cite, the BC-INDEX cite carried a stale `commit pending` placeholder from v1.3's same-burst-coordination ambiguity; PO 5A landed in commit bb088a2 (verified `git log --oneline -- specs/behavioral-contracts/BC-INDEX.md` 2026-05-17T22:50:00Z). Per Production-Grade Rule 4: if an AI agent finds an issue in another AI's output during in-scope work, the default is to fix it in scope. Resolves the placeholder to the actual SHA — purely mechanical, zero content cascade. Supersession chain preserved.

### SE-17c-d body-scope grep (NORMATIVE)

- Post-edit `grep -n "prd.md\` v1\.26\.3" VP-INDEX.md` outside §Trace blocks → 0 matches (the only remaining v1.26.3 cites are inside §Trace v1.2 / v1.3 blocks as preserved historical evidence + this §Trace v1.4 block as before-evidence/cross-reference).
- Post-edit `grep -n "prd.md\` v1\.26\.4" VP-INDEX.md` outside §Trace blocks → 1 match (§References PRD line).
- Post-edit `grep -n "commit pending" VP-INDEX.md` outside §Trace blocks → 0 matches.
- Post-edit `grep -n "commit bb088a2" VP-INDEX.md` outside §Trace blocks → 1 match (§References BC-INDEX line).
- **Renumbering Appendix:** UNCHANGED — all 22 historical→current ID mappings preserved verbatim per append-only ID protection.
- **Per-VP-row content:** UNCHANGED — no source-BC, proof-method, file-path, or VP-ID cell modified.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.4 (PO 5B commit df5605a — F-R106-4 PRD §7 mass pin refresh).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.4 (PO 5A commit bb088a2 — R106 Round 5 BC dual-accept finalization).
- **Spec-steward R5 audit closure:** sole INFO finding (PRD cite stale) resolved.
- **R107 dispatch readiness:** VP-INDEX §References mechanical-coherence stale-zero invariant restored prior to adversary R107 fresh-context pass.
- **Cycle / counter:** cycle-001 counter 0/3 (this is a pre-R107 fix burst; not an adversary novelty pass).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T22:50:00Z` >= chain high-water `2026-05-17T22:30:00Z` (v1.3 §Trace timestamp + frontmatter timestamp). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References PRD cite refresh `v1.26.3` → `v1.26.4` (with supersession chain); §References BC-INDEX `commit pending` → `commit bb088a2`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsections; cross-reference subsection; R107 readiness context.

### Per CLAUDE.md Production-Grade Default Rule 1+4+5

Rule 1: mechanical coherence fix executed in-scope rather than deferred or rationalized as "MVP-acceptable stale cite." Rule 4: BC-INDEX `commit pending` placeholder discovered while in-scope was fixed in-scope rather than surfaced as a follow-up advisory. Rule 5: cheapest path (edit only the PRD line) was rejected in favor of correct path (also resolve the BC-INDEX placeholder discovered in the same hunk). No tech-debt entries created. Renumbering Appendix preserved unchanged per append-only ID protection. §Trace v1.2, v1.3, v1.4 chain continuity preserved verbatim.

---

## §Trace v1.5 — F-R107-4 HIGH + GAP-R46-1 HIGH + F-R107-8 (part 2): VP-INDEX §References Cascade for R107 Round 6C FV Sweep

**Bump:** v1.4 → v1.5.
**Predecessor pin:** v1.4 (commit 01af634 — pre-R107 fix burst: PRD pin v1.26.3 → v1.26.4 + BC-INDEX commit-pending → bb088a2).
**Scope of v1.5 (NORMATIVE — mechanical §References citation refresh for R107 Round 6C parallel-dispatch coordination; NO per-VP-row content change; NO Renumbering Appendix cascade):**

### Change 1 — §References PRD Citation Refresh v1.26.4 → v1.26.5 (NORMATIVE)

- **SE-17f §References PRD line:**
  - Before: `PRD: \`.factory/specs/prd.md\` v1.26.4 (Dispatch 4 commit 1030c65; refreshed to v1.26.4 in F-R106-4 closure, PO 5B commit df5605a — PRD §7 mass pin refresh; supersedes v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).`
  - After: `PRD: \`.factory/specs/prd.md\` v1.26.5 (Dispatch 4 commit 1030c65; refreshed to v1.26.5 in F-R107-3 / GAP-R46-1 closure, PO 6B R107 Round 6B dispatch — commit pending; supersedes v1.26.4 in F-R106-4 closure, PO 5B commit df5605a — PRD §7 mass pin refresh; supersedes v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).`
- **Rationale:** PO 6B (parallel R107 Round 6 dispatch) bumps PRD v1.26.4 → v1.26.5. VP-INDEX §References cascade-tail refreshed to target v1.26.5 in the same dispatch as the 22-VP per-file sweep, preserving the stale-citation-zero invariant. Cite history chain preserved (supersession of v1.26.4 + v1.26.3) per append-only §References audit-trail convention.

### Change 2 — §References BC-INDEX Citation Refresh v1.4 → v1.5 (NORMATIVE)

- **SE-17f §References BC-INDEX line:**
  - Before: `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.4 (commit bb088a2 — PO 5A R106 Round 5 BC-INDEX dual-accept finalization; supersedes v1.2 commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure; supersedes v1.1 Dispatch 3 commit f259ade).`
  - After: `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.5 (commit pending — PO 6A R107 Round 6A BC scope dispatch; supersedes v1.4 commit bb088a2 — PO 5A R106 Round 5 BC-INDEX dual-accept finalization; supersedes v1.2 commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure; supersedes v1.1 Dispatch 3 commit f259ade).`
- **Rationale:** PO 6A (parallel R107 Round 6 dispatch) bumps BC-INDEX v1.4 → v1.5. VP-INDEX §References cascade-tail refreshed to target v1.5 in the same dispatch, preserving the stale-citation-zero invariant. Cite history chain preserved (supersession of v1.4 + v1.2 + v1.1) per append-only §References audit-trail convention.

### SE-17c-d body-scope grep (NORMATIVE)

- Post-edit `grep -n "prd.md\` v1.26.[0-4]" VP-INDEX.md` outside §Trace blocks → 0 matches (the only remaining v1.26.4 / v1.26.3 cites are inside §Trace v1.2 / v1.3 / v1.4 / v1.5 blocks as preserved historical evidence + before-evidence in v1.5).
- Post-edit `grep -n "prd.md\` v1.26.5" VP-INDEX.md` outside §Trace blocks → 1 match (§References PRD line).
- Post-edit `grep -nE "BC-INDEX\.md\` v1\.[1-4]" VP-INDEX.md` outside §Trace blocks → 0 matches.
- Post-edit `grep -n "BC-INDEX.md\` v1.5" VP-INDEX.md` outside §Trace blocks → 1 match (§References BC index line).
- **Renumbering Appendix:** UNCHANGED — all 22 historical→current ID mappings preserved verbatim per append-only ID protection.
- **Per-VP-row content:** UNCHANGED — no source-BC, proof-method, file-path, or VP-ID cell modified.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.5 (commit pending — PO 6B R107 Round 6B PRD + supplements dispatch; supersedes v1.26.4 commit 01af634).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.5 (commit pending — PO 6A R107 Round 6A BC scope dispatch; supersedes v1.4 commit bb088a2).
- **R107 closure chain:** F-R107-4 HIGH (VP-009 ADR-0005 pin refresh — handled in VP-009 v1.0.5 in this dispatch); GAP-R46-1 HIGH (22-VP PRD cite refresh sweep — handled across all 22 VPs in this dispatch); F-R107-8 part 2 (22-VP active BC-INDEX cite addition — handled across all 22 VPs in this dispatch); VP-INDEX cascade tail co-located here.
- **Concurrent dispatches (R107 Round 6):**
  - PO 6A: BC + BC-INDEX scope (BC-INDEX v1.4 → v1.5) — separate scope.
  - PO 6B: PRD + supplements (PRD v1.26.4 → v1.26.5) — separate scope.
  - FV 6C: this dispatch (VP-009 ADR-0005 pin refresh + 22-VP PRD cite refresh + 22-VP BC-INDEX active cite addition + VP-INDEX cascade — THIS file).
  - Architect 6D: SS-forward-compatibility scope — separate scope.
  - BA 6E: L2-INDEX scope — separate scope.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T23:00:00Z` >= chain high-water `2026-05-17T22:50:00Z` (v1.4 §Trace timestamp + frontmatter timestamp). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References PRD cite refresh `v1.26.4` → `v1.26.5` (with supersession chain); §References BC-INDEX cite refresh `v1.4` → `v1.5` (with supersession chain); frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsections; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Rule 1: mechanical citation refresh executed in-scope rather than deferred or rationalized as "MVP-acceptable stale cite." Rule 5: cheapest path (separate VP-INDEX-only commit later) was rejected in favor of correct path (co-locate VP-INDEX cascade with the 22-VP per-file sweep in a single coordinated R107 Round 6C FV dispatch). PRD v1.26.5 and BC-INDEX v1.5 cites are post-PO-6B and post-PO-6A targets (commit pending — will resolve to concrete SHAs during final state-manager pass after parallel dispatches converge). No tech-debt entries created. Renumbering Appendix preserved unchanged per append-only ID protection. §Trace v1.2 / v1.3 / v1.4 chain continuity preserved verbatim.
