---
document_type: verification-properties
level: L3
section: "verification-properties"
version: "1.0"
status: complete
producer: formal-verifier
phase: pre-phase-1-architecture
timestamp: 2026-05-14T20:30:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-forward-compatibility.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0003-license-selection.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md
input-hash: "[live-state]"
traces_to: "16 BCs pre-staged across SS-daemon-lifecycle v1.0.7 (BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001), SS-core-types-and-abi v1.2.8 (BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-FACTORY-001, BC-FACTORY-002, BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002), and SS-engine-module v1.1.15 (BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003); dtu-assessment §DTU Architecture (hook protocol clone surface); Phase 1 PRD dispatch authorization per STATE.md §Phase 1 dispatch; production-grade default per CLAUDE.md §CANONICAL PRINCIPLE"
project: monocle
---

# Verification Properties: Phase 1 Behavioral Contract Catalog

## §Purpose

This artifact authors formally-testable Verification Properties (VPs) against
the 16 Behavioral Contracts (BCs) pre-staged in the Phase 1 architecture
artifacts. Each VP states a mechanical, executable property that asserts the
BC holds under a precisely scoped pre-condition. Each VP is bound to a
verification mechanism — Kani proof, fuzz harness, unit test, or mutation test
— and includes counter-example sketches that an adversary or fuzzer should
generate to refute the property.

The VP catalog is the input to Phase 6 (Formal Hardening). Every VP whose
mechanism is `unit-test` or `fuzz` is also a TDD target during Phase 3. Kani
proofs are deferred to Phase 6 but their harnesses are stubbed in this artifact
so the Phase 1 PRD can pre-stage them.

Per CLAUDE.md §CANONICAL PRINCIPLE — Production-Grade Default: every BC is
covered by at least one VP. No BC is deferred to "Phase 2 verification" or "we
can add tests later." Where a BC has both a wire-format facet and a Rust-surface
facet (BC-PROTO-001 family), each facet receives its own VP.

---

## §Scope

In scope:

- All 16 Phase 1 BCs pre-staged across `SS-daemon-lifecycle.md` v1.0.7,
  `SS-core-types-and-abi.md` v1.2.8, and `SS-engine-module.md` v1.1.15.
- Mechanical property statements (deterministic, executable assertions).
- Verification mechanism selection per VP — Kani / fuzz / unit / mutation.
- Pre-condition / post-condition pairs per VP.
- Counter-example sketches (adversarial inputs that should refute the property).
- Coverage matrix (BC → VP, one-to-one or one-to-many).
- Open verification gaps (DTU-blocked, Phase 4-deferred, etc.).

Out of scope:

- Phase 2+ BCs (deferred until Phase 2 PRD scoping).
- Performance-budget VPs (handled separately under `vsdd-factory:perf-check`).
- WCAG/accessibility VPs (TUI plane has none in Phase 1; deferred to Phase 2
  when TUI accessibility audit becomes scope).
- DTU fidelity VPs (the DTU clone itself is verified against real Claude Code;
  the DTU fidelity scoring procedure in `dtu-assessment.md` §DTU Fidelity
  Measurement Procedure is the canonical verification path, not a VP).

---

## §VP Catalog Overview

The catalog contains exactly 16 VPs, one per BC. Three VPs (VP-AUTH-001,
VP-AUTH-002, VP-FACTORY-002) admit auxiliary fuzz harnesses in addition to
their primary unit-test mechanism — see §Per-VP Detail and §Coverage Matrix
for the mechanism distribution.

| VP ID | BC Source | Property Domain | Primary Mechanism | Auxiliary Mechanism |
|-------|-----------|-----------------|-------------------|---------------------|
| VP-RING-001 | BC-RING-001 (SS-daemon-lifecycle v1.0.7) | JSONL ring record format-version is first key | unit-test | mutation-test |
| VP-AUTH-001 | BC-AUTH-001 (SS-daemon-lifecycle v1.0.7) | Wire format `monocle-v1:<64-hex>`; constant-time comparison | unit-test | fuzz |
| VP-AUTH-002 | BC-AUTH-002 (SS-daemon-lifecycle v1.0.7) | Non-prefixed tokens rejected with HTTP 401 + JSON error body | unit-test | fuzz |
| VP-LOCK-001 | BC-LOCK-001 (SS-daemon-lifecycle v1.0.7) | Lock-file `contract_version: 1` first key; readers gate on field | unit-test | mutation-test |
| VP-ABI-001 | BC-ABI-001 (SS-core-types-and-abi v1.2.8) | `/status` response body contains `abi_version: 1` | unit-test | — |
| VP-ABI-002 | BC-ABI-002 (SS-core-types-and-abi v1.2.8) | `monocle_core::MONOCLE_ABI_VERSION` pub const equals `1` | unit-test | — |
| VP-TYPES-001 | BC-TYPES-001 (SS-core-types-and-abi v1.2.8) | Every pub enum in `monocle-core` carries `#[non_exhaustive]` modulo ADR-0004 exemptions | unit-test | mutation-test |
| VP-FACTORY-001 | BC-FACTORY-001 (SS-core-types-and-abi v1.2.8) | `FactoryAdapter` trait signature stable; no `private::Sealed` supertrait | unit-test | — |
| VP-FACTORY-002 | BC-FACTORY-002 (SS-core-types-and-abi v1.2.8) | `VsddFactoryAdapter::new` + self-referential detection; `None` for absent optionals | unit-test | fuzz |
| VP-PROTO-001a | BC-PROTO-001a (SS-core-types-and-abi v1.2.8) | Proto field number 1 in `HookEnvelope` is `schema_version` | unit-test | — |
| VP-PROTO-001b | BC-PROTO-001b (SS-core-types-and-abi v1.2.8) | Rust `HookEnvelope` struct exposes `pub schema_version: u32`; value `1` | unit-test | — |
| VP-PROTO-002 | BC-PROTO-002 (SS-core-types-and-abi v1.2.8) | Unknown `schema_version` is skipped with warning; no panic | unit-test | fuzz |
| VP-ENGINE-001 | BC-ENGINE-001 (SS-engine-module v1.1.15) | `EngineModule` trait signature stable; `last_event_micros: Option<i64>`; no silent fallback | unit-test | — |
| VP-ENGINE-002 | BC-ENGINE-002 (SS-engine-module v1.1.15) | `ClaudeCodeModule::detect` strict basename match; cmdline ignored | unit-test | — |
| VP-ENGINE-002-ERR | BC-ENGINE-002-ERR (SS-engine-module v1.1.15) | `metadata`/`enrich` return `HomeUnresolvable` with all four home-env vars unset | unit-test | — |
| VP-ENGINE-003 | BC-ENGINE-003 (SS-engine-module v1.1.15) | `hook_paths()` returns exactly 5 entries — one per `HookType` variant | unit-test | — |

### §Mechanism Distribution

| Mechanism | Count (primary) | Count (auxiliary) | Total VPs touched |
|-----------|-----------------|-------------------|-------------------|
| unit-test | 16 | 0 | 16 |
| fuzz | 0 | 4 | 4 |
| mutation-test | 0 | 3 | 3 |
| Kani proof | 0 | 0 | 0 (deferred — see §Open Verification Gaps §G-1) |

Kani proof harnesses are NOT used in Phase 1 because the Phase 1 BCs do not
require model-checking — they are deterministic protocol contracts whose
verification is fully discharged by unit tests and round-trip serde fuzzing.
Phase 2 (trigger-trace state machine) and Phase 3 (wasmtime plugin host) are
the first phases where Kani's strengths (arithmetic overflow, state-machine
invariants on arbitrary inputs) become load-bearing. See §Open Verification
Gaps §G-1 for the Phase 2 trigger Kani pre-stage.

