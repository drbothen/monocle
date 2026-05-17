---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T11:30:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "03a845a"
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
endpoints: `missing_auth_token` for a completely absent `X-Monocle-Authorization` header,
and `invalid_auth_token` for any value-present failure (bad prefix, bad format, wrong secret).
All value-present failures return the same body to prevent attackers from learning whether
their token had the structurally correct prefix. The old body `invalid_auth_token_format`
is retired and does not appear in any Phase 1 response.

## Preconditions

1. The monocle daemon is running with a valid lock file.
2. A request arrives at any authenticated endpoint (`/hooks/*`, `/status`, `/shutdown`).

## Postconditions

1. **Missing header:** If the `X-Monocle-Authorization` header is absent entirely, return HTTP 401 `{"error":"missing_auth_token"}`. This is a structural precondition failure, not an authentication attempt.
2. **Any value-present failure:** If the header is present but its value fails validation for any reason — bad prefix (does not begin with `monocle-v1:`), bad format, empty suffix, secret mismatch — return HTTP 401 `{"error":"invalid_auth_token"}`. All value-present failure modes return the same body intentionally (no format/mismatch distinction in the response body).
3. `Authorization: Bearer <token>` headers on Phase 1 endpoints (Phase 4 OAuth2 attempt) receive HTTP 401 `{"error":"missing_auth_token"}` — `Authorization: Bearer` is not a recognized header name for Phase 1 endpoints; `X-Monocle-Authorization` is absent.

## Invariants

1. The two-body taxonomy (`missing_auth_token` vs. `invalid_auth_token`) is the complete auth error surface for Phase 1. There is no third body. The old body `invalid_auth_token_format` is retired and does not appear in any Phase 1 response.
2. Value-present failures (Rules 2 and 3 in the auth middleware) deliberately return the same body to prevent an attacker from determining whether their token had the structurally correct prefix, even if they cannot read the lock file directly.
3. The distinction between missing and invalid is preserved because a missing header is a client-configuration error (actionable for debugging), not an authentication attempt. The `missing_auth_token` body provides developer-friendly diagnostics at zero security cost.
4. The auth middleware implementation uses `AuthError::Missing` for absent headers and `AuthError::Invalid` for all value-present failures.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-007 | Empty `X-Monocle-Authorization` value (header present but value is empty string) | Empty string does not begin with `monocle-v1:` — returns HTTP 401 `{"error":"invalid_auth_token"}` (value-present, format-fail case) |
| EC-008 | `X-Monocle-Authorization` header absent entirely | Returns HTTP 401 `{"error":"missing_auth_token"}` |
| EC-009 | `X-Monocle-Authorization: monocle-v1:` (prefix present but no hex suffix) | Passes the prefix check but fails the constant-time secret comparison (empty hex string never matches the 64-char secret); returns HTTP 401 `{"error":"invalid_auth_token"}` — the empty suffix is a value-present failure |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| No `X-Monocle-Authorization` header | HTTP 401 `{"error":"missing_auth_token"}` | error |
| `X-Monocle-Authorization: deadbeef...64chars` (no prefix) | HTTP 401 `{"error":"invalid_auth_token"}` | error |
| `X-Monocle-Authorization: monocle-v2:deadbeef...64chars` (wrong version prefix) | HTTP 401 `{"error":"invalid_auth_token"}` | error |
| `X-Monocle-Authorization: monocle-v1:` (prefix only, no hex) | HTTP 401 `{"error":"invalid_auth_token"}` | error |
| `Authorization: Bearer fake-token` (no `X-Monocle-Authorization`) | HTTP 401 `{"error":"missing_auth_token"}` | error |
| `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` (correct format, wrong value) | HTTP 401 `{"error":"invalid_auth_token"}` | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-009 | All 6 test-vector failure modes return the correct HTTP status and body | integration |
| VP-009 | No third error body exists in Phase 1 auth middleware responses | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the auth header validation logic protecting all authenticated hook ingestion daemon endpoints |
| L2 Domain Invariants | N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source |
| Architecture Module | monocle-runtime (daemon binary, auth) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.25 §Daemon Lifecycle Protocol §Start Sequence |
| Forward Compat Contract | FC-06 (F-FC-I005 Phase 4 OAuth2 clarification) |
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
- `architecture/SS-forward-compatibility.md` — FC-06 contract (Phase 4 OAuth2 clarification)

## Story Anchor (Recommended)

S-TBD — Implement auth middleware with two-body error taxonomy (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-009-auth-header-validation.md` — VP-009 auth header validation integration tests
