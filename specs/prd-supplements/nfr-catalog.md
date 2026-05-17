---
document_type: prd-supplement-nfr-catalog
level: L3
version: "1.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T19:00:00Z
phase: 1a
inputs: [prd.md]
input-hash: "6787573"
traces_to: prd.md
---

# Non-Functional Requirements Catalog: Monocle Phase 1

> PRD supplement — extracted from PRD Section 4 (v1.26 restructure, previously inline in PRD v1.25 §4).
> Primary consumers: architect, performance-engineer, formal-verifier.
> Do NOT modify NFR IDs — append-only numbering policy applies.

## NFR Registry

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source |
|----|----------|-------------|--------|------------------|----------|-------------|
| NFR-001 | Latency | Hook ingestion end-to-end response time for `PreToolUse`, `Stop`, `SessionStart`, `UserPromptSubmit` | ≤300ms | Integration test with stopwatch between hook POST and response; Claude Code's upstream timeout ceiling per BC-HOOK-022 | P0 | N/A |
| NFR-002 | Latency | Hook ingestion end-to-end response time for `Notification` | ≤2000ms | Integration test; gene-source BC-HOOK-022 timeout ceiling | P0 | N/A |
| NFR-003 | Latency | Permission prompt overlay render after hook POST receipt | ≤100ms | Integration test with TUI client attached; measures from POST receipt to TUI event dispatch | P0 | N/A |
| NFR-004 | Security | Auth token entropy source | 32 bytes from `rand::rngs::OsRng` (not `thread_rng`) | Code review + unit test asserting `OsRng` usage; source-grep per VP-008 §Pre-conditions (`rand::rngs::OsRng is the entropy source (not thread_rng)`) and Mechanical property item 1 (lock file authToken matches `^[0-9a-f]{64}$`) | P0 | N/A |
| NFR-005 | Security | Hook body size limit (all POST endpoints) | 256 KiB (262,144 bytes); HTTP 413 on excess | Integration test: send 262,145-byte body, assert 413 response per VP-003 §Post-condition 1 (`POST 262,145-byte body to any of the 5 hook endpoints with valid auth → HTTP 413 with exact body {"error":"payload_too_large","limit_bytes":262144}`) | P0 | N/A |
| NFR-006 | Throughput | Bounded event bus with visible drop counter | No unbounded channel; drop counter renders in status bar; 1000 events/sec sustained without queue overflow | Integration test at 1000 events/sec asserting drop counter assertion | P0 | N/A |
| NFR-007 | Build | MSRV | Rust 1.86 (ratatui 0.30 floor) | CI matrix check; `rust-toolchain.toml` | P0 | N/A |
| NFR-008 | Build | Platform targets | macOS + Linux (darwin/linux × amd64/arm64) | GitHub Actions CI matrix | P0 | N/A |
| NFR-009 | Security | Lock file permissions | `0o600` (owner-only read/write) | Integration test: `stat` lock file after daemon start; assert mode is `0600` per VP-005 Post-condition 1 (lock-file `0o600` mode assertion) | P0 | N/A |
| NFR-010 | Correctness | Constant-time auth comparison | `constant_time_eq::constant_time_eq` used for token comparison | Code review; source-grep per VP-008 §Post-condition 5 (`constant_time_eq` source-grep against `monocle-runtime/src/auth.rs` ensuring no `==` on hex secret string appears outside `constant_time_eq`) | P0 | N/A |
| NFR-011 | Forward-compat | DTU clone fidelity | ≥0.95 against fixture corpus | DTU fidelity measurement procedure per dtu-assessment.md | P1 | N/A |
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
| NFR-001 | VP-001 (hook latency probe) |
| NFR-002 | VP-002 (notification latency probe) |
| NFR-003 | Phase 3 TUI verification pending — TUI overlay render latency probe is a Phase 3 deliverable (TUI plane is out of Phase 1 scope per brief §Out of Scope). FV to create VP-023+ in Phase 3 when TUI subsystem is implemented. |
| NFR-004 | VP-008 §Pre-conditions (OsRng source-grep) + Mechanical property 1 (token hex format) |
| NFR-005 | VP-003 §Post-condition 1 (body size limit integration test) |
| NFR-006 | VP-006 (bounded channel + drop counter probe) |
| NFR-007 | Phase 6 formal hardening pending — MSRV CI matrix check is a Phase 6 (formal hardening) deliverable verified by CI matrix configuration. No verification property file required for CI matrix checks; validation is the `rust-toolchain.toml` pin + GitHub Actions matrix config. FV to confirm VP scope in Phase 6 preflight. |
| NFR-008 | Phase 6 formal hardening pending — platform matrix check is a Phase 6 (formal hardening) deliverable verified by CI matrix configuration. No verification property file required for CI matrix checks; validation is the GitHub Actions `[darwin, linux] × [amd64, arm64]` matrix. FV to confirm VP scope in Phase 6 preflight. |
| NFR-009 | VP-005 Post-condition 1 (lock-file 0o600 mode assertion) |
| NFR-010 | VP-008 §Post-condition 5 (constant_time_eq source-grep) |
| NFR-011 | Phase 4 holdout evaluation pending — DTU clone fidelity measurement against fixture corpus is a Phase 4 deliverable (holdout-evaluator agent scope). FV to create VP-023+ in Phase 4 when DTU clone is operational per dtu-assessment.md §Evaluation Criteria. |
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
