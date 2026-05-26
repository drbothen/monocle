//! VSDD Factory adapter implementation (BC-2.02.005, S-012).
//!
//! `VsddFactoryAdapter` detects VSDD factory projects by checking for
//! `.factory/STATE.md` with `document_type: pipeline-state` in its YAML frontmatter,
//! then parses the state file into a typed `FactoryState`.

use std::collections::HashMap;
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
    /// Stored for future use by Phase 3 multi-adapter registry (BC-2.02.005 PC-1).
    _workspace_root: PathBuf,
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
            _workspace_root: workspace_root,
            state_file,
        }
    }
}

/// Extract the YAML frontmatter block from Markdown content.
///
/// Returns `Some(frontmatter_text)` if the content starts with `---\n` and has a
/// closing `---` line. Returns `None` if no valid frontmatter block is found.
fn extract_frontmatter(content: &str) -> Option<&str> {
    // Must start with "---\n" or "---\r\n"
    let after_open = content.strip_prefix("---\n").or_else(|| content.strip_prefix("---\r\n"))?;

    // Find the closing "---" on its own line
    // We search for "\n---" followed by end-of-string, newline, or "\r\n"
    let close_pos = after_open.find("\n---")?;

    // The frontmatter is the text between the opening and closing delimiters
    Some(&after_open[..close_pos])
}

/// Parse a simple scalar value from a frontmatter line.
///
/// Applies AC-013 guards in order:
/// 1. Skip continuation lines (lines starting with whitespace) — caller handles
/// 2. Return None for empty values (after trimming)
/// 3. Return None for flow-style lists (value starts with `[`)
/// 4. Return None for block scalar markers (value is `|` or `>`, possibly with trailing text)
/// 5. Unquote YAML quoted scalars (strip surrounding `'` or `"`)
fn parse_scalar_value(raw_value: &str) -> Option<String> {
    let v = raw_value.trim();

    // Guard 2: empty value
    if v.is_empty() {
        return None;
    }

    // Guard 3: flow-style list
    if v.starts_with('[') {
        return None;
    }

    // Guard 4: block scalar markers
    if v == "|" || v.starts_with("| ") || v.starts_with("|+") || v.starts_with("|-")
        || v == ">" || v.starts_with("> ") || v.starts_with(">+") || v.starts_with(">-")
    {
        return None;
    }

    // Unquote: strip matching surrounding quotes (EC-022)
    let unquoted = if (v.starts_with('"') && v.ends_with('"') && v.len() >= 2)
        || (v.starts_with('\'') && v.ends_with('\'') && v.len() >= 2)
    {
        &v[1..v.len() - 1]
    } else {
        v
    };

    // Guard 2 again: after unquoting, value may be empty
    if unquoted.is_empty() {
        return None;
    }

    Some(unquoted.to_string())
}

/// Parse the YAML frontmatter block into a map of key → optional scalar value.
///
/// Uses a line-by-line parser (NOT a full YAML parser) with AC-013 guards.
/// Keys with complex values (multi-line, flow maps/lists) store `None`.
fn parse_frontmatter_fields(frontmatter: &str) -> HashMap<String, Option<String>> {
    let mut fields = HashMap::new();

    for line in frontmatter.lines() {
        // Guard 1 (AC-013 guard 1): skip continuation/indented lines
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }

        // Skip empty lines and comment lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse "key: value" — split on first ':'
        if let Some((key, rest)) = line.split_once(':') {
            let key = key.trim().to_string();
            if key.is_empty() {
                continue;
            }
            // rest is everything after the first ':', including the leading space
            let value_str = rest.strip_prefix(' ').unwrap_or(rest);
            let parsed = parse_scalar_value(value_str);
            fields.insert(key, parsed);
        }
    }

    fields
}

