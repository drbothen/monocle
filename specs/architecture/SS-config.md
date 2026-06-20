---
document_type: architecture-section
level: L3
section: "config"
subsystem: SS-07
version: "1.4.0"
status: complete
producer: architect
phase: phase-1-expansion
timestamp: 2026-06-20T00:00:00Z
inputs:
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/product-brief.md, version: "1.4.30"}
  - {path: .factory/specs/architecture/SS-conventions-anti-patterns.md, version: "1.30.2"}
input-hash: "[pending]"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: Config

## [Section Content]

## Scope

SS-07 defines the `monocle-config` crate: the configuration persistence layer for
monocle. This crate is responsible for reading and writing `<config_dir>/config.json`
(see §Config File Location for per-platform resolution), managing harness profiles
(id, binary path, display name, optional config dir), CCR binary detection, binding
override stubs, and the sticky-per-project profile selection map.

`monocle-config` is a pure synchronous library crate. It has no async runtime
dependency, no Tokio, and no background threads. All reads and writes are blocking
`std::fs` calls wrapped in the atomic write contract (see §Atomic Write Contract
below). Both the daemon binary and the TUI binary link against `monocle-config`
directly — the crate must remain usable from both contexts without circular
dependencies.

The crate carries `#[forbid(unsafe_code)]` in its crate root.

---

## Config File Location

The config file path is **platform-dependent**. The canonical reference form used in
BCs and user-facing documentation is `<config_dir>/config.json`, where `<config_dir>`
resolves via `directories::ProjectDirs::config_dir()` at runtime. There is no single
universal canonical path; `~/.monocle/config.json` is NOT the canonical path — it is
a shorthand that was used in early docs and is incorrect on all supported platforms.

Resolution is performed via `directories::ProjectDirs::from("", "", "monocle")`,
which returns a `ProjectDirs` struct. The config file path is constructed as:

```
ProjectDirs::config_dir() / "config.json"
```

Platform-specific expansions of `<config_dir>`:

| Platform | `<config_dir>` expansion | Full config path |
|----------|--------------------------|-----------------|
| macOS    | `~/Library/Application Support/monocle/` | `~/Library/Application Support/monocle/config.json` |
| Linux    | `~/.config/monocle/` (XDG-compliant) | `~/.config/monocle/config.json` |
| Windows  | `%APPDATA%\monocle\` | `%APPDATA%\monocle\config.json` |

The `directories` crate handles platform selection at runtime; no platform guards are
required in `monocle-config` code. Any documentation that previously wrote
`~/.monocle/config.json` as the "canonical path" was incorrect and must be updated to
use the `<config_dir>/config.json` shorthand with a note pointing to this table.

### Resolution Strategy

1. Call `directories::ProjectDirs::from("", "", "monocle")`.
2. If `ProjectDirs::from` returns `None` (home directory unresolvable), return
   `ConfigError::HomeUnresolvable`.
3. Construct config path: `project_dirs.config_dir().join("config.json")`.
4. Ensure the parent directory exists (`std::fs::create_dir_all`) before any write.
   Read operations do not require the directory to exist (missing file → default config).

---

## Config Schema v1

The `config.json` file uses a flat JSON object with a mandatory `schema_version` field
as the first key. The full schema for v1:

```json
{
  "schema_version": 1,
  "harness_profiles": [
    {
      "id": "string",
      "display_name": "string",
      "binary_path": "string",
      "config_dir": "string | null"
    }
  ],
  "ccr_path": "string | null",
  "binding_overrides": {},
  "project_profiles": {
    "/absolute/path/to/project": "profile-id"
  },
  "pty_scrollback_rows": 1000
}
```

### Field Definitions

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `schema_version` | `u32` | yes | — | Always `1` for this schema. Enables future migrations. |
| `harness_profiles` | `Vec<HarnessProfile>` | yes | `[]` | List of registered harness profiles. |
| `harness_profiles[].id` | `String` | yes | — | Stable unique identifier for this profile (used as key in `project_profiles` map). |
| `harness_profiles[].display_name` | `String` | yes | — | Human-readable name rendered in the profile picker. |
| `harness_profiles[].binary_path` | `String` | yes | — | Absolute path to the harness binary (e.g., `/usr/local/bin/claude`). |
| `harness_profiles[].config_dir` | `Option<String>` | no | `null` | Optional path to the harness config directory. If `null`, the harness's own default config dir is used. |
| `ccr_path` | `Option<String>` | no | `null` | Absolute path to the `ccr` binary. `null` triggers PATH fallback search (see §CCR Detection). |
| `binding_overrides` | `serde_json::Value` (object) | no | `{}` | Stub for future binding customizations. Stored as an opaque JSON object; not parsed by Phase 1 code beyond round-trip. |
| `project_profiles` | `HashMap<String, String>` | no | `{}` | Maps project directory absolute paths to profile IDs. Populated by the profile picker on selection. |
| `pty_scrollback_rows` | `Option<u32>` | no | `null` (→ 1000) | Scrollback buffer row count for `vt100::Parser`. Absent or `null` applies the 1000-row default. Values above 10000 are clamped to 10000; values of 0 or below are clamped to 1. **Schema shape is SS-07's concern; default/clamp semantics are BC-2.09.007's concern; config load wiring is S-039's responsibility; S-043 consumes the already-loaded value via `App::scrollback_rows`.** |

### Rust Representation

```rust
/// Returns an empty JSON object (`{}`), used as the serde default for `binding_overrides`.
///
/// `serde_json::Value::default()` returns `Value::Null`, not `Value::Object(Map::new())`.
/// Using a named default function guarantees that absent or null `binding_overrides` fields
/// always deserialize to an empty object, consistent with Invariant 3 and the JSON schema
/// where `binding_overrides` is documented as `{}`.
fn default_binding_overrides() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonocleConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub harness_profiles: Vec<HarnessProfile>,
    #[serde(default)]
    pub ccr_path: Option<String>,
    #[serde(default = "default_binding_overrides")]
    pub binding_overrides: serde_json::Value,
    #[serde(default)]
    pub project_profiles: HashMap<String, String>,
    /// Scrollback buffer row count for `vt100::Parser`.
    ///
    /// `None` (absent from JSON) → caller applies the 1000-row default.
    /// Values above 10000 are clamped to 10000; values of 0 are clamped to 1.
    /// `#[serde(default)]` on `Option<u32>` deserializes to `None` when the
    /// field is absent — this is correct because `Option::default()` is `None`.
    ///
    /// Default/clamp semantics: BC-2.09.007 (SS-09).
    /// Config load wiring: S-039 (`App::scrollback_rows` initialization).
    /// Consumer: S-043 reads `App::scrollback_rows` (already loaded by S-039).
    #[serde(default)]
    pub pty_scrollback_rows: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessProfile {
    pub id: String,
    pub display_name: String,
    pub binary_path: String,
    #[serde(default)]
    pub config_dir: Option<String>,
}
```

`MonocleConfig` derives `Default`:

```rust
impl Default for MonocleConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            harness_profiles: vec![],
            ccr_path: None,
            binding_overrides: serde_json::Value::Object(serde_json::Map::new()),
            project_profiles: HashMap::new(),
            pty_scrollback_rows: None,
        }
    }
}
```

---

## Atomic Write Contract

**All writes to `config.json` MUST use `tempfile::persist`.**

The pattern:

```rust
use tempfile::NamedTempFile;
use std::io::Write as _;

fn write_config(path: &Path, config: &MonocleConfig) -> Result<(), ConfigError> {
    let parent = path.parent().ok_or(ConfigError::InvalidPath)?;
    std::fs::create_dir_all(parent)?;

    let mut tmp = NamedTempFile::new_in(parent)?;
    let json = serde_json::to_string_pretty(config)?;
    tmp.write_all(json.as_bytes())?;
    tmp.flush()?;
    tmp.persist(path).map_err(|e| ConfigError::PersistFailed(e.error))?;
    Ok(())
}
```

The temp file is created in the same directory as the target (`new_in(parent)`) to
guarantee the rename is on the same filesystem, making `persist()` (which calls
`rename(2)`) atomic on POSIX systems.

**Naked `std::fs::write` to the config path is forbidden.** This prohibition is
enforced at PR merge by the semgrep rule `monocle-no-direct-config-write` (defined
in `SS-conventions-anti-patterns.md`). BC-2.07.001 provides the testable behavioral
contract for this requirement.

### Why Atomic Write Matters for Config

A crash or power loss between the `open()` and `close()` of a direct `std::fs::write`
leaves a zero-byte or partially-written config file. On next startup, `monocle-config`
would encounter a parse error and return the default config — silently discarding all
harness profiles the user had configured. The `tempfile::persist` pattern eliminates
this window: either the old config remains (rename never executed) or the new config
is in place (rename completed atomically).

---

## Async Integration (F-P1D-010)

`monocle-config` is a synchronous library. All I/O calls (`load_config`, `write_config`,
`detect_ccr`) are blocking `std::fs` operations. This is intentional: config reads are
rare (daemon startup, TUI startup, profile picker selection) and the files are small
(typically <4 KiB), so blocking I/O is acceptable in synchronous contexts.

**However**, callers from async Tokio contexts (`monocle-tui`, `monocle-runtime`) MUST
NOT call blocking `monocle-config` functions directly on the Tokio runtime thread. Doing
so blocks the async executor, violating the Tokio runtime contract and potentially
causing timer skew or deadlock under load.

### Required Pattern for Async Callers

```rust
// In monocle-tui or monocle-runtime (async context):