---

## §Per-VP Detail

Each VP below states: the mechanical property; the verification mechanism;
pre-conditions (test setup); post-conditions (assertions that must hold); and
counter-example sketches (adversarial inputs that, if accepted, would refute
the property).

### §VP-RING-001 — JSONL Ring Record Format-Version First Key

**Traces to:** BC-RING-001 (SS-daemon-lifecycle.md §Drain).

**Mechanical property:** For every `HookEventRecord` constructed via
`HookEventRecord::new(...)`, `serde_json::to_string(&record)` produces a JSON
string whose first non-whitespace character after the opening `{` is the key
`"format_version"` with value `1`. Formally:

```
forall record: HookEventRecord constructed via HookEventRecord::new(...),
  serde_json::to_string(&record).unwrap().starts_with("{\"format_version\":1,")
```

**Mechanism:** unit-test (primary); mutation-test (auxiliary).

**Pre-conditions:**

- `RING_FORMAT_VERSION` const equals `1` (loaded from `monocle-runtime::ring`).
- `HookEventRecord` carries `#[non_exhaustive]` (otherwise the constructor
  contract is moot — see VP-TYPES-001 for the orthogonal exhaustive-enum
  property).
- `serde_json 1` is the project pin (per SS-deps-pin-manifest).

**Post-conditions:**

1. `record.format_version == 1` after construction.
2. Serialized prefix is exactly `{"format_version":1,` (literal string match).
3. Round-trip preservation: `serde_json::from_str::<HookEventRecord>(&s).unwrap().format_version == 1`.

**Counter-example sketches (adversary should attempt):**

1. Reorder struct field declarations in `HookEventRecord` such that
   `session_id` precedes `format_version` — must cause the unit test to fail
   because serde respects declaration order.
2. Change `RING_FORMAT_VERSION` to `0` or `2` — must cause the literal-prefix
   assertion to fail.
3. Use `serde_json::Value`-wrapped serialization that re-orders keys
   alphabetically (e.g., `serde_json::to_value(&record).unwrap().to_string()`)
   — this would order `format_version` after `hook_type` etc.; the unit test
   MUST use direct `to_string(&record)`, not `to_value` round-trip.
4. Replace `#[derive(Serialize)]` with a hand-written impl that emits fields in
   alphabetical order — must cause the unit test to fail.

**Harness location:** `monocle-runtime/tests/jsonl_ring.rs`.

**Mutation-test rationale:** the `format_version: u32` field value `1` is a
prime mutation target (off-by-one, sign-flip). Mutation testing with
`cargo-mutants` ensures the assertion is value-discriminating, not just
key-discriminating.

---

### §VP-AUTH-001 — Auth Token Wire Format and Constant-Time Comparison

**Traces to:** BC-AUTH-001 (SS-daemon-lifecycle.md §Start Sequence).

**Mechanical property:**

1. The lock-file `authToken` field, when read back as a string, matches the
   regex `^[0-9a-f]{64}$` (bare 64-char lowercase hex).
2. The wire-format token presented in `X-Monocle-Authorization` is exactly
   `"monocle-v1:" ++ authToken` (74 characters total: 11-char prefix + 64-char
   hex).
3. `validate_auth_token(presented, expected_secret)` returns `true` iff
   `presented` has the `monocle-v1:` prefix AND the post-prefix hex equals
   `expected_secret` byte-for-byte.
4. The comparison is performed via `constant_time_eq::constant_time_eq` — NOT
   via `==` on `&str` or `String`.

**Mechanism:** unit-test (primary); fuzz (auxiliary).

**Pre-conditions:**

- Daemon has completed start sequence and written `monocle.lock`.
- `constant_time_eq ^0.3` is the project pin (per SS-deps-pin-manifest).
- `rand::rngs::OsRng` is the entropy source (not `thread_rng`).

**Post-conditions:**

1. `lock.authToken` matches `^[0-9a-f]{64}$` (exact length 64, lowercase hex only).
2. Presenting `monocle-v1:<lock.authToken>` to `/status` returns HTTP 200.
3. Presenting `monocle-v1:<lock.authToken with one byte flipped>` returns HTTP 401.
4. Presenting `<lock.authToken>` WITHOUT the `monocle-v1:` prefix returns HTTP 401.
5. The auth middleware's secret comparison uses `constant_time_eq`; this is
   verified by source-grep against `monocle-runtime/src/auth.rs` ensuring no
   `==` on the hex secret string appears outside `constant_time_eq`.

**Counter-example sketches:**

1. Switch `constant_time_eq` to `String::eq` — would still pass functional
   tests but would lose the timing-oracle property; mitigated by the
   source-grep assertion in the harness.
2. Lock file written with `tempfile::persist` interrupted mid-write — partial
   token leaves a < 64-char hex; the regex match must reject.
3. Token generation via `rand::thread_rng()` instead of `OsRng` — passes the
   format regex but fails the entropy source check (verified by
   source-grep against `monocle-runtime/src/lock.rs`).
4. Adversary submits `monocle-v1:` + 64 chars of `0` (all-zero secret) —
   must be rejected because the real secret has 256 bits of entropy.

**Fuzz harness:** `cargo fuzz add fuzz_auth_token_validation`. The fuzz target
constructs arbitrary byte sequences as the `X-Monocle-Authorization` value
and runs `validate_auth_token(input, expected)` against a fixed 64-char hex
secret. The fuzzer should never produce an input that returns `true` other
than the exact expected secret with the `monocle-v1:` prefix. The target asserts
NO panic and NO `true` return for any input differing from the expected secret.

