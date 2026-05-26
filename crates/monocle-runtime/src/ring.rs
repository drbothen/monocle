//! JSONL ring buffer for hook event records (BC-2.01.007, S-008).

use std::path::PathBuf;

/// Ring format version constant — FC-01 forward-compatibility contract.
pub const RING_FORMAT_VERSION: u32 = 1;

/// A single hook event record written to the JSONL ring buffer.
///
/// Fields are in canonical declaration order per SS-core-types-and-abi.md §HookEventRecord.
/// `format_version` MUST serialize as the first JSON key (struct field order preservation).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct HookEventRecord {
    /// FC-01 forward-compatibility version stamp; always set to [`RING_FORMAT_VERSION`].
    pub format_version: u32,
    /// Opaque session identifier (UUID string).
    pub session_id: String,
    /// Unix epoch timestamp in microseconds (signed per SS-core-types-and-abi.md §HookEventRecord).
    pub timestamp_micros: i64,
    /// Process ID of the originating harness process.
    pub pid: u32,
    /// Hook type discriminant (e.g. `"PreToolUse"`, `"SessionStart"`).
    pub hook_type: String,
    /// Tool name present only for tool-context hook types (e.g. `"PreToolUse"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool input JSON present only for tool-context hook types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
}

impl HookEventRecord {
    /// Construct a new record. `format_version` is set to [`RING_FORMAT_VERSION`] internally.
    ///
    /// External callers MUST use this constructor — `#[non_exhaustive]` forbids struct-literal
    /// construction outside `monocle-runtime::ring` (BC-2.01.007 PC-5, Rust E0639).
    pub fn new(
        session_id: String,
        timestamp_micros: i64,
        pid: u32,
        hook_type: String,
        tool_name: Option<String>,
        tool_input: Option<serde_json::Value>,
    ) -> Self {
        Self {
            format_version: RING_FORMAT_VERSION,
            session_id,
            timestamp_micros,
            pid,
            hook_type,
            tool_name,
            tool_input,
        }
    }
}

/// Configuration for ring buffer rotation (SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer
/// Rotation Policy L675-719).
#[derive(Debug, Clone)]
pub struct RotationConfig {
    /// Soft rotation threshold in bytes; rotation is checked on each flush batch (default 50 MiB).
    pub soft_threshold_bytes: u64,
    /// Absolute per-file cap in bytes; rotation is mandatory above this limit (default 100 MiB).
    pub hard_cap_bytes: u64,
    /// Number of rotated files to retain via `.1`...`.N` cascade (default 5).
    pub retained: usize,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            soft_threshold_bytes: 50 * 1024 * 1024,
            hard_cap_bytes: 100 * 1024 * 1024,
            retained: 5,
        }
    }
}

/// Error type for ring buffer operations (E-RING-001 taxonomy).
#[derive(Debug, thiserror::Error)]
pub enum RingError {
    /// Underlying I/O failure during flush or rotation.
    #[error("ring buffer I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization failure while encoding a [`HookEventRecord`].
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// JSONL ring buffer writer.
///
/// Writes [`HookEventRecord`] lines to `<runtime_dir>/monocle-events.jsonl` with
/// post-batch atomic flush via `tempfile::persist` (SS-daemon-lifecycle.md L694).
/// Rotation follows the `.1`...`.5` cascade policy (SS-daemon-lifecycle.md L675-719).
#[allow(dead_code)] // fields are read only after S-008 implementation; stubs use todo!()
pub struct RingBuffer {
    path: PathBuf,
    config: RotationConfig,
}

impl RingBuffer {
    /// Create a new ring buffer pointing at `path` with the given rotation config.
    pub fn new(path: PathBuf, config: RotationConfig) -> Self {
        Self { path, config }
    }

    /// Push a record to the ring buffer.
    ///
    /// Serializes the record as a JSONL line and flushes atomically via `tempfile::persist`
    /// after the batch. DI-001: callers MUST call this before constructing any HTTP response.
    ///
    /// On I/O failure (E-RING-001) the error is returned; callers should log at WARN and
    /// continue accepting events per AC-005.
    #[allow(clippy::todo)]
    pub fn push(&self, _record: &HookEventRecord) -> Result<(), RingError> {
        todo!("S-008: push record to JSONL ring")
    }

    /// Rotate the ring file when it exceeds the soft threshold.
    ///
    /// Implements the `.1`...`.5` cascade: oldest rotated file is removed first, then
    /// each existing rotated file is incremented, and the active file becomes `.1`.
    /// A fresh empty active file is created after rotation.
    /// Hard cap: rotation is forced when active file reaches `hard_cap_bytes`.
    #[allow(clippy::todo)]
    pub fn rotate_if_needed(&self) -> Result<(), RingError> {
        todo!("S-008: rotate ring file")
    }
}
