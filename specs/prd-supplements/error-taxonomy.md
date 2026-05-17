---
document_type: prd-supplement-error-taxonomy
level: L3
version: "1.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T23:00:00Z
phase: 1a
inputs: [prd.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]
input-hash: "6787573"
traces_to: prd.md
---

# Error Taxonomy: Monocle Phase 1

> PRD supplement — extracted from PRD Section 5 (v1.26 restructure, previously inline in PRD v1.25 §5).
> Primary consumers: implementer, test-writer.
> Do NOT reuse or renumber error codes — append-only policy applies.

## Naming Convention

Error codes follow the convention `E-<SUBSYSTEM>-<NNN>` where subsystem abbreviations are:

| Abbreviation | Subsystem | Modules |
|-------------|-----------|---------|
| `DAEMON` | Daemon lifecycle | `monocle-runtime` (start, shutdown, body-limit) |
| `AUTH` | Authentication | `monocle-runtime/src/auth.rs` |
| `LOCK` | Lock file | `monocle-runtime` (lock file lifecycle) |
| `RING` | Ring buffer | `monocle-runtime/src/ring.rs` |
| `FACT` | Factory adapter | `monocle-core/src/factory.rs` |
| `ENG` | Engine module | `monocle-core/src/engine.rs`, `monocle-runtime` |
| `PROTO` | Protocol (proto wire) | `monocle-proto` |

## Error Catalog

| Code | Category | Severity | Exit / HTTP | Message Format | Source BC |
|------|----------|----------|-------------|---------------|-----------|
| E-AUTH-001 | Authentication | Broken | HTTP 401 | `{"error":"missing_auth_token"}` | BC-2.01.009 (both `X-Monocle-Authorization` AND `X-Claude-Code-Ide-Authorization` absent; dual-absence per ADR-0005; old: BC-AUTH-002) |
| E-AUTH-002 | Authentication | Broken | HTTP 401 | `{"error":"invalid_auth_token"}` | BC-2.01.009 (any value-present failure on canonical or alias path; wrong format, wrong secret, or empty value; old: BC-AUTH-002) |
| E-AUTH-003 | Authentication | Cosmetic | WARN log | `WARN: X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization` | BC-2.01.009 INV-6 (alias path entered — emitted on every alias-path request regardless of auth success or failure; per ADR-0005 dual-accept deprecation signaling) |
| E-DAEMON-001 | Body Size | Broken | HTTP 413 | `{"error":"payload_too_large","limit_bytes":262144}` | BC-2.01.003 (old: BC-DAEMON-003) |
| E-DAEMON-002 | Shutdown | Degraded | HTTP 503 | `{"error":"daemon_shutting_down"}` with `Retry-After: 10` header | BC-2.01.004 §Shutdown Signal Handling (old: BC-DAEMON-004) |
| E-DAEMON-003 | Liveness | Broken | HTTP 503 | `{"status":"shutting_down"}` | BC-2.01.001 (healthz during shutdown; old: BC-DAEMON-001) |
| E-DAEMON-004 | Daemon Start | Broken | Exit 1 | `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path` | BC-2.01.005 precondition 2(d) — `DaemonStartError::RuntimeDirUnresolvable` (old: BC-DAEMON-005) |
| E-LOCK-001 | Lock File | Broken | Exit 1 | `ERROR: daemon already running at pid=<N>; exiting` | BC-2.01.005 §Start Sequence step 2b (old: BC-DAEMON-005) |
| E-LOCK-002 | Lock File | Degraded | WARN log | `WARN: stale lock file removed` | BC-2.01.005 §Start Sequence step 2c (old: BC-DAEMON-005) |
| E-LOCK-003 | Lock File | Degraded | WARN log | `WARN: lock file contract_version <N> not recognized; skipping` | BC-2.01.010 EC-010 (old: BC-LOCK-001) |
| E-ENG-001 | Engine Init | Broken | Daemon exit | `ERROR: platform home directory unresolvable (BaseDirs::new() returned None)` | BC-2.03.003 (old: BC-ENGINE-002-ERR) |
| E-FACT-001 | Factory Parse | Degraded | WARN log | `WARN: STATE.md not found at <path>: <io-error>` | BC-2.02.005 (old: BC-FACTORY-002) |
| E-FACT-002 | Factory Parse | Degraded | Returns `None` or `Err` | `WARN: STATE.md malformed: <reason>` | BC-2.02.005 (old: BC-FACTORY-002) |
| E-RING-001 | Ring Buffer | Degraded | Logged | `WARN: ring buffer flush failed: <io-error>` | BC-2.01.007 EC-003 (old: BC-RING-001) |
| E-PROTO-001 | Protocol | Degraded | WARN log | `WARN: HookEnvelope schema_version <N> not recognized; skipping` | BC-2.02.008 EC-027, EC-028 (old: BC-PROTO-002) |

