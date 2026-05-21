//! DTU clone server — axum HTTP server lifecycle and shared state.
//!
//! Provides `CloneState` (shared across handlers) and `start_server`
//! (binds the axum router to a port and begins serving).
//!
//! Source authority:
//! - AC-005 (Rust binary, dtu-claude-code-hooks-v1)
//! - AC-006 (MONOCLE_HOOK_ENDPOINT_BASE and MONOCLE_NO_AUTOSTART env vars)
//! - dtu-assessment.md v1.7.5 §Packaging Decision §Environment Variable Overrides
//! - BC-HOOK-013 (lock file discovery), BC-HOOK-015 (token extraction)

use std::net::SocketAddr;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::dtu::lock_reader::LockFileInfo;

/// Shared state passed to all 5 axum hook endpoint handlers.
///
/// Contains the resolved daemon connection info (read from the monocle lock file
/// per BC-HOOK-015) and the reqwest HTTP client for proxying POSTs.
///
/// BC-HOOK-013 (lock file scan), BC-HOOK-015 (token extraction),
/// BC-HOOK-016 (auth header value)
#[derive(Debug, Clone)]
pub struct CloneState {
    /// HTTP client used to POST synthesized payloads to the monocle daemon.
    /// Initialized once and shared across all handler invocations.
    pub client: reqwest::Client,
    /// Resolved monocle daemon connection details from the lock file.
    pub daemon: LockFileInfo,
    /// Base URL for daemon hook POSTs, e.g. `http://127.0.0.1:7860`.
    /// Derived from `daemon.port` unless `MONOCLE_HOOK_ENDPOINT_BASE` overrides.
    /// AC-006 (env var override)
    pub endpoint_base: String,
}

/// Start the DTU clone HTTP server on the given address.
///
/// The server binds to `addr`, registers the 5 hook endpoint routes, and starts
/// serving. This function does not return until the server shuts down.
///
/// AC-005 (binary artifact), AC-006 (MONOCLE_NO_AUTOSTART)
pub async fn start_server(addr: SocketAddr, state: CloneState) -> Result<()> {
    use crate::dtu::endpoints::build_router;
    use tokio::net::TcpListener;

    let router = build_router(state);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// hooks-settings.json structure — BC-HOOK-007, BC-HOOK-009, BC-HOOK-040
// ──────────────────────────────────────────────────────────────────────────────

/// A single hook entry in hooks-settings.json.
///
/// Per Claude Code hooks specification: each hook type maps to a list of
/// hook matchers (or for the DTU clone, a single URL target).
///
/// BC-HOOK-040: struct-based stable serialization (not HashMap).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HookEntry {
    /// The DTU clone HTTP endpoint URL for this hook type.
    hooks: Vec<HookTarget>,
}

/// A single hook target URL.
///
/// BC-HOOK-040: struct-based stable serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HookTarget {
    /// Full URL of the DTU clone endpoint.
    r#type: String,
    /// URL for the hook POST.
    command: String,
}

/// The full hooks-settings.json content structure.
///
/// Per Claude Code settings format:
/// ```json
/// { "hooks": { "PreToolUse": [...], "Notification": [...], ... } }
/// ```
///
/// BC-HOOK-040: struct-based stable serialization (no HashMap key reordering).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HooksSettings {
    /// The hooks map — exactly 5 entries per BC-HOOK-007.
    hooks: HooksMap,
}

/// The 5-key hooks map in hooks-settings.json.
///
/// Ordered struct fields give stable JSON key ordering per BC-HOOK-040.
/// BC-HOOK-012: role-invariant — same structure regardless of session role.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HooksMap {
    #[serde(rename = "PreToolUse")]
    pre_tool_use: Vec<HookCommandEntry>,
    #[serde(rename = "Notification")]
    notification: Vec<HookCommandEntry>,
    #[serde(rename = "Stop")]
    stop: Vec<HookCommandEntry>,
    #[serde(rename = "SessionStart")]
    session_start: Vec<HookCommandEntry>,
    #[serde(rename = "UserPromptSubmit")]
    user_prompt_submit: Vec<HookCommandEntry>,
}

/// A single hook command entry in the hooks-settings.json list.
///
/// Claude Code hooks format: `{"matcher": "<glob>", "hooks": [{"type": "command", "command": "<cmd>"}]}`
/// The DTU clone uses `curl` as the hook command to POST to the DTU server.
///
/// BC-HOOK-007 (hook schema), BC-HOOK-027 (settings injection via --settings flag)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HookCommandEntry {
    /// Glob matcher — empty string matches all tools.
    matcher: String,
    /// The hook actions list.
    hooks: Vec<HookAction>,
}

/// A single hook action — the actual command Claude Code executes.
///
/// BC-HOOK-040: struct-based, stable field ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HookAction {
    /// Always "command" for DTU clone hook actions.
    r#type: String,
    /// The curl command to POST to the DTU clone endpoint.
    command: String,
}

/// Default DTU clone port for hooks-settings.json.
///
/// Role-invariant per BC-HOOK-012: same port used for all sessions.
/// The actual port is overridden at runtime by MONOCLE_HOOK_ENDPOINT_BASE
/// per AC-006. For the hooks-settings.json template, we use the default.
const DTU_DEFAULT_PORT: u16 = 7860;

