---
document_type: prd
level: L3
version: "1.26.7"
status: draft
producer: vsdd-factory:product-owner
phase: phase-1-spec-crystallization
timestamp: 2026-05-17T04:35:00Z
inputs: [product-brief.md, research/domain-monocle-vision-synthesis.md, architecture/SS-daemon-lifecycle.md, architecture/SS-core-types-and-abi.md, architecture/SS-engine-module.md, architecture/SS-deps-pin-manifest.md, architecture/SS-permissions-phase1.md, architecture/SS-conventions-anti-patterns.md, architecture/SS-forward-compatibility.md, dtu-assessment.md, architecture/adr/ADR-0001-wasmtime-vs-wasmi.md, architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md, architecture/adr/ADR-0003-license-selection.md, architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]
input-hash: "9ac590a"
traces_to: "product-brief.md v1.4.27; vision-synthesis v1.1.2; SS-daemon-lifecycle.md v1.0.32; SS-core-types-and-abi.md v1.2.13; SS-engine-module.md v1.1.20; SS-deps-pin-manifest.md v1.1.17; ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md; architecture/ARCH-INDEX.md; behavioral-contracts/BC-INDEX.md v1.7; 22 BCs sharded under behavioral-contracts/ss-NN/ (Dispatch 2 commit d02bf2a + Dispatch 3 commit f259ade); domain-spec/L2-INDEX.md v1.0.7"
project: monocle
supplements:
  - interface-definitions.md
  - error-taxonomy.md
  - test-vectors.md
  - nfr-catalog.md
---

# Product Requirements Document: Monocle — Phase 1 Forward-Compatibility Contracts

> **Index Document.** This PRD is an index. BC details live in `behavioral-contracts/ss-NN/BC-2.SS.NNN.md`.
> NFR catalog, error taxonomy, interface definitions, and test vectors are in `prd-supplements/`.
> Load supplements on-demand — do not load all 4 unless your task requires all 4.

## 1. Product Overview

### 1.1 Problem

Today, a developer running three Claude Code sessions across two projects faces a fragmentation problem: sessions live in separate tmux windows requiring context switches to check status; concurrent permission prompts from different sessions stall until the developer switches to the right window; factory-pipeline state (vsdd-factory STATE.md) is only visible by manually reading files; and no single view spans multiple harnesses.

Per vision §Vision Statement: "One TUI lens over every Claude-class session you're running, every customization that shapes them, and every workflow driving them — across multiple harnesses and federated across hosts."

### 1.2 Vision

