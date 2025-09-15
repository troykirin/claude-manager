//! Core trait definitions for TUI v2 Shadow Renaissance Architecture
//! 
//! These traits define the contracts for all v2 components, enabling
//! parallel development by shadow agents while maintaining v1 compatibility.

use async_trait::async_trait;
use std::error::Error;
use std::path::Path;
use futures::Stream;
use std::pin::Pin;

/// Core session data provider trait
#[async_trait]
pub trait SessionProvider: Send + Sync {
    type Error: Error + Send + Sync + 'static;
    type Session: SessionData;
    
    async fn load_session(&self, path: &Path) -> Result<Self::Session, Self::Error>;
    async fn list_sessions(&self) -> Result<Vec<SessionMetadata>, Self::Error>;
    fn supports_streaming(&self) -> bool { false }
}

/// Parser abstraction for v2 with parallel processing support
#[async_trait]
pub trait Parser: Send + Sync {
    type Input: AsRef<[u8]>;
    type Output: MessageData;
    type Error: Error + Send + Sync + 'static;
    
    async fn parse(&self, input: Self::Input) -> Result<Vec<Self::Output>, Self::Error>;
    fn parse_streaming(&self) -> Pin<Box<dyn Stream<Item = Result<Self::Output, Self::Error>> + Send>>;
    fn supports_parallel(&self) -> bool { true }
}

/// Insights engine trait for conversation analysis
#[async_trait]
pub trait InsightsEngine: Send + Sync {
    type Message: MessageData;
    type Insight: InsightData;
    type Error: Error + Send + Sync + 'static;
    
    async fn analyze(&self, messages: &[Self::Message]) -> Result<Vec<Self::Insight>, Self::Error>;
    fn compute_metrics(&self, messages: &[Self::Message]) -> MetricsBundle;
    fn extract_patterns(&self, messages: &[Self::Message]) -> Vec<Pattern>;
    fn supports_incremental(&self) -> bool { false }
}

/// UI renderer trait for decoupling presentation layer
#[async_trait]
pub trait UIRenderer: Send + Sync {
    type State: UIState;
    type Event: UIEvent;
    type Error: Error + Send + Sync + 'static;
    
    async fn render(&mut self, state: &Self::State) -> Result<(), Self::Error>;
    async fn handle_event(&mut self, event: Self::Event) -> Result<Option<Action>, Self::Error>;
    fn supports_virtualization(&self) -> bool { false }
}

/// Search engine abstraction with indexing support
#[async_trait]
pub trait SearchEngine: Send + Sync {
    type Query: SearchQuery;
    type Result: SearchResult;
    type Error: Error + Send + Sync + 'static;
    
    async fn search(&self, query: Self::Query) -> Result<Vec<Self::Result>, Self::Error>;
    async fn index(&mut self, documents: Vec<Document>) -> Result<(), Self::Error>;
    fn supports_fuzzy(&self) -> bool { true }
    fn supports_incremental_index(&self) -> bool { false }
}

// Marker traits for associated types

pub trait SessionData: Send + Sync + Clone + 'static {
    fn id(&self) -> &str;
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
}

pub trait MessageData: Send + Sync + Clone + 'static {
    fn content(&self) -> &str;
    fn role(&self) -> MessageRole;
    fn timestamp(&self) -> Option<chrono::DateTime<chrono::Utc>>;
}

pub trait InsightData: Send + Sync + Clone + 'static {
    fn category(&self) -> InsightCategory;
    fn confidence(&self) -> f32;
}

pub trait UIState: Send + Sync + 'static {
    fn current_view(&self) -> ViewType;
}

pub trait UIEvent: Send + Sync + 'static {
    fn as_key_event(&self) -> Option<KeyEvent>;
}

pub trait SearchQuery: Send + Sync + 'static {
    fn query_string(&self) -> &str;
}

pub trait SearchResult: Send + Sync + 'static {
    fn score(&self) -> f32;
    fn highlight(&self) -> Option<String>;
}

// Supporting types

#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
}

#[derive(Debug, Clone)]
pub struct MetricsBundle {
    pub total_messages: usize,
    pub average_message_length: f32,
    pub conversation_duration: std::time::Duration,
    pub tool_usage_count: usize,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub occurrences: usize,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone, Copy)]
pub enum ViewType {
    ConversationList,
    ConversationDetail,
    Search,
    Insights,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub code: crossterm::event::KeyCode,
    pub modifiers: crossterm::event::KeyModifiers,
}

#[derive(Debug, Clone, Copy)]
pub enum PatternType {
    ErrorRecovery,
    ToolChaining,
    ConversationFlow,
    TopicTransition,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum Action {
    Navigate(ViewType),
    Search(String),
    Select(String),
    Exit,
}