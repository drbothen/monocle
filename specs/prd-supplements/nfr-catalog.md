---
document_type: prd-supplement-nfr-catalog
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T12:30:00Z
phase: 1a
inputs: [prd.md]
input-hash: "[live-state]"
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
| NFR-004 | Security | Auth token entropy source | 32 bytes from `rand::rngs::OsRng` (not `thread_rng`) | Code review + unit test asserting `OsRng` usage; source-grep per VP-AUTH-001 §Pre-conditions (`rand::rngs::OsRng is the entropy source (not thread_rng)`) and Mechanical property item 1 (lock file authToken matches `^[0-9a-f]{64}$`) | P0 | N/A |
| NFR-005 | Security | Hook body size limit (all POST endpoints) | 256 KiB (262,144 bytes); HTTP 413 on excess | Integration test: send 262,145-byte body, assert 413 response per VP-DAEMON-003 §Post-condition 1 (`POST 262,145-byte body to any of the 5 hook endpoints with valid auth → HTTP 413 with exact body {"error":"payload_too_large","limit_bytes":262144}`) | P0 | N/A |
| NFR-006 | Throughput | Bounded event bus with visible drop counter | No unbounded channel; drop counter renders in status bar; 1000 events/sec sustained without queue overflow | Integration test at 1000 events/sec asserting drop counter assertion | P0 | N/A |
| NFR-007 | Build | MSRV | Rust 1.86 (ratatui 0.30 floor) | CI matrix check; `rust-toolchain.toml` | P0 | N/A |
| NFR-008 | Build | Platform targets | macOS + Linux (darwin/linux × amd64/arm64) | GitHub Actions CI matrix | P0 | N/A |
| NFR-009 | Security | Lock file permissions | `0o600` (owner-only read/write) | Integration test: `stat` lock file after daemon start; assert mode is `0600` per VP-DAEMON-005 Post-condition 1 (lock-file `0o600` mode assertion) | P0 | N/A |
| NFR-010 | Correctness | Constant-time auth comparison | `constant_time_eq::constant_time_eq` used for token comparison | Code review; source-grep per VP-AUTH-001 §Post-condition 5 (`constant_time_eq` source-grep against `monocle-runtime/src/auth.rs` ensuring no `==` on hex secret string appears outside `constant_time_eq`) | P0 | N/A |
| NFR-011 | Forward-compat | DTU clone fidelity | ≥0.95 against fixture corpus | DTU fidelity measurement procedure per dtu-assessment.md | P1 | N/A |
| NFR-012 | Security | Runtime directory permissions | `0o700` (owner-only access) on newly-created runtime_dir; defense-in-depth with NFR-009 lock-file `0o600` | Integration test: `stat` runtime_dir after daemon start; assert mode is `0700` per VP-DAEMON-005 Post-condition 9 / probe 5.e | P0 | N/A |

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
| NFR-001 | VP-DAEMON-001 (hook latency probe) |
| NFR-002 | VP-DAEMON-002 (notification latency probe) |
| NFR-003 | VP-TUI-001 (overlay render latency) |
| NFR-004 | VP-AUTH-001 §Pre-conditions (OsRng source-grep) + Mechanical property 1 (token hex format) |
| NFR-005 | VP-DAEMON-003 §Post-condition 1 (body size limit integration test) |
| NFR-006 | VP-DAEMON-006 (bounded channel + drop counter probe) |
| NFR-007 | VP-BUILD-001 (MSRV CI matrix check) |
| NFR-008 | VP-BUILD-002 (platform matrix check) |
| NFR-009 | VP-DAEMON-005 Post-condition 1 (lock-file 0o600 mode assertion) |
| NFR-010 | VP-AUTH-001 §Post-condition 5 (constant_time_eq source-grep) |
| NFR-011 | VP-DTU-001 (fidelity measurement against fixture corpus) |
| NFR-012 | VP-DAEMON-005 Post-condition 9 / probe 5.e (runtime_dir 0o700 mode assertion) |
