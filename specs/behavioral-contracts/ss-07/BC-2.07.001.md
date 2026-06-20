---
document_type: behavioral-contract
level: L3
version: "1.1.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T00:00:00Z
phase: phase-1-expansion
inputs:
  - {path: .factory/specs/architecture/SS-config.md, version: "1.0.0"}
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-07
capability: CAP-007
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D2-010, F-P1D3-003]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.07.001: Config File Atomic Write via `tempfile::persist`

## Description

All writes to `<config_dir>/config.json` (the platform-dependent path resolved via
`directories::ProjectDirs::config_dir()` — see SS-config.md §Config File Location for
per-platform expansions) must use the `tempfile::persist` pattern: write serialized JSON to
a `NamedTempFile` created in the same directory as the target, flush and sync the temp file,
then atomically rename it into place via `tempfile::NamedTempFile::persist()`. This ensures
no partial config file is ever observable by a concurrent reader or on crash/power-loss.
Naked `std::fs::write` directly to the config path is categorically forbidden and enforced
at PR merge by the semgrep rule `monocle-no-direct-config-write`.

## Preconditions

1. The caller has a `MonocleConfig` struct populated with the values to be persisted.
2. The config file path is resolved via `directories::ProjectDirs::from("", "", "monocle")`:
   `config_dir().join("config.json")`. If `ProjectDirs::from` returns `None`,
   `ConfigError::HomeUnresolvable` is returned before any write is attempted.
3. The parent directory of the config file path is writable by the current process.
4. The filesystem hosting the parent directory supports atomic `rename(2)` semantics
   (POSIX local filesystems; network filesystems may not guarantee this, but monocle
   only targets local home directories).

## Postconditions

1. The parent directory exists after the write: `std::fs::create_dir_all(parent)` is called
   before the `NamedTempFile` is created. If directory creation fails, `ConfigError::Io(e)`
   is returned and no write is attempted.
2. The `NamedTempFile` is created within the same directory as the target config file
   (`NamedTempFile::new_in(parent)`) — NOT in `/tmp` or the OS default temp directory.
   This guarantees the subsequent `rename(2)` is on the same filesystem and therefore atomic.
3. The complete serialized JSON of `MonocleConfig` (produced via `serde_json::to_string_pretty`)
   is written to the temp file via `tmp.write_all(json.as_bytes())`.
4. The temp file is flushed (`tmp.flush()`) before `persist()` is called.
5. `tmp.persist(path)` is called to atomically rename the temp file to the target config path.
   On success, the config path contains the newly written config and the old content (if any) is
   replaced atomically. On failure, `ConfigError::PersistFailed(e.error)` is returned.
6. On any error, the temp file is cleaned up by the `NamedTempFile` drop impl. No partial content
   is left at the target config path.
7. The function `write_config()` does NOT call `std::fs::write()`, `OpenOptions::create(true).write(true).open(path)`,
   or any other direct-write variant targeting the canonical config path. This property
   is verified by the semgrep rule `monocle-no-direct-config-write` at PR merge time.

## Invariants

1. At no point in time is the config file in a partially-written or zero-byte state observable
   by a concurrent reader. The atomic rename guarantees the file transitions from the old
   complete content to the new complete content with no intermediate state.
2. If `write_config()` returns an `Err`, the previous config file content is unchanged.
   The rename either completes fully or not at all.
3. The temp file path is always within the same directory as the config file, never in
   a cross-filesystem location. This invariant is enforced by `NamedTempFile::new_in(parent)`.
4. `#[forbid(unsafe_code)]` is declared at the `monocle-config` crate root. No unsafe code
   participates in the write path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-070 | Parent directory does not exist on first write | `create_dir_all(parent)` creates it (including all missing ancestors); write proceeds normally |