Monocle is a single-binary Rust TUI that gives developers one `Ctrl-\` popup over every AI coding harness session they are running. It surfaces five information planes: live session roster (Runtime), active customizations per session (Static), workflow pipeline state (Workflow), per-harness profiles (Harness), and a lazygit-style keybinding dispatch layer (TUI philosophy). Monocle is observe-only for workflow state and session transcripts; it owns the action layer only for permission prompts and keybinding dispatch.

The killer scenario per vision §End-to-End Killer Scenario: 4 keystrokes (`Ctrl-\`, `2`, `1`, `Ctrl-\`) resolve two concurrent permission prompts with zero context switches vs. the current 6+ keystrokes + 2 window switches + risk of session timeout.

### 1.3 Competitive Differentiators

| ID | Differentiator | BC Backing |
|----|---------------|------------|
| D-1 | Hook-protocol ingestion at OS-assigned port with versioned auth token | BC-2.01.008, BC-2.01.009, BC-2.01.010, BC-2.01.001, BC-2.01.002 |
| D-2 | VecDeque overlay stack — both concurrent prompts visible simultaneously | BC-2.03.001, BC-2.03.002 |
| D-3 | Forward-compatible ABI via const + non_exhaustive + proto schema_version | BC-2.02.001, BC-2.02.002, BC-2.02.003, BC-2.02.006, BC-2.02.007, BC-2.02.008 |
| D-4 | FactoryAdapter open trait — VsddFactoryAdapter ships in Phase 1; WASM loadable in Phase 3 | BC-2.02.004, BC-2.02.005 |
| D-5 | ClaudeCodeModule strict-basename detect — no false positives from claude-squad/claudio | BC-2.03.002 |
| D-6 | JSONL ring with format_version first key — Phase 2 trigger-trace can read Phase 1 history | BC-2.01.007 |
| D-7 | 256 KiB body size limit with structured error — bounded daemon memory exposure | BC-2.01.003 |
| D-8 | Graceful 10-second drain with crash-recovery checkpoint | BC-2.01.004, BC-2.01.006 |

### 1.4 Target Users

| Persona | Pain | Phase |
|---------|------|-------|
| Multi-session Claude Code developer | Concurrent permission prompts stall sessions; no unified view | Phase 1 |
| Factory-pattern operator | STATE.md only readable via manual cat/tree; no live pipeline visibility | Phase 1 |
| Multi-harness operator (CodeMachine + Claude Code) | No unified cost/session-health view across harnesses | Phase 4 |

### 1.5 Out of Scope

Per vision §Explicit Non-Goals (hard boundaries):
- Does NOT execute workflows — monocle never writes STATE.md, never triggers factory phases
- Does NOT route LLM API requests — CCR integration is detect-on-PATH + config-write only
- Does NOT replace the terminal multiplexer — runs inside tmux, does not replace it
- Does NOT include PM/Worker multi-agent orchestration
- Does NOT own session transcripts — hook events are ephemeral ingestion signals
- Does NOT ship `PostToolUse` hook endpoint in Phase 1 — per JC-2 gene-source parity (any-context BC-HOOK-007 canonical 5-endpoint matrix)
- Does NOT ship WASM plugin SDK in Phase 1 — Phase 3 deliverable per OQ-03
- Does NOT ship rmcp MCP bridge in Phase 1 — Phase 4 deliverable per OQ-09

---

## 2. Behavioral Contracts Index

> Individual BC files live in `behavioral-contracts/ss-NN/` shard directories,
> one shard per subsystem registered in `architecture/ARCH-INDEX.md`.
> Grouped by L2 domain subsystem (CAP-NNN).
> Each BC uses hierarchical numbering: BC-S.SS.NNN where S=section (2 for all
> Phase 1 BCs), SS=subsection (matching L2 subsystem; matches the shard `ss-NN`
> directory), NNN=sequential within subsystem.
> Full index: `behavioral-contracts/BC-INDEX.md`.

### 2.1 Daemon Lifecycle (CAP-001)

> Architecture source: `architecture/SS-daemon-lifecycle.md` | ARCH-INDEX: SS-01

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.01.001 | Healthz Endpoint (Unauthenticated Liveness Probe) | P0 |
| BC-2.01.002 | Status Endpoint (Authenticated Daemon State) | P0 |
| BC-2.01.003 | Body Size Limit (256 KiB, HTTP 413) | P0 |
| BC-2.01.004 | Graceful Shutdown (10-Second Drain) | P0 |
| BC-2.01.005 | Lock File Atomic Lifecycle (Create + Pid Check + Cleanup) | P0 |
| BC-2.01.006 | Crash Recovery Checkpoint | P0 |
| BC-2.01.007 | JSONL Ring Format Version (FC-01) | P0 |
| BC-2.01.008 | Auth Token Wire Format (FC-06) | P0 |
| BC-2.01.009 | Auth Header Validation (Missing and Invalid Token) | P0 |
| BC-2.01.010 | Lock File Contract Version Field | P0 |

> Full contracts: `behavioral-contracts/ss-01/BC-2.01.NNN.md`

### 2.2 Core Types and ABI (CAP-002)

> Architecture source: `architecture/SS-core-types-and-abi.md` | ARCH-INDEX: SS-02

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.02.001 | ABI Version in /status Endpoint (FC-03) | P0 |
| BC-2.02.002 | ABI Version Constant at Crate Root (FC-03) | P0 |
| BC-2.02.003 | Non-Exhaustive Enum Policy (FC-02) | P0 |
| BC-2.02.004 | FactoryAdapter Trait Definition (FC-04 CRITICAL) | P0 |
| BC-2.02.005 | VsddFactoryAdapter Implementation | P0 |
| BC-2.02.006 | HookEnvelope Proto Field Number Contract (FC-05, wire-format) | P0 |
| BC-2.02.007 | HookEnvelope Rust Struct schema_version Field (FC-05, Rust surface) | P0 |
| BC-2.02.008 | Phase 4 schema_version Validation Requirement (FC-05) | P1 |

> Full contracts: `behavioral-contracts/ss-02/BC-2.02.NNN.md`

### 2.3 Engine Module (CAP-003)

> Architecture source: `architecture/SS-engine-module.md` | ARCH-INDEX: SS-03

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.03.001 | EngineModule Trait Definition | P0 |
| BC-2.03.002 | ClaudeCodeModule Implementation (Strict-Basename Detect) | P0 |
| BC-2.03.003 | HomeUnresolvable Error Contract | P0 |
| BC-2.03.004 | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | P0 |

> Full contracts: `behavioral-contracts/ss-03/BC-2.03.NNN.md`

---

## 3. Interface Definition

> **Supplement:** Full interface definitions are in `prd-supplements/interface-definitions.md`.
> Primary consumers: implementer, test-writer.

Phase 1 interface surfaces: HTTP API (5 hook POST endpoints + `/healthz` + `/status` + `/shutdown`), lock file JSON schema, JSONL ring buffer schema. Daemon binds on `127.0.0.1:<os-assigned-port>`. Auth header: canonical `X-Monocle-Authorization: monocle-v1:<64-hex>` (32 bytes `OsRng`); compatibility alias `X-Claude-Code-Ide-Authorization: <raw-64-hex>` accepted per ADR-0005 with WARN deprecation log. Body limit: 256 KiB on authenticated router only. See `prd-supplements/interface-definitions.md` for full schemas, exit codes, dual-accept semantics, and field constraints.

---

## 4. Non-Functional Requirements

> **Supplement:** Full NFR catalog is in `prd-supplements/nfr-catalog.md`.
> Primary consumers: architect, performance-engineer, formal-verifier.

Phase 1 defines 12 NFRs covering performance (NFR-001/002/003 latency, NFR-006 throughput), security (NFR-004 auth entropy, NFR-005 body limit, NFR-009 lock file 0o600, NFR-010 constant-time comparison, NFR-012 runtime_dir 0o700), build (NFR-007 MSRV Rust 1.86, NFR-008 macOS+Linux matrix), and forward-compat (NFR-011 DTU fidelity ≥0.95). See `prd-supplements/nfr-catalog.md` for the complete catalog including validation methods and VP probe citations.

---

## 5. Error Taxonomy

> **Supplement:** Full error taxonomy is in `prd-supplements/error-taxonomy.md`.
> Primary consumers: implementer, test-writer.

Phase 1 defines 15 error codes across 7 subsystem abbreviations (`DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`). Convention: `E-<SUBSYSTEM>-<NNN>`. Severity levels: Broken (fatal, non-zero exit or 4xx/5xx), Degraded (WARN log + graceful continue), Cosmetic (WARN log, zero exit, no functional impact; E-AUTH-003 alias deprecation log). See `prd-supplements/error-taxonomy.md` for the complete catalog including BC source citations, implementation sites, and test file mappings.

---

## 5b. Test Vectors

> **Supplement:** Canonical test vectors are in `prd-supplements/test-vectors.md`.
> Primary consumers: test-writer, holdout-evaluator.

Per-BC test vectors are embedded in each BC file's "Canonical Test Vectors" section. The supplement provides an index by BC ID with test file mapping, plus aggregated critical vectors for the highest-risk behavioral boundaries (auth rejection, body size limit, router separation, JSONL ring key ordering, detect basename). See `prd-supplements/test-vectors.md`.

---

## 6. Competitive Differentiator Traceability

Per vision §Vision Statement and brief §Success Criteria. Every differentiator has BC backing — no unverifiable claims.

> Project-specific extension: tables include a `Verification` column (beyond template minimum) documenting the specific test scenario that verifies the differentiator. Rationale: monocle's killer scenarios are described in the brief and vision; capturing them here prevents regression during adversarial review. See §Trace v1.26.1.

### 6.1 KD-001 — Hook-Protocol Ingestion at OS-Assigned Port

Daemon binds on OS-assigned port; port written to lock file; hook scripts read absolute lock file path (no directory scan, no "highest-port-wins" collision).

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.01.008 | Auth token generated with OsRng and written to lock file at start | Integration test: lock file read after start; port confirmed reachable |
| BC-2.01.009 | Auth header validation rejects requests missing correct Bearer token | Integration test: port confirmed reachable; unauthorized access rejected |
| BC-2.01.010 | Lock file schema contract version `"monocle-lock-v1"` encoded | Integration test: no `~/.claude/ide/` scanning; lock file path is absolute |
| BC-2.01.001 | `/healthz` endpoint returns liveness signal on OS-assigned port | Integration test: lock file read after start; healthz reachable at recorded port |
| BC-2.01.002 | `/status` endpoint authenticated on OS-assigned port | Integration test: port confirmed reachable with Bearer auth |

### 6.2 KD-002 — VecDeque Overlay Stack for Concurrent Prompts

Both permission prompts visible simultaneously; `[↑↓]` rotates stack; `Esc` hides without rejecting.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.03.001 | `EngineModule::on_hook()` returns `HookDecision::Defer` for queued hooks | Killer scenario: 2 concurrent PreToolUse hooks arrive; TUI shows both prompts; 4 keystrokes resolve both |
| BC-2.03.002 | `ClaudeCodeModule::detect()` strict-basename prevents false positives in concurrent session disambiguation | Killer scenario: `on_hook → HookDecision::Defer` path exercises VecDeque routing |

### 6.3 KD-003 — Versioned ABI with Forward-Compatible Extension

`MONOCLE_ABI_VERSION = 1` const; `#[non_exhaustive]` on all public enums; proto `schema_version = 1` first field.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.02.001 | `/status` endpoint exposes `abi_version` field equal to `MONOCLE_ABI_VERSION` | Integration: ABI version in status response matches const; compile-time assertion |
| BC-2.02.002 | `MONOCLE_ABI_VERSION = 1` const defined in `monocle-core` crate root | AST audit (syn 2); compile-time assertion |
| BC-2.02.003 | All public enums carry `#[non_exhaustive]` attribute | AST audit (syn 2) verifies enum annotation policy |
| BC-2.02.006 | `HookEnvelope` proto field numbers are pinned (field 1 = `schema_version`) | Wire-format round-trip test; prost encode/decode field number test |
| BC-2.02.007 | `schema_version = 1` is first field in serialized HookEnvelope | Compile/integration test: schema_version field accessibility |

