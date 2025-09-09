//! Clean integration API for search engines and TUI components

use crate::{
    error::{ClaudeSessionError, Result, BatchParsingResult},
    models::*,
    parser::{SessionParser, ErrorRecoverySettings, ExtractionConfig},
    extractor::{BlockExtractor, ExtractionStats},
    insights::InsightsAnalyzer,
};
use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// High-level API for Claude session parsing and analysis
pub struct ClaudeSessionApi {
    parser: SessionParser,
    extractor: BlockExtractor,
    analyzer: InsightsAnalyzer,
    cache: Arc<RwLock<SessionCache>>,
    config: ApiConfig,
}

/// Configuration for the Claude Session API
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub enable_caching: bool,
    pub max_cache_size: usize,
    pub enable_background_analysis: bool,
    pub performance_monitoring: bool,
    pub auto_insights_extraction: bool,
}

/// In-memory cache for parsed sessions
#[derive(Debug, Default)]
struct SessionCache {
    sessions: HashMap<String, Arc<Session>>,
    insights: HashMap<String, Arc<SessionInsights>>,
    access_count: HashMap<String, usize>,
}

/// Search interface for finding sessions and content
pub struct SessionSearch {
    sessions: Vec<Arc<Session>>,
    indexed_content: HashMap<String, Vec<usize>>, // token -> session indices
}

/// Aggregated statistics across multiple sessions
#[derive(Debug, Clone)]
pub struct AggregateStats {
    pub total_sessions: usize,
    pub total_blocks: usize,
    pub total_words: usize,
    pub programming_languages: HashMap<ProgrammingLanguage, usize>,
    pub common_topics: Vec<Topic>,
    pub average_session_duration: chrono::Duration,
    pub collaboration_patterns: CollaborationSummary,
    pub productivity_trends: ProductivityTrends,
}

/// Collaboration summary across sessions
#[derive(Debug, Clone)]
pub struct CollaborationSummary {
    pub most_common_interaction_style: InteractionStyle,
    pub average_iterative_cycles: f64,
    pub common_question_patterns: Vec<String>,
    pub knowledge_transfer_trend: f64,
}

/// Productivity trends analysis
#[derive(Debug, Clone)]
pub struct ProductivityTrends {
    pub tasks_per_session: f64,
    pub problems_per_session: f64,
    pub average_resolution_time: chrono::Duration,
    pub code_quality_trend: f64,
    pub efficiency_improvement: f64,
}

/// Search query builder for finding specific content
#[derive(Debug, Default)]
pub struct SearchQuery {
    pub text_contains: Vec<String>,
    pub programming_languages: Vec<ProgrammingLanguage>,
    pub roles: Vec<Role>,
    pub date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub topics: Vec<String>,
    pub has_code_blocks: Option<bool>,
    pub has_errors: Option<bool>,
    pub complexity_range: Option<(f64, f64)>,
    pub tool_usage: Vec<String>,
}

/// Search results with relevance scoring
#[derive(Debug)]
pub struct SearchResults {
    pub sessions: Vec<SessionMatch>,
    pub blocks: Vec<BlockMatch>,
    pub total_matches: usize,
    pub search_time_ms: u64,
}

/// Individual session match with relevance
#[derive(Debug)]
pub struct SessionMatch {
    pub session: Arc<Session>,
    pub relevance_score: f64,
    pub matching_blocks: Vec<usize>,
    pub match_reasons: Vec<String>,
}

/// Individual block match with context
#[derive(Debug)]
pub struct BlockMatch {
    pub session_id: uuid::Uuid,
    pub block: Block,
    pub relevance_score: f64,
    pub highlighted_content: String,
    pub context_blocks: Vec<Block>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_cache_size: 1000,
            enable_background_analysis: true,
            performance_monitoring: true,
            auto_insights_extraction: true,
        }
    }
}

impl ClaudeSessionApi {
    /// Create a new API instance with default configuration
    pub fn new() -> Self {
        Self::with_config(ApiConfig::default())
    }

    /// Create API with custom configuration
    pub fn with_config(config: ApiConfig) -> Self {
        let parser = SessionParser::new();
        let extractor = BlockExtractor::new();
        let analyzer = InsightsAnalyzer::new();
        let cache = Arc::new(RwLock::new(SessionCache::default()));

        Self {
            parser,
            extractor,
            analyzer,
            cache,
            config,
        }
    }

