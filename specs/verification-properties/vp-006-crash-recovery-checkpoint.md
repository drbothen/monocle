---
document_type: verification-property
level: L4
version: "1.0.5"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T23:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "5a6c176"
traces_to: prd.md
source_bc: BC-2.01.006
module: monocle-runtime
proof_method: manual+mutation
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

# VP-006: Crash Recovery Checkpoint — JSON Write, Offer, Cleanup

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-006 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

During graceful drain (after AppMode `ShuttingDown`, before lock-file
removal), the daemon writes `<runtime_dir>/monocle.recovery.json` atomically
via `tempfile::persist` with a 4-key schema: `pid` (integer ≥ 1),
`shutdown_reason` (closed enum `"graceful" | "signal" | "forced"`),
`last_app_mode` (non-empty string), `shutdown_utc` (ISO 8601 millisecond
regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`). On next start, if
the file exists and prior exit was non-graceful, the daemon logs WARN
`recovery checkpoint found...` and reads state. The 60-second
`Instant::now()`-anchored window from daemon start offers a recovery
message to an attaching TUI client; on ACCEPT/DECLINE/timeout the recovery
file is deleted with the appropriate state transmission. Malformed recovery
files are deleted with WARN log and no UDS offer.

## Source Contract

- **BC (primary):** BC-2.01.006 — Crash Recovery Checkpoint.
- **Postcondition/Invariant:** 4-key schema with typed-field invariants
  (`pid` integer ≥ 1, `shutdown_reason` 3-element closed enum,
  `last_app_mode` non-empty string, `shutdown_utc` ISO 8601 ms regex);
  write-before-lock-remove ordering during drain; 60-second TUI offer
  window measured from `Instant::now()` at daemon start; malformed-file
  handling.
- **Traces to (historical):** BC-DAEMON-006 (PRD v1.25 §BC-DAEMON-006;
  SS-daemon-lifecycle.md v1.0.30 §Crash Recovery; F-R70-2 BC-VP alignment
  closure — millisecond-precision regex was previously tighter than the
  under-specified BC; PRD v1.6 commit 76570ac brought BC into alignment
  with VP).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test + `tokio::time::pause`/`advance` clock + mock UDS client | Bounded — finite probe set; deterministic clock | Drain-time checkpoint write; 4-key schema; TUI ACCEPT/DECLINE/timeout; malformed-file handling; ordering |
| Mutation test (auxiliary) | cargo-mutants | N/A — mutation surface | Type-flip on `pid`; enum-extension drift on `shutdown_reason`; default-init slip on `last_app_mode`; regex looseness on `shutdown_utc` |

## Mechanism

Integration test (harness at `monocle-runtime/tests/crash_recovery.rs` —
files in `<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test
Type column labels this BC `Integration`). The harness uses
`tempfile::TempDir` to isolate `<runtime_dir>` and a `tokio::time::pause()`
+ `tokio::time::advance()` clock to drive the 60-second window
deterministically. A mock UDS client attaches/declines/times-out within the
controlled clock to exercise the recovery-offer state machine.

## Pre-conditions

- Daemon binary supports the synthetic test mode where the recovery
  file is written eagerly on AppMode → `ShuttingDown` (covered by the
  drain code path).
- `tempfile 3`, `serde_json 1`, and `tokio 1` are the project pins (per
  SS-deps-pin-manifest.md v1.1.17).
- `chrono 0.4` is the project pin (per SS-deps-pin-manifest.md v1.1.17)
  for the `shutdown_utc` ISO 8601 millisecond timestamp formatter. The
  recovery checkpoint emits `shutdown_utc` via
  `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")` per
  SS-daemon-lifecycle.md §Drain step 5 and §Trace v1.0.14 F-R72-1
  rationale (cross-field uniformity with `last_hook_ts` and
  `startTimeUtc`).
- Tests use `tempfile::TempDir` to isolate the runtime directory and
  a `tokio::time::pause()` + `tokio::time::advance()` clock to drive
  the 60-second window deterministically.

## Post-conditions

1. Synthetic shutdown signal injected → after the drain code path
   completes, `<temp_runtime>/monocle.recovery.json` exists with content
   matching the 4-key schema and `shutdown_reason == "graceful"`.
2. Pre-created recovery file at daemon start (with an absent or
   stale-pid lock file) → daemon log captures the WARN
   `recovery checkpoint found; prior daemon exited without clean shutdown`.
