//! Engine module implementations for monocle-runtime (S-015).
//!
//! Exposes [`ClaudeCodeModule`] and its supporting types for use by the daemon
//! startup path and integration tests.

pub mod claude_code;

pub use claude_code::{
    ClaudeCodeModule, EngineVersion, PreflightError, SessionHandle, SpawnArgs, SpawnError,
};
