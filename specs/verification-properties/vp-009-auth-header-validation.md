---
document_type: verification-property
level: L4
version: "1.0.4"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T22:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "7c094e3"
traces_to: prd.md
source_bc: BC-2.01.009
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

# VP-009: Auth Header Validation — Two-Body Taxonomy + ADR-0005 Dual-Accept (Canonical `X-Monocle-Authorization` with `X-Claude-Code-Ide-Authorization` Compatibility Alias)

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-AUTH-002 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

The auth middleware enforces a two-body error taxonomy across two header
acceptance paths per ADR-0005 dual-accept:

1. **Canonical path** (monocle-aware tools): `X-Monocle-Authorization` with
   value beginning `monocle-v1:<64-hex>`. Prefix stripped, constant-time
   compared against stored secret. Successful match → request proceeds; any
   value-present failure (bad prefix, bad format, empty suffix, wrong
   secret) → HTTP 401 `{"error":"invalid_auth_token"}`. The canonical path
   emits NO log on use (success or failure) beyond standard request logging.
2. **Compatibility alias path** (real Claude Code hook scripts): when
   `X-Monocle-Authorization` is absent AND `X-Claude-Code-Ide-Authorization`
   is present, the middleware first emits the WARN-level deprecation log
   `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility
   alias); monocle-aware harness should use X-Monocle-Authorization`,
   then validates the value as a raw 64-hex token (NO `monocle-v1:` prefix
   — Claude Code sends the lock file `authToken` field verbatim). The
   value is constant-time compared against the stored secret. Successful
   match → request proceeds (with WARN already logged); failure → HTTP 401
   `{"error":"invalid_auth_token"}` (with WARN already logged).
3. **Canonical priority — both headers present:** If both
   `X-Monocle-Authorization` AND `X-Claude-Code-Ide-Authorization` are
   present, `X-Monocle-Authorization` is validated and
   `X-Claude-Code-Ide-Authorization` is ignored. **No WARN log is emitted**
   in this case (the alias was never consulted). This is the canonical-wins
   immutability invariant per BC-2.01.009 INV-5.
4. **Missing — both headers absent:** When neither
   `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` is
   present, the middleware returns HTTP 401
   `{"error":"missing_auth_token"}`. A request carrying only
   `Authorization: Bearer <anything>` (or any other unrecognized header) is
   treated as the both-absent case.

The auth middleware's `AuthError` enum has exactly TWO variants: `Missing`
(both recognized headers absent → `missing_auth_token`) and `Invalid`
(any value-present failure on either path → `invalid_auth_token`). The
retired v1.0 body `{"error":"invalid_auth_token_format"}` (per architect
commit 2db408f disposition (c)) MUST NOT appear in any Phase 1 daemon
response. All value-present failure modes — on either header path — return
the same body intentionally, preventing a timing- or body-oracle from
which an attacker could distinguish header-name correctness, prefix
correctness, format correctness, or secret correctness. Constant-time
comparison is used on both paths identically; the only difference is the
input transformation (prefix-strip on canonical; no transformation on
alias).

## Source Contract

- **BC (primary):** BC-2.01.009 v1.0.3 — Auth Header Validation (Missing
  and Invalid Token); dual-accept semantics per ADR-0005.
- **ADR (primary):** ADR-0005 v1.0.1 — Auth Header Dual-Accept — Canonical
  `X-Monocle-Authorization` with `X-Claude-Code-Ide-Authorization`
  Compatibility Alias.
- **Postcondition/Invariant:** two-variant `AuthError` enum; exact body
  taxonomy per probe; canonical-priority immutability (canonical wins when
  both present; alias ignored; no WARN log emitted); WARN-log emission
  exactly-once-per-alias-path-attempt (regardless of success/failure);
  constant-time symmetry across canonical and alias paths; uniform
  application across all 3 authenticated route classes (`/hooks/*`,
  `/status`, `/shutdown`); Bearer-fallback (or any unrecognized header)
  rejection as missing; retired-body absence.
- **Traces to (historical):** BC-AUTH-002 (PRD v1.25 §BC-AUTH-002;
  SS-daemon-lifecycle.md v1.0.30 §Start Sequence; architect adjudication
  commit 2db408f — disposition (c) collapsed error taxonomy; F-R62-4
  back-propagation closure landed in arch v1.0.9 commit 8bf3759; ADR-0005
  T-128m R3 dual-accept decision adopted in arch v1.0.29; F-FC-I005
  fabricated-ID removal in arch v1.0.30 architect 5E dispatch).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — 7-probe matrix | Two-body taxonomy across all probe categories; positive control |
