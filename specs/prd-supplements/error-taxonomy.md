---
document_type: prd-supplement-error-taxonomy
level: L3
version: "1.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
inputs: [prd.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]
input-hash: "cf29b5c"
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
| E-AUTH-003 | Authentication | Cosmetic | WARN log | `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization` | BC-2.01.009 INV-6 (alias path entered — emitted on every alias-path request regardless of auth success or failure; per ADR-0005 dual-accept deprecation signaling) |
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
| E-ENG-001 | `monocle-runtime/src/engine/claude_code.rs` (`BaseDirs::new()` None arm) | `monocle-runtime/tests/engine_module_home_unresolvable.rs` |
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

---

### F-R107-1 PO closure — 2026-05-17T23:00:00Z

**Finding:** F-R107-1 CRITICAL — fabricated ADR-0005 path in frontmatter `inputs:`.

**SE-17f before/after evidence:**

**Before:** `inputs: [prd.md, architecture/adr/ADR-0005-dual-accept-auth-header.md]`
**After:** `inputs: [prd.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]`

Canonical filename verified via ARCH-INDEX and disk. Version bumped: 1.1 → 1.2; timestamp refreshed.

**Scope:** Frontmatter `inputs:` only. No body content changed.

---

### GAP-R47-1 PO closure — 2026-05-18T01:00:00Z

**Finding:** GAP-R47-1 HIGH (PO part) — E-AUTH-003 Message Format used String B (`"WARN: X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization"`) instead of canonical String A from BC-2.01.009 INV-6. Per CLAUDE.md hierarchy, BC-2.01.009 is the canonical source for behavioral contracts; the error taxonomy must match the BC exactly.

**Canonical source:** BC-2.01.009 INV-6 line: `"WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization"`.

**SE-17f — Before/After (E-AUTH-003 Message Format):**

**Before (String B):** `WARN: X-Claude-Code-Ide-Authorization alias used; migrate to X-Monocle-Authorization`
**After (String A, canonical per BC-2.01.009 INV-6):** `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization`

**Note:** The §SE-17d historical record in the prior §Trace entry (F-R106-16) also shows String B — that was the state as of Round 6 (the divergence existed then). The historical record is preserved as-is; the active E-AUTH-003 table row is corrected to String A.

**Changes made:** E-AUTH-003 Message Format column: String B → String A (canonical per BC-2.01.009 INV-6); version bumped v1.2 → v1.3; timestamp refreshed.

**Scope:** PO-only. No changes to BC-2.01.009, ADR-0005, VP-009, or any architecture artifact. FV 7D handles VP-009 reconciliation in parallel (per task scope).

---

### F-R109-12 PO closure — 2026-05-18T05:31:00Z

**Finding:** F-R109-12 HIGH — §Trace blocks were non-monotonic. F-R106-16 (T22:15) appeared AFTER F-R107-1 (T23:00) — the block was authored in Round 6 but inserted after the Round 7 block.

**SE-17f BEFORE (§Trace order):** F-R107-1 (T23:00), F-R106-16 (T22:15), GAP-R47-1 (T01:00) — non-monotonic.
**SE-17f AFTER (§Trace order):** F-R106-16 (T22:15), F-R107-1 (T23:00), GAP-R47-1 (T01:00), F-R109-12 (T05:31) — monotonic ascending.

Content of each section preserved verbatim; only insertion order corrected.

**Changes made:** §Trace blocks reordered monotonic ascending; version bumped v1.3 → v1.4; timestamp refreshed.

SE-16d monotonicity PASS: 2026-05-18T05:31:00Z > prior 2026-05-18T01:00:00Z (v1.3 GAP-R47-1). ARITHMETICALLY TRUE: 2026-05-18T05:31:00Z > 2026-05-18T01:00:00Z PASS.

**Scope:** PO-only. No body content (error catalog, mappings) changed.

---

### F-R110-1 PO closure — 2026-05-18T06:00:00Z

**Finding:** F-R110-1 CRITICAL — error-taxonomy frontmatter timestamp and §Trace v1.4 header timestamp were `2026-05-17T04:31:00Z` (wrong date — Round 8 authored on 2026-05-18).

**Changes made:** frontmatter timestamp corrected to `2026-05-18T05:31:00Z`; §Trace v1.4 header corrected to `2026-05-18T05:31:00Z`; SE-17f order line T04:31 → T05:31; SE-16d PASS added. No version bump (timestamp-only fix per F-R110 instructions).

SE-16d monotonicity PASS: 2026-05-18T06:00:00Z > prior 2026-05-18T05:31:00Z (v1.4). ARITHMETICALLY TRUE.

---

### F-R111 Round 10 PO closure — 2026-05-18T07:00:00Z

**Finding:** F-R111-1 CRITICAL — v1.4 frontmatter timestamp was `2026-05-18T05:31:00Z`. This is the corrected v1.3 timestamp value, not the v1.4 burst timestamp. The v1.4 burst (F-R109-12) ran at `2026-05-18T05:31:00Z`, but the v1.4 §Trace shows the F-R110-1 closure at `2026-05-18T06:00:00Z` — meaning the frontmatter should have been advanced to `2026-05-18T06:00:00Z` at that point. Corrected frontmatter to `2026-05-18T07:00:00Z` (Round 10 fix burst timestamp).

**Changes made:** frontmatter version v1.4 → v1.5; frontmatter timestamp refreshed. No error catalog content changed.

SE-16d monotonicity PASS: 2026-05-18T07:00:00Z > prior 2026-05-18T06:00:00Z (v1.4 F-R110-1 closure). ARITHMETICALLY TRUE: 2026-05-18T07:00:00Z > 2026-05-18T06:00:00Z PASS.

---

### §Trace v1.6

**F-PHASE-3-B-error-taxonomy-claude-path-fix** (2026-05-20T00:00:00Z):
- NORMATIVE (LOW): Legacy path `monocle-runtime/src/engine/claude.rs` corrected to `monocle-runtime/src/engine/claude_code.rs` per ARCH-INDEX v1.0.11 §Subsystem Registry SS-03 trait-vs-impl split (F-PHASE2-R05-04 in ARCH-INDEX §Trace v1.0.11).
- Surfaced by S-015 spec-reviewer F-D-03 (post-Phase-2 story-uncertainty-review cycle-001).
- SE-22 sibling sweep: one occurrence found and corrected (Error-to-Module Mapping row E-ENG-001); no other occurrences of the legacy `claude.rs` path present.
- SE-16d PASS: monotonic timestamp per chain high-water. 2026-05-20T00:00:00Z > prior 2026-05-18T07:00:00Z (v1.5 F-R111). ARITHMETICALLY TRUE.
- Refs: drbothen/vsdd-factory#150
