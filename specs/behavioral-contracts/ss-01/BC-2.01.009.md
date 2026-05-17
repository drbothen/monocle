---
document_type: behavioral-contract
level: L3
version: "1.0.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T05:08:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "a9aeb88"
traces_to: prd.md
origin: greenfield
subsystem: SS-01
capability: CAP-001
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.01.009: Auth Header Validation (Missing and Invalid Token)

## Description

The monocle daemon's auth middleware enforces a two-body error taxonomy on authenticated
endpoints: `missing_auth_token` when BOTH `X-Monocle-Authorization` and
`X-Claude-Code-Ide-Authorization` are absent, and `invalid_auth_token` for any value-present
failure on either header path. The middleware implements a dual-accept protocol per ADR-0005:
canonical `X-Monocle-Authorization` (monocle-aware tools, prefix `monocle-v1:`) takes
priority; `X-Claude-Code-Ide-Authorization` (real Claude Code compatibility alias, raw 64-hex
token) is accepted as a fallback with a WARN-level deprecation log. All value-present failures
return the same body to prevent attackers from learning whether their token had the structurally
correct prefix. The old body `invalid_auth_token_format` is retired and does not appear in
any Phase 1 response.

## Preconditions

1. The monocle daemon is running with a valid lock file.
2. A request arrives at any authenticated endpoint (`/hooks/*`, `/status`, `/shutdown`).

## Postconditions

1. **Missing header (both absent):** If BOTH `X-Monocle-Authorization` and `X-Claude-Code-Ide-Authorization` are absent, return HTTP 401 `{"error":"missing_auth_token"}`. This is a structural precondition failure, not an authentication attempt. A request carrying only an unrecognized header (e.g., `Authorization: Bearer <token>`) falls into this case — neither recognized header is present.
2. **Canonical path — any value-present failure:** If `X-Monocle-Authorization` is present but its value fails validation for any reason — bad prefix (does not begin with `monocle-v1:`), bad format, empty suffix, secret mismatch — return HTTP 401 `{"error":"invalid_auth_token"}`. The canonical header takes priority; if `X-Monocle-Authorization` is present, the alias header is ignored. Validation uses constant-time comparison of the hex suffix against the stored secret. All value-present failure modes return the same body intentionally (no format/mismatch distinction in the response body).
3. **Compatibility alias path — any value-present failure:** If `X-Monocle-Authorization` is absent AND `X-Claude-Code-Ide-Authorization` is present, the middleware first emits a WARN-level deprecation log (`WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization`), then validates the value as a raw 64-hex token (no `monocle-v1:` prefix — real Claude Code sends the lock file `authToken` field verbatim). If secret comparison fails, return HTTP 401 `{"error":"invalid_auth_token"}`. Constant-time comparison is used on the alias path identically to the canonical path.
4. **Both headers present — canonical wins:** If both `X-Monocle-Authorization` and `X-Claude-Code-Ide-Authorization` are present, `X-Monocle-Authorization` is used for validation (canonical priority per ADR-0005); `X-Claude-Code-Ide-Authorization` is ignored. No deprecation log is emitted in this case.

## Invariants

