//! Core data models for Claude session parsing with comprehensive type safety

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use indexmap::IndexMap;

/// Main session container with complete metadata and conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub metadata: SessionMetadata,
    pub blocks: Vec<Block>,
    pub insights: SessionInsights,
    pub statistics: SessionStatistics,
    pub tool_usage: ToolUsageStats,
    pub working_context: WorkingContext,
}

/// Comprehensive session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub file_path: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub file_size_bytes: u64,
    pub line_count: usize,
    pub claude_version: Option<String>,
    pub client_info: Option<ClientInfo>,
    pub session_duration: Option<chrono::Duration>,
    pub conversation_id: Option<String>,
    pub project_context: Option<ProjectContext>,
}

/// Client application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub user_agent: Option<String>,
}

/// Project context extracted from working directory and file paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub working_directory: Option<String>,
    pub project_name: Option<String>,
    pub project_type: Option<ProjectType>,
    pub language_stack: Vec<ProgrammingLanguage>,
    pub frameworks: Vec<String>,
    pub repository_url: Option<String>,
    pub git_branch: Option<String>,
}

/// Supported project types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectType {
    WebApp,
    MobileApp,
    Library,
    CLI,
    DataScience,
    MachineLearning,
    Documentation,
    Configuration,
    Unknown,
}

/// Programming languages detected in the session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProgrammingLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    Go,
    Cpp,
    C,
    Swift,
    Kotlin,
    Ruby,
    PHP,
    Dart,
    Shell,
    SQL,
    HTML,
    CSS,
    Markdown,
    JSON,
    YAML,
    TOML,
    Unknown(String),
}

/// Individual conversation block with rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub sequence_number: usize,
    pub role: Role,
    pub timestamp: DateTime<Utc>,
    pub content: BlockContent,
    pub metadata: BlockMetadata,
    pub tools: Vec<ToolInvocation>,
    pub attachments: Vec<Attachment>,
    pub context_references: Vec<ContextReference>,
}

/// Message roles in Claude conversations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

impl Role {
    pub fn from_string(s: &str) -> crate::Result<Self> {
        match s.to_lowercase().as_str() {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "system" => Ok(Self::System),
            "tool" => Ok(Self::Tool),
            _ => Err(crate::error::ClaudeSessionError::unknown_role(s)),
        }
    }
}

/// Block content with structured data and formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockContent {
    pub raw_text: String,
    pub formatted_text: Option<String>,
    pub tokens: Vec<ContentToken>,
    pub code_blocks: Vec<CodeBlock>,
    pub links: Vec<Link>,
    pub mentions: Vec<Mention>,
    pub word_count: usize,
    pub character_count: usize,
}

/// Tokenized content for search indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentToken {
    pub text: String,
    pub token_type: TokenType,
    pub position: usize,
    pub length: usize,
}

/// Token types for semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenType {
    Word,
    Number,
    Punctuation,
    Code,
    FilePath,
    URL,
    Command,
    Variable,
    Function,
    Class,
    Method,
    Keyword,
    String,
    Comment,
}

/// Code blocks extracted from messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: Option<ProgrammingLanguage>,
    pub content: String,
    pub line_numbers: bool,
    pub filename: Option<String>,
    pub start_position: usize,
    pub end_position: usize,
}

/// URLs and links found in content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub title: Option<String>,
    pub link_type: LinkType,
}

/// Types of links found in content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LinkType {
    External,
    Documentation,
    Repository,
    File,
    Internal,
}

/// Mentions of files, functions, or other entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub text: String,
    pub mention_type: MentionType,
    pub context: Option<String>,
}

/// Types of mentions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MentionType {
    File,
    Function,
    Class,
    Variable,
    Command,
    Person,
    Project,
    Library,
    Tool,
}