    /// Parse a single session file with full analysis
    pub async fn parse_session_file<P: AsRef<Path>>(&self, path: P) -> Result<Arc<Session>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Check cache first
        if self.config.enable_caching {
            let cache = self.cache.read().await;
            if let Some(cached_session) = cache.sessions.get(&path_str) {
                let session_clone = cached_session.clone();
                // Update access count
                drop(cache);
                let mut cache_write = self.cache.write().await;
                *cache_write.access_count.entry(path_str.clone()).or_insert(0) += 1;
                info!("Returning cached session for {}", path_str);
                return Ok(session_clone);
            }
        }

        // Parse the session
        info!("Parsing session file: {}", path_str);
        let mut session = self.parser.parse_file(&path).await?;

        // Extract insights if enabled
        if self.config.auto_insights_extraction {
            session.insights = self.analyzer.analyze_session(&session).await?;
        }

        let arc_session = Arc::new(session);

        // Cache the result
        if self.config.enable_caching {
            let mut cache = self.cache.write().await;
            cache.sessions.insert(path_str.clone(), arc_session.clone());
            cache.access_count.insert(path_str, 1);
            
            // Evict oldest entries if cache is full
            if cache.sessions.len() > self.config.max_cache_size {
                self.evict_least_used_session(&mut cache).await;
            }
        }

