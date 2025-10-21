//! Integration tests for the sysadmin project

use std::fs;

use usr_bin_sysadmin::parser::SysadminParser;
use usr_bin_sysadmin::model::Document;

/// Integration test for parsing and executing a complete sysadmin file
#[test]
fn test_integration_parse_basic_example() {
    // Read the basic example file
    let content = fs::read_to_string("examples/basic.sysadmin")
        .expect("Failed to read basic.sysadmin example file");
    
    // Parse the document
    let doc = SysadminParser::parse(&content).unwrap();
    
    // Validate basic structure
    assert_eq!(doc.sections.len(), 7); // Should have multiple sections
    
    // Validate that it has code blocks
    let code_blocks = doc.code_blocks();
    assert!(!code_blocks.is_empty());
    
    // Validate specific code blocks exist
    assert_eq!(code_blocks.len(), 7); // Should have 7 executable code blocks
    
    // Check first code block (echo "Hello, sysadmin!")
    assert_eq!(code_blocks[0].language, "bash");
    assert_eq!(code_blocks[0].content.trim(), "echo \"Hello, sysadmin!\"");
    
    // Check second code block (date)
    assert_eq!(code_blocks[1].language, "bash");
    assert_eq!(code_blocks[1].content.trim(), "date");
    
    // Check third code block (ls -la)
    assert_eq!(code_blocks[2].language, "bash");
    assert_eq!(code_blocks[2].content.trim(), "ls -la");
    
    // Check fourth code block (df -h)
    assert_eq!(code_blocks[3].language, "bash");
    assert_eq!(code_blocks[3].content.trim(), "df -h");
    
    // Check fifth code block (Python)
    assert_eq!(code_blocks[4].language, "python");
    assert_eq!(code_blocks[4].content.trim(), "print(\"Hello from Python!\")");
}

/// Integration test for parsing and executing a database migration example
#[test]
fn test_integration_parse_database_migration() {
    // Read the database migration example file
    let content = fs::read_to_string("examples/database-migration.sysadmin")
        .expect("Failed to read database-migration.sysadmin example file");
    
    // Parse the document
    let doc = SysadminParser::parse(&content).unwrap();
    
    // Validate basic structure
    assert_eq!(doc.sections.len(), 11); // Should have multiple sections
    
    // Validate that it has code blocks
    let code_blocks = doc.code_blocks();
    assert!(!code_blocks.is_empty());
    
    // Should have 11 executable code blocks
    assert_eq!(code_blocks.len(), 11);
    
    // Check first code block (ssh backuphost)
    assert_eq!(code_blocks[0].language, "bash");
    assert_eq!(code_blocks[0].content.trim(), "ssh backuphost 'ls -lh /var/backups/db/latest.sql.gz'");
    
    // Check second code block (kubectl scale)
    assert_eq!(code_blocks[1].language, "bash");
    assert_eq!(code_blocks[1].content.trim(), "kubectl scale deployment/api-server --replicas=0");
    
    // Check third code block (psql migration)
    assert_eq!(code_blocks[2].language, "bash");
    assert_eq!(code_blocks[2].content.trim(), "psql -h proddb.internal -U dbadmin -f migration-v4.2.sql");
    
    // Check fourth code block (psql verify)
    assert_eq!(code_blocks[3].language, "bash");
    assert_eq!(code_blocks[3].content.trim(), "psql -h proddb.internal -U dbadmin -c \"SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;\"");
    
    // Check fifth code block (kubectl restart)
    assert_eq!(code_blocks[4].language, "bash");
    assert_eq!(code_blocks[4].content.trim(), "kubectl scale deployment/api-server --replicas=5");
}