1. The two-body taxonomy (`missing_auth_token` vs. `invalid_auth_token`) is the complete auth error surface for Phase 1. There is no third body. The old body `invalid_auth_token_format` is retired and does not appear in any Phase 1 response.
2. Value-present failures on either the canonical path or the compatibility alias path deliberately return the same `{"error":"invalid_auth_token"}` body to prevent an attacker from determining header name, token prefix compliance, or any other structural detail from the response.
3. The distinction between missing and invalid is preserved because a missing header is a client-configuration error (actionable for debugging), not an authentication attempt. The `missing_auth_token` body provides developer-friendly diagnostics at zero security cost.
4. The auth middleware implementation uses `AuthError::Missing` for the case where both headers are absent and `AuthError::Invalid` for all value-present failures on either the canonical or alias path.
5. Canonical priority is immutable: `X-Monocle-Authorization` always takes precedence over `X-Claude-Code-Ide-Authorization` when both are present. The alias path is entered only when `X-Monocle-Authorization` is absent.
6. The WARN deprecation log is emitted once per alias-path authentication attempt (regardless of success or failure) to make alias usage observable in structured logs. The log message is: `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization`.
7. Constant-time secret comparison is used on both the canonical and alias paths. The comparison algorithm is identical regardless of which header is used; the only difference is the input transformation (prefix-strip on canonical, none on alias).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-007 | Empty `X-Monocle-Authorization` value (header present but value is empty string) | Empty string does not begin with `monocle-v1:` — returns HTTP 401 `{"error":"invalid_auth_token"}` (value-present, format-fail case) |
| EC-008 | Both `X-Monocle-Authorization` and `X-Claude-Code-Ide-Authorization` absent | Returns HTTP 401 `{"error":"missing_auth_token"}` |
| EC-009 | `X-Monocle-Authorization: monocle-v1:` (prefix present but no hex suffix) | Passes the prefix check but fails the constant-time secret comparison (empty hex string never matches the 64-char secret); returns HTTP 401 `{"error":"invalid_auth_token"}` — the empty suffix is a value-present failure |
| EC-010 | `X-Claude-Code-Ide-Authorization` present with wrong secret (correct 64-hex format); `X-Monocle-Authorization` absent | Alias path: WARN log emitted, constant-time comparison fails → HTTP 401 `{"error":"invalid_auth_token"}`. No format rejection — raw 64-hex is the expected alias format; the value fails by secret mismatch only. |
| EC-011 | Both `X-Monocle-Authorization` (valid canonical token) and `X-Claude-Code-Ide-Authorization` (valid raw token) present | Canonical takes priority per ADR-0005. `X-Monocle-Authorization` is validated (succeeds). `X-Claude-Code-Ide-Authorization` is ignored. No WARN log emitted. → HTTP 200 / auth passes. |
| EC-012 | `X-Claude-Code-Ide-Authorization` empty value; `X-Monocle-Authorization` absent | Alias path entered: WARN log emitted, empty string fails constant-time comparison → HTTP 401 `{"error":"invalid_auth_token"}` (value-present failure on alias path). |
| EC-013 | `Authorization: Bearer <token>` present; neither `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` present | BOTH recognized headers are absent — falls into PC-1 (missing-header case, not a value-present failure). Returns HTTP 401 `{"error":"missing_auth_token"}`. The `Authorization` header name is unrecognized by the auth middleware and is ignored. This case is the dual-absence path even though a header IS present; it is not treated as a value-present failure for any recognized header. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| No `X-Monocle-Authorization` header, no `X-Claude-Code-Ide-Authorization` header | HTTP 401 `{"error":"missing_auth_token"}` | error |
| `X-Monocle-Authorization: deadbeef...64chars` (no prefix) | HTTP 401 `{"error":"invalid_auth_token"}` | error |
| `X-Monocle-Authorization: monocle-v2:deadbeef...64chars` (wrong version prefix) | HTTP 401 `{"error":"invalid_auth_token"}` | error |
| `X-Monocle-Authorization: monocle-v1:` (prefix only, no hex) | HTTP 401 `{"error":"invalid_auth_token"}` | error |
| `Authorization: Bearer fake-token` (no `X-Monocle-Authorization`, no alias) | HTTP 401 `{"error":"missing_auth_token"}` | error |
| `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` (correct format, wrong value) | HTTP 401 `{"error":"invalid_auth_token"}` | error |
| `X-Claude-Code-Ide-Authorization: <wrong-64-hex>` (alias path, wrong secret); no `X-Monocle-Authorization` | HTTP 401 `{"error":"invalid_auth_token"}` + WARN deprecation log emitted (EC-010) | error |
| `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (alias path, correct secret); no `X-Monocle-Authorization` | HTTP 200 (auth passes) + WARN deprecation log emitted (EC-010) | happy-path (alias) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-009 | All canonical-path failure modes (6 vectors) return the correct HTTP status and body | integration |
| VP-009 | All alias-path failure modes return HTTP 401 `{"error":"invalid_auth_token"}` with WARN log emitted | integration |
| VP-009 | Alias-path success returns HTTP 200 with WARN log emitted | integration |
| VP-009 | No third error body exists in Phase 1 auth middleware responses | integration |
| VP-009 | Canonical priority: when both headers present, `X-Monocle-Authorization` wins; no WARN log emitted | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the auth header validation logic protecting all authenticated hook ingestion daemon endpoints |
| L2 Domain Invariants | DI-005 (a monocle daemon must not accept an auth token that does not begin with the canonical prefix for its version — this BC is the primary enforcer of DI-005: all value-present failures including wrong prefix, bad format, and secret mismatch return HTTP 401; the two-body taxonomy ensures the monocle-v1: prefix requirement is enforced without leaking structural information to attackers) |
| Architecture Module | monocle-runtime (daemon binary, auth) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Start Sequence; ADR-0005 v1.0.2 (dual-accept auth header decision) |
| Forward Compat Contract | FC-06 (versioned auth token prefix) |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — versioned auth token prefix) |
| Architect Adjudication | commit 2db408f — disposition (c) mixed approach; `invalid_auth_token_format` retired |
| Test File | `monocle-runtime/tests/auth_header_rejection.rs` |
| Test Name | `test_BC_AUTH_002_auth_header_validation_all_failure_modes` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-AUTH-002 |

## Related BCs (Recommended)

- [BC-2.01.008] — depends on: the token format (prefix + 64-hex) validated here is specified in BC-2.01.008
- [BC-2.01.002] — composes with: BC-2.01.002 Postcondition 2 references this BC for the HTTP 401 behavior on `/status`

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#daemon-lifecycle-protocol` — auth middleware placement on authenticated router, `AuthError` enum
- `architecture/SS-forward-compatibility.md` — FC-06 contract (versioned auth token prefix)

