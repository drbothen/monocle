---
document_type: prd-supplement-error-taxonomy
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T12:30:00Z
phase: 1a
inputs: [prd.md]
input-hash: "742464a"
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
| E-AUTH-001 | Authentication | Broken | HTTP 401 | `{"error":"missing_auth_token"}` | BC-2.01.009 (absent header; old: BC-AUTH-002) |
| E-AUTH-002 | Authentication | Broken | HTTP 401 | `{"error":"invalid_auth_token"}` | BC-2.01.009 (any value-present failure; old: BC-AUTH-002) |
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
| E-AUTH-001 | `monocle-runtime/src/auth.rs` (missing header branch) | `monocle-runtime/tests/auth_header_rejection.rs` |
| E-AUTH-002 | `monocle-runtime/src/auth.rs` (invalid format/value branch) | `monocle-runtime/tests/auth_header_rejection.rs` |
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