/// Integration test for end-to-end workflow with real files
#[test]
fn test_integration_end_to_end_workflow() {
    // Read the basic example file
    let content = fs::read_to_string("examples/basic.sysadmin")
        .expect("Failed to read basic.sysadmin example file");
    
    // Parse the document
    let doc = SysadminParser::parse(&content).unwrap();
    
    // Validate the document structure
    assert_eq!(doc.sections.len(), 7);
    
    // Validate that it has proper headers
    assert_eq!(doc.sections[0].header, Some("Basic Example".to_string()));
    
    // Validate that it has code blocks
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 6);
    
    // Validate that the first code block is bash
    assert_eq!(code_blocks[0].language, "bash");
    assert_eq!(code_blocks[0].content.trim(), "echo \"Hello, sysadmin!\"");
    
    // Validate that the last code block is sh (not python)
    assert_eq!(code_blocks[5].language, "sh");
    assert_eq!(code_blocks[5].content.trim(), "echo \"This uses /bin/sh\"");
    
    // Validate that it has proper step count
    assert_eq!(doc.step_count(), 6);
}

/// Integration test for parsing edge cases in real files
#[test]
fn test_integration_edge_cases() {
    // Create a minimal sysadmin file content to test edge cases
    let content = r#"#!/usr/bin/sysadmin

# Test Document with Edge Cases

This is a test document.

```bash
echo "test"
```

More text here.
"#;

    // Parse the document
    let doc = SysadminParser::parse(content).unwrap();
    
    // Should have one section
    assert_eq!(doc.sections.len(), 1);
    
    // Should have one code block (the one with language)
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
    
    // Validate code block content
    assert_eq!(code_blocks[0].language, "bash");
    assert_eq!(code_blocks[0].content.trim(), "echo \"test\"");
}

/// Integration test for parsing files with various markdown elements
#[test]
fn test_integration_complex_markdown() {
    // Read the database migration example file (more complex markdown)
    let content = fs::read_to_string("examples/database-migration.sysadmin")
        .expect("Failed to read database-migration.sysadmin example file");
    
    // Parse the document
    let doc = SysadminParser::parse(&content).unwrap();
    
    // Validate that it has proper section structure
    assert_eq!(doc.sections.len(), 11);
    
    // Validate that it has proper headers
    assert_eq!(doc.sections[0].header, Some("Database Migration - Q4 2025".to_string()));
    assert_eq!(doc.sections[1].header, Some("Prerequisites".to_string()));
    assert_eq!(doc.sections[2].header, Some("Steps".to_string()));
    
    // Validate that it has proper code blocks in the right sections
    let code_blocks = doc.code_blocks();
    
    // Should have 11 code blocks (from the steps section)
    assert_eq!(code_blocks.len(), 11);
    
    // Validate that all code blocks have bash language
    for block in code_blocks.iter() {
        assert_eq!(block.language, "bash");
    }
    
    // Validate that the first code block contains ssh command
    assert!(code_blocks[0].content.contains("ssh backuphost"));
    
    // Validate that the second code block contains kubectl command
    assert!(code_blocks[1].content.contains("kubectl scale"));
    
    // Validate that the third code block contains psql command
    assert!(code_blocks[2].content.contains("psql -h proddb.internal"));
    
    // Validate that the fourth code block contains psql command for verification
    assert!(code_blocks[3].content.contains("psql -h proddb.internal"));
    
    // Validate that the fifth code block contains kubectl command for restart
    assert!(code_blocks[4].content.contains("kubectl scale"));
}

#[test]
fn test_integration_with_empty_file() {
    let content = "";
    
    // Parse the document
    let doc = SysadminParser::parse(content).unwrap();
    
    // Should have no sections
    assert_eq!(doc.sections.len(), 0);
    
    // Should have no code blocks
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 0);
}

#[test]
fn test_integration_with_only_text() {
    let content = r#"#!/usr/bin/sysadmin

# Text Only Document

This document contains only text.

More text here.
"#;

    // Parse the document
    let doc = SysadminParser::parse(content).unwrap();
    
    // Should have one section
    assert_eq!(doc.sections.len(), 1);
    
    // Should have no code blocks (this is a text-only document)
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 0);
}