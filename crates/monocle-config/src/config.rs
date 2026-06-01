//! Config schema v1 and load/save/config_path functions.
//!
//! Schema: BC-2.07.002.
//! Load resilience: BC-2.07.003.
//! Atomic write: BC-2.07.001.

#![forbid(unsafe_code)]

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

/// A single harness profile entry.
///
/// Per BC-2.07.002 PC-2: fields `id`, `display_name`, `binary_path`, `config_dir`.
/// `#[serde(deny_unknown_fields)]` is NOT applied — forward-compat per BC-2.07.002 PC-8.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarnessProfile {
    /// Unique profile identifier. Must be unique within `harness_profiles`; uniqueness
    /// is enforced by callers (profile picker), not by the schema layer.
    pub id: String,
    /// Human-readable display name for the profile.
    pub display_name: String,
    /// Filesystem path to the harness binary (e.g., `/usr/local/bin/claude`).
    /// Existence is NOT validated at schema level — validated at engine spawn time
    /// (BC-2.03.004).
    pub binary_path: String,
    /// Optional config directory override for the harness.
    #[serde(default)]
    pub config_dir: Option<String>,
}

/// Custom default for `binding_overrides` — must be `Value::Object(Map::new())`,
/// NOT `Value::Null` (which is what `#[serde(default)]` would produce on
/// `serde_json::Value`). Per BC-2.07.002 EC-080 and BC-2.07.002 §Invariants INV-3.
fn default_binding_overrides() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

/// The canonical monocle configuration schema (v1).
///
/// Per BC-2.07.002: `schema_version` is the first serialized field (struct field order
/// controls serde serialization for named structs). `#[serde(deny_unknown_fields)]` is
/// NOT applied, ensuring unknown keys from future versions are silently ignored
/// (BC-2.07.002 PC-8, EC-083).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonocleConfig {
    /// Schema version sentinel. Always `1` in Phase 1. Must be the first JSON key.
    /// Per BC-2.07.002 PC-1 and INV-1.
    pub schema_version: u32,

    /// All configured harness profiles. Default: empty vec (no profiles).
    /// Per BC-2.07.002 PC-2 and EC-078.
    #[serde(default)]
    pub harness_profiles: Vec<HarnessProfile>,

    /// Explicit CCR (Claude Code Router) binary path. When `Some`, CCR detection
    /// checks this path first before falling back to PATH search.
    /// Per BC-2.07.002 PC-3 and BC-2.07.006 §Step 1.
    #[serde(default)]
    pub ccr_path: Option<String>,

    /// Opaque keybinding overrides. Stored and round-tripped as a JSON object.
    /// MUST serialize as `{}` when empty, never `null`.
    /// Per BC-2.07.002 PC-4 and INV-3.
    #[serde(default = "default_binding_overrides")]
    pub binding_overrides: serde_json::Value,

    /// Map from absolute project directory path strings to profile IDs.
    /// Referential integrity (profile ID must exist in `harness_profiles`) is
    /// enforced by the profile picker (BC-2.07.004), not this schema layer.
    /// Per BC-2.07.002 PC-5 and EC-081.
    #[serde(default)]
    pub project_profiles: HashMap<String, String>,
}

impl Default for MonocleConfig {
    fn default() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            harness_profiles: Vec::new(),
            ccr_path: None,
            binding_overrides: default_binding_overrides(),
            project_profiles: HashMap::new(),
        }
    }
}

impl MonocleConfig {
    /// The schema version this Phase 1 binary reads and writes.
    /// Per BC-2.07.002 INV-1: always `1` in Phase 1 code.
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    /// Returns the canonical on-disk path for the monocle config file.
    ///
    /// Uses `directories::ProjectDirs::from("", "", "monocle")`:
    /// - Linux: `~/.config/monocle/config.json`
    /// - macOS: `~/Library/Application Support/monocle/config.json`
    ///
    /// Returns `Err(ConfigError::HomeUnresolvable)` if `ProjectDirs::from`
    /// returns `None`. Per BC-2.07.001 PC-2 and story AC-010.
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let project_dirs = directories::ProjectDirs::from("", "", "monocle")
            .ok_or(ConfigError::HomeUnresolvable)?;
        Ok(project_dirs.config_dir().join("config.json"))
    }
}

// ---------------------------------------------------------------------------
// resolve_profile_for_dir (BC-2.07.004 PC-9)
// ---------------------------------------------------------------------------