**Harness location:** `monocle-runtime/tests/auth_token_lifecycle.rs` (unit);
`fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz).

---

### §VP-AUTH-002 — Non-Prefixed Tokens Rejected with HTTP 401 + Error Body

**Traces to:** BC-AUTH-002 (SS-daemon-lifecycle.md §Start Sequence).

**Mechanical property:** Any `X-Monocle-Authorization` header value not
beginning with the literal prefix `monocle-v1:` causes the daemon to respond
with HTTP 401 and the JSON body
`{"error":"invalid_auth_token_format"}`. Additionally, any
`Authorization: Bearer <anything>` header on a Phase 1 HTTP endpoint is
rejected with HTTP 401 (the Bearer header is not a recognized auth mechanism
on Phase 1 routes).

**Mechanism:** unit-test (primary); fuzz (auxiliary).

**Pre-conditions:**

- Daemon is running with a valid `monocle-v1:` secret in the lock file.
- Authenticated test client has access to the secret for the positive control.

**Post-conditions (per probe):**

| Probe | Header | Expected status | Expected body |
|-------|--------|-----------------|---------------|
| 1 | `Authorization: Bearer fake` | 401 | `{"error":"invalid_auth_token_format"}` |
| 2 | `X-Monocle-Authorization: baretoken` | 401 | `{"error":"invalid_auth_token_format"}` |
| 3 | `X-Monocle-Authorization: monocle-v2:abc` | 401 | `{"error":"invalid_auth_token_format"}` |
| 4 | `X-Monocle-Authorization: monocle-v1:` followed by 64 chars not matching the secret | 401 | (token-mismatch path; not the format-rejection body) |
| 5 | `X-Monocle-Authorization: monocle-v1:` followed by the secret | 200 | (positive control) |

**Counter-example sketches:**

1. Auth middleware accepts `Authorization: Bearer` as a fallback path —
   probe 1 would return 200; the unit test must assert 401.
2. Auth middleware does `presented.contains("monocle-v1:")` instead of
   `strip_prefix("monocle-v1:")` — probe `X-Monocle-Authorization:
   junk-monocle-v1:abc` would be accepted; the unit test asserts strict
   `strip_prefix` behavior.
3. Empty `X-Monocle-Authorization` header value — must return 401 (covered by
   probe 2 with the empty-string variant).

**Fuzz harness:** the `fuzz_auth_token_validation` target from VP-AUTH-001
also covers this property because the format-rejection path runs BEFORE the
constant-time comparison — non-prefixed inputs MUST short-circuit-fail with
the `invalid_auth_token_format` error code.

**Harness location:** `monocle-runtime/tests/auth_header_rejection.rs`.

---

### §VP-LOCK-001 — Lock File `contract_version: 1` First Key

**Traces to:** BC-LOCK-001 (SS-daemon-lifecycle.md §Start Sequence).

**Mechanical property:**

1. The JSON content written to `<runtime_dir>/monocle.lock` is structurally a
   JSON object whose first key (per `serde_json` declaration-order
   serialization) is `contract_version` with integer value `1`.
2. Any lock-file reader (e.g., the TUI client's lock-file ingestion path)
   MUST inspect `contract_version` before consuming other fields; reading code
   asserts `contract_version == 1` and on mismatch logs a warning and skips
   the file gracefully (no panic).

**Mechanism:** unit-test (primary); mutation-test (auxiliary).

**Pre-conditions:**

- Daemon start sequence completes; lock file is written via `tempfile::persist`.
- Lock-file reader code is `pub fn read_lock_file(path: &Path) -> Result<LockFile, LockFileError>`.

**Post-conditions:**

1. `std::fs::read_to_string(&lock_path).unwrap().starts_with("{\"contract_version\":1,")`.
2. `serde_json::from_str::<LockFile>(&content).unwrap().contract_version == 1`.
3. With a synthetic lock file where `contract_version = 2`, the reader logs
   a warning and returns `Err(LockFileError::UnsupportedContractVersion(2))`
   — NOT a panic and NOT a silent acceptance of unknown fields.
4. With a synthetic lock file where `contract_version` is absent entirely, the
   reader returns `Err(LockFileError::MissingContractVersion)`.

**Counter-example sketches:**

1. Lock file written with `serde_json::to_value(&lock).to_string()` (which
   alphabetizes) — would place `app` before `contract_version`; the prefix
   assertion must fail.
2. Reader implements `serde_json::from_str` without an explicit
   `contract_version` check before field access — a future v2 lock file would
   be silently misparsed; the unit test must construct a synthetic v2 file
   and assert the version-gate error.
3. Lock-file writer omits `contract_version` (regression) — readers MUST
   reject; covered by post-condition 4.

**Mutation-test rationale:** the `contract_version` integer value `1` is a
prime mutation target. `cargo-mutants` will attempt to mutate the writer to
`contract_version = 0` and the reader's gate condition; both must be caught
by the unit test.

**Harness location:** `monocle-runtime/tests/lock_file_contract.rs`.

---

### §VP-ABI-001 — `/status` Response Body Contains `abi_version: 1`

**Traces to:** BC-ABI-001 (SS-core-types-and-abi.md §ABI Version Constant).

**Mechanical property:** A `GET /status` request with a valid
`X-Monocle-Authorization` header returns HTTP 200 with a JSON body whose
top-level `abi_version` key has the integer value `1` (equal to
`monocle_core::MONOCLE_ABI_VERSION` as compiled into the daemon binary).

**Mechanism:** unit-test.

**Pre-conditions:**

- Daemon running with a valid lock file.
- Authenticated client holds the lock-file secret.

**Post-conditions:**

1. HTTP 200 status code on `GET /status`.
2. Response body parsed as JSON has key `abi_version` with integer value `1`.
3. The value `1` equals `monocle_core::MONOCLE_ABI_VERSION` at compile time
   (compile-time `const _: () = assert!(MONOCLE_ABI_VERSION == 1)` in the
   binary crate ensures drift between binary and constant is impossible).

**Counter-example sketches:**

1. `/status` handler hardcodes `"abi_version": 2` — must fail the literal
   integer comparison.
2. `MONOCLE_ABI_VERSION` raised to `2` without updating the status handler —
   the compile-time assert catches drift; without the assert, the unit test
   would still catch the runtime mismatch.

**Harness location:** `monocle-runtime/tests/status_endpoint.rs`.

---

### §VP-ABI-002 — `monocle_core::MONOCLE_ABI_VERSION` Pub Const Equals `1`

**Traces to:** BC-ABI-002 (SS-core-types-and-abi.md §ABI Version Constant).

**Mechanical property:**

1. `monocle_core::MONOCLE_ABI_VERSION` is publicly accessible at the crate
   root (no `pub use` from a private module that fails to re-export).
2. Its type is `u32`.
3. Its value is `1`.
4. The constant is usable in const contexts — i.e.,
   `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1);` compiles.

**Mechanism:** unit-test (specifically a compile-time test in
`monocle-core/tests/abi_stability.rs`).

**Pre-conditions:**

- `monocle-core` is the project pinned crate.
- `cargo check --tests` is the verification driver.

**Post-conditions:**

1. The `tests/abi_stability.rs` file contains `const _: () =
   assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "ABI version drift");` and
   compiles cleanly.
2. A runtime assertion `assert_eq!(monocle_core::MONOCLE_ABI_VERSION, 1u32);`
   passes.
3. The type assertion `let _: u32 = monocle_core::MONOCLE_ABI_VERSION;`
   compiles (catches accidental promotion to `u64` or demotion to `u8`).

**Counter-example sketches:**

1. `MONOCLE_ABI_VERSION` re-typed as `u64` — fails the type-pinning let-binding.
2. `MONOCLE_ABI_VERSION` defined as `pub static` instead of `pub const` — fails
   the const-context assertion (statics cannot be used in `const _:` blocks).
3. `MONOCLE_ABI_VERSION` moved into a private module without `pub use` —
   fails to compile because `monocle_core::MONOCLE_ABI_VERSION` is unresolved.

**Harness location:** `monocle-core/tests/abi_stability.rs`.

---

### §VP-TYPES-001 — Every Pub Enum in `monocle-core` Carries `#[non_exhaustive]` Modulo ADR-0004 Exemptions

**Traces to:** BC-TYPES-001 (SS-core-types-and-abi.md §Enum Extensibility).

**Mechanical property:** For every `pub enum E` defined in any source file of
the `monocle-core` crate, exactly one of the following holds:

1. `E` carries `#[non_exhaustive]`, OR
2. `E` is listed in the ADR-0004 exemption set
   `{ "Phase1Permission", "ClaudeCodeTool" }`.

No other exemption is allowed without a new ADR superseding ADR-0004.

**Mechanism:** unit-test (primary, via a `cargo clippy` lint configuration);
mutation-test (auxiliary).

**Pre-conditions:**

- `monocle-core` source tree is the audit scope.
- The exempt-list constant in the test harness is
  `EXEMPT: &[&str] = &["Phase1Permission", "ClaudeCodeTool"]`.

**Post-conditions:**

1. A test harness in `monocle-core/tests/enum_audit.rs` parses every
   `monocle-core/src/**/*.rs` file via `syn 2`, walks all `Item::Enum` nodes,
   and asserts that for each enum either `#[non_exhaustive]` is present in
   the attribute list OR the enum's identifier is in `EXEMPT`.
2. The test fails with a descriptive error listing every offending enum if
   the property is violated.
3. The `cargo clippy --workspace -- -D warnings` invocation passes with the
   project-local lint `non_exhaustive_omitted_patterns` deny-listed for
   `#[allow]` (per SS-conventions-anti-patterns.md).

**Counter-example sketches:**

