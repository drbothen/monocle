---
document_type: prd-supplement-nfr-catalog
level: L3
version: "1.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T04:30:00Z
phase: 1a
inputs: [prd.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]
input-hash: "d4c6ea4"
traces_to: prd.md
---

# Non-Functional Requirements Catalog: Monocle Phase 1

> PRD supplement — extracted from PRD Section 4 (v1.26 restructure, previously inline in PRD v1.25 §4).
> Primary consumers: architect, performance-engineer, formal-verifier.
> Do NOT modify NFR IDs — append-only numbering policy applies.

## NFR Registry

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source |
|----|----------|-------------|--------|------------------|----------|-------------|
| NFR-001 | Latency | Hook ingestion end-to-end response time for `PreToolUse`, `Stop`, `SessionStart`, `UserPromptSubmit` | ≤300ms | Integration test with stopwatch between hook POST and response; Claude Code's upstream timeout ceiling per BC-HOOK-022 (gene-source any-context-lazyclaude deep ingest) | P0 | N/A |
| NFR-002 | Latency | Hook ingestion end-to-end response time for `Notification` | ≤2000ms | Integration test; gene-source BC-HOOK-022 timeout ceiling (gene-source any-context-lazyclaude deep ingest) | P0 | N/A |
| NFR-003 | Latency | Permission prompt overlay render after hook POST receipt | ≤100ms | Integration test with TUI client attached; measures from POST receipt to TUI event dispatch | P0 | N/A |
| NFR-004 | Security | Auth token entropy source | 32 bytes from `rand::rngs::OsRng` (not `thread_rng`) | Code review + unit test asserting `OsRng` usage; source-grep per VP-008 §Pre-conditions (`rand::rngs::OsRng is the entropy source (not thread_rng)`) and Mechanical property item 1 (lock file authToken matches `^[0-9a-f]{64}$`) | P0 | N/A |
| NFR-005 | Security | Hook body size limit (all POST endpoints) | 256 KiB (262,144 bytes); HTTP 413 on excess | Integration test: send 262,145-byte body, assert 413 response per VP-003 §Post-condition 1 (`POST 262,145-byte body to any of the 5 hook endpoints with valid auth → HTTP 413 with exact body {"error":"payload_too_large","limit_bytes":262144}`) | P0 | N/A |
| NFR-006 | Throughput | Bounded event bus with visible drop counter | No unbounded channel; drop counter renders in status bar; 1000 events/sec sustained without queue overflow | Integration test at 1000 events/sec asserting drop counter assertion | P0 | N/A |
| NFR-007 | Build | MSRV | Rust 1.86 (ratatui 0.30 floor) | `rust-toolchain.toml` pin verified by `cargo check`; CI fails if toolchain pin is absent or incorrect; Phase 1 devops deliverable per product-brief.md line 162–163 | P0 | N/A |
| NFR-008 | Build | Platform targets | macOS + Linux (darwin/linux × amd64/arm64) | GitHub Actions CI matrix with `[darwin, linux] × [amd64, arm64]` matrix; Phase 1 devops deliverable per product-brief.md line 162–163 | P0 | N/A |
| NFR-009 | Security | Lock file permissions | `0o600` (owner-only read/write) | Integration test: `stat` lock file after daemon start; assert mode is `0600` per VP-005 Post-condition 1 (lock-file `0o600` mode assertion) | P0 | N/A |
| NFR-010 | Correctness | Constant-time auth comparison on ALL auth paths (canonical + alias) | `constant_time_eq::constant_time_eq` used for token comparison on both canonical (`X-Monocle-Authorization`) and alias (`X-Claude-Code-Ide-Authorization`) paths per ADR-0005 + BC-2.01.009 INV-7 | Code review; source-grep per VP-008 §Post-condition 5 (`constant_time_eq` source-grep against `monocle-runtime/src/auth.rs` ensuring no `==` on hex secret string appears outside `constant_time_eq`) AND VP-009 §"alias-path constant-time comparison" probe (alias path verifies constant_time_eq is also used on `X-Claude-Code-Ide-Authorization` token; FV 5D expanding VP-009 in this same burst) | P0 | N/A |
| NFR-011 | Forward-compat | DTU clone fidelity | ≥0.95 against fixture corpus | DTU fidelity measurement procedure per dtu-assessment.md §"DTU Fidelity Measurement Procedure"; Phase 1 requirement per product-brief.md §Success Criteria (DTU row, line 246) | P1 | N/A |
| NFR-012 | Security | Runtime directory permissions | `0o700` (owner-only access) on newly-created runtime_dir; defense-in-depth with NFR-009 lock-file `0o600` | Integration test: `stat` runtime_dir after daemon start; assert mode is `0700` per VP-005 Post-condition 9 / probe 5.e | P0 | N/A |

