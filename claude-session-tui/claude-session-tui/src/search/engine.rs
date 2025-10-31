//! Core search engine implementation with Tantivy and advanced resilience patterns

use crate::{Session, Block, Role};
use super::{SearchError, SearchResult, CircuitBreaker, ErrorContext, PerformanceImpact, UserImpact};
use tantivy::{
    schema::{Schema, Field, FieldType, TextOptions, IndexRecordOption, TextFieldIndexing},
    Index, IndexReader, IndexWriter, Document, Term, query::{Query, QueryParser},
    collector::{TopDocs, FacetCollector, Count}, 
    Searcher, TantivyError, ReloadPolicy, tokenizer::TextAnalyzer,
};
use parking_lot::{RwLock, Mutex};
use dashmap::DashMap;
use lru::LruCache;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}},
    time::{Duration, Instant},
    collections::HashMap,
};
use tracing::{info, warn, error, debug, instrument};
use tokio::{
    sync::{Semaphore, RwLock as AsyncRwLock},
    time::timeout,
};
use crossbeam::channel::{unbounded, Receiver, Sender};
use uuid::Uuid;

/// High-performance search engine with fault tolerance and resilience
pub struct SearchEngine {
    /// Tantivy index for full-text search
    index: Arc<Index>,
    /// Index reader with automatic reloading
    reader: Arc<IndexReader>,
    /// Index writer with concurrent protection
    writer: Arc<AsyncRwLock<IndexWriter>>,
    /// Query parser for complex search queries
    query_parser: QueryParser,
    /// Schema definition for document structure
    schema: Schema,
    /// Schema field mappings
    fields: SchemaFields,
    /// Search result cache with LRU eviction
    cache: Arc<Mutex<LruCache<String, CachedSearchResult>>>,
    /// Circuit breaker for fault tolerance
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    /// Performance monitoring
    performance_monitor: Arc<PerformanceMonitor>,
    /// Configuration settings
    config: SearchConfig,
    /// Background task handles
    background_tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    /// Concurrent search limiter
    search_semaphore: Arc<Semaphore>,
    /// Index health monitor
    health_monitor: Arc<IndexHealthMonitor>,
}

/// Schema field definitions for structured search
#[derive(Debug, Clone)]
pub struct SchemaFields {
    pub content: Field,
    pub role: Field,
    pub timestamp: Field,
    pub session_id: Field,
    pub block_id: Field,
    pub tool_usage: Field,
    pub code_blocks: Field,
    pub file_paths: Field,
    pub topics: Field,
    pub sentiment: Field,
    pub complexity_score: Field,
    pub project_path: Field,
    pub language: Field,
    pub intent: Field,
    pub word_count: Field,
    pub error_patterns: Field,
    pub solution_patterns: Field,
}

/// Search configuration with performance tuning
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub cache_size: usize,
    pub max_concurrent_searches: usize,
    pub query_timeout: Duration,
    pub index_commit_interval: Duration,
    pub circuit_breaker_failure_threshold: usize,
    pub circuit_breaker_timeout: Duration,
    pub memory_limit_mb: usize,
    pub auto_optimize_interval: Duration,
    pub health_check_interval: Duration,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            cache_size: 10_000,
            max_concurrent_searches: 100,
            query_timeout: Duration::from_secs(10),
            index_commit_interval: Duration::from_secs(30),
            circuit_breaker_failure_threshold: 10,
            circuit_breaker_timeout: Duration::from_secs(60),
            memory_limit_mb: 512,
            auto_optimize_interval: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Performance monitoring and metrics
#[derive(Debug)]
pub struct PerformanceMonitor {
    pub total_searches: AtomicUsize,
    pub successful_searches: AtomicUsize,
    pub failed_searches: AtomicUsize,
    pub cache_hits: AtomicUsize,
    pub cache_misses: AtomicUsize,
    pub average_search_time_ms: AtomicUsize,
    pub documents_indexed: AtomicUsize,
    pub indexing_errors: AtomicUsize,
    pub memory_usage_mb: AtomicUsize,
    pub last_health_check: Arc<Mutex<Option<Instant>>>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            total_searches: AtomicUsize::new(0),
            successful_searches: AtomicUsize::new(0),
            failed_searches: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
            average_search_time_ms: AtomicUsize::new(0),
            documents_indexed: AtomicUsize::new(0),
            indexing_errors: AtomicUsize::new(0),
            memory_usage_mb: AtomicUsize::new(0),
            last_health_check: Arc::new(Mutex::new(None)),
        }
    }
}

