---
document_type: prd-supplement-test-vectors
level: L3
version: "1.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T01:00:00Z
phase: 1a
inputs: [prd.md, behavioral-contracts/, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]
input-hash: "63628a4"
traces_to: prd.md
---

# Canonical Test Vectors: Monocle Phase 1

> PRD supplement — created during v1.26 restructure.
> Phase 1 canonical test vectors are embedded in each BC's "Canonical Test Vectors" section.
> This supplement serves as an index and aggregates the cross-subsystem integration vectors.
> Primary consumers: test-writer, holdout-evaluator.

## BC Test Vector Index

> Full per-BC test vectors are in the individual BC files. This table provides an index
> for test-writer to locate the canonical vectors for each BC without loading all 22 files.

### SS-01: Daemon Lifecycle (CAP-001)

| BC ID | BC File | Vector Count | Test File |
|-------|---------|-------------|-----------|
| BC-2.01.001 | `ss-01/BC-2.01.001.md` | 3 | `monocle-runtime/tests/healthz_endpoint.rs` |
| BC-2.01.002 | `ss-01/BC-2.01.002.md` | 3 | `monocle-runtime/tests/status_endpoint_auth.rs` |
| BC-2.01.003 | `ss-01/BC-2.01.003.md` | 3 | `monocle-runtime/tests/body_size_limit.rs` |
| BC-2.01.004 | `ss-01/BC-2.01.004.md` | 4 | `monocle-runtime/tests/graceful_shutdown.rs` |
| BC-2.01.005 | `ss-01/BC-2.01.005.md` | 4 | `monocle-runtime/tests/lock_file_lifecycle.rs` |
| BC-2.01.006 | `ss-01/BC-2.01.006.md` | 3 | `monocle-runtime/tests/crash_recovery.rs` |
| BC-2.01.007 | `ss-01/BC-2.01.007.md` | 3 | `monocle-runtime/tests/jsonl_ring.rs` |
| BC-2.01.008 | `ss-01/BC-2.01.008.md` | 3 | `monocle-runtime/tests/auth_token_lifecycle.rs` |
| BC-2.01.009 | `ss-01/BC-2.01.009.md` | 8 | `monocle-runtime/tests/auth_header_rejection.rs` |
| BC-2.01.010 | `ss-01/BC-2.01.010.md` | 3 | `monocle-runtime/tests/lock_file_contract.rs` |

### SS-02: Core Types and ABI (CAP-002)

| BC ID | BC File | Vector Count | Test File |
|-------|---------|-------------|-----------|
| BC-2.02.001 | `ss-02/BC-2.02.001.md` | 3 | `monocle-runtime/tests/status_abi_version.rs` |
| BC-2.02.002 | `ss-02/BC-2.02.002.md` | 2 | `monocle-core/tests/abi_stability.rs` |
| BC-2.02.003 | `ss-02/BC-2.02.003.md` | 2 | `monocle-core/tests/enum_audit.rs` |
| BC-2.02.004 | `ss-02/BC-2.02.004.md` | 3 | `monocle-core/tests/factory_trait_surface.rs` |
| BC-2.02.005 | `ss-02/BC-2.02.005.md` | 4 | `monocle-core/tests/factory_self_referential.rs` |
| BC-2.02.006 | `ss-02/BC-2.02.006.md` | 3 | `monocle-proto/tests/wire_field_order.rs` |
| BC-2.02.007 | `ss-02/BC-2.02.007.md` | 2 | `monocle-proto/tests/schema_version.rs` |
| BC-2.02.008 | `ss-02/BC-2.02.008.md` | 2 | Phase 4 integration test (future) |

### SS-03: Engine Module (CAP-003)

| BC ID | BC File | Vector Count | Test File |
|-------|---------|-------------|-----------|
| BC-2.03.001 | `ss-03/BC-2.03.001.md` | 3 | `monocle-core/tests/engine_module_surface.rs` |
| BC-2.03.002 | `ss-03/BC-2.03.002.md` | 5 | `monocle-runtime/tests/engine_module_claude_detect.rs` |
| BC-2.03.003 | `ss-03/BC-2.03.003.md` | 2 | `monocle-runtime/tests/engine_module_home_unresolvable.rs` |
| BC-2.03.004 | `ss-03/BC-2.03.004.md` | 3 | `monocle-runtime/tests/engine_module_claude_methods.rs` |

---

## Critical Test Vectors (Aggregated)

> The following vectors are extracted here because they represent the highest-risk
> behavioral boundaries. Test-writer must implement ALL of these in Phase 1.