## NFR Categories

| Category | Description | Validation Agent |
|----------|-------------|-----------------|
| Latency | End-to-end response time ceilings | performance-engineer |
| Security | Auth entropy, constant-time ops, file permissions | security-reviewer |
| Build | MSRV, platform targets, CI matrix | devops-engineer |
| Throughput | Event bus capacity, drop counter semantics | performance-engineer |
| Correctness | Algorithmic correctness properties | formal-verifier |
| Forward-compat | DTU clone fidelity, ABI stability | holdout-evaluator |

## NFR-to-Module Mapping

| NFR ID | Affected Modules | Architectural Impact |
|--------|-----------------|---------------------|
| NFR-001 | `monocle-runtime` (hook receiver axum router) | Hook handler must return within 300ms wall-clock; no blocking I/O on hook path |
| NFR-002 | `monocle-runtime` (hook receiver; Notification handler) | Notification path may spawn background work; response must still return ≤2000ms |
| NFR-003 | `monocle-runtime` (TUI event dispatch); `monocle-core` (engine layer) | Bounded channel size must be tuned to deliver ≤100ms TUI latency at 1000 events/sec |
| NFR-004 | `monocle-runtime` (auth.rs) | `rand::rngs::OsRng` import required; `thread_rng` is statically forbidden |
| NFR-005 | `monocle-runtime` (axum router layer) | `DefaultBodyLimit::max(256 * 1024)` applied on authenticated router only |
| NFR-006 | `monocle-runtime` (event bus); TUI status bar | `mpsc::channel(N)` with bounded N; drop counter surfaced in status bar widget |
| NFR-007 | All workspace crates | `rust-toolchain.toml` pins Rust 1.86; ratatui 0.30 is MSRV floor reason |
| NFR-008 | CI workflow | GitHub Actions matrix: `[darwin, linux] × [amd64, arm64]` |
| NFR-009 | `monocle-runtime` (daemon start sequence) | Lock file created with `0o600` via `OpenOptions::mode(0o600)` |
| NFR-010 | `monocle-runtime` (auth.rs) | `constant_time_eq` crate pinned; no string `==` on secrets |
| NFR-011 | `monocle-proto`, `monocle-runtime` (DTU surface) | DTU clone hook handler fidelity ≥0.95 per dtu-assessment.md fixture corpus |
| NFR-012 | `monocle-runtime` (daemon start sequence) | runtime_dir created with `DirBuilder::new().mode(0o700)` |

## VP Probe Citations

> Each NFR must be verified by at least one VP probe. The following citations are the
> authoritative links between NFRs and the verification properties that test them.

