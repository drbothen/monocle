---
document_type: plan-doc
level: ops
version: "1.6"
status: active
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T12:00:00Z
phase: 2
visibility: holdout-evaluator-only
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
input-hash: "[live-state]"
traces_to: .factory/stories/STORY-INDEX.md
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

### HS-W1-002: Workspace Compiles on MSRV 1.88 Exactly

**Wave:** 1
**Source:** NFR-007; SS-deps-pin-manifest.md §MSRV Policy
**Scenario:** Evaluator runs `cargo +1.88 build --workspace`. Must succeed without errors.
Then evaluator runs `cargo +1.87 build --workspace`. Must fail (MSRV violation).
**Expected:** 1.88 succeeds; 1.87 fails with MSRV error.
**NOT in any story AC:** Story AC-002 only checks that toolchain.toml pins 1.88, not that 1.87 is rejected.

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

### HS-W2-005: EngineModule detect() Rejects Basename Substring Match but Accepts claude.js

**Wave:** 2
**Source BC:** BC-2.03.002 (PC-4, EC-034, EC-035)
**Scenario:** Evaluator runs three detect() calls in sequence:
1. `ProcessSnapshot { exe_path: Some("/usr/local/bin/claude.js") }` → `detect()` must return `true`
   (the Node.js wrapper is an explicitly allowed name; BC-2.03.002 PC-4 + EC-034)
2. `ProcessSnapshot { exe_path: Some("/usr/local/bin/claude-code-runner") }` → `detect()` must return `false`
   (prefix substring; NOT in the two-element allowed set)
3. `ProcessSnapshot { exe_path: Some("/usr/local/bin/claude-js") }` → `detect()` must return `false`
   (close variant; NOT `"claude.js"` exactly — hyphen vs dot)
**Expected:** claude.js → true; claude-code-runner → false; claude-js → false.
**NOT in any story AC:** Story AC-001 covers the two allowed names; this holdout tests the "almost-claude.js" boundary case (hyphen vs dot) that would expose non-exact matching logic.

### HS-W2-006: Graceful Shutdown Drain Race — Concurrent POST /shutdown During /healthz Drain Transition

**Wave:** 2
**Source BC:** BC-2.01.004 (PC-1, PC-2, PC-5, INV-1, INV-3, EC-050)
**Scenario:** Evaluator starts daemon. Sends SIGTERM to begin drain. Immediately (within 50ms)
concurrently sends a valid authenticated `POST /shutdown` while the daemon is still in the
`AppMode::ShuttingDown` transition (the first signal has been received but in-flight requests
have not yet completed). Also concurrently sends a `GET /healthz`.
Three assertions must hold simultaneously:
1. The concurrent `POST /shutdown` receives HTTP 200 (second shutdown acknowledged) and triggers
   immediate hard close with exit code `2` (admin forced-stop; BC-2.01.004 EC-050).
2. The `GET /healthz` returns HTTP 503 with body `{"status":"shutting_down"}` (PC-3), NOT a
   connection refused or HTTP 200.
3. Exit code written to the OS is `2`, not `0` (BC-2.01.004 PC-8 and EC-050: second authenticated
   `/shutdown` during active drain = admin forced-stop = exit 2, not graceful exit 0).
**Expected:** POST /shutdown → HTTP 200 + exit 2; GET /healthz → HTTP 503; exit code 2.
**NOT in any story AC:** Story ACs test shutdown trigger and exit codes independently. This holdout
tests the race between concurrent second `/shutdown` and active drain — the evaluator cannot trick
the implementation by simply calling /shutdown once and checking exit 0 (the happy path).
**Holdout discipline:** Scenario cannot be satisfied by reading S-005 ACs. The AC describes SIGTERM
trigger (AC-001), POST /shutdown trigger (AC-002), and exit code taxonomy (AC-004) independently.
The race condition, the HTTP 200 acknowledgement of the second /shutdown during active drain, and
the simultaneous /healthz assertion are all derived from BC-2.01.004 EC-050 + INV-1 + INV-3 + PC-8.

