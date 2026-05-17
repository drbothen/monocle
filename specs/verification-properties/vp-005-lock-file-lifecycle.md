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
source_bc: BC-2.01.005
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

# VP-005: Lock File Lifecycle — Atomic Create, Pid-Liveness Gate, Mode 0o600, Cleanup, 4-Path Resolution

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-005 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

The daemon resolves `<runtime_dir>` via an ordered 4-path chain
(`MONOCLE_RUNTIME_DIR` env → `ProjectDirs::runtime_dir()` →
`ProjectDirs::data_local_dir()` → fail-fast `RuntimeDirUnresolvable`). On
start it atomically creates `<runtime_dir>/monocle.lock` via
`tempfile::persist` with mode `0o600`; the containing `<runtime_dir>` is
created with mode `0o700` (defense-in-depth). On start with an existing lock
file the daemon checks pid-liveness via `kill(pid, 0)` and either exits 1
(live) or proceeds with stale-pid recovery (ESRCH). On clean shutdown the
lock file AND `monocle.sock` are removed. Naked `std::fs::write` for the
lock-file path is forbidden (source-grep negative assertion). The asymmetry
with BC-2.03.003 `HomeUnresolvable` is intentional.

## Source Contract

- **BC (primary):** BC-2.01.005 — Lock File Atomic Lifecycle (Create + Pid
  Check + Cleanup).
- **BCs (partial coverage):** BC-2.01.010 (joint mode-and-content assertion
  via `contract_version` first key), BC-2.03.003 (asymmetry rationale with
  `HomeUnresolvable`), BC-2.01.008 (defense-in-depth pairing with `0o700`
  runtime-dir mode).
- **Postcondition/Invariant:** 4-path resolution-chain ordering (EC-057,
  EC-058, EC-059); atomic-create via `tempfile::persist`; mode `0o600` for
  lock file; mode `0o700` for runtime dir on creation; pid-liveness gate;
  clean-shutdown cleanup; `RuntimeDirUnresolvable` fail-fast → exit 1.
- **Traces to (historical):** BC-DAEMON-005 (PRD v1.25 §BC-DAEMON-005;
  SS-daemon-lifecycle.md v1.0.25 §Start Sequence + §Hard Shutdown;
  F-R70-1 closure — hybrid runtime-dir resolution chain disposition (c);
  F-R88-2 wording correction landed in PRD v1.17 commit 27e663c and
  carried forward verbatim into PRD v1.25 commit 7735c84 per C-R90-1).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test + `tempfile::TempDir` + `temp-env` env-var isolation | Bounded — finite probe set per path | All 4 resolution-chain paths; mode-bit assertions; pid-liveness gate; cleanup |
| Mutation test (auxiliary) | cargo-mutants | N/A — mutation surface | `0o600`, `0o700`, `kill(pid, 0)`, chain ordering all mutation surfaces |
| Source-grep (structural) | ripgrep | N/A — static | `tempfile::persist` present; no `std::fs::write` for `monocle.lock`; chain-ordering preserved |

## Mechanism

