//! Core trait definitions for TUI v2 Shadow Renaissance Architecture
//!
//! These traits define the contracts for all v2 components, enabling
//! parallel development by shadow agents while maintaining v1 compatibility.

use async_trait::async_trait;
use futures::Stream;
use std::error::Error;
use std::path::Path;
use std::pin::Pin;

/// Core session data provider trait with streaming support
#[async_trait]
pub trait SessionProvider: Send + Sync {
    type Error: Error + Send + Sync + 'static;
    type Session: SessionData;

    async fn load_session(&self, path: &Path) -> Result<Self::Session, Self::Error>;
    async fn list_sessions(&self) -> Result<Vec<SessionMetadata>, Self::Error>;

    // Streaming support for large datasets
    async fn stream_sessions(
        &self,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Session, Self::Error>> + Send>>;
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Parser abstraction for v2 with parallel processing support
#[async_trait]
pub trait AsyncParser<Input, Output>: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    async fn parse_single(&self, input: Input) -> Result<Output, Self::Error>;
    async fn parse_batch(&self, inputs: Vec<Input>) -> Result<Vec<Output>, Self::Error>;

    // Streaming parse for large files with backpressure control
    async fn parse_stream<S>(
        &self,
        input: S,
    ) -> Pin<Box<dyn Stream<Item = Result<Output, Self::Error>> + Send>>
    where
        S: Stream<Item = Input> + Send + 'static;

    fn supports_parallel(&self) -> bool {
        true
    }
    fn max_concurrency(&self) -> usize {
        10
    }
}

/// Insights engine trait for conversation analysis with incremental support
#[async_trait]
pub trait InsightsEngine<Data>: Send + Sync {
    type Insight: InsightData;
    type Error: Error + Send + Sync + 'static;

    async fn analyze(&self, data: &Data) -> Result<Self::Insight, Self::Error>;
    async fn analyze_batch(&self, data: Vec<Data>) -> Result<Vec<Self::Insight>, Self::Error>;

    // Streaming insights with real-time processing
    async fn stream_insights<S>(
        &self,
        data_stream: S,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Insight, Self::Error>> + Send>>
    where
        S: Stream<Item = Data> + Send + 'static;

    fn supports_incremental(&self) -> bool {
        false
    }
    fn cache_size(&self) -> usize {
        1000
    }
}

/// UI renderer trait for decoupled presentation layer with virtualization
#[async_trait]
pub trait AsyncRenderer<T>: Send + Sync {
    type Error: Error + Send + Sync + 'static;
    type RenderContext: Send + Sync;

    async fn render(&self, data: &T, context: &Self::RenderContext) -> Result<(), Self::Error>;

    // Streaming render with frame rate control
    async fn stream_render<S>(
        &self,
        data_stream: S,
        context: &Self::RenderContext,
    ) -> Result<(), Self::Error>
    where
        S: Stream<Item = T> + Send + Unpin;

    fn supports_virtualization(&self) -> bool {
        false
    }
    fn target_fps(&self) -> u32 {
        60
    }
}

/// Search engine abstraction with indexing and real-time updates
#[async_trait]
pub trait SearchEngine<Query, SearchResult>: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    async fn search(&self, query: Query) -> Result<Vec<SearchResult>, Self::Error>;
    async fn index(&mut self, documents: Vec<Document>) -> Result<(), Self::Error>;

    // Real-time search with streaming results
    async fn stream_search<S>(
        &self,
        query_stream: S,
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<SearchResult>, Self::Error>> + Send>>
    where
        S: Stream<Item = Query> + Send + 'static;

    fn supports_fuzzy(&self) -> bool {
        true
    }
    fn supports_incremental_index(&self) -> bool {
        false
    }
}

// Marker traits for associated types

pub trait SessionData: Send + Sync + Clone + 'static {
    fn id(&self) -> &str;
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn message_count(&self) -> usize;
}

pub trait MessageData: Send + Sync + Clone + 'static {
    fn content(&self) -> &str;
    fn role(&self) -> MessageRole;
    fn timestamp(&self) -> Option<chrono::DateTime<chrono::Utc>>;
}

pub trait InsightData: Send + Sync + Clone + 'static {
    fn category(&self) -> InsightCategory;
    fn confidence(&self) -> f32;
    fn summary(&self) -> &str;
}

// Supporting types

#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
    pub file_path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum MessageRole {
    Human,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Copy)]
pub enum InsightCategory {
    Topic,
    Sentiment,
    Complexity,
    ToolUsage,
    ErrorPattern,
    Performance,
}

/// Performance metrics for v2 components
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub parse_duration: std::time::Duration,
    pub memory_usage: usize,
    pub throughput: f64, // items per second
    pub error_rate: f32,
}

/// Shadow agent marker trait for specialized implementations
pub trait ShadowAgent: Send + Sync {
    const NAME: &'static str;
    const SPECIALIZATION: &'static str;

    fn performance_targets(&self) -> PerformanceMetrics;
}
