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
            "zsh" => "zsh",
            "fish" => "fish",
            _ => "bash", // default fallback
        }
    }

    /// Check if this is a shell-like language
    pub fn is_shell(&self) -> bool {
        matches!(
            self.language.as_str(),
            "bash" | "sh" | "zsh" | "fish"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