/// Resolve the sticky harness profile for `dir` from `config.project_profiles`.
///
/// Pure function — no I/O, no side effects, no logging (BC-2.07.004 INV-2 / PC-9).
///
/// # Algorithm (BC-2.07.004 PC-2/3/4/5/6/7)
///
/// 1. Look up `dir` in `config.project_profiles` as a verbatim string key
///    (INV-1: no normalization here — caller normalizes once at lookup time,
///    identical to write-time normalization from BC-2.07.005 PC-5a).
/// 2. If the key exists, retrieve the profile ID string.
/// 3. Find the first entry in `config.harness_profiles` where `profile.id == profile_id`.
/// 4. If found: return `Some(&profile)` (PC-5: sticky match, picker NOT shown).
/// 5. If the profile ID is dangling (not in `harness_profiles`): return `None`
///    (PC-6: treated as "no match"; dangling entry is NOT removed).
/// 6. If `dir` is not in `project_profiles`: return `None` (PC-7 or PC-8).
///
/// # Edge cases
///
/// - Empty string profile ID (`""`) never matches a valid profile (all valid IDs
///   are non-empty) → returns `None` (BC-2.07.004 EC-105).
/// - `default_config` with no profiles and no entries → `None` (EC-097).
pub fn resolve_profile_for_dir<'a>(
    config: &'a MonocleConfig,
    dir: &str,
) -> Option<&'a HarnessProfile> {
    // BC-2.07.004 PC-2/3/4: look up dir verbatim in project_profiles.
    let profile_id = config.project_profiles.get(dir)?;
    // BC-2.07.004 PC-4/5/6: find first matching profile by id.
    // EC-105: empty string never matches (all valid ids are non-empty).
    if profile_id.is_empty() {
        return None;
    }
    config.harness_profiles.iter().find(|p| &p.id == profile_id)
}

// ---------------------------------------------------------------------------
// load_config
// ---------------------------------------------------------------------------

/// Load the monocle config from `path`.
///
/// Implements the four-case resilience contract from BC-2.07.003:
///
/// - **Case 1 (missing):** Returns `Ok(MonocleConfig::default())`. Silent.
/// - **Case 2 (I/O error, not NotFound):** Returns `Err(ConfigError::Io(e))`.
/// - **Case 3 (parse failure):** Emits `tracing::warn!` and returns `Ok(MonocleConfig::default())`.
/// - **Case 4 (success):** Returns `Ok(config)`.
///
/// Does not panic under any condition (BC-2.07.003 INV-1).
pub fn load_config(path: &Path) -> Result<MonocleConfig, ConfigError> {
    // Case 1: file does not exist → return default silently.
    let raw = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(MonocleConfig::default());
        }
        // Case 2: real I/O error → propagate.
        Err(e) => return Err(ConfigError::Io(e)),
    };

    // Case 3: parse failure → warn + return default.
    let config: MonocleConfig = match serde_json::from_str(&raw) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "monocle-config: failed to parse config file — using default (BC-2.07.003 PC-9)"
            );
            return Ok(MonocleConfig::default());
        }
    };

    // Story AC-004: schema version mismatch check (after successful parse).
    if config.schema_version != MonocleConfig::CURRENT_SCHEMA_VERSION {
        return Err(ConfigError::SchemaMismatch {
            found: config.schema_version,
            expected: MonocleConfig::CURRENT_SCHEMA_VERSION,
        });
    }

    // Case 4: success.
    Ok(config)
}

// ---------------------------------------------------------------------------
// write_config
// ---------------------------------------------------------------------------

/// Write `config` atomically to `path` using `tempfile::persist`.
///
/// Implements BC-2.07.001:
/// 1. Creates parent directory via `std::fs::create_dir_all(parent)` (PC-1).
/// 2. Creates `NamedTempFile::new_in(parent)` — same FS, guarantees atomic rename (PC-2).
/// 3. Serializes with `serde_json::to_string_pretty` (PC-3).
/// 4. Writes and flushes to temp file (PC-3, PC-4).
/// 5. Calls `tmp.persist(path)` for atomic rename (PC-5).
///
/// Direct `std::fs::write` to the config path is FORBIDDEN (semgrep rule
/// `monocle-no-direct-config-write` enforced at PR merge time).
///
/// Returns `Err` on any failure; the original file is never partially written (INV-1, INV-2).
pub fn write_config(config: &MonocleConfig, path: &Path) -> Result<(), ConfigError> {
    use std::io::Write as _;

    // PC-1: resolve parent directory; return error if path has no parent.
    let parent = path.parent().ok_or(ConfigError::InvalidPath)?;

    // PC-1: create parent directory (and all ancestors) if absent.
    std::fs::create_dir_all(parent)?;

    // PC-3: serialize to pretty JSON.
    let json = serde_json::to_string_pretty(config)?;

    // PC-2: create temp file in same directory as target (ensures same filesystem).
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;

    // PC-3/PC-4: write and flush.
    tmp.write_all(json.as_bytes())?;
    tmp.flush()?;

    // PC-5: atomic rename to target path.
    tmp.persist(path)?;

    Ok(())
}