/// Index health monitoring
#[derive(Debug)]
pub struct IndexHealthMonitor {
    pub is_healthy: AtomicBool,
    pub last_check: Arc<Mutex<Option<Instant>>>,
    pub corruption_detected: AtomicBool,
    pub disk_space_critical: AtomicBool,
    pub memory_pressure: AtomicBool,
}

impl Default for IndexHealthMonitor {
    fn default() -> Self {
        Self {
            is_healthy: AtomicBool::new(true),
            last_check: Arc::new(Mutex::new(None)),
            corruption_detected: AtomicBool::new(false),
            disk_space_critical: AtomicBool::new(false),
            memory_pressure: AtomicBool::new(false),
        }
    }
}

/// Cached search result with metadata
#[derive(Debug, Clone)]
pub struct CachedSearchResult {
    pub results: Vec<BlockSearchResult>,
    pub created_at: Instant,
    pub hit_count: usize,
    pub query_time_ms: u64,
}

/// Search result for a conversation block
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockSearchResult {
    pub block_id: Uuid,
    pub session_id: Uuid,
    pub content: String,
    pub role: Role,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub relevance_score: f32,
    pub highlights: Vec<TextHighlight>,
    pub context: SearchContext,
    pub metadata: BlockMetadata,
}

/// Text highlighting for search results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TextHighlight {
    pub field: String,
    pub fragments: Vec<String>,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// Search context information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchContext {
    pub surrounding_blocks: Vec<Uuid>,
    pub project_path: Option<String>,
    pub file_context: Vec<String>,
    pub topic_relevance: HashMap<String, f32>,
}

/// Block metadata for search results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockMetadata {
    pub word_count: usize,
    pub code_blocks: usize,
    pub tool_usage: bool,
    pub sentiment: Option<String>,
    pub complexity_score: Option<f32>,
    pub intent: Option<String>,
}

/// Indexing statistics
#[derive(Debug, Clone)]
pub struct IndexingStats {
    pub documents_processed: usize,
    pub documents_indexed: usize,
    pub processing_time_ms: u64,
    pub errors: Vec<String>,
    pub memory_usage_mb: usize,
}

/// Batch indexing result
#[derive(Debug)]
pub struct BatchIndexingResult {
    pub total_sessions: usize,
    pub successful_sessions: usize,
    pub failed_sessions: usize,
    pub total_blocks: usize,
    pub processing_time_ms: u64,
    pub errors: Vec<String>,
}

impl SearchEngine {
    /// Create a new search engine instance
    #[instrument(skip(index_path))]
    pub async fn new<P: AsRef<Path>>(index_path: P) -> SearchResult<Self> {
        let index_path = index_path.as_ref();
        let config = SearchConfig::default();

        // Create schema with comprehensive field definitions
        let schema = Self::build_schema()?;
        let fields = Self::extract_fields(&schema)?;

        // Initialize or open existing index
        let index = if index_path.exists() {
            Index::open_in_dir(index_path)?
        } else {
            std::fs::create_dir_all(index_path)?;
            Index::create_in_dir(index_path, schema.clone())?
        };

        let index = Arc::new(index);

        // Create index reader with hot reload
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;
        let reader = Arc::new(reader);

        // Create index writer with memory buffer
        let writer = index.writer(128_000_000)?; // 128MB buffer
        let writer = Arc::new(AsyncRwLock::new(writer));

        // Create query parser with all searchable fields
        let query_parser = Self::build_query_parser(&schema, &fields);

        // Initialize components
        let cache = Arc::new(Mutex::new(LruCache::new(
            std::num::NonZeroUsize::new(config.cache_size).unwrap()
        )));

        let circuit_breaker = Arc::new(Mutex::new(CircuitBreaker::new(
            config.circuit_breaker_failure_threshold,
            config.circuit_breaker_timeout,
            5, // Success threshold for half-open -> closed
        )));

        let performance_monitor = Arc::new(PerformanceMonitor::default());
        let health_monitor = Arc::new(IndexHealthMonitor::default());

        let search_semaphore = Arc::new(Semaphore::new(config.max_concurrent_searches));

        let engine = Self {
            index,
            reader,
            writer,
            query_parser,
            schema,
            fields,
            cache,
            circuit_breaker,
            performance_monitor,
            config: config.clone(),
            background_tasks: Arc::new(Mutex::new(Vec::new())),
            search_semaphore,
            health_monitor,
        };

        // Start background tasks
        engine.start_background_tasks().await?;

        info!("Search engine initialized successfully at {:?}", index_path);
        Ok(engine)
    }

