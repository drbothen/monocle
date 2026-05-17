---
document_type: verification-property
level: L4
version: "1.0.1"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T18:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.01.004
module: monocle-runtime
proof_method: manual
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

# VP-004: Graceful Shutdown — 10-Second Drain + 5-Code POSIX Exit Taxonomy

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-004 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

On SIGTERM, SIGINT, or authenticated `POST /shutdown`, the daemon transitions
AppMode to `ShuttingDown` within < 10 ms. After transition, new POSTs to
`/hooks/*` return HTTP 503 with `Retry-After: 10` and body
`{"error":"daemon_shutting_down"}`; `/healthz` returns 503 with
`{"status":"shutting_down"}`; `/status` continues to serve normally. In-flight
requests drain bounded by `tokio::time::timeout(Duration::from_secs(10),
drain_inflight())` after which the daemon proceeds to lock-file removal and
exits with one of 5 deterministic POSIX-correct exit codes per trigger cause
(`0`, `130`, `143`, `2`, `1`). `POST /shutdown` without valid auth returns
HTTP 401.

## Source Contract

- **BC (primary):** BC-2.01.004 — Graceful Shutdown (10-Second Drain).
- **BCs (partial coverage):** BC-2.01.005 (lock-file removal step
  post-drain), BC-2.01.009 (auth taxonomy on `/shutdown` route).
- **Postcondition/Invariant:** AppMode transition latency bound,
  503-with-`Retry-After` invariant on `/hooks/*`, drain-completion
  bound, 5-code POSIX exit taxonomy
  (0/130/143/2/1 — Obs-R70-2 + F-R70-3 closure), auth-on-`/shutdown`
  cross-property.
- **Traces to (historical):** BC-DAEMON-004 (PRD v1.25 §BC-DAEMON-004;
  SS-daemon-lifecycle.md v1.0.25 §Shutdown Signal Handling, §Drain,
  §Hard Shutdown).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test + tokio synthetic-signal harness | Bounded — finite per-trigger probe set | AppMode transition latency, 503 + `Retry-After`, drain bound, exit-code matrix, auth-on-shutdown |
| Synthetic signal injection | tokio oneshot::channel + `trigger_signal` recording | Bounded | Per-trigger exit-code determinism (SIGTERM-vs-SIGINT distinguishability) |
| Cross-property assertion | manual review | N/A | VP-001 + VP-002 + VP-005 + VP-009 cross-property reciprocations |

## Mechanism

Integration test (harness at `monocle-runtime/tests/graceful_shutdown.rs`
and `monocle-runtime/tests/daemon_lifecycle.rs` — files in `<crate>/tests/`
are cargo integration tests; PRD v1.25 §7 RTM Test Type column labels this
BC `Integration`). The harness uses a test-only `oneshot::channel` to
inject synthetic shutdown signals tagged with `trigger_signal: SIGTERM | SIGINT
| PostShutdown` so the harness can distinguish the 130-vs-143-vs-2 exit code
paths without real OS-signal delivery. Drain completion is bounded by
asserting `elapsed < 11 seconds` for the over-budget scenario and
`exit_code == <expected-per-trigger>`.

## Pre-conditions

- Daemon running with a valid lock file.
- `tokio::signal::unix::signal(SignalKind::terminate())` is the SIGTERM
  receiver; `tokio::signal::ctrl_c()` is the SIGINT receiver. The
  signal type that triggered hard shutdown is recorded for exit-code
  selection (per arch v1.0.25 §Hard Shutdown step 6d).
- A test-only `oneshot::channel` is used to inject a synthetic shutdown
  signal (avoiding real OS signal delivery in integration tests).
  Test-harness wrappers inject SIGTERM-flavored and SIGINT-flavored
  synthetic signals to exercise the 130-vs-143 distinction without real
  OS-signal delivery.
- `axum 0.8` and `tokio 1` are the project pins (per
  SS-deps-pin-manifest.md v1.1.15); `tower` is a transitive dependency of
  `axum 0.8` (no direct workspace pin).

## Post-conditions

1. Synthetic shutdown signal injected → AppMode is `ShuttingDown` within
   10 ms (asserted via a `tokio::sync::watch` channel exposing the
   current mode).
2. POST `/hooks/pre-tool-use` after AppMode transition → HTTP 503 with
   header `Retry-After: 10` (exact integer value) and body
   `{"error":"daemon_shutting_down"}`.
3. `GET /healthz` during drain → HTTP 503 + `{"status":"shutting_down"}`.
4. `GET /status` with valid auth during drain → HTTP 200 + full 10-field
   body (read-only continues).
5. With one synthetic in-flight `/hooks/*` POST that holds a 5-second
   sleep, the drain completes within 10 seconds and the daemon exits
   cleanly with exit code `0` (deterministic; graceful drain success).
6. **5-code POSIX exit taxonomy probe matrix (per PRD v1.25 §BC-DAEMON-004
   canonical test vectors; Obs-R70-2 closure):**

   | Scenario | Synthetic input | Expected exit code |
   |----------|-----------------|--------------------|
   | Clean drain | All in-flight POSTs complete within 10s | `0` |
   | SIGINT hard-kill during drain | Second synthetic-SIGINT delivered during drain | `130` (POSIX 128+2) |
   | SIGTERM hard-kill during drain | Second synthetic-SIGTERM delivered during drain | `143` (POSIX 128+15) |
   | Admin forced-stop during drain | Second authenticated `POST /shutdown` during drain | `2` (monocle-specific) |
   | Startup failure | `DaemonStartError::RuntimeDirUnresolvable` (cross-property with VP-005 post-condition 5) | `1` |

   Each row is a deterministic single-code assertion. No tolerance range.
   The over-budget scenario (in-flight 15-second sleep with no second
   signal) reaches the 10-second drain timeout and exits `143`-or-`130`
   depending on which signal originally triggered drain — NOT a tolerance,
   but a per-cause deterministic outcome captured by the test-harness's
   recorded `trigger_signal` field. The harness asserts `elapsed < 11
   seconds` AND `exit_code == <expected-per-trigger>`.