/// Block-level metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub processing_time_ms: Option<u64>,
    pub confidence_score: Option<f64>,
    pub complexity_score: Option<f64>,
    pub sentiment: Option<Sentiment>,
    pub topics: Vec<String>,
    pub intent: Option<ConversationIntent>,
    pub parent_block_id: Option<Uuid>,
    pub thread_id: Option<String>,
}

/// Sentiment analysis results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

/// Conversation intent classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConversationIntent {
    Question,
    Request,
    Explanation,
    Debugging,
    CodeReview,
    Planning,
    Learning,
    Troubleshooting,
    Documentation,
    Implementation,
}

/// Tool invocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub result: Option<ToolResult>,
    pub timestamp: DateTime<Utc>,
    pub execution_time_ms: Option<u64>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: String,
    pub result_type: ToolResultType,
    pub metadata: Option<serde_json::Value>,
    pub files_affected: Vec<String>,
}

/// Types of tool results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolResultType {
    Success,
    Error,
    Warning,
    Information,
    FileContent,
    CommandOutput,
    SearchResults,
}

/// File attachments in conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub file_type: AttachmentType,
    pub size_bytes: Option<u64>,
    pub content: Option<String>,
    pub hash: Option<String>,
}

/// Types of attachments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttachmentType {
    Image,
    Document,
    Code,
    Data,
    Archive,
    Other,
}

/// Context references between blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextReference {
    pub reference_type: ReferenceType,
    pub target_block_id: Uuid,
    pub relevance_score: f64,
    pub description: Option<String>,
}

/// Types of context references
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferenceType {
    Continuation,
    Response,
    Clarification,
    Example,
    Alternative,
    Correction,
}

/// Working context during the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingContext {
    pub files_mentioned: IndexMap<String, FileContext>,
    pub commands_run: Vec<CommandExecution>,
    pub directories_accessed: Vec<String>,
    pub tools_used: Vec<String>,
    pub error_patterns: Vec<ErrorPattern>,
    pub solution_patterns: Vec<SolutionPattern>,
}

/// Context about files mentioned in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub path: String,
    pub file_type: Option<ProgrammingLanguage>,
    pub mentions: usize,
    pub operations: Vec<FileOperation>,
    pub last_accessed: DateTime<Utc>,
}

/// File operations performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileOperation {
    Read,
    Write,
    Create,
    Delete,
    Move,
    Copy,
    Edit,
    Search,
}

/// Command execution records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub command: String,
    pub working_directory: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Error patterns identified in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub error_type: String,
    pub pattern: String,
    pub occurrences: usize,
    pub resolution_attempts: Vec<String>,
    pub resolved: bool,
}

/// Solution patterns identified in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionPattern {
    pub problem_type: String,
    pub solution_approach: String,
    pub tools_used: Vec<String>,
    pub success_rate: f64,
    pub context: String,
}

/// Session-level insights derived from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInsights {
    pub primary_topics: Vec<Topic>,
    pub conversation_flow: ConversationFlow,
    pub learning_outcomes: Vec<LearningOutcome>,
    pub productivity_metrics: ProductivityMetrics,
    pub collaboration_patterns: CollaborationPatterns,
}

/// Topic analysis with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub relevance_score: f64,
    pub mentions: usize,
    pub subtopics: Vec<String>,
    pub related_tools: Vec<String>,
}

/// Conversation flow analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationFlow {
    pub phases: Vec<ConversationPhase>,
    pub transitions: Vec<PhaseTransition>,
    pub complexity_evolution: Vec<f64>,
    pub focus_shifts: usize,
}

/// Conversation phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationPhase {
    pub phase_type: PhaseType,
    pub start_block: usize,
    pub end_block: usize,
    pub duration: chrono::Duration,
    pub primary_activity: String,
}

/// Types of conversation phases
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PhaseType {
    Planning,
    Implementation,
    Debugging,
    Testing,
    Documentation,
    Learning,
    Review,
}

/// Phase transitions in conversation flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub from_phase: PhaseType,
    pub to_phase: PhaseType,
    pub trigger: String,
    pub block_number: usize,
}