// CORRECT: offload blocking I/O to the blocking thread pool
let config = tokio::task::spawn_blocking(|| load_config())
    .await??; // outer ? for JoinError, inner ? for ConfigError

// CORRECT: same pattern for writes
let updated_config = config.clone_with_profile_update(...);
tokio::task::spawn_blocking(move || write_config(&path, &updated_config))
    .await??;

// WRONG: direct call blocks the Tokio runtime thread
let config = load_config()?;  // DO NOT DO THIS in async context
```

`tokio::task::spawn_blocking` moves the blocking call to Tokio's dedicated blocking
thread pool (separate from the async worker threads). The async task awaits the result
without blocking the executor. This pattern is mandatory for all `monocle-config` I/O
calls from async code. Violations are a blocking defect in PR review.

**Daemon startup** (a non-async context via the pre-server setup code in `main.rs`)
may call `load_config()` directly without `spawn_blocking` — the daemon start sequence
executes synchronously before `run_server()` starts the async executor.

## Missing or Corrupted Config Handling

When `load_config()` is called:

1. **File does not exist:** Return `MonocleConfig::default()` with no error. This is
   the expected state on first run.
2. **File exists but fails to read (I/O error):** Return `ConfigError::Io(e)`. Callers
   should surface this to the user (daemon startup: log + continue with default;
   TUI: render warning in status bar).
3. **File exists but fails JSON parse:** Return `MonocleConfig::default()` with a
   logged warning. Rationale: a corrupted config (e.g., interrupted write by a
   non-monocle process) should not prevent daemon startup. The user's profiles are
   lost in this case but monocle remains operational. A future Phase 2 feature may
   offer config repair.
4. **File exists, parses successfully:** Return the deserialized `MonocleConfig`.

**No panics under any of these conditions.** The `load_config()` function signature is:

```rust
pub fn load_config() -> Result<MonocleConfig, ConfigError>
```

`load_config` returns `Err` for two cases only:

- `ConfigError::Io` — filesystem error reading an existing config file (e.g., permission
  denied). Missing file is NOT an I/O error here; it returns `Ok(default)` (case 1 above).
- `ConfigError::HomeUnresolvable` — `ProjectDirs::from("", "", "monocle")` returns `None`
  because the platform home directory cannot be resolved (e.g., `$HOME` unset in a systemd
  `User=` unit). The config file path cannot be constructed without a home directory; this
  is a hard failure distinct from a missing or corrupt file.

Parse errors (case 3) return `Ok(MonocleConfig::default())` — they do not propagate as `Err`.

---

## Profile Picker Logic

### Sticky-Per-Project Selection (BC-2.07.004)

When monocle starts in TUI mode, it determines the active harness profile via:

1. Read the current working directory (or the project root if monocle is launched
   with an explicit project path argument).
2. Look up the directory's absolute path in `config.project_profiles`.
3. If a matching profile ID is found, load that profile without showing the picker.
4. If no match, and `harness_profiles` is non-empty, show the profile picker.
5. If `harness_profiles` is empty, render the TUI with a "No profiles configured"
   notice.

The sticky selection is per-directory-path. Symlinks are not resolved before lookup
(the path used during last selection is the path stored in the map).

### `Ctrl-P` Override (BC-2.07.005)

Pressing `Ctrl-P` in any `AppMode` opens the profile picker overlay regardless of
sticky selection. The picker is rendered as a modal over the current TUI view.

When the user selects a profile from the picker:

1. Update `config.project_profiles[current_dir] = selected_profile_id`.
2. Write the updated config atomically via `tempfile::persist`.
3. Apply the new profile to the running daemon session (the daemon reloads the
   active harness profile from the config on each session launch, not at daemon
   startup — so a picker change takes effect for new sessions without daemon restart).

The profile picker itself is rendered by the TUI (`monocle-tui`). The persistence
logic (steps 1–2) is performed by `monocle-config`. This is the clean separation:
`monocle-config` owns the read/write contract; `monocle-tui` owns the UI rendering.

---

<a id="ccr-detection-bc-2-07-006"></a>
## CCR Detection (BC-2.07.006)

Claude Code Router (`ccr`) is an integrate-external dependency (D-010 in
`product-brief.md`): monocle detects it on PATH, writes per-session JSON, and
sets `ANTHROPIC_BASE_URL`. No CCR API changes are required or expected.

### Detection Algorithm

```rust
pub fn detect_ccr(config: &MonocleConfig) -> Option<PathBuf> {
    // Step 1: explicit config path
    if let Some(ref path_str) = config.ccr_path {
        let path = PathBuf::from(path_str);
        if path.is_file() {
            return Some(path);
        }
        // Configured path does not exist — fall through to PATH search
        // (log a warning; do not error, to avoid blocking startup)
    }

    // Step 2: PATH search
    which::which("ccr").ok()
}
```

The `which` crate (already in `SS-deps-pin-manifest.md`; add if not present) is
used for PATH search. If neither path resolves, `detect_ccr` returns `None` and
the TUI status bar renders "CCR: not found" without blocking other functionality.

The detection result is surfaced in the TUI status bar (BC-2.07.006). It is NOT
cached at startup — it is re-evaluated each time `detect_ccr` is called, so that
a user who installs `ccr` while monocle is running sees the updated status on the
next status bar refresh cycle (typically ≤1 second).

---

## Forward Compatibility

### Schema Version Field

The `schema_version: 1` field is mandatory and is the first key serialized. Its
purpose is to enable future schema migrations. The migration strategy for Phase 2+:

1. Read `schema_version` from the raw `serde_json::Value` before full deserialization.
2. If `schema_version == 1`, deserialize as `MonocleConfig` (current struct).
3. If `schema_version > 1`, delegate to the appropriate migration path (Phase 2+
   concern; not implemented in Phase 1).
4. If `schema_version` is missing, treat as `schema_version: 1` (pre-versioning
   config written by a development build).

### Unknown Fields

Unknown fields in `config.json` are silently ignored during deserialization. This
is achieved by NOT using `#[serde(deny_unknown_fields)]` on `MonocleConfig` or
`HarnessProfile`. A Phase 2 build that writes a new field to `config.json` will
have that field ignored — but preserved — when the same file is read by a Phase 1
binary. Preservation requires that the unknown fields be round-tripped via a
`serde_json::Value` extra-fields capture (if needed) or simply accepted and discarded
(Phase 1 default — no round-trip of unknown fields is required in Phase 1).

