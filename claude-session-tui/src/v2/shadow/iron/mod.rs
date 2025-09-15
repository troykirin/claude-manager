//! Iron - Insights Optimization Shadow Agent
//!
//! Specializes in incremental computation, caching, and real-time insights
//! generation with advanced pattern recognition and performance optimization.

use crate::v2::core::traits::{
    InsightCategory, InsightData, InsightsEngine, PerformanceMetrics, ShadowAgent,
};
use crate::v2::shadow::beru::{BeruMessage, BeruSession};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Iron's advanced conversation insight
#[derive(Debug, Clone)]
pub struct IronInsight {
    pub session_id: String,
    pub category: InsightCategory,
    pub confidence: f32,
    pub summary: String,
    pub metadata: IronInsightMetadata,
    pub computed_at: chrono::DateTime<chrono::Utc>,
}

impl InsightData for IronInsight {
    fn category(&self) -> InsightCategory {
        self.category
    }

    fn confidence(&self) -> f32 {
        self.confidence
    }

    fn summary(&self) -> &str {
        &self.summary
    }
}

#[derive(Debug, Clone)]
pub struct IronInsightMetadata {
    pub computation_time_ms: u64,
    pub data_points: usize,
    pub patterns_found: Vec<String>,
    pub metrics: HashMap<String, f64>,
}

/// Iron's high-performance insights engine with caching
pub struct IronEngine {
    cache: Arc<RwLock<IronCache>>,
    pattern_analyzer: IronPatternAnalyzer,
    metrics_collector: IronMetricsCollector,
    incremental_state: Arc<RwLock<IronIncrementalState>>,
}

