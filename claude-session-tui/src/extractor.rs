//! Advanced block extraction system for conversation parsing and analysis

use crate::{error::Result, models::*, parser::ExtractionConfig};
use regex::Regex;
use std::collections::HashMap;

/// Advanced conversation block extractor with pattern recognition
pub struct BlockExtractor {
    /// Regex patterns for various content types
    patterns: ExtractionPatterns,
    /// Configuration for extraction behavior
    config: ExtractionConfig,
    /// Statistics tracking
    stats: ExtractionStats,
}

/// Pre-compiled regex patterns for efficient extraction
#[derive(Debug)]
struct ExtractionPatterns {
    code_block: Regex,
    inline_code: Regex,
    file_path: Regex,
    url: Regex,
    command: Regex,
    function_call: Regex,
    variable: Regex,
    markdown_link: Regex,
}

/// Statistics for extraction operations
#[derive(Debug, Default, Clone)]
pub struct ExtractionStats {
    pub total_blocks_processed: usize,
    pub code_blocks_extracted: usize,
    pub file_paths_found: usize,
    pub commands_identified: usize,
    pub urls_extracted: usize,
    pub errors_found: usize,
    pub programming_languages_detected: HashMap<ProgrammingLanguage, usize>,
    pub average_complexity_score: f64,
}

impl Default for BlockExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockExtractor {
    /// Create a new block extractor with default patterns
    pub fn new() -> Self {
        Self::with_config(ExtractionConfig {
            extract_code_blocks: true,
            extract_file_paths: true,
            extract_commands: true,
            extract_urls: true,
            tokenize_content: true,
            analyze_sentiment: false,
            detect_programming_languages: true,
        })
    }

    /// Create a block extractor with custom configuration
    pub fn with_config(config: ExtractionConfig) -> Self {
        let patterns = ExtractionPatterns::compile().unwrap_or_else(|e| {
            panic!("Failed to compile extraction patterns: {}", e);
        });

        Self {
            patterns,
            config,
            stats: ExtractionStats::default(),
        }
    }

    /// Extract comprehensive content from a conversation block
    pub fn extract_block_content(&mut self, raw_text: &str) -> Result<BlockContent> {
        self.stats.total_blocks_processed += 1;

        let word_count = raw_text.split_whitespace().count();
        let character_count = raw_text.chars().count();

        let mut content = BlockContent {
            raw_text: raw_text.to_string(),
            formatted_text: Some(self.format_text(raw_text)),
            tokens: Vec::new(),
            code_blocks: Vec::new(),
            links: Vec::new(),
            mentions: Vec::new(),
            word_count,
            character_count,
        };

        // Extract code blocks with language detection
        if self.config.extract_code_blocks {
            content.code_blocks = self.extract_code_blocks(raw_text)?;
            self.stats.code_blocks_extracted += content.code_blocks.len();
        }

        // Extract URLs and links
        if self.config.extract_urls {
            content.links = self.extract_links(raw_text)?;
            self.stats.urls_extracted += content.links.len();
        }

        // Tokenize content for search indexing
        if self.config.tokenize_content {
            content.tokens = self.tokenize_content(raw_text)?;
        }

        // Extract mentions of files, functions, etc.
        content.mentions = self.extract_mentions(raw_text)?;

        Ok(content)
    }

