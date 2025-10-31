//! Comprehensive error handling for the search engine with resilience patterns

use thiserror::Error;
use std::time::Duration;

/// Result type alias for search operations
pub type SearchResult<T> = std::result::Result<T, SearchError>;

/// Comprehensive error types for search operations
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Index error: {message}")]
    Index { message: String, recoverable: bool },

    #[error("Query parsing error: {message} at position {position}")]
    QueryParsing { message: String, position: usize },

    #[error("Pattern recognition error: {details}")]
    PatternRecognition { details: String },

    #[error("IO error during search operation: {source}")]
    Io { source: std::io::Error },

    #[error("Tantivy error: {source}")]
    Tantivy { source: tantivy::TantivyError },

    #[error("Concurrent access error: {operation}")]
    ConcurrentAccess { operation: String },

    #[error("Memory pressure detected: {usage_mb}MB, limit: {limit_mb}MB")]
    MemoryPressure { usage_mb: usize, limit_mb: usize },

    #[error("Index corruption detected: {corruption_type}")]
    IndexCorruption { corruption_type: CorruptionType },

    #[error("Query timeout: operation took {actual:?}, limit: {limit:?}")]
    QueryTimeout { actual: Duration, limit: Duration },

    #[error("Search capacity exceeded: {current}/{limit} concurrent searches")]
    CapacityExceeded { current: usize, limit: usize },

    #[error("Schema mismatch: expected {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },

    #[error("Background task failed: {task_name} - {reason}")]
    BackgroundTaskFailure { task_name: String, reason: String },

    #[error("Index lock timeout: failed to acquire {lock_type} lock")]
    IndexLockTimeout { lock_type: String },

    #[error("Disk space critical: {available_mb}MB available, need {required_mb}MB")]
    DiskSpaceCritical { available_mb: u64, required_mb: u64 },

    #[error("Cache eviction failed: {reason}")]
    CacheEviction { reason: String },

    #[error("Multiple search failures: {failure_count} operations failed")]
    MultipleFailures { failure_count: usize },
}

/// Types of index corruption
#[derive(Debug, Clone)]
pub enum CorruptionType {
    MetadataCorrupt,
    SegmentCorrupt,
    SchemaCorrupt,
    DocumentCorrupt,
    Unknown(String),
}

impl std::fmt::Display for CorruptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MetadataCorrupt => write!(f, "metadata corruption"),
            Self::SegmentCorrupt => write!(f, "segment corruption"),
            Self::SchemaCorrupt => write!(f, "schema corruption"),
            Self::DocumentCorrupt => write!(f, "document corruption"),
            Self::Unknown(details) => write!(f, "unknown corruption: {}", details),
        }
    }
}

impl From<std::io::Error> for SearchError {
    fn from(source: std::io::Error) -> Self {
        Self::Io { source }
    }
}

impl From<tantivy::TantivyError> for SearchError {
    fn from(source: tantivy::TantivyError) -> Self {
        Self::Tantivy { source }
    }
}

