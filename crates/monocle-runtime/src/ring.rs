//! JSONL ring buffer for hook event records (BC-2.01.007, S-008).
//!
//! Extended by S-020 to add:
//! - RAM ring (`VecDeque<HookEventRecord>`) with capacity [`RAM_RING_CAPACITY`] (BC-2.04.012 PC-1)
//! - Async-jsonl flush mode via bounded write-queue (BC-2.04.012 PC-4)
//! - 100 MB per-file rotation with 5-file cascade (BC-2.04.012 PC-2/PC-3)
//! - Crash recovery for partial JSONL lines at EOF (BC-2.04.012 PC-8)
//! - `current_byte_count()` for on-disk tracking (BC-2.04.012 PC-2)
//! - `latest_events()` for zero-disk-read TUI queries (BC-2.04.012 PC-1)
//!
//! ## Async-flush architecture
//!
//! `append()` is a PURE enqueue operation — it pushes to the RAM ring and sends to the
//! bounded write-queue channel.  It NEVER performs disk I/O (BC-2.04.012 PC-4, AC-004).
//!
//! All disk I/O (serialise → write → flush → rotate) is performed exclusively by the
//! flush task, which is spawned via `spawn_flush_task()` at daemon start step 4
//! (BC-2.04.012 PC-4, SS-daemon-wiring.md §Daemon Start Sequence step 4).
//!
//! Crash recovery (`recover_partial_line`) is called synchronously in `new()` BEFORE
//! reading file metadata for byte count initialisation (BC-2.04.012 PC-8).

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
/// This constant can be overridden in tests by using `RingBuffer::with_queue_capacity`.
pub const WRITE_QUEUE_CAPACITY: usize = 4096;

/// Ring format version constant — FC-01 forward-compatibility contract.
///
/// Re-exported from `monocle-ipc::types` where `HookEventRecord` now lives.
/// `monocle-runtime` callers may use either path; this re-export preserves the
/// pre-relocation API so that existing call sites (`crate::ring::RING_FORMAT_VERSION`)
/// continue to compile without modification.
pub use monocle_ipc::types::RING_FORMAT_VERSION;

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
/// Relocated to `monocle-ipc::types` per architect decision F-S022-ADV2-HIGH-002 (ADR-0006):
/// `HookEventRecord` is the canonical IPC + JSONL ring transport format. Defining it in
/// `monocle-ipc` prevents the circular dependency that would arise from `monocle-ipc`
/// importing from `monocle-runtime` (which already depends on `monocle-ipc`).
///
/// This re-export preserves the pre-relocation API (`monocle_runtime::ring::HookEventRecord`)
/// so that existing call sites in `monocle-runtime` production code and tests continue to
/// compile without modification.
pub use monocle_ipc::types::HookEventRecord;