| NFR ID | VP Probe(s) |
|--------|-------------|
| NFR-001 | Phase 3 integration test scope — hook ingestion end-to-end latency ≤300ms requires load-test infrastructure with stopwatch tooling not available in Phase 1. VP-001 is the Healthz Endpoint correctness probe; it does NOT measure hook POST latency. No Phase 1 VP covers this probe. The hook receiver implementation is a Phase 1 deliverable (product-brief.md §Success Criteria hook receiver hardening rows); the end-to-end latency VALIDATION test is a Phase 3 integration test deliverable. VP and test will be authored at Phase 3 entry per cycle-3 story decomposition. |
| NFR-002 | Phase 3 integration test scope — Notification hook end-to-end latency ≤2000ms requires load-test infrastructure with stopwatch tooling not available in Phase 1. VP-002 is the Status Endpoint correctness probe; it does NOT measure notification POST latency. No Phase 1 VP covers this probe. The hook receiver implementation is a Phase 1 deliverable; the sustained latency VALIDATION test is a Phase 3 integration test deliverable. VP and test will be authored at Phase 3 entry per cycle-3 story decomposition. |
| NFR-003 | Phase 3 verification — NFR validates a Phase 3-scoped behavior (TUI permission overlay render is a Phase 3 — Workflow Plane deliverable per product-brief.md §Phase 3 — Workflow Plane (roadmap); Phase 1 ships the daemon and hook ingestion layer but NOT the TUI planes). No Phase 1 VP covers TUI render latency. VP and test will be authored at Phase 3 entry per cycle-3 story decomposition. |
| NFR-004 | VP-008 §Pre-conditions (OsRng source-grep) + Mechanical property 1 (token hex format) |
| NFR-005 | VP-003 §Post-condition 1 (body size limit integration test) |
| NFR-006 | Phase 3 integration test scope — NFR validates bounded-channel throughput at 1000 events/sec sustained, which requires integration-level load testing infrastructure not available in Phase 1. VP-006 is Crash Recovery Checkpoint and does NOT cover throughput. No Phase 1 VP covers this probe. The bounded-channel and drop-counter DESIGN is a Phase 1 deliverable (per product-brief.md §Success Criteria "Drop counter active" row); the sustained load VALIDATION at 1000 events/sec is a Phase 3 integration test deliverable. VP and test will be authored at Phase 3 entry per cycle-3 story decomposition. |
| NFR-007 | Phase 1 devops deliverable — `rust-toolchain.toml` pin + `cargo check` in CI. Validation is CI-config artifact, not a VP file. devops-engineer creates the `rust-toolchain.toml` and GitHub Actions workflow as a Wave 1 Phase 1 story deliverable (per product-brief.md line 162–163: MSRV is a Phase 1 deliverable). No VP file is architecturally appropriate for a CI toolchain config artifact — validation gate is CI green on Wave 1 devops-engineer story delivery, not a FV-authored VP. |
| NFR-008 | Phase 1 devops deliverable — GitHub Actions CI matrix `[darwin, linux] × [amd64, arm64]`. Validation is CI-config artifact, not a VP file. devops-engineer creates the matrix config as a Wave 1 Phase 1 story deliverable (per product-brief.md line 162–163: CI matrix is a Phase 1 deliverable). No VP file is architecturally appropriate for a CI matrix config artifact — validation gate is CI green on Wave 1 devops-engineer story delivery, not a FV-authored VP. |
| NFR-009 | VP-005 Post-condition 1 (lock-file 0o600 mode assertion) |
| NFR-010 | VP-008 §Post-condition 5 (constant_time_eq source-grep on canonical path) AND VP-009 §"alias-path constant-time comparison" probe (alias path; FV 5D expanding VP-009) |
| NFR-011 | Phase 1 requirement — DTU clone `dtu-claude-code-hooks-v1` is a Phase 1 deliverable per product-brief.md §Success Criteria (DTU row, line 246) and dtu-assessment.md. DTU fidelity measurement procedure is defined in dtu-assessment.md §"DTU Fidelity Measurement Procedure". The DTU clone must exist and score ≥0.95 against fixture corpus as a Phase 1 gate. Holdout-evaluator verifies fidelity during Phase 4 evaluation; however, the clone itself and CI integration are Phase 1 deliverables per dtu-assessment.md §"Phase 1 Clone Build Effort". VP-NNN (TBD by FV at Phase 1 story decomposition entry) will be authored in Wave 1 when the DTU clone story is implemented — DTU clone is a Wave 1 priority per dtu-assessment.md §"Phase 1 Clone Build Effort"; FV must create the VP as a Wave 1 Phase 1 gate deliverable, not a post-decomposition optional action. |
| NFR-012 | VP-005 Post-condition 9 / probe 5.e (runtime_dir 0o700 mode assertion) |

---

## §Trace

### F-R105-2 + GAP-R44-1 PO closure — 2026-05-17T19:00:00Z