### HS-W2-007: HookEnvelope Proto Wire Forward-Compatibility — Unknown Phase 4 Field Numbers Survive Round-Trip

**Wave:** 2
**Source BC:** BC-2.02.006 (PC-4, PC-5, EC-024), BC-2.02.007 (PC-1, PC-2), BC-2.02.008 (PC-1, INV-1, EC-027, EC-028)
**Scenario:** Part A (wire forward-compatibility per BC-2.02.006 EC-024 + BC-2.02.008 INV-1):
Evaluator constructs a raw protobuf binary that is a valid `HookEnvelope` with `schema_version: 1`
PLUS an additional unknown field at field number 100 (a Phase 4 addition in the reserved 100–999
range, per BC-2.02.006 PC-4/PC-5). Decodes this binary using the Phase 1 prost-generated
`HookEnvelope` struct. Asserts:
- `envelope.schema_version == 1` (known field survived)
- No panic or decode error (proto3 forward compat; BC-2.02.008 INV-1)
- Unknown field 100 is silently preserved (not returned, not rejected)
Part B (zero schema_version rejection contract per BC-2.02.008 EC-027):
Evaluator constructs `HookEnvelope { schema_version: 0, .. }`. A Phase 1 gate test
verifies that the schema_version field IS accessible as `u32` and equals 0 — the
Phase 1 codebase stores it without active checking (checking is Phase 4's responsibility
per BC-2.02.008 PC-1). The holdout verifies that the code does NOT panic on value 0
and does NOT attempt to decode a schema that does not exist yet.
**Expected:** Part A — `schema_version: 1` intact; unknown field 100 causes no error; proto3 forward compat passes. Part B — `schema_version: 0` message decoded without panic; field accessible as typed u32.
**NOT in any story AC:** S-013 ACs test that schema_version is at field number 1 (AC-001, AC-006), the Rust struct field exists (AC-002, AC-003), and Phase 4 validation contract is stated (AC-004, AC-005). None of the ACs construct a binary with an unknown Phase 4 field number and verify round-trip survival — that is the forward-compat property that BC-2.02.006 EC-024 and BC-2.02.008 INV-1 guarantee.

---

## Wave 3 Holdout Scenarios

### HS-W3-001: Crash Recovery Checkpoint Survives Daemon Restart (Correct Schema + Filename)

