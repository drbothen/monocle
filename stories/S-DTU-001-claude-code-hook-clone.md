---
document_type: story
level: L4
story_id: S-DTU-001
epic_id: EPIC-DTU
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 3
wave: 1
tdd_mode: facade
priority: P0
depends_on: []
blocks: [S-009]
target_module: dtu-clones/claude-code-hooks-v1
subsystems: []
behavioral_contracts: []
verification_properties: []
dtu_required: true
dtu_fidelity: L3
dtu_service: claude-code-hook-protocol
# BC status: pending PO authorship — DTU BCs are authored during Phase 3 TDD story delivery
# (dtu-assessment.md §Clone Development Approach; behavioral contracts derived from BC-HOOK-001..BC-HOOK-041)
inputs:
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
input-hash: "[live-state]"
traces_to: "Implements DTU clone for Claude Code hook protocol (dtu-assessment.md §Clone Development Approach); addresses NFR-011 (DTU fidelity ≥0.95)."
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
hex token is read from the monocle daemon lock file `authToken` field.

### AC-003 (traces to dtu-assessment.md §Monocle-Canonical Fields — correct payload structure)
Each clone POST body contains the monocle-canonical fields from SS-core-types-and-abi.md
v1.2.13 §Non-Exhaustive Inner Structs:
- `PreToolUse`: `{session_id, pid, tool_name, tool_input}`
- `Notification`: `{session_id, pid, notification_type, tool_name, tool_input, message}`
- `Stop`: `{session_id, pid, stop_reason}`
- `SessionStart`: `{session_id, pid, cwd (EX-2), transcript_path (EX-2)}`
- `UserPromptSubmit`: `{session_id, pid, prompt (EX-2)}`

### AC-004 (traces to NFR-011 — ≥0.95 fidelity against fixture corpus)
DTU fidelity score ≥0.95 when clone payloads are compared against the fixture corpus at
`tests/fixtures/dtu/claude-code-hook-2x/` per dtu-assessment.md §DTU Fidelity Measurement Procedure.
CI gate: `dtu-validator` agent validates fidelity before Phase 4 holdout evaluation.

### AC-005 (traces to dtu-assessment.md §Docker Compose — Docker packaging)
The clone is packaged as a Docker container `dtu-claude-code-hooks-v1` at port 8765 (configurable).
`docker-compose.yml` in `dtu-clones/claude-code-hooks-v1/` starts the clone.

### AC-006 (traces to dtu-assessment.md §Environment Variable Overrides)
The clone reads `MONOCLE_HOOK_ENDPOINT_BASE` (default: derives from lock file port) and
`MONOCLE_NO_AUTOSTART=1` (prevents daemon auto-start in test env).

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~800 |
| dtu-assessment.md (endpoint matrix, fidelity procedure) | ~3,000 |
| SS-core-types-and-abi.md (monocle-canonical fields section) | ~1,500 |
| axum or FastAPI server scaffolding | ~800 |
| Fixture corpus structure reference | ~400 |
| Test file | ~600 |
| **Total estimate** | **~7,100** |

## Tasks

- [ ] Implement `dtu-claude-code-hooks-v1` HTTP server (Rust axum or Python FastAPI)
  - Decision: Rust axum to maintain single-language codebase; reuse workspace crates
- [ ] Implement all 5 hook POST handlers with monocle-canonical JSON payloads
- [ ] Implement lock file reader: read `authToken` field; use as `X-Claude-Code-Ide-Authorization` value
- [ ] Create fixture corpus at `tests/fixtures/dtu/claude-code-hook-2x/` with ≥5 scenarios per endpoint
- [ ] Create `dtu-clones/claude-code-hooks-v1/Dockerfile` for Docker packaging
- [ ] Create `dtu-clones/claude-code-hooks-v1/docker-compose.yml`
- [ ] Integration test: start clone → POST to all 5 endpoints → verify monocle daemon accepts (200)
- [ ] DTU fidelity test: compare clone payloads against fixture corpus → score ≥0.95

## Previous Story Intelligence

N/A — DTU stories have NO product story dependencies (Wave 1 priority).
This story runs in parallel with S-001. The only external dependency is that the lock file
schema is known (from dtu-assessment.md) before the implementation, which it is.

## Architecture Compliance Rules

From `specs/dtu-assessment.md` v1.7.5 §Clone Development Approach:
- Clone MUST send `X-Claude-Code-Ide-Authorization` (not `X-Monocle-Authorization`)
  to exercise the daemon's alias code path
- Clone payloads MUST use monocle-canonical fields — gene-source fields alone will
  fail deserialization at the daemon
- Fidelity threshold: ≥0.95 mean field-match score against fixture corpus

**Forbidden Dependencies:**
- DTU clone MUST NOT import production `monocle-runtime` library code
- DTU clone MUST NOT write to any file outside `dtu-clones/claude-code-hooks-v1/`

## Library & Framework Requirements

| Crate/Package | Version | Usage |
|---------------|---------|-------|
| axum | =0.8.9 | HTTP server for clone endpoints |
| tokio | =1.52 | Async runtime |
| serde_json | =1.0.149 | Request/response JSON |

## File Structure Requirements

Files to create:
- `dtu-clones/claude-code-hooks-v1/src/main.rs` — clone HTTP server
- `dtu-clones/claude-code-hooks-v1/src/handlers.rs` — 5 hook handlers
- `dtu-clones/claude-code-hooks-v1/src/lock_reader.rs` — lock file reader for auth token
- `dtu-clones/claude-code-hooks-v1/Dockerfile`
- `dtu-clones/claude-code-hooks-v1/docker-compose.yml`
- `tests/fixtures/dtu/claude-code-hook-2x/pre-tool-use/*.json` — fixture corpus
- `tests/fixtures/dtu/claude-code-hook-2x/notification/*.json`
- `tests/fixtures/dtu/claude-code-hook-2x/stop/*.json`
- `tests/fixtures/dtu/claude-code-hook-2x/session-start/*.json`
- `tests/fixtures/dtu/claude-code-hook-2x/prompt-submit/*.json`
