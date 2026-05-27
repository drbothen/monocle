//! Error taxonomy for `monocle-config`.
//!
//! Per BC-2.07.001 §Edge Cases and BC-2.07.003 §Postconditions.
//! All variants use `thiserror 2.x` derives for library error types.

use thiserror::Error;

/// All failure modes from the monocle-config crate.
///
/// `load_config()` returns `Err` only for `HomeUnresolvable` and
/// `Io(e)` where `e.kind() != ErrorKind::NotFound`. Parse failures
/// are swallowed after logging, per BC-2.07.003 PC-9.
///
/// `write_config()` returns `Err` for `Serialize`, `Io`, `PersistFailed`,
/// `InvalidPath`, and `HomeUnresolvable` per BC-2.07.001.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// JSON serialization failure — structurally impossible given `MonocleConfig`'s
    /// Rust representation, but `serde_json::to_string_pretty` returns `Result`.
    /// Per BC-2.07.001 EC-076.
    #[error("config serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),

    /// The config file path has no parent directory component.
    /// Per BC-2.07.001 EC-074.
    #[error("config path has no parent directory component")]
    InvalidPath,

    /// I/O error — includes permission denied (BC-2.07.003 PC-3/4),
    /// directory creation failure (BC-2.07.001 PC-1/EC-071), and
    /// `IsADirectory` reads (BC-2.07.003 EC-094).
    #[error("config I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// `NamedTempFile::persist()` failed — the file could not be renamed
    /// atomically to the target path. Per BC-2.07.001 PC-5/EC-073.
    #[error("config atomic write (persist) failed: {0}")]
    PersistFailed(#[from] tempfile::PersistError),

    /// `directories::ProjectDirs::from("", "", "monocle")` returned `None`
    /// — no home directory resolvable. Per BC-2.07.001 EC-077 and BC-2.07.003 EC-095.
    #[error("cannot resolve home directory to determine config path")]
    HomeUnresolvable,

    /// The on-disk `schema_version` does not match `MonocleConfig::CURRENT_SCHEMA_VERSION`.
    /// Phase 3 does not perform silent migration; this error signals a schema mismatch.
    /// Per story AC-004.
    #[error("config schema version mismatch: found {found}, expected {expected}")]
    SchemaMismatch { found: u32, expected: u32 },
}