3. Pre-created recovery file + a mock TUI client attached within 60
   simulated seconds via `tokio::time::advance` → mock TUI receives
   the UDS message `{"type":"recovery_available","last_app_mode":
   "<expected>"}`; after sending `Y`, the recovery file no longer
   exists.
4. Pre-created recovery file + mock TUI sends `N` → recovery file no
   longer exists; mock TUI did NOT receive a state-transmission
   payload after the ACK.
5. Pre-created recovery file + no TUI attaches; 61 simulated seconds
   advance → recovery file no longer exists; daemon log captures
   `WARN: recovery offer expired; deleting checkpoint`.
6. Pre-created recovery file with truncated content (e.g., the closing
   `}` removed) → daemon log captures `WARN: recovery file malformed;
   starting fresh`; recovery file no longer exists; no UDS message sent.
7. Recovery file's `shutdown_utc` value matches the strict ISO 8601
   regex above (no millisecond truncation, no missing `Z` suffix).
8. Recovery file is written BEFORE the lock file is removed during
   drain — the test traces the order of filesystem ops via a `tracing`
   subscriber configured to capture file-write events.
9. **Numeric type for `pid`:** `pid` MUST be a JSON integer ≥ 1 (positive
   process ID per POSIX). Verified by integration test parsing the crash
   recovery JSON and asserting `value["pid"].is_i64() &&
   value["pid"].as_i64().unwrap() >= 1`. Cross-property with VP-002
   §Post-condition 7 numeric-types-and-ranges probe (same `pid integer ≥ 1`
   form applied to the `/status` daemon-state JSON `pid` field).
10. **Enum-value-set for `shutdown_reason`:** `shutdown_reason` MUST be a
    JSON string equal to ONE of the 3 enum literals `"graceful"`,
    `"signal"`, or `"forced"` (per BC-2.01.006 §Invariant 1).
    Verified by integration test asserting
    `["graceful", "signal", "forced"].contains(&value["shutdown_reason"].as_str().unwrap())`.
11. **Non-empty-string for `last_app_mode`:** `last_app_mode` MUST be a
    non-empty JSON string (e.g., `"Running"`, `"ShuttingDown"`,
    `"Crashed"`). Verified by integration test asserting
    `value["last_app_mode"].is_string() &&
    !value["last_app_mode"].as_str().unwrap().is_empty()`.

## Counter-examples

1. Recovery file written AFTER lock-file removal — on a hard SIGKILL
   between the two writes, no recovery file exists; the next daemon
   start has no recovery path. The test asserts the recovery file
   comes first (post-condition 8).
2. Recovery file has an extra key (e.g., `tui_attached: false`) — fails
   the exact-4-key schema assertion.
3. `shutdown_utc` format is `YYYY-MM-DD HH:MM:SS` (space separator
   instead of `T`) — fails the strict ISO 8601 regex.
4. 60-second window started from UDS-readiness rather than daemon start
   — under high startup latency the effective window would be
   stretched; the test asserts the start-time baseline by advancing
   the clock past `start_time + 60s` and asserting expiration.
5. TUI declines (`N`) but state is still transmitted — fails
   post-condition 4.
6. **Probe-matrix exhaustiveness regression (`pid` type drift):** `pid`
   serialized as a JSON string `"12345"` instead of an integer
   `12345` — fails Post-condition 9's `is_i64()` assertion. A regression
   class where an implementer changes `pid` from `u32` to `String`
   somewhere in the recovery checkpoint emitter would slip the exact-key
   schema check (the key name `pid` is unchanged) but fail this
   integer-type probe. `cargo-mutants` mutation-test rationale: this
   probe is the leverage point for the type-mutation surface on the
   serialization path.
7. **Probe-matrix exhaustiveness regression (`shutdown_reason` enum
   drift):** `shutdown_reason` set to a value outside the 3-element
   enum, e.g., `"weird"` or `"unknown"` — fails Post-condition 10's
   closed-set membership assertion. A regression class where an
   implementer adds a 4th `shutdown_reason` variant without updating the
   serialization path's allowed-values guard would slip the exact-key
   schema check but fail this enum-value-set probe. `cargo-mutants`
   mutation-test rationale: this probe is the leverage point for the
   enum-extension-without-guard mutation surface.
