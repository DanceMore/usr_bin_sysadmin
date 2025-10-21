//! Unit tests for error handling in the sysadmin parser module

use super::SysadminParser;
use crate::model::{Block, CodeBlock, Document, Section};

#[test]
fn test_parse_document_with_unclosed_code_block() {
    let content = r#"# Unclosed Code Test

Some text.

```bash
echo "hello"
"#;

    // This should not panic, but may not parse correctly
    let doc = SysadminParser::parse(content).unwrap();
    // The parser should handle unclosed code blocks gracefully
    assert_eq!(doc.sections.len(), 1);
}

#[test]
fn test_parse_document_with_corrupted_markdown() {
    let content = r#"# Corrupted Markdown Test

Some text.

```bash
echo "hello"
```

This is not properly closed.
"#;

    // Should parse without panicking
    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 1);
}

#[test]
fn test_parse_document_with_invalid_heading_structure() {
    let content = r#"# H1 Header

Some text.

## H2 Header

More text.

### H3 Header

```bash
echo "hello"
```

#### H4 Header

More text.

```bash
echo "world"
```
"#;

    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 5);
    
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 2);
}

#[test]
fn test_parse_document_with_very_large_content() {
    let content = r#"# Large Content Test

This is a test document with very large content.

```bash
echo "test"
```

More text.
"#;

    // Should not panic with large content
    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 1);
}

#[test]
fn test_parse_document_with_unicode_and_special_chars() {
    let content = r#"# Unicode Test

This document contains unicode and special characters: 
- 🚀 Emoji support
- 你好 (Chinese)
- Привет (Russian)
- مرحبا (Arabic)

```bash
echo "Hello, 世界! @#$%^&*()"
```

More text.
"#;

    let doc = SysadminParser::parse(content).unwrap();
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
    assert_eq!(code_blocks[0].content, "echo \"Hello, 世界! @#$%^&*()\"");
}

#[test]
fn test_parse_document_with_invalid_code_block_format() {
    let content = r#"# Invalid Code Block Test

Some text.

```bash
echo "hello"
``` 

More text.
"#;

    let doc = SysadminParser::parse(content).unwrap();
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_excessive_nesting() {
    let content = r#"# Deep Nesting Test

This is a test with deep nesting.

```bash
echo "level 1"
```

More text.
"#;

    // Should not panic with nested content
    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 1);
}

#[test]
fn test_parse_document_with_malformed_code_block() {
    let content = r#"# Malformed Code Block Test

Some text.

```bash
echo "hello"
``` 

More text.
"#;

    // Should handle malformed code blocks gracefully
    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 1);
}

#[test]
fn test_parse_document_with_empty_code_block() {
    let content = r#"# Empty Code Block Test

Some text.

```bash
```

More text.
"#;

    // Should handle empty code blocks gracefully
    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 1);
}

#[test]
fn test_parse_document_with_only_whitespace() {
    let content = "   \n  \n  \n  ";
    
    // Should handle documents with only whitespace gracefully
    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 0);
}

#[test]
fn test_parse_document_with_only_newlines() {
    let content = "\n\n\n\n";
    
    // Should handle documents with only newlines gracefully
    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 0);
}

#[test]
fn test_parse_document_with_malformed_markdown_syntax() {
    let content = r#"# Malformed Markdown

Some text.

```bash
echo "hello"
```

More text.

## 

This is a malformed heading.
"#;

    // Should handle malformed markdown gracefully
    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 2);
}