use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sysadmin")]
#[command(about = "A shell for sysadmins - executable operational documentation", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to the .sysadmin file
    pub file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a .sysadmin file interactively (default)
    Run {
        /// Path to the .sysadmin file
        file: PathBuf,
    },

    /// Display all steps without executing (dry-run)
    DryRun {
        /// Path to the .sysadmin file
        file: PathBuf,
    },

    /// View the file as formatted documentation
    View {
        /// Path to the .sysadmin file
        file: PathBuf,
    },
}
