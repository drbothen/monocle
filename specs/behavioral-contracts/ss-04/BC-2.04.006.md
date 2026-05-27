---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:05:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-001
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.006: `directories::ProjectDirs::runtime_dir()` Fallback Chain

## Description

The runtime directory — where `monocle.lock`, `monocle.sock`, `monocle.jsonl`, and
`hooks-settings.json` reside — is resolved via a platform-aware four-level fallback chain.
The chain is evaluated in strict priority order; the first non-empty result wins. This BC
defines the authoritative resolution algorithm, the exact fallback levels, the log messages
emitted at each level, and the fail-fast conditions. This algorithm is called by BC-2.04.001
step 1, BC-2.04.002 PC-1, BC-2.04.004 PC-1, and BC-2.04.005 PC-3. All callers use the same
resolution function; there is no per-subcommand variation.

## Preconditions

1. The resolution function is called as part of daemon or CLI startup.
2. The host OS provides a home directory discovery mechanism (for `ProjectDirs` construction).
3. No assumption is made about which platform the binary is running on; the chain covers
   Linux, macOS, and Windows (secondary target per NFR-008).

## Postconditions

The fallback chain is evaluated in the following order. Each level is tried in sequence;
the first level that produces a valid (non-empty, non-None) path is used and the remaining
levels are skipped.

**Level 1 — MONOCLE_RUNTIME_DIR environment variable override.**

PC-1. If the environment variable `MONOCLE_RUNTIME_DIR` is set AND its value is a non-empty
      string, the value is used verbatim as the runtime directory path. No validation of the
      path is performed at resolution time (validation occurs at directory-creation time in
      BC-2.04.001 step 1 / BC-2.01.005 Postcondition 8).
PC-2. The resolution function logs at INFO level:
      `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var`
PC-3. If `MONOCLE_RUNTIME_DIR` is set to an empty string, it is treated as unset. Resolution
      falls through to Level 2.

**Level 2 — `directories::ProjectDirs::runtime_dir()`.**

PC-4. `ProjectDirs::new("monocle", "monocle", "monocle")` is called. If the call returns
      `None` (no home directory available), proceed immediately to Level 4 (Level 3 also
      requires a `ProjectDirs` instance).
PC-5. If `ProjectDirs::new(...)` returns `Some(proj)`, call `proj.runtime_dir()`. This
      returns `Some` on Linux (the XDG runtime directory, typically
      `$XDG_RUNTIME_DIR/monocle`), and `None` on macOS and Windows by platform-ABI design
      (not a misconfiguration).
PC-6. If `proj.runtime_dir()` returns `Some(path)`, use `path` as the runtime directory.
      Log at INFO level: `INFO: runtime_dir from ProjectDirs::runtime_dir()`
PC-7. If `proj.runtime_dir()` returns `None` (expected on macOS/Windows), proceed to Level 3.

**Level 3 — `directories::ProjectDirs::data_local_dir()` fallback.**

PC-8. Using the `ProjectDirs` instance from Level 2 (which returned `Some`), call
      `proj.data_local_dir()`. This method always returns a valid path when `ProjectDirs::new`
      returned `Some` (it never returns `None`).
      Platform resolution:
      - macOS: `$HOME/Library/Application Support/monocle/`
      - Linux (no XDG_RUNTIME_DIR): `$HOME/.local/share/monocle/`
      - Windows: `%APPDATA%/monocle/`
PC-9. Log at INFO level: `INFO: runtime_dir fallback to data_local_dir (platform: <OS>)`
      where `<OS>` is one of: `macos`, `linux`, `windows`.
PC-10. Use `proj.data_local_dir()` as the runtime directory.

**Level 4 — Fail-fast.**

PC-11. Level 4 is reached only if `ProjectDirs::new("monocle", "monocle", "monocle")`
       returned `None`. This occurs only in environments with no usable home directory
       (misconfigured containers, minimal environments with no `/etc/passwd` entry).
PC-12. The resolution function returns `Err(DaemonStartError::RuntimeDirUnresolvable)`.
PC-13. The caller exits with code 70 and writes to stderr:
       `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`

**Post-resolution:**

PC-14. After a valid path is returned by the resolution function, the caller (BC-2.04.001
       step 1 or BC-2.01.005 Postcondition 8) creates the directory with mode `0o700` if
       it does not exist. Directory creation failure is a separate error (not covered by this
       BC; BC-2.04.001 EC-2.04.001-07 covers it).