    /// Extract and classify code blocks with enhanced language detection
    fn extract_code_blocks(&mut self, text: &str) -> Result<Vec<CodeBlock>> {
        let mut code_blocks = Vec::new();

        // Extract fenced code blocks (```language\ncode```)
        for captures in self.patterns.code_block.captures_iter(text) {
            let language_hint = captures.get(1).map(|m| m.as_str());
            let code_content = captures.get(2).unwrap().as_str();
            let start_position = captures.get(0).unwrap().start();
            let end_position = captures.get(0).unwrap().end();

            let detected_language = if let Some(hint) = language_hint {
                self.detect_programming_language_from_hint(hint)
            } else {
                self.detect_programming_language_from_content(code_content)
            };

            // Update statistics
            if let Some(lang) = &detected_language {
                *self
                    .stats
                    .programming_languages_detected
                    .entry(lang.clone())
                    .or_insert(0) += 1;
            }

            // Check for filename hints in comments
            let filename = self.extract_filename_from_code(code_content);

            code_blocks.push(CodeBlock {
                language: detected_language,
                content: code_content.to_string(),
                line_numbers: false,
                filename,
                start_position,
                end_position,
            });
        }

        // Extract inline code (`code`)
        for captures in self.patterns.inline_code.captures_iter(text) {
            let code_content = captures.get(1).unwrap().as_str();
            let start_position = captures.get(0).unwrap().start();
            let end_position = captures.get(0).unwrap().end();

            // Only extract inline code that looks like actual code (has special chars)
            if self.looks_like_code(code_content) {
                let detected_language = self.detect_programming_language_from_content(code_content);

                code_blocks.push(CodeBlock {
                    language: detected_language,
                    content: code_content.to_string(),
                    line_numbers: false,
                    filename: None,
                    start_position,
                    end_position,
                });
            }
        }

        Ok(code_blocks)
    }

    /// Extract URLs and classify link types
    fn extract_links(&mut self, text: &str) -> Result<Vec<Link>> {
        let mut links = Vec::new();

        // Extract HTTP(S) URLs
        for captures in self.patterns.url.captures_iter(text) {
            let url = captures.get(0).unwrap().as_str().to_string();
            let link_type = self.classify_link_type(&url);
            let title = self.extract_link_title(text, &url);

            links.push(Link {
                url,
                title,
                link_type,
            });
        }

        // Extract markdown-style links [title](url)
        for captures in self.patterns.markdown_link.captures_iter(text) {
            let title = captures.get(1).map(|m| m.as_str().to_string());
            let url = captures.get(2).unwrap().as_str().to_string();
            let link_type = self.classify_link_type(&url);

            links.push(Link {
                url,
                title,
                link_type,
            });
        }

        Ok(links)
    }

    /// Advanced tokenization with semantic classification
    fn tokenize_content(&self, text: &str) -> Result<Vec<ContentToken>> {
        let mut tokens = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut position = 0;

        for word in words {
            // Skip to the actual position of this word in the text
            if let Some(word_pos) = text[position..].find(word) {
                position += word_pos;
            }

            let token_type = self.classify_token_advanced(
                word,
                &text[..position + word.len().min(text.len() - position)],
            );

            tokens.push(ContentToken {
                text: word.to_string(),
                token_type,
                position,
                length: word.len(),
            });

            position += word.len();
        }

        Ok(tokens)
    }

    /// Extract mentions of files, functions, classes, etc.
    fn extract_mentions(&self, text: &str) -> Result<Vec<Mention>> {
        let mut mentions = Vec::new();

        // File path mentions
        for captures in self.patterns.file_path.captures_iter(text) {
            let file_path = captures.get(0).unwrap().as_str();
            mentions.push(Mention {
                text: file_path.to_string(),
                mention_type: MentionType::File,
                context: Some(self.extract_mention_context(text, file_path)),
            });
        }

        // Function call mentions
        for captures in self.patterns.function_call.captures_iter(text) {
            let function_name = captures.get(1).unwrap().as_str();
            mentions.push(Mention {
                text: function_name.to_string(),
                mention_type: MentionType::Function,
                context: Some(captures.get(0).unwrap().as_str().to_string()),
            });
        }

        // Variable mentions (simple heuristic)
        for captures in self.patterns.variable.captures_iter(text) {
            let variable_name = captures.get(0).unwrap().as_str();
            if self.looks_like_variable(variable_name) {
                mentions.push(Mention {
                    text: variable_name.to_string(),
                    mention_type: MentionType::Variable,
                    context: None,
                });
            }
        }

        // Command mentions
        for captures in self.patterns.command.captures_iter(text) {
            let command = captures.get(0).unwrap().as_str();
            mentions.push(Mention {
                text: command.to_string(),
                mention_type: MentionType::Command,
                context: Some(self.extract_mention_context(text, command)),
            });
        }

        Ok(mentions)
    }