1. A new contributor adds `pub enum NewError { ... }` to `monocle-core/src/`
   without `#[non_exhaustive]` and not in the exempt list — must fail the
   audit.
2. A contributor sneaks `#[allow(non_exhaustive_omitted_patterns)]` into a
   match site — must fail the clippy step (semgrep rule co-enforces, per
   SS-conventions-anti-patterns.md §Semgrep Rules).
3. A contributor adds `pub enum Phase2Permission` to `monocle-core/` (NOT in
   `monocle-plugin-sdk`) without `#[non_exhaustive]` and not in the exempt
   list — must fail the audit. (Even though ADR-0004 contemplates a parallel
   `Phase 3` enum in the plugin SDK, that enum is in a different crate and
   the audit is `monocle-core`-scoped.)
4. Exempt list expanded silently to add a third enum without an ADR
   superseding ADR-0004 — covered by an orthogonal consistency check that
   greps for the EXEMPT constant length and asserts it equals the count of
   exhaustive enums documented in ADR-0004 (currently 2).

**Mutation-test rationale:** mutating the `EXEMPT` constant length (e.g.,
adding a stray entry) or the `#[non_exhaustive]` attribute presence check
(e.g., flipping `has_attr` to `!has_attr`) must be caught by the audit
harness — this is a high-leverage mutation surface.

**Harness location:** `monocle-core/tests/enum_audit.rs`.

---

### §VP-FACTORY-001 — `FactoryAdapter` Trait Signature Stable; No Sealed Bound

**Traces to:** BC-FACTORY-001 (SS-core-types-and-abi.md §FactoryAdapter Trait).

**Mechanical property:**

1. The trait `monocle_core::factory::FactoryAdapter` exists with the exact
   method set: `detect`, `matches`, `state_file_path`, `read_state`,
   `subscribe`, `display_name`, `abi_version`.
2. The trait's super-bounds are exactly `Send + Sync + 'static` — no
   `private::Sealed` (or any other sealing) supertrait appears.
3. The supporting types
   `{FactoryDetection, FactoryState, BlockingIssue, BlockingSeverity,
   ConvergenceMetrics, FactoryReadError, FactorySubscribeError,
   StateChangeStream}` are all `pub` and accessible from
   `monocle_core::factory::*`.
4. `FactoryState` has the 7 canonical fields:
   `{ phase: String, status: String, awaiting: Option<String>,
   blocking_issues: Vec<BlockingIssue>,
   convergence: Option<ConvergenceMetrics>, cycle: Option<String>,
   custom_fields: HashMap<String, serde_yaml_ng::Value> }`.

**Mechanism:** unit-test (specifically a `cargo check` + `syn 2` parse over
the public trait surface).

**Pre-conditions:**

- `monocle-core` builds cleanly.
- `rustdoc` JSON output is available via `cargo +nightly rustdoc -- -Z unstable-options --output-format json` OR equivalent stable `cargo doc` parsing.

**Post-conditions:**

1. `cargo check --workspace` passes.
2. A `monocle-core/tests/factory_adapter_surface.rs` test uses `syn 2` to
   parse `monocle-core/src/factory.rs`, locates the `trait FactoryAdapter`
   item, and asserts:
   - method count equals 7;
   - method names match the canonical set (HashSet equality);
   - super-trait bounds equal `Send + Sync + 'static` (token-stream match);
   - no `Sealed` identifier appears anywhere in the trait declaration.
3. A `FactoryState` field-name check asserts the HashSet of field identifiers
   equals the 7-field canonical set above.

**Counter-example sketches:**

1. A future refactor adds a `Sealed` supertrait — must fail the substring
   check.
2. A method is renamed (e.g., `display_name` → `name`) — must fail the
   canonical-method-set HashSet equality.
3. A new method `priority` is added without a default body — must fail the
   method count check; a method added WITH a default body is permitted
   per SS-core-types-and-abi.md §Forward Compatibility Guarantees, so the
   audit must distinguish defaulted vs non-defaulted methods (the
   `has_block` check on the `TraitItemFn` syn node distinguishes them).
4. A `FactoryState` field is renamed (e.g., `phase` → `pipeline_phase`) —
   must fail the field-name HashSet equality.

**Harness location:** `monocle-core/tests/factory_adapter_surface.rs`.

---

### §VP-FACTORY-002 — `VsddFactoryAdapter::new` + Self-Referential Detection; `None` for Absent Optionals

**Traces to:** BC-FACTORY-002 (SS-core-types-and-abi.md §FactoryAdapter Trait).

**Mechanical property:**

1. A public constructor `VsddFactoryAdapter::new(workspace_root: PathBuf) ->
   Self` exists.
2. The constructor performs NO validation (it does not panic, does not error,
   does not stat the filesystem); validation happens in `detect()` and
   `read_state()`.
3. The static method `VsddFactoryAdapter::detect(<monocle repo root>)`
   returns `Some(FactoryDetection)` where `display_name == "VSDD Factory"`
   (self-referential test).
4. For a `FactoryState` produced from a STATE.md file lacking
   `current_cycle:`, `state.cycle == None` (NOT `Some("unknown")` or any
   placeholder string).
5. For a `FactoryState` produced from a STATE.md file lacking a §Session
   Resume Checkpoint section, `state.convergence == None`.

**Mechanism:** unit-test (primary); fuzz (auxiliary).

**Pre-conditions:**

- `monocle-core` builds cleanly.
- Test fixture STATE.md files are checked in under
  `monocle-core/tests/fixtures/`:
  - `state_minimal.md` — has `current_cycle:` and §Session Resume Checkpoint.
  - `state_no_cycle.md` — lacks `current_cycle:` frontmatter key.
  - `state_no_checkpoint.md` — has `current_cycle:` but lacks §Session
    Resume Checkpoint section.

**Post-conditions:**

1. `VsddFactoryAdapter::new(PathBuf::from("/nonexistent/path"))` returns a
   value without error or panic.
2. `VsddFactoryAdapter::detect(&monocle_repo_root)` returns
   `Some(d)` where `d.display_name == "VSDD Factory"` and `d.state_file ==
   monocle_repo_root.join(".factory/STATE.md")`.
3. For fixture `state_no_cycle.md`, `state.cycle.is_none()`.
4. For fixture `state_no_checkpoint.md`, `state.convergence.is_none()`.
5. For fixture `state_minimal.md`, `state.cycle == Some("cycle-001".into())`
   (or whatever the fixture declares) — proves the `None` cases are
   discriminating, not a vacuous default.
6. `parse_frontmatter_field` returns `None` for: empty values, flow-style
   lists (`[...]`), block scalars (`|` or `>` lead), and continuation lines.

**Counter-example sketches:**

1. The constructor stats `workspace_root` and panics on absent path — fails
   post-condition 1.
2. `read_state` substitutes `"unknown"` for absent `current_cycle:` — fails
   post-condition 3.
3. `parse_frontmatter_field` accepts `awaiting: [a, b]` and returns
   `Some("[a, b]")` — fails post-condition 6 (the v1.2.3 round-20 fix).
4. `parse_frontmatter_field` accepts `phase: |` block-scalar marker and
   returns `Some("|")` — fails post-condition 6.
5. Self-referential detect fails because the `document_type:
   pipeline-state` substring check is too strict (e.g., requires exact
   line equality including trailing whitespace) — fails post-condition 2.

**Fuzz harness:** `cargo fuzz add fuzz_state_md_parser`. The fuzz target
feeds arbitrary UTF-8 byte sequences into `parse_frontmatter_field(content,
"phase")` and `parse_frontmatter_extra_fields(content, &known_keys)` and
asserts: no panic; no allocation > 1 MiB; flow-style and block-scalar inputs
produce `None` (frontmatter_field) or are skipped (extra_fields). The fuzzer
is seeded with `state_minimal.md`, `state_no_cycle.md`, and adversarial
malformed corpora (truncated frontmatter, mismatched quotes, Unicode
direction overrides, deep nesting markers).