## Severity Definitions

| Severity | Meaning | Exit Code Impact | User-Visible |
|----------|---------|-----------------|--------------|
| Broken | Cannot continue; operation fails completely | Non-zero exit OR HTTP 4xx/5xx with error body | Yes — error logged + returned |
| Degraded | Partial result possible; operation degrades gracefully | Zero exit with WARN log OR HTTP 503 with Retry-After | Yes — WARN logged |
| Cosmetic | Formatting/display issue only; no functional impact | Zero exit | Optional |

## Error-to-Module Mapping

| Error Code | Implementation Site | Test File |
|-----------|---------------------|-----------|
| E-AUTH-001 | `monocle-runtime/src/auth.rs` (both-headers-absent branch; canonical checked first, then alias) | `monocle-runtime/tests/auth_header_rejection.rs` |
| E-AUTH-002 | `monocle-runtime/src/auth.rs` (invalid format/value branch — canonical path OR alias path) | `monocle-runtime/tests/auth_header_rejection.rs` |
| E-AUTH-003 | `monocle-runtime/src/auth.rs` (alias-path WARN log; emitted before constant-time comparison result is known) | `monocle-runtime/tests/auth_header_rejection.rs` |
| E-DAEMON-001 | `monocle-runtime/src/router.rs` (`DefaultBodyLimit::max(256 * 1024)` rejection) | `monocle-runtime/tests/body_size_limit.rs` |
| E-DAEMON-002 | `monocle-runtime/src/lifecycle.rs` (shutdown drain — hook handler returns 503) | `monocle-runtime/tests/graceful_shutdown.rs` |
| E-DAEMON-003 | `monocle-runtime/src/handlers/healthz.rs` (ShuttingDown arm → 503) | `monocle-runtime/tests/healthz_endpoint.rs` |
| E-DAEMON-004 | `monocle-runtime/src/lifecycle.rs` (`DaemonStartError::RuntimeDirUnresolvable`) | `monocle-runtime/tests/lock_file_lifecycle.rs` |
| E-LOCK-001 | `monocle-runtime/src/lock.rs` (live pid check → exit 1) | `monocle-runtime/tests/lock_file_lifecycle.rs` |
| E-LOCK-002 | `monocle-runtime/src/lock.rs` (stale lock cleanup WARN) | `monocle-runtime/tests/lock_file_lifecycle.rs` |
| E-LOCK-003 | `monocle-runtime/src/lock.rs` (unknown contract_version WARN) | `monocle-runtime/tests/lock_file_contract.rs` |
| E-ENG-001 | `monocle-runtime/src/engine/claude.rs` (`BaseDirs::new()` None arm) | `monocle-runtime/tests/engine_module_home_unresolvable.rs` |
| E-FACT-001 | `monocle-core/src/factory.rs` (STATE.md not found) | `monocle-core/tests/factory_self_referential.rs` |
| E-FACT-002 | `monocle-core/src/factory.rs` (STATE.md parse error) | `monocle-core/tests/factory_self_referential.rs` |
| E-RING-001 | `monocle-runtime/src/ring.rs` (flush failure WARN) | `monocle-runtime/tests/jsonl_ring.rs` |
| E-PROTO-001 | `monocle-runtime/src/hooks/envelope.rs` (unknown schema_version WARN) | Phase 4 integration test (future) |

## Message Format Conventions

- `<N>` — integer placeholder (PID, version number, count)
- `<path>` — absolute filesystem path
- `<io-error>` — OS error string from `std::io::Error::to_string()`
- `<reason>` — human-readable parse failure description
- All error JSON bodies use snake_case keys
- HTTP error bodies are valid JSON; always parseable as `{"error": "<code>"}` minimum
- WARN log entries use `tracing::warn!` structured format; not returned to callers

---

## §Trace

### F-R107-1 PO closure — 2026-05-17T23:00:00Z

**Finding:** F-R107-1 CRITICAL — fabricated ADR-0005 path in frontmatter `inputs:`.

**SE-17f before/after evidence:**

**Before:** `inputs: [prd.md, architecture/adr/ADR-0005-dual-accept-auth-header.md]`
**After:** `inputs: [prd.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]`

Canonical filename verified via ARCH-INDEX and disk. Version bumped: 1.1 → 1.2; timestamp refreshed.

**Scope:** Frontmatter `inputs:` only. No body content changed.