/// Build the Node.js hook command for a given hook endpoint path.
///
/// Uses the `req.write(body) + req.end()` pattern per BC-HOOK-037, matching
/// the gene-source hook.js pattern (any-context-lazyclaude).
///
/// HIGH-2 / BC-HOOK-014 §PC-1: The hook command reads `MONOCLE_DTU_PORT` env var
/// with fallback to `DTU_DEFAULT_PORT`. This allows tests and CI to override the
/// DTU server port without rebuilding hooks-settings.json.
///
/// BC-HOOK-023: Content-Type and Content-Length headers always set.
/// BC-HOOK-022: Notification timeout = 2000ms; other hooks = 300ms.
/// BC-HOOK-037: req.write(body) + req.end() pattern.
fn hook_node_command(path: &str, timeout_ms: u32) -> String {
    // Node.js inline hook script per gene-source pattern.
    // Uses http.request with explicit Content-Type + Content-Length headers.
    // BC-HOOK-008: no HTML escaping — the => in arrow functions must appear literally.
    // BC-HOOK-014 §PC-1: port read from process.env.MONOCLE_DTU_PORT with fallback
    // to the default port. This matches the gene-source pattern of reading env vars
    // at hook invocation time (not hardcoding the port at hooks-settings.json generation).
    format!(
        r#"node -e "const b=require('fs').readFileSync('/dev/stdin');const h=require('http');const port=parseInt(process.env.MONOCLE_DTU_PORT||'{default_port}',10);const opts={{hostname:'127.0.0.1',port:port,path:'{path}',method:'POST',timeout:{timeout},headers:{{'Content-Type':'application/json','Content-Length':Buffer.byteLength(b)}}}};const req=h.request(opts,()=>{{}});req.write(b);req.end()""#,
        default_port = DTU_DEFAULT_PORT,
        path = path,
        timeout = timeout_ms,
    )
}

/// Timeout for Notification hooks per BC-HOOK-022.
const NOTIFICATION_TIMEOUT_MS: u32 = 2000;

/// Timeout for all other hooks per BC-HOOK-022.
const DEFAULT_HOOK_TIMEOUT_MS: u32 = 300;

/// Build the canonical hooks-settings.json content.
///
/// Role-invariant per BC-HOOK-012: content is identical across all sessions.
/// BC-HOOK-007: exactly 5 hook keys (PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit).
/// BC-HOOK-022: Notification timeout = 2000ms; others = 300ms.
/// BC-HOOK-040: struct-based (not HashMap) for stable key ordering.
fn build_hooks_settings() -> HooksSettings {
    use crate::dtu::endpoints::paths;

    let make_entry = |path: &str, timeout_ms: u32| -> HookCommandEntry {
        HookCommandEntry {
            matcher: String::new(), // empty matcher = match all tools
            hooks: vec![HookAction {
                r#type: "command".to_string(),
                command: hook_node_command(path, timeout_ms),
            }],
        }
    };

    HooksSettings {
        hooks: HooksMap {
            pre_tool_use: vec![make_entry(paths::PRE_TOOL_USE, DEFAULT_HOOK_TIMEOUT_MS)],
            // BC-HOOK-022: Notification gets 2000ms timeout.
            notification: vec![make_entry(paths::NOTIFICATION, NOTIFICATION_TIMEOUT_MS)],
            stop: vec![make_entry(paths::STOP, DEFAULT_HOOK_TIMEOUT_MS)],
            session_start: vec![make_entry(paths::SESSION_START, DEFAULT_HOOK_TIMEOUT_MS)],
            user_prompt_submit: vec![make_entry(paths::PROMPT_SUBMIT, DEFAULT_HOOK_TIMEOUT_MS)],
        },
    }
}

/// Write the hooks-settings.json file atomically to `runtime_dir` using `tempfile::persist`.
///
/// Returns the path to the written file.
///
/// BC-HOOK-009 (path + mode), BC-HOOK-010 (single file per runtimeDir),
/// BC-HOOK-011 (no cleanup), BC-HOOK-039 (atomic write), BC-HOOK-040 (struct not HashMap),
/// BC-HOOK-041 (filename assertion)
pub fn write_hooks_settings_file(runtime_dir: &std::path::Path) -> Result<std::path::PathBuf> {
    let settings = build_hooks_settings();

    // BC-HOOK-008: no HTML escaping in JSON output.
    // serde_json::to_string uses HTML-escaping by default; we must use a formatter
    // that disables it. The `serde_json::ser::PrettyFormatter` approach:
    let mut buf = Vec::new();
    let mut ser =
        serde_json::Serializer::with_formatter(&mut buf, serde_json::ser::PrettyFormatter::new());
    serde::Serialize::serialize(&settings, &mut ser)?;
    let json_bytes = buf;

    // BC-HOOK-039: atomic write via tempfile::persist — no torn reads possible.
    // Create a NamedTempFile in the same directory to ensure same-filesystem move.
    //
    // HIGH-4 / NFR-009: Set permissions at temp file creation time (not after write)
    // to eliminate the race window where the file exists with default permissions.
    // tempfile::Builder::new().permissions() sets mode at open(2) call on Unix.
    #[cfg(unix)]
    let tmp = {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        tempfile::Builder::new()
            .permissions(permissions)
            .tempfile_in(runtime_dir)?
    };
    #[cfg(not(unix))]
    let tmp = tempfile::NamedTempFile::new_in(runtime_dir)?;

    let mut tmp = tmp;
    use std::io::Write;
    tmp.write_all(&json_bytes)?;

    // BC-HOOK-041: filename must end in hooks-settings.json.
    let target_path = runtime_dir.join("hooks-settings.json");

    // Atomically persist the temp file to the target path.
    // If target already exists it will be replaced atomically (BC-HOOK-010).
    tmp.persist(&target_path)?;

    Ok(target_path)
}
