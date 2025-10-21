use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;

use crate::model::{Block as DocBlock, Document};

pub struct TuiApp {
    document: Document,
    current_step: usize,
    scroll_offset: usize,
}

impl TuiApp {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            current_step: 0,
            scroll_offset: 0,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10),     // Runbook area
                        Constraint::Length(3),   // Status bar
                    ])
                    .split(f.area());

                // Render the runbook content
                let runbook_content = self.render_runbook_content();
                let runbook = Paragraph::new(runbook_content)
                    .block(
                        Block::default()
                            .title("Runbook")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan)),
                    )
                    .wrap(Wrap { trim: true })
                    .scroll((self.scroll_offset as u16, 0));

                f.render_widget(runbook, chunks[0]);

                // Status bar
                let total_steps = self.document.step_count();
                let status_text = if total_steps > 0 {
                    format!(
                        " Step {}/{} | ↑↓: Scroll | n: Next | p: Previous | s: Shell | q: Quit ",
                        self.current_step.min(total_steps),
                        total_steps
                    )
                } else {
                    " No executable steps | q: Quit ".to_string()
                };

                let status = Paragraph::new(status_text)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().bg(Color::DarkGray));

                f.render_widget(status, chunks[1]);
            })?;

            // Handle input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break
                        }
                        KeyCode::Char('n') => {
                            self.next_step();
                        }
                        KeyCode::Char('p') => {
                            self.previous_step();
                        }
                        KeyCode::Char('s') => {
                            self.drop_to_shell(terminal)?;
                        }
                        KeyCode::Up => {
                            self.scroll_offset = self.scroll_offset.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            self.scroll_offset = self.scroll_offset.saturating_add(1);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn render_runbook_content(&self) -> Vec<Line> {
        let mut lines = Vec::new();
        let code_blocks = self.document.code_blocks();

        for (section_idx, section) in self.document.sections.iter().enumerate() {
            // Render header
            if let Some(header) = &section.header {
                let level = section.header_level.unwrap_or(1);
                let header_style = match level {
                    1 => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    2 => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                };

                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("{} {}", "#".repeat(level as usize), header),
                    header_style,
                )));
                lines.push(Line::from(""));
            }

            // Render blocks
            for block in &section.blocks {
                match block {
                    DocBlock::Text(text) => {
                        for line in text.lines() {
                            if !line.trim().is_empty() {
                                lines.push(Line::from(line.to_string()));
                            }
                        }
                        lines.push(Line::from(""));
                    }
                    DocBlock::Code(code) => {
                        // Find which step number this is
                        let step_num = code_blocks
                            .iter()
                            .position(|c| *c == code)
                            .map(|i| i + 1)
                            .unwrap_or(0);

                        let is_current = step_num == self.current_step;
                        let is_completed = step_num < self.current_step;

                        let step_style = if is_current {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else if is_completed {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };

                        let marker = if is_completed {
                            "✓"
                        } else if is_current {
                            "→"
                        } else {
                            " "
                        };

                        lines.push(Line::from(Span::styled(
                            format!("{} Step {} [{}]:", marker, step_num, code.language),
                            step_style,
                        )));

                        let code_style = if is_current {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };

                        for line in code.content.lines() {
                            lines.push(Line::from(Span::styled(
                                format!("  {}", line),
                                code_style,
                            )));
                        }

                        lines.push(Line::from(""));
                    }
                }
            }
        }

        lines
    }

    fn next_step(&mut self) {
        let total_steps = self.document.step_count();
        if self.current_step < total_steps {
            self.current_step += 1;
            self.auto_scroll_to_current_step();
        }
    }

    fn previous_step(&mut self) {
        if self.current_step > 0 {
            self.current_step = self.current_step.saturating_sub(1);
            self.auto_scroll_to_current_step();
        }
    }

    fn auto_scroll_to_current_step(&mut self) {
        // Find the line number where the current step is
        let code_blocks = self.document.code_blocks();
        if self.current_step == 0 || self.current_step > code_blocks.len() {
            return;
        }

        let target_code = code_blocks[self.current_step - 1];
        let mut line_count = 0;

        for section in &self.document.sections {
            // Count header lines
            if section.header.is_some() {
                line_count += 3; // blank, header, blank
            }

            // Count lines in blocks
            for block in &section.blocks {
                match block {
                    DocBlock::Text(text) => {
                        line_count += text.lines().count() + 1; // +1 for blank after
                    }
                    DocBlock::Code(code) => {
                        if code == target_code {
                            // Found it! Set scroll to show this step near the top
                            // Leave some context lines above (5 lines)
                            self.scroll_offset = line_count.saturating_sub(5);
                            return;
                        }
                        line_count += 1; // Step header
                        line_count += code.content.lines().count();
                        line_count += 1; // blank after
                    }
                }
            }
        }
    }

    fn drop_to_shell(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        // Properly restore terminal before spawning shell
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        // Clear screen and show current step
        print!("\x1B[2J\x1B[1;1H"); // Clear screen, move to top
        
        let code_blocks = self.document.code_blocks();
        if self.current_step > 0 && self.current_step <= code_blocks.len() {
            let code = code_blocks[self.current_step - 1];
            println!("{}", "=".repeat(60));
            println!("Current step [{}]:", code.language);
            for line in code.content.lines() {
                println!("  {}", line);
            }
            println!("{}", "=".repeat(60));
            println!("\nDropping to shell. Type 'exit' or press Ctrl-D to return.\n");
        } else {
            println!("\nDropping to shell. Type 'exit' or press Ctrl-D to return.\n");
        }

        // Spawn shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let status = std::process::Command::new(&shell)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()?;

        if let Some(code) = status.code() {
            if code == 130 {
                // User Ctrl-C'd in shell, don't return to TUI
                std::process::exit(130);
            }
        }

        println!("\nReturning to TUI...");
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Re-enter TUI mode
        enable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        terminal.hide_cursor()?;
        terminal.clear()?;

        Ok(())
    }
}
