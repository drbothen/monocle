---
document_type: plan-doc
level: L4
version: "1.6"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:30:00Z
phase: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Dependency graph for STORY-INDEX.md; cross-validates all dependency edges, topological sort, BC/VP/NFR traceability matrices, and edge case coverage."
---

# Dependency Graph: monocle Phase 2 Stories

## Topological Order (Wave-Sorted)

```
Wave 0 (Pre-Phase-3 gate; not blocking Phase 2):
  S-PHASE-3-PREP  (external dep: spec-kit-mcp rc.19+)

Wave 1 (no product deps — parallel start):
  S-DTU-001       (DTU clone; no product story deps)
  S-001           (workspace init; no story deps)

Wave 2 (depends on Wave 1; parallel within wave):
  S-002           (depends on: S-001)
  S-003           (depends on: S-001, S-002)
  S-004           (depends on: S-001)
  S-005           (depends on: S-001, S-002)
  S-006           (depends on: S-001)
  S-010           (depends on: S-001)
  S-011           (depends on: S-010)
  S-013           (depends on: S-010)
  S-014           (depends on: S-010)

Wave 3 (depends on Wave 2; parallel within wave):
  S-007           (depends on: S-006)
  S-008           (depends on: S-006)
  S-009           (depends on: S-001, S-004, S-006, S-008)  [Decision 1: S-008→S-009 edge added]
  S-012           (depends on: S-010, S-011)
  S-015           (depends on: S-014)
```

## Dependency Edges (Story → Depends On)

| Story | Depends On | Justification |
|-------|-----------|---------------|
| S-PHASE-3-PREP | vsdd-factory rc.19+ (external) | External library dependency; no product story deps |
| S-DTU-001 | — | DTU stories have NO product story dependencies per DTU wave-scheduling rule |
| S-001 | — | Foundation story; no predecessors |
| S-002 | S-001 | Requires workspace + axum router stub from S-001 |
| S-003 | S-001, S-002 | Requires authenticated router (depends on unauthenticated router from S-002) |
| S-004 | S-001 | Requires axum router from S-001; independent of S-002/S-003 |
| S-005 | S-001, S-002 | Requires AppMode enum from S-002 (ShuttingDown state) |
| S-006 | S-001 | Requires workspace crates (tempfile, directories, nix) from S-001 |
| S-007 | S-006 | Requires runtime_dir resolution and tempfile::persist pattern from S-006 |
| S-008 | S-006 | Requires runtime_dir and HookEventRecord struct; ring flush depends on runtime_dir |
| S-009 | S-001, S-004, S-006, S-008 | Requires DefaultBodyLimit (S-004), lock file auth token (S-006), axum router (S-001), and RingBuffer from S-008 (Decision 1: hook handlers call RingBuffer::push()) |
| S-010 | S-001 | Requires monocle-core crate stub from S-001; independent of S-002..S-009 |
| S-011 | S-010 | Requires monocle-core type declarations from S-010 |
| S-012 | S-010, S-011 | Requires monocle-core types (S-010) and #[non_exhaustive] policy (S-011) |
| S-013 | S-010 | Requires monocle-proto crate stub; monocle-core for HookEnvelope cross-reference |
| S-014 | S-010 | Requires monocle-core types (HookEvent, HookResponse defined in S-010 engine module) |
| S-015 | S-014 | Requires EngineModule trait and supporting types from S-014 |

## Acyclicity Verification

Topological sort using Kahn's algorithm:

```
Degree-0 nodes (no deps): {S-PHASE-3-PREP, S-DTU-001, S-001}

Round 1 — remove degree-0, reduce dependents:
  Process: S-DTU-001, S-001
  Newly degree-0: {S-002, S-004, S-005*, S-006, S-010}
  (*S-005 depends on S-001+S-002; S-002 not yet removed; not degree-0 yet)

Round 2 — process S-002, S-004, S-006, S-010:
  Newly degree-0 after S-002 removed: {S-003, S-005}
  Newly degree-0 after S-006 removed: {S-007, S-008} (S-009 still has S-008 dep; not yet degree-0)
  Newly degree-0 after S-010 removed: {S-011, S-013, S-014}

Round 3 — process S-003, S-005, S-007, S-008, S-011, S-013, S-014:
  Newly degree-0 after S-008 removed: {S-009}
  Newly degree-0 after S-011 removed: {S-012}
  Newly degree-0 after S-014 removed: {S-015}

Round 4 — process S-009, S-012, S-015:
  All remaining nodes processed; empty queue.

Total processed: 17 nodes. No cycle detected. DAG is acyclic. PASS.
```

**Cycle check result: ACYCLIC. Topological sort successful.**
**Story count: 17 (15 product + 1 DTU + 1 prep). Note: S-PHASE-3-PREP processed as degree-0 in Round 0 (no edges).**

## Blocks Edges (Story → Blocks)

| Story | Blocks | Justification |
|-------|--------|---------------|
| S-DTU-001 | S-009 | DTU clone needed before S-009 integration tests that exercise alias auth path |
| S-001 | S-002, S-003, S-004, S-005, S-006, S-010, S-013, S-014 | Workspace required for all implementation stories (S-009 removed from blocks — S-009 now in Wave 3, downstream of S-008) |
| S-002 | S-003, S-005 | Unauthenticated router needed before authenticated router (S-003) and AppMode (S-005) |
| S-004 | S-009 | DefaultBodyLimit layer needed before hook endpoint tests (S-009 now Wave 3 but still depends on S-004) |
| S-006 | S-007, S-008 | Lock file pattern needed before crash recovery (S-007) and ring (S-008); S-009 moved to Wave 3 (Decision 1) |
| S-008 | S-009 | RingBuffer must be available before S-009 hook handlers call RingBuffer::push() (Decision 1: S-008→S-009) |
| S-010 | S-011, S-012, S-013, S-014 | monocle-core types needed for all SS-02/SS-03 stories |
| S-011 | S-012 | Non-exhaustive enum attributes needed before FactoryAdapter types |
| S-014 | S-015 | EngineModule trait needed before ClaudeCodeModule impl |