| EC-071 | Parent directory creation fails (e.g., permission denied) | `ConfigError::Io(e)` returned; no temp file created; no partial state |
| EC-072 | Process crashes between `tmp.flush()` and `tmp.persist()` | Temp file is left as an orphan in the parent directory (OS cleans up on next `NamedTempFile` drop or reboot); target config path is unchanged (old content or absent — as before) |
| EC-073 | `tmp.persist()` fails because target path is on a read-only filesystem | `ConfigError::PersistFailed(e.error)` returned; temp file dropped and cleaned up; config path unchanged |
| EC-074 | Config path has no parent component (e.g., a bare filename with no directory) | `ConfigError::InvalidPath` returned at the `path.parent().ok_or(ConfigError::InvalidPath)?` check before any I/O |
| EC-075 | Concurrent write from two `monocle` processes to the same config path | Last `persist()` wins atomically; intermediate states are never visible; no corruption |
| EC-076 | `serde_json::to_string_pretty` fails on `MonocleConfig` (structurally impossible given `MonocleConfig`'s Rust representation, but the return type is `Result`) | `ConfigError::Serialize(e)` returned; no I/O attempted |
| EC-077 | `HomeUnresolvable` — `ProjectDirs::from("", "", "monocle")` returns `None` | `ConfigError::HomeUnresolvable` returned; no write attempted |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `write_config(&default_config)` to a writable temp dir | Config file created at target path; content is valid JSON with `schema_version: 1`; no temp file orphan left | happy-path |
| `write_config(&default_config)` when parent dir absent | Parent dir created; config file written; no error | happy-path |
| `write_config(&default_config)` to a read-only parent dir | `Err(ConfigError::Io(_))` or `Err(ConfigError::PersistFailed(_))`; no corruption at target path | error |
| `write_config` → read back via `serde_json::from_str` | Round-trip: deserialized config equals original struct | happy-path |
| Stat target path during `write_config` via concurrent reader | Reader sees either old complete content or new complete content; never zero-byte or partial | invariant |
| Verify no `std::fs::write` call in `monocle-config` source | `cargo semgrep -r monocle-no-direct-config-write monocle-config/` reports 0 findings | static-analysis |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `write_config` produces a valid, complete JSON file at the target path | integration (tempfile::TempDir) |
| VP-TBD | Atomic rename semantics: target transitions from old complete to new complete atomically | integration (concurrent reader thread) |
| VP-TBD | Parent directory creation on first write | integration (tempfile::TempDir, no pre-existing dir) |
| VP-TBD | Error path: no partial content at target on persist failure | integration (read-only dir fixture) |
| VP-TBD | Semgrep rule `monocle-no-direct-config-write` catches naked `std::fs::write` to config path | static-analysis (semgrep CI gate) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability §SS-07 |
| Capability Anchor Justification | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability — this BC specifies the atomic write contract that is the correctness foundation of the entire config persistence layer; without it, any config-writing operation risks data loss |
| L2 Domain Invariants | No domain-spec/invariants.md exists for this project; authority is ARCH-INDEX §SS-07 and SS-config.md §Atomic Write Contract |
| Architecture Module | monocle-config (config.json reader/writer, harness profile schema, profile picker logic) per ARCH-INDEX Subsystem Registry SS-07 |
| Architecture Source | SS-config.md v1.4.0 §Atomic Write Contract |
| Cross-Ref | BC-2.07.002 (schema written by write_config); BC-2.07.003 (read path that relies on atomic writes to never see partial content); BC-2.01.005 (same tempfile::persist pattern for lock file) |
| Brief Features | F-53 (monocle-config reads/writes config.json via tempfile::persist), F-58 (monocle-config atomic write requirement) |
| Test File | `monocle-config/tests/atomic_write.rs` |
| Test Name | `test_BC_2_07_001_config_atomic_write` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.07.002] — composes with: the schema serialized by write_config is defined in BC-2.07.002
- [BC-2.07.003] — depends on: the read path trusts atomic writes will never produce a partial file
- [BC-2.01.005] — composes with: the same tempfile::persist pattern governs the lock file write

## Architecture Anchors

- `architecture/SS-config.md#atomic-write-contract` — write_config implementation pattern, rationale, and semgrep enforcement
- `architecture/SS-conventions-anti-patterns.md` — semgrep rule `monocle-no-direct-config-write` definition

## Story Anchor

S-TBD — Implement monocle-config crate: atomic write, schema v1, default handling (filled by story-writer)

## VP Anchors

VP-TBD — config atomic write integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- Created as new artifact for SS-07 (Config subsystem) per `prd-expansion-scope.md` §3.4 BC-2.07.001
  and `SS-config.md` §Atomic Write Contract.
- Grounded in SS-config.md v1.1.0 §Atomic Write Contract (write pattern, semgrep enforcement, why atomic matters).
- Brief features traced: F-53, F-58.
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact; no prior chain).


## §Trace v1.1.2

**Architecture Source pin cascade: SS-config.md v1.3.0→v1.4.0** (2026-06-20):
- Architecture Source: `SS-config.md v1.3.0` → `SS-config.md v1.4.0`.
- No body propagation required: §Atomic Write Contract is unchanged in v1.4.0.
- SE-16d monotonicity: 2026-06-20 > v1.1.1 date 2026-05-29. PASS.

## §Trace v1.1.1

**F-S025-ADV23-MED-001 Category 8 sweep — Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-config.md v1.1.0` → `SS-config.md v1.3.0` (active pointer was stale by 2 minor versions).
- No substantive BC body prose propagation required: SS-config.md §Atomic Write Contract is unchanged in v1.2.0 and v1.3.0. The v1.2.0 change affected `binding_overrides` serde default (not this BC's scope). The v1.3.0 change corrected §Missing or Corrupted Config Handling error-variant list (BC-2.07.003 scope, not this BC).
- SE-16d monotonicity: v1.1.1 timestamp 2026-05-29T00:00:00Z > v1.1.0 timestamp 2026-05-26T14:00:00Z. PASS.

## §Trace v1.1.0

**F-P1D3-003 HIGH — Deprecated config path replaced with platform-canonical form** (2026-05-26T14:00:00Z):
- Description: `~/.monocle/config.json` → `<config_dir>/config.json` (the platform-dependent
  path resolved via `directories::ProjectDirs::config_dir()`).
- Rationale: SS-config.md v1.1.0 §Config File Location §F-P1D-005 fix established that
  `~/.monocle/config.json` is NOT the canonical path on any supported platform (macOS uses
  `~/Library/Application Support/monocle/config.json`; Linux uses
  `~/.config/monocle/config.json`). Using `<config_dir>/config.json` with a pointer to
  §Config File Location is the correct canonical reference form per SS-config.md v1.1.0.
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-26T14:00:00Z > v1.0.1. PASS.

## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-config.md v1.0.0` → `SS-config.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.