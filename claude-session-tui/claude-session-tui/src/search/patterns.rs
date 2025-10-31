//! Pattern recognition for conversation analysis and learning insights

use super::{SearchError, SearchResult, SearchEngine};
use crate::{Session, Block, Role, ConversationIntent, ProgrammingLanguage};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};
use chrono::{DateTime, Utc, Duration};
use regex::Regex;
use tracing::{debug, warn, instrument};

/// Pattern types for conversation analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    /// Architecture decisions across projects
    ArchitectureDecisions,
    /// Common debugging patterns
    DebuggingPatterns,
    /// Learning progression paths
    LearningProgression,
    /// Multi-agent coordination patterns
    CoordinationPatterns,
    /// Error resolution workflows
    ErrorResolution,
    /// Code review patterns
    CodeReview,
    /// Tool usage patterns
    ToolUsage,
    /// Technology stack patterns
    TechnologyStack,
    /// Problem-solving approaches
    ProblemSolving,
    /// Documentation patterns
    Documentation,
    /// Testing strategies
    TestingStrategies,
    /// Performance optimization patterns
    PerformanceOptimization,
}

/// Pattern match result with confidence and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_type: PatternType,
    pub pattern_id: String,
    pub title: String,
    pub description: String,
    pub confidence_score: f64,
    pub occurrences: Vec<PatternOccurrence>,
    pub related_sessions: Vec<String>,
    pub temporal_distribution: TemporalDistribution,
    pub context: PatternContext,
    pub insights: Vec<PatternInsight>,
}

/// Individual occurrence of a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOccurrence {
    pub session_id: String,
    pub block_ids: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub relevance_score: f64,
    pub context_snippet: String,
    pub extracted_entities: Vec<ExtractedEntity>,
}

/// Temporal distribution of pattern occurrences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDistribution {
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub frequency_by_month: HashMap<String, usize>,
    pub trend: TemporalTrend,
    pub seasonality: Option<SeasonalityPattern>,
}

/// Temporal trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalTrend {
    Increasing,
    Decreasing,
    Stable,
    Cyclical,
    Irregular,
}

/// Seasonality pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalityPattern {
    pub period_days: usize,
    pub amplitude: f64,
    pub confidence: f64,
}

/// Context information for pattern matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContext {
    pub related_technologies: HashSet<String>,
    pub project_contexts: HashSet<String>,
    pub common_intents: HashMap<ConversationIntent, usize>,
    pub user_behavior: UserBehaviorContext,
    pub environmental_factors: EnvironmentalFactors,
}

/// User behavior context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehaviorContext {
    pub interaction_style: String,
    pub common_question_patterns: Vec<String>,
    pub preferred_tools: Vec<String>,
    pub learning_indicators: Vec<String>,
}

/// Environmental factors affecting patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalFactors {
    pub time_of_day_distribution: HashMap<u32, usize>, // Hour -> count
    pub day_of_week_distribution: HashMap<String, usize>,
    pub session_length_correlation: f64,
    pub tool_availability_context: Vec<String>,
}

/// Insights derived from pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInsight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub actionable_recommendations: Vec<String>,
    pub confidence: f64,
    pub supporting_evidence: Vec<EvidenceItem>,
}

/// Types of insights from pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    LearningOpportunity,
    EfficiencyImprovement,
    KnowledgeGap,
    BestPractice,
    AntiPattern,
    TechnologyRecommendation,
    WorkflowOptimization,
    SkillDevelopment,
}

/// Supporting evidence for insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub source: String,
    pub description: String,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

/// Extracted entities from pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub entity_type: EntityType,
    pub value: String,
    pub confidence: f64,
    pub context: String,
}

/// Types of entities that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Technology,
    Framework,
    Library,
    Tool,
    Concept,
    Error,
    Solution,
    Pattern,
    Methodology,
    Person,
    Project,
}

