---
document_type: verification-property
level: L4
version: "1.0.3"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T20:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "7c094e3"
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
  SS-daemon-lifecycle.md v1.0.25 §Crash Recovery; F-R70-2 BC-VP alignment
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
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26.3 §BC-2.01.006 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
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
