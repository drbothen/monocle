---
document_type: story
level: L4
story_id: S-030
epic_id: EPIC-07
version: "1.1"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 5
wave: 4
tdd_mode: strict
priority: P0
depends_on: [S-001]
blocks: [S-025, S-031]
target_module: monocle-config
subsystems: [SS-07]
behavioral_contracts: [BC-2.07.001, BC-2.07.002, BC-2.07.003, BC-2.07.006]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-07/BC-2.07.001.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-07/BC-2.07.002.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-07/BC-2.07.003.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-07/BC-2.07.006.md, version: "1.0.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.07.001 (atomic write via tempfile::persist), BC-2.07.002 (config schema v1: harness profile fields), BC-2.07.003 (missing or corrupted config: default applied), BC-2.07.006 (CCR detection via ccr_path config field)"
---

# S-030: Config Crate Foundation — Schema, Load/Save, detect_ccr, Atomic Writes

## Narrative

As a TUI operator, I want the `monocle-config` crate to define the canonical
configuration schema, load configuration from disk (creating defaults on first run),
save atomically via `tempfile::persist`, and detect the Claude Code Runtime directory,
so that downstream stories (profile picker, TUI skeleton) have a correct, tested
config foundation to build upon.

## Acceptance Criteria

### AC-001 (traces to BC-2.07.001 postcondition PC-1 — MonocleConfig schema)
`monocle-config` exports `MonocleConfig` as a `#[derive(Debug, Clone, Serialize, Deserialize)]`
struct with fields:
- `schema_version: u32` — persisted to JSON; validated on load
- `active_profile: Option<String>` — `null` means "use default"
- `profiles: HashMap<String, ProfileConfig>` — keyed by profile name
- `binding_overrides: serde_json::Value` — always serializes as a JSON object (never null);
  requires `#[serde(default = "default_binding_overrides")]` where
  `fn default_binding_overrides() -> serde_json::Value { serde_json::Value::Object(serde_json::Map::new()) }`
  (plain `#[serde(default)]` produces `Value::Null`, which is incorrect)

### AC-002 (traces to BC-2.07.001 postcondition PC-2 — ProfileConfig)
`ProfileConfig` has fields: `name: String`, `ccr_path: Option<PathBuf>`,
`env_overrides: HashMap<String, String>`, `notes: Option<String>`.

### AC-003 (traces to BC-2.07.002 postcondition PC-1 — load() creates default on missing)
`MonocleConfig::load(path: &Path) -> Result<MonocleConfig, ConfigError>`:
- If the file does not exist, returns `Ok(MonocleConfig::default())` WITHOUT writing
  to disk. Writing the default happens only on the first explicit save.
- If the file exists but is not valid UTF-8 JSON, returns
  `Err(ConfigError::ParseError { path: path.to_owned(), source: serde_json::Error })`.

### AC-004 (traces to BC-2.07.002 postcondition PC-2 — schema_version validation)
On load, if `schema_version` does not equal `MonocleConfig::CURRENT_SCHEMA_VERSION` (= 1),
`load()` returns `Err(ConfigError::SchemaMismatch { found: u32, expected: u32 })`.
No silent migration occurs in Phase 3; migration is a Phase 4 concern.

### AC-005 (traces to BC-2.07.006 postcondition PC-1 — atomic save via tempfile::persist)
`MonocleConfig::save(&self, path: &Path) -> Result<(), ConfigError>` writes config
atomically: serialize to JSON with `serde_json::to_string_pretty`, write to a
`tempfile::NamedTempFile` in the same directory as `path`, then call
`tempfile.persist(path)`. On any I/O error, returns `Err(ConfigError::IoError(e))`.
Direct `std::fs::write` to `path` is FORBIDDEN (semgrep rule `monocle-no-direct-config-write`
must not fire).

### AC-006 (traces to BC-2.07.006 postcondition PC-2 — parent dir creation)
If the parent directory of `path` does not exist, `save()` creates it (and all
missing ancestors) via `std::fs::create_dir_all(parent)` before writing the tempfile.

### AC-007 (traces to BC-2.07.003 postcondition PC-1 — detect_ccr happy path)
`detect_ccr(config: &MonocleConfig) -> Option<PathBuf>`:
- Step 1: If `config.active_profile` is `Some(name)` and `profiles[name].ccr_path`
  is `Some(p)` and `p.exists()`, return `Some(p)`.
