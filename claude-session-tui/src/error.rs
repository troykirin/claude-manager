//! Error types for Claude session parsing

use thiserror::Error;

/// Result type alias for Claude session operations
pub type Result<T> = std::result::Result<T, ClaudeSessionError>;

/// Comprehensive error types for Claude session parsing
#[derive(Error, Debug)]
pub enum ClaudeSessionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error at line {line}: {source}")]
    JsonParsing {
        line: usize,
        source: serde_json::Error,
    },

    #[error("Invalid session format: {message}")]
    InvalidFormat { message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid timestamp format: {value}")]
    InvalidTimestamp { value: String },

    #[error("Unknown role: {role}")]
    UnknownRole { role: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Directory traversal error: {0}")]
    DirectoryTraversal(#[from] walkdir::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("Multiple parsing errors: {count} failures")]
    MultipleParsing { count: usize },

    #[error("Performance threshold exceeded: {operation} took {duration_ms}ms (limit: {limit_ms}ms)")]
    PerformanceThreshold {
        operation: String,
        duration_ms: u64,
        limit_ms: u64,
    },

    #[error("Memory limit exceeded: {usage_mb}MB (limit: {limit_mb}MB)")]
    MemoryLimit { usage_mb: usize, limit_mb: usize },

    #[error("Corrupted session data: {details}")]
    CorruptedData { details: String },

    #[error("Unsupported Claude version: {version}")]
    UnsupportedVersion { version: String },
}

impl ClaudeSessionError {
    /// Create a JSON parsing error with line context
    pub fn json_parsing(line: usize, source: serde_json::Error) -> Self {
        Self::JsonParsing { line, source }
    }

    /// Create an invalid format error
    pub fn invalid_format<S: Into<String>>(message: S) -> Self {
        Self::InvalidFormat {
            message: message.into(),
        }
    }

    /// Create a missing field error
    pub fn missing_field<S: Into<String>>(field: S) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Create an invalid timestamp error
    pub fn invalid_timestamp<S: Into<String>>(value: S) -> Self {
        Self::InvalidTimestamp {
            value: value.into(),
        }
    }

    /// Create an unknown role error
    pub fn unknown_role<S: Into<String>>(role: S) -> Self {
        Self::UnknownRole { role: role.into() }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::JsonParsing { .. }
                | Self::InvalidFormat { .. }
                | Self::MissingField { .. }
                | Self::InvalidTimestamp { .. }
                | Self::UnknownRole { .. }
                | Self::CorruptedData { .. }
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Io(_) | Self::FileNotFound { .. } | Self::DirectoryTraversal(_) => {
                ErrorSeverity::Critical
            }
            Self::JsonParsing { .. }
            | Self::InvalidFormat { .. }
            | Self::CorruptedData { .. }
            | Self::UnsupportedVersion { .. } => ErrorSeverity::Error,
            Self::MissingField { .. }
            | Self::InvalidTimestamp { .. }
            | Self::UnknownRole { .. } => ErrorSeverity::Warning,
            Self::PerformanceThreshold { .. } | Self::MemoryLimit { .. } => ErrorSeverity::Info,
            _ => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Critical,
    Error,
    Warning,
    Info,
}

/// Error context for batch operations
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file_path: String,
    pub line_number: Option<usize>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: ErrorSeverity,
    pub error: String,
    pub recoverable: bool,
}

impl ErrorContext {
    pub fn new<P: Into<String>, E: Into<String>>(
        file_path: P,
        line_number: Option<usize>,
        severity: ErrorSeverity,
        error: E,
        recoverable: bool,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            line_number,
            timestamp: chrono::Utc::now(),
            severity,
            error: error.into(),
            recoverable,
        }
    }
}

/// Batch parsing result with detailed error reporting
#[derive(Debug)]
pub struct BatchParsingResult<T> {
    pub successful: Vec<T>,
    pub failed: Vec<ErrorContext>,
    pub performance_stats: PerformanceStats,
}

impl<T> BatchParsingResult<T> {
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
            performance_stats: PerformanceStats::default(),
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.successful.len() + self.failed.len();
        if total == 0 {
            0.0
        } else {
            self.successful.len() as f64 / total as f64
        }
    }

    pub fn has_critical_errors(&self) -> bool {
        self.failed
            .iter()
            .any(|ctx| ctx.severity == ErrorSeverity::Critical)
    }
}

impl<T> Default for BatchParsingResult<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance statistics for parsing operations
#[derive(Debug, Default, Clone)]
pub struct PerformanceStats {
    pub total_duration_ms: u64,
    pub files_processed: usize,
    pub lines_processed: usize,
    pub bytes_processed: usize,
    pub average_file_time_ms: f64,
    pub peak_memory_usage_mb: usize,
    pub throughput_files_per_sec: f64,
    pub throughput_mb_per_sec: f64,
}

impl PerformanceStats {
    pub fn calculate_throughput(&mut self) {
        if self.total_duration_ms > 0 {
            let duration_sec = self.total_duration_ms as f64 / 1000.0;
            self.throughput_files_per_sec = self.files_processed as f64 / duration_sec;
            self.throughput_mb_per_sec =
                (self.bytes_processed as f64 / 1_048_576.0) / duration_sec;
        }
        
        if self.files_processed > 0 {
            self.average_file_time_ms = self.total_duration_ms as f64 / self.files_processed as f64;
        }
    }
}