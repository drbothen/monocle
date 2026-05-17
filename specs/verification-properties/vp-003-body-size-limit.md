---
document_type: verification-property
level: L4
version: "1.0.3"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T20:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.01.003
module: monocle-runtime
proof_method: manual+fuzz
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-003: Body Size Limit — 256 KiB; HTTP 413 on Excess

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-003 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

POST requests to any authenticated route (the 5 hook endpoints
`/hooks/pre-tool-use`, `/hooks/notification`, `/hooks/stop`,
`/hooks/session-start`, `/hooks/prompt-submit`, and `/status`,
`/shutdown`) with a request body strictly exceeding 256 KiB
(262,144 bytes) are rejected with HTTP 413 and body
`{"error":"payload_too_large","limit_bytes":262144}`. Bodies up to and
including exactly 262,144 bytes succeed. The `DefaultBodyLimit::max(256 *
1024)` layer is mounted on the authenticated router only — `/healthz`
(unauthenticated, GET-only) is NOT subject to the limit. The source layer is
asserted by source-grep (exactly once, on `auth_router`).

## Source Contract

- **BC:** BC-2.01.003 — Body Size Limit (256 KiB, HTTP 413).
- **Postcondition/Invariant:** boundary semantics (`> N` rejected, `<= N`
  accepted, where `N = 262144`); exact error body shape; layer placement
  on authenticated router only; uniform cross-route behavior across all
  authenticated routes including `/status`.
- **Traces to (historical):** BC-DAEMON-003 (PRD v1.25 §BC-DAEMON-003;
  SS-daemon-lifecycle.md v1.0.25 §Body Size Limit).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — finite size set around boundary | Boundary semantics at N-1 / N / N+1 across 5 hook endpoints + `/status`; exact body shape on 413 |
| Fuzz (auxiliary) | cargo-fuzz / libFuzzer | Bounded body length 262_140..=262_150 | Boundary exploration; daemon-panic absence |
| Source-grep (structural) | ripgrep | N/A — static | `DefaultBodyLimit::max(256 * 1024)` appears exactly once in `monocle-runtime/src/server.rs` on `auth_router` only |

## Mechanism