### 6.4 KD-004 — FactoryAdapter Open Trait — Phase 3 WASM Extensibility

`VsddFactoryAdapter` ships Phase 1 as a static implementation; WASM plugin SDK in Phase 3 uses the same trait without code changes.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.02.004 | `FactoryAdapter` trait surface has no sealed supertrait; open for external implementation | `cargo check` no sealed supertrait; AST audit (syn 2) |
| BC-2.02.005 | `VsddFactoryAdapter` self-referential integration test confirms Phase 1 implementation | Self-referential detection test |

### 6.5 KD-005 — Strict-Basename Detection (No False Positives)

`detect()` uses `exe_path.file_name()` == `"claude"` or `"claude.js"`; rejects `claude-squad`, `claudio`, `claude-code-router`.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.03.002 | `ClaudeCodeModule::detect()` applies strict file_name() equality; rejects all non-exact basenames | Unit tests with 5 synthetic ProcessSnapshot instances (true positives: `claude`, `claude.js`; true negatives: `claude-squad`, `claudio`, `claude-code-router`) |

### 6.6 KD-006 — JSONL Ring with format_version First Key

Phase 2 trigger-trace can read Phase 1 history; version field allows future format evolution.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.01.007 | JSONL ring format version `format_version: 1` is first key in every serialized line | Unit test: serialized JSONL line begins with `{"format_version":1,` |

### 6.7 KD-007 — 256 KiB Body Size Limit with Structured Error

Bounded daemon memory exposure per connection; structured error body for machine-readable rejection.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.01.003 | Daemon rejects bodies > 262,144 bytes with HTTP 413 and structured JSON error body | Integration test: 262,145-byte body returns HTTP 413 with correct error body |

### 6.8 KD-008 — Graceful 10-Second Drain with Crash-Recovery Checkpoint

In-flight requests complete before daemon exits; crash-recovery state offered to TUI on reconnect.

| BC ID | Contribution | Verification |
|-------|-------------|-------------|
| BC-2.01.004 | SIGTERM triggers 10-second drain window; new hooks receive HTTP 503 with `Retry-After: 10` | Integration test: SIGTERM triggers drain; new hooks get 503 with Retry-After: 10 |
| BC-2.01.006 | Crash-recovery checkpoint written before shutdown; checkpoint offered to TUI on reconnect | Integration test: crash-recovery state offered to TUI on reconnect |

---

## 7. Requirements Traceability Matrix

> Project-specific extensions: `Source (L2 CAP)` contains brief section citations (monocle L2 domain spec is pending BA Dispatch 6; brief sections serve as interim L2 traceability). `Module(s)` contains architecture subsystem file references. `Test File` is an additional column beyond the template minimum, providing direct test location traceability. See §Trace v1.26.1.