impl SearchError {
    /// Check if the error is recoverable through retry or fallback
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Index { recoverable, .. } => *recoverable,
            Self::QueryParsing { .. } => false,
            Self::PatternRecognition { .. } => true,
            Self::Io { .. } => true,
            Self::Tantivy { source } => {
                // Some Tantivy errors are recoverable
                matches!(
                    source,
                    tantivy::TantivyError::IoError(_) | 
                    tantivy::TantivyError::LockFailure(_)
                )
            }
            Self::ConcurrentAccess { .. } => true,
            Self::MemoryPressure { .. } => true,
            Self::IndexCorruption { .. } => false,
            Self::QueryTimeout { .. } => true,
            Self::CapacityExceeded { .. } => true,
            Self::SchemaMismatch { .. } => false,
            Self::BackgroundTaskFailure { .. } => true,
            Self::IndexLockTimeout { .. } => true,
            Self::DiskSpaceCritical { .. } => false,
            Self::CacheEviction { .. } => true,
            Self::MultipleFailures { .. } => true,
        }
    }

    /// Get error severity for logging and alerting
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::IndexCorruption { .. } | Self::DiskSpaceCritical { .. } => ErrorSeverity::Critical,
            Self::Io { .. } | Self::SchemaMismatch { .. } => ErrorSeverity::Error,
            Self::QueryParsing { .. } | Self::PatternRecognition { .. } => ErrorSeverity::Warning,
            Self::QueryTimeout { .. } | Self::MemoryPressure { .. } => ErrorSeverity::Info,
            _ => ErrorSeverity::Warning,
        }
    }

    /// Get recommended recovery strategy
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Index { recoverable: true, .. } => RecoveryStrategy::Retry,
            Self::IndexCorruption { .. } => RecoveryStrategy::Rebuild,
            Self::MemoryPressure { .. } => RecoveryStrategy::GracefulDegradation,
            Self::QueryTimeout { .. } => RecoveryStrategy::CircuitBreaker,
            Self::CapacityExceeded { .. } => RecoveryStrategy::Throttle,
            Self::BackgroundTaskFailure { .. } => RecoveryStrategy::RestartTask,
            Self::IndexLockTimeout { .. } => RecoveryStrategy::Retry,
            Self::CacheEviction { .. } => RecoveryStrategy::ClearCache,
            _ => RecoveryStrategy::None,
        }
    }

    /// Create an index error with recovery information
    pub fn index_error<S: Into<String>>(message: S, recoverable: bool) -> Self {
        Self::Index {
            message: message.into(),
            recoverable,
        }
    }

    /// Create a query parsing error with position context
    pub fn query_parsing<S: Into<String>>(message: S, position: usize) -> Self {
        Self::QueryParsing {
            message: message.into(),
            position,
        }
    }

    /// Create a pattern recognition error
    pub fn pattern_recognition<S: Into<String>>(details: S) -> Self {
        Self::PatternRecognition {
            details: details.into(),
        }
    }

    /// Create a memory pressure error
    pub fn memory_pressure(usage_mb: usize, limit_mb: usize) -> Self {
        Self::MemoryPressure { usage_mb, limit_mb }
    }

    /// Create an index corruption error
    pub fn index_corruption(corruption_type: CorruptionType) -> Self {
        Self::IndexCorruption { corruption_type }
    }

    /// Create a query timeout error
    pub fn query_timeout(actual: Duration, limit: Duration) -> Self {
        Self::QueryTimeout { actual, limit }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Recovery strategies for different error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    None,
    Retry,
    Rebuild,
    GracefulDegradation,
    CircuitBreaker,
    Throttle,
    RestartTask,
    ClearCache,
}

/// Error context with metadata for debugging and monitoring
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub index_path: Option<String>,
    pub query: Option<String>,
    pub session_id: Option<String>,
    pub performance_impact: PerformanceImpact,
    pub user_impact: UserImpact,
    pub recovery_attempts: usize,
}

/// Performance impact classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceImpact {
    None,
    Minimal,
    Moderate,
    Severe,
}

/// User impact classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserImpact {
    None,
    Delayed,
    Degraded,
    Failed,
}

impl ErrorContext {
    pub fn new<S: Into<String>>(operation: S) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            operation: operation.into(),
            index_path: None,
            query: None,
            session_id: None,
            performance_impact: PerformanceImpact::None,
            user_impact: UserImpact::None,
            recovery_attempts: 0,
        }
    }

    pub fn with_index_path<S: Into<String>>(mut self, path: S) -> Self {
        self.index_path = Some(path.into());
        self
    }

    pub fn with_query<S: Into<String>>(mut self, query: S) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn with_session_id<S: Into<String>>(mut self, session_id: S) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_performance_impact(mut self, impact: PerformanceImpact) -> Self {
        self.performance_impact = impact;
        self
    }

    pub fn with_user_impact(mut self, impact: UserImpact) -> Self {
        self.user_impact = impact;
        self
    }

    pub fn increment_recovery_attempts(&mut self) {
        self.recovery_attempts += 1;
    }
}

/// Circuit breaker for protecting against cascading failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub state: CircuitState,
    pub failure_count: usize,
    pub failure_threshold: usize,
    pub timeout: Duration,
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    pub success_threshold: usize,
    pub half_open_success_count: usize,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration, success_threshold: usize) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            timeout,
            last_failure: None,
            success_threshold,
            half_open_success_count: 0,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure {
                    let now = chrono::Utc::now();
                    if now.signed_duration_since(last_failure).to_std().unwrap_or(Duration::ZERO) > self.timeout {
                        self.state = CircuitState::HalfOpen;
                        self.half_open_success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.half_open_success_count += 1;
                if self.half_open_success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.last_failure = None;
                }
            }
            CircuitState::Open => {}
        }
    }

    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure = Some(chrono::Utc::now());
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_failure = Some(chrono::Utc::now());
            }
            CircuitState::Open => {
                self.last_failure = Some(chrono::Utc::now());
            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.state == CircuitState::Open
    }
}