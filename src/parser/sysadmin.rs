use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

use crate::model::{Block, CodeBlock, Document, Section};

pub struct SysadminParser;

impl SysadminParser {
    /// Parse a .sysadmin file into a Document
    pub fn parse(content: &str) -> Result<Document> {
        let mut document = Document::new();
        let mut current_section = Section::new();

        let mut text_buffer = String::new();
        let mut in_code_block = false;
        let mut code_buffer = String::new();
        let mut code_language = String::new();
        let mut line_number = 1;
        let mut in_heading = false;
        let mut heading_level = 1;

        let parser = Parser::new(content);

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    // Flush any accumulated text
                    if !text_buffer.trim().is_empty() {
                        current_section
                            .blocks
                            .push(Block::Text(text_buffer.clone()));
                        text_buffer.clear();
                    }
                    in_heading = true;
                    heading_level = level as u32;
                }

                Event::End(TagEnd::Heading(_)) => {
                    in_heading = false;

                    // Save current section if it has content
                    if !current_section.blocks.is_empty() || current_section.header.is_some() {
                        document.sections.push(current_section);
                    }

                    // Start new section with this header
                    current_section = Section::with_header(text_buffer.trim().to_string(), heading_level);
                    text_buffer.clear();
                }

                Event::Start(Tag::CodeBlock(kind)) => {
                    // Flush any text before code block
                    if !text_buffer.trim().is_empty() {
                        current_section
                            .blocks
                            .push(Block::Text(text_buffer.clone()));
                        text_buffer.clear();
                    }

                    in_code_block = true;
                    code_language = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        CodeBlockKind::Indented => String::new(),
                    };
                }

                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;

                    // Only add code blocks with a language identifier
                    if !code_language.is_empty() {
                        current_section.blocks.push(Block::Code(CodeBlock {
                            language: code_language.clone(),
                            content: code_buffer.trim_end().to_string(),
                            line_number,
                        }));
                    } else if !code_buffer.trim().is_empty() {
                        // Code blocks without language go into text
                        text_buffer.push_str("```\n");
                        text_buffer.push_str(&code_buffer);
                        text_buffer.push_str("```\n");
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
                    if !in_code_block {
                        text_buffer.push('`');
                        text_buffer.push_str(&text);
                        text_buffer.push('`');
                    }
                }

                Event::SoftBreak => {
                    if in_code_block {
                        code_buffer.push('\n');
                        line_number += 1;
                    } else if !in_heading {
                        text_buffer.push(' ');
                    }
                }

                Event::HardBreak => {
                    if in_code_block {
                        code_buffer.push('\n');
                    } else {
                        text_buffer.push('\n');
                    }
                    line_number += 1;
                }

                Event::Start(Tag::Paragraph) => {
                    if !text_buffer.is_empty() && !text_buffer.ends_with('\n') {
                        text_buffer.push('\n');
                    }
                }

                Event::End(TagEnd::Paragraph) => {
                    text_buffer.push('\n');
                }

                Event::Start(Tag::List(_)) | Event::End(TagEnd::List(_)) => {
                    text_buffer.push('\n');
                }

                Event::Start(Tag::Item) => {
                    text_buffer.push_str("• ");
                }

                Event::End(TagEnd::Item) => {
                    text_buffer.push('\n');
                }

                Event::Start(Tag::Emphasis) => text_buffer.push('*'),
                Event::End(TagEnd::Emphasis) => text_buffer.push('*'),

                Event::Start(Tag::Strong) => text_buffer.push_str("**"),
                Event::End(TagEnd::Strong) => text_buffer.push_str("**"),

                _ => {
                    // Handle other events as needed
                }
            }
        }

        // Flush remaining content
        if !text_buffer.trim().is_empty() {
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
        assert_eq!(code_blocks[0].language, "bash");
    }

    #[test]
    fn test_parse_multiple_sections() {
        let content = r#"# Section One

Text in section one.

```bash
ls -la
```

## Section Two

Text in section two.

```python
print("hello")
```
"#;

        let doc = SysadminParser::parse(content).unwrap();
        assert_eq!(doc.sections.len(), 2);
        assert_eq!(doc.sections[0].header, Some("Section One".to_string()));
        assert_eq!(doc.sections[1].header, Some("Section Two".to_string()));
        assert_eq!(doc.code_blocks().len(), 2);
    }

    #[test]
    fn test_parse_no_language_code_block() {
        let content = r#"# Test

Some text.

```
not executable
```

More text.
"#;

        let doc = SysadminParser::parse(content).unwrap();
        let code_blocks = doc.code_blocks();
        // Code blocks without language are not executable
        assert_eq!(code_blocks.len(), 0);
    }

    #[test]
    fn test_empty_document() {
        let content = "";
        let doc = SysadminParser::parse(content).unwrap();
        assert_eq!(doc.sections.len(), 0);
    }
}
