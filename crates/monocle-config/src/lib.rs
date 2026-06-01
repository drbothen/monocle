//! `monocle-config` — configuration schema, persistence, and CCR detection.
//!
//! Public API surface for S-030 / SS-07:
//! - [`MonocleConfig`]: Config schema v1 (BC-2.07.002).
//! - [`HarnessProfile`]: Harness profile entry (BC-2.07.002 PC-2).
//! - [`load_config`]: Load from disk with resilient default fallback (BC-2.07.003).
//! - [`write_config`]: Atomic write via `tempfile::persist` (BC-2.07.001).
//! - [`detect_ccr`]: CCR binary detection — config path then PATH fallback (BC-2.07.006).
//! - [`ConfigError`]: Error taxonomy for all config operations.

#![forbid(unsafe_code)]

mod config;
mod detect_ccr;
mod error;

pub use config::{load_config, resolve_profile_for_dir, write_config, HarnessProfile, MonocleConfig};
pub use detect_ccr::detect_ccr;
pub use error::ConfigError;