/// Known frontmatter keys that are handled as named fields in `FactoryState`.
///
/// These keys are consumed into named fields and NOT placed into `custom_fields`.
const KNOWN_KEYS: &[&str] = &[
    "document_type",
    "phase",
    "status",
    "awaiting",
    "current_cycle",
    "convergence",
];

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
    fn detect(workspace_root: &Path) -> Option<FactoryDetection>
    where
        Self: Sized,
    {
        let state_file = workspace_root.join(".factory").join("STATE.md");

        // Step 1: file must exist
        if !state_file.exists() {
            return None;
        }

        // Step 2: read the file
        let content = match std::fs::read_to_string(&state_file) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    path = %state_file.display(),
                    error = %e,
                    "VsddFactoryAdapter::detect: failed to read STATE.md"
                );
                return None;
            }
        };

        // Step 3: extract frontmatter only (EC-021: body match is insufficient)
        let frontmatter = extract_frontmatter(&content)?;

        // Step 4: check for `document_type: pipeline-state` in frontmatter
        let fields = parse_frontmatter_fields(frontmatter);
        let is_vsdd = fields
            .get("document_type")
            .and_then(|v| v.as_deref())
            == Some("pipeline-state");

        if !is_vsdd {
            return None;
        }

        Some(FactoryDetection {
            display_name: "VSDD Factory".to_string(),
            workspace_root: workspace_root.to_path_buf(),
            state_file,
        })
    }

    /// Returns `true` if `detection.display_name == "VSDD Factory"`.
    fn matches(&self, detection: &FactoryDetection) -> bool {
        detection.display_name == "VSDD Factory"
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
        // Step 1: check existence → E-FACT-001
        if !self.state_file.exists() {
            tracing::error!(
                error_code = "E-FACT-001",
                path = %self.state_file.display(),
                "factory state file not found"
            );
            return Err(FactoryReadError::NotFound);
        }

        // Step 2: read content
        let content = std::fs::read_to_string(&self.state_file).map_err(|e| {
            tracing::error!(
                error_code = "E-FACT-001",
                path = %self.state_file.display(),
                error = %e,
                "failed to read factory state file"
            );
            FactoryReadError::NotFound
        })?;

        // Step 3: extract frontmatter → E-FACT-002 if absent
        let frontmatter = extract_frontmatter(&content).ok_or_else(|| {
            tracing::error!(
                error_code = "E-FACT-002",
                path = %self.state_file.display(),
                "factory state file has no valid YAML frontmatter block"
            );
            FactoryReadError::ParseError("no valid YAML frontmatter block found".to_string())
        })?;

        // Step 4: parse fields from frontmatter
        let fields = parse_frontmatter_fields(frontmatter);

        // Extract required fields: phase and status
        let phase = fields
            .get("phase")
            .and_then(|v| v.clone())
            .unwrap_or_default();

        let status = fields
            .get("status")
            .and_then(|v| v.clone())
            .unwrap_or_default();

        // Extract optional fields per AC-012 (None for absent, NOT "unknown")
        let awaiting = fields.get("awaiting").and_then(|v| v.clone());
        let cycle = fields.get("current_cycle").and_then(|v| v.clone());

        // convergence: None in Phase 1 (§Session Resume Checkpoint parsing deferred to Phase 3)
        let convergence = None;

        // blocking_issues: empty Vec in Phase 1 (complex YAML structure, not a scalar)
        let blocking_issues = Vec::new();

        // custom_fields: all frontmatter keys NOT in KNOWN_KEYS that have a parsed value
        let custom_fields: HashMap<String, serde_yaml_ng::Value> = fields
            .iter()
            .filter(|(k, _)| !KNOWN_KEYS.contains(&k.as_str()))
            .filter_map(|(k, v)| {
                v.as_ref()
                    .map(|s| (k.clone(), serde_yaml_ng::Value::String(s.clone())))
            })
            .collect();

        tracing::debug!(
            path = %self.state_file.display(),
            phase = %phase,
            status = %status,
            cycle = ?cycle,
            awaiting = ?awaiting,
            "factory state parsed successfully"
        );

        Ok(FactoryState {
            phase,
            status,
            awaiting,
            blocking_issues,
            convergence,
            cycle,
            custom_fields,
        })
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
