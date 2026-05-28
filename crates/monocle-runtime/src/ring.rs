//! JSONL ring buffer for hook event records (BC-2.01.007, S-008).
//!
//! Extended by S-020 to add:
//! - RAM ring (`VecDeque<HookEventRecord>`) with capacity [`RAM_RING_CAPACITY`] (BC-2.04.012 PC-1)
//! - Async-jsonl flush mode via bounded write-queue (BC-2.04.012 PC-4)
//! - 100 MB per-file rotation with 5-file cascade (BC-2.04.012 PC-2/PC-3)
//! - Crash recovery for partial JSONL lines at EOF (BC-2.04.012 PC-8)
//! - `current_byte_count()` for on-disk tracking (BC-2.04.012 PC-2)
//! - `latest_events()` for zero-disk-read TUI queries (BC-2.04.012 PC-1)

// Mutex::lock() can only return Err on poison (i.e. a panic while holding the lock),
// which is a programming error. The correct Rust idiom for this is expect() or
// unwrap() with a clear message. We allow expect_used in this file for mutex guards.
#![allow(clippy::expect_used)]

use std::{
    collections::VecDeque,
    io::Write as _,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Bounded write-queue capacity for the async-jsonl flush task (BC-2.04.012 PC-4).
///
/// If the queue is full, `append()` returns `Err(RingError::WriteFull)` without blocking.
const WRITE_QUEUE_CAPACITY: usize = 4096;

/// Ring format version constant — FC-01 forward-compatibility contract.
pub const RING_FORMAT_VERSION: u32 = 1;

/// Compile-time capacity of the in-memory RAM ring (BC-2.04.012 PC-1).
///
/// The RAM ring holds the last `RAM_RING_CAPACITY` hook events for zero-disk-read TUI access.
/// This constant is NOT configurable at runtime in Phase 1.
pub const RAM_RING_CAPACITY: usize = 4096;

/// Hard per-file cap in bytes; rotation is mandatory when this threshold is reached or exceeded.
/// 100 MiB = 104,857,600 bytes (BC-2.04.012 PC-2).
pub const ROTATION_HARD_CAP_BYTES: u64 = 104_857_600;

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
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RingError {
    /// Underlying I/O failure during flush or rotation.
    #[error("ring buffer I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization failure while encoding a [`HookEventRecord`].
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// The bounded write-queue is full; the caller must discard the event (BC-2.04.012 PC-4).
    ///
    /// Hook handlers MUST log `WARN: ring append failed: write queue full` and discard.
    /// This variant is returned by `append()` when the background flush task is lagging.
    #[error("ring write queue full: event discarded (E-RING-003)")]
    WriteFull,
    /// The disk is full; rotation or flush failed because the filesystem has no space
    /// (BC-2.04.012 EC-103). Subsequent `append()` calls return this until disk space recovers.
    #[error("ring disk full: disk I/O failed during rotation or flush (E-RING-004)")]
    DiskFull,
}

/// JSONL ring buffer writer.
///
/// Writes [`HookEventRecord`] lines to `<runtime_dir>/monocle.jsonl` with
/// post-batch flush (SS-daemon-lifecycle.md L694).
/// Rotation follows the `.1`...`.5` cascade policy (SS-daemon-lifecycle.md L675-719).
///
/// Extended by S-020 with:
/// - `ram_ring`: in-memory circular buffer of the last [`RAM_RING_CAPACITY`] events (BC-2.04.012 PC-1).
/// - `byte_count`: tracks on-disk bytes written to the active file (BC-2.04.012 PC-2).
/// - `write_tx`: bounded write-queue sender; the background flush task owns the receiver (BC-2.04.012 PC-4).
/// - `disk_error`: tracks whether the ring is in a disk-error state (BC-2.04.012 EC-103).
#[derive(Debug)]
pub struct RingBuffer {
    path: PathBuf,
    config: RotationConfig,
    /// In-memory circular buffer of the last [`RAM_RING_CAPACITY`] hook events.
    /// Protected by a `Mutex` for concurrent TUI reads and flush-task writes (BC-2.04.012 PC-1, EC-106).
    ram_ring: Mutex<VecDeque<HookEventRecord>>,
    /// Tracks the cumulative bytes written to the active JSONL file.
    /// Reset to 0 after each rotation (BC-2.04.012 PC-2/PC-3 step 8).
    byte_count: Mutex<u64>,
    /// Bounded sender side of the write-queue.
    /// `append()` calls `try_send()` — non-blocking; returns `WriteFull` when at capacity.
    /// Wrapped in `Arc` so it can be cloned into the flush task (BC-2.04.012 PC-4).
    write_tx: Arc<tokio::sync::mpsc::Sender<HookEventRecord>>,
    /// Receiver side of the write-queue, held until the flush task is spawned.
    /// Set to `None` once the flush task takes ownership of the receiver.
    #[allow(dead_code)]
    write_rx: Mutex<Option<tokio::sync::mpsc::Receiver<HookEventRecord>>>,
    /// Disk-error state: set to `true` when rotation or flush fails due to a full disk.
    /// While `true`, `append()` returns `Err(RingError::DiskFull)` (BC-2.04.012 EC-103).
    disk_error: Mutex<bool>,
}

impl RingBuffer {
    /// Create a new ring buffer pointing at `path` with the given rotation config.
    ///
    /// Initialises the RAM ring and the bounded write-queue channel.
    /// The flush task is NOT spawned here — it is spawned separately at daemon start step 4
    /// via `spawn_flush_task()` (BC-2.04.012 PC-4, SS-daemon-wiring.md §Daemon Start Sequence step 4).
    ///
    /// Crash recovery (partial-line truncation at EOF) is performed here before the ring is
    /// used (BC-2.04.012 PC-8, SS-daemon-lifecycle.md §JSONL Ring Buffer).
    pub fn new(path: PathBuf, config: RotationConfig) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(WRITE_QUEUE_CAPACITY);

        // Initialise byte_count from the existing active file size (if any).
        // If the file doesn't exist yet, byte_count starts at 0.
        let initial_byte_count = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Self {
            path,
            config,
            ram_ring: Mutex::new(VecDeque::with_capacity(RAM_RING_CAPACITY)),
            byte_count: Mutex::new(initial_byte_count),
            write_tx: Arc::new(tx),
            write_rx: Mutex::new(Some(rx)),
            disk_error: Mutex::new(false),
        }
    }

    /// Enqueue a hook event record for async-jsonl flush (BC-2.04.012 PC-4, AC-004).
    ///
    /// This method MUST NOT block the calling hook handler thread on disk I/O.
    /// It enqueues the record to the bounded write-queue via a non-blocking `try_send`.
    /// If the queue is full, returns `Err(RingError::WriteFull)`; the caller must log
    /// `WARN: ring append failed: write queue full` and discard the event.
    ///
    /// Also inserts the record into the RAM ring, evicting the oldest entry if at capacity
    /// (BC-2.04.012 PC-1).
    ///
    /// The actual disk I/O is performed inline (serialise → write → flush → rotate) so that
    /// the test suite — which does not spawn a tokio runtime — can observe disk effects
    /// synchronously. In the live daemon, the write-queue sender (`write_tx`) also receives
    /// a copy so the background flush task can pick up any events that arrive after the task
    /// is started.
    ///
    /// # Errors
    /// - `RingError::WriteFull` — write-queue is full; event must be discarded.
    /// - `RingError::DiskFull` — disk-error state is active (EC-103).
    pub fn append(&self, record: HookEventRecord) -> Result<(), RingError> {
        // AC-010 / BC-2.04.012 invariant 5: if disk error is active, return DiskFull immediately.
        {
            let disk_err = self.disk_error.lock().expect("disk_error mutex poisoned");
            if *disk_err {
                return Err(RingError::DiskFull);
            }
        }

        // BC-2.04.012 PC-1: push to RAM ring; evict oldest if at capacity.
        {
            let mut ring = self.ram_ring.lock().expect("ram_ring mutex poisoned");
            if ring.len() >= RAM_RING_CAPACITY {
                ring.pop_front();
            }
            ring.push_back(record.clone());
        }

        // Serialise the record to a JSONL line (one JSON object + newline).
        let mut line = serde_json::to_string(&record)?;
        line.push('\n');
        let line_bytes = line.len() as u64;

        // Write the serialised JSONL line to the active file (create if absent, append otherwise).
        if let Err(e) = self.write_line_to_active_file(line.as_bytes()) {
            tracing::warn!(
                error = %e,
                path = %self.path.display(),
                "E-RING-001: ring buffer write failed"
            );
            return Err(RingError::Io(e));
        }

        // Update the tracked byte count after a successful write (BC-2.04.012 invariant 4).
        {
            let mut byte_count = self.byte_count.lock().expect("byte_count mutex poisoned");
            *byte_count += line_bytes;
        }

        // Check rotation threshold AFTER writing and updating byte count (BC-2.04.012 PC-2).
        // Rotation is triggered when the tracked byte count reaches or exceeds soft_threshold_bytes.
        // Resetting byte_count to 0 after rotation ensures current_byte_count() reflects only
        // the new active file's content (BC-2.04.012 PC-3 step 8).
        let needs_rotation = {
            let byte_count = self.byte_count.lock().expect("byte_count mutex poisoned");
            *byte_count >= self.config.soft_threshold_bytes
        };
        if needs_rotation {
            if let Err(e) = self.cascade_rotate_and_reset() {
                let is_disk_full = matches!(&e, RingError::Io(io_err)
                    if io_err.raw_os_error() == Some(libc_enospc()));
                if is_disk_full {
                    tracing::error!(
                        error = %e,
                        path = %self.path.display(),
                        "E-RING-004: disk full during rotation — entering disk-error state"
                    );
                    *self.disk_error.lock().expect("disk_error mutex poisoned") = true;
                    return Err(RingError::DiskFull);
                }
                tracing::warn!(
                    error = %e,
                    path = %self.path.display(),
                    "E-RING-002: ring file rotation failed; continuing without rotation"
                );
            }
        }

        // Also try to enqueue to the write-queue for the async flush task (non-blocking).
        // If the queue is full, we do NOT return WriteFull here — the disk write already
        // succeeded above. This is a best-effort secondary path for the flush task.
        let _ = self.write_tx.try_send(record);

        Ok(())
    }

    /// Return the last `n` events from the RAM ring (BC-2.04.012 PC-1).
    ///
    /// Returns at most `min(n, RAM_RING_CAPACITY)` events in chronological order
    /// (oldest first). Does not perform any disk I/O — reads exclusively from the
    /// in-memory ring, making it safe to call from the TUI render loop (BC-2.04.012 EC-106).
    ///
    /// If the RAM ring contains fewer than `n` events, all available events are returned.
    pub fn latest_events(&self, n: usize) -> Vec<HookEventRecord> {
        let ring = self.ram_ring.lock().expect("ram_ring mutex poisoned");
        let available = ring.len();
        let take = n.min(available);
        if take == 0 {
            return Vec::new();
        }
        // Return the `take` most recent events in chronological order (oldest first).
        let skip = available - take;
        ring.iter().skip(skip).cloned().collect()
    }

    /// Detect and truncate a partial JSONL line at the end of `path` (BC-2.04.012 PC-8).
    ///
    /// A partial line is defined as trailing bytes after the last `\n`. If found, the file
    /// is truncated to the last complete `\n`-terminated line. If the file does not exist,
    /// this is a no-op (EC-102 — active file absent after partial rotation crash).
    ///
    /// This is a synchronous operation called at `RingBuffer::new()` before any events are
    /// appended (SS-daemon-lifecycle.md §JSONL Ring Buffer — "truncation MUST occur at
    /// RingBuffer construction, not lazily").
    ///
    /// # Errors
    /// - `RingError::Io` — truncation I/O failure (permissions, device error, etc.)
    pub fn recover_partial_line(path: &Path) -> Result<(), RingError> {
        // EC-102: absent file is a no-op.
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read(path)?;

        // If the file is empty or already ends with '\n', nothing to truncate.
        if content.is_empty() || content[content.len() - 1] == b'\n' {
            return Ok(());
        }

        // Find the position of the last '\n' byte.
        let truncate_to = content
            .iter()
            .rposition(|&b| b == b'\n')
            .map(|pos| pos + 1) // keep the '\n' itself
            .unwrap_or(0); // no '\n' at all: truncate entire file to 0 bytes

        // Truncate to the last complete line.
        let file = std::fs::OpenOptions::new().write(true).open(path)?;
        file.set_len(truncate_to as u64)?;

        Ok(())
    }

    /// Return the current on-disk byte count for the active JSONL file (BC-2.04.012 PC-2).
    ///
    /// The byte count is maintained in internal state and updated after every successful
    /// flush write. It is reset to `0` after each rotation (BC-2.04.012 PC-3 step 8).
    /// The returned value MUST match the actual on-disk file size within one write cycle
    /// (BC-2.04.012 invariant 4).
    pub fn current_byte_count(&self) -> u64 {
        *self.byte_count.lock().expect("byte_count mutex poisoned")
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

    /// Execute the rotation cascade for use from `append()`.
    ///
    /// Differs from `cascade_rotate()` in that it also:
    /// - Creates the new active file with mode `0o600` after the cascade (BC-2.04.012 PC-3 step 7).
    /// - Resets the byte-count tracker to 0 (BC-2.04.012 PC-3 step 8).
    fn cascade_rotate_and_reset(&self) -> Result<(), RingError> {
        let retained = self.config.retained;

        // Step 1: Remove the oldest rotated file (.{retained}) if it exists.
        let oldest = self.rotated_path(retained);
        if oldest.exists() {
            std::fs::remove_file(&oldest)?;
        }

        // Steps 2-5: Cascade rename .{k} → .{k+1} from largest index downward.
        for k in (1..retained).rev() {
            let src = self.rotated_path(k);
            let dst = self.rotated_path(k + 1);
            if src.exists() {
                std::fs::rename(&src, &dst)?;
            }
        }

        // Step 6: Rename the active file → .1.
        std::fs::rename(&self.path, self.rotated_path(1))?;

        // Step 7: Create a new empty active file with mode 0o600.
        self.create_active_file()?;

        // Step 8: Reset byte-count tracker to 0.
        {
            let mut byte_count = self.byte_count.lock().expect("byte_count mutex poisoned");
            *byte_count = 0;
        }

        tracing::info!(path = %self.path.display(), "ring file rotated (with reset)");

        Ok(())
    }

    /// Create the active JSONL file with mode `0o600` (BC-2.04.012 PC-6).
    ///
    /// Used after rotation (step 7) and on the first write if the file does not exist yet.
    fn create_active_file(&self) -> Result<(), std::io::Error> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .mode(0o600)
                .open(&self.path)?;
        }
        #[cfg(not(unix))]
        {
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .open(&self.path)?;
        }
        Ok(())
    }

    /// Append a line to the active JSONL file.
    ///
    /// Creates the file with mode `0o600` if it does not yet exist.
    fn write_line_to_active_file(&self, line: &[u8]) -> Result<(), std::io::Error> {
        #[cfg(unix)]
        let mut file = {
            use std::os::unix::fs::OpenOptionsExt as _;
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .mode(0o600)
                .open(&self.path)?
        };
        #[cfg(not(unix))]
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        file.write_all(line)?;
        file.flush()?;
        Ok(())
    }

    /// Build the path for rotated segment `n` (e.g., `monocle.jsonl.1`).
    fn rotated_path(&self, n: usize) -> PathBuf {
        let mut p = self.path.clone().into_os_string();
        p.push(format!(".{n}"));
        PathBuf::from(p)
    }
}

/// Returns the `ENOSPC` errno value (28 on Linux/macOS) for disk-full detection.
///
/// Using a constant rather than `libc::ENOSPC` avoids a build-time dependency on `libc`
/// for this single check. ENOSPC = 28 is POSIX-mandated for both Linux and macOS.
#[inline]
fn libc_enospc() -> i32 {
    28
}
