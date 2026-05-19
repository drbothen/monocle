---
document_type: plan-doc
level: L4
version: "1.0"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:30:00Z
phase: 2
visibility: holdout-evaluator-only
traces_to: STORY-INDEX.md
---

# Holdout Scenarios: monocle Phase 2

> **PHASE 4 EVALUATOR ACCESS ONLY.**
> This document MUST NOT be shared with implementers or test-writers.
> Holdout scenarios are hidden acceptance scenarios derived from BCs but NOT
> mechanically duplicating any AC from the story corpus. They test the spirit
> and intent of the behavioral contracts from a fresh perspective.
>
> Evaluator: `vsdd-factory:holdout-evaluator`
> Information asymmetry: evaluator has NOT seen story ACs.

---

## Wave 1 Holdout Scenarios

### HS-W1-001: DTU Clone Sends Real Claude Code Auth Pattern

**Wave:** 1
**Source BC:** dtu-assessment.md §Auth Header; ADR-0005
**Scenario:** The holdout evaluator starts the `dtu-claude-code-hooks-v1` clone and the
monocle daemon. The clone reads the daemon's lock file and sends a POST to
`/hooks/pre-tool-use` with `X-Claude-Code-Ide-Authorization: <raw-64-hex>` (no prefix).
**Expected:** HTTP 200 response. WARN log line containing "compatibility alias".
**NOT in any story AC:** Story ACs test individual headers; this tests the clone-to-daemon end-to-end path.

### HS-W1-002: Workspace Compiles on MSRV 1.86 Exactly

**Wave:** 1
**Source:** NFR-007; SS-deps-pin-manifest.md §MSRV Policy
**Scenario:** Evaluator runs `cargo +1.86 build --workspace`. Must succeed without errors.
Then evaluator runs `cargo +1.85 build --workspace`. Must fail (MSRV violation).
**Expected:** 1.86 succeeds; 1.85 fails with MSRV error.
**NOT in any story AC:** Story AC-002 only checks that toolchain.toml pins 1.86, not that 1.85 is rejected.

---

## Wave 2 Holdout Scenarios

### HS-W2-001: Lock File Token Rotation During TUI Reconnect

**Wave:** 2
**Source BC:** BC-2.01.005, BC-2.01.008, BC-2.01.001
**Scenario:** Daemon starts; TUI reads token from lock file. Daemon is then restarted (new
token generated). Evaluator sends GET /healthz without any auth header. Must return 200.
Then sends GET /status with the OLD token. Must return 401.
**Expected:** /healthz always returns 200; /status with stale token returns 401.
**NOT in any story AC:** Tests the interplay of lock file rotation + healthz unauthenticated + auth staleness.

### HS-W2-002: Concurrent Body Limit + Auth Failure

**Wave:** 2
**Source BC:** BC-2.01.003, BC-2.01.009
**Scenario:** Evaluator sends a POST to `/hooks/pre-tool-use` with:
- Body size = 262,145 bytes (exceeds 256 KiB)
- Auth header = missing
**Expected:** HTTP 413 (body limit checked BEFORE auth token extraction).
NOT HTTP 401 (auth failure takes lower precedence than body limit middleware).
**NOT in any story AC:** Story ACs test body limit and auth separately; this tests their ordering.

### HS-W2-003: /status Response ABI Version Matches Compile-Time Const

**Wave:** 2
**Source BC:** BC-2.02.001, BC-2.02.002
**Scenario:** Evaluator starts daemon; calls `GET /status`. Extracts `abi_version` field.
Also reads `monocle_core::MONOCLE_ABI_VERSION` via a compiled test binary.
Both values must be identical.
**Expected:** `/status` `.abi_version` == compile-time const value (both = 1).
**NOT in any story AC:** Tests that the runtime value and the const are not decoupled.

### HS-W2-004: Non-Exhaustive HookType — New Variant Compiles Without Breakage

**Wave:** 2
**Source BC:** BC-2.02.003
**Scenario:** Evaluator adds a new variant `HookType::FutureVariant` to the `HookType` enum
definition. Attempts to compile `monocle-runtime`. Must compile successfully with the wildcard
match arm. All existing match-arms that cover `HookType` should NOT require updating.
**Expected:** Compilation succeeds. `#[non_exhaustive]` enforcement via wildcard arm.
**NOT in any story AC:** Story ACs test the attribute is present; this tests downstream compile behavior.

