---
document_type: story
level: L4
story_id: S-DTU-001
epic_id: EPIC-DTU
version: "1.3"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-20T21:00:00Z
phase: 2
points: 3
wave: 1
tdd_mode: facade
priority: P0
depends_on: []
blocks: [S-009]
target_module: crates/monocle-test-harness
subsystems: []
behavioral_contracts: [BC-HOOK-001, BC-HOOK-002, BC-HOOK-003, BC-HOOK-004, BC-HOOK-005,
  BC-HOOK-006, BC-HOOK-007, BC-HOOK-008, BC-HOOK-009, BC-HOOK-010, BC-HOOK-011,
  BC-HOOK-012, BC-HOOK-013, BC-HOOK-014, BC-HOOK-015, BC-HOOK-016, BC-HOOK-017,
  BC-HOOK-018, BC-HOOK-019, BC-HOOK-020, BC-HOOK-021, BC-HOOK-022, BC-HOOK-023,
  BC-HOOK-024, BC-HOOK-025, BC-HOOK-026, BC-HOOK-027, BC-HOOK-028, BC-HOOK-029,
  BC-HOOK-030, BC-HOOK-031, BC-HOOK-032, BC-HOOK-033, BC-HOOK-034, BC-HOOK-035,
  BC-HOOK-036, BC-HOOK-037, BC-HOOK-038, BC-HOOK-039, BC-HOOK-040, BC-HOOK-041]
# NOTE: BC-HOOK-001..BC-HOOK-041 are any-context behavioral contracts for the hook protocol
# (hooks-r1/r2 ingest rounds, dtu-assessment.md line 121). These BCs are authored by
# product-owner in Phase 3 TDD story delivery. This frontmatter array uses the canonical
# range derived from dtu-assessment.md §Clone Development Approach (lines 149-151).
# Status: draft — awaiting PO authorship of BC-HOOK-NNN files before status=ready.
verification_properties: [VP-DTU-001]
# VP-DTU-001: Phase 4 deferral marker applied per nfr-catalog.md L149 (NFR-011 DTU
# fidelity ≥0.95 is Phase 4 holdout-evaluator scope; FV creates VP-023+ when DTU
# clone is operational in Phase 4). VP-DTU-001 is the pending canonical ID.
dtu_required: true
dtu_fidelity: L3
dtu_service: claude-code-hook-protocol
inputs:
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
  - {path: .factory/specs/architecture/adr/ADR-0005.md, version: "1.0.2"}
input-hash: "[live-state]"
traces_to: "Implements DTU clone for Claude Code hook protocol (dtu-assessment.md §Clone Development Approach §Packaging Decision lines 320-343); addresses NFR-011 (DTU fidelity ≥0.95)."
---

# S-DTU-001: Claude Code Hook Protocol DTU Clone (L3 Behavioral)

## Narrative

As a test infrastructure consumer, I want a behavioral L3 clone of the Claude Code 5-endpoint
hook protocol at fidelity ≥0.95, so that integration tests, holdout evaluation, and formal
hardening can POST realistic hook events to the monocle daemon without requiring a live
Claude Code instance.

## Acceptance Criteria

### AC-001 (traces to dtu-assessment.md §Endpoint Matrix — 5 endpoints implemented)
The clone implements all 5 hook endpoints per dtu-assessment.md v1.7.5 endpoint matrix:
- `POST /hooks/pre-tool-use`
- `POST /hooks/notification`
- `POST /hooks/stop`
- `POST /hooks/session-start`
- `POST /hooks/prompt-submit`

### AC-002 (traces to dtu-assessment.md §Auth Header — alias path replication)
The clone sends `X-Claude-Code-Ide-Authorization: <raw-64-hex>` (the alias header) on
all hook POSTs — matching real Claude Code hook script behavior (BC-HOOK-016). The raw
hex token is read from the monocle daemon lock file `authToken` field per
SS-daemon-lifecycle.md v1.0.33 §Start Sequence (JSON template lines 491-512).