### Auth Header Validation (BC-2.01.009)

> ADR-0005: dual-accept auth. Canonical header `X-Monocle-Authorization: monocle-v1:<64-hex>` takes priority.
> Compatibility alias `X-Claude-Code-Ide-Authorization: <raw-64-hex>` accepted with WARN deprecation log.
> Both headers absent → `missing_auth_token`. Alias-path entries are EC-010 from BC-2.01.009 v1.0.4.

| Input | Expected | Category |
|-------|----------|----------|
| No `X-Monocle-Authorization` header, no `X-Claude-Code-Ide-Authorization` header | HTTP 401 `{"error":"missing_auth_token"}` | error |
| `X-Monocle-Authorization: ` (empty value) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `X-Monocle-Authorization: monocle-v1:` (no hex suffix) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `X-Monocle-Authorization: monocle-v2:<token>` (wrong version prefix) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `Authorization: monocle-v1:<token>` (wrong header name, no alias header) | HTTP 401 `{"error":"missing_auth_token"}` | edge-case |
| `X-Monocle-Authorization: monocle-v1:<correct-64-hex>` | HTTP 200 (passes auth middleware) | happy-path |
| `X-Claude-Code-Ide-Authorization: <wrong-64-hex>` (alias path, wrong secret); no canonical header | HTTP 401 `{"error":"invalid_auth_token"}` + WARN deprecation log emitted | error |
| `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (alias path, correct secret); no canonical header | HTTP 200 (auth passes) + WARN deprecation log emitted | happy-path (alias) |

### Body Size Limit (BC-2.01.003)

| Input Body Size | Expected | Category |
|----------------|----------|----------|
| 262,145 bytes | HTTP 413 `{"error":"payload_too_large","limit_bytes":262144}` | edge-case |
| 262,144 bytes (exactly at limit) | HTTP 200 (hook processed) | edge-case |
| 262,143 bytes | HTTP 200 (hook processed) | edge-case |
| ~1 KiB (normal hook payload) | HTTP 200 (hook processed) | happy-path |

### Healthz vs Status Router Separation (BC-2.01.001, BC-2.01.002)

| Scenario | Input | Expected |
|----------|-------|----------|
| Healthz without auth | `GET /healthz` (no header) | HTTP 200 `{"status":"alive",...}` |
| Status without auth | `GET /status` (no header) | HTTP 401 `{"error":"missing_auth_token"}` |
| Healthz with auth | `GET /healthz` + valid auth header | HTTP 200 (auth header ignored; unauthenticated endpoint) |
| Status with valid auth | `GET /status` + valid auth header | HTTP 200 with 10-field body |

### JSONL Ring format_version First Key (BC-2.01.007)

| Scenario | Expected | Category |
|----------|----------|----------|
| Normal hook event serialized | Line begins with `{"format_version":1,` | happy-path |
| Non-tool hook (Notification) | `tool_name: null`, `tool_input: null`; `format_version` still first | edge-case |
| Near-maximum payload (256 KiB line) | Ring handles without truncation | edge-case |

### ClaudeCodeModule Detect (BC-2.03.002)

| exe_path | Expected detect() | Category |
|----------|-------------------|----------|
| `/usr/local/bin/claude` | `true` | happy-path |
| `/usr/local/bin/claude.js` | `true` | happy-path |
| `/usr/local/bin/claude-squad` | `false` | edge-case |
| `/usr/local/bin/claudio` | `false` | edge-case |
| `/usr/local/bin/claude-code-router` | `false` | edge-case |
| `None` (exe_path absent) | `false` | edge-case |

### Lock File contract_version Field (BC-2.01.010)

| Scenario | Input contract_version | Expected |
|----------|----------------------|----------|
| Known version | `1` | Lock file parsed; daemon proceeds normally |
| Unknown future version | `99` | WARN log `"lock file contract_version 99 not recognized; skipping"` |
| Missing field | (absent from JSON) | Same treatment as unknown version |
| Type coercion (string) | `"1"` | Graceful coerce or skip |

### Lock File Atomic Lifecycle — Start Sequence (BC-2.01.005)

| Scenario | Expected | Category |
|----------|----------|----------|
| No existing lock file | Lock file created at `<runtime_dir>/monocle.lock` with 0o600 mode | happy-path |
| Existing lock file, live pid (`kill(pid, 0)` succeeds) | Exit 1, `ERROR: daemon already running at pid=<N>` | edge-case |
| Existing lock file, dead pid | Stale lock removed (`WARN: stale lock file removed`); daemon starts | edge-case |
| `MONOCLE_RUNTIME_DIR=""` (empty string) | Treated as unset; platform default used; daemon starts | edge-case |
| All resolution paths return None | Exit 1, `ERROR: cannot resolve runtime directory` | edge-case |

---

## Cross-Subsystem Integration Vectors

| Scenario | Input | Step 1 | Step 2 | Final Output |
|----------|-------|--------|--------|-------------|
| Full killer scenario (4 keystrokes) | 2 concurrent PreToolUse hook POSTs with valid auth | Both HTTP 200; both enqueued to bounded channel | TUI shows both prompts simultaneously in VecDeque overlay | `HookDecision::Allow` for each; drain completes; 503 for post-drain arrivals |
| Daemon start → hook → ring flush | Fresh daemon start; one PreToolUse POST; graceful shutdown | Lock file written (0o600); ring created; hook processed | SIGTERM → 10-second drain; ring flushed | `monocle.ring.jsonl` contains one line beginning `{"format_version":1,` |
| Crash recovery flow | Daemon crash; TUI reconnects within 60 seconds | Recovery checkpoint file written at crash time | TUI reads recovery checkpoint; offers recovery to operator | Clean start if >60 seconds since crash |
| ABI version check (Phase 3 forward) | Phase 3 plugin SDK reads `/status` | HTTP 200 with `"abi_version": 1` | Plugin SDK compares against `MONOCLE_ABI_VERSION` const | Match → plugin loads; mismatch → compile-time assertion failure |

---

## Golden File References

| Vector Set | File | Format | BC Coverage |
|-----------|------|--------|------------|
| Auth rejection vectors | `test-data/auth-rejection-vectors.json` | JSON array | BC-2.01.009 |
| Body size limit vectors | (generated inline; no static file needed) | N/A | BC-2.01.003 |
| JSONL ring vectors | `test-data/ring-vectors.jsonl` | JSONL | BC-2.01.007 |
| Detect basename vectors | (inline in unit test) | Rust literals | BC-2.03.002 |

> Note: `test-data/` directory is a Phase 1 test deliverable. test-writer creates these files
> as part of story implementation. This table defines what must exist before Phase 3 closes.

---

## §Trace

### F-R107-1 + GAP-R46-4 PO closure — 2026-05-17T23:00:00Z

**Findings:**
- F-R107-1 CRITICAL — fabricated ADR-0005 path in frontmatter `inputs:`.
- GAP-R46-4 LOW — BC-2.01.009 version pin stale (v1.0.2; current is v1.0.4 post-PO-6A).

**F-R107-1 SE-17f before/after (inputs):**

**Before:** `inputs: [prd.md, behavioral-contracts/, architecture/adr/ADR-0005-dual-accept-auth-header.md]`
**After:** `inputs: [prd.md, behavioral-contracts/, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]`

**GAP-R46-4 SE-17f before/after (line 74 active body reference):**

**Before:** `Alias-path entries are EC-010 from BC-2.01.009 v1.0.2.`
**After:** `Alias-path entries are EC-010 from BC-2.01.009 v1.0.4.`

Note: v1.0.3 was the PO 5A bump target per the finding description, but actual current BC-2.01.009 version post-PO-6A is v1.0.4. The active citation is updated to the actual current version. Historical §Trace prose (lines 174, 176) preserves the original v1.0.2 reference as historical context — those are not updated (they document what version existed at that trace point).

**Changes made:** frontmatter `inputs:` ADR path corrected; line 74 version pin refreshed; version bumped 1.1 → 1.2; timestamp refreshed.

---

### F-R106-3 PO closure — 2026-05-17T22:00:00Z

**Finding:** F-R106-3 CRITICAL — test-vectors.md fully stale wrt ADR-0005 dual-accept auth. BC-2.01.009 v1.0.2 added 2 alias-path vectors (EC-010), bringing canonical vector count from 6 to 8. The BC Vector Index count was 6; the critical-vector table had zero alias-path rows; the canonical header description was missing dual-accept context.

**Canonical source:** BC-2.01.009 v1.0.2 §Canonical Test Vectors (lines 79-86) and §Edge Cases EC-010, EC-011, EC-012; ADR-0005 dual-accept auth header decision.

**SE-17f — Count change:**

**Before:** `| BC-2.01.009 | ... | 6 | ...`
**After:** `| BC-2.01.009 | ... | 8 | ...`

**SE-17c — Before (§Auth Header Validation critical-vector table — 6 rows):**

```
| No `X-Monocle-Authorization` header | HTTP 401 `{"error":"missing_auth_token"}` | error |
| `X-Monocle-Authorization: ` (empty value) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `X-Monocle-Authorization: monocle-v1:` (no hex suffix) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `X-Monocle-Authorization: bearer:<token>` (wrong version prefix) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `Authorization: monocle-v1:<token>` (wrong header name) | HTTP 401 `{"error":"missing_auth_token"}` | edge-case |
| `X-Monocle-Authorization: monocle-v1:<correct-64-hex>` | HTTP 200 (passes auth middleware) | happy-path |
```

**SE-17d — After (§Auth Header Validation critical-vector table — 8 rows, 2 alias-path rows added):**

```
| No `X-Monocle-Authorization` header, no `X-Claude-Code-Ide-Authorization` header | HTTP 401 `{"error":"missing_auth_token"}` | error |
| `X-Monocle-Authorization: ` (empty value) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `X-Monocle-Authorization: monocle-v1:` (no hex suffix) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `X-Monocle-Authorization: bearer:<token>` (wrong version prefix) | HTTP 401 `{"error":"invalid_auth_token"}` | edge-case |
| `Authorization: monocle-v1:<token>` (wrong header name, no alias header) | HTTP 401 `{"error":"missing_auth_token"}` | edge-case |
| `X-Monocle-Authorization: monocle-v1:<correct-64-hex>` | HTTP 200 (passes auth middleware) | happy-path |
| `X-Claude-Code-Ide-Authorization: <wrong-64-hex>` (alias path, wrong secret); no canonical header | HTTP 401 `{"error":"invalid_auth_token"}` + WARN deprecation log emitted | error |
| `X-Claude-Code-Ide-Authorization: <correct-64-hex>` (alias path, correct secret); no canonical header | HTTP 200 (auth passes) + WARN deprecation log emitted | happy-path (alias) |
```

**Changes made:**
- BC Vector Index count for BC-2.01.009: 6 → 8
- §Auth Header Validation critical-vector table: 6 rows → 8 rows (added 2 alias-path rows)
- Row 1 input updated: "No `X-Monocle-Authorization` header" → explicit dual-absence ("no canonical header, no alias header") to match BC-2.01.009 line 79
- Row 5 (wrong header name) clarified: appended "(no alias header)" to avoid ambiguity with dual-header scenarios
- Added ADR-0005 context note above critical-vector table
- Frontmatter: v1.0 → v1.1; timestamp refreshed; ADR-0005 added to inputs
- Version bump: v1.0 → v1.1

---

### F-R108-14 PO closure — 2026-05-18T01:00:00Z

**Finding:** F-R108-14 MEDIUM — §Auth Header Validation table row 4 (wrong version prefix edge case) used `bearer:<token>` as the test input. This is incorrect; `bearer:` is not a valid monocle token prefix form and is not the canonical wrong-version-prefix test vector. The canonical wrong-version-prefix form per BC-2.01.009 and VP-009 is `monocle-v2:<token>` — a valid-looking future version prefix that a Phase 2/3/4 monocle daemon might use, demonstrating that the auth middleware correctly rejects prefix-version mismatches.

**Canonical source:** BC-2.01.009 + VP-009 canonical wrong-version-prefix form. Using `monocle-v2:` makes the test vector self-documenting: it exercises the version-prefix rejection rule (not just a malformed token), which is the actual security property being tested.

**SE-17f — Before/After (row 4, active table at §Auth Header Validation):**

**Before:** `| \`X-Monocle-Authorization: bearer:<token>\` (wrong version prefix) | HTTP 401 \`{"error":"invalid_auth_token"}\` | edge-case |`
**After:** `| \`X-Monocle-Authorization: monocle-v2:<token>\` (wrong version prefix) | HTTP 401 \`{"error":"invalid_auth_token"}\` | edge-case |`

**Note:** The §SE-17d (After) historical record in the prior §Trace entry (F-R106-3) also shows `bearer:<token>` — that accurately records the state as of Round 6 (the bug was present then). The historical record is preserved as-is; the active body row is corrected to `monocle-v2:<token>`.

**Changes made:** Row 4 input in active §Auth Header Validation table: `bearer:<token>` → `monocle-v2:<token>`; version bumped v1.2 → v1.3; timestamp refreshed.

**Scope:** PO-only. No changes to BC-2.01.009, VP-009, or any architecture artifact.
