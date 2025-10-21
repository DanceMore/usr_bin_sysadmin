# Sysadmin - Rust Project Design

## Project Structure

```
sysadmin/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── parser/
│   │   ├── mod.rs           # Parser module exports
│   │   ├── markdown.rs      # Markdown parsing (using pulldown-cmark)
│   │   └── sysadmin.rs      # .sysadmin file parser
│   ├── model/
│   │   ├── mod.rs           # Model exports
│   │   ├── document.rs      # Document and Section types
│   │   └── block.rs         # Block types (Text, Code)
│   ├── executor/
│   │   ├── mod.rs           # Executor module
│   │   └── interactive.rs   # Interactive execution mode
│   ├── ui/
│   │   ├── mod.rs           # UI module
│   │   ├── terminal.rs      # Terminal setup and teardown
│   │   └── renderer.rs      # Rendering content to terminal
│   └── cli.rs               # CLI argument parsing
├── examples/
│   ├── basic.sysadmin
│   └── database-migration.sysadmin
└── tests/
    ├── parser_tests.rs
    └── integration_tests.rs
```

## Core Dependencies (Cargo.toml)

```toml
[package]
name = "sysadmin"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A shell for sysadmins - executable operational documentation"
license = "MIT OR Apache-2.0"

[[bin]]
name = "sysadmin"
path = "src/main.rs"

[dependencies]
# Markdown parsing
pulldown-cmark = "0.11"

# CLI argument parsing
clap = { version = "4.5", features = ["derive"] }

# Terminal UI
crossterm = "0.28"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Serialization (for future features)
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
# Testing
pretty_assertions = "1.4"
```

## Core Types

### Document Model (`src/model/`)

```rust
// src/model/document.rs

/// A parsed .sysadmin document
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// The sections of the document
    pub sections: Vec<Section>,
}

/// A section of a document (could be text, code, or mixed)
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// Optional header for this section
    pub header: Option<String>,
    /// The blocks in this section
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new() -> Self {
        Document {
            sections: Vec::new(),
        }
    }

    /// Get all executable code blocks in order
    pub fn code_blocks(&self) -> Vec<&CodeBlock> {
        self.sections
            .iter()
            .flat_map(|s| &s.blocks)
            .filter_map(|b| match b {
                Block::Code(code) => Some(code),
                _ => None,
            })
            .collect()
    }
}
```

```rust
// src/model/block.rs

/// A block in the document
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Documentation/text content (markdown)
    Text(String),
    /// Executable code block
    Code(CodeBlock),
}

/// An executable code block
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    /// Language/interpreter (bash, sh, python, etc.)
    pub language: String,
    /// The actual code content
    pub content: String,
    /// Line number where this block starts in the source file
    pub line_number: usize,
}

impl CodeBlock {
    /// Get the interpreter command for this language
    pub fn interpreter(&self) -> &str {
        match self.language.as_str() {
            "bash" => "bash",
            "sh" => "sh",
            "python" | "python3" => "python3",
            "ruby" => "ruby",
            "perl" => "perl",
            _ => "bash", // default fallback
        }
    }

    /// Check if this is a shell-like language
    pub fn is_shell(&self) -> bool {
        matches!(self.language.as_str(), "bash" | "sh" | "zsh" | "fish")
    }
}
```

## Parser Design (`src/parser/`)

### Strategy