    /// Enhanced programming language detection from content analysis
    pub(crate) fn detect_programming_language_from_content(
        &self,
        code: &str,
    ) -> Option<ProgrammingLanguage> {
        let code_lower = code.to_lowercase();

        // Rust-specific patterns
        if code.contains("fn ")
            || code.contains("let ")
            || code.contains("impl ")
            || code.contains("struct ")
            || code.contains("enum ")
            || code.contains("trait ")
        {
            return Some(ProgrammingLanguage::Rust);
        }

        // Python-specific patterns
        if code.contains("def ")
            || code.contains("import ")
            || code.contains("from ")
            || code.contains("class ")
            || code_lower.contains("print(")
        {
            return Some(ProgrammingLanguage::Python);
        }

        // JavaScript/TypeScript patterns
        if code.contains("function ")
            || code.contains("const ")
            || code.contains("let ")
            || code.contains("var ")
            || code.contains("=>")
            || code.contains("console.log")
        {
            if code.contains(": ") && (code.contains("interface ") || code.contains("type ")) {
                return Some(ProgrammingLanguage::TypeScript);
            }
            return Some(ProgrammingLanguage::JavaScript);
        }

        // Java patterns
        if code.contains("public class ")
            || code.contains("private ")
            || code.contains("protected ")
            || code.contains("System.out.")
        {
            return Some(ProgrammingLanguage::Java);
        }

        // Go patterns
        if code.contains("func ")
            || code.contains("package ")
            || code.contains("import (")
            || code.contains("fmt.")
        {
            return Some(ProgrammingLanguage::Go);
        }

        // Shell script patterns
        if code.starts_with("#!/bin/bash")
            || code.starts_with("#!/bin/sh")
            || code.contains("echo ")
            || code.contains("$")
        {
            return Some(ProgrammingLanguage::Shell);
        }

        // SQL patterns
        if code_lower.contains("select ")
            || code_lower.contains("insert ")
            || code_lower.contains("update ")
            || code_lower.contains("create table")
        {
            return Some(ProgrammingLanguage::SQL);
        }

        // HTML patterns
        if code.contains("</")
            && code.contains("<")
            && (code.contains("html") || code.contains("div"))
        {
            return Some(ProgrammingLanguage::HTML);
        }

        // CSS patterns
        if code.contains("{")
            && code.contains(":")
            && code.contains(";")
            && (code.contains("color") || code.contains("margin") || code.contains("padding"))
        {
            return Some(ProgrammingLanguage::CSS);
        }

        // JSON patterns
        if (code.starts_with('{') && code.ends_with('}'))
            || (code.starts_with('[') && code.ends_with(']'))
        {
            if serde_json::from_str::<serde_json::Value>(code).is_ok() {
                return Some(ProgrammingLanguage::JSON);
            }
        }

        None
    }

    /// Detect programming language from language hint
    fn detect_programming_language_from_hint(&self, hint: &str) -> Option<ProgrammingLanguage> {
        match hint.to_lowercase().as_str() {
            "rust" | "rs" => Some(ProgrammingLanguage::Rust),
            "python" | "py" => Some(ProgrammingLanguage::Python),
            "javascript" | "js" => Some(ProgrammingLanguage::JavaScript),
            "typescript" | "ts" => Some(ProgrammingLanguage::TypeScript),
            "java" => Some(ProgrammingLanguage::Java),
            "go" => Some(ProgrammingLanguage::Go),
            "cpp" | "c++" => Some(ProgrammingLanguage::Cpp),
            "c" => Some(ProgrammingLanguage::C),
            "swift" => Some(ProgrammingLanguage::Swift),
            "kotlin" => Some(ProgrammingLanguage::Kotlin),
            "ruby" | "rb" => Some(ProgrammingLanguage::Ruby),
            "php" => Some(ProgrammingLanguage::PHP),
            "dart" => Some(ProgrammingLanguage::Dart),
            "shell" | "bash" | "sh" => Some(ProgrammingLanguage::Shell),
            "sql" => Some(ProgrammingLanguage::SQL),
            "html" => Some(ProgrammingLanguage::HTML),
            "css" => Some(ProgrammingLanguage::CSS),
            "markdown" | "md" => Some(ProgrammingLanguage::Markdown),
            "json" => Some(ProgrammingLanguage::JSON),
            "yaml" | "yml" => Some(ProgrammingLanguage::YAML),
            "toml" => Some(ProgrammingLanguage::TOML),
            "" => None, // Empty hint
            _ => Some(ProgrammingLanguage::Unknown(hint.to_string())),
        }
    }

