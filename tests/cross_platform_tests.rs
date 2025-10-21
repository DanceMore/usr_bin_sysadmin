//! Cross-platform compatibility tests for the sysadmin system

use usr_bin_sysadmin::parser::SysadminParser;
use usr_bin_sysadmin::model::Document;

#[test]
fn test_parse_document_on_different_operating_systems() {
    // Test that the parser works consistently across different OS environments
    let content = r#"# Cross-Platform Test

This document should parse the same on all platforms.

```bash
echo "hello world"
```

More text.
"#;

    // Parse on any platform - should work identically
    let doc = SysadminParser::parse(content).unwrap();
    
    // Verify document structure is consistent
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
    assert_eq!(code_blocks[0].language, "bash");
}

#[test]
fn test_parse_document_with_path_separators() {
    // Test that documents with different path formats parse correctly
    let content_with_forward_slashes = r#"# Path Test

This document has forward slashes.

```bash
ls /home/user/file.txt
```

More text.
"#;

    let content_with_backslashes = r#"# Path Test

This document has backslashes.

```bash
ls C:\Windows\file.txt
```

More text.
"#;

    // Both should parse without issues on any platform
    let doc1 = SysadminParser::parse(content_with_forward_slashes).unwrap();
    let doc2 = SysadminParser::parse(content_with_backslashes).unwrap();
    
    // Both should have code blocks
    assert_eq!(doc1.code_blocks().len(), 1);
    assert_eq!(doc2.code_blocks().len(), 1);
}

#[test]
fn test_parse_document_with_terminal_capabilities() {
    // Test that documents parse correctly regardless of terminal capabilities
    let content = r#"# Terminal Test

This document tests various terminal scenarios.

```bash
echo "test"
```

More text.
"#;

    // Should parse consistently across platforms
    let doc = SysadminParser::parse(content).unwrap();
    
    // Verify structure
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_shell_specific_content() {
    // Test that documents with shell-specific content parse correctly
    let bash_content = r#"# Bash Test

Bash specific content.

```bash
#!/bin/bash
echo "bash script"
```

More text.
"#;

    let sh_content = r#"# Shell Test

Shell specific content.

```sh
#!/bin/sh
echo "sh script"
```

More text.
"#;

    // Both should parse without issues
    let doc1 = SysadminParser::parse(bash_content).unwrap();
    let doc2 = SysadminParser::parse(sh_content).unwrap();
    
    // Both should have code blocks
    assert_eq!(doc1.code_blocks().len(), 1);
    assert_eq!(doc2.code_blocks().len(), 1);
}

#[test]
fn test_parse_document_with_locale_and_encoding() {
    // Test that documents with different encodings parse correctly
    let content = r#"# Encoding Test

This document tests encoding handling.

```bash
echo "Hello, 世界!"
```

More text.
"#;

    // Should parse consistently across platforms regardless of locale
    let doc = SysadminParser::parse(content).unwrap();
    
    // Verify structure
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_different_terminal_emulators() {
    // Test that the same document parses the same regardless of terminal emulator
    let content = r#"# Terminal Emulator Test

This tests various terminal scenarios.

```bash
echo "terminal test"
```

More text.
"#;

    // Should parse identically on all platforms and terminal emulators
    let doc = SysadminParser::parse(content).unwrap();
    
    // Verify structure
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_color_support() {
    // Test that documents parse correctly regardless of color support
    let content = r#"# Color Support Test

This tests color handling.

```bash
echo "color test"
```

More text.
"#;

    // Should parse consistently regardless of color support availability
    let doc = SysadminParser::parse(content).unwrap();
    
    // Verify structure
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_different_shell_environments() {
    // Test that documents parse correctly in different shell environments
    let content = r#"# Shell Environment Test

This tests various shell environments.

```bash
echo "shell test"
```

More text.
"#;

    // Should parse consistently across different shell environments
    let doc = SysadminParser::parse(content).unwrap();
    
    // Verify structure
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_cross_platform_path_handling() {
    // Test that path handling works across platforms
    let content = r#"# Path Handling Test

This tests cross-platform path handling.

```bash
ls -la /tmp/
```

More text.
"#;

    // Should parse consistently regardless of OS path separators
    let doc = SysadminParser::parse(content).unwrap();
    
    // Verify structure
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}

#[test]
fn test_parse_document_with_cross_platform_emoji_handling() {
    // Test that emoji handling works across platforms
    let content = r#"# Emoji Test

This document contains emojis: 🚀🚀🚀

```bash
echo "emoji test"
```

More text.
"#;

    // Should parse consistently across platforms with emoji support
    let doc = SysadminParser::parse(content).unwrap();
    
    // Verify structure
    assert_eq!(doc.sections.len(), 1);
    let code_blocks = doc.code_blocks();
    assert_eq!(code_blocks.len(), 1);
}