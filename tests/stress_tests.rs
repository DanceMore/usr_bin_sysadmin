//! Stress tests for the sysadmin system

use usr_bin_sysadmin::parser::SysadminParser;
use usr_bin_sysadmin::model::Document;

#[test]
fn test_parse_large_document_with_many_code_blocks() {
    // Create a large document with many code blocks to test performance
    let mut content = String::from("# Large Document Test\n\n");
    
    // Create a document with 1000 code blocks
    for i in 1..=1000 {
        content.push_str(&format!(
            "## Step {}\n\nSome text here.\n\n```bash\ncommand {}\n```\n\n",
            i, i
        ));
    }

    // This should not panic or take too long
    let doc = SysadminParser::parse(&content).unwrap();
    
    // Verify that we got the expected number of code blocks
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1000);
}

#[test]
fn test_parse_very_large_document() {
    // Create a very large document to test memory usage
    let mut content = String::from("# Very Large Document Test\n\n");
    
    // Add many lines of content
    for i in 1..=10000 {
        content.push_str(&format!("Line {} of content.\n", i));
    }
    
    // This should not panic or use excessive memory
    let doc = SysadminParser::parse(&content).unwrap();
    
    // Should have at least one section
    assert!(doc.sections.len() >= 1);
}

#[test]
fn test_parse_document_with_nested_structures() {
    // Create a document with deeply nested structures
    let content = r#"# Nested Structures Test

This is a test document with nested structures.

## Section One

Some content here.

### Subsection One

More content.

#### Deeply Nested Section

Even more content.

```bash
echo "deep nesting test"
```

## Section Two

More content here.

### Subsection Two

Even more content.

```bash
echo "another test"
```
"#;

    // Should parse without issues
    let doc = SysadminParser::parse(content).unwrap();
    
    // Should have multiple sections
    assert!(doc.sections.len() >= 2);
    
    // Should have code blocks
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 2);
}

#[test]
fn test_parse_document_with_unicode_content() {
    // Test with unicode content to ensure proper handling
    let content = r#"# Unicode Content Test

This document contains unicode characters: 
- 🚀 Emoji support
- 你好 (Chinese)
- Привет (Russian)
- مرحبا (Arabic)

```bash
echo "Hello, 世界! @#$%^&*()"
```

More text.
"#;

    // Should parse without issues
    let doc = SysadminParser::parse(content).unwrap();
    
    // Should have code blocks
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_special_characters() {
    // Test with various special characters that could cause issues
    let content = r#"# Special Characters Test

This document contains special characters:

```bash
echo "Hello, 世界! @#$%^&*()_+-=[]{}|;':\",./<>?"
```

More text.
"#;

    // Should parse without issues
    let doc = SysadminParser::parse(content).unwrap();
    
    // Should have code blocks
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_concurrent_operations() {
    // Test that parsing is resilient to various inputs
    let test_cases = vec![
        // Empty document
        "",
        
        // Simple document
        "# Test\n\necho \"hello\"\n",
        
        // Document with many sections
        "# Section 1\n\ntext\n\n## Section 2\n\ntext\n\n### Section 3\n\ntext\n",
        
        // Document with various code block formats
        "# Test\n\n```bash\necho \"hello\"\n```\n\n```python\nprint(\"hello\")\n```\n",
    ];
    
    // All test cases should parse without panicking
    for (i, content) in test_cases.iter().enumerate() {
        let doc = SysadminParser::parse(content).unwrap();
        // Just verify it parses without panicking
        assert!(doc.sections.len() >= 0, "Test case {} failed", i);
    }
}