/// Configuration for ring buffer rotation (SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer
/// Rotation Policy L675-719).
#[derive(Debug, Clone)]
pub struct RotationConfig {
    /// Soft rotation threshold in bytes; kept for API compatibility (default 50 MiB).
    /// Phase 1 flush task uses `hard_cap_bytes` as the authoritative rotation trigger
    /// (BC-2.04.012 PC-2: "100 MB per-file hard cap").
    pub soft_threshold_bytes: u64,
    /// Absolute per-file cap in bytes; rotation is mandatory above this limit (default 100 MiB).
    /// This is the authoritative threshold checked by the flush task (BC-2.04.012 PC-2).
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
/// Writes [`HookEventRecord`] lines to `<runtime_dir>/monocle.jsonl` asynchronously via
/// the flush task spawned by `spawn_flush_task()`.
///
/// `append()` is a PURE enqueue operation — no disk I/O on the caller thread.
/// The flush task owns all disk I/O, serialisation, and rotation (BC-2.04.012 PC-4).
///
/// Rotation follows the `.1`...`.5` cascade policy, triggered when the active file
/// reaches `config.hard_cap_bytes` (BC-2.04.012 PC-2/PC-3).
///
/// Crash recovery (`recover_partial_line`) is called at construction before the ring is
/// used (BC-2.04.012 PC-8, SS-daemon-lifecycle.md §JSONL Ring Buffer).
#[derive(Debug)]
pub struct RingBuffer {
    path: PathBuf,
    config: RotationConfig,
    /// In-memory circular buffer of the last [`RAM_RING_CAPACITY`] hook events.
    /// Protected by a `Mutex` for concurrent TUI reads and flush-task writes (BC-2.04.012 PC-1, EC-106).
    ram_ring: Mutex<VecDeque<HookEventRecord>>,
    /// Tracks the cumulative bytes written to the active JSONL file by the flush task.
    /// Reset to 0 after each rotation (BC-2.04.012 PC-2/PC-3 step 8).
    /// Updated exclusively by the flush task; read by `current_byte_count()`.
    byte_count: Arc<Mutex<u64>>,
    /// Bounded sender side of the write-queue.
    /// `append()` calls `try_send()` — non-blocking; returns `WriteFull` when at capacity.
    /// Wrapped in `Arc` so it can be cloned into callers that need shared ownership.
    write_tx: Arc<tokio::sync::mpsc::Sender<HookEventRecord>>,
    /// Receiver side of the write-queue, held until `spawn_flush_task()` takes ownership.
    /// Set to `None` once the flush task takes the receiver (BC-2.04.012 PC-4).
    write_rx: Mutex<Option<tokio::sync::mpsc::Receiver<HookEventRecord>>>,
    /// Disk-error state: set to `true` when rotation or flush fails due to a full disk.
    /// While `true`, `append()` returns `Err(RingError::DiskFull)` (BC-2.04.012 EC-103).
    disk_error: Arc<Mutex<bool>>,
}

impl RingBuffer {
    /// Create a new ring buffer pointing at `path` with the given rotation config.
    ///
    /// ## Initialisation sequence (BC-2.04.012 PC-8 + PC-4)
    ///
    /// 1. Call `recover_partial_line(&path)` — truncate any partial JSONL line at EOF.
    ///    On error: log WARN and continue (best-effort; ring is still usable).
    /// 2. Read file metadata to initialise `byte_count` from the post-recovery size.
    ///    If the file does not exist, `byte_count` starts at 0.
    /// 3. Initialise the bounded write-queue channel.
    ///    The flush task is NOT spawned here — call `spawn_flush_task()` separately at
    ///    daemon start step 4 (SS-daemon-wiring.md §Daemon Start Sequence step 4).
    pub fn new(path: PathBuf, config: RotationConfig) -> Self {
        Self::with_queue_capacity(path, config, WRITE_QUEUE_CAPACITY)
    }

    /// Create a ring buffer with a custom write-queue capacity.
    ///
    /// Used in tests to create a small queue that fills quickly, enabling `WriteFull` testing
    /// without spawning the flush task (BC-2.04.012 PC-4 / HIGH-004 adversarial fix).
    /// In production, always use `new()` which uses the canonical `WRITE_QUEUE_CAPACITY`.
    pub fn with_queue_capacity(
        path: PathBuf,
        config: RotationConfig,
        queue_capacity: usize,
    ) -> Self {
        // Step 1 (BC-2.04.012 PC-8): crash recovery BEFORE reading file metadata.
        if let Err(e) = Self::recover_partial_line(&path) {
            tracing::warn!(
                error = %e,
                path = %path.display(),
                "E-RING-005: crash recovery (recover_partial_line) failed — continuing without truncation"
            );
        }

        // Step 2: initialise byte_count from the existing active file size (post-recovery).
        // If the file doesn't exist yet, byte_count starts at 0 (EC-102).
        let initial_byte_count = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        // Step 3: initialise bounded write-queue channel.
        let (tx, rx) = tokio::sync::mpsc::channel(queue_capacity);

        Self {
            path,
            config,
            ram_ring: Mutex::new(VecDeque::with_capacity(RAM_RING_CAPACITY)),
            byte_count: Arc::new(Mutex::new(initial_byte_count)),
            write_tx: Arc::new(tx),
            write_rx: Mutex::new(Some(rx)),
            disk_error: Arc::new(Mutex::new(false)),
        }
    }

