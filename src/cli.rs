use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "annotator", about = "Code review annotation tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// TUI review mode
    Review {
        /// Path to repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Adjust annotation positions after code changes
    Adjust {
        /// Path to repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Automatically resolve non-conflicting adjustments
        #[arg(long)]
        auto_resolve: bool,
    },
    /// Export annotations
    Export {
        /// Path to repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Export format
        #[arg(long, default_value = "markdown")]
        format: ExportFormat,
    },
    /// Show review progress
    Status {
        /// Path to repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Clone, clap::ValueEnum)]
pub enum ExportFormat {
    Markdown,
    Json,
}