impl IronEngine {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(IronCache::new(1000))),
            pattern_analyzer: IronPatternAnalyzer::new(),
            metrics_collector: IronMetricsCollector::new(),
            incremental_state: Arc::new(RwLock::new(IronIncrementalState::new())),
        }
    }

    /// Analyze session with incremental computation
    pub async fn analyze_incremental(
        &self,
        session: &BeruSession,
    ) -> Result<Vec<IronInsight>, IronError> {
        let start_time = std::time::Instant::now();
        let session_id = session.uuid.clone();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached_insights) = cache.get(&session_id) {
                if !self
                    .needs_recomputation(session, &cached_insights.computed_at)
                    .await
                {
                    return Ok(cached_insights.insights.clone());
                }
            }
        }

        // Compute insights incrementally
        let mut insights = Vec::new();

        // Topic analysis
        if let Ok(topic_insight) = self.analyze_topics(session).await {
            insights.push(topic_insight);
        }

        // Sentiment analysis
        if let Ok(sentiment_insight) = self.analyze_sentiment(session).await {
            insights.push(sentiment_insight);
        }

        // Complexity analysis
        if let Ok(complexity_insight) = self.analyze_complexity(session).await {
            insights.push(complexity_insight);
        }

        // Tool usage patterns
        if let Ok(tool_insight) = self.analyze_tool_usage(session).await {
            insights.push(tool_insight);
        }

        // Performance metrics
        if let Ok(perf_insight) = self.analyze_performance(session).await {
            insights.push(perf_insight);
        }

        let computation_time = start_time.elapsed();

        // Cache results
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                session_id.clone(),
                CachedInsights {
                    insights: insights.clone(),
                    computed_at: chrono::Utc::now(),
                    computation_time,
                },
            );
        }

        // Update incremental state
        {
            let mut state = self.incremental_state.write().await;
            state.update_session_stats(session);
        }

        Ok(insights)
    }

    async fn analyze_topics(&self, session: &BeruSession) -> Result<IronInsight, IronError> {
        let topics = self
            .pattern_analyzer
            .extract_topics(&session.messages)
            .await?;
        let primary_topic = topics
            .first()
            .cloned()
            .unwrap_or_else(|| "General".to_string());

        Ok(IronInsight {
            session_id: session.uuid.clone(),
            category: InsightCategory::Topic,
            confidence: 0.85,
            summary: format!("Primary topic: {}", primary_topic),
            metadata: IronInsightMetadata {
                computation_time_ms: 10,
                data_points: session.messages.len(),
                patterns_found: topics,
                metrics: HashMap::from([
                    (
                        "topic_diversity".to_string(),
                        self.calculate_topic_diversity(&topics),
                    ),
                    (
                        "topic_consistency".to_string(),
                        self.calculate_topic_consistency(&topics),
                    ),
                ]),
            },
            computed_at: chrono::Utc::now(),
        })
    }

    async fn analyze_sentiment(&self, session: &BeruSession) -> Result<IronInsight, IronError> {
        let sentiment_scores = self
            .pattern_analyzer
            .analyze_sentiment(&session.messages)
            .await?;
        let avg_sentiment = sentiment_scores.iter().sum::<f32>() / sentiment_scores.len() as f32;

        let sentiment_label = match avg_sentiment {
            x if x > 0.5 => "Positive",
            x if x < -0.5 => "Negative",
            _ => "Neutral",
        };

        Ok(IronInsight {
            session_id: session.uuid.clone(),
            category: InsightCategory::Sentiment,
            confidence: 0.75,
            summary: format!(
                "Overall sentiment: {} ({:.2})",
                sentiment_label, avg_sentiment
            ),
            metadata: IronInsightMetadata {
                computation_time_ms: 15,
                data_points: sentiment_scores.len(),
                patterns_found: vec![sentiment_label.to_string()],
                metrics: HashMap::from([
                    ("avg_sentiment".to_string(), avg_sentiment as f64),
                    (
                        "sentiment_variance".to_string(),
                        self.calculate_variance(&sentiment_scores),
                    ),
                ]),
            },
            computed_at: chrono::Utc::now(),
        })
    }

    async fn analyze_complexity(&self, session: &BeruSession) -> Result<IronInsight, IronError> {
        let complexity_score = self
            .metrics_collector
            .calculate_complexity(&session.messages)
            .await?;

        let complexity_label = match complexity_score {
            x if x > 0.8 => "Very Complex",
            x if x > 0.6 => "Complex",
            x if x > 0.4 => "Moderate",
            x if x > 0.2 => "Simple",
            _ => "Very Simple",
        };

        Ok(IronInsight {
            session_id: session.uuid.clone(),
            category: InsightCategory::Complexity,
            confidence: 0.9,
            summary: format!(
                "Conversation complexity: {} ({:.2})",
                complexity_label, complexity_score
            ),
            metadata: IronInsightMetadata {
                computation_time_ms: 8,
                data_points: session.messages.len(),
                patterns_found: vec![complexity_label.to_string()],
                metrics: HashMap::from([
                    ("complexity_score".to_string(), complexity_score),
                    (
                        "avg_message_length".to_string(),
                        self.calculate_avg_message_length(&session.messages),
                    ),
                ]),
            },
            computed_at: chrono::Utc::now(),
        })
    }

    async fn analyze_tool_usage(&self, session: &BeruSession) -> Result<IronInsight, IronError> {
        let tool_calls: Vec<&str> = session
            .messages
            .iter()
            .filter_map(|msg| msg.tool_calls.as_ref())
            .flat_map(|calls| calls.iter().map(|call| call.name.as_str()))
            .collect();

        let unique_tools: std::collections::HashSet<&str> = tool_calls.iter().cloned().collect();

        Ok(IronInsight {
            session_id: session.uuid.clone(),
            category: InsightCategory::ToolUsage,
            confidence: 1.0,
            summary: format!(
                "Tools used: {} total, {} unique",
                tool_calls.len(),
                unique_tools.len()
            ),
            metadata: IronInsightMetadata {
                computation_time_ms: 5,
                data_points: tool_calls.len(),
                patterns_found: unique_tools.into_iter().map(String::from).collect(),
                metrics: HashMap::from([
                    ("total_tool_calls".to_string(), tool_calls.len() as f64),
                    ("unique_tools".to_string(), unique_tools.len() as f64),
                ]),
            },
            computed_at: chrono::Utc::now(),
        })
    }

    async fn analyze_performance(&self, session: &BeruSession) -> Result<IronInsight, IronError> {
        let response_times = self.calculate_response_times(&session.messages);
        let avg_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;

        Ok(IronInsight {
            session_id: session.uuid.clone(),
            category: InsightCategory::Performance,
            confidence: 0.8,
            summary: format!("Avg response time: {:.1}s", avg_response_time),
            metadata: IronInsightMetadata {
                computation_time_ms: 12,
                data_points: response_times.len(),
                patterns_found: vec![],
                metrics: HashMap::from([
                    ("avg_response_time".to_string(), avg_response_time),
                    (
                        "max_response_time".to_string(),
                        response_times.iter().cloned().fold(0.0, f64::max),
                    ),
                ]),
            },
            computed_at: chrono::Utc::now(),
        })
    }

    async fn needs_recomputation(
        &self,
        session: &BeruSession,
        last_computed: &chrono::DateTime<chrono::Utc>,
    ) -> bool {
        // Recompute if session was updated after last computation
        session.updated_at > *last_computed
    }

    fn calculate_topic_diversity(&self, topics: &[String]) -> f64 {
        if topics.is_empty() {
            return 0.0;
        }

        let unique_topics: std::collections::HashSet<&String> = topics.iter().collect();
        unique_topics.len() as f64 / topics.len() as f64
    }

    fn calculate_topic_consistency(&self, topics: &[String]) -> f64 {
        if topics.is_empty() {
            return 1.0;
        }

        // Simple consistency metric based on topic repetition
        let mut topic_counts = HashMap::new();
        for topic in topics {
            *topic_counts.entry(topic.clone()).or_insert(0) += 1;
        }

        let max_count = topic_counts.values().max().unwrap_or(&0);
        *max_count as f64 / topics.len() as f64
    }

    fn calculate_variance(&self, values: &[f32]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance =
            values.iter().map(|x| (*x - mean).powi(2)).sum::<f32>() / values.len() as f32;
        variance as f64
    }

    fn calculate_avg_message_length(&self, messages: &[BeruMessage]) -> f64 {
        if messages.is_empty() {
            return 0.0;
        }

        let total_length: usize = messages.iter().map(|msg| msg.content().len()).sum();
        total_length as f64 / messages.len() as f64
    }

    fn calculate_response_times(&self, messages: &[BeruMessage]) -> Vec<f64> {
        // Calculate time between consecutive messages
        messages
            .windows(2)
            .map(|pair| {
                let diff = pair[1].created_at - pair[0].created_at;
                diff.num_seconds() as f64
            })
            .collect()
    }
}