| BC ID | Source (L2 CAP) | Module(s) | Priority | Test File | Test Type |
|-------|----------------|-----------|----------|-----------|-----------|
| BC-2.01.001 | §Scope (hook receiver hardening — `/healthz`) | SS-daemon-lifecycle.md v1.0.32 §GET /healthz | P0 | `monocle-runtime/tests/healthz_endpoint.rs` | Integration |
| BC-2.01.002 | §Scope (hook receiver hardening — `/status`) | SS-daemon-lifecycle.md v1.0.32 §GET /status | P0 | `monocle-runtime/tests/status_endpoint_auth.rs` | Integration |
| BC-2.01.003 | §Success Criteria (body size limit) | SS-daemon-lifecycle.md v1.0.32 §Body Size Limit | P0 | `monocle-runtime/tests/body_size_limit.rs` | Integration |
| BC-2.01.004 | §Scope (hook receiver hardening — graceful shutdown) | SS-daemon-lifecycle.md v1.0.32 §Shutdown Signal Handling | P0 | `monocle-runtime/tests/graceful_shutdown.rs` + `monocle-runtime/tests/daemon_lifecycle.rs` | Integration |
| BC-2.01.005 | §Scope (hook receiver hardening — graceful shutdown) | SS-daemon-lifecycle.md v1.0.32 §Start Sequence | P0 | `monocle-runtime/tests/lock_file_lifecycle.rs` | Integration |
| BC-2.01.006 | §Scope (hook receiver hardening — graceful shutdown) | SS-daemon-lifecycle.md v1.0.32 §Crash Recovery | P0 | `monocle-runtime/tests/crash_recovery.rs` | Integration |
| BC-2.01.007 | §Scope (forward-compatibility — JSONL ring) | SS-daemon-lifecycle.md v1.0.32 §Drain | P0 | `monocle-runtime/tests/jsonl_ring.rs` | Integration |
| BC-2.01.008 | §Scope (forward-compatibility — versioned auth token) | SS-daemon-lifecycle.md v1.0.32 §Start Sequence | P0 | `monocle-runtime/tests/auth_token_lifecycle.rs` | Integration |
| BC-2.01.009 | §Scope (forward-compatibility — versioned auth token) | SS-daemon-lifecycle.md v1.0.32 §Start Sequence | P0 | `monocle-runtime/tests/auth_header_rejection.rs` | Integration |
| BC-2.01.010 | §Scope (forward-compatibility — versioned auth token) | SS-daemon-lifecycle.md v1.0.32 §Start Sequence | P0 | `monocle-runtime/tests/lock_file_contract.rs` | Integration |
| BC-2.02.001 | §Scope (forward-compatibility — monocle-core ABI) | SS-core-types-and-abi.md v1.2.13 §ABI Version Constant | P0 | `monocle-runtime/tests/status_abi_version.rs` | Integration |
| BC-2.02.002 | §Scope (forward-compatibility — monocle-core ABI) | SS-core-types-and-abi.md v1.2.13 §ABI Version Constant | P0 | `monocle-core/tests/abi_stability.rs` | Lint/compile |
| BC-2.02.003 | §Scope (forward-compatibility — public enum extensibility) | SS-core-types-and-abi.md v1.2.13 §Enum Extensibility | P0 | `monocle-core/tests/enum_audit.rs` | AST audit (syn 2) |
| BC-2.02.004 | §Scope (forward-compatibility — FactoryAdapter trait) | SS-core-types-and-abi.md v1.2.13 §FactoryAdapter Trait | P0 | `monocle-core/tests/factory_trait_surface.rs` | AST audit (syn 2) |
| BC-2.02.005 | §Success Criteria (factory pattern detection) | SS-core-types-and-abi.md v1.2.13 §VsddFactoryAdapter | P0 | `monocle-core/tests/factory_self_referential.rs` | Integration |
| BC-2.02.006 | §Scope (forward-compatibility — prost wire schemas) | SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas | P0 | `monocle-proto/tests/wire_field_order.rs` | Integration |
| BC-2.02.007 | §Scope (forward-compatibility — prost wire schemas) | SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas | P0 | `monocle-proto/tests/schema_version.rs` | Integration |
| BC-2.02.008 | §Scope (forward-compatibility — prost wire schemas) | SS-core-types-and-abi.md v1.2.13 §Prost Wire Schemas | P1 | Phase 4 integration test (future) | Integration |
| BC-2.03.001 | §Scope §In Scope (ClaudeCodeModule) | SS-engine-module.md v1.1.20 §EngineModule Trait Signature | P0 | `monocle-core/tests/engine_module_surface.rs` | AST audit (syn 2) |
| BC-2.03.002 | §Scope §In Scope (ClaudeCodeModule) | SS-engine-module.md v1.1.20 §ClaudeCodeModule | P0 | `monocle-runtime/tests/engine_module_claude_detect.rs` | Integration |
| BC-2.03.003 | §Scope §In Scope (ClaudeCodeModule) | SS-engine-module.md v1.1.20 §BC-ENGINE-002-ERR | P0 | `monocle-runtime/tests/engine_module_home_unresolvable.rs` | Integration (env-isolation) |
| BC-2.03.004 | §Scope §In Scope (ClaudeCodeModule) | SS-engine-module.md v1.1.20 §Inherent operations | P0 | `monocle-runtime/tests/engine_module_claude_methods.rs` | Integration |
| NFR-012 | §Scope (daemon start — runtime_dir fallback chain; lock-file 0o600 + runtime_dir 0o700) | SS-daemon-lifecycle.md v1.0.32 §Start Sequence | P0 | `monocle-runtime/tests/daemon_lifecycle.rs` | Integration (VP-005 Post-condition 9 / probe 5.e) |

---

## §Trace v1.26 — Template Compliance Remediation (PRD restructure)

**Bump:** v1.25 → v1.26.
**Predecessor pin:** v1.25 commit a71ca67 (D-047 strict CONVERGENCE achieved on monolithic structure; subsequently determined to be structurally non-compliant per template-compliance-audit-r1).

**Scope of v1.26:**
- §3 (Full BC Specifications) DELETED; 22 BCs now live as sharded files in `behavioral-contracts/ss-NN/BC-2.SS.NNN.md` (created in Dispatch 2 commit d02bf2a + Dispatch 3 commit f259ade).
- §4 NFR catalog → `prd-supplements/nfr-catalog.md` + summary reference in PRD §4.
- §5 Error Taxonomy → `prd-supplements/error-taxonomy.md` + summary reference in PRD §5.
- New `prd-supplements/interface-definitions.md` + `prd-supplements/test-vectors.md` created per template.
- `supplements:` frontmatter field populated: `[interface-definitions.md, error-taxonomy.md, test-vectors.md, nfr-catalog.md]`.
- Section ordering aligned to prd-template.md: Overview → BC Index → Interface (ref) → NFR (ref) → Error Taxonomy (ref) → Test Vectors (ref) → Competitive Diff → RTM.
- §Trace history v1.0–v1.25 retired to git PG-5 (preserved at commit a71ca67); v1.26 starts fresh §Trace lineage post-restructure.
- BC IDs renumbered `BC-DAEMON-NNN → BC-2.01.NNN`, `BC-AUTH-NNN → BC-2.01.NNN`, `BC-LOCK-001 → BC-2.01.010`, `BC-RING-001 → BC-2.01.007`, `BC-ABI-NNN → BC-2.02.NNN`, `BC-TYPES-001 → BC-2.02.003`, `BC-FACTORY-NNN → BC-2.02.NNN`, `BC-PROTO-NNN → BC-2.02.NNN`, `BC-ENGINE-NNN → BC-2.03.NNN` per audit §661-714 renumbering map; old IDs preserved in BC-INDEX.md renumbering appendix (Old ID column) per append-only ID policy.
- Old §8 Cross-Cutting Concerns: content preserved in SS-conventions-anti-patterns.md (authoritative source); not replicated in PRD (PRD is an index document, not a conventions reference).
- Old §9 Edge Case Catalog: EC-001 through EC-061 live in individual BC files (EC content embedded per-BC). The PRD no longer maintains a cross-BC EC table (this was a monolith-era artifact; BC sharding makes per-BC EC the canonical location).
- Old §10 Glossary: preserved in full below.

