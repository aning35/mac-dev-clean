use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "macdevclean")]
#[command(about = "MacDevClean is a disk cleanup tool for macOS developers", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan for caches and build artifacts
    Scan {
        /// Output scan results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Clean up caches and build artifacts
    Clean {
        /// Safely clean default removable items without confirmation
        #[arg(long)]
        safe: bool,
        /// Thoroughly clean all items (will prompt for dangerous items)
        #[arg(long)]
        deep: bool,
        /// Dry run, do not actually delete anything
        #[arg(long)]
        dry_run: bool,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Launch the Web UI
    Ui,
    /// View cleanup history
    History,
    /// Manage AI model caches specifically
    Ai {
        #[command(subcommand)]
        command: AiCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize default configuration
    Init,
    /// Edit configuration
    Edit,
}

#[derive(Subcommand)]
pub enum AiCommands {
    /// Scan AI models
    Scan,
    /// Find duplicate AI models
    Duplicates,
}