    /// Build the Tantivy schema for conversation blocks
    fn build_schema() -> SearchResult<Schema> {
        let mut schema_builder = tantivy::schema::Schema::builder();

        // Full-text search fields
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("en_stem")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let content = schema_builder.add_text_field("content", text_options.clone());
        let code_blocks = schema_builder.add_text_field("code_blocks", text_options.clone());
        let file_paths = schema_builder.add_text_field("file_paths", text_options.clone());
        let topics = schema_builder.add_text_field("topics", text_options.clone());
        let error_patterns = schema_builder.add_text_field("error_patterns", text_options.clone());
        let solution_patterns = schema_builder.add_text_field("solution_patterns", text_options.clone());

        // Faceted fields for filtering
        let role = schema_builder.add_facet_field("role", tantivy::schema::FacetOptions::default());
        let tool_usage = schema_builder.add_facet_field("tool_usage", tantivy::schema::FacetOptions::default());
        let language = schema_builder.add_facet_field("language", tantivy::schema::FacetOptions::default());
        let sentiment = schema_builder.add_facet_field("sentiment", tantivy::schema::FacetOptions::default());
        let intent = schema_builder.add_facet_field("intent", tantivy::schema::FacetOptions::default());

        // Date fields
        let timestamp = schema_builder.add_date_field("timestamp", tantivy::schema::DateOptions::default().set_stored().set_indexed());

        // Numeric fields
        let complexity_score = schema_builder.add_f64_field("complexity_score", tantivy::schema::NumericOptions::default().set_stored().set_indexed());
        let word_count = schema_builder.add_u64_field("word_count", tantivy::schema::NumericOptions::default().set_stored().set_indexed());

        // ID fields
        let session_id = schema_builder.add_text_field("session_id", tantivy::schema::TextOptions::default().set_stored().set_indexing_options(
            TextFieldIndexing::default().set_tokenizer("raw")
        ));
        let block_id = schema_builder.add_text_field("block_id", tantivy::schema::TextOptions::default().set_stored().set_indexing_options(
            TextFieldIndexing::default().set_tokenizer("raw")
        ));

        // Project context
        let project_path = schema_builder.add_text_field("project_path", text_options);

        Ok(schema_builder.build())
    }

    /// Extract field references from schema
    fn extract_fields(schema: &Schema) -> SearchResult<SchemaFields> {
        Ok(SchemaFields {
            content: schema.get_field("content")
                .ok_or_else(|| SearchError::index_error("Content field not found in schema", false))?,
            role: schema.get_field("role")
                .ok_or_else(|| SearchError::index_error("Role field not found in schema", false))?,
            timestamp: schema.get_field("timestamp")
                .ok_or_else(|| SearchError::index_error("Timestamp field not found in schema", false))?,
            session_id: schema.get_field("session_id")
                .ok_or_else(|| SearchError::index_error("Session ID field not found in schema", false))?,
            block_id: schema.get_field("block_id")
                .ok_or_else(|| SearchError::index_error("Block ID field not found in schema", false))?,
            tool_usage: schema.get_field("tool_usage")
                .ok_or_else(|| SearchError::index_error("Tool usage field not found in schema", false))?,
            code_blocks: schema.get_field("code_blocks")
                .ok_or_else(|| SearchError::index_error("Code blocks field not found in schema", false))?,
            file_paths: schema.get_field("file_paths")
                .ok_or_else(|| SearchError::index_error("File paths field not found in schema", false))?,
            topics: schema.get_field("topics")
                .ok_or_else(|| SearchError::index_error("Topics field not found in schema", false))?,
            sentiment: schema.get_field("sentiment")
                .ok_or_else(|| SearchError::index_error("Sentiment field not found in schema", false))?,
            complexity_score: schema.get_field("complexity_score")
                .ok_or_else(|| SearchError::index_error("Complexity score field not found in schema", false))?,
            project_path: schema.get_field("project_path")
                .ok_or_else(|| SearchError::index_error("Project path field not found in schema", false))?,
            language: schema.get_field("language")
                .ok_or_else(|| SearchError::index_error("Language field not found in schema", false))?,
            intent: schema.get_field("intent")
                .ok_or_else(|| SearchError::index_error("Intent field not found in schema", false))?,
            word_count: schema.get_field("word_count")
                .ok_or_else(|| SearchError::index_error("Word count field not found in schema", false))?,
            error_patterns: schema.get_field("error_patterns")
                .ok_or_else(|| SearchError::index_error("Error patterns field not found in schema", false))?,
            solution_patterns: schema.get_field("solution_patterns")
                .ok_or_else(|| SearchError::index_error("Solution patterns field not found in schema", false))?,
        })
    }