**Audit reference:** `.factory/plans/template-compliance-audit-r1.md`.
**Dispatch:** Template-compliance remediation Dispatch 4 of 7+.
**Predecessors:** Dispatch 1 architect (ARCH-INDEX), Dispatch 2/3 PO (BC files + BC-INDEX).
**Next:** Dispatch 5 FV shards VP monolith with new BC IDs.

---

## §Trace v1.26.1 — Audit R2 Residual RES-05: §6/§7 Column Schema Reconciliation

**Bump:** v1.26 → v1.26.1.
**Predecessor pin:** v1.26 commit (template-compliance-audit-r1 remediation; §3 deleted, BC sharding, supplement extraction).

**Scope of v1.26.1 (patch — table schema only, no content added or removed):**

### §6 Changes

**From:** Single flat table with columns `Differentiator | Description | BC Backing | Verification`.

**To:** Per-differentiator subsections (`### 6.N KD-NNN — Name`) each containing `| BC ID | Contribution | Verification |` tables, matching prd-template.md §6 pattern.

**Project-specific extension retained:** `Verification` column (3rd column, beyond template's 2-column minimum). Rationale: monocle's killer scenarios are explicitly described in the vision document (v1.1.1) and product brief (v1.4.23). Capturing the verification scenario inline per differentiator prevents drift during adversarial review and ensures every claimed differentiator remains verifiable without cross-referencing the vision. This extension is additive (does not remove required template columns) and is self-documenting via the blockquote note at §6 head.

**Content changes:** None. All 8 differentiators preserved. All BC ID citations preserved. All descriptions preserved (moved into subsection introductory text). All verification notes preserved (moved into `Verification` column).

### §7 Changes

**From:** `| Requirement ID | Brief Section | Architecture Source | Priority | Test File | Test Type |` (6 columns; `Requirement ID` non-template name; `Brief Section` and `Architecture Source` non-template names).

**To:** `| BC ID | Source (L2 CAP) | Module(s) | Priority | Test File | Test Type |` (6 columns).

Column mapping:
- `Requirement ID` → `BC ID` (template column name; same data)
- `Brief Section` → `Source (L2 CAP)` (template column name; monocle's interim L2 traceability pending BA Dispatch 6 domain spec; brief sections are the authoritative source until L2 CAP IDs are assigned)
- `Architecture Source` → `Module(s)` (template column name; architecture file references preserved, shortened for readability)
- `Priority` → `Priority` (unchanged)
- `Test File` → `Test File` (project-specific extension, see below)
- `Test Type` → `Test Type` (template column name; unchanged)

**Project-specific extension retained:** `Test File` column (5th column, beyond template's 5-column schema). Rationale: direct test file path traceability is production-grade quality that reduces implementation ambiguity — implementers and test-writers have explicit file location targets. Extension is additive and self-documenting via the blockquote note at §7 head.

**Content changes:** None. All 22 BC rows + NFR-012 row preserved. All architecture source citations preserved (abbreviated in `Module(s)` column for readability while retaining version pin and subsection reference).

**Audit reference:** `.factory/plans/template-compliance-audit-r2.md` RES-05.
**Dispatch:** Audit R2 residual fix — concurrent with RES-02 (BC VP anchor sweep) and RES-03 (FV VP template compliance).
**Predecessors:** architect RES-01+RES-04 COMPLETE (0af206a).

---

## §Trace v1.26.2 — F-R105-7 Manifest Pin Refresh (v1.1.15 → v1.1.17)

**Bump:** v1.26.1 → v1.26.2.
**Predecessor pin:** v1.26.1 (Audit R2 residual §6/§7 column schema reconciliation; commit in factory-artifacts branch).

**Scope of v1.26.2 (patch — manifest pin only, no content added or removed):**

**Finding:** F-R105-7 MED — PRD `traces_to` frontmatter cited `SS-deps-pin-manifest.md v1.1.15`; architect confirmed delta v1.1.15 → v1.1.17 is structural only (pin-number swap, no content cascade required).

**SE-17c — Before (body-scope grep evidence):**
```
traces_to field: "...SS-deps-pin-manifest.md v1.1.15;..."
```

**SE-17d — After (body-scope grep evidence):**
```
traces_to field: "...SS-deps-pin-manifest.md v1.1.17;..."
```

**Manifest pin replacement count:** 1 occurrence (`traces_to` frontmatter field in prd.md).

**Note:** References to `SS-engine-module.md v1.1.20` in §7 RTM rows (BC-2.03.001 through BC-2.03.004) are the ENGINE MODULE version, NOT the deps-pin-manifest version. These are correct and unchanged.

**Concurrent:** nfr-catalog.md v1.0 → v1.1 (F-R105-2 + GAP-R44-1 VP ID sweep; same burst). interface-definitions.md v1.1 → v1.2 (F-R105-10/11 + GAP-R44-3 lock file schema; same burst).

---

## §Trace v1.26.3 — F-R105-12 + GAP-R44-4 (VP alias + abbreviation count)

**Bump:** v1.26.2 → v1.26.3.
**Predecessor pin:** v1.26.2 (F-R105-7 manifest pin refresh; commit 39082b0 on factory-artifacts).
**Timestamp:** 2026-05-17T19:30:00Z

**Scope of v1.26.3 (patch — two surgical corrections; no content added or removed):**

**Finding F-R105-12 LOW — §7 NFR-012 row stale VP alias:**

SE-17f before/after evidence:

**Before:** `Integration (VP-DAEMON-005 Post-condition 9 / probe 5.e)`
**After:** `Integration (VP-005 Post-condition 9 / probe 5.e)`

Rationale: `VP-DAEMON-005` is the legacy subsystem-scoped alias. VP-INDEX v1.1 §SS-01 table (line 110) maps `VP-DAEMON-005 → VP-005`. The canonical ID per VP-INDEX v1.1 (source of truth) is `VP-005` (title: "Lock File Lifecycle — Atomic Create, Pid Gate, Mode 0o600/0o700"). All VP cross-references must use the canonical VP-NNN form.

SE-17c — before (body-scope grep evidence):
```
§7 NFR-012 row Test Type column: "Integration (VP-DAEMON-005 Post-condition 9 / probe 5.e)"
```

SE-17d — after (body-scope grep evidence):
```
§7 NFR-012 row Test Type column: "Integration (VP-005 Post-condition 9 / probe 5.e)"
```

**Finding GAP-R44-4 LOW — §5a prose "6 subsystem abbreviations" count incorrect:**

SE-17f before/after evidence:

**Before:** `Phase 1 defines 14 error codes across 6 subsystem abbreviations (`DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`).`
**After:** `Phase 1 defines 14 error codes across 7 subsystem abbreviations (`DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`).`

Rationale: Actual enumeration contains 7 distinct abbreviations: `DAEMON`, `AUTH`, `LOCK`, `RING`, `FACT`, `ENG`, `PROTO`. Verified against `prd-supplements/error-taxonomy.md` §Error Catalog (14 rows: E-AUTH-001/002, E-DAEMON-001/002/003/004, E-LOCK-001/002/003, E-ENG-001, E-FACT-001/002, E-RING-001, E-PROTO-001). Count was 6, correct count is 7. The list itself was already correct — only the numeric count required correction.

SE-17c — before (body-scope grep evidence):
```
§5a line: "14 error codes across 6 subsystem abbreviations"
```

SE-17d — after (body-scope grep evidence):
```
§5a line: "14 error codes across 7 subsystem abbreviations"
```

**Concurrent:** Parallel FV dispatch sweeping 22 VP §References to cite PRD v1.26.3. Parallel architect dispatch adjudicating auth-header interop (not in PRD scope). Parallel BA dispatch fixing L2-INDEX anchors (not in PRD scope).

---

## §Trace v1.26.4 — F-R106-4 (RTM pin refresh + ADR-0005 input + E-AUTH-003 count)

**Bump:** v1.26.3 → v1.26.4.
**Predecessor pin:** v1.26.3 (VP alias + abbreviation count; commit on factory-artifacts).
**Timestamp:** 2026-05-17T22:20:00Z

**Scope of v1.26.4 (three-part patch: architecture version pin refresh, ADR-0005 traceability, error count update):**

**Finding F-R106-4 HIGH — §7 RTM + traces_to stale architecture pins:**

Pin replacement summary:

| Field | Before | After | Occurrence Count |
|-------|--------|-------|-----------------|
| `SS-daemon-lifecycle.md` (traces_to + §7 RTM) | v1.0.25 | v1.0.30 | 12 (1 traces_to + 11 RTM rows: BC-2.01.001–BC-2.01.010 + NFR-012) |
| `SS-core-types-and-abi.md` (traces_to + §7 RTM) | v1.2.8 | v1.2.11 | 9 (1 traces_to + 8 RTM rows: BC-2.02.001–BC-2.02.008) |
| `SS-engine-module.md` (traces_to + §7 RTM) | v1.1.15 | v1.1.18 | 5 (1 traces_to + 4 RTM rows: BC-2.03.001–BC-2.03.004) |

**Cross-dispatch coordination:** `SS-daemon-lifecycle.md v1.0.32` is the target version per architect 5E (F-FC-I005 removal + ADR-0005 auth-middleware section). v1.0.30 is the architect 5E commit target for the same burst. This PRD traces_to pins to v1.0.30 as coordinated.

SE-17f before/after evidence:

**Before (traces_to):** `SS-daemon-lifecycle.md v1.0.25; SS-core-types-and-abi.md v1.2.8; SS-engine-module.md v1.1.15`
**After (traces_to):** `SS-daemon-lifecycle.md v1.0.32; SS-core-types-and-abi.md v1.2.13; SS-engine-module.md v1.1.20`

SE-17c — before (§7 RTM rows — representative sample):
```
| BC-2.01.001 | ... | SS-daemon-lifecycle.md v1.0.25 §GET /healthz | ... |
| BC-2.02.001 | ... | SS-core-types-and-abi.md v1.2.8 §ABI Version Constant | ... |
| BC-2.03.001 | ... | SS-engine-module.md v1.1.15 §EngineModule Trait Signature | ... |
```

SE-17d — after (§7 RTM rows — representative sample):
```
| BC-2.01.001 | ... | SS-daemon-lifecycle.md v1.0.32 §GET /healthz | ... |
| BC-2.02.001 | ... | SS-core-types-and-abi.md v1.2.13 §ABI Version Constant | ... |
| BC-2.03.001 | ... | SS-engine-module.md v1.1.20 §EngineModule Trait Signature | ... |
```

**Finding GAP-R45-2 — ADR-0005 missing from inputs/traces_to:**

ADR-0005 (dual-accept auth header) is a canonical architecture decision that affects BC-2.01.008, BC-2.01.009, SS-daemon-lifecycle.md v1.0.32, and all 4 prd-supplements in this burst. It must appear in the PRD's inputs and traces_to fields.

SE-17f: Added `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md` to both `inputs:` array and `traces_to:` string.

**Error count update — E-AUTH-003 addition:**

error-taxonomy.md v1.1 (same burst) adds E-AUTH-003 (Cosmetic, WARN log, alias deprecation per BC-2.01.009 INV-6). Total error codes: 14 → 15.

SE-17f before/after:

**Before:** `Phase 1 defines 14 error codes across 7 subsystem abbreviations... Severity levels: Broken..., Degraded...`
**After:** `Phase 1 defines 15 error codes across 7 subsystem abbreviations... Severity levels: Broken..., Degraded..., Cosmetic (WARN log, zero exit, no functional impact; E-AUTH-003 alias deprecation log)`

**Concurrent:** Parallel PO 5A (BC scope), PO 5C (brief), FV 5D (VPs — VP-009 alias-path expansion), Architect 5E (ADR-0005 path + SS-daemon-lifecycle v1.0.30). All in same R106 Round 5 burst.

---

## §Trace v1.26.5 — F-R107 Round 6B (fabricated ADR path + traces_to refresh)

**Bump:** v1.26.4 → v1.26.5.
**Predecessor pin:** v1.26.4 (F-R106-4 RTM pin refresh + ADR-0005 input + E-AUTH-003 count; commit on factory-artifacts).
**Timestamp:** 2026-05-17T23:00:00Z

**Scope of v1.26.5 (three-part patch: ADR path correction, traces_to refresh, body §Trace correction):**

**Finding F-R107-1 CRITICAL — Fabricated ADR-0005 path in inputs/traces_to/body:**

SE-17f before/after evidence:

**Before (frontmatter `inputs:`):** `architecture/adr/ADR-0005-dual-accept-auth-header.md`
**After (frontmatter `inputs:`):** `architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md`

**Before (frontmatter `traces_to:`):** `...ADR-0005-dual-accept-auth-header.md;...`
**After (frontmatter `traces_to:`):** `...ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md;...`

**Before (body §Trace v1.26.4 SE-17f prose):** `Added \`architecture/adr/ADR-0005-dual-accept-auth-header.md\``
**After (body §Trace v1.26.4 SE-17f prose):** `Added \`architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md\``

Canonical filename verified via ARCH-INDEX and `ls .factory/specs/architecture/adr/`. All 3 occurrences in prd.md corrected.

**Finding F-R107-3 HIGH — traces_to stale pins (brief + BC-INDEX):**

SE-17f before/after evidence:

**Before:** `product-brief.md v1.4.23; ...; behavioral-contracts/BC-INDEX.md v1.1`
**After:** `product-brief.md v1.4.25; ...; behavioral-contracts/BC-INDEX.md v1.5`

Rationale: product-brief.md v1.4.25 and BC-INDEX.md v1.5 are the post-PO-6A Round 6 target versions per dispatch instructions.

**Concurrent:** Parallel PO 6A (BC scope), FV 6C (VPs), Architect 6D (SS-forward-compatibility), BA 6E (L2-INDEX). All in same R107 Round 6 burst.

---

## §Trace v1.26.6 — F-R108-7 + GAP-R47-3 Round 7B (traces_to arch pin refresh + L2-INDEX resolve)

**Bump:** v1.26.5 → v1.26.6.
**Predecessor pin:** v1.26.5 (F-R107 Round 6B fabricated ADR path + traces_to refresh; commit on factory-artifacts).
**Timestamp:** 2026-05-18T01:00:00Z

**Scope of v1.26.6 (two-part patch: arch version pin refresh + L2-INDEX placeholder removal):**

**Finding F-R108-7 HIGH — traces_to stale architecture pins (post-Architect-6D):**

Architect 7C (Round 7 parallel) normalizes timestamps on SS-daemon-lifecycle, SS-core-types-and-abi, and SS-engine-module without version bumps; however the prior PRD `traces_to` was stale from pre-Architect-6D commit 98396fe which bumped those files to the versions now confirmed canonical.

SE-17f before/after evidence:

**Before:** `...SS-daemon-lifecycle.md v1.0.32; SS-core-types-and-abi.md v1.2.13; SS-engine-module.md v1.1.20; ...BC-INDEX.md v1.5; ...`
**After:** `...SS-daemon-lifecycle.md v1.0.31; SS-core-types-and-abi.md v1.2.12; SS-engine-module.md v1.1.19; ...BC-INDEX.md v1.6; ...`

Also refreshed `product-brief.md v1.4.25 → v1.4.26` (brief bumped in this same Round 7B burst per F-R108-8).

**Finding GAP-R47-3 MEDIUM — traces_to "L2-INDEX.md (pending BA Dispatch 6)" placeholder:**

BA Dispatch 6 was completed at commit fcf2b2d producing L2-INDEX v1.0.7. The `(pending BA Dispatch 6)` annotation is stale.

SE-17f before/after evidence:

**Before:** `domain-spec/L2-INDEX.md (pending BA Dispatch 6)`
**After:** `domain-spec/L2-INDEX.md v1.0.7`

**Changes made:** frontmatter `traces_to:` — 5 version pins refreshed (brief, SS-daemon-lifecycle, SS-core-types-and-abi, SS-engine-module, BC-INDEX) + L2-INDEX placeholder resolved; version bumped v1.26.5 → v1.26.6; timestamp refreshed.

**Scope:** PO-only frontmatter patch. No body content changed. No BC, VP, or architecture file changes in this burst.

---

## §Trace v1.26.7 — F-R109 Round 8B (SS pin refresh + §Trace ascending + RTM pins + brief bump)

**Bump:** v1.26.6 → v1.26.7.
**Predecessor pin:** v1.26.6 (F-R108-7 + GAP-R47-3 traces_to arch pin refresh; commit on factory-artifacts).
**Timestamp:** 2026-05-17T04:35:00Z

**Scope of v1.26.7 (three-part patch: RTM SS pin refresh, traces_to update, §Trace ascending reorder):**

**Finding F-R109-5 HIGH — PRD body RTM SS pins stale:**

Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32, SS-core-types-and-abi.md v1.2.11→v1.2.13 (actually from v1.2.8 stale per §7), SS-engine-module.md v1.1.18 → v1.1.20 (actually from v1.1.15 stale). PRD §7 RTM rows and traces_to refreshed.

Pin replacement summary:

| Field | Before | After | Occurrence Count |
|-------|--------|-------|-----------------|
| `SS-daemon-lifecycle.md` (traces_to + §7 RTM) | v1.0.30 (body) / v1.0.31 (traces_to) | v1.0.32 | 12 body + 1 traces_to |
| `SS-core-types-and-abi.md` (traces_to + §7 RTM) | v1.2.11 (body) / v1.2.12 (traces_to) | v1.2.13 | 9 body + 1 traces_to |
| `SS-engine-module.md` (traces_to + §7 RTM) | v1.1.18 (body) / v1.1.19 (traces_to) | v1.1.20 | 5 body + 1 traces_to |
| `product-brief.md` (traces_to) | v1.4.26 | v1.4.27 | 1 traces_to |
| `BC-INDEX.md` (traces_to) | v1.6 | v1.7 | 1 traces_to |

**Finding F-R109-9 HIGH — §Trace blocks descending → ascending:**

§Trace blocks were descending (v1.26.6, v1.26.5, ..., v1.26). Reordered to ascending (v1.26 → v1.26.6 → v1.26.7). Content of each section preserved verbatim; only insertion order corrected.

**Changes made:** §7 RTM SS pins refreshed (3 subsystem docs × 11+8+4 rows); traces_to frontmatter refreshed (5 pins); §Trace blocks reordered ascending; version bumped v1.26.6 → v1.26.7; timestamp refreshed.

**Scope:** PO-only. No BC, VP, or architecture file changes in this burst. Concurrent with Architect 8A (SS doc bumps) and FV 8C.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| ABI | Application Binary Interface. `MONOCLE_ABI_VERSION` identifies the stable contract between `monocle-core` and its consumers (plugin SDK, federation layer). | SS-core-types-and-abi.md §ABI Version Constant |
| BC | Behavioral Contract. A testable specification with preconditions, postconditions, and at least one canonical test vector. | This document |
| `ClaudeCodeModule` | Phase 1 built-in `EngineModule` implementation for Claude Code harness integration. Defined in `monocle-runtime`. | SS-engine-module.md §Phase 1 Implementation: ClaudeCodeModule |
| DTU | Digital Twin Universe. Behavioral clone of the Claude Code hook protocol for testing fidelity and regression detection. | dtu-assessment.md |
| `DaemonStartError::RuntimeDirUnresolvable` | The `DaemonStartError` variant raised when BC-2.01.005 Precondition 2(d) fail-fast triggers (`MONOCLE_RUNTIME_DIR` is unset/empty AND `ProjectDirs::new()` returned `None`). Maps to error code E-DAEMON-004 (exit code 1). | BC-2.01.005 Precondition 2(d); prd-supplements/error-taxonomy.md E-DAEMON-004 |
| `EngineModule` | Trait in `monocle-core::engine` abstracting over AI coding harness adapters. Open (not sealed). | SS-engine-module.md §EngineModule Trait Signature |
| `FactoryAdapter` | Trait in `monocle-core::factory` abstracting over factory-pattern workflow detectors. Open (not sealed). | SS-core-types-and-abi.md §FactoryAdapter Trait |
| `FactoryState` | 7-field canonical struct returned by `FactoryAdapter::read_state()`. Fields: `phase`, `status`, `awaiting`, `blocking_issues`, `convergence`, `cycle`, `custom_fields`. | SS-core-types-and-abi.md §FactoryAdapter Trait |
| FC | Forward-Compatibility item. Pre-Phase-1 contracts locked by human authorization. FC-01 through FC-06. | SS-forward-compatibility.md; product-brief.md §Scope (forward-compatibility contracts sub-bullet) |
| `format_version` | First key in every JSONL ring buffer record. Value `1` for all Phase 1 records. | BC-2.01.007; SS-daemon-lifecycle.md §Drain |
| `HookEventRecord` | Rust struct in `monocle-runtime::ring` written to the JSONL ring buffer. `#[non_exhaustive]`; provides `new()` constructor. | SS-daemon-lifecycle.md §Drain |
| `HookEnvelope` | Proto message in `monocle-proto` with `schema_version` at field number 1. Wire format for Phase 4 federation. | BC-2.02.006, BC-2.02.007; SS-core-types-and-abi.md §Prost Wire Schemas |
| JC-2 | Joint Closure 2: `PostToolUse` omitted from Phase 1 hook endpoint set to preserve gene-source parity with any-context-lazyclaude BC-HOOK-007 canonical 5-endpoint matrix. | vision §Closure Log; brief §Scope |
| `monocle-v1:` | Wire-format prefix for Phase 1 auth tokens. `X-Monocle-Authorization: monocle-v1:<64-hex>`. | BC-2.01.008, BC-2.01.009 |
| `MONOCLE_ABI_VERSION` | `pub const u32 = 1` in `monocle-core::abi`. Exported at crate root. Used by Phase 3 plugin SDK and Phase 4 federation. | BC-2.02.001, BC-2.02.002 |
| `MONOCLE_RUNTIME_DIR` | Environment variable that overrides the runtime directory resolution chain. Per BC-2.01.005 Precondition 2(a), if set and non-empty, this path is used verbatim as the runtime directory. Empty string treated as unset (EC-060 in BC-2.01.005). | BC-2.01.005 Precondition 2(a); prd-supplements/error-taxonomy.md E-DAEMON-004 |
| `#[non_exhaustive]` | Rust attribute preventing exhaustive match and struct literal construction outside the defining crate. Default for all `pub` enums in `monocle-core`. | BC-2.02.003; ADR-0004 |
| OsRng | `rand::rngs::OsRng`. Cryptographically secure random source used for auth token generation. Required; `thread_rng` is forbidden for secrets. | BC-2.01.008; SS-daemon-lifecycle.md §Daemon Lifecycle Protocol §Start Sequence |
| `Phase1Permission` | Exhaustive enum in `monocle-core::permissions`. Five variants. ADR-0004 exempts it from `#[non_exhaustive]`. | ADR-0004; SS-permissions-phase1.md |
| `schema_version` | Proto field number 1 in `HookEnvelope`. Value `1` for all Phase 1 messages. Used by Phase 4 federation to validate message format compatibility. | BC-2.02.006, BC-2.02.007, BC-2.02.008 |
| `VsddFactoryAdapter` | Phase 1 static implementation of `FactoryAdapter`. Detects VSDD Factory workspaces via `document_type: pipeline-state` in `.factory/STATE.md`. | BC-2.02.005 |