**Finding:** F-R105-2 + GAP-R44-1 HIGH — 11 stale VP IDs (old `VP-DOMAIN-NNN` form) and 4 phantom VP IDs in NFR catalog VP Probe Citations table and NFR Registry inline text.

**Bump:** v1.0 → v1.1.

**SE-17c — Before (stale VP ID occurrences):**

```
NFR Registry row inline citations (stale):
  NFR-004 Validation Method: VP-AUTH-001 §Pre-conditions
  NFR-005 Validation Method: VP-DAEMON-003 §Post-condition 1
  NFR-009 Validation Method: VP-DAEMON-005 Post-condition 1
  NFR-010 Validation Method: VP-AUTH-001 §Post-condition 5
  NFR-012 Validation Method: VP-DAEMON-005 Post-condition 9

VP Probe Citations table (stale + phantom):
  NFR-001: VP-DAEMON-001        [stale → VP-001]
  NFR-002: VP-DAEMON-002        [stale → VP-002]
  NFR-003: VP-TUI-001           [phantom]
  NFR-004: VP-AUTH-001          [stale → VP-008]
  NFR-005: VP-DAEMON-003        [stale → VP-003]
  NFR-006: VP-DAEMON-006        [stale → VP-006]
  NFR-007: VP-BUILD-001         [phantom]
  NFR-008: VP-BUILD-002         [phantom]
  NFR-009: VP-DAEMON-005        [stale → VP-005]
  NFR-010: VP-AUTH-001          [stale → VP-008]
  NFR-011: VP-DTU-001           [phantom]
  NFR-012: VP-DAEMON-005        [stale → VP-005]
```

**SE-17d — After (canonical VP IDs + phase-deferral markers):**

```
NFR Registry row inline citations (canonical):
  NFR-004 Validation Method: VP-008 §Pre-conditions
  NFR-005 Validation Method: VP-003 §Post-condition 1
  NFR-009 Validation Method: VP-005 Post-condition 1
  NFR-010 Validation Method: VP-008 §Post-condition 5
  NFR-012 Validation Method: VP-005 Post-condition 9

VP Probe Citations table (canonical + phase-deferral markers):
  NFR-001: VP-001               [canonical]
  NFR-002: VP-002               [canonical]
  NFR-003: Phase 3 TUI verification pending [phantom resolved — phase-deferral]
  NFR-004: VP-008               [canonical]
  NFR-005: VP-003               [canonical]
  NFR-006: VP-006               [canonical]
  NFR-007: Phase 6 formal hardening pending [phantom resolved — phase-deferral]
  NFR-008: Phase 6 formal hardening pending [phantom resolved — phase-deferral]
  NFR-009: VP-005               [canonical]
  NFR-010: VP-008               [canonical]
  NFR-011: Phase 4 holdout evaluation pending [phantom resolved — phase-deferral]
  NFR-012: VP-005               [canonical]
```

**Phantom VP disposition rationale:**

- `VP-TUI-001` (was NFR-003 overlay render latency): TUI plane is explicitly Phase 3 scope per product-brief.md §Out of Scope. No Phase 1 VP file can exist for a Phase 3 subsystem. Phase-deferral marker applied; FV creates VP-023+ when TUI subsystem is implemented in Phase 3.
- `VP-BUILD-001` (was NFR-007 MSRV CI matrix) and `VP-BUILD-002` (was NFR-008 platform matrix): Build-time CI matrix verification is not a verification property in the VSDD sense — it is a CI configuration artifact (`rust-toolchain.toml` + GitHub Actions matrix). No VP file is architecturally appropriate. These are Phase 6 (formal hardening) gate items per the pipeline. Phase-deferral markers applied; FV to confirm VP scope in Phase 6 preflight rather than creating spurious VP files now.
- `VP-DTU-001` (was NFR-011 DTU fidelity ≥0.95): DTU clone evaluation is Phase 4 holdout-evaluator scope per dtu-assessment.md §Evaluation Criteria and pipeline Phase 4 definition. Phase-deferral marker applied; FV creates VP-023+ when DTU clone is operational in Phase 4.

