//! Intelligent insights extraction and conversation analysis

use crate::{
    error::{ClaudeSessionError, Result},
    models::*,
};
use std::collections::{HashMap, HashSet};
use chrono::{Duration, Utc};
use tracing::{debug, info, warn};

/// Advanced insights analyzer for conversation sessions
pub struct InsightsAnalyzer {
    config: AnalysisConfig,
    patterns: InsightPatterns,
}

/// Configuration for insights analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub analyze_conversation_flow: bool,
    pub detect_learning_patterns: bool,
    pub calculate_productivity_metrics: bool,
    pub analyze_collaboration_style: bool,
    pub extract_technical_topics: bool,
    pub track_problem_solving: bool,
    pub measure_code_quality: bool,
    pub identify_knowledge_gaps: bool,
}

/// Pattern matching for insight detection
#[derive(Debug)]
struct InsightPatterns {
    question_indicators: Vec<&'static str>,
    solution_indicators: Vec<&'static str>,
    problem_indicators: Vec<&'static str>,
    learning_indicators: Vec<&'static str>,
    debugging_indicators: Vec<&'static str>,
    implementation_indicators: Vec<&'static str>,
    review_indicators: Vec<&'static str>,
    planning_indicators: Vec<&'static str>,
}

impl Default for InsightsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl InsightsAnalyzer {
    /// Create new insights analyzer with default configuration
    pub fn new() -> Self {
        Self::with_config(AnalysisConfig {
            analyze_conversation_flow: true,
            detect_learning_patterns: true,
            calculate_productivity_metrics: true,
            analyze_collaboration_style: true,
            extract_technical_topics: true,
            track_problem_solving: true,
            measure_code_quality: true,
            identify_knowledge_gaps: true,
        })
    }