Rationale: a user who downgrades from Phase 2 back to Phase 1 should not encounter
a parse error. Silent ignore is correct here.

---

## Behavioral Contracts

Six behavioral contracts govern SS-07, all in the BC-2.07.NNN namespace:

| BC ID | Title | Priority | Key Invariant |
|-------|-------|----------|---------------|
| BC-2.07.001 | Config File Atomic Write via `tempfile::persist` | P0 | All writes use `tempfile::persist`; naked `std::fs::write` to config path is forbidden and semgrep-enforced |
| BC-2.07.002 | Config Schema Version 1: Harness Profile Fields | P0 | `config.json` carries `schema_version: 1`, `harness_profiles`, `ccr_path`, `binding_overrides`, `pty_scrollback_rows`; unknown fields ignored |
| BC-2.07.003 | Config Missing or Corrupted: Default Applied | P0 | Missing file → `MonocleConfig::default()`; parse failure → `MonocleConfig::default()`; no panic |
| BC-2.07.004 | Profile Picker: Sticky-Per-Project | P1 | Last-used profile for a directory is pre-selected on next launch; no picker shown if sticky match found |
| BC-2.07.005 | Profile Picker: `Ctrl-P` Override Shows Picker | P1 | `Ctrl-P` in any AppMode opens picker; selection persists to `config.json` for current directory |
| BC-2.07.006 | CCR Detection via `ccr_path` Config Field | P1 | `ccr_path` checked first; PATH fallback if unset or path missing; detection result surfaced in TUI status bar |

---

## Dependency Graph

`monocle-config` has a minimal, deliberate dependency surface. No async runtime.