- Step 2: If `config.active_profile` is `None` or step 1 fails, check
  `$HOME/.claude/` (platform home dir via `directories::BaseDirs::new()?.home_dir()`);
  if `~/.claude/` exists, return `Some(home/.claude/)`.
- Step 3: Return `None`.

### AC-008 (traces to BC-2.07.003 postcondition PC-2 — detect_ccr never errors)
`detect_ccr` returns `Option<PathBuf>`, never `Result`. If `BaseDirs::new()` returns
`None` (no home directory detectable), `detect_ccr` returns `None`. No panic, no
`unwrap()` on the home-dir lookup.

### AC-009 (traces to BC-2.07.003 postcondition PC-3 — detect_ccr no caching)
`detect_ccr` performs a fresh filesystem probe on every call. No memoization, no
`Once`/`OnceLock`, no static cache. Callers that need caching are responsible for it.

### AC-010 (traces to BC-2.07.001 postcondition PC-3 — config_path() location)
`MonocleConfig::config_path() -> Result<PathBuf, ConfigError>` returns the canonical
on-disk path using `directories::ProjectDirs::from("", "", "monocle")`:
- On Linux: `~/.config/monocle/config.json`
- On macOS: `~/Library/Application Support/monocle/config.json`
If `ProjectDirs::from("", "", "monocle")` returns `None`, returns
`Err(ConfigError::HomeDirUnresolvable)`.

### AC-011 (traces to BC-2.07.002 invariant INV-1 — no partial writes)
A write failure MUST NOT leave a partially-written or corrupt config file at `path`.
The `tempfile::persist` atomic swap guarantee satisfies this invariant: on failure,
the original file (if any) is intact; the tempfile is cleaned up by the OS.

### AC-012 (traces to BC-2.07.001 invariant INV-2 — binding_overrides always object)
`MonocleConfig` serialized to JSON MUST have `"binding_overrides": {}` (or a populated
object), NEVER `"binding_overrides": null`. The `default_binding_overrides()` function
ensures this for freshly constructed configs; deserialization of an older JSON with
`"binding_overrides": null` must be rejected or coerced to `{}` — not silently accepted
as a `Value::Null`.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,800 |
| BC-2.07.001.md | ~900 |
| BC-2.07.002.md | ~800 |
| BC-2.07.003.md | ~700 |
| BC-2.07.006.md | ~700 |
| S-001 workspace setup reference | ~300 |
| tempfile::persist API | ~200 |
| Test files | ~1,200 |
| **Total estimate** | **~6,600** |

## Tasks

- [ ] Create `monocle-config/` crate: `Cargo.toml`, `src/lib.rs`, `src/config.rs`, `src/error.rs`
- [ ] Add `monocle-config` to workspace `Cargo.toml` members
- [ ] Implement `MonocleConfig`, `ProfileConfig` structs with all fields and derive macros
- [ ] Implement `default_binding_overrides()` returning `Value::Object(Map::new())` and annotate field with `#[serde(default = "default_binding_overrides")]`
- [ ] Implement `MonocleConfig::CURRENT_SCHEMA_VERSION: u32 = 1` as an associated constant
- [ ] Implement `MonocleConfig::load(path: &Path)` with missing-file → default and schema_version validation
- [ ] Implement `MonocleConfig::save(&self, path: &Path)` using `tempfile::NamedTempFile` in same dir, `serde_json::to_string_pretty`, `persist()`
- [ ] Implement `MonocleConfig::config_path()` using `ProjectDirs::from("", "", "monocle")`
- [ ] Implement `detect_ccr(config: &MonocleConfig) -> Option<PathBuf>` with two-step fallback, no caching, no Err return
- [ ] Define `ConfigError` enum with variants: `ParseError { path: PathBuf, source: serde_json::Error }`, `SchemaMismatch { found: u32, expected: u32 }`, `IoError(std::io::Error)`, `HomeDirUnresolvable`
- [ ] Unit tests `monocle-config/tests/config_load_save.rs` — covering AC-003 through AC-006, AC-011
- [ ] Unit tests `monocle-config/tests/detect_ccr.rs` — covering AC-007 through AC-009
- [ ] Unit tests `monocle-config/tests/schema_validation.rs` — covering AC-004, AC-012

## Previous Story Intelligence

