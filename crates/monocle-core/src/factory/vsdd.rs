//! VSDD Factory adapter implementation (BC-2.02.005, S-012).
//!
//! `VsddFactoryAdapter` detects VSDD factory projects by checking for
//! `.factory/STATE.md` with `document_type: pipeline-state` in its YAML frontmatter,
//! then parses the state file into a typed `FactoryState`.

use std::path::{Path, PathBuf};

use super::{
    FactoryAdapter, FactoryDetection, FactoryReadError, FactoryState, FactorySubscribeError,
    StateChangeStream,
};

/// Adapter for VSDD factory projects (`.factory/STATE.md`).
///
/// # Construction
///
/// Use [`VsddFactoryAdapter::new`]. No filesystem access occurs at construction time;
/// validation is deferred to [`detect`][FactoryAdapter::detect] and
/// [`read_state`][FactoryAdapter::read_state] (BC-2.02.005 PC-1).
pub struct VsddFactoryAdapter {
    // workspace_root is used by detect() (static method) and read_state(); the field
    // is needed for the impl but appears unused until todo!() bodies are filled in.
    #[allow(dead_code)]
    workspace_root: PathBuf,
    state_file: PathBuf,
}

impl VsddFactoryAdapter {
    /// Infallible constructor. Derives `state_file_path` as
    /// `workspace_root/.factory/STATE.md` with no filesystem access.
    ///
    /// Per BC-2.02.005 PC-1: "No validation is performed at construction time."
    pub fn new(workspace_root: PathBuf) -> Self {
        let state_file = workspace_root.join(".factory").join("STATE.md");
        Self {
            workspace_root,
            state_file,
        }
    }
}

// Stub implementations use `todo!()` intentionally; bodies will be filled by the implementer.
#[allow(clippy::todo)]
#[async_trait::async_trait]
impl FactoryAdapter for VsddFactoryAdapter {
    /// Detect whether `workspace_root` is a VSDD factory project.
    ///
    /// Returns `Some(FactoryDetection)` if and only if:
    /// 1. `workspace_root/.factory/STATE.md` exists, AND
    /// 2. The YAML frontmatter block (`---`-delimited) contains the key
    ///    `document_type: pipeline-state`.
    ///
    /// Returns `None` if the file is absent, the frontmatter is absent, or the key
    /// appears only in the document body (EC-021, BC-2.02.005 INV-1).
    fn detect(_workspace_root: &Path) -> Option<FactoryDetection>
    where
        Self: Sized,
    {
        todo!("S-012: detect VSDD factory — check .factory/STATE.md frontmatter for document_type: pipeline-state")
    }

    /// Returns `true` if the adapter's workspace root matches the detection result's workspace root.
    fn matches(&self, _detection: &FactoryDetection) -> bool {
        todo!("S-012: matches — compare workspace roots")
    }

    /// Path to this adapter's STATE.md file.
    fn state_file_path(&self) -> &Path {
        &self.state_file
    }

    /// Parse the VSDD factory state from `.factory/STATE.md`.
    ///
    /// - Returns `Err(FactoryReadError::NotFound)` if STATE.md does not exist (logs E-FACT-001).
    /// - Returns `Err(FactoryReadError::ParseError)` on parse failure (logs E-FACT-002).
    /// - Returns `Ok(FactoryState)` on success.
    ///
    /// `cycle: None` when `current_cycle:` is absent. `convergence: None` when
    /// §Session Resume Checkpoint section is absent. `"unknown"` MUST NOT appear
    /// as a default for absent fields (BC-2.02.005 PC-3).
    fn read_state(&self) -> Result<FactoryState, FactoryReadError> {
        todo!("S-012: read VSDD factory state — parse YAML frontmatter via serde_yaml_ng")
    }

    /// Phase 1 stub: returns an empty, immediately-terminating stream.
    ///
    /// No file watcher is instantiated (BC-2.02.005 invariant 3).
    fn subscribe(&self) -> Result<StateChangeStream, FactorySubscribeError> {
        Ok(Box::pin(futures::stream::empty()))
    }

    /// Returns the exact display name `"VSDD Factory"` (BC-2.02.005 INV-2).
    fn display_name(&self) -> &str {
        "VSDD Factory"
    }
}