```
monocle-config
  ├── serde             (derive Serialize/Deserialize)
  ├── serde_json        (JSON read/write; Value for binding_overrides)
  ├── directories       (ProjectDirs for config file path resolution)
  ├── tempfile          (NamedTempFile::new_in + persist for atomic write)
  ├── thiserror         (ConfigError type definition)
  └── which             (PATH search for CCR binary detection)
```

### Consumers

| Crate | Usage |
|-------|-------|
| `monocle-runtime` (daemon) | Reads harness profiles on session launch; reads CCR path on hook tmpfile generation |
| `monocle-tui` | Reads sticky profile on startup; writes profile picker selection; reads CCR detection result for status bar |
| `monocle` (binary) | Reads config during daemon auto-start to determine which harness to launch |

`monocle-config` does NOT depend on:
- `monocle-core` (no core types needed; avoids circular dep risk)
- `monocle-runtime` (no runtime dep; config is consumed by runtime, not the other way around)
- `tokio` or any async runtime (all I/O is synchronous; config reads are fast enough that blocking is acceptable)

---

## Error Taxonomy

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("home directory is unresolvable on this platform")]
    HomeUnresolvable,

    #[error("I/O error accessing config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to serialize config to JSON: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("failed to atomically persist config file: {0}")]
    PersistFailed(std::io::Error),

    #[error("config path has no parent directory")]
    InvalidPath,
}
```

`load_config` returns `Ok(MonocleConfig::default())` for both missing-file and
parse-error conditions. Two `Err` variants can propagate: `ConfigError::Io` (for
unreadable files, e.g., permission denied) and `ConfigError::HomeUnresolvable` (when
`ProjectDirs::from()` returns `None`, making the config path unresolvable). Parse errors
are logged via `tracing::warn` and swallowed. This design prevents a corrupted
`config.json` from blocking daemon startup.

---

## Purity Boundary

`monocle-config` is an effectful-shell crate: it performs filesystem I/O (reads and
writes to `<config_dir>/config.json`) and is therefore not formally verifiable in the
same sense as pure-core functions. However, the mutation logic (serialization,
deserialization, default construction) is purely functional and can be unit-tested
without touching the filesystem via in-memory `serde_json::from_str` / `to_string`
calls.

| Function | Classification | Notes |
|----------|---------------|-------|
| `MonocleConfig::default()` | Pure core | No I/O; deterministic |
| `serde_json::from_str::<MonocleConfig>()` | Pure core | No I/O; deterministic |
| `load_config()` | Effectful shell | Reads filesystem |
| `write_config()` | Effectful shell | Writes filesystem via `tempfile::persist` |
| `detect_ccr()` | Effectful shell | Reads filesystem + PATH |
| `resolve_profile_for_dir()` | Pure core | HashMap lookup; no I/O |

Unit tests cover the pure-core functions without filesystem setup. Integration tests
cover the effectful-shell functions using `tempfile::TempDir` for isolation.

---

## §Trace v1.4.0

**Schema extension: `pty_scrollback_rows` field added to MonocleConfig** (2026-06-20):
- Added `pty_scrollback_rows: Option<u32>` with `#[serde(default)]` to `MonocleConfig`.
- Added corresponding `"pty_scrollback_rows": 1000` example to §Config Schema v1 JSON example.
- Added field row to §Field Definitions table.
- Updated `MonocleConfig::default()` impl to include `pty_scrollback_rows: None`.
- Updated §Behavioral Contracts table BC-2.07.002 key-invariant column to enumerate `pty_scrollback_rows`.
- **Backward compatibility:** `Option<u32>` with `#[serde(default)]` is fully backward-compatible.
  Existing config.json files without `pty_scrollback_rows` parse correctly; `None` deserialization
  is correct because `Option::default()` is `None`. No `schema_version` bump required.
  New configs written by Phase 1 code will not emit `pty_scrollback_rows` unless the user sets it;
  Phase 1 omits `Some(1000)` because `None` is the default (application of default at call-site,
  not in the persisted file, is the correct approach — avoids polluting configs with redundant fields).
- **Ownership split:**
  - Schema shape (field presence, type, serde attributes): SS-07 / this document / BC-2.07.002.
  - Default value (1000) and clamp bounds (1..=10000): BC-2.09.007 (SS-09, behavioral owner).
  - Config load wiring (reading `config.pty_scrollback_rows`, applying default, clamping, storing in
    `App::scrollback_rows`): S-039 AC-008 responsibility.
  - Consumer: S-043 reads `App::scrollback_rows` (already loaded and clamped by S-039; does NOT
    re-load from config).
- Trigger: S-039 implementer could not wire AC-008 because `MonocleConfig` lacked the field.
  This is the corrective spec update that unblocks the config load wire.