**Note for FV:** VP citation changes occurred in this document. Under `vp_index_is_vp_catalog_source_of_truth` policy, VP-INDEX.md and verification-architecture.md are unchanged (no new VPs were created). Phantom VPs were resolved via phase-deferral markers, not new VP files. FV should confirm in Phase 3/4/6 preflight that deferral targets remain appropriate.

**Stale ID → Canonical ID mapping (11 occurrences across 6 unique old IDs):**
| Old ID | Canonical ID | Occurrences | Location |
|--------|-------------|-------------|---------|
| VP-DAEMON-001 | VP-001 | 1 | VP Probe Citations NFR-001 |
| VP-DAEMON-002 | VP-002 | 1 | VP Probe Citations NFR-002 |
| VP-DAEMON-003 | VP-003 | 2 | NFR-005 Validation Method + VP Probe Citations NFR-005 |
| VP-DAEMON-005 | VP-005 | 4 | NFR-009 Validation + NFR-012 Validation + VP Probe Citations NFR-009 + NFR-012 |
| VP-DAEMON-006 | VP-006 | 1 | VP Probe Citations NFR-006 |
| VP-AUTH-001 | VP-008 | 4 | NFR-004 Validation + NFR-010 Validation + VP Probe Citations NFR-004 + NFR-010 |

---

### F-R106-15 PO closure — 2026-05-17T22:10:00Z

**Finding:** F-R106-15 MED — NFR-010 Validation Method cited VP-008 §Post-condition 5 only. Per BC-2.01.009 INV-7, constant-time comparison is required on **both** the canonical path and the alias path (ADR-0005). VP-009 (alias-path constant-time probe) must also be cited. FV 5D is expanding VP-009 in parallel to include this probe.

**Canonical source:** BC-2.01.009 INV-7 ("Constant-time comparison applies to BOTH canonical and alias paths"); ADR-0005 §Security Properties.

**SE-17c — Before (NFR-010 Validation Method + VP Probe Citations row):**

```
NFR Registry row NFR-010 Validation Method:
  "Code review; source-grep per VP-008 §Post-condition 5 (`constant_time_eq` source-grep against
   `monocle-runtime/src/auth.rs` ensuring no `==` on hex secret string appears outside `constant_time_eq`)"

VP Probe Citations NFR-010:
  "VP-008 §Post-condition 5 (constant_time_eq source-grep)"
```

**SE-17d — After (NFR-010 Validation Method + Requirement + VP Probe Citations row):**

```
NFR Registry row NFR-010 Requirement:
  "`constant_time_eq::constant_time_eq` used for token comparison on both canonical
   (`X-Monocle-Authorization`) and alias (`X-Claude-Code-Ide-Authorization`) paths per ADR-0005 + BC-2.01.009 INV-7"

NFR Registry row NFR-010 Validation Method:
  "Code review; source-grep per VP-008 §Post-condition 5 AND VP-009 §"alias-path constant-time comparison"
   probe (FV 5D expanding VP-009 in same burst)"

VP Probe Citations NFR-010:
  "VP-008 §Post-condition 5 (constant_time_eq source-grep on canonical path) AND
   VP-009 §"alias-path constant-time comparison" probe (alias path; FV 5D expanding VP-009)"
```

**Changes made:**
- NFR-010 Requirement column: added "on ALL auth paths (canonical + alias)" and ADR-0005 + INV-7 citation
- NFR-010 Validation Method: added VP-009 alias-path probe citation alongside existing VP-008 citation
- VP Probe Citations table NFR-010: updated to cite both VP-008 and VP-009 with path-specific descriptions
- Version bumped: v1.1 → v1.2; timestamp refreshed; ADR-0005 added to inputs

**Note for FV (5D):** VP-009 must include an alias-path constant-time comparison probe to fulfill this NFR-010 citation. The probe should source-grep `monocle-runtime/src/auth.rs` for the alias-path comparison branch and assert `constant_time_eq` is used there as well.

**Scope:** PO-only. No changes to VP-009, VP-008, VP-INDEX.md, or any BC file. NFR-010 update only.

---

### F-R107 Round 6B PO closure — 2026-05-17T23:00:00Z