    /// Enqueue a hook event record for async-jsonl flush (BC-2.04.012 PC-4, AC-004).
    ///
    /// ## Invariants (BC-2.04.012 AC-004, AC-010, invariant 5)
    ///
    /// - This method MUST NOT perform any disk I/O. All disk writes happen in the flush task.
    /// - Enqueues the record to the bounded write-queue via a non-blocking `try_send`.
    /// - Pushes the record into the RAM ring, evicting the oldest entry if at capacity
    ///   (BC-2.04.012 PC-1).
    /// - If the queue is full, returns `Err(RingError::WriteFull)`; the caller must log
    ///   `WARN: ring append failed: write queue full` and discard the event.
    /// - If the disk-error state is active, returns `Err(RingError::DiskFull)` immediately.
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

        // BC-2.04.012 PC-4: non-blocking enqueue to write-queue.
        // On full queue: return WriteFull immediately (no disk I/O, no blocking).
        self.write_tx.try_send(record).map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Full(_) => {
                tracing::warn!("E-RING-003: ring write-queue full — event discarded (WriteFull)");
                RingError::WriteFull
            }
            tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                // Flush task has exited (receiver dropped) — this is a fatal state distinct
                // from a transiently full queue. Log at ERROR to distinguish from Full/WARN.
                tracing::error!(
                    "E-RING-006: ring write-queue channel closed (flush task dead) — event discarded"
                );
                RingError::WriteFull
            }
        })?;

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

        // LOW-001: reads the full file to find the last '\n'. For Phase 1, ring files are
        // bounded to 100 MiB before rotation, so this is acceptable. A streaming/seek-based
        // scan from the end of the file would be more efficient for very large files, but is
        // not necessary until Phase 4 when the hard cap may be tuned higher.
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
    /// The byte count is maintained by the flush task and updated after every successful
    /// flush write. It is reset to `0` after each rotation (BC-2.04.012 PC-3 step 8).
    /// The returned value reflects the last flush cycle's write; in the flush task loop,
    /// it matches the actual on-disk file size exactly (BC-2.04.012 invariant 4).
    pub fn current_byte_count(&self) -> u64 {
        *self.byte_count.lock().expect("byte_count mutex poisoned")
    }

    /// Return a cloned `Arc` to the byte-count tracker.
    ///
    /// Allows tests to hold a reference to `byte_count` independently of the `RingBuffer`,
    /// so they can drop the ring (closing the write channel) and then read the final
    /// byte count after the flush task drains.
    ///
    /// # Note
    ///
    /// This method is public so that integration tests can access it.
    /// Production callers should use `current_byte_count()` instead.
    #[doc(hidden)]
    pub fn byte_count_handle(&self) -> Arc<Mutex<u64>> {
        Arc::clone(&self.byte_count)
    }

    /// Spawn the background flush task that drains the write-queue and writes to disk.
    ///
    /// This method takes ownership of the `write_rx` receiver from the ring buffer.
    /// It MUST be called exactly once, at daemon start step 4
    /// (BC-2.04.012 PC-4, SS-daemon-wiring.md §Daemon Start Sequence step 4).
    ///
    /// ## Flush task loop
    ///
    /// 1. `recv()` from channel — returns `None` when sender is dropped (graceful shutdown).
    /// 2. Serialise record to a JSONL line (one JSON object + `\n`).
    /// 3. Write to the active file (open-append with mode `0o600`).
    /// 4. Update `byte_count` after a successful write (invariant 4).
    /// 5. If `byte_count >= config.hard_cap_bytes`: execute cascade rotation and reset to 0.
    ///
    /// On rotation failure: log WARN and continue without rotation (AC-005).
    /// On disk-full (ENOSPC): set `disk_error = true`, log ERROR (EC-103).
    ///
    /// ## Panics
    ///
    /// Panics if called after the receiver has already been taken (i.e., called twice).
    pub fn spawn_flush_task(&self) -> tokio::task::JoinHandle<()> {
        let rx = self
            .write_rx
            .lock()
            .expect("write_rx mutex poisoned")
            .take()
            .expect("spawn_flush_task() called twice — flush task is already running");

        let path = self.path.clone();
        let config = self.config.clone();
        let byte_count = Arc::clone(&self.byte_count);
        let disk_error = Arc::clone(&self.disk_error);

        tokio::spawn(async move {
            flush_task_loop(rx, path, config, byte_count, disk_error).await;
        })
    }

    /// Push a record to the ring buffer (legacy synchronous API — S-008 compatibility).
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

    /// Rotate the ring file when it exceeds the soft threshold (legacy synchronous path).
    ///
    /// Used by the legacy `push()` API. The flush task uses `hard_cap_bytes` directly.
    pub fn rotate_if_needed(&self) -> Result<(), RingError> {
        // Check current file size; if it doesn't exist yet, nothing to rotate.
        let size = match std::fs::metadata(&self.path) {
            Ok(meta) => meta.len(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(RingError::Io(e)),
        };

        let needs_rotation = size >= self.config.soft_threshold_bytes;
        if !needs_rotation {
            return Ok(());
        }

        self.cascade_rotate()
    }

    /// Execute the `.1`...`.N` rename cascade and retire the active file to `.1`.
    ///
    /// Called by the legacy `rotate_if_needed()` once the size threshold is confirmed.
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

    /// Create the active JSONL file with mode `0o600` (BC-2.04.012 PC-6).
    ///
    /// Used after rotation (step 7) and on the first write if the file does not exist yet.
    fn create_active_file(path: &Path) -> Result<(), std::io::Error> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .mode(0o600)
                .open(path)?;
        }
        #[cfg(not(unix))]
        {
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .open(path)?;
        }
        Ok(())
    }

    /// Build the path for rotated segment `n` (e.g., `monocle.jsonl.1`).
    fn rotated_path(&self, n: usize) -> PathBuf {
        rotated_path_for(&self.path, n)
    }

    /// Forcibly set the disk-error state for testing.
    ///
    /// When set to `true`, subsequent `append()` calls return `Err(RingError::DiskFull)`.
    /// This allows integration tests to verify the EC-103 disk-full error path without
    /// actually filling the filesystem.
    ///
    /// # Note
    ///
    /// This method is public so that integration tests (which compile as separate crates)
    /// can access it.  Production callers MUST NOT use this method — it exists solely to
    /// inject the disk-error state that the flush task sets on ENOSPC.
    #[doc(hidden)]
    pub fn set_disk_error_for_testing(&self, val: bool) {
        *self.disk_error.lock().expect("disk_error mutex poisoned") = val;
    }
}