### AC-003 (traces to dtu-assessment.md §Monocle-Canonical Fields — correct payload structure)
Each clone POST body contains the monocle-canonical fields from SS-core-types-and-abi.md
v1.2.13 §Non-Exhaustive Inner Structs:
- `PreToolUse`: `{session_id, pid, tool_name, tool_input}`
- `Notification`: `{session_id, pid, notification_type, tool_name, tool_input, message}`
- `Stop`: `{session_id, pid, stop_reason}`
- `SessionStart`: `{session_id, pid, cwd (EX-2), transcript_path (EX-2)}`
- `UserPromptSubmit`: `{session_id, pid, prompt (EX-2)}`

### AC-004 (traces to NFR-011 — ≥0.95 fidelity against 25-fixture corpus)
DTU fidelity score ≥0.95 when clone payloads are compared against the full 25-fixture corpus
at `tests/fixtures/dtu/claude-code-hook-2x/` (minimum: 5 fixtures per endpoint × 5 endpoints
= 25 total per dtu-assessment.md §DTU Fidelity Measurement Procedure line 206).

The 25 fixtures MUST include all named fixtures from dtu-assessment.md lines 173-203,
specifically including these semantic edge fixtures:
- `pre-tool-use/non-permission-dropped.json` — must NOT reach the wire (filter validation)
- `pre-tool-use/large-message-boundary.json` — 200 KiB message field boundary

CI gate: `cargo xtask dtu-fidelity` exits 0 per dtu-assessment.md §DTU Fidelity Measurement
Procedure §Tooling. CI workflow at `.github/workflows/dtu-fidelity.yml` triggers on PRs
touching `crates/monocle-ipc/**` or `crates/monocle-runtime/**`.

### AC-005 (traces to dtu-assessment.md §Packaging Decision lines 320-343 — Rust binary form)
The clone is implemented as a Rust binary compiled from `crates/monocle-test-harness/src/dtu/`
with binary name `dtu-claude-code-hooks-v1`:
- Source layout: `crates/monocle-test-harness/src/dtu/`
- Built artifact: `target/[debug|release]/dtu-claude-code-hooks-v1`
- `cargo build --bin dtu-claude-code-hooks-v1` succeeds on macOS + Linux (mirrors
  dtu-assessment.md line 372).

Docker packaging is explicitly a Phase 4 distribution-packaging option, out of scope for
Phase 1. No `Dockerfile` or `docker-compose.yml` is a Phase 1 deliverable.

### AC-006 (traces to dtu-assessment.md §Environment Variable Overrides)
The clone reads `MONOCLE_HOOK_ENDPOINT_BASE` (default: derives from lock file port) and
`MONOCLE_NO_AUTOSTART=1` (prevents daemon auto-start in test env).

## Downstream Consumer Surface (S-009)

S-009 (`monocle-hook-receiver-hardening`) invokes the DTU clone as its test driver.
S-009 implementers must use:
- **Binary entrypoint:** `dtu-claude-code-hooks-v1` (built via `cargo build --bin dtu-claude-code-hooks-v1`)
- **Environment variables:** `MONOCLE_HOOK_ENDPOINT_BASE=http://127.0.0.1:<port>` (derived from lock file port), `MONOCLE_NO_AUTOSTART=1`
- **Expected POST endpoints:** All 5 per AC-001 (`/hooks/pre-tool-use`, `/hooks/notification`, `/hooks/stop`, `/hooks/session-start`, `/hooks/prompt-submit`)
- **Auth header behavior:** Clone always sends `X-Claude-Code-Ide-Authorization: <token>` (alias header path), exercising the daemon's alias code path per BC-HOOK-016

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,000 |
| dtu-assessment.md (endpoint matrix, fidelity procedure, packaging decision) | ~3,200 |
| SS-core-types-and-abi.md (monocle-canonical fields section) | ~1,500 |
| SS-daemon-lifecycle.md v1.0.33 (lock file JSON template §Start Sequence) | ~600 |
| BC files (41 BCs: BC-HOOK-001..BC-HOOK-041) | ~4,100 |
| axum server scaffolding | ~800 |
| Fixture corpus structure reference | ~400 |
| Test file | ~600 |
| **Total estimate** | **~12,200** |

Well within 20% of 200k context window. No split required.

## Tasks

- [ ] Create `crates/monocle-test-harness/` workspace crate (if not already a workspace member)
- [ ] Implement `crates/monocle-test-harness/src/dtu/main.rs` — clone HTTP server (Rust axum)
  - Decision: Rust axum; single-language codebase; reuse workspace crates
