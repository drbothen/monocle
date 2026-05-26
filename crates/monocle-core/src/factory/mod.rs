//! Factory awareness abstraction (BC-2.02.004, BC-2.02.005, S-012).
//!
//! Defines `FactoryAdapter` — an open trait (no `Sealed` bound) with exactly 7 methods,
//! suitable for Phase 1 `VsddFactoryAdapter` and future Phase 3 WASM plugin adapters.

pub mod vsdd;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use futures::Stream;

/// A factory adapter for a specific factory framework (e.g., VSDD Factory).
///
/// # Stability Contract
///
/// This trait has exactly 7 methods (BC-2.02.004 postcondition 1). Any addition requires
/// an ADR and is a breaking change. The trait is intentionally NOT sealed so that Phase 3
/// WASM plugin adapters can implement it outside this crate (BC-2.02.004 postcondition 2).
///
/// Supertrait bounds: `Send + Sync + 'static` ONLY — no `private::Sealed`.
pub trait FactoryAdapter: Send + Sync + 'static {
    /// Detect whether `workspace_root` contains this factory type.
    ///
    /// Returns `Some(FactoryDetection)` if detected, `None` otherwise.
    /// `where Self: Sized` prevents calling on trait objects.
    fn detect(workspace_root: &Path) -> Option<FactoryDetection>
    where
        Self: Sized;

    /// Returns `true` if `workspace_root` is a workspace this adapter can serve.
    ///
    /// Delegates to `Self::detect` — returns `true` iff detection succeeds.
    fn matches(&self, workspace_root: &Path) -> bool;

    /// Path to the factory state file for this adapter instance.
    fn state_file_path(&self) -> &Path;

    /// Read and parse the current factory state from the state file.
    ///
    /// Returns `Err(FactoryReadError::NotFound)` if the state file is absent (E-FACT-001).
    /// Returns `Err(FactoryReadError::ParseError)` if the file cannot be parsed (E-FACT-002).
    fn read_state(&self) -> Result<FactoryState, FactoryReadError>;

    /// Subscribe to state change notifications.
    ///
    /// Phase 1: returns `Ok(Box::pin(futures::stream::empty()))` — no file watcher instantiated.
    /// Phase 3+: may return a live `notify`-backed stream.
    fn subscribe(&self) -> Result<StateChangeStream, FactorySubscribeError>;

    /// Human-readable display name for this factory adapter (e.g., `"VSDD Factory"`).
    fn display_name(&self) -> &str;

    /// ABI version for cross-crate compatibility checking.
    ///
    /// Default implementation returns `crate::MONOCLE_ABI_VERSION`.
    fn abi_version(&self) -> u32 {
        crate::MONOCLE_ABI_VERSION
    }
}

/// Result of a successful factory detection scan.
///
/// Three fields exactly, per SS-core-types-and-abi.md lines 337–344.
#[derive(Debug, Clone)]
pub struct FactoryDetection {
    /// Human-readable name of the detected factory (e.g., `"VSDD Factory"`).
    pub display_name: String,
    /// Absolute path to the workspace root where the factory was detected.
    pub workspace_root: PathBuf,
    /// Path to the factory state file within the workspace.
    pub state_file: PathBuf,
}

/// Parsed factory state, sourced from the factory state file.
///
/// Exactly 7 canonical fields per SS-core-types-and-abi.md §FactoryState (lines 364–400).
/// `raw_frontmatter` is FORBIDDEN — it is a red-line field per BC-2.02.004 INV-2.
/// Absent optional fields carry `None`; consumers display `"—"` (TUI responsibility).
#[derive(Debug, Clone)]
pub struct FactoryState {
    /// Pipeline phase identifier (e.g., `"phase-3-WAVE-2-GATE-PASSED"`).
    pub phase: String,
    /// Workflow status string.
    pub status: String,
    /// What the orchestrator is waiting on; `None` means legitimately absent (not unknown).
    /// Populated from the `awaiting:` YAML frontmatter key.
    pub awaiting: Option<String>,
    /// Structured blocking issues parsed from the state file.
    pub blocking_issues: Vec<BlockingIssue>,
    /// Convergence round and finding counts; `None` when the §Session Resume Checkpoint
    /// section is absent from the state file.
    pub convergence: Option<ConvergenceMetrics>,
    /// Current cycle identifier; `None` when `current_cycle:` is absent from frontmatter.
    pub cycle: Option<String>,
    /// Forward-compatibility escape hatch for unrecognised frontmatter fields.
    /// MUST use `serde_yaml_ng::Value` — `serde_json::Value` is forbidden.
    pub custom_fields: HashMap<String, serde_yaml_ng::Value>,
}

/// A single blocking issue parsed from the factory state file.
#[derive(Debug, Clone)]
pub struct BlockingIssue {
    /// Issue identifier (e.g., `"BC-2.02.004"`).
    pub id: String,
    /// Human-readable description of the blocking issue.
    pub description: String,
    /// Severity of the blocking issue.
    pub severity: BlockingSeverity,
}

/// Severity level of a blocking issue.
///
/// Non-exhaustive: future severity levels may be added without breaking changes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BlockingSeverity {
    /// Showstopper; must be resolved before proceeding.
    Critical,
    /// Significant impact; should be resolved soon.
    High,
    /// Moderate impact; can proceed with caution.
    Medium,
    /// Minimal impact; informational.
    Low,
}

/// Convergence metrics extracted from the §Session Resume Checkpoint section.
#[derive(Debug, Clone)]
pub struct ConvergenceMetrics {
    /// Number of adversarial convergence rounds completed.
    pub round_count: u32,
    /// Total number of findings recorded in the current convergence pass.
    pub finding_count: u32,
}

/// Error returned by [`FactoryAdapter::read_state`].
///
/// Non-exhaustive: additional parse-specific variants may be added in future phases.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FactoryReadError {
    /// The factory state file does not exist (E-FACT-001).
    #[error("factory state file not found")]
    NotFound,
    /// The factory state file exists but cannot be parsed (E-FACT-002).
    #[error("failed to parse factory state: {0}")]
    ParseError(String),
}

/// Error returned by [`FactoryAdapter::subscribe`].
///
/// Non-exhaustive: additional variants may be added when live-watch is implemented.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FactorySubscribeError {
    /// The subscription could not be established.
    #[error("subscription failed: {0}")]
    SubscriptionFailed(String),
}

/// Stream of factory state change notifications.
///
/// Phase 1: always the empty stream from `futures::stream::empty()`.
/// Phase 3+: may be a `notify`-backed live stream.
pub type StateChangeStream = Pin<Box<dyn Stream<Item = FactoryState> + Send>>;
