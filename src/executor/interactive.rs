use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

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

                        // Wait for user confirmation
                        self.wait_for_continue()?;
                    }
                }
            }
        }

        self.renderer.render_completion()?;
        Ok(())
    }

    fn wait_for_continue(&self) -> Result<()> {
        self.renderer.render_prompt()?;

        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Enter => {
                        return Ok(());
                    }
                    KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        println!("\n\nInterrupted.");
                        std::process::exit(130); // Standard exit code for SIGINT
                    }
                    _ => {
                        // Ignore other keys
                    }
                }
            }
        }
    }
}

impl Default for InteractiveExecutor {
    fn default() -> Self {
        Self::new()
    }
}
