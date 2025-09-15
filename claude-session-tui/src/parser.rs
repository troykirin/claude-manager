//! High-performance JSONL parser for Claude session files with streaming support

use crate::{
    error::{BatchParsingResult, ClaudeSessionError, ErrorContext, ErrorSeverity, Result},
    models::*,
};

use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader as AsyncBufReader},
    sync::Semaphore,
    task::JoinHandle,
};
use tracing::{error, info, warn};
use walkdir::WalkDir;

/// High-performance session parser with streaming capabilities
pub struct SessionParser {
    /// Maximum number of concurrent file processing tasks
    max_concurrent_files: usize,
    /// Memory limit for batch operations (in MB)
    memory_limit_mb: usize,
    /// Performance monitoring
    performance_threshold_ms: u64,
    /// Error recovery settings
    error_recovery: ErrorRecoverySettings,
    /// Content extraction settings
    extraction_config: ExtractionConfig,
}

/// Error recovery configuration
#[derive(Debug, Clone)]
pub struct ErrorRecoverySettings {
    pub skip_malformed_lines: bool,
    pub max_consecutive_errors: usize,
    pub continue_on_critical_errors: bool,
    pub detailed_error_reporting: bool,
}

/// Content extraction configuration
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub extract_code_blocks: bool,
    pub extract_file_paths: bool,
    pub extract_commands: bool,
    pub extract_urls: bool,
    pub tokenize_content: bool,
    pub analyze_sentiment: bool,
    pub detect_programming_languages: bool,
}

impl Default for SessionParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionParser {
    /// Create a new session parser with default configuration
    pub fn new() -> Self {
        Self {
            max_concurrent_files: num_cpus::get().min(16),
            memory_limit_mb: 1024,          // 1GB default
            performance_threshold_ms: 5000, // 5 seconds
            error_recovery: ErrorRecoverySettings {
                skip_malformed_lines: true,
                max_consecutive_errors: 10,
                continue_on_critical_errors: false,
                detailed_error_reporting: true,
            },
            extraction_config: ExtractionConfig {
                extract_code_blocks: true,
                extract_file_paths: true,
                extract_commands: true,
                extract_urls: true,
                tokenize_content: true,
                analyze_sentiment: false, // Expensive operation
                detect_programming_languages: true,
            },
        }
    }

    /// Create a parser with custom configuration
    pub fn with_config(
        max_concurrent_files: usize,
        memory_limit_mb: usize,
        error_recovery: ErrorRecoverySettings,
        extraction_config: ExtractionConfig,
    ) -> Self {
        Self {
            max_concurrent_files,
            memory_limit_mb,
            performance_threshold_ms: 5000,
            error_recovery,
            extraction_config,
        }
    }