    /// Advanced token classification with context awareness
    fn classify_token_advanced(&self, token: &str, context: &str) -> TokenType {
        // Check for specific patterns first
        if self.patterns.file_path.is_match(token) {
            return TokenType::FilePath;
        }

        if self.patterns.url.is_match(token) {
            return TokenType::URL;
        }

        if self.patterns.command.is_match(token) {
            return TokenType::Command;
        }

        // Numeric patterns
        if token
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        {
            return TokenType::Number;
        }

        // Programming constructs
        if token.ends_with("()") {
            return TokenType::Function;
        }

        if token.contains("::") || (token.contains('.') && context.contains("class")) {
            return TokenType::Method;
        }

        // Keywords (basic set - could be expanded)
        let keywords = [
            "function",
            "class",
            "struct",
            "enum",
            "interface",
            "type",
            "let",
            "const",
            "var",
            "def",
            "fn",
            "impl",
            "trait",
            "if",
            "else",
            "for",
            "while",
            "match",
            "switch",
            "import",
            "from",
            "use",
            "package",
            "namespace",
        ];

        if keywords.contains(&token.to_lowercase().as_str()) {
            return TokenType::Keyword;
        }

        // String patterns
        if (token.starts_with('"') && token.ends_with('"'))
            || (token.starts_with('\'') && token.ends_with('\''))
            || (token.starts_with('`') && token.ends_with('`'))
        {
            return TokenType::String;
        }

        // Punctuation
        if token.chars().all(|c| c.is_ascii_punctuation()) {
            return TokenType::Punctuation;
        }

        // Default to word
        TokenType::Word
    }

    /// Classify link types with enhanced detection
    fn classify_link_type(&self, url: &str) -> LinkType {
        let url_lower = url.to_lowercase();

        if url_lower.contains("github.com")
            || url_lower.contains("gitlab.com")
            || url_lower.contains("bitbucket.org")
        {
            return LinkType::Repository;
        }

        if url_lower.contains("docs.")
            || url_lower.contains("/docs/")
            || url_lower.contains("documentation")
            || url_lower.contains("/api/")
        {
            return LinkType::Documentation;
        }

        if url.starts_with("file://") || url_lower.contains("localhost") {
            return LinkType::File;
        }

        LinkType::External
    }

    /// Extract filename from code comments or content
    fn extract_filename_from_code(&self, code: &str) -> Option<String> {
        // Look for filename hints in comments
        let filename_patterns = [
            r"//\s*([^\s]+\.\w+)",   // // filename.ext
            r"#\s*([^\s]+\.\w+)",    // # filename.ext
            r"/\*\s*([^\s]+\.\w+)",  // /* filename.ext
            r"<!--\s*([^\s]+\.\w+)", // <!-- filename.ext
        ];

        for pattern in &filename_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(code) {
                    return Some(captures.get(1).unwrap().as_str().to_string());
                }
            }
        }

        None
    }

    /// Check if a string looks like code (heuristic)
    fn looks_like_code(&self, text: &str) -> bool {
        let code_indicators = ['(', ')', '{', '}', '[', ']', ';', '=', '.', ':', '$'];
        let indicator_count = text.chars().filter(|c| code_indicators.contains(c)).count();

        // If more than 20% of characters are code indicators, likely code
        indicator_count as f64 / text.len() as f64 > 0.2
    }

    /// Check if a string looks like a variable name
    fn looks_like_variable(&self, text: &str) -> bool {
        // Variable-like: starts with letter/underscore, contains only alphanumeric/underscore
        text.chars()
            .next()
            .map_or(false, |c| c.is_alphabetic() || c == '_')
            && text.chars().all(|c| c.is_alphanumeric() || c == '_')
            && text.len() > 1
    }

    /// Extract context around a mention
    fn extract_mention_context(&self, full_text: &str, mention: &str) -> String {
        if let Some(pos) = full_text.find(mention) {
            let start = pos.saturating_sub(50);
            let end = (pos + mention.len() + 50).min(full_text.len());
            full_text[start..end].to_string()
        } else {
            String::new()
        }
    }

    /// Extract link title from surrounding context
    fn extract_link_title(&self, text: &str, url: &str) -> Option<String> {
        // Look for markdown-style [title](url) patterns
        let pattern = format!(r"\[([^\]]+)\]\({}\)", regex::escape(url));
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(captures) = re.captures(text) {
                return Some(captures.get(1).unwrap().as_str().to_string());
            }
        }
        None
    }

    /// Format text with basic markdown cleanup
    fn format_text(&self, text: &str) -> String {
        // Remove excessive whitespace and normalize line endings
        let mut formatted = text.replace("\r\n", "\n");
        formatted = formatted.replace("\r", "\n");

        // Remove trailing whitespace from lines
        formatted = formatted
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        // Compress multiple blank lines into single blank line
        let multiple_newlines = Regex::new(r"\n{3,}").unwrap();
        formatted = multiple_newlines
            .replace_all(&formatted, "\n\n")
            .to_string();

        formatted
    }

    /// Get extraction statistics
    pub fn get_stats(&self) -> &ExtractionStats {
        &self.stats
    }

    /// Reset extraction statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExtractionStats::default();
    }
}

