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
/// post-batch flush (SS-daemon-lifecycle.md L694).
/// Rotation follows the `.1`...`.5` cascade policy (SS-daemon-lifecycle.md L675-719).
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
    /// Serializes the record as a JSONL line and appends it to the ring file.
    /// Flushes after each write to ensure durability (SS-daemon-lifecycle.md L694).
    /// DI-001: callers MUST call this before constructing any HTTP response.
    ///
    /// On I/O failure (E-RING-001) the error is returned; callers should log at WARN and
    /// continue accepting events per AC-005.
    /// Rotation failure after a successful write is downgraded to a WARN log (AC-005,
    /// SS-daemon-lifecycle L710): the record is already persisted, so rotation failure
    /// is not fatal to the caller.
    pub fn push(&self, record: &HookEventRecord) -> Result<(), RingError> {
        let mut line = serde_json::to_string(record)?;
        line.push('\n');

        use std::io::Write as _;
        #[cfg(unix)]
        let mut file = {
            use std::os::unix::fs::OpenOptionsExt as _;
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .mode(0o600) // Set permissions at creation time (SS-daemon-lifecycle L693)
                .open(&self.path)?
        };
        #[cfg(not(unix))]
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(line.as_bytes())?;

        if let Err(e) = file.flush() {
            // E-RING-001: callers log at WARN and continue (AC-005).
            tracing::warn!(error = %e, path = %self.path.display(), "E-RING-001: ring buffer flush failed");
            return Err(RingError::Io(e));
        }

        // F-001: rotation failure is degraded, not fatal — the record is already persisted.
        if let Err(e) = self.rotate_if_needed() {
            tracing::warn!(
                error = %e,
                path = %self.path.display(),
                "E-RING-002: ring file rotation failed; continuing without rotation"
            );
        }
        Ok(())
    }

    /// Rotate the ring file when it exceeds the soft threshold.
    ///
    /// Implements the `.1`...`.N` cascade: oldest rotated file is removed first, then
    /// each existing rotated file is incremented (`k` → `k+1`), and the active file
    /// becomes `.1`. The caller's next push will create a fresh active file.
    ///
    /// Rotation triggers when the active file size meets or exceeds
    /// `config.soft_threshold_bytes`. The hard cap is implicitly enforced because
    /// `soft_threshold_bytes <= hard_cap_bytes` — any file exceeding the hard cap has
    /// already exceeded the soft threshold and triggered rotation.
    pub fn rotate_if_needed(&self) -> Result<(), RingError> {
        // Check current file size; if it doesn't exist yet, nothing to rotate.
        let size = match std::fs::metadata(&self.path) {
            Ok(meta) => meta.len(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(RingError::Io(e)),
        };

        // Rotate when the active file exceeds the soft threshold.
        // The hard cap (config.hard_cap_bytes) is implicitly enforced because
        // soft_threshold_bytes <= hard_cap_bytes — any file that exceeds the hard
        // cap has already exceeded the soft threshold and triggered rotation.
        let needs_rotation = size >= self.config.soft_threshold_bytes;
        if !needs_rotation {
            return Ok(());
        }

        self.cascade_rotate()
    }

    /// Execute the `.1`...`.N` rename cascade and retire the active file to `.1`.
    ///
    /// Called by [`rotate_if_needed`] once the size threshold is confirmed.
    fn cascade_rotate(&self) -> Result<(), RingError> {
        let retained = self.config.retained;

        // Remove the oldest rotated file (.{retained}) if it exists.
        let oldest = self.rotated_path(retained);
        if oldest.exists() {
            std::fs::remove_file(&oldest)?;
        }

        // Cascade: rename .{k} → .{k+1} from largest index downward.
        for k in (1..retained).rev() {
            let src = self.rotated_path(k);
            let dst = self.rotated_path(k + 1);
            if src.exists() {
                std::fs::rename(&src, &dst)?;
            }
        }

        // Rename the active file → .1.
        std::fs::rename(&self.path, self.rotated_path(1))?;

        tracing::info!(path = %self.path.display(), "ring file rotated");

        Ok(())
    }

    /// Build the path for rotated segment `n` (e.g., `monocle-events.jsonl.1`).
    fn rotated_path(&self, n: usize) -> PathBuf {
        let mut p = self.path.clone().into_os_string();
        p.push(format!(".{n}"));
        PathBuf::from(p)
    }
}
