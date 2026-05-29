---
document_type: behavioral-contract
level: L3
version: "1.0.2"
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
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.07.003: Config Missing or Corrupted: Default Applied

## Description

`monocle-config`'s `load_config()` function handles absent and corrupted config files
gracefully. A missing config file is the expected state on first run and returns
`MonocleConfig::default()` without error. A parse-failed config file (corrupted by an
interrupted write from a non-monocle process or a development build) also returns
`MonocleConfig::default()` — accompanied by a structured `tracing::warn!` log — rather
than blocking daemon startup. Only an I/O error (e.g., permission denied on a file that
does exist) propagates as `Err(ConfigError::Io(e))`. No panic occurs under any of these
conditions.

## Preconditions

1. `load_config()` is called (typically during daemon startup or TUI startup).
2. The config file path is resolved via `directories::ProjectDirs::from("", "", "monocle")`.
   If resolution returns `None`, `Err(ConfigError::HomeUnresolvable)` is returned — this
   is the only pre-I/O error path.
3. The caller has not pre-checked whether the config file exists; `load_config()` is
   responsible for all existence and parse-error handling.

## Postconditions

**Case 1: File does not exist (first run or deliberate deletion):**
1. `std::fs::read_to_string(path)` returns `Err(io::ErrorKind::NotFound)`.
2. `load_config()` returns `Ok(MonocleConfig::default())`. No error is propagated. No
   log is emitted (missing file on first run is expected and silent).

**Case 2: File exists but fails to read (I/O error — e.g., permission denied, EIO):**
3. `std::fs::read_to_string(path)` returns `Err(e)` where `e.kind() != ErrorKind::NotFound`.
4. `load_config()` returns `Err(ConfigError::Io(e))`. The error propagates to the caller.
5. The daemon startup procedure logs the error and continues with `MonocleConfig::default()`
   as a fallback (the daemon does not panic; it degrades gracefully).
6. The TUI renders a warning in the status bar: "Config unreadable — running with defaults".

**Case 3: File exists and reads successfully but fails JSON parse:**
7. `serde_json::from_str::<MonocleConfig>(&content)` returns `Err(_)`.
8. `load_config()` emits `tracing::warn!("config.json parse failed; using defaults: {:?}", e)`.
9. `load_config()` returns `Ok(MonocleConfig::default())`. The parse error is NOT propagated
   as an `Err` — it is swallowed after logging.
10. Rationale: a corrupted config (e.g., written by a killed non-atomic process) should not
    prevent daemon startup. The user's harness profiles are lost in this recovery scenario,
    but monocle remains operational. A future Phase 2 feature may offer config repair.

**Case 4: File exists and reads and parses successfully:**
11. `load_config()` returns `Ok(config)` where `config` is the deserialized `MonocleConfig`.

**Across all cases:**
12. `load_config()` does NOT panic under any condition. All error variants are handled
    and returned as `Result` or swallowed per the rules above.
13. `load_config()` returns `Err` only for `ConfigError::HomeUnresolvable` (path resolution
    failure) and `ConfigError::Io(e)` where `e.kind() != ErrorKind::NotFound` (readable
    failure). All other error conditions produce `Ok(MonocleConfig::default())`.

## Invariants

1. `load_config()` never panics. The function signature `pub fn load_config() -> Result<MonocleConfig, ConfigError>`
   guarantees all error states are returned, not unwrapped.
2. A missing config file is not an error — it is the correct first-run state. The caller
   MUST NOT treat `Ok(MonocleConfig::default())` from a missing-file path as an error.
3. A parse-failed config file is always recovered to `MonocleConfig::default()`. The parse
   error is logged but not returned. The caller sees the same `Ok(MonocleConfig::default())`
   as Case 1.
4. The distinction between Case 1 (missing) and Case 3 (parse-failed) is not visible to the
   caller via the `Result` return type. It is visible only via the presence or absence of a
   `tracing::warn!` log line.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-088 | Config file is absent (first run) | `Ok(MonocleConfig::default())` returned; no log emitted |