/// Pattern recognition engine
pub struct PatternRecognizer {
    /// Compiled regex patterns for each pattern type
    pattern_matchers: HashMap<PatternType, Vec<CompiledPattern>>,
    /// Entity extractors
    entity_extractors: Vec<EntityExtractor>,
    /// Temporal analyzers
    temporal_analyzer: TemporalAnalyzer,
    /// Context analyzers
    context_analyzer: ContextAnalyzer,
    /// Insight generators
    insight_generator: InsightGenerator,
}

/// Compiled pattern matcher
#[derive(Debug)]
pub struct CompiledPattern {
    pub regex: Regex,
    pub weight: f64,
    pub required_context: Vec<String>,
    pub exclusions: Vec<Regex>,
}

/// Entity extractor for different types
#[derive(Debug)]
pub struct EntityExtractor {
    pub entity_type: EntityType,
    pub patterns: Vec<Regex>,
    pub confidence_threshold: f64,
    pub context_requirements: Vec<String>,
}

/// Temporal analysis component
#[derive(Debug)]
pub struct TemporalAnalyzer {
    pub trend_window_days: i64,
    pub seasonality_min_samples: usize,
}

/// Context analysis component
#[derive(Debug)]
pub struct ContextAnalyzer {
    pub project_detection_patterns: Vec<Regex>,
    pub technology_keywords: HashSet<String>,
}

/// Insight generation component
#[derive(Debug)]
pub struct InsightGenerator {
    pub min_confidence_threshold: f64,
    pub recommendation_templates: HashMap<InsightType, Vec<String>>,
}

impl PatternRecognizer {
    /// Create a new pattern recognizer with default patterns
    pub fn new() -> SearchResult<Self> {
        let pattern_matchers = Self::build_pattern_matchers()?;
        let entity_extractors = Self::build_entity_extractors()?;
        let temporal_analyzer = TemporalAnalyzer::new();
        let context_analyzer = ContextAnalyzer::new()?;
        let insight_generator = InsightGenerator::new();

        Ok(Self {
            pattern_matchers,
            entity_extractors,
            temporal_analyzer,
            context_analyzer,
            insight_generator,
        })
    }

    /// Analyze sessions for patterns
    #[instrument(skip(self, sessions))]
    pub async fn analyze_patterns(
        &self,
        sessions: &[Session],
        pattern_types: &[PatternType],
        min_confidence: f64,
    ) -> SearchResult<Vec<PatternMatch>> {
        let mut pattern_matches = Vec::new();

        for pattern_type in pattern_types {
            let matches = self.find_pattern_matches(sessions, pattern_type, min_confidence).await?;
            pattern_matches.extend(matches);
        }

        // Sort by confidence score
        pattern_matches.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap_or(std::cmp::Ordering::Equal));

