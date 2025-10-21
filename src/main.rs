use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

mod cli;
mod executor;
mod model;
mod parser;
mod ui;

use cli::{Cli, Commands};
use executor::InteractiveExecutor;
use parser::SysadminParser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine which file to process
    let file_path = match &cli.command {
        Some(Commands::Run { file }) => file,
        Some(Commands::DryRun { file }) => file,
        Some(Commands::View { file }) => file,
        None => {
            if let Some(file) = &cli.file {
                file
            } else {
                eprintln!("Error: No file specified");
                eprintln!();
                eprintln!("Usage: sysadmin <file.sysadmin>");
                eprintln!("       sysadmin run <file.sysadmin>");
                eprintln!("       sysadmin dry-run <file.sysadmin>");
                eprintln!("       sysadmin view <file.sysadmin>");
                std::process::exit(1);
            }
        }
    };

    // Read the file
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Parse the document
    let document =
        SysadminParser::parse(&content).context("Failed to parse .sysadmin document")?;

    // Execute based on command
    match &cli.command {
        None | Some(Commands::Run { .. }) => {
            // Default: interactive execution
            let mut executor = InteractiveExecutor::new();
            executor.execute(&document)?;
        }
        Some(Commands::DryRun { .. }) => {
            // Print all steps
            println!("Dry run - {} steps found:\n", document.step_count());

            for (idx, code) in document.code_blocks().iter().enumerate() {
                println!("Step {} [{}]:", idx + 1, code.language);
                for line in code.content.lines() {
                    println!("  {}", line);
                }
                println!();
            }
        }
        Some(Commands::View { .. }) => {
            // Just print the content as-is
            print!("{}", content);
        }
    }

    Ok(())
}