---

### F-R106-16 PO closure — 2026-05-17T22:15:00Z

**Finding:** F-R106-16 MED — error-taxonomy.md not updated for ADR-0005 dual-accept auth. E-AUTH-001 described "absent header" without dual-header semantics (should be: both `X-Monocle-Authorization` AND `X-Claude-Code-Ide-Authorization` absent). E-AUTH-003 for WARN deprecation log was missing from the catalog entirely.

**Canonical sources:** BC-2.01.009 INV-6 (WARN on alias path), INV-7 (constant-time on both paths), EC-010 (alias wrong secret), EC-011 (both present → canonical wins), EC-012 (alias empty value); ADR-0005 §Deprecation Signaling.

**Decision — E-AUTH-003 catalog entry vs. §Severity Definitions Cosmetic row:**

Production-grade default applied: E-AUTH-003 is a defined, documented behavior per BC-2.01.009 INV-6 and ADR-0005. It deserves an explicit catalog entry. The Cosmetic-row approach would leave the behavior undiscoverable via error-code lookup. E-AUTH-003 is classified as **Cosmetic** severity (INV-6 is a deprecation signal; it does not affect the auth outcome and has zero exit-code impact). Adding it as E-AUTH-003 satisfies the append-only numbering policy and makes the behavior machine-referenceable by implementer and test-writer.

**SE-17c — Before (E-AUTH-001 Source BC + Error-to-Module Mapping — single-header semantics):**

```
Error Catalog:
| E-AUTH-001 | Authentication | Broken | HTTP 401 | {"error":"missing_auth_token"} |
  BC-2.01.009 (absent header; old: BC-AUTH-002) |
| E-AUTH-002 | Authentication | Broken | HTTP 401 | {"error":"invalid_auth_token"} |
  BC-2.01.009 (any value-present failure; old: BC-AUTH-002) |
[E-AUTH-003: absent]

Error-to-Module Mapping:
| E-AUTH-001 | monocle-runtime/src/auth.rs (missing header branch) | ... |
| E-AUTH-002 | monocle-runtime/src/auth.rs (invalid format/value branch) | ... |
[E-AUTH-003: absent]
```

**SE-17d — After (E-AUTH-001 dual-absence semantics + E-AUTH-003 Cosmetic WARN entry):**

```
Error Catalog:
| E-AUTH-001 | Authentication | Broken | HTTP 401 | {"error":"missing_auth_token"} |
  BC-2.01.009 (both X-Monocle-Authorization AND X-Claude-Code-Ide-Authorization absent;
  dual-absence per ADR-0005; old: BC-AUTH-002) |
| E-AUTH-002 | Authentication | Broken | HTTP 401 | {"error":"invalid_auth_token"} |
  BC-2.01.009 (any value-present failure on canonical or alias path; wrong format,
  wrong secret, or empty value; old: BC-AUTH-002) |
| E-AUTH-003 | Authentication | Cosmetic | WARN log |
  "WARN: X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization" |
  BC-2.01.009 INV-6 (alias path entered — emitted on every alias-path request regardless
  of auth success or failure; per ADR-0005 dual-accept deprecation signaling) |

Error-to-Module Mapping:
| E-AUTH-001 | monocle-runtime/src/auth.rs (both-headers-absent branch; canonical checked first, then alias) | ... |
| E-AUTH-002 | monocle-runtime/src/auth.rs (invalid format/value branch — canonical path OR alias path) | ... |
| E-AUTH-003 | monocle-runtime/src/auth.rs (alias-path WARN log; emitted before constant-time comparison result is known) | ... |
```

**Changes made:**
- E-AUTH-001 Source BC: "absent header" → "both `X-Monocle-Authorization` AND `X-Claude-Code-Ide-Authorization` absent; dual-absence per ADR-0005"
- E-AUTH-002 Source BC: added "on canonical or alias path" to clarify value-present failures apply to both paths
- E-AUTH-003 added to Error Catalog (Cosmetic severity, WARN log, BC-2.01.009 INV-6, ADR-0005)
- E-AUTH-003 added to Error-to-Module Mapping (auth.rs alias-path WARN branch)
- Error count: 14 → 15 (E-AUTH-003 addition)
- Version bumped: v1.0 → v1.1; timestamp refreshed; ADR-0005 added to inputs

**PRD §5 prose impact:** PRD §5 states "14 error codes across 7 subsystem abbreviations". With E-AUTH-003, the count is 15. PRD v1.26.3 → v1.26.4 bump in same burst updates this count.

**Scope:** PO-only. No changes to BC-2.01.009, ADR-0005, or any other artifact.