## §Trace v1.3.0

**PR review finding** (F-P13-004) (2026-05-26):
- **F-P13-004** [HIGH] `load_config` error variant claim was incomplete — the §Missing or
  Corrupted Config Handling section stated "ConfigError::Io is the only error variant returned
  by load_config", which was incorrect. `load_config` also returns `ConfigError::HomeUnresolvable`
  when `ProjectDirs::from("", "", "monocle")` returns `None` (platform home directory not
  resolvable). This variant was already present in the §Error Taxonomy enum definition and in
  the §Resolution Strategy step 2, but the prose claim contradicted it. Fixed in two locations:
  (1) §Missing or Corrupted Config Handling — replaced "only error variant" claim with an
  explicit two-variant list (`ConfigError::Io` + `ConfigError::HomeUnresolvable`) with conditions;
  (2) §Error Taxonomy paragraph — updated to name both propagating variants.

## §Trace v1.2.0

**Bug fix** (F-P1D9-002) (2026-05-26):
- **F-P1D9-002** [BUG] `serde_json::Value::default()` returns `Value::Null`, not `Value::Object(Map::new())`. The previous annotation `#[serde(default)]` on `binding_overrides: serde_json::Value` would produce a `Null` value whenever `binding_overrides` was absent from the JSON — violating Invariant 3 (binding_overrides must always be an object) and diverging from `MonocleConfig::default()` which correctly initializes to `Value::Object(Map::new())`.
  - Changed `#[serde(default)]` to `#[serde(default = "default_binding_overrides")]` on the `binding_overrides` field.
  - Added `fn default_binding_overrides() -> serde_json::Value` returning `Value::Object(serde_json::Map::new())`. Added inline doc comment explaining why a named function is required (the generic `Value::default()` contract returns `Null`).
  - `MonocleConfig::default()` impl was already correct (`Value::Object(serde_json::Map::new())`); only the deserialization path required fixing.

## §Trace v1.1.0

**Adversarial review corrections** (F-P1D-005, F-P1D-010) (2026-05-26):
- **F-P1D-005** Config path contradiction resolved:
  - Removed the false claim that `~/.monocle/config.json` is the "canonical path".
  - The §Scope description now reads `<config_dir>/config.json` with a pointer to
    §Config File Location.
  - §Config File Location now opens with an explicit statement that the path is
    platform-dependent and that `~/.monocle/config.json` is NOT the canonical path.
  - Added a three-column platform table listing macOS, Linux, and Windows expansions
    with the full resulting path in the third column.
  - Added a note: any prior documentation that used `~/.monocle/config.json` as the
    "canonical path" was incorrect and must be updated to use `<config_dir>/config.json`.
  - §Purity Boundary path reference updated from `~/.monocle/config.json` to
    `<config_dir>/config.json` for consistency.
- **F-P1D-010** Added §Async Integration section specifying that callers from Tokio
  async contexts (monocle-tui, monocle-runtime) MUST wrap all `monocle-config` I/O
  calls in `tokio::task::spawn_blocking`. Includes a correct/incorrect code example.
  Direct calls to `load_config()` or `write_config()` from async tasks are a blocking
  defect in PR review. Exception documented: daemon startup (pre-server synchronous
  context) may call directly.

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- Created as new artifact covering SS-07 (Config subsystem) per `prd-expansion-scope.md`
  §Section 2 (SS-07 description) and §Section 3.4 (BC-2.07.NNN outline).
- Inputs read: `prd-expansion-scope.md` (SS-07 scope, 6 BCs), `product-brief.md`
  lines 146-153 (config scope) and 298-299 (CCR D-010 integrate-external constraint),
  `SS-daemon-lifecycle.md` (document structure conventions), `ARCH-INDEX.md`
  (subsystem registry conventions), `SS-conventions-anti-patterns.md`
  (`tempfile::persist` requirement).
- All six BC-2.07.NNN entries from the expansion scope document are covered in
  §Behavioral Contracts.
- CCR detection algorithm grounded in `product-brief.md` D-010 constraint: detect
  on PATH, no CCR API changes.
- Purity boundary classification applied: `load_config`, `write_config`, `detect_ccr`
  are effectful-shell; `default()`, JSON deserialization, `resolve_profile_for_dir`
  are pure-core.
- `#[forbid(unsafe_code)]` declared in §Scope; no unsafe code in this crate.
- No async dependencies; all I/O synchronous.
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact; no prior chain).