**Findings resolved:** F-R107-1 CRITICAL, F-R107-6 HIGH, F-R107-7 HIGH, F-R107-11 MEDIUM.

**Bump:** v1.2 → v1.3.

**F-R107-1 CRITICAL — fabricated ADR-0005 path:**

SE-17f before/after (frontmatter `inputs:`):

**Before:** `inputs: [prd.md, architecture/adr/ADR-0005-dual-accept-auth-header.md]`
**After:** `inputs: [prd.md, architecture/adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md]`

**F-R107-6 HIGH — NFR-003/007/008/011 phase-deferrals lack human-approved anchor:**

Production-Grade Rule 3 applied: each phase-deferral now cites `product-brief.md §Out of Scope` as the human-approved deferral anchor. Wording pattern: "Phase X verification — NFR validates a Phase X-scoped behavior per product-brief.md §Out of Scope; validation VP/test will be authored at Phase X entry per cycle-X story decomposition." Self-referencing FV TODO markers replaced with concrete anchor citations.

SE-17c — Before (NFR-003 representative):
```
Phase 3 TUI verification pending — TUI overlay render latency probe is a Phase 3 deliverable
(TUI plane is out of Phase 1 scope per brief §Out of Scope). FV to create VP-023+ in Phase 3
when TUI subsystem is implemented.
```

SE-17d — After (NFR-003 representative):
```
Phase 3 verification — NFR validates a Phase 3-scoped behavior (TUI plane is explicitly out of
Phase 1 scope per product-brief.md §Out of Scope...); human-approved deferral per product-brief.md
§Out of Scope. VP and test will be authored at Phase 3 entry per cycle-3 story decomposition.
```

Same pattern applied to NFR-007 (Phase 6 CI matrix), NFR-008 (Phase 6 platform matrix), NFR-011 (Phase 4 DTU fidelity).

**F-R107-7 HIGH — NFR-006 cites VP-006 but VP-006 is Crash Recovery Checkpoint (not throughput):**

VP-006 confirmed as "Crash Recovery Checkpoint — JSON Write, Offer, Cleanup" per VP-INDEX.md §SS-01 row. This VP does NOT probe bounded-channel throughput. No Phase 1 VP covers 1000 events/sec sustained load testing. Decision: phase-defer NFR-006 to Phase 3 integration testing per the human-approved deferral pattern above.

SE-17f before/after (VP Probe Citations NFR-006):

**Before:** `VP-006 (bounded channel + drop counter probe)`
**After:** `Phase 3 verification — NFR validates bounded-channel throughput... human-approved deferral per product-brief.md §Out of Scope...`

**F-R107-11 MEDIUM — NFR-001/002 cite BC-HOOK-022 without gene-source qualifier:**

SE-17c — Before:
```
NFR-001 Validation Method: "...per BC-HOOK-022"
NFR-002 Validation Method: "...gene-source BC-HOOK-022 timeout ceiling"
```

SE-17d — After:
```
NFR-001 Validation Method: "...per BC-HOOK-022 (gene-source any-context-lazyclaude deep ingest)"
NFR-002 Validation Method: "...gene-source BC-HOOK-022 timeout ceiling (gene-source any-context-lazyclaude deep ingest)"
```

**Changes made:** frontmatter `inputs:` ADR path corrected; NFR-001/002 BC-HOOK-022 qualifier added; NFR-003/007/008/011 phase-deferral wording updated with human-approved anchor; NFR-006 VP-006 phantom citation replaced with proper phase-deferral; version bumped 1.2 → 1.3; timestamp refreshed.

---

### F-R108-3 PO closure — 2026-05-18T01:00:00Z

**Finding:** F-R108-3 CRITICAL — NFR-003/006/007/008/011 had fabricated or incorrect brief section anchors. The citations to `product-brief.md §Out of Scope` did not correspond to content that actually exists in that section.

**Pre-adjudicated decisions applied:**

**NFR-003 (TUI overlay ≤100ms):** Phase 3 deferral is legitimate — TUI permission overlay is a Phase 3 Workflow Plane deliverable per product-brief.md §Phase 3 — Workflow Plane (roadmap). However, the prior VP Probe Citations row falsely cited `§Out of Scope: "Does NOT include PM/Worker multi-agent orchestration"` — that Out of Scope item is about agent orchestration, not TUI planes. Corrected to cite the actual Phase 3 roadmap section.