S-001 (Cargo workspace): Workspace `Cargo.toml` is at repo root. Adding `monocle-config`
requires inserting it in the `members = [...]` array. Existing workspace dependencies
include `serde`, `serde_json`, `tempfile`, `directories` — all available as workspace
dep declarations; `monocle-config/Cargo.toml` should use `{ workspace = true }` for
these.

N/A — first config story in EPIC-07.

## Architecture Compliance Rules

From `architecture/SS-config.md` and `architecture/SS-conventions-anti-patterns.md`:
- Atomic writes ONLY via `tempfile::persist` — direct `std::fs::write` is forbidden
  (`monocle-no-direct-config-write` semgrep rule enforced in CI)
- `detect_ccr` MUST return `Option<PathBuf>`, never `Result` — two-step fallback with
  silent None for missing home dir
- `binding_overrides` field requires custom serde default fn — `#[serde(default)]`
  alone produces `Value::Null`, which breaks invariant INV-2
- `directories::ProjectDirs::from("", "", "monocle")` — NOT `ProjectDirs::new(...)`
  (wrong constructor)
- No schema migration in Phase 3 — `SchemaMismatch` error is the correct response
- `monocle-config` is a library crate (not a binary); no `main.rs`

**Forbidden Dependencies for monocle-config:**
- `ratatui` — UI library, not a config concern
- `crossterm` — terminal backend, not a config concern
- `nix` — OS signaling, not a config concern
- If any of these appear in `monocle-config/Cargo.toml`, the build MUST fail

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| serde | workspace pin (features=["derive"]) | `MonocleConfig`, `ProfileConfig` derive |
| serde_json | workspace pin | JSON serialize/deserialize, `Value` for binding_overrides |
| tempfile | workspace pin | `NamedTempFile` for atomic save |
| directories | 6 | `ProjectDirs::from("", "", "monocle")` for config_path(); `BaseDirs::new()` for detect_ccr |
| thiserror | 2.x | `ConfigError` enum |
| tracing | 0.1 | INFO log on load/save; DEBUG log on detect_ccr fallback path |

## File Structure Requirements

Files to create:
- `monocle-config/Cargo.toml` — crate manifest listing workspace deps
- `monocle-config/src/lib.rs` — pub re-exports: `MonocleConfig`, `ProfileConfig`, `ConfigError`, `detect_ccr`
- `monocle-config/src/config.rs` — `MonocleConfig`, `ProfileConfig`, `MonocleConfig::load()`, `save()`, `config_path()`, `default_binding_overrides()`
- `monocle-config/src/detect_ccr.rs` — `detect_ccr(config: &MonocleConfig) -> Option<PathBuf>`
- `monocle-config/src/error.rs` — `ConfigError` enum
- `monocle-config/tests/config_load_save.rs` — load/save integration tests
- `monocle-config/tests/detect_ccr.rs` — detect_ccr unit tests
- `monocle-config/tests/schema_validation.rs` — schema version tests

Files to modify:
- `Cargo.toml` (workspace root) — add `monocle-config` to `members`

## Downstream Consumer Contract

Public API produced by this story for downstream consumption:

```rust
// monocle-config
pub struct MonocleConfig {
    pub schema_version: u32,
    pub active_profile: Option<String>,
    pub profiles: std::collections::HashMap<String, ProfileConfig>,
    #[serde(default = "default_binding_overrides")]
    pub binding_overrides: serde_json::Value,
}

impl MonocleConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
    pub fn load(path: &Path) -> Result<Self, ConfigError>;
    pub fn save(&self, path: &Path) -> Result<(), ConfigError>;
    pub fn config_path() -> Result<PathBuf, ConfigError>;
}

pub struct ProfileConfig {
    pub name: String,
    pub ccr_path: Option<PathBuf>,
    pub env_overrides: HashMap<String, String>,
    pub notes: Option<String>,
}

pub fn detect_ccr(config: &MonocleConfig) -> Option<PathBuf>;

pub enum ConfigError {
    ParseError { path: PathBuf, source: serde_json::Error },
    SchemaMismatch { found: u32, expected: u32 },
    IoError(std::io::Error),
    HomeDirUnresolvable,
}
```

S-025 (TUI skeleton) uses `MonocleConfig::load()` at startup. S-031 (profile picker)
uses `MonocleConfig::save()`, `detect_ccr()`, and `ProfileConfig`.