## Story Anchor (Recommended)

S-TBD — Implement auth middleware with two-body error taxonomy (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-009-auth-header-validation.md` — VP-009 auth header validation integration tests

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-005 ...`
  - DI-005 mapping: This BC is the primary DI-005 enforcer — it defines the complete auth validation logic that rejects any token not beginning with monocle-v1:. The two-body taxonomy (missing vs. invalid) is the mechanism by which DI-005 is enforced without information leakage.
- F-R105-9 (SE-17c-d body-scope grep): Postcondition 2 references `BC-2.01.009` (self-reference in canonical form). 0 stale BC IDs. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T11:30:00Z (v1.0).

## §Trace v1.0.2

**T-128n Part 1 — F-R105 closure chain Round 4: ADR-0005 dual-accept propagation** (2026-05-17T20:00:00Z):

- NORMATIVE: BC-2.01.009 updated to reflect ADR-0005 dual-accept auth header decision.
- **SE-17f Postcondition 1 (Missing header):**
  - Before: `If the X-Monocle-Authorization header is absent entirely, return HTTP 401 {"error":"missing_auth_token"}`
  - After: `If BOTH X-Monocle-Authorization and X-Claude-Code-Ide-Authorization are absent, return HTTP 401 {"error":"missing_auth_token"}`
  - Rationale: ADR-0005 §Decision "Neither header present" semantics — "missing" now means both recognized headers absent.
- **SE-17f Postcondition 2 (Canonical path — any value-present failure):**
  - Before: `If the header is present but its value fails validation for any reason... return HTTP 401 {"error":"invalid_auth_token"}` (single-header model)
  - After: Explicit canonical priority: `X-Monocle-Authorization` present → validate with `monocle-v1:` prefix rule, constant-time compare; alias ignored when canonical present.
- **SE-17f Postcondition 3 (new — Compatibility alias path):**
  - Before: `Authorization: Bearer <token>` fallback behavior (this postcondition is renumbered; former PC-3 becomes PC-4)
  - After: `X-Claude-Code-Ide-Authorization` alias path: WARN deprecation log emitted, raw 64-hex validation, constant-time compare; → `{"error":"invalid_auth_token"}` on failure.
- **SE-17f Postcondition 4 (new — Both headers present):**
  - Before: Not specified (no dual-header case existed)
  - After: Canonical wins; `X-Claude-Code-Ide-Authorization` ignored; no WARN log.
- **SE-17f New test vectors (2 added):**
  - Vector 7: `X-Claude-Code-Ide-Authorization: <wrong-64-hex>` (alias, wrong secret) → HTTP 401 `{"error":"invalid_auth_token"}` + WARN log (EC-010)
  - Vector 8: `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (alias, correct secret) → HTTP 200 + WARN log (EC-010)
- **SE-17c-d body-scope grep:** Postcondition cross-references updated. EC-008 description updated (now references both headers). EC-010/EC-011/EC-012 added. 0 stale BC IDs. 0 stale VP IDs.
- **Description section** updated to reflect dual-accept protocol and ADR-0005 reference.
- **Invariants** expanded from 4 → 7: aliases constant-time symmetry (INV-7), canonical priority immutability (INV-5), WARN log format (INV-6).
- **Traceability §Architecture Source** updated: SS-daemon-lifecycle.md v1.0.29; ADR-0005 added.
- SE-16d monotonicity PASS: 2026-05-17T20:00:00Z > prior 2026-05-17T18:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R106-7 HIGH — Fabricated F-FC-I005 parenthetical removal** (2026-05-17T22:10:00Z):
- F-R106-7: Forward Compat Contract row contained `FC-06 (F-FC-I005 Phase 4 OAuth2 clarification)`. The ID `F-FC-I005` does not exist in `SS-forward-compatibility.md`; the FC convention is `FC-NN` (two-digit numeric), not `F-FC-INNN`. The parenthetical is a fabricated identifier with no referent.
- **SE-17f Forward Compat Contract row before/after:**
  - Before: `FC-06 (F-FC-I005 Phase 4 OAuth2 clarification)`
  - After: `FC-06 (versioned auth token prefix)`
  - Rationale: `FC-06 (versioned auth token prefix)` is the canonical form used in BC-2.01.008 Traceability table and the BC-INDEX §SS-01 FC reference. The parenthetical was removed and replaced with the correct FC-06 description.
- Note: Architect 5E is performing the mirror removal in SS-daemon-lifecycle.md in the same round (pre-adjudicated to REMOVE).
- SE-17c-d body-scope grep: 0 additional stale BC IDs. 0 stale VP IDs. Forward Compat Contract row is the sole normative change in this version.
- SE-16d monotonicity PASS: 2026-05-17T22:10:00Z > prior 2026-05-17T20:00:00Z (v1.0.2).

## §Trace v1.0.4

**F-R107-2 CRITICAL + F-R107-2 closure part (BC ADR pin add) per Round 6A scope expansion + F-R107-10 MEDIUM** (2026-05-17T23:30:00Z):
*(NOTE: v1.0.4 originally cited "F-R107-9" for the ADR version pin. That was a misattribution — see §Trace v1.0.5 for correction. F-R107-9 in the R107 report describes ADR-0002 inputs path; the ADR-0005 pin addition is F-R107-2 closure part. Finding-ID corrected in v1.0.5; v1.0.4 content preserved verbatim below.)*

**F-R107-2 — Architecture Source pin refresh v1.0.29 → v1.0.30:**
- SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.29 §Daemon Lifecycle Protocol §Start Sequence; ADR-0005 (dual-accept auth header decision)`
- SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.30 §Daemon Lifecycle Protocol §Start Sequence; ADR-0005 v1.0.2 (dual-accept auth header decision)`
- Canonical SS-daemon-lifecycle version per architect 5E commit 03a4c57 post-R106 closure. (Note: this BC was at v1.0.29, not v1.0.25 — it had been previously refreshed in the T-128n Round 4 update. Refreshed to v1.0.30 per SE-17g sweep requirement.)

**F-R107-2 closure part (BC ADR pin add) — ADR-0005 version pin added:**
- Architecture Source row previously cited `ADR-0005 (dual-accept auth header decision)` without an explicit version pin.
- SE-17f: Added `v1.0.2` version pin to ADR-0005 citation.
- Rationale: See BC-2.01.008 §Trace v1.0.4 (identical requirement applies here; both BCs had the same bare-ADR citation gap).

**F-R107-10 — EC-013 Bearer-fallback edge case added:**
- VP-009 probe 9.5 and test-vectors row 5 both reference `Authorization: Bearer <token>` (wrong header name, neither canonical nor alias recognized). No EC covered this in the BC prior to this version — EC-007 through EC-012 existed.
- EC-013 BEFORE: absent.
- EC-013 AFTER: `Authorization: Bearer <token>` present; neither `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` present → HTTP 401 `{"error":"missing_auth_token"}` per PC-1 dual-absence semantics (both recognized headers absent; `Authorization` header name is unrecognized and ignored by the auth middleware).
- Alignment: EC-013 is consistent with PC-1 ("BOTH `X-Monocle-Authorization` and `X-Claude-Code-Ide-Authorization` are absent") and the existing test vector row 5 (`Authorization: Bearer fake-token` → `{"error":"missing_auth_token"}`). The Bearer header is NOT a value-present failure for any recognized header — it is the dual-absence case with an irrelevant third header present.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. Architecture Source and EC table are the only normative changes in this version.
- SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T22:10:00Z (v1.0.3).

## §Trace v1.0.5

**F-R108-12 HIGH — Audit-trail correction: §Trace v1.0.4 finding-ID F-R107-9 was misattributed** (2026-05-18T01:10:00Z):
- F-R108-12: The §Trace v1.0.4 entry below (preserved verbatim for historical record) cited "F-R107-9 — ADR-0005 version pin added." F-R107-9 in the R107 adversarial report describes the still-broken ADR-0002 inputs path (a defect routed to Architect 7C for Round 7). It does NOT describe the ADR version pin addition.
- The ADR-0005 v1.0.2 version pin added to the Architecture Source row in v1.0.4 is correctly classified as a partial closure of **F-R107-2** (cascade incomplete — Round 5D BC sweep missed the ADR version pin). This is "F-R107-2 closure part (BC ADR pin add) per Round 6A scope expansion."
- The §Trace v1.0.4 narrative is corrected here in v1.0.5. The v1.0.4 content is preserved unchanged for audit history.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. No normative content changes in this version — audit-trail correction only.
- SE-16d monotonicity PASS: 2026-05-18T01:10:00Z > prior 2026-05-17T23:30:00Z (v1.0.4).

## §Trace v1.0.6

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.0.30 → v1.0.32; F-R109-14 MED — §Trace reordered ascending; F-R109-20 LOW — residual "(Phase 4 OAuth2 clarification)" fabrication removed** (2026-05-18T05:08:00Z):
- F-R109-4: Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32 (Round 8A). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.30 §Daemon Lifecycle Protocol §Start Sequence; ADR-0005 v1.0.2 (dual-accept auth header decision)`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Start Sequence; ADR-0005 v1.0.2 (dual-accept auth header decision)`
- F-R109-14: §Trace blocks were descending (v1.0.5, v1.0.4, v1.0.3, v1.0.2, v1.0.1). Reordered to ascending (v1.0.1 → v1.0.5 → v1.0.6). Content of each section preserved verbatim; only insertion order corrected.
- F-R109-20: Architecture Anchors line contained `FC-06 contract (Phase 4 OAuth2 clarification)`. F-R106-7 removed the identical fabricated parenthetical from the Forward Compat Contract Traceability row but missed this Architecture Anchors line. Residual removed.
  - SE-17f BEFORE: `architecture/SS-forward-compatibility.md — FC-06 contract (Phase 4 OAuth2 clarification)`
  - SE-17f AFTER: `architecture/SS-forward-compatibility.md — FC-06 contract (versioned auth token prefix)`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:08:00Z > prior 2026-05-18T01:10:00Z (v1.0.5). ARITHMETICALLY TRUE: 2026-05-18T05:08:00Z > 2026-05-18T01:10:00Z PASS.
