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
                        Constraint::Min(10),   // Runbook area
                        Constraint::Length(3), // Status bar
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
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Char('n') => self.next_step(),
                        KeyCode::Char('p') => self.previous_step(),
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

                // Add visual separator for top-level sections
                if level == 1 && section_idx > 0 {
                    lines.push(Line::from(Span::styled(
                        "─".repeat(60),
                        Style::default().fg(Color::DarkGray),
                    )));
                }

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
                                // Check for warning/danger markers
                                let styled_line = if line.to_uppercase().contains("WARNING")
                                    || line.to_uppercase().contains("WARN")
                                {
                                    Line::from(vec![
                                        Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
                                        Span::styled(line, Style::default().fg(Color::Yellow)),
                                    ])
                                } else if line.to_uppercase().contains("DANGER")
                                    || line.to_uppercase().contains("CRITICAL")
                                {
                                    Line::from(vec![
                                        Span::styled(
                                            "⚠ ",
                                            Style::default()
                                                .fg(Color::Red)
                                                .add_modifier(Modifier::BOLD),
                                        ),
                                        Span::styled(
                                            line,
                                            Style::default()
                                                .fg(Color::Red)
                                                .add_modifier(Modifier::BOLD),
                                        ),
                                    ])
                                } else if line.to_uppercase().contains("NOTE")
                                    || line.to_uppercase().contains("INFO")
                                {
                                    Line::from(vec![
                                        Span::styled("ℹ ", Style::default().fg(Color::Blue)),
                                        Span::styled(line, Style::default().fg(Color::Gray)),
                                    ])
                                } else {
                                    Line::from(line.to_string())
                                };
                                lines.push(styled_line);
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

                        // Step header styling
                        let (marker, step_style, box_char) = if is_completed {
                            ("✓", Style::default().fg(Color::Green), "│")
                        } else if is_current {
                            (
                                "→",
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                                "┃",
                            )
                        } else {
                            ("○", Style::default().fg(Color::DarkGray), "│")
                        };

                        // Check if this looks like a dangerous command (case-insensitive)
                        let content_lower = code.content.to_lowercase();
                        let is_dangerous = content_lower.contains("rm -rf")
                            || content_lower.contains("drop table")
                            || content_lower.contains("drop database")
                            || content_lower.contains("delete ")
                            || content_lower.contains("--force");

                        let danger_marker = if is_dangerous {
                            Span::styled(
                                " [DANGER]",
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::raw("")
                        };

                        lines.push(Line::from(vec![
                            Span::styled(format!("{} ", marker), step_style),
                            Span::styled(
                                format!("Step {} [{}]:", step_num, code.language),
                                step_style,
                            ),
                            danger_marker,
                        ]));

                        // Code content with syntax-aware styling
                        let code_style = if is_current {
                            Style::default().fg(Color::Green)
                        } else if is_completed {
                            Style::default().fg(Color::Green).add_modifier(Modifier::DIM)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };

                        let prefix_style = if is_current {
                            Style::default().fg(Color::Yellow)
                        } else if is_completed {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };

                        for line in code.content.lines() {
                            // Simple syntax highlighting
                            let highlighted = self.highlight_code_line(line, &code.language, &code_style);

                            let mut spans = vec![Span::styled(format!("{} ", box_char), prefix_style)];
                            spans.extend(highlighted);

                            lines.push(Line::from(spans));
                        }

                        lines.push(Line::from(""));
                    }
                }
            }
        }

        lines
    }

    fn highlight_code_line(&self, line: &str, language: &str, base_style: &Style) -> Vec<Span> {
        // Simple syntax highlighting for shell commands; fallback to raw text for others.
        if language == "bash" || language == "sh" {
            let mut spans = Vec::new();
            let trimmed = line.trim_start();
            let indent_len = line.len().saturating_sub(trimmed.len());
            let indent = &line[..indent_len];

            if !indent.is_empty() {
                spans.push(Span::raw(indent.to_string()));
            }

            if trimmed.is_empty() {
                return spans;
            }

            // Comment
            if trimmed.starts_with('#') {
                spans.push(Span::styled(
                    trimmed.to_string(),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                ));
                return spans;
            }

            let lower = trimmed.to_lowercase();

            // Dangerous keywords highlight whole trimmed line red
            if lower.contains("rm ") || lower.contains("rm -rf") || lower.contains("delete ")
                || lower.contains("drop ") || lower.contains("--force")
            {
                spans.push(Span::styled(trimmed.to_string(), Style::default().fg(Color::Red)));
                return spans;
            }

            // Split on '$' and highlight environment variables
            if trimmed.contains('$') {
                // We'll keep leading piece then each $var piece
                let mut remaining = trimmed;
                while let Some(dollar_idx) = remaining.find('$') {
                    // push before $
                    if dollar_idx > 0 {
                        spans.push(Span::styled(
                            remaining[..dollar_idx].to_string(),
                            *base_style,
                        ));
                    }

                    // process var after $
                    let after = &remaining[dollar_idx + 1..];
                    let var_end = after
                        .find(|c: char| !c.is_alphanumeric() && c != '_')
                        .unwrap_or(after.len());

                    let var = &after[..var_end];
                    spans.push(Span::styled(
                        format!("${}", var),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ));

                    // advance remaining
                    remaining = &after[var_end..];
                }

                if !remaining.is_empty() {
                    spans.push(Span::styled(remaining.to_string(), *base_style));
                }

                return spans;
            }

            // Pipes and redirects are just returned with base style (could be extended)
            spans.push(Span::styled(trimmed.to_string(), *base_style));
            spans
        } else {
            // For other languages, just use base style
            vec![Span::styled(line.to_string(), *base_style)]
        }
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