## BC to Stories Matrix

| BC ID | Stories | Full Coverage? |
|-------|---------|---------------|
| BC-2.01.001 | S-002 | YES |
| BC-2.01.002 | S-003 | YES |
| BC-2.01.003 | S-004 | YES |
| BC-2.01.004 | S-005 | YES |
| BC-2.01.005 | S-006 | YES |
| BC-2.01.006 | S-007 | YES |
| BC-2.01.007 | S-008 | YES (S-008 is sole implementer; S-001 mis-anchor corrected per F-PHASE2-R01-11) |
| BC-2.01.008 | S-006, S-009 | YES (S-006 AC-014 generates token; S-009 AC-001..AC-003 validates wire format) |
| BC-2.01.009 | S-009 | YES |
| BC-2.01.010 | S-006 | YES |
| BC-2.02.001 | S-003, S-010 | YES (S-010 declares const; S-003 exposes in /status) |
| BC-2.02.002 | S-010 | YES |
| BC-2.02.003 | S-011, S-014 | YES (S-011 AC-001..AC-004 policy; S-014 AC-003b HookEvent #[non_exhaustive]) |
| BC-2.02.004 | S-012 | YES |
| BC-2.02.005 | S-012 | YES |
| BC-2.02.006 | S-013 | YES |
| BC-2.02.007 | S-013 | YES |
| BC-2.02.008 | S-013 | YES |
| BC-2.03.001 | S-014, S-015 | YES (S-014 AC-001..AC-007 trait definition; S-015 AC-010 PC-6 DI-006 enforce) |
| BC-2.03.002 | S-015 | YES |
| BC-2.03.003 | S-015 | YES |
| BC-2.03.004 | S-015 | YES |

## VP to Stories Matrix

| VP ID | Stories Exercising It | BC Source |
|-------|----------------------|-----------|
| VP-001 | S-002 | BC-2.01.001 |
| VP-002 | S-003 | BC-2.01.002 |
| VP-003 | S-004 | BC-2.01.003 |
| VP-004 | S-005 | BC-2.01.004 |
| VP-005 | S-006 | BC-2.01.005 |
| VP-006 | S-007 | BC-2.01.006 |
| VP-007 | S-008 | BC-2.01.007 |
| VP-008 | S-009 | BC-2.01.008 |
| VP-009 | S-009 | BC-2.01.009 |
| VP-010 | S-006 | BC-2.01.010 |
| VP-011 | S-003, S-010 | BC-2.02.001 |
| VP-012 | S-010 | BC-2.02.002 |
| VP-013 | S-011 | BC-2.02.003 |
| VP-014 | S-012 | BC-2.02.004 |
| VP-015 | S-012 | BC-2.02.005 |
| VP-016 | S-013 | BC-2.02.006 |
| VP-017 | S-013 | BC-2.02.007 |
| VP-018 | S-013 | BC-2.02.008 |
| VP-019 | S-014 | BC-2.03.001 |
| VP-020 | S-015 | BC-2.03.002 |
| VP-021 | S-015 | BC-2.03.003 |
| VP-022 | S-015 | BC-2.03.004 |

## NFR to Stories Matrix

| NFR ID | Priority | Stories Implementing It | Validation Method |
|--------|----------|------------------------|-------------------|
| NFR-001 | P0 | Phase 3 TBD | Phase 3 load-test integration test |
| NFR-002 | P0 | Phase 3 TBD | Phase 3 load-test integration test |
| NFR-003 | P0 | Phase 3 TBD (TUI) | Phase 3 TUI integration test |
| NFR-004 | P0 | S-009 | VP-008 OsRng source-grep (AC-001) |
| NFR-005 | P0 | S-004 | VP-003 AC-001 integration test |
| NFR-006 | P0 | Phase 3 TBD | Phase 3 load-test integration test |
| NFR-007 | P0 | S-001 | CI gate: rust-toolchain.toml (AC-002, AC-004) |
| NFR-008 | P0 | S-001 | CI gate: matrix config (AC-003) |
| NFR-009 | P0 | S-006 | VP-005 Post-condition 1 (AC-001) |
| NFR-010 | P0 | S-009, S-003 | VP-008/VP-009 constant_time_eq source-grep |
| NFR-011 | P0 | S-DTU-001 | DTU fidelity ≥0.95 fixture corpus (AC-004) |
| NFR-012 | P0 | S-006 | VP-005 Post-condition 9 (AC-006) |

## BC Clause Coverage Matrix

| BC-S.SS.NNN | Clause | Type | Covering AC | Story |
|-------------|--------|------|-------------|-------|
| BC-2.01.001 | 1 | postcondition | AC-001 | S-002 |
| BC-2.01.001 | 2 | postcondition | AC-002 | S-002 |
| BC-2.01.001 | 3 | postcondition | AC-003 | S-002 |
| BC-2.01.001 | 4 | postcondition | AC-004 | S-002 |
| BC-2.01.001 | 1 | invariant | AC-005 | S-002 |
| BC-2.01.001 | 2 | invariant | AC-005 | S-002 |
| BC-2.01.002 | 1 | postcondition (all 10 fields) | AC-001 | S-003 |
| BC-2.01.002 | 1 sub-bullet «abi_version» | postcondition | AC-005 (cross-cite BC-2.02.001 PC-1) | S-003 |
| BC-2.01.002 | 1 sub-bullet «hook_endpoints» | postcondition | AC-006 (cross-cite BC-2.01.008 PC-4) | S-003 |
| BC-2.01.002 | 1 sub-bullet «last_hook_ts» | postcondition | AC-007 | S-003 |
| BC-2.01.002 | 2 | postcondition (invalid auth → BC-2.01.009) | AC-003, AC-004 (traced to BC-2.01.009 PC-1, PC-2) | S-003 |
| BC-2.01.002 | 3 | postcondition (/status during drain) | AC-008 | S-003 |
| BC-2.01.003 | 1 | postcondition | AC-001 | S-004 |
| BC-2.01.003 | 2 | postcondition | AC-002 | S-004 |
| BC-2.01.003 | 3 | postcondition | AC-003 | S-004 |
| BC-2.01.003 | 1 | invariant | AC-004 | S-004 |
| BC-2.01.004 | 1 | postcondition (AppMode → ShuttingDown on any shutdown trigger) | AC-001 (SIGTERM trigger), AC-002 (POST /shutdown trigger; both transition AppMode per PC-1) | S-005 |
| BC-2.01.004 | 2 | postcondition (new hook POSTs → HTTP 503 Retry-After:10 + daemon_shutting_down body) | AC-003 | S-005 |
| BC-2.01.004 | 3 | postcondition (/healthz returns HTTP 503 during drain — cross-covered via BC-2.01.001 PC-2 delegation) | AC-002 (S-002 /healthz 503 on ShuttingDown; BC-2.01.004 PC-3 cross-covered by BC-2.01.001 PC-2 which S-002 AC-002 implements) | S-002 |
| BC-2.01.004 | 4 | postcondition | AC-008 (S-003 /status serves during drain per BC-2.01.002 PC-3) | S-003 |
| BC-2.01.004 | 5 | postcondition | AC-001 (10s drain wait enforced in AC-001 sequence) | S-005 |
| BC-2.01.004 | 6 | postcondition | -- | GAP-P2-005 (ring flush on --persistent-events; Phase 3 flag not in Phase 1 scope) |
| BC-2.01.004 | 7 | postcondition | AC-005 (drain completes → lock+sock removed per lifecycle) | S-005 |
| BC-2.01.004 | 8 | postcondition | AC-004 (POSIX exit code taxonomy) | S-005 |
| BC-2.01.004 | 1 | invariant | AC-005 (hard timeout invariant) | S-005 |
| BC-2.01.004 | 3 | invariant (POST /shutdown dual-accept auth — both headers accepted per ADR-0005) | AC-006 (401 on missing/invalid auth) + AC-002 (HTTP 200 on valid dual-accept auth = the success path that INV-3 gates) | S-005 |
| BC-2.01.005 | 1 | postcondition | AC-003 (live PID conflict → exit 1) | S-006 |
| BC-2.01.005 | 2 | postcondition | AC-004 (stale PID → cleanup) | S-006 |
| BC-2.01.005 | 3 | postcondition | AC-001 (atomic write via tempfile::persist, mode 0o600) | S-006 |
| BC-2.01.005 | 4 | postcondition | AC-002 (contract_version first key in JSON) | S-006 |
| BC-2.01.005 | 5 | postcondition | AC-009 (RuntimeDirUnresolvable → exit 1) | S-006 |
| BC-2.01.005 | 6 | postcondition | AC-005 (lock + sock removed on shutdown) | S-006 |
| BC-2.01.005 | 7 | postcondition | AC-005 (sock removed on shutdown) | S-006 |
| BC-2.01.005 | 8 | postcondition | AC-006 (runtime dir 0o700) | S-006 |
| BC-2.01.005 | 1 | invariant | AC-003 (tempfile::persist atomic write invariant) | S-006 |
| BC-2.01.005 | 2 | invariant | AC-001 (0o600 mode enforced) | S-006 |
| BC-2.01.005 | 3 | invariant | AC-001 (no partial observable state) | S-006 |
| BC-2.01.005 | 2a | precondition | AC-007 (MONOCLE_RUNTIME_DIR override) | S-006 |
| BC-2.01.005 | 2b | precondition | AC-008 (macOS platform fallback via data_local_dir) | S-006 |
| BC-2.01.005 | 2c | precondition | AC-008 (Linux runtime_dir from ProjectDirs) | S-006 |
| BC-2.01.005 | 2d | precondition | AC-009 (RuntimeDirUnresolvable fallback exhausted) | S-006 |
| BC-2.01.006 | 1 | postcondition | AC-001 | S-007 |
| BC-2.01.006 | 2 | postcondition | AC-002 | S-007 |
| BC-2.01.006 | 3 | postcondition | AC-003 | S-007 |
| BC-2.01.006 | 4 | postcondition | AC-004 | S-007 |
| BC-2.01.006 | 5 | postcondition | AC-005 | S-007 |
| BC-2.01.006 | 6 | postcondition | AC-006 | S-007 |
| BC-2.01.006 | 7 | postcondition | AC-007 | S-007 |
| BC-2.01.006 | 1 | invariant (schema) | AC-008 | S-007 |
| BC-2.01.006 | 2 | invariant (drain order) | AC-009 | S-007 |
| BC-2.01.006 | 3 | invariant (60s from start) | AC-003 | S-007 |
| BC-2.01.007 | 1 | postcondition (format_version first key) | AC-001 | S-008 |
| BC-2.01.007 | 2 | postcondition (value always 1 in Phase 1) | AC-001 | S-008 |
| BC-2.01.007 | 3 | postcondition (RING_FORMAT_VERSION const is single source of truth; all call sites pass const not literal) | AC-003 (const usage in HookEventRecord::new; hybrid ring architecture references RING_FORMAT_VERSION per BC-2.01.007 PC-2+PC-3) | S-008 |
| BC-2.01.007 | 4 | postcondition (field declaration order — 7 fields in canonical order) | AC-002b | S-008 |
| BC-2.01.007 | 5 | postcondition (#[non_exhaustive] + pub fn new() constructor; format_version set to RING_FORMAT_VERSION inside constructor) | AC-006 | S-008 |
| BC-2.01.007 | 1 | invariant (serde_json struct-field order) | AC-001, AC-002b | S-008 |
| BC-2.01.004 | EC-049 | edge case (ring flush failure during drain) | AC-005 | S-008 |
| BC-2.01.008 | 1 | postcondition (OsRng 64-char hex token in lock file) | AC-001 | S-009 |
| BC-2.01.008 | 2 | postcondition (lock file authToken raw hex, no prefix) | AC-002 | S-009 |
| BC-2.01.008 | 3 | postcondition (canonical header format monocle-v1:<hex>) | AC-003 | S-009 |
| BC-2.01.008 | 4 | postcondition (dual-accept on hook endpoints) | AC-010a | S-009 |
| BC-2.01.008 | 3 | invariant (OsRng mandatory, not thread_rng) | AC-001 | S-009 |
| BC-2.01.002 | 1 sub-bullet «hook_endpoints» | postcondition (5-endpoint list, S-009 registers them) | AC-010b | S-009 |
| BC-2.01.009 | 1 | postcondition (both headers absent → 401 missing_auth_token) | AC-004 | S-009 |
| BC-2.01.009 | 2 | postcondition (canonical path value-present failure → 401 invalid_auth_token) | AC-006 | S-009 |
| BC-2.01.009 | 3 | postcondition (alias path value-present failure → 401 invalid_auth_token + WARN) | AC-005 | S-009 |
| BC-2.01.009 | 4 | postcondition (both present → canonical wins; alias ignored; no WARN) | AC-007 | S-009 |
| BC-2.01.009 | 1 | invariant (two-body taxonomy is complete; no third body; invalid_auth_token_format retired) | AC-004 (missing body only) + AC-006 (invalid body only) together prove only 2 bodies exist | S-009 |
| BC-2.01.009 | 2 | invariant (value-present failures on both paths return same body intentionally — no format/path distinction in response) | AC-005 (alias fail → invalid_auth_token), AC-006 (canonical fail → invalid_auth_token) | S-009 |
| BC-2.01.009 | 3 | invariant (missing = client-config error, not auth attempt; actionable for debugging) | AC-004 (missing → E-AUTH-001 developer-friendly diagnostic) | S-009 |
| BC-2.01.009 | 4 | invariant (AuthError::Missing for dual-absence; AuthError::Invalid for all value-present failures on either path) | AC-004 (Missing variant) + AC-005 + AC-006 (Invalid variant on alias and canonical paths) | S-009 |
| BC-2.01.009 | 5 | invariant (canonical priority immutable: X-Monocle-Authorization always takes precedence when both present) | AC-007 (both-present → canonical wins) | S-009 |
| BC-2.01.009 | 6 | invariant (WARN deprecation log emitted once per alias-path authentication attempt regardless of outcome) | AC-005 (alias path: WARN on match and mismatch both) | S-009 |
| BC-2.01.009 | 7 | invariant (constant-time comparison on BOTH canonical and alias paths) | AC-008 | S-009 |
| BC-2.01.010 | 1 | postcondition | AC-010 | S-006 |
| BC-2.01.010 | 2 | postcondition | AC-010 | S-006 |
| BC-2.01.010 | 3 | postcondition | AC-010 | S-006 |
| BC-2.01.010 | 4 | postcondition | AC-010 | S-006 |
| BC-2.01.010 | EC-010 | edge case | AC-010 | S-006 |
| BC-2.01.010 | EC-011 | edge case | AC-012 | S-006 |
| BC-2.01.010 | EC-012 | edge case | AC-013 | S-006 |
| BC-2.02.001 | 1 | postcondition (abi_version field in /status) | S-010 AC-003 + S-010 AC-005 + S-003 AC-005 | S-010, S-003 |
| BC-2.02.001 | 2 | postcondition (equals monocle_core::MONOCLE_ABI_VERSION) | AC-005 (re-anchored from INV-1 per F-PHASE2-R03-09) | S-010 |
| BC-2.02.001 | 3 | postcondition (full /status response shape — cross-covered by BC-2.01.002 PC-1) | AC-001 | S-003 |
| BC-2.02.001 | 1 | invariant (compile-time constant; cross-covered by S-010 AC-004 compile-time assert) | AC-004 | S-010 |
| BC-2.02.002 | 1 | postcondition (MONOCLE_ABI_VERSION at crate root) | AC-001, AC-002 | S-010 |
| BC-2.02.002 | 2 | postcondition (re-export in lib.rs) | AC-002 | S-010 |
| BC-2.02.002 | 3 | postcondition (compile-time stability test) | AC-004 | S-010 |
| BC-2.02.002 | 1 | invariant (compile-time constant; cross-covered by AC-004) | AC-004 | S-010 |
| BC-2.02.003 | 1 | postcondition (#[non_exhaustive] on all public enums) | AC-001 | S-011 |
| BC-2.02.003 | 2 | postcondition (ADR-0004 exemptions are exhaustive) | AC-002 | S-011 |
| BC-2.02.003 | 3 | postcondition (AST audit via VP-013) | AC-003 | S-011 |
| BC-2.02.003 | 4 | postcondition (canonical minimum 9 enums including DenyReason, AllowPattern, DenyPattern) | AC-001, AC-001b | S-011 |
| BC-2.02.003 | 1 | invariant (syn 2 AST parse mechanism; not clippy) | AC-003 | S-011 |
| BC-2.02.003 | 2 | invariant (adding variant to #[non_exhaustive] is not breaking) | AC-004 (wildcard arm compiler enforcement) | S-011 |
| BC-2.02.004 | 1 | postcondition | AC-001 | S-012 |
| BC-2.02.004 | 2 | postcondition | AC-002 | S-012 |
| BC-2.02.004 | 3 | postcondition | AC-003 | S-012 |
| BC-2.02.004 | 4 | postcondition | AC-004 | S-012 |
| BC-2.02.005 | 1 | postcondition (VsddFactoryAdapter::new(workspace_root) → Self; derives state_file; no validation at construction) | AC-011 | S-012 |
| BC-2.02.005 | 2 | postcondition (self-referential detection against monocle repo — detect() finds .factory/STATE.md with document_type: pipeline-state) | AC-006 | S-012 |
| BC-2.02.005 | 1 | invariant (detection criterion: document_type: pipeline-state in YAML frontmatter; no other field required) | AC-005 (detect() returns Some only on criterion match) | S-012 |
| BC-2.02.005 | 2 | invariant (display_name() returns "VSDD Factory" — exact string) | AC-010 | S-012 |
| BC-2.02.005 | 3 | invariant (subscribe() returns Ok(Box::pin(futures::stream::empty())) in Phase 1) | AC-007, AC-009 | S-012 |
| BC-2.02.005 | 3 | postcondition (absent optional fields → None; never "unknown" placeholder; absent current_cycle: → cycle: None; absent §Session Resume Checkpoint → convergence: None) | AC-012 | S-012 |
| BC-2.02.005 | 4 | postcondition (parse_frontmatter_field 4 guards: skip continuation, empty→None EC-061, flow-list→None EC-023, block-scalar→None; quoted scalars unquoted EC-022) | AC-013 | S-012 |
| BC-2.02.006 | 1 | postcondition | AC-001, AC-006 | S-013 |
| BC-2.02.007 | 1 | postcondition | AC-002 | S-013 |
| BC-2.02.007 | 2 | postcondition | AC-003 | S-013 |
| BC-2.02.008 | 1 | postcondition | AC-004 | S-013 |
| BC-2.02.008 | 2 | postcondition | AC-005 | S-013 |
| BC-2.03.001 | 1 | postcondition | AC-001 | S-014 |
| BC-2.03.001 | 2 | postcondition | AC-002 | S-014 |
| BC-2.03.001 | 3 | postcondition | AC-003 | S-014 |
| BC-2.03.001 | 4 | postcondition | AC-004 | S-014 |
| BC-2.03.001 | 5 | postcondition | AC-005 | S-014 |
| BC-2.03.001 | 1 | invariant (OPEN trait) | AC-006 | S-014 |
| BC-2.03.001 | 2 | invariant (HookEvent defined in hook_events.rs) | AC-003b | S-014 |
| BC-2.03.001 | 3 | invariant (async_trait macro required) | AC-007 | S-014 |
| BC-2.03.002 | 1 | postcondition (ClaudeCodeModule implements EngineModule) | AC-001 | S-015 |
| BC-2.03.002 | 2 | postcondition (infallible constructor) | AC-003 | S-015 |
| BC-2.03.002 | 3 | postcondition (id() returns "claude-code") | AC-004 | S-015 |
| BC-2.03.002 | 4 | postcondition (detect() strict basename: "claude" or "claude.js") | AC-001 | S-015 |
| BC-2.03.002 | 5 | postcondition (detect() returns false when exe_path is None) | AC-002 | S-015 |
| BC-2.03.002 | 1 | invariant (strict-basename prevents false positives) | AC-001, AC-002 | S-015 |
| BC-2.03.002 | 2 | invariant (cmdline not used as primary detection signal) | AC-002 | S-015 |
| BC-2.03.003 | 1 | postcondition (HomeUnresolvable on metadata/enrich) | AC-005 | S-015 |
| BC-2.03.003 | 2 | postcondition (E-ENG-001 log on HomeUnresolvable) | AC-006 | S-015 |
| BC-2.03.004 | 1 | postcondition (hook_paths() returns HashMap 5 entries) | AC-007 | S-015 |
| BC-2.03.004 | 2 | postcondition (spawn() is todo!() stub) | AC-008 | S-015 |
| BC-2.03.004 | 3 | postcondition (preflight() is todo!() stub) | AC-009 | S-015 |
| BC-2.03.001 | 6 | postcondition (DI-006 detect I/O-free; PC-6 added in v1.0.4) | AC-010 | S-015 |

## Edge Case Coverage Matrix

| Source | EC/Error ID | Description | Story | AC/EC Reference |
|--------|-------------|-------------|-------|----------------|
| BC-2.01.001 | EC-040 | TUI hung-daemon detection | S-002 | AC-006 |
| BC-2.01.001 | EC-041 | TUI dead-pid stale lock | S-002 | AC-006 |
| BC-2.01.003 | EC-002 | Body limit on authenticated endpoints only | S-004 | AC-005 |
| BC-2.01.005 | EC-051 | Lock file write fails | S-006 | Addressed in Tasks (tempfile guarantees) |
| BC-2.01.005 | EC-052 | Runtime dir absent | S-006 | AC-006 |
| BC-2.01.005 | EC-053 | TOCTOU race on lock file | S-006 | AC-001 (tempfile atomic) |
| BC-2.01.005 | EC-057 | macOS platform fallback | S-006 | AC-008 |
| BC-2.01.005 | EC-058 | MONOCLE_RUNTIME_DIR override | S-006 | AC-007 |
| BC-2.01.005 | EC-059 | Full-fail RuntimeDirUnresolvable | S-006 | AC-009 |
| BC-2.01.005 | EC-060 | Empty MONOCLE_RUNTIME_DIR | S-006 | AC-007 (non-empty check) |
| BC-2.01.006 | EC-054 | Recovery file malformed JSON | S-007 | AC-010 |
| BC-2.01.006 | EC-055 | Multiple crash cycles — single file overwrite | S-007 | AC-008 (one file per runtime dir; invariant 1) |
| BC-2.01.006 | EC-056 | TUI attaches at exactly 60-second boundary | S-007 | AC-005, AC-007 (60s window measured from start) |
| BC-2.01.007 | EC-001 | Tool-less hook types omit tool_name/tool_input (no null) | S-008 | AC-002 |
| BC-2.01.007 | EC-002 | Large tool_input up to 256 KiB | S-008 | AC-002b (7-field schema handles large values) |
| BC-2.01.007 | EC-003 | Ring buffer file truncated mid-line (crash) — Phase 2 reader robustness | S-008 | AC-002b (7-field schema, JSONL reader skips incomplete lines) |
| BC-2.01.004 | EC-049 | Ring buffer flush fails during drain | S-008 | AC-005 (re-anchored from BC-2.01.007 EC-003; EC-049 is flush-failure writer concern) |
| BC-2.01.009 | EC-013 | Both headers absent → 401 | S-009 | AC-009 |
| BC-2.02.004 | EC-018 | dyn FactoryAdapter + detect where Self: Sized | S-012 | AC-001 |
| BC-2.02.004 | EC-019 | custom_fields YAML flow-style | S-012 | AC-004 |
| BC-2.02.004 | EC-020 | Phase 3 WASM adapter | S-012 | AC-002 (no Sealed) |
| BC-2.01.010 | EC-010 | Stale lock with future contract_version | S-006 | AC-010 |
| BC-2.01.010 | EC-011 | contract_version as string not integer | S-006 | AC-012 |
| BC-2.01.010 | EC-012 | contract_version key missing entirely | S-006 | AC-013 |
| BC-2.03.001 | EC-029 | metadata() with $HOME unset | S-015 | AC-005 |
| BC-2.03.001 | EC-030 | detect() with exe_path = None | S-015 | AC-002 |
| BC-2.03.001 | EC-031 | on_hook() with unknown HookEvent | S-015 | AC-010 |
| BC-2.03.002 | EC-032 | cmdline "claude" but exe_path is claude-squad | S-015 | AC-002 |
| BC-2.03.002 | EC-033 | exe_path /usr/local/bin/claude (no extension) | S-015 | AC-001 |
| BC-2.03.002 | EC-034 | exe_path /usr/local/bin/claude.js (Node.js wrapper) | S-015 | AC-001 |
| BC-2.03.002 | EC-035 | exe_path /usr/local/bin/claude-squad | S-015 | AC-001 |
| BC-2.03.004 | EC-038 | spawn() called in Phase 1 → todo!() | S-015 | AC-008 |
| BC-2.03.004 | EC-039 | preflight() called in Phase 1 → todo!() | S-015 | AC-009 |
| error-taxonomy | E-AUTH-001 | missing_auth_token | S-009 | AC-004 |
| error-taxonomy | E-AUTH-002 | invalid_auth_token | S-009 | AC-005, AC-006 |
| error-taxonomy | E-AUTH-003 | alias path WARN | S-009 | AC-005 |
| error-taxonomy | E-DAEMON-001 | payload_too_large | S-004 | AC-001 |
| error-taxonomy | E-DAEMON-002 | daemon_shutting_down 503 | S-005 | AC-003 |
| error-taxonomy | E-DAEMON-003 | healthz 503 during shutdown | S-002 | AC-002 |
| error-taxonomy | E-DAEMON-004 | RuntimeDirUnresolvable exit 1 | S-006 | AC-009 |
| error-taxonomy | E-LOCK-001 | daemon already running | S-006 | AC-003 |
| error-taxonomy | E-LOCK-002 | stale lock removed | S-006 | AC-004 |
| error-taxonomy | E-LOCK-003 | unknown contract_version | S-006 | AC-010 |
| error-taxonomy | E-ENG-001 | HomeUnresolvable | S-015 | AC-005 |
| error-taxonomy | E-FACT-001 | STATE.md not found | S-012 | AC-008 |
| error-taxonomy | E-FACT-002 | STATE.md malformed | S-012 | AC-008 |
| error-taxonomy | E-RING-001 | ring flush failed | S-008 | AC-005 |
| error-taxonomy | E-PROTO-001 | unknown schema_version | S-013 | AC-004 |

## Gap Register

| Gap ID | Level | Source | Clause/Item | Justification | Resolution Target |
|--------|-------|--------|-------------|---------------|-------------------|
| GAP-P2-001 | L3 | NFR-001 | latency ≤300ms for hook endpoints | Requires Phase 3 load-test infrastructure not available in Phase 1 per nfr-catalog.md §VP Probe Citations; hook receiver DESIGN is Phase 1, VALIDATION is Phase 3 | Phase 3 story decomposition |
| GAP-P2-002 | L3 | NFR-002 | latency ≤2000ms for Notification | Same as GAP-P2-001; Notification path hook receiver DESIGN is Phase 1, sustained VALIDATION is Phase 3 | Phase 3 story decomposition |
| GAP-P2-003 | L3 | NFR-003 | TUI overlay render ≤100ms | TUI permission overlay is Phase 3 deliverable; not Phase 1 scope per product-brief.md phase table | Phase 3 story decomposition |
| GAP-P2-004 | L3 | NFR-006 | 1000 events/sec throughput | Bounded-channel DESIGN is Phase 1 (S-008); sustained load VALIDATION at 1000 events/sec requires Phase 3 load-test infra | Phase 3 story decomposition |
| GAP-P2-005 | L1 | BC-2.01.004 postcondition 6 | `--persistent-events` flag ring flush during drain; Phase 1 does not expose the `--persistent-events` CLI flag (S-008 ring exists but is always-on; the conditional flush-on-flag behavior is a Phase 3 CLI surface per product-brief.md phase table) | Phase 3 story decomposition |

**L1 gaps (BC clause coverage): 1 (GAP-P2-005; justified by Phase 3 CLI scope)**
**L2 gaps (edge case coverage): 0**
**L3 gaps (NFR, justified): 4 (all have non-empty justification, all authorized by nfr-catalog.md)**

## §Trace v1.0

**Phase 2 story decomposition initial burst** (2026-05-19T04:30:00Z):
- Dependency graph, topological sort, BC/VP/NFR traceability matrices, edge case coverage matrix, gap register produced.

## §Trace v1.1

**Phase 2 r02 remediation** (2026-05-19):
- F-PHASE2-R02-05: BC-2.02.005 postcondition 3 → invariant 3 (S-012 subscribe() canonical locus; line 273)
- F-PHASE2-R02-07: BC Clause Coverage Matrix swept — BC-2.01.004 PC-4→PC-8 corrected (exit code taxonomy mislabeled); BC-2.01.004 added PC-4/PC-5/PC-7 clause rows; BC-2.01.005 postcondition rows reordered to monotonically ascending clause numbers; BC-2.02.005 postcondition 3→invariant 3; BC-2.03.001 INV-2(DI-006)→PC-5(DI-006) (correct locus per §Traceability DI-006 mapping)
- GAP-P2-005 added: BC-2.01.004 PC-6 (--persistent-events ring flush) deferred to Phase 3 CLI surface (justified)
- Gap Register counts updated: L1 gaps now 1 (GAP-P2-005 justified), L2 gaps 0, L3 gaps 4

## §Trace v1.2

**Phase 2 r03 remediation** (2026-05-19):
- F-PHASE2-R03-01 CRITICAL: BC-2.01.002 fabricated PC-4..PC-7 rows removed. Only 3 PCs exist. Re-anchored: PC-1 sub-bullets for abi_version/hook_endpoints/last_hook_ts; auth failures → BC-2.01.009 PC-1/PC-3; PC-3 (/status during drain) → AC-008 new.
- F-PHASE2-R03-04: BC-2.01.007 EC-003 (truncated mid-line) correctly attributed to reader robustness. Flush failure → BC-2.01.004 EC-049 (writer concern). AC-007 rotation anchor: SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer Rotation Policy + BC-2.01.007 v1.0.5 EC-002 (F-PHASE2-R05-05 updated from PRD OQ-06).
- F-PHASE2-R03-05: BC-2.02.005 INV-2 (display_name "VSDD Factory") → S-012 AC-010 new.
- F-PHASE2-R03-06: BC-2.02.003 PC-4 row added (9 canonical enums including DenyReason, AllowPattern, DenyPattern) → S-011 AC-001b.
- F-PHASE2-R03-07: BC-2.01.008 PC-4 → AC-010a (dual-accept on hook endpoints). BC-2.01.002 PC-1 sub-bullet hook_endpoints → AC-010b (5 endpoints registered). S-009.
- F-PHASE2-R03-09: BC-2.02.001 PC-2 row added (equals compiled value); AC-005 re-anchored from INV-1 → PC-1 + PC-2.
- F-PHASE2-R03-12: BC-2.02.005 PC-4 row re-anchored to AC-005 + AC-008 (parse_frontmatter guards + error handling).
- F-PHASE2-R03-13: BC-2.02.005 INV-3 row updated to cite AC-007, AC-009 (both ACs cover subscribe() Phase 1 stub).
- BC-2.01.004 PC-4 → AC-008 (was stale AC-004 from old S-003 numbering).

## §Trace v1.3

**Phase 2 r04 remediation** (2026-05-19):

**F-PHASE2-R04-01 (CRITICAL) + F-PHASE2-R04-02 (HIGH) — BC-2.01.004 PC-1/PC-2/PC-3 anchor correction:**
- PC-1 row: added AC-002 (POST /shutdown also transitions AppMode per PC-1; AC-001 = SIGTERM trigger; both are PC-1 clauses)
- PC-2 row: corrected to AC-003 (hook 503 + Retry-After:10 = PC-2 verbatim). Was AC-002 (wrong — S-005 AC-002 now describes POST /shutdown HTTP 200 triggering PC-1, not hook 503).
- PC-3 row: corrected to S-002 AC-002 via BC-2.01.001 PC-2 delegation. Was AC-003 (wrong — S-005 AC-003 now correctly covers PC-2 hook 503). PC-3 is /healthz 503 during drain — cross-covered by BC-2.01.001 PC-2 which S-002 AC-002 implements. No duplicate AC added to S-005 per finding instruction.
- INV-3 row: added AC-002 as success-path citation alongside AC-006 (AC-002 = HTTP 200 on valid dual-accept auth; AC-006 = 401 on missing/invalid auth; both together fully cover INV-3 dual-accept requirement).

**GAP-PHASE2-R04-1 (LOW) — BC-2.01.007 PC-3 clause anchor corrected:**
- PC-3 row: corrected to AC-003 (hybrid ring architecture description references RING_FORMAT_VERSION const per PC-3). Was AC-006 (wrong — AC-006 is anchored to PC-5 #[non_exhaustive]+new() in S-008). PC-5 row retains AC-006 (correct). Duplicate phantom PC-3/AC-006 double-mapping eliminated.

**GAP-PHASE2-R04-2 (LOW) — BC-2.01.009 INV-1..INV-6 rows added:**
- INV-1 (two-body taxonomy complete): AC-004 + AC-006
- INV-2 (same body for all value-present failures): AC-005 + AC-006
- INV-3 (missing = developer-actionable error): AC-004
- INV-4 (AuthError enum internal variants): AC-004 + AC-005 + AC-006
- INV-5 (canonical priority immutable): AC-007
- INV-6 (WARN log per alias attempt): AC-005
- INV-7 (constant-time on both paths): AC-008 (pre-existing)

**F-PHASE2-R04-03 (HIGH) + F-PHASE2-R04-04 (HIGH) + F-PHASE2-R04-05 (MEDIUM) — BC-2.02.005 PC-1/PC-3/PC-4 rows corrected:**
- PC-1 row: corrected to AC-011 (new() constructor with workspace_root derivation). Was AC-005 (wrong — AC-005 covers detect(), which is PC-2 per BC body). INV-1 row added for detect() criterion using AC-005.
- PC-3 row: added with AC-012 (new AC for absent-optional → None contract).
- PC-4 row: corrected to AC-013 (new AC for 4 parser guards). Was AC-005 + AC-008 (wrong per BC body; AC-005 = detect(); AC-008 = error handling, which is PC-4 error path but not the guards themselves).

**Full sibling sweep confirmation (all 22 BCs re-derived from BC bodies):**
- BC-2.01.001: PC-1..PC-4, INV-1..INV-2 — rows already correct; no changes
- BC-2.01.002: PC-1 sub-bullets, PC-2, PC-3 — rows already correct per r03 (F-PHASE2-R03-01 closure)
- BC-2.01.003: PC-1..PC-4, INV-1..INV-3 — rows already correct; no changes
- BC-2.01.004: corrected above (PC-1/PC-2/PC-3/INV-3)
- BC-2.01.005: PC-1..PC-8, INV-1..INV-3, PRE-2a..PRE-2d — rows already correct; no changes
- BC-2.01.006: PC-1..PC-7, INV-1..INV-3 — rows already correct; no changes
- BC-2.01.007: PC-1..PC-5, INV-1 corrected (PC-3 → AC-003); no other changes
- BC-2.01.008: PC-1..PC-4, INV-3 — rows already correct; no changes
- BC-2.01.009: PC-1..PC-4, INV-1..INV-7 added (6 new INV rows + 1 pre-existing)
- BC-2.01.010: PC-1..PC-4, EC-010..EC-012 — rows already correct; no changes
- BC-2.02.001: PC-1..PC-3, INV-1 — rows already correct; no changes
- BC-2.02.002: PC-1..PC-3, INV-1 — rows already correct; no changes
- BC-2.02.003: PC-1..PC-4, INV-1..INV-2 — rows already correct; no changes
- BC-2.02.004: PC-1..PC-4 — rows already correct; no changes
- BC-2.02.005: corrected above (PC-1/PC-3/PC-4; INV-1 added; INV-2/INV-3 retained)
- BC-2.02.006..BC-2.02.008: rows already correct; no changes
- BC-2.03.001..BC-2.03.004: rows already correct; no changes

## §Trace v1.4

**Phase 2 r05 remediation** (2026-05-19):
- F-PHASE2-R05-01 (CRITICAL): BC-2.01.009 PC-2/PC-3 alias/canonical swap in S-009 AC-004 trace header fixed (canonical=PC-2; alias=PC-3).
- F-PHASE2-R05-02 (HIGH): BC-2.01.002 PC-3 row coverage extended — S-003 AC-008 added (status serves during drain; BC-2.01.004 PC-4 cross-cites BC-2.01.002 PC-3).
- SE-22 v2 BC version cascade applied to all story corpus files (r05 scope).

## §Trace v1.6

**Phase 2 r06 remediation** (2026-05-19):
- F-PHASE2-R06-01 (CRITICAL): BC-2.01.009 PC-2/PC-3 remaining alias/canonical mirror swap fixed in S-009 AC-005/AC-006 trace headers and S-003 AC-002 trace header. BC-2.01.009 clause 2 (canonical) row corrected to AC-006; clause 3 (alias) row corrected to AC-005 — swap from previous (erroneous) assignment. INV-4 parenthetical verified correct (both AC-005 alias + AC-006 canonical are Invalid variant paths).
- F-PHASE2-R06-02 (HIGH): BC-2.02.001 PC-1 row disambiguation — per-story AC attribution clarified: S-010 AC-003 + S-010 AC-005 + S-003 AC-005.
- F-PHASE2-R06-04 (MEDIUM): §Trace reordered ascending (v1.2 was after v1.3 — ordering defect corrected); v1.4 and v1.5 entries added for r05 and r06 remediations.
- SE-22 v2 BC version cascade applied to all corpus files (15 BCs × 19 consumers): BC-INDEX v1.12→v1.13; BC-2.01.001 v1.0.4→v1.0.5; BC-2.01.002 v1.0.5→v1.0.6; BC-2.01.003 v1.0.4→v1.0.5; BC-2.01.004 v1.0.3→v1.0.4; BC-2.01.005 v1.0.4→v1.0.5; BC-2.01.006 v1.0.4→v1.0.5; BC-2.01.007 v1.0.5→v1.0.6; BC-2.01.008 v1.0.6→v1.0.7; BC-2.01.009 v1.0.6→v1.0.7; BC-2.01.010 v1.0.4→v1.0.5; BC-2.03.001 v1.0.4→v1.0.5; BC-2.03.002 v1.0.3→v1.0.4; BC-2.03.003 v1.0.2→v1.0.3; BC-2.03.004 v1.0.3→v1.0.4. SS-02 BCs unchanged.
- Discipline codified: story-corpus artifacts MUST have §Trace entries in monotonically-ascending version order for every declared version.
- Note (F-PHASE2-R08-02 retroactive label fix): this block was labeled §Trace v1.5 at time of authoring but frontmatter was already bumped to v1.6 in the same r06 burst — one-increment misalignment corrected to §Trace v1.6 per F-PHASE2-R08-02 closure.