### HS-W2-005: EngineModule detect() Rejects Basename Substring Match

**Wave:** 2
**Source BC:** BC-2.03.002
**Scenario:** Evaluator creates a `ProcessSnapshot` with `exe_path = PathBuf::from("/usr/local/bin/claude-code-runner")`.
Calls `ClaudeCodeModule::detect(&proc)`. Must return `false`.
Then creates one with `exe_path = PathBuf::from("/path/to/claude")`. Must return `true`.
**Expected:** "claude-code-runner" → false; "claude" → true (basename exact match only).
**NOT in any story AC:** ACs test "claude-squad" and "claudio" but not "claude-code-runner" specifically.

---

## Wave 3 Holdout Scenarios

### HS-W3-001: Crash Recovery Checkpoint Survives Daemon Restart

**Wave:** 3
**Source BC:** BC-2.01.006
**Scenario:** Evaluator triggers an abnormal exit (exit code 2 — drain timeout). Verifies
`monocle-crash.json` exists in `<runtime_dir>`. Starts daemon again. Verifies INFO log
containing "crash checkpoint found". Verifies `monocle-crash.json` is removed after logging.
**Expected:** Checkpoint created on abnormal exit; logged and removed on next start.
**NOT in any story AC:** ACs test write and detect separately; this tests the full lifecycle across two daemon instances.

### HS-W3-002: JSONL Ring format_version Survives Rotation

**Wave:** 3
**Source BC:** BC-2.01.007 (FC-01)
**Scenario:** Evaluator forces ring rotation by submitting enough hook events to exceed 50 MB.
After rotation, sends one more hook event. Reads the new JSONL file and verifies the first
key of the new entry is `format_version`.
**Expected:** After rotation, new records still have `format_version` as first key. No key-order regression.
**NOT in any story AC:** ACs test format_version on individual records, not after rotation.

### HS-W3-003: VsddFactoryAdapter Detects monocle's Own Factory

**Wave:** 3
**Source BC:** BC-2.02.005
**Scenario:** Evaluator runs `VsddFactoryAdapter::detect(path_to_monocle_repo_root)` in an
integration test that uses the REAL monocle `.factory/STATE.md` file. Must return `Some(...)`.
Then modifies STATE.md frontmatter to use `document_type: something-else` and runs detect again. Must return `None`.
**Expected:** Detects correctly against real STATE.md; correctly rejects modified STATE.md.
**NOT in any story AC:** ACs test self-referential detection; this tests the rejection of modified frontmatter.

### HS-W3-004: HomeUnresolvable Does Not Leak Partial Engine State

**Wave:** 3
**Source BC:** BC-2.03.003, BC-2.03.001 postcondition 5
**Scenario:** Evaluator unsets all 4 home-env vars. Calls `ClaudeCodeModule::metadata()`.
Verifies: (1) returns `Err(HomeUnresolvable)`; (2) no partial `EngineMetadata` struct is
returned or logged (no half-initialized state); (3) E-ENG-001 log message exactly matches spec.
**Expected:** Clean error; no partial struct; correct error message format.
**NOT in any story AC:** ACs test the error return path and log message but not the absence of partial state.

### HS-W3-005: FactoryAdapter subscribe() Stream is Empty in Phase 1

**Wave:** 3
**Source BC:** BC-2.02.004 invariant 3; BC-2.02.005 postcondition 3
**Scenario:** Evaluator calls `VsddFactoryAdapter::subscribe()`. Polls the returned stream
with a 100ms timeout. Stream must yield `None` immediately (empty). Must NOT block for 100ms.
**Expected:** `Ok(empty_stream)`; first poll returns `None` (ready); no file watcher instantiated.
**NOT in any story AC:** ACs test that subscribe() returns Ok; this tests that the stream is actually empty and non-blocking.

---

## Wave Coverage Summary

| Wave | Holdout Scenarios | Stories Covered |
|------|------------------|----------------|
| Wave 1 | HS-W1-001, HS-W1-002 | S-DTU-001, S-001 |
| Wave 2 | HS-W2-001..HS-W2-005 | S-002, S-003, S-004, S-006, S-009, S-010, S-011, S-014 |
| Wave 3 | HS-W3-001..HS-W3-005 | S-007, S-008, S-012, S-015 |

**Total holdout scenarios: 12**
**Coverage: ≥1 scenario per wave (required); ≥1 scenario per BC grouping (enforced above)**