#[async_trait]
impl InsightsEngine<BeruSession> for IronEngine {
    type Insight = IronInsight;
    type Error = IronError;

    async fn analyze(&self, data: &BeruSession) -> Result<Self::Insight, Self::Error> {
        let insights = self.analyze_incremental(data).await?;
        insights.into_iter().next().ok_or(IronError::NoInsights)
    }

    async fn analyze_batch(
        &self,
        data: Vec<BeruSession>,
    ) -> Result<Vec<Self::Insight>, Self::Error> {
        let mut all_insights = Vec::new();

        for session in data {
            let session_insights = self.analyze_incremental(&session).await?;
            all_insights.extend(session_insights);
        }

        Ok(all_insights)
    }

    async fn stream_insights<S>(
        &self,
        data_stream: S,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Insight, Self::Error>> + Send>>
    where
        S: Stream<Item = BeruSession> + Send + 'static,
    {
        let stream = data_stream.then(move |session| async move {
            match self.analyze_incremental(&session).await {
                Ok(insights) => Ok(insights.into_iter().next().unwrap_or_else(|| IronInsight {
                    session_id: session.uuid.clone(),
                    category: InsightCategory::Topic,
                    confidence: 0.0,
                    summary: "No insights generated".to_string(),
                    metadata: IronInsightMetadata {
                        computation_time_ms: 0,
                        data_points: 0,
                        patterns_found: vec![],
                        metrics: HashMap::new(),
                    },
                    computed_at: chrono::Utc::now(),
                })),
                Err(e) => Err(e),
            }
        });

        Box::pin(stream)
    }

    fn supports_incremental(&self) -> bool {
        true
    }

    fn cache_size(&self) -> usize {
        1000
    }
}

impl ShadowAgent for IronEngine {
    const NAME: &'static str = "Iron";
    const SPECIALIZATION: &'static str = "Insights Optimization";

    fn performance_targets(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            parse_duration: std::time::Duration::from_millis(100), // 5x faster than v1
            memory_usage: 20 * 1024 * 1024,                        // 20MB for caching
            throughput: 500.0,                                     // 500 sessions per second
            error_rate: 0.02,                                      // 2% error tolerance
        }
    }
}

/// Caching layer for Iron
pub struct IronCache {
    cache: HashMap<String, CachedInsights>,
    max_size: usize,
    access_order: VecDeque<String>,
}

impl IronCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            access_order: VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&CachedInsights> {
        if self.cache.contains_key(key) {
            // Move to front for LRU
            self.access_order.retain(|k| k != key);
            self.access_order.push_front(key.to_string());
            self.cache.get(key)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: String, value: CachedInsights) {
        if self.cache.len() >= self.max_size {
            // Remove LRU item
            if let Some(lru_key) = self.access_order.pop_back() {
                self.cache.remove(&lru_key);
            }
        }

        self.cache.insert(key.clone(), value);
        self.access_order.push_front(key);
    }
}

#[derive(Debug, Clone)]
pub struct CachedInsights {
    pub insights: Vec<IronInsight>,
    pub computed_at: chrono::DateTime<chrono::Utc>,
    pub computation_time: std::time::Duration,
}

/// Pattern analysis for Iron
pub struct IronPatternAnalyzer {
    topic_keywords: HashMap<String, Vec<String>>,
}

