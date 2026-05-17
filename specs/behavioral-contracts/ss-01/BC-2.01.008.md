---
document_type: behavioral-contract
level: L3
version: "1.0.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T18:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "2184d8f"
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

# Behavioral Contract BC-2.01.008: Auth Token Wire Format (FC-06)

## Description

The monocle daemon generates a 32-byte cryptographically random auth token on startup using
`rand::rngs::OsRng` and stores it as a 64-character lowercase hex string in the lock file's
`authToken` field. The wire format for presenting this token to authenticated endpoints is
`monocle-v1:<64-char-hex>`, providing a versioned prefix that reserves future auth model
evolution for Phase 4 federation. Constant-time comparison via `constant_time_eq` prevents
timing oracle attacks.

## Preconditions

1. The monocle daemon has completed its start sequence (steps 1–6 of §Start Sequence).
2. The lock file at `<runtime_dir>/monocle.lock` has been written successfully via `tempfile::persist`.

## Postconditions

1. The lock file `authToken` field contains exactly a 64-character lowercase hexadecimal string (32 bytes from `rand::rngs::OsRng`, hex-encoded). No prefix, no suffix. Regex: `/^[0-9a-f]{64}$/`.
2. The wire format for the auth token presented to the daemon (in the `X-Monocle-Authorization` header) is `monocle-v1:<64-char-hex>` — the literal prefix `monocle-v1:` followed by the lock file's 64-char hex value. Total wire length: 74 characters.
3. The daemon's auth middleware uses `constant_time_eq::constant_time_eq` to compare the hex part (after prefix strip) with the stored secret. The comparison is constant-time to prevent timing oracle attacks.
4. Tokens accepted by Phase 1 daemon's `/status`, `/hooks/*`, and `/shutdown` routes use ONLY `X-Monocle-Authorization: monocle-v1:<64-hex>`. No other header format is a valid auth mechanism on Phase 1 endpoints.

## Invariants

1. The prefix `monocle-v1:` versions the auth model. Phase 4 federation uses `Authorization: Bearer` on a SEPARATE russh/IPC channel — NOT on Phase 1 HTTP endpoints.
2. The `expected_secret` stored in memory and in the lock file is the bare 64-char hex (no prefix). The prefix is a wire-format concern only.
3. `rand::rngs::OsRng` is the entropy source — not `thread_rng`. This is mandatory for production-grade secret generation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-004 | Token rotation on daemon restart | New 32-byte secret generated; any in-flight requests with old token receive HTTP 401 after restart; hook scripts that read token from lock file at request time always have the current token |
| EC-005 | Lock file write fails (filesystem full) | Daemon exits before accepting any requests; no partial lock file with wrong or empty token left on disk (tempfile guarantees) |
| EC-006 | Lock file `contract_version` field | Is `1` (first key); any lock-file reader MUST check `contract_version == 1` before consuming the `authToken` field (per BC-2.01.010) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Read `<runtime_dir>/monocle.lock` after `monocle daemon start` | `authToken` field matches `/^[0-9a-f]{64}$/` | happy-path |
| `GET /status` with `X-Monocle-Authorization: monocle-v1:<authToken-from-lock>` | HTTP 200 | happy-path |
| `GET /status` with `X-Monocle-Authorization: monocle-v1:<wrong-hex>` | HTTP 401 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-008 | Lock file `authToken` matches `/^[0-9a-f]{64}$/` after daemon start | integration |
| VP-008 | Presenting `monocle-v1:<authToken>` to `/status` returns HTTP 200 | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the auth token wire format that secures authenticated access to the hook ingestion daemon's endpoints |
| L2 Domain Invariants | DI-003 (the auth token must be written to the lock file after the port is bound — never before — Postcondition 1 states the lock file authToken is written as part of the start sequence after the listener is bound, per BC-2.01.005 Postcondition 3 which this BC specifies the content for); DI-005 (a monocle daemon must not accept an auth token that does not begin with the canonical prefix for its version — this BC defines the wire format monocle-v1:<64-hex> that is the canonical prefix, which DI-005 requires to be enforced on all auth checks) |
| Architecture Module | monocle-runtime (daemon binary, auth) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.25 §Daemon Lifecycle Protocol §Start Sequence |
| Forward Compat Contract | FC-06 (versioned auth token prefix) |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — versioned auth token prefix) |
| Test File | `monocle-runtime/tests/auth_token_lifecycle.rs` |
| Test Name | `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-AUTH-001 |

## Related BCs (Recommended)

- [BC-2.01.009] — composes with: BC-2.01.009 governs how the auth header is validated (missing vs. invalid); this BC governs the token format that is being validated
- [BC-2.01.010] — composes with: lock file schema (contract_version first key, then authToken) specified in BC-2.01.010
- [BC-2.01.005] — depends on: lock file is created by BC-2.01.005 start sequence; auth token is placed in the lock file during that sequence

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#daemon-lifecycle-protocol` — auth token generation in start sequence
- `architecture/SS-forward-compatibility.md` — FC-06 contract (versioned auth token prefix)

## Story Anchor (Recommended)

S-TBD — Implement auth token generation and lock file writing with OsRng (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-008-auth-token-wire-format.md` — VP-008 auth token wire format integration tests

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-003 ... ; DI-005 ...`
  - DI-003 mapping: This BC specifies the content written to the lock file authToken field as part of the start sequence that BC-2.01.005 governs; the DI-003 ordering (after port bound) is enforced by BC-2.01.005 Postcondition 3. DI-005 mapping: This BC defines the monocle-v1: canonical prefix — the exact prefix DI-005 requires the daemon to enforce.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).