        Ok(arc_session)
    }

    /// Parse multiple session files with comprehensive error reporting
    pub async fn parse_session_files<P: AsRef<Path> + Send + 'static>(&self, paths: Vec<P>) -> BatchParsingResult<Arc<Session>> {
        info!("Starting batch parsing of {} files", paths.len());
        
        let parser_result = self.parser.parse_files_with_error_reporting(paths).await;
        let mut api_result = BatchParsingResult::new();
        
        // Convert sessions to Arc and add insights
        for session in parser_result.successful {
            let mut enhanced_session = session;
            
            if self.config.auto_insights_extraction {
                match self.analyzer.analyze_session(&enhanced_session).await {
                    Ok(insights) => enhanced_session.insights = insights,
                    Err(e) => warn!("Failed to extract insights: {}", e),
                }
            }
            
            api_result.successful.push(Arc::new(enhanced_session));
        }
        
        api_result.failed = parser_result.failed;
        api_result.performance_stats = parser_result.performance_stats;
        
        info!("Batch parsing completed with {:.1}% success rate", api_result.success_rate() * 100.0);
        api_result
    }

    /// Parse all sessions in a directory
    pub async fn parse_directory<P: AsRef<Path>>(&self, dir_path: P) -> BatchParsingResult<Arc<Session>> {
        let sessions = match self.parser.parse_directory(dir_path).await {
            Ok(sessions) => sessions,
            Err(e) => {
                let mut result = BatchParsingResult::new();
                result.failed.push(crate::error::ErrorContext::new(
                    "directory_parse",
                    None,
                    crate::error::ErrorSeverity::Critical,
                    e.to_string(),
                    false,
                ));
                return result;
            }
        };

        let mut result = BatchParsingResult::new();
        
        for session in sessions {
            let mut enhanced_session = session;
            
            if self.config.auto_insights_extraction {
                match self.analyzer.analyze_session(&enhanced_session).await {
                    Ok(insights) => enhanced_session.insights = insights,
                    Err(e) => warn!("Failed to extract insights: {}", e),
                }
            }
            
            result.successful.push(Arc::new(enhanced_session));
        }

        result
    }

    /// Create a search interface from loaded sessions
    pub fn create_search_interface(&self, sessions: Vec<Arc<Session>>) -> SessionSearch {
        SessionSearch::new(sessions)
    }

    /// Calculate aggregate statistics across multiple sessions
    pub async fn calculate_aggregate_stats(&self, sessions: &[Arc<Session>]) -> Result<AggregateStats> {
        if sessions.is_empty() {
            return Err(ClaudeSessionError::invalid_format("No sessions provided for analysis"));
        }

        let total_sessions = sessions.len();
        let total_blocks = sessions.iter().map(|s| s.blocks.len()).sum();
        let total_words = sessions.iter().map(|s| s.statistics.total_words).sum();

        // Aggregate programming languages
        let mut programming_languages = HashMap::new();
        for session in sessions {
            for block in &session.blocks {
                for code_block in &block.content.code_blocks {
                    if let Some(lang) = &code_block.language {
                        *programming_languages.entry(lang.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Find common topics
        let mut topic_frequency = HashMap::new();
        for session in sessions {
            for topic in &session.insights.primary_topics {
                *topic_frequency.entry(topic.name.clone()).or_insert(0) += topic.mentions;
            }
        }
        
        let mut common_topics: Vec<_> = topic_frequency.into_iter()
            .map(|(name, mentions)| Topic {
                name,
                relevance_score: mentions as f64 / total_sessions as f64,
                mentions,
                subtopics: Vec::new(),
                related_tools: Vec::new(),
            })
            .collect();
        common_topics.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        common_topics.truncate(10);

        // Calculate average session duration
        let durations: Vec<_> = sessions.iter()
            .filter_map(|s| s.duration())
            .collect();
        let average_session_duration = if durations.is_empty() {
            chrono::Duration::zero()
        } else {
            durations.iter().sum::<chrono::Duration>() / durations.len() as i32
        };

        // Analyze collaboration patterns
        let collaboration_patterns = self.analyze_aggregate_collaboration(sessions).await?;
        let productivity_trends = self.analyze_productivity_trends(sessions).await?;

        Ok(AggregateStats {
            total_sessions,
            total_blocks,
            total_words,
            programming_languages,
            common_topics,
            average_session_duration,
            collaboration_patterns,
            productivity_trends,
        })
    }

    /// Export sessions to various formats
    pub async fn export_sessions(&self, sessions: &[Arc<Session>], format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Json => {
                let session_refs: Vec<&Session> = sessions.iter().map(|s| s.as_ref()).collect();
                serde_json::to_string_pretty(&session_refs)
                    .map_err(|e| ClaudeSessionError::invalid_format(format!("JSON export failed: {}", e)))
            },
            ExportFormat::Csv => self.export_to_csv(sessions).await,
            ExportFormat::Markdown => self.export_to_markdown(sessions).await,
        }
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            total_entries: cache.sessions.len(),
            total_accesses: cache.access_count.values().sum(),
            cache_hits: cache.access_count.len(),
            memory_usage_estimate: cache.sessions.len() * 1000, // Rough estimate
        }
    }

    /// Clear the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.sessions.clear();
        cache.insights.clear();
        cache.access_count.clear();
        info!("Session cache cleared");
    }

    // Private helper methods

    async fn evict_least_used_session(&self, cache: &mut SessionCache) {
        if let Some((path, _)) = cache.access_count.iter().min_by_key(|(_, &count)| count) {
            let path = path.clone();
            cache.sessions.remove(&path);
            cache.access_count.remove(&path);
            cache.insights.remove(&path);
        }
    }

    async fn analyze_aggregate_collaboration(&self, sessions: &[Arc<Session>]) -> Result<CollaborationSummary> {
        let mut interaction_styles = HashMap::new();
        let mut total_cycles = 0;
        let mut question_patterns = HashMap::new();
        let mut knowledge_transfers = Vec::new();

        for session in sessions {
            let patterns = &session.insights.collaboration_patterns;
            *interaction_styles.entry(patterns.interaction_style).or_insert(0) += 1;
            total_cycles += patterns.iterative_cycles;
            
            for (pattern, count) in &patterns.question_types {
                *question_patterns.entry(pattern.clone()).or_insert(0) += count;
            }
            
            knowledge_transfers.push(patterns.knowledge_transfer);
        }

        let most_common_interaction_style = interaction_styles.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(style, _)| style)
            .unwrap_or(InteractionStyle::TaskOriented);

        let average_iterative_cycles = total_cycles as f64 / sessions.len() as f64;
        
        let common_question_patterns = question_patterns.into_iter()
            .map(|(pattern, _)| pattern)
            .collect();

        let knowledge_transfer_trend = knowledge_transfers.iter().sum::<f64>() / knowledge_transfers.len() as f64;

        Ok(CollaborationSummary {
            most_common_interaction_style,
            average_iterative_cycles,
            common_question_patterns,
            knowledge_transfer_trend,
        })
    }

    async fn analyze_productivity_trends(&self, sessions: &[Arc<Session>]) -> Result<ProductivityTrends> {
        let mut total_tasks = 0;
        let mut total_problems = 0;
        let mut resolution_times = Vec::new();
        let mut quality_scores = Vec::new();
        let mut efficiency_scores = Vec::new();

        for session in sessions {
            let metrics = &session.insights.productivity_metrics;
            total_tasks += metrics.tasks_completed;
            total_problems += metrics.problems_solved;
            resolution_times.extend(metrics.time_to_resolution.iter().cloned());
            quality_scores.push(metrics.code_quality_score);
            efficiency_scores.push(metrics.efficiency_rating);
        }

        let tasks_per_session = total_tasks as f64 / sessions.len() as f64;
        let problems_per_session = total_problems as f64 / sessions.len() as f64;
        
        let average_resolution_time = if resolution_times.is_empty() {
            chrono::Duration::zero()
        } else {
            resolution_times.iter().sum::<chrono::Duration>() / resolution_times.len() as i32
        };

        let code_quality_trend = quality_scores.iter().sum::<f64>() / quality_scores.len() as f64;
        let efficiency_improvement = efficiency_scores.iter().sum::<f64>() / efficiency_scores.len() as f64;

        Ok(ProductivityTrends {
            tasks_per_session,
            problems_per_session,
            average_resolution_time,
            code_quality_trend,
            efficiency_improvement,
        })
    }

    async fn export_to_csv(&self, sessions: &[Arc<Session>]) -> Result<String> {
        let mut csv = String::from("id,file_path,created_at,total_blocks,total_words,session_duration,primary_topics\n");
        
        for session in sessions {
            let duration_minutes = session.duration()
                .map(|d| d.num_minutes())
                .unwrap_or(0);
            
            let topics = session.insights.primary_topics.iter()
                .take(3)
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join(";");

            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                session.id,
                session.metadata.file_path,
                session.metadata.created_at.format("%Y-%m-%d %H:%M:%S"),
                session.statistics.total_blocks,
                session.statistics.total_words,
                duration_minutes,
                topics
            ));
        }

        Ok(csv)
    }

    async fn export_to_markdown(&self, sessions: &[Arc<Session>]) -> Result<String> {
        let mut markdown = String::from("# Claude Sessions Analysis\n\n");
        
        for session in sessions {
            markdown.push_str(&format!("## Session: {}\n\n", session.metadata.file_path));
            markdown.push_str(&format!("- **Created:** {}\n", session.metadata.created_at.format("%Y-%m-%d %H:%M:%S")));
            markdown.push_str(&format!("- **Blocks:** {}\n", session.statistics.total_blocks));
            markdown.push_str(&format!("- **Words:** {}\n", session.statistics.total_words));
            
            if let Some(duration) = session.duration() {
                markdown.push_str(&format!("- **Duration:** {} minutes\n", duration.num_minutes()));
            }
            
            markdown.push_str("\n### Primary Topics\n\n");
            for topic in &session.insights.primary_topics {
                markdown.push_str(&format!("- **{}** (relevance: {:.2}, mentions: {})\n", 
                    topic.name, topic.relevance_score, topic.mentions));
            }
            
            markdown.push_str("\n---\n\n");
        }

        Ok(markdown)
    }
}

