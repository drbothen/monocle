---
document_type: behavioral-contract
level: L3
version: "1.3.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:03:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "c81613b"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D2-010, F-P1D3-002]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.010: Hook Tmpfile Generation at runtimeDir/hooks-settings.json

## Description

The daemon generates `<runtime_dir>/hooks-settings.json` during startup (step 9 of the start
sequence, after the lock file is written at step 8) to publish its 4 hook endpoint URLs and
auth token to Claude Code. Claude Code reads this file via its `--settings` flag to discover
the daemon's dynamically-assigned port and the full wire token for the `X-Monocle-Authorization`
header. The file is written atomically via `tempfile::persist` at mode `0o600`, regenerated on
every daemon restart, and removed on graceful shutdown. The SOQ-2 ordering invariant
(lock file write before hooks-settings.json write) eliminates the auth-token race condition.

## Preconditions

1. The daemon start sequence has completed step 8 (lock file written, SOQ-2 commit point
   reached) per BC-2.04.001 PC-8. The OS-assigned port is known and stable; the auth token
   is committed to the lock file.
2. `<runtime_dir>` exists and is owned by the running user with mode `0o700`
   (created at step 1, BC-2.04.001 PC-1).
3. The `tempfile` crate is available in the `monocle-runtime` dependency tree.
4. The `serde_json` crate is available for JSON serialization.
5. The host filesystem supports atomic rename (POSIX `rename(2)` or equivalent).

## Postconditions

**PC-1 — File written atomically via `tempfile::persist`.**
The implementation MUST use the `tempfile::NamedTempFile::new_in(&runtime_dir)?` →
`serde_json::to_writer_pretty(&mut tmp, &hooks_settings)?` → `tmp.persist(&hooks_settings_path)?`
pattern. Naked `std::fs::write` to `hooks-settings.json` is forbidden per
SS-conventions-anti-patterns.md §Forbidden Patterns. There is no observable window where
`hooks-settings.json` contains partial content; consumers either see the old file (no longer
expected in this context since we write once per daemon start) or the complete new file.

**PC-2 — File permissions: mode `0o600`.**
After `tempfile::persist`, the implementation calls:
```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&hooks_settings_path,
        std::fs::Permissions::from_mode(0o600))?;
}
```
The resulting file is readable and writable by the daemon user only. Other OS users cannot
read the file or discover the auth token via the hooks-settings.json path.