**NFR-006 (throughput 1000 events/sec):** The bounded-channel DESIGN is a Phase 1 deliverable (product-brief.md §Success Criteria "Drop counter active" row). The sustained load VALIDATION test is appropriately a Phase 3 integration test. Prior anchor `§Out of Scope` was wrong — the brief §Out of Scope does not cover throughput load tests. Corrected to distinguish design (Phase 1) from validation test (Phase 3 integration).

**NFR-007 (MSRV Rust 1.86):** RESCOPED to Phase 1. Product-brief.md line 162–163 ("macOS + Linux build targets (darwin/linux × amd64/arm64); CI matrix on GitHub Actions; MSRV Rust 1.86") places CI/MSRV squarely in Phase 1. The previous Phase 6 deferral contradicted the brief. The `rust-toolchain.toml` pin is a Phase 1 devops deliverable. No VP file needed — validation is via CI toolchain config.

**NFR-008 (platform targets darwin/linux × amd64/arm64):** RESCOPED to Phase 1. Same rationale as NFR-007 — brief line 162 is unambiguous. The GitHub Actions CI matrix is a Phase 1 devops deliverable. No VP file needed — validation is via CI matrix config.

**NFR-011 (DTU fidelity ≥0.95):** RESCOPED to Phase 1. Product-brief.md §Success Criteria "DTU clone exists and validates" row (line 246) explicitly requires the DTU clone in Phase 1, contradicting the prior Phase 4 deferral. The DTU clone build and CI integration are Phase 1 deliverables per dtu-assessment.md §"Phase 1 Clone Build Effort". Holdout-evaluator conducts the formal evaluation in Phase 4, but the artifact (clone + fidelity ≥0.95) must exist by end of Phase 1.

**SE-17c — Before (VP Probe Citations rows, summary form):**

```
NFR-003: Phase 3 verification — ...human-approved deferral per product-brief.md §Out of Scope: "Does NOT include PM/Worker multi-agent orchestration"...
NFR-006: ...human-approved deferral per product-brief.md §Out of Scope (Phase 1 scope = hook receiver hardening + forward-compatibility contracts; sustained throughput load tests are Phase 3 integration deliverables)...
NFR-007: Phase 6 verification — ...human-approved deferral per product-brief.md §Out of Scope (Phase 1 scope = hook receiver hardening + forward-compatibility contracts; CI matrix gates are Phase 6 deliverables)...
NFR-008: Phase 6 verification — ...human-approved deferral per product-brief.md §Out of Scope (Phase 1 scope = hook receiver hardening + forward-compatibility contracts; platform matrix gates are Phase 6 deliverables)...
NFR-011: Phase 4 verification — ...human-approved deferral per product-brief.md §Out of Scope (Phase 1 scope = hook receiver hardening + forward-compatibility contracts; DTU clone fidelity measurement is a Phase 4 deliverable)...
```

**SE-17d — After (VP Probe Citations rows, summary form):**

```
NFR-003: Phase 3 verification — TUI permission overlay is Phase 3 Workflow Plane deliverable per product-brief.md §Phase 3 — Workflow Plane (roadmap). No Phase 1 VP for TUI render latency.
NFR-006: Phase 3 integration test scope — bounded-channel DESIGN is Phase 1; sustained 1000 events/sec VALIDATION is Phase 3 integration test per product-brief.md §Success Criteria "Drop counter active" row.
NFR-007: Phase 1 devops deliverable — rust-toolchain.toml + CI per product-brief.md line 162–163. No VP file needed.
NFR-008: Phase 1 devops deliverable — GitHub Actions CI matrix per product-brief.md line 162–163. No VP file needed.
NFR-011: Phase 1 requirement — DTU clone Phase 1 deliverable per product-brief.md §Success Criteria DTU row (line 246) and dtu-assessment.md. Holdout-evaluator verifies in Phase 4.
```