    /// Parse a single JSONL file asynchronously
    pub async fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Session> {
        let path = path.as_ref();
        let start_time = Instant::now();

        info!("Starting to parse session file: {}", path.display());

        // Validate file exists and is readable
        if !path.exists() {
            return Err(ClaudeSessionError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let metadata = fs::metadata(path).await?;
        let file_size = metadata.len();

        // Check memory constraints for large files
        if file_size as usize / 1_048_576 > self.memory_limit_mb {
            return Err(ClaudeSessionError::MemoryLimit {
                usage_mb: (file_size as usize / 1_048_576),
                limit_mb: self.memory_limit_mb,
            });
        }

        let file = fs::File::open(path).await?;
        let reader = AsyncBufReader::new(file);
        let mut lines = reader.lines();

        let mut session = Session::new();
        session.metadata.file_path = path.to_string_lossy().to_string();
        session.metadata.file_size_bytes = file_size;
        session.metadata.last_modified = metadata
            .modified()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(|_| chrono::Utc::now());

        let mut line_number = 0;
        let mut consecutive_errors = 0;
        let mut total_lines = 0;

        // Stream processing of JSONL lines
        while let Some(line) = lines.next_line().await? {
            line_number += 1;
            total_lines += 1;

            match self.parse_jsonl_line(&line, line_number) {
                Ok(Some(raw_message)) => {
                    consecutive_errors = 0;
                    match self.convert_to_block(raw_message, line_number, &session) {
                        Ok(block) => session.add_block(block),
                        Err(e) => {
                            if self.error_recovery.detailed_error_reporting {
                                warn!(
                                    "Failed to convert message to block at line {}: {}",
                                    line_number, e
                                );
                            }
                            if !self.error_recovery.skip_malformed_lines {
                                return Err(e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    // Empty or comment line, skip
                    continue;
                }
                Err(e) => {
                    consecutive_errors += 1;

                    if consecutive_errors > self.error_recovery.max_consecutive_errors {
                        error!(
                            "Too many consecutive errors ({}), aborting parse",
                            consecutive_errors
                        );
                        return Err(ClaudeSessionError::MultipleParsing {
                            count: consecutive_errors,
                        });
                    }

                    if self.error_recovery.skip_malformed_lines {
                        warn!("Skipping malformed line {}: {}", line_number, e);
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        session.metadata.line_count = total_lines;
        session.metadata.created_at = session
            .blocks
            .first()
            .map(|b| b.timestamp)
            .unwrap_or(chrono::Utc::now());

        // Post-process session for insights and analysis
        self.analyze_session(&mut session).await?;

        let duration = start_time.elapsed();
        if duration.as_millis() as u64 > self.performance_threshold_ms {
            warn!(
                "Performance threshold exceeded for {}: {}ms > {}ms",
                path.display(),
                duration.as_millis(),
                self.performance_threshold_ms
            );
        }

        info!(
            "Successfully parsed {} blocks from {} lines in {}ms",
            session.blocks.len(),
            total_lines,
            duration.as_millis()
        );

        Ok(session)
    }

    /// Parse multiple files in parallel
    pub async fn parse_files<P: AsRef<Path> + Send + 'static>(
        &self,
        paths: Vec<P>,
    ) -> Result<Vec<Session>> {
        let start_time = Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_files));

        info!("Starting parallel parsing of {} files", paths.len());

        let tasks: Vec<JoinHandle<Result<Session>>> = paths
            .into_iter()
            .map(|path| {
                let parser = self.clone_config();
                let permit = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = permit.acquire().await.unwrap();
                    parser.parse_file(path).await
                })
            })
            .collect();

        let results = futures::future::try_join_all(tasks)
            .await
            .map_err(|e| ClaudeSessionError::TaskJoin(e))?;

        let mut sessions = Vec::new();
        let mut errors = 0;

        for result in results {
            match result {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    errors += 1;
                    if !self.error_recovery.continue_on_critical_errors {
                        return Err(e);
                    }
                    error!("Failed to parse file: {}", e);
                }
            }
        }

        let duration = start_time.elapsed();
        info!(
            "Parallel parsing completed: {} successful, {} errors in {}ms",
            sessions.len(),
            errors,
            duration.as_millis()
        );

        Ok(sessions)
    }

    /// Parse all JSONL files in a directory
    pub async fn parse_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<Session>> {
        let dir_path = dir_path.as_ref();

        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(ClaudeSessionError::FileNotFound {
                path: dir_path.to_string_lossy().to_string(),
            });
        }

        info!("Scanning directory for JSONL files: {}", dir_path.display());

        let jsonl_files: Vec<PathBuf> = WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("jsonl"))
                    .unwrap_or(false)
            })
            .map(|entry| entry.path().to_path_buf())
            .collect();

        info!("Found {} JSONL files to process", jsonl_files.len());

        if jsonl_files.is_empty() {
            warn!("No JSONL files found in directory: {}", dir_path.display());
            return Ok(Vec::new());
        }

        self.parse_files(jsonl_files).await
    }