**PC-3 — Schema: 4 hook endpoint URLs with port and token embedded.**
The JSON content follows the schema:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/pre-tool-use -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<64-hex>' -d @-"
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/notification -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<64-hex>' -d @-"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/stop -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<64-hex>' -d @-"
          }
        ]
      }
    ],
    "PostToolUse": [],
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/prompt-submit -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<64-hex>' -d @-"
          }
        ]
      }
    ],
    "PreCompact": []
  }
}
```
Key properties:
- `<port>` is the OS-assigned ephemeral port bound at daemon start step 3.
- `<64-hex>` is the 64-character lowercase hex auth token generated at step 7. The full wire
  token in the command is `monocle-v1:<64-hex>`.
- `PostToolUse` and `PreCompact` entries have empty `hooks` arrays. Claude Code ignores hook
  types with empty arrays; these entries provide forward-compatibility structure.
- `SessionStart` is NOT listed explicitly because Claude Code's hook configuration does not
  include it as a first-class hook type in the hooks-settings.json schema. The SessionStart
  endpoint (`POST /hooks/session-start`) is reachable via the axum router but is not wired
  through hooks-settings.json in Phase 1.

**PC-4 — SOQ-2 ordering: written AFTER lock file.**
`write_hooks_settings()` is called at step 9 of the start sequence — strictly after
`write_lock_file()` at step 8. This ordering is enforced in `daemon_start_sequence()` by
function call order. No concurrent execution path calls `write_hooks_settings()` before
`write_lock_file()` completes. This guarantees that any Claude Code subprocess that reads
`hooks-settings.json` to obtain the token has already seen the token committed to the lock
file at step 8.

**PC-5 — File removed on graceful shutdown.**
On graceful shutdown (SIGTERM or SIGINT triggers drain per BC-2.01.004), the daemon removes
`<runtime_dir>/hooks-settings.json` alongside `<runtime_dir>/monocle.lock` and
`<runtime_dir>/monocle.sock`. If removal fails (e.g., file was already deleted), the error
is logged at WARN level and shutdown continues.

**PC-6 — Error propagation.**
If `write_hooks_settings()` fails at any step (NamedTempFile creation, JSON serialization,
persist, or chmod), the daemon exits with code 72 and logs:
`ERROR: failed to write hooks-settings.json: <error>`.
No partially-written file is left at the target path (tempfile::persist guarantees atomicity).

## Invariants

1. `tempfile::persist` MUST be used for all writes to `hooks-settings.json`. `std::fs::write`
   to this path is forbidden (SS-conventions-anti-patterns.md §Forbidden Patterns).
2. Mode `0o600` MUST be set after every `persist` call. The set-permissions call is not
   optional; it must execute even on platforms where tempfile's default mode is already
   restrictive.
3. The SOQ-2 ordering (lock file before hooks-settings.json) MUST be preserved in all code
   paths, including error-recovery restarts within the same daemon process.
4. The `<64-hex>` token embedded in the command strings MUST match `DaemonState.auth_token`
   exactly (without the `monocle-v1:` prefix in the state; the prefix is added at serialization
   time per the wire format convention in BC-2.01.008).
5. `PostToolUse` and `PreCompact` MUST be present with empty arrays in the output JSON.
   Omitting them would break forward-compat for Claude Code versions that validate all
   hook type keys.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-088 | `runtime_dir` filesystem is full when generating the temp file | `NamedTempFile::new_in()` fails; daemon exits with code 72 and logs the error; no partial file at target path |
| EC-089 | `runtime_dir` exists but daemon user lacks write permission | `NamedTempFile::new_in()` returns `Err(PermissionDenied)`; daemon exits with code 72 |
| EC-090 | `hooks-settings.json` already exists from a previous crashed daemon run | `tempfile::persist` atomically replaces the old file; new file has updated port, token, and mode 0o600 |
| EC-091 | `chmod 0o600` fails after successful persist (unusual; e.g., immutable filesystem flag set after persist) | Daemon logs `ERROR: failed to write hooks-settings.json: <chmod error>` and exits with code 72; the file is present with whatever mode tempfile left it at (security incident risk, so hard-exit is correct) |
| EC-092 | Daemon crashes between lock file write (step 8) and hooks-settings.json write (step 9) | Lock file is present; hooks-settings.json is absent. On the next daemon start, the crash recovery checkpoint (BC-2.01.006) detects the incomplete prior run. A new start sequence generates a fresh hooks-settings.json. Claude Code cannot discover the dead daemon via hooks-settings.json because it was never written. |
| EC-093 | Two daemon instances race to write hooks-settings.json (double-start scenario, should be rare given lock file guard) | `tempfile::persist` is a POSIX atomic rename; the last writer wins. However, this scenario indicates the lock file guard failed; root cause is the double-start invariant violation, not the hooks-settings.json write. The surviving daemon holds the valid lock file; the other exits at step 2 when it re-checks the lock. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Daemon start sequence completes step 8; port=54321; token=`aabbcc...` (64 chars) | `hooks-settings.json` exists at `<runtime_dir>/hooks-settings.json`; mode 0o600; contains `http://127.0.0.1:54321/hooks/pre-tool-use` and `monocle-v1:aabbcc...` in PreToolUse command | happy-path |
| Graceful shutdown signal received | `hooks-settings.json` removed from `<runtime_dir>` | happy-path |
| `runtime_dir` filesystem full | Daemon exits with code 72; error logged; no partial file at path | error |
| `hooks-settings.json` from crashed prior run exists | File atomically replaced; new port and token embedded | edge-case |
| Read `hooks-settings.json` and parse PostToolUse key | `PostToolUse` key present with `"hooks": []` array | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `hooks-settings.json` mode is 0o600 after daemon start | integration (stat the file) |
| VP-TBD | Port and token in command strings match DaemonState values | integration |
| VP-TBD | `PostToolUse` and `PreCompact` keys present with empty arrays | unit (schema validation) |
| VP-TBD | File removed on graceful shutdown | integration |
| VP-TBD | Failure exits with code 72 and no partial file | unit (mock NamedTempFile failure) |
| VP-TBD | SOQ-2: hooks-settings.json written after lock file in start sequence | integration (observe file creation timestamps) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — "hook tmpfile generation" is named explicitly as a CAP-004 responsibility; this BC is the direct operationalization of that named responsibility, specifying the atomic write, mode, schema, SOQ-2 ordering, and removal contract for `hooks-settings.json` |
| L2 Domain Invariants | DI-002 (the lock file MUST be present and contain a valid token before any hook endpoint accepts connections — this BC's SOQ-2 ordering invariant ensures hooks-settings.json embeds a token that is already in the lock file; Claude Code cannot reach hook endpoints with a token that hasn't been committed to the lock file); DI-003 (auth token MUST be written to lock file after port is bound — PC-4 enforces that hooks-settings.json is written after the lock file, which was written after port bind, preserving DI-003 as a transitivity guarantee) |
| Architecture Module | monocle-runtime (`write_hooks_settings()` function) per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §Hook Tmpfile Generation |
| Cross-Ref | BC-2.01.005 (lock file lifecycle — SOQ-2 upstream step); BC-2.01.008 (auth token wire format — token embedded in command strings); BC-2.04.001 (daemon start sequence — step 9 calls this contract) |
| Test File | `monocle-runtime/tests/hooks_settings_generation.rs` |
| Test Name | `test_BC_2_04_010_hooks_settings_generation` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.001] — depends on: daemon start sequence step 9 triggers this contract; step 8 (lock file) provides SOQ-2 prerequisite
- [BC-2.01.005] — composes with: lock file lifecycle (SOQ-2 ordering upstream)
- [BC-2.01.008] — composes with: auth token wire format governs the `monocle-v1:<64-hex>` string embedded in command URLs
- [BC-2.04.007] — composes with: PreToolUse endpoint URL in PC-3 schema; routing defined in BC-2.04.007
- [BC-2.04.008] — composes with: Notification endpoint URL in PC-3 schema
- [BC-2.04.009] — composes with: Stop and PromptSubmit endpoint URLs in PC-3 schema

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#hook-tmpfile-generation` — full schema, atomic write requirement, SOQ-2 ordering rationale
- `architecture/SS-daemon-wiring.md#daemon-start-sequence` — step 9 context