**Wave:** 3
**Source BC:** BC-2.01.006 (PC-1, PC-2, PC-5, INV-1, INV-2)
**Scenario:** Evaluator triggers a non-clean daemon exit via SIGTERM during active session
(shutdown_reason = "signal"). Verifies `monocle.recovery.json` (NOT `monocle-crash.json`)
exists in `<runtime_dir>` with this exact 4-field schema per BC-2.01.006 invariant 1:
```json
{"pid":<N>,"shutdown_reason":"signal","last_app_mode":"Running","shutdown_utc":"<ISO-ms>"}
```
Validates: `pid` ≥ 1; `shutdown_reason` is exactly `"signal"` (closed enum); `shutdown_utc`
matches regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` (mandatory millisecond precision).
Starts daemon again. Verifies log contains exactly:
`WARN: recovery checkpoint found; prior daemon exited without clean shutdown`
Verifies `monocle.recovery.json` is REMOVED after the daemon processes the recovery state
(TUI attaches and responds Y, OR 60-second timeout elapses).
**Expected:** `monocle.recovery.json` created (not `monocle-crash.json`); 4-field schema exact; WARN log verbatim; file deleted post-resolution.
**NOT in any story AC:** ACs test write and detect separately; this tests the full lifecycle across two daemon instances AND validates the correct filename and schema are used.

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
**Source BC:** BC-2.03.003, BC-2.03.001 postcondition 6
**Scenario:** Evaluator unsets all 4 home-env vars. Calls `ClaudeCodeModule::metadata()`.
Verifies: (1) returns `Err(HomeUnresolvable)`; (2) no partial `EngineMetadata` struct is
returned or logged (no half-initialized state); (3) E-ENG-001 log message exactly matches spec.
**Expected:** Clean error; no partial struct; correct error message format.
**NOT in any story AC:** ACs test the error return path and log message but not the absence of partial state.

### HS-W3-005: FactoryAdapter subscribe() Stream is Empty in Phase 1

**Wave:** 3
**Source BC:** BC-2.02.005 invariant 3 (canonical locus: "subscribe() returns Ok(Box::pin(futures::stream::empty())) in Phase 1")
**Scenario:** Evaluator calls `VsddFactoryAdapter::subscribe()`. Polls the returned stream
with a 100ms timeout. Stream must yield `None` immediately (empty). Must NOT block for 100ms.
**Expected:** `Ok(empty_stream)`; first poll returns `None` (ready); no file watcher instantiated.
**NOT in any story AC:** ACs test that subscribe() returns Ok; this tests that the stream is actually empty and non-blocking.

### HS-W3-006: Concurrent Body Limit + Auth Failure

**Wave:** 3
**Source BC:** BC-2.01.003, BC-2.01.009
**Scenario:** Evaluator sends a POST to `/hooks/pre-tool-use` with:
- Body size = 262,145 bytes (exceeds 256 KiB)
- Auth header = missing
**Expected:** HTTP 413 (body limit checked BEFORE auth token extraction).
NOT HTTP 401 (auth failure takes lower precedence than body limit middleware).
**NOT in any story AC:** Story ACs test body limit and auth separately; this tests their ordering.
**Wave rationale:** BC-2.01.009 is implemented by S-009 (Wave 3); this scenario requires both
S-004 (body limit, Wave 2) AND S-009 (auth middleware, Wave 3) to be complete. Cannot be
evaluated until Wave 3.

---

## Wave Coverage Summary

| Wave | Holdout Scenarios | Stories Covered |
|------|------------------|----------------|
| Wave 1 | HS-W1-001, HS-W1-002 | S-DTU-001, S-001 |
| Wave 2 | HS-W2-001, HS-W2-003, HS-W2-004, HS-W2-005, HS-W2-006, HS-W2-007 | S-002, S-003, S-004, S-005, S-006, S-010, S-011, S-013, S-014 |
| Wave 3 | HS-W3-001, HS-W3-002, HS-W3-003, HS-W3-004, HS-W3-005, HS-W3-006 | S-007, S-008, S-009, S-012, S-015 |

**Total holdout scenarios: 14**
**Coverage: ≥1 scenario per wave (required); ≥1 scenario per BC grouping (enforced above)**
**Note (F-PHASE2-R03-10, F-PHASE2-R04-06):** HS-W3-006 (originally HS-W2-002) is a Wave 3 scenario. S-009 (BC-2.01.009) is Wave 3; the Concurrent Body Limit + Auth Failure scenario cannot be evaluated without S-009's auth middleware being complete. Corrected to Wave 3 H2 section per F-PHASE2-R04-06.
**Note (GAP-PHASE2-R12-3/R12-4):** HS-W2-006 added for BC-2.01.004 (Graceful Shutdown) coverage; HS-W2-007 added for BC-2.02.006/007/008 (HookEnvelope) coverage. Both Wave 2 scenarios derive from BC body semantics not mechanically repeated in story ACs.

---

## §Trace v1.0

**Phase 2 story decomposition initial burst** (2026-05-19T04:30:00Z):
- Initial holdout scenario decomposition: 12 scenarios across 3 waves. Scenarios derived from BCs but not duplicating any story AC (Phase 4 information asymmetry preserved). Wave coverage summary table produced.

## §Trace v1.1

**Phase 2 r02 remediation** (2026-05-19):
- GAP-PHASE2-R02-4 (LOW): frontmatter `level: ops` and `version: "1.1"` added (initial v1.0 lacked these fields per §Trace coherence audit).

## §Trace v1.2

**Phase 2 r06 cascade** (2026-05-19):
- `traces_to:` updated from `.factory/stories/STORY-INDEX.md v1.4` → `v1.5` per SE-22 v2 forward consumer-ledger sweep (STORY-INDEX bumped v1.4→v1.5 in r07; holdout-scenarios.md traces_to pinned to STORY-INDEX). Frontmatter bumped to v1.2.
- Retrospective §Trace v1.0/v1.1/v1.2 entries added per F-PHASE2-R08-03 closure (initial sibling-sweep gap: zero §Trace entries for 2 declared versions).

## §Trace v1.3

**Phase 2 r10 closure** (2026-05-19):
- F-PHASE2-R10-02 (MEDIUM) / GAP-PHASE2-R10-3: STORY-INDEX pin updated v1.5 → v1.7 per SE-22 v2 consumer-ledger sweep (STORY-INDEX bumped v1.5→v1.6 in r09 burst; then v1.6→v1.7 in this r10 burst for Decision 11 S-001.blocks correction). holdout-scenarios.md carries forward to current STORY-INDEX version as required by SE-22 v2 forward consumer-ledger discipline.
- Frontmatter bumped to v1.3.

## §Trace v1.5

**Path B Wave 6 MSRV propagation tail** (2026-05-29):
- HS-W1-002 scenario updated: title "MSRV 1.86 Exactly" → "MSRV 1.88 Exactly"; `cargo +1.86` → `cargo +1.88`; failure version 1.85 → 1.87 (N-1 from new floor); AC-002 cross-reference updated to pin "1.88". 4 active-content sites updated.
- version bumped 1.4 → 1.5. Closes consumer-story cascade started at architect f3533ce.

## §Trace v1.4

**Phase 2 r12 fix-all burst: GAP-PHASE2-R12-2/R12-3/R12-4 closed** (2026-05-19):
- GAP-PHASE2-R12-2 (LOW): Wave 3 section reordered monotonically — HS-W3-006 (Concurrent Body Limit + Auth Failure) was out-of-order (appeared first in Wave 3 section, preceding HS-W3-001..005). Corrected to HS-W3-001, HS-W3-002, HS-W3-003, HS-W3-004, HS-W3-005, HS-W3-006.
- GAP-PHASE2-R12-3 (LOW): HS-W2-006 added for BC-2.01.004 (Graceful Shutdown). Scenario exercises concurrent POST /shutdown during active /healthz drain transition — the race between a second authenticated /shutdown and the active drain window (EC-050 + INV-1 + INV-3 + PC-8). Cannot be satisfied by reading S-005 ACs individually; derived from BC-2.01.004 body directly.
- GAP-PHASE2-R12-4 (LOW): HS-W2-007 added for BC-2.02.006/BC-2.02.007/BC-2.02.008 (HookEnvelope). Scenario exercises proto wire forward-compatibility: Part A — unknown Phase 4 field numbers survive round-trip decode; Part B — schema_version: 0 message decoded without panic. Derived from BC-2.02.006 EC-024 + BC-2.02.008 INV-1/EC-027; not mechanically stated in S-013 ACs.
- Wave Coverage Summary updated: Wave 2 now covers HS-W2-001, HS-W2-003, HS-W2-004, HS-W2-005, HS-W2-006, HS-W2-007; S-005 and S-013 added to Wave 2 covered stories. Total holdout scenarios: 12 → 14.
- SE-22 v2 cascade: traces_to updated from v1.7 → v1.8 (STORY-INDEX bumped v1.7→v1.8 in this r12 burst). Frontmatter bumped to v1.4.
## §Trace v1.6 — POL-11 version-pin remediation (2026-05-30)

**Bump:** 1.5 → 1.6.
**Scope:** `traces_to:` field: `STORY-INDEX.md v1.8` → `STORY-INDEX.md v5.21` (Option 1 per ADR-0007 §Decision; active live pointer bumped to canonical current version post-remediation-burst bumps).
**SE-16d PASS:** 2026-05-30 >= prior date (patch; no normative behavioral change).