/// Build the path for rotated segment `n` (e.g., `monocle.jsonl.1`).
/// Free function for use by the async flush task (avoids capturing `&self` across await).
fn rotated_path_for(base: &Path, n: usize) -> PathBuf {
    let mut p = base.as_os_str().to_os_string();
    p.push(format!(".{n}"));
    PathBuf::from(p)
}

/// Execute the rotation cascade for use from the async flush task.
///
/// Differs from the legacy `cascade_rotate()` in that it also:
/// - Creates the new active file with mode `0o600` after the cascade (BC-2.04.012 PC-3 step 7).
/// - Resets the byte-count tracker to 0 (BC-2.04.012 PC-3 step 8).
fn cascade_rotate_and_reset(
    path: &Path,
    retained: usize,
    byte_count: &Mutex<u64>,
) -> Result<(), RingError> {
    // Step 1: Remove the oldest rotated file (.{retained}) if it exists.
    let oldest = rotated_path_for(path, retained);
    if oldest.exists() {
        std::fs::remove_file(&oldest)?;
    }

    // Steps 2-5: Cascade rename .{k} → .{k+1} from largest index downward.
    for k in (1..retained).rev() {
        let src = rotated_path_for(path, k);
        let dst = rotated_path_for(path, k + 1);
        if src.exists() {
            std::fs::rename(&src, &dst)?;
        }
    }

    // Step 6: Rename the active file → .1.
    std::fs::rename(path, rotated_path_for(path, 1))?;

    // Step 7: Create a new empty active file with mode 0o600.
    RingBuffer::create_active_file(path)?;

    // Step 8: Reset byte-count tracker to 0.
    {
        let mut count = byte_count.lock().expect("byte_count mutex poisoned");
        *count = 0;
    }

    tracing::info!(path = %path.display(), "ring file rotated (async flush task)");

    Ok(())
}