**Harness location:** `monocle-core/tests/factory_self_referential.rs` (unit);
`fuzz/fuzz_targets/fuzz_state_md_parser.rs` (fuzz).

---

### §VP-PROTO-001a — Proto Field Number 1 in `HookEnvelope` is `schema_version`

**Traces to:** BC-PROTO-001a (SS-core-types-and-abi.md §Prost Wire Schemas).

**Mechanical property:** In `monocle-proto/proto/monocle/v1/hook_envelope.proto`,
the `HookEnvelope` message's field assigned to proto-tag-number `1` has the
field name `schema_version` and type `uint32`. The wire-level invariant is
verified by encoding a `HookEnvelope` and decoding the first field tag.

**Mechanism:** unit-test.

**Pre-conditions:**

- `monocle-proto` build script (`build.rs`) compiles the `.proto` files via
  `prost-build`.
- `prost-reflect` or direct `prost::encoding` is available in `[dev-dependencies]`.

**Post-conditions:**

1. Encoding a `HookEnvelope { schema_version: 1, ... }` via `prost::Message::
   encode_to_vec(&envelope)` produces a byte stream whose first wire-tag
   decodes to field number 1 with wire type `Varint` (proto3 `uint32` =
   varint).
2. A `prost-build`-generated descriptor inspection (via
   `prost_reflect::DescriptorPool::decode(...)` over the FileDescriptorSet
   emitted by `build.rs`) confirms field number 1 is named `schema_version`.

**Counter-example sketches:**

1. The `.proto` file is edited so `schema_version = 5;` — must fail the
   wire-tag decode (the first tag would decode to field 5 instead of 1).
2. A new field `string trace_id = 1;` is inserted, displacing
   `schema_version` to a new number — must fail the field-name lookup.

**Harness location:** `monocle-proto/tests/wire_field_order.rs`.

---

### §VP-PROTO-001b — Rust `HookEnvelope` Struct Exposes `pub schema_version: u32` with Value `1`

**Traces to:** BC-PROTO-001b (SS-core-types-and-abi.md §Prost Wire Schemas).

**Mechanical property:** The prost-build-generated Rust type
`monocle_proto::v1::HookEnvelope` exposes a public field
`schema_version: u32`. For all Phase 1-origin messages (those constructed
inside Phase 1 monocle code), the value of `schema_version` is `1`.

**Mechanism:** unit-test.

**Pre-conditions:**

- `monocle-proto` builds cleanly.
- The `pub use monocle::v1` re-export is present so callers can access
  `monocle_proto::v1::HookEnvelope`.

**Post-conditions:**

1. A unit test constructs a `HookEnvelope` with `schema_version: 1` and any
   `oneof event` variant (e.g., `SessionStartEvent { cwd: "/", transcript_path:
   "" }`) and asserts `envelope.schema_version == 1`.
2. Round-trip serialize/deserialize preserves `schema_version`:
   `HookEnvelope::decode(envelope.encode_to_vec().as_slice()).unwrap()
   .schema_version == 1`.
3. The Rust struct field declaration order is NOT asserted (per BC-PROTO-001b
   normative carve-out — the proto-tag-number is the wire contract, not the
   Rust field declaration order).

**Counter-example sketches:**

1. The `.proto` file changes `uint32 schema_version = 1;` to
   `int32 schema_version = 1;` — would change the Rust type to `i32`; the
   `pub schema_version: u32` type check fails.
2. A constructor helper in `monocle-proto` defaults `schema_version` to `0`
   — fails post-condition 1.

**Harness location:** `monocle-proto/tests/schema_version.rs`.

---

### §VP-PROTO-002 — Unknown `schema_version` Skipped with Warning; No Panic

**Traces to:** BC-PROTO-002 (SS-core-types-and-abi.md §Prost Wire Schemas).

**Mechanical property:** A `HookEnvelope` message with `schema_version = 0`
(or any unrecognized value other than `1`) is processed by the Phase 4
deserialization layer (`monocle-ipc::dispatch`) by:

1. Logging a `tracing::warn!` event with structured field
   `schema_version = <unknown_value>` and a descriptive message.
2. Returning `Ok(())` (skip) — NOT a panic, NOT an error propagated to the
   caller, NOT a fallback parse with `schema_version = 1` assumed.

Phase 1 stubs this behavior via a trivial dispatch function that exists in
`monocle-proto` for Phase 1 testing; Phase 4 inherits and extends it.

**Mechanism:** unit-test (primary); fuzz (auxiliary).

**Pre-conditions:**

- `monocle-proto` exports `pub fn dispatch_envelope(env: &HookEnvelope) ->
  Result<(), DispatchError>` with the Phase 1 stub behavior above.
- `tracing-subscriber` is configured in tests to capture warnings.

**Post-conditions:**

1. `dispatch_envelope(&envelope_with_schema_version_0)` returns `Ok(())`.
2. A `tracing` warning event is emitted with `schema_version = 0` in its
   structured fields (captured via `tracing_subscriber::fmt::layer()` and
   asserted in the test).
3. `dispatch_envelope(&envelope_with_schema_version_99)` returns `Ok(())`
   with an analogous warning.
4. `dispatch_envelope(&envelope_with_schema_version_1)` returns `Ok(())`
   with NO warning emitted (positive control).
5. Calling `dispatch_envelope` 10,000 times with `schema_version = 0`
   does NOT panic, deadlock, or allocate unbounded memory (loose smoke test).

**Counter-example sketches:**

1. Dispatch panics on unknown version — fails post-condition 1.
2. Dispatch propagates `DispatchError::UnknownSchemaVersion` — fails
   post-condition 1 (Phase 4 must not crash on unknown versions per proto3
   forward-compat semantics).
3. Dispatch silently accepts unknown version (no warning emitted) — fails
   post-condition 2; this is the "silent acceptance" regression.

**Fuzz harness:** `cargo fuzz add fuzz_envelope_dispatch`. The fuzz target
constructs `HookEnvelope { schema_version: u32::arbitrary(u)?, event: ... }`
for arbitrary `schema_version` values from `0..u32::MAX` and asserts:
no panic; `dispatch_envelope` returns `Ok(())` for all inputs.

**Harness location:** `monocle-proto/tests/dispatch_unknown_version.rs`
(unit); `fuzz/fuzz_targets/fuzz_envelope_dispatch.rs` (fuzz).

---

### §VP-ENGINE-001 — `EngineModule` Trait Signature Stable; `last_event_micros: Option<i64>`; No Silent Fallback

**Traces to:** BC-ENGINE-001 (SS-engine-module.md §Behavioral Contracts).

**Mechanical property:**

1. The trait `monocle_core::engine::EngineModule` exists with the exact
   method set: `id`, `metadata`, `detect`, `enrich`, `on_hook`.
2. The trait has NO sealed bound (no `private::Sealed` supertrait).
3. `metadata()` returns `Result<EngineMetadata, EngineMetadataError>`;
   `enrich()` returns `Result<EnrichedSession, EngineMetadataError>` (both
   typed-error returns, not `Option<...>`-with-silent-fallback).
4. `EnrichedSession::last_event_micros` has type `Option<i64>` (NOT bare
   `i64`); `None` is distinguishable from any numeric value including the
   Unix epoch `0`.
5. Supporting types `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`,
   `SessionStatus`, `HookResponse`, `HookDecision`, `DeferUntil`,
   `EngineMetadataError` are all `pub` in `monocle_core::engine`.

