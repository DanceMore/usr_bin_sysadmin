use anyhow::Result;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{stdout, Write};

use crate::model::CodeBlock;

pub struct Renderer {
    current_step: usize,
    total_steps: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            total_steps: 0,
        }
    }

    pub fn set_total_steps(&mut self, total: usize) {
        self.total_steps = total;
    }

    /// Render a section header
    pub fn render_header(&self, header: &str, level: u32) -> Result<()> {
        let mut stdout = stdout();

        // Add spacing
        writeln!(stdout)?;

        // Different colors for different header levels
        let color = match level {
            1 => Color::Cyan,
            2 => Color::Blue,
            _ => Color::White,
        };

        execute!(
            stdout,
            SetForegroundColor(color),
            Print(format!("{} {}", "#".repeat(level as usize), header)),
            ResetColor,
            Print("\n")
        )?;

        writeln!(stdout)?;
        stdout.flush()?;
        Ok(())
    }

    /// Render documentation text
    pub fn render_text(&self, text: &str) -> Result<()> {
        let mut stdout = stdout();

        // Simple text rendering - just print it
        for line in text.lines() {
            if !line.trim().is_empty() {
                writeln!(stdout, "{}", line)?;
            }
        }

        stdout.flush()?;
        Ok(())
    }

    /// Render a code block with syntax highlighting (simple version)
    pub fn render_code(&mut self, code: &CodeBlock) -> Result<()> {
        let mut stdout = stdout();

        self.current_step += 1;

        // Step indicator
        writeln!(stdout)?;
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(format!(
                "Step {}/{} [{}]:",
                self.current_step, self.total_steps, code.language
            )),
            ResetColor,
            Print("\n")
        )?;

        // Code content with indentation
        execute!(stdout, SetForegroundColor(Color::Green))?;
        for line in code.content.lines() {
            writeln!(stdout, "  {}", line)?;
        }
        execute!(stdout, ResetColor)?;

        writeln!(stdout)?;
        stdout.flush()?;
        Ok(())
    }

    /// Render the shell prompt
    pub fn render_shell_prompt(&self) -> Result<()> {
        let mut stdout = stdout();

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("→ Dropping into shell. Run the command above, then type "),
            SetForegroundColor(Color::Yellow),
            Print("exit"),
            SetForegroundColor(Color::Cyan),
            Print(" or press "),
            SetForegroundColor(Color::Yellow),
            Print("Ctrl-D"),
            SetForegroundColor(Color::Cyan),
            Print(" to continue."),
            ResetColor,
            Print("\n")
        )?;

        writeln!(stdout)?;
        stdout.flush()?;
        Ok(())
    }

    /// Render completion message
    pub fn render_completion(&self) -> Result<()> {
        let mut stdout = stdout();

        writeln!(stdout)?;
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("✓ All steps completed!"),
            ResetColor,
            Print("\n")
        )?;

        writeln!(stdout)?;
        stdout.flush()?;
        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