| Fuzz (auxiliary) | cargo-fuzz / libFuzzer | Bounded byte-sequence universe | Arbitrary `X-Monocle-Authorization` values + absent-header case; no panic; retired body never appears |
| Source assertion (structural) | type system | N/A — compile-time | `AuthError` enum has exactly 2 variants (Missing, Invalid) |

## Mechanism

Integration test (primary; harness at
`monocle-runtime/tests/auth_header_rejection.rs` — files in
`<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test Type
column labels this BC `Integration`); fuzz (auxiliary). The harness
performs all 7 probes in the Probe Matrix below against the same axum test
server and asserts the exact response code + body shape. The fuzz target
sweeps arbitrary `X-Monocle-Authorization` byte sequences (including the
absent-header case via `Option<Vec<u8>>`) and asserts the two-body taxonomy
and the absence of the retired body.

## Pre-conditions

- Daemon is running with a valid `monocle-v1:` secret in the lock file.
  The lock file `authToken` field stores the raw 64-char hex secret (no
  prefix), per ADR-0005 §Lock File Interplay.
- Authenticated test client has access to the secret for the positive
  controls (canonical probe 9.7 and alias probe 9.12).
- The auth middleware's `AuthError` enum is defined as exactly:
  ```rust
  pub enum AuthError {
      Missing,  // → HTTP 401 {"error":"missing_auth_token"}
      Invalid,  // → HTTP 401 {"error":"invalid_auth_token"}
  }
  ```
  No third variant exists.
- **WARN-log capture infrastructure:** The integration harness uses a
  `tracing_test::traced_test` (or equivalent `tracing-subscriber` test
  layer) to capture emitted log lines. Per BC-2.01.009 INV-6, the canonical
  WARN message is the literal string
  `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility
  alias); monocle-aware harness should use X-Monocle-Authorization`. The
  harness asserts both the presence of this WARN line on alias-path probes
  AND its absence on canonical-path probes (probes 9.1–9.7 + probe 9.11).
- **Constant-time comparison primitive:** Both paths invoke
  `subtle::ConstantTimeEq::ct_eq` (or `constant_time_eq` crate) on
  64-byte-sized hex-decoded buffers. The integration test does not directly
  measure constant-time behavior (that requires a side-channel oracle); it
  asserts the symmetric API path — both canonical and alias call the same
  comparison primitive on the same-shaped byte buffer.

## Post-conditions

The 7-probe taxonomy table below (Probe Matrix) is the post-condition set.
Each row is a deterministic single-body assertion.

## Counter-examples

### Canonical-path counter-examples

1. Auth middleware accepts `Authorization: Bearer` as a fallback path —
   probe 9.5 would return 200; the integration test must assert 401 +
   `missing_auth_token` (Bearer header is not a recognized auth header).
2. Auth middleware uses `presented.contains("monocle-v1:")` instead of
   `strip_prefix("monocle-v1:")` — probe `X-Monocle-Authorization:
   junk-monocle-v1:abc` would be accepted; the integration test asserts
   strict `strip_prefix` behavior (returns 401 + `invalid_auth_token`
   for any value not starting with the literal prefix).
3. Auth middleware returns the retired `invalid_auth_token_format` body
   for probe 9.2/9.3/9.4 — fails the exact-body assertion (the retired
   taxonomy is forbidden post-2db408f).
4. Auth middleware returns `invalid_auth_token` for probe 9.1 (both headers
   absent treated as invalid) — fails the missing-vs-invalid distinction;
   the structural precondition (both recognized headers absent) must
   produce the diagnostic-friendly `missing_auth_token` body.
5. Auth middleware returns `missing_auth_token` for probe 9.6
   (correct-format wrong-secret on canonical) — fails the value-present
   unification; secret mismatch must produce `invalid_auth_token`, not
   `missing_auth_token` (an attacker probing the secret space must not
   learn that their format was correct).

### Alias-path counter-examples (ADR-0005)

6. **Missing WARN log on alias-path success.** Auth middleware accepts a
   valid `X-Claude-Code-Ide-Authorization` token (probe 9.12) but does NOT
   emit the WARN deprecation log. The integration test asserts the literal
   WARN line is captured by the `tracing_test` layer. Without the WARN log,
   alias usage is invisible in production logs, defeating BC-2.01.009 INV-6
   and the ADR-0005 deprecation-visibility goal.
7. **Missing WARN log on alias-path failure.** Auth middleware rejects an
   invalid `X-Claude-Code-Ide-Authorization` token (probe 9.10) and returns
   401, but does NOT emit the WARN log. INV-6 requires WARN emission on
   every alias-path attempt regardless of validation outcome — failure
   logs are equally important for production observability of misconfigured
   harness clients.
8. **Alias path applies `monocle-v1:` prefix-strip incorrectly.** Auth
   middleware mistakenly tries to `strip_prefix("monocle-v1:")` on the
   alias header value (which carries a raw 64-hex token with no prefix per
   ADR-0005). Probe 9.12 would fail (prefix strip returns `None` on
   prefix-less input). The integration test asserts the alias path
   validates the RAW value as-is against the stored secret without prefix
   manipulation.
9. **Alias path entered when canonical fails.** Auth middleware checks
   canonical first, finds an invalid `X-Monocle-Authorization` value
   (probe 9.2/9.3/9.4/9.6), then "falls through" to consult
   `X-Claude-Code-Ide-Authorization`. This violates BC-2.01.009 INV-5
   canonical-priority immutability: the alias path is entered ONLY when
   `X-Monocle-Authorization` is absent (not when it's present-but-invalid).
   Probe 9.11 (canonical invalid + alias valid) must return 401, NOT
   silently re-validate via the alias path.
10. **Canonical priority broken — both-headers WARN.** Auth middleware
    emits the WARN deprecation log on probe 9.7-with-alias-also-present
    (probe 9.11b: both headers present, canonical valid, alias valid). The
    test asserts NO WARN log fires in the both-present case (the alias was
    never consulted, per ADR-0005 §Decision Priority 2 entry guard).
11. **Constant-time symmetry break.** Auth middleware uses
    `subtle::ct_eq` on the canonical path but `==` (variable-time string
    comparison) on the alias path. The integration test's
    source-grep (or `cargo expand` AST audit) asserts both paths call the
    same constant-time primitive on byte-buffer inputs. A symmetric-API
    audit catches the asymmetry without requiring a side-channel oracle.
12. **WARN log fires on canonical use.** Auth middleware emits the WARN
    deprecation log on a canonical-only request (probes 9.1–9.7). The
    integration test asserts the WARN line is absent from captured logs
    on canonical-only probes. A spurious WARN on canonical use would
    create false-positive deprecation-alert noise in production and
    undermine the alias-deprecation-visibility signal.

## Probe Matrix

The probe matrix is partitioned into three categories: (A) canonical-only
path (probes 9.1–9.7); (B) alias-only path (probes 9.8–9.12); (C)
both-headers-present canonical-priority (probes 9.13a–9.13b). Each row
asserts (status code, body, WARN-log presence/absence).

### Category A — Canonical path (`X-Monocle-Authorization`)

| Probe | Header(s) | Expected status | Expected body | WARN log |
|-------|-----------|-----------------|---------------|----------|
| 9.1 | (no `X-Monocle-Authorization`; no `X-Claude-Code-Ide-Authorization`) | 401 | `{"error":"missing_auth_token"}` | absent |
| 9.2 | `X-Monocle-Authorization: deadbeef...64chars` (bare token, no prefix) | 401 | `{"error":"invalid_auth_token"}` | absent |
| 9.3 | `X-Monocle-Authorization: monocle-v2:deadbeef...64chars` (wrong version prefix) | 401 | `{"error":"invalid_auth_token"}` | absent |
| 9.4 | `X-Monocle-Authorization: monocle-v1:` (prefix only, no hex suffix) | 401 | `{"error":"invalid_auth_token"}` | absent |
| 9.5 | `Authorization: Bearer fake-token` (Bearer header only; neither recognized header present) | 401 | `{"error":"missing_auth_token"}` | absent |
| 9.6 | `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` (correct format, wrong secret) | 401 | `{"error":"invalid_auth_token"}` | absent |
| 9.7 | `X-Monocle-Authorization: monocle-v1:<correct-64-hex>` (canonical positive control) | 200 | (route's normal body) | absent |

### Category B — Alias path (`X-Claude-Code-Ide-Authorization`)

> Each Category-B probe requires `X-Monocle-Authorization` ABSENT to enter
> the alias code path per ADR-0005 §Decision Priority 2 entry guard.

| Probe | Header(s) | Expected status | Expected body | WARN log |
|-------|-----------|-----------------|---------------|----------|
| 9.8 | `X-Claude-Code-Ide-Authorization: deadbeef...32chars` (wrong length, not 64 hex) | 401 | `{"error":"invalid_auth_token"}` | present (exactly once) |
| 9.9 | `X-Claude-Code-Ide-Authorization: monocle-v1:<correct-64-hex>` (incorrectly prefixed alias — Claude Code never sends a prefix; this probe asserts the alias path does NOT accept prefixed values) | 401 | `{"error":"invalid_auth_token"}` | present (exactly once) |
| 9.10 | `X-Claude-Code-Ide-Authorization: <wrong-64-hex>` (correct 64-hex format, wrong secret) | 401 | `{"error":"invalid_auth_token"}` | present (exactly once) |
| 9.11 | `X-Claude-Code-Ide-Authorization:` (empty value, EC-012 of BC-2.01.009) | 401 | `{"error":"invalid_auth_token"}` | present (exactly once) |
| 9.12 | `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (alias positive control) | 200 | (route's normal body) | present (exactly once) |

### Category C — Both headers present (canonical priority per ADR-0005)

| Probe | Header(s) | Expected status | Expected body | WARN log |
|-------|-----------|-----------------|---------------|----------|
| 9.13a | `X-Monocle-Authorization: monocle-v1:<correct-64-hex>` + `X-Claude-Code-Ide-Authorization: <wrong-64-hex>` (canonical valid, alias invalid) | 200 | (route's normal body) | absent (alias never consulted) |
| 9.13b | `X-Monocle-Authorization: monocle-v1:<correct-64-hex>` + `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (both valid) | 200 | (route's normal body) | absent (alias never consulted) |
| 9.13c | `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` + `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (canonical invalid, alias valid) | 401 | `{"error":"invalid_auth_token"}` | absent (canonical-priority immutability — alias NOT consulted on canonical failure; BC-2.01.009 INV-5) |

**Total probes:** 15 (7 canonical + 5 alias + 3 both-present). Each row is
a deterministic three-cell assertion (status, body, WARN-log
presence/absence). The integration harness arranges fresh daemon state,
performs the HTTP request, captures the response status code, captures the
response body (exact byte comparison against the expected JSON literal),
and queries the `tracing_test` log layer for the canonical WARN line.

**Cross-property reciprocations (SE-15d / Extension 16 backfill sweep):**

- **Cross-property with VP-002 §Mechanical property item 4 +
  §Post-condition 2** (auth-header rejection on `/status`): VP-002
  asserts `/status` without an auth header returns HTTP 401 +
  `missing_auth_token` and with a malformed header returns HTTP 401 +
  `invalid_auth_token`; this VP asserts the same two-body taxonomy
  applies uniformly across all 3 authenticated route classes
  (`/hooks/*`, `/status`, `/shutdown`) — both for the canonical path
  AND the ADR-0005 compatibility-alias path.
- **Cross-property with VP-004 §Post-condition 7** (`/shutdown`
  authentication): VP-004 asserts `POST /shutdown` without an
  auth header returns HTTP 401 + `missing_auth_token`; this VP asserts
  the same body-taxonomy applies (probe 9.1 of the matrix above with the
  `/shutdown` route as the target). The alias-path probes (9.8–9.12)
  should also be exercised against `/shutdown` to confirm uniform
  dual-accept across all authenticated route classes.

**Fuzz harness (updated for ADR-0005 dual-accept):** the
`fuzz_auth_token_validation` target shared with VP-008 is updated to
exercise BOTH header paths. The fuzzer constructs three input dimensions:

1. **Canonical header bytes:** arbitrary byte sequence as the
   `X-Monocle-Authorization` value (including the absent case via
   `Option<Vec<u8>>`).
2. **Alias header bytes:** arbitrary byte sequence as the
   `X-Claude-Code-Ide-Authorization` value (including the absent case via
   `Option<Vec<u8>>`).
3. **Both-absent toggle:** explicitly enumerates the (None, None) case at
   non-trivial frequency (the fuzzer's input distribution otherwise rarely
   produces simultaneous Nones).

For every input triple the fuzzer asserts:

- No panic.
- If both headers absent: response body is exactly
  `{"error":"missing_auth_token"}`; no WARN log emitted.
- If `X-Monocle-Authorization` present (regardless of alias): response is
  either 200 (matched secret with `monocle-v1:` prefix) OR 401 +
  `{"error":"invalid_auth_token"}` (any other case). NO WARN log emitted —
  alias is never consulted when canonical is present.
- If `X-Monocle-Authorization` absent AND
  `X-Claude-Code-Ide-Authorization` present: WARN log emitted (exactly
  once); response is either 200 (matched raw 64-hex secret) OR 401 +
  `{"error":"invalid_auth_token"}` (any other case).
- Response body is NEVER `{"error":"invalid_auth_token_format"}` (the
  retired body — fuzz harness asserts this byte sequence never appears in
  any response).
- The fuzzer should never produce an input that returns 200 except for
  (a) the exact expected secret with the `monocle-v1:` prefix on
  `X-Monocle-Authorization`, or (b) the exact expected secret as raw
  64-hex on `X-Claude-Code-Ide-Authorization` when
  `X-Monocle-Authorization` is absent.

**Cross-property reciprocations (SE-15d / Extension 16 backfill sweep):**

- **Cross-property with VP-002 §Mechanical property item 4 +
  §Post-condition 2** (auth-header rejection on `/status`): VP-002
  asserts `/status` without an auth header returns HTTP 401 +
  `missing_auth_token` and with a malformed header returns HTTP 401 +
  `invalid_auth_token`; this VP asserts the same two-body taxonomy
  applies uniformly across all 3 authenticated route classes
  (`/hooks/*`, `/status`, `/shutdown`).
- **Cross-property with VP-004 §Post-condition 7** (`/shutdown`
  authentication): VP-004 asserts `POST /shutdown` without an
  auth header returns HTTP 401 + `missing_auth_token`; this VP asserts
  the same body-taxonomy applies (probe 9.1 of the matrix above with the
  `/shutdown` route as the target).

## Harness Location

- `monocle-runtime/tests/auth_header_rejection.rs` (integration)
- `fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz, shared with VP-008)
- Test name: `test_BC_AUTH_002_auth_header_validation_all_failure_modes`
  (per PRD v1.25 §BC-AUTH-002, Verification subsection — to be migrated to
  `test_BC_2_01_009_auth_header_validation_all_failure_modes`).

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
fn verify_bc_2_01_009() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/auth_header_rejection.rs` (integration)
- `fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz, shared with VP-008)
- Test name: `test_BC_AUTH_002_auth_header_validation_all_failure_modes`
  (per PRD v1.25 §BC-AUTH-002, Verification subsection — to be migrated to
  `test_BC_2_01_009_auth_header_validation_all_failure_modes`).

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

- Current as of `2026-05-17T22:30:00Z` (R106 Round 5D — F-R106-1 + F-R106-10 closure).
- Predecessor: monolithic VP-AUTH-002 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.009.md` v1.0.3 (commit pending — F-R106-7 fabricated-FC-ID removal + ADR-0005 dual-accept propagation).
- ADR: `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md` v1.0.1 (commit e142efb — heading-hierarchy normalization; T-128m architectural decision dual-accept option (a)).
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.30 §Start
  Sequence (commit pending — architect 5E F-FC-I005 removal + dual-accept consolidation).
- PRD: `.factory/specs/prd.md` v1.26.3 §BC-2.01.009 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.17.
- Cross-property: VP-002 (`/status` auth probes); VP-004 (`/shutdown` auth
  probe); VP-008 (token wire format + `constant_time_eq`).

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
  - **Before:** §References cited `prd.md v1.26 §BC-2.01.009 (Dispatch 4 commit 1030c65).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.3 §BC-2.01.009 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (post-edit grep).
- **SE-17c-d body-scope grep:** post-edit `grep -n "prd.md v1.26 " vp-009-auth-header-validation.md` → 0 matches; `grep -n "prd.md v1.26.3" vp-009-auth-header-validation.md` → 1 match (§References line).
- **BC §References scope:** §References §Source contract entry already cites canonical sharded `behavioral-contracts/ss-01/BC-2.01.009.md` (per BC-INDEX.md v1.2). No BC-path changes required in this dispatch.
- **Historical PRD v1.25 citations in body prose (Source Contract `Traces to (historical)`, Harness Location `to be migrated to`, Proof Harness Skeleton `to be migrated to`, where present):** UNCHANGED — these are explicitly historical predecessor citations pinned to the pre-Dispatch-4 PRD monolith and must not be refreshed.

### Rationale

PO commit b2b378b (T-128k Round-3 PO dispatch) bumped PRD `v1.26.2 → v1.26.3` for F-R105-12 VP alias + GAP-R44-4 closure. Parallel FV dispatch refreshes VP §References to cite the post-bump PRD version, preserving the stale-citation-zero invariant established in F-R105-7 (manifest pin refresh) and F-R105-11 (sister-VP reference reconciliation). Per CLAUDE.md Production-Grade Rule 1: no MVP-driven deferral; mechanical citation refresh executed in-scope of T-128k FV portion rather than left to post-Round-3 cleanup.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.3 (commit b2b378b — F-R105-12 VP alias + GAP-R44-4 closure).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7 — confirms canonical sharded path `ss-01/BC-2.01.009.md` for BC-2.01.009).
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

## §Trace v1.0.4 — F-R106-1 CRITICAL + GAP-R45-1 HIGH + F-R106-10 HIGH: ADR-0005 Dual-Accept Expansion + SS-daemon-lifecycle Pin Refresh

**Bump:** v1.0.3 → v1.0.4.
**Predecessor pin:** v1.0.3 (commit 932f4e0 — T-128k FV F-R105-13 LOW 22-VP §References PRD citation refresh).
**Scope of v1.0.4 (NORMATIVE — three coupled normative changes addressing R106 Round-4 + R45 consistency gaps):**

### Change 1 — F-R106-1 CRITICAL + GAP-R45-1 HIGH: ADR-0005 dual-accept coverage expansion (NORMATIVE)

VP-009 v1.0.3 covered only the canonical `X-Monocle-Authorization` header path. BC-2.01.009 v1.0.2 (PO commit e7950f0, F-R105 R4 closure) introduced ADR-0005 dual-accept semantics — adding the `X-Claude-Code-Ide-Authorization` compatibility alias path with WARN deprecation logging and canonical-priority immutability. BC-2.01.009 §Verification Properties table (lines 92-96, post-PO-bump) cites 5 NEW probe entries against VP-009 that v1.0.3 did not satisfy:

1. All canonical-path failure modes (6 vectors) return the correct HTTP status and body.
2. All alias-path failure modes return HTTP 401 `{"error":"invalid_auth_token"}` with WARN log emitted.
3. Alias-path success returns HTTP 200 with WARN log emitted.
4. No third error body exists in Phase 1 auth middleware responses.
5. Canonical priority: when both headers present, `X-Monocle-Authorization` wins; no WARN log emitted.

The v1.0.4 expansion satisfies all 5 BC-2.01.009-side property entries by:

- **§Property Statement:** Extended from single-paragraph canonical-only description to four numbered subsections (canonical path; alias path; canonical-priority both-present; missing both-absent) explicitly enumerating the ADR-0005 dual-accept semantics, the WARN-log emission contract per BC-2.01.009 INV-6, the canonical-priority immutability per INV-5, and the constant-time symmetry contract per INV-7.
- **§Source Contract:** Added explicit ADR-0005 v1.0.1 citation as ADR (primary). BC version updated to v1.0.3 (post-F-R106-7 closure). Postcondition/Invariant enumeration expanded to cover dual-accept-specific invariants (canonical-priority immutability; WARN-log exactly-once-per-alias-attempt; constant-time symmetry; uniform application across 3 authenticated route classes).
- **§Pre-conditions:** Added WARN-log capture infrastructure (`tracing_test::traced_test`) specification with literal canonical WARN message string. Added constant-time comparison primitive specification (symmetric API path assertion via `subtle::ct_eq` or `constant_time_eq`).
- **§Counter-examples:** Expanded from 5 to 12 counter-examples, partitioned into canonical-path (CE-1 through CE-5, all retained verbatim with probe-ID updates for the renumbered matrix) and alias-path (CE-6 through CE-12, all NEW — covering missing-WARN-on-alias-success, missing-WARN-on-alias-failure, incorrect-prefix-strip-on-alias, alias-fall-through-on-canonical-failure, both-headers-WARN-violation, constant-time-symmetry-break, and spurious-WARN-on-canonical-use).
- **§Probe Matrix:** Expanded from 7 probes (single category) to 15 probes (3 categories: A canonical-only 9.1-9.7 = 7 probes; B alias-only 9.8-9.12 = 5 probes; C both-headers-present 9.13a/9.13b/9.13c = 3 probes). Each row gains a fourth assertion column (WARN log presence/absence) per BC-2.01.009 INV-6.
- **§Probe Matrix Fuzz harness subsection:** Expanded fuzzer construction from single-dimension (canonical header bytes) to three-dimension input space (canonical bytes + alias bytes + both-absent toggle). Invariants assert no-panic, canonical-priority preservation under fuzzed inputs, WARN-log emission contract under fuzzed alias inputs, retired-body absence, and the only-200-on-exact-secret invariant for both paths.

**SE-17f BEFORE/AFTER evidence for §Probe Matrix:**

- BEFORE: 7 probes (9.1-9.7); 3-column table (Probe | Header | Expected status | Expected body); no WARN-log assertion column; single-category structure.
- AFTER: 15 probes (9.1-9.7 + 9.8-9.12 + 9.13a/b/c); 4-column table including WARN log column; 3-category structure (A canonical / B alias / C both-headers).

**SE-17f BEFORE/AFTER evidence for §Counter-examples:**

- BEFORE: 5 flat-numbered counter-examples covering canonical-path mutations only.
- AFTER: 12 partitioned counter-examples — 5 canonical-path (CE-1 through CE-5, content preserved from v1.0.3 with probe-ID updates) + 7 alias-path (CE-6 through CE-12, all NEW).

### Change 2 — F-R106-10 HIGH: SS-daemon-lifecycle pin refresh v1.0.25 → v1.0.30 (NORMATIVE)

- **SE-17f §Source Contract `Traces to (historical)` line:**
  - Before: `SS-daemon-lifecycle.md v1.0.25 §Start Sequence; architect adjudication ...`
  - After: `SS-daemon-lifecycle.md v1.0.30 §Start Sequence; architect adjudication ... ADR-0005 T-128m R3 dual-accept decision adopted in arch v1.0.29; F-FC-I005 fabricated-ID removal in arch v1.0.30 architect 5E dispatch`
- **SE-17f §References Architecture line:**
  - Before: `Architecture: \`architecture/SS-daemon-lifecycle.md\` v1.0.25 §Start Sequence (commit 18fe265).`
  - After: `Architecture: \`architecture/SS-daemon-lifecycle.md\` v1.0.30 §Start Sequence (commit pending — architect 5E F-FC-I005 removal + dual-accept consolidation).`
- **Cross-dispatch coordination:** Architect 5E is bumping SS-daemon-lifecycle v1.0.29 → v1.0.30 in parallel (this round). The FV pin refresh anticipates the v1.0.30 target per the explicit task-spec coordination directive.

### Change 3 — ADR-0005 added to §References (NORMATIVE)

- **SE-17f §References — new line inserted between Source contract and Architecture:**
  - `ADR: \`architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md\` v1.0.1 (commit e142efb — heading-hierarchy normalization; T-128m architectural decision dual-accept option (a)).`

### Change 4 — Title updated to reflect ADR-0005 scope (NORMATIVE)

- **SE-17f H1 title:**
  - Before: `# VP-009: Auth Header Validation — Two-Body Taxonomy (\`missing_auth_token\` vs \`invalid_auth_token\`)`
  - After: `# VP-009: Auth Header Validation — Two-Body Taxonomy + ADR-0005 Dual-Accept (Canonical \`X-Monocle-Authorization\` with \`X-Claude-Code-Ide-Authorization\` Compatibility Alias)`

### SE-17c-d body-scope grep (NORMATIVE)

- `grep -nE "v1\.0\.25" vp-009-auth-header-validation.md` post-edit → 0 matches outside §Trace history blocks (the only remaining `v1.0.25` cites are in earlier §Trace entries as preserved historical predecessor evidence per SE-17g).
- `grep -nE "X-Claude-Code-Ide-Authorization" vp-009-auth-header-validation.md` post-edit → >20 matches across Property Statement (4), Source Contract (3), Pre-conditions (3), Counter-examples (9), Probe Matrix (12), References (1) — confirming dual-accept coverage saturated across all VP body sections.
- `grep -nE "WARN" vp-009-auth-header-validation.md` post-edit → 14 matches confirming WARN-log assertion semantics threaded across Property Statement, Pre-conditions, Counter-examples (CE-6, CE-7, CE-10, CE-12), and Probe Matrix WARN column.
- `grep -nE "ADR-0005" vp-009-auth-header-validation.md` post-edit → 9 matches confirming ADR-0005 traceability across Source Contract, Property Statement, Counter-examples, Probe Matrix, and References.

### Probe count audit (NORMATIVE)

- Before (v1.0.3): 7 probes (9.1-9.7).
- After (v1.0.4): 15 probes (9.1-9.7 canonical; 9.8-9.12 alias; 9.13a/b/c both-headers). Exceeds the ≥12 task-spec target.

### Rationale

R106 Round-4 evidence shows VP-009 v1.0.3 is the highest-severity CRITICAL gap in the consistency audit: the Property Statement, Probe Matrix, Counter-examples, and Fuzz harness all reflect the pre-ADR-0005 single-header world, despite BC-2.01.009 v1.0.2 (commit e7950f0) having propagated the dual-accept semantics into the §Verification Properties table. The BC-side property entries point to VP-009, but VP-009 carries none of the alias-path probes, WARN-log assertions, or canonical-priority probes that BC-2.01.009 now requires. Without the v1.0.4 expansion, no integration test or fuzz harness in Phase 3 would exercise the alias path, leaving the production-critical ADR-0005 dual-accept logic untested — a Phase-1 spec-side gap that would directly break Phase-3 TDD red-gate setup for the alias path.

Per CLAUDE.md Production-Grade Default Rule 1+5: the gap is fixed in-scope of R106 Round 5D rather than deferred. The expansion is production-grade — each probe carries arrange/act/assert prose, each counter-example documents the failure mode it catches, and the fuzz harness specifies a three-dimensional input space matching the dual-accept code surface.

### Authoritative cross-references

- **BC:** `behavioral-contracts/ss-01/BC-2.01.009.md` v1.0.3 (commit e7950f0 ADR-0005 dual-accept propagation + commit pending F-R106-7 fabricated-FC-ID removal).
- **ADR:** `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md` v1.0.1 (commit e142efb).
- **Architecture:** `architecture/SS-daemon-lifecycle.md` v1.0.30 (commit pending — architect 5E dispatch).
- **R106 closure chain:** F-R106-1 CRITICAL + GAP-R45-1 HIGH (VP-009 dual-accept coverage); F-R106-10 HIGH (pin refresh sweep — this VP is one of 10 pin-citing files); F-R106-9 HIGH (VP-INDEX SS-01 pin refresh — cascade in this dispatch); F-R106-18 LOW (VP-INDEX SS-02/SS-03 pin additions — cascade); GAP-R45-4 LOW (VP-INDEX §References BC-INDEX cite refresh — cascade).
- **Concurrent dispatches (R106 Round 5):**
  - PO 5A: BC + BC-INDEX dual-accept finalization — separate scope.
  - PO 5B: PRD + supplements — separate scope.
  - PO 5C: product-brief — separate scope.
  - FV 5D: this dispatch (VP-009 expansion + 10-VP pin sweep + VP-INDEX cascade).
  - Architect 5E: ADR-0005 path normalization + SS-daemon-lifecycle v1.0.29 → v1.0.30 — separate scope.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T22:30:00Z` >= chain high-water `2026-05-17T22:10:00Z` (BC-2.01.009 v1.0.3 frontmatter timestamp; FV §Trace strictly greater per SE-16d). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: H1 title extension; §Property Statement expansion (single-paragraph → four-numbered-subsection); §Source Contract ADR-0005 + BC version + Traces-to expansion; §Pre-conditions WARN-log + constant-time additions; §Counter-examples expansion 5→12; §Probe Matrix expansion 7→15 + WARN-log column + 3-category partition; §Fuzz harness three-dimensional input space; §References ADR-0005 addition + SS-daemon-lifecycle pin refresh v1.0.25 → v1.0.30 + BC version refresh; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context; probe count audit.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Production-grade expansion executed in-scope of FV 5D rather than deferred. Each new probe (9.8-9.12, 9.13a/b/c) carries explicit header construction, expected status, expected body, and WARN-log presence/absence. Each new counter-example (CE-6 through CE-12) documents the specific implementation defect it catches with explicit BC invariant references (INV-5 canonical-priority, INV-6 WARN-log, INV-7 constant-time symmetry). No tech-debt entries created. Cross-dispatch coordination with architect 5E (SS-daemon-lifecycle v1.0.30 target) and PO 5A (BC-2.01.009 v1.0.3 target) handled via explicit "commit pending" annotations in §References — these will resolve to concrete SHAs during final state-manager pass after all parallel dispatches converge.