**Mechanism:** unit-test (via `syn 2` parse of `monocle-core/src/engine.rs`).

**Pre-conditions:**

- `monocle-core` builds cleanly.

**Post-conditions:**

1. A `monocle-core/tests/engine_module_surface.rs` test parses the trait
   declaration and asserts:
   - method count equals 5;
   - method names match the canonical HashSet
     `{id, metadata, detect, enrich, on_hook}`;
   - super-bounds equal `Send + Sync + 'static` (no `Sealed`);
   - `metadata` return type token-stream matches
     `Result < EngineMetadata , EngineMetadataError >`;
   - `enrich` return type token-stream matches
     `Result < EnrichedSession , EngineMetadataError >`.
2. The same test asserts `EnrichedSession::last_event_micros` field type is
   `Option < i64 >` (not bare `i64`).
3. All eight supporting types resolve via `cargo check` with a probe file
   `let _: monocle_core::engine::EngineMetadata; ...`.

**Counter-example sketches:**

1. A refactor changes `metadata() -> Result<...>` to
   `metadata() -> EngineMetadata` (panicking on home-unresolvable) — fails
   the return-type token-stream match.
2. `last_event_micros` is reverted to bare `i64` with `0` as sentinel — fails
   the field-type assertion. This regression is what the v1.1.8 fix
   (F-R28-1) closed; the VP enforces it.
3. A `private::Sealed` supertrait is added — fails the no-sealed
   assertion. ADR-0004 governs the open trait property; sealing the trait
   would defeat Phase 3 plugin SDK adapter authoring.

**Harness location:** `monocle-core/tests/engine_module_surface.rs`.

---

### §VP-ENGINE-002 — `ClaudeCodeModule::detect` Strict Basename Match; Cmdline Ignored

**Traces to:** BC-ENGINE-002 (SS-engine-module.md §Behavioral Contracts).

**Mechanical property:** `ClaudeCodeModule::detect(&snapshot)` returns `true`
iff `snapshot.exe_path` is `Some(p)` AND `p.file_name() == Some("claude") ||
p.file_name() == Some("claude.js")`. The method NEVER consults
`snapshot.cmdline` for identification.

**Mechanism:** unit-test.

**Pre-conditions:**

- `ProcessSnapshot::new(pid, exe_path, cmdline, start_time_secs)`
  constructor is available (per F-R26-adv-1 fix, v1.1.7).
- `ClaudeCodeModule::new("http://127.0.0.1:7891".into())` constructs a module.

**Post-conditions (per probe):**

| Probe | exe_path | cmdline | Expected `detect()` |
|-------|----------|---------|---------------------|
| (a) | `Some("/usr/local/bin/claude")` | `vec![]` | `true` |
| (b) | `Some("/usr/local/bin/claude-squad")` | `vec![]` | `false` |
| (c) | `None` | `vec!["claude".to_string()]` | `false` (exe_path=None regardless of cmdline) |
| (d) | `Some("/opt/anthropic/claude.js")` | `vec![]` | `true` |
| (e) | `Some("/usr/local/bin/claudio")` | `vec!["claude", "--debug"]` | `false` |
| (f) | `Some("/home/x/bin/claude-code-router")` | `vec![]` | `false` |

**Counter-example sketches:**

1. `detect` uses `cmdline[0].contains("claude")` — probe (c) returns `true`;
   the unit test must assert `false`.
2. `detect` uses `exe_path.starts_with("/usr/local/bin/claude")` (prefix
   match, not basename) — probe (b) returns `true`; the unit test asserts
   `false`.
3. `detect` uses `exe_path.contains("claude")` — probes (b), (e), (f) all
   return `true`; the unit test asserts `false` for each.

**Harness location:** `monocle-runtime/tests/engine_module.rs`.

---

### §VP-ENGINE-002-ERR — `metadata`/`enrich` Return `HomeUnresolvable` with All Four Home-Env Vars Unset

**Traces to:** BC-ENGINE-002-ERR (SS-engine-module.md §Behavioral Contracts).

**Mechanical property:** When `HOME`, `USERPROFILE`, `HOMEDRIVE`, and
`HOMEPATH` are all unset (set to `None::<&str>` via `temp_env::with_vars` /
`async_with_vars`), `ClaudeCodeModule::metadata()` and
`ClaudeCodeModule::enrich(&snapshot)` both return
`Err(EngineMetadataError::HomeUnresolvable)`. The implementation MUST NOT
substitute a relative-path default, a current-directory fallback, or any
non-`HomeUnresolvable` error path.

**Mechanism:** unit-test (with `temp-env ^0.3` env-isolation, per
SS-deps-pin-manifest pin).

**Pre-conditions:**

- `temp-env = { version = "^0.3", features = ["async_closure"] }` in
  `[dev-dependencies]`.
- Test does NOT use `std::env::set_var` / `remove_var` directly; only
  `temp_env::with_vars` / `temp_env::async_with_vars` (RAII cleanup safe
  under panic and multi-threaded harness).

**Post-conditions:**

1. Sync half: inside `temp_env::with_vars([("HOME", None::<&str>),
   ("USERPROFILE", None::<&str>), ("HOMEDRIVE", None::<&str>),
   ("HOMEPATH", None::<&str>)], || { ... })`:
   - `module.metadata().is_err()` is `true`;
   - `matches!(module.metadata().unwrap_err(),
     EngineMetadataError::HomeUnresolvable)` is `true`.
2. Async half: inside `temp_env::async_with_vars([...], async { ... }).await`:
   - `module.enrich(&snapshot).await.is_err()` is `true`;
   - `matches!(module.enrich(&snapshot).await.unwrap_err(),
     EngineMetadataError::HomeUnresolvable)` is `true`.
3. Test passes on Linux and macOS CI runners deterministically. On Windows CI
   the test is best-effort (Windows may resolve `home_dir()` via
   `FOLDERID_Profile` regardless of env-var state); the Linux/macOS gates
   are the canonical assertion.

**Counter-example sketches:**

1. `metadata()` returns `Ok(EngineMetadata { home_dir: PathBuf::from("."), ... })`
   substituting `.` for unresolvable home — fails the `is_err` assertion.
2. `metadata()` returns `Err(EngineMetadataError::Io(...))` instead of
   `HomeUnresolvable` — fails the `matches!` assertion.
3. Test uses `std::env::remove_var` instead of `temp_env::with_vars` — the
   audit (a semgrep rule in SS-conventions-anti-patterns.md
   §Semgrep Rules `monocle-no-raw-env-mutation-in-tests`) fails the harness.
4. Test omits any of the four required env-vars (e.g., only clears `HOME`)
   — Windows may resolve via `USERPROFILE` even on Linux containers with
   `wine`-style env shimming; the test must clear all four.

**Harness location:** `monocle-runtime/tests/engine_module.rs` (alongside
VP-ENGINE-002).

---

### §VP-ENGINE-003 — `hook_paths()` Returns Exactly 5 Entries — One per `HookType` Variant

**Traces to:** BC-ENGINE-003 (SS-engine-module.md §Struct-level inherent operations).

**Mechanical property:** `ClaudeCodeModule::hook_paths()` returns a structure
containing exactly 5 entries, one per `HookType` variant. The path strings
are exactly:

| HookType variant | Path |
|------------------|------|
| `SessionStart` | `/hooks/session-start` |
| `UserPromptSubmit` | `/hooks/prompt-submit` |
| `PreToolUse` | `/hooks/pre-tool-use` |
| `Notification` | `/hooks/notification` |
| `Stop` | `/hooks/stop` |

**Mechanism:** unit-test.

**Pre-conditions:**