impl ExtractionPatterns {
    /// Compile all regex patterns used for extraction
    fn compile() -> Result<Self> {
        Ok(Self {
            code_block: Regex::new(r"(?s)```(\w+)?\s*\n(.*?)\n```")?,
            inline_code: Regex::new(r"`([^`]+)`")?,
            file_path: Regex::new(r"[/\w.-]+\.\w+")?,
            url: Regex::new(r"https?://[^\s)\]]+")?,
            command: Regex::new(r"(?:^|\s)(\$|>|\#)\s*([^\r\n]+)")?,
            function_call: Regex::new(r"(\w+)\s*\([^)]*\)")?,
            variable: Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b")?,
            markdown_link: Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_block_extraction() {
        let mut extractor = BlockExtractor::new();
        let text = "Here's some Rust code:\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\nAnd here's Python:\n```python\nprint(\"Hello, world!\")\n```";

        let content = extractor.extract_block_content(text).unwrap();
        assert_eq!(content.code_blocks.len(), 2);
        assert_eq!(
            content.code_blocks[0].language,
            Some(ProgrammingLanguage::Rust)
        );
        assert_eq!(
            content.code_blocks[1].language,
            Some(ProgrammingLanguage::Python)
        );
    }

    #[test]
    fn test_language_detection_from_content() {
        let extractor = BlockExtractor::new();

        assert_eq!(
            extractor.detect_programming_language_from_content("fn main() { println!(\"test\"); }"),
            Some(ProgrammingLanguage::Rust)
        );

        assert_eq!(
            extractor.detect_programming_language_from_content("def hello():\n    print(\"test\")"),
            Some(ProgrammingLanguage::Python)
        );
    }

    #[test]
    fn test_url_extraction() {
        let mut extractor = BlockExtractor::new();
        let text = "Check out https://github.com/rust-lang/rust and also https://docs.rs/tokio";

        let content = extractor.extract_block_content(text).unwrap();
        assert_eq!(content.links.len(), 2);
        assert_eq!(content.links[0].link_type, LinkType::Repository);
        assert_eq!(content.links[1].link_type, LinkType::Documentation);
    }

    #[test]
    fn test_token_classification() {
        let extractor = BlockExtractor::new();

        assert_eq!(
            extractor.classify_token_advanced("hello()", ""),
            TokenType::Function
        );
        assert_eq!(
            extractor.classify_token_advanced("/path/to/file.txt", ""),
            TokenType::FilePath
        );
        assert_eq!(
            extractor.classify_token_advanced("function", ""),
            TokenType::Keyword
        );
        assert_eq!(
            extractor.classify_token_advanced("123", ""),
            TokenType::Number
        );
    }

    #[test]
    fn test_mention_extraction() {
        let extractor = BlockExtractor::new();
        let text = "Let's edit the file /src/main.rs and call the function process_data()";

        let mentions = extractor.extract_mentions(text).unwrap();
        assert!(mentions.iter().any(|m| m.mention_type == MentionType::File));
        assert!(mentions
            .iter()
            .any(|m| m.mention_type == MentionType::Function));
    }
}
