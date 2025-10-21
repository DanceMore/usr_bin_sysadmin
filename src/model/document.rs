use super::block::{Block, CodeBlock};

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
    /// The level of the header (1-6 for h1-h6)
    pub header_level: Option<u32>,
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

    /// Count total number of executable steps
    pub fn step_count(&self) -> usize {
        self.code_blocks().len()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Section {
    pub fn new() -> Self {
        Section {
            header: None,
            header_level: None,
            blocks: Vec::new(),
        }
    }

    pub fn with_header(header: String, level: u32) -> Self {
        Section {
            header: Some(header),
            header_level: Some(level),
            blocks: Vec::new(),
        }
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