- `ClaudeCodeModule::new("http://127.0.0.1:7891".into())` constructs a module.
- `HookType` is the canonical 5-variant enum from `monocle_core::HookType`.

**Post-conditions:**

1. `module.hook_paths().len() == 5`.
2. For each `HookType` variant `v`, `module.hook_paths().get(&v)` returns
   `Some(&"/hooks/...".to_string())` matching the table above exactly.
3. No extra variants exist (the `match` over `HookType` is exhaustive in the
   harness — adding a 6th variant would fail to compile, which is the
   correct propagation given `#[non_exhaustive]` on `HookType` is for
   external consumers; the trait implementer (this crate) is internal and
   so `HookType` exhaustive matching is valid here).

**Counter-example sketches:**

1. `hook_paths()` returns 4 entries (missing one) — fails
   post-condition 1.
2. A path is typoed (`/hooks/pre_tool_use` with underscore instead of
   hyphen) — fails the exact-string match.
3. A new variant added to `HookType` (e.g., `PostToolUse`) without updating
   `hook_paths()` — the exhaustive match in the harness fails to compile,
   forcing the implementer to update.
4. `spawn()` or `preflight()` are accidentally moved into the
   `EngineModule` trait (they MUST remain inherent methods on
   `ClaudeCodeModule`) — fails an orthogonal source-grep check against
   `monocle-core/src/engine.rs`.
5. The ABI version is read via a trait method (e.g., `module.abi_version()`)
   instead of `monocle_core::MONOCLE_ABI_VERSION` const — fails an
   orthogonal source-grep check.

**Harness location:** `monocle-runtime/tests/engine_module.rs` (alongside
VP-ENGINE-002 and VP-ENGINE-002-ERR).

---

## §Coverage Matrix (BC → VP)

| BC ID | BC Source File | VP ID | Mechanism (primary) |
|-------|----------------|-------|---------------------|
| BC-RING-001 | SS-daemon-lifecycle.md v1.0.7 | VP-RING-001 | unit-test |
| BC-AUTH-001 | SS-daemon-lifecycle.md v1.0.7 | VP-AUTH-001 | unit-test |
| BC-AUTH-002 | SS-daemon-lifecycle.md v1.0.7 | VP-AUTH-002 | unit-test |
| BC-LOCK-001 | SS-daemon-lifecycle.md v1.0.7 | VP-LOCK-001 | unit-test |
| BC-ABI-001 | SS-core-types-and-abi.md v1.2.8 | VP-ABI-001 | unit-test |
| BC-ABI-002 | SS-core-types-and-abi.md v1.2.8 | VP-ABI-002 | unit-test |
| BC-TYPES-001 | SS-core-types-and-abi.md v1.2.8 | VP-TYPES-001 | unit-test |
| BC-FACTORY-001 | SS-core-types-and-abi.md v1.2.8 | VP-FACTORY-001 | unit-test |
| BC-FACTORY-002 | SS-core-types-and-abi.md v1.2.8 | VP-FACTORY-002 | unit-test |
| BC-PROTO-001a | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-001a | unit-test |
| BC-PROTO-001b | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-001b | unit-test |
| BC-PROTO-002 | SS-core-types-and-abi.md v1.2.8 | VP-PROTO-002 | unit-test |
| BC-ENGINE-001 | SS-engine-module.md v1.1.15 | VP-ENGINE-001 | unit-test |
| BC-ENGINE-002 | SS-engine-module.md v1.1.15 | VP-ENGINE-002 | unit-test |
| BC-ENGINE-002-ERR | SS-engine-module.md v1.1.15 | VP-ENGINE-002-ERR | unit-test |
| BC-ENGINE-003 | SS-engine-module.md v1.1.15 | VP-ENGINE-003 | unit-test |

**Coverage:** 16 BCs → 16 VPs (one-to-one). Zero BCs without a VP.

### §Auxiliary Mechanism Coverage

| VP ID | Auxiliary mechanism | Rationale |
|-------|---------------------|-----------|
| VP-RING-001 | mutation-test | `format_version: u32 = 1` is a high-leverage value-mutation target |
| VP-AUTH-001 | fuzz | Adversarial inputs to `validate_auth_token` must never produce a false-`true` |
| VP-AUTH-002 | fuzz | Same fuzz target as VP-AUTH-001 — exercises the prefix-rejection path |
| VP-LOCK-001 | mutation-test | `contract_version: u32 = 1` is a high-leverage value-mutation target |
| VP-TYPES-001 | mutation-test | EXEMPT list length and attribute-presence check are mutation surfaces |
| VP-FACTORY-002 | fuzz | `parse_frontmatter_field` was the v1.2.3 (F-R20-2) regression site; permanent fuzz harness prevents recurrence |
| VP-PROTO-002 | fuzz | Unknown schema-version dispatch must never panic across `u32::MAX` value space |

---

## §Open Verification Gaps

This section enumerates gaps where the BC catalog and the architecture
artifacts identify properties that are NOT formally verified by a Phase 1 VP.
Per CLAUDE.md §CANONICAL PRINCIPLE rule 3, each gap is anchored to a concrete
future story or wave — gaps are NOT generic deferrals.

### §G-1 — Kani Proof Harnesses for Phase 2 Trigger State Machine

**Status:** OPEN — pre-staged for Phase 2.

**Description:** Phase 1 BCs are deterministic protocol contracts that
require no model checking. Phase 2 introduces a `trigger-trace` state machine
whose invariants (no-deadlock, no-orphan-trigger, ring-buffer-monotonicity)
are natural Kani proof targets. This artifact does NOT pre-stage Kani
harnesses because the Phase 2 state-machine specification does not yet exist
(per STATE.md current phase `phase-1-spec-crystallization-entry-pending`).

**Future-attachment:** Phase 2 architecture artifact `SS-trigger-trace.md`
(to be authored during Phase 2 spec crystallization) MUST extend this
verification-properties catalog with Kani-based VPs. The Phase 2 PRD dispatch
explicitly enumerates this as a Phase 2 deliverable.

**Compensating Phase 1 coverage:** None required — there is no Phase 1
state machine. The Phase 1 daemon-lifecycle protocol is governed by
BC-DAEMON-004 (graceful shutdown) which has a `unit-test` Phase 6 coverage
plan; the daemon lifecycle is small enough that exhaustive unit testing
suffices.

### §G-2 — DTU Fidelity Scoring

**Status:** COVERED ELSEWHERE — see `dtu-assessment.md` §DTU Fidelity Measurement Procedure.

**Description:** DTU clone fidelity (target: ≥0.95 mean field-match score
against real Claude Code 2.x fixtures) is the verification path for the hook
protocol DTU clone. This is not a BC; it is a clone-quality measurement
governed by the `dtu-validator` agent.

**Future-attachment:** Wave 1 stories per `dtu-assessment.md` §Clone
Development Approach.

### §G-3 — Phase 4 OAuth2 / Federation Auth

**Status:** OUT OF PHASE 1 SCOPE.

**Description:** BC-AUTH-002 explicitly notes that Phase 4 federation OAuth2
tokens use a separate `Authorization: Bearer` header on a `monocle-ipc`
russh channel, NOT the Phase 1 `X-Monocle-Authorization` surface.
Verification of the Phase 4 federation auth path is a Phase 4 concern and
will receive its own VP (provisionally `VP-FED-AUTH-001`).

**Future-attachment:** Phase 4 architecture artifact (to be authored during
Phase 4 spec crystallization).

### §G-4 — `BC-DAEMON-001` through `BC-DAEMON-006` Verification

**Status:** SCOPED — covered by Phase 1 PRD verification-harness stubs;
not in this VP catalog because the task scope is the 16 architect-staged BCs.

