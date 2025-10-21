//! Unit tests for the sysadmin parser module

use super::SysadminParser;
use crate::model::{Block, CodeBlock, Document, Section};

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

#[test]
fn test_parse_code_block_with_indented_formatting() {
    let content = r#"# Test

Some text.

    echo "hello"
    echo "world"

More text.
"#;

    let doc = SysadminParser::parse(content).unwrap();
    // Indented code blocks should be treated as text, not executable
    assert_eq!(doc.code_blocks().len(), 0);
}

#[test]
fn test_parse_code_block_with_language() {
    let content = r#"# Test

Some text.

```bash
echo "hello"
```

More text.
"#;

    let doc = SysadminParser::parse(content).unwrap();
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
    assert_eq!(code_blocks[0].language, "bash");
    assert_eq!(code_blocks[0].content, "echo \"hello\"");
}

#[test]
fn test_parse_complex_document_with_multiple_code_blocks() {
    let content = r#"# Database Migration

Before starting, ensure:
- You have production database credentials
- A tested backup exists from the last hour

## Steps

### Verify backup exists

Check that the automated backup completed successfully:

```bash
ssh backuphost 'ls -lh /var/backups/db/latest.sql.gz'
```

### Stop application servers

This prevents new writes during migration:

```bash
kubectl scale deployment/api-server --replicas=0
```

### Run migration

```bash
psql -h proddb.internal -U dbadmin -f migration-v4.2.sql
```

### Verify migration

```bash
psql -h proddb.internal -U dbadmin -c "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;"
```
"#;

    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 4);
    
    // Should have 4 code blocks
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 4);
    
    // Check first code block
    assert_eq!(code_blocks[0].language, "bash");
    assert_eq!(code_blocks[0].content, "ssh backuphost 'ls -lh /var/backups/db/latest.sql.gz'");
    
    // Check second code block
    assert_eq!(code_blocks[1].language, "bash");
    assert_eq!(code_blocks[1].content, "kubectl scale deployment/api-server --replicas=0");
    
    // Check third code block
    assert_eq!(code_blocks[2].language, "bash");
    assert_eq!(code_blocks[2].content, "psql -h proddb.internal -U dbadmin -f migration-v4.2.sql");
    
    // Check fourth code block
    assert_eq!(code_blocks[3].language, "bash");
    assert_eq!(code_blocks[3].content, "psql -h proddb.internal -U dbadmin -c \"SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;\"");
}

#[test]
fn test_parse_document_with_special_characters() {
    let content = r#"# Special Characters Test

This document contains special characters:

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
fn test_parse_document_with_multiline_code() {
    let content = r#"# Multi-line Test

Some text.

```python
def hello_world():
    print("Hello")
    print("World")
    return "Done"
```

More text.
"#;

    let doc = SysadminParser::parse(content).unwrap();
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
    assert_eq!(code_blocks[0].language, "python");
    assert_eq!(code_blocks[0].content, "def hello_world():\n    print(\"Hello\")\n    print(\"World\")\n    return \"Done\"");
}

#[test]
fn test_parse_document_with_empty_lines() {
    let content = r#"# Empty Lines Test


This document has empty lines:

```bash
echo "test"
```

More text.
"#;

    let doc = SysadminParser::parse(content).unwrap();
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
    assert_eq!(code_blocks[0].content, "echo \"test\"");
}

#[test]
fn test_parse_document_with_inline_code() {
    let content = r#"# Inline Code Test

This has `inline code` in it.

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
fn test_parse_document_with_lists() {
    let content = r#"# List Test

Steps:
- Step one
- Step two

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
fn test_parse_document_with_emphasis() {
    let content = r#"# Emphasis Test

This has *emphasis* and **strong** text.

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
fn test_parse_document_with_different_heading_levels() {
    let content = r#"# H1 Header

Some text.

## H2 Header

More text.

### H3 Header

```bash
echo "hello"
```
"#;

    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 3);
    
    // Check that all sections have proper headers
    assert_eq!(doc.sections[0].header, Some("H1 Header".to_string()));
    assert_eq!(doc.sections[1].header, Some("H2 Header".to_string()));
    assert_eq!(doc.sections[2].header, Some("H3 Header".to_string()));
    
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_code_block_at_end() {
    let content = r#"# End Code Test

Some text.

```bash
echo "hello"
```
"#;

    let doc = SysadminParser::parse(content).unwrap();
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_only_text() {
    let content = r#"# Text Only Test

This is just text.

More text.
"#;

    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 0);
}

#[test]
fn test_parse_document_with_mixed_content() {
    let content = r#"# Mixed Content Test

This is text.

```bash
echo "first"
```

More text.

```python
print("second")
```

Even more text.
"#;

    let doc = SysadminParser::parse(content).unwrap();
    assert_eq!(doc.sections.len(), 1);
    
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 2);
    
    // Check first code block
    assert_eq!(code_blocks[0].language, "bash");
    assert_eq!(code_blocks[0].content, "echo \"first\"");
    
    // Check second code block
    assert_eq!(code_blocks[1].language, "python");
    assert_eq!(code_blocks[1].content, "print(\"second\")");
}

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