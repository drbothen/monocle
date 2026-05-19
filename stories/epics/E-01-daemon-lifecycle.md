---
document_type: epic
epic_id: EPIC-01
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
subsystems: [SS-01]
capabilities: [CAP-001]
behavioral_contracts: [BC-2.01.001, BC-2.01.002, BC-2.01.003, BC-2.01.004, BC-2.01.005, BC-2.01.006, BC-2.01.007, BC-2.01.008, BC-2.01.009, BC-2.01.010]
verification_properties: [VP-001, VP-002, VP-003, VP-004, VP-005, VP-006, VP-007, VP-008, VP-009, VP-010]
---

# EPIC-01: Daemon Lifecycle

## Purpose

Implement the monocle daemon's core lifecycle: HTTP server startup and shutdown,
lock file management, JSONL ring buffer, auth token/header handling, crash recovery,
and all five hook endpoints. This epic delivers the complete SS-01 Daemon Lifecycle
subsystem and makes the daemon capable of receiving hook events from Claude Code.

## Success Criteria

- All 10 BC-2.01.NNN behavioral contracts pass their verification properties
- DTU clone (EPIC-DTU S-DTU-001) can POST to all 5 hook endpoints with correct auth
- Integration tests green on macOS + Linux (darwin/linux × amd64/arm64 via CI matrix)
- NFR-004 (OsRng), NFR-005 (256 KiB limit), NFR-009 (0o600 lock), NFR-010 (constant-time), NFR-012 (0o700 dir) all pass

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-001 | Cargo Workspace Init + CI/DevOps Setup | 5 | Wave 1 | — |
| S-002 | Healthz Endpoint | 3 | Wave 2 | S-001 |
| S-003 | Status Endpoint | 5 | Wave 2 | S-001, S-002 |
| S-004 | Body Size Limit | 2 | Wave 2 | S-001 |
| S-005 | Graceful Shutdown (10-Second Drain) | 5 | Wave 2 | S-001, S-002 |
| S-006 | Lock File Atomic Lifecycle | 8 | Wave 2 | S-001 |
| S-007 | Crash Recovery Checkpoint | 5 | Wave 3 | S-006 |
| S-008 | JSONL Ring Format Version | 5 | Wave 3 | S-006 |
| S-009 | Auth Token Wire Format + Header Validation | 8 | Wave 3 | S-001, S-004, S-006, S-008 |

## Architecture Scope

- Implementing module: `monocle-runtime` (daemon binary, HTTP server, ring buffer, lock file, auth)
- Architecture source: `architecture/SS-daemon-lifecycle.md` v1.0.32
- Architecture dependency: `architecture/SS-core-types-and-abi.md` v1.2.13 (HookEnvelope types)