Integration test (primary; harness at `monocle-runtime/tests/body_size_limit.rs`
— files in `<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM
Test Type column labels this BC `Integration`); fuzz (auxiliary — boundary
exploration). The integration harness constructs request bodies of sizes
262_143, 262_144, and 262_145 and asserts the corresponding HTTP responses
on each of the 5 hook endpoints + `/status`. The fuzz target sweeps
boundary-adjacent sizes (262_140..=262_150) to catch off-by-one regressions
and asserts daemon-panic absence.

## Pre-conditions

- Daemon running with a valid lock file.
- `axum::extract::DefaultBodyLimit::max(256 * 1024)` is the layer pinned
  to the authenticated router at construction time. The integration test
  asserts the layer is present via a `cargo expand` or source-grep
  inspection of `monocle-runtime/src/server.rs`.
- Test client holds the auth token for the positive controls.

## Post-conditions

1. POST 262,145-byte body to any of the 5 hook endpoints with valid auth →
   HTTP 413 with exact body
   `{"error":"payload_too_large","limit_bytes":262144}`.
2. POST 262,144-byte body (boundary) to any hook endpoint with valid auth →
   HTTP 200 (within limit; processed normally).
3. POST 262,143-byte body (one under) to any hook endpoint with valid auth →
   HTTP 200.
4. POST 262,145-byte body to `/status` with valid auth → HTTP 413
   (cross-route limit coverage). Cross-check VP-002 §Post-condition
   5 (`/status` route inherits the body-limit layer): VP-002 cites
   this VP as the cross-check for the `/status` route's 413 behavior;
   this VP asserts that 262,145-byte bodies on `/status` produce the
   same HTTP 413 response shape as on the 5 hook endpoints.
5. Source-grep asserts `DefaultBodyLimit::max(256 * 1024)` appears
   exactly once in `monocle-runtime/src/server.rs` and is applied to the
   authenticated router only (not the `/healthz` route).

## Counter-examples

1. `DefaultBodyLimit` layer omitted — 262,145-byte body returns HTTP 200
   (the request is processed, exposing unbounded memory); the test must
   assert 413.
2. `DefaultBodyLimit::max(256 * 1024)` applied to the unauthenticated
   router by mistake — `/healthz` would reject oversized bodies but
   `/healthz` is GET-only; benign drift but still wrong; the source-grep
   asserts the layer is on the authenticated router only.
3. Limit set to `262_144` instead of `256 * 1024` (off-by-one constant) —
   functionally identical (both equal 262,144) but the literal constant
   form `256 * 1024` is preferred for readability; the source-grep
   tolerates either form.
4. Error body returns `{"error":"too_large"}` (typo / non-canonical) —
   fails the exact-body assertion.

## Probe Matrix

| Probe | Setup | Expected status | Expected body |
|-------|-------|-----------------|---------------|
| 3.a | POST 262,143 bytes to any of 5 hook endpoints (valid auth) | 200 | normal route body |
| 3.b | POST 262,144 bytes (boundary) to any of 5 hook endpoints | 200 | normal route body |
| 3.c | POST 262,145 bytes to any of 5 hook endpoints | 413 | `{"error":"payload_too_large","limit_bytes":262144}` |
| 3.d | POST 262,145 bytes to `/status` (cross-route inheritance) | 413 | same body as 3.c |
| 3.e | Source-grep: `DefaultBodyLimit::max(256 * 1024)` placement | N/A | layer present on `auth_router` only (exactly once) |
| 3.f | Fuzz: body lengths 262_140..=262_150 sweep | per limit | 413 iff length > 262_144; 200 iff length ≤ 262_144; no daemon panic |

## Harness Location

- `monocle-runtime/tests/body_size_limit.rs` (integration)
- `fuzz/fuzz_targets/fuzz_body_size_boundary.rs` (fuzz, Phase 6 deliverable —
  `cargo fuzz add fuzz_body_size_boundary`)
- Test name: `test_BC_DAEMON_003_body_size_limit_413_on_excess` (per PRD
  v1.25 §BC-DAEMON-003, Verification subsection — to be migrated to
  `test_BC_2_01_003_body_size_limit_413_on_excess`).

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: manual+fuzz
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_01_003() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/body_size_limit.rs` (integration)
- `fuzz/fuzz_targets/fuzz_body_size_boundary.rs` (fuzz, Phase 6 deliverable —
  `cargo fuzz add fuzz_body_size_boundary`)
- Test name: `test_BC_DAEMON_003_body_size_limit_413_on_excess` (per PRD
  v1.25 §BC-DAEMON-003, Verification subsection — to be migrated to
  `test_BC_2_01_003_body_size_limit_413_on_excess`).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: manual+fuzz` per frontmatter; mechanism documented in `## Mechanism` section. |
| Tool support | Available | Tooling pinned in `architecture/SS-deps-pin-manifest.md`; no novel verification tooling required. |
| Estimated proof time | Within Phase-1 budget | `feasibility: feasible` per frontmatter. Coverage details in `## Proof Method` table; probe enumeration in `## Probe Matrix`. |

**Authoritative feasibility verdict:** `feasibility: feasible` per frontmatter (canonical machine-consumed field).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | v1.0.0 (cycle) | vsdd-factory:architect |
| Proof harness committed | pending Phase-3 implementation | vsdd-factory:formal-verifier |
| Proof first passed | pending Phase-6 formal hardening | vsdd-factory:formal-verifier |
| Locked (VERIFIED) | pending (`verification_lock: false`) | vsdd-factory:formal-verifier |

**Authoritative lifecycle state** (canonical machine-consumed fields in frontmatter):

| Field | Current Value |
|-------|---------------|
| `lifecycle_status` | `active` |
| `introduced` | `v1.0.0` |
| `verification_lock` | `false` |
| `proof_completed_date` | `null` |
| `modified` | `[]` |
| `deprecated` | `null` |
| `retired` | `null` |
| `withdrawn` | `null` |

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-DAEMON-003 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.003.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26.3 §BC-2.01.003 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.17.
- Cross-property: VP-002 (`/status` cross-route inheritance).

---

## §Trace v1.0.1 — Audit R2 Residual RES-03: VP Heading Reconciliation to L4 Template

**Bump:** v1.0 → v1.0.1.
**Predecessor pin:** v1.0 (Dispatch 5a/5b commits 7326ff5 + e3824ec — VP monolith decomposition; Dispatch 7 commit 51e77cb — input-hash population).
**Scope of v1.0.1 (Option 3 hybrid — heading reconciliation; NO content removed):**

### Heading changes (NORMATIVE)

- **Renamed** `## Verification Method` → `## Proof Method` (template-strict per L4-verification-property-template.md §Proof Method). Content unchanged; identical Method/Tool/Bounded?/Coverage table preserved verbatim.
- **Added** `## Proof Harness Skeleton` (template-required §Proof Harness Skeleton). Section is a template-strict skeleton block that references the existing rich harness specification (`## Mechanism` for execution narrative, `## Probe Matrix` for probe enumeration, `## Harness Location` for file + test name). The L4 template's skeleton is a Rust code-block stub; monocle's pre-existing `## Mechanism` + `## Harness Location` exceed the template's stub fidelity by carrying execution narrative AND concrete file paths. The new `## Proof Harness Skeleton` section satisfies the template heading requirement without removing the richer Phase-1-specific content.
- **Added** `## Feasibility Assessment` (template-required §Feasibility Assessment). Section is a populated Factor/Assessment/Notes table derived from the `feasibility:` frontmatter field (canonical machine-consumed value) + the `## Proof Method` Bounded?/Coverage columns. The L4 template carries feasibility as both a frontmatter field and a body table; pre-v1.0.1 monocle VPs carried it only in frontmatter. v1.0.1 adds the body table without changing the authoritative frontmatter value.
- **Added** `## Lifecycle` (template-required §Lifecycle). Section is a populated Event/Date/Actor table + an authoritative lifecycle-state mirror of the DF-030 lifecycle frontmatter fields. The L4 template carries lifecycle as both frontmatter fields and a body table; pre-v1.0.1 monocle VPs carried it only in frontmatter. v1.0.1 adds the body table without changing the authoritative frontmatter values.

### Project-specific extensions retained (INFORMATIONAL rationale per SE-17g)

The monocle VP body retains the following sections beyond the L4 template's minimum set; they encode Phase-1-specific verification discipline that emerged during the cycle-001 adversarial review chain:

- `## Mechanism` — execution narrative for the proof harness. Encodes the F-R88-5 discipline of separating test-type-class (Integration/Unit/AST-audit/Proptest) from harness-execution-narrative. Without this section the harness intent reduces to a one-cell `Method` column in `## Proof Method`, which proved insufficient during R85-R87 for adversary fresh-context comprehension.
- `## Pre-conditions` — precondition surface for the harness. Required for proof-harness reproducibility per F-R89/F-R90 work.
- `## Post-conditions` — assertion surface for the harness. The numbered postcondition format (1., 2., 3., …) emerged from R88-R91 BC↔VP round-trip discipline; flat unstructured postcondition prose proved insufficient for fresh-context BC→VP traceability.
- `## Counter-examples` — negative cases. Encodes mutation-test rationale per VP-013's `## Mutation-Test Rationale` extension; even VPs without an explicit mutation block carry counter-example enumeration to make the assertion surface adversary-reviewable.
- `## Probe Matrix` — probe enumeration table. The Probe ID column (e.g., `1.a`, `14.b`, `22.c`) provides direct probe→assertion traceability for the F-R89/F-R90 probe-enumeration discipline that emerged from BC↔VP round-trip cycles. Without this section the probe set is implicit in `## Mechanism` prose, which the adversary repeatedly flagged as insufficient.
- `## Harness Location` — direct test-path traceability (file path + test name). Provides implementer + test-writer agents in Phase 3 with explicit harness implementation targets, eliminating one round-trip during TDD red-gate setup.

### Authoritative cross-references

- **L4 template:** `templates/L4-verification-property-template.md` (canonical heading set: `## Property Statement`, `## Source Contract`, `## Proof Method`, `## Proof Harness Skeleton`, `## Feasibility Assessment`, `## Lifecycle`).
- **Audit R1:** `.factory/plans/template-compliance-audit-r1.md` (D-122 trigger — initial heading-name mismatch identification).
- **Audit R2:** `.factory/plans/template-compliance-audit-r2.md` RES-03 (residual heading-name mismatch on all 22 VP files; this v1.0.1 closes RES-03).

### Concurrent dispatches

- **architect RES-01+RES-04:** COMPLETE (commit 0af206a) — input-hash normalization + ARCH-INDEX Tokens column.
- **PO RES-02+RES-05:** COMPLETE (commit 1a09095) — BC VP anchor sweep + PRD §6/§7 column reconciliation.
- **FV RES-03:** this v1.0.1 (audit R2 residual closure for all 22 VP files).

### Content preservation verification

NO content removed. Heading renames in place. New `## Proof Harness Skeleton` / `## Feasibility Assessment` / `## Lifecycle` sections are derived from existing frontmatter fields + existing body sections; they add structure without changing the authoritative machine-consumed values (which remain in frontmatter per the L4 schema).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T18:00:00Z` >= chain high-water `2026-05-17T17:30:00Z` (PRD v1.26.1 — RES-05 concurrent dispatch). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: heading renames + new section additions (`## Proof Method`, `## Proof Harness Skeleton`, `## Feasibility Assessment`, `## Lifecycle`); frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: project-specific extension rationale (above subsection); audit cross-references; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Chose **Option 3 (hybrid)** — template-aligned form with documented project-specific extensions — over Option 1 (template-strict rename + reorganize, which would risk content drift on the Phase-1-emergent probe enumeration discipline) and Option 2 (extension-only documentation without template heading adoption, which would leave the audit RES-03 WARN unresolved). Option 3 satisfies the L4 template heading requirements (all 6 required headings present) AND preserves the Phase-1 verification discipline (probe matrices, mechanism narratives, harness locations) that the adversarial review chain hardened.

---

## §Trace v1.0.2 — F-R105-7 MED: Manifest Pin Refresh v1.1.15 → v1.1.17

**Bump:** v1.0.1 → v1.0.2.
**Predecessor pin:** v1.0.1 (commit 4090d0b — RES-03 VP heading reconciliation).
**Scope of v1.0.2 (NORMATIVE — stale-pin refresh; NO content cascade):**

### Change set (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** body cited `SS-deps-pin-manifest.md v1.1.15` (References §Dependency pins; pre-edit grep).
  - **After:** body cites `SS-deps-pin-manifest.md v1.1.17` at the same location (post-edit grep).
- **SE-17c-d body-scope grep:** pre-§Trace-block body cites of `1.1.15` → 0 remaining; cites of `1.1.17` → 1 (References §Dependency pins).

### Rationale

Architect confirmed (T-128d, commit 0d0c64b) the manifest delta v1.1.15 → v1.1.17 is **STRUCTURAL ONLY** — version-number swap with no content cascade. Therefore the only required downstream action across VP files is the pin-citation refresh; no substantive change to the VP property statement, proof method, mechanism, pre-conditions, post-conditions, counter-examples, probe matrix, or harness location.

### Authoritative cross-references

- **Manifest:** `architecture/SS-deps-pin-manifest.md` v1.1.17 (commit 0d0c64b — T-128d §Trace reconciliation).
- **R105 closure chain:** F-R105-7 MED — manifest pin refresh sweep across 14 pin-citing VP files (the other 8 VP files do not cite the manifest pin and are unchanged in this T-128g dispatch).
- **Concurrent dispatch:** T-128j FV portion — VP-014 title sync to VP-INDEX canonical + VP-007 sister-VP reference reconciliation `VP-TYPES-001` → `VP-013`.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T19:30:00Z` >= chain high-water `2026-05-17T19:00:00Z` (nfr-catalog.md). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: pin refresh `v1.1.15` → `v1.1.17`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Refreshed the pin citation in-scope of T-128g rather than deferring or routing through a parallel dispatch. The architect's structural-only delta classification (T-128d) authorized this mechanical citation refresh without requiring per-VP content review. No tech-debt entries created.

---

## §Trace v1.0.3 — F-R105-13 LOW: VP §References PRD Citation Refresh v1.26 → v1.26.3

**Bump:** v1.0.2 → v1.0.3.
**Predecessor pin:** v1.0.2 (commit 927fcce — T-128g+T-128j FV — F-R105-7/10/11 (pin refresh + title sync + sister-VP reconciliation)).
**Scope of v1.0.3 (NORMATIVE — §References PRD citation refresh; NO content cascade; NO BC-path changes — BC §References already cite canonical sharded `behavioral-contracts/ss-NN/BC-2.SS.NNN.md` paths):**

### Change set 1 — §References PRD Citation Refresh `v1.26` → `v1.26.3` (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** §References cited `prd.md v1.26 §BC-2.01.003 (Dispatch 4 commit 1030c65).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.3 §BC-2.01.003 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (post-edit grep).
- **SE-17c-d body-scope grep:** post-edit `grep -n "prd.md v1.26 " vp-003-body-size-limit.md` → 0 matches; `grep -n "prd.md v1.26.3" vp-003-body-size-limit.md` → 1 match (§References line).
- **BC §References scope:** §References §Source contract entry already cites canonical sharded `behavioral-contracts/ss-01/BC-2.01.003.md` (per BC-INDEX.md v1.2). No BC-path changes required in this dispatch.
- **Historical PRD v1.25 citations in body prose (Source Contract `Traces to (historical)`, Harness Location `to be migrated to`, Proof Harness Skeleton `to be migrated to`, where present):** UNCHANGED — these are explicitly historical predecessor citations pinned to the pre-Dispatch-4 PRD monolith and must not be refreshed.

### Rationale

PO commit b2b378b (T-128k Round-3 PO dispatch) bumped PRD `v1.26.2 → v1.26.3` for F-R105-12 VP alias + GAP-R44-4 closure. Parallel FV dispatch refreshes VP §References to cite the post-bump PRD version, preserving the stale-citation-zero invariant established in F-R105-7 (manifest pin refresh) and F-R105-11 (sister-VP reference reconciliation). Per CLAUDE.md Production-Grade Rule 1: no MVP-driven deferral; mechanical citation refresh executed in-scope of T-128k FV portion rather than left to post-Round-3 cleanup.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.3 (commit b2b378b — F-R105-12 VP alias + GAP-R44-4 closure).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7 — confirms canonical sharded path `ss-01/BC-2.01.003.md` for BC-2.01.003).
- **R105 closure chain:** F-R105-13 LOW — 22-VP §References PRD citation refresh sweep.
- **Concurrent dispatches (T-128k Round 3):**
  - PO: PRD v1.26.2 → v1.26.3 (F-R105-12 + GAP-R44-4) — COMPLETE (commit b2b378b).
  - architect: auth-header interop adjudication — separate scope.
  - BA: L2-INDEX anchor fixes — separate scope.
  - FV: this §Trace (F-R105-13 — 22-VP §References PRD citation refresh).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T20:30:00Z` >= chain high-water `2026-05-17T19:30:00Z` (this VP's prior v1.0.2 §Trace and PRD v1.26.3 frontmatter). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References PRD citation `v1.26` → `v1.26.3`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### SE-17g META audit (NORMATIVE)

Sweep-wide post-edit re-grep across all 22 VP files: `grep -rE "prd\.md v1\.(26[^.]|26\.[012])(\s|\$)" .factory/specs/verification-properties/vp-*.md` → 0 matches. Sweep-wide re-grep for non-sharded BC paths: `grep -rE "behavioral-contracts/BC-[^I]" .factory/specs/verification-properties/vp-*.md` → 0 matches. F-R105-13 closure verified.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Mechanical citation refresh executed in-scope rather than deferred. PRD v1.26.3 cite is valid as of PO commit b2b378b. No tech-debt entries created. Body-prose historical PRD v1.25 citations preserved unchanged per Production-Grade discipline (historical predecessor citations are not stale; refreshing them would erase audit trail).