        debug!("Found {} pattern matches across {} sessions", pattern_matches.len(), sessions.len());
        Ok(pattern_matches)
    }

    /// Find matches for a specific pattern type
    async fn find_pattern_matches(
        &self,
        sessions: &[Session],
        pattern_type: &PatternType,
        min_confidence: f64,
    ) -> SearchResult<Vec<PatternMatch>> {
        let matchers = self.pattern_matchers.get(pattern_type)
            .ok_or_else(|| SearchError::pattern_recognition(format!("No matchers for pattern type: {:?}", pattern_type)))?;

        let mut occurrences = Vec::new();

        // Find pattern occurrences across sessions
        for session in sessions {
            let session_occurrences = self.find_session_occurrences(session, matchers).await?;
            occurrences.extend(session_occurrences);
        }

        if occurrences.is_empty() {
            return Ok(Vec::new());
        }

        // Group occurrences into pattern matches
        let pattern_matches = self.group_occurrences_into_patterns(
            pattern_type,
            occurrences,
            min_confidence,
        ).await?;

        Ok(pattern_matches)
    }

    /// Find pattern occurrences within a session
    async fn find_session_occurrences(
        &self,
        session: &Session,
        matchers: &[CompiledPattern],
    ) -> SearchResult<Vec<PatternOccurrence>> {
        let mut occurrences = Vec::new();

        for (i, block) in session.blocks.iter().enumerate() {
            for matcher in matchers {
                if let Some(occurrence) = self.test_pattern_match(session, block, i, matcher).await? {
                    occurrences.push(occurrence);
                }
            }
        }

        Ok(occurrences)
    }

    /// Test if a block matches a pattern
    async fn test_pattern_match(
        &self,
        session: &Session,
        block: &Block,
        block_index: usize,
        matcher: &CompiledPattern,
    ) -> SearchResult<Option<PatternOccurrence>> {
        let content = &block.content.raw_text;

        // Test main pattern
        if !matcher.regex.is_match(content) {
            return Ok(None);
        }

        // Test exclusions
        for exclusion in &matcher.exclusions {
            if exclusion.is_match(content) {
                return Ok(None);
            }
        }

        // Check context requirements
        if !self.check_context_requirements(session, block_index, &matcher.required_context) {
            return Ok(None);
        }

        // Extract entities from the match
        let entities = self.extract_entities_from_block(block).await?;

        // Calculate relevance score
        let relevance_score = self.calculate_relevance_score(block, &entities, matcher.weight);

        let occurrence = PatternOccurrence {
            session_id: session.id.to_string(),
            block_ids: vec![block.id.to_string()],
            timestamp: block.timestamp,
            relevance_score,
            context_snippet: self.extract_context_snippet(content, 200),
            extracted_entities: entities,
        };

        Ok(Some(occurrence))
    }

    /// Check if context requirements are met
    fn check_context_requirements(
        &self,
        session: &Session,
        block_index: usize,
        requirements: &[String],
    ) -> bool {
        if requirements.is_empty() {
            return true;
        }

        // Check surrounding blocks for context
        let context_window = 3; // Check 3 blocks before and after
        let start = block_index.saturating_sub(context_window);
        let end = (block_index + context_window + 1).min(session.blocks.len());

        for req in requirements {
            let mut found = false;
            for i in start..end {
                if session.blocks[i].content.raw_text.contains(req) {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }

        true
    }

    /// Extract entities from a conversation block
    async fn extract_entities_from_block(&self, block: &Block) -> SearchResult<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();

        for extractor in &self.entity_extractors {
            let block_entities = self.extract_entities_with_extractor(block, extractor).await?;
            entities.extend(block_entities);
        }

        // Deduplicate entities
        entities.sort_by(|a, b| a.value.cmp(&b.value));
        entities.dedup_by(|a, b| a.value == b.value && a.entity_type == b.entity_type);

        Ok(entities)
    }

    /// Extract entities using a specific extractor
    async fn extract_entities_with_extractor(
        &self,
        block: &Block,
        extractor: &EntityExtractor,
    ) -> SearchResult<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();
        let content = &block.content.raw_text;

        for pattern in &extractor.patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(matched) = cap.get(0) {
                    let value = matched.as_str().to_string();
                    let confidence = self.calculate_entity_confidence(&value, &extractor.entity_type);

                    if confidence >= extractor.confidence_threshold {
                        entities.push(ExtractedEntity {
                            entity_type: extractor.entity_type.clone(),
                            value,
                            confidence,
                            context: self.extract_context_snippet(content, 100),
                        });
                    }
                }
            }
        }

        Ok(entities)
    }

    /// Calculate entity extraction confidence
    fn calculate_entity_confidence(&self, _value: &str, entity_type: &EntityType) -> f64 {
        // Simplified confidence calculation
        // In production, this would use more sophisticated NLP techniques
        match entity_type {
            EntityType::Technology | EntityType::Framework | EntityType::Library => 0.8,
            EntityType::Tool => 0.7,
            EntityType::Error => 0.9,
            EntityType::Solution => 0.6,
            _ => 0.5,
        }
    }

    /// Calculate pattern match relevance score
    fn calculate_relevance_score(
        &self,
        block: &Block,
        entities: &[ExtractedEntity],
        base_weight: f64,
    ) -> f64 {
        let mut score = base_weight;

        // Boost score based on entities found
        score += entities.len() as f64 * 0.1;

        // Boost based on block characteristics
        if block.tools.len() > 0 {
            score += 0.2;
        }

        if block.content.code_blocks.len() > 0 {
            score += 0.3;
        }

        // Boost based on role
        match block.role {
            Role::Assistant => score += 0.1,
            Role::User => score += 0.05,
            _ => {}
        }

        score.min(1.0)
    }

    /// Extract context snippet around pattern match
    fn extract_context_snippet(&self, content: &str, max_length: usize) -> String {
        if content.len() <= max_length {
            content.to_string()
        } else {
            let mut snippet = content.chars().take(max_length).collect::<String>();
            if let Some(last_space) = snippet.rfind(' ') {
                snippet.truncate(last_space);
                snippet.push_str("...");
            }
            snippet
        }
    }

    /// Group pattern occurrences into cohesive pattern matches
    async fn group_occurrences_into_patterns(
        &self,
        pattern_type: &PatternType,
        occurrences: Vec<PatternOccurrence>,
        min_confidence: f64,
    ) -> SearchResult<Vec<PatternMatch>> {
        // Group by similarity and temporal proximity
        let grouped = self.group_similar_occurrences(occurrences).await?;
        
        let mut pattern_matches = Vec::new();

        for group in grouped {
            let pattern_match = self.create_pattern_match_from_group(pattern_type, group).await?;
            
            if pattern_match.confidence_score >= min_confidence {
                pattern_matches.push(pattern_match);
            }
        }

        Ok(pattern_matches)
    }

    /// Group similar occurrences together
    async fn group_similar_occurrences(
        &self,
        mut occurrences: Vec<PatternOccurrence>,
    ) -> SearchResult<Vec<Vec<PatternOccurrence>>> {
        // Sort by timestamp for temporal grouping
        occurrences.sort_by_key(|o| o.timestamp);

        let mut groups = Vec::new();
        let mut current_group = Vec::new();

        for occurrence in occurrences {
            if current_group.is_empty() {
                current_group.push(occurrence);
            } else {
                let last = current_group.last().unwrap();
                let time_diff = occurrence.timestamp.signed_duration_since(last.timestamp);
                
                // Group if within 1 hour and similar context
                if time_diff < Duration::hours(1) && 
                   self.are_occurrences_similar(last, &occurrence) {
                    current_group.push(occurrence);
                } else {
                    if !current_group.is_empty() {
                        groups.push(current_group);
                    }
                    current_group = vec![occurrence];
                }
            }
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        Ok(groups)
    }

    /// Check if two occurrences are similar
    fn are_occurrences_similar(&self, a: &PatternOccurrence, b: &PatternOccurrence) -> bool {
        // Check entity overlap
        let a_entities: HashSet<_> = a.extracted_entities.iter()
            .map(|e| (&e.entity_type, &e.value))
            .collect();
        let b_entities: HashSet<_> = b.extracted_entities.iter()
            .map(|e| (&e.entity_type, &e.value))
            .collect();

        let intersection: HashSet<_> = a_entities.intersection(&b_entities).collect();
        let union: HashSet<_> = a_entities.union(&b_entities).collect();

        if union.is_empty() {
            return false;
        }

        let jaccard_similarity = intersection.len() as f64 / union.len() as f64;
        jaccard_similarity > 0.3 // 30% similarity threshold
    }

    /// Create a pattern match from a group of occurrences
    async fn create_pattern_match_from_group(
        &self,
        pattern_type: &PatternType,
        group: Vec<PatternOccurrence>,
    ) -> SearchResult<PatternMatch> {
        let pattern_id = format!("{:?}_{}", pattern_type, uuid::Uuid::new_v4());
        
        // Calculate confidence score
        let confidence_score = group.iter()
            .map(|o| o.relevance_score)
            .sum::<f64>() / group.len() as f64;

        // Extract common entities
        let all_entities: Vec<_> = group.iter()
            .flat_map(|o| &o.extracted_entities)
            .collect();

        // Build temporal distribution
        let temporal_distribution = self.build_temporal_distribution(&group)?;

        // Build context
        let context = self.build_pattern_context(&group, &all_entities).await?;

        // Generate insights
        let insights = self.generate_pattern_insights(pattern_type, &group, &context).await?;

        // Extract related sessions
        let related_sessions = group.iter()
            .map(|o| o.session_id.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let title = self.generate_pattern_title(pattern_type, &group);
        let description = self.generate_pattern_description(pattern_type, &group, &context);

        Ok(PatternMatch {
            pattern_type: pattern_type.clone(),
            pattern_id,
            title,
            description,
            confidence_score,
            occurrences: group,
            related_sessions,
            temporal_distribution,
            context,
            insights,
        })
    }

    /// Build temporal distribution for pattern occurrences
    fn build_temporal_distribution(&self, occurrences: &[PatternOccurrence]) -> SearchResult<TemporalDistribution> {
        let timestamps: Vec<_> = occurrences.iter().map(|o| o.timestamp).collect();
        
        let first_seen = timestamps.iter().min().copied().unwrap_or_else(Utc::now);
        let last_seen = timestamps.iter().max().copied().unwrap_or_else(Utc::now);

        // Build frequency by month
        let mut frequency_by_month = HashMap::new();
        for timestamp in &timestamps {
            let month_key = format!("{}-{:02}", timestamp.year(), timestamp.month());
            *frequency_by_month.entry(month_key).or_insert(0) += 1;
        }

        // Determine trend (simplified)
        let trend = if timestamps.len() < 2 {
            TemporalTrend::Stable
        } else {
            // Simple trend analysis based on first and second half
            let mid_point = timestamps.len() / 2;
            let first_half = &timestamps[..mid_point];
            let second_half = &timestamps[mid_point..];
            
            if second_half.len() > first_half.len() {
                TemporalTrend::Increasing
            } else if second_half.len() < first_half.len() {
                TemporalTrend::Decreasing
            } else {
                TemporalTrend::Stable
            }
        };

        Ok(TemporalDistribution {
            first_seen,
            last_seen,
            frequency_by_month,
            trend,
            seasonality: None, // Would implement sophisticated seasonality detection
        })
    }

    /// Build pattern context from occurrences and entities
    async fn build_pattern_context(
        &self,
        occurrences: &[PatternOccurrence],
        entities: &[&ExtractedEntity],
    ) -> SearchResult<PatternContext> {
        let mut related_technologies = HashSet::new();
        let mut project_contexts = HashSet::new();
        let mut common_intents = HashMap::new();

        // Extract technologies from entities
        for entity in entities {
            match entity.entity_type {
                EntityType::Technology | EntityType::Framework | EntityType::Library => {
                    related_technologies.insert(entity.value.clone());
                }
                EntityType::Project => {
                    project_contexts.insert(entity.value.clone());
                }
                _ => {}
            }
        }

        // Build user behavior context (simplified)
        let user_behavior = UserBehaviorContext {
            interaction_style: "analytical".to_string(), // Would analyze actual patterns
            common_question_patterns: vec!["How to".to_string(), "What is".to_string()],
            preferred_tools: vec!["code_editor".to_string(), "terminal".to_string()],
            learning_indicators: vec!["explain".to_string(), "example".to_string()],
        };

        // Build environmental factors (simplified)
        let environmental_factors = EnvironmentalFactors {
            time_of_day_distribution: HashMap::new(), // Would analyze actual timestamps
            day_of_week_distribution: HashMap::new(),
            session_length_correlation: 0.5,
            tool_availability_context: vec!["development_environment".to_string()],
        };

        Ok(PatternContext {
            related_technologies,
            project_contexts,
            common_intents,
            user_behavior,
            environmental_factors,
        })
    }

    /// Generate insights from pattern analysis
    async fn generate_pattern_insights(
        &self,
        pattern_type: &PatternType,
        occurrences: &[PatternOccurrence],
        context: &PatternContext,
    ) -> SearchResult<Vec<PatternInsight>> {
        let mut insights = Vec::new();

        match pattern_type {
            PatternType::DebuggingPatterns => {
                insights.extend(self.generate_debugging_insights(occurrences, context));
            }
            PatternType::LearningProgression => {
                insights.extend(self.generate_learning_insights(occurrences, context));
            }
            PatternType::ArchitectureDecisions => {
                insights.extend(self.generate_architecture_insights(occurrences, context));
            }
            _ => {
                // Generic insights for other pattern types
                insights.push(PatternInsight {
                    insight_type: InsightType::BestPractice,
                    title: format!("Pattern identified: {:?}", pattern_type),
                    description: "This pattern has been observed multiple times".to_string(),
                    actionable_recommendations: vec![
                        "Consider documenting this pattern for future reference".to_string()
                    ],
                    confidence: 0.6,
                    supporting_evidence: vec![],
                });
            }
        }

        Ok(insights)
    }

    /// Generate debugging-specific insights
    fn generate_debugging_insights(&self, _occurrences: &[PatternOccurrence], _context: &PatternContext) -> Vec<PatternInsight> {
        vec![
            PatternInsight {
                insight_type: InsightType::WorkflowOptimization,
                title: "Debugging workflow optimization".to_string(),
                description: "Common debugging patterns suggest optimization opportunities".to_string(),
                actionable_recommendations: vec![
                    "Consider creating debugging checklists".to_string(),
                    "Document common error patterns".to_string(),
                ],
                confidence: 0.8,
                supporting_evidence: vec![],
            }
        ]
    }

    /// Generate learning-specific insights
    fn generate_learning_insights(&self, _occurrences: &[PatternOccurrence], _context: &PatternContext) -> Vec<PatternInsight> {
        vec![
            PatternInsight {
                insight_type: InsightType::LearningOpportunity,
                title: "Learning progression identified".to_string(),
                description: "Your learning pattern shows consistent progression in specific areas".to_string(),
                actionable_recommendations: vec![
                    "Continue building on identified strengths".to_string(),
                    "Consider advanced topics in these areas".to_string(),
                ],
                confidence: 0.7,
                supporting_evidence: vec![],
            }
        ]
    }

    /// Generate architecture-specific insights
    fn generate_architecture_insights(&self, _occurrences: &[PatternOccurrence], _context: &PatternContext) -> Vec<PatternInsight> {
        vec![
            PatternInsight {
                insight_type: InsightType::BestPractice,
                title: "Architecture decision pattern".to_string(),
                description: "Consistent architectural choices identified across projects".to_string(),
                actionable_recommendations: vec![
                    "Document architectural decisions for team reference".to_string(),
                    "Create architectural decision records (ADRs)".to_string(),
                ],
                confidence: 0.85,
                supporting_evidence: vec![],
            }
        ]
    }

    /// Generate pattern title
    fn generate_pattern_title(&self, pattern_type: &PatternType, occurrences: &[PatternOccurrence]) -> String {
        let count = occurrences.len();
        match pattern_type {
            PatternType::DebuggingPatterns => format!("Debugging Pattern ({} occurrences)", count),
            PatternType::LearningProgression => format!("Learning Pattern ({} occurrences)", count),
            PatternType::ArchitectureDecisions => format!("Architecture Pattern ({} occurrences)", count),
            _ => format!("{:?} Pattern ({} occurrences)", pattern_type, count),
        }
    }

    /// Generate pattern description
    fn generate_pattern_description(
        &self,
        pattern_type: &PatternType,
        occurrences: &[PatternOccurrence],
        context: &PatternContext,
    ) -> String {
        let tech_list: Vec<_> = context.related_technologies.iter().take(3).collect();
        let tech_str = if tech_list.is_empty() {
            "various technologies".to_string()
        } else {
            tech_list.join(", ")
        };

        format!(
            "This {:?} pattern was observed {} times across multiple sessions, involving {}.",
            pattern_type,
            occurrences.len(),
            tech_str
        )
    }

    /// Build pattern matchers for each pattern type
    fn build_pattern_matchers() -> SearchResult<HashMap<PatternType, Vec<CompiledPattern>>> {
        let mut matchers = HashMap::new();

        // Debugging patterns
        let debugging_patterns = vec![
            CompiledPattern {
                regex: Regex::new(r"(?i)\b(error|exception|bug|issue|problem|fail|crash)\b")?,
                weight: 0.8,
                required_context: vec!["debug", "fix", "solve"].into_iter().map(String::from).collect(),
                exclusions: vec![],
            },
            CompiledPattern {
                regex: Regex::new(r"(?i)\b(stack trace|traceback|backtrace)\b")?,
                weight: 0.9,
                required_context: vec![],
                exclusions: vec![],
            },
        ];
        matchers.insert(PatternType::DebuggingPatterns, debugging_patterns);

        // Architecture decision patterns
        let architecture_patterns = vec![
            CompiledPattern {
                regex: Regex::new(r"(?i)\b(architecture|design|pattern|framework|structure)\b")?,
                weight: 0.7,
                required_context: vec!["choose", "decide", "implement"].into_iter().map(String::from).collect(),
                exclusions: vec![],
            },
        ];
        matchers.insert(PatternType::ArchitectureDecisions, architecture_patterns);

        // Add more pattern types...
        
        Ok(matchers)
    }

    /// Build entity extractors
    fn build_entity_extractors() -> SearchResult<Vec<EntityExtractor>> {
        let mut extractors = Vec::new();

        // Technology extractor
        extractors.push(EntityExtractor {
            entity_type: EntityType::Technology,
            patterns: vec![
                Regex::new(r"\b(Rust|Python|JavaScript|TypeScript|Java|Go|C\+\+)\b")?,
                Regex::new(r"\b(React|Vue|Angular|Django|Flask|Express)\b")?,
            ],
            confidence_threshold: 0.8,
            context_requirements: vec![],
        });

        // Error extractor
        extractors.push(EntityExtractor {
            entity_type: EntityType::Error,
            patterns: vec![
                Regex::new(r"\b\w*Error\b")?,
                Regex::new(r"\b\w*Exception\b")?,
                Regex::new(r"error code \d+")?,
            ],
            confidence_threshold: 0.9,
            context_requirements: vec![],
        });

        Ok(extractors)
    }
}

impl TemporalAnalyzer {
    fn new() -> Self {
        Self {
            trend_window_days: 30,
            seasonality_min_samples: 20,
        }
    }
}

impl ContextAnalyzer {
    fn new() -> SearchResult<Self> {
        Ok(Self {
            project_detection_patterns: vec![
                Regex::new(r"/([^/]+)/([^/]+)\.git")?, // Git URLs
                Regex::new(r"project[:\s]+([a-zA-Z0-9_-]+)")?, // Project mentions
            ],
            technology_keywords: [
                "rust", "python", "javascript", "typescript", "java", "go", "cpp",
                "react", "vue", "angular", "django", "flask", "express", "fastapi"
            ].into_iter().map(String::from).collect(),
        })
    }
}

impl InsightGenerator {
    fn new() -> Self {
        let mut recommendation_templates = HashMap::new();
        
        recommendation_templates.insert(InsightType::LearningOpportunity, vec![
            "Consider exploring advanced concepts in this area".to_string(),
            "Practice implementing this pattern in different contexts".to_string(),
        ]);
        
        recommendation_templates.insert(InsightType::EfficiencyImprovement, vec![
            "Automate this recurring task".to_string(),
            "Create templates or snippets for common patterns".to_string(),
        ]);

        Self {
            min_confidence_threshold: 0.6,
            recommendation_templates,
        }
    }
}