We'll use **pulldown-cmark** for the heavy lifting of markdown parsing, then extract:
- Text content (everything that's not a code block)
- Code blocks with their language identifiers

```rust
// src/parser/sysadmin.rs

use anyhow::{Context, Result};
use pulldown_cmark::{Event, Parser, Tag, CodeBlockKind};
use crate::model::{Document, Section, Block, CodeBlock};

pub struct SysadminParser;

impl SysadminParser {
    /// Parse a .sysadmin file into a Document
    pub fn parse(content: &str) -> Result<Document> {
        let mut document = Document::new();
        let mut current_section = Section {
            header: None,
            blocks: Vec::new(),
        };
        
        let mut text_buffer = String::new();
        let mut in_code_block = false;
        let mut code_buffer = String::new();
        let mut code_language = String::new();
        let mut line_number = 1;
        
        let parser = Parser::new(content);
        
        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    // Flush any accumulated text
                    if !text_buffer.is_empty() {
                        current_section.blocks.push(
                            Block::Text(text_buffer.clone())
                        );
                        text_buffer.clear();
                    }
                }
                
                Event::End(Tag::Heading { .. }) => {
                    // Heading is now in text_buffer
                    // Start a new section with this header
                    if !current_section.blocks.is_empty() || current_section.header.is_some() {
                        document.sections.push(current_section);
                    }
                    current_section = Section {
                        header: Some(text_buffer.trim().to_string()),
                        blocks: Vec::new(),
                    };
                    text_buffer.clear();
                }
                
                Event::Start(Tag::CodeBlock(kind)) => {
                    // Flush any text before code block
                    if !text_buffer.is_empty() {
                        current_section.blocks.push(
                            Block::Text(text_buffer.clone())
                        );
                        text_buffer.clear();
                    }
                    
                    in_code_block = true;
                    code_language = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        CodeBlockKind::Indented => String::from("bash"),
                    };
                }
                
                Event::End(Tag::CodeBlock(_)) => {
                    in_code_block = false;
                    
                    // Only add code blocks with a language identifier
                    if !code_language.is_empty() {
                        current_section.blocks.push(Block::Code(CodeBlock {
                            language: code_language.clone(),
                            content: code_buffer.trim().to_string(),
                            line_number,
                        }));
                    }
                    
                    code_buffer.clear();
                    code_language.clear();
                }
                
                Event::Text(text) => {
                    if in_code_block {
                        code_buffer.push_str(&text);
                    } else {
                        text_buffer.push_str(&text);
                    }
                }
                
                Event::Code(text) => {
                    // Inline code
                    text_buffer.push('`');
                    text_buffer.push_str(&text);
                    text_buffer.push('`');
                }
                
                Event::SoftBreak | Event::HardBreak => {
                    if in_code_block {
                        code_buffer.push('\n');
                    } else {
                        text_buffer.push('\n');
                    }
                    line_number += 1;
                }
                
                _ => {
                    // Handle other events as needed
                }
            }
        }
        
        // Flush remaining content
        if !text_buffer.is_empty() {
            current_section.blocks.push(Block::Text(text_buffer));
        }
        
        if !current_section.blocks.is_empty() || current_section.header.is_some() {
            document.sections.push(current_section);
        }
        
        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_document() {
        let content = r#"# Test Document

This is some text.

```bash
echo "hello world"
```

More text here.
"#;
        
        let doc = SysadminParser::parse(content).unwrap();
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].header, Some("Test Document".to_string()));
        
        let code_blocks = doc.code_blocks();
        assert_eq!(code_blocks.len(), 1);
        assert_eq!(code_blocks[0].content, "echo \"hello world\"");
    }
}
```

## Executor Design (`src/executor/`)

```rust
// src/executor/interactive.rs

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crate::model::Document;
use crate::ui::renderer::Renderer;

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
        let code_blocks = doc.code_blocks();
        let total_steps = code_blocks.len();
        
        for (section_idx, section) in doc.sections.iter().enumerate() {
            // Render section header if present
            if let Some(header) = &section.header {
                self.renderer.render_header(header)?;
            }
            
            // Render each block in the section
            for block in &section.blocks {
                match block {
                    crate::model::Block::Text(text) => {
                        self.renderer.render_text(text)?;
                    }
                    crate::model::Block::Code(code) => {
                        self.renderer.render_code(code)?;
                        
                        // Wait for user confirmation (Ctrl-D or Enter)
                        self.wait_for_continue()?;
                    }
                }
            }
        }
        
        self.renderer.render_completion()?;
        Ok(())
    }
    
    fn wait_for_continue(&self) -> Result<()> {
        println!("\n[Press Enter to continue, or Ctrl-C to exit]");
        
        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Enter | KeyCode::Char('d') if key_event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('c') if key_event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }
    }
}
```

## CLI Design (`src/cli.rs`)

```rust
// src/cli.rs

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
```

## Main Entry Point (`src/main.rs`)

```rust
// src/main.rs

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

mod cli;
mod model;
mod parser;
mod executor;
mod ui;

use cli::{Cli, Commands};
use parser::sysadmin::SysadminParser;
use executor::interactive::InteractiveExecutor;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let file_path = match &cli.command {
        Some(Commands::Run { file }) => file,
        Some(Commands::DryRun { file }) => file,
        Some(Commands::View { file }) => file,
        None => {
            if let Some(file) = &cli.file {
                file
            } else {
                eprintln!("Error: No file specified");
                std::process::exit(1);
            }
        }
    };
    
    // Read the file
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
    
    // Parse the document
    let document = SysadminParser::parse(&content)
        .context("Failed to parse document")?;
    
    // Execute based on command
    match &cli.command {
        Some(Commands::DryRun { .. }) | None if cli.command.is_none() => {
            // Default: interactive execution
            let mut executor = InteractiveExecutor::new();
            executor.execute(&document)?;
        }
        Some(Commands::Run { .. }) => {
            let mut executor = InteractiveExecutor::new();
            executor.execute(&document)?;
        }
        Some(Commands::DryRun { .. }) => {
            // Just print all steps
            for code in document.code_blocks() {
                println!("Step [{}]:", code.language);
                println!("{}", code.content);
                println!();
            }
        }
        Some(Commands::View { .. }) => {
            // Just render as formatted text
            println!("{}", content);
        }
    }
    
    Ok(())
}
```

## Library Export (`src/lib.rs`)

```rust
// src/lib.rs

pub mod model;
pub mod parser;
pub mod executor;
pub mod ui;

// Re-export commonly used types
pub use model::{Document, Section, Block, CodeBlock};
pub use parser::sysadmin::SysadminParser;
```

## Key Design Decisions

### 1. **pulldown-cmark for Parsing**
- Battle-tested, CommonMark compliant
- Event-based streaming parser (memory efficient)
- Handles all markdown edge cases

### 2. **Simple Type Hierarchy**
- `Document` → `Section` → `Block` (Text | Code)
- Easy to serialize/deserialize later
- Clear separation between documentation and executable content

### 3. **crossterm for Terminal UI**
- Cross-platform (Windows, macOS, Linux)
- Low-level control for future TUI features
- Non-blocking input handling

### 4. **clap for CLI**
- Derive-based API (less boilerplate)
- Automatic help generation
- Subcommand support for future expansion

### 5. **Error Handling**
- `anyhow` for application errors (main.rs, CLI)
- `thiserror` for library errors (parser, executor)
- Context-rich error messages

## Next Steps

1. Implement basic parser using pulldown-cmark
2. Implement simple renderer (no fancy TUI yet, just colored output)
3. Wire up the interactive executor
4. Test with example .sysadmin files
5. Add syntax highlighting for code blocks
6. Enhance TUI with split-screen view