    /// Parse JSONL files with comprehensive error reporting
    pub async fn parse_files_with_error_reporting<P: AsRef<Path> + Send + 'static>(
        &self,
        paths: Vec<P>,
    ) -> BatchParsingResult<Session> {
        let start_time = Instant::now();
        let mut result = BatchParsingResult::new();

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_files));
        let paths: Vec<_> = paths.into_iter().collect();
        result.performance_stats.files_processed = paths.len();

        let tasks: Vec<_> = paths
            .into_iter()
            .enumerate()
            .map(|(index, path)| {
                let parser = self.clone_config();
                let permit = semaphore.clone();
                let path_str = path.as_ref().to_string_lossy().to_string();

                tokio::spawn(async move {
                    let _permit = permit.acquire().await.unwrap();
                    (index, path_str, parser.parse_file(path).await)
                })
            })
            .collect();

        let results = futures::future::join_all(tasks).await;

        for (task_index, task_result) in results.into_iter().enumerate() {
            match task_result {
                Ok((_index, path_str, parse_result)) => match parse_result {
                    Ok(session) => {
                        result.performance_stats.lines_processed += session.metadata.line_count;
                        result.performance_stats.bytes_processed +=
                            session.metadata.file_size_bytes as usize;
                        result.successful.push(session);
                    }
                    Err(e) => {
                        let error_context = ErrorContext::new(
                            path_str,
                            None,
                            e.severity(),
                            e.to_string(),
                            e.is_recoverable(),
                        );
                        result.failed.push(error_context);
                    }
                },
                Err(join_error) => {
                    let error_context = ErrorContext::new(
                        format!("task_{}", task_index),
                        None,
                        ErrorSeverity::Critical,
                        format!("Task join error: {}", join_error),
                        false,
                    );
                    result.failed.push(error_context);
                }
            }
        }

        result.performance_stats.total_duration_ms = start_time.elapsed().as_millis() as u64;
        result.performance_stats.calculate_throughput();

        info!(
            "Batch parsing completed: {:.1}% success rate ({}/{} files)",
            result.success_rate() * 100.0,
            result.successful.len(),
            result.successful.len() + result.failed.len()
        );

        result
    }

    /// Parse a single JSONL line into a raw message
    fn parse_jsonl_line(&self, line: &str, line_number: usize) -> Result<Option<RawMessage>> {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            return Ok(None);
        }

        serde_json::from_str(line)
            .map(Some)
            .map_err(|e| ClaudeSessionError::json_parsing(line_number, e))
    }

    /// Convert raw message to structured block
    fn convert_to_block(
        &self,
        raw_message: RawMessage,
        line_number: usize,
        _session: &Session,
    ) -> Result<Block> {
        let role = Role::from_string(&raw_message.role)?;

        let timestamp = raw_message
            .timestamp
            .parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|_| ClaudeSessionError::invalid_timestamp(&raw_message.timestamp))?;

        let content = self.extract_block_content(&raw_message.content)?;

        // Extract data before moving raw_message fields
        let tools = self.extract_tool_invocations(&raw_message)?;
        let attachments = self.extract_attachments(&raw_message)?;
        let thread_id = raw_message.thread_id.clone();

        let block = Block {
            id: uuid::Uuid::new_v4(),
            sequence_number: line_number,
            role,
            timestamp,
            content,
            metadata: BlockMetadata {
                processing_time_ms: None,
                confidence_score: None,
                complexity_score: None,
                sentiment: None,
                topics: Vec::new(),
                intent: None,
                parent_block_id: None,
                thread_id,
            },
            tools,
            attachments,
            context_references: Vec::new(),
        };

        Ok(block)
    }

    /// Extract structured content from raw text
    fn extract_block_content(&self, raw_text: &str) -> Result<BlockContent> {
        let word_count = raw_text.split_whitespace().count();
        let character_count = raw_text.chars().count();

        let mut content = BlockContent {
            raw_text: raw_text.to_string(),
            formatted_text: None,
            tokens: Vec::new(),
            code_blocks: Vec::new(),
            links: Vec::new(),
            mentions: Vec::new(),
            word_count,
            character_count,
        };

        if self.extraction_config.extract_code_blocks {
            content.code_blocks = self.extract_code_blocks(raw_text)?;
        }

        if self.extraction_config.extract_urls {
            content.links = self.extract_links(raw_text)?;
        }

        if self.extraction_config.tokenize_content {
            content.tokens = self.tokenize_content(raw_text)?;
        }

        Ok(content)
    }

    /// Extract code blocks from text
    fn extract_code_blocks(&self, text: &str) -> Result<Vec<CodeBlock>> {
        use regex::Regex;

        let code_block_regex = Regex::new(r"```(\w+)?\n(.*?)\n```")?;
        let mut code_blocks = Vec::new();

        for captures in code_block_regex.captures_iter(text) {
            let language = captures
                .get(1)
                .map(|m| self.detect_programming_language(m.as_str()));

            let content = captures.get(2).unwrap().as_str().to_string();
            let start_position = captures.get(0).unwrap().start();
            let end_position = captures.get(0).unwrap().end();

            code_blocks.push(CodeBlock {
                language,
                content,
                line_numbers: false,
                filename: None,
                start_position,
                end_position,
            });
        }

        Ok(code_blocks)
    }

    /// Extract links from text
    fn extract_links(&self, text: &str) -> Result<Vec<Link>> {
        use regex::Regex;

        let url_regex = Regex::new(r"https?://[^\s)]+").unwrap();
        let mut links = Vec::new();

        for captures in url_regex.captures_iter(text) {
            let url = captures.get(0).unwrap().as_str().to_string();
            let link_type = self.classify_link_type(&url);

            links.push(Link {
                url,
                title: None,
                link_type,
            });
        }

        Ok(links)
    }

    /// Tokenize content for search indexing
    fn tokenize_content(&self, text: &str) -> Result<Vec<ContentToken>> {
        use regex::Regex;

        let token_regex = Regex::new(r"\b\w+\b|[^\w\s]").unwrap();
        let mut tokens = Vec::new();

        for (_position, captures) in token_regex.captures_iter(text).enumerate() {
            let token_text = captures.get(0).unwrap().as_str().to_string();
            let token_type = self.classify_token_type(&token_text);
            let start_pos = captures.get(0).unwrap().start();

            tokens.push(ContentToken {
                text: token_text.clone(),
                token_type,
                position: start_pos,
                length: token_text.len(),
            });
        }

        Ok(tokens)
    }

    /// Extract tool invocations from raw message
    fn extract_tool_invocations(&self, _raw_message: &RawMessage) -> Result<Vec<ToolInvocation>> {
        // This would parse the tools field from the raw message
        // Implementation depends on the actual Claude JSONL format
        Ok(Vec::new())
    }

    /// Extract attachments from raw message
    fn extract_attachments(&self, _raw_message: &RawMessage) -> Result<Vec<Attachment>> {
        // This would parse attachments from the raw message
        // Implementation depends on the actual Claude JSONL format
        Ok(Vec::new())
    }

    /// Detect programming language from code block language hint
    fn detect_programming_language(&self, lang_hint: &str) -> ProgrammingLanguage {
        match lang_hint.to_lowercase().as_str() {
            "rust" | "rs" => ProgrammingLanguage::Rust,
            "python" | "py" => ProgrammingLanguage::Python,
            "javascript" | "js" => ProgrammingLanguage::JavaScript,
            "typescript" | "ts" => ProgrammingLanguage::TypeScript,
            "java" => ProgrammingLanguage::Java,
            "go" => ProgrammingLanguage::Go,
            "cpp" | "c++" => ProgrammingLanguage::Cpp,
            "c" => ProgrammingLanguage::C,
            "swift" => ProgrammingLanguage::Swift,
            "kotlin" => ProgrammingLanguage::Kotlin,
            "ruby" | "rb" => ProgrammingLanguage::Ruby,
            "php" => ProgrammingLanguage::PHP,
            "dart" => ProgrammingLanguage::Dart,
            "shell" | "bash" | "sh" => ProgrammingLanguage::Shell,
            "sql" => ProgrammingLanguage::SQL,
            "html" => ProgrammingLanguage::HTML,
            "css" => ProgrammingLanguage::CSS,
            "markdown" | "md" => ProgrammingLanguage::Markdown,
            "json" => ProgrammingLanguage::JSON,
            "yaml" | "yml" => ProgrammingLanguage::YAML,
            "toml" => ProgrammingLanguage::TOML,
            _ => ProgrammingLanguage::Unknown(lang_hint.to_string()),
        }
    }

    /// Classify link type
    fn classify_link_type(&self, url: &str) -> LinkType {
        if url.contains("github.com") || url.contains("gitlab.com") {
            LinkType::Repository
        } else if url.contains("docs.") || url.contains("/docs/") {
            LinkType::Documentation
        } else if url.starts_with("file://") {
            LinkType::File
        } else {
            LinkType::External
        }
    }

    /// Classify token type for semantic analysis
    fn classify_token_type(&self, token: &str) -> TokenType {
        if token.chars().all(|c| c.is_ascii_digit()) {
            TokenType::Number
        } else if token.chars().all(|c| c.is_ascii_punctuation()) {
            TokenType::Punctuation
        } else if token.starts_with('/') && token.contains('.') {
            TokenType::FilePath
        } else if token.starts_with("http") {
            TokenType::URL
        } else {
            TokenType::Word
        }
    }

    /// Analyze session for insights and patterns
    async fn analyze_session(&self, session: &mut Session) -> Result<()> {
        // This would implement comprehensive session analysis
        // For now, just update basic statistics
        session.statistics.session_duration = session.duration();
        Ok(())
    }

    /// Clone configuration for concurrent processing
    fn clone_config(&self) -> SessionParser {
        SessionParser {
            max_concurrent_files: self.max_concurrent_files,
            memory_limit_mb: self.memory_limit_mb,
            performance_threshold_ms: self.performance_threshold_ms,
            error_recovery: self.error_recovery.clone(),
            extraction_config: self.extraction_config.clone(),
        }
    }
}

