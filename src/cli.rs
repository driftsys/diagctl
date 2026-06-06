use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "diagctl",
    version,
    about = "Automates the tech-diagramming quality gate"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the diagram quality gate on an SVG file
    Check { file: PathBuf },
    /// (not yet implemented) Conservative SVGO-style optimization
    Optimize { file: PathBuf },
    /// (not yet implemented) ASCII grid render + width-aware alignment
    Ascii { file: PathBuf },
    /// (not yet implemented) Re-render and byte-diff freshness check
    Freshness { file: PathBuf },
}