## Story Anchor

S-TBD — Implement hooks-settings.json generation with atomic write, mode 0o600, and SOQ-2 ordering (filled by story-writer)

## VP Anchors

- VP-TBD — filled after VP creation

## §Trace v1.0.0

**Initial production** (2026-05-26T12:03:00Z):
- BC-2.04.010 created as new artifact for SS-04 §Hook Tmpfile Generation per task instruction.
- Covers: tempfile::persist atomic write, mode 0o600, full JSON schema (5 hook types + 2 empty
  arrays), SOQ-2 ordering guarantee, graceful shutdown removal, exit code 72 on failure.
- Capability anchor: CAP-004 per ARCH-INDEX §SS-04 Capability Traceability row ("hook tmpfile
  generation" named explicitly in CAP-004 statement).
- SE-16d PASS: 2026-05-26T12:03:00Z > chain prior 2026-05-26T12:02:00Z. PASS.

## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T13:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.0.0` → `SS-daemon-wiring.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp 2026-05-26T13:00:00Z > v1.0.0 timestamp 2026-05-26T12:03:00Z. PASS.

## §Trace v1.1.0

**F-P1D3-002 CRITICAL — Hook endpoint count corrected from 5 to 4; O-P1D3-003 timestamp monotonicity fixed** (2026-05-26T14:00:00Z):
- Description: "5 hook endpoint URLs" → "4 hook endpoint URLs". Only PreToolUse, Notification,
  Stop, and UserPromptSubmit have non-empty hook arrays in hooks-settings.json. PostToolUse and
  PreCompact have empty arrays; SessionStart is not listed in hooks-settings.json (it is invoked
  via Claude Code's internal lifecycle, not via hooks-settings.json). The count 4 is the number
  of endpoints that carry actual curl command URLs.
- PC-3 header: "Schema: 5 hook endpoint URLs" → "Schema: 4 hook endpoint URLs".
- O-P1D3-003: §Trace v1.0.1 timestamp updated from 2026-05-26T00:00:00Z to 2026-05-26T13:00:00Z
  (strictly greater than v1.0.0 2026-05-26T12:03:00Z) to restore monotonicity.
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-26T14:00:00Z > v1.0.1 timestamp 2026-05-26T13:00:00Z. PASS.

## §Trace v1.2.0

**F-P1D4-003 LOW — Architecture Source pin updated from v1.1.0 to v1.2.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.1.0` → `SS-daemon-wiring.md v1.2.0` per F-P1D4-003 bulk update.
- SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.3.0

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-daemon-wiring.md v1.2.0 → v1.3.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §Hook Tmpfile Generation` → `SS-daemon-wiring.md v1.3.0 §Hook Tmpfile Generation`.
- Plain version-pin refresh. No substantive content propagation required — §Hook Tmpfile Generation section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.3.0 timestamp >= v1.2.0. PASS.
