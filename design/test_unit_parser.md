# Unit Tests for Parser Components

## Parser Module Tests

### SysadminParser Tests

#### Basic Parsing Functionality
```rust
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
```

#### Edge Case Tests
```rust
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
```

## Model Module Tests

### Document Tests
```rust
#[test]
fn test_document_creation() {
    let doc = Document::new();
    assert_eq!(doc.sections.len(), 0);
    assert_eq!(doc.step_count(), 0);
}

#[test]
fn test_code_blocks_extraction() {
    let mut doc = Document::new();
    let mut section = Section::new();
    
    section.blocks.push(Block::Text("Some text".to_string()));
    section.blocks.push(Block::Code(CodeBlock {
        language: "bash".to_string(),
        content: "echo hello".to_string(),
        line_number: 5,
    }));
    section.blocks.push(Block::Text("More text".to_string()));
    
    doc.sections.push(section);
    
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
    assert_eq!(code_blocks[0].content, "echo hello");
}

#[test]
fn test_step_count() {
    let mut doc = Document::new();
    let mut section = Section::new();
    
    // Add one code block
    section.blocks.push(Block::Code(CodeBlock {
        language: "bash".to_string(),
        content: "echo hello".to_string(),
        line_number: 1,
    }));
    
    doc.sections.push(section);
    
    assert_eq!(doc.step_count(), 1);
}
```

### Section Tests
```rust
#[test]
fn test_section_creation() {
    let section = Section::new();
    assert_eq!(section.header, None);
    assert_eq!(section.blocks.len(), 0);
}

#[test]
fn test_section_with_header() {
    let section = Section::with_header("Test Header".to_string(), 2);
    assert_eq!(section.header, Some("Test Header".to_string()));
    assert_eq!(section.header_level, Some(2));
    assert_eq!(section.blocks.len(), 0);
}
```

## Block Module Tests

### CodeBlock Tests
```rust
#[test]
fn test_interpreter_mapping() {
    let code = CodeBlock {
        language: "bash".to_string(),
        content: "echo hello".to_string(),
        line_number: 1,
    };
    assert_eq!(code.interpreter(), "bash");
}

#[test]
fn test_interpreter_fallback() {
    let code = CodeBlock {
        language: "unknown".to_string(),
        content: "echo hello".to_string(),
        line_number: 1,
    };
    assert_eq!(code.interpreter(), "bash"); // Should fallback to bash
}

#[test]
fn test_is_shell() {
    let bash = CodeBlock {
        language: "bash".to_string(),
        content: "".to_string(),
        line_number: 1,
    };
    assert!(bash.is_shell());

    let python = CodeBlock {
        language: "python".to_string(),
        content: "".to_string(),
        line_number: 1,
    };
    assert!(!python.is_shell());
}

#[test]
fn test_is_shell_variants() {
    let sh = CodeBlock {
        language: "sh".to_string(),
        content: "".to_string(),
        line_number: 1,
    };
    assert!(sh.is_shell());

    let zsh = CodeBlock {
        language: "zsh".to_string(),
        content: "".to_string(),
        line_number: 1,
    };
    assert!(zsh.is_shell());

    let fish = CodeBlock {
        language: "fish".to_string(),
        content: "".to_string(),
        line_number: 1,
    };
    assert!(fish.is_shell());
}
```

## UI Module Tests

### Renderer Tests
```rust
#[test]
fn test_renderer_creation() {
    let renderer = Renderer::new();
    assert_eq!(renderer.current_step, 0);
    assert_eq!(renderer.total_steps, 0);
}

#[test]
fn test_renderer_step_tracking() {
    let mut renderer = Renderer::new();
    renderer.set_total_steps(5);
    
    assert_eq!(renderer.total_steps, 5);
}
```

## Executor Module Tests

### InteractiveExecutor Tests
```rust
#[test]
fn test_executor_creation() {
    let executor = InteractiveExecutor::new();
    // Should create without errors
    assert!(true); // Placeholder for actual test logic
}
```