| EC-089 | Config file is zero bytes | JSON parse fails (`EOF`); `tracing::warn!` emitted; `Ok(MonocleConfig::default())` returned |
| EC-090 | Config file contains valid JSON but not a JSON object (e.g., `[]` or `"string"`) | `serde_json` deserialize into `MonocleConfig` fails; `tracing::warn!` emitted; `Ok(MonocleConfig::default())` returned |
| EC-091 | Config file contains valid JSON but `schema_version` field is missing | `serde_json` parse error (required field absent); `tracing::warn!` emitted; `Ok(MonocleConfig::default())` returned |
| EC-092 | Config file contains valid JSON, `schema_version: 1`, but `harness_profiles` has a malformed element (missing required `id` field) | `serde_json` parse error on the profiles array; `tracing::warn!` emitted; `Ok(MonocleConfig::default())` returned (entire config reset, not partial) |
| EC-093 | Config file has `permission denied` at OS level (file exists, mode 0o000) | `ConfigError::Io(e)` returned; daemon logs and continues with defaults; TUI shows "Config unreadable" warning |
| EC-094 | Config file path's parent directory exists but config file is a directory (edge case: `~/.monocle/config.json/`) | `std::fs::read_to_string` returns `Err(IsADirectory)` (kind: Other or Os-specific); treated as I/O error → `ConfigError::Io(e)` |
| EC-095 | `ProjectDirs::from("", "", "monocle")` returns `None` (no resolvable home dir) | `ConfigError::HomeUnresolvable` returned before any I/O; no file access attempted |
| EC-096 | Config file written by a future Phase 2 binary with additional required fields unknown to Phase 1 | If Phase 2 schema drops `schema_version` default or adds non-default required fields, Phase 1 sees a parse error → `tracing::warn!` + default; forward-compat graceful degradation |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Config file absent (TempDir with no config.json) | `Ok(MonocleConfig::default())`; no panic; no log | happy-path (first run) |
| Config file is valid JSON with schema_version 1 and one harness profile | `Ok(config)` with profile populated | happy-path |
| Config file is zero bytes | `Ok(MonocleConfig::default())`; `tracing::warn!` emitted | edge-case |
| Config file contains `{"not_valid_json": ` (truncated) | `Ok(MonocleConfig::default())`; `tracing::warn!` emitted | edge-case |
| Config file has mode 0o000 (permission denied) | `Err(ConfigError::Io(e))` where `e.kind() == PermissionDenied` | error |
| `ProjectDirs` returns None (simulated in test) | `Err(ConfigError::HomeUnresolvable)` | error |
| Config file contains `{}` (empty JSON object, no schema_version) | `Ok(MonocleConfig::default())`; `tracing::warn!` emitted (schema_version required field missing) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Missing config file → `Ok(MonocleConfig::default())` without panic | integration (TempDir, absent file) |
| VP-TBD | Zero-byte config file → `Ok(MonocleConfig::default())` + `tracing::warn!` | integration (TempDir, empty file) |
| VP-TBD | Valid config round-trips through `load_config` | integration (write then load) |
| VP-TBD | Permission-denied config file → `Err(ConfigError::Io(_))` | integration (chmod 0o000 fixture) |
| VP-TBD | No panic under any of the four error cases above | integration (assert no unwrap panic via std::panic::catch_unwind) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability §SS-07 |
| Capability Anchor Justification | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability — this BC specifies the resilience contract of the config load path, which is the precondition for all harness profile management and picker behavior |
| L2 Domain Invariants | No domain-spec/invariants.md exists for this project; authority is ARCH-INDEX §SS-07 and SS-config.md §Missing or Corrupted Config Handling |
| Architecture Module | monocle-config (config.json reader/writer, harness profile schema, profile picker logic) per ARCH-INDEX Subsystem Registry SS-07 |
| Architecture Source | SS-config.md v1.3.0 §Missing or Corrupted Config Handling and §Error Taxonomy |
| Cross-Ref | BC-2.07.001 (atomic write ensures partial files are minimized); BC-2.07.002 (schema parse errors governed by Case 3 here) |
| Brief Features | F-53 (monocle-config reads config.json gracefully) |
| Test File | `monocle-config/tests/load_config_resilience.rs` |
| Test Name | `test_BC_2_07_003_missing_config_default` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.07.001] — depends on: atomic write minimizes the window where Case 3 (parse failure) can occur
- [BC-2.07.002] — depends on: the schema definition determines what constitutes a parse failure

## Architecture Anchors

- `architecture/SS-config.md#missing-or-corrupted-config-handling` — four-case decision tree, error taxonomy
- `architecture/SS-config.md#error-taxonomy` — `ConfigError` variants and their semantics

## Story Anchor

S-TBD — Implement monocle-config crate: atomic write, schema v1, default handling (filled by story-writer)

## VP Anchors

VP-TBD — config load resilience integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- Created as new artifact for SS-07 (Config subsystem) per `prd-expansion-scope.md` §3.4 BC-2.07.003
  and `SS-config.md` §Missing or Corrupted Config Handling and §Error Taxonomy.
- Four-case decision tree (missing / I/O error / parse failure / success) fully specified.
- Brief feature traced: F-53.
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact; no prior chain).


## §Trace v1.0.2

**F-S025-ADV23-MED-001 Category 8 sweep — Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-config.md v1.1.0` → `SS-config.md v1.3.0` (active pointer was stale by 2 minor versions).
- No substantive BC body prose propagation required: this BC's Postcondition 13 already correctly specifies both `ConfigError::HomeUnresolvable` and `ConfigError::Io(e)` as the two Err variants — this matches the v1.3.0 correction to SS-config.md §Missing or Corrupted Config Handling. The BC was written correctly even when pointing to v1.1.0; no body changes needed.
- SE-16d monotonicity: v1.0.2 timestamp 2026-05-29T00:00:00Z > v1.0.1. PASS.

## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-config.md v1.0.0` → `SS-config.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.