## Invariants

1. **Evaluation order is strict.** Level 1 is always checked before Level 2; Level 2 before
   Level 3; Level 3 before Level 4. There is no reordering and no skipping (except when a
   level produces a result, in which case remaining levels are skipped).
2. **Single `ProjectDirs` instance.** `ProjectDirs::new("monocle", "monocle", "monocle")`
   is called at most once per resolution. The result is reused for both Level 2
   (`runtime_dir()`) and Level 3 (`data_local_dir()`).
3. **Empty string treated as unset.** `MONOCLE_RUNTIME_DIR=""` falls through to Level 2.
   The resolution function MUST NOT use an empty string as a runtime directory path.
4. **macOS is a primary target (NFR-008).** Level 3 is the expected resolution path on
   macOS for users who have not set `MONOCLE_RUNTIME_DIR`. This is not a degraded path;
   it is the production-correct resolution for macOS.
5. **Log message must be emitted before directory creation.** The INFO log at each level
   is part of the resolution contract; it allows operators to diagnose which path was
   selected without running `strace`/`dtrace`.
6. **Exactly one of PC-2, PC-6, PC-9 is logged per resolution.** Each call to the resolution
   function emits exactly one INFO log line corresponding to the level that produced the
   result.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-2.04.006-01 | `MONOCLE_RUNTIME_DIR=/tmp/monocle-test` | Level 1 wins; runtime dir is `/tmp/monocle-test`; INFO: `runtime_dir from MONOCLE_RUNTIME_DIR env var` |