7. `POST /shutdown` with no auth header → HTTP 401 +
   `{"error":"missing_auth_token"}` (VP-009 cross-property).

## Counter-examples

1. New hook POSTs during drain return HTTP 200 (drain logic not
   short-circuiting accepts) — fails post-condition 2.
2. `Retry-After` header omitted or set to a different value (e.g., `5`) —
   fails the exact-value assertion.
3. `/status` blocks during drain (returns no response or 503) — fails
   post-condition 4.
4. Drain timeout not enforced (in-flight 15-second sleep allowed to
   complete) — fails the 10-second bound; the test must assert
   `elapsed < 11 seconds` for the over-budget scenario.
5. `POST /shutdown` accepted without auth — fails post-condition 7
   (auth middleware must run on this route).
6. **Exit code 130 returned for a SIGTERM hard-kill scenario** — fails
   the POSIX 128+N convention (128+15 = 143 for SIGTERM, not 130).
   External monitoring (systemd `Restart=on-failure`, k8s
   `terminationGracePeriodSeconds`) would misinterpret the trigger.
   The test harness asserts `exit_code == 143` for the SIGTERM
   second-signal path and `exit_code == 130` for the SIGINT
   second-signal path; conflating the two — i.e., returning `130` for
   both — must be caught (Obs-R70-2 + F-R70-3 closure).
7. **Exit code 2 collides with startup failure** — if implementer sets
   the admin-forced-stop exit to `1` (overlapping with startup
   failure), monitoring systems cannot distinguish operator-initiated
   force-stop from daemon-start failure. The probe matrix asserts
   `exit_code == 2` for the second-`POST /shutdown` path and
   `exit_code == 1` for the `RuntimeDirUnresolvable` startup-failure
   path; identical codes for these two distinct triggers must fail.
8. **Single-binary exit-code (any non-zero accepted) regression** —
   the prior-burst v1.5.1 tolerance (exit code `0` OR `130` for the
   over-budget 15-second scenario) is RETIRED. A harness that accepts
   any non-zero exit code as "hard-killed pass" without distinguishing
   130 vs 143 vs 2 vs 1 fails the new per-cause deterministic-outcome
   assertion. This counter-example sketch is the formal recurrence
   guard against the over-budget BC-vs-VP drift Obs-R70-2 documented.

## Probe Matrix

| Probe | Setup | Expected status | Expected outcome |
|-------|-------|-----------------|------------------|
| 4.a | Synthetic SIGTERM injected | N/A | AppMode → `ShuttingDown` within 10 ms |
| 4.b | POST `/hooks/*` post-drain entry | 503 | `Retry-After: 10` header; body `{"error":"daemon_shutting_down"}` |
| 4.c | GET `/healthz` during drain | 503 | `{"status":"shutting_down"}` (cross VP-001) |
| 4.d | GET `/status` (valid auth) during drain | 200 | full 10-field body (cross VP-002) |
| 4.e | In-flight 5s sleep + clean drain | exit `0` | drain completes within 10s |
| 4.f | Second synthetic-SIGINT during drain | exit `130` | POSIX 128+2; `elapsed < 11s` |
| 4.g | Second synthetic-SIGTERM during drain | exit `143` | POSIX 128+15; `elapsed < 11s` |
| 4.h | Second authenticated `POST /shutdown` during drain | exit `2` | monocle-specific admin force-stop |
| 4.i | `DaemonStartError::RuntimeDirUnresolvable` start | exit `1` | startup-failure code (cross VP-005) |
| 4.j | `POST /shutdown` with no auth header | 401 | `{"error":"missing_auth_token"}` (cross VP-009) |

## Harness Location

- `monocle-runtime/tests/graceful_shutdown.rs` (primary HTTP 503 / `Retry-After`
  probes)
- `monocle-runtime/tests/daemon_lifecycle.rs` (exit-code 5-code POSIX taxonomy
  probes per PRD v1.25 §BC-DAEMON-004 canonical test vectors)
- Test names:
  - `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests`
  - `test_BC_DAEMON_004_exit_codes_posix_distinct`
  (per PRD v1.25 §BC-DAEMON-004, Verification subsection — to be migrated to
  `test_BC_2_01_004_*` post BC renumber propagation into source.)

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: manual
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_01_004() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/graceful_shutdown.rs` (primary HTTP 503 / `Retry-After`
  probes)
- `monocle-runtime/tests/daemon_lifecycle.rs` (exit-code 5-code POSIX taxonomy
  probes per PRD v1.25 §BC-DAEMON-004 canonical test vectors)
- Test names:
  - `test_BC_DAEMON_004_graceful_shutdown_503_on_new_requests`
  - `test_BC_DAEMON_004_exit_codes_posix_distinct`
  (per PRD v1.25 §BC-DAEMON-004, Verification subsection — to be migrated to
  `test_BC_2_01_004_*` post BC renumber propagation into source.)

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: manual` per frontmatter; mechanism documented in `## Mechanism` section. |
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
- Predecessor: monolithic VP-DAEMON-004 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.004.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.004 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-001 (`/healthz` 503), VP-002 (`/status` drain-state
  read-only), VP-005 (lock-file removal post-drain + startup-failure exit
  `1`), VP-009 (auth-on-shutdown taxonomy).

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