/// Learning outcomes from the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningOutcome {
    pub skill_area: String,
    pub concepts_learned: Vec<String>,
    pub complexity_level: ComplexityLevel,
    pub confidence_gain: f64,
    pub practical_application: bool,
}

/// Complexity levels for learning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Productivity metrics for the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub tasks_completed: usize,
    pub problems_solved: usize,
    pub code_quality_score: f64,
    pub efficiency_rating: f64,
    pub collaboration_effectiveness: f64,
    pub time_to_resolution: Vec<chrono::Duration>,
}

/// Collaboration patterns analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationPatterns {
    pub interaction_style: InteractionStyle,
    pub question_types: HashMap<String, usize>,
    pub feedback_quality: f64,
    pub iterative_cycles: usize,
    pub knowledge_transfer: f64,
}

/// Interaction styles with Claude
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum InteractionStyle {
    Exploratory,
    TaskOriented,
    Learning,
    Debugging,
    Creative,
    Analytical,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionStatistics {
    pub total_blocks: usize,
    pub user_blocks: usize,
    pub assistant_blocks: usize,
    pub tool_blocks: usize,
    pub total_words: usize,
    pub total_characters: usize,
    pub code_blocks: usize,
    pub files_referenced: usize,
    pub commands_executed: usize,
    pub errors_encountered: usize,
    pub session_duration: Option<chrono::Duration>,
    pub average_response_time: Option<chrono::Duration>,
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolUsageStats {
    pub tools_by_frequency: IndexMap<String, usize>,
    pub total_tool_calls: usize,
    pub successful_calls: usize,
    pub failed_calls: usize,
    pub average_execution_time: f64,
    pub most_used_tool: Option<String>,
    pub tool_efficiency: HashMap<String, f64>,
}

impl Session {
    /// Create a new empty session
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: SessionMetadata {
                file_path: String::new(),
                created_at: Utc::now(),
                last_modified: Utc::now(),
                file_size_bytes: 0,
                line_count: 0,
                claude_version: None,
                client_info: None,
                session_duration: None,
                conversation_id: None,
                project_context: None,
            },
            blocks: Vec::new(),
            insights: SessionInsights {
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
            },
            statistics: SessionStatistics::default(),
            tool_usage: ToolUsageStats::default(),
            working_context: WorkingContext {
                files_mentioned: IndexMap::new(),
                commands_run: Vec::new(),
                directories_accessed: Vec::new(),
                tools_used: Vec::new(),
                error_patterns: Vec::new(),
                solution_patterns: Vec::new(),
            },
        }
    }

    /// Add a block to the session
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
        self.update_statistics();
    }

    /// Update session statistics based on current blocks
    fn update_statistics(&mut self) {
        self.statistics.total_blocks = self.blocks.len();
        self.statistics.user_blocks = self.blocks.iter().filter(|b| b.role == Role::User).count();
        self.statistics.assistant_blocks = self.blocks.iter().filter(|b| b.role == Role::Assistant).count();
        self.statistics.tool_blocks = self.blocks.iter().filter(|b| b.role == Role::Tool).count();
        
        self.statistics.total_words = self.blocks.iter()
            .map(|b| b.content.word_count)
            .sum();
        
        self.statistics.total_characters = self.blocks.iter()
            .map(|b| b.content.character_count)
            .sum();
        
        self.statistics.code_blocks = self.blocks.iter()
            .map(|b| b.content.code_blocks.len())
            .sum();
    }

    /// Get blocks by role
    pub fn blocks_by_role(&self, role: Role) -> Vec<&Block> {
        self.blocks.iter().filter(|b| b.role == role).collect()
    }

    /// Get session duration
    pub fn duration(&self) -> Option<chrono::Duration> {
        if self.blocks.is_empty() {
            return None;
        }
        
        let first_timestamp = self.blocks.first()?.timestamp;
        let last_timestamp = self.blocks.last()?.timestamp;
        
        Some(last_timestamp - first_timestamp)
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}