**Description:** The daemon endpoints (BC-DAEMON-001 through
BC-DAEMON-006) are pre-staged in `SS-daemon-lifecycle.md` but are NOT in the
16-BC scope of this VP catalog (the architect's task allocation focuses on
the 16 cross-cutting type/auth/lock/ABI/factory/engine BCs). The Phase 1 PRD
will formalize them with the same per-BC verification-harness pattern used in
this artifact.

**Future-attachment:** Phase 1 PRD authoring (T-1, concurrent with this
task per orchestrator dispatch). The product-owner's PRD synthesis includes
verification-harness stubs for BC-DAEMON-* per the same `unit-test`
mechanism pattern. This VP catalog SHOULD be extended in a v1.1 revision to
include `VP-DAEMON-001` through `VP-DAEMON-006` once the PRD lands, OR the
PRD can register them as separate `VP-PRD-DAEMON-*` rows.

### §G-5 — Phase 1 Permission Enum Match-Site Coverage

**Status:** COVERED ELSEWHERE — by ADR-0004 + SS-permissions-phase1.md +
clippy lint configuration.

**Description:** `Phase1Permission` is exhaustive per ADR-0004. Match-site
correctness (every dispatch site covers every variant) is enforced by the
Rust compiler at compile time — no VP is required because the property is
discharged by `cargo check`. The clippy lint
`non_exhaustive_omitted_patterns` deny-listed via `#[allow(...)]` is a
separate concern covered by VP-TYPES-001's mutation-test auxiliary.

**Future-attachment:** N/A — discharged by compiler.

---

## §References (PG-5 historical-anchor framing)

The following cross-artifact references use position-free §-anchors and
either current-pointer version pinning or version-free anchors per
`SS-conventions-anti-patterns.md` §Historical-Anchor Framing Convention
(PG-5). All version pins below are current as of timestamp
`2026-05-14T20:30:00Z`.

1. `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.7 — source of
   BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001. Anchors:
   §Drain (BC-RING-001), §Start Sequence (BC-AUTH-001 + BC-AUTH-002 +
   BC-LOCK-001), §Behavioral Contract Summary (BC table).
2. `.factory/specs/architecture/SS-core-types-and-abi.md` v1.2.8 — source of
   BC-ABI-001, BC-ABI-002, BC-TYPES-001, BC-FACTORY-001, BC-FACTORY-002,
   BC-PROTO-001a, BC-PROTO-001b, BC-PROTO-002. Anchors:
   §ABI Version Constant, §Enum Extensibility, §FactoryAdapter Trait,
   §Prost Wire Schemas, §Phase 1 PRD BC Pre-Staging.
3. `.factory/specs/architecture/SS-engine-module.md` v1.1.15 — source of
   BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003. Anchors:
   §EngineModule Trait Signature, §Behavioral Contracts, §Phase 1
   Implementation, §Struct-level inherent operations.
4. `.factory/specs/architecture/SS-conventions-anti-patterns.md` —
   §Historical-Anchor Framing Convention (PG-5),
   §Section-Anchor Citation Convention (PG-4),
   §Cross-Section Directional Reference Convention (PG-3),
   §Schema-Fact Citation Convention (PG-1),
   §Phantom-ID Convention (PG-2),
   §META-Rule Recipe Sibling-Pattern Convention (PG-RECIPE-SCOPE),
   §Semgrep Rules, §Test Conventions.
5. `.factory/specs/architecture/SS-deps-pin-manifest.md` — canonical pins for
   `constant_time_eq ^0.3`, `temp-env ^0.3` (features = ["async_closure"]),
   `prost 0.14`, `serde_yaml_ng 0.10`, `serde_json 1`, `tracing 0.1`.
6. `.factory/specs/architecture/SS-permissions-phase1.md` — §Phase 1
   Permission Enum, §Exhaustiveness Invariant.
7. `.factory/specs/architecture/SS-forward-compatibility.md` — §Item P3-1 —
   Verdict on Sealed (open-trait rationale referenced by VP-FACTORY-001 and
   VP-ENGINE-001).
8. `.factory/specs/dtu-assessment.md` — §DTU Architecture (hook protocol
   surface), §DTU Fidelity Measurement Procedure (§G-2 deferral target).
9. `.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md` — Phase 3
   wasmtime 44 selection (informs §G-1 future Phase 2/Phase 3 Kani harness
   scope).
10. `.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md`
    — nucleo 0.5 acceptance (no Phase 1 VP impact; referenced for
    completeness).
11. `.factory/specs/architecture/adr/ADR-0003-license-selection.md` —
    license posture (no Phase 1 VP impact; referenced for completeness).
12. `.factory/specs/architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md`
    — VP-TYPES-001 exemption authority. The EXEMPT set
    `{Phase1Permission, ClaudeCodeTool}` is normatively defined here.
13. `CLAUDE.md` — §CANONICAL PRINCIPLE — Production-Grade Default,
    §Correct Agent Routing — The Production-Grade Companion Principle.

---

## §Trace

v1.0 (initial author, 2026-05-14):

- Authored the 16-VP catalog mapping every Phase 1 pre-staged BC to a
  formally-testable verification property.
- Selected `unit-test` as the primary mechanism for all 16 VPs. Rationale:
  Phase 1 BCs are deterministic protocol contracts (token formats, field
  presence, enum exhaustiveness, trait signatures) whose verification is
  fully discharged by deterministic unit tests. Kani model checking is not
  load-bearing in Phase 1; pre-staged for Phase 2 trigger-trace per §G-1.
- Selected 4 fuzz auxiliaries: VP-AUTH-001 + VP-AUTH-002 (single shared fuzz
  target `fuzz_auth_token_validation`), VP-FACTORY-002 (fuzz target
  `fuzz_state_md_parser` exercising the v1.2.3 F-R20-2 regression site),
  VP-PROTO-002 (fuzz target `fuzz_envelope_dispatch` over `u32::MAX`
  schema-version value space).
- Selected 3 mutation-test auxiliaries: VP-RING-001 (`format_version: u32 =
  1` mutation target), VP-LOCK-001 (`contract_version: u32 = 1` mutation
  target), VP-TYPES-001 (`EXEMPT` constant length and attribute-presence
  check mutation surface).
- Counter-example sketches per VP enumerate adversarial inputs that should
  refute the property. Each sketch maps to a concrete past adversary
  finding where one exists (e.g., VP-FACTORY-002 sketch 3 traces to
  F-R20-2 v1.2.3 fix; VP-ENGINE-001 sketch 2 traces to F-R28-1 v1.1.8 fix;
  VP-AUTH-001 sketch 1 traces to FC-06 token-format design).
- Coverage matrix: 16 BCs → 16 VPs (1:1). Open gaps catalogued with
  future-attachment per CLAUDE.md §CANONICAL PRINCIPLE rule 3.
- PG-5 historical-anchor compliance: every cross-artifact citation in
  §References pins a current version (`v1.0.7`, `v1.2.8`, `v1.1.15`) per
  the timestamp `2026-05-14T20:30:00Z`. PG-4 §-anchor compliance: every
  cited §-anchor resolves to an actual heading in the target artifact.
  PG-3 directional compliance: no `above/below` directional qualifiers
  appear in §References. PG-1 schema-fact compliance: dependency pins
  (`temp-env ^0.3`, `constant_time_eq ^0.3`, etc.) reference
  SS-deps-pin-manifest.md as the canonical source.
- This file is the formal-verifier deliverable for Phase 1 PRD pre-staging
  per orchestrator dispatch T-2 (concurrent with product-owner PRD
  synthesis T-1 per STATE.md §Phase 1 dispatch).