impl IronPatternAnalyzer {
    pub fn new() -> Self {
        let mut topic_keywords = HashMap::new();

        // Programming topics
        topic_keywords.insert(
            "Programming".to_string(),
            vec![
                "code".to_string(),
                "function".to_string(),
                "class".to_string(),
                "variable".to_string(),
                "algorithm".to_string(),
                "debug".to_string(),
            ],
        );

        // AI/ML topics
        topic_keywords.insert(
            "AI/ML".to_string(),
            vec![
                "model".to_string(),
                "training".to_string(),
                "neural".to_string(),
                "machine learning".to_string(),
                "deep learning".to_string(),
            ],
        );

        Self { topic_keywords }
    }

    pub async fn extract_topics(&self, messages: &[BeruMessage]) -> Result<Vec<String>, IronError> {
        let mut topic_scores = HashMap::new();

        for message in messages {
            let content = message.content().to_lowercase();

            for (topic, keywords) in &self.topic_keywords {
                let score = keywords
                    .iter()
                    .map(|keyword| content.matches(keyword).count())
                    .sum::<usize>();

                *topic_scores.entry(topic.clone()).or_insert(0) += score;
            }
        }

        let mut topics: Vec<(String, usize)> = topic_scores.into_iter().collect();
        topics.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(topics.into_iter().map(|(topic, _)| topic).take(3).collect())
    }

    pub async fn analyze_sentiment(&self, messages: &[BeruMessage]) -> Result<Vec<f32>, IronError> {
        // Simple sentiment analysis based on keyword matching
        let positive_words = vec!["good", "great", "excellent", "love", "perfect", "amazing"];
        let negative_words = vec!["bad", "terrible", "hate", "awful", "worst", "horrible"];

        let mut sentiments = Vec::new();

        for message in messages {
            let content = message.content().to_lowercase();
            let words: Vec<&str> = content.split_whitespace().collect();

            if words.is_empty() {
                sentiments.push(0.0);
                continue;
            }

            let positive_count = positive_words
                .iter()
                .map(|word| content.matches(word).count())
                .sum::<usize>();

            let negative_count = negative_words
                .iter()
                .map(|word| content.matches(word).count())
                .sum::<usize>();

            let sentiment = (positive_count as f32 - negative_count as f32) / words.len() as f32;
            sentiments.push(sentiment.clamp(-1.0, 1.0));
        }

        Ok(sentiments)
    }
}

/// Metrics collection for Iron
pub struct IronMetricsCollector;

impl IronMetricsCollector {
    pub fn new() -> Self {
        Self
    }

    pub async fn calculate_complexity(&self, messages: &[BeruMessage]) -> Result<f64, IronError> {
        if messages.is_empty() {
            return Ok(0.0);
        }

        // Complexity based on multiple factors
        let avg_length = messages
            .iter()
            .map(|msg| msg.content().len())
            .sum::<usize>() as f64
            / messages.len() as f64;

        let unique_words = messages
            .iter()
            .flat_map(|msg| msg.content().split_whitespace())
            .collect::<std::collections::HashSet<_>>()
            .len() as f64;

        let total_words = messages
            .iter()
            .map(|msg| msg.content().split_whitespace().count())
            .sum::<usize>() as f64;

        let vocab_diversity = if total_words > 0.0 {
            unique_words / total_words
        } else {
            0.0
        };

        // Normalize and combine factors
        let length_factor = (avg_length / 1000.0).clamp(0.0, 1.0);
        let diversity_factor = vocab_diversity.clamp(0.0, 1.0);

        Ok((length_factor + diversity_factor) / 2.0)
    }
}

/// Incremental state for Iron
pub struct IronIncrementalState {
    session_stats: HashMap<String, SessionStats>,
    global_metrics: GlobalMetrics,
}

impl IronIncrementalState {
    pub fn new() -> Self {
        Self {
            session_stats: HashMap::new(),
            global_metrics: GlobalMetrics::new(),
        }
    }

    pub fn update_session_stats(&mut self, session: &BeruSession) {
        let stats = SessionStats {
            message_count: session.messages.len(),
            last_updated: session.updated_at,
            complexity_score: 0.0, // Would be calculated
        };

        self.session_stats.insert(session.uuid.clone(), stats);
        self.global_metrics.total_sessions += 1;
        self.global_metrics.total_messages += session.messages.len();
    }
}

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub message_count: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub complexity_score: f64,
}

#[derive(Debug, Clone)]
pub struct GlobalMetrics {
    pub total_sessions: usize,
    pub total_messages: usize,
    pub avg_complexity: f64,
}

impl GlobalMetrics {
    pub fn new() -> Self {
        Self {
            total_sessions: 0,
            total_messages: 0,
            avg_complexity: 0.0,
        }
    }
}

/// Iron-specific errors
#[derive(Debug, thiserror::Error)]
pub enum IronError {
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),
    #[error("No insights generated")]
    NoInsights,
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Pattern analysis failed: {0}")]
    PatternAnalysisFailed(String),
}
