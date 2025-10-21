//! Unit tests for the sysadmin TUI components

use crate::ui::{Renderer, TuiApp};
use crate::model::{Block, CodeBlock, Document, Section};

#[test]
fn test_renderer_new() {
    let renderer = Renderer::new();
    assert_eq!(renderer.current_step, 0);
    assert_eq!(renderer.total_steps, 0);
}

#[test]
fn test_renderer_set_total_steps() {
    let mut renderer = Renderer::new();
    renderer.set_total_steps(5);
    assert_eq!(renderer.total_steps, 5);
}

#[test]
fn test_tui_app_new() {
    let doc = Document::new();
    let app = TuiApp::new(doc);
    assert_eq!(app.current_step, 0);
    assert_eq!(app.scroll_offset, 0);
    assert_eq!(app.transient_message, None);
}

#[test]
fn test_tui_app_render_runbook_content_empty() {
    let doc = Document::new();
    let mut app = TuiApp::new(doc);
    
    // This should not panic and return an empty content
    let content = app.render_runbook_content();
    assert!(content.is_empty());
}

#[test]
fn test_tui_app_render_runbook_content_with_simple_document() {
    let mut doc = Document::new();
    let section = Section::with_header("Test Section".to_string(), 1);
    doc.sections.push(section);
    
    let mut app = TuiApp::new(doc);
    
    // This should not panic and return content
    let content = app.render_runbook_content();
    assert!(!content.is_empty());
}

#[test]
fn test_tui_app_render_runbook_content_with_code_blocks() {
    let mut doc = Document::new();
    let mut section = Section::with_header("Test Section".to_string(), 1);
    
    // Add a code block to the section
    let code_block = CodeBlock {
        language: "bash".to_string(),
        content: "echo \"hello\"".to_string(),
        line_number: 1,
    };
    
    section.blocks.push(Block::Code(code_block));
    doc.sections.push(section);
    
    let mut app = TuiApp::new(doc);
    
    // This should not panic and return content with code blocks
    let content = app.render_runbook_content();
    assert!(!content.is_empty());
}

#[test]
fn test_tui_app_highlight_code_line() {
    let mut doc = Document::new();
    let mut section = Section::with_header("Test Section".to_string(), 1);
    
    // Add a code block to the section
    let code_block = CodeBlock {
        language: "bash".to_string(),
        content: "echo \"hello\"".to_string(),
        line_number: 1,
    };
    
    section.blocks.push(Block::Code(code_block));
    doc.sections.push(section);
    
    let mut app = TuiApp::new(doc);
    
    // Test the highlighting function
    let base_style = ratatui::style::Style::default();
    let highlighted = app.highlight_code_line("echo \"hello\"", "bash", &base_style);
    
    // Should return at least one span
    assert!(!highlighted.is_empty());
}

#[test]
fn test_tui_app_auto_scroll_to_current_step() {
    let mut doc = Document::new();
    let mut section = Section::with_header("Test Section".to_string(), 1);
    
    // Add a code block to the section
    let code_block = CodeBlock {
        language: "bash".to_string(),
        content: "echo \"hello\"".to_string(),
        line_number: 1,
    };
    
    section.blocks.push(Block::Code(code_block));
    doc.sections.push(section);
    
    let mut app = TuiApp::new(doc);
    
    // Test that the scroll function doesn't panic
    app.auto_scroll_to_current_step();
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_tui_app_next_step() {
    let mut doc = Document::new();
    let mut section = Section::with_header("Test Section".to_string(), 1);
    
    // Add a code block to the section
    let code_block = CodeBlock {
        language: "bash".to_string(),
        content: "echo \"hello\"".to_string(),
        line_number: 1,
    };
    
    section.blocks.push(Block::Code(code_block));
    doc.sections.push(section);
    
    let mut app = TuiApp::new(doc);
    
    // Test that next_step function doesn't panic
    app.next_step();
    assert_eq!(app.current_step, 1);
}

#[test]
fn test_tui_app_previous_step() {
    let mut doc = Document::new();
    let mut section = Section::with_header("Test Section".to_string(), 1);
    
    // Add a code block to the section
    let code_block = CodeBlock {
        language: "bash".to_string(),
        content: "echo \"hello\"".to_string(),
        line_number: 1,
    };
    
    section.blocks.push(Block::Code(code_block));
    doc.sections.push(section);
    
    let mut app = TuiApp::new(doc);
    
    // Test that previous_step function doesn't panic
    app.previous_step();
    assert_eq!(app.current_step, 0);
}