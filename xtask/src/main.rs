//! monocle xtask runner.
//!
//! Dispatches project-level developer tasks that are too complex for a simple
//! shell alias. Subcommands are implemented in sibling modules.
//!
//! Usage:
//!   cargo run -p xtask -- <subcommand> [options]
//!
//! Available subcommands:
//!   dtu-fidelity   Run the DTU fidelity oracle against the 25-fixture corpus.
//!
//! Source authority:
//! - S-DTU-001 AC-007 (`cargo xtask dtu-fidelity` oracle)
//! - dtu-assessment.md v1.7.5 §Tooling (line 247)

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod dtu_fidelity;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Monocle project xtask runner.
#[derive(Parser)]
#[command(name = "xtask", about = "Monocle project task runner", version)]
struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    command: Commands,
}

/// Available xtask subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Run the DTU fidelity oracle against the 25-fixture corpus.
    ///
    /// Drives the monocle-test-harness DTU clone in-process over all 25
    /// fixtures and computes aggregate FixtureScore. Exits 0 iff mean ≥ 0.95.
    DtuFidelity(dtu_fidelity::DtuFidelityArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialise structured logging. Respects RUST_LOG env var.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::DtuFidelity(args) => dtu_fidelity::run(args).await,
    }
}