/// Open (or create) the active file in append mode with `0o600` permissions.
fn open_active_file_append(path: &Path) -> Result<std::fs::File, std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(path)
    }
    #[cfg(not(unix))]
    {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    }
}

/// The flush task loop.
///
/// Drains the write-queue, serialises records to JSONL, and writes to the active file.
/// Performs rotation when the active file reaches `config.hard_cap_bytes`.
/// Terminates when the sender is dropped (receiver returns `None`).
async fn flush_task_loop(
    mut rx: tokio::sync::mpsc::Receiver<HookEventRecord>,
    path: PathBuf,
    config: RotationConfig,
    byte_count: Arc<Mutex<u64>>,
    disk_error: Arc<Mutex<bool>>,
) {
    while let Some(record) = rx.recv().await {
        // Serialise to JSONL line.
        let line = match serde_json::to_string(&record) {
            Ok(mut s) => {
                s.push('\n');
                s
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "E-RING-001: flush task — JSON serialisation failed; record discarded"
                );
                continue;
            }
        };
        let line_bytes = line.len() as u64;

        // Write to active file.
        let write_result = (|| -> Result<(), std::io::Error> {
            let mut file = open_active_file_append(&path)?;
            file.write_all(line.as_bytes())?;
            file.flush()?;
            Ok(())
        })();

        if let Err(e) = write_result {
            // Check for ENOSPC (disk full).
            if e.raw_os_error() == Some(libc_enospc()) {
                tracing::error!(
                    error = %e,
                    path = %path.display(),
                    "E-RING-004: disk full during flush — entering disk-error state"
                );
                *disk_error.lock().expect("disk_error mutex poisoned") = true;
                return; // Terminate flush task; daemon still runs but ring is in error state.
            }
            tracing::warn!(
                error = %e,
                path = %path.display(),
                "E-RING-001: ring buffer write failed in flush task"
            );
            continue;
        }

        // Update byte count after successful write (BC-2.04.012 invariant 4).
        let current_count = {
            let mut count = byte_count.lock().expect("byte_count mutex poisoned");
            *count += line_bytes;
            *count
        };

        // Check rotation threshold: use hard_cap_bytes (BC-2.04.012 PC-2).
        if current_count >= config.hard_cap_bytes {
            if let Err(e) = cascade_rotate_and_reset(&path, config.retained, &byte_count) {
                // Check for ENOSPC during rotation.
                let is_disk_full = matches!(&e, RingError::Io(io_err)
                    if io_err.raw_os_error() == Some(libc_enospc()));
                if is_disk_full {
                    tracing::error!(
                        error = %e,
                        path = %path.display(),
                        "E-RING-004: disk full during rotation — entering disk-error state"
                    );
                    *disk_error.lock().expect("disk_error mutex poisoned") = true;
                    return;
                }
                tracing::warn!(
                    error = %e,
                    path = %path.display(),
                    "E-RING-002: ring file rotation failed; continuing without rotation"
                );
                // MED-001: After a failed rotation, re-read the actual on-disk file size
                // and reset byte_count to match. Without this, the stale accumulated count
                // would keep triggering the rotation threshold on every subsequent write,
                // flooding the log with spurious E-RING-002 warnings.
                let actual_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                *byte_count.lock().expect("byte_count mutex poisoned") = actual_size;
            }
        }
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