    /// Create insights analyzer with custom configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        let patterns = InsightPatterns {
            question_indicators: vec![
                "how do", "what is", "why does", "where can", "when should",
                "which way", "can you help", "is it possible", "?",
            ],
            solution_indicators: vec![
                "you can", "try this", "solution", "fix", "resolve",
                "here's how", "the answer is", "implement", "use this",
            ],
            problem_indicators: vec![
                "error", "issue", "problem", "bug", "fail", "broken",
                "doesn't work", "not working", "exception", "crash",
            ],
            learning_indicators: vec![
                "learn", "understand", "explain", "teach", "show me",
                "what does this mean", "how does this work", "tutorial",
            ],
            debugging_indicators: vec![
                "debug", "trace", "investigate", "figure out", "find the cause",
                "stack trace", "logs", "diagnostic", "troubleshoot",
            ],
            implementation_indicators: vec![
                "implement", "code", "write", "create", "build", "develop",
                "add feature", "make changes", "modify", "update",
            ],
            review_indicators: vec![
                "review", "check", "validate", "verify", "test", "examine",
                "look at", "analyze", "assess", "evaluate",
            ],
            planning_indicators: vec![
                "plan", "design", "architecture", "structure", "organize",
                "strategy", "approach", "workflow", "process", "roadmap",
            ],
        };

        Self { config, patterns }
    }

    /// Analyze a session and extract comprehensive insights
    pub async fn analyze_session(&self, session: &Session) -> Result<SessionInsights> {
        info!("Starting comprehensive session analysis with {} blocks", session.blocks.len());

        let mut insights = SessionInsights {
            primary_topics: Vec::new(),
            conversation_flow: ConversationFlow {
                phases: Vec::new(),
                transitions: Vec::new(),
                complexity_evolution: Vec::new(),
                focus_shifts: 0,
            },
            learning_outcomes: Vec::new(),
            productivity_metrics: ProductivityMetrics {
                tasks_completed: 0,
                problems_solved: 0,
                code_quality_score: 0.0,
                efficiency_rating: 0.0,
                collaboration_effectiveness: 0.0,
                time_to_resolution: Vec::new(),
            },
            collaboration_patterns: CollaborationPatterns {
                interaction_style: InteractionStyle::TaskOriented,
                question_types: HashMap::new(),
                feedback_quality: 0.0,
                iterative_cycles: 0,
                knowledge_transfer: 0.0,
            },
        };

        // Extract primary topics
        if self.config.extract_technical_topics {
            insights.primary_topics = self.extract_primary_topics(session).await?;
        }

        // Analyze conversation flow
        if self.config.analyze_conversation_flow {
            insights.conversation_flow = self.analyze_conversation_flow(session).await?;
        }

        // Detect learning outcomes
        if self.config.detect_learning_patterns {
            insights.learning_outcomes = self.detect_learning_outcomes(session).await?;
        }

        // Calculate productivity metrics
        if self.config.calculate_productivity_metrics {
            insights.productivity_metrics = self.calculate_productivity_metrics(session).await?;
        }

        // Analyze collaboration patterns
        if self.config.analyze_collaboration_style {
            insights.collaboration_patterns = self.analyze_collaboration_patterns(session).await?;
        }

        info!("Session analysis complete - found {} topics, {} phases, {} learning outcomes", 
              insights.primary_topics.len(), 
              insights.conversation_flow.phases.len(),
              insights.learning_outcomes.len());

        Ok(insights)
    }

    /// Extract primary topics with relevance scoring
    async fn extract_primary_topics(&self, session: &Session) -> Result<Vec<Topic>> {
        let mut topic_frequency: HashMap<String, usize> = HashMap::new();
        let mut topic_contexts: HashMap<String, Vec<String>> = HashMap::new();
        let mut tool_associations: HashMap<String, HashSet<String>> = HashMap::new();

        // Analyze all blocks for topic extraction
        for block in &session.blocks {
            let content_lower = block.content.raw_text.to_lowercase();
            
            // Extract technical topics
            let topics = self.extract_technical_topics_from_text(&content_lower);
            for topic in topics {
                *topic_frequency.entry(topic.clone()).or_insert(0) += 1;
                
                // Store context snippets
                let context = self.extract_topic_context(&block.content.raw_text, &topic);
                topic_contexts.entry(topic.clone()).or_default().push(context);

                // Associate with tools used in this block
                for tool in &block.tools {
                    tool_associations.entry(topic.clone()).or_default().insert(tool.tool_name.clone());
                }
            }

            // Extract programming languages as topics
            for code_block in &block.content.code_blocks {
                if let Some(lang) = &code_block.language {
                    let lang_name = format!("{:?}", lang).to_lowercase();
                    *topic_frequency.entry(lang_name.clone()).or_insert(0) += 1;
                    tool_associations.entry(lang_name).or_default();
                }
            }
        }

        // Convert to Topic structs with relevance scoring
        let mut topics: Vec<Topic> = topic_frequency
            .into_iter()
            .map(|(name, mentions)| {
                let relevance_score = self.calculate_topic_relevance(mentions, session.blocks.len());
                let subtopics = self.extract_subtopics(&name, &topic_contexts.get(&name).unwrap_or(&Vec::new()));
                let related_tools = tool_associations.get(&name)
                    .map(|tools| tools.iter().cloned().collect())
                    .unwrap_or_default();

                Topic {
                    name,
                    relevance_score,
                    mentions,
                    subtopics,
                    related_tools,
                }
            })
            .collect();

        // Sort by relevance score and take top topics
        topics.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        topics.truncate(20); // Keep top 20 topics

        Ok(topics)
    }

    /// Analyze conversation flow and phase transitions
    async fn analyze_conversation_flow(&self, session: &Session) -> Result<ConversationFlow> {
        let mut phases = Vec::new();
        let mut transitions = Vec::new();
        let mut complexity_scores = Vec::new();
        let mut current_phase: Option<(PhaseType, usize)> = None;
        let mut focus_shifts = 0;

        for (i, block) in session.blocks.iter().enumerate() {
            let content_lower = block.content.raw_text.to_lowercase();
            let detected_phase = self.detect_conversation_phase(&content_lower);
            let complexity = self.calculate_block_complexity(block);
            complexity_scores.push(complexity);

            match &current_phase {
                None => {
                    // First block - start new phase
                    current_phase = Some((detected_phase, i));
                }
                Some((current_phase_type, phase_start)) => {
                    if detected_phase != *current_phase_type {
                        // Phase transition detected
                        let phase_duration = self.calculate_phase_duration(session, *phase_start, i);
                        let primary_activity = self.extract_primary_activity(session, *phase_start, i);

                        phases.push(ConversationPhase {
                            phase_type: *current_phase_type,
                            start_block: *phase_start,
                            end_block: i - 1,
                            duration: phase_duration,
                            primary_activity,
                        });

                        transitions.push(PhaseTransition {
                            from_phase: *current_phase_type,
                            to_phase: detected_phase,
                            trigger: self.identify_transition_trigger(session, i),
                            block_number: i,
                        });

                        current_phase = Some((detected_phase, i));
                        focus_shifts += 1;
                    }
                }
            }
        }

        // Close final phase
        if let Some((phase_type, phase_start)) = current_phase {
            let phase_duration = self.calculate_phase_duration(session, phase_start, session.blocks.len());
            let primary_activity = self.extract_primary_activity(session, phase_start, session.blocks.len());

            phases.push(ConversationPhase {
                phase_type,
                start_block: phase_start,
                end_block: session.blocks.len() - 1,
                duration: phase_duration,
                primary_activity,
            });
        }

        Ok(ConversationFlow {
            phases,
            transitions,
            complexity_evolution: complexity_scores,
            focus_shifts,
        })
    }

    /// Detect learning outcomes from conversation patterns
    async fn detect_learning_outcomes(&self, session: &Session) -> Result<Vec<LearningOutcome>> {
        let mut outcomes = Vec::new();
        let mut skill_areas: HashMap<String, Vec<String>> = HashMap::new();

        for block in &session.blocks {
            let content_lower = block.content.raw_text.to_lowercase();

            // Detect learning indicators
            if self.contains_patterns(&content_lower, &self.patterns.learning_indicators) {
                // Extract skill area from context
                let skill_area = self.extract_skill_area(&content_lower);
                let concepts = self.extract_learned_concepts(&content_lower);
                
                skill_areas.entry(skill_area).or_default().extend(concepts);
            }

            // Detect successful problem solving (learning indicator)
            if block.role == Role::Assistant && 
               self.contains_patterns(&content_lower, &self.patterns.solution_indicators) {
                let skill_area = self.extract_skill_area_from_solution(&content_lower);
                let concepts = self.extract_solution_concepts(&content_lower);
                
                skill_areas.entry(skill_area).or_default().extend(concepts);
            }
        }

        // Convert to LearningOutcome structs
        for (skill_area, concepts) in skill_areas {
            let complexity_level = self.assess_complexity_level(&concepts);
            let confidence_gain = self.calculate_confidence_gain(&concepts);
            let practical_application = self.has_practical_application(session, &skill_area);

            outcomes.push(LearningOutcome {
                skill_area,
                concepts_learned: concepts.into_iter().collect(),
                complexity_level,
                confidence_gain,
                practical_application,
            });
        }

        Ok(outcomes)
    }

    /// Calculate comprehensive productivity metrics
    async fn calculate_productivity_metrics(&self, session: &Session) -> Result<ProductivityMetrics> {
        let mut tasks_completed = 0;
        let mut problems_solved = 0;
        let mut time_to_resolution = Vec::new();
        let mut problem_start_times: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();

        for block in &session.blocks {
            let content_lower = block.content.raw_text.to_lowercase();

            // Track problem identification
            if self.contains_patterns(&content_lower, &self.patterns.problem_indicators) {
                let problem_key = self.extract_problem_key(&content_lower);
                problem_start_times.insert(problem_key, block.timestamp);
            }

            // Track problem resolution
            if self.contains_patterns(&content_lower, &self.patterns.solution_indicators) {
                problems_solved += 1;
                
                // Calculate resolution time if we have a matching problem
                let solution_key = self.extract_solution_key(&content_lower);
                if let Some(start_time) = problem_start_times.remove(&solution_key) {
                    let resolution_time = block.timestamp - start_time;
                    time_to_resolution.push(resolution_time);
                }
            }

            // Track task completion
            if block.role == Role::Assistant && 
               (content_lower.contains("completed") || content_lower.contains("finished") || 
                content_lower.contains("done") || content_lower.contains("implemented")) {
                tasks_completed += 1;
            }
        }

        let code_quality_score = self.calculate_code_quality_score(session);
        let efficiency_rating = self.calculate_efficiency_rating(session, &time_to_resolution);
        let collaboration_effectiveness = self.calculate_collaboration_effectiveness(session);

        Ok(ProductivityMetrics {
            tasks_completed,
            problems_solved,
            code_quality_score,
            efficiency_rating,
            collaboration_effectiveness,
            time_to_resolution,
        })
    }

    /// Analyze collaboration patterns and interaction styles
    async fn analyze_collaboration_patterns(&self, session: &Session) -> Result<CollaborationPatterns> {
        let mut question_types = HashMap::new();
        let mut iterative_cycles = 0;
        let mut user_clarifications = 0;
        let mut assistant_clarifications = 0;

        let mut previous_topic: Option<String> = None;

        for block in &session.blocks {
            let content_lower = block.content.raw_text.to_lowercase();
            let current_topic = self.extract_primary_topic(&content_lower);

            match block.role {
                Role::User => {
                    // Classify question types
                    for indicator in &self.patterns.question_indicators {
                        if content_lower.contains(indicator) {
                            *question_types.entry(indicator.to_string()).or_insert(0) += 1;
                            break;
                        }
                    }

                    // Detect iterative refinement
                    if let Some(prev_topic) = &previous_topic {
                        if current_topic == *prev_topic {
                            iterative_cycles += 1;
                        }
                    }

                    // Count clarification requests
                    if content_lower.contains("clarify") || content_lower.contains("what do you mean") {
                        user_clarifications += 1;
                    }
                }
                Role::Assistant => {
                    // Count assistant clarifications
                    if content_lower.contains("to clarify") || content_lower.contains("let me explain") {
                        assistant_clarifications += 1;
                    }
                }
                _ => {}
            }

            previous_topic = Some(current_topic);
        }

        let interaction_style = self.determine_interaction_style(session, &question_types);
        let feedback_quality = self.assess_feedback_quality(session);
        let knowledge_transfer = self.calculate_knowledge_transfer(session);

        Ok(CollaborationPatterns {
            interaction_style,
            question_types,
            feedback_quality,
            iterative_cycles,
            knowledge_transfer,
        })
    }

    // Helper methods for analysis

    fn extract_technical_topics_from_text(&self, text: &str) -> Vec<String> {
        let mut topics = Vec::new();
        
        // Programming concepts
        let programming_topics = [
            "async", "await", "promise", "function", "class", "struct", "enum",
            "database", "api", "rest", "graphql", "sql", "json", "xml",
            "docker", "kubernetes", "deployment", "testing", "debugging",
            "performance", "optimization", "security", "authentication",
            "frontend", "backend", "fullstack", "microservices", "monolith"
        ];

        for topic in &programming_topics {
            if text.contains(topic) {
                topics.push(topic.to_string());
            }
        }

        topics
    }

    fn calculate_topic_relevance(&self, mentions: usize, total_blocks: usize) -> f64 {
        let frequency = mentions as f64 / total_blocks as f64;
        // Boost score for topics mentioned multiple times
        frequency * (1.0 + (mentions as f64).ln())
    }

    fn extract_subtopics(&self, _topic: &str, _contexts: &[String]) -> Vec<String> {
        // Placeholder for subtopic extraction logic
        Vec::new()
    }

    fn extract_topic_context(&self, text: &str, topic: &str) -> String {
        if let Some(pos) = text.find(topic) {
            let start = pos.saturating_sub(50);
            let end = (pos + topic.len() + 50).min(text.len());
            text[start..end].to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn detect_conversation_phase(&self, content: &str) -> PhaseType {
        if self.contains_patterns(content, &self.patterns.planning_indicators) {
            PhaseType::Planning
        } else if self.contains_patterns(content, &self.patterns.implementation_indicators) {
            PhaseType::Implementation
        } else if self.contains_patterns(content, &self.patterns.debugging_indicators) {
            PhaseType::Debugging
        } else if self.contains_patterns(content, &self.patterns.review_indicators) {
            PhaseType::Review
        } else if self.contains_patterns(content, &self.patterns.learning_indicators) {
            PhaseType::Learning
        } else {
            PhaseType::Planning // Default
        }
    }

    fn calculate_block_complexity(&self, block: &Block) -> f64 {
        let mut complexity = 0.0;
        
        // Base complexity from content length
        complexity += (block.content.word_count as f64).ln() * 0.1;
        
        // Code blocks increase complexity
        complexity += block.content.code_blocks.len() as f64 * 0.5;
        
        // Tool usage increases complexity
        complexity += block.tools.len() as f64 * 0.3;
        
        // Links and references increase complexity
        complexity += block.content.links.len() as f64 * 0.2;
        
        complexity.min(10.0) // Cap at 10.0
    }

    fn calculate_phase_duration(&self, session: &Session, start: usize, end: usize) -> Duration {
        if start >= session.blocks.len() || end > session.blocks.len() || start >= end {
            return Duration::zero();
        }
        
        let start_time = session.blocks[start].timestamp;
        let end_time = if end == session.blocks.len() {
            session.blocks.last().unwrap().timestamp
        } else {
            session.blocks[end - 1].timestamp
        };
        
        end_time - start_time
    }

    fn extract_primary_activity(&self, session: &Session, start: usize, end: usize) -> String {
        if start >= session.blocks.len() || end > session.blocks.len() {
            return "unknown".to_string();
        }

        let mut activity_indicators = HashMap::new();
        
        for i in start..end.min(session.blocks.len()) {
            let content = &session.blocks[i].content.raw_text.to_lowercase();
            
            for pattern in &self.patterns.implementation_indicators {
                if content.contains(pattern) {
                    *activity_indicators.entry("implementation").or_insert(0) += 1;
                }
            }
            
            for pattern in &self.patterns.debugging_indicators {
                if content.contains(pattern) {
                    *activity_indicators.entry("debugging").or_insert(0) += 1;
                }
            }
        }

        activity_indicators
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(activity, _)| activity.to_string())
            .unwrap_or_else(|| "general discussion".to_string())
    }

    fn identify_transition_trigger(&self, session: &Session, block_index: usize) -> String {
        if block_index >= session.blocks.len() {
            return "unknown".to_string();
        }

        let block = &session.blocks[block_index];
        let content = &block.content.raw_text.to_lowercase();

        if self.contains_patterns(content, &self.patterns.problem_indicators) {
            "problem encountered".to_string()
        } else if self.contains_patterns(content, &self.patterns.solution_indicators) {
            "solution provided".to_string()
        } else if content.contains("let's") || content.contains("now") {
            "natural progression".to_string()
        } else {
            "topic shift".to_string()
        }
    }

    fn contains_patterns(&self, text: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|pattern| text.contains(pattern))
    }

    fn extract_skill_area(&self, _content: &str) -> String {
        // Placeholder for skill area extraction
        "programming".to_string()
    }

    fn extract_learned_concepts(&self, _content: &str) -> Vec<String> {
        // Placeholder for concept extraction
        Vec::new()
    }

    fn extract_skill_area_from_solution(&self, _content: &str) -> String {
        // Placeholder for skill area extraction from solutions
        "problem solving".to_string()
    }

    fn extract_solution_concepts(&self, _content: &str) -> Vec<String> {
        // Placeholder for solution concept extraction
        Vec::new()
    }

    fn assess_complexity_level(&self, concepts: &[String]) -> ComplexityLevel {
        // Simple heuristic based on number of concepts
        match concepts.len() {
            0..=2 => ComplexityLevel::Beginner,
            3..=5 => ComplexityLevel::Intermediate,
            6..=8 => ComplexityLevel::Advanced,
            _ => ComplexityLevel::Expert,
        }
    }

    fn calculate_confidence_gain(&self, concepts: &[String]) -> f64 {
        // Simple confidence calculation based on concept count
        (concepts.len() as f64 * 0.2).min(1.0)
    }

    fn has_practical_application(&self, session: &Session, skill_area: &str) -> bool {
        // Check if there's evidence of practical application
        session.blocks.iter().any(|block| {
            block.content.code_blocks.len() > 0 || 
            block.tools.len() > 0 ||
            block.content.raw_text.contains("implement") ||
            block.content.raw_text.contains("create")
        })
    }

    fn extract_problem_key(&self, _content: &str) -> String {
        // Placeholder for problem key extraction
        "general_problem".to_string()
    }

    fn extract_solution_key(&self, _content: &str) -> String {
        // Placeholder for solution key extraction
        "general_problem".to_string()
    }

    fn calculate_code_quality_score(&self, session: &Session) -> f64 {
        let code_blocks: Vec<_> = session.blocks.iter()
            .flat_map(|block| &block.content.code_blocks)
            .collect();

        if code_blocks.is_empty() {
            return 0.0;
        }

        let mut total_score = 0.0;
        for code_block in &code_blocks {
            let mut block_score = 5.0; // Base score
            
            // Bonus for having comments
            if code_block.content.contains("//") || code_block.content.contains("#") {
                block_score += 1.0;
            }
            
            // Bonus for proper structure (basic heuristic)
            if code_block.content.contains("function") || code_block.content.contains("fn") {
                block_score += 1.0;
            }
            
            total_score += block_score;
        }

        (total_score / code_blocks.len() as f64).min(10.0)
    }

    fn calculate_efficiency_rating(&self, _session: &Session, resolution_times: &[Duration]) -> f64 {
        if resolution_times.is_empty() {
            return 5.0; // Default rating
        }

        let avg_resolution_minutes = resolution_times.iter()
            .map(|d| d.num_minutes() as f64)
            .sum::<f64>() / resolution_times.len() as f64;

        // Higher efficiency for faster resolution (inverted scale)
        (60.0 / avg_resolution_minutes.max(1.0)).min(10.0)
    }

    fn calculate_collaboration_effectiveness(&self, session: &Session) -> f64 {
        let user_blocks = session.blocks.iter().filter(|b| b.role == Role::User).count();
        let assistant_blocks = session.blocks.iter().filter(|b| b.role == Role::Assistant).count();
        
        if user_blocks == 0 || assistant_blocks == 0 {
            return 0.0;
        }

        // Good collaboration has balanced interaction
        let balance_ratio = (user_blocks as f64 / assistant_blocks as f64).min(assistant_blocks as f64 / user_blocks as f64);
        balance_ratio * 10.0
    }

    fn determine_interaction_style(&self, session: &Session, question_types: &HashMap<String, usize>) -> InteractionStyle {
        let total_questions: usize = question_types.values().sum();
        let code_blocks_count: usize = session.blocks.iter()
            .map(|b| b.content.code_blocks.len())
            .sum();

        if code_blocks_count > total_questions * 2 {
            InteractionStyle::TaskOriented
        } else if question_types.contains_key("how do") || question_types.contains_key("what is") {
            InteractionStyle::Learning
        } else if question_types.contains_key("why does") {
            InteractionStyle::Analytical
        } else if session.blocks.iter().any(|b| b.content.raw_text.contains("creative") || b.content.raw_text.contains("design")) {
            InteractionStyle::Creative
        } else {
            InteractionStyle::Exploratory
        }
    }

    fn assess_feedback_quality(&self, session: &Session) -> f64 {
        let assistant_blocks: Vec<_> = session.blocks.iter()
            .filter(|b| b.role == Role::Assistant)
            .collect();

        if assistant_blocks.is_empty() {
            return 0.0;
        }

        let mut quality_score = 0.0;
        for block in &assistant_blocks {
            let content = &block.content.raw_text;
            
            // Quality indicators
            if content.contains("example") || content.contains("for instance") {
                quality_score += 1.0;
            }
            if content.contains("explanation") || content.contains("because") {
                quality_score += 1.0;
            }
            if block.content.code_blocks.len() > 0 {
                quality_score += 2.0;
            }
            if block.content.links.len() > 0 {
                quality_score += 1.0;
            }
        }

        (quality_score / assistant_blocks.len() as f64).min(10.0)
    }

    fn calculate_knowledge_transfer(&self, session: &Session) -> f64 {
        // Simple metric based on progression from questions to understanding
        let user_blocks = session.blocks.iter().filter(|b| b.role == Role::User);
        let question_count = user_blocks.filter(|b| b.content.raw_text.contains('?')).count();
        
        let total_user_blocks = session.blocks.iter().filter(|b| b.role == Role::User).count();
        
        if total_user_blocks == 0 {
            return 0.0;
        }

        let question_ratio = question_count as f64 / total_user_blocks as f64;
        // Lower question ratio at the end suggests better knowledge transfer
        (1.0 - question_ratio) * 10.0
    }

    fn extract_primary_topic(&self, content: &str) -> String {
        // Extract the most prominent topic from content
        let topics = self.extract_technical_topics_from_text(content);
        topics.into_iter().next().unwrap_or_else(|| "general".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_session() -> Session {
        let mut session = Session::new();
        
        // Add a user question
        session.add_block(Block {
            id: Uuid::new_v4(),
            sequence_number: 1,
            role: Role::User,
            timestamp: Utc::now(),
            content: BlockContent {
                raw_text: "How do I implement async functions in Rust?".to_string(),
                formatted_text: None,
                tokens: Vec::new(),
                code_blocks: Vec::new(),
                links: Vec::new(),
                mentions: Vec::new(),
                word_count: 8,
                character_count: 42,
            },
            metadata: BlockMetadata {
                processing_time_ms: None,
                confidence_score: None,
                complexity_score: None,
                sentiment: None,
                topics: Vec::new(),
                intent: Some(ConversationIntent::Question),
                parent_block_id: None,
                thread_id: None,
            },
            tools: Vec::new(),
            attachments: Vec::new(),
            context_references: Vec::new(),
        });

        // Add assistant response with code
        session.add_block(Block {
            id: Uuid::new_v4(),
            sequence_number: 2,
            role: Role::Assistant,
            timestamp: Utc::now(),
            content: BlockContent {
                raw_text: "Here's how you can implement async functions in Rust:\n```rust\nasync fn example() {\n    println!(\"Hello async!\");\n}\n```".to_string(),
                formatted_text: None,
                tokens: Vec::new(),
                code_blocks: vec![CodeBlock {
                    language: Some(ProgrammingLanguage::Rust),
                    content: "async fn example() {\n    println!(\"Hello async!\");\n}".to_string(),
                    line_numbers: false,
                    filename: None,
                    start_position: 60,
                    end_position: 120,
                }],
                links: Vec::new(),
                mentions: Vec::new(),
                word_count: 15,
                character_count: 120,
            },
            metadata: BlockMetadata {
                processing_time_ms: Some(1500),
                confidence_score: Some(0.9),
                complexity_score: Some(6.5),
                sentiment: None,
                topics: vec!["rust".to_string(), "async".to_string()],
                intent: Some(ConversationIntent::Explanation),
                parent_block_id: None,
                thread_id: None,
            },
            tools: Vec::new(),
            attachments: Vec::new(),
            context_references: Vec::new(),
        });

        session
    }

    #[tokio::test]
    async fn test_extract_primary_topics() {
        let analyzer = InsightsAnalyzer::new();
        let session = create_test_session();

        let topics = analyzer.extract_primary_topics(&session).await.unwrap();
        assert!(!topics.is_empty());
        assert!(topics.iter().any(|t| t.name.contains("rust")));
    }

    #[tokio::test]
    async fn test_analyze_conversation_flow() {
        let analyzer = InsightsAnalyzer::new();
        let session = create_test_session();

        let flow = analyzer.analyze_conversation_flow(&session).await.unwrap();
        assert!(!flow.phases.is_empty());
        assert!(!flow.complexity_evolution.is_empty());
    }

    #[tokio::test]
    async fn test_calculate_productivity_metrics() {
        let analyzer = InsightsAnalyzer::new();
        let session = create_test_session();

        let metrics = analyzer.calculate_productivity_metrics(&session).await.unwrap();
        assert!(metrics.code_quality_score > 0.0);
        assert!(metrics.efficiency_rating > 0.0);
    }

    #[test]
    fn test_detect_conversation_phase() {
        let analyzer = InsightsAnalyzer::new();
        
        assert_eq!(analyzer.detect_conversation_phase("let's plan the architecture"), PhaseType::Planning);
        assert_eq!(analyzer.detect_conversation_phase("implement the function"), PhaseType::Implementation);
        assert_eq!(analyzer.detect_conversation_phase("debug this error"), PhaseType::Debugging);
        assert_eq!(analyzer.detect_conversation_phase("let's review and verify"), PhaseType::Review);
    }
}