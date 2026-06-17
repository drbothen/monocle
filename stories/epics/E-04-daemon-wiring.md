---
document_type: epic
epic_id: EPIC-04
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
subsystems: [SS-04]
capabilities: [CAP-004]
behavioral_contracts: [BC-2.04.001, BC-2.04.002, BC-2.04.003, BC-2.04.004, BC-2.04.005, BC-2.04.006, BC-2.04.007, BC-2.04.008, BC-2.04.009, BC-2.04.010, BC-2.04.011, BC-2.04.012]
verification_properties: []
---

# EPIC-04: Daemon Wiring

## Purpose

Implement the monocle daemon binary, CLI subcommands (`daemon start` / `daemon stop`),
daemon startup sequence (port bind, lock file, auth token write — SOQ-2), hook endpoint
routing, bounded event bus with drop counter, JSONL ring rotation, auto-start on TUI
launch, and all BC-2.04.NNN behavioral contracts. This epic delivers the complete SS-04
Daemon Wiring subsystem, wiring together the Phase 1 daemon into a single running process
from CLI entry point through hook event ingestion.

## Success Criteria

- All 12 BC-2.04.NNN behavioral contracts pass their verification properties
- `monocle daemon start` and `monocle daemon stop` CLI subcommands work end-to-end
- Daemon startup sequence completes: port bind + lock file write + auth token write + hook tmpfile
- All 5 hook endpoints (PreToolUse, PostToolUse, Stop, Notification, UserPromptSubmit) route correctly
- Bounded event bus drops under load with drop counter surfaced in status bar
- JSONL ring rotates at capacity with correct format_version key ordering
- Daemon auto-starts on TUI launch unless `MONOCLE_NO_AUTOSTART=1` is set
- `cargo clippy --workspace --all-targets -- -D warnings` → 0 warnings

## Stories

| Story ID | Title | Points | Wave | Depends On |
|----------|-------|--------|------|-----------|
| S-016 | Daemon Binary Crate Init + CLI Subcommands | 5 | Wave 4 | S-001, S-006 |
| S-017 | Daemon Start Sequence (SOQ-2) + Hook Tmpfile Generation | 8 | Wave 5 | S-016, S-006, S-008, S-009, S-015, S-012 |
| S-018 | Hook Endpoint Routing + Bounded Event Bus with Drop Counter | 8 | Wave 5 | S-017, S-002, S-003, S-004, S-009, S-014 |
| S-019 | Daemon Auto-Start on TUI Launch + MONOCLE_NO_AUTOSTART | 5 | Wave 5 | S-016, S-017 |
| S-020 | JSONL Ring Capacity and Rotation Policy | 5 | Wave 5 | S-008, S-017 |
| S-DAEMON-WIRE-FIX-001 | Second-Signal Exit Codes (SigtermDuringDrain=143, SigintDuringDrain=130) | 5 | Wave 8 | S-005, S-016, S-017, S-018 |

**Total: 36 points (31 pts Waves 4-5 + 5 pts Wave 8 deferred fix)**

## Architecture Scope

- Implementing modules: `monocle-runtime` (daemon binary, HTTP server, event bus, ring buffer, hook routing)
- Architecture source: `architecture/SS-daemon-wiring.md` v1.3.0
- Architecture dependency: `architecture/SS-daemon-lifecycle.md` v1.0.33 (lock file, auth token, ring buffer)
- Architecture dependency: `architecture/SS-engine-module.md` v1.1.27 (EngineModule trait integration)