**NFR Registry rows updated:**
- NFR-007 Validation Method: updated from "CI matrix check; rust-toolchain.toml" to cite Phase 1 devops deliverable + brief anchor
- NFR-008 Validation Method: updated from "GitHub Actions CI matrix" to cite Phase 1 devops deliverable + brief anchor
- NFR-011 Validation Method: added brief §Success Criteria line 246 anchor

**Changes made:** VP Probe Citations rows for NFR-003/006/007/008/011 corrected; NFR Registry Validation Method column updated for NFR-007/008/011; version bumped v1.3 → v1.4; timestamp refreshed.

**Scope:** PO-only. No changes to VP files, BC files, ADR files, or any architecture artifact.

---

### F-R109 Round 8B PO closure — 2026-05-17T04:30:00Z

**Findings resolved:** F-R109-3 CRITICAL (NFR-001/002 phantom VP anchors), F-R109-11 HIGH (NFR-007/008/011 narrative anchor tightening), F-R109-12 HIGH (§Trace non-monotonic ordering).

**Bump:** v1.4 → v1.5.

**F-R109-3 CRITICAL — NFR-001/002 phantom VP anchor correction:**

NFR-001 cited `VP-001 (hook latency probe)` but VP-001 is the Healthz Endpoint correctness probe — it does NOT implement hook ingestion latency measurement. NFR-002 cited `VP-002 (notification latency probe)` but VP-002 is the Status Endpoint correctness probe — it does NOT implement notification latency measurement. Both citations were phantom anchors.

Decision: Phase-defer both to Phase 3 integration testing per the pattern established for NFR-006 (R107 Round 6B). Hook receiver implementation is a Phase 1 deliverable; latency VALIDATION requires stopwatch load-test infrastructure which is a Phase 3 integration deliverable.

SE-17f before/after (VP Probe Citations):
- NFR-001: `VP-001 (hook latency probe)` → Phase 3 integration test scope with concrete brief anchor
- NFR-002: `VP-002 (notification latency probe)` → Phase 3 integration test scope with concrete brief anchor

**F-R109-11 HIGH — NFR-007/008/011 anchor tightening (narrative → concrete):**

Round 7B introduced concrete phase anchors but VP Probe Citations rows for NFR-007/008 still ended with "VP scope confirmed by devops-engineer at Phase 1 story delivery" (passive future) and NFR-011 ended with "VP to be authored by FV at Phase 1 story decomposition when DTU clone stories are scheduled" (open-ended).

SE-17f tightening applied:
- NFR-007/008: `VP scope confirmed by devops-engineer at Phase 1 story delivery` → `Wave 1 Phase 1 story deliverable; validation gate is CI green on Wave 1 devops-engineer story delivery`
- NFR-011: `VP to be authored by FV at Phase 1 story decomposition when DTU clone stories are scheduled` → `VP-NNN (TBD by FV at Phase 1 story decomposition entry) will be authored in Wave 1 when the DTU clone story is implemented — DTU clone is a Wave 1 priority per dtu-assessment.md; FV must create the VP as a Wave 1 Phase 1 gate deliverable`

**F-R109-12 HIGH — §Trace non-monotonic ordering corrected:**

§Trace blocks were in non-monotonic order: F-R105-2 (T19:00), F-R107 (T23:00), F-R106-15 (T22:10), F-R108-3 (T01:00). The F-R106-15 block at T22:10 appeared AFTER F-R107 at T23:00 (a non-monotonic insertion — the block was authored in a different burst but inserted out of order).

SE-17f: Reordered to monotonic ascending: F-R105-2 (T19:00) → F-R106-15 (T22:10) → F-R107 (T23:00) → F-R108-3 (T01:00) → F-R109 (T04:30). Content of each section preserved verbatim; only insertion order corrected.

**Changes made:** VP Probe Citations NFR-001/002 rescoped to Phase 3; NFR-007/008 VP anchor tightened; NFR-011 VP anchor tightened; §Trace blocks reordered monotonic ascending; version bumped v1.4 → v1.5; timestamp refreshed.

**Scope:** PO-only. No changes to VP files, BC files, ADR files, or architecture artifacts.