    /// Build query parser with all searchable fields
    fn build_query_parser(schema: &Schema, fields: &SchemaFields) -> QueryParser {
        QueryParser::for_index(
            &Index::create_in_ram(schema.clone()),
            vec![
                fields.content,
                fields.code_blocks,
                fields.file_paths,
                fields.topics,
                fields.error_patterns,
                fields.solution_patterns,
            ],
        )
    }

    /// Start background maintenance tasks
    async fn start_background_tasks(&self) -> SearchResult<()> {
        let mut tasks = self.background_tasks.lock();

        // Health monitoring task
        let health_monitor = Arc::clone(&self.health_monitor);
        let health_check_interval = self.config.health_check_interval;
        let health_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(health_check_interval);
            loop {
                interval.tick().await;
                Self::health_check_task(&health_monitor).await;
            }
        });
        tasks.push(health_task);

        // Auto-optimization task
        let index = Arc::clone(&self.index);
        let optimize_interval = self.config.auto_optimize_interval;
        let optimize_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(optimize_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::optimize_index_task(&index).await {
                    error!("Index optimization failed: {}", e);
                }
            }
        });
        tasks.push(optimize_task);

        info!("Background tasks started successfully");
        Ok(())
    }

    /// Health check background task
    async fn health_check_task(health_monitor: &IndexHealthMonitor) {
        let start = Instant::now();
        
        // Check memory usage
        let memory_usage = Self::get_memory_usage();
        health_monitor.memory_pressure.store(memory_usage > 400, Ordering::Relaxed);

        // Check disk space
        let disk_space_critical = Self::check_disk_space().unwrap_or(false);
        health_monitor.disk_space_critical.store(disk_space_critical, Ordering::Relaxed);

        // Update health status
        let is_healthy = !health_monitor.memory_pressure.load(Ordering::Relaxed) 
            && !health_monitor.disk_space_critical.load(Ordering::Relaxed)
            && !health_monitor.corruption_detected.load(Ordering::Relaxed);
        
        health_monitor.is_healthy.store(is_healthy, Ordering::Relaxed);
        *health_monitor.last_check.lock() = Some(start);

        debug!("Health check completed in {:?}", start.elapsed());
    }

    /// Index optimization background task
    async fn optimize_index_task(index: &Index) -> SearchResult<()> {
        let writer = index.writer(50_000_000)?; // 50MB buffer for optimization
        writer.wait_merging_threads()?;
        Ok(())
    }

    /// Get current memory usage in MB
    fn get_memory_usage() -> usize {
        // Simplified memory usage estimation
        // In production, use a proper memory profiler
        std::process::id() as usize % 512 // Mock implementation
    }

    /// Check if disk space is critical
    fn check_disk_space() -> SearchResult<bool> {
        // Simplified disk space check
        // In production, check actual available space
        Ok(false) // Mock implementation
    }

    /// Check if the search engine is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_monitor.is_healthy.load(Ordering::Relaxed)
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            total_searches: self.performance_monitor.total_searches.load(Ordering::Relaxed),
            successful_searches: self.performance_monitor.successful_searches.load(Ordering::Relaxed),
            failed_searches: self.performance_monitor.failed_searches.load(Ordering::Relaxed),
            cache_hits: self.performance_monitor.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.performance_monitor.cache_misses.load(Ordering::Relaxed),
            average_search_time_ms: self.performance_monitor.average_search_time_ms.load(Ordering::Relaxed),
            documents_indexed: self.performance_monitor.documents_indexed.load(Ordering::Relaxed),
            indexing_errors: self.performance_monitor.indexing_errors.load(Ordering::Relaxed),
            memory_usage_mb: self.performance_monitor.memory_usage_mb.load(Ordering::Relaxed),
        }
    }
}

/// Performance statistics snapshot
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_searches: usize,
    pub successful_searches: usize,
    pub failed_searches: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub average_search_time_ms: usize,
    pub documents_indexed: usize,
    pub indexing_errors: usize,
    pub memory_usage_mb: usize,
}

// Implementation continues in next part due to length constraints...