- [ ] Implement `crates/monocle-test-harness/src/dtu/handlers.rs` — 5 hook POST handlers
  with monocle-canonical JSON payloads
- [ ] Implement `crates/monocle-test-harness/src/dtu/lock_reader.rs` — reads `authToken`
  field from daemon lock file per SS-daemon-lifecycle.md v1.0.33 §Start Sequence
  (JSON template lines 491-512); contract_version field checked first per BC-2.01.010
- [ ] Wire `dtu-claude-code-hooks-v1` binary target in `crates/monocle-test-harness/Cargo.toml`
- [ ] Create fixture corpus at `tests/fixtures/dtu/claude-code-hook-2x/` with exactly 25
  fixtures (5 per endpoint), including all named fixtures from dtu-assessment.md lines 173-203,
  specifically `non-permission-dropped.json` (must NOT reach wire) and
  `large-message-boundary.json` (200 KiB message field)
- [ ] Add `reqwest` and `serde` to `crates/monocle-test-harness/Cargo.toml` dependencies
  (reqwest = "=0.13.0" for HTTP POSTing to daemon; serde with derive for JSON)
- [ ] Create `.github/workflows/dtu-fidelity.yml` — CI workflow triggering on PRs touching
  `crates/monocle-ipc/**` or `crates/monocle-runtime/**`; runs `cargo xtask dtu-fidelity`
- [ ] Implement `cargo xtask dtu-fidelity` command (in `xtask/` crate or `Makefile.toml`):
  starts clone binary → POSTs to all 5 endpoints → scores against fixture corpus → exits 0 iff ≥0.95
- [ ] Integration test: start clone → POST to all 5 endpoints → verify monocle daemon accepts (200)
- [ ] DTU fidelity test: compare clone payloads against 25-fixture corpus → score ≥0.95 → `cargo xtask dtu-fidelity` exits 0

## Previous Story Intelligence

N/A — DTU stories have NO product story dependencies (Wave 1 priority).
This story runs in parallel with S-001. The only external dependency is that the lock file
schema is known (from dtu-assessment.md + SS-daemon-lifecycle.md) before the implementation,
which it is.

Note: `depends_on: []` and `blocks: [S-009]` — this Wave-1 story runs in parallel with
other Wave-1 stories. S-009 may NOT proceed until this story is complete.

## Architecture Compliance Rules

From `specs/dtu-assessment.md` v1.7.5 §Packaging Decision (lines 320-343):
- Clone MUST be a Rust binary in `crates/monocle-test-harness/src/dtu/`
- Binary name: `dtu-claude-code-hooks-v1`; source at `tests/dtu/dtu-claude-code-hooks-v1/`
- Docker packaging is Phase 4 distribution scope — NOT Phase 1 deliverable
- `cargo build --bin dtu-claude-code-hooks-v1` must succeed on macOS + Linux

From `specs/dtu-assessment.md` v1.7.5 §Clone Development Approach:
- Clone MUST send `X-Claude-Code-Ide-Authorization` (not `X-Monocle-Authorization`)
  to exercise the daemon's alias code path (BC-HOOK-016)
- Clone payloads MUST use monocle-canonical fields — gene-source fields alone will
  fail deserialization at the daemon (SS-core-types-and-abi.md v1.2.13)
- Fidelity threshold: ≥0.95 mean field-match score against 25-fixture corpus

From `specs/architecture/adr/ADR-0005.md` v1.0.2 (NFR-011 trace):
- DTU fidelity measurement procedure is referenced in ADR-0005

**Forbidden Dependencies:**
- DTU clone MUST NOT import production `monocle-runtime` library code
- DTU clone MUST NOT write to any file outside `crates/monocle-test-harness/`

## Library & Framework Requirements

| Crate/Package | Version | Usage |
|---------------|---------|-------|
| axum | =0.8.9 | HTTP server for clone endpoints |
| tokio | =1.52.0 | Async runtime (canonical triplet per SS-deps-pin-manifest.md v1.1.18) |
| serde_json | =1.0.149 | Request/response JSON |
| reqwest | =0.13.0 | HTTP client for POSTing to daemon during fidelity tests |
| serde | 1 (derive) | Derive macros for payload structs; `serde = { version = "1", features = ["derive"] }` |