8. **Probe-matrix exhaustiveness regression (`last_app_mode` empty
   string):** `last_app_mode` emitted as an empty string `""` — fails
   Post-condition 11's non-empty assertion. A regression class where a
   `Default::default()` slip emits the empty-string default for an
   uninitialized `AppMode` would slip the exact-key schema check but
   fail this non-empty-string probe. `cargo-mutants` mutation-test
   rationale: this probe is the leverage point for the
   default-initialization mutation surface.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 6.a | Synthetic shutdown signal → drain completes | recovery file exists; 4-key schema; `shutdown_reason == "graceful"` |
| 6.b | Pre-created recovery file at start + absent/stale lock | WARN log `recovery checkpoint found...` |
| 6.c | Pre-created recovery file + mock TUI ACK (`Y`) within 60s | UDS `recovery_available` msg sent; recovery file deleted after ACK |
| 6.d | Pre-created recovery file + mock TUI DECLINE (`N`) within 60s | recovery file deleted; NO state-transmission payload sent |
| 6.e | Pre-created recovery file + no TUI; clock advanced 61s | recovery file deleted; WARN `recovery offer expired` log |
| 6.f | Pre-created malformed recovery file (truncated content) | WARN `recovery file malformed` log; file deleted; no UDS msg |
| 6.g | `shutdown_utc` regex assertion | matches `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` strictly |
| 6.h | Filesystem-op order trace via `tracing` | recovery file write event precedes lock-file remove event |
| 6.i | Typed-field probe: `pid is_i64() && >= 1` | invariant holds |
| 6.j | Typed-field probe: `shutdown_reason` in closed enum set | invariant holds (3 literals only) |
| 6.k | Typed-field probe: `last_app_mode` non-empty string | invariant holds |

## Harness Location

- `monocle-runtime/tests/crash_recovery.rs` (integration)
- Test name: `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup`
  (per PRD v1.25 §BC-DAEMON-006, Verification subsection — to be migrated to
  `test_BC_2_01_006_crash_recovery_checkpoint_offer_and_cleanup`).

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: manual+mutation
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_01_006() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/crash_recovery.rs` (integration)
- Test name: `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup`
  (per PRD v1.25 §BC-DAEMON-006, Verification subsection — to be migrated to
  `test_BC_2_01_006_crash_recovery_checkpoint_offer_and_cleanup`).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: manual+mutation` per frontmatter; mechanism documented in `## Mechanism` section. |
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
- Predecessor: monolithic VP-DAEMON-006 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.006.md`.
- BC index: `behavioral-contracts/BC-INDEX.md` v1.5 (commit pending — PO 6A R107 Round 6A finalization; supersedes v1.4 commit bb088a2 — PO 5A R106 Round 5 BC-INDEX dual-accept finalization; supersedes v1.2 commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure).
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.30 (commit pending — architect 5E F-FC-I005 removal + dual-accept consolidation).
- PRD: `.factory/specs/prd.md` v1.26.5 §BC-2.01.006 (Dispatch 4 commit 1030c65; refreshed to v1.26.5 in F-R107-3 / GAP-R46-1 closure, parallel PO 6B dispatch — commit pending; supersedes v1.26.4 commit 01af634 pre-R107 fix burst + v1.26.3 commit b2b378b F-R105-12 closure).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.17.
- Cross-property: VP-002 (numeric-types-and-ranges probe parity on `pid`).

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
  - **Before:** body cited `SS-deps-pin-manifest.md v1.1.15` at 3 locations (pre-edit grep).
  - **After:** body cites `SS-deps-pin-manifest.md v1.1.17` at the same 3 locations (post-edit grep).
- **SE-17c-d body-scope grep:** pre-§Trace-block body cites of `1.1.15` → 0 remaining; cites of `1.1.17` → 3 (Pre-conditions §serde_json pin, §chrono pin, References §Dependency pins).

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
  - **Before:** §References cited `prd.md v1.26 §BC-2.01.006 (Dispatch 4 commit 1030c65).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.3 §BC-2.01.006 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (post-edit grep).