Integration test (primary; harness at `monocle-runtime/tests/lock_file_lifecycle.rs`
— files in `<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM
Test Type column labels this BC `Integration`); mutation-test (auxiliary —
the `0o600` lock-file mode value, the `0o700` runtime-dir mode value
(defense-in-depth pairing per Post-condition 9 / BC-2.01.005 Postcondition
8), the `kill(pid, 0)` gate, and the 4-path resolution-chain ordering are
mutation surfaces). The harness uses `tempfile::TempDir` to isolate
`<runtime_dir>` per test AND mocks the `directories::ProjectDirs` API
(via dependency injection or `temp-env`-controlled env vars) to exercise
paths (a)-(d) deterministically.

## Pre-conditions

- Runtime directory `<runtime_dir>` is resolved per the 4-path chain. Tests
  use `tempfile::TempDir` to isolate `<runtime_dir>` per test AND mock the
  `directories::ProjectDirs` API (via dependency injection or
  `temp-env`-controlled env vars) to exercise paths (a)-(d)
  deterministically.
- `directories 6` (per SS-deps-pin-manifest.md v1.1.17) is the project pin
  for `ProjectDirs::runtime_dir()` and `ProjectDirs::data_local_dir()`.
- `tempfile 3` is the project pin (per SS-deps-pin-manifest.md v1.1.17).
- `nix 0.30` is the project pin (per SS-deps-pin-manifest.md v1.1.17) for
  the pid-liveness probe; the test asserts
  `nix::sys::signal::kill(Pid::from_raw(pid), None)` per BC-2.01.005
  postcondition 3.
- `temp-env ^0.3` is the project pin for `MONOCLE_RUNTIME_DIR` env
  isolation (shared with VP-021 (renumbered from VP-ENGINE-002-ERR per VP-INDEX.md §Renumbering Appendix), see SS-03 VPs in Dispatch 5b).

## Post-conditions

1. Fresh start with no lock file (after successful runtime-dir resolution
   via any of paths a/b/c) → lock file created at
   `<resolved_runtime_dir>/monocle.lock`; `stat().mode() & 0o777 == 0o600`;
   JSON content begins with `{"contract_version":1,` (cross-property with
   VP-010).
2. Daemon already running (mock: PID file contains current test
   process PID, which is alive) → daemon start returns exit code 1;
   stderr contains the substring `daemon already running at pid=`.
3. Stale lock file (PID file contains `1` or another known-dead PID
   for the test environment, or contains a PID that `kill(0)` ESRCHes)
   → daemon start succeeds; the old file is replaced; the new file
   has the live daemon's PID.
4. Daemon graceful shutdown via synthetic SIGTERM → after drain
   completes, `<resolved_runtime_dir>/monocle.lock` does not exist
   (`Path::exists()` returns `false`). Cross-property with VP-004
   §Mechanical property item 5 (drain completion / lock-file lifecycle
   interaction).
5. **4-path resolution chain probe matrix (per PRD v1.25 §BC-DAEMON-005
   canonical test vectors EC-057/058/059; F-R70-1 closure):**

   | Probe | Setup | Expected resolution path | Expected log | Expected outcome |
   |-------|-------|---------------------------|--------------|------------------|
   | 5.a | `MONOCLE_RUNTIME_DIR=<temp_dir>` set; ProjectDirs mocked to either real OS values or `None` | (a) env override | `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var` | daemon uses `<temp_dir>` as runtime dir; lock file created there |
   | 5.b | `MONOCLE_RUNTIME_DIR` unset; ProjectDirs mocked so `runtime_dir()` returns `Some(<temp_dir>)` | (b) ProjectDirs::runtime_dir() | `INFO: runtime_dir from ProjectDirs::runtime_dir()` | daemon uses ProjectDirs `runtime_dir` path |
   | 5.c | `MONOCLE_RUNTIME_DIR` unset; ProjectDirs mocked so `runtime_dir()` returns `None` AND `data_local_dir()` returns `<temp_dir>` (EC-057 macOS pattern) | (c) ProjectDirs::data_local_dir() | `INFO: runtime_dir fallback to data_local_dir (platform: <os>)` | daemon uses `data_local_dir` path; happy-path on macOS; best-effort resolution on Windows per PRD §8.7 (Phase 1 CI does not formally validate Windows per NFR-008) |
   | 5.d | `MONOCLE_RUNTIME_DIR` unset; `ProjectDirs::new()` mocked to return `None` (EC-059 full-fail pattern) | (d) fail-fast `RuntimeDirUnresolvable` | (no `INFO` log; `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`) | daemon exits 1; NO lock file created at any path; cross-property with VP-004 post-condition 6 exit code `1` |

6. Daemon graceful shutdown → `<resolved_runtime_dir>/monocle.sock`
   does not exist (`Path::exists()` returns `false`).
7. Source-grep over `monocle-runtime/src/lock.rs`:
   - `tempfile::persist` appears at least once.
   - `std::fs::write` does NOT appear for the lock file path
     (an exception list may permit `std::fs::write` for non-lock paths,
     e.g., the recovery checkpoint file via separate path; the test
     restricts the negative match to lines mentioning `"monocle.lock"`).
8. Source-grep over `monocle-runtime/src/start.rs` (or wherever
   `resolve_runtime_dir` is implemented): the resolution chain order
   `MONOCLE_RUNTIME_DIR` → `runtime_dir()` → `data_local_dir()` →
   `RuntimeDirUnresolvable` is preserved (the chain MUST evaluate env
   override first; flipping the order would silently break the
   operator escape hatch).
9. **Runtime-dir mode 0o700 owner-only enforcement (per PRD v1.25
   §BC-DAEMON-005 Postcondition 8 + EC-052 + arch v1.0.25 §Start
   Sequence step 1; F-R75-1 VP-side closure + F-R79-3 PRD-side
   lift_invariants_to_bcs closure):** on runtime-dir creation
   (resolution chain paths b or c when the directory is absent prior to
   `monocle daemon start`), `stat(&runtime_dir).mode() & 0o777 == 0o700`
   (owner-only access). Verification: integration test creates a fresh,
   non-existent runtime_dir path, starts the daemon, then reads the
   directory's mode bits and asserts equality with `0o700`. Probe matrix
   row 5.e below covers this case directly. When the runtime_dir already
   exists from a prior start (idempotent restart path), the daemon MUST
   NOT modify the mode bits of the existing directory; the assertion
   applies only to the newly-created-this-start path. Cross-property
   with VP-008 (the auth token written into `<runtime_dir>/monocle.lock`
   is protected by both the lock-file 0o600 mode AND the containing
   directory's 0o700 mode — defense-in-depth).

## Counter-examples

1. Lock file written via naked `std::fs::write` — would expose a
   partial-write window between truncate and content-write; the
   source-grep negative assertion catches this. (This is also a
   semgrep rule per SS-conventions-anti-patterns.md §Semgrep Rules.)
2. Lock file written with mode `0o644` (group/other readable) — fails
   the `0o600` mode assertion; this is critical because the auth token
   is in the lock file and group/other readability would expose it to
   other OS users.
3. Stale-pid handling skipped (daemon refuses to start because lock
   file exists, without checking liveness) — fails post-condition 3.
4. Lock file not removed on clean shutdown — fails post-condition 4;
   subsequent starts would exercise the stale-pid path unnecessarily.
5. `tempfile::persist` argument `dest_path` set to a path that differs
   from the canonical `<resolved_runtime_dir>/monocle.lock` — fails the
   canonical-path assertion in post-condition 1.
6. **`ProjectDirs::runtime_dir() == None` on macOS triggers fail-fast
   without consulting `data_local_dir()`** — fails post-condition 5.c
   probe (the EC-057 macOS happy path); the daemon would refuse to
   start on the primary-target platform (NFR-008). This is the F-R70-1
   recurrence guard.
7. **Resolution-chain order flipped** (e.g., `ProjectDirs::runtime_dir()`
   evaluated before `MONOCLE_RUNTIME_DIR`) — silently breaks the
   operator escape hatch; an operator setting `MONOCLE_RUNTIME_DIR`
   would have their override ignored on Linux where `runtime_dir()`
   returns `Some`. Post-condition 8 source-grep assertion catches this.
8. **`DaemonStartError::RuntimeDirUnresolvable` raised when only path
   (b) returned `None`** (e.g., on macOS where `runtime_dir()` returns
   `None` but `data_local_dir()` returns `Some`) — fails the
   chain-coverage assertion; path (c) MUST be consulted before the
   fail-fast path (d) fires.
9. **`E-DAEMON-004 RuntimeDirUnresolvable` exit code other than `1`** —
   the fail-fast path MUST exit `1` (startup-failure code per
   VP-004 exit-code taxonomy item 6.5); a `0` or `143` would
   confuse monitoring tools. Cross-property assertion.
10. **Runtime dir created with umask-default mode (F-R75-1 attack
    surface):** implementer uses `std::fs::create_dir_all(&runtime_dir)?`,
    which creates the directory honoring the process umask (typical
    default ~0o022, yielding mode bits ~0o755 — world-readable). VP probe
    5.e fails because `stat(&runtime_dir).mode() & 0o777 != 0o700`.
    Information leak: other OS users can `stat` the runtime dir and
    enumerate monocle's paths (`/monocle.lock`, `/monocle.sock`,
    `/monocle.recovery.json`), aiding reconnaissance of an active
    daemon's token-bearing files (the lock file itself is 0o600, but
    the containing directory's readable mode reveals the path namespace).
    Correct approach: `std::os::unix::fs::DirBuilderExt` —
    `DirBuilder::new().mode(0o700).recursive(true).create(&runtime_dir)`.
    Cross-platform note: on Windows the `mode()` Unix API is not
    available; the Phase 1 0o700 contract is asserted on Linux/macOS
    primary targets per NFR-008 (Windows is a secondary build target
    per PRD §8.7; Phase 1 CI does not formally validate Windows mode
    bits). This is the F-R75-1 recurrence guard.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 5.a | `MONOCLE_RUNTIME_DIR=<temp>` set | path (a) env override; INFO log; lock file at `<temp>/monocle.lock` |
| 5.b | env unset; `ProjectDirs::runtime_dir()` → `Some(<temp>)` | path (b); INFO log; lock at runtime_dir path |
| 5.c | env unset; runtime_dir → `None`; data_local_dir → `Some(<temp>)` (EC-057 macOS) | path (c); INFO log; lock at data_local_dir path |
| 5.d | env unset; `ProjectDirs::new()` → `None` (EC-059) | path (d); ERROR log; exit 1; NO lock file created |
| 5.e | Runtime dir absent prior to start (path (b) or (c) creates it) | Daemon creates dir; mode bits = `0o700` |
| 5.f | Existing live PID lock | Daemon exit 1; stderr `daemon already running at pid=` |
| 5.g | Stale PID lock (ESRCH on `kill(pid, 0)`) | Daemon proceeds; WARN log `stale lock file removed`; new lock written |
| 5.h | Clean shutdown via synthetic SIGTERM | `monocle.lock` AND `monocle.sock` removed (`Path::exists()` returns false) |
| 5.i | Source-grep: `tempfile::persist` present + no `std::fs::write` on `monocle.lock` line | structural assertions pass |
| 5.j | Source-grep: resolution-chain ordering preserved | chain order intact (env → runtime_dir → data_local_dir → fail-fast) |

**Mutation-test rationale:** the `0o600` lock-file mode literal, the
`0o700` runtime-dir mode literal, the `kill(pid, 0)` syscall result
check, AND the resolution-chain ordering (env-first → runtime_dir →
data_local_dir → fail-fast) are prime mutation targets. `cargo-mutants`
will attempt to mutate the lock-file mode to `0o644` (passing
functional tests that don't check mode), to mutate the runtime-dir
mode to `0o755` (umask-default leak — F-R75-1 surface), to flip the
`kill` result interpretation, and to reorder the resolution-chain
conditionals; all must be caught.

## Harness Location

- `monocle-runtime/tests/lock_file_lifecycle.rs` (integration)
- Test name: `test_BC_DAEMON_005_lock_file_create_and_cleanup` (per PRD
  v1.25 §BC-DAEMON-005, Verification subsection — covers the lock-file
  mode/lifecycle assertions AND the EC-057/058/059 resolution-chain
  probes via the canonical test-vector matrix; to be migrated to
  `test_BC_2_01_005_lock_file_create_and_cleanup`).

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
fn verify_bc_2_01_005() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/lock_file_lifecycle.rs` (integration)
- Test name: `test_BC_DAEMON_005_lock_file_create_and_cleanup` (per PRD
  v1.25 §BC-DAEMON-005, Verification subsection — covers the lock-file
  mode/lifecycle assertions AND the EC-057/058/059 resolution-chain
  probes via the canonical test-vector matrix; to be migrated to
  `test_BC_2_01_005_lock_file_create_and_cleanup`).

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
- Predecessor: monolithic VP-DAEMON-005 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.005.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26.3 §BC-2.01.005 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.17.
- Cross-property: VP-004 (drain completion → lock removal; exit-code
  taxonomy with `1` for `RuntimeDirUnresolvable`), VP-008 (defense-in-depth
  with auth-token), VP-010 (`contract_version` JSON-content joint
  assertion).

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
  - **Before:** body cited `SS-deps-pin-manifest.md v1.1.15` at 4 locations (pre-edit grep).
  - **After:** body cites `SS-deps-pin-manifest.md v1.1.17` at the same 4 locations (post-edit grep).
- **SE-17c-d body-scope grep:** pre-§Trace-block body cites of `1.1.15` → 0 remaining; cites of `1.1.17` → 4 (Pre-conditions §directories pin, §tempfile pin, §nix pin, References §Dependency pins).

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

## §Trace v1.0.3 — F-R105-13 LOW: VP §References PRD Citation Refresh v1.26 → v1.26.3 + Sister-VP Reference Reconciliation `VP-ENGINE-002-ERR` → `VP-021`

**Bump:** v1.0.2 → v1.0.3.
**Predecessor pin:** v1.0.2 (commit 927fcce — T-128g+T-128j FV — F-R105-7/10/11 (pin refresh + title sync + sister-VP reconciliation)).
**Scope of v1.0.3 (NORMATIVE — §References PRD citation refresh + sister-VP body reconciliation; NO content cascade; NO BC-path changes — BC §References already cite canonical sharded `behavioral-contracts/ss-NN/BC-2.SS.NNN.md` paths):**

### Change set 1 — §References PRD Citation Refresh `v1.26` → `v1.26.3` (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** §References cited `prd.md v1.26 §BC-2.01.005 (Dispatch 4 commit 1030c65).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.3 §BC-2.01.005 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (post-edit grep).
- **SE-17c-d body-scope grep:** post-edit `grep -n "prd.md v1.26 " vp-005-lock-file-lifecycle.md` → 0 matches; `grep -n "prd.md v1.26.3" vp-005-lock-file-lifecycle.md` → 1 match (§References line).
- **BC §References scope:** §References §Source contract entry already cites canonical sharded `behavioral-contracts/ss-01/BC-2.01.005.md` (per BC-INDEX.md v1.2). No BC-path changes required in this dispatch.
- **Historical PRD v1.25 citations in body prose (Source Contract `Traces to (historical)`, Harness Location `to be migrated to`, Proof Harness Skeleton `to be migrated to`, where present):** UNCHANGED — these are explicitly historical predecessor citations pinned to the pre-Dispatch-4 PRD monolith and must not be refreshed.

### Change set 2 — Sister-VP Reference Reconciliation `VP-ENGINE-002-ERR` → `VP-021` (F-R105-13 SURFACE) (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** Pre-conditions §`temp-env` pin note cited sister VP as `VP-ENGINE-002-ERR` (pre-renumbering form; line 106).
  - **After:** Pre-conditions §`temp-env` pin note cites sister VP as `VP-021 (renumbered from VP-ENGINE-002-ERR per VP-INDEX.md §Renumbering Appendix)`, matching the F-R105-11 reconciliation pattern applied to VP-007 in commit 927fcce.
- **SE-17c-d body-scope grep:** pre-§Trace-block body cites of `VP-ENGINE-002-ERR` → 0 remaining (the only remaining token is inside the §Trace block itself as historical before-evidence).
- **Discovered in-scope:** F-R105-13 sweep grep `grep -nE "VP-(DAEMON|AUTH|RING|LOCK|ABI|TYPES|FACTORY|PROTO|ENGINE)-" vp-*.md` surfaced one residual stale sister-VP ID in VP-005 body prose that was missed by the F-R105-11 sweep (which targeted VP-007 only). Fixed in-scope per CLAUDE.md Production-Grade Rule 4 (AI-built defects are AI's responsibility to fix; same defect class as F-R105-11, same fix pattern).

### Rationale

PO commit b2b378b (T-128k Round-3 PO dispatch) bumped PRD `v1.26.2 → v1.26.3` for F-R105-12 VP alias + GAP-R44-4 closure. Parallel FV dispatch refreshes VP §References to cite the post-bump PRD version, preserving the stale-citation-zero invariant established in F-R105-7 (manifest pin refresh) and F-R105-11 (sister-VP reference reconciliation). Per CLAUDE.md Production-Grade Rule 1: no MVP-driven deferral; mechanical citation refresh executed in-scope of T-128k FV portion rather than left to post-Round-3 cleanup.

**Change set 2 rationale:** Sweep grep for stale renumbered VP IDs surfaced one missed occurrence (VP-005 line 106). Same defect class and same fix pattern as the F-R105-11 reconciliation already applied to VP-007 (commit 927fcce). Per CLAUDE.md Routing Rule 3 ("Surface vs defer — the critical distinction"): fixed in-scope rather than deferring to a separate dispatch; same FV agent owns sister-VP body-prose reconciliation per F-R105-11 precedent.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.3 (commit b2b378b — F-R105-12 VP alias + GAP-R44-4 closure).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7 — confirms canonical sharded path `ss-01/BC-2.01.005.md` for BC-2.01.005).
- **R105 closure chain:** F-R105-13 LOW — 22-VP §References PRD citation refresh sweep.
- **R105 closure chain (additional):** F-R105-13 SURFACE — VP-005 sister-VP reference reconciliation `VP-ENGINE-002-ERR` → `VP-021` (F-R105-11 pattern application to one VP-005 body line missed by the original VP-007-targeted sweep).
- **Concurrent dispatches (T-128k Round 3):**
  - PO: PRD v1.26.2 → v1.26.3 (F-R105-12 + GAP-R44-4) — COMPLETE (commit b2b378b).
  - architect: auth-header interop adjudication — separate scope.
  - BA: L2-INDEX anchor fixes — separate scope.
  - FV: this §Trace (F-R105-13 — 22-VP §References PRD citation refresh + VP-005 sister-VP reconciliation surface fix).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T20:30:00Z` >= chain high-water `2026-05-17T19:30:00Z` (this VP's prior v1.0.2 §Trace and PRD v1.26.3 frontmatter). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References PRD citation `v1.26` → `v1.26.3`; sister-VP reference reconciliation `VP-ENGINE-002-ERR` → `VP-021`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### SE-17g META audit (NORMATIVE)

Sweep-wide post-edit re-grep across all 22 VP files: `grep -rE "prd\.md v1\.(26[^.]|26\.[012])(\s|\$)" .factory/specs/verification-properties/vp-*.md` → 0 matches. Sweep-wide re-grep for non-sharded BC paths: `grep -rE "behavioral-contracts/BC-[^I]" .factory/specs/verification-properties/vp-*.md` → 0 matches. F-R105-13 closure verified.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Mechanical citation refresh executed in-scope rather than deferred. PRD v1.26.3 cite is valid as of PO commit b2b378b. No tech-debt entries created. Body-prose historical PRD v1.25 citations preserved unchanged per Production-Grade discipline (historical predecessor citations are not stale; refreshing them would erase audit trail). Sister-VP body reconciliation co-located with PRD refresh per Routing Rule 3 (surface and fix in-scope, not defer).
