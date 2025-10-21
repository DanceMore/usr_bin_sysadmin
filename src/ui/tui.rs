use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use emojis;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::model::{Block as DocBlock, Document};

/// Centralized emoji icon manager
struct Icons {
    done: &'static str,
    current: &'static str,
    pending: &'static str,
    warning: &'static str,
    danger: &'static str,
    info: &'static str,
}

fn icons() -> Icons {
    Icons {
        done: emojis::get("check_mark_button").map(|e| e.as_str()).unwrap_or("✔"),
        current: emojis::get("arrow_right").map(|e| e.as_str()).unwrap_or("➡"),
        pending: emojis::get("radio_button").map(|e| e.as_str()).unwrap_or("○"),
        warning: emojis::get("warning").map(|e| e.as_str()).unwrap_or("⚠️"),
        danger: emojis::get("fire").map(|e| e.as_str()).unwrap_or("🔥"),
        info: emojis::get("information").map(|e| e.as_str()).unwrap_or("ℹ️"),
    }
}

pub struct TuiApp {
    document: Document,
    current_step: usize,
    scroll_offset: usize,
    transient_message: Option<(String, Instant)>,
}

impl TuiApp {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            current_step: 0,
            scroll_offset: 0,
            transient_message: None,
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
                    .constraints([Constraint::Min(10), Constraint::Length(3)])
                    .split(f.area());
            
                let runbook_content = self.render_runbook_content();
                let runbook = Paragraph::new(runbook_content)
                    .block(
                        Block::default()
                            .title("📘 Runbook")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan)),
                    )
                    .wrap(Wrap { trim: true })
                    .scroll((self.scroll_offset as u16, 0));
            
                f.render_widget(runbook, chunks[0]);
            
                // Status bar
                let total_steps = self.document.step_count();
                let status_text = if total_steps == 0 {
                    " No executable steps | q: Quit ".to_string()
                } else if self.current_step >= total_steps {
                    " ✅ Final step complete! Press 'q' to quit or 'p' to review. ".to_string()
                } else {
                    format!(
                        " Step {}/{} | ↑↓: Scroll | n: Next | p: Previous | s: Shell | q: Quit ",
                        self.current_step.min(total_steps),
                        total_steps
                    )
                };
            
                let status = Paragraph::new(status_text)
                    .alignment(Alignment::Center)
                    .style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::White)),
                    );
            
                f.render_widget(status, chunks[1]);
            
                // Render transient message as a floating single-line overlay (doesn't change Layout)
                const MSG_TTL: Duration = Duration::from_secs(4);
                if let Some((ref msg, when)) = self.transient_message {
                    if Instant::now().saturating_duration_since(when) < MSG_TTL {
                        // Place the overlay directly above the status bar, full width
                        let overlay_area = ratatui::layout::Rect::new(
                            chunks[1].x,
                            chunks[1].y.saturating_sub(1),
                            chunks[1].width,
                            1,
                        );
            
                        let overlay = Paragraph::new(msg.as_str())
                            .alignment(Alignment::Left)
                            .style(
                                Style::default()
                                    .bg(Color::Black)
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .block(Block::default()); // no borders so it doesn't change layout
            
                        f.render_widget(overlay, overlay_area);
                    } else {
                        // message expired
                        // clear it so it stops checking every frame
                        // can't mutate self inside closure because closure borrows &self immutably,
                        // so we leave clearing to the outer loop after draw (see below).
                    }
                }
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
        let i = icons();

        for (section_idx, section) in self.document.sections.iter().enumerate() {
            // Render header
            if let Some(header) = &section.header {
                let level = section.header_level.unwrap_or(1);
                let header_style = match level {
                    1 => Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                    2 => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD),
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
                    format!("📘 {} {}", "#".repeat(level as usize), header),
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
                                let upper = line.to_uppercase();
                                let styled_line = if upper.contains("WARNING") {
                                    Line::from(vec![
                                        Span::styled(
                                            format!("{} ", i.warning),
                                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                                        ),
                                        Span::styled(line, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                                    ])
                                } else if upper.contains("DANGER") || upper.contains("CRITICAL") {
                                    Line::from(vec![
                                        Span::styled(
                                            format!("{} ", i.danger),
                                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                                        ),
                                        Span::styled(line, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                                    ])
                                } else if upper.contains("INFO") || upper.contains("NOTE") {
                                    Line::from(vec![
                                        Span::styled(
                                            format!("{} ", i.info),
                                            Style::default().fg(Color::Blue),
                                        ),
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
                            (i.done, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD), "│")
                        } else if is_current {
                            (i.current, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD), "┃")
                        } else {
                            (i.pending, Style::default().fg(Color::DarkGray), "│")
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
                                format!(" {}", i.danger),
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::raw("")
                        };

                        lines.push(Line::from(vec![
                            Span::styled(format!("{} ", marker), step_style),
                            Span::styled(format!("Step {} [{}]:", step_num, code.language), step_style),
                            danger_marker,
                        ]));

                        // Code content with syntax-aware styling
                        let code_style = if is_current {
                            Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)
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
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
                ));
                return spans;
            }

            let lower = trimmed.to_lowercase();
            if lower.contains("rm ") || lower.contains("rm -rf") || lower.contains("delete ")
                || lower.contains("drop ") || lower.contains("--force")
            {
                spans.push(Span::styled(trimmed.to_string(), Style::default().fg(Color::Red)));
                return spans;
            }
            if trimmed.contains('$') {
                let mut remaining = trimmed;
                while let Some(dollar_idx) = remaining.find('$') {
                    if dollar_idx > 0 {
                        spans.push(Span::styled(remaining[..dollar_idx].to_string(), *base_style));
                    }

                    // process var after $
                    let after = &remaining[dollar_idx + 1..];
                    let var_end = after.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after.len());
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
        } else if total_steps > 0 {
            // Already at final step: set transient in-TUI prompt (won't disturb layout)
            let msg = "🎉 You’ve reached the final step! Press 'q' to quit or 'p' to go back.".to_string();
            self.transient_message = Some((msg, Instant::now()));
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
                line_count += 3;
            }

            // Count lines in blocks
            for block in &section.blocks {
                match block {
                    DocBlock::Text(text) => line_count += text.lines().count() + 1,
                    DocBlock::Code(code) => {
                        if code == target_code {
                            // Found it! Set scroll to show this step near the top
                            // Leave some context lines above (5 lines)
                            self.scroll_offset = line_count.saturating_sub(5);
                            return;
                        }
                        line_count += 1 + code.content.lines().count() + 1;
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