impl Default for ClaudeSessionApi {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionSearch {
    /// Create a new search interface
    pub fn new(sessions: Vec<Arc<Session>>) -> Self {
        let mut search = Self {
            sessions,
            indexed_content: HashMap::new(),
        };
        
        search.build_index();
        search
    }

    /// Build search index for fast content lookup
    fn build_index(&mut self) {
        for (session_idx, session) in self.sessions.iter().enumerate() {
            for block in &session.blocks {
                // Index content tokens
                for token in &block.content.tokens {
                    let normalized_token = token.text.to_lowercase();
                    self.indexed_content.entry(normalized_token)
                        .or_default()
                        .push(session_idx);
                }
                
                // Index topics
                for topic in &block.metadata.topics {
                    let normalized_topic = topic.to_lowercase();
                    self.indexed_content.entry(normalized_topic)
                        .or_default()
                        .push(session_idx);
                }
            }
        }
    }

    /// Search for sessions matching the query
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResults> {
        let start_time = std::time::Instant::now();
        let mut session_matches = Vec::new();
        let mut block_matches = Vec::new();

        for (session_idx, session) in self.sessions.iter().enumerate() {
            let session_match = self.evaluate_session_match(session, &query).await?;
            
            if session_match.relevance_score > 0.0 {
                // Find matching blocks
                for (block_idx, block) in session.blocks.iter().enumerate() {
                    if self.block_matches_query(block, &query) {
                        let block_match = BlockMatch {
                            session_id: session.id,
                            block: block.clone(),
                            relevance_score: self.calculate_block_relevance(block, &query),
                            highlighted_content: self.highlight_matches(&block.content.raw_text, &query.text_contains),
                            context_blocks: self.get_context_blocks(session, block_idx, 2),
                        };
                        block_matches.push(block_match);
                    }
                }
                
                session_matches.push(session_match);
            }
        }

        // Sort by relevance
        session_matches.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        block_matches.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        let total_matches = session_matches.len() + block_matches.len();
        let search_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(SearchResults {
            sessions: session_matches,
            blocks: block_matches,
            total_matches,
            search_time_ms,
        })
    }