| EC-2.04.006-02 | `MONOCLE_RUNTIME_DIR=` (empty string) | Level 1 skipped; falls to Level 2 |
| EC-2.04.006-03 | Linux with `XDG_RUNTIME_DIR=/run/user/1000` set | Level 2 wins; runtime dir is `/run/user/1000/monocle`; INFO: `runtime_dir from ProjectDirs::runtime_dir()` |
| EC-2.04.006-04 | Linux with `XDG_RUNTIME_DIR` unset | Level 2 returns `None`; Level 3 wins; runtime dir is `$HOME/.local/share/monocle`; INFO: `runtime_dir fallback to data_local_dir (platform: linux)` |
| EC-2.04.006-05 | macOS (no `XDG_RUNTIME_DIR`, no `runtime_dir()` support) | Level 2 returns `None`; Level 3 wins; runtime dir is `~/Library/Application Support/monocle/`; INFO: `runtime_dir fallback to data_local_dir (platform: macos)` |
| EC-2.04.006-06 | Container with no home directory (`/etc/passwd` missing entry) | `ProjectDirs::new(...)` returns `None`; Level 4: exit 70; ERROR: `cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path` |
| EC-2.04.006-07 | `MONOCLE_RUNTIME_DIR` set to a relative path (e.g., `./runtime`) | Level 1 wins verbatim; the relative path is used as-is; the caller resolves it relative to the current working directory. This is the operator's responsibility; monocle does not canonicalize the path. |
| EC-2.04.006-08 | Two calls to the resolution function in the same process | Both calls produce the same result (env var and ProjectDirs are deterministic given the same environment); no caching is required but is permitted as an optimization |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `MONOCLE_RUNTIME_DIR=/custom/path` set | Returns `/custom/path`; logs `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var` | happy-path (level 1) |
| Linux, `XDG_RUNTIME_DIR=/run/user/1000`, no env override | Returns `/run/user/1000/monocle`; logs `INFO: runtime_dir from ProjectDirs::runtime_dir()` | happy-path (level 2) |
| macOS, no env override | Returns `~/Library/Application Support/monocle/`; logs `INFO: runtime_dir fallback to data_local_dir (platform: macos)` | happy-path (level 3) |
| Linux, no `XDG_RUNTIME_DIR`, no env override | Returns `~/.local/share/monocle`; logs `INFO: runtime_dir fallback to data_local_dir (platform: linux)` | happy-path (level 3) |
| Container, no home directory | Returns `Err(DaemonStartError::RuntimeDirUnresolvable)`; logs `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path` | error (level 4) |
| `MONOCLE_RUNTIME_DIR=` (empty) | Falls through to platform-default (level 2 or 3) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Level 1 (env override) wins over Level 2 and Level 3 | unit (env mock) |
| VP-TBD | Level 2 wins on Linux with XDG_RUNTIME_DIR set | unit (env mock) |
| VP-TBD | Level 3 wins on macOS (runtime_dir() returns None) | unit (env mock + ProjectDirs mock) |
| VP-TBD | Level 4 returns RuntimeDirUnresolvable when ProjectDirs::new() returns None | unit (ProjectDirs mock) |
| VP-TBD | Empty MONOCLE_RUNTIME_DIR falls through to Level 2 | unit (env mock) |
| VP-TBD | Exactly one INFO log line emitted per resolution call | unit (tracing subscriber capture) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon Lifecycle") per domain-spec/L2-INDEX.md §Capabilities Registry |
| Capability Anchor Justification | CAP-001 ("Daemon Lifecycle") per CAP-001-daemon-lifecycle.md — runtime directory resolution is a foundational component of daemon lifecycle management; all daemon files (lock, socket, JSONL ring, hooks-settings) live under the resolved `runtime_dir`, making this resolution the single point of truth for daemon data locality |
| L2 Domain Invariants | DI-002 (lock file must be present at a known path before hook endpoints accept connections — this BC defines how that path is determined; without a correct runtime_dir resolution, the lock file cannot be created at a predictable location); DI-003 (token written to lock file after port bound — this BC is a prerequisite for DI-003 enforcement since the lock file path must be known before either operation) |
| Architecture Module | `monocle-runtime` per ARCH-INDEX Subsystem Registry SS-04 (`resolve_runtime_dir()` function is classified as "Pure core (impure-adjacent)" per SS-daemon-wiring.md §Module Purity Classification) |
| Architecture Source | SS-daemon-wiring.md v1.0.0 §Daemon Start Sequence Step 1; SS-daemon-lifecycle.md §Start Sequence step 1 (cross-referenced) |
| Cross-Ref | BC-2.01.005 (Postcondition 2 and Precondition 2 specify this same resolution chain; BC-2.04.006 is the normative specification; BC-2.01.005 delegates to it); BC-2.04.001 step 1 (calls this resolution function); BC-2.04.002 PC-1 (calls this function); BC-2.04.004 (uses lock file path derived from runtime_dir) |
| Test File | `monocle-runtime/tests/runtime_dir_resolution.rs` |
| Test Name | `test_BC_2_04_006_runtime_dir_fallback_chain` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.01.005] — BC-2.01.005 Precondition 2 describes the same chain; BC-2.04.006 is the single normative specification; BC-2.01.005 references this BC as the authoritative resolution
- [BC-2.04.001] — depends on this: step 1 of BC-2.04.001 calls this resolution function
- [BC-2.04.002] — depends on this: PC-1 of BC-2.04.002 calls this resolution function
- [BC-2.04.004] — depends on this: runtime_dir is needed to locate the lock file for polling
- [BC-2.04.005] — depends on this: runtime_dir is needed to locate the lock file for PID read

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#daemon-start-sequence-bc-2.04.001` — step 1 references this resolution chain
- `architecture/SS-daemon-lifecycle.md` — §Start Sequence step 1 provides the upstream definition that this BC formalizes

## Story Anchor

S-TBD — Implement `resolve_runtime_dir()` with 4-level fallback chain and INFO logging (filled by story-writer)

## VP Anchors

VP-TBD — Runtime directory resolution unit tests with env mocking (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T12:05:00Z):
- BC-2.04.006 created as new artifact for SS-04 per prd-expansion-scope.md §3.1 (feature
  F-04) and SS-daemon-wiring.md §Daemon Start Sequence Step 1.
- Covers: 4-level fallback chain (MONOCLE_RUNTIME_DIR → runtime_dir() → data_local_dir()
  → fail-fast), INFO log per level, empty-string-as-unset semantics, macOS primary-target
  rationale, 8 edge cases, 6 test vectors, 6 VPs.
- BC-2.01.005 Precondition 2 describes the same chain; this BC is the normative
  specification. BC-2.01.005 was first; this BC was created to be the single authoritative
  source as SS-04 formalizes the wiring layer.
- input-hash: [pending] — to be populated by compute-input-hash after human review.
- SE-16d PASS: 2026-05-26T12:05:00Z > prior 2026-05-26T12:04:00Z (BC-2.04.005).
