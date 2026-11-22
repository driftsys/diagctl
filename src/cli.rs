use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "diagctl",
    version = crate::contract::VERSION_LINE,
    about = "Automates the tech-diagramming quality gate"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the diagram quality gate on an SVG file
    Check {
        /// Where the SVG is displayed: inlined in HTML, or through a Markdown image
        #[arg(long, value_enum, default_value_t = Target::Html)]
        target: Target,
        file: PathBuf,
    },
    /// (not yet implemented) Conservative SVGO-style optimization
    Optimize { file: PathBuf },
    /// (not yet implemented) Re-render and byte-diff freshness check
    Freshness { file: PathBuf },
}

/// Display target for `check`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Target {
    /// SVG inlined in an HTML page
    Html,
    /// SVG shown through a Markdown image (GitHub, GitLab, mdbook)
    Markdown,
}