- **SE-17c-d body-scope grep:** post-edit `grep -n "prd.md v1.26 " vp-006-crash-recovery-checkpoint.md` → 0 matches; `grep -n "prd.md v1.26.3" vp-006-crash-recovery-checkpoint.md` → 1 match (§References line).
- **BC §References scope:** §References §Source contract entry already cites canonical sharded `behavioral-contracts/ss-01/BC-2.01.006.md` (per BC-INDEX.md v1.2). No BC-path changes required in this dispatch.
- **Historical PRD v1.25 citations in body prose (Source Contract `Traces to (historical)`, Harness Location `to be migrated to`, Proof Harness Skeleton `to be migrated to`, where present):** UNCHANGED — these are explicitly historical predecessor citations pinned to the pre-Dispatch-4 PRD monolith and must not be refreshed.

### Rationale

PO commit b2b378b (T-128k Round-3 PO dispatch) bumped PRD `v1.26.2 → v1.26.3` for F-R105-12 VP alias + GAP-R44-4 closure. Parallel FV dispatch refreshes VP §References to cite the post-bump PRD version, preserving the stale-citation-zero invariant established in F-R105-7 (manifest pin refresh) and F-R105-11 (sister-VP reference reconciliation). Per CLAUDE.md Production-Grade Rule 1: no MVP-driven deferral; mechanical citation refresh executed in-scope of T-128k FV portion rather than left to post-Round-3 cleanup.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.3 (commit b2b378b — F-R105-12 VP alias + GAP-R44-4 closure).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7 — confirms canonical sharded path `ss-01/BC-2.01.006.md` for BC-2.01.006).
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

---

## §Trace v1.0.4 — F-R106-10 HIGH: SS-daemon-lifecycle Pin Refresh v1.0.25 → v1.0.30

**Bump:** v1.0.3 → v1.0.4.
**Predecessor pin:** v1.0.3 (commit 932f4e0 — T-128k FV F-R105-13 LOW 22-VP §References PRD citation refresh).
**Scope of v1.0.4 (NORMATIVE — mechanical pin refresh; NO content cascade; NO BC-path changes):**

### Change set (NORMATIVE)

- **SE-17f §Source Contract `Traces to (historical)` line:** `SS-daemon-lifecycle.md v1.0.25 ...` → `SS-daemon-lifecycle.md v1.0.30 ...` (per-VP section name preserved verbatim).
- **SE-17f §References Architecture line:** `Architecture: \`architecture/SS-daemon-lifecycle.md\` v1.0.25 ... (commit 18fe265).` → `Architecture: \`architecture/SS-daemon-lifecycle.md\` v1.0.30 ... (commit pending — architect 5E F-FC-I005 removal + dual-accept consolidation).`
- **SE-17f inline `arch v1.0.NN` body-prose cites (where present in this VP):** refreshed to `arch v1.0.30`.

### SE-17c-d body-scope grep

- Post-edit `grep -nE "SS-daemon-lifecycle.md v1\.0\.25" <this-vp>` → 0 matches outside §Trace history blocks (the only remaining `v1.0.25` cites are in earlier §Trace SE-17f BEFORE evidence as preserved historical predecessor evidence per SE-17g audit-trail discipline).
- Post-edit `grep -nE "v1\.0\.30" <this-vp>` → 2+ matches (§Source Contract Traces-to + §References Architecture, plus any inline `arch v1.0.30` cites in body prose).

### Rationale

Architect 5E is bumping SS-daemon-lifecycle v1.0.29 → v1.0.30 in parallel (R106 Round 5) for F-R106 F-FC-I005 fabricated-FC-ID removal and dual-accept consolidation. The architectural delta is **STRUCTURAL** (FC-ID removal — the architecture sections referenced by this VP are unchanged content). Therefore the only required downstream action across pin-citing VP files is the pin-citation refresh; no substantive change to the VP property statement, proof method, mechanism, pre-conditions, post-conditions, counter-examples, probe matrix, or harness location.

The pin refresh is bundled with the broader R106 Round 5D FV dispatch (10 pin-citing VPs + VP-009 ADR-0005 dual-accept expansion + VP-INDEX 4-fix cascade). Cross-dispatch coordination with architect 5E handled via explicit "commit pending" annotation in §References — this will resolve to a concrete SHA during final state-manager pass after all parallel dispatches converge.

### Authoritative cross-references