## File Structure Requirements

Files to create:
- `crates/monocle-test-harness/src/dtu/main.rs` — clone HTTP server
- `crates/monocle-test-harness/src/dtu/handlers.rs` — 5 hook handlers
- `crates/monocle-test-harness/src/dtu/lock_reader.rs` — lock file reader for auth token
- `.github/workflows/dtu-fidelity.yml` — CI fidelity gate workflow
- `tests/fixtures/dtu/claude-code-hook-2x/pre-tool-use/*.json` — 5 fixtures incl. non-permission-dropped.json, large-message-boundary.json
- `tests/fixtures/dtu/claude-code-hook-2x/notification/*.json` — 5 fixtures
- `tests/fixtures/dtu/claude-code-hook-2x/stop/*.json` — 5 fixtures
- `tests/fixtures/dtu/claude-code-hook-2x/session-start/*.json` — 5 fixtures
- `tests/fixtures/dtu/claude-code-hook-2x/prompt-submit/*.json` — 5 fixtures

Files NOT to create (explicitly out of scope):
- `dtu-clones/claude-code-hooks-v1/Dockerfile` — Phase 4 distribution packaging only
- `dtu-clones/claude-code-hooks-v1/docker-compose.yml` — Phase 4 distribution packaging only

## §Trace

**v1.3** (2026-05-21) — Status flipped ready→done. PR #3 merged at cfeb1346 on develop. Closes F-WAVE1-003 residual sibling-sweep gap (sprint-state and STORY-INDEX were flipped in 69930c3 + 06c94fb but story-file frontmatter was missed).

**v1.2** (2026-05-20T21:00:00Z) — Phase 3 TDD PO dispatch: BC-HOOK-001..041 authored; status draft → ready.
- All 41 behavioral contracts in `behavioral_contracts:` array now have corresponding BC files at
  `.factory/specs/behavioral-contracts/ss-dtu/BC-HOOK-001.md` through `BC-HOOK-041.md`.
- Gene source: `any-context-lazyclaude/internal/core/config/hooks.go` hooks-r1/r2 ingest rounds.
- Status flipped: `draft` → `ready`. Prerequisite gate cleared: deliver-story workflow may proceed.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z > prior v1.1 2026-05-20T00:00:00Z.

**v1.1** (2026-05-20) — Phase 3.B Batch 1 spec-reviewer remediation (F-A-01..F-E-02 findings from cycle-001 Stage-1 review). Refs: drbothen/vsdd-factory#150.
- F-C-02 + F-D-01 CLOSED: AC-005 and File Structure rewritten to canonical Rust binary form per dtu-assessment.md §Packaging Decision (lines 320-343). Dockerfile + docker-compose.yml removed from Phase 1 deliverables. `cargo build --bin dtu-claude-code-hooks-v1` AC added. Docker explicitly marked Phase 4.
- F-C-01 + F-D-03 CLOSED: `cargo xtask dtu-fidelity` oracle added to AC-004. `.github/workflows/dtu-fidelity.yml` trigger condition specified.
- F-B-01 CLOSED: behavioral_contracts array populated with BC-HOOK-001..BC-HOOK-041 per dtu-assessment.md line 121. verification_properties: [VP-DTU-001] added.
- F-B-02 CLOSED: ADR-0005 v1.0.2 added to inputs.
- F-A-01 CLOSED: tokio pin updated to canonical triplet `=1.52.0` in Library table.
- F-A-02 + F-A-03 CLOSED: reqwest `=0.13.0` and serde (derive) added to Library table.
- F-C-04 + F-C-05 CLOSED: AC-004 tightened to require all 25 named fixtures including non-permission-dropped.json and large-message-boundary.json.
- F-D-04 CLOSED: lock_reader.rs task cites SS-daemon-lifecycle.md v1.0.33 §Start Sequence (JSON template lines 491-512).
- F-E-01 + F-E-02 CLOSED: Downstream Consumer Surface section added enumerating S-009 binary entrypoint, env vars, POST endpoints, and alias-header behavior.
- F-D-02 + F-D-05 CLOSED: "or Python FastAPI" removed from Tasks; Token Budget updated (FastAPI alternative removed).
- target_module updated to `crates/monocle-test-harness` (canonical path per §Packaging Decision).
