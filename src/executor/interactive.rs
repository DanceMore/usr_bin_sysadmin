use anyhow::{Context, Result};
use std::env;
use std::process::Command;

use crate::model::{Block, Document};
use crate::ui::Renderer;

pub struct InteractiveExecutor {
    renderer: Renderer,
}

impl InteractiveExecutor {
    pub fn new() -> Self {
        Self {
            renderer: Renderer::new(),
        }
    }

    /// Execute a document interactively
    pub fn execute(&mut self, doc: &Document) -> Result<()> {
        let total_steps = doc.step_count();
        self.renderer.set_total_steps(total_steps);

        for section in &doc.sections {
            // Render section header if present
            if let Some(header) = &section.header {
                let level = section.header_level.unwrap_or(1);
                self.renderer.render_header(header, level)?;
            }

            // Render each block in the section
            for block in &section.blocks {
                match block {
                    Block::Text(text) => {
                        self.renderer.render_text(text)?;
                    }
                    Block::Code(code) => {
                        self.renderer.render_code(code)?;

                        // Drop into a sub-shell for the user to run the command
                        self.drop_to_shell()?;
                    }
                }
            }
        }

        self.renderer.render_completion()?;
        Ok(())
    }

    /// Drop into a sub-shell for the user to execute commands
    fn drop_to_shell(&self) -> Result<()> {
        self.renderer.render_shell_prompt()?;

        // Get the user's shell, default to bash
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

        // Determine shell type from path
        let shell_name = std::path::Path::new(&shell)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("bash");

        // Set a custom prompt to make it obvious we're in a sysadmin sub-shell
        let custom_prompt = "%F{magenta}[sysadmin]%f $ ";
        let custom_ps1 = "\x1b[1;35m[sysadmin]\x1b[0m $ ";
        
        // Spawn a sub-shell with custom prompt
        let mut cmd = Command::new(&shell);
        
        // Set prompt based on shell type
        match shell_name {
            "zsh" => {
                cmd.env("PROMPT", custom_prompt);
                // Also set PS1 for compatibility
                cmd.env("PS1", custom_ps1);
            }
            "fish" => {
                // Fish uses a function, but we can try setting a simple prompt
                cmd.env("fish_greeting", "");
                // Fish doesn't use PS1, we'd need to write a function
                // For now, just let fish use its default
            }
            _ => {
                // bash, sh, and most others use PS1
                cmd.env("PS1", custom_ps1);
            }
        }

        let status = cmd
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .with_context(|| format!("Failed to spawn shell: {}", shell))?;

        if !status.success() {
            if let Some(code) = status.code() {
                if code == 130 {
                    // User pressed Ctrl-C in the shell
                    println!("\nInterrupted.");
                    std::process::exit(130);
                }
            }
        }

        println!(); // Add spacing after shell exits
        Ok(())
    }
}

impl Default for InteractiveExecutor {
    fn default() -> Self {
        Self::new()
    }
}