- **Architecture:** `architecture/SS-daemon-lifecycle.md` v1.0.30 (commit pending — architect 5E dispatch).
- **R106 closure chain:** F-R106-10 HIGH — 10-VP SS-daemon-lifecycle pin refresh sweep (this VP is one of 10 pin-citing files).
- **Concurrent dispatches (R106 Round 5):**
  - PO 5A: BC + BC-INDEX dual-accept finalization — separate scope.
  - PO 5B: PRD + supplements — separate scope.
  - PO 5C: product-brief — separate scope.
  - FV 5D: this dispatch (VP-009 expansion + 10-VP pin sweep + VP-INDEX cascade).
  - Architect 5E: ADR-0005 path normalization + SS-daemon-lifecycle v1.0.29 → v1.0.30 — separate scope.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T22:30:00Z` >= chain high-water `2026-05-17T22:10:00Z` (BC-2.01.009 v1.0.3 frontmatter timestamp; cross-dispatch BC reference). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: pin refresh `v1.0.25` → `v1.0.30`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Mechanical pin refresh executed in-scope rather than deferred. Architect 5E's structural-only delta (FC-ID removal) authorized this mechanical citation refresh without requiring per-VP content review. No tech-debt entries created. Historical v1.0.25 cite preserved in §References "supersedes" annotation per audit-trail discipline.

---

## §Trace v1.0.5 — F-R107-4 HIGH + GAP-R46-1 HIGH + F-R107-8 (part 2): VP §References PRD Cite Refresh v1.26.3 → v1.26.5 + Active BC-INDEX Cite Addition v1.5

**Bump:** v1.0.4 → v1.0.5.
**Predecessor pin:** v1.0.4 (commits 932f4e0 / 7b8d6e8 — prior R105/R106 FV sweeps).
**Scope of v1.0.5 (NORMATIVE — mechanical §References citation refresh + active BC-INDEX cite addition; NO content cascade):**

### Change set 1 — §References PRD Citation Refresh `v1.26.3` → `v1.26.5` (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** §References cited `prd.md v1.26.3 §BC-2.01.006 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.5 §BC-2.01.006 (Dispatch 4 commit 1030c65; refreshed to v1.26.5 in F-R107-3 / GAP-R46-1 closure, parallel PO 6B dispatch — commit pending; supersedes v1.26.4 commit 01af634 pre-R107 fix burst + v1.26.3 commit b2b378b F-R105-12 closure).` (post-edit grep).
- **SE-17c-d body-scope grep (active §References scope):** post-edit `grep -nE "prd\.md.* v1\.26\.[0-4]" vp-006-crash-recovery-checkpoint.md` outside §Trace history blocks → 0 matches; `grep -n "prd.md\` v1.26.5" vp-006-crash-recovery-checkpoint.md` outside §Trace history blocks → 1 match (§References PRD line).
- **Historical PRD citations in §Trace history blocks:** UNCHANGED per SE-17g audit-trail-preservation discipline (predecessor citations document state-at-the-time of each historical bump and must not be refreshed; refreshing them would erase audit trail).

### Change set 2 — §References Active BC-INDEX Cite Addition v1.5 (NORMATIVE)

- **Rationale for active-cite ADDITION (not §Trace history rewrite):** F-R107-8 part 2 identifies stale BC-INDEX v1.2 cites in 22 VPs' `BC §References scope` evidence text inside §Trace v1.0.3 (F-R105-13 history blocks). Per SE-17g audit-trail-preservation discipline, §Trace history blocks are append-only — they document state at the time of the bump and are immutable. The production-grade closure is to ADD an active §References BC-INDEX cite at the current v1.5 target, which makes the v1.5 cite live-authoritative and demotes the v1.2 mention in §Trace v1.0.3 to historical snapshot evidence (its correct semantic). Before this dispatch, no VP had an active BC-INDEX §References cite — the only BC-INDEX version mentions were in §Trace history. Adding the active cite closes F-R107-8 part 2 durably without violating SE-17g.
- **SE-17f before/after evidence:**
  - **Before:** active §References had no `BC index:` line (only `Source contract:` cited the sharded BC path without referencing BC-INDEX version). Pre-edit grep `grep -nE "BC-INDEX" vp-006-crash-recovery-checkpoint.md` outside §Trace blocks → 0 matches.
  - **After:** active §References gained `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.5 (commit pending — PO 6A R107 Round 6A finalization; supersedes v1.4 commit bb088a2 — PO 5A R106 Round 5; supersedes v1.2 commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure).` Post-edit grep `grep -nE "BC-INDEX\.md.* v1\.5" vp-006-crash-recovery-checkpoint.md` outside §Trace blocks → 1 match (§References BC index line).
- **No BC-path or BC-version changes required:** §Source contract entry already cites canonical sharded `behavioral-contracts/ss-01/BC-2.01.006.md` (per BC-INDEX.md v1.5 confirmation). No BC-path edits in this dispatch.

### Rationale

R107 Round 6C (FV scope) closes three findings in coordinated parallel dispatch:

- **F-R107-4 HIGH (VP-009-specific):** VP-009 cited ADR-0005 v1.0.1 in 3 locations (§Source Contract line 89; active §References line 431; §Trace v1.0.4 Authoritative cross-references line 660). Current ADR-0005 is at v1.0.2 (frontmatter `version: "1.0.2"` confirmed at audit time; commit 03a4c57 — F-R106-14 ADR-0005 inputs path normalization + F-R106-7 F-FC-I005 fabrication removal). All 3 cites refreshed to v1.0.2 in this dispatch.
- **GAP-R46-1 HIGH (all-22-VPs):** VP-INDEX was pre-fixed in commit 01af634 (PRD pin v1.26.3 → v1.26.4 pre-R107 fix burst) but the 22 individual VP files' active §References still cited the stale v1.26.3. PO 6B is bumping PRD v1.26.4 → v1.26.5 in parallel; this FV dispatch refreshes all 22 VPs' active §References to the post-PO-6B v1.26.5 target.
- **F-R107-8 part 2 (all-22-VPs):** 22 VPs had stale BC-INDEX v1.2 mentions inside §Trace v1.0.3 history blocks. Per SE-17g audit-trail-preservation, those §Trace mentions are historical snapshots and must not be edited. The durable closure is to ADD active §References BC-INDEX cites at the current v1.5 target (post-PO-6A), making v1.5 the live-authoritative cite and demoting the v1.2 mentions in §Trace v1.0.3 to historical snapshot evidence (their correct SE-17g semantic). PO 6A is bumping BC-INDEX v1.4 → v1.5 in parallel; this FV dispatch targets the post-PO-6A v1.5 version.

Per CLAUDE.md Production-Grade Default Rule 1+5: mechanical citation refresh + durable cite addition executed in-scope of R107 Round 6C rather than deferred. No tech-debt entries created. Historical §Trace block citations preserved unchanged per SE-17g.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.5 (commit pending — PO 6B R107 Round 6B PRD + supplements dispatch; supersedes v1.26.4 commit 01af634 pre-R107 fix burst).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.5 (commit pending — PO 6A R107 Round 6A BC scope dispatch; supersedes v1.4 commit bb088a2 — PO 5A R106 Round 5 BC-INDEX dual-accept finalization).
- **R107 closure chain:** F-R107-4 HIGH (VP-009 ADR-0005 pin refresh — VP-009-only); GAP-R46-1 HIGH (22-VP PRD cite refresh sweep); F-R107-8 part 2 (22-VP active BC-INDEX cite addition).
- **Concurrent dispatches (R107 Round 6):**
  - PO 6A: BC + BC-INDEX scope (BC-INDEX v1.4 → v1.5) — separate scope.
  - PO 6B: PRD + supplements (PRD v1.26.4 → v1.26.5) — separate scope.
  - FV 6C: this dispatch (VP-009 ADR-0005 pin refresh + 22-VP PRD cite refresh + 22-VP BC-INDEX active cite addition + VP-INDEX cascade).
  - Architect 6D: SS-forward-compatibility scope — separate scope.
  - BA 6E: L2-INDEX scope — separate scope.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T23:00:00Z` >= chain high-water `2026-05-17T22:50:00Z` (VP-INDEX v1.4 timestamp; pre-R107 fix burst commit 01af634). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References PRD citation `v1.26.3` → `v1.26.5`; §References BC index cite ADDITION at v1.5; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Mechanical citation refresh + durable cite-addition executed in-scope rather than deferred. PRD v1.26.5 cite is the post-PO-6B target (commit pending — will resolve to concrete SHA during final state-manager pass after parallel dispatches converge). BC-INDEX v1.5 cite is the post-PO-6A target (commit pending — same resolution). No tech-debt entries created. §Trace history blocks preserved unchanged per SE-17g — historical predecessor citations are not stale; refreshing them would erase audit trail.

