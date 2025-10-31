//! High-performance search engine for Claude conversation blocks
//! 
//! Provides blazing-fast, resilient search across millions of conversation blocks
//! with advanced pattern recognition and real-time indexing capabilities.

pub mod engine;
pub mod query;
pub mod patterns;
pub mod indexer;
pub mod cache;
pub mod error;

pub use engine::*;
pub use query::*;
pub use patterns::*;
pub use indexer::*;
pub use cache::*;
pub use error::*;

use crate::{Session, Block, Role};
use std::path::Path;

/// Initialize the search system with default configuration
pub async fn init_search_system<P: AsRef<Path>>(index_dir: P) -> SearchResult<SearchEngine> {
    SearchEngine::new(index_dir).await
}

/// Quick search interface for simple queries
pub async fn search_blocks(
    engine: &SearchEngine,
    query: &str,
    max_results: Option<usize>,
) -> SearchResult<Vec<BlockSearchResult>> {
    let search_query = SearchQuery::simple(query, max_results);
    engine.search(&search_query).await
}

/// Index a session with real-time processing
pub async fn index_session(
    engine: &mut SearchEngine,
    session: &Session,
) -> SearchResult<IndexingStats> {
    engine.index_session(session).await
}

/// Find patterns across indexed conversations
pub async fn find_conversation_patterns(
    engine: &SearchEngine,
    pattern_type: PatternType,
    confidence_threshold: f64,
) -> SearchResult<Vec<PatternMatch>> {
    engine.find_patterns(pattern_type, confidence_threshold).await
}

/// Batch index multiple sessions with progress reporting
pub async fn batch_index_sessions(
    engine: &mut SearchEngine,
    sessions: Vec<&Session>,
    progress_callback: Option<Box<dyn Fn(usize, usize) + Send>>,
) -> SearchResult<BatchIndexingResult> {
    engine.batch_index_sessions(sessions, progress_callback).await
}