/// Raw message structure as it appears in JSONL files
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub thread_id: Option<String>,
    #[serde(default)]
    pub tools: serde_json::Value,
    #[serde(default)]
    pub attachments: serde_json::Value,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_parse_empty_file() {
        let parser = SessionParser::new();
        let temp_file = NamedTempFile::new().unwrap();

        let session = parser.parse_file(temp_file.path()).await.unwrap();
        assert_eq!(session.blocks.len(), 0);
    }

    #[tokio::test]
    async fn test_parse_malformed_jsonl() {
        let parser = SessionParser::new();
        let mut temp_file = tokio::fs::File::create("test.jsonl").await.unwrap();
        temp_file.write_all(b"invalid json\n").await.unwrap();
        temp_file.write_all(b"{\"role\":\"user\",\"content\":\"test\",\"timestamp\":\"2023-01-01T00:00:00Z\"}\n").await.unwrap();
        drop(temp_file);

        let session = parser.parse_file("test.jsonl").await.unwrap();
        assert_eq!(session.blocks.len(), 1);

        tokio::fs::remove_file("test.jsonl").await.unwrap();
    }

    #[test]
    fn test_role_parsing() {
        assert_eq!(Role::from_string("user").unwrap(), Role::User);
        assert_eq!(Role::from_string("assistant").unwrap(), Role::Assistant);
        assert!(Role::from_string("invalid").is_err());
    }

    #[test]
    fn test_programming_language_detection() {
        let parser = SessionParser::new();
        assert_eq!(
            parser.detect_programming_language("rust"),
            ProgrammingLanguage::Rust
        );
        assert_eq!(
            parser.detect_programming_language("python"),
            ProgrammingLanguage::Python
        );
        assert_eq!(
            parser.detect_programming_language("unknown"),
            ProgrammingLanguage::Unknown("unknown".to_string())
        );
    }
}