    // Helper methods for search functionality

    async fn evaluate_session_match(&self, session: &Session, query: &SearchQuery) -> Result<SessionMatch> {
        let mut relevance_score = 0.0;
        let mut matching_blocks = Vec::new();
        let mut match_reasons = Vec::new();

        // Check text content matches
        for text in &query.text_contains {
            let text_lower = text.to_lowercase();
            for (block_idx, block) in session.blocks.iter().enumerate() {
                if block.content.raw_text.to_lowercase().contains(&text_lower) {
                    relevance_score += 1.0;
                    matching_blocks.push(block_idx);
                    match_reasons.push(format!("Contains text: '{}'", text));
                }
            }
        }

        // Check programming language matches
        if !query.programming_languages.is_empty() {
            for block in &session.blocks {
                for code_block in &block.content.code_blocks {
                    if let Some(lang) = &code_block.language {
                        if query.programming_languages.contains(lang) {
                            relevance_score += 2.0;
                            match_reasons.push(format!("Contains {} code", format!("{:?}", lang)));
                        }
                    }
                }
            }
        }

        // Check topic matches
        for topic in &query.topics {
            let topic_lower = topic.to_lowercase();
            for session_topic in &session.insights.primary_topics {
                if session_topic.name.to_lowercase().contains(&topic_lower) {
                    relevance_score += session_topic.relevance_score;
                    match_reasons.push(format!("Topic match: '{}'", topic));
                }
            }
        }

        // Check date range
        if let Some((start, end)) = query.date_range {
            if session.metadata.created_at >= start && session.metadata.created_at <= end {
                relevance_score += 0.5;
                match_reasons.push("Within date range".to_string());
            }
        }

        Ok(SessionMatch {
            session: Arc::new(session.clone()),
            relevance_score,
            matching_blocks,
            match_reasons,
        })
    }

    fn block_matches_query(&self, block: &Block, query: &SearchQuery) -> bool {
        // Check role filter
        if !query.roles.is_empty() && !query.roles.contains(&block.role) {
            return false;
        }

        // Check complexity range
        if let Some((min, max)) = query.complexity_range {
            if let Some(complexity) = block.metadata.complexity_score {
                if complexity < min || complexity > max {
                    return false;
                }
            }
        }

        // Check code blocks requirement
        if let Some(has_code) = query.has_code_blocks {
            if (block.content.code_blocks.is_empty()) == has_code {
                return false;
            }
        }

        true
    }

    fn calculate_block_relevance(&self, block: &Block, query: &SearchQuery) -> f64 {
        let mut relevance = 0.0;

        // Text matches
        for text in &query.text_contains {
            if block.content.raw_text.to_lowercase().contains(&text.to_lowercase()) {
                relevance += 1.0;
            }
        }

        // Programming language matches
        for code_block in &block.content.code_blocks {
            if let Some(lang) = &code_block.language {
                if query.programming_languages.contains(lang) {
                    relevance += 2.0;
                }
            }
        }

        // Boost for assistant responses with solutions
        if block.role == Role::Assistant && block.content.raw_text.contains("solution") {
            relevance += 1.0;
        }

        relevance
    }

    fn highlight_matches(&self, text: &str, search_terms: &[String]) -> String {
        let mut highlighted = text.to_string();
        
        for term in search_terms {
            highlighted = highlighted.replace(term, &format!("**{}**", term));
        }
        
        highlighted
    }

    fn get_context_blocks(&self, session: &Session, block_idx: usize, context_size: usize) -> Vec<Block> {
        let start = block_idx.saturating_sub(context_size);
        let end = (block_idx + context_size + 1).min(session.blocks.len());
        
        session.blocks[start..end].to_vec()
    }
}

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Markdown,
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_accesses: usize,
    pub cache_hits: usize,
    pub memory_usage_estimate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_api_creation() {
        let api = ClaudeSessionApi::new();
        let stats = api.get_cache_stats().await;
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_search_interface() {
        let sessions = vec![]; // Empty for testing
        let search = SessionSearch::new(sessions);
        
        let query = SearchQuery {
            text_contains: vec!["test".to_string()],
            ..Default::default()
        };
        
        let results = search.search(query).await.unwrap();
        assert_eq!(results.total_matches, 0);
    }

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery {
            text_contains: vec!["rust".to_string()],
            programming_languages: vec![ProgrammingLanguage::Rust],
            has_code_blocks: Some(true),
            ..Default::default()
        };
        
        assert_eq!(query.text_contains.len(), 1);
        assert!(query.has_code_blocks.unwrap());
    }
}