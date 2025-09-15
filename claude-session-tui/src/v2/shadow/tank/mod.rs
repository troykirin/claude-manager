//! Tank - Parser Heavy Lifting Shadow Agent
//!
//! Specializes in high-performance async streaming JSONL parsing with parallelization
//! and intelligent error recovery for massive Claude session datasets.

use crate::v2::core::traits::{AsyncParser, PerformanceMetrics, ShadowAgent};
use crate::v2::shadow::beru::{BeruError, BeruMessage, BeruSession};
use async_trait::async_trait;
use futures::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Semaphore;
use tokio_stream::wrappers::LinesStream;

/// Tank's high-performance parallel parser
pub struct TankParser {
    max_concurrency: usize,
    chunk_size: usize,
    semaphore: Arc<Semaphore>,
    error_recovery: TankErrorRecovery,
}

impl TankParser {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            max_concurrency,
            chunk_size: 1000,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            error_recovery: TankErrorRecovery::new(),
        }
    }

    /// Parse JSONL with intelligent chunking and parallel processing
    pub async fn parse_jsonl_parallel(
        &self,
        content: String,
    ) -> Result<Vec<BeruSession>, TankError> {
        let lines: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(String::from)
            .collect();

        if lines.is_empty() {
            return Ok(vec![]);
        }

        // Group lines into sessions based on conversation patterns
        let session_groups = self.group_by_conversation(&lines).await?;

        // Process sessions in parallel with controlled concurrency
        let results = futures::future::try_join_all(session_groups.into_iter().map(|group| {
            let semaphore = Arc::clone(&self.semaphore);
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                self.parse_session_group(group).await
            }
        }))
        .await?;

        Ok(results)
    }

    /// Stream parse with backpressure control
    pub async fn stream_parse<R>(
        &self,
        reader: R,
    ) -> Pin<Box<dyn Stream<Item = Result<BeruSession, TankError>> + Send>>
    where
        R: tokio::io::AsyncRead + Send + Unpin + 'static,
    {
        let buf_reader = BufReader::new(reader);
        let lines_stream = LinesStream::new(buf_reader.lines());
        let max_concurrency = self.max_concurrency;

        // Create a simpler stream that processes lines one by one
        let session_stream =
            lines_stream
                .map_err(TankError::IoError)
                .and_then(move |line| async move {
                    // Simple approach: treat each line as a potential message
                    let temp_parser = TankParser::new(max_concurrency);
                    let lines = vec![line];
                    temp_parser.parse_session_group(lines).await
                });

        Box::pin(session_stream)
    }

    async fn group_by_conversation(&self, lines: &[String]) -> Result<Vec<Vec<String>>, TankError> {
        // Intelligent grouping based on conversation patterns
        let mut groups = Vec::new();
        let mut current_group = Vec::new();
        let mut last_timestamp = None;

        for line in lines {
            // Try to extract timestamp to detect conversation boundaries
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                let timestamp = parsed
                    .get("created_at")
                    .or_else(|| parsed.get("timestamp"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.timestamp());

                // Start new group if significant time gap (>1 hour)
                if let (Some(current_ts), Some(last_ts)) = (timestamp, last_timestamp) {
                    if current_ts - last_ts > 3600 {
                        if !current_group.is_empty() {
                            groups.push(std::mem::take(&mut current_group));
                        }
                    }
                }

                last_timestamp = timestamp;
            }

            current_group.push(line.clone());

            // Limit group size to prevent memory issues
            if current_group.len() >= self.chunk_size {
                groups.push(std::mem::take(&mut current_group));
            }
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        Ok(groups)
    }

    async fn parse_session_group(&self, lines: Vec<String>) -> Result<BeruSession, TankError> {
        let mut messages = Vec::with_capacity(lines.len());
        let mut parse_errors = 0;

        for line in lines {
            match serde_json::from_str::<BeruMessage>(&line) {
                Ok(message) => messages.push(message),
                Err(_e) => {
                    parse_errors += 1;

                    // Attempt flexible parsing
                    if let Ok(recovered_message) = self.recover_malformed_message(&line).await {
                        messages.push(recovered_message);
                    } else if parse_errors > 10 {
                        // Too many errors, fail the whole group
                        return Err(TankError::TooManyParseErrors(parse_errors));
                    }
                }
            }
        }

        if messages.is_empty() {
            return Err(TankError::EmptySession);
        }

        // Create session from messages
        let session_id = messages
            .first()
            .and_then(|m| Some(format!("session_{}", m.uuid)))
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Ok(BeruSession {
            uuid: session_id,
            name: Some(format!("Parsed Session (Tank)")),
            created_at: messages.first().unwrap().created_at,
            updated_at: messages.last().unwrap().created_at,
            messages,
            project_uuid: None,
            conversation_id: uuid::Uuid::new_v4().to_string(),
            metadata: Default::default(),
        })
    }

    async fn recover_malformed_message(&self, line: &str) -> Result<BeruMessage, TankError> {
        // Try various recovery strategies

        // Strategy 1: Fix common JSON issues
        let cleaned = line
            .replace(",}", "}") // Remove trailing commas
            .replace(",]", "]")
            .trim()
            .to_string();

        if let Ok(message) = serde_json::from_str::<BeruMessage>(&cleaned) {
            return Ok(message);
        }

        // Strategy 2: Extract key fields manually
        let partial: serde_json::Value =
            serde_json::from_str(&cleaned).map_err(|e| TankError::RecoveryFailed(e.to_string()))?;

        let uuid = partial
            .get("uuid")
            .or_else(|| partial.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or(&uuid::Uuid::new_v4().to_string())
            .to_string();

        let content = partial
            .get("content")
            .or_else(|| partial.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("[Recovered Content]")
            .to_string();

        let role = partial
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("human")
            .to_string();

        Ok(BeruMessage {
            uuid,
            content: crate::v2::shadow::beru::BeruMessageContent::Text(content),
            role,
            created_at: chrono::Utc::now(),
            updated_at: None,
            parent_uuid: None,
            is_active: true,
            attachments: vec![],
            tool_calls: None,
            usage: None,
            metadata: Default::default(),
        })
    }
}

#[async_trait]
impl AsyncParser<String, BeruSession> for TankParser {
    type Error = TankError;

    async fn parse_single(&self, input: String) -> Result<BeruSession, Self::Error> {
        self.parse_jsonl_parallel(input)
            .await?
            .into_iter()
            .next()
            .ok_or(TankError::EmptySession)
    }

    async fn parse_batch(&self, inputs: Vec<String>) -> Result<Vec<BeruSession>, Self::Error> {
        let results =
            futures::future::try_join_all(inputs.into_iter().map(|input| self.parse_single(input)))
                .await?;

        Ok(results)
    }

    async fn parse_stream<S>(
        &self,
        input: S,
    ) -> Pin<Box<dyn Stream<Item = Result<BeruSession, Self::Error>> + Send>>
    where
        S: Stream<Item = String> + Send + 'static,
    {
        let max_concurrency = self.max_concurrency;
        let stream = input.then(move |content| async move {
            let temp_parser = TankParser::new(max_concurrency);
            temp_parser.parse_single(content).await
        });

        Box::pin(stream)
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    fn max_concurrency(&self) -> usize {
        self.max_concurrency
    }
}

impl ShadowAgent for TankParser {
    const NAME: &'static str = "Tank";
    const SPECIALIZATION: &'static str = "Parser Heavy Lifting";

    fn performance_targets(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            parse_duration: std::time::Duration::from_millis(20), // 50x faster than v1
            memory_usage: 50 * 1024 * 1024,                       // 50MB for large files
            throughput: 5000.0,                                   // 5000 lines per second
            error_rate: 0.05, // 5% error tolerance with recovery
        }
    }
}

/// Error recovery strategies for Tank
#[derive(Clone)]
pub struct TankErrorRecovery {
    max_skip_errors: usize,
    skip_error_types: Vec<TankErrorType>,
}

impl TankErrorRecovery {
    pub fn new() -> Self {
        Self {
            max_skip_errors: 100,
            skip_error_types: vec![TankErrorType::JsonParse, TankErrorType::MissingField],
        }
    }

    pub fn should_skip_error(&self, error: &TankError) -> bool {
        match error {
            TankError::ParseError(_) => self.skip_error_types.contains(&TankErrorType::JsonParse),
            TankError::RecoveryFailed(_) => {
                self.skip_error_types.contains(&TankErrorType::MissingField)
            }
            TankError::TooManyParseErrors(_) => false, // Never skip this
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TankErrorType {
    JsonParse,
    MissingField,
    IoError,
    Timeout,
}

/// Tank-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TankError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
    #[error("Too many parse errors: {0}")]
    TooManyParseErrors(usize),
    #[error("Empty session")]
    EmptySession,
    #[error("Timeout")]
    Timeout,
}

impl From<BeruError> for TankError {
    fn from(err: BeruError) -> Self {
        TankError::ParseError(err.to_string())
    }
}
